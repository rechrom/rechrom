//! Environment proxy routing without native TLS or proxy libraries.
use hyper::Uri;
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::client::legacy::connect::proxy::{SocksV4, SocksV5, Tunnel};
use hyper_util::client::legacy::connect::{Connected, Connection, HttpConnector};
use hyper_util::client::proxy::matcher::Matcher;
use hyper_util::rt::TokioIo;
use std::error::Error;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsConnector;
use tower_service::Service;

type BoxError = Box<dyn Error + Send + Sync>;

trait Stream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Stream for T {}

pub(crate) struct ProxyStream {
    inner: TokioIo<Box<dyn Stream>>,
    proxied: bool,
    h2: bool,
}

impl ProxyStream {
    fn new(stream: impl Stream + 'static, proxied: bool, h2: bool) -> Self {
        Self {
            inner: TokioIo::new(Box::new(stream)),
            proxied,
            h2,
        }
    }
}

impl Connection for ProxyStream {
    fn connected(&self) -> Connected {
        let connected = Connected::new().proxy(self.proxied);
        if self.h2 {
            connected.negotiated_h2()
        } else {
            connected
        }
    }
}

impl hyper::rt::Read for ProxyStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        hyper::rt::Read::poll_read(Pin::new(&mut self.inner), cx, buf)
    }
}

impl hyper::rt::Write for ProxyStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        hyper::rt::Write::poll_write(Pin::new(&mut self.inner), cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        hyper::rt::Write::poll_flush(Pin::new(&mut self.inner), cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        hyper::rt::Write::poll_shutdown(Pin::new(&mut self.inner), cx)
    }
}

#[derive(Clone)]
pub(crate) struct EnvironmentConnector {
    direct: HttpsConnector<HttpConnector>,
    proxy_http: HttpsConnector<HttpConnector>,
    tls: Arc<rustls::ClientConfig>,
    matcher: Arc<Matcher>,
}

impl EnvironmentConnector {
    pub(crate) fn new(tls: rustls::ClientConfig, matcher: Arc<Matcher>) -> Self {
        let mut http = HttpConnector::new();
        http.enforce_http(false);
        let direct = HttpsConnectorBuilder::new()
            .with_tls_config(tls.clone())
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .wrap_connector(http.clone());
        // CONNECT is HTTP/1.1, even when the connection to the proxy uses TLS.
        let proxy_http = HttpsConnectorBuilder::new()
            .with_tls_config(tls.clone())
            .https_or_http()
            .enable_http1()
            .wrap_connector(http);
        let mut tls = tls;
        tls.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Self {
            direct,
            proxy_http,
            tls: Arc::new(tls),
            matcher,
        }
    }

    async fn secure(
        &self,
        stream: impl Stream + 'static,
        dst: &Uri,
    ) -> Result<ProxyStream, BoxError> {
        if dst.scheme_str() != Some("https") {
            return Ok(ProxyStream::new(stream, false, false));
        }
        let host = dst
            .host()
            .ok_or_else(|| io::Error::other("missing TLS hostname"))?;
        let host = host.trim_start_matches('[').trim_end_matches(']');
        let name = rustls::pki_types::ServerName::try_from(host.to_owned())?;
        let stream = TlsConnector::from(self.tls.clone())
            .connect(name, stream)
            .await?;
        let h2 = stream.get_ref().1.alpn_protocol() == Some(b"h2");
        Ok(ProxyStream::new(stream, false, h2))
    }
}

impl Service<Uri> for EnvironmentConnector {
    type Response = ProxyStream;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<ProxyStream, BoxError>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), BoxError>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let mut connector = self.clone();
        Box::pin(async move {
            let Some(proxy) = connector.matcher.intercept(&dst) else {
                let stream = connector.direct.call(dst).await?;
                let h2 = stream.connected().is_negotiated_h2();
                return Ok(ProxyStream::new(TokioIo::new(stream), false, h2));
            };
            let proxy_uri = proxy.uri().clone();
            match proxy_uri.scheme_str().unwrap_or("http") {
                "http" | "https" if dst.scheme_str() == Some("http") => {
                    let stream = connector.proxy_http.call(proxy_uri).await?;
                    // Tell hyper to use absolute-form request targets for the proxy.
                    Ok(ProxyStream::new(TokioIo::new(stream), true, false))
                }
                "http" | "https" => {
                    let mut tunnel = Tunnel::new(proxy_uri, connector.proxy_http.clone());
                    if let Some(auth) = proxy.basic_auth() {
                        tunnel = tunnel.with_auth(auth.clone());
                    }
                    let stream = tunnel.call(dst.clone()).await?;
                    connector.secure(TokioIo::new(stream), &dst).await
                }
                scheme @ ("socks4" | "socks4a" | "socks5" | "socks5h") => {
                    // hyper-util's SOCKS helpers default to 443. Supply the
                    // origin port explicitly for HTTP destinations as well.
                    let mut socks_dst = dst.clone();
                    if dst.port_u16().is_none() {
                        let mut parts = dst.clone().into_parts();
                        let host = dst
                            .host()
                            .ok_or_else(|| io::Error::other("missing origin hostname"))?;
                        let port = if dst.scheme_str() == Some("https") {
                            443
                        } else {
                            80
                        };
                        parts.authority = Some(format!("{host}:{port}").parse()?);
                        socks_dst = Uri::from_parts(parts)?;
                    }
                    let mut parts = proxy_uri.clone().into_parts();
                    parts.scheme = Some(hyper::http::uri::Scheme::HTTP);
                    if proxy_uri.port_u16().is_none() {
                        let host = proxy_uri
                            .host()
                            .ok_or_else(|| io::Error::other("missing proxy hostname"))?;
                        parts.authority = Some(format!("{host}:1080").parse()?);
                    }
                    let proxy_uri = Uri::from_parts(parts)?;
                    let mut tcp = HttpConnector::new();
                    tcp.enforce_http(false);
                    let stream = if scheme.starts_with("socks4") {
                        SocksV4::new(proxy_uri, tcp)
                            .local_dns(scheme == "socks4")
                            .call(socks_dst)
                            .await?
                    } else {
                        let mut socks = SocksV5::new(proxy_uri, tcp).local_dns(scheme == "socks5");
                        if let Some((user, pass)) = proxy.raw_auth() {
                            socks = socks.with_auth(user.into(), pass.into());
                        }
                        socks.call(socks_dst).await?
                    };
                    connector.secure(TokioIo::new(stream), &dst).await
                }
                scheme => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unsupported proxy scheme: {scheme}"),
                )
                .into()),
            }
        })
    }
}

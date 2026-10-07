#![allow(non_snake_case)]

use crate::http_transport_options::HTTPTransportOptions;
use crate::proxy_connector::EnvironmentConnector;
use base64::Engine;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::{self, HeaderMap, HeaderName, HeaderValue};
use hyper::{Method, Request, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::client::proxy::matcher::Matcher;
use hyper_util::rt::{TokioExecutor, TokioTimer};
use std::error::Error;
use std::future::Future;
use std::io::{self, Read};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Notify, Semaphore};
use tower_service::Service;
use url::Url;

type BoxError = Box<dyn Error + Send + Sync>;
struct HTTPClient {
    inner: Client<ConnectTimeout<EnvironmentConnector>, Full<Bytes>>,
    proxy: Arc<Matcher>,
}

impl HTTPClient {
    async fn request(
        &self,
        mut request: Request<Full<Bytes>>,
    ) -> Result<hyper::Response<hyper::body::Incoming>, hyper_util::client::legacy::Error> {
        // Proxy credentials belong on plain HTTP proxy requests or CONNECT,
        // never on the request sent inside a TLS tunnel to the origin.
        let auth = self
            .proxy
            .intercept(request.uri())
            .filter(|proxy| {
                request.uri().scheme_str() == Some("http")
                    && matches!(proxy.uri().scheme_str(), Some("http" | "https"))
            })
            .and_then(|proxy| proxy.basic_auth().cloned());
        request.headers_mut().remove(header::PROXY_AUTHORIZATION);
        if let Some(auth) = auth {
            request
                .headers_mut()
                .insert(header::PROXY_AUTHORIZATION, auth);
        }
        self.inner.request(request).await
    }
}

// cpp: http_transport/http_request.h:12-18
#[derive(Clone, Debug, Default)]
pub struct HTTPRequestData {
    pub method: String,
    pub url: String,
    pub referrer: String,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

// cpp: http_transport/http_request.h:20-25
#[derive(Clone, Debug, Default)]
pub struct HTTPResponseData {
    pub status: i64,
    pub final_url: String,
    pub mime_type: String,
    pub body: Vec<u8>,
}

// cpp: http_transport/http_request.h:27-34
pub trait HTTPRequestOperation {
    fn Poll(&mut self) -> io::Result<Option<HTTPResponseData>>;
}

// A shared executor drives connections as well as requests. A runtime per
// request would prevent idle connections from surviving for pool reuse.
pub(crate) fn runtime() -> io::Result<&'static Runtime> {
    static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();
    match RUNTIME.get_or_init(|| {
        Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("layoutng-http")
            .enable_all()
            .build()
            .map_err(|error| error.to_string())
    }) {
        Ok(runtime) => Ok(runtime),
        Err(error) => Err(io::Error::other(error.clone())),
    }
}

// Bound DNS, TCP and the TLS handshake, rather than timing out a server that
// has already connected but has not sent its response headers yet.
#[derive(Clone)]
struct ConnectTimeout<C> {
    inner: C,
    timeout: Option<Duration>,
}

impl<C> Service<Uri> for ConnectTimeout<C>
where
    C: Service<Uri> + Send,
    C::Future: Send + 'static,
    C::Response: Send + 'static,
    C::Error: Into<BoxError>,
{
    type Response = C::Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, BoxError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let future = self.inner.call(uri);
        let timeout = self.timeout;
        Box::pin(async move {
            match timeout {
                Some(timeout) => tokio::time::timeout(timeout, future)
                    .await
                    .map_err(|_| {
                        Box::new(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "HTTP connect timeout",
                        )) as BoxError
                    })?
                    .map_err(Into::into),
                None => future.await.map_err(Into::into),
            }
        })
    }
}

pub(crate) struct Scheduler {
    pub(crate) options: HTTPTransportOptions,
    pub(crate) slots: Arc<Semaphore>,
    client: HTTPClient,
}

impl Scheduler {
    fn new(options: HTTPTransportOptions) -> io::Result<Self> {
        if options.max_parallel_requests == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "HTTP parallel request limit must be positive",
            ));
        }
        let milliseconds = u64::try_from(options.connect_timeout_ms).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "negative HTTP connect timeout")
        })?;
        let roots =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let timeout = (milliseconds > 0).then(|| Duration::from_millis(milliseconds));
        let client = build_client(roots, timeout)?;
        Ok(Self {
            slots: Arc::new(Semaphore::new(options.max_parallel_requests)),
            options,
            client,
        })
    }
}

fn build_client(roots: rustls::RootCertStore, timeout: Option<Duration>) -> io::Result<HTTPClient> {
    build_client_with_proxy(roots, timeout, Matcher::from_env())
}

fn build_client_with_proxy(
    roots: rustls::RootCertStore,
    timeout: Option<Duration>,
    proxy: Matcher,
) -> io::Result<HTTPClient> {
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(rustls_rustcrypto::provider()))
        .with_safe_default_protocol_versions()
        .map_err(io::Error::other)?
        .with_root_certificates(roots)
        .with_no_client_auth();
    let proxy = Arc::new(proxy);
    let connector = ConnectTimeout {
        inner: EnvironmentConnector::new(tls, proxy.clone()),
        timeout,
    };
    let _enter = runtime()?.enter();
    let client = Client::builder(TokioExecutor::new())
        .pool_timer(TokioTimer::new())
        .timer(TokioTimer::new())
        .build(connector);
    Ok(HTTPClient {
        inner: client,
        proxy,
    })
}

#[derive(Debug)]
pub(crate) struct TransferError {
    pub(crate) message: String,
    pub(crate) retryable: bool,
}

impl TransferError {
    pub(crate) fn new(message: impl Into<String>, retryable: bool) -> Self {
        Self {
            message: message.into(),
            retryable,
        }
    }
}

pub(crate) fn protocol_error(error: impl std::fmt::Display) -> TransferError {
    TransferError::new(error.to_string(), false)
}

pub(crate) fn network_error(error: &(dyn Error + 'static)) -> TransferError {
    let mut current = Some(error);
    let mut message = Vec::new();
    let mut retryable = false;
    let mut invalid_tls = false;
    while let Some(error) = current {
        message.push(error.to_string());
        if error.is::<rustls::Error>() {
            invalid_tls = true;
        }
        if let Some(error) = error.downcast_ref::<io::Error>() {
            invalid_tls |= error
                .get_ref()
                .is_some_and(|inner| inner.is::<rustls::Error>());
            retryable |= matches!(
                error.kind(),
                io::ErrorKind::TimedOut
                    | io::ErrorKind::ConnectionRefused
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::ConnectionAborted
                    | io::ErrorKind::UnexpectedEof
                    | io::ErrorKind::BrokenPipe
                    | io::ErrorKind::NotFound
            );
        }
        if let Some(error) = error.downcast_ref::<hyper::Error>() {
            retryable |= error.is_incomplete_message() || error.is_closed();
        }
        if let Some(error) = error.downcast_ref::<hyper_util::client::legacy::Error>() {
            retryable |= error.is_connect();
        }
        // io::Error::source() skips the boxed inner error itself. Inspect it
        // explicitly so nested connector IO wrappers retain TLS error types.
        current = error
            .downcast_ref::<io::Error>()
            .and_then(|io| io.get_ref().map(|inner| inner as &(dyn Error + 'static)))
            .or_else(|| error.source());
    }
    TransferError::new(message.join(": "), retryable && !invalid_tls)
}

pub(crate) async fn with_stall_timeout<F, T>(
    future: F,
    options: &HTTPTransportOptions,
) -> Result<T, TransferError>
where
    F: Future<Output = Result<T, TransferError>>,
{
    if options.stall_timeout_ms > 0 {
        tokio::time::timeout(
            Duration::from_millis(options.stall_timeout_ms as u64),
            future,
        )
        .await
        .map_err(|_| TransferError::new("transfer made no byte progress", true))?
    } else {
        future.await
    }
}

fn http_url(value: &str) -> Result<Url, TransferError> {
    let url = Url::parse(value).map_err(protocol_error)?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(protocol_error("expected HTTP(S) URL"));
    }
    Ok(url)
}

fn build_headers(
    request: &HTTPRequestData,
    options: &HTTPTransportOptions,
) -> Result<HeaderMap, TransferError> {
    let mut headers = HeaderMap::new();
    for (name, value) in &request.headers {
        headers.append(
            HeaderName::from_bytes(name.as_bytes()).map_err(protocol_error)?,
            HeaderValue::from_str(value).map_err(protocol_error)?,
        );
    }
    for (name, value) in [
        (header::USER_AGENT, &options.user_agent),
        (header::ACCEPT_LANGUAGE, &options.accept_language),
    ] {
        if !headers.contains_key(&name) {
            headers.insert(name, HeaderValue::from_str(value).map_err(protocol_error)?);
        }
    }
    if !headers.contains_key(header::ACCEPT_ENCODING) {
        headers.insert(
            header::ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );
    }
    if !headers.contains_key(header::ACCEPT) {
        headers.insert(header::ACCEPT, HeaderValue::from_static("*/*"));
    }
    if !request.referrer.is_empty() && !headers.contains_key(header::REFERER) {
        headers.insert(
            header::REFERER,
            HeaderValue::from_str(&request.referrer).map_err(protocol_error)?,
        );
    }
    Ok(headers)
}

fn read_limited(reader: impl Read, limit: usize) -> Result<Vec<u8>, TransferError> {
    let mut body = Vec::new();
    reader
        .take((limit as u64).saturating_add(1))
        .read_to_end(&mut body)
        .map_err(protocol_error)?;
    if body.len() > limit {
        return Err(protocol_error("resource exceeds configured size limit"));
    }
    Ok(body)
}

fn decode_body(mut body: Vec<u8>, encodings: &str, limit: usize) -> Result<Vec<u8>, TransferError> {
    for encoding in encodings
        .split(',')
        .rev()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        body = match encoding.to_ascii_lowercase().as_str() {
            "identity" => body,
            "gzip" | "x-gzip" => {
                read_limited(flate2::read::MultiGzDecoder::new(body.as_slice()), limit)?
            }
            "deflate" => {
                // Some HTTP servers send raw DEFLATE despite the zlib framing
                // required for Content-Encoding: deflate.
                let zlib = body.len() >= 2
                    && body[0] & 15 == 8
                    && u16::from_be_bytes([body[0], body[1]]) % 31 == 0;
                if zlib {
                    read_limited(flate2::read::ZlibDecoder::new(body.as_slice()), limit)?
                } else {
                    read_limited(flate2::read::DeflateDecoder::new(body.as_slice()), limit)?
                }
            }
            "br" => read_limited(brotli::Decompressor::new(body.as_slice(), 4096), limit)?,
            other => {
                return Err(protocol_error(format!(
                    "unsupported HTTP content encoding: {other}"
                )))
            }
        };
    }
    Ok(body)
}

pub(crate) async fn fetch_response(
    request: &HTTPRequestData,
    scheduler: &Scheduler,
) -> Result<ResponseStart, TransferError> {
    let options = &scheduler.options;
    let mut url = http_url(&request.url)?;
    let method = Method::from_bytes(request.method.as_bytes()).map_err(protocol_error)?;
    let mut body = request.body.clone();
    let mut headers = build_headers(request, options)?;
    let mut implicit_content_type =
        (!body.is_empty() || method == Method::POST) && !headers.contains_key(header::CONTENT_TYPE);
    if implicit_content_type {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
    }
    for redirects in 0..=10 {
        // Fragments and URL userinfo are not part of the HTTP request target.
        let mut target = url.clone();
        target.set_fragment(None);
        if !url.username().is_empty() && !headers.contains_key(header::AUTHORIZATION) {
            let credentials = format!("{}:{}", url.username(), url.password().unwrap_or_default());
            let decoded = percent_encoding::percent_decode_str(&credentials).collect::<Vec<u8>>();
            let value = format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode(decoded)
            );
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&value).map_err(protocol_error)?,
            );
        }
        target
            .set_username("")
            .map_err(|_| protocol_error("invalid URL username"))?;
        target
            .set_password(None)
            .map_err(|_| protocol_error("invalid URL password"))?;
        let mut outgoing = Request::builder()
            .method(method.clone())
            .uri(target.as_str())
            .body(Full::new(Bytes::copy_from_slice(&body)))
            .map_err(protocol_error)?;
        *outgoing.headers_mut() = headers.clone();
        if options.trace_requests {
            eprintln!("HTTP {} {}", method, url);
        }
        let response = with_stall_timeout(
            async {
                scheduler
                    .client
                    .request(outgoing)
                    .await
                    .map_err(|error| network_error(&error))
            },
            options,
        )
        .await?;
        if options.trace_requests {
            eprintln!(
                "HTTP headers {:?} {} {}",
                response.version(),
                response.status(),
                url
            );
        }
        let status = response.status();
        if matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308) {
            if let Some(location) = response.headers().get(header::LOCATION) {
                if redirects == 10 {
                    return Err(protocol_error("HTTP redirect limit exceeded"));
                }
                let next = url
                    .join(location.to_str().map_err(protocol_error)?)
                    .map_err(protocol_error)?;
                http_url(next.as_str())?;
                if next.origin() != url.origin() {
                    headers.remove(header::AUTHORIZATION);
                    headers.remove(header::COOKIE);
                }
                headers.remove(header::HOST);
                // Match libcurl's custom_request semantics: it retains the
                // explicitly chosen method, but stops POST body replay for
                // 301/302/303. 307/308 preserve both method and body.
                if status.as_u16() == 303 && method != Method::HEAD
                    || matches!(status.as_u16(), 301 | 302)
                        && (method == Method::POST || !body.is_empty())
                {
                    body.clear();
                    headers.remove(header::CONTENT_LENGTH);
                    headers.remove(header::TRANSFER_ENCODING);
                    if implicit_content_type {
                        headers.remove(header::CONTENT_TYPE);
                        implicit_content_type = false;
                    }
                }
                url = next;
                drop(response);
                continue;
            }
        }
        let mime_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let encodings = response
            .headers()
            .get(header::CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        return Ok(ResponseStart {
            incoming: response.into_body(),
            status: i64::from(status.as_u16()),
            final_url: url.to_string(),
            mime_type,
            encodings,
            head_only: method == Method::HEAD,
        });
    }
    unreachable!("redirect limit checked above")
}

pub(crate) struct ResponseStart {
    pub(crate) incoming: hyper::body::Incoming,
    pub(crate) status: i64,
    pub(crate) final_url: String,
    pub(crate) mime_type: String,
    pub(crate) encodings: String,
    pub(crate) head_only: bool,
}

async fn transfer(
    request: &HTTPRequestData,
    scheduler: &Scheduler,
) -> Result<HTTPResponseData, TransferError> {
    let ResponseStart {
        mut incoming,
        status,
        final_url,
        mime_type,
        encodings,
        head_only,
    } = fetch_response(request, scheduler).await?;
    let options = &scheduler.options;
    let mut received = Vec::new();
    while let Some(frame) = with_stall_timeout(
        async {
            incoming
                .frame()
                .await
                .transpose()
                .map_err(|error| network_error(&error))
        },
        options,
    )
    .await?
    {
        if let Ok(chunk) = frame.into_data() {
            if chunk.len() > options.max_response_bytes.saturating_sub(received.len()) {
                return Err(protocol_error("resource exceeds configured size limit"));
            }
            received.extend_from_slice(&chunk);
        }
    }
    let body = if head_only || received.is_empty() {
        received
    } else {
        decode_body(received, &encodings, options.max_response_bytes)?
    };
    Ok(HTTPResponseData {
        status,
        final_url,
        mime_type,
        body,
    })
}

async fn execute(request: HTTPRequestData, state: Arc<State>) {
    let Ok(_slot) = state.scheduler.slots.clone().acquire_owned().await else {
        return;
    };
    let options = &state.scheduler.options;
    for attempt in 0.. {
        let future = transfer(&request, &state.scheduler);
        let result = if options.timeout_ms > 0 {
            tokio::time::timeout(Duration::from_millis(options.timeout_ms as u64), future)
                .await
                .unwrap_or_else(|_| Err(TransferError::new("HTTP request timeout", true)))
        } else {
            future.await
        };
        match result {
            Ok(response) => {
                state.result.lock().expect("HTTP state poisoned").response = Some(response);
                return;
            }
            Err(error) => {
                if state.cancelled.load(Ordering::Relaxed) {
                    return;
                }
                let idempotent = request.method == "GET" || request.method == "HEAD";
                if idempotent && error.retryable && attempt < options.max_retries {
                    if options.trace_requests {
                        eprintln!("HTTP retry {}", request.url);
                    }
                    continue;
                }
                state.result.lock().expect("HTTP state poisoned").error =
                    Some(format!("{}: {}", request.url, error.message));
                return;
            }
        }
    }
}

struct State {
    cancelled: AtomicBool,
    cancel: Notify,
    result: Mutex<WorkerResult>,
    scheduler: Arc<Scheduler>,
}

#[derive(Default)]
struct WorkerResult {
    response: Option<HTTPResponseData>,
    error: Option<String>,
}

struct HTTPRequest {
    state: Arc<State>,
}

impl HTTPRequest {
    fn new(request: HTTPRequestData, scheduler: Arc<Scheduler>) -> io::Result<Self> {
        let state = Arc::new(State {
            cancelled: AtomicBool::new(false),
            cancel: Notify::new(),
            result: Mutex::new(WorkerResult::default()),
            scheduler,
        });
        let worker_state = state.clone();
        runtime()?.spawn(async move {
            tokio::select! {
                biased;
                _ = worker_state.cancel.notified() => {},
                _ = execute(request, worker_state.clone()) => {},
            }
        });
        Ok(Self { state })
    }
}

impl Drop for HTTPRequest {
    fn drop(&mut self) {
        self.state.cancelled.store(true, Ordering::Relaxed);
        self.state.cancel.notify_one();
    }
}

impl HTTPRequestOperation for HTTPRequest {
    fn Poll(&mut self) -> io::Result<Option<HTTPResponseData>> {
        let mut result = self.state.result.lock().expect("HTTP state poisoned");
        if let Some(error) = &result.error {
            return Err(io::Error::other(error.clone()));
        }
        Ok(result.response.take())
    }
}

// cpp: http_transport/http_request.h:36-48
pub struct HTTPRequestTransport {
    pub(crate) scheduler: Arc<Scheduler>,
}

impl HTTPRequestTransport {
    pub fn new(options: HTTPTransportOptions) -> io::Result<Self> {
        Ok(Self {
            scheduler: Arc::new(Scheduler::new(options)?),
        })
    }

    pub fn Start(
        &mut self,
        request: &HTTPRequestData,
    ) -> io::Result<Box<dyn HTTPRequestOperation>> {
        Ok(Box::new(HTTPRequest::new(
            request.clone(),
            self.scheduler.clone(),
        )?))
    }
}

pub fn StartHTTPRequest(
    request: &HTTPRequestData,
    defaults: &HTTPTransportOptions,
) -> io::Result<Box<dyn HTTPRequestOperation>> {
    let scheduler = Arc::new(Scheduler::new(defaults.clone())?);
    Ok(Box::new(HTTPRequest::new(request.clone(), scheduler)?))
}

#[cfg(test)]
mod tls_tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_rustls::TlsAcceptor;

    // Public, locally generated fixture key; never used outside these tests.
    fn server() -> (String, tokio::task::JoinHandle<bool>) {
        let certificate = rustls::pki_types::CertificateDer::from(
            include_bytes!("../tests/fixtures/server.der").to_vec(),
        );
        let key = rustls::pki_types::PrivatePkcs8KeyDer::from(
            include_bytes!("../tests/fixtures/server-key.der").to_vec(),
        );
        let config =
            rustls::ServerConfig::builder_with_provider(Arc::new(rustls_rustcrypto::provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(vec![certificate], key.into())
                .unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("https://{}/tls", listener.local_addr().unwrap());
        let handle = runtime().unwrap().spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let (stream, _) = tokio::time::timeout(Duration::from_secs(3), listener.accept())
                .await
                .unwrap()
                .unwrap();
            let Ok(mut stream) = TlsAcceptor::from(Arc::new(config)).accept(stream).await else {
                return false;
            };
            let mut headers = Vec::new();
            while !headers.ends_with(b"\r\n\r\n") {
                let byte = stream.read_u8().await.unwrap();
                headers.push(byte);
            }
            assert!(headers.starts_with(b"GET /tls HTTP/1.1\r\n"));
            assert!(!String::from_utf8_lossy(&headers)
                .to_ascii_lowercase()
                .contains("proxy-authorization:"));
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nsecure",
                )
                .await
                .unwrap();
            stream.shutdown().await.unwrap();
            true
        });
        (url, handle)
    }

    fn fixture_roots() -> rustls::RootCertStore {
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls::pki_types::CertificateDer::from(
                include_bytes!("../tests/fixtures/ca.der").to_vec(),
            ))
            .unwrap();
        roots
    }

    fn scheduler_with_proxy(proxy: Matcher) -> Scheduler {
        Scheduler {
            options: HTTPTransportOptions {
                max_retries: 0,
                ..Default::default()
            },
            slots: Arc::new(Semaphore::new(1)),
            client: build_client_with_proxy(fixture_roots(), Some(Duration::from_secs(2)), proxy)
                .unwrap(),
        }
    }

    async fn read_headers(stream: &mut (impl tokio::io::AsyncRead + Unpin)) -> String {
        let mut bytes = Vec::new();
        while !bytes.ends_with(b"\r\n\r\n") {
            bytes.push(stream.read_u8().await.unwrap());
        }
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn http_proxy_uses_absolute_target_and_no_proxy_bypasses_it() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let proxy = format!("http://user:pass@{}", listener.local_addr().unwrap());
        let task = runtime().unwrap().spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();
            let headers = read_headers(&mut stream).await;
            assert!(headers.starts_with("GET http://origin.invalid/path?q=1 HTTP/1.1\r\n"));
            assert!(headers
                .to_ascii_lowercase()
                .contains("proxy-authorization: basic dxnlcjpwyxnz\r\n"));
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nproxy",
                )
                .await
                .unwrap();
        });
        let scheduler = scheduler_with_proxy(Matcher::builder().all(&proxy).build());
        let result = runtime()
            .unwrap()
            .block_on(transfer(
                &HTTPRequestData {
                    method: "GET".into(),
                    url: "http://origin.invalid/path?q=1".into(),
                    ..Default::default()
                },
                &scheduler,
            ))
            .unwrap();
        assert_eq!(result.body, b"proxy");
        runtime().unwrap().block_on(task).unwrap();
        let (url, task) = server();
        let scheduler = scheduler_with_proxy(
            Matcher::builder()
                .all("http://127.0.0.1:1")
                .no("127.0.0.0/8")
                .build(),
        );
        assert_eq!(
            runtime()
                .unwrap()
                .block_on(transfer(
                    &HTTPRequestData {
                        method: "GET".into(),
                        url,
                        ..Default::default()
                    },
                    &scheduler
                ))
                .unwrap()
                .body,
            b"secure"
        );
        assert!(runtime().unwrap().block_on(task).unwrap());
    }

    #[test]
    fn https_connect_proxy_keeps_credentials_out_of_origin_request() {
        let (url, origin) = server();
        let destination: Uri = url.parse().unwrap();
        let address = format!(
            "{}:{}",
            destination.host().unwrap(),
            destination.port_u16().unwrap()
        );
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let proxy = format!("http://user:pass@{}", listener.local_addr().unwrap());
        let task = runtime().unwrap().spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();
            let headers = read_headers(&mut stream).await;
            assert!(headers.starts_with(&format!("CONNECT {address} HTTP/1.1\r\n")));
            assert!(headers
                .to_ascii_lowercase()
                .contains("proxy-authorization: basic dxnlcjpwyxnz\r\n"));
            let mut upstream = tokio::net::TcpStream::connect(address).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
            let _ = tokio::io::copy_bidirectional(&mut stream, &mut upstream).await;
        });
        let scheduler = scheduler_with_proxy(Matcher::builder().all(&proxy).build());
        let result = runtime()
            .unwrap()
            .block_on(transfer(
                &HTTPRequestData {
                    method: "GET".into(),
                    url,
                    ..Default::default()
                },
                &scheduler,
            ))
            .unwrap();
        assert_eq!(result.body, b"secure");
        assert!(runtime().unwrap().block_on(origin).unwrap());
        runtime().unwrap().block_on(task).unwrap();
    }

    #[test]
    fn socks5h_proxy_resolves_origin_remotely_and_authenticates() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let proxy = format!("socks5h://user:pass@{}", listener.local_addr().unwrap());
        let task = runtime().unwrap().spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut greeting = [0; 3];
            stream.read_exact(&mut greeting).await.unwrap();
            assert_eq!(greeting, [5, 1, 2]);
            stream.write_all(&[5, 2]).await.unwrap();
            let mut auth = [0; 11];
            stream.read_exact(&mut auth).await.unwrap();
            assert_eq!(&auth, b"\x01\x04user\x04pass");
            stream.write_all(&[1, 0]).await.unwrap();
            let mut head = [0; 5];
            stream.read_exact(&mut head).await.unwrap();
            assert_eq!(&head[..4], &[5, 1, 0, 3]);
            let mut host = vec![0; usize::from(head[4])];
            stream.read_exact(&mut host).await.unwrap();
            assert_eq!(host, b"origin.invalid");
            assert_eq!(stream.read_u16().await.unwrap(), 80);
            stream
                .write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 80])
                .await
                .unwrap();
            let headers = read_headers(&mut stream).await;
            assert!(headers.starts_with("GET /path HTTP/1.1\r\n"));
            assert!(!headers
                .to_ascii_lowercase()
                .contains("proxy-authorization:"));
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nsocks",
                )
                .await
                .unwrap();
        });
        let scheduler = scheduler_with_proxy(Matcher::builder().all(&proxy).build());
        let result = runtime()
            .unwrap()
            .block_on(transfer(
                &HTTPRequestData {
                    method: "GET".into(),
                    url: "http://origin.invalid/path".into(),
                    ..Default::default()
                },
                &scheduler,
            ))
            .unwrap();
        assert_eq!(result.body, b"socks");
        runtime().unwrap().block_on(task).unwrap();
    }

    #[test]
    fn http2_tls_negotiation_multiplexes_requests_on_one_connection() {
        let cert = rustls::pki_types::CertificateDer::from(
            include_bytes!("../tests/fixtures/server.der").to_vec(),
        );
        let key = rustls::pki_types::PrivatePkcs8KeyDer::from(
            include_bytes!("../tests/fixtures/server-key.der").to_vec(),
        );
        let mut config =
            rustls::ServerConfig::builder_with_provider(Arc::new(rustls_rustcrypto::provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(vec![cert], key.into())
                .unwrap();
        config.alpn_protocols = vec![b"h2".to_vec()];
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("https://{}/h2", listener.local_addr().unwrap());
        let task = runtime().unwrap().spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            let stream = TlsAcceptor::from(Arc::new(config))
                .accept(stream)
                .await
                .unwrap();
            assert_eq!(stream.get_ref().1.alpn_protocol(), Some(b"h2".as_slice()));
            let mut connection = h2::server::handshake(stream).await.unwrap();
            let mut responders = Vec::new();
            for _ in 0..2 {
                let (request, responder) = connection.accept().await.unwrap().unwrap();
                assert_eq!(request.uri().path(), "/h2");
                responders.push(responder);
            }
            for mut responder in responders {
                let response = hyper::Response::builder().status(200).body(()).unwrap();
                responder
                    .send_response(response, false)
                    .unwrap()
                    .send_data(Bytes::from_static(b"http2"), true)
                    .unwrap();
            }
            // Drive the connection until the client has consumed both responses.
            tokio::time::timeout(Duration::from_secs(1), async {
                while connection.accept().await.is_some() {}
            })
            .await
            .ok();
        });
        let scheduler = scheduler_with_proxy(Matcher::builder().build());
        let request = HTTPRequestData {
            method: "GET".into(),
            url,
            ..Default::default()
        };
        let (first, second) = runtime().unwrap().block_on(async {
            tokio::join!(
                transfer(&request, &scheduler),
                transfer(&request, &scheduler)
            )
        });
        assert_eq!(first.unwrap().body, b"http2");
        assert_eq!(second.unwrap().body, b"http2");
        runtime().unwrap().block_on(task).unwrap();
    }

    #[test]
    fn connect_timeout_bounds_proxy_tls_handshake() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let proxy = format!("http://{}", listener.local_addr().unwrap());
        let task = runtime().unwrap().spawn(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();
            read_headers(&mut stream).await;
            stream
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await
                .unwrap();
            // A proxy accepts CONNECT but never completes the origin TLS handshake.
            tokio::time::sleep(Duration::from_millis(200)).await;
        });
        let mut scheduler = scheduler_with_proxy(Matcher::builder().all(&proxy).build());
        scheduler.client = build_client_with_proxy(
            fixture_roots(),
            Some(Duration::from_millis(60)),
            Matcher::builder().all(&proxy).build(),
        )
        .unwrap();
        let error = runtime()
            .unwrap()
            .block_on(transfer(
                &HTTPRequestData {
                    method: "GET".into(),
                    url: "https://origin.invalid/".into(),
                    ..Default::default()
                },
                &scheduler,
            ))
            .unwrap_err();
        assert!(
            error.message.contains("HTTP connect timeout"),
            "{}",
            error.message
        );
        runtime().unwrap().block_on(task).unwrap();
    }

    #[test]
    fn rustcrypto_https_accepts_trusted_certificate_and_rejects_untrusted_certificate() {
        let (url, server_task) = server();
        let mut roots = rustls::RootCertStore::empty();
        roots
            .add(rustls::pki_types::CertificateDer::from(
                include_bytes!("../tests/fixtures/ca.der").to_vec(),
            ))
            .unwrap();
        let scheduler = Scheduler {
            options: HTTPTransportOptions {
                max_retries: 0,
                ..Default::default()
            },
            slots: Arc::new(Semaphore::new(1)),
            client: build_client(roots, Some(Duration::from_secs(2))).unwrap(),
        };
        let result = runtime()
            .unwrap()
            .block_on(transfer(
                &HTTPRequestData {
                    method: "GET".into(),
                    url,
                    ..Default::default()
                },
                &scheduler,
            ))
            .unwrap();
        assert_eq!(result.body, b"secure");
        assert!(runtime().unwrap().block_on(server_task).unwrap());
        let (url, server_task) = server();
        let scheduler = Scheduler::new(HTTPTransportOptions {
            max_retries: 0,
            ..Default::default()
        })
        .unwrap();
        let error = runtime()
            .unwrap()
            .block_on(transfer(
                &HTTPRequestData {
                    method: "GET".into(),
                    url,
                    ..Default::default()
                },
                &scheduler,
            ))
            .unwrap_err();
        assert!(
            error.message.contains("UnknownIssuer") || error.message.contains("unknown issuer"),
            "{}",
            error.message
        );
        assert!(!error.retryable);
        assert!(!runtime().unwrap().block_on(server_task).unwrap());
    }
}

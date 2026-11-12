#![allow(non_snake_case)]

use crate::http_transport_options::HTTPTransportOptions;
use crate::proxy_connector::EnvironmentConnector;
use crate::request_trace::RequestTrace;
use base64::Engine;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::{self, HeaderMap, HeaderName, HeaderValue};
use hyper::{Method, Request, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::client::proxy::matcher::Matcher;
use hyper_util::rt::{TokioExecutor, TokioTimer};
use rustls::pki_types::{pem::PemObject, CertificateDer};
use std::error::Error;
use std::future::Future;
use std::io::{self, Read};
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};
use tower_service::Service;
use url::Url;

use crate::HTTPWakeCallback;

type BoxError = Box<dyn Error + Send + Sync>;

// Chromium: net/http/http_network_session.h:73. Hyper's legacy client
// advertises a 16 KiB HTTP/2 header-list limit by default, which rejects
// legitimate responses with large policy/reporting headers before their
// response head can be published (notably YouTube). Match Chromium's network
// session limit instead of treating those headers as a protocol error.
const kSpdyMaxHeaderListSize: u32 = 256 * 1024;

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
    /// None preserves an existing caller-prepared referrer. Document resource
    /// owners always supply Some(document policy), including explicit omission.
    pub referrer_policy: Option<crate::ReferrerPolicy>,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
    pub priority: HTTPRequestPriority,
    pub fetch: Option<crate::NetworkFetchOptions>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum HTTPRequestPriority {
    #[default]
    Normal = 0,
    High = 1,
    /// Chromium's `ResourceLoadPriority::kVeryHigh`, mapped to the network
    /// service's HIGHEST priority for parser/render-blocking resources.
    VeryHigh = 2,
}

// cpp: http_transport/http_request.h:20-25
#[derive(Clone, Debug, Default)]
pub struct HTTPResponseData {
    pub send_start: Option<Instant>,
    pub response_end: Option<Instant>,
    pub expected_content_length: Option<u64>,
    pub headers: Vec<(String, String)>,
    pub status: i64,
    pub final_url: String,
    pub mime_type: String,
    pub content_dpr: Option<f64>,
    pub content_disposition: String,
    pub content_language: String,
    /// Final request referrer after all transport redirects.
    pub request_referrer: Option<String>,
    pub referrer_policy: String,
    pub content_security_policy: Vec<String>,
    pub content_security_policy_report_only: Vec<String>,
    pub response_address_space: crate::IPAddressSpace,
    pub permissions_policy: String,
    pub feature_policy: String,
    pub document_policy: String,
    pub document_policy_report_only: String,
    pub require_document_policy: String,
    pub body: Vec<u8>,
}

// cpp: http_transport/http_request.h:27-34
pub trait HTTPRequestOperation {
    /// Main-thread receipt of response headers, independently of body EOF.
    fn HasResponseStarted(&self) -> Option<bool> {
        None
    }
    /// Explicit cancellation differs from disconnecting a keepalive receiver.
    fn CancelKeepalive(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "HTTP keepalive cancellation producer missing",
        ))
    }
    fn DetachForKeepalive(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "HTTP operation has no native keepalive owner",
        ))
    }

    fn SetPriority(&mut self, _priority: HTTPRequestPriority) {}
    fn Poll(&mut self) -> io::Result<Option<HTTPResponseData>>;
    fn SetWakeCallback(&mut self, _callback: crate::HTTPWakeCallback) {}
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
    priority_slots: Option<Arc<Semaphore>>,
    critical_slots: Option<Arc<Semaphore>>,
    client: HTTPClient,
    /// Hyper does not expose Chromium-style HTTP/2 stream priorities. Keep
    /// visible/deferred and parser/render-blocking traffic on distinct
    /// connection pools so a large ordinary script/image multiplex cannot
    /// consume all body bandwidth after a higher-priority stream's response
    /// headers arrive.
    priority_clients: Vec<HTTPClient>,
    critical_clients: Vec<HTTPClient>,
    priority_client_lanes: Arc<Mutex<Vec<usize>>>,
    critical_client_lanes: Arc<Mutex<Vec<usize>>>,
}

struct ClientLane {
    index: usize,
    available: Arc<Mutex<Vec<usize>>>,
}

impl Drop for ClientLane {
    fn drop(&mut self) {
        self.available
            .lock()
            .expect("HTTP client lane pool poisoned")
            .push(self.index);
    }
}

enum ScheduledClient {
    Ordinary,
    Priority(ClientLane),
    Critical(ClientLane),
}

pub(crate) struct ScheduledSlot {
    _permit: OwnedSemaphorePermit,
    client: ScheduledClient,
}

impl ScheduledSlot {
    fn ordinary(permit: OwnedSemaphorePermit) -> Self {
        Self {
            _permit: permit,
            client: ScheduledClient::Ordinary,
        }
    }
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
        let certificate_file = std::env::var_os("SSL_CERT_FILE");
        let roots = root_certificates(certificate_file.as_deref().map(Path::new))?;
        let timeout = (milliseconds > 0).then(|| Duration::from_millis(milliseconds));
        // Source: third_party/blink/renderer/platform/loader/fetch/
        // resource_load_scheduler.cc; services/network/public/cpp/
        // resource_request.cc.  Parser/render-blocking CSS is VeryHigh while
        // module scripts remain High.  Keep a distinct admission class for
        // each so a large module graph cannot occupy every slot required to
        // reach first paint.  The partitions still sum to the embedder's
        // configured maximum; this does not manufacture extra concurrency.
        let critical = options.max_parallel_requests / 4;
        let reserved = options.max_parallel_requests / 4;
        let ordinary = options.max_parallel_requests - critical - reserved;
        let client = build_client(roots.clone(), timeout)?;
        let mut priority_clients = Vec::with_capacity(reserved);
        for _ in 0..reserved {
            priority_clients.push(build_client(roots.clone(), timeout)?);
        }
        let mut critical_clients = Vec::with_capacity(critical);
        for _ in 0..critical {
            critical_clients.push(build_client(roots.clone(), timeout)?);
        }
        Ok(Self {
            slots: Arc::new(Semaphore::new(ordinary.max(1))),
            priority_slots: (reserved != 0).then(|| Arc::new(Semaphore::new(reserved))),
            critical_slots: (critical != 0).then(|| Arc::new(Semaphore::new(critical))),
            options,
            client,
            priority_clients,
            critical_clients,
            priority_client_lanes: Arc::new(Mutex::new((0..reserved).rev().collect())),
            critical_client_lanes: Arc::new(Mutex::new((0..critical).rev().collect())),
        })
    }

    pub(crate) async fn acquire_slot(
        &self,
        priority: HTTPRequestPriority,
    ) -> Result<ScheduledSlot, tokio::sync::AcquireError> {
        if priority == HTTPRequestPriority::VeryHigh {
            if let Some(critical) = &self.critical_slots {
                return tokio::select! {
                    biased;
                    permit = critical.clone().acquire_owned() => permit.map(|permit| self.critical_slot(permit)),
                    permit = self.acquire_high_or_ordinary_slot() => permit,
                };
            }
        }
        if priority >= HTTPRequestPriority::High {
            return self.acquire_high_or_ordinary_slot().await;
        }
        self.slots
            .clone()
            .acquire_owned()
            .await
            .map(ScheduledSlot::ordinary)
    }

    async fn acquire_high_or_ordinary_slot(
        &self,
    ) -> Result<ScheduledSlot, tokio::sync::AcquireError> {
        if let Some(high) = &self.priority_slots {
            return tokio::select! {
                biased;
                permit = high.clone().acquire_owned() => permit.map(|permit| self.priority_slot(permit)),
                permit = self.slots.clone().acquire_owned() => permit.map(ScheduledSlot::ordinary),
            };
        }
        self.slots
            .clone()
            .acquire_owned()
            .await
            .map(ScheduledSlot::ordinary)
    }

    fn priority_slot(&self, permit: OwnedSemaphorePermit) -> ScheduledSlot {
        ScheduledSlot {
            _permit: permit,
            client: ScheduledClient::Priority(ClientLane {
                index: self
                    .priority_client_lanes
                    .lock()
                    .expect("HTTP priority client lane pool poisoned")
                    .pop()
                    .expect("HTTP priority permit must own a client lane"),
                available: Arc::clone(&self.priority_client_lanes),
            }),
        }
    }

    fn critical_slot(&self, permit: OwnedSemaphorePermit) -> ScheduledSlot {
        ScheduledSlot {
            _permit: permit,
            client: ScheduledClient::Critical(ClientLane {
                index: self
                    .critical_client_lanes
                    .lock()
                    .expect("HTTP critical client lane pool poisoned")
                    .pop()
                    .expect("HTTP critical permit must own a client lane"),
                available: Arc::clone(&self.critical_client_lanes),
            }),
        }
    }

    fn client_for_slot(&self, slot: &ScheduledSlot) -> &HTTPClient {
        match &slot.client {
            ScheduledClient::Ordinary => &self.client,
            ScheduledClient::Priority(lane) => &self.priority_clients[lane.index],
            ScheduledClient::Critical(lane) => &self.critical_clients[lane.index],
        }
    }
}

fn root_certificates(certificate_file: Option<&Path>) -> io::Result<rustls::RootCertStore> {
    let mut roots =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    if let Some(path) = certificate_file {
        // Honor the host's explicit CA configuration without modifying its trust
        // store or relaxing certificate/hostname verification.
        let certificates = CertificateDer::pem_file_iter(path).map_err(|error| {
            io::Error::other(format!("SSL_CERT_FILE could not be read: {error}"))
        })?;
        let mut count = 0;
        for certificate in certificates {
            let certificate = certificate.map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("SSL_CERT_FILE contains invalid PEM: {error}"),
                )
            })?;
            roots.add(certificate).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("SSL_CERT_FILE contains an invalid certificate: {error}"),
                )
            })?;
            count += 1;
        }
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SSL_CERT_FILE contains no certificates",
            ));
        }
    }
    Ok(roots)
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
        // Chromium grows HTTP/2 receive windows with observed bandwidth and
        // consumption. Hyper's fixed defaults can starve later multiplexed
        // streams after headers when a page starts many large assets at once.
        .http2_adaptive_window(true)
        .http2_max_header_list_size(kSpdyMaxHeaderListSize)
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
    let mut blocked_by_policy = false;
    while let Some(error) = current {
        message.push(error.to_string());
        if error.is::<rustls::Error>() {
            invalid_tls = true;
        }
        if let Some(error) = error.downcast_ref::<io::Error>() {
            blocked_by_policy |= matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            );
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
    TransferError::new(
        message.join(": "),
        retryable && !invalid_tls && !blocked_by_policy,
    )
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
                )));
            }
        };
    }
    Ok(body)
}

pub(crate) async fn fetch_response(
    request: &HTTPRequestData,
    scheduler: &Scheduler,
    slot: &ScheduledSlot,
    trace: RequestTrace,
    attempt: usize,
) -> Result<ResponseStart, TransferError> {
    let options = &scheduler.options;
    let mut url = http_url(&request.url)?;
    let mut method = Method::from_bytes(request.method.as_bytes()).map_err(protocol_error)?;
    let mut body = request.body.clone();
    let mut headers = build_headers(request, options)?;
    let mut referrer_policy = request
        .referrer_policy
        .unwrap_or(crate::ReferrerPolicy::UnsafeUrl);
    let mut referrer = headers
        .get(header::REFERER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    referrer = referrer_policy.GenerateReferrer(&referrer, url.as_str());
    let implicit_content_type =
        (!body.is_empty() || method == Method::POST) && !headers.contains_key(header::CONTENT_TYPE);
    if implicit_content_type {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
    }
    let lna_checker = Arc::new(Mutex::new(fetch_policy::RawLocalNetworkAccessChecker::new()));
    let lna_pending =
        Arc::new(crate::local_network_access::LocalNetworkPermissionPending::default());
    let mut tainted_origin = false;
    for redirects in 0..=10 {
        if let Some(fetch) = &request.fetch {
            // FetchContext::CanRequest and KeepAliveURLLoader's retained
            // PolicyContainer check each redirect without a DOM callback.
            use foundation::content_security_policy::{
                CSPCheckHeaderType, CSPRedirectStatus, CSPReportingDisposition, CSPRequest,
            };
            if redirects != 0 {
                lna_checker
                    .lock()
                    .expect("LNA checker poisoned")
                    .ResetForRedirect();
                let prepared = fetch.policy.PrepareRequestURL(url.as_str());
                url = http_url(&prepared.url)?;
                if fetch.upgrade_if_insecure {
                    if let Some(upgraded) =
                        fetch_policy::UpgradeInsecureURL(fetch.policy.as_ref(), url.as_str())
                    {
                        url = http_url(&upgraded)?;
                    }
                }
                let verdict = fetch.policy.AllowRequest(&CSPRequest {
                    url: url.as_str(),
                    url_before_redirects: &request.url,
                    context: fetch.context,
                    world: if fetch.keepalive {
                        foundation::content_security_policy::CSPWorld::Main
                    } else {
                        fetch.world
                    },
                    redirect: if redirects > 1 {
                        CSPRedirectStatus::FollowedRedirect
                    } else {
                        CSPRedirectStatus::NoRedirect
                    },
                    reporting: if fetch.keepalive {
                        CSPReportingDisposition::SuppressReporting
                    } else {
                        CSPReportingDisposition::Report
                    },
                    check_header_type: CSPCheckHeaderType::All,
                });
                let has_violations = !verdict.violations.is_empty();
                fetch
                    .policy_violations
                    .lock()
                    .expect("CSP effect queue poisoned")
                    .extend(verdict.violations);
                if has_violations {
                    if let Some(wake) = &fetch.policy_effect_wake {
                        wake();
                    }
                }
                if !verdict.allowed {
                    fetch.access_check_failed.store(true, Ordering::Release);
                    return Err(protocol_error(
                        "redirect blocked by Content Security Policy",
                    ));
                }
                if !fetch.context_detached.load(Ordering::Acquire) {
                    let mixed = if fetch.keepalive {
                        fetch_policy::CheckRawKeepaliveRedirectMixedContent(
                            fetch.security.as_ref(),
                            fetch.policy.as_ref(),
                            url.as_str(),
                            &request.url,
                            fetch.context,
                            prepared.block_all_mixed_content,
                        )
                    } else {
                        fetch_policy::CheckRawMixedContent(fetch.security.as_ref(),fetch.policy.as_ref(),url.as_str(),&request.url,fetch.context,foundation::content_security_policy::CSPRedirectStatus::FollowedRedirect,prepared.block_all_mixed_content)
                    };
                    if let Some(effect) = mixed.effect {
                        fetch
                            .mixed_content_effects
                            .lock()
                            .expect("mixed content effect queue poisoned")
                            .push(effect);
                        if let Some(wake) = &fetch.policy_effect_wake {
                            wake();
                        }
                    }
                    if let Some(reason) = mixed.blocked {
                        fetch.access_check_failed.store(true, Ordering::Release);
                        return Err(protocol_error(format!(
                            "redirect mixed-content admission: {reason:?}"
                        )));
                    }
                }
            }
            if fetch.mode == crate::FetchMode::Cors {
                let origin = if tainted_origin {
                    "null".to_owned()
                } else {
                    fetch.requestor_origin.Serialize()
                };
                headers.insert(
                    header::ORIGIN,
                    HeaderValue::from_str(&origin)
                        .map_err(|error| protocol_error(error.to_string()))?,
                );
            }
            if fetch.mode == crate::FetchMode::SameOrigin
                && (tainted_origin || !fetch.requestor_origin.IsSameOrigin(url.as_str()))
            {
                return Err(protocol_error(
                    "same-origin Fetch mode rejects cross-origin URL",
                ));
            }
            if fetch.credentials == crate::FetchCredentialsMode::Omit
                || (fetch.credentials == crate::FetchCredentialsMode::SameOrigin
                    && (tainted_origin || !fetch.requestor_origin.IsSameOrigin(url.as_str())))
            {
                headers.remove(header::AUTHORIZATION);
                headers.remove(header::COOKIE);
            }
            if method != Method::GET && method != Method::HEAD
                || headers.contains_key(header::ORIGIN)
            {
                headers.insert(
                    header::ORIGIN,
                    HeaderValue::from_str(&if tainted_origin {
                        "null".into()
                    } else {
                        fetch.requestor_origin.Serialize()
                    })
                    .map_err(protocol_error)?,
                );
            }
        }
        headers.remove(header::REFERER);
        if !referrer.is_empty() {
            headers.insert(
                header::REFERER,
                HeaderValue::from_str(&referrer).map_err(protocol_error)?,
            );
        }
        // Fragments and URL userinfo are not part of the HTTP request target.
        let mut target = url.clone();
        target.set_fragment(None);
        let credentials_allowed = request.fetch.as_ref().is_none_or(|fetch| {
            fetch.credentials == crate::FetchCredentialsMode::Include
                || (fetch.credentials == crate::FetchCredentialsMode::SameOrigin
                    && !tainted_origin
                    && fetch.requestor_origin.IsSameOrigin(url.as_str()))
        });
        if credentials_allowed
            && !url.username().is_empty()
            && !headers.contains_key(header::AUTHORIZATION)
        {
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
        if let Some(fetch) = &request.fetch {
            outgoing
                .extensions_mut()
                .insert(hyper_util::client::legacy::connect::CheckConnection(
                    Arc::new(crate::local_network_access::LocalNetworkAccessGate {
                        fetch: fetch.clone(),
                        url: url.to_string(),
                        checker: lna_checker.clone(),
                        pending: lna_pending.clone(),
                        trace,
                    }),
                ));
        }
        if options.trace_requests {
            eprintln!("HTTP {} {}", method, url);
        }
        trace.event(
            "HTTPRequestStart",
            &[("attempt", attempt as f64), ("redirect", redirects as f64)],
        );
        let send_start = hyper_util::client::legacy::connect::CaptureSendStart::default();
        outgoing.extensions_mut().insert(send_start.clone());
        let client = scheduler.client_for_slot(slot);
        let response = lna_pending
            .WithStallTimeout(
                async {
                    client
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
        let response_address_space = response
            .extensions()
            .get::<fetch_policy::RawNetworkConnection>()
            .map(|connection| match connection.transport {
                fetch_policy::RawNetworkTransport::Proxied => crate::IPAddressSpace::Unknown,
                fetch_policy::RawNetworkTransport::Direct => request
                    .fetch
                    .as_ref()
                    .map(|fetch| connection.AddressSpace(fetch.security.as_ref()))
                    .unwrap_or_else(|| fetch_policy::AddressSpaceFromIP(connection.peer.ip())),
            })
            .unwrap_or(crate::IPAddressSpace::Unknown);
        if let Some(fetch) = &request.fetch {
            if fetch.mode == crate::FetchMode::Cors
                && (tainted_origin || !fetch.requestor_origin.IsSameOrigin(url.as_str()))
            {
                let origin = if tainted_origin {
                    "null".to_owned()
                } else {
                    fetch.requestor_origin.Serialize()
                };
                let allow = response
                    .headers()
                    .get("access-control-allow-origin")
                    .and_then(|value| value.to_str().ok());
                let wildcard =
                    allow == Some("*") && fetch.credentials != crate::FetchCredentialsMode::Include;
                if !wildcard && allow != Some(origin.as_str()) {
                    fetch.access_check_failed.store(true, Ordering::Release);
                    return Err(protocol_error(
                        "image CORS access-control-allow-origin mismatch",
                    ));
                }
                if fetch.credentials == crate::FetchCredentialsMode::Include
                    && response
                        .headers()
                        .get("access-control-allow-credentials")
                        .and_then(|value| value.to_str().ok())
                        != Some("true")
                {
                    fetch.access_check_failed.store(true, Ordering::Release);
                    return Err(protocol_error("image CORS credential response disallowed"));
                }
            }
        }
        let status = response.status();
        trace.event(
            "HTTPResponseHeaders",
            &[
                ("attempt", attempt as f64),
                ("redirect", redirects as f64),
                ("status_code", status.as_u16() as f64),
                (
                    "protocol",
                    match response.version() {
                        hyper::Version::HTTP_09 => 9.0,
                        hyper::Version::HTTP_10 => 10.0,
                        hyper::Version::HTTP_11 => 11.0,
                        hyper::Version::HTTP_2 => 20.0,
                        hyper::Version::HTTP_3 => 30.0,
                        _ => 0.0,
                    },
                ),
                (
                    "content_length",
                    response
                        .headers()
                        .get(header::CONTENT_LENGTH)
                        .and_then(|value| value.to_str().ok())
                        .and_then(|value| value.parse::<u64>().ok())
                        .map_or(-1.0, |length| length as f64),
                ),
            ],
        );
        let redirect_mode = request
            .fetch
            .as_ref()
            .map_or(crate::FetchRedirectMode::Follow, |fetch| fetch.redirect);
        if matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308)
            && response.headers().contains_key(header::LOCATION)
            && redirect_mode == crate::FetchRedirectMode::Error
        {
            return Err(protocol_error("Fetch redirect mode is error"));
        }
        if matches!(status.as_u16(), 301 | 302 | 303 | 307 | 308)
            && redirect_mode == crate::FetchRedirectMode::Follow
        {
            if let Some(location) = response.headers().get(header::LOCATION) {
                if redirects == 10 {
                    return Err(protocol_error("HTTP redirect limit exceeded"));
                }
                let next = url
                    .join(location.to_str().map_err(protocol_error)?)
                    .map_err(protocol_error)?;
                http_url(next.as_str())?;
                if let Some(fetch) = &request.fetch {
                    if fetch.mode == crate::FetchMode::Cors
                        && (!next.username().is_empty() || next.password().is_some())
                    {
                        fetch.access_check_failed.store(true, Ordering::Release);
                        return Err(protocol_error("CORS redirect URL contains credentials"));
                    }
                    if next.origin() != url.origin()
                        && !fetch.requestor_origin.IsSameOrigin(url.as_str())
                    {
                        tainted_origin = true;
                    }
                }
                if next.origin() != url.origin() {
                    headers.remove(header::AUTHORIZATION);
                    headers.remove(header::COOKIE);
                }
                headers.remove(header::HOST);
                // RedirectInfo::ComputeRedirectInfo updates policy from this
                // response and further reduces the current request referrer.
                // Never recover a path already stripped on an earlier hop.
                let policy_header =
                    response_header_values(response.headers(), "referrer-policy").join(",");
                if let Some(policy) = crate::ReferrerPolicy::FromHeader(&policy_header) {
                    referrer_policy = policy;
                }
                referrer = referrer_policy.GenerateReferrer(&referrer, next.as_str());
                // net/url_request/redirect_info.cc:ComputeMethod. Fetch
                // converts POST on 301/302 and non-GET/HEAD on 303 to GET.
                if status.as_u16() == 303 && method != Method::GET && method != Method::HEAD
                    || matches!(status.as_u16(), 301 | 302) && method == Method::POST
                {
                    method = Method::GET;
                    body.clear();
                    headers.remove(header::CONTENT_LENGTH);
                    headers.remove(header::TRANSFER_ENCODING);
                    headers.remove(header::CONTENT_TYPE);
                }
                url = next;
                drop(response);
                continue;
            }
        }
        if let Some(fetch) = &request.fetch {
            if fetch.keepalive && !fetch.succeeded.swap(true, Ordering::AcqRel) {
                fetch.metrics.Record(
                    fetch.context,
                    crate::FetchKeepaliveRequestState::Succeeded,
                    fetch.context_detached.load(Ordering::Acquire),
                );
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
        let referrer_policy =
            response_header_values(response.headers(), "referrer-policy").join(",");
        let content_security_policy =
            response_header_values(response.headers(), "content-security-policy");
        let content_security_policy_report_only =
            response_header_values(response.headers(), "content-security-policy-report-only");
        let permissions_policy =
            response_header_values(response.headers(), "permissions-policy").join(",");
        let feature_policy = response_header_values(response.headers(), "feature-policy").join(",");
        let document_policy =
            response_header_values(response.headers(), "document-policy").join(",");
        let document_policy_report_only =
            response_header_values(response.headers(), "document-policy-report-only").join(",");
        let require_document_policy =
            response_header_values(response.headers(), "require-document-policy").join(",");
        let content_dpr = response_content_dpr(response.headers());
        // ResourceResponse's combined HTTP Content-Language field. Preserve
        // the original octets as Latin-1, as for other HTTP field values.
        let content_language = response
            .headers()
            .get_all(header::CONTENT_LANGUAGE)
            .iter()
            .map(|value| {
                value
                    .as_bytes()
                    .iter()
                    .copied()
                    .map(char::from)
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join(",");
        let content_disposition = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .map(|value| value.as_bytes().iter().copied().map(char::from).collect())
            .unwrap_or_default();
        let expected_content_length = response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok());
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_owned(),
                    value.as_bytes().iter().copied().map(char::from).collect(),
                )
            })
            .collect();
        let send_start = *send_start.0.lock().expect("send start poisoned");
        return Ok(ResponseStart {
            send_start,
            expected_content_length,
            headers,
            incoming: response.into_body(),
            status: i64::from(status.as_u16()),
            final_url: url.to_string(),
            mime_type,
            content_dpr,
            content_disposition,
            content_language,
            request_referrer: Some(referrer),
            referrer_policy,
            content_security_policy,
            content_security_policy_report_only,
            response_address_space,
            permissions_policy,
            feature_policy,
            document_policy,
            document_policy_report_only,
            require_document_policy,
            encodings,
            head_only: method == Method::HEAD,
        });
    }
    unreachable!("redirect limit checked above")
}

pub(crate) struct ResponseStart {
    pub(crate) send_start: Option<Instant>,
    pub expected_content_length: Option<u64>,
    pub headers: Vec<(String, String)>,
    pub(crate) incoming: hyper::body::Incoming,
    pub(crate) status: i64,
    pub(crate) final_url: String,
    pub(crate) mime_type: String,
    pub(crate) content_dpr: Option<f64>,
    pub(crate) content_disposition: String,
    pub(crate) content_language: String,
    pub(crate) request_referrer: Option<String>,
    pub(crate) referrer_policy: String,
    pub(crate) content_security_policy: Vec<String>,
    pub(crate) content_security_policy_report_only: Vec<String>,
    pub(crate) response_address_space: crate::IPAddressSpace,
    pub(crate) permissions_policy: String,
    pub(crate) feature_policy: String,
    pub(crate) document_policy: String,
    pub(crate) document_policy_report_only: String,
    pub(crate) require_document_policy: String,
    pub(crate) encodings: String,
    pub(crate) head_only: bool,
}

fn response_header_values(headers: &HeaderMap, name: &'static str) -> Vec<String> {
    headers
        .get_all(name)
        .iter()
        // Preserve even invalid/empty enforced CSP headers as a policy
        // boundary. Their presence must not disappear from preload admission.
        .map(|value| value.to_str().unwrap_or_default().to_owned())
        .collect()
}

fn response_content_dpr(headers: &HeaderMap) -> Option<f64> {
    // cpp: core/layout/layout_image.cc:177-185. Preserve the response-owned
    // DPR fact until the selected HTML image use applies the override.
    headers
        .get("content-dpr")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
}

#[cfg(test)]
mod response_header_tests {
    use super::*;

    #[test]
    fn content_dpr_accepts_only_positive_finite_numbers() {
        for (header, expected) in [
            ("2.5", Some(2.5)),
            (" 4 ", Some(4.0)),
            ("0", None),
            ("-1", None),
            ("NaN", None),
            ("invalid", None),
        ] {
            let mut headers = HeaderMap::new();
            headers.insert("content-dpr", HeaderValue::from_str(header).unwrap());
            assert_eq!(response_content_dpr(&headers), expected, "{header}");
        }
    }
}

async fn transfer_traced(
    request: &HTTPRequestData,
    scheduler: &Scheduler,
    slot: &ScheduledSlot,
    trace: RequestTrace,
    attempt: usize,
    response_state: Option<&State>,
) -> Result<HTTPResponseData, TransferError> {
    let ResponseStart {
        send_start,
        expected_content_length,
        headers,
        mut incoming,
        status,
        final_url,
        mime_type,
        content_dpr,
        content_disposition,
        content_language,
        request_referrer,
        referrer_policy,
        content_security_policy,
        content_security_policy_report_only,
        response_address_space,
        permissions_policy,
        feature_policy,
        document_policy,
        document_policy_report_only,
        require_document_policy,
        encodings,
        head_only,
    } = fetch_response(request, scheduler, slot, trace, attempt).await?;
    if let Some(state) = response_state {
        state.response_started.store(true, Ordering::Release);
        let wake = state
            .wake
            .lock()
            .expect("HTTP header wake poisoned")
            .clone();
        if let Some(wake) = wake {
            wake();
        }
    }
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
    // ResourceLoadTiming::ResponseEnd records network EOF, before content
    // decoding or document-thread polling (image_resource.cc:177-183).
    let response_end = Some(Instant::now());
    let encoded_bytes = received.len();
    trace.event(
        "HTTPNetworkBodyComplete",
        &[
            ("attempt", attempt as f64),
            ("encoded_bytes", encoded_bytes as f64),
        ],
    );
    let body = if head_only || received.is_empty() {
        received
    } else {
        decode_body(received, &encodings, options.max_response_bytes)?
    };
    trace.event(
        "HTTPBodyComplete",
        &[
            ("attempt", attempt as f64),
            ("encoded_bytes", encoded_bytes as f64),
            ("decoded_bytes", body.len() as f64),
            ("network_eof", 1.0),
        ],
    );
    Ok(HTTPResponseData {
        send_start,
        response_end,
        expected_content_length,
        headers,
        status,
        final_url,
        mime_type,
        content_dpr,
        content_disposition,
        content_language,
        request_referrer,
        referrer_policy,
        content_security_policy,
        content_security_policy_report_only,
        response_address_space,
        permissions_policy,
        feature_policy,
        document_policy,
        document_policy_report_only,
        require_document_policy,
        body,
    })
}

#[cfg(test)]
async fn transfer(
    request: &HTTPRequestData,
    scheduler: &Scheduler,
) -> Result<HTTPResponseData, TransferError> {
    let slot = scheduler
        .acquire_slot(request.priority)
        .await
        .map_err(|_| protocol_error("HTTP admission closed"))?;
    transfer_traced(request, scheduler, &slot, RequestTrace::default(), 0, None).await
}

async fn execute(mut request: HTTPRequestData, state: Arc<State>) {
    'schedule: loop {
        let slot = loop {
            let priority = match state.priority.load(Ordering::Acquire) {
                2 => HTTPRequestPriority::VeryHigh,
                1 => HTTPRequestPriority::High,
                _ => HTTPRequestPriority::Normal,
            };
            let admission = state.scheduler.acquire_slot(priority);
            tokio::select! {
                biased;
                _ = state.priority_changed.notified() => continue,
                permit = admission => break permit,
            }
        };
        let Ok(slot) = slot else {
            state
                .trace
                .failed(0, &protocol_error("HTTP admission closed"), false);
            return;
        };
        state.trace.admitted();
        let options = &state.scheduler.options;
        for attempt in 0.. {
            request.priority = match state.priority.load(Ordering::Acquire) {
                2 => HTTPRequestPriority::VeryHigh,
                1 => HTTPRequestPriority::High,
                _ => HTTPRequestPriority::Normal,
            };
            let priority_changed = state.priority_changed.notified();
            let future = transfer_traced(
                &request,
                &state.scheduler,
                &slot,
                state.trace,
                attempt,
                Some(state.as_ref()),
            );
            let transfer = async {
                if options.timeout_ms > 0 {
                    tokio::time::timeout(Duration::from_millis(options.timeout_ms as u64), future)
                        .await
                        .unwrap_or_else(|_| Err(TransferError::new("HTTP request timeout", true)))
                } else {
                    future.await
                }
            };
            tokio::pin!(priority_changed);
            tokio::pin!(transfer);
            let result = tokio::select! {
                biased;
                _ = &mut priority_changed => {
                    let raised = match state.priority.load(Ordering::Acquire) {
                        2 => HTTPRequestPriority::VeryHigh,
                        1 => HTTPRequestPriority::High,
                        _ => HTTPRequestPriority::Normal,
                    };
                    if raised != request.priority {
                        // Hyper does not expose HTTP/2 stream reprioritization.
                        // Restart an idempotent transfer through the newly chosen
                        // admission class/client. Raising prevents a now-visible
                        // resource from remaining starved on a saturated
                        // connection; lowering releases reserved capacity once
                        // its consumer leaves the viewport.
                        // Dropping the request future cancels the old stream; the
                        // outer loop releases its old-class permit before it
                        // acquires the raised-class one.
                        if request.method == "GET" || request.method == "HEAD" {
                            state.trace.event(
                                "HTTPRequestRestartedForPriority",
                                &[("priority", raised as u8 as f64)],
                            );
                            continue 'schedule;
                        }
                    }
                    // Priority is monotonic and notifications are emitted only
                    // for a successful raise, so this branch is reachable solely
                    // for a coalesced notification consumed after the request had
                    // already observed that value. Continue the same transfer.
                    transfer.await
                }
                result = &mut transfer => result,
            };
            match result {
                Ok(response) => {
                    let mut result = state.result.lock().expect("HTTP state poisoned");
                    result.response = Some(response);
                    state
                        .trace
                        .event("HTTPResponseReady", &[("attempt", attempt as f64)]);
                    return;
                }
                Err(error) => {
                    if state.cancelled.load(Ordering::Relaxed) {
                        state.trace.event("HTTPRequestCancelled", &[]);
                        return;
                    }
                    let idempotent = request.method == "GET" || request.method == "HEAD";
                    let retry = idempotent && error.retryable && attempt < options.max_retries;
                    state.trace.failed(attempt, &error, retry);
                    if retry {
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
}

struct State {
    priority: AtomicU8,
    priority_changed: Notify,
    cancelled: AtomicBool,
    response_started: AtomicBool,
    keepalive: bool,
    detached: Arc<AtomicBool>,
    detached_notify: Notify,
    disconnected_at: Mutex<Option<Instant>>,
    cancel: Notify,
    result: Mutex<WorkerResult>,
    wake: Mutex<Option<HTTPWakeCallback>>,
    scheduler: Arc<Scheduler>,
    trace: RequestTrace,
}

#[derive(Default)]
struct WorkerResult {
    response: Option<HTTPResponseData>,
    error: Option<String>,
}

struct HTTPRequest {
    state: Arc<State>,
    delivered_response_started: bool,
}

impl HTTPRequest {
    fn new(mut request: HTTPRequestData, scheduler: Arc<Scheduler>) -> io::Result<Self> {
        let executor = runtime()?;
        let trace = RequestTrace::queued(&request, false);
        // The URLLoader receiver can disconnect while its Document fetch
        // context is still alive. Keep this connection flag separate from
        // NetworkFetchOptions::context_detached (the shared context property).
        let receiver_detached = Arc::new(AtomicBool::new(false));
        if let Some(fetch) = request.fetch.as_mut() {
            if let Some(wake) = fetch.policy_effect_wake.take() {
                let detached = receiver_detached.clone();
                fetch.policy_effect_wake = Some(Arc::new(move || {
                    if !detached.load(Ordering::Acquire) {
                        wake();
                    }
                }));
            }
        }
        let state = Arc::new(State {
            priority: AtomicU8::new(request.priority as u8),
            priority_changed: Notify::new(),
            cancelled: AtomicBool::new(false),
            response_started: AtomicBool::new(false),
            keepalive: request.fetch.as_ref().is_some_and(|fetch| fetch.keepalive),
            detached: receiver_detached,
            detached_notify: Notify::new(),
            disconnected_at: Mutex::new(None),
            cancel: Notify::new(),
            result: Mutex::new(WorkerResult::default()),
            wake: Mutex::new(None),
            scheduler,
            trace,
        });
        let worker_state = state.clone();
        let fetch = request.fetch.clone();
        executor.spawn(async move {
            let mut trace_guard = trace.guard();
            tokio::select! {
                biased;
                _ = worker_state.cancel.notified() => {},
                _ = async {
                    worker_state.detached_notify.notified().await;
                    let disconnected=*worker_state.disconnected_at.lock().expect("HTTP detach state poisoned");
                    let deadline=disconnected.expect("keepalive detach timestamp")+Duration::from_secs(30);
                    tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
                }, if worker_state.keepalive => {
                    worker_state.cancelled.store(true,Ordering::Relaxed);
                    worker_state.trace.event("KeepAliveDisconnectedTimeout",&[]);
                    worker_state.result.lock().expect("HTTP state poisoned").error=Some("keepalive disconnected loader timeout".into());
                    if let Some(fetch)=&fetch {if fetch.keepalive && !fetch.failed.swap(true,Ordering::AcqRel){fetch.metrics.Record(fetch.context,crate::FetchKeepaliveRequestState::Failed,true);}}
                },
                _ = execute(request, worker_state.clone()) => {
                    trace_guard.finish();
                    if worker_state.result.lock().expect("HTTP state poisoned").error.is_some() {
                        if let Some(fetch)=&fetch {if fetch.keepalive && !fetch.failed.swap(true,Ordering::AcqRel) {fetch.metrics.Record(fetch.context,crate::FetchKeepaliveRequestState::Failed,worker_state.detached.load(Ordering::Acquire));}}
                    }
                    let callback = worker_state
                        .wake
                        .lock()
                        .expect("HTTP wake poisoned")
                        .clone();
                    if let Some(callback) = callback {
                        callback();
                    }
                },
            }
        });
        Ok(Self {
            state,
            delivered_response_started: false,
        })
    }
}

impl Drop for HTTPRequest {
    fn drop(&mut self) {
        if self.state.keepalive {
            let _ = self.DetachForKeepalive();
        } else {
            self.state.cancelled.store(true, Ordering::Relaxed);
            self.state.cancel.notify_one();
        }
    }
}

impl HTTPRequestOperation for HTTPRequest {
    fn HasResponseStarted(&self) -> Option<bool> {
        Some(self.delivered_response_started)
    }
    fn CancelKeepalive(&mut self) -> io::Result<()> {
        self.state.cancelled.store(true, Ordering::Release);
        self.state.cancel.notify_one();
        Ok(())
    }

    fn DetachForKeepalive(&mut self) -> io::Result<()> {
        if !self.state.keepalive {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ordinary HTTP request cannot detach as keepalive",
            ));
        }
        self.state.detached.store(true, Ordering::Release);
        self.state
            .disconnected_at
            .lock()
            .expect("HTTP detach state poisoned")
            .get_or_insert(Instant::now());
        *self.state.wake.lock().expect("HTTP wake poisoned") = None;
        self.state.detached_notify.notify_one();
        Ok(())
    }

    fn SetPriority(&mut self, priority: HTTPRequestPriority) {
        let requested = priority as u8;
        let previous = self.state.priority.swap(requested, Ordering::AcqRel);
        if requested != previous {
            self.state.trace.event(
                if requested > previous {
                    "HTTPRequestPriorityRaised"
                } else {
                    "HTTPRequestPriorityLowered"
                },
                &[("priority", requested as f64)],
            );
            self.state.priority_changed.notify_one();
        }
    }
    fn Poll(&mut self) -> io::Result<Option<HTTPResponseData>> {
        self.delivered_response_started |= self.state.response_started.load(Ordering::Acquire);
        let mut result = self.state.result.lock().expect("HTTP state poisoned");
        if let Some(error) = &result.error {
            return Err(io::Error::other(error.clone()));
        }
        let response = result.response.take();
        if response.is_some() {
            self.state.trace.event("HTTPResponseDelivered", &[]);
        }
        Ok(response)
    }

    fn SetWakeCallback(&mut self, callback: HTTPWakeCallback) {
        *self.state.wake.lock().expect("HTTP wake poisoned") = Some(callback.clone());
        let ready = {
            let result = self.state.result.lock().expect("HTTP state poisoned");
            (!self.delivered_response_started
                && self.state.response_started.load(Ordering::Acquire))
                || result.response.is_some()
                || result.error.is_some()
        };
        if ready {
            callback();
        }
    }
}

// cpp: http_transport/http_request.h:36-48
#[derive(Clone)]
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
    use std::path::PathBuf;
    use std::sync::atomic::AtomicU64;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_rustls::TlsAcceptor;

    struct CertificateFile(PathBuf);

    impl CertificateFile {
        fn new(contents: &[u8]) -> Self {
            static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "rechrom-ca-{}-{}.pem",
                std::process::id(),
                NEXT_FILE.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::write(&path, contents).unwrap();
            Self(path)
        }

        fn fixture() -> Self {
            let encoded = base64::engine::general_purpose::STANDARD
                .encode(include_bytes!("../tests/fixtures/ca.der"));
            Self::new(
                format!("-----BEGIN CERTIFICATE-----\n{encoded}\n-----END CERTIFICATE-----\n")
                    .as_bytes(),
            )
        }
    }

    impl Drop for CertificateFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn configured_ca_file_preserves_public_roots() {
        let file = CertificateFile::fixture();
        let defaults = root_certificates(None).unwrap();
        let configured = root_certificates(Some(&file.0)).unwrap();
        assert_eq!(defaults.len(), webpki_roots::TLS_SERVER_ROOTS.len());
        assert_eq!(configured.len(), defaults.len() + 1);
        assert_eq!(configured.roots[..defaults.len()], defaults.roots);
    }

    #[test]
    fn configured_ca_file_reports_missing_empty_and_invalid_files() {
        let missing = CertificateFile::new(b"");
        std::fs::remove_file(&missing.0).unwrap();
        assert!(root_certificates(Some(&missing.0))
            .unwrap_err()
            .to_string()
            .contains("SSL_CERT_FILE could not be read"));

        for contents in [
            b"".as_slice(),
            b"not a certificate".as_slice(),
            b"-----BEGIN CERTIFICATE-----\n!invalid-base64!\n-----END CERTIFICATE-----\n"
                .as_slice(),
            b"-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----\n".as_slice(),
        ] {
            let file = CertificateFile::new(contents);
            let error = root_certificates(Some(&file.0)).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert!(error.to_string().contains("SSL_CERT_FILE"));
        }
    }

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
            priority_slots: None,
            critical_slots: None,
            client: build_client_with_proxy(fixture_roots(), Some(Duration::from_secs(2)), proxy)
                .unwrap(),
            priority_clients: Vec::new(),
            critical_clients: Vec::new(),
            priority_client_lanes: Arc::new(Mutex::new(Vec::new())),
            critical_client_lanes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[test]
    fn very_high_admission_survives_saturated_module_and_ordinary_pools() {
        let scheduler = Scheduler::new(HTTPTransportOptions {
            max_parallel_requests: 4,
            ..Default::default()
        })
        .unwrap();
        runtime().unwrap().block_on(async {
            // max=4 partitions into two ordinary, one High and one VeryHigh
            // permit. Saturating the first two classes must not make a
            // parser/render-blocking stylesheet wait behind them.
            let _ordinary_1 = scheduler
                .acquire_slot(HTTPRequestPriority::Normal)
                .await
                .unwrap();
            let _ordinary_2 = scheduler
                .acquire_slot(HTTPRequestPriority::Normal)
                .await
                .unwrap();
            let _module = scheduler
                .acquire_slot(HTTPRequestPriority::High)
                .await
                .unwrap();
            let _critical = tokio::time::timeout(
                Duration::from_millis(50),
                scheduler.acquire_slot(HTTPRequestPriority::VeryHigh),
            )
            .await
            .expect("VeryHigh admission must retain its own capacity")
            .unwrap();
        });
    }

    #[test]
    fn concurrent_high_admissions_use_distinct_connection_lanes() {
        let scheduler = Scheduler::new(HTTPTransportOptions {
            max_parallel_requests: 8,
            ..Default::default()
        })
        .unwrap();
        runtime().unwrap().block_on(async {
            let first = scheduler
                .acquire_slot(HTTPRequestPriority::High)
                .await
                .unwrap();
            let second = scheduler
                .acquire_slot(HTTPRequestPriority::High)
                .await
                .unwrap();
            let first_client = scheduler.client_for_slot(&first) as *const HTTPClient;
            let second_client = scheduler.client_for_slot(&second) as *const HTTPClient;
            assert_ne!(first_client, second_client);

            drop(second);
            let replacement = scheduler
                .acquire_slot(HTTPRequestPriority::High)
                .await
                .unwrap();
            assert_eq!(
                scheduler.client_for_slot(&replacement) as *const HTTPClient,
                second_client
            );
            assert_ne!(
                scheduler.client_for_slot(&replacement) as *const HTTPClient,
                first_client
            );
        });
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
    fn http2_tls_negotiation_accepts_chromium_sized_headers_and_multiplexes() {
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
                // Larger than Hyper's 16 KiB default and representative of
                // policy/reporting response headers emitted by large sites.
                let response = hyper::Response::builder()
                    .status(200)
                    .header("x-large-policy", "x".repeat(24 * 1024))
                    .body(())
                    .unwrap();
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
        let file = CertificateFile::fixture();
        let roots = root_certificates(Some(&file.0)).unwrap();
        let scheduler = Scheduler {
            options: HTTPTransportOptions {
                max_retries: 0,
                ..Default::default()
            },
            slots: Arc::new(Semaphore::new(1)),
            priority_slots: None,
            critical_slots: None,
            client: build_client(roots, Some(Duration::from_secs(2))).unwrap(),
            priority_clients: Vec::new(),
            critical_clients: Vec::new(),
            priority_client_lanes: Arc::new(Mutex::new(Vec::new())),
            critical_client_lanes: Arc::new(Mutex::new(Vec::new())),
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

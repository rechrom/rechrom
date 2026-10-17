// cpp: http_transport/http_transport_options.h:6-31
#[derive(Clone, Debug)]
pub struct HTTPTransportOptions {
    pub trace_requests: bool,
    pub connect_timeout_ms: i64,
    pub timeout_ms: i64,
    pub stall_timeout_ms: i64,
    pub max_response_bytes: usize,
    pub max_parallel_requests: usize,
    pub max_retries: usize,
    pub user_agent: String,
    pub accept_language: String,
}

impl Default for HTTPTransportOptions {
    fn default() -> Self {
        Self {
            trace_requests: false,
            connect_timeout_ms: 30_000,
            timeout_ms: 0,
            // Chromium does not impose one generic response-header/body-idle
            // deadline on ordinary navigations. Once the connection is made,
            // keep waiting for network progress until the request is cancelled
            // or the transport reports a concrete error. Embedders can still
            // opt into a finite stall policy explicitly.
            stall_timeout_ms: 0,
            max_response_bytes: 64 * 1024 * 1024,
            max_parallel_requests: 8,
            max_retries: 1,
            user_agent: concat!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) ",
                "AppleWebKit/537.36 (KHTML, like Gecko) ",
                "Chrome/154.0.0.0 Safari/537.36"
            )
            .to_owned(),
            accept_language: "en-US,en;q=0.9".to_owned(),
        }
    }
}

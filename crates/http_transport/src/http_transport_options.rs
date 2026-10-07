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
            // This finite idle limit is an embedder policy, not Chromium's
            // ordinary navigation deadline. A streamed response may legitimately
            // pause beyond 10s; retain cancellation and fail after 30s without
            // network progress rather than replaying a committed document.
            stall_timeout_ms: 30_000,
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

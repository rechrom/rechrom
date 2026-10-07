use std::io;

// cpp: xhr_transport/xml_http_request_transport.h:17-23
#[derive(Clone, Default)]
pub struct XMLHttpRequestData {
    pub method: String,
    pub url: String,
    pub referrer: String,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

// cpp: xhr_transport/xml_http_request_transport.h:25-30
#[derive(Clone, Default)]
pub struct XMLHttpResponseData {
    pub status: i64,
    pub final_url: String,
    pub mime_type: String,
    pub body: Vec<u8>,
}

// cpp: xhr_transport/xml_http_request_transport.h:35-46
pub trait XMLHttpRequestOperation {
    fn Poll(&mut self) -> io::Result<Option<XMLHttpResponseData>>;
}
pub trait XMLHttpRequestTransport {
    fn Start(
        &mut self,
        request: &XMLHttpRequestData,
    ) -> io::Result<Box<dyn XMLHttpRequestOperation>>;
}

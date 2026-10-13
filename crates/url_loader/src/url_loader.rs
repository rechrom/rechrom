use std::io;

// cpp: resource_loader/resource_loader.h:14-21
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum RequestDestination {
    #[default]
    kDocument,
    kStyleSheet,
    kScript,
    kImage,
    kFont,
    kFetch,
}

/// Fetch scheduling intent supplied by the script owner. Async/defer/module
/// requests do not block the HTML parser (Blink FetchParameters::kLazyLoad).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RequestDefer {
    #[default]
    None,
    Lazy,
}

// cpp: resource_loader/resource_loader.h:23-30
#[derive(Clone, Debug)]
pub struct URLRequest {
    pub url: String,
    pub referrer: String,
    /// None inherits the document policy at the resource boundary.
    pub referrer_policy: Option<crate::ReferrerPolicy>,
    pub destination: RequestDestination,
    pub defer: RequestDefer,
    pub method: String,
    // C++ std::string can contain arbitrary POST bytes.
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

impl Default for URLRequest {
    fn default() -> Self {
        Self {
            url: String::new(),
            referrer: String::new(),
            referrer_policy: None,
            destination: RequestDestination::kDocument,
            defer: RequestDefer::None,
            method: "GET".to_owned(),
            body: Vec::new(),
            headers: Vec::new(),
        }
    }
}

// cpp: resource_loader/resource_loader.h:36-42
#[derive(Clone, Debug)]
pub struct URLResponse {
    pub final_url: String,
    pub status_code: i64,
    pub mime_type: String,
    pub text_encoding: String,
    pub referrer_policy: Option<crate::ReferrerPolicy>,
    /// Enforced policies only; report-only headers do not gate preloads.
    pub content_security_policy: Vec<String>,
    pub body: Vec<u8>,
}

impl Default for URLResponse {
    fn default() -> Self {
        Self {
            final_url: String::new(),
            status_code: 0,
            mime_type: String::new(),
            text_encoding: "utf-8".to_owned(),
            referrer_policy: None,
            content_security_policy: Vec::new(),
            body: Vec::new(),
        }
    }
}

// cpp: resource_loader/resource_loader.h:44-50
pub trait URLLoadOperation {
    /// A reused resource can acquire a more urgent consumer. Scheduling does
    /// not change resource identity or create a second byte transfer.
    fn SetDefer(&mut self, _defer: RequestDefer) {}
    fn Poll(&mut self) -> io::Result<Option<URLResponse>>;
    /// Notify the embedding event loop when a previously pending complete
    /// response becomes pollable. Resident/synchronous loaders may invoke the
    /// callback immediately; application-defined loaders can retain polling by
    /// using this default implementation.
    fn SetWakeCallback(&mut self, _callback: crate::URLWakeCallback) {}
}

// cpp: resource_loader/resource_loader.h:52-56
pub trait URLLoader {
    fn LoadStream(
        &mut self,
        request: &URLRequest,
    ) -> io::Result<Box<dyn crate::URLStreamOperation>> {
        Ok(Box::new(crate::stream::BufferedURLStream::new(
            self.Load(request)?,
        )))
    }
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>>;
}

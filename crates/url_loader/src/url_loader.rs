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

// cpp: resource_loader/resource_loader.h:23-30
#[derive(Clone, Debug)]
pub struct URLRequest {
    pub url: String,
    pub referrer: String,
    pub destination: RequestDestination,
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
            destination: RequestDestination::kDocument,
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
    pub body: Vec<u8>,
}

impl Default for URLResponse {
    fn default() -> Self {
        Self {
            final_url: String::new(),
            status_code: 0,
            mime_type: String::new(),
            text_encoding: "utf-8".to_owned(),
            body: Vec::new(),
        }
    }
}

// cpp: resource_loader/resource_loader.h:44-50
pub trait URLLoadOperation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>>;
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

//! Default URL dispatcher. Scheme implementations are private; every caller
//! uses the same URL request, response and operation interfaces.
#![allow(non_snake_case)]
use crate::{
    data::DecodeDataURL, file::FileURLLoad, http::HTTPURLLoad, memory::MemoryURLs,
    metadata::invalid, ready::ReadyURLLoad,
};
use crate::{URLLoadOperation, URLLoader, URLRequest};
use http_transport::{HTTPRequestTransport, HTTPTransportOptions};
use std::io;

// cpp: resource_loader/default_resource_loader.h:9
pub type DefaultURLLoaderOptions = HTTPTransportOptions;

// cpp: resource_loader/default_resource_loader.h:14-31
pub struct DefaultURLLoader {
    options: DefaultURLLoaderOptions,
    http: HTTPRequestTransport,
    memory_urls: MemoryURLs,
}

impl DefaultURLLoader {
    // cpp: resource_loader/default_resource_loader.cc:294-314
    pub fn new(options: DefaultURLLoaderOptions) -> io::Result<Self> {
        let http = HTTPRequestTransport::new(options.clone())?;
        if options.connect_timeout_ms <= 0
            || options.timeout_ms < 0
            || options.stall_timeout_ms < 0
            || options.max_response_bytes == 0
            || options.max_parallel_requests == 0
        {
            return Err(invalid("URL loader limits must be positive"));
        }
        Ok(Self {
            options,
            http,
            memory_urls: MemoryURLs::default(),
        })
    }

    /// Register an explicit memory: URL, independent of fetched-resource caching.
    pub fn RegisterMemoryURL(
        &mut self,
        url: String,
        mime_type: String,
        body: Vec<u8>,
        text_encoding: String,
    ) -> io::Result<()> {
        self.memory_urls
            .RegisterMemoryURL(url, mime_type, body, text_encoding)
    }
}

impl URLLoader for DefaultURLLoader {
    fn LoadStream(
        &mut self,
        request: &URLRequest,
    ) -> io::Result<Box<dyn crate::URLStreamOperation>> {
        if request.url.starts_with("http://") || request.url.starts_with("https://") {
            return Ok(Box::new(crate::http::HTTPURLStream::new(
                request,
                &mut self.http,
            )?));
        }
        if request.url.starts_with("file://") {
            return Ok(Box::new(crate::file_stream::FileURLStream::new(
                request.clone(),
                self.options.max_response_bytes,
            )?));
        }
        Ok(Box::new(crate::stream::BufferedURLStream::new(
            self.Load(request)?,
        )))
    }
    // cpp: resource_loader/default_resource_loader.cc:334-351
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        if self.options.trace_requests {
            eprintln!("URL {} {}", request.method, request.url);
        }
        if request.url.starts_with("memory:") {
            return self.memory_urls.Load(&request.url);
        }
        if request.url.starts_with("data:") {
            return Ok(Box::new(ReadyURLLoad {
                response: Some(DecodeDataURL(request, self.options.max_response_bytes)?),
            }));
        }
        if request.url.starts_with("file://") {
            return Ok(Box::new(FileURLLoad::new(
                request.clone(),
                self.options.max_response_bytes,
            )?));
        }
        if request.url.starts_with("http://") || request.url.starts_with("https://") {
            return Ok(Box::new(HTTPURLLoad::new(request, &mut self.http)?));
        }
        Err(invalid(format!("unsupported URL: {}", request.url)))
    }
}

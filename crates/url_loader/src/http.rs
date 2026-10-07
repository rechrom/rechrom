//! HTTP adapter. Connection management and protocol handling stay in http_transport.
#![allow(non_snake_case)]
use crate::metadata::{MimeTypeFor, ParseContentType};
use crate::{URLLoadOperation, URLRequest, URLResponse};
use http_transport::{HTTPRequestData, HTTPRequestOperation, HTTPRequestTransport};
use std::io;

// cpp: resource_loader/default_resource_loader.cc:236-257
pub(crate) struct HTTPURLLoad {
    request: Box<dyn HTTPRequestOperation>,
    mime: String,
}

impl HTTPURLLoad {
    pub(crate) fn new(
        request: &URLRequest,
        transport: &mut HTTPRequestTransport,
    ) -> io::Result<Self> {
        let request_operation = transport.Start(&HTTPRequestData {
            method: request.method.clone(),
            url: request.url.clone(),
            referrer: request.referrer.clone(),
            body: request.body.clone(),
            headers: request.headers.clone(),
        })?;
        Ok(Self {
            request: request_operation,
            mime: MimeTypeFor(request).to_owned(),
        })
    }
}

impl URLLoadOperation for HTTPURLLoad {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        let Some(response) = self.request.Poll()? else {
            return Ok(None);
        };
        let mut result = URLResponse {
            final_url: response.final_url,
            status_code: response.status,
            mime_type: self.mime.clone(),
            body: response.body,
            ..URLResponse::default()
        };
        if !response.mime_type.is_empty() {
            ParseContentType(
                &response.mime_type,
                &mut result.mime_type,
                &mut result.text_encoding,
            );
        }
        Ok(Some(result))
    }
}

/// Main-document HTTP streams preserve headers-before-body delivery.
pub(crate) struct HTTPURLStream {
    operation: Box<dyn http_transport::HTTPStreamOperation>,
    fallback_mime: String,
}
impl HTTPURLStream {
    pub(crate) fn new(
        request: &URLRequest,
        transport: &mut HTTPRequestTransport,
    ) -> io::Result<Self> {
        Ok(Self {
            operation: transport.StartStream(&HTTPRequestData {
                method: request.method.clone(),
                url: request.url.clone(),
                referrer: request.referrer.clone(),
                body: request.body.clone(),
                headers: request.headers.clone(),
            })?,
            fallback_mime: MimeTypeFor(request).to_owned(),
        })
    }
}
impl crate::URLStreamOperation for HTTPURLStream {
    fn Poll(&mut self, max_bytes: usize) -> io::Result<Option<crate::URLLoadEvent>> {
        use crate::{URLLoadEvent, URLResponseHead};
        use http_transport::HTTPStreamEvent;
        Ok(self.operation.Poll(max_bytes)?.map(|event| match event {
            HTTPStreamEvent::Response {
                status,
                final_url,
                content_type,
            } => {
                let mut head = URLResponseHead {
                    final_url,
                    status_code: status,
                    mime_type: self.fallback_mime.clone(),
                    text_encoding: "utf-8".into(),
                };
                if !content_type.is_empty() {
                    ParseContentType(&content_type, &mut head.mime_type, &mut head.text_encoding);
                }
                URLLoadEvent::Response(head)
            }
            HTTPStreamEvent::Data(data) => URLLoadEvent::Data(data),
            HTTPStreamEvent::Finished => URLLoadEvent::Finished,
        }))
    }
    fn SetWakeCallback(&mut self, callback: crate::URLWakeCallback) {
        self.operation.SetWakeCallback(callback);
    }
}

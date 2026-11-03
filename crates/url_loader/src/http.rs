//! HTTP adapter. Connection management and protocol handling stay in http_transport.
#![allow(non_snake_case)]
use crate::metadata::{MimeTypeFor, ParseContentType};
use crate::{URLLoadOperation, URLRequest, URLResponse};
use http_transport::{
    HTTPRequestData, HTTPRequestOperation, HTTPRequestPriority, HTTPRequestTransport,
};
use std::io;

// cpp: resource_loader/default_resource_loader.cc:236-257
pub(crate) struct HTTPURLLoad {
    request: Box<dyn HTTPRequestOperation>,
    mime: String,
    destination: crate::RequestDestination,
    script_type: crate::ScriptType,
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
            referrer_policy: request.referrer_policy,
            body: request.body.clone(),
            headers: request.headers.clone(),
            priority: PriorityFor(request.destination, request.defer, request.script_type),
        })?;
        Ok(Self {
            request: request_operation,
            mime: MimeTypeFor(request).to_owned(),
            destination: request.destination,
            script_type: request.script_type,
        })
    }
}

impl URLLoadOperation for HTTPURLLoad {
    fn SetDefer(&mut self, defer: crate::RequestDefer) {
        self.request
            .SetPriority(PriorityFor(self.destination, defer, self.script_type));
    }
    fn SetScheduling(
        &mut self,
        destination: crate::RequestDestination,
        defer: crate::RequestDefer,
        script_type: crate::ScriptType,
    ) {
        self.destination = destination;
        self.script_type = script_type;
        self.request
            .SetPriority(PriorityFor(destination, defer, script_type));
    }
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        let Some(response) = self.request.Poll()? else {
            return Ok(None);
        };
        let mut result = URLResponse {
            final_url: response.final_url,
            status_code: response.status,
            mime_type: self.mime.clone(),
            content_dpr: response.content_dpr,
            body: response.body,
            referrer_policy: crate::ReferrerPolicy::FromHeader(&response.referrer_policy),
            content_security_policy: response.content_security_policy,
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

    fn SetWakeCallback(&mut self, callback: crate::URLWakeCallback) {
        self.request.SetWakeCallback(callback);
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
                referrer_policy: request.referrer_policy,
                body: request.body.clone(),
                headers: request.headers.clone(),
                priority: PriorityFor(request.destination, request.defer, request.script_type),
            })?,
            fallback_mime: MimeTypeFor(request).to_owned(),
        })
    }
}

fn PriorityFor(
    destination: crate::RequestDestination,
    defer: crate::RequestDefer,
    script_type: crate::ScriptType,
) -> HTTPRequestPriority {
    match destination {
        crate::RequestDestination::kDocument
        | crate::RequestDestination::kStyleSheet
        | crate::RequestDestination::kFont => HTTPRequestPriority::VeryHigh,
        crate::RequestDestination::kScript
            if script_type == crate::ScriptType::Module || defer == crate::RequestDefer::None =>
        {
            HTTPRequestPriority::High
        }
        // Blink's ResourceFetcher::TypeToPriority starts ordinary images at
        // Low. Visibility/LCP may promote selected images later, but treating
        // every image as High lets image transfers occupy the slots reserved
        // for CSS and parser-blocking work.
        crate::RequestDestination::kImage
        | crate::RequestDestination::kScript
        | crate::RequestDestination::kFetch => HTTPRequestPriority::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_is_very_high_while_modules_keep_high_admission() {
        assert_eq!(
            PriorityFor(
                crate::RequestDestination::kStyleSheet,
                crate::RequestDefer::None,
                crate::ScriptType::Classic,
            ),
            HTTPRequestPriority::VeryHigh
        );
        assert_eq!(
            PriorityFor(
                crate::RequestDestination::kImage,
                crate::RequestDefer::None,
                crate::ScriptType::Classic,
            ),
            HTTPRequestPriority::Normal
        );
        assert_eq!(
            PriorityFor(
                crate::RequestDestination::kScript,
                crate::RequestDefer::Lazy,
                crate::ScriptType::Classic,
            ),
            HTTPRequestPriority::Normal
        );
        assert_eq!(
            PriorityFor(
                crate::RequestDestination::kScript,
                crate::RequestDefer::Lazy,
                crate::ScriptType::Module,
            ),
            HTTPRequestPriority::High
        );
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
                content_dpr,
                referrer_policy,
                content_security_policy,
            } => {
                let mut head = URLResponseHead {
                    final_url,
                    status_code: status,
                    mime_type: self.fallback_mime.clone(),
                    content_dpr,
                    text_encoding: "utf-8".into(),
                    referrer_policy: crate::ReferrerPolicy::FromHeader(&referrer_policy),
                    content_security_policy,
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

//! HTTP adapter. Connection management and protocol handling stay in http_transport.
#![allow(non_snake_case)]
use crate::metadata::{MimeTypeFor, ParseContentType};
use crate::{URLLoadOperation, URLRequest, URLResponse};
use http_transport::{
    HTTPRequestData, HTTPRequestOperation, HTTPRequestPriority, HTTPRequestTransport,
};
use std::{
    io,
    sync::{Arc, Mutex},
};

// Admission retains the exact EncodedFormData/File owners until the native
// reader completes. The callback owns no document/DOM/runtime state.
struct BodyReadState {
    result: Option<Result<Arc<[u8]>, fileapi::native_file::NativeFileError>>,
    wake: Option<crate::URLWakeCallback>,
}
struct PendingHTTPBody {
    request: HTTPRequestData,
    transport: HTTPRequestTransport,
    state: Arc<Mutex<BodyReadState>>,
}
impl PendingHTTPBody {
    fn new(request: &URLRequest, transport: &HTTPRequestTransport) -> Self {
        let state = Arc::new(Mutex::new(BodyReadState {
            result: None,
            wake: None,
        }));
        if let Some(body) = &request.encoded_body {
            let complete = state.clone();
            body.ReadAll(Box::new(move |result| {
                let wake = {
                    let mut complete = complete.lock().expect("upload admission state");
                    complete.result = Some(result);
                    complete.wake.clone()
                };
                if let Some(wake) = wake {
                    wake();
                }
            }));
        } else {
            state.lock().expect("upload admission state").result =
                Some(Ok(Arc::from(request.body.as_slice())));
        }
        Self {
            request: HTTPRequestData {
                method: request.method.clone(),
                url: request.url.clone(),
                referrer: request.referrer.clone(),
                referrer_policy: request.referrer_policy,
                body: Vec::new(),
                headers: request.headers.clone(),
                priority: PriorityFor(request.destination, request.defer, request.script_type),
                fetch: request.fetch.clone(),
            },
            transport: transport.clone(),
            state,
        }
    }
    fn ReadReady(&mut self) -> io::Result<bool> {
        let result = self
            .state
            .lock()
            .expect("upload admission state")
            .result
            .take();
        let Some(result) = result else {
            return Ok(false);
        };
        self.request.body = result.map_err(io::Error::other)?.to_vec();
        Ok(true)
    }
    fn SetWakeCallback(&self, callback: crate::URLWakeCallback) {
        let ready = {
            let mut state = self.state.lock().expect("upload admission state");
            state.wake = Some(callback.clone());
            state.result.is_some()
        };
        if ready {
            callback();
        }
    }
    fn Wake(&self) -> Option<crate::URLWakeCallback> {
        self.state
            .lock()
            .expect("upload admission state")
            .wake
            .clone()
    }
}

// cpp: resource_loader/default_resource_loader.cc:236-257
pub(crate) struct HTTPURLLoad {
    request: Option<Box<dyn HTTPRequestOperation>>,
    pending_body: Option<PendingHTTPBody>,
    mime: String,
    destination: crate::RequestDestination,
    script_type: crate::ScriptType,
}

impl HTTPURLLoad {
    pub(crate) fn new(
        request: &URLRequest,
        transport: &mut HTTPRequestTransport,
    ) -> io::Result<Self> {
        let mut load = Self {
            request: None,
            pending_body: Some(PendingHTTPBody::new(request, transport)),
            mime: MimeTypeFor(request).to_owned(),
            destination: request.destination,
            script_type: request.script_type,
        };
        load.StartReady()?;
        Ok(load)
    }
    fn StartReady(&mut self) -> io::Result<()> {
        let Some(pending) = &mut self.pending_body else {
            return Ok(());
        };
        if !pending.ReadReady()? {
            return Ok(());
        }
        let mut operation = pending.transport.Start(&pending.request)?;
        if let Some(wake) = pending.Wake() {
            operation.SetWakeCallback(wake);
        }
        self.request = Some(operation);
        self.pending_body = None;
        Ok(())
    }
}

impl URLLoadOperation for HTTPURLLoad {
    fn HasResponseStarted(&self) -> Option<bool> {
        self.request
            .as_ref()
            .map_or(Some(false), |request| request.HasResponseStarted())
    }
    fn CancelKeepalive(&mut self) -> io::Result<()> {
        self.request
            .as_mut()
            .ok_or_else(|| io::Error::other("upload body not admitted"))?
            .CancelKeepalive()
    }
    fn DetachForKeepalive(&mut self) -> io::Result<()> {
        self.request
            .as_mut()
            .ok_or_else(|| io::Error::other("upload body not admitted"))?
            .DetachForKeepalive()
    }

    fn SetDefer(&mut self, defer: crate::RequestDefer) {
        let priority = PriorityFor(self.destination, defer, self.script_type);
        if let Some(request) = &mut self.request {
            request.SetPriority(priority);
        }
        if let Some(pending) = &mut self.pending_body {
            pending.request.priority = priority;
        }
    }
    fn SetScheduling(
        &mut self,
        destination: crate::RequestDestination,
        defer: crate::RequestDefer,
        script_type: crate::ScriptType,
    ) {
        self.destination = destination;
        self.script_type = script_type;
        let priority = PriorityFor(destination, defer, script_type);
        if let Some(request) = &mut self.request {
            request.SetPriority(priority);
        }
        if let Some(pending) = &mut self.pending_body {
            pending.request.priority = priority;
        }
    }
    fn SetPriority(&mut self, priority: crate::ResourceLoadPriority) {
        let priority = match priority {
            crate::ResourceLoadPriority::Normal => HTTPRequestPriority::Normal,
            crate::ResourceLoadPriority::High => HTTPRequestPriority::High,
            crate::ResourceLoadPriority::VeryHigh => HTTPRequestPriority::VeryHigh,
        };
        if let Some(request) = &mut self.request {
            request.SetPriority(priority);
        }
        if let Some(pending) = &mut self.pending_body {
            pending.request.priority = priority;
        }
    }
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        self.StartReady()?;
        let Some(request) = &mut self.request else {
            return Ok(None);
        };
        let Some(response) = request.Poll()? else {
            return Ok(None);
        };
        let mut result = URLResponse {
            send_start: response.send_start,
            response_end: response.response_end,
            expected_content_length: response.expected_content_length,
            final_url: response.final_url,
            status_code: response.status,
            mime_type: self.mime.clone(),
            content_dpr: response.content_dpr,
            content_disposition: response.content_disposition,
            content_language: response.content_language,
            body: response.body,
            request_referrer: response.request_referrer,
            referrer_policy: crate::ReferrerPolicy::FromHeader(&response.referrer_policy),
            content_security_policy: response.content_security_policy,
            content_security_policy_report_only: response.content_security_policy_report_only,
            response_address_space: response.response_address_space,
            permissions_policy: response.permissions_policy,
            feature_policy: response.feature_policy,
            document_policy: response.document_policy,
            document_policy_report_only: response.document_policy_report_only,
            require_document_policy: response.require_document_policy,
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
        if let Some(request) = &mut self.request {
            request.SetWakeCallback(callback.clone());
        }
        if let Some(pending) = &self.pending_body {
            pending.SetWakeCallback(callback);
        }
    }
}

/// Main-document HTTP streams preserve headers-before-body delivery.
pub(crate) struct HTTPURLStream {
    operation: Option<Box<dyn http_transport::HTTPStreamOperation>>,
    pending_body: Option<PendingHTTPBody>,
    fallback_mime: String,
}
impl HTTPURLStream {
    pub(crate) fn new(
        request: &URLRequest,
        transport: &mut HTTPRequestTransport,
    ) -> io::Result<Self> {
        let mut stream = Self {
            operation: None,
            pending_body: Some(PendingHTTPBody::new(request, transport)),
            fallback_mime: MimeTypeFor(request).to_owned(),
        };
        stream.StartReady()?;
        Ok(stream)
    }
    fn StartReady(&mut self) -> io::Result<()> {
        let Some(pending) = &mut self.pending_body else {
            return Ok(());
        };
        if !pending.ReadReady()? {
            return Ok(());
        }
        let mut operation = pending.transport.StartStream(&pending.request)?;
        if let Some(wake) = pending.Wake() {
            operation.SetWakeCallback(wake);
        }
        self.operation = Some(operation);
        self.pending_body = None;
        Ok(())
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
        | crate::RequestDestination::kVideo
        | crate::RequestDestination::kTrack
        | crate::RequestDestination::kScript
        | crate::RequestDestination::kFetch
        | crate::RequestDestination::kReport => HTTPRequestPriority::Normal,
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
        self.StartReady()?;
        let Some(operation) = &mut self.operation else {
            return Ok(None);
        };
        Ok(operation.Poll(max_bytes)?.map(|event| match event {
            HTTPStreamEvent::Response {
                status,
                final_url,
                content_type,
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
            } => {
                let mut head = URLResponseHead {
                    final_url,
                    status_code: status,
                    mime_type: self.fallback_mime.clone(),
                    content_dpr,
                    content_disposition,
                    content_language,
                    text_encoding: "utf-8".into(),
                    request_referrer,
                    referrer_policy: crate::ReferrerPolicy::FromHeader(&referrer_policy),
                    content_security_policy,
                    content_security_policy_report_only,
                    response_address_space,
                    permissions_policy,
                    feature_policy,
                    document_policy,
                    document_policy_report_only,
                    require_document_policy,
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
        if let Some(operation) = &mut self.operation {
            operation.SetWakeCallback(callback.clone());
        }
        if let Some(pending) = &self.pending_body {
            pending.SetWakeCallback(callback);
        }
    }
}

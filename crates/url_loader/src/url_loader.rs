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
    kVideo,
    kTrack,
    kFont,
    kFetch,
    kReport,
}

/// Fetch scheduling intent supplied by the script owner. Async/defer/module
/// requests do not block the HTML parser (Blink FetchParameters::kLazyLoad).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RequestDefer {
    #[default]
    None,
    Lazy,
}

/// Dynamic load priority used when a previously background resource becomes
/// user-visible. The destination still owns the initial/base priority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResourceLoadPriority {
    #[default]
    Normal,
    High,
    VeryHigh,
}

/// Script kind used by Blink's `ResourceFetcher::ComputeLoadPriority`.
/// Module graph fetches keep the script type's high base priority even though
/// `ModuleScriptLoader` marks them lazy; only lazy classic scripts are lowered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScriptType {
    #[default]
    Classic,
    Module,
}

// cpp: resource_loader/resource_loader.h:23-30
#[derive(Clone, Debug)]
pub struct URLRequest {
    pub url: String,
    pub referrer: String,
    /// None inherits the document policy at the resource boundary.
    pub referrer_policy: Option<crate::ReferrerPolicy>,
    pub destination: RequestDestination,
    /// Source ResourceRequest::IsAutomaticUpgrade; redirects alone do not set it.
    pub is_automatic_upgrade: bool,
    /// Source ResourceRequest::UpgradeIfInsecure, retained across redirects.
    pub upgrade_if_insecure: bool,
    pub defer: RequestDefer,
    pub script_type: ScriptType,
    pub method: String,
    // C++ std::string can contain arbitrary POST bytes.
    pub body: Vec<u8>,
    /// Immutable upload owner. Native file bytes are admitted by URLLoader,
    /// outside document/Interaction computation, before HTTP transport starts.
    pub encoded_body: Option<std::sync::Arc<fileapi::encoded_form_data::EncodedFormData>>,
    pub headers: Vec<(String, String)>,
    pub fetch: Option<http_transport::NetworkFetchOptions>,
    /// Only document navigations carry these actual initiator/commit facts.
    pub navigation: Option<navigation_history::NavigationRequestMetadata>,
}

impl Default for URLRequest {
    fn default() -> Self {
        Self {
            url: String::new(),
            referrer: String::new(),
            referrer_policy: None,
            destination: RequestDestination::kDocument,
            is_automatic_upgrade: false,
            upgrade_if_insecure: false,
            defer: RequestDefer::None,
            script_type: ScriptType::Classic,
            method: "GET".to_owned(),
            body: Vec::new(),
            encoded_body: None,
            headers: Vec::new(),
            fetch: None,
            navigation: None,
        }
    }
}

// cpp: resource_loader/resource_loader.h:36-42
#[derive(Clone, Debug)]
pub struct URLResponse {
    /// Native network timestamps, before decoding and main-thread delivery.
    pub send_start: Option<std::time::Instant>,
    pub response_end: Option<std::time::Instant>,
    pub expected_content_length: Option<u64>,
    pub final_url: String,
    pub status_code: i64,
    pub mime_type: String,
    pub content_dpr: Option<f64>,
    pub content_disposition: String,
    pub content_language: String,
    pub text_encoding: String,
    /// Actual final request metadata; None means the loader did not expose it.
    pub request_referrer: Option<String>,
    pub referrer_policy: Option<crate::ReferrerPolicy>,
    /// Enforced policies only; report-only headers do not gate preloads.
    pub content_security_policy: Vec<String>,
    pub content_security_policy_report_only: Vec<String>,
    pub response_address_space: http_transport::IPAddressSpace,
    pub permissions_policy: String,
    pub feature_policy: String,
    pub document_policy: String,
    pub document_policy_report_only: String,
    pub require_document_policy: String,
    pub body: Vec<u8>,
}

impl Default for URLResponse {
    fn default() -> Self {
        Self {
            send_start: None,
            response_end: None,
            expected_content_length: None,
            final_url: String::new(),
            status_code: 0,
            mime_type: String::new(),
            content_dpr: None,
            content_disposition: String::new(),
            content_language: String::new(),
            text_encoding: "utf-8".to_owned(),
            request_referrer: None,
            referrer_policy: None,
            content_security_policy: Vec::new(),
            content_security_policy_report_only: Vec::new(),
            response_address_space: http_transport::IPAddressSpace::Unknown,
            permissions_policy: String::new(),
            feature_policy: String::new(),
            document_policy: String::new(),
            document_policy_report_only: String::new(),
            require_document_policy: String::new(),
            body: Vec::new(),
        }
    }
}

// cpp: resource_loader/resource_loader.h:44-50
pub trait URLLoadOperation {
    /// None requires a header-capable backend before the renderer-keepalive
    /// branch may decide ResourceResponse::IsNull. Body EOF is not a proxy.
    fn HasResponseStarted(&self) -> Option<bool> {
        None
    }
    fn CancelKeepalive(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "URL keepalive cancellation producer missing",
        ))
    }
    /// Disconnect the document-side receiver while the network owner keeps
    /// the admitted request alive. Keepalive backends must implement this.
    fn DetachForKeepalive(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "URL operation has no native keepalive owner",
        ))
    }

    /// A reused resource can acquire a more urgent consumer. Scheduling does
    /// not change resource identity or create a second byte transfer.
    fn SetDefer(&mut self, _defer: RequestDefer) {}
    /// A coalesced consumer can raise an in-flight request's scheduling
    /// priority without changing the cached resource identity.
    fn SetScheduling(
        &mut self,
        _destination: RequestDestination,
        defer: RequestDefer,
        _script_type: ScriptType,
    ) {
        self.SetDefer(defer);
    }
    fn SetPriority(&mut self, _priority: ResourceLoadPriority) {}
    fn Poll(&mut self) -> io::Result<Option<URLResponse>>;
    /// Notify the embedding event loop when a previously pending complete
    /// response becomes pollable. Resident/synchronous loaders may invoke the
    /// callback immediately; application-defined loaders can retain polling by
    /// using this default implementation.
    fn SetWakeCallback(&mut self, _callback: crate::URLWakeCallback) {}
}

// cpp: resource_loader/resource_loader.h:52-56
pub trait URLLoader {
    /// Required capability for RawResource keepalive. Default refusal occurs
    /// before starting any transfer; it never sends a request and drops it.
    fn LoadKeepalive(&mut self, _request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "URL loader has no native keepalive producer",
        ))
    }

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

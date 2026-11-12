// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: platform/loader/fetch/resource_request.h (navigation fields),
// core/loader/frame_load_request.cc:87-165; core/loader/ping_loader.cc:132-169.
// This is the resident request consumed by FrameLoadRequest. Conversion into a
// network package request happens only at the immutable loading boundary.
pub use navigation_history::ReferrerPolicy;
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestContext {
    Unspecified,
    Hyperlink,
    Download,
    Ping,
    Location,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestMode {
    NoCors,
    Navigate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetAddressSpace {
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialsMode {
    SameOrigin,
    Include,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedirectMode {
    Follow,
    Manual,
}
pub use navigation_history::RequestorOrigin;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FetchCacheMode {
    #[default]
    Default,
    ValidateCache,
    BypassCache,
}
#[derive(Clone, Debug)]
pub struct ResourceRequestData {
    pub url: String,
    pub cache_mode: FetchCacheMode,
    pub skip_service_worker: bool,
    pub method: String,
    pub body: Option<
        std::sync::Arc<interaction_dispatch::core::fileapi::encoded_form_data::EncodedFormData>,
    >,
    pub headers: Vec<(String, String)>,
    pub referrer: String,
    pub referrer_policy: ReferrerPolicy,
    pub context: RequestContext,
    pub mode: RequestMode,
    pub target_address_space: TargetAddressSpace,
    pub credentials: CredentialsMode,
    pub redirect: RedirectMode,
    pub keepalive: bool,
    pub is_form_submission: bool,
    pub has_user_gesture: bool,
    pub requestor_origin: Option<RequestorOrigin>,
    pub suggested_filename: Option<String>,
}
pub struct ResourceRequest {
    data: RefCell<ResourceRequestData>,
}
impl ResourceRequest {
    pub fn new(url: &str) -> Rc<Self> {
        Rc::new(Self {
            data: RefCell::new(ResourceRequestData {
                url: url.into(),
                cache_mode: FetchCacheMode::Default,
                skip_service_worker: false,
                method: "GET".into(),
                body: None,
                headers: Vec::new(),
                referrer: "about:client".into(),
                referrer_policy: ReferrerPolicy::Default,
                context: RequestContext::Unspecified,
                mode: RequestMode::NoCors,
                target_address_space: TargetAddressSpace::Unknown,
                credentials: CredentialsMode::Include,
                redirect: RedirectMode::Follow,
                keepalive: false,
                is_form_submission: false,
                has_user_gesture: false,
                requestor_origin: None,
                suggested_filename: None,
            }),
        })
    }
    /// FrameLoadRequest::CopyHeadFrom + SetHttpBody share the input's immutable
    /// body value, with independent request-head mutation as in the source.
    pub fn CopyForFrameLoad(&self) -> Rc<Self> {
        Rc::new(Self {
            data: RefCell::new(self.data.borrow().clone()),
        })
    }
    pub fn Read<R>(&self, f: impl FnOnce(&ResourceRequestData) -> R) -> R {
        f(&self.data.borrow())
    }
    pub fn Mutate<R>(&self, f: impl FnOnce(&mut ResourceRequestData) -> R) -> R {
        f(&mut self.data.borrow_mut())
    }
}

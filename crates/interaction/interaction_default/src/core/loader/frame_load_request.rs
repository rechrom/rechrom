// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/loader/frame_load_request.cc:33-79,87-165,169-190;
// .h:70-245. Source-window/script and blob capabilities retain native identity.
use super::{navigation_policy::NavigationPolicy, resource_request::*};
use crate::core::html::html_anchor_element::{AnchorFrameLoadRequest, AnchorResourceRequest};
use interaction_dispatch::core::dom::events::event::EventTargetRef;
use interaction_input::base::time::time::TimeTicks;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
/// FrameTree::FindFrameForNavigation reserved-name policy. Resolution uses
/// the embedding's existing FrameTree and never invents a second registry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameNavigationTarget {
    SelfFrame,
    Parent,
    Top,
    Named(String),
    NewContext,
}
impl From<String> for FrameNavigationTarget {
    fn from(name: String) -> Self {
        // FrameLoadRequest::CleanNavigationTarget cc:142-165. The embedding
        // uses Chromium's stable RemoveDanglingMarkupInTarget feature.
        if name.contains(['\n', '\r', '\t']) && name.contains('<') {
            return Self::NewContext;
        }
        if name.is_empty() || name.eq_ignore_ascii_case("_self") {
            Self::SelfFrame
        } else if name.eq_ignore_ascii_case("_parent") {
            Self::Parent
        } else if name.eq_ignore_ascii_case("_top") {
            Self::Top
        } else if name.eq_ignore_ascii_case("_blank") {
            Self::NewContext
        } else {
            Self::Named(name)
        }
    }
}
impl From<&str> for FrameNavigationTarget {
    fn from(name: &str) -> Self {
        name.to_owned().into()
    }
}
#[derive(Clone, Debug)]
pub struct NavigationInitiator {
    pub origin: RequestorOrigin,
    pub state_token: [u64; 2],
    pub document_token: Option<[u64; 2]>,
    pub world_id: i32,
    pub source_location: Option<NavigationSourceLocation>,
    pub script_tool_invocation_id: Option<[u64; 2]>,
}
pub use interaction_dispatch::bindings::script_invocation::ScriptSourceLocation as NavigationSourceLocation;
pub use interaction_dispatch::core::fileapi::blob_url_registry::BlobURLToken;
pub trait FrameLoadRequestEnvironment {
    fn InitiatorSecurityOriginOwner(&self) -> Rc<dyn foundation::security::SecurityOriginOwner>;
    fn Now(&self) -> TimeTicks;
    fn CurrentInputStartTime(&self) -> Option<TimeTicks>;
    fn BaseURL(&self) -> String;
    fn OutgoingReferrer(&self) -> String;
    fn DocumentReferrerPolicy(&self) -> ReferrerPolicy;
    fn GenerateReferrer(&self, policy: ReferrerPolicy, url: &str, source: &str) -> String;
    fn HasTransientUserActivation(&self) -> bool;
    fn RemoveDanglingMarkupInTargetEnabled(&self) -> bool;
    fn TargetBlankImpliesNoOpener(&self) -> bool;
    fn RelOpenerBcgDependencyHintEnabled(&self) -> bool;
    fn BlobURLSiteDiffersFromTopLevelSite(&self, url: &str) -> bool;
    fn EnforceNoopenerOnBlobURLNavigation(&self) -> bool;
    fn BlobURLPartitioningDisabled(&self) -> bool;
    fn ResolveBlobURLToken(&self, url: &str, top_level: bool) -> std::sync::Arc<BlobURLToken>;
    fn CountUse(&self, feature: &'static str);
    fn ReportPartitioningBlobURLIssue(&self, url: &str);
}
/// Source mojom TriggeringEventInfo distinguishes absence from an untrusted
/// event. A bool alone loses FormSubmission::Create's no-event branch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggeringEventInfo {
    NotFromEvent,
    Trusted,
    Untrusted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientNavigationReason {
    None,
    AnchorClick,
    FrameNavigation,
    Reload,
    FormSubmissionGet,
    FormSubmissionPost,
    HttpHeaderRefresh,
    MetaTagRefresh,
}
pub struct FrameLoadRequest {
    pub resource: Rc<ResourceRequest>,
    initiator_security_origin_owner: Rc<dyn foundation::security::SecurityOriginOwner>,
    pub initiator: NavigationInitiator,
    pub creation_time: TimeTicks,
    pub requestor_base_url: String,
    pub navigation_policy: Cell<NavigationPolicy>,
    pub input_start_time: Cell<TimeTicks>,
    pub source_element: RefCell<Option<EventTargetRef>>,
    pub noopener: Cell<bool>,
    pub explicit_opener: Cell<bool>,
    pub never_send_referrer: Cell<bool>,
    pub href_translate: RefCell<Option<String>>,
    pub triggering_event_trusted: Cell<bool>,
    pub anchor_click: Cell<bool>,
    pub client_navigation_reason: Cell<ClientNavigationReason>,
    pub triggering_event_info: Cell<TriggeringEventInfo>,
    pub force_history_push: Cell<bool>,
    pub resume_deferred_commit_listener: RefCell<Option<Rc<dyn crate::core::navigation_api::navigate_event_dispatch_params::NavigationResumeDeferredCommitListener>>>,
    pub form: RefCell<Option<EventTargetRef>>,
    pub is_unfenced_top_navigation: Cell<bool>,
    pub blob_url_token: RefCell<Option<std::sync::Arc<BlobURLToken>>>,
    environment: Weak<dyn FrameLoadRequestEnvironment>,
}
impl FrameLoadRequest {
    /// Source: core/page/frame_tree.cc:222-249. User navigation modifiers
    /// override target attributes; sandboxed synthetic modifiers do not.
    pub fn NavigationUsesCurrentFrame(&self, sandbox: foundation::security::SandboxFlags) -> bool {
        use foundation::security::SandboxFlags;
        if self.navigation_policy.get() != NavigationPolicy::kNavigationPolicyCurrentTab
            && self.triggering_event_info.get() == TriggeringEventInfo::Untrusted
            && (sandbox.Contains(SandboxFlags::POPUPS)
                || sandbox.Contains(SandboxFlags::PROPAGATES_TO_AUXILIARY))
        {
            self.navigation_policy
                .set(NavigationPolicy::kNavigationPolicyCurrentTab);
        }
        self.navigation_policy.get() != NavigationPolicy::kNavigationPolicyCurrentTab
    }
    pub fn new(
        resource: &ResourceRequest,
        environment: Rc<dyn FrameLoadRequestEnvironment>,
        initiator: NavigationInitiator,
    ) -> Rc<Self> {
        let initiator_security_origin_owner = environment.InitiatorSecurityOriginOwner();
        let resource = resource.CopyForFrameLoad();
        assert_ne!(
            initiator.state_token,
            [0, 0],
            "initiator state token is not empty"
        );
        let requestor_base_url = resource.Read(|r| {
            if r.url.is_empty() || r.url == "about:blank" || r.url == "about:srcdoc" {
                environment.BaseURL()
            } else {
                String::new()
            }
        });
        let (url, mut referrer, mut policy) =
            resource.Read(|r| (r.url.clone(), r.referrer.clone(), r.referrer_policy));
        if referrer == "about:client" {
            referrer = environment.OutgoingReferrer();
        }
        if policy == ReferrerPolicy::Default {
            policy = environment.DocumentReferrerPolicy();
        }
        let referrer = environment.GenerateReferrer(policy, &url, &referrer);
        resource.Mutate(|r| {
            assert!(r.requestor_origin.is_none());
            r.requestor_origin = Some(initiator.origin.clone());
            r.mode = RequestMode::Navigate;
            r.credentials = CredentialsMode::Include;
            r.redirect = RedirectMode::Manual;
            r.referrer = referrer;
            r.referrer_policy = policy;
            r.has_user_gesture |= environment.HasTransientUserActivation();
        });
        Rc::new(Self {
            resource,
            initiator_security_origin_owner,
            initiator,
            creation_time: environment.Now(),
            requestor_base_url,
            navigation_policy: Cell::new(NavigationPolicy::kNavigationPolicyCurrentTab),
            input_start_time: Cell::new(environment.CurrentInputStartTime().unwrap_or_default()),
            source_element: RefCell::new(None),
            noopener: Cell::new(false),
            explicit_opener: Cell::new(false),
            never_send_referrer: Cell::new(false),
            href_translate: RefCell::new(None),
            triggering_event_trusted: Cell::new(false),
            anchor_click: Cell::new(false),
            client_navigation_reason: Cell::new(ClientNavigationReason::None),
            triggering_event_info: Cell::new(TriggeringEventInfo::NotFromEvent),
            force_history_push: Cell::new(false),
            resume_deferred_commit_listener: RefCell::new(None),
            form: RefCell::new(None),
            is_unfenced_top_navigation: Cell::new(false),
            blob_url_token: RefCell::new(None),
            environment: Rc::downgrade(&environment),
        })
    }
    pub fn InitiatorSecurityOriginOwner(
        &self,
    ) -> Rc<dyn foundation::security::SecurityOriginOwner> {
        self.initiator_security_origin_owner.clone()
    }
    fn Environment(&self) -> Rc<dyn FrameLoadRequestEnvironment> {
        self.environment
            .upgrade()
            .expect("originating Window lives through navigation continuation")
    }
    pub fn SetFrameType(&self, top_level: bool) {
        let url = self.resource.Read(|r| r.url.clone());
        if url.starts_with("blob:") {
            *self.blob_url_token.borrow_mut() =
                Some(self.Environment().ResolveBlobURLToken(&url, top_level));
        }
    }
}
impl AnchorResourceRequest for ResourceRequest {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn IntoAny(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}
impl AnchorFrameLoadRequest for FrameLoadRequest {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn IntoAny(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
    fn SetNavigationPolicy(&self, policy: NavigationPolicy) {
        self.navigation_policy.set(policy);
    }
    fn SetClientNavigationReasonAnchorClick(&self) {
        self.anchor_click.set(true);
        self.client_navigation_reason
            .set(ClientNavigationReason::AnchorClick);
    }
    fn SetSourceElement(&self, source: &EventTargetRef) {
        *self.source_element.borrow_mut() = Some(source.clone());
    }
    fn CleanNavigationTarget(&self, target: &str) -> String {
        if target.contains(['\n', '\r', '\t']) && target.contains('<') {
            self.Environment().CountUse("DanglingMarkupInTarget");
            if !target.ends_with('>') {
                self.Environment()
                    .CountUse("DanglingMarkupInTargetNotEndsWithGT");
                if !target.ends_with('\n') {
                    self.Environment()
                        .CountUse("DanglingMarkupInTargetNotEndsWithNewLineOrGT");
                }
            }
            if self.Environment().RemoveDanglingMarkupInTargetEnabled() {
                return "_blank".into();
            }
        }
        target.into()
    }
    fn HandleRelAttribute(&self, target: &str, relations: u32) {
        if relations & 0x1000 != 0 {
            self.never_send_referrer.set(true);
            self.resource.Mutate(|r| {
                r.referrer.clear();
                r.referrer_policy = ReferrerPolicy::Never;
                r.headers.retain(|(k, _)| !k.eq_ignore_ascii_case("origin"));
            });
            self.noopener.set(true);
        }
        if relations & 0x40000 != 0
            || (target.eq_ignore_ascii_case("_blank")
                && relations & 0x80000 == 0
                && self.Environment().TargetBlankImpliesNoOpener())
        {
            self.noopener.set(true);
        }
        if self.Environment().RelOpenerBcgDependencyHintEnabled()
            && relations & 0x80000 != 0
            && !self.noopener.get()
        {
            self.explicit_opener.set(true);
        }
    }
    fn EnforceBlobUrlNoopenerIfNeeded(&self, url: &str) {
        if !url.starts_with("blob:") {
            return;
        }
        if self.Environment().BlobURLSiteDiffersFromTopLevelSite(url) {
            if self.Environment().EnforceNoopenerOnBlobURLNavigation()
                && !self.Environment().BlobURLPartitioningDisabled()
            {
                self.noopener.set(true);
            }
            self.Environment()
                .CountUse("CrossTopLevelSiteBlobURLNavigation");
            self.Environment().ReportPartitioningBlobURLIssue(url);
        }
    }
    fn SetTriggeringEventInfo(&self, trusted: bool) {
        self.triggering_event_trusted.set(trusted);
        self.triggering_event_info.set(if trusted {
            TriggeringEventInfo::Trusted
        } else {
            TriggeringEventInfo::Untrusted
        });
    }
    fn SetInputStartTime(&self, time: TimeTicks) {
        self.input_start_time.set(time);
    }
    fn Noopener(&self) -> bool {
        self.noopener.get()
    }
    fn SetHrefTranslate(&self, value: &str) {
        *self.href_translate.borrow_mut() = Some(value.into());
    }
}

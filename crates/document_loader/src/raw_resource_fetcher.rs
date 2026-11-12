//! PingLoader's FetchParameters -> RawResource producer on the same Document
//! fetch owner. Source ping_loader.cc:132-169; raw_resource.cc:65-72;
//! resource_fetcher.cc:2755-2828. No image admission or image completion path.
use super::ResourceFetcher;
use foundation::content_security_policy::{
    CSPRequestContext, CSPRequestPolicy, CSPViolation, CSPWorld,
};
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use url_loader::{
    FetchCredentialsMode, FetchMode, FetchOrigin, FetchRedirectMode, NetworkFetchOptions,
    URLRequest,
};
#[derive(Clone, Debug)]
pub struct RawResourceFetchOptions {
    pub security: Arc<fetch_policy::RawFetchSecurityContext>,
    pub context: CSPRequestContext,
    pub world: CSPWorld,
    pub requestor_origin: FetchOrigin,
    pub mode: FetchMode,
    pub credentials: FetchCredentialsMode,
    pub redirect: FetchRedirectMode,
}
pub(super) struct RawFetchContext {
    arena: dom::persistent_document::DOMOwnerHandle,
    policy: Arc<dyn CSPRequestPolicy>,
    violations: Arc<Mutex<Vec<CSPViolation>>>,
    mixed_content_effects: Arc<Mutex<Vec<fetch_policy::RawMixedContentEffect>>>,
    local_network_access_effects: Arc<Mutex<Vec<fetch_policy::RawLocalNetworkAccessEffect>>>,
    effect_notifier: std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>>,
    notifying: std::cell::Cell<bool>,
    context_detached: Arc<AtomicBool>,
}
impl RawFetchContext {
    pub(super) fn new(document: &dom::Document, context_detached: Arc<AtomicBool>) -> Self {
        Self {
            arena: document.RootHandle(),
            policy: document.ContentSecurityPolicy(),
            violations: Arc::new(Mutex::new(vec![])),
            mixed_content_effects: Arc::new(Mutex::new(vec![])),
            local_network_access_effects: Arc::new(Mutex::new(vec![])),
            effect_notifier: std::cell::RefCell::new(None),
            notifying: std::cell::Cell::new(false),
            context_detached,
        }
    }
}
impl ResourceFetcher {
    pub fn FetchRawKeepalive(
        &self,
        mut request: URLRequest,
        options: RawResourceFetchOptions,
    ) -> resource::RawResource {
        if self.document.borrow().GetDocument().RootHandle() != self.raw_fetch.arena {
            return resource::RawResource::Failed(
                "RawResource belongs to the original Document fetch context",
            );
        }
        // Binding is required: this context always holds the Document's real
        // CSP owner. Snapshot shares compiled policy AST identities and remains
        // valid for network redirects after Document/renderer disconnection.
        request.fetch = Some(self.BuildRawNetworkOptions(&request, options.clone(), true));
        request.destination = if options.context == CSPRequestContext::CSPReport {
            url_loader::RequestDestination::kReport
        } else {
            url_loader::RequestDestination::kFetch
        };
        let resource = self.engine.FetchRaw(request);
        self.NotifyRawPolicyEffects();
        resource
    }
    fn BuildRawNetworkOptions(
        &self,
        request: &URLRequest,
        options: RawResourceFetchOptions,
        keepalive: bool,
    ) -> NetworkFetchOptions {
        NetworkFetchOptions {
            mode: options.mode,
            credentials: options.credentials,
            redirect: options.redirect,
            keepalive,
            upgrade_if_insecure: request.upgrade_if_insecure,
            requestor_origin: options.requestor_origin,
            world: options.world,
            context: options.context,
            policy: self.raw_fetch.policy.Snapshot(),
            policy_violations: self.raw_fetch.violations.clone(),
            metrics: self.engine.RawResourceMetricsOwner(),
            context_detached: self.raw_fetch.context_detached.clone(),
            succeeded: Arc::new(AtomicBool::new(false)),
            failed: Arc::new(AtomicBool::new(false)),
            access_check_failed: Arc::new(AtomicBool::new(false)),
            security: options.security,
            mixed_content_effects: self.raw_fetch.mixed_content_effects.clone(),
            policy_effect_wake: None,
            local_network_access_effects: self.raw_fetch.local_network_access_effects.clone(),
        }
    }
    /// RawResource::FetchTextTrack. CORS and same-origin semantics remain in
    /// the existing Fetch/CORS transport; this is never a keepalive request.
    pub fn FetchTextTrack(
        &self,
        mut request: URLRequest,
        options: RawResourceFetchOptions,
        client: std::rc::Weak<dyn resource::TextTrackResourceClient>,
    ) -> std::rc::Rc<resource::TextTrackResource> {
        assert_eq!(
            self.document.borrow().GetDocument().RootHandle(),
            self.raw_fetch.arena,
            "text track retains the original Document fetch context"
        );
        assert_eq!(
            options.context,
            CSPRequestContext::Track,
            "original TextTrack FetchContext"
        );
        request.destination = url_loader::RequestDestination::kTrack;
        request.fetch = Some(self.BuildRawNetworkOptions(&request, options, false));
        let resource = self.engine.FetchTextTrack(request, client);
        self.NotifyRawPolicyEffects();
        resource
    }
    pub fn SetRawPolicyEffectNotifier(&self, notifier: std::rc::Rc<dyn Fn()>) {
        *self.raw_fetch.effect_notifier.borrow_mut() = Some(notifier);
    }
    fn NotifyRawPolicyEffects(&self) {
        if self.raw_fetch.notifying.replace(true) {
            return;
        }
        struct Reset<'a>(&'a std::cell::Cell<bool>);
        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _reset = Reset(&self.raw_fetch.notifying);
        let notifier = self.raw_fetch.effect_notifier.borrow().clone();
        if let Some(notifier) = notifier {
            notifier();
        }
    }
    pub fn TakeRawMixedContentEffects(&self) -> Vec<fetch_policy::RawMixedContentEffect> {
        std::mem::take(
            &mut *self
                .raw_fetch
                .mixed_content_effects
                .lock()
                .expect("mixed content effect queue poisoned"),
        )
    }
    pub fn TakeRawLocalNetworkAccessEffects(
        &self,
    ) -> Vec<fetch_policy::RawLocalNetworkAccessEffect> {
        std::mem::take(
            &mut *self
                .raw_fetch
                .local_network_access_effects
                .lock()
                .expect("LNA effects poisoned"),
        )
    }
    pub fn HasPendingRawResources(&self) -> bool {
        self.engine.HasPendingRawResources()
    }
    pub fn PollPendingRawResources(&self) -> usize {
        let completed = self.engine.PollRawResources();
        self.NotifyRawPolicyEffects();
        completed
    }
    pub fn RawResourceMetrics(&self) -> url_loader::FetchKeepaliveMetricsSnapshot {
        self.engine.RawResourceMetrics()
    }
    pub fn SetRawResourceObserver(
        &self,
        observer: Option<std::rc::Rc<dyn resource::RawResourceObserver>>,
    ) {
        self.engine.SetRawResourceObserver(observer)
    }
    /// Immutable output effects from both initial and redirected CSP checks.
    /// The original Document/reporting owner consumes them on its next turn.
    pub fn TakeRawResourcePolicyDeliveryEffects(
        &self,
    ) -> Vec<foundation::content_security_policy::CSPDeliveryEffect> {
        self.TakeRawResourcePolicyViolations()
            .into_iter()
            .flat_map(|violation| self.raw_fetch.policy.PrepareViolationDelivery(violation))
            .collect()
    }
    /// Other source fetch clients (for example HTMLMediaElement) report through
    /// this Document's original CSP delivery queue and task consumer too.
    pub fn QueueContentSecurityPolicyViolations(&self, violations: Vec<CSPViolation>) {
        self.raw_fetch
            .violations
            .lock()
            .expect("CSP effect queue poisoned")
            .extend(violations);
        self.NotifyRawPolicyEffects();
    }
    pub fn TakeRawResourcePolicyViolations(&self) -> Vec<CSPViolation> {
        std::mem::take(
            &mut *self
                .raw_fetch
                .violations
                .lock()
                .expect("CSP effect queue poisoned"),
        )
    }
}

impl ResourceFetcher {
    // Shared FetchContext security inputs; an image remains a native image
    // resource, never RawResource and never counted as keepalive.
    pub(super) fn ConfigureImageNetworkRequest(
        &self,
        request: &mut URLRequest,
        security: Arc<fetch_policy::RawFetchSecurityContext>,
        mode: FetchMode,
        credentials: FetchCredentialsMode,
        context: CSPRequestContext,
        requestor_origin: FetchOrigin,
    ) -> Result<(), &'static str> {
        self.ConfigureSubresourceNetworkRequest(
            request,
            security,
            mode,
            credentials,
            context,
            requestor_origin,
            CSPWorld::Main,
        )
    }
    pub(super) fn ConfigureSubresourceNetworkRequest(
        &self,
        request: &mut URLRequest,
        security: Arc<fetch_policy::RawFetchSecurityContext>,
        mode: FetchMode,
        credentials: FetchCredentialsMode,
        context: CSPRequestContext,
        requestor_origin: FetchOrigin,
        world: CSPWorld,
    ) -> Result<(), &'static str> {
        use foundation::content_security_policy::{
            CSPCheckHeaderType, CSPRedirectStatus, CSPReportingDisposition, CSPRequest,
        };
        let policy = self.raw_fetch.policy.Snapshot();
        let autoupgrade = context == CSPRequestContext::Image
            && fetch_policy::ShouldAutoupgradeImage(security.as_ref(), context, &request.url);
        let prepared = policy.PrepareRequestURL(&request.url);
        request.url = prepared.url;
        request.upgrade_if_insecure |= prepared.upgrade_if_insecure || autoupgrade;
        if autoupgrade && !prepared.upgrade_if_insecure {
            if let Some(upgraded) = fetch_policy::UpgradeInsecureURL(policy.as_ref(), &request.url)
            {
                request.url = upgraded;
                request.is_automatic_upgrade = true;
            }
        }
        let mixed = fetch_policy::CheckRawMixedContent(
            security.as_ref(),
            policy.as_ref(),
            &request.url,
            &request.url,
            context,
            CSPRedirectStatus::NoRedirect,
            prepared.block_all_mixed_content,
        );
        if let Some(effect) = mixed.effect {
            self.raw_fetch
                .mixed_content_effects
                .lock()
                .expect("mixed effects")
                .push(effect);
        }
        if mixed.blocked.is_some() {
            return Err("subresource blocked by MixedContentChecker");
        }
        let verdict = policy.AllowRequest(&CSPRequest {
            url: &request.url,
            url_before_redirects: &request.url,
            context,
            world,
            redirect: CSPRedirectStatus::NoRedirect,
            reporting: CSPReportingDisposition::Report,
            check_header_type: CSPCheckHeaderType::All,
        });
        self.QueueContentSecurityPolicyViolations(verdict.violations);
        if !verdict.allowed {
            return Err("subresource blocked by Content Security Policy");
        }
        request.fetch = Some(NetworkFetchOptions {
            mode,
            credentials,
            redirect: FetchRedirectMode::Follow,
            keepalive: false,
            upgrade_if_insecure: request.upgrade_if_insecure,
            requestor_origin,
            world,
            context,
            policy,
            policy_violations: self.raw_fetch.violations.clone(),
            metrics: self.engine.RawResourceMetricsOwner(),
            context_detached: self.raw_fetch.context_detached.clone(),
            succeeded: Arc::new(AtomicBool::new(false)),
            failed: Arc::new(AtomicBool::new(false)),
            access_check_failed: Arc::new(AtomicBool::new(false)),
            security,
            mixed_content_effects: self.raw_fetch.mixed_content_effects.clone(),
            local_network_access_effects: self.raw_fetch.local_network_access_effects.clone(),
            policy_effect_wake: None,
        });
        Ok(())
    }
}

impl ResourceFetcher {
    /// ResourceFetcher.cc2926-2928,3172-3201. Native clients are notified
    /// synchronously by the same request's cancellation before this returns.
    pub fn BindFetcherTaskRunner(&self, runner: std::rc::Rc<dyn resource::FetcherTaskRunner>) {
        self.engine.BindFetcherTaskRunner(runner);
    }
    pub fn XMLHttpRequestRegistry(&self) -> std::rc::Rc<resource::XMLHttpRequestRegistry> {
        self.engine.XMLHttpRequestRegistry()
    }
    pub fn BlockingRequestCount(&self) -> usize {
        self.engine.BlockingRequestCount()
    }
    pub fn IsInRequestResource(&self) -> bool {
        self.engine.IsInRequestResource()
    }
    pub fn StopFetching(&self) -> std::io::Result<()> {
        self.engine.StopFetching()
    }
    /// ResourceFetcher.cc2343-2390. Call at Document::Shutdown, after the
    /// original loader's StopFetching and old-realm microtask checkpoint.
    pub fn ClearContext(&self) -> std::io::Result<()> {
        self.raw_fetch.effect_notifier.borrow_mut().take();
        self.engine.ClearFetchContext()
    }
}

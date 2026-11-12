//! Native loader, FrameTree, network and task leaves for HTMLAnchorElement.
//! Source hyperlink/download/ping policy remains in interaction. Browser
//! routes the same FrameLoadRequest and resumes target creation only after the
//! embedding has admitted the actual auxiliary context.
use super::*;
use interaction::default::core::{
    html::{
        anchor_element_utils::{
            self, AnchorElementUtilsEnvironment, AnchorNavigationApi, DownloadNavigateEventParams,
            NavigateDispatchResult,
        },
        html_anchor_element::*,
    },
    loader::{
        navigation_policy::{NavigationEventModifiers, NavigationPolicy},
        resource_request::{RequestContext, ResourceRequest},
    },
};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_AUXILIARY_TARGET: AtomicU64 = AtomicU64::new(1);

pub(super) fn HostMetadata(
    request: &FrameLoadRequest,
    auxiliary_target: Option<u64>,
) -> NavigationHostMetadata {
    NavigationHostMetadata {
        policy: request.navigation_policy.get(),
        auxiliary_target,
        noopener: request.noopener.get(),
        explicit_opener: request.explicit_opener.get(),
        never_send_referrer: request.never_send_referrer.get(),
        href_translate: request.href_translate.borrow().clone(),
        suggested_filename: request.resource.Read(|r| r.suggested_filename.clone()),
        triggering_event_info: request.triggering_event_info.get(),
        input_start_time: request.input_start_time.get(),
        client_navigation_reason: request.client_navigation_reason.get(),
        initiator: Some(request.initiator.clone()),
        blob_url_token: request.blob_url_token.borrow().clone(),
    }
}
fn ResourceURLRequest(request: &ResourceRequest) -> URLRequest {
    request.Read(|r| URLRequest {
        url: r.url.clone(),
        referrer: r.referrer.clone(),
        referrer_policy: Some(LoaderReferrerPolicy(r.referrer_policy)),
        method: r.method.clone(),
        encoded_body: r.body.clone(),
        headers: r.headers.clone(),
        ..Default::default()
    })
}
pub(in crate::page) struct NativeAnchorNavigationHost {
    self_: Weak<Self>,
    domain: Weak<super::super::local_frame::LocalFrameDomain>,
    token: u64,
    pending_anchor_targets: RefCell<HashMap<u64, Rc<PendingAuxiliaryNavigation>>>,
}
impl NativeAnchorNavigationHost {
    pub(in crate::page) fn new(
        domain: &Rc<super::super::local_frame::LocalFrameDomain>,
        token: u64,
    ) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            domain: Rc::downgrade(domain),
            token,
            pending_anchor_targets: Default::default(),
        })
    }
    fn Retain(&self) -> Rc<Self> {
        self.self_.upgrade().expect("retained Frame loader service")
    }
    fn FrameDomain(&self) -> Option<Rc<super::super::local_frame::LocalFrameDomain>> {
        self.domain.upgrade()
    }
    fn FrameToken(&self) -> Option<u64> {
        self.FrameDomain()?.Document(self.token)?;
        Some(self.token)
    }
    fn Page(&self) -> Option<Rc<PageState>> {
        self.FrameDomain()?.page.upgrade()
    }
    fn Document(&self) -> Rc<DocumentEngine> {
        self.FrameDomain()
            .and_then(|d| d.Document(self.token))
            .expect("live Frame Document")
    }
    fn Engine(&self) -> Option<interaction::Interaction<'static>> {
        self.FrameDomain()?.FocusInteraction(self.token)
    }
    fn WindowEnvironment(&self) -> Option<Rc<NavigationWindowEnvironment>> {
        let domain = self.FrameDomain()?;
        if self.token == domain.root {
            return domain.root_navigation.borrow().upgrade().filter(|window| {
                window.Live() && Rc::ptr_eq(&window.Document(), &self.Document())
            });
        }
        let frame = domain.Frame(self.token)?;
        let window = frame.navigation.borrow().window.clone()?;
        let navigation = window.navigation.clone();
        (navigation.Live() && Rc::ptr_eq(&navigation.Document(), &self.Document()))
            .then_some(navigation)
    }
    fn WindowBindings(&self) -> Option<Rc<RefCell<WindowJavaScriptBindings>>> {
        self.WindowEnvironment()?.window.upgrade()
    }
    fn IsRoot(&self) -> bool {
        self.FrameDomain().is_some_and(|d| d.root == self.token)
    }
    fn Live(&self) -> bool {
        self.FrameDomain()
            .and_then(|d| d.Document(self.token))
            .is_some_and(|doc| doc.borrow().GetDocument().IsActive())
    }
    fn URL(&self) -> String {
        self.Document()
            .borrow()
            .GetDocument()
            .DocumentURL()
            .to_owned()
    }
    fn Complete(&self, url: &str) -> Option<String> {
        self.Document()
            .borrow()
            .GetDocument()
            .BaseURL()
            .CompleteURL(url)
            .map(|u| u.into())
    }
    fn Active(&self) -> bool {
        self.Engine()
            .is_some_and(|e| e.HasTransientUserActivation())
    }
    fn SameOrigin(&self, url: &str) -> bool {
        let doc = self.Document();
        let doc = doc.borrow();
        let origin = doc.GetDocument().SecurityContext().Origin();
        let destination = dom::persistent_document::SecurityOrigin::CreateForNavigation(
            url,
            None,
            Rc::new(origin.SchemeRegistryForRequests()),
        );
        origin.IsSameOriginWith(&destination)
    }
    fn Loader(&self) -> Rc<document_loader::DocumentLoaderNavigationState> {
        let domain = self.FrameDomain().expect("live FrameTree");
        if self.token == domain.root {
            return self.Page().expect("live Page").DocumentLoaderNavigation();
        }
        domain
            .Frame(self.token)
            .and_then(|f| {
                f.runtime
                    .borrow()
                    .as_ref()
                    .and_then(|r| r.navigation_state.get().cloned())
            })
            .expect("Frame DocumentLoader precedes navigation")
    }
    fn RequestCrossDocument(
        &self,
        request: Rc<FrameLoadRequest>,
        load: FrameLoadType,
        _history: Option<Rc<HistoryItem>>,
    ) {
        let request = self.AnchorRequest(&request, NavigationTarget::SelfFrame, None);
        let Some(domain) = self.FrameDomain() else {
            return;
        };
        if self.token == domain.root {
            if let Some(page) = self.Page() {
                page.client.borrow_mut().DidRequestNavigation(&request);
            }
        } else if let Some(frame) = domain.Frame(self.token) {
            frame.RequestScriptNavigationRequest(request.request, load);
        }
    }
    fn AnchorRequest(
        &self,
        request: &FrameLoadRequest,
        target: NavigationTarget,
        auxiliary_target: Option<u64>,
    ) -> NavigationRequest {
        let mut resource = ResourceURLRequest(&request.resource);
        resource.navigation = Some(navigation_history::NavigationRequestMetadata {
            load_type: FrameLoadType::Standard,
            navigation_type: navigation_history::WebNavigationType::Determine(
                FrameLoadType::Standard,
                false,
                false,
            ),
            is_browser_initiated: false,
            had_transient_user_activation: request.resource.Read(|r| r.has_user_gesture),
            requestor_origin: Some(request.initiator.origin.clone()),
            propagated_text_fragment_token: None,
            internal_scroll_to_text_fragment: None,
            history_item: None,
            back_entries: Arc::from([]),
            forward_entries: Arc::from([]),
            navigation_api_state: None,
        });
        NavigationRequest {
            request: resource,
            target,
            replace_history: false,
            host: Some(HostMetadata(request, auxiliary_target)),
        }
    }
    fn PendingAuxiliary(
        &self,
        request: Rc<FrameLoadRequest>,
        target: NavigationTarget,
    ) -> Rc<PendingAuxiliaryNavigation> {
        request.SetFrameType(true);
        let id = NEXT_AUXILIARY_TARGET.fetch_add(1, Ordering::Relaxed);
        let pending = Rc::new(PendingAuxiliaryNavigation {
            environment: self.self_.clone(),
            document: Rc::downgrade(&self.Document()),
            document_token: self.Document().borrow().GetDocument().Token(),
            id,
            request: self.AnchorRequest(&request, target.clone(), Some(id)),
            target,
            continuation: RefCell::new(None),
        });
        self.pending_anchor_targets
            .borrow_mut()
            .insert(id, pending.clone());
        pending
    }
    fn CompleteAuxiliary(&self, id: u64, accepted: bool) -> bool {
        let pending = self.pending_anchor_targets.borrow_mut().remove(&id);
        let Some(pending) = pending else {
            return false;
        };
        let continuation = pending.continuation.borrow_mut().take();
        if let Some(continuation) = continuation {
            let current = self.Live()
                && pending.document.upgrade().is_some_and(|document| {
                    Rc::ptr_eq(&document, &self.Document())
                        && document.borrow().GetDocument().Token() == pending.document_token
                });
            continuation((accepted && current).then(|| {
                Rc::new(AuxiliaryNavigationTarget {
                    environment: self.self_.clone(),
                    id,
                    target: pending.target.clone(),
                }) as Rc<dyn AnchorNavigationTarget>
            }));
        }
        true
    }
}
impl super::super::local_frame::LocalFrameDomain {
    pub(in crate::page) fn AnchorNavigationHost(
        self: &Rc<Self>,
        token: u64,
    ) -> Option<Rc<NativeAnchorNavigationHost>> {
        self.Document(token)?;
        if token == self.root {
            return Some(
                self.root_anchor_navigation
                    .get_or_init(|| NativeAnchorNavigationHost::new(self, token))
                    .clone(),
            );
        }
        let frame = self.Frame(token)?;
        Some(
            frame
                .anchor_navigation
                .get_or_init(|| NativeAnchorNavigationHost::new(self, token))
                .clone(),
        )
    }
}
impl NavigationWindowEnvironment {
    pub(super) fn AnchorLoaderHost(&self) -> Rc<NativeAnchorNavigationHost> {
        let domain = self
            .page
            .borrow()
            .upgrade()
            .and_then(|page| page.local_frames.get().cloned())
            .expect("live Window FrameTree");
        domain
            .AnchorNavigationHost(
                self.frame
                    .upgrade()
                    .map_or(domain.root, |frame| frame.state.Identity()),
            )
            .expect("Window Frame loader service")
    }
}
impl FrameLoadRequestEnvironment for NativeAnchorNavigationHost {
    fn InitiatorSecurityOriginOwner(&self) -> Rc<dyn foundation::security::SecurityOriginOwner> {
        self.Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .clone()
    }
    fn Now(&self) -> TimeTicks {
        self.Page()
            .expect("Frame clock")
            .InteractionEventClock()
            .Now()
    }
    fn CurrentInputStartTime(&self) -> Option<TimeTicks> {
        super::super::native_event_handler_host::CurrentInputStartTime()
    }
    fn BaseURL(&self) -> String {
        self.Document()
            .borrow()
            .GetDocument()
            .BaseURL()
            .AsStr()
            .to_owned()
    }
    fn OutgoingReferrer(&self) -> String {
        self.URL()
    }
    fn DocumentReferrerPolicy(&self) -> ReferrerPolicy {
        self.Loader()
            .CurrentHistoryItem()
            .map_or(ReferrerPolicy::Default, |i| i.ReferrerPolicy())
    }
    fn GenerateReferrer(&self, policy: ReferrerPolicy, url: &str, source: &str) -> String {
        LoaderReferrerPolicy(policy).GenerateReferrer(source, url)
    }
    fn HasTransientUserActivation(&self) -> bool {
        self.Active()
    }
    fn RemoveDanglingMarkupInTargetEnabled(&self) -> bool {
        true
    }
    fn TargetBlankImpliesNoOpener(&self) -> bool {
        true
    }
    fn RelOpenerBcgDependencyHintEnabled(&self) -> bool {
        false
    }
    fn BlobURLSiteDiffersFromTopLevelSite(&self, url: &str) -> bool {
        let document = self.Document();
        let document = document.borrow();
        let manager = document.GetDocument().SecurityContext().PublicURLManager();
        let Some(origin) = manager.OriginForURL(url) else {
            return false;
        };
        let differs = &foundation::security::SchemefulSite::FromOrigin(&origin)
            != manager.StorageKey().TopLevelSite();
        differs
    }
    fn EnforceNoopenerOnBlobURLNavigation(&self) -> bool {
        true
    }
    fn BlobURLPartitioningDisabled(&self) -> bool {
        self.Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .PublicURLManager()
            .PartitioningDisabled()
    }
    fn ResolveBlobURLToken(
        &self,
        url: &str,
        top_level: bool,
    ) -> Arc<interaction::dispatch::core::fileapi::blob_url_registry::BlobURLToken> {
        let document = self.Document();
        let document = document.borrow();
        let manager = document.GetDocument().SecurityContext().PublicURLManager();
        manager
            .ResolveAsBlobURLToken(
                url,
                top_level,
                true,
                manager.PartitioningDisabled(),
                manager.HasStorageAccess(),
                dom::navigation_tokens::CreateNavigationToken(),
            )
            .0
    }
    fn CountUse(&self, _feature: &'static str) {}
    fn ReportPartitioningBlobURLIssue(&self, url: &str) {
        browser_tracing::instant_with_text("navigation", "PartitioningBlobURLIssue", url, &[]);
    }
}
type TargetContinuation = Box<dyn FnOnce(Option<Rc<dyn AnchorNavigationTarget>>)>;
pub(super) struct PendingAuxiliaryNavigation {
    environment: Weak<NativeAnchorNavigationHost>,
    document: Weak<DocumentEngine>,
    document_token: [u64; 2],
    id: u64,
    request: NavigationRequest,
    target: NavigationTarget,
    continuation: RefCell<Option<TargetContinuation>>,
}
impl AnchorNavigationPending for PendingAuxiliaryNavigation {
    fn Resume(&self, continuation: TargetContinuation) {
        assert!(self
            .continuation
            .borrow_mut()
            .replace(continuation)
            .is_none());
        if let Some(environment) = self.environment.upgrade().filter(|e| e.Live()) {
            if let Some(page) = environment.Page() {
                page.client
                    .borrow_mut()
                    .DidRequestAuxiliaryNavigationTarget(&AuxiliaryNavigationTargetRequest {
                        request_id: self.id,
                        request: self.request.clone(),
                    });
            }
        }
    }
}
struct AuxiliaryNavigationTarget {
    environment: Weak<NativeAnchorNavigationHost>,
    id: u64,
    target: NavigationTarget,
}
impl AnchorNavigationTarget for AuxiliaryNavigationTarget {
    fn IsCurrentFrame(&self) -> bool {
        false
    }
    fn NavigateStandard(&self, request: Rc<dyn AnchorFrameLoadRequest>) {
        let Ok(request) = request.IntoAny().downcast::<FrameLoadRequest>() else {
            return;
        };
        let Some(environment) = self.environment.upgrade().filter(|e| e.Live()) else {
            return;
        };
        let Some(page) = environment.Page() else {
            return;
        };
        let request = environment.AnchorRequest(&request, self.target.clone(), Some(self.id));
        page.client.borrow_mut().DidRequestNavigation(&request);
    }
}
struct LocalNavigationTarget {
    source: Weak<NativeAnchorNavigationHost>,
    destination: Weak<NativeAnchorNavigationHost>,
    document: Weak<DocumentEngine>,
    current: bool,
}
impl AnchorNavigationTarget for LocalNavigationTarget {
    fn IsCurrentFrame(&self) -> bool {
        self.current
    }
    fn NavigateStandard(&self, request: Rc<dyn AnchorFrameLoadRequest>) {
        self.NavigateStandardWithInvocation(request, None);
    }
    fn NavigateStandardWithInvocation(
        &self,
        request: Rc<dyn AnchorFrameLoadRequest>,
        invocation: Option<&dyn ScriptInvocation>,
    ) {
        let Ok(request) = request.IntoAny().downcast::<FrameLoadRequest>() else {
            return;
        };
        let Some(destination) = self.destination.upgrade().filter(|e| e.Live()) else {
            return;
        };
        let Some(document) = self
            .document
            .upgrade()
            .filter(|doc| Rc::ptr_eq(doc, &destination.Document()))
        else {
            return;
        };
        if request.navigation_policy.get() != NavigationPolicy::kNavigationPolicyCurrentTab {
            let Some(source) = self.source.upgrade().filter(|e| e.Live()) else {
                return;
            };
            if request.navigation_policy.get() == NavigationPolicy::kNavigationPolicyDownload {
                if let Some(page) = source.Page() {
                    page.client
                        .borrow_mut()
                        .DidRequestNavigation(&source.AnchorRequest(
                            &request,
                            NavigationTarget::SelfFrame,
                            None,
                        ));
                }
                return;
            }
            let pending = source.PendingAuxiliary(request.clone(), NavigationTarget::NewContext);
            pending.Resume(Box::new(move |target| {
                if let Some(target) = target {
                    target.NavigateStandard(request);
                }
            }));
            return;
        }
        request.SetFrameType(destination.IsRoot());
        if let (Some(invocation), Some(window)) = (invocation, destination.WindowEnvironment()) {
            NavigationApiEnvironment::Navigate(
                window.as_ref(),
                invocation,
                request,
                FrameLoadType::Standard,
            );
        } else {
            destination.RequestCrossDocument(request, FrameLoadType::Standard, None);
        }
        drop(document);
    }
}
struct DelayedNavigationTask(Rc<RefCell<Option<Box<dyn FnOnce()>>>>);
impl AnchorNavigationTaskHandle for DelayedNavigationTask {}
impl Drop for DelayedNavigationTask {
    fn drop(&mut self) {
        self.0.borrow_mut().take();
    }
}
impl AnchorNavigationHost for NativeAnchorNavigationHost {
    fn ServerMapContentPoint(
        &self,
        target: u64,
        absolute: &foundation::gfx::PointF,
    ) -> Option<(f32, f32, f32)> {
        if let Some(frame) = self.FrameDomain().and_then(|d| d.Frame(self.token)) {
            let runtime = frame.runtime.borrow().clone()?;
            let layout = runtime.layout.borrow();
            layout
                .Output()?
                .Node(target)?
                .ServerMapContentPoint(absolute)
        } else {
            let page = self.Page()?;
            page.EnsureLayoutOutputForNode(target);
            let layout = page.layout_engine.borrow();
            layout
                .Output()?
                .Node(target)?
                .ServerMapContentPoint(absolute)
        }
    }
    fn CurrentInputNavigationModifiers(&self) -> Option<NavigationEventModifiers> {
        super::super::native_event_handler_host::CurrentInputNavigationModifiers()
    }
    fn NewTabModifierSetFromIsolatedWorld(&self) -> bool {
        interaction::dispatch::core::events::ui_event_with_key_state::UIEventWithKeyState::NewTabModifierSetFromIsolatedWorld()
    }
    fn SendPings(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        completed_url: &str,
        ping_attribute: Option<&str>,
    ) {
        anchor_element_utils::SendPings(self, invocation, completed_url, ping_attribute);
    }
    fn CanReadContent(&self, completed_url: &str) -> bool {
        self.Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .CanReadContent(completed_url)
    }
    fn HandleDownloadAttribute(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        source: EventTargetRef,
        download: &str,
        completed_url: &str,
        is_trusted: bool,
        request: Rc<dyn AnchorResourceRequest>,
    ) {
        let Ok(request) = request.IntoAny().downcast::<ResourceRequest>() else {
            return;
        };
        anchor_element_utils::HandleDownloadAttributeWithInvocation(
            invocation,
            self,
            source,
            download,
            completed_url,
            is_trusted,
            request,
        );
    }
    fn PostDelayedNavigationTask(
        &self,
        task: Box<dyn FnOnce()>,
    ) -> Rc<dyn AnchorNavigationTaskHandle> {
        let document = Rc::downgrade(&self.Document());
        let token = self.Document().borrow().GetDocument().Token();
        let task = Rc::new(RefCell::new(Some(Box::new(move || {
            if document.upgrade().is_some_and(|document| {
                let document = document.borrow();
                document.GetDocument().IsActive() && document.GetDocument().Token() == token
            }) {
                task();
            }
        }) as Box<dyn FnOnce()>)));
        if let Some(window) = self.WindowBindings() {
            let scheduled = task.clone();
            // ui/events/event_constants.h::kDoubleClickTimeMs.
            window.borrow_mut().EnqueueDelayedNativeTask(
                Duration::from_millis(500),
                Box::new(move || {
                    let task = scheduled.borrow_mut().take();
                    if let Some(task) = task {
                        task();
                    }
                }),
            );
        } else if let Some(page) = self.Page() {
            page.popup_tasks.Post(
                page.document.borrow().GetDocument().RootHandle(),
                0.5,
                super::super::popup_tasks::PageInteractionTask::AnchorNavigation(task.clone()),
            );
        }
        Rc::new(DelayedNavigationTask(task))
    }
    fn SetDelayedNavigationTaskHandle(&self, handle: Rc<dyn AnchorNavigationTaskHandle>) {
        if let Some(interaction) = self.Engine() {
            interaction.SetDelayedNavigationTaskHandle(handle);
        }
    }
    fn CreateHyperlinkFrameLoadRequest(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        request: Rc<dyn AnchorResourceRequest>,
    ) -> Option<Rc<dyn AnchorFrameLoadRequest>> {
        if !self.Live() {
            return None;
        }
        let request = request.IntoAny().downcast::<ResourceRequest>().ok()?;
        request.Mutate(|r| r.context = RequestContext::Hyperlink);
        let document = self.Document();
        let document = document.borrow();
        let document = document.GetDocument();
        Some(FrameLoadRequest::new(
            &request,
            self.Retain(),
            NavigationInitiator {
                origin: RequestorOriginFor(document.SecurityContext().Origin()),
                state_token: self.Loader().InitiatorStateToken(),
                document_token: Some(document.Token()),
                world_id: invocation.map_or(0, |i| i.WorldId()),
                source_location: invocation.and_then(|i| i.CaptureStackSourceLocation()),
                script_tool_invocation_id: invocation.and_then(|i| i.ScriptToolInvocationId()),
            },
        ))
    }
    fn FindOrCreateFrameForNavigation(
        &self,
        request: &Rc<dyn AnchorFrameLoadRequest>,
        target: &str,
    ) -> AnchorNavigationLookup {
        let Ok(request) = request.clone().IntoAny().downcast::<FrameLoadRequest>() else {
            return AnchorNavigationLookup::Denied;
        };
        let Some(domain) = self.FrameDomain() else {
            return AnchorNavigationLookup::Denied;
        };
        let Some(source) = self.FrameToken() else {
            return AnchorNavigationLookup::Denied;
        };
        let target_name = NavigationTarget::from(target);
        let sandbox = self
            .Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .SandboxFlags();
        let token = if request.NavigationUsesCurrentFrame(sandbox) {
            Some(source)
        } else {
            domain.ResolveNavigationTarget(source, &target_name)
        };
        let Some(token) = token else {
            return AnchorNavigationLookup::Pending(self.PendingAuxiliary(request, target_name));
        };
        let url = request.resource.Read(|r| r.url.clone());
        let Ok(parsed) = url::Url::parse(&url) else {
            return AnchorNavigationLookup::Denied;
        };
        let source_origin = self
            .Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .clone();
        let destination_origin = dom::persistent_document::SecurityOrigin::CreateForNavigation(
            &url,
            None,
            Rc::new(source_origin.SchemeRegistryForRequests()),
        );
        if !interaction::default::core::frame::local_frame_navigation::CanNavigate(
            &NativeNavigationContext {
                host: self,
                domain: domain.clone(),
            },
            &token,
            &interaction::default::core::frame::local_frame_navigation::NavigationDestination {
                origin: destination_origin,
                protocol: parsed.scheme(),
                host: parsed.host_str().unwrap_or(""),
            },
        ) {
            return AnchorNavigationLookup::Denied;
        }
        let Some(destination) = domain.AnchorNavigationHost(token).filter(|e| e.Live()) else {
            return AnchorNavigationLookup::Denied;
        };
        AnchorNavigationLookup::Ready(Rc::new(LocalNavigationTarget {
            source: self.self_.clone(),
            destination: Rc::downgrade(&destination),
            document: Rc::downgrade(&destination.Document()),
            current: token == source,
        }))
    }
    fn HrefTranslateEnabled(&self) -> bool {
        false
    }
}
struct NativeNavigationContext<'a> {
    host: &'a NativeAnchorNavigationHost,
    domain: Rc<super::super::local_frame::LocalFrameDomain>,
}
impl interaction::default::core::frame::local_frame_navigation::LocalFrameNavigationContext
    for NativeNavigationContext<'_>
{
    type Frame = u64;
    fn SourceFrame(&self) -> u64 {
        self.host.token
    }
    fn Parent(&self, frame: &u64) -> Option<u64> {
        self.domain.Frame(*frame).map(|frame| frame.parent)
    }
    // The native FrameTree currently has no cross-Page opener Frame owner.
    // Auxiliary context creation and its opener policy stay with the embedding.
    fn Opener(&self, _frame: &u64) -> Option<u64> {
        None
    }
    fn IsInSamePage(&self, a: &u64, b: &u64) -> bool {
        self.domain.Document(*a).is_some() && self.domain.Document(*b).is_some()
    }
    fn SecurityOrigin(&self, frame: &u64) -> Rc<dyn foundation::security::SecurityOriginOwner> {
        self.domain
            .Document(*frame)
            .expect("live target Frame")
            .borrow()
            .GetDocument()
            .SecurityContext()
            .Origin()
            .clone()
    }
    fn IsMainFrame(&self, frame: &u64) -> bool {
        *frame == self.domain.root
    }
    fn IsPageMainFrameFencedFrameRoot(&self, _frame: &u64) -> bool {
        false
    }
    fn IsInFencedFrameTree(&self) -> bool {
        false
    }
    fn SandboxFlags(&self) -> foundation::security::SandboxFlags {
        self.host
            .Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .SandboxFlags()
    }
    fn HasTransientUserActivation(&self) -> bool {
        self.host.Active()
    }
    fn HasStickyUserActivation(&self) -> bool {
        self.host
            .Engine()
            .is_some_and(|e| e.HasStickyUserActivation())
    }
    fn CanNavigateTopWithoutUserGesture(&self) -> bool {
        self.host
            .Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .CanNavigateTopWithoutUserGesture()
    }
    fn AllowPopup(&self) -> bool {
        self.host
            .Page()
            .is_some_and(|page| page.client.borrow().AllowsPopupNavigation())
    }
    fn CountOpenerNavigationWithoutGesture(&self) {
        browser_tracing::instant("navigation", "OpenerNavigationWithoutGesture", &[]);
    }
    fn ReportNavigationError(
        &self,
        target: &u64,
        reason: interaction::default::core::frame::local_frame_navigation::NavigationError,
    ) {
        browser_tracing::instant_with_text(
            "navigation",
            "NavigationError",
            reason.Message(),
            &[
                ("source_frame", self.host.token as f64),
                ("target_frame", *target as f64),
            ],
        );
    }
    fn DidBlockNavigation(
        &self,
        reason: interaction::default::core::frame::local_frame_navigation::NavigationBlockedReason,
    ) {
        browser_tracing::instant_with_text(
            "navigation",
            "DidBlockNavigation",
            &format!("{reason:?}"),
            &[("source_frame", self.host.token as f64)],
        );
    }
}
impl AnchorNavigationApi for NativeAnchorNavigationHost {
    fn DispatchNavigateEvent(
        &self,
        invocation: &dyn ScriptInvocation,
        params: DownloadNavigateEventParams,
    ) -> NavigateDispatchResult {
        let mut event = NavigateEventDispatchParams::new(
            params.url,
            NavigateEventType::CrossDocument,
            FrameLoadType::Standard,
        );
        *event.source_element.borrow_mut() = Some(params.source_element);
        event.download_filename = Some(params.download_filename);
        event.involvement = if params.activation {
            UserNavigationInvolvement::Activation
        } else {
            UserNavigationInvolvement::None
        };
        self.WindowEnvironment()
            .expect("script invocation Window")
            .Api()
            .Dispatch(invocation, Rc::new(event))
    }
    fn InformAboutCanceledNavigationDropped(&self, invocation: &dyn ScriptInvocation) {
        self.WindowEnvironment().expect("script invocation Window").Api().InformAboutCanceledNavigation(invocation, interaction::default::core::navigation_api::navigation_api::CancelNavigationReason::Dropped);
    }
}
impl AnchorElementUtilsEnvironment for NativeAnchorNavigationHost {
    fn HasTransientUserActivation(&self) -> bool {
        self.Active()
    }
    fn IsAdFrame(&self) -> bool {
        self.Document().borrow().GetDocument().IsAdFrame()
    }
    fn IsAdScriptInStack(&self) -> bool {
        false
    }
    fn SandboxedDownloads(&self) -> bool {
        self.Document()
            .borrow()
            .GetDocument()
            .SecurityContext()
            .IsSandboxed(foundation::security::SandboxFlags::DOWNLOADS)
    }
    fn RequestorOrigin(&self) -> RequestorOrigin {
        RequestorOriginFor(
            self.Document()
                .borrow()
                .GetDocument()
                .SecurityContext()
                .Origin(),
        )
    }
    fn DocumentReferrerPolicy(&self) -> ReferrerPolicy {
        FrameLoadRequestEnvironment::DocumentReferrerPolicy(self)
    }
    fn OutgoingReferrer(&self) -> String {
        self.URL()
    }
    fn GenerateReferrer(&self, policy: ReferrerPolicy, url: &str, source: &str) -> String {
        FrameLoadRequestEnvironment::GenerateReferrer(self, policy, url, source)
    }
    fn NavigationApi(&self) -> &dyn AnchorNavigationApi {
        self
    }
    fn DownloadURL(&self, request: Rc<ResourceRequest>) {
        let Some(page) = self.Page() else {
            return;
        };
        page.client.borrow_mut().DidRequestNavigation(&NavigationRequest {
            request: ResourceURLRequest(&request), target: NavigationTarget::SelfFrame, replace_history: false,
            host: Some(NavigationHostMetadata {
                policy: NavigationPolicy::kNavigationPolicyDownload, auxiliary_target: None,
                noopener: false, explicit_opener: false, never_send_referrer: request.Read(|r| r.referrer_policy == ReferrerPolicy::Never),
                href_translate: None, suggested_filename: request.Read(|r| r.suggested_filename.clone()),
                triggering_event_info: interaction::default::core::loader::frame_load_request::TriggeringEventInfo::NotFromEvent,
                input_start_time: TimeTicks::default(), client_navigation_reason: interaction::default::core::loader::frame_load_request::ClientNavigationReason::AnchorClick,
                initiator: None, blob_url_token: None,
            }),
        });
    }
    fn HyperlinkAuditingEnabled(&self) -> bool {
        true
    }
    fn HasArchive(&self) -> bool {
        false
    }
    fn CompleteURL(&self, url: &str) -> String {
        self.Complete(url).unwrap_or_default()
    }
    fn DocumentURL(&self) -> String {
        self.URL()
    }
    fn IsHTTPFamily(&self, url: &str) -> bool {
        url::Url::parse(url).is_ok_and(|u| matches!(u.scheme(), "http" | "https"))
    }
    fn IsHTTP(&self, url: &str) -> bool {
        url::Url::parse(url).is_ok_and(|u| u.scheme() == "http")
    }
    fn CanAccessPingOrigin(&self, url: &str) -> bool {
        self.SameOrigin(url)
    }
    fn FetchLinkAuditPing(
        &self,
        invocation: Option<&dyn ScriptInvocation>,
        request: Rc<ResourceRequest>,
    ) {
        let Some(domain) = self.FrameDomain() else {
            return;
        };
        let Some(resources) = self.FrameToken().and_then(|token| domain.Resources(token)) else {
            return;
        };
        let document = self.Document();
        let document = document.borrow();
        let document = document.GetDocument();
        let origin = document.SecurityContext().Origin();
        let requestor_origin = match origin.Tuple() {
            Some(tuple) => url_loader::FetchOrigin::Tuple {
                scheme: tuple.scheme.clone(),
                host: tuple.host.clone(),
                port: tuple.port,
            },
            None => url_loader::FetchOrigin::Opaque(
                origin
                    .OpaqueNonce()
                    .expect("original opaque request origin"),
            ),
        };
        let mut security = FrameFetchSecurity(document);
        if self.token != domain.root {
            let top = domain.Document(domain.root).expect("live top Frame");
            let top = top.borrow();
            let top_security = FrameFetchSecurity(top.GetDocument());
            let security = Arc::get_mut(&mut security).expect("new request security inputs");
            security.current_frame_is_top = false;
            security.top_frame = top_security.current_frame.clone();
        }
        resources.FetchRawKeepalive(
            ResourceURLRequest(&request),
            document_loader::RawResourceFetchOptions {
                security,
                context: foundation::content_security_policy::CSPRequestContext::Ping,
                world: invocation
                    .filter(|i| i.WorldId() != 0)
                    .map_or(foundation::content_security_policy::CSPWorld::Main, |i| {
                        foundation::content_security_policy::CSPWorld::Isolated(i.WorldIdentity())
                    }),
                requestor_origin,
                mode: url_loader::FetchMode::NoCors,
                credentials: url_loader::FetchCredentialsMode::Include,
                redirect: url_loader::FetchRedirectMode::Follow,
            },
        );
    }
    fn CountUse(&self, _feature: &'static str) {}
    fn CountDeprecation(&self, _feature: &'static str) {}
    fn DownloadAttributeTooLong(&self, _length: usize) {}
}
impl Page {
    pub fn CompleteAuxiliaryNavigationTarget(&self, request_id: u64, accepted: bool) {
        let Some(domain) = self.state.local_frames.get() else {
            return;
        };
        if domain
            .AnchorNavigationHost(domain.root)
            .is_some_and(|root| root.CompleteAuxiliary(request_id, accepted))
        {
            return;
        }
        let frames = domain.frames.borrow().keys().copied().collect::<Vec<_>>();
        for token in frames {
            if domain
                .AnchorNavigationHost(token)
                .is_some_and(|host| host.CompleteAuxiliary(request_id, accepted))
            {
                return;
            }
        }
    }
}

//! LocalFrame's DocumentLoader/LocalDOMWindow assembly. Documents retain their
//! own parser, bindings, globals and task queues; the Page supplies the Agent's
//! one JavaScript runtime and drives all Frames from its existing task turn.
//! Source: frame_loader.cc::CommitDocumentLoader, document_loader.cc::InitializeWindow,
//! local_dom_window.cc::InstallNewDocument, script_controller.cc::WindowProxy.
use super::local_frame::{LocalFrame, LocalFrameDomain};
use super::*;
use dom::content_frames::ContentFrameSource;
use dom::frame_owner_loading::{FrameOwnerLoadEffect, FrameOwnerLoadError, FrameOwnerNavigation};

use javascript::script_engine::{ScriptExecutor, ScriptExecutorHandle};
use open::{OpenDocumentAdapter, OpenDocumentPipeline, OpenDocumentServices, OpenScriptServices};

/// ScriptController's lazy main-world context factory. Sharing this service
/// shares an Agent runtime, never another Frame's global or Document binding.
pub(super) struct LocalFrameScriptFactory {
    pub(super) script: Weak<RefCell<ScriptEngine>>,
    pub(super) xhr: Rc<RefCell<Box<dyn xhr_transport::XMLHttpRequestTransport>>>,
    pub(super) user_agent: String,
    root: RefCell<RootWindowScriptBindings>,
}
struct RootWindowScriptBindings {
    bindings: Weak<RefCell<DOMJavaScriptBindings>>,
    window: Weak<RefCell<WindowJavaScriptBindings>>,
    interaction: Weak<crate::interaction_services::PageInteraction>,
}
impl LocalFrameScriptFactory {
    pub(super) fn new(
        script: Weak<RefCell<ScriptEngine>>,
        xhr: Rc<RefCell<Box<dyn xhr_transport::XMLHttpRequestTransport>>>,
        user_agent: String,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        window: &Rc<RefCell<WindowJavaScriptBindings>>,
        interaction: &Rc<crate::interaction_services::PageInteraction>,
    ) -> Self {
        Self {
            script,
            xhr,
            user_agent,
            root: RefCell::new(RootWindowScriptBindings {
                bindings: Rc::downgrade(bindings),
                window: Rc::downgrade(window),
                interaction: Rc::downgrade(interaction),
            }),
        }
    }
    // LocalDOMWindow::InstallNewDocument / LocalWindowProxyManager::UpdateDocument:
    // the Frame service survives navigation, but every access uses its current
    // Window and Document. These aliases borrow the published owners; they never
    // retain a second Window or a Document's state.
    pub(super) fn RebindRootWindow(
        &self,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        window: &Rc<RefCell<WindowJavaScriptBindings>>,
        interaction: &Rc<crate::interaction_services::PageInteraction>,
    ) {
        *self.root.borrow_mut() = RootWindowScriptBindings {
            bindings: Rc::downgrade(bindings),
            window: Rc::downgrade(window),
            interaction: Rc::downgrade(interaction),
        };
    }
    pub(super) fn RootDOMBindings(&self) -> Option<Rc<RefCell<DOMJavaScriptBindings>>> {
        self.root.borrow().bindings.upgrade()
    }
    pub(super) fn RootWindowBindings(&self) -> Option<Rc<RefCell<WindowJavaScriptBindings>>> {
        self.root.borrow().window.upgrade()
    }
    pub(super) fn RootInteraction(
        &self,
    ) -> Option<Rc<crate::interaction_services::PageInteraction>> {
        self.root.borrow().interaction.upgrade()
    }
}
struct PendingFrameExecutor {
    script: Weak<RefCell<ScriptEngine>>,
    realm: RefCell<Option<JavaScriptRealm>>,
}
impl ScriptExecutor for PendingFrameExecutor {
    fn SupportsModules(&self) -> bool {
        self.script
            .upgrade()
            .expect("live Agent")
            .borrow()
            .SupportsModules()
    }
    fn SetModuleCompilationWake(
        &self,
        wake: Option<javascript::javascript_runtime::JavaScriptModuleCompilationWake>,
    ) {
        self.script
            .upgrade()
            .expect("live Agent")
            .borrow_mut()
            .SetModuleCompilationWake(wake);
    }
    fn WithRuntimeAndRealm(
        &self,
        callback: &mut dyn FnMut(&mut dyn JavaScriptRuntime, &JavaScriptRealm),
    ) {
        let script = self.script.upgrade().expect("live Agent");
        let realm = self
            .realm
            .borrow()
            .clone()
            .expect("Window committed before script execution");
        callback(script.borrow_mut().RuntimeMut(), &realm);
    }
}
#[derive(Default)]
pub(super) struct LocalFrameNavigation {
    pub(super) pending_history_restore: Option<Arc<navigation_history::HistoryItemSnapshot>>,
    pub(super) committed_history_restore: Option<Arc<navigation_history::HistoryItemSnapshot>>,
    pending: Option<FrameOwnerNavigation>,
    parent_delay: Option<dom::load_event_delay_count::LoadEventDelayLease>,
    srcdoc: Rc<[u16]>,
    active: Option<Rc<FrameDocumentLoader>>,
    pub(super) window: Option<Rc<super::local_frame_window::LocalFrameWindow>>,
    pub(super) proxy: Option<javascript::javascript_runtime::HostValue>,
}
struct FrameDocumentLoader {
    frame: Weak<LocalFrame>,
    page: Weak<PageState>,
    document: Rc<DocumentEngine>,
    resources: Rc<ResourceFetcher>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    engine: RefCell<OpenEngine>,
    window: std::cell::OnceCell<Rc<super::local_frame_window::LocalFrameWindow>>,
    script_executor: Option<Rc<PendingFrameExecutor>>,
    executor: Option<ScriptExecutorHandle>,
    client: Rc<RefCell<FrameLoadClient>>,
    dynamic: RefCell<Option<Rc<open::dynamic_scripts::DynamicScriptTasks>>>,
    navigation: FrameOwnerNavigation,
    cancelled: Cell<bool>,
    committed: Cell<bool>,
    runtime: Rc<super::local_frame::LocalFrameRuntime>,
}
struct FrameLoadClient {
    page: Weak<PageState>,
    frame: Weak<LocalFrame>,
}
impl ScriptLoadClient for FrameLoadClient {
    fn DidPumpParser(&mut self) {
        if let Some(frame) = self.frame.upgrade() {
            if let Some(runtime) = frame.runtime.borrow().clone() {
                dom::attribute_lifecycle::DrainAttributeLifecycle(&runtime.document.Handle());
            }
            frame.MarkNavigationDirty();
        }
    }
    fn DidApplyStyleSheet(&mut self, _: &str) {
        self.DidPumpParser();
    }
    fn DidReportScriptError(&mut self, error: &JavaScriptException) {
        if let Some(page) = self.page.upgrade() {
            page.client.borrow_mut().DidReportScriptError(error);
        }
    }
    fn DidFailResource(&mut self, url: &str, error: &str) {
        if let Some(page) = self.page.upgrade() {
            page.client.borrow_mut().DidFail(&format!("{url}: {error}"));
        }
    }
}
impl LocalFrameNavigation {
    pub(super) fn HasPendingNavigation(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn LoadType(&self) -> Option<navigation_history::FrameLoadType> {
        Some(self.active.as_ref()?.navigation.load_type)
    }
    pub(super) fn StopLoading(&mut self) {
        self.pending = None;
        self.parent_delay = None;
        if let Some(active) = self.active.take() {
            active.Cancel();
        }
    }
}
struct FormTaskRunner {
    window: Weak<RefCell<WindowJavaScriptBindings>>,
    page: Weak<PageState>,
}
struct FormTask(Rc<RefCell<Option<Box<dyn FnOnce()>>>>);
impl interaction::default::core::loader::frame_navigation_lifecycle::CancellableNavigationTask
    for FormTask
{
    fn IsActive(&self) -> bool {
        self.0.borrow().is_some()
    }
    fn Cancel(&self) {
        self.0.borrow_mut().take();
    }
}
impl interaction::default::core::loader::frame_navigation_lifecycle::FormSubmissionTaskRunner
    for FormTaskRunner
{
    fn PostDOMManipulationTask(&self, task: Box<dyn FnOnce()>) -> Rc<dyn interaction::default::core::loader::frame_navigation_lifecycle::CancellableNavigationTask>{
        let task = Rc::new(RefCell::new(Some(task)));
        if let Some(window) = self.window.upgrade() {
            let callback = task.clone();
            window.borrow_mut().EnqueueDelayedNativeTask(
                std::time::Duration::ZERO,
                Box::new(move || {
                    let task = callback.borrow_mut().take();
                    if let Some(task) = task {
                        task();
                    }
                }),
            );
        } else if let Some(page) = self.page.upgrade() {
            let document = page.document.borrow().GetDocument().RootHandle();
            page.popup_tasks.Post(
                document,
                0.0,
                super::popup_tasks::PageInteractionTask::FormNavigation(task.clone()),
            );
        }
        Rc::new(FormTask(task))
    }
}
pub(super) struct FormNavigationPreparation {
    domain: Weak<LocalFrameDomain>,
    source: u64,
}
impl FormNavigationPreparation {
    pub(super) fn new(domain: &Rc<LocalFrameDomain>, source: u64) -> Self {
        Self {
            domain: Rc::downgrade(domain),
            source,
        }
    }
}
impl interaction::FormNavigationPreparation for FormNavigationPreparation {
    fn Prepare(
        &self,
        target: &super::NavigationTarget,
        events: &dyn interaction::default::core::loader::frame_navigation_lifecycle::FrameLifecycleEventDispatcher,
    ) -> bool {
        let Some(domain) = self.domain.upgrade() else {
            return false;
        };
        let Some(target) = domain.ResolveNavigationTarget(self.source, target) else {
            return true;
        };
        let frame = domain.Frame(target);
        let lifecycle = frame.as_ref().map_or_else(
            || domain.navigation_lifecycle.clone(),
            |frame| frame.navigation_lifecycle.clone(),
        );
        if !lifecycle.IsNavigationAllowed() {
            return false;
        }
        if let Some(document) = domain.Document(target) {
            if target == self.source {
                let mut cancellation = document.borrow().GetDocument().BeginCancelParsing(true);
                loop {
                    use dom::document_parser_lifecycle::CancelParsingStage;
                    match cancellation.Next() {
                        CancelParsingStage::RegisterParserAbortedStickyFeature => {
                            lifecycle.RegisterParserAbortedStickyFeature()
                        }
                        CancelParsingStage::ReadyStateChanged => events.DispatchReadyStateChanged(),
                        CancelParsingStage::CancelledHttpRefresh => {}
                        CancelParsingStage::Done => break,
                    }
                }
            }
            document
                .borrow()
                .GetDocument()
                .CancelPendingJavaScriptUrls();
        }
        lifecycle.ClearClientNavigation();
        if let Some(frame) = frame {
            frame.navigation.borrow_mut().pending = None;
        }
        true
    }
}
impl LocalFrameDomain {
    pub(super) fn ResolveNavigationTarget(
        &self,
        source: u64,
        target: &super::NavigationTarget,
    ) -> Option<u64> {
        use super::NavigationTarget;
        match target {
            NavigationTarget::SelfFrame => Some(source),
            NavigationTarget::Parent => {
                Some(self.Frame(source).map_or(self.root, |frame| frame.parent))
            }
            NavigationTarget::Top => Some(self.root),
            NavigationTarget::NewContext => None,
            NavigationTarget::Named(name) => {
                fn find(domain: &LocalFrameDomain, parent: u64, name: &[u16]) -> Option<u64> {
                    for frame in domain.Children(parent) {
                        if frame.name.borrow().as_ref() == name {
                            return Some(frame.state.Identity());
                        }
                        if let Some(found) = find(domain, frame.state.Identity(), name) {
                            return Some(found);
                        }
                    }
                    None
                }
                let name = name.encode_utf16().collect::<Vec<_>>();
                let current = self
                    .Frame(source)
                    .filter(|frame| frame.name.borrow().as_ref() == name.as_slice())
                    .map(|frame| frame.state.Identity());
                current
                    .or_else(|| find(self, source, &name))
                    .or_else(|| find(self, self.root, &name))
            }
        }
    }
    /// FrameTree::FindFrameForNavigation resolves reserved names and named
    /// descendants against the same native owners used by WindowProxy.
    pub(super) fn RouteTargetNavigation(
        &self,
        source: u64,
        mut request: NavigationRequest,
        scheduling: Option<interaction::FormSubmissionScheduling>,
    ) {
        let target = self.ResolveNavigationTarget(source, &request.target);
        let external = target.is_none();
        let target = target.unwrap_or(source);
        let target_frame = self.Frame(target);
        let lifecycle = target_frame.as_ref().map_or_else(
            || self.navigation_lifecycle.clone(),
            |frame| frame.navigation_lifecycle.clone(),
        );
        if !lifecycle.IsNavigationAllowed() {
            return;
        }
        let window = if target == self.root {
            self.script_factory
                .get()
                .and_then(|factory| factory.RootWindowBindings())
                .map_or_else(Weak::new, |window| Rc::downgrade(&window))
        } else {
            target_frame
                .as_ref()
                .and_then(|frame| {
                    frame
                        .navigation
                        .borrow()
                        .window
                        .as_ref()
                        .map(|window| Rc::downgrade(window.web_api.WindowBindings()))
                })
                .unwrap_or_default()
        };
        let Some(page) = self.page.upgrade() else {
            return;
        };
        let weak_domain = Rc::downgrade(page.local_frames.get().unwrap());
        let generation = target_frame.as_ref().map(|frame| frame.generation.get());
        let document = self
            .Document(source)
            .map(|document| Rc::downgrade(&document));
        let task_token = scheduling
            .as_ref()
            .map(|scheduling| scheduling.task.clone());
        let runner = FormTaskRunner {
            window,
            page: self.page.Downgrade(),
        };
        let navigate = Box::new(move || {
            let Some(domain) = weak_domain.upgrade() else {
                return;
            };
            if let Some(scheduling) = scheduling {
                let Some(document) = document.as_ref().and_then(Weak::upgrade) else {
                    return;
                };
                let valid = {
                    let owner = document.borrow();
                    let document = owner.GetDocument();
                    scheduling.task.IsCurrent()
                        && document.Token() == scheduling.document_token
                        && document.IsActive()
                        && document
                            .FindNodeById(scheduling.form_owner.Id())
                            .is_some_and(|index| {
                                document.NodeHandle(index) == scheduling.form_owner
                                    && document.IsConnectedToDocument(index)
                            })
                };
                if !valid {
                    return;
                }
            }
            if external {
                if let Some(page) = domain.page.upgrade() {
                    page.client.borrow_mut().DidRequestNavigation(&request);
                }
            } else if target == domain.root {
                request.target = NavigationTarget::SelfFrame;
                if let Some(page) = domain.page.upgrade() {
                    page.client.borrow_mut().DidRequestNavigation(&request);
                }
            } else if let Some(frame) = domain.Frame(target) {
                if frame.generation.get() != generation.unwrap()
                    || frame.state.Lifecycle().GetState()
                        != dom::frame_lifecycle::FrameLifecycleState::Attached
                {
                    return;
                }
                frame.RequestScriptNavigationRequest(
                    request.request,
                    if request.replace_history {
                        navigation_history::FrameLoadType::ReplaceCurrentItem
                    } else {
                        navigation_history::FrameLoadType::Standard
                    },
                );
            }
        });
        if external {
            interaction::default::core::loader::frame_navigation_lifecycle::FrameNavigationLifecycle::PostFormSubmissionTask(&runner, task_token, navigate);
        } else {
            // The target Frame and originating form independently version the
            // task, matching replacement and cross-target cancellation.
            let cancel =
                lifecycle.ScheduleFormSubmissionTaskWithToken(&runner, task_token, navigate);
            drop(cancel);
        }
    }

    pub(super) fn RouteNavigationEffect(
        &self,
        effect: &FrameOwnerLoadEffect,
    ) -> Result<(), FrameOwnerLoadError> {
        match effect {
            FrameOwnerLoadEffect::Navigate { frame, navigation }
            | FrameOwnerLoadEffect::StartNavigation { frame, navigation } => {
                let target = self
                    .Frame(frame.State().Identity())
                    .ok_or(FrameOwnerLoadError::UnexpectedCreatedFrame)?;
                if target.state.Owner() != Some(navigation.owner) {
                    return Err(FrameOwnerLoadError::WrongOwnerDocument);
                }
                {
                    let mut pending = target.navigation.borrow_mut();
                    if pending.parent_delay.is_none() {
                        if let Some(parent) = target.owner_document.upgrade() {
                            let parent = parent.borrow();
                            let doc = parent.GetDocument();
                            pending.parent_delay = Some(
                                doc.LoadEventDelayCounterForDocument(doc.RootHandle())
                                    .Acquire(),
                            );
                        }
                    }
                }
                let mut navigation = navigation.clone();
                self.RestoreChildNavigation(&target, &mut navigation);
                target.navigation.borrow_mut().pending = Some(navigation);
                target.MarkNavigationDirty();
                Ok(())
            }
            FrameOwnerLoadEffect::SrcDocChanged {
                frame_identity,
                value,
            } => {
                let frame = self
                    .Frame(*frame_identity)
                    .ok_or(FrameOwnerLoadError::UnexpectedCreatedFrame)?;
                frame.navigation.borrow_mut().srcdoc = value.clone();
                Ok(())
            }
            _ => Err(FrameOwnerLoadError::HostEffect),
        }
    }
    pub(super) fn HasPendingNavigations(&self) -> bool {
        self.frames.borrow().values().any(|frame| {
            let navigation = frame.navigation.borrow();
            navigation.pending.is_some()
                || navigation
                    .active
                    .as_ref()
                    .is_some_and(|active| active.engine.borrow().IsLoading())
        })
    }
    /// This entry runs after the current JS/DOM invocation has unwound, just as
    /// a browser-process navigation commit enters the renderer as a new task.
    pub(super) fn AdvanceNavigations(&self, milliseconds: f64) -> io::Result<()> {
        let frames: Vec<_> = self.frames.borrow().values().cloned().collect();
        for frame in frames {
            let pending = frame.navigation.borrow_mut().pending.take();
            if let Some(pending) = pending {
                if !pending.load_type.IsTraversal() {
                    if let Some(page) = self.page.upgrade() {
                        page.SaveHistoryState();
                    }
                }
                let next = FrameDocumentLoader::Create(&frame, pending, self.script_factory.get())?;
                let previous = frame.navigation.borrow_mut().active.replace(next.clone());
                if let Some(previous) = previous {
                    previous.Cancel();
                }
                frame.state.SetIsLoading(true);
                next.Start()?;
            }
            let active = frame.navigation.borrow().active.clone();
            if let Some(active) = active {
                active.Advance(milliseconds)?;
            } else if let Some(window) = frame.navigation.borrow().window.clone() {
                window.RunTaskTurn(milliseconds);
            }
        }
        Ok(())
    }
}
impl LocalFrame {
    pub(super) fn WindowPerformanceTiming(
        &self,
    ) -> Option<Rc<webapi::window_bindings::WindowPerformanceTiming>> {
        Some(self.window_performance_timing.borrow().clone())
    }
    pub(super) fn MarkNavigationDirty(&self) {
        if let Some(domain) = self.domain.upgrade() {
            domain.dirty.set(true);
            if let Some(page) = domain.page.upgrade() {
                page.dirty.set(true);
                page.scroll_only.set(false);
                if let Some(runtime) = self.runtime.borrow().as_ref() {
                    runtime
                        .document
                        .borrow()
                        .GetDocument()
                        .Lifecycle()
                        .ScheduleVisualUpdate();
                }
            }
        }
    }
    pub(super) fn NavigationLoadType(&self) -> Option<navigation_history::FrameLoadType> {
        self.navigation.borrow().LoadType()
    }
    pub(super) fn RequestScriptNavigation(&self, url: String, replace: bool) {
        self.RequestScriptNavigationWithLoadType(
            url,
            if replace {
                navigation_history::FrameLoadType::ReplaceCurrentItem
            } else {
                navigation_history::FrameLoadType::Standard
            },
        );
    }
    pub(super) fn RequestScriptNavigationWithLoadType(
        &self,
        url: String,
        load_type: navigation_history::FrameLoadType,
    ) {
        self.RequestScriptNavigationRequest(
            URLRequest {
                url,
                ..Default::default()
            },
            load_type,
        );
    }
    pub(super) fn RequestScriptNavigationRequest(
        &self,
        request: URLRequest,
        load_type: navigation_history::FrameLoadType,
    ) {
        let Some(domain) = self.domain.upgrade() else {
            return;
        };
        let Some(parent) = self.owner_document.upgrade() else {
            return;
        };
        let Some(runtime) = self.runtime.borrow().clone() else {
            return;
        };
        let Some(owner) = self.state.Owner() else {
            return;
        };
        let policy = {
            let parent = parent.borrow();
            let doc = parent.GetDocument();
            let Some(node) = doc.FindNodeById(owner.Id()) else {
                return;
            };
            let Some(policy) = doc.FrameOwnerPolicy(node) else {
                return;
            };
            policy
        };
        let navigation = FrameOwnerNavigation {
            owner,
            request: dom::frame_owner_loading::FrameOwnerResourceRequest {
                url: runtime
                    .document
                    .borrow()
                    .GetDocument()
                    .CompleteURLValue(&request.url),
                method: request.method,
                referrer: request.referrer,
                headers: request.headers,
                body: request.body,
                encoded_body: request.encoded_body,
                navigation: request.navigation,
                referrer_policy: match runtime.resources.ReferrerPolicy() {
                    url_loader::ReferrerPolicy::NoReferrer => {
                        navigation_history::ReferrerPolicy::Never
                    }
                    url_loader::ReferrerPolicy::NoReferrerWhenDowngrade => {
                        navigation_history::ReferrerPolicy::NoReferrerWhenDowngrade
                    }
                    url_loader::ReferrerPolicy::Origin => {
                        navigation_history::ReferrerPolicy::Origin
                    }
                    url_loader::ReferrerPolicy::OriginWhenCrossOrigin => {
                        navigation_history::ReferrerPolicy::OriginWhenCrossOrigin
                    }
                    url_loader::ReferrerPolicy::SameOrigin => {
                        navigation_history::ReferrerPolicy::SameOrigin
                    }
                    url_loader::ReferrerPolicy::StrictOrigin => {
                        navigation_history::ReferrerPolicy::StrictOrigin
                    }
                    url_loader::ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                        navigation_history::ReferrerPolicy::StrictOriginWhenCrossOrigin
                    }
                    url_loader::ReferrerPolicy::UnsafeUrl => {
                        navigation_history::ReferrerPolicy::Always
                    }
                },
                trust_token: None,
                bypass_cache: false,
                skip_service_worker: false,
            },
            policy,
            load_type,
            reason: dom::frame_owner_loading::FrameOwnerNavigationReason::FrameNavigation,
            is_container_initiated: false,
        };
        let Some(frame) = domain.Frame(self.state.Identity()) else {
            return;
        };
        domain
            .RouteNavigationEffect(&FrameOwnerLoadEffect::StartNavigation { frame, navigation })
            .unwrap_or_else(|error| std::panic::panic_any(error));
    }
    pub(super) fn StopNavigation(&self) {
        let active = self.navigation.borrow_mut().active.take();
        self.navigation.borrow_mut().pending = None;
        if let Some(active) = active {
            active.Cancel();
        }
        if let Some(window) = self.navigation.borrow_mut().window.take() {
            window
                .document
                .borrow()
                .GetDocument()
                .SetLifecycleActive(false);
            if let Ok(mut bindings) = window.web_api.WindowBindings().try_borrow_mut() {
                bindings.ResetForNewWindow();
            }
        }
    }
}
impl FrameDocumentLoader {
    fn Create(
        frame: &Rc<LocalFrame>,
        navigation: FrameOwnerNavigation,
        factory: Option<&Rc<LocalFrameScriptFactory>>,
    ) -> io::Result<Rc<FrameDocumentLoader>> {
        let domain = frame
            .domain
            .upgrade()
            .ok_or_else(|| io::Error::other("detached frame"))?;
        let page = domain
            .page
            .upgrade()
            .ok_or_else(|| io::Error::other("detached Page"))?;
        let parent = domain
            .Document(frame.parent)
            .ok_or_else(|| io::Error::other("detached parent"))?;
        let parent_resources = domain
            .Resources(frame.parent)
            .ok_or_else(|| io::Error::other("missing parent FetchContext"))?;
        let document = Rc::new(DocumentEngine::new());
        document
            .borrow_mut()
            .GetDocumentMut()
            .InitializeLocalFrameNavigationDocument(
                parent.borrow().GetDocument(),
                &navigation.policy,
                navigation.request.url.AsStr(),
            );
        let constraints = Rc::new(RefCell::new(
            frame
                .runtime
                .borrow()
                .as_ref()
                .expect("initialized frame")
                .constraints
                .borrow()
                .clone(),
        ));
        let (resources, resource_engine) = parent_resources.ForChildDocument(
            document.Handle(),
            constraints.clone(),
            document.borrow().GetDocument().BaseURL().AsStr().to_owned(),
        );
        let image_document = document.Handle();
        resources.SetHTMLImageFetchSecurity(Rc::new(move || {
            FrameFetchSecurity(image_document.borrow().GetDocument())
        }));
        resources.SetFontFetchSecuritySource(Rc::new(FrameFetchSecurity));
        let runtime = frame
            .PrepareDocument(
                document.clone(),
                None,
                resources.clone(),
                constraints.clone(),
            )
            .map_err(|error| io::Error::other(format!("{error:?}")))?;
        let script_executor = if document.borrow().GetDocument().ScriptingEnabled() {
            let factory = factory.ok_or_else(|| {
                io::Error::other("script-enabled LocalFrame requires its Page Agent factory")
            })?;
            Some(Rc::new(PendingFrameExecutor {
                script: factory.script.clone(),
                realm: Default::default(),
            }))
        } else {
            None
        };
        let executor = script_executor
            .as_ref()
            .map(|executor| executor.clone() as ScriptExecutorHandle);
        let mut engine = OpenEngine::new(resource_engine, executor.clone());
        if let Some(wake) = domain.loading_wake.borrow().clone() {
            engine.SetWakeCallback(wake);
        }
        engine.BindRequiredDocumentPolicy(navigation.policy.required_document.clone());
        engine.NavigationState().SetLoadType(navigation.load_type);
        let client = Rc::new(RefCell::new(FrameLoadClient {
            page: Rc::downgrade(&page),
            frame: Rc::downgrade(frame),
        }));
        Ok(Rc::new(FrameDocumentLoader {
            frame: Rc::downgrade(frame),
            page: Rc::downgrade(&page),
            document,
            resources,
            constraints,
            engine: RefCell::new(engine),
            window: Default::default(),
            script_executor,
            executor,
            client,
            dynamic: Default::default(),
            navigation,
            cancelled: Cell::new(false),
            committed: Cell::new(false),
            runtime,
        }))
    }
}
impl FrameDocumentLoader {
    fn Window(&self) -> &Rc<super::local_frame_window::LocalFrameWindow> {
        self.window
            .get()
            .expect("Window installed before parser entry")
    }
    fn Cancel(&self) {
        self.cancelled.set(true);
        // Detach the original shared parser lifecycle synchronously, even if
        // cancellation reenters from a script borrowing its loader/runtime.
        self.document.borrow().GetDocument().DetachParser();
        self.document
            .borrow()
            .GetDocument()
            .SetLifecycleActive(false);
        if let Ok(mut engine) = self.engine.try_borrow_mut() {
            engine.Stop();
        }
        if let Some(window) = self.window.get() {
            if let Ok(mut bindings) = window.web_api.WindowBindings().try_borrow_mut() {
                bindings.ResetForNewWindow();
            }
        }
    }
    fn Start(self: &Rc<Self>) -> io::Result<()> {
        let url = self.navigation.request.url.AsStr();
        let request = URLRequest {
            url: url.to_owned(),
            method: self.navigation.request.method.clone(),
            referrer: self.navigation.request.referrer.clone(),
            headers: self.navigation.request.headers.clone(),
            body: self.navigation.request.body.clone(),
            encoded_body: self.navigation.request.encoded_body.clone(),
            navigation: self.navigation.request.navigation.clone(),
            destination: url_loader::RequestDestination::kDocument,
            ..Default::default()
        };
        let mut adapter = FrameOpenAdapter(self.clone());
        if url.starts_with("about:") {
            let source = if url == "about:srcdoc" {
                let frame = self
                    .frame
                    .upgrade()
                    .ok_or_else(|| io::Error::other("detached frame"))?;
                let parent = frame
                    .owner_document
                    .upgrade()
                    .ok_or_else(|| io::Error::other("detached owner Document"))?;
                let parent = parent.borrow();
                let doc = parent.GetDocument();
                let node = doc
                    .FindNodeById(self.navigation.owner.Id())
                    .ok_or_else(|| io::Error::other("removed frame owner"))?;
                doc.Node(node)
                    .FindAttribute("srcdoc")
                    .map_or_else(String::new, |attribute| attribute.value.clone())
            } else {
                String::new()
            };
            let mut engine = self.engine.borrow_mut();
            engine.AdmitSuppliedDocumentRequest(&request)?;
            engine.CommitURL(&mut adapter, url.to_owned())?;
            engine.ParseSuppliedDocument(&mut adapter, &source, 4096, 4096)?;
            engine.FinishSuppliedStyleSheets(&mut adapter)?;
            engine.FinishSuppliedLoad(&mut adapter);
        } else {
            self.engine
                .borrow_mut()
                .Open(request, document_loader::DocumentLoadBudget::default())?;
        }
        Ok(())
    }
    fn Advance(self: &Rc<Self>, milliseconds: f64) -> io::Result<()> {
        if self.cancelled.get() {
            self.Cancel();
            return Ok(());
        }
        let mut adapter = FrameOpenAdapter(self.clone());
        {
            let mut engine = self.engine.borrow_mut();
            engine.BeginTaskTurn(milliseconds);
            engine.Advance(&mut adapter)?;
            engine.RunScriptTaskTurn(milliseconds, |scheduler, executor, limit| {
                let mut progressed = 0;
                executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                    progressed = scheduler.PumpModules(runtime, realm, limit);
                });
                Ok(progressed)
            })?;
            engine.EndTaskTurn();
        }
        if self.cancelled.get() {
            self.Cancel();
            return Ok(());
        }
        if self.committed.get() && self.executor.is_some() {
            if let Some(dynamic) = self.dynamic.borrow().as_ref() {
                dynamic.EnqueuePreparedTasks(
                    self.Window().web_api.WindowBindings(),
                    self.client.clone(),
                );
            }
            self.executor
                .as_ref()
                .expect("script-enabled frame")
                .WithRuntimeAndRealm(&mut |runtime, realm| {
                    self.Window()
                        .web_api
                        .RunTaskTurn(runtime, realm, milliseconds, &mut |error| {
                            self.client.borrow_mut().DidReportScriptError(error)
                        });
                    runtime.PerformMicrotaskCheckpoint();
                });
            if let Some(dynamic) = self.dynamic.borrow().as_ref() {
                dynamic.EnqueuePreparedTasks(
                    self.Window().web_api.WindowBindings(),
                    self.client.clone(),
                );
            }
        }
        Ok(())
    }
}
struct FrameOpenAdapter(Rc<FrameDocumentLoader>);
impl OpenDocumentAdapter for FrameOpenAdapter {
    fn InitializeWindowNavigation(
        &mut self,
        loader: Rc<document_loader::DocumentLoaderNavigationState>,
    ) {
        loader.EnsureHistoryItemForWindow(self.0.document.borrow().GetDocument());
        let frame = self.0.frame.upgrade().expect("attached LocalFrame commit");
        let domain = frame.domain.upgrade().expect("live frame domain");
        let _ = self.0.runtime.navigation_state.set(loader.clone());
        frame.navigation.borrow_mut().pending_history_restore = None;
        if let Some(item) = loader.CurrentHistoryItem() {
            item.SetTarget(frame.unique_name.borrow().Value().to_owned());
        }
        frame.unique_name.borrow_mut().Freeze();
        domain.InstallHistoryRestore(frame.state.Identity(), &loader);
        if let (Some(window), Some(executor)) = (self.0.window.get(), self.0.executor.as_ref()) {
            executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                let _ = runtime.WithHostRuntime(realm, &mut |host_runtime| {
                    window
                        .navigation
                        .InitializeCommitted(loader.clone(), host_runtime)
                });
            });
        } else if let Some(page) = self.0.page.upgrade() {
            page.PublishChildHistory(
                loader
                    .LoadType()
                    .unwrap_or(navigation_history::FrameLoadType::ReplaceCurrentItem),
            );
        }
    }
    fn CommitURL(&mut self, url: &str) {
        let owner = &self.0;
        let frame = owner.frame.upgrade().expect("attached LocalFrame commit");
        let domain = frame.domain.upgrade().expect("live frame domain");
        let parent = domain.Document(frame.parent).expect("live parent Document");
        owner
            .document
            .borrow_mut()
            .GetDocumentMut()
            .InitializeLocalFrameNavigationDocument(
                parent.borrow().GetDocument(),
                &owner.navigation.policy,
                url,
            );
        owner
            .document
            .borrow()
            .GetDocument()
            .CommitNavigationDocumentToken();
        owner.resources.SetDocumentURL(url.to_owned());
        for child in domain.Children(frame.state.Identity()) {
            child.DetachForRemove();
        }
        frame.InstallPreparedDocument(owner.runtime.clone());
        if let Some(executor) = &owner.script_executor {
            let factory = domain.script_factory.get().expect("Frame Agent factory");
            let window = factory
                .CreateNavigationWindow(&frame, owner.runtime.clone())
                .unwrap_or_else(|error| std::panic::panic_any(error));
            *executor.realm.borrow_mut() = Some(window.realm.clone());
            assert!(owner.window.set(window.clone()).is_ok());
            frame.navigation.borrow_mut().window = Some(window);
            domain.BindChildEventHandler(frame.state.Identity());
        }
        owner.committed.set(true);
        frame.MarkNavigationDirty();
    }
    fn PrepareDocumentServices(&mut self, _: &str) -> OpenDocumentServices {
        let owner = self.0.clone();
        OpenDocumentServices {
            document: owner.document.Handle(),
            resources: owner.resources.clone(),
            scripts: owner.window.get().map(|window| OpenScriptServices {
                bindings: window.web_api.DOMBindings().clone(),
                pending_host_errors: None,
            }),
            render_blocking_media_matches: Rc::new(move |query| {
                let Some(page) = owner.page.upgrade() else {
                    return false;
                };
                let constraints = owner.constraints.borrow();
                let data = crate::style_services::MediaValuesForLayout(
                    &constraints,
                    page.preferred_color_scheme.get(),
                    page.forced_colors.get(),
                );
                let evaluator =
                    style::production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(
                        &data,
                    );
                evaluator
                    .EvalSet(&style::media_queries::media_query_backend::ParseMediaQuerySet(query))
            }),
        }
    }
    fn InstallDynamicScriptHooks(
        &mut self,
        dynamic: &Rc<open::dynamic_scripts::DynamicScriptTasks>,
    ) {
        let prepare = Rc::downgrade(dynamic);
        self.0
            .Window()
            .connected
            .SetScriptPreparer(Some(Rc::new(move |document, node| {
                if let Some(dynamic) = prepare.upgrade() {
                    dynamic.PrepareConnectedScript(document, node, true)?;
                }
                Ok(())
            })));
        *self.0.dynamic.borrow_mut() = Some(dynamic.clone());
    }
    fn WithScriptLoadClient(&mut self, callback: &mut dyn FnMut(&mut dyn ScriptLoadClient)) {
        callback(&mut *self.0.client.borrow_mut());
    }
    fn UpdateDocumentResources(&mut self, allowance: usize, _: bool) -> io::Result<usize> {
        let owner = self.0.clone();
        let mut client = FrameResourceClient(owner.clone());
        owner.resources.QueueReferencedImages()?;
        let styles = owner
            .resources
            .PollPendingStyleSheetsWithBudget(&mut client, allowance);
        let images = owner
            .resources
            .PollPendingImagesWithBudget(&mut client, allowance.saturating_sub(styles));
        owner.resources.PrepareFontFaces();
        owner.resources.StartPendingFonts();
        let fonts = owner
            .resources
            .PollPendingFontsWithBudget(&mut client, allowance.saturating_sub(styles + images));
        Ok(styles + images + fonts)
    }
    fn HasPendingImageEvents(&self) -> bool {
        self.0
            .window
            .get()
            .is_some_and(|window| window.PendingEvents() != 0)
    }
    fn HasPendingLoadBlockingImages(&self) -> bool {
        self.0.resources.HasPendingLoadBlockingImages()
    }
    fn LoadEventDelayCount(&self) -> u32 {
        self.0.document.borrow().GetDocument().LoadEventDelayCount()
    }
    fn FinishLoad(&mut self, pipeline: &mut OpenDocumentPipeline) {
        if let OpenDocumentPipeline::Script {
            scheduler,
            executor,
        } = pipeline
        {
            self.0
                .Window()
                .web_api
                .ApplyMutation(WebApiMutation::SetDocumentReadyState("complete".into()));
            executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                scheduler.DispatchLifecycleEvent(
                    interaction::event::EventType::kReadyStateChange,
                    runtime,
                    realm,
                    &mut *self.0.client.borrow_mut(),
                );
                scheduler.DispatchLifecycleEvent(
                    interaction::event::EventType::kLoad,
                    runtime,
                    realm,
                    &mut *self.0.client.borrow_mut(),
                );
                runtime.PerformMicrotaskCheckpoint();
            });
        }
        self.0
            .document
            .borrow()
            .GetDocument()
            .SetReadyState(dom::document_parser_lifecycle::DocumentReadyState::Complete);
        self.0.document.borrow().GetDocument().SetLoadEventProgress(
            dom::document_parser_lifecycle::DocumentLoadEventProgress::Completed,
        );
        if let Some(frame) = self.0.frame.upgrade() {
            frame.state.SetIsLoading(false);
            let delay = frame.navigation.borrow_mut().parent_delay.take();
            if let Some(domain) = frame.domain.upgrade() {
                domain.QueueOwnerLoadEvent(&frame, delay);
            }
            frame.MarkNavigationDirty();
        }
    }
    fn FailLoad(&mut self, message: &str) {
        self.0
            .client
            .borrow_mut()
            .DidFailResource(self.0.navigation.request.url.AsStr(), message);
        if let Some(frame) = self.0.frame.upgrade() {
            frame.state.SetIsLoading(false);
            frame.navigation.borrow_mut().parent_delay = None;
        }
    }
}
struct FrameResourceClient(Rc<FrameDocumentLoader>);
impl document_loader::ResourceFetcherClient for FrameResourceClient {
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        match mutation {
            ResourceMutation::PlatformImageResourceReady(image) => {
                self.0.constraints.borrow_mut().images.push(image)
            }
            ResourceMutation::ImageResourceReady(image) => {
                dom::image_resource::AddImageResourceWithContent(
                    &mut self.0.document.borrow_mut(),
                    &mut self.0.constraints.borrow_mut(),
                    image.source,
                    image.image,
                    image.content,
                );
            }
            ResourceMutation::FontFaceResourceChanged { owner, font } => {
                let mut constraints = self.0.constraints.borrow_mut();
                if let Some(engine) = &constraints.font_engine {
                    let mutation = match font {
                        Some(font) => {
                            font_engine::FontMutation::UpsertRule(font_engine::FontRule {
                                id: owner,
                                family: font.family.clone(),
                                state: font_engine::FontFaceState::Ready(font),
                            })
                        }
                        None => font_engine::FontMutation::RemoveRule(owner),
                    };
                    engine
                        .with_mut(|engine| engine.ApplyMutation(mutation))
                        .expect("child FontFaceCache mutation");
                    constraints.fonts = engine.with_mut(|engine| engine.ResolvedFaces());
                }
                self.0.document.borrow_mut().InvalidateLayoutGeometry();
            }
            ResourceMutation::FontResourceReady(font) => {
                let mut constraints = self.0.constraints.borrow_mut();
                constraints.fonts.push(font.font);
                if let Some(engine) = &constraints.font_engine {
                    let faces = constraints.fonts.iter().cloned().collect();
                    engine
                        .with_mut(|engine| {
                            engine.ApplyMutation(font_engine::FontMutation::SetFaces(faces))
                        })
                        .expect("child FontFaceCache admission");
                    constraints.fonts = engine.with_mut(|engine| engine.ResolvedFaces());
                }
                self.0.document.borrow_mut().InvalidateLayoutGeometry();
            }
            ResourceMutation::ResourceLoadFailed(failure) => {
                if failure.kind == page_mutation::ResourceKind::kImage {
                    if let Some(source) = failure.source {
                        self.0
                            .document
                            .borrow_mut()
                            .GetDocumentMut()
                            .SetImageResourceFailed(source);
                    }
                }
                self.0
                    .client
                    .borrow_mut()
                    .DidFailResource(&failure.url, &failure.error);
            }
            ResourceMutation::DocumentImageFrameChanged(update) => {
                let mut constraints = self.0.constraints.borrow_mut();
                let image = constraints
                    .images
                    .iter_mut()
                    .find(|image| image.id == update.frame.resource_id)
                    .expect("child document image owner");
                image.revision = update.frame.revision;
                image.UpdateDocumentFrameMetadata(&update.frame);
                image.document_container_key = Some(update.frame.container_key.clone());
                let image_resource::PaintImageContent::Document(content) = &image.content else {
                    panic!("document frame targeted child bitmap resource")
                };
                image.content =
                    image_resource::PaintImageContent::Document(Arc::new(content.with_variant(
                        update.frame.container_key.clone(),
                        update.frame.record.clone(),
                    )));
                let snapshot = Arc::new(image.clone());
                drop(constraints);
                if let Some(content) = self
                    .0
                    .document
                    .borrow()
                    .GetDocument()
                    .ImageResourceContentFor(&update.source)
                {
                    content.SetImageSnapshot(snapshot);
                }
            }
            ResourceMutation::DocumentImageIntrinsicSizeChanged(update) => {
                let mut constraints = self.0.constraints.borrow_mut();
                let image = constraints
                    .images
                    .iter_mut()
                    .find(|image| image.id == update.resource_id)
                    .expect("child document image owner");
                image.width = update.size.width;
                image.height = update.size.height;
                image.revision = update.revision;
                let metadata = dom::ImageResourceMetadata {
                    id: image.id,
                    natural_width: image.width as f64 / image.resolution_scale,
                    natural_height: image.height as f64 / image.resolution_scale,
                    resolution_scale: image.resolution_scale,
                    orientation: image.orientation,
                    known_to_be_opaque: false,
                    is_cors_same_origin: image.is_cors_same_origin,
                };
                drop(constraints);
                self.0
                    .document
                    .borrow_mut()
                    .GetDocumentMut()
                    .UpdateImageResourceIntrinsicSize(&update.source, metadata);
            }
            ResourceMutation::VideoFrameChanged(_) => {
                std::panic::panic_any(FrameOwnerLoadError::HostEffect)
            }
        }
        if let Some(frame) = self.0.frame.upgrade() {
            frame.MarkNavigationDirty();
        }
    }
    fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        if let Some(window) = self.0.window.get() {
            window.QueueImageEvent(id, loaded);
        }
    }
    fn DispatchStyleSheetEvent(&mut self, id: u64, signature: String, loaded: bool) {
        if let Some(window) = self.0.window.get() {
            window.QueueStyleSheetEvent(id, signature, loaded);
        }
    }
    fn HasJavaScript(&self) -> bool {
        self.0.executor.is_some()
    }
}

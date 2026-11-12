//! LocalDOMWindow, lazy ScriptController context and stable WindowProxy owners.
//! The WindowProxy holds a weak native Frame identity; every access resolves
//! the Frame's current Window and checks its current SecurityOrigin.
use super::local_frame::{LocalFrame, LocalFrameDomain, LocalFrameRuntime};
use super::local_frame_navigation::LocalFrameScriptFactory;
use super::*;
use javascript::javascript_runtime::{
    HostResult, HostValue, JavaScriptExceptionKind, JavaScriptHostBindings,
    JavaScriptNativeFunction,
};
use javascript::script_engine::{ScriptExecutor, ScriptExecutorHandle};
use webapi::dom_bindings::{FrameProperty, FramePropertyGetter};

pub(super) struct LocalFrameWindow {
    pub(super) web_api: WebApiEngine,
    pub(super) executor: ScriptExecutorHandle,
    pub(super) realm: JavaScriptRealm,
    pub(super) document: Rc<DocumentEngine>,
    credentialless: bool,
    pub(super) interaction: Rc<crate::interaction_services::PageInteraction>,
    pub(super) navigation: Rc<super::navigation_api::NavigationWindowEnvironment>,
    pub(super) connected: Rc<open::ConnectedResourceDiscovery>,
    pending_events: Rc<Cell<usize>>,
    page: Weak<PageState>,
}
struct SharedXHR(Rc<RefCell<Box<dyn xhr_transport::XMLHttpRequestTransport>>>);
impl xhr_transport::XMLHttpRequestTransport for SharedXHR {
    fn Start(
        &mut self,
        request: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        self.0.borrow_mut().Start(request)
    }
}
struct FrameScriptExecutor {
    script: Weak<RefCell<ScriptEngine>>,
    realm: JavaScriptRealm,
}
impl ScriptExecutor for FrameScriptExecutor {
    fn SupportsModules(&self) -> bool {
        self.script
            .upgrade()
            .expect("live Page Agent")
            .borrow()
            .SupportsModules()
    }
    fn SetModuleCompilationWake(
        &self,
        wake: Option<javascript::javascript_runtime::JavaScriptModuleCompilationWake>,
    ) {
        self.script
            .upgrade()
            .expect("live Page Agent")
            .borrow_mut()
            .SetModuleCompilationWake(wake);
    }
    fn WithRuntimeAndRealm(
        &self,
        callback: &mut dyn FnMut(&mut dyn JavaScriptRuntime, &JavaScriptRealm),
    ) {
        let script = self.script.upgrade().expect("live Page Agent");
        callback(script.borrow_mut().RuntimeMut(), &self.realm);
    }
}
impl LocalFrameScriptFactory {
    fn PrepareWindow(
        &self,
        frame: &Rc<LocalFrame>,
        runtime: &Rc<LocalFrameRuntime>,
        previous: Option<&WebApiEngine>,
    ) -> WebApiEngine {
        let target = Rc::downgrade(frame);
        let original = runtime.document.clone();
        let effects: Rc<dyn Fn(WebApiEffect)> = Rc::new(move |effect| match effect {
            WebApiEffect::DocumentMutation(mutation) => {
                original
                    .ApplyDOMMutation(&mutation)
                    .unwrap_or_else(|error| std::panic::panic_any(error));
                if let Some(frame) = target.upgrade() {
                    frame.MarkNavigationDirty();
                }
            }
            WebApiEffect::Navigate(navigation) => {
                if let Some(frame) = target.upgrade() {
                    frame.RequestScriptNavigation(navigation.url, navigation.replace_history);
                }
            }
            WebApiEffect::AnimationTick { .. } => {
                if let Some(frame) = target.upgrade() {
                    frame.MarkNavigationDirty();
                }
            }
        });
        let host = FrameDOMHost(frame, runtime);
        let web_api = if let Some(previous) = previous {
            WebApiEngine::ForWindowReuse(runtime.document.Handle(), host, previous, effects)
        } else {
            WebApiEngine::WithPerformanceTiming(
                runtime.document.Handle(),
                host,
                Box::new(SharedXHR(self.xhr.clone())),
                self.user_agent.clone(),
                effects,
                frame
                    .WindowPerformanceTiming()
                    .expect("native LocalDOMWindow clock"),
            )
        };
        web_api.ApplyMutation(WebApiMutation::SetURL(
            runtime
                .document
                .borrow()
                .GetDocument()
                .DocumentURL()
                .to_owned(),
        ));
        let constraints = runtime.constraints.borrow();
        web_api.ApplyMutation(WebApiMutation::SetViewport {
            width: constraints.available_size.width,
            height: constraints.available_size.height,
            scale: constraints.device_pixel_ratio,
        });
        web_api
    }
    fn FinishWindow(
        &self,
        frame: &Rc<LocalFrame>,
        runtime: &Rc<LocalFrameRuntime>,
        web_api: WebApiEngine,
        realm: JavaScriptRealm,
    ) -> Rc<LocalFrameWindow> {
        let executor: ScriptExecutorHandle = Rc::new(FrameScriptExecutor {
            script: self.script.clone(),
            realm: realm.clone(),
        });
        let owner = runtime
            .frame_interaction
            .get()
            .expect("Document input owner precedes Window realm");
        let pending_submissions = owner.pending_submissions.clone();
        let domain = frame.domain.upgrade().expect("live Window frame domain");
        let clock = domain
            .page
            .upgrade()
            .expect("live Window Page")
            .InteractionEventClock();
        let interaction = Rc::new(
            crate::interaction_services::PageInteraction::InstallWithFrameOwner(
                runtime.document.Handle(),
                web_api.DOMBindings(),
                runtime.interaction_state.clone(),
                runtime.editing.selections.clone(),
                clock,
                owner.engine.clone(),
                owner.controller.clone(),
            ),
        );
        if let Some(domain) = frame.domain.upgrade() {
            interaction
                .Engine()
                .SetFormNavigationPreparation(Some(Rc::new(
                    super::local_frame_navigation::FormNavigationPreparation::new(
                        &domain,
                        frame.state.Identity(),
                    ),
                )));
        }
        interaction
            .Engine()
            .FormControlState()
            .SetValidationMessageClient(
                super::validation_message::PageValidationMessageClient::ForChild(frame, runtime),
            );
        interaction.BindFrameDomain(frame.domain.clone());
        interaction.BindRealm(&realm);
        super::local_frame_scroll::InteractionScrollOwner::Child {
            frame: Rc::downgrade(frame),
            runtime: Rc::downgrade(runtime),
        }
        .BindBlockingWheelInvalidation(&web_api.DOMBindings().borrow());
        if let Some(ring) = frame
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
            .and_then(|page| page.kill_ring.borrow().clone())
        {
            interaction.Engine().SetKillRing(ring);
        }
        let form_data_frame = Rc::downgrade(frame);
        let form_data_runtime = Rc::downgrade(runtime);
        interaction.SetFormDataTransaction(Rc::new(move |engine, document, token| {
            use interaction::{
                dispatch::core::fileapi::form_data_object::FormDataConstructionError as E,
                input_type::PrepareFormEntryListResult,
            };
            let frame = form_data_frame
                .upgrade()
                .ok_or(E::InvalidState("The form's Frame is no longer active."))?;
            let runtime = form_data_runtime
                .upgrade()
                .ok_or(E::InvalidState("The form's Document is no longer active."))?;
            if !Rc::ptr_eq(document, &runtime.document.Handle())
                || !frame
                    .runtime
                    .borrow()
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(current, &runtime))
            {
                return Err(E::InvalidState("The form belongs to a retired Document."));
            }
            let domain = frame.domain.upgrade().ok_or(E::InvalidState(
                "The form's Frame domain is no longer active.",
            ))?;
            let page = domain
                .page
                .upgrade()
                .ok_or(E::InvalidState("The form's Page is no longer active."))?;
            page.EnsureLayoutOutput();
            let children = domain
                .PreparePaint(&page.layout_engine.borrow())
                .map_err(|_| E::InvalidState("FormData child layout preparation failed."))?;
            LocalFrameDomain::ReturnPrePaint(children);
            let prepared = {
                let layout = runtime.layout.borrow();
                let output = layout
                    .Output()
                    .ok_or(E::InvalidState("FormData requires current child layout."))?;
                engine
                    .ResumeFormSubmission(
                        document,
                        token,
                        interaction::adapter::page_editing_context::PreparedEditingGeometry::new(
                            output,
                        ),
                    )
                    .map_err(interaction::input_type::FormDataConstructionErrorFor)?
            };
            match prepared {
                Some(PrepareFormEntryListResult::Ready(pending)) => {
                    engine.FinishFormData(document, pending)
                }
                Some(PrepareFormEntryListResult::NeedsLifecycle(_)) => Err(E::InvalidState(
                    "FormData child layout preparation did not complete.",
                )),
                None => Err(E::InvalidState("The form is constructing entry list.")),
            }
        }));
        let completion_frame = Rc::downgrade(frame);
        let completion_runtime = Rc::downgrade(runtime);
        interaction.SetFormSubmissionTransaction(Rc::new(move |engine, document| {
            let (Some(frame), Some(runtime)) = (completion_frame.upgrade(), completion_runtime.upgrade()) else { return; };
            let Some(domain) = frame.domain.upgrade() else { return; };
            let Some(page) = domain.page.upgrade() else { return; };
            loop {
                let Some(submission) = pending_submissions.borrow_mut().pop_front() else { break; };
                page.EnsureLayoutOutput();
                let prepared_children = domain.PreparePaint(&page.layout_engine.borrow());
                match prepared_children {
                    Ok(children) => LocalFrameDomain::ReturnPrePaint(children),
                    Err(error) => { page.client.borrow_mut().DidFail(&error.to_string()); continue; }
                }
                let prepared = {
                    let layout = runtime.layout.borrow();
                    layout.Output().map(|output| engine.ResumeFormSubmission(document, submission.token, interaction::adapter::page_editing_context::PreparedEditingGeometry::new(output)))
                };
                match prepared {
                    Some(Ok(Some(interaction::input_type::PrepareFormEntryListResult::Ready(pending)))) => engine.FinishFormSubmission(document, pending, submission.parameters),
                    Some(Ok(None)) => {},
                    Some(Err(error)) => page.client.borrow_mut().DidFail(&format!("form submission: {error:?}")),
                    _ => page.client.borrow_mut().DidFail("form submission has no prepared child layout output"),
                }
            }
        }));
        if let Some(reader) = frame
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
            .and_then(|page| page.native_file_reader.borrow().clone())
        {
            interaction.SetFormStateFileReader(reader);
        }
        let connected = Rc::new(open::ConnectedResourceDiscovery::new(
            runtime.document.Handle(),
        ));
        connected.SetResources(runtime.resources.clone());
        let navigation = super::navigation_api::NavigationWindowEnvironment::Install(
            frame
                .domain
                .upgrade()
                .map_or_else(Weak::new, |domain| domain.page.Downgrade()),
            Rc::downgrade(frame),
            &runtime.document,
            &web_api,
            &interaction,
            frame
                .WindowPerformanceTiming()
                .and_then(|timing| timing.ExistingEventTimingSource()),
        );
        let window = Rc::new(LocalFrameWindow {
            navigation,
            web_api,
            executor,
            realm,
            document: runtime.document.clone(),
            credentialless: FrameCredentialless(frame),
            interaction,
            connected: connected.clone(),
            pending_events: Rc::new(Cell::new(0)),
            page: frame
                .domain
                .upgrade()
                .map_or_else(Weak::new, |domain| domain.page.Downgrade()),
        });
        let target = Rc::downgrade(frame);
        let original = runtime.document.clone();
        let discovery = connected.clone();
        let resources = runtime.resources.clone();
        let event_window = Rc::downgrade(&window);
        window.web_api.DOMBindings().borrow_mut().SetScopedMutationEmitter(Rc::new(move |mutation, _| {
            let effects = original.ApplyDOMMutation(mutation)?;
            discovery.ProcessRenderBlockingMutations();
            dom::attribute_lifecycle::DrainAttributeLifecycle(&original.Handle());
            for request in effects.resource {
                match request {
                    ResourceRequest::LoadDeferredImage { node_id } => { resources.LoadDeferredHTMLImage(node_id); },
                    ResourceRequest::ResetImage { node_id } | ResourceRequest::ImageReferrerPolicyChanged { node_id, .. } => {
                        if let Some(node) = original.borrow().GetDocument().FindNodeById(node_id) {
                            resources.UpdateHTMLImageLoader(node, dom::persistent_document::ImageLoaderUpdate::IgnorePreviousError, false).unwrap_or_else(|error| std::panic::panic_any(error));
                        }
                    }
                    ResourceRequest::DiscoverConnectedSubtree(request) => {
                        let node = original.borrow().GetDocument().FindNodeById(request.root_node_id);
                        if let Some(node) = node {
                            discovery.PrepareSubtree(node, request.prepare_scripts, &mut |id, loaded| {
                                if let Some(window) = event_window.upgrade() { window.QueueImageEvent(id, loaded); }
                            }).unwrap_or_else(|error| std::panic::panic_any(error));
                        }
                    }
                }
            }
            if let Some(frame) = target.upgrade() { frame.MarkNavigationDirty(); }
            Ok(())
        }));
        window
    }
    pub(super) fn CreateWindow(
        &self,
        frame: &Rc<LocalFrame>,
        runtime: Rc<LocalFrameRuntime>,
    ) -> io::Result<Rc<LocalFrameWindow>> {
        let web_api = self.PrepareWindow(frame, &runtime, None);
        let script = self
            .script
            .upgrade()
            .ok_or_else(|| io::Error::other("Page Agent destroyed"))?;
        let realm = {
            let mut script = script.borrow_mut();
            let parent = script.Realm().clone();
            let global: Rc<RefCell<dyn JavaScriptHostBindings>> = web_api.WindowBindings().clone();
            let realm = script
                .RuntimeMut()
                .CreateChildRealm(&parent, global)
                .map_err(|error| io::Error::other(error.message))?;
            web_api
                .BindInitialHostObjectNamespace(script.RuntimeMut(), &realm)
                .map_err(|error| io::Error::other(error.message))?;
            web_api.BindRuntime(&realm);
            for source in WebApiEngine::BootstrapSources() {
                let result = script.RuntimeMut().EvaluateBootstrap(
                    &realm,
                    &source.source,
                    &source.source_name,
                );
                if let Some(error) = result.exception {
                    return Err(io::Error::other(error.message));
                }
            }
            realm
        };
        Ok(self.FinishWindow(frame, &runtime, web_api, realm))
    }
    pub(super) fn CreateNavigationWindow(
        &self,
        frame: &Rc<LocalFrame>,
        runtime: Rc<LocalFrameRuntime>,
    ) -> io::Result<Rc<LocalFrameWindow>> {
        let previous = frame.navigation.borrow().window.clone();
        if let Some(previous) = &previous {
            let reuse = {
                let old = previous.document.borrow();
                let next = runtime.document.borrow();
                let old = old.GetDocument();
                let next = next.GetDocument();
                // The current embedding admits only nonisolated AgentClusters;
                // compare their actual Window ownership, not a caller default.
                old.IsInitialEmptyDocument()
                    && previous.credentialless == FrameCredentialless(frame)
                    && old
                        .SecurityContext()
                        .Origin()
                        .CanAccess(next.SecurityContext().Origin().as_ref())
                    && old
                        .ExecutionContextIsolation()
                        .CrossOriginIsolatedCapability()
                        == next
                            .ExecutionContextIsolation()
                            .CrossOriginIsolatedCapability()
            };
            if reuse {
                let web_api = self.PrepareWindow(frame, &runtime, Some(&previous.web_api));
                let script = self
                    .script
                    .upgrade()
                    .ok_or_else(|| io::Error::other("Page Agent destroyed"))?;
                let mut script = script.borrow_mut();
                web_api
                    .PrepareHostObjectNamespaceForReuse(script.RuntimeMut(), &previous.realm)
                    .map_err(|error| io::Error::other(error.message))?;
                web_api.BindRuntime(&previous.realm);
                previous
                    .web_api
                    .DOMBindings()
                    .borrow_mut()
                    .DisassociateWindowDocument();
                web_api.PublishDocumentForReuse(script.RuntimeMut(), &previous.realm);
                return Ok(self.FinishWindow(frame, &runtime, web_api, previous.realm.clone()));
            }
            previous
                .web_api
                .WindowBindings()
                .borrow_mut()
                .ResetForNewWindow();
            previous
                .web_api
                .DOMBindings()
                .borrow_mut()
                .DisassociateWindowDocument();
        }
        self.CreateWindow(frame, runtime)
    }
    fn CreateWindowInHost(
        &self,
        frame: &Rc<LocalFrame>,
        runtime: Rc<LocalFrameRuntime>,
        host: &mut dyn JavaScriptHostRuntime,
    ) -> Result<Rc<LocalFrameWindow>, JavaScriptException> {
        let web_api = self.PrepareWindow(frame, &runtime, None);
        let realm = host.CreateChildRealm(web_api.WindowBindings().clone())?;
        let mut outcome = Ok(());
        host.WithRealm(&realm, &mut |host| {
            outcome = (|| {
                web_api.BindInitialHostObjectNamespaceInHost(host)?;
                web_api.BindRuntime(&realm);
                for source in WebApiEngine::BootstrapSources() {
                    let result = host.EvaluateClassicScript(
                        &source.source.encode_utf16().collect::<Vec<_>>(),
                        &source.source_name,
                    );
                    if let Some(error) = result.exception {
                        return Err(error);
                    }
                }
                Ok(())
            })();
        })?;
        outcome?;
        Ok(self.FinishWindow(frame, &runtime, web_api, realm))
    }
}
/// Root and child bindings use the same FrameTree registry and resolve the
/// original content Frame. No second DOM tree or numeric document projection.
pub(super) fn FrameReader(
    domain: Weak<LocalFrameDomain>,
    document: Rc<DocumentEngine>,
) -> FramePropertyGetter {
    Rc::new(move |id, property, host| {
        let Some(domain) = domain.upgrade() else {
            return HostResult {
                value: HostValue::Null(Default::default()),
                ..Default::default()
            };
        };
        let source = {
            let owner = document.borrow();
            let doc = owner.GetDocument();
            doc.FindNodeById(id).and_then(|node| doc.ContentFrame(node))
        };
        let Some(source) = source else {
            return HostResult {
                value: HostValue::Null(Default::default()),
                ..Default::default()
            };
        };
        if source.IsRemoteFrame() {
            return HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "remote WindowProxy requires its remote Frame owner",
            );
        }
        let Some(frame) = domain.Frame(source.State().Identity()) else {
            return HostResult::Failure(JavaScriptExceptionKind::kTypeError, "detached LocalFrame");
        };
        let current_window = frame.navigation.borrow().window.clone();
        let window = match current_window {
            Some(window) => window,
            None => {
                let Some(factory) = domain.script_factory.get() else {
                    return HostResult::Failure(
                        JavaScriptExceptionKind::kTypeError,
                        "LocalFrame ScriptController unavailable",
                    );
                };
                let Some(runtime) = frame.runtime.borrow().clone() else {
                    return HostResult::Failure(
                        JavaScriptExceptionKind::kTypeError,
                        "LocalFrame Document unavailable",
                    );
                };
                let window = match factory.CreateWindowInHost(&frame, runtime, host) {
                    Ok(window) => window,
                    Err(error) => {
                        return HostResult {
                            exception: Some(error),
                            ..Default::default()
                        };
                    }
                };
                frame.navigation.borrow_mut().window = Some(window.clone());
                frame
                    .domain
                    .upgrade()
                    .expect("live child input domain")
                    .BindChildEventHandler(frame.state.Identity());
                window
            }
        };
        if property == FrameProperty::ContentDocument {
            if !CanAccess(&document, &window.document) {
                return HostResult {
                    value: HostValue::Null(Default::default()),
                    ..Default::default()
                };
            }
            let mut result = HostResult::default();
            if let Err(error) = host.WithRealm(&window.realm, &mut |host| {
                let global = host.GlobalObject();
                result = host.GetProperty(&global, "document");
            }) {
                return HostResult {
                    exception: Some(error),
                    ..Default::default()
                };
            }
            return result;
        }
        let existing = frame.navigation.borrow().proxy.clone();
        if let Some(proxy) = existing {
            return HostResult {
                value: proxy,
                ..Default::default()
            };
        }
        let callback = Rc::new(FrameWindowAccess {
            frame: Rc::downgrade(&frame),
            observer: Rc::downgrade(&document),
        });
        let native = host.CreateNativeFunction(callback);
        if !native.Succeeded() {
            return native;
        }
        // The proxy has stable Frame identity across Document commits. The
        // traps forward to the current Window, exactly where origin is checked.
        let factory = host.EvaluateClassicScript(&"(read => { let proxy; proxy = new Proxy({}, { get(_, key) { if (key === 'window' || key === 'self' || key === 'frames') return proxy; return read('get', key); }, set(_, key, value) { return read('set', key, value); } }); return proxy; })".encode_utf16().collect::<Vec<_>>(), "browser:window-proxy");
        if !factory.Succeeded() {
            return factory;
        }
        let result = host.CallValue(&factory.value, &HostValue::default(), &[native.value]);
        if result.Succeeded() {
            frame.navigation.borrow_mut().proxy = Some(result.value.clone());
        }
        result
    })
}
/// WebIDL Window conversion uses the existing Frame's Window/proxy identity,
/// without reading author properties or creating another Window registry.
pub(super) fn WindowViewConverter(
    domain: Weak<LocalFrameDomain>,
) -> webapi::dom_bindings::DOMWindowViewConverter {
    Rc::new(move |value, host| {
        let domain = domain.upgrade()?;
        let factory = domain.script_factory.get()?;
        if let Some(root) = factory.RootInteraction() {
            if let Some(realm) = root.WindowRealm() {
                let mut same = false;
                host.WithRealm(&realm, &mut |host| {
                    let global = host.GlobalObject();
                    same = host.IsSameObject(value, &global);
                })
                .ok()?;
                if same {
                    return root.WindowView();
                }
            }
        }
        for frame in domain.frames.borrow().values() {
            let navigation = frame.navigation.borrow();
            let Some(window) = navigation.window.as_ref() else {
                continue;
            };
            if navigation
                .proxy
                .as_ref()
                .is_some_and(|proxy| host.IsSameObject(value, proxy))
            {
                return window.interaction.WindowView();
            }
            let mut same = false;
            host.WithRealm(&window.realm, &mut |host| {
                let global = host.GlobalObject();
                same = host.IsSameObject(value, &global);
            })
            .ok()?;
            if same {
                return window.interaction.WindowView();
            }
        }
        None
    })
}
pub(super) fn WindowViewProjector(
    domain: Weak<LocalFrameDomain>,
) -> webapi::dom_bindings::DOMWindowViewProjector {
    Rc::new(move |view| {
        let domain = domain.upgrade()?;
        let factory = domain.script_factory.get()?;
        if let Some(root) = factory.RootInteraction() {
            if root
                .WindowView()
                .is_some_and(|current| Rc::ptr_eq(view, &current))
            {
                return Some(
                    factory
                        .RootDOMBindings()?
                        .borrow()
                        .HostObjectNamespace()?
                        .Wrap(HostValue::Object(
                            javascript::javascript_runtime::HostObjectRef { id: 0 },
                        )),
                );
            }
        }
        for frame in domain.frames.borrow().values() {
            let navigation = frame.navigation.borrow();
            let Some(window) = navigation.window.as_ref() else {
                continue;
            };
            if window
                .interaction
                .WindowView()
                .is_some_and(|current| Rc::ptr_eq(view, &current))
            {
                if let Some(proxy) = navigation.proxy.as_ref() {
                    return Some(proxy.clone());
                }
                return Some(
                    window
                        .web_api
                        .DOMBindings()
                        .borrow()
                        .HostObjectNamespace()?
                        .Wrap(HostValue::Object(
                            javascript::javascript_runtime::HostObjectRef { id: 0 },
                        )),
                );
            }
        }
        // DOMWindow::Wrap (dom_window.cc:248-260) returns null when the native
        // Window no longer has a Frame and no existing wrapper was retained.
        Some(HostValue::Null(Default::default()))
    })
}
fn CanAccess(observer: &DocumentEngine, target: &DocumentEngine) -> bool {
    observer
        .borrow()
        .GetDocument()
        .SecurityContext()
        .Origin()
        .CanAccess(
            target
                .borrow()
                .GetDocument()
                .SecurityContext()
                .Origin()
                .as_ref(),
        )
}
struct FrameWindowAccess {
    frame: Weak<LocalFrame>,
    observer: Weak<DocumentEngine>,
}
impl JavaScriptNativeFunction for FrameWindowAccess {
    fn Call(
        &self,
        host: &mut dyn JavaScriptHostRuntime,
        _: &HostValue,
        arguments: &[HostValue],
    ) -> HostResult {
        let Some(frame) = self.frame.upgrade() else {
            return HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "detached WindowProxy",
            );
        };
        let Some(window) = frame.navigation.borrow().window.clone() else {
            return HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "WindowProxy has no current Window",
            );
        };
        let operation = match arguments.first() {
            Some(HostValue::String(value)) => value.as_str(),
            _ => {
                return HostResult::Failure(
                    JavaScriptExceptionKind::kTypeError,
                    "invalid WindowProxy operation",
                );
            }
        };
        let key = match arguments.get(1) {
            Some(HostValue::String(value)) => value.clone(),
            Some(HostValue::String16(value)) => String::from_utf16_lossy(value),
            _ => return HostResult::default(),
        };
        let Some(observer) = self.observer.upgrade() else {
            return HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "WindowProxy caller Document detached",
            );
        };
        if !CanAccess(&observer, &window.document) {
            if operation == "get" && key == "closed" {
                return HostResult {
                    value: HostValue::Boolean(false),
                    ..Default::default()
                };
            }
            return HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "blocked cross-origin Window access",
            );
        }
        let mut result = HostResult::default();
        if let Err(error) = host.WithRealm(&window.realm, &mut |host| {
            let global = host.GlobalObject();
            result = if operation == "set" {
                host.SetProperty(
                    &global,
                    &key,
                    arguments.get(2).unwrap_or(&HostValue::default()),
                )
            } else {
                host.GetProperty(&global, &key)
            };
        }) {
            return HostResult {
                exception: Some(error),
                ..Default::default()
            };
        }
        result
    }
}
fn FrameDOMHost(frame: &Rc<LocalFrame>, runtime: &Rc<LocalFrameRuntime>) -> DOMBindingsHost {
    let style_runtime = runtime.clone();
    let style_frame = Rc::downgrade(frame);
    let invalidate = Rc::downgrade(frame);
    let geometry = runtime.clone();
    let metric = runtime.clone();
    let sheets = runtime.clone();
    let fonts = runtime.clone();
    let domain = frame.domain.clone();
    let viewport_runtime = runtime.clone();
    let viewport_frame = Rc::downgrade(frame);
    let write_runtime = runtime.clone();
    let write_frame = Rc::downgrade(frame);
    let direct_runtime = runtime.clone();
    let direct_frame = Rc::downgrade(frame);
    let focus_domain = frame.domain.clone();
    let focus_frame = frame.state.Identity();
    DOMBindingsHost {
        convert_window_view: Some(WindowViewConverter(frame.domain.clone())),
        project_window_view: Some(WindowViewProjector(frame.domain.clone())),
        read_layout_zoom: Some({
            let runtime = Rc::downgrade(runtime);
            Rc::new(move || {
                runtime.upgrade().map_or(1., |runtime| {
                    runtime.constraints.borrow().layout_zoom_factor as f32
                })
            })
        }),
        retain_drag_image: Some(Rc::new(super::drag_image::RetainImageResource)),
        read_document_focus: Some(Rc::new(move |identity| {
            focus_domain
                .upgrade()?
                .DocumentFocusState(focus_frame, identity)
        })),
        read_viewport_scroll: Some(Box::new(move || {
            let frame = viewport_frame.upgrade()?;
            frame.EnsureFocusLayout(&viewport_runtime);
            let layout = viewport_runtime.layout.borrow();
            let range = layout.Output()?.Root().ScrollRange();
            let owner = viewport_runtime.document.borrow();
            let doc = owner.GetDocument();
            let root = doc.DocumentElement()?;
            let native = doc
                .ResolvedStyleFor(root)
                .map(|resolved| unsafe { &*resolved.computed_style_owner.Get() })?;
            Some(webapi::dom_bindings::ViewportScrollState {
                target_node_id: doc.Node(root).Id(),
                offset: doc.ScrollOffsetFor(root),
                minimum: range.minimum,
                maximum: range.maximum,
                layout_zoom_factor: viewport_runtime.constraints.borrow().layout_zoom_factor,
                smooth_behavior: native.GetScrollBehavior().value() == 2,
                snap: !native.GetScrollSnapType().is_none,
            })
        })),
        write_viewport_scroll: Some(Box::new(move |node, offset, behavior| {
            if let Some(frame) = write_frame.upgrade() {
                frame.SetProgrammaticScrollOffset(
                    &write_runtime,
                    node,
                    offset,
                    behavior,
                    compositor::ScrollSourceType::kAbsoluteScroll,
                );
            }
        })),
        write_scroll: Some(Box::new(move |node, offset| {
            if let Some(frame) = direct_frame.upgrade() {
                frame.SetProgrammaticScrollOffset(
                    &direct_runtime,
                    node,
                    offset,
                    ViewportScrollBehavior::Instant,
                    compositor::ScrollSourceType::kAbsoluteScroll,
                );
            }
        })),
        read_frame: Some(FrameReader(domain, runtime.document.clone())),
        event_clock: Some(
            frame
                .domain
                .upgrade()
                .and_then(|domain| domain.page.upgrade())
                .expect("live Frame clock")
                .InteractionEventClock(),
        ),
        cssom_document: Some((
            runtime.style.borrow().CSSOMSource(),
            runtime.resources.ReferrerPolicyReadHandle(),
        )),
        update_style: Some(Box::new(move |id| {
            let Some(frame) = style_frame.upgrade() else {
                return;
            };
            let Some(domain) = frame.domain.upgrade() else {
                return;
            };
            let Some(page) = domain.page.upgrade() else {
                return;
            };
            if let Some(interaction) = domain.FocusInteraction(frame.state.Identity()) {
                if interaction::adapter::page_form_validation::RefreshControlValidity(
                    &style_runtime.document.Handle(),
                    &interaction.FormControlState(),
                )
                .is_err()
                {
                    return;
                }
            }
            let mut document = style_runtime.document.borrow_mut();
            let _ =
                crate::style_services::ResolveLayoutStylesWithEngineAndForcedColorsUsingFontMetrics(
                    &mut style_runtime.style.borrow_mut(),
                    &mut document,
                    &style_runtime.constraints.borrow(),
                    page.preferred_color_scheme.get(),
                    page.forced_colors.get(),
                    page.style_font_metrics.clone(),
                );
            if let Some(node) = document.GetDocument().FindNodeById(id) {
                crate::style_services::EnsureComputedStyleForNode(
                    &mut style_runtime.style.borrow_mut(),
                    &mut document,
                    &style_runtime.constraints.borrow(),
                    page.preferred_color_scheme.get(),
                    page.forced_colors.get(),
                    node,
                );
            }
        })),
        invalidate_style: Some(Box::new(move || {
            if let Some(frame) = invalidate.upgrade() {
                frame.MarkNavigationDirty();
            }
        })),
        emit_style_sheet: Some(Box::new(move |sheet| {
            sheets.document.AppendStyleSheet(sheet);
        })),
        read_font_loading: Some(Box::new(move || fonts.resources.HasPendingFonts())),
        read_geometry: Some(Box::new(move |id| {
            let frame_layout_zoom = geometry.constraints.borrow().layout_zoom_factor as f32;
            let standardized_browser_zoom = geometry
                .document
                .borrow()
                .GetDocument()
                .StandardizedBrowserZoomEnabled();
            geometry
                .layout
                .borrow()
                .Output()
                .and_then(|output| output.Node(id))
                .map_or_else(Vec::new, |node| {
                    node.HTMLClientRectsForDocument(
                        frame_layout_zoom,
                        standardized_browser_zoom,
                    )
                        .into_iter()
                        .map(|rect| paint::PaintRect {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height,
                        })
                        .collect()
                })
        })),
        read_metric: Some(Box::new(move |id, name| {
            let query = layoutng_assembly::layout_engine::LayoutMetric::try_from(name).ok()?;
            metric
                .layout
                .borrow()
                .Output()?
                .Node(id)
                .and_then(|node| node.Metric(query))
        })),
        ..Default::default()
    }
}

impl LocalFrameWindow {
    pub(super) fn PendingEvents(&self) -> usize {
        self.pending_events.get()
    }
    pub(super) fn QueueImageEvent(&self, id: u64, loaded: bool) {
        let ticket = {
            let owner = self.document.borrow();
            let doc = owner.GetDocument();
            doc.FindNodeById(id)
                .and_then(|node| doc.ExistingHTMLImageLoader(node))
                .and_then(|loader| loader.TakeEventTicket())
        };
        let Some(ticket) = ticket else {
            return;
        };
        if (ticket.event == dom::persistent_document::ImageLoaderEvent::Load) != loaded {
            return;
        }
        let document = self.document.clone();
        let bindings = self.web_api.DOMBindings().clone();
        let engine = self.interaction.Engine().clone();
        let page = self.page.clone();
        let pending = self.pending_events.clone();
        pending.set(pending.get() + 1);
        self.web_api
            .WindowBindings()
            .borrow_mut()
            .EnqueueTaskWithRuntime(
                Box::new(move |runtime, realm| {
                    pending.set(pending.get().saturating_sub(1));
                    let delay = {
                        let owner = document.borrow();
                        let doc = owner.GetDocument();
                        if doc
                            .FindNodeById(id)
                            .is_some_and(|node| doc.NodeHandle(node) == ticket.loader.Element())
                        {
                            ticket.loader.BeginEventDispatch(&ticket)
                        } else {
                            None
                        }
                    };
                    let Some(_delay) = delay else {
                        return;
                    };
                    let skip = {
                        let owner = document.borrow();
                        let doc = owner.GetDocument();
                        doc.FindNodeById(id)
                            .is_some_and(|node| doc.HTMLImageLoaderShouldSkipEventDispatch(node))
                    };
                    if !skip {
                        DispatchDOMEvent(
                            &document,
                            &bindings,
                            &engine,
                            &page,
                            id,
                            if loaded {
                                EventType::kLoad
                            } else {
                                EventType::kCustom
                            },
                            runtime,
                            realm,
                        );
                    }
                }),
                None,
            );
    }
    pub(super) fn QueueStyleSheetEvent(&self, id: u64, signature: String, loaded: bool) {
        let document = self.document.clone();
        let bindings = self.web_api.DOMBindings().clone();
        let engine = self.interaction.Engine().clone();
        let page = self.page.clone();
        let pending = self.pending_events.clone();
        pending.set(pending.get() + 1);
        self.web_api
            .WindowBindings()
            .borrow_mut()
            .EnqueueTaskWithRuntime(
                Box::new(move |runtime, realm| {
                    pending.set(pending.get().saturating_sub(1));
                    let valid = {
                        let owner = document.borrow();
                        let doc = owner.GetDocument();
                        doc.FindNodeById(id).is_some_and(|node| {
                            let node = doc.Node(node);
                            let attr = |name| {
                                node.FindAttribute(name)
                                    .map_or("", |attribute| attribute.value.as_str())
                            };
                            node.IsHTMLElement("link")
                                && format!(
                                    "{}\0{}\0{}\0{}\0{}\0{}",
                                    attr("href"),
                                    attr("rel"),
                                    attr("type"),
                                    node.FindAttribute("disabled").is_some(),
                                    attr("title"),
                                    attr("media")
                                ) == signature
                        })
                    };
                    if valid {
                        DispatchDOMEvent(
                            &document,
                            &bindings,
                            &engine,
                            &page,
                            id,
                            if loaded {
                                EventType::kLoad
                            } else {
                                EventType::kCustom
                            },
                            runtime,
                            realm,
                        );
                    }
                }),
                None,
            );
    }
}
impl LocalFrameDomain {
    pub(super) fn QueueOwnerLoadEvent(
        &self,
        frame: &Rc<LocalFrame>,
        delay: Option<dom::load_event_delay_count::LoadEventDelayLease>,
    ) {
        let Some(owner) = frame.state.Owner() else {
            return;
        };
        let Some(document) = self.Document(frame.parent) else {
            return;
        };
        let Some(factory) = self.script_factory.get() else {
            return;
        };
        let (window, bindings, engine) = if frame.parent == self.root {
            let (Some(window), Some(bindings), Some(interaction)) = (
                factory.RootWindowBindings(),
                factory.RootDOMBindings(),
                factory.RootInteraction(),
            ) else {
                return;
            };
            (window, bindings, interaction.Engine().clone())
        } else {
            let Some(parent) = self.Frame(frame.parent) else {
                return;
            };
            let Some(window) = parent.navigation.borrow().window.clone() else {
                return;
            };
            (
                window.web_api.WindowBindings().clone(),
                window.web_api.DOMBindings().clone(),
                window.interaction.Engine().clone(),
            )
        };
        let page = self.page.Downgrade();
        window.borrow_mut().EnqueueTaskWithRuntime(
            Box::new(move |runtime, realm| {
                let _delay = delay;
                let present = document
                    .borrow()
                    .GetDocument()
                    .FindNodeById(owner.Id())
                    .is_some();
                if present {
                    DispatchDOMEvent(
                        &document,
                        &bindings,
                        &engine,
                        &page,
                        owner.Id(),
                        EventType::kLoad,
                        runtime,
                        realm,
                    );
                }
            }),
            None,
        );
    }
}
fn DispatchDOMEvent(
    document: &DocumentEngine,
    _bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    engine: &interaction::Interaction<'static>,
    page: &Weak<PageState>,
    id: u64,
    kind: EventType,
    runtime: &mut dyn JavaScriptRuntime,
    realm: &JavaScriptRealm,
) {
    let owner = page.upgrade().and_then(|page| {
        page.local_frames
            .get()
            .and_then(|domain| domain.InteractionForDocument(&document.Handle()))
    });
    if let Some(owner) = owner {
        let mut event = MakeSyntheticEvent(kind, id);
        if kind == EventType::kCustom {
            event.custom_type = "error".into();
        }
        runtime
            .WithHostRuntime(realm, &mut |runtime| {
                owner.WithScriptRuntime(runtime, |engine| {
                    engine.DispatchDOMEvent(&mut event, id, &document.Handle());
                });
            })
            .unwrap_or_else(|error| std::panic::panic_any(error));
        return;
    }
    let runtime = RefCell::new(runtime);
    let listeners = Rc::new(
        |invocation: &EventListenerInvocation<'_>,
         source_document: &interaction::ownership::InteractionDocument,
         _: &interaction::ownership::InteractionDOMMutationEmitter| {
            let Some(page) = page.upgrade() else {
                return Default::default();
            };
            let Some(interaction) = page
                .local_frames
                .get()
                .and_then(|frames| frames.InteractionForDocument(source_document))
            else {
                return Default::default();
            };
            interaction.DispatchListeners(
                source_document,
                invocation,
                &mut **runtime.borrow_mut(),
                realm,
                &mut |error| page.client.borrow_mut().DidReportScriptError(error),
            )
        },
    );
    let mut event = MakeSyntheticEvent(kind, id);
    if kind == EventType::kCustom {
        event.custom_type = "error".into();
    }
    engine
        .WithScopedListeners(listeners)
        .DispatchDOMEvent(&mut event, id, &document.Handle());
}

pub(super) fn FrameCredentialless(frame: &LocalFrame) -> bool {
    let Some(owner) = frame.state.Owner() else {
        return false;
    };
    let Some(document) = frame.owner_document.upgrade() else {
        return false;
    };
    let document = document.borrow();
    let doc = document.GetDocument();
    doc.FindNodeById(owner.Id())
        .is_some_and(|node| doc.Node(node).FindAttribute("credentialless").is_some())
}
impl LocalFrameWindow {
    pub(super) fn RunTaskTurn(&self, milliseconds: f64) {
        self.executor.WithRuntimeAndRealm(&mut |runtime, realm| {
            self.web_api
                .RunTaskTurn(runtime, realm, milliseconds, &mut |error| {
                    if let Some(page) = self.page.upgrade() {
                        page.client.borrow_mut().DidReportScriptError(error);
                    }
                });
            runtime.PerformMicrotaskCheckpoint();
        });
    }
}
impl LocalFrameDomain {
    pub(super) fn ChildTaskDeadline(&self, now: std::time::Instant) -> Option<std::time::Instant> {
        if !self.form_state_tasks.borrow().is_empty() {
            return Some(now);
        }
        if self
            .frames
            .borrow()
            .values()
            .any(|frame| frame.navigation.borrow().HasPendingNavigation())
        {
            return Some(now);
        }
        self.frames
            .borrow()
            .values()
            .filter_map(|frame| {
                frame
                    .navigation
                    .borrow()
                    .window
                    .as_ref()
                    .and_then(|window| window.web_api.NextTaskDeadline(now))
            })
            .min()
    }
}

impl LocalFrameDomain {
    /// Resolve the actual Document's current Window through the original Frame
    /// registry. No DOM/layout projection or borrowed registry entry crosses JS.
    pub(crate) fn InteractionForDocument(
        &self,
        document: &interaction::InteractionDocument,
    ) -> Option<Rc<crate::interaction_services::PageInteraction>> {
        if let Some(page) = self.page.upgrade() {
            if Rc::ptr_eq(&page.document.Handle(), document) {
                return page
                    .popup_interaction
                    .borrow()
                    .upgrade()
                    .filter(|owner| owner.OwnsDocument(document));
            }
        }
        for frame in self.frames.borrow().values() {
            let window = frame.navigation.borrow().window.clone();
            if let Some(window) = window {
                let current = frame
                    .runtime
                    .borrow()
                    .as_ref()
                    .is_some_and(|runtime| Rc::ptr_eq(&runtime.document, &window.document));
                if current && window.interaction.OwnsDocument(document) {
                    return Some(window.interaction.clone());
                }
            }
        }
        None
    }
}

impl LocalFrameWindow {
    pub(super) fn QueueSelectionchangeEvent(&self, target: u64) {
        let document = self.document.clone();
        let interaction = self.interaction.clone();
        let engine = self.interaction.Engine().clone();
        let page = self.page.clone();
        self.web_api.WindowBindings().borrow_mut().EnqueueTaskWithRuntime(Box::new(move |runtime, realm| {
            let connected = {
                let mut owner = document.borrow_mut();
                let document = owner.GetDocumentMut();
                document.ClearScheduledSelectionchangeEvent(target);
                document.IsActive() && document.FindNodeById(target).is_some()
            };
            if connected {
                let runtime = RefCell::new(runtime);
                let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>, source_document: &interaction::ownership::InteractionDocument, _: &interaction::ownership::InteractionDOMMutationEmitter| {
                    interaction.DispatchListeners(source_document, invocation, &mut **runtime.borrow_mut(), realm, &mut |error| { if let Some(page) = page.upgrade() { page.client.borrow_mut().DidReportScriptError(error); } })
                });
                let mut event = MakeSyntheticEvent(EventType::kCustom, target);
                event.custom_type = "selectionchange".into();
                engine.WithScopedListeners(listeners).DispatchDOMEvent(&mut event, target, &document.Handle());
            }
        }), None);
    }
}

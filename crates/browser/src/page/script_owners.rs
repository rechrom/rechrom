//! DocumentLoader::InitializeWindow, document_loader.cc:2784-2868,3027-3044;
//! LocalDOMWindow::InstallNewDocument, local_dom_window.cc:925-953.
//! Each new Window receives new WebAPI/context owners. Preparation never
//! rewrites the old Window or publishes a partially constructed owner graph.
use super::*;
use crate::interaction_services::PageInteraction;
use std::rc::Weak;

/// A Document task borrows the frame's runtime with its original realm, even
/// after ScriptController has selected a new Window. Neither backreference
/// keeps the runtime/realm alive independently of its source owners.
struct PageDocumentScriptExecutor {
    engine: Weak<RefCell<ScriptEngine>>,
    realm: std::cell::OnceCell<javascript::javascript_runtime::WeakJavaScriptRealm>,
}
impl PageDocumentScriptExecutor {
    fn new(engine: &Rc<RefCell<ScriptEngine>>) -> Rc<Self> {
        Rc::new(Self {
            engine: Rc::downgrade(engine),
            realm: Default::default(),
        })
    }
    fn Bind(&self, realm: &javascript::javascript_runtime::JavaScriptRealm) {
        assert!(
            self.realm.set(realm.Downgrade()).is_ok(),
            "one realm per Document executor"
        );
    }
}
impl ScriptExecutor for PageDocumentScriptExecutor {
    fn SupportsModules(&self) -> bool {
        self.engine
            .upgrade()
            .expect("live frame module scheduler")
            .borrow()
            .SupportsModules()
    }
    fn SetModuleCompilationWake(
        &self,
        wake: Option<javascript::javascript_runtime::JavaScriptModuleCompilationWake>,
    ) {
        self.engine
            .upgrade()
            .expect("live frame module scheduler")
            .borrow_mut()
            .SetModuleCompilationWake(wake);
    }
    fn WithRuntimeAndRealm(
        &self,
        callback: &mut dyn FnMut(
            &mut dyn JavaScriptRuntime,
            &javascript::javascript_runtime::JavaScriptRealm,
        ),
    ) {
        let engine = self.engine.upgrade().expect("live frame script task");
        let realm = self
            .realm
            .get()
            .and_then(|realm| realm.Upgrade())
            .expect("task belongs to its original live realm");
        let mut engine = engine.borrow_mut();
        callback(engine.RuntimeMut(), &realm);
    }
}

struct SharedXHRTransport(Rc<RefCell<Box<dyn xhr_transport::XMLHttpRequestTransport>>>);
impl xhr_transport::XMLHttpRequestTransport for SharedXHRTransport {
    fn Start(
        &mut self,
        request: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        self.0.borrow_mut().Start(request)
    }
}
/// Transport service identity is Page scoped; requests/tasks remain owned by
/// their original Window. A new Window never steals the old Window's jobs.
pub(super) struct PageWindowScriptFactory {
    pub(in crate::page) xhr: Rc<RefCell<Box<dyn xhr_transport::XMLHttpRequestTransport>>>,
    pub(in crate::page) user_agent: String,
}
impl PageWindowScriptFactory {
    pub(super) fn new(
        xhr: Box<dyn xhr_transport::XMLHttpRequestTransport>,
        user_agent: String,
    ) -> Self {
        Self {
            xhr: Rc::new(RefCell::new(xhr)),
            user_agent,
        }
    }
    pub(super) fn Create(
        &self,
        state: &Rc<PageState>,
        document: &Rc<document::DocumentEngine>,
        host: DOMBindingsHost,
        timing: Rc<webapi::window_bindings::WindowPerformanceTiming>,
    ) -> WebApiEngine {
        let effects = self.Effects(state, document);
        WebApiEngine::WithPerformanceTiming(
            document.Handle(),
            host,
            Box::new(SharedXHRTransport(self.xhr.clone())),
            self.user_agent.clone(),
            effects,
            timing,
        )
    }
    fn CreateForReuse(&self, state: &Rc<PageState>, previous: &WebApiEngine) -> WebApiEngine {
        WebApiEngine::ForWindowReuse(
            state.document.Handle(),
            state.CreateWebApiDOMHost(),
            previous,
            self.Effects(state, &state.document),
        )
    }
    fn Effects(
        &self,
        state: &Rc<PageState>,
        document: &Rc<document::DocumentEngine>,
    ) -> Rc<dyn Fn(WebApiEffect)> {
        let page = Rc::downgrade(state);
        let original = document.clone();
        let identity = document.borrow().GetDocument().RootHandle();
        Rc::new(move |effect: WebApiEffect| {
            let active = page.upgrade().filter(|page| {
                let document = page.document.borrow();
                document.GetDocument().RootHandle() == identity && document.GetDocument().IsActive()
            });
            match effect {
                WebApiEffect::DocumentMutation(mutation) => {
                    if let Some(page) = active {
                        page.ApplyMutation(PageMutation::DOMMutation(mutation));
                    } else {
                        original
                            .ApplyDOMMutation(&mutation)
                            .unwrap_or_else(|error| std::panic::panic_any(error));
                    }
                }
                WebApiEffect::Navigate(navigation) => {
                    if let Some(page) = active {
                        page.client
                            .borrow_mut()
                            .DidRequestNavigation(&NavigationRequest {
                                request: URLRequest {
                                    url: navigation.url,
                                    referrer: navigation.referrer,
                                    ..Default::default()
                                },
                                target: "_self".into(),
                                replace_history: navigation.replace_history,
                                host: None,
                            });
                    }
                }
                WebApiEffect::AnimationTick { monotonic_time_ms } => {
                    if let Some(page) = active {
                        let last_time = page.animation_engine.borrow().LastTime();
                        page.ApplyAnimationMutation(animation::AnimationMutation::Tick(
                            animation::AnimationTick {
                                monotonic_time: (monotonic_time_ms / 1000.0).max(last_time),
                                ..Default::default()
                            },
                        ));
                    }
                }
            }
        })
    }
}

/// Retained by the original DOM binding, independently of which Window is
/// current in PageScripts. All backreferences to Frame/executor/bindings are
/// weak in these source owners; this lease does not form a realm/host cycle.
pub(super) struct PageWindowDocumentOwners {
    document: Rc<document::DocumentEngine>,
    _interaction: Rc<PageInteraction>,
    _executor: javascript::script_engine::ScriptExecutorHandle,
}
impl webapi::dom_bindings::NativeDOMDocumentOwner for PageWindowDocumentOwners {
    fn DocumentIdentity(&self) -> dom::persistent_document::DOMOwnerHandle {
        self.document.borrow().GetDocument().RootHandle()
    }
}
/// The single immutable document/Window owner bundle. It deliberately does
/// not own its realm or ScriptEngine. The Window retains only the smaller
/// Document-owner lease; no retired Page snapshots accumulate in the runtime.
pub(super) struct PageWindowScriptSnapshot {
    pub(super) state: Rc<PageState>,
    pub(super) document: Rc<document::DocumentEngine>,
    pub(super) web_api: WebApiEngine,
    pub(super) document_owners: Rc<PageWindowDocumentOwners>,
    pub(super) interaction: Rc<PageInteraction>,
    pub(super) navigation: Rc<super::super::navigation_api::NavigationWindowEnvironment>,
    pub(super) executor: javascript::script_engine::ScriptExecutorHandle,
    document_executor: Rc<PageDocumentScriptExecutor>,
    script: Weak<RefCell<ScriptEngine>>,
}
impl PageWindowScriptSnapshot {
    pub(super) fn Create(
        state: &Rc<PageState>,
        document: Rc<document::DocumentEngine>,
        web_api: WebApiEngine,
        script: &Rc<RefCell<ScriptEngine>>,
        initial: bool,
        reuse_navigation: Option<Rc<super::super::navigation_api::NavigationWindowEnvironment>>,
    ) -> Rc<Self> {
        let bindings = web_api.DOMBindings().clone();
        let document_executor = PageDocumentScriptExecutor::new(script);
        if initial {
            document_executor.Bind(script.borrow().Realm());
        }
        let executor: javascript::script_engine::ScriptExecutorHandle = document_executor.clone();
        let page = Rc::downgrade(state);
        let original = document.clone();
        let popup_interaction: Rc<RefCell<Weak<PageInteraction>>> = Default::default();
        let popup_mutations = popup_interaction.clone();
        bindings
            .borrow_mut()
            .SetScopedMutationEmitter(Rc::new(move |mutation, runtime| {
                let active = page.upgrade().filter(|page| {
                    let current = page.document.borrow();
                    current.GetDocument().RootHandle()
                        == original.borrow().GetDocument().RootHandle()
                        && current.GetDocument().IsActive()
                });
                let Some(state) = active else {
                    return original.ApplyDOMMutation(mutation).map(|_| ());
                };
                let popup = popup_mutations
                    .borrow()
                    .upgrade()
                    .expect("current Document interaction owner");
                if popup
                    .ApplyPopupMutation(mutation, runtime)
                    .map_err(super::super::popup_tasks::PopupMutationError)?
                {
                    popup
                        .DrainPopupLifecycle(runtime)
                        .map_err(super::super::popup_tasks::PopupMutationError)?;
                    return Ok(());
                }
                let resources = state.resources.borrow().clone();
                let client = RefCell::new(ScopedResourceClient {
                    state: state.clone(),
                    runtime,
                });
                let result = state.TryApplyDOMMutation(mutation, &mut |id, loaded| {
                    if let Some(resources) = &resources {
                        resources.DispatchImageEvent(id, loaded, &mut *client.borrow_mut());
                    }
                });
                // The normal mutation has released all DOM borrows. Resume
                // its source lifecycle in this same host runtime before return.
                let mut client = client.into_inner();
                popup
                    .DrainPopupLifecycle(client.runtime)
                    .map_err(super::super::popup_tasks::PopupMutationError)?;
                result
            }));
        let identity = document.borrow().GetDocument().RootHandle();
        let page = Rc::downgrade(state);
        bindings
            .borrow_mut()
            .SetCanvasDirtyHandler(Rc::new(move |_| {
                if let Some(page) = page.upgrade().filter(|page| {
                    let document = page.document.borrow();
                    document.GetDocument().RootHandle() == identity
                        && document.GetDocument().IsActive()
                }) {
                    page.paint_resources_dirty.set(true);
                    page.dirty.set(true);
                }
            }));
        let mutate = Rc::downgrade(state);
        let original = document.clone();
        let interaction = Rc::new(PageInteraction::InstallWithEmitter(
            document.Handle(),
            &bindings,
            state.interaction.clone(),
            Some(Rc::new(move |output| {
                let active = mutate.upgrade().filter(|page| {
                    let current = page.document.borrow();
                    current.GetDocument().RootHandle()
                        == original.borrow().GetDocument().RootHandle()
                        && current.GetDocument().IsActive()
                });
                if let Some(page) = active {
                    page.ApplyInteractionOutput(output);
                } else if let ::interaction::InteractionOutput::DocumentMutation(mutation) = output
                {
                    original
                        .ApplyDOMMutation(&mutation)
                        .unwrap_or_else(|error| std::panic::panic_any(error));
                }
            })),
            state.layout_editing.selections.clone(),
            state.InteractionEventClock(),
        ));
        interaction
            .Engine()
            .BindWindowPerformance(state.window_performance_timing.EventTimingSource());
        interaction
            .Engine()
            .FormControlState()
            .SetValidationMessageClient(
                super::super::validation_message::PageValidationMessageClient::new(state),
            );
        interaction.BindFrameDomain(Rc::downgrade(state.local_frames.get().unwrap()));
        if initial {
            interaction.BindRealm(script.borrow().Realm());
        }
        if let Some(reader) = state.native_file_reader.borrow().clone() {
            interaction.SetFormStateFileReader(reader);
        }
        let form_data_page = Rc::downgrade(state);
        interaction.SetFormDataTransaction(Rc::new(move |engine, document, token| {
            use interaction::dispatch::core::fileapi::form_data_object::FormDataConstructionError;
            let page = form_data_page
                .upgrade()
                .ok_or(FormDataConstructionError::InvalidState(
                    "The form's Page is no longer active.",
                ))?;
            page.CompleteFormDataConstruction(engine, document, token)
        }));
        let submission_page = Rc::downgrade(state);
        interaction.SetFormSubmissionTransaction(Rc::new(move |engine, document| {
            if let Some(page) = submission_page.upgrade() {
                page.CompleteFormSubmissions(engine, document);
            }
        }));
        let client = state.client.clone();
        *popup_interaction.borrow_mut() = Rc::downgrade(&interaction);
        *state.popup_interaction.borrow_mut() = Rc::downgrade(&interaction);
        interaction.SetScriptErrorReporter(Rc::new(move |error| {
            client.borrow_mut().DidReportScriptError(error)
        }));
        let document_owners = Rc::new(PageWindowDocumentOwners {
            document: document.clone(),
            _interaction: interaction.clone(),
            _executor: executor.clone(),
        });
        bindings
            .borrow_mut()
            .BindNativeDocumentOwner(document_owners.clone());
        let navigation = if let Some(navigation) = reuse_navigation {
            navigation.Rebind(
                state,
                &document,
                &web_api,
                &interaction,
                state.window_performance_timing.ExistingEventTimingSource(),
            );
            navigation
        } else {
            super::super::navigation_api::NavigationWindowEnvironment::Install(
                Rc::downgrade(state),
                Weak::new(),
                &document,
                &web_api,
                &interaction,
                state.window_performance_timing.ExistingEventTimingSource(),
            )
        };
        Rc::new(Self {
            navigation,
            state: state.clone(),
            document,
            web_api,
            document_owners,
            interaction,
            executor,
            document_executor,
            script: Rc::downgrade(script),
        })
    }
    pub(super) fn InteractionEnvironment(&self) -> PageScriptInteractionEnvironment {
        PageScriptInteractionEnvironment {
            bindings: self.web_api.DOMBindings().clone(),
            script: self.script.upgrade().expect("live frame ScriptController"),
            window: self.web_api.WindowBindings().clone(),
            executor: self.executor.clone(),
        }
    }
    pub(super) fn OpenServices(&self) -> OpenScriptServices {
        OpenScriptServices {
            bindings: self.web_api.DOMBindings().clone(),
            pending_host_errors: Some(self.interaction.ExceptionSource()),
        }
    }
}
pub(in crate::page) struct PreparedPageWindow {
    pub(super) expected: Rc<PageWindowScriptSnapshot>,
    pub(super) next: Rc<PageWindowScriptSnapshot>,
    pub(super) state: Rc<PageState>,
}
pub(in crate::page) struct InstalledPageWindow {
    pub(super) prepared: PreparedPageWindow,
    pub(super) old_realm: javascript::javascript_runtime::JavaScriptRealm,
    pub(super) bootstrap_errors: Vec<JavaScriptException>,
    disposition: document_loader::StaticDocumentWindowDisposition,
}
impl InstalledPageWindow {
    pub(in crate::page) fn Engine(&self) -> ::interaction::Interaction<'static> {
        self.prepared.next.interaction.Engine().clone()
    }
    pub(in crate::page) fn InteractionEnvironment(&self) -> PageScriptInteractionEnvironment {
        self.prepared.next.InteractionEnvironment()
    }
    pub(in crate::page) fn Document(&self) -> Rc<document::DocumentEngine> {
        self.prepared.next.document.clone()
    }
    pub(in crate::page) fn State(&self) -> Rc<PageState> {
        self.prepared.state.clone()
    }
    pub(in crate::page) fn Executor(&self) -> javascript::script_engine::ScriptExecutorHandle {
        self.prepared.next.executor.clone()
    }
    pub(in crate::page) fn OpenServices(&self) -> OpenScriptServices {
        self.prepared.next.OpenServices()
    }
    pub(in crate::page) fn BootstrapErrors(&self) -> &[JavaScriptException] {
        &self.bootstrap_errors
    }
}
pub(in crate::page) struct RetiredPageWindow {
    pub(super) realm: javascript::javascript_runtime::JavaScriptRealm,
    pub(super) owners: Rc<PageWindowDocumentOwners>,
}
#[derive(Debug)]
pub(in crate::page) enum PageWindowPublishError {
    StaleWindow,
    SameDocument,
    DocumentNotInstalled,
    RequiresNewWindow,
}

impl PageScripts {
    /// Prepare the new Document domain before any resource discovery.
    pub(in crate::page) fn PrepareNewWindow(
        &self,
        state: Rc<PageState>,
    ) -> Result<PreparedPageWindow, PageWindowPublishError> {
        if state.document.borrow().GetDocument().RootHandle()
            == self.owners.document.borrow().GetDocument().RootHandle()
        {
            return Err(PageWindowPublishError::SameDocument);
        }
        let web_api = self.window_factory.Create(
            &state,
            &state.document,
            state.CreateWebApiDOMHost(),
            state.window_performance_timing.clone(),
        );
        {
            let constraints = state.constraints.borrow();
            web_api.ApplyMutation(WebApiMutation::SetViewport {
                width: constraints.available_size.width,
                height: constraints.available_size.height,
                scale: constraints.device_pixel_ratio,
            });
            web_api.ApplyMutation(WebApiMutation::SetPrefersReducedMotion(
                constraints.prefers_reduced_motion,
            ));
        }
        web_api.ApplyMutation(WebApiMutation::SetPreferredColorScheme(
            state.preferred_color_scheme.get(),
        ));
        web_api.ApplyMutation(WebApiMutation::SetForcedColors(state.forced_colors.get()));
        web_api.ApplyMutation(WebApiMutation::SetURL(
            state
                .document
                .borrow()
                .GetDocument()
                .DocumentURL()
                .to_owned(),
        ));
        let next = PageWindowScriptSnapshot::Create(
            &state,
            state.document.clone(),
            web_api,
            &self.script,
            false,
            None,
        );
        next.navigation.SetPreviousWindow(&self.owners.navigation);
        Ok(PreparedPageWindow {
            expected: self.owners.clone(),
            next,
            state,
        })
    }

    /// Only DocumentLoader's source-selected NewWindow branch calls this.
    /// No caller's invocation/runtime borrow may cross this entry. Bootstrap
    /// installs trusted WebIDL infrastructure; it runs no parser/author script.
    pub(in crate::page) fn CreateNewWindowRealm(
        &self,
        prepared: PreparedPageWindow,
        choice: &document_loader::StaticDocumentWindowChoice,
    ) -> Result<InstalledPageWindow, PageWindowPublishError> {
        if !Rc::ptr_eq(&self.owners, &prepared.expected) {
            return Err(PageWindowPublishError::StaleWindow);
        }
        if choice.SourceDocument()
            != prepared
                .expected
                .document
                .borrow()
                .GetDocument()
                .RootHandle()
        {
            return Err(PageWindowPublishError::StaleWindow);
        }
        if choice.Disposition() != document_loader::StaticDocumentWindowDisposition::NewWindow {
            return Err(PageWindowPublishError::RequiresNewWindow);
        }
        // LocalDOMWindow::Reset -> FrameDestroyed (1119-1168). Document
        // Shutdown has already completed in FrameLoader; execution-context
        // destruction belongs only to this NewWindow branch.
        self.owners
            .web_api
            .WindowBindings()
            .borrow_mut()
            .ResetForNewWindow();
        let mut script = self.script.borrow_mut();
        let old_realm = script.InstallNewWindow(prepared.next.web_api.WindowBindings().clone());
        let realm = script.Realm().clone();
        prepared
            .next
            .web_api
            .BindInitialHostObjectNamespace(script.RuntimeMut(), &realm)
            .unwrap_or_else(|error| std::panic::panic_any(error));
        prepared.next.document_executor.Bind(script.Realm());
        prepared.next.web_api.BindRuntime(script.Realm());
        prepared.next.interaction.BindRealm(script.Realm());
        let bootstrap_errors = script.EvaluateBootstrap(&WebApiEngine::BootstrapSources());
        Ok(InstalledPageWindow {
            prepared,
            old_realm,
            bootstrap_errors,
            disposition: choice.Disposition(),
        })
    }

    /// Same Context/global and Window native identity; fresh Document bindings
    /// have their own HostGraph and wrapper namespace. No WebIDL bootstrap runs.
    pub(in crate::page) fn ReuseInitialWindow(
        &self,
        state: Rc<PageState>,
        choice: &document_loader::StaticDocumentWindowChoice,
    ) -> Result<InstalledPageWindow, PageWindowPublishError> {
        if choice.Disposition()
            != document_loader::StaticDocumentWindowDisposition::ReuseInitialWindow
            || choice.SourceDocument() != self.owners.document.borrow().GetDocument().RootHandle()
        {
            return Err(PageWindowPublishError::StaleWindow);
        }
        if state.document.borrow().GetDocument().RootHandle()
            == self.owners.document.borrow().GetDocument().RootHandle()
        {
            return Err(PageWindowPublishError::SameDocument);
        }
        assert!(Rc::ptr_eq(
            &state.window_performance_timing,
            &self.owners.state.window_performance_timing
        ));
        state.BindWindowEventTiming(
            state.local_frames.get().unwrap().root,
            &state.document,
            &state.window_performance_timing,
        );
        {
            let mut script = self.script.borrow_mut();
            let realm = script.Realm().clone();
            script
                .RuntimeMut()
                .WithHostRuntime(&realm, &mut |runtime| {
                    self.owners.navigation.AbortForWindowReuse(runtime)
                })
                .unwrap_or_else(|error| std::panic::panic_any(error));
        }
        let web_api = self
            .window_factory
            .CreateForReuse(&state, &self.owners.web_api);
        let realm = {
            let mut script = self.script.borrow_mut();
            let realm = script.Realm().clone();
            web_api
                .PrepareHostObjectNamespaceForReuse(script.RuntimeMut(), &realm)
                .unwrap_or_else(|error| std::panic::panic_any(error));
            realm
        };
        let next = PageWindowScriptSnapshot::Create(
            &state,
            state.document.clone(),
            web_api,
            &self.script,
            true,
            Some(self.owners.navigation.clone()),
        );
        Ok(InstalledPageWindow {
            prepared: PreparedPageWindow {
                expected: self.owners.clone(),
                next,
                state,
            },
            old_realm: realm,
            bootstrap_errors: Vec::new(),
            disposition: choice.Disposition(),
        })
    }

    /// Called after the new Document services have been assembled. These
    /// stores invoke no script/callback and borrow no DOM. Every task following
    /// this synchronous commit reads one new immutable owner bundle.
    pub(in crate::page) fn PublishNewWindow(
        &mut self,
        installed: InstalledPageWindow,
    ) -> Result<RetiredPageWindow, PageWindowPublishError> {
        if !Rc::ptr_eq(&self.owners, &installed.prepared.expected) {
            return Err(PageWindowPublishError::StaleWindow);
        }
        let PreparedPageWindow {
            expected: _,
            next,
            state,
        } = installed.prepared;
        if state.document.borrow().GetDocument().RootHandle()
            != next.document.borrow().GetDocument().RootHandle()
        {
            return Err(PageWindowPublishError::DocumentNotInstalled);
        }
        self.owners
            .web_api
            .DOMBindings()
            .borrow_mut()
            .DisassociateWindowDocument();
        if installed.disposition
            == document_loader::StaticDocumentWindowDisposition::ReuseInitialWindow
        {
            let mut script = self.script.borrow_mut();
            let realm = script.Realm().clone();
            next.web_api
                .PublishDocumentForReuse(script.RuntimeMut(), &realm);
        }
        next.web_api
            .ApplyMutation(WebApiMutation::SetBeginFrameSource(
                state.begin_frame_source.borrow().clone(),
            ));
        state.PublishPreparedDocumentInputs();
        state.ConnectDOMBindings(next.web_api.DOMBindings());
        if let Some(factory) = state.local_frames.get().unwrap().script_factory.get() {
            factory.RebindRootWindow(
                next.web_api.DOMBindings(),
                next.web_api.WindowBindings(),
                &next.interaction,
            );
        }
        self.flush_dynamic_scripts = None;
        let owners = std::mem::replace(&mut self.owners, next);
        Ok(RetiredPageWindow {
            realm: installed.old_realm,
            owners: owners.document_owners.clone(),
        })
    }
}

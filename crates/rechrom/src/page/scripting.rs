//! Optional script execution environment; Page owns all navigation and frames.
use super::*;

pub(super) struct PageScripts {
    state: Rc<PageState>,
    engine: interaction::Interaction<'static>,
    runtime: Box<dyn JavaScriptRuntime>,
    realm: JavaScriptRealm,
    bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    window: Rc<RefCell<WindowJavaScriptBindings>>,
    interaction: Option<PageInteraction>,
    scheduler: Option<ScriptScheduler>,
    script_client: Rc<RefCell<dyn ScriptLoadClient>>,
}
impl PageScripts {
    pub(super) fn HasBlockingWheelListener(&self) -> bool {
        self.bindings.borrow().HasBlockingListener("wheel")
    }
    pub(super) fn BlockingWheelListenerTargets(&self) -> Vec<u64> {
        self.bindings.borrow().BlockingListenerTargets("wheel")
    }
    pub(super) fn SetModuleCompilationWake(&mut self, callback: URLWakeCallback) {
        self.runtime.SetModuleCompilationWake(Some(callback));
    }
    // cpp: browser/browser.cc:610-737
    pub(super) fn new(
        state: Rc<PageState>,
        environment: ScriptEnvironment,
        script_client: Rc<RefCell<dyn ScriptLoadClient>>,
    ) -> Self {
        let ScriptEnvironment {
            mut runtime,
            xhr,
            user_agent,
        } = environment;
        let client = state.client.clone();
        let bindings = state.ConnectDOM();
        let mutate = state.clone();
        let interaction = PageInteraction::InstallWithEmitter(
            state.document.clone(),
            &bindings,
            state.interaction.clone(),
            Some(Rc::new(move |m| mutate.ApplyMutation(m))),
            state.layout_editing.selections.clone(),
        );
        let error_client = client.clone();
        interaction.SetScriptErrorReporter(Rc::new(move |error| {
            error_client.borrow_mut().DidReportScriptError(error);
        }));
        let engine = interaction.Engine().clone();
        let scoped_state = state.clone();
        let scoped_bindings = Rc::downgrade(&bindings);
        bindings
            .borrow_mut()
            .SetScopedMutationEmitter(Rc::new(move |mutation, runtime| {
                let bindings = scoped_bindings.upgrade().expect("live Page DOM bindings");
                let mut notification = bindings.borrow().PrepareMutationNotification(mutation);
                let resources = scoped_state.resources.borrow().clone();
                let client = RefCell::new(ScopedResourceClient {
                    state: scoped_state.clone(),
                    runtime,
                });
                scoped_state.ApplyDOMMutation(
                    mutation,
                    &mut || {
                        DOMJavaScriptBindings::DeliverMutationNotification(
                            &bindings,
                            notification.take(),
                            &mut |callback, args| {
                                client.borrow_mut().runtime.Call(
                                    callback,
                                    &javascript::javascript_runtime::HostValue::Object(
                                        javascript::javascript_runtime::HostObjectRef { id: 0 },
                                    ),
                                    args,
                                );
                            },
                        );
                    },
                    &mut |id, loaded| {
                        resources
                            .as_ref()
                            .expect("cached image has Page resources")
                            .DispatchImageEvent(id, loaded, &mut *client.borrow_mut());
                    },
                );
            }));
        let tick = state.clone();
        let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
            bindings.clone(),
            xhr,
            user_agent,
            Some(Box::new(move |ms| {
                tick.ApplyMutation(PageMutation::AnimationTick(page_mutation::AnimationTick {
                    // A delayed BeginFrame may predate an ordinary task's
                    // animation sample. Only empty-sample bookkeeping clamps;
                    // rAF timestamps and actual style samples remain exact.
                    monotonic_time: (ms / 1000.0).max(tick.animation_time.get()),
                    ..Default::default()
                }))
            })),
        )));
        let navigation_client = client.clone();
        window
            .borrow_mut()
            .SetLocationNavigator(Rc::new(move |navigation| {
                navigation_client
                    .borrow_mut()
                    .DidRequestNavigation(&NavigationRequest {
                        request: URLRequest {
                            url: navigation.url,
                            referrer: navigation.referrer,
                            ..Default::default()
                        },
                        target: "_self".into(),
                        replace_history: navigation.replace_history,
                    });
            }));
        {
            let c = state.constraints.borrow();
            window.borrow_mut().SetViewport(
                c.available_size.width,
                c.available_size.height,
                c.device_pixel_ratio,
            );
        }
        let realm = runtime.CreateRealm(window.clone());
        bindings.borrow_mut().BindRuntime(&realm);
        window.borrow_mut().BindRuntime(&realm);
        for (source, name) in [
            (
                webapi::DOMBootstrapSource().to_owned(),
                "browser:dom-webidl",
            ),
            (
                WindowJavaScriptBindings::BootstrapSource(),
                "browser:window-webidl",
            ),
        ] {
            let started = std::env::var_os("BROWSER_PROFILE_INPUT")
                .is_some()
                .then(std::time::Instant::now);
            let result = runtime.EvaluateBootstrap(&realm, &source, name);
            if let Some(start) = started {
                eprintln!(
                    "javascript-realm-profile phase=bootstrap name={name} bytes={} ms={:.3}",
                    source.len(),
                    start.elapsed().as_secs_f64() * 1000.
                );
            }
            if let Some(error) = result.exception {
                client.borrow_mut().DidReportScriptError(&error);
            }
        }
        Self {
            state,
            engine,
            runtime,
            realm,
            bindings,
            window,
            interaction: Some(interaction),
            scheduler: None,
            script_client,
        }
    }
    pub(super) fn Engine(&self) -> interaction::Interaction<'static> {
        self.engine.clone()
    }
    pub(super) fn SetURL(&self, url: String) {
        self.window.borrow_mut().SetURL(url);
    }
    pub(super) fn SetPreferredColorScheme(&self, preference: PreferredColorScheme) {
        self.window.borrow_mut().SetPreferredColorScheme(preference);
    }
    pub(super) fn ResizeViewport(&self, width: f64, height: f64, scale: f64) {
        self.window.borrow_mut().SetViewport(width, height, scale);
    }
    pub(super) fn FinishStyleSheets(&mut self) -> io::Result<()> {
        self.scheduler
            .as_mut()
            .expect("parser scheduler installed")
            .FinishStyleSheets(
                None,
                &mut *self.runtime,
                &self.realm,
                &mut *self.script_client.borrow_mut(),
            )
    }
    pub(super) fn HasPendingDynamicScripts(&self) -> bool {
        self.scheduler
            .as_ref()
            .is_some_and(|s| s.HasPendingDynamicScripts())
    }
    pub(super) fn StopModuleLoading(&self) {
        if let Some(scheduler) = &self.scheduler {
            scheduler.StopModuleLoading();
        }
    }
    pub(super) fn FinishLoad(&mut self) {
        if let Some(scheduler) = &mut self.scheduler {
            scheduler.ConsumeTaskBudget();
        }
        self.bindings.borrow_mut().SetReadyState("complete".into());
        for kind in [EventType::kReadyStateChange, EventType::kLoad] {
            self.scheduler.as_mut().unwrap().DispatchLifecycleEvent(
                kind,
                &mut *self.runtime,
                &self.realm,
                &mut *self.script_client.borrow_mut(),
            );
        }
        self.FlushTasks();
    }
    pub(super) fn DidPaint(&mut self) {
        if let Some(scheduler) = &self.scheduler {
            scheduler.Interaction().DidPaint();
        }
        // Chromium recomputes intersection observations from the committed
        // post-layout geometry, not only when a scroll event is dispatched.
        // The JS helper only queues observer delivery, so callbacks remain a
        // later task and cannot re-enter the lifecycle that just painted.
        let result = self.runtime.Evaluate(
            &self.realm,
            "globalThis.__browserUpdateIntersectionObservations?.()",
            "browser:intersection-observer-update",
        );
        if let Some(error) = result.exception {
            self.state.client.borrow_mut().DidReportScriptError(&error);
        }
    }
    /// Dispatch the CSSOM View per-frame scroll event queue before rAF.
    /// Blink's Document::EnqueueScrollEventForNode stores these events in the
    /// ScriptedAnimationController; they must not wait behind ordinary network,
    /// timer, or image tasks.
    pub(super) fn DispatchPendingScrollEvents(&mut self) {
        let targets = std::mem::take(&mut *self.state.scroll_event_targets.borrow_mut());
        if targets.is_empty() {
            return;
        }
        let mut trace = browser_tracing::span("lifecycle", "Page.DispatchPendingScrollEvents");
        trace.set("targets", targets.len() as f64);
        for target in targets {
            let (exists, is_document) = {
                let owner = self.state.document.borrow();
                let tree = owner.GetDocument();
                (
                    tree.FindNodeById(target).is_some(),
                    tree.Node(tree.Root()).Id() == target,
                )
            };
            if !exists {
                continue;
            }
            let mut event = MakeSyntheticEvent(EventType::kCustom, target);
            event.custom_type = "scroll".into();
            event.bubbles = is_document;
            event.cancelable = false;
            event.composed = false;
            let runtime_cell = RefCell::new(&mut *self.runtime);
            let listeners = Rc::new(
                |invocation: &EventListenerInvocation<'_>,
                 _: &interaction::ownership::InteractionDocument,
                 _: &interaction::ownership::InteractionDOMMutationEmitter| {
                    DispatchListeners(
                        &self.state,
                        &self.bindings,
                        invocation,
                        &mut **runtime_cell.borrow_mut(),
                        &self.realm,
                    )
                },
            );
            self.engine.WithScopedListeners(listeners).DispatchDOMEvent(
                &mut event,
                target,
                &self.state.document,
            );
        }
        self.Checkpoint();
    }
    pub(super) fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.state.QueueImageEvent(id, loaded);
    }
    // cpp: browser/browser.cc:1646-1680
    // Direct PageMutation parsing shares the navigation parser, realm and arena,
    // but does not commit a URL or dispatch navigation complete/load events.
    pub(super) fn ParseDocument(
        &mut self,
        document_loader: &mut DocumentLoader,
        loader: Rc<RefCell<dyn URLLoader>>,
        resources: Rc<ResourceFetcher>,
        current_url: &str,
        html: &str,
        chunk_size: usize,
        token_budget: usize,
    ) -> io::Result<()> {
        self.PrepareDocumentLoader(loader, resources, current_url)?;
        self.scheduler.as_mut().unwrap().ParseDocumentWithLoader(
            document_loader,
            html,
            chunk_size,
            token_budget,
            &mut *self.runtime,
            &self.realm,
            &mut *self.script_client.borrow_mut(),
        )?;
        Ok(())
    }
    fn PrepareDocumentLoader(
        &mut self,
        loader: Rc<RefCell<dyn URLLoader>>,
        resources: Rc<ResourceFetcher>,
        current_url: &str,
    ) -> io::Result<()> {
        self.state.CancelImageEvents();
        let mut scheduler = ScriptScheduler::WithPageInteraction(
            self.state.document.clone(),
            self.bindings.clone(),
            loader.clone(),
            current_url.to_owned(),
            self.interaction.take().unwrap(),
        );
        scheduler.InstallResourceFetcher(resources)?;
        let dynamic = Rc::new(DynamicScriptTasks::new(
            self.state.document.clone(),
            self.bindings.clone(),
            loader.clone(),
            current_url.to_owned(),
            scheduler.StartedScripts(),
            self.runtime.SupportsModules(),
        ));
        scheduler.InstallDynamicScripts(&dynamic)?;
        *self.state.dynamic.borrow_mut() = Some(dynamic.clone());
        let weak_window = Rc::downgrade(&self.window);
        let weak_dynamic = Rc::downgrade(&dynamic);
        let client = self.script_client.clone();
        let weak_state = Rc::downgrade(&self.state);
        let weak_bindings = Rc::downgrade(&self.bindings);
        let engine = self.engine.clone();
        self.window
            .borrow_mut()
            .SetTaskRegistrationFlusher(Rc::new(move || {
                if let (Some(window), Some(dynamic)) =
                    (weak_window.upgrade(), weak_dynamic.upgrade())
                {
                    dynamic.EnqueuePreparedTasks(&window, client.clone());
                    if let (Some(state), Some(bindings)) =
                        (weak_state.upgrade(), weak_bindings.upgrade())
                    {
                        EnqueueImageEventTasks(&state, &window, &bindings, &engine);
                    }
                }
            }));
        self.scheduler = Some(scheduler);
        self.state.measurement.borrow_mut().take();
        Ok(())
    }
    pub(super) fn BeginDocumentLoader(
        &mut self,
        document_loader: &mut DocumentLoader,
        loader: Rc<RefCell<dyn URLLoader>>,
        resources: Rc<ResourceFetcher>,
        current_url: &str,
    ) -> io::Result<()> {
        self.PrepareDocumentLoader(loader, resources, current_url)?;
        self.scheduler.as_mut().unwrap().BeginTaskTurn(true);
        self.scheduler
            .as_mut()
            .unwrap()
            .BeginDocumentLoader(document_loader, &*self.runtime)
    }
    pub(super) fn PumpDocumentLoader(
        &mut self,
        document_loader: &mut DocumentLoader,
        budget: document_loader::DocumentLoadBudget,
    ) -> io::Result<document_loader::DocumentLoadProgress> {
        self.scheduler.as_mut().unwrap().PumpDocumentLoader(
            document_loader,
            budget,
            &mut *self.runtime,
            &self.realm,
            &mut *self.script_client.borrow_mut(),
        )
    }
    pub(super) fn HasPendingRenderBlockingStyleSheets(&self) -> bool {
        self.scheduler
            .as_ref()
            .is_some_and(|s| s.HasPendingRenderBlockingStyleSheets())
    }
    pub(super) fn TraceLoadingState(&self) {
        if let Some(scheduler) = &self.scheduler {
            scheduler.TraceLoadingState();
        }
    }
    pub(super) fn PollParsingCompletion(&mut self) -> io::Result<bool> {
        self.scheduler.as_mut().unwrap().PollParsingCompletion(
            &mut *self.runtime,
            &self.realm,
            &mut *self.script_client.borrow_mut(),
        )
    }
    #[cfg(test)]
    pub(super) fn DrainImageEventTasks(&mut self) {
        while !self.state.pending_image_events.borrow().is_empty() {
            self.FlushTasks();
            let client = self.state.client.clone();
            WindowJavaScriptBindings::RunTasks(
                &self.window,
                &mut *self.runtime,
                &self.realm,
                0.0,
                &mut |error| client.borrow_mut().DidReportScriptError(error),
            );
        }
    }
    pub(super) fn FlushTasks(&self) {
        EnqueueImageEventTasks(&self.state, &self.window, &self.bindings, &self.engine);
        if let Some(dynamic) = self.state.dynamic.borrow().as_ref() {
            dynamic.EnqueuePreparedTasks(&self.window, self.script_client.clone());
        }
    }
    fn Checkpoint(&mut self) {
        self.runtime.PerformMicrotaskCheckpoint();
        for error in self.runtime.TakePendingExceptions(&self.realm) {
            self.state.client.borrow_mut().DidReportScriptError(&error);
        }
        if let Some(scheduler) = &self.scheduler {
            scheduler.ReportInteractionErrors(&mut *self.script_client.borrow_mut());
        }
        self.FlushTasks();
    }
    // cpp: browser/browser.cc:905-920
    pub(super) fn Evaluate(
        &mut self,
        source: &str,
        source_name: &str,
    ) -> io::Result<JavaScriptResult> {
        let result = self.runtime.Evaluate(&self.realm, source, source_name);
        if let Some(error) = &result.exception {
            self.state.client.borrow_mut().DidReportScriptError(error);
        }
        self.Checkpoint();
        Ok(result)
    }
    pub(super) fn SetBeginFrameSource(
        &self,
        source: Option<std::sync::Arc<dyn foundation::begin_frame::BeginFrameSource>>,
    ) {
        self.window.borrow_mut().SetBeginFrameSource(source);
    }
    pub(super) fn HasPendingAnimationFrames(&self) -> bool {
        self.window.borrow().HasPendingAnimationFrames()
    }
    pub(super) fn RunAnimationFrameCallbacks(
        &mut self,
        frame_time: std::time::Instant,
    ) -> io::Result<()> {
        let client = self.state.client.clone();
        WindowJavaScriptBindings::RunAnimationFrameCallbacks(
            &self.window,
            &mut *self.runtime,
            &self.realm,
            frame_time,
            &mut |error| client.borrow_mut().DidReportScriptError(error),
        );
        if let Some(scheduler) = &self.scheduler {
            scheduler.ReportInteractionErrors(&mut *self.script_client.borrow_mut());
        }
        self.FlushTasks();
        Ok(())
    }
    pub(super) fn RunTasks(&mut self, milliseconds: f64) -> io::Result<()> {
        self.FlushTasks();
        if let Some(scheduler) = &mut self.scheduler {
            if !scheduler.HasTaskBudget() {
                return Ok(());
            }
            let progressed = scheduler.PumpModules(
                &mut *self.runtime,
                &self.realm,
                if milliseconds > 0.0 { 16 } else { 1 },
            );
            if progressed != 0 && !(milliseconds > 0.0) {
                scheduler.ConsumeTaskBudget();
                return Ok(());
            }
        }
        let client = self.state.client.clone();
        WindowJavaScriptBindings::RunTasks(
            &self.window,
            &mut *self.runtime,
            &self.realm,
            milliseconds,
            &mut |error| client.borrow_mut().DidReportScriptError(error),
        );
        if let Some(scheduler) = &self.scheduler {
            scheduler.ReportInteractionErrors(&mut *self.script_client.borrow_mut());
        }
        self.FlushTasks();
        Ok(())
    }
    pub(super) fn BeginTaskTurn(&mut self, milliseconds: f64) {
        if let Some(scheduler) = &mut self.scheduler {
            scheduler.BeginTaskTurn(!(milliseconds > 0.0));
        }
    }
    pub(super) fn HasTaskBudget(&self) -> bool {
        self.scheduler
            .as_ref()
            .is_none_or(|scheduler| scheduler.HasTaskBudget())
    }
    pub(super) fn ConsumeTaskBudget(&mut self) {
        if let Some(scheduler) = &mut self.scheduler {
            scheduler.ConsumeTaskBudget();
        }
    }
    // cpp: browser/browser.cc:922-929
    pub(super) fn ApplyMutation(&mut self, mutation: PageMutation) -> io::Result<()> {
        if let PageMutation::DOMMutation(mutation) = mutation {
            let mut notification = self
                .bindings
                .borrow()
                .PrepareMutationNotification(&mutation);
            let resources = self.state.resources.borrow().clone();
            let client = RefCell::new(JavaScriptResourceClient {
                state: self.state.clone(),
                bindings: self.bindings.clone(),
                runtime: &mut *self.runtime,
                realm: &self.realm,
                engine: self.engine.clone(),
            });
            self.state.ApplyDOMMutation(
                &mutation,
                &mut || {
                    DOMJavaScriptBindings::DeliverMutationNotification(
                        &self.bindings,
                        notification.take(),
                        &mut |callback, args| {
                            let mut client = client.borrow_mut();
                            let realm = client.realm;
                            client.runtime.Call(
                                realm,
                                callback,
                                &javascript::javascript_runtime::HostValue::Object(
                                    javascript::javascript_runtime::HostObjectRef { id: 0 },
                                ),
                                args,
                            );
                        },
                    );
                },
                &mut |id, loaded| {
                    resources
                        .as_ref()
                        .expect("cached image has Page resources")
                        .DispatchImageEvent(id, loaded, &mut *client.borrow_mut());
                },
            );
        } else {
            self.state.ApplyMutation(mutation);
        }
        self.FlushTasks();
        Ok(())
    }
    // cpp: browser/browser.cc:894-903
    pub(super) fn Dispatch(
        &mut self,
        input: &interaction::input_event::InputEvent,
        fragments: &FragmentNode,
    ) -> io::Result<interaction::interaction_engine::InteractionResult> {
        let engine = self
            .scheduler
            .as_ref()
            .unwrap()
            .Interaction()
            .Engine()
            .clone();
        let runtime = RefCell::new(&mut *self.runtime);
        let listeners = Rc::new(
            |invocation: &EventListenerInvocation<'_>,
             _: &interaction::ownership::InteractionDocument,
             _: &interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchListeners(
                    &self.state,
                    &self.bindings,
                    invocation,
                    &mut **runtime.borrow_mut(),
                    &self.realm,
                )
            },
        );
        let result =
            engine
                .WithScopedListeners(listeners)
                .Dispatch(input, &self.state.document, fragments);
        drop(runtime);
        self.Checkpoint();
        Ok(result)
    }
}

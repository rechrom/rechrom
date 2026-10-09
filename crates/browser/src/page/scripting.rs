//! Optional JavaScript/Web API runtime used by OpenEngine's loading scheduler.
use super::*;
use javascript::script_engine::ScriptExecutor;
use open::{
    dynamic_scripts::DynamicScriptTasks, script_scheduler::ScriptScheduler, OpenScriptServices,
};

pub(super) struct PageScripts {
    state: Rc<PageState>,
    engine: interaction::Interaction<'static>,
    script: Rc<RefCell<ScriptEngine>>,
    web_api: WebApiEngine,
    interaction: Option<PageInteraction>,
    script_client: Rc<RefCell<dyn ScriptLoadClient>>,
    flush_dynamic_scripts: Option<Rc<dyn Fn()>>,
}
impl PageScripts {
    pub(super) fn HasBlockingWheelListener(&self) -> bool {
        self.web_api
            .DOMBindings()
            .borrow()
            .HasBlockingListener("wheel")
    }
    pub(super) fn BlockingWheelListenerTargets(&self) -> Vec<u64> {
        self.web_api
            .DOMBindings()
            .borrow()
            .BlockingListenerTargets("wheel")
    }
    // cpp: browser/browser.cc:610-737
    pub(super) fn new(
        state: Rc<PageState>,
        environment: ScriptEnvironment,
        script_client: Rc<RefCell<dyn ScriptLoadClient>>,
    ) -> Self {
        let ScriptEnvironment {
            runtime,
            xhr,
            user_agent,
        } = environment;
        let client = state.client.clone();
        let effect_state = state.clone();
        let effect_client = client.clone();
        let emit_web_api_effect: Rc<dyn Fn(WebApiEffect)> = Rc::new(move |effect| match effect {
            WebApiEffect::DocumentMutation(mutation) => {
                effect_state.ApplyMutation(PageMutation::DOMMutation(mutation));
            }
            WebApiEffect::Navigate(navigation) => {
                effect_client
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
            }
            WebApiEffect::AnimationTick { monotonic_time_ms } => {
                effect_state.ApplyAnimationMutation(animation::AnimationMutation::Tick(
                    animation::AnimationTick {
                        // A delayed BeginFrame may predate an ordinary
                        // task's animation sample. Only empty-sample
                        // bookkeeping clamps; rAF timestamps and actual
                        // style samples remain exact.
                        monotonic_time: (monotonic_time_ms / 1000.0)
                            .max(effect_state.animation_engine.borrow().LastTime()),
                        ..Default::default()
                    },
                ));
            }
        });
        let web_api = WebApiEngine::new(
            state.document.Handle(),
            state.CreateWebApiDOMHost(),
            xhr,
            user_agent,
            emit_web_api_effect,
        );
        let bindings = web_api.DOMBindings().clone();
        state.ConnectDOMBindings(&bindings);
        let mutate = state.clone();
        let interaction = PageInteraction::InstallWithEmitter(
            state.document.Handle(),
            &bindings,
            state.interaction.clone(),
            Some(Rc::new(move |output| mutate.ApplyInteractionOutput(output))),
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
        {
            let c = state.constraints.borrow();
            web_api.ApplyMutation(WebApiMutation::SetViewport {
                width: c.available_size.width,
                height: c.available_size.height,
                scale: c.device_pixel_ratio,
            });
        }
        let mut script = ScriptEngine::new(runtime, web_api.WindowBindings().clone());
        web_api.BindRuntime(script.Realm());
        let bootstrap_sources = WebApiEngine::BootstrapSources();
        let started = std::env::var_os("BROWSER_PROFILE_INPUT")
            .is_some()
            .then(std::time::Instant::now);
        let bootstrap_bytes = bootstrap_sources
            .iter()
            .map(|source| source.source.len())
            .sum::<usize>();
        let bootstrap_errors = script.EvaluateBootstrap(&bootstrap_sources);
        if let Some(start) = started {
            eprintln!(
                "javascript-realm-profile phase=bootstrap bytes={} ms={:.3}",
                bootstrap_bytes,
                start.elapsed().as_secs_f64() * 1000.
            );
        }
        for error in bootstrap_errors {
            client.borrow_mut().DidReportScriptError(&error);
        }
        Self {
            state,
            engine,
            script: Rc::new(RefCell::new(script)),
            web_api,
            interaction: Some(interaction),
            script_client,
            flush_dynamic_scripts: None,
        }
    }
    pub(super) fn Engine(&self) -> interaction::Interaction<'static> {
        self.engine.clone()
    }
    pub(super) fn Executor(&self) -> javascript::script_engine::ScriptExecutorHandle {
        self.script.clone()
    }
    pub(super) fn SetURL(&self, url: String) {
        self.web_api.ApplyMutation(WebApiMutation::SetURL(url));
    }
    pub(super) fn SetPreferredColorScheme(&self, preference: PreferredColorScheme) {
        self.web_api
            .ApplyMutation(WebApiMutation::SetPreferredColorScheme(preference));
    }
    pub(super) fn ResizeViewport(&self, width: f64, height: f64, scale: f64) {
        self.web_api.ApplyMutation(WebApiMutation::SetViewport {
            width,
            height,
            scale,
        });
    }
    pub(super) fn FinishLoad(
        &mut self,
        mut scheduler: Option<&mut ScriptScheduler>,
        executor: &dyn ScriptExecutor,
    ) {
        self.web_api
            .ApplyMutation(WebApiMutation::SetDocumentReadyState("complete".into()));
        for kind in [EventType::kReadyStateChange, EventType::kLoad] {
            if let Some(scheduler) = scheduler.as_deref_mut() {
                executor.WithRuntimeAndRealm(&mut |runtime, realm| {
                    scheduler.DispatchLifecycleEvent(
                        kind,
                        runtime,
                        realm,
                        &mut *self.script_client.borrow_mut(),
                    );
                });
            }
        }
        self.FlushTasks();
    }
    pub(super) fn DidPaint(&mut self) {
        if let Some(interaction) = &self.interaction {
            interaction.DidPaint();
        }
        // Chromium recomputes intersection observations from the committed
        // post-layout geometry, not only when a scroll event is dispatched.
        // The JS helper only queues observer delivery, so callbacks remain a
        // later task and cannot re-enter the lifecycle that just painted.
        let result = self.script.borrow_mut().EvaluateRaw(
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
            let bindings = self.web_api.DOMBindings().clone();
            let realm = self.script.borrow().Realm().clone();
            let mut script = self.script.borrow_mut();
            let runtime_cell = RefCell::new(script.RuntimeMut());
            let listeners = Rc::new(
                |invocation: &EventListenerInvocation<'_>,
                 _: &interaction::ownership::InteractionDocument,
                 _: &interaction::ownership::InteractionDOMMutationEmitter| {
                    DispatchListeners(
                        &self.state,
                        &bindings,
                        invocation,
                        &mut **runtime_cell.borrow_mut(),
                        &realm,
                    )
                },
            );
            self.engine.WithScopedListeners(listeners).DispatchDOMEvent(
                &mut event,
                target,
                &self.state.document.Handle(),
            );
        }
        self.Checkpoint();
    }
    pub(super) fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.state.QueueImageEvent(id, loaded);
    }
    pub(super) fn OpenServices(&self) -> OpenScriptServices {
        OpenScriptServices {
            bindings: self.web_api.DOMBindings().clone(),
            pending_host_errors: self
                .interaction
                .as_ref()
                .map(PageInteraction::ExceptionSource),
        }
    }

    pub(super) fn InstallDynamicScriptHooks(&mut self, dynamic: &Rc<DynamicScriptTasks>) {
        let weak_prepare = Rc::downgrade(&dynamic);
        self.state
            .connected_resources
            .SetScriptPreparer(Some(Rc::new(move |tree, node| {
                if let Some(dynamic) = weak_prepare.upgrade() {
                    dynamic.PrepareConnectedScript(tree, node, true)?;
                }
                Ok(())
            })));
        let flush_dynamic_scripts: Rc<dyn Fn()> = {
            let weak_window = Rc::downgrade(self.web_api.WindowBindings());
            let weak_dynamic = Rc::downgrade(&dynamic);
            let client = self.script_client.clone();
            Rc::new(move || {
                if let (Some(window), Some(dynamic)) =
                    (weak_window.upgrade(), weak_dynamic.upgrade())
                {
                    dynamic.EnqueuePreparedTasks(&window, client.clone());
                }
            })
        };
        self.flush_dynamic_scripts = Some(flush_dynamic_scripts.clone());
        let weak_window = Rc::downgrade(self.web_api.WindowBindings());
        let weak_state = Rc::downgrade(&self.state);
        let weak_bindings = Rc::downgrade(self.web_api.DOMBindings());
        let engine = self.engine.clone();
        self.web_api.SetTaskRegistrationFlusher(Rc::new(move || {
            flush_dynamic_scripts();
            if let Some(window) = weak_window.upgrade() {
                if let (Some(state), Some(bindings)) =
                    (weak_state.upgrade(), weak_bindings.upgrade())
                {
                    EnqueueImageEventTasks(&state, &window, &bindings, &engine);
                }
            }
        }));
    }
    #[cfg(test)]
    pub(super) fn DrainImageEventTasks(&mut self) {
        while !self.state.pending_image_events.borrow().is_empty() {
            self.FlushTasks();
            let client = self.state.client.clone();
            let mut script = self.script.borrow_mut();
            let (runtime, realm) = script.RuntimeAndRealm();
            self.web_api.RunTaskTurn(runtime, realm, 0.0, &mut |error| {
                client.borrow_mut().DidReportScriptError(error)
            });
        }
    }
    pub(super) fn FlushTasks(&self) {
        EnqueueImageEventTasks(
            &self.state,
            self.web_api.WindowBindings(),
            self.web_api.DOMBindings(),
            &self.engine,
        );
        if let Some(flush) = &self.flush_dynamic_scripts {
            flush();
        }
    }
    fn Checkpoint(&mut self) {
        let errors = self.script.borrow_mut().Checkpoint();
        self.ReportCheckpointEffects(errors);
    }
    fn ReportCheckpointEffects(&mut self, errors: Vec<JavaScriptException>) {
        for error in errors {
            self.state.client.borrow_mut().DidReportScriptError(&error);
        }
        if let Some(interaction) = &self.interaction {
            for error in interaction.TakePendingExceptions() {
                self.script_client.borrow_mut().DidReportScriptError(&error);
            }
        }
        self.FlushTasks();
    }
    // cpp: browser/browser.cc:905-920
    pub(super) fn Evaluate(
        &mut self,
        source: &str,
        source_name: &str,
    ) -> io::Result<JavaScriptResult> {
        let execution = self.script.borrow_mut().Evaluate(source, source_name);
        if let Some(error) = &execution.result.exception {
            self.state.client.borrow_mut().DidReportScriptError(error);
        }
        self.ReportCheckpointEffects(execution.pending_exceptions);
        Ok(execution.result)
    }
    pub(super) fn SetBeginFrameSource(
        &self,
        source: Option<std::sync::Arc<dyn foundation::begin_frame::BeginFrameSource>>,
    ) {
        self.web_api
            .ApplyMutation(WebApiMutation::SetBeginFrameSource(source));
    }
    pub(super) fn HasPendingAnimationFrames(&self) -> bool {
        self.web_api.HasPendingAnimationFrames()
    }
    pub(super) fn NextTaskDeadline(&self, now: std::time::Instant) -> Option<std::time::Instant> {
        self.web_api.NextTaskDeadline(now)
    }
    pub(super) fn RunAnimationFrameCallbacks(
        &mut self,
        frame_time: std::time::Instant,
    ) -> io::Result<()> {
        let client = self.state.client.clone();
        let mut script = self.script.borrow_mut();
        let (runtime, realm) = script.RuntimeAndRealm();
        self.web_api
            .RunAnimationFrameCallbacks(runtime, realm, frame_time, &mut |error| {
                client.borrow_mut().DidReportScriptError(error)
            });
        if let Some(interaction) = &self.interaction {
            for error in interaction.TakePendingExceptions() {
                self.script_client.borrow_mut().DidReportScriptError(&error);
            }
        }
        self.FlushTasks();
        Ok(())
    }
    pub(super) fn PumpModuleTasks(
        &mut self,
        scheduler: &mut ScriptScheduler,
        executor: &dyn ScriptExecutor,
        limit: usize,
    ) -> io::Result<usize> {
        self.FlushTasks();
        let mut progressed = 0;
        executor.WithRuntimeAndRealm(&mut |runtime, realm| {
            progressed = scheduler.PumpModules(runtime, realm, limit);
        });
        Ok(progressed)
    }

    pub(super) fn RunHostTaskTurn(&mut self, milliseconds: f64) -> io::Result<()> {
        let client = self.state.client.clone();
        let mut script = self.script.borrow_mut();
        let (runtime, realm) = script.RuntimeAndRealm();
        self.web_api
            .RunTaskTurn(runtime, realm, milliseconds, &mut |error| {
                client.borrow_mut().DidReportScriptError(error)
            });
        if let Some(interaction) = &self.interaction {
            for error in interaction.TakePendingExceptions() {
                self.script_client.borrow_mut().DidReportScriptError(&error);
            }
        }
        self.FlushTasks();
        Ok(())
    }
    // cpp: browser/browser.cc:922-929
    pub(super) fn ApplyMutation(&mut self, mutation: PageMutation) -> io::Result<()> {
        if let PageMutation::DOMMutation(mutation) = mutation {
            let bindings = self.web_api.DOMBindings().clone();
            let mut notification = bindings.borrow().PrepareMutationNotification(&mutation);
            let resources = self.state.resources.borrow().clone();
            let realm = self.script.borrow().Realm().clone();
            let mut script = self.script.borrow_mut();
            let client = RefCell::new(JavaScriptResourceClient {
                state: self.state.clone(),
                bindings: bindings.clone(),
                runtime: script.RuntimeMut(),
                realm: &realm,
                engine: self.engine.clone(),
            });
            self.state.ApplyDOMMutation(
                &mutation,
                &mut || {
                    DOMJavaScriptBindings::DeliverMutationNotification(
                        &bindings,
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
        let engine = self.engine.clone();
        let bindings = self.web_api.DOMBindings().clone();
        let realm = self.script.borrow().Realm().clone();
        let mut script = self.script.borrow_mut();
        let runtime = RefCell::new(script.RuntimeMut());
        let listeners = Rc::new(
            |invocation: &EventListenerInvocation<'_>,
             _: &interaction::ownership::InteractionDocument,
             _: &interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchListeners(
                    &self.state,
                    &bindings,
                    invocation,
                    &mut **runtime.borrow_mut(),
                    &realm,
                )
            },
        );
        let result = engine.WithScopedListeners(listeners).Dispatch(
            input,
            &self.state.document.Handle(),
            fragments,
        );
        drop(runtime);
        drop(script);
        self.Checkpoint();
        Ok(result)
    }
}

//! Optional JavaScript/Web API runtime used by OpenEngine's loading scheduler.
use super::*;
use interaction::event::{EventListenerInvocation, EventType, MakeSyntheticEvent};
use javascript::script_engine::ScriptExecutor;
use open::{
    dynamic_scripts::DynamicScriptTasks, script_scheduler::ScriptScheduler, OpenScriptServices,
};

#[path = "script_owners.rs"]
mod owners;
pub(super) use owners::{
    InstalledPageWindow, PageWindowPublishError, PreparedPageWindow, RetiredPageWindow,
};

/// Script-side values needed when installing a document's interaction facade.
pub(super) struct PageScriptInteractionEnvironment {
    pub executor: javascript::script_engine::ScriptExecutorHandle,
    pub bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    pub script: Rc<RefCell<ScriptEngine>>,
    pub window: Rc<RefCell<WindowJavaScriptBindings>>,
}

pub(super) struct PageScripts {
    script: Rc<RefCell<ScriptEngine>>,
    owners: Rc<owners::PageWindowScriptSnapshot>,
    window_factory: owners::PageWindowScriptFactory,
    script_client: Rc<RefCell<dyn ScriptLoadClient>>,
    flush_dynamic_scripts: Option<Rc<dyn Fn()>>,
}
impl PageScripts {
    pub(super) fn SetScriptLoadClient(&mut self, client: Rc<RefCell<dyn ScriptLoadClient>>) {
        self.script_client = client;
    }
    pub(super) fn TraverseToHistoryItem(
        &self,
        item: Arc<navigation_history::HistoryItemSnapshot>,
    ) -> bool {
        self.owners.navigation.TraverseToHistoryItem(item)
    }
    pub(super) fn TraverseToHistoryEntry(&self, key: &str) -> bool {
        self.owners.navigation.TraverseToHistoryEntry(key)
    }
    pub(super) fn HasPointerRawUpdateEventHandlers(&self) -> bool {
        self.owners
            .web_api
            .EventListenerInterest()
            .HasPointerRawUpdateEventHandlers()
    }

    pub(super) fn GarbageCollectionRequested(&self) -> bool {
        self.script.borrow().GarbageCollectionRequested()
    }

    pub(super) fn CollectGarbage(&mut self) {
        self.script.borrow_mut().CollectGarbage();
    }

    pub(super) fn MemoryDiagnostics(
        &self,
    ) -> (
        Option<javascript::javascript_runtime::JavaScriptMemoryDiagnostics>,
        webapi::dom_bindings::DOMHostMemoryDiagnostics,
    ) {
        (
            self.script.borrow().MemoryDiagnostics(),
            self.owners
                .web_api
                .DOMBindings()
                .borrow()
                .MemoryDiagnostics(),
        )
    }
    pub(super) fn HasBlockingWheelListener(&self) -> bool {
        self.owners
            .web_api
            .DOMBindings()
            .borrow()
            .HasBlockingListener("wheel")
    }
    pub(super) fn BlockingWheelListenerTargets(&self) -> Vec<u64> {
        self.owners
            .web_api
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
        let window_factory = owners::PageWindowScriptFactory::new(xhr, user_agent);
        let web_api = window_factory.Create(
            &state,
            &state.document,
            state.CreateWebApiDOMHost(),
            state.window_performance_timing.clone(),
        );
        state.ConnectDOMBindings(web_api.DOMBindings());
        {
            let c = state.constraints.borrow();
            web_api.ApplyMutation(WebApiMutation::SetViewport {
                width: c.available_size.width,
                height: c.available_size.height,
                // LocalFrame::DevicePixelRatio is the inspector override
                // (host DSF / layout zoom) multiplied by LayoutZoomFactor.
                // ConstraintSpace already stores their resolved product.
                scale: c.device_pixel_ratio,
            });
        }
        web_api.ApplyMutation(WebApiMutation::SetPreferredColorScheme(
            state.preferred_color_scheme.get(),
        ));
        web_api.ApplyMutation(WebApiMutation::SetPrefersReducedMotion(
            state.constraints.borrow().prefers_reduced_motion,
        ));
        web_api.ApplyMutation(WebApiMutation::SetForcedColors(state.forced_colors.get()));
        let mut script = ScriptEngine::new(runtime, web_api.WindowBindings().clone());
        let realm = script.Realm().clone();
        web_api
            .BindInitialHostObjectNamespace(script.RuntimeMut(), &realm)
            .unwrap_or_else(|error| std::panic::panic_any(error));
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
        let script = Rc::new(RefCell::new(script));
        let owners = owners::PageWindowScriptSnapshot::Create(
            &state,
            state.document.clone(),
            web_api,
            &script,
            true,
            None,
        );
        let factory = Rc::new(super::local_frame_navigation::LocalFrameScriptFactory::new(
            Rc::downgrade(&script),
            window_factory.xhr.clone(),
            window_factory.user_agent.clone(),
            owners.web_api.DOMBindings(),
            owners.web_api.WindowBindings(),
            &owners.interaction,
        ));
        assert!(state
            .local_frames
            .get()
            .unwrap()
            .script_factory
            .set(factory)
            .is_ok());
        Self {
            script,
            owners,
            window_factory,
            script_client,
            flush_dynamic_scripts: None,
        }
    }
    pub(super) fn InteractionEnvironment(&self) -> PageScriptInteractionEnvironment {
        self.owners.InteractionEnvironment()
    }
    pub(super) fn Engine(&self) -> ::interaction::Interaction<'static> {
        self.owners.interaction.Engine().clone()
    }
    pub(super) fn Executor(&self) -> javascript::script_engine::ScriptExecutorHandle {
        self.owners.executor.clone()
    }
    pub(super) fn InitializeWindowNavigation(
        &self,
        loader: Rc<document_loader::DocumentLoaderNavigationState>,
    ) {
        self.owners
            .executor
            .WithRuntimeAndRealm(&mut |runtime, realm| {
                let _ = runtime.WithHostRuntime(realm, &mut |host_runtime| {
                    self.owners
                        .navigation
                        .InitializeCommitted(loader.clone(), host_runtime);
                });
            });
    }
    pub(super) fn SetURL(&self, url: String) {
        self.owners
            .web_api
            .ApplyMutation(WebApiMutation::SetURL(url));
    }
    pub(super) fn SetPreferredColorScheme(&self, preference: PreferredColorScheme) {
        self.owners
            .web_api
            .ApplyMutation(WebApiMutation::SetPreferredColorScheme(preference));
    }
    pub(super) fn SetForcedColors(&self, forced_colors: ForcedColors) {
        self.owners
            .web_api
            .ApplyMutation(WebApiMutation::SetForcedColors(forced_colors));
    }
    pub(super) fn ResizeViewport(&self, width: f64, height: f64, effective_scale: f64) {
        self.owners
            .web_api
            .ApplyMutation(WebApiMutation::SetViewport {
                width,
                height,
                scale: effective_scale,
            });
    }
    pub(super) fn FinishLoad(
        &mut self,
        scheduler: Option<&mut ScriptScheduler>,
        executor: &dyn ScriptExecutor,
    ) {
        if self
            .owners
            .state
            .document
            .borrow()
            .GetDocument()
            .LoadEventFinished()
        {
            return;
        }
        self.owners
            .web_api
            .ApplyMutation(WebApiMutation::SetDocumentReadyState("complete".into()));
        let scheduler = scheduler.expect("script loading requires its lifecycle scheduler");
        executor.WithRuntimeAndRealm(&mut |runtime, realm| {
            scheduler.DispatchLifecycleEvent(
                EventType::kReadyStateChange,
                runtime,
                realm,
                &mut *self.script_client.borrow_mut(),
            );
        });
        self.Checkpoint();
        // Source: LocalDOMWindow::DispatchLoadEvent (2331-2343) dispatches to
        // Window with Document as legacy target, without a node event path.
        self.owners
            .state
            .document
            .borrow()
            .GetDocument()
            .SetLoadEventProgress(
                dom::document_parser_lifecycle::DocumentLoadEventProgress::InProgress,
            );
        executor.WithRuntimeAndRealm(&mut |runtime, realm| {
            scheduler.DispatchLifecycleEvent(
                EventType::kLoad,
                runtime,
                realm,
                &mut *self.script_client.borrow_mut(),
            );
        });
        self.Checkpoint();
        self.owners
            .state
            .document
            .borrow()
            .GetDocument()
            .SetLoadEventProgress(
                dom::document_parser_lifecycle::DocumentLoadEventProgress::Completed,
            );
        self.FlushTasks();
    }
    pub(super) fn DidPaint(&mut self) {
        self.owners.interaction.DidPaint();
        // Chromium gathers intersection and resize observations from committed
        // post-layout geometry.  The JS helpers only queue observer delivery,
        // so callbacks remain a later task and cannot re-enter this paint.
        let result = self.script.borrow_mut().EvaluateRaw(
            "globalThis.__browserUpdateIntersectionObservations?.();globalThis.__browserUpdateResizeObservations?.()",
            "browser:observer-update",
        );
        if let Some(error) = result.exception {
            self.owners
                .state
                .client
                .borrow_mut()
                .DidReportScriptError(&error);
        }
    }
    /// Dispatch the CSSOM View per-frame scroll event queue before rAF.
    /// Blink's Document::EnqueueScrollEventForNode stores these events in the
    /// ScriptedAnimationController; they must not wait behind ordinary network,
    /// timer, or image tasks.
    pub(super) fn DispatchPendingScrollEvents(&mut self) {
        let targets = std::mem::take(&mut *self.owners.state.scroll_event_targets.borrow_mut());
        if targets.is_empty() {
            return;
        }
        let mut trace = browser_tracing::span("lifecycle", "Page.DispatchPendingScrollEvents");
        trace.set("targets", targets.len() as f64);
        for target in targets {
            let (exists, is_document) = {
                let owner = self.owners.state.document.borrow();
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
            let engine = self.Engine();
            let realm = self.script.borrow().Realm().clone();
            let mut script = self.script.borrow_mut();
            let runtime = RefCell::new(script.RuntimeMut());
            let listeners = Rc::new(
                |invocation: &EventListenerInvocation<'_>,
                 source_document: &::interaction::ownership::InteractionDocument,
                 _: &::interaction::ownership::InteractionDOMMutationEmitter| {
                    self.owners.interaction.DispatchListeners(
                        source_document,
                        invocation,
                        &mut **runtime.borrow_mut(),
                        &realm,
                        &mut |error| {
                            self.owners
                                .state
                                .client
                                .borrow_mut()
                                .DidReportScriptError(error)
                        },
                    )
                },
            );
            engine.WithScopedListeners(listeners).DispatchDOMEvent(
                &mut event,
                target,
                &self.owners.document.Handle(),
            );
        }
        self.Checkpoint();
    }
    pub(super) fn DispatchImageEvent(&mut self, id: u64, loaded: bool) {
        self.owners.state.QueueImageEvent(id, loaded);
    }
    pub(super) fn DispatchStyleSheetEvent(&mut self, id: u64, signature: String, loaded: bool) {
        self.owners
            .state
            .QueueStyleSheetEvent(id, signature, loaded);
    }
    pub(super) fn OpenServices(&self) -> OpenScriptServices {
        self.owners.OpenServices()
    }

    pub(super) fn InstallDynamicScriptHooks(&mut self, dynamic: &Rc<DynamicScriptTasks>) {
        let weak_prepare = Rc::downgrade(&dynamic);
        self.owners
            .state
            .connected_resources
            .SetScriptPreparer(Some(Rc::new(move |tree, node| {
                if let Some(dynamic) = weak_prepare.upgrade() {
                    dynamic.PrepareConnectedScript(tree, node, true)?;
                }
                Ok(())
            })));
        let flush_dynamic_scripts: Rc<dyn Fn()> = {
            let weak_window = Rc::downgrade(self.owners.web_api.WindowBindings());
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
        let weak_window = Rc::downgrade(self.owners.web_api.WindowBindings());
        let weak_state = Rc::downgrade(&self.owners.state);
        let weak_bindings = Rc::downgrade(self.owners.web_api.DOMBindings());
        let engine = self.Engine();
        self.owners
            .web_api
            .SetTaskRegistrationFlusher(Rc::new(move || {
                flush_dynamic_scripts();
                if let Some(window) = weak_window.upgrade() {
                    if let (Some(state), Some(bindings)) =
                        (weak_state.upgrade(), weak_bindings.upgrade())
                    {
                        EnqueueImageEventTasks(&state, &window, &bindings, &engine);
                        EnqueueStyleSheetEventTasks(&state, &window, &bindings, &engine);
                    }
                }
            }));
    }
    #[cfg(test)]
    pub(super) fn DrainImageEventTasks(&mut self) {
        while !self.owners.state.pending_image_events.borrow().is_empty() {
            self.FlushTasks();
            let client = self.owners.state.client.clone();
            let mut script = self.script.borrow_mut();
            let (runtime, realm) = script.RuntimeAndRealm();
            self.owners
                .web_api
                .RunTaskTurn(runtime, realm, 0.0, &mut |error| {
                    client.borrow_mut().DidReportScriptError(error)
                });
        }
    }
    pub(super) fn FlushTasks(&self) {
        EnqueueSelectionchangeEventTasks(
            &self.owners.state,
            self.owners.web_api.WindowBindings(),
            self.owners.web_api.DOMBindings(),
            &self.Engine(),
        );
        EnqueueImageEventTasks(
            &self.owners.state,
            self.owners.web_api.WindowBindings(),
            self.owners.web_api.DOMBindings(),
            &self.Engine(),
        );
        EnqueueStyleSheetEventTasks(
            &self.owners.state,
            self.owners.web_api.WindowBindings(),
            self.owners.web_api.DOMBindings(),
            &self.Engine(),
        );
        if let Some(flush) = &self.flush_dynamic_scripts {
            flush();
        }
    }
    fn ScheduleCustomElementBackupReactions(&self) {
        self.owners
            .web_api
            .DOMBindings()
            .borrow_mut()
            .ScheduleCustomElementBackupReactions()
            .expect("current Document Agent accepts its backup reaction microtask");
    }
    pub(super) fn Checkpoint(&mut self) {
        self.ScheduleCustomElementBackupReactions();
        let errors = self.script.borrow_mut().Checkpoint();
        self.ReportCheckpointEffects(errors);
    }
    fn ReportCheckpointEffects(&mut self, errors: Vec<JavaScriptException>) {
        for error in errors {
            self.owners
                .state
                .client
                .borrow_mut()
                .DidReportScriptError(&error);
        }
        for error in self.owners.interaction.TakePendingExceptions() {
            self.script_client.borrow_mut().DidReportScriptError(&error);
        }
        self.FlushTasks();
    }
    // cpp: browser/browser.cc:905-920
    pub(super) fn Evaluate(
        &mut self,
        source: &str,
        source_name: &str,
    ) -> io::Result<JavaScriptResult> {
        self.EvaluateWithDirectExceptionReporting(source, source_name, true)
    }
    pub(super) fn EvaluateForInspector(
        &mut self,
        source: &str,
        source_name: &str,
    ) -> io::Result<JavaScriptResult> {
        self.EvaluateWithDirectExceptionReporting(source, source_name, false)
    }
    fn EvaluateWithDirectExceptionReporting(
        &mut self,
        source: &str,
        source_name: &str,
        report_direct_exception: bool,
    ) -> io::Result<JavaScriptResult> {
        self.ScheduleCustomElementBackupReactions();
        let execution = self.script.borrow_mut().Evaluate(source, source_name);
        if report_direct_exception {
            if let Some(error) = &execution.result.exception {
                self.owners
                    .state
                    .client
                    .borrow_mut()
                    .DidReportScriptError(error);
            }
        }
        self.ReportCheckpointEffects(execution.pending_exceptions);
        Ok(execution.result)
    }
    pub(super) fn SetBeginFrameSource(
        &self,
        source: Option<std::sync::Arc<dyn foundation::begin_frame::BeginFrameSource>>,
    ) {
        self.owners
            .web_api
            .ApplyMutation(WebApiMutation::SetBeginFrameSource(source));
    }
    pub(super) fn HasPendingAnimationFrames(&self) -> bool {
        self.owners.web_api.HasPendingAnimationFrames()
    }
    pub(super) fn NextTaskDeadline(&self, now: std::time::Instant) -> Option<std::time::Instant> {
        [
            self.owners.web_api.NextTaskDeadline(now),
            self.owners
                .state
                .local_frames
                .get()
                .unwrap()
                .ChildTaskDeadline(now),
        ]
        .into_iter()
        .flatten()
        .min()
    }
    pub(super) fn RunAnimationFrameCallbacks(
        &mut self,
        frame_time: std::time::Instant,
    ) -> io::Result<()> {
        let client = self.owners.state.client.clone();
        let mut script = self.script.borrow_mut();
        let (runtime, realm) = script.RuntimeAndRealm();
        self.owners
            .web_api
            .RunAnimationFrameCallbacks(runtime, realm, frame_time, &mut |error| {
                client.borrow_mut().DidReportScriptError(error)
            });
        for error in self.owners.interaction.TakePendingExceptions() {
            self.script_client.borrow_mut().DidReportScriptError(&error);
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
        if progressed != 0 {
            // A completed module-fetch/compile step is a JavaScript task. Its
            // microtask checkpoint resumes host-deferred import() promises;
            // postponing that checkpoint until an unrelated Window task can
            // leave a fully prepared graph permanently pending when no other
            // transport wake remains. This is the Page-owned equivalent of
            // Chromium's task-end MicrotasksScope checkpoint.
            self.Checkpoint();
        }
        Ok(progressed)
    }

    pub(super) fn RunHostTaskTurn(&mut self, milliseconds: f64) -> io::Result<()> {
        self.ScheduleCustomElementBackupReactions();
        let client = self.owners.state.client.clone();
        let mut script = self.script.borrow_mut();
        let (runtime, realm) = script.RuntimeAndRealm();
        self.owners
            .web_api
            .RunTaskTurn(runtime, realm, milliseconds, &mut |error| {
                client.borrow_mut().DidReportScriptError(error)
            });
        drop(script);
        self.Checkpoint();
        Ok(())
    }
    /// The kMiscPlatformAPI popup task borrows this Window's current runtime
    /// only for its dispatch, then performs the ordinary task-end checkpoint.
    pub(super) fn RunInteractionTask(
        &mut self,
        task: super::popup_tasks::PageInteractionTask,
    ) -> io::Result<()> {
        self.ScheduleCustomElementBackupReactions();
        let interaction = self.owners.interaction.clone();
        let realm = self.script.borrow().Realm().clone();
        let mut task = Some(task);
        let mut result = None;
        let mut script = self.script.borrow_mut();
        let invocation = script.RuntimeMut().WithHostRuntime(&realm, &mut |runtime| {
            result = Some(interaction.WithScriptRuntime(runtime, |scoped| {
                let result = match task.take().expect("one popup task invocation") {
                    super::popup_tasks::PageInteractionTask::LayoutShiftInputTimer => Ok(()),
                    super::popup_tasks::PageInteractionTask::EventHandler { task, .. } => {
                        scoped.RunEventHandlerTimer(&task).map_err(io::Error::other)
                    }
                    super::popup_tasks::PageInteractionTask::ChildScrollbar { .. } => {
                        unreachable!("child task routed by Page")
                    }
                    super::popup_tasks::PageInteractionTask::FormRestore(task)
                    | super::popup_tasks::PageInteractionTask::FormNavigation(task)
                    | super::popup_tasks::PageInteractionTask::AnchorNavigation(task) => {
                        if let Some(task) = task.borrow_mut().take() {
                            task();
                        }
                        Ok(())
                    }
                    super::popup_tasks::PageInteractionTask::SelectionAutoscroll(now) => scoped
                        .AnimateSelectionAutoscroll(&self.owners.document.Handle(), now)
                        .map_err(io::Error::other),
                    super::popup_tasks::PageInteractionTask::DirectoryFilesChosen {
                        target,
                        files,
                    } => {
                        scoped.DirectoryFilesChosen(&self.owners.document.Handle(), target, files);
                        Ok(())
                    }
                    super::popup_tasks::PageInteractionTask::SystemDragTarget {
                        event,
                        action,
                        result,
                    } => scoped
                        .HandleTargetDrag(&self.owners.document.Handle(), &event, &action)
                        .map(|operation| result.set(Some(operation)))
                        .map_err(io::Error::other),
                    super::popup_tasks::PageInteractionTask::SystemDragEnd { event, operation } => {
                        scoped.DragSourceEndedAt(&self.owners.document.Handle(), &event, operation);
                        Ok(())
                    }
                    super::popup_tasks::PageInteractionTask::Popup(task) => scoped
                        .RunPopupTask(&self.owners.document.Handle(), task)
                        .map_err(|error| io::Error::other(error.to_string())),
                    super::popup_tasks::PageInteractionTask::Gesture(task) => scoped
                        .RunGestureTimerTask(&self.owners.document.Handle(), task)
                        .map_err(io::Error::other),
                    super::popup_tasks::PageInteractionTask::Scrollbar(task) => {
                        scoped.RunScrollbarTimerTask(task)
                    }
                };
                interaction.CompleteFormSubmissions(scoped);
                result
            }));
        });
        drop(script);
        self.Checkpoint();
        invocation.map_err(|error| io::Error::other(error.message))?;
        result.expect("popup task runs in its Window invocation")
    }
    // cpp: browser/browser.cc:922-929
    pub(super) fn ApplyMutation(&mut self, mutation: PageMutation) -> io::Result<()> {
        if let PageMutation::DOMMutation(mutation) = mutation {
            let mut script = self.script.borrow_mut();
            let (runtime, realm) = script.RuntimeAndRealm();
            let mut result = Ok(());
            runtime
                .WithHostRuntime(realm, &mut |runtime| {
                    result = (|| {
                        if !self
                            .owners
                            .interaction
                            .ApplyPopupMutation(&mutation, runtime)
                            .map_err(|error| io::Error::other(error.to_string()))?
                        {
                            self.owners
                                .state
                                .ApplyDOMMutation(&mutation, &mut |id, loaded| {
                                    self.owners.state.QueueImageEvent(id, loaded);
                                });
                        }
                        self.owners
                            .interaction
                            .DrainPopupLifecycle(runtime)
                            .map_err(|error| io::Error::other(error.to_string()))
                    })();
                })
                .map_err(|error| io::Error::other(error.message))?;
            result?;
        } else {
            self.owners.state.ApplyMutation(mutation);
        }
        self.ScheduleCustomElementBackupReactions();
        self.FlushTasks();
        Ok(())
    }
    // cpp: browser/browser.cc:894-903
    pub(super) fn Dispatch(
        &mut self,
        input: &::interaction::input_event::InputEvent,
        fragments: PaintFragmentCursor<'_>,
        packet: Option<interaction::frame_aligned_input_queue::BorrowedInputPacket<'_>>,
    ) -> io::Result<::interaction::InteractionResult> {
        let interaction = self.owners.interaction.clone();
        let realm = self.script.borrow().Realm().clone();
        let mut fragments = Some(
            interaction::adapter::interaction::InteractionFragmentSource::new(
                fragments,
                &paint_skia::path_hit_test::SkiaPathHitTestFactory,
            ),
        );
        let mut packet = packet;
        let mut result = None;
        let mut script = self.script.borrow_mut();
        let invocation = script.RuntimeMut().WithHostRuntime(&realm, &mut |runtime| {
            result = Some(interaction.WithScriptRuntime(runtime, |engine| {
                let fragments = fragments.take().expect("one input dispatch invocation");
                let result = if let Some(packet) = packet.take() {
                    engine.TryDispatchPacket(
                        input,
                        &self.owners.document.Handle(),
                        fragments,
                        packet,
                    )
                } else {
                    engine.TryDispatch(input, &self.owners.document.Handle(), fragments)
                };
                interaction.CompleteFormSubmissions(engine);
                result
            }));
        });
        drop(script);
        self.Checkpoint();
        invocation.map_err(|error| io::Error::other(error.message))?;
        result
            .expect("input dispatch runs in its Window invocation")
            .map_err(|error| io::Error::other(format!("native input: {error:?}")))
    }

    pub(super) fn RecomputeMouseHoverState(
        &mut self,
        fragments: PaintFragmentCursor<'_>,
    ) -> io::Result<()> {
        let interaction = self.owners.interaction.clone();
        let realm = self.script.borrow().Realm().clone();
        let mut fragments = Some(
            interaction::adapter::interaction::InteractionFragmentSource::new(
                fragments,
                &paint_skia::path_hit_test::SkiaPathHitTestFactory,
            ),
        );
        let mut result = None;
        let mut script = self.script.borrow_mut();
        let invocation = script.RuntimeMut().WithHostRuntime(&realm, &mut |runtime| {
            result = Some(interaction.WithScriptRuntime(runtime, |engine| {
                let result = engine.RecomputeMouseHoverState(
                    &self.owners.document.Handle(),
                    fragments.take().expect("one synthetic hover invocation"),
                );
                interaction.CompleteFormSubmissions(engine);
                result
            }));
        });
        drop(script);
        self.Checkpoint();
        invocation.map_err(|error| io::Error::other(error.message))?;
        result
            .expect("synthetic hover runs in its Window invocation")
            .map_err(|error| io::Error::other(format!("native synthetic hover: {error:?}")))
    }
}

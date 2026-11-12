#![allow(non_snake_case)]

use dom::{InteractionStateMutation, UserInteractionState, DOM};
use interaction::{
    event::{EventListenerInvocation, EventListenerResult},
    input_event::{FocusEvent, FocusEventType, InputEvent},
    Interaction, InteractionOutput,
};
use javascript::javascript_runtime::{
    JavaScriptException, JavaScriptHostRuntime, JavaScriptRealm, JavaScriptRuntime,
    WeakJavaScriptRealm,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use webapi::dom_bindings::DOMJavaScriptBindings;

pub(crate) struct MonotonicEventClock;
impl interaction::dispatch::core::dom::events::event::EventClock for MonotonicEventClock {
    fn Now(&self) -> interaction::input::base::time::time::TimeTicks {
        static ORIGIN: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let elapsed = ORIGIN.get_or_init(std::time::Instant::now).elapsed();
        interaction::input::base::time::time::TimeTicks::FromInternalValue(
            elapsed.as_micros().min((i64::MAX - 1) as u128) as i64 + 1,
        )
    }
}

/// Browser-owned interaction service on the parser's persistent DOM. Its
/// shared state is also the input to layout/measurement, and dirty flags are
/// available to a frame owner; this service does not own a cached frame.
/// Page-owned continuation entry, invoked by the binding before returning to
/// author script. It borrows this call's Interaction/listener scope only.
pub(crate) type FormSubmissionTransaction =
    Rc<dyn for<'scope> Fn(&Interaction<'scope>, &interaction::InteractionDocument)>;

pub(crate) type FormDataTransaction = Rc<
    dyn for<'scope> Fn(
        &Interaction<'scope>,
        &interaction::InteractionDocument,
        interaction::input_type::FormSubmissionToken,
    ) -> Result<
        Rc<interaction::dispatch::core::fileapi::form_data_object::FormDataObject>,
        interaction::dispatch::core::fileapi::form_data_object::FormDataConstructionError,
    >,
>;

pub struct PageInteraction {
    form_data_transaction: Rc<RefCell<Option<FormDataTransaction>>>,
    form_controller: Rc<interaction::adapter::page_form_controller::PageFormController>,
    form_submission_transaction: Rc<RefCell<Option<FormSubmissionTransaction>>>,
    engine: Interaction<'static>,
    listeners: Rc<PageListenerRouting>,
    document: Rc<RefCell<DOM>>,
    bindings: std::rc::Weak<RefCell<DOMJavaScriptBindings>>,
    state: Rc<RefCell<UserInteractionState>>,
    errors: Rc<RefCell<Vec<JavaScriptException>>>,
    error_reporter: Rc<RefCell<Option<Rc<dyn Fn(&JavaScriptException)>>>>,
    dirty: Rc<Cell<bool>>,
    styles_resolved: Rc<Cell<bool>>,
}

// The registry and realm identities are non-owning. Every listener invocation
// resolves its original Document afresh and lends the current host runtime;
// neither a callback nor an event path survives this dispatch scope.
struct PageListenerRouting {
    document: std::rc::Weak<RefCell<DOM>>,
    bindings: std::rc::Weak<RefCell<DOMJavaScriptBindings>>,
    realm: RefCell<WeakJavaScriptRealm>,
    frames: RefCell<std::rc::Weak<crate::page::local_frame::LocalFrameDomain>>,
}
impl PageListenerRouting {
    /// Borrow the invocation once for the complete synchronous dispatch,
    /// including default actions after author listeners have returned. The
    /// native invoker already selects each listener's original Window/realm.
    fn WithScriptRuntime<T>(
        &self,
        engine: &Interaction<'_>,
        runtime: &mut dyn JavaScriptHostRuntime,
        run: impl FnOnce(&Interaction<'_>) -> T,
    ) -> T {
        let Some(bindings) = self.bindings.upgrade() else {
            return run(engine);
        };
        let mut run = Some(run);
        let mut result = None;
        DOMJavaScriptBindings::RunNavigationInvocation(&bindings, runtime, &mut |script| {
            let listeners = Rc::new(
                |event: &EventListenerInvocation<'_>,
                 document: &interaction::InteractionDocument,
                 _: &interaction::InteractionDOMMutationEmitter| {
                    let Some(bindings) = self.Bindings(document) else {
                        return EventListenerResult::default();
                    };
                    DOMJavaScriptBindings::DispatchNavigationDOMListeners(&bindings, event, script)
                },
            );
            let scoped = engine.WithScopedListeners(listeners);
            let scoped = scoped.WithScriptInvocation(script);
            result = Some(run.take().expect("one native invocation scope")(&scoped));
        });
        result.expect("navigation invocation runs synchronously")
    }

    fn ForDocument(
        &self,
        document: &interaction::InteractionDocument,
    ) -> Option<Rc<PageInteraction>> {
        self.frames
            .borrow()
            .upgrade()?
            .InteractionForDocument(document)
    }
    fn Owns(&self, document: &interaction::InteractionDocument) -> bool {
        self.document
            .upgrade()
            .is_some_and(|owner| Rc::ptr_eq(&owner, document))
    }
    fn Bindings(
        &self,
        document: &interaction::InteractionDocument,
    ) -> Option<Rc<RefCell<DOMJavaScriptBindings>>> {
        if self.Owns(document) {
            self.bindings.upgrade()
        } else {
            self.ForDocument(document)?.listeners.bindings.upgrade()
        }
    }
    fn Dispatch(
        &self,
        document: &interaction::InteractionDocument,
        invocation: &EventListenerInvocation<'_>,
        runtime: &mut dyn JavaScriptHostRuntime,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> EventListenerResult {
        if !self.Owns(document) {
            return self
                .ForDocument(document)
                .map_or_else(EventListenerResult::default, |owner| {
                    owner
                        .listeners
                        .Dispatch(document, invocation, runtime, report_error)
                });
        }
        let Some(bindings) = self.bindings.upgrade() else {
            return EventListenerResult::default();
        };
        let realm = self.realm.borrow().Upgrade();
        let Some(realm) = realm else {
            if self.frames.borrow().upgrade().is_some() {
                return EventListenerResult::default();
            }
            // Standalone parser services have no Frame/Window registry. Their
            // only Document already runs in this call's originating realm.
            return DOMJavaScriptBindings::DispatchEventListenersScoped(
                &bindings,
                invocation,
                runtime,
                report_error,
            );
        };
        let mut result = EventListenerResult::default();
        if let Err(error) = runtime.WithRealm(&realm, &mut |runtime| {
            result = DOMJavaScriptBindings::DispatchEventListenersScoped(
                &bindings,
                invocation,
                runtime,
                report_error,
            );
        }) {
            report_error(&error);
        }
        result
    }
}

impl PageInteraction {
    pub(crate) fn WindowRealm(&self) -> Option<JavaScriptRealm> {
        self.listeners.realm.borrow().Upgrade()
    }
    pub(crate) fn WindowView(
        &self,
    ) -> Option<interaction::dispatch::core::events::ui_event::AbstractViewRef> {
        self.engine
            .ResolveDOMEventTarget(&self.document, 0)?
            .AsAbstractView()
    }
    pub fn SetFormStateTaskRunner(
        &self,
        runner: interaction::adapter::page_form_controller::FormStateTaskRunner,
    ) {
        self.form_controller.SetTaskRunner(runner);
    }
    pub fn SetFormStateFileReader(&self, reader: std::sync::Arc<dyn fileapi::NativeFileReader>) {
        self.form_controller.SetFileReader(reader);
    }
    // cpp: browser/browser.cc:629-631,671-676,702-734
    pub fn Install(
        document: Rc<RefCell<DOM>>,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        state: Rc<RefCell<UserInteractionState>>,
    ) -> Self {
        Self::InstallWithEmitter(
            document,
            bindings,
            state,
            None,
            Rc::new(Default::default()),
            Rc::new(MonotonicEventClock),
        )
    }

    pub(crate) fn InstallWithEmitter(
        document: Rc<RefCell<DOM>>,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        state: Rc<RefCell<UserInteractionState>>,
        emitter: Option<interaction::InteractionOutputEmitter>,
        selections: Rc<layoutng_assembly::editing_state::SelectionState>,
        clock: Rc<dyn interaction::dispatch::core::dom::events::event::EventClock>,
    ) -> Self {
        Self::InstallWithRetainedOwner(document, bindings, state, emitter, selections, clock, None)
    }

    pub(crate) fn InstallWithFrameOwner(
        document: Rc<RefCell<DOM>>,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        state: Rc<RefCell<UserInteractionState>>,
        selections: Rc<layoutng_assembly::editing_state::SelectionState>,
        clock: Rc<dyn interaction::dispatch::core::dom::events::event::EventClock>,
        engine: Interaction<'static>,
        controller: Rc<interaction::adapter::page_form_controller::PageFormController>,
    ) -> Self {
        Self::InstallWithRetainedOwner(
            document,
            bindings,
            state,
            None,
            selections,
            clock,
            Some((engine, controller)),
        )
    }

    fn InstallWithRetainedOwner(
        document: Rc<RefCell<DOM>>,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        state: Rc<RefCell<UserInteractionState>>,
        emitter: Option<interaction::InteractionOutputEmitter>,
        selections: Rc<layoutng_assembly::editing_state::SelectionState>,
        clock: Rc<dyn interaction::dispatch::core::dom::events::event::EventClock>,
        retained: Option<(
            Interaction<'static>,
            Rc<interaction::adapter::page_form_controller::PageFormController>,
        )>,
    ) -> Self {
        let weak = Rc::downgrade(bindings);
        let listeners = Rc::new(PageListenerRouting {
            document: Rc::downgrade(&document),
            bindings: weak.clone(),
            realm: Default::default(),
            frames: Default::default(),
        });
        let mutation_document = document.clone();
        let mutation_bindings = weak.clone();
        let mutation_state = state.clone();
        let dirty = Rc::new(Cell::new(true));
        let styles_resolved = Rc::new(Cell::new(false));
        let mutation_dirty = dirty.clone();
        let mutation_styles = styles_resolved.clone();
        let (retained_engine, retained_controller) = retained
            .map_or((None, None), |(engine, controller)| {
                (Some(engine), Some(controller))
            });
        let engine = retained_engine.unwrap_or_else(|| {
            Interaction::WithSelectionState(
                Rc::new(move |mutation| {
                    if let Some(emitter) = &emitter {
                        emitter(mutation);
                        mutation_styles.set(false);
                        mutation_dirty.set(true);
                        return;
                    }
                    match mutation {
                        InteractionOutput::DocumentMutation(mutation) => {
                            // cpp: browser/browser.cc:950-981
                            // Connected-subtree discovery stays with the resource scheduler.
                            let mut owner = mutation_document.borrow_mut();
                            crate::dom_mutation::ApplyDOMTreeMutation(&mut owner, &mutation);
                            mutation_styles.set(false);
                            mutation_dirty.set(true);
                        }
                        InteractionOutput::StateMutation(mutation) => {
                            ApplyInteractionStateMutation(
                                &mutation_document,
                                &mutation_bindings,
                                &mutation_state,
                                &mutation_dirty,
                                &mutation_styles,
                                mutation,
                            );
                        }
                        InteractionOutput::Effect(_) => {
                            // Standalone service users have no navigation host.
                            // A composed Page always supplies an output emitter.
                        }
                    }
                }),
                None,
                selections,
                clock,
            )
        });
        engine
            .FormControlState()
            .SetValidationPatternEngine(webapi::form_validation::CreateValidationPatternEngine());
        bindings
            .borrow_mut()
            .BindFormValidationState(engine.FormControlState());
        // EventTarget listener interest is a retained read-only owner, separate
        // from the call-scoped JavaScript listener dispatcher. Boundary event
        // phases re-query it after synchronous listener mutation.
        engine.SetEventListenerInterest(Some(Rc::new(bindings.borrow().EventListenerInterest())));
        engine.BindDocumentInputLifecycle(&document);
        let selection_read_engine = engine.clone();
        let selection_read_document = document.clone();
        let selection_write_engine = engine.clone();
        let selection_text_engine = engine.clone();
        bindings
            .borrow_mut()
            .SetDOMSelectionHost(Some(webapi::dom_bindings::DOMSelectionHost {
                read: Rc::new(move || {
                    selection_read_engine.DOMSelectionSnapshot(&selection_read_document)
                }),
                write: Rc::new(move |range| selection_write_engine.SetDOMSelectionFromRange(range)),
                text: Rc::new(move || selection_text_engine.DOMSelectionText()),
            }));
        let text_selection_read_engine = engine.clone();
        let text_selection_read_document = document.clone();
        let text_selection_write_engine = engine.clone();
        let text_selection_write_document = document.clone();
        bindings.borrow_mut().SetTextControlSelectionHost(Some(
            webapi::dom_bindings::TextControlSelectionHost {
                read: Rc::new(move |id| {
                    text_selection_read_engine
                        .TextControlSelectionSnapshot(&text_selection_read_document, id)
                }),
                write: Rc::new(move |id, start, end, backward| {
                    text_selection_write_engine.SetTextControlSelectionRange(
                        &text_selection_write_document,
                        id,
                        start,
                        end,
                        backward,
                    )
                }),
            },
        ));
        let target_engine = engine.clone();
        let target_document = document.clone();
        bindings
            .borrow_mut()
            .SetDOMEventTargetResolver(Rc::new(move |id| {
                target_engine.ResolveDOMEventTarget(&target_document, id)
            }));
        let form_controller = retained_controller.unwrap_or_else(|| {
            interaction::adapter::page_form_controller::PageFormController::Install(
                document.clone(),
                engine.clone(),
            )
        });
        let capture_engine = engine.clone();
        let close_create_engine = engine.clone();
        let close_create_document = document.clone();
        let close_invoke_engine = engine.clone();
        let close_invoke_document = document.clone();
        let close_destroy_engine = engine.clone();
        let close_destroy_document = document.clone();
        let close_listeners = listeners.clone();
        let errors = Rc::new(RefCell::new(Vec::new()));
        let error_reporter: Rc<RefCell<Option<Rc<dyn Fn(&JavaScriptException)>>>> =
            Rc::new(RefCell::new(None));
        let close_errors = errors.clone();
        let close_reporter = error_reporter.clone();
        bindings.borrow().SetCloseWatcherHost(webapi::dom_bindings::CloseWatcherHost {
            create: Rc::new(move |host_id, target, aborted| {
                close_create_engine.CreateScriptCloseWatcher(&close_create_document, host_id, target, aborted).map_err(|error| error.to_string())
            }),
            invoke: Rc::new(move |watcher, request, runtime| {
                let runtime = RefCell::new(runtime);
                let shared_runtime = &runtime;
                let listeners = close_listeners.clone();
                let errors = close_errors.clone();
                let reporter = close_reporter.clone();
                let scoped_listeners = Rc::new(move |invocation: &EventListenerInvocation<'_>, source_document: &interaction::InteractionDocument, _: &interaction::InteractionDOMMutationEmitter| {
                    listeners.Dispatch(source_document, invocation, &mut **shared_runtime.borrow_mut(), &mut |error| {
                        if let Some(report) = reporter.borrow().clone() { report(error); }
                        else { errors.borrow_mut().push(error.clone()); }
                    })
                });
                let result = close_invoke_engine.WithScopedListeners(scoped_listeners).InvokeScriptCloseWatcher(&close_invoke_document, watcher, request).map_err(|error| error.to_string());
                result
            }),
            destroy: Rc::new(move |watcher| close_destroy_engine.DestroyScriptCloseWatcher(&close_destroy_document, watcher)),
        });
        let capture_document = document.clone();
        bindings.borrow_mut().SetPointerCaptureHandler(Some(Rc::new(
            move |operation, pointer_id, target| match operation {
                webapi::dom_bindings::PointerCaptureOperation::Set => {
                    capture_engine.SetPointerCapture(&capture_document, pointer_id, target)?;
                    Ok(false)
                }
                webapi::dom_bindings::PointerCaptureOperation::Release => {
                    capture_engine.ReleasePointerCapture(&capture_document, pointer_id, target)?;
                    Ok(false)
                }
                webapi::dom_bindings::PointerCaptureOperation::Has => {
                    Ok(capture_engine.HasPointerCapture(&capture_document, pointer_id, target))
                }
            },
        )));
        let command_engine = engine.clone();
        let command_document = document.clone();
        let command_listeners = listeners.clone();
        let command_errors = errors.clone();
        let command_reporter = error_reporter.clone();
        bindings.borrow_mut().SetDocumentEditorCommandHandler(Some(Rc::new(move |operation, name, value, runtime| {
            let scoped_runtime = RefCell::new(runtime);
            let listeners = command_listeners.clone();
            let reporter = command_reporter.clone();
            let errors = command_errors.clone();
            let scoped_listeners = Rc::new(move |invocation: &EventListenerInvocation<'_>, source_document: &interaction::InteractionDocument, _: &interaction::InteractionDOMMutationEmitter| {
                listeners.Dispatch(source_document, invocation, &mut **scoped_runtime.borrow_mut(), &mut |error| {
                    if let Some(report) = reporter.borrow().clone() { report(error); }
                    else { errors.borrow_mut().push(error.clone()); }
                })
            });
            let result = command_engine.WithScopedListeners(scoped_listeners).DocumentEditorCommand(&command_document, operation, name, value);
            result
        })));
        let execute_engine = engine.clone();
        let execute_document = document.clone();
        let execute_listeners = listeners.clone();
        let execute_errors = errors.clone();
        let execute_reporter = error_reporter.clone();
        bindings
            .borrow_mut()
            .SetDocumentEditorCommandExecutor(Some(Rc::new(move |name, check_value, runtime| {
                use javascript::javascript_runtime::{
                    HostResult, HostValue, JavaScriptExceptionKind,
                };
                let scoped_runtime = RefCell::new(runtime);
                let shared_runtime = &scoped_runtime;
                let listeners = execute_listeners.clone();
                let errors = execute_errors.clone();
                let reporter = execute_reporter.clone();
                let scoped_listeners = Rc::new(
                    move |invocation: &EventListenerInvocation<'_>,
                          source_document: &interaction::InteractionDocument,
                          _: &interaction::InteractionDOMMutationEmitter| {
                        listeners.Dispatch(
                            source_document,
                            invocation,
                            &mut **shared_runtime.borrow_mut(),
                            &mut |error| {
                                if let Some(report) = reporter.borrow().clone() {
                                    report(error);
                                } else {
                                    errors.borrow_mut().push(error.clone());
                                }
                            },
                        )
                    },
                );
                let scoped = execute_engine.WithScopedListeners(scoped_listeners);
                let prepared = match scoped.PrepareDocumentEditorCommand(&execute_document, name) {
                    Ok(Some(prepared)) => prepared,
                    Ok(None) => {
                        return HostResult {
                            value: HostValue::Boolean(false),
                            ..Default::default()
                        };
                    }
                    Err(error) => {
                        return HostResult::Failure(
                            JavaScriptExceptionKind::kTypeError,
                            &format!("Editing command service required: {error:?}"),
                        );
                    }
                };
                let value = match check_value(
                    prepared.IsValueInterpretedAsHtml(),
                    &mut **scoped_runtime.borrow_mut(),
                ) {
                    Ok(value) => value,
                    Err(error) => return error,
                };
                match prepared.Execute(&value) {
                    Ok(value) => HostResult {
                        value: HostValue::Boolean(value),
                        ..Default::default()
                    },
                    Err(error) => HostResult::Failure(
                        JavaScriptExceptionKind::kTypeError,
                        &format!("Editing command service required: {error:?}"),
                    ),
                }
            })));
        let focus_engine = engine.clone();
        let focus_document = document.clone();
        let focus_bindings = weak.clone();
        let focus_listeners = listeners.clone();
        let focus_errors = errors.clone();
        let focus_reporter = error_reporter.clone();
        bindings
            .borrow_mut()
            .SetFocusChangeHandler(Some(Rc::new(move |id, focus, runtime| {
                let Some(_bindings) = focus_bindings.upgrade() else {
                    return;
                };
                let scoped_runtime = RefCell::new(runtime);
                let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>,
                source_document: &interaction::ownership::InteractionDocument,
                _: &interaction::ownership::InteractionDOMMutationEmitter| {
                focus_listeners.Dispatch(source_document, invocation,
                    &mut **scoped_runtime.borrow_mut(), &mut |error| {
                        let reporter = focus_reporter.borrow().clone();
                        if let Some(reporter) = reporter { reporter(error); }
                        else { focus_errors.borrow_mut().push(error.clone()); }
                    })
            });
                let scoped = focus_engine.WithScopedListeners(listeners);
                scoped.DispatchTargeted(
                    &InputEvent::Focus(FocusEvent {
                        r#type: if focus {
                            FocusEventType::kFocus
                        } else {
                            FocusEventType::kBlur
                        },
                        target_node_id: id,
                        related_target_node_id: None,
                    }),
                    &focus_document,
                );
            })));
        // cpp: browser/browser.cc:668-670
        let form_data_transaction: Rc<RefCell<Option<FormDataTransaction>>> = Default::default();
        let form_submission_transaction: Rc<RefCell<Option<FormSubmissionTransaction>>> =
            Default::default();
        let synthetic_transaction = form_submission_transaction.clone();
        let synthetic_engine = engine.clone();
        let synthetic_document = document.clone();
        let synthetic_bindings = weak.clone();
        let synthetic_listeners = listeners.clone();
        bindings
            .borrow_mut()
            .SetSyntheticEventHandler(Some(Rc::new(move |event, id, runtime| {
                let Some(_bindings) = synthetic_bindings.upgrade() else {
                    return;
                };
                synthetic_listeners.WithScriptRuntime(&synthetic_engine, runtime, |scoped| {
                    scoped.DispatchDOMEvent(event, id, &synthetic_document);
                    let transaction = synthetic_transaction.borrow().clone();
                    if let Some(transaction) = transaction {
                        transaction(scoped, &synthetic_document);
                    }
                });
            })));
        let validation_engine = engine.clone();
        let validation_document = document.clone();
        let validation_bindings = weak.clone();
        let validation_listeners = listeners.clone();
        let validation_errors = errors.clone();
        let validation_reporter = error_reporter.clone();
        bindings.borrow_mut().SetFormValidationHandler(Some(Rc::new(move |id, report, runtime| {
            let Some(_bindings) = validation_bindings.upgrade() else {return Err(interaction::adapter::page_form_validation::ValidationRequired::PatternEngine);};
            let scoped_runtime = RefCell::new(runtime);
            let scoped_listeners = validation_listeners.clone();
            let scoped_reporter = validation_reporter.clone();
            let scoped_errors = validation_errors.clone();
            let listeners = Rc::new(move |invocation: &EventListenerInvocation<'_>, source_document: &interaction::ownership::InteractionDocument, _: &interaction::ownership::InteractionDOMMutationEmitter| {
                scoped_listeners.Dispatch(source_document, invocation, &mut **scoped_runtime.borrow_mut(), &mut |error| {
                    if let Some(reporter) = scoped_reporter.borrow().clone() {reporter(error);} else {scoped_errors.borrow_mut().push(error.clone());}
                })
            });
            let result = validation_engine
                .WithScopedListeners(listeners)
                .CheckValidity(&validation_document, id, report);
            result
        })));
        let submit_transaction = form_submission_transaction.clone();
        let submit_engine = engine.clone();
        let submit_document = document.clone();
        let submit_bindings = weak.clone();
        let submit_listeners = listeners.clone();
        let submit_errors = errors.clone();
        let submit_reporter = error_reporter.clone();
        bindings.borrow_mut().SetFormSubmissionHandler(Some(Rc::new(move |id, submitter, dispatch_event, runtime| {
            let Some(_bindings) = submit_bindings.upgrade() else { return; };
            let scoped_runtime = RefCell::new(runtime);
            let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>, source_document: &interaction::ownership::InteractionDocument, _: &interaction::ownership::InteractionDOMMutationEmitter| {
                submit_listeners.Dispatch(source_document, invocation, &mut **scoped_runtime.borrow_mut(), &mut |error| {
                    if let Some(reporter) = submit_reporter.borrow().clone() { reporter(error); }
                    else { submit_errors.borrow_mut().push(error.clone()); }
                })
            });
            let scoped = submit_engine.WithScopedListeners(listeners);
            let _ = scoped.SubmitForm(&submit_document, id, submitter, dispatch_event);
            let transaction = submit_transaction.borrow().clone();
            if let Some(transaction) = transaction { transaction(&scoped, &submit_document); }
        })));
        let construct_transaction = form_data_transaction.clone();
        let construct_engine = engine.clone();
        let construct_document = document.clone();
        let construct_listeners = listeners.clone();
        let construct_errors = errors.clone();
        let construct_reporter = error_reporter.clone();
        bindings.borrow_mut().SetFormDataConstructor(Some(Rc::new(move |form, submitter, runtime| {
            let scoped_runtime = RefCell::new(runtime);
            let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>, source_document: &interaction::ownership::InteractionDocument, _: &interaction::ownership::InteractionDOMMutationEmitter| {
                construct_listeners.Dispatch(source_document, invocation, &mut **scoped_runtime.borrow_mut(), &mut |error| {
                    if let Some(reporter) = construct_reporter.borrow().clone() { reporter(error); }
                    else { construct_errors.borrow_mut().push(error.clone()); }
                })
            });
            let scoped = construct_engine.WithScopedListeners(listeners);
            match scoped.CreateFormData(&construct_document, form, submitter)? {
                interaction::FormDataCreation::Ready(data) => Ok(data),
                interaction::FormDataCreation::NeedsLifecycle(token) => {
                    let transaction = construct_transaction.borrow().clone().ok_or(
                        interaction::dispatch::core::fileapi::form_data_object::FormDataConstructionError::InvalidState("FormData lifecycle service is unavailable."),
                    )?;
                    transaction(&scoped, &construct_document, token)
                }
            }
        })));
        Self {
            form_controller,
            form_data_transaction,
            form_submission_transaction,
            engine,
            listeners,
            document,
            bindings: weak,
            state,
            errors,
            error_reporter,
            dirty,
            styles_resolved,
        }
    }

    pub(crate) fn BindFrameDomain(
        &self,
        frames: std::rc::Weak<crate::page::local_frame::LocalFrameDomain>,
    ) {
        *self.listeners.frames.borrow_mut() = frames;
    }
    pub(crate) fn BindRealm(&self, realm: &JavaScriptRealm) {
        *self.listeners.realm.borrow_mut() = realm.Downgrade();
    }
    pub(crate) fn OwnsDocument(&self, document: &interaction::InteractionDocument) -> bool {
        self.listeners.Owns(document)
    }
    pub(crate) fn BindingsForDocument(
        &self,
        document: &interaction::InteractionDocument,
    ) -> Option<Rc<RefCell<DOMJavaScriptBindings>>> {
        self.listeners.Bindings(document)
    }
    pub(crate) fn DispatchListeners(
        &self,
        document: &interaction::InteractionDocument,
        invocation: &EventListenerInvocation<'_>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> EventListenerResult {
        let mut result = EventListenerResult::default();
        if let Err(error) = runtime.WithHostRuntime(realm, &mut |runtime| {
            result = self
                .listeners
                .Dispatch(document, invocation, runtime, report_error);
        }) {
            report_error(&error);
        }
        result
    }

    pub(crate) fn SetFormDataTransaction(&self, transaction: FormDataTransaction) {
        *self.form_data_transaction.borrow_mut() = Some(transaction);
    }
    pub(crate) fn SetFormSubmissionTransaction(&self, transaction: FormSubmissionTransaction) {
        *self.form_submission_transaction.borrow_mut() = Some(transaction);
    }
    pub(crate) fn CompleteFormSubmissions(&self, engine: &Interaction<'_>) {
        let transaction = self.form_submission_transaction.borrow().clone();
        if let Some(transaction) = transaction {
            transaction(engine, &self.document);
        }
    }

    pub fn State(&self) -> Rc<RefCell<UserInteractionState>> {
        self.state.clone()
    }
    // cpp: browser/browser.cc:702-734
    // Actual Page reports callback errors at the source callback boundary;
    // standalone service users may still consume the retained error queue.
    pub(crate) fn SetScriptErrorReporter(&self, reporter: Rc<dyn Fn(&JavaScriptException)>) {
        *self.error_reporter.borrow_mut() = Some(reporter);
    }
    pub fn TakePendingExceptions(&self) -> Vec<JavaScriptException> {
        std::mem::take(&mut *self.errors.borrow_mut())
    }
    pub(crate) fn ExceptionSource(&self) -> Rc<dyn Fn() -> Vec<JavaScriptException>> {
        let errors = self.errors.clone();
        Rc::new(move || std::mem::take(&mut *errors.borrow_mut()))
    }
    pub fn IsDirty(&self) -> bool {
        self.dirty.get()
    }
    pub fn StylesResolved(&self) -> bool {
        self.styles_resolved.get()
    }
    // The frame owner acknowledges work only after actual style/layout/paint.
    pub fn DidResolveStyles(&self) {
        self.styles_resolved.set(true);
    }
    pub fn DidPaint(&self) {
        self.dirty.set(false);
    }
    pub fn ApplyStateMutation(&self, mutation: InteractionStateMutation) {
        ApplyInteractionStateMutation(
            &self.document,
            &self.bindings,
            &self.state,
            &self.dirty,
            &self.styles_resolved,
            mutation,
        );
    }
    pub fn Engine(&self) -> &Interaction<'static> {
        &self.engine
    }

    /// Lend the current host invocation to popup lifecycle dispatch. This
    /// scope can synchronously reenter DOM bindings and never retains runtime.
    pub(crate) fn WithScriptRuntime<T>(
        &self,
        runtime: &mut dyn JavaScriptHostRuntime,
        run: impl FnOnce(&Interaction<'_>) -> T,
    ) -> T {
        self.listeners.WithScriptRuntime(&self.engine, runtime, run)
    }

    pub(crate) fn ApplyPopupMutation(
        &self,
        mutation: &dom::dom_mutation::DOMMutation,
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> Result<
        bool,
        interaction::adapter::page_event_dispatcher::pointer_dom_effects::PointerDomError,
    > {
        self.WithScriptRuntime(runtime, |engine| {
            engine.ApplyPopupMutation(&self.document, mutation)
        })
    }

    pub(crate) fn DrainPopupLifecycle(
        &self,
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> Result<(), interaction::adapter::page_event_dispatcher::pointer_dom_effects::PointerDomError>
    {
        self.WithScriptRuntime(runtime, |engine| engine.DrainPopupLifecycle(&self.document))
    }
}

// cpp: browser/browser.cc:985-1001
fn ApplyInteractionStateMutation(
    document: &Rc<RefCell<DOM>>,
    bindings: &std::rc::Weak<RefCell<DOMJavaScriptBindings>>,
    state: &Rc<RefCell<UserInteractionState>>,
    dirty: &Cell<bool>,
    styles_resolved: &Cell<bool>,
    mutation: InteractionStateMutation,
) {
    {
        let owner = document.borrow();
        for id in [
            mutation.state.focused_node_id,
            mutation.state.focus_visible_node_id,
            mutation.state.hovered_node_id,
            mutation.state.pressed_node_id,
            mutation.state.multi_select_focused_option_id,
        ]
        .into_iter()
        .flatten()
        {
            if owner.GetDocument().FindNodeById(id).is_none() {
                dom::error::invalid_argument("interaction state references a missing DOM node");
            }
        }
    }
    if *state.borrow() != mutation.state {
        *state.borrow_mut() = mutation.state;
        if let Some(bindings) = bindings.upgrade() {
            bindings
                .borrow_mut()
                .SetFocusedNode(mutation.state.focused_node_id);
        }
        // User-action selectors and native control paint both consume this
        // snapshot. The next lifecycle republishes it to StyleEngine before
        // resolving the dirty document.
        document
            .borrow_mut()
            .GetDocumentMut()
            .StyleStateMut()
            .impact
            .Merge(dom::style_state::StyleUpdateImpact {
                paint: true,
                ..Default::default()
            });
        styles_resolved.set(false);
        dirty.set(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use javascript::{
        javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
    };

    #[test]
    fn parser_reports_focus_listener_error_before_enclosing_script_error() {
        struct Offline;
        impl url_loader::URLLoader for Offline {
            fn Load(
                &mut self,
                _: &url_loader::URLRequest,
            ) -> std::io::Result<Box<dyn url_loader::URLLoadOperation>> {
                Err(std::io::Error::other("unexpected external resource"))
            }
        }
        #[derive(Default)]
        struct Client(Vec<String>);
        impl open::script_scheduler::ScriptLoadClient for Client {
            fn DidReportScriptError(&mut self, e: &JavaScriptException) {
                self.0.push(e.message.clone());
            }
        }
        let document = Rc::new(RefCell::new(DOM::new()));
        let mutated = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |m| {
                crate::dom_mutation::ApplyDOMTreeMutation(&mut mutated.borrow_mut(), m)
            }),
        )));
        let mut scheduler = open::script_scheduler::ScriptScheduler::new(
            document,
            bindings.clone(),
            Rc::new(RefCell::new(Offline)),
            "https://test.test/".into(),
        );
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        assert!(runtime
            .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let mut client = Client::default();
        scheduler.ParseDocument("<input id=a><input id=b><script>var a=document.getElementById('a');a.focus();a.addEventListener('blur',()=>{throw Error('listener-first')});document.getElementById('b').focus();throw Error('script-second')</script>",256,64,&mut runtime,&realm,&mut client).unwrap();
        assert_eq!(client.0.len(), 2, "{:?}", client.0);
        assert!(client.0[0].contains("listener-first"));
        assert!(client.0[1].contains("script-second"));
    }

    #[test]
    fn source_state_consumer_validates_all_ids_and_invalidates_only_on_change() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML("<input id=a>")));
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(|_| {}),
        )));
        let state = Rc::new(RefCell::new(UserInteractionState::default()));
        let service = PageInteraction::Install(document.clone(), &bindings, state.clone());
        service.DidResolveStyles();
        service.DidPaint();
        service.ApplyStateMutation(InteractionStateMutation::default());
        assert!(service.StylesResolved());
        assert!(!service.IsDirty());
        let node = {
            let owner = document.borrow();
            (0..owner.GetDocument().NodeCount())
                .find_map(|i| {
                    let n = owner.GetDocument().Node(i);
                    (n.FindAttribute("id").is_some_and(|a| a.value == "a")).then_some(n.Id())
                })
                .unwrap()
        };
        service.ApplyStateMutation(InteractionStateMutation {
            state: UserInteractionState {
                focused_node_id: Some(node),
                ..Default::default()
            },
        });
        assert!(!service.StylesResolved());
        assert!(service.IsDirty());
        assert_eq!(state.borrow().focused_node_id, Some(node));
        service.DidResolveStyles();
        service.DidPaint();
        for field in 0..5 {
            let mut invalid = *state.borrow();
            match field {
                0 => invalid.focused_node_id = Some(u64::MAX),
                1 => invalid.focus_visible_node_id = Some(u64::MAX),
                2 => invalid.hovered_node_id = Some(u64::MAX),
                3 => invalid.pressed_node_id = Some(u64::MAX),
                _ => invalid.multi_select_focused_option_id = Some(u64::MAX),
            }
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || service.ApplyStateMutation(InteractionStateMutation { state: invalid })
            ))
            .is_err());
            assert_eq!(state.borrow().focused_node_id, Some(node));
            assert!(service.StylesResolved());
            assert!(!service.IsDirty());
        }
    }

    #[test]
    fn source_window_document_order_and_listener_exception_reporting_survive_reentry() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<input id=a><input id=b>",
        )));
        let mutated = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |m| {
                crate::dom_mutation::ApplyDOMTreeMutation(&mut mutated.borrow_mut(), m)
            }),
        )));
        let service = PageInteraction::Install(
            document,
            &bindings,
            Rc::new(RefCell::new(UserInteractionState::default())),
        );
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        assert!(runtime
            .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
            .Succeeded());
        let result=runtime.Evaluate(&realm,r#"
            var log=[],a=document.getElementById('a'),b=document.getElementById('b');
            window.addEventListener('focusin',()=>log.push('wc'),true);
            document.addEventListener('focusin',()=>log.push('dc'),true);
            a.addEventListener('focusin',()=>log.push('a'));
            document.addEventListener('focusin',()=>log.push('db'));
            window.addEventListener('focusin',()=>log.push('wb'));
            a.focus();
            if(log.join(',')!=='wc,dc,a,db,wb')throw Error('window/document focus order '+log);
            a.addEventListener('blur',()=>{b.setAttribute('data-blur','yes');throw Error('reported listener')});
            b.focus();
            if(document.activeElement!==b||b.getAttribute('data-blur')!=='yes')throw Error('focus failed after listener exception');
        "#,"focus-events");
        assert!(result.Succeeded(), "{:?}", result.exception);
        let errors = service.TakePendingExceptions();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("reported listener"));
        assert!(service.TakePendingExceptions().is_empty());
    }
}

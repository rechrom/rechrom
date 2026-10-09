#![allow(non_snake_case)]

use dom::{InteractionStateMutation, UserInteractionState, DOM};
use interaction::{
    event::{Event, EventListenerInvocation, EventListenerResult, EventType},
    input_event::{FocusEvent, FocusEventType, InputEvent},
    Interaction, InteractionOutput,
};
use javascript::javascript_runtime::{JavaScriptException, JavaScriptHostRuntime};
use layoutng_assembly::fragment_tree::FragmentNode;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use webapi::dom_bindings::DOMJavaScriptBindings;

/// Browser-owned interaction service on the parser's persistent DOM. Its
/// shared state is also the input to layout/measurement, and dirty flags are
/// available to a frame owner; this service does not own a cached frame.
pub struct PageInteraction {
    engine: Interaction<'static>,
    document: Rc<RefCell<DOM>>,
    bindings: std::rc::Weak<RefCell<DOMJavaScriptBindings>>,
    state: Rc<RefCell<UserInteractionState>>,
    errors: Rc<RefCell<Vec<JavaScriptException>>>,
    error_reporter: Rc<RefCell<Option<Rc<dyn Fn(&JavaScriptException)>>>>,
    dirty: Rc<Cell<bool>>,
    styles_resolved: Rc<Cell<bool>>,
}

impl PageInteraction {
    // cpp: browser/browser.cc:629-631,671-676,702-734
    pub fn Install(
        document: Rc<RefCell<DOM>>,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        state: Rc<RefCell<UserInteractionState>>,
    ) -> Self {
        Self::InstallWithEmitter(document, bindings, state, None, Rc::new(Default::default()))
    }

    pub(crate) fn InstallWithEmitter(
        document: Rc<RefCell<DOM>>,
        bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
        state: Rc<RefCell<UserInteractionState>>,
        emitter: Option<interaction::InteractionOutputEmitter>,
        selections: Rc<layoutng_assembly::editing_state::SelectionState>,
    ) -> Self {
        let weak = Rc::downgrade(bindings);
        let mutation_document = document.clone();
        let mutation_bindings = weak.clone();
        let mutation_state = state.clone();
        let dirty = Rc::new(Cell::new(true));
        let styles_resolved = Rc::new(Cell::new(false));
        let mutation_dirty = dirty.clone();
        let mutation_styles = styles_resolved.clone();
        let engine = Interaction::WithSelectionState(
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
        );
        let errors = Rc::new(RefCell::new(Vec::new()));
        let error_reporter: Rc<RefCell<Option<Rc<dyn Fn(&JavaScriptException)>>>> =
            Rc::new(RefCell::new(None));
        let focus_engine = engine.clone();
        let focus_document = document.clone();
        let focus_bindings = weak.clone();
        let focus_errors = errors.clone();
        let focus_reporter = error_reporter.clone();
        bindings
            .borrow_mut()
            .SetFocusChangeHandler(Some(Rc::new(move |id, focus, runtime| {
                let Some(bindings) = focus_bindings.upgrade() else {
                    return;
                };
                let scoped_runtime = RefCell::new(runtime);
                let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>,
                _: &interaction::ownership::InteractionDocument,
                _: &interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchPageListeners(&bindings, &focus_document, invocation,
                    &mut **scoped_runtime.borrow_mut(), &mut |error| {
                        let reporter = focus_reporter.borrow().clone();
                        if let Some(reporter) = reporter { reporter(error); }
                        else { focus_errors.borrow_mut().push(error.clone()); }
                    })
            });
                let scoped = focus_engine.WithScopedListeners(listeners);
                scoped.Dispatch(
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
                    &FragmentNode::default(),
                );
            })));
        // cpp: browser/browser.cc:668-670
        let synthetic_engine = engine.clone();
        let synthetic_document = document.clone();
        let synthetic_bindings = weak.clone();
        let synthetic_errors = errors.clone();
        let synthetic_reporter = error_reporter.clone();
        bindings
            .borrow_mut()
            .SetSyntheticEventHandler(Some(Rc::new(move |event, id, runtime| {
                let Some(bindings) = synthetic_bindings.upgrade() else {
                    return;
                };
                let scoped_runtime = RefCell::new(runtime);
                let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>,
                _: &interaction::ownership::InteractionDocument,
                _: &interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchPageListeners(&bindings, &synthetic_document, invocation,
                    &mut **scoped_runtime.borrow_mut(), &mut |error| {
                        let reporter = synthetic_reporter.borrow().clone();
                        if let Some(reporter) = reporter { reporter(error); }
                        else { synthetic_errors.borrow_mut().push(error.clone()); }
                    })
            });
                synthetic_engine
                    .WithScopedListeners(listeners)
                    .DispatchDOMEvent(event, id, &synthetic_document);
            })));
        let submit_engine = engine.clone();
        let submit_document = document.clone();
        let submit_bindings = weak.clone();
        let submit_errors = errors.clone();
        let submit_reporter = error_reporter.clone();
        bindings.borrow_mut().SetFormSubmissionHandler(Some(Rc::new(move |id, submitter, dispatch_event, runtime| {
            let Some(bindings) = submit_bindings.upgrade() else { return; };
            let scoped_runtime = RefCell::new(runtime);
            let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>, _: &interaction::ownership::InteractionDocument, _: &interaction::ownership::InteractionDOMMutationEmitter| {
                DispatchPageListeners(&bindings, &submit_document, invocation, &mut **scoped_runtime.borrow_mut(), &mut |error| {
                    if let Some(reporter) = submit_reporter.borrow().clone() { reporter(error); }
                    else { submit_errors.borrow_mut().push(error.clone()); }
                })
            });
            submit_engine.WithScopedListeners(listeners).SubmitForm(&submit_document, id, submitter, dispatch_event);
        })));
        Self {
            engine,
            document,
            bindings: weak,
            state,
            errors,
            error_reporter,
            dirty,
            styles_resolved,
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
        // Selector calculation does not consume UserInteractionState. Keep CSS
        // records and let the frame owner refresh native control paint metadata.
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

// cpp: browser/browser.cc:702-734
// Document capture/bubble also invokes Window in source order, using the same
// event identity and accumulated flags.
// Blink WindowEventContext does not connect a node's load event to Window,
// even during capture. Navigation load is dispatched directly to Window.
pub(crate) fn NodeEventReachesWindow(event: &Event) -> bool {
    if event.custom_type.is_empty() {
        event.r#type != EventType::kLoad
    } else {
        event.custom_type != "load"
    }
}

pub(crate) fn DispatchPageListeners(
    bindings: &Rc<RefCell<DOMJavaScriptBindings>>,
    document: &Rc<RefCell<DOM>>,
    invocation: &EventListenerInvocation<'_>,
    runtime: &mut dyn JavaScriptHostRuntime,
    report_error: &mut dyn FnMut(&JavaScriptException),
) -> EventListenerResult {
    let at_document = {
        let owner = document.borrow();
        invocation.current_target_node_id
            == owner.GetDocument().Node(owner.GetDocument().Root()).Id()
    };
    let window = EventListenerInvocation {
        event: invocation.event,
        target_node_id: invocation.target_node_id,
        current_target_node_id: 0,
        phase: invocation.phase,
        capture_listeners: invocation.capture_listeners,
        focused_node_id: invocation.focused_node_id,
    };
    let mut result = EventListenerResult::default();
    let connects_window = NodeEventReachesWindow(invocation.event);
    if at_document && connects_window && invocation.capture_listeners {
        result = DOMJavaScriptBindings::DispatchEventListenersScoped(
            bindings,
            &window,
            runtime,
            report_error,
        );
        if result.stop_propagation || result.stop_immediate_propagation {
            return result;
        }
    }
    fn merge(result: &mut EventListenerResult, next: EventListenerResult) {
        result.prevent_default |= next.prevent_default;
        result.stop_propagation |= next.stop_propagation;
        result.stop_immediate_propagation |= next.stop_immediate_propagation;
    }
    merge(
        &mut result,
        DOMJavaScriptBindings::DispatchEventListenersScoped(
            bindings,
            invocation,
            runtime,
            report_error,
        ),
    );
    if at_document
        && connects_window
        && !invocation.capture_listeners
        && invocation.event.bubbles
        && !result.stop_propagation
        && !result.stop_immediate_propagation
    {
        merge(
            &mut result,
            DOMJavaScriptBindings::DispatchEventListenersScoped(
                bindings,
                &window,
                runtime,
                report_error,
            ),
        );
    }
    result
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
        for field in 0..4 {
            let mut invalid = *state.borrow();
            match field {
                0 => invalid.focused_node_id = Some(u64::MAX),
                1 => invalid.focus_visible_node_id = Some(u64::MAX),
                2 => invalid.hovered_node_id = Some(u64::MAX),
                _ => invalid.pressed_node_id = Some(u64::MAX),
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

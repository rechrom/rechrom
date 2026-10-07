use crate::{
    default_event_handler::DefaultEventHandler,
    event::{Event, EventListenerDispatcher, EventListenerInvocation, EventPhase, EventType},
    event_path::EventPath,
    focus_controller::FocusController,
    input_type::InputTypeContext,
    ownership::{InteractionDOMMutationEmitter, InteractionDocument, ReadNode},
    text_editor::{SyntheticEventDispatcher, TextEditor},
};
use dom::UserInteractionState;
use std::{cell::RefCell, rc::Rc};
// cpp: interaction/event_dispatcher.h:16-21
#[derive(Clone, Copy, Default, Debug)]
pub struct DispatchEventResult {
    pub target_node_id: Option<u64>,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub default_handled: bool,
}
// cpp: interaction/event_dispatcher.h:26-40
// cpp: interaction/event_dispatcher.cc:10-20
#[derive(Clone)]
pub struct EventDispatcher<'a> {
    document: InteractionDocument,
    emit: InteractionDOMMutationEmitter,
    listeners: Rc<RefCell<Option<EventListenerDispatcher<'a>>>>,
    state: Rc<RefCell<UserInteractionState>>,
    editor: Rc<TextEditor>,
    submit_form: Option<Rc<dyn Fn(u64, Option<u64>)>>,
    submitting_forms: Rc<RefCell<Vec<u64>>>,
}
impl<'a> EventDispatcher<'a> {
    pub fn new(
        document: InteractionDocument,
        emit: InteractionDOMMutationEmitter,
        listeners: Rc<RefCell<Option<EventListenerDispatcher<'a>>>>,
        state: Rc<RefCell<UserInteractionState>>,
        editor: Rc<TextEditor>,
        submit_form: Option<Rc<dyn Fn(u64, Option<u64>)>>,
        submitting_forms: Rc<RefCell<Vec<u64>>>,
    ) -> Self {
        Self {
            document,
            emit,
            listeners,
            state,
            editor,
            submit_form,
            submitting_forms,
        }
    }
    fn Context(&self) -> InputTypeContext<'a> {
        let nested = self.clone();
        let dispatch: SyntheticEventDispatcher<'a> = Rc::new(move |event, id| {
            nested.Dispatch(event, id);
        });
        let focus = FocusController::new(
            self.document.clone(),
            self.state.clone(),
            self.editor.clone(),
            dispatch.clone(),
        );
        InputTypeContext {
            submit_form: self.submit_form.clone(),
            submitting_forms: self.submitting_forms.clone(),
            document: self.document.clone(),
            emit_mutation: self.emit.clone(),
            interaction_state: self.state.clone(),
            editor: self.editor.clone(),
            focus_controller: focus,
            dispatch_synthetic: dispatch,
        }
    }
    pub fn SubmitForm(&self, form: u64, submitter: Option<u64>, dispatch_event: bool) {
        crate::input_type::SubmitForm(&self.Context(), form, submitter, dispatch_event);
    }
    // cpp: interaction/event_dispatcher.cc:22-132
    pub fn Dispatch(&self, event: &mut Event, target: u64) -> DispatchEventResult {
        if target == 0 {
            for capture in [true, false] {
                let listener = self.listeners.borrow().clone();
                if let Some(listener) = listener {
                    let focused = self.state.borrow().focused_node_id;
                    let invocation = EventListenerInvocation {
                        event,
                        target_node_id: 0,
                        current_target_node_id: 0,
                        phase: EventPhase::kAtTarget,
                        capture_listeners: capture,
                        // Window events preserve the document's focus just as
                        // node events do. Bindings synchronize activeElement
                        // from this snapshot before invoking listeners.
                        focused_node_id: focused,
                    };
                    let result = listener(&invocation, &self.document, &self.emit);
                    event.default_prevented |= result.prevent_default && event.cancelable;
                    event.propagation_stopped |= result.stop_propagation;
                    event.immediate_propagation_stopped |= result.stop_immediate_propagation;
                    if event.immediate_propagation_stopped {
                        break;
                    }
                }
            }
            return DispatchEventResult {
                target_node_id: Some(0),
                default_prevented: event.default_prevented,
                propagation_stopped: event.propagation_stopped,
                ..Default::default()
            };
        }
        let Some(path) = ReadNode(&self.document, target, |d, _, i| EventPath::new(d, i)) else {
            return DispatchEventResult::default();
        };
        event.target_node_id = target;
        let handler = DefaultEventHandler::new(self.Context());
        let mut activation_target = None;
        if event.r#type == EventType::kClick {
            for &node in path.Nodes() {
                if handler.HasActivationBehavior(node) {
                    activation_target = Some(node);
                    break;
                }
                if !event.bubbles {
                    break;
                }
            }
        }
        let activation_state =
            activation_target.and_then(|n| handler.LegacyPreActivationBehavior(n, event));
        for &node in path.Nodes().iter().skip(1).rev() {
            self.Invoke(event, target, node, EventPhase::kCapturing, true);
            if event.propagation_stopped {
                break;
            }
        }
        if !event.propagation_stopped {
            self.Invoke(event, target, target, EventPhase::kAtTarget, true);
            if !event.immediate_propagation_stopped {
                self.Invoke(event, target, target, EventPhase::kAtTarget, false)
            }
        }
        if !event.propagation_stopped && event.bubbles {
            for &node in path.Nodes().iter().skip(1) {
                self.Invoke(event, target, node, EventPhase::kBubbling, false);
                if event.propagation_stopped {
                    break;
                }
            }
        }
        event.phase = EventPhase::kNone;
        event.current_target_node_id = 0;
        event.capture_listeners = false;
        if let Some(node) = activation_target {
            handler.RunActivationBehavior(node, event, activation_state.as_ref());
        }
        if !event.default_prevented && !event.default_handled {
            handler.Handle(target, event);
            if !event.default_prevented && !event.default_handled && event.bubbles {
                for &node in path.Nodes().iter().skip(1) {
                    handler.Handle(node, event);
                    if event.default_prevented || event.default_handled {
                        break;
                    }
                }
            }
        }
        DispatchEventResult {
            target_node_id: Some(target),
            default_prevented: event.default_prevented,
            propagation_stopped: event.propagation_stopped,
            default_handled: event.default_handled,
        }
    }
    // cpp: interaction/event_dispatcher.cc:65-89
    fn Invoke(
        &self,
        event: &mut Event,
        target: u64,
        current: u64,
        phase: EventPhase,
        capture: bool,
    ) {
        let Some(listener) = self.listeners.borrow().clone() else {
            return;
        };
        event.current_target_node_id = current;
        event.phase = phase;
        event.capture_listeners = capture;
        let focused = self.state.borrow().focused_node_id;
        let result = listener(
            &EventListenerInvocation {
                event,
                target_node_id: target,
                current_target_node_id: current,
                phase,
                capture_listeners: capture,
                focused_node_id: focused,
            },
            &self.document,
            &self.emit,
        );
        if event.cancelable {
            event.default_prevented |= result.prevent_default
        }
        event.propagation_stopped |= result.stop_propagation || result.stop_immediate_propagation;
        event.immediate_propagation_stopped |= result.stop_immediate_propagation;
    }
}

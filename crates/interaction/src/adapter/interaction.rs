//! Retained document/editor owners and synchronous host-only source stages.
//! Raw input is bound by page_interaction to the native EventHandler pipeline.

pub use super::interaction_contract::{
    InteractionEffect, InteractionOutput, InteractionOutputEmitter, InteractionResult,
    InteractionState,
};
use crate::{
    editing_owners::PageEditingOwners,
    event::{Event, EventListenerDispatcher, EventListenerInterestRef},
    event_dispatcher::EventDispatcher,
    input_event::*,
    ownership::{InteractionDOMMutationEmitter, InteractionDocument, ReadNode},
};
use dom::InteractionStateMutation;
use interaction_scroll::core::input::event_handler::EventHandlerScrollRouting;
use layoutng_assembly::{
    editing_state::SelectionState, internal::layout_input::Offset,
    layout_engine::PaintFragmentCursor,
};
use std::{cell::RefCell, ops::Range, rc::Rc};

pub use interaction_hit_test::resident::point_queries::ResidentFragmentSource as InteractionFragmentSource;

// cpp: interaction/interaction_engine.cc:88-118
fn ExplicitTarget(input: &InputEvent) -> Option<u64> {
    match input {
        InputEvent::Focus(v) => (v.target_node_id != 0).then_some(v.target_node_id),
        InputEvent::Mouse(v) => v.target_node_id,
        InputEvent::Pointer(v) => v.target_node_id,
        InputEvent::Touch(_) | InputEvent::Gesture(_) => None,
        InputEvent::Wheel(v) => v.target_node_id,
        InputEvent::Key(v) => v.target_node_id,
        InputEvent::Composition(v) => v.target_node_id,
        InputEvent::TextInput(v) => v.target_node_id,
    }
}
// cpp: interaction/interaction_engine.h:36-56
// Clones retain the same input state/editor and listener slot. This permits
// source synchronous listener reentry without an exclusive Interaction borrow.
#[derive(Clone)]
pub struct Interaction<'a> {
    script_invocation: Option<&'a dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation>,
    event_timing: Rc<RefCell<Option<Rc<interaction_default::core::timing::window_performance::WindowPerformance>>>>,
    pub(super) pointer_binding: Rc<RefCell<Option<PointerInputBinding>>>,
    pointer_input_error: Rc<RefCell<Option<crate::event_dispatcher::PointerInputError>>>,
    pub(super) clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    pub(super) emit: InteractionOutputEmitter,
    form_navigation_preparation: Rc<RefCell<Option<Rc<dyn super::interaction_contract::FormNavigationPreparation>>>>,
    pub(super) listeners: Rc<RefCell<Option<EventListenerDispatcher<'a>>>>,
    listener_interest: Rc<RefCell<Option<EventListenerInterestRef>>>,
    mouse_frame_geometry: Rc<RefCell<Option<crate::event_dispatcher::MouseFrameGeometryRef>>>,
    state: Rc<RefCell<InteractionState>>,
    pub(super) editor: Rc<PageEditingOwners>,
    selections: Rc<SelectionState>,
    default_action_state: Rc<crate::input_type::DefaultActionState>,
    dispatch_state: Rc<RefCell<Option<Rc<crate::event_dispatcher::PageDispatchState>>>>,
    pub(super) native_wheel: Rc<RefCell<Option<Rc<EventHandlerScrollRouting>>>>,
    pub(super) native_scroll: Rc<RefCell<Option<Rc<super::page_scroll_manager::PageScrollManagerOwners>>>>,
    focus_services:
        Rc<RefCell<Option<Rc<super::page_focus_controller::DocumentFocusServices>>>>,
    pub(super) focus_navigation:
        Rc<RefCell<Option<Rc<super::page_focus_navigation::PageFocusNavigation>>>>,
    input_method_state: Rc<RefCell<Rc<
        interaction_default::core::editing::ime::input_method_controller::InputMethodControllerState,
    >>>,
    node_removal_observer:
        Rc<RefCell<Option<Rc<dyn dom::node_removal_observer::DocumentNodeRemovalObserver>>>>,
    pub(super) source_input: super::source_input_managers::ResidentCursorOwners,
    widget_cursor_control: Rc<RefCell<interaction_input::platform::widget::input::cursor_control_handler::WidgetCursorControl>>,
}

/// Canonical editable text state exported to the platform IME host.
/// Offsets are UTF-16 code units, matching Blink and AppKit's text ranges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextInputState {
    pub text: Vec<u16>,
    pub selection: Range<usize>,
    pub composition: Option<Range<usize>>,
}
/// Document exposes only the operation and primitive command result; the
/// descriptor, selection, compiled declarations and Editor remain private.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentEditorCommandOperation {
    Execute,
    Supported,
    Enabled,
    State,
    Indeterm,
    Value,
}
#[derive(Clone, Debug)]
pub enum DocumentEditorCommandResult {
    Boolean(bool),
    Text(String),
}
/// Script Selection's directional endpoints projected from the canonical
/// FrameSelection.  Range ordering remains a DOM concern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMSelectionSnapshot {
    pub anchor: dom::DOMBoundaryPoint,
    pub focus: dom::DOMBoundaryPoint,
}
/// HTMLInputElement/HTMLTextAreaElement selection API projection. The
/// endpoints remain owned by LayoutNG's retained SelectionState.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextControlSelectionSnapshot {
    pub start: usize,
    pub end: usize,
    pub backward: bool,
}
/// The original Document exec scope survives a WebAPI Trusted Types callback.
/// Its fields remain private; the embedding only checks the sink kind and
/// supplies the already checked value to Execute.
pub struct PreparedDocumentEditorCommand<'a> {
    // Drop queued events before clearing the source recursion flag.
    _events: interaction_dispatch::core::dom::events::scoped_event_queue::EventQueueScope<'a>,
    _command: interaction_default::core::editing::editor::EditorExecCommandScope,
    dispatcher: EventDispatcher<'a, 'a>,
    command: interaction_default::core::editing::commands::editor_command_table::EditingCommandType,
}
impl PreparedDocumentEditorCommand<'_> {
    pub fn IsValueInterpretedAsHtml(&self) -> bool {
        use interaction_default::core::editing::commands::editor_command_table::{
            EditingCommandType as C, EditorCommandSource as S,
        };
        self.command == C::InsertHTML
            && self
                .dispatcher
                .EditorCommandSupported(self.command, S::kDom)
    }
    pub fn Execute(
        &self,
        value: &str,
    ) -> Result<bool, super::page_event_dispatcher::EditorCommandError> {
        self.dispatcher.ExecuteEditorCommandFromSource(self.command, value, interaction_default::core::editing::commands::editor_command_table::EditorCommandSource::kDom)
    }
}
/// A constructor continuation carries no DOM/Layout borrow and no navigation
/// parameters. Page resumes it synchronously before the binding returns.
pub enum FormDataCreation {
    Ready(Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>),
    NeedsLifecycle(crate::input_type::FormSubmissionToken),
}

#[derive(Clone)]
pub(super) struct PointerInputBinding {
    pub(super) geometry: Rc<dyn super::page_pointer_hit_test::PagePointerGeometry>,
    pub(super) controls: Rc<super::page_pointer_scroll_controls::PagePointerScrollControlOwners>,
}

// Source: element.cc:10091-10131
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerCaptureError {
    NoActivePointer,
    InvalidState,
}
// cpp: interaction/interaction_engine.cc:149-161
pub(super) struct EmitStateMutationAtExit {
    before: InteractionState,
    current: Rc<RefCell<InteractionState>>,
    emit: InteractionOutputEmitter,
}
impl Drop for EmitStateMutationAtExit {
    fn drop(&mut self) {
        let state = *self.current.borrow();
        if state != self.before {
            (self.emit)(InteractionOutput::StateMutation(InteractionStateMutation {
                state,
            }))
        }
    }
}
impl<'a> Interaction<'a> {
    // cpp: interaction/interaction_engine.cc:122-144
    // Rust's nonnullable Rc callback enforces the source required mutation output.
    pub fn new(
        emit: InteractionOutputEmitter,
        listeners: Option<EventListenerDispatcher<'a>>,
        clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    ) -> Self {
        Self::WithSelectionState(emit, listeners, Rc::new(Default::default()), clock)
    }
    pub fn WithSelectionState(
        emit: InteractionOutputEmitter,
        listeners: Option<EventListenerDispatcher<'a>>,
        selections: Rc<layoutng_assembly::editing_state::SelectionState>,
        clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    ) -> Self {
        let editor = Rc::new(PageEditingOwners::WithSelectionState(selections.clone()));
        Self {
            script_invocation: None,
            event_timing: Default::default(),
            pointer_binding: Rc::new(RefCell::new(None)),
            pointer_input_error: Rc::new(RefCell::new(None)),
            clock,
            emit,
            form_navigation_preparation: Rc::new(RefCell::new(None)),
            listeners: Rc::new(RefCell::new(listeners)),
            listener_interest: Rc::new(RefCell::new(None)),
            mouse_frame_geometry: Rc::new(RefCell::new(None)),
            state: Rc::new(RefCell::new(InteractionState::default())),
            editor,
            selections,
            default_action_state: Rc::new(Default::default()),
            dispatch_state: Rc::new(RefCell::new(None)),
            native_wheel: Rc::new(RefCell::new(None)),
            native_scroll: Rc::new(RefCell::new(None)),
            focus_services: Rc::new(RefCell::new(None)),
            focus_navigation: Rc::new(RefCell::new(None)),
            input_method_state: Rc::new(RefCell::new(Rc::new(Default::default()))),
            node_removal_observer: Rc::new(RefCell::new(None)),
            source_input: Default::default(),
            widget_cursor_control: Default::default(),
        }
    }
    pub fn WithDOMMutationEmitter(
        emit: InteractionDOMMutationEmitter,
        listeners: Option<EventListenerDispatcher<'a>>,
        clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    ) -> Self {
        Self::new(
            Rc::new(move |output| {
                if let InteractionOutput::DocumentMutation(mutation) = output {
                    emit(&mutation)
                }
            }),
            listeners,
            clock,
        )
    }
    // cpp: interaction/interaction_engine.h:47-51
    pub fn SetEventListenerDispatcher(&self, dispatcher: Option<EventListenerDispatcher<'a>>) {
        *self.listeners.borrow_mut() = dispatcher;
    }
    pub fn SetAnchorNavigationHost(
        &self,
        host: Option<
            Rc<dyn interaction_default::core::html::html_anchor_element::AnchorNavigationHost>,
        >,
    ) {
        *self
            .source_input
            .pointer_frame
            .anchor_navigation
            .borrow_mut() = host;
    }
    pub fn SetDelayedNavigationTaskHandle(
        &self,
        handle: Rc<
            dyn interaction_default::core::html::html_anchor_element::AnchorNavigationTaskHandle,
        >,
    ) {
        self.source_input
            .EventHandlerState()
            .SetDelayedNavigationTaskHandle(handle);
    }
    pub fn WithScriptInvocation<'scope>(
        &'scope self,
        invocation: &'scope dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation,
    ) -> Interaction<'scope>
    where
        'a: 'scope,
    {
        let mut scoped = self.WithOptionalScopedListeners(self.listeners.borrow().clone());
        scoped.script_invocation = Some(invocation);
        scoped
    }
    pub(super) fn ScriptInvocation(
        &self,
    ) -> Option<&'a dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation> {
        self.script_invocation
    }
    /// LocalDOMWindow's canonical performance owner, available before a script
    /// realm exists. Scoped listener adapters retain this same Window identity.
    pub fn BindWindowPerformance(
        &self,
        performance: Rc<interaction_default::core::timing::window_performance::WindowPerformance>,
    ) {
        *self.event_timing.borrow_mut() = Some(performance);
    }
    /// Installs the original EventTarget listener-map owner. This is distinct
    /// from the call-scoped listener dispatcher: boundary dispatch re-queries
    /// capture interest after synchronous leave/over listener mutation.
    pub fn SetEventListenerInterest(&self, interest: Option<EventListenerInterestRef>) {
        *self.listener_interest.borrow_mut() = interest;
    }
    /// Installs the Page's live LocalFrameView facts. Interaction consumes this
    /// narrow read-only port while constructing native MouseEvents; it neither
    /// owns nor exposes the Page's LayoutEngine.
    pub fn SetMouseFrameGeometry(
        &self,
        geometry: Option<crate::event_dispatcher::MouseFrameGeometryRef>,
    ) {
        *self.mouse_frame_geometry.borrow_mut() = geometry;
    }
    pub(super) fn NativeMouseFrameGeometry(
        &self,
    ) -> Option<crate::event_dispatcher::MouseFrameGeometryRef> {
        self.mouse_frame_geometry.borrow().clone()
    }
    pub fn BindPointerInput(
        &self,
        geometry: Rc<dyn super::page_pointer_hit_test::PagePointerGeometry>,
        controls: Rc<dyn super::page_pointer_scroll_controls::PagePointerScrollControls>,
    ) {
        let controls = Rc::new(
            super::page_pointer_scroll_controls::PagePointerScrollControlOwners::new(controls),
        );
        *self.pointer_binding.borrow_mut() = Some(PointerInputBinding { geometry, controls });
    }
    pub fn TakeUnbufferedInputRequest(&self) -> bool {
        self.source_input
            .pointer_frame
            .unbuffered_requested
            .replace(false)
    }
    /// EventHandler's ancestor over/out step before forwarding to a local
    /// subframe. Reuses the canonical pointer/mouse boundary dispatch owners.
    pub fn DispatchLocalFrameMouseBoundary(
        &self,
        mouse: &super::input_event::MouseEvent,
        document: &InteractionDocument,
        target: Option<u64>,
        left_frame: bool,
    ) -> Result<(), super::native_interaction::InteractionError> {
        let _emit_state = self.ExitGuard();
        let scale = self
            .mouse_frame_geometry
            .borrow()
            .as_ref()
            .and_then(|geometry| geometry.LayoutZoomFactor())
            .unwrap_or(1.0);
        let native = mouse
            .ToWebEventWithWidgetScale(self.clock.Now(), scale)
            .map_err(|error| {
                super::native_interaction::InteractionError::SourceInput(format!(
                    "boundary mouse input: {error:?}"
                ))
            })?;
        let dispatcher = self.Dispatcher(document, None);
        if target.is_some() && mouse.r#type == MouseEventType::kDown {
            if let Some(binding) = self.pointer_binding.borrow().clone() {
                let point = self
                    .mouse_frame_geometry
                    .borrow()
                    .as_ref()
                    .and_then(|frame| frame.ConvertFromRootFrame(native.PositionInRootFrame()))
                    .expect("prepared local FrameView");
                let hit = binding.geometry.HitTestInLocalFrame(
                    interaction_hit_test::core::layout::hit_test_location::HitTestLocation::FromPointF(point),
                    interaction_hit_test::core::layout::hit_test_request::RequestType::kReadOnly as u32,
                ).map_err(|error| super::native_interaction::InteractionError::SourceInput(format!("parent selection hit: {error:?}")))?;
                super::page_event_dispatcher::gesture_mouse_defaults::PageGestureMouseDefaults::new(
                    &dispatcher, &self.source_input, binding.geometry, binding.controls,
                ).PassPressToSubframe(&native, &hit.result)
                    .map_err(|error| super::native_interaction::InteractionError::SourceInput(format!("parent selection press: {error:?}")))?;
            }
        }
        dispatcher.SetElementUnderMouseAndDispatchBoundaryEvents(
            self.source_input.pointer.as_ref(),
            &self.source_input.mouse,
            &native,
            target,
        );
        self.state.borrow_mut().hovered_node_id = target;
        if mouse.r#type == MouseEventType::kDown {
            self.state.borrow_mut().pressed_node_id = target;
        }
        if mouse.r#type == MouseEventType::kUp {
            self.state.borrow_mut().pressed_node_id = None;
        }
        if left_frame {
            self.source_input.mouse.SetLastMousePositionAsUnknown();
            self.source_input.pointer.RemoveLastMousePosition();
        }
        self.CompleteInput(InteractionResult::default()).map(|_| ())
    }
    pub fn CurrentTouchActionForGesture(
        &self,
    ) -> interaction_input::ui::events::gesture_detection::filtered_gesture_provider::TouchAction
    {
        interaction_input::ui::events::gesture_detection::filtered_gesture_provider::TouchAction(
            self.source_input.pointer_frame.touch_action.get().0 as u8,
        )
    }
    pub fn RunScrollbarTimerTask(
        &self,
        task: super::page_pointer_scroll_controls::ScrollbarTimerTask,
    ) -> std::io::Result<()> {
        let controls = self
            .pointer_binding
            .borrow()
            .as_ref()
            .map(|binding| binding.controls.clone())
            .ok_or_else(|| std::io::Error::other("scrollbar timer input owner is detached"))?;
        controls.RunTimer(task)
    }
    pub fn HandleNativeScrollbarActivities(
        &self,
        activities: Vec<
            layoutng_assembly::internal::layout_scrollable_area::NativeScrollbarActivity,
        >,
    ) -> std::io::Result<()> {
        let controls = self
            .pointer_binding
            .borrow()
            .as_ref()
            .map(|binding| binding.controls.clone())
            .ok_or_else(|| std::io::Error::other("scrollbar input owner is detached"))?;
        controls.HandleNativeActivities(activities)
    }
    pub fn HasTransientUserActivation(&self) -> bool {
        self.source_input
            .pointer_frame
            .activation
            .borrow()
            .IsActive(self.clock.Now())
    }
    pub fn HasStickyUserActivation(&self) -> bool {
        self.source_input
            .pointer_frame
            .activation
            .borrow()
            .HasBeenActive()
    }
    pub fn IsHistoryUserActivationActive(&self) -> bool {
        self.source_input
            .pointer_frame
            .history_activation
            .borrow()
            .IsActive()
    }
    pub fn NotifyHistoryUserActivationForFrame(
        &self,
        document: &InteractionDocument,
        now: interaction_input::base::time::time::TimeTicks,
    ) {
        super::page_event_dispatcher::pointer_dom_effects::RecordHadUserInteraction(
            &self.Dispatcher(document, None),
            true,
            now,
        );
    }
    /// Page calls this on each live local frame after the typed consumption
    /// effect. It deliberately leaves transient activation unchanged.
    pub fn ConsumeHistoryUserActivationForFrame(&self) {
        self.source_input
            .pointer_frame
            .history_activation
            .borrow_mut()
            .Consume();
    }
    pub fn ConsumeHistoryUserActivation(&self) {
        (self.emit)(InteractionOutput::Effect(
            InteractionEffect::ConsumeHistoryUserActivation,
        ));
    }
    pub fn CaptureMouseEventsToWidget(&self, capture: bool) {
        self.source_input
            .event_handler
            .CaptureMouseEventsToWidget(capture, |capture| {
                (self.emit)(InteractionOutput::Effect(
                    InteractionEffect::CaptureMouseEventsToWidget(capture),
                ));
            });
    }
    pub fn ConsumeTransientUserActivation(&self) -> bool {
        self.source_input
            .pointer_frame
            .activation
            .borrow_mut()
            .ConsumeIfActive(self.clock.Now())
    }
    /// Installs EventHandler's retained MouseWheelEventManager. The manager
    /// owns target latching across wheel phases and retains the exact same
    /// ScrollManager owner used by the remaining translated input managers.
    pub fn SetNativeWheelEventHandler(&self, manager: Option<Rc<EventHandlerScrollRouting>>) {
        *self.native_wheel.borrow_mut() = manager;
    }
    pub fn HasRetainedWheelTarget(&self) -> bool {
        self.native_wheel
            .borrow()
            .as_ref()
            .is_some_and(|handler| handler.GetMouseWheelEventManager().HasWheelTarget())
    }
    pub fn ClearRetainedWheelTarget(&self) {
        if let Some(handler) = self.native_wheel.borrow().as_ref().cloned() {
            handler.GetMouseWheelEventManager().Clear();
        }
    }

    /// Connects the single Page ScrollManager to EventHandler's retained wheel
    /// manager using this facade's canonical DOM target registry and mouse
    /// state. `BindDocumentInputLifecycle` must run first for the current
    /// Document owner.
    pub fn BindNativeWheelEventHandler(
        &self,
        scroll: Rc<super::page_scroll_manager::PageScrollManagerOwners>,
        features: interaction_scroll::core::input::mouse_wheel_event_manager::MouseWheelFeatures,
    ) -> bool {
        let Some(dispatch_state) = self.dispatch_state.borrow().as_ref().cloned() else {
            return false;
        };
        let handler = scroll.LocalRootWheelEventHandler(
            dispatch_state.WheelTargetIdentity(),
            self.source_input.mouse.clone(),
            features,
        );
        *self.native_scroll.borrow_mut() = Some(scroll);
        self.SetNativeWheelEventHandler(Some(handler));
        true
    }
    /// EventHandler::BubblingScroll enters the parent's retained manager at
    /// the embedded frame owner, preserving the native ancestor traversal.
    pub fn BubbleLogicalScroll(
        &self,
        direction: interaction_scroll::core::input::scroll_manager::ScrollDirection,
        granularity: interaction_scroll::core::input::scroll_manager::ScrollGranularity,
        starting_node_id: Option<u64>,
    ) -> bool {
        let Some(owners) = self.native_scroll.borrow().clone() else {
            return false;
        };
        owners.ScrollManager().BubblingScroll(
            direction,
            granularity,
            starting_node_id.map(|node| owners.KeyboardScrollNode(node)),
            None,
            false,
        )
    }

    pub fn WithScopedListeners<'b>(&self, listeners: EventListenerDispatcher<'b>) -> Interaction<'b>
    where
        'a: 'b,
    {
        self.WithOptionalScopedListeners(Some(listeners))
    }
    pub(super) fn WithOptionalScopedListeners<'b>(
        &self,
        listeners: Option<EventListenerDispatcher<'b>>,
    ) -> Interaction<'b>
    where
        'a: 'b,
    {
        Interaction {
            script_invocation: self.script_invocation,
            event_timing: self.event_timing.clone(),
            pointer_binding: self.pointer_binding.clone(),
            pointer_input_error: self.pointer_input_error.clone(),
            clock: self.clock.clone(),
            emit: self.emit.clone(),
            form_navigation_preparation: self.form_navigation_preparation.clone(),
            listeners: Rc::new(RefCell::new(listeners)),
            listener_interest: self.listener_interest.clone(),
            mouse_frame_geometry: self.mouse_frame_geometry.clone(),
            state: self.state.clone(),
            editor: self.editor.clone(),
            selections: self.selections.clone(),
            default_action_state: self.default_action_state.clone(),
            dispatch_state: self.dispatch_state.clone(),
            native_wheel: self.native_wheel.clone(),
            native_scroll: self.native_scroll.clone(),
            focus_services: self.focus_services.clone(),
            focus_navigation: self.focus_navigation.clone(),
            input_method_state: self.input_method_state.clone(),
            node_removal_observer: self.node_removal_observer.clone(),
            source_input: self.source_input.clone(),
            widget_cursor_control: self.widget_cursor_control.clone(),
        }
    }

    /// Connects Document's pre-unlink NodeWillBeRemoved stage to the same
    /// retained MouseEventManager state used by production input dispatch.
    pub fn BindDocumentInputLifecycle(&self, document: &InteractionDocument) {
        // LocalFrameHost::SetCloseListener uses the existing Page output seam.
        // The callback retains only a weak emitter, never the Document/engine.
        let emit = Rc::downgrade(&self.emit);
        document
            .borrow_mut()
            .GetDocumentMut()
            .PopupLifecycleStateMut()
            .close_watcher_stack
            .close_signal_host = Some(Rc::new(move || {
            if let Some(emit) = emit.upgrade() {
                emit(InteractionOutput::Effect(
                    InteractionEffect::SignalCloseWatchers,
                ));
            }
        }));
        super::page_form_validation::InstallInputValueSanitizer(
            document,
            self.default_action_state.clone(),
        );
        self.CaptureMouseEventsToWidget(false);
        // LocalFrame navigation clears PointerEventManager/Factory capture and
        // target state before the replacement Document starts receiving input.
        if let Some(transfer) = self
            .source_input
            .drag
            .GetDragState()
            .drag_data_transfer
            .borrow()
            .clone()
        {
            transfer.ClearDragImage();
            transfer.SetAccessPolicyNumb();
        }
        self.source_input.drag.ContextDestroyed();
        if let Some(owner) = self.source_input.autoscroll_services.borrow().clone() {
            self.source_input
                .autoscroll
                .StopAutoscroll(owner.StopServices());
            self.source_input
                .autoscroll
                .StopMiddleClickAutoscroll(owner.StopServices());
        }
        self.source_input.EventHandlerState().ClearRetainedState();
        self.source_input.pointer.Clear();
        self.source_input.gesture.Clear();
        self.source_input.gesture_target.Clear();
        self.source_input
            .pointer_frame
            .activation
            .borrow_mut()
            .Clear();
        self.source_input
            .pointer_frame
            .unbuffered_requested
            .set(false);
        if let Some(binding) = self.pointer_binding.borrow().as_ref() {
            binding.controls.Clear();
        }
        self.source_input.event_handler.ClearMouseCapture();
        self.source_input
            .event_handler
            .ClearLastScrollbarUnderMouse();
        // InputMethodController belongs to LocalFrame's current Document. A
        // replacement document receives fresh composition/range state while
        // scoped Interaction clones continue to share that current owner.
        *self.input_method_state.borrow_mut() = Rc::new(Default::default());
        self.editor.ime_typing.Reset();
        *self.focus_services.borrow_mut() = None;
        *self.focus_navigation.borrow_mut() = None;
        let dispatch_state = crate::event_dispatcher::PageDispatchState::ForDocument(
            &self.dispatch_state,
            document.clone(),
            self.listener_interest.clone(),
        );
        let observer = dispatch_state.NodeRemovalObserver(
            self.source_input.mouse.clone(),
            self.source_input.pointer.clone(),
            self.native_wheel.clone(),
        );
        document
            .borrow_mut()
            .GetDocumentMut()
            .SetNodeRemovalObserver(Some(Rc::downgrade(&observer)));
        *self.node_removal_observer.borrow_mut() = Some(observer);
    }

    /// Installs the retained FocusController/ScopedFocusNavigation owner for
    /// this Document lifetime. Layout and frame ports expose only live scalar
    /// facts/effects; all traversal, focusability and dispatch stay inside
    /// Interaction. Source: focus_controller.cc:2043-2274.
    pub fn BindFocusNavigation(
        &self,
        layout: Rc<dyn super::page_focus_navigation::FocusNavigationLayout>,
        frame: Rc<dyn super::page_focus_navigation::FocusNavigationFramePort>,
        lifecycle: Rc<dyn super::page_focus_controller::DocumentFocusLifecycle>,
        policy: super::page_focus_navigation::FocusNavigationPolicy,
        effects: interaction_default::core::page::focus_controller::FocusEffectEmitter,
    ) -> bool {
        let Some(dispatch_state) = self.dispatch_state.borrow().as_ref().cloned() else {
            return false;
        };
        let targets = dispatch_state.Targets();
        let document = targets.DocumentOwner();
        let selection = super::page_frame_selection::PageFrameSelection::new(
            document.clone(),
            self.selections.clone(),
            self.editor.clone(),
            lifecycle.FrameSelectionHost(),
        );
        let services = Rc::new(super::page_focus_controller::DocumentFocusServices {
            layout: layout.clone(),
            lifecycle,
            selection,
        });
        let frame_tree = frame.FrameTree();
        let actions = super::page_focus_navigation::PageFocusNavigationActions::new(
            document.clone(),
            self.state.clone(),
            targets.clone(),
            self.editor.clone(),
            self.default_action_state.clone(),
            layout.clone(),
            frame,
            services.clone(),
        );
        let special = super::page_focus_navigation::PageFocusNavigationSpecialOwners::new(
            document.clone(),
            self.state.clone(),
            targets.clone(),
            layout.clone(),
            policy,
            frame_tree.clone(),
        );
        *self.focus_navigation.borrow_mut() = Some(Rc::new(
            super::page_focus_navigation::PageFocusNavigation::new(
                document,
                self.state.clone(),
                targets,
                layout,
                special,
                actions,
                effects,
                frame_tree,
            ),
        ));
        *self.focus_services.borrow_mut() = Some(services);
        true
    }
    pub fn FocusDocumentEventTarget(
        &self,
    ) -> Option<Rc<dyn super::page_focus_navigation::FocusDocumentEventTarget>>
    where
        'a: 'static,
    {
        let state = self.dispatch_state.borrow().as_ref()?.clone();
        Some(Rc::new(FocusDocumentEvents {
            engine: self.clone(),
            document: state.Targets().DocumentOwner(),
        }))
    }
    pub fn FocusFrameOwner(
        &self,
    ) -> Option<interaction_default::core::page::focus_controller::FocusFrameRef> {
        Some(self.focus_navigation.borrow().as_ref()?.Frame())
    }
    pub fn FocusLocalFrameEvents(
        &self,
    ) -> Option<interaction_default::core::page::focus_controller::FocusFrameRef> {
        Some(self.focus_navigation.borrow().as_ref()?.LocalFrameEvents())
    }
    pub fn FocusElementOwner(
        &self,
        node: u64,
    ) -> Option<interaction_default::core::page::focus_controller::FocusElementRef> {
        self.focus_navigation.borrow().as_ref()?.Element(node)
    }
    pub fn DocumentActiveElementNode(&self) -> Option<u64> {
        self.focus_navigation.borrow().as_ref()?.ActiveElementNode()
    }
    /// TextInputStateChanged for a focused text control. The platform host
    /// borrows this already-owned value/selection state; it never reaches back
    /// into DOM or editing owners while AppKit synchronously queries the IME.
    pub fn TextInputState(&self, document: &InteractionDocument) -> Option<TextInputState> {
        let focused = self.state.borrow().focused_node_id?;
        if self.selections.ActiveControl() != Some(focused) {
            return None;
        }
        let selection = self.selections.Get(focused)?;
        let composition = self.selections.Composition(focused);
        ReadNode(document, focused, |document, node, index| {
            if !node.IsHTMLElement("input") && !node.IsHTMLElement("textarea") {
                return None;
            }
            let text = document.TextControlInnerEditorValueUTF16(index);
            let selection = selection.Start().min(text.len())..selection.End().min(text.len());
            let composition = composition.map(|range| {
                range.Start().min(text.len())..range.End().min(text.len())
            });
            Some(TextInputState {
                text: text.to_vec(),
                selection,
                composition,
            })
        })?
    }
    pub fn DocumentHasFocus(&self, page_active: bool, page_focused: bool) -> bool {
        self.focus_navigation
            .borrow()
            .as_ref()
            .is_some_and(|owner| owner.DocumentHasFocus(page_active, page_focused))
    }
    pub fn FocusControllerOwner(
        &self,
    ) -> Option<Rc<interaction_default::core::page::focus_controller::FocusController>> {
        Some(self.focus_navigation.borrow().as_ref()?.ControllerOwner())
    }
    /// The same retained FrameSelection that handles focus, editing and
    /// gesture transactions. Query services borrow this identity directly.
    /// Widget-level cursor control precedes ordinary gesture dispatch.
    pub fn WillHandleCursorControlGesture(&self, event: &WebGestureEvent) -> bool {
        if !self
            .widget_cursor_control
            .borrow_mut()
            .WillHandleGestureEvent(event)
        {
            return false;
        }
        if let Some(selection) = self.FrameSelectionOwner() {
            let point = event.PositionInWidget();
            // The embedding completes the selection lifecycle before this
            // source call, and FrameSelection uses its live point-position port.
            selection.MoveCaretSelection((point.x() as i32, point.y() as i32));
        }
        true
    }
    pub fn FrameSelectionOwner(
        &self,
    ) -> Option<Rc<super::page_frame_selection::PageFrameSelection>> {
        self.focus_services
            .borrow()
            .as_ref()
            .map(|services| services.selection.clone())
    }
    /// Script-facing Selection reads project the retained FrameSelection's
    /// original DOM positions.  WebAPI receives boundary values only; it does
    /// not retain another selection or infer positions from wrappers.
    pub fn DOMSelectionSnapshot(
        &self,
        document: &InteractionDocument,
    ) -> Option<DOMSelectionSnapshot> {
        use layoutng_assembly::EditingPositionAnchorType as A;
        let selection = self.FrameSelectionOwner()?.Current()?;
        let owner = document.borrow();
        let document = owner.GetDocument();
        let boundary = |position: layoutng_assembly::EditingPosition| {
            let node = document.FindNodeById(position.node_id?)?;
            let point = match position.anchor {
                A::OffsetInAnchor => dom::DOMBoundaryPoint {
                    node_id: document.Node(node).Id(),
                    offset: position.offset,
                },
                A::BeforeAnchor | A::AfterAnchor => {
                    let parent = document.Node(node).Parent()?;
                    let offset = document
                        .Node(parent)
                        .Children()
                        .iter()
                        .position(|child| *child == node)?
                        + usize::from(position.anchor == A::AfterAnchor);
                    dom::DOMBoundaryPoint {
                        node_id: document.Node(parent).Id(),
                        offset,
                    }
                }
                A::AfterChildren => dom::DOMBoundaryPoint {
                    node_id: document.Node(node).Id(),
                    offset: document.Node(node).Children().len(),
                },
            };
            document
                .ValidateRangeBoundaryPoints(point, point)
                .ok()
                .map(|_| point)
        };
        Some(DOMSelectionSnapshot {
            anchor: boundary(selection.anchor)?,
            focus: boundary(selection.focus)?,
        })
    }
    /// DomSelection::UpdateFrameSelection.  The Range remains DOM-owned; this
    /// publishes its live endpoints through the translated FrameSelection
    /// lifecycle, including typing closure, focus and selectionchange effects.
    pub fn SetDOMSelectionFromRange(&self, range: Option<dom::DOMStaticRange>) -> bool {
        use interaction_default::core::editing::set_selection_options::{
            SetSelectionBy, SetSelectionOptions,
        };
        use layoutng_assembly::{EditingPosition, EditingTextAffinity};
        let Some(frame) = self.FrameSelectionOwner() else {
            return false;
        };
        let selection = range.map(|range| layoutng_assembly::editing_state::DOMSelection {
            anchor: EditingPosition::new(range.start.node_id, range.start.offset),
            focus: EditingPosition::new(range.end.node_id, range.end.offset),
            affinity: EditingTextAffinity::Downstream,
        });
        frame.SetSelection(
            selection,
            SetSelectionOptions {
                should_close_typing: true,
                should_clear_typing_style: true,
                set_selection_by: SetSelectionBy::kSystem,
                ..Default::default()
            },
        );
        true
    }
    /// DomSelection::toString uses the same resident text iterator as copy and
    /// editing commands, rather than walking a JavaScript-side Range model.
    pub fn DOMSelectionText(&self) -> Vec<u16> {
        use super::page_frame_selection::{CompositionTextQuery as Q, CompositionTextResult as R};
        let Some(frame) = self.FrameSelectionOwner() else {
            return Vec::new();
        };
        let Some(selection) = frame.Current() else {
            return Vec::new();
        };
        match frame.CompositionText(Q::SelectedTextForClipboard {
            selection,
            include_image_alt_text: false,
        }) {
            Some(R::Text(text)) => text,
            _ => Vec::new(),
        }
    }
    /// TextControlElement::selectionStart/selectionEnd/selectionDirection.
    /// This reads the canonical cached-or-active control selection and never
    /// reconstructs a range from script or focus state.
    pub fn TextControlSelectionSnapshot(
        &self,
        document: &InteractionDocument,
        id: u64,
    ) -> Option<TextControlSelectionSnapshot> {
        if !super::page_editing_owners::IsTextControl(document, id) {
            return None;
        }
        let selection = self.editor.SelectionFor(id);
        Some(TextControlSelectionSnapshot {
            start: selection.Start(),
            end: selection.End(),
            backward: selection.anchor > selection.focus,
        })
    }
    /// TextControlElement::setSelectionRange. UTF-16 clamping and the
    /// focused-versus-cached ownership split are applied inside Interaction.
    pub fn SetTextControlSelectionRange(
        &self,
        document: &InteractionDocument,
        id: u64,
        start: usize,
        end: usize,
        backward: bool,
    ) -> bool {
        let focused = self.state.borrow().focused_node_id == Some(id);
        if !self.editor.SetControlSelectionRange(
            document,
            id,
            start,
            end,
            backward,
            focused,
        ) {
            return false;
        }
        if !focused {
            return true;
        }
        let selection = self.editor.SelectionFor(id);
        let endpoints = ReadNode(document, id, |document, _, control| {
            document
                .TextControlInnerEditorPosition(control, selection.anchor)
                .zip(document.TextControlInnerEditorPosition(control, selection.focus))
        })
        .flatten();
        let Some((anchor, focus)) = endpoints else {
            return false;
        };
        let Some(frame) = self.FrameSelectionOwner() else {
            return false;
        };
        frame.SetSelection(
            Some(layoutng_assembly::editing_state::DOMSelection {
                anchor: layoutng_assembly::EditingPosition::new(anchor.node_id, anchor.offset),
                focus: layoutng_assembly::EditingPosition::new(focus.node_id, focus.offset),
                affinity: layoutng_assembly::EditingTextAffinity::Downstream,
            }),
            interaction_default::core::editing::set_selection_options::SetSelectionOptions {
                should_close_typing: true,
                do_not_set_focus: true,
                ..Default::default()
            },
        );
        true
    }
    /// Source-owner cursor route. A missing resident binding is explicit; no
    /// compatibility fragment traversal is invoked by this entry.
    pub fn CursorAtResident<'b>(
        &self,
        document: &'b InteractionDocument,
        point: Offset,
        paint_engine: &'b paint::PaintEngine,
        fragments: PaintFragmentCursor<'b>,
        resolver: &'b dyn super::page_cursor::ResidentCursorResolver,
    ) -> Result<Option<crate::cursor::Cursor>, super::page_cursor::ResidentCursorError> {
        super::page_cursor::CursorAtResident(document, point, paint_engine, fragments, resolver)
    }
    /// Native owners shared with full translated managers and cursor reads.
    pub fn SourceInputOwners(&self) -> &super::source_input_managers::ResidentCursorOwners {
        &self.source_input
    }

    fn ResolvePointerCaptureElement(
        &self,
        document: &InteractionDocument,
        target: u64,
        require_connected: bool,
    ) -> Result<interaction_dispatch::core::dom::events::event::EventTargetRef, PointerCaptureError>
    {
        let dispatch = self
            .dispatch_state
            .borrow()
            .as_ref()
            .filter(|dispatch| dispatch.MatchesDocument(document))
            .cloned()
            .ok_or(PointerCaptureError::InvalidState)?;
        {
            let owner = document.borrow();
            let tree = owner.GetDocument();
            let index = tree
                .FindNodeById(target)
                .ok_or(PointerCaptureError::InvalidState)?;
            if tree.Node(index).Type() != dom::persistent_document::DOMNodeType::kElement
                || (require_connected && !tree.IsConnectedToDocument(index))
            {
                return Err(PointerCaptureError::InvalidState);
            }
        }
        dispatch
            .ResolveTarget(target)
            .ok_or(PointerCaptureError::InvalidState)
    }

    /// Borrow the existing document dispatch identity for a native binding
    /// such as DataTransfer::setDragImage. No second target registry is made.
    pub fn ResolveDOMEventTarget(
        &self,
        document: &InteractionDocument,
        target: u64,
    ) -> Option<interaction_dispatch::core::dom::events::event::EventTargetRef> {
        self.dispatch_state
            .borrow()
            .as_ref()
            .filter(|dispatch| dispatch.MatchesDocument(document))?
            .ResolveTarget(target)
    }

    // Source: element.cc:10091-10109; event_handler.cc:1599-1612
    pub fn SetPointerCapture(
        &self,
        document: &InteractionDocument,
        pointer_id: interaction_input::public::common::input::pointer_id::PointerId,
        target: u64,
    ) -> Result<(), PointerCaptureError> {
        if !self.source_input.pointer.IsActive(pointer_id) {
            return Err(PointerCaptureError::NoActivePointer);
        }
        let target = self.ResolvePointerCaptureElement(document, target, true)?;
        // EventHandler::SetPointerCapture is void. The manager may decline a
        // currently active pointer that is not in active-buttons state without
        // converting that into an Element-level exception.
        self.source_input
            .pointer
            .SetPointerCapture(pointer_id, target);
        Ok(())
    }

    // Source: element.cc:10113-10125; event_handler.cc:1614-1625
    pub fn ReleasePointerCapture(
        &self,
        document: &InteractionDocument,
        pointer_id: interaction_input::public::common::input::pointer_id::PointerId,
        target: u64,
    ) -> Result<(), PointerCaptureError> {
        if !self.source_input.pointer.IsActive(pointer_id) {
            return Err(PointerCaptureError::NoActivePointer);
        }
        let target = self.ResolvePointerCaptureElement(document, target, false)?;
        self.source_input
            .pointer
            .ReleasePointerCapture(pointer_id, &target);
        Ok(())
    }

    // Source: element.cc:10128-10131; event_handler.cc:1632-1639
    pub fn HasPointerCapture(
        &self,
        document: &InteractionDocument,
        pointer_id: interaction_input::public::common::input::pointer_id::PointerId,
        target: u64,
    ) -> bool {
        if !self.source_input.pointer.IsActive(pointer_id) {
            return false;
        }
        self.ResolvePointerCaptureElement(document, target, false)
            .is_ok_and(|target| {
                self.source_input
                    .pointer
                    .HasPointerCapture(pointer_id, &target)
            })
    }
    pub fn State(&self) -> InteractionState {
        *self.state.borrow()
    }
    /// Host-provided kill ring, distinct from the clipboard. Cloned scoped
    /// interactions and document lifecycle resets retain this Editor owner.
    pub fn SetKillRing(
        &self,
        ring: Rc<dyn interaction_default::core::editing::kill_ring::KillRingPlatform>,
    ) {
        self.editor.ime_typing.editor.SetKillRing(ring);
    }
    /// Execute the source Yank/YankAndSelect command in a synchronous native
    /// dispatch scope. The command dispatcher selects this typed entry.
    pub fn ExecuteYank(
        &self,
        document: &InteractionDocument,
        select_inserted: bool,
    ) -> Result<bool, super::page_event_dispatcher::EditorCommandError> {
        use interaction_default::core::editing::commands::editor_command_table::EditingCommandType;
        self.Dispatcher(document, None)
            .ExecuteEditorCommand(if select_inserted {
                EditingCommandType::YankAndSelect
            } else {
                EditingCommandType::Yank
            })
    }
    /// Document command entry shares the retained Editor and synchronous
    /// listener scope with native key/menu commands.
    pub fn DocumentEditorCommand(
        &self,
        document: &InteractionDocument,
        operation: DocumentEditorCommandOperation,
        name: &str,
        value: &str,
    ) -> Result<DocumentEditorCommandResult, super::page_event_dispatcher::EditorCommandError> {
        use interaction_default::core::editing::commands::editor_command_table::{
            EditingCommandType, EditingTriState, EditorCommandSource,
        };
        let command = EditingCommandType::FromCommandName(&name.encode_utf16().collect::<Vec<_>>());
        let dispatcher = self.Dispatcher(document, None);
        if !dispatcher.FrameSelection().IsAvailable() {
            return Ok(if operation == DocumentEditorCommandOperation::Value {
                DocumentEditorCommandResult::Text(String::new())
            } else {
                DocumentEditorCommandResult::Boolean(false)
            });
        }
        let source = EditorCommandSource::kDom;
        use DocumentEditorCommandOperation::*;
        Ok(match operation {
            Execute => {
                let Some(prepared) = self.PrepareDocumentEditorCommand(document, name)? else {
                    return Ok(DocumentEditorCommandResult::Boolean(false));
                };
                DocumentEditorCommandResult::Boolean(prepared.Execute(value)?)
            }
            Supported => DocumentEditorCommandResult::Boolean(
                dispatcher.EditorCommandSupported(command, source),
            ),
            Enabled => DocumentEditorCommandResult::Boolean(
                dispatcher.EditorCommandEnabled(command, source)?,
            ),
            State => DocumentEditorCommandResult::Boolean(
                dispatcher.EditorCommandState(command, source)? == EditingTriState::kTrue,
            ),
            Indeterm => DocumentEditorCommandResult::Boolean(
                dispatcher.EditorCommandState(command, source)? == EditingTriState::kMixed,
            ),
            Value => {
                DocumentEditorCommandResult::Text(dispatcher.EditorCommandValue(command, source)?)
            }
        })
    }
    pub fn PrepareDocumentEditorCommand(
        &self,
        document: &InteractionDocument,
        name: &str,
    ) -> Result<
        Option<PreparedDocumentEditorCommand<'a>>,
        super::page_event_dispatcher::EditorCommandError,
    > {
        use interaction_default::core::editing::commands::editor_command_table::EditingCommandType;
        let dispatcher = self.Dispatcher(document, None);
        if !dispatcher.FrameSelection().IsAvailable() {
            return Ok(None);
        }
        let Some(scope) = dispatcher
            .EditingOwners()
            .ime_typing
            .editor
            .TryExecCommandScope()
        else {
            return Ok(None);
        };
        let events = dispatcher.EditorCommandEventQueueScope();
        dispatcher.TidyUpHtmlStructure()?;
        Ok(Some(PreparedDocumentEditorCommand {
            _events: events,
            _command: scope,
            dispatcher,
            command: EditingCommandType::FromCommandName(&name.encode_utf16().collect::<Vec<_>>()),
        }))
    }
    pub(super) fn Dispatcher<'frame>(
        &self,
        document: &InteractionDocument,
        editing_geometry: Option<super::page_editing_context::PreparedEditingGeometry<'frame>>,
    ) -> EventDispatcher<'a, 'frame> {
        let emit = self.emit.clone();
        let dom: InteractionDOMMutationEmitter =
            Rc::new(move |m| emit(InteractionOutput::DocumentMutation(m.clone())));
        let submit_engine = self.clone();
        let submit_document = document.clone();
        let submit_form = Rc::new(move |result| {
            use crate::input_type::PrepareFormEntryListResult;
            let prepared = match result {
                Ok(Some(prepared)) => prepared,
                Ok(None) => return,
                Err(error) => {
                    submit_engine.EmitFormSubmissionError(error);
                    return;
                }
            };
            let token = match &prepared {
                PrepareFormEntryListResult::Ready(pending) => pending.Token(),
                PrepareFormEntryListResult::NeedsLifecycle(token) => token,
            };
            let Some(mut parameters) =
                FormSubmissionParametersFor(&submit_document, token.Form(), token.Submitter())
            else {
                return;
            };
            parameters.scheduling = token.Scheduling();
            match prepared {
                PrepareFormEntryListResult::Ready(pending) => {
                    // Append consumed the call-scoped geometry. Finish uses a
                    // new dispatcher with no Layout borrow through script.
                    submit_engine.FinishFormSubmission(&submit_document, pending, parameters);
                }
                PrepareFormEntryListResult::NeedsLifecycle(token) => {
                    (submit_engine.emit)(InteractionOutput::Effect(
                        InteractionEffect::PrepareFormSubmission(
                            super::interaction_contract::PreparedFormSubmission {
                                token,
                                parameters,
                            },
                        ),
                    ));
                }
            }
        });
        let mut dispatcher = EventDispatcher::new(
            document.clone(),
            dom,
            self.listeners.clone(),
            self.listener_interest.clone(),
            self.mouse_frame_geometry.clone(),
            self.source_input.mouse.clone(),
            self.focus_navigation.clone(),
            self.state.clone(),
            self.editor.clone(),
            self.selections.clone(),
            Some(submit_form),
            self.default_action_state.clone(),
            self.dispatch_state.clone(),
            editing_geometry,
            self.source_input.selection.clone(),
            self.clock.clone(),
            self.focus_services
                .borrow()
                .clone()
                .expect("focus services are bound for the current Document"),
        );
        dispatcher.input_method_state = Some(self.input_method_state.borrow().clone());
        dispatcher.script_invocation = self.script_invocation;
        dispatcher.event_timing = self.event_timing.borrow().clone();
        dispatcher.input_frame = Some(self.source_input.pointer_frame.clone());
        dispatcher.interaction_output = Some(self.emit.clone());
        dispatcher.editor_scroll = self.native_scroll.borrow().clone();
        dispatcher
    }

    /// Publishes native view activation through the same call-scoped event
    /// dispatcher used by input focus transactions. Chromium's Page focus
    /// controller synchronously dispatches Window/Element focus events here;
    /// the embedding must not invoke that controller without this scope.
    pub fn SetPageActive(&self, document: &InteractionDocument, active: bool) {
        let Some(navigation) = self.focus_navigation.borrow().clone() else {
            return;
        };
        let dispatcher = self.Dispatcher(document, None);
        let scope = interaction_default::core::page::focus_controller::FocusDispatchScope {
            focus_events: Some(&dispatcher),
            ..Default::default()
        };
        let controller = navigation.Controller();
        if active {
            controller.SetActive(true);
            controller.SetFocusedWithScope(true, &scope);
        } else {
            controller.SetFocusedWithScope(false, &scope);
            controller.SetActive(false);
        }
    }

    pub fn DOMEventTargets(
        &self,
        document: &InteractionDocument,
    ) -> Rc<super::dom_event::DomEventTargets> {
        crate::event_dispatcher::PageDispatchState::ForDocument(
            &self.dispatch_state,
            document.clone(),
            self.listener_interest.clone(),
        )
        .Targets()
    }
    pub(super) fn NativeDispatchState(
        &self,
        document: &InteractionDocument,
    ) -> Rc<super::page_event_dispatcher::PageDispatchState> {
        super::page_event_dispatcher::PageDispatchState::ForDocument(
            &self.dispatch_state,
            document.clone(),
            self.listener_interest.clone(),
        )
    }
    /// Constructs the no-submitter source entry list and synchronously emits
    /// formdata with the same native data owner. Missing layout/payload facts
    /// are explicit admission results; submission scheduling remains separate.
    pub fn RunPopupTask(
        &self,
        document: &InteractionDocument,
        task: super::page_event_dispatcher::pointer_dom_effects::PopupTask,
    ) -> Result<(), super::page_event_dispatcher::pointer_dom_effects::PointerDomError> {
        super::page_event_dispatcher::pointer_dom_effects::RunPopupTask(
            &self.Dispatcher(document, None),
            task,
        )
    }
    pub fn SetGestureTimerServices(
        &self,
        services: Rc<dyn super::page_event_dispatcher::gesture_input::GestureTimerServices>,
    ) {
        *self.source_input.gesture_target.timer.borrow_mut() = Some(services);
    }
    pub fn SetGestureSettings(
        &self,
        settings: crate::core::input::gesture_manager::GestureSettings,
    ) {
        self.source_input.gesture_target.settings.set(settings);
    }
    pub fn SetGestureObservers(
        &self,
        observers: super::page_event_dispatcher::gesture_input::GestureObservers,
    ) {
        *self.source_input.gesture_target.observers.borrow_mut() = observers;
    }
    /// WebFrameWidget::DragSourceEndedAt enters the same retained drag source
    /// and dispatches dragend with the original DataTransfer identity.
    /// Document animation samples the canonical AutoscrollController with
    /// fresh listener/selection ports on every rendering opportunity.
    pub fn AnimateSelectionAutoscroll(
        &self,
        document: &InteractionDocument,
        now: interaction_input::base::time::time::TimeTicks,
    ) -> Result<(), String> {
        use interaction_default::core::page::autoscroll_controller::AutoscrollType;
        if !matches!(
            self.source_input.autoscroll.autoscroll_type.get(),
            AutoscrollType::Selection | AutoscrollType::DragAndDrop
        ) {
            return Ok(());
        }
        let _emit_state = self.ExitGuard();
        let binding = self
            .pointer_binding
            .borrow()
            .clone()
            .ok_or("AutoscrollController missing native geometry")?;
        binding
            .geometry
            .PrepareLifecycleExceptPaint()
            .map_err(|e| format!("{e:?}"))?;
        super::page_event_dispatcher::gesture_mouse_defaults::PageGestureMouseDefaults::new(
            &self.Dispatcher(document, None),
            &self.source_input,
            binding.geometry,
            binding.controls,
        )
        .AnimateSelectionAutoscroll(now)
        .map_err(|e| e.to_string())
    }
    pub fn DirectoryFilesChosen(
        &self,
        document: &InteractionDocument,
        node: u64,
        files: Result<Vec<std::sync::Arc<fileapi::File>>, fileapi::NativeFileError>,
    ) {
        let _emit_state = self.ExitGuard();
        super::page_event_dispatcher::mouse_drag::DirectoryFilesChosen(
            &self.Dispatcher(document, None),
            node,
            files,
        );
    }
    pub fn DragCaretSelection(&self) -> Option<layoutng_assembly::editing_state::DOMSelection> {
        super::page_event_dispatcher::mouse_drag::CurrentDragCaret(&self.source_input)
    }
    pub fn HandleTargetDrag(
        &self,
        document: &InteractionDocument,
        event: &interaction_input::public::common::input::web_mouse_event::WebMouseEvent,
        action: &interaction_default::core::page::drag_controller::DragTargetAction,
    ) -> Result<interaction_default::core::page::drag_controller::DragTargetOperation, String> {
        let _emit_state = self.ExitGuard();
        let binding = self
            .pointer_binding
            .borrow()
            .clone()
            .ok_or("drag target missing native geometry")?;
        binding
            .geometry
            .PrepareLifecycleExceptPaint()
            .map_err(|e| format!("{e:?}"))?;
        let scroll = self
            .native_scroll
            .borrow()
            .as_ref()
            .map(|scroll| scroll.ScrollManager());
        super::page_event_dispatcher::mouse_drag::TargetDrag(
            &self.Dispatcher(document, None),
            &self.source_input,
            binding.geometry,
            binding.controls,
            scroll,
            event,
            action,
        )
        .map_err(|error| error.to_string())
    }
    pub fn DragSourceEndedAt(
        &self,
        document: &InteractionDocument,
        event: &interaction_input::public::common::input::web_mouse_event::WebMouseEvent,
        operation: interaction_dispatch::core::clipboard::data_transfer::DragOperation,
    ) {
        let _emit_state = self.ExitGuard();
        let Some(binding) = self.pointer_binding.borrow().clone() else {
            return;
        };
        if let Err(error) = super::page_event_dispatcher::mouse_drag::EndDrag(
            &self.Dispatcher(document, None),
            &self.source_input,
            binding.geometry,
            binding.controls,
            event,
            operation,
        ) {
            *self.pointer_input_error.borrow_mut() = Some(error);
        }
    }
    pub fn RunGestureTimerTask(
        &self,
        document: &InteractionDocument,
        task: super::page_event_dispatcher::gesture_input::GestureTimerTask,
    ) -> Result<(), String> {
        let _emit_state = self.ExitGuard();
        super::page_event_dispatcher::gesture_input::RunGestureTimerTask(
            &self.Dispatcher(document, None),
            &self.source_input,
            task,
        )
    }

    pub fn SetPopoverOpen(
        &self,
        document: &InteractionDocument,
        node: u64,
        open: bool,
        source: Option<u64>,
    ) -> Result<(), super::page_event_dispatcher::pointer_dom_effects::PointerDomError> {
        super::page_event_dispatcher::pointer_dom_effects::SetPopoverOpen(
            &self.Dispatcher(document, None),
            node,
            open,
            source,
        )
    }

    pub fn RequestDialogClose(
        &self,
        document: &InteractionDocument,
        node: u64,
    ) -> Result<(), super::page_event_dispatcher::pointer_dom_effects::PointerDomError> {
        super::page_event_dispatcher::pointer_dom_effects::RequestDialogClose(
            &self.Dispatcher(document, None),
            node,
        )
    }
    pub fn CreateScriptCloseWatcher(
        &self,
        document: &InteractionDocument,
        host_id: u64,
        target: Rc<interaction_dispatch::core::dom::events::event_target::EventTarget>,
        aborted: bool,
    ) -> Result<
        Rc<interaction_default::core::html::closewatcher::close_watcher::NativeCloseWatcher>,
        super::page_event_dispatcher::pointer_dom_effects::PointerDomError,
    > {
        super::page_event_dispatcher::pointer_dom_effects::CreateScriptCloseWatcher(
            &self.Dispatcher(document, None),
            host_id,
            target,
            aborted,
        )
    }
    pub fn InvokeScriptCloseWatcher(
        &self,
        document: &InteractionDocument,
        watcher: foundation::popup_types::CloseWatcherId,
        request: bool,
    ) -> Result<(), super::page_event_dispatcher::pointer_dom_effects::PointerDomError> {
        super::page_event_dispatcher::pointer_dom_effects::InvokeScriptCloseWatcher(
            &self.Dispatcher(document, None),
            watcher,
            request,
        )
    }
    pub fn DestroyScriptCloseWatcher(
        &self,
        document: &InteractionDocument,
        watcher: foundation::popup_types::CloseWatcherId,
    ) {
        super::page_event_dispatcher::pointer_dom_effects::DestroyScriptCloseWatcher(
            &self.Dispatcher(document, None),
            watcher,
        );
    }
    pub fn SignalCloseWatchers(
        &self,
        document: &InteractionDocument,
    ) -> Result<(), super::page_event_dispatcher::pointer_dom_effects::PointerDomError> {
        super::page_event_dispatcher::pointer_dom_effects::SignalCloseWatchers(
            &self.Dispatcher(document, None),
        )
    }

    pub fn ApplyPopupMutation(
        &self,
        document: &InteractionDocument,
        mutation: &dom::dom_mutation::DOMMutation,
    ) -> Result<bool, super::page_event_dispatcher::pointer_dom_effects::PointerDomError> {
        super::page_event_dispatcher::pointer_dom_effects::ApplyPopupMutation(
            &self.Dispatcher(document, None),
            mutation,
        )
    }

    pub fn DrainPopupLifecycle(
        &self,
        document: &InteractionDocument,
    ) -> Result<(), super::page_event_dispatcher::pointer_dom_effects::PointerDomError> {
        super::page_event_dispatcher::pointer_dom_effects::DrainPopupLifecycle(
            &self.Dispatcher(document, None),
        )
    }

    pub(super) fn SaveFormControlState(
        &self,
        document: &InteractionDocument,
        node: u64,
    ) -> Vec<Rc<[u16]>> {
        let dispatcher = self.Dispatcher(document, None);
        let result = super::page_input_type::SaveFormControlState(&dispatcher.Context(), node);
        result
    }
    pub(super) fn RestoreFormControlState(
        &self,
        document: &InteractionDocument,
        node: u64,
        state: &[Rc<[u16]>],
        reader: Option<std::sync::Arc<dyn fileapi::NativeFileReader>>,
    ) -> Result<(), navigation_history::FormStateRestoreError> {
        let dispatcher = self.Dispatcher(document, None);
        let result = super::page_input_type::RestoreFormControlState(
            &dispatcher.Context(),
            node,
            state,
            reader,
        );
        result
    }
    pub(super) fn BeginFormStateEventQueueScope(
        &self,
        document: &InteractionDocument,
    ) -> Box<dyn interaction_default::core::html::forms::input_type::EventQueueScope + 'a> {
        self.Dispatcher(document, None).FormStateEventQueueScope()
    }

    pub fn DocumentAutofocusDelegate(&self, document: &InteractionDocument) -> Option<u64> {
        super::page_event_dispatcher::pointer_dom_effects::DocumentAutofocusDelegate(
            &self.Dispatcher(document, None),
        )
    }
    /// Starts FormData(form, submitter) with the current call's native
    /// listener scope. Lifecycle continuation is a value returned to Page;
    /// this method never invokes a lifecycle host or schedules navigation.
    pub fn CreateFormData(
        &self,
        document: &InteractionDocument,
        form: u64,
        submitter: Option<u64>,
    ) -> Result<
        FormDataCreation,
        interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError,
    > {
        let prepared = {
            let dispatcher = self.Dispatcher(document, None);
            let result =
                super::page_input_type::PrepareFormData(&dispatcher.Context(), form, submitter)?;
            result
        };
        match prepared {
            crate::input_type::PrepareFormEntryListResult::Ready(pending) => self
                .FinishFormData(document, pending)
                .map(FormDataCreation::Ready),
            crate::input_type::PrepareFormEntryListResult::NeedsLifecycle(token) => {
                Ok(FormDataCreation::NeedsLifecycle(token))
            }
        }
    }
    /// The caller releases its resident Layout borrow before entering this
    /// synchronous formdata dispatch. The constructor result has a separate
    /// mutable data owner from the object exposed by the event.
    pub fn FinishFormData(
        &self,
        document: &InteractionDocument,
        pending: crate::input_type::PendingFormEntryList,
    ) -> Result<
        Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>,
        interaction_dispatch::core::fileapi::form_data_object::FormDataConstructionError,
    > {
        let entries = self
            .Dispatcher(document, None)
            .FinishEntryList(pending)
            .map_err(crate::input_type::FormDataConstructionErrorFor)?;
        Ok(
            interaction_dispatch::core::fileapi::form_data_object::FormDataObject::new(
                Rc::unwrap_or_clone(entries),
            ),
        )
    }
    pub fn ConstructFormEntryList(
        &self,
        document: &InteractionDocument,
        form: u64,
        encoding: &str,
    ) -> Result<
        Option<Rc<interaction_dispatch::core::fileapi::form_data_object::FormDataObject>>,
        crate::input_type::FormDataAdmission,
    > {
        self.Dispatcher(document, None)
            .ConstructFormEntryList(form, encoding)
    }
    /// Resume only entry construction against the embedding's current
    /// resident output. The returned owner contains no fragment lifetime.
    pub fn ResumeFormSubmission(
        &self,
        document: &InteractionDocument,
        token: crate::input_type::FormSubmissionToken,
        geometry: super::page_editing_context::PreparedEditingGeometry<'_>,
    ) -> Result<
        Option<crate::input_type::PrepareFormEntryListResult>,
        crate::input_type::FormDataAdmission,
    > {
        self.Dispatcher(document, Some(geometry))
            .ResumeFormEntryList(token)
    }
    /// Runs after the caller releases its resident Layout borrow, in the
    /// original listener transaction. The same FormData object is exposed to
    /// listeners, then frozen by the owning form implementation.
    pub fn FinishFormSubmission(
        &self,
        document: &InteractionDocument,
        pending: crate::input_type::PendingFormEntryList,
        parameters: super::interaction_contract::FormSubmissionParameters,
    ) {
        let _submission_scope = pending.Token().SubmissionScope();
        let dispatcher = self.Dispatcher(document, None);
        match dispatcher.FinishEntryList(pending) {
            Ok(entries) => {
                if parameters
                    .scheduling
                    .as_ref()
                    .is_some_and(|scheduling| !scheduling.task.IsCurrent())
                {
                    return;
                }
                let preparation = self.form_navigation_preparation.borrow().clone();
                if preparation.as_ref().is_some_and(|preparation| {
                    !preparation.Prepare(&parameters.target.clone().into(), &dispatcher)
                }) {
                    return;
                }
                (self.emit)(InteractionOutput::Effect(InteractionEffect::SubmitForm {
                    parameters,
                    entries,
                }));
            }
            Err(error) => self.EmitFormSubmissionError(error),
        }
    }
    pub fn SetFormNavigationPreparation(
        &self,
        preparation: Option<Rc<dyn super::interaction_contract::FormNavigationPreparation>>,
    ) {
        *self.form_navigation_preparation.borrow_mut() = preparation;
    }

    fn EmitFormSubmissionError(&self, error: crate::input_type::FormDataAdmission) {
        (self.emit)(InteractionOutput::Effect(
            InteractionEffect::FormSubmissionError(error),
        ));
    }
    pub fn RefreshSelectTextStyleAndContent(&self, document: &InteractionDocument, select: u64) {
        super::page_input_type::RefreshSelectTextStyleAndContent(
            &self.Dispatcher(document, None).Context(),
            select,
        );
    }
    pub fn SelectFinishedParsingChildren(&self, document: &InteractionDocument, select: u64) {
        super::page_input_type::SelectFinishedParsingChildren(
            &self.Dispatcher(document, None).Context(),
            select,
        );
    }
    pub fn FormControlState(&self) -> Rc<crate::input_type::DefaultActionState> {
        self.default_action_state.clone()
    }
    pub fn CheckValidity(
        &self,
        document: &InteractionDocument,
        node: u64,
        report: bool,
    ) -> Result<bool, super::page_form_validation::ValidationRequired> {
        let dispatcher = self.Dispatcher(document, None);
        let context = dispatcher.Context();
        if ReadNode(document, node, |_, n, _| n.IsHTMLElement("form")).unwrap_or(false) {
            super::page_form_validation::CheckFormValidity(&context, node, report)
        } else {
            super::page_form_validation::CheckControlValidity(&context, node, report)
        }
    }
    pub fn SubmitForm(
        &self,
        document: &InteractionDocument,
        form: u64,
        submitter: Option<u64>,
        dispatch_event: bool,
    ) -> Result<(), crate::input_type::RequestSubmitError> {
        self.Dispatcher(document, None)
            .SubmitForm(form, submitter, dispatch_event)
    }
    pub(super) fn ExitGuard(&self) -> EmitStateMutationAtExit {
        EmitStateMutationAtExit {
            before: self.State(),
            current: self.state.clone(),
            emit: self.emit.clone(),
        }
    }
    // cpp: interaction/interaction_engine.cc:147-234
    /// Raw touch/pointer ingress requires the native geometry owner before any
    /// hit test or listener dispatch; mouse state is never substituted.
    pub fn ValidateInput(
        &self,
        input: &InputEvent,
    ) -> Result<(), super::native_interaction::InteractionError> {
        if matches!(
            input,
            InputEvent::Touch(_) | InputEvent::Pointer(_) | InputEvent::Gesture(_)
        ) && self.pointer_binding.borrow().is_none()
        {
            return Err(super::native_interaction::InteractionError::SourceInput(
                "native pointer geometry is not bound".into(),
            ));
        }
        Ok(())
    }
    fn CompleteInput(
        &self,
        result: InteractionResult,
    ) -> Result<InteractionResult, super::native_interaction::InteractionError> {
        if let Some(error) = self.pointer_input_error.borrow_mut().take() {
            return Err(super::native_interaction::InteractionError::SourceInput(
                error.to_string(),
            ));
        }
        Ok(result)
    }
    /// Script-triggered focus uses the same canonical FocusController entry as
    /// native focus input. It needs no hit-test snapshot and retains no former
    /// facade input transaction.
    pub fn DispatchTargeted(
        &self,
        input: &InputEvent,
        document: &InteractionDocument,
    ) -> InteractionResult {
        let InputEvent::Focus(event) = input else {
            panic!("only explicit focus events use DispatchTargeted")
        };
        assert!(ExplicitTarget(input).is_some());
        let _emit_state = self.ExitGuard();
        let result = self.Dispatcher(document, None).DispatchHostFocus(event);
        InteractionResult {
            target_node_id: result.target_node_id,
            default_prevented: result.default_prevented,
            propagation_stopped: result.propagation_stopped,
            input_result: None,
        }
    }

    // cpp: interaction/interaction_engine.cc:237-244
    pub fn DispatchDOMEvent(
        &self,
        event: &mut Event,
        target: u64,
        document: &InteractionDocument,
    ) -> InteractionResult {
        let _emit_state = self.ExitGuard();
        let result = self.Dispatcher(document, None).Dispatch(event, target);
        InteractionResult {
            target_node_id: result.target_node_id,
            default_prevented: result.default_prevented,
            propagation_stopped: result.propagation_stopped,
            input_result: None,
        }
    }
}

// FormSubmission::Create copies form/submitter attributes before ConstructEntryList
// dispatches formdata. Capture only navigation values; controls remain live.
fn FormSubmissionParametersFor(
    owner: &InteractionDocument,
    form: u64,
    submitter: Option<u64>,
) -> Option<super::interaction_contract::FormSubmissionParameters> {
    use interaction_default::core::loader::form_submission::FormSubmissionAttributes;
    let owner = owner.borrow();
    let document = owner.GetDocument();
    let form_index = document.FindNodeById(form)?;
    let submitter_index = submitter.and_then(|id| document.FindNodeById(id));
    let attribute = |index: usize, name: &str| {
        document
            .Node(index)
            .FindAttribute(name)
            .map(|a| a.value.clone())
    };
    let value = |name: &str, override_name: &str| {
        submitter_index
            .and_then(|index| attribute(index, override_name))
            .or_else(|| attribute(form_index, name))
            .unwrap_or_default()
    };
    let base = (0..document.NodeCount()).find(|&index| document.Node(index).IsHTMLElement("base"));
    let mut target = value("target", "formtarget");
    if target.is_empty() {
        target = base
            .and_then(|index| attribute(index, "target"))
            .unwrap_or_default();
    }
    Some(super::interaction_contract::FormSubmissionParameters {
        scheduling: None,
        form,
        submitter,
        action: value("action", "formaction")
            .trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c'))
            .into(),
        base_href: base.and_then(|index| attribute(index, "href")),
        target,
        method: FormSubmissionAttributes::ParseMethod(&value("method", "formmethod")),
        enctype: FormSubmissionAttributes::ParseEncodingType(&value("enctype", "formenctype"))
            .into(),
    })
}

struct FocusDocumentEvents {
    engine: Interaction<'static>,
    document: InteractionDocument,
}
impl super::page_focus_navigation::FocusDocumentEventTarget for FocusDocumentEvents {
    fn FocusEvents(
        &self,
    ) -> Rc<dyn interaction_default::core::page::focus_controller::FocusEventDispatch> {
        Rc::new(self.engine.Dispatcher(&self.document, None))
    }
    fn Dispatch(
        &self,
        event: &mut crate::event::Event,
        target: u64,
        listeners: Option<crate::event::EventListenerDispatcher<'_>>,
    ) {
        if let Some(listeners) = listeners {
            self.engine.WithScopedListeners(listeners).DispatchDOMEvent(
                event,
                target,
                &self.document,
            );
        } else {
            self.engine.DispatchDOMEvent(event, target, &self.document);
        }
    }
}

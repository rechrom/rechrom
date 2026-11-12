//! Retained Page facade. Convenience operations lend the existing source stage
//! owners; raw input binds the translated EventHandler for one synchronous
//! transaction, including nested listener dispatch.
use std::{cell::RefCell, ops::Deref, rc::Rc};

use super::{
    event::EventListenerDispatcher,
    frame_aligned_input_queue::{BorrowedInputPacket, OwnedInputPacket},
    input_event::InputEvent,
    interaction as stages,
    interaction_contract::{
        InteractionDOMMutationEmitter, InteractionDocument, InteractionOutputEmitter,
        InteractionResult,
    },
    native_interaction::InteractionError,
    page_event_handler_binding::{PageEventHandlerEmbedding, PageEventHandlerSource},
};

#[derive(Clone)]
pub struct Interaction<'listener> {
    stages: stages::Interaction<'listener>,
    source: Rc<RefCell<Option<Rc<PageEventHandlerSource>>>>,
    widget_keyboard: Rc<crate::core::frame::web_frame_widget_impl::WidgetKeyboardState>,
}

impl<'listener> Deref for Interaction<'listener> {
    type Target = stages::Interaction<'listener>;
    fn deref(&self) -> &Self::Target {
        &self.stages
    }
}

impl<'listener> Interaction<'listener> {
    pub fn new(
        output: InteractionOutputEmitter,
        listeners: Option<EventListenerDispatcher<'listener>>,
        clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    ) -> Self {
        Self::WithSelectionState(output, listeners, Rc::new(Default::default()), clock)
    }
    pub fn WithSelectionState(
        output: InteractionOutputEmitter,
        listeners: Option<EventListenerDispatcher<'listener>>,
        selections: Rc<layoutng_assembly::editing_state::SelectionState>,
        clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    ) -> Self {
        Self {
            stages: stages::Interaction::WithSelectionState(output, listeners, selections, clock),
            source: Default::default(),
            widget_keyboard: Default::default(),
        }
    }
    pub fn WithDOMMutationEmitter(
        output: InteractionDOMMutationEmitter,
        listeners: Option<EventListenerDispatcher<'listener>>,
        clock: Rc<dyn interaction_dispatch::core::dom::events::event::EventClock>,
    ) -> Self {
        Self {
            stages: stages::Interaction::WithDOMMutationEmitter(output, listeners, clock),
            source: Default::default(),
            widget_keyboard: Default::default(),
        }
    }
    pub fn WithScopedListeners<'scope>(
        &self,
        listeners: EventListenerDispatcher<'scope>,
    ) -> Interaction<'scope>
    where
        'listener: 'scope,
    {
        Interaction {
            stages: self.stages.WithScopedListeners(listeners),
            source: self.source.clone(),
            widget_keyboard: self.widget_keyboard.clone(),
        }
    }
    pub fn WithScriptInvocation<'scope>(
        &'scope self,
        invocation: &'scope dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation,
    ) -> Interaction<'scope>
    where
        'listener: 'scope,
    {
        Interaction {
            stages: self.stages.WithScriptInvocation(invocation),
            source: self.source.clone(),
            widget_keyboard: self.widget_keyboard.clone(),
        }
    }
    pub fn NativeFrameSource(&self) -> Option<Rc<PageEventHandlerSource>> {
        self.source.borrow().clone()
    }
    pub fn BindDocumentInputLifecycle(&self, document: &InteractionDocument) {
        self.stages.BindDocumentInputLifecycle(document);
        let source = self.source.borrow().clone();
        if let Some(source) = source {
            source.nodes.borrow_mut().take();
            let timers = source.embedding.Timers();
            use super::page_event_handler_services::EventHandlerTimerKind;
            for kind in [
                EventHandlerTimerKind::Hover,
                EventHandlerTimerKind::CursorUpdate,
                EventHandlerTimerKind::ActiveInterval,
            ] {
                timers.Stop(kind);
            }
        }
    }

    pub fn TryDispatch<'frame>(
        &self,
        input: &InputEvent,
        document: &InteractionDocument,
        fragments: impl Into<stages::InteractionFragmentSource<'frame>>,
    ) -> Result<InteractionResult, InteractionError> {
        let fragments = fragments.into();
        let source = self.source.borrow().clone().ok_or_else(|| {
            InteractionError::SourceInput("EventHandler frame embedding is not bound".into())
        })?;
        if matches!(input, InputEvent::Composition(_) | InputEvent::TextInput(_)) {
            let identity = source
                .embedding
                .FrameDomain()
                .FocusedLocalFrameInWidgetIdentity();
            let Some(identity) = identity else {
                return Ok(Default::default());
            };
            let focused = if Rc::ptr_eq(&identity, &source.embedding.FrameDomain().Identity()) {
                source
            } else {
                source
                    .embedding
                    .NativeFrameSource(&identity)
                    .ok_or(InteractionError::DetachedFrame)?
            };
            let listeners = self.stages.listeners.borrow().clone();
            return super::page_event_handler_binding::DispatchPageInput(
                focused,
                listeners,
                input,
                fragments,
                None,
                self.stages.ScriptInvocation(),
            );
        }
        if matches!(input, InputEvent::Focus(_)) {
            let listeners = self.stages.listeners.borrow().clone();
            return super::page_event_handler_binding::DispatchPageInput(
                source,
                listeners,
                input,
                fragments,
                None,
                self.stages.ScriptInvocation(),
            );
        }
        let scale = source.embedding.InputWidgetScale();
        let packet = OwnedInputPacket::FromInput(
            input,
            source.embedding.Host().InputTimestamp(input)?,
            scale,
        )?;
        self.TryDispatchPacket(input, document, fragments, packet.Borrow())
    }
    pub fn TryDispatchPacket<'frame>(
        &self,
        input: &InputEvent,
        document: &InteractionDocument,
        fragments: impl Into<stages::InteractionFragmentSource<'frame>>,
        packet: BorrowedInputPacket<'_>,
    ) -> Result<InteractionResult, InteractionError> {
        if matches!(
            input,
            InputEvent::Focus(_) | InputEvent::Composition(_) | InputEvent::TextInput(_)
        ) {
            return self.TryDispatch(input, document, fragments);
        }
        let source = self.source.borrow().clone().ok_or_else(|| {
            InteractionError::SourceInput("EventHandler frame embedding is not bound".into())
        })?;
        if !Rc::ptr_eq(document, &source.document) {
            return Err(InteractionError::DetachedFrame);
        }
        let _handling = self
            .stages
            .source_input
            .pointer_frame
            .EnterHandlingInputEvent();
        let listeners = self.stages.listeners.borrow().clone();
        super::page_event_handler_binding::DispatchPageInput(
            source,
            listeners,
            input,
            fragments.into(),
            Some(packet),
            self.stages.ScriptInvocation(),
        )
    }
    pub fn RecomputeMouseHoverState<'frame>(
        &self,
        document: &InteractionDocument,
        fragments: impl Into<stages::InteractionFragmentSource<'frame>>,
    ) -> Result<(), InteractionError> {
        let source = self.source.borrow().clone().ok_or_else(|| {
            InteractionError::SourceInput("EventHandler frame embedding is not bound".into())
        })?;
        if !Rc::ptr_eq(document, &source.document) {
            return Err(InteractionError::DetachedFrame);
        }
        super::page_event_handler_binding::RecomputePageMouseHoverState(
            source,
            self.stages.listeners.borrow().clone(),
            fragments.into(),
            self.stages.ScriptInvocation(),
        )
    }
    pub fn RunEventHandlerTimer(
        &self,
        task: &crate::core::input::event_handler::EventHandlerTimerTask,
    ) -> Result<(), InteractionError> {
        let source = self.source.borrow().clone().ok_or_else(|| {
            InteractionError::SourceInput("EventHandler frame embedding is not bound".into())
        })?;
        let listeners = self.stages.listeners.borrow().clone();
        super::page_event_handler_binding::RunPageEventHandlerTimer(source, listeners, task)
    }
}

impl Interaction<'static> {
    pub fn SetEventHandlerEmbedding(
        &self,
        document: &InteractionDocument,
        embedding: Rc<dyn PageEventHandlerEmbedding>,
    ) {
        *self.source.borrow_mut() = Some(Rc::new(PageEventHandlerSource {
            interaction: self.stages.clone(),
            document: document.clone(),
            embedding,
            nodes: Default::default(),
            widget_keyboard: self.widget_keyboard.clone(),
        }));
    }
}

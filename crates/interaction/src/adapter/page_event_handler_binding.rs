//! Typed construction of a synchronous EventHandler binding. The embedding
//! supplies frame/client leaves; the retained managers and their ordering stay
//! inside Interaction. No borrowed dispatch scope is retained by the host.
use std::{cell::RefCell, rc::Rc};

use super::{
    native_interaction::InteractionHost,
    page_event_handler_frame::{PageEventHandlerFrameDomain, PageEventHandlerFrameInputClient},
    page_event_handler_hit_test::{PageEventHandlerCoordinates, PageEventHandlerHitTestLifecycle},
    page_event_handler_node::{PageEventHandlerAtomicMoveState, PageEventHandlerNodeGeometry},
    page_event_handler_services::PageEventHandlerTimerServices,
};

/// Durable scalar and geometry connections to the embedding's live frame.
/// The per-call Domain view resolves only bindings in this input transaction;
/// it must not keep a scoped EventHandler after the transaction returns.
pub trait PageEventHandlerEmbedding {
    fn NodeGeometry(&self) -> Rc<dyn PageEventHandlerNodeGeometry>;
    fn AtomicMoveState(&self) -> Rc<dyn PageEventHandlerAtomicMoveState>;
    fn HitTestLifecycle(&self) -> Rc<dyn PageEventHandlerHitTestLifecycle>;
    fn Coordinates(&self) -> Rc<dyn PageEventHandlerCoordinates>;
    fn FrameDomain(&self) -> Rc<dyn PageEventHandlerFrameDomain>;
    fn FrameInputClient(&self) -> Rc<dyn PageEventHandlerFrameInputClient>;
    fn ChromeClient(&self) -> Rc<dyn super::page_event_dispatcher::event_handler_frame_services::PageEventHandlerChromeClient>;
    fn CursorClient(&self) -> Rc<dyn super::page_event_dispatcher::event_handler_frame_services::PageEventHandlerCursorClient>;
    fn MiddleClickAutoscrollFrame(
        &self,
    ) -> Option<Rc<dyn interaction_default::core::page::autoscroll_controller::AutoscrollFrame>>;
    fn Host(&self) -> Rc<dyn InteractionHost>;
    fn Timers(&self) -> Rc<dyn PageEventHandlerTimerServices>;
    fn InputWidgetScale(&self) -> f32;
    fn PageActive(&self) -> bool;
    fn CursorVisible(&self) -> Option<bool>;
    fn NativeFrameSource(&self, identity: &Rc<()>) -> Option<Rc<PageEventHandlerSource>>;
}

/// An opaque retained source handle. It carries only the already existing
/// Interaction/Document owners and frame leaves; a binding lends the current
/// listener scope rather than retaining a runtime or a Layout loan here.
pub struct PageEventHandlerSource {
    pub(super) interaction: super::interaction::Interaction<'static>,
    pub(super) document: crate::InteractionDocument,
    pub(super) embedding: Rc<dyn PageEventHandlerEmbedding>,
    pub(super) nodes: RefCell<Option<Rc<super::page_event_handler_node::PageEventHandlerNodes>>>,
    pub(super) widget_keyboard: Rc<crate::core::frame::web_frame_widget_impl::WidgetKeyboardState>,
}

use super::{
    event::EventListenerDispatcher,
    frame_aligned_input_queue::BorrowedInputPacket,
    input_event::{
        CompositionEvent, ConvertedWebInputEvent, FocusEvent, InputEvent, TextInputEvent,
        WheelEvent,
    },
    interaction::InteractionFragmentSource,
    interaction_contract::{InteractionOutputEmitter, InteractionResult},
    native_interaction::{
        Interaction as NativeInteraction, InteractionAssembly, InteractionError,
        InteractionFocusInput, InteractionInputCompletion, InteractionInputMethod,
        InteractionInputResult, InteractionWheelInput, InteractionWidgetKeyboardInput,
    },
    page_event_dispatcher::{
        event_handler_frame_services::{
            PageEventHandlerChrome, PageEventHandlerDocumentState, PageEventHandlerInput,
        },
        DispatchEventResult, EventDispatcher,
    },
    page_event_handler_factory::{PageEventHandlerFactory, PageEventHandlerManagers},
    page_event_handler_frame::{
        PageEventHandlerFrame, PageEventHandlerFrameRegistry, PageEventHandlerFrameServices,
    },
    page_event_handler_hit_test::{PageEventHandlerHitResult, PageEventHandlerHitTest},
    page_event_handler_node::{PageEventHandlerNodes, RequiredEventHandlerPort},
    page_event_handler_services::{
        EventHandlerTimerKind, PageEventHandlerResizeCoordinates, PageEventHandlerScroll,
        PageEventHandlerTimer, PageEventHandlerWheel,
    },
    page_keyboard_event_manager::PageEventHandlerKeyboard,
    page_scroll_manager::PageScrollManagerOwners,
};
use crate::core::frame::web_frame_widget_impl::{
    FocusedPluginKeyboardState, WidgetKeyboardContext, WidgetKeyboardState,
};
use crate::core::input::event_handler::{EventHandler, EventHandlerFrame, EventHandlerTimerTask};
use interaction_dispatch::public::platform::web_input_event_result::WebInputEventResult;
use interaction_input::public::common::input::web_mouse_wheel_event::WebMouseWheelEvent;
use interaction_input::public::common::input::{
    web_input_event::Type, web_keyboard_event::WebKeyboardEvent,
};
use std::{cell::Cell, collections::HashMap, rc::Weak};

struct NativeFocusBinding<'listener, 'frame>(EventDispatcher<'listener, 'frame>);
impl InteractionFocusInput for NativeFocusBinding<'_, '_> {
    fn HandleFocus(
        &self,
        event: &FocusEvent,
        _output: &InteractionOutputEmitter,
    ) -> Result<InteractionResult, InteractionError> {
        let result = self.0.DispatchHostFocus(event);
        Ok(InteractionResult {
            target_node_id: result.target_node_id,
            default_prevented: result.default_prevented,
            propagation_stopped: result.propagation_stopped,
            input_result: None,
        })
    }

    fn HandleFocusWithLocalEvents(
        &self,
        event: &FocusEvent,
        output: &InteractionOutputEmitter,
        _local_events: &dyn interaction_dispatch::core::dom::events::event_path::LocalEventDispatch,
    ) -> Result<InteractionResult, InteractionError> {
        self.HandleFocus(event, output)
    }
}

struct NativeInputMethodBinding<'listener, 'frame>(EventDispatcher<'listener, 'frame>);
impl InteractionInputMethod for NativeInputMethodBinding<'_, '_> {
    fn HandleComposition(
        &self,
        event: &CompositionEvent,
        _output: &InteractionOutputEmitter,
    ) -> Result<InteractionInputResult, InteractionError> {
        let result = self.0.DispatchHostComposition(event);
        Ok(InteractionInputResult::DOM(InteractionResult {
            target_node_id: result.target_node_id,
            default_prevented: result.default_prevented,
            propagation_stopped: result.propagation_stopped,
            input_result: None,
        }))
    }

    fn HandleTextInput(
        &self,
        event: &TextInputEvent,
        _output: &InteractionOutputEmitter,
    ) -> Result<InteractionInputResult, InteractionError> {
        let result = self.0.DispatchHostTextInput(event);
        Ok(InteractionInputResult::DOM(InteractionResult {
            target_node_id: result.target_node_id,
            default_prevented: result.default_prevented,
            propagation_stopped: result.propagation_stopped,
            input_result: None,
        }))
    }
}

struct WidgetKeyboardBinding<'scope> {
    registry: Weak<BindingRegistry<'scope>>,
    domain: Rc<dyn PageEventHandlerFrameDomain>,
    host: Rc<dyn InteractionHost>,
    state: Rc<WidgetKeyboardState>,
}
impl InteractionWidgetKeyboardInput for WidgetKeyboardBinding<'_> {
    fn HandleKeyboardEvent(
        &self,
        event: &WebKeyboardEvent,
    ) -> Result<WebInputEventResult, InteractionError> {
        if event.GetType() == Type::kChar {
            self.state.HandleCharEvent(self, event)
        } else {
            self.state.HandleKeyEvent(self, event)
        }
    }
}
impl<'scope> WidgetKeyboardContext for WidgetKeyboardBinding<'scope> {
    type Frame = Rc<BoundFrame<'scope>>;
    type Error = InteractionError;
    fn HandlePopupKeyEvent(
        &self,
        event: &WebKeyboardEvent,
    ) -> Result<Option<WebInputEventResult>, Self::Error> {
        self.host.HandlePopupKeyEvent(event)
    }
    fn PagePopupCopyPasteEnabled(&self) -> bool {
        self.host.PagePopupCopyPasteEnabled()
    }
    fn FocusedCoreFrame(&self) -> Result<Option<Self::Frame>, Self::Error> {
        self.domain
            .FocusedCoreFrameIdentity()
            .map(|identity| {
                self.registry
                    .upgrade()
                    .ok_or(InteractionError::DetachedFrame)?
                    .Bind(&identity)
            })
            .transpose()
    }
    fn HasLocalRoot(&self) -> bool {
        self.host.HasLocalRoot()
    }
    fn KeyEvent(&self, frame: &Self::Frame, event: &WebKeyboardEvent) -> WebInputEventResult {
        frame.handler.KeyEvent(event)
    }
    fn HandleAccessKey(&self, frame: &Self::Frame, event: &WebKeyboardEvent) -> bool {
        frame.handler.HandleAccessKey(event)
    }
    fn FocusedPluginKeyboardState(&self) -> Result<FocusedPluginKeyboardState, Self::Error> {
        self.host.FocusedPluginKeyboardState()
    }
    fn SendKeyboardContextMenu(&self) -> Result<(), Self::Error> {
        let Some(frame) = self.FocusedCoreFrame()? else {
            return Ok(());
        };
        frame
            .completion
            .managers
            .ShowKeyboardContextMenu(&frame.handler)
            .map_err(|error| InteractionError::SourceInput(error.to_string()))
    }
}

impl From<RequiredEventHandlerPort> for InteractionError {
    fn from(port: RequiredEventHandlerPort) -> Self {
        Self::SourceInput(format!("EventHandler required port: {port:?}"))
    }
}

struct WheelInput<'scope> {
    dispatcher: EventDispatcher<'scope, 'scope>,
    routing: Rc<interaction_scroll::core::input::event_handler::EventHandlerScrollRouting>,
    fragments: Option<InteractionFragmentSource<'scope>>,
}
impl InteractionWheelInput for WheelInput<'_> {
    fn HandleWheelEvent(
        &self,
        event: &WebMouseWheelEvent,
        _: &WheelEvent,
        _: &InteractionOutputEmitter,
    ) -> Result<WebInputEventResult, InteractionError> {
        let (_, input) = self
            .dispatcher
            .DispatchNativeWheelEventWithResult(
                self.routing.GetMouseWheelEventManager(),
                event,
                self.fragments.ok_or_else(|| {
                    InteractionError::SourceInput(
                        "wheel requires the current resident fragment loan".into(),
                    )
                })?,
            )
            .map_err(|error| InteractionError::SourceInput(format!("{error:?}")))?;
        Ok(input)
    }
}
struct ResizeCoordinates(Rc<dyn PageEventHandlerCoordinates>);
impl PageEventHandlerResizeCoordinates for ResizeCoordinates {
    fn ConvertFromRootFrame(&self, point: foundation::gfx::Point) -> foundation::gfx::Point {
        let converted = self
            .0
            .ConvertPointFromRootFrame(foundation::gfx::PointF::new(
                point.x() as f32,
                point.y() as f32,
            ));
        foundation::gfx::Point::new(converted.x().round() as i32, converted.y().round() as i32)
    }
}

type Pointer<'scope> =
    super::page_event_dispatcher::event_handler_pointer::PageEventHandlerPointer<'scope, 'scope>;
type InputManagers<'scope> = super::page_event_dispatcher::gesture_input::event_handler_managers::PageEventHandlerInputManagers<'scope, 'scope, 'scope>;
type Targeting<'scope> = super::page_event_dispatcher::gesture_input::event_handler_managers::PageEventHandlerGestureTargeting<'scope, 'scope, 'scope>;

struct Completion<'scope> {
    pointer: Rc<Pointer<'scope>>,
    managers: Rc<InputManagers<'scope>>,
    targeting: Rc<Targeting<'scope>>,
    keyboard: Rc<PageEventHandlerKeyboard<'scope, 'scope>>,
}
impl InteractionInputCompletion for Completion<'_> {
    fn Finish(&self) -> Result<(), InteractionError> {
        // Always drain both owners. No pending failure can leak into a later
        // transaction after an earlier port has failed.
        let pointer = self.pointer.Finish();
        let managers = self.managers.Finish();
        let targeting = self.targeting.Finish();
        let keyboard = self.keyboard.Finish();
        pointer
            .and(managers)
            .and(targeting)
            .map_err(|error| InteractionError::SourceInput(error.to_string()))?;
        keyboard.map_err(|error| InteractionError::SourceInput(error.into()))
    }
}

struct BoundFrame<'scope> {
    frame: Rc<PageEventHandlerFrame<'scope>>,
    handler: Rc<EventHandler<'scope, PageEventHandlerHitResult>>,
    input:
        NativeInteraction<'scope, PageEventHandlerHitResult, super::page_editing_owners::PageEditingOwners>,
    completion: Rc<Completion<'scope>>,
    nodes: Rc<PageEventHandlerNodes>,
    _state: super::interaction::EmitStateMutationAtExit,
    _handling: Option<super::source_input_managers::HandlingInputEventScope>,
}

/// The only scoped frame cache. Entries retain existing state handles and the
/// current listener scope, never fragment snapshots or another interaction tree.
struct BindingRegistry<'scope> {
    script_invocation:
        Option<&'scope dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation>,
    root: Rc<PageEventHandlerSource>,
    listeners: Option<EventListenerDispatcher<'scope>>,
    input: Option<&'scope InputEvent>,
    packet: Option<BorrowedInputPacket<'scope>>,
    fragments: Option<InteractionFragmentSource<'scope>>,
    result: Rc<Cell<DispatchEventResult>>,
    bindings: RefCell<HashMap<usize, Rc<BoundFrame<'scope>>>>,
    error: RefCell<Option<InteractionError>>,
    self_: Weak<Self>,
}
impl<'scope> BindingRegistry<'scope> {
    fn Bind(&self, identity: &Rc<()>) -> Result<Rc<BoundFrame<'scope>>, InteractionError> {
        let key = Rc::as_ptr(identity) as usize;
        if let Some(binding) = self.bindings.borrow().get(&key).cloned() {
            return Ok(binding);
        }
        let source = if Rc::ptr_eq(identity, &self.root.embedding.FrameDomain().Identity()) {
            self.root.clone()
        } else {
            self.root
                .embedding
                .NativeFrameSource(identity)
                .ok_or(InteractionError::DetachedFrame)?
        };
        let stage = source
            .interaction
            .WithOptionalScopedListeners(self.listeners.clone());
        let mut dispatcher = stage.Dispatcher(&source.document, None);
        dispatcher.script_invocation = self.script_invocation;
        dispatcher.BindInputDispatchResult(self.result.clone());
        let owners = &stage.source_input;
        let handling = self
            .input
            .is_some()
            .then(|| owners.pointer_frame.EnterHandlingInputEvent());
        let pointer_binding = stage.pointer_binding.borrow().clone().ok_or_else(|| {
            InteractionError::SourceInput("EventHandler resident geometry is not bound".into())
        })?;
        let scroll = stage.native_scroll.borrow().clone().ok_or_else(|| {
            InteractionError::SourceInput("EventHandler ScrollManager is not bound".into())
        })?;
        let routing = stage.native_wheel.borrow().clone().ok_or_else(|| {
            InteractionError::SourceInput("EventHandler MouseWheelEventManager is not bound".into())
        })?;
        let existing_nodes = source.nodes.borrow().clone();
        let nodes = if let Some(nodes) = existing_nodes {
            nodes
        } else {
            let nodes = PageEventHandlerNodes::new(
                source.document.clone(),
                dispatcher.KeyboardTargets(),
                Some(source.embedding.NodeGeometry()),
                Some(source.embedding.AtomicMoveState()),
            )?;
            *source.nodes.borrow_mut() = Some(nodes.clone());
            nodes
        };
        let registry: Rc<dyn PageEventHandlerFrameRegistry<'scope> + 'scope> =
            self.self_.upgrade().expect("live input registry");
        let timers = source.embedding.Timers();
        let state = owners.EventHandlerState();
        let hover_timer = PageEventHandlerTimer::new(
            timers.clone(),
            EventHandlerTimerKind::Hover,
            state.clone(),
            identity.clone(),
        );
        let cursor_timer = PageEventHandlerTimer::new(
            timers.clone(),
            EventHandlerTimerKind::CursorUpdate,
            state.clone(),
            identity.clone(),
        );
        let active_timer = PageEventHandlerTimer::new(
            timers,
            EventHandlerTimerKind::ActiveInterval,
            state.clone(),
            identity.clone(),
        );
        let hover = Rc::new(PageEventHandlerDocumentState::new(
            dispatcher.clone(),
            nodes.clone(),
            state.clone(),
            Some(active_timer.clone()),
            source.embedding.FrameDomain(),
            registry.clone(),
        )?);
        let hit = PageEventHandlerHitTest::new(
            Some(pointer_binding.geometry.clone()),
            Some(source.embedding.HitTestLifecycle()),
            Some(source.embedding.Coordinates()),
            Some(hover),
            nodes.clone(),
            Some(pointer_binding.controls.clone()),
            Some(registry.clone()),
        )?;
        let pointer = Pointer::new(
            &dispatcher,
            owners,
            nodes.clone(),
            Some(pointer_binding.geometry.clone()),
            Some(pointer_binding.controls.clone()),
        )
        .map_err(|error| InteractionError::SourceInput(error.to_string()))?;
        let modifiers = self
            .packet
            .map_or(0, |packet| packet.Event().Event().GetModifiers());
        let managers = InputManagers::WithNativeView(
            &dispatcher,
            owners,
            nodes.clone(),
            Some(pointer_binding.geometry.clone()),
            Some(pointer_binding.controls.clone()),
            scroll.ScrollManager(),
            registry.clone(),
            identity.clone(),
            modifiers,
            source.embedding.PageActive(),
            source.embedding.CursorVisible(),
        )
        .map_err(|error| InteractionError::SourceInput(error.to_string()))?;
        let converted = self
            .input
            .zip(self.packet)
            .map(|(input, packet)| super::native_interaction::ConvertedPacket(input, packet))
            .transpose()?;
        let current_key = match (self.input, converted.as_ref()) {
            (Some(InputEvent::Key(host)), Some(ConvertedWebInputEvent::Keyboard(web))) => {
                Some((host, web))
            }
            _ => None,
        };
        let keyboard = PageEventHandlerKeyboard::new(
            &dispatcher,
            current_key,
            scroll.clone(),
            owners,
            nodes.clone(),
            stage.focus_navigation.borrow().clone(),
        );
        let scroll_adapter = PageEventHandlerScroll::new(
            &scroll,
            pointer_binding.controls.clone(),
            Rc::new(ResizeCoordinates(source.embedding.Coordinates())),
        );
        let wheel_adapter = PageEventHandlerWheel::new(routing.clone(), nodes.clone());
        let targeting = Targeting::new(
            &dispatcher,
            owners,
            nodes.clone(),
            pointer_binding.geometry.clone(),
            pointer_binding.controls.clone(),
            registry.clone(),
            source.embedding.FrameDomain(),
        );
        let local_root_identity = source.embedding.FrameDomain().LocalFrameRootIdentity();
        let local_root_source = if Rc::ptr_eq(identity, &local_root_identity) {
            source.clone()
        } else {
            source
                .embedding
                .NativeFrameSource(&local_root_identity)
                .ok_or(InteractionError::DetachedFrame)?
        };
        let factory = PageEventHandlerFactory::new(
            PageEventHandlerManagers {
                selection: managers.clone(),
                hover_timer,
                cursor_timer,
                registry: local_root_source
                    .interaction
                    .NativeDispatchState(&local_root_source.document),
                scroll: scroll_adapter,
                mouse: managers.clone(),
                wheel: wheel_adapter,
                keyboard: keyboard.clone(),
                pointer: pointer.clone(),
                gesture: managers.clone(),
                active_timer,
                targeting: targeting.clone(),
            },
            Rc::ptr_eq(
                identity,
                &source.embedding.FrameDomain().LocalFrameRootIdentity(),
            ),
        );
        let middle_click_frame = source.embedding.MiddleClickAutoscrollFrame();
        // Blink's middle-click controller belongs to Page, including events
        // entering a child Frame. Reuse the local-root's retained state.
        let middle_click_state = local_root_source
            .interaction
            .SourceInputOwners()
            .autoscroll
            .clone();
        let input_ports = Rc::new(PageEventHandlerInput::new(
            Some(source.embedding.FrameInputClient()),
            middle_click_state.clone(),
            owners.mouse.clone(),
            middle_click_frame.clone(),
        )?);
        let chrome_ports = Rc::new(PageEventHandlerChrome::new(
            dispatcher.clone(),
            Some(source.embedding.ChromeClient()),
            Some(source.embedding.CursorClient()),
        )?);
        let (frame, handler) = PageEventHandlerFrame::Assemble(
            PageEventHandlerFrameServices {
                domain: Some(source.embedding.FrameDomain()),
                registry: Some(registry),
                input: Some(input_ports),
                chrome: Some(chrome_ports),
            },
            hit,
            nodes.clone(),
            source.document.clone(),
            &factory,
            state,
        )?;
        if let Some(middle_click_frame) = middle_click_frame {
            let native_handler = Rc::downgrade(&handler);
            let document = source.document.clone();
            let geometry = pointer_binding.geometry.clone();
            let frame_geometry = stage.NativeMouseFrameGeometry().ok_or_else(|| {
                InteractionError::SourceInput(
                    "middle-click requires native FrameView geometry".into(),
                )
            })?;
            dispatcher.SetMiddleClickDefaultHandler(Rc::new(move |node| {
                let link = {
                    let owner = document.borrow();
                    let document = owner.GetDocument();
                    document.FindNodeById(node).is_some_and(|index| document.EnclosingLinkEventParentOrSelf(index).is_some())
                };
                if link { return; }
                geometry.PrepareLifecycleExceptPaint().unwrap_or_else(|error| std::panic::panic_any(InteractionError::SourceInput(format!("middle-click lifecycle: {error:?}"))));
                let Some(layout) = frame_geometry.AutoscrollLayoutForNode(node).and_then(|layout| layout.FindAutoscrollable(true)) else { return };
                let Some(handler) = native_handler.upgrade() else { return };
                let controller = interaction_default::core::page::autoscroll_controller::AutoscrollController::WithState(middle_click_frame.as_ref(), &middle_click_state);
                handler.StartMiddleClickAutoscroll(layout, &controller);
            }));
        }
        let wheel = Rc::new(WheelInput {
            dispatcher: dispatcher.clone(),
            routing,
            fragments: self.fragments,
        });
        let completion = Rc::new(Completion {
            pointer: pointer.clone(),
            managers: managers.clone(),
            targeting,
            keyboard: keyboard.clone(),
        });
        let input = NativeInteraction::new(InteractionAssembly {
            handler: handler.clone(),
            host: source.embedding.Host(),
            editor: stage.editor.clone(),
            dom_events: Rc::new(dispatcher.clone()),
            output: stage.emit.clone(),
            completion: completion.clone(),
            keyboard: Rc::new(WidgetKeyboardBinding {
                registry: self.self_.clone(),
                domain: source.embedding.FrameDomain(),
                host: source.embedding.Host(),
                state: local_root_source.widget_keyboard.clone(),
            }),
            wheel: Some(wheel.clone()),
            input_method: Some(Rc::new(NativeInputMethodBinding(dispatcher.clone()))),
            focus: Some(Rc::new(NativeFocusBinding(dispatcher.clone()))),
            forms: None,
        });
        let bound = Rc::new(BoundFrame {
            frame,
            handler,
            input,
            completion,
            nodes,
            _state: stage.ExitGuard(),
            _handling: handling,
        });
        self.bindings.borrow_mut().insert(key, bound.clone());
        Ok(bound)
    }
    fn Finish(&self) -> Result<(), InteractionError> {
        let mut first = self.error.borrow_mut().take();
        for binding in self.bindings.borrow().values() {
            if let Err(error) = binding.completion.Finish() {
                if first.is_none() {
                    first = Some(error);
                }
            }
        }
        first.map_or(Ok(()), Err)
    }
}

// Managers resolve source frames synchronously through the registry. Clear its
// scoped cache on all exits (including unwind) to break the cache/manager edge;
// no listener runtime scope may be kept alive by an Rc cycle.
struct RegistryCleanup<'scope>(Rc<BindingRegistry<'scope>>);
impl Drop for RegistryCleanup<'_> {
    fn drop(&mut self) {
        self.0.bindings.borrow_mut().clear();
    }
}
impl<'scope> PageEventHandlerFrameRegistry<'scope> for BindingRegistry<'scope> {
    fn FrameForIdentity(
        &self,
        identity: &Rc<()>,
    ) -> Option<Rc<dyn EventHandlerFrame<'scope, PageEventHandlerHitResult> + 'scope>> {
        match self.Bind(identity) {
            Ok(binding) => Some(binding.frame.clone()),
            Err(error) => {
                let mut slot = self.error.borrow_mut();
                if slot.is_none() {
                    *slot = Some(error);
                }
                None
            }
        }
    }
    fn NodeForTarget(
        &self,
        target: &interaction_dispatch::core::dom::events::event::EventTargetRef,
    ) -> Option<crate::core::input::event_handler::NodeRef> {
        let identity = self
            .root
            .embedding
            .FrameDomain()
            .NodeFrameIdentity(target)?;
        self.FrameForIdentity(&identity)?;
        self.bindings
            .borrow()
            .get(&(Rc::as_ptr(&identity) as usize))?
            .nodes
            .NodeForTarget(target)
    }
    fn EventTarget(
        &self,
        node: &crate::core::input::event_handler::NodeRef,
    ) -> Option<interaction_dispatch::core::dom::events::event::EventTargetRef> {
        self.bindings
            .borrow()
            .values()
            .find_map(|binding| binding.nodes.EventTarget(node))
    }
    fn NodeForHit(
        &self,
        node: &interaction_hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<crate::core::input::event_handler::NodeRef> {
        let identity = self
            .root
            .embedding
            .FrameDomain()
            .HitNodeFrameIdentity(node)?;
        self.FrameForIdentity(&identity)?;
        self.bindings
            .borrow()
            .get(&(Rc::as_ptr(&identity) as usize))?
            .nodes
            .NodeForHit(node)
    }
    fn NodesForHit(
        &self,
        node: &interaction_hit_test::core::layout::hit_test_result::NodeRef,
    ) -> Option<Rc<PageEventHandlerNodes>> {
        let identity = self
            .root
            .embedding
            .FrameDomain()
            .HitNodeFrameIdentity(node)?;
        self.FrameForIdentity(&identity)?;
        Some(
            self.bindings
                .borrow()
                .get(&(Rc::as_ptr(&identity) as usize))?
                .nodes
                .clone(),
        )
    }
    fn OwnerElementForFrame(
        &self,
        identity: &Rc<()>,
    ) -> Option<interaction_dispatch::core::dom::events::event::EventTargetRef> {
        let source = if Rc::ptr_eq(identity, &self.root.embedding.FrameDomain().Identity()) {
            self.root.clone()
        } else {
            self.root.embedding.NativeFrameSource(identity)?
        };
        source.embedding.FrameDomain().OwnerElement()
    }
}

pub(super) fn DispatchPageInput<'scope>(
    source: Rc<PageEventHandlerSource>,
    listeners: Option<EventListenerDispatcher<'scope>>,
    input: &'scope InputEvent,
    fragments: InteractionFragmentSource<'scope>,
    packet: Option<BorrowedInputPacket<'scope>>,
    script_invocation: Option<
        &'scope dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation,
    >,
) -> Result<InteractionResult, InteractionError> {
    let identity = source.embedding.FrameDomain().Identity();
    let registry = Rc::new_cyclic(|self_| BindingRegistry {
        script_invocation,
        root: source,
        listeners,
        input: Some(input),
        packet,
        fragments: Some(fragments),
        result: Rc::new(Cell::new(Default::default())),
        bindings: Default::default(),
        error: Default::default(),
        self_: self_.clone(),
    });
    let _cleanup = RegistryCleanup(registry.clone());
    let binding = registry.Bind(&identity)?;
    let input_result = match packet {
        Some(packet) => binding.input.DispatchPacket(input, packet),
        None => binding.input.Dispatch(input),
    };
    registry.Finish()?;
    let input_result = match input_result? {
        InteractionInputResult::WebInput(result) => Some(result),
        _ => None,
    };
    let result = registry.result.get();
    Ok(InteractionResult {
        target_node_id: result.target_node_id,
        default_prevented: result.default_prevented,
        propagation_stopped: result.propagation_stopped,
        input_result,
    })
}

/// Re-enter EventHandler's retained MouseEventManager for Blink's synthetic
/// hover update. The source manager owns the current pressed-button state and
/// constructs the WebMouseEvent; the embedding only supplies the current
/// resident fragment lease used by hit testing.
pub(super) fn RecomputePageMouseHoverState<'scope>(
    source: Rc<PageEventHandlerSource>,
    listeners: Option<EventListenerDispatcher<'scope>>,
    fragments: InteractionFragmentSource<'scope>,
    script_invocation: Option<
        &'scope dyn interaction_dispatch::bindings::script_invocation::ScriptInvocation,
    >,
) -> Result<(), InteractionError> {
    let identity = source.embedding.FrameDomain().Identity();
    let registry = Rc::new_cyclic(|self_| BindingRegistry {
        script_invocation,
        root: source,
        listeners,
        input: None,
        packet: None,
        fragments: Some(fragments),
        result: Rc::new(Cell::new(Default::default())),
        bindings: Default::default(),
        error: Default::default(),
        self_: self_.clone(),
    });
    let _cleanup = RegistryCleanup(registry.clone());
    let binding = registry.Bind(&identity)?;
    binding.handler.MarkHoverStateDirty();
    binding.handler.RecomputeMouseHoverStateIfNeeded();
    registry.Finish()
}

pub(super) fn RunPageEventHandlerTimer<'scope>(
    source: Rc<PageEventHandlerSource>,
    listeners: Option<EventListenerDispatcher<'scope>>,
    task: &EventHandlerTimerTask,
) -> Result<(), InteractionError> {
    let source = if Rc::ptr_eq(
        &task.frame_identity,
        &source.embedding.FrameDomain().Identity(),
    ) {
        source
    } else {
        let Some(current) = source.embedding.NativeFrameSource(&task.frame_identity) else {
            return Ok(());
        };
        current
    };
    if !source
        .interaction
        .source_input
        .EventHandlerState()
        .IsCurrentTimerTask(task)
    {
        return Ok(());
    }
    let registry = Rc::new_cyclic(|self_| BindingRegistry {
        script_invocation: None,
        root: source,
        listeners,
        input: None,
        packet: None,
        fragments: None,
        result: Rc::new(Cell::new(Default::default())),
        bindings: Default::default(),
        error: Default::default(),
        self_: self_.clone(),
    });
    let _cleanup = RegistryCleanup(registry.clone());
    let binding = registry.Bind(&task.frame_identity)?;
    binding.handler.RunTimerTask(task);
    registry.Finish()
}

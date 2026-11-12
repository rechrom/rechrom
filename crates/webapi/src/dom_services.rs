use super::*;
use interaction::event::{
    Event, EventListenerInvocation, EventListenerResult, EventPhase, EventType, EventTypeName,
};
use interaction::input_event::{InputEvent, MouseButton};
use javascript::javascript_runtime::{
    JavaScriptException, JavaScriptHostRuntime, JavaScriptRealm, JavaScriptResult,
    JavaScriptRuntime,
};

/// Native CloseWatcher operations are routed to Interaction's canonical
/// document stack; only the invocation borrows the current JavaScript runtime.
#[derive(Clone)]
pub struct CloseWatcherHost {
    pub create: Rc<
        dyn Fn(
            u64,
            Rc<interaction::dispatch::core::dom::events::event_target::EventTarget>,
            bool,
        ) -> Result<
            Rc<interaction::default::core::html::closewatcher::close_watcher::NativeCloseWatcher>,
            String,
        >,
    >,
    pub invoke: Rc<
        dyn Fn(
            foundation::popup_types::CloseWatcherId,
            bool,
            &mut dyn JavaScriptHostRuntime,
        ) -> Result<(), String>,
    >,
    pub destroy: Rc<dyn Fn(foundation::popup_types::CloseWatcherId)>,
}

// Rust ownership adapter for the callbacks in DOMBindingsHost.
// Page must supply real resolution and measurement; absent callbacks retain the
// source's default behavior and do not imply that a Page has been implemented.
pub type FormValidationHandler = Rc<
    dyn Fn(
        HostObjectId,
        bool,
        &mut dyn JavaScriptHostRuntime,
    ) -> Result<bool, interaction::adapter::page_form_validation::ValidationRequired>,
>;

pub type FormDataConstructor = Rc<
    dyn Fn(
        HostObjectId,
        Option<HostObjectId>,
        &mut dyn JavaScriptHostRuntime,
    ) -> Result<
        Rc<interaction::dispatch::core::fileapi::form_data_object::FormDataObject>,
        interaction::dispatch::core::fileapi::form_data_object::FormDataConstructionError,
    >,
>;

pub type SyntheticEventHandler =
    Rc<dyn Fn(&mut Event, HostObjectId, &mut dyn JavaScriptHostRuntime)>;
pub type DocumentEditorCommandHandler = Rc<
    dyn Fn(
        interaction::DocumentEditorCommandOperation,
        &str,
        &str,
        &mut dyn JavaScriptHostRuntime,
    ) -> Result<
        interaction::DocumentEditorCommandResult,
        interaction::event_dispatcher::EditorCommandError,
    >,
>;
/// WebAPI checks the optional union while Browser holds Interaction's opaque
/// prepared command transaction. The editing engine never reads script values.
pub type DocumentEditorCommandValueCheck<'a> =
    dyn FnMut(bool, &mut dyn JavaScriptHostRuntime) -> Result<String, HostResult> + 'a;
pub type DocumentEditorCommandExecutor = Rc<
    dyn Fn(
        &str,
        &mut DocumentEditorCommandValueCheck<'_>,
        &mut dyn JavaScriptHostRuntime,
    ) -> HostResult,
>;

/// DOMSelection is owned by Interaction's retained FrameSelection.  WebAPI
/// only projects boundary values and routes mutations; it never stores a
/// second selection or derives one from JavaScript wrapper state.
#[derive(Clone)]
pub struct DOMSelectionHost {
    pub read: Rc<dyn Fn() -> Option<interaction::DOMSelectionSnapshot>>,
    pub write: Rc<dyn Fn(Option<dom::DOMStaticRange>) -> bool>,
    pub text: Rc<dyn Fn() -> Vec<u16>>,
}

/// Text-control selection remains owned by Interaction's retained
/// SelectionState. WebAPI only performs Web IDL conversion and projects the
/// primitive offsets/direction.
#[derive(Clone)]
pub struct TextControlSelectionHost {
    pub read: Rc<dyn Fn(HostObjectId) -> Option<interaction::TextControlSelectionSnapshot>>,
    pub write: Rc<dyn Fn(HostObjectId, usize, usize, bool) -> bool>,
}

pub type DOMEventTargetResolver = Rc<
    dyn Fn(HostObjectId) -> Option<interaction::dispatch::core::dom::events::event::EventTargetRef>,
>;
pub type DOMWindowViewConverter = Rc<
    dyn Fn(
        &HostValue,
        &mut dyn JavaScriptHostRuntime,
    ) -> Option<interaction::dispatch::core::events::ui_event::AbstractViewRef>,
>;
pub type DOMWindowViewProjector = Rc<
    dyn Fn(&interaction::dispatch::core::events::ui_event::AbstractViewRef) -> Option<HostValue>,
>;

/// HTMLFrameOwnerElement's nullable browsing-context projections. The host
/// resolves these from its resident Frame owner, including current security
/// origin; bindings never create a second Page or infer a document from ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameProperty {
    ContentWindow,
    ContentDocument,
}
pub type FramePropertyGetter =
    Rc<dyn Fn(HostObjectId, FrameProperty, &mut dyn JavaScriptHostRuntime) -> HostResult>;

#[derive(Clone, Copy, Debug, Default)]
pub struct DocumentFocusState {
    pub active_element: Option<HostObjectId>,
    pub has_focus: bool,
}

#[derive(Default)]
pub struct DOMBindingsHost {
    pub execute_document_editor_command: Option<DocumentEditorCommandExecutor>,
    pub document_editor_command: Option<DocumentEditorCommandHandler>,
    pub selection: Option<DOMSelectionHost>,
    pub text_control_selection: Option<TextControlSelectionHost>,
    /// Maps this original Document's wrapper id to its existing Interaction
    /// dispatch target. WebAPI never constructs a parallel DOM target owner.
    pub resolve_event_target: Option<DOMEventTargetResolver>,
    /// Existing LocalFrame Window/proxy bindings supply cross-realm conversion
    /// and projection. No WebAPI-owned Window or Frame identity table is made.
    pub convert_window_view: Option<DOMWindowViewConverter>,
    pub project_window_view: Option<DOMWindowViewProjector>,
    /// Original Frame's CSS-to-physical LayoutZoomFactor for drag locations.
    pub read_layout_zoom: Option<Rc<dyn Fn() -> f32>>,
    /// Retain the same CachedImage resource; Raster remains embedding-owned.
    pub retain_drag_image: Option<
        Rc<
            dyn Fn(
                Rc<dom::persistent_document::DOMImageResourceContent>,
            ) -> Rc<
                dyn interaction::dispatch::core::clipboard::data_transfer::DragImageResource,
            >,
        >,
    >,
    /// Live page/frame focus projected for this original Document identity.
    pub read_document_focus: Option<Rc<dyn Fn(u128) -> Option<DocumentFocusState>>>,
    pub validate_form: Option<FormValidationHandler>,
    pub read_frame: Option<FramePropertyGetter>,
    /// Page supplies its clock domain. Standalone bindings use a real local
    /// monotonic host clock when no embedding override is supplied.
    pub event_clock: Option<Rc<dyn interaction::dispatch::core::dom::events::event::EventClock>>,
    pub dispatch_synthetic: Option<SyntheticEventHandler>,
    /// Same Style source and the fetcher's read-only policy slot. Bindings only
    /// route the current Document/world; parser decisions remain inside Style.
    pub cssom_document: Option<(
        Rc<style::DocumentCSSOMSource>,
        foundation::referrer_policy::ReferrerPolicyReadHandle,
    )>,
    pub submit_form: Option<
        Rc<dyn Fn(HostObjectId, Option<HostObjectId>, bool, &mut dyn JavaScriptHostRuntime)>,
    >,
    pub sample_animation:
        Option<Box<dyn FnMut(HostObjectId, u64, f64, Vec<cssom::CSSDeclaration>)>>,
    /// Update normal style, then ensure the queried element's computed style.
    pub update_style: Option<Box<dyn FnMut(HostObjectId)>>,
    pub emit_style_sheet: Option<Box<dyn FnMut(cssom::CSSStyleSheet)>>,
    /// A constructed stylesheet or TreeScope adoption changed without a DOM
    /// node mutation. Page schedules the ordinary style lifecycle.
    pub invalidate_style: Option<Box<dyn FnMut()>>,
    /// None means this used metric is not applicable (for example CSS width
    /// on a non-replaced inline); CSSOM then serializes the computed value.
    pub read_metric: Option<Box<dyn FnMut(HostObjectId, &str) -> Option<f64>>>,
    pub read_viewport_scroll: Option<Box<dyn FnMut() -> Option<ViewportScrollState>>>,
    /// Programmatic viewport scrolls retain their behavior so Page can route
    /// smooth requests through the translated Chromium animation curve.
    pub write_viewport_scroll: Option<
        Box<
            dyn FnMut(
                HostObjectId,
                layoutng_assembly::internal::layout_input::Offset,
                ViewportScrollBehavior,
            ),
        >,
    >,
    pub read_geometry: Option<Box<dyn FnMut(HostObjectId) -> Vec<paint::PaintRect>>>,
    pub write_scroll:
        Option<Box<dyn FnMut(HostObjectId, layoutng_assembly::internal::layout_input::Offset)>>,
    /// Document.fonts reflects the connected resource loader instead of a
    /// JavaScript-only readiness guess. True means at least one selected face
    /// still has transport or decode work.
    pub read_font_loading: Option<Box<dyn FnMut() -> bool>>,
}
pub(super) struct StandaloneEventClock(std::time::Instant);
impl StandaloneEventClock {
    pub(super) fn new() -> Self {
        Self(std::time::Instant::now())
    }
}
impl interaction::dispatch::core::dom::events::event::EventClock for StandaloneEventClock {
    fn Now(&self) -> interaction::input::base::time::time::TimeTicks {
        interaction::input::base::time::time::TimeTicks::FromInternalValue(
            self.0.elapsed().as_micros().min((i64::MAX - 1) as u128) as i64 + 1,
        )
    }
}

// cpp: webapi/dom_bindings.h:84-111
struct BoundEvent {
    event: Event,
    target: HostObjectId,
    current_target: HostObjectId,
    phase: EventPhase,
    dispatching: bool,
    passive_listener: bool,
    stop_propagation: bool,
    stop_immediate_propagation: bool,
    client_x: f64,
    client_y: f64,
    delta_x: f64,
    delta_y: f64,
}

fn event_path_value(event: &Event, current_target: HostObjectId) -> HostValue {
    let nodes = event.canonical.as_ref().map_or_else(
        || {
            let state = event.dispatch_state.borrow();
            state
                .path
                .as_ref()
                .map_or_else(Vec::new, |path| path.ComposedPath(current_target))
        },
        |native| {
            use interaction::dispatch::core::dom::events::event::EventTargetKind;
            native
                .borrow()
                .BaseEvent()
                .composedPath(true)
                .into_iter()
                .filter_map(|target| match target.Kind() {
                    EventTargetKind::Window => Some(0),
                    EventTargetKind::Node => target.NodeId(),
                    // A generic EventTarget has a singleton path and no DOM
                    // node identity; its binding identity is currentTarget.
                    EventTargetKind::Other => Some(current_target),
                })
                .collect()
        },
    );
    let mut values = Vec::with_capacity(nodes.len() + 1);
    values.push(("length".into(), HostValue::Number(nodes.len() as f64)));
    values.extend(
        nodes
            .into_iter()
            .enumerate()
            .map(|(index, id)| (index.to_string(), HostValue::Object(HostObjectRef { id }))),
    );
    HostValue::Record(values)
}

// cpp: webapi/dom_bindings.h:139-141
// Constructed JS Events retain their realm identity while sharing the native
// bound-event state used for path, phase and action flags during dispatch.
#[derive(Clone)]
struct SyntheticDispatch {
    identity: u64,
    handle: HostValue,
    target: HostObjectId,
}

// Event::Trace owns target/path references. A dispatch supplies a temporary
// native root; a retained JS wrapper subsequently owns only the final target.
#[derive(Default)]
pub(super) struct BoundEvents {
    records: HashMap<HostObjectId, BoundEvent>,
    identities: HashMap<u64, HostObjectId>,
    native_identities: HashMap<usize, HostObjectId>,
    active: std::collections::HashSet<HostObjectId>,
    pending: Vec<javascript::javascript_runtime::HostObjectTraceChange>,
}
impl BoundEvents {
    fn NativeDispatchState(
        native: &interaction::dispatch::core::dom::events::event::EventRef,
        fallback_target: HostObjectId,
    ) -> (HostObjectId, HostObjectId, EventPhase) {
        use interaction::dispatch::core::dom::events::event::{EventTargetKind, PhaseType};

        let owner = native.borrow();
        let base = owner.BaseEvent();
        let project =
            |target: Option<interaction::dispatch::core::dom::events::event::EventTargetRef>| {
                target
                    .and_then(|target| match target.Kind() {
                        EventTargetKind::Window => Some(0),
                        EventTargetKind::Node => target.NodeId(),
                        EventTargetKind::Other => None,
                    })
                    .unwrap_or(fallback_target)
            };
        let phase = match base.eventPhase() {
            PhaseType::kNone => EventPhase::kNone,
            PhaseType::kCapturingPhase => EventPhase::kCapturing,
            PhaseType::kAtTarget => EventPhase::kAtTarget,
            PhaseType::kBubblingPhase => EventPhase::kBubbling,
        };
        (project(base.target()), project(base.currentTarget()), phase)
    }

    fn SyncNativeDispatch(
        &mut self,
        id: HostObjectId,
        native: &interaction::dispatch::core::dom::events::event::EventRef,
        fallback_target: HostObjectId,
    ) {
        let (target, current_target, phase) = Self::NativeDispatchState(native, fallback_target);
        let owner = native.borrow();
        let base = owner.BaseEvent();
        let bound = self.records.get_mut(&id).expect("native Event wrapper");
        bound.event.target_node_id = target;
        bound.event.current_target_node_id = current_target;
        bound.event.phase = phase;
        bound.event.custom_type = base.r#type().into();
        bound.event.trusted = base.isTrusted();
        bound.event.default_prevented = base.defaultPrevented();
        bound.target = target;
        bound.current_target = current_target;
        bound.phase = phase;
        bound.dispatching = base.IsBeingDispatched();
        drop(owner);
        self.PublishReferences(id);
    }

    fn PublishReferences(&mut self, id: HostObjectId) {
        use javascript::javascript_runtime::HostObjectTraceChange as C;
        let bound = self.records.get(&id).expect("bound Event");
        let state = bound.event.dispatch_state.borrow();
        let mut targets = state.path.as_ref().map_or_else(Vec::new, |path| {
            path.Nodes().filter(|&id| id != 0).collect()
        });
        targets.extend(
            [
                Some(bound.target),
                Some(bound.current_target),
                bound.event.submitter_node_id,
                bound.event.related_target_node_id,
            ]
            .into_iter()
            .flatten()
            .filter(|&id| id != 0),
        );
        self.pending.push(C::ReferenceList {
            owner: id,
            targets: targets.into(),
        });
    }

    pub(super) fn NativeEvent(
        &self,
        id: u64,
    ) -> Option<interaction::dispatch::core::dom::events::event::EventRef> {
        self.records.get(&id)?.event.canonical.clone()
    }
    pub(super) fn BindConstructedNative(
        &mut self,
        native: &interaction::dispatch::core::dom::events::event::EventRef,
        allocate: impl FnOnce() -> u64,
    ) -> u64 {
        let id = self.BindNative(native, 0, allocate);
        self.EndNative(id);
        self.records[&id]
            .event
            .dispatch_state
            .borrow_mut()
            .final_target = None;
        id
    }
    pub(super) fn BindNative(
        &mut self,
        native: &interaction::dispatch::core::dom::events::event::EventRef,
        target: u64,
        allocate: impl FnOnce() -> u64,
    ) -> u64 {
        let existing = self.native_identities.get(&native.Identity()).copied();
        if let Some(id) = existing {
            if self.records[&id].dispatching {
                self.SyncNativeDispatch(id, native, target);
                return id;
            }
        }
        let id = existing.unwrap_or_else(allocate);
        let identity = existing.map_or(native.Identity() as u64, |id| {
            self.records[&id].event.identity
        });
        let dispatch_state = existing
            .map(|id| self.records[&id].event.dispatch_state.clone())
            .unwrap_or_default();
        {
            let mut state = dispatch_state.borrow_mut();
            state.finished = false;
            state.final_target = None;
        }
        let owner = native.borrow();
        let base = owner.BaseEvent();
        let (target, current_target, phase) = Self::NativeDispatchState(native, target);
        let event = Event {
            r#type: EventType::kCustom,
            custom_type: base.r#type().into(),
            interface_name: owner.InterfaceName().into(),
            canonical: Some(native.clone()),
            identity,
            dispatch_state,
            target_node_id: target,
            current_target_node_id: current_target,
            phase,
            bubbles: base.bubbles(),
            cancelable: base.cancelable(),
            composed: base.composed(),
            trusted: base.isTrusted(),
            default_prevented: base.defaultPrevented(),
            ..Default::default()
        };
        self.identities.insert(identity, id);
        self.Insert(
            id,
            BoundEvent {
                event,
                target,
                current_target,
                phase,
                dispatching: base.IsBeingDispatched(),
                passive_listener: false,
                stop_propagation: false,
                stop_immediate_propagation: false,
                client_x: 0.0,
                client_y: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
            },
        );
        id
    }
    pub(super) fn SetNativePassive(&mut self, id: u64, passive: bool) {
        self.records
            .get_mut(&id)
            .expect("native Event wrapper")
            .passive_listener = passive;
    }
    pub(super) fn EndNative(&mut self, id: u64) {
        let bound = self.records.get_mut(&id).expect("native Event wrapper");
        bound.dispatching = false;
        bound.phase = EventPhase::kNone;
        bound.passive_listener = false;
        let mut state = bound.event.dispatch_state.borrow_mut();
        state.finished = true;
        state.final_target = Some(bound.target);
    }
    fn Insert(&mut self, id: HostObjectId, bound: BoundEvent) {
        if let Some(native) = bound.event.canonical.as_ref() {
            self.native_identities.insert(native.Identity(), id);
        }
        use javascript::javascript_runtime::HostObjectTraceChange as C;
        let started = self.active.insert(id);
        self.records.insert(id, bound);
        self.PublishReferences(id);
        if started {
            self.pending.push(C::Root {
                id,
                reason: 6,
                rooted: true,
            });
        }
    }
    pub(super) fn TakeChanges(
        &mut self,
    ) -> Vec<javascript::javascript_runtime::HostObjectTraceChange> {
        use javascript::javascript_runtime::HostObjectTraceChange as C;
        self.active.retain(|&id| {
            let bound = &self.records[&id];
            let state = bound.event.dispatch_state.borrow();
            if !state.finished {
                return true;
            }
            let targets = [
                state.final_target,
                bound.event.submitter_node_id,
                bound.event.related_target_node_id,
            ]
            .into_iter()
            .flatten()
            .filter(|&id| id != 0)
            .collect::<Vec<_>>();
            self.pending.extend([
                C::ReferenceList {
                    owner: id,
                    targets: targets.into(),
                },
                C::Root {
                    id,
                    reason: 6,
                    rooted: false,
                },
            ]);
            false
        });
        std::mem::take(&mut self.pending)
    }
    pub(super) fn Retire(&mut self, id: HostObjectId) {
        if let Some(bound) = self.records.remove(&id) {
            self.identities.remove(&bound.event.identity);
            if let Some(native) = bound.event.canonical.as_ref() {
                self.native_identities.remove(&native.Identity());
            }
        }
        self.active.remove(&id);
    }
}

pub(super) struct DOMServices {
    synthetic: Rc<RefCell<Option<SyntheticDispatch>>>,
    // The final helper is looked up after listeners, and listeners may replace
    // it through __domWrap. Share this slot with the detached continuation.
    pub(super) event_state: Rc<RefCell<Option<JavaScriptFunction>>>,
    pub(super) host: DOMBindingsHost,
    pub(super) events: Rc<RefCell<BoundEvents>>,
    next_event_object_id: HostObjectId,
}
impl DOMServices {
    pub(super) fn ActiveSyntheticDispatch(&self) -> Option<(HostValue, HostObjectId)> {
        self.synthetic
            .borrow()
            .as_ref()
            .map(|dispatch| (dispatch.handle.clone(), dispatch.target))
    }

    pub(super) fn BindInputEvent(
        &mut self,
        event: interaction::dispatch::core::dom::events::event::EventRef,
    ) -> HostObjectId {
        let id = self.next_event_object_id;
        self.next_event_object_id += 1;
        self.events
            .borrow_mut()
            .BindConstructedNative(&event, || id)
    }

    pub(super) fn EventInterfaceName(&self, id: u64) -> Option<String> {
        self.events
            .borrow()
            .records
            .get(&id)
            .map(|event| event.event.interface_name.clone())
    }
    pub(super) fn MemoryCounts(&self) -> (usize, usize) {
        let events = self.events.borrow();
        (events.records.len(), events.identities.len())
    }
    pub fn new(host: DOMBindingsHost) -> Self {
        Self {
            host,
            synthetic: Rc::new(RefCell::new(None)),
            event_state: Rc::new(RefCell::new(None)),
            events: Default::default(),
            next_event_object_id: 1 << 63,
        }
    }
    pub fn IsEvent(&self, id: HostObjectId) -> bool {
        self.events.borrow().records.contains_key(&id)
    }
}

impl DOMJavaScriptBindings {
    /// Document lifecycle events originate in Open while it owns the current
    /// parser runtime. Chromium still creates one native Event before capture,
    /// target and bubble listener selection; every phase and retained wrapper
    /// observes that same owner. This restores that canonical edge before the
    /// direct current-runtime listener path is entered.
    pub fn PrepareLifecycleEvent(&self, event: &mut Event) {
        if event.canonical.is_some() {
            return;
        }
        assert!(matches!(
            event.r#type,
            EventType::kDOMContentLoaded | EventType::kReadyStateChange | EventType::kLoad
        ));
        use interaction::dispatch::core::dom::events::event::{
            Bubbles, Cancelable, ComposedMode, Event as NativeEvent, EventInterface, EventRef,
        };
        let name = if event.custom_type.is_empty() {
            EventTypeName(event.r#type)
        } else {
            &event.custom_type
        };
        let timestamp = self
            .services
            .host
            .event_clock
            .as_ref()
            .expect("DOM bindings retain their embedding event clock")
            .Now();
        let mut native = NativeEvent::new(
            name,
            if event.bubbles {
                Bubbles::kYes
            } else {
                Bubbles::kNo
            },
            if event.cancelable {
                Cancelable::kYes
            } else {
                Cancelable::kNo
            },
            if event.composed {
                ComposedMode::kComposed
            } else {
                ComposedMode::kScoped
            },
            timestamp,
        );
        native.SetTrusted(event.trusted);
        event.canonical = Some(EventRef::new(native));
    }

    /// Open drives document lifecycle propagation while the parser owns the
    /// current runtime. Keep the translated EventTarget listener selection on
    /// the same phase/filter state that EventDispatcher would establish for
    /// each invocation of the canonical Event.
    pub fn PrepareLifecycleEventInvocation(
        &self,
        event: &Event,
        phase: EventPhase,
        capture_listeners: bool,
    ) {
        use interaction::dispatch::core::dom::events::event::PhaseType;
        let native = event
            .canonical
            .as_ref()
            .expect("lifecycle dispatch retains the canonical Event");
        let mut owner = native.borrow_mut();
        let event = owner.BaseEventMut();
        if event.eventPhase() == PhaseType::kAtTarget {
            event.SetFireOnlyCaptureListenersAtTarget(false);
            event.SetFireOnlyNonCaptureListenersAtTarget(false);
        }
        let native_phase = match phase {
            EventPhase::kNone => PhaseType::kNone,
            EventPhase::kCapturing => PhaseType::kCapturingPhase,
            EventPhase::kAtTarget => PhaseType::kAtTarget,
            EventPhase::kBubbling => PhaseType::kBubblingPhase,
        };
        event.SetEventPhase(native_phase);
        if phase == EventPhase::kAtTarget {
            if capture_listeners {
                event.SetFireOnlyCaptureListenersAtTarget(true);
            } else {
                event.SetFireOnlyNonCaptureListenersAtTarget(true);
            }
        }
    }

    pub fn FinishLifecycleEvent(&self, event: &Event) {
        use interaction::dispatch::core::dom::events::event::PhaseType;
        let native = event
            .canonical
            .as_ref()
            .expect("lifecycle dispatch retains the canonical Event");
        let mut owner = native.borrow_mut();
        let event = owner.BaseEventMut();
        if event.eventPhase() == PhaseType::kAtTarget {
            event.SetFireOnlyCaptureListenersAtTarget(false);
            event.SetFireOnlyNonCaptureListenersAtTarget(false);
        }
        event.SetEventPhase(PhaseType::kNone);
        event.SetCurrentTarget(None);
        event.SetStopPropagation(false);
        event.SetStopImmediatePropagation(false);
    }

    // cpp: webapi/dom_bindings.cc:1102-1208
    // All host/document borrows end before calling JavaScript. Both entry
    // points preserve the same wrapper, listener and synthetic-event logic.
    pub fn DispatchEventListeners(
        bindings: &Rc<RefCell<Self>>,
        invocation: &EventListenerInvocation<'_>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> EventListenerResult {
        let mut result = None;
        runtime
            .WithHostRuntime(realm, &mut |runtime| {
                result = Some(Self::DispatchEventListenersScoped(
                    bindings,
                    invocation,
                    runtime,
                    report_error,
                ));
            })
            .expect("current concrete HostRuntime");
        result.expect("synchronous listener dispatch")
    }

    pub fn DispatchEventListenersScoped(
        bindings: &Rc<RefCell<Self>>,
        invocation: &EventListenerInvocation<'_>,
        runtime: &mut dyn JavaScriptHostRuntime,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> EventListenerResult {
        let namespace = bindings.borrow().HostObjectNamespace();
        let _namespace = namespace
            .as_ref()
            .map(|namespace| runtime.EnterHostObjectNamespace(namespace));
        Self::DispatchEventListenersWithCall(
            bindings,
            invocation,
            &RefCell::new(runtime),
            report_error,
        )
    }

    fn DispatchEventListenersWithCall(
        bindings: &Rc<RefCell<Self>>,
        invocation: &EventListenerInvocation<'_>,
        runtime: &RefCell<&mut dyn JavaScriptHostRuntime>,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> EventListenerResult {
        let (target, event_id) = {
            let mut this = bindings.borrow_mut();
            this.SetFocusedNode(invocation.focused_node_id);
            let Some(target) = this
                .listeners
                .borrow()
                .get(&invocation.current_target_node_id)
                .cloned()
            else {
                return EventListenerResult::default();
            };
            let services = &mut this.services;
            let mut events = services.events.borrow_mut();
            let id = if let Some(&id) = events.identities.get(&invocation.event.identity) {
                id
            } else {
                let id = services.next_event_object_id;
                services.next_event_object_id += 1;
                events.identities.insert(invocation.event.identity, id);
                id
            };
            let (mut x, mut y, mut dx, mut dy) = (0.0, 0.0, 0.0, 0.0);
            match &invocation.event.underlying_event {
                Some(InputEvent::Mouse(v)) => {
                    x = v.position.x;
                    y = v.position.y;
                }
                Some(InputEvent::Pointer(v)) => {
                    x = v.position.x;
                    y = v.position.y;
                }
                Some(InputEvent::Wheel(v)) => {
                    x = v.position.x;
                    y = v.position.y;
                    dx = v.delta.x;
                    dy = v.delta.y;
                }
                _ => {}
            }
            events.Insert(
                id,
                BoundEvent {
                    event: invocation.event.clone(),
                    target: invocation.target_node_id,
                    current_target: invocation.current_target_node_id,
                    phase: invocation.phase,
                    dispatching: true,
                    passive_listener: false,
                    stop_propagation: false,
                    stop_immediate_propagation: false,
                    client_x: x,
                    client_y: y,
                    delta_x: dx,
                    delta_y: dy,
                },
            );
            (target, id)
        };
        let synthetic = {
            let this = bindings.borrow();
            let state = this.services.synthetic.borrow();
            state
                .as_ref()
                .filter(|active| active.identity == invocation.event.identity)
                .map(|active| (this.services.synthetic.clone(), active.handle.clone()))
        };
        let argument = synthetic
            .as_ref()
            .map(|(_, handle)| handle.clone())
            .unwrap_or(HostValue::Object(HostObjectRef { id: event_id }));
        use interaction::dispatch::core::dom::events::{
            event::{EventRef, PassiveMode},
            event_listener::{EventListenerRef, ExecutionContext},
            event_target::ScopedEventListenerInvoker,
        };
        struct Invoker<'a> {
            invoke: RefCell<&'a mut dyn FnMut(&EventListenerRef, &EventRef)>,
        }
        impl ScopedEventListenerInvoker for Invoker<'_> {
            fn Invoke(
                &self,
                listener: &EventListenerRef,
                _: Option<&dyn ExecutionContext>,
                event: &EventRef,
            ) {
                (self.invoke.borrow_mut())(listener, event);
            }
        }
        let native = invocation
            .event
            .canonical
            .as_ref()
            .expect("listener dispatch retains the canonical Event");
        let mut invoke = |selected: &EventListenerRef, native: &EventRef| {
            let listener = Listener::From(selected.as_ref());
            let Some(callback) = listener.callback.Resolve() else {
                return;
            };
            let passive = !matches!(
                native.borrow().BaseEvent().HandlingPassive(),
                PassiveMode::kNotPassive | PassiveMode::kNotPassiveDefault
            );
            {
                let this = bindings.borrow();
                let mut events = this.services.events.borrow_mut();
                let bound = events.records.get_mut(&event_id).unwrap();
                bound.passive_listener = passive;
                // EventTarget may select a trusted legacy event type. The
                // wrapper reads that same temporary source Event::type value.
                bound.event.custom_type = native.borrow().BaseEvent().r#type().to_owned();
            }
            if synthetic.is_some() {
                let update = bindings
                    .borrow()
                    .method_wrappers
                    .get("__eventState")
                    .cloned();
                if let Some(update) = update {
                    runtime.borrow_mut().Call(
                        &update,
                        &HostValue::Object(HostObjectRef { id: 0 }),
                        &[
                            argument.clone(),
                            HostValue::Object(HostObjectRef {
                                id: invocation.target_node_id,
                            }),
                            HostValue::Object(HostObjectRef {
                                id: invocation.current_target_node_id,
                            }),
                            HostValue::Number(invocation.phase as u8 as f64),
                            HostValue::Boolean(passive),
                            HostValue::Object(HostObjectRef { id: event_id }),
                        ],
                    );
                }
            }
            let handler_wrapper = listener
                .attribute
                .then(|| {
                    bindings
                        .borrow()
                        .method_wrappers
                        .get("__callEventHandler")
                        .cloned()
                })
                .flatten();
            let receiver = HostValue::Object(HostObjectRef {
                id: invocation.current_target_node_id,
            });
            let current_event = bindings.borrow().current_event.clone();
            let mut report = |runtime: &mut dyn JavaScriptHostRuntime, error: &HostValue| {
                let diagnostic = runtime.ExceptionMessage(error);
                let location = diagnostic.location;
                report_error(&JavaScriptException {
                    message: String::from_utf16_lossy(&diagnostic.message),
                    source_name: location
                        .as_ref()
                        .map_or_else(String::new, |location| location.source_name.clone()),
                    line: location
                        .as_ref()
                        .map_or(0, |location| location.line as usize),
                    column: location
                        .as_ref()
                        .map_or(0, |location| location.column as usize),
                    ..Default::default()
                });
            };
            event_listener_invocation::Invocation {
                listener,
                callback: &callback,
                runtime,
                receiver,
                argument: argument.clone(),
                handler_wrapper,
                current_event,
                report_error: RefCell::new(&mut report),
            }
            .Invoke(native);
        };
        target.FireEventListenersWithInvoker(
            native,
            Some(&Invoker {
                invoke: RefCell::new(&mut invoke),
            }),
        );
        let this = bindings.borrow();
        let mut events = this.services.events.borrow_mut();
        let bound = events.records.get_mut(&event_id).unwrap();
        bound.dispatching = false;
        bound.phase = EventPhase::kNone;
        bound.passive_listener = false;
        EventListenerResult {
            prevent_default: bound.event.default_prevented,
            stop_propagation: bound.stop_propagation,
            stop_immediate_propagation: bound.stop_immediate_propagation,
        }
    }

    pub fn SetSyntheticEventHandler(&mut self, handler: Option<SyntheticEventHandler>) {
        self.services.host.dispatch_synthetic = handler;
    }

    pub fn SetFormDataConstructor(&mut self, constructor: Option<FormDataConstructor>) {
        self.native_abort.BindFormDataConstructor(constructor);
    }

    pub fn SetFormSubmissionHandler(
        &mut self,
        handler: Option<
            Rc<dyn Fn(HostObjectId, Option<HostObjectId>, bool, &mut dyn JavaScriptHostRuntime)>,
        >,
    ) {
        self.services.host.submit_form = handler;
    }

    // cpp: webapi/dom_bindings.cc:634-653
    pub(super) fn PrepareSyntheticDispatch(
        &mut self,
        receiver: HostObjectId,
        arguments: &[HostValue],
    ) -> HostContinuation {
        let invalid = || {
            Box::new(|_: &mut dyn JavaScriptHostRuntime| Self::type_error("Invalid event dispatch"))
                as HostContinuation
        };
        let [HostValue::String(kind), HostValue::Boolean(bubbles), HostValue::Boolean(cancelable), HostValue::Boolean(composed), handle, ..] =
            arguments
        else {
            return invalid();
        };
        if self.native_abort.NativeTarget(receiver).is_some() {
            let native = arguments
                .get(5)
                .filter(|value| !matches!(value, HostValue::Undefined(_) | HostValue::Null(_)))
                .or_else(|| {
                    if matches!(handle, HostValue::Object(_)) {
                        Some(handle)
                    } else {
                        None
                    }
                })
                .and_then(|value| match value {
                    HostValue::Object(value) => self.services.events.borrow().NativeEvent(value.id),
                    HostValue::Retained(owner, _) => owner.downcast_ref::<javascript::javascript_runtime::JavaScriptNativeObjectReference>()?.NativeOwner::<interaction::dispatch::core::dom::events::event::EventRef>().map(|event| event.as_ref().clone()),
                    _ => None,
                });
            return self.native_abort.PrepareNativeDispatch(
                receiver,
                kind.clone(),
                *bubbles,
                *cancelable,
                *composed,
                handle.clone(),
                native,
                arguments
                    .get(6)
                    .is_none_or(|value| !matches!(value, HostValue::Boolean(false))),
                arguments
                    .get(7)
                    .map_or(!matches!(handle, HostValue::Object(_)), |value| {
                        matches!(value, HostValue::Boolean(true))
                    }),
            );
        }
        let Some(dispatch) = self.services.host.dispatch_synthetic.clone() else {
            return invalid();
        };
        if !self.runtime_realm.IsValid() {
            return invalid();
        }
        // Redispatch a retained native wrapper through its original identity.
        // Constructed JS Events use the synthetic handle projection below.
        if let HostValue::Object(object) = handle {
            if let Some(bound) = self
                .services
                .events
                .borrow_mut()
                .records
                .get_mut(&object.id)
            {
                if bound.dispatching {
                    return invalid();
                }
                bound.event.trusted = false;
                let mut event = bound.event.clone();
                event.propagation_stopped = false;
                event.immediate_propagation_stopped = false;
                return Box::new(move |runtime| {
                    dispatch(&mut event, receiver, runtime);
                    Self::value(HostValue::Boolean(!event.default_prevented))
                });
            }
        }
        let mut event = Event {
            r#type: EventType::kCustom,
            custom_type: kind.clone(),
            target_node_id: receiver,
            bubbles: *bubbles,
            cancelable: *cancelable,
            composed: *composed,
            trusted: false,
            ..Default::default()
        };
        for candidate in [
            EventType::kMouseMove,
            EventType::kMouseDown,
            EventType::kMouseUp,
            EventType::kClick,
            EventType::kDoubleClick,
            EventType::kContextMenu,
            EventType::kMouseEnter,
            EventType::kMouseLeave,
            EventType::kMouseOver,
            EventType::kMouseOut,
            EventType::kPointerMove,
            EventType::kPointerDown,
            EventType::kPointerUp,
            EventType::kPointerCancel,
            EventType::kPointerEnter,
            EventType::kPointerLeave,
            EventType::kWheel,
            EventType::kKeyDown,
            EventType::kKeyPress,
            EventType::kKeyUp,
            EventType::kCompositionStart,
            EventType::kCompositionUpdate,
            EventType::kCompositionEnd,
            EventType::kTextInput,
            EventType::kBeforeInput,
            EventType::kInput,
            EventType::kChange,
            EventType::kFocus,
            EventType::kFocusIn,
            EventType::kBlur,
            EventType::kFocusOut,
            EventType::kSubmit,
            EventType::kReset,
            EventType::kDOMContentLoaded,
            EventType::kReadyStateChange,
            EventType::kLoad,
        ] {
            if kind == EventTypeName(candidate) {
                event.r#type = candidate;
                break;
            }
        }
        let state = self.services.synthetic.clone();
        let handle = handle.clone();
        let event_state = self.services.event_state.clone();
        Box::new(move |runtime| {
            let previous = state.borrow_mut().replace(SyntheticDispatch {
                identity: event.identity,
                handle: handle.clone(),
                target: receiver,
            });
            dispatch(&mut event, receiver, runtime);
            let update = event_state.borrow().clone();
            if let Some(update) = update {
                runtime.Call(
                    &update,
                    &HostValue::Object(HostObjectRef { id: 0 }),
                    &[
                        handle,
                        event
                            .dispatch_state
                            .borrow()
                            .final_target
                            .map_or(HostValue::Null(JavaScriptNull), |id| {
                                HostValue::Object(HostObjectRef { id })
                            }),
                        HostValue::Null(JavaScriptNull),
                        HostValue::Number(0.0),
                        HostValue::Boolean(false),
                    ],
                );
            }
            *state.borrow_mut() = previous;
            Self::value(HostValue::Boolean(!event.default_prevented))
        })
    }

    // cpp: webapi/dom_bindings.cc:319-351
    pub(super) fn GetBoundEvent(&self, receiver: HostObjectId, member: &str) -> Option<HostResult> {
        let events = self.services.events.borrow();
        let bound = events.records.get(&receiver)?;
        let event = &bound.event;
        if let Some(native) = event.canonical.as_ref() {
            let owner = native.borrow();
            let base = owner.BaseEvent();
            let value = match member {
                "type" => Some(HostValue::String(base.r#type().into())),
                "bubbles" => Some(HostValue::Boolean(base.bubbles())),
                "cancelable" => Some(HostValue::Boolean(base.cancelable())),
                "composed" => Some(HostValue::Boolean(base.composed())),
                "isTrusted" => Some(HostValue::Boolean(base.isTrusted())),
                "defaultPrevented" => Some(HostValue::Boolean(base.defaultPrevented())),
                _ => None,
            };
            if let Some(value) = value {
                return Some(Self::value(value));
            }
            if let Some(input) = owner.AsInputEvent() {
                if member == "view" {
                    return Some(self.ProjectInputEventView(receiver, input.view()));
                }
                let value = match member {
                    "data" => Some(
                        input
                            .dataUTF16()
                            .map_or(HostValue::Null(JavaScriptNull), HostValue::String16),
                    ),
                    "inputType" => Some(HostValue::String(input.inputType().into())),
                    "isComposing" => Some(HostValue::Boolean(input.isComposing())),
                    "detail" => Some(HostValue::Number(input.detail() as f64)),
                    "sourceCapabilities" => Some(input.sourceCapabilities().map_or(
                        HostValue::Null(JavaScriptNull),
                        |capabilities| {
                            self.native_abort
                                .InputCapabilitiesValue(receiver, capabilities)
                        },
                    )),
                    "dataTransfer" => Some(
                        input
                            .dataTransfer()
                            .map_or(HostValue::Null(JavaScriptNull), |transfer| {
                                self.native_abort.InputDataTransferValue(receiver, transfer)
                            }),
                    ),
                    _ => None,
                };
                if let Some(value) = value {
                    return Some(Self::value(value));
                }
            }
            if let Some(composition) = owner.AsCompositionEvent() {
                if member == "data" {
                    return Some(Self::value(HostValue::String16(composition.dataUTF16())));
                }
            }
        }
        let value = match member {
            "submitter" => event
                .submitter_node_id
                .map_or(HostValue::Null(JavaScriptNull), |id| {
                    HostValue::Object(HostObjectRef { id })
                }),
            "preventDefault" | "stopPropagation" | "stopImmediatePropagation" => {
                HostValue::Method(HostMethodRef {
                    receiver,
                    name: member.into(),
                })
            }
            "type" => HostValue::String(if event.custom_type.is_empty() {
                EventTypeName(event.r#type).into()
            } else {
                event.custom_type.clone()
            }),
            "target" => {
                let state = event.dispatch_state.borrow();
                if state.finished {
                    state
                        .final_target
                        .map_or(HostValue::Null(JavaScriptNull), |id| {
                            HostValue::Object(HostObjectRef { id })
                        })
                } else {
                    HostValue::Object(HostObjectRef { id: bound.target })
                }
            }
            "currentTarget" => {
                if bound.dispatching {
                    HostValue::Object(HostObjectRef {
                        id: bound.current_target,
                    })
                } else {
                    HostValue::Null(JavaScriptNull)
                }
            }
            "eventPhase" => HostValue::Number(bound.phase as u8 as f64),
            "bubbles" => HostValue::Boolean(event.bubbles),
            "cancelable" => HostValue::Boolean(event.cancelable),
            "composed" => HostValue::Boolean(event.composed),
            "isTrusted" => HostValue::Boolean(event.trusted),
            "defaultPrevented" => HostValue::Boolean(event.default_prevented),
            "key" => HostValue::String(event.key.clone()),
            "code" => HostValue::String(event.code.clone()),
            "location" => HostValue::Number(event.key_location as f64),
            "keyCode" | "which" => HostValue::Number(event.key_code as f64),
            "charCode" => HostValue::Number(event.char_code as f64),
            "isComposing" => HostValue::Boolean(event.is_composing),
            "data" => HostValue::String(event.text.clone()),
            "inputType" => HostValue::String(event.input_type.clone()),
            "button" => HostValue::Number(match event.button {
                MouseButton::kNone => -1.0,
                MouseButton::kPrimary => 0.0,
                MouseButton::kMiddle => 1.0,
                MouseButton::kSecondary => 2.0,
            }),
            "clientX" => HostValue::Number(bound.client_x),
            "clientY" => HostValue::Number(bound.client_y),
            "deltaX" => HostValue::Number(bound.delta_x),
            "deltaY" => HostValue::Number(bound.delta_y),
            "altKey" => HostValue::Boolean(event.modifiers.alt),
            "ctrlKey" => HostValue::Boolean(event.modifiers.control),
            "metaKey" => HostValue::Boolean(event.modifiers.meta),
            "shiftKey" => HostValue::Boolean(event.modifiers.shift),
            "repeat" => HostValue::Boolean(event.repeat),
            _ => return Some(Self::unhandled()),
        };
        Some(Self::value(value))
    }

    pub(super) fn CallDOMService(
        &mut self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostResult> {
        if let Some(result) = self.CallInputEventService(receiver, member, arguments) {
            return Some(result);
        }
        if member == "__eventBrand" {
            return Some(if self.services.IsEvent(receiver) {
                Self::value(HostValue::Boolean(true))
            } else {
                Self::type_error("argument is not an Event")
            });
        }
        if member == "__eventInit" {
            let [HostValue::String(kind), HostValue::Boolean(bubbles), HostValue::Boolean(cancelable), ..] =
                arguments
            else {
                return Some(Self::type_error("Event init conversion required"));
            };
            let mut events = self.services.events.borrow_mut();
            let Some(bound) = events.records.get_mut(&receiver) else {
                return Some(Self::type_error("Illegal Event invocation"));
            };
            let Some(native) = bound.event.canonical.as_ref() else {
                return Some(Self::type_error("Event has no native owner"));
            };
            if native.borrow().BaseEvent().IsBeingDispatched() {
                return Some(HostResult::default());
            }
            native
                .borrow_mut()
                .BaseEventMut()
                .initEvent(kind, *bubbles, *cancelable);
            bound.event.custom_type = kind.clone();
            bound.event.bubbles = *bubbles;
            bound.event.cancelable = *cancelable;
            bound.event.default_prevented = false;
            bound.event.trusted = false;
            bound.stop_propagation = false;
            bound.stop_immediate_propagation = false;
            return Some(HostResult::default());
        }
        if receiver == 0 && member == "__documentDomainGet" {
            let owner = self.document.borrow();
            let origin = owner.GetDocument().SecurityContext().Origin();
            return Some(Self::value(HostValue::String(origin.Domain())));
        }
        if receiver == 0 && member == "__documentDomainSet" {
            let Some(HostValue::String(domain)) = arguments.first() else {
                return Some(Self::type_error("document.domain requires a string"));
            };
            self.document
                .borrow()
                .GetDocument()
                .SecurityContext()
                .Origin()
                .SetDomainFromDOM(domain.clone());
            return Some(HostResult::default());
        }
        if member == "__fontStatus" {
            let is_document = {
                let owner = self.document.borrow();
                let document = owner.GetDocument();
                document
                    .FindNodeById(receiver)
                    .is_some_and(|index| document.Node(index).Type() == DOMNodeType::kDocument)
            };
            if !is_document || !arguments.is_empty() {
                return Some(Self::type_error("Illegal invocation"));
            }
            let loading = self
                .services
                .host
                .read_font_loading
                .as_mut()
                .is_some_and(|read| read());
            return Some(Self::value(HostValue::String(
                if loading { "loading" } else { "loaded" }.into(),
            )));
        }
        if member == "__eventAttribute" {
            let Some(HostValue::String(name)) = arguments.first() else {
                return Some(Self::type_error("Illegal invocation"));
            };
            if !matches!(
                name.as_str(),
                "target"
                    | "currentTarget"
                    | "eventPhase"
                    | "type"
                    | "bubbles"
                    | "cancelable"
                    | "composed"
                    | "isTrusted"
                    | "defaultPrevented"
                    | "timeStamp"
            ) || !self.services.IsEvent(receiver)
            {
                return Some(Self::type_error("Illegal invocation"));
            }
            return self.GetBoundEvent(receiver, name);
        }
        if member == "__eventComposedPath" {
            let events = self.services.events.borrow();
            let Some(bound) = events.records.get(&receiver) else {
                return Some(Self::type_error("Illegal invocation"));
            };
            return Some(Self::value(if bound.dispatching {
                event_path_value(&bound.event, bound.current_target)
            } else {
                HostValue::Record(vec![("length".into(), HostValue::Number(0.0))])
            }));
        }
        if member == "__mutationObserverId" {
            let id = self
                .document
                .borrow_mut()
                .GetDocumentMut()
                .AllocateMutationObserverId();
            return Some(Self::value(HostValue::String(id.0.to_string())));
        }
        if member == "__mutationObserve" {
            let [HostValue::String(id), HostValue::Object(target), HostValue::Boolean(child_list), HostValue::Boolean(attributes), HostValue::Boolean(character_data), HostValue::Boolean(subtree), HostValue::Boolean(attribute_old_value), HostValue::Boolean(character_data_old_value), HostValue::Boolean(has_filter), filter @ ..] =
                arguments
            else {
                return Some(Self::type_error("Invalid mutation observer registration"));
            };
            let Some(id) = id.parse::<u64>().ok().filter(|id| *id != 0) else {
                return Some(Self::type_error("Invalid mutation observer identity"));
            };
            let mut names = std::collections::HashSet::new();
            for name in filter {
                let HostValue::String(name) = name else {
                    return Some(Self::type_error(
                        "Invalid mutation observer attribute filter",
                    ));
                };
                names.insert(name.clone());
            }
            let mut owner = self.document.borrow_mut();
            let tree = owner.GetDocumentMut();
            let Some(target) = tree.FindNodeById(target.id) else {
                return Some(Self::type_error("Mutation observer target must be a Node"));
            };
            let registered = tree.RegisterMutationObserver(
                dom::mutation_observers::MutationObserverId(id),
                target,
                dom::mutation_observers::MutationObserverOptions {
                    child_list: *child_list,
                    attributes: *attributes,
                    character_data: *character_data,
                    subtree: *subtree,
                    attribute_old_value: *attribute_old_value,
                    character_data_old_value: *character_data_old_value,
                    attribute_filter: has_filter.then_some(names),
                },
            );
            return Some(if registered {
                HostResult::default()
            } else {
                Self::type_error("Invalid mutation observer identity")
            });
        }
        if member == "__mutationDisconnect" {
            let [HostValue::String(id)] = arguments else {
                return Some(Self::type_error("Invalid mutation observer identity"));
            };
            let Ok(id) = id.parse::<u64>() else {
                return Some(Self::type_error("Invalid mutation observer identity"));
            };
            self.native_mutation_observers
                .Disconnect(dom::mutation_observers::MutationObserverId(id));
            return Some(HostResult::default());
        }
        if member == "__mutationTakeRecords" {
            let [HostValue::String(id)] = arguments else {
                return Some(Self::type_error("Invalid mutation observer identity"));
            };
            let Ok(id) = id.parse::<u64>() else {
                return Some(Self::type_error("Invalid mutation observer identity"));
            };
            return Some(Self::value(
                self.native_mutation_observers
                    .TakeRecordsValue(self, dom::mutation_observers::MutationObserverId(id)),
            ));
        }
        if member == "__mutationEmptyNodeList" {
            return Some(Self::value(HostValue::Object(
                self.StaticNodeList(Vec::new()),
            )));
        }
        // cpp: webapi/dom_bindings.cc:562-569
        if member == "animationSample" {
            let mut trace = browser_tracing::span("animation", "AnimationSampleHost");
            let [HostValue::Number(effect), HostValue::Number(time), HostValue::String(text)] =
                arguments
            else {
                return Some(Self::type_error("Invalid animation sample"));
            };
            if self.node(receiver).is_none() {
                return Some(Self::type_error("Invalid animation sample"));
            }
            let declarations = style::ParseCSSDeclarationList(text);
            trace.set("declarations", declarations.len() as f64);
            if let Some(sample) = self.services.host.sample_animation.as_mut() {
                sample(receiver, *effect as u64, *time, declarations);
            }
            return Some(HostResult::default());
        }
        if member == "dispatchEvent" {
            // Without an active-runtime continuation, Source rejects dispatch.
            return Some(Self::type_error("Invalid event dispatch"));
        }
        // cpp: webapi/dom_bindings.cc:685-699
        if let Some(bound) = self.services.events.borrow_mut().records.get_mut(&receiver) {
            match member {
                "preventDefault" => {
                    if bound.event.cancelable && !bound.passive_listener {
                        bound.event.default_prevented = true;
                    }
                }
                "stopPropagation" => bound.stop_propagation = true,
                "stopImmediatePropagation" => {
                    bound.stop_propagation = true;
                    bound.stop_immediate_propagation = true;
                }
                _ => return Some(Self::type_error("unknown Event method")),
            }
            if let Some(native) = bound.event.canonical.as_ref() {
                // Event methods update the source owner synchronously, before
                // the author callback can reenter dispatch or default actions.
                let mut owner = native.borrow_mut();
                let event = owner.BaseEventMut();
                match member {
                    "preventDefault" => event.preventDefault(),
                    "stopPropagation" => event.stopPropagation(),
                    "stopImmediatePropagation" => event.stopImmediatePropagation(),
                    _ => unreachable!(),
                }
            }
            return Some(HostResult::default());
        }
        // cpp: webapi/dom_bindings.cc:570-608
        if member == "scrollOffset" {
            let Some(node) = self.node(receiver) else {
                return Some(Self::type_error("Scrolling requires an element"));
            };
            let Some(HostValue::String(axis)) = arguments.first() else {
                return Some(Self::type_error("Invalid scroll axis"));
            };
            // Element::scrollTop/scrollLeft synchronously update layout before
            // reading its canonical offset (element.cc:2678-2690). The existing
            // native metric boundary performs that measurement/overflow clamp.
            if let Some(read) = self.services.host.read_metric.as_mut() {
                let _ = read(
                    receiver,
                    if axis == "x" {
                        "minimumScrollLeft"
                    } else {
                        "minimumScrollTop"
                    },
                );
            }
            let mut offset = self.document.borrow().GetDocument().ScrollOffsetFor(node);
            let zoom = self
                .document
                .borrow()
                .GetDocument()
                .ResolvedStyleFor(node)
                .and_then(|resolved| resolved.ComputedStyle())
                .map_or(1.0, |style| style.EffectiveZoom() as f64);
            if let Some(HostValue::Number(value)) = arguments.get(1) {
                let mut value = if value.is_finite() { *value } else { 0.0 };
                if let Some(read) = self.services.host.read_metric.as_mut() {
                    let (minimum, maximum) = if axis == "x" {
                        ("minimumScrollLeft", "maximumScrollLeft")
                    } else {
                        ("minimumScrollTop", "maximumScrollTop")
                    };
                    let minimum = read(receiver, minimum).unwrap_or(0.0);
                    let maximum = read(receiver, maximum).unwrap_or(minimum).max(minimum);
                    value = value.clamp(minimum, maximum);
                }
                if axis == "x" {
                    offset.x = value * zoom
                } else {
                    offset.y = value * zoom
                }
                if let Some(write) = self.services.host.write_scroll.as_mut() {
                    write(receiver, offset)
                } else {
                    self.document
                        .borrow_mut()
                        .GetDocumentMut()
                        .SetScrollOffset(node, offset)
                }
            }
            return Some(Self::value(HostValue::Number(if axis == "x" {
                offset.x / zoom
            } else {
                offset.y / zoom
            })));
        }
        if member == "metric" {
            let value = match (arguments.first(), self.services.host.read_metric.as_mut()) {
                (Some(HostValue::String(name)), Some(read)) => read(receiver, name).unwrap_or(0.0),
                _ => 0.0,
            };
            return Some(Self::value(HostValue::Number(value)));
        }
        // cpp: webapi/dom_bindings.cc:611-621
        if member == "clientRects" {
            let rects = self
                .services
                .host
                .read_geometry
                .as_mut()
                .map(|read| read(receiver))
                .unwrap_or_default();
            // These are internal numeric snapshots consumed immediately by
            // the unchanged WebIDL DOMRect wrapper. Keep them GC-owned instead
            // of registering fresh permanent host records/proxies on each read.
            let mut fields = Vec::with_capacity(rects.len() + 1);
            fields.push(("length".into(), HostValue::Number(rects.len() as f64)));
            for (index, rect) in rects.into_iter().enumerate() {
                fields.push((
                    index.to_string(),
                    HostValue::Record(vec![
                        ("x".into(), HostValue::Number(rect.x)),
                        ("y".into(), HostValue::Number(rect.y)),
                        ("width".into(), HostValue::Number(rect.width)),
                        ("height".into(), HostValue::Number(rect.height)),
                    ]),
                ));
            }
            return Some(Self::value(HostValue::Record(fields)));
        }
        if member != "computedStyle" {
            return None;
        }
        let (Some(node), Some(HostValue::String(property))) =
            (self.node(receiver), arguments.first())
        else {
            return Some(Self::type_error("Computed style requires an element"));
        };
        let mut trace = browser_tracing::span("dom", "DOM.ReadComputedStyle");
        let layout_dependent = matches!(property.as_str(), "width" | "height");
        trace.set("layout_dependent", layout_dependent as u8 as f64);
        if let Some(update) = &mut self.services.host.update_style {
            update(receiver);
        }
        // CSSComputedStyleDeclaration::GetPropertyCSSValue updates style and
        // any required layout before borrowing the current computed style.
        // No document borrow may survive a reentrant measurement callback.
        let metric = if layout_dependent {
            let (has_box, border_box) = {
                let document = self.document.borrow();
                let Some(resolved) = document.GetDocument().ResolvedStyleFor(node) else {
                    return Some(Self::value(HostValue::String(String::new())));
                };
                // CSSOM resolved width/height use computed lengths for
                // display:none/contents, including SVG hidden layout containers.
                let has_box = resolved.generates_box
                    && unsafe { resolved.computed_style_owner.Get().as_ref() }.is_none_or(
                        |style| {
                            !matches!(
                                style.Display(),
                                foundation::EDisplay::kNone | foundation::EDisplay::kContents
                            )
                        },
                    );
                (
                    has_box,
                    unsafe { resolved.computed_style_owner.Get().as_ref() }.is_some_and(|style| {
                        style.BoxSizing() == foundation::EBoxSizing::kBorderBox
                    }),
                )
            };
            if has_box {
                let name = match (property.as_str(), border_box) {
                    ("width", true) => "borderBoxWidth",
                    ("height", true) => "borderBoxHeight",
                    ("width", false) => "contentWidth",
                    _ => "contentHeight",
                };
                self.services
                    .host
                    .read_metric
                    .as_mut()
                    .and_then(|read| read(receiver, name))
            } else {
                None
            }
        } else {
            None
        };
        let _serialization_trace = browser_tracing::span("dom", "DOM.SerializeComputedStyle");
        let document = self.document.borrow();
        let resolved = document.GetDocument().ResolvedStyleFor(node);
        let Some(resolved) = resolved else {
            return Some(Self::value(HostValue::String(String::new())));
        };
        let Some(s) = resolved.ComputedStyle() else {
            return Some(Self::value(HostValue::String(String::new())));
        };
        let number = |n: f64| {
            if n.is_nan() {
                if n.is_sign_negative() {
                    "-nan".into()
                } else {
                    "nan".into()
                }
            } else if n.is_infinite() {
                if n.is_sign_negative() {
                    "-inf".into()
                } else {
                    "inf".into()
                }
            } else {
                dom_number_string(n)
            }
        };
        let px = |n: f64| format!("{}px", number(n));
        // Source: core/css/properties/longhands/longhands_custom.cc
        // Color::CSSValueFromComputedStyleInternal. CSSOM exposes the
        // resolved current color, not the authored token (which may be an
        // inherited value, currentColor or a custom-property substitution).
        let serialize_color = |color: foundation::Color| {
            use foundation::ColorSpace;
            if foundation::Color::IsLegacyColorSpace(color.GetColorSpace()) {
                let (red, green, blue) = color.ToSRGB();
                let channel = |value: f32| ((value + 1e-7).clamp(0.0, 1.0) * 255.0).round() as i32;
                let red = channel(red);
                let green = channel(green);
                let blue = channel(blue);
                if color.IsOpaque() {
                    format!("rgb({red}, {green}, {blue})")
                } else {
                    // CSSOM serializes legacy alpha through its 8-bit value,
                    // choosing the shortest exact two- or three-decimal form.
                    let alpha = ((color.Alpha() + 1e-7) * 255.0).round().clamp(0.0, 255.0);
                    let two = (alpha * 100.0 / 255.0).round() / 100.0;
                    let serialized = if (two * 255.0).round() == alpha {
                        two
                    } else {
                        (alpha * 1000.0 / 255.0).round() / 1000.0
                    };
                    format!(
                        "rgba({red}, {green}, {blue}, {})",
                        dom_number_string(serialized as f64)
                    )
                }
            } else {
                let name = match color.GetColorSpace() {
                    ColorSpace::kSRGB => "srgb",
                    ColorSpace::kSRGBLinear => "srgb-linear",
                    ColorSpace::kDisplayP3 => "display-p3",
                    ColorSpace::kDisplayP3Linear => "display-p3-linear",
                    ColorSpace::kA98RGB => "a98-rgb",
                    ColorSpace::kProPhotoRGB => "prophoto-rgb",
                    ColorSpace::kRec2020 => "rec2020",
                    ColorSpace::kRec2100Linear => "rec2100-linear",
                    ColorSpace::kXYZD50 => "xyz-d50",
                    ColorSpace::kXYZD65 => "xyz-d65",
                    ColorSpace::kLab => "lab",
                    ColorSpace::kOklab => "oklab",
                    ColorSpace::kLch => "lch",
                    ColorSpace::kOklch => "oklch",
                    ColorSpace::kNone => "srgb",
                    _ => unreachable!("legacy colors were handled above"),
                };
                let component = |value: f32, is_none: bool| {
                    is_none
                        .then(|| "none".into())
                        .unwrap_or_else(|| dom_number_string(value as f64))
                };
                let values = format!(
                    "{} {} {}",
                    component(color.Param0(), color.Param0IsNone()),
                    component(color.Param1(), color.Param1IsNone()),
                    component(color.Param2(), color.Param2IsNone())
                );
                let alpha = if color.IsOpaque() && !color.AlphaIsNone() {
                    String::new()
                } else {
                    format!(" / {}", component(color.Alpha(), color.AlphaIsNone()))
                };
                if foundation::Color::IsLightnessFirstComponent(color.GetColorSpace()) {
                    format!("{name}({values}{alpha})")
                } else {
                    format!("color({name} {values}{alpha})")
                }
            }
        };
        let value = match property.as_str() {
            "display" => if resolved.display_contents {
                "contents"
            } else if !resolved.generates_box {
                "none"
            } else {
                match s.Display() {
                    foundation::EDisplay::kInline => "inline",
                    foundation::EDisplay::kBlock => "block",
                    foundation::EDisplay::kListItem
                    | foundation::EDisplay::kInlineListItem
                    | foundation::EDisplay::kFlowRootListItem
                    | foundation::EDisplay::kInlineFlowRootListItem => "list-item",
                    foundation::EDisplay::kInlineBlock => "inline-block",
                    foundation::EDisplay::kTable => "table",
                    foundation::EDisplay::kInlineTable => "inline-table",
                    foundation::EDisplay::kTableRowGroup => "table-row-group",
                    foundation::EDisplay::kTableHeaderGroup => "table-header-group",
                    foundation::EDisplay::kTableFooterGroup => "table-footer-group",
                    foundation::EDisplay::kTableRow => "table-row",
                    foundation::EDisplay::kTableColumnGroup => "table-column-group",
                    foundation::EDisplay::kTableColumn => "table-column",
                    foundation::EDisplay::kTableCell => "table-cell",
                    foundation::EDisplay::kTableCaption => "table-caption",
                    foundation::EDisplay::kWebkitBox | foundation::EDisplay::kFlex => "flex",
                    foundation::EDisplay::kWebkitInlineBox | foundation::EDisplay::kInlineFlex => {
                        "inline-flex"
                    }
                    foundation::EDisplay::kGrid => "grid",
                    foundation::EDisplay::kInlineGrid => "inline-grid",
                    foundation::EDisplay::kFlowRoot => "flow-root",
                    foundation::EDisplay::kMath => "math",
                    foundation::EDisplay::kBlockMath => "block math",
                    foundation::EDisplay::kRuby => "ruby",
                    foundation::EDisplay::kBlockRuby => "block ruby",
                    foundation::EDisplay::kRubyText => "ruby-text",
                    foundation::EDisplay::kGridLanes => "grid-lanes",
                    foundation::EDisplay::kInlineGridLanes => "inline grid-lanes",
                    foundation::EDisplay::kLayoutCustom => "block",
                    foundation::EDisplay::kInlineLayoutCustom => "inline",
                    foundation::EDisplay::kContents => "contents",
                    foundation::EDisplay::kNone => "none",
                }
            }
            .into(),
            "position" => match s.GetPosition() {
                foundation::EPosition::kStatic => "static",
                foundation::EPosition::kRelative => "relative",
                foundation::EPosition::kAbsolute => "absolute",
                foundation::EPosition::kFixed => "fixed",
                foundation::EPosition::kSticky => "sticky",
            }
            .into(),
            "direction" => if s.Direction() == foundation::TextDirection::kRtl {
                "rtl"
            } else {
                "ltr"
            }
            .into(),
            "visibility" => match s.Visibility() {
                foundation::style_constants::EVisibility::kVisible => "visible",
                foundation::style_constants::EVisibility::kHidden => "hidden",
                foundation::style_constants::EVisibility::kCollapse => "collapse",
            }
            .into(),
            "box-sizing" => if s.BoxSizing() == foundation::EBoxSizing::kBorderBox {
                "border-box"
            } else {
                "content-box"
            }
            .into(),
            "opacity" => number(s.Opacity() as f64),
            "color" => serialize_color(s.GetCurrentColor(None)),
            "zoom" => number(s.Zoom() as f64),
            "font-size" => px(s.GetFontDescription().SpecifiedSize() as f64),
            "font-weight" => number(s.GetFontDescription().Weight().ToFloat() as f64),
            "font-style" => if s.GetFontDescription().Style().ToFloat() != 0.0 {
                "italic"
            } else {
                "normal"
            }
            .into(),
            "line-height" => {
                style::production_css_value::Value::Create(s.LineHeight(), s.EffectiveZoom())
                    .CssText()
                    .Utf8()
            }
            // CSSComputedStyleDeclaration serializes the resolved inherited
            // alignment.  Hydration libraries use this value when replacing
            // one text node with block line wrappers; returning the empty
            // string makes them freeze the wrappers at their `left` fallback
            // even though Layout used the parent's centered alignment.
            "text-align" => match s.GetTextAlign() {
                foundation::ETextAlign::kStart => "start",
                foundation::ETextAlign::kEnd => "end",
                foundation::ETextAlign::kLeft => "left",
                foundation::ETextAlign::kRight => "right",
                foundation::ETextAlign::kCenter => "center",
                foundation::ETextAlign::kJustify => "justify",
                foundation::ETextAlign::kMatchParent => "match-parent",
                foundation::ETextAlign::kWebkitLeft => "-webkit-left",
                foundation::ETextAlign::kWebkitRight => "-webkit-right",
                foundation::ETextAlign::kWebkitCenter => "-webkit-center",
            }
            .into(),
            "text-align-last" => match s.TextAlignLast() {
                foundation::ETextAlignLast::kAuto => "auto",
                foundation::ETextAlignLast::kStart => "start",
                foundation::ETextAlignLast::kEnd => "end",
                foundation::ETextAlignLast::kLeft => "left",
                foundation::ETextAlignLast::kRight => "right",
                foundation::ETextAlignLast::kCenter => "center",
                foundation::ETextAlignLast::kJustify => "justify",
                foundation::ETextAlignLast::kMatchParent => "match-parent",
            }
            .into(),
            "width" | "height" => metric.map(px).unwrap_or_else(|| {
                // Width/Height::CSSValueFromComputedStyleInternal uses the
                // computed Length when no layout object supplies a used size.
                let length = if property == "width" {
                    s.Width()
                } else {
                    s.Height()
                };
                style::production_css_value::Value::Create(length, s.EffectiveZoom())
                    .CssText()
                    .Utf8()
            }),
            "min-width" | "min-height" | "max-width" | "max-height" => {
                // Source: MinWidth/MinHeight/MaxWidth/MaxHeight::
                // CSSValueFromComputedStyleInternal. Unlike width/height these
                // properties always expose the computed Length; there is no
                // layout-object used-size substitution.
                let length = match property.as_str() {
                    "min-width" => s.MinWidth(),
                    "min-height" => s.MinHeight(),
                    "max-width" => s.MaxWidth(),
                    "max-height" => s.MaxHeight(),
                    _ => unreachable!(),
                };
                if length.IsAuto() && !(s.IsFlexOrGridOrCustomItem() && !s.IsEnsuredInDisplayNone())
                {
                    // ComputedStyleUtils::MinWidthOrMinHeightAuto resolves the
                    // ordinary block-item initial value to a used zero. Flex,
                    // grid and custom-layout items retain the `auto` keyword.
                    "0px".into()
                } else if length.IsNone() {
                    "none".into()
                } else {
                    style::production_css_value::Value::Create(length, s.EffectiveZoom())
                        .CssText()
                        .Utf8()
                }
            }
            p if p.starts_with("padding-")
                || p.starts_with("margin-")
                || p.starts_with("border-") =>
            {
                let side = p.split_once('-').unwrap().1;
                let side = side.strip_suffix("-width").unwrap_or(side);
                if p.starts_with("border-") {
                    let width = match side {
                        "top" => s.BorderTopWidth(),
                        "right" => s.BorderRightWidth(),
                        "bottom" => s.BorderBottomWidth(),
                        "left" => s.BorderLeftWidth(),
                        _ => return Some(Self::value(HostValue::String(String::new()))),
                    };
                    px(width as f64 / s.EffectiveZoom().max(f32::MIN_POSITIVE) as f64)
                } else {
                    let length = match (p.starts_with("padding-"), side) {
                        (true, "top") => s.PaddingTop(),
                        (true, "right") => s.PaddingRight(),
                        (true, "bottom") => s.PaddingBottom(),
                        (true, "left") => s.PaddingLeft(),
                        (false, "top") => s.MarginTop(),
                        (false, "right") => s.MarginRight(),
                        (false, "bottom") => s.MarginBottom(),
                        (false, "left") => s.MarginLeft(),
                        _ => return Some(Self::value(HostValue::String(String::new()))),
                    };
                    style::production_css_value::Value::Create(length, s.EffectiveZoom())
                        .CssText()
                        .Utf8()
                }
            }
            p if p.starts_with("--") => resolved
                .custom_properties
                .get(p)
                .and_then(|v| v.clone())
                .unwrap_or_default(),
            _ => String::new(),
        };
        Some(Self::value(HostValue::String(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::dom_mutation::ApplyDOMMutations;
    use dom::persistent_document::ResolvedNodeStyle;
    use interaction::event::{EventType, MakeSyntheticEvent};
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
    use layoutng_assembly::internal::layout_input::{
        Display, ExtendedStyle, Position, TextDirection,
    };

    fn check(runtime: &mut dyn JavaScriptRuntime, realm: &JavaScriptRealm, source: &str) {
        let result = runtime.Evaluate(realm, source, "services-test.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
    fn native_click(document: Rc<RefCell<DOM>>, target: HostObjectId) -> Event {
        use interaction::adapter::event::{EventProjection, EventProjectionOptions};
        use interaction::dispatch::core::dom::events::event::PhaseType;
        let mut event = MakeSyntheticEvent(EventType::kClick, target);
        let projection = EventProjection::new(
            &event,
            interaction::DomEventTargets::new(document),
            &EventProjectionOptions::default(),
        )
        .unwrap();
        let native = projection.EventRef();
        native
            .borrow_mut()
            .BaseEventMut()
            .SetEventPhase(PhaseType::kAtTarget);
        event.canonical = Some(native);
        event
    }
    #[test]
    fn listener_snapshot_reentry_once_passive_errors_and_event_lifetime() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<p id=target></p>",
        )));
        let changed = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |m| {
                ApplyDOMMutations(
                    changed.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(m),
                )
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        check(&mut runtime, &realm, crate::DOMBootstrapSource());
        check(
            &mut runtime,
            &realm,
            r#"
            function check(v,s){if(!v)throw Error(s)}
            var log=[],saved, target=document.getElementById('target');
            function removed(){log.push('removed')}
            function added(){log.push('added')}
            function first(e){saved=e;check(this===target && e.target===target && e.currentTarget===target && e.eventPhase===2 && e.type==='click','event target');log.push('first');target.removeEventListener('click',removed);target.addEventListener('click',added);target.textContent='changed';e.preventDefault();check(!e.defaultPrevented,'passive');}
            target.addEventListener('click',first,{once:true,passive:true});
            target.addEventListener('click',first,{once:false});
            target.addEventListener('click',removed);
            target.onclick=function(e){log.push('attribute');e.preventDefault();};
            target.addEventListener('click',function(){log.push('error');throw Error('listener failed')});
            target.addEventListener('click',function(e){log.push('last');e.stopImmediatePropagation()});
            target.addEventListener('click',function(){log.push('unreachable')});
        "#,
        );
        let target = match bindings
            .borrow_mut()
            .call(
                document
                    .borrow()
                    .GetDocument()
                    .Node(document.borrow().GetDocument().Root())
                    .Id(),
                "getElementById",
                &[HostValue::String("target".into())],
            )
            .value
        {
            HostValue::Object(o) => o.id,
            _ => panic!("target"),
        };
        let event = native_click(document.clone(), target);
        let invocation = EventListenerInvocation {
            event: &event,
            target_node_id: target,
            current_target_node_id: target,
            phase: EventPhase::kAtTarget,
            capture_listeners: false,
            focused_node_id: None,
        };
        let mut errors = Vec::new();
        let result = DOMJavaScriptBindings::DispatchEventListeners(
            &bindings,
            &invocation,
            &mut runtime,
            &realm,
            &mut |e| errors.push(e.clone()),
        );
        assert!(
            result.prevent_default && result.stop_propagation && result.stop_immediate_propagation
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("listener failed"));
        check(
            &mut runtime,
            &realm,
            "check(JSON.stringify(log)===JSON.stringify(['first','attribute','error','last']),'snapshot order');check(target.textContent==='changed' && saved.currentTarget===null && saved.eventPhase===0 && saved.defaultPrevented,'retained event');target.removeEventListener('click',added);",
        );
        let event = native_click(document.clone(), target);
        let invocation = EventListenerInvocation {
            event: &event,
            ..invocation
        };
        let result = DOMJavaScriptBindings::DispatchEventListeners(
            &bindings,
            &invocation,
            &mut runtime,
            &realm,
            &mut |_| {},
        );
        assert!(result.prevent_default);
        check(
            &mut runtime,
            &realm,
            "check(JSON.stringify(log)===JSON.stringify(['first','attribute','error','last','attribute','error','last']),'once and removal');target.onclick=null;check(target.onclick===null,'attribute cleared');",
        );
    }

    #[test]
    fn listener_registration_liveness_preserves_readd_and_attribute_once() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<p id=target></p>",
        )));
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(|_| {}),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        check(&mut runtime, &realm, crate::DOMBootstrapSource());
        check(
            &mut runtime,
            &realm,
            r#"
            var log=[], target=document.getElementById('target');
            function removed(){log.push('removed')}
            function attribute(){log.push('attribute')}
            target.addEventListener('click',function(){
                log.push('first');
                target.removeEventListener('click',removed);
                target.addEventListener('click',removed);
            },{once:true});
            target.addEventListener('click',removed);
            target.addEventListener('click',attribute,{once:true});
            target.onclick=attribute;
        "#,
        );
        let target = {
            let owner = document.borrow();
            let tree = owner.GetDocument();
            tree.Node(super::super::find_element_by_id(tree, tree.Root(), "target").unwrap())
                .Id()
        };
        for expected in [
            "first,attribute,attribute",
            "first,attribute,attribute,attribute,removed",
        ] {
            let event = native_click(document.clone(), target);
            let invocation = EventListenerInvocation {
                event: &event,
                target_node_id: target,
                current_target_node_id: target,
                phase: EventPhase::kAtTarget,
                capture_listeners: false,
                focused_node_id: None,
            };
            DOMJavaScriptBindings::DispatchEventListeners(
                &bindings,
                &invocation,
                &mut runtime,
                &realm,
                &mut |e| panic!("{e:?}"),
            );
            check(
                &mut runtime,
                &realm,
                &format!("if(log.join(',')!=={expected:?})throw Error(log.join(','));"),
            );
        }
    }

    #[test]
    fn computed_style_uses_live_resolved_document_and_measurement_callbacks() {
        let _heap = foundation::LayoutHeapScope::new();
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<div id=box data-info='one'></div>",
        )));
        let node = {
            let tree = document.borrow();
            let tree = tree.GetDocument();
            super::super::find_element_by_id(tree, tree.Root(), "box").unwrap()
        };
        let id = document.borrow().GetDocument().Node(node).Id();
        let update_document = document.clone();
        let measure_document = document.clone();
        let updated = Rc::new(Cell::new(0));
        let count = updated.clone();
        let host = DOMBindingsHost {
            update_style: Some(Box::new(move |_| {
                count.set(count.get() + 1);
                let mut resolved = ResolvedNodeStyle::default();
                resolved.style.display = Display::kInlineFlex;
                resolved.style.position = Position::kSticky;
                resolved.style.direction = TextDirection::kRtl;
                resolved.style.paint.visible = false;
                resolved.style.paint.opacity = 0.25;
                resolved.style.paint.color = foundation::PaintColor {
                    red: 18.0 / 255.0,
                    green: 52.0 / 255.0,
                    blue: 86.0 / 255.0,
                    alpha: 128.0 / 255.0,
                };
                resolved.style.padding.left = 2.125;
                resolved.style.border.top = 1.5;
                resolved.style.extended = Some(
                    ExtendedStyle {
                        font_size: 20.0,
                        font_weight: 700.0,
                        font_italic: true,
                        line_height_percent: Some(150.0),
                        text_align: layoutng_assembly::internal::layout_input::TextAlign::kCenter,
                        text_align_last:
                            layoutng_assembly::internal::layout_input::TextAlignLast::kRight,
                        ..Default::default()
                    }
                    .into(),
                );
                let initial = unsafe {
                    &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton(
                    )
                };
                let mut builder =
                    layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(
                        initial,
                    );
                builder.SetMinHeightOwned(foundation::Length::MinContent().clone());
                builder.SetMinWidthOwned(foundation::Length::Fixed(13.5));
                builder.SetMaxWidthOwned(foundation::Length::Percent(75.0));
                builder.SetMaxHeightOwned(foundation::Length::None().clone());
                resolved
                    .SetComputedStyleOwner(foundation::Persistent::from_ptr(builder.TakeStyle()
                        as *mut layoutng_style::style::computed_style::ComputedStyle));
                std::sync::Arc::make_mut(&mut resolved.custom_properties)
                    .insert("--value".into(), Some(" green ".into()));
                update_document
                    .borrow_mut()
                    .GetDocumentMut()
                    .SetResolvedStyle(node, resolved);
            })),
            read_metric: Some(Box::new(move |target, name| {
                assert_eq!(target, id);
                // Reenter the document to verify no read guard survives a call.
                let tree = &mut *measure_document.borrow_mut();
                assert!(tree.GetDocumentMut().FindNodeById(target).is_some());
                Some(match name {
                    "contentWidth" => 121.234567,
                    "offsetWidth" => 128.0,
                    _ => 0.0,
                })
            })),
            ..Default::default()
        };
        let changed = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document,
            Box::new(move |m| {
                ApplyDOMMutations(
                    changed.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(m),
                )
            }),
            host,
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        check(&mut runtime, &realm, crate::DOMBootstrapSource());
        check(
            &mut runtime,
            &realm,
            r#"
            function check(v,s){if(!v)throw Error(s)}
            var box=document.getElementById('box'),s=getComputedStyle(box);
            check(s.display==='inline-flex' && s.position==='sticky' && s.direction==='rtl' && s.visibility==='hidden','resolved style');
            check(s.opacity==='0.25' && s.fontSize==='20px' && s.fontWeight==='700' && s.fontStyle==='italic' && s.lineHeight==='30px','font/paint values');
            check(s.textAlign==='center' && s.textAlignLast==='right','text alignment');
            check(s.minHeight==='min-content' && s.minWidth==='13.5px' && s.maxWidth==='75%' && s.maxHeight==='none','min/max size serialization');
            // Text splitters preserve alignment by copying the resolved value
            // onto the generated block line wrapper.  Keep this hydration path
            // from silently taking its `left` fallback.
            var line=document.createElement('span');
            line.style.textAlign=s.textAlign||'left';
            check(line.style.textAlign==='center','hydrated line alignment');
            check(s.width==='121.235px' && s.paddingLeft==='2.125px' && s.borderTopWidth==='1.5px' && s.getPropertyValue('--value')===' green ' && s.color==='rgba(18, 52, 86, 0.5)','source serialization');
            check(box.offsetWidth===128,'metric');
            var attrs=box.attributes;check(attrs.length===2 && attrs[0].name==='id' && attrs[0].value==='box' && attrs[1].name==='data-info' && attrs[1].namespaceURI===null,'attribute snapshot');
            box.setAttribute('data-info','two');check(attrs[1].value==='one' && box.attributes[1].value==='two','new attribute snapshot');
            check(new XMLSerializer().serializeToString(box)==='<div id="box" data-info="two"></div>','serializeNode');
        "#,
        );
        assert!(updated.get() >= 10);
    }
}

#[cfg(test)]
mod geometry_snapshot_tests {
    use super::*;
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
    #[test]
    fn client_rects_keep_fresh_public_snapshots_and_overrides_without_permanent_host_records() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<div id=box></div>",
        )));
        let reads = Rc::new(Cell::new(0));
        let read_count = reads.clone();
        let host = DOMBindingsHost {
            read_geometry: Some(Box::new(move |_| {
                let index = read_count.get();
                read_count.set(index + 1);
                if index == 0 {
                    return vec![];
                }
                vec![
                    paint::PaintRect {
                        x: index as f64 + 0.125,
                        y: -2.5,
                        width: -4.25,
                        height: 5.5,
                    },
                    paint::PaintRect {
                        x: 7.0,
                        y: 8.0,
                        width: 0.0,
                        height: 0.0,
                    },
                ]
            })),
            ..Default::default()
        };
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document,
            Box::new(|_| {}),
            host,
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings.clone());
        assert!(runtime
            .Evaluate(&realm, crate::DOMBootstrapSource(), "browser:dom-webidl")
            .Succeeded());
        let records_before = bindings.borrow().records.borrow().len();
        let next_id_before = bindings.borrow().next_collection_id.get();
        let result=runtime.Evaluate(&realm,r#"
            function check(v,m){if(!v)throw Error(m)}
            const box=document.getElementById('box');
            const empty=box.getClientRects();check(empty.length===0 && empty.item(0)===null,'empty list');
            const first=box.getClientRects(),second=box.getClientRects();
            check(Array.isArray(first) && first!==second && first[0]!==second[0],'fresh public list and rect');
            check(first[0] instanceof DOMRect && first[0] instanceof DOMRectReadOnly,'unchanged DOMRect prototypes');
            check(first[0].x===1.125 && first[0].left===-3.125 && first[0].right===1.125 && first.item(0)===first[0] && first.item(9)===null,'snapshot values and item identity');
            first[0].x=123;check(second[0].x===2.125,'independent old snapshot');
            const fromRect=DOMRect.fromRect;let converted=0;
            DOMRect.fromRect=function(rect){converted++;return fromRect.call(this,rect)};
            const third=box.getClientRects();check(converted===2 && third[0].x===3.125,'fromRect override still invoked');
            DOMRect.fromRect=fromRect;
            const original=box.getClientRects;
            box.getClientRects=()=>[new DOMRect(5,6,7,8)];
            const custom=box.getBoundingClientRect();check(custom.x===5 && custom.y===6 && custom.width===7 && custom.height===8,'existing getClientRects override retained');
            delete box.getClientRects;
            for(let i=0;i<32;i++) {
                const list=original.call(box);check(list.length===2 && list[0] instanceof DOMRect,'repeated actual geometry');
            }
        "#,"geometry-snapshot-test.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert_eq!(
            reads.get(),
            36,
            "every actual public query still calls real geometry service"
        );
        assert_eq!(
            bindings.borrow().records.borrow().len(),
            records_before,
            "temporary geometry never enters permanent records registry"
        );
        assert_eq!(
            bindings.borrow().next_collection_id.get(),
            next_id_before,
            "temporary geometry never allocates persistent host IDs"
        );
    }
}

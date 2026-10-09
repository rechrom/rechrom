use super::*;
use interaction::event::{
    Event, EventListenerInvocation, EventListenerResult, EventPhase, EventType, EventTypeName,
};
use interaction::input_event::{InputEvent, MouseButton};
use javascript::javascript_runtime::{
    JavaScriptException, JavaScriptHostRuntime, JavaScriptRealm, JavaScriptResult,
    JavaScriptRuntime,
};

// Rust ownership adapter for the callbacks in DOMBindingsHost.
// Page must supply real resolution and measurement; absent callbacks retain the
// source's default behavior and do not imply that a Page has been implemented.
pub type SyntheticEventHandler =
    Rc<dyn Fn(&mut Event, HostObjectId, &mut dyn JavaScriptHostRuntime)>;

#[derive(Default)]
pub struct DOMBindingsHost {
    pub dispatch_synthetic: Option<SyntheticEventHandler>,
    pub submit_form: Option<
        Rc<dyn Fn(HostObjectId, Option<HostObjectId>, bool, &mut dyn JavaScriptHostRuntime)>,
    >,
    pub sample_animation:
        Option<Box<dyn FnMut(HostObjectId, u64, f64, Vec<cssom::CSSDeclaration>)>>,
    pub update_style: Option<Box<dyn FnMut()>>,
    pub emit_style_sheet: Option<Box<dyn FnMut(cssom::CSSStyleSheet)>>,
    pub read_metric: Option<Box<dyn FnMut(HostObjectId, &str) -> f64>>,
    pub read_geometry: Option<Box<dyn FnMut(HostObjectId) -> Vec<paint::PaintRect>>>,
    pub write_scroll:
        Option<Box<dyn FnMut(HostObjectId, layoutng_assembly::internal::layout_input::Offset)>>,
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

// cpp: webapi/dom_bindings.h:139-141
// Action flags share ownership with the active host continuation, avoiding an
// aliased mutable Event reference while Interaction lends it to a listener.
#[derive(Clone)]
struct SyntheticDispatch {
    identity: u64,
    handle: HostValue,
    cancelable: bool,
    passive: bool,
    flags: EventListenerResult,
}

pub(super) struct DOMServices {
    synthetic: Rc<RefCell<Option<SyntheticDispatch>>>,
    // The final helper is looked up after listeners, and listeners may replace
    // it through __domWrap. Share this slot with the detached continuation.
    pub(super) event_state: Rc<RefCell<Option<JavaScriptFunction>>>,
    pub(super) host: DOMBindingsHost,
    events: HashMap<HostObjectId, BoundEvent>,
    event_wrappers: HashMap<u64, HostObjectId>,
    next_event_object_id: HostObjectId,
}
impl DOMServices {
    pub fn new(host: DOMBindingsHost) -> Self {
        Self {
            host,
            synthetic: Rc::new(RefCell::new(None)),
            event_state: Rc::new(RefCell::new(None)),
            events: HashMap::new(),
            event_wrappers: HashMap::new(),
            next_event_object_id: 1 << 63,
        }
    }
    pub fn IsEvent(&self, id: HostObjectId) -> bool {
        self.events.contains_key(&id)
    }
}

impl DOMJavaScriptBindings {
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
        Self::DispatchEventListenersWithCall(
            bindings,
            invocation,
            &mut |callback, receiver, args| runtime.Call(realm, callback, receiver, args),
            report_error,
        )
    }

    pub fn DispatchEventListenersScoped(
        bindings: &Rc<RefCell<Self>>,
        invocation: &EventListenerInvocation<'_>,
        runtime: &mut dyn JavaScriptHostRuntime,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> EventListenerResult {
        Self::DispatchEventListenersWithCall(
            bindings,
            invocation,
            &mut |callback, receiver, args| runtime.Call(callback, receiver, args),
            report_error,
        )
    }

    fn DispatchEventListenersWithCall(
        bindings: &Rc<RefCell<Self>>,
        invocation: &EventListenerInvocation<'_>,
        call: &mut dyn FnMut(&JavaScriptFunction, &HostValue, &[HostValue]) -> JavaScriptResult,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> EventListenerResult {
        let (matching, event_id) = {
            let mut this = bindings.borrow_mut();
            this.focused_node = invocation.focused_node_id;
            let kind = if invocation.event.custom_type.is_empty() {
                EventTypeName(invocation.event.r#type)
            } else {
                &invocation.event.custom_type
            };
            let matching: Vec<Listener> = this
                .listeners
                .borrow()
                .get(&invocation.current_target_node_id)
                .and_then(|kinds| kinds.get(kind))
                .into_iter()
                .flatten()
                .filter(|l| l.capture == invocation.capture_listeners)
                .cloned()
                .collect();
            if matching.is_empty() {
                return EventListenerResult::default();
            }
            let services = &mut this.services;
            let id = if let Some(&id) = services.event_wrappers.get(&invocation.event.identity) {
                id
            } else {
                let id = services.next_event_object_id;
                services.next_event_object_id += 1;
                services
                    .event_wrappers
                    .insert(invocation.event.identity, id);
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
            services.events.insert(
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
            (matching, id)
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
        for listener in matching {
            {
                let mut this = bindings.borrow_mut();
                // A removed registration stays removed in every in-flight
                // snapshot, including nested dispatch and remove+readd.
                if !listener.active.get() {
                    continue;
                }
                if listener.once {
                    listener.active.set(false);
                    let mut targets = this.listeners.borrow_mut();
                    if let Some(kinds) = targets.get_mut(&listener.target) {
                        if let Some(listeners) = kinds.get_mut(&listener.kind) {
                            listeners.retain(|l| !Rc::ptr_eq(&l.active, &listener.active));
                            if listeners.is_empty() {
                                kinds.remove(&listener.kind);
                            }
                        }
                        if kinds.is_empty() {
                            targets.remove(&listener.target);
                        }
                    }
                }
                this.services
                    .events
                    .get_mut(&event_id)
                    .unwrap()
                    .passive_listener = listener.passive;
            }
            if let Some((state, _)) = &synthetic {
                state.borrow_mut().as_mut().unwrap().passive = listener.passive;
                let update = bindings
                    .borrow()
                    .method_wrappers
                    .get("__eventState")
                    .cloned();
                if let Some(update) = update {
                    call(
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
                            HostValue::Boolean(listener.passive),
                        ],
                    );
                }
            }
            // Attribute event handlers cancel the event on an exact false
            // return. Keep value coercion in the realm, independent of the
            // embedding JavaScript engine's opaque result representation.
            let handler_wrapper = if listener.attribute {
                bindings
                    .borrow()
                    .method_wrappers
                    .get("__callEventHandler")
                    .cloned()
            } else {
                None
            };
            let receiver = HostValue::Object(HostObjectRef {
                id: invocation.current_target_node_id,
            });
            let result = if let Some(wrapper) = handler_wrapper {
                call(
                    &wrapper,
                    &receiver,
                    &[
                        HostValue::JavaScriptFunction(listener.callback.clone()),
                        argument.clone(),
                    ],
                )
            } else {
                call(&listener.callback, &receiver, &[argument.clone()])
            };
            if let Some(error) = result.exception {
                report_error(&error);
            }
            if let Some((state, _)) = &synthetic {
                let flags = state.borrow().as_ref().unwrap().flags;
                let mut this = bindings.borrow_mut();
                let bound = this.services.events.get_mut(&event_id).unwrap();
                bound.event.default_prevented =
                    flags.prevent_default || invocation.event.default_prevented;
                bound.stop_propagation =
                    flags.stop_propagation || invocation.event.propagation_stopped;
                bound.stop_immediate_propagation = flags.stop_immediate_propagation
                    || invocation.event.immediate_propagation_stopped;
            }
            if bindings.borrow().services.events[&event_id].stop_immediate_propagation {
                break;
            }
        }
        let mut this = bindings.borrow_mut();
        let bound = this.services.events.get_mut(&event_id).unwrap();
        bound.dispatching = false;
        bound.phase = EventPhase::kNone;
        EventListenerResult {
            prevent_default: bound.event.default_prevented,
            stop_propagation: bound.stop_propagation,
            stop_immediate_propagation: bound.stop_immediate_propagation,
        }
    }

    pub fn SetSyntheticEventHandler(&mut self, handler: Option<SyntheticEventHandler>) {
        self.services.host.dispatch_synthetic = handler;
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
        &self,
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
        let Some(dispatch) = self.services.host.dispatch_synthetic.clone() else {
            return invalid();
        };
        if !self.runtime_realm.IsValid() {
            return invalid();
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
            let passive = state.borrow().as_ref().is_some_and(|active| active.passive);
            let previous = state.borrow_mut().replace(SyntheticDispatch {
                identity: event.identity,
                handle: handle.clone(),
                cancelable: event.cancelable,
                // Source preserves the prior listener's passive flag until
                // entering the first listener of this dispatch.
                passive,
                flags: EventListenerResult::default(),
            });
            dispatch(&mut event, receiver, runtime);
            let update = event_state.borrow().clone();
            if let Some(update) = update {
                runtime.Call(
                    &update,
                    &HostValue::Object(HostObjectRef { id: 0 }),
                    &[
                        handle,
                        HostValue::Object(HostObjectRef { id: receiver }),
                        HostValue::Null(JavaScriptNull),
                        HostValue::Number(0.0),
                        HostValue::Boolean(false),
                    ],
                );
            }
            let flags = state.borrow().as_ref().unwrap().flags;
            *state.borrow_mut() = previous;
            Self::value(HostValue::Boolean(
                !(event.default_prevented || flags.prevent_default),
            ))
        })
    }

    // cpp: webapi/dom_bindings.cc:319-351
    pub(super) fn GetBoundEvent(&self, receiver: HostObjectId, member: &str) -> Option<HostResult> {
        let bound = self.services.events.get(&receiver)?;
        let event = &bound.event;
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
            "target" => HostValue::Object(HostObjectRef { id: bound.target }),
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
        if member == "mutationObserverState" {
            let [HostValue::Number(count)] = arguments else {
                return Some(Self::type_error("Invalid mutation observer state"));
            };
            if !count.is_finite() || *count < 0.0 {
                return Some(Self::type_error("Invalid mutation observer count"));
            }
            self.mutation_observers_active = *count > 0.0;
            return Some(HostResult::default());
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
        // cpp: webapi/dom_bindings.cc:622-632
        if receiver == 0 && member == "__eventAction" {
            if let (Some(active), Some(HostValue::String(action))) = (
                self.services.synthetic.borrow_mut().as_mut(),
                arguments.first(),
            ) {
                if action == "prevent" && active.cancelable && !active.passive {
                    active.flags.prevent_default = true;
                }
                if action == "stop" || action == "immediate" {
                    active.flags.stop_propagation = true;
                }
                if action == "immediate" {
                    active.flags.stop_immediate_propagation = true;
                }
            }
            return Some(HostResult::default());
        }
        if member == "dispatchEvent" {
            // Without an active-runtime continuation, Source rejects dispatch.
            return Some(Self::type_error("Invalid event dispatch"));
        }
        // cpp: webapi/dom_bindings.cc:685-699
        if let Some(bound) = self.services.events.get_mut(&receiver) {
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
            let mut offset = self.document.borrow().GetDocument().ScrollOffsetFor(node);
            let zoom = self
                .document
                .borrow()
                .GetDocument()
                .ResolvedStyleFor(node)
                .and_then(|r| r.style.extended.as_ref())
                .map_or(1.0, |e| e.effective_zoom as f64);
            if let Some(HostValue::Number(value)) = arguments.get(1) {
                let mut value = if value.is_finite() { *value } else { 0.0 };
                if let Some(read) = self.services.host.read_metric.as_mut() {
                    let (scroll, client) = if axis == "x" {
                        ("scrollWidth", "clientWidth")
                    } else {
                        ("scrollHeight", "clientHeight")
                    };
                    value = value.clamp(
                        0.0,
                        (read(receiver, scroll) - read(receiver, client)).max(0.0),
                    );
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
                (Some(HostValue::String(name)), Some(read)) => read(receiver, name),
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
        if let Some(update) = &mut self.services.host.update_style {
            update();
        }
        // No document borrow survives a measurement callback.
        let resolved = self
            .document
            .borrow()
            .GetDocument()
            .ResolvedStyleFor(node)
            .cloned();
        let Some(resolved) = resolved else {
            return Some(Self::value(HostValue::String(String::new())));
        };
        let s = &resolved.style;
        let initial = layoutng_assembly::internal::layout_input::ExtendedStyle::default();
        let e = s.extended.as_ref().unwrap_or(&initial);
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
        let value = match property.as_str() {
            "display" => {
                const NAMES: [&str; 26] = [
                    "block",
                    "flex",
                    "inline-flex",
                    "grid",
                    "inline-grid",
                    "inline",
                    "table",
                    "inline-table",
                    "table-row",
                    "table-cell",
                    "flow-root",
                    "inline-block",
                    "table-row-group",
                    "table-header-group",
                    "table-footer-group",
                    "table-caption",
                    "table-column-group",
                    "table-column",
                    "grid-lanes",
                    "block",
                    "list-item",
                    "math",
                    "block math",
                    "ruby",
                    "block ruby",
                    "ruby-text",
                ];
                if !resolved.generates_box {
                    "none"
                } else if resolved.display_contents {
                    "contents"
                } else {
                    NAMES[s.display as usize]
                }
                .into()
            }
            "position" => {
                ["static", "absolute", "relative", "fixed", "sticky"][s.position as usize].into()
            }
            "direction" => {
                if s.direction == layoutng_assembly::internal::layout_input::TextDirection::kRtl {
                    "rtl"
                } else {
                    "ltr"
                }
                .into()
            }
            "visibility" => if s.paint.visible { "visible" } else { "hidden" }.into(),
            "opacity" => number(s.paint.opacity as f64),
            "zoom" => number(e.zoom as f64),
            "font-size" => px(e.font_size),
            "font-weight" => number(e.font_weight as f64),
            "font-style" => if e.font_italic { "italic" } else { "normal" }.into(),
            "line-height" => e
                .line_height
                .map(px)
                .or_else(|| e.line_height_percent.map(|v| px(e.font_size * v / 100.0)))
                .unwrap_or_else(|| "normal".into()),
            "width" | "height" => self
                .services
                .host
                .read_metric
                .as_mut()
                .map(|read| {
                    px(read(
                        receiver,
                        if property == "width" {
                            "contentWidth"
                        } else {
                            "contentHeight"
                        },
                    ))
                })
                .unwrap_or_else(|| "auto".into()),
            p if p.starts_with("padding-")
                || p.starts_with("margin-")
                || p.starts_with("border-") =>
            {
                let edges = if p.starts_with("padding-") {
                    &s.padding
                } else if p.starts_with("margin-") {
                    &s.margin
                } else {
                    &s.border
                };
                let side = p.split_once('-').unwrap().1;
                match side.strip_suffix("-width").unwrap_or(side) {
                    "top" => px(edges.top),
                    "right" => px(edges.right),
                    "bottom" => px(edges.bottom),
                    "left" => px(edges.left),
                    _ => String::new(),
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
        let event = MakeSyntheticEvent(EventType::kClick, target);
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
        check(&mut runtime,&realm,"check(JSON.stringify(log)===JSON.stringify(['first','attribute','error','last']),'snapshot order');check(target.textContent==='changed' && saved.currentTarget===null && saved.eventPhase===0 && saved.defaultPrevented,'retained event');target.removeEventListener('click',added);");
        let result = DOMJavaScriptBindings::DispatchEventListeners(
            &bindings,
            &invocation,
            &mut runtime,
            &realm,
            &mut |_| {},
        );
        assert!(result.prevent_default);
        check(&mut runtime,&realm,"check(JSON.stringify(log)===JSON.stringify(['first','attribute','error','last','attribute','error','last']),'once and removal');target.onclick=null;check(target.onclick===null,'attribute cleared');");
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
            let event = MakeSyntheticEvent(EventType::kClick, target);
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
            update_style: Some(Box::new(move || {
                count.set(count.get() + 1);
                let mut resolved = ResolvedNodeStyle::default();
                resolved.style.display = Display::kInlineFlex;
                resolved.style.position = Position::kSticky;
                resolved.style.direction = TextDirection::kRtl;
                resolved.style.paint.visible = false;
                resolved.style.paint.opacity = 0.25;
                resolved.style.padding.left = 2.125;
                resolved.style.border.top = 1.5;
                resolved.style.extended = Some(ExtendedStyle {
                    font_size: 20.0,
                    font_weight: 700.0,
                    font_italic: true,
                    line_height_percent: Some(150.0),
                    ..Default::default()
                });
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
                match name {
                    "contentWidth" => 121.234567,
                    "offsetWidth" => 128.0,
                    _ => 0.0,
                }
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
            check(s.width==='121.235px' && s.paddingLeft==='2.125px' && s.borderTopWidth==='1.5px' && s.getPropertyValue('--value')===' green ' && s.color==='','source serialization');
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

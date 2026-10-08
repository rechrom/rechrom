use crate::{
    event::{Event, EventListenerDispatcher, EventType, MakeEvent},
    event_dispatcher::EventDispatcher,
    focus_controller::FocusController,
    input_event::*,
    input_type::{IsDisabledFormControl, IsTextField},
    ownership::{InteractionDOMMutationEmitter, InteractionDocument, IsElement, ReadNode},
    text_editor::{SyntheticEventDispatcher, TextEditor},
};
use dom::{persistent_document::DOMNodeType, Document, UserInteractionState};
use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::layout_input::{Display, Offset},
};
use page_mutation::{InteractionStateMutation, PageMutation, PageMutationEmitter};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
// cpp: interaction/interaction_engine.h:23-29
pub type InteractionState = UserInteractionState;
#[derive(Clone, Copy, Default, Debug)]
pub struct InteractionResult {
    pub target_node_id: Option<u64>,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
}
// Map a root-space point through the exact same transform and scroll state as paint.
fn LocalPoint(fragment: &FragmentNode, point: Offset, parent: Offset) -> Option<(Offset, Offset)> {
    let absolute = Offset {
        x: parent.x + fragment.offset.x,
        y: parent.y + fragment.offset.y,
    };
    let mut point = point;
    // Match ResolveFragmentGeometry's local paint-property order. Only the
    // paint-state owner contributes these transforms; text/line fragments can
    // borrow its style without establishing another transform node.
    if fragment.paint.establishes_paint_state
        && fragment.paint.has_source
        && fragment.paint.position == layoutng_assembly::internal::layout_input::Position::kSticky
    {
        point.x -= fragment.paint.sticky_offset.x;
        point.y -= fragment.paint.sticky_offset.y;
    }
    if fragment.paint.establishes_paint_state
        && fragment.paint.has_source
        && fragment.paint.style.transform.is_some()
    {
        let transform = fragment.paint.style.transform.as_ref().unwrap();
        let matrix = paint::geometry_mapper::ResolveTransformAroundOrigin(
            transform,
            absolute,
            fragment.size,
            fragment.paint.style.transform_origin.as_ref(),
        );
        point = paint::geometry_mapper::MapPointWithTransform(point, &matrix, true)?;
    }
    if fragment.paint.establishes_paint_state {
        if let Some(transform) = fragment
            .paint
            .svg_shape
            .as_ref()
            .and_then(|shape| shape.local_transform.as_ref())
        {
            point = paint::geometry_mapper::MapPointWithTransform(point, transform, true)?;
        }
    }
    Some((point, absolute))
}
fn ChildOrigin(fragment: &FragmentNode, absolute: Offset) -> Offset {
    if fragment.paint.establishes_paint_state {
        Offset {
            x: absolute.x - fragment.paint.scroll_offset.x,
            y: absolute.y - fragment.paint.scroll_offset.y,
        }
    } else {
        absolute
    }
}
fn HitGeometry(
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
) -> Option<(Offset, Offset, bool)> {
    if fragment.paint.hidden {
        return None;
    }
    let (point, absolute) = LocalPoint(fragment, point, parent)?;
    let inside = point.x >= absolute.x
        && point.y >= absolute.y
        && point.x < absolute.x + fragment.size.width
        && point.y < absolute.y + fragment.size.height;
    use layoutng_assembly::internal::layout_input::Overflow;
    if (!inside && fragment.paint.scroll_container.is_some())
        || (matches!(
            fragment.paint.overflow_x,
            Overflow::kHidden | Overflow::kClip | Overflow::kScroll | Overflow::kAuto
        ) && (point.x < absolute.x || point.x >= absolute.x + fragment.size.width))
        || (matches!(
            fragment.paint.overflow_y,
            Overflow::kHidden | Overflow::kClip | Overflow::kScroll | Overflow::kAuto
        ) && (point.y < absolute.y || point.y >= absolute.y + fragment.size.height))
    {
        return None;
    }
    Some((point, absolute, inside))
}

#[derive(Clone, Copy)]
struct HitLayer<'a> {
    fragment: &'a FragmentNode,
    point: Offset,
    parent: Offset,
    classification: paint::paint_order::PaintLayerClassification,
}

fn ChildrenAreFlexOrGridItems(fragment: &FragmentNode) -> bool {
    fragment.paint.has_source
        && matches!(
            fragment.paint.display,
            Display::kFlex | Display::kGrid | Display::kGridLanes
        )
}

// PaintLayerPainter flattens non-stacking layers into their nearest stacking
// context and stable-sorts the result by stacking level. Preserve the physical
// geometry path with each candidate so hit testing can walk that order in
// reverse without teaching interaction how paint layers are classified.
fn CollectHitLayers<'a>(
    fragment: &'a FragmentNode,
    point: Offset,
    parent: Offset,
    root_point: Offset,
    output: &mut Vec<HitLayer<'a>>,
) {
    let Some((point, absolute, _)) = HitGeometry(fragment, point, parent) else {
        return;
    };
    let children_are_flex_or_grid_items = ChildrenAreFlexOrGridItems(fragment);
    for child in &fragment.children {
        let (child_point, child_parent) = if child.paint.fixed_to_view {
            (root_point, Offset::default())
        } else {
            (point, ChildOrigin(fragment, absolute))
        };
        let classification = paint::paint_order::ClassifyFragmentPaintLayer(
            child,
            false,
            children_are_flex_or_grid_items,
        );
        if classification.is_paint_layer {
            output.push(HitLayer {
                fragment: child,
                point: child_point,
                parent: child_parent,
                classification,
            });
            if !classification.is_stacking_context {
                CollectHitLayers(child, child_point, child_parent, root_point, output);
            }
        } else {
            CollectHitLayers(child, child_point, child_parent, root_point, output);
        }
    }
}

fn HitTestNormalFlow(
    document: &Document,
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
    root_point: Offset,
    include_self: bool,
) -> Option<u64> {
    let (point, absolute, inside) = HitGeometry(fragment, point, parent)?;
    let children_are_flex_or_grid_items = ChildrenAreFlexOrGridItems(fragment);
    for child in fragment.children.iter().rev() {
        if paint::paint_order::ClassifyFragmentPaintLayer(
            child,
            false,
            children_are_flex_or_grid_items,
        )
        .is_paint_layer
        {
            continue;
        }
        let (child_point, child_parent) = if child.paint.fixed_to_view {
            (root_point, Offset::default())
        } else {
            (point, ChildOrigin(fragment, absolute))
        };
        if let Some(target) =
            HitTestNormalFlow(document, child, child_point, child_parent, root_point, true)
        {
            return Some(target);
        }
    }
    (include_self
        && inside
        && !fragment.paint.style.pointer_events_none
        && fragment.paint.style.visible
        && document.FindNodeById(fragment.node_id).is_some())
    .then_some(fragment.node_id)
}

fn HitTestPaintLayer(document: &Document, layer: &HitLayer<'_>, root_point: Offset) -> Option<u64> {
    if layer.classification.is_stacking_context {
        HitTestStackingContext(
            document,
            layer.fragment,
            layer.point,
            layer.parent,
            root_point,
        )
    } else {
        HitTestNormalFlow(
            document,
            layer.fragment,
            layer.point,
            layer.parent,
            root_point,
            true,
        )
    }
}

fn HitTestStackingContext(
    document: &Document,
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
    root_point: Offset,
) -> Option<u64> {
    let (_, _, inside) = HitGeometry(fragment, point, parent)?;
    let mut layers = Vec::new();
    CollectHitLayers(fragment, point, parent, root_point, &mut layers);
    // Stable sorting preserves physical order for equal z-index, matching
    // PaintLayerPainter::CollectLayerChildren. Hit test consumes it backwards.
    layers.sort_by_key(|layer| layer.classification.stacking_level);
    let negative_count = layers.partition_point(|layer| layer.classification.stacking_level < 0);
    for layer in layers[negative_count..].iter().rev() {
        if let Some(target) = HitTestPaintLayer(document, layer, root_point) {
            return Some(target);
        }
    }
    if let Some(target) = HitTestNormalFlow(document, fragment, point, parent, root_point, false) {
        return Some(target);
    }
    for layer in layers[..negative_count].iter().rev() {
        if let Some(target) = HitTestPaintLayer(document, layer, root_point) {
            return Some(target);
        }
    }
    (inside
        && !fragment.paint.style.pointer_events_none
        && fragment.paint.style.visible
        && document.FindNodeById(fragment.node_id).is_some())
    .then_some(fragment.node_id)
}

fn HitTest(
    document: &Document,
    fragment: &FragmentNode,
    point: Offset,
    parent: Offset,
    root_point: Offset,
) -> Option<u64> {
    HitTestStackingContext(document, fragment, point, parent, root_point)
}
fn ControlPoint(
    fragment: &FragmentNode,
    id: u64,
    point: Offset,
    parent: Offset,
    root_point: Offset,
) -> Option<(&FragmentNode, Offset)> {
    let (point, absolute) = LocalPoint(fragment, point, parent)?;
    if fragment.node_id == id && fragment.paint.text_control_caret_metrics.is_some() {
        return Some((
            fragment,
            Offset {
                x: point.x - absolute.x,
                y: point.y - absolute.y,
            },
        ));
    }
    for child in &fragment.children {
        let (child_point, child_parent) = if child.paint.fixed_to_view {
            (root_point, Offset::default())
        } else {
            (point, ChildOrigin(fragment, absolute))
        };
        if let Some(found) = ControlPoint(child, id, child_point, child_parent, root_point) {
            return Some(found);
        }
    }
    None
}
// cpp: interaction/interaction_engine.cc:38-45
fn EventElement(document: &InteractionDocument, target: Option<u64>) -> Option<u64> {
    let owner = document.borrow();
    let d = owner.GetDocument();
    let mut node = target.and_then(|id| d.FindNodeById(id));
    while let Some(i) = node {
        let n = d.Node(i);
        if n.Type() == DOMNodeType::kElement {
            return Some(n.Id());
        }
        node = n.Parent()
    }
    None
}

fn ElementAncestorPath(document: &InteractionDocument, target: u64) -> Vec<u64> {
    ReadNode(document, target, |d, _, mut i| {
        let mut path = Vec::new();
        loop {
            let node = d.Node(i);
            if node.Type() == DOMNodeType::kElement {
                path.push(node.Id());
            }
            let Some(parent) = node.Parent() else { break };
            i = parent;
        }
        path
    })
    .unwrap_or_default()
}

// Chromium's BoundaryEventDispatcher freezes the old/new ancestor chains,
// then sends out, child-to-parent leave, over, and parent-to-child enter.
fn DispatchMouseBoundaryEvents(
    dispatcher: &EventDispatcher<'_>,
    document: &InteractionDocument,
    input: &InputEvent,
    exited: Option<u64>,
    entered: Option<u64>,
) {
    if exited == entered {
        return;
    }
    let exited_path = exited.map_or_else(Vec::new, |id| ElementAncestorPath(document, id));
    let entered_path = entered.map_or_else(Vec::new, |id| ElementAncestorPath(document, id));
    let mut exited_common = exited_path.len();
    let mut entered_common = entered_path.len();
    while exited_common > 0
        && entered_common > 0
        && exited_path[exited_common - 1] == entered_path[entered_common - 1]
    {
        exited_common -= 1;
        entered_common -= 1;
    }
    let dispatch = |kind: EventType, target: u64, related: Option<u64>, bubbles: bool| {
        let mut event = Event {
            r#type: kind,
            underlying_event: Some(input.clone()),
            related_target_node_id: related,
            bubbles,
            cancelable: bubbles,
            ..Default::default()
        };
        dispatcher.Dispatch(&mut event, target);
    };
    if let Some(target) = exited {
        dispatch(EventType::kMouseOut, target, entered, true);
    }
    for &target in &exited_path[..exited_common] {
        dispatch(EventType::kMouseLeave, target, entered, false);
    }
    if let Some(target) = entered {
        dispatch(EventType::kMouseOver, target, exited, true);
    }
    for &target in entered_path[..entered_common].iter().rev() {
        dispatch(EventType::kMouseEnter, target, exited, false);
    }
}
// cpp: interaction/interaction_engine.cc:47-79
fn FocusElement(document: &InteractionDocument, target: u64) -> Option<u64> {
    ReadNode(document, target, |d, n, i| {
        if !IsElement(n, "label") {
            return Some(target);
        }
        fn find(d: &Document, i: usize, id: &str) -> Option<u64> {
            let n = d.Node(i);
            if n.Type() == DOMNodeType::kElement
                && n.FindAttribute("id").is_some_and(|a| a.value == id)
            {
                return Some(n.Id());
            }
            for &child in n.Children() {
                if let Some(found) = find(d, child, id) {
                    return Some(found);
                }
            }
            None
        }
        if let Some(id) = n.FindAttribute("for") {
            return find(d, d.Root(), &id.value);
        }
        fn first(d: &Document, i: usize) -> Option<u64> {
            for &child in d.Node(i).Children() {
                let n = d.Node(child);
                if ["input", "textarea", "select", "button"]
                    .iter()
                    .any(|name| IsElement(n, name))
                {
                    return Some(n.Id());
                }
                if let Some(found) = first(d, child) {
                    return Some(found);
                }
            }
            None
        }
        first(d, i)
    })
    .flatten()
}
// cpp: interaction/interaction_engine.cc:82-85
fn IsFocusableControl(document: &InteractionDocument, id: u64) -> bool {
    ReadNode(document, id, |_, n, _| {
        ["input", "textarea", "select", "button"]
            .iter()
            .any(|name| IsElement(n, name))
    })
    .unwrap_or(false)
}
// cpp: interaction/interaction_engine.cc:88-118
fn ExplicitTarget(input: &InputEvent) -> Option<u64> {
    match input {
        InputEvent::Focus(v) => (v.target_node_id != 0).then_some(v.target_node_id),
        InputEvent::Mouse(v) => v.target_node_id,
        InputEvent::Pointer(v) => v.target_node_id,
        InputEvent::Wheel(v) => v.target_node_id,
        InputEvent::Key(v) => v.target_node_id,
        InputEvent::Composition(v) => v.target_node_id,
        InputEvent::TextInput(v) => v.target_node_id,
    }
}
fn EventPosition(input: &InputEvent) -> Option<Offset> {
    match input {
        InputEvent::Mouse(v) => Some(v.position),
        InputEvent::Pointer(v) => Some(v.position),
        InputEvent::Wheel(v) => Some(v.position),
        _ => None,
    }
}
// cpp: interaction/interaction_engine.h:36-56
// Clones retain the same input state/editor and listener slot. This permits
// source synchronous listener reentry without an exclusive Interaction borrow.
#[derive(Clone)]
pub struct Interaction<'a> {
    emit: PageMutationEmitter,
    listeners: Rc<RefCell<Option<EventListenerDispatcher<'a>>>>,
    state: Rc<RefCell<InteractionState>>,
    editor: Rc<TextEditor>,
    selection_drag: Rc<Cell<Option<u64>>>,
    submit_form: Rc<RefCell<Option<Rc<dyn Fn(u64, Option<u64>)>>>>,
    submitting_forms: Rc<RefCell<Vec<u64>>>,
}
// cpp: interaction/interaction_engine.cc:149-161
struct EmitStateMutationAtExit {
    before: InteractionState,
    current: Rc<RefCell<InteractionState>>,
    emit: PageMutationEmitter,
}
impl Drop for EmitStateMutationAtExit {
    fn drop(&mut self) {
        let state = *self.current.borrow();
        if state != self.before {
            (self.emit)(PageMutation::InteractionStateMutation(
                InteractionStateMutation { state },
            ))
        }
    }
}
impl<'a> Interaction<'a> {
    // cpp: interaction/interaction_engine.cc:122-144
    // Rust's nonnullable Rc callback enforces the source required mutation output.
    pub fn new(emit: PageMutationEmitter, listeners: Option<EventListenerDispatcher<'a>>) -> Self {
        Self::WithSelectionState(emit, listeners, Rc::new(Default::default()))
    }
    pub fn WithSelectionState(
        emit: PageMutationEmitter,
        listeners: Option<EventListenerDispatcher<'a>>,
        selections: Rc<layoutng_assembly::editing_state::SelectionState>,
    ) -> Self {
        Self {
            emit,
            listeners: Rc::new(RefCell::new(listeners)),
            state: Rc::new(RefCell::new(InteractionState::default())),
            editor: Rc::new(TextEditor::WithSelectionState(selections)),
            selection_drag: Rc::new(Cell::new(None)),
            submit_form: Rc::new(RefCell::new(None)),
            submitting_forms: Rc::new(RefCell::new(Vec::new())),
        }
    }
    pub fn WithDOMMutationEmitter(
        emit: InteractionDOMMutationEmitter,
        listeners: Option<EventListenerDispatcher<'a>>,
    ) -> Self {
        Self::new(
            Rc::new(move |mutation| {
                if let PageMutation::DOMMutation(mutation) = mutation {
                    emit(&mutation)
                }
            }),
            listeners,
        )
    }
    // cpp: interaction/interaction_engine.h:47-51
    pub fn SetEventListenerDispatcher(&self, dispatcher: Option<EventListenerDispatcher<'a>>) {
        *self.listeners.borrow_mut() = dispatcher;
    }
    pub fn WithScopedListeners<'b>(
        &self,
        listeners: EventListenerDispatcher<'b>,
    ) -> Interaction<'b> {
        Interaction {
            emit: self.emit.clone(),
            listeners: Rc::new(RefCell::new(Some(listeners))),
            state: self.state.clone(),
            editor: self.editor.clone(),
            selection_drag: self.selection_drag.clone(),
            submit_form: self.submit_form.clone(),
            submitting_forms: self.submitting_forms.clone(),
        }
    }
    /// Same read-only hit test used for event targeting. Selection policy stays
    /// here; the host receives only a platform-independent cursor value.
    pub fn CursorAt(
        &self,
        document: &InteractionDocument,
        fragments: &FragmentNode,
        point: Offset,
    ) -> crate::cursor::Cursor {
        let owner = document.borrow();
        let doc = owner.GetDocument();
        crate::cursor::SelectCursor(
            doc,
            HitTest(doc, fragments, point, Offset::default(), point),
        )
    }
    pub fn State(&self) -> InteractionState {
        *self.state.borrow()
    }
    pub fn Editor(&self) -> Rc<TextEditor> {
        self.editor.clone()
    }
    fn Dispatcher(&self, document: &InteractionDocument) -> EventDispatcher<'a> {
        let emit = self.emit.clone();
        let dom: InteractionDOMMutationEmitter =
            Rc::new(move |m| emit(PageMutation::DOMMutation(m.clone())));
        EventDispatcher::new(
            document.clone(),
            dom,
            self.listeners.clone(),
            self.state.clone(),
            self.editor.clone(),
            self.submit_form.borrow().clone(),
            self.submitting_forms.clone(),
        )
    }
    /// Submission is a document default action; its host callback is deferred
    /// by Page's client, outside listener dispatch and DOM borrows.
    pub fn SetFormSubmissionHandler(&self, handler: Option<Rc<dyn Fn(u64, Option<u64>)>>) {
        *self.submit_form.borrow_mut() = handler;
    }
    pub fn SubmitForm(
        &self,
        document: &InteractionDocument,
        form: u64,
        submitter: Option<u64>,
        dispatch_event: bool,
    ) {
        self.Dispatcher(document)
            .SubmitForm(form, submitter, dispatch_event);
    }
    fn ExitGuard(&self) -> EmitStateMutationAtExit {
        EmitStateMutationAtExit {
            before: self.State(),
            current: self.state.clone(),
            emit: self.emit.clone(),
        }
    }
    // cpp: interaction/interaction_engine.cc:147-234
    pub fn Dispatch(
        &self,
        input: &InputEvent,
        document: &InteractionDocument,
        fragments: &FragmentNode,
    ) -> InteractionResult {
        let _emit_state = self.ExitGuard();
        let previous_hover = self.State().hovered_node_id;
        if matches!(input, InputEvent::Mouse(v) if matches!(v.r#type, MouseEventType::kDown | MouseEventType::kUp))
        {
            self.selection_drag.set(None);
        }
        let leaving =
            matches!(input, InputEvent::Mouse(event) if event.r#type == MouseEventType::kLeave);
        let mut target = ExplicitTarget(input);
        if leaving {
            target = target.or(self.State().hovered_node_id);
            self.state.borrow_mut().hovered_node_id = None;
        }
        if target.is_none() {
            if let Some(point) = EventPosition(input) {
                let owner = document.borrow();
                target = HitTest(
                    owner.GetDocument(),
                    fragments,
                    point,
                    Offset::default(),
                    point,
                )
            } else {
                target = self.State().focused_node_id
            }
        }
        let Some(target) = EventElement(document, target) else {
            return InteractionResult::default();
        };
        if let InputEvent::Mouse(mouse) = input {
            if IsDisabledFormControl(document, target)
                && matches!(
                    mouse.r#type,
                    MouseEventType::kDown
                        | MouseEventType::kUp
                        | MouseEventType::kClick
                        | MouseEventType::kDoubleClick
                )
            {
                return InteractionResult {
                    target_node_id: Some(target),
                    ..Default::default()
                };
            }
        }
        let mut event = MakeEvent(input);
        let dispatcher = self.Dispatcher(document);
        let nested = dispatcher.clone();
        let dispatch: SyntheticEventDispatcher<'a> = Rc::new(move |event, id| {
            nested.Dispatch(event, id);
        });
        let focus = FocusController::new(
            document.clone(),
            self.state.clone(),
            self.editor.clone(),
            dispatch,
        );
        match input {
            InputEvent::Mouse(v) => {
                {
                    let mut state = self.state.borrow_mut();
                    state.hovered_node_id = if v.r#type != MouseEventType::kLeave {
                        Some(target)
                    } else {
                        None
                    };
                    if v.r#type == MouseEventType::kDown {
                        state.pressed_node_id = Some(target)
                    }
                    if v.r#type == MouseEventType::kUp {
                        state.pressed_node_id = None
                    }
                }
                if v.r#type == MouseEventType::kClick {
                    if let Some(control) = FocusElement(document, target) {
                        if IsFocusableControl(document, control)
                            && !IsDisabledFormControl(document, control)
                        {
                            focus.Focus(control, IsTextField(document, control));
                        }
                    }
                }
            }
            InputEvent::Pointer(v) => {
                let mut state = self.state.borrow_mut();
                state.hovered_node_id = if v.r#type != PointerEventType::kLeave {
                    Some(target)
                } else {
                    None
                };
                if v.r#type == PointerEventType::kDown {
                    state.pressed_node_id = Some(target)
                }
                if v.r#type == PointerEventType::kUp || v.r#type == PointerEventType::kCancel {
                    state.pressed_node_id = None
                }
            }
            InputEvent::Key(v) => {
                if v.r#type == KeyEventType::kDown {
                    self.state.borrow_mut().focus_visible_node_id = Some(target)
                }
            }
            InputEvent::Focus(v) => {
                if v.r#type == FocusEventType::kFocus {
                    focus.Focus(target, true)
                } else {
                    focus.Blur(target)
                }
            }
            _ => {}
        }
        if let InputEvent::Mouse(mouse) = input {
            if matches!(
                mouse.r#type,
                MouseEventType::kMove | MouseEventType::kEnter | MouseEventType::kLeave
            ) {
                let entered = (mouse.r#type != MouseEventType::kLeave).then_some(target);
                DispatchMouseBoundaryEvents(&dispatcher, document, input, previous_hover, entered);
                if mouse.r#type != MouseEventType::kMove {
                    return InteractionResult {
                        target_node_id: entered.or(previous_hover),
                        ..Default::default()
                    };
                }
            }
        }
        if matches!(input, InputEvent::Focus(_)) {
            return InteractionResult {
                target_node_id: Some(target),
                ..Default::default()
            };
        }
        let result = dispatcher.Dispatch(&mut event, target);
        if let InputEvent::Mouse(v) = input {
            if v.r#type == MouseEventType::kDoubleClick
                && v.button == MouseButton::kPrimary
                && !result.default_prevented
            {
                if let Some(control) = FocusElement(document, target) {
                    if IsFocusableControl(document, control)
                        && !IsDisabledFormControl(document, control)
                    {
                        focus.Focus(control, IsTextField(document, control));
                        if self.State().focused_node_id == Some(control) {
                            self.SelectWordAtPoint(document, fragments, control, v.position);
                        }
                    }
                }
            }
            if v.r#type == MouseEventType::kDown && !result.default_prevented {
                if let Some(control) = FocusElement(document, target) {
                    if IsFocusableControl(document, control)
                        && !IsDisabledFormControl(document, control)
                    {
                        focus.Focus(control, IsTextField(document, control));
                        if v.button == MouseButton::kPrimary
                            && self.State().focused_node_id == Some(control)
                        {
                            self.selection_drag.set(Some(control));
                            self.PlaceCaretAtPoint(
                                document,
                                fragments,
                                control,
                                v.position,
                                v.modifiers.shift,
                            );
                        }
                    }
                }
            }
            if v.r#type == MouseEventType::kMove && !result.default_prevented {
                if let Some(control) = self
                    .selection_drag
                    .get()
                    .filter(|id| Some(*id) == self.State().focused_node_id)
                {
                    self.PlaceCaretAtPoint(document, fragments, control, v.position, true);
                }
            }
        }
        InteractionResult {
            target_node_id: result.target_node_id,
            default_prevented: result.default_prevented,
            propagation_stopped: result.propagation_stopped,
        }
    }
    fn PlaceCaretAtPoint(
        &self,
        document: &InteractionDocument,
        fragments: &FragmentNode,
        id: u64,
        point: Offset,
        extend: bool,
    ) {
        if let Some((owner, local)) = ControlPoint(fragments, id, point, Offset::default(), point) {
            let empty =
                ReadNode(document, id, |d, _, i| d.ControlValue(i).is_empty()).unwrap_or(true);
            let offset =
                layoutng_assembly::caret::geometry::TextControlOffsetForPoint(owner, local, empty);
            self.editor.PlaceCaret(document, id, offset, extend);
        }
    }
    fn SelectWordAtPoint(
        &self,
        document: &InteractionDocument,
        fragments: &FragmentNode,
        id: u64,
        point: Offset,
    ) {
        if let Some((owner, local)) = ControlPoint(fragments, id, point, Offset::default(), point) {
            let empty =
                ReadNode(document, id, |d, _, i| d.ControlValue(i).is_empty()).unwrap_or(true);
            let offset =
                layoutng_assembly::caret::geometry::TextControlOffsetForPoint(owner, local, empty);
            self.editor.SelectWordAt(document, id, offset);
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
        let result = self.Dispatcher(document).Dispatch(event, target);
        InteractionResult {
            target_node_id: result.target_node_id,
            default_prevented: result.default_prevented,
            propagation_stopped: result.propagation_stopped,
        }
    }
}

// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/event_handling_util.h:1-103
// Source: third_party/blink/renderer/core/input/event_handling_util.cc:1-251
// Rust modules replace includes/namespaces. Borrowed query inputs retain the
// original document, frame and resident hit-test owners at package boundaries.

use std::rc::Rc;
use std::time::Duration;

use interaction_input::base::time::time::TimeTicks;
use interaction_input::public::common::input::web_input_event::WebInputEvent;
use interaction_input::public::common::input::web_mouse_event::WebMouseEvent;
use interaction_input::public::common::input::web_pointer_properties::PointF;

use crate::core::dom::events::event::{EventTargetKind, EventTargetRef};
use crate::core::dom::events::event_dispatch_result::DispatchEventResult;
pub use crate::public::platform::web_input_event_result::WebInputEventResult;

// Source: event_handling_util.h:13-15,26-30; .cc:25-45,203-216
// These are operations on the hit-test package's existing values, with no
// parallel request/result/location storage owned by dispatch. The associated
// flags are HitTestRequest's canonical constants supplied by its owner.
pub trait HitTestValues {
    type Node: ?Sized;
    type Location;
    type Request;
    type Result;
    type Rect;
    type Offset;

    const kReadOnly: u32;
    const kActive: u32;
    const kIgnoreClipping: u32;
    const kAllowChildFrameContent: u32;

    fn IsRectBasedTest(location: &Self::Location) -> bool;
    fn Intersects(location: &Self::Location, rect: &Self::Rect) -> bool;
    fn Request(hit_type: u32) -> Self::Request;
    fn Result(request: Self::Request, location: &Self::Location) -> Self::Result;
    fn Location(point: Self::Offset) -> Self::Location;
    fn InnerNode(result: &Self::Result) -> Option<Rc<Self::Node>>;
    fn FromPointFRound(point: PointF) -> Self::Offset;
}

// Source: event_handling_util.cc:32-44,200-211
// The query traverses the canonical resident fragments. A missing query is
// precisely the source frame's absent ContentLayoutObject.
pub trait FrameHitTest<V: HitTestValues> {
    fn HitTest(&self, location: &V::Location, result: &mut V::Result) -> bool;
}

// Source: event_handling_util.cc:32-43
// View bounds are the source PhysicalRect(zero, view.Size()). These small facts
// are supplied at the query boundary; they are not a retained frame model.
pub struct FrameHitTestInput<'a, V: HitTestValues> {
    pub content_layout: Option<&'a dyn FrameHitTest<V>>,
    pub view_bounds: Option<&'a V::Rect>,
    pub has_document: bool,
    pub has_active_unbounded_elements: bool,
}

// Source: event_handling_util.cc:84-85
pub trait NodeConnectivity {
    fn IsConnected(&self, node: &EventTargetRef) -> bool;
}

// Source: event_handling_util.cc:88-89
pub trait FlatTreeParent<N: ?Sized> {
    type Container: ?Sized;
    fn Parent(&self, node: &N) -> Option<Rc<Self::Container>>;
}

// Source: event_handling_util.cc:92-100
pub trait RootFrameCoordinateView {
    fn ConvertFromRootFrame(&self, point: PointF) -> PointF;
}

// Source: event_handling_util.cc:103-112
pub trait DocumentMouseHitTest<V: HitTestValues> {
    type MouseResult;
    fn PerformMouseEventHitTest(
        &self,
        request: &V::Request,
        point: V::Offset,
        event: &WebMouseEvent,
    ) -> Self::MouseResult;
}

pub struct MouseHitTestFrame<'a, V: HitTestValues, D: DocumentMouseHitTest<V> + ?Sized> {
    pub view: Option<&'a dyn RootFrameCoordinateView>,
    pub document: Option<&'a D>,
    pub values: std::marker::PhantomData<V>,
}

// Source: event_handling_util.cc:138-148
// FrameView retains its own stability history. Dispatch supplies the original
// input timestamp to these two narrow read-only queries.
pub trait FrameStability {
    fn RectInParentIsStableForIOv2(&self, timestamp: TimeTicks) -> bool;
    fn RectInParentIsStable(&self, timestamp: TimeTicks) -> bool;
}

pub struct FrameDiscardInput<'a> {
    pub is_cross_origin_to_outermost_main_frame: bool,
    pub needs_occlusion_tracking: bool,
    pub view: &'a dyn FrameStability,
}

// Source: event_handling_util.cc:158-176
// The typed projection keeps all three source checks distinct: layout object
// is not embedded, embedded frame has no view, or its view is remote/local.
pub enum EmbeddedFrameView<F: ?Sized> {
    Remote,
    Local(Rc<F>),
}
pub enum TargetNodeLayout<F: ?Sized> {
    Other,
    Embedded(Option<EmbeddedFrameView<F>>),
}
pub trait SubframeLookup<N: ?Sized, F: ?Sized> {
    fn TargetNodeLayout(&self, node: &N) -> TargetNodeLayout<F>;
}

// Source: event_handling_util.cc:185-202
// The source's three nullable host projections are explicit alternatives.
// Found retains the same active Element/frame identity supplied by the widget.
pub enum ActiveUnboundedElementLookup<'a, V: HitTestValues, F: ?Sized> {
    NoWebFrame,
    NoWidget,
    NoActiveElement,
    Found {
        active_element: Rc<V::Node>,
        subframe: Option<UnboundedFrameInput<'a, V, F>>,
    },
}

pub struct UnboundedFrameInput<'a, V: HitTestValues, F: ?Sized> {
    pub frame: Rc<F>,
    pub is_descendant_of_from_frame: bool,
    pub content_layout: Option<&'a dyn FrameHitTest<V>>,
    pub view: Option<&'a dyn RootFrameCoordinateView>,
}

pub trait ActiveUnboundedElementSource<V: HitTestValues, F: ?Sized> {
    fn FromLocalFrameRoot(&self, from_frame: &Rc<F>) -> ActiveUnboundedElementLookup<'_, V, F>;
    fn IsDescendantOf(&self, node: &V::Node, ancestor: &V::Node) -> bool;
}

// Source: event_handling_util.cc:226-237
pub trait MouseEventHitTestResult<N: ?Sized> {
    fn IsOverEmbeddedContentView(&self) -> bool;
    fn InnerNode(&self) -> Option<Rc<N>>;
    fn Event(&self) -> &WebMouseEvent;
}
pub trait NodeDocumentFrame<N: ?Sized, F: ?Sized> {
    fn GetDocumentFrame(&self, node: &N) -> Option<Rc<F>>;
}

// Source: event_handling_util.h:52-63
pub struct UnboundedSubframeHitTestResult<V: HitTestValues, F: ?Sized> {
    pub frame: Rc<F>,
    pub location: V::Location,
    pub result: V::Result,
}

// Source: event_handling_util.h:79-83
pub const kDiscardedEventMistakeInterval: Duration = Duration::from_secs(5);

// Source: event_handling_util.h:85-94; .cc:244-248
// Rc owners replace Member tracing while retaining the same live identities.
pub struct PointerEventTarget<E: ?Sized, F: ?Sized, S: ?Sized> {
    pub target_element: Option<Rc<E>>,
    pub target_frame: Option<Rc<F>>,
    pub scrollbar: Option<Rc<S>>,
}
impl<E: ?Sized, F: ?Sized, S: ?Sized> Default for PointerEventTarget<E, F, S> {
    fn default() -> Self {
        Self {
            target_element: None,
            target_frame: None,
            scrollbar: None,
        }
    }
}

// Source: event_handling_util.h:26-30; .cc:25-46
pub fn HitTestResultInFrame<V: HitTestValues>(
    frame: Option<&FrameHitTestInput<'_, V>>,
    location: &V::Location,
    hit_type: u32,
    unbounded_element_enabled: bool,
) -> V::Result {
    debug_assert!(!V::IsRectBasedTest(location));
    let mut result = V::Result(V::Request(hit_type), location);
    let Some(frame) = frame else { return result };
    let Some(content_layout) = frame.content_layout else {
        return result;
    };
    if let Some(rect) = frame.view_bounds {
        if !unbounded_element_enabled || !frame.has_document || !frame.has_active_unbounded_elements
        {
            if !V::Intersects(location, rect) {
                return result;
            }
        }
    }
    content_layout.HitTest(location, &mut result);
    result
}

// Source: event_handling_util.h:29-30 (C++ default argument).
pub fn HitTestResultInFrameDefault<V: HitTestValues>(
    frame: Option<&FrameHitTestInput<'_, V>>,
    location: &V::Location,
    unbounded_element_enabled: bool,
) -> V::Result {
    HitTestResultInFrame::<V>(
        frame,
        location,
        V::kReadOnly | V::kActive,
        unbounded_element_enabled,
    )
}

// Source: event_handling_util.h:32-33; .cc:48-67
pub fn MergeEventResult(
    result_a: WebInputEventResult,
    result_b: WebInputEventResult,
) -> WebInputEventResult {
    const _: () = assert!(WebInputEventResult::kNotHandled as i32 == 0);
    const _: () = assert!(
        (WebInputEventResult::kHandledSuppressed as i32)
            < WebInputEventResult::kHandledApplication as i32
    );
    const _: () = assert!(
        (WebInputEventResult::kHandledApplication as i32)
            < WebInputEventResult::kHandledSystem as i32
    );
    std::cmp::max(result_a, result_b)
}

// Source: event_handling_util.h:34; .cc:69-82
pub fn ToWebInputEventResult(result: DispatchEventResult) -> WebInputEventResult {
    match result {
        DispatchEventResult::kNotCanceled => WebInputEventResult::kNotHandled,
        DispatchEventResult::kCanceledByEventHandler => WebInputEventResult::kHandledApplication,
        DispatchEventResult::kCanceledByDefaultEventHandler => WebInputEventResult::kHandledSystem,
        DispatchEventResult::kCanceledBeforeDispatch => WebInputEventResult::kHandledSuppressed,
    }
}

// Source: event_handling_util.h:36; .cc:84-86
pub fn IsInDocument(target: Option<&EventTargetRef>, nodes: &dyn NodeConnectivity) -> bool {
    target.is_some_and(|target| target.Kind() == EventTargetKind::Node && nodes.IsConnected(target))
}

// Source: event_handling_util.h:38; .cc:88-90
pub fn ParentForClickEvent<N: ?Sized, P: FlatTreeParent<N> + ?Sized>(
    node: &N,
    flat_tree: &P,
) -> Option<Rc<P::Container>> {
    flat_tree.Parent(node)
}

// Source: event_handling_util.h:40-41; .cc:92-101
pub fn ContentPointFromRootFrame<V: HitTestValues>(
    view: Option<&dyn RootFrameCoordinateView>,
    point_in_root_frame: PointF,
) -> V::Offset {
    // Preserve the source fallback to root coordinates when frame.View is null.
    V::FromPointFRound(match view {
        Some(view) => view.ConvertFromRootFrame(point_in_root_frame),
        None => point_in_root_frame,
    })
}

// Source: event_handling_util.h:43-45; .cc:103-113
pub fn PerformMouseEventHitTest<V: HitTestValues, D: DocumentMouseHitTest<V> + ?Sized>(
    frame: Option<&MouseHitTestFrame<'_, V, D>>,
    request: &V::Request,
    event: &WebMouseEvent,
) -> D::MouseResult {
    debug_assert!(frame.is_some());
    let frame = frame.expect("mouse hit test requires a frame");
    debug_assert!(frame.document.is_some());
    frame
        .document
        .expect("mouse hit test requires the frame's document")
        .PerformMouseEventHitTest(
            request,
            ContentPointFromRootFrame::<V>(frame.view, event.PositionInRootFrame()),
            event,
        )
}

// Source: event_handling_util.h:72-77; .cc:115-156
pub fn ShouldDiscardEventTargetingFrame(
    event: &WebInputEvent,
    frame: &FrameDiscardInput<'_>,
    discard_input_events_to_recently_moved_frames: bool,
) -> bool {
    let mut should_discard = false;
    if frame.is_cross_origin_to_outermost_main_frame {
        if frame.needs_occlusion_tracking {
            should_discard = event.GetModifiers() & WebInputEvent::kTargetFrameMovedRecentlyForIOv2
                != 0
                || !frame.view.RectInParentIsStableForIOv2(event.TimeStamp());
        } else if discard_input_events_to_recently_moved_frames {
            should_discard = event.GetModifiers() & WebInputEvent::kTargetFrameMovedRecently != 0
                || !frame.view.RectInParentIsStable(event.TimeStamp());
        }
    }
    if should_discard {
        // Source: event_handling_util.cc:152-153
        // TODO: UseCounter::Count(kDiscardInputEventToMovingIframe).
    }
    should_discard
}

// Source: event_handling_util.h:50; .cc:158-177
pub fn SubframeForTargetNode<N: ?Sized, F: ?Sized>(
    node: Option<&N>,
    frames: &impl SubframeLookup<N, F>,
    is_remote_frame: Option<&mut bool>,
) -> Option<Rc<F>> {
    let node = node?;
    let TargetNodeLayout::Embedded(child_frame_view) = frames.TargetNodeLayout(node) else {
        return None;
    };
    match child_frame_view? {
        EmbeddedFrameView::Remote => {
            if let Some(is_remote_frame) = is_remote_frame {
                *is_remote_frame = true;
            }
            None
        }
        EmbeddedFrameView::Local(frame) => Some(frame),
    }
}

// Source: event_handling_util.h:65-70; .cc:179-224
pub fn SubframeForActiveUnboundedElement<V: HitTestValues, F: ?Sized>(
    from_frame: Option<&Rc<F>>,
    point_in_root_frame: PointF,
    unbounded_element_enabled: bool,
    source: &impl ActiveUnboundedElementSource<V, F>,
) -> Option<UnboundedSubframeHitTestResult<V, F>> {
    if !unbounded_element_enabled || from_frame.is_none() {
        return None;
    }
    let from_frame = from_frame.unwrap();
    let (active_element, subframe) = match source.FromLocalFrameRoot(from_frame) {
        ActiveUnboundedElementLookup::NoWebFrame => return None,
        ActiveUnboundedElementLookup::NoWidget => return None,
        ActiveUnboundedElementLookup::NoActiveElement => return None,
        ActiveUnboundedElementLookup::Found {
            active_element,
            subframe,
        } => (active_element, subframe),
    };
    let subframe = subframe?;
    if Rc::ptr_eq(&subframe.frame, from_frame) || !subframe.is_descendant_of_from_frame {
        return None;
    }
    let content_layout = subframe.content_layout?;
    let subframe_point = ContentPointFromRootFrame::<V>(subframe.view, point_in_root_frame);
    let subframe_location = V::Location(subframe_point);
    let request = V::Request(V::kReadOnly | V::kIgnoreClipping | V::kAllowChildFrameContent);
    let mut subframe_result = V::Result(request, &subframe_location);
    if !content_layout.HitTest(&subframe_location, &mut subframe_result) {
        return None;
    }
    let inner_node = V::InnerNode(&subframe_result)?;
    if !Rc::ptr_eq(&inner_node, &active_element)
        && !source.IsDescendantOf(inner_node.as_ref(), active_element.as_ref())
    {
        return None;
    }
    Some(UnboundedSubframeHitTestResult {
        frame: subframe.frame,
        location: subframe_location,
        result: subframe_result,
    })
}

// Source: event_handling_util.h:47-48; .cc:226-242
pub fn GetTargetSubframe<V: HitTestValues, F: ?Sized>(
    hit_test_result: &impl MouseEventHitTestResult<V::Node>,
    frames: &impl SubframeLookup<V::Node, F>,
    documents: &impl NodeDocumentFrame<V::Node, F>,
    unbounded: &impl ActiveUnboundedElementSource<V, F>,
    unbounded_element_enabled: bool,
    is_remote_frame: Option<&mut bool>,
) -> Option<Rc<F>> {
    if hit_test_result.IsOverEmbeddedContentView() {
        return SubframeForTargetNode(
            hit_test_result.InnerNode().as_deref(),
            frames,
            is_remote_frame,
        );
    }
    if let Some(inner_node) = hit_test_result.InnerNode() {
        let frame = documents.GetDocumentFrame(inner_node.as_ref());
        if let Some(unbounded_result) = SubframeForActiveUnboundedElement::<V, F>(
            frame.as_ref(),
            hit_test_result.Event().PositionInRootFrame(),
            unbounded_element_enabled,
            unbounded,
        ) {
            return Some(unbounded_result.frame);
        }
    }
    None
}

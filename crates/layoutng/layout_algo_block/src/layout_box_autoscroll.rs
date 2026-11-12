//! Source-owned scrolling over resident LayoutBox/ScrollableArea identities.
//! Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
use super::super::scroll_types::{IncludeScrollbarsInRect, ScrollOffset};
use super::super::scrollbar_orientation::ScrollbarOrientation;
use super::*;
use layoutng_style::style::scroll_enums::mojom::blink::{ScrollBehavior, ScrollType};

// core/scroll/scroll_alignment.cc:49-114; scroll_into_view_params.mojom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAlignmentBehavior {
    NoScroll,
    Left,
    Right,
    Top,
    Bottom,
    Center,
    ClosestEdge,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollAlignment {
    pub rect_visible: ScrollAlignmentBehavior,
    pub rect_hidden: ScrollAlignmentBehavior,
    pub rect_partial: ScrollAlignmentBehavior,
}
impl ScrollAlignment {
    pub const fn ToEdgeIfNeeded() -> Self {
        Self {
            rect_visible: ScrollAlignmentBehavior::NoScroll,
            rect_hidden: ScrollAlignmentBehavior::ClosestEdge,
            rect_partial: ScrollAlignmentBehavior::ClosestEdge,
        }
    }
    pub const fn CenterIfNeeded() -> Self {
        Self {
            rect_visible: ScrollAlignmentBehavior::NoScroll,
            rect_hidden: ScrollAlignmentBehavior::Center,
            rect_partial: ScrollAlignmentBehavior::ClosestEdge,
        }
    }
    pub const fn CenterAlways() -> Self {
        Self {
            rect_visible: ScrollAlignmentBehavior::Center,
            rect_hidden: ScrollAlignmentBehavior::Center,
            rect_partial: ScrollAlignmentBehavior::Center,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ScrollIntoViewParams {
    pub align_x: ScrollAlignment,
    pub align_y: ScrollAlignment,
    pub scroll_type: ScrollType,
    pub include_self: bool,
    pub behavior: ScrollBehavior,
    pub is_for_scroll_sequence: bool,
}
impl Default for ScrollIntoViewParams {
    fn default() -> Self {
        Self {
            align_x: ScrollAlignment::ToEdgeIfNeeded(),
            align_y: ScrollAlignment::ToEdgeIfNeeded(),
            scroll_type: ScrollType::kProgrammatic,
            include_self: false,
            behavior: ScrollBehavior::kAuto,
            is_for_scroll_sequence: false,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct NativeScrollOffset {
    pub node_id: u64,
    pub offset: super::super::layout_input::Offset,
    pub scroll_type: ScrollType,
}
#[derive(Default, Debug)]
pub struct NativeScrollResult {
    pub offsets: Vec<NativeScrollOffset>,
    pub programmatic_requests: Vec<super::super::layout_scrollable_area::NativePageScrollRequest>,
    pub did_scroll: bool,
}
impl NativeScrollResult {
    pub fn DidScroll(&self) -> bool {
        self.did_scroll
    }
}

// core/scroll/scroll_into_view_util.cc:480-540. Keep the source range
// containment rule: a target enclosing the snapport counts as fully visible.
fn CalculateScrollAlignment(
    alignment: ScrollAlignment,
    visible: (LayoutUnit, LayoutUnit),
    no_margin: (LayoutUnit, LayoutUnit),
    target: (LayoutUnit, LayoutUnit),
) -> ScrollAlignmentBehavior {
    let intersection = |a: (LayoutUnit, LayoutUnit), b: (LayoutUnit, LayoutUnit)| {
        std::cmp::min(a.1, b.1) - std::cmp::max(a.0, b.0)
    };
    let length = intersection(visible, target);
    if length == visible.1 - visible.0 || length == target.1 - target.0 {
        alignment.rect_visible
    } else if intersection(visible, no_margin) > LayoutUnit::default() {
        alignment.rect_partial
    } else {
        alignment.rect_hidden
    }
}

// core/scroll/scroll_into_view_util.cc:800-923. The viewport is the area's
// scroll-padding-adjusted snapport, not a projected fragment bounding box.
fn GetScrollOffsetToExpose(
    area: &super::super::layout_scrollable_area::PaintLayerScrollableArea,
    local: PhysicalRect,
    margin: PhysicalBoxStrut,
    params: ScrollIntoViewParams,
) -> ScrollOffset {
    let mut no_margin = local;
    no_margin.Move(&area.LocalToScrollOriginOffset());
    let mut target = no_margin;
    target.Expand(&margin);
    let mut visible = area.VisibleScrollSnapportRect(IncludeScrollbarsInRect::kExcludeScrollbars);
    let mut minimum = LayoutUnit::default();
    minimum.SetRawValueI32(1);
    if visible.Width() <= LayoutUnit::default() {
        visible.size.width = minimum;
    }
    if visible.Height() <= LayoutUnit::default() {
        visible.size.height = minimum;
    }
    let old = area.GetScrollOffset();
    let mut x = CalculateScrollAlignment(
        params.align_x,
        (visible.X(), visible.Right()),
        (no_margin.X(), no_margin.Right()),
        (target.X(), target.Right()),
    );
    let mut y = CalculateScrollAlignment(
        params.align_y,
        (visible.Y(), visible.Bottom()),
        (no_margin.Y(), no_margin.Bottom()),
        (target.Y(), target.Bottom()),
    );
    if x == ScrollAlignmentBehavior::ClosestEdge
        && ((target.Right() > visible.Right() && target.Width() < visible.Width())
            || (target.Right() < visible.Right() && target.Width() > visible.Width()))
    {
        x = ScrollAlignmentBehavior::Right;
    }
    if y == ScrollAlignmentBehavior::ClosestEdge
        && ((target.Bottom() > visible.Bottom() && target.Height() < visible.Height())
            || (target.Bottom() < visible.Bottom() && target.Height() > visible.Height()))
    {
        y = ScrollAlignmentBehavior::Bottom;
    }
    visible.Move(&-PhysicalOffset::FromVector2dFRound(&old));
    let offset_x = match x {
        ScrollAlignmentBehavior::NoScroll => old.x(),
        ScrollAlignmentBehavior::Right => (target.Right() - visible.Right()).ToFloat(),
        ScrollAlignmentBehavior::Center => {
            ((target.X() + target.Right() - visible.X() - visible.Right()) / 2_i32).ToFloat()
        }
        _ => (target.X() - visible.X()).ToFloat(),
    };
    let offset_y = match y {
        ScrollAlignmentBehavior::NoScroll => old.y(),
        ScrollAlignmentBehavior::Bottom => (target.Bottom() - visible.Bottom()).ToFloat(),
        ScrollAlignmentBehavior::Center => {
            ((target.Y() + target.Bottom() - visible.Y() - visible.Bottom()) / 2_i32).ToFloat()
        }
        _ => (target.Y() - visible.Y()).ToFloat(),
    };
    ScrollOffset::new(offset_x, offset_y)
}

// core/paint/paint_layer_scrollable_area.cc:2554-2630. The source ScrollableArea
// commits the new offset before the containing-block walk continues.
fn ScrollIntoView(
    area: &mut super::super::layout_scrollable_area::PaintLayerScrollableArea,
    absolute: PhysicalRect,
    margin: PhysicalBoxStrut,
    params: ScrollIntoViewParams,
    result: &mut NativeScrollResult,
) -> PhysicalRect {
    let box_ = area.GetLayoutBox();
    let mut local =
        unsafe { &*box_ }.AbsoluteToLocalRect(&absolute, MapCoordinatesFlags::default());
    let target = GetScrollOffsetToExpose(area, local, margin, params);
    let rounded = gfx::ToRoundedVector2d(&target);
    let clamped = area.ClampScrollOffsetInt(&rounded);
    let old = area.GetScrollOffset();
    let mut new_offset = ScrollOffset::new(clamped.x() as f32, clamped.y() as f32);
    if params.scroll_type == ScrollType::kUser {
        if !area.UserInputScrollable(ScrollbarOrientation::kHorizontalScrollbar) {
            new_offset.set_x(old.x());
        }
        if !area.UserInputScrollable(ScrollbarOrientation::kVerticalScrollbar) {
            new_offset.set_y(old.y());
        }
    }
    let behavior = if params.is_for_scroll_sequence {
        params.behavior
    } else {
        ScrollBehavior::kInstant
    };
    let request = area.PlanScrollIntoView(new_offset, behavior);
    new_offset = request.offset;
    if params.scroll_type == ScrollType::kProgrammatic && params.is_for_scroll_sequence {
        // Source will-be-scrolled geometry is used below even while Smooth
        // waits for its animator. Page routes this request after the native
        // containing-block walk, and never recomputes the target.
        result.did_scroll |= !request.is_noop;
        result.programmatic_requests.push(request);
    } else {
        area.UpdateScrollOffset(new_offset, params.scroll_type);
    }
    if !(params.scroll_type == ScrollType::kProgrammatic && params.is_for_scroll_sequence)
        && old != area.GetScrollOffset()
    {
        result.did_scroll = true;
        let node = unsafe { &*box_ }.GetNode();
        if let Some(node) = unsafe { node.as_ref() } {
            result.offsets.push(NativeScrollOffset {
                node_id: node.InputId(),
                offset: super::super::layout_input::Offset {
                    x: new_offset.x() as f64,
                    y: new_offset.y() as f64,
                },
                scroll_type: params.scroll_type,
            });
        }
    }
    local.Move(&-PhysicalOffset::FromVector2dFRound(&ScrollOffset::new(
        new_offset.x() - old.x(),
        new_offset.y() - old.y(),
    )));
    let mut scrollport = area.LayoutContentRect(IncludeScrollbarsInRect::kExcludeScrollbars);
    let origin = area.ScrollOrigin();
    scrollport.Move(&PhysicalOffset::new(
        LayoutUnit::from_signed(-origin.x()),
        LayoutUnit::from_signed(-origin.y()),
    ));
    scrollport.Move(&-area.LocalToScrollOriginOffset());
    let mut intersect = local;
    intersect.Intersect(&scrollport);
    if intersect.IsEmpty() && !scrollport.IsEmpty() && !local.IsEmpty() {
        unsafe { &*box_ }.LocalToAbsoluteRect(&local, MapCoordinatesFlags::default())
    } else {
        unsafe { &*box_ }.LocalToAbsoluteRect(&intersect, MapCoordinatesFlags::default())
    }
}

// core/scroll/scroll_into_view_util.cc:220-252,296-475,574-604. A Layout
// domain has one local document. Cross-document propagation must be routed by
// its frame owner after this local-root result, never through another Page's
// LayoutObject or a callback reaching back into Page.
pub(super) fn LayoutBoxScrollRectToVisible(
    box_: &LayoutBox,
    absolute: PhysicalRect,
    params: ScrollIntoViewParams,
) -> NativeScrollResult {
    LayoutBoxScrollRectToVisibleWithMargin(
        box_,
        absolute,
        params,
        box_.StyleRef().ScrollMarginStrut(),
    )
}
pub(super) fn LayoutBoxScrollIntoOwnArea(
    box_: &LayoutBox,
    absolute: PhysicalRect,
    margin: PhysicalBoxStrut,
    params: ScrollIntoViewParams,
) -> NativeScrollResult {
    let mut result = NativeScrollResult::default();
    let area = box_.GetScrollableArea();
    if !area.is_null() {
        ScrollIntoView(unsafe { &mut *area }, absolute, margin, params, &mut result);
    }
    result
}

pub(super) fn LayoutBoxScrollRectToVisibleWithMargin(
    box_: &LayoutBox,
    mut absolute: PhysicalRect,
    params: ScrollIntoViewParams,
    margin: PhysicalBoxStrut,
) -> NativeScrollResult {
    let mut result = NativeScrollResult::default();
    if box_.InputOwnerForLayout().InputViewport().is_none() {
        return result;
    }
    let original: *const LayoutBox = box_;
    let mut current = original;
    while let Some(box_) = unsafe { current.as_ref() } {
        if absolute.Width() <= LayoutUnit::default() {
            absolute.size.width = LayoutUnit::from_signed(1);
        }
        if absolute.Height() <= LayoutUnit::default() {
            absolute.size.height = LayoutUnit::from_signed(1);
        }
        let area = box_.GetScrollableArea();
        let is_scroll_owner =
            (box_.IsScrollContainer() && !box_.IsLayoutView()) || box_.ContainingBlock().is_null();
        if is_scroll_owner && !area.is_null() && (params.include_self || current != original) {
            absolute = ScrollIntoView(unsafe { &mut *area }, absolute, margin, params, &mut result);
        }
        if box_.IsFixedToView(std::ptr::null()) {
            break;
        }
        current = box_.ContainingBlock().cast();
    }
    result
}

// core/layout/layout_box.cc:1601-1625. Root-frame conversion is supplied by
// the frame-neutral source port; LayoutBox itself is in document coordinates.
pub(super) fn LayoutBoxAutoscroll(
    box_: &LayoutBox,
    position: &PhysicalOffset,
) -> NativeScrollResult {
    LayoutBoxScrollRectToVisible(
        box_,
        PhysicalRect::new(
            *position,
            PhysicalSize::new(LayoutUnit::from_signed(1), LayoutUnit::from_signed(1)),
        ),
        ScrollIntoViewParams {
            scroll_type: ScrollType::kUser,
            include_self: true,
            ..Default::default()
        },
    )
}

// core/layout/layout_box.cc:1627-1684. Absolute bounds are the union of native
// fragment border quads, followed by the source enclosing-integer conversion.
pub(super) fn LayoutBoxCalculateAutoscrollDirection(
    box_: &LayoutBox,
    point: &gfx::PointF,
    root_frame_offset: PhysicalOffset,
) -> PhysicalOffset {
    let Some(mut belt) = LayoutBoxAutoscrollScrollingBox(box_) else {
        return PhysicalOffset::default();
    };
    belt.Move(&root_frame_offset);
    LayoutBoxCalculateAutoscrollDirectionForRootRect(point, belt)
}
pub(super) fn LayoutBoxAutoscrollScrollingBox(box_: &LayoutBox) -> Option<PhysicalRect> {
    box_.CheckIsNotDestroyed();
    if box_.InputOwnerForLayout().InputViewport().is_none() {
        return None;
    }
    let mut quads = Vector::new();
    box_.QuadsInAncestorInternal(
        &mut quads,
        std::ptr::null(),
        MapCoordinatesFlags::default(),
        BoxQuadType::kBorder,
    );
    let mut bounds = gfx::RectF::default();
    for quad in quads.iter() {
        bounds.Union(quad.BoundingBox());
    }
    let integer = gfx::ToEnclosingRect(bounds);
    let mut belt = PhysicalRect::new(
        PhysicalOffset::new(
            LayoutUnit::from_signed(integer.x()),
            LayoutUnit::from_signed(integer.y()),
        ),
        PhysicalSize::new(
            LayoutUnit::from_signed(integer.width()),
            LayoutUnit::from_signed(integer.height()),
        ),
    );
    box_.ExcludeScrollbars(
        &mut belt,
        OverlayScrollbarClipBehavior::kExcludeOverlayScrollbarSizeForHitTesting,
        ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
    );
    Some(belt)
}
pub(super) fn LayoutBoxCalculateAutoscrollDirectionForRootRect(
    point: &gfx::PointF,
    mut belt: PhysicalRect,
) -> PhysicalOffset {
    let size = LayoutUnit::from_signed(20);
    belt.ContractEdges(size, size, size, size);
    let x = if point.x() < belt.X().ToFloat() {
        -20.0
    } else if point.x() > belt.Right().ToFloat() {
        20.0
    } else {
        0.0
    };
    let y = if point.y() < belt.Y().ToFloat() {
        -20.0
    } else if point.y() > belt.Bottom().ToFloat() {
        20.0
    } else {
        0.0
    };
    PhysicalOffset::FromVector2dFRound(&gfx::Vector2dF::new(x, y))
}

// core/layout/layout_box.cc:1686-1708. LocalOwner traversal is a frame-owner
// operation; within this native tree the parent link is the canonical owner.
pub(super) fn LayoutBoxFindAutoscrollable(
    object: *mut LayoutObject,
    middle_click: bool,
) -> *mut LayoutBox {
    LayoutBoxFindAutoscrollableInFrame(object, middle_click).0
}
pub(super) fn LayoutBoxFindAutoscrollableInFrame(
    mut object: *mut LayoutObject,
    middle_click: bool,
) -> (*mut LayoutBox, bool) {
    while let Some(current) = unsafe { object.as_ref() } {
        let box_ = DynamicTo::<LayoutBox>(object);
        if let Some(box_) = unsafe { box_.as_ref() } {
            if box_.IsUserScrollable() {
                return (box_ as *const _ as *mut _, false);
            }
            if !middle_click && box_.IsFixedToView(std::ptr::null()) {
                return (std::ptr::null_mut(), false);
            }
        }
        object = current.Parent();
        if object.is_null()
            && unsafe { current.GetNode().as_ref() }.is_some_and(|node| node.IsDocumentNode())
        {
            return (std::ptr::null_mut(), true);
        }
    }
    (std::ptr::null_mut(), false)
}

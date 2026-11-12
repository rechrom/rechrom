//! Production point-query bridge over canonical resident Layout/Paint cursors.
//!
//! This is the single moved production implementation previously assembled in
//! the facade. It owns coordinate mapping, contour gates, hit order and
//! read-only control/text-position queries; it retains no document or geometry
//! snapshot. DOM contributes only live admission/document-element facts.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source hit ranges: core/paint/box_fragment_painter.cc:2261-2468,2487-3018;
//! core/paint/paint_layer.cc:1277-1661,1909-2042. The complete source-shaped
//! translations remain in core/paint/; this existing retained-cursor bridge
//! preserves current production behavior while their remaining typed inputs
//! are assembled. Moving it is not a claim of full source hit-test parity.
//! The legacy nearest-text query below also preserves its existing contract;
//! replacing it with complete native PositionForPoint is separate work.

use super::{ResidentCursorGeometry, ResidentCursorGeometryError};
use crate::core::layout::hit_test_location::HitTestLocation;
use foundation::{gfx, DynamicTo, Persistent, PhysicalOffset, PhysicalRect, PhysicalSize};
use layoutng_assembly::internal::{
    layout_box_model_object::LayoutBoxModelObject, layout_scrollable_area::OverflowControlHit,
    scrollbar::Scrollbar,
};
use layoutng_assembly::{
    editing_state::DocumentSelectionPosition,
    fragment_tree::FragmentPaintData,
    internal::layout_input::{Display, FloatSide, NodeKind, Offset},
    layout_engine::PaintFragmentCursor,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use paint::path_hit_test::PaintPathHitTestFactory;
use std::cell::Cell;

/// The two live DOM facts consumed by this point-query contract. Neither DOM
/// storage nor a callback into another engine's object model crosses the port.
pub trait DocumentFacts {
    fn ContainsNode(&self, node_id: u64) -> bool;
    fn DocumentElementNodeId(&self) -> Option<u64>;
}

#[derive(Clone, Copy)]
pub struct ResidentFragmentSource<'a> {
    cursor: PaintFragmentCursor<'a>,
    paths: &'a dyn PaintPathHitTestFactory,
}

impl<'a> ResidentFragmentSource<'a> {
    pub fn new(cursor: PaintFragmentCursor<'a>, paths: &'a dyn PaintPathHitTestFactory) -> Self {
        Self { cursor, paths }
    }
    pub fn Children(self) -> Vec<Self> {
        self.cursor
            .Children()
            .into_iter()
            .map(|cursor| Self::new(cursor, self.paths))
            .collect()
    }

    pub fn Cursor(self) -> PaintFragmentCursor<'a> {
        self.cursor
    }
}

impl std::ops::Deref for ResidentFragmentSource<'_> {
    type Target = FragmentPaintData;

    fn deref(&self) -> &Self::Target {
        &self.cursor
    }
}

// Native input is expressed in CSS viewport pixels. Fragment geometry is in
// LocalFrame::LayoutZoomFactor coordinates, matching Blink's
// LocalFrameView::ViewportToFrame conversion before hit testing.
fn LayoutPoint(fragments: ResidentFragmentSource<'_>, point: Offset) -> Offset {
    let zoom = fragments
        .paint
        .resources
        .as_ref()
        .map_or(1.0, |resources| resources.layout_zoom_factor)
        .max(f64::MIN_POSITIVE);
    Offset {
        x: point.x * zoom,
        y: point.y * zoom,
    }
}

// Map a root-space point through the exact same transform and scroll state as paint.
fn LocalPoint(
    fragment: &FragmentPaintData,
    point: Offset,
    parent: Offset,
) -> Option<(Offset, Offset)> {
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
        let reference_rect = paint::geometry_mapper::ResolveTransformReferenceRect(
            fragment.paint.style.transform_reference_box,
            fragment.size,
            fragment.paint.border,
            fragment.paint.padding,
        );
        let matrix = paint::geometry_mapper::ResolveTransformAroundReferenceBox(
            transform,
            absolute,
            reference_rect,
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
fn ChildOrigin(fragment: &FragmentPaintData, absolute: Offset) -> Offset {
    if fragment.paint.establishes_paint_state {
        Offset {
            x: absolute.x - fragment.paint.pixel_snapped_scrolled_content_offset.x,
            y: absolute.y - fragment.paint.pixel_snapped_scrolled_content_offset.y,
        }
    } else {
        absolute
    }
}

/// Reject an entire query when canonical geometry cannot be constructed;
/// never publish a tentative hit from an incomplete traversal.
struct PointQuery<'a> {
    geometry: ResidentCursorGeometry<'a>,
    error: Cell<Option<ResidentCursorGeometryError>>,
}
impl<'a> PointQuery<'a> {
    fn new(paths: &'a dyn PaintPathHitTestFactory) -> Self {
        Self {
            geometry: ResidentCursorGeometry { paths },
            error: Cell::new(None),
        }
    }
    fn result<T>(&self, value: T) -> Result<T, ResidentCursorGeometryError> {
        match self.error.get() {
            Some(error) => Err(error),
            None => Ok(value),
        }
    }
    // Source: box_fragment_painter.cc:2298-2314,2337-2349,3000-3009.
    // All shape construction borrows the resident compiled ComputedStyle and
    // delegates to canonical ContouredBorderGeometry/BorderShapePainter.
    fn border_contains(
        &self,
        fragment: &ResidentFragmentSource<'_>,
        size: layoutng_assembly::internal::layout_input::Size,
        point: Offset,
        absolute: Offset,
        inner: bool,
        include_fragment_sides: bool,
        allow_border_shape: bool,
    ) -> bool {
        let cursor = fragment.Cursor();
        let style = match cursor.Cursor() {
            layoutng_assembly::layout_engine::FragmentCursor::Physical { fragment, .. } => {
                fragment.Style()
            }
            layoutng_assembly::layout_engine::FragmentCursor::Inline { item, .. } => item.Style(),
        };
        let mut rect = PhysicalRect::new(
            PhysicalOffset::new(
                foundation::LayoutUnit::from_f64(absolute.x),
                foundation::LayoutUnit::from_f64(absolute.y),
            ),
            PhysicalSize::new(
                foundation::LayoutUnit::from_f64(size.width),
                foundation::LayoutUnit::from_f64(size.height),
            ),
        );
        let location =
            HitTestLocation::FromPointF(gfx::PointF::new(point.x as f32, point.y as f32));
        if allow_border_shape && style.HasBorderShape() {
            if inner {
                // Source: only the descendant border-shape clip receives the
                // LayoutBox owner's overflow-clip-margin outsets.
                let object = cursor.Cursor().GetLayoutObject();
                if let Some(box_) = unsafe {
                    foundation::DynamicTo::<layoutng_assembly::internal::layout_box::LayoutBox>(
                        object,
                    )
                    .as_ref()
                } {
                    if box_.ShouldApplyOverflowClipMargin() {
                        rect.Expand(&box_.BorderOutsetsForClipping());
                    }
                }
            }
            return match self.geometry.BorderShapeOuterPath(cursor, rect) {
                Ok(path) => location.IntersectsPath(path.as_ref()),
                Err(error) => {
                    if self.error.get().is_none() {
                        self.error.set(Some(error));
                    }
                    false
                }
            };
        }
        !style.HasBorderRadius()
            || location.IntersectsContouredRect(
                self.geometry
                    .ContouredBorder(cursor, rect, inner, include_fragment_sides)
                    .as_ref(),
            )
    }
}

fn ContouredBorderContainsAt(
    query: &PointQuery<'_>,
    fragment: &ResidentFragmentSource<'_>,
    size: layoutng_assembly::internal::layout_input::Size,
    point: Offset,
    absolute: Offset,
    include_fragment_sides: bool,
) -> bool {
    query.border_contains(
        fragment,
        size,
        point,
        absolute,
        false,
        include_fragment_sides,
        false,
    )
}

fn OuterBorderContains(
    query: &PointQuery<'_>,
    fragment: &ResidentFragmentSource<'_>,
    point: Offset,
    absolute: Offset,
) -> bool {
    query.border_contains(fragment, fragment.size, point, absolute, false, true, true)
}

fn InnerBorderContains(
    query: &PointQuery<'_>,
    fragment: &ResidentFragmentSource<'_>,
    point: Offset,
    absolute: Offset,
) -> bool {
    query.border_contains(fragment, fragment.size, point, absolute, true, false, true)
}

// Chromium: core/paint/box_fragment_painter.cc:2535-2547. A line item uses
// the containing box's style for both contoured-border checks. The first
// check places the containing box geometry at the line item's self-ink origin;
// the second resolves the same style against the line bounds themselves.
fn LineSelfBordersContain(
    query: &PointQuery<'_>,
    fragment: &ResidentFragmentSource<'_>,
    containing_box: Option<&ResidentFragmentSource<'_>>,
    point: Offset,
    absolute: Offset,
) -> bool {
    if fragment.kind != layoutng_assembly::fragment_tree::FragmentKind::kLine {
        return true;
    }
    let Some(containing_box) = containing_box else {
        return true;
    };
    let overflow_origin = fragment
        .paint
        .self_ink_overflow
        .map_or(absolute, |overflow| Offset {
            x: absolute.x + overflow.offset.x,
            y: absolute.y + overflow.offset.y,
        });
    ContouredBorderContainsAt(
        query,
        containing_box,
        containing_box.size,
        point,
        overflow_origin,
        true,
    ) && ContouredBorderContainsAt(query, containing_box, fragment.size, point, absolute, false)
}

fn HitGeometry(
    query: &PointQuery<'_>,
    fragment: &ResidentFragmentSource<'_>,
    point: Offset,
    parent: Offset,
) -> Option<(Offset, Offset, bool, bool)> {
    // Chromium: core/paint/box_fragment_painter.cc:2261-2271. The same
    // canonical fragmentation policy gates both paint and NodeAtPoint; a
    // repeated non-first monolithic fragment cannot contribute descendants or
    // become a hit target through this package's flattened traversal.
    if fragment.paint.hidden
        || fragment.paint.svg_hidden_container
        || !fragment.paint.CanPaintFragmentOccurrence()
    {
        return None;
    }
    let (point, absolute) = LocalPoint(fragment, point, parent)?;
    if fragment.kind == layoutng_assembly::fragment_tree::FragmentKind::kLine {
        if let Some(overflow) = fragment.paint.ink_overflow {
            let left = absolute.x + overflow.offset.x;
            let top = absolute.y + overflow.offset.y;
            if point.x < left
                || point.y < top
                || point.x >= left + overflow.size.width
                || point.y >= top + overflow.size.height
            {
                return None;
            }
        }
        if !OuterBorderContains(query, fragment, point, absolute) {
            return None;
        }
    }
    let hit_bounds = fragment.paint.text_hit_test_bounds.unwrap_or(
        layoutng_assembly::fragment_tree::FragmentPaintRect {
            offset: Offset::default(),
            size: fragment.size,
        },
    );
    let hit_left = absolute.x + hit_bounds.offset.x;
    let hit_top = absolute.y + hit_bounds.offset.y;
    let inside = point.x >= hit_left
        && point.y >= hit_top
        && point.x < hit_left + hit_bounds.size.width
        && point.y < hit_top + hit_bounds.size.height;
    let outside_overflow_clip = fragment.paint.hit_test_overflow_clip.is_some_and(|clip| {
        let left = absolute.x + clip.offset.x;
        let top = absolute.y + clip.offset.y;
        point.x < left
            || point.y < top
            || point.x >= left + clip.size.width
            || point.y >= top + clip.size.height
    });
    if !inside && outside_overflow_clip {
        return None;
    }
    // BoxFragmentPainter::NodeAtPoint: rectangular overflow, display lock and
    // the inner contoured border suppress descendants, not the box's own hit.
    let children_clipped = fragment.paint.child_paint_blocked_by_display_lock
        || outside_overflow_clip
        || (fragment.paint.hit_test_overflow_clip.is_some()
            && !InnerBorderContains(query, fragment, point, absolute));
    Some((point, absolute, inside, children_clipped))
}

fn OverflowControlAt(
    fragment: ResidentFragmentSource<'_>,
    point: Offset,
    absolute: Offset,
) -> OverflowControlHit {
    let object = fragment.Cursor().Cursor().GetLayoutObject();
    let Some(box_) = (unsafe { DynamicTo::<LayoutBoxModelObject>(object).as_ref() }) else {
        return OverflowControlHit::None;
    };
    let Some(area) = (unsafe { box_.GetScrollableArea().as_ref() }) else {
        return OverflowControlHit::None;
    };
    // The source owning area decides which control was hit and returns that
    // exact native owner. Do not reconstruct it from painted axis rectangles.
    area.HitTestOverflowControls(&foundation::ToRoundedPoint(
        &foundation::PhysicalOffset::new(
            foundation::LayoutUnit::from_f64(point.x - absolute.x),
            foundation::LayoutUnit::from_f64(point.y - absolute.y),
        ),
    ))
}

/// A query result borrows the exact resident hit owner, never a copied tree.
#[derive(Clone)]
pub struct ResidentHitTarget<'a> {
    pub node_id: u64,
    pub fragment: PaintFragmentCursor<'a>,
    pub local_point: Offset,
    pub overflow_controls: bool,
    /// HitTestResult::GetScrollbar()->GetLayoutBox() == LayoutView.  Keep the
    /// source owner classification from the same resident hit instead of
    /// performing a second fragment walk in Interaction.
    pub frame_view_scrollbar: bool,
    pub native_scrollbar: Option<Persistent<Scrollbar>>,
}
type CursorHit<'a> = ResidentHitTarget<'a>;

fn ScrollbarHitTarget<'f>(
    document: &dyn DocumentFacts,
    fragment: ResidentFragmentSource<'f>,
    point: Offset,
    absolute: Offset,
) -> Option<CursorHit<'f>> {
    if !(fragment.paint.style.visible
        && !fragment.paint.style.pointer_events_none
        && document.ContainsNode(fragment.node_id))
    {
        return None;
    }
    let native_scrollbar = match OverflowControlAt(fragment, point, absolute) {
        OverflowControlHit::None => return None,
        OverflowControlHit::Scrollbar(owner) => Some(Persistent::from_ptr(owner)),
        OverflowControlHit::Corner | OverflowControlHit::Resizer => None,
    };
    Some(CursorHit {
        node_id: fragment.node_id,
        fragment: fragment.Cursor(),
        local_point: Offset {
            x: point.x - absolute.x,
            y: point.y - absolute.y,
        },
        overflow_controls: true,
        frame_view_scrollbar: native_scrollbar.is_some() && fragment.paint.is_layout_view,
        native_scrollbar,
    })
}

// Chromium: core/paint/box_fragment_painter.cc:2352-2361. Table rows and
// sections participate in descendant hit testing, but are not ordinary event
// targets themselves. Visual-overflow/occlusion testing is a separate request
// kind and does not enter this event-targeting path.
fn CanHitTestSelf(fragment: &FragmentPaintData) -> bool {
    if fragment.kind == layoutng_assembly::fragment_tree::FragmentKind::kLine
        && matches!(fragment.paint.source_kind, NodeKind::kSvgText)
    {
        return false;
    }
    if matches!(fragment.paint.source_kind, NodeKind::kSvgText) {
        return fragment.paint.style.pointer_events_bounding_box;
    }
    !matches!(
        fragment.paint.display,
        Display::kTableRow
            | Display::kTableSection
            | Display::kTableHeaderGroup
            | Display::kTableFooterGroup
    )
}

// Chromium: core/paint/box_fragment_painter.cc:2427-2444. LayoutView has no
// DOM node of its own. When its box is the hit, use the document element only
// when that element is represented by a direct physical child fragment.
fn ViewHitTarget<'f>(
    document: &dyn DocumentFacts,
    fragment: ResidentFragmentSource<'f>,
) -> Option<ResidentFragmentSource<'f>> {
    if !fragment.paint.is_layout_view {
        return None;
    }
    let element_id = document.DocumentElementNodeId()?;
    fragment
        .Children()
        .into_iter()
        .find(|child| child.node_id == element_id)
}

fn SelfHitTarget<'f>(
    query: &PointQuery<'_>,
    document: &dyn DocumentFacts,
    fragment: ResidentFragmentSource<'f>,
    containing_box: Option<&ResidentFragmentSource<'f>>,
    inside: bool,
    point: Offset,
    absolute: Offset,
) -> Option<CursorHit<'f>> {
    if !inside
        || !CanHitTestSelf(&fragment)
        || fragment.paint.style.pointer_events_none
        || !fragment.paint.style.visible
        || !LineSelfBordersContain(query, &fragment, containing_box, point, absolute)
        || !OuterBorderContains(query, &fragment, point, absolute)
    {
        return None;
    }
    if let Some(child) = ViewHitTarget(document, fragment) {
        return Some(CursorHit {
            node_id: child.node_id,
            fragment: child.Cursor(),
            local_point: Offset {
                x: point.x - absolute.x - child.offset.x,
                y: point.y - absolute.y - child.offset.y,
            },
            overflow_controls: false,
            frame_view_scrollbar: false,
            native_scrollbar: None,
        });
    }
    document.ContainsNode(fragment.node_id).then(|| CursorHit {
        node_id: fragment.node_id,
        fragment: fragment.Cursor(),
        local_point: Offset {
            x: point.x - absolute.x,
            y: point.y - absolute.y,
        },
        overflow_controls: false,
        frame_view_scrollbar: false,
        native_scrollbar: None,
    })
}

#[derive(Clone, Copy)]
struct HitLayer<'a> {
    fragment: ResidentFragmentSource<'a>,
    point: Offset,
    parent: Offset,
    classification: paint::paint_order::PaintLayerClassification,
}

fn ChildrenAreFlexOrGridItems(fragment: &FragmentPaintData) -> bool {
    fragment.paint.has_source
        && matches!(
            fragment.paint.display,
            Display::kFlex | Display::kGrid | Display::kGridLanes
        )
}

// PaintLayerPainter flattens non-stacking layers into their nearest stacking
// context and stable-sorts the result by Chromium's three paint-order groups.
// Preserve the physical
// geometry path with each candidate so hit testing can walk that order in
// reverse without teaching interaction how paint layers are classified.
fn CollectHitLayers<'a>(
    query: &PointQuery<'_>,
    fragment: ResidentFragmentSource<'a>,
    point: Offset,
    parent: Offset,
    root_point: Offset,
    output: &mut Vec<HitLayer<'a>>,
) {
    let Some((point, absolute, _, children_clipped)) = HitGeometry(query, &fragment, point, parent)
    else {
        return;
    };
    if children_clipped {
        return;
    }
    let children_are_flex_or_grid_items = ChildrenAreFlexOrGridItems(&fragment);
    for child in fragment.Children() {
        let (child_point, child_parent) = if child.paint.fixed_to_view {
            (root_point, Offset::default())
        } else {
            (point, ChildOrigin(&fragment, absolute))
        };
        let classification = paint::paint_order::ClassifyFragmentPaintLayer(
            &child,
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
                CollectHitLayers(query, child, child_point, child_parent, root_point, output);
            }
        } else {
            CollectHitLayers(query, child, child_point, child_parent, root_point, output);
        }
    }
}

// Chromium: core/paint/box_fragment_painter.cc:2446-2468,2586-2629,
// 2671-2774,2801-2998. The native code exposes four HitTestPhase calls;
// interaction has one point-event API, so it consumes the same phase order
// internally while the resident tree replaces InlineCursor overloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EventHitPhase {
    Foreground,
    Float,
    BlockBackground,
}

fn IsSvgHitObject(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::kSvgGroup
            | NodeKind::kSvgHiddenContainer
            | NodeKind::kSvgResourceClipper
            | NodeKind::kSvgResourceMasker
            | NodeKind::kSvgForeignObject
            | NodeKind::kSvgText
            | NodeKind::kSvgInline
            | NodeKind::kSvgTSpan
            | NodeKind::kSvgTextPath
            | NodeKind::kSvgShape
            | NodeKind::kSvgViewportContainer
    )
}

fn SelfHitPhase(fragment: &FragmentPaintData) -> EventHitPhase {
    if matches!(
        fragment.kind,
        layoutng_assembly::fragment_tree::FragmentKind::kText
            | layoutng_assembly::fragment_tree::FragmentKind::kGeneratedText
            | layoutng_assembly::fragment_tree::FragmentKind::kLine
    ) || fragment.paint.is_layout_inline
        || fragment.paint.box_fragment_is_inline_box
        || fragment.paint.replaced_content.is_some()
        || IsSvgHitObject(fragment.paint.source_kind)
    {
        EventHitPhase::Foreground
    } else {
        EventHitPhase::BlockBackground
    }
}

fn HitTestNormalFlowPhase<'f>(
    query: &PointQuery<'_>,
    document: &dyn DocumentFacts,
    fragment: ResidentFragmentSource<'f>,
    point: Offset,
    parent: Offset,
    root_point: Offset,
    include_self: bool,
    phase: EventHitPhase,
    containing_box: Option<&ResidentFragmentSource<'f>>,
) -> Option<CursorHit<'f>> {
    let (point, absolute, inside, children_clipped) = HitGeometry(query, &fragment, point, parent)?;
    // Overflow controls are painted after this subtree. Give their owner the
    // same frontmost hit priority, especially for overlay scrollbars which
    // otherwise dispatch through to covered links and controls.
    if phase == EventHitPhase::Foreground {
        if let Some(target) = ScrollbarHitTarget(document, fragment, point, absolute) {
            return Some(target);
        }
    }
    let children_are_flex_or_grid_items = ChildrenAreFlexOrGridItems(&fragment);
    if !children_clipped {
        for child in fragment.Children().into_iter().rev() {
            if paint::paint_order::ClassifyFragmentPaintLayer(
                &child,
                false,
                children_are_flex_or_grid_items,
            )
            .is_paint_layer
            {
                continue;
            }
            let child_is_float = child.paint.floating != FloatSide::kNone;
            let child_is_atomic = child.paint.painted_atomically;
            let (child_point, child_parent) = if child.paint.fixed_to_view {
                (root_point, Offset::default())
            } else {
                (point, ChildOrigin(&fragment, absolute))
            };
            let target = match phase {
                EventHitPhase::Foreground if child_is_float => None,
                EventHitPhase::Foreground if child_is_atomic => HitTestNormalFlow(
                    query,
                    document,
                    child,
                    child_point,
                    child_parent,
                    root_point,
                    true,
                    Some(&fragment),
                ),
                EventHitPhase::Foreground => HitTestNormalFlowPhase(
                    query,
                    document,
                    child,
                    child_point,
                    child_parent,
                    root_point,
                    true,
                    phase,
                    Some(&fragment),
                ),
                EventHitPhase::Float if child_is_float => HitTestNormalFlow(
                    query,
                    document,
                    child,
                    child_point,
                    child_parent,
                    root_point,
                    true,
                    Some(&fragment),
                ),
                EventHitPhase::Float if !child_is_atomic => HitTestNormalFlowPhase(
                    query,
                    document,
                    child,
                    child_point,
                    child_parent,
                    root_point,
                    true,
                    phase,
                    Some(&fragment),
                ),
                EventHitPhase::BlockBackground if !child_is_float && !child_is_atomic => {
                    HitTestNormalFlowPhase(
                        query,
                        document,
                        child,
                        child_point,
                        child_parent,
                        root_point,
                        true,
                        phase,
                        Some(&fragment),
                    )
                }
                _ => None,
            };
            if target.is_some() {
                return target;
            }
        }
    }
    (include_self && SelfHitPhase(&fragment) == phase)
        .then(|| {
            SelfHitTarget(
                query,
                document,
                fragment,
                containing_box,
                inside,
                point,
                absolute,
            )
        })
        .flatten()
}

fn HitTestNormalFlow<'f>(
    query: &PointQuery<'_>,
    document: &dyn DocumentFacts,
    fragment: ResidentFragmentSource<'f>,
    point: Offset,
    parent: Offset,
    root_point: Offset,
    include_self: bool,
    containing_box: Option<&ResidentFragmentSource<'f>>,
) -> Option<CursorHit<'f>> {
    for phase in [
        EventHitPhase::Foreground,
        EventHitPhase::Float,
        EventHitPhase::BlockBackground,
    ] {
        if let Some(target) = HitTestNormalFlowPhase(
            query,
            document,
            fragment,
            point,
            parent,
            root_point,
            include_self,
            phase,
            containing_box,
        ) {
            return Some(target);
        }
    }
    None
}

fn HitTestPaintLayer<'f>(
    query: &PointQuery<'_>,
    document: &dyn DocumentFacts,
    layer: &HitLayer<'f>,
    root_point: Offset,
) -> Option<CursorHit<'f>> {
    if layer.classification.is_stacking_context {
        HitTestStackingContext(
            query,
            document,
            layer.fragment,
            layer.point,
            layer.parent,
            root_point,
        )
    } else {
        HitTestNormalFlow(
            query,
            document,
            layer.fragment,
            layer.point,
            layer.parent,
            root_point,
            true,
            None,
        )
    }
}

fn HitTestStackingContext<'f>(
    query: &PointQuery<'_>,
    document: &dyn DocumentFacts,
    fragment: ResidentFragmentSource<'f>,
    point: Offset,
    parent: Offset,
    root_point: Offset,
) -> Option<CursorHit<'f>> {
    let (local_point, absolute, inside, _) = HitGeometry(query, &fragment, point, parent)?;
    if let Some(target) = ScrollbarHitTarget(document, fragment, local_point, absolute) {
        return Some(target);
    }
    let mut layers = Vec::new();
    CollectHitLayers(query, fragment, point, parent, root_point, &mut layers);
    // Stable sorting preserves physical order inside normal flow and for equal
    // z-index, matching PaintLayerPaintOrderIterator. Hit testing consumes the
    // positive-stacked group first, then normal flow, then negative stacked.
    layers.sort_by_key(|layer| {
        let c = layer.classification;
        if layer.fragment.paint.is_overscroll_area_parent {
            (3, 0)
        } else if !c.is_stacked {
            (1, 0)
        } else if c.stacking_level < 0 {
            (0, c.stacking_level)
        } else {
            (2, c.stacking_level)
        }
    });
    let negative_count = layers.partition_point(|layer| {
        layer.classification.is_stacked && layer.classification.stacking_level < 0
    });
    let normal_count =
        layers[negative_count..].partition_point(|layer| !layer.classification.is_stacked);
    let positive_start = negative_count + normal_count;
    let overscroll_start =
        layers.partition_point(|layer| !layer.fragment.paint.is_overscroll_area_parent);
    // Paint appends overscroll-area parents in reverse chaining order, so the
    // inverse hit-test walk consumes this stored group forward.
    for layer in &layers[overscroll_start..] {
        if let Some(target) = HitTestPaintLayer(query, document, layer, root_point) {
            return Some(target);
        }
    }
    for layer in layers[positive_start..overscroll_start].iter().rev() {
        if let Some(target) = HitTestPaintLayer(query, document, layer, root_point) {
            return Some(target);
        }
    }
    for layer in layers[negative_count..positive_start].iter().rev() {
        if let Some(target) = HitTestPaintLayer(query, document, layer, root_point) {
            return Some(target);
        }
    }
    if let Some(target) = HitTestNormalFlow(
        query, document, fragment, point, parent, root_point, false, None,
    ) {
        return Some(target);
    }
    for layer in layers[..negative_count].iter().rev() {
        if let Some(target) = HitTestPaintLayer(query, document, layer, root_point) {
            return Some(target);
        }
    }
    SelfHitTarget(
        query,
        document,
        fragment,
        None,
        inside,
        local_point,
        absolute,
    )
}

pub fn HitTest<'f>(
    document: &dyn DocumentFacts,
    fragment: impl Into<ResidentFragmentSource<'f>>,
    point: Offset,
) -> Result<Option<ResidentHitTarget<'f>>, ResidentCursorGeometryError> {
    let fragment = fragment.into();
    let point = LayoutPoint(fragment, point);
    let query = PointQuery::new(fragment.paths);
    query.result(HitTestStackingContext(
        &query,
        document,
        fragment,
        point,
        Offset::default(),
        point,
    ))
}
pub fn TextPositionAtPoint<'a>(
    fragment: impl Into<ResidentFragmentSource<'a>>,
    point: Offset,
    require_inside: bool,
) -> Result<Option<DocumentSelectionPosition>, ResidentCursorGeometryError> {
    fn visit(
        query: &PointQuery<'_>,
        fragment: ResidentFragmentSource<'_>,
        point: Offset,
        parent: Offset,
        root_point: Offset,
        require_inside: bool,
        best: &mut Option<(f64, f64, DocumentSelectionPosition)>,
    ) {
        let Some((point, absolute, inside, children_clipped)) =
            HitGeometry(query, &fragment, point, parent)
        else {
            return;
        };
        if matches!(
            fragment.kind,
            layoutng_assembly::fragment_tree::FragmentKind::kText
                | layoutng_assembly::fragment_tree::FragmentKind::kGeneratedText
        ) && fragment.paint.text_control_host.is_none()
            && fragment.paint.style.visible
            && !fragment.paint.style.user_select_none
            && (!require_inside || inside)
        {
            let dy = (absolute.y - point.y)
                .max(0.0)
                .max(point.y - absolute.y - fragment.size.height);
            let local = Offset {
                x: point.x - absolute.x,
                y: point.y - absolute.y,
            };
            if let Some(offset) = fragment.Cursor().TextOffsetForPoint(local) {
                let x = fragment
                    .Cursor()
                    .TextInlinePosition(offset)
                    .unwrap_or(local.x);
                let dx = (absolute.x + x - point.x).abs();
                let candidate = DocumentSelectionPosition {
                    tree_order: fragment.paint.logical_tree_order,
                    node_id: fragment.node_id,
                    offset,
                };
                if best
                    .as_ref()
                    .is_none_or(|(old_dy, old_dx, _)| (dy, dx) < (*old_dy, *old_dx))
                {
                    *best = Some((dy, dx, candidate));
                }
            }
        }
        if children_clipped {
            return;
        }
        for child in fragment.Children().into_iter().rev() {
            let (child_point, child_parent) = if child.paint.fixed_to_view {
                (root_point, Offset::default())
            } else {
                (point, ChildOrigin(&fragment, absolute))
            };
            visit(
                query,
                child,
                child_point,
                child_parent,
                root_point,
                require_inside,
                best,
            );
        }
    }

    let fragment = fragment.into();
    let point = LayoutPoint(fragment, point);
    let mut best = None;
    let query = PointQuery::new(fragment.paths);
    visit(
        &query,
        fragment,
        point,
        Offset::default(),
        point,
        require_inside,
        &mut best,
    );
    query.result(best.map(|(_, _, position)| position))
}

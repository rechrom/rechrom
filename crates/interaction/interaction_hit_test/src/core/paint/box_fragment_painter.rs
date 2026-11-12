/*
 * Copyright (C) 2017 The Chromium Authors
 * Use of this source code is governed by a BSD-style license that can be
 * found in the Chromium LICENSE file.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/paint/box_fragment_painter.h:68-84,224-324,334-340
// Source: third_party/blink/renderer/core/paint/box_fragment_painter.cc:102-258,323-393,598-607,2220-3019
// Only the hit-test execution ranges of this shared source are translated.
// The cursors borrow resident FragmentItems/PhysicalFragment links; they do
// not retain geometry, source styles, LayoutObjects, or a second fragment tree.

use foundation::{
    gfx, EPointerEvents, EVisibility, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize,
};

use crate::core::layout::hit_test_location::HitTestLocation;
use crate::core::layout::hit_test_phase::HitTestPhase;
use crate::core::layout::hit_test_request::{HitTestStopNode, ListBasedHitTestBehavior};
use crate::core::layout::hit_test_result::{
    HitTestBoxFragment, HitTestResult, HitTestResultRequest, NodeRef,
};
use crate::core::layout::pointer_events_hit_rules::{EHitTesting, PointerEventsHitRules};
use crate::platform::geometry::contoured_rect::ContouredRectHitTest;
use crate::platform::geometry::path::PathHitTest;

// Rust-only, call-scoped projections of source flags. An adapter reads these
// directly from the same resident fragment; no flag/geometry cache is kept.
#[derive(Clone, Copy, Default)]
pub struct FragmentFlags {
    pub can_traverse: bool,
    pub first_for_node: bool,
    pub can_paint_multiple_fragments: bool,
    pub hidden_for_paint: bool,
    pub atomic_inline: bool,
    pub self_painting_layer: bool,
    pub table_part: bool,
    pub table_row: bool,
    pub table_section: bool,
    pub svg_text: bool,
    pub opaque: bool,
    pub inline_box: bool,
    pub block_in_inline: bool,
    pub floating: bool,
    pub painted_atomically: bool,
    pub floating_descendants: bool,
    pub destroyed_or_moved: bool,
    pub clip_overflow: bool,
    pub scroll_container: bool,
    pub non_overlay_overscroll_container: bool,
    pub non_visible_overflow: bool,
    pub is_box: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Text,
    Line,
    Box,
}
#[derive(Clone, Copy)]
pub struct ItemFlags {
    pub kind: ItemKind,
    pub hidden_for_paint: bool,
    pub svg_text: bool,
    pub svg_bounding_box_transform: bool,
    pub svg_inline: bool,
    pub self_painting_layer: bool,
    pub floating: bool,
    pub inline_box: bool,
    pub block_in_inline: bool,
    pub ellipsis: bool,
    pub destroyed_or_moved: bool,
}

// Source: .cc:102-183,2298-2313,2338-2367,2539-2547,2655
// Borrow the canonical ComputedStyle owner and read only the source properties.
pub trait HitTestStyle {
    fn Visibility(&self) -> EVisibility;
    fn UsedPointerEvents(&self) -> EPointerEvents;
    fn IsTableBackgroundDisplay(&self) -> bool;
    fn HasBorderShape(&self) -> bool;
    fn HasBorderRadius(&self) -> bool;
    fn HasStroke(&self) -> bool;
    fn HasFill(&self) -> bool;
}

// Source: .cc:188-258. These are borrowed LayoutObject identities for the
// culled-inline ancestor walk; Layout retains all storage and sibling links.
pub trait CulledInlineLayout: std::any::Any {
    fn Identity(&self) -> usize;
    fn Parent(&self) -> Option<&dyn CulledInlineLayout>;
    fn HasPreviousOrNextSibling(&self) -> bool;
    fn IsDescendantOf(&self, ancestor: &dyn CulledInlineLayout) -> bool;
    fn IsLayoutInline(&self) -> bool;
}

// Source: .cc:323-393,598-607,2280-2313,2430-2443,2492-2503,3000-3018
// These independent synchronous geometry operations project existing Layout
// and Paint queries. They never mutate a sending engine or select traversal.
pub trait FragmentGeometryQueries<L: ?Sized> {
    fn MayIntersect(
        &self,
        result: &HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn MatchesStopNode(&self, stop: &dyn HitTestStopNode<L>) -> bool;
    fn ChildPaintBlockedByDisplayLock(&self) -> bool;
    fn AtomicInlineItemHiddenForPaint(&self) -> bool;
    fn IsLayoutView(&self) -> bool;
    fn DocumentElement(&self) -> Option<NodeRef>;
    fn ObjectBoundingBox(&self) -> gfx::RectF;
    fn OverflowClipRect(&self) -> PhysicalRect;
    fn OverflowClipMarginOutsets(&self) -> Option<[LayoutUnit; 4]>;
    fn BorderShapeOuterPath(&self, rect: PhysicalRect) -> Box<dyn PathHitTest + '_>;
    fn ContouredInnerBorder(&self, rect: PhysicalRect) -> Box<dyn ContouredRectHitTest + '_>;
    fn ContouredBorder(
        &self,
        rect: PhysicalRect,
        include_fragment_sides: bool,
    ) -> Box<dyn ContouredRectHitTest + '_>;
    fn ApplyFiltersToRect(&self, rect: PhysicalRect) -> PhysicalRect;
    fn VisualOverflowRectIncludingFilters(&self) -> PhysicalRect;
    fn TextCombineBounds(
        &self,
        item: &dyn FragmentItem<L>,
        offset: PhysicalOffset,
    ) -> Option<PhysicalRect>;
    fn HasLayer(&self) -> bool;
    fn ShouldPaintReplacedNormalFlowInline(&self) -> bool;
    fn HitTestReplacedNormalFlowInline(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
    ) -> bool;
    fn LegacyHitTestAllPhases(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn LegacyNodeAtPoint(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn HitTestOverflowControl(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn HitTestCulledInline(
        &self,
        parent: &dyn CulledInlineLayout,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        cursor: InlineCursor<'_, L>,
    ) -> bool;
}

// Source: .h:263-340; .cc:2715-2716,2890-2891. Borrow the resident child slice.
pub trait FragmentChildren<L: ?Sized> {
    fn len(&self) -> usize;
    fn at(&self, index: usize) -> FragmentLink<'_, L>;
}
pub struct FragmentLink<'a, L: ?Sized> {
    pub fragment: PhysicalFragmentRef<'a, L>,
    pub offset: PhysicalOffset,
}
// Call-scoped adapter values preserve the native child/item owner without
// requiring a copied tree, registry, or allocation per visited fragment.
pub enum PhysicalFragmentRef<'a, L: ?Sized> {
    Borrowed(&'a dyn PhysicalFragment<L>),
    Resident(crate::resident::ResidentFragment<'a, L>),
}
impl<'a, L: ?Sized> std::ops::Deref for PhysicalFragmentRef<'a, L> {
    type Target = dyn PhysicalFragment<L> + 'a;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(value) => *value,
            Self::Resident(value) => value,
        }
    }
}
pub enum FragmentItemRef<'a, L: ?Sized> {
    Borrowed(&'a dyn FragmentItem<L>),
    Resident(crate::resident::ResidentFragment<'a, L>),
}
impl<'a, L: ?Sized> std::ops::Deref for FragmentItemRef<'a, L> {
    type Target = dyn FragmentItem<L> + 'a;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(value) => *value,
            Self::Resident(value) => value,
        }
    }
}
pub trait PhysicalFragment<L: ?Sized> {
    fn Flags(&self) -> FragmentFlags;
    fn Size(&self) -> PhysicalSize;
    fn Style(&self) -> &dyn HitTestStyle;
    fn Geometry(&self) -> &dyn FragmentGeometryQueries<L>;
    fn ResultFragment(&self) -> &dyn HitTestBoxFragment;
    fn NodeForHitTest(&self) -> Option<NodeRef>;
    fn GetNode(&self) -> Option<NodeRef>;
    fn SelfOrContainerLayoutObject(&self) -> &dyn CulledInlineLayout;
    fn Children(&self) -> &dyn FragmentChildren<L>;
    fn Items(&self) -> Option<InlineCursor<'_, L>>;
    fn ScrolledContentOffset(&self) -> PhysicalOffset;
    fn OverscrollContentOffset(&self) -> PhysicalOffset;
    fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool;
}

pub trait FragmentItem<L: ?Sized> {
    fn Flags(&self) -> ItemFlags;
    fn Style(&self) -> &dyn HitTestStyle;
    fn NodeForHitTest(&self) -> Option<NodeRef>;
    fn LayoutObject(&self) -> &dyn CulledInlineLayout;
    fn BoxFragment(&self) -> Option<PhysicalFragmentRef<'_, L>>;
    fn LineBoxFragment(&self) -> Option<PhysicalFragmentRef<'_, L>>;
    fn OffsetInContainerFragment(&self) -> PhysicalOffset;
    fn InkOverflowRect(&self) -> PhysicalRect;
    fn SelfInkOverflowRect(&self) -> PhysicalRect;
    fn SvgUnscaledQuad(&self) -> gfx::QuadF;
    fn ObjectBoundingBox(&self) -> gfx::RectF;
    fn ComputeTextBoundsRectForHitTest(
        &self,
        offset: PhysicalOffset,
        visual_overflow: bool,
    ) -> PhysicalRect;
}

// Source: .cc:2487-2489,2518-2525,2613-2616,2805-2807,2854,2945-2946
// A cursor is a borrowed owner and an item position, never an item vector copy.
pub trait FragmentItems<L: ?Sized> {
    fn ContainerFragment(&self) -> &dyn PhysicalFragment<L>;
    fn At(&self, position: usize) -> FragmentItemRef<'_, L>;
    fn SiblingItemIndices(&self, parent: Option<usize>) -> Vec<usize>;
}
// Keep the existing painter input path for the source InlineCursor owner.
pub use crate::core::layout::inline::inline_cursor::InlineCursor;

// Source: .cc:102-130
fn IsVisibleToPaintFragment<L: ?Sized>(fragment: &dyn PhysicalFragment<L>) -> bool {
    let flags = fragment.Flags();
    if flags.hidden_for_paint {
        return false;
    }
    let style = fragment.Style();
    if style.Visibility() != EVisibility::kVisible && !style.IsTableBackgroundDisplay() {
        return false;
    }
    if flags.atomic_inline
        && flags.self_painting_layer
        && fragment.Geometry().AtomicInlineItemHiddenForPaint()
    {
        return false;
    }
    true
}
// Source: .cc:132-142
fn IsVisibleToHitTestStyle<L: ?Sized>(
    style: &dyn HitTestStyle,
    request: &HitTestResultRequest<'_, L>,
) -> bool {
    request.IgnorePointerEventsNone() || style.UsedPointerEvents() != EPointerEvents::kNone
}
// Source: .cc:144-165
fn IsVisibleToHitTestItem<L: ?Sized>(
    item: &dyn FragmentItem<L>,
    request: &HitTestResultRequest<'_, L>,
) -> bool {
    let flags = item.Flags();
    let style = item.Style();
    if !flags.svg_text {
        return !flags.hidden_for_paint
            && style.Visibility() == EVisibility::kVisible
            && IsVisibleToHitTestStyle(style, request);
    }
    if flags.hidden_for_paint {
        return false;
    }
    let rules = PointerEventsHitRules::new(
        EHitTesting::kSvgTextHitTesting,
        request,
        style.UsedPointerEvents(),
    );
    if rules.require_visible && style.Visibility() != EVisibility::kVisible {
        return false;
    }
    if rules.can_hit_bounding_box
        || (rules.can_hit_stroke && (style.HasStroke() || !rules.require_stroke))
        || (rules.can_hit_fill && (style.HasFill() || !rules.require_fill))
    {
        return IsVisibleToHitTestStyle(style, request);
    }
    false
}
// Source: .cc:167-183
fn IsVisibleToHitTestFragment<L: ?Sized>(
    fragment: &dyn PhysicalFragment<L>,
    request: &HitTestResultRequest<'_, L>,
) -> bool {
    if !IsVisibleToHitTestStyle(fragment.Style(), request) {
        return false;
    }
    if IsVisibleToPaintFragment(fragment) {
        return true;
    }
    request.IsHitTestVisualOverflow() && fragment.Flags().table_part
}

// Source: .cc:185-234
fn HitTestCulledInlineAncestors<L: ?Sized>(
    result: &mut HitTestResult<'_, L>,
    parent_cursor: InlineCursor<'_, L>,
    mut current: &dyn CulledInlineLayout,
    limit: &dyn CulledInlineLayout,
    previous_sibling: Option<&dyn FragmentItem<L>>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    queries: &dyn FragmentGeometryQueries<L>,
) -> bool {
    // Chromium's native InlineCursor guarantees a strict descendant here.
    // Resident item projections can also expose the container's own item; in
    // that case there are no culled ancestors between the two objects.
    if current.Identity() == limit.Identity() || !current.IsDescendantOf(limit) {
        return false;
    }
    debug_assert!(current.Identity() != limit.Identity() && current.IsDescendantOf(limit));
    if previous_sibling.is_some_and(|s| current.Identity() == s.LayoutObject().Identity()) {
        return false;
    }
    while let Some(parent) = current.Parent() {
        if parent.Identity() == limit.Identity() {
            break;
        }
        if current.HasPreviousOrNextSibling()
            && previous_sibling
                .is_some_and(|s| !s.Flags().floating && s.LayoutObject().IsDescendantOf(parent))
        {
            break;
        }
        if parent.IsLayoutInline()
            && queries.HitTestCulledInline(parent, result, location, offset, parent_cursor.clone())
        {
            return true;
        }
        current = parent;
    }
    false
}
// Source: .cc:236-258
fn HitTestCulledInlineAncestorsForItem<L: ?Sized>(
    result: &mut HitTestResult<'_, L>,
    container: &dyn PhysicalFragment<L>,
    parent_cursor: InlineCursor<'_, L>,
    item: &dyn FragmentItem<L>,
    previous_sibling: Option<&dyn FragmentItem<L>>,
    location: &HitTestLocation,
    physical_offset: PhysicalOffset,
) -> bool {
    if item.Flags().ellipsis {
        return false;
    }
    let fallback = physical_offset - item.OffsetInContainerFragment();
    HitTestCulledInlineAncestors(
        result,
        parent_cursor,
        item.LayoutObject(),
        container.SelfOrContainerLayoutObject(),
        previous_sibling,
        location,
        fallback,
        container.Geometry(),
    )
}

// Source: .cc:323-347
pub fn HitTestAllPhasesInFragment<L: ?Sized>(
    fragment: &dyn PhysicalFragment<L>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    result: &mut HitTestResult<'_, L>,
) -> bool {
    let flags = fragment.Flags();
    if !flags.can_traverse {
        if !flags.first_for_node && !flags.can_paint_multiple_fragments {
            return false;
        }
        return fragment
            .Geometry()
            .LegacyHitTestAllPhases(result, location, offset);
    }
    if !fragment.Geometry().MayIntersect(result, location, offset) {
        return false;
    }
    BoxFragmentPainter::new(fragment).HitTestAllPhases(result, location, offset)
}
// Source: .cc:349-374
pub fn HitTestReplacedNormalFlowInline<L: ?Sized>(
    fragment: &dyn PhysicalFragment<L>,
    result: &mut HitTestResult<'_, L>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> bool {
    if phase != HitTestPhase::kForeground {
        return false;
    }
    let geometry = fragment.Geometry();
    if !geometry.HasLayer() || !geometry.ShouldPaintReplacedNormalFlowInline() {
        return false;
    }
    if !geometry.MayIntersect(result, location, offset) {
        return false;
    }
    geometry.HitTestReplacedNormalFlowInline(result, location)
}
// Source: .cc:376-393
pub fn NodeAtPointInFragment<L: ?Sized>(
    fragment: &dyn PhysicalFragment<L>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
    result: &mut HitTestResult<'_, L>,
) -> bool {
    let flags = fragment.Flags();
    if !flags.can_traverse {
        if !flags.first_for_node && !flags.can_paint_multiple_fragments {
            return false;
        }
        return fragment
            .Geometry()
            .LegacyNodeAtPoint(result, location, offset, phase);
    }
    if !fragment.Geometry().MayIntersect(result, location, offset) {
        return false;
    }
    BoxFragmentPainter::new(fragment).NodeAtPoint(result, location, offset, phase)
}

// Source: .h:226-256. The mutable result borrow stays at the same address for
// the complete traversal; adjusted contexts only reborrow this same result.
pub struct HitTestContext<'l, 'r, 'q, L: ?Sized> {
    pub phase: HitTestPhase,
    pub location: &'l HitTestLocation,
    pub inline_root_offset: PhysicalOffset,
    pub result: &'r mut HitTestResult<'q, L>,
}
pub trait HitTestBounds {
    fn Add<L: ?Sized>(
        &self,
        result: &mut HitTestResult<'_, L>,
        node: Option<NodeRef>,
        location: &HitTestLocation,
    ) -> ListBasedHitTestBehavior;
}
impl HitTestBounds for PhysicalRect {
    fn Add<L: ?Sized>(
        &self,
        result: &mut HitTestResult<'_, L>,
        node: Option<NodeRef>,
        location: &HitTestLocation,
    ) -> ListBasedHitTestBehavior {
        result.AddNodeToListBasedTestResult(node, location, self)
    }
}
impl HitTestBounds for gfx::QuadF {
    fn Add<L: ?Sized>(
        &self,
        result: &mut HitTestResult<'_, L>,
        node: Option<NodeRef>,
        location: &HitTestLocation,
    ) -> ListBasedHitTestBehavior {
        result.AddNodeToListBasedTestResultQuad(node, location, self)
    }
}
impl<L: ?Sized> HitTestContext<'_, '_, '_, L> {
    // Source: .h:230-237; .cc:2220-2230
    pub fn AddNodeToResult<B: HitTestBounds>(
        &mut self,
        node: Option<NodeRef>,
        fragment: &dyn PhysicalFragment<L>,
        bounds: &B,
        offset: PhysicalOffset,
    ) -> bool {
        if node.is_some() && self.result.InnerNode().is_none() {
            self.result.SetNodeAndFragmentPosition(
                node.clone(),
                Some(fragment.ResultFragment()),
                *self.location.Point() - offset,
            );
        }
        bounds.Add(self.result, node, self.location) == ListBasedHitTestBehavior::kStopHitTesting
    }
    // Source: .h:238-246; .cc:2232-2241
    pub fn AddNodeToResultWithContentOffset<B: HitTestBounds>(
        &mut self,
        node: Option<NodeRef>,
        container: &dyn PhysicalFragment<L>,
        bounds: &B,
        mut offset: PhysicalOffset,
    ) -> bool {
        if container.Flags().scroll_container {
            offset += container.ScrolledContentOffset();
        }
        self.AddNodeToResult(node, container, bounds, offset)
    }
}

// Source: .h:337-340. These are references into the canonical native owners.
pub struct BoxFragmentPainter<'a, L: ?Sized> {
    box_fragment_: &'a dyn PhysicalFragment<L>,
    inline_box_cursor_: Option<InlineCursor<'a, L>>,
    box_item_: Option<&'a dyn FragmentItem<L>>,
}
impl<'a, L: ?Sized> BoxFragmentPainter<'a, L> {
    pub fn new(fragment: &'a dyn PhysicalFragment<L>) -> Self {
        Self {
            box_fragment_: fragment,
            inline_box_cursor_: None,
            box_item_: None,
        }
    }
    pub fn WithInlineItem(
        fragment: &'a dyn PhysicalFragment<L>,
        cursor: InlineCursor<'a, L>,
        item: &'a dyn FragmentItem<L>,
    ) -> Self {
        Self {
            box_fragment_: fragment,
            inline_box_cursor_: Some(cursor),
            box_item_: Some(item),
        }
    }

    // Source: .cc:598-607
    fn InkOverflowIncludingFilters(&self) -> PhysicalRect {
        if let Some(item) = self.box_item_ {
            return self
                .box_fragment_
                .Geometry()
                .ApplyFiltersToRect(item.SelfInkOverflowRect());
        }
        debug_assert!(!self.box_fragment_.Flags().inline_box);
        self.box_fragment_
            .Geometry()
            .VisualOverflowRectIncludingFilters()
    }

    // Source: .h:68-75; .cc:2243-2249
    pub fn NodeAtPoint(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        self.NodeAtPointWithInlineRoot(result, location, offset, offset, phase)
    }
    // Source: .h:76-80; .cc:2251-2259
    pub fn NodeAtPointWithInlineRoot(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        inline_root_offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        self.NodeAtPointWithContext(
            &mut HitTestContext {
                phase,
                location,
                inline_root_offset,
                result,
            },
            offset,
        )
    }

    // Source: .h:263-264; .cc:2261-2425
    pub fn NodeAtPointWithContext(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        offset: PhysicalOffset,
    ) -> bool {
        let fragment = self.box_fragment_;
        let flags = fragment.Flags();
        let geometry = fragment.Geometry();
        debug_assert!(geometry.MayIntersect(hit.result, hit.location, offset));
        if !flags.first_for_node && !flags.can_paint_multiple_fragments {
            return false;
        }
        if hit.phase == HitTestPhase::kForeground
            && !flags.self_painting_layer
            && self.HitTestOverflowControl(hit, offset)
        {
            return true;
        }
        let style = fragment.Style();
        let size = fragment.Size();
        let mut skip_children = hit
            .result
            .GetHitTestRequest()
            .GetStopNode()
            .is_some_and(|stop| geometry.MatchesStopNode(stop))
            || geometry.ChildPaintBlockedByDisplayLock();
        if !skip_children && flags.clip_overflow {
            if !flags.self_painting_layer {
                let mut clip = geometry.OverflowClipRect();
                clip.Move(&offset);
                if !hit.location.Intersects(&clip) {
                    skip_children = true;
                }
            }
            if !skip_children && style.HasBorderShape() {
                let mut rect = PhysicalRect::new(offset, size);
                if let Some([top, right, bottom, left]) = geometry.OverflowClipMarginOutsets() {
                    rect.ExpandEdges(top, right, bottom, left);
                }
                skip_children = !hit
                    .location
                    .IntersectsPath(geometry.BorderShapeOuterPath(rect).as_ref());
            } else if !skip_children && style.HasBorderRadius() {
                skip_children = !hit.location.IntersectsContouredRect(
                    geometry
                        .ContouredInnerBorder(PhysicalRect::new(offset, size))
                        .as_ref(),
                );
            }
        }
        if !skip_children {
            if !flags.scroll_container && !flags.non_overlay_overscroll_container {
                if self.HitTestChildren(hit, offset) {
                    return true;
                }
            } else {
                let scrolled =
                    offset - fragment.OverscrollContentOffset() - fragment.ScrolledContentOffset();
                let mut adjusted = HitTestContext {
                    phase: hit.phase,
                    location: hit.location,
                    inline_root_offset: scrolled,
                    result: &mut *hit.result,
                };
                if self.HitTestChildren(&mut adjusted, scrolled) {
                    return true;
                }
            }
        }
        if !hit.result.GetHitTestRequest().IsHitTestVisualOverflow() {
            if style.HasBorderShape() {
                if !hit.location.IntersectsPath(
                    geometry
                        .BorderShapeOuterPath(PhysicalRect::new(offset, size))
                        .as_ref(),
                ) {
                    return false;
                }
            } else if style.HasBorderRadius()
                && self.HitTestClippedOutByBorder(hit.location, offset)
            {
                return false;
            }
        }
        let mut pointer_events_bounding_box = false;
        let mut hit_self = fragment.IsInSelfHitTestingPhase(hit.phase);
        if hit_self {
            if flags.table_row || flags.table_section {
                hit_self = hit.result.GetHitTestRequest().IsHitTestVisualOverflow();
            } else if flags.svg_text {
                pointer_events_bounding_box =
                    style.UsedPointerEvents() == EPointerEvents::kBoundingBox;
                hit_self = pointer_events_bounding_box
                    || hit.result.GetHitTestRequest().IsHitTestVisualOverflow();
            }
        }
        if hit_self {
            if !IsVisibleToHitTestFragment(fragment, hit.result.GetHitTestRequest()) || flags.opaque
            {
                return false;
            }
        } else if flags.opaque
            && hit.result.HasListBasedResult()
            && IsVisibleToHitTestFragment(fragment, hit.result.GetHitTestRequest())
        {
            hit_self = true;
        }
        if hit_self {
            let mut bounds = PhysicalRect::new(offset, size);
            if hit.result.GetHitTestRequest().IsHitTestVisualOverflow() {
                bounds = self.InkOverflowIncludingFilters();
                bounds.Move(&offset);
            } else if pointer_events_bounding_box {
                bounds = PhysicalRect::EnclosingRect(&geometry.ObjectBoundingBox());
            }
            if flags.inline_box {
                bounds = PhysicalRect::from(foundation::ToPixelSnappedRect(bounds));
            }
            if hit.location.Intersects(&bounds) {
                if let Some(item) = self.box_item_.filter(|item| item.Flags().inline_box) {
                    let cursor = self
                        .inline_box_cursor_
                        .as_ref()
                        .expect("inline box item has its native cursor");
                    if hit.AddNodeToResultWithContentOffset(
                        fragment.NodeForHitTest(),
                        cursor.ContainerFragment(),
                        &bounds,
                        offset - item.OffsetInContainerFragment(),
                    ) {
                        return true;
                    }
                } else {
                    if self.UpdateHitTestResultForView(&bounds, hit) {
                        return true;
                    }
                    if hit.AddNodeToResult(fragment.NodeForHitTest(), fragment, &bounds, offset) {
                        return true;
                    }
                }
            }
        }
        false
    }

    // Source: .h:323-324; .cc:2427-2444
    fn UpdateHitTestResultForView(
        &self,
        bounds: &PhysicalRect,
        hit: &mut HitTestContext<'_, '_, '_, L>,
    ) -> bool {
        let fragment = self.box_fragment_;
        if !fragment.Geometry().IsLayoutView() || hit.result.InnerNode().is_some() {
            return false;
        }
        let Some(element) = fragment.Geometry().DocumentElement() else {
            return false;
        };
        let children = fragment.Children();
        for i in 0..children.len() {
            let child = children.at(i);
            if child
                .fragment
                .GetNode()
                .as_ref()
                .is_some_and(|node| node.DOMIdentity() == element.DOMIdentity())
            {
                return hit.AddNodeToResultWithContentOffset(
                    Some(element),
                    &*child.fragment,
                    bounds,
                    child.offset,
                );
            }
        }
        false
    }
    // Source: .h:82-84; .cc:2446-2468
    pub fn HitTestAllPhases(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        for phase in [
            HitTestPhase::kForeground,
            HitTestPhase::kFloat,
            HitTestPhase::kDescendantBlockBackgrounds,
            HitTestPhase::kSelfBlockBackground,
        ] {
            if self.NodeAtPoint(result, location, offset, phase) {
                return true;
            }
        }
        false
    }
    // Source: .h:302-304; .cc:2470-2511
    fn HitTestTextItem(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        item: &dyn FragmentItem<L>,
        cursor: &InlineCursor<'_, L>,
    ) -> bool {
        debug_assert!(item.Flags().kind == ItemKind::Text);
        if hit.phase != HitTestPhase::kForeground
            || !IsVisibleToHitTestItem(item, hit.result.GetHitTestRequest())
        {
            return false;
        }
        if item.Flags().svg_text && item.Flags().svg_bounding_box_transform {
            let quad = item.SvgUnscaledQuad();
            if !hit.location.Intersects(&quad) {
                return false;
            }
            return hit.AddNodeToResultWithContentOffset(
                item.NodeForHitTest(),
                cursor.ContainerFragment(),
                &quad,
                hit.inline_root_offset,
            );
        }
        let rect = self
            .box_fragment_
            .Geometry()
            .TextCombineBounds(item, hit.inline_root_offset)
            .unwrap_or_else(|| {
                item.ComputeTextBoundsRectForHitTest(
                    hit.inline_root_offset,
                    hit.result.GetHitTestRequest().IsHitTestVisualOverflow(),
                )
            });
        if !hit.location.Intersects(&rect) {
            return false;
        }
        hit.AddNodeToResultWithContentOffset(
            item.NodeForHitTest(),
            cursor.ContainerFragment(),
            &rect,
            hit.inline_root_offset,
        )
    }
    // Source: .h:309-312; .cc:2513-2584
    fn HitTestLineBoxFragment(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        fragment: &dyn PhysicalFragment<L>,
        cursor: &InlineCursor<'_, L>,
        offset: PhysicalOffset,
    ) -> bool {
        let item_value = cursor.Current().expect("line box cursor is positioned");
        let item = &*item_value;
        let mut overflow = item.InkOverflowRect();
        overflow.Move(&offset);
        if !hit.location.Intersects(&overflow) {
            return false;
        }
        if self.HitTestChildrenInItems(
            hit,
            self.box_fragment_,
            cursor.CursorForDescendants(),
            offset,
        ) {
            return true;
        }
        if hit.phase != HitTestPhase::kForeground
            || !IsVisibleToHitTestFragment(self.box_fragment_, hit.result.GetHitTestRequest())
        {
            return false;
        }
        let overflow_location = item.SelfInkOverflowRect().offset + offset;
        if self.HitTestClippedOutByBorder(hit.location, overflow_location) {
            return false;
        }
        let bounds = PhysicalRect::new(offset, fragment.Size());
        if self.box_fragment_.Style().HasBorderRadius()
            && !hit.location.IntersectsContouredRect(
                self.box_fragment_
                    .Geometry()
                    .ContouredBorder(bounds, false)
                    .as_ref(),
            )
        {
            return false;
        }
        if cursor.ContainerFragment().Flags().svg_text || !hit.location.Intersects(&bounds) {
            return false;
        }
        if fragment.Flags().floating_descendants {
            let mut result = HitTestResult::default();
            let mut float_hit = HitTestContext {
                phase: HitTestPhase::kFloat,
                location: hit.location,
                inline_root_offset: hit.inline_root_offset,
                result: &mut result,
            };
            if self.HitTestChildrenInItems(
                &mut float_hit,
                self.box_fragment_,
                cursor.CursorForDescendants(),
                offset,
            ) {
                return false;
            }
        }
        if fragment.Flags().block_in_inline {
            return false;
        }
        hit.AddNodeToResultWithContentOffset(
            fragment.NodeForHitTest(),
            self.box_fragment_,
            &bounds,
            offset - item.OffsetInContainerFragment(),
        )
    }
    // Source: .h:285-291; .cc:2586-2629
    fn HitTestInlineChildBoxFragment(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        fragment: &dyn PhysicalFragment<L>,
        cursor: &InlineCursor<'_, L>,
        offset: PhysicalOffset,
    ) -> bool {
        let flags = fragment.Flags();
        let atomic_pass = if flags.floating {
            if hit.phase != HitTestPhase::kFloat {
                return false;
            }
            true
        } else {
            hit.phase == HitTestPhase::kForeground
        };
        if flags.painted_atomically {
            return atomic_pass
                && HitTestAllPhasesInFragment(fragment, hit.location, offset, hit.result);
        }
        if !fragment
            .Geometry()
            .MayIntersect(hit.result, hit.location, offset)
        {
            return false;
        }
        if flags.inline_box {
            let item_value = cursor.Current().expect("inline fragment cursor has item");
            let item = &*item_value;
            return BoxFragmentPainter::WithInlineItem(fragment, cursor.clone(), item)
                .NodeAtPointWithContext(hit, offset);
        }
        debug_assert!(flags.block_in_inline);
        BoxFragmentPainter::new(fragment).NodeAtPointWithContext(hit, offset)
    }
    // Source: .h:292-295; .cc:2631-2669
    fn HitTestChildBoxItem(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        container: &dyn PhysicalFragment<L>,
        item: &dyn FragmentItem<L>,
        cursor: &InlineCursor<'_, L>,
    ) -> bool {
        if !item.Flags().svg_inline {
            let fragment_value = item
                .BoxFragment()
                .expect("non-SVG box item has native box fragment");
            let fragment = &*fragment_value;
            return self.HitTestInlineChildBoxFragment(
                hit,
                fragment,
                cursor,
                hit.inline_root_offset + item.OffsetInContainerFragment(),
            );
        }
        let descendants = cursor.CursorForDescendants();
        if descendants.Current().is_some() && self.HitTestItemsChildren(hit, container, descendants)
        {
            return true;
        }
        debug_assert!(cursor.ContainerFragment().Flags().svg_text);
        if item.Style().UsedPointerEvents() != EPointerEvents::kBoundingBox
            || hit.phase != HitTestPhase::kForeground
            || !IsVisibleToHitTestItem(item, hit.result.GetHitTestRequest())
        {
            return false;
        }
        let bounds = PhysicalRect::EnclosingRect(&item.ObjectBoundingBox());
        hit.location.Intersects(&bounds)
            && hit.AddNodeToResultWithContentOffset(
                item.NodeForHitTest(),
                cursor.ContainerFragment(),
                &bounds,
                bounds.offset,
            )
    }
    // Source: .h:265-266; .cc:2671-2695
    fn HitTestChildren(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        offset: PhysicalOffset,
    ) -> bool {
        if let Some(cursor) = self.inline_box_cursor_.as_ref() {
            return self.HitTestChildrenInItems(
                hit,
                self.box_fragment_,
                cursor.CursorForDescendants(),
                offset,
            );
        }
        if let Some(cursor) = self.box_fragment_.Items() {
            return self.HitTestChildrenInItems(hit, self.box_fragment_, cursor, offset);
        }
        if hit.phase == HitTestPhase::kFloat {
            return self.box_fragment_.Flags().floating_descendants
                && self.HitTestFloatingChildren(hit, self.box_fragment_, offset);
        }
        self.HitTestBlockChildren(hit.result, hit.location, offset, hit.phase)
    }
    // Source: .h:267-270; .cc:2697-2706
    fn HitTestChildrenInItems(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        container: &dyn PhysicalFragment<L>,
        children: InlineCursor<'_, L>,
        _offset: PhysicalOffset,
    ) -> bool {
        children.Current().is_some() && self.HitTestItemsChildren(hit, container, children)
    }
    // Source: .h:271-274; .cc:2708-2774
    fn HitTestBlockChildren(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        mut phase: HitTestPhase,
    ) -> bool {
        if phase == HitTestPhase::kDescendantBlockBackgrounds {
            phase = HitTestPhase::kSelfBlockBackground;
        }
        let children = self.box_fragment_.Children();
        for i in (0..children.len()).rev() {
            let child = children.at(i);
            let fragment = &*child.fragment;
            let flags = fragment.Flags();
            debug_assert!(flags.is_box);
            if flags.destroyed_or_moved {
                continue;
            }
            let child_offset = offset + child.offset;
            if flags.self_painting_layer {
                if HitTestReplacedNormalFlowInline(fragment, result, location, child_offset, phase)
                {
                    return true;
                }
                continue;
            }
            if flags.floating {
                continue;
            }
            if flags.painted_atomically {
                if phase != HitTestPhase::kForeground
                    || !HitTestAllPhasesInFragment(fragment, location, child_offset, result)
                {
                    continue;
                }
            } else if !NodeAtPointInFragment(fragment, location, child_offset, phase, result) {
                continue;
            }
            if result.InnerNode().is_some() {
                return true;
            }
            if let Some(node) = fragment.NodeForHitTest() {
                result.SetNodeAndFragmentPosition(
                    Some(node),
                    Some(fragment.ResultFragment()),
                    *location.Point() - offset,
                );
                return true;
            }
            let Some(node) = self.box_fragment_.NodeForHitTest() else {
                return true;
            };
            let scrolled = if self.box_fragment_.Flags().scroll_container {
                self.box_fragment_.ScrolledContentOffset()
            } else {
                PhysicalOffset::default()
            };
            result.SetNodeAndFragmentPosition(
                Some(node),
                Some(self.box_fragment_.ResultFragment()),
                *location.Point() - offset - scrolled,
            );
            return true;
        }
        false
    }
    // Source: .h:334-335; .cc:2776-2799
    fn ShouldHitTestCulledInlineAncestors(
        hit: &HitTestContext<'_, '_, '_, L>,
        item: &dyn FragmentItem<L>,
    ) -> bool {
        if hit.phase != HitTestPhase::kForeground || item.Flags().kind == ItemKind::Line {
            return false;
        }
        if hit.result.GetHitTestRequest().ListBased() {
            return true;
        }
        !item.Flags().block_in_inline
    }
    // Source: .h:275-277; .cc:2801-2869
    fn HitTestItemsChildren(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        container: &dyn PhysicalFragment<L>,
        children: InlineCursor<'_, L>,
    ) -> bool {
        let mut cursor = children.clone();
        while let Some(item_value) = cursor.Current() {
            let item = &*item_value;
            if item.Flags().destroyed_or_moved {
                // The retained Layout result can outlive a moved source
                // object until the next lifecycle commit. Chromium marks this
                // NOTREACHED; the safe resident projection must not dereference
                // that dead object while walking the previous artifact.
                cursor.MoveToPreviousSibling();
                continue;
            }
            if item.Flags().self_painting_layer {
                if let Some(fragment_value) = item.BoxFragment() {
                    let fragment = &*fragment_value;
                    if HitTestReplacedNormalFlowInline(
                        fragment,
                        hit.result,
                        hit.location,
                        hit.inline_root_offset + item.OffsetInContainerFragment(),
                        hit.phase,
                    ) {
                        return true;
                    }
                }
                cursor.MoveToPreviousSibling();
                continue;
            }
            match item.Flags().kind {
                ItemKind::Text => {
                    if self.HitTestTextItem(hit, item, &cursor) {
                        return true;
                    }
                }
                ItemKind::Line => {
                    if let Some(fragment_value) = item.LineBoxFragment() {
                        let fragment = &*fragment_value;
                        if self.HitTestLineBoxFragment(
                            hit,
                            fragment,
                            &cursor,
                            hit.inline_root_offset + item.OffsetInContainerFragment(),
                        ) {
                            return true;
                        }
                    } else if self.HitTestItemsChildren(
                        hit,
                        container,
                        cursor.CursorForDescendants(),
                    ) {
                        return true;
                    }
                }
                ItemKind::Box => {
                    if self.HitTestChildBoxItem(hit, container, item, &cursor) {
                        return true;
                    }
                }
            }
            cursor.MoveToPreviousSibling();
            if Self::ShouldHitTestCulledInlineAncestors(hit, item) {
                let child_offset = hit.inline_root_offset + item.OffsetInContainerFragment();
                if HitTestCulledInlineAncestorsForItem(
                    hit.result,
                    container,
                    children.clone(),
                    item,
                    cursor.Current().as_deref(),
                    hit.location,
                    child_offset,
                ) {
                    return true;
                }
            }
        }
        false
    }
    // Source: .h:278-280; .cc:2871-2940
    fn HitTestFloatingChildren(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        container: &dyn PhysicalFragment<L>,
        offset: PhysicalOffset,
    ) -> bool {
        debug_assert!(hit.phase == HitTestPhase::kFloat && container.Flags().floating_descendants);
        if let Some(items) = container.Items() {
            if self.HitTestFloatingChildItems(hit, items, offset) {
                return true;
            }
        }
        let children = container.Children();
        for i in (0..children.len()).rev() {
            let child = children.at(i);
            let fragment = &*child.fragment;
            let flags = fragment.Flags();
            if flags.destroyed_or_moved || flags.self_painting_layer || !flags.is_box {
                continue;
            }
            let child_offset = offset + child.offset;
            if flags.floating {
                if HitTestAllPhasesInFragment(fragment, hit.location, child_offset, hit.result) {
                    return true;
                }
                continue;
            }
            if flags.painted_atomically || !flags.floating_descendants {
                continue;
            }
            if flags.non_visible_overflow {
                if NodeAtPointInFragment(
                    fragment,
                    hit.location,
                    child_offset,
                    HitTestPhase::kFloat,
                    hit.result,
                ) {
                    return true;
                }
                continue;
            }
            if self.HitTestFloatingChildren(hit, fragment, child_offset) {
                return true;
            }
        }
        false
    }
    // Source: .h:281-283; .cc:2942-2998
    fn HitTestFloatingChildItems(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        children: InlineCursor<'_, L>,
        offset: PhysicalOffset,
    ) -> bool {
        let mut cursor = children.clone();
        while let Some(item_value) = cursor.Current() {
            let item = &*item_value;
            if item.Flags().destroyed_or_moved {
                cursor.MoveToPreviousSibling();
                continue;
            }
            match item.Flags().kind {
                ItemKind::Box => {
                    if let Some(fragment_value) = item.BoxFragment() {
                        let fragment = &*fragment_value;
                        let flags = fragment.Flags();
                        if flags.self_painting_layer {
                            cursor.MoveToPreviousSibling();
                            continue;
                        }
                        let child_offset = offset + item.OffsetInContainerFragment();
                        if flags.floating {
                            if HitTestAllPhasesInFragment(
                                fragment,
                                hit.location,
                                child_offset,
                                hit.result,
                            ) {
                                return true;
                            }
                            cursor.MoveToPreviousSibling();
                            continue;
                        }
                        if flags.painted_atomically {
                            cursor.MoveToPreviousSibling();
                            continue;
                        }
                        debug_assert!(flags.inline_box || flags.block_in_inline);
                        if !flags.inline_box {
                            if flags.floating_descendants
                                && self.HitTestFloatingChildren(hit, fragment, child_offset)
                            {
                                return true;
                            }
                            cursor.MoveToPreviousSibling();
                            continue;
                        }
                    }
                    debug_assert!(item.LayoutObject().IsLayoutInline());
                }
                ItemKind::Line => {
                    if item
                        .LineBoxFragment()
                        .is_some_and(|line| !line.Flags().floating_descendants)
                    {
                        cursor.MoveToPreviousSibling();
                        continue;
                    }
                }
                ItemKind::Text => {
                    cursor.MoveToPreviousSibling();
                    continue;
                }
            }
            if self.HitTestFloatingChildItems(hit, cursor.CursorForDescendants(), offset) {
                return true;
            }
            cursor.MoveToPreviousSibling();
        }
        false
    }
    // Source: .h:314-318; .cc:3000-3009
    fn HitTestClippedOutByBorder(
        &self,
        location: &HitTestLocation,
        border_box_location: PhysicalOffset,
    ) -> bool {
        let rect = PhysicalRect::new(border_box_location, self.box_fragment_.Size());
        !location.IntersectsContouredRect(
            self.box_fragment_
                .Geometry()
                .ContouredBorder(rect, true)
                .as_ref(),
        )
    }
    // Source: .h:320-321; .cc:3011-3019
    fn HitTestOverflowControl(
        &self,
        hit: &mut HitTestContext<'_, '_, '_, L>,
        offset: PhysicalOffset,
    ) -> bool {
        self.box_fragment_
            .Geometry()
            .HitTestOverflowControl(hit.result, hit.location, offset)
    }
}

// Source: .h:297-301 declares HitTestTextFragment but this revision has no
// out-of-line definition; the production text path is HitTestTextItem above.

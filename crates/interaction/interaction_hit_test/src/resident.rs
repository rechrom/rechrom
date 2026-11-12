//! Call-scoped projections of Layout's resident PhysicalFragment/FragmentItems.
//!
//! No fragment directory, geometry snapshot, or child vector is constructed.
//! Source traversal remains in BoxFragmentPainter. Values returned by its
//! ports carry only the borrowed source cursor and the same query services.
//!
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Borrowed operations project physical_fragment.h:298-308,363,384-407,549-569;
//! physical_box_fragment.h:353-366,402-423; fragment_item.h:196-223,263-280,
//! 496-509; and inline_cursor.h:676-706. Traversal branches stay in the
//! translated core/paint/box_fragment_painter.rs source owner.
//!
//! This required-services adapter is the production Page hit-test route.
//! Source layer traversal, DOM result owners and virtual geometry operations
//! are supplied by the call-scoped resident binding below.
use crate::{
    core::{
        layout::{
            hit_test_location::HitTestLocation,
            hit_test_phase::HitTestPhase,
            hit_test_request::HitTestStopNode,
            hit_test_result::{HitTestBoxFragment, HitTestResult, NodeRef},
        },
        paint::box_fragment_painter::*,
    },
    platform::geometry::{contoured_rect::ContouredRectHitTest, path::PathHitTest},
};
use foundation::{gfx, EPointerEvents, EVisibility, PhysicalOffset, PhysicalRect, PhysicalSize};
use layoutng_assembly::{
    fragment_item::FragmentItem as NativeItem,
    internal::layout_object::LayoutObject,
    layout_engine::{FragmentCursor, PaintFragmentCursor},
    physical_box_fragment::PhysicalBoxFragment,
    physical_fragment::PhysicalFragment as NativePhysicalFragment,
};
use std::marker::PhantomData;

/// DOM and the existing geometry owners supply their canonical query results.
/// This port does not select descendants, phases, targets, or cursor policy.
pub trait ResidentHitTestServices<L: ?Sized> {
    fn RecordNativeHitTestResult(
        &self,
        result: crate::core::layout::native_hit_test::NativeHitTestResult,
    ) -> bool;
    fn NativeClipPathHitTest(
        &self,
        source: layoutng_assembly::layout_engine::NativeHitTestCursor<'_>,
        location: &HitTestLocation,
    ) -> bool;
    fn SvgForeignObjectLayerHitTest(
        &self,
        source: layoutng_assembly::layout_engine::NativeHitTestCursor<'_>,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
    ) -> crate::core::layout::native_hit_test::NativeHitTestResult;
    /// Borrow only an already prepared local child query. Absence of that
    /// lease for a real local Frame must be reported as a required port.
    fn EmbeddedChildFrameView(
        &self,
        owner: &layoutng_assembly::internal::layout_node_metadata::Node,
    ) -> Result<
        crate::core::layout::layout_embedded_content_hit_test::ChildFrameView<'_>,
        crate::core::layout::native_hit_test::NativeHitTestRequiredPort,
    >;
    fn NewInlineHitTestRegion(
        &self,
    ) -> Box<dyn crate::core::layout::layout_inline::InlineHitTestRegion>;
    fn MayIntersect(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn ShouldPaintReplacedNormalFlowInline(&self, cursor: PaintFragmentCursor<'_>) -> bool;
    fn VisualOverflowRectIncludingFilters(&self, cursor: PaintFragmentCursor<'_>) -> PhysicalRect;

    fn ResolveNode(&self, id: u64) -> Option<NodeRef>;
    fn ResolveNativeNode(
        &self,
        node: &layoutng_assembly::internal::layout_node_metadata::Node,
    ) -> Option<NodeRef>;
    fn DocumentElement(&self) -> Option<NodeRef>;
    fn ObjectBoundingBox(&self, cursor: PaintFragmentCursor<'_>) -> gfx::RectF;
    fn ContouredBorder(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
        inner: bool,
        include_fragment_sides: bool,
    ) -> Box<dyn ContouredRectHitTest + '_>;
    fn BorderShapeOuterPath(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
    ) -> Box<dyn PathHitTest + '_>;
    fn ApplyFiltersToRect(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
    ) -> PhysicalRect;
    fn TextCombineBounds(
        &self,
        cursor: PaintFragmentCursor<'_>,
        item: &dyn FragmentItem<L>,
        offset: PhysicalOffset,
    ) -> Option<PhysicalRect>;
    fn LegacyNodeAtPoint(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn LegacyHitTestAllPhases(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn HitTestReplacedNormalFlowInline(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
    ) -> bool;
    fn HitTestOverflowControl(
        &self,
        cursor: PaintFragmentCursor<'_>,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn HitTestCulledInline(
        &self,
        cursor: PaintFragmentCursor<'_>,
        parent: &dyn CulledInlineLayout,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        inline_cursor: InlineCursor<'_, L>,
    ) -> bool;
}

pub struct ResidentFragment<'a, L: ?Sized> {
    /// Paint client identity. For inline child boxes this remains the
    /// FragmentItem even while `physical` supplies the linked box geometry.
    cursor: PaintFragmentCursor<'a>,
    physical: Option<&'a NativePhysicalFragment>,
    services: &'a dyn ResidentHitTestServices<L>,
    marker: PhantomData<&'a L>,
}
impl<L: ?Sized> Copy for ResidentFragment<'_, L> {}
impl<L: ?Sized> Clone for ResidentFragment<'_, L> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, L: ?Sized> ResidentFragment<'a, L> {
    pub fn new(
        cursor: PaintFragmentCursor<'a>,
        services: &'a dyn ResidentHitTestServices<L>,
    ) -> Self {
        Self {
            cursor,
            physical: None,
            services,
            marker: PhantomData,
        }
    }
    pub fn Cursor(self) -> PaintFragmentCursor<'a> {
        self.cursor
    }
    fn Rebind(&self, cursor: PaintFragmentCursor<'a>) -> Self {
        Self::new(cursor, self.services)
    }
    fn RebindPhysical(&self, physical: &'a NativePhysicalFragment) -> Self {
        Self {
            cursor: self.cursor,
            physical: Some(physical),
            services: self.services,
            marker: PhantomData,
        }
    }
    fn Physical(&self) -> Option<&'a NativePhysicalFragment> {
        self.physical
            .or_else(|| self.cursor.Cursor().PhysicalFragment())
    }
    fn Object(&self) -> &LayoutObject {
        let object = if let Some(fragment) = self.physical {
            fragment.GetSelfOrContainerLayoutObject()
        } else {
            match self.cursor.Cursor() {
                FragmentCursor::Physical { fragment, .. } => {
                    fragment.GetSelfOrContainerLayoutObject()
                }
                FragmentCursor::Inline { item, .. } => item.GetLayoutObject(),
            }
        };
        unsafe { object.as_ref() }.expect("resident layout owner")
    }
    fn Box(&self) -> Option<&'a PhysicalBoxFragment> {
        let fragment = self.Physical()?;
        fragment
            .IsBox()
            .then(|| unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) })
    }
    fn Item(&self) -> &'a NativeItem {
        self.cursor
            .Cursor()
            .InlineItem()
            .expect("positioned native fragment item")
    }
    fn ItemOwner(&self) -> Option<(&'a PhysicalBoxFragment, &'a [NativeItem])> {
        if self.physical.is_some() {
            let box_ = self.Box()?;
            return unsafe { box_.Items().as_ref() }.map(|items| (box_, items.Items()));
        }
        match self.cursor.Cursor() {
            FragmentCursor::Inline {
                container, items, ..
            } => Some((container, items.Items())),
            FragmentCursor::Physical { .. } => {
                let box_ = self.Box()?;
                unsafe { box_.Items().as_ref() }.map(|items| (box_, items.Items()))
            }
        }
    }
    fn Node(
        &self,
        node: *mut layoutng_assembly::internal::layout_node_metadata::Node,
    ) -> Option<NodeRef> {
        let node = unsafe { node.as_ref() }?;
        self.services.ResolveNativeNode(node)
    }
}
macro_rules! native_style {
    ($value:expr) => {{
        if let Some(fragment) = $value.physical {
            fragment.Style()
        } else {
            match $value.cursor.Cursor() {
                FragmentCursor::Physical { fragment, .. } => fragment.Style(),
                FragmentCursor::Inline { item, .. } => item.Style(),
            }
        }
    }};
}
impl<L: ?Sized> HitTestStyle for ResidentFragment<'_, L> {
    fn Visibility(&self) -> EVisibility {
        native_style!(self).Visibility()
    }
    fn UsedPointerEvents(&self) -> EPointerEvents {
        native_style!(self).UsedPointerEvents()
    }
    fn IsTableBackgroundDisplay(&self) -> bool {
        matches!(
            native_style!(self).Display(),
            foundation::EDisplay::kTableRowGroup
                | foundation::EDisplay::kTableRow
                | foundation::EDisplay::kTableColumn
                | foundation::EDisplay::kTableColumnGroup
        )
    }
    fn HasBorderShape(&self) -> bool {
        native_style!(self).HasBorderShape()
    }
    fn HasBorderRadius(&self) -> bool {
        native_style!(self).HasBorderRadius()
    }
    fn HasStroke(&self) -> bool {
        native_style!(self).HasStroke()
    }
    fn HasFill(&self) -> bool {
        native_style!(self).HasFill()
    }
}
impl CulledInlineLayout for LayoutObject {
    fn Identity(&self) -> usize {
        self as *const _ as usize
    }
    fn Parent(&self) -> Option<&dyn CulledInlineLayout> {
        unsafe { self.Parent().as_ref() }.map(|p| p as _)
    }
    fn HasPreviousOrNextSibling(&self) -> bool {
        !self.PreviousSibling().is_null() || !self.NextSibling().is_null()
    }
    fn IsDescendantOf(&self, ancestor: &dyn CulledInlineLayout) -> bool {
        let mut parent = self.Parent();
        while let Some(object) = unsafe { parent.as_ref() } {
            if object as *const _ as usize == ancestor.Identity() {
                return true;
            }
            parent = object.Parent();
        }
        false
    }
    fn IsLayoutInline(&self) -> bool {
        self.IsLayoutInline()
    }
}
impl<L: ?Sized> HitTestBoxFragment for ResidentFragment<'_, L> {
    fn OffsetFromOwnerLayoutBox(&self) -> PhysicalOffset {
        self.Box().map_or(
            PhysicalOffset::default(),
            PhysicalBoxFragment::OffsetFromOwnerLayoutBox,
        )
    }
}
impl<L: ?Sized> PhysicalFragment<L> for ResidentFragment<'_, L> {
    fn Flags(&self) -> FragmentFlags {
        let data = &self.cursor.paint;
        let physical = self.Physical();
        FragmentFlags {
            can_traverse: self.Object().CanTraversePhysicalFragments(),
            first_for_node: self.Box().is_none_or(PhysicalBoxFragment::IsFirstForNode),
            can_paint_multiple_fragments: data.can_paint_multiple_fragments,
            hidden_for_paint: physical.is_some_and(|fragment| fragment.IsHiddenForPaint()),
            atomic_inline: physical.is_some_and(|f| f.IsAtomicInline()),
            self_painting_layer: physical.is_some_and(|fragment| fragment.HasSelfPaintingLayer()),
            table_part: physical.is_some_and(|f| f.IsTablePart()),
            table_row: self.Object().IsTableRow(),
            table_section: self.Object().IsTableSection(),
            svg_text: self.Object().IsSVGText(),
            opaque: physical.is_some_and(|f| f.IsOpaque()),
            inline_box: physical.is_some_and(|f| f.IsInlineBox()),
            block_in_inline: physical.is_some_and(|f| f.IsBlockInInline()),
            floating: physical.is_some_and(|f| f.IsFloating()),
            painted_atomically: physical.is_some_and(|f| f.IsPaintedAtomically()),
            floating_descendants: physical.is_some_and(|f| f.HasFloatingDescendantsForPaint()),
            destroyed_or_moved: physical.is_some_and(|f| f.IsLayoutObjectDestroyedOrMoved()),
            clip_overflow: data.hit_test_overflow_clip.is_some(),
            scroll_container: self.Object().IsScrollContainer(),
            non_overlay_overscroll_container: physical
                .is_some_and(|fragment| fragment.IsNonOverlayOverscrollScrollContainer()),
            non_visible_overflow: self.Object().ShouldClipOverflowAlongEitherAxis(),
            is_box: physical.is_some_and(|f| f.IsBox()),
        }
    }
    fn Size(&self) -> PhysicalSize {
        self.Physical()
            .map_or_else(|| self.cursor.Cursor().Size(), NativePhysicalFragment::Size)
    }
    fn Style(&self) -> &dyn HitTestStyle {
        self
    }
    fn Geometry(&self) -> &dyn FragmentGeometryQueries<L> {
        self
    }
    fn ResultFragment(&self) -> &dyn HitTestBoxFragment {
        self
    }
    fn NodeForHitTest(&self) -> Option<NodeRef> {
        let node = self.Physical().map_or_else(
            || self.Object().NodeForHitTest(),
            |fragment| fragment.NodeForHitTest(),
        );
        self.Node(node)
    }
    fn GetNode(&self) -> Option<NodeRef> {
        let node = self
            .Physical()
            .map_or_else(|| self.Object().GetNode(), NativePhysicalFragment::GetNode);
        self.Node(node)
    }
    fn SelfOrContainerLayoutObject(&self) -> &dyn CulledInlineLayout {
        let object = self
            .Physical()
            .map(|fragment| fragment.GetSelfOrContainerLayoutObject())
            .unwrap_or_else(|| self.Object() as *const LayoutObject as *mut LayoutObject);
        unsafe { object.as_ref() }.expect("resident physical fragment layout owner")
    }
    fn Children(&self) -> &dyn FragmentChildren<L> {
        self
    }
    fn Items(&self) -> Option<InlineCursor<'_, L>> {
        self.ItemOwner().map(|_| InlineCursor::Root(self))
    }
    fn ScrolledContentOffset(&self) -> PhysicalOffset {
        offset(self.cursor.paint.pixel_snapped_scrolled_content_offset)
    }
    fn OverscrollContentOffset(&self) -> PhysicalOffset {
        self.Box().map_or(PhysicalOffset::default(), |box_| {
            let offset = box_.PixelSnappedOverscrollContentOffset();
            PhysicalOffset::FromPoint(&gfx::Point::new(offset.x(), offset.y()))
        })
    }
    fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool {
        let native_phase = match phase {
            HitTestPhase::kForeground => layoutng_assembly::internal::hit_test_phase::HitTestPhase::kForeground,
            HitTestPhase::kFloat => layoutng_assembly::internal::hit_test_phase::HitTestPhase::kFloat,
            HitTestPhase::kDescendantBlockBackgrounds => layoutng_assembly::internal::hit_test_phase::HitTestPhase::kDescendantBlockBackgrounds,
            HitTestPhase::kSelfBlockBackground => layoutng_assembly::internal::hit_test_phase::HitTestPhase::kSelfBlockBackground,
        };
        self.Physical()
            .is_some_and(|fragment| fragment.IsInSelfHitTestingPhase(native_phase))
    }
}
impl<L: ?Sized> FragmentChildren<L> for ResidentFragment<'_, L> {
    fn len(&self) -> usize {
        self.Box().map_or(0, |box_| box_.Children().len())
    }
    fn at(&self, index: usize) -> FragmentLink<'_, L> {
        let link = &self.Box().expect("physical child owner").Children()[index];
        let cursor = self
            .cursor
            .ForPhysicalFragment(unsafe { &*link.fragment.Get() });
        FragmentLink {
            fragment: PhysicalFragmentRef::Resident(self.Rebind(cursor)),
            offset: link.offset,
        }
    }
}
impl<L: ?Sized> FragmentItems<L> for ResidentFragment<'_, L> {
    fn ContainerFragment(&self) -> &dyn PhysicalFragment<L> {
        self
    }
    fn At(&self, position: usize) -> FragmentItemRef<'_, L> {
        let (container, _) = self.ItemOwner().expect("native FragmentItems owner");
        FragmentItemRef::Resident(self.Rebind(self.cursor.ForInlineItem(container, position)))
    }
    fn SiblingItemIndices(&self, parent: Option<usize>) -> Vec<usize> {
        let Some((container, items)) = self.ItemOwner() else {
            return Vec::new();
        };
        let mut cursor = layoutng_assembly::inline_cursor::InlineCursor::from_fragment(container);
        if let Some(parent) = parent {
            cursor.MoveToFragmentItem(&items[parent]);
            cursor = cursor.CursorForDescendants();
        }
        let mut indices = layoutng_assembly::inline_cursor::InlineBackwardCursor::new(&cursor)
            .IntoSiblingItemIndices();
        if let Some(parent) = parent {
            // Native descendant cursors root their item span at parent + 1.
            // Project positions into the unchanged canonical FragmentItems.
            for index in &mut indices {
                *index += parent + 1;
            }
        }
        indices
    }
}
impl<L: ?Sized> FragmentItem<L> for ResidentFragment<'_, L> {
    fn Flags(&self) -> ItemFlags {
        let item = self.Item();
        ItemFlags {
            kind: if item.IsText() {
                ItemKind::Text
            } else if item.Type() == NativeItem::kLine {
                ItemKind::Line
            } else {
                ItemKind::Box
            },
            hidden_for_paint: item.IsHiddenForPaint(),
            svg_text: item.IsSvgText(),
            svg_bounding_box_transform: self
                .cursor
                .paint
                .svg_text
                .as_ref()
                .is_some_and(|text| text.has_transform),
            svg_inline: self.Object().IsSVGInline(),
            self_painting_layer: item.HasSelfPaintingLayer(),
            floating: item.IsFloating(),
            inline_box: item.IsInlineBox(),
            block_in_inline: item.IsBlockInInline(),
            ellipsis: item.IsEllipsis(),
            destroyed_or_moved: item.IsLayoutObjectDestroyedOrMoved(),
        }
    }
    fn Style(&self) -> &dyn HitTestStyle {
        self
    }
    fn NodeForHitTest(&self) -> Option<NodeRef> {
        self.Node(self.Item().NodeForHitTest())
    }
    fn LayoutObject(&self) -> &dyn CulledInlineLayout {
        self.Object()
    }
    fn BoxFragment(&self) -> Option<PhysicalFragmentRef<'_, L>> {
        // FragmentItem remains the Paint client for an inline child box. Its
        // linked PhysicalFragment supplies geometry, but rebinding the cursor
        // to that fragment would discard the item's canonical Paint sidecar.
        let fragment = unsafe { self.Item().BoxFragment().as_ref() }?;
        Some(PhysicalFragmentRef::Resident(self.RebindPhysical(fragment)))
    }
    fn LineBoxFragment(&self) -> Option<PhysicalFragmentRef<'_, L>> {
        let fragment = unsafe { self.Item().LineBoxFragment().as_ref() }?;
        Some(PhysicalFragmentRef::Resident(self.RebindPhysical(fragment)))
    }
    fn OffsetInContainerFragment(&self) -> PhysicalOffset {
        *self.Item().OffsetInContainerFragment()
    }
    fn InkOverflowRect(&self) -> PhysicalRect {
        self.Item().InkOverflowRect()
    }
    fn SelfInkOverflowRect(&self) -> PhysicalRect {
        self.Item().SelfInkOverflowRect()
    }
    fn SvgUnscaledQuad(&self) -> gfx::QuadF {
        self.Item().SvgUnscaledQuad()
    }
    fn ObjectBoundingBox(&self) -> gfx::RectF {
        self.Item()
            .ObjectBoundingBox(unsafe { &*self.ItemOwner().expect("SVG FragmentItems").0.Items() })
    }
    fn ComputeTextBoundsRectForHitTest(
        &self,
        offset: PhysicalOffset,
        visual_overflow: bool,
    ) -> PhysicalRect {
        self.Item()
            .ComputeTextBoundsRectForHitTest(offset, visual_overflow)
    }
}
fn offset(value: layoutng_assembly::internal::layout_input::Offset) -> PhysicalOffset {
    PhysicalOffset::new(
        foundation::LayoutUnit::FromDoubleRound(value.x),
        foundation::LayoutUnit::FromDoubleRound(value.y),
    )
}
impl<L: ?Sized> FragmentGeometryQueries<L> for ResidentFragment<'_, L> {
    fn MayIntersect(
        &self,
        result: &HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.services
            .MayIntersect(self.cursor, result, location, offset)
    }
    fn MatchesStopNode(&self, stop: &dyn HitTestStopNode<L>) -> bool {
        stop.Identity() == self.Object() as *const _ as usize
    }
    fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.Object().ChildPaintBlockedByDisplayLock()
    }
    fn AtomicInlineItemHiddenForPaint(&self) -> bool {
        let mut cursor = layoutng_assembly::inline_cursor::InlineCursor::default();
        cursor.MoveToLayoutObject(self.Object());
        cursor.IsNotNull() && cursor.Current().IsHiddenForPaint()
    }
    fn IsLayoutView(&self) -> bool {
        self.Object().IsLayoutView()
    }
    fn DocumentElement(&self) -> Option<NodeRef> {
        self.services.DocumentElement()
    }
    fn ObjectBoundingBox(&self) -> gfx::RectF {
        self.services.ObjectBoundingBox(self.cursor)
    }
    fn OverflowClipRect(&self) -> PhysicalRect {
        self.Box()
            .expect("overflow clipping physical box")
            .OverflowClipRect(
                foundation::OverlayScrollbarClipBehavior::kExcludeOverlayScrollbarSizeForHitTesting,
            )
    }
    fn OverflowClipMarginOutsets(&self) -> Option<[foundation::LayoutUnit; 4]> {
        let box_ = self
            .Box()
            .filter(|box_| box_.ShouldApplyOverflowClipMargin())?;
        let outsets = box_.OverflowClipMarginOutsets();
        Some([outsets.top, outsets.right, outsets.bottom, outsets.left])
    }
    fn BorderShapeOuterPath(&self, rect: PhysicalRect) -> Box<dyn PathHitTest + '_> {
        self.services.BorderShapeOuterPath(self.cursor, rect)
    }
    fn ContouredInnerBorder(&self, rect: PhysicalRect) -> Box<dyn ContouredRectHitTest + '_> {
        self.services.ContouredBorder(self.cursor, rect, true, true)
    }
    fn ContouredBorder(
        &self,
        rect: PhysicalRect,
        sides: bool,
    ) -> Box<dyn ContouredRectHitTest + '_> {
        self.services
            .ContouredBorder(self.cursor, rect, false, sides)
    }
    fn ApplyFiltersToRect(&self, rect: PhysicalRect) -> PhysicalRect {
        self.services.ApplyFiltersToRect(self.cursor, rect)
    }
    fn VisualOverflowRectIncludingFilters(&self) -> PhysicalRect {
        self.services
            .VisualOverflowRectIncludingFilters(self.cursor)
    }
    fn TextCombineBounds(
        &self,
        item: &dyn FragmentItem<L>,
        offset: PhysicalOffset,
    ) -> Option<PhysicalRect> {
        self.services.TextCombineBounds(self.cursor, item, offset)
    }
    fn HasLayer(&self) -> bool {
        self.Object().HasLayer()
    }
    fn ShouldPaintReplacedNormalFlowInline(&self) -> bool {
        self.services
            .ShouldPaintReplacedNormalFlowInline(self.cursor)
    }
    fn HitTestReplacedNormalFlowInline(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
    ) -> bool {
        self.services
            .HitTestReplacedNormalFlowInline(self.cursor, result, location)
    }
    fn LegacyHitTestAllPhases(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.services
            .LegacyHitTestAllPhases(self.cursor, result, location, offset)
    }
    fn LegacyNodeAtPoint(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        self.services
            .LegacyNodeAtPoint(self.cursor, result, location, offset, phase)
    }
    fn HitTestOverflowControl(
        &self,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.services
            .HitTestOverflowControl(self.cursor, result, location, offset)
    }
    fn HitTestCulledInline(
        &self,
        parent: &dyn CulledInlineLayout,
        result: &mut HitTestResult<'_, L>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        cursor: InlineCursor<'_, L>,
    ) -> bool {
        self.services
            .HitTestCulledInline(self.cursor, parent, result, location, offset, cursor)
    }
}

// Source PaintLayer projections consume the same committed owner borrowed by
// PrePaint. No layer graph, property tree or fragment geometry is duplicated.
use crate::core::paint::{
    hit_testing_transform_state::TransformPaintPropertyNode as HitTransform,
    paint_layer::PaintLayerIteration, paint_layer_hit_test::*,
};
use paint::{
    paint_property_tree::{ClipPaintPropertyNode, PaintPropertyRole, TransformPaintPropertyNode},
    resident_hit_test::{ResidentLayerSelection, ResidentPaintHitTest},
    PaintEngine,
};
use std::{rc::Rc, sync::Arc};
pub type ResidentLayerResult<'r> = HitTestResult<'r, ()>;

/// Live document/frame facts absent from the native Layout/Paint owner. These
/// queries project owners; traversal remains in the translated source modules.
pub trait ResidentLayerServices: ResidentHitTestServices<()> {
    fn QueryError(&self) -> Option<query_services::ResidentQueryError> {
        None
    }
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point;
    fn ClipPathHitTest(&self, cursor: PaintFragmentCursor<'_>, location: &HitTestLocation) -> bool;
    fn TransitionFacts(&self, cursor: PaintFragmentCursor<'_>) -> (bool, bool);
    fn BorderShapePath(
        &self,
        cursor: PaintFragmentCursor<'_>,
        border: PhysicalRect,
    ) -> Option<Box<dyn PathHitTest + '_>>;
}
#[derive(Clone, Copy)]
pub struct ResidentLayer<'a> {
    pub paint: &'a PaintEngine,
    pub root: PaintFragmentCursor<'a>,
    pub cursor: PaintFragmentCursor<'a>,
    pub resident: ResidentPaintHitTest<'a>,
    pub services: &'a dyn ResidentLayerServices,
}
impl<'a> ResidentLayer<'a> {
    pub fn new(
        paint: &'a PaintEngine,
        root: PaintFragmentCursor<'a>,
        cursor: PaintFragmentCursor<'a>,
        services: &'a dyn ResidentLayerServices,
    ) -> Option<Self> {
        Some(Self {
            paint,
            root,
            cursor,
            resident: paint.ResidentLayerHitTest(cursor)?,
            services,
        })
    }
    fn Object(&self) -> &'a LayoutObject {
        unsafe { self.cursor.Cursor().GetLayoutObject().as_ref() }
            .expect("source layer LayoutObject")
    }
    fn Native(&self) -> &'a layoutng_assembly::internal::paint_layer::PaintLayer {
        self.root
            .LayerForPaintObject(
                self.resident
                    .LayoutObjectIdentity()
                    .expect("layer source identity"),
            )
            .expect("source PaintLayer")
    }
    fn ForObject(&self, object: &'a LayoutObject) -> Option<Self> {
        let cursor = self.root.ForLayoutObject(object)?;
        Self::new(self.paint, self.root, cursor, self.services)
    }
    fn ForResident(&self, resident: ResidentPaintHitTest<'a>) -> Option<Self> {
        let cursor = self.root.ForPaintObject(resident.LayoutObjectIdentity()?)?;
        Some(Self {
            cursor,
            resident,
            ..*self
        })
    }
    fn Box(&self) -> Option<&'a PhysicalBoxFragment> {
        let fragment = self.cursor.Cursor().PhysicalFragment()?;
        fragment
            .IsBox()
            .then(|| unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) })
    }
    fn FeaturesValue(&self) -> LayerHitTestFeatures {
        use foundation::RuntimeEnabledFeatures as Features;
        LayerHitTestFeatures {
            unbounded_element: Features::UnboundedElementEnabled(),
            border_radius_for_stacking_context:
                Features::HitTestBorderRadiusForStackingContextEnabled(),
            container_transform_for_preserve_3d:
                Features::HitTestContainerTransformStateForPreserve3dEnabled(),
            stacking_context_is_not_stacked: Features::StackingContextIsNotStackedEnabled(),
            replaced_normal_flow_stacking_inline_paint:
                Features::ReplacedNormalFlowStackingInlinePaintEnabled(),
            canvas_draw_element: self.Object().CanvasDrawElementEnabledForLayout(),
        }
    }
}
impl HitTransform for Arc<TransformPaintPropertyNode> {
    fn MatrixWithOriginApplied(&self) -> gfx::Transform {
        self.as_ref().MatrixWithOriginApplied()
    }
}
impl LayerTransformNode for Arc<TransformPaintPropertyNode> {
    fn NativeOwner(&self) -> &Arc<TransformPaintPropertyNode> {
        self
    }
    fn Identity(&self) -> usize {
        Arc::as_ptr(self) as usize
    }
    fn IsAncestorOf(&self, descendant: &dyn LayerTransformNode) -> bool {
        let mut node = Some(descendant.NativeOwner());
        while let Some(current) = node {
            if Arc::ptr_eq(self, current) {
                return true;
            }
            node = current.parent.as_ref();
        }
        false
    }
}
impl LayerRadiusClip for ClipPaintPropertyNode {
    fn LocalTransformSpace(&self) -> &dyn LayerTransformNode {
        &self.local_transform_space
    }
    fn IntersectsQuad(&self, quad: &gfx::QuadF) -> bool {
        let Some(rect) = self.PaintClipRect() else {
            return true;
        };
        let r = self.PaintClipRadii();
        paint::contoured_rect::FloatRoundedRect::new(
            gfx::RectF::new(
                gfx::PointF::new(rect.x as f32, rect.y as f32),
                gfx::SizeF::new(rect.width as f32, rect.height as f32),
            ),
            paint::contoured_rect::Radii([
                gfx::SizeF::new(r.top_left.x as f32, r.top_left.y as f32),
                gfx::SizeF::new(r.top_right.x as f32, r.top_right.y as f32),
                gfx::SizeF::new(r.bottom_right.x as f32, r.bottom_right.y as f32),
                gfx::SizeF::new(r.bottom_left.x as f32, r.bottom_left.y as f32),
            ]),
        )
        .IntersectsQuad(quad)
    }
}
impl LayerFragmentData for ResidentPaintHitTest<'_> {
    fn NativeOwner(&self) -> ResidentPaintHitTest<'_> {
        *self
    }
    fn Identity(&self) -> usize {
        (*self).Identity()
    }
    fn PaintOffset(&self) -> PhysicalOffset {
        (*self).PaintOffset()
    }
    fn LocalBorderTransform(&self) -> &dyn LayerTransformNode {
        (*self).LocalBorderTransform()
    }
    fn ContentsTransform(&self) -> &dyn LayerTransformNode {
        (*self).ContentsTransform()
    }
    fn PreTransform(&self) -> &dyn LayerTransformNode {
        (*self).PreTransform()
    }
    fn PaintProperties(&self) -> Option<&dyn LayerPaintProperties> {
        self.HasPaintProperties()
            .then_some(self as &dyn LayerPaintProperties)
    }
}
impl LayerPaintProperties for ResidentPaintHitTest<'_> {
    fn HasCSSTransformPropertyNode(&self) -> bool {
        [
            PaintPropertyRole::Translate,
            PaintPropertyRole::Rotate,
            PaintPropertyRole::Scale,
            PaintPropertyRole::Offset,
            PaintPropertyRole::Transform,
        ]
        .into_iter()
        .any(|role| self.Transform(role).is_some())
    }
    fn Perspective(&self) -> Option<&dyn LayerTransformNode> {
        self.Transform(PaintPropertyRole::Perspective)
            .map(|n| n as _)
    }
    fn HasElementCanvasTransform(&self) -> bool {
        self.Transform(PaintPropertyRole::ElementCanvasTransform)
            .is_some()
    }
    fn CSSTransformPropertiesOutsideToInside(
        &self,
    ) -> Box<dyn Iterator<Item = &dyn LayerTransformNode> + '_> {
        Box::new(
            [
                PaintPropertyRole::Translate,
                PaintPropertyRole::Rotate,
                PaintPropertyRole::Scale,
                PaintPropertyRole::Offset,
                PaintPropertyRole::Transform,
            ]
            .into_iter()
            .filter_map(|role| {
                self.Transform(role)
                    .map(|node| node as &dyn LayerTransformNode)
            }),
        )
    }
    fn InnerBorderRadiusClip(&self) -> Option<&dyn LayerRadiusClip> {
        self.Clip(PaintPropertyRole::InnerBorderRadiusClip)
            .map(|clip| clip as _)
    }
}
struct ResidentVisibilityScope;
impl LayerVisibilityScope for ResidentVisibilityScope {}
impl<'r> LayerPhysicalFragment<ResidentLayerResult<'r>> for ResidentLayer<'_> {
    fn MayIntersect(
        &self,
        result: &ResidentLayerResult<'r>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.services
            .MayIntersect(self.cursor, result, location, offset)
    }
    fn NodeAtPoint(
        &self,
        result: &mut ResidentLayerResult<'r>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        NodeAtPointInFragment(
            &ResidentFragment::new(self.cursor, self.services),
            location,
            offset,
            phase,
            result,
        )
    }
}
impl<'r> PaintLayerHitTest<ResidentLayerResult<'r>> for ResidentLayer<'_> {
    fn Identity(&self) -> usize {
        self.resident.Identity()
    }
    fn Facts(&self) -> LayerFacts {
        let layer = self.Native();
        LayerFacts {
            root: layer.IsRootLayer(),
            self_painting: layer.IsSelfPaintingLayer(),
            self_painting_descendant: layer.HasSelfPaintingLayerDescendant(),
            invisible_for_position_visibility: layer.InvisibleForPositionVisibility(),
            ancestor_invisible_for_position_visibility: layer
                .HasAncestorInvisibleForPositionVisibility(),
            preserves_3d: layer.Preserves3D(),
            has_3d_transform: layer.Has3DTransform(),
            has_3d_descendant: layer.Has3DTransformedDescendant(),
            scrollable_area: !layer.GetLayoutObject().GetScrollableArea().is_null(),
            needs_reorder_overlay_controls: self.resident.NeedsReorderOverlayControls(),
        }
    }
    fn LayoutFacts(&self) -> LayerLayoutFacts {
        let object = self.Object();
        let (represented_by_transition_pseudo, has_view_transition_pseudo) =
            self.services.TransitionFacts(self.cursor);
        LayerLayoutFacts {
            layout_identity: object as *const _ as usize,
            can_contain_fixed: object.CanContainFixedPositionObjects(),
            can_contain_absolute: object.CanContainAbsolutePositionObjects(),
            prepaint_clean: true,
            layout_pending: object.NeedsLayout(),
            needs_layout: object.NeedsLayout(),
            child_layout_blocked: object.ChildLayoutBlockedByDisplayLock(),
            fragment_less: object.IsFragmentLessBox(),
            nonzero_effective_opacity: self.resident.HasNonZeroEffectiveOpacity(),
            stacking_context: object.IsStackingContext(),
            represented_by_transition_pseudo,
            unbounded_descendant: object.IsInclusiveDescendantOfUnboundedElement(),
            global_root_scroller: object.IsGlobalRootScroller(),
            svg_foreign_object: object.IsSVGForeignObject(),
            has_clip_path: object.HasClipPath(),
            has_border_shape: object.StyleRef().HasBorderShape(),
            clips_overflow_both_axes: object.ShouldClipOverflowAlongBothAxis(),
            backface_hidden: object.HasHiddenBackface(),
            child_paint_blocked: object.ChildPaintBlockedByDisplayLock(),
            layout_inline: object.IsLayoutInline(),
            can_traverse_fragments: object.CanTraversePhysicalFragments(),
            fragmented: object.IsFragmented(),
            floating: object.IsFloating(),
            replaced_normal_flow_stacking_context: object
                .IsReplacedNormalFlowStackingContext(object.StyleRef()),
            canvas: object.IsCanvas(),
            canvas_layout_subtree: object.InputCanvasLayoutSubtree(),
            view_transition_root: object.IsViewTransitionRoot(),
            has_view_transition_pseudo,
        }
    }
    fn Features(&self) -> LayerHitTestFeatures {
        self.FeaturesValue()
    }
    fn Parent(&self) -> Option<Box<dyn PaintLayerHitTest<ResidentLayerResult<'r>> + '_>> {
        let parent = unsafe { self.Native().Parent().as_ref() }?;
        self.ForObject(parent.GetLayoutObject())
            .map(|layer| Box::new(layer) as _)
    }
    fn NearestAncestorElementLayoutIdentity(&self) -> Option<usize> {
        let mut object = unsafe { self.Object().Parent().as_ref() };
        while let Some(current) = object {
            if unsafe { current.GetNode().as_ref() }.is_some_and(|node| node.IsElementNode()) {
                return Some(current as *const _ as usize);
            }
            object = unsafe { current.Parent().as_ref() };
        }
        None
    }
    fn IsDescendantOfLayout(&self, identity: usize) -> bool {
        let mut object = unsafe { self.Object().Parent().as_ref() };
        while let Some(current) = object {
            if current as *const _ as usize == identity {
                return true;
            }
            object = unsafe { current.Parent().as_ref() };
        }
        false
    }
    fn FirstFragment(&self) -> Rc<dyn LayerFragmentData + '_> {
        Rc::new(self.resident)
    }
    fn FragmentData(&self) -> Box<dyn Iterator<Item = Rc<dyn LayerFragmentData + '_>> + '_> {
        Box::new(
            self.root
                .FragmentsForLayoutObject(self.Object())
                .filter_map(|cursor| {
                    self.paint
                        .ResidentHitTest(cursor)
                        .map(|resident| Rc::new(resident) as Rc<dyn LayerFragmentData>)
                }),
        )
    }
    fn PhysicalFragment(
        &self,
        index: usize,
    ) -> Option<Rc<dyn LayerPhysicalFragment<ResidentLayerResult<'r>> + '_>> {
        let object =
            layoutng_assembly::layout_engine::NativeHitTestCursor::ForObject(self.Object());
        let first = object.FirstPhysicalFragment()?;
        let box_ = unsafe { first.OwnerLayoutBox().as_ref() }?;
        if index >= box_.PhysicalFragmentCount() as usize {
            return None;
        }
        let fragment = unsafe { box_.GetPhysicalFragment(index).as_ref() }?;
        let cursor = self.root.ForPhysicalFragment(fragment);
        Some(Rc::new(Self::new(
            self.paint,
            self.root,
            cursor,
            self.services,
        )?))
    }
    fn HasLayoutBoxWithBlockFragments(&self) -> bool {
        !self.Native().GetLayoutBoxWithBlockFragments().is_null()
    }
    fn CanPaintMultipleFragments(&self) -> bool {
        self.cursor.paint.can_paint_multiple_fragments
    }
    fn CalculateClipRects(
        &self,
        root: &dyn PaintLayerHitTest<ResidentLayerResult<'r>>,
        root_fragment: Rc<dyn LayerFragmentData + '_>,
        fragment: Rc<dyn LayerFragmentData + '_>,
        clip: OverflowClipBehavior,
    ) -> (PhysicalOffset, LayerClipRect, LayerClipRect) {
        let rects = self.paint.CalculateResidentLayerClipRects(self.root, self.resident,
            root_fragment.NativeOwner(), fragment.NativeOwner(), paint::resident_layer_clipper::ResidentLayerClipOptions {
                same_root_layer: self.Identity() == root.Identity(), respect_root_clip: clip == OverflowClipBehavior::Respect,
                unbounded_element: self.FeaturesValue().unbounded_element,
                singular_transform_fix: foundation::RuntimeEnabledFeatures::GeometryMapperSingularTransformFixEnabled(),
                overlay_scrollbar_behavior: foundation::OverlayScrollbarClipBehavior::kExcludeOverlayScrollbarSizeForHitTesting });
        let convert = |rect: paint::resident_layer_clipper::ResidentLayerClipRect| LayerClipRect {
            rect: rect.rect,
            has_radius: rect.has_radius,
            is_infinite: rect.is_infinite,
        };
        (
            rects.layer_offset,
            convert(rects.background),
            convert(rects.foreground),
        )
    }
    fn Projection(
        &self,
        source: &dyn LayerTransformNode,
        destination: &dyn LayerTransformNode,
    ) -> gfx::Transform {
        use paint::geometry_mapper_source::{
            geometry_mapper::GeometryMapper,
            native::{NativeGeometryMapperContext, NativeTransformNode},
        };
        GeometryMapper::<NativeGeometryMapperContext>::SourceToDestinationProjection(
            &NativeTransformNode::from_arc(source.NativeOwner().clone()),
            &NativeTransformNode::from_arc(destination.NativeOwner().clone()),
        )
        .as_source()
        .clone()
    }
    fn EnterAncestorPositionVisibilityScope(&self) -> Box<dyn LayerVisibilityScope + '_> {
        // Native HasAncestorInvisibleForPositionVisibility exports the exact
        // result unconditionally; the source optimization scope owns no data.
        Box::new(ResidentVisibilityScope)
    }
    fn RootFrameToVisualViewport(&self, point: gfx::Point) -> gfx::Point {
        self.services.RootFrameToVisualViewport(point)
    }
    fn HitTestOverflowControl(
        &self,
        result: &mut ResidentLayerResult<'r>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool {
        self.services
            .HitTestOverflowControl(self.cursor, result, location, offset)
    }
    fn LayoutNodeAtPoint(
        &self,
        result: &mut ResidentLayerResult<'r>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        self.services
            .LegacyNodeAtPoint(self.cursor, result, location, offset, phase)
    }
    fn UpdateHitTestResult(&self, result: &mut ResidentLayerResult<'r>, point: PhysicalOffset) {
        if result.InnerNode().is_none() {
            let node = unsafe { self.Object().NodeForHitTest().as_ref() }
                .and_then(|node| self.services.ResolveNativeNode(node));
            if node.is_some() {
                result.SetNodeAndPosition(node, point);
            }
        }
    }
    fn LayoutAncestorNodes(&self) -> Box<dyn Iterator<Item = Option<NodeRef>> + '_> {
        Box::new(
            std::iter::successors(Some(self.Object()), |object| unsafe {
                object.Parent().as_ref()
            })
            .map(|object| {
                unsafe { object.NodeForHitTest().as_ref() }
                    .and_then(|node| self.services.ResolveNativeNode(node))
            }),
        )
    }
    fn EnclosingLinkEventParentOrSelf(&self, node: &NodeRef) -> Option<NodeRef> {
        crate::core::dom::node_hit_test::EnclosingLinkEventParentOrSelf(node)
    }
    fn VisualOverflowRect(&self) -> PhysicalRect {
        self.services
            .VisualOverflowRectIncludingFilters(self.cursor)
    }
    fn ReversePaintOrder(
        &self,
        children: PaintLayerIteration,
    ) -> Box<dyn Iterator<Item = Box<dyn PaintLayerHitTest<ResidentLayerResult<'r>> + '_>> + '_>
    {
        let selection = ResidentLayerSelection {
            negative_z: children.0 & 1 != 0,
            normal_flow: children.0 & 2 != 0,
            positive_z: children.0 & 4 != 0,
        };
        let order = self
            .paint
            .ReverseResidentLayerOrder(self.cursor, selection)
            .expect("prepared canonical layer order");
        Box::new(order.filter_map(|resident| {
            self.ForResident(resident)
                .map(|layer| Box::new(layer) as Box<dyn PaintLayerHitTest<ResidentLayerResult<'r>>>)
        }))
    }
    fn CanvasReversePaintOrder(
        &self,
    ) -> Box<dyn Iterator<Item = Box<dyn PaintLayerHitTest<ResidentLayerResult<'r>> + '_>> + '_>
    {
        self.ReversePaintOrder(PaintLayerIteration::kAllChildren)
    }
    fn OverlayOverflowControlsAfter(
        &self,
        child: &dyn PaintLayerHitTest<ResidentLayerResult<'r>>,
    ) -> Box<
        dyn DoubleEndedIterator<Item = Box<dyn PaintLayerHitTest<ResidentLayerResult<'r>> + '_>>
            + '_,
    > {
        let child_fragment = child.FirstFragment();
        let child_owner = child_fragment.NativeOwner();
        let source = child_owner
            .LayoutObjectIdentity()
            .and_then(|identity| self.root.ForPaintObject(identity));
        let order =
            source.and_then(|source| self.paint.ResidentOverlayControlsAfter(self.cursor, source));
        Box::new(order.into_iter().flatten().filter_map(|resident| {
            self.ForResident(resident)
                .map(|layer| Box::new(layer) as Box<dyn PaintLayerHitTest<ResidentLayerResult<'r>>>)
        }))
    }
    fn LocalOriginInAncestor(
        &self,
        ancestor: &dyn PaintLayerHitTest<ResidentLayerResult<'r>>,
    ) -> PhysicalOffset {
        let root = ancestor.FirstFragment();
        let transform = self.Projection(self.resident.PreTransform(), root.LocalBorderTransform());
        let point = transform.MapPoint(gfx::PointF::new(
            self.resident.PaintOffset().left.ToFloat(),
            self.resident.PaintOffset().top.ToFloat(),
        ));
        PhysicalOffset::FromPointFRound(&point) - root.PaintOffset()
    }
    fn ClipPathHitTest(&self, location: &HitTestLocation) -> bool {
        self.services.ClipPathHitTest(self.cursor, location)
    }
    fn HasLayoutBox(&self) -> bool {
        self.Object().IsBox()
    }
    fn BorderBoxRect(&self) -> PhysicalRect {
        layoutng_assembly::layout_engine::NativeHitTestCursor::ForObject(self.Object())
            .PhysicalBorderBoxRect()
    }
    fn ShouldApplyOverflowClipMargin(&self) -> bool {
        self.Box()
            .is_some_and(|box_| box_.ShouldApplyOverflowClipMargin())
    }
    fn ExpandByBorderOutsetsForClipping(&self, rect: &mut PhysicalRect) {
        if let Some(box_) = self.Box() {
            let outsets = box_.OverflowClipMarginOutsets();
            rect.ExpandEdges(outsets.top, outsets.right, outsets.bottom, outsets.left);
        }
    }
    fn BorderShapeOuterPath(&self, border: PhysicalRect) -> Option<Box<dyn PathHitTest + '_>> {
        self.services.BorderShapePath(self.cursor, border)
    }
    fn ContainerBorderRadiusClips(
        &self,
    ) -> Box<dyn Iterator<Item = (usize, Option<&dyn LayerRadiusClip>)> + '_> {
        Box::new(
            std::iter::successors(
                unsafe { self.Object().Container().as_ref() },
                |object| unsafe { object.Container().as_ref() },
            )
            .map(|object| {
                let clip = self
                    .root
                    .ForLayoutObject(object)
                    .and_then(|cursor| self.paint.ResidentHitTest(cursor))
                    .and_then(|resident| resident.Clip(PaintPropertyRole::InnerBorderRadiusClip))
                    .map(|clip| clip as &dyn LayerRadiusClip);
                (object as *const _ as usize, clip)
            }),
        )
    }
}

/// Canonical contour/path geometry for resident source hit-test ports. The
/// embedding injects its existing backend path factory; no browser algorithm,
/// geometry cache or retained fragment projection is introduced.
pub struct ResidentCursorGeometry<'a> {
    pub paths: &'a dyn paint::path_hit_test::PaintPathHitTestFactory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidentCursorGeometryError {
    MissingLayoutOwner,
    InvalidBorderShape(paint::basic_shape_path::InvalidBasicShape),
}
struct ResidentContour<'a> {
    contour: paint::contoured_rect::ContouredRect,
    paths: &'a dyn paint::path_hit_test::PaintPathHitTestFactory,
}
impl ContouredRectHitTest for ResidentContour<'_> {
    fn IntersectsQuad(&self, quad: &gfx::QuadF) -> bool {
        self.contour.IntersectsQuad(quad, self.paths)
    }
}
struct ResidentPath(Box<dyn paint::path_hit_test::PaintPathHitTest>);
impl PathHitTest for ResidentPath {
    fn CountVerbs(&self) -> i32 {
        self.0.count_verbs()
    }
    fn Intersects(&self, quad: &gfx::QuadF) -> bool {
        self.0.intersects(quad, None)
    }
    fn Contains(&self, point: gfx::PointF) -> bool {
        self.0.contains(point, None)
    }
    fn IntersectsWithWindRule(&self, quad: &gfx::QuadF, rule: foundation::WindRule) -> bool {
        self.0.intersects(quad, Some(PathFill(rule)))
    }
    fn ContainsWithWindRule(&self, point: gfx::PointF, rule: foundation::WindRule) -> bool {
        self.0.contains(point, Some(PathFill(rule)))
    }
}
fn PathFill(rule: foundation::WindRule) -> paint::paint_clip_path::PaintPathFillType {
    match rule {
        foundation::WindRule::RULE_NONZERO => paint::paint_clip_path::PaintPathFillType::Winding,
        foundation::WindRule::RULE_EVENODD => paint::paint_clip_path::PaintPathFillType::EvenOdd,
    }
}
impl ResidentCursorGeometry<'_> {
    // Source: box_fragment_painter.cc:2297-2313,2337-2349,3000-3018;
    // ContouredBorderGeometry:contoured_border_geometry.cc:27-269.
    pub fn ContouredBorder(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
        inner: bool,
        include_fragment_sides: bool,
    ) -> Box<dyn ContouredRectHitTest + '_> {
        let style = match cursor.Cursor() {
            FragmentCursor::Physical { fragment, .. } => fragment.Style(),
            FragmentCursor::Inline { item, .. } => item.Style(),
        };
        let sides = if include_fragment_sides {
            cursor
                .Cursor()
                .PhysicalFragment()
                .filter(|fragment| fragment.IsBox())
                .map(|fragment| {
                    unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) }
                        .SidesToInclude()
                })
                .unwrap_or_default()
        } else {
            Default::default()
        };
        let contour = if inner {
            paint::contoured_rect::border::PixelSnappedContouredInnerBorder(style, rect, sides)
        } else {
            paint::contoured_rect::border::PixelSnappedContouredBorder(style, rect, sides)
        };
        Box::new(ResidentContour {
            contour,
            paths: self.paths,
        })
    }
    // Source: border_shape_utils.cc:13-51; border_shape_painter.cc:42-102.
    // Errors remain explicit: unsupported compiled shapes do not become an
    // empty path or an approximate border hit.
    pub fn BorderShapeOuterPath(
        &self,
        cursor: PaintFragmentCursor<'_>,
        rect: PhysicalRect,
    ) -> Result<Box<dyn PathHitTest>, ResidentCursorGeometryError> {
        let style = match cursor.Cursor() {
            FragmentCursor::Physical { fragment, .. } => fragment.Style(),
            FragmentCursor::Inline { item, .. } => item.Style(),
        };
        let identity = cursor
            .paint
            .LayoutObjectIdentity()
            .ok_or(ResidentCursorGeometryError::MissingLayoutOwner)?;
        let reference = identity.BorderShapeOuterReferenceRect(rect, style);
        let path = self
            .paths
            .create_border_shape(style, gfx::RectF::from(reference))
            .map_err(ResidentCursorGeometryError::InvalidBorderShape)?;
        Ok(Box::new(ResidentPath(path)))
    }
}

pub mod child_frame_query;
/// Owning production point queries; the facade supplies only borrowed inputs.
pub mod point_queries;
pub mod query_services;

/// Native LayoutObject dispatch over call-scoped resident owners.
pub mod native_dispatch;
mod svg_clip;
pub mod svg_dispatch;

pub mod layout_view_query;

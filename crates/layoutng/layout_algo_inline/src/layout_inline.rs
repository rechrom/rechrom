#![allow(non_snake_case)]
#[path = "../../layout_fragment/src/layout_inline_visual_rect.rs"]
mod visual_rect;

#[path = "layout_inline_caret.rs"]
mod layout_inline_caret;

use foundation::graphics_types;
use std::ops::{Deref, DerefMut};

use foundation::{
    gfx, LayoutUnit, PhysicalOffset, PhysicalRect, To, TransformState, Vector, Visitor,
};
use layout_fragment::inline_cursor::InlineCursor;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::outline_type::OutlineType;
use layoutng_style::style::style_difference::StyleDifference;

use super::super::caret_rect::CaretShape;
use super::super::editing::forward::PositionWithAffinity;
use super::super::hit_test_phase::HitTestPhase;
use super::super::layout_block_flow::LayoutBlockFlow;
use super::super::layout_box::LayoutBox;
use super::super::layout_box_model_object::{LayoutBoxModelObject, PaintLayerType};
use super::super::layout_node_metadata::Element;
use super::super::layout_object::{
    BoxQuadType, DraggableRegionValue, HitTestLocation, HitTestResult, IncludeDescendants,
    LayoutObject, LocalRectForBoxQuad, PaintInfo, StyleChangeContext,
};
use super::super::layout_object_child_list::LayoutObjectChildList;
use super::super::layout_text::LayoutText;
use super::super::loader::resource::image_resource_observer::{
    CanDeferInvalidation, WrappedImagePtr,
};
use super::super::map_coordinates_flags::MapCoordinatesFlags;
use super::super::outline_geometry::OutlineOutsetExtent;
use super::super::outline_info::LayoutOutlineInfo;
use super::super::outline_rect_collector::{
    OutlineRectCollector, UnionOutlineRectCollector, VectorOutlineRectCollector,
};
use graphics_types::graphics::visual_rect_flags::VisualRectFlags;

#[path = "../../layout_fragment/src/inline_client_geometry.rs"]
mod layout_fragment_inline_client_geometry;
pub(crate) use layout_fragment_inline_client_geometry::{
    InlineClientGeometry, LayoutInlineClientRectsInContainers,
};

#[path = "../../layout_fragment/src/inline_quad_geometry.rs"]
mod layout_fragment_inline_quad_geometry;
use layout_fragment_inline_quad_geometry::{
    LayoutInlineMarginOutsets, LayoutInlineQuadsForSelfInternal,
    LayoutInlineQuadsInAncestorInternal,
};

// Non-inline methods live mainly in //src/layoutng_inline. Declarations with
// no definition in the supplied tree remain typed but unconnected.
unsafe extern "Rust" {
    fn LayoutInlineNew(element: *mut Element) -> LayoutInline;
    fn LayoutInlineTrace(this: &LayoutInline, visitor: &mut Visitor);
    fn LayoutInlineAddChild(
        this: &mut LayoutInline,
        child: *mut LayoutObject,
        before: *mut LayoutObject,
    );
    fn LayoutInlineLocalBoundingBoxRectForAccessibility(
        this: &LayoutInline,
        include_descendants: IncludeDescendants,
    ) -> gfx::RectF;
    fn LayoutInlinePhysicalLinesBoundingBox(this: &LayoutInline) -> PhysicalRect;
    fn LayoutInlineLinesVisualOverflowBoundingBox(this: &LayoutInline) -> PhysicalRect;
    fn LayoutInlineCollectLineBoxRects(
        this: &LayoutInline,
        collector: &mut dyn FnMut(&PhysicalRect),
    );
    fn LayoutInlineHasInlineFragments(this: &LayoutInline) -> bool;
    fn LayoutInlineClearFirstInlineFragmentItemIndex(this: &mut LayoutInline);
    fn LayoutInlineSetFirstInlineFragmentItemIndex(this: &mut LayoutInline, index: usize);
    fn LayoutInlineUpdateShouldCreateBoxFragment(this: &mut LayoutInline);
    fn LayoutInlineHitTestCulledInline(
        this: &mut LayoutInline,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
        cursor: &InlineCursor,
    ) -> bool;
    fn LayoutInlineDebugRect(this: &LayoutInline) -> PhysicalRect;
    fn LayoutInlineWillBeDestroyed(this: &mut LayoutInline);
    fn LayoutInlineInLayoutNGInlineFormattingContextWillChange(
        this: &mut LayoutInline,
        value: bool,
    );
    fn LayoutInlineStyleDidChange(
        this: &mut LayoutInline,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &StyleChangeContext,
    );
    fn LayoutInlineInvalidateDisplayItemClients(
        this: &LayoutInline,
        reason: foundation::PaintInvalidationReason,
    );
    fn LayoutInlineComputeInitialShouldCreateBoxFragment(this: &LayoutInline) -> bool;
    fn LayoutInlineComputeInitialShouldCreateBoxFragmentWithStyle(
        this: &LayoutInline,
        style: &ComputedStyle,
    ) -> bool;
    fn LayoutInlineCulledInlineVisualOverflowBoundingBox(this: &LayoutInline) -> PhysicalRect;
    fn LayoutInlineAddChildAsBlockInInline(
        this: &mut LayoutInline,
        child: *mut LayoutObject,
        before: *mut LayoutObject,
    );
    fn LayoutInlineCreateAnonymousContainerForBlockChildren(
        this: &LayoutInline,
    ) -> *mut LayoutBlockFlow;
    fn LayoutInlineCreateAnonymousBoxToSplit(
        this: &LayoutInline,
        box_to_split: *const LayoutBox,
    ) -> *mut LayoutBox;
    fn LayoutInlineMarkMayContainAnchor(this: &mut LayoutInline);
    fn LayoutInlinePaint(this: &LayoutInline, info: &PaintInfo);
    fn LayoutInlineNodeAtPoint(
        this: &mut LayoutInline,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn LayoutInlineLayerTypeRequired(this: &LayoutInline) -> PaintLayerType;
    fn LayoutInlineOffsetPoint(this: &LayoutInline, element: *const Element) -> PhysicalOffset;
    fn LayoutInlineBoundingBoxRelativeToFirstFragment(this: &LayoutInline) -> PhysicalRect;
    fn LayoutInlinePositionForPoint(
        this: &LayoutInline,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity;
    fn LayoutInlineDirtyLinesFromChangedChild(this: &mut LayoutInline, child: *mut LayoutObject);
    fn LayoutInlineUpdateHitTestResult(
        this: &LayoutInline,
        result: &mut HitTestResult,
        offset: &PhysicalOffset,
    );
    fn LayoutInlineImageChanged(
        this: &mut LayoutInline,
        image: WrappedImagePtr,
        defer: CanDeferInvalidation,
    );
    fn LayoutInlineAddDraggableRegions(
        this: &mut LayoutInline,
        regions: &mut Vector<DraggableRegionValue>,
    );
}

// LayoutInline construction and its non-inline methods are implemented by
// //src/layoutng_inline. This base-first owner retains the fields and inline
// behavior declared in //src/layoutng.
// cpp: layoutng/internal/layout_inline.h:114-118
// cpp: layoutng/internal/layout_inline.h:337-343
#[repr(C)]
pub struct LayoutInline {
    model_object_: LayoutBoxModelObject,
    children_: LayoutObjectChildList,
    first_fragment_item_index_: usize,
}

// cpp: layoutng/internal/layout_inline.h:352-359
impl foundation::DowncastFrom<LayoutObject> for LayoutInline {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutInline()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutInline, model_object_) == 0);

impl LayoutInline {
    // The constructor and field-mutating bodies belong to //src/layoutng_inline.
    // These accessors keep their storage in the source-owned //src/layoutng type.
    pub fn new_base_for_inline(element: *mut Element) -> Self {
        let result = Self {
            model_object_: LayoutBoxModelObject::new(element.cast()),
            children_: LayoutObjectChildList::default(),
            first_fragment_item_index_: 0,
        };
        result
            .model_object_
            .SetRuntimeClass(super::super::layout_object::LayoutObjectClass::Inline);
        result
    }
    pub fn ChildrenForInline(&self) -> &LayoutObjectChildList {
        &self.children_
    }
    pub fn ModelObjectForInline(&self) -> &LayoutBoxModelObject {
        &self.model_object_
    }
    pub fn SetFirstFragmentItemIndexForInline(&mut self, index: usize) {
        self.first_fragment_item_index_ = index;
    }

    // cpp: layoutng/internal/layout_inline.h:116-118
    pub fn new(element: *mut Element) -> Self {
        unsafe { LayoutInlineNew(element) }
    }
    pub fn Trace(&self, visitor: &mut Visitor) {
        unsafe { LayoutInlineTrace(self, visitor) }
    }

    // cpp: layoutng/internal/layout_inline.h:135-158
    pub fn AddChild(&mut self, child: *mut LayoutObject, before: *mut LayoutObject) {
        unsafe { LayoutInlineAddChild(self, child, before) }
    }
    pub fn AddChildDefault(&mut self, child: *mut LayoutObject) {
        self.AddChild(child, std::ptr::null_mut())
    }
    pub fn MarginOutsets(&self) -> PhysicalBoxStrut {
        LayoutInlineMarginOutsets(self)
    }
    /// cpp: core/layout/layout_inline.cc:430-508. Enumerate source border
    /// rectangles even when Paint culled this inline. The boundary maps each
    /// container occurrence to ancestor coordinates; no paint box is created.
    pub(crate) fn ClientRectsInContainers(&self) -> InlineClientGeometry {
        LayoutInlineClientRectsInContainers(self)
    }

    pub fn LocalBoundingBoxRectF(&self) -> gfx::RectF {
        // Source: core/layout/layout_inline.cc:904-919.
        self.CheckIsNotDestroyed();
        let mut quads = Vector::new();
        self.QuadsForSelfInternalDefault(
            &mut quads,
            std::ptr::null(),
            MapCoordinatesFlags::default(),
            false,
        );
        let Some(first) = quads.first() else {
            return gfx::RectF::default();
        };
        let mut result = first.BoundingBox();
        for quad in &quads[1..] {
            result.Union(quad.BoundingBox());
        }
        result
    }
    pub fn LocalBoundingBoxRectForAccessibility(
        &self,
        include_descendants: IncludeDescendants,
    ) -> gfx::RectF {
        unsafe { LayoutInlineLocalBoundingBoxRectForAccessibility(self, include_descendants) }
    }
    pub fn PhysicalLinesBoundingBox(&self) -> PhysicalRect {
        unsafe { LayoutInlinePhysicalLinesBoundingBox(self) }
    }
    pub fn LinesVisualOverflowBoundingBox(&self) -> PhysicalRect {
        unsafe { LayoutInlineLinesVisualOverflowBoundingBox(self) }
    }
    pub fn VisualOverflowRect(&self) -> PhysicalRect {
        // cpp: core/layout/layout_inline.cc:746-773
        self.CheckIsNotDestroyed();
        let mut overflow_rect = self.LinesVisualOverflowBoundingBox();
        let style = self.StyleRef();
        let outline_info = LayoutOutlineInfo::GetFromStyle(style);
        let outline_outset = LayoutUnit::from_signed(OutlineOutsetExtent(style, &outline_info));
        if outline_outset != LayoutUnit::default() {
            let mut collector = UnionOutlineRectCollector::default();
            if !self.InQuirksModeForLayout() {
                self.AddOutlineRectsForNormalChildren(
                    &mut collector,
                    &PhysicalOffset::default(),
                    style.OutlineRectsShouldIncludeBlockInkOverflow(),
                );
            } else {
                self.AddOutlineRects(
                    &mut collector,
                    std::ptr::null_mut(),
                    &PhysicalOffset::default(),
                    style.OutlineRectsShouldIncludeBlockInkOverflow(),
                );
            }
            let mut outline_rect = collector.Rect();
            if !outline_rect.IsEmpty() {
                outline_rect.Inflate(outline_outset);
                overflow_rect.Unite(&outline_rect);
            }
        }
        overflow_rect
    }
    pub fn HasInlineFragments(&self) -> bool {
        unsafe { LayoutInlineHasInlineFragments(self) }
    }
    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        unsafe { LayoutInlineClearFirstInlineFragmentItemIndex(self) }
    }
    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        unsafe { LayoutInlineSetFirstInlineFragmentItemIndex(self, index) }
    }
    pub fn AddOutlineRects(
        &self,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        // cpp: core/layout/layout_inline.cc:866-872
        self.AddOutlineRectsInternal(
            collector,
            info,
            offset,
            outline_type,
            IncludeDescendants(true),
        )
    }

    // cpp: layoutng/internal/layout_inline.h:120-129
    pub fn FirstChild(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        debug_assert!(std::ptr::eq(&self.children_, self.VirtualChildren()));
        self.children_.FirstChild()
    }

    pub fn LastChild(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        debug_assert!(std::ptr::eq(&self.children_, self.VirtualChildren()));
        self.children_.LastChild()
    }

    // cpp: layoutng/internal/layout_inline.h:138-141
    pub fn GetNode(&self) -> *mut Element {
        self.CheckIsNotDestroyed();
        To::<Element>(self.model_object_.GetNode())
    }

    // cpp: layoutng/internal/layout_inline.h:165-174
    pub fn AlwaysCreateLineBoxes(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.AlwaysCreateLineBoxesForLayoutInline() && !self.IsInLayoutNGInlineFormattingContext()
    }

    pub fn SetAlwaysCreateLineBoxes(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IsInLayoutNGInlineFormattingContext());
        self.SetAlwaysCreateLineBoxesForLayoutInline(value);
    }
    pub fn SetAlwaysCreateLineBoxesDefault(&mut self) {
        self.SetAlwaysCreateLineBoxes(true)
    }

    // cpp: layoutng/internal/layout_inline.h:177-186
    pub fn ShouldCreateBoxFragment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.AlwaysCreateLineBoxesForLayoutInline() && self.IsInLayoutNGInlineFormattingContext()
    }

    pub fn SetShouldCreateBoxFragment(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsInLayoutNGInlineFormattingContext());
        self.SetAlwaysCreateLineBoxesForLayoutInline(value);
    }
    pub fn SetShouldCreateBoxFragmentDefault(&mut self) {
        self.SetShouldCreateBoxFragment(true)
    }

    // cpp: layoutng/internal/layout_inline.h:187-206
    pub fn UpdateShouldCreateBoxFragment(&mut self) {
        unsafe { LayoutInlineUpdateShouldCreateBoxFragment(self) }
    }
    pub fn LocalCaretRect(&self, offset: i32, shape: CaretShape) -> PhysicalRect {
        layout_inline_caret::LayoutInlineLocalCaretRect(self, offset, shape)
    }
    pub fn HitTestCulledInline(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
        cursor: &InlineCursor,
    ) -> bool {
        unsafe { LayoutInlineHitTestCulledInline(self, result, location, offset, cursor) }
    }
    pub fn FirstLineBoxTopLeft(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.FirstLineBoxTopLeftInternal().unwrap_or_default()
    }
    pub fn AbsoluteBoundingBoxRectHandlingEmptyInline(
        &self,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        // Source: core/layout/layout_inline.cc:542-552. The source override
        // accepts flags but uses the default local-to-absolute conversion.
        let _ = mode;
        self.CheckIsNotDestroyed();
        let mut collector = VectorOutlineRectCollector::default();
        self.AddOutlineRects(
            &mut collector,
            std::ptr::null_mut(),
            &PhysicalOffset::default(),
            OutlineType::kIncludeBlockInkOverflow,
        );
        let mut rect =
            layoutng_geometry::geometry::physical_rect::UnionRect(&collector.TakeRects());
        if rect.IsEmpty() {
            rect.offset = self.AnchorPhysicalLocation();
        }
        self.LocalToAbsoluteRect(&rect, MapCoordinatesFlags::default())
    }
    pub fn AbsoluteBoundingBoxRectHandlingEmptyInlineDefault(&self) -> PhysicalRect {
        self.AbsoluteBoundingBoxRectHandlingEmptyInline(MapCoordinatesFlags::empty())
    }
    pub fn DebugRect(&self) -> PhysicalRect {
        unsafe { LayoutInlineDebugRect(self) }
    }

    // cpp: layoutng/internal/layout_inline.h:214-230
    pub fn WillBeDestroyed(&mut self) {
        unsafe { LayoutInlineWillBeDestroyed(self) }
    }
    pub fn InLayoutNGInlineFormattingContextWillChange(&mut self, value: bool) {
        unsafe { LayoutInlineInLayoutNGInlineFormattingContextWillChange(self, value) }
    }
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &StyleChangeContext,
    ) {
        unsafe { LayoutInlineStyleDidChange(self, diff, old_style, new_style, context) }
    }
    pub fn InvalidateDisplayItemClients(&self, reason: foundation::PaintInvalidationReason) {
        unsafe { LayoutInlineInvalidateDisplayItemClients(self, reason) }
    }
    pub fn QuadsInAncestorInternal(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        LayoutInlineQuadsInAncestorInternal(self, quads, ancestor, mode, box_type)
    }

    // cpp: layoutng/internal/layout_inline.h:232-248
    fn QuadsForSelfInternal(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        map_to_ancestor: bool,
        box_type: BoxQuadType,
    ) {
        LayoutInlineQuadsForSelfInternal(self, quads, ancestor, mode, map_to_ancestor, box_type)
    }
    fn QuadsForSelfInternalDefault(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        map_to_ancestor: bool,
    ) {
        self.QuadsForSelfInternal(quads, ancestor, mode, map_to_ancestor, BoxQuadType::kBorder)
    }
    fn AddOutlineRectsInternal(
        &self,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        offset: &PhysicalOffset,
        outline_type: OutlineType,
        include_descendants: IncludeDescendants,
    ) {
        // cpp: core/layout/layout_inline.cc:874-902
        self.CheckIsNotDestroyed();
        self.CollectLineBoxRects(|rect| {
            let mut rect = *rect;
            rect.offset += *offset;
            collector.AddRect(&rect);
        });
        if include_descendants.0 {
            self.AddOutlineRectsForNormalChildren(collector, offset, outline_type);
        }
        if !info.is_null() {
            unsafe { *info = LayoutOutlineInfo::GetFromStyle(self.StyleRef()) };
        }
    }

    // cpp: layoutng/internal/layout_inline.h:208-211
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutInline"
    }

    // cpp: layoutng/internal/layout_inline.h:250-265
    pub fn VirtualChildren(&self) -> &LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        &self.children_
    }

    pub fn VirtualChildrenMut(&mut self) -> &mut LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        &mut self.children_
    }

    fn Children(&self) -> &LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        &self.children_
    }

    fn ChildrenMut(&mut self) -> &mut LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        &mut self.children_
    }

    // cpp: layoutng/internal/layout_inline.h:267-270
    pub fn IsLayoutInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_inline.h:272-335
    fn ComputeInitialShouldCreateBoxFragment(&self) -> bool {
        unsafe { LayoutInlineComputeInitialShouldCreateBoxFragment(self) }
    }
    fn ComputeInitialShouldCreateBoxFragmentWithStyle(&self, style: &ComputedStyle) -> bool {
        unsafe { LayoutInlineComputeInitialShouldCreateBoxFragmentWithStyle(self, style) }
    }
    fn CulledInlineVisualOverflowBoundingBox(&self) -> PhysicalRect {
        unsafe { LayoutInlineCulledInlineVisualOverflowBoundingBox(self) }
    }
    fn CollectLineBoxRects<F: FnMut(&PhysicalRect)>(&self, mut collector: F) {
        unsafe { LayoutInlineCollectLineBoxRects(self, &mut collector) }
    }
    fn AddChildAsBlockInInline(&mut self, child: *mut LayoutObject, before: *mut LayoutObject) {
        unsafe { LayoutInlineAddChildAsBlockInInline(self, child, before) }
    }
    fn CreateAnonymousContainerForBlockChildren(&self) -> *mut LayoutBlockFlow {
        unsafe { LayoutInlineCreateAnonymousContainerForBlockChildren(self) }
    }
    fn CreateAnonymousBoxToSplit(&self, box_to_split: *const LayoutBox) -> *mut LayoutBox {
        unsafe { LayoutInlineCreateAnonymousBoxToSplit(self, box_to_split) }
    }
    fn MarkMayContainAnchor(&mut self) {
        unsafe { LayoutInlineMarkMayContainAnchor(self) }
    }
    fn Paint(&self, info: &PaintInfo) {
        unsafe { LayoutInlinePaint(self, info) }
    }
    fn NodeAtPoint(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        unsafe { LayoutInlineNodeAtPoint(self, result, location, offset, phase) }
    }
    fn LayerTypeRequired(&self) -> PaintLayerType {
        unsafe { LayoutInlineLayerTypeRequired(self) }
    }
    fn OffsetPoint(&self, element: *const Element) -> PhysicalOffset {
        unsafe { LayoutInlineOffsetPoint(self, element) }
    }
    fn BoundingBoxRelativeToFirstFragment(&self) -> PhysicalRect {
        unsafe { LayoutInlineBoundingBoxRelativeToFirstFragment(self) }
    }
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        self.MapVisualRectInternalSource(ancestor, state, flags)
    }
    fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        unsafe { LayoutInlinePositionForPoint(self, point) }
    }
    fn DirtyLinesFromChangedChild(&mut self, child: *mut LayoutObject) {
        unsafe { LayoutInlineDirtyLinesFromChangedChild(self, child) }
    }
    fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutInlineUpdateHitTestResult(self, result, offset) }
    }
    fn ImageChanged(&mut self, image: WrappedImagePtr, defer: CanDeferInvalidation) {
        unsafe { LayoutInlineImageChanged(self, image, defer) }
    }
    fn AddDraggableRegions(&mut self, regions: &mut Vector<DraggableRegionValue>) {
        unsafe { LayoutInlineAddDraggableRegions(self, regions) }
    }
    fn FirstLineBoxTopLeftInternal(&self) -> Option<PhysicalOffset> {
        // Source: core/layout/layout_inline.cc:506-517.
        self.CheckIsNotDestroyed();
        if !self.IsInLayoutNGInlineFormattingContext() {
            return None;
        }
        let mut cursor = InlineCursor::default();
        cursor.MoveToIncludingCulledInline(self);
        cursor
            .IsNotNull()
            .then(|| cursor.CurrentOffsetInFirstContainerFragment())
    }
    fn AnchorPhysicalLocation(&self) -> PhysicalOffset {
        // Source: core/layout/layout_inline.cc:519-540.
        self.CheckIsNotDestroyed();
        if let Some(location) = self.FirstLineBoxTopLeftInternal() {
            return location;
        }
        if let Some(sibling) = unsafe { self.NextSibling().as_ref() } {
            if sibling.IsLayoutInline() {
                return unsafe { &*(sibling as *const LayoutObject as *const LayoutInline) }
                    .AnchorPhysicalLocation();
            }
            if sibling.IsText() {
                return unsafe { &*(sibling as *const LayoutObject as *const LayoutText) }
                    .FirstLineBoxTopLeft();
            }
            if sibling.IsBox() {
                return unsafe { &*(sibling as *const LayoutObject as *const LayoutBox) }
                    .PhysicalLocation();
            }
        }
        if let Some(parent) =
            unsafe { self.Parent().as_ref() }.filter(|parent| parent.IsLayoutInline())
        {
            return unsafe { &*(parent as *const LayoutObject as *const LayoutInline) }
                .AnchorPhysicalLocation();
        }
        PhysicalOffset::default()
    }

    // cpp: layoutng/internal/layout_inline.h:322-332
    pub fn ShouldBeHandledAsInline(&self, _style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    pub fn AnonymousHasStylePropagationOverride(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_inline.h:345-350
    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        if !self.IsInLayoutNGInlineFormattingContext() {
            return 0;
        }
        self.first_fragment_item_index_
    }

    // cpp: layoutng/internal/layout_inline.h:352-357
    pub fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutInline()
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:1119-1128
    pub fn AlwaysCreateLineBoxesForLayoutInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.always_create_line_boxes_for_layout_inline_
    }

    pub fn SetAlwaysCreateLineBoxesForLayoutInline(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.always_create_line_boxes_for_layout_inline_ = value;
    }
}

impl Deref for LayoutInline {
    type Target = LayoutBoxModelObject;

    fn deref(&self) -> &Self::Target {
        &self.model_object_
    }
}

impl DerefMut for LayoutInline {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model_object_
    }
}

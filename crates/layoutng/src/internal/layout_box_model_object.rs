#![allow(non_snake_case, non_camel_case_types)]

use std::ops::{Deref, DerefMut};

use foundation::{gfx, DynamicTo, LayoutUnit, MakeGarbageCollected, Member, PhysicalRect, Visitor};
use layoutng_geometry::geometry::axis::PhysicalAxes;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::outline_type::OutlineType;

use super::caret_rect::CaretShape;
use super::layout_box::LayoutBox;
use super::layout_box_model_tree::LayoutBoxModelObjectBaseForNode;
use super::layout_node_metadata::{ContainerNode, Element};
use super::layout_object::LayoutObject;
use super::layout_scrollable_area::PaintLayerScrollableArea;
use super::outline_rect_collector::OutlineRectCollector;
pub use super::paint_layer::PaintLayer;
use super::sticky_position_scrolling_constraints::{PerAxisData, StickyConstraintsData};
use foundation::PhysicalOffset;

// Declarations without supplied bodies remain typed external contracts.
unsafe extern "Rust" {
    fn LayoutBoxModelObjectComputeStickyPositionConstraints(
        this: &LayoutBoxModelObject,
        layer: &PaintLayer,
        axes: PhysicalAxes,
    ) -> StickyConstraintsData;
    fn LayoutBoxModelObjectVisualOverflowRectIncludingFilters(
        this: &LayoutBoxModelObject,
    ) -> PhysicalRect;
    fn LayoutBoxModelObjectApplyFiltersToRect(
        this: &LayoutBoxModelObject,
        rect: &PhysicalRect,
    ) -> PhysicalRect;
    fn LayoutBoxModelObjectBackgroundTransfersToView(
        this: &LayoutBoxModelObject,
        document_style: *const ComputedStyle,
    ) -> bool;
    fn LayoutBoxModelObjectAddOutlineRectsForNormalChildren(
        this: &LayoutBoxModelObject,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    );
    fn LayoutBoxModelObjectUpdateCanCompositeBackgroundAttachmentFixed(
        this: &mut LayoutBoxModelObject,
        enable: bool,
    );
    fn LayoutBoxModelObjectAdjustedPositionRelativeTo(
        this: &LayoutBoxModelObject,
        offset: &PhysicalOffset,
        element: *const Element,
    ) -> PhysicalOffset;
    fn LayoutBoxModelObjectLocalCaretRectForEmptyElement(
        this: &LayoutBoxModelObject,
        inline_size: LayoutUnit,
        text_indent_offset: LayoutUnit,
        shape: CaretShape,
    ) -> LogicalRect;
    fn LayoutBoxModelObjectAddOutlineRectsForDescendant(
        this: &LayoutBoxModelObject,
        descendant: &LayoutObject,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    );
}

// cpp: layoutng/internal/layout_box_model_object.h:44-51
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintLayerType {
    kNoPaintLayer,
    kNormalPaintLayer,
    kOverflowClipPaintLayer,
    kForcedPaintLayer,
}

// The sole base must remain first: existing layout routines cast between the
// concrete owner and LayoutObject exactly as C++ does.
// cpp: layoutng/internal/layout_box_model_object.h:118-122
// cpp: layoutng/internal/layout_box_model_object.h:355-359
#[repr(C)]
pub struct LayoutBoxModelObject {
    object_: LayoutObject,
    pub(crate) x_sticky_constraints_: Member<PerAxisData>,
    pub(crate) y_sticky_constraints_: Member<PerAxisData>,
    pub(crate) transform_: Option<Box<gfx::Transform>>,
    scrollable_area_: Member<PaintLayerScrollableArea>,
}

// cpp: layoutng/internal/layout_box_model_object.h:368-373
impl foundation::DowncastFrom<LayoutObject> for LayoutBoxModelObject {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsBoxModelObject()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutBoxModelObject, object_) == 0);

// Padding and borders are virtual because LayoutBox and LayoutInline can
// replace the base computations.
// cpp: layoutng/internal/layout_box_model_object.h:192-203
unsafe extern "Rust" {
    fn DispatchLayoutBoxModelObjectPaddingOutsets(
        owner: *const LayoutBoxModelObject,
    ) -> PhysicalBoxStrut;
    fn DispatchLayoutBoxModelObjectBorderOutsets(
        owner: *const LayoutBoxModelObject,
    ) -> PhysicalBoxStrut;
}

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_box_model_tree.cc:33-35
    pub fn new(node: *mut ContainerNode) -> Self {
        let object = Self {
            object_: LayoutBoxModelObjectBaseForNode(node),
            x_sticky_constraints_: Member::default(),
            y_sticky_constraints_: Member::default(),
            transform_: None,
            scrollable_area_: Member::default(),
        };
        object.SetRuntimeClass(super::layout_object::LayoutObjectClass::BoxModelObject);
        object
    }

    // cpp: layoutng/internal/layout_box_model_object.h:171-171
    pub fn TransformForLayout(&self) -> *const gfx::Transform {
        self.transform_
            .as_deref()
            .map_or(std::ptr::null(), |transform| transform as *const _)
    }

    // cpp: layoutng/internal/layout_scrollable_area.cc:306-309
    pub fn GetScrollableArea(&self) -> *mut PaintLayerScrollableArea {
        self.CheckIsNotDestroyed();
        self.scrollable_area_.Get()
    }

    // cpp: layoutng/internal/layout_scrollable_area.cc:311-336
    pub fn UpdateScrollableAreaForLayout(&mut self) {
        self.CheckIsNotDestroyed();
        let requires_scrollable_area = {
            let box_ =
                DynamicTo::<LayoutBox>(self as *mut LayoutBoxModelObject as *mut LayoutObject);
            if box_.is_null() {
                false
            } else {
                let box_ = unsafe { &*box_ };
                box_.IsScrollContainer() || box_.IsOverscrollContainer() || box_.CanResize()
            }
        };
        if requires_scrollable_area == !self.scrollable_area_.Get().is_null() {
            return;
        }
        if self.scrollable_area_.Get().is_null() {
            let box_ =
                DynamicTo::<LayoutBox>(self as *mut LayoutBoxModelObject as *mut LayoutObject);
            assert!(!box_.is_null());
            self.scrollable_area_ = Member::from_ptr(MakeGarbageCollected(
                PaintLayerScrollableArea::new(unsafe { &mut *box_ }),
            ));
        } else {
            unsafe { &mut *self.scrollable_area_.Get() }.Dispose();
            self.scrollable_area_ = Member::default();
        }
    }

    // cpp: layoutng/internal/layout_scrollable_area.cc:338-344
    pub fn DisposeScrollableAreaForLayout(&mut self) {
        self.CheckIsNotDestroyed();
        let area = self.scrollable_area_.Get();
        if !area.is_null() {
            unsafe { &mut *area }.Dispose();
            self.scrollable_area_ = Member::default();
        }
    }

    // cpp: layoutng/internal/layout_scrollable_area.cc:346-352
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.x_sticky_constraints_);
        visitor.Trace(&self.y_sticky_constraints_);
        visitor.Trace(&self.scrollable_area_);
        self.object_.Trace(visitor);
    }

    // These are base virtual bodies. Concrete boxes may override them.
    // cpp: layoutng/internal/layout_box_model_object.h:192-195
    pub fn PaddingOutsetsBase(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        self.ComputedPaddingOutsets()
    }

    pub fn PaddingOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBoxModelObjectPaddingOutsets(self) }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:197-203
    pub fn BorderOutsetsBase(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        PhysicalBoxStrut::new(
            LayoutUnit::from_signed(style.BorderTopWidth()),
            LayoutUnit::from_signed(style.BorderRightWidth()),
            LayoutUnit::from_signed(style.BorderBottomWidth()),
            LayoutUnit::from_signed(style.BorderLeftWidth()),
        )
    }

    pub fn BorderOutsets(&self) -> PhysicalBoxStrut {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBoxModelObjectBorderOutsets(self) }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:205-210
    pub fn BorderPaddingBlockSize(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let border_padding = self.BorderOutsets() + self.PaddingOutsets();
        if self.IsHorizontalWritingMode() {
            border_padding.VerticalSum()
        } else {
            border_padding.HorizontalSum()
        }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:211-216
    pub fn BorderPaddingInlineSize(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let border_padding = self.BorderOutsets() + self.PaddingOutsets();
        if self.IsHorizontalWritingMode() {
            border_padding.HorizontalSum()
        } else {
            border_padding.VerticalSum()
        }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:224-227
    pub fn BackgroundIsKnownToBeOpaqueInRect(&self, _rect: &PhysicalRect) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_box_model_object.h:271-274
    pub fn ShouldBeHandledAsInlineCurrentStyle(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldBeHandledAsInline(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_box_model_object.h:283-286
    pub fn ComputeCanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_box_model_object.h:362-365
    pub fn IsBoxModelObjectBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
}

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_box_model_object.h:124-130
    // cpp: core/layout/layout_box_model_object.cc:360-367
    pub fn DestroyLayer(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.HasLayer() && !self.Layer().is_null());
        self.SetHasLayer(false);
        self.GetMutableForPainting()
            .FirstFragment()
            .SetLayer(std::ptr::null_mut());
        self.SetNeedsPaintPropertyUpdate();
    }

    pub fn ComputeStickyPositionConstraints(
        &self,
        scroll_container_layer: &PaintLayer,
        scroll_axes: PhysicalAxes,
    ) -> StickyConstraintsData {
        unsafe {
            LayoutBoxModelObjectComputeStickyPositionConstraints(
                self,
                scroll_container_layer,
                scroll_axes,
            )
        }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:159-163
    // cpp: core/layout/layout_box_model_object.cc:370-373
    pub fn HasSelfPaintingLayer(&self) -> bool {
        self.CheckIsNotDestroyed();
        let layer = self.Layer();
        !layer.is_null() && unsafe { &*layer }.IsSelfPaintingLayer()
    }

    // cpp: core/layout/layout_box_model_object.h:162-165
    pub fn Layer(&self) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        self.FirstFragment().Layer()
    }

    // Native virtual dispatch for the source LayerTypeRequired overrides.
    // cpp: core/layout/layout_view.h:108-111
    // cpp: core/layout/layout_inline.cc:825-832
    // cpp: core/layout/layout_box.cc:568-587
    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        self.CheckIsNotDestroyed();
        if self.IsLayoutView() {
            return PaintLayerType::kNormalPaintLayer;
        }
        // SVG block/inline descendants paint inside the containing SVG scope.
        // ForeignObject deliberately bypasses LayoutSVGBlock's override.
        // cpp: core/layout/svg/layout_svg_block.h:58-61
        // cpp: core/layout/svg/layout_svg_inline.h:38-41
        if self.IsSVGChild() && !self.IsSVGForeignObject() {
            return PaintLayerType::kNoPaintLayer;
        }
        if self.IsLayoutInline() {
            return if self.IsRelPositioned()
                || self.IsStickyPositioned()
                || self.CreatesGroup()
                || self.StyleRef().ShouldCompositeForCurrentAnimations()
                || self.ShouldApplyPaintContainment()
            {
                PaintLayerType::kNormalPaintLayer
            } else {
                PaintLayerType::kNoPaintLayer
            };
        }
        if self.IsStacked()
            || self.HasHiddenBackface()
            || (foundation::RuntimeEnabledFeatures::StackingContextIsNotStackedEnabled()
                && self.IsReplacedNormalFlowStackingContext(self.StyleRef()))
        {
            return PaintLayerType::kNormalPaintLayer;
        }
        if self.HasNonVisibleOverflow() && !self.IsLayoutReplaced() {
            return PaintLayerType::kOverflowClipPaintLayer;
        }
        // cpp: core/layout/svg/layout_svg_root.cc:544-553
        if self.IsOverscrollContainer() || self.IsSVGRoot() {
            return PaintLayerType::kForcedPaintLayer;
        }
        PaintLayerType::kNoPaintLayer
    }

    // The layer survives style changes whenever the native object continues
    // to require a layer; only a no-layer transition destroys its client.
    // cpp: core/layout/layout_box_model_object.cc:205-223,294-295
    pub(crate) fn UpdateLayerAfterStyleChange(&mut self) {
        let required = self.LayerTypeRequired();
        let layer = self.Layer();
        if required == PaintLayerType::kNoPaintLayer {
            if !layer.is_null() {
                self.DestroyLayer();
            }
        } else if layer.is_null() {
            self.CreateLayerAfterStyleChange();
        } else {
            unsafe { &mut *layer }.UpdateSelfPaintingLayer();
        }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:180-188
    pub fn VisualOverflowRectIncludingFilters(&self) -> PhysicalRect {
        unsafe { LayoutBoxModelObjectVisualOverflowRectIncludingFilters(self) }
    }

    pub fn ApplyFiltersToRect(&self, rect: &PhysicalRect) -> PhysicalRect {
        unsafe { LayoutBoxModelObjectApplyFiltersToRect(self, rect) }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:229-247
    pub fn BackgroundTransfersToView(&self, document_style: *const ComputedStyle) -> bool {
        unsafe { LayoutBoxModelObjectBackgroundTransfersToView(self, document_style) }
    }

    pub fn BackgroundTransfersToViewDefault(&self) -> bool {
        self.BackgroundTransfersToView(std::ptr::null())
    }

    pub fn AddOutlineRectsForNormalChildren(
        &self,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        unsafe {
            LayoutBoxModelObjectAddOutlineRectsForNormalChildren(
                self,
                collector,
                additional_offset,
                outline_type,
            )
        }
    }

    pub fn UpdateCanCompositeBackgroundAttachmentFixed(&mut self, enable: bool) {
        unsafe { LayoutBoxModelObjectUpdateCanCompositeBackgroundAttachmentFixed(self, enable) }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:257-268
    pub fn AdjustedPositionRelativeTo(
        &self,
        offset: &PhysicalOffset,
        element: *const Element,
    ) -> PhysicalOffset {
        unsafe { LayoutBoxModelObjectAdjustedPositionRelativeTo(self, offset, element) }
    }

    pub fn LocalCaretRectForEmptyElement(
        &self,
        inline_size: LayoutUnit,
        text_indent_offset: LayoutUnit,
        shape: CaretShape,
    ) -> LogicalRect {
        unsafe {
            LayoutBoxModelObjectLocalCaretRectForEmptyElement(
                self,
                inline_size,
                text_indent_offset,
                shape,
            )
        }
    }

    pub fn AddOutlineRectsForDescendant(
        &self,
        descendant: &LayoutObject,
        collector: &mut dyn OutlineRectCollector,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        unsafe {
            LayoutBoxModelObjectAddOutlineRectsForDescendant(
                self,
                descendant,
                collector,
                additional_offset,
                outline_type,
            )
        }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:360-360
    // cpp: core/layout/layout_box_model_object.cc:347-357
    fn CreateLayerAfterStyleChange(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.HasLayer() && self.Layer().is_null());
        let owner = self as *mut LayoutBoxModelObject;
        let layer = MakeGarbageCollected(PaintLayer::new(owner));
        let mut painting = self.GetMutableForPainting();
        let first = painting.FirstFragment();
        first.EnsureId();
        first.SetLayer(layer);
        self.SetHasLayer(true);
        self.SetNeedsPaintPropertyUpdate();
    }

    // cpp: layoutng/internal/layout_box_model_object.h:368-373
    pub fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsBoxModelObject()
    }
}

impl Deref for LayoutBoxModelObject {
    type Target = LayoutObject;

    fn deref(&self) -> &Self::Target {
        &self.object_
    }
}

impl DerefMut for LayoutBoxModelObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object_
    }
}

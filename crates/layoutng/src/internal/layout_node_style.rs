#![allow(non_snake_case)]

use foundation::{CSSPropertyID, EOverflow, EPosition, ETransformStyle3D};
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipBothAxis, kOverflowClipX, kOverflowClipY, OverflowClipAxes,
};
use layoutng_style::style::computed_style::ComputedStyle;

use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_object::{LayoutObject, PositionedState};

impl LayoutObject {
    // cpp: layoutng/internal/layout_node_style.cc:36-91
    pub fn ComputeIsFixedContainer(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsViewTransitionRoot() {
            return true;
        }
        let is_document_element = self.IsDocumentElement();
        if !is_document_element && style.HasNonInitialFilter() {
            return true;
        }
        if !is_document_element && style.HasNonInitialBackdropFilter() {
            return true;
        }
        if self.IsCanvasLayoutSubtreeContainer() {
            return true;
        }
        if self.IsLayoutView() || self.IsSVGForeignObject() || self.IsTextControl() {
            return true;
        }
        let parent = self.Parent();
        if self.IsAnonymous()
            && !parent.is_null()
            && unsafe { &*parent }.IsFieldset()
            && unsafe { &*parent }.CanContainFixedPositionObjects()
        {
            return true;
        }
        if (style.HasTransformRelatedProperty()
            || style.TransformStyle3D() == ETransformStyle3D::kPreserve3d)
            && self.IsBox()
        {
            return true;
        }
        if self.IsEligibleForPaintOrLayoutContainment()
            && (self.ShouldApplyPaintContainmentWithStyle(style)
                || self.ShouldApplyLayoutContainmentWithStyle(style)
                || style.HasWillChangeProperty(CSSPropertyID::kContain))
        {
            return true;
        }
        false
    }

    // cpp: layoutng/internal/layout_node_style.cc:93-103
    pub fn ComputeIsAbsoluteContainer(
        &self,
        style: &ComputedStyle,
        is_fixed_container: bool,
    ) -> bool {
        self.CheckIsNotDestroyed();
        is_fixed_container
            || style.GetPosition() != EPosition::kStatic
            || style.HasWillChangeProperty(CSSPropertyID::kPosition)
            || {
                let parent = self.Parent();
                self.IsAnonymous()
                    && !parent.is_null()
                    && unsafe { &*parent }.IsFieldset()
                    && unsafe { &*parent }.CanContainAbsolutePositionObjects()
            }
    }

    // cpp: layoutng/internal/layout_node_style.cc:182-189
    pub fn SetHasBoxDecorationBackground(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        if value == self.bitfields_.has_box_decoration_background_ {
            return;
        }
        self.bitfields_.has_box_decoration_background_ = value;
    }
}

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_node_style.cc:105-120
    pub fn ShouldBeHandledAsInline(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        if style.IsInBlockifyingDisplay() {
            return false;
        }
        if style.IsDisplayInlineType() {
            return true;
        }
        style.IsInInlinifyingDisplay() && !self.IsTablePart()
    }

    // cpp: layoutng/internal/layout_node_style.cc:122-135
    pub fn UpdateFromStyle(&mut self) {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef() as *const ComputedStyle;
        let style = unsafe { &*style };
        self.SetHasBoxDecorationBackground(style.HasBoxDecorationBackground());
        let is_inline = self.ShouldBeHandledAsInline(style);
        self.SetInline(is_inline);
        let positioned_state = self.ToPositionedState(style.GetPosition());
        self.SetPositionState(positioned_state);
        self.SetIsHorizontalWritingMode(style.IsHorizontalWritingMode());
        let is_fixed_container = self.ComputeIsFixedContainer(style);
        self.SetCanContainFixedPositionObjects(is_fixed_container);
        let is_absolute_container = self.ComputeIsAbsoluteContainer(style, is_fixed_container);
        self.SetCanContainAbsolutePositionObjects(is_absolute_container);
    }
}

impl LayoutBox {
    // cpp: layoutng/internal/layout_node_style.cc:137-142
    pub fn ShouldBeHandledAsFloating(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        style.IsFloating()
            && self.ToPositionedState(style.GetPosition())
                != PositionedState::kIsOutOfFlowPositioned
            && !style.IsInsideDisplayIgnoringFloatingChildren()
    }

    // cpp: layoutng/internal/layout_node_style.cc:144-161
    pub fn UpdateFromStyle(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { &mut *(self as *mut LayoutBox).cast::<LayoutBoxModelObject>() }.UpdateFromStyle();
        let style = self.StyleRef() as *const ComputedStyle;
        let style = unsafe { &*style };
        let is_floating = self.ShouldBeHandledAsFloating(style);
        self.SetFloating(is_floating);
        let has_transform_related_property = if self.IsSVGChild() {
            style.HasTransformRelatedPropertyForSVG()
        } else {
            style.HasTransformRelatedProperty()
        };
        self.SetHasTransformRelatedProperty(has_transform_related_property);
        self.SetHasReflection(!style.BoxReflect().is_null());
        let should_clip_overflow = (!self.StyleRef().IsOverflowVisibleAlongBothAxes()
            || self.ShouldApplyPaintContainment())
            && self.RespectsCSSOverflow();
        self.SetHasNonVisibleOverflow(should_clip_overflow);
        self.UpdateScrollableAreaForLayout();
    }

    // cpp: layoutng/internal/layout_node_style.cc:163-180
    pub fn ComputeOverflowClipAxes(&self) -> OverflowClipAxes {
        self.CheckIsNotDestroyed();
        if self.ShouldApplyPaintContainment() {
            return kOverflowClipBothAxis;
        }
        if !self.RespectsCSSOverflow() || !self.HasNonVisibleOverflow() {
            return kNoOverflowClip;
        }
        if self.IsScrollContainer() {
            return kOverflowClipBothAxis;
        }
        (if self.StyleRef().OverflowX() == EOverflow::kVisible {
            kNoOverflowClip
        } else {
            kOverflowClipX
        }) | (if self.StyleRef().OverflowY() == EOverflow::kVisible {
            kNoOverflowClip
        } else {
            kOverflowClipY
        })
    }
}

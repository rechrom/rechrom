//! LayoutObject scroll-container classification and ancestor lookup.

use super::*;

impl LayoutObject {
    pub fn IsContentMovingOverscrollContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsBox() && self.StyleRef().IsContentMovingOverscrollContainer()
    }

    pub fn IsScrollContainerWithScrollMarkerGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        (self.IsScrollContainer() || self.IsDocumentElement())
            && !self.StyleRef().ScrollMarkerGroupNone()
    }

    pub fn IsBackdropForOverscrollAreaParent(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsBackdropForOverscrollAreaParentProvider(self) }
    }

    pub fn IsScrollMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsScrollMarkerProvider(self) }
    }

    pub fn IsScrollMarkerGroupBefore(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsScrollMarkerGroupBeforeProvider(self) }
    }

    pub fn IsScrollContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsLayoutReplaced() {
            return false;
        }
        // SVG containers use LayoutBox storage at the immutable fragment
        // boundary, but correspond to LayoutSVGModelObject in Chromium.
        // A viewport clip must not turn that storage into a CSS scroll owner.
        if self.IsSVGContainer() && !self.IsSVGForeignObject() {
            return false;
        }
        // The extracted LayoutView also carries the document element's style.
        // Chromium's separate LayoutView is always the viewport scroll owner,
        // including when the propagated CSS overflow is visible or hidden.
        self.IsLayoutView() || (self.HasNonVisibleOverflow() && self.StyleRef().IsScrollContainer())
    }

    pub fn IsOverscrollContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        if !self.IsBox() {
            return false;
        }
        if self.IsOverscrollAreaParent() {
            return true;
        }
        self.StyleRef().EffectiveOverscrollContainerType() != EOverscrollContainerType::kNone
    }

    // Source: core/layout/layout_object.cc:1997-2005 and
    // core/layout/layout_object.h:2311-2313.
    #[inline(never)]
    pub fn ContainingScrollContainerAnyAxis(
        &self,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const LayoutBox {
        let layer = self.ContainingScrollContainerLayerAnyAxis(ignore_layout_view_for_fixed_pos);
        if layer.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*layer }.GetLayoutBox()
        }
    }

    pub fn ContainingScrollContainerAnyAxisDefault(&self) -> *const LayoutBox {
        self.ContainingScrollContainerAnyAxis(false)
    }

    // Source: core/layout/layout_object.cc:1953-1973 and
    // core/layout/layout_object.h:2319-2321.
    #[inline(never)]
    pub fn ContainingScrollContainerLayerAnyAxis(
        &self,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer {
        self.CheckIsNotDestroyed();
        let layer = self.EnclosingLayer();
        if layer.is_null() {
            return std::ptr::null();
        }
        let layer_ref = unsafe { &*layer };
        let box_ = layer_ref.GetLayoutBox();
        if !box_.is_null()
            && box_ as *const LayoutObject != self as *const LayoutObject
            && unsafe { &*box_ }.IsScrollContainer()
        {
            return layer;
        }
        let mut is_fixed_to_view = false;
        let scroll_container =
            layer_ref.ContainingScrollContainerLayer(&mut is_fixed_to_view as *mut bool);
        if scroll_container.is_null() || (is_fixed_to_view && ignore_layout_view_for_fixed_pos) {
            std::ptr::null()
        } else {
            scroll_container
        }
    }

    pub fn ContainingScrollContainerLayerAnyAxisDefault(&self) -> *const PaintLayer {
        self.ContainingScrollContainerLayerAnyAxis(false)
    }

    // Source: core/layout/layout_object.cc:1975-1995 and
    // core/layout/layout_object.h:2323-2328.
    #[inline(never)]
    pub fn ContainingScrollContainerLayerForAxis(
        &self,
        axis: PhysicalAxis,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer {
        self.CheckIsNotDestroyed();
        let mut container =
            self.ContainingScrollContainerLayerAnyAxis(ignore_layout_view_for_fixed_pos);
        while !container.is_null() {
            let current = unsafe { &*container };
            let style = current.GetLayoutObject().StyleRef();
            if (axis == PhysicalAxis::kVertical && style.IsOverflowValueScrollableY())
                || (axis == PhysicalAxis::kHorizontal && style.IsOverflowValueScrollableX())
            {
                return container;
            }
            container = current
                .GetLayoutObject()
                .ContainingScrollContainerLayerAnyAxis(ignore_layout_view_for_fixed_pos);
        }
        std::ptr::null()
    }

    pub fn ContainingScrollContainerLayerForAxisDefault(
        &self,
        axis: PhysicalAxis,
    ) -> *const PaintLayer {
        self.ContainingScrollContainerLayerForAxis(axis, false)
    }
}

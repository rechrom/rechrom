//! LayoutObject scroll-container classification and ancestor lookup.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:1537-1540
    pub fn IsContentMovingOverscrollContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsBox() && self.StyleRef().IsContentMovingOverscrollContainer()
    }

    // cpp: layoutng/internal/layout_object.h:1555-1559
    pub fn IsScrollContainerWithScrollMarkerGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        (self.IsScrollContainer() || self.IsDocumentElement())
            && !self.StyleRef().ScrollMarkerGroupNone()
    }

    // cpp: layoutng/internal/layout_object.h:833-833
    pub fn IsBackdropForOverscrollAreaParent(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsBackdropForOverscrollAreaParentProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1655-1655
    pub fn IsScrollMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsScrollMarkerProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1657-1657
    pub fn IsScrollMarkerGroupBefore(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsScrollMarkerGroupBeforeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1542-1553
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

    // cpp: layoutng/internal/layout_object.h:1525-1535
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

    // cpp: layoutng/internal/layout_object.h:2254-2255
    #[inline(never)]
    pub fn ContainingScrollContainerAnyAxis(
        &self,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const LayoutBox {
        unsafe {
            LayoutObjectContainingScrollContainerAnyAxisProvider(
                self,
                ignore_layout_view_for_fixed_pos,
            )
        }
    }

    pub fn ContainingScrollContainerAnyAxisDefault(&self) -> *const LayoutBox {
        self.ContainingScrollContainerAnyAxis(false)
    }

    // cpp: layoutng/internal/layout_object.h:2262-2263
    #[inline(never)]
    pub fn ContainingScrollContainerLayerAnyAxis(
        &self,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer {
        unsafe {
            LayoutObjectContainingScrollContainerLayerAnyAxisProvider(
                self,
                ignore_layout_view_for_fixed_pos,
            )
        }
    }

    pub fn ContainingScrollContainerLayerAnyAxisDefault(&self) -> *const PaintLayer {
        self.ContainingScrollContainerLayerAnyAxis(false)
    }

    // cpp: layoutng/internal/layout_object.h:2268-2270
    #[inline(never)]
    pub fn ContainingScrollContainerLayerForAxis(
        &self,
        axis: PhysicalAxis,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer {
        unsafe {
            LayoutObjectContainingScrollContainerLayerForAxisProvider(
                self,
                axis,
                ignore_layout_view_for_fixed_pos,
            )
        }
    }

    pub fn ContainingScrollContainerLayerForAxisDefault(
        &self,
        axis: PhysicalAxis,
    ) -> *const PaintLayer {
        self.ContainingScrollContainerLayerForAxis(axis, false)
    }
}

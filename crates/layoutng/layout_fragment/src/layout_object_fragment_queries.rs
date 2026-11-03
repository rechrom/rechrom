//! LayoutObject fragment hit testing and coordinate projection.
//!
//! The concrete virtual dispatch remains assembly supplied. Traversal-capable
//! entry points are deliberately out of line so sampling profiles attribute
//! their inclusive cost to layout_fragment instead of LayoutObject core.

use super::*;

impl LayoutObject {
    #[inline(never)]
    pub fn HitTestForOcclusion(&self, rect: &PhysicalRect) -> HitTestResult {
        unsafe { LayoutObjectHitTestForOcclusionProvider(self, rect) }
    }

    #[inline(never)]
    pub fn HitTestAllPhases(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
    ) -> bool {
        unsafe { LayoutObjectHitTestAllPhasesProvider(self, result, location, accumulated_offset) }
    }

    pub fn NodeForHitTest(&self) -> *mut Node {
        unsafe { LayoutObjectNodeForHitTestProvider(self) }
    }

    pub fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutObjectUpdateHitTestResultProvider(self, result, offset) }
    }

    #[inline(never)]
    pub fn NodeAtPoint(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        if self.IsSVGHiddenContainer() {
            return false;
        }
        unsafe {
            LayoutObjectNodeAtPointProvider(self, result, location, accumulated_offset, phase)
        }
    }

    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpacePhysical(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut PhysicalRect,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { LayoutObjectMapToVisualRectPhysicalProvider(self, ancestor, rect, flags) }
    }

    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpaceFloat(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { LayoutObjectMapToVisualRectFloatProvider(self, ancestor, rect, flags) }
    }

    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { DispatchLayoutObjectMapToVisualRectInternal(self, ancestor, state, flags) }
    }

    #[inline(never)]
    pub fn GetPropertyContainer(
        &self,
        skip_info: *mut AncestorSkipInfo,
        state: *mut PropertyTreeStateOrAlias,
        flags: VisualRectFlags,
    ) -> *const LayoutObject {
        unsafe { LayoutObjectGetPropertyContainerProvider(self, skip_info, state, flags) }
    }

    pub fn LocalCaretRect(&self, offset: i32, shape: CaretShape) -> PhysicalRect {
        unsafe { LayoutObjectLocalCaretRectProvider(self, offset, shape) }
    }

    // This is a fixed-cost hit-test admission predicate. Keep it inlineable so
    // object-by-object hit testing does not manufacture a frame per candidate.
    pub fn VisibleToHitTestRequest(&self, request: &HitTestRequest) -> bool {
        unsafe { LayoutObjectVisibleToHitTestRequestProvider(self, request) }
    }

    #[inline(never)]
    pub fn MapLocalToAncestor(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    ) {
        unsafe { LayoutObjectMapLocalToAncestorProvider(self, ancestor, state, mode) }
    }

    #[inline(never)]
    pub fn MapAncestorToLocal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    ) {
        unsafe { LayoutObjectMapAncestorToLocalProvider(self, ancestor, state, mode) }
    }

    // This is a specifically optimized caller-side path, so the dispatch
    // wrapper remains inlineable despite being physically fragment-owned.
    pub(crate) fn MapToVisualRectInAncestorSpaceInternalFastPath(
        &self,
        ancestor_or_null_for_viewport: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
        intersects: &mut bool,
    ) -> bool {
        unsafe {
            LayoutObjectMapToVisualRectInAncestorSpaceInternalFastPathProvider(
                self,
                ancestor_or_null_for_viewport,
                rect,
                flags,
                intersects,
            )
        }
    }
}

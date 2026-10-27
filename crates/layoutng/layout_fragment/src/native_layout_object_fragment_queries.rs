//! LayoutObject fragment hit testing and coordinate projection.
//!
//! The concrete virtual dispatch remains assembly supplied. Traversal-capable
//! entry points are deliberately out of line so sampling profiles attribute
//! their inclusive cost to layout_fragment instead of LayoutObject core.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2559-2559
    #[inline(never)]
    pub fn HitTestForOcclusion(&self, rect: &PhysicalRect) -> HitTestResult {
        unsafe { LayoutObjectHitTestForOcclusionProvider(self, rect) }
    }

    // cpp: layoutng/internal/layout_object.h:2171-2173
    #[inline(never)]
    pub fn HitTestAllPhases(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
    ) -> bool {
        unsafe { LayoutObjectHitTestAllPhasesProvider(self, result, location, accumulated_offset) }
    }

    // cpp: layoutng/internal/layout_object.h:2178-2178
    pub fn NodeForHitTest(&self) -> *mut Node {
        unsafe { LayoutObjectNodeForHitTestProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2179-2179
    pub fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutObjectUpdateHitTestResultProvider(self, result, offset) }
    }

    // cpp: layoutng/internal/layout_object.h:2182-2185
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

    // cpp: layoutng/internal/layout_object.h:2534-2536
    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpacePhysical(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut PhysicalRect,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { LayoutObjectMapToVisualRectPhysicalProvider(self, ancestor, rect, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2538-2540
    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpaceFloat(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { LayoutObjectMapToVisualRectFloatProvider(self, ancestor, rect, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2544-2547
    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { DispatchLayoutObjectMapToVisualRectInternal(self, ancestor, state, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2554-2556
    #[inline(never)]
    pub fn GetPropertyContainer(
        &self,
        skip_info: *mut AncestorSkipInfo,
        state: *mut PropertyTreeStateOrAlias,
        flags: VisualRectFlags,
    ) -> *const LayoutObject {
        unsafe { LayoutObjectGetPropertyContainerProvider(self, skip_info, state, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2623-2624
    pub fn LocalCaretRect(&self, offset: i32, shape: CaretShape) -> PhysicalRect {
        unsafe { LayoutObjectLocalCaretRectProvider(self, offset, shape) }
    }

    // This is a fixed-cost hit-test admission predicate. Keep it inlineable so
    // object-by-object hit testing does not manufacture a frame per candidate.
    // cpp: layoutng/internal/layout_object.h:2651-2651
    pub fn VisibleToHitTestRequest(&self, request: &HitTestRequest) -> bool {
        unsafe { LayoutObjectVisibleToHitTestRequestProvider(self, request) }
    }

    // cpp: layoutng/internal/layout_object.h:2666-2668
    #[inline(never)]
    pub fn MapLocalToAncestor(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    ) {
        unsafe { LayoutObjectMapLocalToAncestorProvider(self, ancestor, state, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2669-2671
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
    // cpp: layoutng/internal/layout_object.h:3414-3418
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

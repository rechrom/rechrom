//! LayoutView fragment-tree queries and coordinate projection.
//!
//! Concrete traversal stays behind assembly providers. The query entry points
//! live here so hit testing and ancestor projection resolve to layout_fragment
//! rather than generic LayoutView self time.

use super::*;

impl LayoutView {
    pub fn HitTestCount(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.hit_test_count_
    }

    pub fn HitTestCacheHits(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.hit_test_cache_hits_
    }

    #[inline(never)]
    pub fn HitTest(&mut self, location: &HitTestLocation, result: &mut HitTestResult) -> bool {
        unsafe { LayoutViewHitTestProvider(self, location, result) }
    }

    #[inline(never)]
    pub fn HitTestNoLifecycleUpdate(
        &mut self,
        location: &HitTestLocation,
        result: &mut HitTestResult,
    ) -> bool {
        unsafe { LayoutViewHitTestNoLifecycleUpdateProvider(self, location, result) }
    }

    #[inline(never)]
    pub fn ClearHitTestCache(&mut self) {
        unsafe { LayoutViewClearHitTestCacheProvider(self) }
    }

    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe {
            LayoutViewMapToVisualRectInAncestorSpaceInternalProvider(self, ancestor, state, flags)
        }
    }

    #[inline(never)]
    pub fn QuadsInAncestorInternal(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        flags: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        unsafe { LayoutViewQuadsInAncestorInternalProvider(self, quads, ancestor, flags, box_type) }
    }

    // Updating the caller-owned result is a small leaf operation and remains
    // inlineable even though its physical owner is the fragment-query package.
    pub fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutViewUpdateHitTestResultProvider(self, result, offset) }
    }

    #[inline(never)]
    pub fn MapAncestorToLocal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    ) {
        unsafe { LayoutViewMapAncestorToLocalProvider(self, ancestor, state, flags) }
    }

    #[inline(never)]
    pub fn MapLocalToAncestor(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    ) {
        unsafe { LayoutViewMapLocalToAncestorProvider(self, ancestor, state, flags) }
    }

    pub fn GetViewTransitionRoot(&self) -> *mut LayoutViewTransitionRoot {
        unsafe { LayoutViewGetViewTransitionRootProvider(self) }
    }
}

//! LayoutObject visual-overflow and paint invalidation dispatch.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2626-2626
    #[inline(never)]
    pub fn DestroyAndCleanupAnonymousWrappers(&mut self, performing_reattach: bool) {
        unsafe { LayoutObjectDestroyAndCleanupAnonymousWrappersProvider(self, performing_reattach) }
    }

    // cpp: layoutng/internal/layout_object.h:2108-2108
    #[inline(never)]
    pub fn InvalidateVisualOverflow(&mut self) {
        unsafe { LayoutObjectInvalidateVisualOverflowProvider(self) }
    }

    // Base classes without line boxes intentionally perform no propagation.
    // cpp: layoutng/internal/layout_object.h:2197-2197
    pub fn DirtyLinesFromChangedChild(&mut self, _child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2834-2834
    #[inline(never)]
    pub fn InvalidatePaint(&self, context: &PaintInvalidatorContext) {
        unsafe { LayoutObjectInvalidatePaintProvider(self, context) }
    }

    // cpp: layoutng/internal/layout_object.h:2843-2843
    #[inline(never)]
    pub fn InvalidateDisplayItemClients(&self, reason: PaintInvalidationReason) {
        unsafe { LayoutObjectInvalidateDisplayItemClientsProvider(self, reason) }
    }
}

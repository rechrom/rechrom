//! LayoutView post-layout publication entry points.
//!
//! The concrete providers remain assembly supplied, while this module owns the
//! observable commit boundary and its sampling frames.

use super::*;

impl LayoutView {
    // cpp: layoutng/internal/layout_view_layout.cc:209-212
    #[inline(never)]
    pub fn UpdateAfterLayout(&mut self) {
        self.CheckIsNotDestroyed();
        self.base_.UpdateAfterLayout();
    }

    // cpp: layoutng/internal/layout_view.h:135-146
    #[inline(never)]
    pub fn CommitPendingSelection(&mut self) {
        unsafe { LayoutViewCommitPendingSelectionProvider(self) }
    }

    // cpp: layoutng/internal/layout_view.h:338-340
    #[inline(never)]
    pub fn CacheScrollDimensions(&mut self) {
        unsafe { LayoutViewCacheScrollDimensionsProvider(self) }
    }
}

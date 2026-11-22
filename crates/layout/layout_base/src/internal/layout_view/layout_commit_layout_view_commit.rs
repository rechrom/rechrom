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
}

//! LayoutObject display-lock admission and context queries.
//!
//! These fixed-cost gates are shared by layout, reuse, prepaint, and paint.
//! They remain inlineable so package ownership does not add a frame to every
//! blocked-state check.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_node_data.cc:51-55
    pub fn ChildLayoutBlockedByDisplayLock(&self) -> bool {
        self.CheckIsNotDestroyed();
        ContentLockBlocksChildLayout(self.GetNode())
    }

    // cpp: layoutng/internal/layout_object.h:3168-3168
    pub fn ChildPrePaintBlockedByDisplayLock(&self) -> bool {
        self.CheckIsNotDestroyed();
        ContentLockBlocksChildPrePaint(self.GetNode())
    }

    // cpp: layoutng/internal/layout_object.h:3170-3170
    pub fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        self.CheckIsNotDestroyed();
        ContentLockBlocksChildPaint(self.GetNode())
    }

    // cpp: third_party/blink/renderer/core/display_lock/display_lock_utilities.cc:781-792
    pub fn HasLockedAncestorPreventingPaint(&self) -> bool {
        self.CheckIsNotDestroyed();
        let mut ancestor = self.Parent();
        while !ancestor.is_null() {
            let object = unsafe { &*ancestor };
            if object.ChildPaintBlockedByDisplayLock() {
                return true;
            }
            ancestor = object.Parent();
        }
        false
    }

    // cpp: layoutng/internal/layout_object.h:3207-3207
    pub fn GetDisplayLockContext(&self) -> *mut DisplayLockContext {
        unsafe { LayoutObjectGetDisplayLockContextProvider(self) }
    }
}

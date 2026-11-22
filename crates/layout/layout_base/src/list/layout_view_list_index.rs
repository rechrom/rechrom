//! LayoutView counters for list and generated-counter objects.
//!
//! These are O(1) state updates, so physical ownership is useful for package
//! boundaries without forcing profiling frames onto every mutation.

use super::*;

impl LayoutView {
    // cpp: layoutng/internal/layout_view.h:214-241
    pub fn AddLayoutCounter(&mut self) {
        self.CheckIsNotDestroyed();
        self.layout_counter_count_ += 1;
    }

    pub fn RemoveLayoutCounter(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.layout_counter_count_ > 0);
        self.layout_counter_count_ -= 1;
    }

    pub fn HasLayoutCounters(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.layout_counter_count_ != 0
    }

    pub fn AddLayoutListItem(&mut self) {
        self.CheckIsNotDestroyed();
        self.layout_list_item_count_ += 1;
    }

    pub fn RemoveLayoutListItem(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.layout_list_item_count_ > 0);
        self.layout_list_item_count_ -= 1;
    }

    pub fn HasLayoutListItems(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.layout_list_item_count_ != 0
    }
}

//! LayoutObject list-marker role queries.
//!
//! These fixed-cost predicates are physically list-owned but remain inlineable
//! because list construction and layout own the surrounding sampled work.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:1661-1661
    pub fn IsListMarkerForSummary(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsListMarkerForSummaryProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1664-1664
    pub fn IsInListMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsInListMarkerProvider(self) }
    }
}

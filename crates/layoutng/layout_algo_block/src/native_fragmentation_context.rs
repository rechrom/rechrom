//! Native ancestor lookup for block fragmentation context.
//!
//! Included by `layout_object_builder` at the reciprocal assembly boundary.
//! Construction may walk the complete containing chain for every new object;
//! keeping that walk here makes multicol setup cost visible as block-algorithm
//! work without adding a trace event per constructed node.

use super::*;

#[inline(never)]
pub(super) fn native_fragmentation_context(mut ancestor: *mut LayoutObject) -> FragmentationType {
    while !ancestor.is_null() {
        let node = unsafe { &*ancestor }.GetNode();
        if !node.is_null() && unsafe { &*node }.InputStyle().column_count > 1 {
            return FragmentationType::kFragmentColumn;
        }
        ancestor = unsafe { &*ancestor }.Parent();
    }
    FragmentationType::kFragmentPage
}

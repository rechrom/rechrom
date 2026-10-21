#![allow(non_snake_case)]

use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_node_metadata::Element;
use foundation::{GCedHeapHashSet, Member};

// cpp: layoutng_out_of_flow/out_of_flow_element_data.cc:41-47
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxDisplayLocksAffectedByAnchorsProvider(
    box_: &LayoutBox,
) -> *const GCedHeapHashSet<Member<Element>> {
    box_.GetLayoutResults()
        .first()
        .map_or(std::ptr::null(), |result| {
            unsafe { &*result.Get() }.DisplayLocksAffectedByAnchors()
        })
}

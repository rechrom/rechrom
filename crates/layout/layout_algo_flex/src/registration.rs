#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, To};
use layout_base::internal::algorithm_entry::LayoutAlgorithmEntryFor;
use layout_base::internal::layout_block::LayoutBlock;
use layout_base::internal::layout_node_metadata::{Element, Node};
use layout_base::internal::layout_object::LayoutObject;
use layout_base::layout_registry::LayoutRegistry;
use layout_style::style::computed_style::ComputedStyle;

use crate::flex_layout_algorithm::FlexLayoutAlgorithm;
use crate::layout_flexible_box::{DispatchFlexStitchedRowGapIndex, LayoutFlexibleBox};

// cpp: layoutng_flex/assembly.cc:10-13
fn CreateFlexObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    let element = To::<Element>(&mut *node);
    MakeGarbageCollected(LayoutFlexibleBox::new(element)).cast()
}

// cpp: layoutng_flex/assembly.cc:14-16
fn CreateAnonymousFlexObject() -> *mut LayoutBlock {
    MakeGarbageCollected(LayoutFlexibleBox::new(std::ptr::null_mut())).cast()
}

// cpp: layoutng_flex/assembly.h:3-3
// cpp: layoutng_flex/assembly.cc:19-23
pub fn Register(registry: &mut LayoutRegistry) {
    registry.objects.flex = Some(CreateFlexObject);
    registry.objects.anonymous_flex = Some(CreateAnonymousFlexObject);
    registry.objects.flex_stitched_row_gap_index = Some(DispatchFlexStitchedRowGapIndex);
    registry.algorithms.flex = LayoutAlgorithmEntryFor::<FlexLayoutAlgorithm>("layout_algo_flex");
}

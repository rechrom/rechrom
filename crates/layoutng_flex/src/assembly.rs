#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, To};
use layoutng_assembly::internal::algorithm_entry::NativeAlgorithmEntry;
use layoutng_assembly::internal::layout_block::LayoutBlock;
use layoutng_assembly::internal::layout_node_metadata::{Element, Node};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::flex_layout_algorithm::FlexLayoutAlgorithm;
use crate::layout_flexible_box::LayoutFlexibleBox;

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
pub fn InstallFlexAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.objects.flex = Some(CreateFlexObject);
    assembly.objects.anonymous_flex = Some(CreateAnonymousFlexObject);
    assembly.algorithms.flex = NativeAlgorithmEntry::<FlexLayoutAlgorithm>();
}

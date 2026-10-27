#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, To};
use layoutng_assembly::internal::algorithm_entry::NativeAlgorithmEntry;
use layoutng_assembly::internal::layout_input_node::LayoutInputNode;
use layoutng_assembly::internal::layout_node_metadata::{Element, Node};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::internal::layout_replaced::{LayoutInputReplaced, LayoutReplaced};
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::natural_sizing_info::PhysicalNaturalSizingInfo;

use crate::replaced_layout_algorithm::ReplacedLayoutAlgorithm;

// cpp: layoutng_replaced/assembly.cc:14-18
fn CreateReplacedObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    MakeGarbageCollected(LayoutInputReplaced::new(To::<Element>(node))).cast()
}

// cpp: layoutng_replaced/assembly.cc:19-23
#[inline(never)]
fn NaturalSizingInfo(node: &LayoutInputNode) -> PhysicalNaturalSizingInfo {
    let _trace = browser_tracing::span("layout", "layout_algo_replaced.NaturalSizingInfo");
    let replaced = To::<LayoutReplaced>(node.GetLayoutBox().cast::<LayoutObject>());
    unsafe { &*replaced }.ComputeNaturalSizingInfo()
}

// cpp: layoutng_replaced/assembly.h:3-3
// cpp: layoutng_replaced/assembly.cc:26-30
pub fn InstallReplacedAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.algorithms.replaced_sizing.performance_owner = Some("layout_algo_replaced");
    assembly.objects.replaced = Some(CreateReplacedObject);
    assembly.algorithms.replaced =
        NativeAlgorithmEntry::<ReplacedLayoutAlgorithm>("layout_algo_replaced");
    assembly.algorithms.replaced_sizing.natural_sizing_info = Some(NaturalSizingInfo);
}

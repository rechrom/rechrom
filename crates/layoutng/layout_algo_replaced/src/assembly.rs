#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, To};
use layoutng_assembly::internal::algorithm_entry::LayoutAlgorithmEntryFor;
use layoutng_assembly::internal::layout_input_node::LayoutInputNode;
use layoutng_assembly::internal::layout_node_metadata::{Element, Node};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::internal::layout_replaced::{LayoutInputReplaced, LayoutReplaced};
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::natural_sizing_info::PhysicalNaturalSizingInfo;

use crate::layout_replaced::LayoutReplacedRespectsCSSOverflow;
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

// Chromium updates this LayoutImage member when the image resource changes.
// The callback keeps the retained-tree update in layout_invalidation while the
// concrete derived representation remains owned by layout_algo_replaced.
fn UpdateImageState(replaced: &mut LayoutReplaced, ratio: f64, resource_error: bool) {
    assert!(replaced.IsLayoutImage());
    let image = unsafe { &mut *(replaced as *mut LayoutReplaced).cast::<LayoutInputReplaced>() };
    image.SetImageDevicePixelRatio(ratio as f32);
    if !resource_error {
        image.UpdateNaturalDimensionsFromElementInput();
    }
}

// cpp: layoutng_replaced/assembly.h:3-3
// cpp: layoutng_replaced/assembly.cc:26-30
pub fn InstallReplacedAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.algorithms.replaced_sizing.performance_owner = Some("layout_algo_replaced");
    assembly.objects.replaced = Some(CreateReplacedObject);
    assembly.objects.replaced_respects_css_overflow = Some(LayoutReplacedRespectsCSSOverflow);
    assembly.objects.replaced_image_state_updater = Some(UpdateImageState);
    assembly.algorithms.replaced =
        LayoutAlgorithmEntryFor::<ReplacedLayoutAlgorithm>("layout_algo_replaced");
    assembly.algorithms.replaced_sizing.natural_sizing_info = Some(NaturalSizingInfo);
}

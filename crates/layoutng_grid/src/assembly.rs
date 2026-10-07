use crate::{grid_layout_algorithm::GridLayoutAlgorithm, layout_grid::LayoutGrid};
use foundation::{MakeGarbageCollected, To};
use layoutng_assembly::{
    internal::{
        algorithm_entry::NativeAlgorithmEntry,
        layout_block::LayoutBlock,
        layout_node_metadata::{Element, Node},
        layout_object::LayoutObject,
    },
    layout_assembly::LayoutAssembly,
};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_grid/assembly.cc:10-12
fn CreateGridObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    MakeGarbageCollected(LayoutGrid::new(To::<Element>(node))).cast()
}
// cpp: layoutng_grid/assembly.cc:13-15
fn CreateAnonymousGridObject() -> *mut LayoutBlock {
    MakeGarbageCollected(LayoutGrid::new(std::ptr::null_mut())).cast()
}
// cpp: layoutng_grid/assembly.h:3-3
// cpp: layoutng_grid/assembly.cc:17-21
pub fn InstallGridAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.objects.grid = Some(CreateGridObject);
    assembly.objects.anonymous_grid = Some(CreateAnonymousGridObject);
    assembly.algorithms.grid = NativeAlgorithmEntry::<GridLayoutAlgorithm>();
}

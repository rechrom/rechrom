use crate::{grid_layout_algorithm::GridLayoutAlgorithm, layout_grid::LayoutGrid};
use foundation::{MakeGarbageCollected, To};
use layout_base::{
    internal::{
        algorithm_entry::LayoutAlgorithmEntryFor,
        layout_block::LayoutBlock,
        layout_box::LayoutBox,
        layout_node_metadata::{Element, Node},
        layout_object::LayoutObject,
        layout_object_factory_set::GridObjectVirtuals,
    },
    layout_registry::LayoutRegistry,
};
use layout_style::style::computed_style::ComputedStyle;

// cpp: layoutng_grid/assembly.cc:10-12
fn CreateGridObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    MakeGarbageCollected(LayoutGrid::new(To::<Element>(node))).cast()
}
// cpp: layoutng_grid/assembly.cc:13-15
fn CreateAnonymousGridObject() -> *mut LayoutBlock {
    MakeGarbageCollected(LayoutGrid::new(std::ptr::null_mut())).cast()
}
// LayoutGrid's overrides of LayoutObject/LayoutBox virtuals. Base dispatches
// them through the factory set each LayoutGrid was created with.
// cpp: layoutng_grid/layout_grid.h:87-96,108-109
fn CreateGridObjectVirtuals() -> GridObjectVirtuals {
    GridObjectVirtuals {
        add_child: |o, child, before| {
            unsafe { &mut *o.cast::<LayoutGrid>() }.AddChild(child, before)
        },
        remove_child: |o, child| unsafe { &mut *o.cast::<LayoutGrid>() }.RemoveChild(child),
        style_did_change: |o, difference, old_style, new_style, context| {
            unsafe { &mut *o.cast::<LayoutGrid>() }
                .StyleDidChange(difference, old_style, new_style, context);
        },
        stitched_row_gap_index: |o, fragment, gap, line| {
            unsafe { &*o.cast::<LayoutGrid>() }.StitchedRowGapIndex(
                fragment,
                gap as u32,
                line.map(|v| v as u32),
            ) as usize
        },
        adjust_anchor_containing_block: |o, data, style, padding, query| {
            unsafe { &*(o as *const LayoutBox).cast::<LayoutGrid>() }
                .AdjustOutOfFlowContainingBlockForAnchor(data, style, padding, query)
        },
    }
}
// cpp: layoutng_grid/assembly.h:3-3
// cpp: layoutng_grid/assembly.cc:17-21
pub fn Register(registry: &mut LayoutRegistry) {
    registry.objects.grid = Some(CreateGridObject);
    registry.objects.anonymous_grid = Some(CreateAnonymousGridObject);
    registry.objects.grid_virtuals = Some(CreateGridObjectVirtuals());
    registry.algorithms.grid = LayoutAlgorithmEntryFor::<GridLayoutAlgorithm>("layout_algo_grid");
}

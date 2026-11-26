#![allow(non_snake_case)]

use layout_base::internal::algorithm_entry::LayoutAlgorithmEntryFor;
use layout_base::layout_registry::LayoutRegistry;

use crate::block_layout_algorithm::BlockLayoutAlgorithm;
use crate::simplified_layout_algorithm::RunSimplifiedLayout;

// cpp: layoutng_block/assembly.h:3-3
// cpp: layoutng_block/assembly.cc:6-9
pub fn Register(registry: &mut LayoutRegistry) {
    registry.algorithms.block =
        LayoutAlgorithmEntryFor::<BlockLayoutAlgorithm>("layout_algo_block");
    // cpp: core/layout/block_node.cc:1491-1501 (BlockNode::RunSimplifiedLayout).
    registry.algorithms.simplified_support.performance_owner = Some("layout_algo_block");
    registry.algorithms.simplified_support.layout = Some(RunSimplifiedLayout);
}

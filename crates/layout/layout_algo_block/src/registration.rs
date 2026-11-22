#![allow(non_snake_case)]

use layout_core::internal::algorithm_entry::LayoutAlgorithmEntryFor;
use layout_core::layout_registry::LayoutRegistry;

use crate::block_layout_algorithm::BlockLayoutAlgorithm;

// cpp: layoutng_block/assembly.h:3-3
// cpp: layoutng_block/assembly.cc:6-9
pub fn Register(registry: &mut LayoutRegistry) {
    registry.algorithms.block =
        LayoutAlgorithmEntryFor::<BlockLayoutAlgorithm>("layout_algo_block");
}

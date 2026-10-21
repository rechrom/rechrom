#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::algorithm_entry::NativeAlgorithm;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::layout_algorithm::{LayoutAlgorithm, LayoutAlgorithmParams};
use layoutng_assembly::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng_assembly::internal::min_max_sizes::MinMaxSizesResult;
use layoutng_assembly::layout_result::LayoutResult;

// cpp: layoutng_replaced/replaced_layout_algorithm.h:18-28
#[repr(C)]
pub struct ReplacedLayoutAlgorithm {
    base: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
}

const _: () = assert!(std::mem::offset_of!(ReplacedLayoutAlgorithm, base) == 0);

impl Deref for ReplacedLayoutAlgorithm {
    type Target = LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for ReplacedLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ReplacedLayoutAlgorithm {
    // cpp: layoutng_replaced/replaced_layout_algorithm.h:21-21
    // cpp: layoutng_replaced/replaced_layout_algorithm.cc:13-17
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        let base = LayoutAlgorithm::from_params(params);
        debug_assert!(params.space.IsNewFormattingContext());
        Self { base }
    }

    // cpp: layoutng_replaced/replaced_layout_algorithm.h:24-24
    // cpp: layoutng_replaced/replaced_layout_algorithm.cc:19-23
    pub fn Layout(&mut self) -> *const LayoutResult {
        debug_assert!(
            self.GetBreakToken().is_null() || unsafe { &*self.GetBreakToken() }.IsBreakBefore()
        );
        self.base.container_builder_.ToBoxFragment()
    }

    // cpp: layoutng_replaced/replaced_layout_algorithm.h:23-23
    // cpp: layoutng_replaced/replaced_layout_algorithm.cc:25-28
    pub fn ComputeMinMaxSizes(&mut self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        unreachable!("ReplacedLayoutAlgorithm::ComputeMinMaxSizes is NOTREACHED in C++")
    }
}

// cpp: layoutng_replaced/replaced_layout_algorithm.h:18-28
impl NativeAlgorithm for ReplacedLayoutAlgorithm {
    fn new(params: &LayoutAlgorithmParams) -> Self {
        ReplacedLayoutAlgorithm::new(params)
    }

    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }

    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes(input)
    }
}

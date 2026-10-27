#![allow(non_snake_case)]

use foundation::{RuntimeEnabledFeatures, To};
use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::fragmentation_utils::{
    FinishFragmentationForFragmentainer, PageNameForChildFragment,
};
use layoutng_assembly::internal::layout_algorithm::{LayoutAlgorithm, LayoutAlgorithmParams};
use layoutng_assembly::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng_assembly::internal::min_max_sizes::MinMaxSizesResult;
use layoutng_assembly::layout_result::LayoutResult;
use layoutng_assembly::physical_box_fragment::PhysicalBoxFragment;

// cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.h:23-51
pub struct SimplifiedOofLayoutAlgorithm {
    pub base_: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
}

#[allow(non_snake_case)]
impl SimplifiedOofLayoutAlgorithm {
    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.h:28-30
    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.cc:14-28
    pub fn new(params: &LayoutAlgorithmParams, last_fragmentainer: &PhysicalBoxFragment) -> Self {
        let mut base_ = LayoutAlgorithm::from_params(params);
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        debug_assert!(last_fragmentainer.IsFragmentainerBox());
        debug_assert!(params.space.HasKnownFragmentainerBlockSize());
        base_
            .container_builder_
            .SetBoxType(last_fragmentainer.GetBoxType());
        let page_name = PageNameForChildFragment(&base_.container_builder_, last_fragmentainer);
        base_.container_builder_.SetPageNameIfNeeded(page_name);
        base_
            .container_builder_
            .SetFragmentBlockSize(params.space.FragmentainerBlockSize());
        base_.container_builder_.SetHasOutOfFlowFragmentChild(true);
        Self { base_ }
    }

    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.h:39-39
    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.cc:30-54
    pub fn ResumeColumnLayout(&mut self, old_fragment_break_token: *const BlockBreakToken) {
        if old_fragment_break_token.is_null()
            || !unsafe { &*old_fragment_break_token }.IsCausedByColumnSpanner()
        {
            return;
        }
        for child_break_token in unsafe { &*old_fragment_break_token }.ChildBreakTokens() {
            let token = To::<BlockBreakToken>(child_break_token.Get());
            if !unsafe { &*token }.InputNode().IsOutOfFlowPositioned() {
                self.base_
                    .container_builder_
                    .AddBreakToken(child_break_token.Get(), false);
            }
        }
        self.base_.container_builder_.SetHasColumnSpanner();
    }

    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.h:42-47
    pub fn SetHasSubsequentChildren(&mut self) {
        self.base_.container_builder_.SetHasSubsequentChildren();
    }

    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.h:32-32
    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.cc:56-59
    pub fn Layout(&mut self) -> *const LayoutResult {
        FinishFragmentationForFragmentainer(&mut self.base_.container_builder_);
        self.base_.container_builder_.ToBoxFragment()
    }

    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.h:33-35
    pub fn ComputeMinMaxSizes(&self, _: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        unreachable!("SimplifiedOofLayoutAlgorithm::ComputeMinMaxSizes is NOTREACHED")
    }

    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.h:49-49
    // cpp: layoutng_out_of_flow/simplified_oof_layout_algorithm.cc:61-64
    pub fn AppendOutOfFlowResult(&mut self, result: &LayoutResult) {
        self.base_
            .container_builder_
            .AddResultAtOffset(result, result.OutOfFlowPositionedOffset());
    }
}

#![allow(non_snake_case)]

use foundation::{LayoutUnit, RuntimeEnabledFeatures, TextWrapStyle, To};
use layoutng::internal::algorithm_forward::InlineChildLayoutContext;
use layoutng::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_algorithm::LayoutAlgorithmParams;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::layout_assembly::LayoutAssembly;
use layoutng_fragment_tree::fragment_items::FragmentItems;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::layout_result::{EStatus, LayoutResult};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_inline::inline_child_layout_context::{
    OptimalInlineChildLayoutContext, SimpleInlineChildLayoutContext,
};
use layoutng_inline::score_line_break_context::{kMaxLinesForBalance, kMaxLinesForOptimal};
use layoutng_inline::text_fit_utils::{MeasurePerBlockScale, ParagraphScale};
use layoutng_style::css::white_space::ShouldWrapLineGreedy;
use layoutng_style::style::text_fit::{TextFitTarget, TextFitType};

use crate::block_layout_algorithm::{BlockLayoutAlgorithm, PreviousInflowPosition};

impl BlockLayoutAlgorithm {
    // cpp: layoutng_block/block_layout_algorithm.h:182
    // cpp: layoutng_block/block_inline_layout.cc:23-105
    pub(crate) fn LayoutInlineChild(&mut self, node: &InlineNode) -> *const LayoutResult {
        // The standalone boundary enters this block algorithm without Blink's
        // earlier intrinsic-size preparation. FragmentItemsBuilder reads the
        // prepared InlineItemsData while constructing the child context.
        node.PrepareLayoutIfNeeded();
        let mut paragraph_scale = ParagraphScale::default();
        if !self.is_measuring_text_fit_ && RuntimeEnabledFeatures::CssTextFitEnabled() {
            let text_fit = self.Style().GetTextFit();
            let grow_consistent = text_fit.Type() == TextFitType::kGrow
                && text_fit.Target() == TextFitTarget::kConsistent;
            let shrink_consistent = text_fit.Type() == TextFitType::kShrink
                && text_fit.Target() == TextFitTarget::kConsistent;
            if grow_consistent || shrink_consistent {
                let _no_side_effects = DisableLayoutSideEffectsScope::new();
                let space = self.GetConstraintSpace();
                let space_without_fragmentation = if space.HasBlockFragmentation() {
                    space.CloneWithoutFragmentation()
                } else {
                    space.clone()
                };
                let mut cloned_param = LayoutAlgorithmParams::new(
                    self.Node().clone(),
                    self.container_builder_.InitialFragmentGeometry(),
                    &space_without_fragmentation,
                );
                cloned_param.previous_result = self.previous_result_;
                let mut cloned_algorithm = BlockLayoutAlgorithm::new(&cloned_param);
                cloned_algorithm.base.relayout_mode_ = self.base.relayout_mode_;
                cloned_algorithm.line_clamp_data_ = self.line_clamp_data_.clone();
                cloned_algorithm.override_text_box_trim_end_child_ =
                    self.override_text_box_trim_end_child_.clone();
                cloned_algorithm.override_text_box_trim_end_break_token_ =
                    self.override_text_box_trim_end_break_token_;
                cloned_algorithm.is_relayout_for_margin_end_trim_ =
                    self.is_relayout_for_margin_end_trim_;
                cloned_algorithm.pending_margin_end_trim_child_ =
                    self.pending_margin_end_trim_child_.clone();
                cloned_algorithm.is_measuring_text_fit_ = true;
                let result = cloned_algorithm.LayoutInlineChildWithScale(node, None);
                let result_ref = unsafe { &*result };
                if result_ref.Status() != EStatus::kSuccess {
                    if result_ref.Status() == EStatus::kBfcBlockOffsetResolved {
                        let bfc_offset = result_ref.BfcBlockOffset();
                        debug_assert!(bfc_offset.is_some());
                        self.container_builder_
                            .SetBfcBlockOffset(bfc_offset.unwrap());
                    } else if result_ref.Status() == EStatus::kTextBoxTrimEndDidNotApply {
                        self.last_non_empty_inflow_child_ =
                            cloned_algorithm.last_non_empty_inflow_child_.clone();
                        self.last_non_empty_break_token_ =
                            cloned_algorithm.last_non_empty_break_token_;
                    }
                    return self.container_builder_.Abort(result_ref.Status());
                }
                paragraph_scale = MeasurePerBlockScale(
                    &InlineNode::new(To::<LayoutBlockFlow>(self.Node().GetLayoutBox())),
                    result_ref.GetPhysicalFragment(),
                    self.ChildAvailableSize().inline_size,
                );
                if (paragraph_scale.scale < 1.0 && !shrink_consistent)
                    || (paragraph_scale.scale > 1.0 && !grow_consistent)
                {
                    paragraph_scale = ParagraphScale::default();
                }
            }
        }
        self.LayoutInlineChildWithScale(node, Some(&paragraph_scale))
    }

    // cpp: layoutng_block/block_layout_algorithm.h:188-190
    // cpp: layoutng_block/block_inline_layout.cc:107-130
    #[inline(never)]
    pub(crate) fn LayoutInlineChildWithScale(
        &mut self,
        node: &InlineNode,
        paragraph_scale: Option<&ParagraphScale>,
    ) -> *const LayoutResult {
        let wrap = node.Style().GetTextWrapStyle();
        if wrap == TextWrapStyle::kPretty {
            if !node.IsScoreLineBreakDisabled() {
                return self
                    .LayoutWithOptimalInlineChildLayoutContext::<{ kMaxLinesForOptimal as usize }>(
                        node,
                        paragraph_scale,
                    );
            }
        } else if wrap == TextWrapStyle::kBalance {
            if !node.IsScoreLineBreakDisabled() {
                return self
                    .LayoutWithOptimalInlineChildLayoutContext::<{ kMaxLinesForBalance as usize }>(
                        node,
                        paragraph_scale,
                    );
            }
        } else {
            debug_assert!(ShouldWrapLineGreedy(wrap));
        }
        // The context keeps a pointer into this algorithm while Layout also
        // mutates the builder, mirroring the C++ stack object's lifetime.
        let builder = &mut self.container_builder_ as *mut _;
        let mut context = SimpleInlineChildLayoutContext::new(node, unsafe { &mut *builder });
        context.EnableMeasuringModeIfNecessary(paragraph_scale);
        self.LayoutWithInlineChildContext(&mut context as *mut _ as *mut InlineChildLayoutContext)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:192-195
    // cpp: layoutng_block/block_inline_layout.cc:132-142
    #[inline(never)]
    pub(crate) fn LayoutWithOptimalInlineChildLayoutContext<const CAPACITY: usize>(
        &mut self,
        child: &InlineNode,
        paragraph_scale: Option<&ParagraphScale>,
    ) -> *const LayoutResult {
        let builder = &mut self.container_builder_ as *mut _;
        let mut context =
            OptimalInlineChildLayoutContext::<CAPACITY>::new(child, unsafe { &mut *builder });
        context.EnableMeasuringModeIfNecessary(paragraph_scale);
        self.LayoutWithInlineChildContext(&mut context as *mut _ as *mut InlineChildLayoutContext)
    }

    // cpp: layoutng_block/block_layout_algorithm.h:282-285
    // cpp: layoutng_block/block_inline_layout.cc:142-221
    pub(crate) fn TryReuseFragmentsFromCache(
        &mut self,
        inline_node: InlineNode,
        previous_inflow_position: *mut PreviousInflowPosition,
        inline_break_token_out: *mut *const InlineBreakToken,
    ) -> bool {
        debug_assert!(!self.previous_result_.is_null());
        if !self.Style().ShouldWrapLineGreedy() {
            return false;
        }
        let previous_fragment = unsafe {
            &*To::<PhysicalBoxFragment>((&*self.previous_result_).GetPhysicalFragment() as *const _)
        };
        let previous_items = previous_fragment.Items();
        debug_assert!(!previous_items.is_null());
        let previous_items = unsafe { &*previous_items };
        FragmentItems::DirtyLinesFromNeedsLayout(unsafe { &*inline_node.GetLayoutBlockFlow() });
        let end_item = previous_items.EndOfReusableItems(previous_fragment);
        debug_assert!(!end_item.is_null());
        if end_item.is_null() || end_item == previous_items.front() as *const _ {
            return false;
        }

        let mut max_lines = 0u32;
        if let Some(lines_until_clamp) = self.line_clamp_data_.LinesUntilClamp(false) {
            if lines_until_clamp <= 1 {
                return false;
            }
            max_lines = (lines_until_clamp - 1) as u32;
        }

        let children_before = self.container_builder_.Children().len();
        let items_builder = self.container_builder_.ItemsBuilder();
        let space = self.GetConstraintSpace();
        debug_assert_eq!(
            unsafe { &*items_builder }.GetWritingDirection(),
            space.GetWritingDirection()
        );
        let result = unsafe { &mut *items_builder }.AddPreviousItems(
            previous_fragment,
            previous_items,
            unsafe { &*end_item },
            &mut self.container_builder_,
            max_lines,
        );
        if !result.succeeded {
            debug_assert_eq!(self.container_builder_.Children().len(), children_before);
            debug_assert_eq!(result.used_block_size, LayoutUnit::default());
            debug_assert!(result.inline_break_token.is_null());
            return false;
        }

        debug_assert!(!self.abort_when_bfc_block_offset_updated_);
        let success = self.ResolveBfcBlockOffset(unsafe { &mut *previous_inflow_position });
        debug_assert!(success);
        debug_assert!(self.container_builder_.BfcBlockOffset().is_some());
        debug_assert!(result.line_count > 0);
        if max_lines != 0 {
            debug_assert!(result.line_count <= max_lines);
            debug_assert!(self.line_clamp_data_.data.IsClampByLines());
            self.line_clamp_data_.data.lines_until_clamp -= result.line_count as i32;
        } else if self.line_clamp_data_.data.IsCountLines() {
            self.line_clamp_data_.data.lines_until_clamp += result.line_count as i32;
        }
        let new_children: Vec<_> = self.container_builder_.Children()[children_before..]
            .iter()
            .map(|child| (child.fragment.Get(), child.offset.block_offset))
            .collect();
        for (fragment, block_offset) in new_children {
            let fragment = unsafe { &*fragment };
            debug_assert!(fragment.IsLineBox());
            self.PropagateBaselineFromLineBox(fragment, block_offset);
        }
        unsafe {
            (*previous_inflow_position).logical_block_offset += result.used_block_size;
            *inline_break_token_out = result.inline_break_token;
        }
        true
    }
}

// cpp: layoutng_block/inline_assembly.h:3-5
// cpp: layoutng_block/block_layout_algorithm.h:26
// cpp: layoutng_block/block_layout_algorithm.h:178
// cpp: layoutng_block/block_inline_layout.cc:225-238
pub fn InstallBlockInlineSupport(assembly: &mut LayoutAssembly) {
    assembly.algorithms.inline_support.layout_block_child = Some(|block, child| {
        unsafe { &mut *(block as *mut BlockLayoutAlgorithm) }.LayoutInlineChild(child)
    });
    assembly.algorithms.inline_support.reuse_fragments = Some(|block, child, position, token| {
        unsafe { &mut *(block as *mut BlockLayoutAlgorithm) }.TryReuseFragmentsFromCache(
            child,
            position as *mut PreviousInflowPosition,
            token,
        )
    });
}

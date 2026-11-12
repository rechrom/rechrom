#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{DynamicTo, PhysicalSize};
use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::internal::algorithm_entry::LayoutAlgorithmImplementation;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::layout_algorithm::{LayoutAlgorithm, LayoutAlgorithmParams};
use layoutng_assembly::internal::layout_input::MediaChildKind;
use layoutng_assembly::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_replaced::LayoutMedia;
use layoutng_assembly::internal::length_utils::ShrinkLogicalSize;
use layoutng_assembly::internal::min_max_sizes::MinMaxSizesResult;
use layoutng_assembly::layout_result::LayoutResult;
use layoutng_geometry::geometry::{
    logical_rect::LogicalRect, writing_mode_converter::WritingModeConverter,
};

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
        if self.Node().IsMedia() {
            self.LayoutMediaChildren();
        }
        self.base.container_builder_.ToBoxFragment()
    }

    // Source: core/layout/replaced_layout_algorithm.cc:70-105.
    fn LayoutMediaChildren(&mut self) {
        let converter = WritingModeConverter::from_logical_size(
            self.GetConstraintSpace().GetWritingDirection(),
            *self.base.container_builder_.Size(),
        );
        let logical_rect = LogicalRect::new(
            self.BorderPadding().StartOffset(),
            ShrinkLogicalSize(*self.base.container_builder_.Size(), self.BorderPadding()),
        );
        let rect = converter.ToPhysicalRect(logical_rect);
        let media = unsafe { &*self.Node().GetLayoutBox().cast::<LayoutMedia>() };
        let mut child = self.Node().FirstChild();
        while child.is_non_null() {
            let mut width = rect.size.width;
            let element = DynamicTo::<Element>(child.GetDOMNode());
            assert!(!element.is_null(), "native media child is an element");
            if unsafe { &*element }
                .InputElementData()
                .as_ref()
                .and_then(|data| data.media_child_kind)
                == Some(MediaChildKind::Controls)
            {
                width = media.ComputePanelWidth(&rect);
            }
            let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
                self.GetConstraintSpace().GetWritingMode(),
                child.Style().GetWritingDirection(),
                true,
                true,
                false,
            );
            builder.SetAvailableSize(
                converter.ToLogicalSize(PhysicalSize::new(width, rect.size.height)),
            );
            builder.SetIsFixedInlineSize(true);
            builder.SetIsFixedBlockSize(true);
            let child_node = BlockNode::from(child.clone());
            let result = child_node.Layout(
                &builder.ToConstraintSpace(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            );
            let result = unsafe { &*result };
            let offset =
                converter.ToLogicalOffset(rect.offset, result.GetPhysicalFragment().Size());
            self.base
                .container_builder_
                .AddResultAtOffset(result, offset);
            child = child.NextSibling();
        }
    }

    // cpp: layoutng_replaced/replaced_layout_algorithm.h:23-23
    // cpp: layoutng_replaced/replaced_layout_algorithm.cc:25-28
    pub fn ComputeMinMaxSizes(&mut self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        unreachable!("ReplacedLayoutAlgorithm::ComputeMinMaxSizes is NOTREACHED in C++")
    }
}

// cpp: layoutng_replaced/replaced_layout_algorithm.h:18-28
impl LayoutAlgorithmImplementation for ReplacedLayoutAlgorithm {
    const LAYOUT_TRACE_NAME: &'static str = "layout_algo_replaced.Layout";
    const MEASURE_TRACE_NAME: &'static str = "layout_algo_replaced.Measure";

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

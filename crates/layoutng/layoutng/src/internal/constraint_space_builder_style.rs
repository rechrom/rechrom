#![allow(non_snake_case)]

use layoutng_style::style::computed_style::ComputedStyle;

use foundation::{kIndefiniteSize, LayoutUnit};
use layoutng_geometry::geometry::logical_size::LogicalSize;

use super::constraint_space::{AutoSizeBehavior, ConstraintSpace, LayoutResultCacheSlot};
use super::constraint_space_builder::ConstraintSpaceBuilder;
use super::layout_input_node::LayoutInputNode;
use super::space_utils::SetOrthogonalFallbackInlineSizeIfNeeded;

// cpp: layoutng/internal/constraint_space_builder.h:731-738,761-763
pub struct MinMaxConstraintSpaceBuilder {
    pub(crate) delegate_: ConstraintSpaceBuilder,
}

impl MinMaxConstraintSpaceBuilder {
    // cpp: layoutng/internal/constraint_space_builder_style.cc:7-18
    pub fn new(
        parent_space: &ConstraintSpace,
        parent_style: &ComputedStyle,
        child: &LayoutInputNode,
        is_new_fc: bool,
    ) -> Self {
        let mut delegate_ = ConstraintSpaceBuilder::new(
            parent_space,
            child.Style().GetWritingDirection(),
            is_new_fc,
        );
        SetOrthogonalFallbackInlineSizeIfNeeded(parent_style, child.clone(), &mut delegate_);
        delegate_.SetCacheSlot(LayoutResultCacheSlot::kMeasure);
        if parent_space.IsInColumnBfc() && !child.CreatesNewFormattingContext() {
            delegate_.SetIsInColumnBfc();
        }
        Self { delegate_ }
    }

    // cpp: layoutng/internal/constraint_space_builder.h:740-759
    pub fn SetAvailableBlockSize(&mut self, block_size: LayoutUnit) {
        self.delegate_
            .SetAvailableSize(LogicalSize::new(kIndefiniteSize, block_size));
    }

    pub fn SetPercentageResolutionBlockSize(&mut self, block_size: LayoutUnit) {
        self.delegate_
            .SetPercentageResolutionSize(LogicalSize::new(kIndefiniteSize, block_size));
    }

    pub fn SetReplacedChildPercentageResolutionBlockSize(&mut self, block_size: LayoutUnit) {
        self.delegate_
            .SetReplacedChildPercentageResolutionSize(LogicalSize::new(
                kIndefiniteSize,
                block_size,
            ));
    }

    pub fn SetBlockAutoBehavior(&mut self, behavior: AutoSizeBehavior) {
        self.delegate_.SetBlockAutoBehavior(behavior);
    }

    pub fn ToConstraintSpace(self) -> ConstraintSpace {
        self.delegate_.ToConstraintSpace()
    }
}

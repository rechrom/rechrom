#![allow(non_snake_case)]
use foundation::{kIndefiniteSize, LayoutUnit, To, Vector};
use layoutng_assembly::internal::{
    algorithm_entry::NativeAlgorithm,
    block_node::BlockNode,
    break_appeal::kBreakAppealPerfect,
    constraint_space_builder::ConstraintSpaceBuilder,
    fragmentation_utils::{
        BreakStatus, FinishFragmentation, InvolvedInBlockFragmentationForBuilder,
        IsEarlyBreakTarget, SetupSpaceBuilderForFragmentationFromBuilder,
    },
    layout_algorithm::{LayoutAlgorithm, LayoutAlgorithmParams, RelayoutAlgorithm},
    layout_input_node::MinMaxSizesFloatInput,
    min_max_sizes::MinMaxSizesResult,
};
use layoutng_assembly::{
    block_break_token::BlockBreakToken, box_fragment_builder::BoxFragmentBuilder,
    layout_result::LayoutResult, logical_box_fragment::LogicalBoxFragment,
    physical_box_fragment::PhysicalBoxFragment,
};
use layoutng_block::block_child_iterator::BlockChildIterator;
use layoutng_geometry::geometry::{logical_offset::LogicalOffset, logical_size::LogicalSize};
use std::ops::{Deref, DerefMut};

// cpp: layoutng_table/table_section_layout_algorithm.h:17-32
#[repr(C)]
pub struct TableSectionLayoutAlgorithm {
    base: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
}
impl Deref for TableSectionLayoutAlgorithm {
    type Target = LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for TableSectionLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(TableSectionLayoutAlgorithm, base) == 0);
impl TableSectionLayoutAlgorithm {
    // cpp: layoutng_table/table_section_layout_algorithm.cc:16-18
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        Self {
            base: LayoutAlgorithm::from_params(params),
        }
    }
    // cpp: layoutng_table/table_section_layout_algorithm.h:25-28
    pub fn ComputeMinMaxSizes(&self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        panic!("Table layout doesn't compute min/max sizes on table sections");
    }
    // cpp: layoutng_table/table_section_layout_algorithm.cc:33-172
    pub fn Layout(&mut self) -> *const LayoutResult {
        let table_handle = self.GetConstraintSpace().CloneTableDataHandle();
        let table = &*table_handle;
        let section = table.sections[self.GetConstraintSpace().TableSectionIndex() as usize];
        let start = section.start_row_index;
        let available =
            LogicalSize::new(self.base.container_builder_.InlineSize(), kIndefiniteSize);
        let mut first_baseline = None;
        let mut last_baseline = None;
        let mut offset = LogicalOffset::default();
        let mut intrinsic = LayoutUnit::default();
        let mut first_noncollapsed = true;
        let mut row_offsets = Vector::from([LayoutUnit::default()]);
        let mut actual_start = 0;
        let mut it = BlockChildIterator::new(self.Node().FirstChild(), self.GetBreakToken(), true);
        loop {
            let entry = it.NextChildDefault();
            let row = entry.block_node;
            if !row.is_non_null() {
                break;
            }
            let token = To::<BlockBreakToken>(entry.token);
            let index = start + entry.index.unwrap();
            debug_assert!(index < start + section.row_count);
            let collapsed = table.rows[index as usize].is_collapsed;
            if !self.early_break_.is_null()
                && IsEarlyBreakTarget(
                    unsafe { &*self.early_break_ },
                    &self.base.container_builder_,
                    &row.base,
                )
            {
                self.base.container_builder_.AddBreakBeforeChild(
                    row.base,
                    Some(kBreakAppealPerfect),
                    false,
                    LogicalOffset::default(),
                );
                break;
            }
            if !first_noncollapsed && !collapsed {
                offset.block_offset += table.table_border_spacing.block_size;
            }
            debug_assert_eq!(
                table.table_writing_direction.GetWritingMode(),
                self.GetConstraintSpace().GetWritingMode()
            );
            let mut builder = ConstraintSpaceBuilder::new(
                self.GetConstraintSpace(),
                table.table_writing_direction,
                true,
            );
            builder.SetAvailableSize(available);
            builder.SetPercentageResolutionSize(available);
            builder.SetIsFixedInlineSize(true);
            builder.SetTableRowData(table_handle.clone(), index);
            if self.GetConstraintSpace().HasBlockFragmentation() {
                SetupSpaceBuilderForFragmentationFromBuilder(
                    &self.base.container_builder_,
                    &row.base,
                    offset.block_offset,
                    &mut builder,
                );
            }
            let space = builder.ToConstraintSpace();
            let result = row.Layout(&space, token, std::ptr::null(), std::ptr::null());
            let result = unsafe { &*result };
            if self.GetConstraintSpace().HasBlockFragmentation() {
                let block_offset = self.FragmentainerOffsetForChildren() + offset.block_offset;
                let status = self.base.BreakBeforeChildIfNeeded(
                    row.base.clone(),
                    result,
                    block_offset,
                    !first_noncollapsed,
                );
                if status == BreakStatus::kNeedsEarlierBreak {
                    let early = self.base.container_builder_.GetEarlyBreak() as *const _;
                    return self
                        .base
                        .RelayoutAndBreakEarlierDefault::<Self>(unsafe { &*early });
                }
                if status == BreakStatus::kBrokeBefore {
                    break;
                }
                debug_assert_eq!(status, BreakStatus::kContinue);
            }
            let physical =
                unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const _) };
            let fragment = LogicalBoxFragment::new(table.table_writing_direction, physical);
            debug_assert!(fragment.FirstBaseline().is_some() && fragment.LastBaseline().is_some());
            if first_baseline.is_none() {
                first_baseline = Some(offset.block_offset + physical.FirstBaseline().unwrap());
            }
            last_baseline = Some(offset.block_offset + physical.LastBaseline().unwrap());
            self.base
                .container_builder_
                .AddResultAtOffset(result, offset);
            offset.block_offset += fragment.BlockSize();
            first_noncollapsed &= collapsed;
            if table.has_collapsed_borders
                && (token.is_null() || !unsafe { &*token }.IsAtBlockEnd())
            {
                if row_offsets.len() == 1 {
                    actual_start = index;
                }
                row_offsets.push(offset.block_offset);
            }
            intrinsic = offset.block_offset;
            if self.base.container_builder_.HasInflowChildBreakInside() {
                break;
            }
        }
        if !it.NextChildDefault().block_node.is_non_null() {
            self.base.container_builder_.SetHasSeenAllChildren();
        }
        let block_size = if self.GetConstraintSpace().IsFixedBlockSize() {
            debug_assert_eq!(section.row_count, 0);
            self.GetConstraintSpace().AvailableSize().block_size
        } else {
            offset.block_offset
                + if self.GetBreakToken().is_null() {
                    LayoutUnit::default()
                } else {
                    unsafe { &*self.GetBreakToken() }.ConsumedBlockSize()
                }
        };
        let builder = &mut self.base.container_builder_;
        builder.SetFragmentsTotalBlockSize(block_size);
        builder.SetIntrinsicBlockSize(intrinsic);
        if let Some(baseline) = first_baseline {
            builder.SetFirstBaseline(baseline);
        }
        if let Some(baseline) = last_baseline {
            builder.SetLastBaseline(baseline);
        }
        builder.SetIsTablePart();
        if table.has_collapsed_borders && row_offsets.len() > 1 {
            builder.SetTableSectionCollapsedBordersGeometry(actual_start, row_offsets);
        }
        if InvolvedInBlockFragmentationForBuilder(builder) {
            let status = FinishFragmentation(builder);
            debug_assert_eq!(status, BreakStatus::kContinue);
        }
        builder.HandleOofsAndSpecialDescendants();
        builder.ToBoxFragment()
    }
}
// cpp: layoutng_table/table_section_layout_algorithm.h:17-32
impl NativeAlgorithm for TableSectionLayoutAlgorithm {
    fn new(params: &LayoutAlgorithmParams) -> Self {
        Self::new(params)
    }
    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }
    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes(input)
    }
}
impl RelayoutAlgorithm<BlockNode> for TableSectionLayoutAlgorithm {
    fn base(&self) -> &LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken> {
        &self.base
    }
    fn base_mut(&mut self) -> &mut LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken> {
        &mut self.base
    }
    unsafe fn from_base<'a>(
        base: &'a LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    ) -> &'a Self {
        &*(base as *const _ as *const Self)
    }
}

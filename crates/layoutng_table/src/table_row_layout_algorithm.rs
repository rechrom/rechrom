#![allow(non_snake_case)]
use crate::{
    table_layout_utils::{RowBaselineTabulator, SetupTableCellConstraintSpaceBuilder},
    table_row_break_token_data::TableRowBreakTokenData,
};
use foundation::{
    kIndefiniteSize, EBreakBetween, HeapVector, LayoutUnit, MakeGarbageCollected, Member, To,
    Traceable, Visitor,
};
use layoutng_assembly::internal::{
    algorithm_entry::NativeAlgorithm,
    block_node::BlockNode,
    constraint_space::{ConstraintSpace, LayoutResultCacheSlot},
    constraint_space_builder::ConstraintSpaceBuilder,
    fragmentation_utils::{
        BreakStatus, FinishFragmentation, InvolvedInBlockFragmentationForBuilder, IsBreakInside,
        JoinFragmentainerBreakValues, SetupSpaceBuilderForFragmentationFromBuilder,
    },
    layout_algorithm::{LayoutAlgorithm, LayoutAlgorithmParams, RelayoutAlgorithm},
    layout_alignment_utils::ComputeContentAlignmentForTableCell,
    layout_input_node::MinMaxSizesFloatInput,
    min_max_sizes::MinMaxSizesResult,
    table_constraint_space_data::{Cell, Row, TableConstraintSpaceData},
};
use layoutng_assembly::{
    block_break_token::BlockBreakToken,
    box_fragment_builder::BoxFragmentBuilder,
    layout_result::{EStatus, LayoutResult},
    logical_box_fragment::LogicalBoxFragment,
    physical_box_fragment::PhysicalBoxFragment,
};
use layoutng_block::block_child_iterator::BlockChildIterator;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use std::ops::{Deref, DerefMut};

// cpp: layoutng_table/table_row_layout_algorithm.cc:20-33
struct ResultWithOffset {
    result: Member<LayoutResult>,
    offset: LogicalOffset,
}
impl Traceable for ResultWithOffset {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.result);
    }
}

struct RowPlacement {
    max_cell_block_size: LayoutUnit,
    break_before: EBreakBetween,
    break_after: EBreakBetween,
    baselines: RowBaselineTabulator,
    results: HeapVector<ResultWithOffset>,
    has_inflow_break_inside: bool,
}

// cpp: layoutng_table/table_row_layout_algorithm.h:18-33
#[repr(C)]
pub struct TableRowLayoutAlgorithm {
    base: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
}
impl Deref for TableRowLayoutAlgorithm {
    type Target = LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for TableRowLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(TableRowLayoutAlgorithm, base) == 0);
impl TableRowLayoutAlgorithm {
    // cpp: layoutng_table/table_row_layout_algorithm.cc:35-37
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        Self {
            base: LayoutAlgorithm::from_params(params),
        }
    }
    // cpp: layoutng_table/table_row_layout_algorithm.h:26-29
    pub fn ComputeMinMaxSizes(&self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        panic!("Table layout doesn't compute min/max sizes on table rows");
    }

    // cpp: layoutng_table/table_row_layout_algorithm.cc:44-93
    fn CreateCellConstraintSpace(
        &self,
        table: &TableConstraintSpaceData,
        cell: &BlockNode,
        token: *const BlockBreakToken,
        data: &Cell,
        row_size: LayoutUnit,
        baseline: Option<LayoutUnit>,
        encompass: bool,
    ) -> ConstraintSpace {
        let has_span = data.rowspan_block_size != kIndefiniteSize;
        let mut size = if has_span {
            data.rowspan_block_size
        } else {
            row_size
        };
        if IsBreakInside(token) && IsBreakInside(self.GetBreakToken()) && !has_span {
            size -= unsafe { &*self.GetBreakToken() }.ConsumedBlockSize()
                - unsafe { &*token }.ConsumedBlockSize();
        }
        debug_assert_eq!(
            table.table_writing_direction.GetWritingMode(),
            self.GetConstraintSpace().GetWritingMode()
        );
        let mut builder = ConstraintSpaceBuilder::new(
            self.GetConstraintSpace(),
            cell.Style().GetWritingDirection(),
            true,
        );
        SetupTableCellConstraintSpaceBuilder(
            table.table_writing_direction,
            cell,
            &data.borders,
            &table.column_locations,
            size,
            self.base.container_builder_.InlineSize(),
            baseline,
            data.start_column,
            data.is_initial_block_size_indefinite,
            table.is_table_block_size_specified,
            table.has_collapsed_borders,
            LayoutResultCacheSlot::kLayout,
            &mut builder,
        );
        if self.GetConstraintSpace().HasBlockFragmentation() {
            SetupSpaceBuilderForFragmentationFromBuilder(
                &self.base.container_builder_,
                &cell.base,
                LayoutUnit::default(),
                &mut builder,
            );
            if encompass {
                builder.SetMinBlockSizeShouldEncompassIntrinsicSize();
            }
        }
        builder.ToConstraintSpace()
    }
    // cpp: layoutng_table/table_row_layout_algorithm.cc:100-124
    fn MinBlockSizeShouldEncompassIntrinsicSize(
        &self,
        cell: &BlockNode,
        data: &Cell,
        row: &Row,
    ) -> bool {
        if !self.GetConstraintSpace().HasBlockFragmentation()
            || cell.IsMonolithic()
            || data.has_descendant_that_depends_on_percentage_block_size
        {
            return false;
        }
        if data.rowspan_block_size != kIndefiniteSize && data.rowspan_block_size != row.block_size {
            return false;
        }
        true
    }
    // cpp: layoutng_table/table_row_layout_algorithm.cc:126-209
    fn PlaceCells(
        &mut self,
        table: &TableConstraintSpaceData,
        row: &Row,
        row_size: LayoutUnit,
        baseline: Option<LayoutUnit>,
    ) -> RowPlacement {
        let mut state = RowPlacement {
            max_cell_block_size: LayoutUnit::default(),
            break_before: EBreakBetween::kAuto,
            break_after: EBreakBetween::kAuto,
            baselines: RowBaselineTabulator::default(),
            results: HeapVector::new(),
            has_inflow_break_inside: false,
        };
        let fragmentation = self.GetConstraintSpace().HasBlockFragmentation();
        let propagate = self.GetConstraintSpace().ShouldPropagateChildBreakValues();
        let mut it = BlockChildIterator::new(self.Node().FirstChild(), self.GetBreakToken(), true);
        loop {
            let entry = it.NextChildDefault();
            let cell = entry.block_node;
            if !cell.is_non_null() {
                break;
            }
            let token = To::<BlockBreakToken>(entry.token);
            let style = cell.Style();
            let data = &table.cells[(row.start_cell_index + entry.index.unwrap()) as usize];
            let encompass = self.MinBlockSizeShouldEncompassIntrinsicSize(&cell, data, row);
            let space = self.CreateCellConstraintSpace(
                table, &cell, token, data, row_size, baseline, encompass,
            );
            let result = cell.Layout(&space, token, std::ptr::null(), std::ptr::null());
            let result_ref = unsafe { &*result };
            debug_assert_eq!(result_ref.Status(), EStatus::kSuccess);
            let offset = LogicalOffset::new(
                table.column_locations[data.start_column as usize].offset
                    - table.table_border_spacing.inline_size,
                LayoutUnit::default(),
            );
            if fragmentation || baseline.is_none() {
                state.results.push(ResultWithOffset {
                    result: Member::from_ptr(result.cast_mut()),
                    offset,
                });
            } else {
                self.base
                    .container_builder_
                    .AddResultAtOffset(result_ref, offset);
            }
            if propagate {
                let before = JoinFragmentainerBreakValues(
                    style.BreakBefore(),
                    result_ref.InitialBreakBefore(),
                );
                let after =
                    JoinFragmentainerBreakValues(style.BreakAfter(), result_ref.FinalBreakAfter());
                state.break_before = JoinFragmentainerBreakValues(state.break_before, before);
                state.break_after = JoinFragmentainerBreakValues(state.break_after, after);
            }
            let span = data.rowspan_block_size != kIndefiniteSize;
            let physical = unsafe {
                &*To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _)
            };
            let fragment = LogicalBoxFragment::new(table.table_writing_direction, physical);
            state.baselines.ProcessCell(
                &fragment,
                ComputeContentAlignmentForTableCell(style),
                span,
                data.has_descendant_that_depends_on_percentage_block_size,
            );
            if encompass {
                state.max_cell_block_size =
                    std::cmp::max(state.max_cell_block_size, fragment.BlockSize());
            }
            let outgoing = physical.GetBreakToken();
            if !outgoing.is_null() && !state.has_inflow_break_inside && !span {
                state.has_inflow_break_inside = !unsafe { &*outgoing }.IsAtBlockEnd();
            }
        }
        state
    }
    // cpp: layoutng_table/table_row_layout_algorithm.cc:39-43,211-278
    pub fn Layout(&mut self) -> *const LayoutResult {
        let table = unsafe { &*self.GetConstraintSpace().TableData() };
        let row = table.rows[self.GetConstraintSpace().TableRowIndex() as usize];
        let fragmentation = self.GetConstraintSpace().HasBlockFragmentation();
        let propagate = self.GetConstraintSpace().ShouldPropagateChildBreakValues();
        let mut baseline = None;
        if !fragmentation {
            baseline = row.baseline;
            if baseline.is_none() {
                let initial = self.PlaceCells(table, &row, row.block_size, None);
                baseline = Some(initial.baselines.ComputeBaseline(row.block_size));
            }
        }
        let mut placed = self.PlaceCells(table, &row, row.block_size, baseline);
        let consumed = if IsBreakInside(self.GetBreakToken()) {
            unsafe { &*To::<TableRowBreakTokenData>((&*self.GetBreakToken()).TokenData()) }
                .previous_consumed_row_block_size
        } else {
            LayoutUnit::default()
        };
        let size = std::cmp::max(placed.max_cell_block_size + consumed, row.block_size);
        if fragmentation {
            if row.block_size != size {
                placed = self.PlaceCells(table, &row, size, None);
            }
            for result in &placed.results {
                self.base
                    .container_builder_
                    .AddResultAtOffset(unsafe { &*result.result.Get() }, result.offset);
            }
        }
        let builder = &mut self.base.container_builder_;
        builder.SetHasSeenAllChildren();
        builder.SetIsKnownToFitInFragmentainer(!placed.has_inflow_break_inside);
        builder.SetIntrinsicBlockSize(placed.max_cell_block_size);
        builder.SetFragmentsTotalBlockSize(size);
        if row.is_collapsed {
            builder.SetIsHiddenForPaint(true);
        }
        builder.SetIsTablePart();
        if propagate {
            builder.SetInitialBreakBefore(placed.break_before);
            builder.SetPreviousBreakAfter(placed.break_after);
        }
        if InvolvedInBlockFragmentationForBuilder(builder) {
            let status = FinishFragmentation(builder);
            debug_assert_eq!(status, BreakStatus::kContinue);
            builder.SetBreakTokenData(
                MakeGarbageCollected(TableRowBreakTokenData::new(
                    consumed + builder.FragmentBlockSize(),
                ))
                .cast(),
            );
        }
        builder.SetBaselines(
            placed
                .baselines
                .ComputeBaseline(builder.FragmentBlockSize()),
        );
        builder.HandleOofsAndSpecialDescendants();
        builder.ToBoxFragment()
    }
}
// cpp: layoutng_table/table_row_layout_algorithm.h:18-33
impl NativeAlgorithm for TableRowLayoutAlgorithm {
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
impl RelayoutAlgorithm<BlockNode> for TableRowLayoutAlgorithm {
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

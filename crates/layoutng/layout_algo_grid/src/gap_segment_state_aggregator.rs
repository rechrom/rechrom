#![allow(non_snake_case)]
#![allow(non_snake_case)]
use crate::internal::gap::cross_gap::CrossGap;
use crate::internal::gap::gap_utils::{
    CellState, CellStates, GapSegmentState, GapSegmentStateAggregator,
    GapSegmentStateAggregatorInternal, GapSegmentStateAggregatorOps, GapSegmentStateRange,
};
use crate::internal::gap::main_gap::MainGap;
use layoutng_style::style::grid_area::GridSpan;

impl GapSegmentStateAggregatorOps for GapSegmentStateAggregator {
    // cpp: layoutng_grid/gap_segment_state_aggregator.cc:14-24
    fn ProcessItem(&mut self, primary_span: &GridSpan, secondary_span: &GridSpan) {
        if primary_span.SpanSize() >= 2 {
            for track_index in primary_span.StartLine()..primary_span.EndLine() {
                self.UpdateGapStateFor(track_index, secondary_span, CellState::kSpanner);
            }
        } else {
            self.UpdateGapStateFor(
                primary_span.StartLine(),
                secondary_span,
                CellState::kOccupied,
            );
        }
    }
    // cpp: layoutng_grid/gap_segment_state_aggregator.cc:26-83,106-107
    fn FinalizeMainGapSegmentStateRangesFor(&self, gap: &mut MainGap, track_index: u32) {
        self.FinalizeRanges(track_index, |range| gap.AddGapSegmentStateRange(&range));
    }
    // cpp: layoutng_grid/gap_segment_state_aggregator.cc:26-83,108-109
    fn FinalizeCrossGapSegmentStateRangesFor(&self, gap: &mut CrossGap, track_index: u32) {
        self.FinalizeRanges(track_index, |range| gap.AddGapSegmentStateRange(&range));
    }
}
impl GapSegmentStateAggregator {
    // cpp: layoutng_grid/gap_segment_state_aggregator.cc:26-83
    // The two source template specializations share one monomorphized callback
    // body; each writes directly to its original MainGap/CrossGap owner.
    fn FinalizeRanges(&self, track_index: u32, mut add: impl FnMut(GapSegmentStateRange)) {
        let current_cells: CellStates = self
            .track_to_cell_states_
            .get(&track_index)
            .cloned()
            .unwrap_or_else(|| vec![CellState::kEmpty; self.cell_count_ as usize]);
        let next_cells: CellStates = self
            .track_to_cell_states_
            .get(&track_index.wrapping_add(1))
            .cloned()
            .unwrap_or_else(|| vec![CellState::kEmpty; self.cell_count_ as usize]);
        let ComputeGapMask = |current: CellState, next: CellState| {
            if current == CellState::kSpanner && next == CellState::kSpanner {
                return GapSegmentState::new(GapSegmentState::kBlocked as u32);
            }
            let mut mask = GapSegmentState::new(GapSegmentState::kNone as u32);
            if current == CellState::kEmpty {
                mask.OrAssignStatus(GapSegmentState::kEmptyBefore);
            }
            if next == CellState::kEmpty {
                mask.OrAssignStatus(GapSegmentState::kEmptyAfter);
            }
            mask
        };
        let mut current_state = ComputeGapMask(current_cells[0], next_cells[0]);
        let mut current_index = 0;
        for i in 1..current_cells.len() {
            let candidate_state = ComputeGapMask(current_cells[i], next_cells[i]);
            if candidate_state.status_ != current_state.status_ {
                if current_state.status_ != GapSegmentState::kNone as u32 {
                    add(GapSegmentStateRange {
                        start: current_index,
                        end: i as u32,
                        state: current_state,
                    });
                }
                current_state = candidate_state;
                current_index = i as u32;
            }
        }
        if current_state.status_ != GapSegmentState::kNone as u32 {
            add(GapSegmentStateRange {
                start: current_index,
                end: current_cells.len() as u32,
                state: current_state,
            });
        }
    }
}
impl GapSegmentStateAggregatorInternal for GapSegmentStateAggregator {
    // cpp: layoutng_grid/gap_segment_state_aggregator.cc:85-101
    fn UpdateGapStateFor(
        &mut self,
        track_index: u32,
        secondary_span: &GridSpan,
        cell_state: CellState,
    ) {
        let count = self.cell_count_ as usize;
        let cell_states = self
            .track_to_cell_states_
            .entry(track_index)
            .or_insert_with(|| vec![CellState::kEmpty; count]);
        for i in secondary_span.StartLine()..secondary_span.EndLine() {
            cell_states[i as usize] = cell_state;
        }
    }
}

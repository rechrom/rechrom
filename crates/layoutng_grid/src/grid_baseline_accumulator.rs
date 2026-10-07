use crate::grid_layout_utils::BaselineAccumulator;
use font_engine::FontBaseline;
use foundation::LayoutUnit;
use layoutng_assembly::internal::grid_item::GridItemData;
use layoutng_assembly::internal::grid_track_collection::{
    GridLayoutTrackCollection, GridTrackBaselines,
};
use layoutng_assembly::logical_box_fragment::LogicalBoxFragment;
use layoutng_style::style::grid_area::GridArea;
use layoutng_style::style::grid_enums::GridTrackSizingDirection::kForRows;

// cpp: layoutng_grid/grid_baseline_accumulator.h:158-170
struct SetIndexAndBaseline {
    set_index: u32,
    baseline: LayoutUnit,
}
struct PositionAndBaseline {
    resolved_position: GridArea,
    baseline: LayoutUnit,
}

// cpp: layoutng_grid/grid_baseline_accumulator.h:23-187
pub struct GridBaselineAccumulator {
    font_baseline_: FontBaseline,
    first_set_index_: u32,
    last_set_index_: u32,
    first_major_baseline_: Option<SetIndexAndBaseline>,
    first_minor_baseline_: Option<SetIndexAndBaseline>,
    first_fallback_baseline_: Option<PositionAndBaseline>,
    last_major_baseline_: Option<SetIndexAndBaseline>,
    last_minor_baseline_: Option<SetIndexAndBaseline>,
    last_fallback_baseline_: Option<PositionAndBaseline>,
}
impl GridBaselineAccumulator {
    // cpp: layoutng_grid/grid_baseline_accumulator.h:27-28,172-182
    pub fn new(font_baseline: FontBaseline) -> Self {
        Self {
            font_baseline_: font_baseline,
            first_set_index_: u32::MAX,
            last_set_index_: u32::MAX,
            first_major_baseline_: None,
            first_minor_baseline_: None,
            first_fallback_baseline_: None,
            last_major_baseline_: None,
            last_minor_baseline_: None,
            last_fallback_baseline_: None,
        }
    }
    // cpp: layoutng_grid/grid_baseline_accumulator.h:38-104
    pub fn AccumulateItem(
        &mut self,
        grid_item: &GridItemData,
        fragment: &LogicalBoxFragment,
        block_offset: LayoutUnit,
    ) {
        let StartsBefore = |a: &GridArea, b: &GridArea| {
            if a.rows.IsTranslatedDefinite() && b.rows.IsTranslatedDefinite() {
                if a.rows.StartLine() < b.rows.StartLine() {
                    return true;
                }
                if a.rows.StartLine() > b.rows.StartLine() {
                    return false;
                }
            }
            if a.columns.IsTranslatedDefinite() && b.columns.IsTranslatedDefinite() {
                return a.columns.StartLine() < b.columns.StartLine();
            }
            false
        };
        let EndsAfter = |a: &GridArea, b: &GridArea| {
            if a.rows.IsTranslatedDefinite() && b.rows.IsTranslatedDefinite() {
                if a.rows.EndLine() > b.rows.EndLine() {
                    return true;
                }
                if a.rows.EndLine() < b.rows.EndLine() {
                    return false;
                }
            }
            if a.columns.IsTranslatedDefinite() && b.columns.IsTranslatedDefinite() {
                return a.columns.EndLine() >= b.columns.EndLine();
            }
            false
        };
        if self.first_fallback_baseline_.as_ref().map_or(true, |b| {
            StartsBefore(&grid_item.resolved_position, &b.resolved_position)
        }) {
            self.first_fallback_baseline_ = Some(PositionAndBaseline {
                resolved_position: grid_item.resolved_position.clone(),
                baseline: block_offset + fragment.FirstBaselineOrSynthesize(self.font_baseline_),
            });
        }
        if self.last_fallback_baseline_.as_ref().map_or(true, |b| {
            EndsAfter(&grid_item.resolved_position, &b.resolved_position)
        }) {
            self.last_fallback_baseline_ = Some(PositionAndBaseline {
                resolved_position: grid_item.resolved_position.clone(),
                baseline: block_offset + fragment.LastBaselineOrSynthesize(self.font_baseline_),
            });
        }
        let set_indices = grid_item.SetIndices(kForRows);
        if self.first_set_index_ == u32::MAX || set_indices.begin < self.first_set_index_ {
            self.first_set_index_ = set_indices.begin;
        }
        if self.last_set_index_ == u32::MAX
            || set_indices.end.wrapping_sub(1) > self.last_set_index_
        {
            self.last_set_index_ = set_indices.end.wrapping_sub(1);
        }
    }
    // cpp: layoutng_grid/grid_baseline_accumulator.h:106-131
    pub fn AccumulateRows(
        &mut self,
        rows: &GridLayoutTrackCollection,
        baselines: &GridTrackBaselines,
    ) {
        for i in 0..rows.GetSetCount() {
            let set_offset = rows.GetSetOffset(i);
            let major_baseline = baselines.major[i as usize];
            if major_baseline != LayoutUnit::Min() {
                let baseline_offset = set_offset + major_baseline;
                if self.first_major_baseline_.is_none() {
                    self.first_major_baseline_ = Some(SetIndexAndBaseline {
                        set_index: i,
                        baseline: baseline_offset,
                    });
                }
                self.last_major_baseline_ = Some(SetIndexAndBaseline {
                    set_index: i,
                    baseline: baseline_offset,
                });
            }
            let minor_baseline = baselines.minor[i as usize];
            if minor_baseline != LayoutUnit::Min() {
                let baseline_offset =
                    set_offset + rows.CalculateSetSpanSizeRange(i, i + 1) - minor_baseline;
                if self.first_minor_baseline_.is_none() {
                    self.first_minor_baseline_ = Some(SetIndexAndBaseline {
                        set_index: i,
                        baseline: baseline_offset,
                    });
                }
                self.last_minor_baseline_ = Some(SetIndexAndBaseline {
                    set_index: i,
                    baseline: baseline_offset,
                });
            }
        }
    }
    // cpp: layoutng_grid/grid_baseline_accumulator.h:133-144
    pub fn FirstBaseline(&self) -> Option<LayoutUnit> {
        if let Some(b) = &self.first_major_baseline_ {
            if b.set_index == self.first_set_index_ {
                return Some(b.baseline);
            }
        }
        if let Some(b) = &self.first_minor_baseline_ {
            if b.set_index == self.first_set_index_ {
                return Some(b.baseline);
            }
        }
        self.first_fallback_baseline_.as_ref().map(|b| b.baseline)
    }
    // cpp: layoutng_grid/grid_baseline_accumulator.h:146-157
    pub fn LastBaseline(&self) -> Option<LayoutUnit> {
        if let Some(b) = &self.last_minor_baseline_ {
            if b.set_index == self.last_set_index_ {
                return Some(b.baseline);
            }
        }
        if let Some(b) = &self.last_major_baseline_ {
            if b.set_index == self.last_set_index_ {
                return Some(b.baseline);
            }
        }
        self.last_fallback_baseline_.as_ref().map(|b| b.baseline)
    }
}
impl BaselineAccumulator for GridBaselineAccumulator {
    // cpp: layoutng_grid/grid_baseline_accumulator.h:30-36
    fn Accumulate(
        &mut self,
        item: &GridItemData,
        fragment: &LogicalBoxFragment,
        block_offset: LayoutUnit,
        _item_stacking_position: LayoutUnit,
        _item_moved_to_earlier_opening: bool,
    ) {
        self.AccumulateItem(item, fragment, block_offset);
    }
    fn FirstBaseline(&self) -> Option<LayoutUnit> {
        GridBaselineAccumulator::FirstBaseline(self)
    }
    fn LastBaseline(&self) -> Option<LayoutUnit> {
        GridBaselineAccumulator::LastBaseline(self)
    }
}

use foundation::Visitor;

use super::grid_track_list::{AutoRepeatType, GridAxisType, GridTrackList};
use super::named_grid_lines_map::NamedGridLinesMap;
use super::ordered_named_grid_lines::OrderedNamedGridLines;

impl foundation::Traceable for ComputedGridTrackList {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ComputedGridTrackList::Trace(self, Some(visitor));
    }
}

// cpp: layoutng_style/style/computed_grid_track_list.h:19-110
pub struct ComputedGridTrackList {
    track_list_: GridTrackList,
    named_grid_lines_: NamedGridLinesMap,
    auto_repeat_named_grid_lines_: NamedGridLinesMap,
    ordered_named_grid_lines_: OrderedNamedGridLines,
    auto_repeat_ordered_named_grid_lines_: OrderedNamedGridLines,
    auto_repeat_insertion_point_: u32,
    auto_repeat_type_: AutoRepeatType,
    axis_type_: GridAxisType,
}

#[allow(non_snake_case)]
impl ComputedGridTrackList {
    // cpp: layoutng_style/style/computed_grid_track_list.h:24-26
    pub fn new(list: &GridTrackList, type_: AutoRepeatType) -> Self {
        Self {
            track_list_: list.clone(),
            named_grid_lines_: NamedGridLinesMap::default(),
            auto_repeat_named_grid_lines_: NamedGridLinesMap::default(),
            ordered_named_grid_lines_: OrderedNamedGridLines::default(),
            auto_repeat_ordered_named_grid_lines_: OrderedNamedGridLines::default(),
            auto_repeat_insertion_point_: 0,
            auto_repeat_type_: type_,
            axis_type_: GridAxisType::kStandaloneAxis,
        }
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:41-43
    pub fn IsSubgriddedAxis(&self) -> bool {
        self.axis_type_ == GridAxisType::kSubgriddedAxis
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:45-54
    pub fn GetNamedGridLines(&self) -> &NamedGridLinesMap {
        &self.named_grid_lines_
    }
    pub fn GetMutableNamedGridLines(&mut self) -> &mut NamedGridLinesMap {
        &mut self.named_grid_lines_
    }
    pub fn SetNamedGridLinesRef(&self) -> &NamedGridLinesMap {
        &self.named_grid_lines_
    }
    pub fn SetNamedGridLines(&mut self, lines: &NamedGridLinesMap) {
        self.named_grid_lines_ = lines.clone();
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:56-61
    pub fn GetAutoRepeatNamedGridLines(&self) -> &NamedGridLinesMap {
        &self.auto_repeat_named_grid_lines_
    }
    pub fn GetMutableAutoRepeatNamedGridLines(&mut self) -> &mut NamedGridLinesMap {
        &mut self.auto_repeat_named_grid_lines_
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:63-71
    pub fn GetOrderedNamedGridLines(&self) -> &OrderedNamedGridLines {
        &self.ordered_named_grid_lines_
    }
    pub fn GetMutableOrderedNamedGridLines(&mut self) -> &mut OrderedNamedGridLines {
        &mut self.ordered_named_grid_lines_
    }
    pub fn SetOrderedNamedGridLines(&mut self, lines: &OrderedNamedGridLines) {
        self.ordered_named_grid_lines_ = lines.clone();
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:73-78
    pub fn GetOrderedAutoRepeatNamedGridLines(&self) -> &OrderedNamedGridLines {
        &self.auto_repeat_ordered_named_grid_lines_
    }
    pub fn GetMutableOrderedAutoRepeatNamedGridLines(&mut self) -> &mut OrderedNamedGridLines {
        &mut self.auto_repeat_ordered_named_grid_lines_
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:80-82
    pub fn GetTrackList(&self) -> &GridTrackList {
        &self.track_list_
    }
    pub fn GetMutableTrackList(&mut self) -> &mut GridTrackList {
        &mut self.track_list_
    }
    pub fn SetTrackList(&mut self, list: &GridTrackList) {
        self.track_list_.AssignFrom(list);
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:84-89
    pub fn GetAutoRepeatInsertionPoint(&self) -> u32 {
        self.auto_repeat_insertion_point_
    }
    pub fn SetAutoRepeatInsertionPoint(&mut self, point: u32) {
        self.auto_repeat_insertion_point_ = point;
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:91-95
    pub fn GetAutoRepeatType(&self) -> AutoRepeatType {
        self.auto_repeat_type_
    }
    pub fn SetAutoRepeatType(&mut self, type_: AutoRepeatType) {
        self.auto_repeat_type_ = type_;
    }
    pub fn GetGridAxisType(&self) -> GridAxisType {
        self.axis_type_
    }
    pub fn SetGridAxisType(&mut self, type_: GridAxisType) {
        self.axis_type_ = type_;
    }

    // cpp: layoutng_style/style/computed_grid_track_list.h:97
    pub fn Trace(&self, _visitor: Option<&mut Visitor>) {}
}

// cpp: layoutng_style/style/computed_grid_track_list.h:22
impl Default for ComputedGridTrackList {
    fn default() -> Self {
        Self {
            track_list_: GridTrackList::default(),
            named_grid_lines_: NamedGridLinesMap::default(),
            auto_repeat_named_grid_lines_: NamedGridLinesMap::default(),
            ordered_named_grid_lines_: OrderedNamedGridLines::default(),
            auto_repeat_ordered_named_grid_lines_: OrderedNamedGridLines::default(),
            auto_repeat_insertion_point_: 0,
            auto_repeat_type_: AutoRepeatType::kNoAutoRepeat,
            axis_type_: GridAxisType::kStandaloneAxis,
        }
    }
}

// cpp: layoutng_style/style/computed_grid_track_list.h:28-39
impl PartialEq for ComputedGridTrackList {
    fn eq(&self, other: &Self) -> bool {
        self.track_list_ == other.track_list_
            && self.named_grid_lines_ == other.named_grid_lines_
            && self.auto_repeat_named_grid_lines_ == other.auto_repeat_named_grid_lines_
            && self.ordered_named_grid_lines_ == other.ordered_named_grid_lines_
            && self.auto_repeat_ordered_named_grid_lines_
                == other.auto_repeat_ordered_named_grid_lines_
            && self.auto_repeat_insertion_point_ == other.auto_repeat_insertion_point_
            && self.auto_repeat_type_ == other.auto_repeat_type_
            && self.axis_type_ == other.axis_type_
    }
}

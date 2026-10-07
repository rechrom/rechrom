use foundation::{String, Vector};
use layoutng_style::style::computed_grid_track_list::ComputedGridTrackList;
use layoutng_style::style::grid_enums::GridTrackSizingDirection;
use layoutng_style::style::grid_track_list::GridAxisType;
use layoutng_style::style::named_grid_lines_map::NamedGridLinesMap;

// cpp: layoutng_grid/grid_named_line_collection.h:15-47
// The source stores borrowed vector pointers; Rust ties all three immutable
// borrows to the maps' lifetime rather than copying line-name state.
pub struct GridNamedLineCollection<'a> {
    named_lines_indexes_: Option<&'a Vector<u32>>,
    auto_repeat_named_lines_indexes_: Option<&'a Vector<u32>>,
    implicit_named_lines_indexes_: Option<&'a Vector<u32>>,
    is_standalone_grid_: bool,
    insertion_point_: u32,
    last_line_: u32,
    auto_repeat_total_tracks_: u32,
    auto_repeat_track_list_length_: u32,
}

impl<'a> GridNamedLineCollection<'a> {
    // cpp: layoutng_grid/grid_named_line_collection.cc:15-79
    pub fn new(
        named_line: &String,
        _track_direction: GridTrackSizingDirection,
        implicit_grid_line_names: &'a NamedGridLinesMap,
        explicit_grid_line_names: &'a NamedGridLinesMap,
        computed_grid_track_list: &'a ComputedGridTrackList,
        last_line: u32,
        auto_repeat_tracks_count: u32,
        is_subgridded_to_parent: bool,
    ) -> Self {
        let is_standalone_grid =
            computed_grid_track_list.GetGridAxisType() == GridAxisType::kStandaloneAxis;
        let are_named_lines_valid = is_subgridded_to_parent || is_standalone_grid;
        let auto_repeat_grid_line_names = computed_grid_track_list.GetAutoRepeatNamedGridLines();
        let mut result = Self {
            named_lines_indexes_: None,
            auto_repeat_named_lines_indexes_: None,
            implicit_named_lines_indexes_: None,
            is_standalone_grid_: is_standalone_grid,
            insertion_point_: 0,
            last_line_: last_line,
            auto_repeat_total_tracks_: auto_repeat_tracks_count,
            auto_repeat_track_list_length_: 0,
        };
        if !explicit_grid_line_names.is_empty() && are_named_lines_valid {
            result.named_lines_indexes_ = explicit_grid_line_names.get(named_line);
        }
        if !auto_repeat_grid_line_names.is_empty() && are_named_lines_valid {
            result.auto_repeat_named_lines_indexes_ = auto_repeat_grid_line_names.get(named_line);
        }
        if !implicit_grid_line_names.is_empty() {
            result.implicit_named_lines_indexes_ = implicit_grid_line_names.get(named_line);
        }
        result.insertion_point_ = computed_grid_track_list.GetAutoRepeatInsertionPoint();
        result.auto_repeat_track_list_length_ = computed_grid_track_list
            .GetTrackList()
            .AutoRepeatTrackCount();
        if result.HasCollapsedAutoRepeat() {
            debug_assert!(!result.is_standalone_grid_);
            result.last_line_ = result.last_line_.wrapping_add(1);
        }
        result
    }

    // cpp: layoutng_grid/grid_named_line_collection.cc:81-83
    fn HasExplicitNamedLines(&self) -> bool {
        self.named_lines_indexes_.is_some() || self.auto_repeat_named_lines_indexes_.is_some()
    }

    // cpp: layoutng_grid/grid_named_line_collection.cc:85-95
    fn HasCollapsedAutoRepeat(&self) -> bool {
        if self.is_standalone_grid_ {
            return false;
        }
        self.auto_repeat_track_list_length_ != 0 && self.auto_repeat_total_tracks_ == 0
    }

    // cpp: layoutng_grid/grid_named_line_collection.cc:97-99
    pub fn HasNamedLines(&self) -> bool {
        self.HasExplicitNamedLines() || self.implicit_named_lines_indexes_.is_some()
    }

    // cpp: layoutng_grid/grid_named_line_collection.cc:101-165
    pub fn Contains(&self, mut line: u32) -> bool {
        assert!(self.HasNamedLines());
        if line > self.last_line_ {
            return false;
        }
        let has_collapsed_auto_repeat = self.HasCollapsedAutoRepeat();
        if has_collapsed_auto_repeat && line >= self.insertion_point_ {
            debug_assert!(!self.is_standalone_grid_);
            line = line.wrapping_add(1);
            debug_assert!(line <= self.last_line_);
        }
        let find = |indexes: Option<&Vector<u32>>, line: u32| {
            indexes.is_some_and(|indexes| indexes.contains(&line))
        };
        if find(self.implicit_named_lines_indexes_, line) {
            return true;
        }
        if self.auto_repeat_track_list_length_ == 0
            || has_collapsed_auto_repeat
            || line < self.insertion_point_
        {
            return find(self.named_lines_indexes_, line);
        }
        if line
            > self
                .insertion_point_
                .wrapping_add(self.auto_repeat_total_tracks_)
        {
            return find(
                self.named_lines_indexes_,
                line.wrapping_sub(self.auto_repeat_total_tracks_.wrapping_sub(1)),
            );
        }
        if self.auto_repeat_total_tracks_ == 0 {
            debug_assert!(!self.is_standalone_grid_);
            return false;
        }
        if line == self.insertion_point_ {
            return find(self.named_lines_indexes_, line)
                || find(self.auto_repeat_named_lines_indexes_, 0);
        }
        if line
            == self
                .insertion_point_
                .wrapping_add(self.auto_repeat_total_tracks_)
        {
            return find(
                self.auto_repeat_named_lines_indexes_,
                self.auto_repeat_track_list_length_,
            ) || find(
                self.named_lines_indexes_,
                self.insertion_point_.wrapping_add(1),
            );
        }
        let auto_repeat_index_in_first_repetition =
            line.wrapping_sub(self.insertion_point_) % self.auto_repeat_track_list_length_;
        if auto_repeat_index_in_first_repetition == 0
            && find(
                self.auto_repeat_named_lines_indexes_,
                self.auto_repeat_track_list_length_,
            )
        {
            return true;
        }
        find(
            self.auto_repeat_named_lines_indexes_,
            auto_repeat_index_in_first_repetition,
        )
    }

    // cpp: layoutng_grid/grid_named_line_collection.cc:167-191
    fn FirstExplicitPosition(&self) -> u32 {
        debug_assert!(self.HasExplicitNamedLines());
        if (self.is_standalone_grid_ && self.auto_repeat_track_list_length_ == 0)
            || self
                .named_lines_indexes_
                .is_some_and(|indexes| indexes[0] <= self.insertion_point_)
        {
            return self.named_lines_indexes_.unwrap()[0];
        }
        if let Some(indexes) = self.auto_repeat_named_lines_indexes_ {
            return indexes[0].wrapping_add(self.insertion_point_);
        }
        let auto_repeat_counted_tracks = if self.auto_repeat_total_tracks_ != 0 {
            self.auto_repeat_total_tracks_ - 1
        } else {
            0
        };
        self.named_lines_indexes_.unwrap()[0].wrapping_add(auto_repeat_counted_tracks)
    }

    // cpp: layoutng_grid/grid_named_line_collection.cc:193-206
    pub fn FirstPosition(&self) -> u32 {
        assert!(self.HasNamedLines());
        let Some(implicit) = self.implicit_named_lines_indexes_ else {
            return self.FirstExplicitPosition();
        };
        if !self.HasExplicitNamedLines() {
            return implicit[0];
        }
        self.FirstExplicitPosition().min(implicit[0])
    }
}

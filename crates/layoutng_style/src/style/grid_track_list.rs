use foundation::{String, Vector};

use super::grid_track_size::GridTrackSize;

// cpp: layoutng_style/style/grid_track_list.h:16-17
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum AutoRepeatType {
    kNoAutoRepeat,
    kAutoFill,
    kAutoFit,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum GridAxisType {
    kStandaloneAxis,
    kSubgriddedAxis,
}

// cpp: layoutng_style/style/grid_track_list.h:21-26
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum GridTrackRepeatType {
    kNoRepeat,
    kAutoFill,
    kAutoFit,
    kInteger,
}

// cpp: layoutng_style/style/grid_track_list.h:19-46
// cpp: layoutng_style/style/grid_track_list.cc:20-24
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridTrackRepeater {
    pub repeat_index: u32,
    pub repeat_size: u32,
    pub repeat_count: u32,
    pub repeat_type: GridTrackRepeatType,
}

#[allow(non_snake_case)]
impl GridTrackRepeater {
    // cpp: layoutng_style/style/grid_track_list.cc:9-16
    pub const fn new(
        repeat_index: u32,
        repeat_size: u32,
        repeat_count: u32,
        repeat_type: GridTrackRepeatType,
    ) -> Self {
        Self {
            repeat_index,
            repeat_size,
            repeat_count,
            repeat_type,
        }
    }

    // cpp: layoutng_style/style/grid_track_list.h:31
    pub fn ToString(&self) -> String {
        unsafe { GridTrackRepeaterToString(self) }
    }
}

// cpp: layoutng_style/style/grid_track_list.h:31
// No definition of this declaration exists in the supplied C++ source tree.
extern "Rust" {
    fn GridTrackRepeaterToString(value: &GridTrackRepeater) -> String;
}

// cpp: layoutng_style/style/grid_track_list.h:48-151
#[derive(Clone)]
pub struct GridTrackList {
    repeaters_: Vector<GridTrackRepeater>,
    repeater_track_sizes_: Vector<GridTrackSize>,
    auto_repeater_index_: u32,
    track_count_without_auto_repeat_: u32,
    track_count_before_auto_repeat_: u32,
    non_auto_repeat_line_count_: u32,
    has_intrinsic_sized_repeater_: bool,
    axis_type_: GridAxisType,
}

#[allow(non_snake_case)]
impl GridTrackList {
    // cpp: layoutng_style/style/grid_track_list.h:52-58
    pub fn with_default_track_size(default_track_size: &GridTrackSize) -> Self {
        Self::with_default_track_size_and_repeat_type(
            default_track_size,
            GridTrackRepeatType::kNoRepeat,
        )
    }
    pub fn with_default_track_size_and_repeat_type(
        default_track_size: &GridTrackSize,
        repeat_type: GridTrackRepeatType,
    ) -> Self {
        let mut list = Self::default();
        let mut sizes = Vector::default();
        sizes.push(default_track_size.clone());
        list.AddRepeater(&sizes, repeat_type, 1, 1);
        list
    }

    // cpp: layoutng_style/style/grid_track_list.h:78-80
    pub const fn TrackCountBeforeAutoRepeat(&self) -> u32 {
        self.track_count_before_auto_repeat_
    }
    // cpp: layoutng_style/style/grid_track_list.h:85
    pub const fn AutoRepeatTrackIndex(&self) -> u32 {
        self.auto_repeater_index_
    }
    // cpp: layoutng_style/style/grid_track_list.h:107-109
    pub const fn HasIntrinsicSizedRepeater(&self) -> bool {
        self.has_intrinsic_sized_repeater_
    }

    // cpp: layoutng_style/style/grid_track_list.h:114
    pub fn ToString(&self) -> String {
        unsafe { GridTrackListToString(self) }
    }
}

// cpp: layoutng_style/style/grid_track_list.h:114
// No definition of this declaration exists in the supplied C++ source tree.
extern "Rust" {
    fn GridTrackListToString(value: &GridTrackList) -> String;
}

// cpp: layoutng_style/style/grid_track_list.h:50-51
// cpp: layoutng_style/style/grid_track_list.h:123-150
impl Default for GridTrackList {
    fn default() -> Self {
        Self {
            repeaters_: Vector::default(),
            repeater_track_sizes_: Vector::default(),
            auto_repeater_index_: u32::MAX,
            track_count_without_auto_repeat_: 0,
            track_count_before_auto_repeat_: 0,
            non_auto_repeat_line_count_: 0,
            has_intrinsic_sized_repeater_: false,
            axis_type_: GridAxisType::kStandaloneAxis,
        }
    }
}

#[allow(non_snake_case)]
impl GridTrackList {
    // cpp: layoutng_style/style/grid_track_list.h:52-54
    pub fn from_default_track_size(default_track_size: &GridTrackSize) -> Self {
        let mut result = Self::default();
        let mut sizes = Vector::default();
        sizes.push(default_track_size.clone());
        result.AddRepeaterDefault(&sizes);
        result
    }

    // cpp: layoutng_style/style/grid_track_list.h:55-58
    pub fn from_default_track_size_with_type(
        default_track_size: &GridTrackSize,
        repeat_type: GridTrackRepeatType,
    ) -> Self {
        let mut result = Self::default();
        let mut sizes = Vector::default();
        sizes.push(default_track_size.clone());
        result.AddRepeaterWithType(&sizes, repeat_type);
        result
    }
}

#[allow(non_snake_case)]
impl GridTrackList {
    // cpp: layoutng_style/style/grid_track_list.cc:26-33
    pub fn RepeatCount(&self, index: u32, auto_value: u32) -> u32 {
        debug_assert!(index < self.RepeaterCount());
        if index == self.auto_repeater_index_ {
            return auto_value;
        }
        self.repeaters_[index as usize].repeat_count
    }

    // cpp: layoutng_style/style/grid_track_list.cc:35-40
    pub fn RepeatIndex(&self, index: u32) -> u32 {
        debug_assert!(!self.IsSubgriddedAxis());
        debug_assert!(index < self.RepeaterCount());
        self.repeaters_[index as usize].repeat_index
    }

    // cpp: layoutng_style/style/grid_track_list.cc:42-45
    pub fn RepeatSize(&self, index: u32) -> u32 {
        debug_assert!(index < self.RepeaterCount());
        self.repeaters_[index as usize].repeat_size
    }

    // cpp: layoutng_style/style/grid_track_list.cc:47-51
    pub fn RepeatType(&self, index: u32) -> GridTrackRepeatType {
        debug_assert!(index < self.RepeaterCount());
        self.repeaters_[index as usize].repeat_type
    }

    // cpp: layoutng_style/style/grid_track_list.cc:53-63
    pub fn RepeatTrackSize(&self, index: u32, n: u32) -> &GridTrackSize {
        debug_assert!(!self.IsSubgriddedAxis());
        debug_assert!(index < self.RepeaterCount());
        debug_assert!(n < self.RepeatSize(index));
        let repeat_index = self.repeaters_[index as usize].repeat_index;
        let track_index = repeat_index.wrapping_add(n);
        debug_assert!((track_index as usize) < self.repeater_track_sizes_.len());
        &self.repeater_track_sizes_[track_index as usize]
    }

    // cpp: layoutng_style/style/grid_track_list.cc:65-71
    pub fn RepeaterCount(&self) -> u32 {
        self.repeaters_.len() as u32
    }
    pub fn TrackCountWithoutAutoRepeat(&self) -> u32 {
        self.track_count_without_auto_repeat_
    }

    // cpp: layoutng_style/style/grid_track_list.cc:73-75
    pub fn AutoRepeatTrackCount(&self) -> u32 {
        if self.HasAutoRepeater() {
            self.repeaters_[self.auto_repeater_index_ as usize].repeat_size
        } else {
            0
        }
    }

    // cpp: layoutng_style/style/grid_track_list.cc:77-80
    pub fn NonAutoRepeatLineCount(&self) -> u32 {
        debug_assert!(self.IsSubgriddedAxis());
        self.non_auto_repeat_line_count_
    }

    // cpp: layoutng_style/style/grid_track_list.cc:82-85
    pub fn IncrementNonAutoRepeatLineCount(&mut self) {
        debug_assert!(self.IsSubgriddedAxis());
        self.non_auto_repeat_line_count_ = self.non_auto_repeat_line_count_.wrapping_add(1);
    }

    // cpp: layoutng_style/style/grid_track_list.cc:87-146
    pub fn AddRepeater(
        &mut self,
        repeater_track_sizes: &Vector<GridTrackSize>,
        repeat_type: GridTrackRepeatType,
        repeat_count: u32,
        repeat_number_of_lines: u32,
    ) -> bool {
        debug_assert!(!self.IsSubgriddedAxis() || repeater_track_sizes.is_empty());
        if !self.IsSubgriddedAxis() && (repeat_count == 0 || repeater_track_sizes.is_empty()) {
            return false;
        }
        debug_assert!(repeat_type == GridTrackRepeatType::kInteger || repeat_count == 1);
        let repeat_size = if self.IsSubgriddedAxis() {
            repeat_number_of_lines
        } else {
            repeater_track_sizes.len() as u32
        };
        match repeat_type {
            GridTrackRepeatType::kNoRepeat | GridTrackRepeatType::kInteger => {
                if repeat_size > self.AvailableTrackCount() / repeat_count {
                    return false;
                }
                if !self.IsSubgriddedAxis() {
                    self.track_count_without_auto_repeat_ = self
                        .track_count_without_auto_repeat_
                        .wrapping_add(repeat_size.wrapping_mul(repeat_count));
                }
            }
            GridTrackRepeatType::kAutoFill | GridTrackRepeatType::kAutoFit => {
                self.track_count_before_auto_repeat_ = self.track_count_without_auto_repeat_;
                self.has_intrinsic_sized_repeater_ = repeater_track_sizes
                    .iter()
                    .any(|track_size| track_size.IsTrackDefinitionIntrinsic());
                if self.HasAutoRepeater() || repeat_size > self.AvailableTrackCount() {
                    return false;
                }
                self.auto_repeater_index_ = self.repeaters_.len() as u32;
            }
        }
        self.repeaters_.push(GridTrackRepeater::new(
            self.repeater_track_sizes_.len() as u32,
            repeat_size,
            repeat_count,
            repeat_type,
        ));
        if !self.IsSubgriddedAxis() {
            self.repeater_track_sizes_
                .extend_from_slice(repeater_track_sizes);
        }
        true
    }

    // cpp: layoutng_style/style/grid_track_list.h:92-97
    pub fn AddRepeaterDefault(&mut self, sizes: &Vector<GridTrackSize>) -> bool {
        self.AddRepeater(sizes, GridTrackRepeatType::kNoRepeat, 1, 1)
    }
    pub fn AddRepeaterWithType(
        &mut self,
        sizes: &Vector<GridTrackSize>,
        repeat_type: GridTrackRepeatType,
    ) -> bool {
        self.AddRepeater(sizes, repeat_type, 1, 1)
    }
    pub fn AddRepeaterWithCount(
        &mut self,
        sizes: &Vector<GridTrackSize>,
        repeat_type: GridTrackRepeatType,
        repeat_count: u32,
    ) -> bool {
        self.AddRepeater(sizes, repeat_type, repeat_count, 1)
    }

    // cpp: layoutng_style/style/grid_track_list.cc:150-160
    pub fn HasAutoRepeater(&self) -> bool {
        self.auto_repeater_index_ != u32::MAX
    }
    pub fn IsSubgriddedAxis(&self) -> bool {
        self.axis_type_ == GridAxisType::kSubgriddedAxis
    }
    pub fn SetAxisType(&mut self, axis_type: GridAxisType) {
        self.axis_type_ = axis_type;
    }

    // cpp: layoutng_style/style/grid_track_list.cc:162-164
    fn AvailableTrackCount(&self) -> u32 {
        u32::MAX
            .wrapping_sub(1)
            .wrapping_sub(self.track_count_without_auto_repeat_)
    }

    // cpp: layoutng_style/style/grid_track_list.h:112
    pub fn Clear(&mut self) {
        unsafe { GridTrackListClear(self) }
    }

    // cpp: layoutng_style/style/grid_track_list.h:116
    // cpp: layoutng_style/style/grid_track_list.cc:166-175
    pub fn AssignFrom(&mut self, other: &Self) {
        self.repeaters_ = other.repeaters_.clone();
        self.repeater_track_sizes_ = other.repeater_track_sizes_.clone();
        self.auto_repeater_index_ = other.auto_repeater_index_;
        self.track_count_without_auto_repeat_ = other.track_count_without_auto_repeat_;
        self.track_count_before_auto_repeat_ = other.track_count_before_auto_repeat_;
        self.non_auto_repeat_line_count_ = other.non_auto_repeat_line_count_;
        self.axis_type_ = other.axis_type_;
        self.has_intrinsic_sized_repeater_ = other.has_intrinsic_sized_repeater_;
    }
}

// cpp: layoutng_style/style/grid_track_list.h:112
// Declared in C++ but no definition exists in the supplied source tree.
extern "Rust" {
    fn GridTrackListClear(value: &mut GridTrackList);
}

// cpp: layoutng_style/style/grid_track_list.h:117
// cpp: layoutng_style/style/grid_track_list.cc:177-186
impl PartialEq for GridTrackList {
    fn eq(&self, other: &Self) -> bool {
        self.TrackCountWithoutAutoRepeat() == other.TrackCountWithoutAutoRepeat()
            && self.RepeaterCount() == other.RepeaterCount()
            && self.auto_repeater_index_ == other.auto_repeater_index_
            && self.repeaters_ == other.repeaters_
            && self.repeater_track_sizes_ == other.repeater_track_sizes_
            && self.non_auto_repeat_line_count_ == other.non_auto_repeat_line_count_
            && self.axis_type_ == other.axis_type_
            && self.has_intrinsic_sized_repeater_ == other.has_intrinsic_sized_repeater_
    }
}

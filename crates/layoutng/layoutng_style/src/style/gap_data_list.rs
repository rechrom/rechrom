use foundation::{EBorderStyle, HeapVector, Length, String, Vector, VectorExt, Visitor, WtfSizeT};

use super::gap_data::{GapData, GapRepeatedValues, GapValue};
use crate::css::style_color::StyleColor;

pub type GapDataVector<T> = HeapVector<GapData<T>, 1>;
const NOT_FOUND: WtfSizeT = WtfSizeT::MAX;

// cpp: layoutng_style/style/gap_data_list.h:23-30
#[derive(Clone, Copy)]
pub struct RegionSlotCounts {
    pub leading: WtfSizeT,
    pub auto_repeat: WtfSizeT,
    pub trailing: WtfSizeT,
    pub auto_idx: WtfSizeT,
    pub leading_has_integer_repeaters: bool,
    pub trailing_has_integer_repeaters: bool,
}

// cpp: layoutng_style/style/gap_data_list.h:35-72
#[allow(non_snake_case)]
pub fn ComputeRegionSlotCounts<T: GapValue>(
    gap_data_list: &GapDataVector<T>,
    gap_count: WtfSizeT,
) -> RegionSlotCounts {
    let mut counts = RegionSlotCounts {
        leading: 0,
        auto_repeat: 0,
        trailing: 0,
        auto_idx: NOT_FOUND,
        leading_has_integer_repeaters: false,
        trailing_has_integer_repeaters: false,
    };
    for i in 0..gap_data_list.size() {
        let gap_data = gap_data_list.at(i);
        let mut is_integer_repeater = false;
        if gap_data.IsRepeaterData() {
            if gap_data.GetValueRepeater().IsAutoRepeater() {
                assert_eq!(counts.auto_idx, NOT_FOUND);
                counts.auto_idx = i;
                continue;
            }
            is_integer_repeater = true;
        }
        let slots = gap_data.GetFixedSlotCount();
        if counts.auto_idx == NOT_FOUND {
            counts.leading += slots;
            counts.leading_has_integer_repeaters |= is_integer_repeater;
        } else {
            counts.trailing += slots;
            counts.trailing_has_integer_repeaters |= is_integer_repeater;
        }
    }
    if counts.auto_idx != NOT_FOUND {
        let combined = counts.leading + counts.trailing;
        if combined < gap_count {
            counts.auto_repeat = gap_count - combined;
        }
    }
    counts
}

// cpp: layoutng_style/style/gap_data_list.h:83-169
#[derive(Clone)]
pub struct GapDataList<T: GapValue> {
    gap_data_list_: GapDataVector<T>,
}

// cpp: layoutng_style/style/gap_data_list.h:91
impl<T: GapValue> Default for GapDataList<T> {
    fn default() -> Self {
        Self {
            gap_data_list_: GapDataVector::default(),
        }
    }
}

#[allow(non_snake_case)]
impl<T: GapValue> GapDataList<T> {
    // cpp: layoutng_style/style/gap_data_list.h:106-115
    pub fn from_vector(gap_data_list: GapDataVector<T>) -> Self {
        assert!(!gap_data_list.empty());
        Self {
            gap_data_list_: gap_data_list,
        }
    }

    pub fn from_value(value: &T) -> Self {
        let mut result = Self::default();
        result
            .gap_data_list_
            .emplace_back(GapData::from_value(value.clone()));
        result
    }

    pub fn with_capacity(size: WtfSizeT) -> Self {
        let mut result = Self::default();
        result.gap_data_list_.ReserveInitialCapacity(size);
        result
    }

    // cpp: layoutng_style/style/gap_data_list.h:117-119
    pub fn AddGapData(&mut self, gap_data: &GapData<T>) {
        self.gap_data_list_.push_back(gap_data.clone());
    }

    // cpp: layoutng_style/style/gap_data_list.h:129-146
    pub fn ToString(&self) -> String
    where
        T: std::fmt::Display,
    {
        use std::fmt::Write;
        let mut result = std::string::String::new();
        for gap_data in self.gap_data_list_.iter() {
            if gap_data.IsRepeaterData() {
                let repeater = gap_data.GetValueRepeater();
                let _ = write!(result, "Repeater: {}, ", repeater.RepeatCount());
                for i in 0..repeater.RepeatedValues().size() {
                    let _ = write!(result, "{} ", repeater.RepeatedValues().at(i));
                }
            } else {
                let _ = write!(result, "Value: {}", gap_data.GetValue());
            }
            result.push_str("; ");
        }
        String::from(result.as_str())
    }

    // cpp: layoutng_style/style/gap_data_list.h:148
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.gap_data_list_);
    }

    // cpp: layoutng_style/style/gap_data_list.h:150
    pub fn GetGapDataList(&self) -> &GapDataVector<T> {
        &self.gap_data_list_
    }

    // cpp: layoutng_style/style/gap_data_list.h:152-154
    pub fn HasSingleValue(&self) -> bool {
        self.gap_data_list_.size() == 1 && !self.gap_data_list_.at(0).IsRepeaterData()
    }

    // cpp: layoutng_style/style/gap_data_list.h:156-161
    pub fn GetSingleValue(&self) -> T {
        debug_assert!(self.HasSingleValue());
        self.gap_data_list_.at(0).GetValue()
    }

    pub fn GetLegacyValue(&self) -> T {
        self.gap_data_list_.at(0).GetValue()
    }
}

#[allow(non_snake_case)]
impl GapDataList<StyleColor> {
    // cpp: layoutng_style/style/gap_data_list.h:93-95
    pub fn DefaultGapColorDataList() -> Self {
        Self::from_value(&StyleColor::CurrentColor())
    }

    // cpp: layoutng_style/style/gap_data_list.h:125-127
    pub fn AddGapDataColor(&mut self, color: &StyleColor) {
        self.gap_data_list_
            .emplace_back(GapData::from_value(color.clone()));
    }
}

#[allow(non_snake_case)]
impl GapDataList<i32> {
    // cpp: layoutng_style/style/gap_data_list.h:97-100
    pub fn DefaultGapWidthDataList() -> Self {
        Self::from_value(&3)
    }

    // cpp: layoutng_style/style/gap_data_list.h:121-123
    pub fn AddGapDataLength(&mut self, length: &Length) {
        self.gap_data_list_
            .emplace_back(GapData::from_value(length.Pixels() as i32));
    }
}

#[allow(non_snake_case)]
impl GapDataList<EBorderStyle> {
    // cpp: layoutng_style/style/gap_data_list.h:102-104
    pub fn DefaultGapStyleDataList() -> Self {
        Self::from_value(&EBorderStyle::kNone)
    }
}

// cpp: layoutng_style/style/gap_data_list.h:163-165
impl<T: GapValue> PartialEq for GapDataList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.gap_data_list_ == other.gap_data_list_
    }
}

// cpp: layoutng_style/style/gap_data_list.h:183-187
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GapDataListRegion {
    Leading,
    Auto,
    Trailing,
}

// cpp: layoutng_style/style/gap_data_list.h:181-369
pub struct GapDataListIterator<'a, T: GapValue> {
    gap_data_list_: &'a GapDataVector<T>,
    gap_count_: WtfSizeT,
    counts_: RegionSlotCounts,
    current_gap_index_: WtfSizeT,
    region_: GapDataListRegion,
    current_region_slots_remaining_: WtfSizeT,
    list_idx_: WtfSizeT,
    repeats_left_: WtfSizeT,
    repeated_value_idx_: WtfSizeT,
}

#[allow(non_snake_case)]
impl<'a, T: GapValue> GapDataListIterator<'a, T> {
    // cpp: layoutng_style/style/gap_data_list.h:193-215
    pub fn new(gap_data_list: &'a GapDataVector<T>, gap_count: WtfSizeT) -> Self {
        assert!(!gap_data_list.empty());
        let counts = ComputeRegionSlotCounts(gap_data_list, gap_count);
        let mut result = Self {
            gap_data_list_: gap_data_list,
            gap_count_: gap_count,
            counts_: counts,
            current_gap_index_: 0,
            region_: GapDataListRegion::Leading,
            current_region_slots_remaining_: 0,
            list_idx_: 0,
            repeats_left_: 0,
            repeated_value_idx_: 0,
        };
        if counts.auto_idx == 0 {
            result.region_ = GapDataListRegion::Auto;
            result.current_region_slots_remaining_ = counts.auto_repeat;
            result.repeated_value_idx_ = 0;
            if result.current_region_slots_remaining_ == 0 {
                result.TransitionToNextRegion();
            }
        } else {
            result.region_ = GapDataListRegion::Leading;
            result.current_region_slots_remaining_ = counts.leading;
            result.list_idx_ = 0;
            result.InitNonAutoDataState();
        }
        result
    }

    // cpp: layoutng_style/style/gap_data_list.h:217
    pub fn HasNext(&self) -> bool {
        self.current_gap_index_ < self.gap_count_
    }

    // cpp: layoutng_style/style/gap_data_list.h:219-227
    pub fn AdvanceUpTo(&mut self, target_index: WtfSizeT) {
        assert!(self.current_gap_index_ <= target_index);
        assert!(target_index <= self.gap_count_);
        while self.current_gap_index_ < target_index {
            self.Next();
        }
    }

    // cpp: layoutng_style/style/gap_data_list.h:229-244
    pub fn Next(&mut self) -> T {
        assert!(self.HasNext());
        let value = self.GetData();
        self.current_region_slots_remaining_ -= 1;
        self.current_gap_index_ += 1;
        if self.current_region_slots_remaining_ > 0 {
            self.AdvanceWithinCurrentRegion();
        } else if self.current_gap_index_ < self.gap_count_ {
            self.TransitionToNextRegion();
        }
        value
    }

    // cpp: layoutng_style/style/gap_data_list.h:247-255
    fn GetData(&self) -> T {
        let index = if self.region_ == GapDataListRegion::Auto {
            self.counts_.auto_idx
        } else {
            self.list_idx_
        };
        let data = self.gap_data_list_.at(index);
        if data.IsRepeaterData() {
            data.GetValueRepeater()
                .RepeatedValues()
                .at(self.repeated_value_idx_)
                .clone()
        } else {
            data.GetValue()
        }
    }

    // cpp: layoutng_style/style/gap_data_list.h:257-263
    fn AdvanceWithinCurrentRegion(&mut self) {
        if self.region_ == GapDataListRegion::Auto {
            self.AdvanceWithinAutoRegion();
        } else {
            self.AdvanceWithinNonAutoRegion();
        }
    }

    // cpp: layoutng_style/style/gap_data_list.h:265-290
    fn AdvanceWithinNonAutoRegion(&mut self) {
        self.repeated_value_idx_ += 1;
        let data = self.gap_data_list_.at(self.list_idx_);
        let value_count = if data.IsRepeaterData() {
            data.GetValueRepeater().RepeatedValues().size()
        } else {
            1
        };
        if self.repeated_value_idx_ == value_count {
            self.repeated_value_idx_ = 0;
            self.repeats_left_ -= 1;
            if self.repeats_left_ == 0 {
                self.list_idx_ += 1;
                self.InitNonAutoDataState();
            }
        }
    }

    // cpp: layoutng_style/style/gap_data_list.h:292-299
    fn AdvanceWithinAutoRegion(&mut self) {
        assert_eq!(self.region_, GapDataListRegion::Auto);
        let size = self
            .gap_data_list_
            .at(self.counts_.auto_idx)
            .GetValueRepeater()
            .RepeatedValues()
            .size();
        self.repeated_value_idx_ = (self.repeated_value_idx_ + 1) % size;
    }

    // cpp: layoutng_style/style/gap_data_list.h:301-339
    fn TransitionToNextRegion(&mut self) {
        match self.region_ {
            GapDataListRegion::Leading => {
                if self.counts_.auto_idx == NOT_FOUND {
                    self.current_region_slots_remaining_ = self.counts_.leading;
                    self.list_idx_ = 0;
                    self.InitNonAutoDataState();
                } else if self.counts_.auto_repeat > 0 {
                    self.region_ = GapDataListRegion::Auto;
                    self.current_region_slots_remaining_ = self.counts_.auto_repeat;
                    self.repeated_value_idx_ = 0;
                } else {
                    self.region_ = GapDataListRegion::Trailing;
                    self.current_region_slots_remaining_ = self.counts_.trailing;
                    self.list_idx_ = self.counts_.auto_idx + 1;
                    self.InitNonAutoDataState();
                }
            }
            GapDataListRegion::Auto => {
                self.region_ = GapDataListRegion::Trailing;
                self.current_region_slots_remaining_ = self.counts_.trailing;
                self.list_idx_ = self.counts_.auto_idx + 1;
                self.InitNonAutoDataState();
            }
            GapDataListRegion::Trailing => assert_eq!(self.current_gap_index_, self.gap_count_),
        }
    }

    // cpp: layoutng_style/style/gap_data_list.h:341-350
    fn InitNonAutoDataState(&mut self) {
        let data = self.gap_data_list_.at(self.list_idx_);
        self.repeats_left_ = if data.IsRepeaterData() {
            assert!(!data.GetValueRepeater().IsAutoRepeater());
            data.GetValueRepeater().RepeatCount()
        } else {
            1
        };
        self.repeated_value_idx_ = 0;
    }
}

// cpp: layoutng_style/style/gap_data_list.h:450-455
#[derive(Default)]
struct FixedRegion {
    start_list_index: WtfSizeT,
    total_slots: WtfSizeT,
    slot_ends: Vector<WtfSizeT>,
}

// cpp: layoutng_style/style/gap_data_list.h:388-520
pub struct GapDataListValueAccessor<'a, T: GapValue> {
    gap_data_list_: &'a GapDataVector<T>,
    gap_count_: WtfSizeT,
    auto_repeat_slot_count_: WtfSizeT,
    auto_idx_: WtfSizeT,
    leading_region_: FixedRegion,
    trailing_region_: FixedRegion,
}

#[allow(non_snake_case)]
impl<'a, T: GapValue> GapDataListValueAccessor<'a, T> {
    // cpp: layoutng_style/style/gap_data_list.h:396-416
    pub fn new(gap_data_list: &'a GapDataVector<T>, gap_count: WtfSizeT) -> Self {
        assert!(!gap_data_list.empty());
        let counts = ComputeRegionSlotCounts(gap_data_list, gap_count);
        let mut result = Self {
            gap_data_list_: gap_data_list,
            gap_count_: gap_count,
            auto_repeat_slot_count_: counts.auto_repeat,
            auto_idx_: counts.auto_idx,
            leading_region_: FixedRegion::default(),
            trailing_region_: FixedRegion::default(),
        };
        if !result.HasAutoRepeater() {
            result.leading_region_ = result.BuildFixedRegion(
                0,
                gap_data_list.size(),
                counts.leading,
                counts.leading_has_integer_repeaters,
            );
        } else {
            result.leading_region_ = result.BuildFixedRegion(
                0,
                counts.auto_idx,
                counts.leading,
                counts.leading_has_integer_repeaters,
            );
            result.trailing_region_ = result.BuildFixedRegion(
                counts.auto_idx + 1,
                gap_data_list.size(),
                counts.trailing,
                counts.trailing_has_integer_repeaters,
            );
        }
        result
    }

    // cpp: layoutng_style/style/gap_data_list.h:420-444
    pub fn ValueAt(&self, index: WtfSizeT) -> T {
        assert!(index < self.gap_count_);
        if !self.HasAutoRepeater() {
            return self.ValueInFixedRegion(
                &self.leading_region_,
                index % self.leading_region_.total_slots,
            );
        }
        let leading = self.leading_region_.total_slots;
        if index < leading {
            return self.ValueInFixedRegion(&self.leading_region_, index);
        }
        if index < leading + self.auto_repeat_slot_count_ {
            let values = self
                .gap_data_list_
                .at(self.auto_idx_)
                .GetValueRepeater()
                .RepeatedValues();
            assert!(values.size() > 0);
            return values.at((index - leading) % values.size()).clone();
        }
        let trailing_start = leading + self.auto_repeat_slot_count_;
        assert!(index >= trailing_start);
        self.ValueInFixedRegion(&self.trailing_region_, index - trailing_start)
    }

    // cpp: layoutng_style/style/gap_data_list.h:457
    fn HasAutoRepeater(&self) -> bool {
        self.auto_idx_ != NOT_FOUND
    }

    // cpp: layoutng_style/style/gap_data_list.h:459-481
    fn BuildFixedRegion(
        &self,
        begin: WtfSizeT,
        end: WtfSizeT,
        total_slots: WtfSizeT,
        has_integer_repeaters: bool,
    ) -> FixedRegion {
        assert!(begin <= end);
        assert!(end <= self.gap_data_list_.size());
        let mut region = FixedRegion {
            start_list_index: begin,
            total_slots,
            slot_ends: Vector::default(),
        };
        if !has_integer_repeaters {
            assert_eq!(total_slots, end - begin);
            return region;
        }
        region.slot_ends.ReserveInitialCapacity(end - begin);
        let mut running_total = 0;
        for i in begin..end {
            running_total += self.gap_data_list_.at(i).GetFixedSlotCount();
            region.slot_ends.push_back(running_total);
        }
        assert_eq!(running_total, total_slots);
        region
    }

    // cpp: layoutng_style/style/gap_data_list.h:483-506
    fn ValueInFixedRegion(&self, region: &FixedRegion, local_index: WtfSizeT) -> T {
        assert!(local_index < region.total_slots);
        if region.slot_ends.empty() {
            return self
                .gap_data_list_
                .at(region.start_list_index + local_index)
                .GetValue();
        }
        debug_assert!((1..region.slot_ends.size())
            .all(|i| region.slot_ends.at(i - 1) <= region.slot_ends.at(i)));
        let mut low = 0;
        let mut high = region.slot_ends.size();
        while low < high {
            let mid = low + (high - low) / 2;
            if *region.slot_ends.at(mid) <= local_index {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        assert!(low < region.slot_ends.size());
        let entry_start = if low == 0 {
            0
        } else {
            *region.slot_ends.at(low - 1)
        };
        let data = self.gap_data_list_.at(region.start_list_index + low);
        if !data.IsRepeaterData() {
            return data.GetValue();
        }
        let values = data.GetValueRepeater().RepeatedValues();
        values
            .at((local_index - entry_start) % values.size())
            .clone()
    }
}

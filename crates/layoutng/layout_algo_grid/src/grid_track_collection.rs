#![allow(non_snake_case)]
// cpp: layoutng_grid/grid_track_collection.cc:14-19
const _: () = assert!(std::mem::size_of::<GridRange>() == 7 * std::mem::size_of::<u32>());

// Compiled once in layoutng_assembly beside the shared header's owning types.
use crate::internal::grid_track_collection::*;
use foundation::{
    kIndefiniteSize, LayoutUnit, Length, MakeGarbageCollected, MinimumValueForLength,
};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_style::style::{
    grid_enums::GridTrackSizingDirection::{self, *},
    grid_track_list::{GridTrackList, GridTrackRepeatType},
    grid_track_size::GridTrackSize,
};

// cpp: layoutng_grid/grid_track_collection.cc:21-23
pub fn RangeEndLine(collection: &dyn GridTrackCollectionBase, index: u32) -> u32 {
    collection
        .RangeStartLine(index)
        .wrapping_add(collection.RangeTrackCount(index))
}
// cpp: layoutng_grid/grid_track_collection.cc:25-48
pub fn RangeIndexFromGridLine(collection: &dyn GridTrackCollectionBase, line: u32) -> u32 {
    let mut upper = collection.RangeCount();
    debug_assert!(upper > 0);
    debug_assert!(
        line < collection
            .RangeStartLine(upper - 1)
            .wrapping_add(collection.RangeTrackCount(upper - 1))
    );
    let mut lower = 0;
    while lower < upper {
        let center = lower.wrapping_add(upper) >> 1;
        let start = collection.RangeStartLine(center);
        if line < start {
            upper = center;
        } else if line < start.wrapping_add(collection.RangeTrackCount(center)) {
            return center;
        } else {
            lower = center + 1;
        }
    }
    lower
}

// cpp: layoutng_grid/grid_track_collection.cc:50-52
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeIsCollapsed(this: &GridRange) -> bool {
    this.properties.HasProperty(PropertyId::kIsCollapsed)
}
// cpp: layoutng_grid/grid_track_collection.cc:54-56
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeIsImplicit(this: &GridRange) -> bool {
    this.properties.HasProperty(PropertyId::kIsImplicit)
}
// cpp: layoutng_grid/grid_track_collection.cc:58-60
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeIsAutoRepeat(this: &GridRange) -> bool {
    this.properties.HasProperty(PropertyId::kIsAutoRepeat)
}
// cpp: layoutng_grid/grid_track_collection.cc:62-64
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeSetIsCollapsed(this: &mut GridRange) {
    this.properties.SetProperty(PropertyId::kIsCollapsed);
}
// cpp: layoutng_grid/grid_track_collection.cc:66-68
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeSetIsImplicit(this: &mut GridRange) {
    this.properties.SetProperty(PropertyId::kIsImplicit);
}
// cpp: layoutng_grid/grid_track_collection.cc:70-72
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeSetIsAutoRepeat(this: &mut GridRange) {
    this.properties.SetProperty(PropertyId::kIsAutoRepeat);
}

// cpp: layoutng_grid/grid_track_collection.cc:76-88
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeBuilderEnsureTrackCoverage(
    this: &mut GridRangeBuilder,
    start: u32,
    span: u32,
    start_index: *mut u32,
    end_index: *mut u32,
) {
    debug_assert_ne!(start, u32::MAX);
    debug_assert_ne!(span, u32::MAX);
    debug_assert!(!start_index.is_null() && !end_index.is_null());
    this.must_sort_grid_lines_ = true;
    this.start_lines_
        .push(TrackBoundaryToRangePair::new(start, start_index));
    this.end_lines_.push(TrackBoundaryToRangePair::new(
        start.wrapping_add(span),
        end_index,
    ));
}

// cpp: layoutng_grid/grid_track_collection.cc:90-319
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeBuilderFinalizeRanges(
    this: &mut GridRangeBuilder,
    needs_intrinsic_track_size: bool,
) -> GridRangeVector {
    debug_assert_eq!(this.start_lines_.len(), this.end_lines_.len());
    if this.must_sort_grid_lines_ {
        this.start_lines_.sort_unstable_by_key(|a| a.grid_line);
        this.end_lines_.sort_unstable_by_key(|a| a.grid_line);
        this.must_sort_grid_lines_ = false;
    }
    let explicit = unsafe { &*this.explicit_tracks_ };
    let implicit = unsafe { &*this.implicit_tracks_ };
    let explicit_count = if explicit.IsSubgriddedAxis() {
        0
    } else {
        explicit.RepeaterCount()
    };
    let count = this.start_lines_.len();
    let mut ranges = GridRangeVector::new();
    let mut auto_fit = false;
    let mut explicit_line = this.start_offset_;
    let mut explicit_index = u32::MAX;
    let mut range_start: u32 = 0;
    let mut set_index: u32 = 0;
    let mut open: u32 = 0;
    let mut next_explicit = if explicit_count != 0 {
        this.start_offset_
    } else {
        u32::MAX
    };
    let mut start_index = 0usize;
    let mut end_index = 0usize;
    loop {
        while start_index < count && range_start >= this.start_lines_[start_index].grid_line {
            start_index += 1;
            open = open.wrapping_add(1);
        }
        while end_index < count && range_start >= this.end_lines_[end_index].grid_line {
            end_index += 1;
            open = open.wrapping_sub(1);
        }
        if end_index >= count {
            break;
        }
        let next_start = if start_index < count {
            this.start_lines_[start_index].grid_line
        } else {
            u32::MAX
        };
        let next_end = this.end_lines_[end_index].grid_line;
        debug_assert!(next_start != u32::MAX || next_end < next_start);
        while range_start == next_explicit {
            explicit_line = next_explicit;
            explicit_index = explicit_index.wrapping_add(1);
            if explicit_index == explicit_count {
                explicit_index = u32::MAX;
                auto_fit = false;
                break;
            }
            auto_fit = explicit.RepeatType(explicit_index) == GridTrackRepeatType::kAutoFit;
            next_explicit = next_explicit.wrapping_add(
                explicit
                    .RepeatSize(explicit_index)
                    .wrapping_mul(explicit.RepeatCount(explicit_index, this.auto_repetitions_)),
            );
        }
        let mut repeater_size = 1;
        let mut range = GridRange {
            begin_set_index: set_index,
            repeater_index: u32::MAX,
            repeater_offset: 0,
            set_count: 0,
            start_line: range_start,
            track_count: next_start.min(next_end).wrapping_sub(range_start),
            properties: TrackSpanProperties::default(),
        };
        debug_assert!(range.track_count > 0);
        if explicit.IsSubgriddedAxis() {
            range.repeater_index = u32::MAX;
            range.repeater_offset = 0;
        } else if explicit_index != u32::MAX {
            repeater_size = explicit.RepeatSize(explicit_index);
            range.repeater_index = explicit_index;
            range.repeater_offset = range_start.wrapping_sub(explicit_line) % repeater_size;
            if matches!(
                explicit.RepeatType(explicit_index),
                GridTrackRepeatType::kAutoFit | GridTrackRepeatType::kAutoFill
            ) {
                range.SetIsAutoRepeat();
            }
        } else {
            range.SetIsImplicit();
            if implicit.RepeaterCount() == 0 {
                range.repeater_index = u32::MAX;
                range.repeater_offset = 0;
            } else {
                repeater_size = implicit.RepeatSize(0);
                range.repeater_index = 0;
                range.repeater_offset = range_start
                    .wrapping_add(repeater_size)
                    .wrapping_sub(explicit_line % repeater_size)
                    % repeater_size;
            }
        }
        if start_index != 0 {
            let mut index = start_index - 1;
            while this.start_lines_[index].grid_line == range.start_line {
                let out = this.start_lines_[index].grid_item_range_index_to_cache;
                if !out.is_null() {
                    unsafe {
                        *out = ranges.len() as u32;
                    }
                }
                if index == 0 {
                    break;
                }
                index -= 1;
            }
        }
        let end_line = range.start_line.wrapping_add(range.track_count);
        let mut index = end_index;
        while index < count && this.end_lines_[index].grid_line == end_line {
            let out = this.end_lines_[index].grid_item_range_index_to_cache;
            if !out.is_null() {
                unsafe {
                    *out = ranges.len() as u32;
                }
            }
            index += 1;
        }
        if auto_fit && open == 1 && !needs_intrinsic_track_size {
            range.SetIsCollapsed();
            range.set_count = 0;
        } else {
            range.set_count = repeater_size.min(range.track_count);
            debug_assert!(range.set_count > 0);
        }
        range_start = range_start.wrapping_add(range.track_count);
        set_index = set_index.wrapping_add(range.set_count);
        ranges.push(range);
    }
    debug_assert_eq!(start_index, count);
    debug_assert_eq!(end_index, count);
    debug_assert_eq!(open, 0);
    if explicit_index != u32::MAX {
        debug_assert_eq!(explicit_index, explicit_count - 1);
        debug_assert_eq!(range_start, next_explicit);
    }
    ranges
}

// cpp: layoutng_grid/grid_track_collection.cc:321-368
#[unsafe(no_mangle)]
pub extern "Rust" fn GridRangeBuilderNewFromTracks(
    explicit: &GridTrackList,
    implicit: &GridTrackList,
    repetitions: u32,
    offset: u32,
) -> GridRangeBuilder {
    debug_assert!(implicit.RepeaterCount() <= 1);
    debug_assert_ne!(repetitions, u32::MAX);
    let count = if explicit.IsSubgriddedAxis() {
        0
    } else {
        explicit.RepeaterCount()
    };
    let mut this = GridRangeBuilder {
        auto_repetitions_: repetitions,
        start_offset_: offset,
        must_sort_grid_lines_: false,
        explicit_tracks_: explicit,
        implicit_tracks_: implicit,
        start_lines_: Vec::with_capacity(count as usize + 1),
        end_lines_: Vec::with_capacity(count as usize + 1),
    };
    let mut line = offset;
    for i in 0..count {
        let tracks = explicit
            .RepeatCount(i, repetitions)
            .wrapping_mul(explicit.RepeatSize(i));
        if tracks == 0 {
            debug_assert!(
                explicit.IsSubgriddedAxis()
                    || explicit.HasIntrinsicSizedRepeater()
                    || implicit.HasIntrinsicSizedRepeater()
            );
            continue;
        }
        this.start_lines_
            .push(TrackBoundaryToRangePair::new(line, std::ptr::null_mut()));
        line = line.wrapping_add(tracks);
        this.end_lines_
            .push(TrackBoundaryToRangePair::new(line, std::ptr::null_mut()));
    }
    this
}

// cpp: layoutng_grid/grid_track_collection.cc:370-404
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetNewWithDefinition(
    count: u32,
    definition: &GridTrackSize,
    indefinite: bool,
) -> GridSet {
    let mut size = definition.clone();
    if size.IsFitContent() {
        if indefinite && size.FitContentTrackBreadth().HasPercent() {
            size = GridTrackSize::new_minmax(Length::Auto(), Length::MaxContent());
        }
    } else {
        let min = if (indefinite && size.MinTrackBreadth().HasPercent())
            || size.HasFlexMinTrackBreadth()
        {
            Length::Auto().clone()
        } else {
            size.MinTrackBreadth().clone()
        };
        let max = if indefinite && size.MaxTrackBreadth().HasPercent() {
            Length::Auto().clone()
        } else {
            size.MaxTrackBreadth().clone()
        };
        size = GridTrackSize::new_minmax(&min, &max);
    }
    let mut this = GridSet::new(count);
    this.track_size = size;
    this.fit_content_limit = kIndefiniteSize;
    this
}
// cpp: layoutng_grid/grid_track_collection.cc:406-409
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetFlexFactor(this: &GridSet) -> f32 {
    debug_assert!(this.track_size.HasFlexMaxTrackBreadth());
    this.track_size.MaxTrackBreadth().FlexValue() * this.track_count as f32
}
// cpp: layoutng_grid/grid_track_collection.cc:411-414
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetBaseSize(this: &GridSet) -> LayoutUnit {
    debug_assert!(!this.IsGrowthLimitLessThanBaseSize());
    this.base_size
}
// cpp: layoutng_grid/grid_track_collection.cc:416-419
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetGrowthLimit(this: &GridSet) -> LayoutUnit {
    debug_assert!(!this.IsGrowthLimitLessThanBaseSize());
    this.growth_limit
}
// cpp: layoutng_grid/grid_track_collection.cc:421-425
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetInitBaseSize(this: &mut GridSet, size: LayoutUnit) {
    debug_assert_ne!(size, kIndefiniteSize);
    this.base_size = size;
    this.EnsureGrowthLimitIsNotLessThanBaseSize();
}
// cpp: layoutng_grid/grid_track_collection.cc:427-433
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetIncreaseBaseSize(this: &mut GridSet, size: LayoutUnit) {
    debug_assert_ne!(size, kIndefiniteSize);
    debug_assert!(this.base_size <= size);
    this.base_size = size;
    this.EnsureGrowthLimitIsNotLessThanBaseSize();
}
// cpp: layoutng_grid/grid_track_collection.cc:435-442
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetIncreaseGrowthLimit(this: &mut GridSet, size: LayoutUnit) {
    debug_assert_ne!(size, kIndefiniteSize);
    debug_assert!(
        !this.IsGrowthLimitLessThanBaseSize()
            && (this.growth_limit == kIndefiniteSize || this.growth_limit <= size)
    );
    this.growth_limit = size;
}
// cpp: layoutng_grid/grid_track_collection.cc:444-447
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetEnsureGrowthLimitIsNotLessThanBaseSize(this: &mut GridSet) {
    if this.IsGrowthLimitLessThanBaseSize() {
        this.growth_limit = this.base_size;
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:449-451
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSetIsGrowthLimitLessThanBaseSize(this: &GridSet) -> bool {
    this.growth_limit != kIndefiniteSize && this.growth_limit < this.base_size
}

// cpp: layoutng_grid/grid_track_collection.cc:453-457
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionRangeStartLine(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> u32 {
    this.ranges_[i as usize].start_line
}
// cpp: layoutng_grid/grid_track_collection.cc:459-463
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionRangeTrackCount(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> u32 {
    this.ranges_[i as usize].track_count
}
// cpp: layoutng_grid/grid_track_collection.cc:465-469
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionRangeSetCount(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> u32 {
    this.ranges_[i as usize].set_count
}
// cpp: layoutng_grid/grid_track_collection.cc:471-475
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionRangeBeginSetIndex(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> u32 {
    this.ranges_[i as usize].begin_set_index
}
// cpp: layoutng_grid/grid_track_collection.cc:477-481
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionRangeProperties(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> TrackSpanProperties {
    this.ranges_[i as usize].properties
}
// cpp: layoutng_grid/grid_track_collection.cc:21-23
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionRangeEndLine(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> u32 {
    RangeEndLine(this, i)
}
// cpp: layoutng_grid/grid_track_collection.cc:25-48
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionRangeIndexFromGridLine(
    this: &GridLayoutTrackCollection,
    line: u32,
) -> u32 {
    RangeIndexFromGridLine(this, line)
}
// cpp: layoutng_grid/grid_track_collection.cc:483-488
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionEndLineOfImplicitGrid(
    this: &GridLayoutTrackCollection,
) -> u32 {
    this.ranges_
        .last()
        .map_or(0, |r| r.start_line.wrapping_add(r.track_count))
}
// cpp: layoutng_grid/grid_track_collection.cc:490-494
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionIsGridLineWithinImplicitGrid(
    this: &GridLayoutTrackCollection,
    line: u32,
) -> bool {
    debug_assert_ne!(line, u32::MAX);
    line <= this.EndLineOfImplicitGrid()
}
// cpp: layoutng_grid/grid_track_collection.cc:496-501
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionGetSetCount(this: &GridLayoutTrackCollection) -> u32 {
    this.ranges_
        .last()
        .map_or(0, |r| r.begin_set_index.wrapping_add(r.set_count))
}
// cpp: layoutng_grid/grid_track_collection.cc:503-506
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionGetSetOffset(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> LayoutUnit {
    this.sets_geometry_[i as usize].offset
}
// cpp: layoutng_grid/grid_track_collection.cc:508-512
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionGetSetTrackCount(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> u32 {
    this.sets_geometry_[i as usize + 1].track_count
}
// cpp: layoutng_grid/grid_track_collection.cc:514-518
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionStartExtraMargin(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> LayoutUnit {
    if i != 0 {
        this.accumulated_gutter_size_delta_ / 2
    } else {
        this.accumulated_start_extra_margin_
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:520-525
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionEndExtraMargin(
    this: &GridLayoutTrackCollection,
    i: u32,
) -> LayoutUnit {
    if (i as usize) < this.sets_geometry_.len() - 1 {
        this.accumulated_gutter_size_delta_ / 2
    } else {
        this.accumulated_end_extra_margin_
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:527-531
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionAdjustSingleSetOffset(
    this: &mut GridLayoutTrackCollection,
    i: u32,
    delta: LayoutUnit,
) {
    this.sets_geometry_[i as usize].offset += delta;
}
// cpp: layoutng_grid/grid_track_collection.cc:533-539
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionAdjustSetOffsets(
    this: &mut GridLayoutTrackCollection,
    i: u32,
    delta: LayoutUnit,
) {
    debug_assert!((i as usize) < this.sets_geometry_.len());
    for index in i as usize..this.sets_geometry_.len() {
        this.AdjustSingleSetOffset(index as u32, delta);
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:541-543
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionCalculateSetSpanSize(
    this: &GridLayoutTrackCollection,
) -> LayoutUnit {
    this.CalculateSetSpanSizeRange(0, this.GetSetCount())
}
// cpp: layoutng_grid/grid_track_collection.cc:545-563
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionCalculateSetSpanSizeRange(
    this: &GridLayoutTrackCollection,
    begin: u32,
    end: u32,
) -> LayoutUnit {
    debug_assert!(begin <= end);
    debug_assert!((end as usize) < this.sets_geometry_.len());
    if begin == end {
        return LayoutUnit::default();
    }
    if this.IsSpanningIndefiniteSet(begin, end) {
        return kIndefiniteSize;
    }
    (this.GetSetOffset(end) - this.gutter_size_ - this.GetSetOffset(begin)).ClampNegativeToZero()
}
// cpp: layoutng_grid/grid_track_collection.cc:565-579
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionIsSpanningIndefiniteSet(
    this: &GridLayoutTrackCollection,
    begin: u32,
    end: u32,
) -> bool {
    if this.last_indefinite_index_.is_empty() {
        return false;
    }
    debug_assert!(begin < end);
    let last = this.last_indefinite_index_[end as usize];
    last != u32::MAX && begin <= last
}

// cpp: layoutng_grid/grid_track_collection.cc:581-817
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionCreateSubgridTrackCollection(
    this: &GridLayoutTrackCollection,
    begin_range: u32,
    end_range: u32,
    gutter: LayoutUnit,
    margin: &BoxStrut,
    bsp: &BoxStrut,
    direction: GridTrackSizingDirection,
    opposite: bool,
    auto_placed: bool,
) -> *mut GridLayoutTrackCollection {
    debug_assert!(begin_range <= end_range);
    debug_assert!((end_range as usize) < this.ranges_.len());
    let child = MakeGarbageCollected(GridLayoutTrackCollection::new(direction));
    let sub = unsafe { &mut *child };
    let begin = this.ranges_[begin_range as usize].begin_set_index;
    let end = this.ranges_[end_range as usize].begin_set_index
        + this.ranges_[end_range as usize].set_count;
    debug_assert!((end as usize) < this.sets_geometry_.len());
    debug_assert!(begin < end);
    let range_count = end_range - begin_range;
    let mut current_set = 0;
    let mut current_line = 0;
    sub.ranges_.reserve(range_count as usize + 1);
    for i in 0..=range_count {
        let index = if opposite {
            end_range - i
        } else {
            begin_range + i
        };
        let mut range = this.ranges_[index as usize];
        range.begin_set_index = current_set;
        current_set += range.set_count;
        range.start_line = current_line;
        current_line += range.track_count;
        sub.properties_ |= &range.properties;
        sub.ranges_.push(range);
    }
    let span = end - begin;
    let gutter_delta = gutter - this.gutter_size_;
    let columns = direction == kForColumns;
    let margin_start = if columns {
        margin.inline_start
    } else {
        margin.block_start
    };
    let bsp_start = if columns {
        bsp.inline_start
    } else {
        bsp.block_start
    };
    let edge_start = margin_start + bsp_start;
    let edge_end = if columns {
        margin.inline_end + bsp.inline_end
    } else {
        margin.block_end + bsp.block_end
    };
    sub.accumulated_gutter_size_delta_ = gutter_delta + this.accumulated_gutter_size_delta_;
    sub.sets_geometry_.reserve(span as usize + 1);
    sub.sets_geometry_.push(SetGeometry::new(bsp_start, 0));
    let (parent_start, parent_end) = ExtraMarginsFromParent(this, sub, begin, end, auto_placed);
    sub.accumulated_start_extra_margin_ =
        edge_start + if opposite { parent_end } else { parent_start };
    sub.accumulated_end_extra_margin_ = edge_end + if opposite { parent_start } else { parent_end };
    let first = if opposite { end } else { begin };
    let mut first_offset = this.sets_geometry_[first as usize].offset;
    if opposite {
        first_offset -= margin_start;
    } else {
        first_offset += margin_start;
    }
    for i in 1..span {
        let index = if opposite { end - i } else { begin + i };
        let mut set = this.sets_geometry_[index as usize];
        if opposite {
            set.offset = first_offset - set.offset;
            set.track_count = this.sets_geometry_[index as usize + 1].track_count;
        } else {
            set.offset -= first_offset;
        }
        debug_assert!(set.track_count > 0);
        set.offset += gutter_delta / 2;
        sub.sets_geometry_.push(set);
    }
    let last = if opposite { begin } else { end };
    let mut set = this.sets_geometry_[last as usize];
    if opposite {
        set.offset = first_offset - set.offset;
        set.track_count = this.sets_geometry_[last as usize + 1].track_count;
    } else {
        set.offset -= first_offset;
    }
    set.offset += gutter_delta - edge_end;
    debug_assert!(set.track_count > 0);
    sub.sets_geometry_.push(set);
    if !this.last_indefinite_index_.is_empty() {
        sub.last_indefinite_index_.reserve(span as usize + 1);
        sub.last_indefinite_index_.push(u32::MAX);
        let mut last_indefinite = u32::MAX;
        for i in 0..span {
            let index = if opposite { end - i - 1 } else { begin + i } as usize;
            if this.last_indefinite_index_[index + 1] != this.last_indefinite_index_[index] {
                last_indefinite = i;
            }
            sub.last_indefinite_index_.push(last_indefinite);
        }
    }
    sub.gutter_size_ = gutter;
    child
}

// cpp: layoutng_grid/grid_track_collection.cc:657-721
fn ExtraMarginsFromParent(
    parent: &GridLayoutTrackCollection,
    sub: &GridLayoutTrackCollection,
    begin: u32,
    end: u32,
    auto_placed: bool,
) -> (LayoutUnit, LayoutUnit) {
    if !auto_placed {
        return (parent.StartExtraMargin(begin), parent.EndExtraMargin(end));
    }
    let mut start = parent.accumulated_start_extra_margin_;
    let mut finish = parent.accumulated_end_extra_margin_;
    let parent_count = parent.EndLineOfImplicitGrid();
    let child_count = sub.EndLineOfImplicitGrid();
    if parent_count == child_count {
        return (start, finish);
    }
    debug_assert!(child_count < parent_count);
    let half = parent.accumulated_gutter_size_delta_ / 2;
    if parent_count - child_count == 1 {
        if start >= finish {
            finish = half;
        } else {
            start = half;
        }
        return (start, finish);
    }
    (start.max(half), finish.max(half))
}

// cpp: layoutng_grid/grid_track_collection.cc:819-882
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionCreateSubgridBaselines(
    this: &GridLayoutTrackCollection,
    begin_range: u32,
    end_range: u32,
    gutter: LayoutUnit,
    margin: &BoxStrut,
    bsp: &BoxStrut,
    direction: GridTrackSizingDirection,
    opposite: bool,
    parent: &GridTrackBaselines,
) -> *mut GridTrackBaselines {
    debug_assert!(begin_range <= end_range);
    let begin = this.ranges_[begin_range as usize].begin_set_index;
    let end = this.ranges_[end_range as usize].begin_set_index
        + this.ranges_[end_range as usize].set_count;
    debug_assert!(begin < end);
    debug_assert!((end as usize) <= parent.major.len() && (end as usize) <= parent.minor.len());
    let span = end - begin;
    let delta = gutter - this.gutter_size_;
    let columns = direction == kForColumns;
    let edge_start = if columns {
        margin.inline_start + bsp.inline_start
    } else {
        margin.block_start + bsp.block_start
    };
    let edge_end = if columns {
        margin.inline_end + bsp.inline_end
    } else {
        margin.block_end + bsp.block_end
    };
    let child = MakeGarbageCollected(GridTrackBaselines::default());
    let sub = unsafe { &mut *child };
    sub.major.reserve(span as usize);
    sub.minor.reserve(span as usize);
    for i in 0..span {
        let mut major = if i == 0 { edge_start } else { delta / 2 };
        let mut minor = if i == span - 1 { edge_end } else { delta / 2 };
        if opposite {
            std::mem::swap(&mut major, &mut minor);
        }
        let index = if opposite { end - i - 1 } else { begin + i } as usize;
        sub.major.push(parent.major[index] - major);
        sub.minor.push(parent.minor[index] - minor);
    }
    if opposite {
        std::mem::swap(&mut sub.major, &mut sub.minor);
    }
    child
}

// cpp: layoutng_grid/grid_track_collection.cc:884-886
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionHasFlexibleTrack(
    this: &GridLayoutTrackCollection,
) -> bool {
    this.properties_.HasProperty(PropertyId::kHasFlexibleTrack)
}
// cpp: layoutng_grid/grid_track_collection.cc:888-890
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionHasIntrinsicTrack(
    this: &GridLayoutTrackCollection,
) -> bool {
    this.properties_.HasProperty(PropertyId::kHasIntrinsicTrack)
}
// cpp: layoutng_grid/grid_track_collection.cc:892-894
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionHasNonDefiniteTrack(
    this: &GridLayoutTrackCollection,
) -> bool {
    this.properties_
        .HasProperty(PropertyId::kHasNonDefiniteTrack)
}
// cpp: layoutng_grid/grid_track_collection.cc:896-899
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionIsDependentOnAvailableSize(
    this: &GridLayoutTrackCollection,
) -> bool {
    this.properties_
        .HasProperty(PropertyId::kIsDependentOnAvailableSize)
}
// cpp: layoutng_grid/grid_track_collection.cc:901-911
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionFirstNonCollapsedLineIndex(
    this: &GridLayoutTrackCollection,
) -> u32 {
    this.ranges_.first().map_or(u32::MAX, |r| {
        if r.IsCollapsed() {
            r.start_line + r.track_count
        } else {
            r.start_line
        }
    })
}
// cpp: layoutng_grid/grid_track_collection.cc:913-916
#[unsafe(no_mangle)]
pub extern "Rust" fn GridLayoutTrackCollectionHasIndefiniteSet(
    this: &GridLayoutTrackCollection,
) -> bool {
    this.last_indefinite_index_
        .last()
        .is_some_and(|&i| i != u32::MAX)
}

// cpp: layoutng_grid/grid_track_collection.cc:918-945
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionNew(
    ranges: GridRangeVector,
    direction: GridTrackSizingDirection,
    store_collapsed: bool,
) -> GridSizingTrackCollection {
    let mut base = GridLayoutTrackCollection::new(direction);
    base.ranges_ = ranges;
    if store_collapsed {
        base.collapsed_track_indexes_.clear();
    }
    let mut count = 0;
    let mut tracks = 0;
    for range in &base.ranges_ {
        if !range.IsCollapsed() {
            tracks += range.track_count;
            count += range.set_count;
        } else if store_collapsed {
            for i in range.start_line..range.start_line + range.track_count {
                base.collapsed_track_indexes_.push(i);
            }
        }
    }
    base.last_indefinite_index_.reserve(count as usize + 1);
    base.sets_geometry_.reserve(count as usize + 1);
    GridSizingTrackCollection {
        base,
        non_collapsed_track_count_: tracks,
        sets_: Vec::with_capacity(count as usize),
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:947-950
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionGetSetAt(
    this: &mut GridSizingTrackCollection,
    i: u32,
) -> *mut GridSet {
    &mut this.sets_[i as usize]
}
// cpp: layoutng_grid/grid_track_collection.cc:952-955
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionGetSetAtConst(
    this: &GridSizingTrackCollection,
    i: u32,
) -> *const GridSet {
    &this.sets_[i as usize]
}
// cpp: layoutng_grid/grid_track_collection.cc:958-960
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionGetSetIterator(
    this: &mut GridSizingTrackCollection,
) -> SetIterator {
    SetIterator::new(this, 0, this.sets_.len() as u32)
}
// cpp: layoutng_grid/grid_track_collection.cc:963-965
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionGetConstSetIterator(
    this: &GridSizingTrackCollection,
) -> ConstSetIterator {
    ConstSetIterator::new(
        (this as *const GridSizingTrackCollection).cast_mut(),
        0,
        this.sets_.len() as u32,
    )
}
// cpp: layoutng_grid/grid_track_collection.cc:968-971
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionGetSetIteratorRange(
    this: &mut GridSizingTrackCollection,
    begin: u32,
    end: u32,
) -> SetIterator {
    SetIterator::new(this, begin, end)
}
// cpp: layoutng_grid/grid_track_collection.cc:973-981
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionTotalTrackSize(
    this: &GridSizingTrackCollection,
) -> LayoutUnit {
    if this.sets_.is_empty() {
        return LayoutUnit::default();
    }
    let mut total = LayoutUnit::default();
    for set in &this.sets_ {
        total += set.BaseSize() + this.base.gutter_size_ * set.track_count;
    }
    total - this.base.gutter_size_
}
// cpp: layoutng_grid/grid_track_collection.cc:983-1001
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionCacheDefiniteSetsGeometry(
    this: &mut GridSizingTrackCollection,
) {
    debug_assert!(
        this.base.sets_geometry_.is_empty() && this.base.last_indefinite_index_.is_empty()
    );
    let mut offset = LayoutUnit::default();
    this.base.last_indefinite_index_.push(u32::MAX);
    this.base.sets_geometry_.push(SetGeometry::new(offset, 0));
    for set in &this.sets_ {
        if set.track_size.IsDefinite() {
            offset += set.base_size + this.base.gutter_size_ * set.track_count;
            this.base
                .last_indefinite_index_
                .push(*this.base.last_indefinite_index_.last().unwrap());
        } else {
            this.base
                .last_indefinite_index_
                .push(this.base.last_indefinite_index_.len() as u32 - 1);
        }
        debug_assert!(this.base.sets_geometry_.last().unwrap().offset <= offset);
        this.base
            .sets_geometry_
            .push(SetGeometry::new(offset, set.track_count));
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:1003-1022
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionCacheInitializedSetsGeometry(
    this: &mut GridSizingTrackCollection,
    mut offset: LayoutUnit,
) {
    this.base.last_indefinite_index_.clear();
    this.base.sets_geometry_.clear();
    this.base.last_indefinite_index_.push(u32::MAX);
    this.base.sets_geometry_.push(SetGeometry::new(offset, 0));
    for set in &this.sets_ {
        if set.growth_limit == kIndefiniteSize {
            this.base
                .last_indefinite_index_
                .push(this.base.last_indefinite_index_.len() as u32 - 1);
        } else {
            offset += set.growth_limit + this.base.gutter_size_ * set.track_count;
            this.base
                .last_indefinite_index_
                .push(*this.base.last_indefinite_index_.last().unwrap());
        }
        debug_assert!(this.base.sets_geometry_.last().unwrap().offset <= offset);
        this.base
            .sets_geometry_
            .push(SetGeometry::new(offset, set.track_count));
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:1024-1039
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionFinalizeSetsGeometry(
    this: &mut GridSizingTrackCollection,
    mut offset: LayoutUnit,
    gutter: LayoutUnit,
) {
    this.base.gutter_size_ = gutter;
    this.base.last_indefinite_index_.clear();
    this.base.sets_geometry_.clear();
    this.base.sets_geometry_.push(SetGeometry::new(offset, 0));
    for set in &this.sets_ {
        offset += set.BaseSize() + gutter * set.track_count;
        debug_assert!(this.base.sets_geometry_.last().unwrap().offset <= offset);
        this.base
            .sets_geometry_
            .push(SetGeometry::new(offset, set.track_count));
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:1041-1046
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionSetIndefiniteGrowthLimitsToBaseSize(
    this: &mut GridSizingTrackCollection,
) {
    for set in &mut this.sets_ {
        if set.GrowthLimit() == kIndefiniteSize {
            set.growth_limit = set.base_size;
        }
    }
}

// cpp: layoutng_grid/grid_track_collection.cc:1050-1157
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionBuildSetsFromLists(
    this: &mut GridSizingTrackCollection,
    explicit: &GridTrackList,
    implicit: &GridTrackList,
    _is_grid_lanes: bool,
    indefinite: bool,
) {
    this.base.properties_.ResetType();
    this.sets_.clear();
    for range in &mut this.base.ranges_ {
        range.properties.ResetType();
        if range.IsCollapsed() {
            continue;
        }
        if range.repeater_index == u32::MAX {
            debug_assert!(range.IsImplicit() || explicit.IsSubgriddedAxis());
            let set = GridSet::new(range.track_count);
            CacheSetProperties(range, &set);
            this.sets_.push(set);
        } else {
            let specified = if range.IsImplicit() {
                implicit
            } else {
                explicit
            };
            let repeater_size = specified.RepeatSize(range.repeater_index);
            debug_assert!(range.repeater_offset < repeater_size);
            let floor = range.track_count / repeater_size;
            let remaining = range.track_count % repeater_size;
            for i in 0..range.set_count {
                let count = floor + u32::from(i < remaining);
                let offset = (range.repeater_offset + i) % repeater_size;
                let size = specified.RepeatTrackSize(range.repeater_index, offset);
                if size.HasPercentage() {
                    range
                        .properties
                        .SetProperty(PropertyId::kIsDependentOnAvailableSize);
                }
                let set = GridSet::new_with_definition(count, size, indefinite);
                CacheSetProperties(range, &set);
                this.sets_.push(set);
            }
        }
        this.base.properties_ |= &range.properties;
    }
}
// cpp: layoutng_grid/grid_track_collection.cc:1067-1102
fn CacheSetProperties(range: &mut GridRange, set: &GridSet) {
    let size = &set.track_size;
    debug_assert!(!size.HasFlexMinTrackBreadth());
    if size.HasAutoMinTrackBreadth() {
        range
            .properties
            .SetProperty(PropertyId::kHasAutoMinimumTrack);
    }
    if size.HasFixedMinTrackBreadth() {
        range
            .properties
            .SetProperty(PropertyId::kHasFixedMinimumTrack);
    }
    if size.HasFixedMaxTrackBreadth() {
        range
            .properties
            .SetProperty(PropertyId::kHasFixedMaximumTrack);
    }
    if size.HasFlexMaxTrackBreadth() {
        range.properties.SetProperty(PropertyId::kHasFlexibleTrack);
        range
            .properties
            .SetProperty(PropertyId::kIsDependentOnAvailableSize);
    }
    if size.HasIntrinsicMinTrackBreadth() || size.HasIntrinsicMaxTrackBreadth() {
        range.properties.SetProperty(PropertyId::kHasIntrinsicTrack);
    }
    if !size.IsDefinite() {
        range
            .properties
            .SetProperty(PropertyId::kHasNonDefiniteTrack);
    }
}

// cpp: layoutng_grid/grid_track_collection.cc:1160-1205
#[unsafe(no_mangle)]
pub extern "Rust" fn GridSizingTrackCollectionInitializeSets(
    this: &mut GridSizingTrackCollection,
    available: LayoutUnit,
) {
    for set in &mut this.sets_ {
        let size = &set.track_size;
        if size.IsFitContent() {
            debug_assert!(
                !size.FitContentTrackBreadth().HasPercent() || available != kIndefiniteSize
            );
            set.fit_content_limit =
                MinimumValueForLength(size.FitContentTrackBreadth(), available) * set.track_count;
        }
        if size.HasFixedMaxTrackBreadth() {
            debug_assert!(!size.MaxTrackBreadth().HasPercent() || available != kIndefiniteSize);
            set.growth_limit =
                MinimumValueForLength(size.MaxTrackBreadth(), available) * set.track_count;
        } else {
            set.growth_limit = kIndefiniteSize;
        }
        let initial = if size.HasFixedMinTrackBreadth() {
            debug_assert!(!size.MinTrackBreadth().HasPercent() || available != kIndefiniteSize);
            MinimumValueForLength(size.MinTrackBreadth(), available) * set.track_count
        } else {
            debug_assert!(size.HasIntrinsicMinTrackBreadth());
            LayoutUnit::default()
        };
        set.InitBaseSize(initial);
    }
}

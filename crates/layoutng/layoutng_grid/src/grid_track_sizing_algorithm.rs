use foundation::{kIndefiniteSize, LayoutUnit, MinimumValueForLength};
use layoutng_assembly::internal::{
    grid_item::{GridItemData, GridItems},
    grid_track_collection::{
        GridLayoutTrackCollection, GridSet, GridSizingTrackCollection, PropertyId, SetIterator,
    },
};
use layoutng_geometry::geometry::{box_strut::BoxStrut, logical_size::LogicalSize};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    computed_style_constants::{ContentDistributionType, ContentPosition, OverflowAlignment},
    grid_enums::GridTrackSizingDirection::{self, *},
    style_content_alignment_data::StyleContentAlignmentData,
};

// cpp: layoutng_grid/grid_track_sizing_algorithm.h:25-38
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GridItemContributionType {
    kForIntrinsicMinimums,
    kForContentBasedMinimums,
    kForMaxContentMinimums,
    kForIntrinsicMaximums,
    kForMaxContentMaximums,
    kForFreeSpace,
}
use GridItemContributionType::*;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SizingConstraint {
    kLayout,
    kMaxContent,
    kMinContent,
}
pub type ContributionSizeFunctionRef<'a> =
    dyn FnMut(GridItemContributionType, &mut GridItemData) -> LayoutUnit + 'a;
// cpp: layoutng_grid/grid_track_sizing_algorithm.h:44-47
pub struct FirstSetGeometry {
    pub gutter_size: LayoutUnit,
    pub start_offset: LayoutUnit,
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.h:39-130
pub struct GridTrackSizingAlgorithm {
    available_size_: LogicalSize,
    min_available_size_: LogicalSize,
    sizing_constraint_: SizingConstraint,
    columns_alignment_: StyleContentAlignmentData,
    rows_alignment_: StyleContentAlignmentData,
}
impl GridTrackSizingAlgorithm {
    // cpp: layoutng_grid/grid_track_sizing_algorithm.h:49-57
    pub fn new(
        style: &ComputedStyle,
        available: &LogicalSize,
        min: &LogicalSize,
        constraint: SizingConstraint,
    ) -> Self {
        Self {
            available_size_: *available,
            min_available_size_: *min,
            sizing_constraint_: constraint,
            columns_alignment_: *style.JustifyContent(),
            rows_alignment_: *style.AlignContent(),
        }
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:27-133
    pub fn CacheGridItemsProperties(tracks: &GridLayoutTrackCollection, items: &mut GridItems) {
        let mut multiple: Vec<*mut GridItemData> = Vec::new();
        let direction = tracks.Direction();
        let range = items.IncludeSubgriddedItems();
        let mut it = range.begin();
        let end = range.end();
        while it.NotEqual(&end) {
            let item = unsafe { &mut *it.Get() };
            it.Advance();
            if !item.MustCachePlacementIndices(direction) {
                continue;
            }
            item.ComputeSetIndices(tracks);
            let indices = *item.RangeIndices(direction);
            let properties = if direction == kForColumns {
                &mut item.column_span_properties
            } else {
                &mut item.row_span_properties
            };
            properties.ResetType();
            if indices.begin == indices.end {
                *properties = tracks.RangeProperties(indices.begin);
            } else {
                multiple.push(item);
            }
        }
        if multiple.is_empty() {
            return;
        }
        multiple.sort_unstable_by_key(|p| unsafe { &**p }.StartLine(direction));
        for property in [
            PropertyId::kHasFlexibleTrack,
            PropertyId::kHasIntrinsicTrack,
            PropertyId::kHasAutoMinimumTrack,
            PropertyId::kHasFixedMinimumTrack,
            PropertyId::kHasFixedMaximumTrack,
        ] {
            let mut index = 0;
            let count = tracks.RangeCount();
            for pointer in &multiple {
                let item = unsafe { &mut **pointer };
                while index < count
                    && (tracks.RangeEndLine(index) <= item.StartLine(direction)
                        || !tracks.RangeProperties(index).HasProperty(property))
                {
                    index += 1;
                }
                if index == count {
                    break;
                }
                if tracks.RangeEndLine(index) <= item.EndLine(direction) {
                    item.SetTrackSpanProperty(property, direction);
                }
            }
        }
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:136-156
    pub fn CacheSubgridItemsProperties(
        tracks: &GridLayoutTrackCollection,
        items: &mut GridItems,
        direction: GridTrackSizingDirection,
    ) {
        let range = items.IncludeSubgriddedItems();
        let mut it = range.begin();
        let end = range.end();
        while it.NotEqual(&end) {
            let item = unsafe { &mut *it.Get() };
            it.Advance();
            if !item.IsSubgrid() {
                continue;
            }
            let indices = *item.RangeIndices(direction);
            let properties = if direction == kForColumns {
                &mut item.column_span_properties
            } else {
                &mut item.row_span_properties
            };
            properties.ResetType();
            for i in indices.begin..=indices.end {
                *properties |= &tracks.RangeProperties(i);
            }
        }
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.h:72-77
    pub fn CalculateGutterSize(
        style: &ComputedStyle,
        available: &LogicalSize,
        direction: GridTrackSizingDirection,
    ) -> LayoutUnit {
        Self::CalculateGutterSizeWithParent(style, available, direction, LayoutUnit::default())
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:159-179
    pub fn CalculateGutterSizeWithParent(
        style: &ComputedStyle,
        available: &LogicalSize,
        direction: GridTrackSizingDirection,
        parent: LayoutUnit,
    ) -> LayoutUnit {
        let columns = direction == kForColumns;
        let gutter = if columns {
            style.ColumnGap()
        } else {
            style.RowGap()
        };
        gutter.as_ref().map_or(parent, |g| {
            MinimumValueForLength(
                g,
                if columns {
                    available.inline_size
                } else {
                    available.block_size
                }
                .ClampIndefiniteToZero(),
            )
        })
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:183-306
    pub fn ComputeFirstSetGeometry(
        tracks: &GridSizingTrackCollection,
        style: &ComputedStyle,
        available: &LogicalSize,
        bsp: &BoxStrut,
    ) -> FirstSetGeometry {
        let columns = tracks.base.Direction() == kForColumns;
        let size = if columns {
            available.inline_size
        } else {
            available.block_size
        };
        let mut result = FirstSetGeometry {
            gutter_size: tracks.base.GutterSize(),
            start_offset: if columns {
                bsp.inline_start
            } else {
                bsp.block_start
            },
        };
        if size == kIndefiniteSize {
            return result;
        }
        let alignment = if columns {
            style.JustifyContent()
        } else {
            style.AlignContent()
        };
        let free = || {
            let free = size - tracks.TotalTrackSize();
            if alignment.Overflow() == OverflowAlignment::kSafe {
                free.ClampNegativeToZero()
            } else {
                free
            }
        };
        let count = tracks.NonCollapsedTrackCount();
        match alignment.Distribution() {
            ContentDistributionType::kSpaceBetween => {
                let space = free();
                if count < 2 || space < LayoutUnit::default() {
                    return result;
                }
                result.gutter_size += space / (count - 1);
                return result;
            }
            ContentDistributionType::kSpaceAround => {
                let space = free();
                if space < LayoutUnit::default() {
                    return result;
                }
                if count < 1 {
                    result.start_offset += space / 2;
                    return result;
                }
                let space = space / count;
                result.start_offset += space / 2;
                result.gutter_size += space;
                return result;
            }
            ContentDistributionType::kSpaceEvenly => {
                let space = free();
                if space < LayoutUnit::default() {
                    return result;
                }
                let space = space / (count + 1);
                result.start_offset += space;
                result.gutter_size += space;
                return result;
            }
            ContentDistributionType::kStretch | ContentDistributionType::kDefault => {}
        }
        match alignment.GetPosition() {
            ContentPosition::kLeft => {
                debug_assert!(columns);
                if foundation::IsLtr(style.Direction()) {
                    return result;
                }
                result.start_offset += free();
            }
            ContentPosition::kRight => {
                debug_assert!(columns);
                if style.Direction() == foundation::TextDirection::kRtl {
                    return result;
                }
                result.start_offset += free();
            }
            ContentPosition::kCenter => result.start_offset += free() / 2,
            ContentPosition::kEnd | ContentPosition::kFlexEnd => result.start_offset += free(),
            _ => {}
        }
        result
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:308-350
    pub fn ComputeUsedTrackSizes(
        &self,
        contribution: &mut ContributionSizeFunctionRef<'_>,
        tracks: &mut GridSizingTrackCollection,
        items: &mut GridItems,
        intrinsic: bool,
    ) {
        if tracks.base.HasIntrinsicTrack() {
            self.ResolveIntrinsicTrackSizes(contribution, tracks, items);
        }
        if intrinsic {
            return;
        }
        tracks.SetIndefiniteGrowthLimitsToBaseSize();
        self.MaximizeTracks(tracks);
        if tracks.base.HasFlexibleTrack() {
            self.ExpandFlexibleTracks(contribution, tracks, items);
        }
        self.StretchAutoTracks(tracks);
    }
}

// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:355-359
fn DefiniteGrowthLimit(set: &GridSet) -> LayoutUnit {
    let limit = set.GrowthLimit();
    if limit == kIndefiniteSize {
        set.BaseSize()
    } else {
        limit
    }
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:364-378
fn AffectedSizeForContribution(set: &GridSet, kind: GridItemContributionType) -> LayoutUnit {
    match kind {
        kForIntrinsicMinimums | kForContentBasedMinimums | kForMaxContentMinimums => set.BaseSize(),
        kForIntrinsicMaximums | kForMaxContentMaximums => DefiniteGrowthLimit(set),
        kForFreeSpace => unreachable!(),
    }
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:380-411
fn GrowAffectedSizeByPlannedIncrease(kind: GridItemContributionType, set: &mut GridSet) {
    set.is_infinitely_growable.write(false);
    let increase = set.planned_increase;
    if increase == kIndefiniteSize {
        return;
    }
    match kind {
        kForIntrinsicMinimums | kForContentBasedMinimums | kForMaxContentMinimums => {
            set.IncreaseBaseSize(set.BaseSize() + increase)
        }
        kForIntrinsicMaximums => {
            set.is_infinitely_growable
                .write(set.GrowthLimit() == kIndefiniteSize);
            set.IncreaseGrowthLimit(DefiniteGrowthLimit(set) + increase);
        }
        kForMaxContentMaximums => set.IncreaseGrowthLimit(DefiniteGrowthLimit(set) + increase),
        kForFreeSpace => unreachable!(),
    }
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:415-434
fn IsContributionAppliedToSet(set: &GridSet, kind: GridItemContributionType) -> bool {
    match kind {
        kForIntrinsicMinimums => set.track_size.HasIntrinsicMinTrackBreadth(),
        kForContentBasedMinimums => set.track_size.HasMinOrMaxContentMinTrackBreadth(),
        kForMaxContentMinimums => set.track_size.HasMaxContentMinTrackBreadth(),
        kForIntrinsicMaximums => set.track_size.HasIntrinsicMaxTrackBreadth(),
        kForMaxContentMaximums => set.track_size.HasMaxContentOrAutoMaxTrackBreadth(),
        kForFreeSpace => true,
    }
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:441-454
fn ShouldUsedSizeGrowBeyondLimit(set: &GridSet, kind: GridItemContributionType) -> bool {
    match kind {
        kForIntrinsicMinimums | kForContentBasedMinimums => {
            set.track_size.HasIntrinsicMaxTrackBreadth()
        }
        kForMaxContentMinimums => set.track_size.HasMaxContentOrAutoMaxTrackBreadth(),
        _ => false,
    }
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:456-467
fn IsDistributionForGrowthLimits(kind: GridItemContributionType) -> bool {
    matches!(kind, kForIntrinsicMaximums | kForMaxContentMaximums)
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:473-473
#[derive(Clone, Copy, PartialEq, Eq)]
enum InfinitelyGrowableBehavior {
    kEnforce,
    kIgnore,
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:474-525
fn GrowthPotentialForSet(
    set: &GridSet,
    kind: GridItemContributionType,
    behavior: InfinitelyGrowableBehavior,
) -> LayoutUnit {
    match kind {
        kForIntrinsicMinimums | kForContentBasedMinimums | kForMaxContentMinimums => {
            let limit = set.GrowthLimit();
            if limit == kIndefiniteSize {
                return kIndefiniteSize;
            }
            let increased = set.BaseSize() + set.item_incurred_increase;
            debug_assert!(increased <= limit);
            limit - increased
        }
        kForIntrinsicMaximums | kForMaxContentMaximums => {
            if behavior == InfinitelyGrowableBehavior::kEnforce
                && set.GrowthLimit() != kIndefiniteSize
                && !unsafe { set.is_infinitely_growable.assume_init() }
            {
                return LayoutUnit::default();
            }
            debug_assert!(
                set.fit_content_limit >= LayoutUnit::default()
                    || set.fit_content_limit == kIndefiniteSize
            );
            if set.fit_content_limit != kIndefiniteSize {
                return (set.fit_content_limit
                    - DefiniteGrowthLimit(set)
                    - set.item_incurred_increase)
                    .ClampNegativeToZero();
            }
            kIndefiniteSize
        }
        kForFreeSpace => {
            let limit = set.GrowthLimit();
            debug_assert_ne!(limit, kIndefiniteSize);
            limit - set.BaseSize()
        }
    }
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:537-545
fn AreEqualFloat(a: f32, b: f32) -> bool {
    (a - b).abs() < f32::EPSILON
}

// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:547-774
// Count and float ratios keep the source conditional template's arithmetic
// types, including unsigned multiplication in the equal-distribution branch.
#[derive(Clone, Copy)]
enum ShareRatio {
    Count(u32),
    Flex(f32),
}
impl ShareRatio {
    fn equal(self, other: Self) -> bool {
        match (self, other) {
            (Self::Count(a), Self::Count(b)) => a == b,
            (Self::Flex(a), Self::Flex(b)) => AreEqualFloat(a, b),
            _ => unreachable!(),
        }
    }
    fn greater(self, other: Self) -> bool {
        match (self, other) {
            (Self::Count(a), Self::Count(b)) => a > b,
            (Self::Flex(a), Self::Flex(b)) => a > b,
            _ => unreachable!(),
        }
    }
    fn sub(&mut self, other: Self) {
        match (self, other) {
            (Self::Count(a), Self::Count(b)) => *a = a.wrapping_sub(b),
            (Self::Flex(a), Self::Flex(b)) => *a -= b,
            _ => unreachable!(),
        }
    }
    fn share(self, space: LayoutUnit, total: Self) -> LayoutUnit {
        let raw = match (self, total) {
            (Self::Count(a), Self::Count(b)) => {
                (space.RawValue() as u32).wrapping_mul(a).wrapping_div(b) as i32
            }
            (Self::Flex(a), Self::Flex(b)) => ((space.RawValue() as f32 * a) / b) as i32,
            _ => unreachable!(),
        };
        LayoutUnit::FromRawValue(raw)
    }
}
fn DistributeExtraSpaceToSets<const EQUAL: bool>(
    mut extra: LayoutUnit,
    flex: f32,
    kind: GridItemContributionType,
    sets: &mut Vec<*mut GridSet>,
    beyond: *mut Vec<*mut GridSet>,
) {
    let enforce = InfinitelyGrowableBehavior::kEnforce;
    let ignore = InfinitelyGrowableBehavior::kIgnore;
    if extra == kIndefiniteSize {
        debug_assert_eq!(kind, kForFreeSpace);
        for pointer in sets {
            let set = unsafe { &mut **pointer };
            set.item_incurred_increase = GrowthPotentialForSet(set, kind, enforce);
        }
        return;
    }
    debug_assert!(extra > LayoutUnit::default());
    if IsDistributionForGrowthLimits(kind) {
        debug_assert_eq!(sets as *mut _, beyond);
    }
    let mut growable = 0u32;
    for pointer in sets.iter() {
        let set = unsafe { &mut **pointer };
        set.item_incurred_increase = LayoutUnit::default();
        if GrowthPotentialForSet(set, kind, enforce) != LayoutUnit::default() {
            growable = growable.wrapping_add(set.track_count);
        }
    }
    debug_assert!(EQUAL || !AreEqualFloat(flex, 0.0));
    let mut sum = if EQUAL {
        ShareRatio::Count(growable)
    } else {
        ShareRatio::Flex(flex)
    };
    let overflowing = match sum {
        ShareRatio::Count(x) => x >= u32::MAX,
        ShareRatio::Flex(x) => x >= u32::MAX as f32,
    };
    if growable != 0 || IsDistributionForGrowthLimits(kind) {
        if AreEqualFloat(flex, 0.0) {
            debug_assert!(EQUAL);
            sets.sort_unstable_by(|a, b| {
                let a = GrowthPotentialForSet(unsafe { &**a }, kind, ignore);
                let b = GrowthPotentialForSet(unsafe { &**b }, kind, ignore);
                match (a == kIndefiniteSize, b == kIndefiniteSize) {
                    (true, true) => std::cmp::Ordering::Equal,
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => a.cmp(&b),
                }
            });
        }
    }
    for pointer in sets.iter() {
        if growable == 0 {
            break;
        }
        let set = unsafe { &mut **pointer };
        let potential = GrowthPotentialForSet(set, kind, enforce);
        set.item_incurred_increase = ExtraSpaceShare::<EQUAL>(
            set,
            potential,
            &mut extra,
            &mut growable,
            &mut sum,
            overflowing,
        );
    }
    if !beyond.is_null() && extra != LayoutUnit::default() {
        let beyond = unsafe { &mut *beyond };
        #[cfg(debug_assertions)]
        {
            let mut previous = LayoutUnit::default();
            for pointer in beyond.iter() {
                let potential = GrowthPotentialForSet(unsafe { &**pointer }, kind, ignore);
                if potential != LayoutUnit::default() {
                    if previous == kIndefiniteSize {
                        debug_assert_eq!(potential, kIndefiniteSize);
                    } else {
                        debug_assert!(potential >= previous || potential == kIndefiniteSize);
                    }
                    previous = potential;
                }
            }
        }
        let potential = |set: &GridSet| {
            if !IsDistributionForGrowthLimits(kind) {
                kIndefiniteSize
            } else {
                GrowthPotentialForSet(set, kind, ignore)
            }
        };
        debug_assert_eq!(growable, 0);
        for pointer in beyond.iter() {
            let set = unsafe { &**pointer };
            if potential(set) != LayoutUnit::default() {
                growable = growable.wrapping_add(set.track_count);
            }
        }
        debug_assert!(EQUAL);
        sum = ShareRatio::Count(growable);
        for pointer in beyond.iter() {
            if growable == 0 {
                break;
            }
            let set = unsafe { &mut **pointer };
            let share = ExtraSpaceShare::<EQUAL>(
                set,
                potential(set),
                &mut extra,
                &mut growable,
                &mut sum,
                overflowing,
            );
            set.item_incurred_increase += share;
        }
    }
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:626-677
fn ExtraSpaceShare<const EQUAL: bool>(
    set: &GridSet,
    potential: LayoutUnit,
    extra: &mut LayoutUnit,
    growable: &mut u32,
    sum: &mut ShareRatio,
    overflowing: bool,
) -> LayoutUnit {
    debug_assert!(potential >= LayoutUnit::default() || potential == kIndefiniteSize);
    if potential == LayoutUnit::default() {
        return LayoutUnit::default();
    }
    let mut count = set.track_count;
    debug_assert!(count <= *growable);
    let mut ratio = if EQUAL {
        ShareRatio::Count(count)
    } else {
        ShareRatio::Flex(set.FlexFactor())
    };
    if ratio.greater(*sum) {
        debug_assert!(overflowing);
        ratio = *sum;
    }
    let mut share = if ratio.equal(*sum) {
        count = *growable;
        *extra
    } else {
        debug_assert!(!sum.equal(if EQUAL {
            ShareRatio::Count(0)
        } else {
            ShareRatio::Flex(0.0)
        }));
        ratio.share(*extra, *sum)
    };
    if potential != kIndefiniteSize {
        share = share.min(potential);
    }
    debug_assert!(share <= *extra);
    *growable = growable.wrapping_sub(count);
    sum.sub(ratio);
    *extra -= share;
    share
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:779-787
fn DistributeExtraSpaceToSetsEqually(
    extra: LayoutUnit,
    kind: GridItemContributionType,
    sets: &mut Vec<*mut GridSet>,
    beyond: *mut Vec<*mut GridSet>,
) {
    DistributeExtraSpaceToSets::<true>(extra, 0.0, kind, sets, beyond);
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:789-797
fn DistributeExtraSpaceToWeightedSets(
    extra: LayoutUnit,
    flex: f32,
    kind: GridItemContributionType,
    sets: &mut Vec<*mut GridSet>,
) {
    DistributeExtraSpaceToSets::<false>(extra, flex, kind, sets, std::ptr::null_mut());
}

impl GridTrackSizingAlgorithm {
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:801-911
    fn IncreaseTrackSizesToAccommodateGridItems(
        &self,
        contribution: &mut ContributionSizeFunctionRef<'_>,
        group: &[*mut GridItemData],
        kind: GridItemContributionType,
        spanning_flex: bool,
        tracks: &mut GridSizingTrackCollection,
    ) {
        let mut it = tracks.GetSetIterator();
        while !it.IsAtEnd() {
            unsafe { &mut *it.CurrentSet() }.planned_increase = kIndefiniteSize;
            it.MoveToNextSet();
        }
        let mut sets = Vec::new();
        let mut beyond = Vec::new();
        let direction = tracks.base.Direction();
        for pointer in group {
            let item = unsafe { &mut **pointer };
            debug_assert!(item.IsSpanningIntrinsicTrack(direction));
            sets.clear();
            beyond.clear();
            let mut flex_sum = 0f32;
            let mut spanned = tracks.base.GutterSize() * item.SpanSize(direction).wrapping_sub(1);
            let mut it = item.SetIterator(tracks);
            while !it.IsAtEnd() {
                let set = unsafe { &mut *it.CurrentSet() };
                it.MoveToNextSet();
                spanned += AffectedSizeForContribution(set, kind);
                if spanning_flex && !set.track_size.HasFlexMaxTrackBreadth() {
                    continue;
                }
                if IsContributionAppliedToSet(set, kind) {
                    if set.planned_increase == kIndefiniteSize {
                        set.planned_increase = LayoutUnit::default();
                    }
                    if spanning_flex {
                        flex_sum = SaturateFloat(flex_sum + set.FlexFactor());
                    }
                    sets.push(set as *mut GridSet);
                    if ShouldUsedSizeGrowBeyondLimit(set, kind) {
                        beyond.push(set as *mut GridSet);
                    }
                }
            }
            if sets.is_empty() {
                continue;
            }
            let extra = (contribution(kind, item) - spanned).ClampNegativeToZero();
            if extra == LayoutUnit::default() {
                continue;
            }
            if !spanning_flex || AreEqualFloat(flex_sum, 0.0) {
                let ptr = if beyond.is_empty() {
                    &mut sets as *mut _
                } else {
                    &mut beyond as *mut _
                };
                DistributeExtraSpaceToSetsEqually(extra, kind, &mut sets, ptr);
            } else {
                debug_assert!(beyond.is_empty());
                DistributeExtraSpaceToWeightedSets(extra, flex_sum, kind, &mut sets);
            }
            for pointer in &sets {
                let set = unsafe { &mut **pointer };
                debug_assert_ne!(set.item_incurred_increase, kIndefiniteSize);
                debug_assert_ne!(set.planned_increase, kIndefiniteSize);
                set.planned_increase = set.item_incurred_increase.max(set.planned_increase);
            }
        }
        let mut it = tracks.GetSetIterator();
        while !it.IsAtEnd() {
            GrowAffectedSizeByPlannedIncrease(kind, unsafe { &mut *it.CurrentSet() });
            it.MoveToNextSet();
        }
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:914-1024
    fn ResolveIntrinsicTrackSizes(
        &self,
        contribution: &mut ContributionSizeFunctionRef<'_>,
        tracks: &mut GridSizingTrackCollection,
        items: &mut GridItems,
    ) {
        let direction = tracks.base.Direction();
        let mut reordered: Vec<*mut GridItemData> = Vec::with_capacity(items.Size() as usize);
        let range = items.IncludeSubgriddedItems();
        let mut it = range.begin();
        let end = range.end();
        while it.NotEqual(&end) {
            let item = unsafe { &mut *it.Get() };
            it.Advance();
            if item.IsSpanningIntrinsicTrack(direction) && item.IsConsideredForSizing(direction) {
                reordered.push(item);
            }
        }
        reordered.sort_unstable_by(|a, b| {
            let a = unsafe { &**a };
            let b = unsafe { &**b };
            let af = a.IsSpanningFlexibleTrack(direction);
            let bf = b.IsSpanningFlexibleTrack(direction);
            if af || bf {
                af.cmp(&bf)
            } else {
                a.SpanSize(direction).cmp(&b.SpanSize(direction))
            }
        });
        let mut begin = 0;
        while begin < reordered.len()
            && !unsafe { &*reordered[begin] }.IsSpanningFlexibleTrack(direction)
        {
            let span = unsafe { &*reordered[begin] }.SpanSize(direction);
            let mut end = begin;
            loop {
                debug_assert!(!unsafe { &*reordered[end] }.IsSpanningFlexibleTrack(direction));
                end += 1;
                if end == reordered.len()
                    || unsafe { &*reordered[end] }.IsSpanningFlexibleTrack(direction)
                    || unsafe { &*reordered[end] }.SpanSize(direction) != span
                {
                    break;
                }
            }
            for kind in [
                kForIntrinsicMinimums,
                kForContentBasedMinimums,
                kForMaxContentMinimums,
                kForIntrinsicMaximums,
                kForMaxContentMaximums,
            ] {
                self.IncreaseTrackSizesToAccommodateGridItems(
                    contribution,
                    &reordered[begin..end],
                    kind,
                    false,
                    tracks,
                );
            }
            begin = end;
        }
        debug_assert!(reordered[begin..]
            .iter()
            .all(|p| unsafe { &**p }.IsSpanningFlexibleTrack(direction)));
        if begin != reordered.len() {
            for kind in [
                kForIntrinsicMinimums,
                kForContentBasedMinimums,
                kForMaxContentMinimums,
            ] {
                self.IncreaseTrackSizesToAccommodateGridItems(
                    contribution,
                    &reordered[begin..],
                    kind,
                    true,
                    tracks,
                );
            }
        }
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:1027-1054
    fn MaximizeTracks(&self, tracks: &mut GridSizingTrackCollection) {
        let free = self.DetermineFreeSpace(tracks);
        if free == LayoutUnit::default() {
            return;
        }
        let mut sets = Vec::with_capacity(tracks.base.GetSetCount() as usize);
        let mut it = tracks.GetSetIterator();
        while !it.IsAtEnd() {
            sets.push(it.CurrentSet());
            it.MoveToNextSet();
        }
        DistributeExtraSpaceToSetsEqually(free, kForFreeSpace, &mut sets, std::ptr::null_mut());
        for pointer in sets {
            let set = unsafe { &mut *pointer };
            set.IncreaseBaseSize(set.BaseSize() + set.item_incurred_increase);
        }
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:1057-1110
    fn StretchAutoTracks(&self, tracks: &mut GridSizingTrackCollection) {
        let columns = tracks.base.Direction() == kForColumns;
        let alignment = if columns {
            self.columns_alignment_
        } else {
            self.rows_alignment_
        };
        if alignment.Distribution() != ContentDistributionType::kStretch
            && (alignment.Distribution() != ContentDistributionType::kDefault
                || alignment.GetPosition() != ContentPosition::kNormal)
        {
            return;
        }
        let mut sets = Vec::new();
        let mut it = tracks.GetSetIterator();
        while !it.IsAtEnd() {
            let set = unsafe { &mut *it.CurrentSet() };
            it.MoveToNextSet();
            if set.track_size.HasAutoMaxTrackBreadth() && !set.track_size.IsFitContent() {
                sets.push(set as *mut GridSet);
            }
        }
        if sets.is_empty() {
            return;
        }
        let mut free = self.DetermineFreeSpace(tracks);
        if free == kIndefiniteSize {
            free = if columns {
                self.min_available_size_.inline_size
            } else {
                self.min_available_size_.block_size
            };
            debug_assert_ne!(free, kIndefiniteSize);
            free -= tracks.TotalTrackSize();
        }
        if free <= LayoutUnit::default() {
            return;
        }
        let beyond = &mut sets as *mut _;
        DistributeExtraSpaceToSetsEqually(free, kForFreeSpace, &mut sets, beyond);
        for pointer in sets {
            let set = unsafe { &mut *pointer };
            set.IncreaseBaseSize(set.BaseSize() + set.item_incurred_increase);
        }
    }

    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:1113-1302
    fn ExpandFlexibleTracks(
        &self,
        contribution: &mut ContributionSizeFunctionRef<'_>,
        tracks: &mut GridSizingTrackCollection,
        items: &mut GridItems,
    ) {
        let free = self.DetermineFreeSpace(tracks);
        if free == LayoutUnit::default() {
            return;
        }
        let mut flexible = Vec::new();
        let mut fr = 0f32;
        let direction = tracks.base.Direction();
        if free != kIndefiniteSize {
            fr = FindFrSize(
                tracks.GetSetIterator(),
                if direction == kForColumns {
                    self.available_size_.inline_size
                } else {
                    self.available_size_.block_size
                },
                tracks.base.GutterSize(),
                &mut flexible,
            );
        } else {
            let range = items.IncludeSubgriddedItems();
            let mut it = range.begin();
            let end = range.end();
            while it.NotEqual(&end) {
                let item = unsafe { &mut *it.Get() };
                it.Advance();
                if !item.IsSpanningFlexibleTrack(direction)
                    || !item.IsConsideredForSizing(direction)
                {
                    continue;
                }
                let size = contribution(kForMaxContentMaximums, item);
                fr = fr.max(FindFrSize(
                    item.SetIterator(tracks),
                    size,
                    tracks.base.GutterSize(),
                    &mut flexible,
                ));
            }
            let mut it = tracks.GetConstSetIterator();
            while !it.IsAtEnd() {
                let set = unsafe { &*it.CurrentSet() };
                it.MoveToNextSet();
                if !set.track_size.HasFlexMaxTrackBreadth() {
                    continue;
                }
                debug_assert!(set.track_count > 0);
                let factor = set.FlexFactor().max(set.track_count as f32);
                fr = fr.max(set.BaseSize().RawValue() as f32 / factor);
            }
        }
        let mut leftover = 0f32;
        let mut it = tracks.GetSetIterator();
        while !it.IsAtEnd() {
            let set = unsafe { &mut *it.CurrentSet() };
            it.MoveToNextSet();
            if !set.track_size.HasFlexMaxTrackBreadth() {
                continue;
            }
            let share = SaturateFloat(fr * set.FlexFactor() + leftover);
            let expanded = LayoutUnit::FromRawValue(SaturateFloat(share + f32::EPSILON) as i32);
            if !expanded.MightBeSaturated() && expanded >= set.BaseSize() {
                set.IncreaseBaseSize(expanded);
                leftover = SaturateFloat(share - expanded.RawValue() as f32).max(0.0);
            }
        }
    }
    // cpp: layoutng_grid/grid_track_sizing_algorithm.cc:1306-1332
    fn DetermineFreeSpace(&self, tracks: &GridSizingTrackCollection) -> LayoutUnit {
        let columns = tracks.base.Direction() == kForColumns;
        match if columns {
            self.sizing_constraint_
        } else {
            SizingConstraint::kLayout
        } {
            SizingConstraint::kLayout => {
                let mut free = if columns {
                    self.available_size_.inline_size
                } else {
                    self.available_size_.block_size
                };
                if free != kIndefiniteSize {
                    free = (free - tracks.TotalTrackSize()).ClampNegativeToZero();
                }
                free
            }
            SizingConstraint::kMaxContent => kIndefiniteSize,
            SizingConstraint::kMinContent => LayoutUnit::default(),
        }
    }
}

// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:1127-1219
fn FindFrSize(
    mut it: SetIterator,
    mut leftover: LayoutUnit,
    gutter: LayoutUnit,
    flexible: &mut Vec<*mut GridSet>,
) -> f32 {
    let mut sum = 0f32;
    let mut track_count = 0u32;
    flexible.clear();
    while !it.IsAtEnd() {
        let set = unsafe { &mut *it.CurrentSet() };
        it.MoveToNextSet();
        if set.track_size.HasFlexMaxTrackBreadth() && !AreEqualFloat(set.FlexFactor(), 0.0) {
            sum = SaturateFloat(sum + set.FlexFactor());
            flexible.push(set);
        } else {
            leftover -= set.BaseSize();
        }
        track_count = track_count.wrapping_add(set.track_count);
    }
    leftover -= gutter * track_count.wrapping_sub(1);
    if leftover < LayoutUnit::default() || flexible.is_empty() {
        return 0.0;
    }
    flexible.sort_unstable_by(|a, b| {
        let a = unsafe { &**a };
        let b = unsafe { &**b };
        let left = a.BaseSize().RawValue() as f32 * b.FlexFactor();
        let right = b.BaseSize().RawValue() as f32 * a.FlexFactor();
        right
            .partial_cmp(&left)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut current = 0;
    while leftover > LayoutUnit::default() && current != flexible.len() {
        sum = sum.max(1.0);
        let mut next = current;
        while next != flexible.len()
            && unsafe { &*flexible[next] }.FlexFactor() * (leftover.RawValue() as f32)
                < (unsafe { &*flexible[next] }.BaseSize().RawValue() as f32 * sum)
        {
            next += 1;
        }
        if current == next {
            debug_assert!(!AreEqualFloat(sum, 0.0));
            return leftover.RawValue() as f32 / sum;
        }
        for pointer in &flexible[current..next] {
            let set = unsafe { &**pointer };
            sum = SaturateFloat(sum - set.FlexFactor());
            leftover -= set.BaseSize();
        }
        current = next;
    }
    0.0
}
// cpp: layoutng_grid/grid_track_sizing_algorithm.cc:18-22
// base::ClampedNumeric<float> uses SaturationDefaultLimits<float>, retaining
// IEEE infinities and NaNs. Saturation to i32 happens only at the raw-unit cast.
fn SaturateFloat(value: f32) -> f32 {
    value
}

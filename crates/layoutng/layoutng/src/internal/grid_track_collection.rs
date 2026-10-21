#![allow(non_snake_case)]

use foundation::{kIndefiniteSize, LayoutUnit, Length, Vector, Visitor};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::grid_enums::GridTrackSizingDirection::{self, kForColumns};
use layoutng_style::style::grid_track_list::GridTrackList;
use layoutng_style::style::grid_track_size::GridTrackSize;

// Definitions of these non-inline methods belong to //src/layoutng_grid.
unsafe extern "Rust" {
    fn GridRangeIsCollapsed(this: &GridRange) -> bool;
    fn GridRangeIsImplicit(this: &GridRange) -> bool;
    fn GridRangeIsAutoRepeat(this: &GridRange) -> bool;
    fn GridRangeSetIsCollapsed(this: &mut GridRange);
    fn GridRangeSetIsImplicit(this: &mut GridRange);
    fn GridRangeSetIsAutoRepeat(this: &mut GridRange);
    fn GridLayoutTrackCollectionFirstNonCollapsedLineIndex(this: &GridLayoutTrackCollection)
        -> u32;
    fn GridLayoutTrackCollectionHasIndefiniteSet(this: &GridLayoutTrackCollection) -> bool;
    fn GridRangeBuilderNew(
        style: &ComputedStyle,
        direction: GridTrackSizingDirection,
        auto_repetitions: u32,
        start_offset: u32,
    ) -> GridRangeBuilder;
    fn GridRangeBuilderNewFromTracks(
        explicit_tracks: &GridTrackList,
        implicit_tracks: &GridTrackList,
        auto_repetitions: u32,
        start_offset: u32,
    ) -> GridRangeBuilder;
    fn GridRangeBuilderEnsureTrackCoverage(
        this: &mut GridRangeBuilder,
        start_line: u32,
        span_length: u32,
        start_index: *mut u32,
        end_index: *mut u32,
    );
    fn GridRangeBuilderFinalizeRanges(
        this: &mut GridRangeBuilder,
        needs_intrinsic_track_size: bool,
    ) -> GridRangeVector;
    fn GridLayoutTrackCollectionRangeStartLine(
        this: &GridLayoutTrackCollection,
        range_index: u32,
    ) -> u32;
    fn GridLayoutTrackCollectionRangeTrackCount(
        this: &GridLayoutTrackCollection,
        range_index: u32,
    ) -> u32;
    fn GridLayoutTrackCollectionRangeEndLine(
        this: &GridLayoutTrackCollection,
        range_index: u32,
    ) -> u32;
    fn GridLayoutTrackCollectionRangeIndexFromGridLine(
        this: &GridLayoutTrackCollection,
        grid_line: u32,
    ) -> u32;
    fn GridLayoutTrackCollectionRangeSetCount(
        this: &GridLayoutTrackCollection,
        range_index: u32,
    ) -> u32;
    fn GridLayoutTrackCollectionRangeBeginSetIndex(
        this: &GridLayoutTrackCollection,
        range_index: u32,
    ) -> u32;
    fn GridLayoutTrackCollectionRangeProperties(
        this: &GridLayoutTrackCollection,
        range_index: u32,
    ) -> TrackSpanProperties;
    fn GridLayoutTrackCollectionEndLineOfImplicitGrid(this: &GridLayoutTrackCollection) -> u32;
    fn GridLayoutTrackCollectionIsGridLineWithinImplicitGrid(
        this: &GridLayoutTrackCollection,
        grid_line: u32,
    ) -> bool;
    fn GridLayoutTrackCollectionGetSetCount(this: &GridLayoutTrackCollection) -> u32;
    fn GridLayoutTrackCollectionGetSetOffset(
        this: &GridLayoutTrackCollection,
        set_index: u32,
    ) -> LayoutUnit;
    fn GridLayoutTrackCollectionGetSetTrackCount(
        this: &GridLayoutTrackCollection,
        set_index: u32,
    ) -> u32;
    fn GridLayoutTrackCollectionStartExtraMargin(
        this: &GridLayoutTrackCollection,
        set_index: u32,
    ) -> LayoutUnit;
    fn GridLayoutTrackCollectionEndExtraMargin(
        this: &GridLayoutTrackCollection,
        set_index: u32,
    ) -> LayoutUnit;
    fn GridLayoutTrackCollectionAdjustSingleSetOffset(
        this: &mut GridLayoutTrackCollection,
        set_index: u32,
        delta: LayoutUnit,
    );
    fn GridLayoutTrackCollectionAdjustSetOffsets(
        this: &mut GridLayoutTrackCollection,
        set_index: u32,
        delta: LayoutUnit,
    );
    fn GridLayoutTrackCollectionCalculateSetSpanSize(
        this: &GridLayoutTrackCollection,
    ) -> LayoutUnit;
    fn GridLayoutTrackCollectionCalculateSetSpanSizeRange(
        this: &GridLayoutTrackCollection,
        begin_set_index: u32,
        end_set_index: u32,
    ) -> LayoutUnit;
    fn GridLayoutTrackCollectionCreateSubgridTrackCollection(
        this: &GridLayoutTrackCollection,
        begin_range_index: u32,
        end_range_index: u32,
        gutter_size: LayoutUnit,
        margin: &BoxStrut,
        border_scrollbar_padding: &BoxStrut,
        direction: GridTrackSizingDirection,
        opposite_direction: bool,
        auto_placed: bool,
    ) -> *mut GridLayoutTrackCollection;
    fn GridLayoutTrackCollectionCreateSubgridBaselines(
        this: &GridLayoutTrackCollection,
        begin_range_index: u32,
        end_range_index: u32,
        gutter_size: LayoutUnit,
        margin: &BoxStrut,
        border_scrollbar_padding: &BoxStrut,
        direction: GridTrackSizingDirection,
        opposite_direction: bool,
        parent_baselines: &GridTrackBaselines,
    ) -> *mut GridTrackBaselines;
    fn GridLayoutTrackCollectionHasFlexibleTrack(this: &GridLayoutTrackCollection) -> bool;
    fn GridLayoutTrackCollectionHasIntrinsicTrack(this: &GridLayoutTrackCollection) -> bool;
    fn GridLayoutTrackCollectionHasNonDefiniteTrack(this: &GridLayoutTrackCollection) -> bool;
    fn GridLayoutTrackCollectionIsDependentOnAvailableSize(
        this: &GridLayoutTrackCollection,
    ) -> bool;
    fn GridLayoutTrackCollectionIsSpanningIndefiniteSet(
        this: &GridLayoutTrackCollection,
        begin_set_index: u32,
        end_set_index: u32,
    ) -> bool;
    fn GridSetNewWithDefinition(
        track_count: u32,
        track_definition: &GridTrackSize,
        is_available_size_indefinite: bool,
    ) -> GridSet;
    fn GridSetFlexFactor(this: &GridSet) -> f32;
    fn GridSetBaseSize(this: &GridSet) -> LayoutUnit;
    fn GridSetGrowthLimit(this: &GridSet) -> LayoutUnit;
    fn GridSetInitBaseSize(this: &mut GridSet, new_base_size: LayoutUnit);
    fn GridSetIncreaseBaseSize(this: &mut GridSet, new_base_size: LayoutUnit);
    fn GridSetIncreaseGrowthLimit(this: &mut GridSet, new_growth_limit: LayoutUnit);
    fn GridSetEnsureGrowthLimitIsNotLessThanBaseSize(this: &mut GridSet);
    fn GridSetIsGrowthLimitLessThanBaseSize(this: &GridSet) -> bool;
    fn GridSizingTrackCollectionNew(
        ranges: GridRangeVector,
        direction: GridTrackSizingDirection,
        should_store_collapsed_track_indexes: bool,
    ) -> GridSizingTrackCollection;
    fn GridSizingTrackCollectionGetSetAt(
        this: &mut GridSizingTrackCollection,
        set_index: u32,
    ) -> *mut GridSet;
    fn GridSizingTrackCollectionGetSetAtConst(
        this: &GridSizingTrackCollection,
        set_index: u32,
    ) -> *const GridSet;
    fn GridSizingTrackCollectionGetSetIterator(this: &mut GridSizingTrackCollection)
        -> SetIterator;
    fn GridSizingTrackCollectionGetConstSetIterator(
        this: &GridSizingTrackCollection,
    ) -> ConstSetIterator;
    fn GridSizingTrackCollectionGetSetIteratorRange(
        this: &mut GridSizingTrackCollection,
        begin_set_index: u32,
        end_set_index: u32,
    ) -> SetIterator;
    fn GridSizingTrackCollectionTotalTrackSize(this: &GridSizingTrackCollection) -> LayoutUnit;
    fn GridSizingTrackCollectionBuildSets(
        this: &mut GridSizingTrackCollection,
        style: &ComputedStyle,
        available_size: &LogicalSize,
    );
    fn GridSizingTrackCollectionBuildSetsFromLists(
        this: &mut GridSizingTrackCollection,
        explicit_tracks: &GridTrackList,
        implicit_tracks: &GridTrackList,
        is_grid_lanes: bool,
        is_available_size_indefinite: bool,
    );
    fn GridSizingTrackCollectionSetIndefiniteGrowthLimitsToBaseSize(
        this: &mut GridSizingTrackCollection,
    );
    fn GridSizingTrackCollectionCacheDefiniteSetsGeometry(this: &mut GridSizingTrackCollection);
    fn GridSizingTrackCollectionCacheInitializedSetsGeometry(
        this: &mut GridSizingTrackCollection,
        first_set_offset: LayoutUnit,
    );
    fn GridSizingTrackCollectionFinalizeSetsGeometry(
        this: &mut GridSizingTrackCollection,
        first_set_offset: LayoutUnit,
        override_gutter_size: LayoutUnit,
    );
    fn GridSizingTrackCollectionInitializeSets(
        this: &mut GridSizingTrackCollection,
        grid_available_size: LayoutUnit,
    );
}

// cpp: layoutng/internal/grid_track_collection.h:27-41
pub trait GridTrackCollectionBase {
    fn RangeCount(&self) -> u32;
    fn RangeStartLine(&self, range_index: u32) -> u32;
    fn RangeTrackCount(&self, range_index: u32) -> u32;
    fn RangeEndLine(&self, range_index: u32) -> u32;
    fn RangeIndexFromGridLine(&self, grid_line: u32) -> u32;
}

// cpp: layoutng/internal/grid_track_collection.h:42-71
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyId {
    kNoPropertyId = 0,
    kHasAutoMinimumTrack = 1 << 0,
    kHasFixedMaximumTrack = 1 << 1,
    kHasFixedMinimumTrack = 1 << 2,
    kHasFlexibleTrack = 1 << 3,
    kHasIntrinsicTrack = 1 << 4,
    kHasNonDefiniteTrack = 1 << 5,
    kIsCollapsed = 1 << 6,
    kIsDependentOnAvailableSize = 1 << 7,
    kIsImplicit = 1 << 8,
    kIsAutoRepeat = 1 << 9,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrackSpanProperties {
    bitmask_: u32,
}

impl TrackSpanProperties {
    // cpp: layoutng/internal/grid_track_collection.h:73-82
    pub fn HasProperty(&self, id: PropertyId) -> bool {
        self.bitmask_ & id as u32 != 0
    }

    pub fn ResetType(&mut self) {
        self.bitmask_ &= PropertyId::kIsCollapsed as u32
            | PropertyId::kIsImplicit as u32
            | PropertyId::kIsAutoRepeat as u32;
    }

    pub fn SetProperty(&mut self, id: PropertyId) {
        self.bitmask_ |= id as u32;
    }
}

// cpp: layoutng/internal/grid_track_collection.h:84-87
impl std::ops::BitOrAssign<&TrackSpanProperties> for TrackSpanProperties {
    fn bitor_assign(&mut self, other: &TrackSpanProperties) {
        self.bitmask_ |= other.bitmask_;
    }
}

// cpp: layoutng/internal/grid_track_collection.h:95-112
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridRange {
    pub begin_set_index: u32,
    pub repeater_index: u32,
    pub repeater_offset: u32,
    pub set_count: u32,
    pub start_line: u32,
    pub track_count: u32,
    pub properties: TrackSpanProperties,
}

pub type GridRangeVector = Vector<GridRange>;

// cpp: layoutng/internal/grid_track_collection.h:94-101
impl GridRange {
    pub fn IsCollapsed(&self) -> bool {
        unsafe { GridRangeIsCollapsed(self) }
    }

    pub fn IsImplicit(&self) -> bool {
        unsafe { GridRangeIsImplicit(self) }
    }

    pub fn IsAutoRepeat(&self) -> bool {
        unsafe { GridRangeIsAutoRepeat(self) }
    }

    pub fn SetIsCollapsed(&mut self) {
        unsafe { GridRangeSetIsCollapsed(self) }
    }

    pub fn SetIsImplicit(&mut self) {
        unsafe { GridRangeSetIsImplicit(self) }
    }

    pub fn SetIsAutoRepeat(&mut self) {
        unsafe { GridRangeSetIsAutoRepeat(self) }
    }
}

// cpp: layoutng/internal/grid_track_collection.h:116-140
#[derive(Clone, Default, PartialEq)]
pub struct GridTrackBaselines {
    pub major: Vector<LayoutUnit>,
    pub minor: Vector<LayoutUnit>,
}

impl GridTrackBaselines {
    pub fn Reset(&mut self, set_count: u32) {
        self.major.resize(set_count as usize, LayoutUnit::Min());
        self.minor.resize(set_count as usize, LayoutUnit::Min());
        self.major.fill(LayoutUnit::Min());
        self.minor.fill(LayoutUnit::Min());
    }

    pub fn Trace(&self, _visitor: &mut Visitor) {}
}

// cpp: layoutng/internal/grid_track_collection.h:146-155
pub struct TrackBoundaryToRangePair {
    pub grid_line: u32,
    pub grid_item_range_index_to_cache: *mut u32,
}

impl TrackBoundaryToRangePair {
    pub fn new(grid_line: u32, grid_item_range_index_to_cache: *mut u32) -> Self {
        Self {
            grid_line,
            grid_item_range_index_to_cache,
        }
    }
}

// cpp: layoutng/internal/grid_track_collection.h:120-179
pub struct GridRangeBuilder {
    pub(crate) auto_repetitions_: u32,
    pub(crate) start_offset_: u32,
    pub(crate) must_sort_grid_lines_: bool,
    pub(crate) explicit_tracks_: *const GridTrackList,
    pub(crate) implicit_tracks_: *const GridTrackList,
    pub(crate) start_lines_: Vector<TrackBoundaryToRangePair>,
    pub(crate) end_lines_: Vector<TrackBoundaryToRangePair>,
}

impl GridRangeBuilder {
    // Package implementation access for the style constructor's named areas.
    pub fn GridPackageEnsureNamedAreaCoverage(&mut self, named_end: u32) {
        let current = self
            .end_lines_
            .last()
            .map_or(self.start_offset_, |line| line.grid_line);
        if current < named_end {
            self.start_lines_
                .push(TrackBoundaryToRangePair::new(current, std::ptr::null_mut()));
            self.end_lines_.push(TrackBoundaryToRangePair::new(
                named_end,
                std::ptr::null_mut(),
            ));
        }
    }
    // cpp: layoutng/internal/grid_track_collection.h:125-128
    pub fn new(
        style: &ComputedStyle,
        direction: GridTrackSizingDirection,
        auto_repetitions: u32,
        start_offset: u32,
    ) -> Self {
        unsafe { GridRangeBuilderNew(style, direction, auto_repetitions, start_offset) }
    }

    // cpp: layoutng/internal/grid_track_collection.h:159-162
    pub(crate) fn new_from_tracks(
        explicit_tracks: &GridTrackList,
        implicit_tracks: &GridTrackList,
        auto_repetitions: u32,
        start_offset: u32,
    ) -> Self {
        unsafe {
            GridRangeBuilderNewFromTracks(
                explicit_tracks,
                implicit_tracks,
                auto_repetitions,
                start_offset,
            )
        }
    }

    pub(crate) fn new_from_tracks_default_offset(
        explicit_tracks: &GridTrackList,
        implicit_tracks: &GridTrackList,
        auto_repetitions: u32,
    ) -> Self {
        Self::new_from_tracks(explicit_tracks, implicit_tracks, auto_repetitions, 0)
    }

    // cpp: layoutng/internal/grid_track_collection.h:134-144
    pub fn EnsureTrackCoverage(
        &mut self,
        start_line: u32,
        span_length: u32,
        start_index: *mut u32,
        end_index: *mut u32,
    ) {
        unsafe {
            GridRangeBuilderEnsureTrackCoverage(
                self,
                start_line,
                span_length,
                start_index,
                end_index,
            )
        }
    }

    pub fn FinalizeRanges(&mut self, needs_intrinsic_track_size: bool) -> GridRangeVector {
        unsafe { GridRangeBuilderFinalizeRanges(self, needs_intrinsic_track_size) }
    }

    pub fn FinalizeRangesDefault(&mut self) -> GridRangeVector {
        self.FinalizeRanges(false)
    }
}

// cpp: layoutng/internal/grid_track_collection.h:185-190
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SetGeometry {
    pub offset: LayoutUnit,
    pub track_count: u32,
}

impl SetGeometry {
    pub fn new(offset: LayoutUnit, track_count: u32) -> Self {
        Self {
            offset,
            track_count,
        }
    }
}

// The C++ virtual IsForSizing() is retained as an explicit discriminant in
// the base-first Rust record, so calls through a base pointer dispatch to the
// sizing override without inventing an aliased trait-object reference.
// cpp: layoutng/internal/grid_track_collection.h:182-342
#[repr(C)]
pub struct GridLayoutTrackCollection {
    pub(crate) is_for_sizing_: bool,
    pub(crate) gutter_size_: LayoutUnit,
    pub(crate) ranges_: GridRangeVector,
    pub(crate) properties_: TrackSpanProperties,
    pub(crate) sets_geometry_: Vector<SetGeometry>,
    pub(crate) track_direction_: GridTrackSizingDirection,
    pub(crate) last_indefinite_index_: Vector<u32>,
    pub(crate) accumulated_gutter_size_delta_: LayoutUnit,
    pub(crate) accumulated_start_extra_margin_: LayoutUnit,
    pub(crate) accumulated_end_extra_margin_: LayoutUnit,
    pub(crate) collapsed_track_indexes_: Vector<u32>,
}

impl GridLayoutTrackCollection {
    // cpp: layoutng/internal/grid_track_collection.h:193-194
    pub fn new(track_direction: GridTrackSizingDirection) -> Self {
        Self {
            is_for_sizing_: false,
            gutter_size_: LayoutUnit::default(),
            ranges_: Vector::default(),
            properties_: TrackSpanProperties::default(),
            sets_geometry_: Vector::default(),
            track_direction_: track_direction,
            last_indefinite_index_: Vector::default(),
            accumulated_gutter_size_delta_: LayoutUnit::default(),
            accumulated_start_extra_margin_: LayoutUnit::default(),
            accumulated_end_extra_margin_: LayoutUnit::default(),
            collapsed_track_indexes_: Vector::default(),
        }
    }

    // cpp: layoutng/internal/grid_track_collection.h:199-199
    pub fn IsForSizing(&self) -> bool {
        self.is_for_sizing_
    }

    // cpp: layoutng/internal/grid_track_collection.h:217-217
    pub fn RangeCount(&self) -> u32 {
        self.ranges_.len() as u32
    }

    // cpp: layoutng/internal/grid_track_collection.h:218-249
    pub fn RangeStartLine(&self, range_index: u32) -> u32 {
        unsafe { GridLayoutTrackCollectionRangeStartLine(self, range_index) }
    }

    pub fn RangeTrackCount(&self, range_index: u32) -> u32 {
        unsafe { GridLayoutTrackCollectionRangeTrackCount(self, range_index) }
    }

    pub fn RangeEndLine(&self, range_index: u32) -> u32 {
        unsafe { GridLayoutTrackCollectionRangeEndLine(self, range_index) }
    }

    pub fn RangeIndexFromGridLine(&self, grid_line: u32) -> u32 {
        unsafe { GridLayoutTrackCollectionRangeIndexFromGridLine(self, grid_line) }
    }

    pub fn RangeSetCount(&self, range_index: u32) -> u32 {
        unsafe { GridLayoutTrackCollectionRangeSetCount(self, range_index) }
    }

    pub fn RangeBeginSetIndex(&self, range_index: u32) -> u32 {
        unsafe { GridLayoutTrackCollectionRangeBeginSetIndex(self, range_index) }
    }

    pub fn RangeProperties(&self, range_index: u32) -> TrackSpanProperties {
        unsafe { GridLayoutTrackCollectionRangeProperties(self, range_index) }
    }

    pub fn EndLineOfImplicitGrid(&self) -> u32 {
        unsafe { GridLayoutTrackCollectionEndLineOfImplicitGrid(self) }
    }

    pub fn IsGridLineWithinImplicitGrid(&self, grid_line: u32) -> bool {
        unsafe { GridLayoutTrackCollectionIsGridLineWithinImplicitGrid(self, grid_line) }
    }

    pub fn GetSetCount(&self) -> u32 {
        unsafe { GridLayoutTrackCollectionGetSetCount(self) }
    }

    pub fn GetSetOffset(&self, set_index: u32) -> LayoutUnit {
        unsafe { GridLayoutTrackCollectionGetSetOffset(self, set_index) }
    }

    pub fn GetSetTrackCount(&self, set_index: u32) -> u32 {
        unsafe { GridLayoutTrackCollectionGetSetTrackCount(self, set_index) }
    }

    pub fn StartExtraMargin(&self, set_index: u32) -> LayoutUnit {
        unsafe { GridLayoutTrackCollectionStartExtraMargin(self, set_index) }
    }

    pub fn StartExtraMarginDefault(&self) -> LayoutUnit {
        self.StartExtraMargin(0)
    }

    pub fn EndExtraMargin(&self, set_index: u32) -> LayoutUnit {
        unsafe { GridLayoutTrackCollectionEndExtraMargin(self, set_index) }
    }

    pub fn EndExtraMarginDefault(&self) -> LayoutUnit {
        self.EndExtraMargin(u32::MAX)
    }

    // cpp: layoutng/internal/grid_track_collection.h:252-254
    pub fn AccumulatedGutterSizeDelta(&self) -> LayoutUnit {
        self.accumulated_gutter_size_delta_
    }

    // cpp: layoutng/internal/grid_track_collection.h:258-288
    pub fn AdjustSingleSetOffset(&mut self, set_index: u32, delta: LayoutUnit) {
        unsafe { GridLayoutTrackCollectionAdjustSingleSetOffset(self, set_index, delta) }
    }

    pub fn AdjustSetOffsets(&mut self, set_index: u32, delta: LayoutUnit) {
        unsafe { GridLayoutTrackCollectionAdjustSetOffsets(self, set_index, delta) }
    }

    pub fn CalculateSetSpanSize(&self) -> LayoutUnit {
        unsafe { GridLayoutTrackCollectionCalculateSetSpanSize(self) }
    }

    pub fn CalculateSetSpanSizeRange(
        &self,
        begin_set_index: u32,
        end_set_index: u32,
    ) -> LayoutUnit {
        unsafe {
            GridLayoutTrackCollectionCalculateSetSpanSizeRange(self, begin_set_index, end_set_index)
        }
    }

    pub fn CreateSubgridTrackCollection(
        &self,
        begin_range_index: u32,
        end_range_index: u32,
        gutter_size: LayoutUnit,
        margin: &BoxStrut,
        border_scrollbar_padding: &BoxStrut,
        direction: GridTrackSizingDirection,
        opposite_direction: bool,
        auto_placed: bool,
    ) -> *mut GridLayoutTrackCollection {
        unsafe {
            GridLayoutTrackCollectionCreateSubgridTrackCollection(
                self,
                begin_range_index,
                end_range_index,
                gutter_size,
                margin,
                border_scrollbar_padding,
                direction,
                opposite_direction,
                auto_placed,
            )
        }
    }

    pub fn CreateSubgridTrackCollectionDefaultPlacement(
        &self,
        begin_range_index: u32,
        end_range_index: u32,
        gutter_size: LayoutUnit,
        margin: &BoxStrut,
        border_scrollbar_padding: &BoxStrut,
        direction: GridTrackSizingDirection,
        opposite_direction: bool,
    ) -> *mut GridLayoutTrackCollection {
        self.CreateSubgridTrackCollection(
            begin_range_index,
            end_range_index,
            gutter_size,
            margin,
            border_scrollbar_padding,
            direction,
            opposite_direction,
            false,
        )
    }

    pub fn CreateSubgridBaselines(
        &self,
        begin_range_index: u32,
        end_range_index: u32,
        gutter_size: LayoutUnit,
        margin: &BoxStrut,
        border_scrollbar_padding: &BoxStrut,
        direction: GridTrackSizingDirection,
        opposite_direction: bool,
        parent_baselines: &GridTrackBaselines,
    ) -> *mut GridTrackBaselines {
        unsafe {
            GridLayoutTrackCollectionCreateSubgridBaselines(
                self,
                begin_range_index,
                end_range_index,
                gutter_size,
                margin,
                border_scrollbar_padding,
                direction,
                opposite_direction,
                parent_baselines,
            )
        }
    }

    // cpp: layoutng/internal/grid_track_collection.h:290-291
    pub fn Direction(&self) -> GridTrackSizingDirection {
        self.track_direction_
    }

    pub fn GutterSize(&self) -> LayoutUnit {
        self.gutter_size_
    }

    // Package implementation access for the style-owned BuildSets body.
    pub fn GridPackageSetGutterSize(&mut self, gutter: LayoutUnit) {
        self.gutter_size_ = gutter;
    }

    // cpp: layoutng/internal/grid_track_collection.h:293-298
    pub fn HasFlexibleTrack(&self) -> bool {
        unsafe { GridLayoutTrackCollectionHasFlexibleTrack(self) }
    }

    pub fn HasIntrinsicTrack(&self) -> bool {
        unsafe { GridLayoutTrackCollectionHasIntrinsicTrack(self) }
    }

    pub fn HasNonDefiniteTrack(&self) -> bool {
        unsafe { GridLayoutTrackCollectionHasNonDefiniteTrack(self) }
    }

    pub fn IsDependentOnAvailableSize(&self) -> bool {
        unsafe { GridLayoutTrackCollectionIsDependentOnAvailableSize(self) }
    }

    // cpp: layoutng/internal/grid_track_collection.h:300-302
    pub fn HasNonCollapsedLine(&self) -> bool {
        self.FirstNonCollapsedLineIndex() != u32::MAX
    }

    // cpp: layoutng/internal/grid_track_collection.h:299
    pub fn FirstNonCollapsedLineIndex(&self) -> u32 {
        unsafe { GridLayoutTrackCollectionFirstNonCollapsedLineIndex(self) }
    }

    // cpp: layoutng/internal/grid_track_collection.h:294-294
    pub fn HasIndefiniteSet(&self) -> bool {
        unsafe { GridLayoutTrackCollectionHasIndefiniteSet(self) }
    }

    // cpp: layoutng/internal/grid_track_collection.h:304-306
    pub fn CollapsedTrackIndexes(&self) -> &Vector<u32> {
        &self.collapsed_track_indexes_
    }

    // cpp: layoutng/internal/grid_track_collection.h:308
    pub fn Trace(&self, _visitor: &mut Visitor) {}

    // cpp: layoutng/internal/grid_track_collection.h:316-317
    pub(crate) fn IsSpanningIndefiniteSet(&self, begin_set_index: u32, end_set_index: u32) -> bool {
        unsafe {
            GridLayoutTrackCollectionIsSpanningIndefiniteSet(self, begin_set_index, end_set_index)
        }
    }
}

impl GridTrackCollectionBase for GridLayoutTrackCollection {
    fn RangeCount(&self) -> u32 {
        self.RangeCount()
    }
    fn RangeStartLine(&self, range_index: u32) -> u32 {
        self.RangeStartLine(range_index)
    }
    fn RangeTrackCount(&self, range_index: u32) -> u32 {
        self.RangeTrackCount(range_index)
    }
    fn RangeEndLine(&self, range_index: u32) -> u32 {
        self.RangeEndLine(range_index)
    }
    fn RangeIndexFromGridLine(&self, grid_line: u32) -> u32 {
        self.RangeIndexFromGridLine(grid_line)
    }
}

// cpp: layoutng/internal/grid_track_collection.h:203-214
impl PartialEq for GridLayoutTrackCollection {
    fn eq(&self, other: &Self) -> bool {
        self.gutter_size_ == other.gutter_size_
            && self.track_direction_ == other.track_direction_
            && self.accumulated_gutter_size_delta_ == other.accumulated_gutter_size_delta_
            && self.accumulated_start_extra_margin_ == other.accumulated_start_extra_margin_
            && self.accumulated_end_extra_margin_ == other.accumulated_end_extra_margin_
            && self.last_indefinite_index_ == other.last_indefinite_index_
            && self.ranges_ == other.ranges_
            && self.sets_geometry_ == other.sets_geometry_
    }
}

// cpp: layoutng/internal/grid_track_collection.h:389-429
pub struct GridSet {
    pub track_count: u32,
    pub track_size: GridTrackSize,
    pub base_size: LayoutUnit,
    pub growth_limit: LayoutUnit,
    pub planned_increase: LayoutUnit,
    pub fit_content_limit: LayoutUnit,
    pub item_incurred_increase: LayoutUnit,
    pub is_infinitely_growable: std::mem::MaybeUninit<bool>,
}

impl GridSet {
    // cpp: layoutng/internal/grid_track_collection.h:390-391
    pub fn new(track_count: u32) -> Self {
        let auto = Length::Auto();
        Self {
            track_count,
            track_size: GridTrackSize::new_minmax(&auto, &auto),
            base_size: LayoutUnit::default(),
            growth_limit: LayoutUnit::default(),
            planned_increase: LayoutUnit::default(),
            fit_content_limit: LayoutUnit::default(),
            item_incurred_increase: LayoutUnit::default(),
            is_infinitely_growable: std::mem::MaybeUninit::uninit(),
        }
    }

    // cpp: layoutng/internal/grid_track_collection.h:397-413
    pub fn new_with_definition(
        track_count: u32,
        track_definition: &GridTrackSize,
        is_available_size_indefinite: bool,
    ) -> Self {
        unsafe {
            GridSetNewWithDefinition(track_count, track_definition, is_available_size_indefinite)
        }
    }

    pub fn FlexFactor(&self) -> f32 {
        unsafe { GridSetFlexFactor(self) }
    }

    pub fn BaseSize(&self) -> LayoutUnit {
        unsafe { GridSetBaseSize(self) }
    }

    pub fn GrowthLimit(&self) -> LayoutUnit {
        unsafe { GridSetGrowthLimit(self) }
    }

    pub fn InitBaseSize(&mut self, size: LayoutUnit) {
        unsafe { GridSetInitBaseSize(self, size) }
    }

    pub fn IncreaseBaseSize(&mut self, size: LayoutUnit) {
        unsafe { GridSetIncreaseBaseSize(self, size) }
    }

    pub fn IncreaseGrowthLimit(&mut self, size: LayoutUnit) {
        unsafe { GridSetIncreaseGrowthLimit(self, size) }
    }

    pub fn EnsureGrowthLimitIsNotLessThanBaseSize(&mut self) {
        unsafe { GridSetEnsureGrowthLimitIsNotLessThanBaseSize(self) }
    }

    pub fn IsGrowthLimitLessThanBaseSize(&self) -> bool {
        unsafe { GridSetIsGrowthLimitLessThanBaseSize(self) }
    }
}

// cpp: layoutng/internal/grid_track_collection.h:435-469
pub struct SetIteratorBase<const IS_CONST: bool> {
    track_collection_: *mut GridSizingTrackCollection,
    current_set_index_: u32,
    end_set_index_: u32,
}

impl<const IS_CONST: bool> SetIteratorBase<IS_CONST> {
    pub fn new(
        track_collection: *mut GridSizingTrackCollection,
        begin_set_index: u32,
        end_set_index: u32,
    ) -> Self {
        debug_assert!(!track_collection.is_null());
        debug_assert!(begin_set_index <= end_set_index);
        debug_assert!(end_set_index <= unsafe { &*track_collection }.base.GetSetCount());
        Self {
            track_collection_: track_collection,
            current_set_index_: begin_set_index,
            end_set_index_: end_set_index,
        }
    }

    // cpp: layoutng/internal/grid_track_collection.h:450-459
    pub fn IsAtEnd(&self) -> bool {
        debug_assert!(self.current_set_index_ <= self.end_set_index_);
        self.current_set_index_ == self.end_set_index_
    }

    pub fn MoveToNextSet(&mut self) -> bool {
        self.current_set_index_ = self
            .current_set_index_
            .wrapping_add(1)
            .min(self.end_set_index_);
        self.current_set_index_ < self.end_set_index_
    }
}

impl SetIteratorBase<false> {
    // cpp: layoutng/internal/grid_track_collection.h:461-464
    pub fn CurrentSet(&self) -> *mut GridSet {
        debug_assert!(self.current_set_index_ < self.end_set_index_);
        unsafe { &mut *self.track_collection_ }.GetSetAt(self.current_set_index_)
    }
}

impl SetIteratorBase<true> {
    // cpp: layoutng/internal/grid_track_collection.h:461-464
    pub fn CurrentSet(&self) -> *const GridSet {
        debug_assert!(self.current_set_index_ < self.end_set_index_);
        unsafe { &*self.track_collection_ }.GetSetAtConst(self.current_set_index_)
    }
}

// cpp: layoutng/internal/grid_track_collection.h:474-475
pub type SetIterator = SetIteratorBase<false>;
pub type ConstSetIterator = SetIteratorBase<true>;

// cpp: layoutng/internal/grid_track_collection.h:432-544
#[repr(C)]
pub struct GridSizingTrackCollection {
    pub base: GridLayoutTrackCollection,
    pub(crate) non_collapsed_track_count_: u32,
    pub(crate) sets_: Vector<GridSet>,
}

impl GridSizingTrackCollection {
    // cpp: layoutng/internal/grid_track_collection.h:484-487
    pub fn new(
        ranges: GridRangeVector,
        direction: GridTrackSizingDirection,
        should_store_collapsed_track_indexes: bool,
    ) -> Self {
        let mut collection = unsafe {
            GridSizingTrackCollectionNew(ranges, direction, should_store_collapsed_track_indexes)
        };
        collection.base.is_for_sizing_ = true;
        collection
    }

    pub fn new_with_default_direction(ranges: GridRangeVector) -> Self {
        Self::new(ranges, kForColumns, false)
    }

    // cpp: layoutng/internal/grid_track_collection.h:490-490
    pub fn IsForSizing(&self) -> bool {
        true
    }

    // cpp: layoutng/internal/grid_track_collection.h:492-505
    pub fn GetSetAt(&mut self, set_index: u32) -> *mut GridSet {
        unsafe { GridSizingTrackCollectionGetSetAt(self, set_index) }
    }

    pub fn GetSetAtConst(&self, set_index: u32) -> *const GridSet {
        unsafe { GridSizingTrackCollectionGetSetAtConst(self, set_index) }
    }

    pub fn GetSetIterator(&mut self) -> SetIterator {
        unsafe { GridSizingTrackCollectionGetSetIterator(self) }
    }

    pub fn GetConstSetIterator(&self) -> ConstSetIterator {
        unsafe { GridSizingTrackCollectionGetConstSetIterator(self) }
    }

    pub fn GetSetIteratorRange(&mut self, begin_set_index: u32, end_set_index: u32) -> SetIterator {
        unsafe {
            GridSizingTrackCollectionGetSetIteratorRange(self, begin_set_index, end_set_index)
        }
    }

    // cpp: layoutng/internal/grid_track_collection.h:506-508
    pub fn NonCollapsedTrackCount(&self) -> u32 {
        self.non_collapsed_track_count_
    }

    // cpp: layoutng/internal/grid_track_collection.h:509-539
    pub fn TotalTrackSize(&self) -> LayoutUnit {
        unsafe { GridSizingTrackCollectionTotalTrackSize(self) }
    }

    pub fn BuildSets(&mut self, style: &ComputedStyle, available_size: &LogicalSize) {
        unsafe { GridSizingTrackCollectionBuildSets(self, style, available_size) }
    }

    pub fn SetIndefiniteGrowthLimitsToBaseSize(&mut self) {
        unsafe { GridSizingTrackCollectionSetIndefiniteGrowthLimitsToBaseSize(self) }
    }

    pub fn CacheDefiniteSetsGeometry(&mut self) {
        unsafe { GridSizingTrackCollectionCacheDefiniteSetsGeometry(self) }
    }

    pub fn CacheInitializedSetsGeometry(&mut self, first_set_offset: LayoutUnit) {
        unsafe { GridSizingTrackCollectionCacheInitializedSetsGeometry(self, first_set_offset) }
    }

    pub fn FinalizeSetsGeometry(
        &mut self,
        first_set_offset: LayoutUnit,
        override_gutter_size: LayoutUnit,
    ) {
        unsafe {
            GridSizingTrackCollectionFinalizeSetsGeometry(
                self,
                first_set_offset,
                override_gutter_size,
            )
        }
    }

    // cpp: layoutng/internal/grid_track_collection.h:541-545
    pub fn BuildSetsFromLists(
        &mut self,
        explicit_tracks: &GridTrackList,
        implicit_tracks: &GridTrackList,
        is_grid_lanes: bool,
        is_available_size_indefinite: bool,
    ) {
        unsafe {
            GridSizingTrackCollectionBuildSetsFromLists(
                self,
                explicit_tracks,
                implicit_tracks,
                is_grid_lanes,
                is_available_size_indefinite,
            )
        }
    }

    pub(crate) fn BuildSetsFromListsDefaultAvailableSize(
        &mut self,
        explicit_tracks: &GridTrackList,
        implicit_tracks: &GridTrackList,
        is_grid_lanes: bool,
    ) {
        self.BuildSetsFromLists(explicit_tracks, implicit_tracks, is_grid_lanes, true)
    }

    pub fn InitializeSets(&mut self, grid_available_size: LayoutUnit) {
        unsafe { GridSizingTrackCollectionInitializeSets(self, grid_available_size) }
    }

    pub(crate) fn InitializeSetsDefault(&mut self) {
        self.InitializeSets(kIndefiniteSize)
    }

    // cpp: layoutng/internal/grid_track_collection.h:547-553
    pub fn AllowFrom(collection: &GridLayoutTrackCollection) -> bool {
        collection.IsForSizing()
    }
}

// cpp: layoutng/internal/grid_track_collection.h:308-308
impl foundation::Traceable for GridSizingTrackCollection {
    fn Trace(&self, visitor: &mut foundation::Visitor<'_>) {
        self.base.Trace(visitor);
    }
}

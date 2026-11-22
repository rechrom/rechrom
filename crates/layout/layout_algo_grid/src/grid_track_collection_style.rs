use crate::grid_track_sizing_algorithm::GridTrackSizingAlgorithm;
use foundation::kIndefiniteSize;
use layout_assembly::internal::grid_track_collection::{
    GridRangeBuilder, GridSizingTrackCollection, TrackBoundaryToRangePair,
};
use layout_geometry::geometry::logical_size::LogicalSize;
use layout_style::style::{
    computed_style::ComputedStyle,
    grid_enums::GridTrackSizingDirection::{self, *},
};

// cpp: layoutng_grid/grid_track_collection_style.cc:10-41
pub fn GridRangeBuilderNew(
    style: &ComputedStyle,
    direction: GridTrackSizingDirection,
    repetitions: u32,
    offset: u32,
) -> GridRangeBuilder {
    let mut builder =
        layout_assembly::grid_track_collection_implementation::GridRangeBuilderNewFromTracks(
            style.TemplateTracks(direction).GetTrackList(),
            style.AutoTracks(direction),
            repetitions,
            offset,
        );
    let mut named_end = offset;
    let areas = style.GridTemplateAreas().Get();
    if !areas.is_null() {
        named_end = named_end.wrapping_add(if direction == kForColumns {
            unsafe { &*areas }.column_count
        } else {
            unsafe { &*areas }.row_count
        });
    }
    builder.GridPackageEnsureNamedAreaCoverage(named_end);
    builder
}

// cpp: layoutng_grid/grid_track_collection_style.cc:43-58
pub fn GridSizingTrackCollectionBuildSets(
    this: &mut GridSizingTrackCollection,
    style: &ComputedStyle,
    available: &LogicalSize,
) {
    let direction = this.base.Direction();
    this.base
        .GridPackageSetGutterSize(GridTrackSizingAlgorithm::CalculateGutterSize(
            style, available, direction,
        ));
    let size = if direction == kForColumns {
        available.inline_size
    } else {
        available.block_size
    };
    this.BuildSetsFromLists(
        style.TemplateTracks(direction).GetTrackList(),
        style.AutoTracks(direction),
        style.IsDisplayGridLanes(),
        size == kIndefiniteSize,
    );
    this.InitializeSets(size);
}

// GridRangeBuilder / GridSizingTrackCollection members declared with the base
// types (layoutng/internal/grid_track_collection.h:125-128,509-539) whose
// ComputedStyle bodies are this grid file. Rust cannot add inherent methods to
// a base type from this crate; only this crate calls them.
pub trait GridRangeBuilderStyleAlgorithms: Sized {
    fn new(
        style: &ComputedStyle,
        direction: GridTrackSizingDirection,
        auto_repetitions: u32,
        start_offset: u32,
    ) -> Self;
}

impl GridRangeBuilderStyleAlgorithms for GridRangeBuilder {
    fn new(
        style: &ComputedStyle,
        direction: GridTrackSizingDirection,
        auto_repetitions: u32,
        start_offset: u32,
    ) -> Self {
        GridRangeBuilderNew(style, direction, auto_repetitions, start_offset)
    }
}

pub trait GridSizingTrackCollectionStyleAlgorithms {
    fn BuildSets(&mut self, style: &ComputedStyle, available_size: &LogicalSize);
}

impl GridSizingTrackCollectionStyleAlgorithms for GridSizingTrackCollection {
    fn BuildSets(&mut self, style: &ComputedStyle, available_size: &LogicalSize) {
        GridSizingTrackCollectionBuildSets(self, style, available_size)
    }
}

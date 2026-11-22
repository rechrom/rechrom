//! Extension traits for the members of base-declared grid types whose bodies
//! are grid algorithms (Chromium `core/layout/grid/`). Modules glob-import
//! this so Chromium's member-call syntax stays unchanged; the calls are
//! direct, static calls inside this crate.

pub use crate::grid_item::{GridItemDataAlgorithms, GridItemsAlgorithms};
pub use crate::grid_track_collection_style::{
    GridRangeBuilderStyleAlgorithms, GridSizingTrackCollectionStyleAlgorithms,
};

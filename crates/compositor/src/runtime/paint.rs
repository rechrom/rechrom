//! Paint input values of the compositor. The main-thread bindings (property
//! node handles, PAC/PaintChunksToCcLayer/raster-invalidation projections)
//! and the Skia CPU paint platform live in layerize; this module keeps the impl raster/tiling facets over the shared
//! cc/paint values of `layerize::commit::paint`.

pub use layerize::commit::paint::*;
pub use paint::paint_engine::{
    DisplayItem as PaintOp, PaintArtifact, PaintChunk, RecordedDisplayItem as DisplayItem,
    RecordedDisplayItemKind,
};
mod picture;
pub use picture::*;
mod raster;
pub use raster::*;
mod record_bindings;

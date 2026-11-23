//! Paint input values of the compositor. The main-thread bindings (property
//! node handles, PAC/PaintChunksToCcLayer/raster-invalidation projections)
//! live in layerize (MAIN_IMPL_SPLIT.md S1a-3); this module keeps the impl
//! raster/tiling facets and the CPU platform services.

pub use layerize::runtime::paint::*;
mod picture;
pub use picture::*;
mod raster;
pub use raster::*;
mod artifact_record;
mod record_bindings;
mod services;
pub use services::{
    PaintConfigurationError, SkiaPaintDiagnostics, SkiaPaintPlatform, SkiaPaintValueServices,
    SkiaRasterTrackingServices,
};

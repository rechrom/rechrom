//! cc::RasterSource metadata facet (cc/raster/raster_source.h). The main
//! side creates the RasterSource (RecordingSource::CreateRasterSource) and the
//! impl tilings read it. Moved from compositor_tile::picture_layer_tiling_types,
//! which re-exports it.

use compositor_tree::geometry::{Rect, Size};

/// Immutable resident raster-source metadata; no playback or platform work.
pub trait TilingRasterSource {
    fn IsSolidColor(&self) -> bool;
    fn size(&self) -> Size;
    fn recorded_bounds(&self) -> Rect;
    fn IntersectsRect(&self, rect: Rect) -> bool;
}

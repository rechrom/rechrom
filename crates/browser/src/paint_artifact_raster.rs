//! Raster entry points over a Paint publication. Raster replays recordings
//! (ops plus the resources they sample); the PaintArtifact that holds them
//! is a Paint value the assembly may name, so the projection lives here.

use paint::paint_engine::{DocumentPaintArtifactRecord, PaintArtifact};

/// SVGImage::PaintRecordForCurrentFrame: the record of an SVG image's
/// current-frame artifact.
pub(crate) fn DocumentPaintRecord(
    record: &DocumentPaintArtifactRecord,
) -> raster::DocumentPaintRecord<'_> {
    raster::DocumentPaintRecord {
        ops: record.record.ops(),
        resources: record.record.resources().map(|resources| &**resources),
        record_size: record.record_size,
    }
}

/// Rasterize a Paint publication onto an opaque white RGBA target.
pub fn RasterizePaintArtifact(artifact: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    raster::RasterizePaintOps(&artifact.ReplayOps(), artifact.resources.as_deref(), width, height)
}

/// Rasterize a Paint publication onto an opaque white scaled RGBA target.
pub fn RasterizePaintArtifactWithScale(
    artifact: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
) -> Vec<u8> {
    raster::RasterizePaintOpsWithScale(
        &artifact.ReplayOps(),
        artifact.resources.as_deref(),
        width,
        height,
        scale,
    )
}

/// Rasterize a Paint publication onto a transparent RGBA target.
pub fn RasterizePaintArtifactTransparent(
    artifact: &PaintArtifact,
    width: u32,
    height: u32,
) -> Vec<u8> {
    raster::RasterizePaintOpsTransparent(&artifact.ReplayOps(), artifact.resources.as_deref(), width, height)
}

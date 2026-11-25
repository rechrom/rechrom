//! The record an SVG document image publishes for one container: Blink's
//! SVGImage::PaintRecordForCurrentFrame hands cc a cc::PaintRecord, which
//! Image::PaintImageForCurrentFrame stores in the cc::PaintImage
//! (third_party/blink/renderer/core/svg/graphics/svg_image.cc;
//! cc/paint/paint_image_builder.h set_paint_record). Raster and image decode
//! replay `record`; only the producing document side reads its own
//! recording back through `producer`.

use std::any::Any;
use std::sync::Arc;

use image_resource::{ContainerKey, IntrinsicSize};

use crate::PaintRecord;

#[derive(Clone)]
pub struct DocumentImagePaintRecord {
    pub record: PaintRecord,
    pub intrinsic_size: IntrinsicSize,
    /// Coordinate space used by the nested recording. Responsive SVG may
    /// paint at a container size different from its intrinsic dimensions.
    pub record_size: IntrinsicSize,
    pub container_key: ContainerKey,
    /// The producer's own recording (Paint's PaintArtifact), the same Arc
    /// whose ops `record` leases. Opaque here; Paint and layerize read it.
    pub producer: Arc<dyn Any + Send + Sync>,
}

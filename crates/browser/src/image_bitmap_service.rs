//! Assembly of Image::AsSkBitmapForCurrentFrame with the injected raster owner.
//! The image producer keeps SVG lifecycle state; replay receives a frozen record.
use image_resource::{
    DocumentImageBitmapFrame, ImageBitmapRasterizer, ImmutableImageBitmap, PaintImage,
};
use std::io;

pub(crate) struct RasterImageBitmapService;
impl ImageBitmapRasterizer for RasterImageBitmapService {
    fn Bitmap(&self, image: &PaintImage) -> io::Result<ImmutableImageBitmap> {
        raster::ImageBitmapRespectingOrientation(image)
    }
    fn Document(&self, frame: &DocumentImageBitmapFrame) -> io::Result<ImmutableImageBitmap> {
        let record = image_resource::downcast_document_paint_record::<
            paint::paint_engine::DocumentPaintArtifactRecord,
        >(&frame.frame.record)
        .ok_or_else(|| io::Error::other("SVG image producer published an unknown paint record"))?;
        raster::RasterizeDocumentImageBitmap(record, frame.width, frame.height)
    }
}

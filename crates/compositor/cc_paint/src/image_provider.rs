//! cc/paint image provider values (ImageProvider::ScopedResult,
//! DecodedDrawImage) through which raster playback reaches decoded images.
use super::{CpuDrawImage, CpuFilterQuality, CpuPaintRecord, CpuWorkletInput};
use crate::skia_backend::RecordSkiaBackend;
use compositor_values::shared_image_format::SharedImageFormat;

pub trait CpuDecodedImageLease: Send {
    fn size(&self) -> crate::geometry::Size;
    fn format(&self) -> &SharedImageFormat;
    fn row_bytes(&self) -> usize;
    fn pixels(&self) -> &[u8];
    // Source: cc/paint/decoded_draw_image.h:75-87. These belong to the actual
    // decoded allocation, not guesses from requested size or cache identity.
    fn scale_adjustment(&self) -> crate::geometry::SizeF;
    // Source SkSize is signed: subrect cache entries use (-src.x, -src.y).
    // Vector2dF preserves that contract; gfx::SizeF clamps negatives to zero.
    fn src_rect_offset(&self) -> crate::geometry::Vector2dF;
    fn filter_quality(&self) -> CpuFilterQuality;
}
pub enum CpuScopedRasterResult<R: RecordSkiaBackend> {
    Decoded(Box<dyn CpuDecodedImageLease>),
    Record(CpuPaintRecord<R>),
    Unavailable,
}
pub trait CpuImageProvider<R: RecordSkiaBackend> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<R>;
}
pub trait CpuWorkletImageProvider<R: RecordSkiaBackend> {
    fn GetPaintRecordResult(&mut self, input: &CpuWorkletInput) -> CpuScopedRasterResult<R>;
}
pub struct CpuPlaybackImageProvider<R: RecordSkiaBackend>(pub Box<dyn CpuImageProvider<R> + Send>);
pub struct CpuWorkletProvider<R: RecordSkiaBackend>(pub Box<dyn CpuWorkletImageProvider<R> + Send>);
pub struct RasterImageProviderRef<'a, R: RecordSkiaBackend>(pub &'a mut dyn CpuImageProvider<R>);
impl<R: RecordSkiaBackend> CpuImageProvider<R> for RasterImageProviderRef<'_, R> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<R> {
        self.0.GetRasterContent(image)
    }
}

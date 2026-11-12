//! Canvas2D's source default image-paint receiver. It borrows finalized PAC
//! lists and applies GetCanvasSnapshot's captured map during recursive playback.
//! Chromium base_rendering_context_2d.cc:1496-1568;
//! local_frame_view.cc:222-246, paint_artifact_compositor.cc:63-81,142-164.
use super::*;
use canvas::{canvas_2d::*, child_paint_record::*};
use compositor_core::{
    adapter::{
        geometry::{SizeF, Vector2dF},
        resource_types::{SharedImageFormat, SinglePlaneFormat},
    },
    runtime::{
        canvas_child_record::CanvasChildRecordReplay,
        paint::{CpuDecodedImageLease, CpuImageProvider},
    },
};
use std::collections::BTreeMap;

pub struct Canvas2DRasterBackendFactory;
struct Canvas2DRasterBackend {
    width: u32,
    height: u32,
    alpha: bool,
    pixels: Vec<u8>,
}
impl Canvas2DBackendFactory for Canvas2DRasterBackendFactory {
    fn Create(&self, width: u32, height: u32, alpha: bool) -> Option<Box<dyn Canvas2DBackend>> {
        let mut backend = Canvas2DRasterBackend {
            width: 0,
            height: 0,
            alpha,
            pixels: Vec::new(),
        };
        backend.Reshape(width, height).ok()?;
        Some(Box::new(backend))
    }
}
impl Canvas2DBackend for Canvas2DRasterBackend {
    fn Reshape(&mut self, width: u32, height: u32) -> io::Result<()> {
        i32::try_from(width)
            .and_then(|_| i32::try_from(height))
            .map_err(|_| io::Error::other("Canvas2D dimensions exceed Skia limits"))?;
        let len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or_else(|| io::Error::other("Canvas2D allocation overflow"))?;
        let mut pixels = Vec::new();
        pixels.try_reserve_exact(len).map_err(io::Error::other)?;
        pixels.resize(len, 0);
        if !self.alpha {
            for pixel in pixels.chunks_exact_mut(4) {
                pixel[3] = 255;
            }
        }
        self.width = width;
        self.height = height;
        self.pixels = pixels;
        Ok(())
    }
    fn Playback(&mut self, commands: &[Canvas2DCommand]) -> io::Result<()> {
        if commands.is_empty() || self.width == 0 || self.height == 0 {
            return Ok(());
        }
        let mut target = RasterTarget::new(
            &mut self.pixels,
            self.width as i32,
            self.height as i32,
            self.width as usize * 4,
            PixelFormat::Rgba8888,
            None,
        )
        .map_err(|e| raster_error("Canvas2D target failed", e))?;
        for command in commands {
            replay_element(command, &mut target)?;
        }
        target
            .finish()
            .map_err(|e| raster_error("Canvas2D target completion failed", e))
    }
    fn Snapshot(&self) -> Option<canvas::CanvasPixelSnapshot> {
        (self.width != 0 && self.height != 0).then(|| canvas::CanvasPixelSnapshot {
            width: self.width,
            height: self.height,
            row_bytes: self.width as usize * 4,
            pixels: self.pixels.clone(),
        })
    }
}

struct ImageLease {
    pixels: Arc<Vec<u8>>,
    size: Size,
    format: SharedImageFormat,
    quality: CpuFilterQuality,
}
impl CpuDecodedImageLease for ImageLease {
    fn size(&self) -> Size {
        self.size
    }
    fn format(&self) -> &SharedImageFormat {
        &self.format
    }
    fn row_bytes(&self) -> usize {
        self.size.width() as usize * 4
    }
    fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    fn scale_adjustment(&self) -> SizeF {
        SizeF::new(1., 1.)
    }
    fn src_rect_offset(&self) -> Vector2dF {
        Vector2dF::default()
    }
    fn filter_quality(&self) -> CpuFilterQuality {
        self.quality
    }
}
pub(super) struct CanvasImageProvider<'a> {
    pub(super) frames: Option<&'a BTreeMap<i32, usize>>,
    pub(super) error: Option<io::Error>,
    pub(super) values: Arc<dyn compositor_core::runtime::paint::CpuPaintValueServices<SkRegion>>,
}
impl CpuImageProvider<SkRegion> for CanvasImageProvider<'_> {
    fn GetRasterContent(&mut self, image: &CpuDrawImage) -> CpuScopedRasterResult<SkRegion> {
        if let Some(record) = image.image.0.GetPaintRecord() {
            return CpuScopedRasterResult::Record(CpuPaintRecord::from_source(
                record.record,
                self.values.clone(),
            ));
        }
        let frame = self
            .frames
            .and_then(|frames| frames.get(&image.image.0.stable_id().0))
            .copied()
            .unwrap_or(image.frame_index);
        match image.image.0.GetSwSkImage(frame) {
            Ok(pixels) => CpuScopedRasterResult::Decoded(Box::new(ImageLease {
                pixels,
                size: image.image.0.size(),
                quality: image.filter_quality,
                format: SinglePlaneFormat::BGRA_8888.into(),
            })),
            Err(error) => {
                self.error.get_or_insert(error);
                CpuScopedRasterResult::Unavailable
            }
        }
    }
}
fn replay_element(command: &Canvas2DCommand, target: &mut RasterTarget<'_>) -> io::Result<()> {
    let resolved = command
        .draw
        .record
        .BackendRecord()
        .downcast_ref::<CanvasRecordWithSnapshots>()
        .ok_or_else(|| {
            io::Error::other("Canvas2D child record lacks call-time snapshot resolution")
        })?;
    let source = resolved
        .source
        .BackendRecord()
        .downcast_ref::<CanvasChildRecordReplay>()
        .ok_or_else(|| {
            io::Error::other("Canvas2D child record uses an unsupported backend lease")
        })?;
    let count = target.save_count();
    target.save();
    let transform = &command.transform;
    let values = std::array::from_fn(|i| transform.rc(i % 4, i / 4) as f32);
    target.concat44(&SkM44::ColMajor(&values));
    // DrawElementInternal isolates image content for global alpha. The only
    // admitted composite/filter states are the original src-over/none defaults.
    let result = (|| {
        if command.global_alpha < 1. {
            let mut paint = SkPaint::default();
            paint.setAlphaf(command.global_alpha);
            let ctm = target.local_to_device();
            let mut inverse = SkM44::default();
            if !ctm.invert(&mut inverse) {
                return Ok(());
            }
            let dst = command.draw.destination;
            let bounds = ctm
                .asM33()
                .mapRect_value(&SkRect::MakeXYWH(dst.x, dst.y, dst.width, dst.height));
            if !bounds.isFinite() {
                return Ok(());
            }
            target.save();
            target.concat44(&inverse);
            target
                .save_layer(Some(&bounds), Some(&paint))
                .map_err(|e| raster_error("Canvas2D alpha layer failed", e))?;
            target.concat44(&ctm);
        }
        match command.draw.filter {
            canvas::draw_element_image::ResolvedElementImageFilter::None => {}
        }
        let src = command.draw.source;
        let dst = command.draw.destination;
        target.translate(dst.x, dst.y);
        target.scale(dst.width / src.width, dst.height / src.height);
        target.translate(-src.x, -src.y);
        target.clip_rect(
            &SkRect::MakeXYWH(src.x, src.y, src.width, src.height),
            SkClipOp::kIntersect,
            false,
        );
        // The prefix belongs to ContentLayerClientImpl's record, after the
        // source crop clip and before its original operations.
        target.translate(source.offset.x(), source.offset.y());
        let mut replay = TranslatedCanvas::new(target, false, 1.);
        replay.canvas_snapshots = Some(&resolved.replacements);
        let mut provider = CanvasImageProvider {
            frames: command.draw.animated_image_frame_index_map.as_deref(),
            error: None,
            values: source.list.paint_op_buffer().values().clone(),
        };
        replay.Playback(
            source.list.paint_op_buffer(),
            &mut RasterPlaybackParams {
                raster_inducing_scroll_offsets: None,
                image_provider: Some(&mut provider),
            },
            true,
            None,
        );
        replay.unwind_to_depth(0);
        replay.finish_replay()?;
        if let Some(error) = provider.error {
            return Err(error);
        }
        Ok(())
    })();
    let restore = target
        .restore_to_count(count)
        .map_err(|e| raster_error("Canvas2D image restore failed", e));
    result.and(restore)
}

impl TranslatedCanvas<'_, '_> {
    pub(super) fn replay_canvas_custom_data(&mut self, id: u32) -> io::Result<()> {
        let Some(snapshots) = self.canvas_snapshots else {
            // CustomData is metadata in ordinary compositor layer replay.
            return Ok(());
        };
        match snapshots
            .get(&id)
            .ok_or_else(|| io::Error::other("unresolved Canvas2D CustomData placeholder"))?
        {
            CanvasSnapshotReplacement::NotCanvas | CanvasSnapshotReplacement::EmptyCanvas => Ok(()),
            CanvasSnapshotReplacement::Bitmap(bitmap) => {
                let info = SkImageInfo::Make(
                    bitmap.width as i32,
                    bitmap.height as i32,
                    SkColorType::kRGBA_8888_SkColorType,
                    SkAlphaType::kPremul_SkAlphaType,
                );
                let pixmap =
                    SkPixmap::new(&info, bitmap.pixels.as_ptr(), bitmap.width as usize * 4);
                let image = unsafe {
                    skia::src::image::SkImage_RasterFactories::RasterFromPixmap(
                        &pixmap,
                        None,
                        std::ptr::null_mut(),
                    )
                }
                .ok_or_else(|| io::Error::other("nested canvas snapshot image failed"))?;
                let bounds = SkRect::MakeWH(bitmap.width as f32, bitmap.height as f32);
                // Source GetCanvasSnapshot draws src==dst, src-over, default
                // image sampling, with ImageAutoDarkMode explicitly disabled.
                unsafe {
                    self.target.draw_image_rect(
                        &image,
                        Some(&bounds),
                        &bounds,
                        &SkSamplingOptions::with_filter(SkFilterMode::kLinear),
                        None,
                        SrcRectConstraint::kStrict_SrcRectConstraint,
                    )
                }
                .map_err(|e| raster_error("nested canvas snapshot draw failed", e))
            }
        }
    }
}

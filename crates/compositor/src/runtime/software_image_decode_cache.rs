//! CPU software ImageDecodeCache over the producer's immutable bitmap allocation.
//! Encoded-image producers retain Chromium's lazy-generated fact, so minified
//! playback creates and reuses a mip-sized allocation. True no-cache images keep
//! GetSwSkImage pixels. Unsupported color conversion remains a typed failure.
use super::{host_owners::*, images::*, paint::*, resources::RasterColorSpace};
use crate::{
    adapter::{
        geometry::{Size, SizeF, Vector2dF},
        layer_tree_host_impl::HostImplImageDecodeCache,
        resource_types::SharedImageFormat,
        tile_manager::{ImageDecodeCacheId, ImageTracingInfo},
    },
    cc::{raster::tile_task::TileTaskRef, tiles::tile_manager::RasterCompletion},
    CompositorTypes as B,
};
use std::{
    collections::{HashMap, VecDeque},
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

pub(crate) struct CpuSoftwareImageDecodeFactory {
    registry: ImageDecodeCacheRegistry<B>,
    failure: Arc<dyn RasterFailureHandler>,
    values: Arc<dyn CpuPaintValueServices<super::skia::SkRegion>>,
}
impl CpuSoftwareImageDecodeFactory {
    pub(crate) fn new(
        registry: ImageDecodeCacheRegistry<B>,
        failure: Arc<dyn RasterFailureHandler>,
        values: Arc<dyn CpuPaintValueServices<super::skia::SkRegion>>,
    ) -> Self {
        Self {
            registry,
            failure,
            values,
        }
    }
}
struct SoftwareImages {
    budget: usize,
    failure: Arc<dyn RasterFailureHandler>,
    values: Arc<dyn CpuPaintValueServices<super::skia::SkRegion>>,
    next_request: AtomicU64,
    scaled_images: Mutex<ScaledImageCache>,
}
impl SoftwareImages {
    // Source: cc/tiles/software_image_decode_cache.cc:538-560.
    fn UseCacheForDrawImage(&self, image: &CpuDrawImage) -> bool {
        if image.image.0.GetPaintRecord().is_some() {
            return false;
        }
        if image.image.0.IsLazyGenerated() {
            return true;
        }
        let target = image.target.as_ref().map(|t| &t.color_space);
        target.is_some_and(|c| c.IsValid() && *c != RasterColorSpace::CreateSRGB())
    }
    fn RequiresUnavailableColorConversion(&self, image: &CpuDrawImage) -> bool {
        let target = image.target.as_ref().map(|target| &target.color_space);
        target.is_some_and(|color| color.IsValid() && *color != RasterColorSpace::CreateSRGB())
    }
    fn Reject(&self, message: &'static str) {
        self.failure
            .OnRasterFailure(io::Error::new(io::ErrorKind::Unsupported, message));
    }

    fn ClearScaledImages(&self) {
        *self.scaled_images.lock().unwrap() = ScaledImageCache::default();
    }
}
struct SoftwareImageOwner {
    id: ImageDecodeCacheId,
    registry: ImageDecodeCacheRegistry<B>,
    cache: Arc<SoftwareImages>,
}
impl Drop for SoftwareImageOwner {
    fn drop(&mut self) {
        self.registry.Unregister(self.id);
    }
}
impl HostImplImageDecodeCache for SoftwareImageOwner {
    fn ImageDecodeCacheIdentity(&self) -> ImageDecodeCacheId {
        self.id
    }
    // Source: software_image_decode_cache.cc:647-655.
    fn ClearCache(&mut self) {
        self.cache.ClearScaledImages();
    }
    // Source: software_image_decode_cache.h:56-58, explicit software no-op.
    fn SetShouldAggressivelyFreeResources(&mut self, _aggressive: bool) {}
}
struct SoftwareImageLease {
    size: Size,
    pixels: Arc<Vec<u8>>,
    scale_adjustment: SizeF,
    src_rect_offset: Vector2dF,
    quality: CpuFilterQuality,
}
impl CpuDecodedImageLease for SoftwareImageLease {
    fn size(&self) -> Size {
        self.size
    }
    fn format(&self) -> &SharedImageFormat {
        if skia::include::core::SkColorType::kN32_SkColorType
            == skia::include::core::SkColorType::SkColorType::kBGRA_8888_SkColorType
        {
            &crate::adapter::resource_types::SinglePlaneFormat::BGRA_8888
        } else {
            &crate::adapter::resource_types::SinglePlaneFormat::RGBA_8888
        }
    }
    fn row_bytes(&self) -> usize {
        self.size.width() as usize * 4
    }
    fn pixels(&self) -> &[u8] {
        self.pixels.as_slice()
    }
    fn scale_adjustment(&self) -> SizeF {
        self.scale_adjustment
    }
    fn src_rect_offset(&self) -> Vector2dF {
        self.src_rect_offset
    }
    fn filter_quality(&self) -> CpuFilterQuality {
        self.quality
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum SoftwareRasterPlan {
    Original,
    SubrectAndScale {
        source_rect: crate::adapter::geometry::Rect,
        target_size: Size,
        scale_adjustment: SizeF,
        src_rect_offset: Vector2dF,
    },
}

// Source: cc/tiles/software_image_decode_cache_utils.cc:318-360.
fn RasterPlan(image: &CpuDrawImage) -> SoftwareRasterPlan {
    RasterPlanFor(
        image.image.0.size(),
        image.source_rect,
        image.filter_quality,
        image.scale,
        image.matrix_is_decomposable,
    )
}

fn RasterPlanFor(
    source_size: Size,
    mut source_rect: crate::adapter::geometry::Rect,
    filter_quality: CpuFilterQuality,
    scale: crate::adapter::geometry::Vector2dF,
    matrix_is_decomposable: bool,
) -> SoftwareRasterPlan {
    use compositor_tile::mipmap_util::MipMapUtil;

    source_rect.Intersect(crate::adapter::geometry::Rect::new(
        0,
        0,
        source_size.width(),
        source_size.height(),
    ));
    if source_rect.IsEmpty() {
        return SoftwareRasterPlan::Original;
    }
    let target_size = Size::new(
        (source_rect.width as f32 * scale.x()).abs().round() as i32,
        (source_rect.height as f32 * scale.y()).abs().round() as i32,
    );
    if target_size.width() == 0 || target_size.height() == 0 {
        return SoftwareRasterPlan::Original;
    }
    let mip_level = MipMapUtil::GetLevelForSize(source_rect.size(), target_size);
    if filter_quality == CpuFilterQuality::kNone || mip_level == 0 || !matrix_is_decomposable {
        return SoftwareRasterPlan::Original;
    }
    let target_size = MipMapUtil::GetSizeForLevel(source_rect.size(), mip_level);
    SoftwareRasterPlan::SubrectAndScale {
        source_rect,
        target_size,
        scale_adjustment: MipMapUtil::GetScaleAdjustmentForSize(source_rect.size(), target_size),
        src_rect_offset: Vector2dF::new(-source_rect.x as f32, -source_rect.y as f32),
    }
}

// A Chromium mip target rounds each axis up, while Skia's generated mip chain
// halves each axis.  Only request a mip when both axes land on the same exact
// power-of-two level.  Otherwise mip-nearest can select a smaller allocation
// (for example 247 -> 123) and then enlarge it to Chromium's 124px target.
fn IsExactMipScale(source_size: Size, target_size: Size) -> bool {
    if source_size.width() <= 0
        || source_size.height() <= 0
        || target_size.width() <= 0
        || target_size.height() <= 0
        || source_size.width() % target_size.width() != 0
        || source_size.height() % target_size.height() != 0
    {
        return false;
    }
    let width_scale = source_size.width() / target_size.width();
    let height_scale = source_size.height() / target_size.height();
    width_scale == height_scale && width_scale > 1 && (width_scale as u32).is_power_of_two()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ScaledImageKey {
    stable_id: crate::adapter::image_animation::PaintImageId,
    frame_index: usize,
    source_allocation: usize,
    source_x: i32,
    source_y: i32,
    source_width: i32,
    source_height: i32,
    target_width: i32,
    target_height: i32,
}

struct ScaledImage {
    // Keeping the candidate alive makes source_allocation a collision-free
    // identity for this cache entry even after its PaintRecord is retired.
    _source: Arc<Vec<u8>>,
    pixels: Arc<Vec<u8>>,
    size: Size,
    scale_adjustment: SizeF,
    src_rect_offset: Vector2dF,
}

#[derive(Default)]
struct ScaledImageCache {
    entries: HashMap<ScaledImageKey, Arc<ScaledImage>>,
    lru: VecDeque<ScaledImageKey>,
    bytes: usize,
}

impl ScaledImageCache {
    fn Get(&mut self, key: ScaledImageKey) -> Option<Arc<ScaledImage>> {
        let image = self.entries.get(&key)?.clone();
        self.lru.retain(|candidate| *candidate != key);
        self.lru.push_back(key);
        Some(image)
    }

    fn Insert(&mut self, key: ScaledImageKey, image: Arc<ScaledImage>, budget: usize) {
        let bytes = image.pixels.len();
        if bytes > budget {
            return;
        }
        while self.bytes.saturating_add(bytes) > budget {
            let Some(oldest) = self.lru.pop_front() else {
                break;
            };
            if let Some(old) = self.entries.remove(&oldest) {
                self.bytes = self.bytes.saturating_sub(old.pixels.len());
            }
        }
        if let Some(old) = self.entries.insert(key, image) {
            self.bytes = self.bytes.saturating_sub(old.pixels.len());
            self.lru.retain(|candidate| *candidate != key);
        }
        self.bytes = self.bytes.saturating_add(bytes);
        self.lru.push_back(key);
    }
}

// Source: cc/tiles/software_image_decode_cache_utils.cc:189-263.
fn GenerateScaledImage(
    source: Arc<Vec<u8>>,
    encoded: Option<&[u8]>,
    source_size: Size,
    source_rect: crate::adapter::geometry::Rect,
    target_size: Size,
    scale_adjustment: SizeF,
    src_rect_offset: Vector2dF,
) -> io::Result<Arc<ScaledImage>> {
    use skia::{
        include::core::{
            SkAlphaType::SkAlphaType,
            SkSamplingOptions::{SkFilterMode, SkMipmapMode, SkSamplingOptions},
        },
        src::core::{SkImageInfo::SkImageInfo, SkPixmap::SkPixmap, SkRect::SkIRect},
    };

    let source_row_bytes = source_size.width() as usize * 4;
    let source_len = source_row_bytes
        .checked_mul(source_size.height() as usize)
        .ok_or_else(|| io::Error::other("software image source allocation overflow"))?;
    if source.len() != source_len {
        return Err(io::Error::other(
            "software image source allocation does not match its dimensions",
        ));
    }
    let source_info = SkImageInfo::Make(
        source_size.width(),
        source_size.height(),
        skia::include::core::SkColorType::kN32_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
    );
    let source_pixmap = SkPixmap::new(&source_info, source.as_ptr(), source_row_bytes);
    let mut source_subset = SkPixmap::default();
    if !source_pixmap.extractSubset(
        &mut source_subset,
        &SkIRect::make_xywh(
            source_rect.x,
            source_rect.y,
            source_rect.width,
            source_rect.height,
        ),
    ) {
        return Err(io::Error::other("software image source subset is empty"));
    }

    let target_row_bytes = target_size.width() as usize * 4;
    let target_len = target_row_bytes
        .checked_mul(target_size.height() as usize)
        .ok_or_else(|| io::Error::other("software image target allocation overflow"))?;
    // Source: SoftwareImageDecodeCache::DecodeImageIfNecessary asks a lazy
    // PaintImageGenerator for the exact supported mip allocation before it
    // falls back to GenerateCacheEntryFromCandidate. JPEGImageDecoder exposes
    // libjpeg's 1/8 scale numerators, so preserve that codec path here.
    let codec_target = if source_rect.x == 0
        && source_rect.y == 0
        && source_rect.width == source_size.width()
        && source_rect.height == source_size.height()
    {
        encoded
            .map(|bytes| {
                image_decoder::jpeg_scaled_decode::decode_to_n32(
                    bytes,
                    target_size.width(),
                    target_size.height(),
                )
            })
            .transpose()?
            .flatten()
    } else {
        None
    };
    if let Some(target) = codec_target {
        return Ok(Arc::new(ScaledImage {
            _source: source,
            pixels: Arc::new(target),
            size: target_size,
            scale_adjustment,
            src_rect_offset,
        }));
    }
    let mut target = vec![0; target_len];
    let target_info = SkImageInfo::Make(
        target_size.width(),
        target_size.height(),
        skia::include::core::SkColorType::kN32_SkColorType,
        SkAlphaType::kPremul_SkAlphaType,
    );
    let target_pixmap = SkPixmap::new(&target_info, target.as_mut_ptr(), target_row_bytes);
    // Source PaintFlags::FilterQualityToSkSamplingOptions(kMedium). Chromium's
    // rounded-up cache dimensions can sit between generated mip levels; use
    // the bilinear component directly for that case instead of selecting a
    // smaller mip and scaling it back up.
    let sampling = if IsExactMipScale(source_rect.size(), target_size) {
        SkSamplingOptions::with_filter_mipmap(SkFilterMode::kLinear, SkMipmapMode::kNearest)
    } else {
        SkSamplingOptions::with_filter(SkFilterMode::kLinear)
    };
    if !unsafe { source_subset.scalePixels(&target_pixmap, &sampling) }
        .map_err(|error| io::Error::other(format!("software image scaling failed: {error}")))?
    {
        return Err(io::Error::other("software image scaling was rejected"));
    }
    Ok(Arc::new(ScaledImage {
        _source: source,
        pixels: Arc::new(target),
        size: target_size,
        scale_adjustment,
        src_rect_offset,
    }))
}
impl ImageDecodeCacheBackend<B> for SoftwareImages {
    // Source: cc/tiles/software_image_decode_cache.cc:199-217.
    fn GetTaskForImageAndRef(
        &self,
        image: &CpuDrawImage,
        _tracing: ImageTracingInfo,
    ) -> ImageTaskResult<B> {
        if self.RequiresUnavailableColorConversion(image) {
            self.Reject("non-sRGB software image conversion backend is not available");
        }
        ImageTaskResult {
            task: None,
            need_unref: false,
            is_at_raster_decode: false,
        }
    }
    fn UnrefImage(&self, image: &CpuDrawImage) {
        assert!(
            !self.RequiresUnavailableColorConversion(image),
            "unavailable color conversions cannot hold a cache ref"
        );
    }
    // Source: software_image_decode_cache.cc:647-655.
    fn ReduceCacheUsage(&self) {
        self.ClearScaledImages();
    }
    // Source: software_image_decode_cache.h:62, explicit software no-op.
    fn RecordCacheStats(&self) {}
    fn MaximumMemoryLimitBytes(&self) -> usize {
        self.budget
    }
    fn QueueImageDecode(
        &self,
        image: CpuDrawImage,
        _speculative: bool,
        completion: Box<dyn FnOnce(u64, ImageDecodeResult) + Send>,
    ) -> u64 {
        let request = self.next_request.fetch_add(1, Ordering::Relaxed);
        assert_ne!(request, 0);
        if self.RequiresUnavailableColorConversion(&image) {
            completion(request, ImageDecodeResult::Failure);
        } else {
            completion(request, ImageDecodeResult::DecodeNotRequired);
        }
        request
    }
    fn UnlockImageDecode(&self, _request: u64) {
        panic!("DecodeNotRequired never creates an ImageController locked request")
    }
    fn ExternalDependencyCompletedForTask(&self, _task: TileTaskRef<RasterCompletion<B>>) {
        panic!("the decoded producer allocation requires no decode task")
    }
    fn FlushDecodeTasks(&self) {}
    // Source: cc/raster/playback_image_provider.cc:43-91;
    // cc/tiles/software_image_decode_cache.cc:583-609. Concrete PaintImages
    // return their original allocation; only producers that retain Chromium's
    // lazy-generated fact enter the mip-sized software cache.
    fn GetRasterContent(
        &self,
        image: &CpuDrawImage,
        gpu: bool,
    ) -> CpuScopedRasterResult<super::skia::SkRegion> {
        if let Some(record) = image.image.0.GetPaintRecord() {
            return CpuScopedRasterResult::Record(CpuPaintRecord::from_source(
                record.record,
                self.values.clone(),
            ));
        }
        if gpu || self.RequiresUnavailableColorConversion(image) {
            self.Reject("software image requires an unavailable GPU/color conversion backend");
            return CpuScopedRasterResult::Unavailable;
        }
        match image.image.0.GetSwSkImage(image.frame_index) {
            Ok(pixels) => {
                let use_cache = self.UseCacheForDrawImage(image);
                match if use_cache {
                    RasterPlan(image)
                } else {
                    SoftwareRasterPlan::Original
                } {
                    SoftwareRasterPlan::Original => {
                        CpuScopedRasterResult::Decoded(Box::new(SoftwareImageLease {
                            size: image.image.0.size(),
                            pixels,
                            scale_adjustment: SizeF::new(1., 1.),
                            src_rect_offset: Vector2dF::new(0., 0.),
                            // Source GetDecodedFilterQuality. Cache-owned original
                            // decodes are capped at bilinear; a concrete eager
                            // PaintImage keeps the authored quality unchanged.
                            quality: if use_cache && image.filter_quality != CpuFilterQuality::kNone
                            {
                                CpuFilterQuality::kLow
                            } else {
                                image.filter_quality
                            },
                        }))
                    }
                    SoftwareRasterPlan::SubrectAndScale {
                        source_rect,
                        target_size,
                        scale_adjustment,
                        src_rect_offset,
                    } => {
                        let key = ScaledImageKey {
                            stable_id: image.image.0.stable_id(),
                            frame_index: image.frame_index,
                            source_allocation: Arc::as_ptr(&pixels) as usize,
                            source_x: source_rect.x,
                            source_y: source_rect.y,
                            source_width: source_rect.width,
                            source_height: source_rect.height,
                            target_width: target_size.width(),
                            target_height: target_size.height(),
                        };
                        let cached = self.scaled_images.lock().unwrap().Get(key);
                        let scaled = match cached {
                            Some(scaled) => Ok(scaled),
                            None => GenerateScaledImage(
                                pixels,
                                image.image.0.GetEncodedData().as_deref(),
                                image.image.0.size(),
                                source_rect,
                                target_size,
                                scale_adjustment,
                                src_rect_offset,
                            )
                            .inspect(|scaled| {
                                self.scaled_images.lock().unwrap().Insert(
                                    key,
                                    scaled.clone(),
                                    self.budget,
                                )
                            }),
                        };
                        match scaled {
                            Ok(scaled) => {
                                CpuScopedRasterResult::Decoded(Box::new(SoftwareImageLease {
                                    size: scaled.size,
                                    pixels: scaled.pixels.clone(),
                                    scale_adjustment: scaled.scale_adjustment,
                                    src_rect_offset: scaled.src_rect_offset,
                                    // Source GetDecodedFilterQuality: preprocessing
                                    // has completed, so the final draw is bilinear.
                                    quality: CpuFilterQuality::kLow,
                                }))
                            }
                            Err(error) => {
                                self.failure.OnRasterFailure(error);
                                CpuScopedRasterResult::Unavailable
                            }
                        }
                    }
                }
            }
            Err(error) => {
                self.failure.OnRasterFailure(error);
                CpuScopedRasterResult::Unavailable
            }
        }
    }
}
impl ImageDecodeFactoryBackend for CpuSoftwareImageDecodeFactory {
    fn ToClosestSkColorType(&self, format: SharedImageFormat) -> SkColorType {
        ClosestColorType(format)
    }
    fn CreateGpuImageDecodeCache(
        &self,
        _provider: Option<RasterContextProvider>,
        _color: SkColorType,
        _budget: usize,
        _max: i32,
        _filter: Option<&dyn DarkModeFilterBackend>,
    ) -> Box<ImageDecodeCache> {
        panic!("CPU source branch never creates a GPU image decode cache")
    }
    fn CreateSoftwareImageDecodeCache(
        &self,
        _color: SkColorType,
        budget: usize,
    ) -> Box<ImageDecodeCache> {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = ImageDecodeCacheId(NEXT.fetch_add(1, Ordering::Relaxed));
        assert_ne!(id.0, 0);
        let cache = Arc::new(SoftwareImages {
            budget,
            failure: self.failure.clone(),
            values: self.values.clone(),
            next_request: AtomicU64::new(1),
            scaled_images: Mutex::new(ScaledImageCache::default()),
        });
        self.registry.Register(id, cache.clone());
        Box::new(ImageDecodeCache(Box::new(SoftwareImageOwner {
            id,
            registry: self.registry.clone(),
            cache,
        })))
    }
}

// Source: components/viz/common/resources/shared_image_format_utils.cc:138-176;
// third_party/skia/include/core/SkColorType.h:32-145. Native enum values.
fn ClosestColorType(format: SharedImageFormat) -> SkColorType {
    assert!(format.is_single_plane());
    if format == crate::adapter::resource_types::SinglePlaneFormat::RGBA_4444 {
        return SkColorType(3);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::RGBA_8888 {
        return SkColorType(4);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::BGRA_8888 {
        return SkColorType(6);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::ALPHA_8 {
        return SkColorType(1);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::BGR_565 {
        return SkColorType(2);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::RGBX_8888 {
        return SkColorType(5);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::BGRX_8888 {
        return SkColorType(5);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::ETC1 {
        return SkColorType(5);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::RGBA_1010102 {
        return SkColorType(7);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::BGRA_1010102 {
        return SkColorType(7);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::R_8 {
        return SkColorType(28);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::R_16 {
        return SkColorType(24);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::RG_1616 {
        return SkColorType(25);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::LUMINANCE_F16 {
        return SkColorType(20);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::R_F16 {
        return SkColorType(20);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::RG_88 {
        return SkColorType(19);
    }
    if format == crate::adapter::resource_types::SinglePlaneFormat::RGBA_F16 {
        return SkColorType(16);
    }
    panic!("source NOTREACHED: unrepresentable single-plane image format")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::geometry::{Rect, Vector2dF};

    #[test]
    fn minified_156_by_96_input_selects_the_source_mip_level() {
        let plan = RasterPlanFor(
            Size::new(156, 96),
            Rect::new(0, 0, 156, 96),
            CpuFilterQuality::kMedium,
            Vector2dF::new(45.5 / 156., 28. / 96.),
            true,
        );
        assert_eq!(
            plan,
            SoftwareRasterPlan::SubrectAndScale {
                source_rect: Rect::new(0, 0, 156, 96),
                target_size: Size::new(78, 48),
                scale_adjustment: SizeF::new(0.5, 0.5),
                src_rect_offset: Vector2dF::new(0., 0.),
            }
        );
    }

    #[test]
    fn subrect_plan_preserves_chromium_offset_and_scale_contract() {
        let plan = RasterPlanFor(
            Size::new(156, 96),
            Rect::new(10, 20, 100, 60),
            CpuFilterQuality::kMedium,
            Vector2dF::new(0.3, 0.3),
            true,
        );
        assert_eq!(
            plan,
            SoftwareRasterPlan::SubrectAndScale {
                source_rect: Rect::new(10, 20, 100, 60),
                target_size: Size::new(50, 30),
                scale_adjustment: SizeF::new(0.5, 0.5),
                src_rect_offset: Vector2dF::new(-10., -20.),
            }
        );
        assert_eq!(
            RasterPlanFor(
                Size::new(156, 96),
                Rect::new(0, 0, 156, 96),
                CpuFilterQuality::kNone,
                Vector2dF::new(0.25, 0.25),
                true,
            ),
            SoftwareRasterPlan::Original
        );
    }

    #[test]
    fn scale_fixture_uses_premultiplied_medium_quality_pixels() {
        let mut source = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                source.extend_from_slice(match (x < 2, y < 2) {
                    (true, true) => &[255, 0, 0, 255],
                    (false, true) => &[0, 255, 0, 255],
                    (true, false) => &[0, 0, 255, 255],
                    (false, false) => &[0, 0, 0, 0],
                });
            }
        }
        let image = GenerateScaledImage(
            Arc::new(source),
            None,
            Size::new(4, 4),
            Rect::new(0, 0, 4, 4),
            Size::new(2, 2),
            SizeF::new(0.5, 0.5),
            Vector2dF::new(0., 0.),
        )
        .unwrap();
        assert_eq!(
            image.pixels.as_slice(),
            &[255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 0, 0, 0]
        );
    }

    #[test]
    fn exact_mip_scale_rejects_rounded_up_axes() {
        assert!(IsExactMipScale(Size::new(160, 56), Size::new(80, 28)));
        assert!(IsExactMipScale(Size::new(234, 60), Size::new(117, 30)));
        assert!(IsExactMipScale(Size::new(160, 64), Size::new(40, 16)));
        assert!(!IsExactMipScale(Size::new(247, 64), Size::new(124, 32)));
        assert!(!IsExactMipScale(Size::new(247, 64), Size::new(62, 16)));
    }
}

//! CPU software ImageDecodeCache boundary for the current eager bitmap producer.
//! Source no-cache images retain the actual PaintImage/GetSwSkImage allocation.
//! Inputs requiring a decoder/color conversion are rejected as backend errors;
//! they are never acknowledged with fabricated pixels or decode completion.
use super::{host_owners::*, native_images::*, paint::*, resources::CpuColorSpace};
use crate::{
    adapter::{
        geometry::{Size, SizeF},
        layer_tree_host_impl::HostImplImageDecodeCache,
        resource_types::SharedImageFormat,
        tile_manager::{ImageDecodeCacheId, ImageTracingInfo},
    },
    cc::{raster::tile_task::TileTaskRef, tiles::tile_manager::RasterCompletion},
    NativeCompositorTypes as B,
};
use std::{
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

pub(crate) struct NativeSoftwareImageDecodeFactory {
    registry: NativeImageDecodeCaches<B>,
    failure: Arc<dyn CpuRasterFailureHandler>,
    values: Arc<dyn CpuPaintValueServices<super::skia::SkRegion>>,
}
impl NativeSoftwareImageDecodeFactory {
    pub(crate) fn new(
        registry: NativeImageDecodeCaches<B>,
        failure: Arc<dyn CpuRasterFailureHandler>,
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
    failure: Arc<dyn CpuRasterFailureHandler>,
    values: Arc<dyn CpuPaintValueServices<super::skia::SkRegion>>,
    next_request: AtomicU64,
}
impl SoftwareImages {
    // Source: cc/tiles/software_image_decode_cache.cc:538-564. The admitted CPU
    // image type is a nontexture bitmap; lazy/generated/animated owners have not
    // crossed the Paint input validation boundary.
    fn UseCacheForDrawImage(&self, image: &CpuDrawImage) -> bool {
        if image.image.0.GetPaintRecord().is_some() {
            return false;
        }
        if image.image.0.IsLazyGenerated() {
            return true;
        }
        let target = image.target.as_ref().map(|t| &t.color_space);
        target.is_some_and(|c| c.IsValid() && *c != CpuColorSpace::CreateSRGB())
    }
    fn Reject(&self, message: &'static str) {
        self.failure
            .OnRasterFailure(io::Error::new(io::ErrorKind::Unsupported, message));
    }
}
struct SoftwareImageOwner {
    id: ImageDecodeCacheId,
    registry: NativeImageDecodeCaches<B>,
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
    // Source: software_image_decode_cache.cc:652-655. This supported branch has
    // no decode-cache entries: GetSwSkImage belongs to the original PaintImage.
    fn ClearCache(&mut self) {}
    // Source: software_image_decode_cache.h:56-58, explicit software no-op.
    fn SetShouldAggressivelyFreeResources(&mut self, _aggressive: bool) {}
}
struct SoftwareImageLease {
    image: CpuPaintImage,
    pixels: Arc<Vec<u8>>,
    quality: CpuFilterQuality,
}
impl CpuDecodedImageLease for SoftwareImageLease {
    fn size(&self) -> Size {
        self.image.0.size()
    }
    fn format(&self) -> &SharedImageFormat {
        &crate::adapter::resource_types::SinglePlaneFormat::RGBA_8888
    }
    fn row_bytes(&self) -> usize {
        self.image.0.size().width() as usize * 4
    }
    fn pixels(&self) -> &[u8] {
        self.pixels.as_slice()
    }
    // Source: cc/raster/playback_image_provider.cc:75-80. No-cache playback
    // keeps original pixels, zero source offset and identity scale adjustment.
    fn scale_adjustment(&self) -> SizeF {
        SizeF::new(1., 1.)
    }
    fn src_rect_offset(&self) -> SizeF {
        SizeF::new(0., 0.)
    }
    fn filter_quality(&self) -> CpuFilterQuality {
        self.quality
    }
}
impl NativeImageDecodeCache<B> for SoftwareImages {
    // Source: cc/tiles/software_image_decode_cache.cc:199-217.
    fn GetTaskForImageAndRef(
        &self,
        image: &CpuDrawImage,
        _tracing: ImageTracingInfo,
    ) -> NativeImageTaskResult<B> {
        if self.UseCacheForDrawImage(image) {
            self.Reject("lazy or non-sRGB software image decode backend is not available");
        }
        NativeImageTaskResult {
            task: None,
            need_unref: false,
            is_at_raster_decode: false,
        }
    }
    fn UnrefImage(&self, image: &CpuDrawImage) {
        assert!(
            !self.UseCacheForDrawImage(image),
            "only no-cache inputs are admitted without a ref"
        );
    }
    // Source: software_image_decode_cache.cc:647-655. Only decode-cache entries
    // are evictable; supported eager PaintImage allocations are external owners.
    fn ReduceCacheUsage(&self) {}
    // Source: software_image_decode_cache.h:62, explicit software no-op.
    fn RecordCacheStats(&self) {}
    fn MaximumMemoryLimitBytes(&self) -> usize {
        self.budget
    }
    fn QueueImageDecode(
        &self,
        image: CpuDrawImage,
        _speculative: bool,
        completion: Box<dyn FnOnce(u64, NativeImageDecodeResult) + Send>,
    ) -> u64 {
        let request = self.next_request.fetch_add(1, Ordering::Relaxed);
        assert_ne!(request, 0);
        if self.UseCacheForDrawImage(&image) {
            completion(request, NativeImageDecodeResult::Failure);
        } else {
            completion(request, NativeImageDecodeResult::DecodeNotRequired);
        }
        request
    }
    fn UnlockImageDecode(&self, _request: u64) {
        panic!("DecodeNotRequired never creates an ImageController locked request")
    }
    fn ExternalDependencyCompletedForTask(&self, _task: TileTaskRef<RasterCompletion<B>>) {
        panic!("no decode task exists for a source no-cache image")
    }
    fn FlushDecodeTasks(&self) {}
    // Source: cc/raster/playback_image_provider.cc:43-91. The current native
    // producer selects the genuine GetSwSkImage/no-cache branch. Errors travel
    // through the raster failure capability and prevent publication of a frame.
    fn GetRasterContent(
        &self,
        image: &CpuDrawImage,
        gpu: bool,
    ) -> CpuScopedRasterResult<super::skia::SkRegion> {
        if let Some(record) = image.image.0.GetPaintRecord() {
            return CpuScopedRasterResult::Record(CpuPaintRecord::from_native(
                record,
                self.values.clone(),
            ));
        }
        if gpu || self.UseCacheForDrawImage(image) {
            self.Reject("software image requires an unavailable GPU/lazy/color conversion backend");
            return CpuScopedRasterResult::Unavailable;
        }
        match image.image.0.GetSwSkImage() {
            Ok(pixels) => CpuScopedRasterResult::Decoded(Box::new(SoftwareImageLease {
                image: image.image.clone(),
                pixels,
                quality: image.filter_quality,
            })),
            Err(error) => {
                self.failure.OnRasterFailure(error);
                CpuScopedRasterResult::Unavailable
            }
        }
    }
}
impl NativeImageDecodeFactory for NativeSoftwareImageDecodeFactory {
    fn ToClosestSkColorType(&self, format: SharedImageFormat) -> SkColorType {
        ClosestColorType(format)
    }
    fn CreateGpuImageDecodeCache(
        &self,
        _provider: Option<RasterContextProvider>,
        _color: SkColorType,
        _budget: usize,
        _max: i32,
        _filter: Option<&dyn NativeDarkModeFilter>,
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

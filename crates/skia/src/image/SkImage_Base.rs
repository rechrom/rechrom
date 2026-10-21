// Source: src/image/SkImage_Base.h:25-26
// Genuine C++ incomplete classes: pointer identity only, no layout or constructor.
unsafe extern "C" {
    pub type GrDirectContext;
    pub type GrImageContext;
}
use super::SkImage::SkImageState;
use crate::src::core::{SkImageInfo::SkImageInfo, SkPixelStorage::SkPixelStorageObject};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
// Native abstract Image_Base inheritance is a typed subtrait; this is not a
// constructible metadata-only image. State/accessor/context glue is uncredited.
// Source: src/image/SkImage_Base.h:37-39
pub const kNeedNewImageUniqueID: u32 = 0;
// Source: src/image/SkImage_Base.h:41-42
pub trait SkImage_Base: super::SkImage::SkImage {
    fn base_state(&self) -> &SkImageBaseState;
    // Source: src/image/SkImage_Base.h:101-101
    fn context(&self) -> *mut GrImageContext {
        core::ptr::null_mut()
    }
    // Source: src/image/SkImage_Base.h:104-104
    fn directContext(&self) -> *mut GrDirectContext {
        core::ptr::null_mut()
    }

    // Source: src/image/SkImage_Base.h:54-54
    fn onPeekPixelsCPU(&self, _pixmap: &mut crate::src::core::SkPixmap::SkPixmap) -> bool {
        false
    }
    // Source: src/image/SkImage_Base.h:56-56
    fn onPeekBitmap(&self) -> Option<&crate::src::core::SkBitmap::SkBitmap> {
        None
    }
    // Source: src/image/SkImage_Base.h:58-64
    unsafe fn onReadPixelsCPU(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
        hint: super::SkImage::CachingHint,
    ) -> Result<bool, crate::RasterError>;
    // Source: src/image/SkImage_Base.h:66-67
    // This is the native base virtual's actual false body. It creates no GPU
    // recorder or readback provider; real derived implementations override it.
    fn readPixelsGraphite(
        &self,
        _recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        _dst: &crate::src::core::SkPixmap::SkPixmap,
        _srcX: i32,
        _srcY: i32,
    ) -> bool {
        false
    }
    // Native raster downcast is a borrow of the actual concrete object.
    fn asRaster(&self) -> Option<&super::SkImage_Raster::SkImage_Raster> {
        None
    }
    // Source: src/image/SkImage_Base.h:70-70
    fn onHasMipmaps(&self) -> bool;
    // Source: src/image/SkImage_Base.h:71-71
    fn onIsProtected(&self) -> bool;
    // Source: src/image/SkImage_Base.h:73-73
    fn onPeekMips(&self) -> Option<&crate::src::core::SkMipmap::SkMipmap> {
        None
    }
    // Source: src/image/SkImage_Base.h:75-77
    fn refMips(&self) -> Option<crate::src::core::SkMipmap::SkMipmap> {
        self.onPeekMips().cloned()
    }
    // Source: src/image/SkImage_Base.h:82-87
    // Source: src/image/SkImage_Base.cpp:41-65
    /// Safety: source pixels/decoder lease and callback/context retain their real
    /// native lifetime through the complete synchronous base implementation.
    unsafe fn onAsyncRescaleAndReadPixels(
        &self,
        info: &SkImageInfo,
        origSrcRect: crate::src::core::SkRect::SkIRect,
        rescaleGamma: super::SkImage::RescaleGamma,
        rescaleMode: super::SkImage::RescaleMode,
        callback: super::SkImage::ReadPixelsCallback,
        context: super::SkImage::ReadPixelsContext,
    ) -> Result<(), crate::RasterError> {
        let image: &dyn super::SkImage::SkImage = self.asImageBase();
        let mut src = crate::src::core::SkBitmap::SkBitmap::default();
        let mut peek = crate::src::core::SkPixmap::SkPixmap::default();
        let srcRect;
        if image.peekPixels(Some(&mut peek)) {
            src.installPixmap(&peek);
            srcRect = origSrcRect;
        } else {
            src.setInfo(&image.imageInfo().makeDimensions(origSrcRect.size()), 0);
            src.allocPixels_current();
            if !image.readPixels_default(src.pixmap(), origSrcRect.x(), origSrcRect.y())? {
                callback(context, None);
                return Ok(());
            }
            srcRect = crate::src::core::SkRect::SkIRect::make_size(src.dimensions());
        }
        super::SkRescaleAndReadPixels::SkRescaleAndReadPixels(
            src,
            info,
            &srcRect,
            rescaleGamma,
            rescaleMode,
            callback,
            context,
        )
    }
    // Source: src/image/SkImage_Base.h:91-99
    // Source: src/image/SkImage_Base.cpp:100-112
    /// Safety: callback and context must remain valid through invocation.
    /// The native CPU base itself reports failure for planar async readback.
    /// This is its actual virtual default, not a substitute RGB/YUV conversion.
    unsafe fn onAsyncRescaleAndReadPixelsYUV420(
        &self,
        _yuv_color_space: crate::src::core::SkImageInfo::SkYUVColorSpace,
        _read_alpha: bool,
        _dst_color_space: Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        _src_rect: crate::src::core::SkRect::SkIRect,
        _dst_size: crate::include::core::SkSize::SkISize,
        _rescale_gamma: super::SkImage::RescaleGamma,
        _rescale_mode: super::SkImage::RescaleMode,
        callback: super::SkImage::ReadPixelsCallback,
        context: super::SkImage::ReadPixelsContext,
    ) {
        unsafe {
            callback(context, None);
        }
    }

    /// CPU specialization of native h112-113: the unused GPU context is fixed
    /// to nullptr. Implementors must return actual retained readable pixels on
    /// success; native false remains allocation/read failure, never a stand-in
    /// for a missing implementation.
    fn getROPixelsCPU(
        &self,
        dst: &mut crate::src::core::SkBitmap::SkBitmap,
        hint: super::SkImage::CachingHint,
    ) -> bool;
    // Source: src/image/SkImage_Base.h:108-108
    /// Native base virtual intentionally does nothing; CPU subclasses inherit
    /// this exact behavior. This is not a missing-backend fallback.
    /// Safety: preserve the originating Surface/Image mutation lease during the
    /// notification, including any genuine derived override's native state.
    unsafe fn generatingSurfaceIsDeleted(&self) {}
    // Source: src/image/SkImage_Base.h:115-115
    fn onMakeSubsetCPU(
        &self,
        recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        subset: &crate::src::core::SkRect::SkIRect,
        props: super::SkImage::RequiredProperties,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError>;
    // Source: src/image/SkImage_Base.h:117-117
    // Actual native base default. Raster retains no original encoded payload;
    // encoded/generator-derived images must supply their own real override.
    fn onRefEncoded(&self) -> Option<Arc<crate::src::core::SkData::SkData>> {
        None
    }
    // Source: src/image/SkImage_Base.h:119-119
    unsafe fn onAsLegacyBitmapCPU(
        &self,
        bitmap: &mut crate::src::core::SkBitmap::SkBitmap,
    ) -> Result<bool, crate::RasterError> {
        unsafe { as_legacy_bitmap(self, bitmap) }
    }
    // Source: src/image/SkImage_Base.h:123-123
    unsafe fn onMakeSurfaceCPU<'rec>(
        &self,
        recorder: Option<&'rec dyn crate::include::core::SkRecorder::SkRecorder>,
        info: &SkImageInfo,
    ) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface::SkSurface<'rec> + 'rec>>;
    // Source: src/image/SkImage_Base.h:137-137
    fn type_(&self) -> Type;
    // Source: src/image/SkImage_Base.h:140-143
    fn isLazyGeneratedCPU(&self) -> bool {
        self.type_() == Type::kLazy
            || self.type_() == Type::kLazyPicture
            || self.type_() == Type::kLazyTexture
    }
    // Source: src/image/SkImage_Base.h:145-147
    fn isRasterBacked(&self) -> bool {
        self.type_() == Type::kRaster || self.type_() == Type::kRasterPinnable
    }
    // Source: src/image/SkImage_Base.h:150-152
    fn isGaneshBacked(&self) -> bool {
        self.type_() == Type::kGanesh || self.type_() == Type::kGaneshYUVA
    }
    // Source: src/image/SkImage_Base.h:155-157
    fn isGraphiteBacked(&self) -> bool {
        self.type_() == Type::kGraphite || self.type_() == Type::kGraphiteYUVA
    }
    // Source: src/image/SkImage_Base.h:159-161
    fn isYUVA(&self) -> bool {
        self.type_() == Type::kGaneshYUVA || self.type_() == Type::kGraphiteYUVA
    }
    // Source: src/image/SkImage_Base.h:163-165
    fn isTextureBackedCPU(&self) -> bool {
        self.isGaneshBacked() || self.isGraphiteBacked()
    }
    // Source: src/image/SkImage_Base.h:173-173
    unsafe fn onReinterpretColorSpaceCPU(
        &self,
        target: Arc<crate::src::core::SkColorSpace::SkColorSpace>,
    ) -> Option<Arc<dyn super::SkImage::SkImage>>;
    // Source: src/image/SkImage_Base.h:177-179
    unsafe fn onMakeWithMipmapsCPU(
        &self,
        _mips: Option<crate::src::core::SkMipmap::SkMipmap>,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
        Ok(None)
    }

    // Forward the existing source-mapped native state mutation exactly once.
    // Raster overrides this with its genuine PixelRef cache notification.
    fn notifyAddedToRasterCache(&self) {
        self.base_state().notifyAddedToRasterCache();
    }
}
// Source: src/image/SkImage_Base.h:125-135
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kRaster,
    kRasterPinnable,
    kLazy,
    kLazyPicture,
    kLazyTexture,
    kGanesh,
    kGaneshYUVA,
    kGraphite,
    kGraphiteYUVA,
}
impl SkImageBaseState {
    // Source: src/image/SkImage_Base.h:169-171
    pub fn notifyAddedToRasterCache(&self) {
        self.fAddedToRasterCache.store(true, Ordering::SeqCst);
    }
}
impl SkImageBaseState {
    // Source: src/image/SkImage_Base.h:181-183
    pub fn getPixelStorages(&self) -> &[Option<Arc<dyn SkPixelStorageObject>>] {
        &self.fPixelStorages
    }
}

// Source: src/image/SkImage_Base.h:192-192
// Rust field order releases retained pixel storage before native SkImage base fields, matching C++ derived-before-base destruction.
// Temporary Vec substitutes STArray<1>; field/push allocation operations are adapted, not counted as literal ported source. Canonical container dependency is recorded.
pub struct SkImageBaseState {
    pub(super) fPixelStorages: Vec<Option<Arc<dyn SkPixelStorageObject>>>,
    fAddedToRasterCache: AtomicBool,
    pub(super) image: SkImageState,
}
// Source: src/image/SkImage_Base.h:186-186
// Source: src/image/SkImage_Base.cpp:26-30
// Source: src/image/SkImage_Base.cpp:32-33
impl SkImageBaseState {
    pub(crate) fn new(
        info: &SkImageInfo,
        id: u32,
        storage: Option<Arc<dyn SkPixelStorageObject>>,
    ) -> Self {
        let mut out = Self {
            fPixelStorages: Vec::new(),
            fAddedToRasterCache: AtomicBool::new(false),
            image: SkImageState::new(info, id),
        };
        if let Some(storage) = storage {
            out.fPixelStorages.push(Some(storage));
        }
        out
    }
}
// Source: src/image/SkImage_Base.h:43-43
// Source: src/image/SkImage_Base.cpp:35-39
impl Drop for SkImageBaseState {
    fn drop(&mut self) {
        if self.fAddedToRasterCache.load(Ordering::SeqCst) {
            crate::src::core::SkBitmapCache::SkNotifyBitmapGenIDIsStale(self.image.fUniqueID);
        }
        // Native destructor body is complete. Release STArray members in their
        // original forward element order explicitly; Vec's own Drop order is not a contract.
        // This remains part of the adapted storage representation, before SkImage base teardown.
        for storage in self.fPixelStorages.drain(..) {
            drop(storage);
        }
    }
}

// Source: src/image/SkImage_Base.cpp:67-83
/// Safety: the original image retains initialized readable pixels throughout
/// copy; the output's new allocation is exclusively writable until return.
pub(crate) unsafe fn as_legacy_bitmap<I: SkImage_Base + ?Sized>(
    image: &I,
    bitmap: &mut crate::src::core::SkBitmap::SkBitmap,
) -> Result<bool, crate::RasterError> {
    let info = image
        .imageInfo()
        .makeColorType(crate::include::core::SkColorType::kN32_SkColorType)
        .makeColorSpace(None);
    if !bitmap.tryAllocPixels(&info, 0) {
        return Ok(false);
    }
    let success = unsafe {
        image.onReadPixelsCPU(
            bitmap.info(),
            bitmap.getPixels().cast(),
            bitmap.rowBytes(),
            0,
            0,
            super::SkImage::CachingHint::kAllow_CachingHint,
        )
    };
    match success {
        Ok(true) => {}
        Ok(false) => {
            bitmap.reset();
            return Ok(false);
        }
        Err(error) => {
            bitmap.reset();
            return Err(error);
        }
    }
    bitmap.setImmutable();
    Ok(true)
}

// Source: src/image/SkImage_Base.cpp:85-98
pub(crate) fn make_subset(
    image: &dyn SkImage_Base,
    recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
    subset: &crate::src::core::SkRect::SkIRect,
    props: super::SkImage::RequiredProperties,
) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
    if subset.is_empty() {
        return Ok(None);
    }
    let bounds = crate::src::core::SkRect::SkIRect::make_wh(image.width(), image.height());
    if !bounds.contains_rect(subset) {
        return Ok(None);
    }
    image.onMakeSubsetCPU(recorder, subset, props)
}
// Source: src/image/SkImage_Base.cpp:113-118
pub(crate) fn make_color_space(
    image: Arc<dyn super::SkImage::SkImage>,
    recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
    target: Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    props: super::SkImage::RequiredProperties,
) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
    let color_type = image.imageInfo().colorType();
    image.makeColorTypeAndColorSpace(recorder, color_type, target, props)
}

// Source: src/image/SkImage_Base.h:203-205
// Genuine derived-owner upcast; retain the original borrowed allocation/vtable
// projection supplied by its implementation, never reinterpret trait metadata.
pub fn as_IB(image: &dyn super::SkImage::SkImage) -> &dyn SkImage_Base {
    image.asImageBase()
}
// Source: src/image/SkImage_Base.h:207-209
pub fn as_IB_shared(image: &Arc<dyn super::SkImage::SkImage>) -> &dyn SkImage_Base {
    image.asImageBase()
}

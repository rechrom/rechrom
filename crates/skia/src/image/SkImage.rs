use crate::include::core::{
    SkAlphaType::SkAlphaType, SkColorType::SkColorType, SkRecorder::SkRecorder, SkSize::SkISize,
};
use crate::src::core::{SkColorSpace::SkColorSpace, SkImageInfo::SkImageInfo, SkRect::SkIRect};
use std::sync::Arc;
// Source: include/core/SkImage.h:45-45
pub mod SkImages {
    // Source: include/core/SkImage.h:48-48
    pub type ReleaseContext = *mut core::ffi::c_void;
    // Source: include/core/SkImage.h:52-52
    pub type RasterReleaseProc = unsafe extern "C" fn(*const core::ffi::c_void, ReleaseContext);
    // Source: include/core/SkImage.h:68-68
    pub use crate::src::image::SkImage_RasterFactories::RasterFromBitmap;
    // Source: include/core/SkImage.h:81-84
    pub use crate::src::image::SkImage_RasterFactories::RasterFromCompressedTextureData;
    // Source: include/core/SkImage.h:106-107
    // The real encoded-data factory transfers the same Data owner through the
    // installed native callback and actual Codec fallback into the lazy image.
    pub use crate::src::codec::SkImageGenerator_FromEncoded::DeferredFromEncodedData;
    // Native default std::nullopt; no alternate image or generator allocation.
    pub unsafe fn DeferredFromEncodedData_default(
        encoded: Option<std::sync::Arc<crate::src::core::SkData::SkData>>,
    ) -> Option<std::sync::Arc<dyn super::SkImage>> {
        unsafe { DeferredFromEncodedData(encoded, None) }
    }
    // Source: include/core/SkImage.h:121-121
    // Public projection of the existing actual generator factory. This adds no
    // new source implementation credit and creates no alternate lazy owner.
    pub use crate::src::image::SkImage_Lazy::DeferredFromGenerator;

    // Source: include/core/SkImage.h:123-125
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum BitDepth {
        kU8,
        kF16,
    }
    // Source: include/core/SkImage.h:145-151
    pub use crate::src::image::SkImage_LazyFactories::DeferredFromPicture_with_props;
    // Source: include/core/SkImage.h:152-157
    pub use crate::src::image::SkImage_LazyFactories::DeferredFromPicture;
    // Source: include/core/SkImage.h:174-174
    pub use crate::src::image::SkImage_RasterFactories::RasterFromPixmapCopy;
    // Source: include/core/SkImage.h:196-198
    pub use crate::src::image::SkImage_RasterFactories::RasterFromPixmap;
    // Source: include/core/SkImage.h:215-217
    pub use crate::src::image::SkImage_RasterFactories::RasterFromData;
    // Source: include/core/SkImage.h:240-245
    pub use crate::src::image::SkImage_RasterFactories::MakeWithFilter;
}

// Rust forms of native default arguments/overloads. These preserve the actual
// retained Arc owner and native producer; no independent pixels or shader state.
impl dyn SkImage + 'static {
    // Source: include/core/SkImage.h:384-385
    pub fn makeShader_default(
        self: &Arc<Self>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        self.makeShaderClamp(sampling, None)
    }
    // Source: include/core/SkImage.h:406-407
    pub fn makeRawShader_default(
        self: &Arc<Self>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        self.makeRawShaderClamp(sampling, None)
    }
    // Source: include/core/SkImage.h:710-711
    /// Safety: source pixels and the exclusive writable dst layout retain their
    /// original read/scaler leases through the real synchronous operation.
    pub unsafe fn scalePixels_default(
        &self,
        dst: &crate::src::core::SkPixmap::SkPixmap,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.scalePixels(dst, sampling, CachingHint::kAllow_CachingHint) }
    }
    // Source: include/core/SkImage.h:730-730
    pub unsafe fn makeScaled(
        self: &Arc<Self>,
        info: &SkImageInfo,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        unsafe { self.makeScaledCPU(info, sampling) }
    }
    // Source: include/core/SkImage.h:806-806
    pub unsafe fn makeNonTextureImage(
        self: &Arc<Self>,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        unsafe { self.makeNonTextureImageCPU() }
    }
    // Source: include/core/SkImage.h:825-827
    /// CPU specialization of the exact native nullptr-context forwarding overload.
    pub unsafe fn makeRasterImage(
        self: &Arc<Self>,
        hint: CachingHint,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        unsafe { self.makeRasterImageCPU(hint) }
    }
    // Default argument of h825; keep the native disallow-cache selection.
    pub unsafe fn makeRasterImage_default(
        self: &Arc<Self>,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        unsafe { self.makeRasterImage(CachingHint::kDisallow_CachingHint) }
    }
}
impl dyn SkImage + '_ {
    // Source: include/core/SkImage.h:551-552
    /// Safety: destination allocation covers the writable native layout,
    /// remains exclusive, and source backing remains immutable/readable.
    pub unsafe fn readPixels_info_default(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            self.readPixels_info_legacy(
                info,
                pixels,
                row_bytes,
                x,
                y,
                CachingHint::kAllow_CachingHint,
            )
        }
    }
    // Source: include/core/SkImage.h:553-554
    pub unsafe fn readPixels_default(
        &self,
        dst: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.readPixels_legacy(dst, x, y, CachingHint::kAllow_CachingHint) }
    }
}

// Source: include/core/SkImage.h:270-270
// Native public name reexports the same complete required image interface.
pub use ImageVirtuals as SkImage;
// Header-separated nested value types require two ordered trait sections; this
// is interface grouping only, with no extra object/provider or invented defaults.
// Native abstract public class uses trait dispatch; its actual retained base fields live in SkImageState.
// State alone does not implement SkImage and cannot create a metadata-only image object.
// Source: include/core/SkImage.h:270. SkImage's actual parent is SkRefCnt;
// neither the public class declaration nor its generator factory imposes a
// universal Rust Send/Sync implementation on every derived backing graph.
// Keep the genuine image usable locally. A cross-thread consumer must require
// an actual Send+Sync-qualified complete owner; Arc alone does not prove that.
// This removes an unsupported blanket requirement rather than asserting one
// for the non-Send unique Codec/Stream/custom-generator graph.
pub trait ImageCore {
    fn state(&self) -> &SkImageState;
    // Adapted/uncredited native inheritance upcast: required on genuine image
    // implementors, never fabricated by a trait-object pointer cast.
    fn asImageBase(&self) -> &dyn crate::src::image::SkImage_Base::SkImage_Base;
    // Source: include/core/SkImage.h:277-277
    fn imageInfo(&self) -> &SkImageInfo {
        &self.state().fInfo
    }
    // Source: include/core/SkImage.h:283-283
    fn width(&self) -> i32 {
        self.state().fInfo.width()
    }
    // Source: include/core/SkImage.h:289-289
    fn height(&self) -> i32 {
        self.state().fInfo.height()
    }
    // Source: include/core/SkImage.h:295-295
    fn dimensions(&self) -> SkISize {
        SkISize::Make(self.state().fInfo.width(), self.state().fInfo.height())
    }
    // Source: include/core/SkImage.h:301-301
    fn bounds(&self) -> SkIRect {
        SkIRect::make_wh(self.state().fInfo.width(), self.state().fInfo.height())
    }
    // Source: include/core/SkImage.h:309-309
    fn uniqueID(&self) -> u32 {
        self.state().fUniqueID
    }
    // Source: include/core/SkImage.h:320-320
    fn alphaType(&self) -> SkAlphaType {
        image_alpha_type(self)
    }
    // Source: include/core/SkImage.h:328-328
    fn colorType(&self) -> SkColorType {
        image_color_type(self)
    }
    // Source: include/core/SkImage.h:342-342
    fn colorSpace(&self) -> Option<&SkColorSpace> {
        image_color_space(self)
    }
    // Source: include/core/SkImage.h:358-358
    fn refColorSpace(&self) -> Option<Arc<SkColorSpace>> {
        image_ref_color_space(self)
    }
    // Source: include/core/SkImage.h:367-367
    fn isAlphaOnly(&self) -> bool {
        image_is_alpha_only(self)
    }
    // Source: include/core/SkImage.h:373-373
    fn isOpaque(&self) -> bool {
        crate::include::core::SkAlphaType::SkAlphaTypeIsOpaque(self.state().fInfo.alphaType())
    }
    // Source: include/core/SkImage.h:427-427
    fn isTextureBacked(&self) -> bool;
    // Source: include/core/SkImage.h:432-432
    fn textureSize(&self) -> usize;
    // Source: include/core/SkImage.h:447-447
    fn isValid(&self, recorder: Option<&dyn SkRecorder>) -> bool;
}
// Source: include/core/SkImage.h:461-464
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CachingHint {
    kAllow_CachingHint,
    kDisallow_CachingHint,
}
// A boxed trait result preserves native unique ownership and virtual destruction.
// No concrete async result or byte-storage provider is synthesized here. The interface
// exposes only const observers, matching unique_ptr<const AsyncReadResult> in callbacks.
// Source: include/core/SkImage.h:560-563
// The actual zero-state native base is !Unpin and neither Clone nor Copy.
// The callback transfers Pin<Box<dyn AsyncReadResult>>; moving that owning
// pointer never moves the complete result allocation or permits extraction.
pub struct AsyncReadResultBase {
    _pin: core::marker::PhantomPinned,
}
impl AsyncReadResultBase {
    // Source: include/core/SkImage.h:580-580
    pub(crate) const fn new() -> Self {
        Self {
            _pin: core::marker::PhantomPinned,
        }
    }
}
// Source: include/core/SkImage.h:565-565
// Virtual default base destruction is provided by the actual pinned Box's
// dynamic drop; this zero-state base adds no counter or separate allocation.
// Source: include/core/SkImage.h:558-559
pub trait AsyncReadResult {
    // Structural projection of the real embedded native base, not a new owner.
    fn async_base(&self) -> &AsyncReadResultBase;
    // Source: include/core/SkImage.h:567-567
    fn count(&self) -> i32;
    // Source: include/core/SkImage.h:575-575
    fn data(&self, i: i32) -> *const std::ffi::c_void;
    // Source: include/core/SkImage.h:577-577
    fn rowBytes(&self, i: i32) -> usize;
}
// Source: include/core/SkImage.h:584-584
pub type ReadPixelsContext = *mut std::ffi::c_void;
// Source: include/core/SkImage.h:589-589
pub type ReadPixelsCallback =
    unsafe fn(ReadPixelsContext, Option<core::pin::Pin<Box<dyn AsyncReadResult>>>);
// Source: include/core/SkImage.h:591-591
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RescaleGamma {
    kSrc,
    kLinear,
}
// Source: include/core/SkImage.h:593-598
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum RescaleMode {
    kNearest,
    kLinear,
    kRepeatedLinear,
    kRepeatedCubic,
}
// Source: include/core/SkImage.h:744-745
// Source: include/core/SkImage.h:747-749
// Source: include/core/SkImage.h:751-751
// Source: include/core/SkImage.h:753-756
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequiredProperties {
    pub fMipmapped: bool,
}
// Required native late virtuals remain required; their real CPU implementations
// belong to genuine derived images, never metadata-only image state.
pub trait ImageVirtuals: ImageCore {
    // Source: include/core/SkImage.h:776-778
    fn makeSubset(
        self: Arc<Self>,
        recorder: Option<&dyn SkRecorder>,
        subset: &SkIRect,
        properties: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError>;
    // Source: include/core/SkImage.h:857-857
    fn isLazyGenerated(&self) -> bool;
    // Source: include/core/SkImage.h:873-875
    fn makeColorSpace(
        self: Arc<Self>,
        recorder: Option<&dyn SkRecorder>,
        target: Option<Arc<SkColorSpace>>,
        properties: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError>;
    // Source: include/core/SkImage.h:891-894
    fn makeColorTypeAndColorSpace(
        self: Arc<Self>,
        recorder: Option<&dyn SkRecorder>,
        targetType: SkColorType,
        target: Option<Arc<SkColorSpace>>,
        properties: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError>;
}
// Source: include/core/SkImage.h:910-911
pub struct SkImageState {
    pub(super) fInfo: SkImageInfo,
    pub(super) fUniqueID: u32,
}
// Source: include/core/SkImage.h:903-903
// Source: src/image/SkImage.cpp:32-37
impl SkImageState {
    pub(crate) fn new(info: &SkImageInfo, uniqueID: u32) -> Self {
        let out = Self {
            fInfo: info.clone(),
            fUniqueID: if uniqueID == super::SkImage_Base::kNeedNewImageUniqueID {
                crate::src::core::SkNextID::SkNextID::ImageID()
            } else {
                uniqueID
            },
        };
        crate::SkASSERT!(@source b"src/image/SkImage.cpp\0".as_ptr(),35; @condition b"info.width() > 0\0".as_ptr(); info.width()>0);
        crate::SkASSERT!(@source b"src/image/SkImage.cpp\0".as_ptr(),36; @condition b"info.height() > 0\0".as_ptr(); info.height()>0);
        out
    }
}
// Source: include/core/SkImage.h:418-418
// Source: src/image/SkImage.cpp:38-45
impl dyn SkImage + '_ {
    /// The pixmap is a borrowed raw view; later pixel access must retain this
    /// image and its immutable backing lease, just as for native peekPixels.
    pub fn peekPixels(&self, pixmap: Option<&mut crate::src::core::SkPixmap::SkPixmap>) -> bool {
        let mut temporary = crate::src::core::SkPixmap::SkPixmap::default();
        self.asImageBase()
            .onPeekPixelsCPU(pixmap.unwrap_or(&mut temporary))
    }
}

// Source: include/core/SkImage.h:502-507
// Source: src/image/SkImage.cpp:47-50
impl dyn SkImage + '_ {
    /// Safety: source is live and initialized; destination covers the declared
    /// writable native layout with exclusive access through the complete copy.
    pub unsafe fn readPixels_info(
        &self,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            self.asImageBase()
                .onReadPixelsCPU(info, pixels, row_bytes, x, y, hint)
        }
    }
}
// Source: include/core/SkImage.h:730-730
// Source: src/image/SkImage.cpp:52-55
impl dyn SkImage + 'static {
    /// Safety: same native source/CPU Surface leases as the full overload below.
    pub unsafe fn makeScaledCPU(
        self: &Arc<Self>,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        unsafe { self.makeScaledWithRecorderCPU(None, info, sampling) }
    }
}
// Source: include/core/SkImage.h:724-724
// Source: src/image/SkImage.cpp:57-61
impl dyn SkImage + 'static {
    /// Safety: supplied recorder is an actual CPU recorder and stays live until
    /// the newly created Surface and all its Canvas operations have completed.
    pub unsafe fn makeScaledWithRecorderCPU(
        self: &Arc<Self>,
        recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        unsafe {
            self.makeScaledWithRecorderAndPropsCPU(
                recorder,
                info,
                sampling,
                &crate::include::core::SkSurfaceProps::SkSurfaceProps::default(),
            )
        }
    }
}
// Source: include/core/SkImage.h:725-728
// Source: src/image/SkImage.cpp:63-86
impl dyn SkImage + 'static {
    /// Safety: readable immutable source and exclusive destination backing obey
    /// native pixel/filter/COW leases. No source alias may race these operations.
    pub unsafe fn makeScaledWithRecorderAndPropsCPU(
        self: &Arc<Self>,
        recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        info: &crate::src::core::SkImageInfo::SkImageInfo,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        _props: &crate::include::core::SkSurfaceProps::SkSurfaceProps,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        if !crate::src::core::SkImageInfoPriv::SkImageInfoIsValid(info) {
            return Ok(None);
        }
        if info == self.imageInfo() {
            return Ok(Some(self.clone()));
        }
        let surface = unsafe { self.asImageBase().onMakeSurfaceCPU(recorder, info) };
        let Some(surface) = surface else {
            return Ok(None);
        };
        let mut paint = crate::src::core::SkPaint::SkPaint::default();
        paint.setBlendMode(crate::src::core::SkBlendMode::SkBlendMode::kSrc);
        let owner = surface
            .as_ptr()
            .expect("native successful surface allocation");
        let canvas = unsafe { super::SkSurface::getCanvas_raw(owner) }
            .expect("native complete surface has Canvas");
        let dst = crate::src::core::SkRect::SkRect::MakeIWH(info.width(), info.height());
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::drawImageRect_raw(
                canvas,
                self,
                None,
                &dst,
                sampling,
                Some(&paint),
                crate::src::core::SkCanvas::SrcRectConstraint::kFast_SrcRectConstraint,
            )
        }?;
        unsafe { super::SkSurface::makeImageSnapshot_raw(owner) }
    }
}
// Source: include/core/SkImage.h:551-552
// Source: src/image/SkImage.cpp:89-93
// CPU selection fixes the native legacy overload's saved directContext to null.
impl dyn SkImage + '_ {
    /// Safety: source initialized pixels and the exclusive destination allocation
    /// cover the declared image layout through the complete native transfer.
    pub unsafe fn readPixels_info_legacy(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.readPixels_info(info, pixels, row_bytes, x, y, hint) }
    }
}

// Source: include/core/SkImage.h:633-638
// Source: src/image/SkImage.cpp:96-109
impl dyn SkImage + '_ {
    /// Safety: original immutable source backing and callback/context remain live
    /// through the complete synchronous CPU dispatch. Backend errors are returned.
    pub unsafe fn asyncRescaleAndReadPixels(
        &self,
        info: &SkImageInfo,
        srcRect: &SkIRect,
        rescaleGamma: RescaleGamma,
        rescaleMode: RescaleMode,
        callback: ReadPixelsCallback,
        context: ReadPixelsContext,
    ) -> Result<(), crate::RasterError> {
        if !SkIRect::make_wh(self.width(), self.height()).contains_rect(srcRect)
            || !crate::src::core::SkImageInfoPriv::SkImageInfoIsValid(info)
        {
            callback(context, None);
            return Ok(());
        }
        self.asImageBase().onAsyncRescaleAndReadPixels(
            info,
            *srcRect,
            rescaleGamma,
            rescaleMode,
            callback,
            context,
        )
    }
}

impl dyn SkImage + '_ {
    // Source: include/core/SkImage.h:669-676
    // Source: src/image/SkImage.cpp:111-133
    /// Safety: callback/context are valid through dispatch; the image and its
    /// backing remain live and immutable through any selected virtual operation.
    pub unsafe fn asyncRescaleAndReadPixelsYUV420(
        &self,
        yuv_color_space: crate::src::core::SkImageInfo::SkYUVColorSpace,
        dst_color_space: Option<Arc<SkColorSpace>>,
        src_rect: &SkIRect,
        dst_size: &SkISize,
        rescale_gamma: RescaleGamma,
        rescale_mode: RescaleMode,
        callback: ReadPixelsCallback,
        context: ReadPixelsContext,
    ) {
        if !SkIRect::make_wh(self.width(), self.height()).contains_rect(src_rect)
            || dst_size.isZero()
            || (dst_size.width() & 1) != 0
            || (dst_size.height() & 1) != 0
        {
            unsafe {
                callback(context, None);
            }
            return;
        }
        unsafe {
            self.asImageBase().onAsyncRescaleAndReadPixelsYUV420(
                yuv_color_space,
                false,
                dst_color_space,
                *src_rect,
                *dst_size,
                rescale_gamma,
                rescale_mode,
                callback,
                context,
            );
        }
    }

    // Source: include/core/SkImage.h:683-690
    // Source: src/image/SkImage.cpp:135-157
    /// Safety: same callback and immutable image leases as the YUV entry.
    pub unsafe fn asyncRescaleAndReadPixelsYUVA420(
        &self,
        yuv_color_space: crate::src::core::SkImageInfo::SkYUVColorSpace,
        dst_color_space: Option<Arc<SkColorSpace>>,
        src_rect: &SkIRect,
        dst_size: &SkISize,
        rescale_gamma: RescaleGamma,
        rescale_mode: RescaleMode,
        callback: ReadPixelsCallback,
        context: ReadPixelsContext,
    ) {
        if !SkIRect::make_wh(self.width(), self.height()).contains_rect(src_rect)
            || dst_size.isZero()
            || (dst_size.width() & 1) != 0
            || (dst_size.height() & 1) != 0
        {
            unsafe {
                callback(context, None);
            }
            return;
        }
        unsafe {
            self.asImageBase().onAsyncRescaleAndReadPixelsYUV420(
                yuv_color_space,
                true,
                dst_color_space,
                *src_rect,
                *dst_size,
                rescale_gamma,
                rescale_mode,
                callback,
                context,
            );
        }
    }
}

// Source: src/image/SkImage.cpp:159-180
impl dyn SkImage + '_ {
    /// Safety: this CPU entry reads initialized immutable source pixels; dst
    /// covers the complete exclusive writable layout. Source/destination aliases
    /// must obey the native Pixmap scaler's nonconflicting pixel-access lease.
    pub unsafe fn scalePixels(
        &self,
        dst: &crate::src::core::SkPixmap::SkPixmap,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        // Native162 GPU directContext lookup is outside this selected CPU API;
        // real CPU read/getROPixels hooks carry the same original caching hint.
        if self.width() == dst.width() && self.height() == dst.height() {
            return unsafe { self.readPixels(dst, 0, 0, hint) };
        }
        let mut bitmap = crate::src::core::SkBitmap::SkBitmap::default();
        if self.asImageBase().getROPixelsCPU(&mut bitmap, hint) {
            let mut pmap = crate::src::core::SkPixmap::SkPixmap::default();
            // Native short-circuit ordering: a failed peek never invokes scaler.
            // The original retains bitmap's actual PixelRef until scaling ends.
            return if bitmap.peekPixels(Some(&mut pmap)) {
                unsafe { pmap.scalePixels(dst, sampling) }
            } else {
                Ok(false)
            };
        }
        Ok(false)
    }
}

// Source: src/image/SkImage.cpp:184-184
fn image_color_type<I: ImageCore + ?Sized>(image: &I) -> SkColorType {
    image.imageInfo().colorType()
}
// Source: src/image/SkImage.cpp:186-186
fn image_alpha_type<I: ImageCore + ?Sized>(image: &I) -> SkAlphaType {
    image.imageInfo().alphaType()
}
// Source: src/image/SkImage.cpp:188-188
fn image_color_space<I: ImageCore + ?Sized>(image: &I) -> Option<&SkColorSpace> {
    image.imageInfo().colorSpace()
}
// Source: src/image/SkImage.cpp:190-190
fn image_ref_color_space<I: ImageCore + ?Sized>(image: &I) -> Option<Arc<SkColorSpace>> {
    image.imageInfo().refColorSpace()
}
// Rust overload names preserve each native overload; &Arc<Self> retains the
// existing owner while the factory receives exactly one cloned image reference.
// Source: include/core/SkImage.h:383-383
// Source: src/image/SkImage.cpp:192-196
impl dyn SkImage + 'static {
    pub fn makeShaderClampWithMatrix(
        self: &Arc<Self>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::Make(
            Some(self.clone()),
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            sampling,
            Some(localMatrix),
            false,
        )
    }
}
// Source: include/core/SkImage.h:384-385
// Source: src/image/SkImage.cpp:198-202
impl dyn SkImage + 'static {
    pub fn makeShaderClamp(
        self: &Arc<Self>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::Make(
            Some(self.clone()),
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            sampling,
            localMatrix,
            false,
        )
    }
}
// Source: include/core/SkImage.h:380-381
// Source: src/image/SkImage.cpp:204-209
impl dyn SkImage + 'static {
    pub fn makeShaderWithMatrix(
        self: &Arc<Self>,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::Make(
            Some(self.clone()),
            tmx,
            tmy,
            sampling,
            Some(localMatrix),
            false,
        )
    }
}
// Source: include/core/SkImage.h:378-379
// Source: src/image/SkImage.cpp:211-216
impl dyn SkImage + 'static {
    pub fn makeShader(
        self: &Arc<Self>,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::Make(
            Some(self.clone()),
            tmx,
            tmy,
            sampling,
            localMatrix,
            false,
        )
    }
}
// Source: include/core/SkImage.h:402-403
// Source: src/image/SkImage.cpp:218-223
impl dyn SkImage + 'static {
    pub fn makeRawShaderWithMatrix(
        self: &Arc<Self>,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::MakeRaw(
            Some(self.clone()),
            tmx,
            tmy,
            sampling,
            Some(localMatrix),
        )
    }
}
// Source: include/core/SkImage.h:405-405
// Source: src/image/SkImage.cpp:225-230
impl dyn SkImage + 'static {
    pub fn makeRawShaderClampWithMatrix(
        self: &Arc<Self>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::MakeRaw(
            Some(self.clone()),
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            sampling,
            Some(localMatrix),
        )
    }
}
// Source: include/core/SkImage.h:406-407
// Source: src/image/SkImage.cpp:232-237
impl dyn SkImage + 'static {
    pub fn makeRawShaderClamp(
        self: &Arc<Self>,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::MakeRaw(
            Some(self.clone()),
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            crate::include::core::SkTileMode::SkTileMode::kClamp,
            sampling,
            localMatrix,
        )
    }
}
// Source: include/core/SkImage.h:400-401
// Source: src/image/SkImage.cpp:239-244
impl dyn SkImage + 'static {
    pub fn makeRawShader(
        self: &Arc<Self>,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        localMatrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        crate::src::shaders::SkImageShader::SkImageShader::MakeRaw(
            Some(self.clone()),
            tmx,
            tmy,
            sampling,
            localMatrix,
        )
    }
}
// Source: include/core/SkImage.h:742-742
// Source: src/image/SkImage.cpp:246-246
impl dyn SkImage + '_ {
    pub fn refEncodedData(&self) -> Option<Arc<crate::src::core::SkData::SkData>> {
        self.asImageBase().onRefEncoded()
    }
}

impl dyn SkImage + '_ {
    // Source: include/core/SkImage.h:543-547
    // Source: src/image/SkImage.cpp:250-254
    pub unsafe fn readPixels(
        &self,
        dst: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            self.readPixels_info(
                dst.info(),
                dst.writable_addr().cast(),
                dst.rowBytes(),
                x,
                y,
                hint,
            )
        }
    }
    // Source: include/core/SkImage.h:553-554
    // Source: src/image/SkImage.cpp:257-260
    /// Safety: same native source and exclusive writable pixmap lease as readPixels.
    pub unsafe fn readPixels_legacy(
        &self,
        dst: &crate::src::core::SkPixmap::SkPixmap,
        x: i32,
        y: i32,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.readPixels(dst, x, y, hint) }
    }
    // Source: include/core/SkImage.h:846-847
    // Source: src/image/SkImage.cpp:263-267
    pub unsafe fn asLegacyBitmap(
        &self,
        bitmap: &mut crate::src::core::SkBitmap::SkBitmap,
        _mode: LegacyBitmapMode,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.asImageBase().onAsLegacyBitmapCPU(bitmap) }
    }
}

// Source: src/image/SkImage.cpp:269-269
fn image_is_alpha_only<I: ImageCore + ?Sized>(image: &I) -> bool {
    crate::src::core::SkImageInfoPriv::SkColorTypeIsAlphaOnly(image.imageInfo().colorType())
}

// Source: src/image/SkImage.cpp:271-288
impl dyn SkImage + 'static {
    /// Safety: source image storage is initialized/readable with its actual
    /// immutable backing lease throughout any native reinterpretation copy.
    pub unsafe fn reinterpretColorSpace(
        self: &Arc<Self>,
        target: Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    ) -> Option<Arc<Self>> {
        let target = target?;
        let current = self
            .colorSpace()
            .unwrap_or_else(|| crate::src::core::SkColorSpace::sk_srgb_singleton().as_ref());
        if crate::src::core::SkColorSpace::SkColorSpace::Equals(
            Some(current),
            Some(target.as_ref()),
        ) || self.isAlphaOnly()
        {
            return Some(self.clone());
        }
        unsafe { self.asImageBase().onReinterpretColorSpaceCPU(target) }
    }
}

// Source: include/core/SkImage.h:806-806
// Source: src/image/SkImage.cpp:290-295
impl dyn SkImage + 'static {
    /// Safety: original immutable CPU backing remains initialized/readable for
    /// any required real raster readback; the Arc retains the actual image owner.
    pub unsafe fn makeNonTextureImageCPU(
        self: &Arc<Self>,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        if !self.isTextureBacked() {
            return Ok(Some(self.clone()));
        }
        unsafe { self.makeRasterImageCPU(CachingHint::kDisallow_CachingHint) }
    }
}

// Source: include/core/SkImage.h:821-822
// Source: src/image/SkImage.cpp:297-320
impl dyn SkImage + 'static {
    /// Safety: this selected CPU entry has no GPU directContext capability.
    /// Source pixels remain initialized/readable and immutable for the complete
    /// native peek/read/copy operation, including every retained source alias.
    /// On successful readPixels the actual CPU hook initializes every destination
    /// pixel; a failed read releases uninitialized raw storage without reading it.
    pub unsafe fn makeRasterImageCPU(
        self: &Arc<Self>,
        hint: CachingHint,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        let mut pm = crate::src::core::SkPixmap::SkPixmap::default();
        if self.peekPixels(Some(&mut pm)) {
            return Ok(Some(self.clone()));
        }
        let info = self.imageInfo();
        let row_bytes = info.minRowBytes();
        let size = info.computeByteSize(row_bytes);
        if crate::src::core::SkImageInfo::SkImageInfo::ByteSizeOverflowed(size) {
            return Ok(None);
        }
        // Original309-312 saved GrDirectContext selection is GPU-only and not
        // represented by this explicit CPU specialization. No context is faked.
        let data = unsafe { crate::src::core::SkData::SkData::MakeUninitialized(size) };
        pm = crate::src::core::SkPixmap::SkPixmap::new(
            &info.makeColorSpace(None),
            data.writable_data(),
            info.minRowBytes(),
        );
        if !unsafe { self.readPixels(&pm, 0, 0, hint) }? {
            return Ok(None);
        }
        Ok(unsafe { super::SkImage_RasterFactories::RasterFromData(info, Some(data), row_bytes) })
    }
}

// Source: include/core/SkImage.h:783-783
// Source: src/image/SkImage.cpp:322-322
impl dyn SkImage + '_ {
    pub fn hasMipmaps(&self) -> bool {
        self.asImageBase().onHasMipmaps()
    }
}
// Source: include/core/SkImage.h:788-788
// Source: src/image/SkImage.cpp:324-324
impl dyn SkImage + '_ {
    pub fn isProtected(&self) -> bool {
        self.asImageBase().onIsProtected()
    }
}

// Source: include/core/SkImage.h:832-834
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegacyBitmapMode {
    kRO_LegacyBitmapMode,
}
// Source: include/core/SkImage.h:913-913
// Source: src/image/SkImage.cpp:326-333
impl dyn SkImage + 'static {
    pub unsafe fn withMipmaps(
        self: &Arc<Self>,
        mips: Option<crate::src::core::SkMipmap::SkMipmap>,
    ) -> Result<Arc<Self>, crate::RasterError> {
        if mips
            .as_ref()
            .is_none_or(|mips| mips.validForRootLevel(self.imageInfo()))
        {
            if let Some(result) = (unsafe { self.asImageBase().onMakeWithMipmapsCPU(mips) })? {
                return Ok(result);
            }
        }
        Ok(self.clone())
    }
}

// Source: include/core/SkImage.h:794-794
// Source: src/image/SkImage.cpp:335-337
impl dyn SkImage + 'static {
    /// Safety: the same initialized immutable source-pixel lease required by
    /// withMipmaps holds through construction/copy of every retained mip level.
    pub unsafe fn withDefaultMipmaps(self: &Arc<Self>) -> Result<Arc<Self>, crate::RasterError> {
        unsafe { self.withMipmaps(None) }
    }
}

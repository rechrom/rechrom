// Source: src/image/SkImage_Raster.cpp:35-35
// The same incomplete context as Image_Base; it has no invented layout/body.
use super::SkImage::{CachingHint, SkImageState};
pub use super::SkImage_Base::GrDirectContext;
use super::SkImage_Base::{SkImageBaseState, SkImage_Base};
use crate::src::core::{
    SkBitmap::SkBitmap, SkData::SkData, SkImageInfo::SkImageInfo, SkMipmap::SkMipmap,
    SkPixelRef::SkPixelRef, SkPixelStorage::SkPixelStorageObject, SkPixmap::SkPixmap,
};
use std::ffi::c_void;
use std::sync::Arc;
// Source: src/image/SkImage_Raster.h:35-39
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SkCopyPixelsMode {
    kIfMutable,
    kAlways,
    kNever,
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:47-55
    pub fn isValid(
        &self,
        recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
    ) -> bool {
        let Some(recorder) = recorder else {
            return false;
        };
        if recorder.cpuRecorder().is_none() {
            return false;
        }
        true
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:73-73
    pub fn getPixelRef(&self) -> Option<&Arc<SkPixelRef>> {
        self.fBitmap.pixelRef()
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:79-79
    // Source: src/image/SkImage_Raster.h:83-85
    pub fn notifyAddedToRasterCache(&self) {
        self.fBitmap
            .pixelRef()
            .expect("raster bitmap requires pixelref")
            .notifyAddedToCache();
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:87-88
    pub fn onHasMipmaps(&self) -> bool {
        self.fMips.is_some()
    }
}
impl SkImage_Raster {
    pub fn onIsProtected(&self) -> bool {
        false
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:90-90
    pub fn onPeekMips(&self) -> Option<&SkMipmap> {
        self.fMips.as_ref()
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:94-94
    pub fn type_(&self) -> super::SkImage_Base::Type {
        super::SkImage_Base::Type::kRaster
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:96-96
    pub fn bitmap(&self) -> SkBitmap {
        self.fBitmap.clone()
    }
}
impl SkImage_Raster {
    pub(crate) fn image_state(&self) -> &SkImageState {
        &self.base.image
    }
}
// Source: src/image/SkImage_Raster.h:127-128
// Deliberately reversed Rust member declaration order: native destructor drops mips, then bitmap, then ImageBase.
pub struct SkImage_Raster {
    fMips: Option<SkMipmap>,
    fBitmap: SkBitmap,
    base: SkImageBaseState,
}
// Source: src/image/SkImage_Raster.cpp:39-44
fn is_not_subset(bm: &SkBitmap) -> bool {
    crate::SkASSERT!(@source b"src/image/SkImage_Raster.cpp\0".as_ptr(),40; @condition b"bm.pixelRef()\0".as_ptr(); bm.pixelRef().is_some());
    let pr = bm.pixelRef().expect("bitmap requires pixelref");
    let dim = crate::include::core::SkSize::SkISize::Make(pr.width(), pr.height());
    crate::SkASSERT!(@source b"src/image/SkImage_Raster.cpp\0".as_ptr(),42; @condition b"dim != bm.dimensions() || bm.pixelRefOrigin().isZero()\0".as_ptr(); dim!=bm.dimensions()||{let p=bm.pixelRefOrigin();p.fX==0&&p.fY==0});
    dim == bm.dimensions()
}
// Source: src/image/SkImage_Raster.cpp:46-49
unsafe extern "C" fn release_data(_addr: *mut c_void, context: *mut c_void) {
    drop(Arc::from_raw(context as *const SkData));
}
// Source: src/image/SkImage_Raster.cpp:51-57
// Source: src/image/SkImage_Raster.cpp:59-60
// Source: src/image/SkImage_Raster.cpp:62-62
impl SkImage_Raster {
    pub(crate) unsafe fn from_data(
        info: &SkImageInfo,
        data: Arc<SkData>,
        rowBytes: usize,
        mips: Option<SkMipmap>,
        id: u32,
    ) -> Self {
        // Native initializes Base, default bitmap, then its lvalue mipmap
        // member retain. Reverse field declaration affects teardown only.
        let base = SkImageBaseState::new(info, id, None);
        let fBitmap = SkBitmap::default();
        let fMips = mips.clone();
        let mut out = Self {
            base,
            fBitmap,
            fMips,
        };
        let addr = data.data() as *mut c_void;
        let context = Arc::into_raw(data) as *mut c_void;
        // The transferred Arc data reference owns this exact callback context through final release.
        unsafe {
            out.fBitmap
                .installPixels(info, addr, rowBytes, Some(release_data), context);
        }
        out.fBitmap.setImmutable();
        let owner = out
            .fBitmap
            .pixelRef()
            .cloned()
            .map(|p| p as Arc<dyn SkPixelStorageObject>);
        out.base.fPixelStorages.push(owner);
        out
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.cpp:64-71
    pub(crate) unsafe fn from_bitmap(
        bm: &SkBitmap,
        mips: Option<SkMipmap>,
        bitmapMayBeMutable: bool,
    ) -> Self {
        let id = if is_not_subset(bm) {
            bm.getGenerationID()
        } else {
            0
        };
        let owner = bm
            .pixelRef()
            .cloned()
            .map(|p| p as Arc<dyn SkPixelStorageObject>);
        let base = SkImageBaseState::new(bm.info(), id, owner);
        let fBitmap = bm.clone();
        let fMips = mips.clone();
        let out = Self {
            base,
            fBitmap,
            fMips,
        };
        crate::SkASSERT!(@source b"src/image/SkImage_Raster.cpp\0".as_ptr(),70; @condition b"bitmapMayBeMutable || fBitmap.isImmutable()\0".as_ptr(); bitmapMayBeMutable||out.fBitmap.isImmutable());
        out
    }
}
// Source: src/image/SkImage_Raster.cpp:73-73
impl Drop for SkImage_Raster {
    fn drop(&mut self) { /* Native empty body; declared fields then drop mips, bitmap, base. */
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.cpp:86-88
    // The Pixmap is the source's nonowning readonly pixel view. It gains no
    // PixelRef owner; use only while this original image/backing remains live.
    // Surface-backed pixels remain readonly until its real COW protocol detaches.
    pub fn onPeekPixels(&self, pm: Option<&mut SkPixmap>) -> bool {
        self.fBitmap.peekPixels(pm)
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.cpp:91-93
    // CPU specialization omits the unused GPU context pointer; destination native assignment uses clone_from, not destruction order.
    pub fn getROPixels(&self, dst: &mut SkBitmap, _hint: CachingHint) -> bool {
        dst.clone_from(&self.fBitmap);
        true
    }
}
// Source: src/image/SkImage_Raster.cpp:106-111
// Source: src/image/SkImage_Raster.cpp:113-118
// Source: src/image/SkImage_Raster.cpp:120-121
// Source: src/image/SkImage_Raster.cpp:75-84
impl SkImage_Raster {
    /// Safety: initialized source backing and writable destination layout obey
    /// the native transfer lease; source/destination do not conflict.
    unsafe fn on_read_pixels(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
    ) -> Result<bool, crate::RasterError> {
        let shallow_copy = self.fBitmap.clone();
        unsafe { shallow_copy.readPixels_info(info, pixels, row_bytes, x, y) }
    }
}

// Source: src/image/SkImage_Raster.cpp:95-104
impl SkImage_Raster {
    /// Safety: a supplied recorder is an actual CPU recorder, as required by the
    /// original raster virtual; returned surface borrows that real recorder.
    unsafe fn on_make_surface<'rec>(
        &self,
        recorder: Option<&'rec dyn crate::include::core::SkRecorder::SkRecorder>,
        info: &SkImageInfo,
    ) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface::SkSurface<'rec> + 'rec>>
    {
        let recorder = match recorder {
            Some(recorder) => recorder
                .cpuRecorder()
                .expect("native raster surface requires CPU recorder"),
            None => crate::src::core::SkCPURecorder::Recorder::TODO(),
        };
        recorder.makeBitmapSurfaceRowBytes(info, 0, None)
    }
}

// Source: src/image/SkImage_Raster.cpp:123-125
pub(crate) unsafe fn copy_bitmap_subset(
    orig: &SkBitmap,
    subset: &crate::src::core::SkRect::SkIRect,
) -> SkBitmap {
    let info = orig.info().makeDimensions(subset.size());
    let mut bitmap = SkBitmap::default();
    if !bitmap.tryAllocPixels(&info, 0) {
        return SkBitmap::default();
    }
    let dst = bitmap.getPixels();
    let src = orig.getAddr(subset.fLeft, subset.fTop);
    if dst.is_null() || src.is_null() {
        crate::SkDEBUGFAIL!(@source b"src/image/SkImage_Raster.cpp\0".as_ptr(),116; b"SkImage_Raster::onMakeSubset with nullptr src or dst\0".as_ptr());
        return SkBitmap::default();
    }
    crate::src::core::SkRectMemcpy::SkRectMemcpy(
        dst,
        bitmap.rowBytes(),
        src,
        orig.rowBytes(),
        bitmap.rowBytes(),
        subset.height(),
    );
    bitmap.setImmutable();
    bitmap
}
// Source: src/image/SkImage_Raster.cpp:127-147
unsafe fn copy_mipmaps(
    src: &SkBitmap,
    source: Option<&SkMipmap>,
) -> Result<Option<SkMipmap>, crate::RasterError> {
    let Some(source) = source else {
        return Ok(None);
    };
    let Some(dst) = (unsafe { SkMipmap::Build(src.pixmap(), None, false) })? else {
        return Ok(None);
    };
    for i in 0..dst.countLevels() {
        let mut src_level = crate::src::core::SkMipmap::Level {
            fPixmap: SkPixmap::default(),
            fScale: crate::include::core::SkSize::SkSize::Make(0.0, 0.0),
        };
        let mut dst_level = src_level.clone();
        assert!(source.getLevel(i, Some(&mut src_level)));
        assert!(dst.getLevel(i, Some(&mut dst_level)));
        // Native ignores semantic false here; dimensions/layouts are those of
        // corresponding real levels. Execution-stage errors still propagate.
        let _ = unsafe {
            src_level.fPixmap.readPixels_raw(
                dst_level.fPixmap.info(),
                dst_level.fPixmap.writable_addr(),
                dst_level.fPixmap.rowBytes(),
                0,
                0,
            )
        }?;
    }
    Ok(Some(dst))
}
// Source: src/image/SkImage_Raster.cpp:149-180
impl SkImage_Raster {
    unsafe fn on_make_subset(
        &self,
        subset: &crate::src::core::SkRect::SkIRect,
        props: super::SkImage::RequiredProperties,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
        if props.fMipmapped {
            let full_copy = *subset
                == crate::src::core::SkRect::SkIRect::make_wh(
                    self.fBitmap.width(),
                    self.fBitmap.height(),
                );
            let mips = if full_copy {
                unsafe { copy_mipmaps(&self.fBitmap, self.fMips.as_ref()) }?
            } else {
                None
            };
            let mut tmp_subset = SkBitmap::default();
            if !self.fBitmap.extractSubset(Some(&mut tmp_subset), subset) {
                return Ok(None);
            }
            let tmp: Arc<dyn super::SkImage::SkImage> =
                Arc::new(unsafe { Self::from_bitmap(&tmp_subset, None, true) });
            crate::SkASSERT!(@source b"src/image/SkImage_Raster.cpp\0".as_ptr(),170; @condition b"!mips || mips->validForRootLevel(tmp->imageInfo())\0".as_ptr(); mips.as_ref().is_none_or(|mips|mips.validForRootLevel(tmp.imageInfo())));
            Ok(Some(unsafe { tmp.withMipmaps(mips) }?))
        } else {
            let copy = unsafe { copy_bitmap_subset(&self.fBitmap, subset) };
            if !copy.isNull() {
                return Ok(unsafe { super::SkImage_RasterFactories::RasterFromBitmap(&copy) });
            }
            Ok(None)
        }
    }
}
impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.cpp:182-187
    // Source: src/image/SkImage_Raster.cpp:189-191
    // Source: src/image/SkImage_Raster.cpp:193-198
    // Source: src/image/SkImage_Raster.cpp:200-200
    // Source: src/image/SkImage_Raster.cpp:202-207
    /// # Safety
    /// Every readable pixel/row is live and initialized during copying. If pixels
    /// are shared (including kNever), they must remain immutable for ALL retained
    /// image/shader/cache uses, or the real Surface COW protocol must detach first.
    /// External aliases must not write/read against an active exclusive operation.
    pub unsafe fn MakeFromBitmap(
        bm: &SkBitmap,
        cpm: SkCopyPixelsMode,
        mips: Option<SkMipmap>,
    ) -> Option<Arc<Self>> {
        if !crate::src::core::SkImageInfoPriv::SkImageInfoIsValid(bm.info())
            || bm.rowBytes() < bm.info().minRowBytes()
        {
            return None;
        }
        if bm.getPixels().is_null() {
            return None;
        }
        if cpm == SkCopyPixelsMode::kAlways
            || (!bm.isImmutable() && cpm != SkCopyPixelsMode::kNever)
        {
            let size = bm.computeByteSize();
            if SkImageInfo::ByteSizeOverflowed(size) {
                return None;
            }
            let data = SkData::MakeWithCopy(bm.getPixels(), size);
            return Some(Arc::new(Self::from_data(
                bm.info(),
                data,
                bm.rowBytes(),
                mips,
                0,
            )));
        }
        Some(Arc::new(Self::from_bitmap(
            bm,
            mips,
            cpm == SkCopyPixelsMode::kNever,
        )))
    }
}

// Source: src/image/SkImage_Raster.h:41. Native retained pixel storage and
// callback contexts have their original lifetime/immutability requirements.
// Image/Shader's readonly interface does not establish arbitrary release-context
// or backing graph Send/Sync, so no universal unsafe marker is asserted here.
impl super::SkImage::ImageCore for SkImage_Raster {
    fn state(&self) -> &SkImageState {
        &self.base.image
    }
    fn asImageBase(&self) -> &dyn super::SkImage_Base::SkImage_Base {
        self
    }
    fn isTextureBacked(&self) -> bool {
        self.isTextureBackedCPU()
    }
    fn textureSize(&self) -> usize {
        0
    }
    fn isValid(&self, recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>) -> bool {
        SkImage_Raster::isValid(self, recorder)
    }
}
impl super::SkImage_Base::SkImage_Base for SkImage_Raster {
    fn onPeekPixelsCPU(&self, pixmap: &mut SkPixmap) -> bool {
        self.onPeekPixels(Some(pixmap))
    }
    unsafe fn onReadPixelsCPU(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        row_bytes: usize,
        x: i32,
        y: i32,
        _hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.on_read_pixels(info, pixels, row_bytes, x, y) }
    }
    unsafe fn onAsLegacyBitmapCPU(
        &self,
        bitmap: &mut SkBitmap,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.on_as_legacy_bitmap(bitmap) }
    }
    unsafe fn onMakeSurfaceCPU<'rec>(
        &self,
        recorder: Option<&'rec dyn crate::include::core::SkRecorder::SkRecorder>,
        info: &SkImageInfo,
    ) -> Option<crate::include::core::SkRefCnt::sk_sp<dyn super::SkSurface::SkSurface<'rec> + 'rec>>
    {
        unsafe { self.on_make_surface(recorder, info) }
    }
    // Source: src/image/SkImage_Raster.h:94-94
    fn type_(&self) -> super::SkImage_Base::Type {
        SkImage_Raster::type_(self)
    }
    // Source: src/image/SkImage_Raster.h:67-67
    fn onPeekBitmap(&self) -> Option<&SkBitmap> {
        Some(&self.fBitmap)
    }
    fn base_state(&self) -> &SkImageBaseState {
        &self.base
    }
    fn onMakeSubsetCPU(
        &self,
        _recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        subset: &crate::src::core::SkRect::SkIRect,
        props: super::SkImage::RequiredProperties,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
        unsafe { self.on_make_subset(subset, props) }
    }
    unsafe fn onMakeWithMipmapsCPU(
        &self,
        mips: Option<SkMipmap>,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
        Ok(unsafe { self.on_make_with_mipmaps(mips) }?
            .map(|image| image as Arc<dyn super::SkImage::SkImage>))
    }

    unsafe fn onReinterpretColorSpaceCPU(
        &self,
        target: Arc<crate::src::core::SkColorSpace::SkColorSpace>,
    ) -> Option<Arc<dyn super::SkImage::SkImage>> {
        unsafe { self.on_reinterpret_color_space(target) }
    }
    fn asRaster(&self) -> Option<&Self> {
        Some(self)
    }
    fn onHasMipmaps(&self) -> bool {
        SkImage_Raster::onHasMipmaps(self)
    }
    fn onIsProtected(&self) -> bool {
        SkImage_Raster::onIsProtected(self)
    }
    fn onPeekMips(&self) -> Option<&SkMipmap> {
        self.fMips.as_ref()
    }
    fn getROPixelsCPU(&self, dst: &mut SkBitmap, hint: CachingHint) -> bool {
        self.getROPixels(dst, hint)
    }
    fn notifyAddedToRasterCache(&self) {
        SkImage_Raster::notifyAddedToRasterCache(self)
    }
}

// Source: src/image/SkImage_Raster.cpp:209-221
impl SkImage_Raster {
    /// Safety: original source's immutable image lease remains valid throughout
    /// sharing/copy; the caller preserves native read-only returned bitmap use.
    unsafe fn on_as_legacy_bitmap(
        &self,
        bitmap: &mut SkBitmap,
    ) -> Result<bool, crate::RasterError> {
        if self.fBitmap.isImmutable() {
            let origin = self.fBitmap.pixelRefOrigin();
            bitmap.setInfo(self.fBitmap.info(), self.fBitmap.rowBytes());
            bitmap.setPixelRef(self.fBitmap.pixelRef().cloned(), origin.fX, origin.fY);
            return Ok(true);
        }
        unsafe { super::SkImage_Base::as_legacy_bitmap(self, bitmap) }
    }
}

// Source: src/image/SkImage_Raster.cpp:223-239
impl SkImage_Raster {
    pub fn makeShaderForPaint(
        self: &Arc<Self>,
        paint: &crate::src::core::SkPaint::SkPaint,
        tmx: crate::include::core::SkTileMode::SkTileMode,
        tmy: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        local_matrix: Option<&crate::src::core::SkMatrix::SkMatrix>,
    ) -> Option<Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        let image: Arc<dyn super::SkImage::SkImage> = self.clone();
        let mut shader = crate::src::shaders::SkImageShader::SkImageShader::Make(
            Some(image),
            tmx,
            tmy,
            sampling,
            local_matrix,
            false,
        )?;
        if crate::src::core::SkImageInfoPriv::SkColorTypeIsAlphaOnly(self.fBitmap.colorType())
            && paint.getShader().is_some()
        {
            shader = crate::src::shaders::SkBlendShader::Blend(
                crate::src::core::SkBlendMode::SkBlendMode::kDstIn,
                paint.refShader(),
                Some(shader),
            )?;
        }
        Some(shader)
    }
}

// Source: src/image/SkImage_Raster.cpp:241-252
impl SkImage_Raster {
    unsafe fn on_make_with_mipmaps(
        &self,
        mut mips: Option<SkMipmap>,
    ) -> Result<Option<Arc<Self>>, crate::RasterError> {
        if mips.is_none() {
            mips = unsafe { SkMipmap::Build(self.fBitmap.pixmap(), None, true) }?;
        }
        Ok(unsafe { Self::MakeFromBitmap(&self.fBitmap, SkCopyPixelsMode::kAlways, mips) })
    }
}
impl super::SkImage::ImageVirtuals for SkImage_Raster {
    fn makeSubset(
        self: Arc<Self>,
        recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        subset: &crate::src::core::SkRect::SkIRect,
        props: super::SkImage::RequiredProperties,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
        super::SkImage_Base::make_subset(&*self, recorder, subset, props)
    }
    fn isLazyGenerated(&self) -> bool {
        self.isLazyGeneratedCPU()
    }
    fn makeColorSpace(
        self: Arc<Self>,
        recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        target: Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        props: super::SkImage::RequiredProperties,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
        super::SkImage_Base::make_color_space(self, recorder, target, props)
    }
    // Source: src/image/SkImage_Raster.cpp:253-269
    fn makeColorTypeAndColorSpace(
        self: Arc<Self>,
        _recorder: Option<&dyn crate::include::core::SkRecorder::SkRecorder>,
        target_type: crate::include::core::SkColorType::SkColorType,
        target: Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        _props: super::SkImage::RequiredProperties,
    ) -> Result<Option<Arc<dyn super::SkImage::SkImage>>, crate::RasterError> {
        let mut src = SkPixmap::default();
        assert!(self.fBitmap.peekPixels(Some(&mut src)));
        let mut dst = SkBitmap::default();
        if !dst.tryAllocPixels(
            &self
                .fBitmap
                .info()
                .makeColorType(target_type)
                .makeColorSpace(target),
            0,
        ) {
            return Ok(None);
        }
        assert!(unsafe { dst.writePixels(&src, 0, 0) }?);
        dst.setImmutable();
        Ok(unsafe { super::SkImage_RasterFactories::RasterFromBitmap(&dst) })
    }
}

// Source: src/image/SkImage_Raster.cpp:271-278
impl SkImage_Raster {
    /// Safety: image retains initialized immutable source pixels for the copy.
    unsafe fn on_reinterpret_color_space(
        &self,
        target: Arc<crate::src::core::SkColorSpace::SkColorSpace>,
    ) -> Option<Arc<dyn super::SkImage::SkImage>> {
        let mut pixmap = self.fBitmap.pixmap().clone();
        pixmap.setColorSpace(Some(target));
        unsafe { super::SkImage_RasterFactories::RasterFromPixmapCopy(&pixmap) }
    }
}

impl SkImage_Raster {
    // Source: src/image/SkImage_Raster.h:119-121.
    /// Safety: same original pixel lifetime/immutability or Surface COW lease as MakeFromBitmap.
    pub unsafe fn MakeFromBitmap_default(
        bm: &SkBitmap,
        cpm: SkCopyPixelsMode,
    ) -> Option<Arc<Self>> {
        unsafe { Self::MakeFromBitmap(bm, cpm, None) }
    }
}

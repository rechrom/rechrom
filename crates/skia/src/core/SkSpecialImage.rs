#![allow(non_snake_case, non_upper_case_globals)]
use crate::include::core::SkSize::SkISize;
use crate::include::core::SkSurfaceProps::SkSurfaceProps;
use crate::src::core::SkImageInfo::SkColorInfo;
use crate::src::core::SkRect::SkIRect;
// Source: src/core/SkSpecialImage.h:51-52
// Native nullable callback pointer; its invocation is unsafe because caller
// supplies the actual retained pixel/context release contract.
pub type ReleaseContext = *mut core::ffi::c_void;
pub type RasterReleaseProc = Option<unsafe extern "C" fn(*mut core::ffi::c_void, ReleaseContext)>;
// Source: src/core/SkSpecialImage.h:34-36
pub const kNeedNewImageUniqueID_SpecialImage: u32 = 0;
// Source: src/core/SkSpecialImage.h:152-156
// Rust stores the abstract base's immutable state separately for later raster composition.
// This is base metadata only; no concrete image/backing implementation is claimed.
pub struct SkSpecialImageBase {
    fSubset: SkIRect,
    fUniqueID: u32,
    fColorInfo: SkColorInfo,
    fProps: SkSurfaceProps,
}
// Source: src/core/SkSpecialImage.h:54-61
impl SkSpecialImageBase {
    pub fn props(&self) -> &SkSurfaceProps {
        &self.fProps
    }
    pub fn width(&self) -> i32 {
        self.fSubset.width()
    }
    pub fn height(&self) -> i32 {
        self.fSubset.height()
    }
    pub fn dimensions(&self) -> SkISize {
        SkISize::Make(self.width(), self.height())
    }
    pub fn subset(&self) -> &SkIRect {
        &self.fSubset
    }
    pub fn uniqueID(&self) -> u32 {
        self.fUniqueID
    }
}
// Source: src/core/SkSpecialImage.h:69-72
impl SkSpecialImageBase {
    pub fn colorInfo(&self) -> &SkColorInfo {
        &self.fColorInfo
    }
    pub fn alphaType(&self) -> crate::include::core::SkAlphaType::SkAlphaType {
        self.fColorInfo.alphaType()
    }
    pub fn colorType(&self) -> crate::include::core::SkColorType::SkColorType {
        self.fColorInfo.colorType()
    }
    pub fn getColorSpace(&self) -> Option<&crate::src::core::SkColorSpace::SkColorSpace> {
        self.fColorInfo.colorSpace()
    }
}
// Native abstract metadata/subset operations; conversions use actual retained raster backing.
// Rust trait composition replaces abstract base-class dispatch; no default backing implementation.
pub trait SkSpecialImage: subset_backing::SubsetBacking + Send + Sync {
    fn base_metadata(&self) -> &SkSpecialImageBase;
    fn props(&self) -> &SkSurfaceProps {
        self.base_metadata().props()
    }
    fn width(&self) -> i32 {
        self.base_metadata().width()
    }
    fn height(&self) -> i32 {
        self.base_metadata().height()
    }
    fn dimensions(&self) -> SkISize {
        self.base_metadata().dimensions()
    }
    fn subset(&self) -> &SkIRect {
        self.base_metadata().subset()
    }
    fn uniqueID(&self) -> u32 {
        self.base_metadata().uniqueID()
    }
    fn colorInfo(&self) -> &SkColorInfo {
        self.base_metadata().colorInfo()
    }
    // Source: src/core/SkSpecialImage.h:63-65
    // Source: src/core/SkSpecialImage.h:132-133
    // These are the exact original non-GPU base predicates, not a substitute
    // GPU owner or context. A real GPU subclass must override its own predicate.
    fn isGaneshBacked(&self) -> bool {
        false
    }
    fn isGraphiteBacked(&self) -> bool {
        false
    }
    fn backingStoreDimensions(&self) -> SkISize;
    fn getSize(&self) -> usize;
    // Source: src/core/SkSpecialImage.h:67-67
    fn isExactFit(&self) -> bool {
        *self.subset() == SkIRect::make_size(self.backingStoreDimensions())
    }
    // Source: src/core/SkSpecialImage.h:77-81
    // Source: src/core/SkSpecialImage.cpp:31-40
    /// Safety: source backing retains asImage's complete immutable/readable
    /// lease, and canvas has the exclusive native operation/COW pixel lease.
    /// No callback may invalidate either owner during this synchronous draw.
    unsafe fn draw(
        &self,
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        x: f32,
        y: f32,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        paint: Option<&crate::src::core::SkPaint::SkPaint>,
        strict: bool,
    ) -> Result<(), crate::RasterError> {
        let dst = crate::src::core::SkRect::SkRect::MakeXYWH(
            x,
            y,
            self.subset().width() as f32,
            self.subset().height() as f32,
        );
        // Native Canvas accepts a nullable sk_sp and discards a null image.
        // The canonical Rust entry borrows a nonnull Arc, so preserve that
        // original nullable dispatch at this ownership boundary.
        if let Some(image) = self.asImage() {
            crate::src::core::SkCanvas::SkCanvas::drawImageRect_raw(
                canvas,
                &image,
                Some(&crate::src::core::SkRect::SkRect::Make(self.subset())),
                &dst,
                sampling,
                paint,
                if strict {
                    crate::src::core::SkCanvas::SrcRectConstraint::kStrict_SrcRectConstraint
                } else {
                    crate::src::core::SkCanvas::SrcRectConstraint::kFast_SrcRectConstraint
                },
            )?;
        }
        Ok(())
    }
    // Source: src/core/SkSpecialImage.h:82-84
    /// Safety: the same source/Canvas leases as draw apply.
    unsafe fn draw_default(
        &self,
        canvas: core::ptr::NonNull<crate::src::core::SkCanvas::SkCanvas<'_>>,
        x: f32,
        y: f32,
    ) -> Result<(), crate::RasterError> {
        self.draw(
            canvas,
            x,
            y,
            &crate::include::core::SkSamplingOptions::SkSamplingOptions::default(),
            None,
            true,
        )
    }
    // Source: src/core/SkSpecialImage.h:138-138
    fn getContext(&self) -> *mut crate::src::image::SkSurface::GrRecordingContext {
        core::ptr::null_mut()
    }
    // Source: src/core/SkSpecialImage.h:112-112
    /// Safety: retain the native readable/immutable backing lease throughout
    /// image creation and every retained shared-image use.
    unsafe fn asImage(&self) -> Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>;
    // Source: src/core/SkSpecialImage.h:124-127
    /// Safety: preserve asImage's complete readable/retained immutable backing
    /// lease for every shader use, including the extracted strict subset view.
    unsafe fn asShader(
        &self,
        tileMode: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        lm: &crate::src::core::SkMatrix::SkMatrix,
        strict: bool,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        // Source: src/core/SkSpecialImage.cpp:44-65
        let mut subsetOrigin = crate::src::core::SkMatrix::SkMatrix::Translate(
            -(self.subset().fLeft as f32),
            -(self.subset().fTop as f32),
        );
        subsetOrigin.postConcat(lm);
        let image = unsafe { self.asImage() }?;
        if strict {
            crate::src::shaders::SkImageShader::SkImageShader::MakeSubset(
                Some(image),
                &crate::src::core::SkRect::SkRect::Make(self.subset()),
                tileMode,
                tileMode,
                sampling,
                Some(&subsetOrigin),
                false,
            )
        } else {
            image.makeShader(tileMode, tileMode, sampling, Some(&subsetOrigin))
        }
    }
    // Source: src/core/SkSpecialImage.h:91-94
    fn makeSubset(&self, subset: &SkIRect) -> Option<std::sync::Arc<dyn SkSpecialImage>> {
        let absolute = subset.make_offset_vec(self.subset().top_left());
        self.onMakeBackingStoreSubset(&absolute)
    }
    // Source: src/core/SkSpecialImage.h:100-102
    fn makePixelOutset(&self) -> Option<std::sync::Arc<dyn SkSpecialImage>> {
        self.onMakeBackingStoreSubset(&self.subset().make_outset(1, 1))
    }
}
mod subset_backing {
    // Source: src/core/SkSpecialImage.h:149-149
    pub trait SubsetBacking {
        fn getROPixels(&self, result: Option<&mut crate::src::core::SkBitmap::SkBitmap>) -> bool;
        fn onMakeBackingStoreSubset(
            &self,
            subset: &super::SkIRect,
        ) -> Option<std::sync::Arc<dyn super::SkSpecialImage>>;
    }
}
// Source: src/core/SkSpecialImage.h:141-144
// Source: src/core/SkSpecialImage.cpp:21-29
impl SkSpecialImageBase {
    pub(crate) fn new(
        subset: SkIRect,
        uniqueID: u32,
        colorInfo: &SkColorInfo,
        props: SkSurfaceProps,
    ) -> Self {
        Self {
            fSubset: subset,
            fUniqueID: if uniqueID == kNeedNewImageUniqueID_SpecialImage {
                crate::src::core::SkNextID::SkNextID::ImageID()
            } else {
                uniqueID
            },
            fColorInfo: colorInfo.clone(),
            fProps: props,
        }
    }
}

// Source: src/core/SkSpecialImage.cpp:68-75
// Source: src/core/SkSpecialImage.cpp:115-116
// Rust composition plus immutable Deref preserves base metadata sharing through the raster object.
// Image/shader conversion methods and read-only subset extraction remain pending real dependencies.
pub struct SkSpecialImage_Raster {
    // Rust declaration order drops derived bitmap before base color-space ownership, as C++ does.
    fBitmap: crate::src::core::SkBitmap::SkBitmap,
    base: SkSpecialImageBase,
}
// Rust sharing adaptation, not additional translated source credit. This is
// deliberately scoped to the sole concrete Raster special image, not SkBitmap:
// * fBitmap's Pixmap descriptor and retained PixelRef never change after new().
// * its shared methods read metadata or produce a separate bitmap descriptor;
//   no safe method reads/writes pixel bytes through the stored raw pointer.
// * PixelRef already supplies canonical synchronized shared ownership and
//   atomic generation/mutability state, and the color-info owners are shareable.
// * asImage/asShader remain unsafe: every caller must preserve the immutable,
//   readable backing lease for image/shader use, including external pixel owners.
// The global cache serializes result/matrix access; cloning a result creates its
// own matrix cache. Moving/cloning/dropping this descriptor does not touch pixels.
unsafe impl Send for SkSpecialImage_Raster {}
unsafe impl Sync for SkSpecialImage_Raster {}
impl core::ops::Deref for SkSpecialImage_Raster {
    type Target = SkSpecialImageBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl SkSpecialImage_Raster {
    fn new(
        subset: SkIRect,
        bm: &crate::src::core::SkBitmap::SkBitmap,
        props: SkSurfaceProps,
    ) -> Self {
        let value = Self {
            base: SkSpecialImageBase::new(
                subset,
                bm.getGenerationID(),
                bm.info().colorInfo(),
                props,
            ),
            fBitmap: bm.clone(),
        };
        crate::SkASSERT!(@source b"src/core/SkSpecialImage.cpp\0".as_ptr(),73; @condition b"bm.pixelRef()\0".as_ptr(); bm.pixelRef().is_some());
        crate::SkASSERT!(@source b"src/core/SkSpecialImage.cpp\0".as_ptr(),74; @condition b"fBitmap.getPixels()\0".as_ptr(); !value.fBitmap.getPixels().is_null());
        value
    }
}
// Source: src/core/SkSpecialImage.cpp:77-79
impl SkSpecialImage_Raster {
    pub fn getROPixels(&self, result: Option<&mut crate::src::core::SkBitmap::SkBitmap>) -> bool {
        self.fBitmap.extractSubset(result, self.subset())
    }
}
// Source: src/core/SkSpecialImage.cpp:81-83
impl SkSpecialImage for SkSpecialImage_Raster {
    fn base_metadata(&self) -> &SkSpecialImageBase {
        &self.base
    }
    fn backingStoreDimensions(&self) -> SkISize {
        self.fBitmap.dimensions()
    }
    fn getSize(&self) -> usize {
        self.fBitmap.computeByteSize()
    }
    // Source: src/core/SkSpecialImage.cpp:92-112
    unsafe fn asShader(
        &self,
        tileMode: crate::include::core::SkTileMode::SkTileMode,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        lm: &crate::src::core::SkMatrix::SkMatrix,
        strict: bool,
    ) -> Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>> {
        if strict {
            let mut subsetBM = crate::src::core::SkBitmap::SkBitmap::default();
            if !self.getROPixels(Some(&mut subsetBM)) {
                return None;
            }
            unsafe { subsetBM.makeShader(tileMode, tileMode, sampling, Some(lm)) }
        } else {
            let mut subsetOrigin = crate::src::core::SkMatrix::SkMatrix::Translate(
                -(self.subset().fLeft as f32),
                -(self.subset().fTop as f32),
            );
            subsetOrigin.postConcat(lm);
            unsafe {
                self.fBitmap
                    .makeShader(tileMode, tileMode, sampling, Some(&subsetOrigin))
            }
        }
    }
    // Source: src/core/SkSpecialImage.cpp:85-85
    unsafe fn asImage(&self) -> Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>> {
        unsafe { self.fBitmap.asImage() }
    }
}
// Source: src/core/SkSpecialImage.cpp:87-90
impl subset_backing::SubsetBacking for SkSpecialImage_Raster {
    fn getROPixels(&self, result: Option<&mut crate::src::core::SkBitmap::SkBitmap>) -> bool {
        SkSpecialImage_Raster::getROPixels(self, result)
    }
    fn onMakeBackingStoreSubset(
        &self,
        subset: &SkIRect,
    ) -> Option<std::sync::Arc<dyn SkSpecialImage>> {
        MakeFromRasterBitmap(*subset, &self.fBitmap, *self.props())
    }
}
// Source: src/core/SkSpecialImage.cpp:120-129
// CPU bitmap ownership is returned through the actual abstract metadata/subset interface.
pub fn MakeFromRasterBitmap(
    subset: SkIRect,
    bm: &crate::src::core::SkBitmap::SkBitmap,
    props: SkSurfaceProps,
) -> Option<std::sync::Arc<dyn SkSpecialImage>> {
    crate::SkASSERT!(@source b"src/core/SkSpecialImage.cpp\0".as_ptr(),123; @condition b"bm.bounds().contains(subset)\0".as_ptr(); bm.bounds().contains_rect(&subset));
    if bm.pixelRef().is_none() {
        return None;
    }
    Some(std::sync::Arc::new(SkSpecialImage_Raster::new(
        subset, bm, props,
    )))
}

// Source: src/core/SkSpecialImage.cpp:131-152
/// Safety: bm's source rows are live, initialized and readable through the copy;
/// tmp is an independent owned destination. The result owns its copied backing.
pub unsafe fn CopyFromRaster(
    subset: SkIRect,
    bm: &crate::src::core::SkBitmap::SkBitmap,
    props: SkSurfaceProps,
) -> Result<Option<std::sync::Arc<dyn SkSpecialImage>>, crate::RasterError> {
    crate::SkASSERT!(@source b"src/core/SkSpecialImage.cpp\0".as_ptr(),134; @condition b"bm.bounds().contains(subset)\0".as_ptr(); bm.bounds().contains_rect(&subset));
    if bm.pixelRef().is_none() {
        return Ok(None);
    }
    let mut tmp = crate::src::core::SkBitmap::SkBitmap::default();
    let info = bm.info().makeDimensions(subset.size());
    if !tmp.tryAllocPixels(&info, 0) {
        return Ok(None);
    }
    if !unsafe {
        bm.readPixels_info(
            tmp.info(),
            tmp.getPixels(),
            tmp.rowBytes(),
            subset.fLeft,
            subset.fTop,
        )
    }? {
        return Ok(None);
    }
    Ok(Some(std::sync::Arc::new(SkSpecialImage_Raster::new(
        SkIRect::make_wh(subset.width(), subset.height()),
        &tmp,
        props,
    ))))
}
// Source: src/core/SkSpecialImage.cpp:154-170
pub fn MakeFromRasterImage(
    subset: SkIRect,
    image: Option<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>>,
    props: SkSurfaceProps,
) -> Option<std::sync::Arc<dyn SkSpecialImage>> {
    let image = image?;
    if subset.is_empty() {
        return None;
    }
    crate::SkASSERT!(@source b"src/core/SkSpecialImage.cpp\0".as_ptr(),161; @condition b"image->bounds().contains(subset)\0".as_ptr(); image.bounds().contains_rect(&subset));
    crate::SkASSERT!(@source b"src/core/SkSpecialImage.cpp\0".as_ptr(),162; @condition b"!image->isTextureBacked()\0".as_ptr(); !image.isTextureBacked());
    let mut bm = crate::src::core::SkBitmap::SkBitmap::default();
    if image.asImageBase().getROPixelsCPU(
        &mut bm,
        crate::src::image::SkImage::CachingHint::kAllow_CachingHint,
    ) {
        return MakeFromRasterBitmap(subset, &bm, props);
    }
    None
}

// Source: src/core/SkSpecialImage.cpp:172-178
// CPU scope has only the sealed Raster backing, so trait dispatch replaces the native Raster cast.
// GPU-backed predicates are excluded, not replaced with fake backend methods.
pub fn AsBitmap(
    image: Option<&dyn SkSpecialImage>,
    result: Option<&mut crate::src::core::SkBitmap::SkBitmap>,
) -> bool {
    match image {
        None => false,
        Some(image) => image.getROPixels(result),
    }
}

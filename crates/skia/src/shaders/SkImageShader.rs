// RasterError interface/propagation adaptation only; UNVERIFIED; no added native-line credit.
// Copyright 2015 Google Inc. BSD-style license.
use crate::include::core::SkTileMode::SkTileMode;
use crate::src::core::SkM44::SkM44;

use crate::include::core::SkSamplingOptions::{SkFilterMode, SkMipmapMode, SkSamplingOptions};
use crate::src::core::{SkMatrix::SkMatrix, SkRect::SkRect};
use crate::src::image::SkImage::SkImage;
use crate::src::shaders::{
    SkShader::SkShader,
    SkShaderBase::{Context, ContextRec, MatrixRec, ShaderType, SkShaderBase},
};
use std::sync::Arc;

impl SkImageShader {
    // Source: src/shaders/SkImageShader.h:73-73
    fn header_type(&self) -> ShaderType {
        ShaderType::kImage
    }
    // Source: src/shaders/SkImageShader.h:77-82
    pub fn tileModeX(&self) -> SkTileMode {
        self.fTileModeX
    }
    pub fn tileModeY(&self) -> SkTileMode {
        self.fTileModeY
    }
    pub fn image(&self) -> Arc<dyn SkImage> {
        self.fImage.clone()
    }
    pub fn sampling(&self) -> SkSamplingOptions {
        self.fSampling
    }
    pub fn subset(&self) -> SkRect {
        self.fSubset
    }
    pub fn isRaw(&self) -> bool {
        self.fRaw
    }
    // Source: src/shaders/SkImageShader.h:75-75
    pub fn CubicResamplerMatrix(B: f32, C: f32) -> SkM44 {
        CubicResamplerMatrix(B, C)
    }
}
// Source: src/shaders/SkImageShader.h:95-105.
// Private non-FFI representation uses native reverse member destruction;
// constructor expressions below retain native source evaluation order.
// The sole retained image is neither copied into a provider nor declared Send.
pub struct SkImageShader {
    // Source: src/shaders/SkImageShader.h:105.
    fClampAsIfUnpremul: bool,
    // Source: src/shaders/SkImageShader.h:82.
    fRaw: bool,
    // Source: src/shaders/SkImageShader.h:81.
    fSubset: SkRect,
    // Source: src/shaders/SkImageShader.h:78.
    fTileModeY: SkTileMode,
    // Source: src/shaders/SkImageShader.h:77.
    fTileModeX: SkTileMode,
    // Source: src/shaders/SkImageShader.h:80.
    fSampling: SkSamplingOptions,
    // Source: src/shaders/SkImageShader.h:79.
    fImage: Arc<dyn SkImage>,
}

// Source: src/shaders/SkImageShader.cpp:47-47
// Source: src/shaders/SkImageShader.cpp:66-69
pub fn CubicResamplerMatrix(B: f32, C: f32) -> SkM44 {
    SkM44::new(
        (1.0 / 6.0) * B,
        -(3.0 / 6.0) * B - C,
        (3.0 / 6.0) * B + 2.0 * C,
        -(1.0 / 6.0) * B - C,
        1.0 - (2.0 / 6.0) * B,
        0.0,
        -3.0 + (12.0 / 6.0) * B + C,
        2.0 - (9.0 / 6.0) * B - C,
        (1.0 / 6.0) * B,
        (3.0 / 6.0) * B + C,
        3.0 - (15.0 / 6.0) * B - 2.0 * C,
        -2.0 + (9.0 / 6.0) * B + C,
        0.0,
        0.0,
        -C,
        (1.0 / 6.0) * B + C,
    )
}

// Source: src/shaders/SkImageShader.cpp:75-76
// Source: src/shaders/SkImageShader.cpp:80-80
// Source: src/shaders/SkImageShader.cpp:84-84
pub fn optimize(tm: SkTileMode, dimension: i32) -> SkTileMode {
    crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),76; @condition b"dimension > 0\0".as_ptr(); dimension > 0);
    if cfg!(SK_BUILD_FOR_ANDROID_FRAMEWORK) {
        tm
    } else if tm != SkTileMode::kDecal && dimension == 1 {
        SkTileMode::kClamp
    } else {
        tm
    }
}

// Source: src/shaders/SkImageShader.cpp:89-90
#[cfg(sk_debug)]
fn needs_subset(
    image: &dyn crate::src::image::SkImage::SkImage,
    subset: &crate::src::core::SkRect::SkRect,
) -> bool {
    *subset != crate::src::core::SkRect::SkRect::MakeIWH(image.width(), image.height())
}

// Source: src/shaders/SkImageShader.cpp:94-106
// Source: src/shaders/SkImageShader.cpp:108-108
// Source: src/shaders/SkImageShader.cpp:111-111
impl SkImageShader {
    // Source: src/shaders/SkImageShader.h:64-69
    pub fn new(
        img: Arc<dyn SkImage>,
        subset: &SkRect,
        tmx: SkTileMode,
        tmy: SkTileMode,
        sampling: &SkSamplingOptions,
        raw: bool,
        clampAsIfUnpremul: bool,
    ) -> Self {
        // Source: src/shaders/SkImageShader.cpp:101-107.
        let fImage = img;
        let fSampling = *sampling;
        let fTileModeX = optimize(tmx, fImage.width());
        let fTileModeY = optimize(tmy, fImage.height());
        let fSubset = *subset;
        let fRaw = raw;
        let fClampAsIfUnpremul = clampAsIfUnpremul;
        let result = Self {
            fImage,
            fSampling,
            fTileModeX,
            fTileModeY,
            fSubset,
            fRaw,
            fClampAsIfUnpremul,
        };
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),108; @condition b"!fRaw || !fClampAsIfUnpremul\0".as_ptr(); !result.fRaw || !result.fClampAsIfUnpremul);
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),111; @condition b"!fRaw || !fSampling.useCubic\0".as_ptr(); !result.fRaw || !result.fSampling.useCubic());
        result
    }
}
// Source: src/shaders/SkImageShader.h:85-85
impl crate::src::core::SkFlattenable::SkFlattenableView for SkImageShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkImageShader")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/shaders/SkImageShader.cpp:115-128
// The legacy wire enum is separate from current sampling options. Rust forbids
// duplicate enum discriminants, so kLast is an associated alias of the same value.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LegacyFilterEnum {
    kNone,
    kLow,
    kMedium,
    kHigh,
    kInheritFromPaint,
    kUseFilterOptions,
    kUseCubicResampler,
}
impl LegacyFilterEnum {
    const kLast: Self = Self::kUseCubicResampler;
}

// Source: src/shaders/SkImageShader.cpp:133-166
impl SkImageShader {
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        fn tile(v: u32) -> SkTileMode {
            match v {
                0 => SkTileMode::kClamp,
                1 => SkTileMode::kRepeat,
                2 => SkTileMode::kMirror,
                _ => SkTileMode::kDecal,
            }
        }
        let tmx = tile(buffer.read32LE(SkTileMode::kDecal as u32));
        let tmy = tile(buffer.read32LE(SkTileMode::kDecal as u32));
        let mut sampling = SkSamplingOptions::default();
        let mut read_sampling = true;
        if buffer.isVersionLT(crate::src::core::SkPicturePriv::kNoFilterQualityShaders_Version)
            && !buffer.readBool()
        {
            read_sampling = false;
        }
        if read_sampling {
            sampling = buffer.readSampling();
        }
        let mut local = SkMatrix::default();
        if buffer.isVersionLT(crate::src::core::SkPicturePriv::kNoShaderLocalMatrix) {
            buffer.readMatrix(&mut local);
        }
        let Some(image) = buffer.readImage()? else {
            return Ok(None);
        };
        let raw = if buffer.isVersionLT(crate::src::core::SkPicturePriv::kRawImageShaders) {
            false
        } else {
            buffer.readBool()
        };
        let shader = if raw {
            Self::MakeRaw(Some(image), tmx, tmy, &sampling, Some(&local))
        } else {
            Self::Make(Some(image), tmx, tmy, &sampling, Some(&local), false)
        };
        Ok(shader.map(crate::src::core::SkFlattenable::FlattenableOwner::Shader))
    }
    // Source: src/shaders/SkImageShader.cpp:168-182
    // Source: src/shaders/SkImageShader.h:87-87
    // Source: src/shaders/SkImageShader.h:87
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeUInt(self.fTileModeX as u32);
        buffer.writeUInt(self.fTileModeY as u32);
        buffer.writeSampling(&self.fSampling);
        // The same retained immutable image supplies the actual writer's native
        // synchronous image/backing lease; no nullable or replacement producer.
        unsafe {
            buffer.writeImage(&*self.fImage);
        }
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),175; @condition b"fClampAsIfUnpremul == false\0".as_ptr(); !self.fClampAsIfUnpremul);
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),179; @condition b"!needs_subset(fImage.get(), fSubset)\0".as_ptr(); !needs_subset(self.fImage.as_ref(),&self.fSubset));
        buffer.writeBool(self.fRaw);

        Ok(())
    }
}
// Source: src/shaders/SkImageShader.cpp:184-186
impl SkShader for SkImageShader {
    // Source: src/shaders/SkImageShader.h:71-71
    fn isOpaque(&self) -> bool {
        self.fImage.isOpaque()
            && self.fTileModeX != SkTileMode::kDecal
            && self.fTileModeY != SkTileMode::kDecal
    }
}

// Source: src/shaders/SkImageShader.cpp:191-192
// Source: src/shaders/SkImageShader.cpp:195-196
// Source: src/shaders/SkImageShader.cpp:201-202
// Source: src/shaders/SkImageShader.cpp:205-208
// Source: src/shaders/SkImageShader.cpp:212-212
pub(crate) fn legacy_shader_can_handle(inv: &crate::src::core::SkMatrix::SkMatrix) -> bool {
    crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),192; @condition b"!inv.hasPerspective()\0".as_ptr(); !inv.hasPerspective());
    if crate::src::core::SkOpts::S32_alpha_D32_filter_DXDY.is_none() && !inv.isScaleTranslate() {
        return false;
    }
    let max_dev_coord = 32767.0f32;
    let src = inv.mapRect_value(&crate::src::core::SkRect::SkRect::MakeWH(
        max_dev_coord,
        max_dev_coord,
    ));
    let max_fixed32dot32 = (i32::MAX as f32) * 0.25f32;
    if !crate::src::core::SkRect::SkRect::MakeLTRB(
        -max_fixed32dot32,
        -max_fixed32dot32,
        max_fixed32dot32,
        max_fixed32dot32,
    )
    .contains(&src)
    {
        return false;
    }
    true
}

// Source: src/shaders/SkImageShader.cpp:215-219
// Source: src/shaders/SkImageShader.cpp:221-222
// Source: src/shaders/SkImageShader.cpp:224-225
// Source: src/shaders/SkImageShader.cpp:227-228
// Source: src/shaders/SkImageShader.cpp:231-233
// Source: src/shaders/SkImageShader.cpp:236-240
// Source: src/shaders/SkImageShader.cpp:242-244
// Source: src/shaders/SkImageShader.cpp:247-247
// Source: src/shaders/SkImageShader.cpp:249-250
// Source: src/shaders/SkImageShader.cpp:263-265
// Source: src/shaders/SkImageShader.cpp:268-270
// Source: src/shaders/SkImageShader.cpp:273-274
// Source: src/shaders/SkImageShader.cpp:277-278
impl SkImageShader {
    #[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
    unsafe fn legacy_context(
        &self,
        rec: &ContextRec<'_>,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Result<Option<*mut dyn Context>, crate::RasterError> {
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),217; @condition b"!needs_subset(fImage.get(), fSubset)\0".as_ptr(); !needs_subset(self.fImage.as_ref(), &self.fSubset));
        if self.fImage.alphaType()
            == crate::include::core::SkAlphaType::SkAlphaType::kUnpremul_SkAlphaType
        {
            return Ok(None);
        }
        if self.fImage.colorType() != crate::include::core::SkColorType::kN32_SkColorType {
            return Ok(None);
        }
        if self.fTileModeX != self.fTileModeY {
            return Ok(None);
        }
        if self.fTileModeX == SkTileMode::kDecal || self.fTileModeY == SkTileMode::kDecal {
            return Ok(None);
        }
        let mut sampling = self.fSampling;
        if sampling.isAniso() {
            sampling = crate::src::core::SkSamplingPriv::SkSamplingPriv::AnisoFallback(
                self.fImage.hasMipmaps(),
            );
        }
        let supported = |sampling: &SkSamplingOptions| {
            let supported = [
                (SkFilterMode::kNearest, SkMipmapMode::kNone),
                (SkFilterMode::kLinear, SkMipmapMode::kNone),
                (SkFilterMode::kLinear, SkMipmapMode::kNearest),
            ];
            for (f, m) in supported {
                if sampling.filter() == f && sampling.mipmap() == m {
                    return true;
                }
            }
            false
        };
        if sampling.useCubic() || !supported(&sampling) {
            return Ok(None);
        }
        if self.fImage.width() > 32767 || self.fImage.height() > 32767 {
            return Ok(None);
        }
        let Some(inv) = rec.fMatrixRec.totalInverse() else {
            return Ok(None);
        };
        if !legacy_shader_can_handle(&inv) {
            return Ok(None);
        }
        if !rec.isLegacyCompatible(self.fImage.colorSpace()) {
            return Ok(None);
        }
        unsafe {
            crate::src::shaders::SkBitmapProcShader::MakeContext(
                self,
                self.fTileModeX,
                self.fTileModeY,
                &sampling,
                Some(self.fImage.asImageBase()),
                rec,
                alloc,
            )
        }
    }
}
// Source: src/shaders/SkImageShader.cpp:282-284
// Source: src/shaders/SkImageShader.cpp:286-288
// Source: src/shaders/SkImageShader.cpp:290-290
impl SkImageShader {
    fn image_query(
        &self,
        texM: Option<&mut SkMatrix>,
        xy: Option<&mut [SkTileMode; 2]>,
    ) -> Option<&dyn SkImage> {
        if let Some(texM) = texM {
            *texM = SkMatrix::default();
        }
        if let Some(xy) = xy {
            xy[0] = self.fTileModeX;
            xy[1] = self.fTileModeY;
        }
        Some(self.fImage.as_ref())
    }
}
// Source: src/shaders/SkImageShader.cpp:293-299
impl SkImageShader {
    // Source: src/shaders/SkImageShader.h:30-35
    pub fn Make(
        image: Option<Arc<dyn SkImage>>,
        tmx: SkTileMode,
        tmy: SkTileMode,
        options: &SkSamplingOptions,
        localMatrix: Option<&SkMatrix>,
        clampAsIfUnpremul: bool,
    ) -> Option<Arc<dyn SkShader>> {
        let subset = if let Some(image) = &image {
            SkRect::MakeIWH(image.width(), image.height())
        } else {
            SkRect::MakeEmpty()
        };
        Self::MakeSubset(
            image,
            &subset,
            tmx,
            tmy,
            options,
            localMatrix,
            clampAsIfUnpremul,
        )
    }
}
// Source: src/shaders/SkImageShader.cpp:302-307
// Source: src/shaders/SkImageShader.cpp:309-310
// Source: src/shaders/SkImageShader.cpp:312-312
// Source: src/shaders/SkImageShader.cpp:314-320
impl SkImageShader {
    // Source: src/shaders/SkImageShader.h:37-41
    pub fn MakeRaw(
        image: Option<Arc<dyn SkImage>>,
        tmx: SkTileMode,
        tmy: SkTileMode,
        options: &SkSamplingOptions,
        localMatrix: Option<&SkMatrix>,
    ) -> Option<Arc<dyn SkShader>> {
        if options.useCubic() {
            return None;
        }
        let Some(image) = image else {
            return Some(crate::src::shaders::SkEmptyShader::Empty());
        };
        let subset = SkRect::MakeIWH(image.width(), image.height());
        // Source: src/shaders/SkImageShader.cpp:314-320.
        // Native passes its lvalue sk_sp image to sk_make_sp: retain the same
        // image for the shader while this parameter remains alive until return.
        let s: Arc<dyn SkShader> = Arc::new(Self::new(
            image.clone(),
            &subset,
            tmx,
            tmy,
            options,
            true,
            false,
        ));
        Some(s.makeWithLocalMatrix(localMatrix.unwrap_or(&SkMatrix::default())))
    }
}
// Source: src/shaders/SkImageShader.cpp:323-330
// Source: src/shaders/SkImageShader.cpp:332-334
// Source: src/shaders/SkImageShader.cpp:337-338
// Source: src/shaders/SkImageShader.cpp:342-343
// Source: src/shaders/SkImageShader.cpp:346-352
impl SkImageShader {
    // Source: src/shaders/SkImageShader.h:45-51
    pub fn MakeSubset(
        image: Option<Arc<dyn SkImage>>,
        subset: &SkRect,
        tmx: SkTileMode,
        tmy: SkTileMode,
        options: &SkSamplingOptions,
        localMatrix: Option<&SkMatrix>,
        clampAsIfUnpremul: bool,
    ) -> Option<Arc<dyn SkShader>> {
        let is_unit = |x: f32| x >= 0.0 && x <= 1.0;
        if options.useCubic() {
            let cubic = options.cubic();
            if !is_unit(cubic.B) || !is_unit(cubic.C) {
                return None;
            }
        }
        if image.is_none() || subset.isEmpty() {
            return Some(crate::src::shaders::SkEmptyShader::Empty());
        }
        let image = image.unwrap();
        if !SkRect::MakeIWH(image.width(), image.height()).contains(subset) {
            return None;
        }
        // Language-only constructor-pack instantiation of the real native helper.
        // This factory always requests makeWithLocalMatrix, including identity
        // when its own optional matrix is absent, exactly as its native body.
        let identity = SkMatrix::default();
        Some(
            crate::src::shaders::SkLocalMatrixShader::SkLocalMatrixShader::MakeWrapped(
                Some(localMatrix.unwrap_or(&identity)),
                || Self::new(image, subset, tmx, tmy, options, false, clampAsIfUnpremul),
            ),
        )
    }
}

// Source: src/shaders/SkImageShader.cpp:357-363
// Source: src/shaders/SkImageShader.cpp:366-366
// Source: src/shaders/SkImageShader.cpp:368-368
// Source: src/shaders/SkImageShader.cpp:370-374
// Source: src/shaders/SkImageShader.cpp:377-377
// Source: src/shaders/SkImageShader.cpp:380-380
// Source: src/shaders/SkImageShader.cpp:382-389
// Source: src/shaders/SkImageShader.cpp:391-392
// Source: src/shaders/SkImageShader.cpp:394-394
// Source: src/shaders/SkImageShader.cpp:398-398
// Source: src/shaders/SkImageShader.cpp:400-400
impl SkImageShader {
    // Arc reference is the native borrowed image plus its retained-owner capability; matrix and
    // sampling remain call-scoped/copy-only. No unsafe borrowed-to-owned pointer cast is used.
    // Source: src/shaders/SkImageShader.h:57-62
    pub fn MakeForDrawRect(
        image: &Arc<dyn SkImage>,
        paint: &crate::src::core::SkPaint::SkPaint,
        sampling: &SkSamplingOptions,
        mut src: SkRect,
        mut dst: SkRect,
        strictSrcSubset: bool,
    ) -> (SkRect, Option<Arc<dyn SkShader>>) {
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),366; @condition b"paint.getStyle() == SkPaint::kFill_Style && !paint.getPathEffect()\0".as_ptr(); paint.getStyle()==crate::src::core::SkPaint::Style::kFill_Style && paint.getPathEffect().is_none());
        let imgBounds = SkRect::MakeIWH(image.width(), image.height());
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),370; @condition b"src.isFinite() && dst.isFinite() && dst.isSorted()\0".as_ptr(); src.isFinite() && dst.isFinite() && dst.isSorted());
        let localMatrix = SkMatrix::RectToRectOrIdentity(
            &src,
            &dst,
            crate::src::core::SkMatrix::ScaleToFit::kFill_ScaleToFit,
        );
        if !imgBounds.contains(&src) {
            if !src.intersect(&imgBounds) {
                return (SkRect::MakeEmpty(), None);
            }
            dst = localMatrix.mapRect_value(&src);
        }
        let imageIsAlphaOnly =
            crate::src::core::SkImageInfoPriv::SkColorTypeIsAlphaOnly(image.colorType());
        let mut imgShader = if strictSrcSubset {
            Self::MakeSubset(
                Some(image.clone()),
                &src,
                SkTileMode::kClamp,
                SkTileMode::kClamp,
                sampling,
                Some(&localMatrix),
                false,
            )
        } else {
            image.makeShader(
                SkTileMode::kClamp,
                SkTileMode::kClamp,
                sampling,
                Some(&localMatrix),
            )
        };
        if imgShader.is_none() {
            return (SkRect::MakeEmpty(), None);
        }
        if imageIsAlphaOnly && paint.getShader().is_some() {
            imgShader = crate::src::shaders::SkBlendShader::Blend(
                crate::src::core::SkBlendMode::SkBlendMode::kDstIn,
                paint.refShader(),
                imgShader,
            );
        }
        (dst, imgShader)
    }
}

// Source: src/shaders/SkImageShader.cpp:403-403
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn RegisterFlattenables() {
    unsafe {
        crate::src::core::SkFlattenable::Register(b"SkImageShader", SkImageShader::CreateProc);
    }
}
// Source: src/shaders/SkImageShader.cpp:407-413
struct MipLevelHelper {
    pm: crate::src::core::SkPixmap::SkPixmap,
    inv: crate::src::core::SkMatrix::SkMatrix,
    gather: *mut crate::src::core::SkRasterPipelineOpContexts::GatherCtx,
    limitX: *mut crate::src::core::SkRasterPipelineOpContexts::TileCtx,
    limitY: *mut crate::src::core::SkRasterPipelineOpContexts::TileCtx,
    decalCtx: *mut crate::src::core::SkRasterPipelineOpContexts::DecalTileCtx,
}
// Native alloc->make<MipLevelHelper>() value initialization uses default Pixmap/Matrix and zero
// pointers. This does not supply image samples or replace required appended stage algorithms.
impl Default for MipLevelHelper {
    fn default() -> Self {
        Self {
            pm: crate::src::core::SkPixmap::SkPixmap::default(),
            inv: crate::src::core::SkMatrix::SkMatrix::default(),
            gather: core::ptr::null_mut(),
            limitX: core::ptr::null_mut(),
            limitY: core::ptr::null_mut(),
            decalCtx: core::ptr::null_mut(),
        }
    }
}
impl MipLevelHelper {
    // Source: src/shaders/SkImageShader.cpp:415-423
    // Source: src/shaders/SkImageShader.cpp:425-427
    // Source: src/shaders/SkImageShader.cpp:430-435
    // Source: src/shaders/SkImageShader.cpp:450-452
    // Source: src/shaders/SkImageShader.cpp:455-458
    // Source: src/shaders/SkImageShader.cpp:463-465
    /// Safety: arena remains live/unreset and contexts remain unmoved through all pipeline uses.
    /// pm references valid source pixels and initialized image metadata before this call.
    unsafe fn allocAndInit(
        &mut self,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
        sampling: &crate::include::core::SkSamplingOptions::SkSamplingOptions,
        tileModeX: SkTileMode,
        tileModeY: SkTileMode,
    ) {
        use crate::src::core::SkRasterPipelineOpContexts::{DecalTileCtx, GatherCtx, TileCtx};
        unsafe {
            self.gather = (*alloc).make(GatherCtx::new(core::ptr::null(), 0, 0.0, 0.0, [0.0; 16]));
            (*self.gather).pixels = self.pm.addr().cast();
            (*self.gather).stride = self.pm.rowBytesAsPixels();
            (*self.gather).width = self.pm.width() as f32;
            (*self.gather).height = self.pm.height() as f32;
            if sampling.useCubic() {
                let cubic = sampling.cubic();
                CubicResamplerMatrix(cubic.B, cubic.C).getColMajor(&mut (*self.gather).weights);
            }
            self.limitX = (*alloc).make(TileCtx::new(0.0, 0.0));
            self.limitY = (*alloc).make(TileCtx::new(0.0, 0.0));
            (*self.limitX).scale = self.pm.width() as f32;
            (*self.limitX).invScale = 1.0 / self.pm.width() as f32;
            (*self.limitY).scale = self.pm.height() as f32;
            (*self.limitY).invScale = 1.0 / self.pm.height() as f32;
            if !sampling.useCubic()
                && sampling.filter()
                    == crate::include::core::SkSamplingOptions::SkFilterMode::kNearest
            {
                (*self.gather).roundDownAtInteger = true;
                (*self.limitX).mirrorBiasDir = 1;
                (*self.limitY).mirrorBiasDir = 1;
            }
            if tileModeX == SkTileMode::kDecal || tileModeY == SkTileMode::kDecal {
                self.decalCtx = (*alloc).make(DecalTileCtx::new([0; 16], 0.0, 0.0));
                (*self.decalCtx).limit_x = (*self.limitX).scale;
                (*self.decalCtx).limit_y = (*self.limitY).scale;
                if (*self.gather).roundDownAtInteger {
                    (*self.decalCtx).inclusiveEdge_x = (*self.decalCtx).limit_x;
                    (*self.decalCtx).inclusiveEdge_y = (*self.decalCtx).limit_y;
                }
            }
        }
    }
}
// Source: src/shaders/SkImageShader.cpp:473-474
// Source: src/shaders/SkImageShader.cpp:477-481
// Source: src/shaders/SkImageShader.cpp:484-484
fn tweak_sampling(
    sampling: crate::include::core::SkSamplingOptions::SkSamplingOptions,
    matrix: &crate::src::core::SkMatrix::SkMatrix,
) -> crate::include::core::SkSamplingOptions::SkSamplingOptions {
    use crate::include::core::SkSamplingOptions::{SkFilterMode, SkSamplingOptions};
    let mut filter = sampling.filter();
    if filter == SkFilterMode::kLinear
        && matrix.getType() <= crate::src::core::SkMatrix::SkMatrix::kTranslate_Mask
        && matrix.getTranslateX() == (matrix.getTranslateX() as i32) as f32
        && matrix.getTranslateY() == (matrix.getTranslateY() as i32) as f32
    {
        filter = SkFilterMode::kNearest;
    }
    SkSamplingOptions::with_filter_mipmap(filter, sampling.mipmap())
}

// Source: src/shaders/SkImageShader.cpp:487-488
// Source: src/shaders/SkImageShader.cpp:491-493
// Source: src/shaders/SkImageShader.cpp:496-497
// Source: src/shaders/SkImageShader.cpp:499-499
// Source: src/shaders/SkImageShader.cpp:501-504
// Source: src/shaders/SkImageShader.cpp:506-507
// Source: src/shaders/SkImageShader.cpp:510-513
// Source: src/shaders/SkImageShader.cpp:516-517
// Source: src/shaders/SkImageShader.cpp:519-519
// Source: src/shaders/SkImageShader.cpp:521-522
// Source: src/shaders/SkImageShader.cpp:526-527
// Source: src/shaders/SkImageShader.cpp:530-530
// Source: src/shaders/SkImageShader.cpp:532-540
// Source: src/shaders/SkImageShader.cpp:542-542
// Source: src/shaders/SkImageShader.cpp:544-544
// Source: src/shaders/SkImageShader.cpp:547-547
// Source: src/shaders/SkImageShader.cpp:549-564
// Source: src/shaders/SkImageShader.cpp:566-577
// Source: src/shaders/SkImageShader.cpp:581-593
// Source: src/shaders/SkImageShader.cpp:595-597
// Source: src/shaders/SkImageShader.cpp:599-601
// Source: src/shaders/SkImageShader.cpp:603-610
// Source: src/shaders/SkImageShader.cpp:612-613
// Source: src/shaders/SkImageShader.cpp:615-616
// Source: src/shaders/SkImageShader.cpp:618-627
// Source: src/shaders/SkImageShader.cpp:629-632
// Source: src/shaders/SkImageShader.cpp:634-638
// Source: src/shaders/SkImageShader.cpp:640-644
// Source: src/shaders/SkImageShader.cpp:646-649
// Source: src/shaders/SkImageShader.cpp:651-654
// Source: src/shaders/SkImageShader.cpp:656-656
// Source: src/shaders/SkImageShader.cpp:658-659
// Source: src/shaders/SkImageShader.cpp:663-665
// Source: src/shaders/SkImageShader.cpp:670-671
// Source: src/shaders/SkImageShader.cpp:673-674
// Source: src/shaders/SkImageShader.cpp:678-681
// Source: src/shaders/SkImageShader.cpp:685-686
// Source: src/shaders/SkImageShader.cpp:689-689
// Source: src/shaders/SkImageShader.cpp:694-698
// Source: src/shaders/SkImageShader.cpp:701-710
// Source: src/shaders/SkImageShader.cpp:715-718
// Source: src/shaders/SkImageShader.cpp:721-722
// Source: src/shaders/SkImageShader.cpp:724-724
// Source: src/shaders/SkImageShader.cpp:726-728
// Source: src/shaders/SkImageShader.cpp:730-732
// Source: src/shaders/SkImageShader.cpp:734-734
// Source: src/shaders/SkImageShader.cpp:738-739
// Source: src/shaders/SkImageShader.cpp:741-747
// Source: src/shaders/SkImageShader.cpp:750-752
// Source: src/shaders/SkImageShader.cpp:754-754
// Source: src/shaders/SkImageShader.cpp:756-759
// Source: src/shaders/SkImageShader.cpp:761-764
// Source: src/shaders/SkImageShader.cpp:766-769
// Source: src/shaders/SkImageShader.cpp:771-774
// Source: src/shaders/SkImageShader.cpp:776-778
// Source: src/shaders/SkImageShader.cpp:780-783
// Source: src/shaders/SkImageShader.cpp:785-787
// Source: src/shaders/SkImageShader.cpp:791-791
// Source: src/shaders/SkImageShader.cpp:793-796
// Source: src/shaders/SkImageShader.cpp:799-799
impl SkImageShader {
    /// Safety: retain this shader and referenced pixels, pipeline, and arena at stable addresses
    /// for all appended stage executions; uphold their exclusive destination/source lease rules.
    unsafe fn append_stages(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        mRec: &crate::src::shaders::SkShaderBase::MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        use crate::include::core::SkAlphaType::SkAlphaType::{
            kPremul_SkAlphaType, kUnpremul_SkAlphaType,
        };
        use crate::include::core::SkColorType::SkColorType::*;
        use crate::include::core::SkSamplingOptions::{SkFilterMode, SkMipmapMode};
        use crate::src::core::SkRasterPipelineOpContexts::{MipmapCtx, SamplerCtx};
        use crate::src::core::SkRasterPipelineOpList::SkRasterPipelineOp as Op;
        #[cfg(sk_debug)]
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),488; @condition b"!needs_subset(fImage.get(), fSubset)\0".as_ptr(); !needs_subset(self.fImage.as_ref(), &self.fSubset));
        let mut sampling = self.fSampling;
        if sampling.isAniso() {
            sampling = crate::src::core::SkSamplingPriv::SkSamplingPriv::AnisoFallback(
                self.fImage.hasMipmaps(),
            );
        }
        let p = rec.fPipeline;
        let alloc = rec.fAlloc;
        let mut baseInv = crate::src::core::SkMatrix::SkMatrix::default();
        if mRec.totalMatrixIsValid() {
            let Some(inv) = mRec.totalInverse() else {
                return Ok(false);
            };
            baseInv = inv;
            baseInv.normalizePerspective();
        }
        crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),510; @condition b"!sampling.useCubic || sampling.mipmap == SkMipmapMode::kNone\0".as_ptr(); !sampling.useCubic() || sampling.mipmap() == SkMipmapMode::kNone);
        let Some(access) = (unsafe {
            crate::src::core::SkMipmapAccessor::SkMipmapAccessor::Make(
                &mut *alloc,
                self.fImage.as_ref(),
                &baseInv,
                sampling.mipmap(),
            )
        })?
        else {
            return Ok(false);
        };
        let access = unsafe { access.as_ref() };
        let mut upper = MipLevelHelper::default();
        (upper.pm, upper.inv) = access.level();
        if !sampling.useCubic() {
            if mRec.totalMatrixIsValid() {
                let mut concat = crate::src::core::SkMatrix::SkMatrix::default();
                concat.setConcat(&upper.inv, &baseInv);
                sampling = tweak_sampling(sampling, &concat);
            }
        }
        if unsafe { mRec.apply(rec, &upper.inv) }.is_none() {
            return Ok(false);
        }
        unsafe {
            upper.allocAndInit(alloc, &sampling, self.fTileModeX, self.fTileModeY);
        }
        let mut lower = MipLevelHelper::default();
        let mut mipmapCtx: *mut MipmapCtx = core::ptr::null_mut();
        let lowerWeight = access.lowerWeight();
        if lowerWeight > 0.0 {
            (lower.pm, lower.inv) = access.lowerLevel();
            mipmapCtx = unsafe {
                (*alloc).make(MipmapCtx {
                    x: [0.0; 16],
                    y: [0.0; 16],
                    r: [0.0; 16],
                    g: [0.0; 16],
                    b: [0.0; 16],
                    a: [0.0; 16],
                    scaleX: 0.0,
                    scaleY: 0.0,
                    lowerWeight: 0.0,
                })
            };
            unsafe {
                (*mipmapCtx).lowerWeight = lowerWeight;
                (*mipmapCtx).scaleX = lower.pm.width() as f32 / upper.pm.width() as f32;
                (*mipmapCtx).scaleY = lower.pm.height() as f32 / upper.pm.height() as f32;
                lower.allocAndInit(alloc, &sampling, self.fTileModeX, self.fTileModeY);
                (*p).append(Op::mipmap_linear_init, mipmapCtx.cast());
            }
        }
        let decalBothAxes =
            self.fTileModeX == SkTileMode::kDecal && self.fTileModeY == SkTileMode::kDecal;
        let append_tiling_and_gather = |level: &MipLevelHelper| unsafe {
            if decalBothAxes {
                (*p).append(Op::decal_x_and_y, level.decalCtx.cast());
            } else {
                match self.fTileModeX {
                    SkTileMode::kClamp => {}
                    SkTileMode::kMirror => (*p).append(Op::mirror_x, level.limitX.cast()),
                    SkTileMode::kRepeat => (*p).append(Op::repeat_x, level.limitX.cast()),
                    SkTileMode::kDecal => (*p).append(Op::decal_x, level.decalCtx.cast()),
                }
                match self.fTileModeY {
                    SkTileMode::kClamp => {}
                    SkTileMode::kMirror => (*p).append(Op::mirror_y, level.limitY.cast()),
                    SkTileMode::kRepeat => (*p).append(Op::repeat_y, level.limitY.cast()),
                    SkTileMode::kDecal => (*p).append(Op::decal_y, level.decalCtx.cast()),
                }
            }
            let ctx = level.gather.cast();
            let null = core::ptr::null_mut();
            match level.pm.colorType() {
                kAlpha_8_SkColorType => (*p).append(Op::gather_a8, ctx),
                kA16_unorm_SkColorType => (*p).append(Op::gather_a16, ctx),
                kA16_float_SkColorType => (*p).append(Op::gather_af16, ctx),
                kR16_float_SkColorType => (*p).append(Op::gather_rf16, ctx),
                kRGB_565_SkColorType => (*p).append(Op::gather_565, ctx),
                kARGB_4444_SkColorType => (*p).append(Op::gather_4444, ctx),
                kR8G8_unorm_SkColorType => (*p).append(Op::gather_rg88, ctx),
                kR16_unorm_SkColorType => (*p).append(Op::gather_r16, ctx),
                kR16G16_unorm_SkColorType => (*p).append(Op::gather_rg1616, ctx),
                kR16G16_float_SkColorType => (*p).append(Op::gather_rgf16, ctx),
                kRGBA_8888_SkColorType => (*p).append(Op::gather_8888, ctx),
                kRGBA_1010102_SkColorType => (*p).append(Op::gather_1010102, ctx),
                kR16G16B16A16_unorm_SkColorType => (*p).append(Op::gather_16161616, ctx),
                kRGBA_F16Norm_SkColorType | kRGBA_F16_SkColorType => {
                    (*p).append(Op::gather_f16, ctx)
                }
                kRGBA_F32_SkColorType => (*p).append(Op::gather_f32, ctx),
                kBGRA_10101010_XR_SkColorType => {
                    (*p).append(Op::gather_10101010_xr, ctx);
                    (*p).append(Op::swap_rb, null);
                }
                kRGBA_10x6_SkColorType => (*p).append(Op::gather_10x6, ctx),
                kGray_8_SkColorType => {
                    (*p).append(Op::gather_a8, ctx);
                    (*p).append(Op::alpha_to_gray, null);
                }
                kR8_unorm_SkColorType => {
                    (*p).append(Op::gather_a8, ctx);
                    (*p).append(Op::alpha_to_red, null);
                }
                kRGB_888x_SkColorType => {
                    (*p).append(Op::gather_8888, ctx);
                    (*p).append(Op::force_opaque, null);
                }
                kRGB_F16F16F16x_SkColorType => {
                    (*p).append(Op::gather_f16, ctx);
                    (*p).append(Op::force_opaque, null);
                }
                kBGRA_1010102_SkColorType => {
                    (*p).append(Op::gather_1010102, ctx);
                    (*p).append(Op::swap_rb, null);
                }
                kRGB_101010x_SkColorType => {
                    (*p).append(Op::gather_1010102, ctx);
                    (*p).append(Op::force_opaque, null);
                }
                kBGR_101010x_XR_SkColorType => {
                    (*p).append(Op::gather_1010102_xr, ctx);
                    (*p).append(Op::force_opaque, null);
                    (*p).append(Op::swap_rb, null);
                }
                kBGR_101010x_SkColorType => {
                    (*p).append(Op::gather_1010102, ctx);
                    (*p).append(Op::force_opaque, null);
                    (*p).append(Op::swap_rb, null);
                }
                kBGRA_8888_SkColorType => {
                    (*p).append(Op::gather_8888, ctx);
                    (*p).append(Op::swap_rb, null);
                }
                kSRGBA_8888_SkColorType => {
                    (*p).append(Op::gather_8888, ctx);
                    (*p).appendTransferFunction(
                        crate::modules::skcms::skcms::skcms_sRGB_TransferFunction(),
                    );
                }
                kUnknown_SkColorType => {
                    crate::SkASSERT!(@source b"src/shaders/SkImageShader.cpp\0".as_ptr(),656; @condition b"false\0".as_ptr(); false)
                }
            }
            if !level.decalCtx.is_null() {
                (*p).append(Op::check_decal_mask, level.decalCtx.cast());
            }
        };
        let append_misc = || unsafe {
            let mut cs = upper.pm.colorSpace();
            let mut at = upper.pm.alphaType();
            if crate::src::core::SkImageInfoPriv::SkColorTypeIsAlphaOnly(upper.pm.colorType())
                && !self.fRaw
            {
                (*p).appendSetRGB(
                    alloc,
                    &[rec.fPaintColor.fR, rec.fPaintColor.fG, rec.fPaintColor.fB],
                );
                cs = rec.fDstCS.map(Arc::as_ref);
                at = kUnpremul_SkAlphaType;
            }
            if sampling.useCubic() {
                (*p).append(
                    if at == kUnpremul_SkAlphaType || self.fClampAsIfUnpremul {
                        Op::clamp_01
                    } else {
                        Op::clamp_gamut
                    },
                    core::ptr::null_mut(),
                );
            }
            if !self.fRaw {
                let steps = (*alloc).make(
                    crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
                        cs,
                        at,
                        rec.fDstCS.map(Arc::as_ref),
                        kPremul_SkAlphaType,
                    ),
                );
                (*steps).applyPipeline(&mut *p);
            }
            true
        };
        let ct = upper.pm.colorType();
        if (ct == kRGBA_8888_SkColorType || ct == kBGRA_8888_SkColorType)
            && !sampling.useCubic()
            && sampling.filter() == SkFilterMode::kLinear
            && sampling.mipmap() != SkMipmapMode::kLinear
            && self.fTileModeX == SkTileMode::kClamp
            && self.fTileModeY == SkTileMode::kClamp
        {
            let mut shouldUseHighPBilerp = false;
            if !rec.fDstBounds.isEmpty() {
                let mut quad = rec
                    .fDstBounds
                    .toQuad(crate::include::core::SkPathTypes::SkPathDirection::kCW);
                baseInv.mapPointsInPlace(&mut quad);
                let mut deviceImageSpace = crate::src::core::SkRect::SkRect::default();
                deviceImageSpace.setBounds(&quad);
                for val in deviceImageSpace.asScalars() {
                    if *val > i16::MAX as f32 || *val < i16::MIN as f32 || !val.is_finite() {
                        shouldUseHighPBilerp = true;
                        break;
                    }
                }
            }
            unsafe {
                (*p).append(
                    if shouldUseHighPBilerp {
                        Op::bilerp_clamp_8888_force_highp
                    } else {
                        Op::bilerp_clamp_8888
                    },
                    upper.gather.cast(),
                );
                if ct == kBGRA_8888_SkColorType {
                    (*p).append(Op::swap_rb, core::ptr::null_mut());
                }
            }
            return Ok(append_misc());
        }
        if (ct == kRGBA_8888_SkColorType || ct == kBGRA_8888_SkColorType)
            && sampling.useCubic()
            && self.fTileModeX == SkTileMode::kClamp
            && self.fTileModeY == SkTileMode::kClamp
        {
            unsafe {
                (*p).append(Op::bicubic_clamp_8888, upper.gather.cast());
                if ct == kBGRA_8888_SkColorType {
                    (*p).append(Op::swap_rb, core::ptr::null_mut());
                }
            }
            return Ok(append_misc());
        }
        let sampler = unsafe {
            (*alloc).make(SamplerCtx {
                x: [0.0; 16],
                y: [0.0; 16],
                fx: [0.0; 16],
                fy: [0.0; 16],
                scalex: [0.0; 16],
                scaley: [0.0; 16],
                weights: [0.0; 16],
                wx: [[0.0; 16]; 4],
                wy: [[0.0; 16]; 4],
            })
        };
        let sample = |setup_x: Op, setup_y: Op, level: &MipLevelHelper| unsafe {
            (*p).append(setup_x, sampler.cast());
            (*p).append(setup_y, sampler.cast());
            append_tiling_and_gather(level);
            (*p).append(Op::accumulate, sampler.cast());
        };
        let sample_level = |level: &MipLevelHelper| unsafe {
            if sampling.useCubic() {
                let cubic = sampling.cubic();
                CubicResamplerMatrix(cubic.B, cubic.C).getColMajor(&mut (*sampler).weights);
                (*p).append(Op::bicubic_setup, sampler.cast());
                sample(Op::bicubic_n3x, Op::bicubic_n3y, level);
                sample(Op::bicubic_n1x, Op::bicubic_n3y, level);
                sample(Op::bicubic_p1x, Op::bicubic_n3y, level);
                sample(Op::bicubic_p3x, Op::bicubic_n3y, level);
                sample(Op::bicubic_n3x, Op::bicubic_n1y, level);
                sample(Op::bicubic_n1x, Op::bicubic_n1y, level);
                sample(Op::bicubic_p1x, Op::bicubic_n1y, level);
                sample(Op::bicubic_p3x, Op::bicubic_n1y, level);
                sample(Op::bicubic_n3x, Op::bicubic_p1y, level);
                sample(Op::bicubic_n1x, Op::bicubic_p1y, level);
                sample(Op::bicubic_p1x, Op::bicubic_p1y, level);
                sample(Op::bicubic_p3x, Op::bicubic_p1y, level);
                sample(Op::bicubic_n3x, Op::bicubic_p3y, level);
                sample(Op::bicubic_n1x, Op::bicubic_p3y, level);
                sample(Op::bicubic_p1x, Op::bicubic_p3y, level);
                sample(Op::bicubic_p3x, Op::bicubic_p3y, level);
                (*p).append(Op::move_dst_src, core::ptr::null_mut());
            } else if sampling.filter() == SkFilterMode::kLinear {
                (*p).append(Op::bilinear_setup, sampler.cast());
                sample(Op::bilinear_nx, Op::bilinear_ny, level);
                sample(Op::bilinear_px, Op::bilinear_ny, level);
                sample(Op::bilinear_nx, Op::bilinear_py, level);
                sample(Op::bilinear_px, Op::bilinear_py, level);
                (*p).append(Op::move_dst_src, core::ptr::null_mut());
            } else {
                append_tiling_and_gather(level);
            }
        };
        sample_level(&upper);
        if !mipmapCtx.is_null() {
            unsafe {
                (*p).append(Op::mipmap_linear_update, mipmapCtx.cast());
            }
            sample_level(&lower);
            unsafe {
                (*p).append(Op::mipmap_linear_finish, mipmapCtx.cast());
            }
        }
        Ok(append_misc())
    }
}

impl SkShaderBase for SkImageShader {
    fn r#type(&self) -> ShaderType {
        self.header_type()
    }
    // Source: src/shaders/SkImageShader.h:93-93
    unsafe fn appendStages(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        mRec: &MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.append_stages(rec, mRec) }
    }
    #[cfg(all(target_arch = "aarch64", target_pointer_width = "64"))]
    // Source: src/shaders/SkImageShader.h:89-89
    unsafe fn onMakeContext(
        &self,
        rec: &ContextRec<'_>,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Result<Option<*mut dyn Context>, crate::RasterError> {
        unsafe { self.legacy_context(rec, alloc) }
    }
    // Source: src/shaders/SkImageShader.h:91-91
    fn onIsAImage(
        &self,
        matrix: Option<&mut SkMatrix>,
        xy: Option<&mut [SkTileMode; 2]>,
    ) -> Option<&dyn SkImage> {
        self.image_query(matrix, xy)
    }
}
// Source: src/shaders/SkImageShader.cpp:804-808
pub fn Image(
    image: Option<Arc<dyn SkImage>>,
    tmx: SkTileMode,
    tmy: SkTileMode,
    options: &SkSamplingOptions,
    localMatrix: Option<&SkMatrix>,
) -> Option<Arc<dyn SkShader>> {
    SkImageShader::Make(image, tmx, tmy, options, localMatrix, false)
}
// Source: src/shaders/SkImageShader.cpp:811-815
pub fn RawImage(
    image: Option<Arc<dyn SkImage>>,
    tmx: SkTileMode,
    tmy: SkTileMode,
    options: &SkSamplingOptions,
    localMatrix: Option<&SkMatrix>,
) -> Option<Arc<dyn SkShader>> {
    SkImageShader::MakeRaw(image, tmx, tmy, options, localMatrix)
}

// Copyright 2014 Google Inc. BSD-style license.
use crate::include::core::{
    SkAlphaType::SkAlphaType::kPremul_SkAlphaType,
    SkColorType::SkColorType,
    SkRefCnt::sk_sp,
    SkSamplingOptions::SkFilterMode,
    SkSize::SkSize,
    SkSurfaceProps::{SkPixelGeometry::kUnknown_SkPixelGeometry, SkSurfaceProps},
    SkTileMode::SkTileMode,
};
use crate::src::core::{
    SkColorSpace::SkColorSpace,
    SkImageInfo::SkImageInfo,
    SkMatrix::{ScaleToFit, SkMatrix},
    SkPicture::SkPicture,
    SkPoint::SkPoint,
    SkRect::SkRect,
    SkResourceCache::{Key, Rec, RecLinks},
    SkScalar::{SK_ScalarNearlyZero, SkScalarNearlyZero},
};
use crate::src::image::SkImage::SkImage;
use std::sync::Arc;
// Source: src/shaders/SkPictureShader.h:47-56
impl SkPictureShader {
    pub fn header_type(&self) -> super::SkShaderBase::ShaderType {
        super::SkShaderBase::ShaderType::kPicture
    }
    pub fn isOpaque(&self) -> bool {
        false
    }
    pub fn picture(&self) -> sk_sp<dyn SkPicture> {
        self.fPicture.clone()
    }
    pub fn tile(&self) -> SkRect {
        self.fTile
    }
    pub fn tileModeX(&self) -> SkTileMode {
        self.fTmx
    }
    pub fn tileModeY(&self) -> SkTileMode {
        self.fTmy
    }
    pub fn filter(&self) -> SkFilterMode {
        self.fFilter
    }
}
// Source: src/shaders/SkPictureShader.h:58-64
pub struct CachedImageInfo {
    pub success: bool,
    pub tileScale: SkSize,
    pub matrixForDraw: SkMatrix,
    pub imageInfo: SkImageInfo,
    pub props: SkSurfaceProps,
}
// Source: src/shaders/SkPictureShader.h:79-79
impl super::SkShaderBase::SkShaderBase for SkPictureShader {
    fn r#type(&self) -> super::SkShaderBase::ShaderType {
        self.header_type()
    }
    unsafe fn appendStages(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        mRec: &super::SkShaderBase::MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        unsafe { self.append_stages(rec, mRec) }
    }
    // Source: src/shaders/SkPictureShader.h:80-82
    // Source: src/shaders/SkPictureShader.cpp:311-320
    #[cfg(SK_ENABLE_LEGACY_SHADERCONTEXT)]
    unsafe fn onMakeContext(
        &self,
        rec: &super::SkShaderBase::ContextRec<'_>,
        alloc: *mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Result<Option<*mut dyn super::SkShaderBase::Context>, crate::RasterError> {
        let bitmapShader = unsafe {
            self.rasterShader(
                &rec.fMatrixRec.totalMatrix(),
                rec.fDstColorType,
                rec.fDstColorSpace,
                &rec.fProps,
            )
        }?;
        let Some(bitmapShader) = bitmapShader else {
            return Ok(None);
        };
        // Native local sk_sp is released on return. The actual bitmap context
        // captures its BitmapProcState/image backing, not a retained shader.
        unsafe { bitmapShader.makeContext(rec, alloc) }
    }
}
impl super::SkShader::SkShader for SkPictureShader {
    fn isOpaque(&self) -> bool {
        SkPictureShader::isOpaque(self)
    }
}
// Source: src/shaders/SkPictureShader.h:85-85
impl crate::src::core::SkFlattenable::SkFlattenableView for SkPictureShader {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkPictureShader")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkShader_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)
    }
}
// Source: src/shaders/SkPictureShader.h:92-95
// Retain the original intrusive Picture owner; scalar members have no teardown
// effects, so its field release has the same observable derived/base lifetime.
// Source: src/shaders/SkPictureShader.h:40
pub struct SkPictureShader {
    fPicture: sk_sp<dyn SkPicture>,
    fTile: SkRect,
    fTmx: SkTileMode,
    fTmy: SkTileMode,
    fFilter: SkFilterMode,
}
// Source: src/shaders/SkPictureShader.cpp:39-45
// A borrowed canonical intrusive handle replaces C++'s sk_ref_sp(this) receiver;
// cloning it retains the SAME complete Picture allocation and sole counter.
pub fn PictureMakeShader(
    picture: &sk_sp<dyn SkPicture>,
    tmx: SkTileMode,
    tmy: SkTileMode,
    filter: SkFilterMode,
    localMatrix: Option<&SkMatrix>,
    tile: Option<&SkRect>,
) -> Option<Arc<dyn super::SkShader::SkShader>> {
    if localMatrix.is_some_and(|matrix| matrix.invert().is_none()) {
        return None;
    }
    Some(SkPictureShader::Make(
        Some(picture.clone()),
        tmx,
        tmy,
        filter,
        localMatrix,
        tile,
    ))
}
// Source: src/shaders/SkPictureShader.cpp:48-48
static gImageFromPictureKeyNamespaceLabel: u32 = 0;
// Source: src/shaders/SkPictureShader.cpp:50-84
// The existing ResourceCache::Key owns native packed words. Serializing the
// actual named fields in native order removes C++ derived-tail layout/padding;
// no pointer, color profile, subset or surface-property identity is discarded.
#[derive(Clone)]
struct ImageFromPictureKey {
    key: Key,
}
impl ImageFromPictureKey {
    fn new(
        cs: &SkColorSpace,
        ct: SkColorType,
        pictureID: u32,
        subset: &SkRect,
        scale: SkSize,
        props: &SkSurfaceProps,
    ) -> Self {
        let contents = [
            cs.toXYZD50Hash(),
            cs.transferFnHash(),
            ct as u32,
            subset.fLeft.to_bits(),
            subset.fTop.to_bits(),
            subset.fRight.to_bits(),
            subset.fBottom.to_bits(),
            scale.fWidth.to_bits(),
            scale.fHeight.to_bits(),
            props.fFlags,
            props.fPixelGeometry as u32,
            props.fTextContrast.to_bits(),
            props.fTextGamma.to_bits(),
        ];
        const KEY_SIZE: usize = 4 + 4 + 4 + 16 + 8 + 16;
        crate::SkASSERT!(@source b"src/shaders/SkPictureShader.cpp\0".as_ptr(),69; @condition b"sizeof(uint32_t) * (&fEndOfStruct - &fColorSpaceXYZHash) == keySize\0".as_ptr(); std::mem::size_of_val(&contents)==KEY_SIZE);
        Self {
            key: Key::new(
                core::ptr::addr_of!(gImageFromPictureKeyNamespaceLabel).cast(),
                crate::src::core::SkPicturePriv::MakeSharedID(pictureID),
                &contents,
            ),
        }
    }
}
// Source: src/shaders/SkPictureShader.cpp:86-109
// Image drops before key as in native reverse field teardown; intrusive cache
// links belong to this actual stable boxed record, not to a surrogate object.
struct ImageFromPictureRec {
    fImage: Arc<dyn SkImage>,
    fKey: ImageFromPictureKey,
    links: RecLinks,
}
// The cached image is immutable after construction, matching native SkImage's
// cross-thread retained-image contract. Only the owning cache mutates links.
unsafe impl Send for ImageFromPictureRec {}
impl ImageFromPictureRec {
    fn new(key: &ImageFromPictureKey, image: Arc<dyn SkImage>) -> Self {
        Self {
            fKey: key.clone(),
            fImage: image,
            links: RecLinks::new(),
        }
    }
    unsafe fn Visitor(base: &dyn Rec, context: *mut core::ffi::c_void) -> bool {
        unsafe {
            // The unique native key namespace admits only this genuine concrete Rec.
            let rec = &*(base as *const dyn Rec as *const Self);
            *context.cast::<Option<Arc<dyn SkImage>>>() = Some(rec.fImage.clone());
            true
        }
    }
}
unsafe impl Rec for ImageFromPictureRec {
    fn links(&self) -> &RecLinks {
        &self.links
    }
    fn links_mut(&mut self) -> &mut RecLinks {
        &mut self.links
    }
    fn getKey(&self) -> &Key {
        &self.fKey.key
    }
    fn bytesUsed(&self) -> usize {
        std::mem::size_of_val(&self.fKey)
            + self.fImage.width() as usize * self.fImage.height() as usize * 4
    }
    fn getCategory(&self) -> *const core::ffi::c_char {
        b"bitmap-shader\0".as_ptr().cast()
    }
    fn diagnostic_only_getDiscardable(
        &self,
    ) -> Option<
        core::ptr::NonNull<
            dyn crate::include::private::chromium::SkDiscardableMemory::SkDiscardableMemory,
        >,
    > {
        None
    }
}
// Source: src/shaders/SkPictureShader.h:45-45
// Source: src/shaders/SkPictureShader.cpp:113-122
impl SkPictureShader {
    pub fn new(
        picture: sk_sp<dyn SkPicture>,
        tmx: SkTileMode,
        tmy: SkTileMode,
        filter: SkFilterMode,
        tile: Option<&SkRect>,
    ) -> Self {
        let fTile = tile.copied().unwrap_or_else(|| picture.cullRect());
        Self {
            fPicture: picture,
            fTile,
            fTmx: tmx,
            fTmy: tmy,
            fFilter: filter,
        }
    }
}
// Source: src/shaders/SkPictureShader.h:42-43
// Source: src/shaders/SkPictureShader.cpp:124-134
impl SkPictureShader {
    pub fn Make(
        picture: Option<sk_sp<dyn SkPicture>>,
        tmx: SkTileMode,
        tmy: SkTileMode,
        filter: SkFilterMode,
        lm: Option<&SkMatrix>,
        tile: Option<&SkRect>,
    ) -> Arc<dyn super::SkShader::SkShader> {
        let Some(picture) = picture else {
            return super::SkEmptyShader::Empty();
        };
        if picture.cullRect().isEmpty() || tile.is_some_and(|tile| tile.isEmpty()) {
            return super::SkEmptyShader::Empty();
        }
        super::SkLocalMatrixShader::SkLocalMatrixShader::MakeWrapped(lm, || {
            Self::new(picture, tmx, tmy, filter, tile)
        })
    }
    // Source: src/shaders/SkPictureShader.cpp:136-166
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        use crate::src::core::SkPicturePriv;
        let mut lm = SkMatrix::default();
        if buffer.isVersionLT(SkPicturePriv::kNoShaderLocalMatrix) {
            buffer.readMatrix(&mut lm);
        }
        let tile_mode = |v| match v {
            0 => SkTileMode::kClamp,
            1 => SkTileMode::kRepeat,
            2 => SkTileMode::kMirror,
            _ => SkTileMode::kDecal,
        };
        let tmx = tile_mode(buffer.read32LE(SkTileMode::kDecal as u32));
        let tmy = tile_mode(buffer.read32LE(SkTileMode::kDecal as u32));
        let tile = buffer.readRect();
        let mut picture = None;
        let mut filter = SkFilterMode::kNearest;
        if buffer.isVersionLT(SkPicturePriv::kNoFilterQualityShaders_Version) {
            if buffer.isVersionLT(SkPicturePriv::kPictureShaderFilterParam_Version) {
                if buffer.readBool() {
                    picture = crate::src::core::SkPicture::PicturePriv_MakeFromBuffer(buffer)?;
                }
            } else {
                let legacyFilter = buffer.read32() as u32;
                if legacyFilter <= SkFilterMode::kLinear as u32 {
                    filter = if legacyFilter == 0 {
                        SkFilterMode::kNearest
                    } else {
                        SkFilterMode::kLinear
                    };
                }
                picture = crate::src::core::SkPicture::PicturePriv_MakeFromBuffer(buffer)?;
            }
        } else {
            filter = if buffer.read32LE(SkFilterMode::kLinear as u32) == 0 {
                SkFilterMode::kNearest
            } else {
                SkFilterMode::kLinear
            };
            picture = crate::src::core::SkPicture::PicturePriv_MakeFromBuffer(buffer)?;
        }
        Ok(Some(
            crate::src::core::SkFlattenable::FlattenableOwner::Shader(Self::Make(
                picture,
                tmx,
                tmy,
                filter,
                Some(&lm),
                Some(&tile),
            )),
        ))
    }
    // Source: src/shaders/SkPictureShader.h:78-78
    // Source: src/shaders/SkPictureShader.cpp:168-174
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeUInt(self.fTmx as u32);
        buffer.writeUInt(self.fTmy as u32);
        buffer.writeRect(&self.fTile);
        buffer.writeUInt(self.fFilter as u32);
        crate::src::core::SkPicturePriv::Flatten(self.fPicture.clone(), buffer)?;
        Ok(())
    }
}
// Source: src/shaders/SkPictureShader.cpp:176-178
fn ref_or_srgb(cs: Option<&Arc<SkColorSpace>>) -> Arc<SkColorSpace> {
    cs.cloned().unwrap_or_else(SkColorSpace::MakeSRGB)
}
// Source: src/shaders/SkPictureShader.h:65-70
// Source: src/shaders/SkPictureShader.cpp:180-242
impl CachedImageInfo {
    pub fn Make(
        bounds: &SkRect,
        totalM: &SkMatrix,
        dstColorType: SkColorType,
        dstColorSpace: Option<&Arc<SkColorSpace>>,
        maxTextureSize: i32,
        propsIn: &SkSurfaceProps,
    ) -> Self {
        let props = propsIn.cloneWithPixelGeometry(kUnknown_SkPixelGeometry);
        let scaledSize = {
            let mut size = SkSize {
                fWidth: 0.0,
                fHeight: 0.0,
            };
            if !totalM.decomposeScale(Some(&mut size), None) {
                let center = SkPoint {
                    fX: bounds.centerX(),
                    fY: bounds.centerY(),
                };
                let area = crate::src::core::SkMatrixPriv::SkMatrixPriv::DifferentialAreaScale(
                    totalM, &center,
                );
                if !area.is_finite() || SkScalarNearlyZero(area, SK_ScalarNearlyZero) {
                    size = SkSize {
                        fWidth: 1.0,
                        fHeight: 1.0,
                    };
                } else {
                    size.fWidth = area.sqrt();
                    size.fHeight = area.sqrt();
                }
            }
            size.fWidth *= bounds.width();
            size.fHeight *= bounds.height();
            const MAX_TILE_AREA: f32 = (2048 * 2048) as f32;
            let tileArea = size.width() * size.height();
            if tileArea > MAX_TILE_AREA {
                let clampScale = (MAX_TILE_AREA / tileArea).sqrt();
                size.set(size.width() * clampScale, size.height() * clampScale);
            }
            if maxTextureSize != 0 {
                if size.width() > maxTextureSize as f32 || size.height() > maxTextureSize as f32 {
                    // std::max's conditional comparison retains native NaN ordering.
                    let largest = if size.width() < size.height() {
                        size.height()
                    } else {
                        size.width()
                    };
                    let downScale = maxTextureSize as f32 / largest;
                    size.set(
                        (size.width() * downScale).floor(),
                        (size.height() * downScale).floor(),
                    );
                }
            }
            size
        };
        let tileSize = scaledSize.toCeil();
        if tileSize.isEmpty() {
            return Self {
                success: false,
                tileScale: SkSize {
                    fWidth: 0.0,
                    fHeight: 0.0,
                },
                matrixForDraw: SkMatrix::default(),
                imageInfo: SkImageInfo::MakeEmpty(),
                props: SkSurfaceProps::default(),
            };
        }
        let tileScale = SkSize {
            fWidth: tileSize.width() as f32 / bounds.width(),
            fHeight: tileSize.height() as f32 / bounds.height(),
        };
        let imgCS = ref_or_srgb(dstColorSpace);
        let imgCT =
            if crate::src::core::SkImageInfoPriv::SkColorTypeMaxBitsPerChannel(dstColorType) <= 8 {
                SkColorType::kRGBA_8888_SkColorType
            } else {
                SkColorType::kRGBA_F16Norm_SkColorType
            };
        Self {
            success: true,
            tileScale,
            matrixForDraw: SkMatrix::RectToRectOrIdentity(
                bounds,
                &SkRect::MakeIWH(tileSize.width(), tileSize.height()),
                ScaleToFit::kFill_ScaleToFit,
            ),
            imageInfo: SkImageInfo::MakeDimensionsWithColorSpace(
                tileSize,
                imgCT,
                kPremul_SkAlphaType,
                Some(imgCS),
            ),
            props,
        }
    }
}
// Source: src/shaders/SkPictureShader.h:73-73
// Source: src/shaders/SkPictureShader.cpp:244-255
impl CachedImageInfo {
    /// Safety: Surface is its live complete intrusive owner; drawing and snapshot
    /// callbacks use only original raw base projections, with no Surface borrow.
    pub unsafe fn makeImage(
        &self,
        surf: Option<sk_sp<dyn crate::src::image::SkSurface::SkSurface<'static>>>,
        pict: &dyn SkPicture,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        let Some(surf) = surf else {
            return Ok(None);
        };
        let raw = surf.as_ptr().expect("native live surface");
        let canvas = unsafe { crate::src::image::SkSurface::getCanvas_raw(raw) }
            .expect("native raster canvas");
        // This matrix-only lease ends before drawPicture can trigger Surface COW.
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::concat(
                core::pin::Pin::new_unchecked(&mut *canvas.as_ptr()),
                &self.matrixForDraw,
            );
        }
        unsafe {
            crate::src::core::SkCanvas::SkCanvas::drawPicture_raw(canvas, Some(pict), None, None)
        }?;
        unsafe { crate::src::image::SkSurface::makeTemporaryImage_raw(raw) }
    }
}
// Source: src/shaders/SkPictureShader.h:87-90
// Source: src/shaders/SkPictureShader.cpp:259-289
impl SkPictureShader {
    unsafe fn rasterShader(
        &self,
        totalM: &SkMatrix,
        dstColorType: SkColorType,
        dstColorSpace: Option<&Arc<SkColorSpace>>,
        propsIn: &SkSurfaceProps,
    ) -> Result<Option<Arc<dyn super::SkShader::SkShader>>, crate::RasterError> {
        unsafe {
            let info =
                CachedImageInfo::Make(&self.fTile, totalM, dstColorType, dstColorSpace, 0, propsIn);
            if !info.success {
                return Ok(None);
            }
            let key = ImageFromPictureKey::new(
                info.imageInfo
                    .colorSpace()
                    .expect("native image color space"),
                info.imageInfo.colorType(),
                self.fPicture.uniqueID(),
                &self.fTile,
                info.tileScale,
                &info.props,
            );
            let mut image: Option<Arc<dyn SkImage>> = None;
            if !crate::src::core::SkResourceCache::Find(
                &key.key,
                ImageFromPictureRec::Visitor,
                (&mut image as *mut Option<Arc<dyn SkImage>>).cast(),
            ) {
                image = info.makeImage(
                    crate::src::image::SkSurface_Raster::Raster(
                        &info.imageInfo,
                        0,
                        Some(&info.props),
                    ),
                    &*self.fPicture,
                )?;
                let Some(image_ref) = image.as_ref() else {
                    return Ok(None);
                };
                crate::src::core::SkResourceCache::Add(
                    Box::new(ImageFromPictureRec::new(&key, image_ref.clone())),
                    core::ptr::null_mut(),
                );
                crate::src::core::SkPicturePriv::AddedToCache(&*self.fPicture);
            }
            let lm = SkMatrix::Scale(1.0 / info.tileScale.width(), 1.0 / info.tileScale.height());
            Ok(image.expect("native successful cache visitor").makeShader(
                self.fTmx,
                self.fTmy,
                &crate::include::core::SkSamplingOptions::SkSamplingOptions::with_filter(
                    self.fFilter,
                ),
                Some(&lm),
            ))
        }
    }
    // Source: src/shaders/SkPictureShader.cpp:291-306
    unsafe fn append_stages(
        &self,
        rec: &crate::src::core::SkEffectPriv::SkStageRec<'_>,
        mRec: &super::SkShaderBase::MatrixRec,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            // The actual arena owns this retained shader through pipeline execution.
            let bitmapShader =
                (*rec.fAlloc).make::<Option<Arc<dyn super::SkShader::SkShader>>>(None);
            *bitmapShader = self.rasterShader(
                &mRec.totalMatrix(),
                rec.fDstColorType,
                rec.fDstCS,
                rec.fSurfaceProps,
            )?;
            let Some(shader) = (*bitmapShader).as_ref() else {
                return Ok(false);
            };
            shader.appendStages(rec, mRec)
        }
    }
}

// Source: src/shaders/SkPictureShader.h:77
// Native protected ordinary declaration: SkPictureShader(SkReadBuffer&).
// No definition occurs in src/shaders/SkPictureShader.cpp. Rust has no bodyless ordinary
// inherent method with this adapted object representation; the declaration remains
// pending rather than inventing an extern ABI, pure-virtual trait, or constructor body.

use super::SkImage::{
    CachingHint, ImageCore, ImageVirtuals, RequiredProperties, SkImage, SkImageState,
    SkImages::BitDepth,
};
use super::SkImage_Base::{SkImageBaseState, SkImage_Base, Type};
use super::SkImage_Lazy::{SharedGenerator, SkImage_Lazy, Validator};
use crate::include::core::{
    SkColorType::SkColorType, SkRecorder::SkRecorder, SkRefCnt::sk_sp, SkSize::SkISize,
    SkSurfaceProps::SkSurfaceProps,
};
use crate::src::core::{
    SkBitmap::SkBitmap, SkCanvas::SkCanvas, SkColorSpace::SkColorSpace, SkData::SkData,
    SkImageInfo::SkImageInfo, SkMatrix::SkMatrix, SkPaint::SkPaint, SkPicture::SkPicture,
    SkRect::SkIRect,
};
use std::ptr::NonNull;
use std::sync::Arc;
// Actual native concrete generator, declared by SkPictureImageGenerator.h.
use super::SkPictureImageGenerator::SkPictureImageGenerator;

// Source: src/image/SkImage_Picture.h:30-31
// The native lazy base is an embedded subobject, never a separately owned image.
pub struct SkImage_Picture {
    lazy: SkImage_Lazy,
}
impl SkImage_Picture {
    // Source: src/image/SkImage_Picture.h:32-35
    // Source: src/image/SkImage_Picture.cpp:36-47
    pub fn Make(
        picture: sk_sp<dyn SkPicture>,
        dimensions: &SkISize,
        matrix: Option<&SkMatrix>,
        paint: Option<&SkPaint>,
        bitDepth: BitDepth,
        colorSpace: Option<Arc<SkColorSpace>>,
        props: SkSurfaceProps,
    ) -> Option<Arc<dyn SkImage>> {
        // The actual factory body is translated in SkPictureImageGenerator.cpp.
        let gen = super::SkImageGeneratorPriv::MakeFromPicture(
            dimensions, picture, matrix, paint, bitDepth, colorSpace, props,
        );
        let validator = Validator::new(SharedGenerator::Make(gen), None, None);
        if validator.fSharedGenerator.is_some() {
            Some(Arc::new(Self::new(validator)))
        } else {
            None
        }
    }
    // Source: src/image/SkImage_Picture.h:37-37
    fn new(validator: Validator) -> Self {
        Self {
            lazy: SkImage_Lazy::derived_value(validator),
        }
    }
    // Source: src/image/SkImage_Picture.h:42-42
    // Source: src/image/SkImage_Picture.cpp:49-52
    pub fn props(&self) -> &SkSurfaceProps {
        let shared = self.lazy.generator();
        let generator = shared.scoped();
        // Native Make guarantees this concrete generator. Its Box allocation
        // stays stable and fProps is immutable for the image's entire lifetime.
        let pictureIG = unsafe {
            &*((&*generator) as *const dyn crate::src::core::SkImageGenerator::SkImageGenerator
                as *const SkPictureImageGenerator)
        };
        let props = std::ptr::from_ref(&pictureIG.fProps);
        drop(generator);
        unsafe { &*props }
    }
    // Source: src/image/SkImage_Picture.h:45-45
    // Source: src/image/SkImage_Picture.cpp:54-63
    /// Safety: retain the native Canvas/Surface lease through clear and replay.
    pub unsafe fn replay<'device>(
        &self,
        canvas: NonNull<SkCanvas<'device>>,
    ) -> Result<(), crate::RasterError> {
        let sharedGenerator = self.lazy.generator();
        let generator = sharedGenerator.scoped();
        let pictureIG = &*((&*generator)
            as *const dyn crate::src::core::SkImageGenerator::SkImageGenerator
            as *const SkPictureImageGenerator);
        SkCanvas::clear_raw(canvas, 0)?;
        SkCanvas::drawPicture_sp_raw(
            canvas,
            &pictureIG.fPicture,
            Some(&pictureIG.fMatrix),
            pictureIG.fPaint.as_ref(),
        )
    }
    // Source: src/image/SkImage_Picture.h:47-47
    // Source: src/image/SkImage_Picture.cpp:65-80
    fn onMakeSubset(&self, subset: &SkIRect) -> Option<Arc<dyn SkImage>> {
        let sharedGenerator = self.lazy.generator();
        let generator = sharedGenerator.scoped();
        let pictureIG = unsafe {
            &*((&*generator) as *const dyn crate::src::core::SkImageGenerator::SkImageGenerator
                as *const SkPictureImageGenerator)
        };
        let mut matrix = pictureIG.fMatrix.clone();
        matrix.postTranslate(-subset.left() as f32, -subset.top() as f32);
        let bitDepth = if self.colorType() == SkColorType::kRGBA_F16_SkColorType {
            BitDepth::kF16
        } else {
            BitDepth::kU8
        };
        Self::Make(
            pictureIG.fPicture.clone(),
            &subset.size(),
            Some(&matrix),
            pictureIG.fPaint.as_ref(),
            bitDepth,
            self.refColorSpace(),
            pictureIG.fProps,
        )
    }
    // Source: src/image/SkImage_Picture.h:51-51
    // Source: src/image/SkImage_Picture.cpp:82-132
    pub fn getImageKeyValues(&self, keyValues: &mut [u32; 6]) -> bool {
        let sharedGenerator = self.lazy.generator();
        let generator = sharedGenerator.scoped();
        let pictureIG = unsafe {
            &*((&*generator) as *const dyn crate::src::core::SkImageGenerator::SkImageGenerator
                as *const SkPictureImageGenerator)
        };
        if pictureIG.fPaint.is_some() {
            return false;
        }
        let ii = sharedGenerator.getInfo();
        if !ii
            .colorSpace()
            .expect("native picture generator has color space")
            .isSRGB()
        {
            return false;
        }
        let m = &pictureIG.fMatrix;
        if !m.isIdentity() && !m.isTranslate() {
            return false;
        }
        let isU8 = ii.colorType() != SkColorType::kRGBA_F16_SkColorType;
        // Same immutable fields as props(); using the already held generator
        // borrow avoids reacquiring the very same nonrecursive native mutex.
        let pixelGeometry = pictureIG.fProps.pixelGeometry() as u32;
        let surfacePropFlags = pictureIG.fProps.flags();
        let width = ii.width();
        let height = ii.height();
        let transX = m.getTranslateX();
        let transY = m.getTranslateY();
        crate::SkASSERT!(@source b"src/image/SkImage_Picture.cpp\0".as_ptr(),114; @condition b"pixelGeometry <= 4\0".as_ptr(); pixelGeometry<=4);
        crate::SkASSERT!(@source b"src/image/SkImage_Picture.cpp\0".as_ptr(),115; @condition b"surfacePropFlags < 8\0".as_ptr(); surfacePropFlags<8);
        crate::SkASSERT!(@source b"src/image/SkImage_Picture.cpp\0".as_ptr(),116; @condition b"SkTFitsIn<uint32_t>(width)\0".as_ptr(); width>=0);
        crate::SkASSERT!(@source b"src/image/SkImage_Picture.cpp\0".as_ptr(),117; @condition b"SkTFitsIn<uint32_t>(height)\0".as_ptr(); height>=0);
        crate::SkASSERT!(@source b"src/image/SkImage_Picture.cpp\0".as_ptr(),118; @condition b"sizeof(float) == sizeof(uint32_t)\0".as_ptr(); std::mem::size_of::<f32>()==std::mem::size_of::<u32>());
        keyValues[0] = (if isU8 { 1 } else { 0 }) | (pixelGeometry << 1) | (surfacePropFlags << 4);
        keyValues[1] = pictureIG.fPicture.uniqueID();
        crate::SkASSERT!(@source b"src/image/SkImage_Picture.cpp\0".as_ptr(),126; @condition b"keyValues[1] != 0\0".as_ptr(); keyValues[1]!=0);
        keyValues[2] = width as u32;
        keyValues[3] = height as u32;
        keyValues[4] = transX.to_bits();
        keyValues[5] = transY.to_bits();
        true
    }
}
// Typed inheritance forwarding shares the one embedded native lazy state.
// These adapters add no duplicate implementation credit for lazy algorithms.
impl ImageCore for SkImage_Picture {
    fn state(&self) -> &SkImageState {
        self.lazy.state()
    }
    fn asImageBase(&self) -> &dyn SkImage_Base {
        self
    }
    fn isTextureBacked(&self) -> bool {
        self.isTextureBackedCPU()
    }
    fn textureSize(&self) -> usize {
        self.lazy.textureSize()
    }
    fn isValid(&self, recorder: Option<&dyn SkRecorder>) -> bool {
        self.lazy.isValid(recorder)
    }
}
impl ImageVirtuals for SkImage_Picture {
    fn makeSubset(
        self: Arc<Self>,
        recorder: Option<&dyn SkRecorder>,
        subset: &SkIRect,
        props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        super::SkImage_Base::make_subset(&*self, recorder, subset, props)
    }
    fn isLazyGenerated(&self) -> bool {
        self.isLazyGeneratedCPU()
    }
    fn makeColorSpace(
        self: Arc<Self>,
        recorder: Option<&dyn SkRecorder>,
        target: Option<Arc<SkColorSpace>>,
        props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        super::SkImage_Base::make_color_space(self, recorder, target, props)
    }
    fn makeColorTypeAndColorSpace(
        self: Arc<Self>,
        _recorder: Option<&dyn SkRecorder>,
        target: SkColorType,
        cs: Option<Arc<SkColorSpace>>,
        _props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        Ok(self.lazy.converted(target, cs))
    }
}
impl SkImage_Base for SkImage_Picture {
    fn base_state(&self) -> &SkImageBaseState {
        self.lazy.base_state()
    }
    unsafe fn onReadPixelsCPU(
        &self,
        info: &SkImageInfo,
        pixels: *mut u8,
        rb: usize,
        x: i32,
        y: i32,
        hint: CachingHint,
    ) -> Result<bool, crate::RasterError> {
        self.lazy.onReadPixelsCPU(info, pixels, rb, x, y, hint)
    }
    fn onHasMipmaps(&self) -> bool {
        self.lazy.onHasMipmaps()
    }
    fn onIsProtected(&self) -> bool {
        self.lazy.onIsProtected()
    }
    fn getROPixelsCPU(&self, dst: &mut SkBitmap, hint: CachingHint) -> bool {
        self.lazy.getROPixelsCPU(dst, hint)
    }
    fn onMakeSubsetCPU(
        &self,
        _recorder: Option<&dyn SkRecorder>,
        subset: &SkIRect,
        _props: RequiredProperties,
    ) -> Result<Option<Arc<dyn SkImage>>, crate::RasterError> {
        Ok(self.onMakeSubset(subset))
    }
    fn onRefEncoded(&self) -> Option<Arc<SkData>> {
        self.lazy.onRefEncoded()
    }
    unsafe fn onMakeSurfaceCPU<'r>(
        &self,
        recorder: Option<&'r dyn SkRecorder>,
        info: &SkImageInfo,
    ) -> Option<sk_sp<dyn super::SkSurface::SkSurface<'r> + 'r>> {
        self.lazy.onMakeSurfaceCPU(recorder, info)
    }
    // Source: src/image/SkImage_Picture.h:39-39
    fn type_(&self) -> Type {
        Type::kLazyPicture
    }
    unsafe fn onReinterpretColorSpaceCPU(&self, cs: Arc<SkColorSpace>) -> Option<Arc<dyn SkImage>> {
        self.lazy.onReinterpretColorSpaceCPU(cs)
    }
}

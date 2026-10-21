// Copyright 2006 The Android Open Source Project. BSD-style license. Unverified translation.
use crate::src::core::{
    SkFlattenable::{self as flat, FactoryResult, FlattenableOwner, SkFlattenableView},
    SkImageFilter::SkImageFilter,
    SkMask::{AllocType, Format, SkMask, SkMaskBuilder},
    SkMaskFilterBase::{SkMaskFilterBase, Type},
    SkMatrix::SkMatrix,
    SkPaint::SkPaint,
    SkPoint::{SkIPoint, SkPoint},
    SkPoint3::SkPoint3,
};
use core::mem::MaybeUninit;
use std::sync::Arc;
// Source: src/effects/SkEmbossMaskFilter.h:36-42
// Native explicit pad remains allowed to be uninitialized until serialization clears it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Light {
    pub fDirection: [f32; 3],
    pub fPad: MaybeUninit<u16>,
    pub fAmbient: u8,
    pub fSpecular: u8,
}
// Source: src/effects/SkEmbossMaskFilter.h:83-84
pub struct SkEmbossMaskFilter {
    fLight: Light,
    fBlurSigma: f32,
}
// Source: src/effects/SkEmbossMaskFilter.cpp:33-48
pub fn Make(blurSigma: f32, light: &Light) -> Option<Arc<dyn SkMaskFilterBase>> {
    if !blurSigma.is_finite() || blurSigma <= 0.0 {
        return None;
    }
    let mut dir = SkPoint3::Make(
        light.fDirection[0],
        light.fDirection[1],
        light.fDirection[2],
    );
    if !dir.normalize() {
        return None;
    }
    let mut light = *light;
    light.fDirection = [dir.x(), dir.y(), dir.z()];
    Some(Arc::new(SkEmbossMaskFilter::new(blurSigma, light)))
}
impl SkEmbossMaskFilter {
    // Source: src/effects/SkEmbossMaskFilter.cpp:76-82
    fn new(blurSigma: f32, light: Light) -> Self {
        crate::SkASSERT!(@source b"src/effects/SkEmbossMaskFilter.cpp\0".as_ptr(),75; @condition b"fBlurSigma > 0\0".as_ptr(); blurSigma>0.0);
        crate::SkASSERT!(@source b"src/effects/SkEmbossMaskFilter.cpp\0".as_ptr(),76; @condition b"SkIsFinite(fLight.fDirection, 3)\0".as_ptr(); light.fDirection.iter().all(|x|x.is_finite()));
        Self {
            fLight: light,
            fBlurSigma: blurSigma,
        }
    }
    // Source: src/effects/SkEmbossMaskFilter.cpp:158-168
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> FactoryResult {
        let mut light = MaybeUninit::<Light>::uninit();
        if unsafe {
            buffer.readByteArray_raw(light.as_mut_ptr().cast(), core::mem::size_of::<Light>())
        } {
            let mut light = unsafe { light.assume_init() };
            light.fPad = MaybeUninit::new(0);
            let sigma = buffer.readScalar();
            return Ok(Make(sigma, &light).map(FlattenableOwner::MaskFilter));
        }
        Ok(None)
    }
    // Source: src/effects/SkEmbossMaskFilter.h:77-81
    fn LegacySpecular(
        direction: SkPoint3,
        color: u32,
        scale: f32,
        ks: f32,
        shininess: f32,
        input: Option<Arc<dyn SkImageFilter>>,
    ) -> Option<Arc<dyn SkImageFilter>> {
        crate::src::effects::imagefilters::SkLightingImageFilter::LegacySpecular(
            direction, color, scale, ks, shininess, input,
        )
    }
}
impl SkFlattenableView for SkEmbossMaskFilter {
    fn getFactory(&self) -> Option<flat::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkEmbossMaskFilter")
    }
    fn getFlattenableType(&self) -> flat::Type {
        flat::Type::kSkMaskFilter_Type
    }
    // Source: src/effects/SkEmbossMaskFilter.cpp:170-175
    fn flatten(
        &self,
        w: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        let mut light = self.fLight;
        light.fPad = MaybeUninit::new(0);
        unsafe {
            w.writeByteArray(
                core::ptr::addr_of!(light).cast(),
                core::mem::size_of::<Light>(),
            );
        }
        w.writeScalar(self.fBlurSigma);
        Ok(())
    }
}
impl SkMaskFilterBase for SkEmbossMaskFilter {
    // Source: src/effects/SkEmbossMaskFilter.cpp:84-86
    fn getFormat(&self) -> Format {
        Format::k3D_Format
    }
    fn r#type(&self) -> Type {
        Type::kEmboss
    }
    // Source: src/effects/SkEmbossMaskFilter.cpp:88-156
    unsafe fn filterMask(
        &self,
        dst: &mut SkMaskBuilder,
        src: &SkMask,
        matrix: &SkMatrix,
        margin: Option<&mut SkIPoint>,
    ) -> Result<bool, crate::RasterError> {
        if src.fFormat != Format::kA8_Format {
            return Ok(false);
        }
        let sigma = matrix.mapRadius(self.fBlurSigma);
        if !crate::src::core::SkBlurMask::BoxBlur(
            dst,
            src,
            sigma,
            crate::include::core::SkBlurTypes::SkBlurStyle::kInner_SkBlurStyle,
            None,
        ) {
            return Ok(false);
        }
        *dst.format() = Format::k3D_Format;
        if let Some(m) = margin {
            let d = crate::src::core::SkFloatingPoint::sk_float_ceil2int(3.0 * sigma);
            m.set(d, d);
        }
        if src.fImage.is_null() {
            return Ok(true);
        }
        let alpha = dst.fImage;
        let total = dst.computeTotalImageSize();
        if total == 0 {
            return Ok(false);
        }
        let plane = dst.computeImageSize();
        crate::SkASSERT!(@source b"src/effects/SkEmbossMaskFilter.cpp\0".as_ptr(),113; @condition b"planeSize != 0\0".as_ptr(); plane!=0);
        *dst.image() = SkMaskBuilder::AllocImage(total, AllocType::kUninit_Alloc);
        core::ptr::copy_nonoverlapping(alpha, dst.fImage.cast_mut(), plane);
        SkMaskBuilder::FreeImage(alpha.cast_mut().cast());
        let mut light = self.fLight;
        let mapped = matrix.mapVectorXY(self.fLight.fDirection[0], self.fLight.fDirection[1]);
        let mut vec = mapped;
        vec.set_length_from(
            mapped.fX,
            mapped.fY,
            SkPoint::length_xy(self.fLight.fDirection[0], self.fLight.fDirection[1]),
        );
        light.fDirection[0] = vec.fX;
        light.fDirection[1] = vec.fY;
        // Actual native kernel dependency is retained, never replaced by a no-op.
        crate::src::effects::SkEmbossMask::Emboss(dst, &light);
        core::ptr::copy_nonoverlapping(src.fImage, dst.fImage.cast_mut(), src.computeImageSize());
        Ok(true)
    }
    // Source: src/effects/SkEmbossMaskFilter.cpp:180-263
    fn asImageFilter(
        &self,
        _ctm: &SkMatrix,
        paint: &SkPaint,
    ) -> (Option<Arc<dyn SkImageFilter>>, bool) {
        use crate::src::core::{SkBlendMode::SkBlendMode as BM, SkColor::SkColor4f};
        use crate::src::effects::imagefilters::{
            SkBlendImageFilter as blend, SkBlurImageFilter as blur,
            SkLightingImageFilter as lighting, SkShaderImageFilter as shader,
        };
        let coverage = blur::Blur(
            self.fBlurSigma,
            self.fBlurSigma,
            crate::include::core::SkTileMode::SkTileMode::kDecal,
            None,
            None,
        );
        let mut src = crate::src::shaders::SkColorShader::Color4f(&paint.getColor4f(), None);
        if paint.getShader().is_some() {
            src = crate::src::shaders::SkBlendShader::Blend(BM::kDstIn, paint.refShader(), src);
        }
        src = src
            .expect("native nonnull source shader")
            .makeWithColorFilter(paint.refColorFilter());
        let srcColor = shader::Shader(src, paint.isDither(), None);
        let ambientf = self.fLight.fAmbient as f32 / 255.0;
        let ambientColor = SkColor4f {
            fR: ambientf,
            fG: ambientf,
            fB: ambientf,
            fA: 1.0,
        };
        let ambient = shader::Shader(
            crate::src::shaders::SkColorShader::Color4f(&ambientColor, None),
            false,
            None,
        );
        let direction = SkPoint3::Make(
            self.fLight.fDirection[0],
            self.fLight.fDirection[1],
            self.fLight.fDirection[2],
        );
        let scale = -255.0 / 32.0;
        let diffuse =
            lighting::DistantLitDiffuse(direction, 0xffffffff, scale, 1.0, coverage.clone(), None);
        let ad = blend::Blend(BM::kPlus, diffuse, ambient, None);
        let adColor = blend::Blend(BM::kModulate, srcColor, ad, None);
        let shininess = ((self.fLight.fSpecular >> 4) + 1) as f32;
        let specular = Self::LegacySpecular(direction, 0xffffffff, scale, 1.0, shininess, coverage);
        let finalFilter = blend::Blend(BM::kPlus, adColor, specular, None);
        (blend::Blend(BM::kDstIn, finalFilter, None, None), true)
    }
}
// SK_FLATTENABLE_HOOKS canonical name (global initialization owns whether/when registered).
pub fn RegisterFlattenables() {
    unsafe {
        flat::Register(b"SkEmbossMaskFilter", SkEmbossMaskFilter::CreateProc);
    }
}
// Source: src/effects/SkEmbossMaskFilter.cpp:50-72
#[cfg(SK_SUPPORT_LEGACY_EMBOSSMASKFILTER)]
pub unsafe fn MakeEmboss(
    blurSigma: f32,
    direction: *const f32,
    ambient: f32,
    specular: f32,
) -> Option<Arc<dyn SkMaskFilterBase>> {
    if direction.is_null() {
        return None;
    }
    let mut light = MaybeUninit::<Light>::uninit();
    let p = light.as_mut_ptr();
    core::ptr::copy_nonoverlapping(
        direction,
        core::ptr::addr_of_mut!((*p).fDirection).cast(),
        3,
    );
    core::ptr::addr_of_mut!((*p).fAmbient)
        .write(crate::src::core::SkColor::SkUnitScalarClampToByte(ambient));
    let pinned = if specular < 0.0 {
        0.0
    } else if 16.0 < specular {
        16.0
    } else {
        specular
    };
    core::ptr::addr_of_mut!((*p).fSpecular)
        .write((pinned * (255.0 / 16.0) + 0.5).to_int_unchecked::<u32>() as u8);
    core::ptr::addr_of_mut!((*p).fPad).write(MaybeUninit::uninit());
    Make(blurSigma, &light.assume_init())
}

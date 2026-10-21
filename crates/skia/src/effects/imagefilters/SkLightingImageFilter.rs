// Copyright 2012 The Android Open Source Project. BSD-style license.
use crate::include::core::{SkSize::SkISize, SkTileMode::SkTileMode};
use crate::src::core::{
    SkColor::{SkColor, SkColorGetB, SkColorGetG, SkColorGetR, SkColorSetARGB},
    SkFlattenable::{Factory, FactoryResult, FlattenableOwner, SkFlattenableView, Type},
    SkImageFilter::SkImageFilter,
    SkImageFilterTypes::{
        Builder, Context, FilterResult, LayerSpace, Mapping, ParameterSpace, ShaderFlags, Vector,
    },
    SkImageFilter_Base::{SkImageFilterBaseHooks, SkImageFilter_Base},
    SkKnownRuntimeEffects::{GetKnownRuntimeEffect, StableKey},
    SkM44::{SkV3, SkV4},
    SkPoint::SkPoint,
    SkPoint3::SkPoint3,
    SkReadBuffer::SkReadBuffer,
    SkRect::{SkIRect, SkRect},
    SkRuntimeEffect::{ChildPtr, SkRuntimeEffectBuilder},
    SkWriteBuffer::SkWriteBuffer,
};
use crate::src::shaders::SkShader::SkShader;
use std::sync::Arc;
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:45-51
#[derive(Clone, Copy, Default)]
struct ZValue {
    fZ: f32,
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:55-72
// The native LayerSpace<ZValue> specialization is local and owns exactly ZValue.
// It is separate from the core module's sealed geometry specializations.
#[derive(Clone, Copy, Default)]
struct LayerZ {
    fData: ZValue,
}
impl LayerZ {
    fn val(&self) -> f32 {
        self.fData.fZ
    }
    fn Map(mapping: &Mapping, z: ParameterSpace<ZValue>) -> Self {
        let z = z.geometry().fZ;
        let vector = mapping.paramToLayer(&ParameterSpace::new(Vector::new(z, z)));
        Self {
            fData: ZValue {
                fZ: crate::src::core::SkFloatingPoint::sk_float_midpoint(vector.x(), vector.y()),
            },
        }
    }
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:77-134
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum LightType {
    kDistant,
    kPoint,
    kSpot,
}
impl LightType {
    const kLast: Self = Self::kSpot;
}
#[derive(Clone, Copy)]
struct Light {
    fType: LightType,
    fLightColor: SkColor,
    fLocationXY: ParameterSpace<SkPoint>,
    fLocationZ: ParameterSpace<ZValue>,
    fDirectionXY: ParameterSpace<Vector>,
    fDirectionZ: ParameterSpace<ZValue>,
    fFalloffExponent: f32,
    fCosCutoffAngle: f32,
}
impl Light {
    fn Point(color: SkColor, location: SkPoint3) -> Self {
        Self {
            fType: LightType::kPoint,
            fLightColor: color,
            fLocationXY: ParameterSpace::new(SkPoint::make(location.fX, location.fY)),
            fLocationZ: ParameterSpace::new(ZValue { fZ: location.fZ }),
            fDirectionXY: ParameterSpace::new(Vector::new(0.0, 0.0)),
            fDirectionZ: ParameterSpace::new(ZValue { fZ: 0.0 }),
            fFalloffExponent: 0.0,
            fCosCutoffAngle: 0.0,
        }
    }
    fn Distant(color: SkColor, direction: SkPoint3) -> Self {
        Self {
            fType: LightType::kDistant,
            fLightColor: color,
            fLocationXY: ParameterSpace::new(SkPoint::make(0.0, 0.0)),
            fLocationZ: ParameterSpace::new(ZValue { fZ: 0.0 }),
            fDirectionXY: ParameterSpace::new(Vector::new(direction.fX, direction.fY)),
            fDirectionZ: ParameterSpace::new(ZValue { fZ: direction.fZ }),
            fFalloffExponent: 0.0,
            fCosCutoffAngle: 0.0,
        }
    }
    fn Spot(
        color: SkColor,
        location: SkPoint3,
        direction: SkPoint3,
        falloff: f32,
        cutoff: f32,
    ) -> Self {
        Self {
            fType: LightType::kSpot,
            fLightColor: color,
            fLocationXY: ParameterSpace::new(SkPoint::make(location.fX, location.fY)),
            fLocationZ: ParameterSpace::new(ZValue { fZ: location.fZ }),
            fDirectionXY: ParameterSpace::new(Vector::new(direction.fX, direction.fY)),
            fDirectionZ: ParameterSpace::new(ZValue { fZ: direction.fZ }),
            fFalloffExponent: falloff,
            fCosCutoffAngle: cutoff,
        }
    }
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:135-164
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum MaterialType {
    kDiffuse,
    kSpecular,
    kEmbossSpecular,
}
impl MaterialType {
    const kLast: Self = Self::kEmbossSpecular;
}
#[derive(Clone, Copy)]
struct Material {
    fType: MaterialType,
    fSurfaceDepth: ParameterSpace<ZValue>,
    fK: f32,
    fShininess: f32,
}
impl Material {
    fn Diffuse(k: f32, depth: f32) -> Self {
        Self {
            fType: MaterialType::kDiffuse,
            fSurfaceDepth: ParameterSpace::new(ZValue { fZ: depth }),
            fK: k,
            fShininess: 0.0,
        }
    }
    fn Specular(k: f32, shine: f32, depth: f32) -> Self {
        Self {
            fType: MaterialType::kSpecular,
            fSurfaceDepth: ParameterSpace::new(ZValue { fZ: depth }),
            fK: k,
            fShininess: shine,
        }
    }
    fn EmbossSpecular(k: f32, shine: f32, depth: f32) -> Self {
        Self {
            fType: MaterialType::kEmbossSpecular,
            fSurfaceDepth: ParameterSpace::new(ZValue { fZ: depth }),
            fK: k,
            fShininess: shine,
        }
    }
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:165-207
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:165-165
pub struct SkLightingImageFilter {
    fLight: Light,
    fMaterial: Material,
    base: SkImageFilter_Base,
}
impl SkLightingImageFilter {
    fn new(light: Light, material: Material, input: Option<Arc<dyn SkImageFilter>>) -> Self {
        let base = SkImageFilter_Base::new(&[input], None);
        Self {
            fLight: light,
            fMaterial: material,
            base,
        }
    }
    fn requiredInput(&self, desired: &LayerSpace<SkIRect>) -> LayerSpace<SkIRect> {
        let mut required = *desired;
        required.outset(LayerSpace::new(SkISize::Make(1, 1)));
        required
    }
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:211-223
unsafe fn make_normal_shader(
    alpha: Option<Arc<dyn SkShader>>,
    edge: &LayerSpace<SkIRect>,
    depth: LayerZ,
) -> Option<Arc<dyn SkShader>> {
    let effect = GetKnownRuntimeEffect(StableKey::kNormal)
        .expect("native known normal effect")
        .clone();
    let mut builder = SkRuntimeEffectBuilder::new(effect);
    builder
        .child(b"alphaMap")
        .assign(alpha.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
    builder
        .uniform(b"edgeBounds")
        .assign(&SkRect::Make(edge.geometry()).makeInset(0.5, 0.5));
    builder.uniform(b"negSurfaceDepth").assign(&(-depth.val()));
    builder.makeShader(None)
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:225-283
unsafe fn make_lighting_shader(
    normal: Option<Arc<dyn SkShader>>,
    lightType: LightType,
    color: SkColor,
    location: LayerSpace<SkPoint>,
    locationZ: LayerZ,
    direction: LayerSpace<Vector>,
    directionZ: LayerZ,
    falloff: f32,
    cutoff: f32,
    material: MaterialType,
    depth: LayerZ,
    k: f32,
    shininess: f32,
) -> Option<Arc<dyn SkShader>> {
    let effect = GetKnownRuntimeEffect(StableKey::kLighting)
        .expect("native known lighting effect")
        .clone();
    let mut builder = SkRuntimeEffectBuilder::new(effect);
    builder
        .child(b"normalMap")
        .assign(normal.map(ChildPtr::Shader).unwrap_or(ChildPtr::Empty));
    builder.uniform(b"materialAndLightType").assign(&SkV4 {
        x: depth.val(),
        y: shininess,
        z: material as i32 as f32,
        w: if lightType == LightType::kPoint {
            0.0
        } else if lightType == LightType::kDistant {
            -1.0
        } else {
            1.0
        },
    });
    builder.uniform(b"lightPosAndSpotFalloff").assign(&SkV4 {
        x: location.x(),
        y: location.y(),
        z: locationZ.val(),
        w: falloff,
    });
    let dir = SkV3 {
        x: direction.x(),
        y: direction.y(),
        z: directionZ.val(),
    };
    let len = dir.length();
    let inv = if len != 0.0 { 1.0 / len } else { 0.0 };
    builder.uniform(b"lightDirAndSpotCutoff").assign(&SkV4 {
        x: inv * dir.x,
        y: inv * dir.y,
        z: inv * dir.z,
        w: cutoff,
    });
    let scale = k / 255.0;
    builder.uniform(b"lightColor").assign(&SkV3 {
        x: SkColorGetR(color) as f32 * scale,
        y: SkColorGetG(color) as f32 * scale,
        z: SkColorGetB(color) as f32 * scale,
    });
    builder.makeShader(None)
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:285-316
fn make_lighting(
    light: Light,
    material: Material,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    use crate::src::core::SkFloatingPoint::SkIsFinite;
    if !SkIsFinite(&[
        material.fK,
        material.fShininess,
        material.fSurfaceDepth.geometry().fZ,
    ]) || material.fK < 0.0
    {
        return None;
    }
    if !light.fLocationXY.geometry().isFinite()
        || !light.fDirectionXY.geometry().isFinite()
        || !SkIsFinite(&[
            light.fFalloffExponent,
            light.fCosCutoffAngle,
            light.fLocationZ.geometry().fZ,
            light.fDirectionZ.geometry().fZ,
        ])
        || light.fCosCutoffAngle < -1.0
        || light.fCosCutoffAngle > 1.0
    {
        return None;
    }
    let mut filter = input;
    if let Some(crop) = crop {
        filter = super::SkCropImageFilter::Crop(crop, SkTileMode::kDecal, filter);
    }
    filter = Some(Arc::new(SkLightingImageFilter::new(
        light, material, filter,
    )));
    if let Some(crop) = crop {
        filter = super::SkCropImageFilter::Crop(crop, SkTileMode::kDecal, filter);
    }
    filter
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:320-326
pub fn DistantLitDiffuse(
    direction: SkPoint3,
    color: SkColor,
    scale: f32,
    kd: f32,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_lighting(
        Light::Distant(color, direction),
        Material::Diffuse(kd, scale),
        input,
        crop,
    )
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:328-334
pub fn PointLitDiffuse(
    location: SkPoint3,
    color: SkColor,
    scale: f32,
    kd: f32,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_lighting(
        Light::Point(color, location),
        Material::Diffuse(kd, scale),
        input,
        crop,
    )
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:336-345
pub fn SpotLitDiffuse(
    location: SkPoint3,
    target: SkPoint3,
    falloff: f32,
    cutoff: f32,
    color: SkColor,
    scale: f32,
    kd: f32,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    let dir = target - location;
    let cos = crate::src::core::SkScalar::SkDegreesToRadians(cutoff).cos();
    make_lighting(
        Light::Spot(color, location, dir, falloff, cos),
        Material::Diffuse(kd, scale),
        input,
        crop,
    )
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:348-354
// Native private SkEmbossMaskFilter factory body; owner declaration is outside
// this file, so expose the exact body here for that owner's future delegation.
pub(crate) fn LegacySpecular(
    direction: SkPoint3,
    color: SkColor,
    scale: f32,
    ks: f32,
    shininess: f32,
    input: Option<Arc<dyn SkImageFilter>>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_lighting(
        Light::Distant(color, direction),
        Material::EmbossSpecular(ks, shininess, scale),
        input,
        None,
    )
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:356-362
pub fn DistantLitSpecular(
    direction: SkPoint3,
    color: SkColor,
    scale: f32,
    ks: f32,
    shininess: f32,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_lighting(
        Light::Distant(color, direction),
        Material::Specular(ks, shininess, scale),
        input,
        crop,
    )
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:364-370
pub fn PointLitSpecular(
    location: SkPoint3,
    color: SkColor,
    scale: f32,
    ks: f32,
    shininess: f32,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    make_lighting(
        Light::Point(color, location),
        Material::Specular(ks, shininess, scale),
        input,
        crop,
    )
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:372-381
pub fn SpotLitSpecular(
    location: SkPoint3,
    target: SkPoint3,
    falloff: f32,
    cutoff: f32,
    color: SkColor,
    scale: f32,
    ks: f32,
    shininess: f32,
    input: Option<Arc<dyn SkImageFilter>>,
    crop: Option<SkRect>,
) -> Option<Arc<dyn SkImageFilter>> {
    let dir = target - location;
    let cos = crate::src::core::SkScalar::SkDegreesToRadians(cutoff).cos();
    make_lighting(
        Light::Spot(color, location, dir, falloff, cos),
        Material::Specular(ks, shininess, scale),
        input,
        crop,
    )
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:383-390
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterLightingImageFilterFlattenables() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkLightingImageFilter",
            SkLightingImageFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkDiffuseLightingImageFilter",
            SkLightingImageFilter::LegacyDiffuseCreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkSpecularLightingImageFilter",
            SkLightingImageFilter::LegacySpecularCreateProc,
        );
    }
}
impl SkLightingImageFilter {
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:394-424
    pub fn CreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let ty = buffer.read32LE(LightType::kSpot as u32);
        let color = buffer.readColor();
        let mut pos = SkPoint3::default();
        let mut dir = SkPoint3::default();
        buffer.readPoint3(&mut pos);
        buffer.readPoint3(&mut dir);
        let light = Light {
            fType: unsafe { core::mem::transmute::<u32, LightType>(ty) },
            fLightColor: color,
            fLocationXY: ParameterSpace::new(SkPoint::make(pos.fX, pos.fY)),
            fLocationZ: ParameterSpace::new(ZValue { fZ: pos.fZ }),
            fDirectionXY: ParameterSpace::new(Vector::new(dir.fX, dir.fY)),
            fDirectionZ: ParameterSpace::new(ZValue { fZ: dir.fZ }),
            fFalloffExponent: buffer.readScalar(),
            fCosCutoffAngle: buffer.readScalar(),
        };
        let material = Material {
            fType: unsafe {
                core::mem::transmute::<u32, MaterialType>(
                    buffer.read32LE(MaterialType::kEmbossSpecular as u32),
                )
            },
            fSurfaceDepth: ParameterSpace::new(ZValue {
                fZ: buffer.readScalar(),
            }),
            fK: buffer.readScalar(),
            fShininess: buffer.readScalar(),
        };
        if !buffer.isValid() {
            return Ok(None);
        }
        Ok(
            make_lighting(light, material, common.getInput(0), common.cropRect())
                .map(FlattenableOwner::ImageFilter),
        )
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:426-463
    fn LegacyDeserializeLight(buffer: &mut SkReadBuffer<'_>) -> Light {
        let ty = buffer.read32LE(LightType::kSpot as u32);
        if !buffer.isValid() {
            return Light::Distant(0, SkPoint3::default());
        }
        let color = SkColorSetARGB(
            255,
            buffer.readScalar() as u32,
            buffer.readScalar() as u32,
            buffer.readScalar() as u32,
        );
        match ty {
            0 => {
                let direction = SkPoint3::Make(
                    buffer.readScalar(),
                    buffer.readScalar(),
                    buffer.readScalar(),
                );
                Light::Distant(color, direction)
            }
            1 => {
                let location = SkPoint3::Make(
                    buffer.readScalar(),
                    buffer.readScalar(),
                    buffer.readScalar(),
                );
                Light::Point(color, location)
            }
            2 => {
                let location = SkPoint3::Make(
                    buffer.readScalar(),
                    buffer.readScalar(),
                    buffer.readScalar(),
                );
                let target = SkPoint3::Make(
                    buffer.readScalar(),
                    buffer.readScalar(),
                    buffer.readScalar(),
                );
                let falloff = buffer.readScalar();
                let cutoff = buffer.readScalar();
                for _ in 0..5 {
                    buffer.readScalar();
                }
                Light::Spot(color, location, target - location, falloff, cutoff)
            }
            _ => unreachable!("native validated light type"),
        }
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:465-477
    fn LegacyDiffuseCreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let light = Self::LegacyDeserializeLight(buffer);
        let scale = buffer.readScalar();
        let kd = buffer.readScalar();
        Ok(make_lighting(
            light,
            Material::Diffuse(kd, scale),
            common.getInput(0),
            common.cropRect(),
        )
        .map(FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:479-492
    fn LegacySpecularCreateProc(buffer: &mut SkReadBuffer<'_>) -> FactoryResult {
        let mut common = crate::src::core::SkImageFilter_Base::Common::default();
        if !common.unflatten(buffer, 1)? {
            return Ok(None);
        }
        let light = Self::LegacyDeserializeLight(buffer);
        let scale = buffer.readScalar();
        let ks = buffer.readScalar();
        let shine = buffer.readScalar();
        Ok(make_lighting(
            light,
            Material::Specular(ks, shine, scale),
            common.getInput(0),
            common.cropRect(),
        )
        .map(FlattenableOwner::ImageFilter))
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:494-516
    fn flatten_impl(&self, b: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        self.base.flatten(b)?;
        b.writeInt(self.fLight.fType as i32);
        b.writeColor(self.fLight.fLightColor);
        b.writePoint(self.fLight.fLocationXY.geometry());
        b.writeScalar(self.fLight.fLocationZ.geometry().fZ);
        let xy = self.fLight.fDirectionXY.geometry();
        b.writePoint(&SkPoint::make(xy.fX, xy.fY));
        b.writeScalar(self.fLight.fDirectionZ.geometry().fZ);
        b.writeScalar(self.fLight.fFalloffExponent);
        b.writeScalar(self.fLight.fCosCutoffAngle);
        b.writeInt(self.fMaterial.fType as i32);
        b.writeScalar(self.fMaterial.fSurfaceDepth.geometry().fZ);
        b.writeScalar(self.fMaterial.fK);
        b.writeScalar(self.fMaterial.fShininess);
        Ok(())
    }
}
// Source: src/effects/imagefilters/SkLightingImageFilter.cpp:179-179
impl SkFlattenableView for SkLightingImageFilter {
    fn getFactory(&self) -> Option<Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkLightingImageFilter")
    }
    fn getFlattenableType(&self) -> Type {
        Type::kSkImageFilter_Type
    }
    fn flatten(&self, b: &mut dyn SkWriteBuffer) -> Result<(), crate::RasterError> {
        self.flatten_impl(b)?;
        Ok(())
    }
}
impl SkImageFilterBaseHooks for SkLightingImageFilter {
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:184-184
    fn onAffectsTransparentBlack(&self) -> Result<bool, crate::RasterError> {
        Ok(true)
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:520-595
    unsafe fn onFilterImage(&self, ctx: &Context<'_>) -> Result<FilterResult, crate::RasterError> {
        let depth = LayerZ::Map(ctx.mapping(), self.fMaterial.fSurfaceDepth);
        let location = ctx.mapping().paramToLayer(&self.fLight.fLocationXY);
        let locationZ = LayerZ::Map(ctx.mapping(), self.fLight.fLocationZ);
        let dir = ctx.mapping().paramToLayer(&self.fLight.fDirectionXY);
        let dirZ = LayerZ::Map(ctx.mapping(), self.fLight.fDirectionZ);
        let required = self.requiredInput(ctx.desiredOutput());
        let child = self.getChildOutput(0, &ctx.withNewDesiredOutput(required))?;
        let mut clamp = required;
        if !child.layerBounds().contains(&required) {
            let edge = |actual: i32, requested: i32, output: i32| {
                if actual == output {
                    output
                } else {
                    requested
                }
            };
            let input = child.layerBounds();
            let output = ctx.desiredOutput();
            clamp = LayerSpace::new(SkIRect::make_ltrb(
                edge(input.left(), required.left(), output.left()),
                edge(input.top(), required.top(), output.top()),
                edge(input.right(), required.right(), output.right()),
                edge(input.bottom(), required.bottom(), output.bottom()),
            ));
        }
        let mut builder = Builder::new(ctx);
        builder.add_sampled(
            child,
            Some(clamp),
            ShaderFlags::kSampledRepeatedly,
            FilterResult::kDefaultSampling,
        );
        builder.eval(
            |inputs| {
                let normal = make_normal_shader(inputs[0].clone(), &clamp, depth);
                Ok(make_lighting_shader(
                    normal,
                    self.fLight.fType,
                    self.fLight.fLightColor,
                    location,
                    locationZ,
                    dir,
                    dirZ,
                    self.fLight.fFalloffExponent,
                    self.fLight.fCosCutoffAngle,
                    self.fMaterial.fType,
                    depth,
                    self.fMaterial.fK,
                    self.fMaterial.fShininess,
                ))
            },
            None,
        )
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:597-603
    fn onGetInputLayerBounds(
        &self,
        mapping: &Mapping,
        desired: &LayerSpace<SkIRect>,
        content: Option<LayerSpace<SkIRect>>,
    ) -> Result<LayerSpace<SkIRect>, crate::RasterError> {
        self.getChildInputLayerBounds(0, mapping, &self.requiredInput(desired), content)
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:605-612
    fn onGetOutputLayerBounds(
        &self,
        _mapping: &Mapping,
        _content: Option<LayerSpace<SkIRect>>,
    ) -> Result<Option<LayerSpace<SkIRect>>, crate::RasterError> {
        Ok(LayerSpace::<SkIRect>::Unbounded())
    }
}
impl SkImageFilter for SkLightingImageFilter {
    fn base(&self) -> &SkImageFilter_Base {
        &self.base
    }
    // Source: src/effects/imagefilters/SkLightingImageFilter.cpp:614-616
    fn computeFastBounds(&self, _src: &SkRect) -> Result<SkRect, crate::RasterError> {
        Ok(crate::src::core::SkRectPriv::SkRectPriv::MakeLargeS32())
    }
}

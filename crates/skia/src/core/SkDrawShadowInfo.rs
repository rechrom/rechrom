// Copyright 2017 Google LLC. BSD-style license.
use super::{SkColor::SkColor, SkPoint3::SkPoint3};
// Source: src/core/SkDrawShadowInfo.h:26-33
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkDrawShadowRec {
    pub fZPlaneParams: SkPoint3,
    pub fLightPos: SkPoint3,
    pub fLightRadius: f32,
    pub fAmbientColor: SkColor,
    pub fSpotColor: SkColor,
    pub fFlags: u32,
}

pub mod SkDrawShadowMetrics {
    use super::SkDrawShadowRec;
    use crate::src::core::{
        SkMatrix::SkMatrix,
        SkPath::SkPath,
        SkPoint::SkPoint,
        SkPoint3::SkPoint3,
        SkRect::SkRect,
        SkScalar::{SK_ScalarNearlyZero, SkScalarInvert, SkScalarNearlyZero},
    };
    // Source: src/core/SkDrawShadowInfo.h:37-38
    pub const kAmbientHeightFactor: f32 = 1.0 / 128.0;
    pub const kAmbientGeomFactor: f32 = 64.0;
    // Source: src/core/SkDrawShadowInfo.h:42-42
    pub const kMaxAmbientRadius: f32 = 300.0 * kAmbientHeightFactor * kAmbientGeomFactor;
    // Source: src/core/SkDrawShadowInfo.h:44-49
    fn divide_and_pin(numer: f32, denom: f32, min: f32, max: f32) -> f32 {
        let result = crate::include::private::SkTPin::SkTPin(
            crate::src::core::SkFloatingPoint::sk_ieee_float_divide(numer, denom),
            min,
            max,
        );
        crate::SkASSERT!(@source b"src/core/SkDrawShadowInfo.h\0".as_ptr(), 47; @condition b"result >= min && result <= max\0".as_ptr(); result>=min&&result<=max);
        result
    }
    // std::min/max's ordered comparisons retain first-argument NaNs and equal
    // signed-zero values; Rust f32::min/max have different nonfinite semantics.
    fn source_min(a: f32, b: f32) -> f32 {
        if b < a {
            b
        } else {
            a
        }
    }
    fn source_max(a: f32, b: f32) -> f32 {
        if a < b {
            b
        } else {
            a
        }
    }
    // Source: src/core/SkDrawShadowInfo.h:51-53
    pub fn AmbientBlurRadius(height: f32) -> f32 {
        source_min(
            height * kAmbientHeightFactor * kAmbientGeomFactor,
            kMaxAmbientRadius,
        )
    }
    // Source: src/core/SkDrawShadowInfo.h:55-57
    pub fn AmbientRecipAlpha(height: f32) -> f32 {
        1.0 + source_max(height * kAmbientHeightFactor, 0.0)
    }
    // Source: src/core/SkDrawShadowInfo.h:59-61
    pub fn SpotBlurRadius(occluderZ: f32, lightZ: f32, lightRadius: f32) -> f32 {
        lightRadius * divide_and_pin(occluderZ, lightZ - occluderZ, 0.0, 0.95)
    }
    // Source: src/core/SkDrawShadowInfo.h:63-70
    pub fn GetSpotParams(
        occluderZ: f32,
        lightX: f32,
        lightY: f32,
        lightZ: f32,
        lightRadius: f32,
        blurRadius: &mut f32,
        scale: &mut f32,
        translate: &mut SkPoint,
    ) {
        let zRatio = divide_and_pin(occluderZ, lightZ - occluderZ, 0.0, 0.95);
        *blurRadius = lightRadius * zRatio;
        *scale = divide_and_pin(lightZ, lightZ - occluderZ, 1.0, 1.95);
        *translate = SkPoint::make(-zRatio * lightX, -zRatio * lightY);
    }
    // Source: src/core/SkDrawShadowInfo.h:72-81
    pub fn GetDirectionalParams(
        occluderZ: f32,
        lightX: f32,
        lightY: f32,
        lightZ: f32,
        lightRadius: f32,
        blurRadius: &mut f32,
        scale: &mut f32,
        translate: &mut SkPoint,
    ) {
        *blurRadius = lightRadius * occluderZ;
        *scale = 1.0;
        const kMaxZRatio: f32 = 64.0 / SK_ScalarNearlyZero;
        let zRatio = divide_and_pin(occluderZ, lightZ, 0.0, kMaxZRatio);
        *translate = SkPoint::make(-zRatio * lightX, -zRatio * lightY);
    }
    // Source: src/core/SkDrawShadowInfo.cpp:18-20
    fn compute_z(x: f32, y: f32, params: &SkPoint3) -> f32 {
        x * params.fX + y * params.fY + params.fZ
    }
    // Source: src/core/SkDrawShadowInfo.h:86-89
    // Source: src/core/SkDrawShadowInfo.cpp:22-122
    pub fn GetSpotShadowTransform(
        lightPos: &SkPoint3,
        lightRadius: f32,
        ctm: &SkMatrix,
        zPlaneParams: &SkPoint3,
        pathBounds: &SkRect,
        directional: bool,
        shadowTransform: &mut SkMatrix,
        radius: &mut f32,
    ) -> bool {
        let heightFunc =
            |x: f32, y: f32| zPlaneParams.fX * x + zPlaneParams.fY * y + zPlaneParams.fZ;
        let occluderHeight = heightFunc(pathBounds.centerX(), pathBounds.centerY());
        if !ctm.hasPerspective() || directional {
            let mut scale = core::mem::MaybeUninit::<f32>::uninit();
            let mut translate = core::mem::MaybeUninit::<SkPoint>::uninit();
            // Both native branches initialize these exact POD outputs before use.
            // Current safe reference output methods need initialized scratch storage.
            scale.write(0.0);
            translate.write(SkPoint::make(0.0, 0.0));
            let (scale, translate) =
                unsafe { (scale.assume_init_mut(), translate.assume_init_mut()) };
            if directional {
                GetDirectionalParams(
                    occluderHeight,
                    lightPos.fX,
                    lightPos.fY,
                    lightPos.fZ,
                    lightRadius,
                    radius,
                    scale,
                    translate,
                );
            } else {
                GetSpotParams(
                    occluderHeight,
                    lightPos.fX,
                    lightPos.fY,
                    lightPos.fZ,
                    lightRadius,
                    radius,
                    scale,
                    translate,
                );
            }
            shadowTransform.setScaleTranslate(*scale, *scale, translate.fX, translate.fY);
            shadowTransform.preConcat(ctm);
        } else {
            if SkScalarNearlyZero(pathBounds.width(), SK_ScalarNearlyZero)
                || SkScalarNearlyZero(pathBounds.height(), SK_ScalarNearlyZero)
            {
                return false;
            }
            let mut pts = [SkPoint::make(0.0, 0.0); 4];
            ctm.mapRectToQuad(&mut pts, pathBounds);
            let mut pts3D = [SkPoint3::default(); 4];
            let mut z = heightFunc(pathBounds.fLeft, pathBounds.fTop);
            pts3D[0].set(pts[0].fX, pts[0].fY, z);
            z = heightFunc(pathBounds.fRight, pathBounds.fTop);
            pts3D[1].set(pts[1].fX, pts[1].fY, z);
            z = heightFunc(pathBounds.fRight, pathBounds.fBottom);
            pts3D[2].set(pts[2].fX, pts[2].fY, z);
            z = heightFunc(pathBounds.fLeft, pathBounds.fBottom);
            pts3D[3].set(pts[3].fX, pts[3].fY, z);
            for i in 0..4 {
                let dz = lightPos.fZ - pts3D[i].fZ;
                if dz <= SK_ScalarNearlyZero {
                    return false;
                }
                let zRatio = pts3D[i].fZ / dz;
                pts3D[i].fX -= (lightPos.fX - pts3D[i].fX) * zRatio;
                pts3D[i].fY -= (lightPos.fY - pts3D[i].fY) * zRatio;
                pts3D[i].fZ = 1.0;
            }
            let mut h0 = pts3D[1].cross(&pts3D[0]).cross(&pts3D[2].cross(&pts3D[3]));
            let mut h1 = pts3D[0].cross(&pts3D[3]).cross(&pts3D[1].cross(&pts3D[2]));
            let h2 = pts3D[0].cross(&pts3D[2]).cross(&pts3D[1].cross(&pts3D[3]));
            if SkScalarNearlyZero(h2.fZ, SK_ScalarNearlyZero) {
                return false;
            }
            let mut v = pts3D[3] - pts3D[0];
            let w = h0 - pts3D[0];
            let mut perpDot = v.fX * w.fY - v.fY * w.fX;
            if perpDot > 0.0 {
                h0 = -h0;
            }
            // Preserve native reuse of w in the second crossing-direction test.
            v = pts3D[1] - pts3D[0];
            perpDot = v.fX * w.fY - v.fY * w.fX;
            if perpDot < 0.0 {
                h1 = -h1;
            }
            shadowTransform.setAll(
                h0.fX / h2.fZ,
                h1.fX / h2.fZ,
                h2.fX / h2.fZ,
                h0.fY / h2.fZ,
                h1.fY / h2.fZ,
                h2.fY / h2.fZ,
                h0.fZ / h2.fZ,
                h1.fZ / h2.fZ,
                1.0,
            );
            let mut toHomogeneous = SkMatrix::default();
            let xScale = 2.0 / (pathBounds.fRight - pathBounds.fLeft);
            let yScale = 2.0 / (pathBounds.fBottom - pathBounds.fTop);
            toHomogeneous.setAll(
                xScale,
                0.0,
                -xScale * pathBounds.fLeft - 1.0,
                0.0,
                yScale,
                -yScale * pathBounds.fTop - 1.0,
                0.0,
                0.0,
                1.0,
            );
            shadowTransform.preConcat(&toHomogeneous);
            *radius = SpotBlurRadius(occluderHeight, lightPos.fZ, lightRadius);
        }
        true
    }
    // Source: src/core/SkDrawShadowInfo.h:92-92
    // Source: src/core/SkDrawShadowInfo.cpp:124-210
    pub fn GetLocalBounds(
        path: &SkPath,
        rec: &SkDrawShadowRec,
        ctm: &SkMatrix,
        bounds: &mut SkRect,
    ) {
        let mut ambientBounds = *path.getBounds();
        let mut occluderZ;
        if SkScalarNearlyZero(rec.fZPlaneParams.fX, SK_ScalarNearlyZero)
            && SkScalarNearlyZero(rec.fZPlaneParams.fY, SK_ScalarNearlyZero)
        {
            occluderZ = rec.fZPlaneParams.fZ;
        } else {
            occluderZ = compute_z(ambientBounds.fLeft, ambientBounds.fTop, &rec.fZPlaneParams);
            occluderZ = source_max(
                occluderZ,
                compute_z(ambientBounds.fRight, ambientBounds.fTop, &rec.fZPlaneParams),
            );
            occluderZ = source_max(
                occluderZ,
                compute_z(
                    ambientBounds.fLeft,
                    ambientBounds.fBottom,
                    &rec.fZPlaneParams,
                ),
            );
            occluderZ = source_max(
                occluderZ,
                compute_z(
                    ambientBounds.fRight,
                    ambientBounds.fBottom,
                    &rec.fZPlaneParams,
                ),
            );
        }
        // Native POD outputs are initialized before consumption in every branch;
        // initialized scratch permits the existing safe reference output API.
        let mut ambientBlur;
        let mut spotBlur = 0.0;
        let mut spotScale = 0.0;
        let mut spotOffset = SkPoint::make(0.0, 0.0);
        let directional = rec.fFlags
            & crate::src::utils::SkShadowUtils::SkShadowFlags::kDirectionalLight_ShadowFlag as u32
            != 0;
        if ctm.hasPerspective() {
            ctm.mapRect_in_place(&mut ambientBounds);
            ambientBlur = AmbientBlurRadius(occluderZ);
            if directional {
                GetDirectionalParams(
                    occluderZ,
                    rec.fLightPos.fX,
                    rec.fLightPos.fY,
                    rec.fLightPos.fZ,
                    rec.fLightRadius,
                    &mut spotBlur,
                    &mut spotScale,
                    &mut spotOffset,
                );
            } else {
                let devLightPos = ctm.mapPoint(SkPoint::make(rec.fLightPos.fX, rec.fLightPos.fY));
                GetSpotParams(
                    occluderZ,
                    devLightPos.fX,
                    devLightPos.fY,
                    rec.fLightPos.fZ,
                    rec.fLightRadius,
                    &mut spotBlur,
                    &mut spotScale,
                    &mut spotOffset,
                );
            }
        } else {
            let devToSrcScale = SkScalarInvert(ctm.getMinScale());
            let devSpaceAmbientBlur = AmbientBlurRadius(occluderZ);
            ambientBlur = devSpaceAmbientBlur * devToSrcScale;
            if directional {
                GetDirectionalParams(
                    occluderZ,
                    rec.fLightPos.fX,
                    rec.fLightPos.fY,
                    rec.fLightPos.fZ,
                    rec.fLightRadius,
                    &mut spotBlur,
                    &mut spotScale,
                    &mut spotOffset,
                );
                if let Some(inverse) = ctm.invert() {
                    spotOffset = inverse.mapVector(spotOffset);
                }
            } else {
                GetSpotParams(
                    occluderZ,
                    rec.fLightPos.fX,
                    rec.fLightPos.fY,
                    rec.fLightPos.fZ,
                    rec.fLightRadius,
                    &mut spotBlur,
                    &mut spotScale,
                    &mut spotOffset,
                );
            }
            spotBlur *= devToSrcScale;
        }
        let mut spotBounds = ambientBounds;
        ambientBounds.outset(ambientBlur, ambientBlur);
        spotBounds.fLeft *= spotScale;
        spotBounds.fTop *= spotScale;
        spotBounds.fRight *= spotScale;
        spotBounds.fBottom *= spotScale;
        spotBounds.offset(spotOffset.fX, spotOffset.fY);
        spotBounds.outset(spotBlur, spotBlur);
        *bounds = ambientBounds;
        bounds.join(&spotBounds);
        bounds.outset(1.0, 1.0);
        if ctm.hasPerspective() {
            if let Some(inverse) = ctm.invert() {
                inverse.mapRect_in_place(bounds);
            }
        }
    }
}
// Language-only namespace projection; no extra algorithm or source credit.
pub use SkDrawShadowMetrics::*;

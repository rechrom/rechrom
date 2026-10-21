// Copyright 2023 Google LLC. BSD-style license.
use crate::include::core::SkScalar::{
    SK_ScalarNearlyZero, SkScalarNearlyEqual, SkScalarNearlyZero,
};
use crate::src::core::{
    SkColorFilter::{Clamp, SkColorFilter},
    SkEffectPriv::SkStageRec,
    SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
use crate::src::effects::colorfilters::SkColorFilterBase::{SkColorFilterBase, Type};
use std::sync::Arc;
// Source: src/effects/colorfilters/SkMatrixColorFilter.h:23-24
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    kRGBA,
    kHSLA,
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.h:21-21
// Source: src/effects/colorfilters/SkMatrixColorFilter.h:45-48
pub struct SkMatrixColorFilter {
    fMatrix: [f32; 20],
    fAlphaIsUnchanged: bool,
    fDomain: Domain,
    fClamp: Clamp,
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:27-28
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:30-32
fn is_alpha_unchanged(matrix: &[f32; 20]) -> bool {
    let srcA = &matrix[15..];
    SkScalarNearlyZero(srcA[0], SK_ScalarNearlyZero)
        && SkScalarNearlyZero(srcA[1], SK_ScalarNearlyZero)
        && SkScalarNearlyZero(srcA[2], SK_ScalarNearlyZero)
        && SkScalarNearlyEqual(srcA[3], 1.0, SK_ScalarNearlyZero)
        && SkScalarNearlyZero(srcA[4], SK_ScalarNearlyZero)
}
impl SkMatrixColorFilter {
    // Source: src/effects/colorfilters/SkMatrixColorFilter.h:26-26
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:35-37
    pub fn new(array: &[f32; 20], domain: Domain, clamp: Clamp) -> Self {
        Self {
            fAlphaIsUnchanged: is_alpha_unchanged(array),
            fDomain: domain,
            fClamp: clamp,
            fMatrix: *array,
        }
    }
    // Source: src/effects/colorfilters/SkMatrixColorFilter.h:34-36
    pub fn domain(&self) -> Domain {
        self.fDomain
    }
    pub fn clamp(&self) -> Clamp {
        self.fClamp
    }
    pub fn matrix(&self) -> &[f32; 20] {
        &self.fMatrix
    }
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.h:40-40
impl crate::src::core::SkFlattenable::SkFlattenableView for SkMatrixColorFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkMatrixColorFilter")
    }
    fn getFlattenableType(&self) -> crate::src::core::SkFlattenable::Type {
        crate::src::core::SkFlattenable::Type::kSkColorFilter_Type
    }
    fn flatten(
        &self,
        writer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.flatten_impl(writer)?;
        Ok(())
    }
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:40-47
impl SkMatrixColorFilter {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        crate::SkASSERT!(@source b"src/effects/colorfilters/SkMatrixColorFilter.cpp\0".as_ptr(),41; @condition b"sizeof(fMatrix) / sizeof(float) == 20\0".as_ptr(); self.fMatrix.len()==20);
        buffer.writeScalarArray(&self.fMatrix);
        buffer.writeBool(self.fDomain == Domain::kRGBA);
        buffer.writeBool(self.fClamp == Clamp::kYes);

        Ok(())
    }
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:49-61
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let mut matrix = [0.0; 20];
        if !buffer.readScalarArray(&mut matrix) {
            return Ok(None);
        }
        let is_rgba = buffer.readBool();
        let clamp =
            if buffer.isVersionLT(crate::src::core::SkPicturePriv::kUnclampedMatrixColorFilter) {
                Clamp::kYes
            } else if buffer.readBool() {
                Clamp::kYes
            } else {
                Clamp::kNo
            };
        Ok((if is_rgba {
            Matrix(&matrix, clamp)
        } else {
            HSLAMatrix(&matrix)
        })
        .map(crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter))
    }
}
impl SkColorFilterBase for SkMatrixColorFilter {
    // Source: src/effects/colorfilters/SkMatrixColorFilter.h:30-30
    // Source: src/effects/colorfilters/SkMatrixColorFilter.h:32-32
    fn onIsAlphaUnchanged(&self) -> bool {
        self.fAlphaIsUnchanged
    }
    fn r#type(&self) -> Type {
        Type::kMatrix
    }
    // Source: src/effects/colorfilters/SkMatrixColorFilter.h:43-43
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:63-65
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:67-67
    fn onAsAColorMatrix(&self, matrix: Option<&mut [f32; 20]>) -> bool {
        if let Some(matrix) = matrix {
            *matrix = self.fMatrix;
        }
        true
    }
    // Source: src/effects/colorfilters/SkMatrixColorFilter.h:28-28
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:70-73
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:75-77
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:79-80
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:82-83
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:85-86
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:88-90
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:92-92
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:94-95
    // Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:97-97
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        shaderIsOpaque: bool,
    ) -> Result<bool, crate::RasterError> {
        let willStayOpaque = shaderIsOpaque && self.fAlphaIsUnchanged;
        let hsla = self.fDomain == Domain::kHSLA;
        let clamp = self.fClamp == Clamp::kYes;
        let p = unsafe { &mut *rec.fPipeline };
        unsafe {
            if !shaderIsOpaque {
                p.append(Op::unpremul, std::ptr::null_mut());
            }
            if hsla {
                p.append(Op::rgb_to_hsl, std::ptr::null_mut());
            }
            if true {
                p.append(Op::matrix_4x5, self.fMatrix.as_ptr().cast_mut().cast());
            }
            if hsla {
                p.append(Op::hsl_to_rgb, std::ptr::null_mut());
            }
            if clamp {
                p.append(Op::clamp_01, std::ptr::null_mut());
            } else {
                p.append(Op::clamp_a_01, std::ptr::null_mut());
            }
            if !willStayOpaque {
                p.append(Op::premul, std::ptr::null_mut());
            }
        }
        Ok(true)
    }
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:102-106
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:108-108
fn MakeMatrix(array: &[f32; 20], domain: Domain, clamp: Clamp) -> Option<Arc<dyn SkColorFilter>> {
    if !crate::src::core::SkFloatingPoint::SkIsFinite(array) {
        return None;
    }
    Some(Arc::new(SkMatrixColorFilter::new(array, domain, clamp)))
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:111-112
pub fn Matrix(array: &[f32; 20], clamp: Clamp) -> Option<Arc<dyn SkColorFilter>> {
    MakeMatrix(array, Domain::kRGBA, clamp)
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:115-116
pub fn MatrixFromColorMatrix(
    cm: &crate::src::effects::SkColorMatrix::SkColorMatrix,
    clamp: Clamp,
) -> Option<Arc<dyn SkColorFilter>> {
    MakeMatrix(&cm.fMat, Domain::kRGBA, clamp)
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:119-120
pub fn HSLAMatrix(array: &[f32; 20]) -> Option<Arc<dyn SkColorFilter>> {
    MakeMatrix(array, Domain::kHSLA, Clamp::kYes)
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:123-124
pub fn HSLAMatrixFromColorMatrix(
    cm: &crate::src::effects::SkColorMatrix::SkColorMatrix,
) -> Option<Arc<dyn SkColorFilter>> {
    MakeMatrix(&cm.fMat, Domain::kHSLA, Clamp::kYes)
}
// Source: src/effects/colorfilters/SkMatrixColorFilter.cpp:127-131
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterMatrixColorFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkMatrixColorFilter",
            SkMatrixColorFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkColorFilter_Matrix",
            SkMatrixColorFilter::CreateProc,
        );
    }
}

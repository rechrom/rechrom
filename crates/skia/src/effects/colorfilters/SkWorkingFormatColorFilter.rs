// Copyright 2023 Google LLC. BSD-style license.
use super::SkColorFilterBase::{ColorFilterEvaluationError, SkColorFilterBase, Type};
use crate::include::core::SkAlphaType::SkAlphaType;
use crate::modules::skcms::src::skcms_public::{skcms_Matrix3x3, skcms_TransferFunction};
use crate::src::core::{
    SkBlendMode::SkBlendMode, SkColor::SkColor, SkColorData::SkPMColor4f,
    SkColorFilter::SkColorFilter, SkColorSpace::SkColorSpace,
    SkColorSpaceXformSteps::SkColorSpaceXformSteps, SkEffectPriv::SkStageRec,
};
use std::sync::Arc;

// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.h:25-29
// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.h:36-41
// None stores the native use-destination flag without reading inactive POD payloads.
pub struct SkWorkingFormatCalculator {
    fTF: Option<skcms_TransferFunction>,
    fGamut: Option<skcms_Matrix3x3>,
    fAT: Option<SkAlphaType>,
}

// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.h:44-44
impl SkWorkingFormatColorFilter {
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.h:53-55
    pub fn r#type(&self) -> Type {
        Type::kWorkingFormat
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.h:64-64
    pub fn child(&self) -> Arc<dyn SkColorFilter> {
        self.fChild.clone()
    }
}

// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.h:78-79
// Declaration order preserves the native reverse member destruction order.
pub struct SkWorkingFormatColorFilter {
    fWorkingFormatCalculator: SkWorkingFormatCalculator,
    fChild: Arc<dyn SkColorFilter>,
}

impl SkWorkingFormatColorFilter {
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:28-35
    pub fn new(
        child: Arc<dyn SkColorFilter>,
        tf: Option<&skcms_TransferFunction>,
        gamut: Option<&skcms_Matrix3x3>,
        at: Option<&SkAlphaType>,
    ) -> Self {
        let calculator = SkWorkingFormatCalculator::new(tf, gamut, at);
        Self {
            fChild: child,
            fWorkingFormatCalculator: calculator,
        }
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:37-40
    pub fn workingFormat(
        &self,
        dst: &Arc<SkColorSpace>,
        at: &mut SkAlphaType,
    ) -> Option<Arc<SkColorSpace>> {
        self.fWorkingFormatCalculator.workingFormat(dst, at)
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:42-77
    pub unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        shader_is_opaque: bool,
    ) -> Result<bool, crate::RasterError> {
        use SkAlphaType::kPremul_SkAlphaType;
        let dst_cs = rec.fDstCS.cloned().unwrap_or_else(SkColorSpace::MakeSRGB);
        let mut working_at = kPremul_SkAlphaType;
        let working_cs = self.workingFormat(&dst_cs, &mut working_at);
        // ColorInfo's type is identical on both sides and not used by XformSteps.
        let dst_to_working = (*rec.fAlloc).make(SkColorSpaceXformSteps::new(
            Some(&dst_cs),
            kPremul_SkAlphaType,
            working_cs.as_deref(),
            working_at,
        ));
        let working_to_dst = (*rec.fAlloc).make(SkColorSpaceXformSteps::new(
            working_cs.as_deref(),
            working_at,
            Some(&dst_cs),
            kPremul_SkAlphaType,
        ));
        let working_rec = SkStageRec {
            fPipeline: rec.fPipeline,
            fAlloc: rec.fAlloc,
            fDstColorType: rec.fDstColorType,
            fDstCS: working_cs.as_ref(),
            fPaintColor: rec.fPaintColor,
            fSurfaceProps: rec.fSurfaceProps,
            fDstBounds: rec.fDstBounds,
        };
        (*dst_to_working).applyPipeline(&mut *rec.fPipeline);
        if !self.fChild.appendStages(&working_rec, shader_is_opaque)? {
            return Ok(false);
        }
        (*working_to_dst).applyPipeline(&mut *rec.fPipeline);
        Ok(true)
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:79-98
    pub fn onFilterColor4f(
        &self,
        original: &SkPMColor4f,
        raw_dst_cs: Option<&Arc<SkColorSpace>>,
    ) -> Result<SkPMColor4f, ColorFilterEvaluationError> {
        use SkAlphaType::kPremul_SkAlphaType;
        let dst_cs = raw_dst_cs.cloned().unwrap_or_else(SkColorSpace::MakeSRGB);
        let mut working_at = kPremul_SkAlphaType;
        let working_cs = self.workingFormat(&dst_cs, &mut working_at);
        let mut color = *original;
        SkColorSpaceXformSteps::new(
            Some(&dst_cs),
            kPremul_SkAlphaType,
            working_cs.as_deref(),
            working_at,
        )
        .apply(color.vec_mut());
        color = self.fChild.onFilterColor4f(&color, working_cs.as_ref())?;
        SkColorSpaceXformSteps::new(
            working_cs.as_deref(),
            working_at,
            Some(&dst_cs),
            kPremul_SkAlphaType,
        )
        .apply(color.vec_mut());
        Ok(color)
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:100-100
    pub fn onIsAlphaUnchanged(&self) -> bool {
        self.fChild.isAlphaUnchanged()
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:102-104
    pub fn onAsAColorMode(
        &self,
        color: Option<&mut SkColor>,
        mode: Option<&mut SkBlendMode>,
    ) -> bool {
        self.fChild.asAColorMode(color, mode)
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:106-108
    pub fn onAsAColorMatrix(&self, matrix: Option<&mut [f32; 20]>) -> bool {
        self.fChild.asAColorMatrix(matrix)
    }
}

// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.h:68-68
impl crate::src::core::SkFlattenable::SkFlattenableView for SkWorkingFormatColorFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkWorkingFormatColorFilter")
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
// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:110-113
impl SkWorkingFormatColorFilter {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeFlattenable(Some(&*self.fChild))?;
        self.fWorkingFormatCalculator.flatten(buffer)?;
        Ok(())
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:115-135
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let child = buffer.readColorFilter()?;
        let use_dst_tf = buffer.readBool();
        let use_dst_gamut = buffer.readBool();
        let use_dst_at = buffer.readBool();
        // Initialized scratch arrays define only corrupt-stream native indeterminate reads;
        // valid streams fill the exact native payload, inactive payloads are never consulted.
        let tf = if use_dst_tf {
            None
        } else {
            let mut v = [0.0; 7];
            buffer.readScalarArray(&mut v);
            Some(skcms_TransferFunction {
                g: v[0],
                a: v[1],
                b: v[2],
                c: v[3],
                d: v[4],
                e: v[5],
                f: v[6],
            })
        };
        let gamut = if use_dst_gamut {
            None
        } else {
            let mut v = [0.0; 9];
            buffer.readScalarArray(&mut v);
            Some(skcms_Matrix3x3 {
                vals: [[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]],
            })
        };
        let at = if use_dst_at {
            None
        } else {
            let v =
                buffer.read32LE(crate::include::core::SkAlphaType::kLastEnum_SkAlphaType as u32);
            Some(unsafe { std::mem::transmute::<u32, SkAlphaType>(v) })
        };
        Ok(
            WithWorkingFormat(child, tf.as_ref(), gamut.as_ref(), at.as_ref())
                .map(crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter),
        )
    }
}
// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:140-152
pub fn WithWorkingFormat(
    child: Option<Arc<dyn SkColorFilter>>,
    tf: Option<&skcms_TransferFunction>,
    gamut: Option<&skcms_Matrix3x3>,
    at: Option<&SkAlphaType>,
) -> Option<Arc<dyn SkColorFilter>> {
    let child = child?;
    Some(Arc::new(SkWorkingFormatColorFilter::new(
        child, tf, gamut, at,
    )))
}

// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:154-156
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterWorkingFormatColorFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkWorkingFormatColorFilter",
            SkWorkingFormatColorFilter::CreateProc,
        );
    }
}
impl SkWorkingFormatCalculator {
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:158-173
    pub fn new(
        tf: Option<&skcms_TransferFunction>,
        gamut: Option<&skcms_Matrix3x3>,
        at: Option<&SkAlphaType>,
    ) -> Self {
        Self {
            fTF: tf.copied(),
            fGamut: gamut.copied(),
            fAT: at.copied(),
        }
    }
    // Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:175-193
    pub fn workingFormat(
        &self,
        dst: &Arc<SkColorSpace>,
        at: &mut SkAlphaType,
    ) -> Option<Arc<SkColorSpace>> {
        let mut tf = self
            .fTF
            .unwrap_or(crate::src::core::SkColorSpace::SkNamedTransferFn::kLinear);
        let mut gamut = self.fGamut.unwrap_or(skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        });
        if self.fTF.is_none() {
            let numerical = dst.isNumericalTransferFn(&mut tf);
            crate::SkAssertResult!(@source b"src/effects/colorfilters/SkWorkingFormatColorFilter.cpp\0".as_ptr(),181; @condition b"dstCS->isNumericalTransferFn(&tf)\0".as_ptr(); numerical);
        }
        if self.fGamut.is_none() {
            let available = dst.toXYZD50(&mut gamut);
            crate::SkAssertResult!(@source b"src/effects/colorfilters/SkWorkingFormatColorFilter.cpp\0".as_ptr(),186; @condition b"dstCS->toXYZD50(&gamut)\0".as_ptr(); available);
        }
        *at = self.fAT.unwrap_or(SkAlphaType::kPremul_SkAlphaType);
        SkColorSpace::MakeRGB(&tf, &gamut)
    }
}

// Native inherited virtual forwarding; no second object, owner or execution algorithm.
impl SkColorFilterBase for SkWorkingFormatColorFilter {
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        opaque: bool,
    ) -> Result<bool, crate::RasterError> {
        Self::appendStages(self, rec, opaque)
    }
    fn r#type(&self) -> Type {
        Self::r#type(self)
    }
    fn onIsAlphaUnchanged(&self) -> bool {
        Self::onIsAlphaUnchanged(self)
    }
    fn onAsAColorMode(&self, color: Option<&mut SkColor>, mode: Option<&mut SkBlendMode>) -> bool {
        Self::onAsAColorMode(self, color, mode)
    }
    fn onAsAColorMatrix(&self, matrix: Option<&mut [f32; 20]>) -> bool {
        Self::onAsAColorMatrix(self, matrix)
    }
    fn onFilterColor4f(
        &self,
        color: &SkPMColor4f,
        dst: Option<&Arc<SkColorSpace>>,
    ) -> Result<SkPMColor4f, ColorFilterEvaluationError> {
        Self::onFilterColor4f(self, color, dst)
    }
}
// Source: src/effects/colorfilters/SkWorkingFormatColorFilter.cpp:195-209
impl SkWorkingFormatCalculator {
    pub fn flatten(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeBool(self.fTF.is_none());
        buffer.writeBool(self.fGamut.is_none());
        buffer.writeBool(self.fAT.is_none());
        if let Some(tf) = self.fTF {
            buffer.writeScalarArray(&[tf.g, tf.a, tf.b, tf.c, tf.d, tf.e, tf.f]);
        }
        if let Some(g) = self.fGamut {
            buffer.writeScalarArray(&[
                g.vals[0][0],
                g.vals[0][1],
                g.vals[0][2],
                g.vals[1][0],
                g.vals[1][1],
                g.vals[1][2],
                g.vals[2][0],
                g.vals[2][1],
                g.vals[2][2],
            ]);
        }
        if let Some(at) = self.fAT {
            buffer.writeInt(at as i32);
        }

        Ok(())
    }
}

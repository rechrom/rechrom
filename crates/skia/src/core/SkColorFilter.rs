use std::sync::Arc;
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::{SkBlendMode::SkBlendMode, SkColor::SkColor};
use crate::src::effects::colorfilters::SkColorFilterBase::SkColorFilterBase;
// Native public/base inheritance is expressed as a trait hierarchy; required CPU append is
// inherited. Scalar evaluation surfaces the canonical runner's pending-stage errors.
// Source: include/core/SkColorFilter.h:35-35
// Source: include/core/SkColorFilter.h:41-41
// Source: include/core/SkColorFilter.h:47-47
// Source: include/core/SkColorFilter.h:50-50
// Source: include/core/SkColorFilter.h:76-76
// Adapted native base construction: Rust trait composition has no separately
// constructible base subobject. Real concrete factories initialize Arc owners;
// this replaces the implicit SkFlattenable/SkRefCntBase count=1 construction.
pub trait SkColorFilter: SkColorFilterBase {
    // Source: src/core/SkColorFilter.cpp:25-26
    fn asAColorMode(&self, color: Option<&mut SkColor>, mode: Option<&mut SkBlendMode>) -> bool {
        self.onAsAColorMode(color, mode)
    }
    // Source: src/core/SkColorFilter.cpp:29-30
    fn asAColorMatrix(&self, matrix: Option<&mut [f32; 20]>) -> bool {
        self.onAsAColorMatrix(matrix)
    }
    // Source: src/core/SkColorFilter.cpp:33-34
    fn isAlphaUnchanged(&self) -> bool {
        self.onIsAlphaUnchanged()
    }
    // Source: include/core/SkColorFilter.h:56-57
    // Source: src/core/SkColorFilter.cpp:44-52
    // Adapted: preserve source color conversion/evaluation order but expose incomplete runner stages.
    fn filterColor4f(
        &self,
        srcColor: &crate::src::core::SkColor::SkColor4f,
        srcCS: Option<&crate::src::core::SkColorSpace::SkColorSpace>,
        dstCS: Option<&std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    ) -> Result<
        crate::src::core::SkColor::SkColor4f,
        crate::src::effects::colorfilters::SkColorFilterBase::ColorFilterEvaluationError,
    > {
        use crate::include::core::SkAlphaType::SkAlphaType::{
            kPremul_SkAlphaType, kUnpremul_SkAlphaType,
        };
        let mut color = crate::src::core::SkColorData::SkPMColor4f {
            fR: srcColor.fR,
            fG: srcColor.fG,
            fB: srcColor.fB,
            fA: srcColor.fA,
        };
        crate::src::core::SkColorSpaceXformSteps::SkColorSpaceXformSteps::new(
            srcCS,
            kUnpremul_SkAlphaType,
            dstCS.map(std::sync::Arc::as_ref),
            kPremul_SkAlphaType,
        )
        .apply(color.vec_mut());
        Ok(self.onFilterColor4f(&color, dstCS)?.pinAlpha().unpremul())
    }
}
impl<T: SkColorFilterBase + ?Sized> SkColorFilter for T {}
// Source: include/core/SkColorFilter.h:64-64
pub use crate::src::effects::colorfilters::SkComposeColorFilter::makeComposed;

// Source: include/core/SkColorFilter.h:95-95
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clamp {
    kNo,
    kYes,
}

// Source: include/core/SkColorFilter.h:98-98
// Source: include/core/SkColorFilter.h:103-103
pub use crate::src::effects::colorfilters::SkMatrixColorFilter::{HSLAMatrix, Matrix};

// Source: include/core/SkColorFilter.h:84-87
pub fn Compose(
    outer: Option<&Arc<dyn SkColorFilter>>,
    inner: Option<Arc<dyn SkColorFilter>>,
) -> Option<Arc<dyn SkColorFilter>> {
    if let Some(outer) = outer {
        Some(outer.makeComposed(inner))
    } else {
        inner
    }
}

// Source: include/core/SkColorFilter.h:92-93
pub use crate::src::effects::colorfilters::SkBlendModeColorFilter::{Blend, BlendColor};

// Source: include/core/SkColorFilter.h:105-106
pub use crate::src::effects::colorfilters::SkColorSpaceXformColorFilter::{
    LinearToSRGBGamma, SRGBToLinearGamma,
};

// Source: include/core/SkColorFilter.h:120-120
// Source: include/core/SkColorFilter.h:128-131
// Source: include/core/SkColorFilter.h:136-136
pub use crate::src::effects::colorfilters::SkTableColorFilter::{Table, TableARGB, TableArray};

impl dyn SkColorFilter {
    // Source: src/core/SkColorFilter.cpp:54-67
    pub fn makeWithWorkingColorSpace(
        self: &Arc<Self>,
        workingSpace: Option<Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    ) -> Option<Arc<dyn SkColorFilter>> {
        let Some(workingSpace) = workingSpace else {
            return Some(self.clone());
        };
        let mut tf = crate::src::core::SkColorSpace::SkNamedTransferFn::kLinear;
        let mut toXYZ = crate::modules::skcms::src::skcms_public::skcms_Matrix3x3 {
            vals: [[0.0; 3]; 3],
        };
        workingSpace.transferFn(&mut tf);
        workingSpace.toXYZD50(&mut toXYZ);
        crate::src::core::SkColorFilterPriv::WithWorkingFormat(
            Some(self.clone()),
            Some(&tf),
            Some(&toXYZ),
            None,
        )
    }
}

// Source: include/core/SkColorFilter.h:107-107
pub use crate::src::effects::colorfilters::SkRuntimeColorFilter::Lerp;

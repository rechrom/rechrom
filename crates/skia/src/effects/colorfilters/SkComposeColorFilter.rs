// Copyright 2023 Google LLC. BSD-style license.
use crate::src::core::{SkColorFilter::SkColorFilter, SkEffectPriv::SkStageRec};
use crate::src::effects::colorfilters::SkColorFilterBase::{SkColorFilterBase, Type};
use std::sync::Arc;
// Source: src/effects/colorfilters/SkComposeColorFilter.h:19-19
// Source: src/effects/colorfilters/SkComposeColorFilter.h:39-40
// Rust drops fields in declaration order; reverse the C++ declarations so inner releases before outer.
pub struct SkComposeColorFilter {
    fInner: Arc<dyn SkColorFilter>,
    fOuter: Arc<dyn SkColorFilter>,
}
impl SkComposeColorFilter {
    // Source: src/effects/colorfilters/SkComposeColorFilter.h:37-37
    // Source: src/effects/colorfilters/SkComposeColorFilter.cpp:19-21
    fn new(outer: Arc<dyn SkColorFilter>, inner: Arc<dyn SkColorFilter>) -> Self {
        Self {
            fOuter: outer,
            fInner: inner,
        }
    }
    // Source: src/effects/colorfilters/SkComposeColorFilter.h:27-28
    pub fn outer(&self) -> Arc<dyn SkColorFilter> {
        self.fOuter.clone()
    }
    pub fn inner(&self) -> Arc<dyn SkColorFilter> {
        self.fInner.clone()
    }
}
impl SkColorFilterBase for SkComposeColorFilter {
    // Source: src/effects/colorfilters/SkComposeColorFilter.h:21-21
    // Source: src/effects/colorfilters/SkComposeColorFilter.cpp:24-24
    // Source: src/effects/colorfilters/SkComposeColorFilter.cpp:26-26
    fn onIsAlphaUnchanged(&self) -> bool {
        self.fOuter.isAlphaUnchanged() && self.fInner.isAlphaUnchanged()
    }
    // Source: src/effects/colorfilters/SkComposeColorFilter.h:23-23
    // Source: src/effects/colorfilters/SkComposeColorFilter.h:25-25
    // Source: src/effects/colorfilters/SkComposeColorFilter.cpp:29-32
    // Source: src/effects/colorfilters/SkComposeColorFilter.cpp:34-34
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        shaderIsOpaque: bool,
    ) -> Result<bool, crate::RasterError> {
        let mut innerIsOpaque = shaderIsOpaque;
        if !self.fInner.isAlphaUnchanged() {
            innerIsOpaque = false;
        }
        unsafe {
            Ok(self.fInner.appendStages(rec, shaderIsOpaque)?
                && self.fOuter.appendStages(rec, innerIsOpaque)?)
        }
    }
    fn r#type(&self) -> Type {
        Type::kCompose
    }
}
// Source: src/effects/colorfilters/SkComposeColorFilter.h:35-35
impl crate::src::core::SkFlattenable::SkFlattenableView for SkComposeColorFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkComposeColorFilter")
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
// Source: src/effects/colorfilters/SkComposeColorFilter.cpp:37-40
impl SkComposeColorFilter {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        buffer.writeFlattenable(Some(&*self.fOuter))?;
        buffer.writeFlattenable(Some(&*self.fInner))?;
        Ok(())
    }
    // Source: src/effects/colorfilters/SkComposeColorFilter.cpp:42-46
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        let outer = buffer.readColorFilter()?;
        let inner = buffer.readColorFilter()?;
        Ok((if let Some(outer) = outer {
            Some(outer.makeComposed(inner))
        } else {
            inner
        })
        .map(crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter))
    }
}
// Source: src/effects/colorfilters/SkComposeColorFilter.cpp:48-50
// Source: src/effects/colorfilters/SkComposeColorFilter.cpp:53-53
// Arc borrow models the original const method taking a retained reference to this.
pub fn makeComposed(
    outer: &Arc<dyn SkColorFilter>,
    inner: Option<Arc<dyn SkColorFilter>>,
) -> Arc<dyn SkColorFilter> {
    let Some(inner) = inner else {
        return outer.clone();
    };
    Arc::new(SkComposeColorFilter::new(outer.clone(), inner))
}
impl dyn SkColorFilter {
    pub fn makeComposed(
        self: &Arc<Self>,
        inner: Option<Arc<dyn SkColorFilter>>,
    ) -> Arc<dyn SkColorFilter> {
        makeComposed(self, inner)
    }
}
// Source: src/effects/colorfilters/SkComposeColorFilter.cpp:56-56
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterComposeColorFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkComposeColorFilter",
            SkComposeColorFilter::CreateProc,
        );
    }
}

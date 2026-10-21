// Copyright 2015 Google Inc. BSD-style license.
use super::SkColorFilterBase::Type;
use crate::src::core::{SkBitmap::SkBitmap, SkColorTable::SkColorTable};
use std::sync::Arc;
// Source: src/effects/colorfilters/SkTableColorFilter.h:22-22
// Source: src/effects/colorfilters/SkTableColorFilter.h:40-40
pub struct SkTableColorFilter {
    fTable: Arc<SkColorTable>,
}
impl SkTableColorFilter {
    // Source: src/effects/colorfilters/SkTableColorFilter.h:24-26
    pub fn new(table: Arc<SkColorTable>) -> Self {
        Self { fTable: table }
    }
    // Source: src/effects/colorfilters/SkTableColorFilter.h:28-28
    pub fn r#type(&self) -> Type {
        Type::kTable
    }
    // Source: src/effects/colorfilters/SkTableColorFilter.h:34-34
    pub fn bitmap(&self) -> &SkBitmap {
        self.fTable.bitmap()
    }
}

use super::SkColorFilterBase::SkColorFilterBase;
use crate::src::core::{
    SkColorFilter::SkColorFilter, SkEffectPriv::SkStageRec, SkRasterPipelineOpContexts::TablesCtx,
    SkRasterPipelineOpList::SkRasterPipelineOp as Op,
};
impl SkColorFilterBase for SkTableColorFilter {
    fn r#type(&self) -> Type {
        Type::kTable
    }
    // Source: src/effects/colorfilters/SkTableColorFilter.h:30-30
    // Source: src/effects/colorfilters/SkTableColorFilter.cpp:25-44
    unsafe fn appendStages(
        &self,
        rec: &SkStageRec<'_>,
        shaderIsOpaque: bool,
    ) -> Result<bool, crate::RasterError> {
        unsafe {
            let p = &mut *rec.fPipeline;
            if !shaderIsOpaque {
                p.append(Op::unpremul, std::ptr::null_mut());
            }
            let tables = (*rec.fAlloc).make(TablesCtx {
                a: self.fTable.alphaTable(),
                r: self.fTable.redTable(),
                g: self.fTable.greenTable(),
                b: self.fTable.blueTable(),
            });
            p.append(Op::byte_tables, tables.cast());
            let definitelyOpaque = shaderIsOpaque && *(*tables).a.add(0xff) == 0xff;
            if !definitelyOpaque {
                p.append(Op::premul, std::ptr::null_mut());
            }
        }
        Ok(true)
    }
}
// Source: src/effects/colorfilters/SkTableColorFilter.h:38-38
impl crate::src::core::SkFlattenable::SkFlattenableView for SkTableColorFilter {
    fn getFactory(&self) -> Option<crate::src::core::SkFlattenable::Factory> {
        Some(Self::CreateProc)
    }
    fn getTypeName(&self) -> Option<&str> {
        Some("SkTableColorFilter")
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
// Source: src/effects/colorfilters/SkTableColorFilter.cpp:46-48
impl SkTableColorFilter {
    fn flatten_impl(
        &self,
        buffer: &mut dyn crate::src::core::SkWriteBuffer::SkWriteBuffer,
    ) -> Result<(), crate::RasterError> {
        self.fTable.flatten(buffer);
        Ok(())
    }
    // Source: src/effects/colorfilters/SkTableColorFilter.cpp:50-52
    pub fn CreateProc(
        buffer: &mut crate::src::core::SkReadBuffer::SkReadBuffer<'_>,
    ) -> crate::src::core::SkFlattenable::FactoryResult {
        Ok(Table(SkColorTable::Deserialize(buffer))
            .map(crate::src::core::SkFlattenable::FlattenableOwner::ColorFilter))
    }
}
// Source: src/effects/colorfilters/SkTableColorFilter.cpp:56-58
pub fn TableArray(table: Option<&[u8; 256]>) -> Option<Arc<dyn SkColorFilter>> {
    Table(SkColorTable::Make(table))
}
// Source: src/effects/colorfilters/SkTableColorFilter.cpp:60-65
pub fn TableARGB(
    tableA: Option<&[u8; 256]>,
    tableR: Option<&[u8; 256]>,
    tableG: Option<&[u8; 256]>,
    tableB: Option<&[u8; 256]>,
) -> Option<Arc<dyn SkColorFilter>> {
    Table(SkColorTable::MakeARGB(tableA, tableR, tableG, tableB))
}
// Source: src/effects/colorfilters/SkTableColorFilter.cpp:67-72
pub fn Table(table: Option<Arc<SkColorTable>>) -> Option<Arc<dyn SkColorFilter>> {
    let table = table?;
    Some(Arc::new(SkTableColorFilter::new(table)))
}
// Source: src/effects/colorfilters/SkTableColorFilter.cpp:74-78
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub unsafe fn SkRegisterTableColorFilterFlattenable() {
    unsafe {
        crate::src::core::SkFlattenable::Register(
            b"SkTableColorFilter",
            SkTableColorFilter::CreateProc,
        );
        crate::src::core::SkFlattenable::Register(
            b"SkTable_ColorFilter",
            SkTableColorFilter::CreateProc,
        );
    }
}

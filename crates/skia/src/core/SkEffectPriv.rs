// Copyright 2019 Google LLC. BSD-style license.
use crate::include::core::{SkColorType::SkColorType, SkSurfaceProps::SkSurfaceProps};
use crate::src::core::{
    SkArenaAlloc::SkArenaAlloc, SkColor::SkColor4f, SkColorSpace::SkColorSpace,
    SkRasterPipeline::SkRasterPipeline, SkRect::SkRect,
};

// Source: src/core/SkEffectPriv.h:21-27
// Source: src/core/SkEffectPriv.h:31-31
// Pipeline/arena pointers retain source aliasing and caller ownership; effects do not own them.
// References express the source's borrowed color-space and surface-property lifetimes.
#[derive(Clone, Copy)]
pub struct SkStageRec<'a> {
    pub fPipeline: *mut SkRasterPipeline,
    pub fAlloc: *mut SkArenaAlloc,
    pub fDstColorType: SkColorType,
    pub fDstCS: Option<&'a std::sync::Arc<SkColorSpace>>,
    pub fPaintColor: SkColor4f,
    pub fSurfaceProps: &'a SkSurfaceProps,
    pub fDstBounds: SkRect,
}

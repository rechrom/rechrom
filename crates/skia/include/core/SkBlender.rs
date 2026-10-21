// Copyright 2021 Google LLC. BSD-style license.
use crate::include::core::SkRefCnt::sk_sp;
use crate::src::core::SkBlendMode::SkBlendMode;
use crate::src::core::SkBlenderBase::SkBlenderBase;
use crate::src::core::SkFlattenable::SkFlattenable;
// Source: include/core/SkBlender.h:18-28
// Rust's public trait combines the native retained identity and the internal
// CPU view for sound trait upcasting (the C++ internal as_BB static cast).
// Refcounts/destruction belong solely to the canonical SkRefCntObject owner.
pub trait SkBlender: SkFlattenable + SkBlenderBase + Send + Sync {}
impl dyn SkBlender {
    // Source: include/core/SkBlender.h:23-23
    pub fn Mode(mode: SkBlendMode) -> sk_sp<dyn SkBlender> {
        crate::src::core::SkBlendModeBlender::BlenderMode(mode)
    }
}

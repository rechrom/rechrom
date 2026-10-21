// Copyright 2016 Google Inc. BSD-style license.
use crate::src::core::SkBlendMode::SkBlendMode;
// Source: src/core/SkBlendModePriv.h:24-30
pub const kCustom_SkBlendMode: u8 = 0xff;
pub fn SkBlendMode_CaresAboutRBOrder(mode: SkBlendMode) -> bool {
    mode > SkBlendMode::kLastSeparableMode
}
// Source: src/core/SkBlendModePriv.h:35-35
pub use crate::src::core::SkBlendMode::SkBlendMode_Apply;
// Source: src/core/SkBlendModePriv.h:37-41
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkBlendFastPath {
    kNormal,
    kSrcOver,
    kSkipDrawing,
}
// Source: src/core/SkBlendModePriv.h:47-47
pub use crate::src::core::SkBlendMode::CheckFastPath;

// Source: src/core/SkBlendModePriv.h:32-33
pub use crate::src::core::SkBlendMode::{
    SkBlendMode_AppendStages, SkBlendMode_ShouldPreScaleCoverage,
};
// Source: src/core/SkBlendModePriv.h:49-49
pub use crate::src::core::SkBlendModeBlender::GetBlendModeSingleton;

// Copyright 2019 Google LLC. BSD-style license.
// Source: include/core/SkTileMode.h:13-13
// Source: include/core/SkTileMode.h:18-18
// Source: include/core/SkTileMode.h:23-23
// Source: include/core/SkTileMode.h:29-29
// Source: include/core/SkTileMode.h:34-34
// Source: include/core/SkTileMode.h:36-36
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkTileMode {
    kClamp,
    kRepeat,
    kMirror,
    kDecal,
}
impl SkTileMode {
    pub const kLastTileMode: Self = Self::kDecal;
}
// Source: include/core/SkTileMode.h:39-39
pub const kSkTileModeCount: i32 = SkTileMode::kLastTileMode as i32 + 1;

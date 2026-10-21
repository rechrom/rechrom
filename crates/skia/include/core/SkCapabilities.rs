// Copyright 2022 Google LLC. BSD-style license.
use crate::include::sksl::SkSLVersion::Version;
// Source: include/core/SkCapabilities.h:19-19
// Source: include/core/SkCapabilities.h:32-32
// Native retained singleton is represented by the existing Arc CPU capability owner.
pub struct SkCapabilities {
    pub(crate) fSkSLVersion: Version,
}
impl SkCapabilities {
    // Source: include/core/SkCapabilities.h:28-28
    // Protected native default construction initializes its real declared member.
    pub(crate) const fn new() -> Self {
        Self {
            fSkSLVersion: Version::k100,
        }
    }

    // Source: include/core/SkCapabilities.h:21-21
    pub fn RasterBackend() -> std::sync::Arc<Self> {
        crate::src::core::SkCapabilities::RasterBackend()
    }
    // Source: include/core/SkCapabilities.h:23-23
    pub fn skslVersion(&self) -> Version {
        self.fSkSLVersion
    }
}

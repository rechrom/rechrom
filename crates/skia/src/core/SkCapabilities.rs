// Copyright 2022 Google LLC. BSD-style license.
pub use crate::include::core::SkCapabilities::SkCapabilities;
// Source: src/core/SkCapabilities.cpp:12-20
pub fn RasterBackend() -> std::sync::Arc<SkCapabilities> {
    static CAPS: std::sync::OnceLock<std::sync::Arc<SkCapabilities>> = std::sync::OnceLock::new();
    CAPS.get_or_init(|| {
        std::sync::Arc::new(SkCapabilities {
            fSkSLVersion: crate::include::sksl::SkSLVersion::Version::k100,
        })
    })
    .clone()
}

// Source: src/core/SkCapabilities.cpp:22-24
impl SkCapabilities {
    pub(crate) fn initSkCaps(&mut self, shaderCaps: &crate::src::sksl::SkSLUtil::ShaderCaps) {
        self.fSkSLVersion = shaderCaps.supportedSkSLVerion();
    }
}

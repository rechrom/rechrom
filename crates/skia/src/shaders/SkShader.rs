// Copyright 2006 The Android Open Source Project. BSD-style license.
// Rust trait objects and Arc express the source's virtual dispatch and retained sk_sp ownership.
// Every shader must implement the full CPU base contract; appendStages has no fallback implementation.

// Source: include/core/SkShader.h:36-36
// Native parent provides the virtual flattening/shader protocol, not a
// universal Send/Sync proof for every retained dynamic image/decoder graph.
// Cross-thread users require actually qualified complete objects explicitly.
pub trait SkShader: crate::src::shaders::SkShaderBase::SkShaderBase {
    // Source: include/core/SkShader.h:42-42
    fn isOpaque(&self) -> bool;
    // Source: include/core/SkShader.h:48-48
    // Source: src/shaders/SkShader.cpp:22-23
    fn isAImage(
        &self,
        localMatrix: Option<&mut crate::src::core::SkMatrix::SkMatrix>,
        xy: Option<&mut [crate::include::core::SkTileMode::SkTileMode; 2]>,
    ) -> Option<&dyn crate::src::image::SkImage::SkImage> {
        self.onIsAImage(localMatrix, xy)
    }
    // Source: include/core/SkShader.h:50-51
    fn is_a_image(&self) -> bool {
        self.isAImage(None, None).is_some()
    }
}

impl dyn SkShader {
    // Source: src/shaders/SkShader.cpp:26-27
    // Source: src/shaders/SkShader.cpp:29-37
    // Source: src/shaders/SkShader.cpp:40-40
    // Source: include/core/SkShader.h:61-61
    pub fn makeWithLocalMatrix(
        self: &std::sync::Arc<Self>,
        localMatrix: &crate::src::core::SkMatrix::SkMatrix,
    ) -> std::sync::Arc<dyn SkShader> {
        let mut other = crate::src::core::SkMatrix::SkMatrix::default();
        let proxy = self.makeAsALocalMatrixShader(Some(&mut other));
        let (base, matrix) = if let Some(proxy) = proxy {
            (
                proxy,
                crate::src::shaders::SkShaderBase::ConcatLocalMatrices(localMatrix, &other),
            )
        } else {
            (self.clone(), localMatrix.clone())
        };
        std::sync::Arc::new(
            crate::src::shaders::SkLocalMatrixShader::SkLocalMatrixShader::new(base, &matrix),
        )
    }
    // Source: src/shaders/SkShader.cpp:43-44
    // Source: include/core/SkShader.h:67-67
    pub fn makeWithColorFilter(
        self: &std::sync::Arc<Self>,
        filter: Option<std::sync::Arc<dyn crate::src::core::SkColorFilter::SkColorFilter>>,
    ) -> Option<std::sync::Arc<dyn SkShader>> {
        crate::src::shaders::SkColorFilterShader::SkColorFilterShader::Make(
            Some(self.clone()),
            1.0,
            filter,
        )
    }
    // Source: src/shaders/SkShader.cpp:47-50
    // Source: include/core/SkShader.h:88-89
    pub fn makeWithWorkingColorSpace(
        self: &std::sync::Arc<Self>,
        inputCS: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
        outputCS: Option<std::sync::Arc<crate::src::core::SkColorSpace::SkColorSpace>>,
    ) -> Option<std::sync::Arc<dyn SkShader>> {
        crate::src::shaders::SkWorkingColorSpaceShader::SkWorkingColorSpaceShader::Make(
            Some(self.clone()),
            inputCS,
            outputCS,
            false,
        )
    }
}

// Source: include/core/SkShader.h:92-95
// Native private empty base construction/inheritance is the existing trait
// composition. Concrete factories retain the original owner; no separate base
// instance or provider is introduced. This adaptation adds zero body credit.

// Public native namespace declarations forward to the already translated real
// factories. Rust overload suffixes preserve the two Color and Blend signatures.
// Source: include/core/SkShader.h:99-99
pub use super::SkEmptyShader::Empty;
// Source: include/core/SkShader.h:100-101
pub use super::SkColorShader::{Color, Color4f};
// Source: include/core/SkShader.h:102-103
pub use super::SkBlendShader::{Blend, BlendWithBlender};
// Source: include/core/SkShader.h:104-104
pub use super::SkCoordClampShader::CoordClamp;
// Source: include/core/SkShader.h:109-112
pub use super::SkImageShader::Image;
// Source: include/core/SkShader.h:117-120
pub use super::SkImageShader::RawImage;

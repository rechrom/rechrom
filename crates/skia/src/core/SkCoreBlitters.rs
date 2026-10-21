// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBlitter::SkBlitterStorage;
use crate::src::core::SkPixmap::SkPixmap;
// Source: src/core/SkCoreBlitters.h:32-38
// Rust composition carries the original base allocation owner and Pixmap
// descriptor. Pixel backing storage remains owned by the original caller.
pub struct SkRasterBlitter {
    pub(crate) fDevice: SkPixmap,
    pub(crate) base: SkBlitterStorage,
}
impl SkRasterBlitter {
    /// Safety: writable pixel storage must outlive all blitter calls.
    pub unsafe fn new(device: &SkPixmap) -> Self {
        Self {
            fDevice: device.clone(),
            base: SkBlitterStorage::new(),
        }
    }
}
// Source: src/core/SkCoreBlitters.h:40-54
// Actual native base storage, using the canonical legacy Context trait. Context
// storage and object destruction belong to the caller's arena; this pointer does
// not acquire ownership. Its shader borrow is never erased to a static lifetime.
// The native retained shader drops before the raster base, as in C++ teardown.
pub struct SkShaderBlitter<'shader> {
    pub(crate) fShader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    pub(crate) fShaderContext:
        core::ptr::NonNull<dyn crate::src::shaders::SkShaderBase::Context + 'shader>,
    pub(crate) raster: SkRasterBlitter,
}

// Source: src/core/SkCoreBlitters.h:58-73
// The concrete mask override and trait forwarders live in SkBlitter_ARGB32;
// they depend on the real Paint and mask dispatch owners.
pub struct SkARGB32_Blitter {
    // Source: src/core/SkCoreBlitters.h:70-72
    // Adapted declaration order puts the same native raster parent last.
    pub(crate) fSrcA: u8,
    pub(crate) fPMColor: u32,
    pub(crate) fColor: u32,
    pub(crate) raster: SkRasterBlitter,
}
// Source: src/core/SkCoreBlitters.h:75-86
pub struct SkARGB32_Opaque_Blitter {
    pub(crate) argb: SkARGB32_Blitter,
}
impl SkARGB32_Opaque_Blitter {
    /// Safety: the destination backing store must outlive all blitter calls.
    pub unsafe fn new(device: &SkPixmap, paint: &crate::src::core::SkPaint::SkPaint) -> Self {
        let argb = SkARGB32_Blitter::new(device, paint);
        crate::SkASSERT!(@source b"src/core/SkCoreBlitters.h\0".as_ptr(), 79; @condition b"paint.getAlpha() == 0xFF\0".as_ptr(); paint.getAlpha() == 255);
        Self { argb }
    }
}
// Source: src/core/SkCoreBlitters.h:88-98
pub struct SkARGB32_Black_Blitter {
    pub(crate) opaque: SkARGB32_Opaque_Blitter,
}
impl SkARGB32_Black_Blitter {
    /// Safety: the destination backing store must outlive all blitter calls.
    pub unsafe fn new(device: &SkPixmap, paint: &crate::src::core::SkPaint::SkPaint) -> Self {
        let opaque = SkARGB32_Opaque_Blitter::new(device, paint);
        crate::SkASSERT!(@source b"src/core/SkCoreBlitters.h\0".as_ptr(), 92; @condition b"paint.getColor() == SK_ColorBLACK\0".as_ptr(); paint.getColor() == crate::src::core::SkColor::SK_ColorBLACK);
        Self { opaque }
    }
}

// Source: src/core/SkCoreBlitters.h:99-115
// SkAutoMalloc supplies the real uninitialized, aligned malloc/free ownership.
// It drops the scratch span before the native shader base, matching the explicit
// derived destructor's sk_free followed by base destruction in C++.
pub struct SkARGB32_Shader_Blitter<'shader> {
    pub(crate) fBuffer: crate::src::core::SkAutoMalloc::SkAutoMalloc,
    pub(crate) fProc32: crate::src::core::SkBlitRow::Proc32,
    pub(crate) fProc32Blend: crate::src::core::SkBlitRow::Proc32,
    pub(crate) fShadeDirectlyIntoDevice: bool,
    pub(crate) shader: SkShaderBlitter<'shader>,
}

// Source: src/core/SkCoreBlitters.h:119-125
// Rust reexports the actual factory implementation; its explicit MissingStage
// Result preserves partial-stage failure rather than substituting a blitter.
pub use crate::src::core::SkRasterPipelineBlitter::SkCreateRasterPipelineBlitter;
// Source: src/core/SkCoreBlitters.h:127-130
// The native overload receives an explicit suffix in Rust.
pub use crate::src::core::SkRasterPipelineBlitter::SkCreateRasterPipelineBlitter_from_pipeline;

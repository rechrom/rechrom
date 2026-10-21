// Copyright 2017 Google LLC. BSD-style license.
use crate::src::core::SkBlitter::SkBlitter;
// Source: src/core/SkAutoBlitterChoose.h:28-28
pub type SkBlitterSizedArena = crate::src::core::SkArenaAlloc::SkSTArenaAlloc<2736>;
// Source: src/core/SkAutoBlitterChoose.h:30-41
// Rust adaptation: native choosing constructor/operator-> become new followed
// by choose(Pin<&mut Self>)/get. Construction must finish before pinning: a
// returned-by-value self must never retain pointers into its pre-move arena.
impl<'arena> SkAutoBlitterChoose<'arena> {
    // Source: src/core/SkAutoBlitterChoose.h:32-32
    pub fn new() -> Self {
        Self {
            fBlitter: None,
            fAlloc: SkBlitterSizedArena::new(0),
        }
    }
    // Source: src/core/SkAutoBlitterChoose.h:42-42
    pub fn get(&self) -> Option<core::ptr::NonNull<dyn SkBlitter + 'arena>> {
        self.fBlitter
    }
    // Source: src/core/SkAutoBlitterChoose.h:44-59
    /// Safety: uphold Draw's pixel/context lease and chooser contracts. The
    /// returned blitter belongs to this pinned inline arena; neither this owner
    /// nor its arena may be reset, moved, or destroyed before the pointer's use
    /// ends. Access to the returned mutable blitter must be exclusive.
    /// Image-shader source backing must also remain disjoint from every
    /// destination span; retaining the shader does not establish that lease.
    /// The pinned owner borrow lasts for the arena object lifetime. Only fAlloc
    /// is projected, without moving/replacing it, and the callback result is
    /// returned as a raw pointer with that same lifetime.
    pub unsafe fn choose(
        self: core::pin::Pin<&'arena mut Self>,
        draw: &crate::src::core::SkDraw::Draw<'_>,
        ctm: Option<&crate::src::core::SkMatrix::SkMatrix>,
        paint: &crate::src::core::SkPaint::SkPaint,
        devBounds: &crate::src::core::SkRect::SkRect,
        drawCoverage: crate::src::core::SkDrawTypes::SkDrawCoverage,
    ) -> Result<Option<core::ptr::NonNull<dyn SkBlitter + 'arena>>, crate::RasterError> {
        let this = self.get_unchecked_mut();
        crate::SkASSERT!(@source b"src/core/SkAutoBlitterChoose.h\0".as_ptr(), 49; @condition b"!fBlitter\0".as_ptr(); this.fBlitter.is_none());
        let props = crate::src::core::SkSurfacePriv::SkSurfacePropsCopyOrDefault(draw.fProps);
        let alloc = core::pin::Pin::new_unchecked(&mut this.fAlloc).pinned_arena();
        let chosen = (draw.fBlitterChooser)(
            &draw.fDst,
            ctm.unwrap_or_else(|| draw.fCTM.expect("native Draw requires fCTM")),
            paint,
            alloc,
            drawCoverage,
            draw.fRC.expect("native Draw requires fRC").clip_shader(),
            &props,
            devBounds,
        )?;
        // Store only a successful callback result. The raw trait-object
        // lifetime is already the actual arena borrow; no erasure is required.
        this.fBlitter = chosen;
        Ok(this.fBlitter)
    }
}
// Source: src/core/SkAutoBlitterChoose.h:63-66
// Allocation and destruction belong to the actual inline arena. The arena is
// !Unpin: choose projects Pin<&mut Self> into this field before allocation.
// The native SurfacePriv helper remains a real dependency, not a fake default.
pub struct SkAutoBlitterChoose<'arena> {
    fBlitter: Option<core::ptr::NonNull<dyn SkBlitter + 'arena>>,
    fAlloc: SkBlitterSizedArena,
}

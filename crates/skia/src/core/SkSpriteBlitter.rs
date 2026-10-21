// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBlitter::SkBlitterStorage;
use crate::src::core::SkPixmap::SkPixmap;
// Source: src/core/SkSpriteBlitter.h:24-24
// Rust externalizes the native virtual setup contract; its implementations keep
// the same borrowed paint/pixmap lifetime requirements as the native method.
pub trait SkSpriteBlitterInterface: crate::src::core::SkBlitter::SkBlitter {
    // Source: src/core/SkSpriteBlitter.h:33-33
    // Native Sprite's rectangle slot is pure virtual, even though the general
    // Blitter base has a default rectangle implementation. Require the actual
    // Sprite override; do not inherit that broader base default here.
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32);

    /// Safety: paint/source/destination storage must remain live throughout use;
    /// destination access must be exclusive and all later coordinates in bounds.
    unsafe fn setup(
        self: core::pin::Pin<&mut Self>,
        dst: &SkPixmap,
        left: i32,
        top: i32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> bool;
}

// Source: src/core/SkSpriteBlitter.h:37-41
// Composition retains the original borrowed paint pointer; its lifetime is not
// replaced by an owned paint clone. All access through it requires unsafe code.
pub struct SkSpriteBlitter {
    // Adapted private declaration order: source Pixmap dies before destination,
    // then the same embedded Blitter base. Constructor expression order is kept.
    pub(crate) fPaint: *const crate::src::core::SkPaint::SkPaint,
    pub(crate) fTop: i32,
    pub(crate) fLeft: i32,
    pub(crate) fSource: SkPixmap,
    pub(crate) fDst: SkPixmap,
    pub(crate) base: SkBlitterStorage,
}

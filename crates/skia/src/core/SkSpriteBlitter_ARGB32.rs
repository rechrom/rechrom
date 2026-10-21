// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBlitRow::{Proc32, SkBlitRow};
use crate::src::core::SkPixmap::SkPixmap;
use crate::src::core::SkSpriteBlitter::SkSpriteBlitter;
// Source: src/core/SkSpriteBlitter_ARGB32.cpp:22-60
pub struct Sprite_D32_S32 {
    // Source: src/core/SkSpriteBlitter_ARGB32.cpp:56-57
    // Adapted field order: implicit derived-member destruction precedes parent.
    fAlpha: u32,
    fProc32: Proc32,
    sprite: SkSpriteBlitter,
}
impl Sprite_D32_S32 {
    // Source: src/core/SkSpriteBlitter_ARGB32.cpp:24-37
    /// Safety: source pixel storage must remain live/readable for sprite calls.
    pub unsafe fn new(src: &SkPixmap, alpha: u32) -> Self {
        crate::SkASSERT!(@source b"src/core/SkSpriteBlitter_ARGB32.cpp\0".as_ptr(), 25; @condition b"src.colorType() == kN32_SkColorType\0".as_ptr(); src.colorType() == crate::include::core::SkColorType::kN32_SkColorType);
        let mut flags32 = 0;
        if alpha != 255 {
            flags32 |= SkBlitRow::kGlobalAlpha_Flag32;
        }
        if !src.isOpaque() {
            flags32 |= SkBlitRow::kSrcPixelAlpha_Flag32;
        }
        Self {
            sprite: SkSpriteBlitter::new(src),
            fProc32: SkBlitRow::Factory32(flags32),
            fAlpha: alpha,
        }
    }
    // Source: src/core/SkSpriteBlitter_ARGB32.cpp:39-53
    /// Safety: setup must have supplied valid destination and sprite offsets;
    /// addressed destination/source rows must be disjoint initialized storage.
    pub unsafe fn blitRect(
        self: core::pin::Pin<&mut Self>,
        x: i32,
        y: i32,
        width: i32,
        mut height: i32,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::SkASSERT!(@source b"src/core/SkSpriteBlitter_ARGB32.cpp\0".as_ptr(), 40; @condition b"width > 0 && height > 0\0".as_ptr(); width > 0 && height > 0);
        let mut dst = this.sprite.fDst.writable_addr32(x, y);
        let mut src = this
            .sprite
            .fSource
            .addr32_xy(x - this.sprite.fLeft, y - this.sprite.fTop);
        let dstRB = this.sprite.fDst.rowBytes();
        let srcRB = this.sprite.fSource.rowBytes();
        let proc = this.fProc32;
        let alpha = this.fAlpha;
        loop {
            proc(
                core::slice::from_raw_parts_mut(dst, width as usize),
                core::slice::from_raw_parts(src, width as usize),
                width as usize,
                alpha,
            );
            dst = dst.cast::<u8>().wrapping_add(dstRB).cast::<u32>();
            src = src.cast::<u8>().wrapping_add(srcRB).cast::<u32>();
            height -= 1;
            if height == 0 {
                break;
            }
        }
    }
}

// Rust inheritance forwarding only; native rectangle/setup algorithms remain
// in their original source-mapped implementations.
unsafe impl crate::src::core::SkBlitter::SkBlitter for Sprite_D32_S32 {
    fn allocBlitMemory(self: core::pin::Pin<&mut Self>, sz: usize) -> *mut u8 {
        let this = unsafe { self.get_unchecked_mut() };
        this.sprite.base.allocBlitMemory(sz)
    }
    unsafe fn blitH(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::src::core::SkBlitter_Sprite::sprite_blit_h(
            core::pin::Pin::new_unchecked(&mut *this),
            x,
            y,
            width,
        )
    }
    unsafe fn blitV(self: core::pin::Pin<&mut Self>, x: i32, y: i32, height: i32, alpha: u8) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::src::core::SkBlitter_Sprite::sprite_blit_v(
            core::pin::Pin::new_unchecked(&mut *this),
            x,
            y,
            height,
            alpha,
        )
    }
    unsafe fn blitMask(
        self: core::pin::Pin<&mut Self>,
        mask: &crate::src::core::SkMask::SkMask,
        clip: &crate::src::core::SkRect::SkIRect,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::src::core::SkBlitter_Sprite::sprite_blit_mask(
            core::pin::Pin::new_unchecked(&mut *this),
            mask,
            clip,
        )
    }
    unsafe fn blitAntiH(
        self: core::pin::Pin<&mut Self>,
        _: i32,
        _: i32,
        _: &mut [u8],
        _: &mut [i16],
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        crate::src::core::SkBlitter_Sprite::sprite_blit_anti_h()
    }
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        Sprite_D32_S32::blitRect(
            core::pin::Pin::new_unchecked(&mut *this),
            x,
            y,
            width,
            height,
        )
    }
}
impl crate::src::core::SkSpriteBlitter::SkSpriteBlitterInterface for Sprite_D32_S32 {
    // Source: src/core/SkSpriteBlitter.h:33-33
    // Same real complete object's existing rectangle override, without another
    // pixel loop, owner, base-default fallback or erased-vtable cast.
    unsafe fn blitRect(self: core::pin::Pin<&mut Self>, x: i32, y: i32, width: i32, height: i32) {
        <Self as crate::src::core::SkBlitter::SkBlitter>::blitRect(self, x, y, width, height);
    }

    unsafe fn setup(
        self: core::pin::Pin<&mut Self>,
        dst: &SkPixmap,
        left: i32,
        top: i32,
        paint: &crate::src::core::SkPaint::SkPaint,
    ) -> bool {
        self.get_unchecked_mut().sprite.setup(dst, left, top, paint)
    }
}

// Source: src/core/SkSpriteBlitter_ARGB32.cpp:64-79
impl SkSpriteBlitter {
    /// Safety: allocator and source storage must remain live for all calls on
    /// the returned arena-owned sprite. setup must precede pixel operations.
    /// Rust's concrete return preserves the selected subclass until the native
    /// sprite virtual dispatch hierarchy is represented by its shared adapter.
    pub unsafe fn ChooseL32(
        source: &SkPixmap,
        paint: &crate::src::core::SkPaint::SkPaint,
        allocator: &mut crate::src::core::SkArenaAlloc::SkArenaAlloc,
    ) -> Option<core::ptr::NonNull<Sprite_D32_S32>> {
        if paint.getColorFilter().is_some() {
            return None;
        }
        if paint.getMaskFilter().is_some() {
            return None;
        }
        if source.colorType() == crate::include::core::SkColorType::kN32_SkColorType
            && paint.isSrcOver()
        {
            return core::ptr::NonNull::new(
                allocator.make(Sprite_D32_S32::new(source, paint.getAlpha() as u32)),
            );
        }
        None
    }
}

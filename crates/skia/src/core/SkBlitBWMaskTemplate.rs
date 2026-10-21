// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::src::core::{SkMask::SkMask, SkPixmap::SkPixmap, SkRect::SkIRect};
// Source: src/core/SkBlitBWMaskTemplate.h:19-19
pub const fn ClearLow3Bits(x: i32) -> u32 {
    (x as u32) >> 3 << 3
}
// Source: src/core/SkBlitBWMaskTemplate.h:30-125
// The C++ repeated-include macros become one monomorphized T/callback driver.
// T retains the selected destination pixel width; no fabricated pixel owner.
/// Safety: the mask rows, destination pixels and callback accesses must be live,
/// initialized and valid. Unset mask bits must not be accessed by the callback;
/// aligned edge addresses may precede the clipped span. Full-width native masks
/// must have zero padding bits when those bits exceed destination width.
pub unsafe fn blit_bw_mask<T>(
    dstPixmap: &SkPixmap,
    srcMask: &SkMask,
    clip: &SkIRect,
    mut blit8: impl FnMut(u8, *mut T),
) {
    crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 32; @condition b"clip.fRight <= srcMask.fBounds.fRight\0".as_ptr(); clip.fRight <= srcMask.fBounds.fRight);
    let cx = clip.fLeft;
    let cy = clip.fTop;
    let maskLeft = srcMask.fBounds.fLeft;
    let mask_rowBytes = srcMask.fRowBytes;
    let bitmap_rowBytes = dstPixmap.rowBytes();
    let mut height = clip.height() as u32;
    crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 41; @condition b"mask_rowBytes != 0\0".as_ptr(); mask_rowBytes != 0);
    crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 42; @condition b"bitmap_rowBytes != 0\0".as_ptr(); bitmap_rowBytes != 0);
    crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 43; @condition b"height != 0\0".as_ptr(); height != 0);
    debug_assert!(core::mem::size_of::<T>() == dstPixmap.info().bytesPerPixel() as usize);
    let mut bits = srcMask.getAddr1(cx, cy);
    let mut device = dstPixmap.writable_addr_xy(cx, cy).cast::<T>();
    if cx == maskLeft && clip.fRight == srcMask.fBounds.fRight {
        loop {
            let mut dst = device;
            let mut rb = mask_rowBytes;
            loop {
                let mask = *bits;
                bits = bits.add(1);
                blit8(mask, dst);
                dst = dst.wrapping_add(8);
                rb -= 1;
                if rb == 0 {
                    break;
                }
            }
            device = device
                .cast::<u8>()
                .wrapping_add(bitmap_rowBytes)
                .cast::<T>();
            height -= 1;
            if height == 0 {
                break;
            }
        }
    } else {
        let left_edge = cx - maskLeft;
        crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 64; @condition b"left_edge >= 0\0".as_ptr(); left_edge >= 0);
        let rite_edge = clip.fRight - maskLeft;
        crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 66; @condition b"rite_edge > left_edge\0".as_ptr(); rite_edge > left_edge);
        let mut left_mask = 255i32 >> (left_edge & 7);
        let mut rite_mask = 255i32 << (8 - (rite_edge & 7));
        rite_mask &= 255;
        let mut full_runs = (rite_edge >> 3) - ((left_edge + 7) >> 3);
        if rite_mask == 0 {
            crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 76; @condition b"full_runs >= 0\0".as_ptr(); full_runs >= 0);
            full_runs -= 1;
            rite_mask = 255;
        }
        if left_mask == 255 {
            full_runs -= 1;
        }
        device = device.wrapping_sub((left_edge & 7) as usize);
        if full_runs < 0 {
            left_mask &= rite_mask;
            crate::SkASSERT!(@source b"src/core/SkBlitBWMaskTemplate.h\0".as_ptr(), 90; @condition b"left_mask != 0\0".as_ptr(); left_mask != 0);
            loop {
                let mask = *bits & left_mask as u8;
                blit8(mask, device);
                bits = bits.wrapping_add(mask_rowBytes as usize);
                device = device
                    .cast::<u8>()
                    .wrapping_add(bitmap_rowBytes)
                    .cast::<T>();
                height -= 1;
                if height == 0 {
                    break;
                }
            }
        } else {
            loop {
                let mut runs = full_runs;
                let mut dst = device;
                let mut b = bits;
                let mut mask = *b & left_mask as u8;
                b = b.add(1);
                blit8(mask, dst);
                dst = dst.wrapping_add(8);
                loop {
                    runs -= 1;
                    if runs < 0 {
                        break;
                    }
                    mask = *b;
                    b = b.add(1);
                    blit8(mask, dst);
                    dst = dst.wrapping_add(8);
                }
                mask = *b & rite_mask as u8;
                blit8(mask, dst);
                bits = bits.wrapping_add(mask_rowBytes as usize);
                device = device
                    .cast::<u8>()
                    .wrapping_add(bitmap_rowBytes)
                    .cast::<T>();
                height -= 1;
                if height == 0 {
                    break;
                }
            }
        }
    }
}

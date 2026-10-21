// Copyright 2015 Google Inc. BSD-style license.
use crate::src::core::Sk4px::Sk4px;
use crate::src::core::SkColor::{SK_ColorBLACK, SkColorGetA, SkPreMultiplyColor};
#[cfg(target_arch = "aarch64")]
pub mod neon {
    use super::*;
    use crate::src::core::SkColorPriv::{
        SkAlpha255To256, SkAlphaMulQ, SkGetPackedA32, SkGetPackedB32, SkGetPackedG32,
        SkGetPackedR32, SK_BGRA_A32_SHIFT,
    };
    use std::arch::aarch64::*;
    // Source: src/opts/SkBlitMask_opts.h:24-27
    const NEON_A: usize = crate::src::core::SkColorPriv::SK_BGRA_A32_SHIFT as usize / 8;
    const NEON_R: usize = crate::src::core::SkColorPriv::SK_BGRA_R32_SHIFT as usize / 8;
    const NEON_G: usize = crate::src::core::SkColorPriv::SK_BGRA_G32_SHIFT as usize / 8;
    const NEON_B: usize = crate::src::core::SkColorPriv::SK_BGRA_B32_SHIFT as usize / 8;
    // Source: src/opts/SkBlitMask_opts.h:29-31
    unsafe fn SkAlpha255To256_neon8(alpha: uint8x8_t) -> uint16x8_t {
        vaddw_u8(vdupq_n_u16(1), alpha)
    }
    // Source: src/opts/SkBlitMask_opts.h:33-35
    unsafe fn SkAlphaMul_neon8(color: uint8x8_t, scale: uint16x8_t) -> uint8x8_t {
        vshrn_n_u16::<8>(vmulq_u16(vmovl_u8(color), scale))
    }
    // Source: src/opts/SkBlitMask_opts.h:37-46
    unsafe fn SkAlphaMulQ_neon8(color: uint8x8x4_t, scale: uint16x8_t) -> uint8x8x4_t {
        uint8x8x4_t(
            SkAlphaMul_neon8(color.0, scale),
            SkAlphaMul_neon8(color.1, scale),
            SkAlphaMul_neon8(color.2, scale),
            SkAlphaMul_neon8(color.3, scale),
        )
    }
    // Source: src/opts/SkBlitMask_opts.h:49-119
    /// Safety: valid disjoint source-mask/destination rows; height>0, width>=0.
    pub unsafe fn blit_mask_d32_a8_neon<const IS_TRANSLUCENT: bool>(
        dst: *mut u32,
        mut dstRB: usize,
        maskPtr: *const u8,
        maskRB: usize,
        color: u32,
        width: i32,
        mut height: i32,
    ) {
        let pmc = SkPreMultiplyColor(color);
        let colorAlpha = SkGetPackedA32(pmc);
        let mut device = dst;
        let mut mask = maskPtr;
        let mut vpmc = core::mem::MaybeUninit::<[uint8x8_t; 4]>::uninit();
        let mask_adjust = (maskRB as isize).wrapping_sub(width as isize);
        dstRB = dstRB.wrapping_sub((width as usize) << 2);
        if width >= 8 {
            let mut v = [vdup_n_u8(0); 4];
            v[NEON_A] = vdup_n_u8(colorAlpha as u8);
            v[NEON_R] = vdup_n_u8(SkGetPackedR32(pmc) as u8);
            v[NEON_G] = vdup_n_u8(SkGetPackedG32(pmc) as u8);
            v[NEON_B] = vdup_n_u8(SkGetPackedB32(pmc) as u8);
            vpmc.write(v);
        }
        loop {
            let mut w = width;
            while w >= 8 {
                let vmask = vld1_u8(mask);
                let vmask256 = SkAlpha255To256_neon8(vmask);
                let vpmc = vpmc.assume_init_ref();
                let vscale = if IS_TRANSLUCENT {
                    vsubw_u8(vdupq_n_u16(256), SkAlphaMul_neon8(vpmc[NEON_A], vmask256))
                } else {
                    vsubw_u8(vdupq_n_u16(256), vmask)
                };
                let vdev = vld4_u8(device.cast());
                let mut vdev = [vdev.0, vdev.1, vdev.2, vdev.3];
                vdev[NEON_A] = vadd_u8(
                    SkAlphaMul_neon8(vpmc[NEON_A], vmask256),
                    SkAlphaMul_neon8(vdev[NEON_A], vscale),
                );
                vdev[NEON_R] = vadd_u8(
                    SkAlphaMul_neon8(vpmc[NEON_R], vmask256),
                    SkAlphaMul_neon8(vdev[NEON_R], vscale),
                );
                vdev[NEON_G] = vadd_u8(
                    SkAlphaMul_neon8(vpmc[NEON_G], vmask256),
                    SkAlphaMul_neon8(vdev[NEON_G], vscale),
                );
                vdev[NEON_B] = vadd_u8(
                    SkAlphaMul_neon8(vpmc[NEON_B], vmask256),
                    SkAlphaMul_neon8(vdev[NEON_B], vscale),
                );
                vst4_u8(
                    device.cast(),
                    uint8x8x4_t(vdev[0], vdev[1], vdev[2], vdev[3]),
                );
                mask = mask.add(8);
                device = device.add(8);
                w -= 8;
            }
            loop {
                let old = w;
                w -= 1;
                if old <= 0 {
                    break;
                }
                let vmask = *mask as u32;
                mask = mask.add(1);
                let vmask256 = SkAlpha255To256(vmask);
                let vscale = if IS_TRANSLUCENT {
                    256 - SkAlphaMulQ(colorAlpha, vmask256)
                } else {
                    256 - vmask
                };
                *device = SkAlphaMulQ(pmc, vmask256) + SkAlphaMulQ(*device, vscale);
                device = device.add(1);
            }
            device = device.cast::<u8>().wrapping_add(dstRB).cast();
            mask = mask.wrapping_offset(mask_adjust);
            height -= 1;
            if height == 0 {
                break;
            }
        }
    }
    // Source: src/opts/SkBlitMask_opts.h:121-125
    pub unsafe fn blit_mask_d32_a8_general(
        dst: *mut u32,
        dstRB: usize,
        mask: *const u8,
        maskRB: usize,
        color: u32,
        w: i32,
        h: i32,
    ) {
        blit_mask_d32_a8_neon::<true>(dst, dstRB, mask, maskRB, color, w, h);
    }
    // Source: src/opts/SkBlitMask_opts.h:128-132
    pub unsafe fn blit_mask_d32_a8_opaque(
        dst: *mut u32,
        dstRB: usize,
        mask: *const u8,
        maskRB: usize,
        color: u32,
        w: i32,
        h: i32,
    ) {
        blit_mask_d32_a8_neon::<false>(dst, dstRB, mask, maskRB, color, w, h);
    }
    // Source: src/opts/SkBlitMask_opts.h:135-172
    /// Safety: valid disjoint source-mask/destination rows; height>0, width>=0.
    pub unsafe fn blit_mask_d32_a8_black(
        dst: *mut u32,
        mut dstRB: usize,
        maskPtr: *const u8,
        maskRB: usize,
        width: i32,
        mut height: i32,
    ) {
        let mut device = dst;
        let mut mask = maskPtr;
        let mask_adjust = (maskRB as isize).wrapping_sub(width as isize);
        dstRB = dstRB.wrapping_sub((width as usize) << 2);
        loop {
            let mut w = width;
            while w >= 8 {
                let vmask = vld1_u8(mask);
                let vscale = vsubw_u8(vdupq_n_u16(256), vmask);
                let vdevice = vld4_u8(device.cast());
                let vdevice = SkAlphaMulQ_neon8(vdevice, vscale);
                let mut vdevice = [vdevice.0, vdevice.1, vdevice.2, vdevice.3];
                vdevice[NEON_A] = vadd_u8(vdevice[NEON_A], vmask);
                vst4_u8(
                    device.cast(),
                    uint8x8x4_t(vdevice[0], vdevice[1], vdevice[2], vdevice[3]),
                );
                mask = mask.add(8);
                device = device.add(8);
                w -= 8;
            }
            loop {
                let old = w;
                w -= 1;
                if old <= 0 {
                    break;
                }
                let vmask = *mask as u32;
                mask = mask.add(1);
                let vscale = 256 - vmask;
                *device = SkAlphaMulQ(*device, vscale) + (vmask << SK_BGRA_A32_SHIFT);
                device = device.add(1);
            }
            device = device.cast::<u8>().wrapping_add(dstRB).cast();
            mask = mask.wrapping_offset(mask_adjust);
            height -= 1;
            if height == 0 {
                break;
            }
        }
    }
}

pub mod baseline {
    use super::*;
    // Source: src/opts/SkBlitMask_opts.h:357-373
    /// Safety: every addressed destination/mask pixel must be valid and initialized;
    /// writable destination rows must not overlap mask storage. maskRB may be zero.
    pub unsafe fn blit_mask_d32_a8_general(
        mut dst: *mut u32,
        dstRB: usize,
        mut mask: *const u8,
        maskRB: usize,
        color: u32,
        w: i32,
        mut h: i32,
    ) {
        let s = Sk4px::DupPMColor(SkPreMultiplyColor(color));
        let kernel = |d: Sk4px, aa: Sk4px| {
            let left = s.approxMulDiv255(&aa);
            let right = d.approxMulDiv255(&left.alphas().inv());
            left + right
        };
        loop {
            let old = h;
            h -= 1;
            if old <= 0 {
                break;
            }
            Sk4px::MapDstAlpha(w, dst, mask, &kernel);
            dst = dst.wrapping_add(dstRB / core::mem::size_of::<u32>());
            mask = mask.wrapping_add(maskRB);
        }
    }
    // Source: src/opts/SkBlitMask_opts.h:376-393
    /// Safety: same storage contract as the general branch; color must be opaque.
    pub unsafe fn blit_mask_d32_a8_opaque(
        mut dst: *mut u32,
        dstRB: usize,
        mut mask: *const u8,
        maskRB: usize,
        color: u32,
        w: i32,
        mut h: i32,
    ) {
        crate::SkASSERT!(@source b"src/opts/SkBlitMask_opts.h\0".as_ptr(),379; @condition b"SkColorGetA(color) == 0xFF\0".as_ptr(); SkColorGetA(color)==255);
        let s = Sk4px::DupPMColor(SkPreMultiplyColor(color));
        let kernel = |d: Sk4px, aa: Sk4px| s.approxMulDiv255(&aa) + d.approxMulDiv255(&aa.inv());
        loop {
            let old = h;
            h -= 1;
            if old <= 0 {
                break;
            }
            Sk4px::MapDstAlpha(w, dst, mask, &kernel);
            dst = dst.wrapping_add(dstRB / core::mem::size_of::<u32>());
            mask = mask.wrapping_add(maskRB);
        }
    }
    // Source: src/opts/SkBlitMask_opts.h:396-413
    /// Safety: same row storage and non-overlap contract as the general branch.
    pub unsafe fn blit_mask_d32_a8_black(
        mut dst: *mut u32,
        dstRB: usize,
        mut mask: *const u8,
        maskRB: usize,
        w: i32,
        mut h: i32,
    ) {
        let kernel = |d: Sk4px, aa: Sk4px| {
            (aa & Sk4px::new([0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]))
                + d.approxMulDiv255(&aa.inv())
        };
        loop {
            let old = h;
            h -= 1;
            if old <= 0 {
                break;
            }
            Sk4px::MapDstAlpha(w, dst, mask, &kernel);
            dst = dst.wrapping_add(dstRB / core::mem::size_of::<u32>());
            mask = mask.wrapping_add(maskRB);
        }
    }
}
// Native build-time branch selection, with baseline kept separately available for verification.
#[cfg(not(target_arch = "aarch64"))]
use baseline::{blit_mask_d32_a8_black, blit_mask_d32_a8_general, blit_mask_d32_a8_opaque};
#[cfg(target_arch = "aarch64")]
use neon::{blit_mask_d32_a8_black, blit_mask_d32_a8_general, blit_mask_d32_a8_opaque};
// Source: src/opts/SkBlitMask_opts.h:416-426
/// Safety: same row storage and non-overlap contract as the general branch.
pub unsafe fn blit_mask_d32_a8(
    dst: *mut u32,
    dstRB: usize,
    mask: *const u8,
    maskRB: usize,
    color: u32,
    w: i32,
    h: i32,
) {
    if color == SK_ColorBLACK {
        blit_mask_d32_a8_black(dst, dstRB, mask, maskRB, w, h);
    } else if SkColorGetA(color) == 255 {
        blit_mask_d32_a8_opaque(dst, dstRB, mask, maskRB, color, w, h);
    } else {
        blit_mask_d32_a8_general(dst, dstRB, mask, maskRB, color, w, h);
    }
}

// Copyright 2006 The Android Open Source Project. BSD-style license.
// Unverified direct translation of the native CPU emboss kernel.
use super::SkEmbossMaskFilter::Light;
use crate::src::core::SkMask::{Format, SkMaskBuilder};
// Source: src/effects/SkEmbossMask.cpp:22-28
fn nonzero_to_one(x: i32) -> i32 {
    (((x | x.wrapping_neg()) as u32) >> 31) as i32
}
// Source: src/effects/SkEmbossMask.cpp:30-37
fn neq_to_one(x: i32, max: i32) -> i32 {
    crate::SkASSERT!(@source b"src/effects/SkEmbossMask.cpp\0".as_ptr(),34; @condition b"x >= 0 && x <= max\0".as_ptr(); x>=0&&x<=max);
    (((x - max) as u32) >> 31) as i32
}
// Source: src/effects/SkEmbossMask.cpp:39-46
fn neq_to_mask(x: i32, max: i32) -> i32 {
    crate::SkASSERT!(@source b"src/effects/SkEmbossMask.cpp\0".as_ptr(),43; @condition b"x >= 0 && x <= max\0".as_ptr(); x>=0&&x<=max);
    (x - max) >> 31
}
// Source: src/effects/SkEmbossMask.cpp:48-51
fn div255(x: u32) -> u32 {
    crate::SkASSERT!(@source b"src/effects/SkEmbossMask.cpp\0".as_ptr(),49; @condition b"x <= (255*255)\0".as_ptr(); x<=255*255);
    x * ((1 << 24) / 255) >> 24
}
// Source: src/effects/SkEmbossMask.h:16-19
// Static-only class projects to this module function; no instantiable class marker.
// Source: src/effects/SkEmbossMask.cpp:55-120
/// Safety: mask owns live writable three-plane 3D storage sized by its bounds
/// and rowBytes; the alpha plane is initialized. Original signed size/fixed
/// arithmetic domains hold; Light direction follows the native normalized input.
pub unsafe fn Emboss(mask: &mut SkMaskBuilder, light: &Light) {
    crate::SkASSERT!(@source b"src/effects/SkEmbossMask.cpp\0".as_ptr(),56; @condition b"mask->fFormat == SkMask::k3D_Format\0".as_ptr(); (mask.fFormat)==(Format::k3D_Format));
    let specular = light.fSpecular as i32;
    let ambient = light.fAmbient as i32;
    let lx = crate::include::private::SkFixed::SkFloatToFixed(light.fDirection[0]);
    let ly = crate::include::private::SkFixed::SkFloatToFixed(light.fDirection[1]);
    let lz = crate::include::private::SkFixed::SkFloatToFixed(light.fDirection[2]);
    let lz_dot_nz = lz * 32;
    let lz_dot8 = lz >> 8;
    let planeSize = mask.computeImageSize();
    let mut alpha = mask.fImage.cast_mut();
    let mut multiply = alpha.add(planeSize);
    let mut additive = multiply.add(planeSize);
    let rowBytes = mask.fRowBytes as i32;
    let maxy = mask.fBounds.height() - 1;
    let maxx = mask.fBounds.width() - 1;
    let mut prev_row = 0;
    for y in 0..=maxy {
        let next_row = neq_to_mask(y, maxy) & rowBytes;
        for x in 0..=maxx {
            let nx = alpha.offset((x + neq_to_one(x, maxx)) as isize).read() as i32
                - alpha.offset((x - nonzero_to_one(x)) as isize).read() as i32;
            let ny = alpha.offset((x + next_row) as isize).read() as i32
                - alpha.offset((x - prev_row) as isize).read() as i32;
            let numer = lx * nx + ly * ny + lz_dot_nz;
            let mut mul = ambient;
            let mut add = 0;
            if numer > 0 {
                let denom = crate::src::core::SkMathPriv::SkSqrt32(nx * nx + ny * ny + 32 * 32);
                let mut dot = numer / denom;
                dot >>= 8;
                mul = (mul + dot).min(255);
                let mut hilite = ((2 * dot - lz_dot8) * lz_dot8) >> 8;
                if hilite > 0 {
                    hilite = hilite.min(255);
                    add = hilite;
                    let mut i = specular >> 4;
                    while i > 0 {
                        add = div255((add * hilite) as u32) as i32;
                        i -= 1;
                    }
                }
            }
            debug_assert!((0..=255).contains(&mul) && (0..=255).contains(&add));
            multiply.offset(x as isize).write(mul as u8);
            additive.offset(x as isize).write(add as u8);
        }
        alpha = alpha.offset(rowBytes as isize);
        multiply = multiply.offset(rowBytes as isize);
        additive = additive.offset(rowBytes as isize);
        prev_row = rowBytes;
    }
}

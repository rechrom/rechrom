// Copyright 2006 The Android Open Source Project. BSD-style license.

// Source: src/core/SkBlurMask.cpp:35-35
// Source: src/core/SkBlurMask.cpp:37-38
// Source: src/core/SkBlurMask.cpp:41-42
const BLUR_SIGMA_SCALE: f32 = 0.57735;
pub fn ConvertRadiusToSigma(radius: f32) -> f32 {
    if radius > 0.0 {
        BLUR_SIGMA_SCALE * radius + 0.5
    } else {
        0.0
    }
}
pub fn ConvertSigmaToRadius(sigma: f32) -> f32 {
    if sigma > 0.5 {
        (sigma - 0.5) / BLUR_SIGMA_SCALE
    } else {
        0.0
    }
}

// Rust trait substitution for the source AlphaIter templates; cursors retain pointer/row semantics.
pub trait MaskAlphaCursor: crate::src::core::SkMaskBlurFilter::AlphaCursor {
    fn advance_row(&mut self, row_bytes: u32);
}
macro_rules! mask_alpha_cursor {
    ($t:ty) => {
        impl MaskAlphaCursor for $t {
            fn advance_row(&mut self, row_bytes: u32) {
                <$t>::advance_row(self, row_bytes);
            }
        }
    };
}
mask_alpha_cursor!(crate::src::core::SkMask::BWAlphaIter);
mask_alpha_cursor!(crate::src::core::SkMask::A8AlphaIter);
mask_alpha_cursor!(crate::src::core::SkMask::ARGB32AlphaIter);
mask_alpha_cursor!(crate::src::core::SkMask::LCD16AlphaIter);

// Source: src/core/SkBlurMask.cpp:46-59
// Source: src/core/SkBlurMask.cpp:61-63
pub unsafe fn merge_src_with_blur<A: MaskAlphaCursor>(
    mut dst: *mut u8,
    mut dst_rb: i32,
    mut src: A,
    src_rb: u32,
    mut blur: *const u8,
    mut blur_rb: i32,
    sw: i32,
    mut sh: i32,
) {
    use crate::src::core::SkColorPriv::{SkAlpha255To256, SkAlphaMul};
    dst_rb -= sw;
    blur_rb -= sw;
    loop {
        sh -= 1;
        if sh < 0 {
            break;
        }
        let mut row_src = src;
        for _ in (0..sw).rev() {
            unsafe {
                *dst = SkAlphaMul(*blur as u32, SkAlpha255To256(row_src.value() as u32)) as u8;
            }
            dst = dst.wrapping_add(1);
            row_src.increment();
            blur = blur.wrapping_add(1);
        }
        dst = dst.wrapping_offset(dst_rb as isize);
        src.advance_row(src_rb);
        blur = blur.wrapping_offset(blur_rb as isize);
    }
}

// Source: src/core/SkBlurMask.cpp:67-79
// Source: src/core/SkBlurMask.cpp:81-82
pub unsafe fn clamp_solid_with_orig<A: MaskAlphaCursor>(
    mut dst: *mut u8,
    dst_row_bytes: i32,
    mut src: A,
    src_row_bytes: u32,
    sw: i32,
    mut sh: i32,
) {
    loop {
        sh -= 1;
        if sh < 0 {
            break;
        }
        let mut row_src = src;
        for _ in (0..sw).rev() {
            let s = unsafe { row_src.value() } as u32;
            let d = unsafe { *dst } as u32;
            // Exact scalar expansion of SkMulDiv255Round from include/private/SkMath.h.
            let prod = s * d + 128;
            unsafe {
                *dst = (s + d - ((prod + (prod >> 8)) >> 8)) as u8;
            }
            dst = dst.wrapping_add(1);
            row_src.increment();
        }
        dst = dst.wrapping_offset((dst_row_bytes - sw) as isize);
        src.advance_row(src_row_bytes);
    }
}

// Source: src/core/SkBlurMask.cpp:86-96
// Source: src/core/SkBlurMask.cpp:98-99
// Source: src/core/SkBlurMask.cpp:101-102
pub unsafe fn clamp_outer_with_orig<A: MaskAlphaCursor>(
    mut dst: *mut u8,
    dst_row_bytes: i32,
    mut src: A,
    src_row_bytes: u32,
    sw: i32,
    mut sh: i32,
) {
    use crate::src::core::SkColorPriv::{SkAlpha255To256, SkAlphaMul};
    loop {
        sh -= 1;
        if sh < 0 {
            break;
        }
        let mut row_src = src;
        for _ in (0..sw).rev() {
            let src_value = unsafe { row_src.value() } as u32;
            if src_value != 0 {
                unsafe {
                    *dst = SkAlphaMul(*dst as u32, SkAlpha255To256(255 - src_value)) as u8;
                }
            }
            dst = dst.wrapping_add(1);
            row_src.increment();
        }
        dst = dst.wrapping_offset((dst_row_bytes - sw) as isize);
        src.advance_row(src_row_bytes);
    }
}

// Source: src/core/SkBlurMask.cpp:107-116
// Source: src/core/SkBlurMask.cpp:118-118
// Source: src/core/SkBlurMask.cpp:121-122
// Source: src/core/SkBlurMask.cpp:125-130
// Source: src/core/SkBlurMask.cpp:134-134
// Source: src/core/SkBlurMask.cpp:136-136
// Source: src/core/SkBlurMask.cpp:138-138
// Source: src/core/SkBlurMask.cpp:140-140
// Source: src/core/SkBlurMask.cpp:142-142
// Source: src/core/SkBlurMask.cpp:146-146
// Source: src/core/SkBlurMask.cpp:149-150
// Source: src/core/SkBlurMask.cpp:153-156
// Source: src/core/SkBlurMask.cpp:158-158
// Source: src/core/SkBlurMask.cpp:161-194
// Source: src/core/SkBlurMask.cpp:196-227
// Source: src/core/SkBlurMask.cpp:229-230
// Source: src/core/SkBlurMask.cpp:232-233
// Source: src/core/SkBlurMask.cpp:235-238
// Source: src/core/SkBlurMask.cpp:240-240
// Source: src/core/SkBlurMask.cpp:242-275
// Source: src/core/SkBlurMask.cpp:277-277
// Source: src/core/SkBlurMask.cpp:280-280
/// Safety: source image bytes and row storage must remain live and readable during filtering.
pub unsafe fn BoxBlur(
    dst: &mut crate::src::core::SkMask::SkMaskBuilder,
    src: &crate::src::core::SkMask::SkMask,
    sigma: f32,
    style: crate::include::core::SkBlurTypes::SkBlurStyle,
    mut margin: Option<&mut crate::src::core::SkPoint::SkIPoint>,
) -> bool {
    use crate::include::core::SkBlurTypes::SkBlurStyle::*;
    use crate::src::core::{
        SkMask::*, SkMaskBlurFilter::SkMaskBlurFilter, SkPoint::SkIPoint, SkRect::SkIRect,
    };
    if !matches!(
        src.fFormat,
        Format::kBW_Format | Format::kA8_Format | Format::kARGB32_Format | Format::kLCD16_Format
    ) {
        return false;
    }
    let blur_filter = SkMaskBlurFilter::new(sigma as f64, sigma as f64);
    if blur_filter.hasNoBlur() {
        if style == kOuter_SkBlurStyle {
            *dst.image() = core::ptr::null_mut();
            *dst.bounds() = SkIRect::make_empty();
            *dst.rowBytes() = dst.fBounds.width() as u32;
            *dst.format() = Format::kA8_Format;
            if let Some(m) = margin.as_deref_mut() {
                *m = SkIPoint { fX: 0, fY: 0 };
            }
            return true;
        }
        return false;
    }
    let border = unsafe { blur_filter.blur(src, dst) };
    if !src.fImage.is_null() && dst.fImage.is_null() {
        return false;
    }
    if let Some(m) = margin.as_deref_mut() {
        *m = border;
    }
    if src.fImage.is_null() {
        if style == kInner_SkBlurStyle {
            *dst.bounds() = src.fBounds;
            *dst.rowBytes() = dst.fBounds.width() as u32;
        }
        return true;
    }
    // The source repeats the same format switch in each style. Macro expansion preserves each switch.
    macro_rules! dispatch {
        ($operation:ident,$($arg:expr),*)=>{
            match src.fFormat {
                Format::kBW_Format=>unsafe{$operation($($arg,)*BWAlphaIter::new(src.fImage,0),src.fRowBytes,src.fBounds.width(),src.fBounds.height())},
                Format::kA8_Format=>unsafe{$operation($($arg,)*A8AlphaIter::new(src.fImage),src.fRowBytes,src.fBounds.width(),src.fBounds.height())},
                Format::kARGB32_Format=>unsafe{$operation($($arg,)*ARGB32AlphaIter::new(src.fImage.cast()),src.fRowBytes,src.fBounds.width(),src.fBounds.height())},
                Format::kLCD16_Format=>unsafe{$operation($($arg,)*LCD16AlphaIter::new(src.fImage.cast()),src.fRowBytes,src.fBounds.width(),src.fBounds.height())},
                _=>crate::SK_ABORT!(@source b"src/core/SkBlurMask.cpp\0".as_ptr(),194; "Unhandled format."),
            }
        };
    }
    match style {
        kNormal_SkBlurStyle => {}
        kSolid_SkBlurStyle => {
            let dst_start = dst
                .fImage
                .cast_mut()
                .wrapping_offset(border.fX as isize + border.fY as isize * dst.fRowBytes as isize);
            dispatch!(clamp_solid_with_orig, dst_start, dst.fRowBytes as i32);
        }
        kOuter_SkBlurStyle => {
            let dst_start = dst
                .fImage
                .cast_mut()
                .wrapping_offset(border.fX as isize + border.fY as isize * dst.fRowBytes as isize);
            dispatch!(clamp_outer_with_orig, dst_start, dst.fRowBytes as i32);
        }
        kInner_SkBlurStyle => {
            let blur = core::mem::replace(dst, SkMaskBuilder::new());
            let _auto_free_blur_mask =
                unsafe { SkAutoMaskFreeImage::from_raw(blur.fImage.cast_mut()) };
            *dst = SkMaskBuilder::from_image(
                core::ptr::null_mut(),
                src.fBounds,
                src.fBounds.width() as u32,
                blur.fFormat,
            );
            let dst_size = dst.computeImageSize();
            if dst_size == 0 {
                return false;
            }
            *dst.image() = SkMaskBuilder::AllocImage(dst_size, AllocType::kUninit_Alloc);
            let blur_start = blur
                .fImage
                .wrapping_offset(border.fX as isize + border.fY as isize * blur.fRowBytes as isize);
            // Argument ordering differs for merge's extra blur row; retain source-before-blur evaluation.
            macro_rules! merge {
                ($it:expr) => {
                    unsafe {
                        merge_src_with_blur(
                            dst.fImage.cast_mut(),
                            dst.fRowBytes as i32,
                            $it,
                            src.fRowBytes,
                            blur_start,
                            blur.fRowBytes as i32,
                            src.fBounds.width(),
                            src.fBounds.height(),
                        )
                    }
                };
            }
            match src.fFormat {
                Format::kBW_Format => merge!(BWAlphaIter::new(src.fImage, 0)),
                Format::kA8_Format => merge!(A8AlphaIter::new(src.fImage)),
                Format::kARGB32_Format => merge!(ARGB32AlphaIter::new(src.fImage.cast())),
                Format::kLCD16_Format => merge!(LCD16AlphaIter::new(src.fImage.cast())),
                _ => {
                    crate::SK_ABORT!(@source b"src/core/SkBlurMask.cpp\0".as_ptr(),227; "Unhandled format.")
                }
            }
        }
    }
    true
}

// Source: src/core/SkBlurMask.cpp:319-321
// Source: src/core/SkBlurMask.cpp:323-324
// Source: src/core/SkBlurMask.cpp:327-328
// Source: src/core/SkBlurMask.cpp:330-331
// Source: src/core/SkBlurMask.cpp:333-334
// Source: src/core/SkBlurMask.cpp:336-336
fn gaussianIntegral(x: f32) -> f32 {
    if x > 1.5 {
        return 0.0;
    }
    if x < -1.5 {
        return 1.0;
    }
    let x2 = x * x;
    let x3 = x2 * x;
    if x > 0.5 {
        return 0.5625 - (x3 / 6.0 - 3.0 * x2 * 0.25 + 1.125 * x);
    }
    if x > -0.5 {
        return 0.5 - (0.75 * x - x3 / 3.0);
    }
    0.4375 + (-x3 / 6.0 - 3.0 * x2 * 0.25 - 1.125 * x)
}

// Source: src/core/SkBlurMask.cpp:347-348
// Source: src/core/SkBlurMask.cpp:350-350
// Source: src/core/SkBlurMask.cpp:352-352
// Source: src/core/SkBlurMask.cpp:354-358
// Borrowed slices replace caller-owned arrays, retaining caller ownership.
pub fn ComputeBlurProfile(profile: &mut [u8], size: i32, sigma: f32) {
    crate::SkASSERT!(@source b"src/core/SkBlurMask.cpp\0".as_ptr(),348; @condition b"SkScalarCeilToInt(6*sigma) == size\0".as_ptr(); crate::src::core::SkFloatingPoint::sk_float_ceil2int(6.0 * sigma)==size);
    let center = size >> 1;
    let invr = 1.0 / (2.0 * sigma);
    profile[0] = 255;
    for x in 1..size {
        let scaled_x = ((center - x) as f32 - 0.5) * invr;
        let gi = gaussianIntegral(scaled_x);
        profile[x as usize] = 255 - (255.0 * gi) as u8;
    }
}

// Source: src/core/SkBlurMask.cpp:369-370
// Source: src/core/SkBlurMask.cpp:372-375
// Source: src/core/SkBlurMask.cpp:378-378
pub fn ProfileLookup(profile: &[u8], loc: i32, blurredWidth: i32, sharpWidth: i32) -> u8 {
    let dx = (((loc << 1) + 1) - blurredWidth).abs() - sharpWidth;
    let mut ox = dx >> 1;
    if ox < 0 {
        ox = 0;
    }
    profile[ox as usize]
}

// Source: src/core/SkBlurMask.cpp:381-382
// Source: src/core/SkBlurMask.cpp:384-385
// Source: src/core/SkBlurMask.cpp:387-387
// Source: src/core/SkBlurMask.cpp:390-390
// Source: src/core/SkBlurMask.cpp:392-392
// Source: src/core/SkBlurMask.cpp:394-400
pub fn ComputeBlurredScanline(pixels: &mut [u8], profile: &[u8], width: u32, sigma: f32) {
    let profile_size = crate::src::core::SkFloatingPoint::sk_float_ceil2int(6.0 * sigma) as u32;
    // Vec<MaybeUninit<u8>> preserves the unused temporary allocation without zero filling.
    let _horizontal_scanline = vec![core::mem::MaybeUninit::<u8>::uninit(); width as usize];
    let sw = width.wrapping_sub(profile_size);
    let center = (profile_size & !1).wrapping_sub(1) as i32;
    let w = sw.wrapping_sub(center as u32) as i32;
    for x in 0..width {
        if profile_size <= sw {
            pixels[x as usize] = ProfileLookup(profile, x as i32, width as i32, w);
        } else {
            let span = sw as f32 / (2.0 * sigma);
            let gi_x = 1.5 - (x as f32 + 0.5) / (2.0 * sigma);
            pixels[x as usize] =
                (255.0 * (gaussianIntegral(gi_x) - gaussianIntegral(gi_x + span))) as u8;
        }
    }
}

// Source: src/core/SkBlurMask.cpp:405-413
// Source: src/core/SkBlurMask.cpp:416-418
// Source: src/core/SkBlurMask.cpp:421-424
// Source: src/core/SkBlurMask.cpp:426-428
// Source: src/core/SkBlurMask.cpp:430-431
// Source: src/core/SkBlurMask.cpp:433-436
// Source: src/core/SkBlurMask.cpp:438-438
// Source: src/core/SkBlurMask.cpp:441-441
// Source: src/core/SkBlurMask.cpp:443-443
// Source: src/core/SkBlurMask.cpp:445-447
// Source: src/core/SkBlurMask.cpp:450-451
// Source: src/core/SkBlurMask.cpp:453-454
// Source: src/core/SkBlurMask.cpp:456-456
// Source: src/core/SkBlurMask.cpp:458-459
// Source: src/core/SkBlurMask.cpp:461-462
// Source: src/core/SkBlurMask.cpp:464-467
// Source: src/core/SkBlurMask.cpp:471-471
// Source: src/core/SkBlurMask.cpp:473-475
// Source: src/core/SkBlurMask.cpp:477-481
// Source: src/core/SkBlurMask.cpp:483-483
// Source: src/core/SkBlurMask.cpp:485-486
// Source: src/core/SkBlurMask.cpp:488-491
// Source: src/core/SkBlurMask.cpp:493-496
// Source: src/core/SkBlurMask.cpp:502-502
pub unsafe fn BlurRect(
    sigma: f32,
    dst: &mut crate::src::core::SkMask::SkMaskBuilder,
    src: &crate::src::core::SkRect::SkRect,
    style: crate::include::core::SkBlurTypes::SkBlurStyle,
    margin: Option<&mut crate::src::core::SkPoint::SkIPoint>,
    create_mode: crate::src::core::SkMask::CreateMode,
) -> bool {
    use crate::include::core::SkBlurTypes::SkBlurStyle::*;
    use crate::src::core::{
        SkFloatingPoint::{sk_float_ceil2int, sk_float_floor2int, sk_float_round2int},
        SkMask::*,
        SkPoint::SkIPoint,
    };
    let profile_size = sk_float_ceil2int(6.0 * sigma);
    if profile_size <= 0 {
        return false;
    }
    let pad = profile_size / 2;
    if let Some(m) = margin {
        *m = SkIPoint { fX: pad, fY: pad };
    }
    dst.bounds().set_ltrb(
        sk_float_round2int(src.fLeft - pad as f32),
        sk_float_round2int(src.fTop - pad as f32),
        sk_float_round2int(src.fRight + pad as f32),
        sk_float_round2int(src.fBottom + pad as f32),
    );
    *dst.rowBytes() = dst.fBounds.width() as u32;
    *dst.format() = Format::kA8_Format;
    *dst.image() = core::ptr::null_mut();
    let sw = sk_float_floor2int(src.width());
    let sh = sk_float_floor2int(src.height());
    if create_mode == CreateMode::kJustComputeBounds_CreateMode {
        if style == kInner_SkBlurStyle {
            *dst.bounds() = src.round();
            *dst.rowBytes() = sw as u32;
        }
        return true;
    }
    let mut profile = vec![0u8; profile_size as usize];
    ComputeBlurProfile(&mut profile, profile_size, sigma);
    let dst_size = dst.computeImageSize();
    if dst_size == 0 {
        return false;
    }
    let dp = SkMaskBuilder::AllocImage(dst_size, AllocType::kUninit_Alloc);
    *dst.image() = dp;
    let (dst_height, dst_width) = (dst.fBounds.height(), dst.fBounds.width());
    let mut outptr = dp;
    let mut horizontal_scanline = vec![0u8; dst_width as usize];
    let mut vertical_scanline = vec![0u8; dst_height as usize];
    ComputeBlurredScanline(&mut horizontal_scanline, &profile, dst_width as u32, sigma);
    ComputeBlurredScanline(&mut vertical_scanline, &profile, dst_height as u32, sigma);
    for y in 0..dst_height {
        for x in 0..dst_width {
            // Exact scalar expansion of SkMulDiv255Round.
            let prod =
                horizontal_scanline[x as usize] as u32 * vertical_scanline[y as usize] as u32 + 128;
            let maskval = (prod + (prod >> 8)) >> 8;
            unsafe {
                *outptr = maskval as u8;
            }
            outptr = outptr.wrapping_add(1);
        }
    }
    if style == kInner_SkBlurStyle {
        let src_size = (src.width() * src.height()) as usize;
        if src_size == 0 {
            return false;
        }
        *dst.image() = SkMaskBuilder::AllocImage(src_size, AllocType::kUninit_Alloc);
        for y in 0..sh {
            let blur_scanline = dp.wrapping_offset(((y + pad) * dst_width + pad) as isize);
            let inner_scanline = dst.fImage.cast_mut().wrapping_offset((y * sw) as isize);
            unsafe {
                core::ptr::copy_nonoverlapping(blur_scanline, inner_scanline, sw as usize);
            }
        }
        unsafe {
            SkMaskBuilder::FreeImage(dp.cast());
        }
        *dst.bounds() = src.round();
        *dst.rowBytes() = sw as u32;
    } else if style == kOuter_SkBlurStyle {
        for y in pad..dst_height - pad {
            let dst_scanline = dp.wrapping_offset((y * dst_width + pad) as isize);
            unsafe {
                core::ptr::write_bytes(dst_scanline, 0, sw as usize);
            }
        }
    } else if style == kSolid_SkBlurStyle {
        for y in pad..dst_height - pad {
            let dst_scanline = dp.wrapping_offset((y * dst_width + pad) as isize);
            unsafe {
                core::ptr::write_bytes(dst_scanline, 255, sw as usize);
            }
        }
    }
    true
}

// Source: src/core/SkBlurMask.cpp:509-515
// Source: src/core/SkBlurMask.cpp:518-518
// Source: src/core/SkBlurMask.cpp:520-520
// Source: src/core/SkBlurMask.cpp:522-522
// Source: src/core/SkBlurMask.cpp:524-524
// Source: src/core/SkBlurMask.cpp:526-526
// Source: src/core/SkBlurMask.cpp:528-528
// Source: src/core/SkBlurMask.cpp:530-534
// Source: src/core/SkBlurMask.cpp:540-542
// Source: src/core/SkBlurMask.cpp:545-546
// Source: src/core/SkBlurMask.cpp:548-550
// Source: src/core/SkBlurMask.cpp:552-552
// Source: src/core/SkBlurMask.cpp:554-556
// Source: src/core/SkBlurMask.cpp:559-561
// Source: src/core/SkBlurMask.cpp:563-565
// Source: src/core/SkBlurMask.cpp:570-572
// Source: src/core/SkBlurMask.cpp:574-575
// Source: src/core/SkBlurMask.cpp:577-580
// Source: src/core/SkBlurMask.cpp:587-589
// Source: src/core/SkBlurMask.cpp:591-592
// Source: src/core/SkBlurMask.cpp:594-600
// Source: src/core/SkBlurMask.cpp:602-602
// Source: src/core/SkBlurMask.cpp:610-616
// Source: src/core/SkBlurMask.cpp:618-621
// Source: src/core/SkBlurMask.cpp:625-641
// Source: src/core/SkBlurMask.cpp:643-645
// Source: src/core/SkBlurMask.cpp:647-653
// Source: src/core/SkBlurMask.cpp:655-655
// Source: src/core/SkBlurMask.cpp:658-660
// Source: src/core/SkBlurMask.cpp:663-663
pub unsafe fn BlurGroundTruth(
    sigma: f32,
    dst: &mut crate::src::core::SkMask::SkMaskBuilder,
    src: &crate::src::core::SkMask::SkMask,
    style: crate::include::core::SkBlurTypes::SkBlurStyle,
    margin: Option<&mut crate::src::core::SkPoint::SkIPoint>,
) -> bool {
    use crate::include::core::SkBlurTypes::SkBlurStyle::*;
    use crate::src::core::{SkMask::*, SkPoint::SkIPoint};
    if src.fFormat != Format::kA8_Format {
        return false;
    }
    let variance = sigma * sigma;
    let mut window_size = crate::src::core::SkFloatingPoint::sk_float_ceil2int(sigma * 6.0);
    window_size |= 1;
    let mut gauss_window = vec![0f32; window_size as usize];
    let half_window = window_size >> 1;
    gauss_window[half_window as usize] = 1.0;
    let mut window_sum = 1.0;
    for x in 1..=half_window {
        let gaussian = (-(x * x) as f32 / (2.0 * variance)).exp();
        gauss_window[(half_window + x) as usize] = gaussian;
        gauss_window[(half_window - x) as usize] = gaussian;
        window_sum += 2.0 * gaussian;
    }
    let pad = half_window;
    if let Some(m) = margin {
        *m = SkIPoint { fX: pad, fY: pad };
    }
    *dst.bounds() = src.fBounds;
    dst.bounds().outset(pad, pad);
    *dst.rowBytes() = dst.fBounds.width() as u32;
    *dst.format() = Format::kA8_Format;
    *dst.image() = core::ptr::null_mut();
    if !src.fImage.is_null() {
        let dst_size = dst.computeImageSize();
        if dst_size == 0 {
            return false;
        }
        let (src_width, src_height, dst_width) = (
            src.fBounds.width(),
            src.fBounds.height(),
            dst.fBounds.width(),
        );
        let src_pixels = src.fImage;
        let dst_pixels = SkMaskBuilder::AllocImage(dst_size, AllocType::kUninit_Alloc);
        let auto_free_dst_pixels = unsafe { SkAutoMaskFreeImage::from_raw(dst_pixels) };
        let pad_width = src_width + 4 * pad;
        let pad_height = src_height;
        let pad_size = pad_width * pad_height;
        let mut pad_pixels = vec![0u8; pad_size as usize];
        for y in 0..src_height {
            let padptr = pad_pixels
                .as_mut_ptr()
                .wrapping_offset((y * pad_width + 2 * pad) as isize);
            // The reference algorithm intentionally uses srcWidth here, irrespective of row bytes.
            let srcptr = src_pixels.wrapping_offset((y * src_width) as isize);
            unsafe {
                core::ptr::copy_nonoverlapping(srcptr, padptr, src_width as usize);
            }
        }
        let tmp_width = pad_height + 4 * pad;
        let tmp_height = pad_width - 2 * pad;
        let tmp_size = tmp_width * tmp_height;
        let mut tmp_image = vec![0f32; tmp_size as usize];
        for y in 0..pad_height {
            let src_scanline = (y * pad_width) as usize;
            for x in pad..pad_width - pad {
                let out_pixel = ((x - pad) * tmp_width + y + 2 * pad) as usize;
                let window_center = src_scanline + x as usize;
                for i in -pad..=pad {
                    tmp_image[out_pixel] += gauss_window[(pad + i) as usize]
                        * pad_pixels[(window_center as isize + i as isize) as usize] as f32;
                }
                tmp_image[out_pixel] /= window_sum;
            }
        }
        for y in 0..tmp_height {
            let src_scanline = (y * tmp_width) as usize;
            for x in pad..tmp_width - pad {
                let window_center = src_scanline + x as usize;
                let mut final_value = 0.0;
                for i in -pad..=pad {
                    final_value += gauss_window[(pad + i) as usize]
                        * tmp_image[(window_center as isize + i as isize) as usize];
                }
                final_value /= window_sum;
                let out_pixel = dst_pixels.wrapping_offset(((x - pad) * dst_width + y) as isize);
                let integer_pixel = (final_value + 0.5) as i32;
                unsafe {
                    *out_pixel = integer_pixel.max(0).clamp(0, 255) as u8;
                }
            }
        }
        *dst.image() = dst_pixels;
        match style {
            kNormal_SkBlurStyle => {}
            kSolid_SkBlurStyle => unsafe {
                clamp_solid_with_orig(
                    dst_pixels
                        .wrapping_offset(pad as isize * dst.fRowBytes as isize + pad as isize),
                    dst.fRowBytes as i32,
                    A8AlphaIter::new(src_pixels),
                    src.fRowBytes,
                    src_width,
                    src_height,
                );
            },
            kOuter_SkBlurStyle => unsafe {
                clamp_outer_with_orig(
                    dst_pixels
                        .wrapping_offset(pad as isize * dst.fRowBytes as isize + pad as isize),
                    dst.fRowBytes as i32,
                    A8AlphaIter::new(src_pixels),
                    src.fRowBytes,
                    src_width,
                    src_height,
                );
            },
            kInner_SkBlurStyle => {
                let src_size = src.computeImageSize();
                if src_size == 0 {
                    return false;
                }
                *dst.image() = SkMaskBuilder::AllocImage(src_size, AllocType::kUninit_Alloc);
                unsafe {
                    merge_src_with_blur(
                        dst.fImage.cast_mut(),
                        src.fRowBytes as i32,
                        A8AlphaIter::new(src_pixels),
                        src.fRowBytes,
                        dst_pixels
                            .wrapping_offset(pad as isize * dst.fRowBytes as isize + pad as isize),
                        dst.fRowBytes as i32,
                        src_width,
                        src_height,
                    );
                    SkMaskBuilder::FreeImage(dst_pixels.cast());
                }
            }
        }
        // Equivalent to the source unique_ptr.release(): allocation now belongs to dst or was freed.
        core::mem::forget(auto_free_dst_pixels);
    }
    if style == kInner_SkBlurStyle {
        *dst.bounds() = src.fBounds;
        *dst.rowBytes() = src.fRowBytes;
    }
    true
}

// Source: src/core/SkBlurMask.h:22-23
// Native class contains only static methods; no object state or constructor body is invented.
pub struct SkBlurMask;
impl SkBlurMask {
    // Source: src/core/SkBlurMask.h:24-27
    #[must_use]
    pub unsafe fn BlurRect(
        sigma: f32,
        dst: &mut crate::src::core::SkMask::SkMaskBuilder,
        src: &crate::src::core::SkRect::SkRect,
        style: crate::include::core::SkBlurTypes::SkBlurStyle,
        margin: Option<&mut crate::src::core::SkPoint::SkIPoint>,
        create_mode: crate::src::core::SkMask::CreateMode,
    ) -> bool {
        unsafe { BlurRect(sigma, dst, src, style, margin, create_mode) }
    }
}
// Source: src/core/SkBlurMask.h:28-31
// Declaration only: the admitted native cpp has no SkBlurMask::BlurRRect definition.
// This function type records its real parameter/result contract without inventing a callable body.
pub type BlurRRectDeclaration = unsafe fn(
    f32,
    &mut crate::src::core::SkMask::SkMaskBuilder,
    &crate::src::core::SkRRect::SkRRect,
    crate::include::core::SkBlurTypes::SkBlurStyle,
    Option<&mut crate::src::core::SkPoint::SkIPoint>,
    crate::src::core::SkMask::CreateMode,
) -> bool;

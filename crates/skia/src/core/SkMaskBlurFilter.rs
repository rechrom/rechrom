// Copyright 2017 Google LLC. BSD-style license.
// Rust index ranges replace the three adjacent pointer ranges, retaining their shared scratch storage.

// Source: src/core/SkMaskBlurFilter.h:20-20
// Source: src/core/SkMaskBlurFilter.h:24-24
// Source: src/core/SkMaskBlurFilter.h:27-27
// Source: src/core/SkMaskBlurFilter.h:30-30
// Source: src/core/SkMaskBlurFilter.h:33-34
pub struct SkMaskBlurFilter {
    fSigmaW: f64,
    fSigmaH: f64,
}

// Source: src/core/SkMaskBlurFilter.cpp:25-25
// Source: src/core/SkMaskBlurFilter.cpp:27-29
// Source: src/core/SkMaskBlurFilter.cpp:31-33
// Source: src/core/SkMaskBlurFilter.cpp:77-78
// Source: src/core/SkMaskBlurFilter.cpp:82-84
// Source: src/core/SkMaskBlurFilter.cpp:86-86
// Source: src/core/SkMaskBlurFilter.cpp:89-89
// Source: src/core/SkMaskBlurFilter.cpp:91-91
pub struct PlanGauss {
    fWeight: u64,
    fBorder: i32,
    fSlidingWindow: i32,
    fPass0Size: usize,
    fPass1Size: usize,
    fPass2Size: usize,
}
impl PlanGauss {
    pub fn new(sigma: f64) -> Self {
        let possible_window =
            (sigma * 3.0 * (2.0 * std::f64::consts::PI).sqrt() / 4.0 + 0.5).floor() as i32;
        let window = 1.max(possible_window);
        let fPass0Size = (window - 1) as usize;
        let fPass1Size = (window - 1) as usize;
        let fPass2Size = (if (window & 1) == 1 {
            window - 1
        } else {
            window
        }) as usize;
        let fBorder = if (window & 1) == 1 {
            3 * ((window - 1) / 2)
        } else {
            3 * (window / 2) - 1
        };
        let fSlidingWindow = 2 * fBorder + 1;
        let window2 = window * window;
        let window3 = window2 * window;
        let divisor = if (window & 1) == 1 {
            window3
        } else {
            window3 + window2
        };
        let fWeight = (1.0 / divisor as f64 * ((1u64 << 32) as f64)).round() as u64;
        Self {
            fWeight,
            fBorder,
            fSlidingWindow,
            fPass0Size,
            fPass1Size,
            fPass2Size,
        }
    }
    pub fn bufferSize(&self) -> usize {
        self.fPass0Size + self.fPass1Size + self.fPass2Size
    }
    pub fn border(&self) -> i32 {
        self.fBorder
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:94-94
// Source: src/core/SkMaskBlurFilter.cpp:96-108
pub struct Scan<'a> {
    fWeight: u64,
    fNoChangeCount: i32,
    buffer: &'a mut [u32],
    starts: [usize; 3],
    ends: [usize; 3],
}

// Language substitution for the AlphaIter template; implementors preserve cursor arithmetic.
pub trait AlphaCursor: Copy {
    fn increment(&mut self);
    fn decrement(&mut self);
    fn less_than(&self, end: &Self) -> bool;
    /// Safety: cursor must point at a live readable pixel.
    unsafe fn value(&self) -> u8;
}
macro_rules! alpha_cursor {
    ($t:ty) => {
        impl AlphaCursor for $t {
            fn increment(&mut self) {
                <$t>::increment(self);
            }
            fn decrement(&mut self) {
                <$t>::decrement(self);
            }
            fn less_than(&self, end: &Self) -> bool {
                <$t>::less_than(self, end)
            }
            unsafe fn value(&self) -> u8 {
                unsafe { <$t>::value(self) }
            }
        }
    };
}
alpha_cursor!(crate::src::core::SkMask::BWAlphaIter);
alpha_cursor!(crate::src::core::SkMask::A8AlphaIter);
alpha_cursor!(crate::src::core::SkMask::ARGB32AlphaIter);
alpha_cursor!(crate::src::core::SkMask::LCD16AlphaIter);

// Source: src/core/SkMaskBlurFilter.cpp:110-114
// Source: src/core/SkMaskBlurFilter.cpp:116-116
// Source: src/core/SkMaskBlurFilter.cpp:118-120
// Source: src/core/SkMaskBlurFilter.cpp:123-127
// Source: src/core/SkMaskBlurFilter.cpp:129-129
// Source: src/core/SkMaskBlurFilter.cpp:131-133
// Source: src/core/SkMaskBlurFilter.cpp:135-137
// Source: src/core/SkMaskBlurFilter.cpp:139-141
// Source: src/core/SkMaskBlurFilter.cpp:145-149
// Source: src/core/SkMaskBlurFilter.cpp:151-151
// Source: src/core/SkMaskBlurFilter.cpp:153-155
// Source: src/core/SkMaskBlurFilter.cpp:157-159
// Source: src/core/SkMaskBlurFilter.cpp:161-163
// Source: src/core/SkMaskBlurFilter.cpp:165-165
// Source: src/core/SkMaskBlurFilter.cpp:169-169
// Source: src/core/SkMaskBlurFilter.cpp:171-171
// Source: src/core/SkMaskBlurFilter.cpp:173-180
// Source: src/core/SkMaskBlurFilter.cpp:182-182
// Source: src/core/SkMaskBlurFilter.cpp:184-186
// Source: src/core/SkMaskBlurFilter.cpp:188-190
// Source: src/core/SkMaskBlurFilter.cpp:192-194
impl Scan<'_> {
    /// Safety: source range and all destination addresses must remain valid for this scan.
    pub unsafe fn blur<A: AlphaCursor>(
        &mut self,
        srcBegin: A,
        srcEnd: A,
        mut dst: *mut u8,
        dstStride: i32,
        dstEnd: *mut u8,
    ) {
        let mut cursor = self.starts;
        self.buffer.fill(0);
        let (mut sum0, mut sum1, mut sum2) = (0u32, 0u32, 0u32);
        let mut src = srcBegin;
        while src.less_than(&srcEnd) {
            let leading_edge = unsafe { src.value() } as u32;
            sum0 = sum0.wrapping_add(leading_edge);
            sum1 = sum1.wrapping_add(sum0);
            sum2 = sum2.wrapping_add(sum1);
            unsafe {
                *dst = self.finalScale(sum2);
            }
            sum2 = sum2.wrapping_sub(self.buffer[cursor[2]]);
            self.buffer[cursor[2]] = sum1;
            cursor[2] = if cursor[2] + 1 < self.ends[2] {
                cursor[2] + 1
            } else {
                self.starts[2]
            };
            sum1 = sum1.wrapping_sub(self.buffer[cursor[1]]);
            self.buffer[cursor[1]] = sum0;
            cursor[1] = if cursor[1] + 1 < self.ends[1] {
                cursor[1] + 1
            } else {
                self.starts[1]
            };
            sum0 = sum0.wrapping_sub(self.buffer[cursor[0]]);
            self.buffer[cursor[0]] = leading_edge;
            cursor[0] = if cursor[0] + 1 < self.ends[0] {
                cursor[0] + 1
            } else {
                self.starts[0]
            };
            src.increment();
            dst = dst.wrapping_offset(dstStride as isize);
        }
        for _ in 0..self.fNoChangeCount {
            let leading_edge = 0;
            sum0 = sum0.wrapping_add(leading_edge);
            sum1 = sum1.wrapping_add(sum0);
            sum2 = sum2.wrapping_add(sum1);
            unsafe {
                *dst = self.finalScale(sum2);
            }
            sum2 = sum2.wrapping_sub(self.buffer[cursor[2]]);
            self.buffer[cursor[2]] = sum1;
            cursor[2] = if cursor[2] + 1 < self.ends[2] {
                cursor[2] + 1
            } else {
                self.starts[2]
            };
            sum1 = sum1.wrapping_sub(self.buffer[cursor[1]]);
            self.buffer[cursor[1]] = sum0;
            cursor[1] = if cursor[1] + 1 < self.ends[1] {
                cursor[1] + 1
            } else {
                self.starts[1]
            };
            sum0 = sum0.wrapping_sub(self.buffer[cursor[0]]);
            self.buffer[cursor[0]] = leading_edge;
            cursor[0] = if cursor[0] + 1 < self.ends[0] {
                cursor[0] + 1
            } else {
                self.starts[0]
            };
            dst = dst.wrapping_offset(dstStride as isize);
        }
        self.buffer.fill(0);
        sum0 = 0;
        sum1 = 0;
        sum2 = 0;
        let mut dst_cursor = dstEnd;
        let mut src = srcEnd;
        while dst_cursor > dst {
            dst_cursor = dst_cursor.wrapping_offset(-(dstStride as isize));
            src.decrement();
            let leading_edge = unsafe { src.value() } as u32;
            sum0 = sum0.wrapping_add(leading_edge);
            sum1 = sum1.wrapping_add(sum0);
            sum2 = sum2.wrapping_add(sum1);
            unsafe {
                *dst_cursor = self.finalScale(sum2);
            }
            sum2 = sum2.wrapping_sub(self.buffer[cursor[2]]);
            self.buffer[cursor[2]] = sum1;
            cursor[2] = if cursor[2] + 1 < self.ends[2] {
                cursor[2] + 1
            } else {
                self.starts[2]
            };
            sum1 = sum1.wrapping_sub(self.buffer[cursor[1]]);
            self.buffer[cursor[1]] = sum0;
            cursor[1] = if cursor[1] + 1 < self.ends[1] {
                cursor[1] + 1
            } else {
                self.starts[1]
            };
            sum0 = sum0.wrapping_sub(self.buffer[cursor[0]]);
            self.buffer[cursor[0]] = leading_edge;
            cursor[0] = if cursor[0] + 1 < self.ends[0] {
                cursor[0] + 1
            } else {
                self.starts[0]
            };
        }
    }
}
// Source: src/core/SkMaskBlurFilter.cpp:199-199
// Source: src/core/SkMaskBlurFilter.cpp:201-202
// Source: src/core/SkMaskBlurFilter.cpp:205-212
impl Scan<'_> {
    fn finalScale(&self, sum: u32) -> u8 {
        ((self
            .fWeight
            .wrapping_mul(sum as u64)
            .wrapping_add(1u64 << 31))
            >> 32) as u8
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:215-221
// Source: src/core/SkMaskBlurFilter.cpp:223-227
// Source: src/core/SkMaskBlurFilter.cpp:230-235
impl PlanGauss {
    pub fn makeBlurScan<'a>(&self, width: i32, buffer: &'a mut [u32]) -> Scan<'a> {
        let start1 = self.fPass0Size;
        let start2 = start1 + self.fPass1Size;
        let end2 = start2 + self.fPass2Size;
        let no_change_count = if self.fSlidingWindow > width {
            self.fSlidingWindow - width
        } else {
            0
        };
        Scan {
            fWeight: self.fWeight,
            fNoChangeCount: no_change_count,
            buffer: &mut buffer[..end2],
            starts: [0, start1, start2],
            ends: [start1, start2, end2],
        }
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:254-256
// Source: src/core/SkMaskBlurFilter.cpp:258-259
impl SkMaskBlurFilter {
    pub fn new(sigma_w: f64, sigma_h: f64) -> Self {
        let pin = |x: f64| {
            let hi = if 135.0 < x { 135.0 } else { x };
            if 0.0 < hi {
                hi
            } else {
                0.0
            }
        };
        let result = Self {
            fSigmaW: pin(sigma_w),
            fSigmaH: pin(sigma_h),
        };
        crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),258; @condition b"sigmaW >= 0\0".as_ptr(); sigma_w>=0.0);
        crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),259; @condition b"sigmaH >= 0\0".as_ptr(); sigma_h>=0.0);
        result
    }
}
// Source: src/core/SkMaskBlurFilter.cpp:262-262
// Source: src/core/SkMaskBlurFilter.cpp:267-267
// Source: src/core/SkMaskBlurFilter.cpp:271-271
// Source: src/core/SkMaskBlurFilter.cpp:273-273
impl SkMaskBlurFilter {
    pub fn hasNoBlur(&self) -> bool {
        // Preserve the upstream compile-time threshold switch as a Rust cfg of the same name.
        let no_window_sigma = if cfg!(SK_USE_LARGER_NO_BLUR_THRESHOLD) {
            0.531923
        } else {
            1.0 / 3.0
        };
        self.fSigmaW < no_window_sigma && self.fSigmaH <= no_window_sigma
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:278-279
// Source: src/core/SkMaskBlurFilter.cpp:281-284
unsafe fn bw_to_a8(a8: *mut u8, from: *const u8, width: i32) {
    crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),279; @condition b"0 < width && width <= 8\0".as_ptr(); 0 < width && width <= 8);
    let masks = unsafe { *from };
    for i in 0..width {
        unsafe {
            *a8.add(i as usize) = if ((masks >> (7 - i)) & 1) != 0 {
                255
            } else {
                0
            };
        }
    }
}
// Source: src/core/SkMaskBlurFilter.cpp:287-288
// Source: src/core/SkMaskBlurFilter.cpp:290-295
unsafe fn lcd_to_a8(a8: *mut u8, from: *const u8, width: i32) {
    crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),288; @condition b"0 < width && width <= 8\0".as_ptr(); 0 < width && width <= 8);
    use crate::src::core::SkColorData::{SkPacked16ToB32, SkPacked16ToG32, SkPacked16ToR32};
    for i in 0..width {
        let rgb = unsafe { *from.cast::<u16>().add(i as usize) } as u32;
        let r = SkPacked16ToR32(rgb);
        let g = SkPacked16ToG32(rgb);
        let b = SkPacked16ToB32(rgb);
        unsafe {
            *a8.add(i as usize) = ((r + g + b) / 3) as u8;
        }
    }
}
// Source: src/core/SkMaskBlurFilter.cpp:298-302
// Source: src/core/SkMaskBlurFilter.cpp:305-305
unsafe fn argb32_to_a8(a8: *mut u8, from: *const u8, width: i32) {
    crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),299; @condition b"0 < width && width <= 8\0".as_ptr(); 0 < width && width <= 8);
    for i in 0..width {
        let rgba = unsafe { *from.cast::<u32>().add(i as usize) };
        unsafe {
            *a8.add(i as usize) = crate::src::core::SkColorPriv::SkGetPackedA32(rgba) as u8;
        }
    }
}
type ToA8 = unsafe fn(*mut u8, *const u8, i32);

// Source: src/core/SkMaskBlurFilter.cpp:307-307
// A fixed array replaces skvx::Vec; all operations retain lane-wise unsigned wrapping semantics.
type Fp88 = [u16; 8];

// Source: src/core/SkMaskBlurFilter.cpp:309-309
// Source: src/core/SkMaskBlurFilter.cpp:314-320
// Source: src/core/SkMaskBlurFilter.cpp:322-322
// Source: src/core/SkMaskBlurFilter.cpp:326-326
unsafe fn load(mut from: *const u8, width: i32, to_a8: Option<ToA8>) -> Fp88 {
    let mut tmp = [0u8; 8];
    if let Some(convert) = to_a8 {
        unsafe {
            convert(tmp.as_mut_ptr(), from, width);
        }
        from = tmp.as_ptr();
    } else if width < 8 {
        for i in 0..width {
            tmp[i as usize] = unsafe { *from.add(i as usize) };
        }
        from = tmp.as_ptr();
    }
    core::array::from_fn(|i| (unsafe { *from.add(i) } as u16) << 8)
}
// Source: src/core/SkMaskBlurFilter.cpp:329-337
unsafe fn store(to: *mut u8, v: &Fp88, width: i32) {
    let b: [u8; 8] = core::array::from_fn(|i| (v[i] >> 8) as u8);
    if width == 8 {
        unsafe {
            core::ptr::copy_nonoverlapping(b.as_ptr(), to, 8);
        }
    } else {
        let buffer = b;
        for i in 0..width {
            unsafe {
                *to.add(i as usize) = buffer[i as usize];
            }
        }
    }
}
// Source: src/core/SkMaskBlurFilter.cpp:342-343
const ZERO: u16 = 0;
const HALF: u16 = 0x80;

// Language substitutions for vector multiply-high and unsigned lane accumulation.
fn mulhi(a: &Fp88, b: &Fp88) -> Fp88 {
    core::array::from_fn(|i| ((a[i] as u32 * b[i] as u32) >> 16) as u16)
}
fn add_lanes(dst: &mut Fp88, v: Fp88) {
    for i in 0..8 {
        dst[i] = dst[i].wrapping_add(v[i]);
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:424-427
// Source: src/core/SkMaskBlurFilter.cpp:429-430
// Source: src/core/SkMaskBlurFilter.cpp:433-433
// Source: src/core/SkMaskBlurFilter.cpp:436-437
// Source: src/core/SkMaskBlurFilter.cpp:440-441
fn blur_x_radius_1(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d0: &mut Fp88,
    d8: &mut Fp88,
) {
    let v1 = mulhi(s0, g1);
    let v0 = mulhi(s0, g0);
    add_lanes(d0, v1);
    add_lanes(d0, [ZERO, v0[0], v0[1], v0[2], v0[3], v0[4], v0[5], v0[6]]);
    add_lanes(d8, [v0[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, v1[0], v1[1], v1[2], v1[3], v1[4], v1[5]]);
    add_lanes(d8, [v1[6], v1[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
}

// Source: src/core/SkMaskBlurFilter.cpp:445-451
// Source: src/core/SkMaskBlurFilter.cpp:454-454
// Source: src/core/SkMaskBlurFilter.cpp:457-458
// Source: src/core/SkMaskBlurFilter.cpp:461-462
// Source: src/core/SkMaskBlurFilter.cpp:465-466
// Source: src/core/SkMaskBlurFilter.cpp:469-470
fn blur_x_radius_2(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d0: &mut Fp88,
    d8: &mut Fp88,
) {
    let v0 = mulhi(s0, g0);
    let v1 = mulhi(s0, g1);
    let v2 = mulhi(s0, g2);
    add_lanes(d0, v2);
    add_lanes(d0, [ZERO, v1[0], v1[1], v1[2], v1[3], v1[4], v1[5], v1[6]]);
    add_lanes(d8, [v1[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, v0[0], v0[1], v0[2], v0[3], v0[4], v0[5]]);
    add_lanes(d8, [v0[6], v0[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, v1[0], v1[1], v1[2], v1[3], v1[4]]);
    add_lanes(d8, [v1[5], v1[6], v1[7], ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, v2[0], v2[1], v2[2], v2[3]]);
    add_lanes(d8, [v2[4], v2[5], v2[6], v2[7], ZERO, ZERO, ZERO, ZERO]);
}

// Source: src/core/SkMaskBlurFilter.cpp:473-480
// Source: src/core/SkMaskBlurFilter.cpp:483-483
// Source: src/core/SkMaskBlurFilter.cpp:486-487
// Source: src/core/SkMaskBlurFilter.cpp:490-491
// Source: src/core/SkMaskBlurFilter.cpp:494-495
// Source: src/core/SkMaskBlurFilter.cpp:498-499
// Source: src/core/SkMaskBlurFilter.cpp:502-503
// Source: src/core/SkMaskBlurFilter.cpp:506-507
fn blur_x_radius_3(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d0: &mut Fp88,
    d8: &mut Fp88,
) {
    let v0 = mulhi(s0, g0);
    let v1 = mulhi(s0, g1);
    let v2 = mulhi(s0, g2);
    let v3 = mulhi(s0, g3);
    add_lanes(d0, v3);
    add_lanes(d0, [ZERO, v2[0], v2[1], v2[2], v2[3], v2[4], v2[5], v2[6]]);
    add_lanes(d8, [v2[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, v1[0], v1[1], v1[2], v1[3], v1[4], v1[5]]);
    add_lanes(d8, [v1[6], v1[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, v0[0], v0[1], v0[2], v0[3], v0[4]]);
    add_lanes(d8, [v0[5], v0[6], v0[7], ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, v1[0], v1[1], v1[2], v1[3]]);
    add_lanes(d8, [v1[4], v1[5], v1[6], v1[7], ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, ZERO, v2[0], v2[1], v2[2]]);
    add_lanes(d8, [v2[3], v2[4], v2[5], v2[6], v2[7], ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, v3[0], v3[1]]);
    add_lanes(d8, [v3[2], v3[3], v3[4], v3[5], v3[6], v3[7], ZERO, ZERO]);
}

// Source: src/core/SkMaskBlurFilter.cpp:510-518
// Source: src/core/SkMaskBlurFilter.cpp:521-521
// Source: src/core/SkMaskBlurFilter.cpp:524-525
// Source: src/core/SkMaskBlurFilter.cpp:528-529
// Source: src/core/SkMaskBlurFilter.cpp:532-533
// Source: src/core/SkMaskBlurFilter.cpp:536-537
// Source: src/core/SkMaskBlurFilter.cpp:540-541
// Source: src/core/SkMaskBlurFilter.cpp:544-545
// Source: src/core/SkMaskBlurFilter.cpp:548-549
// Source: src/core/SkMaskBlurFilter.cpp:552-552
fn blur_x_radius_4(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d0: &mut Fp88,
    d8: &mut Fp88,
) {
    let v0 = mulhi(s0, g0);
    let v1 = mulhi(s0, g1);
    let v2 = mulhi(s0, g2);
    let v3 = mulhi(s0, g3);
    let v4 = mulhi(s0, g4);
    add_lanes(d0, v4);
    add_lanes(d0, [ZERO, v3[0], v3[1], v3[2], v3[3], v3[4], v3[5], v3[6]]);
    add_lanes(d8, [v3[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, v2[0], v2[1], v2[2], v2[3], v2[4], v2[5]]);
    add_lanes(d8, [v2[6], v2[7], ZERO, ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, v1[0], v1[1], v1[2], v1[3], v1[4]]);
    add_lanes(d8, [v1[5], v1[6], v1[7], ZERO, ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, v0[0], v0[1], v0[2], v0[3]]);
    add_lanes(d8, [v0[4], v0[5], v0[6], v0[7], ZERO, ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, ZERO, v1[0], v1[1], v1[2]]);
    add_lanes(d8, [v1[3], v1[4], v1[5], v1[6], v1[7], ZERO, ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, v2[0], v2[1]]);
    add_lanes(d8, [v2[2], v2[3], v2[4], v2[5], v2[6], v2[7], ZERO, ZERO]);
    add_lanes(d0, [ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, v3[0]]);
    add_lanes(d8, [v3[1], v3[2], v3[3], v3[4], v3[5], v3[6], v3[7], ZERO]);
    add_lanes(d8, v4);
}

// Source: src/core/SkMaskBlurFilter.cpp:555-555
type BlurX = fn(&Fp88, &Fp88, &Fp88, &Fp88, &Fp88, &Fp88, &mut Fp88, &mut Fp88);

// Source: src/core/SkMaskBlurFilter.cpp:558-562
// Source: src/core/SkMaskBlurFilter.cpp:564-564
// Source: src/core/SkMaskBlurFilter.cpp:567-569
// Source: src/core/SkMaskBlurFilter.cpp:571-571
// Source: src/core/SkMaskBlurFilter.cpp:573-574
// Source: src/core/SkMaskBlurFilter.cpp:576-577
// Source: src/core/SkMaskBlurFilter.cpp:581-582
// Source: src/core/SkMaskBlurFilter.cpp:584-584
// Source: src/core/SkMaskBlurFilter.cpp:586-587
// Source: src/core/SkMaskBlurFilter.cpp:589-591
// Source: src/core/SkMaskBlurFilter.cpp:595-597
unsafe fn blur_row(
    blur: BlurX,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    mut src: *const u8,
    src_w: i32,
    mut dst: *mut u8,
    dst_w: i32,
) {
    let (mut d0, mut d8) = ([HALF; 8], [HALF; 8]);
    let mut x = 0;
    while x <= src_w - 8 {
        blur(
            &unsafe { load(src, 8, None) },
            g0,
            g1,
            g2,
            g3,
            g4,
            &mut d0,
            &mut d8,
        );
        unsafe {
            store(dst, &d0, 8);
        }
        d0 = d8;
        d8 = [HALF; 8];
        src = src.wrapping_add(8);
        dst = dst.wrapping_add(8);
        x += 8;
    }
    let src_tail = src_w - x;
    if src_tail > 0 {
        blur(
            &unsafe { load(src, src_tail, None) },
            g0,
            g1,
            g2,
            g3,
            g4,
            &mut d0,
            &mut d8,
        );
        let dst_tail = 8.min(dst_w - x);
        unsafe {
            store(dst, &d0, dst_tail);
        }
        d0 = d8;
        dst = dst.wrapping_offset(dst_tail as isize);
        x += dst_tail;
    }
    let dst_tail = dst_w - x;
    if dst_tail > 0 {
        unsafe {
            store(dst, &d0, dst_tail);
        }
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:602-605
// Source: src/core/SkMaskBlurFilter.cpp:607-611
// Source: src/core/SkMaskBlurFilter.cpp:614-617
unsafe fn blur_x_rect(
    blur: BlurX,
    gauss: &[u16; 6],
    mut src: *const u8,
    src_stride: usize,
    src_w: i32,
    mut dst: *mut u8,
    dst_stride: usize,
    dst_w: i32,
    dst_h: i32,
) {
    let (g0, g1, g2, g3, g4) = (
        [gauss[0]; 8],
        [gauss[1]; 8],
        [gauss[2]; 8],
        [gauss[3]; 8],
        [gauss[4]; 8],
    );
    for _ in 0..dst_h {
        unsafe {
            blur_row(blur, &g0, &g1, &g2, &g3, &g4, src, src_w, dst, dst_w);
        }
        src = src.wrapping_add(src_stride);
        dst = dst.wrapping_add(dst_stride);
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:621-623
// Source: src/core/SkMaskBlurFilter.cpp:625-628
// Source: src/core/SkMaskBlurFilter.cpp:630-632
// Source: src/core/SkMaskBlurFilter.cpp:634-636
// Source: src/core/SkMaskBlurFilter.cpp:638-640
// Source: src/core/SkMaskBlurFilter.cpp:642-643
unsafe fn direct_blur_x(
    radius: i32,
    gauss: &[u16; 6],
    src: *const u8,
    src_stride: usize,
    src_w: i32,
    dst: *mut u8,
    dst_stride: usize,
    dst_w: i32,
    dst_h: i32,
) {
    let blur: BlurX = match radius {
        1 => blur_x_radius_1,
        2 => blur_x_radius_2,
        3 => blur_x_radius_3,
        4 => blur_x_radius_4,
        _ => {
            crate::SkASSERTF!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),643; @condition b"false\0".as_ptr(); false,"The radius %d is not handled\n",radius as core::ffi::c_int);
            return;
        }
    };
    unsafe {
        blur_x_rect(
            blur, gauss, src, src_stride, src_w, dst, dst_stride, dst_w, dst_h,
        );
    }
}

// Lane-wise wrapping addition replaces skvx vector addition.
fn sum_lanes(a: Fp88, b: Fp88) -> Fp88 {
    core::array::from_fn(|i| a[i].wrapping_add(b[i]))
}

// Source: src/core/SkMaskBlurFilter.cpp:697-702
// Source: src/core/SkMaskBlurFilter.cpp:704-706
// Source: src/core/SkMaskBlurFilter.cpp:708-708
fn blur_y_radius_1(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d: &mut [Fp88; 8],
) -> Fp88 {
    let v0 = mulhi(s0, g0);
    let v1 = mulhi(s0, g1);
    let answer = sum_lanes(d[0], v1);
    d[0] = sum_lanes(d[1], v0);
    d[1] = sum_lanes(v1, [HALF; 8]);
    answer
}

// Source: src/core/SkMaskBlurFilter.cpp:711-717
// Source: src/core/SkMaskBlurFilter.cpp:719-723
// Source: src/core/SkMaskBlurFilter.cpp:725-725
fn blur_y_radius_2(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d: &mut [Fp88; 8],
) -> Fp88 {
    let v0 = mulhi(s0, g0);
    let v1 = mulhi(s0, g1);
    let v2 = mulhi(s0, g2);
    let answer = sum_lanes(d[0], v2);
    d[0] = sum_lanes(d[1], v1);
    d[1] = sum_lanes(d[2], v0);
    d[2] = sum_lanes(d[3], v1);
    d[3] = sum_lanes(v2, [HALF; 8]);
    answer
}

// Source: src/core/SkMaskBlurFilter.cpp:728-735
// Source: src/core/SkMaskBlurFilter.cpp:737-743
// Source: src/core/SkMaskBlurFilter.cpp:745-745
fn blur_y_radius_3(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d: &mut [Fp88; 8],
) -> Fp88 {
    let v0 = mulhi(s0, g0);
    let v1 = mulhi(s0, g1);
    let v2 = mulhi(s0, g2);
    let v3 = mulhi(s0, g3);
    let answer = sum_lanes(d[0], v3);
    d[0] = sum_lanes(d[1], v2);
    d[1] = sum_lanes(d[2], v1);
    d[2] = sum_lanes(d[3], v0);
    d[3] = sum_lanes(d[4], v1);
    d[4] = sum_lanes(d[5], v2);
    d[5] = sum_lanes(v3, [HALF; 8]);
    answer
}

// Source: src/core/SkMaskBlurFilter.cpp:748-756
// Source: src/core/SkMaskBlurFilter.cpp:758-766
// Source: src/core/SkMaskBlurFilter.cpp:768-768
fn blur_y_radius_4(
    s0: &Fp88,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    d: &mut [Fp88; 8],
) -> Fp88 {
    let v0 = mulhi(s0, g0);
    let v1 = mulhi(s0, g1);
    let v2 = mulhi(s0, g2);
    let v3 = mulhi(s0, g3);
    let v4 = mulhi(s0, g4);
    let answer = sum_lanes(d[0], v4);
    d[0] = sum_lanes(d[1], v3);
    d[1] = sum_lanes(d[2], v2);
    d[2] = sum_lanes(d[3], v1);
    d[3] = sum_lanes(d[4], v0);
    d[4] = sum_lanes(d[5], v1);
    d[5] = sum_lanes(d[6], v2);
    d[6] = sum_lanes(d[7], v3);
    d[7] = sum_lanes(v4, [HALF; 8]);
    answer
}

// Source: src/core/SkMaskBlurFilter.cpp:771-771
type BlurY = fn(&Fp88, &Fp88, &Fp88, &Fp88, &Fp88, &Fp88, &mut [Fp88; 8]) -> Fp88;

// Source: src/core/SkMaskBlurFilter.cpp:774-781
// Source: src/core/SkMaskBlurFilter.cpp:783-787
// Source: src/core/SkMaskBlurFilter.cpp:790-797
// Source: src/core/SkMaskBlurFilter.cpp:800-801
// Source: src/core/SkMaskBlurFilter.cpp:803-804
// Source: src/core/SkMaskBlurFilter.cpp:806-807
// Source: src/core/SkMaskBlurFilter.cpp:809-810
unsafe fn blur_column(
    to_a8: Option<ToA8>,
    blur: BlurY,
    radius: i32,
    width: i32,
    g0: &Fp88,
    g1: &Fp88,
    g2: &Fp88,
    g3: &Fp88,
    g4: &Fp88,
    mut src: *const u8,
    src_rb: usize,
    src_h: i32,
    mut dst: *mut u8,
    dst_rb: usize,
) {
    let mut d = [[HALF; 8]; 8];
    let flush = |mut to: *mut u8, v0: &Fp88, v1: &Fp88| {
        unsafe {
            store(to, v0, width);
        }
        to = to.wrapping_add(dst_rb);
        unsafe {
            store(to, v1, width);
        }
        to.wrapping_add(dst_rb)
    };
    for _ in 0..src_h {
        let s = unsafe { load(src, width, to_a8) };
        let b = blur(&s, g0, g1, g2, g3, g4, &mut d);
        unsafe {
            store(dst, &b, width);
        }
        src = src.wrapping_add(src_rb);
        dst = dst.wrapping_add(dst_rb);
    }
    if radius >= 1 {
        dst = flush(dst, &d[0], &d[1]);
    }
    if radius >= 2 {
        dst = flush(dst, &d[2], &d[3]);
    }
    if radius >= 3 {
        dst = flush(dst, &d[4], &d[5]);
    }
    if radius >= 4 {
        flush(dst, &d[6], &d[7]);
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:815-818
// Source: src/core/SkMaskBlurFilter.cpp:820-824
// Source: src/core/SkMaskBlurFilter.cpp:826-833
// Source: src/core/SkMaskBlurFilter.cpp:836-841
unsafe fn blur_y_rect(
    to_a8: Option<ToA8>,
    stride_of_8: i32,
    blur: BlurY,
    radius: i32,
    gauss: &[u16; 6],
    mut src: *const u8,
    src_rb: usize,
    src_w: i32,
    src_h: i32,
    mut dst: *mut u8,
    dst_rb: usize,
) {
    let (g0, g1, g2, g3, g4) = (
        [gauss[0]; 8],
        [gauss[1]; 8],
        [gauss[2]; 8],
        [gauss[3]; 8],
        [gauss[4]; 8],
    );
    let mut x = 0;
    while x <= src_w - 8 {
        unsafe {
            blur_column(
                to_a8, blur, radius, 8, &g0, &g1, &g2, &g3, &g4, src, src_rb, src_h, dst, dst_rb,
            );
        }
        src = src.wrapping_offset(stride_of_8 as isize);
        dst = dst.wrapping_add(8);
        x += 8;
    }
    let x_tail = src_w - x;
    if x_tail > 0 {
        unsafe {
            blur_column(
                to_a8, blur, radius, x_tail, &g0, &g1, &g2, &g3, &g4, src, src_rb, src_h, dst,
                dst_rb,
            );
        }
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:845-848
// Source: src/core/SkMaskBlurFilter.cpp:850-855
// Source: src/core/SkMaskBlurFilter.cpp:857-861
// Source: src/core/SkMaskBlurFilter.cpp:863-867
// Source: src/core/SkMaskBlurFilter.cpp:869-873
// Source: src/core/SkMaskBlurFilter.cpp:875-876
unsafe fn direct_blur_y(
    to_a8: Option<ToA8>,
    stride_of_8: i32,
    radius: i32,
    gauss: &[u16; 6],
    src: *const u8,
    src_rb: usize,
    src_w: i32,
    src_h: i32,
    dst: *mut u8,
    dst_rb: usize,
) {
    let blur: BlurY = match radius {
        1 => blur_y_radius_1,
        2 => blur_y_radius_2,
        3 => blur_y_radius_3,
        4 => blur_y_radius_4,
        _ => {
            crate::SkASSERTF!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),876; @condition b"false\0".as_ptr(); false,"The radius %d is not handled\n",radius as core::ffi::c_int);
            return;
        }
    };
    unsafe {
        blur_y_rect(
            to_a8,
            stride_of_8,
            blur,
            radius,
            gauss,
            src,
            src_rb,
            src_w,
            src_h,
            dst,
            dst_rb,
        );
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:880-883
// Source: src/core/SkMaskBlurFilter.cpp:885-886
// Source: src/core/SkMaskBlurFilter.cpp:888-889
// Source: src/core/SkMaskBlurFilter.cpp:891-891
// Source: src/core/SkMaskBlurFilter.cpp:893-896
// Source: src/core/SkMaskBlurFilter.cpp:900-901
// Source: src/core/SkMaskBlurFilter.cpp:903-904
// Source: src/core/SkMaskBlurFilter.cpp:906-908
// Source: src/core/SkMaskBlurFilter.cpp:910-912
// Source: src/core/SkMaskBlurFilter.cpp:915-916
// Source: src/core/SkMaskBlurFilter.cpp:918-919
// Source: src/core/SkMaskBlurFilter.cpp:921-922
// Source: src/core/SkMaskBlurFilter.cpp:927-952
// Source: src/core/SkMaskBlurFilter.cpp:956-958
// Source: src/core/SkMaskBlurFilter.cpp:960-960
pub unsafe fn small_blur(
    sigma_x: f64,
    sigma_y: f64,
    src: &crate::src::core::SkMask::SkMask,
    dst: &mut crate::src::core::SkMask::SkMaskBuilder,
) -> crate::src::core::SkPoint::SkIPoint {
    use crate::src::core::{
        SkGaussFilter::SkGaussFilter,
        SkMask::{Format, SkMaskBuilder},
        SkPoint::SkIPoint,
    };
    crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),881; @condition b"sigmaX == sigmaY\0".as_ptr(); (sigma_x)==(sigma_y));
    crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),882; @condition b"0.01 <= sigmaX && sigmaX < 2\0".as_ptr(); 0.01<=sigma_x && sigma_x<2.0);
    crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),883; @condition b"0.01 <= sigmaY && sigmaY < 2\0".as_ptr(); 0.01<=sigma_y && sigma_y<2.0);
    let filter_x = SkGaussFilter::new(sigma_x);
    let filter_y = SkGaussFilter::new(sigma_y);
    let radius_x = filter_x.radius();
    let radius_y = filter_y.radius();
    crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),891; @condition b"radiusX <= 4 && radiusY <= 4\0".as_ptr(); radius_x<=4 && radius_y<=4);
    let prepare_gauss = |filter: &SkGaussFilter, factors: &mut [u16; 6]| {
        let mut i = 0;
        for &d in filter.as_slice() {
            factors[i] = (d * ((1u32 << 16) as f64)).round() as u16;
            i += 1;
        }
    };
    // Inactive factors are initialized in Rust; radius specializations never consume them.
    let (mut factors_x, mut factors_y) = ([0u16; 6], [0u16; 6]);
    prepare_gauss(&filter_x, &mut factors_x);
    prepare_gauss(&filter_y, &mut factors_y);
    *dst = SkMaskBuilder::PrepareDestination(radius_x, radius_y, src);
    if src.fImage.is_null() {
        return SkIPoint {
            fX: radius_x,
            fY: radius_y,
        };
    }
    if dst.fImage.is_null() {
        dst.bounds().set_empty();
        return SkIPoint { fX: 0, fY: 0 };
    }
    let (src_w, src_h) = (src.fBounds.width(), src.fBounds.height());
    let (dst_w, dst_h) = (dst.fBounds.width(), dst.fBounds.height());
    let (src_rb, dst_rb) = (src.fRowBytes as usize, dst.fRowBytes as usize);
    let dst_image = dst.fImage.cast_mut();
    let (to_a8, stride): (Option<ToA8>, i32) = match src.fFormat {
        Format::kBW_Format => (Some(bw_to_a8), 1),
        Format::kA8_Format => (None, 8),
        Format::kARGB32_Format => (Some(argb32_to_a8), 32),
        Format::kLCD16_Format => (Some(lcd_to_a8), 16),
        _ => {
            crate::SK_ABORT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),952; "Unhandled format.")
        }
    };
    unsafe {
        direct_blur_y(
            to_a8,
            stride,
            radius_y,
            &factors_y,
            src.fImage,
            src_rb,
            src_w,
            src_h,
            dst_image.wrapping_offset(radius_x as isize),
            dst_rb,
        );
    }
    unsafe {
        direct_blur_x(
            radius_x,
            &factors_x,
            dst_image.wrapping_offset(radius_x as isize),
            dst_rb,
            src_w,
            dst_image,
            dst_rb,
            dst_w,
            dst_h,
        );
    }
    SkIPoint {
        fX: radius_x,
        fY: radius_y,
    }
}

// Source: src/core/SkMaskBlurFilter.cpp:965-965
// Source: src/core/SkMaskBlurFilter.cpp:967-968
// Source: src/core/SkMaskBlurFilter.cpp:972-972
// Source: src/core/SkMaskBlurFilter.cpp:974-975
// Source: src/core/SkMaskBlurFilter.cpp:977-979
// Source: src/core/SkMaskBlurFilter.cpp:981-983
// Source: src/core/SkMaskBlurFilter.cpp:985-987
// Source: src/core/SkMaskBlurFilter.cpp:990-994
// Source: src/core/SkMaskBlurFilter.cpp:996-997
// Source: src/core/SkMaskBlurFilter.cpp:1000-1001
// Source: src/core/SkMaskBlurFilter.cpp:1004-1005
// Source: src/core/SkMaskBlurFilter.cpp:1007-1007
// Source: src/core/SkMaskBlurFilter.cpp:1010-1018
// Source: src/core/SkMaskBlurFilter.cpp:1020-1027
// Source: src/core/SkMaskBlurFilter.cpp:1029-1036
// Source: src/core/SkMaskBlurFilter.cpp:1038-1045
// Source: src/core/SkMaskBlurFilter.cpp:1047-1049
// Source: src/core/SkMaskBlurFilter.cpp:1054-1057
// Source: src/core/SkMaskBlurFilter.cpp:1059-1060
// Source: src/core/SkMaskBlurFilter.cpp:1063-1063
impl SkMaskBlurFilter {
    /// Safety: source image and row storage must remain valid throughout filtering.
    pub unsafe fn blur(
        &self,
        src: &crate::src::core::SkMask::SkMask,
        dst: &mut crate::src::core::SkMask::SkMaskBuilder,
    ) -> crate::src::core::SkPoint::SkIPoint {
        use crate::src::core::{SkMask::*, SkPoint::SkIPoint};
        if self.fSigmaW < 2.0 && self.fSigmaH < 2.0 {
            return unsafe { small_blur(self.fSigmaW, self.fSigmaH, src, dst) };
        }
        // Vec storage replaces the scoped arena; scratch allocations remain local to this call.
        let plan_w = PlanGauss::new(self.fSigmaW);
        let plan_h = PlanGauss::new(self.fSigmaH);
        let (border_w, border_h) = (plan_w.border(), plan_h.border());
        crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),979; @condition b"borderH >= 0 && borderW >= 0\0".as_ptr(); border_h>=0 && border_w>=0);
        *dst = SkMaskBuilder::PrepareDestination(border_w, border_h, src);
        if src.fImage.is_null() {
            return SkIPoint {
                fX: border_w,
                fY: border_h,
            };
        }
        if dst.fImage.is_null() {
            dst.bounds().set_empty();
            return SkIPoint { fX: 0, fY: 0 };
        }
        let (src_w, src_h, dst_w, dst_h) = (
            src.fBounds.width(),
            src.fBounds.height(),
            dst.fBounds.width(),
            dst.fBounds.height(),
        );
        crate::SkASSERT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),994; @condition b"srcW >= 0 && srcH >= 0 && dstW >= 0 && dstH >= 0\0".as_ptr(); src_w>=0 && src_h>=0 && dst_w>=0 && dst_h>=0);
        let buffer_size = plan_w.bufferSize().max(plan_h.bufferSize());
        let mut buffer = vec![0u32; buffer_size];
        let (tmp_w, tmp_h) = (src_h, dst_w);
        if tmp_h > i32::MAX / tmp_w {
            return SkIPoint { fX: 0, fY: 0 };
        }
        let mut tmp = vec![0u8; (tmp_w * tmp_h) as usize];
        let mut scan_w = plan_w.makeBlurScan(src_w, &mut buffer);
        macro_rules! horizontal {
            ($start:expr,$end:expr) => {{
                let mut start = $start;
                let mut end = $end;
                for y in 0..src_h {
                    let tmp_start = tmp.as_mut_ptr().wrapping_offset(y as isize);
                    unsafe {
                        scan_w.blur(
                            start,
                            end,
                            tmp_start,
                            tmp_w,
                            tmp_start.wrapping_offset((tmp_w * tmp_h) as isize),
                        );
                    }
                    start.advance_row(src.fRowBytes);
                    end.advance_row(src.fRowBytes);
                }
            }};
        }
        match src.fFormat {
            Format::kBW_Format => horizontal!(
                BWAlphaIter::new(src.fImage, 0),
                BWAlphaIter::new(src.fImage.wrapping_offset((src_w / 8) as isize), src_w % 8)
            ),
            Format::kA8_Format => horizontal!(
                A8AlphaIter::new(src.fImage),
                A8AlphaIter::new(src.fImage.wrapping_offset(src_w as isize))
            ),
            Format::kARGB32_Format => {
                let argb = src.fImage.cast::<u32>();
                horizontal!(
                    ARGB32AlphaIter::new(argb),
                    ARGB32AlphaIter::new(argb.wrapping_offset(src_w as isize))
                );
            }
            Format::kLCD16_Format => {
                let lcd = src.fImage.cast::<u16>();
                horizontal!(
                    LCD16AlphaIter::new(lcd),
                    LCD16AlphaIter::new(lcd.wrapping_offset(src_w as isize))
                );
            }
            _ => {
                crate::SK_ABORT!(@source b"src/core/SkMaskBlurFilter.cpp\0".as_ptr(),1049; "Unhandled format.")
            }
        }
        let mut scan_h = plan_h.makeBlurScan(tmp_w, &mut buffer);
        for y in 0..tmp_h {
            let tmp_start = tmp.as_ptr().wrapping_offset((y * tmp_w) as isize);
            let dst_start = dst.fImage.cast_mut().wrapping_offset(y as isize);
            unsafe {
                scan_h.blur(
                    A8AlphaIter::new(tmp_start),
                    A8AlphaIter::new(tmp_start.wrapping_offset(tmp_w as isize)),
                    dst_start,
                    dst.fRowBytes as i32,
                    dst_start.wrapping_offset((dst.fRowBytes * dst_h as u32) as isize),
                );
            }
        }
        SkIPoint {
            fX: border_w,
            fY: border_h,
        }
    }
}

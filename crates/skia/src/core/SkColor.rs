// Copyright 2006 The Android Open Source Project. BSD-style license.
// Source: include/core/SkColor.h:27-27
// Source: include/core/SkColor.h:38-38
pub type SkAlpha = u8;
pub type SkColor = u32;
// Source: include/core/SkColor.h:50-53
pub const fn SkColorSetARGB(a: u32, r: u32, g: u32, b: u32) -> SkColor {
    // Source: include/core/SkColor.h:50-53
    // Original constexpr may form a constant only if the enabled assertion
    // does not reach its non-constexpr abort. Runtime failure keeps the real
    // Chromium abort producer; compile-time failure cannot execute that FFI.
    const fn constant_check(a: u32, r: u32, g: u32, b: u32) {
        #[cfg(sk_debug)]
        assert!(
            a <= 255 && r <= 255 && g <= 255 && b <= 255,
            "include/core/SkColor.h:51: check(a <= 255 && r <= 255 && g <= 255 && b <= 255)"
        );
    }
    fn runtime_check(a: u32, r: u32, g: u32, b: u32) {
        crate::SkASSERT!(@source b"include/core/SkColor.h\0".as_ptr(),51;
            @condition b"a <= 255 && r <= 255 && g <= 255 && b <= 255\0".as_ptr();
            a<=255&&r<=255&&g<=255&&b<=255);
    }
    unsafe {
        core::intrinsics::const_eval_select((a, r, g, b), constant_check, runtime_check);
    }
    (a << 24) | (r << 16) | (g << 8) | b
}
// Source: include/core/SkColor.h:58-58
// Source: include/core/SkColor.h:62-62
// Source: include/core/SkColor.h:66-66
// Source: include/core/SkColor.h:70-70
// Source: include/core/SkColor.h:74-74
pub const fn SkColorSetRGB(r: u32, g: u32, b: u32) -> SkColor {
    SkColorSetARGB(255, r, g, b)
}
pub const fn SkColorGetA(c: SkColor) -> u32 {
    (c >> 24) & 255
}
pub const fn SkColorGetR(c: SkColor) -> u32 {
    (c >> 16) & 255
}
pub const fn SkColorGetG(c: SkColor) -> u32 {
    (c >> 8) & 255
}
pub const fn SkColorGetB(c: SkColor) -> u32 {
    c & 255
}
// Source: include/core/SkColor.h:83-85
pub const fn SkColorSetA(c: SkColor, a: u32) -> SkColor {
    (c & 0x00ffffff) | (a << 24)
}
// Source: include/core/SkColor.h:90-90
// Source: include/core/SkColor.h:95-95
// Source: include/core/SkColor.h:100-100
// Source: include/core/SkColor.h:104-104
// Source: include/core/SkColor.h:109-109
// Source: include/core/SkColor.h:114-114
// Source: include/core/SkColor.h:119-119
// Source: include/core/SkColor.h:123-123
// Source: include/core/SkColor.h:127-127
// Source: include/core/SkColor.h:132-132
// Source: include/core/SkColor.h:136-136
// Source: include/core/SkColor.h:140-140
// Source: include/core/SkColor.h:144-144
// Source: include/core/SkColor.h:148-148
pub const SK_AlphaTRANSPARENT: SkAlpha = 0;
pub const SK_AlphaOPAQUE: SkAlpha = 255;
pub const SK_ColorTRANSPARENT: SkColor = 0;
pub const SK_ColorBLACK: SkColor = 0xff000000;
pub const SK_ColorDKGRAY: SkColor = 0xff444444;
pub const SK_ColorGRAY: SkColor = 0xff888888;
pub const SK_ColorLTGRAY: SkColor = 0xffcccccc;
pub const SK_ColorWHITE: SkColor = 0xffffffff;
pub const SK_ColorRED: SkColor = 0xffff0000;
pub const SK_ColorGREEN: SkColor = 0xff00ff00;
pub const SK_ColorBLUE: SkColor = 0xff0000ff;
pub const SK_ColorYELLOW: SkColor = 0xffffff00;
pub const SK_ColorCYAN: SkColor = 0xff00ffff;
pub const SK_ColorMAGENTA: SkColor = 0xffff00ff;
// Source: include/core/SkColor.h:170-172
pub fn SkColorToHSV(c: SkColor, hsv: &mut [f32; 3]) {
    SkRGBToHSV(SkColorGetR(c), SkColorGetG(c), SkColorGetB(c), hsv)
}
// Source: include/core/SkColor.h:197-199
// Rust has no overloads; the opaque variant receives a distinct name.
pub fn SkHSVToColorOpaque(hsv: &[f32; 3]) -> SkColor {
    SkHSVToColor(255, hsv)
}
// Source: include/core/SkColor.h:206-206
pub type SkPMColor = u32;
// Source: include/core/SkColor.h:229-233
// Source: include/core/SkColor.h:235-236
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SkColorChannel {
    kR,
    kG,
    kB,
    kA,
}
impl SkColorChannel {
    pub const kLastEnum: Self = Self::kA;
}
// Source: include/core/SkColor.h:239-244
// Source: include/core/SkColor.h:246-251
pub const kRed_SkColorChannelFlag: u32 = 1;
pub const kGreen_SkColorChannelFlag: u32 = 2;
pub const kBlue_SkColorChannelFlag: u32 = 4;
pub const kAlpha_SkColorChannelFlag: u32 = 8;
pub const kGray_SkColorChannelFlag: u32 = 16;
pub const kGrayAlpha_SkColorChannelFlags: u32 = 24;
pub const kRG_SkColorChannelFlags: u32 = 3;
pub const kRGB_SkColorChannelFlags: u32 = 7;
pub const kRGBA_SkColorChannelFlags: u32 = 15;
// Source: include/core/SkColor.h:262-267
// Rust const parameter carries premultiplication at the type level: true = premul.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SkRGBA4f<const PREMUL: bool> {
    pub fR: f32,
    pub fG: f32,
    pub fB: f32,
    pub fA: f32,
}
// Source: include/core/SkColor.h:274-276
// Source: include/core/SkColor.h:283-285
impl<const P: bool> PartialEq for SkRGBA4f<P> {
    fn eq(&self, other: &Self) -> bool {
        self.fA == other.fA && self.fR == other.fR && self.fG == other.fG && self.fB == other.fB
    }
}
// Source: include/core/SkColor.h:292-294
impl<const P: bool> std::ops::Mul<f32> for SkRGBA4f<P> {
    type Output = Self;
    fn mul(self, scale: f32) -> Self {
        Self {
            fR: self.fR * scale,
            fG: self.fG * scale,
            fB: self.fB * scale,
            fA: self.fA * scale,
        }
    }
}
// Source: include/core/SkColor.h:301-303
impl<const P: bool> std::ops::Mul for SkRGBA4f<P> {
    type Output = Self;
    fn mul(self, scale: Self) -> Self {
        Self {
            fR: self.fR * scale.fR,
            fG: self.fG * scale.fG,
            fB: self.fB * scale.fB,
            fA: self.fA * scale.fA,
        }
    }
}
impl<const P: bool> SkRGBA4f<P> {
    // Source: include/core/SkColor.h:309-309
    // Source: include/core/SkColor.h:315-315
    // Source: include/core/SkColor.h:318-318
    pub fn vec(&self) -> &[f32; 4] {
        // repr(C) four homogeneous fields preserve contiguous component access.
        unsafe { &*(std::ptr::addr_of!(self.fR).cast::<[f32; 4]>()) }
    }
    pub fn vec_mut(&mut self) -> &mut [f32; 4] {
        unsafe { &mut *(std::ptr::addr_of_mut!(self.fR).cast::<[f32; 4]>()) }
    }
    pub fn array(&self) -> [f32; 4] {
        [self.fR, self.fG, self.fB, self.fA]
    }
}
// Source: include/core/SkColor.h:325-328
// Source: include/core/SkColor.h:335-338
impl<const P: bool> std::ops::Index<usize> for SkRGBA4f<P> {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        crate::SkASSERT!(@source b"include/core/SkColor.h\0".as_ptr(),326; @condition b"index >= 0 && index < 4\0".as_ptr(); index<4);
        &self.vec()[index]
    }
}
impl<const P: bool> std::ops::IndexMut<usize> for SkRGBA4f<P> {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        crate::SkASSERT!(@source b"include/core/SkColor.h\0".as_ptr(),336; @condition b"index >= 0 && index < 4\0".as_ptr(); index<4);
        &mut self.vec_mut()[index]
    }
}
impl<const P: bool> SkRGBA4f<P> {
    // Source: include/core/SkColor.h:345-348
    // Source: include/core/SkColor.h:351-356
    pub fn isOpaque(&self) -> bool {
        crate::SkASSERT!(@source b"include/core/SkColor.h\0".as_ptr(),346; @condition b"fA <= 1.0f && fA >= 0.0f\0".as_ptr(); self.fA<=1.0&&self.fA>=0.0);
        self.fA == 1.0
    }
    pub fn fitsInBytes(&self) -> bool {
        crate::SkASSERT!(@source b"include/core/SkColor.h\0".as_ptr(),352; @condition b"fA >= 0.0f && fA <= 1.0f\0".as_ptr(); self.fA>=0.0&&self.fA<=1.0);
        self.fR >= 0.0
            && self.fR <= 1.0
            && self.fG >= 0.0
            && self.fG <= 1.0
            && self.fB >= 0.0
            && self.fB <= 1.0
    }
}
// Source: include/core/SkColor.h:386-389
impl SkRGBA4f<false> {
    pub fn premul(&self) -> SkRGBA4f<true> {
        SkRGBA4f {
            fR: self.fR * self.fA,
            fG: self.fG * self.fA,
            fB: self.fB * self.fA,
            fA: self.fA,
        }
    }
}
// Source: include/core/SkColor.h:396-397
// Source: include/core/SkColor.h:399-405
impl SkRGBA4f<true> {
    pub fn unpremul(&self) -> SkRGBA4f<false> {
        if self.fA == 0.0 {
            SkRGBA4f {
                fR: 0.0,
                fG: 0.0,
                fB: 0.0,
                fA: 0.0,
            }
        } else {
            let invAlpha = 1.0 / self.fA;
            SkRGBA4f {
                fR: self.fR * invAlpha,
                fG: self.fG * invAlpha,
                fB: self.fB * invAlpha,
                fA: self.fA,
            }
        }
    }
}
impl<const P: bool> SkRGBA4f<P> {
    // Source: include/core/SkColor.h:416-418
    pub fn makeOpaque(&self) -> Self {
        Self { fA: 1.0, ..*self }
    }
    // Source: include/core/SkColor.h:425-427
    pub fn pinAlpha(&self) -> Self {
        let a = self.fA;
        Self {
            fA: if !(0.0 < a) {
                0.0
            } else if 1.0 < a {
                1.0
            } else {
                a
            },
            ..*self
        }
    }
    // Source: include/core/SkColor.h:431-433
    // Source: include/core/SkColor.h:437-439
    pub fn withAlpha(&self, a: f32) -> Self {
        Self { fA: a, ..*self }
    }
    pub fn withAlphaByte(&self, a: u8) -> Self {
        Self {
            fA: a as f32 / 255.0,
            ..*self
        }
    }
}
// Source: include/core/SkColor.h:448-448
pub type SkColor4f = SkRGBA4f<false>;
use super::SkColorData::SkPMColor4f;
// Source: include/core/SkColor.h:455-468
pub mod SkColors {
    use super::SkColor4f;
    pub const kTransparent: SkColor4f = SkColor4f {
        fR: 0.0,
        fG: 0.0,
        fB: 0.0,
        fA: 0.0,
    };
    pub const kBlack: SkColor4f = SkColor4f {
        fR: 0.0,
        fG: 0.0,
        fB: 0.0,
        fA: 1.0,
    };
    pub const kDkGray: SkColor4f = SkColor4f {
        fR: 0.25,
        fG: 0.25,
        fB: 0.25,
        fA: 1.0,
    };
    pub const kGray: SkColor4f = SkColor4f {
        fR: 0.5,
        fG: 0.5,
        fB: 0.5,
        fA: 1.0,
    };
    pub const kLtGray: SkColor4f = SkColor4f {
        fR: 0.75,
        fG: 0.75,
        fB: 0.75,
        fA: 1.0,
    };
    pub const kWhite: SkColor4f = SkColor4f {
        fR: 1.0,
        fG: 1.0,
        fB: 1.0,
        fA: 1.0,
    };
    pub const kRed: SkColor4f = SkColor4f {
        fR: 1.0,
        fG: 0.0,
        fB: 0.0,
        fA: 1.0,
    };
    pub const kGreen: SkColor4f = SkColor4f {
        fR: 0.0,
        fG: 1.0,
        fB: 0.0,
        fA: 1.0,
    };
    pub const kBlue: SkColor4f = SkColor4f {
        fR: 0.0,
        fG: 0.0,
        fB: 1.0,
        fA: 1.0,
    };
    pub const kYellow: SkColor4f = SkColor4f {
        fR: 1.0,
        fG: 1.0,
        fB: 0.0,
        fA: 1.0,
    };
    pub const kCyan: SkColor4f = SkColor4f {
        fR: 0.0,
        fG: 1.0,
        fB: 1.0,
        fA: 1.0,
    };
    pub const kMagenta: SkColor4f = SkColor4f {
        fR: 1.0,
        fG: 0.0,
        fB: 1.0,
        fA: 1.0,
    };
}
// Source: src/core/SkColor.cpp:18-20
// Source: src/core/SkColor.cpp:22-25
// Source: src/core/SkColor.cpp:27-29
// Source: src/core/SkColor.cpp:31-33
// Source: src/core/SkColor.cpp:35-37
// Source: src/core/SkColor.cpp:39-41
// Source: src/core/SkColor.cpp:43-45
// Source: include/core/SkColor.h:216-216
pub fn SkPreMultiplyARGB(a: u32, r: u32, g: u32, b: u32) -> SkPMColor {
    super::SkColorPriv::SkPremultiplyARGBInline(a, r, g, b)
}
// Source: include/core/SkColor.h:224-224
pub fn SkPreMultiplyColor(c: SkColor) -> SkPMColor {
    SkPreMultiplyARGB(
        SkColorGetA(c),
        SkColorGetR(c),
        SkColorGetG(c),
        SkColorGetB(c),
    )
}
pub fn SkPMColorSetARGB(a: u8, r: u8, g: u8, b: u8) -> SkPMColor {
    super::SkColorPriv::SkPackARGB32(a as u32, r as u32, g as u32, b as u32)
}
pub fn SkPMColorGetA(c: SkPMColor) -> u8 {
    (c >> 24) as u8
}
pub fn SkPMColorGetR(c: SkPMColor) -> u8 {
    (c >> 16) as u8
}
pub fn SkPMColorGetG(c: SkPMColor) -> u8 {
    (c >> 8) as u8
}
pub fn SkPMColorGetB(c: SkPMColor) -> u8 {
    c as u8
}
// Source: src/core/SkColor.cpp:49-52
// Source: src/core/SkColor.cpp:54-54
// Source: src/core/SkColor.cpp:56-57
fn ByteToScalar(x: u32) -> f32 {
    crate::SkASSERT!(@source b"src/core/SkColor.cpp\0".as_ptr(),50; @condition b"x <= 255\0".as_ptr(); x<=255);
    x as f32 / 255.0
}
fn ByteDivToScalar(numer: i32, denom: u32) -> f32 {
    numer as f32 / denom as i32 as f32
}
// Source: src/core/SkColor.cpp:59-60
// Source: src/core/SkColor.cpp:62-64
// Source: src/core/SkColor.cpp:66-67
// Source: src/core/SkColor.cpp:69-74
// Source: src/core/SkColor.cpp:76-77
// Source: src/core/SkColor.cpp:79-86
// Source: src/core/SkColor.cpp:88-92
// Source: src/core/SkColor.cpp:94-97
// Source: include/core/SkColor.h:160-160
pub fn SkRGBToHSV(r: u32, g: u32, b: u32, hsv: &mut [f32; 3]) {
    let min = r.min(g.min(b));
    let max = r.max(g.max(b));
    let delta = max - min;
    let v = ByteToScalar(max);
    crate::SkASSERT!(@source b"src/core/SkColor.cpp\0".as_ptr(),67; @condition b"v >= 0 && v <= SK_Scalar1\0".as_ptr(); v>=0.0&&v<=1.0);
    if delta == 0 {
        *hsv = [0.0, 0.0, v];
        return;
    }
    let s = ByteDivToScalar(delta as i32, max);
    crate::SkASSERT!(@source b"src/core/SkColor.cpp\0".as_ptr(),77; @condition b"s >= 0 && s <= SK_Scalar1\0".as_ptr(); s>=0.0&&s<=1.0);
    let mut h = if r == max {
        ByteDivToScalar(g.wrapping_sub(b) as i32, delta)
    } else if g == max {
        2.0 + ByteDivToScalar(b.wrapping_sub(r) as i32, delta)
    } else {
        4.0 + ByteDivToScalar(r.wrapping_sub(g) as i32, delta)
    };
    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    }
    crate::SkASSERT!(@source b"src/core/SkColor.cpp\0".as_ptr(),92; @condition b"h >= 0 && h < SkIntToScalar(360)\0".as_ptr(); h>=0.0&&h<360.0);
    *hsv = [h, s, v];
}
// Source: src/core/SkColor.cpp:99-100
// Source: src/core/SkColor.cpp:102-103
// Source: src/core/SkColor.cpp:105-105
// Source: src/core/SkColor.cpp:107-112
// Source: src/core/SkColor.cpp:114-116
// Source: src/core/SkColor.cpp:118-118
// Source: src/core/SkColor.cpp:120-130
// Source: include/core/SkColor.h:185-185
pub fn SkHSVToColor(a: u32, hsv: &[f32; 3]) -> SkColor {
    let s = crate::include::private::SkTPin::SkTPin(hsv[1], 0.0, 1.0);
    let v = crate::include::private::SkTPin::SkTPin(hsv[2], 0.0, 1.0);
    let round = |x: f32| crate::include::core::SkScalar::SkScalarRoundToInt(x) as u32;
    let v_byte = round(v * 255.0);
    if crate::include::core::SkScalar::SkScalarNearlyZero(
        s,
        crate::include::core::SkScalar::SK_ScalarNearlyZero,
    ) {
        return SkColorSetARGB(a, v_byte, v_byte, v_byte);
    }
    let hx = if hsv[0] < 0.0 || hsv[0] >= 360.0 {
        0.0
    } else {
        hsv[0] / 60.0
    };
    let w = crate::include::core::SkScalar::SkScalarFloorToScalar(hx);
    let f = hx - w;
    let p = round((1.0 - s) * v * 255.0);
    let q = round((1.0 - s * f) * v * 255.0);
    let t = round((1.0 - s * (1.0 - f)) * v * 255.0);
    crate::SkASSERT!(@source b"src/core/SkColor.cpp\0".as_ptr(),120; @condition b"(unsigned)(w) < 6\0".as_ptr(); (w as u32)<6);
    let (r, g, b) = match w as u32 {
        0 => (v_byte, t, p),
        1 => (q, v_byte, p),
        2 => (p, v_byte, t),
        3 => (p, q, v_byte),
        4 => (t, p, v_byte),
        _ => (v_byte, p, q),
    };
    SkColorSetARGB(a, r, g, b)
}

// Source: src/core/SkColor.cpp:134-139
// Source: src/core/SkColor.cpp:141-144
impl SkColor4f {
    // Source: include/core/SkColor.h:365-365
    // Source: include/core/SkColor.h:450-450
    pub fn FromColor(bgra: SkColor) -> Self {
        let c = super::SkSwizzlePriv::swizzle_rb(super::SkSwizzlePriv::Sk4f_fromL32(bgra));
        Self {
            fR: c[0],
            fG: c[1],
            fB: c[2],
            fA: c[3],
        }
    }
    // Source: include/core/SkColor.h:373-373
    // Source: include/core/SkColor.h:451-451
    pub fn toSkColor(&self) -> SkColor {
        super::SkSwizzlePriv::Sk4f_toL32(super::SkSwizzlePriv::swizzle_rb(self.array()))
    }
}
// Source: src/core/SkColor.cpp:146-149
// Source: src/core/SkColor.cpp:151-156
impl SkColor4f {
    // Source: include/core/SkColor.h:452-452
    // Source: include/core/SkColor.h:408-408
    pub fn toBytes_RGBA(&self) -> u32 {
        super::SkSwizzlePriv::Sk4f_toL32(self.array())
    }
    // Source: include/core/SkColor.h:453-453
    // Source: include/core/SkColor.h:409-409
    pub fn FromBytes_RGBA(c: u32) -> Self {
        let c = super::SkSwizzlePriv::Sk4f_fromL32(c);
        Self {
            fR: c[0],
            fG: c[1],
            fB: c[2],
            fA: c[3],
        }
    }
}
// Source: src/core/SkColor.cpp:158-163
// Source: src/core/SkColor.cpp:165-168
// Source: src/core/SkColor.cpp:170-175
impl SkPMColor4f {
    // Source: include/core/SkColor.h:379-379
    pub fn FromPMColor(c: SkPMColor) -> Self {
        let c = super::SkSwizzlePriv::swizzle_rb_if_bgra(super::SkSwizzlePriv::Sk4f_fromL32(c));
        Self {
            fR: c[0],
            fG: c[1],
            fB: c[2],
            fA: c[3],
        }
    }
    // Source: include/core/SkColor.h:408-408
    pub fn toBytes_RGBA(&self) -> u32 {
        super::SkSwizzlePriv::Sk4f_toL32(self.array())
    }
    // Source: include/core/SkColor.h:409-409
    pub fn FromBytes_RGBA(c: u32) -> Self {
        let c = super::SkSwizzlePriv::Sk4f_fromL32(c);
        Self {
            fR: c[0],
            fG: c[1],
            fB: c[2],
            fA: c[3],
        }
    }
}
// Source: src/core/SkColor.cpp:177-182
impl super::SkColorPriv::SkColorConverter {
    pub fn new(src: &[SkColor]) -> Self {
        Self {
            fColors4f: src.iter().map(|&c| SkColor4f::FromColor(c)).collect(),
        }
    }
}

// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::private::SkFixed::{SkFixed, SkFixedMul};
use crate::src::core::SkFDot6::*;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkIRect;
// Source: src/core/SkEdge.cpp:50-51
// UNVERIFIED native scaled fixed-point type aliases; no numerical conversion.
type SkFDot6Scaled = i32;
type SkFixedScaled = i32;

// Source: src/core/SkEdge.h:24-24
#[inline]
fn SkEdge_Compute_DY(top: i32, y0: i32) -> i32 {
    top.wrapping_shl(6).wrapping_add(32).wrapping_sub(y0)
}
// Source: src/core/SkEdge.h:41-41
// Retain nontrivial destruction for arena footer selection, even though the
// original virtual base destructor has no side effects.
impl Drop for SkEdge {
    fn drop(&mut self) {}
}
// Source: src/core/SkEdge.h:42-50
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum Type {
    kLine,
    kQuad,
    kCubic,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum Winding {
    kCW = 1,
    kCCW = -1,
}
// Source: src/core/SkEdge.h:92-92
// Source: src/core/SkEdge.h:95-95
// Existing segment-count/curve-shift fields are kept in the contiguous Rust field group.
// Source: src/core/SkEdge.h:53-68
#[repr(C)]
pub struct SkEdge {
    pub fNext: *mut SkEdge,
    pub fPrev: *mut SkEdge,
    pub fX: i32,
    pub fDxDy: i32,
    pub fFirstY: i32,
    pub fLastY: i32,
    pub fEdgeType: Type,
    pub fWinding: Winding,
    pub fSegmentCount: u8,
    pub fCurveShift: u8,
}
impl SkEdge {
    pub fn new() -> Self {
        Self {
            fNext: core::ptr::null_mut(),
            fPrev: core::ptr::null_mut(),
            fX: 0,
            fDxDy: 0,
            fFirstY: 0,
            fLastY: 0,
            fEdgeType: Type::kLine,
            fWinding: Winding::kCW,
            fSegmentCount: 0,
            fCurveShift: 0,
        }
    }
    // Source: src/core/SkEdge.h:76-78
    pub fn hasNextSegment(&self) -> bool {
        self.fSegmentCount != 0
    }
    // Source: src/core/SkEdge.h:85-87
    pub fn segmentsLeft(&self) -> u8 {
        self.fSegmentCount
    }
}
// Source: src/core/SkEdge.h:122-134
#[repr(C)]
pub struct SkQuadraticEdge {
    pub edge: SkEdge,
    fQx: i32,
    fQy: i32,
    fQDxDt: i32,
    fQDyDt: i32,
    fQD2xDt2: i32,
    fQD2yDt2: i32,
    fQLastX: i32,
    fQLastY: i32,
}
// Source: src/core/SkEdge.h:150-160
#[repr(C)]
pub struct SkCubicEdge {
    pub edge: SkEdge,
    fCx: i32,
    fCy: i32,
    fCDxDt: SkFixedScaled,
    fCDyDt: SkFixedScaled,
    fCD2xDt2: i32,
    fCD2yDt2: i32,
    fCD3xDt3: i32,
    fCD3yDt3: i32,
    fCLastX: i32,
    fCLastY: i32,
    fToFixedShift: u8,
}
impl SkQuadraticEdge {
    pub fn new() -> Self {
        Self {
            edge: SkEdge::new(),
            fQx: 0,
            fQy: 0,
            fQDxDt: 0,
            fQDyDt: 0,
            fQD2xDt2: 0,
            fQD2yDt2: 0,
            fQLastX: 0,
            fQLastY: 0,
        }
    }
}
impl SkCubicEdge {
    pub fn new() -> Self {
        Self {
            edge: SkEdge::new(),
            fCx: 0,
            fCy: 0,
            fCDxDt: 0,
            fCDyDt: 0,
            fCD2xDt2: 0,
            fCD2yDt2: 0,
            fCD3xDt3: 0,
            fCD3yDt3: 0,
            fCLastX: 0,
            fCLastY: 0,
            fToFixedShift: 0,
        }
    }
}
impl SkEdge {
    // Source: src/core/SkEdge.h:103-110
    /// Safety: both adjacent links denote live linked edges; each neighbor's
    /// links may be read without concurrent mutation for this operation.
    pub unsafe fn validate(&self) {
        crate::SkASSERT!(@source b"src/core/SkEdge.h\0".as_ptr(), 104; @condition b"fPrev && fNext\0".as_ptr(); !self.fPrev.is_null() && !self.fNext.is_null());
        crate::SkASSERT!(@source b"src/core/SkEdge.h\0".as_ptr(), 105; @condition b"fPrev->fNext == this\0".as_ptr(); (unsafe { (*self.fPrev).fNext }) == ( self as *const Self as *mut Self));
        crate::SkASSERT!(@source b"src/core/SkEdge.h\0".as_ptr(), 106; @condition b"fNext->fPrev == this\0".as_ptr(); (unsafe { (*self.fNext).fPrev }) == ( self as *const Self as *mut Self));
        crate::SkASSERT!(@source b"src/core/SkEdge.h\0".as_ptr(), 108; @condition b"fFirstY <= fLastY\0".as_ptr(); self.fFirstY <= self.fLastY);
        crate::SkASSERT!(@source b"src/core/SkEdge.h\0".as_ptr(), 109; @condition b"fWinding == Winding::kCW || fWinding == Winding::kCCW\0".as_ptr(); self.fWinding == Winding::kCW || self.fWinding == Winding::kCCW);
    }

    // Source: src/core/SkEdge.h:74-74
    // Source: src/core/SkEdge.h:168-213
    /// Native unclipped overload. EVEN selects the same original build-time
    /// quantization branch used by the existing clipped overload.
    pub fn setLineUnclipped<const EVEN: bool>(&mut self, p0: &SkPoint, p1: &SkPoint) -> bool {
        let (mut x0, mut y0, mut x1, mut y1) = (
            quantize::<EVEN>(p0.fX),
            quantize::<EVEN>(p0.fY),
            quantize::<EVEN>(p1.fX),
            quantize::<EVEN>(p1.fY),
        );
        let mut winding = Winding::kCW;
        if y0 > y1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
            winding = Winding::kCCW;
        }
        let top = SkFDot6Round(y0);
        let bot = SkFDot6Round(y1);
        if top == bot {
            return false;
        }
        let slope = SkFDot6Div(x1.wrapping_sub(x0), y1.wrapping_sub(y0));
        let dy = SkEdge_Compute_DY(top, y0);
        self.fX = SkFDot6ToFixed(x0.wrapping_add(SkFixedMul(slope, dy)));
        self.fDxDy = slope;
        self.fFirstY = top;
        self.fLastY = bot - 1;
        self.fEdgeType = Type::kLine;
        self.fSegmentCount = 0;
        self.fWinding = winding;
        self.fCurveShift = 0;
        true
    }
}

// Source: src/core/SkEdge.cpp:53-57
fn SkFDot6ToFixedDiv2(value: i32) -> SkFixed {
    value.wrapping_shl(9)
}
// The upstream build-time even-rounding branch is a const generic, allowing both
// exact quantization paths without imposing a package-wide compile feature.
fn quantize<const EVEN: bool>(x: f32) -> i32 {
    if EVEN {
        SkScalarRoundToFDot6(x, 0)
    } else {
        SkFloatToFDot6(x)
    }
}
// Rust adaptation of SkDebugf output: libc formats the original %g/%d/%u
// conversion strings; the resulting bytes are written to the debug stderr sink.
#[cfg(sk_debug)]
extern "C" {
    fn snprintf(
        dst: *mut core::ffi::c_char,
        n: usize,
        format: *const core::ffi::c_char,
        ...
    ) -> core::ffi::c_int;
}
// Source: src/core/SkEdge.h:102-102
// Source: src/core/SkEdge.cpp:62-71
#[cfg(sk_debug)]
impl SkEdge {
    pub fn dump(&self) {
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 63; @condition b"fSegmentCount == 0\0".as_ptr(); (self.fSegmentCount) == (0));
        let mut buffer = [0u8; 2048];
        let n = unsafe {
            snprintf(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                b"line edge: firstY:%d lastY:%d x:%g dx/dy:%g\n\twinding:%d curveShift:%u\n\0"
                    .as_ptr()
                    .cast(),
                self.fFirstY,
                self.fLastY,
                (self.fX as f32 * (1.0f32 / 65536.0)) as f64,
                (self.fDxDy as f32 * (1.0f32 / 65536.0)) as f64,
                self.fWinding as i8 as i32,
                self.fCurveShift as u32,
            )
        };
        if n > 0 {
            use std::io::Write;
            let _ = std::io::stderr().write_all(&buffer[..(n as usize).min(buffer.len() - 1)]);
        }
    }
}
// Source: src/core/SkEdge.h:138-138
// Source: src/core/SkEdge.cpp:74-92
#[cfg(sk_debug)]
impl SkQuadraticEdge {
    pub fn dump(&self) {
        let mut buffer = [0u8; 2048];
        let n = unsafe {
            snprintf(buffer.as_mut_ptr().cast(),buffer.len(),b"quad edge; %u segment(s) left: firstY:%d lastY:%d x:%g dx/dy:%g\n\tqx:%g qy:%g dqx:%g dqy:%g ddqx:%g ddqy:%g qLastX:%g qLastY:%g\n\twinding:%d curveShift:%u\n\0".as_ptr().cast(),self.edge.fSegmentCount as u32,self.edge.fFirstY,self.edge.fLastY,(self.edge.fX as f32 * (1.0f32/65536.0)) as f64,(self.edge.fDxDy as f32 * (1.0f32/65536.0)) as f64,(self.fQx as f32 * (1.0f32/65536.0)) as f64,(self.fQy as f32 * (1.0f32/65536.0)) as f64,(self.fQDxDt as f32 * (1.0f32/65536.0)) as f64,(self.fQDyDt as f32 * (1.0f32/65536.0)) as f64,(self.fQD2xDt2 as f32 * (1.0f32/65536.0)) as f64,(self.fQD2yDt2 as f32 * (1.0f32/65536.0)) as f64,(self.fQLastX as f32 * (1.0f32/65536.0)) as f64,(self.fQLastY as f32 * (1.0f32/65536.0)) as f64,self.edge.fWinding as i8 as i32,self.edge.fCurveShift as u32)
        };
        if n > 0 {
            use std::io::Write;
            let _ = std::io::stderr().write_all(&buffer[..(n as usize).min(buffer.len() - 1)]);
        }
    }
}
// Source: src/core/SkEdge.h:164-164
// Source: src/core/SkEdge.cpp:95-116
#[cfg(sk_debug)]
impl SkCubicEdge {
    pub fn dump(&self) {
        let mut buffer = [0u8; 2048];
        let n = unsafe {
            snprintf(buffer.as_mut_ptr().cast(),buffer.len(),b"cube edge; %u segment(s) left: firstY:%d lastY:%d x:%g dx/dy:%g\nqx:%g qy:%g dcx:%g dcy:%g ddcx:%g ddcy:%g dddcx:%g dddcy:%g cLastX:%g cLastY:%g\n\twinding:%d curveShift:%u toFixedShift:%u\n\0".as_ptr().cast(),self.edge.fSegmentCount as u32,self.edge.fFirstY,self.edge.fLastY,(self.edge.fX as f32 * (1.0f32/65536.0)) as f64,(self.edge.fDxDy as f32 * (1.0f32/65536.0)) as f64,(self.fCx as f32 * (1.0f32/65536.0)) as f64,(self.fCy as f32 * (1.0f32/65536.0)) as f64,(self.fCDxDt as f32 * (1.0f32/65536.0)) as f64,(self.fCDyDt as f32 * (1.0f32/65536.0)) as f64,(self.fCD2xDt2 as f32 * (1.0f32/65536.0)) as f64,(self.fCD2yDt2 as f32 * (1.0f32/65536.0)) as f64,(self.fCD3xDt3 as f32 * (1.0f32/65536.0)) as f64,(self.fCD3yDt3 as f32 * (1.0f32/65536.0)) as f64,(self.fCLastX as f32 * (1.0f32/65536.0)) as f64,(self.fCLastY as f32 * (1.0f32/65536.0)) as f64,self.edge.fWinding as i8 as i32,self.edge.fCurveShift as u32,self.fToFixedShift as u32)
        };
        if n > 0 {
            use std::io::Write;
            let _ = std::io::stderr().write_all(&buffer[..(n as usize).min(buffer.len() - 1)]);
        }
    }
}
impl SkEdge {
    // Source: src/core/SkEdge.cpp:120-171
    pub fn setLine<const EVEN: bool>(
        &mut self,
        p0: &SkPoint,
        p1: &SkPoint,
        clip: Option<&SkIRect>,
    ) -> bool {
        let (mut x0, mut y0, mut x1, mut y1) = (
            quantize::<EVEN>(p0.fX),
            quantize::<EVEN>(p0.fY),
            quantize::<EVEN>(p1.fX),
            quantize::<EVEN>(p1.fY),
        );
        let mut winding = Winding::kCW;
        if y0 > y1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
            winding = Winding::kCCW;
        }
        let top = SkFDot6Round(y0);
        let bot = SkFDot6Round(y1);
        if top == bot {
            return false;
        }
        if let Some(clip) = clip {
            if top >= clip.fBottom || bot <= clip.fTop {
                return false;
            }
        }
        let slope = SkFDot6Div(x1.wrapping_sub(x0), y1.wrapping_sub(y0));
        let dy = SkEdge_Compute_DY(top, y0);
        self.fX = SkFDot6ToFixed(x0.wrapping_add(SkFixedMul(slope, dy)));
        self.fDxDy = slope;
        self.fFirstY = top;
        self.fLastY = bot - 1;
        self.fEdgeType = Type::kLine;
        self.fSegmentCount = 0;
        self.fWinding = winding;
        self.fCurveShift = 0;
        if let Some(clip) = clip {
            self.chopLineWithClip(clip);
        }
        true
    }
    // Source: src/core/SkEdge.cpp:173-176
    pub fn nextSegment(&mut self) -> bool {
        crate::SkDEBUGFAILF!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 174; "Shouldn't be asking a linear edge to go to the next curve.");
        false
    }
    // Source: src/core/SkEdge.cpp:182-214
    pub fn updateLine(&mut self, x_start: i32, y_start: i32, x_end: i32, y_end: i32) -> bool {
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 184; @condition b"fSegmentCount != 0\0".as_ptr(); self.fSegmentCount != 0);
        let y0 = SkFixedToFDot6(y_start);
        let y1 = SkFixedToFDot6(y_end);
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 189; @condition b"y0 <= y1\0".as_ptr(); y0 <= y1);
        let top = SkFDot6Round(y0);
        let bot = SkFDot6Round(y1);
        if top == bot {
            return false;
        }
        let x0 = SkFixedToFDot6(x_start);
        let x1 = SkFixedToFDot6(x_end);
        let slope = SkFDot6Div(x1.wrapping_sub(x0), y1.wrapping_sub(y0));
        let dy = SkEdge_Compute_DY(top, y0);
        self.fX = SkFDot6ToFixed(x0.wrapping_add(SkFixedMul(slope, dy)));
        self.fDxDy = slope;
        self.fFirstY = top;
        self.fLastY = bot - 1;
        true
    }
    // Source: src/core/SkEdge.cpp:216-229
    fn chopLineWithClip(&mut self, clip: &SkIRect) {
        let top = self.fFirstY;
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 220; @condition b"top < clip.fBottom\0".as_ptr(); top < clip.fBottom);
        if top < clip.fTop {
            crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 225; @condition b"fLastY >= clip.fTop\0".as_ptr(); self.fLastY >= clip.fTop);
            self.fX = self
                .fX
                .wrapping_add(self.fDxDy.wrapping_mul(clip.fTop - top));
            self.fFirstY = clip.fTop;
        }
    }
}
// Source: src/core/SkEdge.cpp:237-237
const MAX_COEFF_SHIFT: i32 = 6;
// Source: src/core/SkEdge.cpp:245-253
fn cheap_distance(mut dx: i32, mut dy: i32) -> i32 {
    dx = dx.wrapping_abs();
    dy = dy.wrapping_abs();
    if dx > dy {
        dx.wrapping_add(dy / 2)
    } else {
        dy.wrapping_add(dx / 2)
    }
}
// Source: src/core/SkEdge.cpp:255-270
fn diff_to_steps(dx: i32, dy: i32, accuracy: i32) -> i32 {
    let mut dist = cheap_distance(dx, dy);
    dist = dist.wrapping_add(1 << (2 + accuracy)) >> (3 + accuracy);
    (32 - (dist as u32).leading_zeros() as i32) >> 1
}
impl SkQuadraticEdge {
    // Source: src/core/SkEdge.cpp:272-387
    pub fn setQuadratic<const EVEN: bool>(&mut self, pts: &[SkPoint; 3]) -> bool {
        let (mut x0, mut y0, x1, y1, mut x2, mut y2) = (
            quantize::<EVEN>(pts[0].fX),
            quantize::<EVEN>(pts[0].fY),
            quantize::<EVEN>(pts[1].fX),
            quantize::<EVEN>(pts[1].fY),
            quantize::<EVEN>(pts[2].fX),
            quantize::<EVEN>(pts[2].fY),
        );
        let mut winding = Winding::kCW;
        if y0 > y2 {
            std::mem::swap(&mut x0, &mut x2);
            std::mem::swap(&mut y0, &mut y2);
            winding = Winding::kCCW;
        }
        crate::SkASSERTF!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 297; @condition b"y0 <= y1 && y1 <= y2\0".as_ptr(); y0 <= y1 && y1 <= y2, "curve must be monotonic");
        let top = SkFDot6Round(y0);
        let bot = SkFDot6Round(y2);
        if top == bot {
            return false;
        }
        let delta_x = (2 * x1 - x0 - x2) >> 2;
        let delta_y = (2 * y1 - y0 - y2) >> 2;
        let mut shift = diff_to_steps(delta_x, delta_y, 0);
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 324; @condition b"shift >= 0\0".as_ptr(); shift >= 0);
        if shift == 0 {
            shift = 1;
        } else if shift > MAX_COEFF_SHIFT {
            shift = MAX_COEFF_SHIFT;
        }
        self.edge.fWinding = winding;
        self.edge.fEdgeType = Type::kQuad;
        self.edge.fSegmentCount = (1 << shift) as u8;
        self.edge.fCurveShift = (shift - 1) as u8;
        let mut a_half = SkFDot6ToFixedDiv2(x0 - x1 - x1 + x2);
        let mut b_half = SkFDot6ToFixed(x1 - x0);
        self.fQDxDt = b_half.wrapping_add(a_half >> shift);
        self.fQD2xDt2 = a_half >> (shift - 1);
        a_half = SkFDot6ToFixedDiv2(y0 - y1 - y1 + y2);
        b_half = SkFDot6ToFixed(y1 - y0);
        self.fQDyDt = b_half.wrapping_add(a_half >> shift);
        self.fQD2yDt2 = a_half >> (shift - 1);
        self.fQx = SkFDot6ToFixed(x0);
        self.fQy = SkFDot6ToFixed(y0);
        self.fQLastX = SkFDot6ToFixed(x2);
        self.fQLastY = SkFDot6ToFixed(y2);
        self.nextSegment()
    }
    // Source: src/core/SkEdge.cpp:389-424
    pub fn nextSegment(&mut self) -> bool {
        let mut count = self.edge.fSegmentCount;
        let mut old_x = self.fQx;
        let mut old_y = self.fQy;
        let mut dx = self.fQDxDt;
        let mut dy = self.fQDyDt;
        let shift = self.edge.fCurveShift;
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 399; @condition b"count > 0\0".as_ptr(); count > 0);
        let (new_x, new_y, success) = loop {
            count -= 1;
            let (new_x, new_y) = if count > 0 {
                let x = old_x.wrapping_add(dx >> shift);
                dx = dx.wrapping_add(self.fQD2xDt2);
                let y = old_y.wrapping_add(dy >> shift);
                dy = dy.wrapping_add(self.fQD2yDt2);
                (x, y)
            } else {
                (self.fQLastX, self.fQLastY)
            };
            let success = self.edge.updateLine(old_x, old_y, new_x, new_y);
            old_x = new_x;
            old_y = new_y;
            if count == 0 || success {
                break (new_x, new_y, success);
            }
        };
        self.fQx = new_x;
        self.fQy = new_y;
        self.fQDxDt = dx;
        self.fQDyDt = dy;
        self.edge.fSegmentCount = count;
        success
    }
}
// Source: src/core/SkEdge.cpp:428-431
fn SkFDot6UpShift(x: i32, up_shift: i32) -> i32 {
    debug_assert_eq!(x.wrapping_shl(up_shift as u32) >> up_shift, x);
    x.wrapping_shl(up_shift as u32)
}
// Source: src/core/SkEdge.cpp:441-448
fn cubic_delta_from_line(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let one_third = (a * 8 - b * 15 + 6 * c + d) * 19 >> 9;
    let two_third = (a + 6 * b - c * 15 + d * 8) * 19 >> 9;
    one_third.wrapping_abs().max(two_third.wrapping_abs())
}
impl SkCubicEdge {
    // Source: src/core/SkEdge.cpp:450-601
    pub fn setCubic<const EVEN: bool>(&mut self, pts: &[SkPoint; 4]) -> bool {
        let (mut x0, mut y0, mut x1, mut y1, mut x2, mut y2, mut x3, mut y3) = (
            quantize::<EVEN>(pts[0].fX),
            quantize::<EVEN>(pts[0].fY),
            quantize::<EVEN>(pts[1].fX),
            quantize::<EVEN>(pts[1].fY),
            quantize::<EVEN>(pts[2].fX),
            quantize::<EVEN>(pts[2].fY),
            quantize::<EVEN>(pts[3].fX),
            quantize::<EVEN>(pts[3].fY),
        );
        let mut winding = Winding::kCW;
        if y0 > y3 {
            std::mem::swap(&mut x0, &mut x3);
            std::mem::swap(&mut x1, &mut x2);
            std::mem::swap(&mut y0, &mut y3);
            std::mem::swap(&mut y1, &mut y2);
            winding = Winding::kCCW;
        }
        let top = SkFDot6Round(y0);
        let bot = SkFDot6Round(y3);
        if top == bot {
            return false;
        }
        let dx = cubic_delta_from_line(x0, x1, x2, x3);
        let dy = cubic_delta_from_line(y0, y1, y2, y3);
        let mut step_exponent = diff_to_steps(dx, dy, 2) + 1;
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 499; @condition b"stepExponent > 0\0".as_ptr(); step_exponent > 0);
        if step_exponent > MAX_COEFF_SHIFT {
            step_exponent = MAX_COEFF_SHIFT;
        }
        let mut precision_up_scale = 6;
        let mut to_fixed_shift = step_exponent + precision_up_scale - 10;
        if to_fixed_shift < 0 {
            to_fixed_shift = 0;
            precision_up_scale = 10 - step_exponent;
        }
        self.edge.fWinding = winding;
        self.edge.fEdgeType = Type::kCubic;
        self.edge.fSegmentCount = (1 << step_exponent) as u8;
        self.edge.fCurveShift = step_exponent as u8;
        self.fToFixedShift = to_fixed_shift as u8;
        let mut a_scaled: SkFDot6Scaled =
            SkFDot6UpShift(x3 + 3 * (x1 - x2) - x0, precision_up_scale);
        let mut b_scaled: SkFDot6Scaled =
            SkFDot6UpShift(3 * (x0 - 2 * x1 + x2), precision_up_scale);
        let mut c_scaled: SkFDot6Scaled = SkFDot6UpShift(3 * (x1 - x0), precision_up_scale);
        self.fCDxDt = (a_scaled >> (2 * step_exponent)) + (b_scaled >> step_exponent) + c_scaled;
        self.fCD2xDt2 = (3 * a_scaled >> (step_exponent - 1)) + 2 * b_scaled;
        self.fCD3xDt3 = 3 * a_scaled >> (step_exponent - 1);
        a_scaled = SkFDot6UpShift(y3 + 3 * (y1 - y2) - y0, precision_up_scale);
        b_scaled = SkFDot6UpShift(3 * (y0 - 2 * y1 + y2), precision_up_scale);
        c_scaled = SkFDot6UpShift(3 * (y1 - y0), precision_up_scale);
        self.fCDyDt = (a_scaled >> (2 * step_exponent)) + (b_scaled >> step_exponent) + c_scaled;
        self.fCD2yDt2 = (3 * a_scaled >> (step_exponent - 1)) + 2 * b_scaled;
        self.fCD3yDt3 = 3 * a_scaled >> (step_exponent - 1);
        self.fCx = SkFDot6ToFixed(x0);
        self.fCy = SkFDot6ToFixed(y0);
        self.fCLastX = SkFDot6ToFixed(x3);
        self.fCLastY = SkFDot6ToFixed(y3);
        self.nextSegment()
    }
    // Source: src/core/SkEdge.cpp:603-663
    pub fn nextSegment(&mut self) -> bool {
        let mut count = self.edge.fSegmentCount;
        let mut old_x = self.fCx;
        let mut old_y = self.fCy;
        let step_exponent = self.edge.fCurveShift;
        let to_fixed_shift = self.fToFixedShift;
        crate::SkASSERT!(@source b"src/core/SkEdge.cpp\0".as_ptr(), 612; @condition b"count > 0\0".as_ptr(); count > 0);
        let (new_x, new_y, success) = loop {
            count -= 1;
            let (new_x, mut new_y) = if count > 0 {
                let new_x = old_x.wrapping_add(self.fCDxDt >> to_fixed_shift);
                self.fCDxDt = self.fCDxDt.wrapping_add(self.fCD2xDt2 >> step_exponent);
                self.fCD2xDt2 = self.fCD2xDt2.wrapping_add(self.fCD3xDt3);
                let new_y = old_y.wrapping_add(self.fCDyDt >> to_fixed_shift);
                self.fCDyDt = self.fCDyDt.wrapping_add(self.fCD2yDt2 >> step_exponent);
                self.fCD2yDt2 = self.fCD2yDt2.wrapping_add(self.fCD3yDt3);
                (new_x, new_y)
            } else {
                (self.fCLastX, self.fCLastY)
            };
            if new_y < old_y {
                new_y = old_y;
            }
            let success = self.edge.updateLine(old_x, old_y, new_x, new_y);
            old_x = new_x;
            old_y = new_y;
            if count == 0 || success {
                break (new_x, new_y, success);
            }
        };
        self.fCx = new_x;
        self.fCy = new_y;
        self.edge.fSegmentCount = count;
        success
    }
}

// Drawing safety: unsafe render entrypoints require the SkBlitter primitive
// contract: live exclusive destination/context/arena leases, valid bounds and
// initialized terminating sparse runs. Clip wrappers preserve this caller lease;
// clipping alone does not establish ownership or backing-storage lifetime.
// Copyright 2011 The Android Open Source Project. BSD-style license.
use crate::src::core::SkBlitter::SkBlitter;
use crate::src::core::SkColorPriv::SkAlphaMul;
use crate::src::core::SkRect::{SkIRect, SkRect};
use crate::src::core::SkRegion::{Cliperator, SkRegion};
use crate::src::core::SkScan::{SkScan, SkXRect, XRect_roundOut, XRect_set, XRect_set_i};
// Source: src/core/SkScan_Antihair.cpp:29-29
const HLINE_STACK_BUFFER: usize = 100;
// Source: src/core/SkScan_Antihair.cpp:31-35
fn scale_alpha_by_coverage(value: u32, coverage: i32) -> u32 {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 32; @condition b"value <= 255\0".as_ptr(); value <= 255);
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 33; @condition b"coverage >= 0 && coverage <= SK_FDot6One\0".as_ptr(); coverage >= 0 && coverage <= 64);
    (value * coverage as u32) >> 6
}
// Source: src/core/SkScan_Antihair.cpp:39-41
fn fixed_to_alpha(f: i32) -> u32 {
    ((f >> 8) & 0xff) as u32
}
// Source: src/core/SkScan_Antihair.cpp:43-68
unsafe fn call_hline_blitter(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    mut x: i32,
    y: i32,
    mut count: i32,
    alpha: u32,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 45; @condition b"count > 0\0".as_ptr(); count > 0);
    // Native sparse buffers only initialize visited slots. Rust slice validity
    // requires initialized storage; reseed the mutable run head on every call.
    let mut runs = [0i16; HLINE_STACK_BUFFER + 1];
    let mut aa = [0u8; HLINE_STACK_BUFFER];
    loop {
        aa[0] = alpha as u8;
        let mut n = count;
        if n > HLINE_STACK_BUFFER as i32 {
            n = HLINE_STACK_BUFFER as i32;
        }
        runs[0] = n as i16;
        runs[n as usize] = 0;
        blitter.as_mut().blitAntiH(x, y, &mut aa, &mut runs);
        x += n;
        count -= n;
        if count <= 0 {
            break;
        }
    }
}
// Source: src/core/SkScan_Antihair.cpp:74-90
// Rust uses one borrowing owner and a tag for the four original stack objects;
// the cap/line algorithms below preserve each concrete orientation branch.
#[derive(Clone, Copy)]
enum AntiHairKind {
    HLine,
    Horish,
    VLine,
    Vertish,
}
struct AntiHairBlitter<'a> {
    kind: AntiHairKind,
    blitter: core::pin::Pin<&'a mut dyn SkBlitter>,
}
// Source: src/core/SkScan_Antihair.cpp:95-96
// Source: src/core/SkScan_Antihair.cpp:97-120
unsafe fn hline_cap(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    x: i32,
    mut f: i32,
    slope: i32,
    coverage: i32,
) -> i32 {
    f = f.wrapping_add(32768);
    let y = f >> 16;
    let a = fixed_to_alpha(f);
    let mut ma = scale_alpha_by_coverage(a, coverage);
    if ma != 0 {
        call_hline_blitter(blitter.as_mut(), x, y, 1, ma);
    }
    ma = scale_alpha_by_coverage(255 - a, coverage);
    if ma != 0 {
        call_hline_blitter(blitter.as_mut(), x, y - 1, 1, ma);
    }
    f.wrapping_sub(32768)
}
// Source: src/core/SkScan_Antihair.cpp:122-142
unsafe fn hline_line(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    mut x: i32,
    stop: i32,
    mut f: i32,
    slope: i32,
) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 123; @condition b"x < stopx\0".as_ptr(); x < stop);
    let count = stop - x;
    f = f.wrapping_add(32768);
    let y = f >> 16;
    let mut a = fixed_to_alpha(f);
    if a != 0 {
        call_hline_blitter(blitter.as_mut(), x, y, count, a);
    }
    a = 255 - a;
    if a != 0 {
        call_hline_blitter(blitter.as_mut(), x, y - 1, count, a);
    }
    f.wrapping_sub(32768)
}
// Source: src/core/SkScan_Antihair.cpp:147-148
// Source: src/core/SkScan_Antihair.cpp:149-159
unsafe fn horish_cap(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    x: i32,
    mut f: i32,
    slope: i32,
    coverage: i32,
) -> i32 {
    f = f.wrapping_add(32768);
    let lower_y = f >> 16;
    let a = fixed_to_alpha(f);
    let a0 = scale_alpha_by_coverage(255 - a, coverage);
    let a1 = scale_alpha_by_coverage(a, coverage);
    blitter.as_mut().blitAntiV2(x, lower_y - 1, a0, a1);
    f.wrapping_add(slope).wrapping_sub(32768)
}
// Source: src/core/SkScan_Antihair.cpp:161-174
unsafe fn horish_line(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    mut x: i32,
    stop: i32,
    mut f: i32,
    slope: i32,
) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 162; @condition b"x < stopx\0".as_ptr(); x < stop);
    f = f.wrapping_add(32768);
    loop {
        let lower_y = f >> 16;
        let a = fixed_to_alpha(f);
        blitter.as_mut().blitAntiV2(x, lower_y - 1, 255 - a, a);
        f = f.wrapping_add(slope);
        x += 1;
        if x >= stop {
            break;
        }
    }
    f.wrapping_sub(32768)
}
// Source: src/core/SkScan_Antihair.cpp:178-179
// Source: src/core/SkScan_Antihair.cpp:180-197
unsafe fn vline_cap(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    x: i32,
    mut f: i32,
    slope: i32,
    coverage: i32,
) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 181; @condition b"0 == dx\0".as_ptr(); (slope) == ( 0));
    f = f.wrapping_add(32768);
    let xx = f >> 16;
    let a = fixed_to_alpha(f);
    let mut ma = scale_alpha_by_coverage(a, coverage);
    if ma != 0 {
        blitter.as_mut().blitV(xx, x, 1, ma as u8);
    }
    ma = scale_alpha_by_coverage(255 - a, coverage);
    if ma != 0 {
        blitter.as_mut().blitV(xx - 1, x, 1, ma as u8);
    }
    f.wrapping_sub(32768)
}
// Source: src/core/SkScan_Antihair.cpp:199-216
unsafe fn vline_line(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    mut x: i32,
    stop: i32,
    mut f: i32,
    slope: i32,
) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 200; @condition b"y < stopy\0".as_ptr(); x < stop);
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 201; @condition b"0 == dx\0".as_ptr(); (slope) == ( 0));
    f = f.wrapping_add(32768);
    let xx = f >> 16;
    let mut a = fixed_to_alpha(f);
    if a != 0 {
        blitter.as_mut().blitV(xx, x, stop - x, a as u8);
    }
    a = 255 - a;
    if a != 0 {
        blitter.as_mut().blitV(xx - 1, x, stop - x, a as u8);
    }
    f.wrapping_sub(32768)
}
// Source: src/core/SkScan_Antihair.cpp:221-222
// Source: src/core/SkScan_Antihair.cpp:223-232
unsafe fn vertish_cap(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    x: i32,
    mut f: i32,
    slope: i32,
    coverage: i32,
) -> i32 {
    f = f.wrapping_add(32768);
    let xx = f >> 16;
    let a = fixed_to_alpha(f);
    blitter.as_mut().blitAntiH2(
        xx - 1,
        x,
        scale_alpha_by_coverage(255 - a, coverage),
        scale_alpha_by_coverage(a, coverage),
    );
    f.wrapping_add(slope).wrapping_sub(32768)
}
// Source: src/core/SkScan_Antihair.cpp:234-245
unsafe fn vertish_line(
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    mut x: i32,
    stop: i32,
    mut f: i32,
    slope: i32,
) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 235; @condition b"y < stopy\0".as_ptr(); x < stop);
    f = f.wrapping_add(32768);
    loop {
        let xx = f >> 16;
        let a = fixed_to_alpha(f);
        blitter.as_mut().blitAntiH2(xx - 1, x, 255 - a, a);
        f = f.wrapping_add(slope);
        x += 1;
        if x >= stop {
            break;
        }
    }
    f.wrapping_sub(32768)
}
// Rust dispatch adaptation for the original virtual calls. The selected
// concrete algorithm remains in source order above; no stage is substituted.
impl AntiHairBlitter<'_> {
    unsafe fn drawCap(&mut self, x: i32, f: i32, slope: i32, coverage: i32) -> i32 {
        match self.kind {
            AntiHairKind::HLine => hline_cap(self.blitter.as_mut(), x, f, slope, coverage),
            AntiHairKind::Horish => horish_cap(self.blitter.as_mut(), x, f, slope, coverage),
            AntiHairKind::VLine => vline_cap(self.blitter.as_mut(), x, f, slope, coverage),
            AntiHairKind::Vertish => vertish_cap(self.blitter.as_mut(), x, f, slope, coverage),
        }
    }
    unsafe fn drawLine(&mut self, x: i32, stop: i32, f: i32, slope: i32) -> i32 {
        match self.kind {
            AntiHairKind::HLine => hline_line(self.blitter.as_mut(), x, stop, f, slope),
            AntiHairKind::Horish => horish_line(self.blitter.as_mut(), x, stop, f, slope),
            AntiHairKind::VLine => vline_line(self.blitter.as_mut(), x, stop, f, slope),
            AntiHairKind::Vertish => vertish_line(self.blitter.as_mut(), x, stop, f, slope),
        }
    }
}
// Source: src/core/SkScan_Antihair.cpp:248-252
fn fastfixdiv(a: i32, b: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 249; @condition b"(SkLeftShift(a, 16) >> 16) == a\0".as_ptr(); (a.wrapping_shl(16) >> 16) == ( a));
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 250; @condition b"b != 0\0".as_ptr(); (b) != (0));
    a.wrapping_shl(16) / b
}
// Source: src/core/SkScan_Antihair.cpp:258-264
fn bad_int(x: i32) -> i32 {
    x & x.wrapping_neg()
}
fn any_bad_ints(a: i32, b: i32, c: i32, d: i32) -> i32 {
    (bad_int(a) | bad_int(b) | bad_int(c) | bad_int(d)) >> (core::mem::size_of::<i32>() * 8 - 1)
}
// Source: src/core/SkScan_Antihair.cpp:276-279
#[cfg(sk_debug)]
fn canConvertFDot6ToFixed(x: i32) -> bool {
    x.abs() <= (i32::MAX >> (16 - 6))
}
// Source: src/core/SkScan_Antihair.cpp:284-286
fn fd6_frac(x: i32) -> i32 {
    x & 63
}
// Source: src/core/SkScan_Antihair.cpp:294-305
fn partial_pixel_coverage(pos: i32) -> i32 {
    let result = fd6_frac(pos.wrapping_sub(1)) + 1;
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 303; @condition b"result > 0 && result <= SK_FDot6One\0".as_ptr(); result > 0 && result <= 64);
    result
}
// Source: src/core/SkScan_Antihair.cpp:307-531
unsafe fn do_anti_hairline(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut clip: Option<&SkIRect>,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    use crate::include::private::SkFixed::{SkFixedCeilToInt, SkFixedFloorToInt};
    use crate::src::core::SkFDot6::*;
    if any_bad_ints(x0, y0, x1, y1) != 0 {
        return;
    }
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 318; @condition b"canConvertFDot6ToFixed(x0)\0".as_ptr(); canConvertFDot6ToFixed(x0));
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 319; @condition b"canConvertFDot6ToFixed(y0)\0".as_ptr(); canConvertFDot6ToFixed(y0));
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 320; @condition b"canConvertFDot6ToFixed(x1)\0".as_ptr(); canConvertFDot6ToFixed(x1));
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 321; @condition b"canConvertFDot6ToFixed(y1)\0".as_ptr(); canConvertFDot6ToFixed(y1));
    if (x1 - x0).abs() > SkIntToFDot6(511) || (y1 - y0).abs() > SkIntToFDot6(511) {
        let hx = (x0 >> 1) + (x1 >> 1);
        let hy = (y0 >> 1) + (y1 >> 1);
        do_anti_hairline(x0, y0, hx, hy, clip, blitter.as_mut());
        do_anti_hairline(hx, hy, x1, y1, clip, blitter.as_mut());
        return;
    }
    let (mut start_coverage, mut stop_coverage);
    let (mut istart, mut istop);
    let (mut fstart, slope);
    let kind;
    if (x1 - x0).abs() > (y1 - y0).abs() {
        if x0 > x1 {
            core::mem::swap(&mut x0, &mut x1);
            core::mem::swap(&mut y0, &mut y1);
        }
        istart = SkFDot6Floor(x0);
        istop = SkFDot6Ceil(x1);
        if y0 == y1 {
            slope = 0;
            kind = AntiHairKind::HLine;
            fstart = SkFDot6ToFixed(y0);
        } else {
            slope = fastfixdiv(y1 - y0, x1 - x0);
            crate::SkASSERTF!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 362; @condition b"slope >= -SK_Fixed1 && slope <= SK_Fixed1\0".as_ptr(); slope >= -65536 && slope <= 65536, "should be vertical or mostly vertical");
            let dx_to_center = 32 - fd6_frac(x0);
            fstart = SkFDot6ToFixed(y0)
                .wrapping_add((slope.wrapping_mul(dx_to_center).wrapping_add(32)) >> 6);
            kind = AntiHairKind::Horish;
        }
        crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 370; @condition b"istop > istart\0".as_ptr(); istop > istart);
        if istop - istart == 1 {
            start_coverage = x1 - x0;
            crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 374; @condition b"startCoverage >= 0 && startCoverage <= SK_FDot6One\0".as_ptr(); start_coverage >= 0 && start_coverage <= 64);
            stop_coverage = 0;
        } else {
            start_coverage = 64 - fd6_frac(x0);
            stop_coverage = fd6_frac(x1);
        }
        if let Some(c) = clip {
            if istart >= c.fRight || istop <= c.fLeft {
                return;
            }
            if istart < c.fLeft {
                fstart = fstart.wrapping_add(slope.wrapping_mul(c.fLeft - istart));
                istart = c.fLeft;
                start_coverage = 64;
                if istop - istart == 1 {
                    start_coverage = partial_pixel_coverage(x1);
                    stop_coverage = 0;
                }
            }
            if istop > c.fRight {
                istop = c.fRight;
                stop_coverage = 0;
            }
            crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 400; @condition b"istart <= istop\0".as_ptr(); istart <= istop);
            if istart == istop {
                return;
            }
            let (mut top, mut bottom);
            if slope >= 0 {
                top = SkFixedFloorToInt(fstart.wrapping_sub(32768));
                bottom = SkFixedCeilToInt(
                    fstart
                        .wrapping_add((istop - istart - 1).wrapping_mul(slope))
                        .wrapping_add(32768),
                );
            } else {
                bottom = SkFixedCeilToInt(fstart.wrapping_add(32768));
                top = SkFixedFloorToInt(
                    fstart
                        .wrapping_add((istop - istart - 1).wrapping_mul(slope))
                        .wrapping_sub(32768),
                );
            }
            top -= 1;
            bottom += 1;
            if top >= c.fBottom || bottom <= c.fTop {
                return;
            }
            if c.fTop <= top && c.fBottom >= bottom {
                clip = None;
            }
        }
    } else {
        if y0 > y1 {
            core::mem::swap(&mut x0, &mut x1);
            core::mem::swap(&mut y0, &mut y1);
        }
        istart = SkFDot6Floor(y0);
        istop = SkFDot6Ceil(y1);
        if x0 == x1 {
            if y0 == y1 {
                return;
            }
            slope = 0;
            kind = AntiHairKind::VLine;
            fstart = SkFDot6ToFixed(x0);
        } else {
            slope = fastfixdiv(x1 - x0, y1 - y0);
            crate::SkASSERTF!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 442; @condition b"slope <= SK_Fixed1 && slope >= -SK_Fixed1\0".as_ptr(); slope <= 65536 && slope >= -65536, "should be horizontal or mostly horizontal");
            let dy_to_center = 32 - fd6_frac(y0);
            fstart = SkFDot6ToFixed(x0)
                .wrapping_add((slope.wrapping_mul(dy_to_center).wrapping_add(32)) >> 6);
            kind = AntiHairKind::Vertish;
        }
        crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 450; @condition b"istop > istart\0".as_ptr(); istop > istart);
        if istop - istart == 1 {
            start_coverage = y1 - y0;
            crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 454; @condition b"startCoverage >= 0 && startCoverage <= SK_FDot6One\0".as_ptr(); start_coverage >= 0 && start_coverage <= 64);
            stop_coverage = 0;
        } else {
            start_coverage = 64 - fd6_frac(y0);
            stop_coverage = fd6_frac(y1);
        }
        if let Some(c) = clip {
            if istart >= c.fBottom || istop <= c.fTop {
                return;
            }
            if istart < c.fTop {
                fstart = fstart.wrapping_add(slope.wrapping_mul(c.fTop - istart));
                istart = c.fTop;
                start_coverage = 64;
                if istop - istart == 1 {
                    start_coverage = partial_pixel_coverage(y1);
                    stop_coverage = 0;
                }
            }
            if istop > c.fBottom {
                istop = c.fBottom;
                stop_coverage = 0;
            }
            crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 480; @condition b"istart <= istop\0".as_ptr(); istart <= istop);
            if istart == istop {
                return;
            }
            let (mut left, mut right);
            if slope >= 0 {
                left = SkFixedFloorToInt(fstart.wrapping_sub(32768));
                right = SkFixedCeilToInt(
                    fstart
                        .wrapping_add((istop - istart - 1).wrapping_mul(slope))
                        .wrapping_add(32768),
                );
            } else {
                right = SkFixedCeilToInt(fstart.wrapping_add(32768));
                left = SkFixedFloorToInt(
                    fstart
                        .wrapping_add((istop - istart - 1).wrapping_mul(slope))
                        .wrapping_sub(32768),
                );
            }
            left -= 1;
            right += 1;
            if left >= c.fRight || right <= c.fLeft {
                return;
            }
            if c.fLeft <= left && c.fRight >= right {
                clip = None;
            }
        }
    }
    let mut rect_clipper = Box::pin(if let Some(c) = clip {
        crate::src::core::SkBlitter::SkBlitterClipper::Rect(
            crate::src::core::SkBlitter::SkRectClipBlitter::new(blitter.as_mut(), *c),
        )
    } else {
        crate::src::core::SkBlitter::SkBlitterClipper::Direct(blitter.as_mut())
    });
    let mut blitter = rect_clipper.as_mut().blitter();
    let mut hair_blitter = AntiHairBlitter { kind, blitter };
    #[cfg(sk_debug)]
    if start_coverage > 0 && stop_coverage > 0 {
        crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 518; @condition b"istart < istop - 1\0".as_ptr(); istart < istop - 1);
    };
    fstart = hair_blitter.drawCap(istart, fstart, slope, start_coverage);
    istart += 1;
    let full_spans = istop - istart - i32::from(stop_coverage > 0);
    if full_spans > 0 {
        fstart = hair_blitter.drawLine(istart, istart + full_spans, fstart, slope);
    }
    if stop_coverage > 0 {
        hair_blitter.drawCap(istop - 1, fstart, slope, stop_coverage);
    }
}
impl SkScan {
    // Source: src/core/SkScan_Antihair.cpp:533-609
    pub unsafe fn AntiHairLineRgn(
        src: &[crate::src::core::SkPoint::SkPoint],
        clip: Option<&SkRegion>,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        use crate::src::core::SkFDot6::*;
        use crate::src::core::SkPoint::SkPoint;
        if src.is_empty() || clip.is_some_and(|c| c.is_empty()) {
            return;
        }
        crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 538; @condition b"clip == nullptr || !clip->getBounds().isEmpty()\0".as_ptr(); clip.is_none_or(|c| !c.get_bounds().is_empty()));
        let max = 32767.0;
        let fixed_bounds = SkRect::MakeLTRB(-max, -max, max, max);
        let clip_bounds = clip.map(|c| SkRect::Make(c.get_bounds()).makeOutset(1.0, 1.0));
        for i in 0..src.len() - 1 {
            let mut pts = [SkPoint::default(); 2];
            let line = [src[i], src[i + 1]];
            if !crate::src::core::SkLineClipper::SkLineClipper::IntersectLine(
                &line,
                &fixed_bounds,
                &mut pts,
            ) {
                continue;
            }
            if let Some(c) = clip_bounds {
                let line = pts;
                if !crate::src::core::SkLineClipper::SkLineClipper::IntersectLine(
                    &line, &c, &mut pts,
                ) {
                    continue;
                }
            }
            let x0 = SkScalarToFDot6(pts[0].fX);
            let y0 = SkScalarToFDot6(pts[0].fY);
            let x1 = SkScalarToFDot6(pts[1].fX);
            let y1 = SkScalarToFDot6(pts[1].fY);
            if let Some(c) = clip {
                let left = x0.min(x1);
                let top = y0.min(y1);
                let right = x0.max(x1);
                let bottom = y0.max(y1);
                let ir = SkIRect::make_ltrb(
                    SkFDot6Floor(left) - 1,
                    SkFDot6Floor(top) - 1,
                    SkFDot6Ceil(right) + 1,
                    SkFDot6Ceil(bottom) + 1,
                );
                if c.quick_reject_rect(&ir) {
                    continue;
                }
                if !c.quick_contains(&ir) {
                    let mut iter = Cliperator::new(c, ir);
                    while !iter.done() {
                        do_anti_hairline(x0, y0, x1, y1, Some(iter.rect()), blitter.as_mut());
                        iter.next();
                    }
                    continue;
                }
            }
            do_anti_hairline(x0, y0, x1, y1, None, blitter.as_mut());
        }
    }
    // Source: src/core/SkScan_Antihair.cpp:611-621
    pub unsafe fn AntiHairRect(
        rect: &SkRect,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        use crate::src::core::SkPoint::SkPoint;
        let pts = [
            SkPoint::make(rect.fLeft, rect.fTop),
            SkPoint::make(rect.fRight, rect.fTop),
            SkPoint::make(rect.fRight, rect.fBottom),
            SkPoint::make(rect.fLeft, rect.fBottom),
            SkPoint::make(rect.fLeft, rect.fTop),
        ];
        Self::AntiHairLine(&pts, clip, blitter.as_mut());
    }
}
// Source: src/core/SkScan_Antihair.cpp:625-625
type FDot8 = i32;
// Source: src/core/SkScan_Antihair.cpp:627-629
fn SkFixedToFDot8(x: i32) -> FDot8 {
    x.wrapping_add(0x80) >> 8
}
// Source: src/core/SkScan_Antihair.cpp:631-655
unsafe fn do_scanline(
    l: FDot8,
    top: i32,
    r: FDot8,
    alpha: u32,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 633; @condition b"L < R\0".as_ptr(); l < r);
    if (l >> 8) == ((r - 1) >> 8) {
        blitter
            .as_mut()
            .blitV(l >> 8, top, 1, SkAlphaMul(alpha, (r - l) as u32) as u8);
        return;
    }
    let mut left = l >> 8;
    if (l & 0xff) != 0 {
        blitter.as_mut().blitV(
            left,
            top,
            1,
            SkAlphaMul(alpha, (256 - (l & 0xff)) as u32) as u8,
        );
        left += 1;
    }
    let rite = r >> 8;
    let width = rite - left;
    if width > 0 {
        call_hline_blitter(blitter.as_mut(), left, top, width, alpha);
    }
    if (r & 0xff) != 0 {
        blitter
            .as_mut()
            .blitV(rite, top, 1, SkAlphaMul(alpha, (r & 0xff) as u32) as u8);
    }
}
// Source: src/core/SkScan_Antihair.cpp:657-699
unsafe fn antifilldot8(
    l: FDot8,
    t: FDot8,
    r: FDot8,
    b: FDot8,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    fill_inner: bool,
) {
    if l >= r || t >= b {
        return;
    }
    let mut top = t >> 8;
    if top == ((b - 1) >> 8) {
        do_scanline(l, top, r, (b - t - 1) as u32, blitter.as_mut());
        return;
    }
    if (t & 0xff) != 0 {
        do_scanline(l, top, r, (256 - (t & 0xff)) as u32, blitter.as_mut());
        top += 1;
    }
    let bot = b >> 8;
    let height = bot - top;
    if height > 0 {
        let mut left = l >> 8;
        if left == ((r - 1) >> 8) {
            blitter.as_mut().blitV(left, top, height, (r - l - 1) as u8);
        } else {
            if (l & 0xff) != 0 {
                blitter
                    .as_mut()
                    .blitV(left, top, height, (256 - (l & 0xff)) as u8);
                left += 1;
            }
            let rite = r >> 8;
            let width = rite - left;
            if width > 0 && fill_inner {
                blitter.as_mut().blitRect(left, top, width, height);
            }
            if (r & 0xff) != 0 {
                blitter.as_mut().blitV(rite, top, height, (r & 0xff) as u8);
            }
        }
    }
    if (b & 0xff) != 0 {
        do_scanline(l, bot, r, (b & 0xff) as u32, blitter.as_mut());
    }
}
// Source: src/core/SkScan_Antihair.cpp:701-705
unsafe fn antifill_xrect(xr: &SkXRect, mut blitter: core::pin::Pin<&mut dyn SkBlitter>) {
    antifilldot8(
        SkFixedToFDot8(xr.fLeft),
        SkFixedToFDot8(xr.fTop),
        SkFixedToFDot8(xr.fRight),
        SkFixedToFDot8(xr.fBottom),
        blitter.as_mut(),
        true,
    );
}
impl SkScan {
    // Source: src/core/SkScan_Antihair.cpp:709-746
    pub unsafe fn AntiFillXRect(
        xr: &SkXRect,
        clip: Option<&SkRegion>,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if let Some(clip) = clip {
            let mut outer_bounds = SkIRect::default();
            XRect_roundOut(xr, &mut outer_bounds);
            if clip.is_rect() {
                let clip_bounds = clip.get_bounds();
                if clip_bounds.contains_rect(&outer_bounds) {
                    antifill_xrect(xr, blitter.as_mut());
                } else {
                    let mut tmp = SkXRect::default();
                    XRect_set_i(&mut tmp, clip_bounds);
                    if tmp.intersect(xr) {
                        antifill_xrect(&tmp, blitter.as_mut());
                    }
                }
            } else {
                let mut clipper = Cliperator::new(clip, outer_bounds);
                while !clipper.done() {
                    let mut tmp = SkXRect::default();
                    XRect_set_i(&mut tmp, clipper.rect());
                    if tmp.intersect(xr) {
                        antifill_xrect(&tmp, blitter.as_mut());
                    }
                    clipper.next();
                }
            }
        } else {
            antifill_xrect(xr, blitter.as_mut());
        }
    }
    // Source: src/core/SkScan_Antihair.cpp:748-763
    pub unsafe fn AntiFillXRectRasterClip(
        xr: &SkXRect,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_bw() {
            Self::AntiFillXRect(xr, Some(clip.bw_rgn()), blitter.as_mut());
        } else {
            let mut outer_bounds = SkIRect::default();
            XRect_roundOut(xr, &mut outer_bounds);
            if clip.quick_contains(&outer_bounds) {
                Self::AntiFillXRect(xr, None, blitter.as_mut());
            } else {
                let mut wrapper = Box::pin(
                    crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                        clip,
                        blitter.as_mut(),
                    ),
                );
                let (rgn, mut blitter) = wrapper.as_mut().parts();
                Self::AntiFillXRect(xr, Some(rgn), blitter.as_mut());
            }
        }
    }
}
// Source: src/core/SkScan_Antihair.cpp:769-774
unsafe fn antifill_rect(r: &SkRect, mut blitter: core::pin::Pin<&mut dyn SkBlitter>) {
    let mut xr = SkXRect::default();
    XRect_set(&mut xr, r);
    antifill_xrect(&xr, blitter.as_mut());
}
impl SkScan {
    // Source: src/core/SkScan_Antihair.cpp:784-810
    pub unsafe fn AntiFillRect(
        orig_r: &SkRect,
        clip: Option<&SkRegion>,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if let Some(clip) = clip {
            let mut new_r = SkRect::Make(clip.get_bounds());
            if !new_r.intersect(orig_r) {
                return;
            }
            let outer_bounds = new_r.roundOut();
            if clip.is_rect() {
                antifill_rect(&new_r, blitter.as_mut());
            } else {
                let mut clipper = Cliperator::new(clip, outer_bounds);
                while !clipper.done() {
                    new_r.set(clipper.rect());
                    if new_r.intersect(orig_r) {
                        antifill_rect(&new_r, blitter.as_mut());
                    }
                    clipper.next();
                }
            }
        } else {
            antifill_rect(orig_r, blitter.as_mut());
        }
    }
    // Source: src/core/SkScan_Antihair.cpp:812-821
    pub unsafe fn AntiFillRectRasterClip(
        r: &SkRect,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_bw() {
            Self::AntiFillRect(r, Some(clip.bw_rgn()), blitter.as_mut());
        } else {
            let mut wrap = Box::pin(
                crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                    clip,
                    blitter.as_mut(),
                ),
            );
            let (rgn, mut blitter) = wrap.as_mut().parts();
            Self::AntiFillRect(r, Some(rgn), blitter.as_mut());
        }
    }
}
// Source: src/core/SkScan_Antihair.cpp:827-831
unsafe fn fillcheckrect(
    l: i32,
    t: i32,
    r: i32,
    b: i32,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    if l < r && t < b {
        blitter.as_mut().blitRect(l, t, r - l, b - t);
    }
}
// Source: src/core/SkScan_Antihair.cpp:833-835
fn SkScalarToFDot8(x: f32) -> FDot8 {
    (x * 256.0) as i32
}
// Source: src/core/SkScan_Antihair.cpp:837-839
fn FDot8Floor(x: FDot8) -> i32 {
    x >> 8
}
// Source: src/core/SkScan_Antihair.cpp:841-843
fn FDot8Ceil(x: FDot8) -> i32 {
    x.wrapping_add(0xff) >> 8
}
// Source: src/core/SkScan_Antihair.cpp:846-850
fn InvAlphaMul(a: u32, b: u32) -> u8 {
    (a + b - crate::include::private::SkMath::SkMulDiv255Round(a, b)) as u8
}
// Source: src/core/SkScan_Antihair.cpp:852-880
unsafe fn inner_scanline(
    l: FDot8,
    top: i32,
    r: FDot8,
    alpha: u32,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 854; @condition b"L < R\0".as_ptr(); l < r);
    if (l >> 8) == ((r - 1) >> 8) {
        let mut wid_clamp = r - l;
        wid_clamp -= wid_clamp >> 8;
        blitter
            .as_mut()
            .blitV(l >> 8, top, 1, InvAlphaMul(alpha, wid_clamp as u32));
        return;
    }
    let mut left = l >> 8;
    if (l & 0xff) != 0 {
        blitter
            .as_mut()
            .blitV(left, top, 1, InvAlphaMul(alpha, (l & 0xff) as u32));
        left += 1;
    }
    let rite = r >> 8;
    let width = rite - left;
    if width > 0 {
        call_hline_blitter(blitter.as_mut(), left, top, width, alpha);
    }
    if (r & 0xff) != 0 {
        blitter
            .as_mut()
            .blitV(rite, top, 1, InvAlphaMul(alpha, (!r & 0xff) as u32));
    }
}
// Source: src/core/SkScan_Antihair.cpp:882-915
unsafe fn innerstrokedot8(
    l: FDot8,
    t: FDot8,
    r: FDot8,
    b: FDot8,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 884; @condition b"L < R && T < B\0".as_ptr(); l < r && t < b);
    let mut top = t >> 8;
    if top == ((b - 1) >> 8) {
        let alpha = 256 - (b - t);
        if alpha != 0 {
            inner_scanline(l, top, r, alpha as u32, blitter.as_mut());
        }
        return;
    }
    if (t & 0xff) != 0 {
        inner_scanline(l, top, r, (t & 0xff) as u32, blitter.as_mut());
        top += 1;
    }
    let bot = b >> 8;
    let height = bot - top;
    if height > 0 {
        if (l & 0xff) != 0 {
            blitter
                .as_mut()
                .blitV(l >> 8, top, height, (l & 0xff) as u8);
        }
        if (r & 0xff) != 0 {
            blitter
                .as_mut()
                .blitV(r >> 8, top, height, (!r & 0xff) as u8);
        }
    }
    if (b & 0xff) != 0 {
        inner_scanline(l, bot, r, (!b & 0xff) as u32, blitter.as_mut());
    }
}
// Source: src/core/SkScan_Antihair.cpp:917-924
fn align_thin_stroke(edge1: &mut FDot8, edge2: &mut FDot8) {
    crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 918; @condition b"edge1 <= edge2\0".as_ptr(); *edge1 <= *edge2);
    if FDot8Floor(*edge1) == FDot8Floor(*edge2) {
        *edge2 -= *edge1 & 0xff;
        *edge1 &= !0xff;
    }
}
impl SkScan {
    // Source: src/core/SkScan_Antihair.cpp:926-1015
    pub unsafe fn AntiFrameRect(
        r: &SkRect,
        stroke_size: &crate::src::core::SkPoint::SkPoint,
        clip: Option<&SkRegion>,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        crate::SkASSERT!(@source b"src/core/SkScan_Antihair.cpp\0".as_ptr(), 928; @condition b"strokeSize.fX >= 0 && strokeSize.fY >= 0\0".as_ptr(); stroke_size.fX >= 0.0 && stroke_size.fY >= 0.0);
        let mut rx = stroke_size.fX / 2.0;
        let mut ry = stroke_size.fY / 2.0;
        if r.width() == 0.0 {
            ry = 0.0;
        }
        if r.height() == 0.0 {
            rx = 0.0;
        }
        let mut outer_l = SkScalarToFDot8(r.fLeft - rx);
        let mut outer_t = SkScalarToFDot8(r.fTop - ry);
        let mut outer_r = SkScalarToFDot8(r.fRight + rx);
        let mut outer_b = SkScalarToFDot8(r.fBottom + ry);
        let mut outer = SkIRect::make_ltrb(
            FDot8Floor(outer_l),
            FDot8Floor(outer_t),
            FDot8Ceil(outer_r),
            FDot8Ceil(outer_b),
        );
        if clip.is_some_and(|c| c.quick_reject_rect(&outer)) {
            return;
        }
        let needs_clip = clip.is_some_and(|c| !c.contains_rect(&outer));
        let mut clipper = Box::pin(crate::src::core::SkBlitter::SkBlitterClipper::apply(
            blitter.as_mut(),
            if needs_clip { clip } else { None },
            Some(&outer),
        ));
        let mut blitter = clipper.as_mut().blitter();
        rx = stroke_size.fX - rx;
        ry = stroke_size.fY - ry;
        let mut inner_l = SkScalarToFDot8(r.fLeft + rx);
        let mut inner_t = SkScalarToFDot8(r.fTop + ry);
        let mut inner_r = SkScalarToFDot8(r.fRight - rx);
        let mut inner_b = SkScalarToFDot8(r.fBottom - ry);
        if stroke_size.fX < 1.0 || stroke_size.fY < 1.0 {
            align_thin_stroke(&mut outer_l, &mut inner_l);
            align_thin_stroke(&mut outer_t, &mut inner_t);
            align_thin_stroke(&mut inner_r, &mut outer_r);
            align_thin_stroke(&mut inner_b, &mut outer_b);
        }
        antifilldot8(outer_l, outer_t, outer_r, outer_b, blitter.as_mut(), false);
        outer.set_ltrb(
            FDot8Ceil(outer_l),
            FDot8Ceil(outer_t),
            FDot8Floor(outer_r),
            FDot8Floor(outer_b),
        );
        if inner_l >= inner_r || inner_t >= inner_b {
            fillcheckrect(
                outer.fLeft,
                outer.fTop,
                outer.fRight,
                outer.fBottom,
                blitter.as_mut(),
            );
        } else {
            let inner = SkIRect::make_ltrb(
                FDot8Floor(inner_l),
                FDot8Floor(inner_t),
                FDot8Ceil(inner_r),
                FDot8Ceil(inner_b),
            );
            fillcheckrect(
                outer.fLeft,
                outer.fTop,
                outer.fRight,
                inner.fTop,
                blitter.as_mut(),
            );
            fillcheckrect(
                outer.fLeft,
                inner.fTop,
                inner.fLeft,
                inner.fBottom,
                blitter.as_mut(),
            );
            fillcheckrect(
                inner.fRight,
                inner.fTop,
                outer.fRight,
                inner.fBottom,
                blitter.as_mut(),
            );
            fillcheckrect(
                outer.fLeft,
                inner.fBottom,
                outer.fRight,
                outer.fBottom,
                blitter.as_mut(),
            );
            innerstrokedot8(inner_l, inner_t, inner_r, inner_b, blitter.as_mut());
        }
    }
    // Source: src/core/SkScan_Antihair.cpp:1017-1025
    pub unsafe fn AntiFrameRectRasterClip(
        r: &SkRect,
        stroke_size: &crate::src::core::SkPoint::SkPoint,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_bw() {
            Self::AntiFrameRect(r, stroke_size, Some(clip.bw_rgn()), blitter.as_mut());
        } else {
            let mut wrap = Box::pin(
                crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                    clip,
                    blitter.as_mut(),
                ),
            );
            let (rgn, mut blitter) = wrap.as_mut().parts();
            Self::AntiFrameRect(r, stroke_size, Some(rgn), blitter.as_mut());
        }
    }
}

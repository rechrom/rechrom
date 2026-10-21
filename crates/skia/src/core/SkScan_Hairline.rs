// Drawing safety: unsafe render entrypoints require the SkBlitter primitive
// contract: live exclusive destination/context/arena leases, valid bounds and
// initialized terminating sparse runs. Clip wrappers preserve this caller lease;
// clipping alone does not establish ownership or backing-storage lifetime.
// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::private::SkFixed::SkFixedDiv;
use crate::src::core::SkBlitter::{SkBlitter, SkBlitterClipper};
use crate::src::core::SkFDot6::*;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::{SkIRect, SkRect};
use crate::src::core::SkRegion::SkRegion;
use crate::src::core::SkScan::SkScan;
// Source: src/core/SkScan_Hairline.cpp:39-53
// Source: src/core/SkScan_Hairline.cpp:55-69
unsafe fn horiline(
    mut x: i32,
    stopx: i32,
    mut fy: i32,
    dy: i32,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 57; @condition b"x < stopx\0".as_ptr(); x < stopx);
    if let Some(direct) = blitter.as_mut().canDirectBlit() {
        let pm = direct.pm;
        let value = direct.value;
        match pm.info().bytesPerPixel() {
            1 => loop {
                unsafe {
                    assert_direct_bounds(&pm, x, fy >> 16);
                    pm.writable_addr8(x, fy >> 16).write(value as u8);
                }
                fy = fy.wrapping_add(dy);
                x += 1;
                if x >= stopx {
                    break;
                }
            },
            2 => loop {
                unsafe {
                    assert_direct_bounds(&pm, x, fy >> 16);
                    pm.writable_addr16(x, fy >> 16).write(value as u16);
                }
                fy = fy.wrapping_add(dy);
                x += 1;
                if x >= stopx {
                    break;
                }
            },
            4 => loop {
                unsafe {
                    assert_direct_bounds(&pm, x, fy >> 16);
                    pm.writable_addr32(x, fy >> 16).write(value as u32);
                }
                fy = fy.wrapping_add(dy);
                x += 1;
                if x >= stopx {
                    break;
                }
            },
            8 => loop {
                unsafe {
                    assert_direct_bounds(&pm, x, fy >> 16);
                    pm.writable_addr64(x, fy >> 16).write(value);
                }
                fy = fy.wrapping_add(dy);
                x += 1;
                if x >= stopx {
                    break;
                }
            },
            _ => {}
        }
    } else {
        loop {
            blitter.as_mut().blitH(x, fy >> 16, 1);
            fy = fy.wrapping_add(dy);
            x += 1;
            if x >= stopx {
                break;
            }
        }
    }
}
// Source: src/core/SkScan_Hairline.cpp:71-90
unsafe fn vertline(
    mut y: i32,
    stopy: i32,
    mut fx: i32,
    dx: i32,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 79; @condition b"y < stopy\0".as_ptr(); y < stopy);
    if let Some(direct) = blitter.as_mut().canDirectBlit() {
        let pm = direct.pm;
        let value = direct.value;
        match pm.info().bytesPerPixel() {
            1 => loop {
                unsafe {
                    assert_direct_bounds(&pm, fx >> 16, y);
                    pm.writable_addr8(fx >> 16, y).write(value as u8);
                }
                fx = fx.wrapping_add(dx);
                y += 1;
                if y >= stopy {
                    break;
                }
            },
            2 => loop {
                unsafe {
                    assert_direct_bounds(&pm, fx >> 16, y);
                    pm.writable_addr16(fx >> 16, y).write(value as u16);
                }
                fx = fx.wrapping_add(dx);
                y += 1;
                if y >= stopy {
                    break;
                }
            },
            4 => loop {
                unsafe {
                    assert_direct_bounds(&pm, fx >> 16, y);
                    pm.writable_addr32(fx >> 16, y).write(value as u32);
                }
                fx = fx.wrapping_add(dx);
                y += 1;
                if y >= stopy {
                    break;
                }
            },
            8 => loop {
                unsafe {
                    assert_direct_bounds(&pm, fx >> 16, y);
                    pm.writable_addr64(fx >> 16, y).write(value);
                }
                fx = fx.wrapping_add(dx);
                y += 1;
                if y >= stopy {
                    break;
                }
            },
            _ => {}
        }
    } else {
        loop {
            blitter.as_mut().blitH(fx >> 16, y, 1);
            fx = fx.wrapping_add(dx);
            y += 1;
            if y >= stopy {
                break;
            }
        }
    }
}
// Rust safety adaptation: the native direct interface carries caller clipping
// assumptions. Public safe Scan entrypoints validate bounds before raw writes.
fn assert_direct_bounds(pm: &crate::src::core::SkPixmap::SkPixmap, x: i32, y: i32) {
    assert!(
        x >= 0 && x < pm.width() && y >= 0 && y < pm.height(),
        "hairline direct pixel lies outside its writable pixmap lease"
    );
}
// Source: src/core/SkScan_Hairline.cpp:96-99
#[cfg(sk_debug)]
fn canConvertFDot6ToFixed(x: i32) -> bool {
    x.abs() <= (i32::MAX >> (16 - 6))
}
impl SkScan {
    // Source: src/core/SkScan_Hairline.cpp:102-214
    pub unsafe fn HairLineRgn(
        src: &[SkPoint],
        clip: Option<&SkRegion>,
        mut orig_blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if src.is_empty() {
            return;
        }
        let max = 32767.0;
        let fixed_bounds = SkRect::MakeLTRB(-max, -max, max, max);
        let clip_bounds = clip.map(|clip| SkRect::Make(clip.get_bounds()));
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
            if let Some(clip_bounds) = clip_bounds {
                let line = pts;
                if !crate::src::core::SkLineClipper::SkLineClipper::IntersectLine(
                    &line,
                    &clip_bounds,
                    &mut pts,
                ) {
                    continue;
                }
            }
            let mut x0 = SkScalarToFDot6(pts[0].fX);
            let mut y0 = SkScalarToFDot6(pts[0].fY);
            let mut x1 = SkScalarToFDot6(pts[1].fX);
            let mut y1 = SkScalarToFDot6(pts[1].fY);
            crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 139; @condition b"canConvertFDot6ToFixed(x0)\0".as_ptr(); canConvertFDot6ToFixed(x0));
            crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 140; @condition b"canConvertFDot6ToFixed(y0)\0".as_ptr(); canConvertFDot6ToFixed(y0));
            crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 141; @condition b"canConvertFDot6ToFixed(x1)\0".as_ptr(); canConvertFDot6ToFixed(x1));
            crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 142; @condition b"canConvertFDot6ToFixed(y1)\0".as_ptr(); canConvertFDot6ToFixed(y1));
            let mut needs_clip = false;
            if let Some(clip) = clip {
                let bounds = clip.get_bounds();
                let clip_r = SkIRect::make_ltrb(
                    SkIntToFDot6(bounds.fLeft),
                    SkIntToFDot6(bounds.fTop),
                    SkIntToFDot6(bounds.fRight),
                    SkIntToFDot6(bounds.fBottom),
                );
                let mut pts_r = SkIRect::make_ltrb(x0, y0, x1, y1);
                pts_r.sort();
                pts_r.fRight += SK_FDot6One;
                pts_r.fBottom += SK_FDot6One;
                if !SkIRect::intersects(&pts_r, &clip_r) {
                    continue;
                }
                needs_clip = !clip.is_rect() || !clip_r.contains_rect(&pts_r);
            }
            let mut clipper = Box::pin(SkBlitterClipper::apply(
                orig_blitter.as_mut(),
                if needs_clip { clip } else { None },
                None,
            ));
            let mut blitter = clipper.as_mut().blitter();
            let dx = x1 - x0;
            let dy = y1 - y0;
            if dx.abs() > dy.abs() {
                if x0 > x1 {
                    core::mem::swap(&mut x0, &mut x1);
                    core::mem::swap(&mut y0, &mut y1);
                }
                let ix0 = SkFDot6Round(x0);
                let ix1 = SkFDot6Round(x1);
                if ix0 == ix1 {
                    continue;
                }
                let slope = SkFixedDiv(dy, dx);
                let start_y =
                    SkFDot6ToFixed(y0).wrapping_add(slope.wrapping_mul((32 - x0) & 63) >> 6);
                horiline(ix0, ix1, start_y, slope, blitter.as_mut());
            } else {
                if y0 > y1 {
                    core::mem::swap(&mut x0, &mut x1);
                    core::mem::swap(&mut y0, &mut y1);
                }
                let iy0 = SkFDot6Round(y0);
                let iy1 = SkFDot6Round(y1);
                if iy0 == iy1 {
                    continue;
                }
                let slope = SkFixedDiv(dx, dy);
                let start_x =
                    SkFDot6ToFixed(x0).wrapping_add(slope.wrapping_mul((32 - y0) & 63) >> 6);
                vertline(iy0, iy1, start_x, slope, blitter.as_mut());
            }
        }
    }
}
// Source: src/core/SkScan_Hairline.cpp:216-222
struct DrawingParameters<'a> {
    clip: Option<&'a SkRegion>,
    inset_clip: Option<&'a SkRect>,
    outset_clip: Option<&'a SkRect>,
    blitter: core::pin::Pin<&'a mut dyn SkBlitter>,
    lineproc: HairRgnProc,
}
use super::SkScan::HairRgnProc;
impl SkScan {
    // Source: src/core/SkScan_Hairline.cpp:226-260
    pub unsafe fn HairRect(
        rect: &SkRect,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        use crate::include::core::SkScalar::SkScalarFloorToInt;
        let mut r = SkIRect::make_ltrb(
            SkScalarFloorToInt(rect.fLeft),
            SkScalarFloorToInt(rect.fTop),
            SkScalarFloorToInt(rect.fRight + 1.0),
            SkScalarFloorToInt(rect.fBottom + 1.0),
        );
        if !r.intersect(&clip.get_bounds().make_outset(1, 1)) {
            return;
        }
        if clip.quick_reject(&r) {
            return;
        }
        if !clip.quick_contains(&r) {
            if clip.is_bw() {
                let mut clipper = Box::pin(SkBlitterClipper::apply(
                    blitter.as_mut(),
                    Some(clip.bw_rgn()),
                    None,
                ));
                hair_rect_blit(&r, clipper.as_mut().blitter());
            } else {
                let mut wrapper = Box::pin(
                    crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                        clip,
                        blitter.as_mut(),
                    ),
                );
                let (rgn, mut blitter) = wrapper.as_mut().parts();
                let mut clipper =
                    Box::pin(SkBlitterClipper::apply(blitter.as_mut(), Some(rgn), None));
                hair_rect_blit(&r, clipper.as_mut().blitter());
            }
        } else {
            hair_rect_blit(&r, blitter.as_mut());
        }
    }
}
// Source: src/core/SkScan_Hairline.cpp:261-276
unsafe fn hair_rect_blit(r: &SkIRect, mut blitter: core::pin::Pin<&mut dyn SkBlitter>) {
    let width = r.width();
    let height = r.height();
    if (width | height) == 0 {
        return;
    }
    if width <= 2 || height <= 2 {
        blitter.as_mut().blitRect(r.fLeft, r.fTop, width, height);
        return;
    }
    blitter.as_mut().blitH(r.fLeft, r.fTop, width);
    blitter
        .as_mut()
        .blitRect(r.fLeft, r.fTop + 1, 1, height - 2);
    blitter
        .as_mut()
        .blitRect(r.fRight - 1, r.fTop + 1, 1, height - 2);
    blitter.as_mut().blitH(r.fLeft, r.fBottom - 1, width);
}
// Source: src/core/SkScan_Hairline.cpp:280-281
const MAX_CUBIC_SUBDIVIDE_LEVEL: usize = 9;
const MAX_QUAD_SUBDIVIDE_LEVEL: usize = 5;
// Source: src/core/SkScan_Hairline.cpp:283-283
// Canonical geometry uses SkPoint as the two-lane coefficient value.
type Float2 = SkPoint;
// Source: src/core/SkScan_Hairline.cpp:285-304
fn compute_int_quad_dist(pts: &[SkPoint; 3]) -> u32 {
    use crate::include::core::SkScalar::SkScalarCeilToInt;
    let mut dx = ((pts[0].fX + pts[2].fX) / 2.0) - pts[1].fX;
    let mut dy = ((pts[0].fY + pts[2].fY) / 2.0) - pts[1].fY;
    dx = dx.abs();
    dy = dy.abs();
    let idx = SkScalarCeilToInt(dx) as u32;
    let idy = SkScalarCeilToInt(dy) as u32;
    if idx > idy {
        idx.wrapping_add(idy >> 1)
    } else {
        idy.wrapping_add(idx >> 1)
    }
}
// Source: src/core/SkScan_Hairline.cpp:306-311
fn float2_is_finite(x: Float2) -> [u32; 2] {
    let x = crate::src::core::SkGeometry::from_point(&x);
    let exp_mask = 0xff << 23;
    core::array::from_fn(|i| {
        if (x[i].to_bits() & exp_mask) != exp_mask {
            u32::MAX
        } else {
            0
        }
    })
}
// Source: src/core/SkScan_Hairline.cpp:315-347
unsafe fn hair_quad(
    pts: &[SkPoint; 3],
    clip: Option<&SkRegion>,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    level: usize,
    lineproc: HairRgnProc,
) {
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 320; @condition b"level <= kMaxQuadSubdivideLevel\0".as_ptr(); level <= MAX_QUAD_SUBDIVIDE_LEVEL);
    let coeff = crate::src::core::SkGeometry::SkQuadCoeff::from_points(pts);
    let lines = 1usize << level;
    let mut t = SkPoint::make(0.0, 0.0);
    let dt = SkPoint::make(1.0 / lines as f32, 1.0 / lines as f32);
    let mut tmp =
        [core::mem::MaybeUninit::<SkPoint>::uninit(); (1 << MAX_QUAD_SUBDIVIDE_LEVEL) + 1];
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 330; @condition b"(unsigned)lines < std::size(tmp)\0".as_ptr(); lines < tmp.len());
    tmp[0].write(pts[0]);
    let mut is_finite = [u32::MAX; 2];
    for i in 1..lines {
        t = t + dt;
        let p = coeff.eval(t);
        let mask = float2_is_finite(p);
        is_finite[0] &= mask[0];
        is_finite[1] &= mask[1];
        tmp[i].write(p);
    }
    if crate::src::core::SkVx::all(&is_finite) {
        tmp[lines].write(pts[2]);
        // Every point in this prefix was initialized above; the unused native
        // stack-array tail remains uninitialized and is never referenced.
        let tmp = unsafe { core::slice::from_raw_parts(tmp.as_ptr().cast::<SkPoint>(), lines + 1) };
        lineproc(tmp, clip, blitter.as_mut());
    }
}
// Source: src/core/SkScan_Hairline.cpp:349-360
fn compute_nocheck_quad_bounds(pts: &[SkPoint; 3]) -> SkRect {
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 350; @condition b"SkIsFinite(&pts[0].fX, 6)\0".as_ptr(); pts.iter().all(|p| p.fX.is_finite() && p.fY.is_finite()));
    let mut min = crate::src::core::SkGeometry::from_point(&pts[0]);
    let mut max = min;
    for i in 1..3 {
        let pair = crate::src::core::SkGeometry::from_point(&pts[i]);
        min = crate::src::core::SkVx::min(min, pair);
        max = crate::src::core::SkVx::max(max, pair);
    }
    SkRect::MakeLTRB(min[0], min[1], max[0], max[1])
}
// Source: src/core/SkScan_Hairline.cpp:362-364
fn is_inverted(r: &SkRect) -> bool {
    r.fLeft > r.fRight || r.fTop > r.fBottom
}
// Source: src/core/SkScan_Hairline.cpp:368-372
fn geometric_overlap(a: &SkRect, b: &SkRect) -> bool {
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 369; @condition b"!is_inverted(a) && !is_inverted(b)\0".as_ptr(); !is_inverted(a) && !is_inverted(b));
    a.fLeft < b.fRight && b.fLeft < a.fRight && a.fTop < b.fBottom && b.fTop < a.fBottom
}
// Source: src/core/SkScan_Hairline.cpp:376-380
fn geometric_contains(outer: &SkRect, inner: &SkRect) -> bool {
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 377; @condition b"!is_inverted(outer) && !is_inverted(inner)\0".as_ptr(); !is_inverted(outer) && !is_inverted(inner));
    inner.fRight <= outer.fRight
        && inner.fLeft >= outer.fLeft
        && inner.fBottom <= outer.fBottom
        && inner.fTop >= outer.fTop
}
// Source: src/core/SkScan_Hairline.cpp:382-394
unsafe fn hairquad(pts: &[SkPoint; 3], d: &mut DrawingParameters<'_>, level: usize) {
    let mut clip = d.clip;
    if let Some(inset_clip) = d.inset_clip {
        crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 384; @condition b"d.outsetClip\0".as_ptr(); d.outset_clip.is_some());
        let bounds = compute_nocheck_quad_bounds(pts);
        if !geometric_overlap(d.outset_clip.unwrap(), &bounds) {
            return;
        } else if geometric_contains(inset_clip, &bounds) {
            clip = None;
        }
    }
    hair_quad(pts, clip, d.blitter.as_mut(), level, d.lineproc);
}
// Source: src/core/SkScan_Hairline.cpp:396-400
fn max_component(value: [f32; 2]) -> f32 {
    if value[0] < value[1] {
        value[1]
    } else {
        value[0]
    }
}
// Source: src/core/SkScan_Hairline.cpp:402-424
fn compute_cubic_segs(pts: &[SkPoint; 4]) -> usize {
    let p0 = pts[0];
    let p1 = pts[1];
    let p2 = pts[2];
    let p3 = pts[3];
    let one_third = 1.0 / 3.0;
    let two_third = 2.0 / 3.0;
    let p13 = p3 * one_third + p0 * two_third;
    let p23 = p0 * one_third + p3 * two_third;
    let a = crate::src::core::SkGeometry::from_point(&(p1 - p13));
    let b = crate::src::core::SkGeometry::from_point(&(p2 - p23));
    let diff = max_component(crate::src::core::SkVx::max(
        crate::src::core::SkVx::abs(a),
        crate::src::core::SkVx::abs(b),
    ));
    let mut tol = 1.0 / 8.0;
    for i in 0..MAX_CUBIC_SUBDIVIDE_LEVEL {
        if diff < tol {
            return 1 << i;
        }
        tol *= 4.0;
    }
    1 << MAX_CUBIC_SUBDIVIDE_LEVEL
}
// Source: src/core/SkScan_Hairline.cpp:426-428
fn lt_90(p0: SkPoint, pivot: SkPoint, p2: SkPoint) -> bool {
    SkPoint::dot_product(p0 - pivot, p2 - pivot) >= 0.0
}
// Source: src/core/SkScan_Hairline.cpp:431-436
fn quick_cubic_niceness_check(pts: &[SkPoint; 4]) -> bool {
    lt_90(pts[1], pts[0], pts[3])
        && lt_90(pts[2], pts[0], pts[3])
        && lt_90(pts[1], pts[3], pts[0])
        && lt_90(pts[2], pts[3], pts[0])
}
// Source: src/core/SkScan_Hairline.cpp:440-476
unsafe fn hair_cubic(
    pts: &[SkPoint; 4],
    clip: Option<&SkRegion>,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    lineproc: HairRgnProc,
) {
    let lines = compute_cubic_segs(pts);
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 445; @condition b"lines > 0\0".as_ptr(); lines > 0);
    if lines == 1 {
        lineproc(&[pts[0], pts[3]], clip, blitter.as_mut());
        return;
    }
    let coeff = crate::src::core::SkGeometry::SkCubicCoeff::new(pts);
    let mut t = SkPoint::make(0.0, 0.0);
    let dt = SkPoint::make(1.0 / lines as f32, 1.0 / lines as f32);
    let mut tmp =
        [core::mem::MaybeUninit::<SkPoint>::uninit(); (1 << MAX_CUBIC_SUBDIVIDE_LEVEL) + 1];
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 458; @condition b"(unsigned)lines < std::size(tmp)\0".as_ptr(); lines < tmp.len());
    tmp[0].write(pts[0]);
    let mut is_finite = [u32::MAX; 2];
    for i in 1..lines {
        t = t + dt;
        let p = coeff.eval(t);
        let mask = float2_is_finite(p);
        is_finite[0] &= mask[0];
        is_finite[1] &= mask[1];
        tmp[i].write(p);
    }
    if crate::src::core::SkVx::all(&is_finite) {
        tmp[lines].write(pts[3]);
        let tmp = unsafe { core::slice::from_raw_parts(tmp.as_ptr().cast::<SkPoint>(), lines + 1) };
        lineproc(tmp, clip, blitter.as_mut());
    }
}
// Source: src/core/SkScan_Hairline.cpp:478-489
fn compute_nocheck_cubic_bounds(pts: &[SkPoint; 4]) -> SkRect {
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 479; @condition b"SkIsFinite(&pts[0].fX, 8)\0".as_ptr(); pts.iter().all(|p| p.fX.is_finite() && p.fY.is_finite()));
    let mut min = crate::src::core::SkGeometry::from_point(&pts[0]);
    let mut max = min;
    for i in 1..4 {
        let pair = crate::src::core::SkGeometry::from_point(&pts[i]);
        min = crate::src::core::SkVx::min(min, pair);
        max = crate::src::core::SkVx::max(max, pair);
    }
    SkRect::MakeLTRB(min[0], min[1], max[0], max[1])
}
// Source: src/core/SkScan_Hairline.cpp:491-514
unsafe fn haircubic(pts: &[SkPoint; 4], d: &mut DrawingParameters<'_>, _level: usize) {
    let mut clip = d.clip;
    if let Some(inset_clip) = d.inset_clip {
        crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 493; @condition b"d.outsetClip\0".as_ptr(); d.outset_clip.is_some());
        let bounds = compute_nocheck_cubic_bounds(pts);
        if !geometric_overlap(d.outset_clip.unwrap(), &bounds) {
            return;
        } else if geometric_contains(inset_clip, &bounds) {
            clip = None;
        }
    }
    if quick_cubic_niceness_check(pts) {
        hair_cubic(pts, clip, d.blitter.as_mut(), d.lineproc);
    } else {
        let mut tmp = [SkPoint::default(); 13];
        let mut ts = [0.0; 3];
        let count = crate::src::core::SkGeometry::SkChopCubicAtMaxCurvature(
            pts,
            Some(&mut tmp),
            Some(&mut ts),
        );
        for i in 0..count {
            let pts = <&[SkPoint; 4]>::try_from(&tmp[i * 3..i * 3 + 4]).unwrap();
            hair_cubic(pts, clip, d.blitter.as_mut(), d.lineproc);
        }
    }
}
// Source: src/core/SkScan_Hairline.cpp:516-529
fn compute_quad_level(pts: &[SkPoint; 3]) -> usize {
    let d = compute_int_quad_dist(pts);
    let mut level = ((33 - d.leading_zeros()) >> 1) as usize;
    if level > MAX_QUAD_SUBDIVIDE_LEVEL {
        level = MAX_QUAD_SUBDIVIDE_LEVEL;
    }
    level
}
// Source: src/core/SkScan_Hairline.cpp:531-533
fn optional_eq<T: PartialEq>(opt: Option<T>, other: T) -> bool {
    opt.is_some_and(|v| v == other)
}
// Source: src/core/SkScan_Hairline.cpp:539-588
fn extend_pts<const CAP: u8>(
    prev: Option<crate::include::core::SkPathTypes::SkPathVerb>,
    next: Option<crate::include::core::SkPathTypes::SkPathVerb>,
    pts: &mut [SkPoint],
) {
    use crate::include::core::SkPathTypes::SkPathVerb::*;
    crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 542; @condition b"SkPaint::kSquare_Cap == capStyle || SkPaint::kRound_Cap == capStyle\0".as_ptr(); CAP == 2 || CAP == 1);
    let cap_outset = if CAP == 2 {
        0.5
    } else {
        core::f32::consts::PI / 8.0
    };
    if optional_eq(prev, kMove) {
        let mut ctrl = 0;
        let mut controls = pts.len() - 1;
        let mut tangent;
        loop {
            ctrl += 1;
            tangent = pts[0] - pts[ctrl];
            if !tangent.is_zero() {
                break;
            }
            controls -= 1;
            if controls == 0 {
                break;
            }
        }
        if tangent.is_zero() {
            tangent.set(1.0, 0.0);
            controls = pts.len() - 1;
        } else {
            tangent.normalize();
        }
        let mut first = 0;
        loop {
            pts[first].fX += tangent.fX * cap_outset;
            pts[first].fY += tangent.fY * cap_outset;
            first += 1;
            controls += 1;
            if controls >= pts.len() {
                break;
            }
        }
    }
    if next.is_none() || next == Some(kMove) || next == Some(kClose) {
        let end = pts.len() - 1;
        let mut ctrl = end;
        let mut controls = pts.len() - 1;
        let mut tangent;
        loop {
            ctrl -= 1;
            tangent = pts[end] - pts[ctrl];
            if !tangent.is_zero() {
                break;
            }
            controls -= 1;
            if controls == 0 {
                break;
            }
        }
        if tangent.is_zero() {
            tangent.set(-1.0, 0.0);
            controls = pts.len() - 1;
        } else {
            tangent.normalize();
        }
        let mut last = end;
        loop {
            pts[last].fX += tangent.fX * cap_outset;
            pts[last].fY += tangent.fY * cap_outset;
            controls += 1;
            if controls >= pts.len() {
                break;
            }
            last -= 1;
        }
    }
}
// Source: src/core/SkScan_Hairline.cpp:590-600
unsafe fn hairconic(p: &[SkPoint; 3], d: &mut DrawingParameters<'_>, conic_weight: f32) {
    let mut converter = crate::src::core::SkGeometry::SkAutoConicToQuads::new();
    let tol = 1.0 / 4.0;
    let quad_ptr = converter.computeQuads(p, conic_weight, tol).as_ptr();
    let count = converter.countQuads();
    for i in 0..count {
        // Native computeQuads storage remains initialized and immovable until
        // converter drops; adjacent quadratics overlap their endpoint.
        let p = unsafe { &*quad_ptr.add(i * 2).cast::<[SkPoint; 3]>() };
        let level = compute_quad_level(p);
        hairquad(p, d, level);
    }
}
// Source: src/core/SkScan_Hairline.cpp:603-619
fn is_next_contour_closed(mut scanner: crate::src::core::SkPathIter::SkPathIter<'_>) -> bool {
    use crate::include::core::SkPathTypes::SkPathVerb::*;
    let verb = scanner
        .next()
        .expect("valid move is followed by a contour record")
        .fVerb;
    if verb == kClose {
        return true;
    }
    if verb == kMove {
        return false;
    }
    while let Some(next) = scanner.peekNextVerb() {
        if next == kMove {
            return false;
        }
        if next == kClose {
            return true;
        }
        scanner.next();
    }
    false
}
// Source: src/core/SkScan_Hairline.cpp:622-682
unsafe fn hair_path<const CAP: u8>(
    raw: &crate::src::core::SkPathRaw::SkPathRaw,
    rclip: &crate::src::core::SkRasterClip::SkRasterClip,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    lineproc: HairRgnProc,
) {
    if raw.empty() {
        return;
    }
    let cap_out = if CAP == 0 { 1 } else { 2 };
    let ibounds = raw.bounds().roundOut().make_outset(cap_out, cap_out);
    if rclip.quick_reject(&ibounds) {
        return;
    }
    if !rclip.quick_contains(&ibounds) {
        if rclip.is_bw() {
            hair_path_segments::<CAP>(
                raw,
                Some(rclip.bw_rgn()),
                rclip.is_rect(),
                blitter.as_mut(),
                lineproc,
            );
        } else {
            let mut wrap = Box::pin(
                crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                    rclip,
                    blitter.as_mut(),
                ),
            );
            let (clip, mut blitter) = wrap.as_mut().parts();
            hair_path_segments::<CAP>(raw, Some(clip), rclip.is_rect(), blitter.as_mut(), lineproc);
        }
    } else {
        hair_path_segments::<CAP>(raw, None, false, blitter.as_mut(), lineproc);
    }
}
// Source: src/core/SkScan_Hairline.cpp:665-682
// Source: src/core/SkScan_Hairline.cpp:683-760
// Rust splits clipping-owner selection from the borrowed draw scope; all
// numerical culling, caps, subdivision and original iterator order are retained.
unsafe fn hair_path_segments<const CAP: u8>(
    raw: &crate::src::core::SkPathRaw::SkPathRaw,
    clip: Option<&SkRegion>,
    is_rect: bool,
    mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    lineproc: HairRgnProc,
) {
    use crate::include::core::SkPathTypes::SkPathVerb::*;
    use crate::src::core::SkPath::SkPath;
    let mut inset_storage = SkRect::default();
    let mut outset_storage = SkRect::default();
    if let Some(clip) = clip {
        inset_storage.set(clip.get_bounds());
        outset_storage = inset_storage.makeOutset(1.0, 1.0);
        inset_storage.inset(1.0, 1.0);
        if is_inverted(&inset_storage) {
            inset_storage.setEmpty();
        }
    }
    let inset_clip = if clip.is_some() && is_rect {
        Some(&inset_storage)
    } else {
        None
    };
    let outset_clip = if clip.is_some() {
        Some(&outset_storage)
    } else {
        None
    };
    let mut pts = [SkPoint::default(); 4];
    let mut first_pt = SkPoint::default();
    let mut last_pt = SkPoint::default();
    let _converter = crate::src::core::SkGeometry::SkAutoConicToQuads::new();
    let mut is_closed = false;
    let mut params = DrawingParameters {
        clip,
        inset_clip,
        outset_clip,
        blitter: blitter.as_mut(),
        lineproc,
    };
    let is_butt_cap = CAP == 0;
    let mut prev_verb = None;
    let mut iter = raw.iter();
    while let Some(rec) = iter.next() {
        let verb = rec.fVerb;
        let count = rec.fPoints.len();
        pts[..count].copy_from_slice(rec.fPoints);
        let weight = rec.fConicWeight;
        let next_verb = iter.peekNextVerb();
        match verb {
            kMove => {
                first_pt = pts[0];
                last_pt = pts[0];
                is_closed = !is_butt_cap && is_next_contour_closed(iter.clone());
            }
            kLine => {
                if !is_butt_cap && (!is_closed || SkPath::IsLineDegenerate(pts[0], pts[1], true)) {
                    extend_pts::<CAP>(prev_verb, next_verb, &mut pts[..2]);
                }
                lineproc(&pts[..2], params.clip, params.blitter.as_mut());
                last_pt = pts[1];
            }
            kQuad => {
                if !is_butt_cap
                    && (!is_closed || SkPath::IsQuadDegenerate(pts[0], pts[1], pts[2], true))
                {
                    extend_pts::<CAP>(prev_verb, next_verb, &mut pts[..3]);
                }
                let p = <&[SkPoint; 3]>::try_from(&pts[..3]).unwrap();
                hairquad(p, &mut params, compute_quad_level(p));
                last_pt = pts[2];
            }
            kConic => {
                if !is_butt_cap
                    && (!is_closed || SkPath::IsQuadDegenerate(pts[0], pts[1], pts[2], true))
                {
                    extend_pts::<CAP>(prev_verb, next_verb, &mut pts[..3]);
                }
                let p = <&[SkPoint; 3]>::try_from(&pts[..3]).unwrap();
                hairconic(p, &mut params, weight);
                last_pt = pts[2];
            }
            kCubic => {
                if !is_butt_cap
                    && (!is_closed
                        || SkPath::IsCubicDegenerate(pts[0], pts[1], pts[2], pts[3], true))
                {
                    extend_pts::<CAP>(prev_verb, next_verb, &mut pts);
                }
                haircubic(&pts, &mut params, MAX_CUBIC_SUBDIVIDE_LEVEL);
                last_pt = pts[3];
            }
            kClose => {
                pts[0] = last_pt;
                pts[1] = first_pt;
                if !is_butt_cap && optional_eq(prev_verb, kMove) {
                    extend_pts::<CAP>(prev_verb, next_verb, &mut pts[..2]);
                }
                lineproc(&pts[..2], params.clip, params.blitter.as_mut());
            }
        }
        if !is_butt_cap {
            if optional_eq(prev_verb, kMove)
                && (verb as u8) >= (kLine as u8)
                && (verb as u8) <= (kCubic as u8)
            {
                first_pt = pts[0];
            }
            prev_verb = Some(verb);
        }
    }
}
impl SkScan {
    // Source: src/core/SkScan_Hairline.cpp:762-764
    pub unsafe fn HairPath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        hair_path::<0>(raw, clip, blitter.as_mut(), Self::HairLineRgn);
    }
    // Source: src/core/SkScan_Hairline.cpp:766-768
    pub unsafe fn AntiHairPath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        hair_path::<0>(raw, clip, blitter.as_mut(), Self::AntiHairLineRgn);
    }
    // Source: src/core/SkScan_Hairline.cpp:770-772
    pub unsafe fn HairSquarePath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        hair_path::<2>(raw, clip, blitter.as_mut(), Self::HairLineRgn);
    }
    // Source: src/core/SkScan_Hairline.cpp:774-776
    pub unsafe fn AntiHairSquarePath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        hair_path::<2>(raw, clip, blitter.as_mut(), Self::AntiHairLineRgn);
    }
    // Source: src/core/SkScan_Hairline.cpp:778-780
    pub unsafe fn HairRoundPath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        hair_path::<1>(raw, clip, blitter.as_mut(), Self::HairLineRgn);
    }
    // Source: src/core/SkScan_Hairline.cpp:782-784
    pub unsafe fn AntiHairRoundPath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        hair_path::<1>(raw, clip, blitter.as_mut(), Self::AntiHairLineRgn);
    }
}
impl SkScan {
    // Source: src/core/SkScan_Hairline.cpp:788-831
    pub unsafe fn FrameRect(
        r: &SkRect,
        stroke_size: &SkPoint,
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        crate::SkASSERT!(@source b"src/core/SkScan_Hairline.cpp\0".as_ptr(), 790; @condition b"strokeSize.fX >= 0 && strokeSize.fY >= 0\0".as_ptr(); stroke_size.fX >= 0.0 && stroke_size.fY >= 0.0);
        if stroke_size.fX < 0.0 || stroke_size.fY < 0.0 {
            return;
        }
        let dx = stroke_size.fX;
        let dy = stroke_size.fY;
        let rx = dx / 2.0;
        let ry = dy / 2.0;
        let mut outer = SkRect::MakeLTRB(r.fLeft - rx, r.fTop - ry, r.fRight + rx, r.fBottom + ry);
        if r.width() <= dx || r.height() <= dy {
            if r.width() == 0.0 {
                outer.fTop = r.fTop;
                outer.fBottom = r.fBottom;
            }
            if r.height() == 0.0 {
                outer.fLeft = r.fLeft;
                outer.fRight = r.fRight;
            }
            Self::FillRectRasterClip(&outer, clip, blitter.as_mut());
            return;
        }
        let mut tmp = SkRect::MakeLTRB(outer.fLeft, outer.fTop, outer.fRight, outer.fTop + dy);
        Self::FillRectRasterClip(&tmp, clip, blitter.as_mut());
        tmp.fTop = outer.fBottom - dy;
        tmp.fBottom = outer.fBottom;
        Self::FillRectRasterClip(&tmp, clip, blitter.as_mut());
        tmp.setLTRB(
            outer.fLeft,
            outer.fTop + dy,
            outer.fLeft + dx,
            outer.fBottom - dy,
        );
        Self::FillRectRasterClip(&tmp, clip, blitter.as_mut());
        tmp.fLeft = outer.fRight - dx;
        tmp.fRight = outer.fRight;
        Self::FillRectRasterClip(&tmp, clip, blitter.as_mut());
    }
    // Source: src/core/SkScan_Hairline.cpp:833-849
    pub unsafe fn HairLine(
        pts: &[SkPoint],
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_bw() {
            Self::HairLineRgn(pts, Some(clip.bw_rgn()), blitter.as_mut());
        } else {
            let r = SkRect::BoundsOrEmpty(pts).makeOutset(0.5, 0.5);
            if !clip.quick_contains(&r.roundOut()) {
                let mut wrap = Box::pin(
                    crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                        clip,
                        blitter.as_mut(),
                    ),
                );
                let (rgn, mut blitter) = wrap.as_mut().parts();
                Self::HairLineRgn(pts, Some(rgn), blitter.as_mut());
            } else {
                Self::HairLineRgn(pts, None, blitter.as_mut());
            }
        }
    }
}

impl SkScan {
    // Source: src/core/SkScan_Hairline.cpp:851-867
    pub unsafe fn AntiHairLine(
        pts: &[SkPoint],
        clip: &crate::src::core::SkRasterClip::SkRasterClip,
        mut blitter: core::pin::Pin<&mut dyn SkBlitter>,
    ) {
        if clip.is_bw() {
            Self::AntiHairLineRgn(pts, Some(clip.bw_rgn()), blitter.as_mut());
        } else {
            let r = SkRect::BoundsOrEmpty(pts);
            if !clip.quick_contains(&r.roundOut().make_outset(1, 1)) {
                let mut wrap = Box::pin(
                    crate::src::core::SkRasterClip::SkAAClipBlitterWrapper::from_clip(
                        clip,
                        blitter.as_mut(),
                    ),
                );
                let (rgn, mut blitter) = wrap.as_mut().parts();
                Self::AntiHairLineRgn(pts, Some(rgn), blitter.as_mut());
            } else {
                Self::AntiHairLineRgn(pts, None, blitter.as_mut());
            }
        }
    }
}

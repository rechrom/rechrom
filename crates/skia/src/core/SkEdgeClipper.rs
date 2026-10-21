// Copyright 2009 The Android Open Source Project. BSD-style license.
use crate::include::core::SkPathTypes::SkPathVerb;
use crate::src::core::SkLineClipper::SkLineClipper;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::SkRect;
// Source: src/core/SkEdgeClipper.cpp:21-52
fn quick_reject(bounds: &SkRect, clip: &SkRect) -> bool {
    bounds.fTop >= clip.fBottom || bounds.fBottom <= clip.fTop
}
fn clamp_le(value: &mut f32, max: f32) {
    if *value > max {
        *value = max;
    }
}
fn clamp_ge(value: &mut f32, min: f32) {
    if *value < min {
        *value = min;
    }
}
fn sort_increasing_Y<const N: usize>(dst: &mut [SkPoint; N], src: &[SkPoint; N]) -> bool {
    if src[0].fY > src[N - 1].fY {
        for i in 0..N {
            dst[i] = src[N - i - 1];
        }
        true
    } else {
        dst.copy_from_slice(src);
        false
    }
}
impl SkEdgeClipper {
    // Source: src/core/SkEdgeClipper.h:27-27
    pub fn new(canCullToTheRight: bool) -> Self {
        Self {
            fCurrPoint: 0,
            fCurrVerb: 0,
            fCurrVerbStop: 0,
            fCanCullToTheRight: canCullToTheRight,
            fPoints: [SkPoint::default(); 54],
            fVerbs: [SkPathVerb::kLine; 18],
        }
    }
    // Source: src/core/SkEdgeClipper.h:35-35
    pub fn canCullToTheRight(&self) -> bool {
        self.fCanCullToTheRight
    }
    // Source: src/core/SkEdgeClipper.cpp:54-69
    pub fn clipLine(&mut self, p0: SkPoint, p1: SkPoint, clip: &SkRect) -> bool {
        self.fCurrPoint = 0;
        self.fCurrVerb = 0;
        let mut lines = [SkPoint::default(); SkLineClipper::kMaxPoints];
        let pts = [p0, p1];
        let lineCount = SkLineClipper::ClipLine(&pts, clip, &mut lines, self.fCanCullToTheRight);
        for i in 0..lineCount {
            self.appendLine(lines[i], lines[i + 1]);
        }
        self.fCurrVerbStop = self.fCurrVerb;
        self.fCurrPoint = 0;
        self.fCurrVerb = 0;
        self.fCurrVerbStop != self.fCurrVerb
    }
}
// Source: src/core/SkEdgeClipper.cpp:73-99
fn chopMonoQuadAt(c0: f32, c1: f32, c2: f32, target: f32, t: &mut f32) -> bool {
    let a = c0 - c1 - c1 + c2;
    let b = 2.0 * (c1 - c0);
    let c = c0 - target;
    let mut roots = [0.0; 2];
    let count = crate::src::core::SkGeometry::SkFindUnitQuadRoots(a, b, c, &mut roots);
    if count != 0 {
        *t = roots[0];
        return true;
    }
    false
}
fn chopMonoQuadAtY(pts: &[SkPoint; 3], y: f32, t: &mut f32) -> bool {
    chopMonoQuadAt(pts[0].fY, pts[1].fY, pts[2].fY, y, t)
}
fn chopMonoQuadAtX(pts: &[SkPoint; 3], x: f32, t: &mut f32) -> bool {
    chopMonoQuadAt(pts[0].fX, pts[1].fX, pts[2].fX, x, t)
}
// Source: src/core/SkEdgeClipper.cpp:101-147
fn chop_quad_in_Y(pts: &mut [SkPoint; 3], clip: &SkRect) {
    use crate::src::core::SkGeometry::SkChopQuadAt;
    let mut t = 0.0;
    let mut tmp = [SkPoint::default(); 5];
    if pts[0].fY < clip.fTop {
        if chopMonoQuadAtY(pts, clip.fTop, &mut t) {
            SkChopQuadAt(pts, &mut tmp, t);
            tmp[2].fY = clip.fTop;
            clamp_ge(&mut tmp[3].fY, clip.fTop);
            pts[0] = tmp[2];
            pts[1] = tmp[3];
        } else {
            for p in pts.iter_mut() {
                if p.fY < clip.fTop {
                    p.fY = clip.fTop;
                }
            }
        }
    }
    if pts[2].fY > clip.fBottom {
        if chopMonoQuadAtY(pts, clip.fBottom, &mut t) {
            SkChopQuadAt(pts, &mut tmp, t);
            clamp_le(&mut tmp[1].fY, clip.fBottom);
            tmp[2].fY = clip.fBottom;
            pts[1] = tmp[1];
            pts[2] = tmp[2];
        } else {
            for p in pts.iter_mut() {
                if p.fY > clip.fBottom {
                    p.fY = clip.fBottom;
                }
            }
        }
    }
}

impl SkEdgeClipper {
    // Source: src/core/SkEdgeClipper.cpp:150-225
    fn clipMonoQuad(&mut self, srcPts: &[SkPoint; 3], clip: &SkRect) {
        use crate::src::core::SkGeometry::SkChopQuadAt;
        let mut pts = [SkPoint::default(); 3];
        let mut reverse = sort_increasing_Y(&mut pts, srcPts);
        if pts[2].fY <= clip.fTop || pts[0].fY >= clip.fBottom {
            return;
        }
        chop_quad_in_Y(&mut pts, clip);
        if pts[0].fX > pts[2].fX {
            pts.swap(0, 2);
            reverse = !reverse;
        }
        crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 167; @condition b"pts[0].fX <= pts[1].fX\0".as_ptr(); pts[0].fX <= pts[1].fX);
        crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 168; @condition b"pts[1].fX <= pts[2].fX\0".as_ptr(); pts[1].fX <= pts[2].fX);
        if pts[2].fX <= clip.fLeft {
            self.appendVLine(clip.fLeft, pts[0].fY, pts[2].fY, reverse);
            return;
        }
        if pts[0].fX >= clip.fRight {
            if !self.canCullToTheRight() {
                self.appendVLine(clip.fRight, pts[0].fY, pts[2].fY, reverse);
            }
            return;
        }
        let mut t = 0.0;
        let mut tmp = [SkPoint::default(); 5];
        if pts[0].fX < clip.fLeft {
            if chopMonoQuadAtX(&pts, clip.fLeft, &mut t) {
                SkChopQuadAt(&pts, &mut tmp, t);
                self.appendVLine(clip.fLeft, tmp[0].fY, tmp[2].fY, reverse);
                tmp[2].fX = clip.fLeft;
                clamp_ge(&mut tmp[3].fX, clip.fLeft);
                pts[0] = tmp[2];
                pts[1] = tmp[3];
            } else {
                self.appendVLine(clip.fLeft, pts[0].fY, pts[2].fY, reverse);
                return;
            }
        }
        if pts[2].fX > clip.fRight {
            if chopMonoQuadAtX(&pts, clip.fRight, &mut t) {
                SkChopQuadAt(&pts, &mut tmp, t);
                clamp_le(&mut tmp[1].fX, clip.fRight);
                tmp[2].fX = clip.fRight;
                self.appendQuad((&tmp[..3]).try_into().unwrap(), reverse);
                self.appendVLine(clip.fRight, tmp[2].fY, tmp[4].fY, reverse);
            } else {
                // std::min preserves the first operand for unordered values.
                if clip.fRight < pts[1].fX {
                    pts[1].fX = clip.fRight;
                }
                if clip.fRight < pts[2].fX {
                    pts[2].fX = clip.fRight;
                }
                self.appendQuad(&pts, reverse);
            }
        } else {
            self.appendQuad(&pts, reverse);
        }
    }
    // Source: src/core/SkEdgeClipper.cpp:227-252
    pub fn clipQuad(&mut self, srcPts: &[SkPoint; 3], clip: &SkRect) -> bool {
        use crate::src::core::SkGeometry::{SkChopQuadAtXExtrema, SkChopQuadAtYExtrema};
        self.fCurrPoint = 0;
        self.fCurrVerb = 0;
        let bounds = SkRect::BoundsOrEmpty(srcPts);
        if !quick_reject(&bounds, clip) {
            let mut monoY = [SkPoint::default(); 5];
            let countY = SkChopQuadAtYExtrema(srcPts, &mut monoY);
            for y in 0..=countY {
                let mut monoX = [SkPoint::default(); 5];
                let countX = SkChopQuadAtXExtrema(
                    (&monoY[y * 2..y * 2 + 3]).try_into().unwrap(),
                    &mut monoX,
                );
                for x in 0..=countX {
                    self.clipMonoQuad((&monoX[x * 2..x * 2 + 3]).try_into().unwrap(), clip);
                    crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 241; @condition b"fCurrVerb - fVerbs < kMaxVerbs\0".as_ptr(); self.fCurrVerb < 18);
                    crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 242; @condition b"fCurrPoint - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint <= 54);
                }
            }
        }
        self.fCurrVerbStop = self.fCurrVerb;
        self.fCurrPoint = 0;
        self.fCurrVerb = 0;
        self.fCurrVerbStop != self.fCurrVerb
    }
}
// Source: src/core/SkEdgeClipper.cpp:255-278
fn mono_cubic_closestT(src: &[f32; 4], mut x: f32) -> f32 {
    let mut t = 0.5;
    let mut bestT = 0.0;
    let mut step = 0.25;
    let d = src[0];
    let a = src[3] + 3.0 * (src[1] - src[2]) - d;
    let b = 3.0 * (src[2] - src[1] - src[1] + d);
    let c = 3.0 * (src[1] - d);
    x -= d;
    let mut closest = f32::MAX;
    loop {
        let loc = ((a * t + b) * t + c) * t;
        let dist = (loc - x).abs();
        if closest > dist {
            closest = dist;
            bestT = t;
        }
        let lastT = t;
        t += if loc < x { step } else { -step };
        step *= 0.5;
        if !(closest > 0.25 && lastT != t) {
            break;
        }
    }
    bestT
}
// Source: src/core/SkEdgeClipper.cpp:280-285
fn chop_mono_cubic_at_y(src: &[SkPoint; 4], y: f32, dst: &mut [SkPoint; 7]) {
    use crate::src::core::SkGeometry::{SkChopCubicAt, SkChopMonoCubicAtY};
    if SkChopMonoCubicAtY(src, y, dst) {
        return;
    }
    SkChopCubicAt(src, dst, mono_cubic_closestT(&src.map(|p| p.fY), y));
}
// Source: src/core/SkEdgeClipper.cpp:288-332
fn chop_cubic_in_Y(pts: &mut [SkPoint; 4], clip: &SkRect) {
    if pts[0].fY < clip.fTop {
        let mut tmp = [SkPoint::default(); 7];
        chop_mono_cubic_at_y(pts, clip.fTop, &mut tmp);
        if tmp[3].fY < clip.fTop && tmp[4].fY < clip.fTop && tmp[5].fY < clip.fTop {
            let tmp2: [SkPoint; 4] = tmp[3..7].try_into().unwrap();
            chop_mono_cubic_at_y(&tmp2, clip.fTop, &mut tmp);
        }
        tmp[3].fY = clip.fTop;
        clamp_ge(&mut tmp[4].fY, clip.fTop);
        pts[0] = tmp[3];
        pts[1] = tmp[4];
        pts[2] = tmp[5];
    }
    if pts[3].fY > clip.fBottom {
        let mut tmp = [SkPoint::default(); 7];
        chop_mono_cubic_at_y(pts, clip.fBottom, &mut tmp);
        tmp[3].fY = clip.fBottom;
        clamp_le(&mut tmp[2].fY, clip.fBottom);
        pts[1] = tmp[1];
        pts[2] = tmp[2];
        pts[3] = tmp[3];
    }
}
// Source: src/core/SkEdgeClipper.cpp:334-339
fn chop_mono_cubic_at_x(src: &[SkPoint; 4], x: f32, dst: &mut [SkPoint; 7]) {
    use crate::src::core::SkGeometry::{SkChopCubicAt, SkChopMonoCubicAtX};
    if SkChopMonoCubicAtX(src, x, dst) {
        return;
    }
    SkChopCubicAt(src, dst, mono_cubic_closestT(&src.map(|p| p.fX), x));
}
impl SkEdgeClipper {
    // Source: src/core/SkEdgeClipper.cpp:342-403
    fn clipMonoCubic(&mut self, src: &[SkPoint; 4], clip: &SkRect) {
        let mut pts = [SkPoint::default(); 4];
        let mut reverse = sort_increasing_Y(&mut pts, src);
        if pts[3].fY <= clip.fTop || pts[0].fY >= clip.fBottom {
            return;
        }
        chop_cubic_in_Y(&mut pts, clip);
        if pts[0].fX > pts[3].fX {
            pts.swap(0, 3);
            pts.swap(1, 2);
            reverse = !reverse;
        }
        if pts[3].fX <= clip.fLeft {
            self.appendVLine(clip.fLeft, pts[0].fY, pts[3].fY, reverse);
            return;
        }
        if pts[0].fX >= clip.fRight {
            if !self.canCullToTheRight() {
                self.appendVLine(clip.fRight, pts[0].fY, pts[3].fY, reverse);
            }
            return;
        }
        if pts[0].fX < clip.fLeft {
            let mut tmp = [SkPoint::default(); 7];
            chop_mono_cubic_at_x(&pts, clip.fLeft, &mut tmp);
            self.appendVLine(clip.fLeft, tmp[0].fY, tmp[3].fY, reverse);
            tmp[3].fX = clip.fLeft;
            clamp_ge(&mut tmp[4].fX, clip.fLeft);
            pts[0] = tmp[3];
            pts[1] = tmp[4];
            pts[2] = tmp[5];
        }
        if pts[3].fX > clip.fRight {
            let mut tmp = [SkPoint::default(); 7];
            chop_mono_cubic_at_x(&pts, clip.fRight, &mut tmp);
            tmp[3].fX = clip.fRight;
            clamp_le(&mut tmp[2].fX, clip.fRight);
            self.appendCubic((&tmp[..4]).try_into().unwrap(), reverse);
            self.appendVLine(clip.fRight, tmp[3].fY, tmp[6].fY, reverse);
        } else {
            self.appendCubic(&pts, reverse);
        }
    }
}
// Source: src/core/SkEdgeClipper.cpp:405-418
fn compute_cubic_bounds(pts: &[SkPoint; 4]) -> SkRect {
    SkRect::BoundsOrEmpty(pts)
}
fn too_big_for_reliable_float_math(r: &SkRect) -> bool {
    let limit = (1 << 22) as f32;
    r.fLeft < -limit || r.fTop < -limit || r.fRight > limit || r.fBottom > limit
}
impl SkEdgeClipper {
    // Source: src/core/SkEdgeClipper.cpp:420-454
    pub fn clipCubic(&mut self, srcPts: &[SkPoint; 4], clip: &SkRect) -> bool {
        use crate::src::core::SkGeometry::{SkChopCubicAtXExtrema, SkChopCubicAtYExtrema};
        self.fCurrPoint = 0;
        self.fCurrVerb = 0;
        let bounds = compute_cubic_bounds(srcPts);
        if bounds.fBottom > clip.fTop && bounds.fTop < clip.fBottom {
            if too_big_for_reliable_float_math(&bounds) {
                return self.clipLine(srcPts[0], srcPts[3], clip);
            } else {
                let mut monoY = [SkPoint::default(); 10];
                let countY = SkChopCubicAtYExtrema(srcPts, &mut monoY);
                for y in 0..=countY {
                    let mut monoX = [SkPoint::default(); 10];
                    let countX = SkChopCubicAtXExtrema(
                        (&monoY[y * 3..y * 3 + 4]).try_into().unwrap(),
                        &mut monoX,
                    );
                    for x in 0..=countX {
                        self.clipMonoCubic((&monoX[x * 3..x * 3 + 4]).try_into().unwrap(), clip);
                        crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 443; @condition b"fCurrVerb - fVerbs < kMaxVerbs\0".as_ptr(); self.fCurrVerb<18);
                        crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 444; @condition b"fCurrPoint - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint<=54);
                    }
                }
            }
        }
        self.fCurrVerbStop = self.fCurrVerb;
        self.fCurrPoint = 0;
        self.fCurrVerb = 0;
        self.fCurrVerbStop != self.fCurrVerb
    }
    // Source: src/core/SkEdgeClipper.cpp:458-464
    fn appendLine(&mut self, p0: SkPoint, p1: SkPoint) {
        self.fVerbs[self.fCurrVerb] = SkPathVerb::kLine;
        self.fCurrVerb += 1;
        crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 460; @condition b"fCurrPoint + 2 - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint + 2 <= 54);
        self.fPoints[self.fCurrPoint] = p0;
        self.fPoints[self.fCurrPoint + 1] = p1;
        self.fCurrPoint += 2;
    }
    // Source: src/core/SkEdgeClipper.cpp:466-477
    fn appendVLine(&mut self, x: f32, mut y0: f32, mut y1: f32, reverse: bool) {
        self.fVerbs[self.fCurrVerb] = SkPathVerb::kLine;
        self.fCurrVerb += 1;
        if reverse {
            core::mem::swap(&mut y0, &mut y1);
        }
        crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 473; @condition b"fCurrPoint + 2 - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint + 2 <= 54);
        self.fPoints[self.fCurrPoint] = SkPoint::make(x, y0);
        self.fPoints[self.fCurrPoint + 1] = SkPoint::make(x, y1);
        self.fCurrPoint += 2;
    }
    // Source: src/core/SkEdgeClipper.cpp:479-492
    fn appendQuad(&mut self, pts: &[SkPoint; 3], reverse: bool) {
        self.fVerbs[self.fCurrVerb] = SkPathVerb::kQuad;
        self.fCurrVerb += 1;
        crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 482; @condition b"fCurrPoint + 3 - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint + 3 <= 54);
        if reverse {
            self.fPoints[self.fCurrPoint] = pts[2];
            self.fPoints[self.fCurrPoint + 2] = pts[0];
        } else {
            self.fPoints[self.fCurrPoint] = pts[0];
            self.fPoints[self.fCurrPoint + 2] = pts[2];
        }
        self.fPoints[self.fCurrPoint + 1] = pts[1];
        self.fCurrPoint += 3;
    }
    // Source: src/core/SkEdgeClipper.cpp:494-506
    fn appendCubic(&mut self, pts: &[SkPoint; 4], reverse: bool) {
        self.fVerbs[self.fCurrVerb] = SkPathVerb::kCubic;
        self.fCurrVerb += 1;
        crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 497; @condition b"fCurrPoint + 4 - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint + 4 <= 54);
        if reverse {
            for i in 0..4 {
                self.fPoints[self.fCurrPoint + i] = pts[3 - i];
            }
        } else {
            self.fPoints[self.fCurrPoint..self.fCurrPoint + 4].copy_from_slice(pts);
        }
        self.fCurrPoint += 4;
    }
    // Source: src/core/SkEdgeClipper.cpp:508-536
    pub fn next(&mut self, pts: &mut [SkPoint; 4]) -> Option<SkPathVerb> {
        crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 509; @condition b"fCurrVerb <= fCurrVerbStop\0".as_ptr(); self.fCurrVerb <= self.fCurrVerbStop);
        if self.fCurrVerb >= self.fCurrVerbStop {
            return None;
        }
        let verb = self.fVerbs[self.fCurrVerb];
        self.fCurrVerb += 1;
        let count = match verb {
            SkPathVerb::kLine => {
                crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 517; @condition b"fCurrPoint + 2 - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint + 2 <= 54);
                2
            }
            SkPathVerb::kQuad => {
                crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 522; @condition b"fCurrPoint + 3 - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint + 3 <= 54);
                3
            }
            SkPathVerb::kCubic => {
                crate::SkASSERT_RELEASE!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 527; @condition b"fCurrPoint + 4 - fPoints <= kMaxPoints\0".as_ptr(); self.fCurrPoint + 4 <= 54);
                4
            }
            _ => {
                crate::SkDEBUGFAIL!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 532; b"unexpected verb in quadclippper2 iter\0".as_ptr());
                0
            }
        };
        assert!(self.fCurrPoint + count <= 54);
        pts[..count].copy_from_slice(&self.fPoints[self.fCurrPoint..self.fCurrPoint + count]);
        self.fCurrPoint += count;
        Some(verb)
    }
}

// Source: src/core/SkEdgeClipper.cpp:541-555
#[cfg(sk_debug)]
unsafe fn assert_monotonic(coord: *const f32, count: usize) {
    if coord.read() > coord.add((count - 1) * 2).read() {
        for i in 1..count {
            crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 544; @condition b"coord[2 * (i - 1)] >= coord[i * 2]\0".as_ptr(); coord.add(2*(i-1)).read()>=coord.add(i*2).read());
        }
    } else if coord.read() < coord.add((count - 1) * 2).read() {
        for i in 1..count {
            crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 548; @condition b"coord[2 * (i - 1)] <= coord[i * 2]\0".as_ptr(); coord.add(2*(i-1)).read()<=coord.add(i*2).read());
        }
    } else {
        for i in 1..count {
            crate::SkASSERT!(@source b"src/core/SkEdgeClipper.cpp\0".as_ptr(), 552; @condition b"coord[2 * (i - 1)] == coord[i * 2]\0".as_ptr(); coord.add(2*(i-1)).read()==coord.add(i*2).read());
        }
    }
}
// Source: src/core/SkEdgeClipper.h:66-66
// Source: src/core/SkQuadClipper.h:67-67
// Source: src/core/SkEdgeClipper.cpp:557-561
#[cfg(sk_debug)]
pub unsafe fn sk_assert_monotonic_y(pts: *const SkPoint, count: i32) {
    if count > 1 {
        assert_monotonic(core::ptr::addr_of!((*pts).fY), count as usize);
    }
}
// Source: src/core/SkEdgeClipper.h:65-65
// Source: src/core/SkQuadClipper.h:66-66
// Source: src/core/SkEdgeClipper.cpp:563-567
#[cfg(sk_debug)]
pub unsafe fn sk_assert_monotonic_x(pts: *const SkPoint, count: i32) {
    if count > 1 {
        assert_monotonic(core::ptr::addr_of!((*pts).fX), count as usize);
    }
}
impl SkEdgeClipper {
    // Source: src/core/SkEdgeClipper.cpp:570-610
    pub fn ClipPath(
        raw: &crate::src::core::SkPathRaw::SkPathRaw<'_>,
        clip: &SkRect,
        canCullToTheRight: bool,
        mut consume: impl FnMut(&mut Self, bool),
    ) {
        use crate::src::core::SkGeometry::SkAutoConicToQuads;
        use crate::src::core::SkPathPriv::{Edge, SkPathEdgeIter};
        let mut quadder = SkAutoConicToQuads::new();
        const CONIC_TOL: f32 = 0.25;
        let mut iter = SkPathEdgeIter::new(raw);
        let mut clipper = Self::new(canCullToTheRight);
        while let Some(e) = iter.next() {
            match e.fEdge {
                Edge::kLine => {
                    if clipper.clipLine(e.fPts[0], e.fPts[1], clip) {
                        consume(&mut clipper, e.fIsNewContour);
                    }
                }
                Edge::kQuad => {
                    if clipper.clipQuad(e.fPts.try_into().unwrap(), clip) {
                        consume(&mut clipper, e.fIsNewContour);
                    }
                }
                Edge::kConic => {
                    let quadPts = quadder.computeQuads(
                        e.fPts.try_into().unwrap(),
                        e.fConicWeight.unwrap(),
                        CONIC_TOL,
                    );
                    for pts in quadPts.windows(3).step_by(2) {
                        if clipper.clipQuad(pts.try_into().unwrap(), clip) {
                            consume(&mut clipper, e.fIsNewContour);
                        }
                    }
                }
                Edge::kCubic => {
                    if clipper.clipCubic(e.fPts.try_into().unwrap(), clip) {
                        consume(&mut clipper, e.fIsNewContour);
                    }
                }
            }
        }
    }
}

// Source: src/core/SkEdgeClipper.h:45-54
// Cursor indices retain the original fixed buffers while allowing this value to move.
pub struct SkEdgeClipper {
    fCurrPoint: usize,
    fCurrVerb: usize,
    fCurrVerbStop: usize,
    fCanCullToTheRight: bool,
    fPoints: [SkPoint; 54],
    fVerbs: [SkPathVerb; 18],
}

// Source: src/core/SkEdgeClipper.h:68-68
#[cfg(not(sk_debug))]
macro_rules! sk_assert_monotonic_x {
    ($pts:expr,$count:expr) => {{}};
}
#[cfg(not(sk_debug))]
pub(crate) use sk_assert_monotonic_x;
// Source: src/core/SkEdgeClipper.h:69-69
#[cfg(not(sk_debug))]
macro_rules! sk_assert_monotonic_y {
    ($pts:expr,$count:expr) => {{}};
}
#[cfg(not(sk_debug))]
pub(crate) use sk_assert_monotonic_y;

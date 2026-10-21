// Copyright 2009 The Android Open Source Project. BSD-style license.
use crate::src::core::SkGeometry::SkChopCubicAt;
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::{SkIRect, SkRect};
// Source: src/core/SkCubicClipper.h:24-35
pub struct SkCubicClipper {
    fClip: SkRect,
}
impl SkCubicClipper {
    // Source: src/core/SkCubicClipper.cpp:16-18
    pub fn new() -> Self {
        Self {
            fClip: SkRect {
                fLeft: 0.0,
                fTop: 0.0,
                fRight: 0.0,
                fBottom: 0.0,
            },
        }
    }
    // Source: src/core/SkCubicClipper.cpp:20-23
    pub fn setClip(&mut self, clip: &SkIRect) {
        self.fClip = SkRect {
            fLeft: clip.fLeft as f32,
            fTop: clip.fTop as f32,
            fRight: clip.fRight as f32,
            fBottom: clip.fBottom as f32,
        };
    }
    // Source: src/core/SkCubicClipper.cpp:26-31
    pub fn ChopMonoAtY(pts: &[SkPoint; 4], y: f32, t: &mut f32) -> bool {
        let ycrv = [pts[0].fY - y, pts[1].fY - y, pts[2].fY - y, pts[3].fY - y];
        Self::bisect(&ycrv, t)
    }
    // Source: src/core/SkCubicClipper.cpp:34-65
    // Optional upstream NEWTON_RAPHSON branch, separate entry point preserves
    // the source build-time choice without adding a crate-wide feature.
    pub fn ChopMonoAtYNewton(pts: &[SkPoint; 4], y: f32, t: &mut f32) -> bool {
        let ycrv = [pts[0].fY - y, pts[1].fY - y, pts[2].fY - y, pts[3].fY - y];
        let mut t1 = ycrv[0] / (ycrv[0] - ycrv[3]);
        let tol = 1.0 / 16384.0;
        let mut iters = 0;
        let converged = loop {
            let t0 = t1;
            let y01 = interp(ycrv[0], ycrv[1], t0);
            let y12 = interp(ycrv[1], ycrv[2], t0);
            let y23 = interp(ycrv[2], ycrv[3], t0);
            let y012 = interp(y01, y12, t0);
            let y123 = interp(y12, y23, t0);
            let y0123 = interp(y012, y123, t0);
            let yder = (y123 - y012) * 3.0;
            t1 -= y0123 / yder;
            let converged = (t1 - t0).abs() <= tol;
            iters += 1;
            if converged || iters >= 5 {
                break converged;
            }
        };
        *t = t1;
        if t1 < 0.0 || t1 > 1.0 {
            return false;
        }
        converged
    }
    // Source: src/core/SkCubicClipper.cpp:69-104
    fn bisect(ycrv: &[f32; 4], t: &mut f32) -> bool {
        let (mut t_neg, mut t_pos);
        if ycrv[0] < 0.0 {
            if ycrv[3] < 0.0 {
                return false;
            }
            t_neg = 0.0;
            t_pos = 1.0;
        } else if ycrv[0] > 0.0 {
            if ycrv[3] > 0.0 {
                return false;
            }
            t_neg = 1.0;
            t_pos = 0.0;
        } else {
            *t = 0.0;
            return true;
        }
        let tol = 1.0 / 65536.0;
        loop {
            let t_mid = (t_pos + t_neg) / 2.0;
            let y01 = interp(ycrv[0], ycrv[1], t_mid);
            let y12 = interp(ycrv[1], ycrv[2], t_mid);
            let y23 = interp(ycrv[2], ycrv[3], t_mid);
            let y012 = interp(y01, y12, t_mid);
            let y123 = interp(y12, y23, t_mid);
            let y0123 = interp(y012, y123, t_mid);
            if y0123 == 0.0 {
                *t = t_mid;
                return true;
            }
            if y0123 < 0.0 {
                t_neg = t_mid;
            } else {
                t_pos = t_mid;
            }
            if (t_pos - t_neg).abs() <= tol {
                break;
            }
        }
        *t = (t_neg + t_pos) / 2.0;
        true
    }
    // Source: src/core/SkCubicClipper.cpp:109-156
    pub fn clipCubic(&self, src: &[SkPoint; 4], dst: &mut [SkPoint; 4]) -> bool {
        let reverse;
        if src[0].fY > src[3].fY {
            *dst = [src[3], src[2], src[1], src[0]];
            reverse = true;
        } else {
            *dst = *src;
            reverse = false;
        }
        let ctop = self.fClip.fTop;
        let cbot = self.fClip.fBottom;
        if dst[3].fY <= ctop || dst[0].fY >= cbot {
            return false;
        }
        let mut t = 0.0;
        let mut tmp = [dst[0]; 7];
        if dst[0].fY < ctop && Self::ChopMonoAtY(dst, ctop, &mut t) {
            SkChopCubicAt(dst, &mut tmp, t);
            dst[0] = tmp[3];
            dst[1] = tmp[4];
            dst[2] = tmp[5];
        }
        if dst[3].fY > cbot && Self::ChopMonoAtY(dst, cbot, &mut t) {
            SkChopCubicAt(dst, &mut tmp, t);
            dst[1] = tmp[1];
            dst[2] = tmp[2];
            dst[3] = tmp[3];
        }
        if reverse {
            dst.swap(0, 3);
            dst.swap(1, 2);
        }
        true
    }
}
// Inlined SkScalarInterp has the exact upstream order of floating operations.
fn interp(a: f32, b: f32, t: f32) -> f32 {
    debug_assert!(t >= 0.0 && t <= 1.0);
    a + (b - a) * t
}

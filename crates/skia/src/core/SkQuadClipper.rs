// Copyright 2009 The Android Open Source Project. BSD-style license.
use crate::src::core::SkGeometry::{SkChopQuadAt, SkFindUnitQuadRoots};
use crate::src::core::SkPoint::SkPoint;
use crate::src::core::SkRect::{SkIRect, SkRect};
// Source: src/core/SkQuadClipper.h:24-34
pub struct SkQuadClipper {
    fClip: SkRect,
}
// Source: src/core/SkQuadClipper.h:40-40
// Source: src/core/SkQuadClipper.h:48-56
// This is the actual native second clipper's storage, not a rendering fallback.
// Its declared clip/append/next methods have no definitions in the pinned tree.
// Keep every originally indeterminate pointer/point/verb byte behind MaybeUninit;
// neither default construction nor layout queries inspect uninitialized values.
#[repr(C)]
pub struct SkQuadClipper2 {
    fCurrPoint: core::mem::MaybeUninit<*mut SkPoint>,
    fCurrVerb: core::mem::MaybeUninit<*mut crate::src::core::SkPath::Verb>,
    fPoints: [core::mem::MaybeUninit<SkPoint>; kMaxPoints],
    fVerbs: [core::mem::MaybeUninit<crate::src::core::SkPath::Verb>; kMaxVerbs],
}
// Source: src/core/SkQuadClipper.h:52-53
const kMaxVerbs: usize = 13;
const kMaxPoints: usize = 32;
// Source: src/core/SkQuadClipper.h:40-40
// Adapted implicit native default: retain indeterminate storage without reading it.
impl Default for SkQuadClipper2 {
    fn default() -> Self {
        Self {
            fCurrPoint: core::mem::MaybeUninit::uninit(),
            fCurrVerb: core::mem::MaybeUninit::uninit(),
            fPoints: [core::mem::MaybeUninit::uninit(); kMaxPoints],
            fVerbs: [core::mem::MaybeUninit::uninit(); kMaxVerbs],
        }
    }
}

impl SkQuadClipper {
    // Source: src/core/SkQuadClipper.h:26-26
    // Source: src/core/SkQuadClipper.cpp:14-16
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
    // Source: src/core/SkQuadClipper.h:28-28
    // Source: src/core/SkQuadClipper.cpp:18-21
    pub fn setClip(&mut self, clip: &SkIRect) {
        self.fClip = SkRect {
            fLeft: clip.fLeft as f32,
            fTop: clip.fTop as f32,
            fRight: clip.fRight as f32,
            fBottom: clip.fBottom as f32,
        };
    }
}
// Source: src/core/SkQuadClipper.cpp:25-42
fn chopMonoQuadAt(c0: f32, c1: f32, c2: f32, target: f32) -> Option<f32> {
    let a = c0 - c1 - c1 + c2;
    let b = 2.0 * (c1 - c0);
    let c = c0 - target;
    let mut roots = [0.0; 2];
    if SkFindUnitQuadRoots(a, b, c, &mut roots) != 0 {
        Some(roots[0])
    } else {
        None
    }
}
// Source: src/core/SkQuadClipper.cpp:44-46
fn chopMonoQuadAtY(pts: &[SkPoint; 3], y: f32) -> Option<f32> {
    chopMonoQuadAt(pts[0].fY, pts[1].fY, pts[2].fY, y)
}
impl SkQuadClipper {
    // Source: src/core/SkQuadClipper.h:30-30
    // Source: src/core/SkQuadClipper.cpp:54-118
    pub fn clipQuad(&self, src: &[SkPoint; 3], dst: &mut [SkPoint; 3]) -> bool {
        let reverse;
        if src[0].fY > src[2].fY {
            *dst = [src[2], src[1], src[0]];
            reverse = true;
        } else {
            *dst = *src;
            reverse = false;
        }
        let ctop = self.fClip.fTop;
        let cbot = self.fClip.fBottom;
        if dst[2].fY <= ctop || dst[0].fY >= cbot {
            return false;
        }
        let mut tmp = [dst[0]; 5];
        if dst[0].fY < ctop {
            if let Some(t) = chopMonoQuadAtY(dst, ctop) {
                SkChopQuadAt(dst, &mut tmp, t);
                dst[0] = tmp[2];
                dst[1] = tmp[3];
            } else {
                for pt in dst.iter_mut() {
                    if pt.fY < ctop {
                        pt.fY = ctop;
                    }
                }
            }
        }
        if dst[2].fY > cbot {
            if let Some(t) = chopMonoQuadAtY(dst, cbot) {
                SkChopQuadAt(dst, &mut tmp, t);
                dst[1] = tmp[1];
                dst[2] = tmp[2];
            } else {
                for pt in dst.iter_mut() {
                    if pt.fY > cbot {
                        pt.fY = cbot;
                    }
                }
            }
        }
        if reverse {
            dst.swap(0, 2);
        }
        true
    }
}

// Copyright 2018 Google LLC. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsTCurve.h and SkTQuad/SkTConic/SkTCubic wrappers.
use super::{
    SkIntersections::SkIntersections,
    SkPathOpsConic::SkDConic,
    SkPathOpsCubic::SkDCubic,
    SkPathOpsLine::SkDLine,
    SkPathOpsPoint::{SkDPoint, SkDVector},
    SkPathOpsQuad::SkDQuad,
    SkPathOpsRect::SkDRect,
};
use crate::src::core::SkArenaAlloc::SkArenaAlloc;
use core::any::Any;
pub trait SkTCurve: Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn point(&self, n: usize) -> &SkDPoint;
    fn point_mut(&mut self, n: usize) -> &mut SkDPoint;
    fn collapsed(&self) -> bool;
    fn controlsInside(&self) -> bool;
    fn dxdyAtT(&self, t: f64) -> SkDVector;
    fn hullIntersectsQuad(&self, q: &SkDQuad, linear: &mut bool) -> bool;
    fn hullIntersectsConic(&self, c: &SkDConic, linear: &mut bool) -> bool;
    fn hullIntersectsCubic(&self, c: &SkDCubic, linear: &mut bool) -> bool;
    fn hullIntersects(&self, c: &dyn SkTCurve, linear: &mut bool) -> bool;
    fn intersectRay(&self, i: &mut SkIntersections, l: &SkDLine) -> usize;
    fn IsConic(&self) -> bool;
    fn make(&self, heap: &mut SkArenaAlloc) -> *mut dyn SkTCurve;
    fn maxIntersections(&self) -> usize;
    fn otherPts(&self, odd: usize, p: &mut [*const SkDPoint; 4]);
    fn pointCount(&self) -> usize;
    fn pointLast(&self) -> usize;
    fn ptAtT(&self, t: f64) -> SkDPoint;
    fn setBounds(&self, r: &mut SkDRect);
    fn subDivide(&self, t1: f64, t2: f64, c: &mut dyn SkTCurve);
}
#[derive(Clone, Copy, Default)]
pub struct SkTQuad {
    pub fQuad: SkDQuad,
}
#[derive(Clone, Copy, Default)]
pub struct SkTConic {
    pub fConic: SkDConic,
}
#[derive(Clone, Copy, Default)]
pub struct SkTCubic {
    pub fCubic: SkDCubic,
}
macro_rules! curve_wrapper {
    ($wrapper:ident,$field:ident,$primitive:ident,$ray:ident,$hull:ident,$conic:expr,$count:expr) => {
        impl $wrapper {
            pub fn new(c: &$primitive) -> Self {
                Self { $field: *c }
            }
        }
        impl SkTCurve for $wrapper {
            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }
            fn point(&self, n: usize) -> &SkDPoint {
                &self.$field[n]
            }
            fn point_mut(&mut self, n: usize) -> &mut SkDPoint {
                &mut self.$field[n]
            }
            fn collapsed(&self) -> bool {
                self.$field.collapsed()
            }
            fn controlsInside(&self) -> bool {
                self.$field.controlsInside()
            }
            fn dxdyAtT(&self, t: f64) -> SkDVector {
                self.$field.dxdyAtT(t)
            }
            fn hullIntersectsQuad(&self, q: &SkDQuad, l: &mut bool) -> bool {
                q.$hull(&self.$field, l)
            }
            fn hullIntersectsConic(&self, q: &SkDConic, l: &mut bool) -> bool {
                q.$hull(&self.$field, l)
            }
            fn hullIntersectsCubic(&self, q: &SkDCubic, l: &mut bool) -> bool {
                q.$hull(&self.$field, l)
            }
            fn hullIntersects(&self, q: &dyn SkTCurve, l: &mut bool) -> bool {
                q.$hull(&self.$field, l)
            }
            fn intersectRay(&self, i: &mut SkIntersections, l: &SkDLine) -> usize {
                i.$ray(&self.$field, l)
            }
            fn IsConic(&self) -> bool {
                $conic
            }
            fn make(&self, heap: &mut SkArenaAlloc) -> *mut dyn SkTCurve {
                heap.make(Self::default()) as *mut dyn SkTCurve
            }
            fn maxIntersections(&self) -> usize {
                $primitive::kMaxIntersections
            }
            fn otherPts(&self, odd: usize, p: &mut [*const SkDPoint; 4]) {
                let pts = self.$field.otherPts(odd);
                for i in 0..pts.len() {
                    p[i] = pts[i];
                }
            }
            fn pointCount(&self) -> usize {
                $count
            }
            fn pointLast(&self) -> usize {
                $count - 1
            }
            fn ptAtT(&self, t: f64) -> SkDPoint {
                self.$field.ptAtT(t)
            }
            fn setBounds(&self, r: &mut SkDRect) {
                r.setBounds(&self.$field)
            }
            fn subDivide(&self, t1: f64, t2: f64, c: &mut dyn SkTCurve) {
                c.as_any_mut()
                    .downcast_mut::<Self>()
                    .expect("matching source curve owner")
                    .$field = self.$field.subDivide(t1, t2);
            }
        }
    };
}
curve_wrapper!(
    SkTQuad,
    fQuad,
    SkDQuad,
    intersectRayQuad,
    hullIntersectsQuad,
    false,
    3
);
curve_wrapper!(
    SkTConic,
    fConic,
    SkDConic,
    intersectRayConic,
    hullIntersectsConic,
    true,
    3
);
curve_wrapper!(
    SkTCubic,
    fCubic,
    SkDCubic,
    intersectRayCubic,
    hullIntersectsCubic,
    false,
    4
);

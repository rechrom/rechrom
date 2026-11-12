// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkPathOpsBounds.h. Lines are nonempty PathOps bounds.
use super::{SkPathOpsPoint::SkDPoint, SkPathOpsTypes::AlmostLessOrEqualUlps};
use crate::src::core::{SkPoint::SkPoint, SkRect::SkRect};
use core::ops::{Deref, DerefMut};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SkPathOpsBounds {
    pub base: SkRect,
}
impl Deref for SkPathOpsBounds {
    type Target = SkRect;
    fn deref(&self) -> &SkRect {
        &self.base
    }
}
impl DerefMut for SkPathOpsBounds {
    fn deref_mut(&mut self) -> &mut SkRect {
        &mut self.base
    }
}
impl SkPathOpsBounds {
    pub fn Intersects(a: &Self, b: &Self) -> bool {
        AlmostLessOrEqualUlps(a.fLeft, b.fRight)
            && AlmostLessOrEqualUlps(b.fLeft, a.fRight)
            && AlmostLessOrEqualUlps(a.fTop, b.fBottom)
            && AlmostLessOrEqualUlps(b.fTop, a.fBottom)
    }
    pub fn addLTRB(&mut self, left: f32, top: f32, right: f32, bottom: f32) {
        if left < self.fLeft {
            self.fLeft = left;
        }
        if top < self.fTop {
            self.fTop = top;
        }
        if right > self.fRight {
            self.fRight = right;
        }
        if bottom > self.fBottom {
            self.fBottom = bottom;
        }
    }
    pub fn addBounds(&mut self, b: &Self) {
        self.add(b);
    }
    pub fn add(&mut self, b: &Self) {
        self.addLTRB(b.fLeft, b.fTop, b.fRight, b.fBottom);
    }
    pub fn addPoint(&mut self, p: &SkPoint) {
        self.addLTRB(p.fX, p.fY, p.fX, p.fY);
    }
    pub fn addDPoint(&mut self, p: &SkDPoint) {
        if p.fX < (self.fLeft as f64) {
            self.fLeft = p.fX as f32;
        }
        if p.fY < (self.fTop as f64) {
            self.fTop = p.fY as f32;
        }
        if p.fX > (self.fRight as f64) {
            self.fRight = p.fX as f32;
        }
        if p.fY > (self.fBottom as f64) {
            self.fBottom = p.fY as f32;
        }
    }
    pub fn almostContains(&self, p: &SkPoint) -> bool {
        AlmostLessOrEqualUlps(self.fLeft, p.fX)
            && AlmostLessOrEqualUlps(p.fX, self.fRight)
            && AlmostLessOrEqualUlps(self.fTop, p.fY)
            && AlmostLessOrEqualUlps(p.fY, self.fBottom)
    }
    pub fn contains(&self, p: &SkPoint) -> bool {
        self.fLeft <= p.fX && self.fTop <= p.fY && self.fRight >= p.fX && self.fBottom >= p.fY
    }
}

// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkIntersections.h/.cpp. Sorted native intersection owner.
use super::{SkPathOpsPoint::SkDPoint, SkPathOpsTypes::*};
use core::{
    borrow::Borrow,
    ops::{Index, IndexMut},
};
#[derive(Clone)]
pub struct SkIntersections {
    pub(super) fPt: [SkDPoint; 13],
    pub(super) fPt2: [SkDPoint; 2],
    pub(super) fT: [[f64; 13]; 2],
    pub(super) fIsCoincident: [u16; 2],
    pub(super) fNearlySame: [bool; 2],
    pub(super) fUsed: usize,
    pub(super) fMax: usize,
    pub(super) fAllowNear: bool,
    pub(super) fSwap: bool,
}
impl Default for SkIntersections {
    fn default() -> Self {
        let mut i = Self {
            fPt: [SkDPoint::default(); 13],
            fPt2: [SkDPoint::default(); 2],
            fT: [[0.0; 13]; 2],
            fIsCoincident: [0; 2],
            fNearlySame: [false; 2],
            fUsed: 0,
            fMax: 0,
            fAllowNear: true,
            fSwap: false,
        };
        i.reset();
        i
    }
}
impl Index<usize> for SkIntersections {
    type Output = [f64; 13];
    fn index(&self, n: usize) -> &Self::Output {
        &self.fT[n]
    }
}
impl IndexMut<usize> for SkIntersections {
    fn index_mut(&mut self, n: usize) -> &mut Self::Output {
        &mut self.fT[n]
    }
}
impl SkIntersections {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn allowNear(&mut self, allow: bool) {
        self.fAllowNear = allow;
    }
    pub fn clearCoincidence(&mut self, index: usize) {
        let bit = 1 << index;
        self.fIsCoincident[0] &= !bit;
        self.fIsCoincident[1] &= !bit;
    }
    pub fn hasT(&self, t: f64) -> bool {
        self.fUsed > 0
            && if t == 0.0 {
                self.fT[0][0] == 0.0
            } else {
                self.fT[0][self.fUsed - 1] == 1.0
            }
    }
    pub fn hasOppT(&self, t: f64) -> bool {
        self.fUsed > 0 && (self.fT[1][0] == t || self.fT[1][self.fUsed - 1] == t)
    }
    pub fn insertSwap(&mut self, one: f64, two: f64, pt: &SkDPoint) -> i32 {
        if self.fSwap {
            self.insert(two, one, pt)
        } else {
            self.insert(one, two, pt)
        }
    }
    pub fn isCoincident(&self, index: usize) -> bool {
        self.fIsCoincident[0] & (1 << index) != 0
    }
    pub fn nearlySame(&self, index: usize) -> bool {
        self.fNearlySame[index]
    }
    pub fn pt(&self, index: usize) -> &SkDPoint {
        &self.fPt[index]
    }
    pub fn pt2(&self, index: usize) -> &SkDPoint {
        &self.fPt2[index]
    }
    pub fn reset(&mut self) {
        self.fAllowNear = true;
        self.fUsed = 0;
        self.fIsCoincident = [0; 2];
    }
    pub fn set(&mut self, swap: bool, index: usize, t: f64) {
        self.fT[swap as usize][index] = t;
    }
    pub fn setMax(&mut self, max: usize) {
        self.fMax = max;
    }
    pub fn swap(&mut self) {
        self.fSwap ^= true;
    }
    pub fn swapped(&self) -> bool {
        self.fSwap
    }
    pub fn used(&self) -> usize {
        self.fUsed
    }
    pub fn unBumpT(&mut self, index: usize) -> bool {
        self.fT[0][index] = self.fT[0][index] * (1.0 + BUMP_EPSILON * 2.0) - BUMP_EPSILON;
        if !between(0.0, self.fT[0][index], 1.0) {
            self.fUsed = 0;
            false
        } else {
            true
        }
    }
    pub fn closestTo(&self, start: f64, end: f64, test: &SkDPoint, closestDist: &mut f64) -> i32 {
        let mut closest = -1;
        *closestDist = f32::MAX as f64;
        for i in 0..self.fUsed {
            if !between(start, self.fT[0][i], end) {
                continue;
            }
            let dist = test.distanceSquared(&self.fPt[i]);
            if *closestDist > dist {
                *closestDist = dist;
                closest = i as i32;
            }
        }
        closest
    }
    pub fn flip(&mut self) {
        for i in 0..self.fUsed {
            self.fT[1][i] = 1.0 - self.fT[1][i];
        }
    }
    pub fn insert(&mut self, one: f64, two: f64, pt: impl Borrow<SkDPoint>) -> i32 {
        let pt = pt.borrow();
        if self.fIsCoincident[0] == 3 && between(self.fT[0][0], one, self.fT[0][1]) {
            return -1;
        }

        for i in 0..self.fUsed {
            let oldOne = self.fT[0][i];
            let oldTwo = self.fT[1][i];
            if one == oldOne && two == oldTwo {
                return -1;
            }
            if more_roughly_equal(oldOne, one) && more_roughly_equal(oldTwo, two) {
                if (!precisely_zero(one) || precisely_zero(oldOne))
                    && (!precisely_equal(one, 1.0) || precisely_equal(oldOne, 1.0))
                    && (!precisely_zero(two) || precisely_zero(oldTwo))
                    && (!precisely_equal(two, 1.0) || precisely_equal(oldTwo, 1.0))
                {
                    return -1;
                }

                self.fPt.copy_within(i + 1..self.fUsed, i);
                self.fT[0].copy_within(i + 1..self.fUsed, i);
                self.fT[1].copy_within(i + 1..self.fUsed, i);
                let clear = !((1_u16 << i) - 1);
                for bits in &mut self.fIsCoincident {
                    *bits -= (*bits >> 1) & clear;
                }
                self.fUsed -= 1;
                break;
            }
        }
        let mut index = 0;
        while index < self.fUsed {
            if self.fT[0][index] > one {
                break;
            }
            index += 1;
        }
        if self.fUsed >= self.fMax {
            self.fUsed = 0;
            return 0;
        }
        if self.fUsed > index {
            self.fPt.copy_within(index..self.fUsed, index + 1);
            self.fT[0].copy_within(index..self.fUsed, index + 1);
            self.fT[1].copy_within(index..self.fUsed, index + 1);
            let clear = !((1_u16 << index) - 1);
            for bits in &mut self.fIsCoincident {
                *bits += *bits & clear;
            }
        }
        self.fPt[index] = *pt;
        if one < 0.0 || one > 1.0 {
            return -1;
        }
        if two < 0.0 || two > 1.0 {
            return -1;
        }
        self.fT[0][index] = one;
        self.fT[1][index] = two;
        self.fUsed += 1;

        index as i32
    }
    pub fn insertNear(
        &mut self,
        one: f64,
        two: f64,
        pt1: impl Borrow<SkDPoint>,
        pt2: impl Borrow<SkDPoint>,
    ) {
        let pt1 = pt1.borrow();
        let pt2 = pt2.borrow();

        let idx = (one != 0.0) as usize;
        self.fNearlySame[idx] = true;
        self.insert(one, two, pt1);
        self.fPt2[idx] = *pt2;
    }
    pub fn insertCoincident(&mut self, one: f64, two: f64, pt: impl Borrow<SkDPoint>) -> i32 {
        let pt = pt.borrow();
        let i = self.insertSwap(one, two, pt);
        if i >= 0 {
            self.setCoincident(i as usize);
        }
        i
    }
    pub fn setCoincident(&mut self, index: usize) {
        let bit = 1 << index;
        self.fIsCoincident[0] |= bit;
        self.fIsCoincident[1] |= bit;
    }
    pub fn merge(&mut self, a: &Self, ai: usize, b: &Self, bi: usize) {
        self.reset();
        self.fT[0][0] = a.fT[0][ai];
        self.fT[1][0] = b.fT[0][bi];
        self.fPt[0] = a.fPt[ai];
        self.fPt2[0] = b.fPt[bi];
        self.fUsed = 1;
    }
    pub fn mostOutside(&self, start: f64, end: f64, origin: &SkDPoint) -> i32 {
        let mut result = None;
        for i in 0..self.fUsed {
            if !between(start, self.fT[0][i], end) {
                continue;
            }
            let Some(best) = result else {
                result = Some(i);
                continue;
            };
            let best = self.fPt[best] - *origin;
            let test = self.fPt[i] - *origin;
            if test.crossCheck(&best) < 0.0 {
                result = Some(i);
            }
        }
        result.map_or(-1, |i| i as i32)
    }
    pub fn removeOne(&mut self, index: usize) {
        self.fUsed -= 1;
        if self.fUsed <= index {
            return;
        }
        self.fPt.copy_within(index + 1..=self.fUsed, index);
        self.fT[0].copy_within(index + 1..=self.fUsed, index);
        self.fT[1].copy_within(index + 1..=self.fUsed, index);
        let coBit = self.fIsCoincident[0] & (1 << index);
        self.fIsCoincident[0] -= ((self.fIsCoincident[0] >> 1) & !((1 << index) - 1)) + coBit;

        self.fIsCoincident[1] -= ((self.fIsCoincident[1] >> 1) & !((1 << index) - 1)) + coBit;
    }
}

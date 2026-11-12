// Copyright 2012 Google Inc. BSD-style license in LICENSE.
// Source: src/pathops/SkOpCubicHull.cpp.
use super::{
    SkPathOpsCubic::{other_two, SkDCubic},
    SkPathOpsTypes::*,
};
fn rotate(cubic: &SkDCubic, zero: usize, index: usize, rot: &mut SkDCubic) -> bool {
    let dy = cubic[index].fY - cubic[zero].fY;
    let dx = cubic[index].fX - cubic[zero].fX;
    if approximately_zero(dy) {
        if approximately_zero(dx) {
            return false;
        }
        *rot = *cubic;
        if dy != 0.0 {
            rot[index].fY = cubic[zero].fY;
            let mask = other_two(index, zero);
            let side1 = index ^ mask;
            let side2 = zero ^ mask;
            if approximately_equal(cubic[side1].fY, cubic[zero].fY) {
                rot[side1].fY = cubic[zero].fY;
            }
            if approximately_equal(cubic[side2].fY, cubic[zero].fY) {
                rot[side2].fY = cubic[zero].fY;
            }
        }
        return true;
    }
    for i in 0..4 {
        rot[i].fX = cubic[i].fX * dx + cubic[i].fY * dy;
        rot[i].fY = cubic[i].fY * dx - cubic[i].fX * dy;
    }
    true
}
fn side(x: f64) -> i32 {
    (x > 0.0) as i32 + (x >= 0.0) as i32
}
impl SkDCubic {
    pub fn convexHull(&self, order: &mut [usize; 4]) -> usize {
        let mut yMin = 0;
        for i in 1..4 {
            if self[yMin].fY > self[i].fY
                || (self[yMin].fY == self[i].fY && self[yMin].fX > self[i].fX)
            {
                yMin = i;
            }
        }
        order[0] = yMin;
        let mut midX = None;
        let mut backup = None;
        for _pass in 0..2 {
            for i in 0..4 {
                if i == yMin {
                    continue;
                }
                let mask = other_two(yMin, i);
                let side1 = yMin ^ mask;
                let side2 = i ^ mask;
                let mut rot = Self::default();
                if !rotate(self, yMin, i, &mut rot) {
                    order[1] = side1;
                    order[2] = side2;
                    return 3;
                }
                let sides = side(rot[side1].fY - rot[yMin].fY) ^ side(rot[side2].fY - rot[yMin].fY);
                if sides == 2 {
                    if midX.is_some() {
                        order[0] = 0;
                        order[1] = 3;
                        if self[1] == self[0] || self[1] == self[3] {
                            order[2] = 2;
                            return 3;
                        }
                        if self[2] == self[0] || self[2] == self[3] {
                            order[2] = 1;
                            return 3;
                        }
                        let small1 = self[1]
                            .distanceSquared(&self[0])
                            .min(self[1].distanceSquared(&self[3]));
                        let small2 = self[2]
                            .distanceSquared(&self[0])
                            .min(self[2].distanceSquared(&self[3]));
                        if approximately_zero(small1.min(small2)) {
                            order[2] = if small1 < small2 { 2 } else { 1 };
                            return 3;
                        }
                    }
                    midX = Some(i);
                } else if sides == 0 {
                    backup = Some(i);
                }
            }
            if midX.is_some() {
                break;
            }
            let Some(b) = backup else {
                break;
            };
            yMin = b;
            backup = None;
        }
        let midX = midX.unwrap_or(yMin ^ 3);
        let mask = other_two(yMin, midX);
        let least = yMin ^ mask;
        let most = midX ^ mask;
        order[0] = yMin;
        order[1] = least;
        let mut midPath = Self::default();
        if !rotate(self, least, most, &mut midPath) {
            order[2] = midX;
            return 3;
        }
        let sides =
            side(midPath[yMin].fY - midPath[least].fY) ^ side(midPath[midX].fY - midPath[least].fY);
        if sides != 2 {
            order[2] = most;
            return 3;
        }
        order[2] = midX;
        order[3] = most;
        4
    }
}

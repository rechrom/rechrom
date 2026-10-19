// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/simple_enclosed_region.h:1-124
// Source: cc/base/simple_enclosed_region.cc:1-151
use super::region::Region;
use crate::adapter::{
    geometry::{Point, Rect},
    region_backend::SkRegionBackend,
};

// Source: cc/base/simple_enclosed_region.h:19-31,84-86
#[derive(Clone, Copy, Debug, Default)]
pub struct SimpleEnclosedRegion {
    rect_: Rect,
}
impl SimpleEnclosedRegion {
    // Source: cc/base/simple_enclosed_region.h:27
    pub fn new(rect: Rect) -> Self {
        Self { rect_: rect }
    }
    // Source: cc/base/simple_enclosed_region.h:28
    pub fn FromCoordinates(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            rect_: Rect::new(x, y, w, h),
        }
    }
    // Source: cc/base/simple_enclosed_region.h:29
    pub fn FromSize(w: i32, h: i32) -> Self {
        Self {
            rect_: Rect::new(0, 0, w, h),
        }
    }
    // Source: cc/base/simple_enclosed_region.h:30
    // Source: cc/base/simple_enclosed_region.cc:22-25
    pub fn FromRegion<B: SkRegionBackend>(region: &Region<B>) -> Self {
        let mut result = Self::default();
        for rect in region {
            result.Union(rect);
        }
        result
    }
    // Source: cc/base/simple_enclosed_region.h:33-36
    pub fn AssignRect(&mut self, rect: Rect) -> &Self {
        self.rect_ = rect;
        self
    }
    // Source: cc/base/simple_enclosed_region.h:37-40
    pub fn Assign(&mut self, region: &Self) -> &Self {
        self.rect_ = region.rect_;
        self
    }
    // Source: cc/base/simple_enclosed_region.h:42
    pub fn IsEmpty(&self) -> bool {
        self.rect_.IsEmpty()
    }
    // Source: cc/base/simple_enclosed_region.h:43
    pub fn Clear(&mut self) {
        self.rect_ = Rect::default();
    }
    // Source: cc/base/simple_enclosed_region.h:44
    pub fn GetRegionComplexity(&self) -> usize {
        if self.rect_.IsEmpty() {
            0
        } else {
            1
        }
    }
    // Source: cc/base/simple_enclosed_region.h:46
    pub fn ContainsPoint(&self, point: Point) -> bool {
        self.rect_.ContainsPoint(point)
    }
    // Source: cc/base/simple_enclosed_region.h:47
    pub fn ContainsRect(&self, rect: Rect) -> bool {
        self.rect_.Contains(rect)
    }
    // Source: cc/base/simple_enclosed_region.h:48-50
    pub fn Contains(&self, region: &Self) -> bool {
        self.rect_.Contains(region.rect_)
    }
    // Source: cc/base/simple_enclosed_region.h:52-54
    pub fn IntersectsRect(&self, rect: Rect) -> bool {
        self.rect_.Intersects(rect)
    }
    // Source: cc/base/simple_enclosed_region.h:55-57
    pub fn Intersects(&self, region: &Self) -> bool {
        self.rect_.Intersects(region.rect_)
    }
    // Source: cc/base/simple_enclosed_region.h:60-62
    pub fn SubtractRegion(&mut self, sub_region: &Self) {
        self.Subtract(sub_region.rect_);
    }
    // Source: cc/base/simple_enclosed_region.h:64-66
    pub fn UnionRegion(&mut self, new_region: &Self) {
        self.Union(new_region.rect_);
    }
    // Source: cc/base/simple_enclosed_region.h:67
    pub fn Intersect(&mut self, in_rect: Rect) {
        self.rect_.Intersect(in_rect);
    }
    // Source: cc/base/simple_enclosed_region.h:68-70
    pub fn IntersectRegion(&mut self, in_region: &Self) {
        self.Intersect(in_region.rect_);
    }
    // Source: cc/base/simple_enclosed_region.h:72-75
    pub fn Equals(&self, other: &Self) -> bool {
        let both_empty = self.rect_.IsEmpty() && other.rect_.IsEmpty();
        both_empty || self.rect_ == other.rect_
    }
    // Source: cc/base/simple_enclosed_region.h:77
    pub fn bounds(&self) -> Rect {
        self.rect_
    }
    // Source: cc/base/simple_enclosed_region.h:82
    pub fn ToString(&self) -> String {
        self.rect_.ToString()
    }

    // Source: cc/base/simple_enclosed_region.h:59
    // Source: cc/base/simple_enclosed_region.cc:29-73
    pub fn Subtract(&mut self, sub_rect: Rect) {
        if !self.rect_.Intersects(sub_rect) {
            return;
        }
        if sub_rect.Contains(self.rect_) {
            self.rect_ = Rect::default();
            return;
        }
        let left = self.rect_.x;
        let right = self.rect_.right();
        let top = self.rect_.y;
        let bottom = self.rect_.bottom();
        let delta_left = sub_rect.x - left;
        let delta_right = right - sub_rect.right();
        let delta_top = sub_rect.y - top;
        let delta_bottom = bottom - sub_rect.bottom();
        let mut horizontal_top = top;
        let mut horizontal_bottom = bottom;
        if delta_top > delta_bottom {
            horizontal_bottom = sub_rect.y;
        } else {
            horizontal_top = sub_rect.bottom();
        }
        let mut vertical_left = left;
        let mut vertical_right = right;
        if delta_left > delta_right {
            vertical_right = sub_rect.x;
        } else {
            vertical_left = sub_rect.right();
        }
        self.rect_ = Rect::new(
            left,
            horizontal_top,
            right - left,
            horizontal_bottom - horizontal_top,
        );
        let vertical_rect = Rect::new(
            vertical_left,
            top,
            vertical_right - vertical_left,
            bottom - top,
        );
        if RectIsLargerArea(vertical_rect, self.rect_) {
            self.rect_ = vertical_rect;
        }
    }

    // Source: cc/base/simple_enclosed_region.h:63
    // Source: cc/base/simple_enclosed_region.cc:75-144
    pub fn Union(&mut self, new_rect: Rect) {
        if new_rect.IsEmpty() {
            return;
        }
        if self.rect_.Contains(new_rect) {
            return;
        }
        if new_rect.Contains(self.rect_) {
            self.rect_ = new_rect;
            return;
        }
        let mut left = self.rect_.x;
        let mut top = self.rect_.y;
        let mut right = self.rect_.right();
        let mut bottom = self.rect_.bottom();
        let mut new_left = new_rect.x;
        let mut new_top = new_rect.y;
        let mut new_right = new_rect.right();
        let mut new_bottom = new_rect.bottom();
        if new_top <= top && new_bottom >= bottom {
            if new_left < left && new_right >= left {
                left = new_left;
            }
            if new_right > right && new_left <= right {
                right = new_right;
            }
        } else if new_left <= left && new_right >= right {
            if new_top < top && new_bottom >= top {
                top = new_top;
            }
            if new_bottom > bottom && new_top <= bottom {
                bottom = new_bottom;
            }
        } else if top <= new_top && bottom >= new_bottom {
            if left < new_left && right >= new_left {
                new_left = left;
            }
            if right > new_right && left <= new_right {
                new_right = right;
            }
        } else if left <= new_left && right >= new_right {
            if top < new_top && bottom >= new_top {
                new_top = top;
            }
            if bottom > new_bottom && top <= new_bottom {
                new_bottom = bottom;
            }
        }
        self.rect_ = Rect::new(left, top, right - left, bottom - top);
        let rect_area = self.rect_.width as i64 * self.rect_.height as i64;
        let adjusted_new_rect = Rect::new(
            new_left,
            new_top,
            new_right - new_left,
            new_bottom - new_top,
        );
        let adjust_new_rect_area = adjusted_new_rect.width as i64 * adjusted_new_rect.height as i64;
        let mut overlap = self.rect_;
        overlap.Intersect(adjusted_new_rect);
        let overlap_area = overlap.width as i64 * overlap.height as i64;
        if adjust_new_rect_area * 2 > rect_area + overlap_area {
            self.rect_ = adjusted_new_rect;
        }
    }
    // Source: cc/base/simple_enclosed_region.h:79-80
    // Source: cc/base/simple_enclosed_region.cc:146-149
    pub fn GetRect(&self, i: usize) -> Rect {
        debug_assert!(i < self.GetRegionComplexity());
        self.rect_
    }
}

// Source: cc/base/simple_enclosed_region.cc:16-20
fn RectIsLargerArea(a: Rect, b: Rect) -> bool {
    let a_area = a.width as i64 * a.height as i64;
    let b_area = b.width as i64 * b.height as i64;
    a_area > b_area
}
impl PartialEq for SimpleEnclosedRegion {
    // Source: cc/base/simple_enclosed_region.h:88-91
    fn eq(&self, other: &Self) -> bool {
        self.Equals(other)
    }
    // Source: cc/base/simple_enclosed_region.h:93-96
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Eq for SimpleEnclosedRegion {}
// Source: cc/base/simple_enclosed_region.h:98-104
pub fn SubtractSimpleEnclosedRegions(
    a: &SimpleEnclosedRegion,
    b: &SimpleEnclosedRegion,
) -> SimpleEnclosedRegion {
    let mut result = *a;
    result.SubtractRegion(b);
    result
}
// Source: cc/base/simple_enclosed_region.h:106-112
pub fn IntersectSimpleEnclosedRegions(
    a: &SimpleEnclosedRegion,
    b: &SimpleEnclosedRegion,
) -> SimpleEnclosedRegion {
    let mut result = *a;
    result.IntersectRegion(b);
    result
}
// Source: cc/base/simple_enclosed_region.h:114-120
pub fn UnionSimpleEnclosedRegions(
    a: &SimpleEnclosedRegion,
    b: &SimpleEnclosedRegion,
) -> SimpleEnclosedRegion {
    let mut result = *a;
    result.UnionRegion(b);
    result
}

// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/region.h:1-179
// Source: cc/base/region.cc:1-176
use super::simple_enclosed_region::SimpleEnclosedRegion;
use crate::adapter::{
    geometry::{Point, Rect, TracedValue, Vector2d},
    region_backend::{SkIRect, SkRegionBackend, SkRegionOp},
};

// Source: cc/base/region.h:33-40,118-120
// Source: cc/base/region.cc:19-33
#[derive(Clone, Default)]
pub struct Region<B: SkRegionBackend> {
    skregion_: B,
}
impl<B: SkRegionBackend> Region<B> {
    // Source: cc/base/region.cc:21
    pub fn FromSkRegion(region: &B) -> Self {
        Self {
            skregion_: region.clone(),
        }
    }
    // Source: cc/base/region.cc:25-27
    pub fn MoveFrom(other: &mut Self) -> Self {
        let mut result = Self::default();
        std::mem::swap(&mut result.skregion_, &mut other.skregion_);
        result
    }
    // Source: cc/base/region.cc:29-31
    pub fn new(rect: Rect) -> Self {
        Self {
            skregion_: B::from_rect(SkIRect::FromRect(rect)),
        }
    }
    // Source: cc/base/region.cc:35-38
    pub fn AssignRect(&mut self, rect: Rect) -> &Self {
        self.skregion_ = B::from_rect(SkIRect::FromRect(rect));
        self
    }
    // Source: cc/base/region.cc:40-43
    pub fn Assign(&mut self, region: &Self) -> &Self {
        self.skregion_ = region.skregion_.clone();
        self
    }
    // Source: cc/base/region.cc:45-48
    pub fn AssignMove(&mut self, other: &mut Self) -> &mut Self {
        std::mem::swap(&mut self.skregion_, &mut other.skregion_);
        self
    }
    // Source: cc/base/region.cc:50-53
    pub fn Offset(&mut self, offset: Vector2d) -> &Self {
        self.skregion_.translate(offset.x, offset.y);
        self
    }
    // Source: cc/base/region.h:47-50
    // Source: cc/base/region.cc:55-59
    pub fn Empty() -> &'static Self {
        // The backend provides the immutable NoDestructor-equivalent singleton;
        // this preserves one empty Region per backend without a global registry.
        B::shared_empty_region()
    }
    // Source: cc/base/region.cc:61-63
    pub fn Swap(&mut self, region: &mut Self) {
        std::mem::swap(&mut region.skregion_, &mut self.skregion_);
    }
    // Source: cc/base/region.cc:65-67
    pub fn Clear(&mut self) {
        self.skregion_.setEmpty();
    }
    // Source: cc/base/region.cc:69-71
    pub fn IsEmpty(&self) -> bool {
        self.skregion_.isEmpty()
    }
    // Source: cc/base/region.cc:73-75
    pub fn GetRegionComplexity(&self) -> i32 {
        self.skregion_.computeRegionComplexity()
    }
    // Source: cc/base/region.cc:77-79
    pub fn GetBoundaryPath(&self, path: &mut B::BoundaryPath) {
        *path = self.skregion_.getBoundaryPath();
    }
    // Source: cc/base/region.h:58-60
    pub fn Contains<O: RegionContains<B> + ?Sized>(&self, operand: &O) -> bool {
        operand.ContainsIn(self)
    }
    // Source: cc/base/region.h:62-63
    pub fn Intersects<O: RegionIntersects<B> + ?Sized>(&self, operand: &O) -> bool {
        operand.IntersectsWith(self)
    }
    // Source: cc/base/region.h:65-67
    pub fn Subtract<O: RegionSubtract<B> + ?Sized>(&mut self, operand: &O) {
        operand.SubtractFrom(self);
    }
    // Source: cc/base/region.h:68-70
    pub fn Union<O: RegionUnion<B> + ?Sized>(&mut self, operand: &O) {
        operand.UnionInto(self);
    }
    // Source: cc/base/region.h:71-72
    pub fn Intersect<O: RegionIntersect<B> + ?Sized>(&mut self, operand: &O) {
        operand.IntersectWith(self);
    }
    // Source: cc/base/region.h:74-76
    pub fn Equals(&self, other: &Self) -> bool {
        self.skregion_ == other.skregion_
    }
    // Source: cc/base/region.h:78-80
    pub fn bounds(&self) -> Rect {
        self.skregion_.getBounds().ToRect()
    }
    // Source: cc/base/region.cc:146-157
    pub fn ToString(&self) -> String {
        if self.IsEmpty() {
            return Rect::default().ToString();
        }
        let mut result = String::new();
        for rect in self {
            if !result.is_empty() {
                result += " | ";
            }
            result += &rect.ToString();
        }
        result
    }
    // Source: cc/base/region.cc:159-166
    pub fn AsValueInto(&self, result: &mut impl TracedValue) {
        for rect in self {
            result.AppendInteger(rect.x);
            result.AppendInteger(rect.y);
            result.AppendInteger(rect.width);
            result.AppendInteger(rect.height);
        }
    }
    // Source: cc/base/region.h:115
    // Source: cc/base/region.cc:168-170
    pub fn begin(&self) -> RegionIterator<'_, B> {
        RegionIterator::new(self)
    }
    // Source: cc/base/region.h:116
    pub fn end(&self) -> RegionIterator<'_, B> {
        RegionIterator::default()
    }
}

// Rust's trait-based overload dispatch keeps each C++ overload's own body.
pub trait RegionContains<B: SkRegionBackend> {
    fn ContainsIn(&self, region: &Region<B>) -> bool;
}
impl<B: SkRegionBackend> RegionContains<B> for Point {
    // Source: cc/base/region.cc:81-83
    fn ContainsIn(&self, region: &Region<B>) -> bool {
        region.skregion_.contains_point(self.x(), self.y())
    }
}
impl<B: SkRegionBackend> RegionContains<B> for Rect {
    // Source: cc/base/region.cc:85-89
    fn ContainsIn(&self, region: &Region<B>) -> bool {
        if self.IsEmpty() {
            return true;
        }
        region.skregion_.contains_rect(SkIRect::FromRect(*self))
    }
}
impl<B: SkRegionBackend> RegionContains<B> for Region<B> {
    // Source: cc/base/region.cc:91-95
    fn ContainsIn(&self, region: &Region<B>) -> bool {
        if self.IsEmpty() {
            return true;
        }
        region.skregion_.contains_region(&self.skregion_)
    }
}
pub trait RegionIntersects<B: SkRegionBackend> {
    fn IntersectsWith(&self, region: &Region<B>) -> bool;
}
impl<B: SkRegionBackend> RegionIntersects<B> for Rect {
    // Source: cc/base/region.cc:97-99
    fn IntersectsWith(&self, region: &Region<B>) -> bool {
        region.skregion_.intersects_rect(SkIRect::FromRect(*self))
    }
}
impl<B: SkRegionBackend> RegionIntersects<B> for Region<B> {
    // Source: cc/base/region.cc:101-103
    fn IntersectsWith(&self, region: &Region<B>) -> bool {
        region.skregion_.intersects_region(&self.skregion_)
    }
}
pub trait RegionSubtract<B: SkRegionBackend> {
    fn SubtractFrom(&self, region: &mut Region<B>);
}
impl<B: SkRegionBackend> RegionSubtract<B> for Rect {
    // Source: cc/base/region.cc:105-107
    fn SubtractFrom(&self, region: &mut Region<B>) {
        region
            .skregion_
            .op_rect(SkIRect::FromRect(*self), SkRegionOp::Difference);
    }
}
impl<B: SkRegionBackend> RegionSubtract<B> for Region<B> {
    // Source: cc/base/region.cc:109-111
    fn SubtractFrom(&self, region: &mut Region<B>) {
        region
            .skregion_
            .op_region(&self.skregion_, SkRegionOp::Difference);
    }
}
impl<B: SkRegionBackend> RegionSubtract<B> for SimpleEnclosedRegion {
    // Source: cc/base/region.cc:113-118
    fn SubtractFrom(&self, region: &mut Region<B>) {
        for i in 0..self.GetRegionComplexity() {
            region
                .skregion_
                .op_rect(SkIRect::FromRect(self.GetRect(i)), SkRegionOp::Difference);
        }
    }
}
pub trait RegionUnion<B: SkRegionBackend> {
    fn UnionInto(&self, region: &mut Region<B>);
}
impl<B: SkRegionBackend> RegionUnion<B> for Rect {
    // Source: cc/base/region.cc:120-122
    fn UnionInto(&self, region: &mut Region<B>) {
        region
            .skregion_
            .op_rect(SkIRect::FromRect(*self), SkRegionOp::Union);
    }
}
impl<B: SkRegionBackend> RegionUnion<B> for Region<B> {
    // Source: cc/base/region.cc:124-126
    fn UnionInto(&self, region: &mut Region<B>) {
        region
            .skregion_
            .op_region(&self.skregion_, SkRegionOp::Union);
    }
}
impl<B: SkRegionBackend> RegionUnion<B> for [SkIRect] {
    // Source: cc/base/region.cc:128-136
    fn UnionInto(&self, region: &mut Region<B>) {
        if self.is_empty() {
            return;
        }
        let mut batch_region = B::default();
        batch_region.setRects(self);
        region.skregion_.op_region(&batch_region, SkRegionOp::Union);
    }
}
pub trait RegionIntersect<B: SkRegionBackend> {
    fn IntersectWith(&self, region: &mut Region<B>);
}
impl<B: SkRegionBackend> RegionIntersect<B> for Rect {
    // Source: cc/base/region.cc:138-140
    fn IntersectWith(&self, region: &mut Region<B>) {
        region
            .skregion_
            .op_rect(SkIRect::FromRect(*self), SkRegionOp::Intersect);
    }
}
impl<B: SkRegionBackend> RegionIntersect<B> for Region<B> {
    // Source: cc/base/region.cc:142-144
    fn IntersectWith(&self, region: &mut Region<B>) {
        region
            .skregion_
            .op_region(&self.skregion_, SkRegionOp::Intersect);
    }
}

// Source: cc/base/region.h:85-113
pub struct RegionIterator<'a, B: SkRegionBackend> {
    iterator_: Option<B::RectIterator<'a>>,
    current_: Option<Rect>,
}
impl<B: SkRegionBackend> Default for RegionIterator<'_, B> {
    // Source: cc/base/region.h:90-91
    fn default() -> Self {
        Self {
            iterator_: None,
            current_: None,
        }
    }
}
impl<'a, B: SkRegionBackend> RegionIterator<'a, B> {
    // Source: cc/base/region.cc:172-174
    fn new(region: &'a Region<B>) -> Self {
        let mut iterator = region.skregion_.rects();
        let current = iterator.next().map(SkIRect::ToRect);
        Self {
            iterator_: Some(iterator),
            current_: current,
        }
    }
    // Source: cc/base/region.h:95-98
    pub fn Advance(&mut self) -> &mut Self {
        self.current_ = self
            .iterator_
            .as_mut()
            .and_then(|iterator| iterator.next())
            .map(SkIRect::ToRect);
        self
    }
}
impl<B: SkRegionBackend> std::ops::Deref for RegionIterator<'_, B> {
    type Target = Rect;
    // Source: cc/base/region.h:93
    fn deref(&self) -> &Rect {
        self.current_.as_ref().expect("region iterator is at end")
    }
}
impl<B: SkRegionBackend> PartialEq for RegionIterator<'_, B> {
    // Source: cc/base/region.h:100-104
    fn eq(&self, other: &Self) -> bool {
        debug_assert!(other.current_.is_none());
        self.current_.is_none()
    }
    // Source: cc/base/region.h:106
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl<B: SkRegionBackend> std::iter::Iterator for RegionIterator<'_, B> {
    type Item = Rect;
    // Source: cc/base/region.h:93-98
    fn next(&mut self) -> Option<Rect> {
        let current = self.current_?;
        self.Advance();
        Some(current)
    }
}
impl<'a, B: SkRegionBackend> IntoIterator for &'a Region<B> {
    type Item = Rect;
    type IntoIter = RegionIterator<'a, B>;
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}
impl<B: SkRegionBackend> PartialEq for Region<B> {
    // Source: cc/base/region.h:122-124
    fn eq(&self, other: &Self) -> bool {
        self.Equals(other)
    }
    // Source: cc/base/region.h:126-128
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl<B: SkRegionBackend> std::ops::AddAssign<Vector2d> for Region<B> {
    // Source: cc/base/region.cc:50-53
    fn add_assign(&mut self, offset: Vector2d) {
        self.Offset(offset);
    }
}
impl<B: SkRegionBackend> std::ops::Add<Vector2d> for &Region<B> {
    type Output = Region<B>;
    // Source: cc/base/region.h:130-134
    fn add(self, offset: Vector2d) -> Self::Output {
        let mut result = self.clone();
        result += offset;
        result
    }
}
// Source: cc/base/region.h:136-146
pub fn SubtractRegions<B: SkRegionBackend, O: RegionSubtract<B> + ?Sized>(
    a: &Region<B>,
    b: &O,
) -> Region<B> {
    let mut result = a.clone();
    result.Subtract(b);
    result
}
// Source: cc/base/region.h:148-158
pub fn IntersectRegions<B: SkRegionBackend, O: RegionIntersect<B> + ?Sized>(
    a: &Region<B>,
    b: &O,
) -> Region<B> {
    let mut result = a.clone();
    result.Intersect(b);
    result
}
// Source: cc/base/region.h:160-170
pub fn UnionRegions<B: SkRegionBackend, O: RegionUnion<B> + ?Sized>(
    a: &Region<B>,
    b: &O,
) -> Region<B> {
    let mut result = a.clone();
    result.Union(b);
    result
}
// Source: cc/base/region.h:172-175
// PrintTo is a declaration whose definition belongs to //cc:test_support.
// The production crate exposes ToString, as required by the source comment.

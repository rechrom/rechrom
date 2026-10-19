//! Typed SkRegion service boundary. The owning Skia implementation supplies
//! canonical region operations and allocation; compositor never substitutes
//! a rectangle-list union/difference algorithm for those operations.
use super::geometry::Rect;
use crate::cc::base::region::Region;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkIRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
impl SkIRect {
    pub fn FromRect(rect: Rect) -> Self {
        Self {
            left: rect.x,
            top: rect.y,
            right: rect.right(),
            bottom: rect.bottom(),
        }
    }
    pub fn ToRect(self) -> Rect {
        Rect::FromBounds(self.left, self.top, self.right, self.bottom)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkRegionOp {
    Difference,
    Intersect,
    Union,
    XOR,
}

pub trait SkRegionBackend: Clone + Default + PartialEq + 'static {
    type BoundaryPath;
    type RectIterator<'a>: Iterator<Item = SkIRect>
    where
        Self: 'a;
    fn from_rect(rect: SkIRect) -> Self;
    fn shared_empty_region() -> &'static Region<Self>;
    fn translate(&mut self, x: i32, y: i32);
    fn setEmpty(&mut self);
    fn isEmpty(&self) -> bool;
    fn computeRegionComplexity(&self) -> i32;
    fn getBoundaryPath(&self) -> Self::BoundaryPath;
    fn contains_point(&self, x: i32, y: i32) -> bool;
    fn contains_rect(&self, rect: SkIRect) -> bool;
    fn contains_region(&self, region: &Self) -> bool;
    fn intersects_rect(&self, rect: SkIRect) -> bool;
    fn intersects_region(&self, region: &Self) -> bool;
    fn op_rect(&mut self, rect: SkIRect, op: SkRegionOp);
    fn op_region(&mut self, region: &Self, op: SkRegionOp);
    fn setRects(&mut self, rects: &[SkIRect]);
    fn getBounds(&self) -> SkIRect;
    fn rects(&self) -> Self::RectIterator<'_>;
}

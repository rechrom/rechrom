//! gfx range/rectangle dependencies consumed by translated scroll snapping.
//! Adapter code, excluded from cc source coverage; no selection policy lives here.
use super::input_geometry::{PointF, RectF};
fn min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RangeF {
    start_: f32,
    end_: f32,
}
impl RangeF {
    pub const fn new(start: f32, end: f32) -> Self {
        Self {
            start_: start,
            end_: end,
        }
    }
    pub const fn At(position: f32) -> Self {
        Self::new(position, position)
    }
    pub const fn InvalidRange() -> Self {
        Self::At(f32::MAX)
    }
    pub fn IsValid(self) -> bool {
        self != Self::InvalidRange()
    }
    pub const fn start(self) -> f32 {
        self.start_
    }
    pub const fn end(self) -> f32 {
        self.end_
    }
    pub fn set_start(&mut self, value: f32) {
        self.start_ = value;
    }
    pub fn set_end(&mut self, value: f32) {
        self.end_ = value;
    }
    pub fn GetMin(self) -> f32 {
        if self.start_ < self.end_ {
            self.start_
        } else {
            self.end_
        }
    }
    pub fn GetMax(self) -> f32 {
        if self.start_ > self.end_ {
            self.start_
        } else {
            self.end_
        }
    }
    pub fn length(self) -> f32 {
        self.GetMax() - self.GetMin()
    }
    pub fn is_reversed(self) -> bool {
        self.start_ > self.end_
    }
    pub fn is_empty(self) -> bool {
        self.start_ == self.end_
    }
    pub fn IsBoundedBy(self, range: Self) -> bool {
        self.IsValid()
            && range.IsValid()
            && self.GetMin() >= range.GetMin()
            && self.GetMax() <= range.GetMax()
    }
    pub fn Contains(self, range: Self) -> bool {
        range.IsBoundedBy(self)
            && (range.GetMax() != self.GetMax() || range.is_empty() == self.is_empty())
    }
    pub fn Intersect(self, range: Self) -> Self {
        let intersection_min = max(self.GetMin(), range.GetMin());
        let intersection_max = min(self.GetMax(), range.GetMax());
        if intersection_min < intersection_max || self.Contains(range) || range.Contains(self) {
            Self::new(intersection_min, intersection_max)
        } else {
            Self::InvalidRange()
        }
    }
    pub fn Intersects(self, range: Self) -> bool {
        self.Intersect(range).IsValid()
    }
}
impl RectF {
    pub fn Contains(self, rect: Self) -> bool {
        rect.x() >= self.x()
            && rect.right() <= self.right()
            && rect.y() >= self.y()
            && rect.bottom() <= self.bottom()
    }
    pub fn IsEmpty(self) -> bool {
        self.width() == 0.0 || self.height() == 0.0
    }
    pub fn CenterPoint(self) -> PointF {
        PointF::new(
            self.x() + self.width() / 2.0,
            self.y() + self.height() / 2.0,
        )
    }
    pub fn set_height(&mut self, height: f32) {
        *self = Self::new(self.x(), self.y(), self.width(), height);
    }
    pub fn Union(&mut self, rect: Self) {
        if self.IsEmpty() {
            *self = rect;
            return;
        }
        if rect.IsEmpty() {
            return;
        }
        let rx = min(self.x(), rect.x());
        let ry = min(self.y(), rect.y());
        let rr = max(self.right(), rect.right());
        let rb = max(self.bottom(), rect.bottom());
        *self = Self::new(rx, ry, rr - rx, rb - ry);
        let mut width = self.width();
        let mut height = self.height();
        if self.right() < rr && width < f32::MAX {
            width = max(8.0 * f32::EPSILON, width).next_up();
        }
        if self.bottom() < rb && height < f32::MAX {
            height = max(8.0 * f32::EPSILON, height).next_up();
        }
        *self = Self::new(self.x(), self.y(), width, height);
    }
    pub fn ToString(self) -> String {
        format!(
            "{},{} {}x{}",
            self.x(),
            self.y(),
            self.width(),
            self.height()
        )
    }
}

//! gfx range/rectangle dependencies consumed by translated scroll snapping.
//! Adapter code, excluded from cc source coverage; no selection policy lives here.
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

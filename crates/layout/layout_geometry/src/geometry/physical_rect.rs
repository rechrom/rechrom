// cpp: layoutng_geometry/geometry/physical_rect.cc:5-10
// PhysicalRect, PhysicalOffset, PhysicalSize and Vector belong to foundation.
// A local extension trait carries the methods defined by this C++ package.
use super::box_strut::PhysicalBoxStrut;
use foundation::{FloorToInt, LayoutUnit, PhysicalOffset, PhysicalRect, PhysicalSize, Vector};

// cpp: layoutng_geometry/geometry/physical_rect.cc:14-147
#[allow(non_snake_case)]
pub trait PhysicalRectExt {
    fn DistanceAsSize(&self, target: PhysicalOffset) -> PhysicalSize;
    fn SquaredDistanceTo(&self, point: &PhysicalOffset) -> LayoutUnit;
    fn ContainsRect(&self, other: &PhysicalRect) -> bool;
    fn IntersectsRect(&self, other: &PhysicalRect) -> bool;
    fn IntersectsInclusively(&self, other: &PhysicalRect) -> bool;
    fn Unite(&mut self, other: &PhysicalRect);
    fn UniteIfNonZero(&mut self, other: &PhysicalRect);
    fn UniteEvenIfEmpty(&mut self, other: &PhysicalRect);
    fn Expand(&mut self, strut: &PhysicalBoxStrut);
    fn ExpandEdgesToPixelBoundaries(&mut self);
    fn Contract(&mut self, strut: &PhysicalBoxStrut);
    fn Intersect(&mut self, other: &PhysicalRect);
    fn InclusiveIntersect(&mut self, other: &PhysicalRect) -> bool;
}

#[allow(non_snake_case)]
impl PhysicalRectExt for PhysicalRect {
    // cpp: layoutng_geometry/geometry/physical_rect.cc:14-26
    fn DistanceAsSize(&self, mut target: PhysicalOffset) -> PhysicalSize {
        target -= self.offset;
        let mut distance = PhysicalSize::default();
        if target.left < LayoutUnit::default() {
            distance.width = -target.left;
        } else if target.left > self.size.width {
            distance.width = target.left - self.size.width;
        }
        if target.top < LayoutUnit::default() {
            distance.height = -target.top;
        } else if target.top > self.size.height {
            distance.height = target.top - self.size.height;
        }
        distance
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:28-38
    fn SquaredDistanceTo(&self, point: &PhysicalOffset) -> LayoutUnit {
        let mut x1 = self.X();
        let mut x2 = self.Right();
        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
        }
        let diff_x = point.left - clamp_unit(point.left, x1, x2);
        let mut y1 = self.Y();
        let mut y2 = self.Bottom();
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }
        let diff_y = point.top - clamp_unit(point.top, y1, y2);
        diff_x * diff_x + diff_y * diff_y
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:40-43
    fn ContainsRect(&self, other: &PhysicalRect) -> bool {
        self.offset.left <= other.offset.left
            && self.offset.top <= other.offset.top
            && self.Right() >= other.Right()
            && self.Bottom() >= other.Bottom()
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:45-50
    fn IntersectsRect(&self, other: &PhysicalRect) -> bool {
        !self.IsEmpty()
            && !other.IsEmpty()
            && self.offset.left < other.Right()
            && other.offset.left < self.Right()
            && self.offset.top < other.Bottom()
            && other.offset.top < self.Bottom()
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:52-56
    fn IntersectsInclusively(&self, other: &PhysicalRect) -> bool {
        self.offset.left <= other.Right()
            && other.offset.left <= self.Right()
            && self.offset.top <= other.Bottom()
            && other.offset.top <= self.Bottom()
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:58-67
    fn Unite(&mut self, other: &PhysicalRect) {
        if other.IsEmpty() {
            return;
        }
        if self.IsEmpty() {
            *self = *other;
            return;
        }
        self.UniteEvenIfEmpty(other);
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:69-78
    fn UniteIfNonZero(&mut self, other: &PhysicalRect) {
        if other.size.IsZero() {
            return;
        }
        if self.size.IsZero() {
            *self = *other;
            return;
        }
        self.UniteEvenIfEmpty(other);
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:80-94
    fn UniteEvenIfEmpty(&mut self, other: &PhysicalRect) {
        let left = min_unit(self.offset.left, other.offset.left);
        let top = min_unit(self.offset.top, other.offset.top);
        let right = max_unit(self.Right(), other.Right());
        let bottom = max_unit(self.Bottom(), other.Bottom());
        self.size = PhysicalSize::new(right - left, bottom - top);
        self.offset = PhysicalOffset::new(right - self.size.width, bottom - self.size.height);
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:96-98
    fn Expand(&mut self, strut: &PhysicalBoxStrut) {
        self.ExpandEdges(strut.top, strut.right, strut.bottom, strut.left);
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:100-109
    fn ExpandEdgesToPixelBoundaries(&mut self) {
        let left = FloorToInt(self.offset.left);
        let top = FloorToInt(self.offset.top);
        let max_right = (self.offset.left + self.size.width).Ceil();
        let max_bottom = (self.offset.top + self.size.height).Ceil();
        self.offset.left = LayoutUnit::from_signed(left);
        self.offset.top = LayoutUnit::from_signed(top);
        self.size.width = LayoutUnit::from_signed(max_right - left);
        self.size.height = LayoutUnit::from_signed(max_bottom - top);
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:111-113
    fn Contract(&mut self, strut: &PhysicalBoxStrut) {
        self.ExpandEdges(-strut.top, -strut.right, -strut.bottom, -strut.left);
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:115-130
    fn Intersect(&mut self, other: &PhysicalRect) {
        let mut new_offset =
            PhysicalOffset::new(max_unit(self.X(), other.X()), max_unit(self.Y(), other.Y()));
        let mut new_max_point = PhysicalOffset::new(
            min_unit(self.Right(), other.Right()),
            min_unit(self.Bottom(), other.Bottom()),
        );
        if new_offset.left >= new_max_point.left || new_offset.top >= new_max_point.top {
            new_offset = PhysicalOffset::default();
            new_max_point = PhysicalOffset::default();
        }
        self.offset = new_offset;
        self.size = PhysicalSize::new(
            new_max_point.left - new_offset.left,
            new_max_point.top - new_offset.top,
        );
    }

    // cpp: layoutng_geometry/geometry/physical_rect.cc:132-147
    fn InclusiveIntersect(&mut self, other: &PhysicalRect) -> bool {
        let new_offset =
            PhysicalOffset::new(max_unit(self.X(), other.X()), max_unit(self.Y(), other.Y()));
        let new_max_point = PhysicalOffset::new(
            min_unit(self.Right(), other.Right()),
            min_unit(self.Bottom(), other.Bottom()),
        );
        if new_offset.left > new_max_point.left || new_offset.top > new_max_point.top {
            *self = PhysicalRect::default();
            return false;
        }
        self.offset = new_offset;
        self.size = PhysicalSize::new(
            new_max_point.left - new_offset.left,
            new_max_point.top - new_offset.top,
        );
        true
    }
}

// cpp: layoutng_geometry/geometry/physical_rect.cc:151-156
#[allow(non_snake_case)]
pub fn UnionRect(rects: &Vector<PhysicalRect>) -> PhysicalRect {
    let mut result = PhysicalRect::default();
    for rect in rects.iter() {
        result.Unite(rect);
    }
    result
}

// cpp: layoutng_geometry/geometry/physical_rect.cc:158-168
#[allow(non_snake_case)]
pub fn UnionRectEvenIfEmpty(rects: &Vector<PhysicalRect>) -> PhysicalRect {
    let count = rects.len();
    if count == 0 {
        return PhysicalRect::default();
    }
    let mut result = rects[0];
    for i in 1..count {
        result.UniteEvenIfEmpty(&rects[i]);
    }
    result
}

fn clamp_unit(value: LayoutUnit, min: LayoutUnit, max: LayoutUnit) -> LayoutUnit {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
fn min_unit(a: LayoutUnit, b: LayoutUnit) -> LayoutUnit {
    if a <= b {
        a
    } else {
        b
    }
}
fn max_unit(a: LayoutUnit, b: LayoutUnit) -> LayoutUnit {
    if a >= b {
        a
    } else {
        b
    }
}

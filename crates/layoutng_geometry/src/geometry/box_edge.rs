// cpp: layoutng_geometry/geometry/box_edge.h:8-9
// Pending connection to //src/foundation:blink_geometry_api.
use foundation::LayoutUnit;

// cpp: layoutng_geometry/geometry/box_edge.h:13-21
/// One-dimensional projection of a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxEdge {
    pub offset: LayoutUnit,
    pub size: LayoutUnit,
}

#[allow(non_snake_case)]
impl BoxEdge {
    pub fn new(offset: LayoutUnit, size: LayoutUnit) -> Self {
        Self { offset, size }
    }

    // cpp: layoutng_geometry/geometry/box_edge.h:23-24
    pub fn End(&self) -> LayoutUnit {
        self.offset + self.size
    }

    pub fn IsEmpty(&self) -> bool {
        self.size <= LayoutUnit::default()
    }

    // cpp: layoutng_geometry/geometry/box_edge.h:26-29
    pub fn Contains(&self, value: LayoutUnit) -> bool {
        value >= self.offset && value < self.End()
    }

    // cpp: layoutng_geometry/geometry/box_edge.h:30-32
    pub fn ContainsEdge(&self, other: BoxEdge) -> bool {
        self.offset <= other.offset && self.End() >= other.End()
    }

    // cpp: layoutng_geometry/geometry/box_edge.h:33-36
    pub fn Intersects(&self, other: BoxEdge) -> bool {
        !self.IsEmpty()
            && !other.IsEmpty()
            && self.End() > other.offset
            && self.offset < other.End()
    }

    // cpp: layoutng_geometry/geometry/box_edge.h:38-39
    pub fn Move(&mut self, delta: LayoutUnit) {
        self.offset += delta;
    }
}

// cpp: layoutng_geometry/geometry/box_edge.h:41-44
impl std::ops::AddAssign<LayoutUnit> for BoxEdge {
    fn add_assign(&mut self, delta: LayoutUnit) {
        self.offset += delta;
    }
}

// cpp: layoutng_geometry/geometry/box_edge.h:45-48
impl std::ops::SubAssign<LayoutUnit> for BoxEdge {
    fn sub_assign(&mut self, delta: LayoutUnit) {
        self.offset -= delta;
    }
}

// cpp: layoutng_geometry/geometry/box_edge.h:50-59
// Equality is derived above. C++ operator+/- become Rust arithmetic traits.
impl std::ops::Add<LayoutUnit> for BoxEdge {
    type Output = Self;
    fn add(self, delta: LayoutUnit) -> Self {
        Self::new(self.offset + delta, self.size)
    }
}

impl std::ops::Sub<LayoutUnit> for BoxEdge {
    type Output = Self;
    fn sub(self, delta: LayoutUnit) -> Self {
        Self::new(self.offset - delta, self.size)
    }
}

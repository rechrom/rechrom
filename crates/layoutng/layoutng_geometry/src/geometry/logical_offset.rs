// cpp: layoutng_geometry/geometry/logical_offset.h:10-13
// cpp: layoutng_geometry/geometry/logical_offset.cc:7-11
// Pending foundation types: LayoutUnit, PhysicalOffset, PhysicalSize,
// WritingDirectionMode, String and StrCat.
use super::writing_mode_converter::WritingModeConverter;
use foundation::{LayoutUnit, PhysicalOffset, PhysicalSize, StrCat, String, WritingDirectionMode};

// cpp: layoutng_geometry/geometry/logical_offset.h:19-38
/// Position relative to the parent rectangle in logical coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalOffset {
    pub inline_offset: LayoutUnit,
    pub block_offset: LayoutUnit,
}

#[allow(non_snake_case)]
impl LogicalOffset {
    pub fn new(inline_offset: LayoutUnit, block_offset: LayoutUnit) -> Self {
        Self {
            inline_offset,
            block_offset,
        }
    }

    // cpp: layoutng_geometry/geometry/logical_offset.h:40-49
    // cpp: layoutng_geometry/geometry/logical_offset.cc:15-21
    pub fn ConvertToPhysical(
        &self,
        writing_direction: WritingDirectionMode,
        outer_size: PhysicalSize,
        inner_size: PhysicalSize,
    ) -> PhysicalOffset {
        WritingModeConverter::new(writing_direction, outer_size).ToPhysicalOffset(*self, inner_size)
    }

    // cpp: layoutng_geometry/geometry/logical_offset.h:72-87
    // Componentwise comparisons cannot implement Rust PartialOrd faithfully:
    // one coordinate may be equal while the other is greater.
    pub fn GreaterThan(&self, other: &Self) -> bool {
        self.inline_offset > other.inline_offset && self.block_offset > other.block_offset
    }
    pub fn GreaterOrEqual(&self, other: &Self) -> bool {
        self.inline_offset >= other.inline_offset && self.block_offset >= other.block_offset
    }
    pub fn LessThan(&self, other: &Self) -> bool {
        self.inline_offset < other.inline_offset && self.block_offset < other.block_offset
    }
    pub fn LessOrEqual(&self, other: &Self) -> bool {
        self.inline_offset <= other.inline_offset && self.block_offset <= other.block_offset
    }

    // cpp: layoutng_geometry/geometry/logical_offset.h:89
    // cpp: layoutng_geometry/geometry/logical_offset.cc:23-25
    pub fn ToString(&self) -> String {
        StrCat(&[
            self.inline_offset.ToString().into(),
            String::from(","),
            self.block_offset.ToString().into(),
        ])
    }
}

// cpp: layoutng_geometry/geometry/logical_offset.h:51-56
// Equality is derived above. The deleted f64 and test-only i32 constructors
// have no production Rust equivalent; `new` requires two LayoutUnit values.
impl std::ops::Add<Self> for LogicalOffset {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(
            self.inline_offset + other.inline_offset,
            self.block_offset + other.block_offset,
        )
    }
}

// cpp: layoutng_geometry/geometry/logical_offset.h:58-61
impl std::ops::AddAssign<Self> for LogicalOffset {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

// cpp: layoutng_geometry/geometry/logical_offset.h:63-67
impl std::ops::SubAssign<Self> for LogicalOffset {
    fn sub_assign(&mut self, other: Self) {
        self.inline_offset -= other.inline_offset;
        self.block_offset -= other.block_offset;
    }
}

// cpp: layoutng_geometry/geometry/logical_offset.h:92
// cpp: layoutng_geometry/geometry/logical_offset.cc:27-29
impl std::fmt::Display for LogicalOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ToString())
    }
}

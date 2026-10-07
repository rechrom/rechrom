// cpp: layoutng_geometry/geometry/logical_size.h:8-13
// Pending connection to foundation geometry and writing-mode APIs.
use super::box_strut::BoxStrut;
use super::logical_offset::LogicalOffset;
use foundation::{kIndefiniteSize, IsHorizontalWritingMode, LayoutUnit, PhysicalSize, WritingMode};

// cpp: layoutng_geometry/geometry/logical_size.h:19-42
/// Width and height in the logical coordinate system.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalSize {
    pub inline_size: LayoutUnit,
    pub block_size: LayoutUnit,
}

#[allow(non_snake_case)]
impl LogicalSize {
    pub fn new(inline_size: LayoutUnit, block_size: LayoutUnit) -> Self {
        Self {
            inline_size,
            block_size,
        }
    }

    // cpp: layoutng_geometry/geometry/logical_size.h:49-51
    pub fn IsEmpty(&self) -> bool {
        self.inline_size == LayoutUnit::default() || self.block_size == LayoutUnit::default()
    }

    // cpp: layoutng_geometry/geometry/logical_size.h:53-56
    pub fn Expand(&mut self, inline_offset: LayoutUnit, block_offset: LayoutUnit) {
        self.inline_size += inline_offset;
        self.block_size += block_offset;
    }

    // cpp: layoutng_geometry/geometry/logical_size.h:58-61
    pub fn Shrink(&mut self, inline_offset: LayoutUnit, block_offset: LayoutUnit) {
        self.inline_size -= inline_offset;
        self.block_size -= block_offset;
    }

    // cpp: layoutng_geometry/geometry/logical_size.h:63-66
    pub fn ClampNegativeToZero(&self) -> Self {
        Self::new(
            self.inline_size.ClampNegativeToZero(),
            self.block_size.ClampNegativeToZero(),
        )
    }

    // cpp: layoutng_geometry/geometry/logical_size.h:68-71
    pub fn ClampIndefiniteToZero(&self) -> Self {
        Self::new(
            self.inline_size.ClampIndefiniteToZero(),
            self.block_size.ClampIndefiniteToZero(),
        )
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:28-35
// The deleted double constructor remains unavailable; the testing-only int
// constructor is defined outside this production Bazel package.

// cpp: layoutng_geometry/geometry/logical_size.h:44-47
impl std::ops::Mul<f32> for LogicalSize {
    type Output = Self;
    fn mul(self, scale: f32) -> Self {
        Self::new(
            LayoutUnit::from_f32(self.inline_size.ToFloat() * scale),
            LayoutUnit::from_f32(self.block_size.ToFloat() * scale),
        )
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:74
#[allow(non_upper_case_globals)]
pub const kIndefiniteLogicalSize: LogicalSize = LogicalSize {
    inline_size: kIndefiniteSize,
    block_size: kIndefiniteSize,
};

// cpp: layoutng_geometry/geometry/logical_size.h:76-78
impl std::ops::Sub<BoxStrut> for LogicalSize {
    type Output = Self;
    fn sub(self, b: BoxStrut) -> Self {
        Self::new(
            self.inline_size - b.InlineSum(),
            self.block_size - b.BlockSum(),
        )
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:80-84
impl std::ops::SubAssign<BoxStrut> for LogicalSize {
    fn sub_assign(&mut self, b: BoxStrut) {
        self.inline_size -= b.InlineSum();
        self.block_size -= b.BlockSum();
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:86-88
impl std::ops::Add<BoxStrut> for LogicalSize {
    type Output = Self;
    fn add(self, b: BoxStrut) -> Self {
        Self::new(
            self.inline_size + b.InlineSum(),
            self.block_size + b.BlockSum(),
        )
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:90-94
impl std::ops::Add<LogicalSize> for LogicalOffset {
    type Output = Self;
    fn add(self, size: LogicalSize) -> Self {
        Self::new(
            self.inline_offset + size.inline_size,
            self.block_offset + size.block_size,
        )
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:96-100
impl std::ops::AddAssign<LogicalSize> for LogicalOffset {
    fn add_assign(&mut self, size: LogicalSize) {
        *self = *self + size;
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:102-105
#[allow(non_snake_case)]
pub fn ToLogicalSize(size: PhysicalSize, mode: WritingMode) -> LogicalSize {
    if IsHorizontalWritingMode(mode) {
        LogicalSize::new(size.width, size.height)
    } else {
        LogicalSize::new(size.height, size.width)
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:107-111
#[allow(non_snake_case)]
pub fn ToPhysicalSize(size: LogicalSize, mode: WritingMode) -> PhysicalSize {
    if IsHorizontalWritingMode(mode) {
        PhysicalSize::new(size.inline_size, size.block_size)
    } else {
        PhysicalSize::new(size.block_size, size.inline_size)
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:113
// This ostream insertion operator has no definition in the supplied C++ tree.
// It remains an unresolved source symbol; no output behavior is invented here.

// cpp: layoutng_geometry/geometry/logical_size.h:115-127
/// An offset delta that can be interpreted as either a size or an offset.
/// Rust uses explicit From conversions in place of C++ public inheritance.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalDelta {
    pub inline_size: LayoutUnit,
    pub block_size: LayoutUnit,
}

impl LogicalDelta {
    pub fn new(inline_size: LayoutUnit, block_size: LayoutUnit) -> Self {
        Self {
            inline_size,
            block_size,
        }
    }
}

impl From<LogicalDelta> for LogicalSize {
    fn from(delta: LogicalDelta) -> Self {
        Self::new(delta.inline_size, delta.block_size)
    }
}

impl From<LogicalDelta> for LogicalOffset {
    fn from(delta: LogicalDelta) -> Self {
        Self::new(delta.inline_size, delta.block_size)
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:115-127
// The C++ implicit conversion lets offset += delta use LogicalOffset::operator+=.
impl std::ops::AddAssign<LogicalDelta> for LogicalOffset {
    fn add_assign(&mut self, delta: LogicalDelta) {
        *self += LogicalOffset::from(delta);
    }
}

// cpp: layoutng_geometry/geometry/logical_size.h:129-131
impl std::ops::Sub<LogicalOffset> for LogicalOffset {
    type Output = LogicalDelta;
    fn sub(self, other: LogicalOffset) -> LogicalDelta {
        LogicalDelta::new(
            self.inline_offset - other.inline_offset,
            self.block_offset - other.block_offset,
        )
    }
}

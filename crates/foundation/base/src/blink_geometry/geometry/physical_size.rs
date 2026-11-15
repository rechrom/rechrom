// C++: src/foundation/blink_geometry/geometry/physical_size.h

use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::gfx_geometry::{Size, SizeF};

use super::layout_unit::LayoutUnit;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AspectRatioFit {
    kAspectRatioFitShrink,
    kAspectRatioFitGrow,
}

// C++ struct PhysicalSize -> Rust struct + impl.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalSize {
    pub width: LayoutUnit,
    pub height: LayoutUnit,
}

impl PhysicalSize {
    pub fn new(width: LayoutUnit, height: LayoutUnit) -> Self {
        Self { width, height }
    }

    pub fn IsEmpty(&self) -> bool {
        self.width == LayoutUnit::new() || self.height == LayoutUnit::new()
    }

    pub fn IsZero(&self) -> bool {
        self.width == LayoutUnit::new() && self.height == LayoutUnit::new()
    }

    pub fn HasFraction(&self) -> bool {
        self.width.HasFraction() || self.height.HasFraction()
    }

    pub fn ScaleFloat(&mut self, scale: f32) {
        self.width *= scale;
        self.height *= scale;
    }

    pub fn Scale(&mut self, scale: LayoutUnit) {
        self.width *= scale;
        self.height *= scale;
    }

    pub fn ExpandedTo(&self, other: &Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    pub fn ShrunkTo(&self, other: &Self) -> Self {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }

    pub fn ClampNegativeToZero(&mut self) {
        self.width = self.width.max(LayoutUnit::new());
        self.height = self.height.max(LayoutUnit::new());
    }

    pub fn FromSizeFRound(size: &SizeF) -> Self {
        Self::new(
            LayoutUnit::FromFloatRound(size.width()),
            LayoutUnit::FromFloatRound(size.height()),
        )
    }

    pub fn FromSizeFFloor(size: &SizeF) -> Self {
        Self::new(
            LayoutUnit::FromFloatFloor(size.width()),
            LayoutUnit::FromFloatFloor(size.height()),
        )
    }

    pub fn FromSize(size: &Size) -> Self {
        Self::new(
            LayoutUnit::from_signed(size.width()),
            LayoutUnit::from_signed(size.height()),
        )
    }
}

impl From<PhysicalSize> for SizeF {
    fn from(size: PhysicalSize) -> Self {
        SizeF::new(size.width.ToFloat(), size.height.ToFloat())
    }
}

impl Add for PhysicalSize {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.width + other.width, self.height + other.height)
    }
}

impl AddAssign for PhysicalSize {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Neg for PhysicalSize {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.width, -self.height)
    }
}

impl Sub for PhysicalSize {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.width - other.width, self.height - other.height)
    }
}

impl SubAssign for PhysicalSize {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul<f32> for PhysicalSize {
    type Output = Self;
    fn mul(self, scale: f32) -> Self {
        Self::new(
            LayoutUnit::from_f32(self.width * scale),
            LayoutUnit::from_f32(self.height * scale),
        )
    }
}

pub fn ToRoundedSize(size: &PhysicalSize) -> Size {
    Size::new(size.width.Round(), size.height.Round())
}

pub fn ToFlooredSize(size: &PhysicalSize) -> Size {
    Size::new(size.width.Floor(), size.height.Floor())
}

pub fn ToCeiledSize(size: &PhysicalSize) -> Size {
    Size::new(size.width.Ceil(), size.height.Ceil())
}

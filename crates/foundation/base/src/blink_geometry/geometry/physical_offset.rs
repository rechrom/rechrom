// C++: src/foundation/blink_geometry/geometry/physical_offset.h

use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use crate::gfx_geometry::{Point, PointF, Vector2d, Vector2dF};

use super::layout_unit::LayoutUnit;

// C++ template struct PhysicalFixedOffset<ValueType> -> Rust generic struct + impl.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhysicalFixedOffset<ValueType> {
    pub left: ValueType,
    pub top: ValueType,
}

impl<ValueType> PhysicalFixedOffset<ValueType> {
    pub fn new(left: ValueType, top: ValueType) -> Self {
        Self { left, top }
    }
}

impl<ValueType: Default + PartialEq> PhysicalFixedOffset<ValueType> {
    pub fn IsZero(&self) -> bool {
        self.left == ValueType::default() && self.top == ValueType::default()
    }
}

pub trait OffsetValue: Copy + Default + Ord {
    fn HasFraction(self) -> bool;
    fn FromFloatFloor(value: f32) -> Self;
    fn FromFloatRound(value: f32) -> Self;
    fn Round(self) -> i32;
    fn Floor(self) -> i32;
    fn Ceil(self) -> i32;
}

impl OffsetValue for LayoutUnit {
    fn HasFraction(self) -> bool {
        LayoutUnit::HasFraction(&self)
    }
    fn FromFloatFloor(value: f32) -> Self {
        LayoutUnit::FromFloatFloor(value)
    }
    fn FromFloatRound(value: f32) -> Self {
        LayoutUnit::FromFloatRound(value)
    }
    fn Round(self) -> i32 {
        LayoutUnit::Round(&self)
    }
    fn Floor(self) -> i32 {
        LayoutUnit::Floor(&self)
    }
    fn Ceil(self) -> i32 {
        LayoutUnit::Ceil(&self)
    }
}

impl<ValueType: OffsetValue> PhysicalFixedOffset<ValueType> {
    pub fn HasFraction(&self) -> bool {
        self.left.HasFraction() || self.top.HasFraction()
    }

    pub fn ClampNegativeToZero(&mut self) {
        self.left = self.left.max(ValueType::default());
        self.top = self.top.max(ValueType::default());
    }

    pub fn FromPointFFloor(point: &PointF) -> Self {
        Self::new(
            ValueType::FromFloatFloor(point.x()),
            ValueType::FromFloatFloor(point.y()),
        )
    }

    pub fn FromPointFRound(point: &PointF) -> Self {
        Self::new(
            ValueType::FromFloatRound(point.x()),
            ValueType::FromFloatRound(point.y()),
        )
    }

    pub fn FromVector2dFFloor(vector: &Vector2dF) -> Self {
        Self::new(
            ValueType::FromFloatFloor(vector.x()),
            ValueType::FromFloatFloor(vector.y()),
        )
    }

    pub fn FromVector2dFRound(vector: &Vector2dF) -> Self {
        Self::new(
            ValueType::FromFloatRound(vector.x()),
            ValueType::FromFloatRound(vector.y()),
        )
    }
}

impl PhysicalFixedOffset<LayoutUnit> {
    pub fn FromPoint(point: &Point) -> Self {
        Self::new(
            LayoutUnit::from_signed(point.x()),
            LayoutUnit::from_signed(point.y()),
        )
    }

    pub fn FromVector2d(vector: &Vector2d) -> Self {
        Self::new(
            LayoutUnit::from_signed(vector.x()),
            LayoutUnit::from_signed(vector.y()),
        )
    }

    pub fn Scale(&mut self, scale: f32) {
        self.left *= scale;
        self.top *= scale;
    }
}

impl From<PhysicalFixedOffset<LayoutUnit>> for PointF {
    fn from(value: PhysicalFixedOffset<LayoutUnit>) -> Self {
        PointF::new(value.left.ToFloat(), value.top.ToFloat())
    }
}

impl From<PhysicalFixedOffset<LayoutUnit>> for Vector2dF {
    fn from(value: PhysicalFixedOffset<LayoutUnit>) -> Self {
        Vector2dF::new(value.left.ToFloat(), value.top.ToFloat())
    }
}

impl<T: Copy + Add<Output = T>> Add for PhysicalFixedOffset<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.left + other.left, self.top + other.top)
    }
}

impl<T: Copy + Add<Output = T>> AddAssign for PhysicalFixedOffset<T> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<T: Neg<Output = T>> Neg for PhysicalFixedOffset<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.left, -self.top)
    }
}

impl<T: Copy + Sub<Output = T>> Sub for PhysicalFixedOffset<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.left - other.left, self.top - other.top)
    }
}

impl<T: Copy + Sub<Output = T>> SubAssign for PhysicalFixedOffset<T> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

pub type PhysicalOffset = PhysicalFixedOffset<LayoutUnit>;

pub fn ToRoundedPoint<ValueType: OffsetValue>(offset: &PhysicalFixedOffset<ValueType>) -> Point {
    Point::new(offset.left.Round(), offset.top.Round())
}

pub fn ToFlooredPoint<ValueType: OffsetValue>(offset: &PhysicalFixedOffset<ValueType>) -> Point {
    Point::new(offset.left.Floor(), offset.top.Floor())
}

pub fn ToCeiledPoint<ValueType: OffsetValue>(offset: &PhysicalFixedOffset<ValueType>) -> Point {
    Point::new(offset.left.Ceil(), offset.top.Ceil())
}

pub fn ToRoundedVector2d<ValueType: OffsetValue>(
    offset: &PhysicalFixedOffset<ValueType>,
) -> Vector2d {
    Vector2d::new(offset.left.Round(), offset.top.Round())
}

pub fn ToFlooredVector2d<ValueType: OffsetValue>(
    offset: &PhysicalFixedOffset<ValueType>,
) -> Vector2d {
    Vector2d::new(offset.left.Floor(), offset.top.Floor())
}

pub fn ToCeiledVector2d<ValueType: OffsetValue>(
    offset: &PhysicalFixedOffset<ValueType>,
) -> Vector2d {
    Vector2d::new(offset.left.Ceil(), offset.top.Ceil())
}

// C++: src/foundation/blink_geometry/geometry/layout_unit.h

use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Sub, SubAssign};

// C++ Storage template parameter: the original file instantiates i32 and i64.
pub trait FixedStorage: Copy + Debug + Eq + Ord {
    type Unsigned: Copy + Debug + Eq;
    const BITS: u32;
    const MIN: i128;
    const MAX: i128;
    fn from_i128_wrapping(value: i128) -> Self;
    fn to_i128(self) -> i128;
    fn to_unsigned_wrapping(self) -> Self::Unsigned;
    fn saturating_add(self, rhs: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn saturating_mul_i32(self, rhs: i32) -> Self;
    fn saturating_neg(self) -> Self;
}

impl FixedStorage for i32 {
    type Unsigned = u32;
    const BITS: u32 = 32;
    const MIN: i128 = i32::MIN as i128;
    const MAX: i128 = i32::MAX as i128;

    fn from_i128_wrapping(value: i128) -> Self {
        value as i32
    }

    fn to_i128(self) -> i128 {
        self as i128
    }

    fn to_unsigned_wrapping(self) -> Self::Unsigned {
        self as u32
    }

    #[inline(always)]
    fn saturating_add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }

    #[inline(always)]
    fn saturating_sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }

    #[inline(always)]
    fn saturating_mul_i32(self, rhs: i32) -> Self {
        self.saturating_mul(rhs)
    }

    #[inline(always)]
    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl FixedStorage for i64 {
    type Unsigned = u64;
    const BITS: u32 = 64;
    const MIN: i128 = i64::MIN as i128;
    const MAX: i128 = i64::MAX as i128;

    fn from_i128_wrapping(value: i128) -> Self {
        value as i64
    }

    fn to_i128(self) -> i128 {
        self as i128
    }

    fn to_unsigned_wrapping(self) -> Self::Unsigned {
        self as u64
    }

    #[inline(always)]
    fn saturating_add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }

    #[inline(always)]
    fn saturating_sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }

    #[inline(always)]
    fn saturating_mul_i32(self, rhs: i32) -> Self {
        self.saturating_mul(i64::from(rhs))
    }

    #[inline(always)]
    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

// C++ class FixedPoint<fractional_bits, Storage> -> Rust struct + impl.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct FixedPoint<const FRACTIONAL_BITS: u32, Storage: FixedStorage> {
    value_: Storage,
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> FixedPoint<FRACTIONAL_BITS, Storage> {
    pub const kFractionalBits: u32 = FRACTIONAL_BITS;
    pub const kIntegralBits: u32 = Storage::BITS - FRACTIONAL_BITS;
    pub const kFixedPointDenominator: i128 = 1_i128 << FRACTIONAL_BITS;
    pub const kRawValueMax: i128 = Storage::MAX;
    pub const kRawValueMin: i128 = Storage::MIN;
    pub const kIntMax: i128 = Storage::MAX / Self::kFixedPointDenominator;
    pub const kIntMin: i128 = Storage::MIN / Self::kFixedPointDenominator;

    // C++: ClampRawValue<T>. f32/f64 callers use the float variant below.
    pub fn ClampRawValue(raw_value: i128) -> Storage {
        Storage::from_i128_wrapping(raw_value.clamp(Storage::MIN, Storage::MAX))
    }

    fn ClampRawFloat(raw_value: f64) -> Storage {
        if raw_value.is_nan() {
            return Storage::from_i128_wrapping(0);
        }
        if raw_value >= Storage::MAX as f64 {
            return Storage::from_i128_wrapping(Storage::MAX);
        }
        if raw_value <= Storage::MIN as f64 {
            return Storage::from_i128_wrapping(Storage::MIN);
        }
        Storage::from_i128_wrapping(raw_value.trunc() as i128)
    }

    // C++: FixedPoint(), FixedPoint(signed int), FixedPoint(unsigned int).
    pub fn new() -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(0))
    }

    pub fn from_signed(value: i32) -> Self {
        Self::FromRawValue(Self::SaturatedSetNonAsmSigned(value))
    }

    pub fn from_unsigned(value: u32) -> Self {
        Self::FromRawValue(Self::SaturatedSetNonAsmUnsigned(value))
    }

    // C++: FixedPoint(integral value wider than int).
    pub fn from_wide(value: i64) -> Self {
        Self::FromRawValueWithClamp(value as i128 * Self::kFixedPointDenominator)
    }

    // C++: FixedPoint(float), FixedPoint(double).
    pub fn from_f32(value: f32) -> Self {
        Self::FromRawValue(Self::ClampRawFloat(
            (value * Self::kFixedPointDenominator as f32) as f64,
        ))
    }

    pub fn from_f64(value: f64) -> Self {
        Self::FromRawValue(Self::ClampRawFloat(
            value * Self::kFixedPointDenominator as f64,
        ))
    }

    // C++: FromFloatCeil, FromFloatFloor, FromFloatRound, FromDoubleRound.
    pub fn FromFloatCeil(value: f32) -> Self {
        Self::FromRawValue(Self::ClampRawFloat(
            (value * Self::kFixedPointDenominator as f32).ceil() as f64,
        ))
    }

    pub fn FromFloatFloor(value: f32) -> Self {
        Self::FromRawValue(Self::ClampRawFloat(
            (value * Self::kFixedPointDenominator as f32).floor() as f64,
        ))
    }

    pub fn FromFloatRound(value: f32) -> Self {
        Self::FromRawValue(Self::ClampRawFloat(
            (value * Self::kFixedPointDenominator as f32).round() as f64,
        ))
    }

    pub fn FromDoubleRound(value: f64) -> Self {
        Self::FromRawValue(Self::ClampRawFloat(
            (value * Self::kFixedPointDenominator as f64).round(),
        ))
    }

    // C++: FromRawValue and FromRawValueWithClamp.
    #[inline(always)]
    pub fn FromRawValue(raw_value: Storage) -> Self {
        Self { value_: raw_value }
    }

    pub fn FromRawValueWithClamp(raw_value: i128) -> Self {
        Self::FromRawValue(Self::ClampRawValue(raw_value))
    }

    // C++: FromFloatEncompassRound.
    pub fn FromFloatEncompassRound(start_value: f32, end_value: f32) -> (Self, Self) {
        if start_value < end_value {
            (
                Self::FromFloatFloor(start_value),
                Self::FromFloatCeil(end_value),
            )
        } else if start_value > end_value {
            (
                Self::FromFloatCeil(start_value),
                Self::FromFloatFloor(end_value),
            )
        } else {
            let position = Self::FromFloatFloor(start_value);
            (position, position)
        }
    }

    // C++: lossless constructor from FixedPoint with different parameters.
    pub fn from_lossless<const SOURCE_BITS: u32, SourceStorage: FixedStorage>(
        source: FixedPoint<SOURCE_BITS, SourceStorage>,
    ) -> Self {
        assert!(Storage::BITS > SourceStorage::BITS);
        assert!(FRACTIONAL_BITS >= SOURCE_BITS);
        assert!(Self::kIntegralBits >= FixedPoint::<SOURCE_BITS, SourceStorage>::kIntegralBits);
        Self::FromRawValue(Storage::from_i128_wrapping(
            source.RawValue().to_i128() << (FRACTIONAL_BITS - SOURCE_BITS),
        ))
    }

    // C++: the three FromFixed<source_fractional_bits> overloads.
    pub fn FromFixed<const SOURCE_BITS: u32>(value: i128) -> Self {
        if SOURCE_BITS == FRACTIONAL_BITS {
            return Self::FromRawValueWithClamp(value);
        }
        if SOURCE_BITS > FRACTIONAL_BITS {
            return Self::FromRawValueWithClamp(value >> (SOURCE_BITS - FRACTIONAL_BITS));
        }
        let bits_diff = FRACTIONAL_BITS - SOURCE_BITS;
        if value >= Self::kRawValueMax >> bits_diff {
            return Self::Max();
        }
        if value <= Self::kRawValueMin >> bits_diff {
            return Self::Min();
        }
        Self::FromRawValue(Storage::from_i128_wrapping(value << bits_diff))
    }

    // C++: To<Target> and ToCeil<Target>.
    #[inline(always)]
    pub fn To<const TARGET_BITS: u32, TargetStorage: FixedStorage>(
        &self,
    ) -> FixedPoint<TARGET_BITS, TargetStorage> {
        // C++ selects the exact-fractional-bits FromFixed(Storage) overload
        // for widening conversions. That overload is a raw integer move; it
        // does not pass through the saturating i128 conversion used by the
        // lossy overloads. Keep that distinction here because glyph advances
        // convert TextRunLayoutUnit (i32.16) to InlineLayoutUnit (i64.16) in
        // the innermost line-breaking loops.
        if TARGET_BITS == FRACTIONAL_BITS && TargetStorage::BITS >= Storage::BITS {
            return FixedPoint::<TARGET_BITS, TargetStorage>::FromRawValue(
                TargetStorage::from_i128_wrapping(self.RawValue().to_i128()),
            );
        }
        FixedPoint::<TARGET_BITS, TargetStorage>::FromFixed::<FRACTIONAL_BITS>(
            self.RawValue().to_i128(),
        )
    }

    pub fn ToCeil<const TARGET_BITS: u32, TargetStorage: FixedStorage>(
        &self,
    ) -> FixedPoint<TARGET_BITS, TargetStorage> {
        assert!(TARGET_BITS < FRACTIONAL_BITS);
        assert!(TargetStorage::BITS < Storage::BITS);
        let bits_diff = FRACTIONAL_BITS - TARGET_BITS;
        let mut raw_value = self.RawValue().to_i128() >> bits_diff;
        if self.RawValue().to_i128() & ((1_i128 << bits_diff) - 1) != 0 {
            raw_value += 1;
        }
        FixedPoint::<TARGET_BITS, TargetStorage>::FromRawValueWithClamp(raw_value)
    }

    // C++: ToInt, ToFloat, ToDouble, ToUnsigned.
    pub fn ToInt(&self) -> Storage {
        Storage::from_i128_wrapping(self.value_.to_i128() / Self::kFixedPointDenominator)
    }

    #[inline(always)]
    pub fn ToFloat(&self) -> f32 {
        self.value_.to_i128() as f32 / Self::kFixedPointDenominator as f32
    }

    pub fn ToDouble(&self) -> f64 {
        self.value_.to_i128() as f64 / Self::kFixedPointDenominator as f64
    }

    pub fn ToUnsigned(&self) -> Storage::Unsigned {
        debug_assert!(self.value_.to_i128() >= 0);
        self.ToInt().to_unsigned_wrapping()
    }

    // C++: operator++(int) returns the incremented value.
    pub fn PostIncrement(&mut self) -> Self {
        self.value_ = self
            .value_
            .saturating_add(Storage::from_i128_wrapping(Self::kFixedPointDenominator));
        *self
    }

    // C++: RawValue and both SetRawValue overloads.
    #[inline(always)]
    pub fn RawValue(&self) -> Storage {
        self.value_
    }

    pub fn SetRawValueI32(&mut self, value: i32) {
        self.value_ = Storage::from_i128_wrapping(value as i128);
    }

    pub fn SetRawValueI64(&mut self, value: i64) {
        if Storage::BITS < 64 {
            debug_assert!(
                value as i128 > Self::kRawValueMin && (value as i128) < Self::kRawValueMax
            );
        }
        self.value_ = Storage::from_i128_wrapping(value as i128);
    }

    // C++: Abs, Ceil, Round, Floor.
    pub fn Abs(&self) -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(self.value_.to_i128().abs()))
    }

    pub fn Ceil(&self) -> Storage {
        if self.value_.to_i128() >= Self::kRawValueMax - Self::kFixedPointDenominator + 1 {
            return Storage::from_i128_wrapping(Self::kIntMax);
        }
        if self.value_.to_i128() >= 0 {
            return Storage::from_i128_wrapping(
                (self.value_.to_i128() + Self::kFixedPointDenominator - 1)
                    / Self::kFixedPointDenominator,
            );
        }
        self.ToInt()
    }

    pub fn Round(&self) -> Storage {
        Storage::from_i128_wrapping(
            self.ToInt().to_i128()
                + ((self.Fraction().RawValue().to_i128() + (Self::kFixedPointDenominator / 2))
                    >> FRACTIONAL_BITS),
        )
    }

    pub fn Floor(&self) -> Storage {
        if self.value_.to_i128() <= Self::kRawValueMin + Self::kFixedPointDenominator - 1 {
            return Storage::from_i128_wrapping(Self::kIntMin);
        }
        Storage::from_i128_wrapping(self.value_.to_i128() >> FRACTIONAL_BITS)
    }

    // C++: ClampNegativeToZero, ClampPositiveToZero, ClampIndefiniteToZero.
    pub fn ClampNegativeToZero(&self) -> Self {
        if self.value_.to_i128() < 0 {
            Self::new()
        } else {
            *self
        }
    }

    pub fn ClampPositiveToZero(&self) -> Self {
        if self.value_.to_i128() > 0 {
            Self::new()
        } else {
            *self
        }
    }

    pub fn ClampIndefiniteToZero(&self) -> Self {
        if self.value_.to_i128() == -Self::kFixedPointDenominator {
            return Self::new();
        }
        debug_assert!(self.value_.to_i128() >= 0);
        *self
    }

    // C++: HasFraction, IsInteger, Fraction, MightBeSaturated.
    pub fn HasFraction(&self) -> bool {
        self.RawValue().to_i128() % Self::kFixedPointDenominator != 0
    }

    pub fn IsInteger(&self) -> bool {
        !self.HasFraction()
    }

    pub fn Fraction(&self) -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(
            self.RawValue().to_i128() % Self::kFixedPointDenominator,
        ))
    }

    pub fn MightBeSaturated(&self) -> bool {
        self.RawValue().to_i128() == Self::kRawValueMax
            || self.RawValue().to_i128() == Self::kRawValueMin
    }

    // C++: Epsilon, AddEpsilon, Max, Min, NearlyMax, NearlyMin, Clamp.
    pub fn Epsilon() -> f32 {
        1.0 / Self::kFixedPointDenominator as f32
    }

    pub fn AddEpsilon(&self) -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(
            if self.value_.to_i128() < Self::kRawValueMax {
                self.value_.to_i128() + 1
            } else {
                self.value_.to_i128()
            },
        ))
    }

    pub fn Max() -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(Self::kRawValueMax))
    }

    pub fn Min() -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(Self::kRawValueMin))
    }

    pub fn NearlyMax() -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(
            Self::kRawValueMax - Self::kFixedPointDenominator / 2,
        ))
    }

    pub fn NearlyMin() -> Self {
        Self::FromRawValue(Storage::from_i128_wrapping(
            Self::kRawValueMin + Self::kFixedPointDenominator / 2,
        ))
    }

    pub fn Clamp(value: f64) -> Self {
        Self::FromRawValue(Self::ClampRawFloat(
            (value * Self::kFixedPointDenominator as f64).floor(),
        ))
    }

    // C++: MulDiv, defined after the fixed-point division operator.
    pub fn MulDiv(&self, m: Self, d: Self) -> Self {
        let n = self.RawValue().to_i128() * m.RawValue().to_i128();
        let q = n / d.RawValue().to_i128();
        Self::FromRawValueWithClamp(q)
    }

    // C++: NullOptIf and NullOptIfMin.
    pub fn NullOptIf(&self, null_value: Self) -> Option<Self> {
        if *self == null_value {
            None
        } else {
            Some(*self)
        }
    }

    pub fn NullOptIfMin(&self) -> Option<Self> {
        self.NullOptIf(Self::Min())
    }

    // C++: SaturatedSetNonAsm(int) and SaturatedSetNonAsm(unsigned).
    fn SaturatedSetNonAsmSigned(value: i32) -> Storage {
        let value = value as i128;
        if value > Self::kIntMax {
            Storage::from_i128_wrapping(Self::kRawValueMax)
        } else if value < Self::kIntMin {
            Storage::from_i128_wrapping(Self::kRawValueMin)
        } else {
            Storage::from_i128_wrapping(value << FRACTIONAL_BITS)
        }
    }

    fn SaturatedSetNonAsmUnsigned(value: u32) -> Storage {
        let value = value as i128;
        if value >= Self::kIntMax {
            Storage::from_i128_wrapping(Self::kRawValueMax)
        } else {
            Storage::from_i128_wrapping(value << FRACTIONAL_BITS)
        }
    }
}

pub type LayoutUnit = FixedPoint<6, i32>;
pub type TextRunLayoutUnit = FixedPoint<16, i32>;
pub type InlineLayoutUnit = FixedPoint<16, i64>;

// C++: implicit conversion operators for float, double, and bool.
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> From<FixedPoint<FRACTIONAL_BITS, Storage>>
    for f32
{
    fn from(value: FixedPoint<FRACTIONAL_BITS, Storage>) -> Self {
        value.ToFloat()
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> From<FixedPoint<FRACTIONAL_BITS, Storage>>
    for f64
{
    fn from(value: FixedPoint<FRACTIONAL_BITS, Storage>) -> Self {
        value.ToDouble()
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> From<FixedPoint<FRACTIONAL_BITS, Storage>>
    for bool
{
    fn from(value: FixedPoint<FRACTIONAL_BITS, Storage>) -> Self {
        value.RawValue().to_i128() != 0
    }
}

// C++: kIndefiniteSize is LayoutUnit(-1).
pub const kIndefiniteSize: LayoutUnit = FixedPoint { value_: -64 };

// C++: LayoutUnit comparisons with int.
impl PartialEq<i32> for LayoutUnit {
    fn eq(&self, other: &i32) -> bool {
        *self == LayoutUnit::from_signed(*other)
    }
}

impl PartialEq<LayoutUnit> for i32 {
    fn eq(&self, other: &LayoutUnit) -> bool {
        LayoutUnit::from_signed(*self) == *other
    }
}

impl PartialOrd<i32> for LayoutUnit {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.partial_cmp(&LayoutUnit::from_signed(*other))
    }
}

impl PartialOrd<LayoutUnit> for i32 {
    fn partial_cmp(&self, other: &LayoutUnit) -> Option<Ordering> {
        LayoutUnit::from_signed(*self).partial_cmp(other)
    }
}

// C++: FixedPoint three-way comparisons with float and double.
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> PartialEq<f32>
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    fn eq(&self, other: &f32) -> bool {
        self.ToFloat() == *other
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> PartialOrd<f32>
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.ToFloat().partial_cmp(other)
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> PartialEq<f64>
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    fn eq(&self, other: &f64) -> bool {
        self.ToDouble() == *other
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> PartialOrd<f64>
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.ToDouble().partial_cmp(other)
    }
}

// C++: BoundedMultiply and operator*(FixedPoint, FixedPoint), i32 storage.
pub fn BoundedMultiply<const FRACTIONAL_BITS: u32>(
    a: FixedPoint<FRACTIONAL_BITS, i32>,
    b: FixedPoint<FRACTIONAL_BITS, i32>,
) -> FixedPoint<FRACTIONAL_BITS, i32> {
    let mut result = (a.RawValue() as i64 * b.RawValue() as i64)
        / FixedPoint::<FRACTIONAL_BITS, i32>::kFixedPointDenominator as i64;
    let high = (result >> 32) as i32;
    let low = result as i32;
    let saturated = (((a.RawValue() ^ b.RawValue()) as u32) >> 31)
        .wrapping_add(FixedPoint::<FRACTIONAL_BITS, i32>::kRawValueMax as u32);
    if high != low >> 31 {
        result = saturated as i32 as i64;
    }
    FixedPoint::<FRACTIONAL_BITS, i32>::FromRawValue(result as i32)
}

impl<const FRACTIONAL_BITS: u32> Mul for FixedPoint<FRACTIONAL_BITS, i32> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        BoundedMultiply(self, rhs)
    }
}

// C++: operator*(FixedPoint, integral).
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> Mul<i32>
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::FromRawValue(self.RawValue().saturating_mul_i32(rhs))
    }
}

// C++: LayoutUnit multiplication with float/double, both operand orders.
impl Mul<f32> for LayoutUnit {
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        self.ToFloat() * rhs
    }
}

impl Mul<f64> for LayoutUnit {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        self.ToDouble() * rhs
    }
}

impl Mul<LayoutUnit> for i32 {
    type Output = LayoutUnit;

    fn mul(self, rhs: LayoutUnit) -> Self::Output {
        rhs * self
    }
}

impl Mul<LayoutUnit> for f32 {
    type Output = f32;

    fn mul(self, rhs: LayoutUnit) -> Self::Output {
        self * rhs.ToFloat()
    }
}

impl Mul<LayoutUnit> for f64 {
    type Output = f64;

    fn mul(self, rhs: LayoutUnit) -> Self::Output {
        self * rhs.ToDouble()
    }
}

// C++: operator/(FixedPoint, FixedPoint), i32 storage.
impl<const FRACTIONAL_BITS: u32> Div for FixedPoint<FRACTIONAL_BITS, i32> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let raw_value =
            Self::kFixedPointDenominator * self.RawValue() as i128 / rhs.RawValue() as i128;
        Self::FromRawValueWithClamp(raw_value)
    }
}

// C++: operator/(FixedPoint, integral).
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> Div<i32>
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::FromRawValue(Storage::from_i128_wrapping(
            self.RawValue().to_i128() / rhs as i128,
        ))
    }
}

// C++: LayoutUnit division with float/double and reverse operand orders.
impl Div<f32> for LayoutUnit {
    type Output = f32;

    fn div(self, rhs: f32) -> Self::Output {
        self.ToFloat() / rhs
    }
}

impl Div<f64> for LayoutUnit {
    type Output = f64;

    fn div(self, rhs: f64) -> Self::Output {
        self.ToDouble() / rhs
    }
}

impl Div<LayoutUnit> for f32 {
    type Output = f32;

    fn div(self, rhs: LayoutUnit) -> Self::Output {
        self / rhs.ToFloat()
    }
}

impl Div<LayoutUnit> for f64 {
    type Output = f64;

    fn div(self, rhs: LayoutUnit) -> Self::Output {
        self / rhs.ToDouble()
    }
}

impl Div<LayoutUnit> for i32 {
    type Output = LayoutUnit;

    fn div(self, rhs: LayoutUnit) -> Self::Output {
        LayoutUnit::from_signed(self) / rhs
    }
}

// C++: operator%(FixedPoint, FixedPoint), i32 storage.
impl<const FRACTIONAL_BITS: u32> Rem for FixedPoint<FRACTIONAL_BITS, i32> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::FromRawValueWithClamp(self.RawValue() as i128 % rhs.RawValue() as i128)
    }
}

// C++: operator+ and operator+= for matching precision and compatible storage.
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage, SourceStorage: FixedStorage>
    Add<FixedPoint<FRACTIONAL_BITS, SourceStorage>> for FixedPoint<FRACTIONAL_BITS, Storage>
{
    type Output = Self;

    fn add(self, rhs: FixedPoint<FRACTIONAL_BITS, SourceStorage>) -> Self::Output {
        debug_assert!(SourceStorage::BITS <= Storage::BITS);
        let rhs = Storage::from_i128_wrapping(rhs.RawValue().to_i128());
        Self::FromRawValue(self.RawValue().saturating_add(rhs))
    }
}

// C++: addition with integral, float and double values.
impl Add<i32> for LayoutUnit {
    type Output = LayoutUnit;

    fn add(self, rhs: i32) -> Self::Output {
        self + LayoutUnit::from_signed(rhs)
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> Add<f32>
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    type Output = f32;

    fn add(self, rhs: f32) -> Self::Output {
        self.ToFloat() + rhs
    }
}

impl Add<f64> for LayoutUnit {
    type Output = f64;

    fn add(self, rhs: f64) -> Self::Output {
        self.ToDouble() + rhs
    }
}

impl Add<LayoutUnit> for i32 {
    type Output = LayoutUnit;

    fn add(self, rhs: LayoutUnit) -> Self::Output {
        LayoutUnit::from_signed(self) + rhs
    }
}

impl Add<LayoutUnit> for f32 {
    type Output = f32;

    fn add(self, rhs: LayoutUnit) -> Self::Output {
        self + rhs.ToFloat()
    }
}

impl Add<LayoutUnit> for f64 {
    type Output = f64;

    fn add(self, rhs: LayoutUnit) -> Self::Output {
        self + rhs.ToDouble()
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage, SourceStorage: FixedStorage>
    AddAssign<FixedPoint<FRACTIONAL_BITS, SourceStorage>> for FixedPoint<FRACTIONAL_BITS, Storage>
{
    fn add_assign(&mut self, rhs: FixedPoint<FRACTIONAL_BITS, SourceStorage>) {
        self.value_ = ((*self) + rhs).RawValue();
    }
}

// C++: operator-(LayoutUnit, LayoutUnit) and operator-=(FixedPoint, FixedPoint).
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> Sub
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::FromRawValue(self.RawValue().saturating_sub(rhs.RawValue()))
    }
}

// C++: subtraction with integral, float and double values.
impl Sub<i32> for LayoutUnit {
    type Output = LayoutUnit;

    fn sub(self, rhs: i32) -> Self::Output {
        self - LayoutUnit::from_signed(rhs)
    }
}

impl Sub<f32> for LayoutUnit {
    type Output = f32;

    fn sub(self, rhs: f32) -> Self::Output {
        self.ToFloat() - rhs
    }
}

impl Sub<f64> for LayoutUnit {
    type Output = f64;

    fn sub(self, rhs: f64) -> Self::Output {
        self.ToDouble() - rhs
    }
}

impl Sub<LayoutUnit> for i32 {
    type Output = LayoutUnit;

    fn sub(self, rhs: LayoutUnit) -> Self::Output {
        LayoutUnit::from_signed(self) - rhs
    }
}

impl Sub<LayoutUnit> for f32 {
    type Output = f32;

    fn sub(self, rhs: LayoutUnit) -> Self::Output {
        self - rhs.ToFloat()
    }
}

impl Sub<LayoutUnit> for f64 {
    type Output = f64;

    fn sub(self, rhs: LayoutUnit) -> Self::Output {
        self - rhs.ToDouble()
    }
}

impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> SubAssign
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    fn sub_assign(&mut self, rhs: Self) {
        self.value_ = ((*self) - rhs).RawValue();
    }
}

// C++: unary operator- uses ClampedNumeric to saturate the minimum value.
impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> Neg
    for FixedPoint<FRACTIONAL_BITS, Storage>
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::FromRawValue(self.RawValue().saturating_neg())
    }
}

// C++: LayoutUnit compound assignment overloads.
impl AddAssign<i32> for LayoutUnit {
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + LayoutUnit::from_signed(rhs);
    }
}

impl AddAssign<f32> for LayoutUnit {
    fn add_assign(&mut self, rhs: f32) {
        *self = LayoutUnit::from_f32(*self + rhs);
    }
}

impl AddAssign<LayoutUnit> for f32 {
    fn add_assign(&mut self, rhs: LayoutUnit) {
        *self += rhs.ToFloat();
    }
}

impl SubAssign<i32> for LayoutUnit {
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - LayoutUnit::from_signed(rhs);
    }
}

impl SubAssign<f32> for LayoutUnit {
    fn sub_assign(&mut self, rhs: f32) {
        *self = LayoutUnit::from_f32(*self - rhs);
    }
}

impl SubAssign<LayoutUnit> for f32 {
    fn sub_assign(&mut self, rhs: LayoutUnit) {
        *self -= rhs.ToFloat();
    }
}

impl MulAssign<LayoutUnit> for LayoutUnit {
    fn mul_assign(&mut self, rhs: LayoutUnit) {
        *self = *self * rhs;
    }
}

impl MulAssign<f32> for LayoutUnit {
    fn mul_assign(&mut self, rhs: f32) {
        *self = LayoutUnit::from_f32(*self * rhs);
    }
}

impl MulAssign<LayoutUnit> for f32 {
    fn mul_assign(&mut self, rhs: LayoutUnit) {
        *self *= rhs.ToFloat();
    }
}

impl DivAssign<LayoutUnit> for LayoutUnit {
    fn div_assign(&mut self, rhs: LayoutUnit) {
        *self = *self / rhs;
    }
}

impl DivAssign<f32> for LayoutUnit {
    fn div_assign(&mut self, rhs: f32) {
        *self = LayoutUnit::from_f32(*self / rhs);
    }
}

impl DivAssign<LayoutUnit> for f32 {
    fn div_assign(&mut self, rhs: LayoutUnit) {
        *self /= rhs.ToFloat();
    }
}

// C++: IntMod, SnapSizeToPixel and the rounding helpers.
pub fn IntMod(a: LayoutUnit, b: LayoutUnit) -> LayoutUnit {
    LayoutUnit::FromRawValue(a.RawValue() % b.RawValue())
}

pub fn SnapSizeToPixel(size: LayoutUnit, location: LayoutUnit) -> i32 {
    let fraction = location.Fraction();
    let result = (fraction + size).Round() - fraction.Round();
    if result == 0 && (size.RawValue() > 4 || size.RawValue() < -4) {
        return if size.RawValue() > 0 { 1 } else { -1 };
    }
    result
}

pub fn SnapSizeToPixelAllowingZero(size: LayoutUnit, location: LayoutUnit) -> i32 {
    let fraction = location.Fraction();
    (fraction + size).Round() - fraction.Round()
}

pub fn RoundToInt(value: LayoutUnit) -> i32 {
    value.Round()
}

pub fn FloorToInt(value: LayoutUnit) -> i32 {
    value.Floor()
}

pub fn CeilToInt(value: LayoutUnit) -> i32 {
    value.Ceil()
}

pub fn AbsoluteValue(value: LayoutUnit) -> LayoutUnit {
    value.Abs()
}

// C++ non-ARM-GCC branch; Rust does not use that GCC inline-assembly branch.
pub fn GetMaxSaturatedSetResultForTesting() -> i32 {
    LayoutUnit::kRawValueMax as i32
}

pub fn GetMinSaturatedSetResultForTesting() -> i32 {
    LayoutUnit::kRawValueMin as i32
}

// C++ std::integral auto overloads other than int. Rust requires an impl per
// primitive type; each expansion is the same source template body.
macro_rules! impl_integral_ops {
    ($integer:ty, $to_layout:expr) => {
        impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> Mul<$integer>
            for FixedPoint<FRACTIONAL_BITS, Storage>
        {
            type Output = Self;

            fn mul(self, rhs: $integer) -> Self::Output {
                Self::FromRawValueWithClamp(self.RawValue().to_i128() * rhs as i128)
            }
        }

        impl<const FRACTIONAL_BITS: u32, Storage: FixedStorage> Div<$integer>
            for FixedPoint<FRACTIONAL_BITS, Storage>
        {
            type Output = Self;

            fn div(self, rhs: $integer) -> Self::Output {
                Self::FromRawValue(Storage::from_i128_wrapping(
                    self.RawValue().to_i128() / rhs as i128,
                ))
            }
        }

        impl Add<$integer> for LayoutUnit {
            type Output = LayoutUnit;

            fn add(self, rhs: $integer) -> Self::Output {
                self + ($to_layout)(rhs)
            }
        }

        impl Add<LayoutUnit> for $integer {
            type Output = LayoutUnit;

            fn add(self, rhs: LayoutUnit) -> Self::Output {
                ($to_layout)(self) + rhs
            }
        }

        impl Sub<$integer> for LayoutUnit {
            type Output = LayoutUnit;

            fn sub(self, rhs: $integer) -> Self::Output {
                self - ($to_layout)(rhs)
            }
        }

        impl Sub<LayoutUnit> for $integer {
            type Output = LayoutUnit;

            fn sub(self, rhs: LayoutUnit) -> Self::Output {
                ($to_layout)(self) - rhs
            }
        }

        impl Mul<LayoutUnit> for $integer {
            type Output = LayoutUnit;

            fn mul(self, rhs: LayoutUnit) -> Self::Output {
                rhs * self
            }
        }

        impl Div<LayoutUnit> for $integer {
            type Output = LayoutUnit;

            fn div(self, rhs: LayoutUnit) -> Self::Output {
                ($to_layout)(self) / rhs
            }
        }

        impl AddAssign<$integer> for LayoutUnit {
            fn add_assign(&mut self, rhs: $integer) {
                *self = *self + rhs;
            }
        }

        impl SubAssign<$integer> for LayoutUnit {
            fn sub_assign(&mut self, rhs: $integer) {
                *self = *self - rhs;
            }
        }
    };
}

impl_integral_ops!(i8, |value: i8| LayoutUnit::from_signed(value as i32));
impl_integral_ops!(i16, |value: i16| LayoutUnit::from_signed(value as i32));
impl_integral_ops!(i64, |value: i64| LayoutUnit::from_wide(value));
impl_integral_ops!(isize, |value: isize| LayoutUnit::FromRawValueWithClamp(
    value as i128 * LayoutUnit::kFixedPointDenominator
));
impl_integral_ops!(u8, |value: u8| LayoutUnit::from_unsigned(value as u32));
impl_integral_ops!(u16, |value: u16| LayoutUnit::from_unsigned(value as u32));
impl_integral_ops!(u32, |value: u32| LayoutUnit::from_unsigned(value));
impl_integral_ops!(u64, |value: u64| LayoutUnit::FromRawValueWithClamp(
    value as i128 * LayoutUnit::kFixedPointDenominator
));
impl_integral_ops!(usize, |value: usize| LayoutUnit::FromRawValueWithClamp(
    value as i128 * LayoutUnit::kFixedPointDenominator
));

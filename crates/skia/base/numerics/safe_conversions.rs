// Source: base/numerics/safe_conversions.h. C++ implicit conversions are explicit
// Rust functions; only statically contained pairs implement StrictCast.
#![allow(non_snake_case)]
pub use super::safe_conversions_impl::*;
use core::ops::Not;
pub fn IsValueInRangeForNumericType<D: Numeric, S: UncheckedInput>(v: S) -> bool
where
    S::Value: Cast<D>,
{
    v.value().range().IsValid()
}
pub fn checked_cast<D: Numeric, S: UncheckedInput>(v: S) -> D
where
    S::Value: Cast<D>,
{
    checked_cast_with::<D, CheckOnFailure, S>(v)
}
pub fn checked_cast_with<D: Numeric, H: FailureHandler, S: UncheckedInput>(v: S) -> D
where
    S::Value: Cast<D>,
{
    let n = v.value();
    if n.range().IsValid() {
        n.cast()
    } else {
        H::HandleFailure()
    }
}
pub fn saturated_cast<D: Numeric, S: UncheckedInput>(v: S) -> D
where
    S::Value: Cast<D>,
{
    v.value().default_saturate()
}

pub fn strict_cast<D: Numeric, S: UncheckedInput>(v: S) -> D
where
    S::Value: StrictCast<D>,
{
    v.value().cast()
}
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct StrictNumeric<T: Numeric> {
    pub(crate) value_: T,
}
impl<T: Numeric> StrictNumeric<T> {
    pub const fn zero() -> Self {
        Self { value_: T::ZERO }
    }
    pub fn new<S: UncheckedInput>(v: S) -> Self
    where
        S::Value: StrictCast<T>,
    {
        Self {
            value_: strict_cast(v),
        }
    }
    pub fn Cast<D: Numeric>(self) -> D
    where
        T: StrictCast<D>,
    {
        strict_cast(self.value_)
    }
    pub const fn RawValue(self) -> T {
        self.value_
    }
}
impl<T: Numeric> Default for StrictNumeric<T> {
    fn default() -> Self {
        Self::zero()
    }
}
impl<T: Numeric> Input for StrictNumeric<T> {
    type Value = T;
    fn value(self) -> T {
        self.value_
    }
}
impl<T: Numeric> Not for StrictNumeric<T> {
    type Output = bool;
    fn not(self) -> bool {
        self.value_ == T::ZERO
    }
}
pub fn MakeStrictNum<S: UncheckedInput>(v: S) -> StrictNumeric<S::Value> {
    StrictNumeric { value_: v.value() }
}
pub type SizeT = StrictNumeric<usize>;
pub trait FloatRound: Numeric {
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
}
impl FloatRound for f32 {
    fn floor(self) -> Self {
        self.floor()
    }
    fn ceil(self) -> Self {
        self.ceil()
    }
    fn round(self) -> Self {
        self.round()
    }
}
impl FloatRound for f64 {
    fn floor(self) -> Self {
        self.floor()
    }
    fn ceil(self) -> Self {
        self.ceil()
    }
    fn round(self) -> Self {
        self.round()
    }
}
pub fn ClampFloor<D: Numeric + Integer, S: FloatRound + Cast<D>>(v: S) -> D {
    saturated_cast(v.floor())
}
pub fn ClampCeil<D: Numeric + Integer, S: FloatRound + Cast<D>>(v: S) -> D {
    saturated_cast(v.ceil())
}
pub fn ClampRound<D: Numeric + Integer, S: FloatRound + Cast<D>>(v: S) -> D {
    saturated_cast(v.round())
}
// ARMEL ssat/usat selected path remains an actual child-producer dependency.
// Custom SaturationHandler follows the native distinct branch below.
// Source: safe_conversions.h:306-318 with safe_conversions_impl.h:697-707.
impl<T: Numeric, R: StrictInput> PartialEq<R> for StrictNumeric<T>
where
    T: SafeComparison<R::Value>,
{
    fn eq(&self, r: &R) -> bool {
        self.value_.safe_eq(r.value())
    }
    fn ne(&self, r: &R) -> bool {
        self.value_.safe_ne(r.value())
    }
}
impl<T: Numeric, R: StrictInput> PartialOrd<R> for StrictNumeric<T>
where
    T: SafeComparison<R::Value>,
{
    fn partial_cmp(&self, r: &R) -> Option<core::cmp::Ordering> {
        let v = r.value();
        if self.value_.safe_lt(v) {
            Some(core::cmp::Ordering::Less)
        } else if self.value_.safe_eq(v) {
            Some(core::cmp::Ordering::Equal)
        } else if self.value_.safe_gt(v) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &R) -> bool {
        self.value_.safe_lt(r.value())
    }
    fn le(&self, r: &R) -> bool {
        self.value_.safe_le(r.value())
    }
    fn gt(&self, r: &R) -> bool {
        self.value_.safe_gt(r.value())
    }
    fn ge(&self, r: &R) -> bool {
        self.value_.safe_ge(r.value())
    }
}
// Source: safe_conversions.h:114-157,206-220. Custom handler has its true bounds
// and all three result branches; no default handler substituted for it.
pub trait SaturationHandler<T: Numeric> {
    fn max() -> T;
    fn lowest() -> T;
    fn NaN() -> T;
    fn Overflow() -> T;
    fn Underflow() -> T;
}
pub struct SaturationDefaultLimits;
impl<T: Numeric> SaturationHandler<T> for SaturationDefaultLimits {
    fn max() -> T {
        T::MAX
    }
    fn lowest() -> T {
        T::LOWEST
    }
    fn NaN() -> T {
        T::nan()
    }
    fn Overflow() -> T {
        T::overflow()
    }
    fn Underflow() -> T {
        T::underflow()
    }
}
pub fn saturated_cast_with<D: Numeric, H: SaturationHandler<D>, S: UncheckedInput>(v: S) -> D
where
    S::Value: RangeWith<D>,
{
    let n = v.value();
    let c = n.range_with(H::lowest(), H::max());
    if !c.IsOverflowFlagSet() {
        if !c.IsUnderflowFlagSet() {
            n.cast()
        } else {
            H::Underflow()
        }
    } else if !S::Value::FLOAT || !c.IsUnderflowFlagSet() {
        H::Overflow()
    } else {
        H::NaN()
    }
}

// Source: native IsStrictOp/IsClampedOp overload constraints, no Checked->raw cast.
pub trait StrictInput: Input {}
impl<T: Numeric> StrictInput for T {}
impl<T: Numeric> StrictInput for StrictNumeric<T> {}
pub trait UncheckedInput: Input {}
impl<T: Numeric> UncheckedInput for T {}
impl<T: Numeric> UncheckedInput for StrictNumeric<T> {}

// Source: safe_conversions.h:306-318 / clamped_math.h:229-234, primitive-left comparisons.
impl<T: Numeric> PartialEq<StrictNumeric<T>> for i8
where
    i8: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for i8
where
    i8: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for u8
where
    u8: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for u8
where
    u8: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for i16
where
    i16: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for i16
where
    i16: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for u16
where
    u16: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for u16
where
    u16: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for i32
where
    i32: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for i32
where
    i32: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for u32
where
    u32: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for u32
where
    u32: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for i64
where
    i64: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for i64
where
    i64: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for u64
where
    u64: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for u64
where
    u64: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for isize
where
    isize: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for isize
where
    isize: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for usize
where
    usize: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for usize
where
    usize: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for f32
where
    f32: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for f32
where
    f32: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for f64
where
    f64: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for f64
where
    f64: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<StrictNumeric<T>> for bool
where
    bool: SafeComparison<T>,
{
    fn eq(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<StrictNumeric<T>> for bool
where
    bool: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &StrictNumeric<T>) -> Option<core::cmp::Ordering> {
        if self.safe_lt(r.value_) {
            Some(core::cmp::Ordering::Less)
        } else if self.safe_eq(r.value_) {
            Some(core::cmp::Ordering::Equal)
        } else if self.safe_gt(r.value_) {
            Some(core::cmp::Ordering::Greater)
        } else {
            None
        }
    }
    fn lt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &StrictNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}

// Source: safe_conversions.h:80-108,148-157,206-236. Explicit CTFE-only
// primitive specializations: no runtime trait call is mislabeled const.
#[macro_export]
macro_rules! NativeCheckedCastConst {
    (i8,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (u8,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i8),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i8),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i8)
            }
        }
    }};
    (i8,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u8)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u8)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (i16,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (value <= 1 as u8),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(
                    (true) && (value <= 1 as u8),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u8)
            }
        }
    }};
    (i8,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= (i8::MIN as i16)) && (value <= (i8::MAX as i16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= (i8::MIN as i16)) && (value <= (i8::MAX as i16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as i16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as i16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (u16,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i16),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i16),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i16)
            }
        }
    }};
    (i8,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= (u8::MAX as u16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= (u8::MAX as u16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= (i16::MAX as u16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= (i16::MAX as u16)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (i32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= 1 as u16),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(
                    (true) && (value <= 1 as u16),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u16)
            }
        }
    }};
    (i8,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= (i8::MIN as i32)) && (value <= (i8::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= (i8::MIN as i32)) && (value <= (i8::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= (i16::MIN as i32)) && (value <= (i16::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= (i16::MIN as i32)) && (value <= (i16::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (value <= (u16::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (value <= (u16::MAX as i32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (u32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i32),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i32),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i32)
            }
        }
    }};
    (i8,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (u8::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (u8::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (i16::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (i16::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (u16::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (u16::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (i32::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (i32::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (i64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= (isize::MAX as u32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= 1 as u32),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(
                    (true) && (value <= 1 as u32),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u32)
            }
        }
    }};
    (i8,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= (i8::MIN as i64)) && (value <= (i8::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= (i8::MIN as i64)) && (value <= (i8::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= (i16::MIN as i64)) && (value <= (i16::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= (i16::MIN as i64)) && (value <= (i16::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= (u16::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= (u16::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= (i32::MIN as i64)) && (value <= (i32::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= (i32::MIN as i64)) && (value <= (i32::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= (u32::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= (u32::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (u64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= (isize::MIN as i64)) && (value <= (isize::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= (usize::MAX as i64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i64),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(
                    (value >= 0) && (value <= 1 as i64),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as i64)
            }
        }
    }};
    (i8,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i8::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (u8::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (u8::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i16::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i16::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (u16::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (u16::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i32::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i32::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (u32::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (u32::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i64::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (i64::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (isize,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (isize::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (isize::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= (usize::MAX as u64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= 1 as u64),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(
                    (true) && (value <= 1 as u64),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as u64)
            }
        }
    }};
    (i8,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= (i8::MIN as isize)) && (value <= (i8::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= (i8::MIN as isize)) && (value <= (i8::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (value <= (u8::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= (i16::MIN as isize)) && (value <= (i16::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= (i16::MIN as isize)) && (value <= (i16::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (value <= (u16::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (value <= (u16::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= (i32::MIN as isize)) && (value <= (i32::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (value <= (u32::MAX as isize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (usize,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (value <= 1 as isize),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(
                    (value >= 0) && (value <= 1 as isize),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as isize)
            }
        }
    }};
    (i8,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (i8::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (i8::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (u8::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (u8::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (i16::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (i16::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (u16::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (u16::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (i32::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (i32::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (u32::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (i64::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (isize::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= (isize::MAX as usize)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (f32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= 1 as usize),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(
                    (true) && (value <= 1 as usize),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as usize)
            }
        }
    }};
    (i8,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i8::MIN as f32)) && (value <= (i8::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i8::MIN as f32)) && (value <= (i8::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= (u8::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= (u8::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i16::MIN as f32)) && (value <= (i16::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i16::MIN as f32)) && (value <= (i16::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= (u16::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= (u16::MAX as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i32::MIN as f32))
                        && (value <= ((i32::MAX & !(((1 as i32) << 7) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i32::MIN as f32))
                        && (value <= ((i32::MAX & !(((1 as i32) << 7) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= ((u32::MAX & !(((1 as u32) << 8) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= ((u32::MAX & !(((1 as u32) << 8) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i64::MIN as f32))
                        && (value <= ((i64::MAX & !(((1 as i64) << 39) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (i64::MIN as f32))
                        && (value <= ((i64::MAX & !(((1 as i64) << 39) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= ((u64::MAX & !(((1 as u64) << 40) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= ((u64::MAX & !(((1 as u64) << 40) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (isize::MIN as f32))
                        && (value <= ((isize::MAX & !(((1 as isize) << 7) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value >= (isize::MIN as f32))
                        && (value <= ((isize::MAX & !(((1 as isize) << 39) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= ((usize::MAX & !(((1 as usize) << 8) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0)
                        && (value <= ((usize::MAX & !(((1 as usize) << 40) - 1)) as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (f64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f64)
            }
        }
    }};
    (bool,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= (1 as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(
                    (value > -1.0) && (value <= (1 as f32)),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as f32)
            }
        }
    }};
    (i8,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i8::MIN as f64)) && (value <= (i8::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i8::MIN as f64)) && (value <= (i8::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i8)
            }
        }
    }};
    (u8,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (u8::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (u8::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8)
            }
        }
    }};
    (i16,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i16::MIN as f64)) && (value <= (i16::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i16::MIN as f64)) && (value <= (i16::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i16)
            }
        }
    }};
    (u16,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (u16::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (u16::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u16)
            }
        }
    }};
    (i32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i32::MIN as f64)) && (value <= (i32::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i32::MIN as f64)) && (value <= (i32::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i32)
            }
        }
    }};
    (u32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (u32::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (u32::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u32)
            }
        }
    }};
    (i64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i64::MIN as f64))
                        && (value <= ((i64::MAX & !(((1 as i64) << 10) - 1)) as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (i64::MIN as f64))
                        && (value <= ((i64::MAX & !(((1 as i64) << 10) - 1)) as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as i64)
            }
        }
    }};
    (u64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= ((u64::MAX & !(((1 as u64) << 11) - 1)) as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= ((u64::MAX & !(((1 as u64) << 11) - 1)) as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u64)
            }
        }
    }};
    (isize,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (isize::MIN as f64)) && (value <= (isize::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (isize::MIN as f64))
                        && (value <= ((isize::MAX & !(((1 as isize) << 10) - 1)) as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as isize)
            }
        }
    }};
    (usize,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (usize::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0)
                        && (value <= ((usize::MAX & !(((1 as usize) << 11) - 1)) as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as usize)
            }
        }
    }};
    (f32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (f32::MIN as f64)) && (value <= (f32::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value >= (f32::MIN as f64)) && (value <= (f32::MAX as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value as f32)
            }
        }
    }};
    (f64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
    (bool,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (1 as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(
                    (value > -1.0) && (value <= (1 as f64)),
                    "native checked_cast invalid constant evaluation"
                );
                (value != 0 as f64)
            }
        }
    }};
    (i8,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i8)
            }
        }
    }};
    (u8,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u8)
            }
        }
    }};
    (i16,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i16)
            }
        }
    }};
    (u16,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u16)
            }
        }
    }};
    (i32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i32)
            }
        }
    }};
    (u32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u32)
            }
        }
    }};
    (i64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as i64)
            }
        }
    }};
    (u64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as u64)
            }
        }
    }};
    (isize,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as isize)
            }
        }
    }};
    (usize,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as usize)
            }
        }
    }};
    (f32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as f32)
            }
        }
    }};
    (f64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                (value as u8 as f64)
            }
        }
    }};
    (bool,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(
                    (true) && (true),
                    "native checked_cast invalid constant evaluation"
                );
                value
            }
        }
    }};
}
#[macro_export]
macro_rules! NativeSaturatedCastConst {
    (i8,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i8;
                if upper {
                    if lower {
                        (value != 0 as i8)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i8;
                if upper {
                    if lower {
                        (value != 0 as i8)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u8);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u8);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = value <= 1 as u8;
                if upper {
                    if lower {
                        (value != 0 as u8)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                let lower = true;
                let upper = value <= 1 as u8;
                if upper {
                    if lower {
                        (value != 0 as u8)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= (i8::MIN as i16);
                let upper = value <= (i8::MAX as i16);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= (i8::MIN as i16);
                let upper = value <= (i8::MAX as i16);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as i16);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as i16);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i16;
                if upper {
                    if lower {
                        (value != 0 as i16)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i16;
                if upper {
                    if lower {
                        (value != 0 as i16)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u16);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u16);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= (u8::MAX as u16);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= (u8::MAX as u16);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= (i16::MAX as u16);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= (i16::MAX as u16);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= 1 as u16;
                if upper {
                    if lower {
                        (value != 0 as u16)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                let lower = true;
                let upper = value <= 1 as u16;
                if upper {
                    if lower {
                        (value != 0 as u16)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= (i8::MIN as i32);
                let upper = value <= (i8::MAX as i32);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= (i8::MIN as i32);
                let upper = value <= (i8::MAX as i32);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as i32);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as i32);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= (i16::MIN as i32);
                let upper = value <= (i16::MAX as i32);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= (i16::MIN as i32);
                let upper = value <= (i16::MAX as i32);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = value <= (u16::MAX as i32);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = value <= (u16::MAX as i32);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i32;
                if upper {
                    if lower {
                        (value != 0 as i32)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i32;
                if upper {
                    if lower {
                        (value != 0 as i32)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u32);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u32);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (u8::MAX as u32);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (u8::MAX as u32);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (i16::MAX as u32);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (i16::MAX as u32);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (u16::MAX as u32);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (u16::MAX as u32);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (i32::MAX as u32);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (i32::MAX as u32);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= (isize::MAX as u32);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= 1 as u32;
                if upper {
                    if lower {
                        (value != 0 as u32)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                let lower = true;
                let upper = value <= 1 as u32;
                if upper {
                    if lower {
                        (value != 0 as u32)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= (i8::MIN as i64);
                let upper = value <= (i8::MAX as i64);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= (i8::MIN as i64);
                let upper = value <= (i8::MAX as i64);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as i64);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as i64);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= (i16::MIN as i64);
                let upper = value <= (i16::MAX as i64);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= (i16::MIN as i64);
                let upper = value <= (i16::MAX as i64);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= (u16::MAX as i64);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= (u16::MAX as i64);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= (i32::MIN as i64);
                let upper = value <= (i32::MAX as i64);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= (i32::MIN as i64);
                let upper = value <= (i32::MAX as i64);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= (u32::MAX as i64);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= (u32::MAX as i64);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= (isize::MIN as i64);
                let upper = value <= (isize::MAX as i64);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= (usize::MAX as i64);
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i64;
                if upper {
                    if lower {
                        (value != 0 as i64)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                let lower = value >= 0;
                let upper = value <= 1 as i64;
                if upper {
                    if lower {
                        (value != 0 as i64)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u64);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i8::MAX as u64);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (u8::MAX as u64);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (u8::MAX as u64);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i16::MAX as u64);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i16::MAX as u64);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (u16::MAX as u64);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (u16::MAX as u64);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i32::MAX as u64);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i32::MAX as u64);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (u32::MAX as u64);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (u32::MAX as u64);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i64::MAX as u64);
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (i64::MAX as u64);
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (isize::MAX as u64);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (isize::MAX as u64);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= (usize::MAX as u64);
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= 1 as u64;
                if upper {
                    if lower {
                        (value != 0 as u64)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                let lower = true;
                let upper = value <= 1 as u64;
                if upper {
                    if lower {
                        (value != 0 as u64)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= (i8::MIN as isize);
                let upper = value <= (i8::MAX as isize);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= (i8::MIN as isize);
                let upper = value <= (i8::MAX as isize);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as isize);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = value <= (u8::MAX as isize);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= (i16::MIN as isize);
                let upper = value <= (i16::MAX as isize);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= (i16::MIN as isize);
                let upper = value <= (i16::MAX as isize);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = value <= (u16::MAX as isize);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = value <= (u16::MAX as isize);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= (i32::MIN as isize);
                let upper = value <= (i32::MAX as isize);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = value <= (u32::MAX as isize);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = true;
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = value <= 1 as isize;
                if upper {
                    if lower {
                        (value != 0 as isize)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                let lower = value >= 0;
                let upper = value <= 1 as isize;
                if upper {
                    if lower {
                        (value != 0 as isize)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (i8::MAX as usize);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (i8::MAX as usize);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (u8::MAX as usize);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (u8::MAX as usize);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (i16::MAX as usize);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (i16::MAX as usize);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (u16::MAX as usize);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (u16::MAX as usize);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (i32::MAX as usize);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (i32::MAX as usize);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (u32::MAX as usize);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (i64::MAX as usize);
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (isize::MAX as usize);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= (isize::MAX as usize);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= 1 as usize;
                if upper {
                    if lower {
                        (value != 0 as usize)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                let lower = true;
                let upper = value <= 1 as usize;
                if upper {
                    if lower {
                        (value != 0 as usize)
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i8::MIN as f32);
                let upper = value <= (i8::MAX as f32);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if false || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i8::MIN as f32);
                let upper = value <= (i8::MAX as f32);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if false || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= (u8::MAX as f32);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if false || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= (u8::MAX as f32);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if false || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i16::MIN as f32);
                let upper = value <= (i16::MAX as f32);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if false || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i16::MIN as f32);
                let upper = value <= (i16::MAX as f32);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if false || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= (u16::MAX as f32);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if false || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= (u16::MAX as f32);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if false || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i32::MIN as f32);
                let upper = value <= ((i32::MAX & !(((1 as i32) << 7) - 1)) as f32);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if false || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i32::MIN as f32);
                let upper = value <= ((i32::MAX & !(((1 as i32) << 7) - 1)) as f32);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if false || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= ((u32::MAX & !(((1 as u32) << 8) - 1)) as f32);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if false || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= ((u32::MAX & !(((1 as u32) << 8) - 1)) as f32);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if false || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i64::MIN as f32);
                let upper = value <= ((i64::MAX & !(((1 as i64) << 39) - 1)) as f32);
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if false || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (i64::MIN as f32);
                let upper = value <= ((i64::MAX & !(((1 as i64) << 39) - 1)) as f32);
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if false || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= ((u64::MAX & !(((1 as u64) << 40) - 1)) as f32);
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if false || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= ((u64::MAX & !(((1 as u64) << 40) - 1)) as f32);
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if false || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (isize::MIN as f32);
                let upper = value <= ((isize::MAX & !(((1 as isize) << 7) - 1)) as f32);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if false || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value >= (isize::MIN as f32);
                let upper = value <= ((isize::MAX & !(((1 as isize) << 39) - 1)) as f32);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if false || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= ((usize::MAX & !(((1 as usize) << 8) - 1)) as f32);
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if false || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= ((usize::MAX & !(((1 as usize) << 40) - 1)) as f32);
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if false || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if false || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if false || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if false || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if false || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= (1 as f32);
                if upper {
                    if lower {
                        (value != 0 as f32)
                    } else {
                        false
                    }
                } else if false || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                let lower = value > -1.0;
                let upper = value <= (1 as f32);
                if upper {
                    if lower {
                        (value != 0 as f32)
                    } else {
                        false
                    }
                } else if false || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i8::MIN as f64);
                let upper = value <= (i8::MAX as f64);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if false || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i8::MIN as f64);
                let upper = value <= (i8::MAX as f64);
                if upper {
                    if lower {
                        (value as i8)
                    } else {
                        i8::MIN
                    }
                } else if false || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (u8::MAX as f64);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if false || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (u8::MAX as f64);
                if upper {
                    if lower {
                        (value as u8)
                    } else {
                        u8::MIN
                    }
                } else if false || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i16::MIN as f64);
                let upper = value <= (i16::MAX as f64);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if false || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i16::MIN as f64);
                let upper = value <= (i16::MAX as f64);
                if upper {
                    if lower {
                        (value as i16)
                    } else {
                        i16::MIN
                    }
                } else if false || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (u16::MAX as f64);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if false || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (u16::MAX as f64);
                if upper {
                    if lower {
                        (value as u16)
                    } else {
                        u16::MIN
                    }
                } else if false || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i32::MIN as f64);
                let upper = value <= (i32::MAX as f64);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if false || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i32::MIN as f64);
                let upper = value <= (i32::MAX as f64);
                if upper {
                    if lower {
                        (value as i32)
                    } else {
                        i32::MIN
                    }
                } else if false || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (u32::MAX as f64);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if false || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (u32::MAX as f64);
                if upper {
                    if lower {
                        (value as u32)
                    } else {
                        u32::MIN
                    }
                } else if false || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i64::MIN as f64);
                let upper = value <= ((i64::MAX & !(((1 as i64) << 10) - 1)) as f64);
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if false || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (i64::MIN as f64);
                let upper = value <= ((i64::MAX & !(((1 as i64) << 10) - 1)) as f64);
                if upper {
                    if lower {
                        (value as i64)
                    } else {
                        i64::MIN
                    }
                } else if false || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= ((u64::MAX & !(((1 as u64) << 11) - 1)) as f64);
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if false || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= ((u64::MAX & !(((1 as u64) << 11) - 1)) as f64);
                if upper {
                    if lower {
                        (value as u64)
                    } else {
                        u64::MIN
                    }
                } else if false || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (isize::MIN as f64);
                let upper = value <= (isize::MAX as f64);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if false || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (isize::MIN as f64);
                let upper = value <= ((isize::MAX & !(((1 as isize) << 10) - 1)) as f64);
                if upper {
                    if lower {
                        (value as isize)
                    } else {
                        isize::MIN
                    }
                } else if false || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (usize::MAX as f64);
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if false || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= ((usize::MAX & !(((1 as usize) << 11) - 1)) as f64);
                if upper {
                    if lower {
                        (value as usize)
                    } else {
                        usize::MIN
                    }
                } else if false || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (f32::MIN as f64);
                let upper = value <= (f32::MAX as f64);
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if false || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value >= (f32::MIN as f64);
                let upper = value <= (f32::MAX as f64);
                if upper {
                    if lower {
                        (value as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if false || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if false || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if false || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (1 as f64);
                if upper {
                    if lower {
                        (value != 0 as f64)
                    } else {
                        false
                    }
                } else if false || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                let lower = value > -1.0;
                let upper = value <= (1 as f64);
                if upper {
                    if lower {
                        (value != 0 as f64)
                    } else {
                        false
                    }
                } else if false || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
    (i8,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i8)
                    } else {
                        i8::MIN
                    }
                } else if true || lower {
                    i8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u8,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u8)
                    } else {
                        u8::MIN
                    }
                } else if true || lower {
                    u8::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i16,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i16)
                    } else {
                        i16::MIN
                    }
                } else if true || lower {
                    i16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u16,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u16)
                    } else {
                        u16::MIN
                    }
                } else if true || lower {
                    u16::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i32)
                    } else {
                        i32::MIN
                    }
                } else if true || lower {
                    i32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u32)
                    } else {
                        u32::MIN
                    }
                } else if true || lower {
                    u32::MAX
                } else {
                    0
                }
            }
        }
    }};
    (i64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as i64)
                    } else {
                        i64::MIN
                    }
                } else if true || lower {
                    i64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (u64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as u64)
                    } else {
                        u64::MIN
                    }
                } else if true || lower {
                    u64::MAX
                } else {
                    0
                }
            }
        }
    }};
    (isize,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as isize)
                    } else {
                        isize::MIN
                    }
                } else if true || lower {
                    isize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (usize,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as usize)
                    } else {
                        usize::MIN
                    }
                } else if true || lower {
                    usize::MAX
                } else {
                    0
                }
            }
        }
    }};
    (f32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as f32)
                    } else {
                        f32::NEG_INFINITY
                    }
                } else if true || lower {
                    f32::INFINITY
                } else {
                    f32::NAN
                }
            }
        }
    }};
    (f64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        (value as u8 as f64)
                    } else {
                        f64::NEG_INFINITY
                    }
                } else if true || lower {
                    f64::INFINITY
                } else {
                    f64::NAN
                }
            }
        }
    }};
    (bool,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                let lower = true;
                let upper = true;
                if upper {
                    if lower {
                        value
                    } else {
                        false
                    }
                } else if true || lower {
                    true
                } else {
                    false
                }
            }
        }
    }};
}
#[macro_export]
macro_rules! NativeStrictCastConst {
    (i8,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (u8,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,i8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i8)
            }
        }
    }};
    (i8,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (i16,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,u8,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u8 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u8)
            }
        }
    }};
    (i8,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (u16,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,i16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i16)
            }
        }
    }};
    (i8,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (i32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,u16,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u16 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u16)
            }
        }
    }};
    (i8,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (u32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,i32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i32)
            }
        }
    }};
    (i8,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (i64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,u32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u32)
            }
        }
    }};
    (i8,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (u64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,i64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: i64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as i64)
            }
        }
    }};
    (i8,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (isize,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,u64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: u64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as u64)
            }
        }
    }};
    (i8,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (usize,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,isize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: isize = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as isize)
            }
        }
    }};
    (i8,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (f32,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,usize,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: usize = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as usize)
            }
        }
    }};
    (i8,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (f64,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(true, "native strict_cast range constraint");
                (value as f64)
            }
        }
    }};
    (bool,f32,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f32 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as f32)
            }
        }
    }};
    (i8,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i8)
            }
        }
    }};
    (u8,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u8)
            }
        }
    }};
    (i16,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i16)
            }
        }
    }};
    (u16,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u16)
            }
        }
    }};
    (i32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i32)
            }
        }
    }};
    (u32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u32)
            }
        }
    }};
    (i64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as i64)
            }
        }
    }};
    (u64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as u64)
            }
        }
    }};
    (isize,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as isize)
            }
        }
    }};
    (usize,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as usize)
            }
        }
    }};
    (f32,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value as f32)
            }
        }
    }};
    (f64,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
    (bool,f64,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: f64 = $value;
                assert!(false, "native strict_cast range constraint");
                (value != 0 as f64)
            }
        }
    }};
    (i8,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i8)
            }
        }
    }};
    (u8,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u8)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u8)
            }
        }
    }};
    (i16,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i16)
            }
        }
    }};
    (u16,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u16)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u16)
            }
        }
    }};
    (i32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i32)
            }
        }
    }};
    (u32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u32)
            }
        }
    }};
    (i64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as i64)
            }
        }
    }};
    (u64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as u64)
            }
        }
    }};
    (isize,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as isize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as isize)
            }
        }
    }};
    (usize,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as usize)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as usize)
            }
        }
    }};
    (f32,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as f32)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as f32)
            }
        }
    }};
    (f64,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as f64)
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                (value as u8 as f64)
            }
        }
    }};
    (bool,bool,$value:expr) => {{
        #[cfg(target_pointer_width = "32")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            const {
                let value: bool = $value;
                assert!(true, "native strict_cast range constraint");
                value
            }
        }
    }};
}

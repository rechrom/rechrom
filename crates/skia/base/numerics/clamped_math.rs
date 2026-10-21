// Source: base/numerics/clamped_math.h:18-237.
#![allow(non_snake_case)]
use super::checked_math_impl::*;
use super::clamped_math_impl::*;
use super::safe_conversions::*;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ClampedNumeric<T: Numeric> {
    value_: T,
}
impl<T: Numeric> ClampedNumeric<T> {
    pub const fn zero() -> Self {
        Self { value_: T::ZERO }
    }
    pub fn new<S: UncheckedInput>(s: S) -> Self
    where
        S::Value: Cast<T>,
    {
        Self {
            value_: saturated_cast(s),
        }
    }
    pub fn Cast<D: Numeric>(self) -> ClampedNumeric<D>
    where
        T: Cast<D>,
    {
        ClampedNumeric::new(self.value_)
    }
    pub fn Into<D: Numeric>(self) -> D
    where
        T: Cast<D>,
    {
        saturated_cast(self.value_)
    }
    pub const fn RawValue(self) -> T {
        self.value_
    }
}
impl<T: Numeric> Default for ClampedNumeric<T> {
    fn default() -> Self {
        Self::zero()
    }
}
impl<T: Numeric> Input for ClampedNumeric<T> {
    type Value = T;
    fn value(self) -> T {
        self.value_
    }
}
macro_rules! op {
    ($trait:ident,$method:ident,$assign:ident,$amethod:ident,$raw:ident) => {
        impl<T: Numeric, R: UncheckedInput> $trait<R> for ClampedNumeric<T>
        where
            T: Promote<R::Value> + ClampedBinary<R::Value, <T as Promote<R::Value>>::Max>,
        {
            type Output = ClampedNumeric<<T as Promote<R::Value>>::Max>;
            fn $method(self, r: R) -> Self::Output {
                ClampedNumeric {
                    value_: self.value_.$raw(r.value()),
                }
            }
        }
        impl<T: Numeric, R: UncheckedInput> $assign<R> for ClampedNumeric<T>
        where
            T: ClampedBinary<R::Value, T>,
        {
            fn $amethod(&mut self, r: R) {
                self.value_ = self.value_.$raw(r.value())
            }
        }
    };
}
op!(Add, add, AddAssign, add_assign, clamp_add);
op!(Sub, sub, SubAssign, sub_assign, clamp_sub);
op!(Mul, mul, MulAssign, mul_assign, clamp_mul);
op!(Div, div, DivAssign, div_assign, clamp_div);
impl<T: Numeric, R: UncheckedInput> Rem<R> for ClampedNumeric<T>
where
    T: Promote<R::Value>
        + CheckedModulo<R::Value, <T as Promote<R::Value>>::Max>
        + Cast<<T as Promote<R::Value>>::Max>,
{
    type Output = ClampedNumeric<<T as Promote<R::Value>>::Max>;
    fn rem(self, r: R) -> Self::Output {
        ClampedNumeric {
            value_: ClampedModOp(self.value_, r.value()),
        }
    }
}
impl<T: Numeric, R: UncheckedInput> RemAssign<R> for ClampedNumeric<T>
where
    T: CheckedModulo<R::Value, T> + Cast<T>,
{
    fn rem_assign(&mut self, r: R) {
        self.value_ = ClampedModOp(self.value_, r.value())
    }
}
impl<T: IntegralOps> Neg for ClampedNumeric<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            value_: self.value_.saturated_neg(),
        }
    }
}
impl<T: IntegralOps> Not for ClampedNumeric<T> {
    type Output = ClampedNumeric<T::Unsigned>;
    fn not(self) -> Self::Output {
        ClampedNumeric {
            value_: self.value_.invert(),
        }
    }
}
impl<T: IntegralOps> ClampedNumeric<T> {
    pub fn Abs(self) -> Self {
        Self {
            value_: self.value_.saturated_abs(),
        }
    }
    pub fn UnsignedAbs(self) -> ClampedNumeric<T::Unsigned> {
        ClampedNumeric {
            value_: self.value_.unsigned_abs(),
        }
    }
}
impl<T: IntegralOps, R: UncheckedInput> Shl<R> for ClampedNumeric<T>
where
    R::Value: UnsignedShiftAmount,
{
    type Output = Self;
    fn shl(self, r: R) -> Self {
        Self {
            value_: self.value_.clamped_lsh(r.value().shift_amount()),
        }
    }
}
impl<T: IntegralOps, R: UncheckedInput> Shr<R> for ClampedNumeric<T>
where
    R::Value: UnsignedShiftAmount,
{
    type Output = Self;
    fn shr(self, r: R) -> Self {
        Self {
            value_: self.value_.clamped_rsh(r.value().shift_amount()),
        }
    }
}
impl<T: IntegralOps, R: UncheckedInput> ShlAssign<R> for ClampedNumeric<T>
where
    R::Value: UnsignedShiftAmount,
{
    fn shl_assign(&mut self, r: R) {
        *self = *self << r
    }
}
impl<T: IntegralOps, R: UncheckedInput> ShrAssign<R> for ClampedNumeric<T>
where
    R::Value: UnsignedShiftAmount,
{
    fn shr_assign(&mut self, r: R) {
        *self = *self >> r
    }
}
macro_rules! clampfn {
    ($name:ident,$method:ident) => {
        pub fn $name<L: UncheckedInput, R: UncheckedInput>(
            l: L,
            r: R,
        ) -> ClampedNumeric<<L::Value as Promote<R::Value>>::Max>
        where
            L::Value:
                Promote<R::Value> + ClampedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
        {
            ClampedNumeric {
                value_: l.value().$method(r.value()),
            }
        }
    };
}
clampfn!(ClampAdd, clamp_add);
clampfn!(ClampSub, clamp_sub);
clampfn!(ClampMul, clamp_mul);
clampfn!(ClampDiv, clamp_div);
pub fn ClampMod<L: UncheckedInput, R: UncheckedInput>(
    l: L,
    r: R,
) -> ClampedNumeric<<L::Value as Promote<R::Value>>::Max>
where
    L::Value: Promote<R::Value>
        + CheckedModulo<R::Value, <L::Value as Promote<R::Value>>::Max>
        + Cast<<L::Value as Promote<R::Value>>::Max>,
{
    ClampedNumeric {
        value_: ClampedModOp(l.value(), r.value()),
    }
}
pub fn ClampLsh<L: UncheckedInput, R: UncheckedInput>(l: L, r: R) -> ClampedNumeric<L::Value>
where
    L::Value: IntegralOps,
    R::Value: UnsignedShiftAmount,
{
    ClampedNumeric {
        value_: l.value().clamped_lsh(r.value().shift_amount()),
    }
}
pub fn ClampRsh<L: UncheckedInput, R: UncheckedInput>(l: L, r: R) -> ClampedNumeric<L::Value>
where
    L::Value: IntegralOps,
    R::Value: UnsignedShiftAmount,
{
    ClampedNumeric {
        value_: l.value().clamped_rsh(r.value().shift_amount()),
    }
}
// Source: clamped_math_impl.h:35-57 and clamped_math.h:74-90.
macro_rules! float_unary{($($t:ty),*)=>{$(impl Neg for ClampedNumeric<$t>{type Output=Self;fn neg(self)->Self{Self{value_:-self.value_}}}impl ClampedNumeric<$t>{pub fn Abs(self)->Self{Self{value_:if self.value_<0.0{-self.value_}else{self.value_}}}})*}}
float_unary!(f32, f64);
// Source: clamped_math.h:124-145, native pre/post increment/decrement protocol.
impl<T: Numeric + ClampedBinary<T, T>> ClampedNumeric<T>
where
    i32: Cast<T>,
{
    pub fn increment(&mut self) -> &mut Self {
        *self += saturated_cast::<T, _>(1i32);
        self
    }
    pub fn post_increment(&mut self) -> Self {
        let old = *self;
        self.increment();
        old
    }
    pub fn decrement(&mut self) -> &mut Self {
        *self -= saturated_cast::<T, _>(1i32);
        self
    }
    pub fn post_decrement(&mut self) -> Self {
        let old = *self;
        self.decrement();
        old
    }
}
// Source: clamped_math_impl.h:287-318. Max uses MaxExponentPromotion, Min uses
// LowestValuePromotion, and comparison is the native IsGreater/IsLess (not host cmp).
pub fn ClampMax<L, R>(l: L, r: R) -> ClampedNumeric<<L as Promote<R>>::Max>
where
    L: Numeric + Promote<R> + Cast<R> + Cast<<L as Promote<R>>::Max>,
    R: Numeric + Cast<L> + Cast<<L as Promote<R>>::Max>,
{
    ClampedNumeric {
        value_: if IsGreater(l, r) {
            saturated_cast(l)
        } else {
            saturated_cast(r)
        },
    }
}
pub fn ClampMin<L, R>(l: L, r: R) -> ClampedNumeric<<L as Promote<R>>::Lowest>
where
    L: Numeric + Promote<R> + Cast<R> + Cast<<L as Promote<R>>::Lowest>,
    R: Numeric + Cast<L> + Cast<<L as Promote<R>>::Lowest>,
{
    ClampedNumeric {
        value_: if IsLess(l, r) {
            saturated_cast(l)
        } else {
            saturated_cast(r)
        },
    }
}
// Source: clamped_math.h comparison macro expansions.
impl<T: Numeric, R: UncheckedInput> PartialEq<R> for ClampedNumeric<T>
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
impl<T: Numeric, R: UncheckedInput> PartialOrd<R> for ClampedNumeric<T>
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
macro_rules! bitfn {
    ($name:ident,$method:ident) => {
        pub fn $name<L: UncheckedInput, R: UncheckedInput>(
            l: L,
            r: R,
        ) -> ClampedNumeric<<L::Value as BitPair<R::Value>>::Output>
        where
            L::Value: BitPair<R::Value>,
        {
            ClampedNumeric {
                value_: l.value().$method(r.value()),
            }
        }
    };
}
bitfn!(ClampAnd, bit_and);
bitfn!(ClampOr, bit_or);
bitfn!(ClampXor, bit_xor);
// Method names differ from the Rust operator names; expand explicitly.
impl<T: Numeric + BitPair<R::Value>, R: UncheckedInput> core::ops::BitAnd<R> for ClampedNumeric<T> {
    type Output = ClampedNumeric<<T as BitPair<R::Value>>::Output>;
    fn bitand(self, r: R) -> Self::Output {
        ClampedNumeric {
            value_: self.value_.bit_and(r.value()),
        }
    }
}
impl<T: Numeric + BitPair<R::Value>, R: UncheckedInput> core::ops::BitOr<R> for ClampedNumeric<T> {
    type Output = ClampedNumeric<<T as BitPair<R::Value>>::Output>;
    fn bitor(self, r: R) -> Self::Output {
        ClampedNumeric {
            value_: self.value_.bit_or(r.value()),
        }
    }
}
impl<T: Numeric + BitPair<R::Value>, R: UncheckedInput> core::ops::BitXor<R> for ClampedNumeric<T> {
    type Output = ClampedNumeric<<T as BitPair<R::Value>>::Output>;
    fn bitxor(self, r: R) -> Self::Output {
        ClampedNumeric {
            value_: self.value_.bit_xor(r.value()),
        }
    }
}
impl<T: Numeric> UncheckedInput for ClampedNumeric<T> {}
// Source: clamped_math.h assignment operators with unsigned bitwise temporary.
macro_rules! bit_assign {
    ($trait:ident,$method:ident,$raw:ident) => {
        impl<T: Numeric + BitPair<R::Value>, R: UncheckedInput> core::ops::$trait<R>
            for ClampedNumeric<T>
        where
            <T as BitPair<R::Value>>::Output: Cast<T>,
        {
            fn $method(&mut self, r: R) {
                self.value_ = self.value_.$raw(r.value()).cast()
            }
        }
    };
}
bit_assign!(BitAndAssign, bitand_assign, bit_and);
bit_assign!(BitOrAssign, bitor_assign, bit_or);
bit_assign!(BitXorAssign, bitxor_assign, bit_xor);
impl<T: Numeric> ClampedNumeric<T> {
    pub fn Max<R>(self, r: R) -> ClampedNumeric<<T as Promote<R>>::Max>
    where
        T: Promote<R> + Cast<R> + Cast<<T as Promote<R>>::Max>,
        R: Numeric + Cast<T> + Cast<<T as Promote<R>>::Max>,
    {
        ClampMax(self.value_, r)
    }
    pub fn Min<R>(self, r: R) -> ClampedNumeric<<T as Promote<R>>::Lowest>
    where
        T: Promote<R> + Cast<R> + Cast<<T as Promote<R>>::Lowest>,
        R: Numeric + Cast<T> + Cast<<T as Promote<R>>::Lowest>,
    {
        ClampMin(self.value_, r)
    }
}

// Source: native MathWrapper/MathOp/variadic template operation markers.
use super::safe_math_shared_impl::{AssignMath, BinaryMath, MathOperation, VariadicMath};
pub struct ClampedAddOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for ClampedAddOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedAddOp, R> for L
where
    L::Value: Promote<R::Value> + ClampedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
{
    type Output = ClampedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedAddOp, R> for ClampedNumeric<T>
where
    Self: core::ops::AddAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::AddAssign::add_assign(self, r);
        self
    }
}
pub struct ClampedSubOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for ClampedSubOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedSubOp, R> for L
where
    L::Value: Promote<R::Value> + ClampedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
{
    type Output = ClampedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedSubOp, R> for ClampedNumeric<T>
where
    Self: core::ops::SubAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::SubAssign::sub_assign(self, r);
        self
    }
}
pub struct ClampedMulOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for ClampedMulOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedMulOp, R> for L
where
    L::Value: Promote<R::Value> + ClampedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
{
    type Output = ClampedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedMulOp, R> for ClampedNumeric<T>
where
    Self: core::ops::MulAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::MulAssign::mul_assign(self, r);
        self
    }
}
pub struct ClampedDivOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for ClampedDivOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedDivOp, R> for L
where
    L::Value: Promote<R::Value> + ClampedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
{
    type Output = ClampedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedDivOp, R> for ClampedNumeric<T>
where
    Self: core::ops::DivAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::DivAssign::div_assign(self, r);
        self
    }
}
pub struct ClampedModOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for ClampedModOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedModOp, R> for L
where
    L::Value: Promote<R::Value>
        + CheckedModulo<R::Value, <L::Value as Promote<R::Value>>::Max>
        + Cast<<L::Value as Promote<R::Value>>::Max>,
{
    type Output = ClampedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedModOp, R> for ClampedNumeric<T>
where
    Self: core::ops::RemAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::RemAssign::rem_assign(self, r);
        self
    }
}
pub fn ClampMathOp<Op, L, R, Rest>(
    l: L,
    r: R,
    rest: Rest,
) -> <L as VariadicMath<Op, R, Rest>>::Output
where
    L: VariadicMath<Op, R, Rest>,
{
    <L as VariadicMath<Op, R, Rest>>::math_op(l, r, rest)
}

// Source: remaining bitwise/shift/min/max variadic operation markers.
pub struct ClampedAndOp;
impl<L: Numeric + BitPair<R>, R: Numeric> MathOperation<L, R> for ClampedAndOp {
    type Result = <L as BitPair<R>>::Output;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedAndOp, R> for L
where
    L::Value: BitPair<R::Value>,
    <L::Value as BitPair<R::Value>>::Output: Numeric,
{
    type Output = ClampedNumeric<<L::Value as BitPair<R::Value>>::Output>;
    fn math_op(self, r: R) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedAndOp, R> for ClampedNumeric<T>
where
    Self: core::ops::BitAndAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::BitAndAssign::bitand_assign(self, r);
        self
    }
}
pub struct ClampedOrOp;
impl<L: Numeric + BitPair<R>, R: Numeric> MathOperation<L, R> for ClampedOrOp {
    type Result = <L as BitPair<R>>::Output;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedOrOp, R> for L
where
    L::Value: BitPair<R::Value>,
    <L::Value as BitPair<R::Value>>::Output: Numeric,
{
    type Output = ClampedNumeric<<L::Value as BitPair<R::Value>>::Output>;
    fn math_op(self, r: R) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedOrOp, R> for ClampedNumeric<T>
where
    Self: core::ops::BitOrAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::BitOrAssign::bitor_assign(self, r);
        self
    }
}
pub struct ClampedXorOp;
impl<L: Numeric + BitPair<R>, R: Numeric> MathOperation<L, R> for ClampedXorOp {
    type Result = <L as BitPair<R>>::Output;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedXorOp, R> for L
where
    L::Value: BitPair<R::Value>,
    <L::Value as BitPair<R::Value>>::Output: Numeric,
{
    type Output = ClampedNumeric<<L::Value as BitPair<R::Value>>::Output>;
    fn math_op(self, r: R) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedXorOp, R> for ClampedNumeric<T>
where
    Self: core::ops::BitXorAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::BitXorAssign::bitxor_assign(self, r);
        self
    }
}
pub struct ClampedLshOp;
impl<L: Numeric + IntegralOps, R: Numeric + UnsignedShiftAmount> MathOperation<L, R>
    for ClampedLshOp
{
    type Result = L;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedLshOp, R> for L
where
    L::Value: Numeric + IntegralOps,
    R::Value: UnsignedShiftAmount,
{
    type Output = ClampedNumeric<L::Value>;
    fn math_op(self, r: R) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedLshOp, R> for ClampedNumeric<T>
where
    Self: core::ops::ShlAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::ShlAssign::shl_assign(self, r);
        self
    }
}
pub struct ClampedRshOp;
impl<L: Numeric + IntegralOps, R: Numeric + UnsignedShiftAmount> MathOperation<L, R>
    for ClampedRshOp
{
    type Result = L;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedRshOp, R> for L
where
    L::Value: Numeric + IntegralOps,
    R::Value: UnsignedShiftAmount,
{
    type Output = ClampedNumeric<L::Value>;
    fn math_op(self, r: R) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric, R> AssignMath<ClampedRshOp, R> for ClampedNumeric<T>
where
    Self: core::ops::ShrAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::ShrAssign::shr_assign(self, r);
        self
    }
}
pub struct ClampedMaxOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for ClampedMaxOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedMaxOp, R> for L
where
    L::Value: Promote<R::Value> + Cast<R::Value> + Cast<<L::Value as Promote<R::Value>>::Max>,
    R::Value: Cast<L::Value> + Cast<<L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: Numeric,
{
    type Output = ClampedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        ClampMax(self.value(), r.value())
    }
}
pub struct ClampedMinOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for ClampedMinOp {
    type Result = <L as Promote<R>>::Lowest;
}
impl<L: UncheckedInput, R: UncheckedInput> BinaryMath<ClampedMinOp, R> for L
where
    L::Value: Promote<R::Value> + Cast<R::Value> + Cast<<L::Value as Promote<R::Value>>::Lowest>,
    R::Value: Cast<L::Value> + Cast<<L::Value as Promote<R::Value>>::Lowest>,
    <L::Value as Promote<R::Value>>::Lowest: Numeric,
{
    type Output = ClampedNumeric<<L::Value as Promote<R::Value>>::Lowest>;
    fn math_op(self, r: R) -> Self::Output {
        ClampMin(self.value(), r.value())
    }
}

// Source: safe_math_shared_impl.h:186-206; concrete native primitive-left domains.
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for i8
where
    i8: Promote<T> + ClampedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i8 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for i8
where
    i8: Promote<T> + ClampedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i8 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for i8
where
    i8: Promote<T> + ClampedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i8 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for i8
where
    i8: Promote<T> + ClampedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i8 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for i8
where
    i8: Promote<T> + CheckedModulo<T, <i8 as Promote<T>>::Max> + Cast<<i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i8 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for i8
where
    i8: BitPair<T>,
    <i8 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i8 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for i8
where
    i8: BitPair<T>,
    <i8 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i8 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for i8
where
    i8: BitPair<T>,
    <i8 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i8 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for i8 {
    type Output = ClampedNumeric<i8>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for i8 {
    type Output = ClampedNumeric<i8>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for u8
where
    u8: Promote<T> + ClampedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u8 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for u8
where
    u8: Promote<T> + ClampedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u8 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for u8
where
    u8: Promote<T> + ClampedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u8 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for u8
where
    u8: Promote<T> + ClampedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u8 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for u8
where
    u8: Promote<T> + CheckedModulo<T, <u8 as Promote<T>>::Max> + Cast<<u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u8 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for u8
where
    u8: BitPair<T>,
    <u8 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u8 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for u8
where
    u8: BitPair<T>,
    <u8 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u8 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for u8
where
    u8: BitPair<T>,
    <u8 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u8 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for u8 {
    type Output = ClampedNumeric<u8>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for u8 {
    type Output = ClampedNumeric<u8>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for i16
where
    i16: Promote<T> + ClampedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i16 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for i16
where
    i16: Promote<T> + ClampedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i16 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for i16
where
    i16: Promote<T> + ClampedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i16 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for i16
where
    i16: Promote<T> + ClampedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i16 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for i16
where
    i16: Promote<T> + CheckedModulo<T, <i16 as Promote<T>>::Max> + Cast<<i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i16 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for i16
where
    i16: BitPair<T>,
    <i16 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i16 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for i16
where
    i16: BitPair<T>,
    <i16 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i16 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for i16
where
    i16: BitPair<T>,
    <i16 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i16 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for i16 {
    type Output = ClampedNumeric<i16>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for i16 {
    type Output = ClampedNumeric<i16>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for u16
where
    u16: Promote<T> + ClampedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u16 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for u16
where
    u16: Promote<T> + ClampedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u16 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for u16
where
    u16: Promote<T> + ClampedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u16 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for u16
where
    u16: Promote<T> + ClampedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u16 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for u16
where
    u16: Promote<T> + CheckedModulo<T, <u16 as Promote<T>>::Max> + Cast<<u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u16 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for u16
where
    u16: BitPair<T>,
    <u16 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u16 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for u16
where
    u16: BitPair<T>,
    <u16 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u16 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for u16
where
    u16: BitPair<T>,
    <u16 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u16 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for u16 {
    type Output = ClampedNumeric<u16>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for u16 {
    type Output = ClampedNumeric<u16>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for i32
where
    i32: Promote<T> + ClampedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i32 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for i32
where
    i32: Promote<T> + ClampedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i32 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for i32
where
    i32: Promote<T> + ClampedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i32 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for i32
where
    i32: Promote<T> + ClampedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i32 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for i32
where
    i32: Promote<T> + CheckedModulo<T, <i32 as Promote<T>>::Max> + Cast<<i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i32 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for i32
where
    i32: BitPair<T>,
    <i32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i32 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for i32
where
    i32: BitPair<T>,
    <i32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i32 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for i32
where
    i32: BitPair<T>,
    <i32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i32 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for i32 {
    type Output = ClampedNumeric<i32>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for i32 {
    type Output = ClampedNumeric<i32>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for u32
where
    u32: Promote<T> + ClampedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u32 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for u32
where
    u32: Promote<T> + ClampedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u32 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for u32
where
    u32: Promote<T> + ClampedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u32 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for u32
where
    u32: Promote<T> + ClampedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u32 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for u32
where
    u32: Promote<T> + CheckedModulo<T, <u32 as Promote<T>>::Max> + Cast<<u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u32 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for u32
where
    u32: BitPair<T>,
    <u32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u32 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for u32
where
    u32: BitPair<T>,
    <u32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u32 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for u32
where
    u32: BitPair<T>,
    <u32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u32 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for u32 {
    type Output = ClampedNumeric<u32>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for u32 {
    type Output = ClampedNumeric<u32>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for i64
where
    i64: Promote<T> + ClampedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i64 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for i64
where
    i64: Promote<T> + ClampedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i64 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for i64
where
    i64: Promote<T> + ClampedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i64 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for i64
where
    i64: Promote<T> + ClampedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i64 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for i64
where
    i64: Promote<T> + CheckedModulo<T, <i64 as Promote<T>>::Max> + Cast<<i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<i64 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for i64
where
    i64: BitPair<T>,
    <i64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i64 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for i64
where
    i64: BitPair<T>,
    <i64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i64 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for i64
where
    i64: BitPair<T>,
    <i64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<i64 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for i64 {
    type Output = ClampedNumeric<i64>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for i64 {
    type Output = ClampedNumeric<i64>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for u64
where
    u64: Promote<T> + ClampedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u64 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for u64
where
    u64: Promote<T> + ClampedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u64 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for u64
where
    u64: Promote<T> + ClampedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u64 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for u64
where
    u64: Promote<T> + ClampedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u64 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for u64
where
    u64: Promote<T> + CheckedModulo<T, <u64 as Promote<T>>::Max> + Cast<<u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<u64 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for u64
where
    u64: BitPair<T>,
    <u64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u64 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for u64
where
    u64: BitPair<T>,
    <u64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u64 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for u64
where
    u64: BitPair<T>,
    <u64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<u64 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for u64 {
    type Output = ClampedNumeric<u64>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for u64 {
    type Output = ClampedNumeric<u64>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for isize
where
    isize: Promote<T> + ClampedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<isize as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for isize
where
    isize: Promote<T> + ClampedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<isize as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for isize
where
    isize: Promote<T> + ClampedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<isize as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for isize
where
    isize: Promote<T> + ClampedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<isize as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for isize
where
    isize: Promote<T>
        + CheckedModulo<T, <isize as Promote<T>>::Max>
        + Cast<<isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<isize as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for isize
where
    isize: BitPair<T>,
    <isize as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<isize as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for isize
where
    isize: BitPair<T>,
    <isize as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<isize as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for isize
where
    isize: BitPair<T>,
    <isize as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<isize as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for isize {
    type Output = ClampedNumeric<isize>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for isize {
    type Output = ClampedNumeric<isize>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for usize
where
    usize: Promote<T> + ClampedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<usize as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for usize
where
    usize: Promote<T> + ClampedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<usize as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for usize
where
    usize: Promote<T> + ClampedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<usize as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for usize
where
    usize: Promote<T> + ClampedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<usize as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for usize
where
    usize: Promote<T>
        + CheckedModulo<T, <usize as Promote<T>>::Max>
        + Cast<<usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<usize as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for usize
where
    usize: BitPair<T>,
    <usize as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<usize as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for usize
where
    usize: BitPair<T>,
    <usize as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<usize as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for usize
where
    usize: BitPair<T>,
    <usize as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<usize as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shl<ClampedNumeric<T>> for usize {
    type Output = ClampedNumeric<usize>;
    fn shl(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampLsh(self, r)
    }
}
impl<T: Numeric + UnsignedShiftAmount> core::ops::Shr<ClampedNumeric<T>> for usize {
    type Output = ClampedNumeric<usize>;
    fn shr(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampRsh(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for f32
where
    f32: Promote<T> + ClampedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f32 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for f32
where
    f32: Promote<T> + ClampedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f32 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for f32
where
    f32: Promote<T> + ClampedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f32 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for f32
where
    f32: Promote<T> + ClampedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f32 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for f32
where
    f32: Promote<T> + CheckedModulo<T, <f32 as Promote<T>>::Max> + Cast<<f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f32 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for f32
where
    f32: BitPair<T>,
    <f32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<f32 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for f32
where
    f32: BitPair<T>,
    <f32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<f32 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for f32
where
    f32: BitPair<T>,
    <f32 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<f32 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for f64
where
    f64: Promote<T> + ClampedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f64 as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for f64
where
    f64: Promote<T> + ClampedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f64 as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for f64
where
    f64: Promote<T> + ClampedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f64 as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for f64
where
    f64: Promote<T> + ClampedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f64 as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for f64
where
    f64: Promote<T> + CheckedModulo<T, <f64 as Promote<T>>::Max> + Cast<<f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<f64 as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for f64
where
    f64: BitPair<T>,
    <f64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<f64 as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for f64
where
    f64: BitPair<T>,
    <f64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<f64 as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for f64
where
    f64: BitPair<T>,
    <f64 as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<f64 as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}
impl<T: Numeric> core::ops::Add<ClampedNumeric<T>> for bool
where
    bool: Promote<T> + ClampedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<bool as Promote<T>>::Max>;
    fn add(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAdd(self, r)
    }
}
impl<T: Numeric> core::ops::Sub<ClampedNumeric<T>> for bool
where
    bool: Promote<T> + ClampedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<bool as Promote<T>>::Max>;
    fn sub(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampSub(self, r)
    }
}
impl<T: Numeric> core::ops::Mul<ClampedNumeric<T>> for bool
where
    bool: Promote<T> + ClampedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<bool as Promote<T>>::Max>;
    fn mul(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMul(self, r)
    }
}
impl<T: Numeric> core::ops::Div<ClampedNumeric<T>> for bool
where
    bool: Promote<T> + ClampedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<bool as Promote<T>>::Max>;
    fn div(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampDiv(self, r)
    }
}
impl<T: Numeric> core::ops::Rem<ClampedNumeric<T>> for bool
where
    bool:
        Promote<T> + CheckedModulo<T, <bool as Promote<T>>::Max> + Cast<<bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: Numeric,
{
    type Output = ClampedNumeric<<bool as Promote<T>>::Max>;
    fn rem(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampMod(self, r)
    }
}
impl<T: Numeric> core::ops::BitAnd<ClampedNumeric<T>> for bool
where
    bool: BitPair<T>,
    <bool as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<bool as BitPair<T>>::Output>;
    fn bitand(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampAnd(self, r)
    }
}
impl<T: Numeric> core::ops::BitOr<ClampedNumeric<T>> for bool
where
    bool: BitPair<T>,
    <bool as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<bool as BitPair<T>>::Output>;
    fn bitor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampOr(self, r)
    }
}
impl<T: Numeric> core::ops::BitXor<ClampedNumeric<T>> for bool
where
    bool: BitPair<T>,
    <bool as BitPair<T>>::Output: Numeric,
{
    type Output = ClampedNumeric<<bool as BitPair<T>>::Output>;
    fn bitxor(self, r: ClampedNumeric<T>) -> Self::Output {
        ClampXor(self, r)
    }
}

// Source: safe_conversions.h:306-318 / clamped_math.h:229-234, primitive-left comparisons.
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for i8
where
    i8: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for i8
where
    i8: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for u8
where
    u8: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for u8
where
    u8: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for i16
where
    i16: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for i16
where
    i16: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for u16
where
    u16: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for u16
where
    u16: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for i32
where
    i32: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for i32
where
    i32: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for u32
where
    u32: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for u32
where
    u32: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for i64
where
    i64: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for i64
where
    i64: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for u64
where
    u64: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for u64
where
    u64: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for isize
where
    isize: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for isize
where
    isize: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for usize
where
    usize: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for usize
where
    usize: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for f32
where
    f32: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for f32
where
    f32: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for f64
where
    f64: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for f64
where
    f64: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}
impl<T: Numeric> PartialEq<ClampedNumeric<T>> for bool
where
    bool: SafeComparison<T>,
{
    fn eq(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_eq(r.value_)
    }
    fn ne(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ne(r.value_)
    }
}
impl<T: Numeric> PartialOrd<ClampedNumeric<T>> for bool
where
    bool: SafeComparison<T>,
{
    fn partial_cmp(&self, r: &ClampedNumeric<T>) -> Option<core::cmp::Ordering> {
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
    fn lt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_lt(r.value_)
    }
    fn le(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_le(r.value_)
    }
    fn gt(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_gt(r.value_)
    }
    fn ge(&self, r: &ClampedNumeric<T>) -> bool {
        self.safe_ge(r.value_)
    }
}

// Source: base/numerics/clamped_math.h:34-36,86-91; base/numerics/clamped_math_impl.h:42-53.
// Concrete same-format constructor and the actual unsigned saturated-Abs identity
// are genuinely const; no runtime trait method is called during evaluation.
impl ClampedNumeric<i64> {
    pub const fn const_new(value: i64) -> Self {
        Self { value_: value }
    }
    pub const fn const_abs(self) -> Self {
        let value = self.value_;
        let abs = if value < 0 {
            0u64.wrapping_sub(value as u64)
        } else {
            value as u64
        };
        Self {
            value_: abs.wrapping_sub(((abs as i64) < 0) as u64) as i64,
        }
    }
}

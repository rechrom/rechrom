// Source: base/numerics/checked_math.h. State/owner and actual short-circuit rules.
#![allow(non_snake_case)]
use super::checked_math_impl::*;
use super::safe_conversions::*;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Sub, SubAssign,
};
#[derive(Clone, Copy)]
pub struct CheckedNumeric<T: StateValue> {
    state_: T::Storage,
}
impl<T: StateValue> CheckedNumeric<T> {
    pub fn new<S: CheckedInput>(v: S) -> Self
    where
        S::Value: Cast<T>,
    {
        Self {
            state_: T::state_from(v.value(), v.valid()),
        }
    }
    pub fn zero() -> Self {
        Self {
            state_: T::Storage::from(T::ZERO, true),
        }
    }
    pub fn IsValid(self) -> bool
    where
        T: Cast<T>,
    {
        self.IsValidFor::<T>()
    }
    pub fn IsValidFor<D: Numeric>(self) -> bool
    where
        T: Cast<D>,
    {
        self.state_.valid() && self.state_.value().range().IsValid()
    }
    pub fn IsValidAnd<D: Numeric, F: FnOnce(D) -> bool>(self, predicate: F) -> bool
    where
        T: Cast<D>,
    {
        self.IsValidFor::<D>() && predicate(self.state_.value().cast())
    }
    pub fn IsInvalidOr<D: Numeric, F: FnOnce(D) -> bool>(self, predicate: F) -> bool
    where
        T: Cast<D>,
    {
        !self.IsValidFor::<D>() || predicate(self.state_.value().cast())
    }
    #[must_use]
    pub fn AssignIfValid<D: Numeric>(self, out: &mut D) -> bool
    where
        T: Cast<D>,
    {
        if self.IsValidFor::<D>() {
            *out = self.state_.value().cast();
            true
        } else {
            false
        }
    }
    pub fn ValueOrDie(self) -> StrictNumeric<T>
    where
        T: Cast<T>,
    {
        self.ValueOrDieFor::<T>()
    }
    pub fn ValueOrDieFor<D: Numeric>(self) -> StrictNumeric<D>
    where
        T: Cast<D>,
    {
        if self.IsValidFor::<D>() {
            StrictNumeric {
                value_: self.state_.value().cast(),
            }
        } else {
            CheckOnFailure::HandleFailure()
        }
    }
    pub fn ValueOrDefault<S: UncheckedInput>(self, v: S) -> StrictNumeric<T>
    where
        T: Cast<T>,
        S::Value: Cast<T>,
    {
        self.ValueOrDefaultFor::<T, S>(v)
    }
    pub fn ValueOrDefaultFor<D: Numeric, S: UncheckedInput>(self, v: S) -> StrictNumeric<D>
    where
        T: Cast<D>,
        S::Value: Cast<D>,
    {
        if self.IsValidFor::<D>() {
            StrictNumeric {
                value_: self.state_.value().cast(),
            }
        } else {
            StrictNumeric {
                value_: checked_cast(v),
            }
        }
    }
    pub fn Cast<D: StateValue>(self) -> CheckedNumeric<D>
    where
        T: Cast<D>,
    {
        CheckedNumeric {
            state_: D::state_from(self.state_.value(), self.state_.valid()),
        }
    }
    // Native result is zero-initialized before short-circuiting; failed ops may
    // write it (builtin overflow) and that exact invalid state's bits are retained.
    fn math<S: Input>(self, r: S, op: impl FnOnce(T, S::Value, &mut T) -> bool) -> Self {
        let mut result = T::ZERO;
        let valid =
            self.state_.valid() && r.valid() && op(self.state_.value(), r.value(), &mut result);
        Self {
            state_: T::Storage::from(result, valid),
        }
    }
}
impl<T: StateValue> Default for CheckedNumeric<T> {
    fn default() -> Self {
        Self::zero()
    }
}
impl<T: StateValue> Input for CheckedNumeric<T> {
    type Value = T;
    fn value(self) -> T {
        self.state_.value()
    }
    fn valid(self) -> bool {
        self.state_.valid()
    }
}
// Source: native MathWrapper result_type is MaxExponentPromotion for arithmetic.
macro_rules! binary {
    ($trait:ident,$method:ident,$assign:ident,$amethod:ident,$raw:ident) => {
        impl<T: StateValue, R: Input> $trait<R> for CheckedNumeric<T>
        where
            T: Promote<R::Value> + CheckedBinary<R::Value, <T as Promote<R::Value>>::Max>,
            <T as Promote<R::Value>>::Max: StateValue,
        {
            type Output = CheckedNumeric<<T as Promote<R::Value>>::Max>;
            fn $method(self, r: R) -> Self::Output {
                let mut result = <<T as Promote<R::Value>>::Max as Numeric>::ZERO;
                let valid = self.state_.valid()
                    && r.valid()
                    && CheckedBinary::$raw(self.state_.value(), r.value(), &mut result);
                CheckedNumeric {
                    state_: <<T as Promote<R::Value>>::Max as StateValue>::Storage::from(
                        result, valid,
                    ),
                }
            }
        }
        impl<T: StateValue, R: Input> $assign<R> for CheckedNumeric<T>
        where
            T: CheckedBinary<R::Value, T>,
        {
            fn $amethod(&mut self, r: R) {
                *self = self.math(r, |l, r, o| CheckedBinary::$raw(l, r, o))
            }
        }
    };
}
binary!(Add, add, AddAssign, add_assign, add);
binary!(Sub, sub, SubAssign, sub_assign, sub);
binary!(Mul, mul, MulAssign, mul_assign, mul);
binary!(Div, div, DivAssign, div_assign, div);
impl<T: StateValue, R: Input> Rem<R> for CheckedNumeric<T>
where
    T: Promote<R::Value> + CheckedModulo<R::Value, <T as Promote<R::Value>>::Max>,
    <T as Promote<R::Value>>::Max: StateValue,
{
    type Output = CheckedNumeric<<T as Promote<R::Value>>::Max>;
    fn rem(self, r: R) -> Self::Output {
        let mut result = <<T as Promote<R::Value>>::Max as Numeric>::ZERO;
        let valid =
            self.state_.valid() && r.valid() && self.state_.value().modulo(r.value(), &mut result);
        CheckedNumeric {
            state_: <<T as Promote<R::Value>>::Max as StateValue>::Storage::from(result, valid),
        }
    }
}
impl<T: StateValue, R: Input> RemAssign<R> for CheckedNumeric<T>
where
    T: CheckedModulo<R::Value, T>,
{
    fn rem_assign(&mut self, r: R) {
        *self = self.math(r, |l, r, o| l.modulo(r, o))
    }
}
impl<T: StateValue + IntegralOps + Cast<T>> Neg for CheckedNumeric<T> {
    type Output = Self;
    fn neg(self) -> Self {
        let v = self.state_.value().negate();
        let ok = self.IsValid() && (!T::SIGNED || v != T::LOWEST);
        Self {
            state_: T::Storage::from(v, ok),
        }
    }
}
impl<T: StateValue + IntegralOps + Cast<T::Unsigned>> Not for CheckedNumeric<T>
where
    T::Unsigned: StateValue,
{
    type Output = CheckedNumeric<T::Unsigned>;
    fn not(self) -> Self::Output {
        CheckedNumeric {
            state_: <T::Unsigned as StateValue>::Storage::from(
                self.state_.value().invert(),
                self.state_.valid(),
            ),
        }
    }
}
impl<T: StateValue + IntegralOps + Cast<T>> CheckedNumeric<T> {
    pub fn Abs(self) -> Self {
        if self.state_.value().negative() {
            -self
        } else {
            self
        }
    }
    pub fn UnsignedAbs(self) -> CheckedNumeric<T::Unsigned>
    where
        T::Unsigned: StateValue,
    {
        CheckedNumeric {
            state_: <T::Unsigned as StateValue>::Storage::from(
                self.state_.value().unsigned_abs(),
                self.state_.valid(),
            ),
        }
    }
}
pub fn IsValidForType<D: Numeric, T: StateValue + Cast<D>>(v: CheckedNumeric<T>) -> bool {
    v.IsValidFor::<D>()
}
pub fn ValueOrDieForType<D: Numeric, T: StateValue + Cast<D>>(
    v: CheckedNumeric<T>,
) -> StrictNumeric<D> {
    v.ValueOrDieFor::<D>()
}
pub fn ValueOrDefaultForType<D: Numeric, T: StateValue + Cast<D>, S: UncheckedInput>(
    v: CheckedNumeric<T>,
    d: S,
) -> StrictNumeric<D>
where
    S::Value: Cast<D>,
{
    v.ValueOrDefaultFor::<D, S>(d)
}
macro_rules! checkfn {
    ($name:ident,$raw:ident) => {
        pub fn $name<L: Input, R: Input>(
            l: L,
            r: R,
        ) -> CheckedNumeric<<L::Value as Promote<R::Value>>::Max>
        where
            L::Value:
                Promote<R::Value> + CheckedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
            <L::Value as Promote<R::Value>>::Max: StateValue,
        {
            let mut result = <<L::Value as Promote<R::Value>>::Max as Numeric>::ZERO;
            let valid =
                l.valid() && r.valid() && CheckedBinary::$raw(l.value(), r.value(), &mut result);
            CheckedNumeric {
                state_: <<L::Value as Promote<R::Value>>::Max as StateValue>::Storage::from(
                    result, valid,
                ),
            }
        }
    };
}
checkfn!(CheckAdd, add);
checkfn!(CheckSub, sub);
checkfn!(CheckMul, mul);
checkfn!(CheckDiv, div);
pub fn CheckMod<L: Input, R: Input>(
    l: L,
    r: R,
) -> CheckedNumeric<<L::Value as Promote<R::Value>>::Max>
where
    L::Value: Promote<R::Value> + CheckedModulo<R::Value, <L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    let mut result = <<L::Value as Promote<R::Value>>::Max as Numeric>::ZERO;
    let valid = l.valid() && r.valid() && l.value().modulo(r.value(), &mut result);
    CheckedNumeric {
        state_: <<L::Value as Promote<R::Value>>::Max as StateValue>::Storage::from(result, valid),
    }
}
// Source: checked_math.h:220-226; original float negate/Abs retains NaN state.
macro_rules! float_unary{($($t:ty),*)=>{$(impl Neg for CheckedNumeric<$t>{type Output=Self;fn neg(self)->Self{let mut result=0.0;let success=CheckedBinary::sub(0.0 as $t,self.state_.value(),&mut result);Self{state_:<$t as StateValue>::Storage::from(result,self.IsValid()&&success)}}}impl CheckedNumeric<$t>{pub fn Abs(self)->Self{if self.state_.value().negative(){-self}else{self}}})*}}
float_unary!(f32, f64);
// Source: checked_math_impl.h:366-413 and checked_math.h arithmetic wrappers.
impl<T: StateValue + IntegralOps, R: Input> core::ops::Shl<R> for CheckedNumeric<T>
where
    R::Value: ShiftAmount,
{
    type Output = Self;
    fn shl(self, r: R) -> Self {
        let mut result = T::ZERO;
        let valid = self.state_.valid()
            && r.valid()
            && self
                .state_
                .value()
                .checked_lsh(r.value().shift_amount(), &mut result);
        Self {
            state_: T::Storage::from(result, valid),
        }
    }
}
impl<T: StateValue + IntegralOps, R: Input> core::ops::Shr<R> for CheckedNumeric<T>
where
    R::Value: ShiftAmount,
{
    type Output = Self;
    fn shr(self, r: R) -> Self {
        let mut result = T::ZERO;
        let valid = self.state_.valid()
            && r.valid()
            && self
                .state_
                .value()
                .checked_rsh(r.value().shift_amount(), &mut result);
        Self {
            state_: T::Storage::from(result, valid),
        }
    }
}
impl<T: StateValue + IntegralOps, R: Input> core::ops::ShlAssign<R> for CheckedNumeric<T>
where
    R::Value: ShiftAmount,
{
    fn shl_assign(&mut self, r: R) {
        *self = *self << r
    }
}
impl<T: StateValue + IntegralOps, R: Input> core::ops::ShrAssign<R> for CheckedNumeric<T>
where
    R::Value: ShiftAmount,
{
    fn shr_assign(&mut self, r: R) {
        *self = *self >> r
    }
}
pub fn CheckLsh<L: Input, R: Input>(l: L, r: R) -> CheckedNumeric<L::Value>
where
    L::Value: StateValue + IntegralOps,
    R::Value: ShiftAmount,
{
    let mut result = L::Value::ZERO;
    let valid =
        l.valid() && r.valid() && l.value().checked_lsh(r.value().shift_amount(), &mut result);
    CheckedNumeric {
        state_: <L::Value as StateValue>::Storage::from(result, valid),
    }
}
pub fn CheckRsh<L: Input, R: Input>(l: L, r: R) -> CheckedNumeric<L::Value>
where
    L::Value: StateValue + IntegralOps,
    R::Value: ShiftAmount,
{
    let mut result = L::Value::ZERO;
    let valid =
        l.valid() && r.valid() && l.value().checked_rsh(r.value().shift_amount(), &mut result);
    CheckedNumeric {
        state_: <L::Value as StateValue>::Storage::from(result, valid),
    }
}
// Source: checked_math.h:239-259. Named methods express pre/post mutation without
// inventing postfix operators in Rust.
impl<T: StateValue + CheckedBinary<T, T> + Cast<T>> CheckedNumeric<T>
where
    i32: Cast<T>,
{
    pub fn increment(&mut self) -> &mut Self {
        *self += checked_cast::<T, _>(1i32);
        self
    }
    pub fn post_increment(&mut self) -> Self {
        let old = *self;
        self.increment();
        old
    }
    pub fn decrement(&mut self) -> &mut Self {
        *self -= checked_cast::<T, _>(1i32);
        self
    }
    pub fn post_decrement(&mut self) -> Self {
        let old = *self;
        self.decrement();
        old
    }
}
// Source: checked_math_impl.h:415-478, native unsigned bitwise result type.
macro_rules! checkbit {
    ($name:ident,$method:ident) => {
        pub fn $name<L: Input, R: Input>(
            l: L,
            r: R,
        ) -> CheckedNumeric<<L::Value as BitPair<R::Value>>::Output>
        where
            L::Value: BitPair<R::Value>,
            <L::Value as BitPair<R::Value>>::Output: StateValue,
        {
            let valid = l.valid() && r.valid();
            let mut result = <<L::Value as BitPair<R::Value>>::Output as Numeric>::ZERO;
            if valid {
                result = l.value().$method(r.value())
            }
            CheckedNumeric {
                state_: <<L::Value as BitPair<R::Value>>::Output as StateValue>::Storage::from(
                    result, valid,
                ),
            }
        }
    };
}
checkbit!(CheckAnd, bit_and);
checkbit!(CheckOr, bit_or);
checkbit!(CheckXor, bit_xor);
impl<T: StateValue + BitPair<R::Value>, R: Input> core::ops::BitAnd<R> for CheckedNumeric<T>
where
    <T as BitPair<R::Value>>::Output: StateValue,
{
    type Output = CheckedNumeric<<T as BitPair<R::Value>>::Output>;
    fn bitand(self, r: R) -> Self::Output {
        let mut value = <<T as BitPair<R::Value>>::Output as Numeric>::ZERO;
        let valid = self.state_.valid() && r.valid();
        if valid {
            value = self.state_.value().bit_and(r.value())
        }
        CheckedNumeric {
            state_: <<T as BitPair<R::Value>>::Output as StateValue>::Storage::from(value, valid),
        }
    }
}
impl<T: StateValue + BitPair<R::Value>, R: Input> core::ops::BitOr<R> for CheckedNumeric<T>
where
    <T as BitPair<R::Value>>::Output: StateValue,
{
    type Output = CheckedNumeric<<T as BitPair<R::Value>>::Output>;
    fn bitor(self, r: R) -> Self::Output {
        let mut value = <<T as BitPair<R::Value>>::Output as Numeric>::ZERO;
        let valid = self.state_.valid() && r.valid();
        if valid {
            value = self.state_.value().bit_or(r.value())
        }
        CheckedNumeric {
            state_: <<T as BitPair<R::Value>>::Output as StateValue>::Storage::from(value, valid),
        }
    }
}
impl<T: StateValue + BitPair<R::Value>, R: Input> core::ops::BitXor<R> for CheckedNumeric<T>
where
    <T as BitPair<R::Value>>::Output: StateValue,
{
    type Output = CheckedNumeric<<T as BitPair<R::Value>>::Output>;
    fn bitxor(self, r: R) -> Self::Output {
        let mut value = <<T as BitPair<R::Value>>::Output as Numeric>::ZERO;
        let valid = self.state_.valid() && r.valid();
        if valid {
            value = self.state_.value().bit_xor(r.value())
        }
        CheckedNumeric {
            state_: <<T as BitPair<R::Value>>::Output as StateValue>::Storage::from(value, valid),
        }
    }
}
// Source: checked_math_impl.h:486-548. Native Max/Min cast selected operand to
// the declared result type before final destination validity check.
pub fn CheckMax<L, R>(l: L, r: R) -> CheckedNumeric<<L::Value as Promote<R::Value>>::Max>
where
    L: Input,
    R: Input,
    L::Value: Promote<R::Value> + Cast<R::Value> + Cast<<L::Value as Promote<R::Value>>::Max>,
    R::Value: Cast<L::Value> + Cast<<L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    let valid = l.valid() && r.valid();
    let mut result = <<L::Value as Promote<R::Value>>::Max as Numeric>::ZERO;
    if valid {
        let a = l.value();
        let b = r.value();
        result = if IsGreater(a, b) { a.cast() } else { b.cast() }
    }
    CheckedNumeric {
        state_: <<L::Value as Promote<R::Value>>::Max as StateValue>::Storage::from(result, valid),
    }
}
pub fn CheckMin<L, R>(l: L, r: R) -> CheckedNumeric<<L::Value as Promote<R::Value>>::Lowest>
where
    L: Input,
    R: Input,
    L::Value: Promote<R::Value> + Cast<R::Value> + Cast<<L::Value as Promote<R::Value>>::Lowest>,
    R::Value: Cast<L::Value> + Cast<<L::Value as Promote<R::Value>>::Lowest>,
    <L::Value as Promote<R::Value>>::Lowest: StateValue,
{
    let valid = l.valid() && r.valid();
    let mut result = <<L::Value as Promote<R::Value>>::Lowest as Numeric>::ZERO;
    if valid {
        let a = l.value();
        let b = r.value();
        result = if IsLess(a, b) { a.cast() } else { b.cast() }
    }
    CheckedNumeric {
        state_: <<L::Value as Promote<R::Value>>::Lowest as StateValue>::Storage::from(
            result, valid,
        ),
    }
}
// Source: checked_math.h:369-384. Concrete native uintptr_t/size_t domain.
/// Safety: T has the native sizeof/layout, and all pointer use follows the original allocation lease.
pub unsafe fn PointerAdd<T>(lhs: *mut T, rhs: StrictNumeric<usize>) -> *mut T {
    let offset = CheckMul(core::mem::size_of::<T>(), rhs.RawValue());
    let result = CheckAdd(lhs.expose_provenance(), offset).ValueOrDie();
    core::ptr::with_exposed_provenance_mut(result.RawValue())
}
/// Safety: same native object-layout/allocation preconditions as PointerAdd.
pub unsafe fn PointerSub<T>(lhs: *mut T, rhs: StrictNumeric<usize>) -> *mut T {
    let offset = CheckMul(core::mem::size_of::<T>(), rhs.RawValue());
    let result = CheckSub(lhs.expose_provenance(), offset).ValueOrDie();
    core::ptr::with_exposed_provenance_mut(result.RawValue())
}
// Source: checked_math.h assignment bitwise operations. The destination is zero
// before evaluation and receives no write when the unsigned result is out of range.
macro_rules! bit_assign {
    ($trait:ident,$method:ident,$raw:ident) => {
        impl<T: StateValue + BitPair<R::Value>, R: Input> core::ops::$trait<R> for CheckedNumeric<T>
        where
            <T as BitPair<R::Value>>::Output: Cast<T>,
        {
            fn $method(&mut self, r: R) {
                let mut result = T::ZERO;
                let valid = self.state_.valid() && r.valid() && {
                    let value = self.state_.value().$raw(r.value());
                    if value.range().IsValid() {
                        result = value.cast();
                        true
                    } else {
                        false
                    }
                };
                self.state_ = T::Storage::from(result, valid)
            }
        }
    };
}
bit_assign!(BitAndAssign, bitand_assign, bit_and);
bit_assign!(BitOrAssign, bitor_assign, bit_or);
bit_assign!(BitXorAssign, bitxor_assign, bit_xor);
impl<T: StateValue> CheckedNumeric<T> {
    pub fn Max<R>(self, r: R) -> CheckedNumeric<<T as Promote<R::Value>>::Max>
    where
        R: Input,
        T: Promote<R::Value> + Cast<R::Value> + Cast<<T as Promote<R::Value>>::Max>,
        R::Value: Cast<T> + Cast<<T as Promote<R::Value>>::Max>,
        <T as Promote<R::Value>>::Max: StateValue,
    {
        CheckMax(self, r)
    }
    pub fn Min<R>(self, r: R) -> CheckedNumeric<<T as Promote<R::Value>>::Lowest>
    where
        R: Input,
        T: Promote<R::Value> + Cast<R::Value> + Cast<<T as Promote<R::Value>>::Lowest>,
        R::Value: Cast<T> + Cast<<T as Promote<R::Value>>::Lowest>,
        <T as Promote<R::Value>>::Lowest: StateValue,
    {
        CheckMin(self, r)
    }
}

// Source: native MathWrapper/MathOp/variadic template operation markers.
use super::safe_math_shared_impl::{AssignMath, BinaryMath, MathOperation, VariadicMath};
pub struct CheckedAddOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for CheckedAddOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: Input, R: Input> BinaryMath<CheckedAddOp, R> for L
where
    L::Value: Promote<R::Value> + CheckedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    type Output = CheckedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedAddOp, R> for CheckedNumeric<T>
where
    Self: core::ops::AddAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::AddAssign::add_assign(self, r);
        self
    }
}
pub struct CheckedSubOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for CheckedSubOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: Input, R: Input> BinaryMath<CheckedSubOp, R> for L
where
    L::Value: Promote<R::Value> + CheckedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    type Output = CheckedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedSubOp, R> for CheckedNumeric<T>
where
    Self: core::ops::SubAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::SubAssign::sub_assign(self, r);
        self
    }
}
pub struct CheckedMulOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for CheckedMulOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: Input, R: Input> BinaryMath<CheckedMulOp, R> for L
where
    L::Value: Promote<R::Value> + CheckedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    type Output = CheckedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedMulOp, R> for CheckedNumeric<T>
where
    Self: core::ops::MulAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::MulAssign::mul_assign(self, r);
        self
    }
}
pub struct CheckedDivOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for CheckedDivOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: Input, R: Input> BinaryMath<CheckedDivOp, R> for L
where
    L::Value: Promote<R::Value> + CheckedBinary<R::Value, <L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    type Output = CheckedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedDivOp, R> for CheckedNumeric<T>
where
    Self: core::ops::DivAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::DivAssign::div_assign(self, r);
        self
    }
}
pub struct CheckedModOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for CheckedModOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: Input, R: Input> BinaryMath<CheckedModOp, R> for L
where
    L::Value: Promote<R::Value> + CheckedModulo<R::Value, <L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    type Output = CheckedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedModOp, R> for CheckedNumeric<T>
where
    Self: core::ops::RemAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::RemAssign::rem_assign(self, r);
        self
    }
}
pub fn CheckMathOp<Op, L, R, Rest>(
    l: L,
    r: R,
    rest: Rest,
) -> <L as VariadicMath<Op, R, Rest>>::Output
where
    L: VariadicMath<Op, R, Rest>,
{
    <L as VariadicMath<Op, R, Rest>>::math_op(l, r, rest)
}

// Source: checked_math.h constructor overload domains (Clamped is not arithmetic).
pub trait CheckedInput: Input {}
impl<T: Numeric> CheckedInput for T {}
impl<T: Numeric> CheckedInput for StrictNumeric<T> {}
impl<T: StateValue> CheckedInput for CheckedNumeric<T> {}

// Source: remaining bitwise/shift/min/max variadic operation markers.
pub struct CheckedAndOp;
impl<L: Numeric + BitPair<R>, R: Numeric> MathOperation<L, R> for CheckedAndOp {
    type Result = <L as BitPair<R>>::Output;
}
impl<L: Input, R: Input> BinaryMath<CheckedAndOp, R> for L
where
    L::Value: BitPair<R::Value>,
    <L::Value as BitPair<R::Value>>::Output: StateValue,
{
    type Output = CheckedNumeric<<L::Value as BitPair<R::Value>>::Output>;
    fn math_op(self, r: R) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedAndOp, R> for CheckedNumeric<T>
where
    Self: core::ops::BitAndAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::BitAndAssign::bitand_assign(self, r);
        self
    }
}
pub struct CheckedOrOp;
impl<L: Numeric + BitPair<R>, R: Numeric> MathOperation<L, R> for CheckedOrOp {
    type Result = <L as BitPair<R>>::Output;
}
impl<L: Input, R: Input> BinaryMath<CheckedOrOp, R> for L
where
    L::Value: BitPair<R::Value>,
    <L::Value as BitPair<R::Value>>::Output: StateValue,
{
    type Output = CheckedNumeric<<L::Value as BitPair<R::Value>>::Output>;
    fn math_op(self, r: R) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedOrOp, R> for CheckedNumeric<T>
where
    Self: core::ops::BitOrAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::BitOrAssign::bitor_assign(self, r);
        self
    }
}
pub struct CheckedXorOp;
impl<L: Numeric + BitPair<R>, R: Numeric> MathOperation<L, R> for CheckedXorOp {
    type Result = <L as BitPair<R>>::Output;
}
impl<L: Input, R: Input> BinaryMath<CheckedXorOp, R> for L
where
    L::Value: BitPair<R::Value>,
    <L::Value as BitPair<R::Value>>::Output: StateValue,
{
    type Output = CheckedNumeric<<L::Value as BitPair<R::Value>>::Output>;
    fn math_op(self, r: R) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedXorOp, R> for CheckedNumeric<T>
where
    Self: core::ops::BitXorAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::BitXorAssign::bitxor_assign(self, r);
        self
    }
}
pub struct CheckedLshOp;
impl<L: Numeric + IntegralOps, R: Numeric + ShiftAmount> MathOperation<L, R> for CheckedLshOp {
    type Result = L;
}
impl<L: Input, R: Input> BinaryMath<CheckedLshOp, R> for L
where
    L::Value: StateValue + IntegralOps,
    R::Value: ShiftAmount,
{
    type Output = CheckedNumeric<L::Value>;
    fn math_op(self, r: R) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedLshOp, R> for CheckedNumeric<T>
where
    Self: core::ops::ShlAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::ShlAssign::shl_assign(self, r);
        self
    }
}
pub struct CheckedRshOp;
impl<L: Numeric + IntegralOps, R: Numeric + ShiftAmount> MathOperation<L, R> for CheckedRshOp {
    type Result = L;
}
impl<L: Input, R: Input> BinaryMath<CheckedRshOp, R> for L
where
    L::Value: StateValue + IntegralOps,
    R::Value: ShiftAmount,
{
    type Output = CheckedNumeric<L::Value>;
    fn math_op(self, r: R) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue, R> AssignMath<CheckedRshOp, R> for CheckedNumeric<T>
where
    Self: core::ops::ShrAssign<R>,
{
    fn math_op_assign(&mut self, r: R) -> &mut Self {
        core::ops::ShrAssign::shr_assign(self, r);
        self
    }
}
pub struct CheckedMaxOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for CheckedMaxOp {
    type Result = <L as Promote<R>>::Max;
}
impl<L: Input, R: Input> BinaryMath<CheckedMaxOp, R> for L
where
    L::Value: Promote<R::Value> + Cast<R::Value> + Cast<<L::Value as Promote<R::Value>>::Max>,
    R::Value: Cast<L::Value> + Cast<<L::Value as Promote<R::Value>>::Max>,
    <L::Value as Promote<R::Value>>::Max: StateValue,
{
    type Output = CheckedNumeric<<L::Value as Promote<R::Value>>::Max>;
    fn math_op(self, r: R) -> Self::Output {
        CheckMax(self, r)
    }
}
pub struct CheckedMinOp;
impl<L: Numeric + Promote<R>, R: Numeric> MathOperation<L, R> for CheckedMinOp {
    type Result = <L as Promote<R>>::Lowest;
}
impl<L: Input, R: Input> BinaryMath<CheckedMinOp, R> for L
where
    L::Value: Promote<R::Value> + Cast<R::Value> + Cast<<L::Value as Promote<R::Value>>::Lowest>,
    R::Value: Cast<L::Value> + Cast<<L::Value as Promote<R::Value>>::Lowest>,
    <L::Value as Promote<R::Value>>::Lowest: StateValue,
{
    type Output = CheckedNumeric<<L::Value as Promote<R::Value>>::Lowest>;
    fn math_op(self, r: R) -> Self::Output {
        CheckMin(self, r)
    }
}

// Source: safe_math_shared_impl.h:186-206; concrete native primitive-left domains.
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for i8
where
    i8: Promote<T> + CheckedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i8 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for i8
where
    i8: Promote<T> + CheckedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i8 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for i8
where
    i8: Promote<T> + CheckedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i8 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for i8
where
    i8: Promote<T> + CheckedBinary<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i8 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for i8
where
    i8: Promote<T> + CheckedModulo<T, <i8 as Promote<T>>::Max>,
    <i8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i8 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for i8
where
    i8: BitPair<T>,
    <i8 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i8 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for i8
where
    i8: BitPair<T>,
    <i8 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i8 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for i8
where
    i8: BitPair<T>,
    <i8 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i8 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for i8 {
    type Output = CheckedNumeric<i8>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for i8 {
    type Output = CheckedNumeric<i8>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for u8
where
    u8: Promote<T> + CheckedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u8 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for u8
where
    u8: Promote<T> + CheckedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u8 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for u8
where
    u8: Promote<T> + CheckedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u8 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for u8
where
    u8: Promote<T> + CheckedBinary<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u8 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for u8
where
    u8: Promote<T> + CheckedModulo<T, <u8 as Promote<T>>::Max>,
    <u8 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u8 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for u8
where
    u8: BitPair<T>,
    <u8 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u8 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for u8
where
    u8: BitPair<T>,
    <u8 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u8 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for u8
where
    u8: BitPair<T>,
    <u8 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u8 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for u8 {
    type Output = CheckedNumeric<u8>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for u8 {
    type Output = CheckedNumeric<u8>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for i16
where
    i16: Promote<T> + CheckedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i16 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for i16
where
    i16: Promote<T> + CheckedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i16 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for i16
where
    i16: Promote<T> + CheckedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i16 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for i16
where
    i16: Promote<T> + CheckedBinary<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i16 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for i16
where
    i16: Promote<T> + CheckedModulo<T, <i16 as Promote<T>>::Max>,
    <i16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i16 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for i16
where
    i16: BitPair<T>,
    <i16 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i16 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for i16
where
    i16: BitPair<T>,
    <i16 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i16 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for i16
where
    i16: BitPair<T>,
    <i16 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i16 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for i16 {
    type Output = CheckedNumeric<i16>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for i16 {
    type Output = CheckedNumeric<i16>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for u16
where
    u16: Promote<T> + CheckedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u16 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for u16
where
    u16: Promote<T> + CheckedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u16 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for u16
where
    u16: Promote<T> + CheckedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u16 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for u16
where
    u16: Promote<T> + CheckedBinary<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u16 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for u16
where
    u16: Promote<T> + CheckedModulo<T, <u16 as Promote<T>>::Max>,
    <u16 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u16 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for u16
where
    u16: BitPair<T>,
    <u16 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u16 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for u16
where
    u16: BitPair<T>,
    <u16 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u16 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for u16
where
    u16: BitPair<T>,
    <u16 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u16 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for u16 {
    type Output = CheckedNumeric<u16>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for u16 {
    type Output = CheckedNumeric<u16>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for i32
where
    i32: Promote<T> + CheckedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i32 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for i32
where
    i32: Promote<T> + CheckedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i32 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for i32
where
    i32: Promote<T> + CheckedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i32 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for i32
where
    i32: Promote<T> + CheckedBinary<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i32 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for i32
where
    i32: Promote<T> + CheckedModulo<T, <i32 as Promote<T>>::Max>,
    <i32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i32 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for i32
where
    i32: BitPair<T>,
    <i32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i32 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for i32
where
    i32: BitPair<T>,
    <i32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i32 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for i32
where
    i32: BitPair<T>,
    <i32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i32 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for i32 {
    type Output = CheckedNumeric<i32>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for i32 {
    type Output = CheckedNumeric<i32>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for u32
where
    u32: Promote<T> + CheckedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u32 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for u32
where
    u32: Promote<T> + CheckedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u32 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for u32
where
    u32: Promote<T> + CheckedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u32 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for u32
where
    u32: Promote<T> + CheckedBinary<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u32 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for u32
where
    u32: Promote<T> + CheckedModulo<T, <u32 as Promote<T>>::Max>,
    <u32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u32 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for u32
where
    u32: BitPair<T>,
    <u32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u32 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for u32
where
    u32: BitPair<T>,
    <u32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u32 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for u32
where
    u32: BitPair<T>,
    <u32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u32 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for u32 {
    type Output = CheckedNumeric<u32>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for u32 {
    type Output = CheckedNumeric<u32>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for i64
where
    i64: Promote<T> + CheckedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i64 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for i64
where
    i64: Promote<T> + CheckedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i64 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for i64
where
    i64: Promote<T> + CheckedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i64 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for i64
where
    i64: Promote<T> + CheckedBinary<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i64 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for i64
where
    i64: Promote<T> + CheckedModulo<T, <i64 as Promote<T>>::Max>,
    <i64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<i64 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for i64
where
    i64: BitPair<T>,
    <i64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i64 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for i64
where
    i64: BitPair<T>,
    <i64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i64 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for i64
where
    i64: BitPair<T>,
    <i64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<i64 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for i64 {
    type Output = CheckedNumeric<i64>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for i64 {
    type Output = CheckedNumeric<i64>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for u64
where
    u64: Promote<T> + CheckedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u64 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for u64
where
    u64: Promote<T> + CheckedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u64 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for u64
where
    u64: Promote<T> + CheckedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u64 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for u64
where
    u64: Promote<T> + CheckedBinary<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u64 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for u64
where
    u64: Promote<T> + CheckedModulo<T, <u64 as Promote<T>>::Max>,
    <u64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<u64 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for u64
where
    u64: BitPair<T>,
    <u64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u64 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for u64
where
    u64: BitPair<T>,
    <u64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u64 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for u64
where
    u64: BitPair<T>,
    <u64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<u64 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for u64 {
    type Output = CheckedNumeric<u64>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for u64 {
    type Output = CheckedNumeric<u64>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for isize
where
    isize: Promote<T> + CheckedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<isize as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for isize
where
    isize: Promote<T> + CheckedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<isize as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for isize
where
    isize: Promote<T> + CheckedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<isize as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for isize
where
    isize: Promote<T> + CheckedBinary<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<isize as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for isize
where
    isize: Promote<T> + CheckedModulo<T, <isize as Promote<T>>::Max>,
    <isize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<isize as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for isize
where
    isize: BitPair<T>,
    <isize as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<isize as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for isize
where
    isize: BitPair<T>,
    <isize as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<isize as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for isize
where
    isize: BitPair<T>,
    <isize as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<isize as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for isize {
    type Output = CheckedNumeric<isize>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for isize {
    type Output = CheckedNumeric<isize>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for usize
where
    usize: Promote<T> + CheckedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<usize as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for usize
where
    usize: Promote<T> + CheckedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<usize as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for usize
where
    usize: Promote<T> + CheckedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<usize as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for usize
where
    usize: Promote<T> + CheckedBinary<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<usize as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for usize
where
    usize: Promote<T> + CheckedModulo<T, <usize as Promote<T>>::Max>,
    <usize as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<usize as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for usize
where
    usize: BitPair<T>,
    <usize as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<usize as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for usize
where
    usize: BitPair<T>,
    <usize as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<usize as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for usize
where
    usize: BitPair<T>,
    <usize as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<usize as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shl<CheckedNumeric<T>> for usize {
    type Output = CheckedNumeric<usize>;
    fn shl(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckLsh(self, r)
    }
}
impl<T: StateValue + ShiftAmount> core::ops::Shr<CheckedNumeric<T>> for usize {
    type Output = CheckedNumeric<usize>;
    fn shr(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckRsh(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for f32
where
    f32: Promote<T> + CheckedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f32 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for f32
where
    f32: Promote<T> + CheckedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f32 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for f32
where
    f32: Promote<T> + CheckedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f32 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for f32
where
    f32: Promote<T> + CheckedBinary<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f32 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for f32
where
    f32: Promote<T> + CheckedModulo<T, <f32 as Promote<T>>::Max>,
    <f32 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f32 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for f32
where
    f32: BitPair<T>,
    <f32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<f32 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for f32
where
    f32: BitPair<T>,
    <f32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<f32 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for f32
where
    f32: BitPair<T>,
    <f32 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<f32 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for f64
where
    f64: Promote<T> + CheckedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f64 as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for f64
where
    f64: Promote<T> + CheckedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f64 as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for f64
where
    f64: Promote<T> + CheckedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f64 as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for f64
where
    f64: Promote<T> + CheckedBinary<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f64 as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for f64
where
    f64: Promote<T> + CheckedModulo<T, <f64 as Promote<T>>::Max>,
    <f64 as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<f64 as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for f64
where
    f64: BitPair<T>,
    <f64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<f64 as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for f64
where
    f64: BitPair<T>,
    <f64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<f64 as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for f64
where
    f64: BitPair<T>,
    <f64 as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<f64 as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}
impl<T: StateValue> core::ops::Add<CheckedNumeric<T>> for bool
where
    bool: Promote<T> + CheckedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<bool as Promote<T>>::Max>;
    fn add(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAdd(self, r)
    }
}
impl<T: StateValue> core::ops::Sub<CheckedNumeric<T>> for bool
where
    bool: Promote<T> + CheckedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<bool as Promote<T>>::Max>;
    fn sub(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckSub(self, r)
    }
}
impl<T: StateValue> core::ops::Mul<CheckedNumeric<T>> for bool
where
    bool: Promote<T> + CheckedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<bool as Promote<T>>::Max>;
    fn mul(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMul(self, r)
    }
}
impl<T: StateValue> core::ops::Div<CheckedNumeric<T>> for bool
where
    bool: Promote<T> + CheckedBinary<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<bool as Promote<T>>::Max>;
    fn div(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckDiv(self, r)
    }
}
impl<T: StateValue> core::ops::Rem<CheckedNumeric<T>> for bool
where
    bool: Promote<T> + CheckedModulo<T, <bool as Promote<T>>::Max>,
    <bool as Promote<T>>::Max: StateValue,
{
    type Output = CheckedNumeric<<bool as Promote<T>>::Max>;
    fn rem(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckMod(self, r)
    }
}
impl<T: StateValue> core::ops::BitAnd<CheckedNumeric<T>> for bool
where
    bool: BitPair<T>,
    <bool as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<bool as BitPair<T>>::Output>;
    fn bitand(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckAnd(self, r)
    }
}
impl<T: StateValue> core::ops::BitOr<CheckedNumeric<T>> for bool
where
    bool: BitPair<T>,
    <bool as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<bool as BitPair<T>>::Output>;
    fn bitor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckOr(self, r)
    }
}
impl<T: StateValue> core::ops::BitXor<CheckedNumeric<T>> for bool
where
    bool: BitPair<T>,
    <bool as BitPair<T>>::Output: StateValue,
{
    type Output = CheckedNumeric<<bool as BitPair<T>>::Output>;
    fn bitxor(self, r: CheckedNumeric<T>) -> Self::Output {
        CheckXor(self, r)
    }
}

// Source: checked_math.h:27-48,114-148,264-286; real concrete CTFE domain.
impl CheckedNumeric<i8> {
    pub const fn const_new(value: i8) -> Self {
        Self {
            state_: IntegerState::<i8>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: i8) -> StrictNumeric<i8> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i8>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i8>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i8>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<u8> {
    pub const fn const_new(value: u8) -> Self {
        Self {
            state_: IntegerState::<u8>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: u8) -> StrictNumeric<u8> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u8>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u8>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u8>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<i16> {
    pub const fn const_new(value: i16) -> Self {
        Self {
            state_: IntegerState::<i16>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: i16) -> StrictNumeric<i16> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i16>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i16>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i16>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<u16> {
    pub const fn const_new(value: u16) -> Self {
        Self {
            state_: IntegerState::<u16>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: u16) -> StrictNumeric<u16> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u16>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u16>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u16>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<i32> {
    pub const fn const_new(value: i32) -> Self {
        Self {
            state_: IntegerState::<i32>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: i32) -> StrictNumeric<i32> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i32>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i32>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i32>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<u32> {
    pub const fn const_new(value: u32) -> Self {
        Self {
            state_: IntegerState::<u32>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: u32) -> StrictNumeric<u32> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u32>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u32>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u32>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<i64> {
    pub const fn const_new(value: i64) -> Self {
        Self {
            state_: IntegerState::<i64>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: i64) -> StrictNumeric<i64> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i64>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i64>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<i64>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<u64> {
    pub const fn const_new(value: u64) -> Self {
        Self {
            state_: IntegerState::<u64>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: u64) -> StrictNumeric<u64> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u64>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u64>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<u64>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<isize> {
    pub const fn const_new(value: isize) -> Self {
        Self {
            state_: IntegerState::<isize>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: isize) -> StrictNumeric<isize> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<isize>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<isize>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<isize>::const_from(value, valid),
        }
    }
}
impl CheckedNumeric<usize> {
    pub const fn const_new(value: usize) -> Self {
        Self {
            state_: IntegerState::<usize>::const_from(value, true),
        }
    }
    pub const fn const_is_valid(self) -> bool {
        self.state_.const_valid()
    }
    pub const fn const_value_or_default(self, default: usize) -> StrictNumeric<usize> {
        StrictNumeric {
            value_: if self.state_.const_valid() {
                self.state_.const_value()
            } else {
                default
            },
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_add(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_add(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<usize>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_sub(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_sub(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<usize>::const_from(value, valid),
        }
    }
    #[cfg(all(
        not(base_numeric_armel),
        not(any(base_numeric_wasm, base_numeric_asmjs)),
        not(base_numeric_compiler_msvc),
        any(base_numeric_compiler_clang, base_numeric_compiler_gcc)
    ))]
    pub const fn const_mul(self, r: Self) -> Self {
        let mut value = 0;
        let valid = self.state_.const_valid() && r.state_.const_valid() && {
            let (result, overflow) = self
                .state_
                .const_value()
                .overflowing_mul(r.state_.const_value());
            value = result;
            !overflow
        };
        Self {
            state_: IntegerState::<usize>::const_from(value, valid),
        }
    }
}
// Always a const block: invalid ValueOrDie remains a compile-time error,
// while the ordinary runtime ValueOrDie still invokes the real native handler.
#[macro_export]
macro_rules! NativeCheckedValueOrDieConst {
    ($t:ty,$value:expr) => {
        const {
            let value: $crate::base::numerics::checked_math::CheckedNumeric<$t> = $value;
            assert!(
                value.const_is_valid(),
                "native ValueOrDie invalid constant evaluation"
            );
            value.const_value_or_default(0).RawValue()
        }
    };
}

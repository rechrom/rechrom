//! ARM child implementations. Cast/range/promotion producers are the real main
//! safe_conversions module; no wider replacement type is introduced here.
#![allow(non_snake_case, non_upper_case_globals)]
use super::safe_conversions::{checked_cast, saturated_cast};
use super::safe_conversions_impl::{Cast, Input, Integer, Numeric, Promote};
use core::{marker::PhantomData, mem::MaybeUninit, ops::Mul};

// Source: base/numerics/safe_conversions_impl.h:452-459
// Existing helper's formula, staged only to make the child contract explicit;
// this repeated definition carries zero new source credit.
pub const fn integer_arithmetic_safe<D: Numeric, T: Numeric, U: Numeric>() -> bool {
    !D::FLOAT
        && !T::FLOAT
        && !U::FLOAT
        && (D::SIGNED as u8) >= (T::SIGNED as u8)
        && (D::DIGITS as u32 + D::SIGNED as u32) >= 2 * (T::DIGITS as u32 + T::SIGNED as u32)
        && (D::SIGNED as u8) >= (U::SIGNED as u8)
        && (D::DIGITS as u32 + D::SIGNED as u32) >= 2 * (U::DIGITS as u32 + U::SIGNED as u32)
}
type Fast<T, U> = <T as Promote<U>>::Fast;
type Big<T, U> = <T as Promote<U>>::Big;

// Source: base/numerics/safe_math_arm_impl.h:19-45
pub struct CheckedMulFastAsmOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Promote<U>, U: Integer> CheckedMulFastAsmOp<T, U> {
    pub const is_supported: bool = cfg!(base_numeric_ndebug) && T::FAST_CONTAINED;
    /// result is the native writable destination. On range failure it is not
    /// written, unlike the compiler overflow builtins' truncated output.
    pub unsafe fn Do<V: Numeric>(x: T, y: U, result: *mut V) -> bool
    where
        T: Cast<Fast<T, U>>,
        U: Cast<Fast<T, U>>,
        Fast<T, U>: Mul<Output = Fast<T, U>> + Cast<V>,
    {
        let presult = <T as Cast<Fast<T, U>>>::cast(x) * <U as Cast<Fast<T, U>>>::cast(y);
        if !<Fast<T, U> as Cast<V>>::range(presult).IsValid() {
            return false;
        }
        unsafe {
            result.write(<Fast<T, U> as Cast<V>>::cast(presult));
        }
        true
    }
}

// Source: base/numerics/safe_math_arm_impl.h:48-74
pub struct ClampedAddFastAsmOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Promote<U>, U: Integer> ClampedAddFastAsmOp<T, U>
where
    Big<T, U>: Cast<i32>,
{
    pub const is_supported: bool =
        cfg!(base_numeric_ndebug) && T::BIG_CONTAINED && <Big<T, U> as Cast<i32>>::CONTAINED;
}
#[cfg(all(base_numeric_armel, target_arch = "arm"))]
impl<T: Integer + Input<Value = T> + Cast<i32>, U: Integer + Input<Value = U> + Cast<i32>>
    ClampedAddFastAsmOp<T, U>
{
    #[inline(always)]
    pub fn Do<V: Numeric>(x: T, y: U) -> V
    where
        i32: Cast<V> + Input<Value = i32>,
    {
        if integer_arithmetic_safe::<i32, T, U>() {
            saturated_cast::<V, i32>(<T as Cast<i32>>::cast(x) + <U as Cast<i32>>::cast(y))
        } else {
            let x_i32 = checked_cast::<i32, T>(x);
            let y_i32 = checked_cast::<i32, U>(y);
            let result: i32;
            unsafe {
                core::arch::asm!("qadd {result}, {first}, {second}",result=out(reg) result,
                first=in(reg) x_i32,second=in(reg) y_i32,options(nomem,nostack));
            }
            saturated_cast::<V, i32>(result)
        }
    }
}

// Source: base/numerics/safe_math_arm_impl.h:77-103
pub struct ClampedSubFastAsmOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Promote<U>, U: Integer> ClampedSubFastAsmOp<T, U>
where
    Big<T, U>: Cast<i32>,
{
    pub const is_supported: bool =
        cfg!(base_numeric_ndebug) && T::BIG_CONTAINED && <Big<T, U> as Cast<i32>>::CONTAINED;
}
#[cfg(all(base_numeric_armel, target_arch = "arm"))]
impl<T: Integer + Input<Value = T> + Cast<i32>, U: Integer + Input<Value = U> + Cast<i32>>
    ClampedSubFastAsmOp<T, U>
{
    #[inline(always)]
    pub fn Do<V: Numeric>(x: T, y: U) -> V
    where
        i32: Cast<V> + Input<Value = i32>,
    {
        if integer_arithmetic_safe::<i32, T, U>() {
            saturated_cast::<V, i32>(<T as Cast<i32>>::cast(x) - <U as Cast<i32>>::cast(y))
        } else {
            let x_i32 = checked_cast::<i32, T>(x);
            let y_i32 = checked_cast::<i32, U>(y);
            let result: i32;
            unsafe {
                core::arch::asm!("qsub {result}, {first}, {second}",result=out(reg) result,
                first=in(reg) x_i32,second=in(reg) y_i32,options(nomem,nostack));
            }
            saturated_cast::<V, i32>(result)
        }
    }
}

// Source: base/numerics/safe_math_arm_impl.h:106-129
pub struct ClampedMulFastAsmOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Promote<U>, U: Integer> ClampedMulFastAsmOp<T, U> {
    pub const is_supported: bool =
        cfg!(base_numeric_ndebug) && CheckedMulFastAsmOp::<T, U>::is_supported;
    pub fn Do<V: Numeric>(x: T, y: U) -> V
    where
        T: Cast<Fast<T, U>>,
        U: Cast<Fast<T, U>>,
        Fast<T, U>: Mul<Output = Fast<T, U>> + Cast<V> + Input<Value = Fast<T, U>>,
    {
        const {
            assert!(
                (!integer_arithmetic_safe::<i32, T, U>()
                    && !integer_arithmetic_safe::<u32, T, U>())
                    || T::FAST_CONTAINED
            );
        }
        if !integer_arithmetic_safe::<i32, T, U>() && !integer_arithmetic_safe::<u32, T, U>() {
            let mut result = MaybeUninit::<V>::uninit();
            if unsafe { CheckedMulFastAsmOp::<T, U>::Do(x, y, result.as_mut_ptr()) } {
                unsafe { result.assume_init() }
            } else {
                if x.negative() ^ y.negative() {
                    V::LOWEST
                } else {
                    V::MAX
                }
            }
        } else {
            // Native static_assert: use only with the same contained promotion.
            saturated_cast::<V, Fast<T, U>>(
                <T as Cast<Fast<T, U>>>::cast(x) * <U as Cast<Fast<T, U>>>::cast(y),
            )
        }
    }
}
#[cfg(all(base_numeric_armel, not(target_arch = "arm")))]
compile_error!("native __ARMEL__ qadd/qsub selected without actual Rust ARM32 codegen target");

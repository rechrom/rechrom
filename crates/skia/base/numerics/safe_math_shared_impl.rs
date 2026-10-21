//! Shared native numeric language mapping. Runtime generic dispatch is separate
//! from the concrete const primitive entry points; no runtime API is called const.
#![allow(non_snake_case, non_upper_case_globals)]
use super::safe_conversions_impl::{Input, Numeric};

// Source: base/numerics/safe_math_shared_impl.h:15-22
#[cfg(not(base_numeric_native_inputs_configured))]
compile_error!("Bind actual compiler __asmjs__/__wasm__/__ARMEL__/__clang__/NDEBUG inputs before selecting native numeric children");
#[cfg(not(any(base_numeric_asmjs, base_numeric_wasm)))]
pub use super::safe_math_clang_gcc_impl::*;

// Source: base/numerics/safe_math_shared_impl.h:28-101
// Native unavailable fast paths remain false + the real CheckOnFailure trap,
// not a successful arithmetic fallback.
#[cfg(any(base_numeric_asmjs, base_numeric_wasm))]
mod unavailable {
    use super::super::safe_conversions_impl::CheckOnFailure;
    use core::marker::PhantomData;
    macro_rules! checked {
        ($name:ident) => {
            pub struct $name<T, U>(PhantomData<(T, U)>);
            impl<T, U> $name<T, U> {
                pub const is_supported: bool = false;
                pub unsafe fn Do<V>(_: T, _: U, _: *mut V) -> bool {
                    CheckOnFailure::HandleFailure::<bool>()
                }
            }
        };
    }
    checked!(CheckedAddFastOp);
    checked!(CheckedSubFastOp);
    checked!(CheckedMulFastOp);
    macro_rules! clamped {
        ($name:ident) => {
            pub struct $name<T, U>(PhantomData<(T, U)>);
            impl<T, U> $name<T, U> {
                pub const is_supported: bool = false;
                pub fn Do<V: Default>(_: T, _: U) -> V {
                    CheckOnFailure::HandleFailure::<V>()
                }
            }
        };
    }
    clamped!(ClampedAddFastOp);
    clamped!(ClampedSubFastOp);
    clamped!(ClampedMulFastOp);
    pub struct ClampedNegFastOp<T>(PhantomData<T>);
    impl<T> ClampedNegFastOp<T> {
        pub const is_supported: bool = false;
        pub fn Do(_: T) -> T
        where
            T: Default,
        {
            CheckOnFailure::HandleFailure::<T>()
        }
    }
}
#[cfg(any(base_numeric_asmjs, base_numeric_wasm))]
pub use unavailable::*;

// Source: base/numerics/safe_math_shared_impl.h:107-120
pub trait UnsignedOrFloatForSize {
    type Type;
}
macro_rules! unsigned_type {($($t:ty=>$u:ty),* $(,)?)=>{$(
    impl UnsignedOrFloatForSize for $t {type Type=$u;}
)*}}
unsigned_type!(i8=>u8,u8=>u8,i16=>u16,u16=>u16,i32=>u32,u32=>u32,
 i64=>u64,u64=>u64,isize=>usize,usize=>usize,f32=>f32,f64=>f64);

// Source: base/numerics/safe_math_shared_impl.h:128-164
pub trait UnaryNumeric: Copy + UnsignedOrFloatForSize {
    fn negate(self) -> Self;
    fn abs(self) -> Self;
}
pub trait IntegralUnary: UnaryNumeric {
    fn invert(self) -> <Self as UnsignedOrFloatForSize>::Type;
}
pub fn NegateWrapper<T: UnaryNumeric>(value: T) -> T {
    value.negate()
}
pub fn InvertWrapper<T: IntegralUnary>(value: T) -> <T as UnsignedOrFloatForSize>::Type {
    value.invert()
}
pub fn AbsWrapper<T: UnaryNumeric>(value: T) -> T {
    value.abs()
}
macro_rules! integral_unary {
    ($t:ty,$u:ty,$neg:ident,$inv:ident,$abs:ident) => {
        pub const fn $neg(value: $t) -> $t {
            (0 as $u).wrapping_sub(value as $u) as $t
        }
        pub const fn $inv(value: $t) -> $u {
            (!value) as $u
        }
        pub const fn $abs(value: $t) -> $t {
            // Native SafeUnsignedAbs performs unsigned subtraction; casting back
            // keeps the signed-min bit pattern. No signed abs overflow is invoked.
            (if value < 0 as $t {
                (0 as $u).wrapping_sub(value as $u)
            } else {
                value as $u
            }) as $t
        }
        impl UnaryNumeric for $t {
            fn negate(self) -> Self {
                $neg(self)
            }
            fn abs(self) -> Self {
                $abs(self)
            }
        }
        impl IntegralUnary for $t {
            fn invert(self) -> $u {
                $inv(self)
            }
        }
    };
}
integral_unary!(i8, u8, negate_i8, invert_i8, abs_i8);
integral_unary!(u8, u8, negate_u8, invert_u8, abs_u8);
integral_unary!(i16, u16, negate_i16, invert_i16, abs_i16);
integral_unary!(u16, u16, negate_u16, invert_u16, abs_u16);
integral_unary!(i32, u32, negate_i32, invert_i32, abs_i32);
integral_unary!(u32, u32, negate_u32, invert_u32, abs_u32);
integral_unary!(i64, u64, negate_i64, invert_i64, abs_i64);
integral_unary!(u64, u64, negate_u64, invert_u64, abs_u64);
integral_unary!(isize, usize, negate_isize, invert_isize, abs_isize);
integral_unary!(usize, usize, negate_usize, invert_usize, abs_usize);
macro_rules! float_unary {
    ($t:ty,$neg:ident,$abs:ident) => {
        pub const fn $neg(value: $t) -> $t {
            -value
        }
        pub const fn $abs(value: $t) -> $t {
            if value < 0.0 {
                -value
            } else {
                value
            }
        }
        impl UnaryNumeric for $t {
            fn negate(self) -> Self {
                $neg(self)
            }
            fn abs(self) -> Self {
                $abs(self)
            }
        }
    };
}
float_unary!(f32, negate_f32, abs_f32);
float_unary!(f64, negate_f64, abs_f64);

// Source: base/numerics/safe_math_shared_impl.h:166-175
// A template-template parameter becomes an operation marker + a pair-indexed
// associated result type. Missing result_type means no trait implementation,
// preserving the native SFINAE rejection rather than a default result type.
pub trait MathOperation<L: Numeric, R: Numeric> {
    type Result: Numeric;
}
pub trait MathWrapperFor<L: Input, R: Input> {
    type Math;
    type Type: Numeric;
}
impl<M, L: Input, R: Input> MathWrapperFor<L, R> for M
where
    M: MathOperation<L::Value, R::Value>,
{
    type Math = M;
    type Type = <M as MathOperation<L::Value, R::Value>>::Result;
}

// Source: base/numerics/safe_math_shared_impl.h:180-206
// Native template argument packs are a Rust tuple of already-evaluated values,
// not a formatter/provider. Implementations are the original main-header MathOp
// and its constraints; there is deliberately no substitute arithmetic body here.
pub trait VariadicMath<Op, R, Rest> {
    type Output;
    fn math_op(self, rhs: R, rest: Rest) -> Self::Output;
}
pub trait BinaryMath<Op, R> {
    type Output;
    fn math_op(self, rhs: R) -> Self::Output;
}
pub trait AssignMath<Op, R> {
    fn math_op_assign(&mut self, rhs: R) -> &mut Self;
}
#[macro_export]
macro_rules! BASE_NUMERIC_ARITHMETIC_VARIADIC {
    ($name:ident,$op:ty)=>{
        pub fn $name<L,R,Rest>(lhs:L,rhs:R,rest:Rest)-><L as $crate::base::numerics::safe_math_shared_impl::VariadicMath<$op,R,Rest>>::Output
        where L:$crate::base::numerics::safe_math_shared_impl::VariadicMath<$op,R,Rest> {
            <L as $crate::base::numerics::safe_math_shared_impl::VariadicMath<$op,R,Rest>>::math_op(lhs,rhs,rest)
        }
    }
}
#[macro_export]
macro_rules! BASE_NUMERIC_ARITHMETIC_OPERATORS {
    ($class:ident,$op:ty,$trait:ident,$method:ident,$assign_trait:ident,$assign_method:ident,$function:ident)=>{
        impl<L,R> core::ops::$trait<R> for $class<L>
        where $class<L>:$crate::base::numerics::safe_math_shared_impl::BinaryMath<$op,R> {
            type Output=<$class<L> as $crate::base::numerics::safe_math_shared_impl::BinaryMath<$op,R>>::Output;
            fn $method(self,rhs:R)->Self::Output {
                <$class<L> as $crate::base::numerics::safe_math_shared_impl::BinaryMath<$op,R>>::math_op(self,rhs)
            }
        }
        impl<L,R> core::ops::$assign_trait<R> for $class<L>
        where $class<L>:$crate::base::numerics::safe_math_shared_impl::AssignMath<$op,R> {
            fn $assign_method(&mut self,rhs:R) {
                <$class<L> as $crate::base::numerics::safe_math_shared_impl::AssignMath<$op,R>>::math_op_assign(self,rhs);
            }
        }
        $crate::BASE_NUMERIC_ARITHMETIC_VARIADIC!($function,$op);
    }
}

// Source: native variadic overload folds MathOp(MathOp(lhs,rhs),args...) in order.
impl<Op, L, R> VariadicMath<Op, R, ()> for L
where
    L: BinaryMath<Op, R>,
{
    type Output = <L as BinaryMath<Op, R>>::Output;
    fn math_op(self, r: R, _: ()) -> Self::Output {
        <L as BinaryMath<Op, R>>::math_op(self, r)
    }
}
impl<Op, L, R, H, Tail> VariadicMath<Op, R, (H, Tail)> for L
where
    L: BinaryMath<Op, R>,
    <L as BinaryMath<Op, R>>::Output: VariadicMath<Op, H, Tail>,
{
    type Output = <<L as BinaryMath<Op, R>>::Output as VariadicMath<Op, H, Tail>>::Output;
    fn math_op(self, r: R, rest: (H, Tail)) -> Self::Output {
        let first = <L as BinaryMath<Op, R>>::math_op(self, r);
        <<L as BinaryMath<Op, R>>::Output as VariadicMath<Op, H, Tail>>::math_op(
            first, rest.0, rest.1,
        )
    }
}

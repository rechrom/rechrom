//! Original native compiler/ARM branch selection, not Rust target-env inference.
#![allow(non_snake_case, non_upper_case_globals)]
use super::native_overflow_builtins::BuiltinTriple;
use super::safe_conversions_impl::{Cast, Input, Integer, Numeric, Promote};
use core::{marker::PhantomData, mem::MaybeUninit, ops::Mul};

// Source: base/numerics/safe_math_clang_gcc_impl.h:17-22
#[cfg(base_numeric_armel)]
pub use super::safe_math_arm_impl::{
    CheckedMulFastAsmOp, ClampedAddFastAsmOp, ClampedMulFastAsmOp, ClampedSubFastAsmOp,
};
// Source: base/numerics/safe_math_clang_gcc_impl.h:30-73
#[cfg(not(base_numeric_armel))]
mod unavailable_asm {
    use super::super::safe_conversions_impl::CheckOnFailure;
    use core::marker::PhantomData;
    pub struct CheckedMulFastAsmOp<T, U>(PhantomData<(T, U)>);
    impl<T, U> CheckedMulFastAsmOp<T, U> {
        pub const is_supported: bool = false;
        pub unsafe fn Do<V>(_: T, _: U, _: *mut V) -> bool {
            CheckOnFailure::HandleFailure::<bool>()
        }
    }
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
    clamped!(ClampedAddFastAsmOp);
    clamped!(ClampedSubFastAsmOp);
    clamped!(ClampedMulFastAsmOp);
}
#[cfg(not(base_numeric_armel))]
pub use unavailable_asm::*;

// Source: base/numerics/safe_math_clang_gcc_impl.h:76-83
pub struct CheckedAddFastOp<T, U>(PhantomData<(T, U)>);
impl<T, U> CheckedAddFastOp<T, U> {
    pub const is_supported: bool = true;
    #[inline(always)]
    pub unsafe fn Do<V>(x: T, y: U, result: *mut V) -> bool
    where
        T: BuiltinTriple<U, V>,
    {
        unsafe { <T as BuiltinTriple<U, V>>::add(x, y, result) }
    }
}
// Source: base/numerics/safe_math_clang_gcc_impl.h:85-92
pub struct CheckedSubFastOp<T, U>(PhantomData<(T, U)>);
impl<T, U> CheckedSubFastOp<T, U> {
    pub const is_supported: bool = true;
    #[inline(always)]
    pub unsafe fn Do<V>(x: T, y: U, result: *mut V) -> bool
    where
        T: BuiltinTriple<U, V>,
    {
        unsafe { <T as BuiltinTriple<U, V>>::sub(x, y, result) }
    }
}

// Source: base/numerics/safe_math_clang_gcc_impl.h:94-119
pub struct CheckedMulFastOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Cast<isize> + Cast<usize>, U: Integer + Cast<isize> + Cast<usize>>
    CheckedMulFastOp<T, U>
{
    #[cfg(base_numeric_compiler_clang)]
    pub const is_supported: bool = (<T as Cast<isize>>::CONTAINED && <U as Cast<isize>>::CONTAINED)
        || (<T as Cast<usize>>::CONTAINED && <U as Cast<usize>>::CONTAINED);
    #[cfg(all(not(base_numeric_compiler_clang), base_numeric_compiler_gcc))]
    pub const is_supported: bool = true;
}
impl<T: Integer + Promote<U>, U: Integer> CheckedMulFastOp<T, U> {
    #[inline(always)]
    pub unsafe fn Do<V: Numeric>(x: T, y: U, result: *mut V) -> bool
    where
        T: BuiltinTriple<U, V> + Cast<<T as Promote<U>>::Fast>,
        U: Cast<<T as Promote<U>>::Fast>,
        <T as Promote<U>>::Fast: Mul<Output = <T as Promote<U>>::Fast> + Cast<V>,
    {
        if CheckedMulFastAsmOp::<T, U>::is_supported {
            unsafe { CheckedMulFastAsmOp::<T, U>::Do(x, y, result) }
        } else {
            unsafe { <T as BuiltinTriple<U, V>>::mul(x, y, result) }
        }
    }
}

// Source: base/numerics/safe_math_clang_gcc_impl.h:121-128
pub struct ClampedAddFastOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Promote<U>, U: Integer> ClampedAddFastOp<T, U>
where
    <T as Promote<U>>::Big: Cast<i32>,
{
    pub const is_supported: bool = ClampedAddFastAsmOp::<T, U>::is_supported;
}
impl<T: Integer + Input<Value = T> + Cast<i32>, U: Integer + Input<Value = U> + Cast<i32>>
    ClampedAddFastOp<T, U>
{
    #[inline(always)]
    pub fn Do<V: Numeric>(x: T, y: U) -> V
    where
        i32: Cast<V> + Input<Value = i32>,
    {
        ClampedAddFastAsmOp::<T, U>::Do::<V>(x, y)
    }
}
// Source: base/numerics/safe_math_clang_gcc_impl.h:130-137
pub struct ClampedSubFastOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Promote<U>, U: Integer> ClampedSubFastOp<T, U>
where
    <T as Promote<U>>::Big: Cast<i32>,
{
    pub const is_supported: bool = ClampedSubFastAsmOp::<T, U>::is_supported;
}
impl<T: Integer + Input<Value = T> + Cast<i32>, U: Integer + Input<Value = U> + Cast<i32>>
    ClampedSubFastOp<T, U>
{
    #[inline(always)]
    pub fn Do<V: Numeric>(x: T, y: U) -> V
    where
        i32: Cast<V> + Input<Value = i32>,
    {
        ClampedSubFastAsmOp::<T, U>::Do::<V>(x, y)
    }
}
// Source: base/numerics/safe_math_clang_gcc_impl.h:139-146
pub struct ClampedMulFastOp<T, U>(PhantomData<(T, U)>);
impl<T: Integer + Promote<U>, U: Integer> ClampedMulFastOp<T, U> {
    pub const is_supported: bool = ClampedMulFastAsmOp::<T, U>::is_supported;
    #[inline(always)]
    pub fn Do<V: Numeric>(x: T, y: U) -> V
    where
        T: Cast<<T as Promote<U>>::Fast>,
        U: Cast<<T as Promote<U>>::Fast>,
        <T as Promote<U>>::Fast: Mul<Output = <T as Promote<U>>::Fast>
            + Cast<V>
            + Input<Value = <T as Promote<U>>::Fast>,
    {
        ClampedMulFastAsmOp::<T, U>::Do::<V>(x, y)
    }
}

// Source: base/numerics/safe_math_clang_gcc_impl.h:148-169
pub struct ClampedNegFastOp<T>(PhantomData<T>);
pub trait NegFastDispatch: Integer {
    fn clamped_neg_fast(self) -> Self;
}
impl<T: NegFastDispatch> ClampedNegFastOp<T> {
    pub const is_supported: bool = T::SIGNED;
    #[inline(always)]
    pub fn Do(value: T) -> T {
        value.clamped_neg_fast()
    }
}
macro_rules! neg_dispatch {($($t:ty),* $(,)?)=>{$(
    impl NegFastDispatch for $t {
        #[inline(always)]
        fn clamped_neg_fast(self)->Self {
            if !ClampedSubFastAsmOp::<Self,Self>::is_supported {
                let mut result=MaybeUninit::<Self>::uninit();
                if unsafe {<Self as BuiltinTriple<Self,Self>>::sub(Self::ZERO,self,result.as_mut_ptr())} {
                    unsafe {result.assume_init()}
                } else {Self::MAX}
            } else {ClampedSubFastOp::<Self,Self>::Do::<Self>(Self::ZERO,self)}
        }
    }
)*}}
neg_dispatch!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

#[cfg(not(base_numeric_native_inputs_configured))]
compile_error!("native safe-math input configuration not supplied");
#[cfg(not(any(base_numeric_compiler_clang, base_numeric_compiler_gcc)))]
compile_error!("Only the actual native Clang/GCC builtin domain is mapped here");

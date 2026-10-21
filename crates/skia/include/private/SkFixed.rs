use crate::include::private::SkTFitsIn::SkCastFrom;
use crate::src::core::SkMathPriv::{SkCppAdd, SkCppDiv, SkCppMul, SkCppShr, SkCppSub};
macro_rules! native_const_assert {
 ($condition:expr,$line:expr,$text:expr)=>{{
  #[cfg(sk_debug)] {
   const fn at_const(value:bool){assert!(value);}
   fn at_runtime(value:bool){crate::SkASSERT!(@source b"include/private/SkFixed.h\0".as_ptr(),$line; @condition $text; value);}
   core::intrinsics::const_eval_select(($condition,),at_const,at_runtime)
  }
 }};
}
use crate::src::core::SkFloatingPoint::{sk_float_saturate2int, sk_float_saturate2int64};

// Source: include/private/SkFixed.h:25-34
pub type SkFixed = i32;
pub const SK_Fixed1: i32 = 1 << 16;
pub const SK_FixedHalf: i32 = 1 << 15;
pub const SK_FixedQuarter: i32 = 1 << 14;
pub const SK_FixedMax: i32 = 0x7fffffff;
pub const SK_FixedMin: i32 = -SK_FixedMax;
pub const SK_FixedPI: i32 = 0x3243f;
pub const SK_FixedSqrt2: i32 = 92682;
pub const SK_FixedTanPIOver8: i32 = 0x6a0a;
pub const SK_FixedRoot2Over2: i32 = 0xb505;

// Source: include/private/SkFixed.h:41-41
#[inline]
pub const fn SkFixedToFloat<T: Copy + [const] SkCppMul<f32>>(x: T) -> <T as SkCppMul<f32>>::Output {
    x.cpp_mul(&1.52587890625e-5f32)
}

// Source: include/private/SkFixed.h:42-42
#[cfg(not(sk_native_arm_vfpv3))]
#[inline]
pub const fn SkFloatToFixed<T>(x: T) -> SkFixed
where
    T: Copy + [const] SkCppMul<i32>,
    f32: [const] SkCastFrom<<T as SkCppMul<i32>>::Output>,
{
    sk_float_saturate2int(f32::sk_cast_from(x.cpp_mul(&SK_Fixed1)))
}

// Source: include/private/SkFixed.h:45-50
#[cfg(sk_debug)]
#[inline]
pub const fn SkFloatToFixed_Check(x: f32) -> SkFixed {
    let n64 = (x * SK_Fixed1 as f32) as i64;
    let n32 = n64 as i32;
    native_const_assert!(n64 == n32 as i64, 48, b"n64 == n32\0".as_ptr());
    n32
}

// Source: include/private/SkFixed.h:52-52
#[cfg(all(not(sk_debug), not(sk_native_arm_vfpv3)))]
#[inline]
pub const fn SkFloatToFixed_Check<T>(x: T) -> SkFixed
where
    T: [const] SkCppMul<i32>,
    f32: [const] SkCastFrom<<T as SkCppMul<i32>>::Output>,
{
    SkFloatToFixed(x)
}

// Source: include/private/SkFixed.h:55-55
#[inline]
pub const fn SkFixedToDouble<T: Copy + [const] SkCppMul<f64>>(
    x: T,
) -> <T as SkCppMul<f64>>::Output {
    x.cpp_mul(&1.52587890625e-5f64)
}

// Source: include/private/SkFixed.h:56-56
#[inline]
pub const fn SkDoubleToFixed<T>(x: T) -> SkFixed
where
    T: Copy + [const] SkCppMul<i32>,
    i32: [const] SkCastFrom<<T as SkCppMul<i32>>::Output>,
{
    i32::sk_cast_from(x.cpp_mul(&SK_Fixed1))
}

// Source: include/private/SkFixed.h:62-68
#[cfg(sk_debug)]
#[inline]
pub const fn SkIntToFixed(n: i32) -> SkFixed {
    native_const_assert!(
        n >= -32768 && n <= 32767,
        64,
        b"n >= -32768 && n <= 32767\0".as_ptr()
    );
    (n as u32).wrapping_shl(16) as i32
}

// Source: include/private/SkFixed.h:73-73
#[cfg(not(sk_debug))]
#[inline]
pub const fn SkIntToFixed<T>(n: T) -> SkFixed
where
    u32: [const] SkCastFrom<T>,
{
    (u32::sk_cast_from(n) << 16) as SkFixed
}

// Source: include/private/SkFixed.h:76-76
#[inline]
pub const fn SkFixedRoundToInt<T>(x: T) -> <<T as SkCppAdd<i32>>::Output as SkCppShr<i32>>::Output
where
    T: Copy + [const] SkCppAdd<i32>,
    <T as SkCppAdd<i32>>::Output: Copy + [const] SkCppShr<i32>,
{
    x.cpp_add(&SK_FixedHalf).cpp_shr(&16)
}

// Source: include/private/SkFixed.h:77-77
#[inline]
pub const fn SkFixedCeilToInt<T>(
    x: T,
) -> <<<T as SkCppAdd<i32>>::Output as SkCppSub<i32>>::Output as SkCppShr<i32>>::Output
where
    T: Copy + [const] SkCppAdd<i32>,
    <T as SkCppAdd<i32>>::Output: Copy + [const] SkCppSub<i32>,
    <<T as SkCppAdd<i32>>::Output as SkCppSub<i32>>::Output: Copy + [const] SkCppShr<i32>,
{
    x.cpp_add(&SK_Fixed1).cpp_sub(&1).cpp_shr(&16)
}

// Source: include/private/SkFixed.h:78-78
#[inline]
pub const fn SkFixedFloorToInt<T: Copy + [const] SkCppShr<i32>>(
    x: T,
) -> <T as SkCppShr<i32>>::Output {
    x.cpp_shr(&16)
}

// Source: include/private/SkFixed.h:80-82
#[inline]
pub const fn SkFixedRoundToFixed(x: SkFixed) -> SkFixed {
    (x.wrapping_add(SK_FixedHalf) as u32 & 0xffff0000) as i32
}

// Source: include/private/SkFixed.h:83-85
#[inline]
pub const fn SkFixedCeilToFixed(x: SkFixed) -> SkFixed {
    (x.wrapping_add(SK_Fixed1).wrapping_sub(1) as u32 & 0xffff0000) as i32
}

// Source: include/private/SkFixed.h:86-88
#[inline]
pub const fn SkFixedFloorToFixed(x: SkFixed) -> SkFixed {
    (x as u32 & 0xffff0000) as i32
}

// Source: include/private/SkFixed.h:90-90
#[inline]
pub const fn SkFixedAve<L, R>(a: L, b: R) -> <<L as SkCppAdd<R>>::Output as SkCppShr<i32>>::Output
where
    L: Copy + [const] SkCppAdd<R>,
    R: Copy,
    <L as SkCppAdd<R>>::Output: Copy + [const] SkCppShr<i32>,
{
    a.cpp_add(&b).cpp_shr(&1)
}

// Source: include/private/SkFixed.h:93-94
#[inline]
pub const fn SkFixedDiv<N, D>(numer: N, denom: D) -> SkFixed
where
    N: Copy,
    D: Copy,
    i64: [const] SkCastFrom<N>
        + [const] SkCppDiv<D>
        + [const] SkCastFrom<<i64 as SkCppDiv<D>>::Output>,
{
    let numer = i64::sk_cast_from(numer).wrapping_shl(16);
    let quotient = i64::sk_cast_from(numer.cpp_div(&denom));
    crate::include::private::SkTo::SkToS32(crate::include::private::SkTPin::SkTPin(
        quotient,
        i32::MIN as i64,
        i32::MAX as i64,
    ))
}

// Source: include/private/SkFixed.h:96-98
#[inline]
pub const fn SkFixedMul(a: SkFixed, b: SkFixed) -> SkFixed {
    ((a as i64 * b as i64) >> 16) as i32
}

// Source: include/private/SkFixed.h:104-120.
// Actual __ARM_VFPV3__ compiler input; this runtime-only ISA override is not
// claimed constexpr. ARM sreg holds the same float register as native +w.
#[cfg(all(sk_native_arm_vfpv3, target_arch = "arm"))]
#[inline(always)]
pub fn SkFloatToFixed_arm(mut x: f32) -> SkFixed {
    unsafe {
        core::arch::asm!("vcvt.s32.f32 {x}, {x}, #16",x=inout(sreg)x,options(nomem,nostack));
    }
    x.to_bits() as i32
}
#[cfg(all(sk_native_arm_vfpv3, target_arch = "arm"))]
pub fn SkFloatToFixed<T>(x: T) -> SkFixed
where
    f32: SkCastFrom<T>,
{
    SkFloatToFixed_arm(f32::sk_cast_from(x))
}
#[cfg(all(sk_native_arm_vfpv3, not(target_arch = "arm")))]
compile_error!("native __ARM_VFPV3__ requires the actual ARM VFP register/instruction target");
#[cfg(all(not(sk_debug), sk_native_arm_vfpv3, target_arch = "arm"))]
pub fn SkFloatToFixed_Check<T>(x: T) -> SkFixed
where
    f32: SkCastFrom<T>,
{
    SkFloatToFixed(x)
}

// Source: include/private/SkFixed.h:124-124
pub use SkFixedToFloat as SkFixedToScalar;

// Source: include/private/SkFixed.h:125-125
pub use SkFloatToFixed as SkScalarToFixed;

// Source: include/private/SkFixed.h:129-141
pub type SkFixed3232 = i64;
pub const SkFixed3232Max: i64 = i64::MAX;
pub const SkFixed3232Min: i64 = -SkFixed3232Max;
#[inline]
pub const fn SkIntToFixed3232<T>(x: T) -> SkFixed3232
where
    i64: [const] SkCastFrom<T>,
{
    i64::sk_cast_from(x).wrapping_shl(32)
}
#[inline]
pub const fn SkFixed3232ToInt<T>(x: T) -> i32
where
    T: Copy + [const] SkCppShr<i32>,
    i32: [const] SkCastFrom<<T as SkCppShr<i32>>::Output>,
{
    i32::sk_cast_from(x.cpp_shr(&32))
}
#[inline]
pub const fn SkFixedToFixed3232<T>(x: T) -> SkFixed3232
where
    i64: [const] SkCastFrom<T>,
{
    i64::sk_cast_from(x).wrapping_shl(16)
}
#[inline]
pub const fn SkFixed3232ToFixed<T>(x: T) -> SkFixed
where
    T: Copy + [const] SkCppShr<i32>,
    i32: [const] SkCastFrom<<T as SkCppShr<i32>>::Output>,
{
    i32::sk_cast_from(x.cpp_shr(&16))
}
#[inline]
pub const fn SkFloatToFixed3232<T>(x: T) -> SkFixed3232
where
    T: Copy + [const] SkCppMul<f32>,
    f32: [const] SkCastFrom<<T as SkCppMul<f32>>::Output>,
{
    sk_float_saturate2int64(f32::sk_cast_from(x.cpp_mul(&(65536.0f32 * 65536.0f32))))
}
#[inline]
pub const fn SkFixed3232ToFloat<T: Copy + [const] SkCppMul<f32>>(
    x: T,
) -> <T as SkCppMul<f32>>::Output {
    x.cpp_mul(&(1.0f32 / (65536.0f32 * 65536.0f32)))
}
pub use SkFloatToFixed3232 as SkScalarToFixed3232;

// Native preprocessor operand language entrypoints. `@with ...; |result| body`
// includes the caller's complete full-expression in the scope of all genuine
// borrowed source operands and operator temporaries, preserving non-Copy owners
// and reverse temporary destruction. The short arm is the isolated-expression
// value form. No custom class is implicitly moved or cloned to read an operand.
// Source: include/private/SkFixed.h:41.
#[macro_export]
macro_rules! SkFixedToFloat {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppMul::cpp_mul(__x,&1.52587890625e-5f32);let $result=__a;$body}};
 ($x:expr)=>{$crate::SkFixedToFloat!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:55.
#[macro_export]
macro_rules! SkFixedToDouble {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppMul::cpp_mul(__x,&1.52587890625e-5f64);let $result=__a;$body}};
 ($x:expr)=>{$crate::SkFixedToDouble!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:56.
#[macro_export]
macro_rules! SkDoubleToFixed {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppMul::cpp_mul(__x,&$crate::include::private::SkFixed::SK_Fixed1);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__a);$body}};
 ($x:expr)=>{$crate::SkDoubleToFixed!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:42.
#[macro_export]
macro_rules! SkFloatToFixed {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;#[cfg(not(sk_native_arm_vfpv3))] let __a=$crate::src::core::SkMathPriv::SkCppMul::cpp_mul(__x,&$crate::include::private::SkFixed::SK_Fixed1);
 #[cfg(not(sk_native_arm_vfpv3))] let $result=$crate::src::core::SkFloatingPoint::sk_float_saturate2int($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(&__a));
 #[cfg(sk_native_arm_vfpv3)] let $result=$crate::include::private::SkFixed::SkFloatToFixed_arm($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(__x));$body}};
 ($x:expr)=>{$crate::SkFloatToFixed!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:52.
#[macro_export]
macro_rules! SkFloatToFixed_Check {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;#[cfg(sk_debug)] let $result=$crate::include::private::SkFixed::SkFloatToFixed_Check($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(__x));
 #[cfg(all(not(sk_debug),not(sk_native_arm_vfpv3)))] let __a=$crate::src::core::SkMathPriv::SkCppMul::cpp_mul(__x,&$crate::include::private::SkFixed::SK_Fixed1);
 #[cfg(all(not(sk_debug),not(sk_native_arm_vfpv3)))] let $result=$crate::src::core::SkFloatingPoint::sk_float_saturate2int($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(&__a));
 #[cfg(all(not(sk_debug),sk_native_arm_vfpv3))] let $result=$crate::include::private::SkFixed::SkFloatToFixed_arm($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(__x));$body}};
 ($x:expr)=>{$crate::SkFloatToFixed_Check!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:73.
#[macro_export]
macro_rules! SkIntToFixed {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;#[cfg(sk_debug)] let $result=$crate::include::private::SkFixed::SkIntToFixed($crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(__x));
 #[cfg(not(sk_debug))] let $result=($crate::src::core::SkMathPriv::SkCppConvertTo::<u32>::cpp_convert_to(__x)<<16) as i32;$body}};
 ($x:expr)=>{$crate::SkIntToFixed!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:76.
#[macro_export]
macro_rules! SkFixedRoundToInt {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppAdd::cpp_add(__x,&$crate::include::private::SkFixed::SK_FixedHalf);let $result=$crate::src::core::SkMathPriv::SkCppShr::cpp_shr(&__a,&16i32);$body}};
 ($x:expr)=>{$crate::SkFixedRoundToInt!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:77.
#[macro_export]
macro_rules! SkFixedCeilToInt {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppAdd::cpp_add(__x,&$crate::include::private::SkFixed::SK_Fixed1);let __b=$crate::src::core::SkMathPriv::SkCppSub::cpp_sub(&__a,&1i32);let $result=$crate::src::core::SkMathPriv::SkCppShr::cpp_shr(&__b,&16i32);$body}};
 ($x:expr)=>{$crate::SkFixedCeilToInt!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:78.
#[macro_export]
macro_rules! SkFixedFloorToInt {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let $result=$crate::src::core::SkMathPriv::SkCppShr::cpp_shr(__x,&16i32);$body}};
 ($x:expr)=>{$crate::SkFixedFloorToInt!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:90.
#[macro_export]
macro_rules! SkFixedAve {
 (@with $a:expr,$b:expr; |$result:ident| $body:expr)=>{{let __a=&$a;let __b=&$b;let __sum=$crate::src::core::SkMathPriv::SkCppAdd::cpp_add(__a,__b);let $result=$crate::src::core::SkMathPriv::SkCppShr::cpp_shr(&__sum,&1i32);$body}};
 ($a:expr,$b:expr)=>{$crate::SkFixedAve!(@with $a,$b; |__result| __result)};
}
// Source: include/private/SkFixed.h:93.
#[macro_export]
macro_rules! SkFixedDiv {
 (@with $numer:expr,$denom:expr; |$result:ident| $body:expr)=>{{let __numer=&$numer;let __denom=&$denom;let __shifted=$crate::src::core::SkMathPriv::SkCppConvertTo::<i64>::cpp_convert_to(__numer).wrapping_shl(16);
 let __q=$crate::src::core::SkMathPriv::SkCppDiv::cpp_div(&__shifted,__denom);
 let __converted=$crate::src::core::SkMathPriv::SkCppConvertTo::<i64>::cpp_convert_to(&__q);
 let $result=$crate::include::private::SkTo::SkToS32($crate::include::private::SkTPin::SkTPin(__converted,i32::MIN as i64,i32::MAX as i64));$body}};
 ($numer:expr,$denom:expr)=>{$crate::SkFixedDiv!(@with $numer,$denom; |__result| __result)};
}
// Source: include/private/SkFixed.h:134.
#[macro_export]
macro_rules! SkIntToFixed3232 {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i64>::cpp_convert_to(__x).wrapping_shl(32);$body}};
 ($x:expr)=>{$crate::SkIntToFixed3232!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:135.
#[macro_export]
macro_rules! SkFixed3232ToInt {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppShr::cpp_shr(__x,&32i32);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__a);$body}};
 ($x:expr)=>{$crate::SkFixed3232ToInt!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:136.
#[macro_export]
macro_rules! SkFixedToFixed3232 {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i64>::cpp_convert_to(__x).wrapping_shl(16);$body}};
 ($x:expr)=>{$crate::SkFixedToFixed3232!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:137.
#[macro_export]
macro_rules! SkFixed3232ToFixed {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppShr::cpp_shr(__x,&16i32);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__a);$body}};
 ($x:expr)=>{$crate::SkFixed3232ToFixed!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:138.
#[macro_export]
macro_rules! SkFloatToFixed3232 {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __a=$crate::src::core::SkMathPriv::SkCppMul::cpp_mul(__x,&(65536.0f32*65536.0f32));let $result=$crate::src::core::SkFloatingPoint::sk_float_saturate2int64($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(&__a));$body}};
 ($x:expr)=>{$crate::SkFloatToFixed3232!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:139.
#[macro_export]
macro_rules! SkFixed3232ToFloat {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let $result=$crate::src::core::SkMathPriv::SkCppMul::cpp_mul(__x,&(1.0f32/(65536.0f32*65536.0f32)));$body}};
 ($x:expr)=>{$crate::SkFixed3232ToFloat!(@with $x; |__result| __result)};
}
// Source: include/private/SkFixed.h:124.
#[macro_export]
macro_rules! SkFixedToScalar{($($tokens:tt)*)=>{$crate::SkFixedToFloat!($($tokens)*)};}
// Source: include/private/SkFixed.h:125.
#[macro_export]
macro_rules! SkScalarToFixed{($($tokens:tt)*)=>{$crate::SkFloatToFixed!($($tokens)*)};}
// Source: include/private/SkFixed.h:141.
#[macro_export]
macro_rules! SkScalarToFixed3232{($($tokens:tt)*)=>{$crate::SkFloatToFixed3232!($($tokens)*)};}

// Genuine native long-double operations use the compiler-owned stack slots from
// text6's bridge. No Rust long-double layout/value ABI or constexpr is asserted.
#[cfg(sk_native_long_double_ffi)]
pub mod long_double {
    use crate::interop::native_long_double as ld;
    use crate::src::core::SkFloatingPoint::{sk_float_saturate2int, sk_float_saturate2int64};
    // Source: include/private/SkFixed.h:41.
    /// Safety: continuation does not unwind through the native frame or let its temporary lease escape.
    pub unsafe fn with_SkFixedToFloat<R, F>(x: ld::ValueRef<'_>, f: F) -> R
    where
        F: for<'a> FnOnce(ld::ValueRef<'a>) -> R,
    {
        unsafe {
            ld::with_slots(|[mut scale, mut result, _, _]| {
                scale.from_f32(1.52587890625e-5f32);
                result.mul(x, scale.as_ref());
                f(result.as_ref())
            })
        }
    }
    // Source: include/private/SkFixed.h:42.
    /// Safety: no unwind across the native primitive callback.
    pub unsafe fn SkFloatToFixed(x: ld::ValueRef<'_>) -> i32 {
        #[cfg(not(sk_native_arm_vfpv3))]
        {
            unsafe {
                ld::with_slots(|[mut scale, mut result, _, _]| {
                    scale.from_i64(super::SK_Fixed1 as i64);
                    result.mul(x, scale.as_ref());
                    sk_float_saturate2int(result.as_ref().to_f32())
                })
            }
        }
        #[cfg(sk_native_arm_vfpv3)]
        {
            super::SkFloatToFixed_arm(x.to_f32())
        }
    }
    // Source: include/private/SkFixed.h:55.
    /// Safety: continuation does not unwind or let the temporary lease escape.
    pub unsafe fn with_SkFixedToDouble<R, F>(x: ld::ValueRef<'_>, f: F) -> R
    where
        F: for<'a> FnOnce(ld::ValueRef<'a>) -> R,
    {
        unsafe {
            ld::with_slots(|[mut scale, mut result, _, _]| {
                scale.from_f64(1.52587890625e-5f64);
                result.mul(x, scale.as_ref());
                f(result.as_ref())
            })
        }
    }
    // Source: include/private/SkFixed.h:56.
    /// Safety: native integral cast result is representable as int; no unwind.
    pub unsafe fn SkDoubleToFixed(x: ld::ValueRef<'_>) -> i32 {
        unsafe {
            ld::with_slots(|[mut scale, mut result, _, _]| {
                scale.from_i64(super::SK_Fixed1 as i64);
                result.mul(x, scale.as_ref());
                result.as_ref().to_i32()
            })
        }
    }
    // Source: include/private/SkFixed.h:93.
    /// Safety: original numerator/quotient int64 conversions and division are defined; no unwind.
    pub unsafe fn SkFixedDiv(numer: ld::ValueRef<'_>, denom: ld::ValueRef<'_>) -> i32 {
        unsafe {
            ld::with_slots(|[mut shifted, mut result, _, _]| {
                shifted.from_i64(numer.to_i64().wrapping_shl(16));
                result.div(shifted.as_ref(), denom);
                crate::include::private::SkTo::SkToS32(crate::include::private::SkTPin::SkTPin(
                    result.as_ref().to_i64(),
                    i32::MIN as i64,
                    i32::MAX as i64,
                ))
            })
        }
    }
    // Source: include/private/SkFixed.h:138.
    /// Safety: no unwind across the actual compiler primitive frame.
    pub unsafe fn SkFloatToFixed3232(x: ld::ValueRef<'_>) -> i64 {
        unsafe {
            ld::with_slots(|[mut scale, mut result, _, _]| {
                scale.from_f32(65536.0f32 * 65536.0f32);
                result.mul(x, scale.as_ref());
                sk_float_saturate2int64(result.as_ref().to_f32())
            })
        }
    }
    // Source: include/private/SkFixed.h:139.
    /// Safety: continuation does not unwind or let the temporary lease escape.
    pub unsafe fn with_SkFixed3232ToFloat<R, F>(x: ld::ValueRef<'_>, f: F) -> R
    where
        F: for<'a> FnOnce(ld::ValueRef<'a>) -> R,
    {
        unsafe {
            ld::with_slots(|[mut scale, mut result, _, _]| {
                scale.from_f32(1.0f32 / (65536.0f32 * 65536.0f32));
                result.mul(x, scale.as_ref());
                f(result.as_ref())
            })
        }
    }
}

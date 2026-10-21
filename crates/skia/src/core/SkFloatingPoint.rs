#![allow(non_snake_case)]

use crate::include::private::SkTFitsIn::SkCastFrom;
use crate::src::core::SkMathPriv::{SkCppAdd, SkCppMul};
// Source: include/private/SkFloatingPoint.h:118-129.
// Actual std overload resolution language protocol. Integral overloads promote
// to double; real floating overloads keep float/double/actual long-double type.
// A translated user class supplies only its genuine available conversion/overload.
pub trait SkCppFloor {
    type Output;
    fn cpp_floor(&self) -> Self::Output;
}
pub trait SkCppCeil {
    type Output;
    fn cpp_ceil(&self) -> Self::Output;
}
pub trait SkCppRound {
    type Output;
    fn cpp_round(&self) -> Self::Output;
}
macro_rules! cpp_rounding_float {
    ($t:ty) => {
        impl SkCppFloor for $t {
            type Output = $t;
            fn cpp_floor(&self) -> $t {
                self.floor()
            }
        }
        impl SkCppCeil for $t {
            type Output = $t;
            fn cpp_ceil(&self) -> $t {
                self.ceil()
            }
        }
        impl SkCppRound for $t {
            type Output = $t;
            fn cpp_round(&self) -> $t {
                self.round()
            }
        }
    };
}
cpp_rounding_float!(f32);
cpp_rounding_float!(f64);
macro_rules! cpp_rounding_integer {
 ($($t:ty),*)=>{$(
  impl SkCppFloor for $t{type Output=f64;fn cpp_floor(&self)->f64{(*self as f64).floor()}}
  impl SkCppCeil for $t{type Output=f64;fn cpp_ceil(&self)->f64{(*self as f64).ceil()}}
  impl SkCppRound for $t{type Output=f64;fn cpp_round(&self)->f64{(*self as f64).round()}}
 )*};
}
cpp_rounding_integer!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);
impl SkCppFloor for bool {
    type Output = f64;
    fn cpp_floor(&self) -> f64 {
        (*self as u8 as f64).floor()
    }
}
impl SkCppCeil for bool {
    type Output = f64;
    fn cpp_ceil(&self) -> f64 {
        (*self as u8 as f64).ceil()
    }
}
impl SkCppRound for bool {
    type Output = f64;
    fn cpp_round(&self) -> f64 {
        (*self as u8 as f64).round()
    }
}
// Source: include/private/SkFloatingPoint.h:40-40
// Native floating-type constraint specialized to this CPU closure's scalar float/double types.
mod native_float_type {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}
pub trait SkFloatingPointValue:
    native_float_type::Sealed
    + Copy
    + PartialEq
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
{
}
impl SkFloatingPointValue for f32 {}
impl SkFloatingPointValue for f64 {}

// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.

// Source: include/private/SkFloatingPoint.h:19-19
pub const SK_FloatSqrt2: f32 = 1.41421356;

// Source: include/private/SkFloatingPoint.h:20-20
pub const SK_FloatPI: f32 = 3.14159265;

// Source: include/private/SkFloatingPoint.h:21-21
pub const SK_DoublePI: f64 = 3.14159265358979323846264338327950288;

// Source: include/private/SkFloatingPoint.h:23-25
#[inline]
pub const fn sk_float_sgn(x: f32) -> i32 {
    (0.0 < x) as i32 - (x < 0.0) as i32
}

// Source: include/private/SkFloatingPoint.h:27-29
#[inline]
pub const fn sk_float_degrees_to_radians(degrees: f32) -> f32 {
    degrees * (SK_FloatPI / 180.0)
}

// Source: include/private/SkFloatingPoint.h:31-33
#[inline]
pub const fn sk_float_radians_to_degrees(radians: f32) -> f32 {
    radians * (180.0 / SK_FloatPI)
}

// Source: include/private/SkFloatingPoint.h:38-38
#[inline]
pub fn sk_float_round<T>(x: T) -> f32
where
    f64: SkCastFrom<T>,
{
    sk_double_round(f64::sk_cast_from(x)) as f32
}

// Source: include/private/SkFloatingPoint.h:40-43
#[inline]
pub const fn SkIsNaN<T: SkFloatingPointValue + [const] PartialEq>(x: T) -> bool {
    x != x
}

// Source: include/private/SkFloatingPoint.h:49-55
#[inline]
pub fn SkIsFinite<T>(values: &[T]) -> bool
where
    T: SkFloatingPointValue,
{
    let x = values[0];
    let mut prod = x - x;
    for &v in &values[1..] {
        prod = prod * v;
    }
    prod == prod
}

// Source: include/private/SkFloatingPoint.h:57-66
#[inline]
pub fn SkIsFiniteArray<T>(array: &[T]) -> bool
where
    T: SkFloatingPointValue,
{
    let x = array[0];
    let mut prod = x - x;
    for &v in &array[1..] {
        prod = prod * v;
    }
    prod == prod
}

// Source: include/private/SkFloatingPoint.h:68-68
pub const SK_MaxS32FitsInFloat: i32 = 2147483520;

// Source: include/private/SkFloatingPoint.h:69-69
pub const SK_MinS32FitsInFloat: i32 = -SK_MaxS32FitsInFloat;

// Source: include/private/SkFloatingPoint.h:72-72
pub const SK_MaxS64FitsInFloat: i64 = (i64::MAX >> (63 - 24)) << (63 - 24);

// Source: include/private/SkFloatingPoint.h:73-73
pub const SK_MinS64FitsInFloat: i64 = -SK_MaxS64FitsInFloat;

// Source: include/private/SkFloatingPoint.h:89-94
#[inline]
pub const fn sk_float_saturate2int(mut x: f32) -> i32 {
    #[cfg(sk_native_nonclang_msvc)]
    if SkIsNaN(x) {
        return SK_MaxS32FitsInFloat;
    }
    x = if x < SK_MaxS32FitsInFloat as f32 {
        x
    } else {
        SK_MaxS32FitsInFloat as f32
    };
    x = if x > SK_MinS32FitsInFloat as f32 {
        x
    } else {
        SK_MinS32FitsInFloat as f32
    };
    x as i32
}

// Source: include/private/SkFloatingPoint.h:99-104
#[inline]
pub const fn sk_double_saturate2int(mut x: f64) -> i32 {
    #[cfg(sk_native_nonclang_msvc)]
    if SkIsNaN(x) {
        return i32::MAX;
    }
    x = if x < i32::MAX as f64 {
        x
    } else {
        i32::MAX as f64
    };
    x = if x > i32::MIN as f64 {
        x
    } else {
        i32::MIN as f64
    };
    x as i32
}

// Source: include/private/SkFloatingPoint.h:109-114
#[inline]
pub const fn sk_float_saturate2int64(mut x: f32) -> i64 {
    #[cfg(sk_native_nonclang_msvc)]
    if SkIsNaN(x) {
        return SK_MaxS64FitsInFloat;
    }
    x = if x < SK_MaxS64FitsInFloat as f32 {
        x
    } else {
        SK_MaxS64FitsInFloat as f32
    };
    x = if x > SK_MinS64FitsInFloat as f32 {
        x
    } else {
        SK_MinS64FitsInFloat as f32
    };
    x as i64
}

// Source: include/private/SkFloatingPoint.h:118-118
#[inline]
pub fn sk_float_floor2int<T: SkCppFloor>(x: T) -> i32
where
    f32: SkCastFrom<T::Output>,
{
    sk_float_saturate2int(f32::sk_cast_from(x.cpp_floor()))
}

// Source: include/private/SkFloatingPoint.h:119-119
#[inline]
pub fn sk_float_round2int<T>(x: T) -> i32
where
    f64: SkCastFrom<T>,
{
    sk_float_saturate2int(sk_float_round(x))
}

// Source: include/private/SkFloatingPoint.h:120-120
#[inline]
pub fn sk_float_ceil2int<T: SkCppCeil>(x: T) -> i32
where
    f32: SkCastFrom<T::Output>,
{
    sk_float_saturate2int(f32::sk_cast_from(x.cpp_ceil()))
}

// Source: include/private/SkFloatingPoint.h:122-122
#[inline]
pub fn sk_float_floor2int_no_saturate<T: SkCppFloor>(x: T) -> i32
where
    i32: SkCastFrom<T::Output>,
{
    i32::sk_cast_from(x.cpp_floor())
}

// Source: include/private/SkFloatingPoint.h:123-123
#[inline]
pub fn sk_float_round2int_no_saturate<T>(x: T) -> i32
where
    f64: SkCastFrom<T>,
{
    sk_float_round(x) as i32
}

// Source: include/private/SkFloatingPoint.h:124-124
#[inline]
pub fn sk_float_ceil2int_no_saturate<T: SkCppCeil>(x: T) -> i32
where
    i32: SkCastFrom<T::Output>,
{
    i32::sk_cast_from(x.cpp_ceil())
}

// Source: include/private/SkFloatingPoint.h:126-126
#[inline]
pub fn sk_double_round<T>(x: T) -> <<T as SkCppAdd<f64>>::Output as SkCppFloor>::Output
where
    T: SkCppAdd<f64>,
    <T as SkCppAdd<f64>>::Output: SkCppFloor,
{
    x.cpp_add(&0.5f64).cpp_floor()
}

// Source: include/private/SkFloatingPoint.h:127-127
#[inline]
pub fn sk_double_floor2int<T: SkCppFloor>(x: T) -> i32
where
    i32: SkCastFrom<T::Output>,
{
    i32::sk_cast_from(x.cpp_floor())
}

// Source: include/private/SkFloatingPoint.h:128-128
#[inline]
pub fn sk_double_round2int<T: SkCppRound>(x: T) -> i32
where
    i32: SkCastFrom<T::Output>,
{
    i32::sk_cast_from(x.cpp_round())
}

// Source: include/private/SkFloatingPoint.h:129-129
#[inline]
pub fn sk_double_ceil2int<T: SkCppCeil>(x: T) -> i32
where
    i32: SkCastFrom<T::Output>,
{
    i32::sk_cast_from(x.cpp_ceil())
}

// Source: include/private/SkFloatingPoint.h:135-137
#[inline]
pub const fn sk_double_to_float(x: f64) -> f32 {
    x as f32
}

// Source: include/private/SkFloatingPoint.h:139-139
pub const SK_FloatNaN: f32 = f32::NAN;

// Source: include/private/SkFloatingPoint.h:140-140
pub const SK_FloatInfinity: f32 = f32::INFINITY;

// Source: include/private/SkFloatingPoint.h:141-141
pub const SK_FloatNegativeInfinity: f32 = f32::NEG_INFINITY;

// Source: include/private/SkFloatingPoint.h:143-143
pub const SK_DoubleNaN: f64 = f64::NAN;

// Source: include/private/SkFloatingPoint.h:146-149
#[inline]
pub const fn sk_float_midpoint(a: f32, b: f32) -> f32 {
    (0.5 * (a as f64 + b as f64)) as f32
}

// Source: include/private/SkFloatingPoint.h:151-151
#[inline]
pub fn sk_float_rsqrt(x: f32) -> f32 {
    1.0 / x.sqrt()
}

// Source: include/private/SkFloatingPoint.h:160-162
#[inline]
pub const fn sk_ieee_float_divide(numer: f32, denom: f32) -> f32 {
    numer / denom
}

// Source: include/private/SkFloatingPoint.h:165-167
#[inline]
pub const fn sk_ieee_double_divide(numer: f64, denom: f64) -> f64 {
    numer / denom
}

// Source: src/core/SkFloatingPoint.cpp:19-28
fn magnitude(a: f64) -> f64 {
    const EXTRACT_MAGNITUDE: u64 = 0x7ff0000000000000;
    f64::from_bits(a.to_bits() & EXTRACT_MAGNITUDE)
}

// Source: src/core/SkFloatingPoint.cpp:30-52
pub fn sk_doubles_nearly_equal_ulps(a: f64, b: f64, max_ulps_diff: u8) -> bool {
    let a_mag = magnitude(a);
    let min_mag = f64::MIN_POSITIVE;
    let first = if a_mag < min_mag { min_mag } else { a_mag };
    let b_mag = magnitude(b);
    let max_magnitude = if first < b_mag { b_mag } else { first };
    let tolerance = max_magnitude * (f64::EPSILON * (max_ulps_diff as u32 + 1) as f64);
    a == b || (b - a).abs() < tolerance
}

// Source: src/core/SkFloatingPoint.cpp:54-56
pub fn sk_double_nearly_zero(a: f64) -> bool {
    a == 0.0 || a.abs() < f32::EPSILON as f64
}

// Source: include/private/SkFloatingPoint.h:49-55.
// C++ left-fold pack language projection. A recursive pair encodes arbitrary
// arity without a fixed maximum; each multiplication preserves its own promoted
// result type. Only the final assignment converts back to the original T.
pub trait SkFiniteMultiply<Rhs> {
    type Output;
    fn multiply(&self, rhs: &Rhs) -> Self::Output;
}
impl<L, R> SkFiniteMultiply<R> for L
where
    L: SkCppMul<R>,
{
    type Output = <L as SkCppMul<R>>::Output;
    fn multiply(&self, rhs: &R) -> Self::Output {
        self.cpp_mul(rhs)
    }
}
pub trait SkFinitePack<Prod> {
    type Output;
    fn fold_with<R, F: FnOnce(&Self::Output) -> R>(&self, prod: &Prod, f: F) -> R;
}
impl<Prod> SkFinitePack<Prod> for () {
    type Output = Prod;
    fn fold_with<R, F: FnOnce(&Prod) -> R>(&self, prod: &Prod, f: F) -> R {
        f(prod)
    }
}
impl<'operand, Prod, H, Tail> SkFinitePack<Prod> for (&'operand H, Tail)
where
    Prod: SkFiniteMultiply<H>,
    Tail: SkFinitePack<<Prod as SkFiniteMultiply<H>>::Output>,
{
    type Output = <Tail as SkFinitePack<<Prod as SkFiniteMultiply<H>>::Output>>::Output;
    fn fold_with<R, F: FnOnce(&Self::Output) -> R>(&self, prod: &Prod, f: F) -> R {
        let next = prod.multiply(self.0);
        self.1.fold_with(&next, f)
    }
}
pub fn SkIsFinitePack<T, Pack>(x: T, values: &Pack) -> bool
where
    T: SkFloatingPointValue + crate::src::core::SkMathPriv::SkCppAssignBorrowed<Pack::Output>,
    Pack: SkFinitePack<T>,
{
    let initial = x - x;
    let mut prod = initial;
    values.fold_with(&initial, |value| {
        crate::src::core::SkMathPriv::SkCppAssignBorrowed::cpp_assign_borrowed(&mut prod, value);
    });
    prod == prod
}
// Source: include/private/SkFloatingPoint.h:57-66. Exact pointer/count interface;
// caller preserves native readable array[0] even for count<=0, and every visited element.
pub unsafe fn SkIsFiniteArray_raw<T: SkFloatingPointValue>(array: *const T, count: i32) -> bool {
    let x = unsafe { *array };
    let mut prod = x - x;
    let mut i = 1;
    while i < count {
        prod = prod * unsafe { *array.add(i as usize) };
        i += 1;
    }
    prod == prod
}

// Macro full-expression form retains every real overload intermediate until caller continuation completes.
// Source: include/private/SkFloatingPoint.h:38.
#[macro_export]
macro_rules! sk_float_round {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __double=$crate::src::core::SkMathPriv::SkCppConvertTo::<f64>::cpp_convert_to(__x);let __sum=$crate::src::core::SkMathPriv::SkCppAdd::cpp_add(&__double,&0.5f64);let __rounded=$crate::src::core::SkFloatingPoint::SkCppFloor::cpp_floor(&__sum);let $result=__rounded as f32;$body}};
 ($x:expr)=>{$crate::sk_float_round!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:126.
#[macro_export]
macro_rules! sk_double_round {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __sum=$crate::src::core::SkMathPriv::SkCppAdd::cpp_add(__x,&0.5f64);let $result=$crate::src::core::SkFloatingPoint::SkCppFloor::cpp_floor(&__sum);$body}};
 ($x:expr)=>{$crate::sk_double_round!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:118.
#[macro_export]
macro_rules! sk_float_floor2int {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __rounded=$crate::src::core::SkFloatingPoint::SkCppFloor::cpp_floor(__x);let $result=$crate::src::core::SkFloatingPoint::sk_float_saturate2int($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(&__rounded));$body}};
 ($x:expr)=>{$crate::sk_float_floor2int!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:120.
#[macro_export]
macro_rules! sk_float_ceil2int {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __rounded=$crate::src::core::SkFloatingPoint::SkCppCeil::cpp_ceil(__x);let $result=$crate::src::core::SkFloatingPoint::sk_float_saturate2int($crate::src::core::SkMathPriv::SkCppConvertTo::<f32>::cpp_convert_to(&__rounded));$body}};
 ($x:expr)=>{$crate::sk_float_ceil2int!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:122.
#[macro_export]
macro_rules! sk_float_floor2int_no_saturate {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __rounded=$crate::src::core::SkFloatingPoint::SkCppFloor::cpp_floor(__x);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__rounded);$body}};
 ($x:expr)=>{$crate::sk_float_floor2int_no_saturate!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:124.
#[macro_export]
macro_rules! sk_float_ceil2int_no_saturate {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __rounded=$crate::src::core::SkFloatingPoint::SkCppCeil::cpp_ceil(__x);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__rounded);$body}};
 ($x:expr)=>{$crate::sk_float_ceil2int_no_saturate!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:127.
#[macro_export]
macro_rules! sk_double_floor2int {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __rounded=$crate::src::core::SkFloatingPoint::SkCppFloor::cpp_floor(__x);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__rounded);$body}};
 ($x:expr)=>{$crate::sk_double_floor2int!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:128.
#[macro_export]
macro_rules! sk_double_round2int {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __rounded=$crate::src::core::SkFloatingPoint::SkCppRound::cpp_round(__x);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__rounded);$body}};
 ($x:expr)=>{$crate::sk_double_round2int!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:129.
#[macro_export]
macro_rules! sk_double_ceil2int {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __rounded=$crate::src::core::SkFloatingPoint::SkCppCeil::cpp_ceil(__x);let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__rounded);$body}};
 ($x:expr)=>{$crate::sk_double_ceil2int!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:119.
#[macro_export]
macro_rules! sk_float_round2int {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __double=$crate::src::core::SkMathPriv::SkCppConvertTo::<f64>::cpp_convert_to(__x);let __sum=$crate::src::core::SkMathPriv::SkCppAdd::cpp_add(&__double,&0.5f64);let __rounded=$crate::src::core::SkFloatingPoint::SkCppFloor::cpp_floor(&__sum);let __float=__rounded as f32;let $result=$crate::src::core::SkFloatingPoint::sk_float_saturate2int(__float);$body}};
 ($x:expr)=>{$crate::sk_float_round2int!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:123.
#[macro_export]
macro_rules! sk_float_round2int_no_saturate {
 (@with $x:expr; |$result:ident| $body:expr)=>{{let __x=&$x;let __double=$crate::src::core::SkMathPriv::SkCppConvertTo::<f64>::cpp_convert_to(__x);let __sum=$crate::src::core::SkMathPriv::SkCppAdd::cpp_add(&__double,&0.5f64);let __rounded=$crate::src::core::SkFloatingPoint::SkCppFloor::cpp_floor(&__sum);let __float=__rounded as f32;let $result=$crate::src::core::SkMathPriv::SkCppConvertTo::<i32>::cpp_convert_to(&__float);$body}};
 ($x:expr)=>{$crate::sk_float_round2int_no_saturate!(@with $x; |__result| __result)};
}
// Source: include/private/SkFloatingPoint.h:49-55.
#[macro_export]
macro_rules! SkIsFinite {
 (@pack)=>{()};
 (@pack $head:expr $(,$tail:expr)*)=>{(&$head,$crate::SkIsFinite!(@pack $($tail),*))};
 (@with $first:expr $(,$value:expr)*; |$result:ident| $body:expr)=>{{
  let __first=&$first;
  let __values=$crate::SkIsFinite!(@pack $($value),*);
  let $result=$crate::src::core::SkFloatingPoint::SkIsFinitePack(*__first,&__values);
  $body
 }};
 ($first:expr $(,$value:expr)*)=>{$crate::SkIsFinite!(@with $first $(,$value)*; |__result| __result)};
}

// Actual compiler long-double protocol, separately gated by the real native
// bridge's build/link availability, not inferred from Cargo host layout.
#[cfg(sk_native_long_double_ffi)]
pub mod long_double {
    use crate::interop::native_long_double as ld;
    // Staged caller projection for the real same-type long-double array overload.
    // Bind `ld` to the companion interop/native_long_double.rs module after root
    // registers and links the genuine compiler TU. No native long-double by-value
    // Rust ABI or generic owning floating type is claimed.
    // Source: include/private/SkFloatingPoint.h:57-66
    /// Safety: values has count>0 actual native long-double input leases; no unwind
    /// across the synchronous native stack-slot callback. All operands remain live.
    pub unsafe fn SkIsFiniteLongDoubleArray(values: &[ld::ValueRef<'_>]) -> bool {
        unsafe {
            ld::with_slots(|[mut prod, mut scratch, _, _]| {
                let x = values[0];
                prod.sub(x, x);
                for value in &values[1..] {
                    scratch.mul(prod.as_ref(), *value);
                    core::mem::swap(&mut prod, &mut scratch);
                }
                let result = prod.as_ref();
                result.eq(result)
            })
        }
    }
    // Source: include/private/SkFloatingPoint.h:40-43
    // This native function is constexpr. The runtime projection below is expressly
    // NOT claimed to implement its constant-evaluation domain.
    pub fn SkIsNaNLongDoubleRuntime(x: ld::ValueRef<'_>) -> bool {
        x.ne(x)
    }
    // Source: include/private/SkFloatingPoint.h:122-124
    // Actual std overload calls use native long-double slots; final integral casts
    // retain C++ domain requirements instead of Rust's saturating `as` semantics.
    /// Safety: native conversion result must be representable as int; no unwind.
    pub unsafe fn sk_float_floor2int_no_saturate_long_double(x: ld::ValueRef<'_>) -> i32 {
        unsafe {
            ld::with_slots(|[mut result, _, _, _]| {
                result.floor(x);
                result.as_ref().to_i32()
            })
        }
    }
    /// Safety: native conversion result must be representable as int; no unwind.
    pub unsafe fn sk_float_ceil2int_no_saturate_long_double(x: ld::ValueRef<'_>) -> i32 {
        unsafe {
            ld::with_slots(|[mut result, _, _, _]| {
                result.ceil(x);
                result.as_ref().to_i32()
            })
        }
    }
    // Source: include/private/SkFloatingPoint.h:128
    /// Safety: native conversion result must be representable as int; no unwind.
    pub unsafe fn sk_double_round2int_long_double(x: ld::ValueRef<'_>) -> i32 {
        unsafe {
            ld::with_slots(|[mut result, _, _, _]| {
                result.round(x);
                result.as_ref().to_i32()
            })
        }
    }

    // Source: include/private/SkFloatingPoint.h:126
    /// Safety: the continuation must not unwind through the native frame and must
    /// not let its temporary long-double input lease escape.
    pub unsafe fn with_sk_double_round_long_double<R, F>(x: ld::ValueRef<'_>, f: F) -> R
    where
        F: for<'a> FnOnce(ld::ValueRef<'a>) -> R,
    {
        unsafe {
            ld::with_slots(|[mut half, mut sum, mut rounded, _]| {
                half.from_f64(0.5);
                sum.add(x, half.as_ref());
                rounded.floor(sum.as_ref());
                f(rounded.as_ref())
            })
        }
    }

    // Source: include/private/SkFloatingPoint.h:38,119,123.
    // The original round macro explicitly converts its input to double first.
    pub fn sk_float_round_long_double(x: ld::ValueRef<'_>) -> f32 {
        super::sk_float_round(x.to_f64())
    }
    pub fn sk_float_round2int_long_double(x: ld::ValueRef<'_>) -> i32 {
        super::sk_float_saturate2int(sk_float_round_long_double(x))
    }
    /// Safety: the rounded float is representable as native int.
    pub unsafe fn sk_float_round2int_no_saturate_long_double(x: ld::ValueRef<'_>) -> i32 {
        unsafe { sk_float_round_long_double(x).to_int_unchecked::<i32>() }
    }
    // Source: include/private/SkFloatingPoint.h:118,120.
    /// Safety: no unwind across the native primitive frame.
    pub unsafe fn sk_float_floor2int_long_double(x: ld::ValueRef<'_>) -> i32 {
        unsafe {
            ld::with_slots(|[mut result, _, _, _]| {
                result.floor(x);
                super::sk_float_saturate2int(result.as_ref().to_f32())
            })
        }
    }
    /// Safety: no unwind across the native primitive frame.
    pub unsafe fn sk_float_ceil2int_long_double(x: ld::ValueRef<'_>) -> i32 {
        unsafe {
            ld::with_slots(|[mut result, _, _, _]| {
                result.ceil(x);
                super::sk_float_saturate2int(result.as_ref().to_f32())
            })
        }
    }
    // Source: include/private/SkFloatingPoint.h:127,129.
    pub use sk_float_ceil2int_no_saturate_long_double as sk_double_ceil2int_long_double;
    pub use sk_float_floor2int_no_saturate_long_double as sk_double_floor2int_long_double;
}

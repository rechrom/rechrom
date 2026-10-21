//! Actual checked native static casts. Runtime assertion policy follows the
//! selected native SK_DEBUG condition; no Rust-debug OR overrides release.
use crate::include::private::SkTFitsIn::{sk_strip_enum, SkCastFrom, SkTFitsIn};
// Generic language constraints differ when native SkASSERT erases its operand.
// This trait supplies no state or conversion algorithm beyond that expression.
pub const trait SkToCast<S>: Sized {
    fn sk_to_cast(source: S) -> Self;
}
#[cfg(sk_debug)]
const impl<D, S> SkToCast<S> for D
where
    D: sk_strip_enum + [const] SkCastFrom<S>,
    S: sk_strip_enum
        + [const] PartialOrd
        + [const] SkCastFrom<D>
        + [const] SkCastFrom<i32>
        + [const] SkCastFrom<<D as sk_strip_enum>::Type>,
{
    fn sk_to_cast(source: S) -> Self {
        checked_fit_assert(SkTFitsIn::<D, S>(source));
        <D as SkCastFrom<S>>::sk_cast_from(source)
    }
}
#[cfg(not(sk_debug))]
const impl<D: [const] SkCastFrom<S>, S> SkToCast<S> for D {
    fn sk_to_cast(source: S) -> Self {
        <D as SkCastFrom<S>>::sk_cast_from(source)
    }
}
// Source: include/private/SkTo.h:16-19
pub const fn SkTo<D: [const] SkToCast<S>, S>(source: S) -> D {
    D::sk_to_cast(source)
}
macro_rules! native_cast {
    ($name:ident,$d:ty) => {
        pub const fn $name<S>(source: S) -> $d
        where
            $d: [const] SkToCast<S>,
        {
            SkTo::<$d, S>(source)
        }
    };
}
// Source: include/private/SkTo.h:21
native_cast!(SkToS8, i8);
// Source: include/private/SkTo.h:22
native_cast!(SkToU8, u8);
// Source: include/private/SkTo.h:23
native_cast!(SkToS16, i16);
// Source: include/private/SkTo.h:24
native_cast!(SkToU16, u16);
// Source: include/private/SkTo.h:25
native_cast!(SkToS32, i32);
// Source: include/private/SkTo.h:26
native_cast!(SkToU32, u32);
// Source: include/private/SkTo.h:27
native_cast!(SkToS64, i64);
// Source: include/private/SkTo.h:28
native_cast!(SkToU64, u64);
// Source: include/private/SkTo.h:29
native_cast!(SkToInt, core::ffi::c_int);
// Source: include/private/SkTo.h:30
native_cast!(SkToUInt, core::ffi::c_uint);
// Source: include/private/SkTo.h:31
native_cast!(SkToSizeT, usize);

/// Projection of C++'s bool conversion, preserving its original borrowed input.
pub const trait SkBoolCast {
    fn sk_bool_cast(&self) -> bool;
}
macro_rules! numeric_bool {
 ($($t:ty),* $(,)?)=>{$(const impl SkBoolCast for $t {fn sk_bool_cast(&self)->bool {*self!=0 as $t}})*};
}
numeric_bool!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
const impl SkBoolCast for bool {
    fn sk_bool_cast(&self) -> bool {
        *self
    }
}
const impl<T: ?Sized> SkBoolCast for *const T {
    fn sk_bool_cast(&self) -> bool {
        !self.is_null()
    }
}
const impl<T: ?Sized> SkBoolCast for *mut T {
    fn sk_bool_cast(&self) -> bool {
        !self.is_null()
    }
}
const impl<T> SkBoolCast for Option<T> {
    fn sk_bool_cast(&self) -> bool {
        self.is_some()
    }
}
// Source: include/core/SkRefCnt.h:301
// Native sk_sp::operator bool is non-constexpr. Its implementation intentionally
// stays runtime-only; a const SkToBool cannot invent a const native client.
impl<T: crate::include::core::SkRefCnt::SkIntrusiveRefCounted + ?Sized> SkBoolCast
    for crate::include::core::SkRefCnt::sk_sp<T>
{
    fn sk_bool_cast(&self) -> bool {
        self.get().is_some()
    }
}
// Source: include/private/SkTo.h:35-37
pub const fn SkToBool<T: [const] SkBoolCast + ?Sized>(value: &T) -> bool {
    value.sk_bool_cast()
}
// Source: include/private/SkTo.h:17
// Constant evaluation diagnoses the same failed native precondition. At runtime
// the canonical assertion keeps Chromium source location and termination policy.
#[cfg(sk_debug)]
const fn checked_fit_assert(fits: bool) {
    const fn during_const(fits: bool) {
        assert!(fits, "SkTFitsIn<D>(s)");
    }
    fn during_runtime(fits: bool) {
        crate::SkASSERT!(@source b"include/private/SkTo.h\0".as_ptr(),17; @condition b"SkTFitsIn<D>(s)\0".as_ptr(); fits);
    }
    core::intrinsics::const_eval_select((fits,), during_const, during_runtime)
}

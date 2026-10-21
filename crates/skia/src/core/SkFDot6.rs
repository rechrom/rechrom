use crate::include::private::SkFixed::{SkFixed, SkFixedDiv};

// Source: src/core/SkFDot6.h:18-18
pub type SkFDot6 = i32;

// Source: src/core/SkFDot6.h:25-40
#[inline]
pub fn SkScalarRoundToFDot6(x: f32, shift: i32) -> SkFDot6 {
    let fractional_bits = 6 + shift;
    let magic = (1i64.wrapping_shl((52 - fractional_bits) as u32)) as f64 * 1.5;
    let value = x as f64 + magic;
    value.to_bits() as u32 as i32
}

// Source: src/core/SkFDot6.h:42-42
pub const SK_FDot6One: i32 = 64;

// Source: src/core/SkFDot6.h:43-43
pub const SK_FDot6Half: i32 = 32;

// Source: src/core/SkFDot6.h:46-50
#[cfg(sk_debug)]
#[inline]
pub const fn SkIntToFDot6(x: i32) -> SkFDot6 {
    debug_assert!(x >= i32::MIN >> 6 && x <= i32::MAX >> 6);
    x.wrapping_shl(6)
}

// Source: src/core/SkFDot6.h:52-52
#[cfg(not(sk_debug))]
#[inline]
pub const fn SkIntToFDot6(x: i32) -> SkFDot6 {
    x.wrapping_shl(6)
}

// Source: src/core/SkFDot6.h:55-55
#[inline]
pub fn SkFDot6Floor(x: i32) -> i32 {
    x >> 6
}

// Source: src/core/SkFDot6.h:56-56
#[inline]
pub fn SkFDot6Ceil(x: i32) -> i32 {
    x.wrapping_add(63) >> 6
}

// Source: src/core/SkFDot6.h:57-57
#[inline]
pub fn SkFDot6Round(x: i32) -> i32 {
    x.wrapping_add(SK_FDot6Half) >> 6
}

// Source: src/core/SkFDot6.h:59-59
#[inline]
pub fn SkFixedToFDot6(x: i32) -> i32 {
    x >> 10
}

// Source: src/core/SkFDot6.h:61-65
#[inline]
pub fn SkFDot6ToFixed(x: SkFDot6) -> SkFixed {
    debug_assert_eq!(x.wrapping_shl(10) >> 10, x);
    x.wrapping_shl(10)
}

// Source: src/core/SkFDot6.h:67-67
#[inline]
pub fn SkFloatToFDot6(x: f32) -> SkFDot6 {
    (x * SK_FDot6One as f32) as i32
}

// Source: src/core/SkFDot6.h:68-68
pub use SkFloatToFDot6 as SkScalarToFDot6;

// Source: src/core/SkFDot6.h:69-69
#[inline]
pub fn SkFDot6ToFloat(x: SkFDot6) -> f32 {
    x as f32 * 0.015625
}

// Source: src/core/SkFDot6.h:70-70
pub use SkFDot6ToFloat as SkFDot6ToScalar;

// Source: src/core/SkFDot6.h:72-80
#[inline]
pub fn SkFDot6Div(a: SkFDot6, b: SkFDot6) -> SkFixed {
    crate::SkASSERT!(@source b"src/core/SkFDot6.h\0".as_ptr(),73; @condition b"b != 0\0".as_ptr(); b != 0);
    if i16::try_from(a).is_ok() {
        a.wrapping_shl(16) / b
    } else {
        SkFixedDiv(a, b)
    }
}

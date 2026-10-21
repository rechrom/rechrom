// Source: base/numerics/clamped_math_impl.h:20-149,191-226,320-344.
#![allow(non_snake_case)]
use super::checked_math_impl::*;
use super::safe_conversions::*;
pub trait ClampedBinary<R, V> {
    fn clamp_add(self, r: R) -> V;
    fn clamp_sub(self, r: R) -> V;
    fn clamp_mul(self, r: R) -> V;
    fn clamp_div(self, r: R) -> V;
}
pub fn SaturatedNegWrapper<T: IntegralOps>(v: T) -> T {
    v.saturated_neg()
}
pub fn SaturatedAbsWrapper<T: IntegralOps>(v: T) -> T {
    v.saturated_abs()
}
pub fn ClampedModOp<L: Numeric + CheckedModulo<R, V> + Cast<V>, R: Numeric, V: Numeric>(
    x: L,
    y: R,
) -> V {
    let mut result = V::ZERO;
    if x.modulo(y, &mut result) {
        result
    } else {
        x.cast()
    }
}
pub fn ClampedLshOp<T: IntegralOps>(x: T, s: u64) -> T {
    x.clamped_lsh(s)
}
pub fn ClampedRshOp<T: IntegralOps>(x: T, s: u64) -> T {
    x.clamped_rsh(s)
}

// Source: clamped_math_impl.h:326-343, i8/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for i8
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, i8/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for i8
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, u8/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for u8
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, u8/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for u8
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, i16/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for i16
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, i16/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for i16
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, u16/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for u16
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, u16/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for u16
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, i32/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for i32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, i32/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for i32
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, u32/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for u32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, u32/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for u32
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, i64/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for i64
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, i64/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for i64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, u64/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for u64
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, u64/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for u64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, isize/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for isize
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, isize/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for isize
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, usize/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for usize
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, usize/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for usize
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f32/i8 float specialization.
impl<V: Numeric> ClampedBinary<i8, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: i8) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: i8) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: i8) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: i8) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/u8 float specialization.
impl<V: Numeric> ClampedBinary<u8, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: u8) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: u8) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: u8) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: u8) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/i16 float specialization.
impl<V: Numeric> ClampedBinary<i16, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: i16) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: i16) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: i16) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: i16) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/u16 float specialization.
impl<V: Numeric> ClampedBinary<u16, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: u16) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: u16) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: u16) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: u16) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/i32 float specialization.
impl<V: Numeric> ClampedBinary<i32, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: i32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: i32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: i32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: i32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/u32 float specialization.
impl<V: Numeric> ClampedBinary<u32, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: u32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: u32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: u32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: u32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/i64 float specialization.
impl<V: Numeric> ClampedBinary<i64, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: i64) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: i64) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: i64) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: i64) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/u64 float specialization.
impl<V: Numeric> ClampedBinary<u64, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: u64) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: u64) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: u64) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: u64) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/isize float specialization.
impl<V: Numeric> ClampedBinary<isize, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: isize) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: isize) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: isize) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: isize) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/usize float specialization.
impl<V: Numeric> ClampedBinary<usize, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: usize) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: usize) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: usize) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: usize) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for f32
where
    f32: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f32) + (r as f32))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f32) - (r as f32))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f32) * (r as f32))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f32) / (r as f32))
    }
}

// Source: clamped_math_impl.h:326-343, f32/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for f32
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/i8 float specialization.
impl<V: Numeric> ClampedBinary<i8, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: i8) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: i8) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: i8) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: i8) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/u8 float specialization.
impl<V: Numeric> ClampedBinary<u8, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: u8) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: u8) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: u8) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: u8) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/i16 float specialization.
impl<V: Numeric> ClampedBinary<i16, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: i16) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: i16) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: i16) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: i16) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/u16 float specialization.
impl<V: Numeric> ClampedBinary<u16, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: u16) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: u16) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: u16) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: u16) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/i32 float specialization.
impl<V: Numeric> ClampedBinary<i32, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: i32) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: i32) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: i32) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: i32) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/u32 float specialization.
impl<V: Numeric> ClampedBinary<u32, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: u32) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: u32) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: u32) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: u32) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/i64 float specialization.
impl<V: Numeric> ClampedBinary<i64, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: i64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: i64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: i64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: i64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/u64 float specialization.
impl<V: Numeric> ClampedBinary<u64, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: u64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: u64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: u64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: u64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/isize float specialization.
impl<V: Numeric> ClampedBinary<isize, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: isize) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: isize) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: isize) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: isize) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/usize float specialization.
impl<V: Numeric> ClampedBinary<usize, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: usize) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: usize) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: usize) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: usize) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/f32 float specialization.
impl<V: Numeric> ClampedBinary<f32, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f32) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f32) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f32) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f32) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

// Source: clamped_math_impl.h:326-343, f64/f64 float specialization.
impl<V: Numeric> ClampedBinary<f64, V> for f64
where
    f64: Cast<V>,
{
    fn clamp_add(self, r: f64) -> V {
        saturated_cast((self as f64) + (r as f64))
    }
    fn clamp_sub(self, r: f64) -> V {
        saturated_cast((self as f64) - (r as f64))
    }
    fn clamp_mul(self, r: f64) -> V {
        saturated_cast((self as f64) * (r as f64))
    }
    fn clamp_div(self, r: f64) -> V {
        saturated_cast((self as f64) / (r as f64))
    }
}

use super::safe_math_shared_impl::{ClampedAddFastOp, ClampedMulFastOp, ClampedSubFastOp};

// Source: clamped_math_impl.h:68-149, i8/i8.
impl<V: Numeric> ClampedBinary<i8, V> for i8
where
    i8: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <i8 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, i8>::is_supported {
            return ClampedAddFastOp::<i8, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, i8>::is_supported {
            return ClampedSubFastOp::<i8, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<i8, i8>::is_supported {
            return ClampedMulFastOp::<i8, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/u8.
impl<V: Numeric> ClampedBinary<u8, V> for i8
where
    i8: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <i8 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, u8>::is_supported {
            return ClampedAddFastOp::<i8, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, u8>::is_supported {
            return ClampedSubFastOp::<i8, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<i8, u8>::is_supported {
            return ClampedMulFastOp::<i8, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/i16.
impl<V: Numeric> ClampedBinary<i16, V> for i8
where
    i8: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <i8 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, i16>::is_supported {
            return ClampedAddFastOp::<i8, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, i16>::is_supported {
            return ClampedSubFastOp::<i8, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<i8, i16>::is_supported {
            return ClampedMulFastOp::<i8, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/u16.
impl<V: Numeric> ClampedBinary<u16, V> for i8
where
    i8: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <i8 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, u16>::is_supported {
            return ClampedAddFastOp::<i8, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, u16>::is_supported {
            return ClampedSubFastOp::<i8, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<i8, u16>::is_supported {
            return ClampedMulFastOp::<i8, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/i32.
impl<V: Numeric> ClampedBinary<i32, V> for i8
where
    i8: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <i8 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, i32>::is_supported {
            return ClampedAddFastOp::<i8, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, i32>::is_supported {
            return ClampedSubFastOp::<i8, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<i8, i32>::is_supported {
            return ClampedMulFastOp::<i8, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/u32.
impl<V: Numeric> ClampedBinary<u32, V> for i8
where
    i8: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <i8 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, u32>::is_supported {
            return ClampedAddFastOp::<i8, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, u32>::is_supported {
            return ClampedSubFastOp::<i8, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<i8, u32>::is_supported {
            return ClampedMulFastOp::<i8, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/i64.
impl<V: Numeric> ClampedBinary<i64, V> for i8
where
    i8: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <i8 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, i64>::is_supported {
            return ClampedAddFastOp::<i8, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, i64>::is_supported {
            return ClampedSubFastOp::<i8, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<i8, i64>::is_supported {
            return ClampedMulFastOp::<i8, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/u64.
impl<V: Numeric> ClampedBinary<u64, V> for i8
where
    i8: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <i8 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, u64>::is_supported {
            return ClampedAddFastOp::<i8, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, u64>::is_supported {
            return ClampedSubFastOp::<i8, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<i8, u64>::is_supported {
            return ClampedMulFastOp::<i8, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/isize.
impl<V: Numeric> ClampedBinary<isize, V> for i8
where
    i8: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <i8 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, isize>::is_supported {
            return ClampedAddFastOp::<i8, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, isize>::is_supported {
            return ClampedSubFastOp::<i8, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<i8, isize>::is_supported {
            return ClampedMulFastOp::<i8, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i8/usize.
impl<V: Numeric> ClampedBinary<usize, V> for i8
where
    i8: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <i8 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i8, usize>::is_supported {
            return ClampedAddFastOp::<i8, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i8 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i8 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i8 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i8, usize>::is_supported {
            return ClampedSubFastOp::<i8, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<i8, usize>::is_supported {
            return ClampedMulFastOp::<i8, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/i8.
impl<V: Numeric> ClampedBinary<i8, V> for u8
where
    u8: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <u8 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, i8>::is_supported {
            return ClampedAddFastOp::<u8, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, i8>::is_supported {
            return ClampedSubFastOp::<u8, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<u8, i8>::is_supported {
            return ClampedMulFastOp::<u8, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/u8.
impl<V: Numeric> ClampedBinary<u8, V> for u8
where
    u8: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <u8 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, u8>::is_supported {
            return ClampedAddFastOp::<u8, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, u8>::is_supported {
            return ClampedSubFastOp::<u8, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<u8, u8>::is_supported {
            return ClampedMulFastOp::<u8, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/i16.
impl<V: Numeric> ClampedBinary<i16, V> for u8
where
    u8: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <u8 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, i16>::is_supported {
            return ClampedAddFastOp::<u8, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, i16>::is_supported {
            return ClampedSubFastOp::<u8, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<u8, i16>::is_supported {
            return ClampedMulFastOp::<u8, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/u16.
impl<V: Numeric> ClampedBinary<u16, V> for u8
where
    u8: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <u8 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, u16>::is_supported {
            return ClampedAddFastOp::<u8, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, u16>::is_supported {
            return ClampedSubFastOp::<u8, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<u8, u16>::is_supported {
            return ClampedMulFastOp::<u8, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/i32.
impl<V: Numeric> ClampedBinary<i32, V> for u8
where
    u8: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <u8 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, i32>::is_supported {
            return ClampedAddFastOp::<u8, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, i32>::is_supported {
            return ClampedSubFastOp::<u8, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<u8, i32>::is_supported {
            return ClampedMulFastOp::<u8, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/u32.
impl<V: Numeric> ClampedBinary<u32, V> for u8
where
    u8: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <u8 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, u32>::is_supported {
            return ClampedAddFastOp::<u8, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, u32>::is_supported {
            return ClampedSubFastOp::<u8, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<u8, u32>::is_supported {
            return ClampedMulFastOp::<u8, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/i64.
impl<V: Numeric> ClampedBinary<i64, V> for u8
where
    u8: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <u8 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, i64>::is_supported {
            return ClampedAddFastOp::<u8, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, i64>::is_supported {
            return ClampedSubFastOp::<u8, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<u8, i64>::is_supported {
            return ClampedMulFastOp::<u8, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/u64.
impl<V: Numeric> ClampedBinary<u64, V> for u8
where
    u8: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <u8 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, u64>::is_supported {
            return ClampedAddFastOp::<u8, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, u64>::is_supported {
            return ClampedSubFastOp::<u8, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<u8, u64>::is_supported {
            return ClampedMulFastOp::<u8, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/isize.
impl<V: Numeric> ClampedBinary<isize, V> for u8
where
    u8: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <u8 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, isize>::is_supported {
            return ClampedAddFastOp::<u8, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, isize>::is_supported {
            return ClampedSubFastOp::<u8, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<u8, isize>::is_supported {
            return ClampedMulFastOp::<u8, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u8/usize.
impl<V: Numeric> ClampedBinary<usize, V> for u8
where
    u8: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <u8 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u8, usize>::is_supported {
            return ClampedAddFastOp::<u8, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u8 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u8 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u8 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u8, usize>::is_supported {
            return ClampedSubFastOp::<u8, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<u8, usize>::is_supported {
            return ClampedMulFastOp::<u8, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/i8.
impl<V: Numeric> ClampedBinary<i8, V> for i16
where
    i16: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <i16 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, i8>::is_supported {
            return ClampedAddFastOp::<i16, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, i8>::is_supported {
            return ClampedSubFastOp::<i16, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<i16, i8>::is_supported {
            return ClampedMulFastOp::<i16, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/u8.
impl<V: Numeric> ClampedBinary<u8, V> for i16
where
    i16: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <i16 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, u8>::is_supported {
            return ClampedAddFastOp::<i16, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, u8>::is_supported {
            return ClampedSubFastOp::<i16, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<i16, u8>::is_supported {
            return ClampedMulFastOp::<i16, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/i16.
impl<V: Numeric> ClampedBinary<i16, V> for i16
where
    i16: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <i16 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, i16>::is_supported {
            return ClampedAddFastOp::<i16, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, i16>::is_supported {
            return ClampedSubFastOp::<i16, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<i16, i16>::is_supported {
            return ClampedMulFastOp::<i16, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/u16.
impl<V: Numeric> ClampedBinary<u16, V> for i16
where
    i16: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <i16 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, u16>::is_supported {
            return ClampedAddFastOp::<i16, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, u16>::is_supported {
            return ClampedSubFastOp::<i16, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<i16, u16>::is_supported {
            return ClampedMulFastOp::<i16, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/i32.
impl<V: Numeric> ClampedBinary<i32, V> for i16
where
    i16: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <i16 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, i32>::is_supported {
            return ClampedAddFastOp::<i16, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, i32>::is_supported {
            return ClampedSubFastOp::<i16, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<i16, i32>::is_supported {
            return ClampedMulFastOp::<i16, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/u32.
impl<V: Numeric> ClampedBinary<u32, V> for i16
where
    i16: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <i16 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, u32>::is_supported {
            return ClampedAddFastOp::<i16, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, u32>::is_supported {
            return ClampedSubFastOp::<i16, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<i16, u32>::is_supported {
            return ClampedMulFastOp::<i16, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/i64.
impl<V: Numeric> ClampedBinary<i64, V> for i16
where
    i16: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <i16 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, i64>::is_supported {
            return ClampedAddFastOp::<i16, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, i64>::is_supported {
            return ClampedSubFastOp::<i16, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<i16, i64>::is_supported {
            return ClampedMulFastOp::<i16, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/u64.
impl<V: Numeric> ClampedBinary<u64, V> for i16
where
    i16: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <i16 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, u64>::is_supported {
            return ClampedAddFastOp::<i16, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, u64>::is_supported {
            return ClampedSubFastOp::<i16, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<i16, u64>::is_supported {
            return ClampedMulFastOp::<i16, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/isize.
impl<V: Numeric> ClampedBinary<isize, V> for i16
where
    i16: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <i16 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, isize>::is_supported {
            return ClampedAddFastOp::<i16, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, isize>::is_supported {
            return ClampedSubFastOp::<i16, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<i16, isize>::is_supported {
            return ClampedMulFastOp::<i16, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i16/usize.
impl<V: Numeric> ClampedBinary<usize, V> for i16
where
    i16: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <i16 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i16, usize>::is_supported {
            return ClampedAddFastOp::<i16, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i16 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i16 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i16 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i16, usize>::is_supported {
            return ClampedSubFastOp::<i16, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<i16, usize>::is_supported {
            return ClampedMulFastOp::<i16, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/i8.
impl<V: Numeric> ClampedBinary<i8, V> for u16
where
    u16: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <u16 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, i8>::is_supported {
            return ClampedAddFastOp::<u16, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, i8>::is_supported {
            return ClampedSubFastOp::<u16, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<u16, i8>::is_supported {
            return ClampedMulFastOp::<u16, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/u8.
impl<V: Numeric> ClampedBinary<u8, V> for u16
where
    u16: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <u16 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, u8>::is_supported {
            return ClampedAddFastOp::<u16, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, u8>::is_supported {
            return ClampedSubFastOp::<u16, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<u16, u8>::is_supported {
            return ClampedMulFastOp::<u16, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/i16.
impl<V: Numeric> ClampedBinary<i16, V> for u16
where
    u16: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <u16 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, i16>::is_supported {
            return ClampedAddFastOp::<u16, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, i16>::is_supported {
            return ClampedSubFastOp::<u16, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<u16, i16>::is_supported {
            return ClampedMulFastOp::<u16, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/u16.
impl<V: Numeric> ClampedBinary<u16, V> for u16
where
    u16: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <u16 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, u16>::is_supported {
            return ClampedAddFastOp::<u16, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, u16>::is_supported {
            return ClampedSubFastOp::<u16, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<u16, u16>::is_supported {
            return ClampedMulFastOp::<u16, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/i32.
impl<V: Numeric> ClampedBinary<i32, V> for u16
where
    u16: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <u16 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, i32>::is_supported {
            return ClampedAddFastOp::<u16, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, i32>::is_supported {
            return ClampedSubFastOp::<u16, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<u16, i32>::is_supported {
            return ClampedMulFastOp::<u16, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/u32.
impl<V: Numeric> ClampedBinary<u32, V> for u16
where
    u16: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <u16 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, u32>::is_supported {
            return ClampedAddFastOp::<u16, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, u32>::is_supported {
            return ClampedSubFastOp::<u16, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<u16, u32>::is_supported {
            return ClampedMulFastOp::<u16, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/i64.
impl<V: Numeric> ClampedBinary<i64, V> for u16
where
    u16: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <u16 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, i64>::is_supported {
            return ClampedAddFastOp::<u16, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, i64>::is_supported {
            return ClampedSubFastOp::<u16, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<u16, i64>::is_supported {
            return ClampedMulFastOp::<u16, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/u64.
impl<V: Numeric> ClampedBinary<u64, V> for u16
where
    u16: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <u16 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, u64>::is_supported {
            return ClampedAddFastOp::<u16, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, u64>::is_supported {
            return ClampedSubFastOp::<u16, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<u16, u64>::is_supported {
            return ClampedMulFastOp::<u16, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/isize.
impl<V: Numeric> ClampedBinary<isize, V> for u16
where
    u16: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <u16 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, isize>::is_supported {
            return ClampedAddFastOp::<u16, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, isize>::is_supported {
            return ClampedSubFastOp::<u16, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<u16, isize>::is_supported {
            return ClampedMulFastOp::<u16, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u16/usize.
impl<V: Numeric> ClampedBinary<usize, V> for u16
where
    u16: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <u16 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u16, usize>::is_supported {
            return ClampedAddFastOp::<u16, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u16 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u16 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u16 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u16, usize>::is_supported {
            return ClampedSubFastOp::<u16, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<u16, usize>::is_supported {
            return ClampedMulFastOp::<u16, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/i8.
impl<V: Numeric> ClampedBinary<i8, V> for i32
where
    i32: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <i32 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, i8>::is_supported {
            return ClampedAddFastOp::<i32, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, i8>::is_supported {
            return ClampedSubFastOp::<i32, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<i32, i8>::is_supported {
            return ClampedMulFastOp::<i32, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/u8.
impl<V: Numeric> ClampedBinary<u8, V> for i32
where
    i32: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <i32 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, u8>::is_supported {
            return ClampedAddFastOp::<i32, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, u8>::is_supported {
            return ClampedSubFastOp::<i32, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<i32, u8>::is_supported {
            return ClampedMulFastOp::<i32, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/i16.
impl<V: Numeric> ClampedBinary<i16, V> for i32
where
    i32: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <i32 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, i16>::is_supported {
            return ClampedAddFastOp::<i32, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, i16>::is_supported {
            return ClampedSubFastOp::<i32, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<i32, i16>::is_supported {
            return ClampedMulFastOp::<i32, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/u16.
impl<V: Numeric> ClampedBinary<u16, V> for i32
where
    i32: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <i32 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, u16>::is_supported {
            return ClampedAddFastOp::<i32, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, u16>::is_supported {
            return ClampedSubFastOp::<i32, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<i32, u16>::is_supported {
            return ClampedMulFastOp::<i32, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/i32.
impl<V: Numeric> ClampedBinary<i32, V> for i32
where
    i32: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <i32 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, i32>::is_supported {
            return ClampedAddFastOp::<i32, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, i32>::is_supported {
            return ClampedSubFastOp::<i32, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<i32, i32>::is_supported {
            return ClampedMulFastOp::<i32, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/u32.
impl<V: Numeric> ClampedBinary<u32, V> for i32
where
    i32: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <i32 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, u32>::is_supported {
            return ClampedAddFastOp::<i32, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, u32>::is_supported {
            return ClampedSubFastOp::<i32, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<i32, u32>::is_supported {
            return ClampedMulFastOp::<i32, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/i64.
impl<V: Numeric> ClampedBinary<i64, V> for i32
where
    i32: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <i32 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, i64>::is_supported {
            return ClampedAddFastOp::<i32, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, i64>::is_supported {
            return ClampedSubFastOp::<i32, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<i32, i64>::is_supported {
            return ClampedMulFastOp::<i32, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/u64.
impl<V: Numeric> ClampedBinary<u64, V> for i32
where
    i32: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <i32 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, u64>::is_supported {
            return ClampedAddFastOp::<i32, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, u64>::is_supported {
            return ClampedSubFastOp::<i32, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<i32, u64>::is_supported {
            return ClampedMulFastOp::<i32, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/isize.
impl<V: Numeric> ClampedBinary<isize, V> for i32
where
    i32: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <i32 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, isize>::is_supported {
            return ClampedAddFastOp::<i32, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, isize>::is_supported {
            return ClampedSubFastOp::<i32, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<i32, isize>::is_supported {
            return ClampedMulFastOp::<i32, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i32/usize.
impl<V: Numeric> ClampedBinary<usize, V> for i32
where
    i32: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <i32 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i32, usize>::is_supported {
            return ClampedAddFastOp::<i32, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i32 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i32 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i32 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i32, usize>::is_supported {
            return ClampedSubFastOp::<i32, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<i32, usize>::is_supported {
            return ClampedMulFastOp::<i32, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/i8.
impl<V: Numeric> ClampedBinary<i8, V> for u32
where
    u32: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <u32 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, i8>::is_supported {
            return ClampedAddFastOp::<u32, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, i8>::is_supported {
            return ClampedSubFastOp::<u32, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<u32, i8>::is_supported {
            return ClampedMulFastOp::<u32, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/u8.
impl<V: Numeric> ClampedBinary<u8, V> for u32
where
    u32: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <u32 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, u8>::is_supported {
            return ClampedAddFastOp::<u32, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, u8>::is_supported {
            return ClampedSubFastOp::<u32, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<u32, u8>::is_supported {
            return ClampedMulFastOp::<u32, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/i16.
impl<V: Numeric> ClampedBinary<i16, V> for u32
where
    u32: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <u32 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, i16>::is_supported {
            return ClampedAddFastOp::<u32, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, i16>::is_supported {
            return ClampedSubFastOp::<u32, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<u32, i16>::is_supported {
            return ClampedMulFastOp::<u32, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/u16.
impl<V: Numeric> ClampedBinary<u16, V> for u32
where
    u32: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <u32 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, u16>::is_supported {
            return ClampedAddFastOp::<u32, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, u16>::is_supported {
            return ClampedSubFastOp::<u32, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<u32, u16>::is_supported {
            return ClampedMulFastOp::<u32, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/i32.
impl<V: Numeric> ClampedBinary<i32, V> for u32
where
    u32: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <u32 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, i32>::is_supported {
            return ClampedAddFastOp::<u32, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, i32>::is_supported {
            return ClampedSubFastOp::<u32, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<u32, i32>::is_supported {
            return ClampedMulFastOp::<u32, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/u32.
impl<V: Numeric> ClampedBinary<u32, V> for u32
where
    u32: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <u32 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, u32>::is_supported {
            return ClampedAddFastOp::<u32, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, u32>::is_supported {
            return ClampedSubFastOp::<u32, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<u32, u32>::is_supported {
            return ClampedMulFastOp::<u32, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/i64.
impl<V: Numeric> ClampedBinary<i64, V> for u32
where
    u32: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <u32 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, i64>::is_supported {
            return ClampedAddFastOp::<u32, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, i64>::is_supported {
            return ClampedSubFastOp::<u32, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<u32, i64>::is_supported {
            return ClampedMulFastOp::<u32, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/u64.
impl<V: Numeric> ClampedBinary<u64, V> for u32
where
    u32: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <u32 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, u64>::is_supported {
            return ClampedAddFastOp::<u32, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, u64>::is_supported {
            return ClampedSubFastOp::<u32, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<u32, u64>::is_supported {
            return ClampedMulFastOp::<u32, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/isize.
impl<V: Numeric> ClampedBinary<isize, V> for u32
where
    u32: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <u32 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, isize>::is_supported {
            return ClampedAddFastOp::<u32, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, isize>::is_supported {
            return ClampedSubFastOp::<u32, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<u32, isize>::is_supported {
            return ClampedMulFastOp::<u32, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u32/usize.
impl<V: Numeric> ClampedBinary<usize, V> for u32
where
    u32: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <u32 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u32, usize>::is_supported {
            return ClampedAddFastOp::<u32, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u32 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u32 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u32 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u32, usize>::is_supported {
            return ClampedSubFastOp::<u32, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<u32, usize>::is_supported {
            return ClampedMulFastOp::<u32, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/i8.
impl<V: Numeric> ClampedBinary<i8, V> for i64
where
    i64: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <i64 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, i8>::is_supported {
            return ClampedAddFastOp::<i64, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, i8>::is_supported {
            return ClampedSubFastOp::<i64, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<i64, i8>::is_supported {
            return ClampedMulFastOp::<i64, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/u8.
impl<V: Numeric> ClampedBinary<u8, V> for i64
where
    i64: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <i64 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, u8>::is_supported {
            return ClampedAddFastOp::<i64, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, u8>::is_supported {
            return ClampedSubFastOp::<i64, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<i64, u8>::is_supported {
            return ClampedMulFastOp::<i64, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/i16.
impl<V: Numeric> ClampedBinary<i16, V> for i64
where
    i64: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <i64 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, i16>::is_supported {
            return ClampedAddFastOp::<i64, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, i16>::is_supported {
            return ClampedSubFastOp::<i64, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<i64, i16>::is_supported {
            return ClampedMulFastOp::<i64, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/u16.
impl<V: Numeric> ClampedBinary<u16, V> for i64
where
    i64: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <i64 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, u16>::is_supported {
            return ClampedAddFastOp::<i64, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, u16>::is_supported {
            return ClampedSubFastOp::<i64, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<i64, u16>::is_supported {
            return ClampedMulFastOp::<i64, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/i32.
impl<V: Numeric> ClampedBinary<i32, V> for i64
where
    i64: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <i64 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, i32>::is_supported {
            return ClampedAddFastOp::<i64, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, i32>::is_supported {
            return ClampedSubFastOp::<i64, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<i64, i32>::is_supported {
            return ClampedMulFastOp::<i64, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/u32.
impl<V: Numeric> ClampedBinary<u32, V> for i64
where
    i64: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <i64 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, u32>::is_supported {
            return ClampedAddFastOp::<i64, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, u32>::is_supported {
            return ClampedSubFastOp::<i64, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<i64, u32>::is_supported {
            return ClampedMulFastOp::<i64, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/i64.
impl<V: Numeric> ClampedBinary<i64, V> for i64
where
    i64: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <i64 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, i64>::is_supported {
            return ClampedAddFastOp::<i64, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, i64>::is_supported {
            return ClampedSubFastOp::<i64, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<i64, i64>::is_supported {
            return ClampedMulFastOp::<i64, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/u64.
impl<V: Numeric> ClampedBinary<u64, V> for i64
where
    i64: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <i64 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, u64>::is_supported {
            return ClampedAddFastOp::<i64, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, u64>::is_supported {
            return ClampedSubFastOp::<i64, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<i64, u64>::is_supported {
            return ClampedMulFastOp::<i64, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/isize.
impl<V: Numeric> ClampedBinary<isize, V> for i64
where
    i64: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <i64 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, isize>::is_supported {
            return ClampedAddFastOp::<i64, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, isize>::is_supported {
            return ClampedSubFastOp::<i64, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<i64, isize>::is_supported {
            return ClampedMulFastOp::<i64, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, i64/usize.
impl<V: Numeric> ClampedBinary<usize, V> for i64
where
    i64: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <i64 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<i64, usize>::is_supported {
            return ClampedAddFastOp::<i64, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<i64 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<i64 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<i64 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<i64, usize>::is_supported {
            return ClampedSubFastOp::<i64, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<i64, usize>::is_supported {
            return ClampedMulFastOp::<i64, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/i8.
impl<V: Numeric> ClampedBinary<i8, V> for u64
where
    u64: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <u64 as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, i8>::is_supported {
            return ClampedAddFastOp::<u64, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, i8>::is_supported {
            return ClampedSubFastOp::<u64, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<u64, i8>::is_supported {
            return ClampedMulFastOp::<u64, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/u8.
impl<V: Numeric> ClampedBinary<u8, V> for u64
where
    u64: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <u64 as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, u8>::is_supported {
            return ClampedAddFastOp::<u64, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, u8>::is_supported {
            return ClampedSubFastOp::<u64, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<u64, u8>::is_supported {
            return ClampedMulFastOp::<u64, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/i16.
impl<V: Numeric> ClampedBinary<i16, V> for u64
where
    u64: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <u64 as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, i16>::is_supported {
            return ClampedAddFastOp::<u64, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, i16>::is_supported {
            return ClampedSubFastOp::<u64, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<u64, i16>::is_supported {
            return ClampedMulFastOp::<u64, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/u16.
impl<V: Numeric> ClampedBinary<u16, V> for u64
where
    u64: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <u64 as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, u16>::is_supported {
            return ClampedAddFastOp::<u64, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, u16>::is_supported {
            return ClampedSubFastOp::<u64, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<u64, u16>::is_supported {
            return ClampedMulFastOp::<u64, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/i32.
impl<V: Numeric> ClampedBinary<i32, V> for u64
where
    u64: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <u64 as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, i32>::is_supported {
            return ClampedAddFastOp::<u64, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, i32>::is_supported {
            return ClampedSubFastOp::<u64, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<u64, i32>::is_supported {
            return ClampedMulFastOp::<u64, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/u32.
impl<V: Numeric> ClampedBinary<u32, V> for u64
where
    u64: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <u64 as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, u32>::is_supported {
            return ClampedAddFastOp::<u64, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, u32>::is_supported {
            return ClampedSubFastOp::<u64, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<u64, u32>::is_supported {
            return ClampedMulFastOp::<u64, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/i64.
impl<V: Numeric> ClampedBinary<i64, V> for u64
where
    u64: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <u64 as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, i64>::is_supported {
            return ClampedAddFastOp::<u64, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, i64>::is_supported {
            return ClampedSubFastOp::<u64, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<u64, i64>::is_supported {
            return ClampedMulFastOp::<u64, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/u64.
impl<V: Numeric> ClampedBinary<u64, V> for u64
where
    u64: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <u64 as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, u64>::is_supported {
            return ClampedAddFastOp::<u64, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, u64>::is_supported {
            return ClampedSubFastOp::<u64, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<u64, u64>::is_supported {
            return ClampedMulFastOp::<u64, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/isize.
impl<V: Numeric> ClampedBinary<isize, V> for u64
where
    u64: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <u64 as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, isize>::is_supported {
            return ClampedAddFastOp::<u64, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, isize>::is_supported {
            return ClampedSubFastOp::<u64, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<u64, isize>::is_supported {
            return ClampedMulFastOp::<u64, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, u64/usize.
impl<V: Numeric> ClampedBinary<usize, V> for u64
where
    u64: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <u64 as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<u64, usize>::is_supported {
            return ClampedAddFastOp::<u64, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<u64 as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<u64 as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<u64 as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<u64, usize>::is_supported {
            return ClampedSubFastOp::<u64, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<u64, usize>::is_supported {
            return ClampedMulFastOp::<u64, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/i8.
impl<V: Numeric> ClampedBinary<i8, V> for isize
where
    isize: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <isize as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, i8>::is_supported {
            return ClampedAddFastOp::<isize, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, i8>::is_supported {
            return ClampedSubFastOp::<isize, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<isize, i8>::is_supported {
            return ClampedMulFastOp::<isize, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/u8.
impl<V: Numeric> ClampedBinary<u8, V> for isize
where
    isize: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <isize as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, u8>::is_supported {
            return ClampedAddFastOp::<isize, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, u8>::is_supported {
            return ClampedSubFastOp::<isize, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<isize, u8>::is_supported {
            return ClampedMulFastOp::<isize, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/i16.
impl<V: Numeric> ClampedBinary<i16, V> for isize
where
    isize: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <isize as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, i16>::is_supported {
            return ClampedAddFastOp::<isize, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, i16>::is_supported {
            return ClampedSubFastOp::<isize, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<isize, i16>::is_supported {
            return ClampedMulFastOp::<isize, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/u16.
impl<V: Numeric> ClampedBinary<u16, V> for isize
where
    isize: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <isize as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, u16>::is_supported {
            return ClampedAddFastOp::<isize, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, u16>::is_supported {
            return ClampedSubFastOp::<isize, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<isize, u16>::is_supported {
            return ClampedMulFastOp::<isize, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/i32.
impl<V: Numeric> ClampedBinary<i32, V> for isize
where
    isize: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <isize as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, i32>::is_supported {
            return ClampedAddFastOp::<isize, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, i32>::is_supported {
            return ClampedSubFastOp::<isize, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<isize, i32>::is_supported {
            return ClampedMulFastOp::<isize, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/u32.
impl<V: Numeric> ClampedBinary<u32, V> for isize
where
    isize: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <isize as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, u32>::is_supported {
            return ClampedAddFastOp::<isize, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, u32>::is_supported {
            return ClampedSubFastOp::<isize, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<isize, u32>::is_supported {
            return ClampedMulFastOp::<isize, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/i64.
impl<V: Numeric> ClampedBinary<i64, V> for isize
where
    isize: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <isize as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, i64>::is_supported {
            return ClampedAddFastOp::<isize, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, i64>::is_supported {
            return ClampedSubFastOp::<isize, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<isize, i64>::is_supported {
            return ClampedMulFastOp::<isize, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/u64.
impl<V: Numeric> ClampedBinary<u64, V> for isize
where
    isize: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <isize as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, u64>::is_supported {
            return ClampedAddFastOp::<isize, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, u64>::is_supported {
            return ClampedSubFastOp::<isize, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<isize, u64>::is_supported {
            return ClampedMulFastOp::<isize, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/isize.
impl<V: Numeric> ClampedBinary<isize, V> for isize
where
    isize: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <isize as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, isize>::is_supported {
            return ClampedAddFastOp::<isize, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, isize>::is_supported {
            return ClampedSubFastOp::<isize, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<isize, isize>::is_supported {
            return ClampedMulFastOp::<isize, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, isize/usize.
impl<V: Numeric> ClampedBinary<usize, V> for isize
where
    isize: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <isize as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<isize, usize>::is_supported {
            return ClampedAddFastOp::<isize, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<isize as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<isize as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<isize as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<isize, usize>::is_supported {
            return ClampedSubFastOp::<isize, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<isize, usize>::is_supported {
            return ClampedMulFastOp::<isize, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/i8.
impl<V: Numeric> ClampedBinary<i8, V> for usize
where
    usize: CheckedBinary<i8, V>,
    V: Cast<i8>,
    i32: Cast<V>,
    <usize as Promote<i8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, i8>::is_supported {
            return ClampedAddFastOp::<usize, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i8) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, i8>::is_supported {
            return ClampedSubFastOp::<usize, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i8) -> V {
        if ClampedMulFastOp::<usize, i8>::is_supported {
            return ClampedMulFastOp::<usize, i8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/u8.
impl<V: Numeric> ClampedBinary<u8, V> for usize
where
    usize: CheckedBinary<u8, V>,
    V: Cast<u8>,
    i32: Cast<V>,
    <usize as Promote<u8>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, u8>::is_supported {
            return ClampedAddFastOp::<usize, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u8) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u8>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u8>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u8>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u8>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, u8>::is_supported {
            return ClampedSubFastOp::<usize, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u8) -> V {
        if ClampedMulFastOp::<usize, u8>::is_supported {
            return ClampedMulFastOp::<usize, u8>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u8) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/i16.
impl<V: Numeric> ClampedBinary<i16, V> for usize
where
    usize: CheckedBinary<i16, V>,
    V: Cast<i16>,
    i32: Cast<V>,
    <usize as Promote<i16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, i16>::is_supported {
            return ClampedAddFastOp::<usize, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i16) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, i16>::is_supported {
            return ClampedSubFastOp::<usize, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i16) -> V {
        if ClampedMulFastOp::<usize, i16>::is_supported {
            return ClampedMulFastOp::<usize, i16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/u16.
impl<V: Numeric> ClampedBinary<u16, V> for usize
where
    usize: CheckedBinary<u16, V>,
    V: Cast<u16>,
    i32: Cast<V>,
    <usize as Promote<u16>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, u16>::is_supported {
            return ClampedAddFastOp::<usize, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u16) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u16>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u16>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u16>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u16>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, u16>::is_supported {
            return ClampedSubFastOp::<usize, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u16) -> V {
        if ClampedMulFastOp::<usize, u16>::is_supported {
            return ClampedMulFastOp::<usize, u16>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u16) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/i32.
impl<V: Numeric> ClampedBinary<i32, V> for usize
where
    usize: CheckedBinary<i32, V>,
    V: Cast<i32>,
    i32: Cast<V>,
    <usize as Promote<i32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, i32>::is_supported {
            return ClampedAddFastOp::<usize, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i32) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, i32>::is_supported {
            return ClampedSubFastOp::<usize, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i32) -> V {
        if ClampedMulFastOp::<usize, i32>::is_supported {
            return ClampedMulFastOp::<usize, i32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/u32.
impl<V: Numeric> ClampedBinary<u32, V> for usize
where
    usize: CheckedBinary<u32, V>,
    V: Cast<u32>,
    i32: Cast<V>,
    <usize as Promote<u32>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, u32>::is_supported {
            return ClampedAddFastOp::<usize, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u32) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u32>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u32>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u32>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u32>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, u32>::is_supported {
            return ClampedSubFastOp::<usize, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u32) -> V {
        if ClampedMulFastOp::<usize, u32>::is_supported {
            return ClampedMulFastOp::<usize, u32>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u32) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/i64.
impl<V: Numeric> ClampedBinary<i64, V> for usize
where
    usize: CheckedBinary<i64, V>,
    V: Cast<i64>,
    i32: Cast<V>,
    <usize as Promote<i64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, i64>::is_supported {
            return ClampedAddFastOp::<usize, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: i64) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<i64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<i64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<i64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<i64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, i64>::is_supported {
            return ClampedSubFastOp::<usize, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: i64) -> V {
        if ClampedMulFastOp::<usize, i64>::is_supported {
            return ClampedMulFastOp::<usize, i64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: i64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/u64.
impl<V: Numeric> ClampedBinary<u64, V> for usize
where
    usize: CheckedBinary<u64, V>,
    V: Cast<u64>,
    i32: Cast<V>,
    <usize as Promote<u64>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, u64>::is_supported {
            return ClampedAddFastOp::<usize, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: u64) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<u64>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<u64>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<u64>>::Max as Numeric>::FLOAT)
                    || <V as Cast<u64>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, u64>::is_supported {
            return ClampedSubFastOp::<usize, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: u64) -> V {
        if ClampedMulFastOp::<usize, u64>::is_supported {
            return ClampedMulFastOp::<usize, u64>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: u64) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/isize.
impl<V: Numeric> ClampedBinary<isize, V> for usize
where
    usize: CheckedBinary<isize, V>,
    V: Cast<isize>,
    i32: Cast<V>,
    <usize as Promote<isize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, isize>::is_supported {
            return ClampedAddFastOp::<usize, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: isize) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<isize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<isize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<isize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<isize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, isize>::is_supported {
            return ClampedSubFastOp::<usize, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: isize) -> V {
        if ClampedMulFastOp::<usize, isize>::is_supported {
            return ClampedMulFastOp::<usize, isize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: isize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

// Source: clamped_math_impl.h:68-149, usize/usize.
impl<V: Numeric> ClampedBinary<usize, V> for usize
where
    usize: CheckedBinary<usize, V>,
    V: Cast<usize>,
    i32: Cast<V>,
    <usize as Promote<usize>>::Fast: Cast<V> + UncheckedInput,
{
    fn clamp_add(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedAddFastOp::<usize, usize>::is_supported {
            return ClampedAddFastOp::<usize, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if r.negative() { V::LOWEST } else { V::MAX };
        if self.add(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_sub(self, r: usize) -> V {
        const {
            assert!(
                (V::SIGNED == <<usize as Promote<usize>>::Max as Numeric>::SIGNED
                    && V::DIGITS == <<usize as Promote<usize>>::Max as Numeric>::DIGITS
                    && V::FLOAT == <<usize as Promote<usize>>::Max as Numeric>::FLOAT)
                    || <V as Cast<usize>>::CONTAINED
            );
        }
        if ClampedSubFastOp::<usize, usize>::is_supported {
            return ClampedSubFastOp::<usize, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if !r.negative() { V::LOWEST } else { V::MAX };
        if self.sub(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_mul(self, r: usize) -> V {
        if ClampedMulFastOp::<usize, usize>::is_supported {
            return ClampedMulFastOp::<usize, usize>::Do::<V>(self, r);
        }
        let mut result = V::ZERO;
        let saturated = if self.negative() ^ r.negative() {
            V::LOWEST
        } else {
            V::MAX
        };
        if self.mul(r, &mut result) {
            result
        } else {
            saturated
        }
    }
    fn clamp_div(self, r: usize) -> V {
        let mut result = V::ZERO;
        if self.div(r, &mut result) {
            result
        } else if self != 0 {
            if self.negative() ^ r.negative() {
                V::LOWEST
            } else {
                V::MAX
            }
        } else {
            V::nan()
        }
    }
}

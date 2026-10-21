// Source: include/private/SkMath.h:17-26
pub const SK_MaxS16: i16 = i16::MAX;
pub const SK_MinS16: i16 = -SK_MaxS16;
pub const SK_MaxS32: i32 = i32::MAX;
pub const SK_MinS32: i32 = -SK_MaxS32;
pub const SK_NaN32: i32 = i32::MIN;
pub const SK_MaxS64: i64 = i64::MAX;
pub const SK_MinS64: i64 = -SK_MaxS64;
// Source: include/private/SkMath.h:30-35
pub fn sk_64_mul(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}
// Source: include/private/SkMath.h:37-43
// Rust names the C++ overloads by operand width. Wrapping shifts retain the
// unsigned bit shift before conversion back to the signed representation.
pub const fn SkLeftShift32(value: i32, shift: i32) -> i32 {
    (value as u32).wrapping_shl(shift as u32) as i32
}
pub const fn SkLeftShift64(value: i64, shift: i32) -> i64 {
    (value as u64).wrapping_shl(shift as u32) as i64
}
// Source: include/private/SkMath.h:47-53
// Generic integer operators are represented with a minimal value trait,
// implementing the same (value & (value - 1)) == 0 expression for each width.
pub trait Pow2Integer: Copy {
    fn is_pow2(self) -> bool;
}
macro_rules! impl_pow2 {
    ($($t:ty),*) => {$(impl Pow2Integer for $t {
        fn is_pow2(self) -> bool {(self & self.wrapping_sub(1)) == 0}
    })*};
}
impl_pow2!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
pub fn SkIsPow2<T: Pow2Integer>(value: T) -> bool {
    value.is_pow2()
}
// Source: include/private/SkMath.h:57-67
pub fn SkMul16ShiftRound(a: u32, b: u32, shift: i32) -> u32 {
    crate::SkASSERT!(@source b"include/private/SkMath.h\0".as_ptr(), 62; @condition b"a <= 32767\0".as_ptr(); a <= 32767);
    crate::SkASSERT!(@source b"include/private/SkMath.h\0".as_ptr(), 63; @condition b"b <= 32767\0".as_ptr(); b <= 32767);
    crate::SkASSERT!(@source b"include/private/SkMath.h\0".as_ptr(), 64; @condition b"shift > 0 && shift <= 8\0".as_ptr(); shift > 0 && shift <= 8);
    let prod = a * b + (1u32 << (shift - 1));
    (prod + (prod >> shift)) >> shift
}
// Source: include/private/SkMath.h:69-75
pub fn SkMulDiv255Round(a: u32, b: u32) -> u32 {
    SkMul16ShiftRound(a, b, 8)
}

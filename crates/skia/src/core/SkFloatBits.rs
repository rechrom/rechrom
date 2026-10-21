// Source-aligned Skia translation. Rust methods use snake_case; fields retain upstream names.
#![allow(non_snake_case)]

// Source: src/core/SkFloatBits.h:20-26
#[inline]
pub fn SkSignBitTo2sCompliment(mut x: i32) -> i32 {
    if x < 0 {
        x &= 0x7fffffff;
        x = -x;
    }
    x
}

// Source: src/core/SkFloatBits.h:31-38
#[inline]
pub fn Sk2sComplimentToSignBit(mut x: i32) -> i32 {
    let sign = x >> 31;
    x = (x ^ sign).wrapping_sub(sign);
    x |= sign.wrapping_shl(31);
    x
}

// Source: src/core/SkFloatBits.h:41-45
#[inline]
pub fn SkFloat2Bits(value: f32) -> u32 {
    value.to_bits()
}

// Source: src/core/SkFloatBits.h:48-52
#[inline]
pub fn SkBits2Float(bits: u32) -> f32 {
    f32::from_bits(bits)
}

// Source: src/core/SkFloatBits.h:59-61
#[inline]
pub fn SkFloatAs2sCompliment(x: f32) -> i32 {
    SkSignBitTo2sCompliment(SkFloat2Bits(x) as i32)
}

// Source: src/core/SkFloatBits.h:66-68
#[inline]
pub fn Sk2sComplimentAsFloat(x: i32) -> f32 {
    SkBits2Float(Sk2sComplimentToSignBit(x) as u32)
}

// Source: src/core/SkFloatBits.h:72-72
pub use SkFloatAs2sCompliment as SkScalarAs2sCompliment;

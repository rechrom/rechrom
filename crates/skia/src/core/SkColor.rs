//! Source: src/core/SkColor.cpp; SkColorPriv.h premultiplication. See ../../LICENSE.
use crate::include::core::SkColor::SkColor4f;

pub fn premultiply(color: SkColor4f) -> [u8; 4] {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8;
    let alpha = byte(color.fA);
    [
        mul_div_255_round(byte(color.fR), alpha),
        mul_div_255_round(byte(color.fG), alpha),
        mul_div_255_round(byte(color.fB), alpha),
        alpha,
    ]
}

pub use crate::include::private::SkMath::SkMulDiv255Round as mul_div_255_round;

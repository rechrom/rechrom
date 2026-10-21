#![allow(non_snake_case)]
use super::SkRect::{SkIRect, SkRect};
// Source: src/core/SkValidationUtils.h:16-18
// Raw native enum storage permits validating invalid values without constructing an invalid Rust enum.
pub fn SkIsValidMode(mode: i32) -> bool {
    (mode as u32) <= super::SkBlendMode::SkBlendMode::kLastMode as u32
}
// Source: src/core/SkValidationUtils.h:22-24
pub fn SkIsValidIRect(rect: &SkIRect) -> bool {
    rect.width() >= 0 && rect.height() >= 0
}
// Source: src/core/SkValidationUtils.h:28-32
pub fn SkIsValidRect(rect: &SkRect) -> bool {
    (rect.fLeft <= rect.fRight)
        && (rect.fTop <= rect.fBottom)
        && super::SkFloatingPoint::SkIsFinite(&[rect.width(), rect.height()])
}

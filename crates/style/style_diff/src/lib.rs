#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Comparison of old and newly resolved canonical styles.
//!
//! The native `ComputedStyle` remains the value owner. This package owns the
//! pipeline comparison operation and returns Chromium's native difference
//! category without projecting or copying style state.

use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleDifference};
use layoutng_style::style::computed_style_base::ComputedStyleBase;

/// cpp: ComputedStyle::ComputeDifference.
#[inline]
pub fn ComputeDifference(
    old_style: &ComputedStyle,
    new_style: &ComputedStyle,
) -> ComputedStyleDifference {
    ComputedStyle::ComputeDifference(old_style, new_style)
}

/// cpp: ComputedStyleBase::FieldInvalidationDiff.
#[inline]
pub fn FieldInvalidationDiff(old_style: &ComputedStyle, new_style: &ComputedStyle) -> u64 {
    ComputedStyleBase::FieldInvalidationDiff(old_style, new_style)
}

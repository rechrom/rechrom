// C++: layoutng_inline/hyphen_result.cc. HyphenResult is physically owned by
// this package's hyphen_result_interface boundary source.
#![allow(non_snake_case)]

#[allow(unused_imports)]
use crate::algorithms::*;
use font_engine::HarfBuzzShaper;
use foundation::{Member, String};
use layout_base::internal::hyphen_result::HyphenResult;
use layout_style::style::computed_style::ComputedStyle;

// cpp: layoutng_inline/hyphen_result.cc:12-16
#[inline(never)]
pub fn ShapeHyphenResult(result: &mut HyphenResult, style: &ComputedStyle) {
    let hyphen = style.HyphenString();
    result.text_ = if hyphen.IsNull() {
        String::default()
    } else {
        // AtomicString may retain Latin-1 storage for the ordinary hyphen.
        // Blink's String(AtomicString) preserves either backing; requiring a
        // 16-bit span here made font/style changes expose a layout crash.
        String::from_utf16(hyphen.utf16_units().unwrap_or_default())
    };
    let shaper = HarfBuzzShaper::new(result.text_.clone());
    result.shape_result_ = Member::from_ptr(shaper.Shape(style.GetFont(), style.Direction()));
}

// cpp: layoutng/internal/hyphen_result.h:22-22,34-34
// The shaping members of base's HyphenResult record (Rust cannot add
// inherent methods to a base type; imported with `crate::algorithms`).
pub trait HyphenResultAlgorithms {
    fn new(style: &ComputedStyle) -> HyphenResult;
    fn Shape(&mut self, style: &ComputedStyle);
}

impl HyphenResultAlgorithms for HyphenResult {
    fn new(style: &ComputedStyle) -> HyphenResult {
        let mut result = HyphenResult::default();
        ShapeHyphenResult(&mut result, style);
        result
    }
    fn Shape(&mut self, style: &ComputedStyle) {
        ShapeHyphenResult(self, style)
    }
}

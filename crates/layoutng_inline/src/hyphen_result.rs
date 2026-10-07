// C++: layoutng_inline/hyphen_result.cc. HyphenResult is owned by //src/layoutng.
#![allow(non_snake_case)]

use font_engine::HarfBuzzShaper;
use foundation::{Member, String};
use layoutng::internal::hyphen_result::HyphenResult;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_inline/hyphen_result.cc:12-16
#[unsafe(no_mangle)]
pub extern "Rust" fn ShapeHyphenResult(result: &mut HyphenResult, style: &ComputedStyle) {
    let hyphen = style.HyphenString();
    result.text_ = if hyphen.IsNull() {
        String::default()
    } else {
        String::from_utf16(hyphen.Span16())
    };
    let shaper = HarfBuzzShaper::new(result.text_.clone());
    result.shape_result_ = Member::from_ptr(shaper.Shape(style.GetFont(), style.Direction()));
}

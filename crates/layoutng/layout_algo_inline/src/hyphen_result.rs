// C++: layoutng_inline/hyphen_result.cc. HyphenResult is physically owned by
// this package's native_hyphen_result boundary source.
#![allow(non_snake_case)]

use font_engine::HarfBuzzShaper;
use foundation::{Member, String};
use layoutng::internal::hyphen_result::HyphenResult;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_inline/hyphen_result.cc:12-16
#[unsafe(no_mangle)]
#[inline(never)]
pub extern "Rust" fn ShapeHyphenResult(result: &mut HyphenResult, style: &ComputedStyle) {
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

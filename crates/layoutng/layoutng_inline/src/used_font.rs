// C++: layoutng_inline/used_font.cc.
// UsedFont's class and fields are owned by //src/layoutng/internal/used_font.h.
// These exported Rust symbols match the cross-package calls in that type's
// existing translation once both source trees share one assembly crate.
#![allow(non_snake_case)]

use font_engine::FontBaseline;
use foundation::LayoutUnit;
use layoutng::internal::used_font::UsedFont;

// cpp: layoutng_inline/used_font.cc:9-15
#[no_mangle]
pub extern "Rust" fn UsedFontFixedAscentForBaseline(
    font: &UsedFont,
    baseline: FontBaseline,
) -> LayoutUnit {
    let font_data = font.PrimaryFont();
    if !font_data.is_null() {
        return LayoutUnit::from_f32(
            unsafe { &*font_data }
                .GetFontMetrics()
                .FloatAscentFor(baseline)
                * font.text_fit_scaling_factor_,
        );
    }
    LayoutUnit::default()
}

// cpp: layoutng_inline/used_font.cc:17-24
#[no_mangle]
pub extern "Rust" fn UsedFontFixedDescent(font: &UsedFont) -> LayoutUnit {
    let font_data = font.PrimaryFont();
    if !font_data.is_null() {
        return LayoutUnit::from_f32(
            unsafe { &*font_data }
                .GetFontMetrics()
                .FloatDescentFor(FontBaseline::kAlphabeticBaseline)
                * font.text_fit_scaling_factor_,
        );
    }
    LayoutUnit::default()
}

// cpp: layoutng_inline/used_font.cc:26-32
#[no_mangle]
pub extern "Rust" fn UsedFontFixedDescentForBaseline(
    font: &UsedFont,
    baseline: FontBaseline,
) -> LayoutUnit {
    let font_data = font.PrimaryFont();
    if !font_data.is_null() {
        return LayoutUnit::from_f32(
            unsafe { &*font_data }
                .GetFontMetrics()
                .FloatDescentFor(baseline)
                * font.text_fit_scaling_factor_,
        );
    }
    LayoutUnit::default()
}

// cpp: layoutng_inline/used_font.cc:34-42
#[no_mangle]
pub extern "Rust" fn UsedFontUnderlineThickness(font: &UsedFont) -> Option<f32> {
    let font_data = font.PrimaryFont();
    if !font_data.is_null() {
        if let Some(thickness) = unsafe { &*font_data }.GetFontMetrics().UnderlineThickness() {
            return Some(thickness * font.text_fit_scaling_factor_);
        }
    }
    None
}

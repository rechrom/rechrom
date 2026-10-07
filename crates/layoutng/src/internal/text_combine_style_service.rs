#![allow(non_snake_case)]

use layoutng_style::style::computed_style::ComputedStyleBuilder;

// The implementations belong to //src/layoutng_inline/text_combine_style_service.cc.
unsafe extern "Rust" {
    // cpp: layoutng/internal/text_combine_style_service.h:9
    pub fn AdjustStyleForCombinedText(builder: &mut ComputedStyleBuilder);

    // cpp: layoutng/internal/text_combine_style_service.h:10
    pub fn AdjustStyleForTextCombine(builder: &mut ComputedStyleBuilder);
}

#![allow(non_snake_case, unused_unsafe)]

use layout_style::style::computed_style::ComputedStyleBuilder;

// Bodies: base `inline/text_combine_style_service.rs` (snapshot layoutng_inline).
// Chromium core/layout/layout_text_combine.cc (AssertStyleIsValid) and style_adjuster.cc text-combine adjustments.
#[cfg(feature = "translation_in_progress")]
#[path = "../inline/text_combine_style_service.rs"]
pub mod text_combine_style_members;

#[cfg(feature = "translation_in_progress")]
pub use self::text_combine_style_members::{
    AdjustStyleForCombinedText,
    AdjustStyleForTextCombine,
};
#[cfg(not(feature = "translation_in_progress"))]
unsafe extern "Rust" {
    // cpp: layoutng/internal/text_combine_style_service.h:9
    pub fn AdjustStyleForCombinedText(builder: &mut ComputedStyleBuilder);

    // cpp: layoutng/internal/text_combine_style_service.h:10
    pub fn AdjustStyleForTextCombine(builder: &mut ComputedStyleBuilder);
}

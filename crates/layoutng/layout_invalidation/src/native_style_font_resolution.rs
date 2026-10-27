//! Native style-input font validation and resolver lookup.
//!
//! Included by `boundary::native_input` at the reciprocal assembly boundary.
//! The StyleEngine identity fast path bypasses reconstruction entirely; when
//! reconstruction is required, this module owns input validation and request
//! projection. The font engine remains responsible for selection and caching.

use super::*;

#[inline(never)]
pub(super) fn ResolveNativeStyleFont(
    builder: &mut ComputedStyleBuilder,
    extra: &ExtendedStyle,
    fonts: &mut dyn NativeFontResolver,
) {
    let font_writing_mode = match builder.GetWritingMode() {
        NativeWritingMode::kHorizontalTb => WritingMode::kHorizontalTb,
        NativeWritingMode::kVerticalRl => WritingMode::kVerticalRl,
        NativeWritingMode::kVerticalLr => WritingMode::kVerticalLr,
        _ => Invalid("font writing mode outside public input contract"),
    };
    Number(extra.font_size, "font_size", true);
    if !extra.font_weight.is_finite() || extra.font_weight < 1.0 || extra.font_weight > 1000.0 {
        Invalid("font_weight");
    }
    Number(extra.letter_spacing, "letter_spacing", false);
    Number(extra.word_spacing, "word_spacing", false);
    // The public Rust strings are valid UTF-8 by construction.
    for family in &extra.font_families {
        if family.is_empty() {
            Invalid("font_family UTF-8");
        }
    }
    let request = NativeFontRequest::new_auto(
        extra.font_size,
        extra.letter_spacing,
        extra.word_spacing,
        font_writing_mode,
        &extra.language,
        &extra.font_families,
        extra.font_weight,
        extra.font_italic,
        builder.ComputeFontOrientation(),
    );
    let font = fonts.Resolve(&request);
    builder.SetFont(Member::from_ptr(font as *mut _));
}

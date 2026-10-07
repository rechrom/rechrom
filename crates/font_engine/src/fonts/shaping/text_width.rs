#![allow(non_snake_case)]

use crate::fonts::font::Font;
use crate::text::native::bidi_paragraph::BidiParagraph;
use foundation::StringView;

use super::harfbuzz_shaper::HarfBuzzShaper;

// The signature comes from the public shaping header. Both shaper and bidi
// interfaces are same-package dependencies whose source files remain pending.
// cpp: font_engine/fonts/shaping/text_width.h:10-14
// cpp: font_engine/fonts/shaping/text_width.cc:13-26
pub fn ComputeTextWidth(text: &StringView, font: *const Font) -> f32 {
    if text.IsEmpty() {
        return 0.0;
    }
    let shaper = HarfBuzzShaper::new(text.ToString());
    let shape = shaper.Shape(font, BidiParagraph::BaseDirectionForStringOrLtr(text));
    if shape.is_null() {
        0.0
    } else {
        unsafe { &*shape }.Width()
    }
}

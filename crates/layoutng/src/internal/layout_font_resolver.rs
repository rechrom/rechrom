use font_engine::{Font, FontOrientation, SimpleFontData};

use super::layout_input::{FontSmoothing, FontVariation, WritingMode};

// cpp: layoutng/internal/layout_font_resolver.h:18-31
pub struct NativeFontRequest<'a> {
    pub size: f64,
    pub specified_size: f64,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub writing_mode: WritingMode,
    pub language: &'a str,
    pub families: &'a [String],
    pub weight: f64,
    pub italic: bool,
    pub orientation: FontOrientation,
    pub smoothing: FontSmoothing,
}

impl<'a> NativeFontRequest<'a> {
    // cpp: layoutng/internal/layout_font_resolver.h:30-30
    pub fn new_auto(
        size: f64,
        letter_spacing: f64,
        word_spacing: f64,
        writing_mode: WritingMode,
        language: &'a str,
        families: &'a [String],
        weight: f64,
        italic: bool,
        orientation: FontOrientation,
    ) -> Self {
        Self {
            size,
            specified_size: size,
            letter_spacing,
            word_spacing,
            writing_mode,
            language,
            families,
            weight,
            italic,
            orientation,
            smoothing: FontSmoothing::kAuto,
        }
    }
}

// cpp: layoutng/internal/layout_font_resolver.h:33-42
#[allow(non_snake_case)]
pub trait NativeFontResolver {
    fn Resolve(&mut self, request: &NativeFontRequest<'_>) -> &mut Font;

    fn FaceIndex(&self, _font_data: *const SimpleFontData) -> usize {
        0
    }

    fn Variations(&self, _font_data: *const SimpleFontData) -> &[FontVariation] {
        &[]
    }
}

// cpp: layoutng/internal/layout_font_resolver.h:44-44
// The implementation is in boundary/native_input.cc, translated with that file.
pub use super::boundary::native_input::CurrentNativeFontResolver;

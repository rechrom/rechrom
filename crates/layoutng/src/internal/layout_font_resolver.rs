use font_engine::{Font, FontDescription, FontOrientation, SimpleFontData};

use super::layout_input::{FontSmoothing, FontVariation, WritingMode};

// cpp: layoutng/internal/layout_font_resolver.h:18-31
pub struct NativeFontRequest<'a> {
    /// The immutable computed description produced by Style. Layout only
    /// rebinds it to the ConstraintSpace's resolved faces; it must not rebuild
    /// the description from the smaller face-selection key.
    pub base_description: Option<&'a FontDescription>,
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
            base_description: None,
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

    pub fn WithBaseDescription(mut self, description: &'a FontDescription) -> Self {
        self.base_description = Some(description);
        self.specified_size = description.SpecifiedSize() as f64;
        self.smoothing = match description.FontSmoothing() {
            font_engine::FontSmoothingMode::kNoSmoothing => FontSmoothing::kNone,
            font_engine::FontSmoothingMode::kAntialiased => FontSmoothing::kAntialiased,
            font_engine::FontSmoothingMode::kSubpixelAntialiased => {
                FontSmoothing::kSubpixelAntialiased
            }
            font_engine::FontSmoothingMode::kAutoSmoothing => FontSmoothing::kAuto,
        };
        self
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

// Layout carries writing-mode context, while font selection only needs the
// resolved orientation. Matching and caching belong entirely to FontEngine.
impl NativeFontResolver for font_engine::FontEngine {
    fn Resolve(&mut self, request: &NativeFontRequest<'_>) -> &mut Font {
        font_engine::FontEngine::ResolveFont(
            self,
            &font_engine::resolver::FontRequest {
                base_description: request.base_description,
                size: request.size,
                specified_size: request.specified_size,
                letter_spacing: request.letter_spacing,
                word_spacing: request.word_spacing,
                language: request.language,
                families: request.families,
                weight: request.weight,
                italic: request.italic,
                orientation: request.orientation,
                smoothing: request.smoothing,
            },
        )
    }

    fn FaceIndex(&self, font_data: *const SimpleFontData) -> usize {
        font_engine::FontEngine::FaceIndex(self, font_data)
    }

    fn Variations(&self, font_data: *const SimpleFontData) -> &[FontVariation] {
        font_engine::FontEngine::Variations(self, font_data)
    }
}

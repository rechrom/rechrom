// Platform font-catalog metadata used by FontFallbackIterator's translated
// selection stages.  The browser owns discovery of concrete platform faces;
// FontEngine owns character-, locale-, and presentation-sensitive selection.

// cpp: font_engine/fonts/font_fallback_iterator.cc:99-198
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FontFallbackRole {
    /// A CSS family face (for example @font-face). It participates only when
    /// its family was requested and never leaks into platform fallback.
    #[default]
    kFamilyOnly,
    /// A normal system text face. This is also the safe default for embedders
    /// which explicitly publish a platform fallback catalog.
    kText,
    /// A monochrome symbol face, including text-presentation emoji.
    kSymbol,
    /// A color emoji face.
    kEmoji,
    /// A face with OpenType math coverage.
    kMath,
    /// The platform last-resort face. It is tried after character-bearing
    /// system candidates and before the first family candidate's .notdef.
    kLastResort,
}

/// Immutable facts about one resolved face. `locales` contains BCP-47 tags in
/// preference order; an empty list means the face is language-neutral.
#[derive(Clone, Debug, PartialEq)]
pub struct FontFallbackMetadata {
    pub role: FontFallbackRole,
    pub locales: Vec<String>,
    pub weight: f32,
    pub italic: bool,
}

impl Default for FontFallbackMetadata {
    fn default() -> Self {
        Self {
            role: FontFallbackRole::kFamilyOnly,
            locales: Vec::new(),
            weight: 400.0,
            italic: false,
        }
    }
}

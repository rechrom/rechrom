// C++: font_engine/fonts/font_platform_data.h/.cc. HarfBuzzFace and its
// back-reference remain GC members; OpenTypeFont and the native backend use
// reference counting as in the source.
use super::font_backend::FontBackend;
use super::font_orientation::{FontOrientation, IsVerticalAnyUpright, IsVerticalNonCJKUpright};
use super::resolved_font_features::ResolvedFontFeatures;
use super::shaping::harfbuzz_face::HarfBuzzFace;
use super::text_rendering_mode::TextRenderingMode;
use crate::text::native::opentype_font::OpenTypeFont;
use foundation::{MakeGarbageCollected, Member, Traceable, Visitor};
use std::cell::RefCell;
use std::sync::Arc;

// cpp: font_engine/fonts/font_platform_data.h:54-151
pub struct FontPlatformData {
    typeface_: Option<Arc<OpenTypeFont>>,
    backend_: Option<Arc<dyn FontBackend>>,
    pub text_size_: f32,
    pub synthetic_bold_: bool,
    pub synthetic_italic_: bool,
    pub avoid_embedded_bitmaps_: bool,
    pub text_rendering_: TextRenderingMode,
    pub orientation_: FontOrientation,
    pub resolved_font_features_: ResolvedFontFeatures,
    pub unicode_ranges_: Vec<(u32, u32)>,
    harfbuzz_face_: RefCell<Member<HarfBuzzFace>>,
    is_hash_table_deleted_value_: bool,
}

// cpp: font_engine/fonts/font_platform_data.cc:31-44
impl Default for FontPlatformData {
    fn default() -> Self {
        Self {
            typeface_: None,
            backend_: None,
            text_size_: 0.0,
            synthetic_bold_: false,
            synthetic_italic_: false,
            avoid_embedded_bitmaps_: false,
            text_rendering_: TextRenderingMode::kAutoTextRendering,
            orientation_: FontOrientation::kHorizontal,
            resolved_font_features_: Vec::new(),
            unicode_ranges_: Vec::new(),
            harfbuzz_face_: RefCell::new(Member::default()),
            is_hash_table_deleted_value_: false,
        }
    }
}

impl Clone for FontPlatformData {
    fn clone(&self) -> Self {
        Self {
            typeface_: self.typeface_.clone(),
            backend_: self.backend_.clone(),
            text_size_: self.text_size_,
            synthetic_bold_: self.synthetic_bold_,
            synthetic_italic_: self.synthetic_italic_,
            avoid_embedded_bitmaps_: self.avoid_embedded_bitmaps_,
            text_rendering_: self.text_rendering_,
            orientation_: self.orientation_,
            resolved_font_features_: self.resolved_font_features_.clone(),
            unicode_ranges_: self.unicode_ranges_.clone(),
            harfbuzz_face_: RefCell::new(Member::default()),
            is_hash_table_deleted_value_: false,
        }
    }
}

impl FontPlatformData {
    pub fn deleted() -> Self {
        Self {
            is_hash_table_deleted_value_: true,
            ..Self::default()
        }
    }

    // cpp: font_engine/fonts/font_platform_data.cc:46-79
    pub fn with_text_size(&self, text_size: f32) -> Self {
        Self::new(
            self.typeface_
                .clone()
                .expect("empty font-cache key has no font data"),
            self.backend_.clone(),
            text_size,
            self.synthetic_bold_,
            self.synthetic_italic_,
            self.text_rendering_,
            self.resolved_font_features_.clone(),
            self.orientation_,
            self.unicode_ranges_.clone(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        typeface: Arc<OpenTypeFont>,
        backend: Option<Arc<dyn FontBackend>>,
        text_size: f32,
        synthetic_bold: bool,
        synthetic_italic: bool,
        text_rendering: TextRenderingMode,
        resolved_font_features: ResolvedFontFeatures,
        orientation: FontOrientation,
        unicode_ranges: Vec<(u32, u32)>,
    ) -> Self {
        Self {
            typeface_: Some(typeface.WithSize(text_size as f64)),
            backend_: backend,
            text_size_: text_size,
            synthetic_bold_: synthetic_bold,
            synthetic_italic_: synthetic_italic,
            avoid_embedded_bitmaps_: false,
            text_rendering_: text_rendering,
            orientation_: orientation,
            resolved_font_features_: resolved_font_features,
            unicode_ranges_: unicode_ranges,
            harfbuzz_face_: RefCell::new(Member::default()),
            is_hash_table_deleted_value_: false,
        }
    }

    // cpp: font_engine/fonts/font_platform_data.h:88-124
    pub fn size(&self) -> f32 {
        self.text_size_
    }
    pub fn SyntheticBold(&self) -> bool {
        self.synthetic_bold_
    }
    pub fn SyntheticItalic(&self) -> bool {
        self.synthetic_italic_
    }
    pub fn Backend(&self) -> Option<&dyn FontBackend> {
        self.backend_.as_deref()
    }
    pub fn Orientation(&self) -> FontOrientation {
        self.orientation_
    }
    pub fn ResolvedFeatures(&self) -> &ResolvedFontFeatures {
        &self.resolved_font_features_
    }
    pub fn IsVerticalAnyUpright(&self) -> bool {
        IsVerticalAnyUpright(self.orientation_)
    }
    pub fn IsVerticalNonCJKUpright(&self) -> bool {
        IsVerticalNonCJKUpright(self.orientation_)
    }
    pub fn SetOrientation(&mut self, orientation: FontOrientation) {
        self.orientation_ = orientation;
    }
    pub fn SetSyntheticBold(&mut self, synthetic_bold: bool) {
        self.synthetic_bold_ = synthetic_bold;
    }
    pub fn SetSyntheticItalic(&mut self, synthetic_italic: bool) {
        self.synthetic_italic_ = synthetic_italic;
    }
    pub fn SetAvoidEmbeddedBitmaps(&mut self, embedded_bitmaps: bool) {
        self.avoid_embedded_bitmaps_ = embedded_bitmaps;
    }
    pub fn IsHashTableDeletedValue(&self) -> bool {
        self.is_hash_table_deleted_value_
    }

    // cpp: font_engine/fonts/font_platform_data.cc:84-87
    pub fn RawFont(&self) -> &OpenTypeFont {
        self.typeface_
            .as_deref()
            .expect("empty font-cache key has no font data")
    }

    // cpp: font_engine/fonts/font_platform_data.cc:89-119
    pub fn UniqueID(&self) -> u32 {
        self.typeface_
            .as_ref()
            .map_or(0, |font| font.FaceIdentity())
    }

    pub fn GetHash(&self) -> u32 {
        let mut h = self.UniqueID();
        h ^= 0x01010101u32.wrapping_mul(
            ((self.is_hash_table_deleted_value_ as u32) << 3)
                | ((self.orientation_ as u32) << 2)
                | ((self.synthetic_bold_ as u32) << 1)
                | (self.synthetic_italic_ as u32),
        );
        h ^ self.text_size_.to_bits()
    }

    pub fn FontFamilyName(&self) -> String {
        self.RawFont().FamilyName()
    }

    pub fn GetPostScriptName(&self) -> String {
        self.typeface_
            .as_ref()
            .map_or_else(String::new, |font| font.PostScriptName())
    }

    // cpp: font_engine/fonts/font_platform_data.cc:120-133
    pub fn FontContainsCharacter(&self, character: u32) -> bool {
        if !self.unicode_ranges_.is_empty()
            && !self
                .unicode_ranges_
                .iter()
                .any(|&(start, end)| character >= start && character <= end)
        {
            return false;
        }
        self.RawFont().GlyphForCharacter(character).is_some()
            // HarfBuzz synthesizes NBSP from the face's space glyph when
            // cmap omits NBSP. Keep that successful shaping result in the
            // current face; unicode-range above still applies to NBSP itself.
            || (character == 0xa0 && self.RawFont().GlyphForCharacter(0x20).is_some())
    }

    pub fn GetHarfBuzzFace(&self) -> *mut HarfBuzzFace {
        let mut face = self.harfbuzz_face_.borrow_mut();
        if face.Get().is_null() {
            *face = Member::from_ptr(MakeGarbageCollected(HarfBuzzFace::new(self)));
        }
        face.Get()
    }
}

impl PartialEq for FontPlatformData {
    fn eq(&self, other: &Self) -> bool {
        self.UniqueID() == other.UniqueID()
            && self.text_size_ == other.text_size_
            && self.is_hash_table_deleted_value_ == other.is_hash_table_deleted_value_
            && self.synthetic_bold_ == other.synthetic_bold_
            && self.synthetic_italic_ == other.synthetic_italic_
            && self.avoid_embedded_bitmaps_ == other.avoid_embedded_bitmaps_
            && self.text_rendering_ == other.text_rendering_
            && self.resolved_font_features_ == other.resolved_font_features_
            && self.orientation_ == other.orientation_
            && self.unicode_ranges_ == other.unicode_ranges_
    }
}

impl Traceable for FontPlatformData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&*self.harfbuzz_face_.borrow());
    }
}

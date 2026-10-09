// C++: font_engine/fonts/font_variant_alternates.h
// Construction, setters, IsNormal and inline state/accessors are mapped.
// Font-feature-values resolution, hash and equality remain untranslated.
use foundation::AtomicString;

use super::resolved_font_features::ResolvedFontFeatures;

// cpp: font_engine/fonts/font_variant_alternates.h:20-21,87-100
#[derive(Default)]
pub struct FontVariantAlternates {
    stylistic_: Option<AtomicString>,
    swash_: Option<AtomicString>,
    ornaments_: Option<AtomicString>,
    annotation_: Option<AtomicString>,
    styleset_: Vec<AtomicString>,
    character_variant_: Vec<AtomicString>,
    historical_forms_: bool,
    resolved_features_: ResolvedFontFeatures,
    is_resolved_: bool,
}

#[allow(non_snake_case)]
impl FontVariantAlternates {
    // cpp: platform/fonts/font_variant_alternates.cc:13,71-105.
    pub fn Create() -> Self { Self::default() }
    pub fn IsNormal(&self) -> bool {
        self.stylistic_.is_none() && !self.historical_forms_ && self.swash_.is_none()
            && self.ornaments_.is_none() && self.annotation_.is_none()
            && self.styleset_.is_empty() && self.character_variant_.is_empty()
    }
    pub fn SetStylistic(&mut self, value: AtomicString) { self.stylistic_ = Some(value); }
    pub fn SetSwash(&mut self, value: AtomicString) { self.swash_ = Some(value); }
    pub fn SetOrnaments(&mut self, value: AtomicString) { self.ornaments_ = Some(value); }
    pub fn SetAnnotation(&mut self, value: AtomicString) { self.annotation_ = Some(value); }
    pub fn SetHistoricalForms(&mut self) { self.historical_forms_ = true; }
    pub fn SetStyleset(&mut self, value: Vec<AtomicString>) { self.styleset_ = value; }
    pub fn SetCharacterVariant(&mut self, value: Vec<AtomicString>) { self.character_variant_ = value; }

    // cpp: font_engine/fonts/font_variant_alternates.h:36-38
    pub fn Stylistic(&self) -> *const AtomicString {
        self.stylistic_
            .as_ref()
            .map_or(std::ptr::null(), |value| value)
    }

    // cpp: font_engine/fonts/font_variant_alternates.h:39
    pub fn HistoricalForms(&self) -> bool {
        self.historical_forms_
    }

    // cpp: font_engine/fonts/font_variant_alternates.h:40
    pub fn Swash(&self) -> *const AtomicString {
        self.swash_.as_ref().map_or(std::ptr::null(), |value| value)
    }

    // cpp: font_engine/fonts/font_variant_alternates.h:41-43
    pub fn Ornaments(&self) -> *const AtomicString {
        self.ornaments_
            .as_ref()
            .map_or(std::ptr::null(), |value| value)
    }

    // cpp: font_engine/fonts/font_variant_alternates.h:44-46
    pub fn Annotation(&self) -> *const AtomicString {
        self.annotation_
            .as_ref()
            .map_or(std::ptr::null(), |value| value)
    }

    // cpp: font_engine/fonts/font_variant_alternates.h:48
    pub fn Styleset(&self) -> &[AtomicString] {
        &self.styleset_
    }

    // cpp: font_engine/fonts/font_variant_alternates.h:49-51
    pub fn CharacterVariant(&self) -> &[AtomicString] {
        &self.character_variant_
    }
}

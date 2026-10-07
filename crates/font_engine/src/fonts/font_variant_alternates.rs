// C++: font_engine/fonts/font_variant_alternates.h
// The supplied tree declares but does not define construction, setters,
// resolution, hash, equality, or IsNormal. This state and its inline
// accessors are mapped; the file remains blocked.
use foundation::AtomicString;

use super::resolved_font_features::ResolvedFontFeatures;

// cpp: font_engine/fonts/font_variant_alternates.h:20-21,87-100
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

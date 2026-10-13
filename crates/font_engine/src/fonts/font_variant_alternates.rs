// C++: font_engine/fonts/font_variant_alternates.h
// Named alternate resolution follows platform/fonts/font_variant_alternates.cc.
use foundation::AtomicString;

use super::resolved_font_features::ResolvedFontFeatures;
use super::shaping::font_features::FontFeatureValue;

// cpp: font_engine/fonts/font_variant_alternates.h:20-21,87-100
#[derive(Clone, Debug, Default)]
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

// cpp: platform/fonts/font_variant_alternates.cc:215-226.
// is_resolved_ is a getter precondition, not semantic font/cache identity.
impl PartialEq for FontVariantAlternates {
    fn eq(&self, other: &Self) -> bool {
        self.stylistic_ == other.stylistic_
            && self.historical_forms_ == other.historical_forms_
            && self.styleset_ == other.styleset_
            && self.character_variant_ == other.character_variant_
            && self.swash_ == other.swash_
            && self.ornaments_ == other.ornaments_
            && self.annotation_ == other.annotation_
            && self.resolved_features_ == other.resolved_features_
    }
}
impl Eq for FontVariantAlternates {}

#[allow(non_snake_case)]
impl FontVariantAlternates {
    // cpp: platform/fonts/font_variant_alternates.cc:13,71-105.
    pub fn Create() -> Self {
        Self::default()
    }

    // cpp: platform/fonts/font_variant_alternates.cc:44-64.
    pub fn GetResolvedFontFeatures(&self) -> &ResolvedFontFeatures {
        debug_assert!(self.is_resolved_);
        &self.resolved_features_
    }

    pub fn Clone(other: &Self) -> Self {
        let mut result = other.clone();
        result.is_resolved_ = false;
        result
    }

    // cpp: platform/fonts/font_variant_alternates.cc:100-190.
    // Resolve a copy for one requested CSS family. The symbolic value in the
    // canonical computed description remains available for inheritance.
    pub fn Resolve(
        &self,
        resolve_stylistic: impl Fn(&AtomicString) -> Vec<u32>,
        resolve_styleset: impl Fn(&AtomicString) -> Vec<u32>,
        resolve_character_variant: impl Fn(&AtomicString) -> Vec<u32>,
        resolve_swash: impl Fn(&AtomicString) -> Vec<u32>,
        resolve_ornaments: impl Fn(&AtomicString) -> Vec<u32>,
        resolve_annotation: impl Fn(&AtomicString) -> Vec<u32>,
    ) -> Self {
        let mut result = Self::Clone(self);
        let mut push = |tag: [u8; 4], value| {
            result.resolved_features_.push(FontFeatureValue {
                tag: u32::from_be_bytes(tag),
                value,
            });
        };
        if let Some(alias) = &self.swash_ {
            let indices = resolve_swash(alias);
            if !indices.is_empty() {
                assert_eq!(indices.len(), 1);
                push(*b"swsh", indices[0]);
                push(*b"cswh", indices[0]);
            }
        }
        if let Some(alias) = &self.ornaments_ {
            let indices = resolve_ornaments(alias);
            if !indices.is_empty() {
                assert_eq!(indices.len(), 1);
                push(*b"ornm", indices[0]);
            }
        }
        if let Some(alias) = &self.annotation_ {
            let indices = resolve_annotation(alias);
            if !indices.is_empty() {
                assert_eq!(indices.len(), 1);
                push(*b"nalt", indices[0]);
            }
        }
        if let Some(alias) = &self.stylistic_ {
            let indices = resolve_stylistic(alias);
            if !indices.is_empty() {
                assert_eq!(indices.len(), 1);
                push(*b"salt", indices[0]);
            }
        }
        for alias in &self.styleset_ {
            for index in resolve_styleset(alias) {
                // The pinned implementation accepts zero as well as 1..99.
                if index <= 99 {
                    push(
                        [
                            b's',
                            b's',
                            b'0' + (index / 10) as u8,
                            b'0' + (index % 10) as u8,
                        ],
                        1,
                    );
                }
            }
        }
        for alias in &self.character_variant_ {
            let indices = resolve_character_variant(alias);
            if !indices.is_empty() && indices.len() <= 2 && indices[0] <= 99 {
                let index = indices[0];
                push(
                    [
                        b'c',
                        b'v',
                        b'0' + (index / 10) as u8,
                        b'0' + (index % 10) as u8,
                    ],
                    indices.get(1).copied().unwrap_or(1),
                );
            }
        }
        if self.historical_forms_ {
            push(*b"hist", 1);
        }
        result.is_resolved_ = true;
        result
    }
    pub fn IsNormal(&self) -> bool {
        self.stylistic_.is_none()
            && !self.historical_forms_
            && self.swash_.is_none()
            && self.ornaments_.is_none()
            && self.annotation_.is_none()
            && self.styleset_.is_empty()
            && self.character_variant_.is_empty()
    }
    pub fn SetStylistic(&mut self, value: AtomicString) {
        self.stylistic_ = Some(value);
    }
    pub fn SetSwash(&mut self, value: AtomicString) {
        self.swash_ = Some(value);
    }
    pub fn SetOrnaments(&mut self, value: AtomicString) {
        self.ornaments_ = Some(value);
    }
    pub fn SetAnnotation(&mut self, value: AtomicString) {
        self.annotation_ = Some(value);
    }
    pub fn SetHistoricalForms(&mut self) {
        self.historical_forms_ = true;
    }
    pub fn SetStyleset(&mut self, value: Vec<AtomicString>) {
        self.styleset_ = value;
    }
    pub fn SetCharacterVariant(&mut self, value: Vec<AtomicString>) {
        self.character_variant_ = value;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn alias(value: &str) -> AtomicString {
        AtomicString::FromUtf8(value.as_bytes())
    }
    fn no_lookup(_: &AtomicString) -> Vec<u32> {
        vec![]
    }
    fn feature(tag: &[u8; 4], value: u32) -> FontFeatureValue {
        FontFeatureValue {
            tag: u32::from_be_bytes(*tag),
            value,
        }
    }

    #[test]
    fn resolves_all_six_types_in_source_order_without_changing_symbolic_value() {
        let mut value = FontVariantAlternates::Create();
        value.SetStylistic(alias("stylistic"));
        value.SetSwash(alias("swash"));
        value.SetOrnaments(alias("ornaments"));
        value.SetAnnotation(alias("annotation"));
        value.SetStyleset(vec![alias("set"), alias("unknown")]);
        value.SetCharacterVariant(vec![alias("variant"), alias("default")]);
        value.SetHistoricalForms();
        let symbolic = value.clone();
        let resolved = value.Resolve(
            |name| {
                assert_eq!(name, &alias("stylistic"));
                vec![4]
            },
            |name| {
                if name == &alias("set") {
                    vec![2, 0, 99, 100]
                } else {
                    vec![]
                }
            },
            |name| {
                if name == &alias("variant") {
                    vec![8, 3]
                } else {
                    vec![9]
                }
            },
            |name| {
                assert_eq!(name, &alias("swash"));
                vec![5]
            },
            |name| {
                assert_eq!(name, &alias("ornaments"));
                vec![6]
            },
            |name| {
                assert_eq!(name, &alias("annotation"));
                vec![7]
            },
        );
        assert_eq!(
            resolved.GetResolvedFontFeatures(),
            &[
                feature(b"swsh", 5),
                feature(b"cswh", 5),
                feature(b"ornm", 6),
                feature(b"nalt", 7),
                feature(b"salt", 4),
                feature(b"ss02", 1),
                feature(b"ss00", 1),
                feature(b"ss99", 1),
                feature(b"cv08", 3),
                feature(b"cv09", 1),
                feature(b"hist", 1),
            ]
        );
        assert_eq!(value, symbolic);
        assert!(!value.is_resolved_);
        assert_eq!(resolved.Styleset(), symbolic.Styleset());
    }

    #[test]
    fn missing_aliases_still_resolve_historical_forms() {
        let mut value = FontVariantAlternates::Create();
        value.SetStyleset(vec![alias("missing")]);
        value.SetSwash(alias("missing"));
        value.SetHistoricalForms();
        let resolved = value.Resolve(
            no_lookup, no_lookup, no_lookup, no_lookup, no_lookup, no_lookup,
        );
        assert_eq!(resolved.GetResolvedFontFeatures(), &[feature(b"hist", 1)]);
    }

    #[test]
    fn character_variant_bounds_cardinality_and_zero_match_source() {
        let mut value = FontVariantAlternates::Create();
        value.SetCharacterVariant(
            ["zero", "max", "large", "too-many", "missing"]
                .map(alias)
                .to_vec(),
        );
        let resolved = value.Resolve(
            no_lookup,
            no_lookup,
            |name| match name.Utf8().as_str() {
                "zero" => vec![0, 0],
                "max" => vec![99],
                "large" => vec![100],
                "too-many" => vec![3, 4, 5],
                _ => vec![],
            },
            no_lookup,
            no_lookup,
            no_lookup,
        );
        assert_eq!(
            resolved.GetResolvedFontFeatures(),
            &[feature(b"cv00", 0), feature(b"cv99", 1)]
        );
    }

    #[test]
    fn semantic_equality_compares_resolved_values_and_ignores_getter_precondition() {
        let mut value = FontVariantAlternates::Create();
        value.SetStyleset(vec![alias("named")]);
        let first = value.Resolve(
            no_lookup,
            |_| vec![2],
            no_lookup,
            no_lookup,
            no_lookup,
            no_lookup,
        );
        let same = value.Resolve(
            no_lookup,
            |_| vec![2],
            no_lookup,
            no_lookup,
            no_lookup,
            no_lookup,
        );
        let changed = value.Resolve(
            no_lookup,
            |_| vec![3],
            no_lookup,
            no_lookup,
            no_lookup,
            no_lookup,
        );
        assert_eq!(first, same);
        assert_eq!(first, first.clone());
        assert_eq!(first, FontVariantAlternates::Clone(&first));
        assert_ne!(first, changed);
        assert_ne!(value, first);
    }
}

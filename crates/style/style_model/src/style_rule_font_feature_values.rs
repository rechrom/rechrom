// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/style_rule_font_feature_values.h
// cpp: third_party/blink/renderer/core/css/style_rule_font_feature_values.cc
// Oilpan tracing/allocation and exported-symbol declarations are represented
// by Rust ownership; all rule/storage behavior from the source pair is here.

#![allow(non_snake_case, non_camel_case_types)]

use foundation::{AtomicString, String};
use std::collections::HashMap;

// cpp: style_rule_font_feature_values.h:14-19
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureIndicesWithPriority {
    pub indices: Vec<u32>,
    pub layer_order: u16,
}
impl FeatureIndicesWithPriority {
    fn new(indices: Vec<u32>) -> Self {
        Self {
            indices,
            layer_order: u16::MAX,
        }
    }
}

pub type FontFeatureAliases = HashMap<AtomicString, FeatureIndicesWithPriority>;

// cpp: style_rule_font_feature_values.h:22-48
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFeatureType {
    kStylistic,
    kStyleset,
    kCharacterVariant,
    kSwash,
    kOrnaments,
    kAnnotation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleRuleFontFeature {
    type_: FontFeatureType,
    feature_aliases_: FontFeatureAliases,
}
impl StyleRuleFontFeature {
    // cpp: style_rule_font_feature_values.cc:15-17
    pub fn new(type_: FontFeatureType) -> Self {
        Self {
            type_,
            feature_aliases_: HashMap::new(),
        }
    }
    // cpp: style_rule_font_feature_values.cc:26-31
    pub fn UpdateAlias(&mut self, alias: AtomicString, features: Vec<u32>) {
        self.feature_aliases_
            .insert(alias, FeatureIndicesWithPriority::new(features));
    }
    // cpp: style_rule_font_feature_values.cc:33-37
    pub fn OverrideAliasesIn(&self, destination: &mut FontFeatureAliases) {
        for (key, value) in &self.feature_aliases_ {
            destination.insert(key.clone(), value.clone());
        }
    }
    pub fn GetFeatureType(&self) -> FontFeatureType {
        self.type_
    }
}

// cpp: style_rule_font_feature_values.h:53-92
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontFeatureValuesStorage {
    stylistic_: FontFeatureAliases,
    styleset_: FontFeatureAliases,
    character_variant_: FontFeatureAliases,
    swash_: FontFeatureAliases,
    ornaments_: FontFeatureAliases,
    annotation_: FontFeatureAliases,
}
impl FontFeatureValuesStorage {
    // cpp: style_rule_font_feature_values.cc:39-53
    pub fn new(
        stylistic: FontFeatureAliases,
        styleset: FontFeatureAliases,
        character_variant: FontFeatureAliases,
        swash: FontFeatureAliases,
        ornaments: FontFeatureAliases,
        annotation: FontFeatureAliases,
    ) -> Self {
        Self {
            stylistic_: stylistic,
            styleset_: styleset,
            character_variant_: character_variant,
            swash_: swash,
            ornaments_: ornaments,
            annotation_: annotation,
        }
    }

    // cpp: style_rule_font_feature_values.cc:55-83
    pub fn ResolveStylistic(&self, alias: &AtomicString) -> Vec<u32> {
        Self::ResolveInternal(&self.stylistic_, alias)
    }
    pub fn ResolveStyleset(&self, alias: &AtomicString) -> Vec<u32> {
        Self::ResolveInternal(&self.styleset_, alias)
    }
    pub fn ResolveCharacterVariant(&self, alias: &AtomicString) -> Vec<u32> {
        Self::ResolveInternal(&self.character_variant_, alias)
    }
    pub fn ResolveSwash(&self, alias: &AtomicString) -> Vec<u32> {
        Self::ResolveInternal(&self.swash_, alias)
    }
    pub fn ResolveOrnaments(&self, alias: &AtomicString) -> Vec<u32> {
        Self::ResolveInternal(&self.ornaments_, alias)
    }
    pub fn ResolveAnnotation(&self, alias: &AtomicString) -> Vec<u32> {
        Self::ResolveInternal(&self.annotation_, alias)
    }

    // cpp: style_rule_font_feature_values.cc:85-98
    pub fn SetLayerOrder(&mut self, layer_order: u16) {
        for aliases in [
            &mut self.stylistic_,
            &mut self.styleset_,
            &mut self.character_variant_,
            &mut self.swash_,
            &mut self.ornaments_,
            &mut self.annotation_,
        ] {
            for value in aliases.values_mut() {
                value.layer_order = layer_order;
            }
        }
    }

    // cpp: style_rule_font_feature_values.cc:100-126
    pub fn FuseUpdate(&mut self, other: &Self, other_layer_order: u32) {
        fn merge(own: &mut FontFeatureAliases, other: &FontFeatureAliases, other_layer_order: u32) {
            for (key, value) in other {
                let mut incoming = value.clone();
                // Source assigns unsigned to uint16_t; preserve truncation.
                incoming.layer_order = other_layer_order as u16;
                match own.get_mut(key) {
                    None => {
                        own.insert(key.clone(), incoming);
                    }
                    Some(existing) if other_layer_order >= existing.layer_order as u32 => {
                        *existing = incoming;
                    }
                    Some(_) => {}
                }
            }
        }
        merge(&mut self.stylistic_, &other.stylistic_, other_layer_order);
        merge(&mut self.styleset_, &other.styleset_, other_layer_order);
        merge(
            &mut self.character_variant_,
            &other.character_variant_,
            other_layer_order,
        );
        merge(&mut self.swash_, &other.swash_, other_layer_order);
        merge(&mut self.ornaments_, &other.ornaments_, other_layer_order);
        merge(&mut self.annotation_, &other.annotation_, other_layer_order);
    }

    // cpp: style_rule_font_feature_values.cc:128-137
    fn ResolveInternal(aliases: &FontFeatureAliases, alias: &AtomicString) -> Vec<u32> {
        aliases
            .get(alias)
            .map(|value| value.indices.clone())
            .unwrap_or_default()
    }
}

// The quoting decision belongs to css_parsing_utils::FontFamilyNeedsQuoting;
// making it mandatory prevents this rule object from inventing a second CSS
// font-family grammar.
pub trait FontFamilySerializer {
    fn SerializeFontFamily(family: &AtomicString) -> String;
}

// cpp: style_rule_font_feature_values.h:94-146
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleRuleFontFeatureValues {
    families_: Vec<AtomicString>,
    feature_values_storage_: FontFeatureValuesStorage,
}
impl StyleRuleFontFeatureValues {
    // cpp: style_rule_font_feature_values.cc:139-156
    pub fn new(
        families: Vec<AtomicString>,
        stylistic: FontFeatureAliases,
        styleset: FontFeatureAliases,
        character_variant: FontFeatureAliases,
        swash: FontFeatureAliases,
        ornaments: FontFeatureAliases,
        annotation: FontFeatureAliases,
    ) -> Self {
        Self {
            families_: families,
            feature_values_storage_: FontFeatureValuesStorage::new(
                stylistic,
                styleset,
                character_variant,
                swash,
                ornaments,
                annotation,
            ),
        }
    }
    pub fn GetFamilies(&self) -> &[AtomicString] {
        &self.families_
    }
    // cpp: style_rule_font_feature_values.cc:164-166
    pub fn SetFamilies(&mut self, families: Vec<AtomicString>) {
        self.families_ = families;
    }
    // cpp: style_rule_font_feature_values.cc:168-177
    pub fn FamilyAsString<S: FontFamilySerializer>(&self) -> String {
        let mut units = Vec::new();
        for (index, family) in self.families_.iter().enumerate() {
            let serialized = S::SerializeFontFamily(family);
            units.extend_from_slice(serialized.Span16().unwrap_or_default());
            if index + 1 < self.families_.len() {
                units.extend_from_slice(&[b',' as u16, b' ' as u16]);
            }
        }
        String::from_utf16(&units)
    }
    pub fn Storage(&self) -> &FontFeatureValuesStorage {
        &self.feature_values_storage_
    }
    pub fn GetStylistic(&mut self) -> &mut FontFeatureAliases {
        &mut self.feature_values_storage_.stylistic_
    }
    pub fn GetStyleset(&mut self) -> &mut FontFeatureAliases {
        &mut self.feature_values_storage_.styleset_
    }
    pub fn GetCharacterVariant(&mut self) -> &mut FontFeatureAliases {
        &mut self.feature_values_storage_.character_variant_
    }
    pub fn GetSwash(&mut self) -> &mut FontFeatureAliases {
        &mut self.feature_values_storage_.swash_
    }
    pub fn GetOrnaments(&mut self) -> &mut FontFeatureAliases {
        &mut self.feature_values_storage_.ornaments_
    }
    pub fn GetAnnotation(&mut self) -> &mut FontFeatureAliases {
        &mut self.feature_values_storage_.annotation_
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aliases(name: &str, indices: &[u32]) -> FontFeatureAliases {
        HashMap::from([(
            AtomicString::from_str(name),
            FeatureIndicesWithPriority::new(indices.to_vec()),
        )])
    }

    #[test]
    fn higher_or_equal_layer_wins_and_lower_layer_does_not() {
        let mut own = FontFeatureValuesStorage::new(
            aliases("wide", &[1]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        own.SetLayerOrder(7);
        let low = FontFeatureValuesStorage::new(
            aliases("wide", &[2]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        );
        own.FuseUpdate(&low, 6);
        assert_eq!(
            own.ResolveStylistic(&AtomicString::from_str("wide")),
            vec![1]
        );
        own.FuseUpdate(&low, 7);
        assert_eq!(
            own.ResolveStylistic(&AtomicString::from_str("wide")),
            vec![2]
        );
    }

    #[test]
    fn feature_rule_override_and_missing_alias_match_source() {
        let mut rule = StyleRuleFontFeature::new(FontFeatureType::kStyleset);
        rule.UpdateAlias(AtomicString::from_str("display"), vec![3, 4]);
        let mut destination = aliases("old", &[9]);
        rule.OverrideAliasesIn(&mut destination);
        assert_eq!(
            destination[&AtomicString::from_str("display")].indices,
            vec![3, 4]
        );
        assert_eq!(
            FontFeatureValuesStorage::default()
                .ResolveAnnotation(&AtomicString::from_str("missing")),
            Vec::<u32>::new()
        );
    }
}

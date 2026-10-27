// Copyright 1999-2026 The Chromium Authors and other contributors.
// Use of this source code is governed by the license in Chromium's
// third_party/blink/renderer/core/css/resolver/scoped_style_resolver.h.
//
// Extracted from ScopedStyleResolver, shared with the retained document Style
// owner. cpp: scoped_style_resolver.cc:357-389,409-436.

use crate::cascade_layer_map::CascadeLayerMap;
use crate::cascade_layered::CascadeLayered;
use foundation::{AtomicString, String};
use std::collections::{hash_map::Entry, HashMap};
use std::rc::Rc;
use style_model::style_rule_font_feature_values::{
    FontFeatureValuesStorage, StyleRuleFontFeatureValues,
};

/// Document-author font aliases collected from active native RuleSets.
/// The owner filters scope and media, supplies the canonical layer map, and
/// appends rules in stylesheet/source order. This registry performs no parsing
/// or font selection and retains no DOM, stylesheet, or live resolver owner.
#[derive(Clone, Debug, Default)]
pub struct FontFeatureValuesRegistry {
    storage_: HashMap<String, FontFeatureValuesStorage>,
    rules_: HashMap<String, Vec<Rc<StyleRuleFontFeatureValues>>>,
}

// Retained RuleSet allocations may change on reparsing without changing the
// effective aliases. Compare fused values and their layer priorities, rather
// than pointers or the list of overridden source rules, for invalidation.
impl PartialEq for FontFeatureValuesRegistry {
    fn eq(&self, other: &Self) -> bool {
        self.storage_ == other.storage_
    }
}
impl Eq for FontFeatureValuesRegistry {}

impl FontFeatureValuesRegistry {
    pub fn AddRules(
        &mut self,
        rules: &[CascadeLayered<Rc<StyleRuleFontFeatureValues>>],
        layers: Option<&CascadeLayerMap>,
    ) {
        for layered in rules {
            let order = layers
                .zip(layered.layer.as_ref())
                .map_or(CascadeLayerMap::kImplicitOuterLayerOrder, |(map, layer)| {
                    map.GetLayerOrder(layer)
                });
            for family in layered.value.GetFamilies() {
                let key = Self::FamilyKey(family);
                match self.storage_.entry(key.clone()) {
                    Entry::Vacant(entry) => {
                        let mut storage = layered.value.Storage().clone();
                        storage.SetLayerOrder(order);
                        entry.insert(storage);
                    }
                    Entry::Occupied(mut entry) => entry
                        .get_mut()
                        .FuseUpdate(layered.value.Storage(), u32::from(order)),
                }
                self.rules_
                    .entry(key)
                    .or_default()
                    .push(layered.value.clone());
            }
        }
    }

    pub fn Clear(&mut self) {
        self.storage_.clear();
        self.rules_.clear();
    }

    pub fn FontFeatureValuesForFamily(
        &self,
        family: &AtomicString,
    ) -> Option<&FontFeatureValuesStorage> {
        if self.storage_.is_empty() || family.empty() {
            return None;
        }
        self.storage_.get(&Self::FamilyKey(family))
    }

    pub fn FontFeatureValuesRulesForFamily(
        &self,
        family: &AtomicString,
    ) -> Option<&Vec<Rc<StyleRuleFontFeatureValues>>> {
        if self.rules_.is_empty() || family.empty() {
            return None;
        }
        self.rules_.get(&Self::FamilyKey(family))
    }

    fn FamilyKey(family: &AtomicString) -> String {
        String::from_utf16(family.utf16_units().unwrap_or_default()).FoldCase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_style_sheets::ActiveStyleSheet;
    use crate::cascade_layer::CascadeLayer;
    use crate::cascade_layer_map::CascadeLayerRuleSet;
    use style_model::style_rule_font_feature_values::FeatureIndicesWithPriority;

    fn rule(families: &[&str], aliases: &[(&str, &[u32])]) -> Rc<StyleRuleFontFeatureValues> {
        Rc::new(StyleRuleFontFeatureValues::new(
            families
                .iter()
                .map(|family| AtomicString::from_str(family))
                .collect(),
            HashMap::new(),
            aliases
                .iter()
                .map(|(name, values)| {
                    (
                        AtomicString::from_str(name),
                        FeatureIndicesWithPriority {
                            indices: values.to_vec(),
                            layer_order: CascadeLayerMap::kImplicitOuterLayerOrder,
                        },
                    )
                })
                .collect(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        ))
    }

    fn styleset(registry: &FontFeatureValuesRegistry, family: &str, alias: &str) -> Vec<u32> {
        registry
            .FontFeatureValuesForFamily(&AtomicString::from_str(family))
            .expect("known family")
            .ResolveStyleset(&AtomicString::from_str(alias))
    }

    #[test]
    fn source_order_merges_individual_aliases_and_retains_rules_for_all_families() {
        let first = rule(&["Example", "Other"], &[("first", &[1]), ("shared", &[2])]);
        let last = rule(&["Example"], &[("shared", &[3, 4]), ("last", &[5])]);
        let mut registry = FontFeatureValuesRegistry::default();
        registry.AddRules(&[CascadeLayered::new(first.clone(), None)], None);
        registry.AddRules(&[CascadeLayered::new(last.clone(), None)], None);
        assert_eq!(styleset(&registry, "Example", "first"), [1]);
        assert_eq!(styleset(&registry, "Example", "shared"), [3, 4]);
        assert_eq!(styleset(&registry, "Example", "last"), [5]);
        assert_eq!(styleset(&registry, "Other", "shared"), [2]);
        let retained = registry
            .FontFeatureValuesRulesForFamily(&AtomicString::from_str("EXAMPLE"))
            .unwrap();
        assert_eq!(retained.len(), 2);
        assert!(Rc::ptr_eq(&retained[0], &first));
        assert!(Rc::ptr_eq(&retained[1], &last));
        assert!(registry
            .FontFeatureValuesForFamily(&AtomicString::from_str("unknown"))
            .is_none());
        assert!(registry
            .FontFeatureValuesForFamily(&AtomicString::default())
            .is_none());
    }

    #[test]
    fn unicode_families_fold_fully_but_aliases_are_case_sensitive() {
        let mut registry = FontFeatureValuesRegistry::default();
        registry.AddRules(
            &[
                CascadeLayered::new(
                    rule(&["Straße", "Σςσ", "İI", "𐐀"], &[("Readable", &[2])]),
                    None,
                ),
                CascadeLayered::new(rule(&["STRASSE"], &[("readable", &[3])]), None),
            ],
            None,
        );
        for family in ["strasse", "ΣΣΣ", "i\u{307}i", "𐐨"] {
            assert_eq!(styleset(&registry, family, "Readable"), [2]);
            assert!(styleset(&registry, family, "READABLE").is_empty());
        }
        assert_eq!(styleset(&registry, "straße", "readable"), [3]);
        assert!(registry
            .FontFeatureValuesForFamily(&AtomicString::from_str("ii"))
            .is_none());
    }

    struct Rules(CascadeLayer);
    impl CascadeLayerRuleSet for Rules {
        fn CascadeLayers(&self) -> Option<&CascadeLayer> {
            Some(&self.0)
        }
    }

    #[test]
    fn canonical_cross_sheet_layers_win_before_source_order_and_outer_wins_all_layers() {
        let root_a = CascadeLayer::default();
        let low_a = root_a.GetOrAddSubLayer(&[AtomicString::from_str("low")]);
        let high_a = root_a.GetOrAddSubLayer(&[AtomicString::from_str("high")]);
        let root_b = CascadeLayer::default();
        let low_b = root_b.GetOrAddSubLayer(&[AtomicString::from_str("low")]);
        let high_b = root_b.GetOrAddSubLayer(&[AtomicString::from_str("high")]);
        let sheets = [
            ActiveStyleSheet::new(Rc::new(()), Some(Rc::new(Rules(root_a)))),
            ActiveStyleSheet::new(Rc::new(()), Some(Rc::new(Rules(root_b)))),
        ];
        let layers = CascadeLayerMap::new(&sheets);
        let mut registry = FontFeatureValuesRegistry::default();
        registry.AddRules(
            &[
                CascadeLayered::new(rule(&["Example"], &[("low", &[1])]), Some(low_a)),
                CascadeLayered::new(
                    rule(
                        &["Example"],
                        &[("shared", &[2]), ("same", &[2]), ("outer", &[2])],
                    ),
                    Some(high_a),
                ),
                CascadeLayered::new(rule(&["Example"], &[("outer", &[9])]), None),
            ],
            Some(&layers),
        );
        registry.AddRules(
            &[
                CascadeLayered::new(
                    rule(
                        &["Example"],
                        &[("shared", &[3]), ("low", &[3]), ("new", &[3])],
                    ),
                    Some(low_b),
                ),
                CascadeLayered::new(
                    rule(&["Example"], &[("same", &[4]), ("outer", &[4])]),
                    Some(high_b),
                ),
            ],
            Some(&layers),
        );
        assert_eq!(styleset(&registry, "Example", "shared"), [2]);
        assert_eq!(styleset(&registry, "Example", "low"), [3]);
        assert_eq!(styleset(&registry, "Example", "new"), [3]);
        assert_eq!(styleset(&registry, "Example", "same"), [4]);
        assert_eq!(styleset(&registry, "Example", "outer"), [9]);
    }

    #[test]
    fn semantic_equality_ignores_allocations_and_overridden_rules_and_clear_keeps_snapshots() {
        let mut registry = FontFeatureValuesRegistry::default();
        registry.AddRules(
            &[
                CascadeLayered::new(rule(&["Example"], &[("alias", &[1])]), None),
                CascadeLayered::new(rule(&["Example"], &[("alias", &[2])]), None),
            ],
            None,
        );
        let mut rebuilt = FontFeatureValuesRegistry::default();
        rebuilt.AddRules(
            &[CascadeLayered::new(
                rule(&["EXAMPLE"], &[("alias", &[2])]),
                None,
            )],
            None,
        );
        assert_eq!(registry, rebuilt);
        let previous = registry.clone();
        registry.AddRules(
            &[CascadeLayered::new(
                rule(&["Example"], &[("alias", &[3])]),
                None,
            )],
            None,
        );
        assert_ne!(registry, previous);
        assert_eq!(styleset(&previous, "Example", "alias"), [2]);
        registry.Clear();
        assert_eq!(registry, FontFeatureValuesRegistry::default());
        assert!(registry
            .FontFeatureValuesRulesForFamily(&AtomicString::from_str("Example"))
            .is_none());
        assert_eq!(styleset(&previous, "Example", "alias"), [2]);
    }
}

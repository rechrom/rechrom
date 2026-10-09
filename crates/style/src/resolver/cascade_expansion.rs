// Copyright 2020 The Chromium Authors. All rights reserved.
// BSD-style license; see the LICENSE file.
// cpp: third_party/blink/renderer/core/css/resolver/cascade_expansion.h
// cpp: third_party/blink/renderer/core/css/resolver/cascade_expansion.cc
// cpp: third_party/blink/renderer/core/css/resolver/cascade_expansion-inl.h
// Source ledger (physical/effective/mapped/omitted/pending):
//   cascade_expansion.h:      88 / 27 / 13 / 14 / 0
//   cascade_expansion.cc:     75 / 58 / 51 /  7 / 0
//   cascade_expansion-inl.h:  84 / 64 / 55 /  9 / 0
//   total:                  247 /149 /119 / 30 / 0
// Effective removes comments/blank lines but retains source delimiters,
// declarations and preprocessing; mapped + omitted + pending = effective.
// Production h:22-30,79-84, cc:14-52,56-73 and inl:18-80 are fully mapped.
// Omitted h:5-6,8-15,17,19,86,88; cc:5,7-8,10,12,54,75;
// inl:5-6,10,12-14,16,82,84: preprocessing/includes, namespace and forward
// declaration boilerplate only. No production source remains pending.
// Expansion preserves original logical/surrogate property IDs; conversion
// to physical properties remains in StyleCascade::ResolveSurrogate, as in C++.
// Original CSSPropertyValue storage and required property metadata adapters
// do not add alternate declaration/key models or inflate source counts.
#![allow(non_snake_case, non_upper_case_globals)]
use super::cascade_filter::CascadeFilter;
use super::cascade_priority::CascadePriority;
use super::match_result::{MatchedProperties, MatchedPropertySet};
use crate::css_property_value::{CSSPropertyValue, CSSPropertyValueBackend};
use crate::properties::css_property::CSSProperty;
use crate::valid_property_filter::ValidPropertyFilter;
use foundation::{
    kIntFirstCSSProperty, kIntLastCSSProperty, AtomicString, CSSPropertyID, ConvertToCSSPropertyID,
};

// The snapshot contains existing CSSPropertyValue objects from the erased
// CSSPropertyValueSet/RuleHandle, retaining its actual CSSValue ownership.
// Only original property storage and generated/CustomProperty metadata cross
// this boundary; filtering, priorities and all expansion decisions stay Rust.
pub trait CascadeExpansionBackend {
    type Dispatch: CSSPropertyValueBackend;
    type Document;
    fn Properties(&self, set: &dyn MatchedPropertySet) -> Vec<CSSPropertyValue<Self::Dispatch>>;
    fn CustomPropertyMetadata(&self, name: &AtomicString, document: &Self::Document)
        -> CSSProperty;
    fn IsAffectedByAll(&self, property: &CSSProperty) -> bool;
}
// cpp: cascade_expansion.h:26-29.
pub const kMaxDeclarationIndex: usize = u16::MAX as usize;
pub const kMaxMatchedPropertiesIndex: usize = u16::MAX as usize;
// cpp: cascade_expansion.cc:14-37.
fn AddValidPropertiesFilter(filter: CascadeFilter, matched: &MatchedProperties) -> CascadeFilter {
    let flag = match matched.data_.valid_property_filter() {
        x if x == ValidPropertyFilter::kNoFilter as u8 => return filter,
        x if x == ValidPropertyFilter::kCue as u8 => CSSProperty::kValidForCue,
        x if x == ValidPropertyFilter::kFirstLetter as u8 => CSSProperty::kValidForFirstLetter,
        x if x == ValidPropertyFilter::kFirstLine as u8 => CSSProperty::kValidForFirstLine,
        x if x == ValidPropertyFilter::kMarker as u8 => CSSProperty::kValidForMarker,
        x if x == ValidPropertyFilter::kHighlightLegacy as u8 => {
            CSSProperty::kValidForHighlightLegacy
        }
        x if x == ValidPropertyFilter::kHighlight as u8 => CSSProperty::kValidForHighlight,
        x if x == ValidPropertyFilter::kPositionTry as u8 => CSSProperty::kValidForPositionTry,
        x if x == ValidPropertyFilter::kPageContext as u8 => CSSProperty::kValidForPageContext,
        _ => unreachable!("source valid-property filter must be enum member"),
    };
    filter.Add(flag)
}
// cpp: cascade_expansion.cc:39-51. CSSSelector source bit values are
// kMatchLink=1,kMatchVisited=2,kMatchAll=3, as existing MatchResult stores.
fn AddLinkFilter(filter: CascadeFilter, matched: &MatchedProperties) -> CascadeFilter {
    match matched.data_.link_match_type() {
        2 => filter.Add(CSSProperty::kVisited),
        1 => filter.Add(CSSProperty::kNotVisited),
        3 => filter,
        _ => filter
            .Add(CSSProperty::kVisited)
            .Add(CSSProperty::kNotVisited),
    }
}
// cpp: cascade_expansion.cc:55-61.
pub fn CreateExpansionFilter(matched: &MatchedProperties) -> CascadeFilter {
    AddLinkFilter(
        AddValidPropertiesFilter(CascadeFilter::new(), matched),
        matched,
    )
}
// cpp: cascade_expansion.cc:63-73.
pub fn IsInAllExpansion<B: CascadeExpansionBackend>(id: CSSPropertyID, backend: &B) -> bool {
    let property = CSSProperty::Get(id);
    !property.IsShorthand()
        && (backend.IsAffectedByAll(property)
            || property
                .GetUnvisitedProperty()
                .is_some_and(|unvisited| backend.IsAffectedByAll(unvisited)))
}
// cpp: cascade_expansion.h:79-84; cascade_expansion-inl.h:18-80.
pub fn ExpandCascade<B: CascadeExpansionBackend>(
    matched: &MatchedProperties,
    document: &B::Document,
    index: usize,
    backend: &B,
    mut custom: impl FnMut(CascadePriority, &AtomicString),
    mut regular: impl FnMut(CascadePriority, CSSPropertyID),
) {
    let properties = backend.Properties(matched.properties.as_ref());
    if properties.len() > kMaxDeclarationIndex + 1 || index > kMaxMatchedPropertiesIndex {
        return;
    }
    let filter = CreateExpansionFilter(matched);
    let expand_visited = !filter.Requires(CSSProperty::kNotVisited);
    for (declaration, reference) in properties.iter().enumerate() {
        let id = reference.PropertyID();
        let data = &matched.data_;
        let priority = CascadePriority::FromParts(
            data.origin,
            reference.IsImportant(),
            data.tree_order,
            data.is_inline_style(),
            data.is_try_style(),
            data.is_try_tactics_style,
            data.layer_order,
            index as u16,
            declaration as u16,
        );
        if id == CSSPropertyID::kVariable {
            let property = backend.CustomPropertyMetadata(reference.CustomPropertyName(), document);
            if filter.Accepts(&property) {
                custom(priority, reference.CustomPropertyName());
            }
        } else if id == CSSPropertyID::kAll {
            for i in kIntFirstCSSProperty..=kIntLastCSSProperty {
                let expanded = ConvertToCSSPropertyID(i);
                if !IsInAllExpansion(expanded, backend) {
                    continue;
                }
                if filter.Accepts(CSSProperty::Get(expanded)) {
                    regular(priority, expanded);
                }
            }
        } else {
            let property = CSSProperty::Get(id);
            if filter.Accepts(property) {
                regular(priority, id);
            }
            if expand_visited {
                if let Some(visited) = property.GetVisitedProperty() {
                    if filter.Accepts(visited) {
                        regular(priority, visited.PropertyID());
                    }
                }
            }
        }
    }
}

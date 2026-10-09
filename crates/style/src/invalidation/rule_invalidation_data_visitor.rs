// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/invalidation/rule_invalidation_data_visitor.h:17,72-77,94-192,207-225,251-268,276-290,294-309,313-329,347-351
// cpp: third_party/blink/renderer/core/css/invalidation/rule_invalidation_data_visitor.cc:287-341
// Complete local feature types and restore guards. The containing visitor's
// selector traversal requires CSSSelector, StyleScope and InvalidationSet and
// is not declared here. Rust Vec stores the source Vector<AtomicString, 4>.

use super::invalidation_flags::InvalidationFlags;
use foundation::AtomicString;
use std::ops::{Deref, DerefMut};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum RuleInvalidationDataVisitorType {
    kBuilder,
    kTracer,
}

// cpp: rule_invalidation_data_visitor.h:72-77
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeatureMetadata {
    pub uses_first_line_rules: bool,
    pub uses_window_inactive_selector: bool,
    pub max_direct_adjacent_selectors: u32,
}

// cpp: rule_invalidation_data_visitor.h:94-192
#[derive(Clone, Default)]
pub struct InvalidationSetFeatures {
    pub classes: Vec<AtomicString>,
    pub attributes: Vec<AtomicString>,
    pub ids: Vec<AtomicString>,
    pub tag_names: Vec<AtomicString>,
    pub emitted_tag_names: Vec<AtomicString>,
    pub custom_pseudo_names: Vec<AtomicString>,
    pub max_direct_adjacent_selectors: u32,
    // Number of previously evaluated child/descendant combinators. This tracks
    // whether a compound selector within a logical combination in :has() is
    // in subject position; it is not the maximum adjacent sibling count.
    pub descendant_features_depth: u32,
    pub invalidation_flags: InvalidationFlags,
    pub content_pseudo_crossing: bool,
    pub has_nth_pseudo: bool,
    pub has_features_for_rule_set_invalidation: bool,
}

#[allow(non_snake_case)]
impl InvalidationSetFeatures {
    // cpp: rule_invalidation_data_visitor.cc:287-316
    pub fn Merge(&mut self, other: &Self) {
        self.classes.extend(other.classes.iter().cloned());
        self.attributes.extend(other.attributes.iter().cloned());
        self.custom_pseudo_names
            .extend(other.custom_pseudo_names.iter().cloned());
        self.ids.extend(other.ids.iter().cloned());
        // Tags already emitted into an ID/class/attribute invalidation set must
        // remain separate from tags destined for the type rule invalidation set.
        if other.has_features_for_rule_set_invalidation {
            self.emitted_tag_names
                .extend(other.tag_names.iter().cloned());
        } else {
            self.tag_names.extend(other.tag_names.iter().cloned());
        }
        self.emitted_tag_names
            .extend(other.emitted_tag_names.iter().cloned());
        self.max_direct_adjacent_selectors = self
            .max_direct_adjacent_selectors
            .max(other.max_direct_adjacent_selectors);
        self.invalidation_flags.Merge(&other.invalidation_flags);
        self.content_pseudo_crossing |= other.content_pseudo_crossing;
        self.has_nth_pseudo |= other.has_nth_pseudo;
    }

    // cpp: rule_invalidation_data_visitor.cc:318-324
    pub fn HasFeatures(&self) -> bool {
        !self.classes.is_empty()
            || !self.attributes.is_empty()
            || !self.ids.is_empty()
            || !self.tag_names.is_empty()
            || !self.emitted_tag_names.is_empty()
            || !self.custom_pseudo_names.is_empty()
            || self.invalidation_flags.InvalidatesParts()
    }

    // cpp: rule_invalidation_data_visitor.cc:326-330
    pub fn HasIdClassOrAttribute(&self) -> bool {
        !self.classes.is_empty() || !self.attributes.is_empty() || !self.ids.is_empty()
    }

    // cpp: rule_invalidation_data_visitor.h:102-108
    pub fn NarrowToClass(&mut self, class_name: &AtomicString) {
        if self.Size() == 1 && (!self.ids.is_empty() || !self.classes.is_empty()) {
            return;
        }
        self.ClearFeatures();
        self.classes.push(class_name.clone());
    }

    // cpp: rule_invalidation_data_visitor.h:109-116
    pub fn NarrowToAttribute(&mut self, attribute: &AtomicString) {
        if self.Size() == 1
            && (!self.ids.is_empty() || !self.classes.is_empty() || !self.attributes.is_empty())
        {
            return;
        }
        self.ClearFeatures();
        self.attributes.push(attribute.clone());
    }

    // cpp: rule_invalidation_data_visitor.h:117-123
    pub fn NarrowToId(&mut self, id: &AtomicString) {
        if self.Size() == 1 && !self.ids.is_empty() {
            return;
        }
        self.ClearFeatures();
        self.ids.push(id.clone());
    }

    // cpp: rule_invalidation_data_visitor.h:124-130
    pub fn NarrowToTag(&mut self, tag_name: &AtomicString) {
        if self.Size() == 1 {
            return;
        }
        self.ClearFeatures();
        self.tag_names.push(tag_name.clone());
    }

    // cpp: rule_invalidation_data_visitor.h:131-138
    pub fn NarrowToCustomPseudo(&mut self, custom_pseudo_name: &AtomicString) {
        // The source checks the incoming name here, not custom_pseudo_names.
        if self.Size() == 1
            && (!self.ids.is_empty()
                || !self.classes.is_empty()
                || !self.attributes.is_empty()
                || !custom_pseudo_name.empty())
        {
            return;
        }
        self.ClearFeatures();
        self.custom_pseudo_names.push(custom_pseudo_name.clone());
    }

    // cpp: rule_invalidation_data_visitor.cc:332-341
    pub fn NarrowToFeatures(&mut self, other: &Self) {
        let size = self.Size();
        let other_size = other.Size();
        if size == 0 || (1 <= other_size && other_size < size) {
            self.ClearFeatures();
            self.Merge(other);
        }
    }

    // cpp: rule_invalidation_data_visitor.h:140-147
    pub fn ClearFeatures(&mut self) {
        self.classes.clear();
        self.attributes.clear();
        self.ids.clear();
        self.tag_names.clear();
        self.emitted_tag_names.clear();
        self.custom_pseudo_names.clear();
    }

    // cpp: rule_invalidation_data_visitor.h:148-152
    pub fn Size(&self) -> u32 {
        (self.classes.len() as u32)
            .wrapping_add(self.attributes.len() as u32)
            .wrapping_add(self.ids.len() as u32)
            .wrapping_add(self.tag_names.len() as u32)
            .wrapping_add(self.emitted_tag_names.len() as u32)
            .wrapping_add(self.custom_pseudo_names.len() as u32)
    }
}

// cpp: rule_invalidation_data_visitor.h:207-225
// An exclusive borrow replaces the source's pointer alias. Features() allows
// updates through that borrow while the restoration guard is alive.
pub struct AutoRestoreMaxDirectAdjacentSelectors<'a> {
    features_: Option<&'a mut InvalidationSetFeatures>,
    original_value_: u32,
}
#[allow(non_snake_case)]
impl<'a> AutoRestoreMaxDirectAdjacentSelectors<'a> {
    pub fn new(features: Option<&'a mut InvalidationSetFeatures>) -> Self {
        let original_value_ = features
            .as_ref()
            .map_or(0, |f| f.max_direct_adjacent_selectors);
        Self {
            features_: features,
            original_value_,
        }
    }
    pub fn Features(&mut self) -> Option<&mut InvalidationSetFeatures> {
        self.features_.as_deref_mut()
    }
}
impl Drop for AutoRestoreMaxDirectAdjacentSelectors<'_> {
    fn drop(&mut self) {
        if let Some(features) = &mut self.features_ {
            features.max_direct_adjacent_selectors = self.original_value_;
        }
    }
}

// cpp: rule_invalidation_data_visitor.h:251-268
pub struct AutoRestoreDescendantFeaturesDepth<'a> {
    features_: Option<&'a mut InvalidationSetFeatures>,
    original_value_: u32,
}
#[allow(non_snake_case)]
impl<'a> AutoRestoreDescendantFeaturesDepth<'a> {
    pub fn new(features: Option<&'a mut InvalidationSetFeatures>) -> Self {
        let original_value_ = features.as_ref().map_or(0, |f| f.descendant_features_depth);
        Self {
            features_: features,
            original_value_,
        }
    }
    pub fn Features(&mut self) -> Option<&mut InvalidationSetFeatures> {
        self.features_.as_deref_mut()
    }
}
impl Drop for AutoRestoreDescendantFeaturesDepth<'_> {
    fn drop(&mut self) {
        if let Some(features) = &mut self.features_ {
            features.descendant_features_depth = self.original_value_;
        }
    }
}

// cpp: rule_invalidation_data_visitor.h:276-290
pub struct AutoRestoreWholeSubtreeInvalid<'a> {
    features_: &'a mut InvalidationSetFeatures,
    original_value_: bool,
}
impl<'a> AutoRestoreWholeSubtreeInvalid<'a> {
    pub fn new(features: &'a mut InvalidationSetFeatures) -> Self {
        let original_value_ = features.invalidation_flags.WholeSubtreeInvalid();
        Self {
            features_: features,
            original_value_,
        }
    }
}
impl Drop for AutoRestoreWholeSubtreeInvalid<'_> {
    fn drop(&mut self) {
        self.features_
            .invalidation_flags
            .SetWholeSubtreeInvalid(self.original_value_);
    }
}
impl Deref for AutoRestoreWholeSubtreeInvalid<'_> {
    type Target = InvalidationSetFeatures;
    fn deref(&self) -> &Self::Target {
        self.features_
    }
}
impl DerefMut for AutoRestoreWholeSubtreeInvalid<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.features_
    }
}

// cpp: rule_invalidation_data_visitor.h:294-309
pub struct AutoRestoreTreeBoundaryCrossingFlag<'a> {
    features_: &'a mut InvalidationSetFeatures,
    original_value_: bool,
}
impl<'a> AutoRestoreTreeBoundaryCrossingFlag<'a> {
    pub fn new(features: &'a mut InvalidationSetFeatures) -> Self {
        let original_value_ = features.invalidation_flags.TreeBoundaryCrossing();
        Self {
            features_: features,
            original_value_,
        }
    }
}
impl Drop for AutoRestoreTreeBoundaryCrossingFlag<'_> {
    fn drop(&mut self) {
        self.features_
            .invalidation_flags
            .SetTreeBoundaryCrossing(self.original_value_);
    }
}
impl Deref for AutoRestoreTreeBoundaryCrossingFlag<'_> {
    type Target = InvalidationSetFeatures;
    fn deref(&self) -> &Self::Target {
        self.features_
    }
}
impl DerefMut for AutoRestoreTreeBoundaryCrossingFlag<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.features_
    }
}

// cpp: rule_invalidation_data_visitor.h:313-329
pub struct AutoRestoreInsertionPointCrossingFlag<'a> {
    features_: &'a mut InvalidationSetFeatures,
    original_value_: bool,
}
impl<'a> AutoRestoreInsertionPointCrossingFlag<'a> {
    pub fn new(features: &'a mut InvalidationSetFeatures) -> Self {
        let original_value_ = features.invalidation_flags.InsertionPointCrossing();
        Self {
            features_: features,
            original_value_,
        }
    }
}
impl Drop for AutoRestoreInsertionPointCrossingFlag<'_> {
    fn drop(&mut self) {
        self.features_
            .invalidation_flags
            .SetInsertionPointCrossing(self.original_value_);
    }
}
impl Deref for AutoRestoreInsertionPointCrossingFlag<'_> {
    type Target = InvalidationSetFeatures;
    fn deref(&self) -> &Self::Target {
        self.features_
    }
}
impl DerefMut for AutoRestoreInsertionPointCrossingFlag<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.features_
    }
}

// cpp: rule_invalidation_data_visitor.h:347-351
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PositionType {
    kSubject,
    kAncestor,
}
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FeatureInvalidationType {
    kNormalInvalidation,
    kRequiresSubtreeInvalidation,
}

#[cfg(test)]
#[path = "rule_invalidation_data_visitor_test.rs"]
mod tests;

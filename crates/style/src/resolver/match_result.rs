/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
 * Copyright (C) 2013 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

// cpp: third_party/blink/renderer/core/css/resolver/match_result.h:51-285
// cpp: third_party/blink/renderer/core/css/resolver/match_result.cc:49-103

use super::cascade_origin::CascadeOrigin;
use super::match_flags::{MatchFlag, MatchFlags};
use crate::valid_property_filter::ValidPropertyFilter;
use foundation::AtomicString;
use layoutng_style::style::computed_style_constants::{PseudoId, PseudoIdFlags};
use std::collections::HashSet;
use std::rc::Rc;

// These three interfaces are the exact information MatchResult consumes from
// the owning CSS/DOM objects. Keeping them here avoids making resolver depend
// on a browser Page or a particular DOM storage implementation.
pub trait MatchedPropertySet {
    fn AsAny(&self) -> &dyn std::any::Any;
    fn Equals(&self, other: &dyn MatchedPropertySet) -> bool;
    fn GetHash(&self) -> u32;
    fn ModifiedSinceHashing(&self) -> bool;
}

pub trait MixinParameterBindings {
    fn AsAny(&self) -> &dyn std::any::Any;
    fn Equals(&self, other: &dyn MixinParameterBindings) -> bool;
    fn GetHash(&self) -> u32;
}

pub trait TreeScope {}

// cpp: match_result.h:51-81
// The four C++ bit-fields share one byte. This representation preserves the
// eight-byte layout and explicitly initialized padding used by byte comparison
// and the matched-properties cache. Equality over all these fields is therefore
// equivalent to memcmp of the complete initialized representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedPropertiesData {
    pub tree_order: u16,
    packed_match_flags_: u8,
    pub origin: CascadeOrigin,
    pub layer_order: u16,
    pub is_try_tactics_style: bool,
    pub padding: u8,
}

impl Default for MatchedPropertiesData {
    fn default() -> Self {
        Self {
            tree_order: 0,
            // cpp: css_selector.h:705-708: kMatchAll = kMatchLink | kMatchVisited.
            // cpp: valid_property_filter.h:16: kNoFilter = 0.
            packed_match_flags_: (1 | 2) | ((ValidPropertyFilter::kNoFilter as u8) << 2),
            origin: CascadeOrigin::kNone,
            // cpp: cascade_layer_map.h:24-25: kImplicitOuterLayerOrder.
            layer_order: u16::MAX,
            is_try_tactics_style: false,
            padding: 0,
        }
    }
}

// Rust exposes C++ bit-field access through explicit getters and setters.
impl MatchedPropertiesData {
    // cpp: match_result.h:60
    pub const fn link_match_type(&self) -> u8 {
        self.packed_match_flags_ & 0b11
    }
    pub fn set_link_match_type(&mut self, value: u8) {
        self.packed_match_flags_ = (self.packed_match_flags_ & !0b11) | (value & 0b11);
    }

    // cpp: match_result.h:61-63
    pub const fn valid_property_filter(&self) -> u8 {
        (self.packed_match_flags_ >> 2) & 0b1111
    }
    pub fn set_valid_property_filter(&mut self, value: u8) {
        self.packed_match_flags_ =
            (self.packed_match_flags_ & !(0b1111 << 2)) | ((value & 0b1111) << 2);
    }

    // cpp: match_result.h:64
    pub const fn is_inline_style(&self) -> bool {
        self.packed_match_flags_ & (1 << 6) != 0
    }
    pub fn set_is_inline_style(&mut self, value: bool) {
        self.packed_match_flags_ = (self.packed_match_flags_ & !(1 << 6)) | ((value as u8) << 6);
    }

    // cpp: match_result.h:67
    pub const fn is_try_style(&self) -> bool {
        self.packed_match_flags_ & (1 << 7) != 0
    }
    pub fn set_is_try_style(&mut self, value: bool) {
        self.packed_match_flags_ = (self.packed_match_flags_ & !(1 << 7)) | ((value as u8) << 7);
    }
}

const _: () = assert!(std::mem::size_of::<MatchedPropertiesData>() == 8);

// cpp: match_result.h:105-114
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MatchedPropertiesHash {
    pub hash: u32,
    pub data: MatchedPropertiesData,
}

// cpp: match_result.h:49-102
#[derive(Clone)]
pub struct MatchedProperties {
    pub properties: Rc<dyn MatchedPropertySet>,
    pub mixin_parameter_bindings: Option<Rc<dyn MixinParameterBindings>>,
    pub data_: MatchedPropertiesData,
}

impl MatchedProperties {
    pub fn new(
        properties: Rc<dyn MatchedPropertySet>,
        mixin_parameter_bindings: Option<Rc<dyn MixinParameterBindings>>,
        data: MatchedPropertiesData,
    ) -> Self {
        Self {
            properties,
            mixin_parameter_bindings,
            data_: data,
        }
    }
}

// cpp: match_result.h:287-291. Mixin bindings and the remaining Data fields
// deliberately do not participate in this source equality operation.
impl PartialEq for MatchedProperties {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.properties, &other.properties)
            && self.data_.link_match_type() == other.data_.link_match_type()
    }
}

impl Eq for MatchedProperties {}

// cpp: match_result.h:124-285
pub struct MatchResult {
    matched_properties_: Vec<MatchedProperties>,
    matched_properties_hashes_: Vec<MatchedPropertiesHash>,
    tree_scopes_: Vec<Rc<dyn TreeScope>>,
    custom_highlight_names_: HashSet<AtomicString>,
    is_cacheable_: bool,
    depends_on_size_container_queries_: bool,
    depends_on_style_container_queries_: bool,
    depends_on_scroll_state_container_queries_: bool,
    depends_on_anchored_container_queries_: bool,
    first_line_depends_on_size_container_queries_: bool,
    depends_on_static_viewport_units_: bool,
    depends_on_dynamic_viewport_units_: bool,
    depends_on_root_unit_container_queries_: bool,
    conditionally_affects_animations_: bool,
    has_non_universal_highlight_pseudo_styles_: bool,
    has_non_ua_highlight_pseudo_styles_: bool,
    highlights_depend_on_size_container_queries_: bool,
    flags_: MatchFlags,
    current_tree_order_: u16,
    pseudo_element_styles_: PseudoIdFlags,
    #[cfg(debug_assertions)]
    last_origin_: CascadeOrigin,
}

impl Default for MatchResult {
    fn default() -> Self {
        Self {
            matched_properties_: Vec::with_capacity(64),
            matched_properties_hashes_: Vec::with_capacity(64),
            tree_scopes_: Vec::with_capacity(4),
            custom_highlight_names_: HashSet::new(),
            is_cacheable_: true,
            depends_on_size_container_queries_: false,
            depends_on_style_container_queries_: false,
            depends_on_scroll_state_container_queries_: false,
            depends_on_anchored_container_queries_: false,
            first_line_depends_on_size_container_queries_: false,
            depends_on_static_viewport_units_: false,
            depends_on_dynamic_viewport_units_: false,
            depends_on_root_unit_container_queries_: false,
            conditionally_affects_animations_: false,
            has_non_universal_highlight_pseudo_styles_: false,
            has_non_ua_highlight_pseudo_styles_: false,
            highlights_depend_on_size_container_queries_: false,
            flags_: 0,
            current_tree_order_: 0,
            pseudo_element_styles_: PseudoIdFlags::default(),
            #[cfg(debug_assertions)]
            last_origin_: CascadeOrigin::kNone,
        }
    }
}

impl MatchResult {
    // cpp: match_result.cc:49-82
    pub fn AddMatchedProperties(
        &mut self,
        properties: Rc<dyn MatchedPropertySet>,
        mixin_parameter_bindings: Option<Rc<dyn MixinParameterBindings>>,
        mut data: MatchedPropertiesData,
    ) {
        let mixin_hash = mixin_parameter_bindings
            .as_ref()
            .map_or(0, |bindings| bindings.GetHash());
        data.tree_order = self.current_tree_order_;
        self.matched_properties_hashes_.push(MatchedPropertiesHash {
            hash: properties.GetHash() ^ mixin_hash,
            data,
        });
        if properties.ModifiedSinceHashing() {
            self.is_cacheable_ = false;
        }
        #[cfg(debug_assertions)]
        {
            debug_assert_ne!(data.origin, CascadeOrigin::kNone);
            debug_assert!((data.origin as u8) >= (self.last_origin_ as u8));
            if !self.tree_scopes_.is_empty() {
                debug_assert_eq!(data.origin, CascadeOrigin::kAuthor);
            }
            self.last_origin_ = data.origin;
        }
        self.matched_properties_.push(MatchedProperties::new(
            properties,
            mixin_parameter_bindings,
            data,
        ));
    }

    pub fn HasMatchedProperties(&self) -> bool {
        !self.matched_properties_.is_empty()
    }

    // cpp: match_result.cc:84-89
    pub fn BeginAddingAuthorRulesForTreeScope(&mut self, tree_scope: Rc<dyn TreeScope>) {
        self.current_tree_order_ = self.tree_scopes_.len().min(u16::MAX as usize) as u16;
        self.tree_scopes_.push(tree_scope);
    }

    pub fn AddCustomHighlightName(&mut self, name: AtomicString) {
        self.custom_highlight_names_.insert(name);
    }
    pub fn CustomHighlightNames(&self) -> &HashSet<AtomicString> {
        &self.custom_highlight_names_
    }
    pub fn SetIsCacheable(&mut self, cacheable: bool) {
        self.is_cacheable_ = cacheable;
    }
    pub fn IsCacheable(&self) -> bool {
        self.is_cacheable_
    }

    pub fn SetDependsOnSizeContainerQueries(&mut self) {
        self.depends_on_size_container_queries_ = true;
    }
    pub fn DependsOnSizeContainerQueries(&self) -> bool {
        self.depends_on_size_container_queries_
    }
    pub fn SetDependsOnStyleContainerQueries(&mut self) {
        self.depends_on_style_container_queries_ = true;
    }
    pub fn DependsOnStyleContainerQueries(&self) -> bool {
        self.depends_on_style_container_queries_
    }
    pub fn SetDependsOnScrollStateContainerQueries(&mut self) {
        self.depends_on_scroll_state_container_queries_ = true;
    }
    pub fn DependsOnScrollStateContainerQueries(&self) -> bool {
        self.depends_on_scroll_state_container_queries_
    }
    pub fn SetDependsOnAnchoredContainerQueries(&mut self) {
        self.depends_on_anchored_container_queries_ = true;
    }
    pub fn DependsOnAnchoredContainerQueries(&self) -> bool {
        self.depends_on_anchored_container_queries_
    }
    pub fn SetFirstLineDependsOnSizeContainerQueries(&mut self) {
        self.first_line_depends_on_size_container_queries_ = true;
    }
    pub fn FirstLineDependsOnSizeContainerQueries(&self) -> bool {
        self.first_line_depends_on_size_container_queries_
    }
    pub fn SetDependsOnStaticViewportUnits(&mut self) {
        self.depends_on_static_viewport_units_ = true;
    }
    pub fn DependsOnStaticViewportUnits(&self) -> bool {
        self.depends_on_static_viewport_units_
    }
    pub fn SetDependsOnDynamicViewportUnits(&mut self) {
        self.depends_on_dynamic_viewport_units_ = true;
    }
    pub fn DependsOnDynamicViewportUnits(&self) -> bool {
        self.depends_on_dynamic_viewport_units_
    }
    pub fn SetDependsOnRootUnitContainerQueries(&mut self) {
        self.depends_on_root_unit_container_queries_ = true;
    }
    pub fn DependsOnRootUnitContainerQueries(&self) -> bool {
        self.depends_on_root_unit_container_queries_
    }
    pub fn SetConditionallyAffectsAnimations(&mut self) {
        self.conditionally_affects_animations_ = true;
    }
    pub fn ConditionallyAffectsAnimations(&self) -> bool {
        self.conditionally_affects_animations_
    }
    pub fn SetHasNonUniversalHighlightPseudoStyles(&mut self) {
        self.has_non_universal_highlight_pseudo_styles_ = true;
    }
    pub fn HasNonUniversalHighlightPseudoStyles(&self) -> bool {
        self.has_non_universal_highlight_pseudo_styles_
    }
    pub fn SetHasNonUaHighlightPseudoStyles(&mut self) {
        self.has_non_ua_highlight_pseudo_styles_ = true;
    }
    pub fn HasNonUaHighlightPseudoStyles(&self) -> bool {
        self.has_non_ua_highlight_pseudo_styles_
    }
    pub fn SetHighlightsDependOnSizeContainerQueries(&mut self) {
        self.highlights_depend_on_size_container_queries_ = true;
    }
    pub fn HighlightsDependOnSizeContainerQueries(&self) -> bool {
        self.highlights_depend_on_size_container_queries_
    }

    pub fn HasFlag(&self, flag: MatchFlag) -> bool {
        self.flags_ & flag as MatchFlags != 0
    }
    pub fn AddFlags(&mut self, flags: MatchFlags) {
        self.flags_ |= flags;
    }

    pub fn SetHasPseudoElementStyle(&mut self, pseudo: PseudoId) {
        debug_assert!(pseudo >= PseudoId::kFirstPublicPseudoId);
        debug_assert!(pseudo <= PseudoId::kLastTrackedPublicPseudoId);
        self.pseudo_element_styles_.Set(pseudo);
    }
    pub fn PseudoElementStyles(&self) -> PseudoIdFlags {
        self.pseudo_element_styles_
    }
    pub fn GetMatchedProperties(&self) -> &[MatchedProperties] {
        &self.matched_properties_
    }
    pub fn GetMatchedPropertiesHash(&self) -> &[MatchedPropertiesHash] {
        &self.matched_properties_hashes_
    }

    pub fn CurrentTreeScope(&self) -> Option<&dyn TreeScope> {
        self.tree_scopes_.last().map(|scope| scope.as_ref())
    }
    // Owning handle counterpart for callers that recursively mutate cascade
    // state while retaining the exact scope identity from the match result.
    pub fn ScopeFromTreeOrderHandle(&self, tree_order: u16) -> Rc<dyn TreeScope> {
        self.tree_scopes_[tree_order as usize].clone()
    }
    pub fn ScopeFromTreeOrder(&self, tree_order: u16) -> &dyn TreeScope {
        self.tree_scopes_[tree_order as usize].as_ref()
    }

    // cpp: match_result.cc:91-103. The source intentionally resets only this
    // subset; other dependency/highlight flags describe the entire match pass.
    pub fn Reset(&mut self) {
        self.matched_properties_.clear();
        self.matched_properties_hashes_.clear();
        self.is_cacheable_ = true;
        self.depends_on_size_container_queries_ = false;
        #[cfg(debug_assertions)]
        {
            self.last_origin_ = CascadeOrigin::kNone;
        }
        self.current_tree_order_ = 0;
        self.tree_scopes_.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PropertySet {
        hash: u32,
        modified: bool,
    }
    impl MatchedPropertySet for PropertySet {
        fn AsAny(&self) -> &dyn std::any::Any {
            self
        }
        fn Equals(&self, other: &dyn MatchedPropertySet) -> bool {
            other
                .AsAny()
                .downcast_ref::<Self>()
                .is_some_and(|other| self.hash == other.hash && self.modified == other.modified)
        }
        fn GetHash(&self) -> u32 {
            self.hash
        }
        fn ModifiedSinceHashing(&self) -> bool {
            self.modified
        }
    }
    struct Bindings(u32);
    impl MixinParameterBindings for Bindings {
        fn AsAny(&self) -> &dyn std::any::Any {
            self
        }
        fn Equals(&self, other: &dyn MixinParameterBindings) -> bool {
            other
                .AsAny()
                .downcast_ref::<Self>()
                .is_some_and(|other| self.0 == other.0)
        }
        fn GetHash(&self) -> u32 {
            self.0
        }
    }
    struct Scope;
    impl TreeScope for Scope {}

    // Verify the representation required by match_result.h:74-79: all four
    // bit fields are independent, assignment truncates to the C++ field width,
    // and equality includes the explicitly stored padding byte.
    #[test]
    fn PackedDataPreservesAllBitsAndEquality() {
        let mut data = MatchedPropertiesData::default();
        assert_eq!(std::mem::size_of_val(&data), 8);
        assert_eq!(data.link_match_type(), 3);
        assert_eq!(data.valid_property_filter(), 0);
        assert!(!data.is_inline_style());
        assert!(!data.is_try_style());
        assert_eq!(data.layer_order, u16::MAX);
        data.set_is_inline_style(true);
        data.set_is_try_style(true);
        data.set_valid_property_filter(0xFF);
        data.set_link_match_type(0xFE);
        assert_eq!(data.link_match_type(), 2);
        assert_eq!(data.valid_property_filter(), 15);
        assert!(data.is_inline_style());
        assert!(data.is_try_style());
        let mut equal = data;
        assert_eq!(equal, data);
        equal.padding = 1;
        assert_ne!(equal, data);
        data.set_is_inline_style(false);
        assert!(!data.is_inline_style());
        assert!(data.is_try_style());
        assert_eq!(data.valid_property_filter(), 15);
        assert_eq!(data.link_match_type(), 2);
    }

    #[test]
    fn match_result_preserves_tree_order_hash_and_source_reset_subset() {
        let mut result = MatchResult::default();
        result.BeginAddingAuthorRulesForTreeScope(Rc::new(Scope));
        result.SetDependsOnStyleContainerQueries();
        result.SetDependsOnSizeContainerQueries();
        result.AddFlags(MatchFlag::kAffectedByHover as MatchFlags);
        result.AddCustomHighlightName(AtomicString::from_str("mark"));

        let properties: Rc<dyn MatchedPropertySet> = Rc::new(PropertySet {
            hash: 0x55,
            modified: false,
        });
        let bindings: Rc<dyn MixinParameterBindings> = Rc::new(Bindings(0x0f));
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kAuthor;
        result.AddMatchedProperties(properties, Some(bindings), data);

        assert!(result.HasMatchedProperties());
        assert_eq!(result.GetMatchedProperties()[0].data_.tree_order, 0);
        assert_eq!(result.GetMatchedPropertiesHash()[0].hash, 0x5a);
        assert!(result.IsCacheable());
        assert!(result.HasFlag(MatchFlag::kAffectedByHover));
        assert!(result.CurrentTreeScope().is_some());

        result.Reset();
        assert!(!result.HasMatchedProperties());
        assert!(result.IsCacheable());
        assert!(!result.DependsOnSizeContainerQueries());
        assert!(result.DependsOnStyleContainerQueries());
        assert!(result.HasFlag(MatchFlag::kAffectedByHover));
        assert!(result.CurrentTreeScope().is_none());
        assert!(result
            .CustomHighlightNames()
            .contains(&AtomicString::from_str("mark")));
    }

    #[test]
    fn modified_property_set_disables_matched_properties_cache() {
        let mut result = MatchResult::default();
        let properties: Rc<dyn MatchedPropertySet> = Rc::new(PropertySet {
            hash: 7,
            modified: true,
        });
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kUserAgent;
        result.AddMatchedProperties(properties, None, data);
        assert!(!result.IsCacheable());
    }
}

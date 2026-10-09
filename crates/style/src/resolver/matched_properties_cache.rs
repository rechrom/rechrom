// Copyright 1999-2013 the Chromium authors and contributors.
// GNU Library General Public License version 2 or later; see COPYING.LIB.
// cpp: third_party/blink/renderer/core/css/resolver/matched_properties_cache.h
// cpp: third_party/blink/renderer/core/css/resolver/matched_properties_cache.cc
// The existing MatchResult/MatchedProperties, CSSPropertyValueSet, mixin
// bindings, ComputedStyle/Builder and StyleResolverState are used directly.
// Keys hold weak references to the original property storage; entries hold
// Persistent roots to actual GC ComputedStyles. No alternate property model,
// default style or backend implementation is supplied.

// Source ledger: physical counts all source lines. Effective removes blank and
// comment-only lines but retains delimiters/declarations and preprocessing.
// mapped + omitted + pending = effective; Rust adapters do not inflate counts.
//                         physical effective mapped omitted pending
// Complete source h:          216       132     82      50       0
// Complete source cc:         535       346    297      49       0
// Complete source total:      751       478    379      99       0
// Production h:45-201 and cc:50-409,418-521 are fully mapped, except the
// boilerplate/debug/metrics listed below. Default ctor cc:86 maps to new().
// Omitted h:24-39 (includes/guards/namespace),41-43 (forwards),47,60,62,
// 120,122,129,131,139-143,171 (allocation/access/friend annotations),92-97,
// 113,169 (GC Trace),124-126 (deleted-copy/debug destructor),204,206-212
// (debug stream/VectorTraits),214,216 (namespace/guard).
// Omitted cc:31-48 (includes/namespace),52,54-59,245-246,273,504 (debug),
// 304-308 (metrics),411-416 (GC Trace),523-535 (debug stream/namespace).
// Non-null bucket values encode the source's in-construction nullptr guards:
// Rust constructs a complete bucket before inserting it, so a partially
// constructed bucket cannot be observed by Find/Clear/weak cleanup.
// Explicit weak cleanup retains the 500-entry limit, 300-entry LRU target,
// virtual clock wrapping and mutation/liveness checks from the source.

#![allow(non_snake_case)]

use super::cascade_filter::CascadeFilter;
use super::match_result::{
    MatchResult, MatchedProperties, MatchedPropertiesData, MatchedPropertySet,
    MixinParameterBindings,
};
use super::style_resolver_state::{
    ComputedStyleHandle, StyleResolverState, StyleResolverStateBackend,
};
use crate::css_property_value_set::{
    CSSPropertyValueSet, CSSPropertyValueSetBackend, CSSPropertyValueSetRuleHandle,
};
use foundation::{EUserModify, HashInts};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::computed_style_constants::PseudoId;
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

// Object-safe adapters expose the existing classes' content equality. Cache
// metadata equality and all key/hit decisions remain in this module.
impl<D: CSSPropertyValueSetBackend> MatchedPropertySet for CSSPropertyValueSet<D>
where
    Self: 'static,
    CSSPropertyValueSetRuleHandle<D>: 'static,
{
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn GetHash(&self) -> u32 {
        CSSPropertyValueSet::GetHash(self)
    }
    fn ModifiedSinceHashing(&self) -> bool {
        CSSPropertyValueSet::ModifiedSinceHashing(self)
    }
    fn Equals(&self, other: &dyn MatchedPropertySet) -> bool {
        if let Some(other) = other.AsAny().downcast_ref::<Self>() {
            return CSSPropertyValueSet::Equals(self, other);
        }
        other
            .AsAny()
            .downcast_ref::<CSSPropertyValueSetRuleHandle<D>>()
            .is_some_and(|other| {
                other.WithPropertySet(|other| CSSPropertyValueSet::Equals(self, other))
            })
    }
}
impl<D: CSSPropertyValueSetBackend> CSSPropertyValueSetRuleHandle<D> {
    fn WithPropertySet<R>(&self, f: impl FnOnce(&CSSPropertyValueSet<D>) -> R) -> R {
        match self {
            Self::Immutable(set) => f(set),
            Self::Mutable(set) => f(&set.borrow()),
        }
    }
}
impl<D: CSSPropertyValueSetBackend> MatchedPropertySet for CSSPropertyValueSetRuleHandle<D>
where
    Self: 'static,
    CSSPropertyValueSet<D>: 'static,
{
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn GetHash(&self) -> u32 {
        self.WithPropertySet(CSSPropertyValueSet::GetHash)
    }
    fn ModifiedSinceHashing(&self) -> bool {
        self.WithPropertySet(CSSPropertyValueSet::ModifiedSinceHashing)
    }
    fn Equals(&self, other: &dyn MatchedPropertySet) -> bool {
        self.WithPropertySet(|set| MatchedPropertySet::Equals(set, other))
    }
}
impl<D: crate::style_rule::MixinParameterBindingsDependencies> MixinParameterBindings
    for crate::style_rule::MixinParameterBindings<D>
where
    Self: 'static,
{
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn GetHash(&self) -> u32 {
        crate::style_rule::MixinParameterBindings::GetHash(self)
    }
    fn Equals(&self, other: &dyn MixinParameterBindings) -> bool {
        other
            .AsAny()
            .downcast_ref::<Self>()
            .is_some_and(|other| self == other)
    }
}

// These operations are owned by DOM and StyleAdjuster, whose types are
// supplied by the existing state backend. No cache decisions are forwarded.
pub trait MatchedPropertiesCacheBackend: StyleResolverStateBackend {
    type ElementTypeForCache: Clone + Eq;
    fn StyleUserModify(&self, style: &ComputedStyle) -> EUserModify;
    fn BuilderUserModify(&self, builder: &ComputedStyleBuilder) -> EUserModify;
    fn CacheEntryIsStyleAdjusted(&self, element_type: &Self::ElementTypeForCache) -> bool;
    fn StyleAdjusterIsCacheCompatible(
        &self,
        parent_a: &ComputedStyle,
        layout_parent_a: &ComputedStyle,
        parent_b: &ComputedStyle,
        layout_parent_b: &ComputedStyle,
    ) -> bool;
    fn ElementIsAtShadowBoundary(&self, element: &Self::Element) -> bool;
    fn ElementCascadeFilter(&self, element: &Self::Element) -> CascadeFilter;
}

// cpp: matched_properties_cache.h:52-57
pub struct CachedMatchedPropertiesKey {
    properties: Weak<dyn MatchedPropertySet>,
    mixin_parameter_bindings: Option<Weak<dyn MixinParameterBindings>>,
    data: MatchedPropertiesData,
}
impl CachedMatchedPropertiesKey {
    fn new(property: &MatchedProperties) -> Self {
        Self {
            properties: Rc::downgrade(&property.properties),
            mixin_parameter_bindings: property
                .mixin_parameter_bindings
                .as_ref()
                .map(Rc::downgrade),
            data: property.data_,
        }
    }
}
// cpp: matched_properties_cache.h:59-90
pub struct Entry<E> {
    pub computed_style: ComputedStyleHandle,
    pub parent_computed_style: ComputedStyleHandle,
    pub layout_parent_style: ComputedStyleHandle,
    pub originating_element_computed_style: Option<ComputedStyleHandle>,
    pub element_type: E,
    pub last_used: u32,
}
impl<E> Entry<E> {
    pub fn HasRunStyleAdjuster<B: MatchedPropertiesCacheBackend<ElementTypeForCache = E>>(
        &self,
        backend: &B,
    ) -> bool {
        backend.CacheEntryIsStyleAdjusted(&self.element_type)
    }
}
// cpp: matched_properties_cache.h:45-57,98-117
pub struct CachedMatchedProperties<E> {
    pub matched_properties: Vec<CachedMatchedPropertiesKey>,
    pub entries: Vec<Entry<E>>,
}
impl<E> CachedMatchedProperties<E> {
    // cpp: matched_properties_cache.cc:64-79
    fn new(entry: Entry<E>, properties: &[MatchedProperties]) -> Self {
        Self {
            matched_properties: properties
                .iter()
                .map(CachedMatchedPropertiesKey::new)
                .collect(),
            entries: vec![entry],
        }
    }
    // cpp: matched_properties_cache.cc:81-84
    pub fn Clear(&mut self) {
        self.matched_properties.clear();
        self.entries.clear();
    }
    // cpp: matched_properties_cache.cc:236-269
    pub fn CorrespondsTo(&self, lookup: &[MatchedProperties]) -> bool {
        if lookup.len() != self.matched_properties.len() {
            return false;
        }
        for (lookup, cached) in lookup.iter().zip(&self.matched_properties) {
            let Some(properties) = cached.properties.upgrade() else {
                return false;
            };
            if properties.ModifiedSinceHashing()
                || !lookup.properties.Equals(properties.as_ref())
                || lookup.data_ != cached.data
            {
                return false;
            }
            match (
                &lookup.mixin_parameter_bindings,
                &cached.mixin_parameter_bindings,
            ) {
                (None, None) => (),
                (Some(lookup), Some(cached)) => {
                    let Some(cached) = cached.upgrade() else {
                        return false;
                    };
                    if !Rc::ptr_eq(lookup, &cached) && !lookup.Equals(cached.as_ref()) {
                        return false;
                    }
                }
                _ => return false,
            }
        }
        true
    }
    // cpp: matched_properties_cache.cc:271-279
    pub fn RefreshKey(&mut self, lookup: &[MatchedProperties]) {
        for (lookup, cached) in lookup.iter().zip(&mut self.matched_properties) {
            cached.properties = Rc::downgrade(&lookup.properties);
            cached.mixin_parameter_bindings =
                lookup.mixin_parameter_bindings.as_ref().map(Rc::downgrade);
        }
    }
    // cpp: matched_properties_cache.cc:418-429
    fn ShouldRemoveMPCEntry(&self) -> bool {
        self.matched_properties.iter().any(|key| {
            key.properties
                .upgrade()
                .is_none_or(|properties| properties.ModifiedSinceHashing())
                || key
                    .mixin_parameter_bindings
                    .as_ref()
                    .is_some_and(|bindings| bindings.upgrade().is_none())
        })
    }
}
// cpp: matched_properties_cache.h:128-149
pub struct AdditionalHash {
    pub hash: u32,
}
pub struct Key<'a> {
    result_: &'a MatchResult,
    hash_: u32,
}
impl<'a> Key<'a> {
    // cpp: matched_properties_cache.cc:88-96
    pub fn new(result: &'a MatchResult, additional: AdditionalHash) -> Self {
        Self {
            result_: result,
            hash_: if result.IsCacheable() {
                ComputeMatchedPropertiesHash(result, additional.hash)
            } else {
                0
            },
        }
    }
    // The unsigned-hash constructor is also the source's test/debug entry.
    pub fn WithHash(result: &'a MatchResult, hash: u32) -> Self {
        Self {
            result_: result,
            hash_: hash,
        }
    }
    pub fn IsCacheable(&self) -> bool {
        self.result_.IsCacheable()
    }
}
// cpp: matched_properties_cache.cc:50-62
fn ComputeMatchedPropertiesHash(result: &MatchResult, additional_hash: u32) -> u32 {
    // Serialize the exact initialized 12-byte MatchedPropertiesHash layout,
    // avoiding unsafe reads of Rust padding. The Data representation is the
    // existing MatchResult's eight-byte source layout, including padding.
    let hashes = result.GetMatchedPropertiesHash();
    let mut bytes = Vec::with_capacity(hashes.len() * 12);
    for entry in hashes {
        bytes.extend_from_slice(&entry.hash.to_ne_bytes());
        let data = entry.data;
        bytes.extend_from_slice(&data.tree_order.to_ne_bytes());
        bytes.push(
            data.link_match_type()
                | (data.valid_property_filter() << 2)
                | ((data.is_inline_style() as u8) << 6)
                | ((data.is_try_style() as u8) << 7),
        );
        bytes.push(data.origin as u8);
        bytes.extend_from_slice(&data.layer_order.to_ne_bytes());
        bytes.push(data.is_try_tactics_style as u8);
        bytes.push(data.padding);
    }
    let hash = HashInts(
        foundation::rapidhash::rapidhash(&bytes) as u32,
        additional_hash,
    );
    // cpp: platform/wtf/hash_traits.h:396-401, EnsureValidHash.
    if hash == 0 || hash == u32::MAX {
        1
    } else {
        hash
    }
}
// Persistent is the existing GC root; a non-null root ensures the reference
// remains live while the cache/state owns it. No unrooted style escapes here.
fn style(handle: &ComputedStyleHandle) -> &ComputedStyle {
    let pointer = handle.Get();
    assert!(!pointer.is_null(), "cache style roots must be non-null");
    unsafe { &*pointer }
}
// cpp: matched_properties_cache.h:119-201
pub struct MatchedPropertiesCache<E> {
    cache_: HashMap<u32, CachedMatchedProperties<E>>,
    cache_entries_: u32,
    clock_: u32,
}
impl<E: Eq> MatchedPropertiesCache<E> {
    // cpp: matched_properties_cache.cc:86
    pub fn new() -> Self {
        Self {
            cache_: HashMap::new(),
            cache_entries_: 0,
            clock_: 0,
        }
    }
    // cpp: matched_properties_cache.cc:98-234
    pub fn Find<B: MatchedPropertiesCacheBackend<ElementTypeForCache = E>>(
        &mut self,
        key: &Key<'_>,
        element_type: &E,
        style_type: PseudoId,
        state: &StyleResolverState<'_, B>,
        backend: &B,
    ) -> Option<&Entry<E>> {
        let item = self.cache_.get(&key.hash_)?;
        if !item.CorrespondsTo(key.result_.GetMatchedProperties()) {
            self.cache_entries_ = self.cache_entries_.wrapping_sub(item.entries.len() as u32);
            self.cache_.remove(&key.hash_);
            return None;
        }
        let parent = state.ParentStyle().expect("MPC requires parent style");
        let parent_style = style(&parent);
        let item = self
            .cache_
            .get_mut(&key.hash_)
            .expect("bucket remains present");
        let mut hit = None;
        for index in (0..item.entries.len()).rev() {
            let entry = &item.entries[index];
            let adjusted = entry.HasRunStyleAdjuster(backend);
            if adjusted && *element_type != entry.element_type {
                continue;
            }
            if state.IsForHighlight() != entry.originating_element_computed_style.is_some() {
                continue;
            }
            let cached_parent = style(&entry.parent_computed_style);
            if state.IsForHighlight() {
                let originating = state
                    .OriginatingElementStyle()
                    .expect("highlight has originating style");
                let originating = style(&originating);
                let cached_originating = style(
                    entry
                        .originating_element_computed_style
                        .as_ref()
                        .expect("highlight cache entry has originating style"),
                );
                if !parent_style.NonHighlightOriginatingElementDataEqual(cached_parent)
                    || !originating.HighlightOriginatingElementDataEqual(cached_originating)
                    || originating.DarkColorScheme() != cached_originating.DarkColorScheme()
                    || originating.InsideLink() != cached_originating.InsideLink()
                {
                    continue;
                }
            } else {
                if !parent_style.InheritedEqualIncludingInheritedVariables(cached_parent) {
                    continue;
                }
                if adjusted {
                    if style_type != style(&entry.computed_style).StyleType() {
                        continue;
                    }
                    let layout_parent = state
                        .LayoutParentStyle()
                        .expect("adjusted MPC requires layout parent style");
                    if !backend.StyleAdjusterIsCacheCompatible(
                        parent_style,
                        style(&layout_parent),
                        cached_parent,
                        style(&entry.layout_parent_style),
                    ) {
                        continue;
                    }
                }
                if style(&entry.computed_style).HasExplicitInheritance()
                    && parent.Get() != entry.parent_computed_style.Get()
                    && !parent_style.NonInheritedEqual(cached_parent)
                {
                    continue;
                }
            }
            if backend.ElementIsAtShadowBoundary(&state.GetElement())
                && backend.StyleUserModify(cached_parent)
                    != ComputedStyleInitialValues::InitialUserModify()
            {
                continue;
            }
            if cached_parent.IsEnsuredInDisplayNone() && !parent_style.IsEnsuredInDisplayNone() {
                continue;
            }
            hit = Some(index);
            break;
        }
        let index = hit?;
        item.entries[index].last_used = self.clock_;
        self.clock_ = self.clock_.wrapping_add(1);
        item.RefreshKey(key.result_.GetMatchedProperties());
        Some(&item.entries[index])
    }
    // cpp: matched_properties_cache.cc:281-309
    pub fn Add(
        &mut self,
        key: &Key<'_>,
        element_type: E,
        computed_style: ComputedStyleHandle,
        parent_computed_style: ComputedStyleHandle,
        layout_parent_style: ComputedStyleHandle,
        originating_element_computed_style: Option<ComputedStyleHandle>,
    ) {
        let entry = Entry {
            computed_style,
            parent_computed_style,
            layout_parent_style,
            originating_element_computed_style,
            element_type,
            last_used: self.clock_,
        };
        self.clock_ = self.clock_.wrapping_add(1);
        match self.cache_.entry(key.hash_) {
            std::collections::hash_map::Entry::Vacant(bucket) => {
                bucket.insert(CachedMatchedProperties::new(
                    entry,
                    key.result_.GetMatchedProperties(),
                ));
            }
            std::collections::hash_map::Entry::Occupied(mut bucket) => {
                bucket.get_mut().entries.push(entry);
            }
        }
        self.cache_entries_ = self.cache_entries_.wrapping_add(1);
    }
    // cpp: matched_properties_cache.cc:311-322
    pub fn Clear(&mut self) {
        for entry in self.cache_.values_mut() {
            entry.Clear();
        }
        self.cache_.clear();
        self.cache_entries_ = 0;
    }
    // cpp: matched_properties_cache.cc:324-328
    pub fn ClearViewportDependent(&mut self) {
        self.EraseEntriesIf(|entry| style(&entry.computed_style).HasViewportUnits());
    }
    // cpp: matched_properties_cache.cc:330-367
    pub fn IsStyleCacheable(builder: &ComputedStyleBuilder) -> bool {
        if builder.HasAttrFunction() {
            return false;
        }
        if builder.Zoom() != ComputedStyleInitialValues::InitialZoom() {
            return false;
        }
        if builder.HasContainerRelativeValue() {
            return false;
        }
        if builder.HasAnchorFunctions() {
            return false;
        }
        if builder.HasSiblingFunctions() {
            return false;
        }
        if builder.AffectedByFunctionalMedia() || builder.AffectedByFunctionalNavigation() {
            return false;
        }
        if builder.HasElementDependentRandomFunctions() {
            return false;
        }
        true
    }
    // cpp: matched_properties_cache.cc:369-409
    pub fn IsCacheable<B: MatchedPropertiesCacheBackend<ElementTypeForCache = E>>(
        state: &StyleResolverState<'_, B>,
        backend: &B,
    ) -> bool {
        let parent = state
            .ParentStyle()
            .expect("MPC eligibility requires parent style");
        let builder = state.StyleBuilder();
        if !Self::IsStyleCacheable(&builder) {
            return false;
        }
        if state.HasTreeScopedReference() {
            return false;
        }
        if backend.ElementIsAtShadowBoundary(&state.GetElement())
            && backend.BuilderUserModify(&builder) != backend.StyleUserModify(style(&parent))
        {
            return false;
        }
        if !backend.ElementCascadeFilter(&state.GetElement()).IsEmpty() {
            return false;
        }
        true
    }
    // cpp: matched_properties_cache.cc:433-451
    fn EraseEntriesIf(&mut self, mut predicate: impl FnMut(&Entry<E>) -> bool) {
        self.cache_.retain(|_, item| {
            let before = item.entries.len();
            item.entries.retain(|entry| !predicate(entry));
            self.cache_entries_ = self
                .cache_entries_
                .wrapping_sub((before - item.entries.len()) as u32);
            !item.entries.is_empty()
        });
    }
    // cpp: matched_properties_cache.cc:453-521
    // Existing Rc-backed MatchResult property storage supplies liveness through
    // Weak::upgrade. The engine invokes this at its weak-cleanup point; Find
    // also rejects expired or mutated keys before touching cached styles.
    pub fn CleanMatchedPropertiesCache(&mut self) {
        const CACHE_LIMIT: u32 = 500;
        const PRUNE_TARGET: usize = 300;
        if self.cache_entries_ <= CACHE_LIMIT {
            self.cache_.retain(|_, item| {
                if item.ShouldRemoveMPCEntry() {
                    self.cache_entries_ =
                        self.cache_entries_.wrapping_sub(item.entries.len() as u32);
                    false
                } else {
                    true
                }
            });
            return;
        }
        let mut live_entries = Vec::with_capacity(self.cache_entries_ as usize);
        self.cache_.retain(|_, item| {
            if item.ShouldRemoveMPCEntry() {
                self.cache_entries_ = self.cache_entries_.wrapping_sub(item.entries.len() as u32);
                false
            } else {
                live_entries.extend(item.entries.iter().map(|entry| entry.last_used));
                true
            }
        });
        if live_entries.len() > PRUNE_TARGET {
            let cutoff_index = live_entries.len() - PRUNE_TARGET - 1;
            live_entries.select_nth_unstable(cutoff_index);
            let cutoff = live_entries[cutoff_index];
            self.EraseEntriesIf(|entry| entry.last_used <= cutoff);
        }
    }
}

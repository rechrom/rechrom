// Copyright 1999-2026 The Chromium Authors and other contributors.
// Use of this source code is governed by the license in Chromium's
// third_party/blink/renderer/core/css/resolver/scoped_style_resolver.h.
// Source ledger: comment/blank lines stripped; braces retained.
// scoped_style_resolver.h: physical 191, effective 111, mapped 80,
// omitted 31, production pending 0.
// scoped_style_resolver.cc: physical 554, effective 422, mapped 364,
// omitted 58, production pending 0.
// Omitted: license/header/namespace/access/export/GC/copy-delete scaffolding,
// DCHECK/debug-only reference-group verification and GC Trace.
// h:72 CounterStyleRulesChanged has no definition in the source pair; omitted.
// Source has no independent property registry body; property registration stays
// in StyleEngine. All author-sheet matching, layer/at-rule registry and implicit
// scope lifecycle control present in this source pair is mapped below.
// h mapped: 61,65-66,68-69,71,74,76,78-81,83-90,108,110-129,134-137,139-146,148,150-154,156,158-160,162-164,166-168
// h mapped: 170,176-180,182-183,185-187
// h omitted: 29-30,32-41,43,45-52,58-60,62-63,72,131,133,189,191
// cc mapped: 59-67,69-75,77-82,84,87-89,91-106,108-113,115-124,126-128,137-154,156-160,162,164-170,172-187,189-193
// cc mapped: 195-198,200-202,204-206,208-213,215-224,226-227,229-233,235,240-242,244-248,250-258,260-262,267-270
// cc mapped: 273-275,281-284,301-304,306-315,317-323,325-331,333-341,343,346-350,352-355,357,362-364,366-389,391-392
// cc mapped: 394-399,401-407,409-413,415-419,421-422,424-428,430-433,435-436,446-456,458-471,476-479,483-490,492-494
// cc mapped: 496-497,499-503,505-512,514-520,522-523,527,529-534,539-540
// cc omitted: 29,31,33-55,57,163,228,285,288-299,345,393,444,481,495,542-552,554
// Shared-mutability integration gap: canonical collector RuleSet handles are
// Rc<RuleSet<B>>, while StyleSheetContents currently exposes Rc<RefCell<RuleSet>>.
// RuleSetCompactRulesIfNeeded is a required operation on that actual owner;
// it must preserve allocation identity and cannot clone or substitute rules.
// This module does not introduce a second rule model or a mutation registry.

#![allow(non_snake_case, non_camel_case_types)]

use crate::active_style_sheets::ActiveStyleSheet;
use crate::cascade_layer::CascadeLayer;
use crate::cascade_layer_map::{CascadeLayerMap, CascadeLayerRuleSet};
use crate::cascade_layered::CascadeLayered;
use crate::element_rule_collector::{
    AddRuleSetToRuleSetGroupList, ElementRuleCollector, ElementRuleCollectorBackend, MatchRequest,
    RuleSetGroup,
};
use crate::resolver::cascade_origin::CascadeOrigin;
use crate::resolver::media_query_result::MediaQueryResultFlags;
use crate::rule_set::{
    FontFaceRuleKind, FunctionRuleKind, RuleSet, RuleSetBackend, RuleSetStyleScope, TypedRuleRef,
};
use crate::style_rule_font_feature_values::{FontFeatureValuesStorage, StyleRuleFontFeatureValues};
use crate::style_sheet_contents::{StyleSheetContents, StyleSheetContentsBackend};
use foundation::{AtomicString, String};
use std::cell::{Cell, Ref, RefCell};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

pub type ActiveStyleSheetVector<B> =
    Vec<ActiveStyleSheet<<B as ElementRuleCollectorBackend>::CSSStyleSheet, RuleSet<B>>>;
pub type VisitedStyleSheetContents<B> =
    HashMap<usize, Rc<StyleSheetContents<<B as ScopedStyleResolverSheetBackend>::ContentsBackend>>>;
type KeyframesRuleMap<B> = HashMap<
    AtomicString,
    CascadeLayered<Rc<<B as crate::style_rule::StyleRuleDependencies>::StyleRuleKeyframes>>,
>;
type PositionTryRuleMap<B> = HashMap<
    AtomicString,
    CascadeLayered<Rc<<B as crate::style_rule::StyleRuleDependencies>::StyleRulePositionTry>>,
>;
type FunctionRuleMap<B> = HashMap<AtomicString, CascadeLayered<TypedRuleRef<B, FunctionRuleKind>>>;

/// TreeScope/Document, timeline and DOM-owned implicit scope data operations.
/// ScopedStyleResolver is the actual resolver type returned at scope boundaries.
pub trait ScopedStyleResolverScopeBackend:
    ElementRuleCollectorBackend<StyleRuleFontFeatureValues = StyleRuleFontFeatureValues> + 'static
{
    fn ScopeParent(scope: &Self::TreeScope) -> Option<Rc<Self::TreeScope>>;
    fn ScopeResolver(scope: &Self::TreeScope) -> Option<Rc<ScopedStyleResolver<Self>>>
    where
        Self: ScopedStyleResolverBackend;
    fn ScopeRoot(scope: &Self::TreeScope) -> Rc<Self::ContainerNode>;
    fn ScopeRootIsDocument(scope: &Self::TreeScope) -> bool;
    fn ScopeDocument(scope: &Self::TreeScope) -> Rc<Self::Document>;
    fn DocumentElement(document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn ScopeShadowHost(scope: &Self::TreeScope) -> Rc<Self::Element>;
    fn ElementMarkKeyframesSubtreeStyleChange(element: &Self::Element);
    fn TimelineInvalidateKeyframeEffects(document: &Self::Document, scope: &Self::TreeScope);
    type StyleScopeData;
    fn ElementEnsureStyleScopeData(element: &Self::Element) -> Rc<Self::StyleScopeData>;
    fn ElementStyleScopeData(element: &Self::Element) -> Option<Rc<Self::StyleScopeData>>;
    fn ScopeDataAddTriggeredImplicitScope(data: &Self::StyleScopeData, scope: &Self::StyleScope);
    fn ScopeDataRemoveTriggeredImplicitScope(data: &Self::StyleScopeData, scope: &Self::StyleScope);
}

/// Existing sheet/content/RuleFeatureSet owners, never resolver algorithms.
pub trait ScopedStyleResolverSheetBackend: ScopedStyleResolverScopeBackend {
    type ContentsBackend: StyleSheetContentsBackend;
    fn SheetMediaQueryResultFlags(sheet: &Self::CSSStyleSheet) -> MediaQueryResultFlags;
    fn ActualSheetContents(
        sheet: &Self::CSSStyleSheet,
    ) -> Rc<StyleSheetContents<Self::ContentsBackend>>;
    fn SheetOwnerParentOrShadowHost(sheet: &Self::CSSStyleSheet) -> Option<Rc<Self::Element>>;
    fn SheetIsAdoptedByScope(sheet: &Self::CSSStyleSheet, scope: &Self::TreeScope) -> bool;
    fn RuleSetCompactRulesIfNeeded(rules: &Rc<RuleSet<Self>>);
    fn FeaturesMerge(features: &mut Self::RuleFeatureSet, other: &Self::RuleFeatureSet);
}

/// Counter maps, font selector/cache and named-rule value accessors.
pub trait ScopedStyleResolverRegistryBackend: ScopedStyleResolverSheetBackend {
    type CounterStyleMap;
    type CSSFontSelector;
    type FontFace;
    type PageRuleCollector;
    fn NewAuthorCounterStyleMap(scope: &Rc<Self::TreeScope>) -> Rc<Self::CounterStyleMap>;
    fn CounterMapAddCounterStyles(map: &Self::CounterStyleMap, rules: &RuleSet<Self>);
    fn CounterMapDispose(map: &Self::CounterStyleMap);
    fn DocumentFontSelector(document: &Self::Document) -> Rc<Self::CSSFontSelector>;
    fn FontFaceCreate(
        document: &Self::Document,
        rule: &CascadeLayered<TypedRuleRef<Self, FontFaceRuleKind>>,
        is_user: bool,
    ) -> Option<Rc<Self::FontFace>>;
    fn FontFaceCacheAdd(
        selector: &Self::CSSFontSelector,
        rule: &TypedRuleRef<Self, FontFaceRuleKind>,
        font: Rc<Self::FontFace>,
    );
    fn DocumentResolverInvalidateMatchedPropertiesCache(document: &Self::Document);
    fn KeyframesName(rule: &Self::StyleRuleKeyframes) -> AtomicString;
    fn KeyframesVendorPrefixed(rule: &Self::StyleRuleKeyframes) -> bool;
    fn PositionTryName(rule: &Self::StyleRulePositionTry) -> AtomicString;
    fn FoldCase(string: &String) -> String;
    fn PageCollectorMatchRules(
        collector: &mut Self::PageRuleCollector,
        rules: &Rc<RuleSet<Self>>,
        origin: CascadeOrigin,
        scope: &Rc<Self::TreeScope>,
        layer_map: Option<&CascadeLayerMap>,
    );
}
pub trait ScopedStyleResolverBackend: ScopedStyleResolverRegistryBackend {}

// Adapter for the existing canonical RuleSet, not a replacement representation.
impl<B: RuleSetBackend> CascadeLayerRuleSet for RuleSet<B> {
    fn CascadeLayers(&self) -> Option<&CascadeLayer> {
        self.HasCascadeLayers()
            .then(|| RuleSet::CascadeLayers(self))
    }
}

fn CompareLayerOrder(
    map: Option<&CascadeLayerMap>,
    old: Option<&CascadeLayer>,
    new: Option<&CascadeLayer>,
) -> Ordering {
    if old == new {
        Ordering::Equal
    } else {
        map.expect("different layers require the cascade map")
            .CompareLayerOrder(old, new)
    }
}

// cpp: resolver/style_resolver_utils.h:37-57
fn AddNameDefiningRules<T: Clone>(
    input: &[CascadeLayered<T>],
    layers: Option<&CascadeLayerMap>,
    output: &mut HashMap<AtomicString, CascadeLayered<T>>,
    name: impl Fn(&T) -> AtomicString,
) {
    for rule in input {
        let key = name(&rule.value);
        if output.get(&key).is_none_or(|old| {
            CompareLayerOrder(layers, old.layer.as_ref(), rule.layer.as_ref()) != Ordering::Greater
        }) {
            output.insert(key, rule.clone());
        }
    }
}

// cpp: scoped_style_resolver.h:61-185
pub struct ScopedStyleResolver<B: ScopedStyleResolverBackend> {
    backend: Rc<B>,
    scope_: Rc<B::TreeScope>,
    active_style_sheets_: RefCell<ActiveStyleSheetVector<B>>,
    media_query_result_flags_: RefCell<MediaQueryResultFlags>,
    rule_set_groups_: RefCell<Vec<RuleSetGroup<B>>>,
    keyframes_rule_map_: RefCell<KeyframesRuleMap<B>>,
    position_try_rule_map_: RefCell<PositionTryRuleMap<B>>,
    function_rule_map_: RefCell<FunctionRuleMap<B>>,
    font_feature_values_storage_map_: RefCell<HashMap<String, FontFeatureValuesStorage>>,
    font_feature_values_rule_map_: RefCell<HashMap<String, Vec<Rc<StyleRuleFontFeatureValues>>>>,
    counter_style_map_: RefCell<Option<Rc<B::CounterStyleMap>>>,
    cascade_layer_map_: RefCell<Option<Rc<CascadeLayerMap>>>,
    has_unresolved_keyframes_rule_: Cell<bool>,
    needs_append_all_sheets_: Cell<bool>,
}
impl<B: ScopedStyleResolverBackend> ScopedStyleResolver<B> {
    // cpp: scoped_style_resolver.h:61
    pub fn new(backend: Rc<B>, scope: Rc<B::TreeScope>) -> Self {
        Self {
            backend,
            scope_: scope,
            active_style_sheets_: RefCell::new(Vec::new()),
            media_query_result_flags_: RefCell::new(MediaQueryResultFlags::default()),
            rule_set_groups_: RefCell::new(Vec::new()),
            keyframes_rule_map_: RefCell::new(HashMap::new()),
            position_try_rule_map_: RefCell::new(HashMap::new()),
            function_rule_map_: RefCell::new(HashMap::new()),
            font_feature_values_storage_map_: RefCell::new(HashMap::new()),
            font_feature_values_rule_map_: RefCell::new(HashMap::new()),
            counter_style_map_: RefCell::new(None),
            cascade_layer_map_: RefCell::new(None),
            has_unresolved_keyframes_rule_: Cell::new(false),
            needs_append_all_sheets_: Cell::new(false),
        }
    }
    // cpp: scoped_style_resolver.h:65-90
    pub fn GetTreeScope(&self) -> &Rc<B::TreeScope> {
        &self.scope_
    }
    pub fn GetCounterStyleMap(&self) -> Option<Rc<B::CounterStyleMap>> {
        self.counter_style_map_.borrow().clone()
    }
    pub fn HasCascadeLayerMap(&self) -> bool {
        self.cascade_layer_map_.borrow().is_some()
    }
    pub fn GetCascadeLayerMap(&self) -> Option<Rc<CascadeLayerMap>> {
        self.cascade_layer_map_.borrow().clone()
    }
    pub fn GetActiveStyleSheets(&self) -> Ref<'_, ActiveStyleSheetVector<B>> {
        self.active_style_sheets_.borrow()
    }
    // cpp: scoped_style_resolver.h:116-120
    pub fn SetHasUnresolvedKeyframesRule(&self) {
        self.has_unresolved_keyframes_rule_.set(true);
    }
    pub fn NeedsAppendAllSheets(&self) -> bool {
        self.needs_append_all_sheets_.get()
    }
    pub fn SetNeedsAppendAllSheets(&self) {
        self.needs_append_all_sheets_.set(true);
    }
    // cpp: scoped_style_resolver.cc:59-67
    pub fn Parent(&self) -> Option<Rc<Self>> {
        let mut scope = B::ScopeParent(&self.scope_);
        while let Some(current) = scope {
            if let Some(resolver) = B::ScopeResolver(&current) {
                return Some(resolver);
            }
            scope = B::ScopeParent(&current);
        }
        None
    }
    // cpp: scoped_style_resolver.cc:69-75
    fn AddKeyframeRules(&self, rules: &RuleSet<B>) {
        for rule in rules.KeyframesRules() {
            self.AddKeyframeStyle(rule);
        }
    }
    // cpp: scoped_style_resolver.cc:77-82
    fn EnsureCounterStyleMap(&self) -> Rc<B::CounterStyleMap> {
        if let Some(map) = self.GetCounterStyleMap() {
            return map;
        }
        let map = B::NewAuthorCounterStyleMap(&self.scope_);
        *self.counter_style_map_.borrow_mut() = Some(map.clone());
        map
    }
    // cpp: scoped_style_resolver.cc:84-106
    fn AddFontFaceRules(&self, rules: &RuleSet<B>) {
        if !B::ScopeRootIsDocument(&self.scope_) {
            return;
        }
        let document = B::ScopeDocument(&self.scope_);
        let selector = B::DocumentFontSelector(&document);
        for rule in rules.FontFaceRules() {
            if let Some(font) = B::FontFaceCreate(&document, rule, false) {
                B::FontFaceCacheAdd(&selector, &rule.value, font);
            }
        }
        if !rules.FontFaceRules().is_empty() {
            B::DocumentResolverInvalidateMatchedPropertiesCache(&document);
        }
    }
    // cpp: scoped_style_resolver.cc:108-113
    fn AddCounterStyleRules(&self, rules: &RuleSet<B>) {
        if !rules.CounterStyleRules().is_empty() {
            B::CounterMapAddCounterStyles(&self.EnsureCounterStyleMap(), rules);
        }
    }
    // cpp: scoped_style_resolver.cc:115-154
    pub fn AppendActiveStyleSheets(
        &self,
        index: usize,
        sheets: &[ActiveStyleSheet<B::CSSStyleSheet, RuleSet<B>>],
    ) {
        for sheet in &sheets[index..] {
            self.media_query_result_flags_
                .borrow_mut()
                .Add(&B::SheetMediaQueryResultFlags(&sheet.style_sheet));
            let Some(rules) = &sheet.rule_set else {
                continue;
            };
            let duplicate = self
                .active_style_sheets_
                .borrow()
                .last()
                .and_then(|sheet| sheet.rule_set.as_ref())
                .is_some_and(|last| Rc::ptr_eq(last, rules));
            if !duplicate {
                self.active_style_sheets_.borrow_mut().push(sheet.clone());
                B::RuleSetCompactRulesIfNeeded(rules);
                self.AddKeyframeRules(rules);
                self.AddFontFaceRules(rules);
                self.AddCounterStyleRules(rules);
                let layers = self.GetCascadeLayerMap();
                AddNameDefiningRules(
                    rules.PositionTryRules(),
                    layers.as_deref(),
                    &mut self.position_try_rule_map_.borrow_mut(),
                    |rule| B::PositionTryName(rule),
                );
                AddNameDefiningRules(
                    rules.FunctionRules(),
                    layers.as_deref(),
                    &mut self.function_rule_map_.borrow_mut(),
                    |rule| rule.Name().clone(),
                );
                self.AddFontFeatureValuesRules(rules);
                AddRuleSetToRuleSetGroupList(
                    rules.clone(),
                    &mut self.rule_set_groups_.borrow_mut(),
                );
            }
            self.AddImplicitScopeTriggers(&sheet.style_sheet, rules);
        }
    }
    // cpp: scoped_style_resolver.cc:156-170
    pub fn CollectFeaturesTo(
        &self,
        features: &mut B::RuleFeatureSet,
        visited: &mut VisitedStyleSheetContents<B>,
    ) {
        B::MutableMediaQueryResultFlags(features).Add(&self.media_query_result_flags_.borrow());
        for sheet in self.active_style_sheets_.borrow().iter() {
            let contents = B::ActualSheetContents(&sheet.style_sheet);
            if contents.HasOneClient()
                || visited
                    .insert(Rc::as_ptr(&contents) as usize, contents)
                    .is_none()
            {
                B::FeaturesMerge(
                    features,
                    sheet.rule_set.as_ref().expect("active rule set").Features(),
                );
            }
        }
    }
    // cpp: scoped_style_resolver.cc:172-187
    pub fn ResetStyle(&self) {
        self.RemoveImplicitScopeTriggers();
        self.active_style_sheets_.borrow_mut().clear();
        self.rule_set_groups_.borrow_mut().clear();
        self.media_query_result_flags_.borrow_mut().Clear();
        self.keyframes_rule_map_.borrow_mut().clear();
        self.position_try_rule_map_.borrow_mut().clear();
        self.font_feature_values_storage_map_.borrow_mut().clear();
        self.font_feature_values_rule_map_.borrow_mut().clear();
        self.function_rule_map_.borrow_mut().clear();
        if let Some(map) = self.GetCounterStyleMap() {
            B::CounterMapDispose(&map);
        }
        *self.cascade_layer_map_.borrow_mut() = None;
        self.needs_append_all_sheets_.set(false);
    }
    // cpp: scoped_style_resolver.cc:189-202
    pub fn KeyframeStylesForAnimation(
        &self,
        name: &AtomicString,
    ) -> Option<Rc<B::StyleRuleKeyframes>> {
        self.keyframes_rule_map_
            .borrow()
            .get(name)
            .map(|rule| rule.value.clone())
    }
    // cpp: scoped_style_resolver.cc:204-213
    fn AddKeyframeStyle(&self, rule: &CascadeLayered<Rc<B::StyleRuleKeyframes>>) {
        let name = B::KeyframesName(&rule.value);
        let should_override = self
            .keyframes_rule_map_
            .borrow()
            .get(&name)
            .is_none_or(|old| self.KeyframeStyleShouldOverride(rule, old));
        if should_override {
            self.keyframes_rule_map_
                .borrow_mut()
                .insert(name, rule.clone());
        }
    }
    // cpp: scoped_style_resolver.cc:215-224
    fn KeyframeStyleShouldOverride(
        &self,
        new: &CascadeLayered<Rc<B::StyleRuleKeyframes>>,
        old: &CascadeLayered<Rc<B::StyleRuleKeyframes>>,
    ) -> bool {
        let old_prefixed = B::KeyframesVendorPrefixed(&old.value);
        if B::KeyframesVendorPrefixed(&new.value) != old_prefixed {
            return old_prefixed;
        }
        CompareLayerOrder(
            self.GetCascadeLayerMap().as_deref(),
            old.layer.as_ref(),
            new.layer.as_ref(),
        ) != Ordering::Greater
    }
    // cpp: scoped_style_resolver.cc:226-233
    pub fn InvalidationRootForTreeScope(scope: &B::TreeScope) -> Rc<B::Element> {
        if B::ScopeRootIsDocument(scope) {
            B::DocumentElement(&B::ScopeDocument(scope)).expect("document element")
        } else {
            B::ScopeShadowHost(scope)
        }
    }
    // cpp: scoped_style_resolver.cc:235-275
    pub fn KeyframesRulesAdded(scope: &B::TreeScope) {
        let document = B::ScopeDocument(scope);
        if B::DocumentElement(&document).is_none() {
            return;
        }
        let resolver = B::ScopeResolver(scope);
        let parent = B::ScopeParent(scope).and_then(|parent| B::ScopeResolver(&parent));
        let mut unresolved = false;
        if let Some(resolver) = resolver {
            if resolver.has_unresolved_keyframes_rule_.replace(false) {
                unresolved = true;
            }
        }
        if let Some(resolver) = parent {
            if resolver.has_unresolved_keyframes_rule_.replace(false) {
                unresolved = true;
            }
        }
        if unresolved {
            B::ElementMarkKeyframesSubtreeStyleChange(&Self::InvalidationRootForTreeScope(scope));
            return;
        }
        B::TimelineInvalidateKeyframeEffects(&document, scope);
    }
    // cpp: scoped_style_resolver.cc:281-304
    fn ForAllStylesheets(
        &self,
        collector: &mut ElementRuleCollector<'_, B>,
        scope_root: Rc<B::ContainerNode>,
        mut collect: impl FnMut(&mut ElementRuleCollector<'_, B>, &MatchRequest<'_, B>),
    ) {
        for group in self.rule_set_groups_.borrow().iter() {
            let request =
                MatchRequest::ForCollector(group, Some(scope_root.clone()), None, collector);
            collect(collector, &request);
        }
    }
    // cpp: scoped_style_resolver.cc:306-315
    pub fn CollectMatchingElementScopeRules(
        &self,
        root: Rc<B::ContainerNode>,
        collector: &mut ElementRuleCollector<'_, B>,
        parts: Option<&B::PartNames>,
    ) {
        self.ForAllStylesheets(collector, root, |collector, request| {
            collector.CollectMatchingRules(request, parts)
        });
    }
    // cpp: scoped_style_resolver.cc:317-323
    pub fn CollectMatchingShadowHostRules(&self, collector: &mut ElementRuleCollector<'_, B>) {
        self.ForAllStylesheets(
            collector,
            B::ScopeRoot(&self.scope_),
            |collector, request| collector.CollectMatchingShadowHostRules(request),
        );
    }
    // cpp: scoped_style_resolver.cc:325-331
    pub fn CollectMatchingSlottedRules(&self, collector: &mut ElementRuleCollector<'_, B>) {
        self.ForAllStylesheets(
            collector,
            B::ScopeRoot(&self.scope_),
            |collector, request| collector.CollectMatchingSlottedRules(request),
        );
    }
    // cpp: scoped_style_resolver.cc:333-341
    pub fn CollectMatchingPartPseudoRules(
        &self,
        collector: &mut ElementRuleCollector<'_, B>,
        parts: Option<&B::PartNames>,
    ) {
        self.ForAllStylesheets(
            collector,
            B::ScopeRoot(&self.scope_),
            |collector, request| collector.CollectMatchingPartPseudoRules(request, parts),
        );
    }
    // cpp: scoped_style_resolver.cc:343-350
    pub fn MatchPageRules(&self, collector: &mut B::PageRuleCollector) {
        let layers = self.GetCascadeLayerMap();
        for sheet in self.active_style_sheets_.borrow().iter() {
            B::PageCollectorMatchRules(
                collector,
                sheet.rule_set.as_ref().expect("active rule set"),
                CascadeOrigin::kAuthor,
                &self.scope_,
                layers.as_deref(),
            );
        }
    }
    // cpp: scoped_style_resolver.cc:352-355
    pub fn RebuildCascadeLayerMap(
        &self,
        sheets: &[ActiveStyleSheet<B::CSSStyleSheet, RuleSet<B>>],
    ) {
        *self.cascade_layer_map_.borrow_mut() = Some(Rc::new(CascadeLayerMap::new(sheets)));
    }
    // cpp: scoped_style_resolver.cc:357-389
    fn AddFontFeatureValuesRules(&self, rules: &RuleSet<B>) {
        if !B::ScopeRootIsDocument(&self.scope_) {
            return;
        }
        let layers = self.GetCascadeLayerMap();
        for layered in rules.FontFeatureValuesRules() {
            for family in layered.value.GetFamilies() {
                let order = layers
                    .as_ref()
                    .zip(layered.layer.as_ref())
                    .map_or(CascadeLayerMap::kImplicitOuterLayerOrder, |(map, layer)| {
                        map.GetLayerOrder(layer)
                    });
                let key = B::FoldCase(&String::from_utf16(
                    family.utf16_units().unwrap_or_default(),
                ));
                {
                    let mut storage = self.font_feature_values_storage_map_.borrow_mut();
                    match storage.entry(key.clone()) {
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            let mut value = layered.value.Storage().clone();
                            value.SetLayerOrder(order);
                            entry.insert(value);
                            self.font_feature_values_rule_map_
                                .borrow_mut()
                                .insert(key.clone(), Vec::new());
                        }
                        std::collections::hash_map::Entry::Occupied(mut entry) => entry
                            .get_mut()
                            .FuseUpdate(layered.value.Storage(), order as u32),
                    }
                }
                self.font_feature_values_rule_map_
                    .borrow_mut()
                    .get_mut(&key)
                    .expect("font feature family")
                    .push(layered.value.clone());
            }
        }
    }
    // cpp: scoped_style_resolver.cc:391-399
    pub fn PositionTryForName(&self, name: &AtomicString) -> Option<Rc<B::StyleRulePositionTry>> {
        self.position_try_rule_map_
            .borrow()
            .get(name)
            .map(|rule| rule.value.clone())
    }
    // cpp: scoped_style_resolver.cc:401-407
    pub fn FunctionForName(&self, name: &String) -> Option<TypedRuleRef<B, FunctionRuleKind>> {
        let name = if name.IsNull() {
            AtomicString::default()
        } else {
            AtomicString::from_utf16(name.Span16().unwrap_or_default())
        };
        self.function_rule_map_
            .borrow()
            .get(&name)
            .map(|rule| rule.value.clone())
    }
    // cpp: scoped_style_resolver.cc:409-422
    pub fn FontFeatureValuesForFamily(
        &self,
        family: AtomicString,
    ) -> Option<Ref<'_, FontFeatureValuesStorage>> {
        if self.font_feature_values_storage_map_.borrow().is_empty() || family.empty() {
            return None;
        }
        let key = B::FoldCase(&String::from_utf16(
            family.utf16_units().unwrap_or_default(),
        ));
        Ref::filter_map(self.font_feature_values_storage_map_.borrow(), |map| {
            map.get(&key)
        })
        .ok()
    }
    // cpp: scoped_style_resolver.cc:424-436
    pub fn FontFeatureValuesRulesForFamily(
        &self,
        family: AtomicString,
    ) -> Option<Ref<'_, Vec<Rc<StyleRuleFontFeatureValues>>>> {
        if self.font_feature_values_rule_map_.borrow().is_empty() || family.empty() {
            return None;
        }
        let key = B::FoldCase(&String::from_utf16(
            family.utf16_units().unwrap_or_default(),
        ));
        Ref::filter_map(self.font_feature_values_rule_map_.borrow(), |map| {
            map.get(&key)
        })
        .ok()
    }
    // cpp: scoped_style_resolver.cc:446-456
    fn ImplicitScopeTrigger(&self, sheet: &B::CSSStyleSheet) -> Option<Rc<B::Element>> {
        if let Some(owner) = B::SheetOwnerParentOrShadowHost(sheet) {
            return Some(owner);
        }
        if B::SheetIsAdoptedByScope(sheet, &self.scope_) && !B::ScopeRootIsDocument(&self.scope_) {
            return Some(B::ScopeShadowHost(&self.scope_));
        }
        None
    }
    // cpp: scoped_style_resolver.cc:458-479
    fn ForEachImplicitScopeTrigger(
        &self,
        sheet: &B::CSSStyleSheet,
        rules: &RuleSet<B>,
        mut func: impl FnMut(&B::Element, &B::StyleScope),
    ) {
        for interval in rules.ScopeIntervals() {
            let mut scope = interval.value.as_deref();
            while let Some(current) = scope {
                if self.backend.StyleScopeIsImplicit(current) {
                    if let Some(root) = self.ImplicitScopeTrigger(sheet) {
                        func(&root, current);
                    }
                }
                scope = current.Parent();
            }
        }
    }
    // cpp: scoped_style_resolver.cc:483-490
    fn AddImplicitScopeTriggers(&self, sheet: &B::CSSStyleSheet, rules: &RuleSet<B>) {
        self.ForEachImplicitScopeTrigger(sheet, rules, |element, scope| {
            self.AddImplicitScopeTrigger(element, scope)
        });
    }
    // cpp: scoped_style_resolver.cc:492-497
    fn AddImplicitScopeTrigger(&self, element: &B::Element, scope: &B::StyleScope) {
        B::ScopeDataAddTriggeredImplicitScope(&B::ElementEnsureStyleScopeData(element), scope);
    }
    // cpp: scoped_style_resolver.cc:499-503
    fn RemoveImplicitScopeTriggers(&self) {
        for sheet in self.active_style_sheets_.borrow().iter() {
            self.RemoveImplicitScopeTriggersForSheet(
                &sheet.style_sheet,
                sheet.rule_set.as_ref().expect("active rule set"),
            );
        }
    }
    // cpp: scoped_style_resolver.cc:505-512
    fn RemoveImplicitScopeTriggersForSheet(&self, sheet: &B::CSSStyleSheet, rules: &RuleSet<B>) {
        self.ForEachImplicitScopeTrigger(sheet, rules, |element, scope| {
            self.RemoveImplicitScopeTrigger(element, scope)
        });
    }
    // cpp: scoped_style_resolver.cc:514-520
    fn RemoveImplicitScopeTrigger(&self, element: &B::Element, scope: &B::StyleScope) {
        if let Some(data) = B::ElementStyleScopeData(element) {
            B::ScopeDataRemoveTriggeredImplicitScope(&data, scope);
        }
    }
    // cpp: scoped_style_resolver.cc:522-540
    pub fn QuietlySwapActiveStyleSheets(&self, other: &mut ActiveStyleSheetVector<B>) {
        self.RemoveImplicitScopeTriggers();
        std::mem::swap(&mut *self.active_style_sheets_.borrow_mut(), other);
        self.rule_set_groups_.borrow_mut().clear();
        let sheets = self.active_style_sheets_.borrow().clone();
        for sheet in &sheets {
            let rules = sheet.rule_set.as_ref().expect("active rule set");
            AddRuleSetToRuleSetGroupList(rules.clone(), &mut self.rule_set_groups_.borrow_mut());
            self.AddImplicitScopeTriggers(&sheet.style_sheet, rules);
        }
        self.RebuildCascadeLayerMap(&sheets);
    }
}

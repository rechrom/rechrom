/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
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
// cpp: third_party/blink/renderer/core/css/rule_set.h
// cpp: third_party/blink/renderer/core/css/rule_set.cc
// Maps RuleData, RuleMap storage/compaction and filtered transfer, selector
// bucketing/AddRule, RuleSet storage/intervals, recursive sheet/mixin ingestion,
// filtered diffs, media-result/route queries, substring matcher construction,
// and whole-RuleSet compaction. No production runtime bodies remain pending.
// Source ledger (effective = nonblank/noncomment/nonpreprocessor source lines;
// braces count, GC/layout/access-label/platform boilerplate excluded):
// h: 863 physical; 432 effective; 431 mapped; 1 source-only declaration omitted.
// cc: 1948 physical; 1406 effective; 1354 mapped; 52 debug-only lines omitted.
// Prior batch: h 359 mapped, cc 823 mapped. This batch adds h 62 effective
// lines at 239-243,353-357,360-367,373-384,390-398,409-417,657-672,699-702;
// h:361 (DISALLOW_NEW) and 363 (access label) are excluded.
// This batch adds cc 452 effective lines at 984-1292,1306-1477,1598-1657.
// cc:1311 (trace),1312 (DCHECK),1377 (debug all_rules),1451 (debug
// allow_unsorted) are excluded, not mapped. Previous debug-only omissions
// include cc:594,744-745. ApplyingMixin GC Trace at h:368-372 is omitted.
// The entire source pair is accounted for by mapped ranges/marked bodies,
// the boilerplate exclusions above, and these exact final ranges:
// Final batch maps h:532-536,706-710 (10 effective lines) and cc:1698-1794,
// 1800-1801 (79 effective lines). h:704 declares SortKeyframesRulesIfNeeded,
// which has no definition or call in the source pair; omitted, not mapped.
// Debug-only omitted, not mapped: cc:1796-1798 (3 effective lines),1809-1862
// (49 effective lines). Their preprocessor guards are boilerplate exclusions.
// Production runtime pending: h 0; cc 0.
// External import/navigation/clone/container/scope/property/binding/inspector
// bodies are mandatory typed operations; their source files are not claimed.

use crate::cascade_layer::CascadeLayer;
use crate::cascade_layered::CascadeLayered;
use crate::css_selector::{
    CSSSelector, CSSSelectorComplex, GetPseudoId, LinkMatchMask, MatchType, PseudoType,
    QualifiedName, RelationType,
};
use crate::css_selector_list::CSSSelectorList;
use crate::invalidation::selector_pre_match::SelectorPreMatch;
use crate::resolver::media_query_result::{
    MediaQueryResultFlags, MediaQuerySet, MediaQuerySetResult,
};
use crate::style_rule::{
    StyleRule, StyleRuleBase, StyleRuleDependencies, StyleRuleFontFace, StyleRuleFunction,
    StyleRulePage, StyleRuleProperty,
};
use crate::valid_property_filter::ValidPropertyFilter;
use foundation::{AtomicString, String};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::collections::HashMap;
use std::rc::Rc;

// cpp: rule_set.h:63-69
pub type AddRuleFlags = u32;
pub const kRuleHasNoSpecialState: AddRuleFlags = 0;
pub const kRuleIsVisitedDependent: AddRuleFlags = 1;
pub const kRuleIsStartingStyle: AddRuleFlags = 2;

// cpp: rule_set.h:219-234. The union's two interpretations are selected by
// RuleMap::compacted; keeping one integer preserves the same state transition.
#[derive(Clone, Copy, Debug, Default)]
pub struct Extent {
    pub index: u32,
    pub length: usize,
}

// Required dependency on core/css/robin_hood_map.h. No replacement hashing
// algorithm is supplied here: Insert must retain its fallible native contract.
pub trait RuleBucketTable: Sized {
    fn New(initial_capacity: usize) -> Self;
    fn Find(&self, key: &AtomicString) -> Option<&Extent>;
    fn FindMut(&mut self, key: &AtomicString) -> Option<&mut Extent>;
    fn Insert(&mut self, key: &AtomicString) -> Option<&mut Extent>;
    fn Entries(&self) -> Vec<(AtomicString, Extent)>;
}

// These are dependencies owned by selector/filter/evaluator/invalidation/DOM,
// not alternate StyleRule or selector entities. All operations are required.
pub trait RuleSetStyleScope: Sized {
    fn From(&self) -> Option<&CSSSelectorList>;
    fn To(&self) -> Option<&CSSSelectorList>;
    fn Parent(&self) -> Option<&Self>;
}

// Owned UTF-8 pattern/id inputs for the external base::SubstringSetMatcher.
// Matcher allocation, Build failure and matching remain required dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSetSubstringPattern {
    pub pattern: std::string::String,
    pub id: i32,
}

pub trait RuleSetBackend:
    StyleRuleDependencies<
    SelectorList = CSSSelectorList,
    StyleScope: RuleSetStyleScope,
    MediaQuerySet: MediaQuerySet + 'static,
>
{
    type BucketTable: RuleBucketTable;
    type RuleFeatureSet;
    type MediaQueryEvaluator;
    type NavigationState: PartialEq;
    type Document;
    type SubstringSetMatcher;
    fn CueShadowPseudoId() -> AtomicString;
    fn StringForUAShadowPseudoId(pseudo_id: PseudoId) -> AtomicString;
    fn SelectorIsEasy(selector: CSSSelectorComplex<'_>) -> bool;
    fn CollectIdentifierHashes(
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::StyleScope>,
        backing: &mut Vec<u16>,
        subject_filter: &mut u32,
    );
    // inspector/invalidation_set_to_selector_map.h: SelectorScope lifecycle.
    fn BeginSelector(rule: &Rc<StyleRule<Self>>, selector_index: u32);
    fn EndSelector();
    fn CollectFeaturesFromSelector(
        features: &mut Self::RuleFeatureSet,
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::StyleScope>,
    ) -> SelectorPreMatch;
    fn NewFeatures() -> Self::RuleFeatureSet;
    fn MutableMediaQueryResultFlags(
        features: &mut Self::RuleFeatureSet,
    ) -> &mut MediaQueryResultFlags;
    fn EvalMedia(
        evaluator: &Self::MediaQueryEvaluator,
        queries: &Self::MediaQuerySet,
        flags: &mut MediaQueryResultFlags,
    ) -> bool;
    fn DidResultsChange(
        evaluator: &Self::MediaQueryEvaluator,
        results: &[MediaQuerySetResult],
    ) -> bool;
    fn NavigationStateForDocument(
        document: Option<&Self::Document>,
    ) -> Option<&Self::NavigationState>;
    fn NewSubstringSetMatcher() -> Self::SubstringSetMatcher;
    fn BuildSubstringSetMatcher(
        matcher: &mut Self::SubstringSetMatcher,
        patterns: &[RuleSetSubstringPattern],
    ) -> bool;
    fn SubstringAnyMatch(matcher: &Self::SubstringSetMatcher, value: &[u8]) -> bool;
}

// A GC base-pointer downcast retains its allocation. These owning projections
// do the same without copying the inline enum payload into a fresh Rc.
pub trait TypedRuleKind<B: RuleSetBackend> {
    type Target;
    fn Get(rule: &StyleRuleBase<B>) -> Option<&Self::Target>;
}
pub struct TypedRuleRef<B: RuleSetBackend, K: TypedRuleKind<B>> {
    base: Rc<StyleRuleBase<B>>,
    kind: std::marker::PhantomData<K>,
}
impl<B: RuleSetBackend, K: TypedRuleKind<B>> TypedRuleRef<B, K> {
    pub fn new(base: Rc<StyleRuleBase<B>>) -> Self {
        assert!(
            K::Get(&base).is_some(),
            "typed rule projection requires its variant"
        );
        Self {
            base,
            kind: std::marker::PhantomData,
        }
    }
    pub fn Base(&self) -> &Rc<StyleRuleBase<B>> {
        &self.base
    }
    pub fn PtrEq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.base, &other.base)
    }
}
impl<B: RuleSetBackend, K: TypedRuleKind<B>> Clone for TypedRuleRef<B, K> {
    fn clone(&self) -> Self {
        Self::new(self.base.clone())
    }
}
impl<B: RuleSetBackend, K: TypedRuleKind<B>> std::ops::Deref for TypedRuleRef<B, K> {
    type Target = K::Target;
    fn deref(&self) -> &Self::Target {
        K::Get(&self.base).unwrap()
    }
}
macro_rules! inline_rule_kind {
    ($kind:ident, $variant:ident, $target:ident) => {
        pub struct $kind;
        impl<B: RuleSetBackend> TypedRuleKind<B> for $kind {
            type Target = $target<B>;
            fn Get(rule: &StyleRuleBase<B>) -> Option<&Self::Target> {
                if let StyleRuleBase::$variant(rule) = rule {
                    Some(rule)
                } else {
                    None
                }
            }
        }
    };
}
inline_rule_kind!(PageRuleKind, Page, StyleRulePage);
inline_rule_kind!(FontFaceRuleKind, FontFace, StyleRuleFontFace);
inline_rule_kind!(PropertyRuleKind, Property, StyleRuleProperty);
inline_rule_kind!(FunctionRuleKind, Function, StyleRuleFunction);

// Read-only sheets and mixin maps stay independent of the sheet's loader,
// parser and client backend. Snapshots only clone handles, never rule payloads.
pub trait RuleSetSheetView<B: RuleSetBackend> {
    fn PreImportLayerStatementRules(&self) -> Vec<Rc<StyleRuleBase<B>>>;
    fn ImportRules(&self) -> Vec<Rc<StyleRuleBase<B>>>;
    fn ChildRules(&self) -> Vec<Rc<StyleRuleBase<B>>>;
}
pub trait RuleSetMixinMapView<B: RuleSetBackend> {
    fn Mixins(&self) -> &HashMap<AtomicString, Rc<StyleRuleBase<B>>>;
    fn MapIdentifier(&self) -> Option<u64>;
    fn MediaQueryResultFlags(&self) -> &MediaQueryResultFlags;
    fn MediaQuerySetResults(&self) -> &[MediaQuerySetResult];
}
impl<B: crate::style_sheet_contents::StyleSheetContentsBackend> RuleSetSheetView<B>
    for crate::style_sheet_contents::StyleSheetContents<B>
{
    fn PreImportLayerStatementRules(&self) -> Vec<Rc<StyleRuleBase<B>>> {
        self.PreImportLayerStatementRules().to_vec()
    }
    fn ImportRules(&self) -> Vec<Rc<StyleRuleBase<B>>> {
        self.ImportRules().to_vec()
    }
    fn ChildRules(&self) -> Vec<Rc<StyleRuleBase<B>>> {
        self.ChildRules().to_vec()
    }
}
impl<B: crate::style_sheet_contents::StyleSheetContentsBackend> RuleSetMixinMapView<B>
    for crate::style_sheet_contents::MixinMap<B>
{
    fn Mixins(&self) -> &HashMap<AtomicString, Rc<StyleRuleBase<B>>> {
        &self.mixins
    }
    fn MapIdentifier(&self) -> Option<u64> {
        self.map_identifier
    }
    fn MediaQueryResultFlags(&self) -> &MediaQueryResultFlags {
        &self.media_query_result_flags
    }
    fn MediaQuerySetResults(&self) -> &[MediaQuerySetResult] {
        &self.media_query_set_results
    }
}

pub struct RuleSetMixinBinding<B: RuleSetBackend> {
    pub argument: Option<Rc<B::CSSVariableData>>,
    pub default_value: Option<Rc<B::CSSVariableData>>,
    pub syntax: B::CSSSyntaxDefinition,
}
pub struct RuleSetCQDependentValue<B: RuleSetBackend> {
    pub data: Rc<B::CSSVariableData>,
    pub container_queries: Rc<B::ContainerQuerySet>,
}
pub type RuleSetMixinLocals<B> = HashMap<String, Rc<<B as StyleRuleDependencies>::CSSVariableData>>;
pub type RuleSetConditionalLocals<B> = HashMap<String, Vec<RuleSetCQDependentValue<B>>>;

// Mandatory operations owned by actual import/navigation/clone/container/
// scope/property/binding/inspector classes. No default success or synthetic
// rule/sheet is supplied. Implement CloneRule by StyleRuleBase::Clone using
// the real StyleRuleCloneDependencies assembled by the caller.
pub trait RuleSetIngestionBackend: RuleSetBackend {
    fn CloneRule(
        rule: &StyleRuleBase<Self>,
        parent: Option<Rc<StyleRule<Self>>>,
        bindings: Option<Rc<Self::MixinParameterBindings>>,
    ) -> Rc<StyleRuleBase<Self>>;
    fn MediaRuleQueries(
        rule: &crate::style_rule::StyleRuleMedia<Self>,
    ) -> Option<Rc<Self::MediaQuerySet>>;
    fn ContainerRuleQueries(
        rule: &crate::style_rule::StyleRuleContainer<Self>,
    ) -> Rc<Self::ContainerQuerySet>;
    fn ScopeRuleScope(rule: &crate::style_rule::StyleRuleScope<Self>) -> Rc<Self::StyleScope>;
    fn CopyContainerQueriesWithParent(
        queries: &Self::ContainerQuerySet,
        parent: Rc<Self::ContainerQuerySet>,
    ) -> Rc<Self::ContainerQuerySet>;
    fn CopyStyleScopeWithParent(
        scope: &Self::StyleScope,
        parent: Rc<Self::StyleScope>,
    ) -> Rc<Self::StyleScope>;
    fn NestedDeclarationsInnerStyleRule(
        rule: &Self::StyleRuleNestedDeclarations,
    ) -> Rc<StyleRule<Self>>;
    fn FunctionDeclarationValues(
        rule: &Self::StyleRuleFunctionDeclarations,
    ) -> Vec<(String, Rc<Self::CSSVariableData>)>;
    fn NewMixinParameterBindings(
        bindings: HashMap<String, RuleSetMixinBinding<Self>>,
        locals: RuleSetMixinLocals<Self>,
        conditional_locals: RuleSetConditionalLocals<Self>,
        parent: Option<Rc<Self::MixinParameterBindings>>,
    ) -> Rc<Self::MixinParameterBindings>;
    fn IngestionDocument(medium: &Self::MediaQueryEvaluator) -> Option<&Self::Document>;
    fn CreateRouteIfNeeded(rule: &Self::StyleRuleLocation, document: Option<&Self::Document>);
    fn EvaluateNavigationQuery(
        query: &Self::NavigationQuery,
        document: Option<&Self::Document>,
    ) -> bool;
    fn IngestionImportIsSupported(rule: &Self::StyleRuleImport) -> bool;
    fn IngestionImportMediaQueries(rule: &Self::StyleRuleImport)
        -> Option<Rc<Self::MediaQuerySet>>;
    fn ImportIsLayered(rule: &Self::StyleRuleImport) -> bool;
    fn ImportLayerName(rule: &Self::StyleRuleImport) -> &[AtomicString];
    fn ImportScope(rule: &Self::StyleRuleImport) -> Option<Rc<Self::StyleScope>>;
    fn IngestionImportStyleSheet(
        rule: &Self::StyleRuleImport,
    ) -> Option<Rc<dyn RuleSetSheetView<Self>>>;
    fn CopyNavigationState(state: &Self::NavigationState) -> Rc<Self::NavigationState>;
    fn BeginStyleSheetContents(sheet: &dyn RuleSetSheetView<Self>);
    fn EndStyleSheetContents();
}

// cpp: rule_set.h:360-367,373-374. Rc identity is the cycle key, not mixin content.
pub struct ApplyingMixin<B: RuleSetBackend> {
    pub mixin: Rc<StyleRuleBase<B>>,
    pub invoking_apply_rule: Rc<StyleRuleBase<B>>,
    pub mixin_parameter_bindings: Rc<B::MixinParameterBindings>,
}
pub type ApplyMixinsStack<B> = Vec<ApplyingMixin<B>>;
struct StyleSheetContentsScope<B: RuleSetIngestionBackend>(std::marker::PhantomData<B>);
impl<B: RuleSetIngestionBackend> Drop for StyleSheetContentsScope<B> {
    fn drop(&mut self) {
        B::EndStyleSheetContents();
    }
}

// Runtime diagnostic tracing scope is owned by the inspector dependency.
// The guard keeps EndSelector paired on early returns and unwinding.
struct SelectorScope<B: RuleSetBackend>(std::marker::PhantomData<B>);
impl<B: RuleSetBackend> SelectorScope<B> {
    fn new(rule: &Rc<StyleRule<B>>, selector_index: u32) -> Self {
        B::BeginSelector(rule, selector_index);
        Self(std::marker::PhantomData)
    }
}
impl<B: RuleSetBackend> Drop for SelectorScope<B> {
    fn drop(&mut self) {
        B::EndSelector();
    }
}

// cpp: rule_set.cc:87-117
fn DetermineValidPropertyFilter<B: RuleSetBackend>(
    selector: CSSSelectorComplex<'_>,
) -> ValidPropertyFilter {
    for current in selector.SimpleSelectors() {
        if current.Match() == MatchType::kPseudoElement
            && *current.Value() == B::CueShadowPseudoId()
        {
            return ValidPropertyFilter::kCue;
        }
        let result = match current.GetPseudoType() {
            PseudoType::kPseudoCue => Some(ValidPropertyFilter::kCue),
            PseudoType::kPseudoFirstLetter => Some(ValidPropertyFilter::kFirstLetter),
            PseudoType::kPseudoFirstLine => Some(ValidPropertyFilter::kFirstLine),
            PseudoType::kPseudoMarker => Some(ValidPropertyFilter::kMarker),
            PseudoType::kPseudoSelection
            | PseudoType::kPseudoTargetText
            | PseudoType::kPseudoGrammarError
            | PseudoType::kPseudoSpellingError
            | PseudoType::kPseudoHighlight
            | PseudoType::kPseudoSearchText => Some(ValidPropertyFilter::kHighlight),
            _ => None,
        };
        if let Some(result) = result {
            return result;
        }
    }
    ValidPropertyFilter::kNoFilter
}
// cpp: rule_set.cc:119-143
fn SelectorListHasLinkOrVisited(list: Option<&CSSSelectorList>) -> bool {
    list.is_some_and(|list| {
        list.ComplexSelectors()
            .any(|selector| selector.HasLinkOrVisited())
    })
}
fn DetermineLinkMatchType<B: RuleSetBackend>(
    flags: AddRuleFlags,
    selector: CSSSelectorComplex<'_>,
    scope: Option<&B::StyleScope>,
) -> u32 {
    let scoped = scope.is_some_and(|scope| {
        SelectorListHasLinkOrVisited(scope.From()) || SelectorListHasLinkOrVisited(scope.To())
    });
    if selector.HasLinkOrVisited() || scoped {
        if flags & kRuleIsVisitedDependent != 0 {
            LinkMatchMask::kMatchVisited as u32
        } else {
            LinkMatchMask::kMatchLink as u32
        }
    } else {
        LinkMatchMask::kMatchAll as u32
    }
}

// cpp: rule_set.h:81-177; rule_set.cc:145-233
pub struct RuleData<B: RuleSetBackend> {
    rule: Rc<StyleRule<B>>,
    selector_index: u32,
    position: u32,
    specificity: u32,
    link_match_type: u32,
    valid_property_filter: ValidPropertyFilter,
    is_entirely_covered_by_bucketing: bool,
    is_easy: bool,
    is_starting_style: bool,
    bloom_hash_size: u8,
    bloom_hash_pos: u32,
    subject_filter: u32,
}
impl<B: RuleSetBackend> Clone for RuleData<B> {
    fn clone(&self) -> Self {
        Self {
            rule: self.rule.clone(),
            selector_index: self.selector_index,
            position: self.position,
            specificity: self.specificity,
            link_match_type: self.link_match_type,
            valid_property_filter: self.valid_property_filter,
            is_entirely_covered_by_bucketing: self.is_entirely_covered_by_bucketing,
            is_easy: self.is_easy,
            is_starting_style: self.is_starting_style,
            bloom_hash_size: self.bloom_hash_size,
            bloom_hash_pos: self.bloom_hash_pos,
            subject_filter: self.subject_filter,
        }
    }
}
impl<B: RuleSetBackend> RuleData<B> {
    pub const kSelectorIndexBits: usize = 13;
    pub const kPositionBits: usize = 18;
    pub fn new(
        rule: Rc<StyleRule<B>>,
        selector_index: u32,
        position: u32,
        scope: Option<&B::StyleScope>,
        flags: AddRuleFlags,
        backing: &mut Vec<u16>,
    ) -> Self {
        let selector_index = selector_index & ((1 << Self::kSelectorIndexBits) - 1);
        let selector = rule.Selectors().ComplexAt(selector_index as usize);
        let specificity = selector.Specificity() & 0xffffff;
        let link_match_type = DetermineLinkMatchType::<B>(flags, selector, scope);
        let valid_property_filter = DetermineValidPropertyFilter::<B>(selector);
        let mut result = Self {
            rule,
            selector_index,
            position: position & ((1 << Self::kPositionBits) - 1),
            specificity,
            link_match_type,
            valid_property_filter,
            is_entirely_covered_by_bucketing: false,
            is_easy: false,
            is_starting_style: flags & kRuleIsStartingStyle != 0,
            bloom_hash_size: 0,
            bloom_hash_pos: 0,
            subject_filter: 0,
        };
        result.ComputeBloomFilterHashes(scope, backing);
        result
    }
    pub fn GetPosition(&self) -> u32 {
        self.position
    }
    pub fn Rule(&self) -> &Rc<StyleRule<B>> {
        &self.rule
    }
    pub fn Selector(&self) -> CSSSelectorComplex<'_> {
        self.rule
            .Selectors()
            .ComplexAt(self.selector_index as usize)
    }
    // cpp: rule_set.h:104-106. The cursor borrows the actual shared selectors;
    // their coverage bit has interior mutation, just like the GC-owned source.
    pub fn MutableSelector(&self) -> CSSSelectorComplex<'_> {
        self.Selector()
    }
    pub fn SelectorIndex(&self) -> u32 {
        self.selector_index
    }
    pub fn IsEntirelyCoveredByBucketing(&self) -> bool {
        self.is_entirely_covered_by_bucketing
    }
    pub fn SelectorIsEasy(&self) -> bool {
        self.is_easy
    }
    pub fn IsStartingStyle(&self) -> bool {
        self.is_starting_style
    }
    pub fn Specificity(&self) -> u32 {
        self.specificity
    }
    pub fn LinkMatchType(&self) -> u32 {
        self.link_match_type
    }
    pub fn GetValidPropertyFilter(&self) -> ValidPropertyFilter {
        self.valid_property_filter
    }
    pub fn RejectElement(&self, element_filter: u32) -> bool {
        element_filter & self.subject_filter != self.subject_filter
    }
    pub fn DescendantSelectorIdentifierHashes<'a>(&self, backing: &'a [u16]) -> &'a [u16] {
        let start = self.bloom_hash_pos as usize;
        &backing[start..start + self.bloom_hash_size as usize]
    }
    pub fn ComputeEntirelyCoveredByBucketing(&mut self) {
        self.is_easy = B::SelectorIsEasy(self.Selector());
        self.is_entirely_covered_by_bucketing = self
            .Selector()
            .SimpleSelectors()
            .all(|selector| selector.IsCoveredByBucketing());
    }
    pub fn ResetEntirelyCoveredByBucketing(&mut self) {
        UnmarkAsCoveredByBucketing(self.Selector());
        self.is_entirely_covered_by_bucketing = false;
    }
    pub fn ComputeBloomFilterHashes(
        &mut self,
        scope: Option<&B::StyleScope>,
        backing: &mut Vec<u16>,
    ) {
        if backing.len() >= 16777216 {
            return;
        }
        self.bloom_hash_pos = backing.len() as u32;
        let mut subject_filter = self.subject_filter;
        B::CollectIdentifierHashes(self.Selector(), scope, backing, &mut subject_filter);
        self.subject_filter = subject_filter;
        self.bloom_hash_size = (backing.len() - self.bloom_hash_pos as usize).min(255) as u8;
        self.bloom_hash_pos = reuse_bloom_tail(backing, self.bloom_hash_pos, self.bloom_hash_size);
    }
    pub fn MovedToDifferentRuleSet(&mut self, old: &[u16], new: &mut Vec<u16>, position: u32) {
        let new_pos = new.len();
        new.extend_from_slice(self.DescendantSelectorIdentifierHashes(old));
        self.bloom_hash_pos = new_pos as u32 & 0xffffff;
        self.position = position & ((1 << Self::kPositionBits) - 1);
    }
}
// cpp: rule_set.cc:213-223; kept separate so the source's truncation/reuse
// algorithm can be checked independently of incomplete selector dependencies.
fn reuse_bloom_tail(backing: &mut Vec<u16>, position: u32, size: u8) -> u32 {
    let pos = position as usize;
    let len = size as usize;
    if len > 0 && pos >= len && backing[pos - len..pos] == backing[pos..pos + len] {
        backing.truncate(pos);
        position - u32::from(size)
    } else {
        position
    }
}
// cpp: rule_set.cc:274-574
fn ShouldStopExtractingAtPseudoElement(pseudo_type: PseudoType) -> bool {
    use PseudoType::*;
    match pseudo_type {
        kPseudoCheckMark
        | kPseudoPickerIcon
        | kPseudoExpandIcon
        | kPseudoFirstLetter
        | kPseudoScrollButton
        | kPseudoScrollMarker
        | kPseudoAfter
        | kPseudoBefore
        | kPseudoInterestButton
        | kPseudoBackdrop
        | kPseudoMarker
        | kPseudoColumn
        | kPseudoViewTransition
        | kPseudoViewTransitionGroup
        | kPseudoViewTransitionGroupChildren
        | kPseudoViewTransitionImagePair
        | kPseudoViewTransitionNew
        | kPseudoViewTransitionOld
        | kPseudoScrollMarkerGroup
        | kPseudoOverscrollAreaParent
        | kPseudoOverscrollBackdrop
        | kPseudoSkeleton => true,
        kPseudoCue
        | kPseudoFirstLine
        | kPseudoSelection
        | kPseudoScrollbar
        | kPseudoScrollbarButton
        | kPseudoScrollbarCorner
        | kPseudoScrollbarThumb
        | kPseudoScrollbarTrack
        | kPseudoScrollbarTrackPiece
        | kPseudoSlotted
        | kPseudoPart
        | kPseudoResizer
        | kPseudoSearchText
        | kPseudoTargetText
        | kPseudoHighlight
        | kPseudoSpellingError
        | kPseudoGrammarError
        | kPseudoPlaceholder
        | kPseudoFileSelectorButton
        | kPseudoDetailsContent
        | kPseudoPermissionIcon
        | kPseudoPicker
        | kPseudoSelectListbox
        | kPseudoWebKitCustomElement
        | kPseudoBlinkInternalElement => false,
        _ => panic!("missing pseudo-element bucketing classification"),
    }
}
struct BucketingValues {
    id: AtomicString,
    class_name: AtomicString,
    attr_name: AtomicString,
    attr_value: AtomicString,
    is_exact_attr: bool,
    custom_pseudo_element_name: AtomicString,
    tag_name: AtomicString,
    part_name: AtomicString,
    ua_shadow_pseudo: AtomicString,
    pseudo_type: PseudoType,
    has_slotted: bool,
}
impl Default for BucketingValues {
    fn default() -> Self {
        Self {
            id: AtomicString::default(),
            class_name: AtomicString::default(),
            attr_name: AtomicString::default(),
            attr_value: AtomicString::default(),
            is_exact_attr: false,
            custom_pseudo_element_name: AtomicString::default(),
            tag_name: AtomicString::default(),
            part_name: AtomicString::default(),
            ua_shadow_pseudo: AtomicString::default(),
            pseudo_type: PseudoType::kPseudoUnknown,
            has_slotted: false,
        }
    }
}
fn ExtractBucketingValues<S: RuleSetStyleScope>(
    selector: &CSSSelector,
    scope: Option<&S>,
    values: &mut BucketingValues,
    ua_shadow_id: fn(PseudoId) -> AtomicString,
) -> bool {
    use MatchType::*;
    use PseudoType::*;
    match selector.Match() {
        kId => values.id = selector.Value().clone(),
        kClass => values.class_name = selector.Value().clone(),
        kTag => values.tag_name = selector.TagQName().LocalName().clone(),
        kPseudoElement | kPseudoClass | kPagePseudoClass => {
            if selector.Match() == kPseudoElement
                && ShouldStopExtractingAtPseudoElement(selector.GetPseudoType())
            {
                return false;
            }
            match selector.GetPseudoType() {
                kPseudoFocus
                | kPseudoCue
                | kPseudoLink
                | kPseudoVisited
                | kPseudoWebkitAnyLink
                | kPseudoAnyLink
                | kPseudoFocusVisible
                | kPseudoHost
                | kPseudoHostContext
                | kPseudoSlotted
                | kPseudoRoot
                | kPseudoActiveViewTransition
                | kPseudoUnbounded => {
                    values.pseudo_type = selector.GetPseudoType();
                    values.has_slotted |= values.pseudo_type == kPseudoSlotted;
                }
                kPseudoPicker if *selector.Argument() != AtomicString::from_str("select") => {}
                kPseudoPicker
                | kPseudoPlaceholder
                | kPseudoDetailsContent
                | kPseudoPermissionIcon
                | kPseudoFileSelectorButton
                | kPseudoSelectListbox
                | kPseudoScrollbarButton
                | kPseudoScrollbarCorner
                | kPseudoScrollbarThumb
                | kPseudoScrollbarTrack
                | kPseudoScrollbarTrackPiece => {
                    if values.pseudo_type == kPseudoUnknown {
                        values.pseudo_type = selector.GetPseudoType();
                        values.ua_shadow_pseudo = ua_shadow_id(GetPseudoId(values.pseudo_type));
                    }
                }
                kPseudoWebKitCustomElement | kPseudoBlinkInternalElement => {
                    values.custom_pseudo_element_name = selector.Value().clone();
                }
                kPseudoPart => values.part_name = selector.Value().clone(),
                kPseudoIs | kPseudoWhere | kPseudoParent => {
                    if let Some(list) = selector.SelectorListOrParent() {
                        if list.IsSingleComplexSelector() {
                            assert!(ExtractBucketingValues(
                                list.First().unwrap().First(),
                                scope,
                                values,
                                ua_shadow_id
                            ));
                        }
                    }
                }
                kPseudoScope => {
                    if let Some(scope) = scope {
                        if let Some(list) = scope.From() {
                            if list.IsSingleComplexSelector() {
                                assert!(ExtractBucketingValues(
                                    list.First().unwrap().First(),
                                    scope.Parent(),
                                    values,
                                    ua_shadow_id
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        kAttributeSet => {
            values.is_exact_attr = false;
            values.attr_name = selector.Attribute().LocalName().clone();
            values.attr_value = AtomicString::from_str("");
        }
        kAttributeExact | kAttributeHyphen | kAttributeList | kAttributeContain
        | kAttributeBegin | kAttributeEnd => {
            values.is_exact_attr = selector.Match() == kAttributeExact;
            values.attr_name = selector.Attribute().LocalName().clone();
            values.attr_value = selector.Value().clone();
        }
        _ => {}
    }
    true
}
fn ExtractBestBucketingValues<S: RuleSetStyleScope>(
    component: CSSSelectorComplex<'_>,
    scope: Option<&S>,
    values: &mut BucketingValues,
    ua_shadow_id: fn(PseudoId) -> AtomicString,
) {
    let mut current = Some(component);
    while let Some(selector) = current {
        if !ExtractBucketingValues(selector.First(), scope, values, ua_shadow_id) {
            return;
        }
        match selector.Relation() {
            RelationType::kSubSelector => current = selector.NextSimpleSelector(),
            RelationType::kUAShadow => {
                let originating = selector
                    .NextSimpleSelector()
                    .expect("UA shadow relation needs an originating compound");
                let mut originating_values = BucketingValues::default();
                ExtractBestBucketingValues(
                    originating,
                    scope,
                    &mut originating_values,
                    ua_shadow_id,
                );
                values.has_slotted |= originating_values.has_slotted;
                return;
            }
            _ => return,
        }
    }
}

// cpp: rule_set.h:676-679
#[derive(Clone, Copy, PartialEq, Eq)]
enum BucketCoverage {
    kIgnore,
    kCompute,
}

// cpp: rule_set.cc:550-585 (subject compound only)
fn MarkAsCoveredByBucketing(
    selector: CSSSelectorComplex<'_>,
    should_mark: impl Fn(&CSSSelector) -> bool,
) {
    for current in selector.SimpleSelectors() {
        if should_mark(current) {
            current.SetCoveredByBucketing(true);
        }
        if current.Relation() != RelationType::kSubSelector {
            break;
        }
    }
}
fn UnmarkAsCoveredByBucketing(selector: CSSSelectorComplex<'_>) {
    for current in selector.SimpleSelectors() {
        current.SetCoveredByBucketing(false);
        if current.Relation() != RelationType::kSubSelector {
            break;
        }
    }
}

// cpp: rule_set.h:214-338; rule_set.cc:1487-1595
pub struct RuleMap<B: RuleSetBackend> {
    buckets: Option<B::BucketTable>,
    backing: Vec<RuleData<B>>,
    bucket_number: Vec<u32>,
    num_buckets: u32,
    compacted: bool,
}
impl<B: RuleSetBackend> Default for RuleMap<B> {
    fn default() -> Self {
        Self {
            buckets: None,
            backing: Vec::new(),
            bucket_number: Vec::new(),
            num_buckets: 0,
            compacted: false,
        }
    }
}
impl<B: RuleSetBackend> RuleMap<B> {
    pub fn Add(&mut self, key: &AtomicString, rule: &RuleData<B>) -> bool {
        debug_assert!(!self.compacted);
        let table = self.buckets.get_or_insert_with(|| B::BucketTable::New(8));
        if table.Find(key).is_none() {
            let Some(extent) = table.Insert(key) else {
                return false;
            };
            extent.index = self.num_buckets;
            self.num_buckets += 1;
        }
        let extent = table
            .FindMut(key)
            .expect("successful insertion must be findable");
        let mut copy = rule.clone();
        copy.ComputeEntirelyCoveredByBucketing();
        self.bucket_number.push(extent.index);
        extent.length += 1;
        self.backing.push(copy);
        true
    }
    pub fn Find(&self, key: &AtomicString) -> &[RuleData<B>] {
        let Some(extent) = self.buckets.as_ref().and_then(|table| table.Find(key)) else {
            return &[];
        };
        let start = extent.index as usize;
        &self.backing[start..start + extent.length]
    }
    pub fn IsEmpty(&self) -> bool {
        self.backing.is_empty()
    }
    pub fn IsCompacted(&self) -> bool {
        self.compacted
    }
    pub fn Buckets(&self) -> impl Iterator<Item = (AtomicString, &[RuleData<B>])> {
        self.buckets
            .as_ref()
            .map(|table| table.Entries())
            .unwrap_or_default()
            .into_iter()
            .map(move |(key, extent)| {
                let start = extent.index as usize;
                (key, &self.backing[start..start + extent.length])
            })
    }
    pub fn Compact(&mut self) {
        if self.compacted {
            return;
        }
        if self.backing.is_empty() {
            debug_assert!(self.bucket_number.is_empty());
            self.compacted = true;
            return;
        }
        self.backing.shrink_to_fit();
        let (counts, mut order) =
            counting_sort_positions(&self.bucket_number, self.num_buckets as usize);
        let table = self.buckets.as_mut().expect("nonempty map has buckets");
        for (key, extent) in table.Entries() {
            table.FindMut(&key).unwrap().index = counts[extent.index as usize];
        }
        counting_sort_permute(
            &mut self.backing,
            &mut self.bucket_number,
            &mut order,
            &counts,
        );
        self.bucket_number.clear();
        self.compacted = true;
    }
    pub fn Uncompact(&mut self) {
        self.bucket_number.resize(self.backing.len(), 0);
        self.num_buckets = 0;
        if let Some(table) = self.buckets.as_mut() {
            for (key, extent) in table.Entries() {
                self.bucket_number[extent.index as usize..extent.index as usize + extent.length]
                    .fill(self.num_buckets);
                table.FindMut(&key).unwrap().index = self.num_buckets;
                self.num_buckets += 1;
            }
        }
        self.compacted = false;
    }
}
// cpp: rule_set.cc:1534-1579. These helpers split the exact counting sort into
// prefix/order computation and the same in-place cycles used by Chromium.
fn counting_sort_positions(numbers: &[u32], bucket_count: usize) -> (Vec<u32>, Vec<u32>) {
    let mut counts = vec![0u32; bucket_count];
    let mut order = Vec::with_capacity(numbers.len());
    for &bucket in numbers {
        order.push(counts[bucket as usize]);
        counts[bucket as usize] += 1;
    }
    let mut sum = 0;
    for count in &mut counts {
        debug_assert!(*count > 0);
        let new_sum = sum + *count;
        *count = sum;
        sum = new_sum;
    }
    (counts, order)
}
fn counting_sort_permute<T>(
    backing: &mut [T],
    numbers: &mut [u32],
    order: &mut [u32],
    counts: &[u32],
) {
    let mut index = 0;
    while index < backing.len() {
        let correct = (counts[numbers[index] as usize] + order[index]) as usize;
        if correct == index {
            index += 1;
        } else {
            backing.swap(index, correct);
            numbers.swap(index, correct);
            order.swap(index, correct);
        }
    }
}

// cpp: rule_set.cc:237-264. Explicit field borrows replace the source's
// member access, preserving the same fallback and coverage state changes.
fn AddToMapBucket<B: RuleSetBackend>(
    key: &AtomicString,
    map: &mut RuleMap<B>,
    rule_data: &RuleData<B>,
    universal_rules: &mut Vec<RuleData<B>>,
    need_compaction: &mut bool,
) {
    if map.IsCompacted() {
        map.Uncompact();
    }
    if !map.Add(key, rule_data) {
        let copy = rule_data.clone();
        UnmarkAsCoveredByBucketing(copy.MutableSelector());
        AddToVectorBucket(universal_rules, &copy, need_compaction);
        return;
    }
    *need_compaction = true;
}
fn AddToVectorBucket<B: RuleSetBackend>(
    rules: &mut Vec<RuleData<B>>,
    rule_data: &RuleData<B>,
    need_compaction: &mut bool,
) {
    let mut copy = rule_data.clone();
    copy.ComputeEntirelyCoveredByBucketing();
    rules.push(copy);
    *need_compaction = true;
}

// cpp: rule_set.h:597-606; rule_set.cc:909-920. Rc preserves pointer identity for external payloads;
// CascadeLayer already owns its own stable pointer identity.
pub struct Interval<T> {
    pub value: Option<T>,
    pub start_position: u32,
}
fn add_interval<T>(
    value: Option<T>,
    position: u32,
    intervals: &mut Vec<Interval<T>>,
    same: impl Fn(Option<&T>, Option<&T>) -> bool,
) {
    if same(
        value.as_ref(),
        intervals
            .last()
            .and_then(|interval| interval.value.as_ref()),
    ) {
        return;
    }
    intervals.push(Interval {
        value,
        start_position: position,
    });
}
fn same_rc<T>(left: Option<&Rc<T>>, right: Option<&Rc<T>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        _ => false,
    }
}
fn layer_for_position<'a>(
    intervals: &'a [Interval<CascadeLayer>],
    outer: Option<&'a CascadeLayer>,
    position: u32,
) -> Option<&'a CascadeLayer> {
    if intervals.is_empty() || intervals[0].start_position > position {
        return outer;
    }
    for index in 1..intervals.len() {
        if intervals[index].start_position > position {
            return intervals[index - 1].value.as_ref();
        }
    }
    intervals.last().unwrap().value.as_ref()
}

// cpp: rule_set.h:730-833. Required feature state is constructed by its own
// dependency; all the empty collections/flags below are source initializers.
pub struct RuleSet<B: RuleSetBackend> {
    id_rules: RuleMap<B>,
    class_rules: RuleMap<B>,
    attr_rules: RuleMap<B>,
    tag_rules: RuleMap<B>,
    ua_shadow_pseudo_element_rules: RuleMap<B>,
    input_rules: RuleMap<B>,
    attr_substring_matchers: HashMap<AtomicString, B::SubstringSetMatcher>,
    link_pseudo_class_rules: Vec<RuleData<B>>,
    cue_pseudo_rules: Vec<RuleData<B>>,
    focus_pseudo_class_rules: Vec<RuleData<B>>,
    focus_visible_pseudo_class_rules: Vec<RuleData<B>>,
    scrollbar_rules: Vec<RuleData<B>>,
    universal_rules: Vec<RuleData<B>>,
    shadow_host_rules: Vec<RuleData<B>>,
    part_pseudo_rules: Vec<RuleData<B>>,
    slotted_pseudo_element_rules: Vec<RuleData<B>>,
    active_view_transition_rules: Vec<RuleData<B>>,
    unbounded_pseudo_class_rules: Vec<RuleData<B>>,
    root_element_rules: Vec<RuleData<B>>,
    features: B::RuleFeatureSet,
    page_rules: Vec<CascadeLayered<TypedRuleRef<B, PageRuleKind>>>,
    font_face_rules: Vec<CascadeLayered<TypedRuleRef<B, FontFaceRuleKind>>>,
    font_feature_values_rules: Vec<CascadeLayered<Rc<B::StyleRuleFontFeatureValues>>>,
    view_transition_rules: Vec<CascadeLayered<Rc<B::StyleRuleViewTransition>>>,
    keyframes_rules: Vec<CascadeLayered<Rc<B::StyleRuleKeyframes>>>,
    property_rules: Vec<CascadeLayered<TypedRuleRef<B, PropertyRuleKind>>>,
    counter_style_rules: Vec<CascadeLayered<Rc<B::StyleRuleCounterStyle>>>,
    position_try_rules: Vec<CascadeLayered<Rc<B::StyleRulePositionTry>>>,
    function_rules: Vec<CascadeLayered<TypedRuleRef<B, FunctionRuleKind>>>,
    font_palette_values_rules: Vec<Rc<B::StyleRuleFontPaletteValues>>,
    media_query_set_results: Vec<MediaQuerySetResult>,
    navigation_state: Option<Rc<B::NavigationState>>,
    has_bucket_for_style_attr: bool,
    must_check_universal_bucket_for_shadow_host: bool,
    rule_count: u32,
    need_compaction: bool,
    implicit_outer_layer: Option<CascadeLayer>,
    layer_intervals: Vec<Interval<CascadeLayer>>,
    container_query_intervals: Vec<Interval<Rc<B::ContainerQuerySet>>>,
    scope_intervals: Vec<Interval<Rc<B::StyleScope>>>,
    bloom_hash_backing: Vec<u16>,
    depends_on_mixins: bool,
    based_on_mixin_map_identifier: Option<u64>,
}
impl<B: RuleSetBackend> RuleSet<B> {
    // cpp: rule_set.cc:235; rule_set.h:792-833
    pub fn new() -> Self {
        Self {
            id_rules: RuleMap::default(),
            class_rules: RuleMap::default(),
            attr_rules: RuleMap::default(),
            tag_rules: RuleMap::default(),
            ua_shadow_pseudo_element_rules: RuleMap::default(),
            input_rules: RuleMap::default(),
            attr_substring_matchers: HashMap::new(),
            link_pseudo_class_rules: Vec::new(),
            cue_pseudo_rules: Vec::new(),
            focus_pseudo_class_rules: Vec::new(),
            focus_visible_pseudo_class_rules: Vec::new(),
            scrollbar_rules: Vec::new(),
            universal_rules: Vec::new(),
            shadow_host_rules: Vec::new(),
            part_pseudo_rules: Vec::new(),
            slotted_pseudo_element_rules: Vec::new(),
            active_view_transition_rules: Vec::new(),
            unbounded_pseudo_class_rules: Vec::new(),
            root_element_rules: Vec::new(),
            page_rules: Vec::new(),
            font_face_rules: Vec::new(),
            font_feature_values_rules: Vec::new(),
            view_transition_rules: Vec::new(),
            keyframes_rules: Vec::new(),
            property_rules: Vec::new(),
            counter_style_rules: Vec::new(),
            position_try_rules: Vec::new(),
            function_rules: Vec::new(),
            font_palette_values_rules: Vec::new(),
            media_query_set_results: Vec::new(),
            layer_intervals: Vec::new(),
            container_query_intervals: Vec::new(),
            scope_intervals: Vec::new(),
            bloom_hash_backing: Vec::new(),
            features: B::NewFeatures(),
            navigation_state: None,
            has_bucket_for_style_attr: false,
            must_check_universal_bucket_for_shadow_host: false,
            rule_count: 0,
            need_compaction: false,
            implicit_outer_layer: None,
            depends_on_mixins: false,
            based_on_mixin_map_identifier: None,
        }
    }
    // cpp: rule_set.h:681-689; rule_set.cc:587-890
    fn FindBestBucketAndAdd(
        &mut self,
        component: CSSSelectorComplex<'_>,
        rule_data: &RuleData<B>,
        scope: Option<&B::StyleScope>,
        coverage: BucketCoverage,
    ) {
        use PseudoType::*;
        let mut values = BucketingValues::default();
        ExtractBestBucketingValues(component, scope, &mut values, B::StringForUAShadowPseudoId);
        if values.has_slotted {
            AddToVectorBucket(
                &mut self.slotted_pseudo_element_rules,
                rule_data,
                &mut self.need_compaction,
            );
            return;
        }
        if !values.ua_shadow_pseudo.empty() {
            assert!(values.part_name.empty());
            AddToMapBucket(
                &values.ua_shadow_pseudo,
                &mut self.ua_shadow_pseudo_element_rules,
                rule_data,
                &mut self.universal_rules,
                &mut self.need_compaction,
            );
            return;
        }
        if !values.part_name.empty() {
            assert!(values.ua_shadow_pseudo.empty());
            AddToVectorBucket(
                &mut self.part_pseudo_rules,
                rule_data,
                &mut self.need_compaction,
            );
            return;
        }
        if values.pseudo_type == kPseudoFocus {
            if coverage == BucketCoverage::kCompute {
                MarkAsCoveredByBucketing(component, |s| {
                    s.Match() == MatchType::kPseudoClass && s.GetPseudoType() == kPseudoFocus
                });
            }
            AddToVectorBucket(
                &mut self.focus_pseudo_class_rules,
                rule_data,
                &mut self.need_compaction,
            );
            return;
        }
        if values.pseudo_type == kPseudoFocusVisible {
            if coverage == BucketCoverage::kCompute {
                MarkAsCoveredByBucketing(component, |s| {
                    s.Match() == MatchType::kPseudoClass && s.GetPseudoType() == kPseudoFocusVisible
                });
            }
            AddToVectorBucket(
                &mut self.focus_visible_pseudo_class_rules,
                rule_data,
                &mut self.need_compaction,
            );
            return;
        }
        if matches!(
            values.pseudo_type,
            kPseudoScrollbarButton
                | kPseudoScrollbarCorner
                | kPseudoScrollbarThumb
                | kPseudoScrollbarTrack
                | kPseudoScrollbarTrackPiece
        ) {
            AddToVectorBucket(
                &mut self.scrollbar_rules,
                rule_data,
                &mut self.need_compaction,
            );
            return;
        }
        if values.pseudo_type == kPseudoActiveViewTransition {
            if coverage == BucketCoverage::kCompute {
                MarkAsCoveredByBucketing(component, |s| {
                    s.Match() == MatchType::kPseudoClass
                        && s.GetPseudoType() == kPseudoActiveViewTransition
                });
            }
            AddToVectorBucket(
                &mut self.active_view_transition_rules,
                rule_data,
                &mut self.need_compaction,
            );
            return;
        }
        if values.pseudo_type == kPseudoUnbounded {
            if coverage == BucketCoverage::kCompute {
                MarkAsCoveredByBucketing(component, |s| {
                    s.Match() == MatchType::kPseudoClass && s.GetPseudoType() == kPseudoUnbounded
                });
            }
            AddToVectorBucket(
                &mut self.unbounded_pseudo_class_rules,
                rule_data,
                &mut self.need_compaction,
            );
            return;
        }
        if !values.id.empty() {
            if coverage == BucketCoverage::kCompute {
                MarkAsCoveredByBucketing(component, |s| {
                    s.Match() == MatchType::kId && *s.Value() == values.id
                });
            }
            AddToMapBucket(
                &values.id,
                &mut self.id_rules,
                rule_data,
                &mut self.universal_rules,
                &mut self.need_compaction,
            );
            return;
        }
        if !values.class_name.empty() {
            if coverage == BucketCoverage::kCompute {
                MarkAsCoveredByBucketing(component, |s| {
                    s.Match() == MatchType::kClass && *s.Value() == values.class_name
                });
            }
            AddToMapBucket(
                &values.class_name,
                &mut self.class_rules,
                rule_data,
                &mut self.universal_rules,
                &mut self.need_compaction,
            );
            return;
        }
        if !values.attr_name.empty() {
            if values.tag_name == AtomicString::from_str("input")
                && values.attr_name == AtomicString::from_str("type")
                && values.is_exact_attr
            {
                if coverage == BucketCoverage::kCompute {
                    MarkAsCoveredByBucketing(component, |s| {
                        s.Match() == MatchType::kTag
                            && *s.TagQName().LocalName() == AtomicString::from_str("input")
                            && *s.TagQName().NamespaceURI() == AtomicString::from_str("*")
                    });
                }
                AddToMapBucket(
                    &values.attr_value.ToAsciiLower(),
                    &mut self.input_rules,
                    rule_data,
                    &mut self.universal_rules,
                    &mut self.need_compaction,
                );
                return;
            }
            AddToMapBucket(
                &values.attr_name,
                &mut self.attr_rules,
                rule_data,
                &mut self.universal_rules,
                &mut self.need_compaction,
            );
            if values.attr_name == AtomicString::from_str("style") {
                self.has_bucket_for_style_attr = true;
            }
            return;
        }
        if !values.custom_pseudo_element_name.empty() {
            AddToMapBucket(
                &values.custom_pseudo_element_name,
                &mut self.ua_shadow_pseudo_element_rules,
                rule_data,
                &mut self.universal_rules,
                &mut self.need_compaction,
            );
            return;
        }
        match values.pseudo_type {
            kPseudoCue => {
                AddToVectorBucket(
                    &mut self.cue_pseudo_rules,
                    rule_data,
                    &mut self.need_compaction,
                );
                return;
            }
            kPseudoLink | kPseudoVisited | kPseudoAnyLink | kPseudoWebkitAnyLink => {
                if coverage == BucketCoverage::kCompute {
                    MarkAsCoveredByBucketing(component, |s| {
                        s.Match() == MatchType::kPseudoClass
                            && matches!(s.GetPseudoType(), kPseudoAnyLink | kPseudoWebkitAnyLink)
                    });
                }
                AddToVectorBucket(
                    &mut self.link_pseudo_class_rules,
                    rule_data,
                    &mut self.need_compaction,
                );
                return;
            }
            kPseudoFocus | kPseudoFocusVisible | kPseudoSlotted => unreachable!("handled above"),
            kPseudoHost | kPseudoHostContext => {
                AddToVectorBucket(
                    &mut self.shadow_host_rules,
                    rule_data,
                    &mut self.need_compaction,
                );
                return;
            }
            kPseudoRoot => {
                if coverage == BucketCoverage::kCompute {
                    MarkAsCoveredByBucketing(component, |s| {
                        s.Match() == MatchType::kPseudoClass && s.GetPseudoType() == kPseudoRoot
                    });
                }
                AddToVectorBucket(
                    &mut self.root_element_rules,
                    rule_data,
                    &mut self.need_compaction,
                );
                return;
            }
            _ => {}
        }
        if !values.tag_name.empty() {
            if coverage == BucketCoverage::kCompute {
                MarkAsCoveredByBucketing(component, |s| {
                    s.Match() == MatchType::kTag
                        && *s.TagQName().LocalName() == values.tag_name
                        && *s.TagQName().NamespaceURI() == AtomicString::from_str("*")
                });
            }
            AddToMapBucket(
                &values.tag_name,
                &mut self.tag_rules,
                rule_data,
                &mut self.universal_rules,
                &mut self.need_compaction,
            );
            return;
        }
        if component.IsScopeContaining() || component.IsOrContainsHostPseudoClass() {
            self.must_check_universal_bucket_for_shadow_host = true;
        }
        MarkAsCoveredByBucketing(component, |s| {
            s.Match() == MatchType::kUniversalTag && *s.TagQName() == QualifiedName::AnyQName()
        });
        AddToVectorBucket(
            &mut self.universal_rules,
            rule_data,
            &mut self.need_compaction,
        );
    }
    pub fn AddRule(
        &mut self,
        rule: Rc<StyleRule<B>>,
        selector_index: u32,
        flags: AddRuleFlags,
        container_queries: Option<Rc<B::ContainerQuerySet>>,
        layer: Option<CascadeLayer>,
        scope: Option<Rc<B::StyleScope>>,
    ) {
        if selector_index >= 1 << RuleData::<B>::kSelectorIndexBits
            || self.rule_count >= 1 << RuleData::<B>::kPositionBits
        {
            return;
        }
        let mut data = RuleData::new(
            rule.clone(),
            selector_index,
            self.rule_count,
            scope.as_deref(),
            flags,
            &mut self.bloom_hash_backing,
        );
        self.rule_count += 1;
        {
            let _selector_scope = SelectorScope::<B>::new(&rule, selector_index);
            if B::CollectFeaturesFromSelector(&mut self.features, data.Selector(), scope.as_deref())
                == SelectorPreMatch::kNeverMatches
            {
                return;
            }
        }
        self.FindBestBucketAndAdd(
            data.MutableSelector(),
            &data,
            scope.as_deref(),
            BucketCoverage::kCompute,
        );
        if data.LinkMatchType() == LinkMatchMask::kMatchLink as u32 {
            data.ResetEntirelyCoveredByBucketing();
            let visited = RuleData::new(
                rule,
                data.SelectorIndex(),
                data.GetPosition(),
                scope.as_deref(),
                flags | kRuleIsVisitedDependent,
                &mut self.bloom_hash_backing,
            );
            self.FindBestBucketAndAdd(
                visited.MutableSelector(),
                &visited,
                scope.as_deref(),
                BucketCoverage::kIgnore,
            );
        }
        self.AddRuleToLayerIntervals(layer, data.GetPosition());
        add_interval(
            container_queries,
            data.GetPosition(),
            &mut self.container_query_intervals,
            same_rc,
        );
        add_interval(
            scope,
            data.GetPosition(),
            &mut self.scope_intervals,
            same_rc,
        );
    }
    // cpp: rule_set.h:419-622 (storage accessors, excluding pending compact)
    pub fn Features(&self) -> &B::RuleFeatureSet {
        &self.features
    }
    pub fn IdRules(&self, key: &AtomicString) -> &[RuleData<B>] {
        self.id_rules.Find(key)
    }
    pub fn ClassRules(&self, key: &AtomicString) -> &[RuleData<B>] {
        self.class_rules.Find(key)
    }
    pub fn AttrRules(&self, key: &AtomicString) -> &[RuleData<B>] {
        self.attr_rules.Find(key)
    }
    pub fn TagRules(&self, key: &AtomicString) -> &[RuleData<B>] {
        self.tag_rules.Find(key)
    }
    pub fn InputRules(&self, key: &AtomicString) -> &[RuleData<B>] {
        self.input_rules.Find(key)
    }
    pub fn UAShadowPseudoElementRules(&self, key: &AtomicString) -> &[RuleData<B>] {
        self.ua_shadow_pseudo_element_rules.Find(key)
    }
    pub fn LinkPseudoClassRules(&self) -> &[RuleData<B>] {
        &self.link_pseudo_class_rules
    }
    pub fn CuePseudoRules(&self) -> &[RuleData<B>] {
        &self.cue_pseudo_rules
    }
    pub fn FocusPseudoClassRules(&self) -> &[RuleData<B>] {
        &self.focus_pseudo_class_rules
    }
    pub fn FocusVisiblePseudoClassRules(&self) -> &[RuleData<B>] {
        &self.focus_visible_pseudo_class_rules
    }
    pub fn ScrollbarRules(&self) -> &[RuleData<B>] {
        &self.scrollbar_rules
    }
    pub fn UniversalRules(&self) -> &[RuleData<B>] {
        &self.universal_rules
    }
    pub fn ShadowHostRules(&self) -> &[RuleData<B>] {
        &self.shadow_host_rules
    }
    pub fn PartPseudoRules(&self) -> &[RuleData<B>] {
        &self.part_pseudo_rules
    }
    pub fn SlottedPseudoElementRules(&self) -> &[RuleData<B>] {
        &self.slotted_pseudo_element_rules
    }
    pub fn ActiveViewTransitionRules(&self) -> &[RuleData<B>] {
        &self.active_view_transition_rules
    }
    pub fn UnboundedPseudoClassRules(&self) -> &[RuleData<B>] {
        &self.unbounded_pseudo_class_rules
    }
    pub fn RootElementRules(&self) -> &[RuleData<B>] {
        &self.root_element_rules
    }
    pub fn PageRules(&self) -> &[CascadeLayered<TypedRuleRef<B, PageRuleKind>>] {
        &self.page_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddPageRule(
        &mut self,
        rule: TypedRuleRef<B, PageRuleKind>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.page_rules.push(CascadeLayered::new(rule, layer));
    }
    pub fn FontFaceRules(&self) -> &[CascadeLayered<TypedRuleRef<B, FontFaceRuleKind>>] {
        &self.font_face_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddFontFaceRule(
        &mut self,
        rule: TypedRuleRef<B, FontFaceRuleKind>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.font_face_rules.push(CascadeLayered::new(rule, layer));
    }
    pub fn FontFeatureValuesRules(&self) -> &[CascadeLayered<Rc<B::StyleRuleFontFeatureValues>>] {
        &self.font_feature_values_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddFontFeatureValuesRule(
        &mut self,
        rule: Rc<B::StyleRuleFontFeatureValues>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.font_feature_values_rules
            .push(CascadeLayered::new(rule, layer));
    }
    pub fn ViewTransitionRules(&self) -> &[CascadeLayered<Rc<B::StyleRuleViewTransition>>] {
        &self.view_transition_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddViewTransitionRule(
        &mut self,
        rule: Rc<B::StyleRuleViewTransition>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.view_transition_rules
            .push(CascadeLayered::new(rule, layer));
    }
    pub fn KeyframesRules(&self) -> &[CascadeLayered<Rc<B::StyleRuleKeyframes>>] {
        &self.keyframes_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddKeyframesRule(
        &mut self,
        rule: Rc<B::StyleRuleKeyframes>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.keyframes_rules.push(CascadeLayered::new(rule, layer));
    }
    pub fn PropertyRules(&self) -> &[CascadeLayered<TypedRuleRef<B, PropertyRuleKind>>] {
        &self.property_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddPropertyRule(
        &mut self,
        rule: TypedRuleRef<B, PropertyRuleKind>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.property_rules.push(CascadeLayered::new(rule, layer));
    }
    pub fn CounterStyleRules(&self) -> &[CascadeLayered<Rc<B::StyleRuleCounterStyle>>] {
        &self.counter_style_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddCounterStyleRule(
        &mut self,
        rule: Rc<B::StyleRuleCounterStyle>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.counter_style_rules
            .push(CascadeLayered::new(rule, layer));
    }
    pub fn PositionTryRules(&self) -> &[CascadeLayered<Rc<B::StyleRulePositionTry>>] {
        &self.position_try_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddPositionTryRule(
        &mut self,
        rule: Rc<B::StyleRulePositionTry>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.position_try_rules
            .push(CascadeLayered::new(rule, layer));
    }
    pub fn FunctionRules(&self) -> &[CascadeLayered<TypedRuleRef<B, FunctionRuleKind>>] {
        &self.function_rules
    }
    // cpp: rule_set.cc:922-982 category insertion
    pub fn AddFunctionRule(
        &mut self,
        rule: TypedRuleRef<B, FunctionRuleKind>,
        layer: Option<CascadeLayer>,
    ) {
        self.need_compaction = true;
        self.function_rules.push(CascadeLayered::new(rule, layer));
    }
    pub fn FontPaletteValuesRules(&self) -> &[Rc<B::StyleRuleFontPaletteValues>] {
        &self.font_palette_values_rules
    }
    pub fn AddFontPaletteValuesRule(&mut self, rule: Rc<B::StyleRuleFontPaletteValues>) {
        self.need_compaction = true;
        self.font_palette_values_rules.push(rule);
    }
    pub fn HasAnyAttrRules(&self) -> bool {
        !self.attr_rules.IsEmpty()
    }
    pub fn HasAnyInputRules(&self) -> bool {
        !self.input_rules.IsEmpty()
    }
    pub fn HasSlottedRules(&self) -> bool {
        !self.slotted_pseudo_element_rules.is_empty()
    }
    pub fn HasPartPseudoRules(&self) -> bool {
        !self.part_pseudo_rules.is_empty()
    }
    pub fn HasBucketForStyleAttribute(&self) -> bool {
        self.has_bucket_for_style_attr
    }
    pub fn MustCheckUniversalBucketForShadowHost(&self) -> bool {
        self.must_check_universal_bucket_for_shadow_host
    }
    pub fn HasUAShadowPseudoElementRules(&self) -> bool {
        !self.ua_shadow_pseudo_element_rules.IsEmpty()
    }
    pub fn HasCascadeLayers(&self) -> bool {
        self.implicit_outer_layer.is_some()
    }
    pub fn CascadeLayers(&self) -> &CascadeLayer {
        self.implicit_outer_layer
            .as_ref()
            .expect("cascade layers must exist")
    }
    pub fn RuleCount(&self) -> u32 {
        self.rule_count
    }
    // cpp: rule_set.h:532-536
    pub fn CompactRulesIfNeeded(&mut self) {
        if self.need_compaction {
            self.CompactRules();
        }
    }
    pub fn AssertCompacted(&self) {
        debug_assert!(!self.need_compaction);
    }
    pub fn SingleScope(&self) -> Option<&B::StyleScope> {
        if self.scope_intervals.len() == 1 && self.scope_intervals[0].start_position == 0 {
            self.scope_intervals[0].value.as_deref()
        } else {
            None
        }
    }
    pub fn DependingOnMixins(&self) -> bool {
        self.depends_on_mixins
    }
    pub fn DependingOnOutdatedMixins(&self, current_identifier: Option<u64>) -> bool {
        self.depends_on_mixins && self.based_on_mixin_map_identifier != current_identifier
    }
    pub fn LayerIntervals(&self) -> &[Interval<CascadeLayer>] {
        &self.layer_intervals
    }
    pub fn ContainerQueryIntervals(&self) -> &[Interval<Rc<B::ContainerQuerySet>>] {
        &self.container_query_intervals
    }
    pub fn ScopeIntervals(&self) -> &[Interval<Rc<B::StyleScope>>] {
        &self.scope_intervals
    }
    pub fn BloomHashBacking(&self) -> &[u16] {
        &self.bloom_hash_backing
    }
    // cpp: rule_set.h:716-725; rule_set.cc:1479-1485
    pub fn EnsureImplicitOuterLayer(&mut self) -> CascadeLayer {
        self.implicit_outer_layer
            .get_or_insert_with(CascadeLayer::default)
            .clone()
    }
    pub fn GetOrAddSubLayer(
        &mut self,
        layer: Option<CascadeLayer>,
        name: &[AtomicString],
    ) -> CascadeLayer {
        layer
            .unwrap_or_else(|| self.EnsureImplicitOuterLayer())
            .GetOrAddSubLayer(name)
    }
    // cpp: rule_set.cc:892-920
    pub fn AddRuleToLayerIntervals(&mut self, layer: Option<CascadeLayer>, position: u32) {
        let layer = match layer {
            Some(layer) => layer,
            None if self.layer_intervals.is_empty() => return,
            None => self.EnsureImplicitOuterLayer(),
        };
        add_interval(
            Some(layer),
            position,
            &mut self.layer_intervals,
            |left, right| left == right,
        );
    }
    // cpp: rule_set.cc:1294-1304
    pub fn MatchMediaForAddRules(
        &mut self,
        evaluator: &B::MediaQueryEvaluator,
        queries: Option<Rc<B::MediaQuerySet>>,
    ) -> bool {
        let Some(queries) = queries else {
            return true;
        };
        let result = B::EvalMedia(
            evaluator,
            &queries,
            B::MutableMediaQueryResultFlags(&mut self.features),
        );
        self.media_query_set_results
            .push(MediaQuerySetResult::new(queries, result));
        result
    }
    // cpp: rule_set.cc:1659-1696
    pub fn CanIgnoreEntireList(
        &self,
        list: &[RuleData<B>],
        key: &AtomicString,
        value: &AtomicString,
    ) -> bool {
        let expected = self.attr_rules.Find(key);
        debug_assert_eq!(list.len(), expected.len());
        if !list.is_empty() {
            debug_assert_eq!(list.as_ptr(), expected.as_ptr());
        }
        if list.len() < 50 || value.length() == 0 {
            return false;
        }
        let Some(matcher) = self.attr_substring_matchers.get(key) else {
            return false;
        };
        !B::SubstringAnyMatch(matcher, value.ToAsciiLower().Utf8().as_bytes())
    }
    // cpp: rule_set.h:707-710; rule_set.cc:1698-1760
    fn CreateSubstringMatchers(
        attr_map: &RuleMap<B>,
        scope_intervals: &[Interval<Rc<B::StyleScope>>],
        substring_matcher_map: &mut HashMap<AtomicString, B::SubstringSetMatcher>,
    ) {
        for (attr, ruleset) in attr_map.Buckets() {
            if ruleset.len() < 50 {
                continue;
            }
            let mut patterns = Vec::<RuleSetSubstringPattern>::new();
            let mut rule_index = 0;
            // Each compacted bucket preserves insertion/position order. Seek
            // forward through the actual scope intervals once for this bucket.
            let mut scope_cursor = 0;
            for rule in ruleset {
                while scope_cursor < scope_intervals.len()
                    && scope_intervals[scope_cursor].start_position <= rule.GetPosition()
                {
                    scope_cursor += 1;
                }
                let scope = scope_cursor
                    .checked_sub(1)
                    .and_then(|index| scope_intervals[index].value.as_deref());
                let mut values = BucketingValues::default();
                ExtractBestBucketingValues(
                    rule.Selector(),
                    scope,
                    &mut values,
                    B::StringForUAShadowPseudoId,
                );
                debug_assert_ne!(values.attr_name.length(), 0);
                if values.attr_value.length() == 0 {
                    if values.is_exact_attr {
                        continue;
                    }
                    patterns.clear();
                    break;
                }
                let pattern = values.attr_value.ToAsciiLower().Utf8();
                if !patterns.iter().any(|existing| existing.pattern == pattern) {
                    patterns.push(RuleSetSubstringPattern {
                        pattern,
                        id: rule_index,
                    });
                }
                rule_index += 1;
            }
            if patterns.is_empty() {
                continue;
            }
            let mut matcher = B::NewSubstringSetMatcher();
            if B::BuildSubstringSetMatcher(&mut matcher, &patterns) {
                // WTF HashMap::insert retains an existing entry.
                substring_matcher_map.entry(attr).or_insert(matcher);
            }
        }
    }
    // cpp: rule_set.h:706; rule_set.cc:1762-1794,1800-1801.
    // EXPENSIVE_DCHECKS sort validation at 1796-1798 is explicitly omitted.
    fn CompactRules(&mut self) {
        debug_assert!(self.need_compaction);
        self.id_rules.Compact();
        self.class_rules.Compact();
        self.attr_rules.Compact();
        Self::CreateSubstringMatchers(
            &self.attr_rules,
            &self.scope_intervals,
            &mut self.attr_substring_matchers,
        );
        self.tag_rules.Compact();
        self.input_rules.Compact();
        self.ua_shadow_pseudo_element_rules.Compact();
        self.link_pseudo_class_rules.shrink_to_fit();
        self.cue_pseudo_rules.shrink_to_fit();
        self.focus_pseudo_class_rules.shrink_to_fit();
        self.focus_visible_pseudo_class_rules.shrink_to_fit();
        self.scrollbar_rules.shrink_to_fit();
        self.universal_rules.shrink_to_fit();
        self.shadow_host_rules.shrink_to_fit();
        self.part_pseudo_rules.shrink_to_fit();
        self.slotted_pseudo_element_rules.shrink_to_fit();
        self.active_view_transition_rules.shrink_to_fit();
        self.unbounded_pseudo_class_rules.shrink_to_fit();
        self.page_rules.shrink_to_fit();
        self.font_face_rules.shrink_to_fit();
        self.font_palette_values_rules.shrink_to_fit();
        self.keyframes_rules.shrink_to_fit();
        self.property_rules.shrink_to_fit();
        self.counter_style_rules.shrink_to_fit();
        self.position_try_rules.shrink_to_fit();
        self.layer_intervals.shrink_to_fit();
        self.view_transition_rules.shrink_to_fit();
        self.bloom_hash_backing.shrink_to_fit();
        self.need_compaction = false;
    }
    // cpp: rule_set.cc:1864-1888
    pub fn DidMediaQueryResultsChange(&self, evaluator: &B::MediaQueryEvaluator) -> bool {
        B::DidResultsChange(evaluator, &self.media_query_set_results)
    }
    pub fn DidRoutesChange(&self, document: Option<&B::Document>) -> bool {
        let current = B::NavigationStateForDocument(document);
        if current.is_some() != self.navigation_state.is_some() {
            return true;
        }
        current.is_some_and(|current| current != self.navigation_state.as_deref().unwrap())
    }
    pub fn GetLayerForTest(&self, rule: &RuleData<B>) -> Option<&CascadeLayer> {
        layer_for_position(
            &self.layer_intervals,
            self.implicit_outer_layer.as_ref(),
            rule.GetPosition(),
        )
    }
}

// A set owns the included handles while hashing only allocation identity.
// Equal rule contents never make two distinct rules interchangeable in a diff.
pub struct StyleRuleIdentitySet<B: RuleSetBackend> {
    rules: HashMap<*const StyleRule<B>, Rc<StyleRule<B>>>,
}
impl<B: RuleSetBackend> Default for StyleRuleIdentitySet<B> {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
}
impl<B: RuleSetBackend> StyleRuleIdentitySet<B> {
    pub fn Insert(&mut self, rule: Rc<StyleRule<B>>) {
        self.rules.insert(Rc::as_ptr(&rule), rule);
    }
    pub fn Contains(&self, rule: &Rc<StyleRule<B>>) -> bool {
        self.rules.contains_key(&Rc::as_ptr(rule))
    }
}
impl<B: RuleSetBackend> FromIterator<Rc<StyleRule<B>>> for StyleRuleIdentitySet<B> {
    fn from_iter<T: IntoIterator<Item = Rc<StyleRule<B>>>>(iter: T) -> Self {
        let mut result = Self::default();
        for rule in iter {
            result.Insert(rule);
        }
        result
    }
}
fn scope_at_position<B: RuleSetBackend>(
    intervals: &[Interval<Rc<B::StyleScope>>],
    position: u32,
) -> Option<Rc<B::StyleScope>> {
    let end = intervals.partition_point(|interval| interval.start_position <= position);
    end.checked_sub(1)
        .and_then(|index| intervals[index].value.clone())
}
// Split fields make the source's ownership of diff position/bloom/scope updates
// explicit while allowing RuleMap to append to its own bucket simultaneously.
struct DiffRuleTransfer<'a, B: RuleSetBackend> {
    old: &'a RuleSet<B>,
    bloom: &'a mut Vec<u16>,
    scopes: &'a mut Vec<Interval<Rc<B::StyleScope>>>,
    count: &'a mut u32,
}
impl<B: RuleSetBackend> DiffRuleTransfer<'_, B> {
    // cpp: rule_set.h:699-702; rule_set.cc:1364-1379
    fn NewlyAddedFromDifferentRuleSet(&mut self, data: &mut RuleData<B>) {
        let scope = scope_at_position::<B>(&self.old.scope_intervals, data.GetPosition());
        data.MovedToDifferentRuleSet(&self.old.bloom_hash_backing, self.bloom, *self.count);
        add_interval(scope, *self.count, self.scopes, same_rc);
        *self.count += 1;
    }
}
impl<B: RuleSetBackend> RuleMap<B> {
    // cpp: rule_set.h:239-243; rule_set.cc:1598-1657
    fn AddFilteredRulesFromOtherSet(
        &mut self,
        other: &Self,
        only_include: &StyleRuleIdentitySet<B>,
        transfer: &mut DiffRuleTransfer<'_, B>,
        universal: &mut Vec<RuleData<B>>,
    ) {
        if self.compacted {
            self.Uncompact();
        }
        let add = |dst: &mut Self,
                   key: &AtomicString,
                   data: &RuleData<B>,
                   transfer: &mut DiffRuleTransfer<'_, B>,
                   universal: &mut Vec<RuleData<B>>| {
            if !only_include.Contains(&data.rule) {
                return;
            }
            if dst.Add(key, data) {
                transfer.NewlyAddedFromDifferentRuleSet(dst.backing.last_mut().unwrap());
            } else {
                let mut copy = data.clone();
                UnmarkAsCoveredByBucketing(copy.MutableSelector());
                copy.ComputeEntirelyCoveredByBucketing();
                transfer.NewlyAddedFromDifferentRuleSet(&mut copy);
                universal.push(copy);
            }
        };
        if other.compacted {
            if let Some(buckets) = other.buckets.as_ref() {
                for (key, extent) in buckets.Entries() {
                    for data in
                        &other.backing[extent.index as usize..extent.index as usize + extent.length]
                    {
                        add(self, &key, data, transfer, universal);
                    }
                }
            }
        } else {
            let mut keys = vec![AtomicString::default(); other.num_buckets as usize];
            if let Some(buckets) = other.buckets.as_ref() {
                for (key, extent) in buckets.Entries() {
                    keys[extent.index as usize] = key;
                }
            }
            for (index, data) in other.backing.iter().enumerate() {
                add(
                    self,
                    &keys[other.bucket_number[index] as usize],
                    data,
                    transfer,
                    universal,
                );
            }
        }
    }
}
impl<B: RuleSetBackend> RuleSet<B> {
    // cpp: rule_set.h:409-417; rule_set.cc:1381-1394
    fn AddFilteredRulesFromOtherBucket(
        other: &[RuleData<B>],
        only_include: &StyleRuleIdentitySet<B>,
        transfer: &mut DiffRuleTransfer<'_, B>,
        destination: &mut Vec<RuleData<B>>,
    ) {
        for data in other {
            if only_include.Contains(&data.rule) {
                let mut copy = data.clone();
                transfer.NewlyAddedFromDifferentRuleSet(&mut copy);
                destination.push(copy);
            }
        }
    }
    // cpp: rule_set.h:409-411; rule_set.cc:1396-1453
    // Diff rulesets are for CheckIfAnyRuleMatches; layers, containers and
    // non-style rule categories deliberately do not transfer.
    pub fn AddFilteredRulesFromOtherSet(
        &mut self,
        other: &Self,
        only_include: &StyleRuleIdentitySet<B>,
    ) {
        if other.rule_count == 0 {
            return;
        }
        let mut transfer = DiffRuleTransfer {
            old: other,
            bloom: &mut self.bloom_hash_backing,
            scopes: &mut self.scope_intervals,
            count: &mut self.rule_count,
        };
        macro_rules! map_bucket {
            ($field:ident) => {
                self.$field.AddFilteredRulesFromOtherSet(
                    &other.$field,
                    only_include,
                    &mut transfer,
                    &mut self.universal_rules,
                );
            };
        }
        map_bucket!(id_rules);
        map_bucket!(class_rules);
        map_bucket!(attr_rules);
        map_bucket!(tag_rules);
        map_bucket!(input_rules);
        map_bucket!(ua_shadow_pseudo_element_rules);
        macro_rules! vector_bucket {
            ($field:ident) => {
                Self::AddFilteredRulesFromOtherBucket(
                    &other.$field,
                    only_include,
                    &mut transfer,
                    &mut self.$field,
                );
            };
        }
        vector_bucket!(link_pseudo_class_rules);
        vector_bucket!(cue_pseudo_rules);
        vector_bucket!(focus_pseudo_class_rules);
        vector_bucket!(focus_visible_pseudo_class_rules);
        vector_bucket!(scrollbar_rules);
        vector_bucket!(universal_rules);
        vector_bucket!(shadow_host_rules);
        vector_bucket!(part_pseudo_rules);
        vector_bucket!(slotted_pseudo_element_rules);
        vector_bucket!(active_view_transition_rules);
        vector_bucket!(unbounded_pseudo_class_rules);
        vector_bucket!(root_element_rules);
        self.need_compaction = true;
    }
}

// cpp: rule_set.h:353-357,373-384,390-398,657-672;
// cpp: rule_set.cc:984-1292,1306-1362,1455-1477
impl<B: RuleSetIngestionBackend> RuleSet<B> {
    pub fn AddChildRules(
        &mut self,
        parent_rule: Option<Rc<StyleRule<B>>>,
        rules: &[Rc<StyleRuleBase<B>>],
        medium: &B::MediaQueryEvaluator,
        mixins: &dyn RuleSetMixinMapView<B>,
        flags: AddRuleFlags,
        container_queries: Option<Rc<B::ContainerQuerySet>>,
        layer: Option<CascadeLayer>,
        scope: Option<Rc<B::StyleScope>>,
        stack: &mut ApplyMixinsStack<B>,
    ) {
        for base in rules {
            use StyleRuleBase::*;
            match base.as_ref() {
                Style(rule) => self.AddStyleRule(
                    rule.clone(),
                    parent_rule.clone(),
                    medium,
                    mixins,
                    flags,
                    stack,
                    container_queries.clone(),
                    layer.clone(),
                    scope.clone(),
                ),
                Page(_) => self.AddPageRule(TypedRuleRef::new(base.clone()), layer.clone()),
                Location(rule) => B::CreateRouteIfNeeded(rule, B::IngestionDocument(medium)),
                Navigation(rule) => {
                    if B::EvaluateNavigationQuery(
                        rule.GetNavigationQuery(),
                        B::IngestionDocument(medium),
                    ) {
                        self.AddChildRules(
                            parent_rule.clone(),
                            rule.ChildRules(),
                            medium,
                            mixins,
                            flags,
                            container_queries.clone(),
                            layer.clone(),
                            scope.clone(),
                            stack,
                        );
                    }
                }
                Media(rule) => {
                    if self.MatchMediaForAddRules(medium, B::MediaRuleQueries(rule)) {
                        self.AddChildRules(
                            parent_rule.clone(),
                            rule.ChildRules(),
                            medium,
                            mixins,
                            flags,
                            container_queries.clone(),
                            layer.clone(),
                            scope.clone(),
                            stack,
                        );
                    }
                }
                FontFace(_) => self.AddFontFaceRule(TypedRuleRef::new(base.clone()), layer.clone()),
                FontPaletteValues(rule) => self.AddFontPaletteValuesRule(rule.clone()),
                FontFeatureValues(rule) => {
                    self.AddFontFeatureValuesRule(rule.clone(), layer.clone())
                }
                Keyframes(rule) => self.AddKeyframesRule(rule.clone(), layer.clone()),
                Property(_) => self.AddPropertyRule(TypedRuleRef::new(base.clone()), layer.clone()),
                CounterStyle(rule) => self.AddCounterStyleRule(rule.clone(), layer.clone()),
                ViewTransition(rule) => self.AddViewTransitionRule(rule.clone(), layer.clone()),
                PositionTry(rule) => self.AddPositionTryRule(rule.clone(), layer.clone()),
                Function(_) => self.AddFunctionRule(TypedRuleRef::new(base.clone()), layer.clone()),
                Supports(rule) => {
                    if rule.ConditionIsSupported() {
                        self.AddChildRules(
                            parent_rule.clone(),
                            rule.ChildRules(),
                            medium,
                            mixins,
                            flags,
                            container_queries.clone(),
                            layer.clone(),
                            scope.clone(),
                            stack,
                        );
                    }
                }
                Container(rule) => {
                    let own = B::ContainerRuleQueries(rule);
                    let inner = container_queries.as_ref().map_or_else(
                        || own.clone(),
                        |parent| B::CopyContainerQueriesWithParent(&own, parent.clone()),
                    );
                    self.AddChildRules(
                        parent_rule.clone(),
                        rule.ChildRules(),
                        medium,
                        mixins,
                        flags,
                        Some(inner),
                        layer.clone(),
                        scope.clone(),
                        stack,
                    );
                }
                LayerBlock(rule) => {
                    let inner = self.GetOrAddSubLayer(layer.clone(), rule.GetName());
                    self.AddChildRules(
                        parent_rule.clone(),
                        rule.ChildRules(),
                        medium,
                        mixins,
                        flags,
                        container_queries.clone(),
                        Some(inner),
                        scope.clone(),
                        stack,
                    );
                }
                LayerStatement(rule) => {
                    for name in rule.GetNames() {
                        self.GetOrAddSubLayer(layer.clone(), name);
                    }
                }
                Scope(rule) => {
                    let own = B::ScopeRuleScope(rule);
                    let inner = scope.as_ref().map_or_else(
                        || own.clone(),
                        |parent| B::CopyStyleScopeWithParent(&own, parent.clone()),
                    );
                    self.AddChildRules(
                        parent_rule.clone(),
                        rule.ChildRules(),
                        medium,
                        mixins,
                        flags,
                        container_queries.clone(),
                        layer.clone(),
                        Some(inner),
                        stack,
                    );
                }
                StartingStyle(rule) => self.AddChildRules(
                    parent_rule.clone(),
                    rule.ChildRules(),
                    medium,
                    mixins,
                    flags | kRuleIsStartingStyle,
                    container_queries.clone(),
                    layer.clone(),
                    scope.clone(),
                    stack,
                ),
                ApplyMixin(_) => self.ApplyMixin(
                    parent_rule.clone(),
                    base.clone(),
                    medium,
                    mixins,
                    flags,
                    container_queries.clone(),
                    layer.clone(),
                    scope.clone(),
                    stack,
                ),
                Contents(rule) => {
                    let applying = stack.last().expect("@contents is inside a mixin");
                    let ApplyMixin(apply) = applying.invoking_apply_rule.as_ref() else {
                        unreachable!()
                    };
                    let source = if apply.HasContentsBlock() {
                        Some(applying.invoking_apply_rule.as_ref())
                    } else if !rule.ChildRules().is_empty() {
                        Some(base.as_ref())
                    } else {
                        None
                    };
                    if let Some(source) = source {
                        let cloned = B::CloneRule(
                            source,
                            parent_rule.clone(),
                            Some(applying.mixin_parameter_bindings.clone()),
                        );
                        let children = match cloned.as_ref() {
                            ApplyMixin(rule) => rule.ChildRules(),
                            Contents(rule) => rule.ChildRules(),
                            _ => panic!("clone must retain the group rule type"),
                        };
                        self.AddChildRules(
                            parent_rule.clone(),
                            children,
                            medium,
                            mixins,
                            flags,
                            container_queries.clone(),
                            layer.clone(),
                            scope.clone(),
                            stack,
                        );
                    }
                }
                NestedDeclarations(rule) => self.AddStyleRule(
                    B::NestedDeclarationsInnerStyleRule(rule),
                    parent_rule.clone(),
                    medium,
                    mixins,
                    flags,
                    stack,
                    container_queries.clone(),
                    layer.clone(),
                    scope.clone(),
                ),
                Result(_) => {
                    let applying = stack.last().expect("@result is inside a mixin");
                    let cloned = B::CloneRule(
                        base,
                        parent_rule.clone(),
                        Some(applying.mixin_parameter_bindings.clone()),
                    );
                    let Result(rule) = cloned.as_ref() else {
                        panic!("clone must retain @result type")
                    };
                    self.AddChildRules(
                        parent_rule.clone(),
                        rule.ChildRules(),
                        medium,
                        mixins,
                        flags,
                        container_queries.clone(),
                        layer.clone(),
                        scope.clone(),
                        stack,
                    );
                }
                // Definitions and other non-style-bearing statements are not
                // ingested by AddChildRules in Chromium.
                _ => {}
            }
        }
    }
    // cpp: rule_set.cc:1133-1198
    pub fn FlattenMixinLocals(
        &mut self,
        rules: &[Rc<StyleRuleBase<B>>],
        medium: &B::MediaQueryEvaluator,
        container_queries: Option<Rc<B::ContainerQuerySet>>,
        locals: &mut RuleSetMixinLocals<B>,
        conditional_locals: &mut RuleSetConditionalLocals<B>,
    ) {
        for base in rules {
            match base.as_ref() {
                StyleRuleBase::Media(rule) => {
                    if self.MatchMediaForAddRules(medium, B::MediaRuleQueries(rule)) {
                        self.FlattenMixinLocals(
                            rule.ChildRules(),
                            medium,
                            container_queries.clone(),
                            locals,
                            conditional_locals,
                        );
                    }
                }
                StyleRuleBase::Supports(rule) => {
                    if rule.ConditionIsSupported() {
                        self.FlattenMixinLocals(
                            rule.ChildRules(),
                            medium,
                            container_queries.clone(),
                            locals,
                            conditional_locals,
                        );
                    }
                }
                StyleRuleBase::Container(rule) => {
                    let own = B::ContainerRuleQueries(rule);
                    let inner = container_queries.as_ref().map_or_else(
                        || own.clone(),
                        |parent| B::CopyContainerQueriesWithParent(&own, parent.clone()),
                    );
                    self.FlattenMixinLocals(
                        rule.ChildRules(),
                        medium,
                        Some(inner),
                        locals,
                        conditional_locals,
                    );
                }
                StyleRuleBase::FunctionDeclarations(rule) => {
                    for (name, data) in B::FunctionDeclarationValues(rule) {
                        if let Some(queries) = container_queries.as_ref() {
                            let values = conditional_locals.entry(name).or_default();
                            values.retain(|value| !Rc::ptr_eq(&value.container_queries, queries));
                            values.push(RuleSetCQDependentValue {
                                data,
                                container_queries: queries.clone(),
                            });
                        } else {
                            locals.insert(name.clone(), data);
                            conditional_locals.remove(&name);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    // cpp: rule_set.cc:1200-1292
    pub fn ApplyMixin(
        &mut self,
        parent_rule: Option<Rc<StyleRule<B>>>,
        apply_base: Rc<StyleRuleBase<B>>,
        medium: &B::MediaQueryEvaluator,
        mixins: &dyn RuleSetMixinMapView<B>,
        flags: AddRuleFlags,
        container_queries: Option<Rc<B::ContainerQuerySet>>,
        layer: Option<CascadeLayer>,
        scope: Option<Rc<B::StyleScope>>,
        stack: &mut ApplyMixinsStack<B>,
    ) {
        if self.depends_on_mixins {
            assert_eq!(
                self.based_on_mixin_map_identifier,
                mixins.MapIdentifier(),
                "one RuleSet uses one mixin map"
            );
        } else {
            self.depends_on_mixins = true;
            self.based_on_mixin_map_identifier = mixins.MapIdentifier();
        }
        let StyleRuleBase::ApplyMixin(apply) = apply_base.as_ref() else {
            panic!("ApplyMixin requires @apply")
        };
        let Some(mixin_base) = mixins.Mixins().get(apply.GetName()) else {
            return;
        };
        let StyleRuleBase::Mixin(mixin) = mixin_base.as_ref() else {
            panic!("mixin map contains a non-mixin")
        };
        if stack
            .iter()
            .any(|entry| Rc::ptr_eq(&entry.mixin, mixin_base))
        {
            return;
        }
        if apply.GetArguments().len() > mixin.GetParameters().len() {
            return;
        }
        let mut bindings = HashMap::new();
        for (index, parameter) in mixin.GetParameters().iter().enumerate() {
            let argument = apply.GetArguments().get(index).cloned().flatten();
            if argument.is_none() && parameter.default_value.is_none() {
                continue;
            }
            bindings.insert(
                parameter.name.clone(),
                RuleSetMixinBinding {
                    argument,
                    default_value: parameter.default_value.clone(),
                    syntax: parameter.r#type.clone(),
                },
            );
        }
        let mut locals = HashMap::new();
        let mut conditional_locals = HashMap::new();
        self.FlattenMixinLocals(
            mixin.ChildRules(),
            medium,
            None,
            &mut locals,
            &mut conditional_locals,
        );
        let bindings = B::NewMixinParameterBindings(
            bindings,
            locals,
            conditional_locals,
            stack
                .last()
                .map(|entry| entry.mixin_parameter_bindings.clone()),
        );
        stack.push(ApplyingMixin {
            mixin: mixin_base.clone(),
            invoking_apply_rule: apply_base.clone(),
            mixin_parameter_bindings: bindings,
        });
        self.AddChildRules(
            parent_rule,
            mixin.ChildRules(),
            medium,
            mixins,
            flags,
            container_queries,
            layer,
            scope,
            stack,
        );
        stack.pop();
        B::MutableMediaQueryResultFlags(&mut self.features).Add(mixins.MediaQueryResultFlags());
        self.media_query_set_results
            .extend_from_slice(mixins.MediaQuerySetResults());
    }
    // cpp: rule_set.cc:1306-1362
    pub fn AddRulesFromSheet(
        &mut self,
        sheet: &dyn RuleSetSheetView<B>,
        medium: &B::MediaQueryEvaluator,
        mixins: &dyn RuleSetMixinMapView<B>,
        layer: Option<CascadeLayer>,
        scope: Option<Rc<B::StyleScope>>,
    ) {
        for rule in sheet.PreImportLayerStatementRules() {
            let StyleRuleBase::LayerStatement(rule) = rule.as_ref() else {
                panic!("pre-import rule is @layer statement")
            };
            for name in rule.GetNames() {
                self.GetOrAddSubLayer(layer.clone(), name);
            }
        }
        for rule in sheet.ImportRules() {
            let StyleRuleBase::Import(rule) = rule.as_ref() else {
                panic!("import list contains a non-import")
            };
            if !B::IngestionImportIsSupported(rule)
                || !self.MatchMediaForAddRules(medium, B::IngestionImportMediaQueries(rule))
            {
                continue;
            }
            let import_layer = if B::ImportIsLayered(rule) {
                Some(self.GetOrAddSubLayer(layer.clone(), B::ImportLayerName(rule)))
            } else {
                layer.clone()
            };
            if let Some(imported) = B::IngestionImportStyleSheet(rule) {
                self.AddRulesFromSheet(
                    imported.as_ref(),
                    medium,
                    mixins,
                    import_layer,
                    B::ImportScope(rule),
                );
            }
        }
        B::BeginStyleSheetContents(sheet);
        let _contents_scope = StyleSheetContentsScope::<B>(std::marker::PhantomData);
        self.AddChildRules(
            None,
            &sheet.ChildRules(),
            medium,
            mixins,
            kRuleHasNoSpecialState,
            None,
            layer,
            scope,
            &mut vec![],
        );
        self.navigation_state =
            B::NavigationStateForDocument(B::IngestionDocument(medium)).map(B::CopyNavigationState);
    }
    // cpp: rule_set.cc:1455-1477. The current source adds the supplied style
    // rule directly; clone/renest occurs in @contents/@result above.
    pub fn AddStyleRule(
        &mut self,
        style_rule: Rc<StyleRule<B>>,
        _parent_rule: Option<Rc<StyleRule<B>>>,
        medium: &B::MediaQueryEvaluator,
        mixins: &dyn RuleSetMixinMapView<B>,
        flags: AddRuleFlags,
        stack: &mut ApplyMixinsStack<B>,
        container_queries: Option<Rc<B::ContainerQuerySet>>,
        layer: Option<CascadeLayer>,
        scope: Option<Rc<B::StyleScope>>,
    ) {
        for selector in style_rule.Selectors().ComplexSelectors() {
            let index = style_rule.Selectors().SelectorIndex(selector.First());
            self.AddRule(
                style_rule.clone(),
                index as u32,
                flags,
                container_queries.clone(),
                layer.clone(),
                scope.clone(),
            );
        }
        if let Some(children) = style_rule.ChildRules() {
            self.AddChildRules(
                Some(style_rule.clone()),
                &children,
                medium,
                mixins,
                flags,
                container_queries,
                layer,
                scope,
                stack,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::String;
    // Test adapters supply only external dependencies; selector/list/rule
    // bodies and every bucket operation below are the production entities.
    struct TestBackend;
    struct TestProperties;
    struct TestQueries(bool);
    impl MediaQuerySet for TestQueries {}
    impl crate::style_rule::StyleRulePropertySet for TestProperties {
        type CSSValue = ();
        fn IsMutable(&self) -> bool {
            false
        }
        fn MutableCopy(&self) -> Rc<Self> {
            panic!("unused property mutation")
        }
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            false
        }
        fn GetPropertyCSSValue(
            &self,
            _: crate::css_property_names::CSSPropertyID,
        ) -> Option<Rc<()>> {
            None
        }
    }
    #[derive(Default)]
    struct TestScope {
        from: Option<Rc<CSSSelectorList>>,
        to: Option<Rc<CSSSelectorList>>,
        parent: Option<Rc<TestScope>>,
    }
    impl RuleSetStyleScope for TestScope {
        fn From(&self) -> Option<&CSSSelectorList> {
            self.from.as_deref()
        }
        fn To(&self) -> Option<&CSSSelectorList> {
            self.to.as_deref()
        }
        fn Parent(&self) -> Option<&Self> {
            self.parent.as_deref()
        }
    }
    #[allow(unused_variables)]
    impl StyleRuleDependencies for TestBackend {
        type SelectorList = CSSSelectorList;
        type CSSPropertyValueSet = TestProperties;
        type CSSLazyParsingState = ();
        type MixinParameterBindings = TestMixinBindings;
        type StyleScope = TestScope;
        type MediaQuerySet = TestQueries;
        type ContainerQuerySet = ();
        type ContainerQuery = ();
        type ConditionalExpNode = ();
        type NavigationQuery = ();
        type CSSSyntaxDefinition = ();
        type CSSVariableData = u32;
        type CSSPrivateVariable = ();
        type ExecutionContext = ();
        type StyleRuleImport = ();
        type StyleRuleFontPaletteValues = ();
        type StyleRuleFontFeatureValues = ();
        type StyleRuleFontFeature = ();
        type StyleRuleKeyframes = ();
        type StyleRuleKeyframe = ();
        type StyleRuleNestedDeclarations = ();
        type StyleRuleFunctionDeclarations = Vec<(String, Rc<u32>)>;
        type StyleRuleNamespace = ();
        type StyleRuleCounterStyle = ();
        type StyleRuleViewTransition = ();
        type StyleRulePositionTry = ();
        type StyleRuleLocation = ();
        fn ParseDeclarationListForLazyStyle(
            state: &Self::CSSLazyParsingState,
            offset: usize,
        ) -> Rc<Self::CSSPropertyValueSet> {
            panic!("dependency is outside the bucket test fixture")
        }
        fn ParseCustomPropertyName(text: &String) -> String {
            panic!("dependency is outside the bucket test fixture")
        }
        fn ConsumeSupportsCondition(context: &Self::ExecutionContext, text: &String) -> bool {
            panic!("dependency is outside the bucket test fixture")
        }
        fn ContainerQuerySetToString(set: &Self::ContainerQuerySet) -> String {
            panic!("dependency is outside the bucket test fixture")
        }
        fn ParseContainerQuerySet(
            context: &Self::ExecutionContext,
            text: &String,
        ) -> Option<Rc<Self::ContainerQuerySet>> {
            panic!("dependency is outside the bucket test fixture")
        }
        fn SingleContainerQuery(set: &Self::ContainerQuerySet) -> Option<&Self::ContainerQuery> {
            panic!("dependency is outside the bucket test fixture")
        }
        fn ParseContainerCondition(
            context: &Self::ExecutionContext,
            text: &String,
        ) -> Option<Rc<Self::ConditionalExpNode>> {
            panic!("dependency is outside the bucket test fixture")
        }
        fn SerializeContainerCondition(node: &Self::ConditionalExpNode) -> String {
            panic!("dependency is outside the bucket test fixture")
        }
        fn ContainerQuerySelectorName(query: &Self::ContainerQuery) -> AtomicString {
            panic!("dependency is outside the bucket test fixture")
        }
        fn NewContainerQuery(
            name: AtomicString,
            node: Rc<Self::ConditionalExpNode>,
        ) -> Rc<Self::ContainerQuery> {
            panic!("dependency is outside the bucket test fixture")
        }
        fn NewContainerQuerySet(
            queries: Vec<Rc<Self::ContainerQuery>>,
        ) -> Rc<Self::ContainerQuerySet> {
            panic!("dependency is outside the bucket test fixture")
        }
        fn ParseNavigationQuery(text: &String) -> Option<Rc<Self::NavigationQuery>> {
            panic!("dependency is outside the bucket test fixture")
        }
        fn CopyPrivateVariable(
            variable: &Self::CSSPrivateVariable,
        ) -> Rc<Self::CSSPrivateVariable> {
            panic!("dependency is outside the bucket test fixture")
        }
    }
    struct TestBucketTable(HashMap<AtomicString, Extent>);
    impl RuleBucketTable for TestBucketTable {
        fn New(_: usize) -> Self {
            Self(HashMap::new())
        }
        fn Find(&self, key: &AtomicString) -> Option<&Extent> {
            self.0.get(key)
        }
        fn FindMut(&mut self, key: &AtomicString) -> Option<&mut Extent> {
            self.0.get_mut(key)
        }
        fn Insert(&mut self, key: &AtomicString) -> Option<&mut Extent> {
            if *key == AtomicString::from_str("fail") {
                return None;
            }
            Some(self.0.entry(key.clone()).or_default())
        }
        fn Entries(&self) -> Vec<(AtomicString, Extent)> {
            self.0
                .iter()
                .map(|(key, extent)| (key.clone(), *extent))
                .collect()
        }
    }
    #[derive(Default)]
    struct TestFeatures {
        flags: MediaQueryResultFlags,
        collected: usize,
        never_matches: bool,
    }
    struct TestSubstringMatcher(Vec<RuleSetSubstringPattern>);
    impl RuleSetBackend for TestBackend {
        type BucketTable = TestBucketTable;
        type RuleFeatureSet = TestFeatures;
        type MediaQueryEvaluator = ();
        type NavigationState = ();
        type Document = ();
        type SubstringSetMatcher = TestSubstringMatcher;
        fn CueShadowPseudoId() -> AtomicString {
            AtomicString::from_str("-internal-media-track-cue")
        }
        fn StringForUAShadowPseudoId(_: PseudoId) -> AtomicString {
            AtomicString::from_str("test-ua-shadow")
        }
        fn SelectorIsEasy(_: CSSSelectorComplex<'_>) -> bool {
            false
        }
        fn CollectIdentifierHashes(
            _: CSSSelectorComplex<'_>,
            _: Option<&TestScope>,
            _: &mut Vec<u16>,
            _: &mut u32,
        ) {
        }
        fn BeginSelector(_: &Rc<StyleRule<Self>>, _: u32) {}
        fn EndSelector() {}
        fn CollectFeaturesFromSelector(
            features: &mut TestFeatures,
            _: CSSSelectorComplex<'_>,
            _: Option<&TestScope>,
        ) -> SelectorPreMatch {
            features.collected += 1;
            if features.never_matches {
                SelectorPreMatch::kNeverMatches
            } else {
                SelectorPreMatch::kMayMatch
            }
        }
        fn NewFeatures() -> TestFeatures {
            TestFeatures::default()
        }
        fn MutableMediaQueryResultFlags(features: &mut TestFeatures) -> &mut MediaQueryResultFlags {
            &mut features.flags
        }
        fn EvalMedia(_: &(), query: &TestQueries, _: &mut MediaQueryResultFlags) -> bool {
            query.0
        }
        fn DidResultsChange(_: &(), _: &[MediaQuerySetResult]) -> bool {
            panic!("unused media evaluator")
        }
        fn NavigationStateForDocument(_: Option<&()>) -> Option<&()> {
            None
        }
        fn NewSubstringSetMatcher() -> TestSubstringMatcher {
            TestSubstringMatcher(Vec::new())
        }
        fn BuildSubstringSetMatcher(
            matcher: &mut TestSubstringMatcher,
            patterns: &[RuleSetSubstringPattern],
        ) -> bool {
            if patterns
                .iter()
                .any(|pattern| pattern.pattern == "reject-build")
            {
                return false;
            }
            matcher.0 = patterns.to_vec();
            true
        }
        fn SubstringAnyMatch(matcher: &TestSubstringMatcher, value: &[u8]) -> bool {
            matcher.0.iter().any(|pattern| {
                value
                    .windows(pattern.pattern.len())
                    .any(|window| window == pattern.pattern.as_bytes())
            })
        }
    }
    struct TestMixinBindings {
        bindings: HashMap<String, RuleSetMixinBinding<TestBackend>>,
        locals: RuleSetMixinLocals<TestBackend>,
        conditional: RuleSetConditionalLocals<TestBackend>,
        parent: Option<Rc<TestMixinBindings>>,
    }
    struct TestMixinMap {
        mixins: HashMap<AtomicString, Rc<StyleRuleBase<TestBackend>>>,
        identifier: Option<u64>,
        flags: MediaQueryResultFlags,
        results: Vec<MediaQuerySetResult>,
    }
    impl Default for TestMixinMap {
        fn default() -> Self {
            Self {
                mixins: HashMap::new(),
                identifier: Some(7),
                flags: MediaQueryResultFlags::default(),
                results: vec![],
            }
        }
    }
    impl RuleSetMixinMapView<TestBackend> for TestMixinMap {
        fn Mixins(&self) -> &HashMap<AtomicString, Rc<StyleRuleBase<TestBackend>>> {
            &self.mixins
        }
        fn MapIdentifier(&self) -> Option<u64> {
            self.identifier
        }
        fn MediaQueryResultFlags(&self) -> &MediaQueryResultFlags {
            &self.flags
        }
        fn MediaQuerySetResults(&self) -> &[MediaQuerySetResult] {
            &self.results
        }
    }
    struct TestSheet {
        pre_import: Vec<Rc<StyleRuleBase<TestBackend>>>,
        imports: Vec<Rc<StyleRuleBase<TestBackend>>>,
        children: Vec<Rc<StyleRuleBase<TestBackend>>>,
    }
    impl RuleSetSheetView<TestBackend> for TestSheet {
        fn PreImportLayerStatementRules(&self) -> Vec<Rc<StyleRuleBase<TestBackend>>> {
            self.pre_import.clone()
        }
        fn ImportRules(&self) -> Vec<Rc<StyleRuleBase<TestBackend>>> {
            self.imports.clone()
        }
        fn ChildRules(&self) -> Vec<Rc<StyleRuleBase<TestBackend>>> {
            self.children.clone()
        }
    }
    thread_local! {
        static QUERIES: std::cell::RefCell<Vec<Rc<TestQueries>>> = const { std::cell::RefCell::new(vec![]) };
        static BINDINGS: std::cell::RefCell<Vec<Rc<TestMixinBindings>>> = const { std::cell::RefCell::new(vec![]) };
    }
    impl RuleSetIngestionBackend for TestBackend {
        fn CloneRule(
            _: &StyleRuleBase<Self>,
            _: Option<Rc<StyleRule<Self>>>,
            _: Option<Rc<Self::MixinParameterBindings>>,
        ) -> Rc<StyleRuleBase<Self>> {
            panic!("clone is unused by this fixture")
        }
        fn MediaRuleQueries(
            rule: &crate::style_rule::StyleRuleMedia<Self>,
        ) -> Option<Rc<TestQueries>> {
            rule.MediaQueries().map(|query| {
                QUERIES.with(|queries| {
                    queries
                        .borrow()
                        .iter()
                        .find(|candidate| std::ptr::eq(candidate.as_ref(), query))
                        .expect("registered actual query")
                        .clone()
                })
            })
        }
        fn ContainerRuleQueries(_: &crate::style_rule::StyleRuleContainer<Self>) -> Rc<()> {
            panic!("unused container accessor")
        }
        fn ScopeRuleScope(_: &crate::style_rule::StyleRuleScope<Self>) -> Rc<TestScope> {
            panic!("unused scope accessor")
        }
        fn CopyContainerQueriesWithParent(_: &(), _: Rc<()>) -> Rc<()> {
            panic!("unused container parent")
        }
        fn CopyStyleScopeWithParent(_: &TestScope, _: Rc<TestScope>) -> Rc<TestScope> {
            panic!("unused scope parent")
        }
        fn NestedDeclarationsInnerStyleRule(_: &()) -> Rc<StyleRule<Self>> {
            panic!("unused nested declarations")
        }
        fn FunctionDeclarationValues(
            rule: &Self::StyleRuleFunctionDeclarations,
        ) -> Vec<(String, Rc<u32>)> {
            rule.clone()
        }
        fn NewMixinParameterBindings(
            bindings: HashMap<String, RuleSetMixinBinding<Self>>,
            locals: RuleSetMixinLocals<Self>,
            conditional: RuleSetConditionalLocals<Self>,
            parent: Option<Rc<TestMixinBindings>>,
        ) -> Rc<TestMixinBindings> {
            let result = Rc::new(TestMixinBindings {
                bindings,
                locals,
                conditional,
                parent,
            });
            BINDINGS.with(|seen| seen.borrow_mut().push(result.clone()));
            result
        }
        fn IngestionDocument(_: &()) -> Option<&()> {
            None
        }
        fn CreateRouteIfNeeded(_: &(), _: Option<&()>) {
            panic!("unused location")
        }
        fn EvaluateNavigationQuery(_: &(), _: Option<&()>) -> bool {
            panic!("unused navigation")
        }
        fn IngestionImportIsSupported(_: &()) -> bool {
            panic!("unused import")
        }
        fn IngestionImportMediaQueries(_: &()) -> Option<Rc<TestQueries>> {
            panic!("unused import")
        }
        fn ImportIsLayered(_: &()) -> bool {
            panic!("unused import")
        }
        fn ImportLayerName(_: &()) -> &[AtomicString] {
            panic!("unused import")
        }
        fn ImportScope(_: &()) -> Option<Rc<TestScope>> {
            panic!("unused import")
        }
        fn IngestionImportStyleSheet(_: &()) -> Option<Rc<dyn RuleSetSheetView<Self>>> {
            panic!("unused import")
        }
        fn CopyNavigationState(_: &()) -> Rc<()> {
            panic!("unused navigation")
        }
        fn BeginStyleSheetContents(_: &dyn RuleSetSheetView<Self>) {}
        fn EndStyleSheetContents() {}
    }
    #[test]
    fn child_recursion_tracks_media_order_layers_and_starting_style_without_copying_rules() {
        use crate::style_rule::{
            StyleRuleLayerBlock, StyleRuleLayerStatement, StyleRuleMedia, StyleRuleStartingStyle,
            StyleRuleSupports,
        };
        let parent = rule(vec![value_selector(MatchType::kId, "parent")]);
        let nested = rule(vec![value_selector(MatchType::kClass, "nested")]);
        parent.AddChildRule(Rc::new(StyleRuleBase::StartingStyle(
            StyleRuleStartingStyle::new(vec![Rc::new(StyleRuleBase::Style(nested.clone()))]),
        )));
        let skipped = rule(vec![value_selector(MatchType::kClass, "skipped")]);
        let rejected = Rc::new(TestQueries(false));
        QUERIES.with(|queries| queries.borrow_mut().push(rejected.clone()));
        let sheet = TestSheet {
            pre_import: vec![Rc::new(StyleRuleBase::LayerStatement(
                StyleRuleLayerStatement::new(vec![vec![AtomicString::from_str("before")]]),
            ))],
            imports: vec![],
            children: vec![
                Rc::new(StyleRuleBase::Media(StyleRuleMedia::new(
                    Some(rejected.clone()),
                    vec![Rc::new(StyleRuleBase::Style(skipped.clone()))],
                ))),
                Rc::new(StyleRuleBase::Supports(StyleRuleSupports::new(
                    String::from("false"),
                    false,
                    vec![Rc::new(StyleRuleBase::Style(skipped))],
                ))),
                Rc::new(StyleRuleBase::LayerBlock(StyleRuleLayerBlock::new(
                    vec![AtomicString::from_str("body")],
                    vec![Rc::new(StyleRuleBase::Style(parent.clone()))],
                ))),
            ],
        };
        let mut set = RuleSet::<TestBackend>::new();
        set.AddRulesFromSheet(&sheet, &(), &TestMixinMap::default(), None, None);
        assert_eq!(set.RuleCount(), 2);
        set.id_rules.Compact();
        set.class_rules.Compact();
        let parent_data = &set.IdRules(&AtomicString::from_str("parent"))[0];
        let nested_data = &set.ClassRules(&AtomicString::from_str("nested"))[0];
        assert!(Rc::ptr_eq(parent_data.Rule(), &parent));
        assert!(Rc::ptr_eq(nested_data.Rule(), &nested));
        assert_eq!(parent_data.GetPosition(), 0);
        assert_eq!(nested_data.GetPosition(), 1);
        assert!(!parent_data.IsStartingStyle());
        assert!(nested_data.IsStartingStyle());
        assert!(set
            .ClassRules(&AtomicString::from_str("skipped"))
            .is_empty());
        assert_eq!(set.media_query_set_results.len(), 1);
        // Trait-object vtables may be duplicated; rule/query identity is the
        // allocation address, independent of trait metadata.
        assert!(std::ptr::addr_eq(
            set.media_query_set_results[0].MediaQueries(),
            rejected.as_ref() as &dyn MediaQuerySet
        ));
        assert!(!set.media_query_set_results[0].Result());
        let body = set
            .CascadeLayers()
            .GetOrAddSubLayer(&[AtomicString::from_str("body")]);
        assert_eq!(set.GetLayerForTest(parent_data), Some(&body));
        assert_eq!(set.GetLayerForTest(nested_data), Some(&body));
    }
    #[test]
    fn mixin_cycles_bind_defaults_and_locals_then_merge_media_dependencies() {
        use crate::style_rule::{StyleRuleApplyMixin, StyleRuleFunctionParameter, StyleRuleMixin};
        BINDINGS.with(|seen| seen.borrow_mut().clear());
        let name = AtomicString::from_str("--recursive");
        let style = rule(vec![value_selector(MatchType::kClass, "from-mixin")]);
        let cyclic_apply = Rc::new(StyleRuleBase::ApplyMixin(
            StyleRuleApplyMixin::WithoutContents(name.clone(), vec![]),
        ));
        let local = Rc::new(9);
        let mixin = Rc::new(StyleRuleBase::Mixin(StyleRuleMixin::new(
            name.clone(),
            vec![
                StyleRuleFunctionParameter {
                    name: String::from("--default"),
                    r#type: (),
                    default_value: Some(Rc::new(42)),
                },
                StyleRuleFunctionParameter {
                    name: String::from("--missing"),
                    r#type: (),
                    default_value: None,
                },
            ],
            vec![
                Rc::new(StyleRuleBase::FunctionDeclarations(Rc::new(vec![(
                    String::from("--local"),
                    local.clone(),
                )]))),
                Rc::new(StyleRuleBase::Style(style.clone())),
                cyclic_apply.clone(),
            ],
        )));
        let queries = Rc::new(TestQueries(true));
        let map = TestMixinMap {
            mixins: HashMap::from([(name.clone(), mixin)]),
            identifier: Some(99),
            flags: MediaQueryResultFlags {
                unit_flags: 4,
                is_viewport_dependent: true,
                is_device_dependent: false,
            },
            results: vec![MediaQuerySetResult::new(queries, true)],
        };
        let mut set = RuleSet::<TestBackend>::new();
        let mut stack = vec![];
        set.ApplyMixin(
            None,
            cyclic_apply,
            &(),
            &map,
            0,
            None,
            None,
            None,
            &mut stack,
        );
        assert!(stack.is_empty());
        assert_eq!(set.RuleCount(), 1);
        assert!(set.DependingOnMixins());
        assert!(!set.DependingOnOutdatedMixins(Some(99)));
        assert!(set.DependingOnOutdatedMixins(Some(100)));
        assert_eq!(set.features.flags.unit_flags, 4);
        assert!(set.features.flags.is_viewport_dependent);
        assert_eq!(set.media_query_set_results.len(), 1);
        BINDINGS.with(|seen| {
            let seen = seen.borrow();
            assert_eq!(seen.len(), 1);
            let binding = &seen[0];
            assert_eq!(binding.bindings.len(), 1);
            assert_eq!(
                **binding
                    .bindings
                    .get(&String::from("--default"))
                    .unwrap()
                    .default_value
                    .as_ref()
                    .unwrap(),
                42
            );
            assert!(Rc::ptr_eq(
                binding.locals.get(&String::from("--local")).unwrap(),
                &local
            ));
            assert!(binding.conditional.is_empty());
            assert!(binding.parent.is_none());
        });
        let excess = Rc::new(StyleRuleBase::ApplyMixin(
            StyleRuleApplyMixin::WithoutContents(name, vec![None, None, None]),
        ));
        set.ApplyMixin(None, excess, &(), &map, 0, None, None, None, &mut stack);
        assert_eq!(set.RuleCount(), 1);
        let missing = Rc::new(StyleRuleBase::ApplyMixin(
            StyleRuleApplyMixin::WithoutContents(AtomicString::from_str("--absent"), vec![]),
        ));
        let mut empty = RuleSet::<TestBackend>::new();
        empty.ApplyMixin(None, missing, &(), &map, 0, None, None, None, &mut stack);
        assert!(empty.DependingOnMixins());
        assert_eq!(empty.RuleCount(), 0);
    }
    #[test]
    fn locals_replace_only_same_container_identity_and_unconditional_resets_overrides() {
        let mut set = RuleSet::<TestBackend>::new();
        let mut locals = HashMap::new();
        let mut conditional = HashMap::new();
        let name = String::from("--local");
        let first = Rc::new(());
        let second = Rc::new(());
        let declarations = |value| {
            vec![Rc::new(StyleRuleBase::FunctionDeclarations(Rc::new(vec![
                (name.clone(), Rc::new(value)),
            ])))]
        };
        set.FlattenMixinLocals(
            &declarations(1),
            &(),
            Some(first.clone()),
            &mut locals,
            &mut conditional,
        );
        set.FlattenMixinLocals(
            &declarations(2),
            &(),
            Some(second.clone()),
            &mut locals,
            &mut conditional,
        );
        set.FlattenMixinLocals(
            &declarations(3),
            &(),
            Some(first),
            &mut locals,
            &mut conditional,
        );
        let values = conditional.get(&name).unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(*values[0].data, 2);
        assert_eq!(*values[1].data, 3);
        assert!(Rc::ptr_eq(&values[0].container_queries, &second));
        set.FlattenMixinLocals(&declarations(4), &(), None, &mut locals, &mut conditional);
        assert_eq!(**locals.get(&name).unwrap(), 4);
        assert!(conditional.is_empty());
    }
    #[test]
    fn diff_filters_by_identity_and_relocates_scope_and_bloom_for_both_map_states() {
        for compacted in [false, true] {
            let retained = rule(vec![value_selector(MatchType::kClass, "same")]);
            let equal_content_other = rule(vec![value_selector(MatchType::kClass, "same")]);
            let mut source = RuleSet::<TestBackend>::new();
            let scope = Rc::new(TestScope::default());
            source.AddRule(equal_content_other, 0, 0, None, None, None);
            source.AddRule(retained.clone(), 0, 0, None, None, Some(scope.clone()));
            source.bloom_hash_backing = vec![10, 20, 30];
            source.class_rules.backing[1].bloom_hash_pos = 1;
            source.class_rules.backing[1].bloom_hash_size = 2;
            if compacted {
                source.class_rules.Compact();
            }
            let only_include = [retained.clone()].into_iter().collect();
            let mut diff = RuleSet::<TestBackend>::new();
            diff.AddFilteredRulesFromOtherSet(&source, &only_include);
            diff.class_rules.Compact();
            let data = diff.ClassRules(&AtomicString::from_str("same"));
            assert_eq!(data.len(), 1);
            assert!(Rc::ptr_eq(data[0].Rule(), &retained));
            assert_eq!(data[0].GetPosition(), 0);
            assert_eq!(
                data[0].DescendantSelectorIdentifierHashes(&diff.bloom_hash_backing),
                &[20, 30]
            );
            assert!(Rc::ptr_eq(
                diff.scope_intervals[0].value.as_ref().unwrap(),
                &scope
            ));
            assert_eq!(diff.scope_intervals[0].start_position, 0);
            assert!(diff.layer_intervals.is_empty());
            assert!(diff.container_query_intervals.is_empty());
        }
    }
    #[test]
    fn category_projection_keeps_base_identity() {
        let base = Rc::new(StyleRuleBase::FontFace(
            StyleRuleFontFace::<TestBackend>::new(Rc::new(TestProperties)),
        ));
        let mut set = RuleSet::<TestBackend>::new();
        set.AddChildRules(
            None,
            &[base.clone()],
            &(),
            &TestMixinMap::default(),
            0,
            None,
            None,
            None,
            &mut vec![],
        );
        let projected = &set.FontFaceRules()[0].value;
        assert!(Rc::ptr_eq(projected.Base(), &base));
        let StyleRuleBase::FontFace(payload) = base.as_ref() else {
            unreachable!()
        };
        assert!(std::ptr::eq(&**projected, payload));
    }
    fn value_selector(match_type: MatchType, value: &str) -> CSSSelector {
        let mut selector = CSSSelector::default();
        selector.SetMatch(match_type);
        selector.SetValue(AtomicString::from_str(value), false);
        selector
    }
    fn pseudo(match_type: MatchType, pseudo_type: PseudoType) -> CSSSelector {
        let mut selector = CSSSelector::default();
        selector.SetMatch(match_type);
        selector.SetPseudoType(pseudo_type);
        selector
    }
    fn selector_list(mut selectors: Vec<CSSSelector>) -> Rc<CSSSelectorList> {
        selectors.last_mut().unwrap().SetLastInComplexSelector(true);
        CSSSelectorList::AdoptSelectorVector(selectors)
    }
    fn rule(selectors: Vec<CSSSelector>) -> Rc<StyleRule<TestBackend>> {
        Rc::new(StyleRule::CreateWithoutProperties(
            selector_list(selectors).as_ref().clone(),
        ))
    }
    fn add(set: &mut RuleSet<TestBackend>, rule: Rc<StyleRule<TestBackend>>) {
        set.AddRule(rule, 0, kRuleHasNoSpecialState, None, None, None);
    }
    #[test]
    fn compaction_builds_substring_matchers_with_scope_and_slow_path_fallbacks() {
        use crate::css_selector::{AttributeMatchType, CSSSelectorAttributeContext};
        struct Attributes;
        impl CSSSelectorAttributeContext for Attributes {
            fn IsCaseSensitiveAttribute(&self, _: &QualifiedName) -> bool {
                true
            }
        }
        fn attribute(name: &str, value: &str, match_type: MatchType) -> CSSSelector {
            CSSSelector::FromAttribute(
                match_type,
                QualifiedName::new(
                    AtomicString::default(),
                    AtomicString::from_str(name),
                    AtomicString::default(),
                ),
                AttributeMatchType::kCaseSensitive,
                (match_type != MatchType::kAttributeSet).then(|| AtomicString::from_str(value)),
                &Attributes,
            )
        }
        let mut set = RuleSet::<TestBackend>::new();
        set.CompactRulesIfNeeded();
        assert!(!set.id_rules.IsCompacted());
        for (name, count) in [
            ("text", 50),
            ("small", 49),
            ("presence", 50),
            ("empty-prefix", 50),
            ("all-empty", 50),
            ("failed", 50),
        ] {
            for index in 0..count {
                let (value, match_type) = match (name, index) {
                    ("text", 0) | ("all-empty", _) => ("", MatchType::kAttributeExact),
                    ("text", 1) => ("TeST", MatchType::kAttributeExact),
                    ("text", 3) => ("DIFFERENT", MatchType::kAttributeContain),
                    ("presence", 49) => ("", MatchType::kAttributeSet),
                    ("empty-prefix", 49) => ("", MatchType::kAttributeBegin),
                    ("failed", _) => ("reject-build", MatchType::kAttributeExact),
                    _ => ("test", MatchType::kAttributeExact),
                };
                add(&mut set, rule(vec![attribute(name, value, match_type)]));
            }
        }
        for value in ["SCOPE-A", "SCOPE-B"] {
            let scope = Rc::new(TestScope {
                from: Some(selector_list(vec![attribute(
                    "scoped",
                    value,
                    MatchType::kAttributeExact,
                )])),
                ..TestScope::default()
            });
            for _ in 0..25 {
                set.AddRule(
                    rule(vec![pseudo(
                        MatchType::kPseudoClass,
                        PseudoType::kPseudoScope,
                    )]),
                    0,
                    kRuleHasNoSpecialState,
                    None,
                    None,
                    Some(scope.clone()),
                );
            }
        }
        add(&mut set, rule(vec![value_selector(MatchType::kId, "id")]));
        add(
            &mut set,
            rule(vec![value_selector(MatchType::kClass, "class")]),
        );
        set.universal_rules.reserve(64);
        add(
            &mut set,
            rule(vec![pseudo(
                MatchType::kPseudoElement,
                PseudoType::kPseudoBefore,
            )]),
        );
        set.layer_intervals.reserve(64);
        set.AddRuleToLayerIntervals(Some(CascadeLayer::default()), 0);
        set.bloom_hash_backing.reserve(64);
        set.bloom_hash_backing.extend([11, 22]);
        set.CompactRulesIfNeeded();
        set.AssertCompacted();
        assert!(!set.need_compaction);
        for map in [
            &set.id_rules,
            &set.class_rules,
            &set.attr_rules,
            &set.tag_rules,
            &set.input_rules,
            &set.ua_shadow_pseudo_element_rules,
        ] {
            assert!(map.IsCompacted());
        }
        assert_eq!(set.IdRules(&AtomicString::from_str("id")).len(), 1);
        assert_eq!(set.ClassRules(&AtomicString::from_str("class")).len(), 1);
        assert_eq!(set.universal_rules.capacity(), set.universal_rules.len());
        assert_eq!(set.layer_intervals.capacity(), set.layer_intervals.len());
        assert_eq!(
            set.bloom_hash_backing.capacity(),
            set.bloom_hash_backing.len()
        );
        let text = AtomicString::from_str("text");
        assert_eq!(
            set.attr_substring_matchers[&text].0,
            vec![
                RuleSetSubstringPattern {
                    pattern: "test".into(),
                    id: 0
                },
                RuleSetSubstringPattern {
                    pattern: "different".into(),
                    id: 2
                },
            ]
        );
        for (value, ignored) in [
            ("unrelated", true),
            ("xxTeSTxx", false),
            ("different", false),
            ("", false),
        ] {
            assert_eq!(
                set.CanIgnoreEntireList(
                    set.AttrRules(&text),
                    &text,
                    &AtomicString::from_str(value),
                ),
                ignored
            );
        }
        let scoped = AtomicString::from_str("scoped");
        assert_eq!(
            set.attr_substring_matchers[&scoped].0,
            vec![
                RuleSetSubstringPattern {
                    pattern: "scope-a".into(),
                    id: 0
                },
                RuleSetSubstringPattern {
                    pattern: "scope-b".into(),
                    id: 25
                },
            ]
        );
        for name in ["small", "presence", "empty-prefix", "all-empty", "failed"] {
            let key = AtomicString::from_str(name);
            assert!(!set.attr_substring_matchers.contains_key(&key));
            assert!(!set.CanIgnoreEntireList(
                set.AttrRules(&key),
                &key,
                &AtomicString::from_str("unrelated"),
            ));
        }
        assert_eq!(set.attr_substring_matchers.len(), 2);
        set.CompactRulesIfNeeded();
        assert_eq!(set.attr_substring_matchers.len(), 2);
    }
    #[test]
    fn bucket_priorities_mark_only_matching_subject_selectors() {
        let mut set = RuleSet::<TestBackend>::new();
        let focused = rule(vec![
            value_selector(MatchType::kId, "id"),
            pseudo(MatchType::kPseudoClass, PseudoType::kPseudoFocus),
        ]);
        add(&mut set, focused.clone());
        assert_eq!(set.FocusPseudoClassRules().len(), 1);
        assert!(set.id_rules.IsEmpty());
        assert!(!focused.SelectorAt(0).IsCoveredByBucketing());
        assert!(focused.SelectorAt(1).IsCoveredByBucketing());
        let mut id = value_selector(MatchType::kId, "id");
        id.SetRelation(RelationType::kDescendant);
        let ancestor = rule(vec![id, value_selector(MatchType::kId, "ancestor")]);
        add(&mut set, ancestor.clone());
        assert!(ancestor.SelectorAt(0).IsCoveredByBucketing());
        assert!(!ancestor.SelectorAt(1).IsCoveredByBucketing());
        set.id_rules.Compact();
        assert_eq!(set.IdRules(&AtomicString::from_str("id")).len(), 1);
        assert!(Rc::ptr_eq(
            set.IdRules(&AtomicString::from_str("id"))[0].Rule(),
            &ancestor
        ));
        add(&mut set, rule(vec![value_selector(MatchType::kId, "id")]));
        assert!(!set.id_rules.IsCompacted());
        set.id_rules.Compact();
        assert_eq!(set.IdRules(&AtomicString::from_str("id")).len(), 2);
    }
    #[test]
    fn nested_single_selector_scope_and_shadow_slot_keep_source_bucketing() {
        let mut set = RuleSet::<TestBackend>::new();
        let mut single_is = pseudo(MatchType::kPseudoClass, PseudoType::kPseudoIs);
        single_is.SetSelectorList(Some(selector_list(vec![value_selector(
            MatchType::kClass,
            "inside",
        )])));
        add(&mut set, rule(vec![single_is]));
        set.class_rules.Compact();
        assert_eq!(set.ClassRules(&AtomicString::from_str("inside")).len(), 1);
        assert!(
            !set.ClassRules(&AtomicString::from_str("inside"))[0].IsEntirelyCoveredByBucketing()
        );
        let mut host = pseudo(MatchType::kPseudoClass, PseudoType::kPseudoHost);
        host.SetLastInComplexSelector(true);
        let mut multi_is = pseudo(MatchType::kPseudoClass, PseudoType::kPseudoIs);
        multi_is.SetSelectorList(Some(selector_list(vec![
            host,
            value_selector(MatchType::kClass, "alternative"),
        ])));
        add(&mut set, rule(vec![multi_is]));
        assert_eq!(set.UniversalRules().len(), 1);
        assert!(set.MustCheckUniversalBucketForShadowHost());
        let scope = Rc::new(TestScope {
            from: Some(selector_list(vec![value_selector(
                MatchType::kId,
                "scope-start",
            )])),
            ..TestScope::default()
        });
        set.AddRule(
            rule(vec![pseudo(
                MatchType::kPseudoClass,
                PseudoType::kPseudoScope,
            )]),
            0,
            0,
            None,
            None,
            Some(scope.clone()),
        );
        set.id_rules.Compact();
        assert_eq!(set.IdRules(&AtomicString::from_str("scope-start")).len(), 1);
        assert!(Rc::ptr_eq(
            set.scope_intervals.last().unwrap().value.as_ref().unwrap(),
            &scope
        ));
        let mut ua_shadow = pseudo(MatchType::kPseudoElement, PseudoType::kPseudoPlaceholder);
        ua_shadow.SetRelation(RelationType::kUAShadow);
        let mut slotted = pseudo(MatchType::kPseudoElement, PseudoType::kPseudoSlotted);
        slotted.SetSelectorList(Some(selector_list(vec![CSSSelector::FromTag(
            QualifiedName::AnyQName(),
            false,
        )])));
        add(&mut set, rule(vec![ua_shadow, slotted]));
        assert_eq!(set.SlottedPseudoElementRules().len(), 1);
        assert!(!set.HasUAShadowPseudoElementRules());
        add(
            &mut set,
            rule(vec![
                pseudo(MatchType::kPseudoElement, PseudoType::kPseudoBefore),
                pseudo(MatchType::kPseudoClass, PseudoType::kPseudoFocus),
            ]),
        );
        assert_eq!(set.UniversalRules().len(), 2);
        assert!(set.FocusPseudoClassRules().is_empty());
    }
    #[test]
    fn input_type_bucket_normalizes_values_but_retains_namespace_checks() {
        use crate::css_selector::{AttributeMatchType, CSSSelectorAttributeContext};
        struct Attributes;
        impl CSSSelectorAttributeContext for Attributes {
            fn IsCaseSensitiveAttribute(&self, _: &QualifiedName) -> bool {
                false
            }
        }
        let mut set = RuleSet::<TestBackend>::new();
        for namespace in ["*", "http://www.w3.org/1999/xhtml"] {
            let tag = CSSSelector::FromTag(
                QualifiedName::new(
                    AtomicString::default(),
                    AtomicString::from_str("input"),
                    AtomicString::from_str(namespace),
                ),
                false,
            );
            let attr = CSSSelector::FromAttribute(
                MatchType::kAttributeExact,
                QualifiedName::new(
                    AtomicString::default(),
                    AtomicString::from_str("type"),
                    AtomicString::default(),
                ),
                AttributeMatchType::kCaseSensitive,
                Some(AtomicString::from_str("TeXT")),
                &Attributes,
            );
            let shared = rule(vec![attr, tag]);
            add(&mut set, shared.clone());
            assert!(!shared.SelectorAt(0).IsCoveredByBucketing());
            assert_eq!(
                shared.SelectorAt(1).IsCoveredByBucketing(),
                namespace == "*"
            );
        }
        set.input_rules.Compact();
        assert_eq!(set.InputRules(&AtomicString::from_str("text")).len(), 2);
        assert!(!set.HasAnyAttrRules());
        let style = CSSSelector::FromAttribute(
            MatchType::kAttributeSet,
            QualifiedName::new(
                AtomicString::default(),
                AtomicString::from_str("style"),
                AtomicString::default(),
            ),
            AttributeMatchType::kCaseSensitive,
            None,
            &Attributes,
        );
        add(&mut set, rule(vec![style]));
        assert!(set.HasBucketForStyleAttribute());
    }
    #[test]
    fn visited_split_preserves_rule_identity_and_clears_selector_coverage() {
        let mut set = RuleSet::<TestBackend>::new();
        let shared = rule(vec![
            value_selector(MatchType::kId, "link"),
            pseudo(MatchType::kPseudoClass, PseudoType::kPseudoLink),
        ]);
        add(&mut set, shared.clone());
        assert_eq!(set.RuleCount(), 1);
        set.id_rules.Compact();
        let data = set.IdRules(&AtomicString::from_str("link"));
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].GetPosition(), data[1].GetPosition());
        assert_eq!(data[0].LinkMatchType(), LinkMatchMask::kMatchLink as u32);
        assert_eq!(data[1].LinkMatchType(), LinkMatchMask::kMatchVisited as u32);
        assert!(data.iter().all(|data| Rc::ptr_eq(data.Rule(), &shared)));
        assert!(!shared.SelectorAt(0).IsCoveredByBucketing());
        assert!(!data[1].IsEntirelyCoveredByBucketing());
    }
    #[test]
    fn failed_bucket_insertion_preserves_matching_and_feature_rejection_keeps_position() {
        let mut set = RuleSet::<TestBackend>::new();
        let failed = rule(vec![value_selector(MatchType::kId, "fail")]);
        add(&mut set, failed.clone());
        assert_eq!(set.UniversalRules().len(), 1);
        assert!(Rc::ptr_eq(set.UniversalRules()[0].Rule(), &failed));
        assert!(!failed.SelectorAt(0).IsCoveredByBucketing());
        assert!(!set.UniversalRules()[0].IsEntirelyCoveredByBucketing());
        set.features.never_matches = true;
        add(
            &mut set,
            rule(vec![value_selector(MatchType::kClass, "reject")]),
        );
        assert_eq!(set.RuleCount(), 2);
        assert!(set.class_rules.IsEmpty());
        assert_eq!(set.features.collected, 2);
        set.AddRule(
            failed,
            1 << RuleData::<TestBackend>::kSelectorIndexBits,
            0,
            None,
            None,
            None,
        );
        assert_eq!(set.RuleCount(), 2);
        assert_eq!(set.features.collected, 2);
    }
    #[test]
    fn counting_sort_preserves_each_buckets_insertion_order() {
        let mut values = vec!["a0", "b0", "c0", "a1", "c1", "b1", "a2"];
        let mut numbers = vec![0, 1, 2, 0, 2, 1, 0];
        let (starts, mut order) = counting_sort_positions(&numbers, 3);
        assert_eq!(starts, vec![0, 3, 5]);
        counting_sort_permute(&mut values, &mut numbers, &mut order, &starts);
        assert_eq!(values, vec!["a0", "a1", "a2", "b0", "b1", "c0", "c1"]);
        assert_eq!(numbers, vec![0, 0, 0, 1, 1, 2, 2]);
    }
    #[test]
    fn counting_sort_exhaustive_small_inputs_preserves_bucket_order() {
        for length in 1..=7u32 {
            for encoded in 0..3u32.pow(length) {
                let mut code = encoded;
                let mut numbers: Vec<_> = (0..length)
                    .map(|_| {
                        let value = code % 3;
                        code /= 3;
                        value
                    })
                    .collect();
                // Native bucket numbers are dense and every bucket is nonempty.
                let used: Vec<_> = (0..3).filter(|bucket| numbers.contains(bucket)).collect();
                for number in &mut numbers {
                    *number = used.iter().position(|bucket| bucket == number).unwrap() as u32;
                }
                let mut values: Vec<_> = (0..length).collect();
                let mut expected = values.clone();
                expected.sort_by_key(|&index| numbers[index as usize]);
                let (starts, mut order) = counting_sort_positions(&numbers, used.len());
                counting_sort_permute(&mut values, &mut numbers, &mut order, &starts);
                assert_eq!(values, expected);
            }
        }
    }
    #[test]
    fn bloom_tail_reuses_only_adjacent_equal_hashes_and_discards_overflow_tail() {
        let mut hashes = vec![1, 2, 1, 2];
        assert_eq!(reuse_bloom_tail(&mut hashes, 2, 2), 0);
        assert_eq!(hashes, vec![1, 2]);
        hashes.extend([3, 4]);
        assert_eq!(reuse_bloom_tail(&mut hashes, 2, 2), 2);
        assert_eq!(hashes, vec![1, 2, 3, 4]);
        let mut clamped = vec![7; 255];
        clamped.extend(vec![7; 300]);
        assert_eq!(reuse_bloom_tail(&mut clamped, 255, 255), 0);
        assert_eq!(clamped.len(), 255);
    }
    #[test]
    fn intervals_compare_pointer_identity_and_retain_return_to_null() {
        let first = Rc::new(7);
        let equal_value_other_identity = Rc::new(7);
        let mut intervals = Vec::new();
        add_interval(None, 0, &mut intervals, same_rc::<i32>);
        assert!(intervals.is_empty());
        add_interval(Some(first.clone()), 2, &mut intervals, same_rc);
        add_interval(Some(first), 5, &mut intervals, same_rc);
        add_interval(Some(equal_value_other_identity), 6, &mut intervals, same_rc);
        add_interval(None, 8, &mut intervals, same_rc);
        add_interval(None, 10, &mut intervals, same_rc);
        assert_eq!(
            intervals
                .iter()
                .map(|entry| entry.start_position)
                .collect::<Vec<_>>(),
            vec![2, 6, 8]
        );
    }
    #[test]
    fn layer_positions_use_outer_layer_before_first_interval() {
        let outer = CascadeLayer::default();
        let first = outer.GetOrAddSubLayer(&[AtomicString::from_str("a")]);
        let second = outer.GetOrAddSubLayer(&[AtomicString::from_str("b")]);
        let intervals = vec![
            Interval {
                value: Some(first.clone()),
                start_position: 3,
            },
            Interval {
                value: Some(second.clone()),
                start_position: 5,
            },
            Interval {
                value: Some(outer.clone()),
                start_position: 9,
            },
        ];
        assert_eq!(
            layer_for_position(&intervals, Some(&outer), 0),
            Some(&outer)
        );
        assert_eq!(
            layer_for_position(&intervals, Some(&outer), 3),
            Some(&first)
        );
        assert_eq!(
            layer_for_position(&intervals, Some(&outer), 4),
            Some(&first)
        );
        assert_eq!(
            layer_for_position(&intervals, Some(&outer), 5),
            Some(&second)
        );
        assert_eq!(
            layer_for_position(&intervals, Some(&outer), 9),
            Some(&outer)
        );
    }
}

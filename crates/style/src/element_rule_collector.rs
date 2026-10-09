/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003-2011 Apple Inc. All rights reserved.
 * GNU Library General Public License version 2 or later; see COPYING.LIB.
 */
// cpp: third_party/blink/renderer/core/css/element_rule_collector.h
// cpp: third_party/blink/renderer/core/css/element_rule_collector.cc
// cpp: third_party/blink/renderer/core/css/resolver/match_request.h/.cc
// Source ledger, Chromium commit 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
// /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/:
// element_rule_collector.h: physical=333 effective=161 mapped=129 omitted=32 pending=0.
// element_rule_collector.cc: physical=1515 effective=948 mapped=741 omitted=207 pending=0.
// resolver/match_request.h: physical=317 effective=153 mapped=110 omitted=43 pending=0.
// resolver/match_request.cc: physical=56 effective=29 mapped=29 omitted=0 pending=0.
// Effective excludes comments/copyright, blanks, preprocessing/includes,
// namespaces and lines consisting only of braces/parentheses/semicolons.
// Collector header omitted: 44-49 forward declarations, 52/61/85/87,
// 101-103 test-only Selector, 105/119-123/128/134/149/151/157-159,
// 230 perf-map API, 244/246/251/255/303/304 (GC/macros/access/copy/default-dtor).
// Collector implementation omitted: perf/statistics 78-84,215-219,376-418,
// 583-589,597-600,606-609,706-709,715-733,756-762,1297-1363,1398-1400,
// 1446-1449,1459-1480; debug-only 120/202/450/457/538,543-568,631-636,
// 643-651,809-815,828/1240/1438; layout/access/default-dtor/CFI-only
// 91/93/186/348/350/367/443/788. Everything else in h:51-329 and
// cc:90-1512 is mapped, including recursive CSSOM lookup, tracking and sort.
// MatchRequest header omitted: 38-39/59/61,79-97 debug equality,100-104 GC,
// 106/138/145/147/184/186/196/198/204/216/220/228/230/238/289/292,
// 309-313 VectorTraits layout. Remaining h:58-307 and cc:8-54 map here.
// The actual CSS selectors, StyleRule/RuleData/RuleSet, cascade layer map,
// property sets, bindings and MatchResult are reused. Required external
// methods only call DOM/checker/filter/frame/query/CSSOM collaborators;
// collector branches, bitmap iterators, intervals and cascade order stay here.
#![allow(non_snake_case, non_camel_case_types)]

use crate::cascade_layer_map::CascadeLayerMap;
use crate::css_selector::{CSSSelectorComplex, GetPseudoId, MatchType};
use crate::properties::css_property::CSSProperty;
use crate::resolver::cascade_origin::CascadeOrigin;
use crate::resolver::match_flags::{MatchFlag, MatchFlags};
use crate::resolver::match_result::{
    MatchResult, MatchedPropertiesData, MatchedPropertySet,
    MixinParameterBindings as MatchMixinBindings, TreeScope as MatchTreeScope,
};
use crate::rule_set::{Interval, RuleData, RuleSet, RuleSetBackend};
use crate::style_rule::StyleRule;
use crate::valid_property_filter::ValidPropertyFilter;
use foundation::{AtomicString, CSSPropertyID, EInsideLink};
use layoutng_style::style::computed_style_constants::{IsHighlightPseudoElement, PseudoId};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

// Genuine external selector-checker mode and StyleRequest search-text enum.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectorCheckerMode {
    kResolvingStyle,
    kCollectingStyleRules,
    kCollectingCSSRules,
    kQueryingRules,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchTextRequest {
    kNone,
    kCurrent,
    kNotCurrent,
}

/// Required operations belonging to DOM, SelectorChecker, StyleScopeFrame,
/// container evaluation and CSSOM. Rule/selector/property/result storage is
/// always the existing concrete model. No selector matches or defaults are
/// supplied by this interface. SelectorFilter currently exposes only Mark in
/// its module; this associated type is the real filter, not a second Bloom.
pub trait ElementRuleCollectorBackend:
    RuleSetBackend<
    CSSPropertyValueSet: MatchedPropertySet + 'static,
    MixinParameterBindings: MatchMixinBindings + 'static,
>
{
    type Element;
    type ContainerNode;
    type ScopedStyleResolver;
    type TreeScope: MatchTreeScope + 'static;
    type ElementResolveContext;
    type StyleRecalcContext: Clone;
    type SelectorFilter;
    type StyleRequest: Clone;
    type PartNames;
    type SelectorChecker;
    type SelectorCheckingContext;
    type CheckerMatchResult;
    type StyleScopeFrame;
    type ContainerSelectorCache;
    type Attribute;
    type CSSRule;
    type CSSRuleCollection;
    type CSSStyleSheet;
    type StyleSheetContents;
    type StyleRuleUsageTracker;

    fn ContextElement(&self, c: &Self::ElementResolveContext) -> Rc<Self::Element>;
    fn ContextUltimateOriginatingElementOrSelf(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Rc<Self::Element>;
    fn ContextParentElement(&self, c: &Self::ElementResolveContext) -> Option<Rc<Self::Element>>;
    fn ElementIsPseudoElement(&self, e: &Self::Element) -> bool;
    fn LayoutTraversalParentOfUltimateOriginatingElement(
        &self,
        e: &Self::Element,
    ) -> Option<Rc<Self::Element>>;
    fn ParentStackIsConsistent(
        &self,
        f: &Self::SelectorFilter,
        parent: Option<&Self::Element>,
    ) -> bool;
    fn FastRejectSelector(&self, f: &Self::SelectorFilter, hashes: &[u16]) -> bool;
    fn DefaultStyleRequest(&self) -> Self::StyleRequest;
    fn RequestPseudoId(&self, r: &Self::StyleRequest) -> PseudoId;
    fn RequestPseudoArgument(&self, r: &Self::StyleRequest) -> AtomicString;
    fn RequestSearchText(&self, r: &Self::StyleRequest) -> SearchTextRequest;
    fn RecalcStyleScopeFrame(
        &self,
        c: &Self::StyleRecalcContext,
    ) -> Option<Rc<Self::StyleScopeFrame>>;
    fn RecalcIsEnsuringStyle(&self, c: &Self::StyleRecalcContext) -> bool;
    fn RecalcHasOldStyle(&self, c: &Self::StyleRecalcContext) -> bool;
    fn RecalcSizeContainer(&self, c: &Self::StyleRecalcContext) -> Option<Rc<Self::Element>>;
    fn RecalcTrySet(&self, c: &Self::StyleRecalcContext) -> Option<Rc<Self::CSSPropertyValueSet>>;
    fn RecalcTryTacticsSet(
        &self,
        c: &Self::StyleRecalcContext,
    ) -> Option<Rc<Self::CSSPropertyValueSet>>;
    fn NewStyleScopeFrame(
        &self,
        origin: Rc<Self::Element>,
        parent: Option<Rc<Self::StyleScopeFrame>>,
    ) -> Rc<Self::StyleScopeFrame>;
    fn ParentFrameOrThis(
        &self,
        frame: &Rc<Self::StyleScopeFrame>,
        origin: &Self::Element,
    ) -> Rc<Self::StyleScopeFrame>;
    fn FrameHasSeenImplicitScope(
        &self,
        frame: &Self::StyleScopeFrame,
        scope: &Self::StyleScope,
    ) -> bool;
    fn StyleScopeIsImplicit(&self, scope: &Self::StyleScope) -> bool;
    fn NewSelectorCheckingContext(
        &self,
        c: &Self::ElementResolveContext,
    ) -> Self::SelectorCheckingContext;
    fn CheckingElement(&self, c: &Self::SelectorCheckingContext) -> Rc<Self::Element>;
    fn CheckingPseudoElement(&self, c: &Self::SelectorCheckingContext)
        -> Option<Rc<Self::Element>>;
    fn SetCheckingStyleScopeFrame(
        &self,
        c: &mut Self::SelectorCheckingContext,
        f: Rc<Self::StyleScopeFrame>,
    );
    fn SetCheckingScope(
        &self,
        c: &mut Self::SelectorCheckingContext,
        scope: Option<Rc<Self::ContainerNode>>,
    );
    fn SetCheckingTreeScope(
        &self,
        c: &mut Self::SelectorCheckingContext,
        scope: Option<Rc<Self::TreeScope>>,
    );
    fn SetCheckingPseudo(
        &self,
        c: &mut Self::SelectorCheckingContext,
        id: PseudoId,
        argument: &AtomicString,
    );
    fn SetCheckingVTTOrigin(
        &self,
        c: &mut Self::SelectorCheckingContext,
        origin: Option<Rc<Self::Element>>,
    );
    fn SetCheckingSearchTextCurrent(&self, c: &mut Self::SelectorCheckingContext, current: bool);
    fn SetCheckingStyleScope(
        &self,
        c: &mut Self::SelectorCheckingContext,
        scope: Option<Rc<Self::StyleScope>>,
    );
    fn SetCheckingSelectorAndVisited(
        &self,
        c: &mut Self::SelectorCheckingContext,
        s: CSSSelectorComplex<'_>,
        visited: bool,
    );
    fn NewSelectorChecker(
        &self,
        parts: Option<&Self::PartNames>,
        request: &Self::StyleRequest,
        mode: SelectorCheckerMode,
        ua: bool,
    ) -> Self::SelectorChecker;
    fn NewCheckerMatchResult(&self) -> Self::CheckerMatchResult;
    fn CheckerMatch(
        &self,
        checker: &Self::SelectorChecker,
        c: &mut Self::SelectorCheckingContext,
        result: &mut Self::CheckerMatchResult,
    ) -> bool;
    fn EasySelectorMatch(
        &self,
        s: CSSSelectorComplex<'_>,
        element: &Self::Element,
        pseudo: Option<&Self::Element>,
        id: PseudoId,
        result: &mut Self::CheckerMatchResult,
    ) -> bool;
    fn MatchDynamicPseudo(&self, result: &Self::CheckerMatchResult) -> PseudoId;
    fn MatchFlags(&self, result: &Self::CheckerMatchResult) -> MatchFlags;
    fn MatchProximity(&self, result: &Self::CheckerMatchResult) -> u32;
    fn MatchCustomHighlightName(&self, result: &Self::CheckerMatchResult) -> AtomicString;
    fn GetPseudoElement(
        &self,
        e: &Self::Element,
        id: PseudoId,
        argument: &AtomicString,
    ) -> Option<Rc<Self::Element>>;
    fn ForceStartingStyle(&self, e: &Self::Element) -> bool;
    fn ScopeOwnerShadowHost(&self, scope: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn ScopeTreeScope(&self, scope: &Self::ContainerNode) -> Rc<Self::TreeScope>;
    fn ElementTreeScope(&self, e: &Self::Element) -> Rc<Self::TreeScope>;
    fn ScopeScopedStyleResolver(
        &self,
        scope: &Self::ContainerNode,
    ) -> Option<Rc<Self::ScopedStyleResolver>>;
    fn ResolverCascadeLayerMap(
        &self,
        resolver: &Self::ScopedStyleResolver,
    ) -> Option<Rc<CascadeLayerMap>>;
    fn UserCascadeLayerMap(&self, document: &Self::Document) -> Option<Rc<CascadeLayerMap>>;
    fn ElementDocument(&self, e: &Self::Element) -> Rc<Self::Document>;
    fn AttributeOrClassBloomFilter(&self, e: &Self::Element) -> u32;
    fn ShadowPseudoId(&self, e: &Self::Element) -> AtomicString;
    fn IsVTTElement(&self, e: &Self::Element) -> bool;
    fn HasID(&self, e: &Self::Element) -> bool;
    fn IdForStyleResolution(&self, e: &Self::Element) -> AtomicString;
    fn IsStyledElement(&self, e: &Self::Element) -> bool;
    fn HasClass(&self, e: &Self::Element) -> bool;
    fn ClassNames(&self, e: &Self::Element) -> Vec<AtomicString>;
    fn IsHTMLElement(&self, e: &Self::Element) -> bool;
    fn IsHTMLDocument(&self, d: &Self::Document) -> bool;
    fn Attributes(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>>;
    fn AttributesWithoutStyleUpdate(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>>;
    fn AttributesWithoutUpdate(&self, e: &Self::Element) -> Vec<Rc<Self::Attribute>>;
    fn AttributeLocalName(&self, a: &Self::Attribute) -> AtomicString;
    fn AttributeNamespaceURI(&self, a: &Self::Attribute) -> AtomicString;
    fn AttributeValue(&self, a: &Self::Attribute) -> AtomicString;
    fn LocalName(&self, e: &Self::Element) -> AtomicString;
    fn LocalNameForSelectorMatching(&self, e: &Self::Element) -> AtomicString;
    fn InputTypeAttribute(&self, e: &Self::Element) -> AtomicString;
    fn IsLink(&self, e: &Self::Element) -> bool;
    fn MatchesFocusPseudoClass(&self, e: &Self::Element, id: PseudoId) -> bool;
    fn MatchesFocusVisiblePseudoClass(&self, e: &Self::Element) -> bool;
    fn MatchesActiveViewTransitionPseudoClass(&self, e: &Self::Element) -> bool;
    fn IsUnboundedHTMLElementActive(&self, e: &Self::Element) -> bool;
    fn IsDocumentElement(&self, e: &Self::Element) -> bool;
    fn NewContainerSelectorCache(&self) -> Self::ContainerSelectorCache;
    fn ContainerQuerySetParent(
        &self,
        set: &Self::ContainerQuerySet,
    ) -> Option<Rc<Self::ContainerQuerySet>>;
    fn ContainerQueries(&self, set: &Self::ContainerQuerySet) -> Vec<Rc<Self::ContainerQuery>>;
    fn DetermineContainerStartingElement(
        &self,
        e: &Self::Element,
        pseudo: PseudoId,
        query: &Self::ContainerQuery,
        nearest: Option<Rc<Self::Element>>,
    ) -> Option<Rc<Self::Element>>;
    fn EvalAndAddContainerQuery(
        &self,
        start: Option<Rc<Self::Element>>,
        recalc: &Self::StyleRecalcContext,
        query: &Self::ContainerQuery,
        cache: &mut Self::ContainerSelectorCache,
        result: &mut MatchResult,
    ) -> bool;
    fn SetContainerDependencyFlags(&self, query: &Self::ContainerQuery, result: &mut MatchResult);
    fn PropertyIDs(&self, set: &Self::CSSPropertyValueSet) -> Vec<CSSPropertyID>;
    fn PropertiesIsEmpty(&self, set: &Self::CSSPropertyValueSet) -> bool;
    fn PropertiesHasProperty(&self, set: &Self::CSSPropertyValueSet, id: CSSPropertyID) -> bool;
    fn CollectionLength(&self, c: &Self::CSSRuleCollection) -> usize;
    fn CollectionItemInternal(
        &self,
        c: &Self::CSSRuleCollection,
        index: usize,
    ) -> Rc<Self::CSSRule>;
    fn StyleRuleFromCSSRule(&self, r: &Self::CSSRule) -> Option<Rc<StyleRule<Self>>>;
    fn ImportSheetFromCSSRule(&self, r: &Self::CSSRule) -> Option<Rc<Self::CSSStyleSheet>>;
    fn CSSRuleChildren(&self, r: &Self::CSSRule) -> Option<Rc<Self::CSSRuleCollection>>;
    fn NestedDeclarationsFromCSSRule(
        &self,
        r: &Self::CSSRule,
    ) -> Option<(Rc<StyleRule<Self>>, Rc<Self::CSSRule>)>;
    fn SheetRuleCollection(&self, sheet: &Self::CSSStyleSheet) -> Rc<Self::CSSRuleCollection>;
    fn ActiveScopeStyleSheets(&self, scope: &Self::TreeScope) -> Vec<Rc<Self::CSSStyleSheet>>;
    fn ActiveUserStyleSheets(&self, document: &Self::Document) -> Vec<Rc<Self::CSSStyleSheet>>;
    fn SheetContents(&self, sheet: &Self::CSSStyleSheet) -> Rc<Self::StyleSheetContents>;
    fn StartOrStopSelectorMapTracking(
        &self,
        scope: Option<&Self::TreeScope>,
        document: &Self::Document,
    );
    fn LookupStyleSheetContentsForRule(
        &self,
        rule: &StyleRule<Self>,
    ) -> Option<Rc<Self::StyleSheetContents>>;
    fn ContentsClientInTreeScope(
        &self,
        c: &Self::StyleSheetContents,
        scope: &Self::TreeScope,
    ) -> Option<Rc<Self::CSSStyleSheet>>;
    fn CreateCSSOMWrapper(&self, rule: &Rc<StyleRule<Self>>, position: usize) -> Rc<Self::CSSRule>;
    fn TrackRule(
        &self,
        tracker: &mut Self::StyleRuleUsageTracker,
        sheet: Option<Rc<Self::CSSStyleSheet>>,
        rule: &Rc<StyleRule<Self>>,
    );
}

// cpp: match_request.h:63-136; match_request.cc:8-45.
pub struct RuleSetGroup<B: ElementRuleCollectorBackend> {
    rule_sets: Vec<Rc<RuleSet<B>>>,
    first_index: u32,
    single_scope: u32,
    not_single_scope: u32,
    attr: u32,
    input: u32,
    universal: u32,
    link: u32,
    focus: u32,
    focus_visible: u32,
    need_style_synchronized: bool,
}
impl<B: ElementRuleCollectorBackend> RuleSetGroup<B> {
    pub const kRulesetsRoom: usize = 32;
    pub fn new(group_index: u32) -> Self {
        Self {
            rule_sets: Vec::new(),
            first_index: group_index * 32,
            single_scope: 0,
            not_single_scope: 0,
            attr: 0,
            input: 0,
            universal: 0,
            link: 0,
            focus: 0,
            focus_visible: 0,
            need_style_synchronized: false,
        }
    }
    pub fn IsEmpty(&self) -> bool {
        self.rule_sets.is_empty()
    }
    pub fn IsFull(&self) -> bool {
        self.rule_sets.len() == Self::kRulesetsRoom
    }
    pub fn AddRuleSet(&mut self, set: Rc<RuleSet<B>>) {
        assert!(!self.IsFull());
        let bit = 1u32 << self.rule_sets.len();
        if set.HasAnyAttrRules() {
            self.attr |= bit;
            if set.HasBucketForStyleAttribute() {
                self.need_style_synchronized = true;
            }
        }
        if !set.UniversalRules().is_empty() {
            self.universal |= bit;
        }
        if set.HasAnyInputRules() {
            self.input |= bit;
        }
        if !set.LinkPseudoClassRules().is_empty() {
            self.link |= bit;
        }
        if !set.FocusPseudoClassRules().is_empty() {
            self.focus |= bit;
        }
        if !set.FocusVisiblePseudoClassRules().is_empty() {
            self.focus_visible |= bit;
        }
        if set.SingleScope().is_some() {
            self.single_scope |= bit;
        } else {
            self.not_single_scope |= bit;
        }
        self.rule_sets.push(set);
    }
}
// cpp: match_request.cc:47-54.
pub fn AddRuleSetToRuleSetGroupList<B: ElementRuleCollectorBackend>(
    set: Rc<RuleSet<B>>,
    groups: &mut Vec<RuleSetGroup<B>>,
) {
    if groups.last().map_or(true, |g| g.IsFull()) {
        groups.push(RuleSetGroup::new(groups.len() as u32));
    }
    groups.last_mut().unwrap().AddRuleSet(set);
}
pub struct RuleSetWithIndex<'a, B: ElementRuleCollectorBackend> {
    pub rule_set: &'a RuleSet<B>,
    pub style_sheet_index: u32,
}
pub struct RuleSetIterator<'a, B: ElementRuleCollectorBackend> {
    group: &'a RuleSetGroup<B>,
    bitmap: u32,
}
impl<'a, B: ElementRuleCollectorBackend> Iterator for RuleSetIterator<'a, B> {
    type Item = RuleSetWithIndex<'a, B>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.bitmap == 0 {
            return None;
        }
        let index = self.bitmap.trailing_zeros();
        self.bitmap &= self.bitmap - 1;
        Some(RuleSetWithIndex {
            rule_set: &self.group.rule_sets[index as usize],
            style_sheet_index: index + self.group.first_index,
        })
    }
}
// cpp: match_request.h:142-307. Borrowing the group keeps RuleData buckets
// immutable for the complete matching operation, as required by Chromium.
pub struct MatchRequest<'a, B: ElementRuleCollectorBackend> {
    group: &'a RuleSetGroup<B>,
    scope: Option<Rc<B::ContainerNode>>,
    vtt: Option<Rc<B::Element>>,
    enabled: u32,
}
impl<'a, B: ElementRuleCollectorBackend> MatchRequest<'a, B> {
    pub fn new(
        group: &'a RuleSetGroup<B>,
        scope: Option<Rc<B::ContainerNode>>,
        vtt: Option<Rc<B::Element>>,
    ) -> Self {
        Self {
            group,
            scope,
            vtt,
            enabled: group.single_scope | group.not_single_scope,
        }
    }
    pub fn ForCollector(
        group: &'a RuleSetGroup<B>,
        scope: Option<Rc<B::ContainerNode>>,
        vtt: Option<Rc<B::Element>>,
        collector: &ElementRuleCollector<'_, B>,
    ) -> Self {
        let mut result = Self {
            group,
            scope,
            vtt,
            enabled: group.not_single_scope,
        };
        for bundle in (RuleSetIterator {
            group,
            bitmap: group.single_scope,
        }) {
            if !collector.CanRejectScope(bundle.rule_set.SingleScope().unwrap()) {
                result.enabled |= 1 << (bundle.style_sheet_index - group.first_index);
            }
        }
        result
    }
    pub fn Scope(&self) -> Option<&Rc<B::ContainerNode>> {
        self.scope.as_ref()
    }
    pub fn VTTOriginatingElement(&self) -> Option<&Rc<B::Element>> {
        self.vtt.as_ref()
    }
    fn Iter(&self, bitmap: u32) -> RuleSetIterator<'a, B> {
        RuleSetIterator {
            group: self.group,
            bitmap: bitmap & self.enabled,
        }
    }
    pub fn AllRuleSets(&self) -> RuleSetIterator<'a, B> {
        self.Iter(self.enabled)
    }
    pub fn HasAnyRuleSetsWithAttrRules(&self) -> bool {
        self.group.attr & self.enabled != 0
    }
    pub fn RuleSetsWithAttrRules(&self) -> RuleSetIterator<'a, B> {
        self.Iter(self.group.attr)
    }
    pub fn RuleSetsWithInputRules(&self) -> RuleSetIterator<'a, B> {
        self.Iter(self.group.input)
    }
    pub fn RuleSetsWithUniversalRules(&self) -> RuleSetIterator<'a, B> {
        self.Iter(self.group.universal)
    }
    pub fn RuleSetsWithLinkPseudoClassRules(&self) -> RuleSetIterator<'a, B> {
        self.Iter(self.group.link)
    }
    pub fn HasAnyRuleSetsWithFocusPseudoClassRules(&self) -> bool {
        self.group.focus & self.enabled != 0
    }
    pub fn RuleSetsWithFocusPseudoClassRules(&self) -> RuleSetIterator<'a, B> {
        self.Iter(self.group.focus)
    }
    pub fn HasAnyRuleSetsWithFocusVisiblePseudoClassRules(&self) -> bool {
        self.group.focus_visible & self.enabled != 0
    }
    pub fn RuleSetsWithFocusVisiblePseudoClassRules(&self) -> RuleSetIterator<'a, B> {
        self.Iter(self.group.focus_visible)
    }
    pub fn NeedStyleSynchronized(&self) -> bool {
        self.group.need_style_synchronized
    }
}

// cpp: element_rule_collector.h:51-124. Owns the same StyleRule allocation;
// RuleData itself cannot escape the borrowed RuleSet.
pub struct MatchedRule<B: ElementRuleCollectorBackend> {
    sort_key_: u64,
    position_: u64,
    rule_: Rc<StyleRule<B>>,
    link_match_type_: u8,
    valid_property_filter_: ValidPropertyFilter,
    selector_index_: u16,
}
impl<B: ElementRuleCollectorBackend> MatchedRule<B> {
    fn new(data: &RuleData<B>, layer: u16, proximity: u32, sheet: u32) -> Self {
        Self {
            sort_key_: ((layer as u64) << 48)
                | ((data.Specificity() as u64) << 16)
                | (65535 - proximity.min(65535)) as u64,
            position_: ((sheet as u64) << 18) + data.GetPosition() as u64,
            rule_: data.Rule().clone(),
            link_match_type_: data.LinkMatchType() as u8,
            valid_property_filter_: data.GetValidPropertyFilter(),
            selector_index_: data.SelectorIndex() as u16,
        }
    }
    pub fn Rule(&self) -> &Rc<StyleRule<B>> {
        &self.rule_
    }
    pub fn LayerOrder(&self) -> u16 {
        (self.sort_key_ >> 48) as u16
    }
    pub fn SortKey(&self) -> u64 {
        self.sort_key_
    }
    pub fn GetPosition(&self) -> u64 {
        self.position_
    }
    pub fn LinkMatchType(&self) -> u8 {
        self.link_match_type_
    }
    pub fn GetValidPropertyFilter(&self, ua: bool) -> ValidPropertyFilter {
        if ua {
            ValidPropertyFilter::kNoFilter
        } else {
            self.valid_property_filter_
        }
    }
    pub fn SelectorIndex(&self) -> u16 {
        self.selector_index_
    }
}
// css_rule_list.h IndexedRule, with the genuine external CSSOM object.
pub struct IndexedRule<B: ElementRuleCollectorBackend> {
    pub rule: Option<Rc<B::CSSRule>>,
    pub tree_scope: Option<Rc<B::TreeScope>>,
    pub index: i32,
}
pub type StyleRuleList<B> = Vec<Rc<StyleRule<B>>>;
pub type RuleIndexList<B> = Vec<IndexedRule<B>>;

// cpp: element_rule_collector.cc:91-215. Keeps the actual checking context and
// frame; only the source's precomputed values are duplicated in this wrapper.
struct ContextWithStyleScopeFrame<B: ElementRuleCollectorBackend> {
    _frame: Rc<B::StyleScopeFrame>,
    context: B::SelectorCheckingContext,
    layer_map: Option<Rc<CascadeLayerMap>>,
    reject_starting_styles: bool,
    can_use_easy_selector_matching: bool,
}
impl<B: ElementRuleCollectorBackend> ContextWithStyleScopeFrame<B> {
    fn new(
        backend: &B,
        context: &B::ElementResolveContext,
        request: &MatchRequest<'_, B>,
        pseudo: &B::StyleRequest,
        recalc: &B::StyleRecalcContext,
        mode: SelectorCheckerMode,
        ua: bool,
        no_sheet: bool,
    ) -> Self {
        let origin = backend.ContextUltimateOriginatingElementOrSelf(context);
        let frame =
            backend.NewStyleScopeFrame(origin.clone(), backend.RecalcStyleScopeFrame(recalc));
        let mut checking = backend.NewSelectorCheckingContext(context);
        let document = backend.ElementDocument(&backend.ContextElement(context));
        let layer_map = if request.vtt.is_some() || ua || no_sheet {
            None
        } else if let Some(scope) = request.scope.as_deref() {
            backend
                .ScopeScopedStyleResolver(scope)
                .and_then(|resolver| backend.ResolverCascadeLayerMap(&resolver))
        } else {
            backend.UserCascadeLayerMap(&document)
        };
        backend
            .SetCheckingStyleScopeFrame(&mut checking, backend.ParentFrameOrThis(&frame, &origin));
        backend.SetCheckingScope(&mut checking, request.scope.clone());
        backend.SetCheckingTreeScope(
            &mut checking,
            request.scope.as_deref().map(|s| backend.ScopeTreeScope(s)),
        );
        let id = backend.RequestPseudoId(pseudo);
        let argument = backend.RequestPseudoArgument(pseudo);
        backend.SetCheckingPseudo(&mut checking, id, &argument);
        backend.SetCheckingVTTOrigin(&mut checking, request.vtt.clone());
        match backend.RequestSearchText(pseudo) {
            SearchTextRequest::kNone => (),
            SearchTextRequest::kCurrent => {
                backend.SetCheckingSearchTextCurrent(&mut checking, true)
            }
            SearchTextRequest::kNotCurrent => {
                backend.SetCheckingSearchTextCurrent(&mut checking, false)
            }
        }
        let element = backend.CheckingElement(&checking);
        let mut pseudo_element = backend.CheckingPseudoElement(&checking);
        if pseudo_element.is_none() && id != PseudoId::kPseudoIdNone {
            pseudo_element = backend.GetPseudoElement(&element, id, &argument);
        }
        let mut force = false;
        if let Some(pseudo) = pseudo_element {
            force = backend.ForceStartingStyle(&pseudo);
        }
        if !force {
            force = backend.ForceStartingStyle(&element);
        }
        let reject = (backend.RecalcIsEnsuringStyle(recalc)
            || backend.RecalcHasOldStyle(recalc)
            || mode != SelectorCheckerMode::kResolvingStyle)
            && !force;
        let is_host = request
            .scope
            .as_deref()
            .and_then(|s| backend.ScopeOwnerShadowHost(s))
            .is_some_and(|h| Rc::ptr_eq(&h, &element));
        let easy = request.vtt.is_none()
            && !is_host
            && !(backend.CheckingPseudoElement(&checking).is_some()
                && id != PseudoId::kPseudoIdNone);
        Self {
            _frame: frame,
            context: checking,
            layer_map,
            reject_starting_styles: reject,
            can_use_easy_selector_matching: easy,
        }
    }
}
fn AdjustLinkMatchType(inside: EInsideLink, kind: u8) -> u8 {
    if inside == EInsideLink::kNotInsideLink {
        1
    } else {
        kind
    }
}
fn LinkMatchTypeFromInsideLink(inside: EInsideLink) -> u8 {
    match inside {
        EInsideLink::kNotInsideLink => 3,
        EInsideLink::kInsideVisitedLink => 2,
        EInsideLink::kInsideUnvisitedLink => 1,
    }
}
fn SeekInterval<'a, T>(
    intervals: &'a [Interval<T>],
    cursor: &mut usize,
    position: u32,
) -> Option<&'a T> {
    while *cursor < intervals.len() && intervals[*cursor].start_position <= position {
        *cursor += 1;
    }
    if *cursor == 0 {
        None
    } else {
        intervals[*cursor - 1].value.as_ref()
    }
}

pub fn FindStyleRule<B: ElementRuleCollectorBackend>(
    backend: &B,
    rules: Option<&B::CSSRuleCollection>,
    target: &Rc<StyleRule<B>>,
) -> Option<Rc<B::CSSRule>> {
    let rules = rules?;
    for index in 0..backend.CollectionLength(rules) {
        let rule = backend.CollectionItemInternal(rules, index);
        if let Some(style) = backend.StyleRuleFromCSSRule(&rule) {
            if Rc::ptr_eq(&style, target) {
                return Some(rule);
            }
            if let Some(result) =
                FindStyleRule(backend, backend.CSSRuleChildren(&rule).as_deref(), target)
            {
                return Some(result);
            }
        } else if let Some(sheet) = backend.ImportSheetFromCSSRule(&rule) {
            if let Some(result) =
                FindStyleRule(backend, Some(&backend.SheetRuleCollection(&sheet)), target)
            {
                return Some(result);
            }
        } else if let Some(result) =
            FindStyleRule(backend, backend.CSSRuleChildren(&rule).as_deref(), target)
        {
            return Some(result);
        } else if let Some((inner, wrapper)) = backend.NestedDeclarationsFromCSSRule(&rule) {
            if Rc::ptr_eq(&inner, target) {
                return Some(wrapper);
            }
        }
    }
    None
}
pub fn SlowFindStyleSheet<B: ElementRuleCollectorBackend>(
    backend: &B,
    scope: Option<&B::TreeScope>,
    document: &B::Document,
    rule: &Rc<StyleRule<B>>,
) -> Option<Rc<B::CSSStyleSheet>> {
    if let Some(scope) = scope {
        for sheet in backend.ActiveScopeStyleSheets(scope) {
            if FindStyleRule(backend, Some(&backend.SheetRuleCollection(&sheet)), rule).is_some() {
                return Some(sheet);
            }
        }
    }
    for sheet in backend.ActiveUserStyleSheets(document) {
        if FindStyleRule(backend, Some(&backend.SheetRuleCollection(&sheet)), rule).is_some() {
            return Some(sheet);
        }
    }
    None
}
pub fn FindStyleSheet<B: ElementRuleCollectorBackend>(
    backend: &B,
    scope: Option<&B::TreeScope>,
    document: &B::Document,
    rule: &Rc<StyleRule<B>>,
) -> Option<Rc<B::CSSStyleSheet>> {
    backend.StartOrStopSelectorMapTracking(scope, document);
    if let Some(contents) = backend.LookupStyleSheetContentsForRule(rule) {
        if let Some(scope) = scope {
            return backend.ContentsClientInTreeScope(&contents, scope);
        }
        for sheet in backend.ActiveUserStyleSheets(document) {
            if Rc::ptr_eq(&backend.SheetContents(&sheet), &contents) {
                return Some(sheet);
            }
        }
        None
    } else {
        SlowFindStyleSheet(backend, scope, document, rule)
    }
}

pub struct ElementRuleCollector<'a, B: ElementRuleCollectorBackend> {
    backend: &'a B,
    context_: &'a B::ElementResolveContext,
    style_recalc_context_: B::StyleRecalcContext,
    selector_filter_: &'a B::SelectorFilter,
    pseudo_style_request_: B::StyleRequest,
    mode_: SelectorCheckerMode,
    can_use_fast_reject_: bool,
    matching_ua_rules_: bool,
    matching_rules_from_no_style_sheet_: bool,
    suppress_visited_: bool,
    inside_link_: EInsideLink,
    current_rule_tree_scope_: Option<Rc<B::TreeScope>>,
    matched_rules_: Vec<MatchedRule<B>>,
    container_selector_cache_: B::ContainerSelectorCache,
    css_rule_list_: Option<RuleIndexList<B>>,
    style_rule_list_: Option<StyleRuleList<B>>,
    result_: &'a mut MatchResult,
}
impl<'a, B: ElementRuleCollectorBackend> ElementRuleCollector<'a, B> {
    // cpp: element_rule_collector.cc:427-473
    pub fn new(
        backend: &'a B,
        context: &'a B::ElementResolveContext,
        recalc: &B::StyleRecalcContext,
        filter: &'a B::SelectorFilter,
        result: &'a mut MatchResult,
        inside: EInsideLink,
    ) -> Self {
        let element = backend.ContextElement(context);
        let parent = if backend.ElementIsPseudoElement(&element) {
            backend.LayoutTraversalParentOfUltimateOriginatingElement(&element)
        } else {
            backend.ContextParentElement(context)
        };
        let fast = backend.ParentStackIsConsistent(filter, parent.as_deref());
        Self {
            backend,
            context_: context,
            style_recalc_context_: recalc.clone(),
            selector_filter_: filter,
            pseudo_style_request_: backend.DefaultStyleRequest(),
            mode_: SelectorCheckerMode::kResolvingStyle,
            can_use_fast_reject_: fast,
            matching_ua_rules_: false,
            matching_rules_from_no_style_sheet_: false,
            suppress_visited_: false,
            inside_link_: inside,
            current_rule_tree_scope_: None,
            matched_rules_: Vec::new(),
            container_selector_cache_: backend.NewContainerSelectorCache(),
            css_rule_list_: None,
            style_rule_list_: None,
            result_: result,
        }
    }
    pub fn SetMode(&mut self, mode: SelectorCheckerMode) {
        self.mode_ = mode;
    }
    pub fn SetPseudoElementStyleRequest(&mut self, request: &B::StyleRequest) {
        self.pseudo_style_request_ = request.clone();
    }
    pub fn SetMatchingUARules(&mut self, value: bool) {
        self.matching_ua_rules_ = value;
    }
    pub fn SetMatchingRulesFromNoStyleSheet(&mut self, value: bool) {
        self.matching_rules_from_no_style_sheet_ = value;
    }
    pub fn SetSuppressVisited(&mut self, value: bool) {
        self.suppress_visited_ = value;
    }
    pub fn MatchedResult(&self) -> &MatchResult {
        self.result_
    }
    pub fn MatchedStyleRuleList(&mut self) -> Option<StyleRuleList<B>> {
        self.style_rule_list_.take()
    }
    pub fn MatchedCSSRuleList(&mut self) -> Option<RuleIndexList<B>> {
        self.css_rule_list_.take()
    }
    pub fn ClearMatchedRules(&mut self) {
        self.matched_rules_.clear();
    }
    fn EnsureStyleRuleList(&mut self) -> &mut StyleRuleList<B> {
        self.style_rule_list_.get_or_insert_with(Vec::new)
    }
    fn EnsureRuleList(&mut self) -> &mut RuleIndexList<B> {
        self.css_rule_list_.get_or_insert_with(Vec::new)
    }
    pub fn GetPseudoId(&self) -> PseudoId {
        self.backend.RequestPseudoId(&self.pseudo_style_request_)
    }
    pub fn GetPseudoArgument(&self) -> AtomicString {
        self.backend
            .RequestPseudoArgument(&self.pseudo_style_request_)
    }
    pub fn BeginAddingAuthorRulesForTreeScope(&mut self, scope: Rc<B::TreeScope>) {
        self.current_rule_tree_scope_ = Some(scope.clone());
        self.result_.BeginAddingAuthorRulesForTreeScope(scope);
    }
    pub fn MatchedRulesForTest(&self) -> &[MatchedRule<B>] {
        &self.matched_rules_
    }
    pub fn CanRejectScope(&self, scope: &B::StyleScope) -> bool {
        if !self.backend.StyleScopeIsImplicit(scope) {
            return false;
        }
        self.backend
            .RecalcStyleScopeFrame(&self.style_recalc_context_)
            .is_some_and(|f| !self.backend.FrameHasSeenImplicitScope(&f, scope))
    }
    // cpp: element_rule_collector.cc:492-545
    pub fn AddElementStyleProperties(
        &mut self,
        set: Option<Rc<B::CSSPropertyValueSet>>,
        origin: CascadeOrigin,
        cacheable: bool,
        inline: bool,
    ) {
        let Some(set) = set else {
            return;
        };
        let mut data = MatchedPropertiesData::default();
        data.origin = origin;
        data.set_link_match_type(AdjustLinkMatchType(self.inside_link_, 3));
        data.set_is_inline_style(inline);
        self.result_.AddMatchedProperties(set, None, data);
        if !cacheable {
            self.result_.SetIsCacheable(false);
        }
    }
    pub fn AddTryStyleProperties(&mut self) {
        let Some(set) = self.backend.RecalcTrySet(&self.style_recalc_context_) else {
            return;
        };
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kAuthor;
        data.set_link_match_type(AdjustLinkMatchType(self.inside_link_, 3));
        data.set_valid_property_filter(ValidPropertyFilter::kPositionTry as u8);
        data.set_is_try_style(true);
        self.result_.AddMatchedProperties(set, None, data);
        self.result_.SetIsCacheable(false);
    }
    pub fn AddTryTacticsStyleProperties(&mut self) {
        let Some(set) = self
            .backend
            .RecalcTryTacticsSet(&self.style_recalc_context_)
        else {
            return;
        };
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kAuthor;
        data.is_try_tactics_style = true;
        data.set_link_match_type(AdjustLinkMatchType(self.inside_link_, 3));
        self.result_.AddMatchedProperties(set, None, data);
        self.result_.SetIsCacheable(false);
    }
    fn EvaluateAndAddContainerQueries(&mut self, set: &Rc<B::ContainerQuerySet>) -> bool {
        let mut current = Some(set.clone());
        let element = self.backend.ContextElement(self.context_);
        let pseudo = self.GetPseudoId();
        while let Some(set) = current {
            let mut matched = false;
            for query in self.backend.ContainerQueries(&set) {
                let start = self.backend.DetermineContainerStartingElement(
                    &element,
                    pseudo,
                    &query,
                    self.backend
                        .RecalcSizeContainer(&self.style_recalc_context_),
                );
                if self.backend.EvalAndAddContainerQuery(
                    start,
                    &self.style_recalc_context_,
                    &query,
                    &mut self.container_selector_cache_,
                    self.result_,
                ) {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return false;
            }
            current = self.backend.ContainerQuerySetParent(&set);
        }
        true
    }
    fn AddContainerDependencyFlags(&mut self, set: &Rc<B::ContainerQuerySet>) {
        let mut current = Some(set.clone());
        while let Some(set) = current {
            for query in self.backend.ContainerQueries(&set) {
                self.backend
                    .SetContainerDependencyFlags(&query, self.result_);
            }
            current = self.backend.ContainerQuerySetParent(&set);
        }
    }
    fn AffectsAnimations(&self, data: &RuleData<B>) -> bool {
        for id in self.backend.PropertyIDs(&data.Rule().Properties()) {
            if id == CSSPropertyID::kAll {
                return true;
            }
            if id == CSSPropertyID::kVariable {
                continue;
            }
            if CSSProperty::Get(id).IsAnimationProperty() {
                return true;
            }
        }
        false
    }
    // cpp: element_rule_collector.cc:589-738,741-771. Pure perf/statistics
    // variants share this same production matching body.
    fn CollectMatchingRulesForList<const STOP: bool>(
        &mut self,
        rules: &[RuleData<B>],
        set: &RuleSet<B>,
        sheet: u32,
        checker: &B::SelectorChecker,
        context: &mut ContextWithStyleScopeFrame<B>,
    ) -> bool {
        if rules.is_empty() {
            return false;
        }
        let (mut layer_cursor, mut query_cursor, mut scope_cursor) = (0, 0, 0);
        let element = self.backend.CheckingElement(&context.context);
        let bloom = self.backend.AttributeOrClassBloomFilter(&element);
        let pseudo_element = self.backend.CheckingPseudoElement(&context.context);
        let is_pseudo = pseudo_element.is_some() || self.GetPseudoId() != PseudoId::kPseudoIdNone;
        for data in rules {
            if data.RejectElement(bloom)
                || (self.can_use_fast_reject_
                    && self.backend.FastRejectSelector(
                        self.selector_filter_,
                        data.DescendantSelectorIdentifierHashes(set.BloomHashBacking()),
                    ))
            {
                continue;
            }
            let selector = data.Selector();
            if is_pseudo && !selector.MatchesPseudoElement() {
                continue;
            }
            if data.IsStartingStyle() && context.reject_starting_styles {
                continue;
            }
            let scope =
                SeekInterval(set.ScopeIntervals(), &mut scope_cursor, data.GetPosition()).cloned();
            self.backend
                .SetCheckingStyleScope(&mut context.context, scope.clone());
            let easy = context.can_use_easy_selector_matching && scope.is_none();
            let mut result = self.backend.NewCheckerMatchResult();
            if easy && data.IsEntirelyCoveredByBucketing() {
            } else if easy && data.SelectorIsEasy() {
                if !self.backend.EasySelectorMatch(
                    selector,
                    &element,
                    pseudo_element.as_deref(),
                    self.GetPseudoId(),
                    &mut result,
                ) {
                    continue;
                }
            } else {
                self.backend.SetCheckingSelectorAndVisited(
                    &mut context.context,
                    selector,
                    !self.suppress_visited_ && data.LinkMatchType() == 2,
                );
                let matched = self
                    .backend
                    .CheckerMatch(checker, &mut context.context, &mut result);
                self.result_.AddFlags(self.backend.MatchFlags(&result));
                if !matched {
                    continue;
                }
            }
            if STOP {
                return true;
            }
            let queries = SeekInterval(
                set.ContainerQueryIntervals(),
                &mut query_cursor,
                data.GetPosition(),
            )
            .cloned();
            if let Some(ref queries) = queries {
                if self.GetPseudoId() != PseudoId::kPseudoIdNone
                    || self.backend.MatchDynamicPseudo(&result) == PseudoId::kPseudoIdNone
                {
                    if !self.EvaluateAndAddContainerQueries(queries) {
                        if self.AffectsAnimations(data) {
                            self.result_.SetConditionallyAffectsAnimations();
                        }
                        continue;
                    }
                } else {
                    self.AddContainerDependencyFlags(queries);
                }
            }
            let layer = SeekInterval(set.LayerIntervals(), &mut layer_cursor, data.GetPosition());
            let order = match (context.layer_map.as_deref(), layer) {
                (Some(map), Some(layer)) => map.GetLayerOrder(layer),
                _ => CascadeLayerMap::kImplicitOuterLayerOrder,
            };
            self.DidMatchRule(
                data,
                order,
                queries.as_ref(),
                self.backend.MatchProximity(&result),
                &result,
                sheet,
            );
        }
        false
    }
    // cpp: element_rule_collector.cc:805-1089. Attribute snapshots are refreshed
    // after every matching call; source-order bucket iteration stays explicit.
    fn CollectMatchingRulesInternal<const STOP: bool>(
        &mut self,
        request: &MatchRequest<'_, B>,
        parts: Option<&B::PartNames>,
    ) -> bool {
        let checker = self.backend.NewSelectorChecker(
            parts,
            &self.pseudo_style_request_,
            self.mode_,
            self.matching_ua_rules_,
        );
        let mut context = ContextWithStyleScopeFrame::new(
            self.backend,
            self.context_,
            request,
            &self.pseudo_style_request_,
            &self.style_recalc_context_,
            self.mode_,
            self.matching_ua_rules_,
            self.matching_rules_from_no_style_sheet_,
        );
        let element = self.backend.CheckingElement(&context.context);
        let shadow_id = self.backend.ShadowPseudoId(&element);
        if !shadow_id.empty() {
            for bundle in request.AllRuleSets() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.UAShadowPseudoElementRules(&shadow_id),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if self.backend.IsVTTElement(&element) {
            for bundle in request.AllRuleSets() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.CuePseudoRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if !self.matching_ua_rules_
            && request.scope.as_deref().is_some_and(|scope| {
                !Rc::ptr_eq(
                    &self.backend.ElementTreeScope(&element),
                    &self.backend.ScopeTreeScope(scope),
                )
            })
        {
            return false;
        }
        if self.backend.HasID(&element) {
            let id = self.backend.IdForStyleResolution(&element);
            for bundle in request.AllRuleSets() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.IdRules(&id),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if self.backend.IsStyledElement(&element) && self.backend.HasClass(&element) {
            for name in self.backend.ClassNames(&element) {
                for bundle in request.AllRuleSets() {
                    if self.CollectMatchingRulesForList::<STOP>(
                        bundle.rule_set.ClassRules(&name),
                        bundle.rule_set,
                        bundle.style_sheet_index,
                        &checker,
                        &mut context,
                    ) {
                        return true;
                    }
                }
            }
        }
        if request.HasAnyRuleSetsWithAttrRules() {
            let lower = !self.backend.IsHTMLElement(&element)
                && self
                    .backend
                    .IsHTMLDocument(&self.backend.ElementDocument(&element));
            let mut attributes = if request.NeedStyleSynchronized() {
                self.backend.Attributes(&element)
            } else {
                self.backend.AttributesWithoutStyleUpdate(&element)
            };
            let mut index = 0;
            while index < attributes.len() {
                let name = self.backend.AttributeLocalName(&attributes[index]);
                let name = if lower
                    && self
                        .backend
                        .AttributeNamespaceURI(&attributes[index])
                        .IsNull()
                {
                    name.ToAsciiLower()
                } else {
                    name
                };
                for bundle in request.RuleSetsWithAttrRules() {
                    let list = bundle.rule_set.AttrRules(&name);
                    let value = self.backend.AttributeValue(&attributes[index]);
                    if list.is_empty() || bundle.rule_set.CanIgnoreEntireList(list, &name, &value) {
                        continue;
                    }
                    if self.CollectMatchingRulesForList::<STOP>(
                        list,
                        bundle.rule_set,
                        bundle.style_sheet_index,
                        &checker,
                        &mut context,
                    ) {
                        return true;
                    }
                    attributes = self.backend.AttributesWithoutUpdate(&element);
                }
                index += 1;
            }
        }
        if self.backend.LocalName(&element) == AtomicString::from_str("input") {
            let value = self.backend.InputTypeAttribute(&element);
            if !value.IsNull() {
                let value = value.ToAsciiLower();
                for bundle in request.RuleSetsWithInputRules() {
                    if self.CollectMatchingRulesForList::<STOP>(
                        bundle.rule_set.InputRules(&value),
                        bundle.rule_set,
                        bundle.style_sheet_index,
                        &checker,
                        &mut context,
                    ) {
                        return true;
                    }
                }
            }
        }
        if self.backend.IsLink(&element) {
            for bundle in request.RuleSetsWithLinkPseudoClassRules() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.LinkPseudoClassRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if request.HasAnyRuleSetsWithFocusPseudoClassRules()
            && self
                .backend
                .MatchesFocusPseudoClass(&element, PseudoId::kPseudoIdNone)
        {
            for bundle in request.RuleSetsWithFocusPseudoClassRules() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.FocusPseudoClassRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if request.HasAnyRuleSetsWithFocusVisiblePseudoClassRules()
            && self.backend.MatchesFocusVisiblePseudoClass(&element)
        {
            for bundle in request.RuleSetsWithFocusVisiblePseudoClassRules() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.FocusVisiblePseudoClassRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if self
            .backend
            .MatchesActiveViewTransitionPseudoClass(&element)
        {
            for bundle in request.AllRuleSets() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.ActiveViewTransitionRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if self.backend.IsUnboundedHTMLElementActive(&element) {
            for bundle in request.AllRuleSets() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.UnboundedPseudoClassRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if self.GetPseudoId() >= PseudoId::kPseudoIdScrollbarThumb
            && self.GetPseudoId() <= PseudoId::kPseudoIdScrollbarCorner
        {
            for bundle in request.AllRuleSets() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.ScrollbarRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        if self.backend.IsDocumentElement(&element) {
            for bundle in request.AllRuleSets() {
                if self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.RootElementRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                ) {
                    return true;
                }
            }
        }
        let name = if self.matching_ua_rules_ {
            self.backend.LocalName(&element)
        } else {
            self.backend.LocalNameForSelectorMatching(&element)
        };
        for bundle in request.AllRuleSets() {
            if self.CollectMatchingRulesForList::<STOP>(
                bundle.rule_set.TagRules(&name),
                bundle.rule_set,
                bundle.style_sheet_index,
                &checker,
                &mut context,
            ) {
                return true;
            }
        }
        for bundle in request.RuleSetsWithUniversalRules() {
            if self.CollectMatchingRulesForList::<STOP>(
                bundle.rule_set.UniversalRules(),
                bundle.rule_set,
                bundle.style_sheet_index,
                &checker,
                &mut context,
            ) {
                return true;
            }
        }
        false
    }
    pub fn CollectMatchingRules(
        &mut self,
        request: &MatchRequest<'_, B>,
        parts: Option<&B::PartNames>,
    ) {
        self.CollectMatchingRulesInternal::<false>(request, parts);
    }
    pub fn CheckIfAnyRuleMatches(&mut self, request: &MatchRequest<'_, B>) -> bool {
        self.CollectMatchingRulesInternal::<true>(request, None)
    }
    fn CollectMatchingShadowHostRulesInternal<const STOP: bool>(
        &mut self,
        request: &MatchRequest<'_, B>,
    ) -> bool {
        let checker = self.backend.NewSelectorChecker(
            None,
            &self.pseudo_style_request_,
            self.mode_,
            self.matching_ua_rules_,
        );
        let mut context = ContextWithStyleScopeFrame::new(
            self.backend,
            self.context_,
            request,
            &self.pseudo_style_request_,
            &self.style_recalc_context_,
            self.mode_,
            self.matching_ua_rules_,
            self.matching_rules_from_no_style_sheet_,
        );
        for bundle in request.AllRuleSets() {
            if self.CollectMatchingRulesForList::<STOP>(
                bundle.rule_set.ShadowHostRules(),
                bundle.rule_set,
                bundle.style_sheet_index,
                &checker,
                &mut context,
            ) {
                return true;
            }
            if bundle.rule_set.MustCheckUniversalBucketForShadowHost()
                && self.CollectMatchingRulesForList::<STOP>(
                    bundle.rule_set.UniversalRules(),
                    bundle.rule_set,
                    bundle.style_sheet_index,
                    &checker,
                    &mut context,
                )
            {
                return true;
            }
        }
        false
    }
    pub fn CollectMatchingShadowHostRules(&mut self, request: &MatchRequest<'_, B>) {
        self.CollectMatchingShadowHostRulesInternal::<false>(request);
    }
    pub fn CheckIfAnyShadowHostRuleMatches(&mut self, request: &MatchRequest<'_, B>) -> bool {
        self.CollectMatchingShadowHostRulesInternal::<true>(request)
    }
    pub fn CollectMatchingSlottedRules(&mut self, request: &MatchRequest<'_, B>) {
        let checker = self.backend.NewSelectorChecker(
            None,
            &self.pseudo_style_request_,
            self.mode_,
            self.matching_ua_rules_,
        );
        let mut context = ContextWithStyleScopeFrame::new(
            self.backend,
            self.context_,
            request,
            &self.pseudo_style_request_,
            &self.style_recalc_context_,
            self.mode_,
            self.matching_ua_rules_,
            self.matching_rules_from_no_style_sheet_,
        );
        for bundle in request.AllRuleSets() {
            self.CollectMatchingRulesForList::<false>(
                bundle.rule_set.SlottedPseudoElementRules(),
                bundle.rule_set,
                bundle.style_sheet_index,
                &checker,
                &mut context,
            );
        }
    }
    pub fn CollectMatchingPartPseudoRules(
        &mut self,
        request: &MatchRequest<'_, B>,
        parts: Option<&B::PartNames>,
    ) {
        let checker = self.backend.NewSelectorChecker(
            parts,
            &self.pseudo_style_request_,
            self.mode_,
            self.matching_ua_rules_,
        );
        let mut context = ContextWithStyleScopeFrame::new(
            self.backend,
            self.context_,
            request,
            &self.pseudo_style_request_,
            &self.style_recalc_context_,
            self.mode_,
            self.matching_ua_rules_,
            self.matching_rules_from_no_style_sheet_,
        );
        for bundle in request.AllRuleSets() {
            self.CollectMatchingRulesForList::<false>(
                bundle.rule_set.PartPseudoRules(),
                bundle.rule_set,
                bundle.style_sheet_index,
                &checker,
                &mut context,
            );
        }
    }
    // cpp: element_rule_collector.cc:1367-1460.
    fn DidMatchRule(
        &mut self,
        data: &RuleData<B>,
        layer: u16,
        queries: Option<&Rc<B::ContainerQuerySet>>,
        proximity: u32,
        result: &B::CheckerMatchResult,
        sheet: u32,
    ) {
        let pseudo = self.backend.MatchDynamicPseudo(result);
        if pseudo != PseudoId::kPseudoIdNone && self.GetPseudoId() == PseudoId::kPseudoIdNone {
            if self.mode_ == SelectorCheckerMode::kCollectingCSSRules
                || self.mode_ == SelectorCheckerMode::kCollectingStyleRules
            {
                return;
            }
            if pseudo > PseudoId::kLastTrackedPublicPseudoId {
                return;
            }
            let properties = data.Rule().Properties();
            if [
                PseudoId::kPseudoIdCheckMark,
                PseudoId::kPseudoIdBefore,
                PseudoId::kPseudoIdAfter,
                PseudoId::kPseudoIdExpandIcon,
                PseudoId::kPseudoIdPickerIcon,
                PseudoId::kPseudoIdInterestButton,
            ]
            .contains(&pseudo)
                && !self
                    .backend
                    .PropertiesHasProperty(&properties, CSSPropertyID::kContent)
            {
                return;
            }
            if self.backend.PropertiesIsEmpty(&properties) {
                return;
            }
            self.result_.SetHasPseudoElementStyle(pseudo);
            if IsHighlightPseudoElement(pseudo) {
                let selector = data.Selector();
                let universal = if GetPseudoId(selector.GetPseudoType()) == pseudo {
                    selector.IsLastInComplexSelector()
                } else if let Some(next) = selector.NextSimpleSelector() {
                    next.IsLastInComplexSelector()
                        && GetPseudoId(next.GetPseudoType()) == pseudo
                        && selector.Match() == MatchType::kUniversalTag
                        && selector.TagQName().Prefix() == &AtomicString::from_str("*")
                } else {
                    false
                };
                if !universal || queries.is_some() {
                    self.result_.SetHasNonUniversalHighlightPseudoStyles();
                }
                if !self.matching_ua_rules_ {
                    self.result_.SetHasNonUaHighlightPseudoStyles();
                }
                if queries.is_some() {
                    self.result_.SetHighlightsDependOnSizeContainerQueries();
                }
                if pseudo == PseudoId::kPseudoIdHighlight {
                    self.result_
                        .AddCustomHighlightName(self.backend.MatchCustomHighlightName(result));
                }
            } else if pseudo == PseudoId::kPseudoIdFirstLine && queries.is_some() {
                self.result_.SetFirstLineDependsOnSizeContainerQueries();
            }
        } else {
            if data.IsStartingStyle() {
                self.result_
                    .AddFlags(MatchFlag::kAffectedByStartingStyle as MatchFlags);
            }
            self.matched_rules_
                .push(MatchedRule::new(data, layer, proximity, sheet));
        }
    }
    fn SortMatchedRules(&mut self) {
        if self.matched_rules_.len() > 1 {
            self.matched_rules_
                .sort_unstable_by_key(|r| (r.SortKey(), r.GetPosition()));
        }
    }
    fn AppendCSSOMWrapperForRule(&mut self, rule: &MatchedRule<B>, position: usize) {
        if rule.LinkMatchType() & LinkMatchTypeFromInsideLink(self.inside_link_) == 0 {
            return;
        }
        let scope = self.current_rule_tree_scope_.clone();
        let wrapper = if let Some(ref scope) = scope {
            let mut found = None;
            for sheet in self.backend.ActiveScopeStyleSheets(scope) {
                found = FindStyleRule(
                    self.backend,
                    Some(&self.backend.SheetRuleCollection(&sheet)),
                    rule.Rule(),
                );
                if found.is_some() {
                    break;
                }
            }
            found
        } else {
            Some(self.backend.CreateCSSOMWrapper(rule.Rule(), position))
        };
        self.EnsureRuleList().push(IndexedRule {
            rule: wrapper,
            tree_scope: scope,
            index: rule.SelectorIndex() as i32,
        });
    }
    pub fn SortAndTransferMatchedRules(
        &mut self,
        origin: CascadeOrigin,
        vtt: bool,
        tracker: Option<&mut B::StyleRuleUsageTracker>,
    ) {
        if self.matched_rules_.is_empty() {
            return;
        }
        self.SortMatchedRules();
        if self.mode_ == SelectorCheckerMode::kCollectingStyleRules {
            let rules = self
                .matched_rules_
                .iter()
                .map(|r| r.Rule().clone())
                .collect::<Vec<_>>();
            self.EnsureStyleRuleList().extend(rules);
            return;
        }
        if self.mode_ == SelectorCheckerMode::kCollectingCSSRules {
            // Temporarily move the owning collection to allow appending output.
            let rules = std::mem::take(&mut self.matched_rules_);
            for (position, rule) in rules.iter().enumerate() {
                self.AppendCSSOMWrapperForRule(rule, position);
            }
            self.matched_rules_ = rules;
            return;
        }
        for rule in &self.matched_rules_ {
            let mut data = MatchedPropertiesData::default();
            data.origin = origin;
            data.layer_order = rule.LayerOrder();
            data.set_link_match_type(AdjustLinkMatchType(self.inside_link_, rule.LinkMatchType()));
            data.set_valid_property_filter(
                rule.GetValidPropertyFilter(self.matching_ua_rules_) as u8
            );
            data.set_is_inline_style(vtt);
            let bindings = rule
                .Rule()
                .GetMixinParameterBindings()
                .cloned()
                .map(|b| b as Rc<dyn MatchMixinBindings>);
            self.result_
                .AddMatchedProperties(rule.Rule().Properties(), bindings, data);
        }
        if let Some(tracker) = tracker {
            self.AddMatchedRulesToTracker(tracker);
        }
    }
    pub fn AddMatchedRulesToTracker(&self, tracker: &mut B::StyleRuleUsageTracker) {
        let document = self
            .backend
            .ElementDocument(&self.backend.ContextElement(self.context_));
        for rule in &self.matched_rules_ {
            let sheet = FindStyleSheet(
                self.backend,
                self.current_rule_tree_scope_.as_deref(),
                &document,
                rule.Rule(),
            );
            self.backend.TrackRule(tracker, sheet, rule.Rule());
        }
    }
    pub fn ScopedRuleTreeScope<'s>(
        &'s mut self,
        scope: Rc<B::TreeScope>,
    ) -> ScopedRuleTreeScope<'s, 'a, B> {
        let previous = self.current_rule_tree_scope_.replace(scope);
        ScopedRuleTreeScope {
            collector: self,
            previous,
        }
    }
}
// cpp: element_rule_collector.h:252-265. RAII overrides only the rule's origin
// scope, preserving the MatchResult's cascading/tree-scoped reference scope.
pub struct ScopedRuleTreeScope<'s, 'a, B: ElementRuleCollectorBackend> {
    collector: &'s mut ElementRuleCollector<'a, B>,
    previous: Option<Rc<B::TreeScope>>,
}
impl<'s, 'a, B: ElementRuleCollectorBackend> Deref for ScopedRuleTreeScope<'s, 'a, B> {
    type Target = ElementRuleCollector<'a, B>;
    fn deref(&self) -> &Self::Target {
        self.collector
    }
}
impl<'s, 'a, B: ElementRuleCollectorBackend> DerefMut for ScopedRuleTreeScope<'s, 'a, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.collector
    }
}
impl<B: ElementRuleCollectorBackend> Drop for ScopedRuleTreeScope<'_, '_, B> {
    fn drop(&mut self) {
        self.collector.current_rule_tree_scope_ = self.previous.take();
    }
}

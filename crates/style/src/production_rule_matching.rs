// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//!
//! Production bridge from the translated RuleSet buckets to the retained
//! SelectorChecker used by PersistentDocument.
//!
//! Ordinary element and supported generated-pseudo rules use this bridge as
//! their production candidate and selector-matching path. Rules needing recalc
//! context that is not yet carried here remain explicit compatibility fallbacks.
#![allow(non_snake_case)]

use crate::active_style_sheets::ActiveStyleSheet;
use crate::cascade_layer_map::CascadeLayerMap;
use crate::css_selector::{CSSSelector, GetPseudoId, MatchType, PseudoType, RelationType};
use crate::element_rule_collector::{
    AddRuleSetToRuleSetGroupList, ElementRuleCollector, MatchRequest,
};
use crate::persistent_selector::{PersistentPartNames, PersistentSelectorService};
use crate::production_element_rule_collector::{
    ProductionElementResolveContext, ProductionElementRuleCollectorRuntime,
    ProductionStyleRecalcContext, ProductionTreeScope,
};
use crate::production_rule_context::{CanMatchProductionScopeChain, ProductionPseudoRequest};
use crate::production_style_sheet::{Backend, ProductionStyleSheet};
use crate::resolver::{
    cascade_origin::CascadeOrigin,
    match_result::{MatchResult, MatchedProperties},
    production_container_runtime::ProductionContainerRuntime,
};
use crate::rule_set::{RuleData, RuleSet};
use dom::persistent_document::DOMNodeType;
use dom::Document;
use foundation::{AtomicString, EInsideLink};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone)]
pub(crate) struct ProductionTransferredRule {
    pub sheet_index: usize,
    pub position: u32,
    pub specificity: u32,
    pub proximity: u32,
    pub matched: MatchedProperties,
}

pub(crate) struct ProductionRuleSetInput<'a> {
    pub sheet_index: usize,
    pub rule_set: Rc<RuleSet<Backend>>,
    pub routing: &'a ProductionRuleRouting,
    /// Parent element of the CSSStyleSheet owner (or adopted-sheet shadow
    /// host), matching ScopedStyleResolver::ImplicitScopeTrigger.
    pub implicit_scope_trigger: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProductionRuleMatchReport {
    pub collector_invocations: usize,
    pub transferred_property_sets: usize,
    pub candidates_considered: usize,
    pub skipped_starting_style: usize,
    pub skipped_pseudo: usize,
    pub skipped_unsupported_selector: usize,
    pub skipped_scope: usize,
    pub skipped_container_query: usize,
    pub selector_matches: usize,
}

pub(crate) struct ProductionRuleMatchCollection {
    pub report: ProductionRuleMatchReport,
    pub rules: Vec<ProductionTransferredRule>,
}

/// cpp: StyleEngine::ApplyRuleSetInvalidationForElement / AnyRuleCausesInvalidation.
/// Only changed old/new RuleSets participate. The translated collector stops at
/// its first match and retains dynamic-state flags even for a nonmatching rule.
/// UA implementation elements use the collector's cross-tree bucket. Author
/// scope, :host, ::part and ::slotted retain the owner's separate path.
pub(crate) fn InvalidateProductionRuleSets(
    rule_sets: &[Rc<RuleSet<Backend>>],
    document: &Document,
    selectors: &mut PersistentSelectorService,
    elements: &[usize],
) -> Option<HashSet<usize>> {
    use crate::resolver::match_flags::MatchFlag;
    let mut trace = browser_tracing::span("style", "StyleEngine.InvalidateProductionRuleSets");
    // cpp: StyleEngine::ApplyRuleSetChanges passes HeapHashSet<Member<RuleSet>>.
    // Several CSSStyleSheet owners can share one immutable contents/RuleSet.
    // This is an existential invalidation check, not ordered cascade matching:
    // probing that same RuleSet again cannot add a match or a dynamic flag.
    let mut seen = HashSet::with_capacity(rule_sets.len());
    let unique_rule_sets = rule_sets
        .iter()
        .filter(|rules| seen.insert(Rc::as_ptr(rules) as usize))
        .collect::<Vec<_>>();
    trace.set("input_rule_sets", rule_sets.len() as f64);
    trace.set("unique_rule_sets", unique_rule_sets.len() as f64);
    trace.set(
        "duplicate_rule_sets",
        (rule_sets.len() - unique_rule_sets.len()) as f64,
    );
    trace.set("elements", elements.len() as f64);
    let unsupported = if document.GetCompatibilityMode()
        == dom::persistent_document::CompatibilityMode::kQuirks
    {
        Some(("quirks", String::new()))
    } else {
        unique_rule_sets.iter().find_map(|rules| {
            rules.AllRuleData().iter().find_map(|data| {
                // CheckIfAnyRuleMatches returns before cascade/pseudo output
                // materialization (ElementRuleCollector's STOP branch). Only the
                // selector checker's capabilities constrain invalidation here;
                // HasSupportedPseudoTarget belongs to rule-output routing below.
                let reason = if rules.ScopeForRule(data).is_some() {
                    Some("scope")
                } else if ProductionUnsupportedParsedFeature(
                    &data.Rule().Selectors().CopySelectors(),
                )
                .is_some()
                {
                    Some("selector")
                } else if data.Selector().SimpleSelectors().any(|selector| {
                    matches!(
                        selector.GetPseudoType(),
                        PseudoType::kPseudoHost
                            | PseudoType::kPseudoHostContext
                            | PseudoType::kPseudoPart
                            | PseudoType::kPseudoSlotted
                    )
                }) {
                    Some("tree_scope")
                } else {
                    None
                };
                reason.map(|reason| {
                    (
                        reason,
                        if browser_tracing::enabled() {
                            data.Selector().SelectorText().Utf8()
                        } else {
                            String::new()
                        },
                    )
                })
            })
        })
    };
    if let Some((reason, selector)) = unsupported {
        if browser_tracing::enabled() {
            browser_tracing::instant_with_text(
                "style",
                "RuleSetInvalidationFallback",
                &format!("reason={reason} selector={selector}"),
                &[],
            );
        }
        return None;
    }
    // cpp: StyleEngine::ApplyRuleSetInvalidationForTreeScope descends into
    // shadow trees when changed rules address UA implementation elements.
    // ElementRuleCollector checks the UA bucket before rejecting ordinary
    // selectors outside the request's TreeScope.
    let crosses_ua_shadow = unique_rule_sets
        .iter()
        .any(|rules| rules.HasUAShadowPseudoElementRules());
    let mut groups = Vec::new();
    for rules in unique_rule_sets {
        AddRuleSetToRuleSetGroupList(rules.clone(), &mut groups);
    }
    trace.set("groups", groups.len() as f64);
    let session = selectors.StyleMatchSession(document);
    let runtime = ProductionElementRuleCollectorRuntime::new(&session);
    let mut invalidated = HashSet::new();
    for &index in elements {
        if !crosses_ua_shadow && document.TreeScopeRoot(index) != document.Root() {
            continue;
        }
        let context = ProductionElementResolveContext { element: index };
        let recalc = ProductionStyleRecalcContext::ForElement(document, index, None);
        let filter = runtime.SelectorFilterForElement(index);
        let mut result = MatchResult::default();
        let mut collector = ElementRuleCollector::<Backend, _>::new(
            &runtime,
            &context,
            &recalc,
            &filter,
            &mut result,
            EInsideLink::kNotInsideLink,
        );
        collector.BeginAddingAuthorRulesForTreeScope(Rc::new(ProductionTreeScope(document.Root())));
        for group in &groups {
            let request = MatchRequest::new(group, Some(Rc::new(document.Root())), None);
            if collector.CheckIfAnyRuleMatches(&request)
                || [
                    MatchFlag::kAffectedByDrag,
                    MatchFlag::kAffectedByFocusWithin,
                    MatchFlag::kAffectedByHover,
                    MatchFlag::kAffectedByActive,
                ]
                .into_iter()
                .any(|flag| collector.MatchedResult().HasFlag(flag))
            {
                invalidated.insert(index);
                // The retained output stores ::placeholder's computed style
                // on its originating control. Resolve that same owner when
                // Chromium's collector invalidates its real UA element.
                if document.ShadowPseudoId(index) == Some("-webkit-input-placeholder") {
                    if let Some(host) = document.OwnerShadowHost(index) {
                        invalidated.insert(host);
                    }
                }
                break;
            }
        }
    }
    Some(invalidated)
}

/// Production SelectorChecker capability boundary.
///
/// The generic persistent-query boundary deliberately stays narrower than the
/// style resolver. Autofill has a retained DOM owner and is implemented by
/// SelectorChecker, so the production resolver accepts those four typed
/// pseudo classes while continuing to delegate every other simple selector to
/// the shared capability gate. Selector-list arguments are checked
/// recursively; this keeps `:is(input:autofill)` behind the same typed gate as
/// a direct `input:autofill` selector.
pub(crate) fn ProductionUnsupportedParsedFeature(
    selectors: &[CSSSelector],
) -> Option<std::string::String> {
    use PseudoType::*;

    for selector in selectors {
        if matches!(
            selector.GetPseudoType(),
            kPseudoAutofill
                | kPseudoWebKitAutofill
                | kPseudoAutofillPreviewed
                | kPseudoAutofillSelected
        ) {
            continue;
        }
        if let Some(list) = selector.SelectorList() {
            // These are the selector-list-bearing pseudo classes accepted by
            // CSSSelectorParser's persistent capability boundary. Validate
            // the outer typed pseudo here so an autofill pseudo nested in an
            // otherwise unsupported functional pseudo cannot open the gate.
            if !matches!(
                selector.GetPseudoType(),
                kPseudoIs
                    | kPseudoAny
                    | kPseudoWhere
                    | kPseudoNot
                    | kPseudoHas
                    | kPseudoNthChild
                    | kPseudoNthLastChild
                    | kPseudoNthOfType
                    | kPseudoNthLastOfType
                    | kPseudoHost
                    | kPseudoHostContext
                    | kPseudoSlotted
                    | kPseudoParent
            ) {
                return Some(selector.Value().Utf8());
            }
            if let Some(feature) = ProductionUnsupportedParsedFeature(&list.CopySelectors()) {
                return Some(feature);
            }
            continue;
        }
        if let Some(feature) =
            crate::parser::css_selector_parser::CSSSelectorParser::UnsupportedParsedFeature(
                std::slice::from_ref(selector),
            )
        {
            return Some(feature);
        }
    }
    None
}

/// Build the StyleScope identity -> stylesheet-owner trigger table used by
/// StyleScopeData::TriggersScope. Multiple CSSStyleSheet clients may share one
/// RuleSet, so every trigger remains in the value vector.
pub(crate) fn ProductionImplicitScopeRoots<'a>(
    inputs: impl IntoIterator<Item = (&'a RuleSet<Backend>, Option<usize>)>,
) -> HashMap<usize, Vec<usize>> {
    let mut implicit_scope_roots = HashMap::<usize, Vec<usize>>::new();
    for (rule_set, trigger) in inputs {
        let Some(trigger) = trigger else { continue };
        for interval in rule_set.ScopeIntervals() {
            let Some(scope_root) = interval.value.as_deref() else {
                continue;
            };
            let mut scope = Some(scope_root);
            let mut has_implicit_scope = false;
            while let Some(current) = scope {
                if current.IsImplicit() {
                    has_implicit_scope = true;
                }
                scope = current.Parent();
            }
            if has_implicit_scope {
                let roots = implicit_scope_roots
                    .entry(scope_root as *const _ as usize)
                    .or_default();
                if !roots.contains(&trigger) {
                    roots.push(trigger);
                }
            }
        }
    }
    implicit_scope_roots
}

/// The source StyleResolver shadow pass that selected a scoped RuleSet.
/// Scope selection stays with the caller; this bridge owns the translated
/// ElementRuleCollector invocation and matched-property transfer.
pub(crate) enum ProductionShadowRulePass {
    Host,
    Slotted,
    Part(PersistentPartNames),
    /// StyleResolver::MatchElementScopeRules collects ordinary element rules
    /// and :host::part rules into one sort/transfer batch.
    ElementScope(Option<PersistentPartNames>),
}

fn PartAttributeNames(document: &Document, element: usize) -> Option<PersistentPartNames> {
    if !document.IsInShadowTree(element) {
        return None;
    }
    let value = &document
        .Node(element)
        .Attributes()
        .iter()
        .find(|attribute| attribute.local_name == "part" && attribute.namespace_uri.is_empty())?
        .value;
    // SpaceSplitString/DOMTokenList use Infra ASCII whitespace. Rust's
    // split_ascii_whitespace additionally accepts vertical tab, which Blink
    // deliberately does not treat as a token separator.
    let names = value
        .split(|character| matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
        .filter(|name| !name.is_empty())
        .map(AtomicString::from_str)
        .collect::<Vec<_>>();
    (!names.is_empty()).then(|| PersistentPartNames::FromNames(names))
}

/// cpp: resolver/style_resolver.cc:946-955,969-1030;
///      part_names.cc:22-48.
///
/// Returns each outer TreeScope that can address `element` through ::part,
/// paired with the exact part-name set visible in that scope. Every host's
/// retained exportparts NamesMap is queued after matching its own scope and
/// before advancing to its owner host. An absent or empty map terminates the
/// chain; names are never implicitly forwarded.
pub(crate) fn ProductionPartRuleScopes(
    document: &Document,
    element: usize,
) -> Vec<(usize, PersistentPartNames)> {
    let Some(mut current_names) = PartAttributeNames(document, element) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut owner_shadow_host = document.OwnerShadowHost(element);
    while let Some(host) = owner_shadow_host {
        result.push((document.TreeScopeRoot(host), current_names.clone()));
        let Some(map) = document.PartNamesMap(host).filter(|map| !map.IsEmpty()) else {
            break;
        };
        current_names.PushMap(map);
        owner_shadow_host = document.OwnerShadowHost(host);
    }
    result
}

fn RuleIdentity(data: &RuleData<Backend>) -> usize {
    Rc::as_ptr(data.Rule()) as usize
}

fn AppendRuleIdentities(bucket: &[RuleData<Backend>], identities: &mut HashSet<usize>) {
    identities.extend(bucket.iter().map(RuleIdentity));
}

fn HasSupportedPseudoTarget(data: &RuleData<Backend>) -> bool {
    let mut saw_pseudo = false;
    for selector in data.Selector().SimpleSelectors() {
        if selector.Match() == MatchType::kPseudoElement {
            // A ProductionPseudoRequest represents one pseudo on an element.
            // Nested ::before::marker/::after::marker needs the generated
            // pseudo's style as its originating style and therefore remains
            // on DocumentStyleEngine's explicit nested request path.
            if saw_pseudo {
                return false;
            }
            saw_pseudo = true;
            // cpp: selector_checker.cc:3239-3250,3324-3350 and
            // element_rule_collector.cc:826-834. UA-shadow implementation
            // elements are real retained elements. Their typed custom pseudo
            // selector is matched against Element::ShadowPseudoId(), and the
            // kUAShadow relation then crosses to the originating host. Keep
            // both source custom-element categories on that path; the DOM
            // owner decides whether a concrete implementation element exists.
            if matches!(
                selector.GetPseudoType(),
                PseudoType::kPseudoWebKitCustomElement | PseudoType::kPseudoBlinkInternalElement
            ) {
                return true;
            }
            if !matches!(
                selector.GetPseudoType(),
                crate::css_selector::PseudoType::kPseudoSlotted
                    | crate::css_selector::PseudoType::kPseudoPart
            ) && !matches!(
                GetPseudoId(selector.GetPseudoType()),
                PseudoId::kPseudoIdBefore
                    | PseudoId::kPseudoIdAfter
                    | PseudoId::kPseudoIdFirstLetter
                    | PseudoId::kPseudoIdPlaceholder
                    | PseudoId::kPseudoIdSelection
                    | PseudoId::kPseudoIdSearchText
                    | PseudoId::kPseudoIdTargetText
                    | PseudoId::kPseudoIdHighlight
                    | PseudoId::kPseudoIdSpellingError
                    | PseudoId::kPseudoIdGrammarError
            ) {
                return false;
            }
        }
        if selector.Relation() != RelationType::kSubSelector {
            break;
        }
    }
    true
}

/// Revision-scoped routing plan built beside the cached RuleSet. Element hot
/// paths only query this identity map after selecting candidates from buckets.
pub(crate) struct ProductionRuleRouting {
    authoritative: HashMap<usize, Option<usize>>,
    projected_by_identity: HashMap<usize, usize>,
    authoritative_rule_indices: HashSet<usize>,
    compatibility: HashSet<usize>,
    // Immutable classification from this revision/media-owned RuleSet. Dynamic
    // collector, matching and transfer statistics are still gathered per query.
    fallback_report: ProductionRuleMatchReport,
    ordinary_signature_complete: bool,
}

impl ProductionRuleRouting {
    pub(crate) fn Build(rule_set: &RuleSet<Backend>, sheet: &ProductionStyleSheet) -> Self {
        let mut special_rules = HashSet::new();
        for bucket in [
            rule_set.CuePseudoRules(),
            rule_set.ScrollbarRules(),
            rule_set.ActiveViewTransitionRules(),
            rule_set.UnboundedPseudoClassRules(),
        ] {
            AppendRuleIdentities(bucket, &mut special_rules);
        }
        #[derive(Default)]
        struct Route {
            projected: Option<usize>,
            expected: usize,
            selectors: BTreeSet<u32>,
            fallback: bool,
        }
        let mut routes = HashMap::<usize, Route>::new();
        for data in rule_set.AllRuleData() {
            let identity = RuleIdentity(data);
            let projected = sheet.ProjectedRuleIndexForRetainedStyleRule(data.Rule());
            if projected.is_none() && data.Rule().GetMixinParameterBindings().is_none() {
                continue;
            }
            let route = routes.entry(identity).or_insert_with(|| Route {
                projected,
                expected: data.Rule().Selectors().ComplexSelectors().count(),
                ..Default::default()
            });
            route.selectors.insert(data.SelectorIndex());
            let unsupported_scope = rule_set
                .ScopeForRule(data)
                .is_some_and(|scope| !CanMatchProductionScopeChain(&scope));
            route.fallback |= !HasSupportedPseudoTarget(data)
                || special_rules.contains(&identity)
                || projected.is_some_and(|projected| {
                    sheet
                        .RetainedDeclarationsForProjectedRule(projected)
                        .is_none()
                })
                || unsupported_scope
                || ProductionUnsupportedParsedFeature(&data.Rule().Selectors().CopySelectors())
                    .is_some();
        }
        let projected_by_identity = routes
            .iter()
            .filter_map(|(&identity, route)| route.projected.map(|projected| (identity, projected)))
            .collect::<HashMap<_, _>>();
        let authoritative = routes
            .iter()
            .filter_map(|(&identity, route)| {
                (!route.fallback && route.selectors.len() == route.expected)
                    .then_some((identity, route.projected))
            })
            .collect::<HashMap<_, _>>();
        let authoritative_rule_indices =
            authoritative.values().filter_map(|value| *value).collect();
        let compatibility = routes
            .keys()
            .filter(|identity| !authoritative.contains_key(identity))
            .copied()
            .collect();
        let mut routing = Self {
            authoritative,
            projected_by_identity,
            authoritative_rule_indices,
            compatibility,
            fallback_report: ProductionRuleMatchReport::default(),
            ordinary_signature_complete: false,
        };
        routing.fallback_report = routing.StaticFallbackReport(rule_set);
        // The ordinary compatibility probe does not encode highlight names,
        // starting-style recalc state, or cross-tree/scope contexts. Keep all
        // those cases conservative, even when the special rule is unmatched.
        // An empty signature may survive a rule revision only when both versions
        // exclude global sheet inputs (for example @function in inline values).
        // RuleAt includes pre-import layers, imports and namespaces as well as
        // child rules; inactive groups and all non-style variants are rejected.
        let flat_ordinary_sheet = sheet.diagnostics.is_empty()
            && (0..sheet.contents.RuleCount()).all(|index| {
                sheet.contents.RuleAt(index).AsStyle().is_some_and(|rule| {
                    rule.ChildRules().is_none_or(|children| children.is_empty())
                })
            });
        routing.ordinary_signature_complete = flat_ordinary_sheet
            && rule_set.AllRuleData().iter().all(|data| {
                let identity = RuleIdentity(data);
                routing.authoritative.contains_key(&identity)
                    && routing.ProjectedRuleIndex(identity) == Some(data.GetPosition() as usize)
                    && data.Rule().Selectors().ComplexSelectors().count() == 1
                    && !data.IsStartingStyle()
                    && rule_set.ScopeForRule(data).is_none()
                    && rule_set.ContainerQueriesForRule(data).is_none()
                    && data.Rule().GetMixinParameterBindings().is_none()
                    && !special_rules.contains(&identity)
                    && data.Selector().SimpleSelectors().all(|selector| {
                        selector.Match() != MatchType::kPseudoElement
                            && selector.SelectorList().is_none()
                            && !matches!(
                                selector.GetPseudoType(),
                                PseudoType::kPseudoHost | PseudoType::kPseudoHostContext
                            )
                    })
            });
        routing
    }

    fn StaticFallbackReport(&self, rule_set: &RuleSet<Backend>) -> ProductionRuleMatchReport {
        let mut report = ProductionRuleMatchReport::default();
        let mut reported_fallback_rules = HashSet::new();
        let mut reported_scopes = HashSet::new();
        let mut reported_container_sets = HashSet::new();
        for data in rule_set.AllRuleData() {
            let identity = RuleIdentity(data);
            let report_key = self
                .ProjectedRuleIndex(identity)
                .map_or((false, identity), |projected| (true, projected));
            if self.authoritative.contains_key(&identity)
                || !reported_fallback_rules.insert(report_key)
            {
                continue;
            }
            if !HasSupportedPseudoTarget(data) {
                report.skipped_pseudo += 1;
            } else if ProductionUnsupportedParsedFeature(&data.Rule().Selectors().CopySelectors())
                .is_some()
            {
                report.skipped_unsupported_selector += 1;
            } else if let Some(scope) = rule_set
                .ScopeForRule(data)
                .filter(|scope| !CanMatchProductionScopeChain(scope))
            {
                if reported_scopes.insert(Rc::as_ptr(&scope) as usize) {
                    report.skipped_scope += 1;
                }
            } else if let Some(queries) = rule_set.ContainerQueriesForRule(data) {
                if reported_container_sets.insert(Rc::as_ptr(&queries) as usize) {
                    report.skipped_container_query += 1;
                }
            }
        }
        report
    }

    pub(crate) fn OrdinarySignatureComplete(&self) -> bool {
        self.ordinary_signature_complete
    }

    pub(crate) fn IsAuthoritativeProjectedRule(&self, rule_index: usize) -> bool {
        self.authoritative_rule_indices.contains(&rule_index)
    }

    pub(crate) fn AuthoritativeRuleCount(&self) -> usize {
        self.authoritative.len()
    }

    pub(crate) fn IsAuthoritativeRetainedRule(
        &self,
        rule: &Rc<crate::style_rule::StyleRule<Backend>>,
    ) -> bool {
        self.authoritative
            .contains_key(&(Rc::as_ptr(rule) as usize))
    }

    pub(crate) fn RequiresCompatibilityFallback(
        &self,
        rule: &Rc<crate::style_rule::StyleRule<Backend>>,
    ) -> bool {
        self.compatibility.contains(&(Rc::as_ptr(rule) as usize))
    }

    fn ProjectedRuleIndex(&self, identity: usize) -> Option<usize> {
        self.projected_by_identity.get(&identity).copied()
    }
}

/// Runs the production element through translated RuleSet buckets and writes
/// the collector's transferred property sets directly into `cascade_result`.
///
/// Inputs are one cascade origin in active-sheet order. Rules whose recalc
/// context is not represented by the element or supported pseudo request are removed
/// before SortAndTransferMatchedRules and remain owned by the caller's
/// explicit compatibility route.
pub(crate) fn CollectProductionRuleMatchesInto<'d>(
    inputs: &[ProductionRuleSetInput<'_>],
    document: &'d Document,
    element_index: usize,
    selectors: &mut PersistentSelectorService,
    origin: CascadeOrigin,
    pseudo: &ProductionPseudoRequest,
    container_runtime: Option<&'d ProductionContainerRuntime<'d>>,
    cascade_result: &mut MatchResult,
) -> ProductionRuleMatchCollection {
    let node = document.Node(element_index);
    if node.Type() != DOMNodeType::kElement || inputs.is_empty() {
        return ProductionRuleMatchCollection {
            report: ProductionRuleMatchReport::default(),
            rules: Vec::new(),
        };
    }

    // cpp: selector_checker.cc:MatchesUAShadowElement. ::placeholder is not a
    // virtual pseudo request. Blink matches the retained implementation
    // element in the text control's UA shadow tree, then crosses kUAShadow to
    // the host for the originating compound.
    let placeholder_request;
    let (matching_element, matching_pseudo) = if pseudo.id == PseudoId::kPseudoIdPlaceholder {
        let Some(placeholder) =
            document.UAShadowPseudoElement(element_index, "-webkit-input-placeholder")
        else {
            return ProductionRuleMatchCollection {
                report: ProductionRuleMatchReport::default(),
                rules: Vec::new(),
            };
        };
        placeholder_request = ProductionPseudoRequest::Element();
        (placeholder, &placeholder_request)
    } else {
        (element_index, pseudo)
    };

    let mut report = ProductionRuleMatchReport::default();
    for input in inputs {
        let cached = &input.routing.fallback_report;
        report.skipped_pseudo += cached.skipped_pseudo;
        report.skipped_unsupported_selector += cached.skipped_unsupported_selector;
        report.skipped_scope += cached.skipped_scope;
        report.skipped_container_query += cached.skipped_container_query;
    }

    let mut groups = Vec::new();
    for input in inputs {
        AddRuleSetToRuleSetGroupList(input.rule_set.clone(), &mut groups);
    }
    let author_layer_map = (origin == CascadeOrigin::kAuthor).then(|| {
        let active = inputs
            .iter()
            .map(|input| ActiveStyleSheet::new(Rc::new(()), Some(input.rule_set.clone())))
            .collect::<Vec<_>>();
        Rc::new(CascadeLayerMap::new(&active))
    });

    // cpp: scoped_style_resolver.cc:453-485. Detect every implicit scope,
    // including implicit parents that do not directly own a StyleRule, and
    // attach the trigger set to the interval's full scope chain. The same
    // StyleScope may be shared by multiple CSSStyleSheet clients, so this is a
    // vector rather than a single reconstructed root.
    let implicit_scope_roots = ProductionImplicitScopeRoots(
        inputs
            .iter()
            .map(|input| (input.rule_set.as_ref(), input.implicit_scope_trigger)),
    );

    let session = selectors.StyleMatchSession(document);
    let runtime = ProductionElementRuleCollectorRuntime::with_production_inputs(
        &session,
        author_layer_map,
        implicit_scope_roots,
        container_runtime,
    );
    let context = ProductionElementResolveContext {
        element: matching_element,
    };
    let recalc = ProductionStyleRecalcContext::ForStyleRequest(
        document,
        element_index,
        pseudo.id,
        container_runtime.and_then(|runtime| {
            runtime.NearestSizeContainerForStyleRequest(element_index, pseudo.id)
        }),
    );
    let first_result = cascade_result.GetMatchedProperties().len();
    let selector_filter = runtime.SelectorFilterForElement(context.element);
    let mut collector = ElementRuleCollector::<Backend, _>::new(
        &runtime,
        &context,
        &recalc,
        &selector_filter,
        cascade_result,
        EInsideLink::kNotInsideLink,
    );
    collector.SetPseudoElementStyleRequest(matching_pseudo);
    collector.SetMatchingUARules(origin == CascadeOrigin::kUserAgent);
    if origin == CascadeOrigin::kAuthor {
        // All active document sheets belong to the document tree scope.  The
        // translated StyleCascade consults this exact MatchResult ownership
        // when applying author declarations; the previous manual cascade did
        // not observe the missing scope entry.
        collector.BeginAddingAuthorRulesForTreeScope(Rc::new(ProductionTreeScope(document.Root())));
    }
    for group in &groups {
        let request = MatchRequest::new(group, None, None);
        collector.CollectMatchingRules(&request, None);
        report.collector_invocations += 1;
    }
    report.candidates_considered = collector.CandidatesConsidered();
    report.selector_matches = collector.MatchedRulesForTest().len();
    collector.RetainMatchedRules(|matched| {
        inputs
            .get(matched.StyleSheetIndex())
            .is_some_and(|input| input.routing.IsAuthoritativeRetainedRule(matched.Rule()))
    });
    collector.SortAndTransferMatchedRules(origin, false, None);
    report.transferred_property_sets =
        collector.MatchedResult().GetMatchedProperties().len() - first_result;

    let rules = collector
        .MatchedRulesForTest()
        .iter()
        .zip(collector.MatchedResult().GetMatchedProperties()[first_result..].iter())
        .map(|(rule, matched)| ProductionTransferredRule {
            sheet_index: inputs[rule.StyleSheetIndex()].sheet_index,
            position: rule.PositionInStyleSheet(),
            specificity: rule.Specificity(),
            proximity: rule.Proximity(),
            matched: matched.clone(),
        })
        .collect();
    ProductionRuleMatchCollection { report, rules }
}

/// cpp: resolver/style_resolver.cc:715-802,862-879,929-1031;
/// element_rule_collector.cc:1085-1165.
///
/// Runs one already-selected shadow-tree scope through the exact translated
/// special bucket. `scope_root` is the TreeScope that owns `inputs`: the
/// attached ShadowRoot for `:host`/`::slotted`, or the outer scope root for
/// `::part`. Selecting the relevant scoped resolver remains the StyleResolver
/// owner's job, as it does in Blink.
pub(crate) fn CollectProductionShadowRuleMatchesInto(
    inputs: &[ProductionRuleSetInput<'_>],
    document: &Document,
    element_index: usize,
    selectors: &mut PersistentSelectorService,
    scope_root: usize,
    pass: &ProductionShadowRulePass,
    container_runtime: Option<&ProductionContainerRuntime<'_>>,
    cascade_result: &mut MatchResult,
) -> ProductionRuleMatchCollection {
    let node = document.Node(element_index);
    if node.Type() != DOMNodeType::kElement || inputs.is_empty() {
        return ProductionRuleMatchCollection {
            report: ProductionRuleMatchReport::default(),
            rules: Vec::new(),
        };
    }
    debug_assert_eq!(document.TreeScopeRoot(scope_root), scope_root);

    let mut groups = Vec::new();
    for input in inputs {
        AddRuleSetToRuleSetGroupList(input.rule_set.clone(), &mut groups);
    }
    let active = inputs
        .iter()
        .map(|input| ActiveStyleSheet::new(Rc::new(()), Some(input.rule_set.clone())))
        .collect::<Vec<_>>();
    let author_layer_map = Some(Rc::new(CascadeLayerMap::new(&active)));

    let implicit_scope_roots = ProductionImplicitScopeRoots(
        inputs
            .iter()
            .map(|input| (input.rule_set.as_ref(), input.implicit_scope_trigger)),
    );
    let session = selectors.StyleMatchSession(document);
    let runtime = ProductionElementRuleCollectorRuntime::with_production_inputs(
        &session,
        author_layer_map,
        implicit_scope_roots,
        container_runtime,
    );
    let context = ProductionElementResolveContext {
        element: element_index,
    };
    let recalc = ProductionStyleRecalcContext::ForElement(
        document,
        element_index,
        container_runtime.and_then(|runtime| runtime.NearestSizeContainer(element_index)),
    );
    let first_result = cascade_result.GetMatchedProperties().len();
    let selector_filter = runtime.SelectorFilterForElement(context.element);
    let mut collector = ElementRuleCollector::<Backend, _>::new(
        &runtime,
        &context,
        &recalc,
        &selector_filter,
        cascade_result,
        EInsideLink::kNotInsideLink,
    );
    // StyleResolver::MatchElementScopeRules selects the corresponding source
    // RuleSets but begins the transfer/matching scope on the instance itself.
    // Conflating these scopes silently drops SVG use rules in fallback recalc.
    let matching_scope = if matches!(pass, ProductionShadowRulePass::ElementScope(_)) {
        document.TreeScopeRoot(element_index)
    } else {
        scope_root
    };
    collector.BeginAddingAuthorRulesForTreeScope(Rc::new(ProductionTreeScope(matching_scope)));
    for group in &groups {
        let request = MatchRequest::new(group, Some(Rc::new(matching_scope)), None);
        match pass {
            ProductionShadowRulePass::Host => collector.CollectMatchingShadowHostRules(&request),
            ProductionShadowRulePass::Slotted => collector.CollectMatchingSlottedRules(&request),
            ProductionShadowRulePass::Part(names) => {
                collector.CollectMatchingPartPseudoRules(&request, Some(names))
            }
            ProductionShadowRulePass::ElementScope(names) => {
                collector.CollectMatchingRules(&request, None);
                if let Some(names) = names {
                    collector.CollectMatchingPartPseudoRules(&request, Some(names));
                }
            }
        }
    }

    let mut report = ProductionRuleMatchReport {
        collector_invocations: groups.len(),
        candidates_considered: collector.CandidatesConsidered(),
        selector_matches: collector.MatchedRulesForTest().len(),
        ..Default::default()
    };
    collector.RetainMatchedRules(|matched| {
        inputs
            .get(matched.StyleSheetIndex())
            .is_some_and(|input| input.routing.IsAuthoritativeRetainedRule(matched.Rule()))
    });
    collector.SortAndTransferMatchedRules(CascadeOrigin::kAuthor, false, None);
    report.transferred_property_sets =
        collector.MatchedResult().GetMatchedProperties().len() - first_result;
    let rules = collector
        .MatchedRulesForTest()
        .iter()
        .zip(collector.MatchedResult().GetMatchedProperties()[first_result..].iter())
        .map(|(rule, matched)| ProductionTransferredRule {
            sheet_index: inputs[rule.StyleSheetIndex()].sheet_index,
            position: rule.PositionInStyleSheet(),
            specificity: rule.Specificity(),
            proximity: rule.Proximity(),
            matched: matched.clone(),
        })
        .collect();
    ProductionRuleMatchCollection { report, rules }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production_style_sheet::{ParseStyleSheet, ProductionMediaQueryEvaluator};
    use crate::{
        media_queries::{production_container_query::ContainerState, MediaValuesCachedData},
        parser::css_parser_mode::CSSParserMode,
        resolver::{
            production_container_runtime::ProductionContainerRuntime,
            production_style_builder::custom_properties::CustomProperties,
        },
    };
    use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMShadowRootMode};
    use foundation::{AtomicString, Persistent, String as WTFString};
    use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};

    fn parse(css: &str) -> crate::production_style_sheet::ProductionStyleSheet {
        ParseStyleSheet(
            &WTFString::from_utf16(&css.encode_utf16().collect::<Vec<_>>()),
            CSSParserMode::kHTMLStandardMode,
        )
    }

    #[test]
    fn production_rule_set_buckets_feed_retained_selector_checker() {
        let sheet = parse(
            ".hit{color:red} #target > span.hit[data-k='v']{display:block} \
             .absent.hit{width:1px} @media (min-width:100px){span.hit{height:3px}}",
        );
        let medium = ProductionMediaQueryEvaluator::Screen(800.0, 600.0);
        let rule_set = Rc::new(sheet.BuildRuleSet(&medium));

        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let html = document.CreateElementDefault(DOMNamespace::kHTML, "html".into());
        document.AppendChild(document.Root(), html);
        let parent = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "id".into(),
                value: "target".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(html, parent);
        let element = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![
                DOMAttribute {
                    local_name: "class".into(),
                    value: "hit".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "data-k".into(),
                    value: "v".into(),
                    ..Default::default()
                },
            ],
        );
        document.AppendChild(parent, element);

        let mut selectors = PersistentSelectorService::default();
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut result = MatchResult::default();
        let collection = CollectProductionRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set: rule_set.clone(),
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            element,
            &mut selectors,
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Element(),
            None,
            &mut result,
        );
        let report = &collection.report;

        assert_eq!(report.skipped_starting_style, 0);
        assert_eq!(report.skipped_scope, 0);
        assert_eq!(report.skipped_container_query, 0);
        assert_eq!(report.collector_invocations, 1, "{report:?}");
        assert_eq!(report.transferred_property_sets, 3, "{report:?}");
        assert_eq!(report.selector_matches, 3, "{report:?}");
        assert!(report.candidates_considered > report.selector_matches);
        assert_eq!(collection.rules.len(), 3);
        assert_eq!(result.GetMatchedProperties().len(), 3);
        assert_eq!(
            routing.fallback_report,
            routing.StaticFallbackReport(&rule_set)
        );
        // Static diagnostics preserve multiplicity for repeated collection,
        // including fallback rules that never enter the translated cascade.
        let fallback_sheet = parse(".hit::cue{color:red}.hit::-webkit-scrollbar{width:7px}");
        let fallback_set = Rc::new(fallback_sheet.BuildRuleSet(&medium));
        let fallback = ProductionRuleRouting::Build(&fallback_set, &fallback_sheet);
        assert!(fallback.fallback_report.skipped_pseudo > 0);
        assert_eq!(
            fallback.fallback_report,
            fallback.StaticFallbackReport(&fallback_set)
        );
        for _ in 0..3 {
            let mut output = MatchResult::default();
            let actual = CollectProductionRuleMatchesInto(
                &[ProductionRuleSetInput {
                    sheet_index: 0,
                    rule_set: fallback_set.clone(),
                    routing: &fallback,
                    implicit_scope_trigger: None,
                }],
                document,
                element,
                &mut selectors,
                CascadeOrigin::kAuthor,
                &ProductionPseudoRequest::Element(),
                None,
                &mut output,
            )
            .report;
            assert_eq!(
                actual.skipped_pseudo,
                fallback.fallback_report.skipped_pseudo
            );
            assert_eq!(
                actual.skipped_unsupported_selector,
                fallback.fallback_report.skipped_unsupported_selector
            );
            assert_eq!(actual.skipped_scope, fallback.fallback_report.skipped_scope);
            assert_eq!(
                actual.skipped_container_query,
                fallback.fallback_report.skipped_container_query
            );
        }

        for (transferred, matched) in collection.rules.iter().zip(result.GetMatchedProperties()) {
            assert!(Rc::ptr_eq(
                &transferred.matched.properties,
                &matched.properties
            ));
            assert_ne!(matched.properties.GetHash(), 0);
        }
    }

    #[test]
    fn supported_pseudo_request_uses_translated_rule_set_and_collector() {
        let sheet = parse(
            ".hit{color:red}.hit::before{content:'typed';display:block}\
             .hit::first-letter{font-size:21px}",
        );
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let element = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(document.Root(), element);

        let mut result = MatchResult::default();
        let collection = CollectProductionRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set: rule_set.clone(),
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            element,
            &mut PersistentSelectorService::default(),
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Pseudo(PseudoId::kPseudoIdBefore, None),
            None,
            &mut result,
        );
        assert_eq!(
            collection.report.skipped_pseudo, 0,
            "{:?}",
            collection.report
        );
        assert_eq!(
            collection.report.selector_matches, 1,
            "{:?}",
            collection.report
        );
        assert_eq!(collection.report.transferred_property_sets, 1);
        assert_eq!(collection.rules.len(), 1);
        assert_eq!(result.GetMatchedProperties().len(), 1);

        let mut result = MatchResult::default();
        let collection = CollectProductionRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set: rule_set.clone(),
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            element,
            &mut PersistentSelectorService::default(),
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Pseudo(PseudoId::kPseudoIdFirstLetter, None),
            None,
            &mut result,
        );
        assert_eq!(
            collection.report.skipped_pseudo, 0,
            "{:?}",
            collection.report
        );
        assert_eq!(
            collection.report.selector_matches, 1,
            "{:?}",
            collection.report
        );
        assert_eq!(collection.report.transferred_property_sets, 1);
        assert_eq!(result.GetMatchedProperties().len(), 1);

        // Invalidation asks about originating elements, not whether this
        // request can publish a computed style for the pseudo itself.
        let mut owner = html::html_parser::ParseHTML(
            "<!doctype html><div class=hit></div><div class=miss></div>",
        );
        let document = owner.GetDocumentMut();
        let elements = (0..document.NodeCount())
            .filter(|&index| document.Node(index).IsHTMLElement("div"))
            .collect::<Vec<_>>();
        assert_eq!(elements.len(), 2);
        for pseudo in ["::backdrop", "::marker", "::first-line", "::before::marker"] {
            let sheet = parse(&format!(".hit{pseudo}{{color:red}}"));
            let rules =
                Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
            let invalidated = InvalidateProductionRuleSets(
                &[rules],
                document,
                &mut PersistentSelectorService::default(),
                &elements,
            )
            .expect("translated pseudo selector must not require full-rule fallback");
            assert_eq!(invalidated, HashSet::from([elements[0]]), "{pseudo}");
        }
    }

    #[test]
    fn placeholder_request_matches_retained_ua_shadow_element() {
        let sheet = parse(
            ".other::placeholder{font-size:44px}\
             .hit::first-letter{font-size:43px}\
             .hit::placeholder{font-size:30px}",
        );
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let input = document.CreateElement(
            DOMNamespace::kHTML,
            "input".into(),
            vec![
                DOMAttribute {
                    local_name: "class".into(),
                    value: "hit".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "placeholder".into(),
                    value: "fixture".into(),
                    ..Default::default()
                },
            ],
        );
        document.AppendChild(document.Root(), input);

        let mut result = MatchResult::default();
        let collection = CollectProductionRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set,
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            input,
            &mut PersistentSelectorService::default(),
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Pseudo(PseudoId::kPseudoIdPlaceholder, None),
            None,
            &mut result,
        );

        assert_eq!(
            collection.report.skipped_pseudo, 0,
            "{:?}",
            collection.report
        );
        assert_eq!(
            collection.report.selector_matches, 1,
            "{:?}",
            collection.report
        );
        assert_eq!(collection.report.transferred_property_sets, 1);
        assert_eq!(collection.rules.len(), 1);
        assert_eq!(result.GetMatchedProperties().len(), 1);

        document.RemoveAttributeDefault(input, "placeholder");
        assert!(document
            .UAShadowPseudoElement(input, "-webkit-input-placeholder")
            .is_none());
        document.SetAttribute(
            input,
            DOMAttribute {
                local_name: "placeholder".into(),
                value: "again".into(),
                ..Default::default()
            },
        );
        assert!(document
            .UAShadowPseudoElement(input, "-webkit-input-placeholder")
            .is_some());
        document.SetAttribute(
            input,
            DOMAttribute {
                local_name: "type".into(),
                value: "checkbox".into(),
                ..Default::default()
            },
        );
        assert!(document
            .UAShadowPseudoElement(input, "-webkit-input-placeholder")
            .is_none());
    }

    #[test]
    fn starting_style_uses_recalc_old_style_owner() {
        let sheet = parse("@starting-style{.hit{font-size:31px}}.hit{width:7px}");
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let element = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(document.Root(), element);

        let input = [ProductionRuleSetInput {
            sheet_index: 0,
            rule_set,
            routing: &routing,
            implicit_scope_trigger: None,
        }];
        let mut selectors = PersistentSelectorService::default();
        let mut initial = MatchResult::default();
        let report = CollectProductionRuleMatchesInto(
            &input,
            document,
            element,
            &mut selectors,
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Element(),
            None,
            &mut initial,
        )
        .report;
        assert_eq!(report.skipped_starting_style, 0);
        assert_eq!(report.selector_matches, 2, "{report:?}");
        assert!(initial.HasFlag(crate::resolver::match_flags::MatchFlag::kAffectedByStartingStyle));

        document.SetResolvedStyle(element, Default::default());
        let mut subsequent = MatchResult::default();
        let report = CollectProductionRuleMatchesInto(
            &input,
            document,
            element,
            &mut selectors,
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Element(),
            None,
            &mut subsequent,
        )
        .report;
        assert_eq!(report.skipped_starting_style, 0);
        assert_eq!(report.selector_matches, 1, "{report:?}");
        assert!(
            !subsequent.HasFlag(crate::resolver::match_flags::MatchFlag::kAffectedByStartingStyle)
        );
    }

    #[test]
    fn shadow_host_rules_use_attached_shadow_tree_scope() {
        let sheet = parse(
            ":host{width:11px}:host(.chosen){height:12px}\
             :host-context(.outer){opacity:.5}:is(:host,.never){color:red}\
             :host(.absent){display:none}",
        );
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let outer = document.CreateElement(
            DOMNamespace::kHTML,
            "section".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "outer".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(document.Root(), outer);
        let host = document.CreateElement(
            DOMNamespace::kHTML,
            "x-host".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "chosen".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(outer, host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);

        let mut result = MatchResult::default();
        let collection = CollectProductionShadowRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set,
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            host,
            &mut PersistentSelectorService::default(),
            shadow,
            &ProductionShadowRulePass::Host,
            None,
            &mut result,
        );
        assert_eq!(
            collection.report.selector_matches, 4,
            "{:?}",
            collection.report
        );
        assert_eq!(collection.report.transferred_property_sets, 4);
        assert_eq!(result.GetMatchedProperties().len(), 4);
    }

    #[test]
    fn slotted_rules_use_named_slot_assignment_in_the_owning_scope() {
        let sheet = parse("::slotted(.chosen){width:21px}::slotted(.absent){display:none}");
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let host = document.CreateElementDefault(DOMNamespace::kHTML, "x-host".into());
        document.AppendChild(document.Root(), host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let slot = document.CreateElement(
            DOMNamespace::kHTML,
            "slot".into(),
            vec![DOMAttribute {
                local_name: "name".into(),
                value: "named".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(shadow, slot);
        let light = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![
                DOMAttribute {
                    local_name: "slot".into(),
                    value: "named".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "class".into(),
                    value: "chosen".into(),
                    ..Default::default()
                },
            ],
        );
        document.AppendChild(host, light);

        let mut result = MatchResult::default();
        let collection = CollectProductionShadowRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set,
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            light,
            &mut PersistentSelectorService::default(),
            shadow,
            &ProductionShadowRulePass::Slotted,
            None,
            &mut result,
        );
        assert_eq!(document.AssignedSlot(light), Some(slot));
        assert_eq!(
            collection.report.selector_matches, 1,
            "{:?}",
            collection.report
        );
        assert_eq!(collection.report.transferred_property_sets, 1);
    }

    #[test]
    fn part_rules_use_outer_scope_and_exact_part_name_set() {
        let sheet = parse(
            "x-host::part(label){width:31px}\
             x-host::part(label badge){height:32px}\
             x-host::part(other){display:none}",
        );
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let host = document.CreateElementDefault(DOMNamespace::kHTML, "x-host".into());
        document.AppendChild(document.Root(), host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let inner = document.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        document.AppendChild(shadow, inner);

        let mut result = MatchResult::default();
        let collection = CollectProductionShadowRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set,
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            inner,
            &mut PersistentSelectorService::default(),
            document.Root(),
            &ProductionShadowRulePass::Part(PersistentPartNames::FromNames([
                AtomicString::from_str("label"),
            ])),
            None,
            &mut result,
        );
        assert_eq!(document.OwnerShadowHost(inner), Some(host));
        assert_eq!(
            collection.report.selector_matches, 1,
            "{:?}",
            collection.report
        );
        assert_eq!(collection.report.transferred_property_sets, 1);
    }

    #[test]
    fn exportparts_forwards_and_renames_across_every_shadow_host() {
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let outer = document.CreateElementDefault(DOMNamespace::kHTML, "x-outer".into());
        document.AppendChild(document.Root(), outer);
        let outer_shadow = document.AttachShadowRoot(outer, DOMShadowRootMode::kOpen);
        let middle = document.CreateElement(
            DOMNamespace::kHTML,
            "x-middle".into(),
            vec![DOMAttribute {
                local_name: "exportparts".into(),
                value: "middle: outer, badge: outer-badge".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(outer_shadow, middle);
        let middle_shadow = document.AttachShadowRoot(middle, DOMShadowRootMode::kOpen);
        let inner = document.CreateElement(
            DOMNamespace::kHTML,
            "x-inner".into(),
            vec![DOMAttribute {
                local_name: "exportparts".into(),
                value: "leaf: middle, leaf: badge".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(middle_shadow, inner);
        let inner_shadow = document.AttachShadowRoot(inner, DOMShadowRootMode::kOpen);
        let target = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "part".into(),
                value: "leaf".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(inner_shadow, target);

        let scopes = ProductionPartRuleScopes(document, target);
        assert_eq!(
            scopes.iter().map(|(scope, _)| *scope).collect::<Vec<_>>(),
            vec![middle_shadow, outer_shadow, document.Root()]
        );
        assert!(scopes[0].1.Contains(&AtomicString::from_str("leaf")));
        assert!(!scopes[0].1.Contains(&AtomicString::from_str("middle")));
        assert!(scopes[1].1.Contains(&AtomicString::from_str("middle")));
        assert!(scopes[1].1.Contains(&AtomicString::from_str("badge")));
        assert!(!scopes[1].1.Contains(&AtomicString::from_str("leaf")));
        assert!(scopes[2].1.Contains(&AtomicString::from_str("outer")));
        assert!(scopes[2].1.Contains(&AtomicString::from_str("outer-badge")));
        assert!(!scopes[2].1.Contains(&AtomicString::from_str("middle")));

        // Each retained name set is consumed by SelectorChecker's ::part
        // branch in its corresponding TreeScope. The other two selectors in
        // the shared RuleSet must be rejected by host/name matching.
        let sheet = parse(
            "x-inner::part(leaf){width:1px}\
             x-middle::part(middle){height:2px}\
             x-outer::part(outer){opacity:.5}",
        );
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut selectors = PersistentSelectorService::default();
        for (scope, names) in &scopes {
            let mut result = MatchResult::default();
            let collection = CollectProductionShadowRuleMatchesInto(
                &[ProductionRuleSetInput {
                    sheet_index: 0,
                    rule_set: rule_set.clone(),
                    routing: &routing,
                    implicit_scope_trigger: None,
                }],
                document,
                target,
                &mut selectors,
                *scope,
                &ProductionShadowRulePass::Part(names.clone()),
                None,
                &mut result,
            );
            assert_eq!(
                collection.report.selector_matches, 1,
                "scope={scope} report={:?}",
                collection.report
            );
            assert_eq!(collection.report.transferred_property_sets, 1);
        }

        document.RemoveAttributeDefault(middle, "exportparts");
        let stopped = ProductionPartRuleScopes(document, target);
        assert_eq!(
            stopped.iter().map(|(scope, _)| *scope).collect::<Vec<_>>(),
            vec![middle_shadow, outer_shadow]
        );
    }

    #[test]
    fn vertical_tab_does_not_split_part_dom_token_list() {
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let host = document.CreateElementDefault(DOMNamespace::kHTML, "x-host".into());
        document.AppendChild(document.Root(), host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let target = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "part".into(),
                value: "one\u{000b}two three".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(shadow, target);
        let scopes = ProductionPartRuleScopes(document, target);
        assert_eq!(scopes.len(), 1);
        assert!(scopes[0]
            .1
            .Contains(&AtomicString::from_str("one\u{000b}two")));
        assert!(scopes[0].1.Contains(&AtomicString::from_str("three")));
        assert!(!scopes[0].1.Contains(&AtomicString::from_str("one")));
    }

    #[test]
    fn explicit_scope_uses_translated_boundaries_and_proximity_order() {
        // The nearer scope is deliberately earlier in source order. Chromium
        // resolves an equal-layer/equal-specificity tie by scoping proximity
        // before source order, so it must be transferred last.
        let sheet = parse(
            "@scope (.inner){.hit{width:1px}} \
             @scope (.outer) to (.limit){.hit{width:2px}}",
        );
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);

        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let outer = document.CreateElement(
            DOMNamespace::kHTML,
            "section".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "outer".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(document.Root(), outer);
        let inner = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "inner".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(outer, inner);
        let hit = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(inner, hit);
        let limit = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "limit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(outer, limit);
        let blocked = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(limit, blocked);
        let outside = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(document.Root(), outside);

        let input = [ProductionRuleSetInput {
            sheet_index: 0,
            rule_set,
            routing: &routing,
            implicit_scope_trigger: None,
        }];
        let mut selectors = PersistentSelectorService::default();
        let mut result = MatchResult::default();
        let collection = CollectProductionRuleMatchesInto(
            &input,
            document,
            hit,
            &mut selectors,
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Element(),
            None,
            &mut result,
        );
        assert_eq!(
            collection.report.skipped_scope, 0,
            "{:?}",
            collection.report
        );
        assert_eq!(
            collection.report.selector_matches, 2,
            "{:?}",
            collection.report
        );
        assert_eq!(
            collection
                .rules
                .iter()
                .map(|rule| rule.proximity)
                .collect::<Vec<_>>(),
            vec![2, 1],
            "farther scope must transfer before the nearer scope"
        );
        assert_eq!(result.GetMatchedProperties().len(), 2);

        for element in [blocked, outside] {
            let mut result = MatchResult::default();
            let collection = CollectProductionRuleMatchesInto(
                &input,
                document,
                element,
                &mut selectors,
                CascadeOrigin::kAuthor,
                &ProductionPseudoRequest::Element(),
                None,
                &mut result,
            );
            assert!(collection.rules.is_empty(), "{:?}", collection.report);
            assert!(result.GetMatchedProperties().is_empty());
        }
    }

    #[test]
    fn implicit_scope_owner_and_container_query_run_in_translated_collector() {
        let _heap = foundation::LayoutHeapScope::new();
        let sheet = parse(
            "@scope {.implicit{color:blue}} \
             @container (width > 100px){.hit{display:block}}",
        );
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = dom::DOM::new();
        let document = owner.GetDocumentMut();
        let container = document.CreateElementDefault(DOMNamespace::kHTML, "section".into());
        document.AppendChild(document.Root(), container);
        let element = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "hit implicit".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(container, element);

        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetContainerType(1);
        let container_style = Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
        let writing_direction = unsafe { &*container_style.Get() }.GetWritingDirection();
        let current_styles = HashMap::from([(container, container_style)]);
        let states = HashMap::from([(
            container,
            ContainerState::SizeOnly(200.0, 50.0, writing_direction),
        )]);
        let custom = HashMap::<usize, Rc<CustomProperties>>::new();
        let media = MediaValuesCachedData {
            media_type: foundation::String::from("screen"),
            viewport_width: 800.0,
            viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        };
        let runtime = ProductionContainerRuntime {
            document,
            media: &media,
            states: &states,
            stored_custom: &custom,
            current_styles: Some(&current_styles),
            current_custom: None,
            environment: None,
            registered: None,
        };

        let mut result = MatchResult::default();
        let collection = CollectProductionRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set: rule_set.clone(),
                routing: &routing,
                implicit_scope_trigger: Some(container),
            }],
            document,
            element,
            &mut PersistentSelectorService::default(),
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Element(),
            Some(&runtime),
            &mut result,
        );
        assert_eq!(
            collection.report.skipped_scope, 0,
            "{:?}",
            collection.report
        );
        assert_eq!(
            collection.report.skipped_container_query, 0,
            "{:?}",
            collection.report
        );
        assert_eq!(
            collection.report.selector_matches, 2,
            "{:?}",
            collection.report
        );
        assert_eq!(collection.report.transferred_property_sets, 2);
        assert_eq!(result.GetMatchedProperties().len(), 2);
        assert!(result.DependsOnSizeContainerQueries());

        // The same shared StyleScope without an owner trigger must not become
        // document-wide; the independent true container rule still matches.
        let mut ownerless_result = MatchResult::default();
        let ownerless = CollectProductionRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set: rule_set.clone(),
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            element,
            &mut PersistentSelectorService::default(),
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Element(),
            Some(&runtime),
            &mut ownerless_result,
        );
        assert_eq!(
            ownerless.report.selector_matches, 1,
            "{:?}",
            ownerless.report
        );
        assert_eq!(ownerless_result.GetMatchedProperties().len(), 1);

        let narrow_states = HashMap::from([(
            container,
            ContainerState::SizeOnly(80.0, 50.0, writing_direction),
        )]);
        let narrow_runtime = ProductionContainerRuntime {
            states: &narrow_states,
            ..runtime
        };
        let mut narrow_result = MatchResult::default();
        let narrow = CollectProductionRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set: rule_set.clone(),
                routing: &routing,
                implicit_scope_trigger: Some(container),
            }],
            document,
            element,
            &mut PersistentSelectorService::default(),
            CascadeOrigin::kAuthor,
            &ProductionPseudoRequest::Element(),
            Some(&narrow_runtime),
            &mut narrow_result,
        );
        assert_eq!(narrow.report.selector_matches, 1, "{:?}", narrow.report);
        assert_eq!(narrow_result.GetMatchedProperties().len(), 1);
        assert!(narrow_result.DependsOnSizeContainerQueries());
    }
    #[test]
    fn svg_use_element_scope_keeps_clone_matching_separate_from_rule_owner() {
        let sheet =
            parse(".shape{fill:blue} #source-parent .shape{fill:red} use .shape{fill:green}");
        let rule_set =
            Rc::new(sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0)));
        let routing = ProductionRuleRouting::Build(&rule_set, &sheet);
        let mut owner = html::html_parser::ParseHTML(
            r##"<svg><defs id="source-parent"><rect id="box" class="shape" width="10" height="10"/></defs><use href="#box"/></svg>"##,
        );
        let document = owner.GetDocumentMut();
        document.UpdateSVGUseInstances();
        let host = (0..document.NodeCount())
            .find(|&i| document.Node(i).IsElement("use", DOMNamespace::kSVG))
            .unwrap();
        let instance = document.SVGUseInstanceRoot(host).unwrap();
        assert_ne!(document.TreeScopeRoot(instance), document.Root());
        let mut result = MatchResult::default();
        let collection = CollectProductionShadowRuleMatchesInto(
            &[ProductionRuleSetInput {
                sheet_index: 0,
                rule_set,
                routing: &routing,
                implicit_scope_trigger: None,
            }],
            document,
            instance,
            &mut PersistentSelectorService::default(),
            document.Root(),
            &ProductionShadowRulePass::ElementScope(None),
            None,
            &mut result,
        );
        assert_eq!(
            collection.report.selector_matches, 1,
            "match clone attributes, never source ancestors or use host"
        );
        assert_eq!(collection.report.transferred_property_sets, 1);
        assert_eq!(result.GetMatchedProperties().len(), 1);
    }
}

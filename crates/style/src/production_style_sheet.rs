// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Concrete assembly of CSSParserImpl's style/media/layer/font/keyframe consumers.
//! Source ranges come from CSSParserImplRuleObserver, never a brace walker.
#![allow(non_snake_case, unused_variables)]
use crate::{
    active_style_sheets::ActiveStyleSheet, cascade_layer_map::CascadeLayerMap,
    cascade_layered::CascadeLayered, invalidation::selector_pre_match::SelectorPreMatch,
    style_rule_keyframe::KeyframeOffset,
};
use crate::{
    css_property_value::CSSPropertyValue,
    css_property_value_set::*,
    css_selector::{CSSSelector, CSSSelectorComplex, MatchType, PseudoType, QualifiedName},
    css_selector_list::CSSSelectorList,
    css_value::{CSSValue, CSSValuePayload},
    media_queries::{
        media_query_backend::MediaQueryFeatureFlags,
        media_query_backend::MediaQueryValueBackend,
        media_query_evaluator::MediaQueryEvaluator,
        media_query_set::MediaQuerySet as ConcreteQuerySet,
        media_values_cached::{MediaValuesCached, MediaValuesCachedData},
    },
    parser::{
        allowed_rules::AllowedRules,
        css_at_rule_id::{AtRuleRuntimeFeatures, CSSAtRuleID},
        css_nesting_type::CSSNestingType,
        css_parser_context::*,
        css_parser_impl::*,
        css_parser_mode::*,
        css_parser_token::CSSParserTokenType::*,
        css_parser_token_stream::{
            BlockGuard, CSSParserTokenStream, EnableUnicodeRanges, TokenStreamTokenizer,
        },
        css_selector_parser::{CSSSelectorParser, SelectorParserContext, SelectorParserOptions},
        media_query_parser::MediaQueryParser,
    },
    production_css_value::{
        self as values, CSSVariableData, ProductionCSSValueDispatch, PropertyValue,
    },
    resolver::media_query_result::{MediaQueryResultFlags, MediaQuerySetResult},
    rule_set::*,
    style_rule::*,
    style_sheet_contents::*,
};
use foundation::{AtomicString, CSSPropertyID, CSSValueID, Member, String, StringView};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};
pub type ContainerSet = crate::production_container_parser::ContainerSet;
pub type ContainerCondition = crate::production_container_parser::ContainerCondition;
pub type QuerySet = ConcreteQuerySet<crate::css_numeric_literal_value::CSSNumericLiteralValue>;
impl crate::resolver::media_query_result::MediaQuerySet for QuerySet {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unavailable {}
pub type SourceURL = values::ProductionURL;
#[derive(Clone, Copy)]
pub enum Feature {
    Rule,
    Property,
    AtRule(CSSAtRuleID),
    ContainerRangeSyntax,
    ContainerStyleQuery,
}
pub struct Backend;

/// Owned media environment used while building a production RuleSet.  Keeping
/// the cached values here makes the RuleSet API independent of evaluator
/// lifetimes while still running Blink's translated media evaluator.
#[derive(Clone)]
pub struct ProductionMediaQueryEvaluator {
    values: MediaValuesCached,
    media_type_override: Option<String>,
}
impl ProductionMediaQueryEvaluator {
    pub fn FromCachedData(data: &MediaValuesCachedData) -> Self {
        Self {
            values: MediaValuesCached::new(data),
            media_type_override: None,
        }
    }
    /// CSSDefaultStyleSheets builds supplemental UA RuleSets with a dedicated
    /// media-type evaluator (for example `ua-forced-colors`) while retaining
    /// the same parsed stylesheet owner.
    pub fn ForMediaType(media_type: &str, data: &MediaValuesCachedData) -> Self {
        Self {
            values: MediaValuesCached::new(data),
            media_type_override: Some(String::from(media_type)),
        }
    }
    pub fn Screen(viewport_width: f64, viewport_height: f64) -> Self {
        let mut data = MediaValuesCachedData::default();
        data.media_type = String::from("screen");
        data.viewport_width = viewport_width;
        data.viewport_height = viewport_height;
        data.small_viewport_width = viewport_width;
        data.small_viewport_height = viewport_height;
        data.large_viewport_width = viewport_width;
        data.large_viewport_height = viewport_height;
        data.dynamic_viewport_width = viewport_width;
        data.dynamic_viewport_height = viewport_height;
        data.device_width = viewport_width as i32;
        data.device_height = viewport_height as i32;
        Self::FromCachedData(&data)
    }
    fn evaluator(&self) -> MediaQueryEvaluator<'_, MediaQueryValueBackend<'_>> {
        self.media_type_override.as_ref().map_or_else(
            || MediaQueryEvaluator::ForMediaValues(&self.values),
            |media_type| MediaQueryEvaluator::ForMediaType(media_type.clone(), &self.values),
        )
    }
    pub fn EvalSet(&self, queries: &QuerySet) -> bool {
        self.evaluator().Eval(queries)
    }
}
impl Default for ProductionMediaQueryEvaluator {
    fn default() -> Self {
        Self::Screen(0.0, 0.0)
    }
}

pub struct ProductionRuleBucketTable(HashMap<AtomicString, Extent>);
impl RuleBucketTable for ProductionRuleBucketTable {
    fn New(initial_capacity: usize) -> Self {
        Self(HashMap::with_capacity(initial_capacity))
    }
    fn Find(&self, key: &AtomicString) -> Option<&Extent> {
        self.0.get(key)
    }
    fn FindMut(&mut self, key: &AtomicString) -> Option<&mut Extent> {
        self.0.get_mut(key)
    }
    fn Insert(&mut self, key: &AtomicString) -> Option<&mut Extent> {
        Some(self.0.entry(key.clone()).or_default())
    }
    fn Entries(&self) -> Vec<(AtomicString, Extent)> {
        self.0
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AttributeInvalidationKey {
    Class(AtomicString),
    Id(AtomicString),
    Attribute(AtomicString),
}
pub(crate) struct AttributeInvalidation {
    pub set: crate::invalidation::invalidation_set::InvalidationSetRef,
    pub siblings: bool,
    pub cross_tree: bool,
}

#[derive(Default)]
pub struct ProductionRuleFeatureSet {
    pub(crate) non_descendant_has_invalidation: bool,
    pub(crate) attribute_invalidation: HashMap<AttributeInvalidationKey, AttributeInvalidation>,
    pub(crate) scope_pseudo_states: BTreeSet<PseudoType>,
    pub(crate) non_descendant_pseudo_states: BTreeSet<PseudoType>,
    pub(crate) media_query_result_flags: MediaQueryResultFlags,
    pub(crate) pseudo_state_invalidation:
        BTreeMap<PseudoType, crate::invalidation::invalidation_set::InvalidationSetRef>,
}

// RuleInvalidationDataVisitor distinguishes a pseudo on the subject from one
// which invalidates descendants, siblings or :has() subjects. Retain those
// target features while RuleSet still owns the compiled selector, rather than
// reparsing a CSSOM projection during each input event.
fn user_action_features(
    selector: CSSSelectorComplex<'_>,
) -> (BTreeSet<PseudoType>, BTreeSet<PseudoType>) {
    use crate::css_selector::RelationType;
    let (mut any, mut relational, mut subject) = (BTreeSet::new(), BTreeSet::new(), true);
    for simple in selector.SimpleSelectors() {
        let pseudo = simple.GetPseudoType();
        if matches!(
            simple.Match(),
            MatchType::kPseudoClass | MatchType::kPseudoElement
        ) && matches!(
            pseudo,
            PseudoType::kPseudoHover
                | PseudoType::kPseudoActive
                | PseudoType::kPseudoActiveOption
                | PseudoType::kPseudoFocus
                | PseudoType::kPseudoFocusVisible
                | PseudoType::kPseudoFocusWithin
                | PseudoType::kPseudoMultiSelectFocus
                | PseudoType::kPseudoSpatialNavigationFocus
                | PseudoType::kPseudoAnyLink
                | PseudoType::kPseudoWebkitAnyLink
                | PseudoType::kPseudoLink
                | PseudoType::kPseudoVisited
                | PseudoType::kPseudoPlaceholderShown
                | PseudoType::kPseudoDefault
                | PseudoType::kPseudoHasDatalist
                | PseudoType::kPseudoIndeterminate
                | PseudoType::kPseudoHas
                | PseudoType::kPseudoDir
        ) {
            any.insert(pseudo);
            if !subject {
                relational.insert(pseudo);
            }
        }
        if let Some(list) = simple.SelectorListOrParent() {
            for nested in list.ComplexSelectors() {
                let (nested_any, nested_relational) = user_action_features(nested);
                any.extend(nested_any.iter().copied());
                relational.extend(nested_relational);
                if !subject
                    || !matches!(
                        pseudo,
                        PseudoType::kPseudoIs
                            | PseudoType::kPseudoAny
                            | PseudoType::kPseudoWhere
                            | PseudoType::kPseudoNot
                            | PseudoType::kPseudoParent
                    )
                {
                    relational.extend(nested_any);
                }
            }
        }
        if !matches!(
            simple.Relation(),
            RelationType::kSubSelector | RelationType::kPseudoChild
        ) {
            subject = false;
        }
    }
    (any, relational)
}

fn non_descendant_user_action_selector(selector: CSSSelectorComplex<'_>) -> bool {
    use crate::css_selector::RelationType::*;
    selector.SimpleSelectors().any(|simple| {
        matches!(
            simple.Relation(),
            kDirectAdjacent | kIndirectAdjacent | kShadowSlot | kShadowPart
        ) || matches!(
            simple.GetPseudoType(),
            PseudoType::kPseudoHas
                | PseudoType::kPseudoNthChild
                | PseudoType::kPseudoNthLastChild
                | PseudoType::kPseudoNthOfType
                | PseudoType::kPseudoNthLastOfType
        ) || simple.SelectorListOrParent().is_some_and(|list| {
            list.ComplexSelectors()
                .any(non_descendant_user_action_selector)
        })
    })
}

fn non_descendant_has_selector(selector: CSSSelectorComplex<'_>) -> bool {
    use crate::css_selector::RelationType::*;
    selector.SimpleSelectors().any(|simple| {
        matches!(simple.Relation(), kDirectAdjacent | kIndirectAdjacent | kShadowSlot | kShadowPart)
            || (matches!(simple.GetPseudoType(), PseudoType::kPseudoNthChild | PseudoType::kPseudoNthLastChild)
                && simple.SelectorListOrParent().is_some_and(|list| list.ComplexSelectors()
                    .any(|nested| user_action_features(nested).0.contains(&PseudoType::kPseudoHas))))
            // The relative selector inside :has controls ancestor traversal,
            // not the descendant subjects scheduled from this Has anchor.
            || (simple.GetPseudoType() != PseudoType::kPseudoHas
                && simple.SelectorListOrParent().is_some_and(|list| list.ComplexSelectors().any(non_descendant_has_selector)))
    })
}

// cpp: RuleInvalidationDataVisitor::AddFeaturesToInvalidationSets and
// StyleEngine::{Class,Id,Attribute}ChangedForElement. Admit ordinary tree
// relations here; cross-tree keys retain the owner's conservative route.
fn collect_attribute_features(
    features: &mut ProductionRuleFeatureSet,
    selector: CSSSelectorComplex<'_>,
    scope: Option<&crate::style_scope::StyleScope>,
) {
    use crate::css_selector::RelationType::*;
    use crate::invalidation::invalidation_set::DescendantInvalidationSet;
    use AttributeInvalidationKey as Key;
    fn keys(selector: CSSSelectorComplex<'_>, result: &mut Vec<Key>) {
        for simple in selector.SimpleSelectors() {
            match simple.Match() {
                MatchType::kClass => result.push(Key::Class(simple.Value().clone())),
                MatchType::kId => result.push(Key::Id(simple.Value().clone())),
                MatchType::kPseudoClass if simple.GetPseudoType() == PseudoType::kPseudoTarget => {
                    result.push(Key::Attribute(AtomicString::from_str("id")))
                }
                _ if simple.IsAttributeSelector() => {
                    result.push(Key::Attribute(simple.Attribute().LocalName().clone()))
                }
                _ => {}
            }
            if let Some(list) = simple.SelectorListOrParent() {
                for nested in list.ComplexSelectors() {
                    keys(nested, result);
                }
            }
        }
    }
    fn relations(selector: CSSSelectorComplex<'_>, siblings: &mut bool, cross: &mut bool) {
        for simple in selector.SimpleSelectors() {
            *siblings |= matches!(
                simple.Relation(),
                kDirectAdjacent
                    | kIndirectAdjacent
                    | kRelativeDirectAdjacent
                    | kRelativeIndirectAdjacent
            );
            *cross |= matches!(simple.Relation(), kUAShadow | kShadowSlot | kShadowPart)
                || matches!(
                    simple.GetPseudoType(),
                    PseudoType::kPseudoHost
                        | PseudoType::kPseudoHostContext
                        | PseudoType::kPseudoPart
                        | PseudoType::kPseudoSlotted
                );
            if let Some(list) = simple.SelectorListOrParent() {
                for nested in list.ComplexSelectors() {
                    relations(nested, siblings, cross);
                }
            }
        }
    }
    let (mut siblings, mut cross_tree) = (false, scope.is_some());
    relations(selector, &mut siblings, &mut cross_tree);
    let mut changed_keys = Vec::new();
    if cross_tree {
        keys(selector, &mut changed_keys);
    } else {
        let mut subject = true;
        for simple in selector.SimpleSelectors() {
            if !subject {
                match simple.Match() {
                    MatchType::kClass => changed_keys.push(Key::Class(simple.Value().clone())),
                    MatchType::kId => changed_keys.push(Key::Id(simple.Value().clone())),
                    MatchType::kPseudoClass
                        if simple.GetPseudoType() == PseudoType::kPseudoTarget =>
                    {
                        changed_keys.push(Key::Attribute(AtomicString::from_str("id")))
                    }
                    _ if simple.IsAttributeSelector() => {
                        changed_keys.push(Key::Attribute(simple.Attribute().LocalName().clone()))
                    }
                    _ => {}
                }
            }
            if let Some(list) = simple.SelectorListOrParent() {
                for nested in list.ComplexSelectors() {
                    // Logical combinations may carry ancestor/sibling keys
                    // even while their outer compound is the subject.
                    if !subject
                        || nested
                            .SimpleSelectors()
                            .any(|s| s.Relation() != kSubSelector)
                    {
                        keys(nested, &mut changed_keys);
                    }
                }
            }
            if simple.Relation() != kSubSelector && simple.Relation() != kPseudoChild {
                subject = false;
            }
        }
    }
    for key in changed_keys {
        let entry = features
            .attribute_invalidation
            .entry(key)
            .or_insert_with(|| AttributeInvalidation {
                set: DescendantInvalidationSet::Create(),
                siblings: false,
                cross_tree: false,
            });
        entry.siblings |= siblings;
        entry.cross_tree |= cross_tree;
        collect_user_action_subject(&mut entry.set.borrow_mut(), selector);
    }
}

fn collect_user_action_features(
    features: &mut ProductionRuleFeatureSet,
    selector: CSSSelectorComplex<'_>,
    scope: Option<&crate::style_scope::StyleScope>,
) {
    use crate::invalidation::invalidation_set::DescendantInvalidationSet;
    let mut pseudos = user_action_features(selector).1;
    if pseudos.contains(&PseudoType::kPseudoHas) {
        features.non_descendant_has_invalidation |=
            scope.is_some() || non_descendant_has_selector(selector);
    }
    if non_descendant_user_action_selector(selector) {
        features
            .non_descendant_pseudo_states
            .extend(pseudos.iter().copied());
    }
    let mut scoped = BTreeSet::new();
    if let Some(scope) = scope {
        collect_scope_pseudo_states(scope, &mut scoped);
    }
    features.scope_pseudo_states.extend(scoped.iter().copied());
    pseudos.extend(scoped.iter().copied());
    for pseudo in pseudos {
        let mut set = features
            .pseudo_state_invalidation
            .entry(pseudo)
            .or_insert_with(DescendantInvalidationSet::Create)
            .borrow_mut();
        // Scope activation can depend on a user-action selector outside this rule.
        if scoped.contains(&pseudo) {
            set.SetWholeSubtreeInvalid();
            continue;
        }
        collect_user_action_subject(&mut set, selector);
    }
}

fn collect_user_action_subject(
    set: &mut crate::invalidation::invalidation_set::InvalidationSet,
    selector: CSSSelectorComplex<'_>,
) {
    use crate::css_selector::RelationType;
    let (mut class, mut attribute, mut tag, mut custom_pseudo) = (None, None, None, None);
    for simple in selector.SimpleSelectors() {
        match simple.Match() {
            MatchType::kId => {
                set.AddId(&simple.Value());
                return;
            }
            MatchType::kClass if class.is_none() => class = Some((*simple.Value()).clone()),
            MatchType::kTag if tag.is_none() => tag = Some(simple.TagQName().LocalName().clone()),
            MatchType::kPseudoElement
                if matches!(
                    simple.GetPseudoType(),
                    PseudoType::kPseudoWebKitCustomElement
                        | PseudoType::kPseudoBlinkInternalElement
                ) =>
            {
                custom_pseudo = Some((*simple.Value()).clone())
            }
            _ if simple.IsAttributeSelector() && attribute.is_none() => {
                attribute = Some(simple.Attribute().LocalName().clone())
            }
            _ => {}
        }
        if !matches!(
            simple.Relation(),
            RelationType::kSubSelector | RelationType::kPseudoChild
        ) {
            break;
        }
    }
    if let Some(class) = class {
        set.AddClass(&class);
    } else if let Some(attribute) = attribute {
        set.AddAttribute(&attribute);
    } else if let Some(tag) = tag {
        set.AddTagName(&tag);
    } else if let Some(custom_pseudo) = custom_pseudo {
        set.AddCustomPseudoName(&custom_pseudo);
    } else {
        set.SetWholeSubtreeInvalid();
    }
}

fn collect_scope_pseudo_states(
    scope: &crate::style_scope::StyleScope,
    pseudos: &mut BTreeSet<PseudoType>,
) {
    for list in scope.From().into_iter().chain(scope.To()) {
        for selector in list.ComplexSelectors() {
            pseudos.extend(user_action_features(selector).0);
        }
    }
    if let Some(parent) = scope.Parent() {
        collect_scope_pseudo_states(parent, pseudos);
    }
}

pub struct ProductionSubstringSetMatcher(Vec<RuleSetSubstringPattern>);

pub struct ProductionMixinParameterBindings {
    bindings: HashMap<String, RuleSetMixinBinding<Backend>>,
    locals: RuleSetMixinLocals<Backend>,
    conditional_locals: RuleSetConditionalLocals<Backend>,
    parent: Option<Rc<Self>>,
}

impl ProductionMixinParameterBindings {
    pub(crate) fn Bindings(&self) -> &HashMap<String, RuleSetMixinBinding<Backend>> {
        &self.bindings
    }

    pub(crate) fn BaseLocals(&self) -> &RuleSetMixinLocals<Backend> {
        &self.locals
    }

    pub(crate) fn ConditionalLocals(&self) -> &RuleSetConditionalLocals<Backend> {
        &self.conditional_locals
    }

    pub(crate) fn Parent(&self) -> Option<Rc<Self>> {
        self.parent.clone()
    }
}

impl crate::resolver::match_result::MixinParameterBindings for ProductionMixinParameterBindings {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }

    fn Equals(&self, other: &dyn crate::resolver::match_result::MixinParameterBindings) -> bool {
        other
            .AsAny()
            .downcast_ref::<Self>()
            .is_some_and(|other| std::ptr::eq(self, other))
    }

    fn GetHash(&self) -> u32 {
        let address = self as *const Self as usize;
        (address as u32) ^ ((address >> 32) as u32)
    }
}

pub struct ProductionRuleSetDiff {
    pub old: RuleSetHandle<Backend>,
    pub changed_rules: Vec<Rc<StyleRuleBase<Backend>>>,
    pub new_rules: Option<RuleSetHandle<Backend>>,
    pub cleared: bool,
    pub unrepresentable: bool,
}
type ParserSyntax<B>=<<B as CSSParserImplRuleBackend>::RuleDependencies as StyleRuleDependencies>::CSSSyntaxDefinition;
type ParserVariableData<B> =
    <<B as CSSParserImplRuleBackend>::RuleDependencies as StyleRuleDependencies>::CSSVariableData;
fn unavailable<T>(operation: &'static str) -> T {
    panic!("production stylesheet collaborator is unavailable: {operation}")
}
thread_local! {
    static MEDIA_EDGES:RefCell<Vec<Rc<QuerySet>>>=const{RefCell::new(Vec::new())};
    static VALUE_DIAGNOSTICS: RefCell<Vec<Diagnostic>> = const { RefCell::new(Vec::new()) };
}
pub(crate) fn ValueDiagnostic(
    kind: DiagnosticKind,
    offset: u32,
    operation: &'static str,
    source_line: u32,
) {
    VALUE_DIAGNOSTICS.with(|diagnostics| {
        diagnostics.borrow_mut().push(Diagnostic {
            kind,
            offset,
            operation,
            source_line,
        })
    });
}
struct MediaEdges(Vec<Rc<QuerySet>>, Vec<Diagnostic>);
impl MediaEdges {
    fn new() -> Self {
        Self(
            MEDIA_EDGES.with(|values| std::mem::take(&mut *values.borrow_mut())),
            VALUE_DIAGNOSTICS.with(|values| std::mem::take(&mut *values.borrow_mut())),
        )
    }
}
impl Drop for MediaEdges {
    fn drop(&mut self) {
        MEDIA_EDGES.with(|values| *values.borrow_mut() = std::mem::take(&mut self.0));
        VALUE_DIAGNOSTICS.with(|values| *values.borrow_mut() = std::mem::take(&mut self.1));
    }
}
// Source consumer adapter: CSSParserTokenStream already owns component-block
// skipping and declaration boundaries. Reuses the exact original source range
// when entering the existing production property grammar; no CSS serialization.
fn ConsumeOriginalComponentValue<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> String {
    let start = stream.LookAheadOffset();
    stream.SkipUntilPeekedTypeIs(&[]);
    let end = stream.LookAheadOffset();
    stream.StringRangeAt(start, end - start).ToString()
}

// css_variable_parser.cc:687-734. The token stream owns component-block
// traversal, so a comma boundary here only observes top-level commas. This
// keeps the parser's original text and token boundaries instead of serializing
// the consumed tokens back to CSS.
fn ConsumeOriginalVariableValue<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
    comma_ends_declaration: bool,
) -> String {
    stream.ConsumeWhitespace();
    let start = stream.LookAheadOffset();
    if comma_ends_declaration {
        stream.SkipUntilPeekedTypeIs(&[kCommaToken]);
    } else {
        stream.SkipUntilPeekedTypeIs(&[]);
    }
    let end = stream.LookAheadOffset();
    stream.StringRangeAt(start, end - start).ToString()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Invalid,
    Unsupported,
}
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub offset: u32,
    pub operation: &'static str,
    pub source_line: u32,
}
#[derive(Clone, Debug)]
pub struct SourceRange {
    pub header_start: u32,
    pub header_end: u32,
    pub body_start: u32,
    pub body_end: u32,
    pub rule_type: RuleType,
    pub declaration_end: u32,
    pub font_feature_type: Option<crate::style_rule_font_feature_values::FontFeatureType>,
}
struct SourceFrame {
    range: SourceRange,
    body: bool,
    properties: Vec<(u32, u32)>,
    children: Vec<SourceRange>,
}
#[derive(Default)]
pub struct SourceObserver {
    stack: Vec<SourceFrame>,
    source: String,
    ranges: Vec<SourceRange>,
    pub diagnostics: Vec<Diagnostic>,
    selector_ranges: Vec<std::ops::Range<u32>>,
}
impl crate::parser::css_selector_parser::CSSSelectorParserObserver for SourceObserver {
    fn ObserveSelector(&mut self, start: u32, end: u32) {
        self.selector_ranges.push(start..end);
    }
}
impl CSSParserImplObserver for SourceObserver {
    fn ObserveErroneousAtRule(&mut self, offset: u32, id: CSSAtRuleID) {
        self.diagnostics.push(Diagnostic {
            kind: if matches!(
                id,
                CSSAtRuleID::kCSSAtRuleMedia
                    | CSSAtRuleID::kCSSAtRuleLayer
                    | CSSAtRuleID::kCSSAtRuleFontFace
                    | CSSAtRuleID::kCSSAtRuleFontFeatureValues
                    | CSSAtRuleID::kCSSAtRuleStylistic
                    | CSSAtRuleID::kCSSAtRuleStyleset
                    | CSSAtRuleID::kCSSAtRuleCharacterVariant
                    | CSSAtRuleID::kCSSAtRuleSwash
                    | CSSAtRuleID::kCSSAtRuleOrnaments
                    | CSSAtRuleID::kCSSAtRuleAnnotation
                    | CSSAtRuleID::kCSSAtRuleKeyframes
                    | CSSAtRuleID::kCSSAtRuleWebkitKeyframes
                    | CSSAtRuleID::kCSSAtRuleContainer
                    | CSSAtRuleID::kCSSAtRuleProperty
                    | CSSAtRuleID::kCSSAtRuleScope
                    | CSSAtRuleID::kCSSAtRuleStartingStyle
                    | CSSAtRuleID::kCSSAtRuleSupports
                    | CSSAtRuleID::kCSSAtRuleImport
                    | CSSAtRuleID::kCSSAtRuleNamespace
            ) {
                DiagnosticKind::Invalid
            } else {
                DiagnosticKind::Unsupported
            },
            offset,
            operation: "CSSParserImpl::ConsumeAtRuleContents",
            source_line: 1244,
        });
    }
}
impl CSSParserImplRuleObserver for SourceObserver {
    fn ObserveComment(&mut self, _: u32, _: u32) {}
    fn StartRuleHeader(&mut self, rule_type: RuleType, offset: u32) {
        if self.stack.last().is_some_and(|frame| !frame.body) {
            self.stack.pop();
        }
        self.stack.push(SourceFrame {
            range: SourceRange {
                header_start: offset,
                header_end: offset,
                body_start: offset,
                body_end: offset,
                rule_type,
                declaration_end: offset,
                font_feature_type: None,
            },
            body: false,
            properties: Vec::new(),
            children: Vec::new(),
        });
    }
    fn EndRuleHeader(&mut self, offset: u32) {
        if let Some(frame) = self.stack.last_mut() {
            frame.range.header_end = offset;
        }
    }
    fn StartRuleBody(&mut self, offset: u32) {
        // inspector_css_parser_observer.cc:82-103. An identifier initially
        // observed as an invalid declaration can instead become a nested rule.
        if self.stack.len() >= 2 {
            let index = self.stack.len() - 2;
            let child_start = self.stack[index + 1].range.header_start;
            if self.stack[index]
                .properties
                .last()
                .is_some_and(|p| p.0 == child_start)
            {
                self.stack[index].properties.pop();
            }
        }
        if let Some(frame) = self.stack.last_mut() {
            frame.range.body_start = offset;
            frame.body = true;
        }
    }
    fn EndRuleBody(&mut self, offset: u32) {
        while self.stack.last().is_some_and(|frame| !frame.body) {
            self.stack.pop();
        }
        let Some(mut frame) = self.stack.pop() else {
            return;
        };
        frame.range.body_end = offset;
        frame.range.declaration_end =
            if frame.range.rule_type == RuleType::kStyle && !frame.children.is_empty() {
                frame
                    .properties
                    .last()
                    .map_or(frame.range.body_start, |p| p.1)
            } else {
                offset
            };
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(frame.range.clone());
        }
        self.ranges.push(frame.range);
    }
    fn ObserveErroneousAtRuleWithProperties(
        &mut self,
        offset: u32,
        id: CSSAtRuleID,
        _: &[CSSPropertyID],
    ) {
        self.ObserveErroneousAtRule(offset, id);
    }
    fn ObserveNestedDeclarations(&mut self, index: usize) {
        // inspector_css_parser_observer.cc:371-447. No source tokenization:
        // transfer declaration observations after the preceding child rule.
        while self.stack.last().is_some_and(|frame| !frame.body) {
            self.stack.pop();
        }
        let Some(frame) = self.stack.last_mut() else {
            return;
        };
        let preceding = index.checked_sub(1).and_then(|i| frame.children.get(i));
        let split = frame
            .properties
            .iter()
            .rposition(|p| preceding.is_some_and(|rule| p.0 <= rule.body_end))
            .map_or(0, |i| i + 1);
        let properties = frame.properties.split_off(split);
        let start = properties.first().map_or_else(
            || preceding.map_or(frame.range.body_start, |range| range.body_end + 1),
            |p| p.0,
        );
        let end = properties.last().map_or(start, |p| p.1);
        let range = SourceRange {
            header_start: start,
            header_end: start,
            body_start: start,
            body_end: end,
            declaration_end: end,
            rule_type: RuleType::kNestedDeclarations,
            font_feature_type: None,
        };
        frame
            .children
            .insert(index.min(frame.children.len()), range.clone());
        self.ranges.push(range);
    }
    fn ObserveProperty(&mut self, start: u32, end: u32, _: bool, parsed: bool) {
        let end = end
            + u32::from(
                StringView::from(&self.source).Span16().get(end as usize) == Some(&(b';' as u16)),
            );
        if let Some(frame) = self.stack.last_mut() {
            frame.properties.push((start, end));
            if !parsed
                && matches!(
                    frame.range.rule_type,
                    RuleType::kFontFace | RuleType::kKeyframe
                )
            {
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::Invalid,
                    offset: start,
                    operation: "CSSParserImpl::ConsumeDeclaration",
                    source_line: 3425,
                });
            }
        }
    }
    fn ObserveFontFeatureType(
        &mut self,
        feature: crate::style_rule_font_feature_values::FontFeatureType,
    ) {
        if let Some(frame) = self.stack.last_mut() {
            debug_assert_eq!(frame.range.rule_type, RuleType::kFontFeature);
            frame.range.font_feature_type = Some(feature);
        }
    }
}
impl RuleSetStyleScope for Unavailable {
    fn From(&self) -> Option<&CSSSelectorList> {
        match *self {}
    }
    fn To(&self) -> Option<&CSSSelectorList> {
        match *self {}
    }
    fn Parent(&self) -> Option<&Self> {
        match *self {}
    }
}
impl RuleBucketTable for Unavailable {
    fn New(_: usize) -> Self {
        unavailable("RuleBucketTable")
    }
    fn Find(&self, _: &AtomicString) -> Option<&Extent> {
        match *self {}
    }
    fn FindMut(&mut self, _: &AtomicString) -> Option<&mut Extent> {
        match *self {}
    }
    fn Insert(&mut self, _: &AtomicString) -> Option<&mut Extent> {
        match *self {}
    }
    fn Entries(&self) -> Vec<(AtomicString, Extent)> {
        match *self {}
    }
}
impl CSSUrlRequestModifiersConsumer<CSSParserContext<Backend>> for Backend {
    type Modifiers = crate::style_rule_import::CSSUrlRequestModifiers<Backend>;
    fn CSSURLRequestModifiersEnabled() -> bool {
        false
    }
    fn ConsumeUrlRequestModifiers<T: TokenStreamTokenizer>(
        _: &mut CSSParserTokenStream<'_, T>,
        _: &CSSParserContext<Self>,
        modifiers: &mut Self::Modifiers,
    ) -> bool {
        false
    }
}
/// Returned rules own the real translated selector and value allocations.
/// Source indices are UTF-16 offsets, including comments and escapes verbatim.
pub struct ProductionStyleSheet {
    pub contents: Rc<StyleSheetContents<Backend>>,
    pub source: String,
    pub rules: Vec<ProductionStyleRule>,
    pub layer_statements: Vec<LayerStatement>,
    pub font_faces: Vec<ProductionFontFaceRule>,
    pub imports: Vec<ProductionImportRule>,
    pub namespaces: Vec<Rc<crate::style_rule_namespace::StyleRuleNamespace>>,
    pub import_loads: Vec<ImportLoad>,
    pub nested_declarations: Vec<ProductionNestedDeclarationsRule>,
    pub supports: Vec<ProductionSupportsRule>,
    pub scopes: Vec<ProductionScopeRule>,
    pub property_registration_effects:
        Vec<crate::property_registration::PropertyRegistrationEffect>,
    pub keyframes: Vec<ProductionKeyframesRule>,
    pub functions: Vec<ProductionFunctionRule>,
    pub mixins: Vec<ProductionMixinRule>,
    pub selector_ranges: Vec<std::ops::Range<u32>>,
    pub diagnostics: Vec<Diagnostic>,
    projected_rule_declarations: Vec<ProjectedRuleDeclaration>,
    projected_rule_indices: HashMap<usize, usize>,
}

struct ProjectedRuleDeclaration {
    retained: Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>,
    properties: Rc<Vec<PropertyValue>>,
    source_range: SourceRange,
}

/// Result of looking up the declaration vector for one projected CSSOM rule.
///
/// `RequiresLegacyFallback` means that the production parser reported an
/// unsupported operation inside that rule's source range. Callers that still
/// surface the legacy `DocumentStyleError` property/operation details must use
/// their legacy declaration path for that rule.
#[derive(Clone)]
pub enum ProjectedRuleDeclarations {
    Typed(Rc<Vec<PropertyValue>>),
    RequiresLegacyFallback,
}

pub(crate) enum ProductionProjectionRule<'a> {
    Style(&'a ProductionStyleRule),
    NestedDeclarations(&'a ProductionNestedDeclarationsRule),
}

impl ProductionProjectionRule<'_> {
    pub(crate) fn SourceOrder(&self) -> u32 {
        match self {
            Self::Style(rule) => rule.source_order,
            Self::NestedDeclarations(rule) => rule.source_order,
        }
    }

    fn SourceRange(&self) -> &SourceRange {
        match self {
            Self::Style(rule) => &rule.source_range,
            Self::NestedDeclarations(rule) => &rule.source_range,
        }
    }

    fn Properties(&self) -> Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>> {
        match self {
            Self::Style(rule) => rule.rule.Properties(),
            Self::NestedDeclarations(rule) => rule.rule.Properties(),
        }
    }
}

impl ProductionStyleSheet {
    /// Builds the translated RuleSet directly from the retained typed sheet.
    /// Callers do not need to walk projected CSSOM rules or reconstruct media,
    /// layer, container, or scope context.
    pub fn BuildRuleSet(&self, medium: &ProductionMediaQueryEvaluator) -> RuleSet<Backend> {
        let mixins = self.contents.ExtractMixins(medium);
        self.contents.CreateUnconnectedRuleSet(medium, &mixins)
    }

    /// Returns the sheet-owned cached RuleSet, rebuilding it when media or
    /// mixin dependencies have changed.
    pub fn EnsureRuleSet(&self, medium: &ProductionMediaQueryEvaluator) -> RuleSetHandle<Backend> {
        let mixins = self.contents.ExtractMixins(medium);
        self.contents.EnsureRuleSet(medium, &mixins)
    }

    /// Returns the stylesheet-owned mixin selected by Chromium's cached
    /// `StyleSheetContents::ExtractMixins` traversal for the current medium.
    /// The returned handle retains the parser-created StyleRuleBase allocation.
    pub fn MixinForName(
        &self,
        medium: &ProductionMediaQueryEvaluator,
        name: &AtomicString,
    ) -> Option<Rc<StyleRuleBase<Backend>>> {
        self.contents
            .ExtractMixins(medium)
            .mixins
            .get(name)
            .cloned()
    }

    /// Creates the document-level function registry for this sheet. The
    /// multi-sheet constructor below is the same owner boundary used when a
    /// Document has more than one active author sheet.
    pub fn FunctionRegistry(
        &self,
        medium: &ProductionMediaQueryEvaluator,
    ) -> ProductionFunctionRegistry {
        ProductionFunctionRegistry::FromStyleSheets(&[self], medium)
    }

    /// Returns the retained typed declarations at the same `rule_index` used
    /// by `ProjectStyleSheet`. Repeated successful lookups return the same
    /// `Rc<Vec<_>>`; values are cloned from the parser's typed property store
    /// once, without serializing or parsing CSS again.
    ///
    /// The values retain the parser mode used to create this sheet. In
    /// particular, this helper does not reinterpret author declarations as UA
    /// declarations; callers needing UA semantics must parse the sheet with
    /// `CSSParserMode::kUASheetMode` at the original parse boundary.
    pub fn TypedDeclarationsForProjectedRule(
        &self,
        rule_index: usize,
    ) -> Option<Rc<Vec<PropertyValue>>> {
        match self.ProjectedRuleDeclarationsAt(rule_index)? {
            ProjectedRuleDeclarations::Typed(properties) => Some(properties),
            ProjectedRuleDeclarations::RequiresLegacyFallback => None,
        }
    }

    /// Returns the parser-owned declaration set for the projected rule.
    ///
    /// This is the ownership form consumed by MatchResult and StyleCascade in
    /// Chromium.  Keep the flattened vector accessor above while the final
    /// property application adapter is being removed, but do not make the
    /// production cascade reconstruct declaration ownership from that vector.
    pub(crate) fn RetainedDeclarationsForProjectedRule(
        &self,
        rule_index: usize,
    ) -> Option<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>> {
        let entry = self.projected_rule_declarations.get(rule_index)?;
        if self.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == DiagnosticKind::Unsupported
                && diagnostic.offset >= entry.source_range.header_start
                && diagnostic.offset <= entry.source_range.body_end
        }) {
            return None;
        }
        Some(entry.retained.clone())
    }

    /// Distinguishes an out-of-range index from a rule that must temporarily
    /// use the legacy declaration path because its source range contains an
    /// unsupported production-parser diagnostic.
    pub fn ProjectedRuleDeclarationsAt(
        &self,
        rule_index: usize,
    ) -> Option<ProjectedRuleDeclarations> {
        let entry = self.projected_rule_declarations.get(rule_index)?;
        if self.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind == DiagnosticKind::Unsupported
                && diagnostic.offset >= entry.source_range.header_start
                && diagnostic.offset <= entry.source_range.body_end
        }) {
            return Some(ProjectedRuleDeclarations::RequiresLegacyFallback);
        }
        Some(ProjectedRuleDeclarations::Typed(entry.properties.clone()))
    }

    /// Maps translated RuleSet ownership back to the CSSOM projection without
    /// serializing or reparsing selector text. Ordinary StyleRule ingestion
    /// retains this exact `Rc`, including the inner StyleRule owned by nested
    /// declarations. Mixin clones are routed separately through their bindings.
    pub(crate) fn ProjectedRuleIndexForRetainedStyleRule(
        &self,
        retained: &Rc<StyleRule<Backend>>,
    ) -> Option<usize> {
        self.projected_rule_indices
            .get(&(Rc::as_ptr(retained) as usize))
            .copied()
    }

    pub(crate) fn ProjectionRulesInSourceOrder(&self) -> Vec<ProductionProjectionRule<'_>> {
        let mut rules = self
            .rules
            .iter()
            .map(ProductionProjectionRule::Style)
            .chain(
                self.nested_declarations
                    .iter()
                    .map(ProductionProjectionRule::NestedDeclarations),
            )
            .collect::<Vec<_>>();
        rules.sort_by_key(ProductionProjectionRule::SourceOrder);
        rules
    }

    fn BuildProjectedRuleDeclarations(&mut self) {
        let projected = self
            .ProjectionRulesInSourceOrder()
            .into_iter()
            .enumerate()
            .map(|(index, rule)| {
                let properties = rule.Properties();
                let identity = match rule {
                    ProductionProjectionRule::Style(rule) => Some(Rc::as_ptr(&rule.rule) as usize),
                    ProductionProjectionRule::NestedDeclarations(rule) => {
                        Some(Rc::as_ptr(rule.rule.InnerStyleRule()) as usize)
                    }
                };
                (
                    identity,
                    index,
                    ProjectedRuleDeclaration {
                        retained: properties.clone(),
                        properties: Rc::new(match properties.as_ref() {
                            CSSPropertyValueSetRuleHandle::Immutable(set) => {
                                set.Properties().to_vec()
                            }
                            CSSPropertyValueSetRuleHandle::Mutable(set) => {
                                set.borrow().Properties().to_vec()
                            }
                        }),
                        source_range: rule.SourceRange().clone(),
                    },
                )
            })
            .collect::<Vec<_>>();
        self.projected_rule_indices = projected
            .iter()
            .filter_map(|(identity, index, _)| identity.map(|identity| (identity, *index)))
            .collect();
        self.projected_rule_declarations = projected
            .into_iter()
            .map(|(_, _, declaration)| declaration)
            .collect();
    }
}
pub struct ProductionStyleRule {
    pub rule: Rc<StyleRule<Backend>>,
    pub selector_source: String,
    pub declaration_source: String,
    pub scope_conditions: Vec<Rc<crate::style_scope::StyleScope>>,
    pub container_conditions: Vec<Rc<ContainerSet>>,
    pub media: Vec<Rc<QuerySet>>,
    pub layers: Vec<LayerSegment>,
    pub source_order: u32,
    pub source_range: SourceRange,
    pub parent_rule_for_nesting: Option<Rc<StyleRule<Backend>>>,
}
pub type ProductionFunctionHandle = TypedRuleRef<Backend, FunctionRuleKind>;

/// Parser/source projection for one retained @function definition. Runtime
/// lookup uses ProductionFunctionRegistry, which applies media and layer order
/// through the translated RuleSet owner.
pub struct ProductionFunctionRule {
    pub rule: Rc<StyleRuleBase<Backend>>,
    pub media: Vec<Rc<QuerySet>>,
    pub layers: Vec<LayerSegment>,
    pub source_order: u32,
    pub source_range: SourceRange,
}

/// Parser/source projection for one retained @mixin definition. Active lookup
/// remains owned by StyleSheetContents::ExtractMixins, including its media
/// dependency cache.
pub struct ProductionMixinRule {
    pub rule: Rc<StyleRuleBase<Backend>>,
    pub media: Vec<Rc<QuerySet>>,
    pub source_order: u32,
    pub source_range: SourceRange,
}

/// Document-level @function owner corresponding to ScopedStyleResolver's
/// FunctionRuleMap. Values retain the parser-created base rule through
/// TypedRuleRef; no function declaration is copied or reparsed.
pub struct ProductionFunctionRegistry {
    functions: HashMap<AtomicString, CascadeLayered<ProductionFunctionHandle>>,
}

impl ProductionFunctionRegistry {
    // resolver/style_resolver_utils.h:37-57 and scoped_style_resolver.cc:137-154.
    pub fn FromStyleSheets(
        sheets: &[&ProductionStyleSheet],
        medium: &ProductionMediaQueryEvaluator,
    ) -> Self {
        let rule_sets = sheets
            .iter()
            .map(|sheet| Rc::new(sheet.BuildRuleSet(medium)))
            .collect::<Vec<_>>();
        Self::FromRuleSets(&rule_sets)
    }

    // ScopedStyleResolver::AppendActiveStyleSheets collects FunctionRules from
    // the already compiled active RuleSets. Ordinary recalc must not compile
    // every sheet again just to consult its document-level name map.
    pub(crate) fn FromRuleSets(rule_sets: &[Rc<RuleSet<Backend>>]) -> Self {
        let active = rule_sets
            .iter()
            .map(|rules| ActiveStyleSheet::new(Rc::new(()), Some(rules.clone())))
            .collect::<Vec<_>>();
        let layers = CascadeLayerMap::new(&active);
        let mut functions: HashMap<AtomicString, CascadeLayered<ProductionFunctionHandle>> =
            HashMap::new();
        for rules in rule_sets {
            for rule in rules.FunctionRules() {
                let name = rule.value.Name().clone();
                let replace = functions.get(&name).is_none_or(|old| {
                    let order = if old.layer == rule.layer {
                        std::cmp::Ordering::Equal
                    } else {
                        layers.CompareLayerOrder(old.layer.as_ref(), rule.layer.as_ref())
                    };
                    order != std::cmp::Ordering::Greater
                });
                if replace {
                    functions.insert(name, rule.clone());
                }
            }
        }
        Self { functions }
    }

    pub fn FunctionForName(&self, name: &AtomicString) -> Option<ProductionFunctionHandle> {
        self.functions.get(name).map(|rule| rule.value.clone())
    }

    pub fn len(&self) -> usize {
        self.functions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}
pub type ImportMetadata = crate::style_rule_import::ParsedStyleRuleImport<
    Backend,
    crate::style_scope::StyleScope,
    QuerySet,
>;
pub struct ConcreteImport {
    pub metadata: ImportMetadata,
    parent: RefCell<Option<std::rc::Weak<StyleSheetContents<Backend>>>>,
    request: std::cell::Cell<bool>,
}
pub struct ProductionImportRule {
    pub rule: Rc<ConcreteImport>,
    pub source_range: SourceRange,
    pub source_order: u32,
}
/// The caller can resolve/fetch this request through its document loader.
/// Parsing does not perform network I/O or mark a resource as loaded.
pub struct ImportLoad {
    pub rule: Rc<ConcreteImport>,
    pub source_order: u32,
    pub source_range: SourceRange,
}
pub struct ProductionNestedDeclarationsRule {
    pub rule: Rc<crate::style_rule_nested_declarations::StyleRuleNestedDeclarations<Backend>>,
    pub declaration_source: String,
    pub scope_conditions: Vec<Rc<crate::style_scope::StyleScope>>,
    pub container_conditions: Vec<Rc<ContainerSet>>,
    pub media: Vec<Rc<QuerySet>>,
    pub layers: Vec<LayerSegment>,
    pub source_order: u32,
    pub source_range: SourceRange,
}
pub struct ProductionScopeRule {
    pub prelude_source: String,
    pub scope: Rc<crate::style_scope::StyleScope>,
    pub source_range: SourceRange,
    pub source_order: u32,
}
pub struct ProductionSupportsRule {
    pub rule: Rc<StyleRuleBase<Backend>>,
    pub source_range: SourceRange,
    pub source_order: u32,
}
pub type ConcreteKeyframe = crate::style_rule_keyframe::StyleRuleKeyframe<Backend>;
// Parser-only collaborator for css_keyframes_rule.h:42-81. The CSSOM adapter
// can project these Rc keyframe/property owners without any reparsing.
pub struct ConcreteKeyframes {
    pub name: AtomicString,
    pub vendor_prefixed: bool,
    pub keys: Vec<Rc<ConcreteKeyframe>>,
}
pub struct ProductionFontFaceRule {
    pub rule: StyleRuleFontFace<Backend>,
    pub declaration_source: String,
    pub media: Vec<Rc<QuerySet>>,
    pub layers: Vec<LayerSegment>,
    pub source_order: u32,
    pub source_range: SourceRange,
}
pub struct ProductionKeyframeRule {
    pub rule: Rc<ConcreteKeyframe>,
    pub key_source: String,
    pub declaration_source: String,
    pub source_range: SourceRange,
}
pub struct ProductionKeyframesRule {
    pub rule: Rc<ConcreteKeyframes>,
    pub keyframes: Vec<ProductionKeyframeRule>,
    pub media: Vec<Rc<QuerySet>>,
    pub layers: Vec<LayerSegment>,
    pub source_order: u32,
    pub source_range: SourceRange,
}
fn SourceSlice(source: &String, start: u32, end: u32) -> String {
    StringView::from(source)
        .Substring(start, end.saturating_sub(start))
        .ToString()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayerSegment {
    Named(Vec<AtomicString>),
    Anonymous(u32),
}
pub struct LayerStatement {
    pub names: Vec<LayerName>,
    pub parent_layers: Vec<LayerSegment>,
    pub source_order: u32,
}
pub fn ParseStyleSheet(text: &String, mode: CSSParserMode) -> ProductionStyleSheet {
    ParseStyleSheetWithURL(text, mode, SourceURL::Null)
}

/// Parses a stylesheet with the response/document URL installed on the same
/// CSSParserContext that owns its typed declarations. URL-bearing values then
/// retain both author spelling and resolved resource identity.
pub fn ParseStyleSheetWithBase(
    text: &String,
    mode: CSSParserMode,
    base_url: &str,
) -> ProductionStyleSheet {
    ParseStyleSheetWithURL(text, mode, SourceURL::FromString(base_url))
}

pub(crate) fn InlineStyleSheetParserContext() -> Rc<CSSParserContext<Backend>> {
    StyleSheetParserContext(CSSParserMode::kHTMLStandardMode, SourceURL::Null)
}

fn StyleSheetParserContext(
    mode: CSSParserMode,
    base_url: SourceURL,
) -> Rc<CSSParserContext<Backend>> {
    Rc::new(CSSParserContext::<Backend>::new(
        base_url,
        true,
        None,
        mode,
        String::from(""),
        false,
        SecureContextMode::kInsecureContext,
        None,
        None,
        crate::css_resource_fetch_restriction::ResourceFetchRestriction::kNone,
    ))
}

fn ParseStyleSheetWithURL(
    text: &String,
    mode: CSSParserMode,
    base_url: SourceURL,
) -> ProductionStyleSheet {
    let _edges = MediaEdges::new();
    let context = StyleSheetParserContext(mode, base_url);
    let mut contents = StyleSheetContents::<Backend>::new(context.clone(), String::default(), None);
    let mut observer = SourceObserver {
        source: text.clone(),
        ..SourceObserver::default()
    };
    CSSParserImpl::<Backend>::ParseStyleSheetForInspector(
        text,
        Some(&context),
        &mut contents,
        &mut observer,
    );
    observer
        .diagnostics
        .extend(VALUE_DIAGNOSTICS.with(|values| std::mem::take(&mut *values.borrow_mut())));
    observer.ranges.sort_by_key(|range| range.header_start);
    let mut output = ProductionStyleSheet {
        contents: contents.clone(),
        source: text.clone(),
        rules: Vec::new(),
        layer_statements: Vec::new(),
        font_faces: Vec::new(),
        imports: Vec::new(),
        namespaces: Vec::new(),
        import_loads: Vec::new(),
        nested_declarations: Vec::new(),
        supports: Vec::new(),
        scopes: Vec::new(),
        property_registration_effects: Vec::new(),
        keyframes: Vec::new(),
        functions: Vec::new(),
        mixins: Vec::new(),
        selector_ranges: observer.selector_ranges,
        diagnostics: observer.diagnostics,
        projected_rule_declarations: Vec::new(),
        projected_rule_indices: HashMap::new(),
    };
    let mut ranges = observer.ranges;
    let mut order = 0;
    for index in 0..contents.RuleCount() {
        Flatten(
            &contents.RuleAt(index),
            text,
            &mut output,
            &mut ranges,
            &[],
            &[],
            &[],
            &[],
            &mut order,
            None,
            true,
        );
    }
    output.BuildProjectedRuleDeclarations();
    output
}
fn TakeRange(ranges: &mut Vec<SourceRange>, rule_type: RuleType) -> Option<SourceRange> {
    ranges
        .iter()
        .position(|range| range.rule_type == rule_type)
        .map(|index| ranges.remove(index))
}
fn Flatten(
    rule: &Rc<StyleRuleBase<Backend>>,
    source: &String,
    output: &mut ProductionStyleSheet,
    ranges: &mut Vec<SourceRange>,
    media: &[Rc<QuerySet>],
    containers: &[Rc<ContainerSet>],
    scopes: &[Rc<crate::style_scope::StyleScope>],
    layers: &[LayerSegment],
    order: &mut u32,
    parent: Option<Rc<StyleRule<Backend>>>,
    active: bool,
) {
    let position = *order;
    *order += 1;
    match rule.as_ref() {
        StyleRuleBase::Style(rule) => {
            let Some(range) = TakeRange(ranges, RuleType::kStyle) else {
                return;
            };
            let source_view = StringView::from(source);
            output.rules.push(ProductionStyleRule {
                rule: rule.clone(),
                selector_source: source_view
                    .Substring(range.header_start, range.header_end - range.header_start)
                    .ToString(),
                declaration_source: source_view
                    .Substring(range.body_start, range.declaration_end - range.body_start)
                    .ToString(),
                scope_conditions: scopes.to_vec(),
                container_conditions: containers.to_vec(),
                media: media.to_vec(),
                layers: layers.to_vec(),
                source_order: position,
                source_range: range,
                parent_rule_for_nesting: parent.clone(),
            });
            if !active {
                output.rules.pop();
            }
            if let Some(children) = rule.ChildRules() {
                for child in children.iter() {
                    Flatten(
                        child,
                        source,
                        output,
                        ranges,
                        media,
                        containers,
                        scopes,
                        layers,
                        order,
                        Some(rule.clone()),
                        active,
                    );
                }
            }
        }
        StyleRuleBase::FontFace(rule) => {
            let Some(range) = TakeRange(ranges, RuleType::kFontFace) else {
                return;
            };
            output.font_faces.push(ProductionFontFaceRule {
                rule: StyleRuleFontFace::new(match rule.Properties() {
                    CSSPropertyValueSetRuleHandle::Immutable(set) => {
                        CSSPropertyValueSetRuleHandle::FromImmutable(set.clone())
                    }
                    CSSPropertyValueSetRuleHandle::Mutable(set) => {
                        CSSPropertyValueSetRuleHandle::FromMutable(set.clone())
                    }
                }),
                declaration_source: SourceSlice(source, range.body_start, range.body_end),
                media: media.to_vec(),
                layers: layers.to_vec(),
                source_order: position,
                source_range: range,
            });
            if !active || !containers.is_empty() {
                let removed = output.font_faces.pop();
                if !containers.is_empty() {
                    output.diagnostics.push(Diagnostic {
                        kind: DiagnosticKind::Unsupported,
                        offset: removed.map_or(0, |rule| rule.source_range.header_start),
                        operation: "container-dependent font-face registration",
                        source_line: 1606,
                    });
                }
            }
        }
        StyleRuleBase::Keyframes(rule) => {
            let Some(range) = TakeRange(ranges, RuleType::kKeyframes) else {
                return;
            };
            let mut keyframes = Vec::new();
            for keyframe in &rule.keys {
                let Some(child_range) = TakeRange(ranges, RuleType::kKeyframe) else {
                    break;
                };
                keyframes.push(ProductionKeyframeRule {
                    rule: keyframe.clone(),
                    key_source: SourceSlice(
                        source,
                        child_range.header_start,
                        child_range.header_end,
                    ),
                    declaration_source: SourceSlice(
                        source,
                        child_range.body_start,
                        child_range.body_end,
                    ),
                    source_range: child_range,
                });
            }
            output.keyframes.push(ProductionKeyframesRule {
                rule: rule.clone(),
                keyframes,
                media: media.to_vec(),
                layers: layers.to_vec(),
                source_order: position,
                source_range: range,
            });
            if !active || !containers.is_empty() {
                let removed = output.keyframes.pop();
                if !containers.is_empty() {
                    output.diagnostics.push(Diagnostic {
                        kind: DiagnosticKind::Unsupported,
                        offset: removed.map_or(0, |rule| rule.source_range.header_start),
                        operation: "container-dependent keyframes registration",
                        source_line: 1606,
                    });
                }
            }
        }
        StyleRuleBase::Scope(scope_rule) => {
            let scope = Rc::new(crate::style_scope::StyleScope::new(
                scope_rule.GetStyleScope().FromRule(),
                scope_rule.GetStyleScope().ToList(),
            ));
            if let Some(range) = TakeRange(ranges, RuleType::kScope) {
                output.scopes.push(ProductionScopeRule {
                    prelude_source: SourceSlice(source, range.header_start, range.header_end),
                    scope: scope.clone(),
                    source_range: range,
                    source_order: position,
                });
            }
            let mut scope_chain = scopes.to_vec();
            scope_chain.push(scope);
            for child in scope_rule.ChildRules() {
                Flatten(
                    child,
                    source,
                    output,
                    ranges,
                    media,
                    containers,
                    &scope_chain,
                    layers,
                    order,
                    parent.clone(),
                    active,
                );
            }
        }
        StyleRuleBase::Property(property) => {
            let Some(range) = TakeRange(ranges, RuleType::kProperty) else {
                return;
            };
            if active {
                if let (Some(syntax), Some(inherits)) = (
                    crate::property_registration::ConvertSyntax(property.GetSyntax().as_deref()),
                    crate::property_registration::ConvertInherits(property.Inherits().as_deref()),
                ) {
                    if let Ok(initial) = crate::property_registration::ConvertInitial(
                        property.GetInitialValue().as_deref(),
                        &syntax,
                    ) {
                        output.property_registration_effects.push(
                            crate::property_registration::PropertyRegistrationEffect {
                                name: AtomicString::from_utf16(
                                    property.GetName().Span16().unwrap_or_default(),
                                ),
                                syntax_text: property
                                    .GetSyntax()
                                    .and_then(|v| match v.Payload() {
                                        CSSValuePayload::kStringClass(v) => Some(v.0.clone()),
                                        _ => None,
                                    })
                                    .unwrap_or_default(),
                                initial_source: property.GetInitialValue().and_then(|v| {
                                    match v.Payload() {
                                        CSSValuePayload::kUnparsedDeclarationClass(v) => {
                                            Some(v.data.clone())
                                        }
                                        _ => None,
                                    }
                                }),
                                syntax,
                                inherits,
                                initial,
                                source_order: position,
                                media: media.to_vec(),
                                layers: layers.to_vec(),
                                source_range: range,
                            },
                        );
                    }
                }
            }
        }
        StyleRuleBase::Container(container) => {
            TakeRange(ranges, RuleType::kContainer);
            let mut chain = containers.to_vec();
            chain.push(Rc::new(container.GetContainerQuerySet().clone()));
            for child in container.ChildRules() {
                Flatten(
                    child,
                    source,
                    output,
                    ranges,
                    media,
                    &chain,
                    scopes,
                    layers,
                    order,
                    parent.clone(),
                    active,
                );
            }
        }
        StyleRuleBase::Supports(supports_rule) => {
            if let Some(range) = TakeRange(ranges, RuleType::kSupports) {
                output.supports.push(ProductionSupportsRule {
                    rule: rule.clone(),
                    source_range: range,
                    source_order: position,
                });
            }
            for child in supports_rule.ChildRules() {
                Flatten(
                    child,
                    source,
                    output,
                    ranges,
                    media,
                    containers,
                    scopes,
                    layers,
                    order,
                    parent.clone(),
                    active && supports_rule.ConditionIsSupported(),
                );
            }
        }
        StyleRuleBase::StartingStyle(starting_style_rule) => {
            TakeRange(ranges, RuleType::kStartingStyle);
            for child in starting_style_rule.ChildRules() {
                Flatten(
                    child,
                    source,
                    output,
                    ranges,
                    media,
                    containers,
                    scopes,
                    layers,
                    order,
                    parent.clone(),
                    active,
                );
            }
        }
        StyleRuleBase::Import(rule) => {
            if let Some(range) = TakeRange(ranges, RuleType::kImport) {
                output.imports.push(ProductionImportRule {
                    rule: rule.clone(),
                    source_range: range.clone(),
                    source_order: position,
                });
                if rule.request.get() {
                    output.import_loads.push(ImportLoad {
                        rule: rule.clone(),
                        source_order: position,
                        source_range: range,
                    });
                }
            }
        }
        StyleRuleBase::Namespace(rule) => output.namespaces.push(rule.clone()),
        StyleRuleBase::NestedDeclarations(rule) => {
            if let Some(range) = TakeRange(ranges, RuleType::kNestedDeclarations) {
                if active {
                    output
                        .nested_declarations
                        .push(ProductionNestedDeclarationsRule {
                            rule: rule.clone(),
                            declaration_source: SourceSlice(
                                source,
                                range.body_start,
                                range.body_end,
                            ),
                            scope_conditions: scopes.to_vec(),
                            container_conditions: containers.to_vec(),
                            media: media.to_vec(),
                            layers: layers.to_vec(),
                            source_order: position,
                            source_range: range,
                        });
                }
            }
        }
        StyleRuleBase::Media(rule) => {
            let mut media = media.to_vec();
            let query = rule.MediaQueries().unwrap();
            media.push(
                MEDIA_EDGES
                    .with(|values| {
                        values
                            .borrow()
                            .iter()
                            .find(|edge| std::ptr::eq(edge.as_ref(), query))
                            .cloned()
                    })
                    .expect("media query edge retained for parser lifetime"),
            );
            for child in rule.ChildRules() {
                Flatten(
                    child,
                    source,
                    output,
                    ranges,
                    &media,
                    containers,
                    scopes,
                    layers,
                    order,
                    parent.clone(),
                    active,
                );
            }
        }
        StyleRuleBase::LayerBlock(rule) => {
            let mut layers = layers.to_vec();
            layers.push(
                if rule.GetName().len() == 1 && rule.GetName()[0].Utf8().is_empty() {
                    LayerSegment::Anonymous(position)
                } else {
                    LayerSegment::Named(rule.GetName().clone())
                },
            );
            for child in rule.ChildRules() {
                Flatten(
                    child,
                    source,
                    output,
                    ranges,
                    media,
                    containers,
                    scopes,
                    &layers,
                    order,
                    parent.clone(),
                    active,
                );
            }
        }
        StyleRuleBase::LayerStatement(rule) => {
            output.layer_statements.push(LayerStatement {
                names: rule.GetNames().clone(),
                parent_layers: layers.to_vec(),
                source_order: position,
            });
        }
        StyleRuleBase::Function(_) => {
            if let Some(range) = TakeRange(ranges, RuleType::kFunction) {
                if active {
                    output.functions.push(ProductionFunctionRule {
                        rule: rule.clone(),
                        media: media.to_vec(),
                        layers: layers.to_vec(),
                        source_order: position,
                        source_range: range,
                    });
                }
            }
        }
        StyleRuleBase::Mixin(_) => {
            if let Some(range) = TakeRange(ranges, RuleType::kMixin) {
                if active {
                    output.mixins.push(ProductionMixinRule {
                        rule: rule.clone(),
                        media: media.to_vec(),
                        source_order: position,
                        source_range: range,
                    });
                }
            }
        }
        StyleRuleBase::FontFeatureValues(_) | StyleRuleBase::FontFeature(_) => {
            // The native parser-owned rules are ingested by RuleSet and the
            // document font-feature registry. They have no independent flat
            // style-rule projection; do not classify retained rules as missing.
        }
        StyleRuleBase::ApplyMixin(_)
        | StyleRuleBase::Contents(_)
        | StyleRuleBase::Result(_)
        | StyleRuleBase::FunctionDeclarations(_) => {
            // These rules are retained below @function/@mixin/style owners and
            // consumed by RuleSet expansion. They do not independently project
            // into the flat CSSOM style-rule view.
        }
        _ => output.diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Unsupported,
            offset: 0,
            operation: "production stylesheet rule projection",
            source_line: 1606,
        }),
    }
}
impl CSSParserImplBackend for Backend {
    type Platform = Backend;
    type ValueBackend = ProductionCSSValueDispatch;
    type StyleSheetContents = Rc<StyleSheetContents<Backend>>;
    type CSSParserObserver = SourceObserver;
    type CSSLazyParsingState = Unavailable;
    type MediaQuerySet = QuerySet;
    type StyleRule = StyleRule<Backend>;
    type StyleRuleBase = StyleRuleBase<Backend>;
    fn AtRuleFeatures() -> AtRuleRuntimeFeatures {
        AtRuleRuntimeFeatures {
            css_mixins: true,
            css_functions: true,
            ..AtRuleRuntimeFeatures::default()
        }
    }
    fn ConsumeAtRuleContents<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        id: CSSAtRuleID,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed_rules: AllowedRules,
        nesting_type: CSSNestingType,
        parent_rule_for_nesting: Option<Rc<Self::StyleRule>>,
    ) -> Option<Rc<Self::StyleRuleBase>> {
        if !allowed_rules.Has(id) {
            parser.ConsumeErroneousAtRule(stream, id);
            return None;
        }
        match id {
            CSSAtRuleID::kCSSAtRuleMedia => {
                parser.ConsumeMediaRule(stream, nesting_type, parent_rule_for_nesting)
            }
            CSSAtRuleID::kCSSAtRuleLayer => {
                parser.ConsumeLayerRule(stream, nesting_type, parent_rule_for_nesting)
            }
            CSSAtRuleID::kCSSAtRuleContainer => {
                parser.ConsumeContainerRule(stream, nesting_type, parent_rule_for_nesting)
            }
            CSSAtRuleID::kCSSAtRuleProperty => parser.ConsumePropertyRule(stream),
            CSSAtRuleID::kCSSAtRuleScope => {
                parser.ConsumeScopeRule(stream, nesting_type, parent_rule_for_nesting)
            }
            CSSAtRuleID::kCSSAtRuleSupports => {
                parser.ConsumeSupportsRule(stream, nesting_type, parent_rule_for_nesting)
            }
            CSSAtRuleID::kCSSAtRuleStartingStyle => {
                parser.ConsumeStartingStyleRule(stream, nesting_type, parent_rule_for_nesting)
            }
            CSSAtRuleID::kCSSAtRuleImport => {
                let modifiers = <Backend as CSSParserImplRuleBackend>::NewUrlRequestModifiers();
                let uri = ConsumeStringOrURI::<_, Backend, _>(stream, parser.GetContext()?, None);
                parser.ConsumeImportRule(uri, stream, &modifiers)
            }
            CSSAtRuleID::kCSSAtRuleNamespace => parser.ConsumeNamespaceRule(stream),
            CSSAtRuleID::kCSSAtRuleFontFace => parser.ConsumeFontFaceRule(stream),
            CSSAtRuleID::kCSSAtRuleFontFeatureValues => parser.ConsumeFontFeatureValuesRule(stream),
            CSSAtRuleID::kCSSAtRuleStylistic
            | CSSAtRuleID::kCSSAtRuleStyleset
            | CSSAtRuleID::kCSSAtRuleCharacterVariant
            | CSSAtRuleID::kCSSAtRuleSwash
            | CSSAtRuleID::kCSSAtRuleOrnaments
            | CSSAtRuleID::kCSSAtRuleAnnotation => parser
                .ConsumeFontFeatureRule(id, stream)
                .map(|rule| Rc::new(StyleRuleBase::FontFeature(Rc::new(rule)))),
            CSSAtRuleID::kCSSAtRuleKeyframes => parser.ConsumeKeyframesRule(false, stream),
            CSSAtRuleID::kCSSAtRuleWebkitKeyframes => parser.ConsumeKeyframesRule(true, stream),
            CSSAtRuleID::kCSSAtRuleFunction => parser.ConsumeFunctionRule(stream),
            CSSAtRuleID::kCSSAtRuleMixin => parser.ConsumeMixinRule(stream),
            CSSAtRuleID::kCSSAtRuleApplyMixin => parser.ConsumeApplyMixinRule(stream),
            CSSAtRuleID::kCSSAtRuleContents => parser.ConsumeContentsRule(stream),
            CSSAtRuleID::kCSSAtRuleResult => parser.ConsumeResultRule(stream),
            _ => {
                parser.ConsumeErroneousAtRule(stream, id);
                None
            }
        }
    }
    fn ConsumeQualifiedRule<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed_rules: AllowedRules,
        nesting_type: CSSNestingType,
        parent_rule_for_nesting: Option<Rc<Self::StyleRule>>,
    ) -> Option<Rc<Self::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let result = parser.ConsumeQualifiedRule(
            stream,
            allowed_rules,
            nesting_type,
            parent_rule_for_nesting,
        );
        if result.is_none()
            && allowed_rules.Has(crate::parser::allowed_rules::QualifiedRuleType::kKeyframe)
        {
            ValueDiagnostic(
                DiagnosticKind::Invalid,
                start,
                "CSSParserImpl::ConsumeKeyframeKeyList",
                3599,
            );
        }
        result
    }
}

// cpp: at_rule_descriptor_parser.cc:57-86,121-278,335-433.
// BlockGuard and EnableUnicodeRanges retain the translated token semantics.
type DescriptorResult = Result<Rc<values::Value>, DiagnosticKind>;
fn ConsumeFamilyName<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
    descriptor: bool,
) -> Option<String> {
    if stream.Peek().GetType() == kStringToken {
        return Some(stream.ConsumeIncludingWhitespace().Value().ToString());
    }
    let first = stream.Peek().Value().ToString().Utf8().to_ascii_lowercase();
    if descriptor
        && [
            "serif",
            "sans-serif",
            "monospace",
            "cursive",
            "fantasy",
            "system-ui",
            "ui-serif",
            "ui-sans-serif",
            "ui-monospace",
            "ui-rounded",
            "math",
            "fangsong",
        ]
        .contains(&first.as_str())
    {
        return None;
    }
    let mut words = Vec::new();
    while stream.Peek().GetType() == kIdentToken {
        words.push(
            stream
                .ConsumeIncludingWhitespace()
                .Value()
                .ToString()
                .Utf8(),
        );
    }
    if words.len() == 1
        && [
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "default",
        ]
        .contains(&first.as_str())
    {
        return None;
    }
    (!words.is_empty())
        .then(|| String::from_utf16(&words.join(" ").encode_utf16().collect::<Vec<_>>()))
}
fn ConsumeFontSource<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
    context: &CSSParserContext<Backend>,
) -> Result<values::CSSFontFaceSrcValue, DiagnosticKind> {
    use values::FontFaceResource;
    let resource = if stream.Peek().FunctionId() == Some(CSSValueID::kLocal) {
        let mut guard = BlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let name = ConsumeFamilyName(&mut guard, false).ok_or(DiagnosticKind::Invalid)?;
        if !guard.AtEnd() {
            return Err(DiagnosticKind::Invalid);
        }
        FontFaceResource::Local(name)
    } else if stream.Peek().GetType() == kUrlToken {
        let url = stream.ConsumeIncludingWhitespace().Value().ToString();
        FontFaceResource::Url(values::CSSUrlData::new(
            url.clone(),
            context.CompleteNonEmptyURL(&url),
        ))
    } else if stream.Peek().FunctionId() == Some(CSSValueID::kUrl) {
        let mut guard = BlockGuard::new(stream);
        guard.ConsumeWhitespace();
        if guard.Peek().GetType() != kStringToken {
            return Err(DiagnosticKind::Invalid);
        }
        let url = guard.ConsumeIncludingWhitespace().Value().ToString();
        if !guard.AtEnd() {
            return Err(DiagnosticKind::Invalid);
        }
        FontFaceResource::Url(values::CSSUrlData::new(
            url.clone(),
            context.CompleteNonEmptyURL(&url),
        ))
    } else {
        return Err(DiagnosticKind::Invalid);
    };
    stream.ConsumeWhitespace();
    let mut value = values::CSSFontFaceSrcValue {
        resource,
        format: None,
        technologies: Vec::new(),
    };
    if matches!(value.resource, FontFaceResource::Url(_)) {
        if stream.Peek().FunctionId() == Some(CSSValueID::kFormat) {
            let mut guard = BlockGuard::new(stream);
            guard.ConsumeWhitespace();
            if !matches!(guard.Peek().GetType(), kIdentToken | kStringToken) {
                return Err(DiagnosticKind::Invalid);
            }
            let token = guard.ConsumeIncludingWhitespace();
            let format = token.Value().ToString();
            let keyword = format.Utf8().to_ascii_lowercase();
            let supported = ["collection", "opentype", "truetype", "woff", "woff2"]
                .contains(&keyword.as_str())
                || token.GetType() == kStringToken
                    && [
                        "woff-variations",
                        "truetype-variations",
                        "opentype-variations",
                        "woff2-variations",
                    ]
                    .contains(&keyword.as_str());
            if !supported || !guard.AtEnd() {
                return Err(DiagnosticKind::Invalid);
            }
            value.format = Some(format);
        }
        stream.ConsumeWhitespace();
        if stream.Peek().FunctionId() == Some(CSSValueID::kTech) {
            let mut guard = BlockGuard::new(stream);
            guard.ConsumeWhitespace();
            loop {
                let id = guard.Peek().Id();
                if !matches!(
                    id,
                    CSSValueID::kFeaturesOpentype
                        | CSSValueID::kFeaturesAat
                        | CSSValueID::kColorCOLRv0
                        | CSSValueID::kColorCOLRv1
                        | CSSValueID::kColorSbix
                        | CSSValueID::kColorCBDT
                        | CSSValueID::kVariations
                        | CSSValueID::kPalettes
                ) {
                    return Err(DiagnosticKind::Unsupported);
                }
                guard.ConsumeIncludingWhitespace();
                if !value.technologies.contains(&id) {
                    value.technologies.push(id);
                }
                if guard.AtEnd() {
                    break;
                }
                if guard.Peek().GetType() != kCommaToken {
                    return Err(DiagnosticKind::Invalid);
                }
                guard.ConsumeIncludingWhitespace();
            }
        }
    }
    stream.ConsumeWhitespace();
    if !stream.AtEnd() && stream.Peek().GetType() != kCommaToken {
        return Err(DiagnosticKind::Invalid);
    }
    Ok(value)
}
fn ParseFontFaceDescriptor(
    id: crate::parser::at_rule_descriptors::AtRuleDescriptorID,
    text: &String,
    source_offset: u32,
    context: &CSSParserContext<Backend>,
) -> DescriptorResult {
    use crate::css_primitive_value::UnitType;
    use crate::parser::at_rule_descriptors::AtRuleDescriptorID as D;
    let mut stream: CSSParserTokenStream<crate::parser::css_tokenizer::CSSTokenizer> =
        CSSParserTokenStream::new(StringView::from(text), 0);
    let mut stream = EnableUnicodeRanges::new(&mut stream, id == D::UnicodeRange);
    stream.ConsumeWhitespace();
    let value = match id {
        D::FontFamily => {
            let family = ConsumeFamilyName(&mut stream, true).ok_or(DiagnosticKind::Invalid)?;
            Rc::new(values::Value::new(CSSValuePayload::kFontFamilyClass(
                values::CSSFontFamilyValue(family),
            )))
        }
        D::Src => {
            let mut sources = Vec::new();
            loop {
                let offset = source_offset + stream.LookAheadOffset();
                match ConsumeFontSource(&mut stream, context) {
                    Ok(value) => sources.push(Rc::new(values::Value::new(
                        CSSValuePayload::kFontFaceSrcClass(value),
                    ))),
                    Err(kind) => {
                        ValueDiagnostic(
                            kind,
                            offset,
                            "AtRuleDescriptorParser::ConsumeFontFaceSrc",
                            256,
                        );
                        stream.SkipUntilPeekedTypeIs(&[kCommaToken]);
                    }
                }
                if stream.Peek().GetType() != kCommaToken {
                    break;
                }
                stream.ConsumeIncludingWhitespace();
            }
            if sources.is_empty() {
                return Err(DiagnosticKind::Invalid);
            }
            values::list(sources, values::ListSeparator::Comma)
        }
        D::UnicodeRange => {
            let mut ranges = Vec::new();
            loop {
                if stream.Peek().GetType() != kUnicodeRangeToken {
                    return Err(DiagnosticKind::Invalid);
                }
                let token = stream.ConsumeIncludingWhitespace();
                let from = token.UnicodeRangeStart();
                let to = token.UnicodeRangeEnd();
                if from < 0 || from > to || to > 0x10ffff {
                    return Err(DiagnosticKind::Invalid);
                }
                ranges.push(Rc::new(values::Value::new(
                    CSSValuePayload::kUnicodeRangeClass(values::CSSUnicodeRangeValue {
                        from: from as u32,
                        to: to as u32,
                    }),
                )));
                if stream.Peek().GetType() != kCommaToken {
                    break;
                }
                stream.ConsumeIncludingWhitespace();
            }
            values::list(ranges, values::ListSeparator::Comma)
        }
        D::FontDisplay => {
            let keyword = stream.Peek().Id();
            if !matches!(
                keyword,
                CSSValueID::kAuto
                    | CSSValueID::kBlock
                    | CSSValueID::kSwap
                    | CSSValueID::kFallback
                    | CSSValueID::kOptional
            ) {
                return Err(DiagnosticKind::Invalid);
            }
            stream.ConsumeIncludingWhitespace();
            values::identifier(keyword)
        }
        D::FontWeight => {
            let keyword = stream.Peek().Id();
            if matches!(
                keyword,
                CSSValueID::kNormal | CSSValueID::kBold | CSSValueID::kAuto
            ) {
                stream.ConsumeIncludingWhitespace();
                values::identifier(keyword)
            } else {
                let mut weights = Vec::new();
                for _ in 0..2 {
                    if stream.Peek().GetType() == kFunctionToken {
                        return Err(DiagnosticKind::Unsupported);
                    }
                    if stream.Peek().GetType() != kNumberToken
                        || !(1.0..=1000.0).contains(&stream.Peek().NumericValue())
                    {
                        return Err(DiagnosticKind::Invalid);
                    }
                    weights.push(values::numeric(
                        stream.ConsumeIncludingWhitespace().NumericValue(),
                        UnitType::kNumber,
                    ));
                    if stream.AtEnd() {
                        break;
                    }
                }
                if weights.len() == 1 {
                    weights.remove(0)
                } else {
                    values::list(weights, values::ListSeparator::Space)
                }
            }
        }
        D::FontStyle => {
            let keyword = stream.Peek().Id();
            if !matches!(
                keyword,
                CSSValueID::kNormal
                    | CSSValueID::kItalic
                    | CSSValueID::kAuto
                    | CSSValueID::kOblique
            ) {
                return Err(DiagnosticKind::Invalid);
            }
            stream.ConsumeIncludingWhitespace();
            if keyword == CSSValueID::kOblique && !stream.AtEnd() {
                return Err(DiagnosticKind::Unsupported);
            }
            values::identifier(keyword)
        }
        D::FontStretch => {
            let parsed = crate::parser::production_property_parser::ParsePropertyWithContext(
                CSSPropertyID::kFontStretch,
                text,
                false,
                context,
            )
            .map_err(|error| match error.kind {
                crate::parser::production_property_parser::PropertyParseErrorKind::Unsupported => {
                    DiagnosticKind::Unsupported
                }
                _ => DiagnosticKind::Invalid,
            })?;
            let value = parsed.first().ok_or(DiagnosticKind::Invalid)?;
            if value.IsImportant() || value.Value().IsCSSWideKeyword() {
                return Err(DiagnosticKind::Invalid);
            }
            stream.SkipUntilPeekedTypeIs(&[]);
            value.ValueRef()
        }
        D::AscentOverride | D::DescentOverride | D::LineGapOverride | D::SizeAdjust => {
            if id != D::SizeAdjust && stream.Peek().Id() == CSSValueID::kNormal {
                stream.ConsumeIncludingWhitespace();
                values::identifier(CSSValueID::kNormal)
            } else {
                if stream.Peek().GetType() == kFunctionToken {
                    return Err(DiagnosticKind::Unsupported);
                }
                if stream.Peek().GetType() != kPercentageToken || stream.Peek().NumericValue() < 0.0
                {
                    return Err(DiagnosticKind::Invalid);
                }
                values::numeric(
                    stream.ConsumeIncludingWhitespace().NumericValue(),
                    UnitType::kPercentage,
                )
            }
        }
        D::FontVariant => {
            let mut variants = Vec::new();
            loop {
                let id = stream.Peek().Id();
                if !matches!(
                    id,
                    CSSValueID::kNormal | CSSValueID::kSmallCaps | CSSValueID::kAll
                ) {
                    return Err(DiagnosticKind::Invalid);
                }
                stream.ConsumeIncludingWhitespace();
                if id == CSSValueID::kAll {
                    if !variants.is_empty() {
                        return Err(DiagnosticKind::Invalid);
                    }
                    return if stream.AtEnd() {
                        Ok(values::identifier(id))
                    } else {
                        Err(DiagnosticKind::Invalid)
                    };
                }
                variants.push(values::identifier(id));
                if stream.Peek().GetType() != kCommaToken {
                    break;
                }
                stream.ConsumeIncludingWhitespace();
            }
            values::list(variants, values::ListSeparator::Comma)
        }
        D::FontFeatureSettings | D::FontVariationSettings => {
            return Err(DiagnosticKind::Unsupported);
        }
        _ => return Err(DiagnosticKind::Invalid),
    };
    if stream.AtEnd() {
        Ok(value)
    } else {
        Err(DiagnosticKind::Invalid)
    }
}
impl crate::style_rule_keyframe::StyleRuleKeyframeBackend for Backend {
    type ExecutionContext = CSSParserContext<Backend>;
    type PropertySet = CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>;
    fn ParseKeyframeKeyList(
        context: &Self::ExecutionContext,
        text: &String,
    ) -> Option<Vec<KeyframeOffset>> {
        CSSParserImpl::<Backend>::ParseKeyframeKeyList(Some(context), text)
    }
    fn TimelineRangeNameToString(name: crate::style_rule_keyframe::TimelineNamedRange) -> String {
        use crate::style_rule_keyframe::TimelineNamedRange as R;
        String::from(match name {
            R::kNone => "none",
            R::kCover => "cover",
            R::kContain => "contain",
            R::kEntry => "entry",
            R::kEntryCrossing => "entry-crossing",
            R::kExit => "exit",
            R::kExitCrossing => "exit-crossing",
            R::kScroll => "scroll",
        })
    }
    fn FormatNumber(value: f64) -> String {
        values::numeric(value, crate::css_primitive_value::UnitType::kNumber).CssText()
    }
    fn PropertiesAsText(properties: &Self::PropertySet) -> String {
        match properties {
            CSSPropertyValueSetRuleHandle::Immutable(set) => SerializeProperties(set),
            CSSPropertyValueSetRuleHandle::Mutable(set) => SerializeProperties(&set.borrow()),
        }
    }
}
pub(crate) fn SerializeProperties(set: &CSSPropertyValueSet<ProductionCSSValueDispatch>) -> String {
    let mut text = std::string::String::new();
    for index in 0..set.PropertyCount() {
        let property = set.PropertyAt(index);
        if index != 0 {
            text.push(' ');
        }
        text.push_str(&property.Name().ToAtomicString().Utf8());
        text.push_str(": ");
        text.push_str(&property.Value().CssText().Utf8());
        if property.IsImportant() {
            text.push_str(" !important");
        }
        text.push(';');
    }
    String::from_utf16(&text.encode_utf16().collect::<Vec<_>>())
}

impl CSSParserImplValueBackend for Backend {
    type CSSVariableData = CSSVariableData;
    fn ParseAsUnresolvedCSSPropertyID(
        token: &crate::parser::css_parser_token::CSSParserToken,
        context: &CSSParserContext<Self::Platform>,
    ) -> CSSPropertyID {
        {
            let name = token.Value().ToString().Utf8();
            if name.starts_with("--") {
                CSSPropertyID::kVariable
            } else {
                crate::css_property_names::FindProperty(name.to_ascii_lowercase().as_bytes())
                    .map_or(CSSPropertyID::kInvalid, |property| {
                        let id = property.id_and_exposed_bit
                            & !crate::css_property_names::kNotKnownExposedPropertyBit;
                        assert!((0..=foundation::kLastUnresolvedCSSProperty as i32).contains(&id));
                        unsafe { std::mem::transmute::<i32, CSSPropertyID>(id) }
                    })
            }
        }
    }
    fn ParseDescriptorValue<T: TokenStreamTokenizer>(
        rule: RuleType,
        id: crate::parser::at_rule_descriptors::AtRuleDescriptorID,
        variable: &AtomicString,
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        properties: &mut Vec<CSSPropertyValue<Self::ValueBackend>>,
    ) {
        let start = stream.LookAheadOffset();
        if matches!(rule, RuleType::kFunction | RuleType::kMixin) {
            use crate::parser::at_rule_descriptors::AtRuleDescriptorID;
            if !matches!(
                id,
                AtRuleDescriptorID::Result | AtRuleDescriptorID::Variable
            ) || id == AtRuleDescriptorID::Result && rule == RuleType::kMixin
            {
                stream.SkipUntilPeekedTypeIs(&[]);
                return;
            }
            let text = ConsumeOriginalVariableValue(stream, false);
            let parsed = crate::parser::production_property_parser::ParseCustomPropertyWithContext(
                "--function-descriptor",
                &text,
                false,
                context,
            );
            if let Ok(parsed) = parsed {
                let name = if id == AtRuleDescriptorID::Variable {
                    crate::css_property_name::CSSPropertyName::custom(variable.clone())
                } else {
                    crate::css_property_name::CSSPropertyName::new(CSSPropertyID::kResult)
                };
                properties.push(CSSPropertyValue::new(
                    &name,
                    parsed.ValueRef(),
                    false,
                    false,
                    -1,
                    false,
                ));
            } else {
                ValueDiagnostic(
                    DiagnosticKind::Invalid,
                    start,
                    "AtRuleDescriptorParser::ParseAtFunctionOrMixinDescriptor",
                    577,
                );
            }
            return;
        }
        if rule == RuleType::kProperty {
            if let Some(value) =
                crate::property_registration::ParseAtPropertyDescriptor(id, stream, context)
            {
                properties.push(CSSPropertyValue::new(
                    &crate::css_property_name::CSSPropertyName::new(
                        crate::parser::at_rule_descriptors::AtRuleDescriptorIDAsCSSPropertyID(id),
                    ),
                    value,
                    false,
                    false,
                    -1,
                    false,
                ));
            } else {
                ValueDiagnostic(
                    DiagnosticKind::Invalid,
                    start,
                    "AtRuleDescriptorParser::ParseAtPropertyDescriptor",
                    459,
                );
            }
            return;
        }
        let text = ConsumeOriginalComponentValue(stream);
        match ParseFontFaceDescriptor(id, &text, start, context) {
            Ok(value) if rule == RuleType::kFontFace => properties.push(CSSPropertyValue::new(
                &crate::css_property_name::CSSPropertyName::new(
                    crate::parser::at_rule_descriptors::AtRuleDescriptorIDAsCSSPropertyID(id),
                ),
                value,
                false,
                false,
                -1,
                false,
            )),
            Ok(_) => {}
            Err(kind) => ValueDiagnostic(
                kind,
                start,
                "AtRuleDescriptorParser::ParseFontFaceDescriptor",
                349,
            ),
        }
    }
    fn CSSPropertyParserParseValue<T: TokenStreamTokenizer>(
        property: CSSPropertyID,
        allow_important: bool,
        stream: &mut CSSParserTokenStream<'_, T>,
        context: Option<&CSSParserContext<Self::Platform>>,
        properties: &mut Vec<CSSPropertyValue<Self::ValueBackend>>,
        rule: RuleType,
    ) {
        let text = ConsumeOriginalComponentValue(stream);
        match crate::parser::production_property_parser::ParsePropertyWithContext(property, &text, false, context.unwrap()) {
            Ok(parsed) if allow_important || parsed.iter().all(|value| !value.IsImportant()) => properties.extend(parsed),
            Ok(_) => {},
            // cpp: css_parser_impl.cc:3366-3378. A generated property whose
            // runtime Exposure predicate is false is an unknown declaration
            // for this document and is discarded. It must not route the
            // otherwise retained rule through the compatibility cascade: that
            // parser evaluates the same runtime predicate and discards it too.
            Err(error)
                if error.kind
                    == crate::parser::production_property_parser::PropertyParseErrorKind::Unsupported
                    && !error.operation.starts_with("CSSProperty::Exposure /") =>
            {
                ValueDiagnostic(
                    DiagnosticKind::Unsupported,
                    stream.LookAheadOffset().saturating_sub(text.length()),
                    error.operation,
                    error.source_line,
                )
            }
            Err(_) => {},
        }
    }
    fn ConsumeCSSWideKeyword<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        allow_important: bool,
        important: &mut bool,
    ) -> Option<Rc<crate::css_value::CSSValue<Self::ValueBackend>>> {
        let saved = stream.Save();
        let text = ConsumeOriginalComponentValue(stream);
        let result = crate::parser::production_property_parser::ParseCustomPropertyWithContext(
            "--wide", &text, false, context,
        )
        .ok();
        if let Some(property) = result {
            let value = property.ValueRef();
            if value.IsCSSWideKeyword() && (allow_important || !property.IsImportant()) {
                *important = property.IsImportant();
                return Some(value);
            }
        }
        stream.EnsureLookAhead();
        stream.Restore(saved);
        None
    }
    fn ConsumeVariableParserUnparsedDeclaration<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        allow_important: bool,
        animation_tainted: bool,
        must_contain_variable_reference: bool,
        restricted_value: bool,
        comma_ends_declaration: bool,
        important: &mut bool,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Rc<Self::CSSVariableData>> {
        let text = ConsumeOriginalComponentValue(stream);
        let property = crate::parser::production_property_parser::ParseCustomPropertyWithContext(
            "--unparsed",
            &text,
            false,
            context,
        )
        .ok()?;
        if property.IsImportant() && !allow_important {
            return None;
        }
        let CSSValuePayload::kUnparsedDeclarationClass(declaration) = property.Value().Payload()
        else {
            return None;
        };
        if must_contain_variable_reference && !declaration.data.NeedsVariableResolution() {
            return None;
        }
        *important = property.IsImportant();
        let mut data = (*declaration.data).clone();
        data.is_animation_tainted = animation_tainted;
        Some(Rc::new(data))
    }
    fn NewCSSUnparsedDeclarationValue(
        data: Rc<Self::CSSVariableData>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<crate::css_value::CSSValue<Self::ValueBackend>> {
        Rc::new(values::Value::new(
            CSSValuePayload::kUnparsedDeclarationClass(values::CSSUnparsedDeclarationValue {
                data,
                mode: context.Mode(),
                base_url: context.BaseURL().clone(),
            }),
        ))
    }
    fn VisitedColumnRuleColorFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature {
        Feature::Property
    }
    fn IdentifierWasQuirky(
        value: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSIdentifierValue,
    ) -> bool {
        false
    }
    fn IdentifierValueID(
        value: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSIdentifierValue,
    ) -> CSSValueID {
        value.0
    }
    fn QuirksModeCursorHandFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature {
        Feature::Property
    }
}

impl crate::css_selector::CSSSelectorParentRule for StyleRule<Backend> {
    fn Selectors(&self) -> &CSSSelectorList {
        self.Selectors()
    }
}
impl crate::parser::css_supports_parser::CSSSupportsBackend for Backend {
    fn SupportsComplexSelector<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool {
        let context = SelectorParserContext {
            html: true,
            quirks: parser.GetMode() == CSSParserMode::kHTMLQuirksMode,
        };
        let options = SelectorParserOptions {
            namespace_context: parser.style_sheet_.as_ref().map(|sheet| {
                sheet.as_ref()
                    as &dyn crate::parser::css_selector_parser::CSSSelectorNamespaceContext
            }),
            parent_rule_for_nesting: None,
            features: Default::default(),
            // CSSSupportsParser::SelectorFn uses the same selector grammar as
            // the containing sheet. Built-in sheets may name UA shadow pseudo
            // elements and may continue a selector past such an element.
            ua_sheet_mode: parser.GetMode() == CSSParserMode::kUASheetMode,
            runtime_context: None,
            usage_context: None,
            ident_integer_parser: None,
        };
        CSSSelectorParser::SupportsComplexSelectorStream(stream, &context, &options)
    }
    fn CSSSupportsAtRuleFunctionEnabled() -> bool {
        false
    }
    fn CSSSupportsNamedFeatureFunctionEnabled() -> bool {
        false
    }
    fn IsSupportedNamedFeature(id: CSSValueID) -> bool {
        id == CSSValueID::kAnchorPositionFollowsTransforms
    }
    fn IsBlinkFeatureEnabled(name: &String) -> bool {
        // Chromium's built-in html.css is parsed in UA sheet mode and uses
        // blink-feature() to select between the enabled and disabled forms of
        // a rule.  Keep that decision in the parser's runtime-feature backend;
        // projecting the sheet first and treating every feature as disabled
        // silently activates obsolete fallback rules in the production
        // cascade.  This is the complete feature set referenced by html.css at
        // the pinned Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
        match name.Utf8().as_str() {
            // Runtime registry entries with stable status at the pinned
            // revision.  These are enabled in a normal renderer process.
            "AutofillPreviewGenericFontFamily"
            | "AutofillPreviewIgnoreAuthorFont"
            | "HTMLAreaElementDisplayNone"
            | "HTMLInputElementDropWebkitClearButton"
            | "OverlayGlobalRuleRemoval"
            | "OverlayProperty"
            | "RemovePositionAnchorUAStyle"
            | "TableDefaultBorderColorCurrentColor" => true,

            // Keep shared Layout feature ownership where it already exists.
            "AppearanceBase" => foundation::RuntimeEnabledFeatures::AppearanceBaseEnabled(),
            "UnboundedElement" => foundation::RuntimeEnabledFeatures::UnboundedElementEnabled(),

            // Experimental features are disabled unless an embedding supplies
            // a runtime override.  Rechrom's current RuntimeEnabledFeatures
            // owner has no override surface for these entries.
            "CustomizableCombobox"
            | "CustomizableSelectMultiplePopup"
            | "FilterableSelect"
            | "MenuElements" => false,
            _ => {
                ValueDiagnostic(
                    DiagnosticKind::Unsupported,
                    0,
                    "RuntimeEnabledFeatures::IsFeatureEnabledFromString",
                    309,
                );
                false
            }
        }
    }
}
impl CSSParserImplPageBackend for Backend {
    type SelectorParserContext = SelectorParserContext;
    fn SelectorParserContext(
        document: Option<DocumentHandle<Self::Platform>>,
    ) -> Self::SelectorParserContext {
        SelectorParserContext {
            html: true,
            quirks: false,
        }
    }
}

impl CSSParserImplKeyframeBackend for Backend {
    fn ConsumeTimelineRangeNameAndPercent<T: TokenStreamTokenizer>(
        context: Option<&CSSParserContext<Self::Platform>>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSValueList>> {
        let id = stream.Peek().Id();
        if id == CSSValueID::kScroll {
            ValueDiagnostic(
                DiagnosticKind::Unsupported,
                stream.LookAheadOffset(),
                "ScrollTimelineNamedRangeScroll feature",
                4794,
            );
            return None;
        }
        if !matches!(
            id,
            CSSValueID::kCover
                | CSSValueID::kContain
                | CSSValueID::kEntry
                | CSSValueID::kEntryCrossing
                | CSSValueID::kExit
                | CSSValueID::kExitCrossing
        ) {
            return None;
        }
        stream.ConsumeIncludingWhitespace();
        if stream.Peek().GetType() == kFunctionToken {
            ValueDiagnostic(
                DiagnosticKind::Unsupported,
                stream.LookAheadOffset(),
                "css_parsing_utils::ConsumeTimelineRangeNameAndPercent math",
                4808,
            );
            return None;
        }
        if stream.Peek().GetType() != kPercentageToken {
            return None;
        }
        let percent = stream.ConsumeIncludingWhitespace().NumericValue();
        Some(Rc::new(values::CSSValueList::new(
            vec![
                values::identifier(id),
                values::numeric(percent, crate::css_primitive_value::UnitType::kPercentage),
            ],
            values::ListSeparator::Space,
        )))
    }
    fn TimelineRangeListName(
        list: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSValueList,
    ) -> crate::style_rule_keyframe::TimelineNamedRange {
        use crate::style_rule_keyframe::TimelineNamedRange as R;
        match list.values.first().map(|value| value.Payload()) {
            Some(CSSValuePayload::kIdentifierClass(value)) => match value.0 {
                CSSValueID::kCover => R::kCover,
                CSSValueID::kContain => R::kContain,
                CSSValueID::kEntry => R::kEntry,
                CSSValueID::kEntryCrossing => R::kEntryCrossing,
                CSSValueID::kExit => R::kExit,
                CSSValueID::kExitCrossing => R::kExitCrossing,
                _ => R::kNone,
            },
            _ => R::kNone,
        }
    }
    fn TimelineRangeListClampedPercent(
        list: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSValueList,
    ) -> f64 {
        match list.values.get(1).map(|value| value.Payload()) {
            Some(CSSValuePayload::kNumericLiteralClass(value)) => value.DoubleValue(),
            _ => 0.0,
        }
    }
}

impl CSSParserImplSheetBackend for Backend {
    type ParseStyleSheetTimer = Unavailable;
    type DocumentView = Unavailable;
    type UkmAggregator = Unavailable;
    type TextPosition = Unavailable;
    fn DocumentView(document: &DocumentHandle<Self::Platform>) -> Option<Rc<Self::DocumentView>> {
        unavailable("CSSParserImplSheetBackend::DocumentView")
    }
    fn ViewUkmAggregator(view: &Self::DocumentView) -> Option<Rc<Self::UkmAggregator>> {
        unavailable("CSSParserImplSheetBackend::ViewUkmAggregator")
    }
    fn GetScopedParseStyleSheetTimer(
        aggregator: &Self::UkmAggregator,
    ) -> Self::ParseStyleSheetTimer {
        unavailable("CSSParserImplSheetBackend::GetScopedParseStyleSheetTimer")
    }
    fn TraceBeginStyleSheet(
        base_url: &<Self::Platform as CSSParserContextPlatform>::URL,
        mode: CSSParserMode,
    ) {
        unavailable("CSSParserImplSheetBackend::TraceBeginStyleSheet")
    }
    fn TraceBeginStyleSheetParse() {
        unavailable("CSSParserImplSheetBackend::TraceBeginStyleSheetParse")
    }
    fn TraceEndStyleSheetParse() {
        unavailable("CSSParserImplSheetBackend::TraceEndStyleSheetParse")
    }
    fn TraceEndStyleSheet(token_count: u32, length: u32) {
        unavailable("CSSParserImplSheetBackend::TraceEndStyleSheet")
    }
    fn NewLazyParsingState(
        context: &CSSParserContext<Self::Platform>,
        text: &String,
        sheet: &mut Self::StyleSheetContents,
    ) -> Self::CSSLazyParsingState {
        unavailable("CSSParserImplSheetBackend::NewLazyParsingState")
    }
    fn AnyOwnerDocument(
        sheet: &Self::StyleSheetContents,
    ) -> Option<DocumentHandle<Self::Platform>> {
        unavailable("CSSParserImplSheetBackend::AnyOwnerDocument")
    }
    fn MinimumTextPosition() -> Self::TextPosition {
        unavailable("CSSParserImplSheetBackend::MinimumTextPosition")
    }
    fn GetTextPosition(
        document: &DocumentHandle<Self::Platform>,
        offset: u32,
        text: &String,
        position: &mut Self::TextPosition,
    ) {
        unavailable("CSSParserImplSheetBackend::GetTextPosition")
    }
    fn SetImportPositionHint(rule: &Self::StyleRuleBase, position: Self::TextPosition) {
        unavailable("CSSParserImplSheetBackend::SetImportPositionHint")
    }
    fn ParserAppendRule(sheet: &mut Self::StyleSheetContents, rule: Rc<Self::StyleRuleBase>) {
        sheet.ParserAppendRule(rule)
    }
    fn SetHasSyntacticallyValidCSSHeader(sheet: &mut Self::StyleSheetContents, valid: bool) {
        sheet.SetHasSyntacticallyValidCSSHeader(valid)
    }
}

impl CSSParserImplRuleBackend for Backend {
    type RuleDependencies = Backend;
    fn CountAtRule(context: &CSSParserContext<Self::Platform>, id: CSSAtRuleID) {
        context.CountWebFeature(Feature::AtRule(id))
    }
    fn CascadeLayersFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature {
        Feature::Rule
    }
    fn QuotedKeyframesFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature {
        Feature::Rule
    }
    fn NewUrlRequestModifiers() -> Self::Modifiers {
        crate::style_rule_import::CSSUrlRequestModifiers::default()
    }
    fn CSSSupportsForImportRulesEnabled() -> bool {
        true
    }
    fn CSSScopeImportEnabled() -> bool {
        false
    }
    fn CSSRevertRuleEnabled() -> bool {
        false
    }
    fn ConsumeSupportsCondition<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> SupportsResult {
        crate::parser::css_supports_parser::CSSSupportsParser::ConsumeSupportsCondition(
            stream, parser,
        )
    }
    fn ConsumeStyleScope<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope>>
    {
        crate::style_scope::StyleScope::Consume(parser, stream, CSSNestingType::kNone, None)
    }
    fn ParseMediaQuerySetString(
        text: String,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<Self::MediaQuerySet> {
        Rc::new(
            MediaQueryParser::new(MediaQueryFeatureFlags::default())
                .ParseMediaQuerySet(StringView::from(&text)),
        )
    }
    fn ParseMediaQuerySet<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<Self::MediaQuerySet> {
        Rc::new(
            MediaQueryParser::new(MediaQueryFeatureFlags::default())
                .ParseMediaQuerySetFromStream(stream),
        )
    }
    fn CachedMediaQuery(edge: Member<Self::MediaQuerySet>) -> Option<Rc<Self::MediaQuerySet>> {
        MEDIA_EDGES.with(|values| {
            values
                .borrow()
                .iter()
                .find(|query| Rc::as_ptr(query) == edge.Get())
                .cloned()
        })
    }
    fn MediaQueryCacheEdge(query: &Rc<Self::MediaQuerySet>) -> Member<Self::MediaQuerySet> {
        MEDIA_EDGES.with(|values| values.borrow_mut().push(query.clone()));
        Member::from_ptr(Rc::as_ptr(query) as *mut QuerySet)
    }
    fn NewImportRule(
        uri: AtomicString,
        layer: Vec<AtomicString>,
        scope: Option<
            Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope>,
        >,
        supported: bool,
        supports: String,
        media: Rc<Self::MediaQuerySet>,
        origin_clean: bool,
        modifiers: &Self::Modifiers,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleImport>
    {
        Rc::new(ConcreteImport {
            metadata: ImportMetadata::new(
                String::FromUtf8(uri.Utf8().as_bytes()),
                layer,
                scope,
                supported,
                supports,
                media,
                if origin_clean {
                    crate::css_origin_clean::OriginClean::kTrue
                } else {
                    crate::css_origin_clean::OriginClean::kFalse
                },
                modifiers,
            ),
            parent: RefCell::new(None),
            request: std::cell::Cell::new(false),
        })
    }
    fn NewNamespaceRule(
        prefix: AtomicString,
        uri: AtomicString,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleNamespace>
    {
        Rc::new(crate::style_rule_namespace::StyleRuleNamespace::new(
            prefix, uri,
        ))
    }
    fn NewKeyframeRule(
        keys: Vec<KeyframeOffset>,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleKeyframe>
    {
        Rc::new(ConcreteKeyframe::new(
            keys,
            CSSPropertyValueSetRuleHandle::FromImmutable(properties),
        ))
    }
    fn NewKeyframesRule(
        name: String,
        vendor_prefixed: bool,
        keys: Vec<Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleKeyframe>>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleKeyframes>
    {
        Rc::new(ConcreteKeyframes {
            name: AtomicString::from_utf16(StringView::from(&name).Span16()),
            vendor_prefixed,
            keys,
        })
    }
fn NewNestedDeclarationsRule(nesting: CSSNestingType, inner: crate::style_rule::StyleRule<Self::RuleDependencies>)
    -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleNestedDeclarations>{
        Rc::new(
            crate::style_rule_nested_declarations::StyleRuleNestedDeclarations::new(nesting, inner),
        )
    }
fn NewFunctionDeclarationsRule(properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>)
    -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleFunctionDeclarations>{
        Rc::new(
            crate::style_rule_function_declarations::StyleRuleFunctionDeclarations::new(
                CSSPropertyValueSetRuleHandle::FromImmutable(properties),
            ),
        )
    }
    fn ConsumeMixinRuleListOrNestedDeclarationList<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Rc<Self::StyleRule>,
        rules: &mut Vec<Rc<Self::StyleRuleBase>>,
    ) {
        // css_parser_impl.cc:2696-2731. The fake parent allocation must remain
        // alive because copied nested selectors retain its identity.
        parser.ConsumeRuleListOrNestedDeclarationList(stream, nesting, Some(parent), rules)
    }
    fn ConsumeFontFamily<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Vec<Rc<crate::css_value::CSSValue<Self::ValueBackend>>>> {
        crate::parser::production_property_parser::ConsumeFontFamily(stream).ok()
    }
    fn FontFamilyValue(
        value: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSFontFamilyValue,
    ) -> AtomicString {
        AtomicString::from_utf16(StringView::from(&value.0).Span16())
    }
    fn ConsumeNonNegativeIntegerOrNumberCalc<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Rc<crate::css_value::CSSValue<Self::ValueBackend>>> {
        let start = stream.LookAheadOffset();
        match crate::parser::production_property_parser::ConsumeNonNegativeIntegerOrNumberCalc(
            stream,
        ) {
            Ok(value) => Some(value),
            Err(error) => {
                if error.kind == crate::parser::production_property_parser::PropertyParseErrorKind::Unsupported {
                    ValueDiagnostic(DiagnosticKind::Unsupported, start, error.operation, error.source_line);
                }
                None
            }
        }
    }
    fn PrimitiveNumberValueIfKnown(
        value: &crate::css_value::CSSValue<Self::ValueBackend>,
    ) -> Option<f64> {
        match value.Payload() {
            CSSValuePayload::kNumericLiteralClass(number) => number.GetValueIfKnown(),
            CSSValuePayload::kMathFunctionClass(math) => math
                .expression
                .GetValueIfKnown()
                .map(|number| math.ClampToPermittedRange(number)),
            _ => None,
        }
    }
    fn StartsCustomPropertyDeclaration<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool {
        let token = stream.Peek().clone();
        // cpp: css_variable_parser.cc:34-52. Only a valid custom-property
        // identifier may be consumed speculatively. In particular, a style
        // rule can begin with a block-start token while recovering from a
        // nested rule; consuming that token through Consume() violates the
        // token stream's block guard contract.
        if token.GetType() != kIdentToken {
            return false;
        }
        let name = token.Value().ToString().Utf8();
        if name.len() < 3 || !name.starts_with("--") {
            return false;
        }
        let save = stream.Save();
        stream.ConsumeIncludingWhitespace();
        let result = stream.Peek().GetType() == kColonToken;
        stream.Restore(save);
        result
    }
    fn ConsumeSelector<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: Option<&CSSParserContext<Self::Platform>>,
        nesting: CSSNestingType,
        parent: Option<Rc<Self::StyleRule>>,
        semicolon_aborts_nested_selector: bool,
        sheet: Option<&mut Self::StyleSheetContents>,
        observer: Option<&mut Self::CSSParserObserver>,
        arena: &mut Vec<CSSSelector>,
        has_visited_pseudo: &mut bool,
    ) -> std::ops::Range<usize> {
        let start = arena.len();
        let ua_sheet_mode =
            context.is_some_and(|context| context.Mode() == CSSParserMode::kUASheetMode);
        let context = SelectorParserContext {
            html: true,
            quirks: context.is_some_and(|c| c.Mode() == CSSParserMode::kHTMLQuirksMode),
        };
        let options = SelectorParserOptions {
            namespace_context: sheet.as_ref().map(|sheet| {
                sheet.as_ref()
                    as &dyn crate::parser::css_selector_parser::CSSSelectorNamespaceContext
            }),
            parent_rule_for_nesting: parent
                .map(|parent| parent as Rc<dyn crate::css_selector::CSSSelectorParentRule>),
            features: Default::default(),
            // cpp: css_parser_impl.cc ConsumeStyleRule passes the parser
            // context through to CSSSelectorParser. UA mode is semantic: it
            // admits Blink's internal/custom pseudo-element selector grammar
            // used throughout html.css. Keeping this false parsed the exact UA
            // resource with author-sheet restrictions and silently dropped
            // otherwise valid built-in rules.
            ua_sheet_mode,
            runtime_context: None,
            usage_context: None,
            ident_integer_parser: None,
        };
        CSSSelectorParser::ConsumeSelector(
            stream,
            &context,
            nesting,
            &options,
            semicolon_aborts_nested_selector,
            observer.map(|observer| {
                observer as &mut dyn crate::parser::css_selector_parser::CSSSelectorParserObserver
            }),
            arena,
            Some(has_visited_pseudo),
        );
        start..arena.len()
    }
    fn HasAVX2AndPCLMUL() -> bool {
        unavailable("CSSParserImplRuleBackend::HasAVX2AndPCLMUL")
    }
    fn FindLengthOfDeclarationList(text: &StringView) -> usize {
        unavailable("CSSParserImplRuleBackend::FindLengthOfDeclarationList")
    }
    fn FindLengthOfDeclarationListAVX2(text: &StringView) -> usize {
        unavailable("CSSParserImplRuleBackend::FindLengthOfDeclarationListAVX2")
    }
    fn LazyParsingStateHandle(
        state: &Self::CSSLazyParsingState,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::CSSLazyParsingState>
    {
        unavailable("CSSParserImplRuleBackend::LazyParsingStateHandle")
    }
    fn CSSNestingFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature {
        Feature::Rule
    }
    fn PropertyRegistrationConvertSyntax(
        value: Option<&crate::css_value::CSSValue<Self::ValueBackend>>,
    ) -> Option<ParserSyntax<Self>> {
        crate::property_registration::ConvertSyntax(value)
    }
    fn PropertyRegistrationConvertInherits(
        value: Option<&crate::css_value::CSSValue<Self::ValueBackend>>,
    ) -> Option<bool> {
        crate::property_registration::ConvertInherits(value)
    }
    fn PropertyRegistrationConvertInitial(
        value: Option<&crate::css_value::CSSValue<Self::ValueBackend>>,
        syntax: &ParserSyntax<Self>,
        context: &CSSParserContext<Self::Platform>,
        name: &String,
    ) -> Option<Option<Rc<crate::css_value::CSSValue<Self::ValueBackend>>>> {
        match crate::property_registration::ConvertInitial(value, syntax) {
            Ok(value) => Some(value),
            Err(crate::css_syntax_definition::SyntaxValueError::Unsupported(_)) => {
                ValueDiagnostic(
                    DiagnosticKind::Unsupported,
                    0,
                    "CSSSyntaxDefinition::ConsumeSingleType",
                    187,
                );
                None
            }
            Err(_) => None,
        }
    }
    fn ConsumeCounterStyleNameInPrelude<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> AtomicString {
        unavailable("CSSParserImplRuleBackend::ConsumeCounterStyleNameInPrelude")
    }
    fn NewCounterStyleRule(
        name: AtomicString,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<
        <Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleCounterStyle,
    > {
        unavailable("CSSParserImplRuleBackend::NewCounterStyleRule")
    }
fn NewFontPaletteValuesRule(name: AtomicString, properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>)
    -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleFontPaletteValues>{
        unavailable("CSSParserImplRuleBackend::NewFontPaletteValuesRule")
    }
    fn NewPositionTryRule(
        name: AtomicString,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<
        <Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRulePositionTry,
    > {
        unavailable("CSSParserImplRuleBackend::NewPositionTryRule")
    }
    fn NewLocationRule(
        name: AtomicString,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleLocation>
    {
        unavailable("CSSParserImplRuleBackend::NewLocationRule")
    }
fn NewViewTransitionRule(properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>)
    -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleViewTransition>{
        unavailable("CSSParserImplRuleBackend::NewViewTransitionRule")
    }
    fn ParseNavigationQuery<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<
        Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::NavigationQuery>,
    > {
        unavailable("CSSParserImplRuleBackend::ParseNavigationQuery")
    }
    fn ParseContainerQuerySet<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<
        Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::ContainerQuerySet>,
    > {
        crate::parser::container_query_parser::ContainerQueryParser::new(
            crate::production_container_parser::ProductionContainerBackend::new(context),
        )
        .ParseContainerQuerySet(stream)
    }
    fn ConsumeStyleScopeForRule<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<Self::StyleRule>>,
    ) -> Option<Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope>>
    {
        crate::style_scope::StyleScope::Consume(parser, stream, nesting, parent)
    }
    fn CreateImplicitStyleScope(
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope> {
        crate::style_scope::StyleScope::CreateImplicit()
    }
    fn ScopeRuleForNesting(
        scope: &mut Rc<
            <Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope,
        >,
    ) -> Option<Rc<Self::StyleRule>> {
        scope.RuleForNesting()
    }
    fn ConsumeSyntaxDefinition<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<ParserSyntax<Self>> {
        crate::css_syntax_definition::CSSSyntaxDefinition::Consume(stream)
    }
    fn ConsumeSyntaxComponent<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<ParserSyntax<Self>> {
        crate::css_syntax_definition::CSSSyntaxDefinition::ConsumeComponent(stream)
    }
    fn CreateUniversalSyntax() -> ParserSyntax<Self> {
        crate::css_syntax_definition::CSSSyntaxDefinition::CreateUniversal()
    }
    fn SyntaxIsUniversal(syntax: &ParserSyntax<Self>) -> bool {
        syntax.IsUniversal()
    }
    fn VariableDataNeedsVariableResolution(data: &ParserVariableData<Self>) -> bool {
        data.NeedsVariableResolution()
    }
    fn ParseSyntaxDefaultValue(
        syntax: &ParserSyntax<Self>,
        data: &ParserVariableData<Self>,
        context: &CSSParserContext<Self::Platform>,
    ) -> bool {
        syntax.ParseTokens(data).is_ok()
    }
    fn ConsumeUnparsedDeclaration<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        comma_ends_declaration: bool,
    ) -> Option<Rc<ParserVariableData<Self>>> {
        let text = ConsumeOriginalVariableValue(stream, comma_ends_declaration);
        let property = crate::parser::production_property_parser::ParseCustomPropertyWithContext(
            "--function-value",
            &text,
            false,
            context,
        )
        .ok()?;
        let CSSValuePayload::kUnparsedDeclarationClass(value) = property.Value().Payload() else {
            return None;
        };
        Some(value.data.clone())
    }
    fn NewPrivateVariable(
        name: AtomicString,
        syntax: ParserSyntax<Self>,
        default_value: Option<Rc<ParserVariableData<Self>>>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::CSSPrivateVariable>
    {
        unavailable("CSSParserImplRuleBackend::NewPrivateVariable")
    }
    fn ConsumeMixinArguments<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        arguments: &mut Vec<Option<Rc<ParserVariableData<Self>>>>,
    ) -> bool {
        // css_variable_parser.cc:421-489. Keep this in lockstep with custom
        // function argument grammar: top-level commas split arguments, while
        // an outer {} block admits commas as part of one argument.
        let mut guard = BlockGuard::new(stream);
        guard.ConsumeWhitespace();
        while !guard.AtEnd() {
            let value = if guard.Peek().GetType() == kLeftBraceToken {
                let mut brace = BlockGuard::new(&mut guard);
                brace.ConsumeWhitespace();
                if brace.AtEnd() {
                    return false;
                }
                let value = Self::ConsumeUnparsedDeclaration(&mut brace, context, false);
                if value.is_none() || !brace.AtEnd() {
                    return false;
                }
                value
            } else {
                if Self::StartsCustomPropertyDeclaration(&mut guard) {
                    return false;
                }
                Self::ConsumeUnparsedDeclaration(&mut guard, context, true)
            };
            let Some(value) = value else {
                return false;
            };
            arguments.push(Some(value));
            if guard.Peek().GetType() == kCommaToken {
                guard.ConsumeIncludingWhitespace();
                if guard.AtEnd() || guard.Peek().GetType() == kCommaToken {
                    return false;
                }
            } else if !guard.AtEnd() {
                return false;
            }
        }
        true
    }
    fn ParseCustomMediaDefinition<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Rc<Self::MediaQuerySet>> {
        unavailable("CSSParserImplRuleBackend::ParseCustomMediaDefinition")
    }
}

impl CSSParserContextPlatform for Backend {
    type URL = SourceURL;
    type TextEncoding = Option<String>;
    type Referrer = String;
    type ReferrerPolicy = ();
    type DOMWrapperWorld = Unavailable;
    type ExecutionContext = Unavailable;
    type WebFeature = Feature;
    type WebDXFeature = Feature;
    fn NullURL() -> Self::URL {
        SourceURL::Null
    }
    fn EmptyURL() -> Self::URL {
        SourceURL::Value(String::from(""))
    }
    fn EmptyTextEncoding() -> Self::TextEncoding {
        None
    }
    fn IsEncodingValid(encoding: &Self::TextEncoding) -> bool {
        encoding.is_some()
    }
    fn EmptyReferrer() -> Self::Referrer {
        String::from("")
    }
    fn MakeReferrer(referrer: String, policy: Self::ReferrerPolicy) -> Self::Referrer {
        referrer
    }
    fn StrippedForUseAsReferrer(url: &Self::URL) -> String {
        url.AsString().cloned().unwrap_or_default()
    }
    fn ResolveURL(
        base: &Self::URL,
        input: &String,
        _encoding: Option<&Self::TextEncoding>,
    ) -> Self::URL {
        if input.empty() {
            return SourceURL::Value(String::from(""));
        }
        let input_utf8 = input.Utf8();
        let resolved = base
            .AsString()
            .and_then(|base| url::Url::parse(&base.Utf8()).ok())
            .and_then(|base| base.join(&input_utf8).ok())
            .or_else(|| url::Url::parse(&input_utf8).ok())
            .map(|url| url.to_string())
            .unwrap_or(input_utf8);
        SourceURL::Value(String::FromUtf8(resolved.as_bytes()))
    }
    fn CSSParserIgnoreCharsetForURLsEnabled() -> bool {
        true
    }
    fn CountDeprecation(context: Option<&Rc<Self::ExecutionContext>>, feature: Self::WebFeature) {
        unavailable("CSSParserContextPlatform::CountDeprecation")
    }
}

impl StyleRuleDependencies for Backend {
    type SelectorList = CSSSelectorList;
    type CSSPropertyValueSet = CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>;
    type CSSLazyParsingState = Unavailable;
    type MixinParameterBindings = ProductionMixinParameterBindings;
    type StyleScope = crate::style_scope::StyleScope;
    type MediaQuerySet = QuerySet;
    type ContainerQuerySet = ContainerSet;
    type ContainerQuery =
        crate::container_query::ContainerQuery<values::Value, values::CSSUnparsedDeclarationValue>;
    type ConditionalExpNode = ContainerCondition;
    type NavigationQuery = Unavailable;
    type CSSSyntaxDefinition = crate::css_syntax_definition::CSSSyntaxDefinition;
    type CSSVariableData = CSSVariableData;
    type CSSPrivateVariable = Unavailable;
    type ExecutionContext = Unavailable;
    type StyleRuleImport = ConcreteImport;
    type StyleRuleFontPaletteValues = Unavailable;
    type StyleRuleFontFeatureValues =
        crate::style_rule_font_feature_values::StyleRuleFontFeatureValues;
    type StyleRuleFontFeature = crate::style_rule_font_feature_values::StyleRuleFontFeature;
    type StyleRuleKeyframes = ConcreteKeyframes;
    type StyleRuleKeyframe = ConcreteKeyframe;
    type StyleRuleNestedDeclarations =
        crate::style_rule_nested_declarations::StyleRuleNestedDeclarations<Backend>;
    type StyleRuleFunctionDeclarations =
        crate::style_rule_function_declarations::StyleRuleFunctionDeclarations<
            ProductionCSSValueDispatch,
        >;
    type StyleRuleNamespace = crate::style_rule_namespace::StyleRuleNamespace;
    type StyleRuleCounterStyle = Unavailable;
    type StyleRuleViewTransition = Unavailable;
    type StyleRulePositionTry = Unavailable;
    type StyleRuleLocation = Unavailable;
    fn ParseDeclarationListForLazyStyle(
        state: &Self::CSSLazyParsingState,
        offset: usize,
    ) -> Rc<Self::CSSPropertyValueSet> {
        unavailable("StyleRuleDependencies::ParseDeclarationListForLazyStyle")
    }
    fn ParseCustomPropertyName(text: &String) -> String {
        crate::parser::css_parser_impl::ParseCustomPropertyName(StringView::from(text))
    }
    fn ConsumeSupportsCondition(context: &Self::ExecutionContext, text: &String) -> bool {
        unavailable("StyleRuleDependencies::ConsumeSupportsCondition")
    }
    fn ContainerQuerySetToString(set: &Self::ContainerQuerySet) -> String {
        set.ToString()
    }
    fn ParseContainerQuerySet(
        context: &Self::ExecutionContext,
        text: &String,
    ) -> Option<Rc<Self::ContainerQuerySet>> {
        match *context {}
    }
    fn SingleContainerQuery(set: &Self::ContainerQuerySet) -> Option<&Self::ContainerQuery> {
        set.SingleQuery()
    }
    fn ParseContainerCondition(
        context: &Self::ExecutionContext,
        text: &String,
    ) -> Option<Rc<Self::ConditionalExpNode>> {
        match *context {}
    }
    fn SerializeContainerCondition(node: &Self::ConditionalExpNode) -> String {
        node.Serialize()
    }
    fn ContainerQuerySelectorName(query: &Self::ContainerQuery) -> AtomicString {
        query.SelectorName()
    }
    fn NewContainerQuery(
        name: AtomicString,
        node: Rc<Self::ConditionalExpNode>,
    ) -> Rc<Self::ContainerQuery> {
        Rc::new(crate::container_query::ContainerQuery::new(
            name,
            Some(node),
        ))
    }
    fn NewContainerQuerySet(queries: Vec<Rc<Self::ContainerQuery>>) -> Rc<Self::ContainerQuerySet> {
        Rc::new(ContainerSet::new(queries))
    }
    fn ParseNavigationQuery(text: &String) -> Option<Rc<Self::NavigationQuery>> {
        unavailable("StyleRuleDependencies::ParseNavigationQuery")
    }
    fn CopyPrivateVariable(variable: &Self::CSSPrivateVariable) -> Rc<Self::CSSPrivateVariable> {
        unavailable("StyleRuleDependencies::CopyPrivateVariable")
    }
}

impl StyleSheetContentsBackend for Backend {
    type Platform = Backend;
    type ParserBackend = Backend;
    type CSSStyleSheet = Unavailable;
    type Node = Unavailable;
    type TreeScope = Unavailable;
    type Resource = Unavailable;
    type RuleSetDiff = ProductionRuleSetDiff;
    fn ImportMediaQueries(rule: &Self::StyleRuleImport) -> Option<Rc<Self::MediaQuerySet>> {
        Some(rule.metadata.MediaQueries())
    }
    fn ImportStyleSheet(rule: &Self::StyleRuleImport) -> Option<Rc<StyleSheetContents<Self>>> {
        None
    }
    fn ImportParentStyleSheet(
        rule: &Self::StyleRuleImport,
    ) -> Option<Rc<StyleSheetContents<Self>>> {
        rule.parent
            .borrow()
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
    }
    fn SetImportParentStyleSheet(
        rule: &Self::StyleRuleImport,
        parent: &Rc<StyleSheetContents<Self>>,
    ) {
        *rule.parent.borrow_mut() = Some(Rc::downgrade(parent));
    }
    fn ClearImportParentStyleSheet(rule: &Self::StyleRuleImport) {
        rule.parent.borrow_mut().take();
    }
    fn RequestImportStyleSheet(rule: &Self::StyleRuleImport) {
        rule.request.set(true);
    }
    fn ImportIsLoading(rule: &Self::StyleRuleImport) -> bool {
        false
    }
    fn ImportIsSupported(rule: &Self::StyleRuleImport) -> bool {
        rule.metadata.IsSupported()
    }
    fn NamespacePrefix(rule: &Self::StyleRuleNamespace) -> AtomicString {
        rule.Prefix()
    }
    fn NamespaceURI(rule: &Self::StyleRuleNamespace) -> AtomicString {
        rule.Uri()
    }
    fn MediaQueriesForRule(rule: &StyleRuleMedia<Self>) -> Option<Rc<Self::MediaQuerySet>> {
        rule.MediaQueries().map(|query| Rc::new(query.clone()))
    }
    fn CounterStyleHasFailedOrCanceledSubresources(rule: &Self::StyleRuleCounterStyle) -> bool {
        unavailable("StyleSheetContentsBackend::CounterStyleHasFailedOrCanceledSubresources")
    }
    fn OwnerDocument(client: &Self::CSSStyleSheet) -> Option<Rc<Self::Document>> {
        unavailable("StyleSheetContentsBackend::OwnerDocument")
    }
    fn OwnerNode(client: &Self::CSSStyleSheet) -> Option<Rc<Self::Node>> {
        unavailable("StyleSheetContentsBackend::OwnerNode")
    }
    fn IsConstructed(client: &Self::CSSStyleSheet) -> bool {
        unavailable("StyleSheetContentsBackend::IsConstructed")
    }
    fn ClientLoadCompleted(client: &Self::CSSStyleSheet) -> bool {
        unavailable("StyleSheetContentsBackend::ClientLoadCompleted")
    }
    fn SheetLoaded(client: &Self::CSSStyleSheet) -> bool {
        unavailable("StyleSheetContentsBackend::SheetLoaded")
    }
    fn SetClientToPendingState(client: &Self::CSSStyleSheet) {
        unavailable("StyleSheetContentsBackend::SetClientToPendingState")
    }
    fn IsAdoptedByTreeScope(client: &Self::CSSStyleSheet, scope: &Self::TreeScope) -> bool {
        unavailable("StyleSheetContentsBackend::IsAdoptedByTreeScope")
    }
    fn NodeTreeScope(node: &Self::Node) -> &Self::TreeScope {
        unavailable("StyleSheetContentsBackend::NodeTreeScope")
    }
    fn NodeIsConnected(node: &Self::Node) -> bool {
        unavailable("StyleSheetContentsBackend::NodeIsConnected")
    }
    fn NodeDocument(node: &Self::Node) -> Rc<Self::Document> {
        unavailable("StyleSheetContentsBackend::NodeDocument")
    }
    fn NotifyLoadedSheetAndAllCriticalSubresources(node: &Self::Node, error_occurred: bool) {
        unavailable("StyleSheetContentsBackend::NotifyLoadedSheetAndAllCriticalSubresources")
    }
    fn SetNeedsActiveStyleUpdate(document: &Self::Document, scope: &Self::TreeScope) {
        unavailable("StyleSheetContentsBackend::SetNeedsActiveStyleUpdate")
    }
    fn RemoveFontFaceRule(document: &Self::Document, rule: &StyleRuleFontFace<Self>) {
        unavailable("StyleSheetContentsBackend::RemoveFontFaceRule")
    }
    fn ParserDocument(document: &Rc<Self::Document>) -> DocumentHandle<Self::Platform> {
        unavailable("StyleSheetContentsBackend::ParserDocument")
    }
    fn TraceParseAuthorStyleSheet(resource: &Self::Resource) {
        unavailable("StyleSheetContentsBackend::TraceParseAuthorStyleSheet")
    }
    fn ResourceErrorOccurred(resource: &Self::Resource) -> bool {
        unavailable("StyleSheetContentsBackend::ResourceErrorOccurred")
    }
    fn ResourceIsCorsSameOrigin(resource: &Self::Resource) -> bool {
        unavailable("StyleSheetContentsBackend::ResourceIsCorsSameOrigin")
    }
    fn ResourceSheetText(
        resource: &Self::Resource,
        context: &CSSParserContext<Self::Platform>,
        check: MIMETypeCheck,
    ) -> String {
        unavailable("StyleSheetContentsBackend::ResourceSheetText")
    }
    fn ResourceSourceMapHeader(resource: &Self::Resource) -> String {
        unavailable("StyleSheetContentsBackend::ResourceSourceMapHeader")
    }
    fn ResourceDeprecatedSourceMapHeader(resource: &Self::Resource) -> String {
        unavailable("StyleSheetContentsBackend::ResourceDeprecatedSourceMapHeader")
    }
    fn MediaEvaluatorDocument(medium: &Self::MediaQueryEvaluator) -> Option<&Self::Document> {
        None
    }
    fn AddRulesFromSheet(
        rule_set: &mut RuleSet<Self>,
        sheet: &StyleSheetContents<Self>,
        medium: &Self::MediaQueryEvaluator,
        mixins: &MixinMap<Self>,
    ) {
        rule_set.AddRulesFromSheet(sheet, medium, mixins, None, None)
    }
    fn CompactRulesIfNeeded(rule_set: &mut RuleSet<Self>) {
        rule_set.CompactRulesIfNeeded()
    }
    fn NewRuleSetDiff(old: RuleSetHandle<Self>) -> Self::RuleSetDiff {
        ProductionRuleSetDiff {
            old,
            changed_rules: Vec::new(),
            new_rules: None,
            cleared: false,
            unrepresentable: false,
        }
    }
    fn AddRuleDiff(diff: &mut Self::RuleSetDiff, rule: Rc<StyleRuleBase<Self>>) {
        diff.changed_rules.push(rule)
    }
    fn MarkDiffUnrepresentable(diff: &mut Self::RuleSetDiff) {
        diff.unrepresentable = true
    }
    fn NewRuleSetCleared(diff: &mut Self::RuleSetDiff) {
        diff.cleared = true
    }
    fn NewRuleSetCreated(diff: &mut Self::RuleSetDiff, rules: RuleSetHandle<Self>) {
        diff.new_rules = Some(rules)
    }
}

impl RuleSetIngestionBackend for Backend {
    fn CloneRule(
        rule: &StyleRuleBase<Self>,
        parent: Option<Rc<StyleRule<Self>>>,
        bindings: Option<Rc<Self::MixinParameterBindings>>,
    ) -> Rc<StyleRuleBase<Self>> {
        fn clone_child(
            child: &StyleRuleBase<Backend>,
            parent: Option<Rc<StyleRule<Backend>>>,
            bindings: Option<Rc<ProductionMixinParameterBindings>>,
        ) -> Rc<StyleRuleBase<Backend>> {
            let clone_children = |children: &[Rc<StyleRuleBase<Backend>>]| {
                children
                    .iter()
                    .map(|child| clone_child(child, parent.clone(), bindings.clone()))
                    .collect::<Vec<_>>()
            };
            match child {
                StyleRuleBase::Style(style) => {
                    let parent_for_selector = parent
                        .clone()
                        .map(|parent| parent as Rc<dyn crate::css_selector::CSSSelectorParentRule>);
                    let selectors = style.Selectors().Copy().Renest(parent_for_selector);
                    let cloned = Rc::new(StyleRule::Create(
                        selectors.as_ref().clone(),
                        style.Properties().ImmutableCopyIfNeeded(),
                        bindings.clone(),
                    ));
                    if let Some(children) = style.ChildRules() {
                        for child in children.iter() {
                            cloned.AddChildRule(clone_child(
                                child,
                                Some(cloned.clone()),
                                bindings.clone(),
                            ));
                        }
                    }
                    Rc::new(StyleRuleBase::Style(cloned))
                }
                StyleRuleBase::NestedDeclarations(nested) => {
                    let inner = nested.InnerStyleRule();
                    let parent_for_selector = parent
                        .map(|parent| parent as Rc<dyn crate::css_selector::CSSSelectorParentRule>);
                    let selectors = inner.Selectors().Copy().Renest(parent_for_selector);
                    let cloned = StyleRule::Create(
                        selectors.as_ref().clone(),
                        inner.Properties().ImmutableCopyIfNeeded(),
                        bindings,
                    );
                    Rc::new(StyleRuleBase::NestedDeclarations(Rc::new(
                        crate::style_rule_nested_declarations::StyleRuleNestedDeclarations::new(
                            nested.NestingType(),
                            cloned,
                        ),
                    )))
                }
                StyleRuleBase::Result(group) => Rc::new(StyleRuleBase::Result(
                    crate::style_rule::StyleRuleResult::CopyWithRules(
                        group,
                        clone_children(group.ChildRules()),
                    ),
                )),
                StyleRuleBase::ApplyMixin(group) => Rc::new(StyleRuleBase::ApplyMixin(
                    crate::style_rule::StyleRuleApplyMixin::CopyWithRules(
                        group,
                        clone_children(group.ChildRules()),
                    ),
                )),
                StyleRuleBase::Contents(group) => Rc::new(StyleRuleBase::Contents(
                    crate::style_rule::StyleRuleContentsStatement::CopyWithRules(
                        group,
                        clone_children(group.ChildRules()),
                    ),
                )),
                StyleRuleBase::Media(group) => Rc::new(StyleRuleBase::Media(
                    crate::style_rule::StyleRuleMedia::CopyWithRules(
                        group,
                        clone_children(group.ChildRules()),
                    ),
                )),
                StyleRuleBase::Supports(group) => Rc::new(StyleRuleBase::Supports(
                    crate::style_rule::StyleRuleSupports::CopyWithRules(
                        group,
                        clone_children(group.ChildRules()),
                    ),
                )),
                StyleRuleBase::Container(group) => Rc::new(StyleRuleBase::Container(
                    crate::style_rule::StyleRuleContainer::CopyWithRules(
                        group,
                        clone_children(group.ChildRules()),
                    ),
                )),
                _ => panic!("unsupported rule inside @mixin expansion"),
            }
        }

        // cpp: style_rule.cc:604-624,642-821. RuleSet invokes this only for
        // @apply/@contents/@result group clones; their children must be
        // re-nested and carry the exact binding owner into MatchResult.
        match rule {
            StyleRuleBase::ApplyMixin(rule) => Rc::new(StyleRuleBase::ApplyMixin(
                crate::style_rule::StyleRuleApplyMixin::CopyWithRules(
                    rule,
                    rule.ChildRules()
                        .iter()
                        .map(|child| clone_child(child, parent.clone(), bindings.clone()))
                        .collect(),
                ),
            )),
            StyleRuleBase::Contents(rule) => Rc::new(StyleRuleBase::Contents(
                crate::style_rule::StyleRuleContentsStatement::CopyWithRules(
                    rule,
                    rule.ChildRules()
                        .iter()
                        .map(|child| clone_child(child, parent.clone(), bindings.clone()))
                        .collect(),
                ),
            )),
            StyleRuleBase::Result(rule) => Rc::new(StyleRuleBase::Result(
                crate::style_rule::StyleRuleResult::CopyWithRules(
                    rule,
                    rule.ChildRules()
                        .iter()
                        .map(|child| clone_child(child, parent.clone(), bindings.clone()))
                        .collect(),
                ),
            )),
            _ => unreachable!("RuleSet only clones @apply/@contents/@result during ingestion"),
        }
    }
    fn MediaRuleQueries(rule: &StyleRuleMedia<Self>) -> Option<Rc<Self::MediaQuerySet>> {
        rule.MediaQueries().map(|queries| Rc::new(queries.clone()))
    }
    fn ContainerRuleQueries(
        rule: &crate::style_rule::StyleRuleContainer<Self>,
    ) -> Rc<Self::ContainerQuerySet> {
        Rc::new(rule.GetContainerQuerySet().clone())
    }
    fn ScopeRuleScope(rule: &crate::style_rule::StyleRuleScope<Self>) -> Rc<Self::StyleScope> {
        rule.StyleScopeHandle()
    }
    fn CopyContainerQueriesWithParent(
        queries: &Self::ContainerQuerySet,
        parent: Rc<Self::ContainerQuerySet>,
    ) -> Rc<Self::ContainerQuerySet> {
        queries.CopyWithParent(Some(parent))
    }
    fn CopyStyleScopeWithParent(
        scope: &Self::StyleScope,
        parent: Rc<Self::StyleScope>,
    ) -> Rc<Self::StyleScope> {
        scope.CopyWithParent(Some(parent))
    }
    fn NestedDeclarationsInnerStyleRule(
        rule: &Self::StyleRuleNestedDeclarations,
    ) -> Rc<StyleRule<Self>> {
        // Chromium's StyleRuleNestedDeclarations::InnerStyleRule returns its
        // retained member, not a copy. Routing, invalidation and RuleSet diff
        // filters all compare this identity with the stylesheet's owner.
        rule.InnerStyleRule().clone()
    }
    fn FunctionDeclarationValues(
        rule: &Self::StyleRuleFunctionDeclarations,
    ) -> Vec<(String, Rc<Self::CSSVariableData>)> {
        let properties = rule.Properties();
        let collect = |values: &[CSSPropertyValue<ProductionCSSValueDispatch>]| {
            values
                .iter()
                .filter_map(|property| {
                    if property.PropertyID() != CSSPropertyID::kVariable {
                        return None;
                    }
                    let CSSValuePayload::kUnparsedDeclarationClass(value) =
                        property.Value().Payload()
                    else {
                        return None;
                    };
                    Some((
                        String::from_utf16(
                            property
                                .CustomPropertyName()
                                .utf16_units()
                                .unwrap_or_default(),
                        ),
                        value.data.clone(),
                    ))
                })
                .collect()
        };
        match properties.as_ref() {
            CSSPropertyValueSetRuleHandle::Immutable(set) => collect(set.Properties()),
            CSSPropertyValueSetRuleHandle::Mutable(set) => collect(set.borrow().Properties()),
        }
    }
    fn NewMixinParameterBindings(
        bindings: HashMap<String, RuleSetMixinBinding<Self>>,
        locals: RuleSetMixinLocals<Self>,
        conditional_locals: RuleSetConditionalLocals<Self>,
        parent: Option<Rc<Self::MixinParameterBindings>>,
    ) -> Rc<Self::MixinParameterBindings> {
        Rc::new(ProductionMixinParameterBindings {
            bindings,
            locals,
            conditional_locals,
            parent,
        })
    }
    fn IngestionDocument(_medium: &Self::MediaQueryEvaluator) -> Option<&Self::Document> {
        None
    }
    fn CreateRouteIfNeeded(rule: &Self::StyleRuleLocation, _document: Option<&Self::Document>) {
        match *rule {}
    }
    fn EvaluateNavigationQuery(
        query: &Self::NavigationQuery,
        _document: Option<&Self::Document>,
    ) -> bool {
        match *query {}
    }
    fn IngestionImportIsSupported(rule: &Self::StyleRuleImport) -> bool {
        rule.metadata.IsSupported()
    }
    fn IngestionImportMediaQueries(
        rule: &Self::StyleRuleImport,
    ) -> Option<Rc<Self::MediaQuerySet>> {
        Some(rule.metadata.MediaQueries())
    }
    fn ImportIsLayered(rule: &Self::StyleRuleImport) -> bool {
        !rule.metadata.GetLayerName().is_empty()
    }
    fn ImportLayerName(rule: &Self::StyleRuleImport) -> &[AtomicString] {
        rule.metadata.GetLayerName()
    }
    fn ImportScope(rule: &Self::StyleRuleImport) -> Option<Rc<Self::StyleScope>> {
        rule.metadata.scope.clone()
    }
    fn IngestionImportStyleSheet(
        _rule: &Self::StyleRuleImport,
    ) -> Option<Rc<dyn RuleSetSheetView<Self>>> {
        // Detached parser output publishes an ImportLoad.  Once its loader
        // supplies child contents this hook can return that retained sheet.
        None
    }
    fn CopyNavigationState(_state: &Self::NavigationState) -> Rc<Self::NavigationState> {
        Rc::new(())
    }
    fn BeginStyleSheetContents(_sheet: &dyn RuleSetSheetView<Self>) {}
    fn EndStyleSheetContents() {}
}

impl RuleSetBackend for Backend {
    type BucketTable = ProductionRuleBucketTable;
    type RuleFeatureSet = ProductionRuleFeatureSet;
    type MediaQueryEvaluator = ProductionMediaQueryEvaluator;
    type NavigationState = ();
    type Document = ();
    type SubstringSetMatcher = ProductionSubstringSetMatcher;
    fn CueShadowPseudoId() -> AtomicString {
        AtomicString::from_str("-internal-media-track-cue")
    }
    fn StringForUAShadowPseudoId(pseudo_id: PseudoId) -> AtomicString {
        match pseudo_id {
            PseudoId::kPseudoIdPlaceholder => AtomicString::from_str("-webkit-input-placeholder"),
            // Unknown UA-shadow pseudo ids remain in the universal bucket.
            // This is conservative: it can cost a selector check but cannot
            // drop a rule.
            _ => AtomicString::default(),
        }
    }
    fn SelectorIsEasy(selector: CSSSelectorComplex<'_>) -> bool {
        crate::selector_checker::EasySelectorChecker::IsEasy(selector)
    }
    fn CollectIdentifierHashes(
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::StyleScope>,
        backing: &mut Vec<u16>,
        subject_filter: &mut u32,
    ) {
        crate::selector_filter::CollectIdentifierHashes(selector, scope, backing, subject_filter);
    }
    fn BeginSelector(_rule: &Rc<StyleRule<Self>>, _selector_index: u32) {}
    fn EndSelector() {}
    fn CollectFeaturesFromSelector(
        features: &mut Self::RuleFeatureSet,
        selector: CSSSelectorComplex<'_>,
        scope: Option<&Self::StyleScope>,
    ) -> SelectorPreMatch {
        collect_user_action_features(features, selector, scope);
        collect_attribute_features(features, selector, scope);
        SelectorPreMatch::kMayMatch
    }
    fn NewFeatures() -> Self::RuleFeatureSet {
        ProductionRuleFeatureSet::default()
    }
    fn MutableMediaQueryResultFlags(
        features: &mut Self::RuleFeatureSet,
    ) -> &mut MediaQueryResultFlags {
        &mut features.media_query_result_flags
    }
    fn EvalMedia(
        evaluator: &Self::MediaQueryEvaluator,
        queries: &Self::MediaQuerySet,
        flags: &mut MediaQueryResultFlags,
    ) -> bool {
        evaluator.evaluator().EvalSet(queries, Some(flags), None)
    }
    fn DidResultsChange(
        evaluator: &Self::MediaQueryEvaluator,
        results: &[MediaQuerySetResult],
    ) -> bool {
        evaluator
            .evaluator()
            .DidResultsChange(results.iter().filter_map(|result| {
                result
                    .MediaQueries()
                    .AsAny()
                    .downcast_ref::<QuerySet>()
                    .map(|queries| (queries, result.Result()))
            }))
    }
    fn NavigationStateForDocument(
        document: Option<&Self::Document>,
    ) -> Option<&Self::NavigationState> {
        None
    }
    fn NewSubstringSetMatcher() -> Self::SubstringSetMatcher {
        ProductionSubstringSetMatcher(Vec::new())
    }
    fn BuildSubstringSetMatcher(
        matcher: &mut Self::SubstringSetMatcher,
        patterns: &[RuleSetSubstringPattern],
    ) -> bool {
        matcher.0 = patterns.to_vec();
        true
    }
    fn SubstringAnyMatch(matcher: &Self::SubstringSetMatcher, value: &[u8]) -> bool {
        matcher.0.iter().any(|pattern| {
            !pattern.pattern.is_empty()
                && value
                    .windows(pattern.pattern.len())
                    .any(|candidate| candidate == pattern.pattern.as_bytes())
        })
    }
}

impl CSSPropertyValueSetBackend for ProductionCSSValueDispatch {
    type CSSStyleDeclaration = Unavailable;
    type ExecutionContext = Unavailable;
    fn ShorthandForProperty(id: CSSPropertyID) -> Vec<CSSPropertyID> {
        crate::parser::production_property_metadata::ShorthandFor(id).to_vec()
    }
    fn IsInSameLogicalPropertyGroupWithDifferentMappingLogic(
        a: CSSPropertyID,
        b: CSSPropertyID,
    ) -> bool {
        crate::properties::css_property::CSSProperty::Get(a)
            .IsInSameLogicalPropertyGroupWithDifferentMappingLogic(b)
    }
    fn SerializeShorthand(set: &CSSPropertyValueSet<Self>, id: CSSPropertyID) -> String {
        unavailable("CSSPropertyValueSetBackend::SerializeShorthand")
    }
    fn AsText(set: &CSSPropertyValueSet<Self>) -> String {
        unavailable("CSSPropertyValueSetBackend::AsText")
    }
    fn CreateIdentifier(id: CSSValueID) -> Rc<CSSValue<Self>> {
        values::identifier(id)
    }
    fn DeclarationPropertyValueSet(
        style: &Self::CSSStyleDeclaration,
    ) -> Option<&CSSPropertyValueSet<Self>> {
        unavailable("CSSPropertyValueSetBackend::DeclarationPropertyValueSet")
    }
    fn DeclarationPropertyMatches(
        style: &Self::CSSStyleDeclaration,
        id: CSSPropertyID,
        value: &CSSValue<Self>,
    ) -> bool {
        unavailable("CSSPropertyValueSetBackend::DeclarationPropertyMatches")
    }
    fn NewCSSStyleDeclaration(
        context: Option<&Self::ExecutionContext>,
        set: &MutableCSSPropertyValueSet<Self>,
    ) -> Rc<Self::CSSStyleDeclaration> {
        unavailable("CSSPropertyValueSetBackend::NewCSSStyleDeclaration")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn parse(css: &str) -> ProductionStyleSheet {
        ParseStyleSheet(
            &String::from_utf16(&css.encode_utf16().collect::<Vec<_>>()),
            CSSParserMode::kHTMLStandardMode,
        )
    }
    fn feature_values_rule(
        sheet: &ProductionStyleSheet,
        index: usize,
    ) -> Rc<crate::style_rule_font_feature_values::StyleRuleFontFeatureValues> {
        let rule = sheet.contents.RuleAt(index);
        let StyleRuleBase::FontFeatureValues(rule) = rule.as_ref() else {
            panic!("expected parser-owned @font-feature-values")
        };
        rule.clone()
    }

    #[test]
    fn font_feature_values_retain_all_types_families_and_duplicate_alias_order() {
        let sheet = parse(
            r#"@font-feature-values "Quoted, Face", Unquoted Face, Esc\61 ped, "serif" {
                @stylistic { named: 1; }
                @styleset { readable: 1 2 3; readable: 4 5; Readable: 6; retained: 7; }
                @character-variant { single: 8; pair: 9 10; }
                @swash { named: 11; }
                @ornaments { named: 12; }
                @annotation { named: 13; }
                @styleset { readable: 14 15; }
            } tail { width: 17px; }"#,
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.contents.RuleCount(), 2);
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selector_source.Utf8(), "tail ");
        let rule = feature_values_rule(&sheet, 0);
        assert_eq!(
            rule.GetFamilies()
                .iter()
                .map(AtomicString::Utf8)
                .collect::<Vec<_>>(),
            vec!["Quoted, Face", "Unquoted Face", "Escaped", "serif"],
        );
        let storage = rule.Storage();
        let named = AtomicString::from_str("named");
        assert_eq!(storage.ResolveStylistic(&named), vec![1]);
        assert_eq!(
            storage.ResolveStyleset(&AtomicString::from_str("readable")),
            vec![14, 15]
        );
        assert_eq!(
            storage.ResolveStyleset(&AtomicString::from_str("Readable")),
            vec![6]
        );
        assert_eq!(
            storage.ResolveStyleset(&AtomicString::from_str("retained")),
            vec![7]
        );
        assert_eq!(
            storage.ResolveCharacterVariant(&AtomicString::from_str("single")),
            vec![8]
        );
        assert_eq!(
            storage.ResolveCharacterVariant(&AtomicString::from_str("pair")),
            vec![9, 10]
        );
        assert_eq!(storage.ResolveSwash(&named), vec![11]);
        assert_eq!(storage.ResolveOrnaments(&named), vec![12]);
        assert_eq!(storage.ResolveAnnotation(&named), vec![13]);
        let set = sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::default());
        assert_eq!(set.FontFeatureValuesRules().len(), 1);
        assert!(Rc::ptr_eq(&set.FontFeatureValuesRules()[0].value, &rule));
    }

    #[test]
    fn font_feature_values_reject_generic_and_invalid_family_preludes() {
        for family in [
            "serif",
            "sans-serif",
            "monospace",
            "cursive",
            "fantasy",
            "system-ui",
            "math",
            "-webkit-body",
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "default",
            "Named, serif",
            "Named,",
            "",
            "12px",
        ] {
            let sheet = parse(&format!(
                "@font-feature-values {family} {{ @styleset {{ readable: 2; }} }} tail {{width:1px}}"
            ));
            assert_eq!(sheet.contents.RuleCount(), 1, "{family}");
            assert_eq!(sheet.rules.len(), 1, "{family}");
            assert!(!sheet.diagnostics.is_empty(), "{family}");
            assert!(
                sheet
                    .diagnostics
                    .iter()
                    .all(|d| d.kind == DiagnosticKind::Invalid),
                "{family}: {:?}",
                sheet.diagnostics
            );
        }
        // CSS-wide words are permitted as part of a multi-word family name.
        // The pinned keyword table has no ui-* or fangsong generic IDs, so
        // these spellings are ordinary named families in this source slice.
        let sheet = parse(
            "@font-feature-values inherit Font, default Face, ui-serif, ui-sans-serif, \
             ui-monospace, ui-rounded, fangsong { @styleset { readable:2 } }",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(feature_values_rule(&sheet, 0).GetFamilies().len(), 7);
    }

    #[test]
    fn font_feature_values_number_math_rounds_clamps_and_retains_zero() {
        let sheet = parse(
            "@font-feature-values Face { @styleset {
                values: 0 calc(1 + 1) calc(2.5) calc(-3) min(8, 4) max(1, 5) clamp(1, 6, 9) 999999999999;
                advanced: abs(-2) pow(2, 3) sqrt(9) mod(8, 3) sin(90deg) round(up, 2.1) calc(abs(-4) + 1);
            } @character-variant { pair: calc(2.49) calc(3.5); } }",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let rule = feature_values_rule(&sheet, 0);
        assert_eq!(
            rule.Storage()
                .ResolveStyleset(&AtomicString::from_str("values")),
            vec![0, 2, 3, 0, 4, 5, 6, i32::MAX as u32],
        );
        assert_eq!(
            rule.Storage()
                .ResolveCharacterVariant(&AtomicString::from_str("pair")),
            vec![2, 4],
        );
        assert_eq!(
            rule.Storage()
                .ResolveStyleset(&AtomicString::from_str("advanced")),
            vec![2, 8, 3, 2, 1, 3, 5],
        );
    }

    #[test]
    fn font_feature_values_invalid_blocks_are_discarded_and_following_blocks_recover() {
        for block in [
            "@styleset { prior: 1; broken: -1; }",
            "@styleset { prior: 1; broken: 1.5; }",
            "@styleset { prior: 1; broken: calc(2px); }",
            "@styleset { prior: 1; broken: calc(20%); }",
            "@styleset { prior: 1; broken: var(--index); }",
            "@styleset { prior: 1; broken: ; }",
            "@styleset { prior: 1; broken: 1!important; }",
            "@styleset { prior: 1; broken 2; }",
            "@styleset { prior: 1; \"broken\": 2; }",
            "@stylistic { prior: 1; broken: 2 3; }",
            "@character-variant { prior: 1; broken: 2 3 4; }",
            "@swash { prior: 1; broken: 2 3; }",
            "@ornaments { prior: 1; broken: 2 3; }",
            "@annotation { prior: 1; broken: 2 3; }",
        ] {
            let sheet = parse(&format!(
                "@font-feature-values Face {{ {block} @styleset {{ good:2 }} }} tail {{width:1px}}"
            ));
            assert_eq!(sheet.contents.RuleCount(), 2, "{block}");
            assert_eq!(sheet.rules.len(), 1, "{block}");
            assert!(!sheet.diagnostics.is_empty(), "{block}");
            assert!(
                sheet
                    .diagnostics
                    .iter()
                    .all(|d| d.kind == DiagnosticKind::Invalid),
                "{block}: {:?}",
                sheet.diagnostics
            );
            let rule = feature_values_rule(&sheet, 0);
            let storage = rule.Storage();
            assert_eq!(
                storage.ResolveStyleset(&AtomicString::from_str("good")),
                vec![2],
                "{block}"
            );
            let prior = AtomicString::from_str("prior");
            assert!(storage.ResolveStyleset(&prior).is_empty(), "{block}");
            assert!(storage.ResolveStylistic(&prior).is_empty(), "{block}");
            assert!(
                storage.ResolveCharacterVariant(&prior).is_empty(),
                "{block}"
            );
            assert!(storage.ResolveSwash(&prior).is_empty(), "{block}");
            assert!(storage.ResolveOrnaments(&prior).is_empty(), "{block}");
            assert!(storage.ResolveAnnotation(&prior).is_empty(), "{block}");
        }
    }

    #[test]
    fn font_feature_values_unsupported_numeric_evaluation_is_explicit_and_recovers() {
        for value in ["sign(1em)", "calc(1em / 1px)", "sibling-index()"] {
            let sheet = parse(&format!(
                "@font-feature-values Face {{ @styleset {{ prior:1; unsupported:{value}; }} @styleset {{ good:2 }} }}"
            ));
            assert!(
                sheet
                    .diagnostics
                    .iter()
                    .any(|d| d.kind == DiagnosticKind::Unsupported),
                "{value}: {:?}",
                sheet.diagnostics
            );
            let rule = feature_values_rule(&sheet, 0);
            assert!(rule
                .Storage()
                .ResolveStyleset(&AtomicString::from_str("prior"))
                .is_empty());
            assert_eq!(
                rule.Storage()
                    .ResolveStyleset(&AtomicString::from_str("good")),
                vec![2]
            );
        }
    }

    #[test]
    fn font_feature_values_native_ingestion_respects_layers_media_and_supports() {
        let sheet = parse(
            "@layer typography { @media screen and (min-width:600px) {
                @font-feature-values Face { @styleset { wide:2 } }
            }} @media print { @font-feature-values Print { @styleset { print:3 } } }
            @supports (unknown:x) { @font-feature-values Hidden { @styleset { hidden:4 } } }
            @font-feature-values Always { @styleset { always:5 } }",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert!(sheet.rules.is_empty());
        let wide = sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0));
        assert_eq!(wide.FontFeatureValuesRules().len(), 2);
        let layered = &wide.FontFeatureValuesRules()[0];
        assert_eq!(layered.value.GetFamilies()[0].Utf8(), "Face");
        assert_eq!(
            layered.layer.as_ref().unwrap().GetName().Utf8(),
            "typography"
        );
        assert!(wide.FontFeatureValuesRules()[1].layer.is_none());
        let narrow = sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::Screen(400.0, 600.0));
        assert_eq!(narrow.FontFeatureValuesRules().len(), 1);
        assert_eq!(
            narrow.FontFeatureValuesRules()[0].value.GetFamilies()[0].Utf8(),
            "Always"
        );
    }

    #[test]
    fn font_feature_values_observer_records_nested_feature_type_and_eof_recovers() {
        use crate::style_rule_font_feature_values::FontFeatureType;
        let seed = parse("");
        let context = seed.contents.ParserContext().clone();
        let mut contents =
            StyleSheetContents::<Backend>::new(context.clone(), String::default(), None);
        let text = String::from(
            "@font-feature-values Face { @stylistic {a:1} @styleset {a:2} \
             @character-variant {a:3} @swash {a:4} @ornaments {a:5} @annotation {a:6} }",
        );
        let mut observer = SourceObserver {
            source: text.clone(),
            ..SourceObserver::default()
        };
        CSSParserImpl::<Backend>::ParseStyleSheetForInspector(
            &text,
            Some(&context),
            &mut contents,
            &mut observer,
        );
        assert!(observer.diagnostics.is_empty());
        assert!(observer.stack.is_empty());
        assert_eq!(observer.ranges.len(), 7);
        assert_eq!(
            observer
                .ranges
                .iter()
                .filter_map(|range| range.font_feature_type)
                .collect::<Vec<_>>(),
            vec![
                FontFeatureType::kStylistic,
                FontFeatureType::kStyleset,
                FontFeatureType::kCharacterVariant,
                FontFeatureType::kSwash,
                FontFeatureType::kOrnaments,
                FontFeatureType::kAnnotation,
            ],
        );
        assert_eq!(
            observer.ranges.last().unwrap().rule_type,
            RuleType::kFontFeatureValues
        );

        let sheet = parse("@font-feature-values Face { @styleset { readable:2");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(
            feature_values_rule(&sheet, 0)
                .Storage()
                .ResolveStyleset(&AtomicString::from_str("readable")),
            vec![2]
        );
        let empty = parse("@font-feature-values Face {} tail{width:1px}");
        assert!(empty.diagnostics.is_empty(), "{:?}", empty.diagnostics);
        assert_eq!(empty.contents.RuleCount(), 2);
    }

    #[test]
    fn native_rule_selector_and_value_allocations_preserve_original_ranges() {
        let sheet = parse(
            "/*leading*/ a\\+b[data-v='a}b']/**/ { color : red ; width: 12px; --x:var(--y, 2px); } /*tail*/",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(
            sheet.rules[0].selector_source.Utf8(),
            "a\\+b[data-v='a}b']/**/ "
        );
        assert_eq!(
            sheet.rules[0].declaration_source.Utf8(),
            " color : red ; width: 12px; --x:var(--y, 2px); "
        );
        assert_eq!(
            StyleRulePropertySet::GetPropertyCSSValue(
                sheet.rules[0].rule.Properties().as_ref(),
                CSSPropertyID::kWidth
            )
            .unwrap()
            .CssText()
            .Utf8(),
            "12px"
        );
        assert_eq!(sheet.contents.RuleCount(), 1);
    }
    #[test]
    fn starting_style_retains_its_child_for_projection_and_rule_set_ingestion() {
        let sheet = parse("@starting-style{.hit{font-size:31px}}.hit{width:7px}");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.contents.RuleCount(), 2);
        assert!(matches!(
            sheet.contents.RuleAt(0).as_ref(),
            StyleRuleBase::StartingStyle(_)
        ));
        assert_eq!(sheet.rules.len(), 2);

        let set = sheet.BuildRuleSet(&ProductionMediaQueryEvaluator::default());
        let rules = set.ClassRules(&AtomicString::from_str("hit"));
        assert_eq!(rules.len(), 2);
        assert_eq!(
            rules.iter().filter(|rule| rule.IsStartingStyle()).count(),
            1
        );
    }
    #[test]
    fn nested_media_and_layers_keep_source_order_and_typed_queries() {
        let sheet = parse(
            "@layer reset, theme; @media screen { @layer theme { a {width:12px} @media(min-width:700px){b{height:23px}} } } @layer {c{opacity:.5}} d{width:5px}",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.layer_statements.len(), 1);
        assert_eq!(sheet.layer_statements[0].source_order, 0);
        assert_eq!(sheet.rules.len(), 4);
        assert_eq!(sheet.rules[0].media.len(), 1);
        assert_eq!(sheet.rules[1].media.len(), 2);
        assert_eq!(sheet.rules[1].media[1].QueryVector().len(), 1);
        assert_eq!(
            sheet.rules[0].layers,
            vec![LayerSegment::Named(vec![AtomicString::from_str("theme")])]
        );
        assert!(matches!(
            sheet.rules[2].layers.as_slice(),
            [LayerSegment::Anonymous(_)]
        ));
        assert!(sheet.rules[3].layers.is_empty());
        assert!(sheet
            .rules
            .windows(2)
            .all(|pair| pair[0].source_order < pair[1].source_order));
        assert_eq!(sheet.rules[1].declaration_source.Utf8(), "height:23px");
    }
    #[test]
    fn translated_recovery_handles_bad_selectors_and_unsupported_rules() {
        let sheet = parse(
            "!bad{width:99px}@counter-style bad{ignored{}} @font-face{font-family:missing;src:url('a}b')} @media screen; good{width:7px}",
        );
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selector_source.Utf8(), "good");
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unsupported));
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
    }
    #[test]
    fn block_start_during_qualified_rule_recovery_is_not_consumed_as_an_ident() {
        // css-syntax recovery may ask ConsumeStyleRule to inspect a stray
        // block-start token. Chromium's StartsCustomPropertyDeclaration exits
        // before speculative consumption unless the token is a valid --name.
        let sheet = parse("{} good{width:7px}");
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selector_source.Utf8(), "good");
    }
    #[test]
    fn declarations_accept_comments_nested_blocks_and_eof_recovery() {
        let sheet = parse(
            "a{--payload:[a;{b:'}'}];width:var(--size,17px);color:red!important;color:blue; height:21px",
        );
        assert_eq!(sheet.rules.len(), 1);
        let properties = sheet.rules[0].rule.Properties();
        assert!(StyleRulePropertySet::GetPropertyCSSValue(
            properties.as_ref(),
            CSSPropertyID::kWidth
        )
        .unwrap()
        .IsUnparsedDeclaration());
        assert_eq!(
            StyleRulePropertySet::GetPropertyCSSValue(properties.as_ref(), CSSPropertyID::kColor)
                .unwrap()
                .CssText()
                .Utf8(),
            "red"
        );
        assert_eq!(
            StyleRulePropertySet::GetPropertyCSSValue(properties.as_ref(), CSSPropertyID::kHeight)
                .unwrap()
                .CssText()
                .Utf8(),
            "21px"
        );
    }
    #[test]
    fn nesting_and_native_context_pseudos_recover_without_panics() {
        let sheet = parse(
            "@media screen{!bad{} :playing{width:98px} :open{width:99px} good{width:7px}} outer{width:3px;& child{height:99px} @media screen{width:88px}} tail{height:5px}",
        );
        assert_eq!(sheet.rules.len(), 5);
        let names: Vec<_> = sheet
            .rules
            .iter()
            .map(|rule| rule.selector_source.Utf8())
            .collect();
        assert_eq!(names, [":open", "good", "outer", "& child", "tail"]);
        assert!(!sheet.selector_ranges.is_empty());
        assert!(sheet.selector_ranges.iter().any(|range| {
            SourceSlice(&sheet.source, range.start, range.end)
                .Utf8()
                .trim()
                == ":open"
        }));
    }
    fn property(
        properties: &CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>,
        id: CSSPropertyID,
    ) -> Option<Rc<values::Value>> {
        StyleRulePropertySet::GetPropertyCSSValue(properties, id)
    }
    #[test]
    fn font_face_descriptors_are_typed_and_keep_exact_body_source() {
        let body = " /* } */ font-family: 'Demo'; src: local(Demo Face), bogus(x), url('a}b.woff2') format(woff2) tech(variations); unicode-range: U+0-7F, U+4??; font-weight: 900 100; font-stretch: 75% 125%; font-display: swap; ascent-override: 90%; size-adjust: 110%; ";
        let sheet = parse(&format!(
            "@media screen{{@layer fonts{{@font-face{{{body}}}}}}}"
        ));
        assert_eq!(sheet.font_faces.len(), 1);
        let face = &sheet.font_faces[0];
        assert_eq!(face.declaration_source.Utf8(), body);
        assert_eq!(face.media.len(), 1);
        assert_eq!(
            face.layers,
            vec![LayerSegment::Named(vec![AtomicString::from_str("fonts")])]
        );
        let properties = face.rule.Properties();
        assert!(matches!(
            property(properties, CSSPropertyID::kFontFamily)
                .unwrap()
                .Payload(),
            CSSValuePayload::kFontFamilyClass(_)
        ));
        let src = property(properties, CSSPropertyID::kSrc).unwrap();
        let CSSValuePayload::kValueListClass(sources) = src.Payload() else {
            panic!("typed src list")
        };
        assert_eq!(sources.values.len(), 2);
        assert!(matches!(
            sources.values[0].Payload(),
            CSSValuePayload::kFontFaceSrcClass(values::CSSFontFaceSrcValue {
                resource: values::FontFaceResource::Local(_),
                ..
            })
        ));
        assert_eq!(
            src.CssText().Utf8(),
            "local(\"Demo Face\"), url(\"a}b.woff2\") format(\"woff2\") tech(variations)"
        );
        assert_eq!(
            property(properties, CSSPropertyID::kUnicodeRange)
                .unwrap()
                .CssText()
                .Utf8(),
            "U+0-7F, U+400-4FF"
        );
        assert_eq!(
            property(properties, CSSPropertyID::kFontWeight)
                .unwrap()
                .CssText()
                .Utf8(),
            "900 100"
        );
        assert_eq!(
            property(properties, CSSPropertyID::kFontStretch)
                .unwrap()
                .CssText()
                .Utf8(),
            "75% 125%"
        );
        assert_eq!(
            property(properties, CSSPropertyID::kFontDisplay)
                .unwrap()
                .CssText()
                .Utf8(),
            "swap"
        );
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
        assert!(!sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unsupported));
    }
    #[test]
    fn keyframe_consumers_recover_and_retain_names_offsets_and_declarations() {
        let sheet = parse(
            "@layer motion{@media screen{@-webkit-keyframes f\\61 de { from, 50% {opacity:0;--x:var(--y);width:8px!important;} 120%{opacity:.9} unknown{width:0} to {opacity:1} } @keyframes fade{entry -20%{height:4px}}}} tail{width:1px}",
        );
        assert_eq!(sheet.keyframes.len(), 2);
        let first = &sheet.keyframes[0];
        assert_eq!(first.rule.name.Utf8(), "fade");
        assert!(first.rule.vendor_prefixed);
        assert_eq!(first.keyframes.len(), 2);
        assert_eq!(first.media.len(), 1);
        assert_eq!(first.keyframes[0].key_source.Utf8(), "from, 50% ");
        assert_eq!(
            first.keyframes[0].declaration_source.Utf8(),
            "opacity:0;--x:var(--y);width:8px!important;"
        );
        assert_eq!(
            first.keyframes[0]
                .rule
                .Keys()
                .iter()
                .map(|key| key.percent)
                .collect::<Vec<_>>(),
            vec![0.0, 0.5]
        );
        assert!(property(&first.keyframes[0].rule.Properties(), CSSPropertyID::kWidth).is_none());
        let custom = first.keyframes[0].rule.Properties();
        let CSSPropertyValueSetRuleHandle::Immutable(custom) = custom.as_ref() else {
            panic!("immutable keyframes")
        };
        let declaration = custom
            .GetPropertyCSSValue(&AtomicString::from_str("--x"))
            .unwrap();
        let CSSValuePayload::kUnparsedDeclarationClass(declaration) = declaration.Payload() else {
            panic!("unparsed custom property")
        };
        assert!(declaration.data.is_animation_tainted);
        assert_eq!(first.keyframes[1].rule.KeyText().Utf8(), "100%");
        assert!(!sheet.keyframes[1].rule.vendor_prefixed);
        assert_eq!(
            sheet.keyframes[1].keyframes[0].rule.KeyText().Utf8(),
            "entry -20%"
        );
        assert_eq!(sheet.rules[0].selector_source.Utf8(), "tail");
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
    }
    #[test]
    fn unsupported_descriptors_invalid_preludes_and_eof_are_recoverable() {
        let sheet = parse(
            "@font-face extra{src:url(no)} @keyframes none{from{width:99px}} @font-face{font-family:serif;src:url(a) tech(incremental),local('Demo');font-weight:bolder;font-display:swap!important;unicode-range:U+110000; font-feature-settings:'liga';font-style:oblique 20deg;size-adjust:calc(50%);font-family:'Valid'} @keyframes 'Quoted😀'{from{opacity:0!important}to{opacity:1",
        );
        assert_eq!(sheet.font_faces.len(), 1);
        let properties = sheet.font_faces[0].rule.Properties();
        assert_eq!(
            property(properties, CSSPropertyID::kFontFamily)
                .unwrap()
                .CssText()
                .Utf8(),
            "Valid"
        );
        assert_eq!(
            property(properties, CSSPropertyID::kSrc)
                .unwrap()
                .CssText()
                .Utf8(),
            "local(\"Demo\")"
        );
        assert!(property(properties, CSSPropertyID::kFontDisplay).is_none());
        assert!(property(properties, CSSPropertyID::kFontWeight).is_none());
        assert_eq!(sheet.keyframes.len(), 1);
        assert_eq!(sheet.keyframes[0].rule.name.Utf8(), "Quoted😀");
        assert_eq!(sheet.keyframes[0].keyframes.len(), 2);
        assert_eq!(
            sheet.keyframes[0].keyframes[1].declaration_source.Utf8(),
            "opacity:1"
        );
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unsupported));
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
    }
    #[test]
    fn supports_boolean_grammar_selector_features_and_inactive_ranges() {
        let sheet = parse(
            "@supports (unknown-prop:x){hidden{width:1px}} @supports not (unknown-prop:x){visible{width:2px}} @supports (width:1px) and selector(.a > .b){typed{height:3px}} @supports (width:1px) or (height:1px) and (color:red){invalid{}} @supports font-format(woff2){font{}} @supports font-tech(incremental){future{}} tail{width:4px}",
        );
        assert_eq!(
            sheet
                .rules
                .iter()
                .map(|r| r.selector_source.Utf8())
                .collect::<Vec<_>>(),
            vec!["visible", "typed", "font", "tail"]
        );
        assert_eq!(sheet.rules[3].declaration_source.Utf8(), "width:4px");
        assert_eq!(sheet.supports.len(), 5);
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
        let nested = parse(
            "@supports ((width:1px) and (height:2px)){nested{width:5px}} @supports not not (width:1px){bad{}} good{}",
        );
        assert_eq!(nested.rules.len(), 2);
        assert_eq!(nested.rules[0].selector_source.Utf8(), "nested");
    }
    #[test]
    fn import_effects_namespace_selectors_and_late_rule_recovery() {
        let sheet = parse(
            "@layer reset; @import url('theme.css') layer(theme) supports(display:grid) screen; @import 'unused.css' supports((unknown:value)); @namespace svg 'http://www.w3.org/2000/svg'; svg|rect{width:1px} @import 'late.css'; @namespace late 'urn:late'; tail{height:2px}",
        );
        assert_eq!(sheet.imports.len(), 2);
        assert_eq!(sheet.import_loads.len(), 2);
        let import = &sheet.imports[0].rule;
        assert_eq!(import.metadata.Href().Utf8(), "theme.css");
        assert_eq!(
            import.metadata.GetLayerName(),
            &vec![AtomicString::from_str("theme")]
        );
        assert!(import.metadata.IsSupported());
        assert!(!sheet.imports[1].rule.metadata.IsSupported());
        assert_eq!(import.metadata.MediaQueries().MediaText().Utf8(), "screen");
        assert!(import.parent.borrow().as_ref().unwrap().upgrade().is_some());
        assert_eq!(sheet.namespaces.len(), 1);
        assert_eq!(
            sheet
                .contents
                .NamespaceURIFromPrefix(&AtomicString::from_str("svg"))
                .Utf8(),
            "http://www.w3.org/2000/svg"
        );
        assert_eq!(sheet.rules.len(), 2);
        let selector = sheet.rules[0].rule.Selectors().First().unwrap();
        assert_eq!(
            selector.TagQName().NamespaceURI().Utf8(),
            "http://www.w3.org/2000/svg"
        );
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
    }
    #[test]
    fn nested_rules_and_bare_declarations_preserve_typed_parent_and_source_order() {
        let sheet = parse(
            ".outer{width:1px;&>.child{width:2px}width:3px;@media screen{height:4px;&>.inner{height:5px}height:6px}height:7px}tail{}",
        );
        assert_eq!(sheet.rules.len(), 4);
        assert!(Rc::ptr_eq(
            sheet.rules[1].parent_rule_for_nesting.as_ref().unwrap(),
            &sheet.rules[0].rule
        ));
        assert_eq!(sheet.rules[0].declaration_source.Utf8(), "width:1px;");
        let projected = crate::production_style_sheet_projection::ProjectStyleSheet(&sheet);
        assert_eq!(
            projected
                .rules
                .iter()
                .filter_map(|rule| rule.declarations.first())
                .map(|d| d.value.as_str())
                .collect::<Vec<_>>(),
            vec!["1px", "2px", "3px", "4px", "5px", "6px", "7px"]
        );
        assert_eq!(projected.rules[1].selector_text, ":is(.outer) > .child");
        assert_eq!(projected.rules[2].declaration_text, "width:3px;");
        assert_eq!(projected.rules[3].media_conditions, vec!["screen"]);
        assert_eq!(projected.rules[4].selector_text, ":is(.outer) > .inner");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let recovered = parse("a{width:1px;@counter-style bad{ignored{}}& b{height:2px}}tail{}");
        assert_eq!(recovered.rules.len(), 3);
        assert!(recovered
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unsupported));
    }
    #[test]
    fn projected_rule_typed_declarations_share_cssom_order_and_identity() {
        let mut sheet = parse(".outer{width:1px;&>.child{height:2px}color:red}tail{opacity:.5}");
        let projected = crate::production_style_sheet_projection::ProjectStyleSheet(&sheet);
        assert_eq!(projected.rules.len(), 4);

        let expected = projected
            .rules
            .iter()
            .map(|rule| {
                rule.declarations
                    .iter()
                    .map(|declaration| declaration.value.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let typed = (0..projected.rules.len())
            .map(|index| {
                sheet
                    .TypedDeclarationsForProjectedRule(index)
                    .expect("typed projected declarations")
            })
            .collect::<Vec<_>>();
        let actual = typed
            .iter()
            .map(|properties| {
                properties
                    .iter()
                    .map(|property| property.Value().CssText().Utf8())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            actual,
            vec![vec!["1px"], vec!["2px"], vec!["red"], vec!["0.5"]]
        );
        assert!(Rc::ptr_eq(
            &typed[2],
            &sheet.TypedDeclarationsForProjectedRule(2).unwrap()
        ));
        assert!(Rc::ptr_eq(
            &sheet.RetainedDeclarationsForProjectedRule(0).unwrap(),
            &sheet.rules[0].rule.Properties()
        ));
        assert!(!Rc::ptr_eq(&typed[1], &typed[2]));
        assert!(sheet
            .TypedDeclarationsForProjectedRule(projected.rules.len())
            .is_none());

        let nested = &sheet.nested_declarations[0].rule;
        let retained = Backend::NestedDeclarationsInnerStyleRule(nested);
        assert!(Rc::ptr_eq(&retained, nested.InnerStyleRule()));
        assert!(Rc::ptr_eq(
            &retained,
            &Backend::NestedDeclarationsInnerStyleRule(nested)
        ));
        assert_eq!(
            sheet.ProjectedRuleIndexForRetainedStyleRule(&retained),
            Some(2)
        );

        let nested_range = sheet.nested_declarations[0].source_range.clone();
        sheet.diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Unsupported,
            offset: nested_range.body_start,
            operation: "test unsupported translated operation",
            source_line: line!(),
        });
        assert!(matches!(
            sheet.ProjectedRuleDeclarationsAt(2),
            Some(ProjectedRuleDeclarations::RequiresLegacyFallback)
        ));
        assert!(sheet.TypedDeclarationsForProjectedRule(2).is_none());
        assert!(sheet.TypedDeclarationsForProjectedRule(1).is_some());
    }

    #[test]
    fn unexposed_declaration_does_not_poison_retained_rule() {
        let sheet = parse(".clamp{-webkit-line-clamp:2;line-clamp:2;-webkit-box-orient:vertical}");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let declarations = sheet
            .TypedDeclarationsForProjectedRule(0)
            .expect("the exposed declarations remain authoritative");
        assert_eq!(
            declarations
                .iter()
                .map(|declaration| declaration.PropertyID())
                .collect::<Vec<_>>(),
            vec![
                CSSPropertyID::kWebkitLineClamp,
                CSSPropertyID::kWebkitBoxOrient,
            ]
        );
        assert!(sheet.RetainedDeclarationsForProjectedRule(0).is_some());
    }
    #[test]
    fn source_ranges_are_utf16_and_duplicate_media_retains_same_query_owner() {
        let sheet = parse("é😀{width:1px} @media screen{a{width:2px}}@media screen{b{width:3px}}");
        assert_eq!(sheet.rules[0].selector_source.Utf8(), "é😀");
        assert_eq!(sheet.rules[0].declaration_source.Utf8(), "width:1px");
        assert_eq!(sheet.rules[0].source_range.header_end, 3);
        assert!(Rc::ptr_eq(
            &sheet.rules[1].media[0],
            &sheet.rules[2].media[0]
        ));
    }
    #[test]
    fn container_rule_projects_typed_chain_and_exact_declarations() {
        let sheet = parse(
            "@container Card (100px < width <= 600px), style(--Theme: RED!important){@container scroll-state(stuck: top){.item{width : 17px ;--x:var(--a, [2px]);}}}",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].container_conditions.len(), 2);
        assert_eq!(
            sheet.rules[0].declaration_source.Utf8(),
            "width : 17px ;--x:var(--a, [2px]);"
        );
        let css = crate::production_style_sheet_projection::ProjectStyleSheet(&sheet);
        let chain = &css.rules[0].container_conditions;
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].queries.len(), 2);
        assert_eq!(chain[0].queries[0].name.Utf8(), "Card");
        let ::cssom::CSSContainerCondition::Nested(inner) =
            chain[0].queries[0].condition.as_ref().unwrap()
        else {
            panic!("size condition")
        };
        let ::cssom::CSSContainerCondition::Feature(feature) = inner.as_ref() else {
            panic!("typed feature")
        };
        assert_eq!(feature.name.Utf8(), "width");
        assert_eq!(feature.left.operator, ::cssom::CSSContainerOperator::Less);
        assert_eq!(
            feature.right.operator,
            ::cssom::CSSContainerOperator::LessEqual
        );
    }
    #[test]
    fn container_rule_keeps_media_math_as_typed_css_value() {
        let sheet = parse(
            "@container (width >= calc(10em + 640px)){.item{width:17px}}@container anchored(fallback: --card flip-block){.fallback{height:3px}}",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.rules.len(), 2);
        let condition = &sheet.rules[0].container_conditions[0];
        assert_eq!(condition.ToString().Utf8(), "(width >= calc(10em + 640px))");
        assert_eq!(
            sheet.rules[1].container_conditions[0].ToString().Utf8(),
            "anchored(fallback: --card flip-block)"
        );
    }
    #[test]
    fn container_style_range_tokens_roundtrip_and_bad_prelude_recovers() {
        let sheet = parse(
            "@container style(1px < var(--Limit, calc(3px > 2px)) < 8px){a{color:red}}@container none{bad{}}tail{width:2px}",
        );
        assert_eq!(sheet.rules.len(), 2);
        assert!(sheet.rules[1].container_conditions.is_empty());
        let css = crate::production_style_sheet_projection::ProjectStyleSheet(&sheet);
        let ::cssom::CSSContainerCondition::Function(::cssom::CSSContainerFunction::Style, inner) =
            css.rules[0].container_conditions[0].queries[0]
                .condition
                .as_ref()
                .unwrap()
        else {
            panic!("style function")
        };
        let ::cssom::CSSContainerCondition::Feature(feature) = inner.as_ref() else {
            panic!("style range")
        };
        let data = feature.reference.as_ref().unwrap();
        let restored = crate::production_container_projection::RestoreContainerVariableData(data);
        assert_eq!(
            restored.original_text.Utf8(),
            "var(--Limit, calc(3px > 2px))"
        );
        assert_eq!(restored.tokens.len(), data.tokens.len());
        assert!(restored.NeedsVariableResolution());
        assert_eq!(
            crate::production_container_projection::ProjectContainerVariableData(&restored),
            *data
        );
    }
    #[test]
    fn scopes_keep_native_boundaries_and_nested_scope_chains() {
        let sheet = parse(
            "@scope (.root, #other) to (.stop){.item{width:1px}@scope to (.limit){.deep{height:2px}}}@scope{.implicit{color:red}}@scope (a::before){bad{}}tail{width:3px}",
        );
        assert_eq!(sheet.scopes.len(), 3);
        assert_eq!(sheet.rules.len(), 4);
        assert_eq!(sheet.rules[1].scope_conditions.len(), 2);
        assert!(sheet.rules[3].scope_conditions.is_empty());
        let projected = crate::production_style_sheet_projection::ProjectStyleSheet(&sheet);
        assert_eq!(
            projected.rules[0].scope_conditions[0].root,
            ::cssom::CSSScopeRoot::ExplicitSelectorList(".root, #other".into())
        );
        assert_eq!(
            projected.rules[0].scope_conditions[0].limit,
            Some(".stop".into())
        );
        assert_eq!(
            projected
                .rules
                .iter()
                .find(|rule| rule.declaration_text == "height:2px")
                .unwrap()
                .scope_conditions[1]
                .root,
            ::cssom::CSSScopeRoot::ImplicitStylesheetOwner
        );
        assert_eq!(
            projected
                .rules
                .iter()
                .find(|rule| rule.declaration_text == "height:2px")
                .unwrap()
                .scope_conditions[1]
                .limit,
            Some(".limit".into())
        );
        assert_eq!(sheet.rules[0].declaration_source.Utf8(), "width:1px");
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
    }
    #[test]
    fn property_registrations_validate_schema_initial_and_keep_descriptor_source() {
        let body = "syntax: '<length>+ | auto'; inherits: false; initial-value: 001px 2vw;";
        let sheet = parse(&format!(
            "@media screen{{@layer settings{{@property --size{{{body}}}}}}}@property --any{{syntax:'*';inherits:true}}@property --bad{{syntax:'<length>';inherits:false;initial-value:1em}}@property --vars{{syntax:'*';inherits:true;initial-value:var(--x)}}@property --wide{{syntax:'*';inherits:true;initial-value:initial}}@property --missing{{syntax:'<integer>';inherits:false}}tail{{width:3px}}"
        ));
        assert_eq!(sheet.property_registration_effects.len(), 2);
        assert_eq!(sheet.rules.len(), 1);
        let effect = &sheet.property_registration_effects[0];
        assert_eq!(effect.name.Utf8(), "--size");
        assert!(!effect.inherits);
        assert_eq!(effect.syntax.Components().len(), 2);
        let CSSValuePayload::kValueListClass(list) = effect.initial.as_ref().unwrap().Payload()
        else {
            panic!("typed repeated initial")
        };
        assert_eq!(list.values.len(), 2);
        assert!(sheet.property_registration_effects[1].initial.is_none());
        let projected = crate::production_style_sheet_projection::ProjectStyleSheet(&sheet);
        assert_eq!(projected.property_rules.len(), 2);
        assert_eq!(projected.property_rules[0].declaration_text, body);
        assert_eq!(
            projected.property_rules[0].initial_value_text,
            Some("001px 2vw".into())
        );
        assert_eq!(projected.property_rules[0].media_conditions, vec!["screen"]);
        assert_eq!(projected.property_rules[0].layer_name, "settings");
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Invalid));
    }
    #[test]
    fn property_descriptor_syntax_and_unsupported_initial_recover() {
        let sheet = parse(
            "@property --bad{syntax:'<length> +';inherits:true;initial-value:1px}@property --important{syntax:'*';inherits:false!important;initial-value:x}@property --math{syntax:'<number>';inherits:false;initial-value:calc(1 + 2)}@property --good{syntax:'<color>';inherits:false;initial-value:#1234}a{height:4px}",
        );
        assert_eq!(sheet.property_registration_effects.len(), 1);
        assert_eq!(sheet.property_registration_effects[0].name.Utf8(), "--good");
        assert_eq!(sheet.rules.len(), 1);
        assert!(sheet
            .diagnostics
            .iter()
            .any(|d| d.kind == DiagnosticKind::Unsupported));
        let projected = crate::production_style_sheet_projection::ProjectStyleSheet(&sheet);
        assert!(matches!(
            projected.property_rules[0].initial_value,
            Some(::cssom::CSSRegisteredPropertyValue::Color { .. })
        ));
    }
    #[test]
    fn production_rule_set_ingests_buckets_media_layers_and_scopes() {
        let sheet = parse(
            "@layer reset,theme;\
             #plain{width:1px}\
             @media screen and (min-width:600px){@layer theme{.wide{height:2px}}}\
             @media print{.print-only{height:3px}}\
             @scope (.root) to (.stop){article{padding-left:4px}}",
        );
        let medium = ProductionMediaQueryEvaluator::Screen(800.0, 600.0);
        let set = sheet.BuildRuleSet(&medium);

        // RuleCount also includes the @scope boundary selectors retained for
        // source-ordered scope bucketing.
        assert_eq!(set.RuleCount(), 5);
        assert_eq!(set.IdRules(&AtomicString::from_str("plain")).len(), 1);
        assert_eq!(set.ClassRules(&AtomicString::from_str("wide")).len(), 1);
        assert!(set
            .ClassRules(&AtomicString::from_str("print-only"))
            .is_empty());
        assert_eq!(set.TagRules(&AtomicString::from_str("article")).len(), 1);

        let theme_rule = &set.ClassRules(&AtomicString::from_str("wide"))[0];
        let theme = set
            .CascadeLayers()
            .GetOrAddSubLayer(&[AtomicString::from_str("theme")]);
        assert_eq!(set.LayerForRule(theme_rule), Some(&theme));

        let article = &set.TagRules(&AtomicString::from_str("article"))[0];
        let scope = set.ScopeForRule(article).expect("scope interval");
        assert!(scope.From().is_some());
        assert!(scope.To().is_some());
    }
    #[test]
    fn production_rule_set_rebuilds_when_media_results_change() {
        let sheet = parse("@media (min-width:600px){.wide{width:1px}} .always{width:2px}");
        let narrow = ProductionMediaQueryEvaluator::Screen(500.0, 600.0);
        let wide = ProductionMediaQueryEvaluator::Screen(800.0, 600.0);
        let first = sheet.EnsureRuleSet(&narrow);
        assert!(first
            .borrow()
            .ClassRules(&AtomicString::from_str("wide"))
            .is_empty());
        let second = sheet.EnsureRuleSet(&wide);
        assert!(!Rc::ptr_eq(&first, &second));
        assert_eq!(
            second
                .borrow()
                .ClassRules(&AtomicString::from_str("wide"))
                .len(),
            1
        );
    }

    #[test]
    fn function_parser_retains_declarations_and_registry_identity() {
        let sheet = parse(
            "@function --size(--base type(<length>): 4px) returns type(<length>) {\
                 --local: 7px; result: var(--base);\
             }",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.functions.len(), 1);
        let base = sheet.functions[0].rule.clone();
        let StyleRuleBase::Function(function) = base.as_ref() else {
            panic!("retained rule must be @function")
        };
        assert_eq!(function.Name().Utf8(), "--size");
        assert_eq!(function.GetParameters().len(), 1);
        assert_eq!(function.GetParameters()[0].name.Utf8(), "--base");
        assert_eq!(
            function.GetParameters()[0]
                .default_value
                .as_ref()
                .unwrap()
                .original_text
                .Utf8(),
            "4px"
        );
        assert_eq!(function.GetReturnType().ToString().Utf8(), "<length>");
        let children = function.ChildRules();
        let declarations = children
            .iter()
            .find_map(|rule| match rule.as_ref() {
                StyleRuleBase::FunctionDeclarations(declarations) => Some(declarations.clone()),
                _ => None,
            })
            .expect("function declaration block");
        let locals = <Backend as RuleSetIngestionBackend>::FunctionDeclarationValues(&declarations);
        assert!(locals.iter().any(|(name, value)| {
            name.Utf8() == "--local" && value.original_text.Utf8() == "7px"
        }));

        let registry = sheet.FunctionRegistry(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0));
        let retained = registry
            .FunctionForName(&AtomicString::from_str("--size"))
            .expect("registered function");
        assert!(Rc::ptr_eq(retained.Base(), &base));
    }

    #[test]
    fn stylesheet_mixin_registry_retains_parser_rule_and_respects_media() {
        let sheet = parse(
            "@media (min-width:600px) { @mixin --card(--edge: 3px) {\
                 --local: 9px; color: red;\
             } }",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        assert_eq!(sheet.mixins.len(), 1);
        let name = AtomicString::from_str("--card");
        assert!(sheet
            .MixinForName(&ProductionMediaQueryEvaluator::Screen(500.0, 600.0), &name)
            .is_none());
        let active = sheet
            .MixinForName(&ProductionMediaQueryEvaluator::Screen(800.0, 600.0), &name)
            .expect("active mixin");
        assert!(Rc::ptr_eq(&active, &sheet.mixins[0].rule));
        let StyleRuleBase::Mixin(mixin) = active.as_ref() else {
            panic!("retained rule must be @mixin")
        };
        assert_eq!(mixin.GetParameters().len(), 1);
        assert_eq!(
            mixin.GetParameters()[0]
                .default_value
                .as_ref()
                .unwrap()
                .original_text
                .Utf8(),
            "3px"
        );
        assert!(mixin
            .ChildRules()
            .iter()
            .any(|rule| { matches!(rule.as_ref(), StyleRuleBase::FunctionDeclarations(_)) }));
    }

    #[test]
    fn document_function_registry_applies_sheet_and_layer_order() {
        let early = parse("@function --pick() { result: 1px; }");
        let later_layered = parse("@layer theme { @function --pick() { result: 2px; } }");
        let later_unlayered = parse("@function --pick() { result: 3px; }");
        let medium = ProductionMediaQueryEvaluator::Screen(800.0, 600.0);
        let registry = ProductionFunctionRegistry::FromStyleSheets(
            &[&early, &later_layered, &later_unlayered],
            &medium,
        );
        let selected = registry
            .FunctionForName(&AtomicString::from_str("--pick"))
            .expect("document function");
        assert!(Rc::ptr_eq(
            selected.Base(),
            &later_unlayered.functions[0].rule
        ));
    }

    #[test]
    fn apply_mixin_arguments_and_contents_keep_typed_fake_parent_owner() {
        let sheet = parse(
            "@mixin --box(--a, --b) {\
                 @result { width: var(--a); @contents; }\
             }\
             .target { @apply --box(4px, 5px){ color: red; } }",
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let target = sheet
            .contents
            .ChildRules()
            .iter()
            .find_map(|rule| match rule.as_ref() {
                StyleRuleBase::Style(style) if style.SelectorsText().Utf8().contains(".target") => {
                    Some(style.clone())
                }
                _ => None,
            })
            .expect("target style rule");
        let children = target.ChildRules().expect("nested @apply");
        let apply = children
            .iter()
            .find_map(|rule| match rule.as_ref() {
                StyleRuleBase::ApplyMixin(apply) => Some(apply),
                _ => None,
            })
            .expect("typed @apply rule");
        assert!(apply.HasContentsBlock());
        assert_eq!(apply.GetArguments().len(), 2);
        assert_eq!(
            apply.GetArguments()[0]
                .as_ref()
                .unwrap()
                .original_text
                .Utf8(),
            "4px"
        );
        assert_eq!(
            apply.GetArguments()[1]
                .as_ref()
                .unwrap()
                .original_text
                .Utf8(),
            "5px"
        );
        assert!(!apply.ChildRules().is_empty());

        let medium = ProductionMediaQueryEvaluator::Screen(800.0, 600.0);
        let set = sheet.BuildRuleSet(&medium);
        // cpp: style_rule.cc:604-624,642-821. The invoking rule plus the
        // recursively cloned @result/@contents declarations retain the
        // .target selector; every expansion clone carries the exact binding
        // owner into MatchResult.
        let target_rules = set.ClassRules(&AtomicString::from_str("target"));
        assert_eq!(target_rules.len(), 5);
        assert_eq!(
            target_rules
                .iter()
                .filter(|data| data.Rule().GetMixinParameterBindings().is_some())
                .count(),
            3
        );
    }
}

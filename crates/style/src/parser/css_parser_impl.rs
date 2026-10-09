// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source ledger: effective = nonblank source lines after lexical comment
// stripping, with braces counted, minus the precise exclusions below.
// css_parser_impl.h: physical 515; effective 300; mapped 300; pending 0.
// css_parser_impl.cc: physical 3651; effective 2771; mapped 2771; pending 0.
// Mapped .h:58-70,74-268,274-420,473-510. Required external grammar and
// constructor signatures count as mapped dependency boundaries.
// Mapped .cc:78-3649, excluding the debug-only lines listed below.
// Omitted .h:1-57,63-66,71-73,269-273,511-515: include/forward/class
// boilerplate, STACK_ALLOCATED, deleted copy operations, friend and wrappers.
// Omitted .cc:1-77,3650-3651: includes/using and namespace wrappers.
// Omitted .cc debug-only:102-104,116,622,787,865,1278,1352,1406,
// 1956,3008,3082,3154,3158,3189,3209,3290,3430.
// Production logic pending in this source pair: 0.
// The new rule consumers use actual CSSSelectorList, StyleRule/StyleRuleBase,
// CSSPropertyValueSetRuleHandle and StyleSheetContents types. External grammar
// bodies, class constructors and GC-edge adaptation remain mandatory typed operations.
// Base backend dispatch assembly must forward to the mapped RuleBackend
// consumers; no backend, accepting default, empty placeholder rule or secondary
// property store is installed here.

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use super::allowed_rules::{AllowedRules, QualifiedRuleType};
use super::css_at_rule_id::{AtRuleRuntimeFeatures, CSSAtRuleID, CssAtRuleID};
use super::css_nesting_type::CSSNestingType;
use super::css_parser_context::{CSSParserContext, CSSParserContextPlatform};
use super::css_parser_context::{DocumentHandle, SecureContextMode};
use super::css_parser_mode::{CSSDeferPropertyParsing, CSSParserMode};
use super::css_parser_token::CSSParserTokenType::*;
use super::css_parser_token_stream::{
    BlockGuard, Boundary, CSSParserTokenStream, RestoringBlockGuard, TokenStreamTokenizer,
};
use super::css_tokenizer::CSSTokenizer;
use crate::css_property_value::{CSSPropertyValue, CSSPropertyValueBackend};
use crate::css_property_value_set::{
    CSSPropertyValueSet, CSSPropertyValueSetBackend, CSSPropertyValueSetRuleHandle,
    ImmutableCSSPropertyValueSet, MutableCSSPropertyValueSet, SetResult,
};
use crate::css_selector::{
    CSSSelector, CSSSelectorParserContext, MatchType, PseudoType, QualifiedName,
};
use crate::css_selector_list::CSSSelectorList;
use crate::css_value::CSSValuePayload;
use crate::style_rule::RuleType;
use crate::style_rule_keyframe::KeyframeOffset;
use foundation::{AtomicString, CSSPropertyID, CSSValueID, Member, String, StringView};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// cpp: css_parser_impl.cc:156-180
pub fn RuleTypeForParserMode(mode: CSSParserMode) -> crate::style_rule::RuleType {
    use crate::style_rule::RuleType;
    match mode {
        CSSParserMode::kCSSFontFaceRuleMode => RuleType::kFontFace,
        CSSParserMode::kCSSKeyframeRuleMode => RuleType::kKeyframe,
        CSSParserMode::kCSSPropertyRuleMode => RuleType::kProperty,
        CSSParserMode::kCSSFontPaletteValuesRuleMode => RuleType::kFontPaletteValues,
        CSSParserMode::kCSSPositionTryRuleMode => RuleType::kPositionTry,
        CSSParserMode::kCSSFunctionDescriptorsMode => RuleType::kFunction,
        CSSParserMode::kCSSCounterStyleRuleMode => RuleType::kCounterStyle,
        _ => RuleType::kStyle,
    }
}

// cpp: css_parser_impl.cc:182-198
pub fn ToStyleRuleFontFeatureType(
    rule_id: CSSAtRuleID,
) -> Option<crate::style_rule_font_feature_values::FontFeatureType> {
    use crate::style_rule_font_feature_values::FontFeatureType;
    match rule_id {
        CSSAtRuleID::kCSSAtRuleStylistic => Some(FontFeatureType::kStylistic),
        CSSAtRuleID::kCSSAtRuleStyleset => Some(FontFeatureType::kStyleset),
        CSSAtRuleID::kCSSAtRuleCharacterVariant => Some(FontFeatureType::kCharacterVariant),
        CSSAtRuleID::kCSSAtRuleSwash => Some(FontFeatureType::kSwash),
        CSSAtRuleID::kCSSAtRuleOrnaments => Some(FontFeatureType::kOrnaments),
        CSSAtRuleID::kCSSAtRuleAnnotation => Some(FontFeatureType::kAnnotation),
        _ => None,
    }
}

// Required hook for the CSS URL request-modifier grammar. The implementation
// lives with css_parsing_utils, exactly as the C++ helper delegates there; this
// parser helper neither accepts nor discards modifiers on its own.
pub trait CSSUrlRequestModifiersConsumer<C> {
    type Modifiers;
    fn CSSURLRequestModifiersEnabled() -> bool;
    fn ConsumeUrlRequestModifiers<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &C,
        modifiers: &mut Self::Modifiers,
    ) -> bool;
}

fn EqualIgnoringAsciiCase(view: &foundation::StringView, ascii: &[u8]) -> bool {
    fn lower(unit: u16) -> u16 {
        if (b'A' as u16..=b'Z' as u16).contains(&unit) {
            unit + (b'a' - b'A') as u16
        } else {
            unit
        }
    }
    view.length() as usize == ascii.len()
        && view
            .Span16()
            .iter()
            .zip(ascii)
            .all(|(left, right)| lower(*left) == lower(*right as u16))
}

// cpp: css_parser_impl.cc:78-123
// This may consume tokens when it fails, matching the source contract.
pub fn ConsumeStringOrURI<C, M, T>(
    stream: &mut CSSParserTokenStream<'_, T>,
    context: &C,
    mut modifiers: Option<&mut M::Modifiers>,
) -> AtomicString
where
    M: CSSUrlRequestModifiersConsumer<C>,
    T: TokenStreamTokenizer,
{
    let token = stream.Peek();
    if token.GetType() == kStringToken || token.GetType() == kUrlToken {
        return AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
    }
    if token.GetType() != kFunctionToken || !EqualIgnoringAsciiCase(&token.Value(), b"url") {
        return AtomicString::default();
    }

    let mut result = AtomicString::default();
    {
        let mut guard = BlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let uri = guard.ConsumeIncludingWhitespace();
        if uri.GetType() != kBadStringToken {
            let should_consume_modifiers =
                M::CSSURLRequestModifiersEnabled() && modifiers.is_some();
            let consumed_modifiers = should_consume_modifiers
                && M::ConsumeUrlRequestModifiers(
                    &mut guard,
                    context,
                    modifiers.as_deref_mut().unwrap(),
                );
            if (!should_consume_modifiers || consumed_modifiers) && guard.UncheckedAtEnd() {
                debug_assert_eq!(uri.GetType(), kStringToken);
                result = AtomicString::from_utf16(uri.Value().Span16());
            }
        }
    }
    stream.ConsumeWhitespace();
    result
}

// cpp: css_parser_impl.h:58-61
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseSheetResult {
    kSucceeded,
    kHasUnallowedImportRule,
}

// cpp: css_parser_impl.h:74-98
pub const kRegularRules: AllowedRules =
    AllowedRules::FromQualifiedRules(&[QualifiedRuleType::kStyle]).Union(
        AllowedRules::FromAtRules(&[
            CSSAtRuleID::kCSSAtRuleViewTransition,
            CSSAtRuleID::kCSSAtRuleFontFace,
            CSSAtRuleID::kCSSAtRuleFontPaletteValues,
            CSSAtRuleID::kCSSAtRuleKeyframes,
            CSSAtRuleID::kCSSAtRuleLayer,
            CSSAtRuleID::kCSSAtRuleMedia,
            CSSAtRuleID::kCSSAtRulePage,
            CSSAtRuleID::kCSSAtRulePositionTry,
            CSSAtRuleID::kCSSAtRuleProperty,
            CSSAtRuleID::kCSSAtRuleLocation,
            CSSAtRuleID::kCSSAtRuleNavigation,
            CSSAtRuleID::kCSSAtRuleContainer,
            CSSAtRuleID::kCSSAtRuleCounterStyle,
            CSSAtRuleID::kCSSAtRuleScope,
            CSSAtRuleID::kCSSAtRuleStartingStyle,
            CSSAtRuleID::kCSSAtRuleSupports,
            CSSAtRuleID::kCSSAtRuleWebkitKeyframes,
            CSSAtRuleID::kCSSAtRuleFontFeatureValues,
            CSSAtRuleID::kCSSAtRuleFunction,
            CSSAtRuleID::kCSSAtRuleMixin,
            CSSAtRuleID::kCSSAtRuleCustomMedia,
        ]),
    );
// cpp: css_parser_impl.h:102-107
pub const kTopLevelRules: AllowedRules = kRegularRules.Union(AllowedRules::FromAtRules(&[
    CSSAtRuleID::kCSSAtRuleCharset,
    CSSAtRuleID::kCSSAtRuleImport,
    CSSAtRuleID::kCSSAtRuleNamespace,
]));
// cpp: css_parser_impl.h:110
pub const kKeyframeRules: AllowedRules =
    AllowedRules::FromQualifiedRules(&[QualifiedRuleType::kKeyframe]);
// cpp: css_parser_impl.h:113-120
pub const kFontFeatureRules: AllowedRules = AllowedRules::FromAtRules(&[
    CSSAtRuleID::kCSSAtRuleAnnotation,
    CSSAtRuleID::kCSSAtRuleCharacterVariant,
    CSSAtRuleID::kCSSAtRuleOrnaments,
    CSSAtRuleID::kCSSAtRuleStylistic,
    CSSAtRuleID::kCSSAtRuleStyleset,
    CSSAtRuleID::kCSSAtRuleSwash,
]);
// cpp: css_parser_impl.h:123-140
pub const kPageMarginRules: AllowedRules = AllowedRules::FromAtRules(&[
    CSSAtRuleID::kCSSAtRuleTopLeftCorner,
    CSSAtRuleID::kCSSAtRuleTopLeft,
    CSSAtRuleID::kCSSAtRuleTopCenter,
    CSSAtRuleID::kCSSAtRuleTopRight,
    CSSAtRuleID::kCSSAtRuleTopRightCorner,
    CSSAtRuleID::kCSSAtRuleBottomLeftCorner,
    CSSAtRuleID::kCSSAtRuleBottomLeft,
    CSSAtRuleID::kCSSAtRuleBottomCenter,
    CSSAtRuleID::kCSSAtRuleBottomRight,
    CSSAtRuleID::kCSSAtRuleBottomRightCorner,
    CSSAtRuleID::kCSSAtRuleLeftTop,
    CSSAtRuleID::kCSSAtRuleLeftMiddle,
    CSSAtRuleID::kCSSAtRuleLeftBottom,
    CSSAtRuleID::kCSSAtRuleRightTop,
    CSSAtRuleID::kCSSAtRuleRightMiddle,
    CSSAtRuleID::kCSSAtRuleRightBottom,
]);
// cpp: css_parser_impl.h:147-152
pub const kConditionalRules: AllowedRules = AllowedRules::FromAtRules(&[
    CSSAtRuleID::kCSSAtRuleMedia,
    CSSAtRuleID::kCSSAtRuleSupports,
    CSSAtRuleID::kCSSAtRuleContainer,
    CSSAtRuleID::kCSSAtRuleNavigation,
]);
// cpp: css_parser_impl.h:159-166
pub const kNestedGroupRules: AllowedRules = kConditionalRules.Union(AllowedRules::FromAtRules(&[
    CSSAtRuleID::kCSSAtRuleLayer,
    CSSAtRuleID::kCSSAtRuleScope,
    CSSAtRuleID::kCSSAtRuleStartingStyle,
    CSSAtRuleID::kCSSAtRuleViewTransition,
    CSSAtRuleID::kCSSAtRuleApplyMixin,
]));

// cpp: css_parser_impl.h:169-179
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeOffset {
    pub start: u32,
    pub end: u32,
}
impl RangeOffset {
    pub fn new(start: u32, end: u32) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }
    pub const fn Ignore() -> Self {
        Self { start: 0, end: 0 }
    }
}

// Only the four actual StyleRuleBase queries read by ComputeNewAllowedRules.
// Implemented by the real rule type; there is no default rule classification.
pub trait CSSRuleOrdering {
    fn IsCharsetRule(&self) -> bool;
    fn IsLayerStatementRule(&self) -> bool;
    fn IsImportRule(&self) -> bool;
    fn IsNamespaceRule(&self) -> bool;
}
// Source observer calls in .cc:809,830,847.
pub trait CSSParserImplObserver {
    fn ObserveErroneousAtRule(&mut self, offset: u32, id: CSSAtRuleID);
}

// Required dependencies of the partially translated parser. Associated types
// denote actual missing Chromium types, never substitute empty CSS enums.
// The rule dispatch operations below are assembly boundaries: a production
// RuleBackend forwards them to the mapped methods on CSSParserImpl. Declaration
// consumers are native methods below; grammar dependencies have no defaults.
pub trait CSSParserImplBackend: Sized {
    type Platform: CSSParserContextPlatform;
    type ValueBackend: CSSPropertyValueSetBackend;
    type StyleSheetContents;
    type CSSParserObserver: CSSParserImplObserver;
    type CSSLazyParsingState;
    type MediaQuerySet;
    type StyleRule;
    type StyleRuleBase: CSSRuleOrdering;
    fn AtRuleFeatures() -> AtRuleRuntimeFeatures;
    fn ConsumeAtRuleContents<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        id: CSSAtRuleID,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed_rules: AllowedRules,
        nesting_type: CSSNestingType,
        parent_rule_for_nesting: Option<Rc<Self::StyleRule>>,
    ) -> Option<Rc<Self::StyleRuleBase>>;
    fn ConsumeQualifiedRule<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed_rules: AllowedRules,
        nesting_type: CSSNestingType,
        parent_rule_for_nesting: Option<Rc<Self::StyleRule>>,
    ) -> Option<Rc<Self::StyleRuleBase>>;
}

// Required external property/variable/descriptor grammar operations. All
// declaration and block consumers below execute their native mapped bodies.
pub trait CSSParserImplValueBackend: CSSParserImplBackend {
    type CSSVariableData;
    fn ParseAsUnresolvedCSSPropertyID(
        token: &super::css_parser_token::CSSParserToken,
        context: &CSSParserContext<Self::Platform>,
    ) -> CSSPropertyID;
    fn ParseDescriptorValue<T: TokenStreamTokenizer>(
        rule: RuleType,
        id: super::at_rule_descriptors::AtRuleDescriptorID,
        variable: &AtomicString,
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        properties: &mut Vec<CSSPropertyValue<Self::ValueBackend>>,
    );
    fn CSSPropertyParserParseValue<T: TokenStreamTokenizer>(
        property: CSSPropertyID,
        allow_important: bool,
        stream: &mut CSSParserTokenStream<'_, T>,
        context: Option<&CSSParserContext<Self::Platform>>,
        properties: &mut Vec<CSSPropertyValue<Self::ValueBackend>>,
        rule: RuleType,
    );
    fn ConsumeCSSWideKeyword<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        allow_important: bool,
        important: &mut bool,
    ) -> Option<Rc<crate::css_value::CSSValue<Self::ValueBackend>>>;
    fn ConsumeVariableParserUnparsedDeclaration<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        allow_important: bool,
        animation_tainted: bool,
        must_contain_variable_reference: bool,
        restricted_value: bool,
        comma_ends_declaration: bool,
        important: &mut bool,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Rc<Self::CSSVariableData>>;
    fn NewCSSUnparsedDeclarationValue(
        data: Rc<Self::CSSVariableData>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<crate::css_value::CSSValue<Self::ValueBackend>>;
    fn VisitedColumnRuleColorFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature;

    fn IdentifierWasQuirky(
        value: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSIdentifierValue,
    ) -> bool;
    fn IdentifierValueID(
        value: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSIdentifierValue,
    ) -> CSSValueID;
    fn QuirksModeCursorHandFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature;
}
pub trait CSSParserImplPageBackend: CSSParserImplBackend {
    type SelectorParserContext: CSSSelectorParserContext;
    fn SelectorParserContext(
        document: Option<DocumentHandle<Self::Platform>>,
    ) -> Self::SelectorParserContext;
}
// External timeline grammar and typed list conversion; key-list body is local.
pub trait CSSParserImplKeyframeBackend: CSSParserImplBackend {
    fn ConsumeTimelineRangeNameAndPercent<T: TokenStreamTokenizer>(
        context: Option<&CSSParserContext<Self::Platform>>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSValueList>>;
    fn TimelineRangeListName(
        list: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSValueList,
    ) -> crate::style_rule_keyframe::TimelineNamedRange;
    fn TimelineRangeListClampedPercent(
        list: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSValueList,
    ) -> f64;
}
pub trait CSSParserImplInlineElementBackend: CSSParserImplValueBackend {
    type Element;
    fn ElementDocument(element: &Self::Element) -> DocumentHandle<Self::Platform>;
    fn ElementSheetParserContext(element: &Self::Element) -> Rc<CSSParserContext<Self::Platform>>;
    fn IsHTMLElement(element: &Self::Element) -> bool;
    fn ElementDocumentInQuirksMode(element: &Self::Element) -> bool;
    fn ElementSheetContents(element: &mut Self::Element) -> &mut Self::StyleSheetContents;
}

// Source-owned sheet/platform operations, including trace/timer calls. Rule
// appends execute inside the rule-list callback before parsing the next rule.
pub trait CSSParserImplSheetBackend: CSSParserImplBackend {
    type ParseStyleSheetTimer;
    type DocumentView;
    type UkmAggregator;
    type TextPosition;
    fn DocumentView(document: &DocumentHandle<Self::Platform>) -> Option<Rc<Self::DocumentView>>;
    fn ViewUkmAggregator(view: &Self::DocumentView) -> Option<Rc<Self::UkmAggregator>>;
    fn GetScopedParseStyleSheetTimer(
        aggregator: &Self::UkmAggregator,
    ) -> Self::ParseStyleSheetTimer;
    fn TraceBeginStyleSheet(
        base_url: &<Self::Platform as CSSParserContextPlatform>::URL,
        mode: CSSParserMode,
    );
    fn TraceBeginStyleSheetParse();
    fn TraceEndStyleSheetParse();
    fn TraceEndStyleSheet(token_count: u32, length: u32);
    fn NewLazyParsingState(
        context: &CSSParserContext<Self::Platform>,
        text: &String,
        sheet: &mut Self::StyleSheetContents,
    ) -> Self::CSSLazyParsingState;
    fn AnyOwnerDocument(sheet: &Self::StyleSheetContents)
        -> Option<DocumentHandle<Self::Platform>>;
    fn MinimumTextPosition() -> Self::TextPosition;
    fn GetTextPosition(
        document: &DocumentHandle<Self::Platform>,
        offset: u32,
        text: &String,
        position: &mut Self::TextPosition,
    );
    fn SetImportPositionHint(rule: &Self::StyleRuleBase, position: Self::TextPosition);
    fn ParserAppendRule(sheet: &mut Self::StyleSheetContents, rule: Rc<Self::StyleRuleBase>);
    fn SetHasSyntacticallyValidCSSHeader(sheet: &mut Self::StyleSheetContents, valid: bool);
}

// cpp: css_parser_impl.h:287-376,389-393. The source classes already
// translated in style_rule.rs use their actual selector/property store here.
// Only missing external grammar/class construction operations are required.
// Backend assembly forwards its two base dispatch hooks to these consumers;
// neither dispatch hook is a fallback parser or an accepting default.
pub trait CSSParserImplRuleBackend:
    CSSParserImplValueBackend
    + CSSParserImplPageBackend
    + CSSParserImplKeyframeBackend
    + CSSUrlRequestModifiersConsumer<CSSParserContext<Self::Platform>>
    + CSSParserImplBackend<
        StyleRule = crate::style_rule::StyleRule<Self::RuleDependencies>,
        StyleRuleBase = crate::style_rule::StyleRuleBase<Self::RuleDependencies>,
        StyleSheetContents = Rc<
            crate::style_sheet_contents::StyleSheetContents<Self::RuleDependencies>,
        >,
    >
{
    type RuleDependencies: crate::style_sheet_contents::StyleSheetContentsBackend<
        SelectorList = CSSSelectorList,
        CSSPropertyValueSet = CSSPropertyValueSetRuleHandle<Self::ValueBackend>,
        MediaQuerySet = Self::MediaQuerySet,
        StyleRuleFontFeature = crate::style_rule_font_feature_values::StyleRuleFontFeature,
        StyleRuleFontFeatureValues = crate::style_rule_font_feature_values::StyleRuleFontFeatureValues,
    >;
    fn CountAtRule(context: &CSSParserContext<Self::Platform>, id: CSSAtRuleID);
    fn CascadeLayersFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature;
    fn QuotedKeyframesFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature;
    fn NewUrlRequestModifiers() -> Self::Modifiers;
    fn CSSSupportsForImportRulesEnabled() -> bool;
    fn CSSScopeImportEnabled() -> bool;
    fn CSSRevertRuleEnabled() -> bool;
    fn ConsumeSupportsCondition<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> SupportsResult;
    fn ConsumeStyleScope<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope>>;
    fn ParseMediaQuerySetString(
        text: String,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<Self::MediaQuerySet>;
    fn ParseMediaQuerySet<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<Self::MediaQuerySet>;
    // Required GC-edge adaptation: the returned Rc must retain the same query
    // allocation, and cached edges must remain live for the parser lifetime.
    fn CachedMediaQuery(edge: Member<Self::MediaQuerySet>) -> Option<Rc<Self::MediaQuerySet>>;
    fn MediaQueryCacheEdge(query: &Rc<Self::MediaQuerySet>) -> Member<Self::MediaQuerySet>;
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
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleImport>;
    fn NewNamespaceRule(
        prefix: AtomicString,
        uri: AtomicString,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleNamespace>;
    fn NewKeyframeRule(
        keys: Vec<KeyframeOffset>,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleKeyframe>;
    fn NewKeyframesRule(
        name: String,
        vendor_prefixed: bool,
        keys: Vec<Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleKeyframe>>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleKeyframes>;
    fn NewNestedDeclarationsRule(nesting: CSSNestingType, inner: crate::style_rule::StyleRule<Self::RuleDependencies>)
        -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleNestedDeclarations>;
    fn NewFunctionDeclarationsRule(properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>)
        -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleFunctionDeclarations>;

    // Rust ownership adapter for the C++ Oilpan-allocated fake parent used by
    // @mixin/@apply/@contents/@result. Nested selectors may retain this exact
    // allocation after parsing, so passing only `&mut StyleRule` would let the
    // owner die when this function returns.
    fn ConsumeMixinRuleListOrNestedDeclarationList<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Rc<std::cell::RefCell<Self::StyleRule>>,
        rules: &mut Vec<Rc<Self::StyleRuleBase>>,
    );
    fn ConsumeFontFamily<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Vec<Rc<crate::css_value::CSSValue<Self::ValueBackend>>>>;
    fn FontFamilyValue(
        value: &<Self::ValueBackend as crate::css_value::CSSValueDispatch>::CSSFontFamilyValue,
    ) -> AtomicString;
    fn ConsumeNonNegativeIntegerOrNumberCalc<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Rc<crate::css_value::CSSValue<Self::ValueBackend>>>;
    // DynamicTo<CSSPrimitiveValue> and GetValueIfKnown: nonprimitive and
    // unresolved math values both fail rather than inventing numeric aliases.
    fn PrimitiveNumberValueIfKnown(
        value: &crate::css_value::CSSValue<Self::ValueBackend>,
    ) -> Option<f64>;

    // Missing owners: CSSSelectorParser, CSSVariableParser, fast declaration
    // scanner and CSSLazyParsingState. Ranges select the actual shared arena.
    fn StartsCustomPropertyDeclaration<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool;
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
    ) -> std::ops::Range<usize>;
    fn HasAVX2AndPCLMUL() -> bool;
    fn FindLengthOfDeclarationList(text: &StringView) -> usize;
    fn FindLengthOfDeclarationListAVX2(text: &StringView) -> usize;
    // Must retain the same Oilpan/Rc allocation, never clone lazy state data.
    fn LazyParsingStateHandle(
        state: &Self::CSSLazyParsingState,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::CSSLazyParsingState>;
    fn CSSNestingFeature() -> <Self::Platform as CSSParserContextPlatform>::WebFeature;
    // External grammars and class constructors from .cc:1886-3001. These are
    // required operations on the actual rule/query/value types, without defaults.
    fn PropertyRegistrationConvertSyntax(
        value: Option<&crate::css_value::CSSValue<Self::ValueBackend>>,
    ) -> Option<ParserSyntax<Self>>;
    fn PropertyRegistrationConvertInherits(
        value: Option<&crate::css_value::CSSValue<Self::ValueBackend>>,
    ) -> Option<bool>;
    // The outer Option denotes conversion failure; an inner None is the
    // source's valid null initial value for universal syntax. The local parser
    // context uses this custom property name and kInvalid, rejecting random().
    fn PropertyRegistrationConvertInitial(
        value: Option<&crate::css_value::CSSValue<Self::ValueBackend>>,
        syntax: &ParserSyntax<Self>,
        context: &CSSParserContext<Self::Platform>,
        name: &String,
    ) -> Option<Option<Rc<crate::css_value::CSSValue<Self::ValueBackend>>>>;
    fn ConsumeCounterStyleNameInPrelude<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> AtomicString;
    fn NewCounterStyleRule(
        name: AtomicString,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<
        <Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleCounterStyle,
    >;
    fn NewFontPaletteValuesRule(name: AtomicString, properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>)
        -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleFontPaletteValues>;
    fn NewPositionTryRule(
        name: AtomicString,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<
        <Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRulePositionTry,
    >;
    fn NewLocationRule(
        name: AtomicString,
        properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleLocation>;
    fn NewViewTransitionRule(properties: Rc<CSSPropertyValueSet<Self::ValueBackend>>)
        -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleRuleViewTransition>;
    fn ParseNavigationQuery<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<
        Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::NavigationQuery>,
    >;
    fn ParseContainerQuerySet<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<
        Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::ContainerQuerySet>,
    >;
    fn ConsumeStyleScopeForRule<T: TokenStreamTokenizer>(
        parser: &mut CSSParserImpl<'_, Self>,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<Self::StyleRule>>,
    ) -> Option<Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope>>;
    fn CreateImplicitStyleScope(
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope>;
    // Required adaptation of the scope-owned parent edge. Must preserve its
    // allocation/selector identity while returning the nesting rule.
    fn ScopeRuleForNesting(
        scope: &mut Rc<
            <Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::StyleScope,
        >,
    ) -> Option<Rc<Self::StyleRule>>;
    fn ConsumeSyntaxDefinition<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<ParserSyntax<Self>>;
    fn ConsumeSyntaxComponent<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<ParserSyntax<Self>>;
    fn CreateUniversalSyntax() -> ParserSyntax<Self>;
    fn SyntaxIsUniversal(syntax: &ParserSyntax<Self>) -> bool;
    fn VariableDataNeedsVariableResolution(data: &ParserVariableData<Self>) -> bool;
    // CSSParserLocalContext::CreateWithoutPropertyForSyntaxParsing(), with both
    // animation and attr taint false, on data.OriginalText().
    fn ParseSyntaxDefaultValue(
        syntax: &ParserSyntax<Self>,
        data: &ParserVariableData<Self>,
        context: &CSSParserContext<Self::Platform>,
    ) -> bool;
    // ConsumeUnparsedDeclaration: allow-important, animation-tainted,
    // must-contain-variable-reference and restricted-value are all false.
    fn ConsumeUnparsedDeclaration<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        comma_ends_declaration: bool,
    ) -> Option<Rc<ParserVariableData<Self>>>;
    fn NewPrivateVariable(
        name: AtomicString,
        syntax: ParserSyntax<Self>,
        default_value: Option<Rc<ParserVariableData<Self>>>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Rc<<Self::RuleDependencies as crate::style_rule::StyleRuleDependencies>::CSSPrivateVariable>;
    fn ConsumeMixinArguments<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
        arguments: &mut Vec<Option<Rc<ParserVariableData<Self>>>>,
    ) -> bool;
    fn ParseCustomMediaDefinition<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &CSSParserContext<Self::Platform>,
    ) -> Option<Rc<Self::MediaQuerySet>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupportsResult {
    kSupported,
    kUnsupported,
    kParseFailure,
}
type ParserSyntax<B> = <<B as CSSParserImplRuleBackend>::RuleDependencies as crate::style_rule::StyleRuleDependencies>::CSSSyntaxDefinition;
type ParserVariableData<B> = <<B as CSSParserImplRuleBackend>::RuleDependencies as crate::style_rule::StyleRuleDependencies>::CSSVariableData;

// Additional source observer operations. No defaults: inspecting a rule must
// record the same header/body/property/nested-rule events as Chromium.
pub trait CSSParserImplRuleObserver: CSSParserImplObserver {
    fn ObserveComment(&mut self, start: u32, end: u32);
    fn StartRuleHeader(&mut self, rule_type: RuleType, offset: u32);
    fn EndRuleHeader(&mut self, offset: u32);
    fn StartRuleBody(&mut self, offset: u32);
    fn EndRuleBody(&mut self, offset: u32);
    // .cc:1943-1959: failed registration descriptors accompany the error.
    fn ObserveErroneousAtRuleWithProperties(
        &mut self,
        offset: u32,
        id: CSSAtRuleID,
        properties: &[CSSPropertyID],
    );
    fn ObserveNestedDeclarations(&mut self, index: usize);
    fn ObserveProperty(&mut self, start: u32, end: u32, important: bool, parsed: bool);
    fn ObserveFontFeatureType(
        &mut self,
        feature: crate::style_rule_font_feature_values::FontFeatureType,
    );
}

// cpp: css_parser_impl.cc:262-307. Stable insertion partition, followed by
// reverse-priority deduplication. usize::MAX reproduces unsigned wrap when
// all declarations are important; its value is never indexed in that case.
pub fn FilterProperties<D: CSSPropertyValueBackend>(
    values: &mut [CSSPropertyValue<D>],
    unused_entries: &mut usize,
    seen_properties: &mut HashSet<CSSPropertyID>,
    seen_custom_properties: &mut HashSet<AtomicString>,
) {
    let mut last_nonimportant_idx = values.len().wrapping_sub(1);
    for i in (0..values.len()).rev() {
        if values[i].IsImportant() {
            if i != last_nonimportant_idx {
                values[i..=last_nonimportant_idx].rotate_left(1);
            }
            last_nonimportant_idx = last_nonimportant_idx.wrapping_sub(1);
        }
    }
    for i in (0..values.len()).rev() {
        let property = &values[i];
        let inserted = if property.PropertyID() == CSSPropertyID::kVariable {
            seen_custom_properties.insert(property.CustomPropertyName().clone())
        } else {
            seen_properties.insert(property.PropertyID())
        };
        if inserted {
            *unused_entries -= 1;
            values[*unused_entries] = values[i].clone();
        }
    }
}

// cpp: css_parser_impl.cc:309-368
pub fn CreateCSSPropertyValueSet<B: CSSParserImplValueBackend>(
    properties: &mut Vec<CSSPropertyValue<B::ValueBackend>>,
    mode: CSSParserMode,
    document: Option<&DocumentHandle<B::Platform>>,
) -> Rc<CSSPropertyValueSet<B::ValueBackend>> {
    if mode != CSSParserMode::kHTMLQuirksMode
        && (properties.len() < 2
            || (properties.len() == 2 && properties[0].PropertyID() != properties[1].PropertyID()))
    {
        let result = ImmutableCSSPropertyValueSet::Create(properties, mode, false);
        properties.truncate(0);
        return result;
    }
    let mut seen_properties = HashSet::new();
    let mut seen_custom_properties = HashSet::new();
    let mut unused_entries = properties.len();
    FilterProperties(
        properties,
        &mut unused_entries,
        &mut seen_properties,
        &mut seen_custom_properties,
    );
    let mut count_cursor_hand = false;
    if let Some(document) = document {
        if mode == CSSParserMode::kHTMLQuirksMode
            && seen_properties.contains(&CSSPropertyID::kCursor)
        {
            let mut contains_cursor_hand = false;
            let mut contains_cursor_pointer = false;
            // The source intentionally examines the entire working vector,
            // including unused entries, for this compatibility use counter.
            for property in properties.iter() {
                if let CSSValuePayload::kIdentifierClass(value) = property.Value().Payload() {
                    if B::IdentifierWasQuirky(value) {
                        contains_cursor_hand = true;
                    } else if B::IdentifierValueID(value) == CSSValueID::kPointer {
                        contains_cursor_pointer = true;
                    }
                }
            }
            if contains_cursor_hand && !contains_cursor_pointer {
                document.CountUse(B::QuirksModeCursorHandFeature());
                count_cursor_hand = true;
            }
        }
    }
    let result = ImmutableCSSPropertyValueSet::Create(
        &properties[unused_entries..],
        mode,
        count_cursor_hand,
    );
    properties.truncate(0);
    result
}

// cpp: css_variable_parser.cc:31-43; css_parser_impl.cc:608-619
pub fn ParseCustomPropertyName(name_text: StringView) -> String {
    let mut stream: CSSParserTokenStream<CSSTokenizer> = CSSParserTokenStream::new(name_text, 0);
    let token = stream.Peek();
    if token.GetType() != kIdentToken {
        return String::default();
    }
    let value = token.Value();
    if value.length() < 3 || value.Span16()[0] != b'-' as u16 || value.Span16()[1] != b'-' as u16 {
        return String::default();
    }
    stream.ConsumeIncludingWhitespace();
    if !stream.AtEnd() {
        return String::default();
    }
    String::from_utf16(value.Span16())
}

// cpp: css_parser_impl.h:490-510
// Vec/Rust HashMap replace HeapVector/HeapHashMap storage while their element
// types come from the actual backend. Member retains the source non-owning GC
// edge for the query cache; shared rule results retain stable object identity.
pub struct CSSParserImpl<'a, B: CSSParserImplBackend> {
    pub(crate) parsed_properties_: Vec<CSSPropertyValue<B::ValueBackend>>,
    pub(crate) context_: Option<&'a CSSParserContext<B::Platform>>,
    pub(crate) style_sheet_: Option<&'a mut B::StyleSheetContents>,
    pub(crate) observer_: Option<&'a mut B::CSSParserObserver>,
    pub(crate) lazy_state_: Option<&'a mut B::CSSLazyParsingState>,
    pub(crate) arena_: Vec<CSSSelector>,
    pub(crate) in_nested_style_rule_: bool,
    pub(crate) in_mixin_: bool,
    pub(crate) media_query_cache_: HashMap<String, Member<B::MediaQuerySet>>,
}
impl<'a, B: CSSParserImplBackend> CSSParserImpl<'a, B> {
    pub const kRegularRules: AllowedRules = kRegularRules;
    pub const kTopLevelRules: AllowedRules = kTopLevelRules;
    pub const kKeyframeRules: AllowedRules = kKeyframeRules;
    pub const kFontFeatureRules: AllowedRules = kFontFeatureRules;
    pub const kPageMarginRules: AllowedRules = kPageMarginRules;
    pub const kConditionalRules: AllowedRules = kConditionalRules;
    pub const kNestedGroupRules: AllowedRules = kNestedGroupRules;

    // cpp: css_parser_impl.h:67-70; css_parser_impl.cc:200-205
    pub fn new(
        context: Option<&'a CSSParserContext<B::Platform>>,
        style_sheet: Option<&'a mut B::StyleSheetContents>,
    ) -> Self {
        Self {
            parsed_properties_: Vec::new(),
            context_: context,
            style_sheet_: style_sheet,
            observer_: None,
            lazy_state_: None,
            arena_: Vec::new(),
            in_nested_style_rule_: false,
            in_mixin_: false,
            media_query_cache_: HashMap::new(),
        }
    }

    // cpp: css_parser_impl.h:252-253
    pub fn GetContext(&self) -> Option<&CSSParserContext<B::Platform>> {
        self.context_
    }
    // An exclusive borrow preserves the source mutable pointer's capabilities.
    pub fn GetStyleSheet(&mut self) -> Option<&mut B::StyleSheetContents> {
        self.style_sheet_.as_deref_mut()
    }
    // Rust adapter for the original consumers' writes to parsed_properties_.
    // External required backends receive the actual vector, so they can append
    // shorthand expansions without introducing a second property store.
    pub fn ParsedPropertiesForConsumer(&mut self) -> &mut Vec<CSSPropertyValue<B::ValueBackend>> {
        &mut self.parsed_properties_
    }

    // cpp: css_parser_impl.h:268; css_parser_impl.cc:3647-3649
    pub fn GetMode(&self) -> CSSParserMode {
        self.context_
            .expect("GetMode requires a non-null CSSParserContext")
            .Mode()
    }

    // cpp: css_parser_impl.cc:621-633
    pub fn ConsumeSupportsDeclaration<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> bool
    where
        B: CSSParserImplValueBackend,
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        debug_assert!(self.parsed_properties_.is_empty());
        let observer_copy = self.observer_.take();
        self.ConsumeDeclaration(stream, RuleType::kStyle, false);
        self.observer_ = observer_copy;
        let result = !self.parsed_properties_.is_empty();
        self.parsed_properties_.truncate(0); // Source resize(0) retains capacity.
        result
    }

    // cpp: css_parser_impl.h:274-280; css_parser_impl.cc:745-791
    pub fn ConsumeRuleList<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed_rules: AllowedRules,
        allow_cdo_cdc_tokens: bool,
        nesting_type: CSSNestingType,
        parent_rule_for_nesting: Option<Rc<B::StyleRule>>,
        mut callback: impl FnMut(Rc<B::StyleRuleBase>, u32),
    ) -> bool {
        self.ConsumeRuleListWithParser(
            stream,
            allowed_rules,
            allow_cdo_cdc_tokens,
            nesting_type,
            parent_rule_for_nesting,
            |_, rule, offset| callback(rule, offset),
        )
    }

    fn ConsumeRuleListWithParser<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        mut allowed_rules: AllowedRules,
        allow_cdo_cdc_tokens: bool,
        nesting_type: CSSNestingType,
        parent_rule_for_nesting: Option<Rc<B::StyleRule>>,
        mut callback: impl FnMut(&mut Self, Rc<B::StyleRuleBase>, u32),
    ) -> bool {
        let mut seen_rule = false;
        let mut seen_import_or_namespace_rule = false;
        let mut first_rule_valid = false;
        while !stream.AtEnd() {
            let offset = stream.Offset();
            let rule = match stream.UncheckedPeek().GetType() {
                kWhitespaceToken => {
                    stream.UncheckedConsume();
                    continue;
                }
                kAtKeywordToken => self.ConsumeAtRule(
                    stream,
                    allowed_rules,
                    nesting_type,
                    parent_rule_for_nesting.clone(),
                ),
                kCDOToken | kCDCToken if allow_cdo_cdc_tokens => {
                    stream.UncheckedConsume();
                    continue;
                }
                _ => B::ConsumeQualifiedRule(
                    self,
                    stream,
                    allowed_rules,
                    nesting_type,
                    parent_rule_for_nesting.clone(),
                ),
            };
            if !seen_rule {
                seen_rule = true;
                first_rule_valid = rule.is_some();
            }
            if let Some(rule) = rule {
                allowed_rules = ComputeNewAllowedRules(
                    allowed_rules,
                    Some(rule.as_ref()),
                    &mut seen_import_or_namespace_rule,
                );
                callback(self, rule, offset);
            }
            debug_assert!(stream.Offset() > offset);
        }
        first_rule_valid
    }

    // cpp: css_parser_impl.cc:795-813
    pub fn ConsumeEndOfPreludeForAtRuleWithoutBlock<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        id: CSSAtRuleID,
    ) -> bool {
        stream.ConsumeWhitespace();
        if stream.AtEnd() {
            return true;
        }
        if stream.UncheckedPeek().GetType() == kSemicolonToken {
            stream.UncheckedConsume();
            return true;
        }
        if let Some(observer) = &mut self.observer_ {
            observer.ObserveErroneousAtRule(stream.Offset(), id);
        }
        self.ConsumeErroneousAtRule(stream, id);
        false
    }
    // cpp: css_parser_impl.cc:822-842
    pub fn ConsumeEndOfPreludeForAtRuleWithBlock<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        id: CSSAtRuleID,
    ) -> bool {
        stream.ConsumeWhitespace();
        if stream.AtEnd() {
            if let Some(observer) = &mut self.observer_ {
                observer.ObserveErroneousAtRule(stream.Offset(), id);
            }
            return false;
        }
        if stream.UncheckedPeek().GetType() == kLeftBraceToken {
            return true;
        }
        self.ConsumeErroneousAtRule(stream, id);
        false
    }
    // cpp: css_parser_impl.cc:844-858
    pub fn ConsumeErroneousAtRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        id: CSSAtRuleID,
    ) {
        if let Some(observer) = &mut self.observer_ {
            observer.ObserveErroneousAtRule(stream.Offset(), id);
        }
        stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken, kSemicolonToken]);
        if !stream.AtEnd() {
            if stream.UncheckedPeek().GetType() == kLeftBraceToken {
                let _guard = BlockGuard::new(stream);
            } else {
                stream.UncheckedConsume();
            }
        }
    }

    // cpp: css_parser_impl.cc:860-872
    fn ConsumeAtRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed_rules: AllowedRules,
        nesting_type: CSSNestingType,
        parent_rule_for_nesting: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        debug_assert_eq!(stream.Peek().GetType(), kAtKeywordToken);
        let name_token = stream.ConsumeIncludingWhitespace();
        let id = CssAtRuleID(&name_token.Value(), B::AtRuleFeatures());
        B::ConsumeAtRuleContents(
            self,
            id,
            stream,
            allowed_rules,
            nesting_type,
            parent_rule_for_nesting,
        )
    }
}

impl<B: CSSParserImplValueBackend> CSSParserImpl<'_, B> {
    // cpp: css_parser_impl.cc:635-653
    pub fn ParseDeclarationListForInspector(
        text: &String,
        context: Option<&CSSParserContext<B::Platform>>,
        observer: &mut B::CSSParserObserver,
    ) where
        B: CSSParserImplRuleBackend,
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        let mut parser = CSSParserImpl::<B>::new(context, None);
        parser.observer_ = Some(observer);
        parser
            .observer_
            .as_deref_mut()
            .unwrap()
            .StartRuleHeader(RuleType::kStyle, 0);
        parser.observer_.as_deref_mut().unwrap().EndRuleHeader(1);
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        parser
            .observer_
            .as_deref_mut()
            .unwrap()
            .StartRuleBody(stream.Offset());
        parser.ConsumeBlockContents(
            &mut stream,
            RuleType::kStyle,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        parser
            .observer_
            .as_deref_mut()
            .unwrap()
            .EndRuleBody(stream.LookAheadOffset());
    }
    // cpp: css_parser_impl.cc:674-691
    pub fn ParseDeclarationListForLazyStyle(
        text: &String,
        offset: u32,
        context: &CSSParserContext<B::Platform>,
    ) -> Rc<CSSPropertyValueSet<B::ValueBackend>>
    where
        B: CSSParserImplRuleBackend,
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), offset);
        let mut guard = BlockGuard::new(&mut stream);
        let mut parser = CSSParserImpl::<B>::new(Some(context), None);
        parser.ConsumeBlockContents(
            &mut guard,
            RuleType::kStyle,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        let document = context.GetDocument();
        CreateCSSPropertyValueSet::<B>(
            &mut parser.parsed_properties_,
            context.Mode(),
            document.as_ref(),
        )
    }
    // cpp: css_parser_impl.cc:3425-3551
    pub fn ConsumeDeclaration<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        rule: RuleType,
        has_visited_pseudo: bool,
    ) -> bool
    where
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        use super::at_rule_descriptors::AtRuleDescriptorID;
        let start = stream.Offset();
        debug_assert_eq!(stream.Peek().GetType(), kIdentToken);
        let lhs = stream.ConsumeIncludingWhitespace().clone();
        if stream.Peek().GetType() != kColonToken {
            return false;
        }
        stream.UncheckedConsume();
        stream.EnsureLookAhead();
        let count = self.parsed_properties_.len();
        let descriptor = matches!(
            rule,
            RuleType::kFontFace
                | RuleType::kFontPaletteValues
                | RuleType::kProperty
                | RuleType::kLocation
                | RuleType::kCounterStyle
                | RuleType::kViewTransition
                | RuleType::kFunction
                | RuleType::kMixin
        );
        let context = self.context_.expect("declaration context");
        let descriptor_id = if descriptor {
            lhs.ParseAsAtRuleDescriptorID()
        } else {
            AtRuleDescriptorID::Invalid
        };
        let property_id = if descriptor {
            CSSPropertyID::kInvalid
        } else {
            B::ParseAsUnresolvedCSSPropertyID(&lhs, context)
        };
        let valid_id = if descriptor {
            descriptor_id != AtRuleDescriptorID::Invalid
        } else {
            property_id != CSSPropertyID::kInvalid
        };
        const _: () = assert!(AtRuleDescriptorID::Invalid as u64 == 0);
        const _: () = assert!(CSSPropertyID::kInvalid as u64 == 0);
        let mut important = false;
        stream.ConsumeWhitespace();
        if valid_id {
            if descriptor {
                let variable = if descriptor_id == AtRuleDescriptorID::Variable {
                    AtomicString::from_utf16(lhs.Value().Span16())
                } else {
                    AtomicString::default()
                };
                B::ParseDescriptorValue(
                    rule,
                    descriptor_id,
                    &variable,
                    stream,
                    context,
                    &mut self.parsed_properties_,
                );
            } else if property_id == CSSPropertyID::kVariable {
                if !matches!(
                    rule,
                    RuleType::kStyle | RuleType::kScope | RuleType::kKeyframe
                ) {
                    return false;
                }
                let variable = AtomicString::from_utf16(lhs.Value().Span16());
                if !self.ConsumeVariableValue(
                    stream,
                    &variable,
                    rule != RuleType::kKeyframe,
                    rule == RuleType::kKeyframe,
                ) {
                    return false;
                }
            } else if self.observer_.is_some() {
                let saved = stream.Save();
                self.ConsumeDeclarationValue(stream, property_id, true, rule);
                if self.parsed_properties_.len() != count {
                    important = self.parsed_properties_.last().unwrap().IsImportant();
                } else {
                    stream.Restore(saved);
                    B::ConsumeVariableParserUnparsedDeclaration(
                        stream,
                        true,
                        false,
                        false,
                        true,
                        false,
                        &mut important,
                        context,
                    );
                }
            } else {
                if context.IsUseCounterRecordingEnabled()
                    && has_visited_pseudo
                    && property_id == CSSPropertyID::kColumnRuleColor
                {
                    context.CountWebFeature(B::VisitedColumnRuleColorFeature());
                }
                self.ConsumeDeclarationValue(stream, property_id, true, rule);
            }
        }
        if self.observer_.is_some()
            && matches!(
                rule,
                RuleType::kStyle
                    | RuleType::kScope
                    | RuleType::kKeyframe
                    | RuleType::kProperty
                    | RuleType::kPositionTry
                    | RuleType::kFontFace
                    | RuleType::kFunction
                    | RuleType::kCounterStyle
                    | RuleType::kFontPaletteValues
            )
        {
            if !valid_id {
                B::ConsumeVariableParserUnparsedDeclaration(
                    stream,
                    true,
                    false,
                    false,
                    true,
                    false,
                    &mut important,
                    context,
                );
            }
            stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken, kSemicolonToken]);
            self.observer_.as_deref_mut().unwrap().ObserveProperty(
                start,
                stream.LookAheadOffset(),
                important,
                self.parsed_properties_.len() != count,
            );
        }
        self.parsed_properties_.len() != count
    }
    // cpp: css_parser_impl.cc:3553-3583
    pub fn ConsumeVariableValue<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        name: &AtomicString,
        allow_important: bool,
        animation_tainted: bool,
    ) -> bool {
        stream.EnsureLookAhead();
        let context = self.context_.expect("variable context");
        let mut important = false;
        let value = if let Some(value) =
            B::ConsumeCSSWideKeyword(stream, context, allow_important, &mut important)
        {
            value
        } else {
            let Some(data) = B::ConsumeVariableParserUnparsedDeclaration(
                stream,
                allow_important,
                animation_tainted,
                false,
                false,
                false,
                &mut important,
                context,
            ) else {
                return false;
            };
            B::NewCSSUnparsedDeclarationValue(data, context)
        };
        self.parsed_properties_.push(CSSPropertyValue::new(
            &crate::css_property_name::CSSPropertyName::custom(name.clone()),
            value,
            important,
            false,
            0,
            false,
        ));
        context.CountProperty(CSSPropertyID::kVariable);
        true
    }
    // cpp: css_parser_impl.cc:3587-3597
    pub fn ConsumeDeclarationValue<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        property: CSSPropertyID,
        is_in_declaration_list: bool,
        rule: RuleType,
    ) {
        let allow_important =
            is_in_declaration_list && rule != RuleType::kKeyframe && rule != RuleType::kPositionTry;
        B::CSSPropertyParserParseValue(
            property,
            allow_important,
            stream,
            self.context_,
            &mut self.parsed_properties_,
            rule,
        );
    }
    // cpp: css_parser_impl.h:181-186; css_parser_impl.cc:207-227
    pub fn ParseValue(
        declaration: &mut MutableCSSPropertyValueSet<B::ValueBackend>,
        unresolved: CSSPropertyID,
        text: StringView,
        important: bool,
        context: Option<&CSSParserContext<B::Platform>>,
    ) -> SetResult {
        let mut parser = CSSParserImpl::<B>::new(context, None);
        let mut stream: CSSParserTokenStream<CSSTokenizer> = CSSParserTokenStream::new(text, 0);
        parser.ConsumeDeclarationValue(
            &mut stream,
            unresolved,
            false,
            RuleTypeForParserMode(declaration.CssParserMode()),
        );
        if parser.parsed_properties_.is_empty() {
            return SetResult::kParseError;
        }
        if important {
            for property in &mut parser.parsed_properties_ {
                property.SetImportant();
            }
        }
        declaration.AddParsedProperties(&parser.parsed_properties_)
    }
    // cpp: css_parser_impl.h:193-196; css_parser_impl.cc:229-240
    pub fn ParseValueIntoVector(
        result: &mut Vec<CSSPropertyValue<B::ValueBackend>>,
        unresolved: CSSPropertyID,
        text: StringView,
        context: Option<&CSSParserContext<B::Platform>>,
    ) -> usize {
        let mut parser = CSSParserImpl::<B>::new(context, None);
        let mut stream: CSSParserTokenStream<CSSTokenizer> = CSSParserTokenStream::new(text, 0);
        parser.ConsumeDeclarationValue(&mut stream, unresolved, false, RuleType::kStyle);
        let count = parser.parsed_properties_.len();
        result.extend(parser.parsed_properties_);
        count
    }
    // cpp: css_parser_impl.h:197-203; css_parser_impl.cc:242-260
    pub fn ParseVariableValue(
        declaration: &mut MutableCSSPropertyValueSet<B::ValueBackend>,
        name: &AtomicString,
        text: StringView,
        important: bool,
        context: Option<&CSSParserContext<B::Platform>>,
        animation_tainted: bool,
    ) -> SetResult {
        let mut parser = CSSParserImpl::<B>::new(context, None);
        let mut stream: CSSParserTokenStream<CSSTokenizer> = CSSParserTokenStream::new(text, 0);
        if !parser.ConsumeVariableValue(&mut stream, name, false, animation_tainted) {
            return SetResult::kParseError;
        }
        if important {
            parser
                .parsed_properties_
                .last_mut()
                .expect("successful variable consumer appends a property")
                .SetImportant();
        }
        declaration.AddParsedProperties(&parser.parsed_properties_)
    }
    // cpp: css_parser_impl.h:207-211; css_parser_impl.cc:389-404
    pub fn ParseInlineStyleDeclaration(
        text: &String,
        mode: CSSParserMode,
        secure: SecureContextMode,
        document: Option<&DocumentHandle<B::Platform>>,
    ) -> Rc<CSSPropertyValueSet<B::ValueBackend>>
    where
        B: CSSParserImplRuleBackend,
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        let context = CSSParserContext::FromMode(mode, secure, None);
        let mut parser = CSSParserImpl::<B>::new(Some(&context), None);
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        parser.ConsumeBlockContents(
            &mut stream,
            RuleType::kStyle,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        CreateCSSPropertyValueSet::<B>(&mut parser.parsed_properties_, mode, document)
    }
    // cpp: css_parser_impl.h:215-217; css_parser_impl.cc:406-430
    pub fn ParseDeclarationList(
        declaration: &mut MutableCSSPropertyValueSet<B::ValueBackend>,
        text: &String,
        context: Option<&CSSParserContext<B::Platform>>,
    ) -> bool
    where
        B: CSSParserImplRuleBackend,
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        let mut parser = CSSParserImpl::<B>::new(context, None);
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        parser.ConsumeBlockContents(
            &mut stream,
            RuleTypeForParserMode(declaration.CssParserMode()),
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        if parser.parsed_properties_.is_empty() {
            return false;
        }
        let mut unused_entries = parser.parsed_properties_.len();
        FilterProperties(
            &mut parser.parsed_properties_,
            &mut unused_entries,
            &mut HashSet::new(),
            &mut HashSet::new(),
        );
        declaration.AddParsedProperties(&parser.parsed_properties_[unused_entries..])
            != SetResult::kParseError
    }
    // cpp: css_parser_impl.h:220-224; css_parser_impl.cc:432-454
    pub fn ParseNestedDeclarationsRule(
        context: Option<&CSSParserContext<B::Platform>>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
        text: StringView,
    ) -> Option<Rc<B::StyleRuleBase>>
    where
        B: CSSParserImplRuleBackend,
        B::CSSParserObserver: CSSParserImplRuleObserver,
    {
        let mut parser = CSSParserImpl::<B>::new(context, None);
        let mut stream: CSSParserTokenStream<CSSTokenizer> = CSSParserTokenStream::new(text, 0);
        let mut children = Vec::new();
        parser.ConsumeBlockContents(
            &mut stream,
            RuleType::kStyle,
            nesting,
            parent,
            Some(0),
            Some(&mut children),
            false,
        );
        if children.len() == 1 {
            children.pop()
        } else {
            None
        }
    }
}
impl<B: CSSParserImplInlineElementBackend + CSSParserImplRuleBackend> CSSParserImpl<'_, B>
where
    B::CSSParserObserver: CSSParserImplRuleObserver,
{
    // cpp: css_parser_impl.h:204-206; css_parser_impl.cc:370-387
    pub fn ParseInlineStyleDeclarationForElement(
        text: &String,
        element: &mut B::Element,
    ) -> Rc<CSSPropertyValueSet<B::ValueBackend>> {
        let document = B::ElementDocument(element);
        let context = CSSParserContext::CopyWithDocument(
            &B::ElementSheetParserContext(element),
            Some(&document),
        );
        let mode = if B::IsHTMLElement(element) && !B::ElementDocumentInQuirksMode(element) {
            CSSParserMode::kHTMLStandardMode
        } else {
            CSSParserMode::kHTMLQuirksMode
        };
        context.SetMode(mode);
        let mut parser =
            CSSParserImpl::<B>::new(Some(&context), Some(B::ElementSheetContents(element)));
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        parser.ConsumeBlockContents(
            &mut stream,
            RuleType::kStyle,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        CreateCSSPropertyValueSet::<B>(&mut parser.parsed_properties_, mode, Some(&document))
    }
}
impl<B: CSSParserImplBackend> CSSParserImpl<'_, B> {
    // cpp: css_parser_impl.h:225-230; css_parser_impl.cc:456-484
    pub fn ParseRule(
        text: &String,
        context: Option<&CSSParserContext<B::Platform>>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
        sheet: Option<&mut B::StyleSheetContents>,
        allowed: AllowedRules,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let mut parser = CSSParserImpl::<B>::new(context, sheet);
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        stream.ConsumeWhitespace();
        if stream.UncheckedAtEnd() {
            return None;
        }
        let rule = if stream.UncheckedPeek().GetType() == kAtKeywordToken {
            parser.ConsumeAtRule(&mut stream, allowed, nesting, parent)
        } else {
            B::ConsumeQualifiedRule(&mut parser, &mut stream, allowed, nesting, parent)
        }?;
        stream.ConsumeWhitespace();
        if !stream.UncheckedAtEnd() {
            return None;
        }
        Some(rule)
    }
    // cpp: css_parser_impl.h:244; css_parser_impl.cc:608-619
    pub fn ParseCustomPropertyName(name: StringView) -> String {
        ParseCustomPropertyName(name)
    }
}
impl<B: CSSParserImplSheetBackend> CSSParserImpl<'_, B> {
    // cpp: css_parser_impl.cc:655-672
    pub fn ParseStyleSheetForInspector(
        text: &String,
        context: Option<&CSSParserContext<B::Platform>>,
        sheet: &mut B::StyleSheetContents,
        observer: &mut B::CSSParserObserver,
    ) {
        let mut parser = CSSParserImpl::<B>::new(context, Some(sheet));
        parser.observer_ = Some(observer);
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        let valid = parser.ConsumeRuleListWithParser(
            &mut stream,
            kTopLevelRules,
            true,
            CSSNestingType::kNone,
            None,
            |parser, rule, _| {
                if !rule.IsCharsetRule() {
                    B::ParserAppendRule(parser.style_sheet_.as_deref_mut().unwrap(), rule);
                }
            },
        );
        B::SetHasSyntacticallyValidCSSHeader(parser.style_sheet_.as_deref_mut().unwrap(), valid);
    }
    // cpp: css_parser_impl.h:231-236; css_parser_impl.cc:486-544
    pub fn ParseStyleSheet(
        text: &String,
        context: &CSSParserContext<B::Platform>,
        sheet: &mut B::StyleSheetContents,
        defer: CSSDeferPropertyParsing,
        allow_import_rules: bool,
    ) -> ParseSheetResult {
        let mut _timer = None;
        if let Some(document) = context.GetDocument() {
            if let Some(view) = B::DocumentView(&document) {
                if let Some(aggregator) = B::ViewUkmAggregator(&view) {
                    _timer = Some(B::GetScopedParseStyleSheetTimer(&aggregator));
                }
            }
        }
        B::TraceBeginStyleSheet(context.BaseURL(), context.Mode());
        B::TraceBeginStyleSheetParse();
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        let mut lazy = if defer == CSSDeferPropertyParsing::kYes {
            Some(B::NewLazyParsingState(context, text, sheet))
        } else {
            None
        };
        let mut parser = CSSParserImpl::<B>::new(Some(context), Some(sheet));
        parser.lazy_state_ = lazy.as_mut();
        let mut result = ParseSheetResult::kSucceeded;
        let first_rule_valid = parser.ConsumeRuleListWithParser(
            &mut stream,
            kTopLevelRules,
            true,
            CSSNestingType::kNone,
            None,
            |parser, rule, offset| {
                if rule.IsCharsetRule() {
                    return;
                }
                if rule.IsImportRule() {
                    if !allow_import_rules || context.IsForMarkupSanitization() {
                        result = ParseSheetResult::kHasUnallowedImportRule;
                        return;
                    }
                    if let Some(document) =
                        B::AnyOwnerDocument(parser.style_sheet_.as_deref().unwrap())
                    {
                        let mut position = B::MinimumTextPosition();
                        B::GetTextPosition(&document, offset, text, &mut position);
                        B::SetImportPositionHint(&rule, position);
                    }
                }
                B::ParserAppendRule(parser.style_sheet_.as_deref_mut().unwrap(), rule);
            },
        );
        B::SetHasSyntacticallyValidCSSHeader(
            parser.style_sheet_.as_deref_mut().unwrap(),
            first_rule_valid,
        );
        B::TraceEndStyleSheetParse();
        B::TraceEndStyleSheet(stream.TokenCount(), text.length());
        result
    }
}
impl<B: CSSParserImplPageBackend> CSSParserImpl<'_, B> {
    // cpp: css_parser_impl.h:237-239; css_parser_impl.cc:547-593
    pub fn ParsePageSelector<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        _sheet: Option<&mut B::StyleSheetContents>,
        context: &CSSParserContext<B::Platform>,
    ) -> Option<Rc<CSSSelectorList>> {
        stream.ConsumeWhitespace();
        let mut type_selector = AtomicString::default();
        if stream.Peek().GetType() == kIdentToken {
            type_selector = AtomicString::from_utf16(stream.Consume().Value().Span16());
        }
        let mut pseudo = AtomicString::default();
        if stream.Peek().GetType() == kColonToken {
            stream.Consume();
            if stream.Peek().GetType() != kIdentToken {
                return None;
            }
            pseudo = AtomicString::from_utf16(stream.Consume().Value().Span16());
        }
        stream.ConsumeWhitespace();
        let mut selectors = Vec::new();
        if !type_selector.IsNull() {
            selectors.push(CSSSelector::FromTag(
                QualifiedName::new(
                    AtomicString::default(),
                    type_selector,
                    AtomicString::from_str("*"),
                ),
                false,
            ));
        }
        if !pseudo.IsNull() {
            let mut selector = CSSSelector::default();
            selector.SetMatch(MatchType::kPagePseudoClass);
            selector.UpdatePseudoPage(
                pseudo.ToAsciiLower(),
                &B::SelectorParserContext(context.GetDocument()),
            );
            if selector.GetPseudoType() == PseudoType::kPseudoUnknown {
                return None;
            }
            if !selectors.is_empty() {
                selectors[0].SetLastInComplexSelector(false);
            }
            selectors.push(selector);
        }
        if selectors.is_empty() {
            selectors.push(CSSSelector::default());
        }
        selectors[0].SetForPage();
        selectors.last_mut().unwrap().SetLastInComplexSelector(true);
        Some(CSSSelectorList::AdoptSelectorVector(selectors))
    }
}
impl<B: CSSParserImplKeyframeBackend> CSSParserImpl<'_, B> {
    // cpp: css_parser_impl.cc:3599-3645
    pub fn ConsumeKeyframeKeyList<T: TokenStreamTokenizer>(
        context: Option<&CSSParserContext<B::Platform>>,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Vec<KeyframeOffset>> {
        use crate::style_rule_keyframe::TimelineNamedRange;
        let mut result = Vec::new();
        loop {
            stream.ConsumeWhitespace();
            let token = stream.Peek();
            if token.GetType() == kPercentageToken
                && token.NumericValue() >= 0.
                && token.NumericValue() <= 100.
            {
                result.push(KeyframeOffset::new(
                    TimelineNamedRange::kNone,
                    token.NumericValue() / 100.,
                ));
                stream.ConsumeIncludingWhitespace();
            } else if token.GetType() == kIdentToken {
                if EqualIgnoringAsciiCase(&token.Value(), b"from") {
                    result.push(KeyframeOffset::new(TimelineNamedRange::kNone, 0.));
                    stream.ConsumeIncludingWhitespace();
                } else if EqualIgnoringAsciiCase(&token.Value(), b"to") {
                    result.push(KeyframeOffset::new(TimelineNamedRange::kNone, 1.));
                    stream.ConsumeIncludingWhitespace();
                } else {
                    let list = B::ConsumeTimelineRangeNameAndPercent(context, stream)?;
                    result.push(KeyframeOffset::new(
                        B::TimelineRangeListName(&list),
                        B::TimelineRangeListClampedPercent(&list) / 100.,
                    ));
                }
            } else {
                return None;
            }
            if stream.Peek().GetType() != kCommaToken {
                return Some(result);
            }
            stream.Consume();
        }
    }
    // cpp: css_parser_impl.h:241-243; css_parser_impl.cc:595-606
    pub fn ParseKeyframeKeyList(
        context: Option<&CSSParserContext<B::Platform>>,
        text: &String,
    ) -> Option<Vec<KeyframeOffset>> {
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(text), 0);
        let result = Self::ConsumeKeyframeKeyList(context, &mut stream);
        if stream.AtEnd() {
            result
        } else {
            None
        }
    }
}

impl<B: CSSParserImplRuleBackend> CSSParserImpl<'_, B>
where
    B::CSSParserObserver: CSSParserImplRuleObserver,
{
    // cpp: css_parser_impl.cc:3002-3107. Arena cleanup runs on every return.
    pub fn ConsumeStyleRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
        nested: bool,
        invalid_rule_error: &mut bool,
    ) -> Option<Rc<B::StyleRule>> {
        if !self.in_nested_style_rule_ {
            debug_assert!(self.arena_.is_empty());
        }
        let clear_arena = !self.in_nested_style_rule_;
        let result = (|| {
            if let Some(observer) = &mut self.observer_ {
                observer.StartRuleHeader(RuleType::kStyle, stream.LookAheadOffset());
            }
            let ambiguous = B::StartsCustomPropertyDeclaration(stream);
            let mut has_visited_pseudo = false;
            let range = B::ConsumeSelector(
                stream,
                self.context_,
                nesting,
                parent,
                nested,
                self.style_sheet_.as_deref_mut(),
                self.observer_.as_deref_mut(),
                &mut self.arena_,
                &mut has_visited_pseudo,
            );
            let selectors = self.arena_[range].to_vec();
            if selectors.is_empty() {
                stream.EnsureLookAhead();
                if nested {
                    stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken, kSemicolonToken]);
                } else {
                    stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken]);
                }
            }
            if let Some(observer) = &mut self.observer_ {
                observer.EndRuleHeader(stream.LookAheadOffset());
            }
            if stream.Peek().GetType() != kLeftBraceToken {
                return None;
            }
            if ambiguous {
                if nested {
                    return None;
                }
                let _guard = BlockGuard::new(stream);
                return None;
            }
            if selectors.is_empty() {
                let _guard = BlockGuard::new(stream);
                *invalid_rule_error = true;
                return None;
            }
            if self.observer_.is_none() && self.lazy_state_.is_some() {
                debug_assert!(self.style_sheet_.is_some());
                let text = stream.RemainingText();
                let text = text.Substring(1, text.length() - 1);
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                let len = if B::HasAVX2AndPCLMUL() {
                    B::FindLengthOfDeclarationListAVX2(&text)
                } else {
                    B::FindLengthOfDeclarationList(&text)
                };
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                let len = B::FindLengthOfDeclarationList(&text);
                if len != 0 {
                    let start = stream.Offset();
                    stream.SkipToEndOfBlock((len + 2) as u32);
                    return Some(Rc::new(crate::style_rule::StyleRule::CreateLazy(
                        (*CSSSelectorList::AdoptSelectorVector(selectors)).clone(),
                        B::LazyParsingStateHandle(self.lazy_state_.as_deref().unwrap()),
                        start as usize,
                    )));
                }
            }
            let mut guard = BlockGuard::new(stream);
            Some(self.ConsumeStyleRuleContents(selectors, &mut guard, has_visited_pseudo))
        })();
        if clear_arena {
            self.arena_.truncate(0);
        }
        result
    }
    // cpp: css_parser_impl.cc:3109-3131. Allocate the actual parent before
    // parsing its children; the same Rc is passed into selector construction.
    pub fn ConsumeStyleRuleContents<T: TokenStreamTokenizer>(
        &mut self,
        selectors: Vec<CSSSelector>,
        stream: &mut CSSParserTokenStream<'_, T>,
        has_visited_pseudo: bool,
    ) -> Rc<B::StyleRule> {
        let rule = Rc::new(crate::style_rule::StyleRule::CreateWithoutProperties(
            (*CSSSelectorList::AdoptSelectorVector(selectors)).clone(),
        ));
        let mut children = Vec::new();
        if let Some(observer) = &mut self.observer_ {
            observer.StartRuleBody(stream.Offset());
        }
        self.ConsumeBlockContents(
            stream,
            RuleType::kStyle,
            CSSNestingType::kNesting,
            Some(rule.clone()),
            None,
            Some(&mut children),
            has_visited_pseudo,
        );
        if let Some(observer) = &mut self.observer_ {
            observer.EndRuleBody(stream.LookAheadOffset());
        }
        for child in children {
            rule.AddChildRule(child);
        }
        let properties = self.TakePropertySet(self.GetMode());
        rule.SetProperties(CSSPropertyValueSetRuleHandle::FromImmutable(properties));
        rule
    }
    // cpp: css_parser_impl.cc:3146-3277
    pub fn ConsumeBlockContents<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        rule_type: RuleType,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
        mut declarations_start: Option<usize>,
        mut children: Option<&mut Vec<Rc<B::StyleRuleBase>>>,
        has_visited_pseudo: bool,
    ) {
        debug_assert!(self.parsed_properties_.is_empty());
        loop {
            debug_assert!(!stream.HasLookAhead() || stream.AtEnd());
            if self.observer_.is_some() && !stream.HasLookAhead() {
                loop {
                    let start = stream.Offset();
                    if !stream.ConsumeCommentOrNothing() {
                        break;
                    }
                    self.observer_
                        .as_deref_mut()
                        .unwrap()
                        .ObserveComment(start, stream.Offset());
                }
            }
            if stream.AtEnd() {
                break;
            }
            match stream.UncheckedPeek().GetType() {
                kWhitespaceToken | kSemicolonToken => {
                    stream.UncheckedConsume();
                    continue;
                }
                kAtKeywordToken => {
                    let token = stream.ConsumeIncludingWhitespace();
                    let id = CssAtRuleID(&token.Value(), B::AtRuleFeatures());
                    let mut ignored = false;
                    let child = self.ConsumeNestedRule(
                        Some(id),
                        rule_type,
                        stream,
                        nesting,
                        parent.clone(),
                        &mut ignored,
                    );
                    debug_assert!(!ignored);
                    if let (Some(child), Some(children)) = (child, children.as_deref_mut()) {
                        self.EmitDeclarationsRuleIfNeeded(
                            rule_type,
                            nesting,
                            parent.as_deref(),
                            declarations_start,
                            children,
                        );
                        declarations_start = Some(self.parsed_properties_.len());
                        children.push(child);
                    }
                    continue;
                }
                kIdentToken => {
                    let saved = stream.Save();
                    let consumed = {
                        let mut boundary = Boundary::new(stream, kSemicolonToken);
                        self.ConsumeDeclaration(&mut boundary, rule_type, has_visited_pseudo)
                    };
                    if consumed {
                        if !stream.AtEnd() {
                            debug_assert_eq!(stream.UncheckedPeek().GetType(), kSemicolonToken);
                            stream.UncheckedConsume();
                        }
                        continue;
                    } else if stream.Peek().GetType() == kSemicolonToken {
                        stream.UncheckedConsume();
                        continue;
                    }
                    stream.Restore(saved);
                }
                kFunctionToken => {
                    stream.SkipUntilPeekedTypeIs(&[kSemicolonToken]);
                    if !stream.UncheckedAtEnd() {
                        stream.UncheckedConsume();
                    }
                    continue;
                }
                _ => {}
            }
            if !matches!(
                nesting,
                CSSNestingType::kNone | CSSNestingType::kFunction | CSSNestingType::kMixin
            ) {
                let mut invalid = false;
                let child = self.ConsumeNestedRule(
                    None,
                    rule_type,
                    stream,
                    nesting,
                    parent.clone(),
                    &mut invalid,
                );
                if let Some(child) = child {
                    if let Some(children) = children.as_deref_mut() {
                        self.EmitDeclarationsRuleIfNeeded(
                            rule_type,
                            nesting,
                            parent.as_deref(),
                            declarations_start,
                            children,
                        );
                        declarations_start = Some(self.parsed_properties_.len());
                        children.push(child);
                    }
                    continue;
                } else if invalid {
                    continue;
                }
                stream.EnsureLookAhead();
            }
            stream.SkipUntilPeekedTypeIs(&[kSemicolonToken]);
            if !stream.UncheckedAtEnd() {
                stream.UncheckedConsume();
            }
        }
        if let Some(children) = children {
            self.EmitDeclarationsRuleIfNeeded(
                rule_type,
                nesting,
                parent.as_deref(),
                declarations_start,
                children,
            );
        }
    }
    // cpp: css_parser_impl.cc:3285-3331
    pub fn ConsumeRuleListOrNestedDeclarationList<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
        children: &mut Vec<Rc<B::StyleRuleBase>>,
    ) {
        if matches!(
            nesting,
            CSSNestingType::kNesting | CSSNestingType::kFunction | CSSNestingType::kMixin
        ) {
            let rule = match nesting {
                CSSNestingType::kFunction => RuleType::kFunction,
                CSSNestingType::kMixin => RuleType::kMixin,
                _ => RuleType::kStyle,
            };
            self.ConsumeBlockContents(
                stream,
                rule,
                nesting,
                parent,
                Some(0),
                Some(children),
                false,
            );
        } else {
            self.ConsumeRuleList(stream, kRegularRules, false, nesting, parent, |rule, _| {
                children.push(rule)
            });
        }
    }
    // cpp: css_parser_impl.cc:3378-3409
    pub fn ConsumeNestedRule<T: TokenStreamTokenizer>(
        &mut self,
        id: Option<CSSAtRuleID>,
        parent_type: RuleType,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
        invalid: &mut bool,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let outer = std::mem::take(&mut self.parsed_properties_);
        let old_nested = self.in_nested_style_rule_;
        self.in_nested_style_rule_ = old_nested || parent_type == RuleType::kStyle;
        let child = if let Some(id) = id {
            B::ConsumeAtRuleContents(
                self,
                id,
                stream,
                AllowedNestedRules(parent_type, self.in_nested_style_rule_, self.in_mixin_),
                nesting,
                parent,
            )
        } else {
            self.ConsumeStyleRule(stream, nesting, parent, true, invalid)
                .map(|rule| Rc::new(crate::style_rule::StyleRuleBase::Style(rule)))
        };
        self.parsed_properties_ = outer;
        self.in_nested_style_rule_ = old_nested;
        if child.is_some()
            && !matches!(
                parent_type,
                RuleType::kPage | RuleType::kScope | RuleType::kFunction
            )
        {
            self.context_
                .unwrap()
                .CountWebFeature(B::CSSNestingFeature());
        }
        child
    }
    fn ObserveRuleHeaderAndBody(&mut self, rule_type: RuleType, start: u32, end: u32, body: u32) {
        if let Some(observer) = &mut self.observer_ {
            observer.StartRuleHeader(rule_type, start);
            observer.EndRuleHeader(end);
            observer.StartRuleBody(body);
        }
    }
    fn EndObservedRuleBody(&mut self, offset: u32) {
        if let Some(observer) = &mut self.observer_ {
            observer.EndRuleBody(offset);
        }
    }
    fn TakePropertySet(&mut self, mode: CSSParserMode) -> Rc<CSSPropertyValueSet<B::ValueBackend>> {
        let document = self.context_.unwrap().GetDocument();
        CreateCSSPropertyValueSet::<B>(&mut self.parsed_properties_, mode, document.as_ref())
    }

    // cpp: css_parser_impl.cc:874-986
    pub fn ConsumeAtRuleContents<T: TokenStreamTokenizer>(
        &mut self,
        id: CSSAtRuleID,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed: AllowedRules,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        if !allowed.Has(id) {
            self.ConsumeErroneousAtRule(stream, id);
            return None;
        }
        let context = self.context_.expect("at-rule parsing requires a context");
        if id != CSSAtRuleID::kCSSAtRuleInvalid && context.IsUseCounterRecordingEnabled() {
            B::CountAtRule(context, id);
        }
        stream.EnsureLookAhead();
        use CSSAtRuleID::*;
        match id {
            kCSSAtRuleMedia => return self.ConsumeMediaRule(stream, nesting, parent),
            kCSSAtRuleSupports => return self.ConsumeSupportsRule(stream, nesting, parent),
            kCSSAtRuleStartingStyle => {
                return self.ConsumeStartingStyleRule(stream, nesting, parent);
            }
            kCSSAtRuleFontFace => return self.ConsumeFontFaceRule(stream),
            kCSSAtRuleFontFeatureValues => return self.ConsumeFontFeatureValuesRule(stream),
            kCSSAtRuleWebkitKeyframes => return self.ConsumeKeyframesRule(true, stream),
            kCSSAtRuleKeyframes => return self.ConsumeKeyframesRule(false, stream),
            kCSSAtRulePage => return self.ConsumePageRule(stream),
            kCSSAtRuleCharset => return self.ConsumeCharsetRule(stream),
            kCSSAtRuleImport => {
                let mut modifiers = B::NewUrlRequestModifiers();
                let uri = ConsumeStringOrURI::<_, B, _>(stream, context, Some(&mut modifiers));
                stream.EnsureLookAhead();
                return self.ConsumeImportRule(uri, stream, &modifiers);
            }
            kCSSAtRuleNamespace => return self.ConsumeNamespaceRule(stream),
            kCSSAtRuleStylistic
            | kCSSAtRuleStyleset
            | kCSSAtRuleCharacterVariant
            | kCSSAtRuleSwash
            | kCSSAtRuleOrnaments
            | kCSSAtRuleAnnotation => {
                return self.ConsumeFontFeatureRule(id, stream).map(|rule| {
                    Rc::new(crate::style_rule::StyleRuleBase::FontFeature(Rc::new(rule)))
                });
            }
            kCSSAtRuleTopLeftCorner
            | kCSSAtRuleTopLeft
            | kCSSAtRuleTopCenter
            | kCSSAtRuleTopRight
            | kCSSAtRuleTopRightCorner
            | kCSSAtRuleBottomLeftCorner
            | kCSSAtRuleBottomLeft
            | kCSSAtRuleBottomCenter
            | kCSSAtRuleBottomRight
            | kCSSAtRuleBottomRightCorner
            | kCSSAtRuleLeftTop
            | kCSSAtRuleLeftMiddle
            | kCSSAtRuleLeftBottom
            | kCSSAtRuleRightTop
            | kCSSAtRuleRightMiddle
            | kCSSAtRuleRightBottom => return self.ConsumePageMarginRule(id, stream),
            kCSSAtRuleInvalid | kCount => {
                self.ConsumeErroneousAtRule(stream, id);
                return None;
            }
            kCSSAtRuleViewTransition => return self.ConsumeViewTransitionRule(stream),
            kCSSAtRuleContainer => return self.ConsumeContainerRule(stream, nesting, parent),
            kCSSAtRuleFontPaletteValues => return self.ConsumeFontPaletteValuesRule(stream),
            kCSSAtRuleLayer => return self.ConsumeLayerRule(stream, nesting, parent),
            kCSSAtRuleProperty => return self.ConsumePropertyRule(stream),
            kCSSAtRuleLocation => return self.ConsumeLocationRule(stream),
            kCSSAtRuleNavigation => return self.ConsumeNavigationRule(stream, nesting, parent),
            kCSSAtRuleScope => return self.ConsumeScopeRule(stream, nesting, parent),
            kCSSAtRuleCounterStyle => return self.ConsumeCounterStyleRule(stream),
            kCSSAtRuleFunction => return self.ConsumeFunctionRule(stream),
            kCSSAtRuleMixin => return self.ConsumeMixinRule(stream),
            kCSSAtRuleApplyMixin => return self.ConsumeApplyMixinRule(stream),
            kCSSAtRuleContents => return self.ConsumeContentsRule(stream),
            kCSSAtRuleResult => return self.ConsumeResultRule(stream),
            kCSSAtRulePrivate => return self.ConsumePrivateRule(stream),
            kCSSAtRulePositionTry => return self.ConsumePositionTryRule(stream),
            kCSSAtRuleCustomMedia => return self.ConsumeCustomMediaRule(stream),
        }
    }

    // cpp: css_parser_impl.cc:988-1038
    pub fn ConsumeQualifiedRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        allowed: AllowedRules,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        use crate::style_rule::StyleRuleBase;
        if allowed.Has(QualifiedRuleType::kStyle) {
            let mut invalid_rule_error = false;
            return self
                .ConsumeStyleRule(stream, nesting, parent, false, &mut invalid_rule_error)
                .map(|rule| Rc::new(StyleRuleBase::Style(rule)));
        }
        if allowed.Has(QualifiedRuleType::kKeyframe) {
            stream.EnsureLookAhead();
            let start = stream.LookAheadOffset();
            let mut keys = Self::ConsumeKeyframeKeyList(self.context_, stream);
            stream.ConsumeWhitespace();
            let prelude = RangeOffset::new(start, stream.LookAheadOffset());
            if stream.Peek().GetType() != kLeftBraceToken {
                keys = None;
                stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken]);
            }
            if stream.AtEnd() {
                return None;
            }
            let mut guard = BlockGuard::new(stream);
            return self.ConsumeKeyframeStyleRule(keys, prelude, &mut guard);
        }
        stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken]);
        if stream.Peek().GetType() == kLeftBraceToken {
            let _guard = BlockGuard::new(stream);
        }
        None
    }

    // cpp: css_parser_impl.cc:1040-1070
    pub fn ConsumePageMarginRule<T: TokenStreamTokenizer>(
        &mut self,
        id: CSSAtRuleID,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, id) {
            return None;
        }
        let end = stream.LookAheadOffset();
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kPageMargin, start, end, guard.Offset());
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kPageMargin,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(self.GetMode());
        Some(Rc::new(crate::style_rule::StyleRuleBase::PageMargin(
            crate::style_rule::StyleRulePageMargin::new(
                id,
                CSSPropertyValueSetRuleHandle::FromImmutable(properties),
            ),
        )))
    }

    // cpp: css_parser_impl.cc:1072-1087. Preserve the source's AtEnd test
    // before consuming the string (do not silently reinterpret @charset).
    pub fn ConsumeCharsetRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        if stream.Peek().GetType() != kStringToken || !stream.AtEnd() {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCharset);
            return None;
        }
        stream.ConsumeIncludingWhitespace();
        if !self.ConsumeEndOfPreludeForAtRuleWithoutBlock(stream, CSSAtRuleID::kCSSAtRuleCharset) {
            return None;
        }
        Some(Rc::new(crate::style_rule::StyleRuleBase::Charset(
            crate::style_rule::StyleRuleCharset::new(),
        )))
    }

    // cpp: css_parser_impl.cc:1089-1206
    pub fn ConsumeImportRule<T: TokenStreamTokenizer>(
        &mut self,
        uri: AtomicString,
        stream: &mut CSSParserTokenStream<'_, T>,
        modifiers: &B::Modifiers,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if uri.IsNull() {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleImport);
            return None;
        }
        let layer = ConsumeImportLayer(stream);
        let context = self.context_.unwrap();
        if !layer.is_empty() {
            context.CountWebFeature(B::CascadeLayersFeature());
        }
        stream.ConsumeWhitespace();
        let mut supports = StringView::default();
        let mut supported = SupportsResult::kSupported;
        if B::CSSSupportsForImportRulesEnabled()
            && stream.Peek().GetType() == kFunctionToken
            && stream.Peek().FunctionId() == Some(CSSValueID::kSupports)
        {
            {
                let mut guard = BlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let supports_start = guard.Offset();
                let save = guard.Save();
                if guard.Peek().GetType() == kIdentToken
                    && self.ConsumeSupportsDeclaration(&mut guard)
                {
                    supported = SupportsResult::kSupported;
                } else {
                    guard.Restore(save);
                    supported = B::ConsumeSupportsCondition(self, &mut guard);
                }
                supports = guard.StringRangeAt(supports_start, guard.Offset() - supports_start);
            }
            if supported == SupportsResult::kParseFailure {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleImport);
                return None;
            }
        }
        stream.ConsumeWhitespace();
        let mut scope = None;
        if B::CSSScopeImportEnabled() && stream.Peek().FunctionId() == Some(CSSValueID::kScope) {
            let mut guard = RestoringBlockGuard::new(stream);
            guard.ConsumeWhitespace();
            scope = B::ConsumeStyleScope(self, &mut guard);
            if !guard.Release() {
                scope = None;
            }
        }
        stream.ConsumeWhitespace();
        let media_start = stream.Offset();
        stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken, kSemicolonToken]);
        let end = stream.LookAheadOffset();
        let media_text = stream
            .StringRangeAt(media_start, end - media_start)
            .ToString();
        let media = B::ParseMediaQuerySetString(media_text, context);
        if !self.ConsumeEndOfPreludeForAtRuleWithoutBlock(stream, CSSAtRuleID::kCSSAtRuleImport) {
            return None;
        }
        self.ObserveRuleHeaderAndBody(RuleType::kImport, start, end, end);
        self.EndObservedRuleBody(end);
        Some(Rc::new(crate::style_rule::StyleRuleBase::Import(
            B::NewImportRule(
                uri,
                layer,
                scope,
                supported == SupportsResult::kSupported,
                supports.ToString(),
                media,
                context.IsOriginClean(),
                modifiers,
            ),
        )))
    }

    // cpp: css_parser_impl.cc:1208-1229
    pub fn ConsumeNamespaceRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let mut prefix = AtomicString::default();
        if stream.Peek().GetType() == kIdentToken {
            prefix = AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
        }
        let uri = ConsumeStringOrURI::<_, B, _>(stream, self.context_.unwrap(), None);
        if uri.IsNull() {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleNamespace);
            return None;
        }
        if !self.ConsumeEndOfPreludeForAtRuleWithoutBlock(stream, CSSAtRuleID::kCSSAtRuleNamespace)
        {
            return None;
        }
        Some(Rc::new(crate::style_rule::StyleRuleBase::Namespace(
            B::NewNamespaceRule(prefix, uri),
        )))
    }

    // cpp: css_parser_impl.cc:1274-1322
    pub fn CreateDeclarationsRule(
        &mut self,
        nesting: CSSNestingType,
        selectors: Option<&[CSSSelector]>,
        start: usize,
    ) -> Rc<B::StyleRuleBase> {
        let mut declarations = self.parsed_properties_[start..].to_vec();
        let context = self.context_.unwrap();
        use crate::style_rule::{StyleRule, StyleRuleBase};
        let selectors = match nesting {
            CSSNestingType::kNone => unreachable!("nested declarations require a nesting type"),
            CSSNestingType::kNesting => CSSSelectorList::CopyFromSelectors(selectors),
            CSSNestingType::kScope => WhereScopeSelector(),
            CSSNestingType::kFunction | CSSNestingType::kMixin => {
                let document = context.GetDocument();
                return Rc::new(StyleRuleBase::FunctionDeclarations(
                    B::NewFunctionDeclarationsRule(CreateCSSPropertyValueSet::<B>(
                        &mut declarations,
                        CSSParserMode::kCSSFunctionDescriptorsMode,
                        document.as_ref(),
                    )),
                ));
            }
        };
        let document = context.GetDocument();
        let properties =
            CreateCSSPropertyValueSet::<B>(&mut declarations, context.Mode(), document.as_ref());
        let list = CSSSelectorList::AdoptSelectorVector(selectors);
        Rc::new(StyleRuleBase::NestedDeclarations(
            B::NewNestedDeclarationsRule(
                nesting,
                StyleRule::Create(
                    (*list).clone(),
                    CSSPropertyValueSetRuleHandle::FromImmutable(properties),
                    None,
                ),
            ),
        ))
    }

    // cpp: css_parser_impl.cc:1324-1363
    pub fn EmitDeclarationsRuleIfNeeded(
        &mut self,
        rule_type: RuleType,
        nesting: CSSNestingType,
        parent: Option<&B::StyleRule>,
        start: Option<usize>,
        children: &mut Vec<Rc<B::StyleRuleBase>>,
    ) {
        if rule_type == RuleType::kPage {
            return;
        }
        let Some(start) = start else {
            return;
        };
        if start >= self.parsed_properties_.len() && self.observer_.is_none() {
            return;
        }
        let selectors = parent.map(|rule| rule.Selectors().CopySelectors());
        children.push(self.CreateDeclarationsRule(nesting, selectors.as_deref(), start));
        if let Some(observer) = &mut self.observer_ {
            observer.ObserveNestedDeclarations(children.len() - 1);
        }
        self.parsed_properties_.truncate(start);
    }

    // cpp: css_parser_impl.cc:1365-1439
    pub fn ConsumeMediaRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let save = stream.Save();
        let start = stream.LookAheadOffset();
        stream.SkipUntilPeekedTypeIs(&[kLeftBraceToken, kSemicolonToken]);
        let end = stream.LookAheadOffset();
        let text = stream.StringRangeAt(start, end - start).ToString();
        let media = match self
            .media_query_cache_
            .get(&text)
            .and_then(|edge| B::CachedMediaQuery(*edge))
        {
            Some(media) => media,
            None => {
                stream.Restore(save);
                let mut brace_boundary = Boundary::new(stream, kLeftBraceToken);
                let mut semicolon_boundary = Boundary::new(&mut brace_boundary, kSemicolonToken);
                B::ParseMediaQuerySet(&mut semicolon_boundary, self.context_.unwrap())
            }
        };
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleMedia) {
            return None;
        }
        self.media_query_cache_
            .insert(text, B::MediaQueryCacheEdge(&media));
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kMedia, start, end, guard.Offset());
        if let Some(sheet) = self.style_sheet_.as_deref() {
            sheet.SetHasMediaQueries();
        }
        let mut rules = Vec::new();
        self.ConsumeRuleListOrNestedDeclarationList(&mut guard, nesting, parent, &mut rules);
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Media(
            crate::style_rule::StyleRuleMedia::new(Some(media), rules),
        )))
    }

    // cpp: css_parser_impl.cc:1441-1485
    pub fn ConsumeSupportsRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let supported = B::ConsumeSupportsCondition(self, stream);
        if supported == SupportsResult::kParseFailure {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleSupports);
            return None;
        }
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleSupports) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kSupports, start, end, guard.Offset());
        let text = SimplifyWhiteSpace(guard.StringRangeAt(start, end - start));
        let mut rules = Vec::new();
        self.ConsumeRuleListOrNestedDeclarationList(&mut guard, nesting, parent, &mut rules);
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Supports(
            crate::style_rule::StyleRuleSupports::new(
                text,
                supported == SupportsResult::kSupported,
                rules,
            ),
        )))
    }

    // cpp: css_parser_impl.cc:1487-1517
    pub fn ConsumeStartingStyleRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleStartingStyle)
        {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kStartingStyle, start, end, guard.Offset());
        let mut rules = Vec::new();
        self.ConsumeRuleListOrNestedDeclarationList(&mut guard, nesting, parent, &mut rules);
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::StartingStyle(
            crate::style_rule::StyleRuleStartingStyle::new(rules),
        )))
    }

    // cpp: css_parser_impl.cc:1519-1553
    pub fn ConsumeFontFaceRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleFontFace) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kFontFace, start, end, guard.Offset());
        if let Some(sheet) = self.style_sheet_.as_deref() {
            sheet.SetHasFontFaceRule();
        }
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kFontFace,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(CSSParserMode::kCSSFontFaceRuleMode);
        Some(Rc::new(crate::style_rule::StyleRuleBase::FontFace(
            crate::style_rule::StyleRuleFontFace::new(
                CSSPropertyValueSetRuleHandle::FromImmutable(properties),
            ),
        )))
    }

    // cpp: css_parser_impl.cc:1555-1606
    pub fn ConsumeKeyframesRule<T: TokenStreamTokenizer>(
        &mut self,
        prefixed: bool,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let token = stream.Peek();
        let name = if token.GetType() == kIdentToken
            && IsValidIdentAnimationName(&token.Value(), B::CSSRevertRuleEnabled())
        {
            token.Value().ToString()
        } else if token.GetType() == kStringToken && token.Value().length() != 0 {
            self.context_
                .unwrap()
                .CountWebFeature(B::QuotedKeyframesFeature());
            token.Value().ToString()
        } else {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleKeyframes);
            return None;
        };
        stream.ConsumeIncludingWhitespace();
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleKeyframes) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kKeyframes, start, end, guard.Offset());
        let mut keys = Vec::new();
        self.ConsumeRuleList(
            &mut guard,
            kKeyframeRules,
            false,
            CSSNestingType::kNone,
            None,
            |rule, _| match rule.as_ref() {
                crate::style_rule::StyleRuleBase::Keyframe(key) => keys.push(Rc::clone(key)),
                _ => unreachable!("keyframe-only rule list produced a different rule class"),
            },
        );
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Keyframes(
            B::NewKeyframesRule(name, prefixed, keys),
        )))
    }

    // cpp: css_parser_impl.cc:1608-1696
    pub fn ConsumeFontFeatureRuleBlock<T: TokenStreamTokenizer>(
        &mut self,
        feature: crate::style_rule_font_feature_values::FontFeatureType,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<crate::style_rule_font_feature_values::StyleRuleFontFeature> {
        use crate::style_rule_font_feature_values::{FontFeatureType as F, StyleRuleFontFeature};
        let maximum = match feature {
            F::kCharacterVariant => 2,
            F::kStyleset => usize::MAX,
            _ => 1,
        };
        let mut rule = StyleRuleFontFeature::new(feature);
        while !stream.AtEnd() {
            let start = stream.Offset();
            if stream.Peek().GetType() != kIdentToken {
                return None;
            }
            let alias =
                AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
            if stream.Peek().GetType() != kColonToken {
                return None;
            }
            stream.UncheckedConsume();
            stream.ConsumeWhitespace();
            let mut values = Vec::new();
            stream.ConsumeWhitespace();
            loop {
                if values.len() == maximum {
                    return None;
                }
                values.push(B::ConsumeNonNegativeIntegerOrNumberCalc(
                    stream,
                    self.context_.unwrap(),
                )?);
                if stream.Peek().GetType() == kSemicolonToken || stream.AtEnd() {
                    break;
                }
            }
            if !stream.AtEnd() {
                stream.ConsumeIncludingWhitespace();
            }
            if values.is_empty() {
                return None;
            }
            let mut numbers = Vec::with_capacity(values.len());
            for value in values {
                let number = B::PrimitiveNumberValueIfKnown(&value)?;
                // ClampTo<int> followed by Vector<uint32_t>'s conversion.
                numbers.push(number.clamp(i32::MIN as f64, i32::MAX as f64) as i32 as u32);
            }
            if let Some(observer) = &mut self.observer_ {
                observer.ObserveProperty(start, stream.LookAheadOffset(), false, true);
            }
            if stream.Peek().GetType() == kSemicolonToken {
                stream.UncheckedConsume();
            }
            stream.ConsumeWhitespace();
            rule.UpdateAlias(alias, numbers);
        }
        Some(rule)
    }

    // cpp: css_parser_impl.cc:1698-1736
    pub fn ConsumeFontFeatureRule<T: TokenStreamTokenizer>(
        &mut self,
        id: CSSAtRuleID,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<crate::style_rule_font_feature_values::StyleRuleFontFeature> {
        let start = stream.LookAheadOffset();
        let feature = ToStyleRuleFontFeatureType(id)?;
        stream.ConsumeWhitespace();
        let end = stream.LookAheadOffset();
        if stream.Peek().GetType() != kLeftBraceToken {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        if let Some(observer) = &mut self.observer_ {
            observer.StartRuleHeader(RuleType::kFontFeature, start);
            observer.ObserveFontFeatureType(feature);
            observer.EndRuleHeader(end);
            observer.StartRuleBody(guard.Offset());
        }
        guard.ConsumeWhitespace();
        let rule = self.ConsumeFontFeatureRuleBlock(feature, &mut guard);
        if let Some(observer) = &mut self.observer_ {
            observer.EndRuleBody(guard.Offset());
            if rule.is_none() {
                observer.ObserveErroneousAtRule(start, id);
            }
        }
        rule
    }

    // cpp: css_parser_impl.cc:1738-1842
    pub fn ConsumeFontFeatureValuesRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        use crate::style_rule_font_feature_values::{
            FontFeatureAliases, FontFeatureType as F, StyleRuleFontFeatureValues,
        };
        let start = stream.LookAheadOffset();
        let list = B::ConsumeFontFamily(stream, self.context_.unwrap());
        let list = match list {
            Some(list) if !list.is_empty() => list,
            _ => {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFontFeatureValues);
                return None;
            }
        };
        let mut families = Vec::with_capacity(list.len());
        for value in list {
            if let CSSValuePayload::kFontFamilyClass(value) = value.Payload() {
                families.push(B::FontFamilyValue(value));
            } else {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFontFeatureValues);
                return None;
            }
        }
        let end = stream.LookAheadOffset();
        if !self
            .ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleFontFeatureValues)
        {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kFontFeatureValues, start, end, guard.Offset());
        let mut aliases: [FontFeatureAliases; 6] =
            std::array::from_fn(|_| FontFeatureAliases::new());
        let mut features = Vec::new();
        let mut had_valid_rules = false;
        let first_valid = self.ConsumeRuleList(
            &mut guard,
            kFontFeatureRules,
            false,
            CSSNestingType::kNone,
            None,
            |rule, _| {
                had_valid_rules = true;
                match rule.as_ref() {
                    crate::style_rule::StyleRuleBase::FontFeature(feature) => {
                        features.push(Rc::clone(feature))
                    }
                    _ => unreachable!("font-feature-only list produced a different rule class"),
                }
            },
        );
        if first_valid || had_valid_rules {
            for rule in features {
                let index = match rule.GetFeatureType() {
                    F::kStylistic => 0,
                    F::kStyleset => 1,
                    F::kCharacterVariant => 2,
                    F::kSwash => 3,
                    F::kOrnaments => 4,
                    F::kAnnotation => 5,
                };
                rule.OverrideAliasesIn(&mut aliases[index]);
            }
        }
        let [stylistic, styleset, variant, swash, ornaments, annotation] = aliases;
        let rule = StyleRuleFontFeatureValues::new(
            families, stylistic, styleset, variant, swash, ornaments, annotation,
        );
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(
            crate::style_rule::StyleRuleBase::FontFeatureValues(Rc::new(rule)),
        ))
    }

    // cpp: css_parser_impl.cc:1845-1884
    pub fn ConsumePageRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let selectors = Self::ParsePageSelector(
            stream,
            self.style_sheet_.as_deref_mut(),
            self.context_.unwrap(),
        );
        let selectors = match selectors {
            Some(selectors) if selectors.IsValid() => selectors,
            _ => {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRulePage);
                return None;
            }
        };
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRulePage) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kPage, start, end, guard.Offset());
        let mut children = Vec::new();
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kPage,
            CSSNestingType::kNone,
            None,
            None,
            Some(&mut children),
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(self.GetMode());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Page(
            crate::style_rule::StyleRulePage::new(
                selectors,
                CSSPropertyValueSetRuleHandle::FromImmutable(properties),
                children,
            ),
        )))
    }

    // cpp: css_parser_impl.cc:1886-1965
    pub fn ConsumePropertyRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if !IsValidVariableNameToken(stream.Peek()) {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleProperty);
            return None;
        }
        let name = stream.ConsumeIncludingWhitespace().Value().ToString();
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleProperty) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kProperty, start, end, guard.Offset());
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kProperty,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let rule = crate::style_rule::StyleRuleProperty::<B::RuleDependencies>::new(
            name,
            CSSPropertyValueSetRuleHandle::FromImmutable(
                self.TakePropertySet(CSSParserMode::kCSSPropertyRuleMode),
            ),
        );
        let syntax = B::PropertyRegistrationConvertSyntax(rule.GetSyntax().as_deref());
        let inherits = B::PropertyRegistrationConvertInherits(rule.Inherits().as_deref());
        let initial = syntax.as_ref().and_then(|syntax| {
            B::PropertyRegistrationConvertInitial(
                rule.GetInitialValue().as_deref(),
                syntax,
                self.context_.unwrap(),
                rule.GetName(),
            )
        });
        let invalid = syntax.is_none() || inherits.is_none() || initial.is_none();
        if invalid {
            if let Some(observer) = &mut self.observer_ {
                let mut failed = Vec::new();
                if syntax.is_none() {
                    failed.push(CSSPropertyID::kSyntax);
                }
                if inherits.is_none() {
                    failed.push(CSSPropertyID::kInherits);
                }
                if initial.is_none() && syntax.is_some() {
                    failed.push(CSSPropertyID::kInitialValue);
                }
                observer.ObserveErroneousAtRuleWithProperties(
                    start,
                    CSSAtRuleID::kCSSAtRuleProperty,
                    &failed,
                );
            }
            return None;
        }
        Some(Rc::new(crate::style_rule::StyleRuleBase::Property(rule)))
    }

    // cpp: css_parser_impl.cc:2016-2050
    pub fn ConsumeNavigationRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let query = match B::ParseNavigationQuery(stream) {
            Some(query) => query,
            None => {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleNavigation);
                return None;
            }
        };
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleNavigation) {
            return None;
        }
        let end = stream.LookAheadOffset();
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kNavigation, start, end, guard.Offset());
        let mut rules = Vec::new();
        self.ConsumeRuleListOrNestedDeclarationList(&mut guard, nesting, parent, &mut rules);
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Navigation(
            crate::style_rule::StyleRuleNavigation::new(query, rules),
        )))
    }

    // cpp: css_parser_impl.cc:2137-2176
    pub fn ConsumeScopeRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let scope = B::ConsumeStyleScopeForRule(self, stream, nesting, parent);
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleScope) {
            return None;
        }
        let mut scope = scope.unwrap_or_else(B::CreateImplicitStyleScope);
        // Source records the body start before entering the BlockGuard here.
        self.ObserveRuleHeaderAndBody(RuleType::kScope, start, end, stream.Offset());
        let mut guard = BlockGuard::new(stream);
        let mut rules = Vec::new();
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kScope,
            CSSNestingType::kScope,
            B::ScopeRuleForNesting(&mut scope),
            Some(0),
            Some(&mut rules),
            false,
        );
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Scope(
            crate::style_rule::StyleRuleScope::new(scope, rules),
        )))
    }

    // cpp: css_parser_impl.cc:2210-2252
    pub fn ConsumeContainerRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let queries = match B::ParseContainerQuerySet(stream, self.context_.unwrap()) {
            Some(queries) => queries,
            None => {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleContainer);
                return None;
            }
        };
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleContainer) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kContainer, start, end, guard.Offset());
        let mut rules = Vec::new();
        self.ConsumeRuleListOrNestedDeclarationList(&mut guard, nesting, parent, &mut rules);
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Container(
            crate::style_rule::StyleRuleContainer::new(queries, rules),
        )))
    }

    // cpp: css_parser_impl.cc:2254-2348
    pub fn ConsumeLayerRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
        nesting: CSSNestingType,
        parent: Option<Rc<B::StyleRule>>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let mut names = Vec::new();
        while !stream.AtEnd()
            && stream.Peek().GetType() != kLeftBraceToken
            && stream.Peek().GetType() != kSemicolonToken
        {
            if !names.is_empty() {
                if stream.Peek().GetType() != kCommaToken {
                    self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
                    return None;
                }
                stream.ConsumeIncludingWhitespace();
            }
            let name = ConsumeCascadeLayerName(stream);
            if name.is_empty() {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
                return None;
            }
            names.push(name);
        }
        if stream.AtEnd() || stream.Peek().GetType() == kSemicolonToken {
            if names.is_empty() || nesting == CSSNestingType::kNesting {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
                return None;
            }
            let end = stream.LookAheadOffset();
            if !self.ConsumeEndOfPreludeForAtRuleWithoutBlock(stream, CSSAtRuleID::kCSSAtRuleLayer)
            {
                return None;
            }
            self.ObserveRuleHeaderAndBody(RuleType::kLayerStatement, start, end, end);
            self.EndObservedRuleBody(end);
            return Some(Rc::new(crate::style_rule::StyleRuleBase::LayerStatement(
                crate::style_rule::StyleRuleLayerStatement::new(names),
            )));
        }
        let name = if names.is_empty() {
            vec![AtomicString::from_str("")]
        } else if names.len() > 1 {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLayer);
            return None;
        } else {
            names.pop().unwrap()
        };
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleLayer) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kLayerBlock, start, end, guard.Offset());
        let mut rules = Vec::new();
        self.ConsumeRuleListOrNestedDeclarationList(&mut guard, nesting, parent, &mut rules);
        self.EndObservedRuleBody(guard.Offset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::LayerBlock(
            crate::style_rule::StyleRuleLayerBlock::new(name, rules),
        )))
    }

    // cpp: css_parser_impl.cc:2401-2416
    fn ConsumeFunctionType<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<ParserSyntax<B>> {
        if stream.Peek().FunctionId() == Some(CSSValueID::kType) {
            let parsed = {
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let syntax = B::ConsumeSyntaxDefinition(&mut guard);
                if syntax.is_some() && guard.Release() {
                    syntax
                } else {
                    None
                }
            };
            if parsed.is_some() {
                stream.ConsumeWhitespace();
                return parsed;
            }
        }
        B::ConsumeSyntaxComponent(stream)
    }

    // cpp: css_parser_impl.cc:2418-2484
    pub fn ConsumeFunctionRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if stream.Peek().GetType() != kFunctionToken {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFunction);
            return None;
        }
        let name = AtomicString::from_utf16(stream.Peek().Value().Span16());
        let parameters = {
            let mut guard = BlockGuard::new(stream);
            guard.ConsumeWhitespace();
            self.ConsumeFunctionParameters(&mut guard)
        };
        let parameters = match parameters {
            Some(parameters) => parameters,
            None => {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFunction);
                return None;
            }
        };
        stream.ConsumeWhitespace();
        let return_type = if stream.Peek().Id() == CSSValueID::kReturns {
            stream.ConsumeIncludingWhitespace();
            match Self::ConsumeFunctionType(stream) {
                Some(syntax) => syntax,
                None => {
                    self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFunction);
                    return None;
                }
            }
        } else {
            B::CreateUniversalSyntax()
        };
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleFunction) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kFunction, start, end, guard.Offset());
        let mut rules = Vec::new();
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kFunction,
            CSSNestingType::kFunction,
            None,
            Some(0),
            Some(&mut rules),
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Function(
            crate::style_rule::StyleRuleFunction::new(name, parameters, rules, return_type),
        )))
    }

    // cpp: css_parser_impl.cc:2486-2556. The closure reproduces the arena scope
    // guard, including every prelude failure after the nested-style early exit.
    pub fn ConsumeMixinRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if self.in_nested_style_rule_ {
            return None;
        }
        let result = (|| {
            if stream.Peek().GetType() != kIdentToken && stream.Peek().GetType() != kFunctionToken {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleMixin);
                return None;
            }
            if !StartsWithAscii(&stream.Peek().Value(), b"--") {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleMixin);
                return None;
            }
            let name = AtomicString::from_utf16(stream.Peek().Value().Span16());
            let parameters = if stream.Peek().GetType() == kIdentToken {
                stream.ConsumeIncludingWhitespace();
                Some(Vec::new())
            } else {
                let mut guard = BlockGuard::new(stream);
                guard.ConsumeWhitespace();
                self.ConsumeFunctionParameters(&mut guard)
            };
            let parameters = match parameters {
                Some(parameters) => parameters,
                None => {
                    self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleMixin);
                    return None;
                }
            };
            stream.ConsumeWhitespace();
            if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleMixin) {
                return None;
            }
            let end = stream.LookAheadOffset();
            self.ObserveRuleHeaderAndBody(RuleType::kMixin, start, end, stream.Offset());
            let mut guard = BlockGuard::new(stream);
            let mut rules = Vec::new();
            self.ConsumeBlockContents(
                &mut guard,
                RuleType::kMixin,
                CSSNestingType::kMixin,
                None,
                Some(0),
                Some(&mut rules),
                false,
            );
            self.EndObservedRuleBody(guard.LookAheadOffset());
            Some(Rc::new(crate::style_rule::StyleRuleBase::Mixin(
                crate::style_rule::StyleRuleMixin::new(name, parameters, rules),
            )))
        })();
        self.arena_.clear();
        result
    }

    // cpp: css_parser_impl.cc:2558-2594
    pub fn ConsumeResultRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleResult) {
            return None;
        }
        stream.EnsureLookAhead();
        if let Some(observer) = &mut self.observer_ {
            observer.StartRuleHeader(RuleType::kResult, start);
            observer.EndRuleHeader(stream.LookAheadOffset());
        }
        let old_in_mixin = std::mem::replace(&mut self.in_mixin_, true);
        let parent = self.ConsumeDeclarationListForMixins(stream);
        self.in_mixin_ = old_in_mixin;
        let rules = {
            let mut parent_rule = parent.borrow_mut();
            parent_rule.EnsureChildRules();
            let mut child_rules = parent_rule.ChildRulesMut().unwrap();
            std::mem::take(&mut *child_rules)
        };
        Some(Rc::new(crate::style_rule::StyleRuleBase::Result(
            crate::style_rule::StyleRuleResult::new(rules),
        )))
    }

    // cpp: css_parser_impl.cc:2599-2653
    fn ConsumePrivateVariable<T: TokenStreamTokenizer>(
        &self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<
        Rc<<B::RuleDependencies as crate::style_rule::StyleRuleDependencies>::CSSPrivateVariable>,
    > {
        let mut stream = Boundary::new(stream, kSemicolonToken);
        if !IsValidVariableNameToken(stream.Peek()) {
            return None;
        }
        let name = AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
        let syntax = Self::ConsumeFunctionType(&mut stream);
        let default = if stream.Peek().GetType() == kColonToken {
            stream.ConsumeIncludingWhitespace();
            Some(B::ConsumeUnparsedDeclaration(
                &mut stream,
                self.context_.unwrap(),
                false,
            )?)
        } else {
            None
        };
        let syntax = syntax.unwrap_or_else(B::CreateUniversalSyntax);
        if let Some(default) = &default {
            if !B::VariableDataNeedsVariableResolution(default)
                && !B::SyntaxIsUniversal(&syntax)
                && !B::ParseSyntaxDefaultValue(&syntax, default, self.context_.unwrap())
            {
                return None;
            }
        }
        if !stream.AtEnd() {
            return None;
        }
        Some(B::NewPrivateVariable(
            name,
            syntax,
            default,
            self.context_.unwrap(),
        ))
    }

    // cpp: css_parser_impl.cc:2655-2694
    pub fn ConsumePrivateRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRulePrivate) {
            return None;
        }
        stream.EnsureLookAhead();
        let end = stream.LookAheadOffset();
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kPrivate, start, end, guard.Offset());
        let mut variables = Vec::new();
        guard.ConsumeWhitespace();
        while !guard.AtEnd() {
            if let Some(variable) = self.ConsumePrivateVariable(&mut guard) {
                variables.push(variable);
            } else {
                guard.SkipUntilPeekedTypeIs(&[kSemicolonToken]);
            }
            if !guard.AtEnd() {
                guard.ConsumeIncludingWhitespace();
            }
        }
        self.EndObservedRuleBody(guard.LookAheadOffset());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Private(
            crate::style_rule::StyleRulePrivate::new(variables),
        )))
    }

    // cpp: css_parser_impl.cc:2696-2731. This fake parent and its empty property
    // set are required by Chromium for selector cloning in @apply, @contents
    // and @result, rather than standing in for an unparsed real rule.
    pub fn ConsumeDeclarationListForMixins<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Rc<std::cell::RefCell<B::StyleRule>> {
        let mut guard = BlockGuard::new(stream);
        if let Some(observer) = &mut self.observer_ {
            observer.StartRuleBody(guard.Offset());
        }
        let mut dummy = CSSSelector::FromParent(None, true);
        dummy.SetLastInSelectorList(true);
        dummy.SetLastInComplexSelector(true);
        let selectors = CSSSelectorList::AdoptSelectorVector(vec![dummy]);
        let properties =
            ImmutableCSSPropertyValueSet::Create(&[], CSSParserMode::kHTMLStandardMode, false);
        let parent = Rc::new(std::cell::RefCell::new(
            crate::style_rule::StyleRule::Create(
                (*selectors).clone(),
                CSSPropertyValueSetRuleHandle::FromImmutable(properties),
                None,
            ),
        ));
        let mut rules = Vec::new();
        B::ConsumeMixinRuleListOrNestedDeclarationList(
            self,
            &mut guard,
            CSSNestingType::kNesting,
            parent.clone(),
            &mut rules,
        );
        {
            let mut parent_rule = parent.borrow_mut();
            for rule in rules {
                parent_rule.AddChildRule(rule);
            }
        }
        self.EndObservedRuleBody(guard.Offset());
        parent
    }

    // cpp: css_parser_impl.cc:2733-2796
    pub fn ConsumeApplyMixinRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if (stream.Peek().GetType() != kIdentToken && stream.Peek().GetType() != kFunctionToken)
            || !StartsWithAscii(&stream.Peek().Value(), b"--")
        {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleApplyMixin);
            return None;
        }
        let name = AtomicString::from_utf16(stream.Peek().Value().Span16());
        let mut arguments = Vec::new();
        if stream.Peek().GetType() == kIdentToken {
            stream.ConsumeIncludingWhitespace();
        } else if !B::ConsumeMixinArguments(stream, self.context_.unwrap(), &mut arguments) {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleApplyMixin);
            return None;
        }
        stream.EnsureLookAhead();
        if let Some(observer) = &mut self.observer_ {
            observer.StartRuleHeader(RuleType::kApplyMixin, start);
            observer.EndRuleHeader(stream.LookAheadOffset());
        }
        if stream.AtEnd() || stream.Peek().GetType() == kSemicolonToken {
            if !stream.AtEnd() {
                stream.UncheckedConsume();
            }
            if let Some(observer) = &mut self.observer_ {
                observer.StartRuleBody(stream.Offset());
                observer.EndRuleBody(stream.Offset());
            }
            return Some(Rc::new(crate::style_rule::StyleRuleBase::ApplyMixin(
                crate::style_rule::StyleRuleApplyMixin::WithoutContents(name, arguments),
            )));
        }
        if stream.Peek().GetType() != kLeftBraceToken {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleApplyMixin);
            return None;
        }
        let parent = self.ConsumeDeclarationListForMixins(stream);
        let rules = {
            let mut parent_rule = parent.borrow_mut();
            parent_rule.EnsureChildRules();
            let mut child_rules = parent_rule.ChildRulesMut().unwrap();
            std::mem::take(&mut *child_rules)
        };
        Some(Rc::new(crate::style_rule::StyleRuleBase::ApplyMixin(
            crate::style_rule::StyleRuleApplyMixin::WithContents(name, arguments, rules),
        )))
    }

    // cpp: css_parser_impl.cc:2798-2833
    pub fn ConsumeContentsRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        stream.ConsumeWhitespace();
        if stream.AtEnd() || stream.Peek().GetType() == kSemicolonToken {
            self.ObserveRuleHeaderAndBody(
                RuleType::kContents,
                start,
                stream.Offset(),
                stream.Offset(),
            );
            self.EndObservedRuleBody(stream.Offset());
            if !stream.AtEnd() {
                stream.UncheckedConsume();
            }
            return Some(Rc::new(crate::style_rule::StyleRuleBase::Contents(
                crate::style_rule::StyleRuleContentsStatement::new(Vec::new()),
            )));
        }
        if stream.Peek().GetType() != kLeftBraceToken {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleContents);
            return None;
        }
        if let Some(observer) = &mut self.observer_ {
            observer.StartRuleHeader(RuleType::kContents, start);
            observer.EndRuleHeader(stream.LookAheadOffset());
        }
        let parent = self.ConsumeDeclarationListForMixins(stream);
        let rules = {
            let mut parent_rule = parent.borrow_mut();
            parent_rule.EnsureChildRules();
            let mut child_rules = parent_rule.ChildRulesMut().unwrap();
            std::mem::take(&mut *child_rules)
        };
        Some(Rc::new(crate::style_rule::StyleRuleBase::Contents(
            crate::style_rule::StyleRuleContentsStatement::new(rules),
        )))
    }

    // cpp: css_parser_impl.cc:2839-2908
    pub fn ConsumeFunctionParameters<T: TokenStreamTokenizer>(
        &self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Vec<crate::style_rule::StyleRuleFunctionParameter<B::RuleDependencies>>> {
        let mut parameters = Vec::new();
        let mut first = true;
        loop {
            stream.ConsumeWhitespace();
            if first && stream.Peek().GetType() == kRightParenthesisToken {
                break;
            }
            if !IsValidVariableNameToken(stream.Peek()) {
                return None;
            }
            let name = stream.ConsumeIncludingWhitespace().Value().ToString();
            let syntax = Self::ConsumeFunctionType(stream);
            let default = if stream.Peek().GetType() == kColonToken {
                stream.ConsumeIncludingWhitespace();
                B::ConsumeUnparsedDeclaration(stream, self.context_.unwrap(), true)
            } else {
                None
            };
            if let (Some(syntax), Some(default)) = (&syntax, &default) {
                if !B::VariableDataNeedsVariableResolution(default)
                    && !B::ParseSyntaxDefaultValue(syntax, default, self.context_.unwrap())
                {
                    return None;
                }
            }
            parameters.push(crate::style_rule::StyleRuleFunctionParameter {
                name,
                r#type: syntax.unwrap_or_else(B::CreateUniversalSyntax),
                default_value: default,
            });
            if stream.Peek().GetType() == kRightParenthesisToken {
                break;
            }
            if stream.Peek().GetType() != kCommaToken {
                return None;
            }
            stream.ConsumeIncludingWhitespace();
            first = false;
        }
        Some(parameters)
    }

    // cpp: css_parser_impl.cc:2967-3000
    pub fn ConsumeCustomMediaRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        if !IsDashedIdent(stream.Peek()) {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCustomMedia);
            return None;
        }
        let name = AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
        if let Some(value) = GetCustomMediaBooleanValue(stream.Peek()) {
            stream.ConsumeIncludingWhitespace();
            if !self.ConsumeEndOfPreludeForAtRuleWithoutBlock(
                stream,
                CSSAtRuleID::kCSSAtRuleCustomMedia,
            ) {
                return None;
            }
            return Some(Rc::new(crate::style_rule::StyleRuleBase::CustomMedia(
                crate::style_rule::StyleRuleCustomMedia::FromBoolean(name, value),
            )));
        }
        let media = match B::ParseCustomMediaDefinition(stream, self.context_.unwrap()) {
            Some(media) => media,
            None => {
                self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCustomMedia);
                return None;
            }
        };
        if !self
            .ConsumeEndOfPreludeForAtRuleWithoutBlock(stream, CSSAtRuleID::kCSSAtRuleCustomMedia)
        {
            return None;
        }
        Some(Rc::new(crate::style_rule::StyleRuleBase::CustomMedia(
            crate::style_rule::StyleRuleCustomMedia::FromMediaQuery(name, media),
        )))
    }
    // cpp: css_parser_impl.cc:1967-2014
    pub fn ConsumeLocationRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if !IsDashedIdent(stream.Peek()) {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleLocation);
            return None;
        }
        let name = AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleLocation) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kLocation, start, end, guard.Offset());
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kLocation,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(self.GetMode());
        Some(Rc::new(crate::style_rule::StyleRuleBase::Location(
            B::NewLocationRule(name, properties),
        )))
    }

    // cpp: css_parser_impl.cc:2052-2089
    pub fn ConsumeCounterStyleRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let name = B::ConsumeCounterStyleNameInPrelude(stream, self.context_.unwrap());
        if name.IsNull() {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleCounterStyle);
            return None;
        }
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleCounterStyle)
        {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kCounterStyle, start, end, guard.Offset());
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kCounterStyle,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(CSSParserMode::kCSSCounterStyleRuleMode);
        Some(Rc::new(crate::style_rule::StyleRuleBase::CounterStyle(
            B::NewCounterStyleRule(name, properties),
        )))
    }

    // cpp: css_parser_impl.cc:2091-2135
    pub fn ConsumeFontPaletteValuesRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if !IsDashedIdent(stream.Peek()) {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRuleFontPaletteValues);
            return None;
        }
        let name = AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
        let end = stream.LookAheadOffset();
        if !self
            .ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleFontPaletteValues)
        {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kFontPaletteValues, start, end, guard.Offset());
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kFontPaletteValues,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(CSSParserMode::kCSSFontPaletteValuesRuleMode);
        Some(Rc::new(
            crate::style_rule::StyleRuleBase::FontPaletteValues(B::NewFontPaletteValuesRule(
                name, properties,
            )),
        ))
    }

    // cpp: css_parser_impl.cc:2178-2208
    pub fn ConsumeViewTransitionRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        let end = stream.LookAheadOffset();
        if !self
            .ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRuleViewTransition)
        {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kViewTransition, start, end, guard.Offset());
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kViewTransition,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(self.GetMode());
        Some(Rc::new(crate::style_rule::StyleRuleBase::ViewTransition(
            B::NewViewTransitionRule(properties),
        )))
    }

    // cpp: css_parser_impl.cc:2350-2395
    pub fn ConsumePositionTryRule<T: TokenStreamTokenizer>(
        &mut self,
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let start = stream.LookAheadOffset();
        if !(stream.Peek().GetType() == kIdentToken
            && (StartsWithAscii(&stream.Peek().Value(), b"--")
                || (self.GetMode() == CSSParserMode::kUASheetMode
                    && StartsWithAscii(&stream.Peek().Value(), b"-internal-"))))
        {
            self.ConsumeErroneousAtRule(stream, CSSAtRuleID::kCSSAtRulePositionTry);
            return None;
        }
        let name = AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16());
        let end = stream.LookAheadOffset();
        if !self.ConsumeEndOfPreludeForAtRuleWithBlock(stream, CSSAtRuleID::kCSSAtRulePositionTry) {
            return None;
        }
        let mut guard = BlockGuard::new(stream);
        self.ObserveRuleHeaderAndBody(RuleType::kPositionTry, start, end, guard.Offset());
        self.ConsumeBlockContents(
            &mut guard,
            RuleType::kPositionTry,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(guard.LookAheadOffset());
        let properties = self.TakePropertySet(CSSParserMode::kCSSPositionTryRuleMode);
        Some(Rc::new(crate::style_rule::StyleRuleBase::PositionTry(
            B::NewPositionTryRule(name, properties),
        )))
    }

    // cpp: css_parser_impl.cc:2910-2941
    pub fn ConsumeKeyframeStyleRule<T: TokenStreamTokenizer>(
        &mut self,
        keys: Option<Vec<KeyframeOffset>>,
        prelude: RangeOffset,
        block: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<B::StyleRuleBase>> {
        let keys = keys?;
        self.ObserveRuleHeaderAndBody(
            RuleType::kKeyframe,
            prelude.start,
            prelude.end,
            block.Offset(),
        );
        self.ConsumeBlockContents(
            block,
            RuleType::kKeyframe,
            CSSNestingType::kNone,
            None,
            None,
            None,
            false,
        );
        self.EndObservedRuleBody(block.LookAheadOffset());
        let properties = self.TakePropertySet(CSSParserMode::kCSSKeyframeRuleMode);
        Some(Rc::new(crate::style_rule::StyleRuleBase::Keyframe(
            B::NewKeyframeRule(keys, properties),
        )))
    }
}

// CSSVariableParser::IsValidVariableName and css_parsing_utils::IsDashedIdent
// intentionally differ on the two-character identifier "--".
fn StartsWithAscii(value: &StringView, prefix: &[u8]) -> bool {
    value.Span16().len() >= prefix.len()
        && value
            .Span16()
            .iter()
            .zip(prefix)
            .all(|(left, right)| *left == *right as u16)
}

// cpp: css_parser_impl.cc:3335-3373
fn AllowedNestedRules(
    parent: RuleType,
    in_nested_style_rule: bool,
    in_mixin: bool,
) -> AllowedRules {
    match parent {
        RuleType::kScope if !in_nested_style_rule => kRegularRules,
        RuleType::kScope | RuleType::kStyle => {
            let mut allowed = kNestedGroupRules
                .Union(AllowedRules::FromAtRules(&[CSSAtRuleID::kCSSAtRulePrivate]));
            if in_mixin {
                allowed = allowed.Union(AllowedRules::FromAtRules(&[
                    CSSAtRuleID::kCSSAtRuleContents,
                ]));
                allowed.Remove(CSSAtRuleID::kCSSAtRuleLayer);
            }
            allowed
        }
        RuleType::kMixin => kConditionalRules.Union(AllowedRules::FromAtRules(&[
            CSSAtRuleID::kCSSAtRulePrivate,
            CSSAtRuleID::kCSSAtRuleResult,
        ])),
        RuleType::kPage => kPageMarginRules,
        RuleType::kFunction => kConditionalRules,
        _ => AllowedRules::default(),
    }
}
fn IsDashedIdent(token: &super::css_parser_token::CSSParserToken) -> bool {
    token.GetType() == kIdentToken && StartsWithAscii(&token.Value(), b"--")
}
fn IsValidVariableNameToken(token: &super::css_parser_token::CSSParserToken) -> bool {
    IsDashedIdent(token) && token.Value().length() >= 3
}
// cpp: css_parser_impl.cc:2944-2963. Case-sensitive extension booleans.
fn GetCustomMediaBooleanValue(token: &super::css_parser_token::CSSParserToken) -> Option<bool> {
    if token.GetType() != kIdentToken {
        return None;
    }
    let value = token.Value();
    if value.Span16() == [b't' as u16, b'r' as u16, b'u' as u16, b'e' as u16] {
        Some(true)
    } else if value.Span16()
        == [
            b'f' as u16,
            b'a' as u16,
            b'l' as u16,
            b's' as u16,
            b'e' as u16,
        ]
    {
        Some(false)
    } else {
        None
    }
}

// cpp: css_parser_impl.cc:1103-1118 (the layer branch in ConsumeImportRule).
fn ConsumeImportLayer<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> Vec<AtomicString> {
    if stream.Peek().GetType() == kIdentToken && stream.Peek().Id() == CSSValueID::kLayer {
        stream.ConsumeIncludingWhitespace();
        return vec![AtomicString::from_str("")];
    }
    if stream.Peek().GetType() == kFunctionToken
        && stream.Peek().FunctionId() == Some(CSSValueID::kLayer)
    {
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let name = ConsumeCascadeLayerName(&mut guard);
        if !name.is_empty() && guard.AtEnd() {
            guard.Release();
            return name;
        }
    }
    Vec::new()
}

// cpp: css_parser_impl.cc:1238-1257. CSSSelector(AtomicString("scope"),
// false) has no runtime-dependent classification; construct its known pseudo.
fn WhereScopeSelector() -> Vec<CSSSelector> {
    let mut inner = CSSSelector::default();
    inner.SetMatch(MatchType::kPseudoClass);
    inner.SetPseudoType(PseudoType::kPseudoScope);
    inner.SetValue(AtomicString::from_str("scope"), false);
    inner.SetLastInComplexSelector(true);
    inner.SetLastInSelectorList(true);
    let list = CSSSelectorList::AdoptSelectorVector(vec![inner]);
    let mut selector = CSSSelector::default();
    selector.SetWhere(list);
    selector.SetScopeContaining(true);
    selector.SetLastInComplexSelector(true);
    selector.SetLastInSelectorList(true);
    vec![selector]
}

// Required source helper css_parsing_utils.cc:4572-4576, expanded here so
// keyframes prelude validation uses the source's exact ASCII comparisons.
fn IsValidIdentAnimationName(name: &StringView, revert_rule_enabled: bool) -> bool {
    ![
        b"".as_slice(),
        b"none",
        b"default",
        b"initial",
        b"inherit",
        b"unset",
        b"revert",
        b"revert-layer",
    ]
    .iter()
    .any(|keyword| EqualIgnoringAsciiCase(name, keyword))
        && !(revert_rule_enabled && EqualIgnoringAsciiCase(name, b"revert-rule"))
}

// WTF::String::SimplifyWhiteSpace uses unicode::IsSpaceOrNewline.
// Work directly on UTF-16 so surrogates and the source predicate are preserved.
fn SimplifyWhiteSpace(text: StringView) -> String {
    let mut units = Vec::new();
    let mut pending_space = false;
    for &unit in text.Span16() {
        if foundation::unicode::IsSpaceOrNewline(unit) {
            pending_space = !units.is_empty();
        } else {
            if pending_space {
                units.push(0x20);
                pending_space = false;
            }
            units.push(unit);
        }
    }
    String::from_utf16(&units)
}

// cpp: css_parser_impl.cc:693-742
pub fn ComputeNewAllowedRules<R: CSSRuleOrdering>(
    old_allowed_rules: AllowedRules,
    rule: Option<&R>,
    seen_import_or_namespace_rule: &mut bool,
) -> AllowedRules {
    let Some(rule) = rule else {
        return old_allowed_rules;
    };
    let mut new_allowed_rules = old_allowed_rules;
    if rule.IsCharsetRule() || (rule.IsLayerStatementRule() && !*seen_import_or_namespace_rule) {
        new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
    } else if rule.IsImportRule() {
        *seen_import_or_namespace_rule = true;
        new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
    } else if rule.IsNamespaceRule() {
        *seen_import_or_namespace_rule = true;
        new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
        new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleImport);
    } else {
        new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleCharset);
        new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleImport);
        new_allowed_rules.Remove(CSSAtRuleID::kCSSAtRuleNamespace);
    }
    new_allowed_rules
}

// cpp: css_parser_impl.cc:127-154
// The exact underlying alias is core/css/style_rule.h:98:
// StyleRuleBase::LayerName = Vector<AtomicString, 1>.
pub fn ConsumeCascadeLayerName<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> Vec<AtomicString> {
    let savepoint = stream.Save();
    let mut name = Vec::new();
    while !stream.AtEnd() && stream.Peek().GetType() == kIdentToken {
        let name_part = stream.Consume();
        name.push(AtomicString::from_utf16(name_part.Value().Span16()));
        if stream.Peek().GetType() != kDelimiterToken || stream.Peek().Delimiter() != b'.' as u16 {
            break;
        }
        let inner_savepoint = stream.Save();
        stream.Consume();
        if stream.Peek().GetType() != kIdentToken {
            stream.Restore(inner_savepoint);
            break;
        }
    }
    if name.is_empty() {
        stream.Restore(savepoint);
    } else {
        stream.ConsumeWhitespace();
    }
    name
}

#[cfg(test)]
mod tests {
    use super::super::css_tokenizer::CSSTokenizer;
    use super::*;
    use foundation::StringView;

    #[test]
    fn rule_dispatch_identities_and_at_rule_preludes_use_real_tokenizer() {
        // At-rule spelling is decoded by CSSTokenizer before the same dispatch
        // identifier conversion used by ConsumeAtRule. Unknown names remain
        // invalid and font-feature subtypes stay distinct.
        for (source, expected, feature) in [
            ("@font-face {}", CSSAtRuleID::kCSSAtRuleFontFace, None),
            (
                r"@styl\69 stic {}",
                CSSAtRuleID::kCSSAtRuleStylistic,
                Some(crate::style_rule_font_feature_values::FontFeatureType::kStylistic),
            ),
            (
                "@character-variant {}",
                CSSAtRuleID::kCSSAtRuleCharacterVariant,
                Some(crate::style_rule_font_feature_values::FontFeatureType::kCharacterVariant),
            ),
            ("@unknown {}", CSSAtRuleID::kCSSAtRuleInvalid, None),
        ] {
            let mut stream: CSSParserTokenStream<CSSTokenizer> =
                CSSParserTokenStream::new(StringView::from(source), 0);
            assert_eq!(stream.Peek().GetType(), kAtKeywordToken);
            let name = stream.ConsumeIncludingWhitespace();
            let id = CssAtRuleID(&name.Value(), AtRuleRuntimeFeatures::default());
            assert_eq!(id, expected);
            assert_eq!(ToStyleRuleFontFeatureType(id), feature);
        }
        // Import layer() must restore its entire function on grammar failure,
        // allowing the media parser to consume it as general-enclosed.
        for (source, names, tail) in [
            ("layer screen", vec![""], "screen"),
            (r"layer(foo.b\61 r) screen", vec!["foo", "bar"], "screen"),
            ("layer() screen", vec![], "layer"),
            ("layer(foo.) screen", vec![], "layer"),
            ("layer(foo extra) screen", vec![], "layer"),
        ] {
            let mut stream: CSSParserTokenStream<CSSTokenizer> =
                CSSParserTokenStream::new(StringView::from(source), 0);
            let names: Vec<_> = names.into_iter().map(AtomicString::from_str).collect();
            assert_eq!(ConsumeImportLayer(&mut stream), names);
            stream.ConsumeWhitespace();
            assert_eq!(stream.Peek().Value().ToString(), String::from(tail));
        }
        // Identifier escapes are decoded before these predicates. Registration
        // and function parameters reject "--"; extension/location names accept
        // it, and custom-media boolean values are case-sensitive.
        for (source, dashed, variable, boolean) in [
            ("--", true, false, None),
            (r"--n\61 me", true, true, None),
            ("name", false, false, None),
            (r"tr\75 e", false, false, Some(true)),
            ("false", false, false, Some(false)),
            ("TRUE", false, false, None),
            ("'true'", false, false, None),
            ("true()", false, false, None),
        ] {
            let mut stream: CSSParserTokenStream<CSSTokenizer> =
                CSSParserTokenStream::new(StringView::from(source), 0);
            let token = stream.Peek();
            assert_eq!(IsDashedIdent(token), dashed, "{source}");
            assert_eq!(IsValidVariableNameToken(token), variable, "{source}");
            assert_eq!(GetCustomMediaBooleanValue(token), boolean, "{source}");
        }
        for (source, enabled, valid) in [
            ("spin", false, true),
            (r"n\6f ne", false, false),
            ("DEFAULT", false, false),
            ("revert-layer", false, false),
            ("revert-rule", false, true),
            ("revert-rule", true, false),
        ] {
            let mut stream: CSSParserTokenStream<CSSTokenizer> =
                CSSParserTokenStream::new(StringView::from(source), 0);
            let token = stream.Peek();
            assert_eq!(token.GetType(), kIdentToken);
            assert_eq!(IsValidIdentAnimationName(&token.Value(), enabled), valid);
        }
    }

    // Typed payload fixtures test property storage/control only; no fixture
    // consumes CSS syntax. Uninstantiable payloads forbid all unrelated casts.
    use crate::css_property_name::CSSPropertyName;
    use crate::css_value::*;
    #[derive(Clone, Copy)]
    enum Never {}
    impl CSSValueSubclass for Never {
        fn CustomCSSText(&self) -> String {
            match *self {}
        }
        fn Equals(&self, _: &Self) -> bool {
            match *self {}
        }
    }
    impl CSSValueCustomHash for Never {
        fn CustomHash(&self) -> u32 {
            match *self {}
        }
    }
    impl CSSValueRandom for Never {
        fn HasRandomFunctions(&self) -> bool {
            match *self {}
        }
    }
    impl CSSValueSubresources for Never {
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            match *self {}
        }
    }
    impl CSSValueUrl<()> for Never {
        fn ReResolveUrl(&self, _: &()) {
            match *self {}
        }
    }
    impl CSSValueListUrls for Never {
        fn MayContainUrl(&self) -> bool {
            match *self {}
        }
    }
    impl CSSValueListSubclass<StoreDispatch> for Never {
        fn AsValueList(&self) -> &Never {
            match *self {}
        }
    }
    impl CSSValuePairSubclass<StoreDispatch> for Never {
        fn AsValuePair(&self) -> &Never {
            match *self {}
        }
    }
    impl CSSValueTreeScope<StoreDispatch> for Never {
        fn PopulateWithTreeScope<'a>(&'a self, _: Option<&'a ()>) -> &'a CSSValue<StoreDispatch> {
            match *self {}
        }
    }
    struct StoredText(&'static str);
    impl CSSValueSubclass for StoredText {
        fn CustomCSSText(&self) -> String {
            String::from(self.0)
        }
        fn Equals(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }
    impl CSSValueCustomHash for StoredText {
        fn CustomHash(&self) -> u32 {
            unreachable!("no hash query in store-control tests")
        }
    }
    struct StoreDispatch;
    macro_rules! unavailable_payloads { ($($name:ident),+ $(,)?) => { $(type $name = Never;)+ }; }
    impl CSSValueDispatch for StoreDispatch {
        type Document = ();
        type TreeScope = ();
        type CSSStringValue = StoredText;
        unavailable_payloads!(
            CSSNumericLiteralValue,
            CSSMathFunctionValue,
            CSSIdentifierValue,
            CSSScopedKeywordValue,
            CSSColor,
            CSSUnresolvedColorValue,
            CSSColorMixValue,
            CSSAlphaColorValue,
            CSSContrastColorValue,
            CSSCounterValue,
            CSSCounterContentValue,
            CSSQuadValue,
            CSSCustomIdentValue,
            CSSURIValue,
            CSSURLPatternValue,
            CSSValuePair,
            CSSLightDarkValuePair,
            CSSParamValuePair,
            CSSScrollValue,
            CSSViewValue,
            CSSRatioValue,
            CSSRelativeColorValue,
            CSSBasicShapeCircleValue,
            CSSBasicShapeEllipseValue,
            CSSBasicShapePolygonValue,
            CSSBasicShapeInsetValue,
            CSSBasicShapeRectValue,
            CSSBasicShapeXYWHValue,
            CSSPathValue,
            CSSShapeValue,
            CSSImageValue,
            CSSCursorImageValue,
            CSSCrossfadeValue,
            CSSPaintValue,
            CSSLinearGradientValue,
            CSSRadialGradientValue,
            CSSConicGradientValue,
            CSSConstantGradientValue,
            CSSColorImageValue,
            CSSLinearTimingFunctionValue,
            CSSCubicBezierTimingFunctionValue,
            CSSStepsTimingFunctionValue,
            CSSProgressValue,
            CSSBorderImageSliceValue,
            CSSDynamicRangeLimitMixValue,
            CSSFontFeatureValue,
            CSSFontFaceSrcValue,
            CSSFontFamilyValue,
            CSSFontStyleRangeValue,
            CSSFontVariationValue,
            CSSAlternateValue,
            CSSInheritedValue,
            CSSInitialValue,
            CSSUnsetValue,
            CSSRevertValue,
            CSSRevertLayerValue,
            CSSRevertRuleValue,
            CSSReflectValue,
            CSSShadowValue,
            CSSUnicodeRangeValue,
            CSSGridTemplateAreasValue,
            CSSPaletteMixValue,
            CSSRayValue,
            CSSUnparsedDeclarationValue,
            CSSPendingSubstitutionValue,
            CSSPendingSystemFontValue,
            CSSInvalidVariableValue,
            CSSCyclicVariableValue,
            CSSFlipRevertValue,
            CSSLayoutFunctionValue,
            CSSContentDistributionValue,
            CSSKeyframeShorthandValue,
            CSSInitialColorValue,
            CSSImageSetOptionValue,
            CSSImageSetTypeValue,
            CSSRepeatStyleValue,
            CSSSuperellipseValue,
            CSSSymbolsValue,
            CSSTriggerAttachmentValue,
            CSSRepeatValue,
            CSSValueList,
            CSSFunctionValue,
            CSSImageSetValue,
            CSSBracketedValueList,
            CSSGridAutoRepeatValue,
            CSSGridIntegerRepeatValue,
            CSSAxisValue
        );
        fn CreateIdentifierFromLength(_: &foundation::Length) -> Rc<CSSValue<Self>> {
            unreachable!()
        }
        fn CreatePrimitiveFromLength(_: &foundation::Length, _: f32) -> Rc<CSSValue<Self>> {
            unreachable!()
        }
    }
    impl CSSPropertyValueBackend for StoreDispatch {
        fn MatchingShorthandsForLonghand(_: CSSPropertyID) -> Vec<CSSPropertyID> {
            unreachable!()
        }
        fn IsAffectedByAll(_: CSSPropertyID) -> bool {
            unreachable!()
        }
    }
    impl CSSPropertyValueSetBackend for StoreDispatch {
        type CSSStyleDeclaration = ();
        type ExecutionContext = ();
        fn ShorthandForProperty(id: CSSPropertyID) -> Vec<CSSPropertyID> {
            assert!(matches!(
                id,
                CSSPropertyID::kWidth | CSSPropertyID::kHeight | CSSPropertyID::kVariable
            ));
            Vec::new()
        }
        fn IsInSameLogicalPropertyGroupWithDifferentMappingLogic(
            _: CSSPropertyID,
            _: CSSPropertyID,
        ) -> bool {
            unreachable!()
        }
        fn SerializeShorthand(_: &CSSPropertyValueSet<Self>, _: CSSPropertyID) -> String {
            unreachable!()
        }
        fn AsText(_: &CSSPropertyValueSet<Self>) -> String {
            unreachable!()
        }
        fn CreateIdentifier(_: CSSValueID) -> Rc<CSSValue<Self>> {
            unreachable!()
        }
        fn DeclarationPropertyValueSet(_: &()) -> Option<&CSSPropertyValueSet<Self>> {
            unreachable!()
        }
        fn DeclarationPropertyMatches(_: &(), _: CSSPropertyID, _: &CSSValue<Self>) -> bool {
            unreachable!()
        }
        fn NewCSSStyleDeclaration(_: Option<&()>, _: &MutableCSSPropertyValueSet<Self>) -> Rc<()> {
            unreachable!()
        }
    }
    fn stored_property(
        name: CSSPropertyName,
        text: &'static str,
        important: bool,
    ) -> CSSPropertyValue<StoreDispatch> {
        CSSPropertyValue::new(
            &name,
            Rc::new(CSSValue::new(CSSValuePayload::kStringClass(StoredText(
                text,
            )))),
            important,
            false,
            0,
            false,
        )
    }
    #[test]
    fn declaration_filter_preserves_important_priority_and_custom_property_identity_in_real_store()
    {
        let custom = |name| CSSPropertyName::custom(AtomicString::from_str(name));
        let mut properties = vec![
            stored_property(
                CSSPropertyName::new(CSSPropertyID::kWidth),
                "early important",
                true,
            ),
            stored_property(custom("--x"), "old custom", false),
            stored_property(
                CSSPropertyName::new(CSSPropertyID::kHeight),
                "height",
                false,
            ),
            stored_property(
                CSSPropertyName::new(CSSPropertyID::kWidth),
                "later normal",
                false,
            ),
            stored_property(custom("--X"), "distinct case", false),
            stored_property(custom("--x"), "custom winner", true),
            stored_property(custom("--x"), "later custom normal", false),
        ];
        let mut unused = properties.len();
        FilterProperties(
            &mut properties,
            &mut unused,
            &mut HashSet::new(),
            &mut HashSet::new(),
        );
        let immutable = ImmutableCSSPropertyValueSet::Create(
            &properties[unused..],
            CSSParserMode::kHTMLStandardMode,
            false,
        );
        assert_eq!(immutable.PropertyCount(), 4);
        assert_eq!(immutable.PropertyAt(0).PropertyID(), CSSPropertyID::kHeight);
        assert_eq!(
            immutable.PropertyAt(1).CustomPropertyName(),
            &AtomicString::from_str("--X")
        );
        assert_eq!(
            immutable
                .GetPropertyCSSValue(CSSPropertyID::kWidth)
                .unwrap()
                .CssText(),
            String::from("early important")
        );
        assert_eq!(
            immutable
                .GetPropertyCSSValue(&AtomicString::from_str("--x"))
                .unwrap()
                .CssText(),
            String::from("custom winner")
        );
        let mut mutable =
            MutableCSSPropertyValueSet::<StoreDispatch>::new(CSSParserMode::kHTMLStandardMode);
        assert_eq!(
            mutable.AddParsedProperties(&properties[unused..]),
            SetResult::kChangedPropertySet
        );
        assert_eq!(mutable.PropertyCount(), 4);
        assert!(mutable.PropertyIsImportant(CSSPropertyID::kWidth));
        assert!(mutable.PropertyIsImportant(&AtomicString::from_str("--x")));
    }
    #[test]
    fn declaration_filter_all_important_and_empty_vectors_keep_unsigned_partition_semantics() {
        let mut properties = vec![
            stored_property(CSSPropertyName::new(CSSPropertyID::kWidth), "first", true),
            stored_property(CSSPropertyName::new(CSSPropertyID::kWidth), "last", true),
        ];
        let mut unused = properties.len();
        FilterProperties(
            &mut properties,
            &mut unused,
            &mut HashSet::new(),
            &mut HashSet::new(),
        );
        assert_eq!(unused, 1);
        assert_eq!(properties[unused].Value().CssText(), String::from("last"));
        let mut empty: Vec<CSSPropertyValue<StoreDispatch>> = Vec::new();
        let mut unused = 0;
        FilterProperties(
            &mut empty,
            &mut unused,
            &mut HashSet::new(),
            &mut HashSet::new(),
        );
        assert_eq!(unused, 0);
    }
    #[test]
    fn custom_property_name_requires_one_valid_ident_and_decodes_real_tokenizer_escapes() {
        for (source, expected) in [
            ("--name", "--name"),
            ("--name  ", "--name"),
            (r"--n\61 me", "--name"),
            ("--名字", "--名字"),
        ] {
            assert_eq!(
                ParseCustomPropertyName(StringView::from(source)),
                String::FromUtf8(expected.as_bytes())
            );
        }
        for source in [
            "",
            "123",
            "!",
            "--",
            "name",
            " --name",
            "--name other",
            "--name!important",
            "--name;",
            "var(--name)",
        ] {
            assert!(
                ParseCustomPropertyName(StringView::from(source)).IsNull(),
                "{source}"
            );
        }
    }
    // These are already-classified rule metadata, not CSS parsing results.
    enum OrderingFixture {
        Charset,
        LayerStatement,
        Import,
        Namespace,
        Regular,
    }

    struct NoUrlModifiers;
    impl CSSUrlRequestModifiersConsumer<()> for NoUrlModifiers {
        type Modifiers = ();
        fn CSSURLRequestModifiersEnabled() -> bool {
            false
        }
        fn ConsumeUrlRequestModifiers<T: TokenStreamTokenizer>(
            _: &mut CSSParserTokenStream<'_, T>,
            _: &(),
            _: &mut Self::Modifiers,
        ) -> bool {
            unreachable!("disabled modifier parser must not be called")
        }
    }

    #[test]
    fn string_or_uri_preserves_function_block_and_fast_token_paths() {
        for (source, expected) in [
            ("\"plain\" ;", "plain"),
            ("url(asset.svg) ;", "asset.svg"),
            ("url(\"quoted.svg\") ;", "quoted.svg"),
        ] {
            let mut stream: CSSParserTokenStream<CSSTokenizer> =
                CSSParserTokenStream::new(StringView::from(source), 0);
            let parsed = ConsumeStringOrURI::<(), NoUrlModifiers, _>(&mut stream, &(), None);
            assert_eq!(parsed, AtomicString::from_str(expected));
            assert_eq!(stream.Peek().GetType(), kSemicolonToken);
        }

        let mut rejected: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from("image(\"x\")"), 0);
        assert!(ConsumeStringOrURI::<(), NoUrlModifiers, _>(&mut rejected, &(), None).IsNull());
    }

    #[test]
    fn parser_mode_selects_the_same_mutable_declaration_rule_type() {
        use crate::css_selector::{
            CSSSelector, CSSSelectorParserContext, MatchType, PseudoType, QualifiedName,
        };
        use crate::css_selector_list::CSSSelectorList;
        use crate::style_rule::RuleType;
        use crate::style_rule_keyframe::KeyframeOffset;
        assert_eq!(
            RuleTypeForParserMode(CSSParserMode::kCSSFontFaceRuleMode),
            RuleType::kFontFace
        );
        assert_eq!(
            RuleTypeForParserMode(CSSParserMode::kCSSCounterStyleRuleMode),
            RuleType::kCounterStyle
        );
        assert_eq!(
            RuleTypeForParserMode(CSSParserMode::kHTMLStandardMode),
            RuleType::kStyle
        );
    }
    impl CSSRuleOrdering for OrderingFixture {
        fn IsCharsetRule(&self) -> bool {
            matches!(self, Self::Charset)
        }
        fn IsLayerStatementRule(&self) -> bool {
            matches!(self, Self::LayerStatement)
        }
        fn IsImportRule(&self) -> bool {
            matches!(self, Self::Import)
        }
        fn IsNamespaceRule(&self) -> bool {
            matches!(self, Self::Namespace)
        }
    }
    #[test]
    fn ordering_preserves_early_layer_statements_then_closes_import_namespace_phase() {
        let mut seen = false;
        let mut allowed =
            ComputeNewAllowedRules(kTopLevelRules, Some(&OrderingFixture::Charset), &mut seen);
        assert!(!allowed.Has(CSSAtRuleID::kCSSAtRuleCharset));
        for _ in 0..3 {
            allowed =
                ComputeNewAllowedRules(allowed, Some(&OrderingFixture::LayerStatement), &mut seen);
        }
        assert!(!seen);
        assert!(
            allowed.Has(CSSAtRuleID::kCSSAtRuleImport)
                && allowed.Has(CSSAtRuleID::kCSSAtRuleNamespace)
        );
        allowed = ComputeNewAllowedRules(allowed, Some(&OrderingFixture::Import), &mut seen);
        assert!(seen);
        assert!(allowed.Has(CSSAtRuleID::kCSSAtRuleImport));
        allowed =
            ComputeNewAllowedRules(allowed, Some(&OrderingFixture::LayerStatement), &mut seen);
        assert!(
            !allowed.Has(CSSAtRuleID::kCSSAtRuleImport)
                && !allowed.Has(CSSAtRuleID::kCSSAtRuleNamespace)
        );
        let mut seen = false;
        let namespaces =
            ComputeNewAllowedRules(kTopLevelRules, Some(&OrderingFixture::Namespace), &mut seen);
        assert!(seen && namespaces.Has(CSSAtRuleID::kCSSAtRuleNamespace));
        assert!(!namespaces.Has(CSSAtRuleID::kCSSAtRuleImport));
        let regular =
            ComputeNewAllowedRules(kTopLevelRules, Some(&OrderingFixture::Regular), &mut false);
        assert!(regular.Has(QualifiedRuleType::kStyle));
        assert!(!regular.Has(CSSAtRuleID::kCSSAtRuleCharset));
        assert!(!regular.Has(CSSAtRuleID::kCSSAtRuleImport));
        assert!(!regular.Has(CSSAtRuleID::kCSSAtRuleNamespace));
        assert_eq!(
            ComputeNewAllowedRules::<OrderingFixture>(kTopLevelRules, None, &mut false),
            kTopLevelRules
        );
    }
    #[test]
    fn scope_specific_rule_sets_and_range_offsets_match_source() {
        assert!(kRegularRules.Has(QualifiedRuleType::kStyle));
        assert!(!kRegularRules.Has(QualifiedRuleType::kKeyframe));
        assert!(!kRegularRules.Has(CSSAtRuleID::kCSSAtRuleImport));
        assert!(kKeyframeRules.Has(QualifiedRuleType::kKeyframe));
        assert!(!kKeyframeRules.Has(QualifiedRuleType::kStyle));
        assert!(kNestedGroupRules.Has(CSSAtRuleID::kCSSAtRuleApplyMixin));
        assert!(!kRegularRules.Has(CSSAtRuleID::kCSSAtRuleApplyMixin));
        assert!(!kConditionalRules.Has(QualifiedRuleType::kStyle));
        assert!(kConditionalRules.Has(CSSAtRuleID::kCSSAtRuleNavigation));
        assert!(kFontFeatureRules.Has(CSSAtRuleID::kCSSAtRuleSwash));
        assert!(!kFontFeatureRules.Has(CSSAtRuleID::kCSSAtRuleFontFace));
        assert!(kPageMarginRules.Has(CSSAtRuleID::kCSSAtRuleTopLeftCorner));
        assert!(kPageMarginRules.Has(CSSAtRuleID::kCSSAtRuleRightBottom));
        assert_eq!(RangeOffset::Ignore(), RangeOffset { start: 0, end: 0 });
        assert_eq!(RangeOffset::new(3, 9), RangeOffset { start: 3, end: 9 });
    }
    #[test]
    fn layer_name_uses_real_tokenizer_and_restores_a_trailing_dot() {
        use super::super::css_parser_token::CSSParserTokenType::*;
        use super::super::css_tokenizer::CSSTokenizer;
        use foundation::StringView;
        let mut stream: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from("base.theme. next"), 0);
        stream.Peek();
        let names = ConsumeCascadeLayerName(&mut stream);
        assert_eq!(
            names,
            vec![
                AtomicString::from_str("base"),
                AtomicString::from_str("theme")
            ]
        );
        assert_eq!(stream.Peek().GetType(), kDelimiterToken);
        assert_eq!(stream.Peek().Delimiter(), b'.' as u16);
        assert_eq!(stream.Offset(), 10);
        let mut failure: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from(".theme"), 0);
        failure.Peek();
        let original = failure.Offset();
        assert!(ConsumeCascadeLayerName(&mut failure).is_empty());
        assert_eq!(failure.Offset(), original);
        assert_eq!(failure.Peek().Delimiter(), b'.' as u16);
        let mut whitespace: CSSParserTokenStream<CSSTokenizer> =
            CSSParserTokenStream::new(StringView::from("a.b  ;"), 0);
        whitespace.Peek();
        assert_eq!(ConsumeCascadeLayerName(&mut whitespace).len(), 2);
        assert_eq!(whitespace.Peek().GetType(), kSemicolonToken);
        assert_eq!(whitespace.Offset(), 5);
    }
}

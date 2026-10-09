// Copyright 2020 The Chromium Authors. All rights reserved.
// BSD-style license; see the LICENSE file.
// cpp: third_party/blink/renderer/core/css/resolver/style_cascade.h
// cpp: third_party/blink/renderer/core/css/resolver/style_cascade.cc
// Complete StyleCascade production logic, including token/variable/function
// substitution, attr/env/if, numeric coercion and safe-area processing.
// Source ledger (physical/effective/mapped/omitted/pending):
//   style_cascade.h:   868 /  406 /  340 /  66 / 0
//   style_cascade.cc: 3155 / 2405 / 2247 / 158 / 0
//   total:           4023 / 2811 / 2587 / 224 / 0
// Effective means nonblank source lines after removing comments, retaining
// declarations, delimiters and preprocessing. mapped + omitted + pending =
// effective. Batch 1 mapped 711; batch 2 adds 1300; batch 3 adds 576
// (h 49 + cc 527), and classifies the final 24 boilerplate/diagnostic lines
// as omitted. No production source remains pending.
// Mapped source is every effective line in h:71-864 and cc:85-3143 except
// the explicit omissions below. Required external class API adapters and the
// separate CascadeInterpolations dependency do not inflate this ledger.
// Omitted header: 5-6,8,10-31,33,35-49,51,53-54,56,72,74-76,78,80-81,
// 201-202,245,301,303,340,503,505,799,818-820,866,868.
// Omitted implementation: 5,7-8,10-79,81,83,89,111,114,156,172,186,282,
// 285,292,362,382-383,400,406,412,513,684,902,926,939,945,948,965,
// 981-982,990,996-997,1010,1019,1021,1171-1172,1185,1189,1343-1344,
// 1367,1369,1424-1425,1433-1434,1462-1463,1466,1542,1658,1757,1849,
// 1998,2105,2367-2368,2374-2375,2380,2430,2497,2555,2755-2756,2793,
// 2862,2864,2868,2870-2872,2906,3095-3097,3141,3145-3147,3149-3153,3155.
// Omissions are preprocessing/includes, namespace/forward/allocator/access/
// deleted-copy/alias boilerplate, test friendship, diagnostics and metrics.
// Header 245 ApplyAppearance and 799 ApplyReferencesSafeAreaInsetBottom are
// unused declarations without definitions in this source pair and have no
// production body to map; they are explicitly omitted, not fake methods.
// Free map lookup helpers cc:209-227 are integrated into the Rust HashMap
// get/cloned paths, retaining missing-vs-explicit-null variable shadowing.
// The container query helper cc:232-268 and local conditional visitor remain
// Rust control over required typed external DOM/query/AST collaborators.
#![allow(non_snake_case)]

use super::cascade_filter::CascadeFilter;
use super::cascade_map::CascadeMap;
use super::cascade_origin::CascadeOrigin;
use super::cascade_priority::CascadePriority;
use super::match_result::{
    MatchResult, MatchedProperties, MatchedPropertySet, MixinParameterBindings, TreeScope,
};
use super::style_resolver_state::{ResolverValue, StyleResolverState, StyleResolverStateBackend};
use crate::css_primitive_value::UnitType;
use crate::css_property_name::CSSPropertyName;
use crate::kleene_value::{KleeneAnd, KleeneValue};
use crate::media_queries::media_query_exp::{
    MediaQueryExpBounds, MediaQueryExpValue, MediaQueryOperator,
};
use crate::parser::css_parser_context::{CSSParserContext, CSSParserContextPlatform};
use crate::parser::css_parser_local_context::CSSParserLocalContext;
use crate::parser::css_parser_token::{
    BlockType, CSSParserToken, CSSParserTokenType, NeedsInsertedComment,
};
use crate::parser::css_parser_token_stream::{BlockGuard, CSSParserTokenStream};
use crate::parser::css_tokenizer::CSSTokenizer;
use crate::properties::css_property::{CSSProperty, Flags};
use foundation::Length;
use foundation::{
    gfx, AtomicString, CSSBitset, CSSPropertyID, ConvertToCSSPropertyID, EOverflow, EPosition,
    EScrollbarWidth, WritingDirectionMode, WritingMode,
};
use foundation::{CSSValueID, String, StringView};
use layoutng_style::style::appearance::AppearanceValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use CSSParserTokenType::*;
type ParserContext<B> = CSSParserContext<<B as StyleCascadeBackend>::ParserPlatform>;
type TokenStream<'a> = CSSParserTokenStream<'a, CSSTokenizer>;
type Variable<B> = Rc<<B as StyleCascadeBackend>::VariableData>;
type VariableMap<B> = HashMap<AtomicString, Option<Variable<B>>>;
type Frame<B> = Rc<RefCell<FunctionContext<B>>>;

/// Genuine external class APIs: CSSProperty/CustomProperty metadata, the
/// separate ExpandCascade helper, CascadeResolver, DOM/layout and animation
/// interpolation objects. Collection priorities and application decisions stay
/// in StyleCascade. Every operation is required, with no default implementation.
pub trait StyleCascadeBackend: StyleResolverStateBackend {
    // Actual external CSSVariableData, CSSSyntaxDefinition and parser context.
    type VariableData;
    type Syntax;
    type ParserPlatform: CSSParserContextPlatform;
    type StyleRuleFunction;
    type Rule;
    type RuleGroup;
    type ContainerQuerySet;
    type ResolverCycleNode;
    type ResolverLock;
    type AnchorScope;
    fn VariableText(&self, data: &Self::VariableData) -> String;
    fn VariableFeatures(&self, data: &Self::VariableData) -> u32;
    fn VariableNeedsResolution(&self, data: &Self::VariableData) -> bool;
    fn VariableAnimationTainted(&self, data: &Self::VariableData) -> bool;
    fn VariableAttrTainted(&self, data: &Self::VariableData) -> bool;
    fn ExtractVariableFeatures(&self, token: &CSSParserToken) -> u32;
    fn NewVariableData(
        &self,
        text: String,
        animation_tainted: bool,
        attr_tainted: bool,
        features: u32,
    ) -> Variable<Self>;
    fn StripTrailingWhitespaceAndComments(&self, text: StringView) -> StringView;
    fn UnparsedData(&self, value: &ResolverValue<Self>) -> Variable<Self>;
    fn UnparsedParserContext(&self, value: &ResolverValue<Self>)
        -> Option<Rc<ParserContext<Self>>>;
    fn StrictParserContext(&self, document: &Self::Document) -> Rc<ParserContext<Self>>;
    fn PendingShorthand(
        &self,
        value: &ResolverValue<Self>,
    ) -> (CSSPropertyID, Rc<ResolverValue<Self>>);
    fn NewUnparsedValue(
        &self,
        data: Variable<Self>,
        context: Option<Rc<ParserContext<Self>>>,
    ) -> Rc<ResolverValue<Self>>;
    fn NewCyclicVariableValue(&self) -> Rc<ResolverValue<Self>>;
    fn NewInvalidVariableValue(&self) -> Rc<ResolverValue<Self>>;
    fn ConsumeCSSWideKeyword(
        &self,
        stream: &mut TokenStream<'_>,
        context: &ParserContext<Self>,
    ) -> Option<Rc<ResolverValue<Self>>>;
    fn ParseSingleValue(
        &self,
        id: CSSPropertyID,
        stream: &mut TokenStream<'_>,
        context: &ParserContext<Self>,
    ) -> Option<Rc<ResolverValue<Self>>>;
    fn ParseShorthand(
        &self,
        id: CSSPropertyID,
        stream: &mut TokenStream<'_>,
        context: Option<&ParserContext<Self>>,
    ) -> Option<Vec<(CSSPropertyID, Rc<ResolverValue<Self>>)>>;
    fn ShorthandCacheValue(
        &self,
        resolver: &Self::CascadeResolver,
    ) -> Option<Rc<ResolverValue<Self>>>;
    fn ShorthandCacheProperties(
        &self,
        resolver: &Self::CascadeResolver,
    ) -> Vec<(CSSPropertyID, Rc<ResolverValue<Self>>)>;
    fn SetShorthandCache(
        &self,
        resolver: &mut Self::CascadeResolver,
        value: Rc<ResolverValue<Self>>,
        properties: Vec<(CSSPropertyID, Rc<ResolverValue<Self>>)>,
    );
    fn PropertyCycleNode(&self, name: &CSSPropertyName) -> Self::ResolverCycleNode;
    fn FunctionCycleNode(&self, function: &Self::StyleRuleFunction) -> Self::ResolverCycleNode;
    fn LocalCycleNode(
        &self,
        name: &AtomicString,
        function: Option<&Self::StyleRuleFunction>,
    ) -> Self::ResolverCycleNode;
    fn ResolverDetectCycle(
        &self,
        resolver: &mut Self::CascadeResolver,
        node: &Self::ResolverCycleNode,
    ) -> bool;
    fn ResolverInCycle(&self, resolver: &Self::CascadeResolver) -> bool;
    fn ResolverLock(
        &self,
        resolver: &mut Self::CascadeResolver,
        node: Self::ResolverCycleNode,
    ) -> Self::ResolverLock;
    fn ResolverUnlock(&self, resolver: &mut Self::CascadeResolver, lock: Self::ResolverLock);
    fn ResolverAllowSubstitution(
        &self,
        resolver: &Self::CascadeResolver,
        data: Option<&Self::VariableData>,
    ) -> bool;
    fn ResolverNextFunctionInvocationCount(&self, resolver: &mut Self::CascadeResolver) -> usize;
    fn ResolverFunctionInvocationCount(&self, resolver: &Self::CascadeResolver) -> usize;
    fn ResolverRandomValueCount(&self, resolver: &Self::CascadeResolver) -> usize;
    fn ResolverSetRandomValueCount(&self, resolver: &mut Self::CascadeResolver, count: usize);
    fn PropertyRegistrationSyntax(
        &self,
        document: &Self::Document,
        name: &AtomicString,
    ) -> Option<Rc<Self::Syntax>>;
    fn SyntaxIsUniversal(&self, syntax: &Self::Syntax) -> bool;
    fn SyntaxParse(
        &self,
        syntax: &Self::Syntax,
        text: String,
        context: &ParserContext<Self>,
        local: &mut CSSParserLocalContext,
        animation_tainted: bool,
    ) -> Option<Rc<ResolverValue<Self>>>;
    fn ConvertRegisteredPropertyValue(
        &self,
        state: &StyleResolverState<'_, Self>,
        value: Rc<ResolverValue<Self>>,
        context: &ParserContext<Self>,
    ) -> Rc<ResolverValue<Self>>;
    fn ConvertRegisteredPropertyVariableData(
        &self,
        value: &ResolverValue<Self>,
        animation_tainted: bool,
        attr_tainted: bool,
    ) -> Variable<Self>;
    fn BuilderVariableData(
        &self,
        state: &StyleResolverState<'_, Self>,
        name: &AtomicString,
        inherited: bool,
    ) -> Option<Variable<Self>>;
    fn BuilderInitialData(
        &self,
        state: &StyleResolverState<'_, Self>,
    ) -> Option<Rc<Self::StyleInitialData>>;
    fn InitialDataVariableData(
        &self,
        initial: &Self::StyleInitialData,
        name: &AtomicString,
    ) -> Option<Variable<Self>>;
    fn ParentSetChildHasExplicitInheritance(&self, state: &StyleResolverState<'_, Self>);
    fn ParentVariableData(
        &self,
        state: &StyleResolverState<'_, Self>,
        name: &AtomicString,
    ) -> Option<Variable<Self>>;
    fn MarkPropertyRegistryReferenced(&self, document: &Self::Document, name: &AtomicString);
    fn IsValidVariableName(&self, name: StringView) -> bool;
    fn ConsumeIdentFunction(
        &self,
        stream: &mut TokenStream<'_>,
        context: &ParserContext<Self>,
    ) -> Rc<ResolverValue<Self>>;
    fn ComputeIdent(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> AtomicString;
    fn CSSFunctionsEnabled(&self) -> bool;
    fn FlipRevertPropertyID(&self, value: &ResolverValue<Self>) -> CSSPropertyID;
    fn FlipRevertedValue(
        &self,
        from: CSSPropertyID,
        value: Rc<ResolverValue<Self>>,
        flip: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> Rc<ResolverValue<Self>>;
    fn EnterAnchorScope(
        &self,
        property: CSSPropertyID,
        state: &StyleResolverState<'_, Self>,
    ) -> Self::AnchorScope;
    fn MathHasInvalidAnchorFunctions(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> bool;
    fn LeaveAnchorScope(&self, scope: Self::AnchorScope);
    fn MixinParent(
        &self,
        bindings: &dyn MixinParameterBindings,
    ) -> Option<Rc<dyn MixinParameterBindings>>;
    fn MixinParameters(
        &self,
        bindings: &dyn MixinParameterBindings,
    ) -> Vec<(
        AtomicString,
        Option<Variable<Self>>,
        Option<Variable<Self>>,
        Rc<Self::Syntax>,
    )>;
    fn MixinBaseLocals(&self, bindings: &dyn MixinParameterBindings) -> VariableMap<Self>;
    fn MixinConditionalLocals(
        &self,
        bindings: &dyn MixinParameterBindings,
    ) -> Vec<(
        AtomicString,
        Vec<(Rc<Self::ContainerQuerySet>, Variable<Self>)>,
    )>;
    fn FindFunctionAcrossScopes(
        &self,
        document: &Self::Document,
        name: &AtomicString,
        scope: Option<&dyn TreeScope>,
    ) -> Option<(Rc<Self::StyleRuleFunction>, Rc<dyn TreeScope>)>;
    fn FunctionName(&self, function: &Self::StyleRuleFunction) -> AtomicString;
    fn FunctionParameters(
        &self,
        function: &Self::StyleRuleFunction,
    ) -> Vec<(AtomicString, Rc<Self::Syntax>, Option<Variable<Self>>)>;
    fn FunctionReturnType(&self, function: &Self::StyleRuleFunction) -> Rc<Self::Syntax>;
    fn ConsumeFunctionArguments(
        &self,
        stream: &mut TokenStream<'_>,
        parameter_count: usize,
    ) -> Vec<String>;
    fn FunctionAsGroup(&self, function: &Self::StyleRuleFunction) -> Rc<Self::RuleGroup>;
    fn RuleChildren(&self, group: &Self::RuleGroup) -> Vec<Rc<Self::Rule>>;
    fn FunctionRuleKind(&self, rule: &Self::Rule) -> FunctionRuleKind;
    fn FunctionRuleGroup(&self, rule: &Self::Rule) -> Rc<Self::RuleGroup>;
    fn FunctionRuleDeclarations(&self, rule: &Self::Rule)
        -> Vec<(CSSPropertyName, Variable<Self>)>;
    fn SupportsConditionIsSupported(&self, rule: &Self::Rule) -> bool;
    fn EvaluateFunctionalMediaRule(&self, rule: &Self::Rule, document: &Self::Document) -> bool;
    fn ContainerRuleQueries(&self, rule: &Self::Rule) -> Rc<Self::ContainerQuerySet>;
    fn EvaluateNavigationRule(&self, rule: &Self::Rule, document: &Self::Document) -> bool;
    type AttrType;
    type ShadowRoot;
    type ContainerQuery;
    type ContainerSelector;
    type ContainerChange;
    type MediaQueryFeatureExpNode;
    type ConditionalExpNode;
    type NavigationExpNode;
    type NavigationTest;
    type MediaQuerySet;
    fn AttributeCycleNode(&self, name: &AtomicString) -> Self::ResolverCycleNode;
    fn CSSArgumentGrammarEnabled(&self) -> bool;
    fn ConsumeAttrType(&self, stream: &mut TokenStream<'_>) -> Option<Self::AttrType>;
    fn DefaultAttrType(&self) -> Self::AttrType;
    fn AttrTypeIsSyntax(&self, attr_type: &Self::AttrType) -> bool;
    fn ParseAttrType(
        &self,
        attr_type: &Self::AttrType,
        value: String,
        context: &ParserContext<Self>,
        local: &mut CSSParserLocalContext,
    ) -> Option<Rc<ResolverValue<Self>>>;
    fn ElementLowercaseIfNecessary(
        &self,
        element: &Self::Element,
        name: AtomicString,
    ) -> AtomicString;
    fn ElementGetAttributeNS(
        &self,
        element: &Self::Element,
        namespace: &AtomicString,
        name: &AtomicString,
    ) -> String;
    fn ParseVariableDeclarationValue(
        &self,
        value: String,
        animation_tainted: bool,
        context: &ParserContext<Self>,
    ) -> Option<Variable<Self>>;
    fn ElementSupportsBaseAppearance(
        &self,
        element: &Self::Element,
        appearance: AppearanceValue,
    ) -> bool;
    fn SetNeedsToUpdateComplexSafeAreaConstraints(&self, document: &Self::Document);
    fn ElementTreeScopeRootShadowRoot(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::ShadowRoot>>;
    fn ShadowRootIsUserAgent(&self, root: &Self::ShadowRoot) -> bool;
    fn ResolveEnvironmentVariable(
        &self,
        document: &Self::Document,
        name: &AtomicString,
        indices: Vec<u32>,
        record_metrics: bool,
    ) -> Option<Variable<Self>>;
    fn ContainerQueries(&self, queries: &Self::ContainerQuerySet) -> Vec<Rc<Self::ContainerQuery>>;
    fn QuerySelector(&self, query: &Self::ContainerQuery) -> Rc<Self::ContainerSelector>;
    fn SelectorSelectsAnyContainer(&self, selector: &Self::ContainerSelector) -> bool;
    fn QuerySetDependencyFlags(&self, query: &Self::ContainerQuery, result: &mut MatchResult);
    fn QueryDetermineStartingElement(
        &self,
        element: &Self::Element,
        pseudo: layoutng_style::style::computed_style_constants::PseudoId,
        selector: &Self::ContainerSelector,
        nearest_size_container: Option<&Self::Element>,
    ) -> Option<Rc<Self::Element>>;
    fn QueryFindContainer(
        &self,
        starting: Option<&Self::Element>,
        selector: &Self::ContainerSelector,
        scope: Option<&dyn TreeScope>,
    ) -> Option<Rc<Self::Element>>;
    fn NearestContainerChange(&self) -> Self::ContainerChange;
    fn DescendantContainersChange(&self) -> Self::ContainerChange;
    fn QueryEvalAndAdd(
        &self,
        container: &Self::Element,
        query: &Self::ContainerQuery,
        change: Self::ContainerChange,
        result: &mut MatchResult,
    ) -> bool;
    fn NumericSyntaxParse(
        &self,
        text: String,
        context: &ParserContext<Self>,
        local: &mut CSSParserLocalContext,
        animation_tainted: bool,
        attr_tainted: bool,
    ) -> Option<Rc<ResolverValue<Self>>>;
    fn ResolverCurrentPropertyName(
        &self,
        resolver: &Self::CascadeResolver,
    ) -> Option<CSSPropertyName>;
    fn PrimitiveIsCalculated(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsPx(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsPercentage(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsLength(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsResolvableBeforeLayout(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsNumber(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsAngle(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsTime(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsResolution(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveConvertToUnzoomedLength(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> Length;
    fn PrimitiveCreateFromLength(&self, length: Length, zoom: f32) -> Rc<ResolverValue<Self>>;
    fn PrimitiveComputeNumber(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64;
    fn PrimitiveComputeDegrees(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64;
    fn PrimitiveComputeSeconds(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64;
    fn PrimitiveComputeDotsPerPixel(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64;
    fn NewNumericLiteralValue(&self, value: f64, unit: UnitType) -> Rc<ResolverValue<Self>>;
    fn FeatureBounds(
        &self,
        feature: &Self::MediaQueryFeatureExpNode,
    ) -> MediaQueryExpBounds<ResolverValue<Self>>;
    fn FeatureName(&self, feature: &Self::MediaQueryFeatureExpNode) -> AtomicString;
    fn FeatureReferenceValue(
        &self,
        feature: &Self::MediaQueryFeatureExpNode,
    ) -> Rc<ResolverValue<Self>>;
    fn EvalStyleRange(
        &self,
        reference: &ResolverValue<Self>,
        bound: &ResolverValue<Self>,
        op: MediaQueryOperator,
        left: bool,
    ) -> KleeneValue;
    fn VariableEqualsIgnoringAttrTainting(
        &self,
        left: &Self::VariableData,
        right: &Self::VariableData,
    ) -> bool;
    fn ConsumeIfCondition(
        &self,
        context: &ParserContext<Self>,
        stream: &mut TokenStream<'_>,
    ) -> Rc<Self::ConditionalExpNode>;
    fn EvaluateIfExpression(
        &self,
        expression: &Self::ConditionalExpNode,
        visitor: &mut dyn IfConditionVisitor<Self>,
    ) -> KleeneValue;
    fn NavigationNodeTest(&self, node: &Self::NavigationExpNode) -> Rc<Self::NavigationTest>;
    fn SetNeedsStyleUpdateOnNavigation(&self, document: &Self::Document);
    fn EvaluateFunctionalNavigationQuery(
        &self,
        document: &Self::Document,
        test: &Self::NavigationTest,
    ) -> bool;
    fn EvaluateFunctionalMediaQuery(
        &self,
        document: &Self::Document,
        query: &Self::MediaQuerySet,
    ) -> bool;
    fn IsSimpleSum(&self, text: String) -> bool;
    type CascadeResolver;
    type ActiveInterpolationsMap;
    type ActiveInterpolations;
    type PropertyHandle;
    type ScrollbarGutter;
    fn NewCascadeResolver(&self, filter: CascadeFilter) -> Self::CascadeResolver;
    fn ResolverFilter(&self, resolver: &Self::CascadeResolver) -> CascadeFilter;
    fn ResolverRejects(&self, resolver: &mut Self::CascadeResolver, property: &CSSProperty)
        -> bool;
    fn ResolverCollectFlags(
        &self,
        resolver: &mut Self::CascadeResolver,
        property: &CSSProperty,
        origin: CascadeOrigin,
    );
    fn ResolverFlags(&self, resolver: &Self::CascadeResolver) -> Flags;
    fn ResolverAuthorFlags(&self, resolver: &Self::CascadeResolver) -> Flags;
    fn ResolverRejectedFlags(&self, resolver: &Self::CascadeResolver) -> Flags;
    fn CustomPropertyMetadata(&self, name: &AtomicString, document: &Self::Document)
        -> CSSProperty;
    fn SurrogateFor(&self, property: &CSSProperty, direction: WritingDirectionMode) -> CSSProperty;
    // Calls the distinct cascade_expansion.h ExpandCascade, whose output uses
    // the existing CascadePriority and CSSPropertyName rather than a new model.
    fn ExpandCascade(
        &self,
        properties: &MatchedProperties,
        document: &Self::Document,
        index: usize,
    ) -> Vec<(CascadePriority, CSSPropertyName)>;
    fn PropertyValueAt(
        &self,
        set: &dyn MatchedPropertySet,
        declaration: usize,
    ) -> Rc<ResolverValue<Self>>;
    fn InterpolationEntries(
        &self,
        map: &Self::ActiveInterpolationsMap,
    ) -> Vec<(CSSPropertyName, Rc<Self::ActiveInterpolations>)>;
    fn NativePropertyHandle(&self, property: &CSSProperty) -> Self::PropertyHandle;
    fn CustomPropertyHandle(&self, name: &AtomicString) -> Self::PropertyHandle;
    fn FindInterpolation(
        &self,
        map: &Self::ActiveInterpolationsMap,
        handle: &Self::PropertyHandle,
    ) -> Option<Rc<Self::ActiveInterpolations>>;
    fn FirstInterpolationIsInvalidatable(
        &self,
        interpolations: &Self::ActiveInterpolations,
    ) -> bool;
    // These two external animation operations construct their real
    // InterpolationTypesMap/CSSInterpolationEnvironment from the supplied
    // state/cascade/resolver and apply their own stack or transition object.
    fn ApplyInvalidatableInterpolationStack(
        &self,
        interpolations: &Self::ActiveInterpolations,
        state: &StyleResolverState<'_, Self>,
        cascade: &mut StyleCascade<'_, '_, Self>,
        resolver: &mut Self::CascadeResolver,
    );
    fn ApplyTransitionInterpolation(
        &self,
        interpolations: &Self::ActiveInterpolations,
        state: &StyleResolverState<'_, Self>,
        cascade: &mut StyleCascade<'_, '_, Self>,
        resolver: &mut Self::CascadeResolver,
    );
    fn ApplyPhysicalProperty(
        &self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
    );
    fn EnsureScopedValue(
        &self,
        value: Rc<ResolverValue<Self>>,
        scope: Option<&dyn TreeScope>,
    ) -> Rc<ResolverValue<Self>>;
    fn NewUnsetValue(&self) -> Rc<ResolverValue<Self>>;
    fn MathValueHasAnchorFunctions(&self, value: &ResolverValue<Self>) -> bool;
    fn UnparsedContainsSafeAreaInsetBottom(&self, value: &ResolverValue<Self>) -> bool;
    fn DocumentTreeScope(&self, document: &Self::Document) -> Rc<dyn TreeScope>;
    fn DocumentElement(&self, document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn StandardizedBrowserZoomEnabled(&self, document: &Self::Document) -> bool;
    fn SmallerViewportUnitsEnabled(&self) -> bool;
    fn BuilderScrollbarGutter(
        &self,
        builder: &layoutng_style::style::computed_style::ComputedStyleBuilder,
    ) -> Self::ScrollbarGutter;
    fn ComputeScrollbarWidthsForViewportUnits(
        &self,
        document: &Self::Document,
        overflow_x: EOverflow,
        overflow_y: EOverflow,
        gutter: Self::ScrollbarGutter,
        width: EScrollbarWidth,
        writing_mode: WritingMode,
    ) -> gfx::Size;
    fn BorderImageLonghands(&self) -> &[CSSPropertyID];
    fn CSSLineClampEnabled(&self) -> bool;
    fn CSSLineClampAsShorthandEnabled(&self) -> bool;
}

// cpp dependency: resolver/cascade_interpolations.h:22-62. These borrowed map
// references retain the source's requirement that incoming maps outlive cascade.
struct CascadeInterpolationEntry<'a, M> {
    map: &'a M,
    origin: CascadeOrigin,
}
struct CascadeInterpolations<'a, M> {
    entries: Vec<CascadeInterpolationEntry<'a, M>>,
}
impl<'a, M> CascadeInterpolations<'a, M> {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(4),
        }
    }
    fn Add(&mut self, map: &'a M, origin: CascadeOrigin) {
        assert!(
            self.entries.len() < 4,
            "CascadeInterpolations source limit is four maps"
        );
        self.entries.push(CascadeInterpolationEntry { map, origin });
    }
    fn Reset(&mut self) {
        self.entries.clear();
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionRuleKind {
    Declarations,
    Supports,
    Media,
    Container,
    Navigation,
    Other,
}
// cpp: style_cascade.h:300-373; cc:1033-1175.
pub struct TokenSequence {
    last_token: CSSParserToken,
    last_non_whitespace_token: CSSParserToken,
    original_text: String,
    animation_tainted: bool,
    features: u32,
    attr_taint_ranges: Vec<(u32, u32)>,
}
impl TokenSequence {
    pub fn new() -> Self {
        Self {
            last_token: CSSParserToken::new(kEOFToken, BlockType::kNotBlock),
            last_non_whitespace_token: CSSParserToken::new(kEOFToken, BlockType::kNotBlock),
            original_text: String::new(),
            animation_tainted: false,
            features: 0,
            attr_taint_ranges: Vec::new(),
        }
    }
    fn FromData<B: StyleCascadeBackend>(b: &B, data: &B::VariableData) -> Self {
        let mut out = Self::new();
        out.animation_tainted = b.VariableAnimationTainted(data);
        out.features = b.VariableFeatures(data) & !(1 << 4);
        out
    }
    pub fn IsAnimationTainted(&self) -> bool {
        self.animation_tainted
    }
    pub fn OriginalText(&self) -> String {
        self.original_text.clone()
    }
    pub fn GetAttrTaintedRanges(&self) -> &[(u32, u32)] {
        &self.attr_taint_ranges
    }
    fn NonWhitespace(token: &CSSParserToken) -> bool {
        !matches!(token.GetType(), kWhitespaceToken | kCommentToken)
    }
    fn AppendText<B: StyleCascadeBackend>(
        &mut self,
        b: &B,
        text: StringView,
        tainted: bool,
        limit: usize,
    ) -> bool {
        if self.original_text.length() as usize + text.length() as usize > limit {
            return false;
        }
        let start = self.original_text.length();
        let mut tokenizer = CSSTokenizer::new(text.clone(), 0);
        let first = tokenizer.TokenizeSingleWithComments();
        if !first.IsEOF() {
            self.features |= b.ExtractVariableFeatures(&first);
            if NeedsInsertedComment(&self.last_token, &first) {
                self.original_text.push_str("/**/");
            }
            self.last_token = first.CopyWithoutValue();
            if Self::NonWhitespace(&first) {
                self.last_non_whitespace_token = first;
            }
            loop {
                let token = tokenizer.TokenizeSingleWithComments();
                if token.IsEOF() {
                    break;
                }
                self.features |= b.ExtractVariableFeatures(&token);
                self.last_token = token.CopyWithoutValue();
                if Self::NonWhitespace(&token) {
                    self.last_non_whitespace_token = token;
                }
            }
        }
        self.original_text.push_string(&text.ToString());
        if tainted {
            self.attr_taint_ranges
                .push((start, self.original_text.length()));
        }
        true
    }
    fn AppendValue<B: StyleCascadeBackend>(
        &mut self,
        b: &B,
        value: &ResolverValue<B>,
        tainted: bool,
        limit: usize,
    ) -> bool {
        self.AppendText(b, StringView::from(&value.CssText()), tainted, limit)
    }
    fn AppendData<B: StyleCascadeBackend>(
        &mut self,
        b: &B,
        data: &B::VariableData,
        tainted: bool,
        limit: usize,
    ) -> bool {
        if !self.AppendText(b, StringView::from(&b.VariableText(data)), tainted, limit) {
            return false;
        }
        self.animation_tainted |= b.VariableAnimationTainted(data);
        true
    }
    fn AppendToken<B: StyleCascadeBackend>(
        &mut self,
        b: &B,
        token: &CSSParserToken,
        tainted: bool,
        text: StringView,
    ) {
        self.features |= b.ExtractVariableFeatures(token);
        let start = self.original_text.length();
        if NeedsInsertedComment(&self.last_token, token) {
            self.original_text.push_str("/**/");
        }
        self.last_token = token.CopyWithoutValue();
        if Self::NonWhitespace(token) {
            self.last_non_whitespace_token = token.clone();
        }
        self.original_text.push_string(&text.ToString());
        if tainted {
            self.attr_taint_ranges
                .push((start, self.original_text.length()));
        }
    }
    fn AppendSequence<B: StyleCascadeBackend>(
        &mut self,
        b: &B,
        sequence: &TokenSequence,
        tainted: bool,
        limit: usize,
    ) -> bool {
        if !self.AppendText(
            b,
            StringView::from(&sequence.original_text),
            tainted || !sequence.attr_taint_ranges.is_empty(),
            limit,
        ) {
            return false;
        }
        self.animation_tainted |= sequence.animation_tainted;
        true
    }
    fn AppendFallback<B: StyleCascadeBackend>(
        &mut self,
        b: &B,
        sequence: &TokenSequence,
        tainted: bool,
        limit: usize,
    ) -> bool {
        if self.original_text.length() as usize + sequence.original_text.length() as usize > limit {
            return false;
        }
        let start = self.original_text.length();
        let text = StringView::from(&sequence.original_text);
        if text.IsEmpty() {
            return true;
        }
        let text = b.StripTrailingWhitespaceAndComments(text);
        let mut tokenizer = CSSTokenizer::new(text.clone(), 0);
        let first = tokenizer.TokenizeSingleWithComments();
        if NeedsInsertedComment(&self.last_token, &first) {
            self.original_text.push_str("/**/");
        }
        self.original_text.push_string(&text.ToString());
        self.last_token = sequence.last_non_whitespace_token.clone();
        self.last_non_whitespace_token = sequence.last_non_whitespace_token.clone();
        self.animation_tainted |= sequence.animation_tainted;
        self.features |= sequence.features;
        if tainted {
            self.attr_taint_ranges
                .push((start, self.original_text.length()));
        }
        true
    }
    fn BuildVariableData<B: StyleCascadeBackend>(&self, b: &B) -> Variable<B> {
        b.NewVariableData(
            self.original_text.clone(),
            self.animation_tainted,
            !self.attr_taint_ranges.is_empty(),
            self.features,
        )
    }
}
// cpp: style_cascade.h:502-541. Owned handles preserve the source call-stack
// lifetime while allowing reentrant resolution of locals in the same frame.
struct FunctionContext<B: StyleCascadeBackend> {
    function: Option<Rc<B::StyleRuleFunction>>,
    tree_scope: Option<Rc<dyn TreeScope>>,
    arguments: VariableMap<B>,
    locals: VariableMap<B>,
    unresolved_locals: VariableMap<B>,
    local_types: HashMap<AtomicString, Rc<B::Syntax>>,
    parent: Option<Frame<B>>,
    invocation_count: usize,
}
// cpp: style_cascade.h:822-825,854,856-857,861,863-864
pub struct StyleCascade<'a, 's, B: StyleCascadeBackend> {
    state_: &'a StyleResolverState<'s, B>,
    backend_: &'a B,
    match_result_: MatchResult,
    interpolations_: CascadeInterpolations<'a, B::ActiveInterpolationsMap>,
    map_: CascadeMap,
    has_applied_: bool,
    needs_collect_from_match_result_: bool,
    needs_collect_from_interpolations_: bool,
    depends_on_cascade_affecting_property_: bool,
    effective_zoom_changed_: bool,
}
impl<'a, 's, B: StyleCascadeBackend> StyleCascade<'a, 's, B> {
    // cpp: style_cascade.h:79
    pub fn new(state: &'a StyleResolverState<'s, B>, backend: &'a B) -> Self {
        Self {
            state_: state,
            backend_: backend,
            match_result_: MatchResult::default(),
            interpolations_: CascadeInterpolations::new(),
            map_: CascadeMap::new(),
            has_applied_: false,
            needs_collect_from_match_result_: false,
            needs_collect_from_interpolations_: false,
            depends_on_cascade_affecting_property_: false,
            effective_zoom_changed_: false,
        }
    }
    pub fn GetMatchResult(&self) -> &MatchResult {
        &self.match_result_
    }
    // cpp: style_cascade.cc:284-288
    pub fn MutableMatchResult(&mut self) -> &mut MatchResult {
        self.needs_collect_from_match_result_ = true;
        &mut self.match_result_
    }
    // cpp: style_cascade.cc:290-295
    pub fn AddInterpolations(
        &mut self,
        map: &'a B::ActiveInterpolationsMap,
        origin: CascadeOrigin,
    ) {
        self.needs_collect_from_interpolations_ = true;
        self.interpolations_.Add(map, origin);
    }
    // cpp: style_cascade.cc:297-334
    pub fn Apply(&mut self, filter: CascadeFilter) {
        self.CollectDeclarationsIfNeeded();
        self.state_.InvalidateLengthConversionData();
        if self.has_applied_ {
            self.map_.ClearAppliedFlags();
        }
        self.has_applied_ = true;
        let b = self.backend_;
        let mut resolver = b.NewCascadeResolver(filter);
        self.ApplyCascadeAffecting(&mut resolver);
        self.ApplyViewportUnitAffecting(&mut resolver);
        self.ApplyHighPriority(&mut resolver);
        self.state_.UpdateFont();
        if self.map_.NativeBitset().Has(CSSPropertyID::kLineHeight) {
            self.LookupAndApplyNative(CSSPropertyID::kLineHeight, &mut resolver);
        }
        self.state_.UpdateLineHeight();
        self.ApplyWideOverlapping(&mut resolver);
        self.ApplyMatchResult(&mut resolver);
        self.ApplyInterpolations(&mut resolver);
        self.state_
            .SetComputedStyleFlagsFromAuthorFlags(b.ResolverAuthorFlags(&resolver));
        if b.ResolverFlags(&resolver) & CSSProperty::kAnimation != 0 {
            self.state_.StyleBuilderMut().SetCanAffectAnimations();
        }
        if b.ResolverRejectedFlags(&resolver) & CSSProperty::kLegacyOverlapping != 0 {
            self.state_.SetRejectedLegacyOverlapping();
        }
        self.ApplyUnresolvedEnv(&mut resolver);
    }
    // cpp: style_cascade.cc:336-339
    pub fn ReleaseImportantSet(&mut self) -> Option<CSSBitset> {
        self.CollectDeclarationsIfNeeded();
        self.map_.ReleaseImportantSet()
    }
    // cpp: style_cascade.h:122
    pub fn InlineStyleLost(&self) -> bool {
        self.map_.InlineStyleLost()
    }
    // cpp: style_cascade.cc:341-347
    pub fn Reset(&mut self) {
        self.map_.Reset();
        self.match_result_.Reset();
        self.interpolations_.Reset();
        self.has_applied_ = false;
        self.depends_on_cascade_affecting_property_ = false;
    }
    // cpp: style_cascade.cc:349-378; h:148-153
    pub fn Resolve(
        &mut self,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        tree_scope: Option<Rc<dyn TreeScope>>,
        mixin: Option<Rc<dyn MixinParameterBindings>>,
        origin: CascadeOrigin,
        resolver: &mut B::CascadeResolver,
    ) -> Option<Rc<ResolverValue<B>>> {
        let property = self.PropertyMetadata(name);
        let property = self.ResolveSurrogate(property);
        let mut origin = origin;
        let resolved = self.ResolveValue(
            &property,
            name,
            value,
            tree_scope,
            mixin,
            CascadePriority::FromOrigin(origin),
            &mut origin,
            resolver,
        );
        if resolved.IsCyclicVariableValue() {
            return None;
        }
        if resolved.IsInvalidVariableValue() {
            return Some(self.backend_.NewUnsetValue());
        }
        Some(resolved)
    }
    // cpp: style_cascade.cc:380-417
    pub fn GetCascadedValues(&self) -> HashMap<CSSPropertyName, Rc<ResolverValue<B>>> {
        let mut result = HashMap::new();
        for id in self.map_.NativeBitset().begin() {
            let name = CSSPropertyName::new(id);
            let priority = self.map_.At(&name);
            if Self::IsInterpolation(priority) || !priority.HasOrigin() {
                continue;
            }
            result.insert(name, self.ValueAt(priority));
        }
        for name in self.map_.GetCustomMap().keys() {
            let name = CSSPropertyName::custom(name.clone());
            let priority = self.map_.At(&name);
            if Self::IsInterpolation(priority) {
                continue;
            }
            result.insert(name, self.ValueAt(priority));
        }
        result
    }
    // cpp: style_cascade.cc:419-440. Static state-local resolver overload.
    pub fn ResolveForState(
        state: &'a StyleResolverState<'s, B>,
        backend: &'a B,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        mixin: Option<Rc<dyn MixinParameterBindings>>,
    ) -> Option<Rc<ResolverValue<B>>> {
        let mut cascade = Self::new(state, backend);
        let mut resolver = backend.NewCascadeResolver(CascadeFilter::new());
        cascade.Resolve(
            name,
            value,
            scope,
            mixin,
            CascadeOrigin::kNone,
            &mut resolver,
        )
    }
    // cpp: style_cascade.cc:461-470
    fn CollectDeclarationsIfNeeded(&mut self) {
        if self.needs_collect_from_match_result_ {
            self.CollectFromMatchResult();
            self.needs_collect_from_match_result_ = false;
        }
        if self.needs_collect_from_interpolations_ {
            self.CollectFromInterpolations();
            self.needs_collect_from_interpolations_ = false;
        }
    }
    // cpp: style_cascade.cc:472-494
    fn CollectFromMatchResult(&mut self) {
        self.AddExplicitDefaults();
        for index in 0..self.match_result_.GetMatchedProperties().len() {
            let declarations = self.backend_.ExpandCascade(
                &self.match_result_.GetMatchedProperties()[index],
                self.GetDocument(),
                index,
            );
            for (priority, name) in declarations {
                if name.IsCustomProperty() {
                    self.map_.AddCustom(name.ToAtomicString().clone(), priority);
                } else {
                    let property = self.ResolveSurrogate(*CSSProperty::Get(name.Id()));
                    self.map_.Add(property.PropertyID(), priority);
                }
            }
        }
    }
    // cpp: style_cascade.cc:496-531
    fn CollectFromInterpolations(&mut self) {
        for index in 0..self.interpolations_.entries.len() {
            let entry = &self.interpolations_.entries[index];
            let origin = entry.origin;
            let entries = self.backend_.InterpolationEntries(entry.map);
            for (name, _) in entries {
                let priority = CascadePriority::FromParts(
                    origin,
                    false,
                    0,
                    false,
                    false,
                    false,
                    0,
                    name.Id() as u16,
                    index as u16,
                );
                if name.IsCustomProperty() {
                    self.map_.AddCustom(name.ToAtomicString().clone(), priority);
                } else {
                    let property = self.ResolveSurrogate(*CSSProperty::Get(name.Id()));
                    self.map_.Add(property.PropertyID(), priority);
                    if let Some(visited) = property.GetVisitedProperty() {
                        self.map_.Add(visited.PropertyID(), priority);
                    }
                }
            }
        }
    }
    // cpp: style_cascade.cc:541-603
    fn AddExplicitDefaults(&mut self) {
        if !self.effective_zoom_changed_ {
            return;
        }
        for id in [
            CSSPropertyID::kBorderTopWidth,
            CSSPropertyID::kBorderRightWidth,
            CSSPropertyID::kBorderBottomWidth,
            CSSPropertyID::kBorderLeftWidth,
            CSSPropertyID::kOutlineWidth,
            CSSPropertyID::kColumnRuleWidth,
            CSSPropertyID::kRowRuleWidth,
        ] {
            self.map_
                .Add(id, CascadePriority::FromOrigin(CascadeOrigin::kNone));
        }
        if self
            .backend_
            .StandardizedBrowserZoomEnabled(self.GetDocument())
        {
            for id in [
                CSSPropertyID::kLetterSpacing,
                CSSPropertyID::kLineHeight,
                CSSPropertyID::kStrokeDasharray,
                CSSPropertyID::kStrokeDashoffset,
                CSSPropertyID::kTextIndent,
                CSSPropertyID::kTextShadow,
                CSSPropertyID::kTextUnderlineOffset,
                CSSPropertyID::kWebkitTextStrokeWidth,
                CSSPropertyID::kWebkitBorderHorizontalSpacing,
                CSSPropertyID::kWebkitBorderVerticalSpacing,
                CSSPropertyID::kWordSpacing,
            ] {
                self.map_
                    .Add(id, CascadePriority::FromOrigin(CascadeOrigin::kNone));
            }
        }
    }
    // cpp: style_cascade.cc:605-612
    fn ResetAndCollectAgain(&mut self) {
        self.map_.Reset();
        self.depends_on_cascade_affecting_property_ = false;
        self.needs_collect_from_match_result_ = true;
        self.needs_collect_from_interpolations_ = true;
        self.CollectDeclarationsIfNeeded();
    }
    // cpp: style_cascade.cc:614-655
    fn ApplyCascadeAffecting(&mut self, resolver: &mut B::CascadeResolver) {
        let direction = self.state_.StyleBuilder().Direction();
        let writing_mode = self.state_.StyleBuilder().GetWritingMode();
        let zoom = self.state_.StyleBuilder().EffectiveZoom();
        for id in [
            CSSPropertyID::kDirection,
            CSSPropertyID::kWritingMode,
            CSSPropertyID::kZoom,
        ] {
            if self.map_.NativeBitset().Has(id) {
                self.LookupAndApplyNative(id, resolver);
            }
        }
        let mut recollect = false;
        if self.depends_on_cascade_affecting_property_
            && (direction != self.state_.StyleBuilder().Direction()
                || writing_mode != self.state_.StyleBuilder().GetWritingMode())
        {
            recollect = true;
        }
        if zoom != self.state_.StyleBuilder().EffectiveZoom() {
            self.effective_zoom_changed_ = true;
            recollect = true;
        }
        if recollect {
            self.ResetAndCollectAgain();
        }
    }
    // cpp: style_cascade.cc:659-696
    fn ApplyViewportUnitAffecting(&mut self, resolver: &mut B::CascadeResolver) {
        use layoutng_style::style::computed_style_constants::PseudoId;
        if !self.IsRootElement()
            || self.state_.IsForPseudoElement()
            || self
                .match_result_
                .PseudoElementStyles()
                .Has(PseudoId::kPseudoIdScrollbar)
        {
            return;
        }
        for id in [
            CSSPropertyID::kOverflowX,
            CSSPropertyID::kOverflowY,
            CSSPropertyID::kScrollbarGutter,
            CSSPropertyID::kScrollbarWidth,
        ] {
            if self.map_.NativeBitset().Has(id) {
                self.LookupAndApplyNative(id, resolver);
            }
        }
        let to_subtract = {
            let builder = self.state_.StyleBuilder();
            self.backend_.ComputeScrollbarWidthsForViewportUnits(
                self.GetDocument(),
                builder.OverflowX(),
                builder.OverflowY(),
                self.backend_.BuilderScrollbarGutter(&builder),
                builder.ScrollbarWidth(),
                builder.GetWritingMode(),
            )
        };
        if self.backend_.SmallerViewportUnitsEnabled() {
            self.state_
                .SubtractScrollbarsFromViewportUnits(&to_subtract);
        }
        self.state_
            .StyleBuilderMut()
            .SetUnconditionalScrollbarSize(&to_subtract);
    }
    // cpp: style_cascade.cc:698-706
    fn ApplyHighPriority(&mut self, resolver: &mut B::CascadeResolver) {
        let mut bits = self.map_.HighPriorityBits();
        while bits != 0 {
            let index = bits.trailing_zeros();
            bits &= bits - 1;
            self.LookupAndApplyNative(ConvertToCSSPropertyID(index as i32), resolver);
        }
    }
    // cpp: style_cascade.cc:708-804
    fn ApplyWideOverlapping(&mut self, resolver: &mut B::CascadeResolver) {
        let border_image = self.backend_.BorderImageLonghands().to_vec();
        self.ApplyWide(CSSPropertyID::kWebkitBorderImage, &border_image, resolver);
        self.ApplyWide(
            CSSPropertyID::kPerspectiveOrigin,
            &[
                CSSPropertyID::kWebkitPerspectiveOriginX,
                CSSPropertyID::kWebkitPerspectiveOriginY,
            ],
            resolver,
        );
        self.ApplyWide(
            CSSPropertyID::kTransformOrigin,
            &[
                CSSPropertyID::kWebkitTransformOriginX,
                CSSPropertyID::kWebkitTransformOriginY,
                CSSPropertyID::kWebkitTransformOriginZ,
            ],
            resolver,
        );
        self.ApplyWide(
            CSSPropertyID::kVerticalAlign,
            &[CSSPropertyID::kBaselineSource],
            resolver,
        );
        self.ApplyWide(
            CSSPropertyID::kWebkitBoxDecorationBreak,
            &[CSSPropertyID::kBoxDecorationBreak],
            resolver,
        );
        if self.backend_.CSSLineClampEnabled() && !self.backend_.CSSLineClampAsShorthandEnabled() {
            self.ApplyWide(
                CSSPropertyID::kLineClamp,
                &[CSSPropertyID::kAlternativeWebkitLineClampLonghand],
                resolver,
            );
        }
    }
    fn ApplyWide(
        &mut self,
        wide: CSSPropertyID,
        narrow: &[CSSPropertyID],
        resolver: &mut B::CascadeResolver,
    ) {
        let property = CSSProperty::Get(wide);
        if !self.backend_.ResolverFilter(resolver).Accepts(property) {
            return;
        }
        let name = property.GetCSSPropertyName();
        if self.map_.Find(&name).is_none() {
            return;
        }
        self.LookupAndApplyProperty(&name, property, resolver);
        let priority = self.map_.At(&name);
        for id in narrow {
            if let Some(p) = self.map_.FindMut(&CSSPropertyName::new(*id)) {
                if *p < priority {
                    *p = CascadePriority::WithAlreadyApplied(*p, true);
                }
            }
        }
    }
    // cpp: style_cascade.cc:810-852
    fn ApplyMatchResult(&mut self, resolver: &mut B::CascadeResolver) {
        let native: Vec<_> = self.map_.NativeBitset().BeginAfterHighPriority().collect();
        for id in native {
            let name = CSSPropertyName::new(id);
            let priority = self.map_.At(&name);
            if priority.IsAlreadyApplied() || Self::IsInterpolation(priority) {
                continue;
            }
            let property = CSSProperty::Get(id);
            if self.backend_.ResolverRejects(resolver, property) {
                continue;
            }
            self.LookupAndApplyDeclaration(&name, property, resolver);
        }
        let custom: Vec<_> = self.map_.GetCustomMap().keys().cloned().collect();
        for name in custom {
            let name = CSSPropertyName::custom(name);
            let priority = self.map_.At(&name);
            if priority.IsAlreadyApplied() || Self::IsInterpolation(priority) {
                continue;
            }
            let property = self.PropertyMetadata(&name);
            if self.backend_.ResolverRejects(resolver, &property) {
                continue;
            }
            self.LookupAndApplyDeclaration(&name, &property, resolver);
        }
    }
    // cpp: style_cascade.cc:854-860
    fn ApplyInterpolations(&mut self, resolver: &mut B::CascadeResolver) {
        for index in 0..self.interpolations_.entries.len() {
            let entry = &self.interpolations_.entries[index];
            let map = entry.map;
            let origin = entry.origin;
            self.ApplyInterpolationMap(map, origin, index, resolver);
        }
    }
    // cpp: style_cascade.cc:862-895
    fn ApplyInterpolationMap(
        &mut self,
        map: &B::ActiveInterpolationsMap,
        origin: CascadeOrigin,
        index: usize,
        resolver: &mut B::CascadeResolver,
    ) {
        for (name, interpolations) in self.backend_.InterpolationEntries(map) {
            let priority = CascadePriority::WithAlreadyApplied(
                CascadePriority::FromParts(
                    origin,
                    false,
                    0,
                    false,
                    false,
                    false,
                    0,
                    name.Id() as u16,
                    index as u16,
                ),
                true,
            );
            let raw_property = self.PropertyMetadata(&name);
            if self.backend_.ResolverRejects(resolver, &raw_property) {
                continue;
            }
            let property = self.ResolveSurrogate(raw_property);
            let name = if name.IsCustomProperty() {
                name
            } else {
                property.GetCSSPropertyName()
            };
            let Some(p) = self.map_.FindMut(&name) else {
                continue;
            };
            if *p >= priority {
                continue;
            }
            *p = priority;
            self.ApplyInterpolation(&name, &property, priority, &interpolations, resolver);
        }
    }
    // cpp: style_cascade.cc:897-934
    fn ApplyInterpolation(
        &mut self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        priority: CascadePriority,
        interpolations: &B::ActiveInterpolations,
        resolver: &mut B::CascadeResolver,
    ) {
        let backend = self.backend_;
        if backend.FirstInterpolationIsInvalidatable(interpolations) {
            backend.ApplyInvalidatableInterpolationStack(
                interpolations,
                self.state_,
                self,
                resolver,
            );
        } else {
            backend.ApplyTransitionInterpolation(interpolations, self.state_, self, resolver);
        }
        if let Some(visited) = property.GetVisitedProperty() {
            let visited_name = visited.GetCSSPropertyName();
            if let Some(p) = self.map_.FindMut(&visited_name) {
                if priority < *p {
                    *p = CascadePriority::WithAlreadyApplied(*p, false);
                    self.LookupAndApplyProperty(&visited_name, visited, resolver);
                }
            }
        }
        let _ = name; // Custom names are retained for interpolation environment callbacks.
    }
    // cpp: style_cascade.cc:936-941
    pub fn LookupAndApply(&mut self, name: &CSSPropertyName, resolver: &mut B::CascadeResolver) {
        let property = self.PropertyMetadata(name);
        self.LookupAndApplyProperty(name, &property, resolver);
    }
    fn LookupAndApplyNative(&mut self, id: CSSPropertyID, resolver: &mut B::CascadeResolver) {
        self.LookupAndApplyProperty(&CSSPropertyName::new(id), CSSProperty::Get(id), resolver);
    }
    // cpp: style_cascade.cc:943-960
    fn LookupAndApplyProperty(
        &mut self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        resolver: &mut B::CascadeResolver,
    ) {
        if self.map_.Find(name).is_none() {
            return;
        }
        if self.backend_.ResolverRejects(resolver, property) {
            return;
        }
        self.LookupAndApplyValue(name, property, resolver);
    }
    // cpp: style_cascade.cc:962-972
    fn LookupAndApplyValue(
        &mut self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        resolver: &mut B::CascadeResolver,
    ) {
        let priority = self.map_.At(name);
        if (priority.GetOrigin() as u8) < CascadeOrigin::kAnimation as u8 {
            self.LookupAndApplyDeclaration(name, property, resolver);
        } else if (priority.GetOrigin() as u8) >= CascadeOrigin::kAnimation as u8 {
            self.LookupAndApplyInterpolation(name, property, resolver);
        }
    }
    // cpp: style_cascade.cc:974-1000. Resolve before applying the scoped value.
    fn LookupAndApplyDeclaration(
        &mut self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        resolver: &mut B::CascadeResolver,
    ) {
        let p = self
            .map_
            .FindMut(name)
            .expect("lookup declaration has priority");
        if p.IsAlreadyApplied() {
            return;
        }
        *p = CascadePriority::WithAlreadyApplied(*p, true);
        let priority = *p;
        let mut origin = priority.GetOrigin();
        let value = if origin == CascadeOrigin::kNone {
            self.backend_.NewUnsetValue()
        } else {
            self.ValueAt(priority)
        };
        let scope = self.GetTreeScope(priority);
        let mixin = self.GetMixinParameterBindings(priority);
        let value = self.ResolveValue(
            property,
            name,
            value,
            scope.clone(),
            mixin,
            priority,
            &mut origin,
            resolver,
        );
        let value = self.backend_.EnsureScopedValue(value, scope.as_deref());
        self.backend_
            .ApplyPhysicalProperty(name, property, self.state_, &value);
    }
    // cpp: style_cascade.cc:1002-1027. The complete function precedes
    // the requested 1029 cutoff; the IsRootElement helper follows it.
    fn LookupAndApplyInterpolation(
        &mut self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        resolver: &mut B::CascadeResolver,
    ) {
        let p = self
            .map_
            .FindMut(name)
            .expect("lookup interpolation has priority");
        if p.IsAlreadyApplied() {
            return;
        }
        *p = CascadePriority::WithAlreadyApplied(*p, true);
        let priority = *p;
        if property.IsVisited() {
            return;
        }
        let map = self.interpolations_.entries[priority.GetDeclarationIndex()].map;
        let handle = self.ToPropertyHandle(name, priority);
        let interpolations = self
            .backend_
            .FindInterpolation(map, &handle)
            .expect("source CHECK: interpolation handle must exist");
        self.ApplyInterpolation(name, property, priority, &interpolations, resolver);
    }
    // cpp: style_cascade.cc:145-166; matched-property data access uses the
    // actual erased CSSPropertyValueSet class API, not another property model.
    fn ValueAt(&self, priority: CascadePriority) -> Rc<ResolverValue<B>> {
        self.backend_.PropertyValueAt(
            self.match_result_.GetMatchedProperties()[priority.GetRuleIndex()]
                .properties
                .as_ref(),
            priority.GetDeclarationIndex(),
        )
    }
    fn PropertyMetadata(&self, name: &CSSPropertyName) -> CSSProperty {
        if name.IsCustomProperty() {
            self.backend_
                .CustomPropertyMetadata(name.ToAtomicString(), self.GetDocument())
        } else {
            *CSSProperty::Get(name.Id())
        }
    }
    // cpp: style_cascade.cc:168-176
    fn ToPropertyHandle(
        &self,
        name: &CSSPropertyName,
        priority: CascadePriority,
    ) -> B::PropertyHandle {
        let id = ConvertToCSSPropertyID(priority.GetRuleIndex() as i32);
        if id == CSSPropertyID::kVariable {
            self.backend_.CustomPropertyHandle(name.ToAtomicString())
        } else {
            self.backend_.NativePropertyHandle(CSSProperty::Get(id))
        }
    }
    // cpp: style_cascade.cc:195-207
    fn IsInterpolation(priority: CascadePriority) -> bool {
        matches!(
            priority.GetOrigin(),
            CascadeOrigin::kAnimation | CascadeOrigin::kTransition
        )
    }
    // cpp: style_cascade.cc:1029-1031
    fn IsRootElement(&self) -> bool {
        let element = self.state_.GetElement();
        self.backend_
            .DocumentElement(self.GetDocument())
            .is_some_and(|root| Rc::ptr_eq(&root, &element))
    }
    // cpp: style_cascade.cc:3106-3128
    fn GetDocument(&self) -> &B::Document {
        self.state_.GetDocument()
    }
    fn GetTreeScope(&self, priority: CascadePriority) -> Option<Rc<dyn TreeScope>> {
        match priority.GetOrigin() {
            CascadeOrigin::kAuthor => {
                let properties =
                    &self.match_result_.GetMatchedProperties()[priority.GetRuleIndex()];
                Some(
                    self.match_result_
                        .ScopeFromTreeOrderHandle(properties.data_.tree_order),
                )
            }
            CascadeOrigin::kAuthorPresentationalHint => {
                Some(self.backend_.DocumentTreeScope(self.GetDocument()))
            }
            _ => None,
        }
    }
    fn GetMixinParameterBindings(
        &self,
        priority: CascadePriority,
    ) -> Option<Rc<dyn MixinParameterBindings>> {
        if priority.GetOrigin() == CascadeOrigin::kAuthor {
            self.match_result_.GetMatchedProperties()[priority.GetRuleIndex()]
                .mixin_parameter_bindings
                .clone()
        } else {
            None
        }
    }
    // cpp: style_cascade.cc:3130-3143
    fn ResolveSurrogate(&mut self, property: CSSProperty) -> CSSProperty {
        if !property.IsSurrogate() {
            return property;
        }
        self.depends_on_cascade_affecting_property_ = true;
        let builder = self.state_.StyleBuilder();
        self.backend_.SurrogateFor(
            &property,
            WritingDirectionMode::new(builder.GetWritingMode(), builder.Direction()),
        )
    }
    // cpp: style_cascade.cc:3101-3104
    fn TreatAsRevertLayer(&self, priority: CascadePriority) -> bool {
        priority.IsTryStyle()
            && !matches!(
                self.state_.StyleBuilder().GetPosition(),
                EPosition::kAbsolute | EPosition::kFixed
            )
    }
    // cpp: style_cascade.cc:3046-3099. Safe-area token resolution and simple-sum check; use counter omitted.
    fn ApplyUnresolvedEnv(&mut self, _resolver: &mut B::CascadeResolver) {
        if !self.state_.StyleBuilder().HasEnvSafeAreaInsetBottom()
            || !self.map_.NativeBitset().Has(CSSPropertyID::kBottom)
        {
            return;
        }
        let priority = self.map_.At(&CSSPropertyName::new(CSSPropertyID::kBottom));
        if (priority.GetOrigin() as u8) >= CascadeOrigin::kAnimation as u8 {
            return;
        }
        let value = self.ValueAt(priority);
        if !value.IsUnparsedDeclaration() {
            return;
        }
        if !self.backend_.UnparsedContainsSafeAreaInsetBottom(&value) {
            return;
        }
        let node = self
            .backend_
            .PropertyCycleNode(&CSSPropertyName::new(CSSPropertyID::kBottom));
        self.WithResolverLock(_resolver, node, |this, resolver| {
            let mut sequence = TokenSequence::new();
            let properties = &this.match_result_.GetMatchedProperties()[priority.GetRuleIndex()];
            let scope = this
                .match_result_
                .ScopeFromTreeOrderHandle(properties.data_.tree_order);
            let text = this
                .backend_
                .VariableText(&this.backend_.UnparsedData(&value));
            let context = this
                .backend_
                .UnparsedParserContext(&value)
                .expect("source safe-area unparsed value has parser context");
            let mut stream = TokenStream::new(StringView::from(&text), 0);
            if !this.ResolveTokensInto(
                &mut stream,
                Some(scope),
                resolver,
                &context,
                None,
                kEOFToken,
                &mut sequence,
            ) {
                return;
            }
            if this.backend_.IsSimpleSum(sequence.OriginalText()) {
                this.state_
                    .StyleBuilderMut()
                    .SetIsBottomRelativeToSafeAreaInset(true);
            }
        })
    }
}
impl<'a, 's, B: StyleCascadeBackend> StyleCascade<'a, 's, B> {
    fn WithResolverLock<T>(
        &mut self,
        resolver: &mut B::CascadeResolver,
        node: B::ResolverCycleNode,
        f: impl FnOnce(&mut Self, &mut B::CascadeResolver) -> T,
    ) -> T {
        let b = self.backend_;
        let lock = b.ResolverLock(resolver, node);
        let result = f(self, resolver);
        b.ResolverUnlock(resolver, lock);
        result
    }
    // cpp: style_cascade.cc:1177-1211.
    fn ResolveValue(
        &mut self,
        property: &CSSProperty,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        mixin: Option<Rc<dyn MixinParameterBindings>>,
        priority: CascadePriority,
        origin: &mut CascadeOrigin,
        resolver: &mut B::CascadeResolver,
    ) -> Rc<ResolverValue<B>> {
        let result = self
            .ResolveSubstitutionsValue(property, name, value, scope.clone(), mixin, resolver)
            .expect("Chromium Resolve requires non-null substitution result");
        if result.IsRevertValue() {
            return self.ResolveRevert(property, name, priority, origin, resolver);
        }
        if result.IsRevertLayerValue() || self.TreatAsRevertLayer(priority) {
            return self.ResolveRevertLayer(property, name, priority, origin, resolver);
        }
        if result.IsRevertRuleValue() {
            let p = self.map_.FindRevertRule(name, priority).copied();
            return self.ResolveRevertTo(p, property, name, priority, origin, resolver);
        }
        if result.IsFlipRevertValue() {
            let id = self.backend_.FlipRevertPropertyID(&result);
            let to = self.ResolveSurrogate(*CSSProperty::Get(id));
            let to_name = to.GetCSSPropertyName();
            let unflipped = self.ResolveRevertLayer(&to, &to_name, priority, origin, resolver);
            let flipped =
                self.backend_
                    .FlipRevertedValue(to.PropertyID(), unflipped, &result, self.state_);
            return self.ResolveValue(
                property, name, flipped, scope, None, priority, origin, resolver,
            );
        }
        self.backend_
            .ResolverCollectFlags(resolver, property, *origin);
        if result.IsMathFunctionValue() {
            return self.ResolveMathFunction(property, result, scope);
        }
        result
    }
    // cpp: style_cascade.cc:179-193,1513-1599.
    fn ResolveRevert(
        &mut self,
        property: &CSSProperty,
        name: &CSSPropertyName,
        priority: CascadePriority,
        origin: &mut CascadeOrigin,
        resolver: &mut B::CascadeResolver,
    ) -> Rc<ResolverValue<B>> {
        let target = match *origin {
            CascadeOrigin::kUserAgent => CascadeOrigin::kNone,
            CascadeOrigin::kUser => CascadeOrigin::kUserAgent,
            CascadeOrigin::kAuthorPresentationalHint
            | CascadeOrigin::kAuthor
            | CascadeOrigin::kAnimation => CascadeOrigin::kUser,
            CascadeOrigin::kNone | CascadeOrigin::kTransition => {
                unreachable!("source NOTREACHED: revert origin")
            }
        };
        if target == CascadeOrigin::kNone {
            return self.backend_.NewUnsetValue();
        }
        let p = self.map_.FindForOrigin(name, target).copied();
        self.ResolveRevertTo(p, property, name, priority, origin, resolver)
    }
    fn ResolveRevertLayer(
        &mut self,
        property: &CSSProperty,
        name: &CSSPropertyName,
        priority: CascadePriority,
        origin: &mut CascadeOrigin,
        resolver: &mut B::CascadeResolver,
    ) -> Rc<ResolverValue<B>> {
        let p = self
            .map_
            .FindRevertLayer(name, priority.ForLayerComparison())
            .copied();
        self.ResolveRevertTo(p, property, name, priority, origin, resolver)
    }
    fn ResolveRevertTo(
        &mut self,
        p: Option<CascadePriority>,
        property: &CSSProperty,
        name: &CSSPropertyName,
        priority: CascadePriority,
        origin: &mut CascadeOrigin,
        resolver: &mut B::CascadeResolver,
    ) -> Rc<ResolverValue<B>> {
        let Some(p) = p.filter(|p| p.HasOrigin() && *p < priority) else {
            *origin = CascadeOrigin::kNone;
            return self.backend_.NewUnsetValue();
        };
        *origin = p.GetOrigin();
        let value = self.ValueAt(p);
        self.ResolveValue(
            property,
            name,
            value,
            self.GetTreeScope(p),
            self.GetMixinParameterBindings(p),
            p,
            origin,
            resolver,
        )
    }
    // cpp: style_cascade.cc:1625-1650.
    fn ResolveMathFunction(
        &self,
        property: &CSSProperty,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
    ) -> Rc<ResolverValue<B>> {
        let b = self.backend_;
        if !b.MathValueHasAnchorFunctions(&value) {
            return value;
        }
        let anchor_scope = b.EnterAnchorScope(property.PropertyID(), self.state_);
        let scoped = b.EnsureScopedValue(value, scope.as_deref());
        let invalid = b.MathHasInvalidAnchorFunctions(&scoped, self.state_);
        b.LeaveAnchorScope(anchor_scope);
        if invalid {
            b.NewUnsetValue()
        } else {
            scoped
        }
    }
    // cpp: style_cascade.cc:1322-1359.
    fn ResolveSubstitutionsValue(
        &mut self,
        property: &CSSProperty,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        mut mixin: Option<Rc<dyn MixinParameterBindings>>,
        resolver: &mut B::CascadeResolver,
    ) -> Option<Rc<ResolverValue<B>>> {
        if mixin.is_some() {
            if value.IsUnparsedDeclaration() {
                if !self
                    .backend_
                    .VariableNeedsResolution(&self.backend_.UnparsedData(&value))
                {
                    mixin = None;
                }
            } else if !value.IsPendingSubstitutionValue() {
                mixin = None;
            }
        }
        let mut chain = Vec::new();
        while let Some(binding) = mixin {
            mixin = self.backend_.MixinParent(binding.as_ref());
            chain.push(binding);
        }
        chain.reverse();
        self.MakeFunctionContextFromMixinAndResolveSubstitutions(
            property, name, value, scope, &chain, 0, None, resolver,
        )
    }
    // cpp: style_cascade.cc:1213-1320.
    fn MakeFunctionContextFromMixinAndResolveSubstitutions(
        &mut self,
        property: &CSSProperty,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        chain: &[Rc<dyn MixinParameterBindings>],
        index: usize,
        frame: Option<Frame<B>>,
        resolver: &mut B::CascadeResolver,
    ) -> Option<Rc<ResolverValue<B>>> {
        if index == chain.len() {
            if value.IsUnparsedDeclaration() {
                return if name.IsCustomProperty() {
                    Some(self.ResolveCustomProperty(property, name, value, scope, frame, resolver))
                } else {
                    Some(
                        self.ResolveVariableReference(
                            property, name, value, scope, frame, resolver,
                        ),
                    )
                };
            }
            if value.IsPendingSubstitutionValue() {
                return Some(
                    self.ResolvePendingSubstitution(property, name, value, scope, frame, resolver),
                );
            }
            return Some(value);
        }
        let unparsed = if value.IsUnparsedDeclaration() {
            value.clone()
        } else {
            self.backend_.PendingShorthand(&value).1
        };
        let context = self.GetParserContext(&unparsed);
        let b = self.backend_;
        let bindings = &chain[index];
        let mut arguments = HashMap::new();
        let mut defaults = HashMap::new();
        let mut types = HashMap::new();
        for (name, argument, default, syntax) in b.MixinParameters(bindings.as_ref()) {
            types.insert(name.clone(), syntax.clone());
            self.ResolveFunctionParameter(
                &name,
                argument,
                default,
                &syntax,
                scope.clone(),
                resolver,
                &context,
                frame.clone(),
                &mut arguments,
                &mut defaults,
            );
        }
        if !self.ResolveUnresolvedFunctionDefaults(
            &defaults,
            &types,
            None,
            None,
            frame.clone(),
            resolver,
            &context,
            &mut arguments,
        ) {
            return None;
        }
        let mut locals = b.MixinBaseLocals(bindings.as_ref());
        for (name, candidates) in b.MixinConditionalLocals(bindings.as_ref()) {
            self.state_.StyleBuilderMut().SetHasContainerRelativeValue();
            for (queries, data) in candidates.into_iter().rev() {
                if self.EvaluateContainerQueries(&queries, scope.as_deref()) {
                    locals.insert(name.clone(), Some(data));
                    break;
                }
            }
        }
        let ctx = Rc::new(RefCell::new(FunctionContext {
            function: None,
            tree_scope: None,
            arguments,
            locals: HashMap::new(),
            unresolved_locals: locals,
            local_types: types,
            parent: frame,
            invocation_count: 0,
        }));
        self.ApplyLocalVariables(resolver, &context, ctx.clone());
        self.MakeFunctionContextFromMixinAndResolveSubstitutions(
            property,
            name,
            value,
            scope,
            chain,
            index + 1,
            Some(ctx),
            resolver,
        )
    }
    // cpp: style_cascade.cc:1361-1416.
    fn ResolveCustomProperty(
        &mut self,
        _property: &CSSProperty,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        frame: Option<Frame<B>>,
        resolver: &mut B::CascadeResolver,
    ) -> Rc<ResolverValue<B>> {
        let node = self.backend_.PropertyCycleNode(name);
        self.WithResolverLock(resolver, node, |this, resolver| {
            let b = this.backend_;
            let original = b.UnparsedData(&value);
            let mut data = Some(original.clone());
            if b.VariableNeedsResolution(&original) {
                data = this.ResolveVariableData(
                    original.clone(),
                    scope,
                    &this.GetParserContext(&value),
                    frame,
                    resolver,
                );
            }
            let registered =
                b.PropertyRegistrationSyntax(this.GetDocument(), &name.ToAtomicString());
            if let (Some(syntax), Some(data)) = (&registered, &data) {
                let features = b.VariableFeatures(data);
                if !b.SyntaxIsUniversal(syntax)
                    && ((features & ((1 << 0) | (1 << 2)) != 0)
                        || (features & (1 << 1) != 0 && this.IsRootElement()))
                {
                    b.ResolverDetectCycle(
                        resolver,
                        &b.PropertyCycleNode(&CSSPropertyName::new(CSSPropertyID::kFontSize)),
                    );
                }
                if features & (1 << 2) != 0 {
                    b.ResolverDetectCycle(
                        resolver,
                        &b.PropertyCycleNode(&CSSPropertyName::new(CSSPropertyID::kLineHeight)),
                    );
                }
            }
            if b.ResolverInCycle(resolver) {
                return b.NewCyclicVariableValue();
            }
            let Some(data) = data else {
                return b.NewInvalidVariableValue();
            };
            if Rc::ptr_eq(&data, &original) {
                return value;
            }
            if let Some(keyword) = this.ParseAsCSSWideKeyword(&data, &this.GetParserContext(&value))
            {
                return keyword;
            }
            b.NewUnparsedValue(data, b.UnparsedParserContext(&value))
        })
    }
    // cpp: style_cascade.cc:1418-1456.
    fn ResolveVariableReference(
        &mut self,
        property: &CSSProperty,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        frame: Option<Frame<B>>,
        resolver: &mut B::CascadeResolver,
    ) -> Rc<ResolverValue<B>> {
        let node = self.backend_.PropertyCycleNode(name);
        self.WithResolverLock(resolver, node, |this, resolver| {
            let b = this.backend_;
            let data = b.UnparsedData(&value);
            let context = this.GetParserContext(&value);
            this.state_.StyleBuilderMut().SetHasVariableReference();
            let mut sequence = TokenSequence::new();
            let text = b.VariableText(&data);
            let mut stream = TokenStream::new(StringView::from(&text), 0);
            if this.ResolveTokensInto(
                &mut stream,
                scope,
                resolver,
                &context,
                frame,
                kEOFToken,
                &mut sequence,
            ) {
                let text = sequence.OriginalText();
                let mut stream = TokenStream::WithAttrTaintRanges(
                    StringView::from(&text),
                    Some(sequence.GetAttrTaintedRanges()),
                );
                if let Some(parsed) =
                    b.ParseSingleValue(property.PropertyID(), &mut stream, &context)
                {
                    return parsed;
                }
            }
            b.NewUnsetValue()
        })
    }
    // cpp: style_cascade.cc:1458-1510.
    fn ResolvePendingSubstitution(
        &mut self,
        property: &CSSProperty,
        name: &CSSPropertyName,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        frame: Option<Frame<B>>,
        resolver: &mut B::CascadeResolver,
    ) -> Rc<ResolverValue<B>> {
        let node = self.backend_.PropertyCycleNode(name);
        self.WithResolverLock(resolver, node, |this, resolver| {
            let b = this.backend_;
            this.state_.StyleBuilderMut().SetHasVariableReference();
            let cached = b
                .ShorthandCacheValue(resolver)
                .is_some_and(|cached| Rc::ptr_eq(&cached, &value));
            if !cached {
                let (id, shorthand) = b.PendingShorthand(&value);
                let data = b.UnparsedData(&shorthand);
                let mut sequence = TokenSequence::new();
                let text = b.VariableText(&data);
                let mut stream = TokenStream::new(StringView::from(&text), 0);
                if !this.ResolveTokensInto(
                    &mut stream,
                    scope,
                    resolver,
                    &this.GetParserContext(&shorthand),
                    frame,
                    kEOFToken,
                    &mut sequence,
                ) {
                    return b.NewUnsetValue();
                }
                let text = sequence.OriginalText();
                let mut stream = TokenStream::WithAttrTaintRanges(
                    StringView::from(&text),
                    Some(sequence.GetAttrTaintedRanges()),
                );
                let context = b.UnparsedParserContext(&shorthand);
                let Some(properties) = b.ParseShorthand(id, &mut stream, context.as_deref()) else {
                    return b.NewUnsetValue();
                };
                b.SetShorthandCache(resolver, value, properties);
            }
            let unvisited = if property.IsVisited() {
                property.GetUnvisitedProperty().expect("visited property")
            } else {
                property
            };
            for (id, value) in b.ShorthandCacheProperties(resolver) {
                let longhand = this.ResolveSurrogate(*CSSProperty::Get(id));
                if unvisited.HasEqualCSSPropertyName(&longhand) {
                    return value;
                }
            }
            b.NewUnsetValue()
        })
    }
    // cpp: style_cascade.cc:1652-1670.
    fn ResolveVariableData(
        &mut self,
        data: Variable<B>,
        scope: Option<Rc<dyn TreeScope>>,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        resolver: &mut B::CascadeResolver,
    ) -> Option<Variable<B>> {
        let mut sequence = TokenSequence::FromData(self.backend_, &data);
        let text = self.backend_.VariableText(&data);
        let mut stream = TokenStream::new(StringView::from(&text), 0);
        if !self.ResolveTokensInto(
            &mut stream,
            scope,
            resolver,
            context,
            frame,
            kEOFToken,
            &mut sequence,
        ) {
            return None;
        }
        Some(sequence.BuildVariableData(self.backend_))
    }
    // cpp: style_cascade.cc:1672-1742. Complete token dispatch and nesting.
    fn ResolveTokensInto(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        stop: CSSParserTokenType,
        out: &mut TokenSequence,
    ) -> bool {
        let mut success = true;
        let mut nesting = 0;
        loop {
            let token = stream.Peek().clone();
            if token.IsEOF() || (token.GetType() == stop && nesting == 0) {
                break;
            }
            match token.FunctionId() {
                Some(CSSValueID::kVar) => {
                    let mut guard = BlockGuard::new(stream);
                    success &= self.ResolveVarInto(
                        &mut guard,
                        scope.clone(),
                        resolver,
                        context,
                        frame.clone(),
                        out,
                    );
                }
                Some(CSSValueID::kInherit) => {
                    let mut guard = BlockGuard::new(stream);
                    success &= self.ResolveInheritInto(
                        &mut guard,
                        scope.clone(),
                        resolver,
                        context,
                        frame.clone(),
                        out,
                    );
                }
                Some(CSSValueID::kEnv) => {
                    let mut guard = BlockGuard::new(stream);
                    success &=
                        self.ResolveEnvInto(&mut guard, scope.clone(), resolver, context, out);
                }
                Some(CSSValueID::kAttr) => {
                    let mut guard = BlockGuard::new(stream);
                    self.state_.StyleBuilderMut().SetHasAttrFunction();
                    success &= self.ResolveAttrInto(
                        &mut guard,
                        scope.clone(),
                        resolver,
                        context,
                        frame.clone(),
                        out,
                    );
                }
                Some(CSSValueID::kInternalAutoBase) => {
                    let mut guard = BlockGuard::new(stream);
                    success &=
                        self.ResolveAutoBaseInto(&mut guard, scope.clone(), resolver, context, out);
                }
                Some(CSSValueID::kIf) => {
                    let mut guard = BlockGuard::new(stream);
                    success &= self.ResolveIfInto(
                        &mut guard,
                        scope.clone(),
                        resolver,
                        context,
                        frame.clone(),
                        out,
                    );
                }
                _ if token.GetType() == kFunctionToken
                    && self.backend_.IsValidVariableName(token.Value())
                    && self.backend_.CSSFunctionsEnabled() =>
                {
                    let mut guard = BlockGuard::new(stream);
                    success &= self.ResolveFunctionInto(
                        token.Value(),
                        scope.clone(),
                        &mut guard,
                        resolver,
                        context,
                        frame.clone(),
                        out,
                    );
                }
                _ => {
                    if token.GetBlockType() == BlockType::kBlockStart {
                        nesting += 1;
                    } else if token.GetBlockType() == BlockType::kBlockEnd {
                        if nesting == 0 {
                            break;
                        }
                        nesting -= 1;
                    }
                    let start = stream.Offset();
                    stream.ConsumeRaw();
                    let end = stream.Offset();
                    out.AppendToken(
                        self.backend_,
                        &token,
                        stream.IsAttrTainted(start, end),
                        stream.StringRangeAt(start, end - start),
                    );
                }
            }
        }
        success
    }
    // cpp: style_cascade.cc:1732-1826.
    fn ResolveVarInto(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        out: &mut TokenSequence,
    ) -> bool {
        let name = self.ConsumeAndComputeVariableName(stream, context);
        let mut current = frame.clone();
        while let Some(local) = current {
            self.LookupAndApplyLocalVariable(&name, resolver, context, local.clone());
            let data = local.borrow().locals.get(&name).cloned();
            if let Some(data) = data {
                return self
                    .AppendDataWithFallback(data, stream, scope, resolver, context, frame, out);
            }
            let argument = local.borrow().arguments.get(&name).cloned();
            if let Some(data) = argument {
                return self
                    .AppendDataWithFallback(data, stream, scope, resolver, context, frame, out);
            }
            current = local.borrow().parent.clone();
        }
        let b = self.backend_;
        let property = b.CustomPropertyMetadata(&name, self.GetDocument());
        if b.PropertyRegistrationSyntax(self.GetDocument(), &name)
            .is_some()
        {
            b.MarkPropertyRegistryReferenced(self.GetDocument(), &name);
        }
        let property_name = CSSPropertyName::custom(name.clone());
        if !b.ResolverDetectCycle(resolver, &b.PropertyCycleNode(&property_name)) {
            self.LookupAndApply(&property_name, resolver);
        }
        let mut data = b.BuilderVariableData(self.state_, &name, property.IsInherited());
        if !b.ResolverAllowSubstitution(resolver, data.as_deref()) {
            data = None;
        }
        if b.ResolverInCycle(resolver) {
            return false;
        }
        self.AppendDataWithFallback(data, stream, scope, resolver, context, frame, out)
    }
    // cpp: style_cascade.cc:1828-1868.
    fn ResolveInheritInto(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        out: &mut TokenSequence,
    ) -> bool {
        if let Some(frame) = frame {
            let parent = frame.borrow().parent.clone();
            return self.ResolveVarInto(stream, scope, resolver, context, parent, out);
        }
        let name = self.ConsumeAndComputeVariableName(stream, context);
        let property = self
            .backend_
            .CustomPropertyMetadata(&name, self.GetDocument());
        if !property.IsInherited() {
            self.state_.StyleBuilderMut().SetHasExplicitInheritance();
            self.backend_
                .ParentSetChildHasExplicitInheritance(self.state_);
        }
        let data = self.GetInheritedVariableData(&name);
        self.AppendDataWithFallback(data, stream, scope, resolver, context, None, out)
    }
    // cpp: style_cascade.cc:1870-2014.
    fn ResolveFunctionInto(
        &mut self,
        name: StringView,
        scope: Option<Rc<dyn TreeScope>>,
        stream: &mut TokenStream<'_>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        parent: Option<Frame<B>>,
        out: &mut TokenSequence,
    ) -> bool {
        self.state_.StyleBuilderMut().SetAffectedByCSSFunction();
        self.state_.SetHasTreeScopedReference();
        let b = self.backend_;
        let Some((function, function_scope)) = b.FindFunctionAcrossScopes(
            self.GetDocument(),
            &AtomicString::from_utf16(name.Span16()),
            scope.as_deref(),
        ) else {
            return false;
        };
        let node = b.FunctionCycleNode(&function);
        if b.ResolverDetectCycle(resolver, &node) {
            return false;
        }
        self.WithResolverLock(resolver, node, |this, resolver| {
            let parameters = b.FunctionParameters(&function);
            let supplied = b.ConsumeFunctionArguments(stream, parameters.len());
            if !stream.AtEnd() {
                return false;
            }
            let mut arguments = HashMap::new();
            let mut defaults = HashMap::new();
            let mut types = HashMap::new();
            for (index, (name, syntax, default)) in parameters.into_iter().enumerate() {
                types.insert(name.clone(), syntax.clone());
                if let Some(text) = supplied.get(index) {
                    let data = b.NewVariableData(text.clone(), false, false, 1 << 4);
                    this.ResolveFunctionParameter(
                        &name,
                        Some(data),
                        default,
                        &syntax,
                        scope.clone(),
                        resolver,
                        context,
                        parent.clone(),
                        &mut arguments,
                        &mut defaults,
                    );
                } else if let Some(default) = default {
                    defaults.insert(name, Some(default));
                } else {
                    return false;
                }
            }
            let invocation = b.ResolverNextFunctionInvocationCount(resolver);
            if !this.ResolveUnresolvedFunctionDefaults(
                &defaults,
                &types,
                Some(function.clone()),
                Some(function_scope.clone()),
                parent.clone(),
                resolver,
                context,
                &mut arguments,
            ) {
                return false;
            }
            let mut result = None;
            let mut locals = HashMap::new();
            this.FlattenFunctionBody(
                &b.FunctionAsGroup(&function),
                &function_scope,
                &mut result,
                &mut locals,
            );
            let Some(result) = result else {
                return false;
            };
            let frame = Rc::new(RefCell::new(FunctionContext {
                function: Some(function.clone()),
                tree_scope: Some(function_scope.clone()),
                arguments,
                locals: HashMap::new(),
                unresolved_locals: locals,
                local_types: types,
                parent,
                invocation_count: invocation,
            }));
            this.ApplyLocalVariables(resolver, context, frame.clone());
            if b.ResolverInCycle(resolver) {
                return false;
            }
            let mut local = this.GetCSSParserLocalContext(
                Some(&frame),
                Some(&CSSPropertyName::new(CSSPropertyID::kResult)),
            );
            let data = this.ResolveTypedExpression(
                result,
                Some(function_scope),
                Some(b.FunctionReturnType(&function)),
                resolver,
                context,
                Some(frame),
                &mut local,
            );
            let Some(data) = data else {
                return false;
            };
            out.AppendData(b, &data, b.VariableAttrTainted(&data), 2097152)
        })
    }
    // cpp: style_cascade.cc:2026-2047.
    fn ResolveFunctionParameter(
        &mut self,
        name: &AtomicString,
        mut argument: Option<Variable<B>>,
        default: Option<Variable<B>>,
        syntax: &Rc<B::Syntax>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        arguments: &mut VariableMap<B>,
        defaults: &mut VariableMap<B>,
    ) {
        if let Some(data) = argument {
            let mut local = self.GetCSSParserLocalContext(
                frame.as_ref(),
                Some(&CSSPropertyName::custom(name.clone())),
            );
            argument = self.ResolveTypedExpression(
                data,
                scope,
                Some(syntax.clone()),
                resolver,
                context,
                frame,
                &mut local,
            );
        }
        if let Some(data) = argument {
            arguments.insert(name.clone(), Some(data));
        } else if let Some(default) = default {
            defaults.insert(name.clone(), Some(default));
        } else {
            arguments.insert(name.clone(), None);
        }
    }
    // cpp: style_cascade.cc:2058-2095.
    fn ResolveUnresolvedFunctionDefaults(
        &mut self,
        defaults: &VariableMap<B>,
        types: &HashMap<AtomicString, Rc<B::Syntax>>,
        function: Option<Rc<B::StyleRuleFunction>>,
        scope: Option<Rc<dyn TreeScope>>,
        parent: Option<Frame<B>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        arguments: &mut VariableMap<B>,
    ) -> bool {
        if !defaults.is_empty() {
            let frame = Rc::new(RefCell::new(FunctionContext {
                function,
                tree_scope: scope,
                arguments: arguments.clone(),
                locals: HashMap::new(),
                unresolved_locals: defaults.clone(),
                local_types: types.clone(),
                parent,
                invocation_count: self.backend_.ResolverFunctionInvocationCount(resolver),
            }));
            self.ApplyLocalVariables(resolver, context, frame.clone());
            if self.backend_.ResolverInCycle(resolver) {
                return false;
            }
            for (name, data) in &frame.borrow().locals {
                arguments
                    .entry(name.clone())
                    .or_insert_with(|| data.clone());
            }
        }
        true
    }
    // cpp: style_cascade.cc:2097-2123.
    fn AppendDataWithFallback(
        &mut self,
        data: Option<Variable<B>>,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        out: &mut TokenSequence,
    ) -> bool {
        if let Some(data) = data {
            return out.AppendData(
                self.backend_,
                &data,
                self.backend_.VariableAttrTainted(&data),
                2097152,
            );
        }
        if stream.Peek().GetType() == kCommaToken {
            stream.ConsumeRaw();
            stream.ConsumeWhitespace();
            let mut fallback = TokenSequence::new();
            if self.ResolveTokensInto(
                stream,
                scope,
                resolver,
                context,
                frame,
                kEOFToken,
                &mut fallback,
            ) {
                return out.AppendFallback(
                    self.backend_,
                    &fallback,
                    !fallback.attr_taint_ranges.is_empty(),
                    2097152,
                );
            }
        }
        false
    }
    // cpp: style_cascade.cc:2125-2144.
    fn GetCSSParserLocalContext(
        &self,
        frame: Option<&Frame<B>>,
        name: Option<&CSSPropertyName>,
    ) -> CSSParserLocalContext {
        let Some(name) = name else {
            return CSSParserLocalContext::CreateWithoutPropertyForAtRules();
        };
        if let Some(frame) = frame {
            let frame = frame.borrow();
            if let Some(function) = &frame.function {
                return CSSParserLocalContext::WithCustomFunction(
                    name.clone(),
                    CSSPropertyID::kInvalid,
                    self.backend_.FunctionName(function),
                    frame.invocation_count,
                );
            }
        }
        CSSParserLocalContext::new(name.clone(), CSSPropertyID::kInvalid)
    }
    // cpp: style_cascade.cc:2154-2190.
    fn ResolveTypedExpression(
        &mut self,
        mut data: Variable<B>,
        scope: Option<Rc<dyn TreeScope>>,
        syntax: Option<Rc<B::Syntax>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        local: &mut CSSParserLocalContext,
    ) -> Option<Variable<B>> {
        let b = self.backend_;
        if b.VariableNeedsResolution(&data) {
            let Some(resolved) = self.ResolveVariableData(data, scope, context, frame, resolver)
            else {
                return None;
            };
            data = resolved;
        }
        let Some(syntax) = syntax.filter(|syntax| !b.SyntaxIsUniversal(syntax)) else {
            return Some(data);
        };
        local.SetRandomValueCount(b.ResolverRandomValueCount(resolver));
        let Some(value) = b.SyntaxParse(&syntax, b.VariableText(&data), context, local, false)
        else {
            return None;
        };
        b.ResolverSetRandomValueCount(resolver, local.RandomValueCount());
        let value = b.ConvertRegisteredPropertyValue(self.state_, value, context);
        Some(b.ConvertRegisteredPropertyVariableData(
            &value,
            b.VariableAnimationTainted(&data),
            b.VariableAttrTainted(&data),
        ))
    }
    // cpp: style_cascade.cc:2192-2207.
    fn FindVariableType(
        &self,
        name: &AtomicString,
        mut frame: Option<Frame<B>>,
    ) -> Option<Rc<B::Syntax>> {
        while let Some(current) = frame {
            if let Some(syntax) = current.borrow().local_types.get(name) {
                return Some(syntax.clone());
            }
            frame = current.borrow().parent.clone();
        }
        self.backend_
            .PropertyRegistrationSyntax(self.GetDocument(), name)
    }
    // Direct function resolution dependencies: cc:2209-2349.
    fn ApplyLocalVariables(
        &mut self,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Frame<B>,
    ) {
        let names: Vec<_> = frame.borrow().unresolved_locals.keys().cloned().collect();
        for name in names {
            self.LookupAndApplyLocalVariable(&name, resolver, context, frame.clone());
        }
    }
    fn LookupAndApplyLocalVariable(
        &mut self,
        name: &AtomicString,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Frame<B>,
    ) {
        if frame.borrow().locals.contains_key(name) {
            return;
        }
        let data = frame.borrow().unresolved_locals.get(name).cloned();
        let Some(Some(data)) = data else {
            return;
        };
        let syntax = frame.borrow().local_types.get(name).cloned();
        let function = frame.borrow().function.clone();
        let scope = frame.borrow().tree_scope.clone();
        let b = self.backend_;
        let node = b.LocalCycleNode(name, function.as_deref());
        let resolved = if b.ResolverDetectCycle(resolver, &node) {
            None
        } else {
            self.WithResolverLock(resolver, node, |this, resolver| {
                let mut local = this.GetCSSParserLocalContext(
                    Some(&frame),
                    Some(&CSSPropertyName::custom(name.clone())),
                );
                let resolved = this.ResolveTypedExpression(
                    data,
                    scope,
                    syntax,
                    resolver,
                    context,
                    Some(frame.clone()),
                    &mut local,
                );
                let Some(data) = resolved else {
                    return None;
                };
                if let Some(keyword) = this.ParseAsCSSWideKeyword(&data, context) {
                    return this.GetKeywordVariableData(
                        name,
                        &keyword,
                        resolver,
                        context,
                        Some(frame.clone()),
                    );
                }
                Some(data)
            })
        };
        frame
            .borrow_mut()
            .locals
            .entry(name.clone())
            .or_insert(resolved);
    }
    fn FlattenFunctionBody(
        &mut self,
        group: &B::RuleGroup,
        scope: &Rc<dyn TreeScope>,
        result: &mut Option<Variable<B>>,
        locals: &mut VariableMap<B>,
    ) {
        let b = self.backend_;
        for child in b.RuleChildren(group) {
            let recurse = match b.FunctionRuleKind(&child) {
                FunctionRuleKind::Declarations => {
                    for (name, data) in b.FunctionRuleDeclarations(&child) {
                        if name.IsCustomProperty() {
                            locals.insert(name.ToAtomicString().clone(), Some(data));
                        } else if name.Id() == CSSPropertyID::kResult {
                            *result = Some(data);
                        }
                    }
                    false
                }
                FunctionRuleKind::Supports => b.SupportsConditionIsSupported(&child),
                FunctionRuleKind::Media => {
                    self.state_.StyleBuilderMut().SetAffectedByFunctionalMedia();
                    b.EvaluateFunctionalMediaRule(&child, self.GetDocument())
                }
                FunctionRuleKind::Container => {
                    self.state_.StyleBuilderMut().SetHasContainerRelativeValue();
                    self.EvaluateContainerQueries(
                        &b.ContainerRuleQueries(&child),
                        Some(scope.as_ref()),
                    )
                }
                FunctionRuleKind::Navigation => {
                    self.state_
                        .StyleBuilderMut()
                        .SetAffectedByFunctionalNavigation();
                    b.EvaluateNavigationRule(&child, self.GetDocument())
                }
                FunctionRuleKind::Other => false,
            };
            if recurse {
                self.FlattenFunctionBody(&b.FunctionRuleGroup(&child), scope, result, locals);
            }
        }
    }
    // cc:100-120. Ident parsing/computation belongs to external parsing/value
    // utilities; valid-name fallback and stream control remain in this class.
    fn ConsumeAndComputeVariableName(
        &self,
        stream: &mut TokenStream<'_>,
        context: &ParserContext<B>,
    ) -> AtomicString {
        stream.ConsumeWhitespace();
        if stream.Peek().GetType() == kIdentToken {
            return AtomicString::from_utf16(
                stream.ConsumeIncludingWhitespaceRaw().Value().Span16(),
            );
        }
        let value = self.backend_.ConsumeIdentFunction(stream, context);
        let ident = self.backend_.ComputeIdent(&value, self.state_);
        if self
            .backend_
            .IsValidVariableName(StringView::from(&String::from_utf16(
                ident.utf16_units().expect("computed ident is text"),
            )))
        {
            ident
        } else {
            AtomicString::from_str("unknown")
        }
    }
    // cc:130-136;2998-3007.
    fn ParseAsCSSWideKeyword(
        &self,
        data: &B::VariableData,
        context: &ParserContext<B>,
    ) -> Option<Rc<ResolverValue<B>>> {
        let text = self.backend_.VariableText(data);
        let mut stream = TokenStream::new(StringView::from(&text), 0);
        stream.ConsumeWhitespace();
        let value = self.backend_.ConsumeCSSWideKeyword(&mut stream, context);
        if stream.AtEnd() {
            value
        } else {
            None
        }
    }
    fn GetParserContext(&self, value: &ResolverValue<B>) -> Rc<ParserContext<B>> {
        self.backend_
            .UnparsedParserContext(value)
            .unwrap_or_else(|| self.backend_.StrictParserContext(self.GetDocument()))
    }
    // cc:2564-2617;2977-2982. Variable storage/registry reads belong to the
    // actual style and registry objects; initial/inherit/unset selection is Rust.
    fn GetInheritedVariableData(&self, name: &AtomicString) -> Option<Variable<B>> {
        if self.state_.ParentStyle().is_none() {
            self.GetInitialVariableData(name)
        } else {
            self.backend_.ParentVariableData(self.state_, name)
        }
    }
    fn GetKeywordVariableData(
        &mut self,
        name: &AtomicString,
        keyword: &ResolverValue<B>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
    ) -> Option<Variable<B>> {
        if let Some(frame) = frame {
            if keyword.IsInitialValue() {
                return frame.borrow().arguments.get(name).cloned().flatten();
            }
            if keyword.IsInheritedValue() {
                let parent = frame.borrow().parent.clone();
                let text = String::from_utf16(name.utf16_units().expect("variable name is text"));
                let mut stream = TokenStream::new(StringView::from(&text), 0);
                let mut out = TokenSequence::new();
                if self.ResolveVarInto(&mut stream, None, resolver, context, parent, &mut out) {
                    return Some(out.BuildVariableData(self.backend_));
                }
            }
            return None;
        }
        let property = self
            .backend_
            .CustomPropertyMetadata(name, self.GetDocument());
        if keyword.IsInitialValue() {
            return self.GetInitialVariableData(name);
        }
        if keyword.IsInheritedValue() {
            return self.GetInheritedVariableData(name);
        }
        if keyword.IsUnsetValue() {
            return if self.state_.IsInheritedForUnset(&property) {
                self.GetInheritedVariableData(name)
            } else {
                self.GetInitialVariableData(name)
            };
        }
        None
    }
    // cpp: style_cascade.cc:442-459. This overload performs only substitution,
    // preserving the source nullable result and static state-local resolver.
    pub fn ResolveSubstitutionsForState(
        state: &'a StyleResolverState<'s, B>,
        backend: &'a B,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        _mixin: Option<Rc<dyn MixinParameterBindings>>,
    ) -> Option<Rc<ResolverValue<B>>> {
        let mut cascade = Self::new(state, backend);
        let mut resolver = backend.NewCascadeResolver(CascadeFilter::new());
        let context = cascade.GetParserContext(&value);
        let text = backend.VariableText(&backend.UnparsedData(&value));
        let mut stream = TokenStream::new(StringView::from(&text), 0);
        let mut sequence = TokenSequence::new();
        if !cascade.ResolveTokensInto(
            &mut stream,
            scope,
            &mut resolver,
            &context,
            None,
            kEOFToken,
            &mut sequence,
        ) {
            return None;
        }
        Some(backend.NewUnparsedValue(sequence.BuildVariableData(backend), Some(context)))
    }
    // cpp: style_cascade.cc:2564-2571.
    fn GetInitialVariableData(&self, name: &AtomicString) -> Option<Variable<B>> {
        let Some(initial) = self.backend_.BuilderInitialData(self.state_) else {
            return None;
        };
        self.backend_.InitialDataVariableData(&initial, name)
    }
}
// cpp: style_cascade.cc:2867-2919. The source local Handler becomes a typed
// visitor, so AST evaluation stays with the real ConditionalExpNode object.
pub trait IfConditionVisitor<B: StyleCascadeBackend> {
    fn EvaluateNavigationExpNode(&mut self, node: &B::NavigationExpNode) -> KleeneValue;
    fn EvaluateMediaQueryFeatureExpNode(
        &mut self,
        node: &B::MediaQueryFeatureExpNode,
    ) -> KleeneValue;
    fn EvaluateMediaQuerySet(&mut self, query: &B::MediaQuerySet) -> KleeneValue;
}
struct IfHandler<'c, 'r, 'a, 's, B: StyleCascadeBackend> {
    cascade: &'c mut StyleCascade<'a, 's, B>,
    resolver: &'r mut B::CascadeResolver,
    context: &'r ParserContext<B>,
    frame: Option<Frame<B>>,
    scope: Option<Rc<dyn TreeScope>>,
    attr_tainted: &'r mut bool,
}
impl<B: StyleCascadeBackend> IfConditionVisitor<B> for IfHandler<'_, '_, '_, '_, B> {
    fn EvaluateNavigationExpNode(&mut self, node: &B::NavigationExpNode) -> KleeneValue {
        self.cascade
            .state_
            .StyleBuilderMut()
            .SetAffectedByFunctionalNavigation();
        let b = self.cascade.backend_;
        let document = self.cascade.GetDocument();
        b.SetNeedsStyleUpdateOnNavigation(document);
        if b.EvaluateFunctionalNavigationQuery(document, &b.NavigationNodeTest(node)) {
            KleeneValue::kTrue
        } else {
            KleeneValue::kFalse
        }
    }
    fn EvaluateMediaQueryFeatureExpNode(
        &mut self,
        node: &B::MediaQueryFeatureExpNode,
    ) -> KleeneValue {
        self.cascade.EvalIfStyleFeature(
            node,
            self.scope.clone(),
            self.resolver,
            self.context,
            self.frame.clone(),
            self.attr_tainted,
        )
    }
    fn EvaluateMediaQuerySet(&mut self, query: &B::MediaQuerySet) -> KleeneValue {
        self.cascade
            .state_
            .StyleBuilderMut()
            .SetAffectedByFunctionalMedia();
        if self
            .cascade
            .backend_
            .EvaluateFunctionalMediaQuery(self.cascade.GetDocument(), query)
        {
            KleeneValue::kTrue
        } else {
            KleeneValue::kFalse
        }
    }
}
impl<'a, 's, B: StyleCascadeBackend> StyleCascade<'a, 's, B> {
    // cpp: style_cascade.cc:85-98.
    fn ConsumeVariableName(stream: &mut TokenStream<'_>) -> AtomicString {
        stream.ConsumeWhitespace();
        let token = stream.ConsumeIncludingWhitespaceRaw();
        AtomicString::from_utf16(token.Value().Span16())
    }
    fn ConsumeIfVariableName(stream: &mut TokenStream<'_>) -> AtomicString {
        stream.ConsumeWhitespace();
        if stream.Peek().GetType() != kIdentToken {
            return AtomicString::default();
        }
        AtomicString::from_utf16(stream.ConsumeIncludingWhitespace().Value().Span16())
    }
    // cpp: style_cascade.cc:232-268. The selector is an external class;
    // skipping, dependency updates, container choice and change selection are Rust.
    fn EvaluateContainerQueries(
        &mut self,
        queries: &B::ContainerQuerySet,
        scope: Option<&dyn TreeScope>,
    ) -> bool {
        let b = self.backend_;
        let element = self.state_.GetElement();
        let nearest = self.state_.NearestSizeContainer();
        for query in b.ContainerQueries(queries) {
            let selector = b.QuerySelector(&query);
            if !b.SelectorSelectsAnyContainer(&selector) {
                continue;
            }
            b.QuerySetDependencyFlags(&query, &mut self.match_result_);
            let start = b.QueryDetermineStartingElement(
                &element,
                self.state_.GetPseudoId(),
                &selector,
                nearest.as_deref(),
            );
            let Some(container) = b.QueryFindContainer(start.as_deref(), &selector, scope) else {
                continue;
            };
            let change = if start
                .as_ref()
                .is_some_and(|start| Rc::ptr_eq(start, &container))
            {
                b.NearestContainerChange()
            } else {
                b.DescendantContainersChange()
            };
            if b.QueryEvalAndAdd(&container, &query, change, &mut self.match_result_) {
                return true;
            }
        }
        false
    }
    // cpp: style_cascade.cc:270-280.
    fn IsVariableNameOnly(&self, text: StringView) -> bool {
        if !self.backend_.IsValidVariableName(text.clone()) {
            return false;
        }
        let mut stream = TokenStream::new(text, 0);
        if stream.Peek().GetType() != kIdentToken {
            return false;
        }
        stream.ConsumeIncludingWhitespace();
        stream.AtEnd()
    }
    // cpp: style_cascade.cc:2351-2390.
    fn ResolveEnvInto(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        out: &mut TokenSequence,
    ) -> bool {
        self.state_.StyleBuilderMut().SetHasEnv();
        let name = Self::ConsumeVariableName(stream);
        if name == AtomicString::from_str("safe-area-inset-bottom") {
            self.state_.StyleBuilderMut().SetHasEnvSafeAreaInsetBottom();
            self.backend_
                .SetNeedsToUpdateComplexSafeAreaConstraints(self.GetDocument());
        }
        let mut indices = Vec::new();
        if !stream.AtEnd() && stream.Peek().GetType() != kCommaToken {
            loop {
                let token = stream.ConsumeIncludingWhitespaceRaw();
                indices.push(token.NumericValue() as u32);
                if stream.Peek().GetType() != kNumberToken {
                    break;
                }
            }
        }
        let data = self.GetEnvironmentVariable(&name, indices);
        self.AppendDataWithFallback(data, stream, scope, resolver, context, None, out)
    }
    // cpp: style_cascade.cc:2392-2521.
    fn ResolveAttrInto(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        out: &mut TokenSequence,
    ) -> bool {
        let b = self.backend_;
        let mut missing_type = false;
        let (name, attr_type) = if b.CSSArgumentGrammarEnabled() {
            let mut first = TokenSequence::new();
            if !self.ResolveTokensInto(
                stream,
                scope.clone(),
                resolver,
                context,
                frame.clone(),
                kCommaToken,
                &mut first,
            ) {
                return false;
            }
            let text = first.OriginalText();
            let mut first_stream = TokenStream::new(StringView::from(&text), 0);
            let name = Self::ConsumeIfVariableName(&mut first_stream);
            let attr_type = b.ConsumeAttrType(&mut first_stream).unwrap_or_else(|| {
                missing_type = true;
                b.DefaultAttrType()
            });
            if name.IsNull() || !first_stream.AtEnd() {
                return false;
            }
            (name, attr_type)
        } else {
            let name = Self::ConsumeVariableName(stream);
            let attr_type = b.ConsumeAttrType(stream).unwrap_or_else(|| {
                missing_type = true;
                b.DefaultAttrType()
            });
            (name, attr_type)
        };
        let node = b.AttributeCycleNode(&name);
        if b.ResolverDetectCycle(resolver, &node) {
            return false;
        }
        self.WithResolverLock(resolver, node, |this, resolver| {
            let element = this.state_.GetUltimateOriginatingElementOrSelf();
            let name = b.ElementLowercaseIfNecessary(&element, name);
            let attribute = b.ElementGetAttributeNS(&element, &AtomicString::default(), &name);
            let mut substituted = attribute.clone();
            if !attribute.IsNull() && b.AttrTypeIsSyntax(&attr_type) {
                let mut sequence = TokenSequence::new();
                let mut attribute_stream = TokenStream::new(StringView::from(&attribute), 0);
                if b.ParseVariableDeclarationValue(attribute.clone(), false, context)
                    .is_none()
                {
                    substituted = String::new();
                } else if !this.ResolveTokensInto(
                    &mut attribute_stream,
                    scope.clone(),
                    resolver,
                    context,
                    frame.clone(),
                    kEOFToken,
                    &mut sequence,
                ) {
                    substituted = String::new();
                } else {
                    substituted = sequence.OriginalText();
                }
            }
            if b.ResolverInCycle(resolver) {
                substituted = String::new();
            }
            let mut local = CSSParserLocalContext::CreateWithoutPropertyForSyntaxParsing();
            let mut function = local.EnterFunction(CSSValueID::kAttr);
            let substitution = if substituted.IsNull() {
                None
            } else {
                b.ParseAttrType(&attr_type, substituted, context, function.Context())
            };
            if let Some(value) = substitution {
                return out.AppendValue(b, &value, true, 2097152);
            }
            let mut fallback = TokenSequence::new();
            if stream.Peek().GetType() == kCommaToken {
                stream.ConsumeRaw();
                stream.ConsumeWhitespace();
                if !this.ResolveTokensInto(
                    stream,
                    scope,
                    resolver,
                    context,
                    frame,
                    kEOFToken,
                    &mut fallback,
                ) {
                    return false;
                }
            } else if missing_type {
                if !fallback.AppendText(b, StringView::from("''"), true, 2097152) {
                    return false;
                }
            } else {
                return false;
            }
            out.AppendFallback(b, &fallback, true, 2097152)
        })
    }
    // cpp: style_cascade.cc:2523-2562.
    fn ResolveAutoBaseInto(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        out: &mut TokenSequence,
    ) -> bool {
        let name = CSSPropertyName::new(CSSPropertyID::kAppearance);
        let b = self.backend_;
        if b.ResolverDetectCycle(resolver, &b.PropertyCycleNode(&name)) {
            return false;
        }
        self.LookupAndApply(&name, resolver);
        let element = self.state_.GetElement();
        let builder = self.state_.StyleBuilder();
        let mut base = false;
        if b.ElementSupportsBaseAppearance(&element, builder.Appearance()) {
            base = true;
        } else if builder.InBaseAppearance() {
            base = !(b.ElementSupportsBaseAppearance(&element, AppearanceValue::kBase)
                || b.ElementSupportsBaseAppearance(&element, AppearanceValue::kBaseSelect));
        }
        drop(builder);
        if base {
            stream.SkipUntilPeekedTypeIs(&[kCommaToken]);
            stream.ConsumeIncludingWhitespace();
        }
        self.ResolveTokensInto(stream, scope, resolver, context, None, kCommaToken, out)
    }
    // cpp: style_cascade.cc:2984-2996.
    fn GetEnvironmentVariable(
        &self,
        name: &AtomicString,
        indices: Vec<u32>,
    ) -> Option<Variable<B>> {
        let b = self.backend_;
        let root = b.ElementTreeScopeRootShadowRoot(&self.state_.GetElement());
        let ua = root
            .as_ref()
            .is_some_and(|root| b.ShadowRootIsUserAgent(root));
        b.ResolveEnvironmentVariable(self.GetDocument(), name, indices, !ua)
    }
    // cpp: style_cascade.cc:2955-2975. Shared by keyword and conditional paths.
    fn ResolveLikeVar(
        &mut self,
        name: &AtomicString,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
    ) -> Option<Variable<B>> {
        let text = String::from_utf16(name.utf16_units().expect("variable name text"));
        let mut stream = TokenStream::new(StringView::from(&text), 0);
        let mut sequence = TokenSequence::new();
        if self.ResolveVarInto(&mut stream, None, resolver, context, frame, &mut sequence) {
            Some(sequence.BuildVariableData(self.backend_))
        } else {
            None
        }
    }
    // cpp: style_cascade.cc:2619-2731.
    pub fn CoerceIntoNumericValueForState(
        state: &'a StyleResolverState<'s, B>,
        backend: &'a B,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        context: &ParserContext<B>,
    ) -> Option<Rc<ResolverValue<B>>> {
        let mut cascade = Self::new(state, backend);
        let mut resolver = backend.NewCascadeResolver(CascadeFilter::new());
        let mut tainted = false;
        let mut local = CSSParserLocalContext::CreateWithoutPropertyForAtRules();
        cascade.CoerceIntoNumericValueInternal(
            value,
            scope,
            &mut resolver,
            context,
            None,
            &mut local,
            &mut tainted,
        )
    }
    fn CoerceIntoNumericValueInternal(
        &mut self,
        value: Rc<ResolverValue<B>>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        local: &mut CSSParserLocalContext,
        tainted: &mut bool,
    ) -> Option<Rc<ResolverValue<B>>> {
        let b = self.backend_;
        let text = b.VariableText(&b.UnparsedData(&value));
        let text = StringView::from(&text);
        let data = if self.IsVariableNameOnly(text.clone()) {
            self.ResolveLikeVar(
                &AtomicString::from_utf16(text.Span16()),
                resolver,
                context,
                frame,
            )
        } else {
            let mut stream = TokenStream::new(text, 0);
            let mut sequence = TokenSequence::new();
            if self.ResolveTokensInto(
                &mut stream,
                scope,
                resolver,
                context,
                frame,
                kEOFToken,
                &mut sequence,
            ) {
                Some(sequence.BuildVariableData(b))
            } else {
                None
            }
        };
        let Some(data) = data else {
            return None;
        };
        *tainted |= b.VariableAttrTainted(&data);
        local.SetRandomValueCount(b.ResolverRandomValueCount(resolver));
        let Some(parsed) = b.NumericSyntaxParse(
            b.VariableText(&data),
            context,
            local,
            b.VariableAnimationTainted(&data),
            b.VariableAttrTainted(&data),
        ) else {
            return None;
        };
        b.ResolverSetRandomValueCount(resolver, local.RandomValueCount());
        if b.ResolverCurrentPropertyName(resolver).is_none() && parsed.HasRandomFunctions() {
            return None;
        }
        if !parsed.IsPrimitiveValue() {
            return None;
        }
        if !b.PrimitiveIsCalculated(&parsed)
            && (b.PrimitiveIsPx(&parsed) || b.PrimitiveIsPercentage(&parsed))
        {
            return Some(parsed);
        }
        if b.PrimitiveIsLength(&parsed)
            || b.PrimitiveIsPercentage(&parsed)
            || !b.PrimitiveIsResolvableBeforeLayout(&parsed)
        {
            return Some(b.PrimitiveCreateFromLength(
                b.PrimitiveConvertToUnzoomedLength(&parsed, self.state_),
                1.0,
            ));
        }
        if b.PrimitiveIsNumber(&parsed) {
            return Some(b.NewNumericLiteralValue(
                b.PrimitiveComputeNumber(&parsed, self.state_),
                UnitType::kNumber,
            ));
        }
        if b.PrimitiveIsAngle(&parsed) {
            return Some(b.NewNumericLiteralValue(
                b.PrimitiveComputeDegrees(&parsed, self.state_),
                UnitType::kDegrees,
            ));
        }
        if b.PrimitiveIsTime(&parsed) {
            return Some(b.NewNumericLiteralValue(
                b.PrimitiveComputeSeconds(&parsed, self.state_),
                UnitType::kSeconds,
            ));
        }
        if b.PrimitiveIsResolution(&parsed) {
            return Some(b.NewNumericLiteralValue(
                b.PrimitiveComputeDotsPerPixel(&parsed, self.state_),
                UnitType::kDotsPerPixel,
            ));
        }
        None
    }
    // cpp: style_cascade.cc:2733-2852.
    fn EvalIfStyleFeature(
        &mut self,
        feature: &B::MediaQueryFeatureExpNode,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        tainted: &mut bool,
    ) -> KleeneValue {
        let b = self.backend_;
        let name = b.ResolverCurrentPropertyName(resolver);
        let mut local = self.GetCSSParserLocalContext(frame.as_ref(), name.as_ref());
        let mut function = local.EnterFunction(CSSValueID::kIf);
        let bounds = b.FeatureBounds(feature);
        if bounds.IsRange() {
            let mut result = KleeneValue::kTrue;
            let Some(reference) = self.CoerceIntoNumericValueInternal(
                b.FeatureReferenceValue(feature),
                scope.clone(),
                resolver,
                context,
                frame.clone(),
                function.Context(),
                tainted,
            ) else {
                return KleeneValue::kFalse;
            };
            for (comparison, left) in [(&bounds.left, true), (&bounds.right, false)] {
                if !comparison.IsValid() {
                    continue;
                }
                let value = Self::QueryCSSValue(&comparison.value);
                let Some(bound) = self.CoerceIntoNumericValueInternal(
                    value,
                    scope.clone(),
                    resolver,
                    context,
                    frame.clone(),
                    function.Context(),
                    tainted,
                ) else {
                    return KleeneValue::kFalse;
                };
                result = KleeneAnd(
                    result,
                    b.EvalStyleRange(&reference, &bound, comparison.op, left),
                );
            }
            return result;
        }
        let name = b.FeatureName(feature);
        let computed = self.ResolveLikeVar(&name, resolver, context, frame.clone());
        if b.ResolverInCycle(resolver) {
            return KleeneValue::kFalse;
        }
        if computed
            .as_ref()
            .is_some_and(|data| b.VariableAttrTainted(data))
        {
            *tainted = true;
        }
        if !bounds.right.value.IsValid() {
            return if computed.is_some() {
                KleeneValue::kTrue
            } else {
                KleeneValue::kFalse
            };
        }
        let query = Self::QueryCSSValue(&bounds.right.value);
        let query_data = if query.IsCSSWideKeyword() {
            if query.IsRevertValue() || query.IsRevertLayerValue() {
                return KleeneValue::kFalse;
            }
            let data = self.GetKeywordVariableData(&name, &query, resolver, context, frame);
            if computed.is_none() && data.is_none() {
                return KleeneValue::kTrue;
            }
            data
        } else {
            let syntax = self.FindVariableType(&name, frame.clone());
            self.ResolveTypedExpression(
                b.UnparsedData(&query),
                scope,
                syntax,
                resolver,
                context,
                frame,
                function.Context(),
            )
        };
        let (Some(computed), Some(query_data)) = (computed, query_data) else {
            return KleeneValue::kFalse;
        };
        if b.VariableAttrTainted(&query_data) {
            *tainted = true;
        }
        if b.VariableEqualsIgnoringAttrTainting(&computed, &query_data) {
            KleeneValue::kTrue
        } else {
            KleeneValue::kFalse
        }
    }
    fn QueryCSSValue(value: &MediaQueryExpValue<ResolverValue<B>>) -> Rc<ResolverValue<B>> {
        match value {
            MediaQueryExpValue::Value(value) => value.clone(),
            _ => unreachable!("source GetCSSValue requires CSSValue"),
        }
    }
    // cpp: style_cascade.cc:2854-2919. Own Handler methods are above; the
    // external ConditionalExpNode only calls them and applies its AST operators.
    fn EvalIfCondition(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        tainted: &mut bool,
    ) -> bool {
        let b = self.backend_;
        let root = b.ConsumeIfCondition(context, stream);
        stream.ConsumeWhitespace();
        stream.ConsumeIncludingWhitespace();
        let mut handler = IfHandler {
            cascade: self,
            resolver,
            context,
            frame,
            scope,
            attr_tainted: tainted,
        };
        b.EvaluateIfExpression(&root, &mut handler) == KleeneValue::kTrue
    }
    // cpp: style_cascade.cc:2921-2953.
    fn ResolveIfInto(
        &mut self,
        stream: &mut TokenStream<'_>,
        scope: Option<Rc<dyn TreeScope>>,
        resolver: &mut B::CascadeResolver,
        context: &ParserContext<B>,
        frame: Option<Frame<B>>,
        out: &mut TokenSequence,
    ) -> bool {
        stream.ConsumeWhitespace();
        let mut tainted = false;
        let mut eval = self.EvalIfCondition(
            stream,
            scope.clone(),
            resolver,
            context,
            frame.clone(),
            &mut tainted,
        );
        while !eval {
            stream.SkipUntilPeekedTypeIs(&[kSemicolonToken]);
            if stream.AtEnd() {
                return false;
            }
            stream.ConsumeIncludingWhitespace();
            if stream.AtEnd() {
                return false;
            }
            eval = self.EvalIfCondition(
                stream,
                scope.clone(),
                resolver,
                context,
                frame.clone(),
                &mut tainted,
            );
        }
        let mut result = TokenSequence::new();
        if !self.ResolveTokensInto(
            stream,
            scope,
            resolver,
            context,
            frame,
            kSemicolonToken,
            &mut result,
        ) {
            return false;
        }
        out.AppendSequence(self.backend_, &result, tainted, usize::MAX)
    }
}

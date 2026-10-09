// Copyright 1999-2026 The Chromium Authors and other contributors.
// Use of this source code is governed by the license in Chromium's
// third_party/blink/renderer/core/css/style_engine.h.
// Source ledger: comments and blanks stripped; braces retained.
// Effective = mapped + omitted boilerplate + production pending.
// Omitted: headers/namespace/access/macro/friend scaffolding, DCHECK/TRACE,
// debug-only checks, GC Trace/default destructor, Stats, and pure recalc timers.
// Isolated declarations with no C++ definition: h:415,878,933,955.
// No Pending* or internal-body backend delegation.
// style_engine.h: physical 1326, effective 843, mapped 678,
// omitted 165, production pending 0.
// style_engine.cc: physical 5095, effective 3981, mapped 3626,
// omitted 355, production pending 0.
// Final batch closes all previously pending production declarations/bodies:
// counter/font/property/viewport/environment and safe-area state, vision filters,
// DOM lifecycle roots/ancestor analysis, style recalc/layout rebuild, size and
// anchored interleaving, counter/function scope lookup, URI/image/random caches,
// inspector rule traversal, getters and retained owner fields.
// Batch h: effective 195, mapped 182, omitted 13, pending 0.
// Batch cc: effective 926, mapped 923, omitted 3, pending 0.
// Exact effective source lines closed by this batch (includes omitted lines):
// Batch h: 130,237-240,286,290-293,306,378-385,391,411-412,415,482-484,503,506,587-594,597,600,615-617,619,623-624
// Batch h: 626,640,648,650-655,657-658,660-662,664-668,674-676,678-680,682,684,686-693,697,699-701,705,712-721,723
// Batch h: 749-750,754-758,769-772,774,777-782,787,789-792,794-796,805-807,811,814-815,818,820-821,825,839-840
// Batch h: 850-851,853,871-872,878,890,892-893,895,897-898,933,942,955,969-970,972-978,988-989,994,1124,1130
// Batch h: 1138-1139,1154-1155,1160-1163,1174,1237-1243,1260,1265-1266,1274,1279,1295-1296,1298-1299,1309-1316
// Batch cc: 225-234,236-247,363-369,373-375,547,549-553,556-562,564-566,571-572,773-784,808-815,3238-3243,3245-3251
// Batch cc:3256-3260,3262-3264,3266-3274,3354,3359-3361,3363-3381,3385-3389,3401-3407,3409-3419,3421,3423-3431
// Batch cc:3433-3455,3457-3460,3462,3475-3485,3487-3493,3645-3649,3651-3654,3656-3658,3660-3665,3667-3671,3673-3674
// Batch cc:3676-3683,3685-3695,3697-3698,3702-3708,3713-3723,3725-3726,3775-3776,3791,3794-3795,3797-3799,3804,3810
// Batch cc:3812,3822-3825,3827-3833,3835-3845,3847-3850,3852-3853,3855-3857,3859-3863,3865-3868,3873-3874,3877
// Batch cc:3879-3880,3887-3890,3892,3894-3896,3898-3899,3901-3913,3915-3916,3926-3930,3932,3934,3942-3944,3946-3949
// Batch cc:3953-3955,3958-3961,3963,3968-3969,3971-3973,3975-3977,3980-3992,3994-4006,4008-4012,4014-4015,4017-4018
// Batch cc:4020-4021,4023-4027,4029-4034,4036-4045,4049-4051,4054-4056,4058-4061,4063-4071,4073-4074,4076-4081
// Batch cc:4083-4085,4087-4100,4102-4115,4117-4120,4122-4125,4127-4140,4142-4144,4147,4151-4152,4154-4158,4160-4174
// Batch cc:4176,4188-4193,4195,4205,4207-4210,4212-4219,4221-4223,4225-4227,4229-4238,4240,4245-4251,4253-4254
// Batch cc:4265-4268,4270,4283,4285-4294,4296-4305,4307-4311,4318,4320-4321,4323-4326,4330-4331,4333-4334,4336-4339
// Batch cc:4341-4342,4348-4349,4353-4356,4358-4369,4373-4387,4389-4395,4397-4398,4400-4401,4403-4405,4409-4411
// Batch cc:4413-4414,4416-4417,4424-4429,4552-4554,4556-4559,4561-4569,4574-4577,4583-4587,4645-4647,4649-4657
// Batch cc:4707-4712,4745-4748,4750-4755,4765-4787,4789-4799,4801-4806,4920-4923,4925-4932,4934-4936,4945-4958
// Batch cc:4960-4985,4989-4991,4995-5000,5002-5011,5015-5031,5035-5037,5044-5050,5052,5054,5058-5059,5061-5072
// Batch cc:5074-5075,5077-5085,5087-5093

use crate::active_style_sheets::{
    ActiveSheetsChange, ActiveStyleSheet, AffectedByMediaValueChange, ChangedRuleSets,
    CompareActiveStyleSheets, RuleSetDiff, StyleSheetMediaQueries,
};
use crate::cascade_layer::CascadeLayer;
use crate::cascade_layer_map::{CascadeLayerMap, CascadeLayerRuleSet};
use crate::cascade_layered::CascadeLayered;
use crate::color_scheme_flags::{ColorSchemeFlag, ColorSchemeFlags};
use crate::css_selector::{PseudoType, QualifiedName};
use crate::media_queries::forced_colors::ForcedColors;
use crate::media_queries::preferred_color_scheme::PreferredColorScheme;
use crate::media_queries::preferred_contrast::PreferredContrast;
use crate::media_value_change::MediaValueChange;
use crate::pending_sheet_type::PendingSheetType;
use crate::resolver::media_query_result::MediaQueryResultFlags;
use crate::style_sheet_contents::RenderBlockingBehavior;
use crate::vision_deficiency::{CreateVisionDeficiencyFilterUrl, VisionDeficiency};
use foundation::{AtomicString, Color, String};
use layoutng_style::style::computed_style_constants::PseudoId;
use layoutng_style::style::computed_style_constants::ViewportUnitFlag;
use layoutng_style::style::position_try_fallbacks::{
    kNoTryTactics, PositionTryFallback, TryTacticList,
};
use std::cell::Cell;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};

// cpp: third_party/blink/renderer/core/css/style_engine.h:127
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidationScope {
    kInvalidateCurrentScope,
    kInvalidateAllScopes,
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:129
pub type StyleSheetKey = AtomicString;

// Dependency types keep their allocation identity. No dependency has a default
// implementation: assembly must supply the actual Document, collection, parser,
// evaluator, and resolver operations. This batch does not translate their owners.
pub type ActiveStyleSheetVector<B> = Vec<
    ActiveStyleSheet<
        <B as StyleEngineSheetBackend>::CSSStyleSheet,
        <B as StyleEngineSheetBackend>::RuleSet,
    >,
>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebCssOrigin {
    kAuthor,
    kUser,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    kLight,
    kDark,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleChangeReason {
    DeclarativeContent,
    Settings,
    Shadow,
    ActiveStylesheetsUpdate,
    MediaQuery,
    NavigationQuery,
    PlatformColorChange,
    FunctionRuleChange,
    StyleRuleChange,
    PropertyRegistration,
    ViewportUnits,
    AffectedByHas,
    AttributeChange,
    EnvironmentVariableChanged,
    ViewportDefiningElement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleChangeType {
    kLocalStyleChange,
    kSubtreeStyleChange,
}

// Shared active-sheet state. Declaration source: style_engine.h:1063-1067.
pub struct ActiveSheetState<B: StyleEngineBackend> {
    pub document_scope_dirty: bool,
    pub tree_scopes_removed: bool,
    pub user_style_dirty: bool,
    pub dirty_tree_scopes: HashMap<usize, Rc<B::TreeScope>>,
    pub active_tree_scopes: HashMap<usize, Rc<B::TreeScope>>,
    pub active_user_style_sheets: ActiveStyleSheetVector<B>,
}

/// Required operations owned by Document/Node/Element, never by StyleEngine.
pub trait StyleEngineDocumentBackend: Sized + 'static {
    type Document;
    type TreeScope;
    type Node;
    type ContainerNode;
    type Element;
    fn DocumentTreeScope(document: &Self::Document) -> Rc<Self::TreeScope>;
    fn DocumentHasFrame(document: &Self::Document) -> bool;
    fn DocumentIsActive(document: &Self::Document) -> bool;
    fn DocumentIsDetached(document: &Self::Document) -> bool;
    fn ScheduleLayoutTreeUpdateIfNeeded(document: &Self::Document);
    fn NodeIsConnected(node: &Self::Node) -> bool;
    fn ContainerNodeAsNode(node: &Self::ContainerNode) -> &Self::Node;
    fn NodeIsXSLStyleSheet(node: &Self::Node) -> bool;
    fn NodeTreeScope(node: &Self::Node) -> Rc<Self::TreeScope>;
    fn ContainingShadowRoot(node: &Self::Node) -> Option<Rc<Self::TreeScope>>;
    fn TreeScopeRootIsConnected(scope: &Self::TreeScope) -> bool;
    // None denotes the absent RenderBlockingResourceManager from the source.
    fn AddPendingRenderBlockingStylesheet(
        document: &Self::Document,
        node: &Self::Node,
    ) -> Option<bool>;
    fn RemovePendingRenderBlockingStylesheet(
        document: &Self::Document,
        node: &Self::Node,
    ) -> Option<bool>;
    fn DocumentHasBody(document: &Self::Document) -> bool;
    fn CountPendingStylesheetAddedAfterBodyStarted(document: &Self::Document);
    fn DidAddPendingParserBlockingStylesheet(document: &Self::Document);
    fn DidLoadAllPendingParserBlockingStylesheets(document: &Self::Document);
    fn DidRemoveAllPendingStylesheets(document: &Self::Document);
    fn NewElement(
        document: &Rc<Self::Document>,
        prefix: AtomicString,
        local_name: AtomicString,
        namespace: AtomicString,
    ) -> Rc<Self::Element>;
    fn DocumentElement(document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn ElementSetNeedsStyleRecalc(
        element: &Self::Element,
        change: StyleChangeType,
        reason: StyleChangeReason,
    );
}

/// Required StyleSheetCollection/CSSStyleSheet/StyleSheetContents operations.
pub trait StyleEngineSheetBackend: StyleEngineDocumentBackend {
    type StyleSheet;
    type CSSStyleSheet: StyleSheetMediaQueries;
    type StyleSheetContents;
    type CSSParserContext: PartialEq;
    type StyleSheetCollection;
    type RuleSet: CascadeLayerRuleSet;
    type MixinMap;
    fn NewStyleSheetCollection(scope: &Rc<Self::TreeScope>) -> Rc<Self::StyleSheetCollection>;
    fn CollectionUpdateStyleSheetList(collection: &Self::StyleSheetCollection);
    fn CollectionActiveStyleSheets(
        collection: &Self::StyleSheetCollection,
    ) -> ActiveStyleSheetVector<Self>;
    fn CollectionStyleSheetsForList(
        collection: &Self::StyleSheetCollection,
    ) -> Vec<Rc<Self::StyleSheet>>;
    fn CollectionMarkSheetListDirty(collection: &Self::StyleSheetCollection);
    fn CollectionAddCandidate(collection: &Self::StyleSheetCollection, node: &Self::Node);
    fn CollectionRemoveCandidate(collection: &Self::StyleSheetCollection, node: &Self::Node);
    fn AddedAdoptedToTreeScope(sheet: &Self::CSSStyleSheet, scope: &Self::TreeScope);
    fn RemovedAdoptedFromTreeScope(sheet: &Self::CSSStyleSheet, scope: &Self::TreeScope);
    fn NewCSSStyleSheet(
        contents: Rc<Self::StyleSheetContents>,
        document: &Rc<Self::Document>,
    ) -> Rc<Self::CSSStyleSheet>;
    fn SheetContents(sheet: &Self::CSSStyleSheet) -> Rc<Self::StyleSheetContents>;
    fn ContentsCacheableForStyleElement(contents: &Self::StyleSheetContents) -> bool;
    fn ContentsParserContext(
        contents: &Self::StyleSheetContents,
    ) -> Option<Rc<Self::CSSParserContext>>;
    fn ContentsHasSingleOwnerDocument(contents: &Self::StyleSheetContents) -> bool;
    fn ContentsSetUsedFromTextCache(contents: &Self::StyleSheetContents);
    fn ContentsHasMediaQueries(contents: &Self::StyleSheetContents) -> bool;
    fn ContentsClearRuleSet(contents: &Self::StyleSheetContents);
    // base::FastHash is external; substituting a Rust hasher changes cache keys.
    fn FastHash(bytes: &[u8]) -> usize;
}

/// Required MediaQueryEvaluator, navigation expression and retained query types.
pub trait StyleEngineQueryBackend: StyleEngineSheetBackend {
    type MediaQueryEvaluator;
    type MediaQuerySet;
    type NavigationTestExpression;
    type TextTrack;
    type URLPattern;
    fn NewFrameMediaQueryEvaluator(document: &Self::Document) -> Rc<Self::MediaQueryEvaluator>;
    fn NewMediaQueryEvaluatorForType(media_type: &str) -> Rc<Self::MediaQueryEvaluator>;
    fn SheetMatchesMediaQueries(
        sheet: &Self::CSSStyleSheet,
        evaluator: &Self::MediaQueryEvaluator,
    ) -> bool;
    fn ContentsEnsureRuleSet(
        contents: &Self::StyleSheetContents,
        evaluator: &Self::MediaQueryEvaluator,
        mixins: &Self::MixinMap,
    ) -> Rc<Self::RuleSet>;
    fn ContentsCreateUnconnectedRuleSet(
        contents: &Self::StyleSheetContents,
        evaluator: &Self::MediaQueryEvaluator,
        mixins: &Self::MixinMap,
    ) -> Option<Rc<Self::RuleSet>>;
    fn EvalMediaQuery(
        evaluator: &Self::MediaQueryEvaluator,
        query: &Self::MediaQuerySet,
        flags: &mut MediaQueryResultFlags,
    ) -> bool;
    fn NavigationMatches(exp: &Self::NavigationTestExpression, document: &Self::Document) -> bool;
    fn MediaQueryResultsChanged(
        evaluator: &Self::MediaQueryEvaluator,
        results: &[(Rc<Self::MediaQuerySet>, bool)],
    ) -> bool;
    fn TextTrackCSSStyleSheets(track: &Self::TextTrack) -> Vec<Rc<Self::CSSStyleSheet>>;
    fn TextTrackOwner(track: &Self::TextTrack) -> Option<Rc<Self::Element>>;
}

/// Only resolver operations used by the independent bodies of this batch.
pub trait StyleEngineResolverBackend: StyleEngineSheetBackend + StyleEngineLayoutBackend {
    type StyleResolver;
    type CSSGlobalRuleSet;
    type ViewportStyleResolver;
    fn ViewportNeedsUpdate(resolver: &Self::ViewportStyleResolver) -> bool;
    fn ViewportUpdate(resolver: &Self::ViewportStyleResolver);
    fn ViewportSetNeedsUpdate(resolver: &Self::ViewportStyleResolver);
    fn GlobalRuleSetIsDirty(rules: &Self::CSSGlobalRuleSet) -> bool;
    fn WatchedSelectorsRuleSet(rules: &Self::CSSGlobalRuleSet) -> Option<Rc<Self::RuleSet>>;
    fn DocumentRulesSelectorsRuleSet(rules: &Self::CSSGlobalRuleSet) -> Option<Rc<Self::RuleSet>>;
    fn ResolverInvalidateInitialStyle(resolver: &Self::StyleResolver);
    fn ResolverUpdateMediaType(resolver: &Self::StyleResolver);
    fn InitWatchedSelectorsRuleSet(rules: &Self::CSSGlobalRuleSet, document: &Self::Document);
    fn DocumentHasScopedStyleResolver(document: &Self::Document) -> bool;
    fn ScopedResolverSetNeedsAppendAllSheets(document: &Self::Document);
    fn ResolverStyleForViewport(resolver: &Self::StyleResolver) -> Rc<Self::ComputedStyle>;
}

/// Required frame-view, ComputedStyle, LayoutObject and Element operations.
pub trait StyleEngineLayoutBackend: StyleEngineDocumentBackend {
    type LayoutObject;
    type ComputedStyle;
    type ComputedStyleDifference: PartialEq;
    fn ColorSchemeFlagsIsNormal(style: &Self::ComputedStyle) -> bool;
    fn UsedColorScheme(style: &Self::ComputedStyle) -> ColorScheme;
    fn DocumentViewHasSubtreeLayoutRoots(document: &Self::Document) -> Option<bool>;
    fn LayoutObjectElement(object: &Self::LayoutObject) -> Option<Rc<Self::Element>>;
    fn ElementHasFirstChild(element: &Self::Element) -> bool;
    fn ElementIsShadowHost(element: &Self::Element) -> bool;
    fn ElementIsActiveSlot(element: &Self::Element) -> bool;
    fn WhitespaceChildrenMayChange(object: &Self::LayoutObject) -> bool;
    fn SetWhitespaceChildrenMayChange(object: &Self::LayoutObject, value: bool);
    fn WasNotifiedOfSubtreeChange(object: &Self::LayoutObject) -> bool;
    fn NotifyOfSubtreeChange(object: &Self::LayoutObject) -> bool;
    fn MarkAncestorsWithChildNeedsStyleRecalc(element: &Self::Element);
    fn InvalidatePendingSVGResources(document: &Self::Document);
    fn FirstElementWithin(scope: &Self::TreeScope) -> Option<Rc<Self::Element>>;
    fn NextElementIncludingPseudo(element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ElementShadowRoot(element: &Self::Element) -> Option<Rc<Self::TreeScope>>;
    fn ElementComputedStyle(element: &Self::Element) -> Option<Rc<Self::ComputedStyle>>;
    fn PseudoElementStylesDependOnFunc(
        element: &Self::Element,
        predicate: &dyn Fn(&Self::ComputedStyle) -> bool,
    ) -> bool;
    fn AffectedByFunctionalMedia(style: &Self::ComputedStyle) -> bool;
    fn AffectedByFunctionalNavigation(style: &Self::ComputedStyle) -> bool;
    fn DocumentLayoutView(document: &Self::Document) -> Rc<Self::LayoutObject>;
    fn LayoutObjectStyle(object: &Self::LayoutObject) -> Rc<Self::ComputedStyle>;
    fn ComputeStyleDifference(
        a: &Self::ComputedStyle,
        b: &Self::ComputedStyle,
    ) -> Self::ComputedStyleDifference;
    fn EqualStyleDifference() -> Self::ComputedStyleDifference;
    fn LayoutObjectSetStyle(object: &Self::LayoutObject, style: Rc<Self::ComputedStyle>);
}

/// Constructors and color/view state owned by Document, Settings, and frame objects.
pub trait StyleEngineBootstrapBackend: StyleEngineResolverBackend {
    type StyleContainmentScopeTree;
    type CSSFontSelector;
    type Settings;
    type ViewportSize;
    type ColorProvider;
    fn NewStyleContainmentScopeTree() -> Rc<Self::StyleContainmentScopeTree>;
    fn NewStyleResolver(document: &Rc<Self::Document>) -> Rc<Self::StyleResolver>;
    fn NewGlobalRuleSet() -> Rc<Self::CSSGlobalRuleSet>;
    fn NewViewportStyleResolver(document: &Rc<Self::Document>) -> Rc<Self::ViewportStyleResolver>;
    fn FrameHasPagePopupOwner(document: &Self::Document) -> bool;
    fn NewPopupCSSFontSelector(document: &Rc<Self::Document>) -> Rc<Self::CSSFontSelector>;
    fn NewCSSFontSelector(document: &Rc<Self::Document>) -> Rc<Self::CSSFontSelector>;
    fn RegisterFontInvalidationCallbacks(
        selector: &Self::CSSFontSelector,
        engine: Weak<StyleEngine<Self>>,
    ) where
        Self: StyleEngineBackend;
    fn FrameOwnerColorSchemes(
        document: &Self::Document,
    ) -> Option<(ColorScheme, PreferredColorScheme)>;
    fn DocumentIsMainFrame(document: &Self::Document) -> bool;
    fn VisualViewportIsActive(document: &Self::Document) -> bool;
    fn DocumentSettings(document: &Self::Document) -> Option<Rc<Self::Settings>>;
    fn SettingsInForcedColors(settings: &Self::Settings) -> bool;
    fn SettingsPreferredColorScheme(settings: &Self::Settings) -> PreferredColorScheme;
    fn SettingsPreferredContrast(settings: &Self::Settings) -> PreferredContrast;
    fn SettingsForceDarkModeEnabled(settings: &Self::Settings) -> bool;
    fn MediaFeatureForcedColors(document: &Self::Document) -> Option<ForcedColors>;
    fn MediaFeaturePreferredColorScheme(document: &Self::Document) -> Option<PreferredColorScheme>;
    fn MediaFeaturePreferredContrast(document: &Self::Document) -> Option<PreferredContrast>;
    fn PreferencePreferredColorScheme(document: &Self::Document) -> Option<PreferredColorScheme>;
    fn PreferencePreferredContrast(document: &Self::Document) -> Option<PreferredContrast>;
    fn DocumentPrinting(document: &Self::Document) -> bool;
    fn CountColorSchemeFeature(document: &Self::Document, feature: ColorSchemeFeature);
    fn DocumentHasView(document: &Self::Document) -> bool;
    fn AboutBlankDarkModeOnUserActionEnabled() -> bool;
    fn DocumentURLIsAboutBlank(document: &Self::Document) -> bool;
    fn PageOpenedByDOM(document: &Self::Document) -> bool;
    fn FrameLoaderIsOnInitialEmptyDocument(document: &Self::Document) -> bool;
    fn ViewSetUseColorAdjustBackground(
        document: &Self::Document,
        mode: UseColorAdjustBackground,
        changed: bool,
    );
    fn ColorProviderForPainting(
        document: &Self::Document,
        scheme: ColorScheme,
        forced: bool,
    ) -> Rc<Self::ColorProvider>;
    fn DocumentInWebAppScope(document: &Self::Document) -> bool;
    fn DocumentIsInitialProfile(document: &Self::Document) -> bool;
    fn ThemeSystemCanvasColor(
        scheme: ColorScheme,
        provider: &Self::ColorProvider,
        web_app_initial_profile: bool,
    ) -> Color;
    fn LayoutViewIfPresent(document: &Self::Document) -> Option<Rc<Self::LayoutObject>>;
    fn InvalidatePaintForViewAndDescendants(view: &Self::LayoutObject);
    fn ViewportSizeFromLayoutView(view: Option<&Self::LayoutObject>) -> Self::ViewportSize;
}

/// StyleSheetCollection preparation and its committed result; the engine owns
/// application of the diff. Finish must invoke engine.ApplyRuleSetChanges.
pub trait StyleEngineCollectionBackend: StyleEngineQueryBackend {
    fn EmptyMixinMap() -> Self::MixinMap;
    fn CollectionPrepareUpdateActiveStyleSheets(
        collection: &Self::StyleSheetCollection,
        medium: &Self::MediaQueryEvaluator,
    );
    fn CollectionFinishUpdateActiveStyleSheets(
        collection: &Self::StyleSheetCollection,
        mixins: &Self::MixinMap,
        engine: &StyleEngine<Self>,
    ) where
        Self: StyleEngineBackend;
    fn CollectionMixins(collection: &Self::StyleSheetCollection) -> Self::MixinMap;
    fn MixinMapHasMixins(mixins: &Self::MixinMap) -> bool;
    fn MixinMapIdentifier(mixins: &Self::MixinMap) -> Option<u64>;
    fn MixinMapSetIdentifier(mixins: &mut Self::MixinMap, identifier: Option<u64>);
    fn MixinMapMerge(mixins: &mut Self::MixinMap, local: &Self::MixinMap);
    fn ParentTreeScope(scope: &Self::TreeScope) -> Option<Rc<Self::TreeScope>>;
    fn CollectionHasStyleSheetCandidateNodes(collection: &Self::StyleSheetCollection) -> bool;
    fn TreeScopeHasAdoptedStyleSheets(scope: &Self::TreeScope) -> bool;
    fn ActiveStyleSheetsUpdated(document: &Self::Document);
    fn GlobalRuleSetUpdate(rules: &Self::CSSGlobalRuleSet, document: &Self::Document)
    where
        Self: StyleEngineResolverBackend;
}

/// RuleSet records, scoped resolver, registries, and font cache operations.
pub trait StyleEngineRuleBackend: StyleEngineResolverBackend + StyleEngineBootstrapBackend {
    type RuleSetDiff: RuleSetDiff<Self::RuleSet>;
    type ScopedStyleResolver;
    type StyleRuleFontFace;
    type StyleRuleKeyframes;
    type StyleRuleProperty;
    type StyleRuleFontPaletteValues;
    type StyleRulePositionTry;
    type StyleRuleFunction;
    type StyleRuleViewTransition;
    type PropertyRegistration;
    type CSSValue;
    type FontFace;
    type CounterStyleMap;
    type RuleSetGroup;
    type StyleInitialData;
    fn RuleSetKeyframes(rules: &Self::RuleSet)
        -> Vec<CascadeLayered<Rc<Self::StyleRuleKeyframes>>>;
    fn RuleSetFontFaces(rules: &Self::RuleSet) -> Vec<CascadeLayered<Rc<Self::StyleRuleFontFace>>>;
    fn RuleSetProperties(rules: &Self::RuleSet)
        -> Vec<CascadeLayered<Rc<Self::StyleRuleProperty>>>;
    fn RuleSetFontPalettes(rules: &Self::RuleSet) -> Vec<Rc<Self::StyleRuleFontPaletteValues>>;
    fn RuleSetPositionTries(
        rules: &Self::RuleSet,
    ) -> Vec<CascadeLayered<Option<Rc<Self::StyleRulePositionTry>>>>;
    fn RuleSetFunctions(rules: &Self::RuleSet) -> Vec<CascadeLayered<Rc<Self::StyleRuleFunction>>>;
    fn RuleSetViewTransitions(
        rules: &Self::RuleSet,
    ) -> Vec<CascadeLayered<Rc<Self::StyleRuleViewTransition>>>;
    fn RuleSetHasFontFeatureValues(rules: &Self::RuleSet) -> bool;
    fn RuleSetHasCounterStyles(rules: &Self::RuleSet) -> bool;
    fn RuleSetCompactRulesIfNeeded(rules: &Self::RuleSet);
    fn GlobalRuleSetMarkDirty(rules: &Self::CSSGlobalRuleSet);
    fn ResolverInvalidateMatchedPropertiesCache(resolver: &Self::StyleResolver);
    fn TreeScopeScopedStyleResolver(
        scope: &Self::TreeScope,
    ) -> Option<Rc<Self::ScopedStyleResolver>>;
    fn EnsureScopedStyleResolver(scope: &Self::TreeScope) -> Rc<Self::ScopedStyleResolver>;
    fn ClearScopedStyleResolver(scope: &Self::TreeScope);
    fn ScopedResolverNeedsAppendAllSheets(resolver: &Self::ScopedStyleResolver) -> bool;
    fn ScopedResolverSetAppendAllSheets(resolver: &Self::ScopedStyleResolver);
    fn ScopedResolverResetStyle(resolver: &Self::ScopedStyleResolver);
    fn ScopedResolverCascadeLayerMap(
        resolver: &Self::ScopedStyleResolver,
    ) -> Option<Rc<CascadeLayerMap>>;
    fn ScopedResolverRebuildCascadeLayerMap(
        resolver: &Self::ScopedStyleResolver,
        sheets: &ActiveStyleSheetVector<Self>,
    );
    fn ScopedResolverAppendActiveStyleSheets(
        resolver: &Self::ScopedStyleResolver,
        start: usize,
        sheets: &ActiveStyleSheetVector<Self>,
    );
    fn ScopedResolverKeyframesRulesAdded(scope: &Self::TreeScope);
    fn FontFaceCacheClearCSSConnected(selector: &Self::CSSFontSelector) -> bool;
    fn FontFaceCreate(
        document: &Rc<Self::Document>,
        rule: &CascadeLayered<Rc<Self::StyleRuleFontFace>>,
        is_user: bool,
    ) -> Option<Rc<Self::FontFace>>;
    fn FontFaceCacheAdd(
        selector: &Self::CSSFontSelector,
        rule: &Self::StyleRuleFontFace,
        face: Rc<Self::FontFace>,
    );
    fn FontFaceGeneralInvalidation(selector: &Self::CSSFontSelector);
    fn KeyframesName(rule: &Self::StyleRuleKeyframes) -> AtomicString;
    fn KeyframesVendorPrefixed(rule: &Self::StyleRuleKeyframes) -> bool;
    fn PropertyName(rule: &Self::StyleRuleProperty) -> AtomicString;
    fn MaybeCreateDeclaredProperty(
        document: &Self::Document,
        name: &AtomicString,
        rule: &Self::StyleRuleProperty,
    ) -> Option<Rc<Self::PropertyRegistration>>;
    fn RemoveDeclaredProperties(document: &Self::Document);
    fn DeclareProperty(
        document: &Self::Document,
        name: AtomicString,
        registration: &Self::PropertyRegistration,
    );
    fn FontPaletteName(rule: &Self::StyleRuleFontPaletteValues) -> AtomicString;
    fn FontPaletteFamilyValue(rule: &Self::StyleRuleFontPaletteValues) -> Rc<Self::CSSValue>;
    fn CSSValueListItems(value: &Self::CSSValue) -> Option<Vec<Rc<Self::CSSValue>>>;
    fn CSSFontFamilyValue(value: &Self::CSSValue) -> Option<AtomicString>;
    fn FoldCase(value: &String) -> String;
    fn PositionTryName(rule: &Self::StyleRulePositionTry) -> AtomicString;
    fn FunctionName(rule: &Self::StyleRuleFunction) -> AtomicString;
    fn ViewTransitionNavigation(rule: &Self::StyleRuleViewTransition) -> ViewTransitionNavigation;
    fn ViewTransitionTypes(rule: &Self::StyleRuleViewTransition) -> Vec<String>;
    fn TwoPhaseViewTransitionEnabled() -> bool;
    fn OnViewTransitionsStyleUpdated(
        document: &Self::Document,
        cross_document: bool,
        types: Vec<String>,
        preview_types: Option<Vec<String>>,
    );
    fn CreateUserCounterStyleMap(document: &Self::Document) -> Rc<Self::CounterStyleMap>;
    fn CounterStyleMapDispose(map: &Self::CounterStyleMap);
    fn CounterStyleMapAddCounterStyles(map: &Self::CounterStyleMap, rules: &Self::RuleSet);
    fn NewRuleSetGroup(index: u32) -> Self::RuleSetGroup;
    fn RuleSetGroupAdd(group: &mut Self::RuleSetGroup, rules: Rc<Self::RuleSet>);
    fn RuleSetGroupIsFull(group: &Self::RuleSetGroup) -> bool;
    fn RuleSetGroupIsEmpty(group: &Self::RuleSetGroup) -> bool;
}

/// Selector matching contexts and traversal objects. Matching stays with the
/// collector; invalidation policy and recursive traversal stay in StyleEngine.
pub trait StyleEngineInvalidationBackend: StyleEngineRuleBackend {
    type SelectorFilter;
    type SelectorFilterMark;
    type StyleScopeFrame;
    type StyleRecalcContext;
    type ElementRuleCollector;
    type NthIndexCache;
    fn HasPendingForcedStyleRecalc(document: &Self::Document) -> bool;
    fn InvalidationRootForTreeScope(scope: &Self::TreeScope) -> Rc<Self::Element>;
    fn ElementHasSubtreeStyleChange(element: &Self::Element) -> bool;
    fn NewSelectorFilter() -> Self::SelectorFilter;
    fn SelectorFilterPushAllParentsOf(filter: &mut Self::SelectorFilter, scope: &Self::TreeScope);
    fn SelectorFilterPushParent(filter: &mut Self::SelectorFilter, element: &Self::Element);
    fn SelectorFilterPopParent(filter: &mut Self::SelectorFilter, element: &Self::Element);
    fn SelectorFilterSetMark(filter: &Self::SelectorFilter) -> Self::SelectorFilterMark;
    fn SelectorFilterPopTo(filter: &mut Self::SelectorFilter, mark: Self::SelectorFilterMark);
    fn TreeScopeShadowHost(scope: &Self::TreeScope) -> Option<Rc<Self::Element>>;
    fn TreeScopeRootNode(scope: &Self::TreeScope) -> Rc<Self::ContainerNode>;
    fn ContainerNodeShadowHost(node: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn ContainerElementChildren(node: &Self::ContainerNode) -> Vec<Rc<Self::Element>>;
    fn ElementChildren(element: &Self::Element) -> Vec<Rc<Self::Element>>;
    fn ElementHasPartAttribute(element: &Self::Element) -> bool;
    fn ElementIsHTMLSlot(element: &Self::Element) -> bool;
    fn FlattenedAssignedElements(slot: &Self::Element) -> Vec<Rc<Self::Element>>;
    fn NewStyleScopeFrame(
        element: &Self::Element,
        parent: Option<&Self::StyleScopeFrame>,
    ) -> Self::StyleScopeFrame;
    fn NewNthIndexCache(document: &Self::Document) -> Self::NthIndexCache;
    fn RecalcContextFromAncestors(
        element: &Self::Element,
        frame: &Self::StyleScopeFrame,
    ) -> Self::StyleRecalcContext;
    fn RecalcContextFromParent(
        parent: &Self::StyleRecalcContext,
        element: &Self::Element,
        frame: &Self::StyleScopeFrame,
    ) -> Self::StyleRecalcContext;
    fn AffectedByCSSFunction(style: &Self::ComputedStyle) -> bool;
    fn NewElementRuleCollector(
        element: &Self::Element,
        context: &Self::StyleRecalcContext,
        filter: &Self::SelectorFilter,
    ) -> Self::ElementRuleCollector;
    fn CollectorCheckAnyRuleMatches(
        collector: &mut Self::ElementRuleCollector,
        group: &Self::RuleSetGroup,
        scope: &Self::TreeScope,
    ) -> bool;
    fn CollectorCheckAnyShadowHostRuleMatches(
        collector: &mut Self::ElementRuleCollector,
        group: &Self::RuleSetGroup,
        scope: &Self::TreeScope,
    ) -> bool;
    fn CollectorMatchFlags(collector: &Self::ElementRuleCollector) -> InvalidationMatchFlags;
    fn RuleSetHasSlottedRules(rules: &Self::RuleSet) -> bool;
    fn RuleSetHasPartPseudoRules(rules: &Self::RuleSet) -> bool;
    fn RuleSetHasUAShadowPseudoElementRules(rules: &Self::RuleSet) -> bool;
}

#[derive(Clone, Copy)]
pub struct InvalidationMatchFlags {
    pub affected_by_drag: bool,
    pub affected_by_focus_within: bool,
    pub affected_by_hover: bool,
    pub affected_by_active: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewTransitionNavigation {
    kAuto,
    kNone,
    kUnspecified,
    kPreview,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UseColorAdjustBackground {
    kNo,
    kIfBaseNotTransparent,
    kYes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSchemeFeature {
    kForcedDarkMode,
    kPreferredColorSchemeDark,
    kPreferredColorSchemeDarkSetting,
    kColorSchemeDarkSupportedOnRoot,
}

/// Resolver teardown, scope trees, font selector and inline-sheet constructors.
pub trait StyleEngineLifecycleBackend: StyleEngineRuleBackend {
    type ScrollTargetGroupScopeTree;
    type StyleRuleUsageTracker;
    type Font;
    type FontSelector;
    type FontInvalidationReason;
    type CSSPropertyValueSet;
    type DocumentStyleEnvironmentVariables;
    type StyleInvalidationRoot;
    type StyleRecalcRoot;
    type LayoutTreeRebuildRoot;
    type TextPosition;
    type SheetURL;
    type TextEncoding;
    fn NewScrollTargetGroupScopeTree() -> Rc<Self::ScrollTargetGroupScopeTree>;
    fn ResolverSetRuleUsageTracker(
        resolver: &Self::StyleResolver,
        tracker: Option<Rc<Self::StyleRuleUsageTracker>>,
    );
    fn ResolverComputeFont(
        resolver: &Self::StyleResolver,
        element: &Self::Element,
        style: &Self::ComputedStyle,
        properties: &Self::CSSPropertyValueSet,
    ) -> Option<Rc<Self::Font>>;
    fn ResolverDispose(resolver: &Self::StyleResolver);
    fn GlobalRuleSetDispose(rules: &Self::CSSGlobalRuleSet);
    fn NewStyleInvalidationRoot() -> Self::StyleInvalidationRoot;
    fn NewStyleRecalcRoot() -> Self::StyleRecalcRoot;
    fn NewLayoutTreeRebuildRoot() -> Self::LayoutTreeRebuildRoot;
    fn ClearStyleInvalidationRoot(root: &mut Self::StyleInvalidationRoot);
    fn ClearStyleRecalcRoot(root: &mut Self::StyleRecalcRoot);
    fn ClearLayoutTreeRebuildRoot(root: &mut Self::LayoutTreeRebuildRoot);
    fn FontFaceCacheClearAll(selector: &Self::CSSFontSelector);
    fn EnvironmentVariablesDetachFromParent(variables: &Self::DocumentStyleEnvironmentVariables);
    fn FontSelectorUpdateGenericFontFamilySettings(
        selector: &Self::CSSFontSelector,
        document: &Self::Document,
    );
    fn FontFaceCacheRemove(selector: &Self::CSSFontSelector, rule: &Self::StyleRuleFontFace);
    fn ElementAsNode(element: &Self::Element) -> &Self::Node;
    fn NullSheetURL() -> Self::SheetURL;
    fn DocumentEncoding(document: &Self::Document) -> Option<Self::TextEncoding>;
    fn InlineParserContext(
        document: &Rc<Self::Document>,
        base_url: &Self::SheetURL,
        encoding: Option<Self::TextEncoding>,
    ) -> Rc<Self::CSSParserContext>;
    fn NewStyleSheetContents(
        parser_context: Rc<Self::CSSParserContext>,
    ) -> Rc<Self::StyleSheetContents>;
    fn ContentsSetRenderBlocking(
        contents: &Self::StyleSheetContents,
        behavior: RenderBlockingBehavior,
    );
    fn CreateInlineSheet(
        contents: Rc<Self::StyleSheetContents>,
        element: &Rc<Self::Element>,
        position: Self::TextPosition,
    ) -> Rc<Self::CSSStyleSheet>;
    fn ContentsParseString(contents: &Self::StyleSheetContents, text: &String);
    fn ElementIsInShadowTree(element: &Self::Element) -> bool;
    fn ElementTitle(element: &Self::Element) -> String;
    fn SheetSetTitle(sheet: &Self::CSSStyleSheet, title: String);
    fn ElementMarkSubtreeNeedsStyleRecalcForFontUpdates(element: &Self::Element);
    fn InvalidateSubtreeLayoutForFontUpdates(view: &Self::LayoutObject);
    fn FontsUpdated(document: &Self::Document);
    fn DocumentInStyleRecalc(document: &Self::Document) -> bool;
    fn ElementInActiveDocument(element: &Self::Element) -> bool;
    fn DocumentHasSubtreeStyleChange(document: &Self::Document) -> bool;
    fn ElementParentNode(element: &Self::Element) -> Option<Rc<Self::ContainerNode>>;
    fn ContainerHasSubtreeStyleChange(node: &Self::ContainerNode) -> bool;
    fn AncestorsOrAncestorSiblingsAffectedByHas(element: &Self::Element) -> bool;
    fn SiblingsAffectedByHasFlags(element: &Self::Element) -> u32;
    fn AffectedByLogicalCombinationsInHas(element: &Self::Element) -> bool;
    fn HasSiblingsAffectedByHasForSiblingDescendantRelationship(element: &Self::Element) -> bool;
    fn NodeAsElement(node: &Self::Node) -> Option<Rc<Self::Element>>;
    fn PreviousElementSibling(node: &Self::Node) -> Option<Rc<Self::Element>>;
}

/// Feature aggregation keeps shared contents visited identities in the engine.
pub trait StyleEngineFeatureBackend: StyleEngineRuleBackend {
    type RuleFeatureSet;
    fn SheetMediaQueryResultFlags(sheet: &Self::CSSStyleSheet) -> MediaQueryResultFlags;
    fn FeaturesMediaQueryResultFlags(
        features: &mut Self::RuleFeatureSet,
    ) -> &mut MediaQueryResultFlags;
    fn ContentsRuleSet(contents: &Self::StyleSheetContents) -> Rc<Self::RuleSet>;
    fn RuleSetFeatures(rules: &Self::RuleSet) -> Rc<Self::RuleFeatureSet>;
    fn FeaturesMerge(features: &mut Self::RuleFeatureSet, other: &Self::RuleFeatureSet);
    fn ScopedResolverCollectFeatures(
        resolver: &Self::ScopedStyleResolver,
        features: &mut Self::RuleFeatureSet,
        visited: &mut HashMap<usize, Rc<Self::StyleSheetContents>>,
    );
    fn InitialDataViewportUnitFlags(data: &Self::StyleInitialData) -> u32;
    fn ComputedStyleViewportUnitFlags(style: &Self::ComputedStyle) -> u32;
}

/// Counter contexts and layout objects own ordinal/counter values; recursion
/// and list-item invalidation policy are translated in StyleEngine.
pub trait StyleEngineCounterBackend: StyleEngineLayoutBackend {
    type CountersAttachmentContext;
    type LayoutCounter;
    type LayoutListItem;
    type LayoutInlineListItem;
    type ListItemOrdinal;
    type PseudoElement;
    type ContentData;
    type AltCounterContentData;
    fn NewCountersAttachmentContext() -> Self::CountersAttachmentContext;
    fn SetCounterAttachmentRootIsDocumentElement(context: &mut Self::CountersAttachmentContext);
    fn CounterContextEnterObject(
        context: &mut Self::CountersAttachmentContext,
        object: &Self::LayoutObject,
    );
    fn CounterContextLeaveObject(
        context: &mut Self::CountersAttachmentContext,
        object: &Self::LayoutObject,
    );
    fn ElementLayoutObject(element: &Self::Element) -> Option<Rc<Self::LayoutObject>>;
    fn LayoutAsListItem(object: &Self::LayoutObject) -> Option<Rc<Self::LayoutListItem>>;
    fn LayoutAsInlineListItem(
        object: &Self::LayoutObject,
    ) -> Option<Rc<Self::LayoutInlineListItem>>;
    fn ListItemOrdinal(item: &Self::LayoutListItem) -> Rc<Self::ListItemOrdinal>;
    fn InlineListItemOrdinal(item: &Self::LayoutInlineListItem) -> Rc<Self::ListItemOrdinal>;
    fn OrdinalUseExplicitValue(ordinal: &Self::ListItemOrdinal) -> bool;
    fn OrdinalMarkDirty(ordinal: &Self::ListItemOrdinal);
    fn ListItemOrdinalValueChanged(item: &Self::LayoutListItem);
    fn InlineListItemOrdinalValueChanged(item: &Self::LayoutInlineListItem);
    fn CSSListCounterAccountingEnabled() -> bool;
    fn ContentBehavesAsNormal(style: &Self::ComputedStyle) -> bool;
    fn LayoutTreeBuilderElementChildren(element: &Self::Element) -> Vec<Rc<Self::Element>>;
    fn NextLayoutObjectInPreOrder(
        object: &Self::LayoutObject,
        root: &Self::LayoutObject,
    ) -> Option<Rc<Self::LayoutObject>>;
    fn LayoutAsCounter(object: &Self::LayoutObject) -> Option<Rc<Self::LayoutCounter>>;
    fn LayoutCounterIdentifier(counter: &Self::LayoutCounter) -> AtomicString;
    fn LayoutCounterSeparatorIsNull(counter: &Self::LayoutCounter) -> bool;
    fn CounterValues(
        context: &mut Self::CountersAttachmentContext,
        object: &Self::LayoutObject,
        identifier: &AtomicString,
        single: bool,
    ) -> Vec<i32>;
    fn LayoutCounterUpdate(counter: &Self::LayoutCounter, values: Vec<i32>);
    fn LayoutObjectPseudoElement(object: &Self::LayoutObject) -> Option<Rc<Self::PseudoElement>>;
    fn CreateMutableAltContentDataForCountersIfNeeded(
        pseudo: &Self::PseudoElement,
    ) -> Option<Rc<Self::ContentData>>;
    fn ContentDataNext(content: &Self::ContentData) -> Option<Rc<Self::ContentData>>;
    fn ContentAsAltCounter(content: &Self::ContentData) -> Option<Rc<Self::AltCounterContentData>>;
    fn AltCounterUpdateText(
        content: &Self::AltCounterContentData,
        context: &mut Self::CountersAttachmentContext,
        engine: &StyleEngine<Self>,
        object: &Self::LayoutObject,
    ) where
        Self: StyleEngineBackend;
}

// Ownership adapter for a C++ Member<T> that can be cleared from shared engine
// entry points. get() retains the object before calls that may reenter/detach.
struct Retained<T>(RefCell<Option<Rc<T>>>);
impl<T> Retained<T> {
    fn new(value: Option<Rc<T>>) -> Self {
        Self(RefCell::new(value))
    }
    fn get(&self) -> Option<Rc<T>> {
        self.0.borrow().clone()
    }
    fn set(&self, value: Option<Rc<T>>) {
        *self.0.borrow_mut() = value;
    }
}

/// DOM relationships and dependency flags used by selector invalidation.
pub trait StyleEngineMutationBackend: StyleEngineLifecycleBackend {
    type SpaceSplitString;
    fn NodeStyleEngine(node: &Self::Node) -> Rc<StyleEngine<Self>>
    where
        Self: StyleEngineBackend;
    fn SpaceSplitStringValues(classes: &Self::SpaceSplitString) -> &[AtomicString];
    fn ElementParentElement(element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ParentOrShadowHostElement(element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ParentElementOrShadowRoot(element: &Self::Element) -> Option<Rc<Self::ContainerNode>>;
    fn ContainerAsElement(node: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn NodeParentNode(node: &Self::Node) -> Option<Rc<Self::ContainerNode>>;
    fn NodeNextSibling(node: &Self::Node) -> Option<Rc<Self::Node>>;
    fn NodePreviousSibling(node: &Self::Node) -> Option<Rc<Self::Node>>;
    fn ChildrenAffectedByForwardPositionalRules(parent: &Self::ContainerNode) -> bool;
    fn ChildrenAffectedByBackwardPositionalRules(parent: &Self::ContainerNode) -> bool;
    fn ChildrenAffectedByIndirectAdjacentRules(parent: &Self::ContainerNode) -> bool;
    fn ChildrenAffectedByDirectAdjacentRules(parent: &Self::Element) -> bool;
    fn AffectedByPseudoInHas(element: &Self::Element) -> bool;
    fn AffectedBySubjectHas(element: &Self::Element) -> bool;
    fn AffectedByNonSubjectHas(element: &Self::Element) -> bool;
    fn SetSiblingsAffectedByHasFlags(element: &Self::Element, flags: u32);
    fn SetAncestorsOrAncestorSiblingsAffectedByHas(element: &Self::Element);
    fn ElementDescendants(element: &Self::Element) -> Vec<Rc<Self::Element>>;
    fn HasAttrFunction(style: &Self::ComputedStyle) -> bool;
    fn PseudoElementStylesDependOnAttr(element: &Self::Element) -> bool;
    fn ElementNeedsStyleRecalc(element: &Self::Element) -> bool;
    fn ElementHasID(element: &Self::Element) -> bool;
    fn ElementIDForStyleResolution(element: &Self::Element) -> AtomicString;
    fn ElementHasClass(element: &Self::Element) -> bool;
    fn ElementClassNames(element: &Self::Element) -> Rc<Self::SpaceSplitString>;
    fn ElementAttributeNames(element: &Self::Element) -> Vec<QualifiedName>;
}

/// RuleInvalidationData owns collection/matching of invalidation sets.
pub trait StyleEngineRuleInvalidationBackend: StyleEngineFeatureBackend {
    type RuleInvalidationData;
    type InvalidationLists;
    type InvalidationSet;
    fn GlobalRuleFeatureSet(rules: &Self::CSSGlobalRuleSet) -> Rc<Self::RuleFeatureSet>;
    fn FeaturesRuleInvalidationData(
        features: &Self::RuleFeatureSet,
    ) -> Rc<Self::RuleInvalidationData>;
    fn NewInvalidationLists() -> Self::InvalidationLists;
    fn UsesHasInsideNth(data: &Self::RuleInvalidationData) -> bool;
    fn NeedsHasInvalidationForClassChange(data: &Self::RuleInvalidationData) -> bool;
    fn NeedsHasInvalidationForClass(data: &Self::RuleInvalidationData, name: &AtomicString)
        -> bool;
    fn NeedsHasInvalidationForAttributeChange(data: &Self::RuleInvalidationData) -> bool;
    fn NeedsHasInvalidationForAttribute(
        data: &Self::RuleInvalidationData,
        name: &QualifiedName,
    ) -> bool;
    fn NeedsHasInvalidationForIdChange(data: &Self::RuleInvalidationData) -> bool;
    fn NeedsHasInvalidationForId(data: &Self::RuleInvalidationData, id: &AtomicString) -> bool;
    fn NeedsHasInvalidationForPseudoStateChange(data: &Self::RuleInvalidationData) -> bool;
    fn NeedsHasInvalidationForPseudoClass(
        data: &Self::RuleInvalidationData,
        pseudo: PseudoType,
    ) -> bool;
    fn NeedsHasInvalidationForInsertionOrRemoval(data: &Self::RuleInvalidationData) -> bool;
    fn NeedsHasInvalidationForInsertedOrRemovedElement(
        data: &Self::RuleInvalidationData,
        element: &Self::Element,
    ) -> bool;
    fn InvalidatesParts(data: &Self::RuleInvalidationData) -> bool;
    fn HasSelectorForId(data: &Self::RuleInvalidationData, id: &AtomicString) -> bool;
    fn RuleInvalidationMaxDirectAdjacentSelectors(data: &Self::RuleInvalidationData) -> u32;
    fn CollectInvalidationSetsForClass(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        element: &Self::Element,
        name: &AtomicString,
    );
    fn CollectInvalidationSetsForAttribute(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        element: &Self::Element,
        name: &QualifiedName,
    );
    fn CollectInvalidationSetsForId(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        element: &Self::Element,
        id: &AtomicString,
    );
    fn CollectInvalidationSetsForPseudoClass(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        element: &Self::Element,
        pseudo: PseudoType,
    );
    fn CollectSiblingInvalidationSetForId(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        element: &Self::Element,
        id: &AtomicString,
        min: u32,
    );
    fn CollectSiblingInvalidationSetForClass(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        element: &Self::Element,
        name: &AtomicString,
        min: u32,
    );
    fn CollectSiblingInvalidationSetForAttribute(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        element: &Self::Element,
        name: &QualifiedName,
        min: u32,
    );
    fn CollectUniversalSiblingInvalidationSet(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
        min: u32,
    );
    fn CollectNthInvalidationSet(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
    );
    fn CollectPartInvalidationSet(
        data: &Self::RuleInvalidationData,
        lists: &mut Self::InvalidationLists,
    );
    fn TreeCountingInvalidationSet() -> Rc<Self::InvalidationSet>;
    fn InvalidationListsPushSibling(
        lists: &mut Self::InvalidationLists,
        set: Rc<Self::InvalidationSet>,
    );
}

/// PendingInvalidations/StyleInvalidator and UA sheet owners; no engine policy.
pub trait StyleEnginePendingInvalidationBackend:
    StyleEngineRuleInvalidationBackend + StyleEngineLifecycleBackend + StyleEngineQueryBackend
{
    type PendingInvalidations;
    type StyleInvalidator;
    type PropertyRegistry;
    type ViewTransition;
    fn NewPendingInvalidations() -> Self::PendingInvalidations;
    fn ScheduleInvalidationSetsForNode(
        pending: &Self::PendingInvalidations,
        lists: Self::InvalidationLists,
        node: &Self::Node,
    );
    fn ScheduleSiblingInvalidationsAsDescendants(
        pending: &Self::PendingInvalidations,
        lists: Self::InvalidationLists,
        parent: &Self::ContainerNode,
    );
    fn StyleInvalidationRootElement(
        root: &Self::StyleInvalidationRoot,
    ) -> Option<Rc<Self::Element>>;
    fn NewStyleInvalidator(pending: &Self::PendingInvalidations) -> Self::StyleInvalidator;
    fn StyleInvalidatorInvalidate(
        invalidator: &mut Self::StyleInvalidator,
        document: &Self::Document,
        root: Option<&Self::Element>,
    );
    fn FeaturesHasViewportDependentMediaQueries(features: &Self::RuleFeatureSet) -> bool;
    fn DocumentPropertyRegistry(document: &Self::Document) -> Option<Rc<Self::PropertyRegistry>>;
    fn PropertyRegistryViewportUnitFlags(registry: &Self::PropertyRegistry) -> u32;
    fn GlobalHasFullscreenUAStyle(rules: &Self::CSSGlobalRuleSet) -> bool;
    fn DefaultSheetsEnsureFullscreen(element: &Self::Element);
    fn DefaultSheetsEnsureElement(element: &Self::Element) -> bool;
    fn DefaultSheetsEnsurePseudoElement(pseudo: PseudoId) -> bool;
    fn DefaultSheetsEnsureForcedColors() -> bool;
    fn ElementViewTransition(element: &Self::Element) -> Option<Rc<Self::ViewTransition>>;
    fn ViewTransitionUAStyleSheet(transition: &Self::ViewTransition) -> Rc<Self::CSSStyleSheet>;
    fn DefaultSheetsScreenEvaluator() -> Rc<Self::MediaQueryEvaluator>;
}

// cpp: style_engine.cc:1553-1706
struct PseudoHasInvalidationTraversalContext<B: StyleEngineBackend> {
    first_element_: Option<Rc<B::Element>>,
    is_first_element_shadow_host_: bool,
    traverse_to_parent_of_first_element_: bool,
    for_element_affected_by_pseudo_in_has_: bool,
}
impl<B: StyleEngineBackend> PseudoHasInvalidationTraversalContext<B> {
    fn new(first: Option<Rc<B::Element>>, shadow: bool, parent: bool) -> Self {
        Self {
            first_element_: first,
            is_first_element_shadow_host_: shadow,
            traverse_to_parent_of_first_element_: parent,
            for_element_affected_by_pseudo_in_has_: false,
        }
    }
    fn SetForElementAffectedByPseudoInHas(mut self) -> Self {
        self.for_element_affected_by_pseudo_in_has_ = true;
        self
    }
    fn ForAttributeOrPseudoStateChange(changed: &B::Element) -> Self {
        let ancestors = B::AncestorsOrAncestorSiblingsAffectedByHas(changed);
        let mut first = None;
        let mut shadow = false;
        if ancestors {
            first = B::ElementParentElement(changed);
            if first.is_none() {
                first = B::ParentOrShadowHostElement(changed);
                shadow = first.is_some();
            }
        }
        let previous = if B::SiblingsAffectedByHasFlags(changed) != 0 {
            B::PreviousElementSibling(B::ElementAsNode(changed))
        } else {
            None
        };
        if let Some(previous) = previous {
            first = Some(previous);
            shadow = false;
        }
        Self::new(first, shadow, ancestors)
    }
    fn ForInsertion(
        parent: Option<Rc<B::Element>>,
        shadow_child: bool,
        previous: Option<Rc<B::Element>>,
    ) -> Self {
        let mut first = parent;
        let mut shadow = false;
        let mut ancestors = false;
        if let Some(first) = &first {
            ancestors = B::AncestorsOrAncestorSiblingsAffectedByHas(first);
            shadow = shadow_child;
        }
        if let Some(previous) = previous {
            first = Some(previous);
            shadow = false;
        }
        Self::new(first, shadow, ancestors)
    }
    fn ForRemoval(
        parent: Option<Rc<B::Element>>,
        shadow_child: bool,
        mut previous: Option<Rc<B::Element>>,
        removed: &B::Element,
    ) -> Self {
        let ancestors = B::AncestorsOrAncestorSiblingsAffectedByHas(removed);
        let mut first = if ancestors { parent } else { None };
        let mut shadow = first.is_some() && shadow_child;
        if B::SiblingsAffectedByHasFlags(removed) == 0 {
            previous = None;
        }
        if let Some(previous) = previous {
            first = Some(previous);
            shadow = false;
        }
        Self::new(first, shadow, ancestors)
    }
    fn ForAllChildrenRemoved(parent: &Rc<B::Element>) -> Self {
        Self::new(
            Some(parent.clone()),
            false,
            B::AncestorsOrAncestorSiblingsAffectedByHas(parent),
        )
    }
}

/// Remaining foreign values, registries, cache owners and scope-tree operations.
pub trait StyleEngineValueBackend:
    StyleEngineInvalidationBackend
    + StyleEnginePendingInvalidationBackend
    + StyleEngineCounterBackend
    + StyleEngineCollectionBackend
{
    type CounterStyle;
    type ReferenceFilterOperation;
    type SVGResource;
    type ComputedStyleBuilder;
    type FilterOperations;
    type StyleImageCache;
    type ImageResourceContent;
    type FetchParameters;
    type RandomCacheKey;
    type RandomCachingKey: Eq + std::hash::Hash;
    fn RuleInvalidationUsesFirstLineRules(data: &Self::RuleInvalidationData) -> bool;
    fn RuleInvalidationUsesWindowInactiveSelector(data: &Self::RuleInvalidationData) -> bool;
    fn UpdateDocumentRulesSelectorsRuleSet(
        rules: &Self::CSSGlobalRuleSet,
        document: &Self::Document,
    );
    fn CounterStylesMarkAllDirty(document: &Self::Document, scopes: &[Rc<Self::TreeScope>]);
    fn CounterStylesResolveAllReferences(document: &Self::Document, scopes: &[Rc<Self::TreeScope>]);
    fn LayoutInvalidateCounterStyleChanges(view: &Self::LayoutObject);
    fn LayoutInvalidateSubtreePositionTry(view: &Self::LayoutObject, mark_style_dirty: bool);
    fn ComputedStyleIsNullOrEnsured(style: Option<&Self::ComputedStyle>) -> bool;
    fn SpecifiedFontSizeEqual(a: &Self::ComputedStyle, b: &Self::ComputedStyle) -> bool;
    fn ComputedStyleFontsEquivalent(a: &Self::ComputedStyle, b: &Self::ComputedStyle) -> bool;
    fn LineHeightEqual(a: &Self::ComputedStyle, b: &Self::ComputedStyle) -> bool;
    fn ComputedStyleHasEnv(style: &Self::ComputedStyle) -> bool;
    fn ComputedStyleBottomIsAuto(style: &Self::ComputedStyle) -> bool;
    fn ComputedStyleHasEnvSafeAreaInsetBottom(style: &Self::ComputedStyle) -> bool;
    fn ComputedStyleBottomRelativeToSafeAreaInset(style: &Self::ComputedStyle) -> bool;
    fn ComputedStyleHasCounterDirectives(style: &Self::ComputedStyle) -> bool;
    fn ComputedStyleContainsStyle(style: &Self::ComputedStyle) -> bool;
    fn ComputedStyleScrollTargetGroupNone(style: &Self::ComputedStyle) -> bool;
    fn PseudoElementStylesAffectCounters(element: &Self::Element) -> bool;
    fn ContainmentRemoveScope(tree: &Self::StyleContainmentScopeTree, element: &Self::Element);
    fn ScrollTargetRemoveScope(tree: &Self::ScrollTargetGroupScopeTree, element: &Self::Element);
    fn ContainmentUpdateItems(tree: &Self::StyleContainmentScopeTree);
    fn ScrollTargetUpdateItems(tree: &Self::ScrollTargetGroupScopeTree);
    fn RescheduleSiblingInvalidationsAsDescendants(
        pending: &Self::PendingInvalidations,
        element: &Self::Element,
    );
    fn CollectorCollectMatchingUserRules(
        collector: &mut Self::ElementRuleCollector,
        group: &Self::RuleSetGroup,
    );
    fn NewDocumentCSSParserContext(document: &Rc<Self::Document>) -> Rc<Self::CSSParserContext>;
    fn NewEnvironmentVariables(
        document: &Rc<Self::Document>,
    ) -> Rc<Self::DocumentStyleEnvironmentVariables>;
    fn PropertyRegistryIsEmpty(registry: &Self::PropertyRegistry) -> bool;
    fn NewStyleInitialData(
        document: &Self::Document,
        registry: &Self::PropertyRegistry,
    ) -> Rc<Self::StyleInitialData>;
    fn AuthorCounterStyleMap(scope: &Self::TreeScope) -> Option<Rc<Self::CounterStyleMap>>;
    fn UACounterStyleMap() -> Rc<Self::CounterStyleMap>;
    fn CounterStyleMapFindAcrossScopes(
        map: &Self::CounterStyleMap,
        name: &AtomicString,
    ) -> Option<Rc<Self::CounterStyle>>;
    fn DecimalCounterStyle() -> Rc<Self::CounterStyle>;
    fn ScopedResolverFunctionForName(
        resolver: &Self::ScopedStyleResolver,
        name: &AtomicString,
    ) -> Option<Rc<Self::StyleRuleFunction>>;
    fn CSSValueIsList(value: &Self::CSSValue) -> bool;
    fn ExtractColorSchemes(document: &Self::Document, value: &Self::CSSValue) -> ColorSchemeFlags;
    fn DocumentColorSchemeChanged(document: &Self::Document);
    fn PageVisionDeficiency(document: &Self::Document) -> VisionDeficiency;
    fn URIValueEnsureResourceReference(url: &AtomicString) -> Rc<Self::SVGResource>;
    fn SVGResourceLoadWithoutCSP(resource: &Self::SVGResource, document: &Self::Document);
    fn NewReferenceFilterOperation(
        url: AtomicString,
        resource: Rc<Self::SVGResource>,
    ) -> Rc<Self::ReferenceFilterOperation>;
    fn NewFilterOperations() -> Self::FilterOperations;
    fn FilterOperationsPush(
        ops: &mut Self::FilterOperations,
        operation: Rc<Self::ReferenceFilterOperation>,
    );
    fn BuilderSetFilter(builder: &mut Self::ComputedStyleBuilder, ops: Self::FilterOperations);
    fn NewStyleImageCache() -> Self::StyleImageCache;
    fn ImageCacheImageContent(
        cache: &Self::StyleImageCache,
        document: &Self::Document,
        params: &mut Self::FetchParameters,
    ) -> Rc<Self::ImageResourceContent>;
    fn RandomKeyIsElementScoped(key: &Self::RandomCacheKey) -> bool;
    fn RandomNameForCaching(key: &Self::RandomCacheKey) -> AtomicString;
    fn NewRandomCachingKey(
        key: &Self::RandomCacheKey,
        element: Option<&Rc<Self::Element>>,
    ) -> Rc<Self::RandomCachingKey>;
    fn RandDouble() -> f64;
}

/// Traversal root and DOM/LayoutObject lifecycle operations. State decisions
/// and ordering stay in StyleEngine; scope tokens retain real foreign owners.
pub trait StyleEngineTraversalBackend:
    StyleEngineValueBackend + StyleEngineMutationBackend
{
    type StyleRecalcChange;
    type ScriptForbiddenScope;
    type PseudoHasCacheScope;
    type SelectorFilterParentScope;
    type WhitespaceAttacher;
    fn StyleInvalidationRootNode(root: &Self::StyleInvalidationRoot) -> Option<Rc<Self::Node>>;
    fn StyleRecalcRootNode(root: &Self::StyleRecalcRoot) -> Option<Rc<Self::Node>>;
    fn LayoutTreeRebuildRootNode(root: &Self::LayoutTreeRebuildRoot) -> Option<Rc<Self::Node>>;
    fn StyleRecalcRootElement(root: &Self::StyleRecalcRoot) -> Rc<Self::Element>;
    fn LayoutTreeRebuildRootElement(root: &Self::LayoutTreeRebuildRoot) -> Rc<Self::Element>;
    fn StyleInvalidationRootUpdate(
        root: &mut Self::StyleInvalidationRoot,
        ancestor: Option<&Self::ContainerNode>,
        node: Option<&Self::Node>,
    );
    fn StyleRecalcRootUpdate(
        root: &mut Self::StyleRecalcRoot,
        ancestor: Option<&Self::ContainerNode>,
        node: Option<&Self::Node>,
    );
    fn LayoutTreeRebuildRootUpdate(
        root: &mut Self::LayoutTreeRebuildRoot,
        ancestor: Option<&Self::ContainerNode>,
        node: Option<&Self::Node>,
    );
    fn StyleInvalidationRootSubtreeModified(
        root: &mut Self::StyleInvalidationRoot,
        parent: &Self::ContainerNode,
    );
    fn StyleRecalcRootSubtreeModified(
        root: &mut Self::StyleRecalcRoot,
        parent: &Self::ContainerNode,
    );
    fn StyleRecalcRootFlatTreePositionChanged(root: &mut Self::StyleRecalcRoot, node: &Self::Node);
    fn LayoutTreeRebuildRootSubtreeModified(
        root: &mut Self::LayoutTreeRebuildRoot,
        element: &Self::Element,
    );
    fn DocumentAsNode(document: &Self::Document) -> &Self::Node;
    fn NodeIsDocument(node: &Self::Node) -> bool;
    fn NodeIsShadowRoot(node: &Self::Node) -> bool;
    fn NodeParentOrShadowHost(node: &Self::Node) -> Option<Rc<Self::Node>>;
    fn LayoutTreeBuilderParent(node: &Self::Node) -> Option<Rc<Self::Node>>;
    fn NodeShadowRoot(node: &Self::Node) -> Option<Rc<Self::Node>>;
    fn IsPotentialStyleRecalcRoot(node: &Self::Node) -> bool;
    fn ComputedStyleIsInterleavingRoot(style: Option<&Self::ComputedStyle>) -> bool;
    fn FlatTreeParentElement(element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ElementStyleRecalcParent(element: &Self::Element) -> Option<Rc<Self::ContainerNode>>;
    fn ContainerStyleRecalcParent(node: &Self::ContainerNode) -> Option<Rc<Self::ContainerNode>>;
    fn ContainerClearChildNeedsStyleRecalc(node: &Self::ContainerNode);
    fn ElementClearChildNeedsStyleRecalc(element: &Self::Element);
    fn ElementClearChildNeedsReattachLayoutTree(element: &Self::Element);
    fn ElementRecalcTraversalRootAncestor(element: &Self::Element);
    fn ElementReattachParent(element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ElementIsHTMLBody(element: &Self::Element) -> bool;
    fn ElementIsHTMLHtml(element: &Self::Element) -> bool;
    fn HTMLRootPropagateWritingModeFromBody(element: &Self::Element);
    fn DocumentViewportDefiningElement(document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn DocumentFirstBodyElement(document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn ElementNeedsReattachLayoutTree(element: &Self::Element) -> bool;
    fn ElementSetChildNeedsStyleRecalc(element: &Self::Element);
    fn LayoutIsBlock(object: &Self::LayoutObject) -> bool;
    fn CloneComputedStyle(style: &Self::ComputedStyle) -> Rc<Self::ComputedStyle>;
    fn ResolverPropagateStyleToViewport(resolver: &Self::StyleResolver);
    fn NewStyleRecalcChange() -> Self::StyleRecalcChange;
    fn NewAncestorStyleRecalcContext(element: &Self::Element) -> Self::StyleRecalcContext;
    fn NewScriptForbiddenScope() -> Self::ScriptForbiddenScope;
    fn NewPseudoHasCacheScope(document: &Self::Document) -> Self::PseudoHasCacheScope;
    fn NewSelectorFilterParentScope(
        parent: Option<&Self::Element>,
    ) -> Self::SelectorFilterParentScope;
    fn ElementRecalcStyle(
        element: &Self::Element,
        change: Self::StyleRecalcChange,
        context: &Self::StyleRecalcContext,
    );
    fn NewWhitespaceAttacher() -> Self::WhitespaceAttacher;
    fn ElementRebuildLayoutTree(element: &Self::Element, attacher: &mut Self::WhitespaceAttacher);
    fn ElementRebuildSizeContainerAncestor(element: &Self::Element);
    fn ElementRebuildTraversalRootAncestor(element: &Self::Element);
    fn ElementReattachLayoutTreeChildren(element: &Self::Element);
    fn ElementAsPseudoElement(element: &Self::Element) -> Option<Rc<Self::PseudoElement>>;
    fn PseudoAsElement(element: &Self::PseudoElement) -> &Self::Element;
    fn PseudoUltimateOriginatingElement(element: &Self::PseudoElement) -> Rc<Self::Element>;
}

/// Container query evaluator and anchor/fallback value objects. Each operation
/// belongs to the named external object, rather than a StyleEngine entry point.
pub trait StyleEngineInterleavingBackend: StyleEngineTraversalBackend {
    type ContainerQueryData;
    type ContainerQueryEvaluator;
    type LogicalSize;
    type LogicalAxes;
    type PhysicalSize;
    type PhysicalAxes;
    type WritingDirectionMode;
    type AnchorEvaluator;
    type TryValueFlips;
    type OutOfFlowData;
    fn HighlightDataDependsOnSizeQueries(style: &Self::ComputedStyle) -> bool;
    fn HighlightsDependOnSizeQueries(style: &Self::ComputedStyle) -> bool;
    fn HasAnyHighlightPseudoStyles(style: &Self::ComputedStyle) -> bool;
    fn HasNonUAHighlightStyles(style: &Self::ComputedStyle) -> bool;
    fn ElementParentComputedStyle(element: &Self::Element) -> Option<Rc<Self::ComputedStyle>>;
    fn ElementRecalcHighlightStyles(
        element: &Self::Element,
        context: &Self::StyleRecalcContext,
        style: &Self::ComputedStyle,
        parent: Option<&Self::ComputedStyle>,
    ) -> Rc<Self::ComputedStyle>;
    fn ElementSetComputedStyle(element: &Self::Element, style: Rc<Self::ComputedStyle>);
    fn LayoutSetStyleWithoutApplyingChanges(
        object: &Self::LayoutObject,
        style: Rc<Self::ComputedStyle>,
    );
    fn NewStyleRecalcContext() -> Self::StyleRecalcContext;
    fn ContextSetSizeContainer(context: &mut Self::StyleRecalcContext, container: &Self::Element);
    fn ChangeSuppressRecalc(change: Self::StyleRecalcChange) -> Self::StyleRecalcChange;
    fn ChangeForceDescendantSizeContainers(
        change: Self::StyleRecalcChange,
    ) -> Self::StyleRecalcChange;
    fn ChangeForceSizeContainer(change: Self::StyleRecalcChange) -> Self::StyleRecalcChange;
    fn ChangeForceChildren(change: Self::StyleRecalcChange) -> Self::StyleRecalcChange;
    fn ChangeForceMarkReattach(change: Self::StyleRecalcChange) -> Self::StyleRecalcChange;
    fn ChangeForceReattach(change: Self::StyleRecalcChange) -> Self::StyleRecalcChange;
    fn ElementContainerQueryData(element: &Self::Element) -> Option<Rc<Self::ContainerQueryData>>;
    fn ContainerDataSkippedRecalc(data: &Self::ContainerQueryData) -> bool;
    fn EnsureContainerQueryEvaluator(element: &Self::Element) -> Rc<Self::ContainerQueryEvaluator>;
    fn ElementContainerQueryEvaluator(
        element: &Self::Element,
    ) -> Option<Rc<Self::ContainerQueryEvaluator>>;
    fn SizeContainerChanged(
        evaluator: &Self::ContainerQueryEvaluator,
        size: Self::PhysicalSize,
        axes: Self::PhysicalAxes,
    ) -> ContainerQueryChange;
    fn EmptyPhysicalSize() -> Self::PhysicalSize;
    fn NoPhysicalAxes() -> Self::PhysicalAxes;
    fn LogicalToPhysicalSize(
        size: &Self::LogicalSize,
        style: &Self::ComputedStyle,
    ) -> Self::PhysicalSize;
    fn ClampNegativeSizeToZero(size: &mut Self::PhysicalSize);
    fn AdjustPhysicalSizeForAbsoluteZoom(
        size: Self::PhysicalSize,
        style: &Self::ComputedStyle,
    ) -> Self::PhysicalSize;
    fn LogicalToPhysicalAxes(
        axes: Self::LogicalAxes,
        style: &Self::ComputedStyle,
    ) -> Self::PhysicalAxes;
    fn ClearCachedPseudoElementStyles(style: &Self::ComputedStyle);
    fn HasFirstLinePseudoStyle(style: &Self::ComputedStyle) -> bool;
    fn FirstLineDependsOnSizeQueries(style: &Self::ComputedStyle) -> bool;
    fn PositionAreaDeclarations(
        area: &layoutng_style::style::position_area::PositionArea,
    ) -> Rc<Self::CSSPropertyValueSet>;
    fn FallbackScopedName(fallback: &PositionTryFallback) -> Option<Rc<foundation::ScopedCSSName>>;
    fn ScopedNameTreeScope(name: &foundation::ScopedCSSName) -> Option<Rc<Self::TreeScope>>;
    fn ResolverPositionTryRule(
        resolver: &Self::StyleResolver,
        scope: &Self::TreeScope,
        name: &AtomicString,
    ) -> Option<Rc<Self::StyleRulePositionTry>>;
    fn PositionTryProperties(rule: &Self::StyleRulePositionTry) -> Rc<Self::CSSPropertyValueSet>;
    fn NewTryValueFlips() -> Self::TryValueFlips;
    fn FlipSet(
        flips: &Self::TryValueFlips,
        tactics: &TryTacticList,
        direction: &Self::WritingDirectionMode,
    ) -> Option<Rc<Self::CSSPropertyValueSet>>;
    fn ContextSetAnchorEvaluator(
        context: &mut Self::StyleRecalcContext,
        evaluator: Option<&Self::AnchorEvaluator>,
    );
    fn ContextSetTrySet(
        context: &mut Self::StyleRecalcContext,
        set: Option<Rc<Self::CSSPropertyValueSet>>,
    );
    fn ContextSetTryTacticsSet(
        context: &mut Self::StyleRecalcContext,
        set: Option<Rc<Self::CSSPropertyValueSet>>,
    );
    fn EvaluatorApplyAnchoredChanges(
        evaluator: &Self::ContainerQueryEvaluator,
        change: Self::StyleRecalcChange,
        fallback: &PositionTryFallback,
        direction: &Self::WritingDirectionMode,
    ) -> Self::StyleRecalcChange;
    fn ElementOutOfFlowData(element: &Self::Element) -> Option<Rc<Self::OutOfFlowData>>;
    fn OutOfFlowApplyPendingFallbackAndScrollShift(
        data: &Self::OutOfFlowData,
        layout: Option<&Self::LayoutObject>,
    ) -> bool;
    fn OutOfFlowInvalidatePositionTryNames(
        data: &Self::OutOfFlowData,
        names: &HashSet<AtomicString>,
    ) -> bool;
    fn LayoutInvalidateAnchorPositioning(object: &Self::LayoutObject);
    fn LayoutTreeRootNode(element: &Self::Element) -> Option<Rc<Self::Node>>;
    fn LayoutTreeNext(node: &Self::Node, root: &Self::Element) -> Option<Rc<Self::Node>>;
    fn LayoutTreeNextSkippingChildren(
        element: &Self::Element,
        root: &Self::Element,
    ) -> Option<Rc<Self::Node>>;
}

/// Inspector scopes and foreign rule/selector graph access.
pub trait StyleEngineInspectorBackend:
    StyleEngineInvalidationBackend + StyleEngineFeatureBackend
{
    type StyleRuleBase;
    type InspectorStyleRule;
    type InspectorSelector;
    type InspectorContentsScope;
    type InspectorSelectorScope;
    fn ContentsChildRules(contents: &Self::StyleSheetContents) -> Vec<Rc<Self::StyleRuleBase>>;
    fn InspectorAsStyleRule(rule: &Self::StyleRuleBase) -> Option<Rc<Self::InspectorStyleRule>>;
    fn InspectorGroupChildRules(rule: &Self::StyleRuleBase)
        -> Option<Vec<Rc<Self::StyleRuleBase>>>;
    fn InspectorSelectors(rule: &Self::InspectorStyleRule) -> Vec<Rc<Self::InspectorSelector>>;
    fn NewInspectorContentsScope(
        contents: &Self::StyleSheetContents,
    ) -> Self::InspectorContentsScope;
    fn NewInspectorSelectorScope(
        rule: &Self::InspectorStyleRule,
        selector: &Self::InspectorSelector,
    ) -> Self::InspectorSelectorScope;
    fn FeaturesRevisitSelector(features: &Self::RuleFeatureSet, selector: &Self::InspectorSelector);
}

// cpp: container_query_evaluator.h: Change (foreign evaluator result).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ContainerQueryChange {
    kNone,
    kNearestContainer,
    kDescendantContainers,
}

struct FlagReset<'a>(&'a Cell<bool>, bool);
impl<'a> FlagReset<'a> {
    fn set(flag: &'a Cell<bool>, value: bool) -> Self {
        Self(flag, flag.replace(value))
    }
}
impl Drop for FlagReset<'_> {
    fn drop(&mut self) {
        self.0.set(self.1);
    }
}

pub trait StyleEngineBackend:
    StyleEngineQueryBackend
    + StyleEngineResolverBackend
    + StyleEngineLayoutBackend
    + StyleEngineBootstrapBackend
    + StyleEngineCollectionBackend
    + StyleEngineRuleBackend
    + StyleEngineInvalidationBackend
    + StyleEngineLifecycleBackend
    + StyleEngineFeatureBackend
    + StyleEngineCounterBackend
    + StyleEngineMutationBackend
    + StyleEngineRuleInvalidationBackend
    + StyleEnginePendingInvalidationBackend
    + StyleEngineValueBackend
    + StyleEngineTraversalBackend
    + StyleEngineInterleavingBackend
    + StyleEngineInspectorBackend
{
}

// cpp: style_engine.cc:152-165
const FONT_FACE_RULES: u32 = 1 << 0;
const KEYFRAMES_RULES: u32 = 1 << 1;
const PROPERTY_RULES: u32 = 1 << 2;
const COUNTER_STYLE_RULES: u32 = 1 << 3;
const LAYER_RULES: u32 = 1 << 4;
const FONT_PALETTE_RULES: u32 = 1 << 5;
const POSITION_TRY_RULES: u32 = 1 << 6;
const FONT_FEATURE_RULES: u32 = 1 << 7;
const VIEW_TRANSITION_RULES: u32 = 1 << 8;
const FUNCTION_RULES: u32 = 1 << 9;
const RULE_SET_FLAGS_ALL: u32 = !0;

// cpp: style_engine.cc:2873-2918
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AtRulePriority {
    is_user_style: bool,
    layer_order: u16,
}
impl Ord for AtRulePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_user_style != other.is_user_style {
            other.is_user_style.cmp(&self.is_user_style)
        } else {
            self.layer_order.cmp(&other.layer_order)
        }
    }
}
impl PartialOrd for AtRulePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
struct AtRuleCascadeMap {
    map_: HashMap<AtomicString, AtRulePriority>,
}
impl AtRuleCascadeMap {
    fn new() -> Self {
        Self {
            map_: HashMap::new(),
        }
    }
    fn AddAndCascade(&mut self, name: AtomicString, priority: AtRulePriority) -> bool {
        if self.map_.get(&name).is_some_and(|old| priority < *old) {
            return false;
        }
        self.map_.insert(name, priority);
        true
    }
}
// cpp: cascade_layer_map.h:40-51
// cpp: style_engine.cc:2183-2188
fn FlagsCauseInvalidation(flags: InvalidationMatchFlags) -> bool {
    flags.affected_by_drag
        || flags.affected_by_focus_within
        || flags.affected_by_hover
        || flags.affected_by_active
}
fn CompareLayerOrder(
    map: Option<&CascadeLayerMap>,
    old: Option<&CascadeLayer>,
    new: Option<&CascadeLayer>,
) -> Ordering {
    if old == new {
        Ordering::Equal
    } else {
        map.expect("distinct layers require cascade layer map")
            .CompareLayerOrder(old, new)
    }
}

fn OptionRcPtrEq<T>(a: Option<&Rc<T>>, b: Option<&Rc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn identity<T>(value: &Rc<T>) -> usize {
    Rc::as_ptr(value) as usize
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:132-137
// The StyleEngine class manages style-related state for the document. There is
// a 1-1 relationship of Document to StyleEngine. The document calls the
// StyleEngine when the document is updated in a way that impacts styles.
pub struct StyleEngine<B: StyleEngineBackend> {
    // cpp: style_engine.h:1124,1138-1140,1260,1274,1279,1295-1316
    is_env_dirty_: Cell<bool>,
    has_complex_safe_area_constraints_: Cell<bool>,
    vision_deficiency_: Cell<VisionDeficiency>,
    vision_deficiency_filter_: Retained<B::ReferenceFilterOperation>,
    style_image_cache_: B::StyleImageCache,
    fill_or_clip_path_uri_value_cache_: RefCell<HashMap<AtomicString, Weak<B::CSSValue>>>,
    try_value_flips_: B::TryValueFlips,
    anchored_element_dirty_set_: RefCell<HashMap<usize, Rc<B::Element>>>,
    element_keeps_random_caching_key_alive_:
        RefCell<HashMap<usize, (Option<Weak<B::Element>>, HashSet<Rc<B::RandomCachingKey>>)>>,
    random_base_value_cache_: RefCell<Vec<(Weak<B::RandomCachingKey>, f64)>>,
    element_shared_random_base_value_cache_: RefCell<HashMap<AtomicString, f64>>,

    document_: Rc<B::Document>,
    pending_invalidations_: B::PendingInvalidations,
    // cpp: style_engine.h:1035,1057,1121,1130,1146-1150,1186,1188
    scroll_target_group_scope_tree_: Retained<B::ScrollTargetGroupScopeTree>,
    tracker_: Retained<B::StyleRuleUsageTracker>,
    environment_variables_: Retained<B::DocumentStyleEnvironmentVariables>,
    style_invalidation_root_: RefCell<B::StyleInvalidationRoot>,
    style_recalc_root_: RefCell<B::StyleRecalcRoot>,
    layout_tree_rebuild_root_: RefCell<B::LayoutTreeRebuildRoot>,
    viewport_unit_dirty_flags_: Cell<u32>,
    needs_to_update_complex_safe_area_constraints_: Cell<bool>,

    // Added state declarations are individually accounted in the file ledger.
    style_containment_scope_tree_: Retained<B::StyleContainmentScopeTree>,
    font_selector_: Retained<B::CSSFontSelector>,
    fonts_need_update_: Cell<bool>,
    counter_styles_need_update_: Cell<bool>,
    position_try_styles_dirty_: Cell<bool>,
    dirty_position_try_names_: RefCell<HashSet<AtomicString>>,
    user_rule_set_groups_: RefCell<Vec<B::RuleSetGroup>>,
    keyframes_rule_map_: RefCell<HashMap<AtomicString, CascadeLayered<Rc<B::StyleRuleKeyframes>>>>,
    font_palette_values_rule_map_:
        RefCell<HashMap<(AtomicString, String), Rc<B::StyleRuleFontPaletteValues>>>,
    user_counter_style_map_: RefCell<Option<Rc<B::CounterStyleMap>>>,
    user_cascade_layer_map_: RefCell<Option<Rc<CascadeLayerMap>>>,
    user_function_rule_map_:
        RefCell<HashMap<AtomicString, CascadeLayered<Rc<B::StyleRuleFunction>>>>,
    initial_data_: RefCell<Option<Rc<B::StyleInitialData>>>,
    view_transition_rule_: RefCell<Option<CascadeLayered<Rc<B::StyleRuleViewTransition>>>>,
    view_transition_preview_rule_: RefCell<Option<CascadeLayered<Rc<B::StyleRuleViewTransition>>>>,
    owner_preferred_color_scheme_: Cell<PreferredColorScheme>,
    preferred_contrast_: Cell<PreferredContrast>,
    forced_colors_: Cell<ForcedColors>,
    force_dark_mode_enabled_: Cell<bool>,
    page_color_schemes_: Cell<ColorSchemeFlags>,
    color_scheme_background_: Cell<Color>,
    forced_background_color_: Cell<Color>,
    viewport_size_: RefCell<Option<B::ViewportSize>>,

    document_style_sheet_collection_: Rc<B::StyleSheetCollection>,
    style_sheet_collection_map_:
        RefCell<HashMap<usize, (Weak<B::TreeScope>, Rc<B::StyleSheetCollection>)>>,
    pub sheet_state: RefCell<ActiveSheetState<B>>,
    pending_parser_blocking_stylesheets_: Cell<i32>,
    inspector_style_sheet_list_: RefCell<Vec<Rc<B::CSSStyleSheet>>>,
    injected_author_style_sheets_: RefCell<Vec<(StyleSheetKey, Rc<B::CSSStyleSheet>)>>,
    injected_user_style_sheets_: RefCell<Vec<(StyleSheetKey, Rc<B::CSSStyleSheet>)>>,
    text_tracks_: RefCell<HashMap<usize, Rc<B::TextTrack>>>,
    vtt_originating_element_: RefCell<Option<Rc<B::Element>>>,
    text_to_sheet_cache_: RefCell<HashMap<AtomicString, Weak<B::StyleSheetContents>>>,
    navigation_locations_: RefCell<HashMap<AtomicString, Option<Rc<B::URLPattern>>>>,
    functional_media_query_results_: RefCell<HashMap<usize, (Rc<B::MediaQuerySet>, bool)>>,
    functional_navigation_query_results_:
        RefCell<HashMap<usize, (Rc<B::NavigationTestExpression>, bool)>>,
    functional_media_query_result_flags_: RefCell<MediaQueryResultFlags>,
    media_query_evaluator_: RefCell<Option<Rc<B::MediaQueryEvaluator>>>,
    resolver_: Retained<B::StyleResolver>,
    global_rule_set_: Retained<B::CSSGlobalRuleSet>,
    viewport_resolver_: Retained<B::ViewportStyleResolver>,
    owner_color_scheme_: Cell<ColorScheme>,
    pub preferred_color_scheme: Cell<PreferredColorScheme>,
    parent_for_detached_subtree_: RefCell<Option<Rc<B::LayoutObject>>>,

    // cpp: third_party/blink/renderer/core/css/style_engine.h:1044
    pending_script_blocking_stylesheets_: Cell<i32>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1069
    preferred_stylesheet_set_name_: RefCell<String>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1073
    counters_changed_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1075-1080
    uses_root_relative_units_: Cell<bool>,
    uses_glyph_relative_units_: Cell<bool>,
    uses_line_height_units_: Cell<bool>,
    uses_tree_counting_functions_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1083
    style_affected_by_layout_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1093
    skipped_container_recalc_: Cell<i64>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1094
    in_layout_tree_rebuild_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1095
    in_container_query_style_recalc_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1096
    in_position_try_style_recalc_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1097
    in_scroll_markers_attachment_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1098
    in_dom_removal_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1099
    in_detach_scope_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1100-1102
    in_apply_animation_update_: Cell<bool>,
    in_ensure_computed_style_: Cell<bool>,
    viewport_style_dirty_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1106
    needs_style_update_on_navigation_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1111
    allow_mark_style_dirty_from_recalc_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1115
    allow_mark_for_reattach_from_rebuild_layout_tree_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1118
    allow_skip_style_recalc_: Cell<bool>,
    // cpp: third_party/blink/renderer/core/css/style_engine.h:1158
    style_for_element_count_: Cell<u32>,
}

impl<B: StyleEngineBackend> StyleEngine<B> {
    // cpp: style_engine.h:758-772
    pub fn DetachedFromParent(&self, parent: Option<Rc<B::LayoutObject>>) {
        if self.in_detach_scope_.get() {
            *self.parent_for_detached_subtree_.borrow_mut() = parent;
        }
    }

    // cpp: style_engine.cc:363-375
    pub fn CreateInspectorStyleSheet(&self) -> Rc<B::CSSStyleSheet> {
        let context = B::NewDocumentCSSParserContext(&self.document_);
        let contents = B::NewStyleSheetContents(context);
        let sheet = B::NewCSSStyleSheet(contents, &self.document_);
        self.inspector_style_sheet_list_
            .borrow_mut()
            .push(sheet.clone());
        self.MarkDocumentDirty();
        self.UpdateActiveStyle();
        sheet
    }
    // cpp: style_engine.cc:547-572
    pub fn DocumentRulesSelectorsChanged(&self) {
        let global = self.global_rule_set_.get().expect("global rules");
        let old = B::DocumentRulesSelectorsRuleSet(&global);
        B::UpdateDocumentRulesSelectorsRuleSet(&global, &self.document_);
        let new = B::DocumentRulesSelectorsRuleSet(&global);
        let mut changed = ChangedRuleSets::default();
        if let Some(rules) = old {
            changed.Insert(rules);
        }
        if let Some(rules) = new {
            changed.Insert(rules);
        }
        self.InvalidateForRuleSetChanges(
            &B::DocumentTreeScope(&self.document_),
            &changed,
            Self::GetRuleSetFlags(&changed),
            InvalidationScope::kInvalidateAllScopes,
        );
        self.UpdateActiveStyle();
    }
    // cpp: style_engine.cc:773-784
    pub fn UpdateCounterStyles(&self) {
        if !self.counter_styles_need_update_.get() {
            return;
        }
        let scopes = self.GetActiveTreeScopes();
        B::CounterStylesMarkAllDirty(&self.document_, &scopes);
        B::CounterStylesResolveAllReferences(&self.document_, &scopes);
        if let Some(view) = B::LayoutViewIfPresent(&self.document_) {
            B::LayoutInvalidateCounterStyleChanges(&view);
        }
        self.counter_styles_need_update_.set(false);
    }
    // cpp: style_engine.cc:808-815
    pub fn InvalidatePositionTryStyles(&self) {
        if !self.position_try_styles_dirty_.replace(false) {
            return;
        }
        B::LayoutInvalidateSubtreePositionTry(&B::DocumentLayoutView(&self.document_), true);
    }
    // cpp: style_engine.cc:3238-3260
    fn LoadVisionDeficiencyFilter(&self) {
        let old = self.vision_deficiency_.get();
        let value = B::PageVisionDeficiency(&self.document_);
        self.vision_deficiency_.set(value);
        if value == old {
            return;
        }
        if value == VisionDeficiency::kNoVisionDeficiency {
            self.vision_deficiency_filter_.set(None);
        } else {
            let url = CreateVisionDeficiencyFilterUrl(value);
            let resource = B::URIValueEnsureResourceReference(&url);
            B::SVGResourceLoadWithoutCSP(&resource, &self.document_);
            self.vision_deficiency_filter_
                .set(Some(B::NewReferenceFilterOperation(url, resource)));
        }
    }
    // cpp: style_engine.cc:3262-3264
    pub fn VisionDeficiencyChanged(&self) {
        self.MarkViewportStyleDirty();
    }
    // cpp: style_engine.cc:3266-3274
    pub fn ApplyVisionDeficiencyStyle(&self, builder: &mut B::ComputedStyleBuilder) {
        self.LoadVisionDeficiencyFilter();
        if let Some(filter) = self.vision_deficiency_filter_.get() {
            let mut ops = B::NewFilterOperations();
            B::FilterOperationsPush(&mut ops, filter);
            B::BuilderSetFilter(builder, ops);
        }
    }
    // cpp: style_engine.cc:3354-3361
    pub fn StyleMaybeAffectedByLayout(&self, element: &B::Element) -> bool {
        self.StyleAffectedByLayout()
            || B::ComputedStyleIsNullOrEnsured(B::ElementComputedStyle(element).as_deref())
    }
    // cpp: style_engine.cc:3363-3389
    pub fn UpdateRootRelativeUnits(
        &self,
        old: Option<&B::ComputedStyle>,
        new: Option<&B::ComputedStyle>,
    ) -> bool {
        let Some(new) = new else {
            return false;
        };
        if !self.UsesRootRelativeUnits() {
            return false;
        }
        let rem_changed = old.is_none_or(|old| !B::SpecifiedFontSizeEqual(old, new));
        let glyphs_changed = old.is_none_or(|old| {
            self.UsesGlyphRelativeUnits() && !B::ComputedStyleFontsEquivalent(old, new)
        });
        let line_changed =
            old.is_none_or(|old| self.UsesLineHeightUnits() && !B::LineHeightEqual(old, new));
        if rem_changed || glyphs_changed || line_changed {
            B::ResolverInvalidateMatchedPropertiesCache(&self.GetStyleResolver());
            return true;
        }
        false
    }
    // cpp: style_engine.cc:3401-3407
    pub fn EnvironmentVariableChanged(&self) {
        self.is_env_dirty_.set(true);
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverInvalidateMatchedPropertiesCache(&resolver);
        }
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:3409-3419
    pub fn InvalidateEnvDependentStylesIfNeeded(&self) {
        if !self.is_env_dirty_.replace(false) {
            return;
        }
        self.MarkElementsForRecalc(
            &B::DocumentTreeScope(&self.document_),
            StyleChangeReason::EnvironmentVariableChanged,
            &B::ComputedStyleHasEnv,
        );
    }
    // cpp: style_engine.cc:225-247
    fn ElementHasComplexSafeAreaConstraint(element: &B::Element, bottom_parent: bool) -> bool {
        let Some(style) = B::ElementComputedStyle(element) else {
            return false;
        };
        if B::ComputedStyleIsNullOrEnsured(Some(&style)) {
            return false;
        }
        let bottom = !B::ComputedStyleBottomIsAuto(&style);
        if B::ComputedStyleHasEnvSafeAreaInsetBottom(&style)
            && (bottom || bottom_parent)
            && !B::ComputedStyleBottomRelativeToSafeAreaInset(&style)
        {
            return true;
        }
        for child in B::LayoutTreeBuilderElementChildren(element) {
            if Self::ElementHasComplexSafeAreaConstraint(&child, bottom) {
                return true;
            }
        }
        false
    }
    // cpp: style_engine.cc:3421-3431
    pub fn HasComplexSafeAreaConstraints(&self) -> bool {
        if self.needs_to_update_complex_safe_area_constraints_.get() {
            let value = B::DocumentElement(&self.document_)
                .is_some_and(|root| Self::ElementHasComplexSafeAreaConstraint(&root, false));
            self.has_complex_safe_area_constraints_.set(value);
            if !value {
                self.needs_to_update_complex_safe_area_constraints_
                    .set(false);
            }
        }
        self.has_complex_safe_area_constraints_.get()
    }
    // cpp: style_engine.cc:3433-3455
    pub fn NodeWillBeRemoved(&self, node: &B::Node) {
        if let Some(element) = B::NodeAsElement(node) {
            if let Some(style) = B::ElementComputedStyle(&element) {
                if B::ComputedStyleHasCounterDirectives(&style)
                    || B::ComputedStyleContainsStyle(&style)
                    || B::PseudoElementStylesAffectCounters(&element)
                {
                    self.MarkCountersDirty();
                }
                if B::ComputedStyleContainsStyle(&style) {
                    if let Some(tree) = self.GetStyleContainmentScopeTree() {
                        B::ContainmentRemoveScope(&tree, &element);
                    }
                }
                if !B::ComputedStyleScrollTargetGroupNone(&style) {
                    if let Some(tree) = self.GetScrollTargetGroupScopeTree() {
                        B::ScrollTargetRemoveScope(&tree, &element);
                    }
                }
            }
            B::RescheduleSiblingInvalidationsAsDescendants(&self.pending_invalidations_, &element);
        }
    }
    // cpp: style_engine.cc:3457-3485
    pub fn ChildrenRemoved(&self, parent: &B::ContainerNode) {
        if !B::NodeIsConnected(B::ContainerNodeAsNode(parent)) {
            return;
        }
        if self.InDOMRemoval() {
            if self.NeedsStyleInvalidation() {
                self.UpdateStyleInvalidationRoot(None, None);
            }
            if self.NeedsStyleRecalc() {
                self.UpdateStyleRecalcRoot(None, None);
            }
            return;
        }
        B::StyleInvalidationRootSubtreeModified(
            &mut self.style_invalidation_root_.borrow_mut(),
            parent,
        );
        B::StyleRecalcRootSubtreeModified(&mut self.style_recalc_root_.borrow_mut(), parent);
    }
    // cpp: style_engine.cc:3487-3493
    pub fn CollectMatchingUserRules(&self, collector: &mut B::ElementRuleCollector) {
        for group in self.user_rule_set_groups_.borrow().iter() {
            B::CollectorCollectMatchingUserRules(collector, group);
        }
    }
    // cpp: style_engine.cc:3645-3658
    pub fn KeyframeStylesForAnimation(
        &self,
        name: &AtomicString,
    ) -> Option<Rc<B::StyleRuleKeyframes>> {
        self.keyframes_rule_map_
            .borrow()
            .get(name)
            .map(|rule| rule.value.clone())
    }
    // cpp: style_engine.cc:3660-3674
    pub fn FontPaletteValuesForNameAndFamily(
        &self,
        palette: AtomicString,
        family: AtomicString,
    ) -> Option<Rc<B::StyleRuleFontPaletteValues>> {
        if palette.empty() {
            return None;
        }
        self.font_palette_values_rule_map_
            .borrow()
            .get(&(
                palette,
                B::FoldCase(&String::from_utf16(
                    family.utf16_units().unwrap_or_default(),
                )),
            ))
            .cloned()
    }
    // cpp: style_engine.cc:3676-3683
    pub fn EnsureEnvironmentVariables(&self) -> Rc<B::DocumentStyleEnvironmentVariables> {
        if let Some(vars) = self.environment_variables_.get() {
            return vars;
        }
        let vars = B::NewEnvironmentVariables(&self.document_);
        self.environment_variables_.set(Some(vars.clone()));
        vars
    }
    // cpp: style_engine.cc:3685-3695
    pub fn MaybeCreateAndGetInitialData(&self) -> Option<Rc<B::StyleInitialData>> {
        if self.initial_data_.borrow().is_none() {
            if let Some(registry) = B::DocumentPropertyRegistry(&self.document_) {
                if !B::PropertyRegistryIsEmpty(&registry) {
                    let data = B::NewStyleInitialData(&self.document_, &registry);
                    *self.initial_data_.borrow_mut() = Some(data);
                }
            }
        }
        self.initial_data_.borrow().clone()
    }
    // cpp: style_engine.cc:3697-3726
    pub fn RecalcHighlightStylesForSizeContainer(&self, container: &B::Element) -> bool {
        let style = B::ElementComputedStyle(container).expect("container style");
        let depends = B::HighlightDataDependsOnSizeQueries(&style)
            || B::HighlightsDependOnSizeQueries(&style);
        if !B::HasAnyHighlightPseudoStyles(&style)
            || !B::HasNonUAHighlightStyles(&style)
            || !depends
        {
            return false;
        }
        let mut context = B::NewStyleRecalcContext();
        B::ContextSetSizeContainer(&mut context, container);
        let new = B::ElementRecalcHighlightStyles(
            container,
            &context,
            &style,
            B::ElementParentComputedStyle(container).as_deref(),
        );
        if !Rc::ptr_eq(&style, &new) {
            B::ElementSetComputedStyle(container, new.clone());
            if let Some(object) = B::ElementLayoutObject(container) {
                B::LayoutSetStyleWithoutApplyingChanges(&object, new);
            }
        }
        depends
    }
    // cpp: style_engine.cc:3775-3810
    pub fn RecalcStyleForSizeContainer(
        &self,
        container: &B::Element,
        change: B::StyleRecalcChange,
    ) {
        let mut change = B::ChangeSuppressRecalc(change);
        B::ElementSetChildNeedsStyleRecalc(container);
        B::StyleRecalcRootUpdate(
            &mut self.style_recalc_root_.borrow_mut(),
            None,
            Some(B::ElementAsNode(container)),
        );
        if self.RecalcHighlightStylesForSizeContainer(container) {
            change = B::ChangeForceDescendantSizeContainers(change);
        }
        self.RecalcStyleWithContext(change, &B::NewAncestorStyleRecalcContext(container));
    }
    // cpp: style_engine.cc:3812-3850
    pub fn UpdateStyleForNonEligibleSizeContainer(&self, container: &B::Element) {
        if B::ElementContainerQueryData(container).is_none() {
            return;
        }
        let mut change = B::NewStyleRecalcChange();
        let result = B::SizeContainerChanged(
            &B::EnsureContainerQueryEvaluator(container),
            B::EmptyPhysicalSize(),
            B::NoPhysicalAxes(),
        );
        match result {
            ContainerQueryChange::kNone => (),
            ContainerQueryChange::kNearestContainer => change = B::ChangeForceSizeContainer(change),
            ContainerQueryChange::kDescendantContainers => {
                change = B::ChangeForceDescendantSizeContainers(change)
            }
        }
        if result != ContainerQueryChange::kNone {
            B::ClearCachedPseudoElementStyles(
                &B::ElementComputedStyle(container).expect("container style"),
            );
        }
        let _allow = AllowMarkForReattachFromRebuildLayoutTreeScope::new(self);
        let _cq = FlagReset::set(&self.in_container_query_style_recalc_, true);
        self.RecalcStyleForSizeContainer(container, change);
    }
    // cpp: style_engine.cc:3852-3863
    pub fn PostInterleavedRecalcUpdate(&self, _root: &B::Element) {
        if let Some(tree) = self.GetStyleContainmentScopeTree() {
            B::ContainmentUpdateItems(&tree);
        }
        if let Some(tree) = self.GetScrollTargetGroupScopeTree() {
            B::ScrollTargetUpdateItems(&tree);
        }
        B::InvalidatePendingSVGResources(&self.document_);
    }
    // cpp: style_engine.cc:3865-3973
    pub fn UpdateStyleAndLayoutTreeForSizeContainer(
        &self,
        container: &B::Element,
        logical_size: &B::LogicalSize,
        axes: B::LogicalAxes,
    ) {
        let cq = FlagReset::set(&self.in_container_query_style_recalc_, true);
        let style =
            B::LayoutObjectStyle(&B::ElementLayoutObject(container).expect("container layout"));
        let mut size = B::LogicalToPhysicalSize(logical_size, &style);
        B::ClampNegativeSizeToZero(&mut size);
        let size = B::AdjustPhysicalSizeForAbsoluteZoom(size, &style);
        let physical_axes = B::LogicalToPhysicalAxes(axes, &style);
        let mut change = B::NewStyleRecalcChange();
        let result = B::SizeContainerChanged(
            &B::EnsureContainerQueryEvaluator(container),
            size,
            physical_axes,
        );
        let data = B::ElementContainerQueryData(container).expect("container query data");
        match result {
            ContainerQueryChange::kNone => {
                if !B::ContainerDataSkippedRecalc(&data) {
                    return;
                }
            }
            ContainerQueryChange::kNearestContainer => change = B::ChangeForceSizeContainer(change),
            ContainerQueryChange::kDescendantContainers => {
                change = B::ChangeForceDescendantSizeContainers(change)
            }
        }
        if result != ContainerQueryChange::kNone {
            B::ClearCachedPseudoElementStyles(&style);
            if B::HasFirstLinePseudoStyle(&style) && B::FirstLineDependsOnSizeQueries(&style) {
                change = B::ChangeForceReattach(B::ChangeForceMarkReattach(change));
            }
        }
        let _nth = B::NewNthIndexCache(&self.document_);
        self.UpdateViewportSize();
        self.RecalcStyleForSizeContainer(container, change);
        if B::ElementNeedsReattachLayoutTree(container) {
            self.ReattachContainerSubtree(container);
        } else if self.NeedsLayoutTreeRebuild() {
            self.RestrictRebuildRoot(container);
            self.RebuildLayoutTree(Some(container));
        }
        if B::DocumentElement(&self.document_)
            .is_some_and(|root| std::ptr::eq(root.as_ref(), container))
        {
            B::ResolverPropagateStyleToViewport(&self.GetStyleResolver());
        }
        drop(cq);
        self.PostInterleavedRecalcUpdate(container);
    }
    // cpp: style_engine.cc:3975-3992
    pub fn TrySetFromFallback(
        &self,
        fallback: &PositionTryFallback,
    ) -> Option<Option<Rc<B::CSSPropertyValueSet>>> {
        if !fallback.GetPositionArea().IsNone() {
            return Some(Some(B::PositionAreaDeclarations(
                fallback.GetPositionArea(),
            )));
        }
        if let Some(name) = B::FallbackScopedName(fallback) {
            return self
                .GetPositionTryRule(&name)
                .map(|rule| Some(B::PositionTryProperties(&rule)));
        }
        Some(None)
    }
    // cpp: style_engine.cc:3994-4061
    pub fn UpdateStyleAndLayoutTreeForOutOfFlow(
        &self,
        element: &B::Element,
        fallback: Option<&PositionTryFallback>,
        anchor: Option<&B::AnchorEvaluator>,
        direction: &B::WritingDirectionMode,
    ) -> bool {
        let mut try_set = None;
        let mut tactics = kNoTryTactics;
        let mut fallback_value = PositionTryFallback::default();
        if let Some(fallback) = fallback {
            tactics = *fallback.GetTryTactic();
            let Some(set) = self.TrySetFromFallback(fallback) else {
                return false;
            };
            try_set = set;
            fallback_value = fallback.clone();
        }
        let tactics_set = B::FlipSet(&self.try_value_flips_, &tactics, direction);
        let pt = FlagReset::set(&self.in_position_try_style_recalc_, true);
        let _nth = B::NewNthIndexCache(&self.document_);
        self.UpdateViewportSize();
        let mut context = B::NewAncestorStyleRecalcContext(element);
        B::ContextSetAnchorEvaluator(&mut context, anchor);
        B::ContextSetTrySet(&mut context, try_set);
        B::ContextSetTryTacticsSet(&mut context, tactics_set);
        let mut change = B::ChangeForceChildren(B::NewStyleRecalcChange());
        if let Some(evaluator) = B::ElementContainerQueryEvaluator(element) {
            change =
                B::EvaluatorApplyAnchoredChanges(&evaluator, change, &fallback_value, direction);
        }
        if let Some(pseudo) = B::ElementAsPseudoElement(element) {
            self.RecalcPositionTryStyleForPseudoElement(&pseudo, change, &context);
        } else {
            B::ElementSetChildNeedsStyleRecalc(element);
            B::StyleRecalcRootUpdate(
                &mut self.style_recalc_root_.borrow_mut(),
                None,
                Some(B::ElementAsNode(element)),
            );
            self.RecalcStyleWithContext(change, &context);
        }
        if self.NeedsLayoutTreeRebuild() {
            self.RestrictRebuildRoot(element);
            self.RebuildLayoutTree(Some(element));
        }
        drop(pt);
        self.PostInterleavedRecalcUpdate(element);
        true
    }
    fn RestrictRebuildRoot(&self, element: &B::Element) {
        let root = B::LayoutTreeRebuildRootNode(&self.layout_tree_rebuild_root_.borrow())
            .expect("rebuild root");
        if B::NodeIsDocument(&root) {
            B::ClearLayoutTreeRebuildRoot(&mut self.layout_tree_rebuild_root_.borrow_mut());
            B::LayoutTreeRebuildRootUpdate(
                &mut self.layout_tree_rebuild_root_.borrow_mut(),
                None,
                Some(B::ElementAsNode(element)),
            );
        }
    }
    // cpp: style_engine.cc:4063-4071
    pub fn GetPositionTryRule(
        &self,
        name: &foundation::ScopedCSSName,
    ) -> Option<Rc<B::StyleRulePositionTry>> {
        let scope =
            B::ScopedNameTreeScope(name).unwrap_or_else(|| B::DocumentTreeScope(&self.document_));
        B::ResolverPositionTryRule(&self.GetStyleResolver(), &scope, name.GetName())
    }
    // cpp: style_engine.cc:4073-4100
    pub fn RecalcStyleWithContext(
        &self,
        change: B::StyleRecalcChange,
        context: &B::StyleRecalcContext,
    ) {
        let _script = B::NewScriptForbiddenScope();
        let _skip = SkipStyleRecalcScope::new(self);
        let _has = B::NewPseudoHasCacheScope(&self.document_);
        let root = B::StyleRecalcRootElement(&self.style_recalc_root_.borrow());
        let parent = B::FlatTreeParentElement(&root);
        let _filter = B::NewSelectorFilterParentScope(parent.as_deref());
        B::ElementRecalcStyle(&root, change, context);
        let mut ancestor = B::ElementStyleRecalcParent(&root);
        while let Some(current) = ancestor {
            if !self.InInterleavedStyleRecalc() {
                if let Some(element) = B::ContainerAsElement(&current) {
                    B::ElementRecalcTraversalRootAncestor(&element);
                }
            }
            B::ContainerClearChildNeedsStyleRecalc(&current);
            ancestor = B::ContainerStyleRecalcParent(&current);
        }
        B::ClearStyleRecalcRoot(&mut self.style_recalc_root_.borrow_mut());
        if parent.is_none() || B::ElementIsHTMLBody(&root) {
            self.PropagateWritingModeAndDirectionToHTMLRoot();
        }
    }
    // cpp: style_engine.cc:4102-4115
    pub fn RecalcPositionTryStyleForPseudoElement(
        &self,
        pseudo: &B::PseudoElement,
        change: B::StyleRecalcChange,
        context: &B::StyleRecalcContext,
    ) {
        let _script = B::NewScriptForbiddenScope();
        let _skip = SkipStyleRecalcScope::new(self);
        let _has = B::NewPseudoHasCacheScope(&self.document_);
        let originating = B::PseudoUltimateOriginatingElement(pseudo);
        let parent = B::FlatTreeParentElement(&originating);
        let _filter = B::NewSelectorFilterParentScope(parent.as_deref());
        B::ElementRecalcStyle(B::PseudoAsElement(pseudo), change, context);
    }
    // cpp: style_engine.cc:4136-4140
    pub fn RecalcStyle(&self) {
        let root = B::StyleRecalcRootElement(&self.style_recalc_root_.borrow());
        self.RecalcStyleWithContext(
            B::NewStyleRecalcChange(),
            &B::NewAncestorStyleRecalcContext(&root),
        );
    }
    // cpp: style_engine.cc:4122-4140
    fn RebuildLayoutTreeForTraversalRootAncestors(
        &self,
        mut parent: Option<Rc<B::Element>>,
        container_parent: Option<&B::Element>,
    ) {
        let mut is_container_ancestor = false;
        while let Some(ancestor) = parent {
            if container_parent.is_some_and(|container| std::ptr::eq(ancestor.as_ref(), container))
            {
                is_container_ancestor = true;
            }
            if is_container_ancestor {
                B::ElementRebuildSizeContainerAncestor(&ancestor);
            } else {
                B::ElementRebuildTraversalRootAncestor(&ancestor);
            }
            B::ElementClearChildNeedsStyleRecalc(&ancestor);
            B::ElementClearChildNeedsReattachLayoutTree(&ancestor);
            parent = B::ElementReattachParent(&ancestor);
        }
    }
    // cpp: style_engine.cc:4142-4174
    pub fn RebuildLayoutTree(&self, container: Option<&B::Element>) {
        let propagate;
        {
            let _rebuild = FlagReset::set(&self.in_layout_tree_rebuild_, true);
            let _filter = B::NewSelectorFilterParentScope(None);
            let root = B::LayoutTreeRebuildRootElement(&self.layout_tree_rebuild_root_.borrow());
            {
                let mut attacher = B::NewWhitespaceAttacher();
                B::ElementRebuildLayoutTree(&root, &mut attacher);
            }
            let container_parent = container.and_then(B::ElementReattachParent);
            self.RebuildLayoutTreeForTraversalRootAncestors(
                B::ElementReattachParent(&root),
                container_parent.as_deref(),
            );
            B::ClearLayoutTreeRebuildRoot(&mut self.layout_tree_rebuild_root_.borrow_mut());
            propagate = B::ElementIsHTMLHtml(&root) || B::ElementIsHTMLBody(&root);
        }
        if propagate {
            self.PropagateWritingModeAndDirectionToHTMLRoot();
            if self.NeedsLayoutTreeRebuild() {
                self.RebuildLayoutTree(container);
            }
        }
    }
    // cpp: style_engine.cc:4176-4193
    fn ReattachContainerSubtree(&self, container: &B::Element) {
        let _rebuild = FlagReset::set(&self.in_layout_tree_rebuild_, true);
        B::ElementReattachLayoutTreeChildren(container);
        let parent = B::ElementReattachParent(container);
        // C++ starts at the container itself, before following its reattach parent.
        B::ElementRebuildTraversalRootAncestor(container);
        B::ElementClearChildNeedsStyleRecalc(container);
        B::ElementClearChildNeedsReattachLayoutTree(container);
        self.RebuildLayoutTreeForTraversalRootAncestors(
            B::ElementReattachParent(container),
            parent.as_deref(),
        );
        B::ClearLayoutTreeRebuildRoot(&mut self.layout_tree_rebuild_root_.borrow_mut());
    }
    // cpp: style_engine.cc:4195-4238
    pub fn UpdateStyleAndLayoutTree(&self) {
        self.UpdateViewportStyle();
        if B::DocumentElement(&self.document_).is_some() {
            self.UpdateViewportSize();
            let _nth = B::NewNthIndexCache(&self.document_);
            if self.NeedsStyleRecalc() {
                let old = B::DocumentViewportDefiningElement(&self.document_);
                self.RecalcStyle();
                let new = B::DocumentViewportDefiningElement(&self.document_);
                if !OptionRcPtrEq(old.as_ref(), new.as_ref()) {
                    self.ViewportDefiningElementDidChange();
                }
            }
            if self.NeedsLayoutTreeRebuild() {
                self.RebuildLayoutTree(None);
            }
            if let Some(tree) = self.GetStyleContainmentScopeTree() {
                B::ContainmentUpdateItems(&tree);
            }
            if let Some(tree) = self.GetScrollTargetGroupScopeTree() {
                B::ScrollTargetUpdateItems(&tree);
            }
            self.UpdateCounters();
        } else {
            B::ClearStyleRecalcRoot(&mut self.style_recalc_root_.borrow_mut());
        }
        self.UpdateColorSchemeBackground(false);
        B::ResolverPropagateStyleToViewport(&self.GetStyleResolver());
    }
    // cpp: style_engine.cc:4240-4268
    pub fn ViewportDefiningElementDidChange(&self) {
        let root = B::DocumentElement(&self.document_).expect("document element");
        if B::ElementNeedsReattachLayoutTree(&root) {
            return;
        }
        let Some(body) = B::DocumentFirstBodyElement(&self.document_) else {
            return;
        };
        if B::ElementNeedsReattachLayoutTree(&body) {
            return;
        }
        if let Some(object) = B::ElementLayoutObject(&body) {
            if B::LayoutIsBlock(&object) {
                B::LayoutObjectSetStyle(
                    &object,
                    B::CloneComputedStyle(&B::LayoutObjectStyle(&object)),
                );
            }
        }
    }
    // cpp: style_engine.cc:4270-4294
    pub fn FirstBodyElementChanged(&self, body: Option<&B::Element>) {
        let root = B::DocumentElement(&self.document_);
        let dirty = body.or(root.as_deref()).expect("dirty root");
        if let Some(body) = body {
            if B::ElementLayoutObject(body).is_none_or(|object| !B::LayoutIsBlock(&object)) {
                return;
            }
        }
        B::ElementSetNeedsStyleRecalc(
            dirty,
            StyleChangeType::kLocalStyleChange,
            StyleChangeReason::ViewportDefiningElement,
        );
    }
    // cpp: style_engine.cc:4296-4305
    pub fn UpdateStyleInvalidationRoot(
        &self,
        ancestor: Option<&B::ContainerNode>,
        dirty: Option<&B::Node>,
    ) {
        if !B::DocumentIsActive(&self.document_) {
            return;
        }
        let (ancestor, dirty) = if self.InDOMRemoval() {
            (None, Some(B::DocumentAsNode(&self.document_)))
        } else {
            (ancestor, dirty)
        };
        B::StyleInvalidationRootUpdate(
            &mut self.style_invalidation_root_.borrow_mut(),
            ancestor,
            dirty,
        );
    }
    // cpp: style_engine.cc:4307-4331
    pub fn UpdateStyleRecalcRoot(
        &self,
        ancestor: Option<&B::ContainerNode>,
        dirty: Option<&B::Node>,
    ) {
        if !B::DocumentIsActive(&self.document_) || B::DocumentInStyleRecalc(&self.document_) {
            return;
        }
        let (ancestor, dirty) = if self.InDOMRemoval() {
            (None, Some(B::DocumentAsNode(&self.document_)))
        } else {
            (ancestor, dirty)
        };
        B::StyleRecalcRootUpdate(&mut self.style_recalc_root_.borrow_mut(), ancestor, dirty);
    }
    // cpp: style_engine.cc:4333-4349
    pub fn UpdateLayoutTreeRebuildRoot(
        &self,
        ancestor: Option<&B::ContainerNode>,
        dirty: Option<&B::Node>,
    ) {
        if !B::DocumentIsActive(&self.document_) || self.InRebuildLayoutTree() {
            return;
        }
        B::LayoutTreeRebuildRootUpdate(
            &mut self.layout_tree_rebuild_root_.borrow_mut(),
            ancestor,
            dirty,
        );
    }
    // cpp: style_engine.cc:4353-4356
    fn AnalysisParent(node: &B::Node) -> Option<Rc<B::Node>> {
        if B::NodeIsShadowRoot(node) {
            B::NodeParentOrShadowHost(node)
        } else {
            B::LayoutTreeBuilderParent(node)
        }
    }
    // cpp: style_engine.cc:4358-4369
    fn IsRootOrSibling(root: Option<&B::Node>, node: &B::Node) -> bool {
        let Some(root) = root else {
            return false;
        };
        if std::ptr::eq(root, node) {
            return true;
        }
        Self::AnalysisParent(root).is_some_and(|parent| {
            Self::AnalysisParent(node).is_some_and(|other| Rc::ptr_eq(&parent, &other))
        })
    }
    // cpp: style_engine.cc:4373-4387
    fn AnalyzeInclusiveAncestor(&self, node: &B::Node) -> AncestorAnalysis {
        if Self::IsRootOrSibling(
            B::StyleRecalcRootNode(&self.style_recalc_root_.borrow()).as_deref(),
            node,
        ) || Self::IsRootOrSibling(
            B::StyleInvalidationRootNode(&self.style_invalidation_root_.borrow()).as_deref(),
            node,
        ) {
            return AncestorAnalysis::kStyleRoot;
        }
        if let Some(element) = B::NodeAsElement(node) {
            if B::ComputedStyleIsInterleavingRoot(B::ElementComputedStyle(&element).as_deref()) {
                return AncestorAnalysis::kInterleavingRoot;
            }
        }
        AncestorAnalysis::kNone
    }
    // cpp: style_engine.cc:4389-4395
    fn AnalyzeExclusiveAncestor(&self, node: &B::Node) -> AncestorAnalysis {
        if B::IsPotentialStyleRecalcRoot(node) {
            AncestorAnalysis::kStyleRoot
        } else {
            self.AnalyzeInclusiveAncestor(node)
        }
    }
    // cpp: style_engine.cc:4397-4417
    pub fn AnalyzeAncestors(&self, node: &B::Node) -> AncestorAnalysis {
        let mut analysis = self.AnalyzeInclusiveAncestor(node);
        let mut ancestor = B::LayoutTreeBuilderParent(node);
        while let Some(current) = ancestor {
            if analysis == AncestorAnalysis::kStyleRoot {
                return analysis;
            }
            if let Some(root) = B::NodeShadowRoot(&current) {
                analysis = analysis.max(self.AnalyzeExclusiveAncestor(&root));
            }
            analysis = analysis.max(self.AnalyzeExclusiveAncestor(&current));
            ancestor = B::LayoutTreeBuilderParent(&current);
        }
        analysis
    }
    // cpp: style_engine.cc:4424-4429
    pub fn MarkStyleDirtyAllowed(&self) -> bool {
        if B::DocumentInStyleRecalc(&self.document_) || self.InInterleavedStyleRecalc() {
            self.allow_mark_style_dirty_from_recalc_.get()
        } else {
            !self.InRebuildLayoutTree()
        }
    }
    // cpp: style_engine.cc:4556-4587
    pub fn SetPageColorSchemes(&self, value: Option<&B::CSSValue>) {
        if !B::DocumentIsActive(&self.document_) {
            return;
        }
        let old = self.page_color_schemes_.get();
        let new = value
            .filter(|v| B::CSSValueIsList(v))
            .map_or(ColorSchemeFlag::kNormal as u8, |v| {
                B::ExtractColorSchemes(&self.document_, v)
            });
        self.page_color_schemes_.set(new);
        if old != new {
            if let Some(resolver) = self.resolver_.get() {
                B::ResolverInvalidateMatchedPropertiesCache(&resolver);
            }
        }
        self.MarkAllElementsForStyleRecalc(StyleChangeReason::PlatformColorChange);
        self.UpdateColorScheme();
        self.UpdateColorSchemeBackground(false);
    }
    // cpp: style_engine.cc:4645-4657
    pub fn SetOwnerColorScheme(&self, color: ColorScheme, preferred: PreferredColorScheme) {
        if self.owner_preferred_color_scheme_.get() != preferred {
            self.owner_preferred_color_scheme_.set(preferred);
            B::DocumentColorSchemeChanged(&self.document_);
        }
        if self.owner_color_scheme_.get() != color {
            self.owner_color_scheme_.set(color);
            self.UpdateColorSchemeBackground(true);
        }
    }
    // cpp: style_engine.cc:4707-4712
    pub fn ColorAdjustBackgroundColor(&self) -> Color {
        if self.forced_colors_.get() != ForcedColors::kNone {
            self.ForcedBackgroundColor()
        } else {
            self.color_scheme_background_.get()
        }
    }
    // cpp: style_engine.cc:4745-4748
    pub fn NeedsFullStyleUpdate(&self) -> bool {
        self.NeedsActiveStyleUpdate()
            || self.IsViewportStyleDirty()
            || self.viewport_unit_dirty_flags_.get() != 0
            || self.is_env_dirty_.get()
    }
    // cpp: style_engine.cc:4750-4755
    pub fn PropagateWritingModeAndDirectionToHTMLRoot(&self) {
        if let Some(root) = B::DocumentElement(&self.document_) {
            if B::ElementIsHTMLHtml(&root) {
                B::HTMLRootPropagateWritingModeFromBody(&root);
            }
        }
    }
    // cpp: style_engine.cc:4765-4787
    pub fn FindCounterStyleAcrossScopes(
        &self,
        name: &AtomicString,
        mut scope: Option<Rc<B::TreeScope>>,
    ) -> Rc<B::CounterStyle> {
        let mut target = None;
        while let Some(current) = scope {
            if let Some(map) = B::AuthorCounterStyleMap(&current) {
                target = Some(map);
                break;
            }
            scope = B::ParentTreeScope(&current);
        }
        let target = target
            .or_else(|| self.user_counter_style_map_.borrow().clone())
            .unwrap_or_else(B::UACounterStyleMap);
        B::CounterStyleMapFindAcrossScopes(&target, name).unwrap_or_else(B::DecimalCounterStyle)
    }
    // cpp: style_engine.cc:4790-4806
    pub fn FindFunctionAcrossScopes(
        &self,
        name: &AtomicString,
        mut scope: Option<Rc<B::TreeScope>>,
    ) -> (Option<Rc<B::StyleRuleFunction>>, Option<Rc<B::TreeScope>>) {
        while let Some(current) = scope {
            if let Some(resolver) = B::TreeScopeScopedStyleResolver(&current) {
                if let Some(function) = B::ScopedResolverFunctionForName(&resolver, name) {
                    return (Some(function), Some(current));
                }
            }
            scope = B::ParentTreeScope(&current);
        }
        (
            self.user_function_rule_map_
                .borrow()
                .get(name)
                .map(|rule| rule.value.clone()),
            None,
        )
    }
    // cpp: style_engine.cc:4920-4923
    pub fn AddCachedFillOrClipPathURIValue(&self, key: AtomicString, value: &Rc<B::CSSValue>) {
        self.fill_or_clip_path_uri_value_cache_
            .borrow_mut()
            .insert(key, Rc::downgrade(value));
    }
    // cpp: style_engine.cc:4925-4932
    pub fn GetCachedFillOrClipPathURIValue(&self, key: &AtomicString) -> Option<Rc<B::CSSValue>> {
        self.fill_or_clip_path_uri_value_cache_
            .borrow()
            .get(key)
            .and_then(Weak::upgrade)
    }
    // cpp: style_engine.cc:4934-4936
    pub fn BaseURLChanged(&self) {
        self.fill_or_clip_path_uri_value_cache_.borrow_mut().clear();
    }
    // cpp: style_engine.cc:4945-4958
    fn UpdateLastSuccessfulPositionFallbackAndAnchorScrollShift(element: &B::Element) -> bool {
        if let Some(data) = B::ElementOutOfFlowData(element) {
            let layout = B::ElementLayoutObject(element);
            if B::OutOfFlowApplyPendingFallbackAndScrollShift(&data, layout.as_deref()) {
                if let Some(layout) = layout {
                    B::LayoutInvalidateAnchorPositioning(&layout);
                    return true;
                }
            }
        }
        false
    }
    // cpp: style_engine.cc:4960-4985
    fn InvalidatePositionTryNames(
        root: Option<Rc<B::Element>>,
        names: &HashSet<AtomicString>,
    ) -> bool {
        let Some(root) = root else {
            return false;
        };
        let mut invalidated = false;
        // The node cursor must include text/non-element nodes as in the source.
        let mut cursor = B::LayoutTreeRootNode(&root);
        while let Some(node) = cursor {
            if let Some(element) = B::NodeAsElement(&node) {
                if let Some(data) = B::ElementOutOfFlowData(&element) {
                    if B::OutOfFlowInvalidatePositionTryNames(&data, names) {
                        if let Some(layout) = B::ElementLayoutObject(&element) {
                            B::LayoutInvalidateAnchorPositioning(&layout);
                            invalidated = true;
                        }
                    }
                }
                if B::ComputedStyleIsNullOrEnsured(B::ElementComputedStyle(&element).as_deref()) {
                    cursor = B::LayoutTreeNextSkippingChildren(&element, &root);
                    continue;
                }
            }
            cursor = B::LayoutTreeNext(&node, &root);
        }
        invalidated
    }
    // cpp: style_engine.cc:4989-5011
    pub fn UpdateLastSuccessfulPositionFallbacksAndAnchorScrollShift(&self) -> bool {
        let mut invalidated = false;
        let names = self.dirty_position_try_names_.borrow().clone();
        if !names.is_empty() {
            invalidated |=
                Self::InvalidatePositionTryNames(B::DocumentElement(&self.document_), &names);
            self.dirty_position_try_names_.borrow_mut().clear();
        }
        let elements: Vec<_> = self
            .anchored_element_dirty_set_
            .borrow()
            .values()
            .cloned()
            .collect();
        if !elements.is_empty() {
            for element in elements {
                invalidated |=
                    Self::UpdateLastSuccessfulPositionFallbackAndAnchorScrollShift(&element);
            }
            self.anchored_element_dirty_set_.borrow_mut().clear();
        }
        invalidated
    }
    // cpp: style_engine.cc:5016-5031
    fn RevisitStyleRulesForInspector(
        &self,
        features: &B::RuleFeatureSet,
        rules: Vec<Rc<B::StyleRuleBase>>,
    ) {
        for rule in rules {
            if let Some(style) = B::InspectorAsStyleRule(&rule) {
                for selector in B::InspectorSelectors(&style) {
                    let _scope = B::NewInspectorSelectorScope(&style, &selector);
                    B::FeaturesRevisitSelector(features, &selector);
                }
            } else if let Some(children) = B::InspectorGroupChildRules(&rule) {
                self.RevisitStyleRulesForInspector(features, children);
            }
        }
    }
    // cpp: style_engine.cc:5035-5050
    pub fn RevisitStyleSheetForInspector(
        &self,
        contents: &B::StyleSheetContents,
        features: Option<&B::RuleFeatureSet>,
    ) {
        let _scope = B::NewInspectorContentsScope(contents);
        self.RevisitStyleRulesForInspector(
            &self.GetRuleFeatureSet(),
            B::ContentsChildRules(contents),
        );
        if let Some(features) = features {
            self.RevisitStyleRulesForInspector(features, B::ContentsChildRules(contents));
        }
    }
    // cpp: style_engine.cc:5052-5059
    pub fn NavigationsMayHaveChanged(&self) {
        self.SetNeedsActiveStyleUpdate(&B::DocumentTreeScope(&self.document_));
        self.InvalidateFunctionalNavigationDependentStylesIfNeeded();
    }
    // cpp: style_engine.cc:5061-5093
    pub fn GetCachedRandomBaseValue(
        &self,
        key: &B::RandomCacheKey,
        element: Option<&Rc<B::Element>>,
    ) -> f64 {
        if !B::RandomKeyIsElementScoped(key) {
            let name = B::RandomNameForCaching(key);
            return *self
                .element_shared_random_base_value_cache_
                .borrow_mut()
                .entry(name)
                .or_insert_with(B::RandDouble);
        }
        let caching_key = B::NewRandomCachingKey(key, element);
        {
            // Weak-key lifetime entries reproduce the source GC ephemeron owner:
            // a dead Element no longer keeps its per-element keys alive.
            let mut lifetimes = self.element_keeps_random_caching_key_alive_.borrow_mut();
            lifetimes.retain(|_, (owner, _)| {
                owner.as_ref().is_none_or(|owner| owner.strong_count() != 0)
            });
            let id = element.map_or(0, identity);
            lifetimes
                .entry(id)
                .or_insert_with(|| (element.map(Rc::downgrade), HashSet::new()))
                .1
                .insert(caching_key.clone());
        }
        let mut cache = self.random_base_value_cache_.borrow_mut();
        cache.retain(|(key, _)| key.strong_count() != 0);
        for (weak, value) in cache.iter() {
            if let Some(old) = weak.upgrade() {
                if old == caching_key {
                    return *value;
                }
            }
        }
        let value = B::RandDouble();
        cache.push((Rc::downgrade(&caching_key), value));
        value
    }
    // cpp: style_engine.h:378-380
    pub fn UsesFirstLineRules(&self) -> bool {
        B::RuleInvalidationUsesFirstLineRules(&self.RuleInvalidationData())
    }
    pub fn UsesWindowInactiveSelector(&self) -> bool {
        B::RuleInvalidationUsesWindowInactiveSelector(&self.RuleInvalidationData())
    }
    // cpp: style_engine.h:589-591
    pub fn FlatTreePositionChanged(&self, node: &B::Node) {
        B::StyleRecalcRootFlatTreePositionChanged(&mut self.style_recalc_root_.borrow_mut(), node);
    }
    pub fn PseudoElementRemoved(&self, element: &B::Element) {
        B::LayoutTreeRebuildRootSubtreeModified(
            &mut self.layout_tree_rebuild_root_.borrow_mut(),
            element,
        );
    }
    // cpp: style_engine.h:650-652
    pub fn MarkLastSuccessfulPositionFallbackDirtyForElement(&self, element: Rc<B::Element>) {
        self.anchored_element_dirty_set_
            .borrow_mut()
            .insert(identity(&element), element);
    }
    pub fn MarkAnchorRememberedOffsetsChanged(&self, element: Rc<B::Element>) {
        self.anchored_element_dirty_set_
            .borrow_mut()
            .insert(identity(&element), element);
    }
    // cpp: style_engine.h:664-666
    pub fn GetUserCounterStyleMap(&self) -> Option<Rc<B::CounterStyleMap>> {
        self.user_counter_style_map_.borrow().clone()
    }
    pub fn GetUserCascadeLayerMap(&self) -> Option<Rc<CascadeLayerMap>> {
        self.user_cascade_layer_map_.borrow().clone()
    }
    // cpp: style_engine.h:686-688
    pub fn NeedsStyleInvalidation(&self) -> bool {
        B::StyleInvalidationRootNode(&self.style_invalidation_root_.borrow()).is_some()
    }
    pub fn NeedsStyleRecalc(&self) -> bool {
        B::StyleRecalcRootNode(&self.style_recalc_root_.borrow()).is_some()
    }
    pub fn NeedsLayoutTreeRebuild(&self) -> bool {
        B::LayoutTreeRebuildRootNode(&self.layout_tree_rebuild_root_.borrow()).is_some()
    }
    // cpp: style_engine.h:749-757
    pub fn GetInterleavingRecalcRoot(&self) -> Option<Rc<B::Element>> {
        if self.InInterleavedStyleRecalc() {
            B::StyleRecalcRootNode(&self.style_recalc_root_.borrow())
                .and_then(|node| B::NodeAsElement(&node))
        } else {
            None
        }
    }
    // cpp: style_engine.h:775-775
    pub fn GetPageColorSchemes(&self) -> ColorSchemeFlags {
        self.page_color_schemes_.get()
    }
    pub fn GetPreferredColorScheme(&self) -> PreferredColorScheme {
        self.preferred_color_scheme.get()
    }
    pub fn GetPreferredContrast(&self) -> PreferredContrast {
        self.preferred_contrast_.get()
    }
    pub fn GetForceDarkModeEnabled(&self) -> bool {
        self.force_dark_mode_enabled_.get()
    }
    pub fn GetForcedColors(&self) -> ForcedColors {
        self.forced_colors_.get()
    }
    pub fn ForcedBackgroundColor(&self) -> Color {
        self.forced_background_color_.get()
    }
    pub fn CacheImageContent(
        &self,
        params: &mut B::FetchParameters,
    ) -> Rc<B::ImageResourceContent> {
        B::ImageCacheImageContent(&self.style_image_cache_, &self.document_, params)
    }
    // cpp: style_engine.h:805-807
    pub fn ActiveUserStyleSheets(&self) -> ActiveStyleSheetVector<B> {
        self.sheet_state.borrow().active_user_style_sheets.clone()
    }
    // cpp: style_engine.h:811-815
    pub fn GetViewportSize(&self) -> std::cell::Ref<'_, B::ViewportSize> {
        std::cell::Ref::map(self.viewport_size_.borrow(), |size| {
            size.as_ref()
                .expect("viewport size updated before resolving style")
        })
    }
    // cpp: style_engine.h:892-898
    pub fn GetDocumentStyleSheetCollection(&self) -> Rc<B::StyleSheetCollection> {
        self.document_style_sheet_collection_.clone()
    }
    // cpp: style_engine.cc:4552-4554
    pub fn ColorSchemeChanged(&self) {
        self.UpdateColorScheme();
    }

    // cpp: style_engine.h:880-883
    pub fn GetRuleFeatureSet(&self) -> Rc<B::RuleFeatureSet> {
        B::GlobalRuleFeatureSet(&self.global_rule_set_.get().expect("global rule set"))
    }
    fn RuleInvalidationData(&self) -> Rc<B::RuleInvalidationData> {
        B::FeaturesRuleInvalidationData(&self.GetRuleFeatureSet())
    }
    // cpp: style_engine.h:373-377
    pub fn MaxDirectAdjacentSelectors(&self) -> u32 {
        B::RuleInvalidationMaxDirectAdjacentSelectors(&self.RuleInvalidationData())
    }
    // cpp: style_engine.h:448-450
    pub fn GetPendingNodeInvalidations(&self) -> &B::PendingInvalidations {
        &self.pending_invalidations_
    }
    // cpp: style_engine.cc:1494-1509
    pub fn PossiblyScheduleNthPseudoInvalidations(node: &B::Node) {
        if B::NodeAsElement(node).is_none() {
            return;
        }
        let Some(parent) = B::NodeParentNode(node) else {
            return;
        };
        if (B::ChildrenAffectedByForwardPositionalRules(&parent)
            && B::NodeNextSibling(node).is_some())
            || (B::ChildrenAffectedByBackwardPositionalRules(&parent)
                && B::NodePreviousSibling(node).is_some())
        {
            B::NodeStyleEngine(node).ScheduleNthPseudoInvalidations(&parent);
        }
    }
    // cpp: style_engine.cc:2146-2172
    pub fn ScheduleNthPseudoInvalidations(&self, parent: &B::ContainerNode) {
        let mut lists = B::NewInvalidationLists();
        B::CollectNthInvalidationSet(&self.RuleInvalidationData(), &mut lists);
        if self.uses_tree_counting_functions_.get() {
            B::InvalidationListsPushSibling(&mut lists, B::TreeCountingInvalidationSet());
        }
        B::ScheduleInvalidationSetsForNode(
            &self.pending_invalidations_,
            lists,
            B::ContainerNodeAsNode(parent),
        );
    }
    // cpp: style_engine.cc:1511-1543
    fn InvalidateElementAffectedByHas(&self, element: &B::Element, pseudo_only: bool) {
        if pseudo_only && !B::AffectedByPseudoInHas(element) {
            return;
        }
        if B::AffectedBySubjectHas(element) {
            B::ElementSetNeedsStyleRecalc(
                element,
                StyleChangeType::kLocalStyleChange,
                StyleChangeReason::AffectedByHas,
            );
            if B::UsesHasInsideNth(&self.RuleInvalidationData()) {
                Self::PossiblyScheduleNthPseudoInvalidations(B::ElementAsNode(element));
            }
        }
        if B::AffectedByNonSubjectHas(element) {
            let data = self.RuleInvalidationData();
            let mut lists = B::NewInvalidationLists();
            B::CollectInvalidationSetsForPseudoClass(
                &data,
                &mut lists,
                element,
                PseudoType::kPseudoHas,
            );
            B::ScheduleInvalidationSetsForNode(
                &self.pending_invalidations_,
                lists,
                B::ElementAsNode(element),
            );
        }
    }
    // cpp: style_engine.cc:1708-1752
    fn InvalidateAncestorsOrSiblingsAffectedByHas(
        &self,
        context: PseudoHasInvalidationTraversalContext<B>,
    ) {
        let mut traverse_parent = context.traverse_to_parent_of_first_element_;
        let mut element = context.first_element_;
        let mut shadow_host = None;
        if context.is_first_element_shadow_host_ {
            shadow_host = element.take();
        }
        while let Some(current) = element {
            traverse_parent |= B::AncestorsOrAncestorSiblingsAffectedByHas(&current);
            let traverse_previous = B::SiblingsAffectedByHasFlags(&current) != 0;
            self.InvalidateElementAffectedByHas(
                &current,
                context.for_element_affected_by_pseudo_in_has_,
            );
            if traverse_previous {
                if let Some(previous) = B::PreviousElementSibling(B::ElementAsNode(&current)) {
                    element = Some(previous);
                    continue;
                }
            }
            if !traverse_parent {
                return;
            }
            element = B::ElementParentElement(&current);
            if element.is_none() {
                shadow_host = B::ParentOrShadowHostElement(&current);
            }
            traverse_parent = false;
        }
        if let Some(host) = shadow_host {
            self.InvalidateElementAffectedByHas(
                &host,
                context.for_element_affected_by_pseudo_in_has_,
            );
        }
    }
    // cpp: style_engine.cc:1754-1762
    fn InvalidateChangedElementAffectedByLogicalCombinationsInHas(
        &self,
        element: &B::Element,
        pseudo_only: bool,
    ) {
        if !B::AffectedByLogicalCombinationsInHas(element) {
            return;
        }
        self.InvalidateElementAffectedByHas(element, pseudo_only);
    }
    // cpp: style_engine.cc:1764-1799
    pub fn ClassChangedForElement(&self, classes: &B::SpaceSplitString, element: &B::Element) {
        if self.ShouldSkipInvalidationFor(element) {
            return;
        }
        let data = self.RuleInvalidationData();
        if B::NeedsHasInvalidationForClassChange(&data) && Self::PossiblyAffectingHasState(element)
        {
            for class in B::SpaceSplitStringValues(classes) {
                if B::NeedsHasInvalidationForClass(&data, class) {
                    self.InvalidateChangedElementAffectedByLogicalCombinationsInHas(element, false);
                    self.InvalidateAncestorsOrSiblingsAffectedByHas(
                        PseudoHasInvalidationTraversalContext::ForAttributeOrPseudoStateChange(
                            element,
                        ),
                    );
                    break;
                }
            }
        }
        if self.IsSubtreeAndSiblingsStyleDirty(element) {
            return;
        }
        let mut lists = B::NewInvalidationLists();
        for class in B::SpaceSplitStringValues(classes) {
            B::CollectInvalidationSetsForClass(&data, &mut lists, element, class);
        }
        B::ScheduleInvalidationSetsForNode(
            &self.pending_invalidations_,
            lists,
            B::ElementAsNode(element),
        );
    }
    // cpp: style_engine.cc:1801-1885
    pub fn ClassNamesChangedForElement(
        &self,
        old: &B::SpaceSplitString,
        new: &B::SpaceSplitString,
        element: &B::Element,
    ) {
        if self.ShouldSkipInvalidationFor(element) {
            return;
        }
        let old = B::SpaceSplitStringValues(old);
        if old.is_empty() {
            self.ClassChangedForElement(new, element);
            return;
        }
        let new = B::SpaceSplitStringValues(new);
        let data = self.RuleInvalidationData();
        let schedule = !self.IsSubtreeAndSiblingsStyleDirty(element);
        let mut possibly_affecting = B::NeedsHasInvalidationForClassChange(&data)
            && Self::PossiblyAffectingHasState(element);
        if !schedule && !possibly_affecting {
            return;
        }
        let mut remaining = vec![false; old.len()];
        let mut lists = B::NewInvalidationLists();
        let mut affecting = false;
        for new_class in new {
            let mut found = false;
            for (i, old_class) in old.iter().enumerate() {
                if new_class == old_class {
                    remaining[i] = true;
                    found = true;
                }
            }
            if !found {
                if schedule {
                    B::CollectInvalidationSetsForClass(&data, &mut lists, element, new_class);
                }
                if possibly_affecting && B::NeedsHasInvalidationForClass(&data, new_class) {
                    affecting = true;
                    possibly_affecting = false;
                }
            }
        }
        for (i, old_class) in old.iter().enumerate() {
            if remaining[i] {
                continue;
            }
            if schedule {
                B::CollectInvalidationSetsForClass(&data, &mut lists, element, old_class);
            }
            if possibly_affecting && B::NeedsHasInvalidationForClass(&data, old_class) {
                affecting = true;
                possibly_affecting = false;
            }
        }
        if schedule {
            B::ScheduleInvalidationSetsForNode(
                &self.pending_invalidations_,
                lists,
                B::ElementAsNode(element),
            );
        }
        if affecting {
            self.InvalidateChangedElementAffectedByLogicalCombinationsInHas(element, false);
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForAttributeOrPseudoStateChange(element),
            );
        }
    }
    // cpp: style_engine.cc:1889-1895
    fn HasAttributeDependentStyle(element: &B::Element) -> bool {
        if B::ElementComputedStyle(element).is_some_and(|style| B::HasAttrFunction(&style)) {
            return true;
        }
        B::PseudoElementStylesDependOnAttr(element)
    }
    // cpp: style_engine.cc:1899-1939
    pub fn AttributeChangedForElement(&self, name: &QualifiedName, element: &B::Element) {
        if self.ShouldSkipInvalidationFor(element) {
            return;
        }
        let data = self.RuleInvalidationData();
        if B::NeedsHasInvalidationForAttributeChange(&data)
            && Self::PossiblyAffectingHasState(element)
            && B::NeedsHasInvalidationForAttribute(&data, name)
        {
            self.InvalidateChangedElementAffectedByLogicalCombinationsInHas(element, false);
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForAttributeOrPseudoStateChange(element),
            );
        }
        if self.IsSubtreeAndSiblingsStyleDirty(element) {
            return;
        }
        let mut lists = B::NewInvalidationLists();
        B::CollectInvalidationSetsForAttribute(&data, &mut lists, element, name);
        B::ScheduleInvalidationSetsForNode(
            &self.pending_invalidations_,
            lists,
            B::ElementAsNode(element),
        );
        if !B::ElementNeedsStyleRecalc(element) && Self::HasAttributeDependentStyle(element) {
            // Attribute name in the tracing reason is diagnostic-only.
            B::ElementSetNeedsStyleRecalc(
                element,
                StyleChangeType::kLocalStyleChange,
                StyleChangeReason::AttributeChange,
            );
        }
    }
    // cpp: style_engine.cc:1941-1980
    pub fn IdChangedForElement(
        &self,
        old: &AtomicString,
        new: &AtomicString,
        element: &B::Element,
    ) {
        if self.ShouldSkipInvalidationFor(element) {
            return;
        }
        let data = self.RuleInvalidationData();
        if B::NeedsHasInvalidationForIdChange(&data)
            && Self::PossiblyAffectingHasState(element)
            && ((!old.empty() && B::NeedsHasInvalidationForId(&data, old))
                || (!new.empty() && B::NeedsHasInvalidationForId(&data, new)))
        {
            self.InvalidateChangedElementAffectedByLogicalCombinationsInHas(element, false);
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForAttributeOrPseudoStateChange(element),
            );
        }
        if self.IsSubtreeAndSiblingsStyleDirty(element) {
            return;
        }
        let mut lists = B::NewInvalidationLists();
        if !old.empty() {
            B::CollectInvalidationSetsForId(&data, &mut lists, element, old);
        }
        if !new.empty() {
            B::CollectInvalidationSetsForId(&data, &mut lists, element, new);
        }
        B::ScheduleInvalidationSetsForNode(
            &self.pending_invalidations_,
            lists,
            B::ElementAsNode(element),
        );
    }
    // cpp: style_engine.cc:1982-2021
    pub fn PseudoStateChangedForElement(
        &self,
        pseudo: PseudoType,
        element: &B::Element,
        descendants_or_siblings: bool,
        ancestors_or_siblings: bool,
    ) {
        debug_assert!(descendants_or_siblings || ancestors_or_siblings);
        if self.ShouldSkipInvalidationFor(element) {
            return;
        }
        let data = self.RuleInvalidationData();
        if ancestors_or_siblings
            && B::NeedsHasInvalidationForPseudoStateChange(&data)
            && Self::PossiblyAffectingHasState(element)
            && B::NeedsHasInvalidationForPseudoClass(&data, pseudo)
        {
            self.InvalidateChangedElementAffectedByLogicalCombinationsInHas(element, true);
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForAttributeOrPseudoStateChange(element)
                    .SetForElementAffectedByPseudoInHas(),
            );
        }
        if !descendants_or_siblings || self.IsSubtreeAndSiblingsStyleDirty(element) {
            return;
        }
        let mut lists = B::NewInvalidationLists();
        B::CollectInvalidationSetsForPseudoClass(&data, &mut lists, element, pseudo);
        B::ScheduleInvalidationSetsForNode(
            &self.pending_invalidations_,
            lists,
            B::ElementAsNode(element),
        );
    }
    // cpp: style_engine.cc:2023-2039
    pub fn PartChangedForElement(&self, element: &B::Element) {
        if self.ShouldSkipInvalidationFor(element)
            || self.IsSubtreeAndSiblingsStyleDirty(element)
            || self.IsDocumentScope(&B::NodeTreeScope(B::ElementAsNode(element)))
            || !B::InvalidatesParts(&self.RuleInvalidationData())
        {
            return;
        }
        B::ElementSetNeedsStyleRecalc(
            element,
            StyleChangeType::kLocalStyleChange,
            StyleChangeReason::AttributeChange,
        );
    }
    // cpp: style_engine.cc:2041-2057
    pub fn ExportpartsChangedForElement(&self, element: &B::Element) {
        if self.ShouldSkipInvalidationFor(element)
            || self.IsSubtreeAndSiblingsStyleDirty(element)
            || B::ElementShadowRoot(element).is_none()
        {
            return;
        }
        let mut lists = B::NewInvalidationLists();
        B::CollectPartInvalidationSet(&self.RuleInvalidationData(), &mut lists);
        B::ScheduleInvalidationSetsForNode(
            &self.pending_invalidations_,
            lists,
            B::ElementAsNode(element),
        );
    }
    // cpp: style_engine.cc:2059-2094
    fn ScheduleSiblingInvalidationsForElement(
        &self,
        element: &B::Element,
        parent: &B::ContainerNode,
        min: u32,
    ) {
        debug_assert!(min != 0);
        let mut lists = B::NewInvalidationLists();
        let data = self.RuleInvalidationData();
        if B::ElementHasID(element) {
            B::CollectSiblingInvalidationSetForId(
                &data,
                &mut lists,
                element,
                &B::ElementIDForStyleResolution(element),
                min,
            );
        }
        if B::ElementHasClass(element) {
            for class in B::SpaceSplitStringValues(&B::ElementClassNames(element)) {
                B::CollectSiblingInvalidationSetForClass(&data, &mut lists, element, class, min);
            }
        }
        for name in B::ElementAttributeNames(element) {
            B::CollectSiblingInvalidationSetForAttribute(&data, &mut lists, element, &name, min);
        }
        B::CollectUniversalSiblingInvalidationSet(&data, &mut lists, min);
        B::ScheduleSiblingInvalidationsAsDescendants(&self.pending_invalidations_, lists, parent);
    }
    // cpp: style_engine.cc:2096-2119
    pub fn ScheduleInvalidationsForInsertedSibling(
        &self,
        mut before: Option<Rc<B::Element>>,
        inserted: &B::Element,
    ) {
        let parent = B::ElementParentNode(inserted).expect("inserted sibling parent");
        let affected = if B::ChildrenAffectedByIndirectAdjacentRules(&parent) {
            u32::MAX
        } else {
            self.MaxDirectAdjacentSelectors()
        };
        let Some(parent) = B::ParentElementOrShadowRoot(inserted) else {
            return;
        };
        self.ScheduleSiblingInvalidationsForElement(inserted, &parent, 1);
        let mut i = 1;
        while let Some(element) = before {
            if i > affected {
                break;
            }
            self.ScheduleSiblingInvalidationsForElement(&element, &parent, i);
            i = i.wrapping_add(1);
            before = B::PreviousElementSibling(B::ElementAsNode(&element));
        }
    }
    // cpp: style_engine.cc:2121-2144
    pub fn ScheduleInvalidationsForRemovedSibling(
        &self,
        mut before: Option<Rc<B::Element>>,
        removed: &B::Element,
        after: &B::Element,
    ) {
        let parent = B::ElementParentNode(after).expect("remaining sibling parent");
        let affected = if B::ChildrenAffectedByIndirectAdjacentRules(&parent) {
            u32::MAX
        } else {
            self.MaxDirectAdjacentSelectors()
        };
        let Some(parent) = B::ParentElementOrShadowRoot(after) else {
            return;
        };
        self.ScheduleSiblingInvalidationsForElement(removed, &parent, 1);
        let mut i = 1;
        while let Some(element) = before {
            if i > affected {
                break;
            }
            self.ScheduleSiblingInvalidationsForElement(&element, &parent, i);
            i = i.wrapping_add(1);
            before = B::PreviousElementSibling(B::ElementAsNode(&element));
        }
    }
    // cpp: style_engine.cc:2263-2302
    pub fn ScheduleInvalidationsForHasPseudoAffectedByInsertionOrRemoval(
        &self,
        parent: Option<&B::ContainerNode>,
        before_change: Option<&B::Node>,
        changed: &B::Element,
        removal: bool,
    ) {
        let (parent, shadow_child) = match parent {
            Some(parent) => {
                if let Some(element) = B::ContainerAsElement(parent) {
                    (Some(element), false)
                } else {
                    (B::ContainerNodeShadowHost(parent), true)
                }
            }
            None => (None, false),
        };
        let Some(parent) = parent else {
            return;
        };
        if self.ShouldSkipInvalidationFor(&parent)
            || !B::NeedsHasInvalidationForInsertionOrRemoval(&self.RuleInvalidationData())
        {
            return;
        }
        let previous = Self::SelfOrPreviousSibling(before_change);
        if removal {
            self.ScheduleInvalidationsForHasPseudoAffectedByRemoval(
                Some(parent),
                previous,
                changed,
                shadow_child,
            );
        } else {
            self.ScheduleInvalidationsForHasPseudoAffectedByInsertion(
                Some(parent),
                previous,
                changed,
                shadow_child,
            );
        }
    }
    // cpp: style_engine.cc:2304-2374
    fn ScheduleInvalidationsForHasPseudoAffectedByInsertion(
        &self,
        parent: Option<Rc<B::Element>>,
        previous: Option<Rc<B::Element>>,
        inserted: &B::Element,
        shadow_child: bool,
    ) {
        let mut possibly = false;
        let mut descendants = false;
        if Self::InsertionOrRemovalPossiblyAffectHasStateOfPreviousSiblings(previous.as_deref()) {
            B::SetSiblingsAffectedByHasFlags(
                inserted,
                B::SiblingsAffectedByHasFlags(previous.as_ref().unwrap()),
            );
            possibly = true;
            descendants = B::HasSiblingsAffectedByHasForSiblingDescendantRelationship(inserted);
        }
        if Self::InsertionOrRemovalPossiblyAffectHasStateOfAncestorsOrAncestorSiblings(
            parent.as_deref(),
        ) {
            B::SetAncestorsOrAncestorSiblingsAffectedByHas(inserted);
            possibly = true;
            descendants = true;
        }
        if !possibly {
            return;
        }
        let data = self.RuleInvalidationData();
        let mut needs =
            B::ChildrenAffectedByDirectAdjacentRules(parent.as_ref().expect("insertion parent"));
        if !needs && B::NeedsHasInvalidationForInsertedOrRemovedElement(&data, inserted) {
            needs = true;
        }
        if descendants {
            for element in B::ElementDescendants(inserted) {
                B::SetAncestorsOrAncestorSiblingsAffectedByHas(&element);
                if !needs && B::NeedsHasInvalidationForInsertedOrRemovedElement(&data, &element) {
                    needs = true;
                }
            }
        }
        if needs {
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForInsertion(parent, shadow_child, previous),
            );
            return;
        }
        if B::NeedsHasInvalidationForPseudoStateChange(&data) {
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForInsertion(parent, shadow_child, previous)
                    .SetForElementAffectedByPseudoInHas(),
            );
        }
    }
    // cpp: style_engine.cc:2376-2423
    fn ScheduleInvalidationsForHasPseudoAffectedByRemoval(
        &self,
        parent: Option<Rc<B::Element>>,
        previous: Option<Rc<B::Element>>,
        removed: &B::Element,
        shadow_child: bool,
    ) {
        if !Self::InsertionOrRemovalPossiblyAffectHasStateOfAncestorsOrAncestorSiblings(
            parent.as_deref(),
        ) && !Self::InsertionOrRemovalPossiblyAffectHasStateOfPreviousSiblings(
            previous.as_deref(),
        ) {
            return;
        }
        if B::ChildrenAffectedByDirectAdjacentRules(parent.as_ref().expect("removal parent")) {
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForRemoval(
                    parent,
                    shadow_child,
                    previous,
                    removed,
                ),
            );
            return;
        }
        let data = self.RuleInvalidationData();
        if B::NeedsHasInvalidationForInsertedOrRemovedElement(&data, removed) {
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForRemoval(
                    parent,
                    shadow_child,
                    previous,
                    removed,
                ),
            );
            return;
        }
        for element in B::ElementDescendants(removed) {
            if B::NeedsHasInvalidationForInsertedOrRemovedElement(&data, &element) {
                self.InvalidateAncestorsOrSiblingsAffectedByHas(
                    PseudoHasInvalidationTraversalContext::ForRemoval(
                        parent,
                        shadow_child,
                        previous,
                        removed,
                    ),
                );
                return;
            }
        }
        if B::NeedsHasInvalidationForPseudoStateChange(&data) {
            self.InvalidateAncestorsOrSiblingsAffectedByHas(
                PseudoHasInvalidationTraversalContext::ForRemoval(
                    parent,
                    shadow_child,
                    previous,
                    removed,
                )
                .SetForElementAffectedByPseudoInHas(),
            );
        }
    }
    // cpp: style_engine.cc:2425-2446
    pub fn ScheduleInvalidationsForHasPseudoWhenAllChildrenRemoved(&self, parent: &Rc<B::Element>) {
        if self.ShouldSkipInvalidationFor(parent) {
            return;
        }
        let data = self.RuleInvalidationData();
        if !B::NeedsHasInvalidationForInsertionOrRemoval(&data)
            || !Self::InsertionOrRemovalPossiblyAffectHasStateOfAncestorsOrAncestorSiblings(Some(
                parent,
            ))
        {
            return;
        }
        self.InvalidateAncestorsOrSiblingsAffectedByHas(
            PseudoHasInvalidationTraversalContext::ForAllChildrenRemoved(parent),
        );
    }
    // cpp: style_engine.cc:2448-2454
    pub fn InvalidateStyle(&self) {
        let mut invalidator = B::NewStyleInvalidator(&self.pending_invalidations_);
        let root = B::StyleInvalidationRootElement(&self.style_invalidation_root_.borrow());
        B::StyleInvalidatorInvalidate(&mut invalidator, &self.document_, root.as_deref());
        B::ClearStyleInvalidationRoot(&mut self.style_invalidation_root_.borrow_mut());
    }
    // cpp: style_engine.cc:2466-2472
    pub fn HasViewportDependentMediaQueries(&self) -> bool {
        self.UpdateActiveStyle();
        B::FeaturesHasViewportDependentMediaQueries(&self.GetRuleFeatureSet())
            || self
                .functional_media_query_result_flags_
                .borrow()
                .is_viewport_dependent
    }
    // cpp: style_engine.cc:2474-2478
    pub fn HasViewportDependentPropertyRegistrations(&self) -> bool {
        self.UpdateActiveStyle();
        B::DocumentPropertyRegistry(&self.document_)
            .is_some_and(|registry| B::PropertyRegistryViewportUnitFlags(&registry) != 0)
    }
    // cpp: style_engine.cc:2682-2685
    pub fn CollectFeaturesTo(&self, features: &mut B::RuleFeatureSet) {
        self.CollectUserStyleFeaturesTo(features);
        self.CollectScopedStyleFeaturesTo(features);
    }
    // cpp: style_engine.cc:2687-2696
    pub fn EnsureUAStyleForFullscreen(&self, element: &B::Element) {
        let global = self.global_rule_set_.get().expect("global rule set");
        if B::GlobalHasFullscreenUAStyle(&global) {
            return;
        }
        B::DefaultSheetsEnsureFullscreen(element);
        B::GlobalRuleSetMarkDirty(&global);
        self.UpdateActiveStyle();
    }
    // cpp: style_engine.cc:2698-2705
    pub fn EnsureUAStyleForElement(&self, element: &B::Element) {
        if B::DefaultSheetsEnsureElement(element) {
            B::GlobalRuleSetMarkDirty(&self.global_rule_set_.get().expect("global rule set"));
            self.UpdateActiveStyle();
        }
    }
    // cpp: style_engine.cc:2707-2715
    pub fn EnsureUAStyleForPseudoElement(&self, pseudo: PseudoId) {
        if B::DefaultSheetsEnsurePseudoElement(pseudo) {
            B::GlobalRuleSetMarkDirty(&self.global_rule_set_.get().expect("global rule set"));
            self.UpdateActiveStyle();
        }
    }
    // cpp: style_engine.cc:2717-2726
    pub fn EnsureUAStyleForForcedColors(&self) {
        if B::DefaultSheetsEnsureForcedColors() {
            B::GlobalRuleSetMarkDirty(&self.global_rule_set_.get().expect("global rule set"));
            if B::DocumentIsActive(&self.document_) {
                self.UpdateActiveStyle();
            }
        }
    }
    // cpp: style_engine.cc:2728-2737
    pub fn ActiveViewTransitionStyle(&self, element: &B::Element) -> Option<Rc<B::RuleSet>> {
        let transition = B::ElementViewTransition(element)?;
        let sheet = B::ViewTransitionUAStyleSheet(&transition);
        let contents = B::SheetContents(&sheet);
        Some(B::ContentsEnsureRuleSet(
            &contents,
            &B::DefaultSheetsScreenEvaluator(),
            &B::EmptyMixinMap(),
        ))
    }
    // cpp: style_engine.cc:2769-2774
    pub fn HasRulesForId(&self, id: &AtomicString) -> bool {
        B::HasSelectorForId(&self.RuleInvalidationData(), id)
    }

    // cpp: style_engine.cc:864-874
    pub fn UpdateCounters(&self) {
        if !self.CountersChanged() {
            return;
        }
        let Some(root) = B::DocumentElement(&self.document_) else {
            return;
        };
        self.counters_changed_.set(false);
        let mut context = B::NewCountersAttachmentContext();
        B::SetCounterAttachmentRootIsDocumentElement(&mut context);
        self.UpdateCountersForElement(&root, &mut context);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:880-892
    fn UpdateLayoutCounters(object: &B::LayoutObject, context: &mut B::CountersAttachmentContext) {
        let mut child = B::NextLayoutObjectInPreOrder(object, object);
        while let Some(current) = child {
            if let Some(counter) = B::LayoutAsCounter(&current) {
                let values = B::CounterValues(
                    context,
                    object,
                    &B::LayoutCounterIdentifier(&counter),
                    B::LayoutCounterSeparatorIsNull(&counter),
                );
                B::LayoutCounterUpdate(&counter, values);
            }
            child = B::NextLayoutObjectInPreOrder(&current, object);
        }
    }
    // cpp: style_engine.cc:896-913
    fn UpdateAltCounters(
        &self,
        object: &B::LayoutObject,
        context: &mut B::CountersAttachmentContext,
    ) {
        let Some(pseudo) = B::LayoutObjectPseudoElement(object) else {
            return;
        };
        let mut content = B::CreateMutableAltContentDataForCountersIfNeeded(&pseudo);
        while let Some(current) = content {
            if let Some(counter) = B::ContentAsAltCounter(&current) {
                B::AltCounterUpdateText(&counter, context, self, object);
            }
            content = B::ContentDataNext(&current);
        }
    }
    // cpp: style_engine.cc:917-952
    fn UpdateCountersForElement(
        &self,
        element: &B::Element,
        context: &mut B::CountersAttachmentContext,
    ) {
        let object = B::ElementLayoutObject(element);
        if let Some(object) = &object {
            B::CounterContextEnterObject(context, object);
            if let Some(item) = B::LayoutAsListItem(object) {
                let ordinal = B::ListItemOrdinal(&item);
                if B::CSSListCounterAccountingEnabled() || !B::OrdinalUseExplicitValue(&ordinal) {
                    B::OrdinalMarkDirty(&ordinal);
                    B::ListItemOrdinalValueChanged(&item);
                }
            } else if let Some(item) = B::LayoutAsInlineListItem(object) {
                let ordinal = B::InlineListItemOrdinal(&item);
                if B::CSSListCounterAccountingEnabled() || !B::OrdinalUseExplicitValue(&ordinal) {
                    B::OrdinalMarkDirty(&ordinal);
                    B::InlineListItemOrdinalValueChanged(&item);
                }
            }
            if B::ElementComputedStyle(element)
                .is_some_and(|style| !B::ContentBehavesAsNormal(&style))
            {
                self.UpdateAltCounters(object, context);
                Self::UpdateLayoutCounters(object, context);
            }
        }
        for child in B::LayoutTreeBuilderElementChildren(element) {
            self.UpdateCountersForElement(&child, context);
        }
        if let Some(object) = &object {
            B::CounterContextLeaveObject(context, object);
        }
    }
    // cpp: style_engine.cc:954-956
    pub fn SetNeedsToUpdateComplexSafeAreaConstraints(&self) {
        self.needs_to_update_complex_safe_area_constraints_
            .set(true);
    }
    // cpp: style_engine.cc:958-966
    pub fn ShadowRootInsertedToDocument(&self, scope: &Rc<B::TreeScope>) {
        debug_assert!(B::TreeScopeRootIsConnected(scope));
        if B::DocumentIsDetached(&self.document_) || !B::TreeScopeHasAdoptedStyleSheets(scope) {
            return;
        }
        self.EnsureStyleSheetCollectionFor(scope);
        self.SetNeedsActiveStyleUpdate(scope);
        self.sheet_state
            .borrow_mut()
            .active_tree_scopes
            .insert(identity(scope), scope.clone());
    }
    // cpp: style_engine.cc:968-974
    pub fn ShadowRootRemovedFromDocument(&self, scope: &Rc<B::TreeScope>) {
        self.style_sheet_collection_map_
            .borrow_mut()
            .remove(&identity(scope));
        {
            let mut state = self.sheet_state.borrow_mut();
            state.active_tree_scopes.remove(&identity(scope));
            state.dirty_tree_scopes.remove(&identity(scope));
            state.tree_scopes_removed = true;
        }
        self.ResetAuthorStyle(scope);
    }
    // cpp: style_engine.h:427-430
    pub fn GetStyleResolver(&self) -> Rc<B::StyleResolver> {
        self.resolver_.get().expect("document style resolver")
    }
    // cpp: style_engine.h:433-435
    pub fn GetStyleContainmentScopeTree(&self) -> Option<Rc<B::StyleContainmentScopeTree>> {
        self.style_containment_scope_tree_.get()
    }
    // cpp: style_engine.cc:993-999
    pub fn EnsureStyleContainmentScopeTree(&self) -> Rc<B::StyleContainmentScopeTree> {
        if let Some(tree) = self.style_containment_scope_tree_.get() {
            return tree;
        }
        let tree = B::NewStyleContainmentScopeTree();
        self.style_containment_scope_tree_.set(Some(tree.clone()));
        tree
    }
    // cpp: style_engine.h:438-440
    pub fn GetScrollTargetGroupScopeTree(&self) -> Option<Rc<B::ScrollTargetGroupScopeTree>> {
        self.scroll_target_group_scope_tree_.get()
    }
    // cpp: style_engine.cc:1001-1007
    pub fn EnsureScrollTargetGroupScopeTree(&self) -> Rc<B::ScrollTargetGroupScopeTree> {
        if let Some(tree) = self.scroll_target_group_scope_tree_.get() {
            return tree;
        }
        let tree = B::NewScrollTargetGroupScopeTree();
        self.scroll_target_group_scope_tree_.set(Some(tree.clone()));
        tree
    }
    // cpp: style_engine.cc:1009-1015
    pub fn SetRuleUsageTracker(&self, tracker: Option<Rc<B::StyleRuleUsageTracker>>) {
        self.tracker_.set(tracker);
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverSetRuleUsageTracker(&resolver, self.tracker_.get());
        }
    }
    // cpp: style_engine.cc:1017-1023
    pub fn ComputeFont(
        &self,
        element: &B::Element,
        style: &B::ComputedStyle,
        properties: &B::CSSPropertyValueSet,
    ) -> Option<Rc<B::Font>> {
        self.UpdateActiveStyle();
        B::ResolverComputeFont(&self.GetStyleResolver(), element, style, properties)
    }
    // cpp: style_engine.cc:1042-1056
    fn ClearResolvers(&self) {
        B::ClearScopedStyleResolver(&B::DocumentTreeScope(&self.document_));
        for scope in self.GetActiveTreeScopes() {
            B::ClearScopedStyleResolver(&scope);
        }
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverDispose(&resolver);
            self.resolver_.set(None);
        }
    }
    // cpp: style_engine.cc:1058-1081
    pub fn DidDetach(&self) {
        self.ClearResolvers();
        if let Some(global) = self.global_rule_set_.get() {
            B::GlobalRuleSetDispose(&global);
        }
        self.global_rule_set_.set(None);
        {
            let mut state = self.sheet_state.borrow_mut();
            state.dirty_tree_scopes.clear();
            state.active_tree_scopes.clear();
        }
        self.viewport_resolver_.set(None);
        *self.media_query_evaluator_.borrow_mut() = None;
        B::ClearStyleInvalidationRoot(&mut self.style_invalidation_root_.borrow_mut());
        B::ClearStyleRecalcRoot(&mut self.style_recalc_root_.borrow_mut());
        B::ClearLayoutTreeRebuildRoot(&mut self.layout_tree_rebuild_root_.borrow_mut());
        if let Some(selector) = self.font_selector_.get() {
            B::FontFaceCacheClearAll(&selector);
        }
        self.font_selector_.set(None);
        if let Some(variables) = self.environment_variables_.get() {
            B::EnvironmentVariablesDetachFromParent(&variables);
        }
        self.environment_variables_.set(None);
        self.style_containment_scope_tree_.set(None);
        self.inspector_style_sheet_list_.borrow_mut().clear();
    }
    // cpp: style_engine.h:512-512
    pub fn GetFontSelector(&self) -> Option<Rc<B::CSSFontSelector>> {
        self.font_selector_.get()
    }
    // cpp: style_engine.cc:1106-1119
    pub fn UpdateGenericFontFamilySettings(&self) {
        debug_assert!(B::DocumentIsActive(&self.document_));
        let Some(selector) = self.font_selector_.get() else {
            return;
        };
        B::FontSelectorUpdateGenericFontFamilySettings(&selector, &self.document_);
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverInvalidateMatchedPropertiesCache(&resolver);
        }
    }
    // cpp: style_engine.cc:1121-1134
    pub fn RemoveFontFaceRules(&self, rules: &[Rc<B::StyleRuleFontFace>]) {
        let Some(selector) = self.font_selector_.get() else {
            return;
        };
        for rule in rules {
            B::FontFaceCacheRemove(&selector, rule);
        }
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverInvalidateMatchedPropertiesCache(&resolver);
        }
    }
    // cpp: style_engine.cc:1165-1203
    pub fn CreateSheet(
        &self,
        element: &Rc<B::Element>,
        text: &String,
        position: B::TextPosition,
        kind: PendingSheetType,
        render_blocking: RenderBlockingBehavior,
    ) -> Rc<B::CSSStyleSheet> {
        if kind != PendingSheetType::kNonBlocking {
            self.AddPendingBlockingSheet(B::ElementAsNode(element), kind);
        }
        let context = B::InlineParserContext(
            &self.document_,
            &B::NullSheetURL(),
            B::DocumentEncoding(&self.document_),
        );
        let sheet = if let Some(contents) = self.FindStyleSheetContents(text, Some(&context)) {
            B::ContentsSetRenderBlocking(&contents, render_blocking);
            B::CreateInlineSheet(contents, element, position)
        } else {
            let contents = B::NewStyleSheetContents(context);
            let sheet = B::CreateInlineSheet(contents.clone(), element, position);
            B::ContentsSetRenderBlocking(&contents, render_blocking);
            B::ContentsParseString(&contents, text);
            if B::ContentsCacheableForStyleElement(&contents) {
                self.AddStyleSheetContents(text, Some(&contents));
            }
            sheet
        };
        if !B::ElementIsInShadowTree(element) {
            let title = B::ElementTitle(element);
            if !title.empty() {
                B::SheetSetTitle(&sheet, title.clone());
                self.SetPreferredStylesheetSetNameIfNotSet(&title);
            }
        }
        sheet
    }
    // cpp: style_engine.cc:1263-1271
    pub fn CollectUserStyleFeaturesTo(&self, features: &mut B::RuleFeatureSet) {
        let sheets = self.sheet_state.borrow().active_user_style_sheets.clone();
        for sheet in sheets {
            B::FeaturesMediaQueryResultFlags(features)
                .Add(&B::SheetMediaQueryResultFlags(&sheet.style_sheet));
            let contents = B::SheetContents(&sheet.style_sheet);
            let rules = B::ContentsRuleSet(&contents);
            B::FeaturesMerge(features, &B::RuleSetFeatures(&rules));
        }
    }
    // cpp: style_engine.cc:1273-1286
    pub fn CollectScopedStyleFeaturesTo(&self, features: &mut B::RuleFeatureSet) {
        let mut visited = HashMap::new();
        if let Some(resolver) =
            B::TreeScopeScopedStyleResolver(&B::DocumentTreeScope(&self.document_))
        {
            B::ScopedResolverCollectFeatures(&resolver, features, &mut visited);
        }
        for scope in self.GetActiveTreeScopes() {
            if let Some(resolver) = B::TreeScopeScopedStyleResolver(&scope) {
                B::ScopedResolverCollectFeatures(&resolver, features, &mut visited);
            }
        }
    }
    // cpp: style_engine.cc:1288-1295
    pub fn MarkViewportUnitDirty(&self, flag: ViewportUnitFlag) {
        let flag = flag as u32;
        if self.viewport_unit_dirty_flags_.get() & flag != 0 {
            return;
        }
        self.viewport_unit_dirty_flags_
            .set(self.viewport_unit_dirty_flags_.get() | flag);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:1318-1340
    pub fn InvalidateViewportUnitStylesIfNeeded(&self) {
        if self.viewport_unit_dirty_flags_.get() == 0 {
            return;
        }
        let flags = self.viewport_unit_dirty_flags_.replace(0);
        let initial_data = self.initial_data_.borrow().clone();
        if initial_data
            .as_ref()
            .is_some_and(|data| B::InitialDataViewportUnitFlags(data) & flags != 0)
        {
            self.InvalidateInitialData();
            self.MarkAllElementsForStyleRecalc(StyleChangeReason::ViewportUnits);
            return;
        }
        self.MarkElementsForRecalc(
            &B::DocumentTreeScope(&self.document_),
            StyleChangeReason::ViewportUnits,
            &|style| B::ComputedStyleViewportUnitFlags(style) & flags != 0,
        );
    }
    // cpp: style_engine.cc:1342-1361
    pub fn InvalidateStyleAndLayoutForFontUpdates(&self) {
        if !self.fonts_need_update_.get() {
            return;
        }
        self.fonts_need_update_.set(false);
        if let Some(root) = B::DocumentElement(&self.document_) {
            B::ElementMarkSubtreeNeedsStyleRecalcForFontUpdates(&root);
        }
        if let Some(view) = B::LayoutViewIfPresent(&self.document_) {
            B::InvalidateSubtreeLayoutForFontUpdates(&view);
        }
    }
    // cpp: style_engine.cc:1373-1386
    pub fn FontsNeedUpdate(
        &self,
        _selector: Option<&B::FontSelector>,
        _reason: B::FontInvalidationReason,
    ) {
        if !B::DocumentIsActive(&self.document_) {
            return;
        }
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverInvalidateMatchedPropertiesCache(&resolver);
        }
        self.MarkViewportStyleDirty();
        self.MarkFontsNeedUpdate();
        B::FontsUpdated(&self.document_);
    }
    // cpp: style_engine.cc:1404-1437
    pub fn ShouldSkipInvalidationFor(&self, element: &B::Element) -> bool {
        if !B::ElementInActiveDocument(element) {
            return true;
        }
        assert!(
            self.global_rule_set_.get().is_some(),
            "global rule set must exist for active documents"
        );
        B::DocumentInStyleRecalc(&self.document_)
    }
    // cpp: style_engine.cc:1439-1451
    pub fn IsSubtreeAndSiblingsStyleDirty(&self, element: &B::Element) -> bool {
        if B::DocumentHasSubtreeStyleChange(&self.document_) {
            return true;
        }
        let Some(root) = B::DocumentElement(&self.document_) else {
            return true;
        };
        if B::ElementHasSubtreeStyleChange(&root) {
            return true;
        }
        let Some(parent) = B::ElementParentNode(element) else {
            return true;
        };
        B::ContainerHasSubtreeStyleChange(&parent)
    }
    // cpp: style_engine.cc:1455-1459
    pub fn PossiblyAffectingHasState(element: &B::Element) -> bool {
        B::AncestorsOrAncestorSiblingsAffectedByHas(element)
            || B::SiblingsAffectedByHasFlags(element) != 0
            || B::AffectedByLogicalCombinationsInHas(element)
    }
    // cpp: style_engine.cc:1461-1472
    pub fn InsertionOrRemovalPossiblyAffectHasStateOfAncestorsOrAncestorSiblings(
        parent: Option<&B::Element>,
    ) -> bool {
        parent.is_some_and(|parent| {
            B::AncestorsOrAncestorSiblingsAffectedByHas(parent)
                || B::HasSiblingsAffectedByHasForSiblingDescendantRelationship(parent)
        })
    }
    // cpp: style_engine.cc:1474-1480
    pub fn InsertionOrRemovalPossiblyAffectHasStateOfPreviousSiblings(
        previous: Option<&B::Element>,
    ) -> bool {
        previous.is_some_and(|previous| B::SiblingsAffectedByHasFlags(previous) != 0)
    }
    // cpp: style_engine.cc:1482-1490
    pub fn SelfOrPreviousSibling(node: Option<&B::Node>) -> Option<Rc<B::Element>> {
        let node = node?;
        B::NodeAsElement(node).or_else(|| B::PreviousElementSibling(node))
    }

    // cpp: style_engine.cc:2823-2865
    fn InvalidateForRuleSetChanges(
        &self,
        scope: &Rc<B::TreeScope>,
        changed: &ChangedRuleSets<B::RuleSet>,
        flags: u32,
        invalidation: InvalidationScope,
    ) {
        if B::HasPendingForcedStyleRecalc(&self.document_)
            || B::DocumentElement(&self.document_).is_none()
            || changed.IsEmpty()
        {
            return;
        }
        let root = B::InvalidationRootForTreeScope(scope);
        if B::ElementHasSubtreeStyleChange(&root) {
            return;
        }
        let mut filter = B::NewSelectorFilter();
        B::SelectorFilterPushAllParentsOf(&mut filter, scope);
        let frame_element = B::TreeScopeShadowHost(scope)
            .unwrap_or_else(|| B::DocumentElement(&self.document_).expect("document element"));
        let frame = B::NewStyleScopeFrame(&frame_element, None);
        let _nth_index_cache = B::NewNthIndexCache(&self.document_);
        self.ApplyRuleSetInvalidationForTreeScope(
            scope,
            &B::TreeScopeRootNode(scope),
            &mut filter,
            &frame,
            changed,
            flags,
            invalidation,
        );
    }
    // cpp: style_engine.cc:2190-2204
    fn AnyRuleCausesInvalidation(
        scope: &B::TreeScope,
        group: &B::RuleSetGroup,
        collector: &mut B::ElementRuleCollector,
        is_shadow_host: bool,
    ) -> bool {
        if B::CollectorCheckAnyRuleMatches(collector, group, scope)
            || FlagsCauseInvalidation(B::CollectorMatchFlags(collector))
        {
            return true;
        }
        if is_shadow_host
            && (B::CollectorCheckAnyShadowHostRuleMatches(collector, group, scope)
                || FlagsCauseInvalidation(B::CollectorMatchFlags(collector)))
        {
            return true;
        }
        false
    }
    // cpp: style_engine.cc:2208-2261
    fn ApplyRuleSetInvalidationForElement(
        &self,
        scope: &B::TreeScope,
        element: &B::Element,
        filter: &B::SelectorFilter,
        context: &B::StyleRecalcContext,
        changed: &ChangedRuleSets<B::RuleSet>,
        flags: u32,
        is_shadow_host: bool,
    ) {
        if flags & FUNCTION_RULES != 0
            && B::ElementComputedStyle(element)
                .as_ref()
                .is_some_and(|style| B::AffectedByCSSFunction(style))
        {
            B::ElementSetNeedsStyleRecalc(
                element,
                StyleChangeType::kLocalStyleChange,
                StyleChangeReason::FunctionRuleChange,
            );
            return;
        }
        let mut collector = B::NewElementRuleCollector(element, context, filter);
        let mut index = 0;
        let mut group = B::NewRuleSetGroup(index);
        index += 1;
        let mut matched = false;
        for rules in changed.Values() {
            B::RuleSetGroupAdd(&mut group, rules.clone());
            if B::RuleSetGroupIsFull(&group) {
                if Self::AnyRuleCausesInvalidation(scope, &group, &mut collector, is_shadow_host) {
                    matched = true;
                    break;
                }
                group = B::NewRuleSetGroup(index);
                index += 1;
            }
        }
        if !B::RuleSetGroupIsEmpty(&group) && !matched {
            matched =
                Self::AnyRuleCausesInvalidation(scope, &group, &mut collector, is_shadow_host);
        }
        if matched {
            B::ElementSetNeedsStyleRecalc(
                element,
                StyleChangeType::kLocalStyleChange,
                StyleChangeReason::StyleRuleChange,
            );
        }
    }
    // cpp: style_engine.cc:2456-2464
    fn InvalidateSlottedElements(&self, slot: &B::Element, reason: StyleChangeReason) {
        for element in B::FlattenedAssignedElements(slot) {
            B::ElementSetNeedsStyleRecalc(&element, StyleChangeType::kLocalStyleChange, reason);
        }
    }
    // cpp: style_engine.cc:2502-2578
    pub fn ApplyRuleSetInvalidationForTreeScope(
        &self,
        scope: &Rc<B::TreeScope>,
        node: &B::ContainerNode,
        filter: &mut B::SelectorFilter,
        frame: &B::StyleScopeFrame,
        changed: &ChangedRuleSets<B::RuleSet>,
        flags: u32,
        mut invalidation: InvalidationScope,
    ) {
        let mut invalidate_slotted = false;
        let mut invalidate_part = false;
        if let Some(host) = B::ContainerNodeShadowHost(node) {
            B::SelectorFilterPopParent(filter, &host);
            let context = B::RecalcContextFromAncestors(&host, frame);
            self.ApplyRuleSetInvalidationForElement(
                scope, &host, filter, &context, changed, flags, true,
            );
            B::SelectorFilterPushParent(filter, &host);
            if B::ElementHasSubtreeStyleChange(&host) || B::ElementComputedStyle(&host).is_none() {
                return;
            }
            for rules in changed.Values() {
                if B::RuleSetHasSlottedRules(&rules) {
                    invalidate_slotted = true;
                    break;
                }
                if B::RuleSetHasPartPseudoRules(&rules) {
                    invalidate_part = true;
                    break;
                }
            }
        }
        if invalidation != InvalidationScope::kInvalidateAllScopes {
            for rules in changed.Values() {
                if B::RuleSetHasUAShadowPseudoElementRules(&rules)
                    || B::RuleSetHasPartPseudoRules(&rules)
                {
                    invalidation = InvalidationScope::kInvalidateAllScopes;
                    break;
                }
            }
        }
        for child in B::ContainerElementChildren(node) {
            let context = B::RecalcContextFromAncestors(&child, frame);
            self.ApplyRuleSetInvalidationForSubtree(
                scope,
                &child,
                filter,
                &context,
                frame,
                changed,
                flags,
                invalidation,
                invalidate_slotted,
                invalidate_part,
            );
        }
    }
    // cpp: style_engine.cc:2580-2653
    fn ApplyRuleSetInvalidationForSubtree(
        &self,
        scope: &Rc<B::TreeScope>,
        element: &B::Element,
        filter: &mut B::SelectorFilter,
        parent_context: &B::StyleRecalcContext,
        parent_frame: &B::StyleScopeFrame,
        changed: &ChangedRuleSets<B::RuleSet>,
        flags: u32,
        invalidation: InvalidationScope,
        invalidate_slotted: bool,
        invalidate_part: bool,
    ) {
        let frame = B::NewStyleScopeFrame(element, Some(parent_frame));
        if invalidate_part && B::ElementHasPartAttribute(element) {
            B::ElementSetNeedsStyleRecalc(
                element,
                StyleChangeType::kLocalStyleChange,
                StyleChangeReason::StyleRuleChange,
            );
        } else {
            let context = B::RecalcContextFromParent(parent_context, element, &frame);
            self.ApplyRuleSetInvalidationForElement(
                scope, element, filter, &context, changed, flags, false,
            );
        }
        if B::ElementIsHTMLSlot(element) && invalidate_slotted {
            self.InvalidateSlottedElements(element, StyleChangeReason::StyleRuleChange);
        }
        if invalidation == InvalidationScope::kInvalidateAllScopes {
            if let Some(shadow) = B::ElementShadowRoot(element) {
                let mark = B::SelectorFilterSetMark(filter);
                B::SelectorFilterPushParent(filter, element);
                // The Chromium call passes kInvalidateAllScopes as the flags
                // argument and leaves the scope argument at its declared default.
                // Retain that source behavior rather than correcting it here.
                self.ApplyRuleSetInvalidationForTreeScope(
                    scope,
                    &B::TreeScopeRootNode(&shadow),
                    filter,
                    &frame,
                    changed,
                    InvalidationScope::kInvalidateAllScopes as u32,
                    InvalidationScope::kInvalidateCurrentScope,
                );
                B::SelectorFilterPopTo(filter, mark);
            }
        }
        if !B::ElementHasSubtreeStyleChange(element) && B::ElementComputedStyle(element).is_some() {
            let mark = B::SelectorFilterSetMark(filter);
            B::SelectorFilterPushParent(filter, element);
            let context = B::RecalcContextFromParent(parent_context, element, &frame);
            for child in B::ElementChildren(element) {
                self.ApplyRuleSetInvalidationForSubtree(
                    scope,
                    &child,
                    filter,
                    &context,
                    &frame,
                    changed,
                    flags,
                    invalidation,
                    invalidate_slotted,
                    invalidate_part,
                );
            }
            B::SelectorFilterPopTo(filter, mark);
        }
    }

    // cpp: style_engine.cc:167-202
    fn GetRuleSetFlags(rules: &ChangedRuleSets<B::RuleSet>) -> u32 {
        let mut flags = 0;
        for rules in rules.Values() {
            if !B::RuleSetKeyframes(&rules).is_empty() {
                flags |= KEYFRAMES_RULES;
            }
            if !B::RuleSetFontFaces(&rules).is_empty() {
                flags |= FONT_FACE_RULES;
            }
            if !B::RuleSetFontPalettes(&rules).is_empty() {
                flags |= FONT_PALETTE_RULES;
            }
            if B::RuleSetHasFontFeatureValues(&rules) {
                flags |= FONT_FEATURE_RULES;
            }
            if !B::RuleSetProperties(&rules).is_empty() {
                flags |= PROPERTY_RULES;
            }
            if B::RuleSetHasCounterStyles(&rules) {
                flags |= COUNTER_STYLE_RULES;
            }
            if rules.CascadeLayers().is_some() {
                flags |= LAYER_RULES;
            }
            if !B::RuleSetPositionTries(&rules).is_empty() {
                flags |= POSITION_TRY_RULES;
            }
            if !B::RuleSetViewTransitions(&rules).is_empty() {
                flags |= VIEW_TRANSITION_RULES;
            }
            if !B::RuleSetFunctions(&rules).is_empty() {
                flags |= FUNCTION_RULES;
            }
        }
        flags
    }
    // cpp: style_engine.cc:2933-3079
    fn ApplyUserRuleSetChanges(
        &self,
        old: &ActiveStyleSheetVector<B>,
        new: &ActiveStyleSheetVector<B>,
    ) {
        let global = self
            .global_rule_set_
            .get()
            .expect("active style global rules");
        let mut changed = ChangedRuleSets::default();
        let change = CompareActiveStyleSheets(old, new, &[] as &[B::RuleSetDiff], &mut changed);
        if change == ActiveSheetsChange::kNoActiveSheetsChanged {
            return;
        }
        B::GlobalRuleSetMarkDirty(&global);
        let mut flags = Self::GetRuleSetFlags(&changed);
        let mut invalidated_fonts = false;
        let document_scope = B::DocumentTreeScope(&self.document_);
        if flags & LAYER_RULES != 0 {
            *self.user_cascade_layer_map_.borrow_mut() = Some(Rc::new(CascadeLayerMap::new(new)));
            if let Some(resolver) = self.resolver_.get() {
                B::ResolverInvalidateMatchedPropertiesCache(&resolver);
            }
            if change == ActiveSheetsChange::kActiveSheetsChanged {
                flags = RULE_SET_FLAGS_ALL;
            }
        }
        if flags & FONT_FACE_RULES != 0 {
            if let Some(scoped) = B::TreeScopeScopedStyleResolver(&document_scope) {
                B::ScopedResolverSetAppendAllSheets(&scoped);
                self.MarkDocumentDirty();
            } else if self.ClearFontFaceCacheAndAddUserFonts(new) {
                invalidated_fonts = true;
            }
        }
        if flags & KEYFRAMES_RULES != 0 {
            if change == ActiveSheetsChange::kActiveSheetsChanged {
                self.ClearKeyframeRules();
            }
            for sheet in new {
                self.AddUserKeyframeRules(sheet.rule_set.as_ref().expect("user sheet rule set"));
            }
            B::ScopedResolverKeyframesRulesAdded(&document_scope);
        }
        if flags & COUNTER_STYLE_RULES != 0 {
            if change == ActiveSheetsChange::kActiveSheetsChanged {
                let map = self.user_counter_style_map_.borrow().clone();
                if let Some(map) = map {
                    B::CounterStyleMapDispose(&map);
                }
            }
            for sheet in new {
                let rules = sheet.rule_set.as_ref().expect("user sheet rule set");
                if B::RuleSetHasCounterStyles(&rules) {
                    B::CounterStyleMapAddCounterStyles(&self.EnsureUserCounterStyleMap(), rules);
                }
            }
            self.MarkCounterStylesNeedUpdate();
        }
        if flags & (PROPERTY_RULES | FONT_PALETTE_RULES | FONT_FEATURE_RULES) != 0 {
            if flags & PROPERTY_RULES != 0 {
                self.ClearPropertyRules();
                let mut cascade = AtRuleCascadeMap::new();
                self.AddPropertyRulesFromSheets(&mut cascade, new, true);
            }
            if flags & FONT_PALETTE_RULES != 0 {
                self.font_palette_values_rule_map_.borrow_mut().clear();
                self.AddFontPaletteValuesRulesFromSheets(new);
                self.MarkFontsNeedUpdate();
            }
            // Chromium does not collect @font-feature-values from user sheets.
            if let Some(scoped) = B::TreeScopeScopedStyleResolver(&document_scope) {
                B::ScopedResolverSetAppendAllSheets(&scoped);
                self.MarkDocumentDirty();
            }
        }
        if flags & POSITION_TRY_RULES != 0 {
            self.MarkPositionTryStylesDirty(&changed);
        }
        if flags & FUNCTION_RULES != 0 {
            B::ResolverInvalidateMatchedPropertiesCache(
                self.resolver_
                    .get()
                    .as_ref()
                    .expect("user function resolver"),
            );
            self.user_function_rule_map_.borrow_mut().clear();
            let layers = self.user_cascade_layer_map_.borrow().clone();
            for sheet in new {
                for rule in
                    B::RuleSetFunctions(sheet.rule_set.as_ref().expect("user sheet rule set"))
                {
                    let name = B::FunctionName(&rule.value);
                    let mut functions = self.user_function_rule_map_.borrow_mut();
                    let should_replace = functions.get(&name).is_none_or(|old| {
                        CompareLayerOrder(
                            layers.as_deref(),
                            old.layer.as_ref(),
                            rule.layer.as_ref(),
                        ) != Ordering::Greater
                    });
                    if should_replace {
                        functions.insert(name, rule);
                    }
                }
            }
        }
        for rules in changed.Values() {
            B::RuleSetCompactRulesIfNeeded(&rules);
        }
        let mut groups = Vec::new();
        for sheet in new {
            if groups.last().is_none_or(B::RuleSetGroupIsFull) {
                groups.push(B::NewRuleSetGroup(groups.len() as u32));
            }
            B::RuleSetGroupAdd(
                groups.last_mut().unwrap(),
                sheet
                    .rule_set
                    .as_ref()
                    .expect("user sheet rule set")
                    .clone(),
            );
        }
        *self.user_rule_set_groups_.borrow_mut() = groups;
        self.InvalidateForRuleSetChanges(
            &document_scope,
            &changed,
            flags,
            InvalidationScope::kInvalidateAllScopes,
        );
        if invalidated_fonts {
            B::FontFaceGeneralInvalidation(
                self.font_selector_.get().as_ref().expect("font selector"),
            );
        }
    }
    // cpp: style_engine.cc:3081-3236
    pub fn ApplyRuleSetChanges(
        &self,
        scope: &Rc<B::TreeScope>,
        old: &ActiveStyleSheetVector<B>,
        new: &ActiveStyleSheetVector<B>,
        diffs: &[B::RuleSetDiff],
    ) {
        let global = self
            .global_rule_set_
            .get()
            .expect("active style global rules");
        let mut changed = ChangedRuleSets::default();
        let mut change = CompareActiveStyleSheets(old, new, diffs, &mut changed);
        let mut flags = Self::GetRuleSetFlags(&changed);
        let document_scope = self.IsDocumentScope(scope);
        let mut rebuild_fonts = change == ActiveSheetsChange::kActiveSheetsChanged
            && flags & FONT_FACE_RULES != 0
            && document_scope;
        let mut rebuild_properties = false;
        let mut rebuild_palettes = false;
        let scoped = B::TreeScopeScopedStyleResolver(scope);
        if scoped
            .as_ref()
            .is_some_and(|scoped| B::ScopedResolverNeedsAppendAllSheets(scoped))
        {
            rebuild_fonts = true;
            rebuild_properties = true;
            rebuild_palettes = true;
            change = ActiveSheetsChange::kActiveSheetsChanged;
        }
        if change == ActiveSheetsChange::kNoActiveSheetsChanged {
            return;
        }
        B::GlobalRuleSetMarkDirty(&global);
        if flags & KEYFRAMES_RULES != 0 {
            B::ScopedResolverKeyframesRulesAdded(scope);
        }
        if flags & COUNTER_STYLE_RULES != 0 {
            self.MarkCounterStylesNeedUpdate();
        }
        let mut append_start = 0;
        let mut rebuild_layers = flags & LAYER_RULES != 0;
        if let Some(scoped) = &scoped {
            if new.is_empty() {
                rebuild_layers = false;
                self.ResetAuthorStyle(scope);
            } else if change == ActiveSheetsChange::kActiveSheetsAppended {
                append_start = old.len();
            } else {
                rebuild_layers =
                    flags & LAYER_RULES != 0 || B::ScopedResolverCascadeLayerMap(scoped).is_some();
                B::ScopedResolverResetStyle(scoped);
            }
        }
        if rebuild_layers {
            B::ScopedResolverRebuildCascadeLayerMap(&B::EnsureScopedStyleResolver(scope), new);
        }
        if flags & LAYER_RULES != 0 {
            if let Some(resolver) = self.resolver_.get() {
                B::ResolverInvalidateMatchedPropertiesCache(&resolver);
            }
            if change == ActiveSheetsChange::kActiveSheetsChanged {
                flags = RULE_SET_FLAGS_ALL;
                if document_scope {
                    rebuild_fonts = true;
                }
            }
        }
        let users = self.sheet_state.borrow().active_user_style_sheets.clone();
        if (flags & PROPERTY_RULES != 0 || rebuild_properties) && document_scope {
            self.ClearPropertyRules();
            let mut cascade = AtRuleCascadeMap::new();
            self.AddPropertyRulesFromSheets(&mut cascade, &users, true);
            self.AddPropertyRulesFromSheets(&mut cascade, new, false);
        }
        if (flags & FONT_PALETTE_RULES != 0 || rebuild_palettes) && document_scope {
            self.font_palette_values_rule_map_.borrow_mut().clear();
            self.AddFontPaletteValuesRulesFromSheets(&users);
            self.AddFontPaletteValuesRulesFromSheets(new);
        }
        let mut invalidated_fonts = false;
        if document_scope {
            let rebuilt = rebuild_fonts && self.ClearFontFaceCacheAndAddUserFonts(&users);
            invalidated_fonts =
                flags & (FONT_FACE_RULES | FONT_PALETTE_RULES | FONT_FEATURE_RULES) != 0 || rebuilt;
        }
        if flags & POSITION_TRY_RULES != 0 {
            self.MarkPositionTryStylesDirty(&changed);
        }
        if flags & VIEW_TRANSITION_RULES != 0 && document_scope {
            self.AddViewTransitionRules(new);
        }
        if flags & FUNCTION_RULES != 0 {
            if let Some(resolver) = self.resolver_.get() {
                B::ResolverInvalidateMatchedPropertiesCache(&resolver);
            }
        }
        if !new.is_empty() {
            B::ScopedResolverAppendActiveStyleSheets(
                &B::EnsureScopedStyleResolver(scope),
                append_start,
                new,
            );
        }
        self.InvalidateForRuleSetChanges(
            scope,
            &changed,
            flags,
            InvalidationScope::kInvalidateCurrentScope,
        );
        if invalidated_fonts {
            B::FontFaceGeneralInvalidation(
                self.font_selector_.get().as_ref().expect("font selector"),
            );
        }
    }
    // cpp: style_engine.cc:976-991
    pub fn ResetAuthorStyle(&self, scope: &Rc<B::TreeScope>) {
        let Some(scoped) = B::TreeScopeScopedStyleResolver(scope) else {
            return;
        };
        if let Some(global) = self.global_rule_set_.get() {
            B::GlobalRuleSetMarkDirty(&global);
        }
        if self.IsDocumentScope(scope) {
            B::ScopedResolverResetStyle(&scoped);
            return;
        }
        B::ClearScopedStyleResolver(scope);
    }
    // cpp: style_engine.cc:1083-1104
    fn ClearFontFaceCacheAndAddUserFonts(&self, sheets: &ActiveStyleSheetVector<B>) -> bool {
        let mut changed = false;
        if self
            .font_selector_
            .get()
            .as_ref()
            .is_some_and(|font| B::FontFaceCacheClearCSSConnected(font))
        {
            changed = true;
            if let Some(resolver) = self.resolver_.get() {
                B::ResolverInvalidateMatchedPropertiesCache(&resolver);
            }
        }
        for sheet in sheets {
            if self.AddUserFontFaceRules(sheet.rule_set.as_ref().expect("user sheet rule set")) {
                changed = true;
            }
        }
        changed
    }
    // cpp: style_engine.cc:1363-1366
    pub fn MarkFontsNeedUpdate(&self) {
        self.fonts_need_update_.set(true);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:1368-1371
    pub fn MarkCounterStylesNeedUpdate(&self) {
        self.counter_styles_need_update_.set(true);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:786-806
    fn MarkPositionTryStylesDirty(&self, changed: &ChangedRuleSets<B::RuleSet>) {
        for rules in changed.Values() {
            for rule in B::RuleSetPositionTries(&rules) {
                if let Some(rule) = rule.value {
                    self.dirty_position_try_names_
                        .borrow_mut()
                        .insert(B::PositionTryName(&rule));
                }
            }
        }
        self.position_try_styles_dirty_.set(true);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:3495-3497
    fn ClearKeyframeRules(&self) {
        self.keyframes_rule_map_.borrow_mut().clear();
    }
    // cpp: style_engine.cc:3499-3501
    fn ClearPropertyRules(&self) {
        B::RemoveDeclaredProperties(&self.document_);
    }
    // cpp: style_engine.cc:3503-3512
    fn AddPropertyRulesFromSheets(
        &self,
        cascade: &mut AtRuleCascadeMap,
        sheets: &ActiveStyleSheetVector<B>,
        user: bool,
    ) {
        for sheet in sheets {
            if let Some(rules) = &sheet.rule_set {
                self.AddPropertyRules(cascade, rules, user);
            }
        }
    }
    // cpp: style_engine.cc:3514-3521
    fn AddFontPaletteValuesRulesFromSheets(&self, sheets: &ActiveStyleSheetVector<B>) {
        for sheet in sheets {
            if let Some(rules) = &sheet.rule_set {
                self.AddFontPaletteValuesRules(rules);
            }
        }
    }
    // cpp: style_engine.cc:3523-3540
    fn AddUserFontFaceRules(&self, rules: &B::RuleSet) -> bool {
        let Some(selector) = self.font_selector_.get() else {
            return false;
        };
        let font_rules = B::RuleSetFontFaces(&rules);
        for rule in &font_rules {
            if let Some(face) = B::FontFaceCreate(&self.document_, rule, true) {
                B::FontFaceCacheAdd(&selector, &rule.value, face);
            }
        }
        if !font_rules.is_empty() {
            if let Some(resolver) = self.resolver_.get() {
                B::ResolverInvalidateMatchedPropertiesCache(&resolver);
            }
        }
        !font_rules.is_empty()
    }
    // cpp: style_engine.cc:3542-3548
    fn AddUserKeyframeRules(&self, rules: &B::RuleSet) {
        for rule in B::RuleSetKeyframes(&rules) {
            self.AddUserKeyframeStyle(rule);
        }
    }
    // cpp: style_engine.cc:3550-3559
    fn AddUserKeyframeStyle(&self, rule: CascadeLayered<Rc<B::StyleRuleKeyframes>>) {
        let name = B::KeyframesName(&rule.value);
        let old = self.keyframes_rule_map_.borrow().get(&name).cloned();
        if old
            .as_ref()
            .is_none_or(|old| self.UserKeyframeStyleShouldOverride(&rule, old))
        {
            self.keyframes_rule_map_.borrow_mut().insert(name, rule);
        }
    }
    // cpp: style_engine.cc:3561-3570
    fn UserKeyframeStyleShouldOverride(
        &self,
        new: &CascadeLayered<Rc<B::StyleRuleKeyframes>>,
        old: &CascadeLayered<Rc<B::StyleRuleKeyframes>>,
    ) -> bool {
        if B::KeyframesVendorPrefixed(&new.value) != B::KeyframesVendorPrefixed(&old.value) {
            return B::KeyframesVendorPrefixed(&old.value);
        }
        CompareLayerOrder(
            self.user_cascade_layer_map_.borrow().as_deref(),
            old.layer.as_ref(),
            new.layer.as_ref(),
        ) != Ordering::Greater
    }
    // cpp: style_engine.cc:204-223
    fn ConvertFontFamilyToVector(value: &B::CSSValue) -> Vec<AtomicString> {
        let Some(items) = B::CSSValueListItems(value) else {
            return Vec::new();
        };
        let mut families = Vec::with_capacity(items.len());
        for item in items {
            let Some(family) = B::CSSFontFamilyValue(&item) else {
                return Vec::new();
            };
            families.push(family);
        }
        families
    }
    // cpp: style_engine.cc:3609-3619
    fn AddFontPaletteValuesRules(&self, rules: &B::RuleSet) {
        for rule in B::RuleSetFontPalettes(&rules) {
            for family in Self::ConvertFontFamilyToVector(&B::FontPaletteFamilyValue(&rule)) {
                let family = B::FoldCase(&String::from_utf16(
                    family.utf16_units().unwrap_or_default(),
                ));
                self.font_palette_values_rule_map_
                    .borrow_mut()
                    .insert((B::FontPaletteName(&rule), family), rule.clone());
            }
        }
    }
    // cpp: style_engine.cc:2873-2931
    fn AtRuleLayerOrder(&self, user: bool, layer: Option<&CascadeLayer>) -> u16 {
        let Some(layer) = layer else {
            return CascadeLayerMap::kImplicitOuterLayerOrder;
        };
        let map = if user {
            self.user_cascade_layer_map_.borrow().clone()
        } else {
            B::TreeScopeScopedStyleResolver(&B::DocumentTreeScope(&self.document_))
                .and_then(|scoped| B::ScopedResolverCascadeLayerMap(&scoped))
        };
        map.as_ref()
            .map_or(CascadeLayerMap::kImplicitOuterLayerOrder, |map| {
                map.GetLayerOrder(layer)
            })
    }
    // cpp: style_engine.cc:3621-3643
    fn AddPropertyRules(&self, cascade: &mut AtRuleCascadeMap, rules: &B::RuleSet, user: bool) {
        for rule in B::RuleSetProperties(&rules) {
            let name = B::PropertyName(&rule.value);
            let Some(registration) =
                B::MaybeCreateDeclaredProperty(&self.document_, &name, &rule.value)
            else {
                continue;
            };
            let priority = AtRulePriority {
                is_user_style: user,
                layer_order: self.AtRuleLayerOrder(user, rule.layer.as_ref()),
            };
            if !cascade.AddAndCascade(name.clone(), priority) {
                continue;
            }
            B::DeclareProperty(&self.document_, name, &registration);
            self.PropertyRegistryChanged();
        }
    }
    // cpp: style_engine.cc:3391-3399
    pub fn PropertyRegistryChanged(&self) {
        self.MarkAllElementsForStyleRecalc(StyleChangeReason::PropertyRegistration);
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverInvalidateMatchedPropertiesCache(&resolver);
        }
        self.InvalidateInitialData();
    }
    // cpp: style_engine.cc:2867-2869
    fn InvalidateInitialData(&self) {
        *self.initial_data_.borrow_mut() = None;
    }
    // cpp: style_engine.cc:4757-4763
    fn EnsureUserCounterStyleMap(&self) -> Rc<B::CounterStyleMap> {
        self.user_counter_style_map_
            .borrow_mut()
            .get_or_insert_with(|| B::CreateUserCounterStyleMap(&self.document_))
            .clone()
    }
    // cpp: style_engine.cc:3572-3607
    fn AddViewTransitionRules(&self, sheets: &ActiveStyleSheetVector<B>) {
        *self.view_transition_rule_.borrow_mut() = None;
        for sheet in sheets {
            let Some(rules) = &sheet.rule_set else {
                continue;
            };
            let transitions = B::RuleSetViewTransitions(&rules);
            if transitions.is_empty() {
                continue;
            }
            let layers = B::TreeScopeScopedStyleResolver(&B::DocumentTreeScope(&self.document_))
                .and_then(|scoped| B::ScopedResolverCascadeLayerMap(&scoped));
            for rule in transitions {
                let target = if B::ViewTransitionNavigation(&rule.value)
                    == ViewTransitionNavigation::kPreview
                {
                    assert!(B::TwoPhaseViewTransitionEnabled());
                    &self.view_transition_preview_rule_
                } else {
                    &self.view_transition_rule_
                };
                let mut target = target.borrow_mut();
                if target.as_ref().is_none_or(|old| {
                    CompareLayerOrder(layers.as_deref(), old.layer.as_ref(), rule.layer.as_ref())
                        != Ordering::Greater
                }) {
                    *target = Some(rule);
                }
            }
        }
        self.UpdateViewTransitionOptIn();
    }
    // cpp: style_engine.cc:2739-2767
    pub fn UpdateViewTransitionOptIn(&self) {
        let mut enabled = false;
        let mut types = Vec::new();
        let preview = self
            .view_transition_preview_rule_
            .borrow()
            .as_ref()
            .map(|rule| B::ViewTransitionTypes(&rule.value));
        if let Some(rule) = self.view_transition_rule_.borrow().as_ref() {
            match B::ViewTransitionNavigation(&rule.value) {
                ViewTransitionNavigation::kAuto => {
                    enabled = true;
                    types = B::ViewTransitionTypes(&rule.value);
                }
                ViewTransitionNavigation::kNone | ViewTransitionNavigation::kUnspecified => {}
                ViewTransitionNavigation::kPreview => unreachable!("preview rules are separate"),
            }
        }
        B::OnViewTransitionsStyleUpdated(&self.document_, enabled, types, preview);
    }

    // cpp: style_engine.cc:251-286
    pub fn new(document: Rc<B::Document>) -> Rc<Self> {
        let scope_tree = B::NewStyleContainmentScopeTree();
        let document_collection = B::NewStyleSheetCollection(&B::DocumentTreeScope(&document));
        let engine = Rc::new_cyclic(|weak| {
            let mut resolver = None;
            let mut global = None;
            let mut font_selector = None;
            let mut viewport = None;
            let mut owner_color = ColorScheme::kLight;
            let mut owner_preferred = PreferredColorScheme::kLight;
            if B::DocumentHasFrame(&document) {
                resolver = Some(B::NewStyleResolver(&document));
                global = Some(B::NewGlobalRuleSet());
                let font = Self::CreateCSSFontSelectorFor(&document);
                B::RegisterFontInvalidationCallbacks(&font, weak.clone());
                font_selector = Some(font);
                if let Some((color, preferred)) = B::FrameOwnerColorSchemes(&document) {
                    owner_color = color;
                    owner_preferred = preferred;
                }
                if B::DocumentIsMainFrame(&document) && B::VisualViewportIsActive(&document) {
                    viewport = Some(B::NewViewportStyleResolver(&document));
                }
            }
            Self {
                is_env_dirty_: Cell::new(false),
                has_complex_safe_area_constraints_: Cell::new(false),
                vision_deficiency_: Cell::new(VisionDeficiency::kNoVisionDeficiency),
                vision_deficiency_filter_: Retained::new(None),
                style_image_cache_: B::NewStyleImageCache(),
                fill_or_clip_path_uri_value_cache_: RefCell::new(HashMap::new()),
                try_value_flips_: B::NewTryValueFlips(),
                anchored_element_dirty_set_: RefCell::new(HashMap::new()),
                element_keeps_random_caching_key_alive_: RefCell::new(HashMap::new()),
                random_base_value_cache_: RefCell::new(Vec::new()),
                element_shared_random_base_value_cache_: RefCell::new(HashMap::new()),
                document_: document.clone(),
                pending_invalidations_: B::NewPendingInvalidations(),
                scroll_target_group_scope_tree_: Retained::new(None),
                tracker_: Retained::new(None),
                environment_variables_: Retained::new(None),
                style_invalidation_root_: RefCell::new(B::NewStyleInvalidationRoot()),
                style_recalc_root_: RefCell::new(B::NewStyleRecalcRoot()),
                layout_tree_rebuild_root_: RefCell::new(B::NewLayoutTreeRebuildRoot()),
                viewport_unit_dirty_flags_: Cell::new(0),
                needs_to_update_complex_safe_area_constraints_: Cell::new(false),
                style_containment_scope_tree_: Retained::new(Some(scope_tree)),
                document_style_sheet_collection_: document_collection,
                style_sheet_collection_map_: RefCell::new(HashMap::new()),
                sheet_state: RefCell::new(ActiveSheetState {
                    document_scope_dirty: true,
                    tree_scopes_removed: false,
                    user_style_dirty: false,
                    dirty_tree_scopes: HashMap::new(),
                    active_tree_scopes: HashMap::new(),
                    active_user_style_sheets: Vec::new(),
                }),
                pending_parser_blocking_stylesheets_: Cell::new(0),
                pending_script_blocking_stylesheets_: Cell::new(0),
                inspector_style_sheet_list_: RefCell::new(Vec::new()),
                injected_author_style_sheets_: RefCell::new(Vec::new()),
                injected_user_style_sheets_: RefCell::new(Vec::new()),
                text_tracks_: RefCell::new(HashMap::new()),
                vtt_originating_element_: RefCell::new(None),
                text_to_sheet_cache_: RefCell::new(HashMap::new()),
                navigation_locations_: RefCell::new(HashMap::new()),
                functional_media_query_results_: RefCell::new(HashMap::new()),
                functional_navigation_query_results_: RefCell::new(HashMap::new()),
                functional_media_query_result_flags_: RefCell::new(MediaQueryResultFlags::default()),
                media_query_evaluator_: RefCell::new(None),
                resolver_: Retained::new(resolver),
                global_rule_set_: Retained::new(global),
                viewport_resolver_: Retained::new(viewport),
                font_selector_: Retained::new(font_selector),
                owner_color_scheme_: Cell::new(owner_color),
                owner_preferred_color_scheme_: Cell::new(owner_preferred),
                preferred_color_scheme: Cell::new(PreferredColorScheme::kLight),
                preferred_contrast_: Cell::new(PreferredContrast::kNoPreference),
                forced_colors_: Cell::new(ForcedColors::kNone),
                force_dark_mode_enabled_: Cell::new(false),
                page_color_schemes_: Cell::new(ColorSchemeFlag::kNormal as u8),
                color_scheme_background_: Cell::new(Color::default()),
                forced_background_color_: Cell::new(Color::default()),
                viewport_size_: RefCell::new(None),
                parent_for_detached_subtree_: RefCell::new(None),
                preferred_stylesheet_set_name_: RefCell::new(String::default()),
                counters_changed_: Cell::new(false),
                uses_root_relative_units_: Cell::new(false),
                uses_glyph_relative_units_: Cell::new(false),
                uses_line_height_units_: Cell::new(false),
                uses_tree_counting_functions_: Cell::new(false),
                style_affected_by_layout_: Cell::new(false),
                skipped_container_recalc_: Cell::new(0),
                in_layout_tree_rebuild_: Cell::new(false),
                in_container_query_style_recalc_: Cell::new(false),
                in_position_try_style_recalc_: Cell::new(false),
                in_scroll_markers_attachment_: Cell::new(false),
                in_dom_removal_: Cell::new(false),
                in_detach_scope_: Cell::new(false),
                in_apply_animation_update_: Cell::new(false),
                in_ensure_computed_style_: Cell::new(false),
                viewport_style_dirty_: Cell::new(false),
                fonts_need_update_: Cell::new(false),
                counter_styles_need_update_: Cell::new(false),
                position_try_styles_dirty_: Cell::new(false),
                needs_style_update_on_navigation_: Cell::new(false),
                allow_mark_style_dirty_from_recalc_: Cell::new(false),
                allow_mark_for_reattach_from_rebuild_layout_tree_: Cell::new(false),
                allow_skip_style_recalc_: Cell::new(false),
                style_for_element_count_: Cell::new(0),
                dirty_position_try_names_: RefCell::new(HashSet::new()),
                user_rule_set_groups_: RefCell::new(Vec::new()),
                keyframes_rule_map_: RefCell::new(HashMap::new()),
                font_palette_values_rule_map_: RefCell::new(HashMap::new()),
                user_counter_style_map_: RefCell::new(None),
                user_cascade_layer_map_: RefCell::new(None),
                user_function_rule_map_: RefCell::new(HashMap::new()),
                initial_data_: RefCell::new(None),
                view_transition_rule_: RefCell::new(None),
                view_transition_preview_rule_: RefCell::new(None),
            }
        });
        engine.UpdateColorScheme();
        engine.UpdateViewportSize();
        engine
    }
    // cpp: style_engine.cc:144-150
    fn CreateCSSFontSelectorFor(document: &Rc<B::Document>) -> Rc<B::CSSFontSelector> {
        debug_assert!(B::DocumentHasFrame(document));
        if B::FrameHasPagePopupOwner(document) {
            B::NewPopupCSSFontSelector(document)
        } else {
            B::NewCSSFontSelector(document)
        }
    }
    // cpp: style_engine.cc:4431-4437
    pub fn SupportsDarkColorScheme(&self) -> bool {
        let flags = self.page_color_schemes_.get();
        flags & ColorSchemeFlag::kDark as u8 != 0
            && (flags & ColorSchemeFlag::kLight as u8 == 0
                || self.preferred_color_scheme.get() == PreferredColorScheme::kDark)
    }
    // cpp: style_engine.cc:4439-4519
    pub fn UpdateColorScheme(&self) {
        let Some(settings) = B::DocumentSettings(&self.document_) else {
            return;
        };
        let old_forced = self
            .forced_colors_
            .replace(if B::SettingsInForcedColors(&settings) {
                ForcedColors::kActive
            } else {
                ForcedColors::kNone
            });
        let old_preferred =
            self.preferred_color_scheme
                .replace(if B::DocumentIsMainFrame(&self.document_) {
                    B::SettingsPreferredColorScheme(&settings)
                } else {
                    self.owner_preferred_color_scheme_.get()
                });
        let old_contrast = self
            .preferred_contrast_
            .replace(B::SettingsPreferredContrast(&settings));
        let old_force_dark = self
            .force_dark_mode_enabled_
            .replace(B::SettingsForceDarkModeEnabled(&settings));
        if let Some(forced) = B::MediaFeatureForcedColors(&self.document_) {
            self.forced_colors_.set(forced);
        }
        let color_override = B::MediaFeaturePreferredColorScheme(&self.document_);
        let contrast_override = B::MediaFeaturePreferredContrast(&self.document_);
        if let Some(preferred) =
            color_override.or_else(|| B::PreferencePreferredColorScheme(&self.document_))
        {
            self.preferred_color_scheme.set(preferred);
        }
        if let Some(contrast) =
            contrast_override.or_else(|| B::PreferencePreferredContrast(&self.document_))
        {
            self.preferred_contrast_.set(contrast);
        }
        if B::DocumentPrinting(&self.document_) {
            self.preferred_color_scheme
                .set(PreferredColorScheme::kLight);
            self.preferred_contrast_
                .set(PreferredContrast::kNoPreference);
            self.force_dark_mode_enabled_.set(false);
        }
        if old_forced != self.forced_colors_.get()
            || old_preferred != self.preferred_color_scheme.get()
            || old_contrast != self.preferred_contrast_.get()
            || old_force_dark != self.force_dark_mode_enabled_.get()
        {
            self.PlatformColorsChanged();
        }
        self.UpdateColorSchemeMetrics();
    }
    // cpp: style_engine.cc:4521-4550
    fn UpdateColorSchemeMetrics(&self) {
        let settings = B::DocumentSettings(&self.document_).expect("color-scheme settings");
        if B::SettingsForceDarkModeEnabled(&settings) {
            B::CountColorSchemeFeature(&self.document_, ColorSchemeFeature::kForcedDarkMode);
        }
        if self.preferred_color_scheme.get() == PreferredColorScheme::kDark {
            B::CountColorSchemeFeature(
                &self.document_,
                ColorSchemeFeature::kPreferredColorSchemeDark,
            );
        }
        if B::SettingsPreferredColorScheme(&settings) == PreferredColorScheme::kDark {
            B::CountColorSchemeFeature(
                &self.document_,
                ColorSchemeFeature::kPreferredColorSchemeDarkSetting,
            );
        }
        if self.page_color_schemes_.get() & ColorSchemeFlag::kDark as u8 != 0 {
            B::CountColorSchemeFeature(
                &self.document_,
                ColorSchemeFeature::kColorSchemeDarkSupportedOnRoot,
            );
        }
    }
    // cpp: style_engine.cc:1388-1402
    pub fn PlatformColorsChanged(&self) {
        self.UpdateForcedBackgroundColor();
        self.UpdateColorSchemeBackground(true);
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverInvalidateMatchedPropertiesCache(&resolver);
        }
        self.MarkAllElementsForStyleRecalc(StyleChangeReason::PlatformColorChange);
        if let Some(view) = B::LayoutViewIfPresent(&self.document_) {
            B::InvalidatePaintForViewAndDescendants(&view);
        }
    }
    // cpp: style_engine.cc:4673-4681
    fn UpdateForcedBackgroundColor(&self) {
        let scheme = ColorScheme::kLight;
        let provider = B::ColorProviderForPainting(
            &self.document_,
            scheme,
            self.forced_colors_.get() != ForcedColors::kNone,
        );
        self.forced_background_color_.set(B::ThemeSystemCanvasColor(
            scheme,
            &provider,
            B::DocumentInWebAppScope(&self.document_)
                && B::DocumentIsInitialProfile(&self.document_),
        ));
    }
    // cpp: style_engine.cc:4683-4705
    fn AdjustAboutBlankColorScheme(&self, root: ColorScheme) -> ColorScheme {
        let likely_user_initiated = B::DocumentIsMainFrame(&self.document_)
            && B::DocumentURLIsAboutBlank(&self.document_)
            && !B::PageOpenedByDOM(&self.document_);
        if self.preferred_color_scheme.get() == PreferredColorScheme::kDark && likely_user_initiated
        {
            ColorScheme::kDark
        } else {
            root
        }
    }
    // cpp: style_engine.cc:4589-4643
    pub fn UpdateColorSchemeBackground(&self, changed: bool) {
        if !B::DocumentHasView(&self.document_) {
            return;
        }
        let mut use_background = UseColorAdjustBackground::kNo;
        if self.forced_colors_.get() != ForcedColors::kNone {
            if B::DocumentIsMainFrame(&self.document_) {
                use_background = UseColorAdjustBackground::kIfBaseNotTransparent;
            }
        } else {
            let mut root_scheme = ColorScheme::kLight;
            if let Some(root) = B::DocumentElement(&self.document_) {
                if let Some(style) = B::ElementComputedStyle(&root) {
                    root_scheme = B::UsedColorScheme(&style);
                    if B::AboutBlankDarkModeOnUserActionEnabled() {
                        root_scheme = self.AdjustAboutBlankColorScheme(root_scheme);
                    }
                } else if self.SupportsDarkColorScheme() {
                    root_scheme = ColorScheme::kDark;
                }
            }
            self.color_scheme_background_
                .set(if root_scheme == ColorScheme::kLight {
                    Color::kWhite
                } else {
                    Color::FromRGB(0x12, 0x12, 0x12)
                });
            if B::DocumentIsMainFrame(&self.document_) {
                if root_scheme == ColorScheme::kDark {
                    use_background = UseColorAdjustBackground::kIfBaseNotTransparent;
                }
            } else if root_scheme != self.owner_color_scheme_.get()
                && !B::FrameLoaderIsOnInitialEmptyDocument(&self.document_)
            {
                use_background = UseColorAdjustBackground::kYes;
            }
        }
        B::ViewSetUseColorAdjustBackground(&self.document_, use_background, changed);
    }
    // cpp: style_engine.cc:4938-4941
    pub fn UpdateViewportSize(&self) {
        *self.viewport_size_.borrow_mut() = Some(B::ViewportSizeFromLayoutView(
            B::LayoutViewIfPresent(&self.document_).as_deref(),
        ));
    }
    // cpp: style_engine.cc:648-656
    fn PrepareUpdateActiveStyleSheetsInShadow(
        &self,
        scope: &Rc<B::TreeScope>,
        medium: &B::MediaQueryEvaluator,
    ) {
        debug_assert!(!self.IsDocumentScope(scope));
        let collection = self
            .StyleSheetCollectionFor(scope)
            .expect("dirty scope collection");
        B::CollectionPrepareUpdateActiveStyleSheets(&collection, medium);
    }
    // cpp: style_engine.cc:658-671
    fn UpdateActiveUserStyleSheets(&self) {
        debug_assert!(self.sheet_state.borrow().user_style_dirty);
        let mut new_sheets = Vec::new();
        let empty_mixins = B::EmptyMixinMap();
        let injected = self.injected_user_style_sheets_.borrow().clone();
        for (_, sheet) in injected {
            if let Some(rules) = self.RuleSetForSheet(&sheet, &empty_mixins) {
                new_sheets.push(ActiveStyleSheet::new(sheet, Some(rules)));
            }
        }
        let old = self.sheet_state.borrow().active_user_style_sheets.clone();
        self.ApplyUserRuleSetChanges(&old, &new_sheets);
        self.sheet_state.borrow_mut().active_user_style_sheets = new_sheets;
    }
    // cpp: style_engine.cc:673-734
    pub fn UpdateActiveStyleSheets(&self) {
        if !self.NeedsActiveStyleSheetUpdate() {
            return;
        }
        debug_assert!(B::DocumentIsActive(&self.document_));
        if self.sheet_state.borrow().user_style_dirty {
            self.UpdateActiveUserStyleSheets();
        }
        let medium = self.EnsureMediaQueryEvaluator();
        if self.ShouldUpdateDocumentStyleSheetCollection() {
            B::CollectionPrepareUpdateActiveStyleSheets(
                &self.document_style_sheet_collection_,
                &medium,
            );
        }
        if self.ShouldUpdateShadowTreeStyleSheetCollection() {
            let scopes: Vec<_> = self
                .sheet_state
                .borrow()
                .dirty_tree_scopes
                .values()
                .cloned()
                .collect();
            for scope in scopes {
                self.PrepareUpdateActiveStyleSheetsInShadow(&scope, &medium);
            }
        }
        if self.ShouldUpdateDocumentStyleSheetCollection() {
            let mixins = B::CollectionMixins(&self.document_style_sheet_collection_);
            B::CollectionFinishUpdateActiveStyleSheets(
                &self.document_style_sheet_collection_,
                &mixins,
                self,
            );
        }
        if self.ShouldUpdateShadowTreeStyleSheetCollection() {
            let scopes: Vec<_> = self
                .sheet_state
                .borrow()
                .dirty_tree_scopes
                .values()
                .cloned()
                .collect();
            for scope in scopes {
                let collection = self
                    .StyleSheetCollectionFor(&scope)
                    .expect("dirty scope collection");
                let mixins = self.EffectiveMixinsForTreeScope(&scope);
                B::CollectionFinishUpdateActiveStyleSheets(&collection, &mixins, self);
                if !B::CollectionHasStyleSheetCandidateNodes(&collection)
                    && !B::TreeScopeHasAdoptedStyleSheets(&scope)
                {
                    self.sheet_state
                        .borrow_mut()
                        .active_tree_scopes
                        .remove(&identity(&scope));
                    debug_assert!(B::TreeScopeScopedStyleResolver(&scope).is_none());
                }
            }
        }
        B::ActiveStyleSheetsUpdated(&self.document_);
        let mut state = self.sheet_state.borrow_mut();
        state.dirty_tree_scopes.clear();
        state.document_scope_dirty = false;
        state.tree_scopes_removed = false;
        state.user_style_dirty = false;
    }
    // cpp: style_engine.cc:736-771
    fn EffectiveMixinsForTreeScope(&self, scope: &Rc<B::TreeScope>) -> B::MixinMap {
        let parent = B::ParentTreeScope(scope);
        let Some(collection) = self.StyleSheetCollectionFor(scope) else {
            return parent.map_or_else(B::EmptyMixinMap, |parent| {
                self.EffectiveMixinsForTreeScope(&parent)
            });
        };
        let local = B::CollectionMixins(&collection);
        let Some(parent) = parent else {
            return local;
        };
        let mut inherited = self.EffectiveMixinsForTreeScope(&parent);
        if !B::MixinMapHasMixins(&inherited) {
            return local;
        }
        let inherited_id = B::MixinMapIdentifier(&inherited);
        B::MixinMapMerge(&mut inherited, &local);
        B::MixinMapSetIdentifier(
            &mut inherited,
            B::MixinMapIdentifier(&local).or(inherited_id),
        );
        inherited
    }
    // cpp: style_engine.cc:829-836
    pub fn UpdateActiveStyle(&self) {
        debug_assert!(B::DocumentIsActive(&self.document_));
        self.UpdateViewport();
        self.UpdateActiveStyleSheets();
        self.UpdateGlobalRuleSet();
    }
    // cpp: style_engine.h:926-931
    pub fn UpdateGlobalRuleSet(&self) {
        debug_assert!(!self.NeedsActiveStyleSheetUpdate());
        if let Some(rules) = self.global_rule_set_.get() {
            B::GlobalRuleSetUpdate(&rules, &self.document_);
        }
    }
    // cpp: style_engine.cc:838-862
    pub fn ActiveStyleSheetsForInspector(&self) -> ActiveStyleSheetVector<B> {
        if B::DocumentIsActive(&self.document_) {
            self.UpdateActiveStyle();
        }
        let scopes = self.GetActiveTreeScopes();
        let mut sheets = B::CollectionActiveStyleSheets(&self.document_style_sheet_collection_);
        for scope in scopes {
            if let Some(collection) = self.StyleSheetCollectionFor(&scope) {
                sheets.extend(B::CollectionActiveStyleSheets(&collection));
            }
        }
        sheets
    }

    // cpp: style_engine.cc:4714-4724
    pub fn MarkAllElementsForStyleRecalc(&self, reason: StyleChangeReason) {
        if let Some(root) = B::DocumentElement(&self.document_) {
            B::ElementSetNeedsStyleRecalc(&root, StyleChangeType::kSubtreeStyleChange, reason);
        }
        self.functional_media_query_results_.borrow_mut().clear();
        self.functional_media_query_result_flags_
            .borrow_mut()
            .Clear();
        self.functional_navigation_query_results_
            .borrow_mut()
            .clear();
    }
    // cpp: style_engine.cc:1299-1315
    fn MarkElementsForRecalc(
        &self,
        scope: &Rc<B::TreeScope>,
        reason: StyleChangeReason,
        predicate: &dyn Fn(&B::ComputedStyle) -> bool,
    ) {
        let mut next = B::FirstElementWithin(scope);
        while let Some(element) = next {
            if let Some(root) = B::ElementShadowRoot(&element) {
                self.MarkElementsForRecalc(&root, reason, predicate);
            }
            if let Some(style) = B::ElementComputedStyle(&element) {
                if predicate(&style) || B::PseudoElementStylesDependOnFunc(&element, predicate) {
                    B::ElementSetNeedsStyleRecalc(
                        &element,
                        StyleChangeType::kLocalStyleChange,
                        reason,
                    );
                }
            }
            next = B::NextElementIncludingPseudo(&element);
        }
    }
    // cpp: style_engine.cc:525-537
    fn MediaQueryAffectingScopeValueChanged(
        &self,
        scope: &Rc<B::TreeScope>,
        change: MediaValueChange,
    ) {
        let collection = self
            .StyleSheetCollectionFor(scope)
            .expect("media query scope collection");
        if AffectedByMediaValueChange(&B::CollectionActiveStyleSheets(&collection), change) {
            self.SetNeedsActiveStyleUpdate(scope);
        }
        self.InvalidateFunctionalMediaDependentStylesIfNeeded();
    }
    // cpp: style_engine.cc:539-545
    pub fn WatchedSelectorsChanged(&self) {
        let rules = self.global_rule_set_.get().expect("global rule set");
        B::InitWatchedSelectorsRuleSet(&rules, &self.document_);
        self.MarkAllElementsForStyleRecalc(StyleChangeReason::DeclarativeContent);
    }
    // cpp: style_engine.cc:582-589
    fn MediaQueryAffectingTreeScopesValueChanged(
        &self,
        scopes: &[Rc<B::TreeScope>],
        change: MediaValueChange,
    ) {
        for scope in scopes {
            debug_assert!(!self.IsDocumentScope(scope));
            self.MediaQueryAffectingScopeValueChanged(scope, change);
        }
    }
    // cpp: style_engine.cc:607-634
    fn MediaQueryAffectingTextTracksValueChanged(
        &self,
        tracks: &[Rc<B::TextTrack>],
        _change: MediaValueChange,
    ) {
        if tracks.is_empty() {
            return;
        }
        for track in tracks {
            let mut needs_recalc = false;
            for sheet in B::TextTrackCSSStyleSheets(track) {
                let contents = B::SheetContents(&sheet);
                if B::ContentsHasMediaQueries(&contents) {
                    needs_recalc = true;
                    B::ContentsClearRuleSet(&contents);
                }
            }
            if needs_recalc {
                if let Some(owner) = B::TextTrackOwner(track) {
                    B::ElementSetNeedsStyleRecalc(
                        &owner,
                        StyleChangeType::kSubtreeStyleChange,
                        StyleChangeReason::Shadow,
                    );
                }
            }
        }
    }
    // cpp: style_engine.cc:636-646
    pub fn MediaQueryAffectingValueChanged(&self, change: MediaValueChange) {
        let user_affected =
            AffectedByMediaValueChange(&self.sheet_state.borrow().active_user_style_sheets, change);
        if user_affected {
            self.MarkUserStyleDirty();
        }
        self.MediaQueryAffectingScopeValueChanged(&B::DocumentTreeScope(&self.document_), change);
        self.MediaQueryAffectingTreeScopesValueChanged(&self.GetActiveTreeScopes(), change);
        let tracks: Vec<_> = self.text_tracks_.borrow().values().cloned().collect();
        self.MediaQueryAffectingTextTracksValueChanged(&tracks, change);
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverUpdateMediaType(&resolver);
        }
    }
    // cpp: style_engine.cc:4726-4743
    pub fn UpdateViewportStyle(&self) {
        if !self.viewport_style_dirty_.get() {
            return;
        }
        self.viewport_style_dirty_.set(false);
        let Some(resolver) = self.resolver_.get() else {
            return;
        };
        let style = B::ResolverStyleForViewport(&resolver);
        let view = B::DocumentLayoutView(&self.document_);
        if B::ComputeStyleDifference(&style, &B::LayoutObjectStyle(&view))
            != B::EqualStyleDifference()
        {
            B::LayoutObjectSetStyle(&view, style);
        }
    }
    // cpp: style_engine.cc:2776-2786
    pub fn InitialStyleChanged(&self) {
        self.InvalidateInitialStyle();
        self.MarkViewportStyleDirty();
        self.UpdateViewportStyle();
        self.MediaQueryAffectingValueChanged(MediaValueChange::kOther);
        self.MarkAllElementsForStyleRecalc(StyleChangeReason::Settings);
    }
    // cpp: style_engine.cc:2794-2797
    pub fn UAStyleChanged(&self) {
        self.MarkAllElementsForStyleRecalc(StyleChangeReason::Settings);
    }
    // cpp: style_engine.cc:2799-2821
    pub fn ViewportStyleSettingChanged(&self) {
        if let Some(resolver) = self.viewport_resolver_.get() {
            B::ViewportSetNeedsUpdate(&resolver);
        }
        if B::DocumentHasScopedStyleResolver(&self.document_) {
            self.MarkDocumentDirty();
            B::ScopedResolverSetNeedsAppendAllSheets(&self.document_);
            self.MarkAllElementsForStyleRecalc(StyleChangeReason::ActiveStylesheetsUpdate);
        }
    }
    // cpp: style_engine.cc:3283-3297
    pub fn InvalidateFunctionalMediaDependentStylesIfNeeded(&self) {
        let Some(evaluator) = self.media_query_evaluator_.borrow().clone() else {
            return;
        };
        let results: Vec<_> = self
            .functional_media_query_results_
            .borrow()
            .values()
            .cloned()
            .collect();
        if !B::MediaQueryResultsChanged(&evaluator, &results) {
            return;
        }
        self.functional_media_query_results_.borrow_mut().clear();
        self.functional_media_query_result_flags_
            .borrow_mut()
            .Clear();
        self.MarkElementsForRecalc(
            &B::DocumentTreeScope(&self.document_),
            StyleChangeReason::MediaQuery,
            &B::AffectedByFunctionalMedia,
        );
    }
    // cpp: style_engine.cc:3323-3340
    pub fn InvalidateFunctionalNavigationDependentStylesIfNeeded(&self) {
        let has_changes = self
            .functional_navigation_query_results_
            .borrow()
            .values()
            .any(|(expression, result)| {
                B::NavigationMatches(expression, &self.document_) != *result
            });
        if !has_changes {
            return;
        }
        self.functional_navigation_query_results_
            .borrow_mut()
            .clear();
        self.MarkElementsForRecalc(
            &B::DocumentTreeScope(&self.document_),
            StyleChangeReason::NavigationQuery,
            &B::AffectedByFunctionalNavigation,
        );
    }

    // cpp: style_engine.h:869-869
    pub fn GetDocument(&self) -> &Rc<B::Document> {
        &self.document_
    }

    // cpp: style_engine.cc:290-303
    pub fn EnsureStyleSheetCollectionFor(
        &self,
        scope: &Rc<B::TreeScope>,
    ) -> Rc<B::StyleSheetCollection> {
        if self.IsDocumentScope(scope) {
            return self.document_style_sheet_collection_.clone();
        }
        let mut collections = self.style_sheet_collection_map_.borrow_mut();
        // Oilpan removes dead WeakMember keys. Rust may reuse an expired
        // allocation's address before this map is visited, so expire it here.
        if collections
            .get(&identity(scope))
            .is_some_and(|(key, _)| key.upgrade().is_none())
        {
            collections.remove(&identity(scope));
        }
        collections
            .entry(identity(scope))
            .or_insert_with(|| (Rc::downgrade(scope), B::NewStyleSheetCollection(scope)))
            .1
            .clone()
    }
    // cpp: style_engine.cc:305-317
    pub fn StyleSheetCollectionFor(
        &self,
        scope: &Rc<B::TreeScope>,
    ) -> Option<Rc<B::StyleSheetCollection>> {
        if self.IsDocumentScope(scope) {
            return Some(self.document_style_sheet_collection_.clone());
        }
        self.style_sheet_collection_map_
            .borrow()
            .get(&identity(scope))
            .filter(|(key, _)| key.upgrade().is_some_and(|key| Rc::ptr_eq(&key, scope)))
            .map(|(_, collection)| collection.clone())
    }
    // cpp: style_engine.cc:290-303
    fn IsDocumentScope(&self, scope: &Rc<B::TreeScope>) -> bool {
        Rc::ptr_eq(scope, &B::DocumentTreeScope(&self.document_))
    }

    // cpp: style_engine.cc:319-325
    pub fn StyleSheetsForStyleSheetList(&self, scope: &Rc<B::TreeScope>) -> Vec<Rc<B::StyleSheet>> {
        let collection = self.EnsureStyleSheetCollectionFor(scope);
        B::CollectionUpdateStyleSheetList(&collection);
        B::CollectionStyleSheetsForList(&collection)
    }
    // cpp: style_engine.h:238-240
    pub fn InjectedAuthorStyleSheets(
        &self,
    ) -> std::cell::Ref<'_, Vec<(StyleSheetKey, Rc<B::CSSStyleSheet>)>> {
        self.injected_author_style_sheets_.borrow()
    }
    // cpp: style_engine.h:242-244
    pub fn InspectorStyleSheets(&self) -> std::cell::Ref<'_, Vec<Rc<B::CSSStyleSheet>>> {
        self.inspector_style_sheet_list_.borrow()
    }

    // cpp: style_engine.cc:327-341
    pub fn InjectSheet(
        &self,
        key: StyleSheetKey,
        contents: Rc<B::StyleSheetContents>,
        origin: WebCssOrigin,
    ) {
        let sheets = if origin == WebCssOrigin::kUser {
            &self.injected_user_style_sheets_
        } else {
            &self.injected_author_style_sheets_
        };
        sheets
            .borrow_mut()
            .push((key, B::NewCSSStyleSheet(contents, &self.document_)));
        if origin == WebCssOrigin::kUser {
            self.MarkUserStyleDirty();
        } else {
            self.MarkDocumentDirty();
        }
    }
    // cpp: style_engine.cc:343-361
    pub fn RemoveInjectedSheet(&self, key: &StyleSheetKey, origin: WebCssOrigin) {
        let sheets = if origin == WebCssOrigin::kUser {
            &self.injected_user_style_sheets_
        } else {
            &self.injected_author_style_sheets_
        };
        let mut sheets = sheets.borrow_mut();
        if let Some(index) = sheets.iter().rposition(|(entry, _)| entry == key) {
            sheets.remove(index);
            drop(sheets);
            if origin == WebCssOrigin::kUser {
                self.MarkUserStyleDirty();
            } else {
                self.MarkDocumentDirty();
            }
        }
    }

    // cpp: style_engine.cc:377-400
    pub fn AddPendingBlockingSheet(&self, node: &B::Node, kind: PendingSheetType) {
        debug_assert!(matches!(
            kind,
            PendingSheetType::kBlocking | PendingSheetType::kDynamicRenderBlocking
        ));
        let is_render_blocking =
            B::AddPendingRenderBlockingStylesheet(&self.document_, node).unwrap_or(false);
        if kind != PendingSheetType::kBlocking {
            return;
        }
        self.pending_script_blocking_stylesheets_
            .set(self.pending_script_blocking_stylesheets_.get() + 1);
        if !is_render_blocking {
            self.pending_parser_blocking_stylesheets_
                .set(self.pending_parser_blocking_stylesheets_.get() + 1);
            if B::DocumentHasBody(&self.document_) {
                B::CountPendingStylesheetAddedAfterBodyStarted(&self.document_);
            }
            B::DidAddPendingParserBlockingStylesheet(&self.document_);
        }
    }
    // cpp: style_engine.cc:403-438
    pub fn RemovePendingBlockingSheet(&self, node: &B::Node, kind: PendingSheetType) {
        debug_assert!(matches!(
            kind,
            PendingSheetType::kBlocking | PendingSheetType::kDynamicRenderBlocking
        ));
        if B::NodeIsConnected(node) {
            self.SetNeedsActiveStyleUpdate(&B::NodeTreeScope(node));
        }
        let is_render_blocking =
            B::RemovePendingRenderBlockingStylesheet(&self.document_, node).unwrap_or(false);
        if kind != PendingSheetType::kBlocking {
            return;
        }
        if !is_render_blocking {
            let count = self.pending_parser_blocking_stylesheets_.get();
            debug_assert!(count > 0);
            self.pending_parser_blocking_stylesheets_.set(count - 1);
            if count == 1 {
                B::DidLoadAllPendingParserBlockingStylesheets(&self.document_);
            }
        }
        let count = self.pending_script_blocking_stylesheets_.get();
        debug_assert!(count > 0);
        self.pending_script_blocking_stylesheets_.set(count - 1);
        if count != 1 {
            return;
        }
        B::DidRemoveAllPendingStylesheets(&self.document_);
    }

    // cpp: style_engine.cc:440-445
    pub fn SetNeedsActiveStyleUpdate(&self, scope: &Rc<B::TreeScope>) {
        debug_assert!(B::TreeScopeRootIsConnected(scope));
        if B::DocumentIsActive(&self.document_) {
            self.MarkTreeScopeDirty(scope);
        }
    }
    // cpp: style_engine.cc:447-460
    pub fn AddStyleSheetCandidateNode(&self, node: &B::Node) {
        if !B::NodeIsConnected(node) || B::DocumentIsDetached(&self.document_) {
            return;
        }
        debug_assert!(!B::NodeIsXSLStyleSheet(node));
        let scope = B::NodeTreeScope(node);
        B::CollectionAddCandidate(&self.EnsureStyleSheetCollectionFor(&scope), node);
        self.SetNeedsActiveStyleUpdate(&scope);
        if !self.IsDocumentScope(&scope) {
            self.sheet_state
                .borrow_mut()
                .active_tree_scopes
                .insert(identity(&scope), scope);
        }
    }
    // cpp: style_engine.cc:462-486
    pub fn RemoveStyleSheetCandidateNode(
        &self,
        node: &B::Node,
        insertion_point: &B::ContainerNode,
    ) {
        let insertion_point = B::ContainerNodeAsNode(insertion_point);
        debug_assert!(!B::NodeIsXSLStyleSheet(node));
        debug_assert!(B::NodeIsConnected(insertion_point));
        let scope = B::ContainingShadowRoot(node)
            .or_else(|| B::ContainingShadowRoot(insertion_point))
            .unwrap_or_else(|| B::DocumentTreeScope(&self.document_));
        let Some(collection) = self.StyleSheetCollectionFor(&scope) else {
            return;
        };
        B::CollectionRemoveCandidate(&collection, node);
        self.SetNeedsActiveStyleUpdate(&scope);
    }
    // cpp: style_engine.cc:488-492
    pub fn ModifiedStyleSheetCandidateNode(&self, node: &B::Node) {
        if B::NodeIsConnected(node) {
            self.SetNeedsActiveStyleUpdate(&B::NodeTreeScope(node));
        }
    }
    // cpp: style_engine.cc:494-508
    pub fn AdoptedStyleSheetAdded(&self, scope: &Rc<B::TreeScope>, sheet: &B::CSSStyleSheet) {
        if B::DocumentIsDetached(&self.document_) {
            return;
        }
        B::AddedAdoptedToTreeScope(sheet, scope);
        if !B::TreeScopeRootIsConnected(scope) {
            return;
        }
        self.EnsureStyleSheetCollectionFor(scope);
        if !self.IsDocumentScope(scope) {
            self.sheet_state
                .borrow_mut()
                .active_tree_scopes
                .insert(identity(scope), scope.clone());
        }
        self.SetNeedsActiveStyleUpdate(scope);
    }
    // cpp: style_engine.cc:510-523
    pub fn AdoptedStyleSheetRemoved(&self, scope: &Rc<B::TreeScope>, sheet: &B::CSSStyleSheet) {
        if B::DocumentIsDetached(&self.document_) {
            return;
        }
        B::RemovedAdoptedFromTreeScope(sheet, scope);
        if !B::TreeScopeRootIsConnected(scope) || self.StyleSheetCollectionFor(scope).is_none() {
            return;
        }
        self.SetNeedsActiveStyleUpdate(scope);
    }
    // cpp: style_engine.h:260-269
    pub fn GetActiveTreeScopes(&self) -> Vec<Rc<B::TreeScope>> {
        let scopes: Vec<_> = self
            .sheet_state
            .borrow()
            .active_tree_scopes
            .values()
            .cloned()
            .collect();
        debug_assert!(scopes.iter().all(|scope| !self.IsDocumentScope(scope)));
        scopes
    }

    // cpp: style_engine.cc:1136-1147
    pub fn MarkTreeScopeDirty(&self, scope: &Rc<B::TreeScope>) {
        if self.IsDocumentScope(scope) {
            self.MarkDocumentDirty();
            return;
        }
        let collection = self
            .StyleSheetCollectionFor(scope)
            .expect("shadow scope collection");
        B::CollectionMarkSheetListDirty(&collection);
        self.sheet_state
            .borrow_mut()
            .dirty_tree_scopes
            .insert(identity(scope), scope.clone());
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:1149-1153
    pub fn MarkDocumentDirty(&self) {
        self.sheet_state.borrow_mut().document_scope_dirty = true;
        B::CollectionMarkSheetListDirty(&self.document_style_sheet_collection_);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:1155-1158
    pub fn MarkUserStyleDirty(&self) {
        self.sheet_state.borrow_mut().user_style_dirty = true;
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:1160-1163
    pub fn MarkViewportStyleDirty(&self) {
        self.viewport_style_dirty_.set(true);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: style_engine.cc:574-576
    pub fn ShouldUpdateDocumentStyleSheetCollection(&self) -> bool {
        self.sheet_state.borrow().document_scope_dirty
    }
    // cpp: style_engine.cc:578-580
    pub fn ShouldUpdateShadowTreeStyleSheetCollection(&self) -> bool {
        !self.sheet_state.borrow().dirty_tree_scopes.is_empty()
    }
    // cpp: style_engine.h:855-858
    pub fn NeedsActiveStyleSheetUpdate(&self) -> bool {
        let state = self.sheet_state.borrow();
        state.tree_scopes_removed
            || state.document_scope_dirty
            || !state.dirty_tree_scopes.is_empty()
            || state.user_style_dirty
    }
    // cpp: style_engine.cc:817-821
    pub fn UpdateViewport(&self) {
        if let Some(resolver) = self.viewport_resolver_.get() {
            B::ViewportUpdate(&resolver);
        }
    }
    // cpp: style_engine.cc:823-827
    pub fn NeedsActiveStyleUpdate(&self) -> bool {
        self.viewport_resolver_
            .get()
            .as_ref()
            .is_some_and(|r| B::ViewportNeedsUpdate(r))
            || self.NeedsActiveStyleSheetUpdate()
            || self
                .global_rule_set_
                .get()
                .as_ref()
                .is_some_and(|rules| B::GlobalRuleSetIsDirty(&rules))
    }

    // cpp: style_engine.cc:1212-1219
    fn TextCacheKey(text: &String) -> AtomicString {
        if text.length() >= 1024 {
            let bytes = if text.Is8Bit() {
                text.Span8().expect("8-bit string storage").to_vec()
            } else {
                text.Span16()
                    .expect("16-bit string storage")
                    .iter()
                    .flat_map(|unit| unit.to_ne_bytes())
                    .collect()
            };
            AtomicString::from_latin1(&B::FastHash(&bytes).to_ne_bytes())
        } else if text.IsNull() {
            AtomicString::default()
        } else {
            AtomicString::from_utf16(text.Span16().expect("non-null string storage"))
        }
    }
    // cpp: style_engine.cc:1205-1235
    pub fn FindStyleSheetContents(
        &self,
        text: &String,
        context: Option<&B::CSSParserContext>,
    ) -> Option<Rc<B::StyleSheetContents>> {
        let context = context?;
        let key = Self::TextCacheKey(text);
        let cached = self.text_to_sheet_cache_.borrow().get(&key)?.upgrade();
        let Some(contents) = cached else {
            self.text_to_sheet_cache_.borrow_mut().remove(&key);
            return None;
        };
        if !B::ContentsCacheableForStyleElement(&contents) {
            self.text_to_sheet_cache_.borrow_mut().remove(&key);
            return None;
        }
        let Some(stored_context) = B::ContentsParserContext(&contents) else {
            self.text_to_sheet_cache_.borrow_mut().remove(&key);
            return None;
        };
        if stored_context.as_ref() != context {
            return None;
        }
        debug_assert!(B::ContentsHasSingleOwnerDocument(&contents));
        B::ContentsSetUsedFromTextCache(&contents);
        Some(contents)
    }
    // cpp: style_engine.cc:1237-1261
    pub fn AddStyleSheetContents(
        &self,
        text: &String,
        contents: Option<&Rc<B::StyleSheetContents>>,
    ) {
        let Some(contents) = contents else {
            return;
        };
        if !B::ContentsCacheableForStyleElement(contents)
            || B::ContentsParserContext(contents).is_none()
        {
            return;
        }
        self.text_to_sheet_cache_
            .borrow_mut()
            .insert(Self::TextCacheKey(text), Rc::downgrade(contents));
    }
    // cpp: style_engine.cc:2667-2674
    pub fn SetPreferredStylesheetSetNameIfNotSet(&self, name: &String) {
        debug_assert!(!name.empty());
        if !self.preferred_stylesheet_set_name_.borrow().empty() {
            return;
        }
        *self.preferred_stylesheet_set_name_.borrow_mut() = name.clone();
        self.MarkDocumentDirty();
    }
    // cpp: style_engine.cc:2676-2680
    pub fn SetHttpDefaultStyle(&self, content: &String) {
        if !content.empty() {
            self.SetPreferredStylesheetSetNameIfNotSet(content);
        }
    }

    // cpp: style_engine.h:307-310
    pub fn WatchedSelectorsRuleSet(&self) -> Option<Rc<B::RuleSet>> {
        B::WatchedSelectorsRuleSet(
            self.global_rule_set_
                .get()
                .as_ref()
                .expect("global rule set"),
        )
    }
    // cpp: style_engine.h:311-314
    pub fn DocumentRulesSelectorsRuleSet(&self) -> Option<Rc<B::RuleSet>> {
        B::DocumentRulesSelectorsRuleSet(
            self.global_rule_set_
                .get()
                .as_ref()
                .expect("global rule set"),
        )
    }

    // cpp: style_engine.cc:591-593
    pub fn AddTextTrack(&self, track: Rc<B::TextTrack>) {
        self.text_tracks_
            .borrow_mut()
            .insert(identity(&track), track);
    }
    // cpp: style_engine.cc:595-597
    pub fn RemoveTextTrack(&self, track: &Rc<B::TextTrack>) {
        self.text_tracks_.borrow_mut().remove(&identity(track));
    }
    // cpp: style_engine.cc:599-605
    pub fn EnsureVTTOriginatingElement(&self) -> Rc<B::Element> {
        self.vtt_originating_element_
            .borrow_mut()
            .get_or_insert_with(|| {
                B::NewElement(
                    &self.document_,
                    foundation::g_null_atom.clone(),
                    foundation::g_empty_atom.clone(),
                    foundation::g_empty_atom.clone(),
                )
            })
            .clone()
    }

    // cpp: style_engine.cc:1025-1031
    pub fn RuleSetForSheet(
        &self,
        sheet: &B::CSSStyleSheet,
        mixins: &B::MixinMap,
    ) -> Option<Rc<B::RuleSet>> {
        let evaluator = self.EnsureMediaQueryEvaluator();
        if !B::SheetMatchesMediaQueries(sheet, &evaluator) {
            return None;
        }
        Some(B::ContentsEnsureRuleSet(
            &B::SheetContents(sheet),
            &evaluator,
            mixins,
        ))
    }
    // cpp: style_engine.cc:1033-1040
    pub fn CreateUnconnectedRuleSet(
        &self,
        sheet: &B::CSSStyleSheet,
        mixins: &B::MixinMap,
    ) -> Option<Rc<B::RuleSet>> {
        let evaluator = self.EnsureMediaQueryEvaluator();
        if !B::SheetMatchesMediaQueries(sheet, &evaluator) {
            return None;
        }
        B::ContentsCreateUnconnectedRuleSet(&B::SheetContents(sheet), &evaluator, mixins)
    }

    // cpp: style_engine.cc:2788-2792
    pub fn InvalidateInitialStyle(&self) {
        if let Some(resolver) = self.resolver_.get() {
            B::ResolverInvalidateInitialStyle(&resolver);
        }
    }

    // cpp: style_engine.cc:3276-3281
    pub fn EvaluateFunctionalMediaQuery(&self, query: Rc<B::MediaQuerySet>) -> bool {
        let result = B::EvalMediaQuery(
            &self.EnsureMediaQueryEvaluator(),
            &query,
            &mut self.functional_media_query_result_flags_.borrow_mut(),
        );
        // HeapHashMap::insert preserves the first result for an existing key.
        self.functional_media_query_results_
            .borrow_mut()
            .entry(identity(&query))
            .or_insert((query, result));
        result
    }

    // cpp: style_engine.cc:3299-3304
    pub fn EvaluateFunctionalNavigationQuery(
        &self,
        expression: Rc<B::NavigationTestExpression>,
    ) -> bool {
        let result = B::NavigationMatches(&expression, &self.document_);
        self.functional_navigation_query_results_
            .borrow_mut()
            .entry(identity(&expression))
            .or_insert((expression, result));
        result
    }
    // cpp: style_engine.cc:3306-3315
    pub fn AddURLPatternFromLocation(
        &self,
        name: AtomicString,
        pattern: Option<Rc<B::URLPattern>>,
    ) {
        debug_assert!(name.length() >= 2 && name.at(0) == b'-' as u16 && name.at(1) == b'-' as u16);
        self.navigation_locations_
            .borrow_mut()
            .entry(name)
            .or_insert(pattern);
    }
    // cpp: style_engine.cc:3317-3321
    pub fn FindURLPatternByLocation(&self, name: &AtomicString) -> Option<Rc<B::URLPattern>> {
        self.navigation_locations_
            .borrow()
            .get(name)
            .cloned()
            .flatten()
    }

    // cpp: style_engine.cc:3342-3352
    pub fn EnsureMediaQueryEvaluator(&self) -> Rc<B::MediaQueryEvaluator> {
        self.media_query_evaluator_
            .borrow_mut()
            .get_or_insert_with(|| {
                if B::DocumentHasFrame(&self.document_) {
                    B::NewFrameMediaQueryEvaluator(&self.document_)
                } else {
                    B::NewMediaQueryEvaluatorForType("all")
                }
            })
            .clone()
    }

    // cpp: style_engine.h:294-296
    pub fn GetOwnerColorScheme(&self) -> ColorScheme {
        self.owner_color_scheme_.get()
    }

    // cpp: style_engine.cc:4659-4671
    pub fn ResolveColorSchemeForEmbedding(
        &self,
        style: Option<&B::ComputedStyle>,
    ) -> PreferredColorScheme {
        if style.is_none_or(B::ColorSchemeFlagsIsNormal) {
            return self.preferred_color_scheme.get();
        }
        if B::UsedColorScheme(style.expect("embedder style")) == ColorScheme::kDark {
            PreferredColorScheme::kDark
        } else {
            PreferredColorScheme::kLight
        }
    }
    // cpp: style_engine.cc:4866-4900
    pub fn MarkForLayoutTreeChangesAfterDetach(&self) {
        let Some(object) = self.parent_for_detached_subtree_.borrow().clone() else {
            return;
        };
        if let Some(element) = B::LayoutObjectElement(&object) {
            let mut mark_ancestors = false;
            if (B::ElementHasFirstChild(&element)
                || B::ElementIsShadowHost(&element)
                || B::ElementIsActiveSlot(&element))
                && !B::WhitespaceChildrenMayChange(&object)
            {
                B::SetWhitespaceChildrenMayChange(&object, true);
                mark_ancestors = true;
            }
            if !B::WasNotifiedOfSubtreeChange(&object) && B::NotifyOfSubtreeChange(&object) {
                mark_ancestors = true;
            }
            if mark_ancestors {
                B::MarkAncestorsWithChildNeedsStyleRecalc(&element);
            }
        }
        *self.parent_for_detached_subtree_.borrow_mut() = None;
    }
    // cpp: style_engine.cc:4902-4904
    pub fn InvalidateSVGResourcesAfterDetach(&self) {
        B::InvalidatePendingSVGResources(&self.document_);
    }
    // cpp: style_engine.cc:4906-4918
    pub fn AllowSkipStyleRecalcForScope(&self) -> bool {
        if self.InContainerQueryStyleRecalc() {
            return true;
        }
        B::DocumentViewHasSubtreeLayoutRoots(&self.document_).is_none_or(|roots| !roots)
    }
    // cpp: style_engine.h:337-339
    pub fn SetNeedsStyleUpdateOnNavigation(&self) {
        self.needs_style_update_on_navigation_.set(true);
    }

    // cpp: style_engine.h:340-342
    pub fn NeedsStyleUpdateOnNavigation(&self) -> bool {
        self.needs_style_update_on_navigation_.get()
    }

    // cpp: style_engine.h:355-357
    pub fn PreferredStylesheetSetName(&self) -> String {
        self.preferred_stylesheet_set_name_.borrow().clone()
    }

    // cpp: style_engine.h:366-368
    pub fn HasPendingScriptBlockingSheets(&self) -> bool {
        self.pending_script_blocking_stylesheets_.get() > 0
    }

    // cpp: style_engine.h:369-371
    pub fn HaveScriptBlockingStylesheetsLoaded(&self) -> bool {
        !self.HasPendingScriptBlockingSheets()
    }

    // cpp: style_engine.h:467-467
    pub fn InApplyAnimationUpdate(&self) -> bool {
        self.in_apply_animation_update_.get()
    }

    // cpp: style_engine.h:480-480
    pub fn InEnsureComputedStyle(&self) -> bool {
        self.in_ensure_computed_style_.get()
    }

    // cpp: style_engine.h:510-510
    pub fn SkipStyleRecalcAllowed(&self) -> bool {
        self.allow_skip_style_recalc_.get()
    }

    // cpp: style_engine.h:602-602
    pub fn StyleForElementCount(&self) -> u32 {
        self.style_for_element_count_.get()
    }

    // cpp: style_engine.h:603-603
    pub fn IncStyleForElementCount(&self) {
        self.style_for_element_count_
            .set(self.style_for_element_count_.get() + 1);
    }

    // cpp: style_engine.h:631-631
    pub fn IsViewportStyleDirty(&self) -> bool {
        self.viewport_style_dirty_.get()
    }

    // cpp: style_engine.cc:4419-4422
    pub fn MarkReattachAllowed(&self) -> bool {
        !self.InRebuildLayoutTree() || self.allow_mark_for_reattach_from_rebuild_layout_tree_.get()
    }

    // cpp: style_engine.h:799-799
    pub fn GetHumanReadableName(&self) -> &'static str {
        "StyleEngine"
    }

    // cpp: style_engine.h:388-388
    pub fn SetStyleAffectedByLayout(&self) {
        self.style_affected_by_layout_.set(true);
    }

    // cpp: style_engine.h:389-389
    pub fn StyleAffectedByLayout(&self) -> bool {
        self.style_affected_by_layout_.get()
    }

    // cpp: style_engine.h:393-393
    pub fn SkippedContainerRecalc(&self) -> bool {
        self.skipped_container_recalc_.get() != 0
    }

    // cpp: style_engine.h:394-394
    pub fn IncrementSkippedContainerRecalc(&self) {
        self.skipped_container_recalc_
            .set(self.skipped_container_recalc_.get() + 1);
    }

    // cpp: style_engine.h:395-395
    pub fn DecrementSkippedContainerRecalc(&self) {
        self.skipped_container_recalc_
            .set(self.skipped_container_recalc_.get() - 1);
    }

    // cpp: style_engine.h:397-397
    pub fn UsesLineHeightUnits(&self) -> bool {
        self.uses_line_height_units_.get()
    }

    // cpp: style_engine.h:398-400
    pub fn SetUsesLineHeightUnits(&self, uses_line_height_units: bool) {
        self.uses_line_height_units_.set(uses_line_height_units);
    }

    // cpp: style_engine.h:402-402
    pub fn UsesGlyphRelativeUnits(&self) -> bool {
        self.uses_glyph_relative_units_.get()
    }

    // cpp: style_engine.h:403-405
    pub fn SetUsesGlyphRelativeUnits(&self, uses_glyph_relative_units: bool) {
        self.uses_glyph_relative_units_
            .set(uses_glyph_relative_units);
    }

    // cpp: style_engine.h:407-407
    pub fn UsesRootRelativeUnits(&self) -> bool {
        self.uses_root_relative_units_.get()
    }

    // cpp: style_engine.h:408-410
    pub fn SetUsesRootRelativeUnits(&self, uses_root_relative_units: bool) {
        self.uses_root_relative_units_.set(uses_root_relative_units);
    }

    // cpp: style_engine.h:413-413
    pub fn SetUsesTreeCountingFunctions(&self) {
        self.uses_tree_counting_functions_.set(true);
    }

    // cpp: style_engine.h:417-417
    pub fn CountersChanged(&self) -> bool {
        self.counters_changed_.get()
    }

    // cpp: style_engine.h:418-418
    pub fn MarkCountersDirty(&self) {
        self.counters_changed_.set(true);
    }

    // cpp: style_engine.h:419-419
    pub fn MarkCountersClean(&self) {
        self.counters_changed_.set(false);
    }

    // cpp: style_engine.h:724-724
    pub fn InRebuildLayoutTree(&self) -> bool {
        self.in_layout_tree_rebuild_.get()
    }

    // cpp: style_engine.h:725-725
    pub fn InDOMRemoval(&self) -> bool {
        self.in_dom_removal_.get()
    }

    // cpp: style_engine.h:726-726
    pub fn InDetachLayoutTree(&self) -> bool {
        self.in_detach_scope_.get()
    }

    // cpp: style_engine.h:727-729
    pub fn InContainerQueryStyleRecalc(&self) -> bool {
        self.in_container_query_style_recalc_.get()
    }

    // cpp: style_engine.h:730-732
    pub fn InPositionTryStyleRecalc(&self) -> bool {
        self.in_position_try_style_recalc_.get()
    }

    // cpp: style_engine.h:733-735
    pub fn InInterleavedStyleRecalc(&self) -> bool {
        self.InContainerQueryStyleRecalc() || self.InPositionTryStyleRecalc()
    }

    // cpp: style_engine.h:736-739
    pub fn SetInScrollMarkersAttachment(&self, in_scroll_markers_attachment: bool) {
        debug_assert!(!self.in_scroll_markers_attachment_.get() || !in_scroll_markers_attachment);
        self.in_scroll_markers_attachment_
            .set(in_scroll_markers_attachment);
    }

    // cpp: style_engine.h:740-742
    pub fn InScrollMarkersAttachment(&self) -> bool {
        self.in_scroll_markers_attachment_.get()
    }
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:486-495
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AncestorAnalysis {
    kNone,
    kInterleavingRoot,
    kStyleRoot,
}

// cpp: style_engine.h:150-166. The mark happens while the detach flag is set;
// SVG invalidation happens only after restoring the enclosing flag value.
pub struct DetachLayoutTreeScope<'a, B: StyleEngineBackend> {
    engine_: &'a StyleEngine<B>,
    old_value_: bool,
}
impl<'a, B: StyleEngineBackend> DetachLayoutTreeScope<'a, B> {
    pub fn new(engine: &'a StyleEngine<B>) -> Self {
        Self {
            engine_: engine,
            old_value_: engine.in_detach_scope_.replace(true),
        }
    }
}
impl<B: StyleEngineBackend> Drop for DetachLayoutTreeScope<'_, B> {
    fn drop(&mut self) {
        self.engine_.MarkForLayoutTreeChangesAfterDetach();
        self.engine_.in_detach_scope_.set(self.old_value_);
        self.engine_.InvalidateSVGResourcesAfterDetach();
    }
}

// cpp: style_engine.h:217-230
pub struct SkipStyleRecalcScope<'a> {
    value_: &'a Cell<bool>,
    old_value_: bool,
}
impl<'a> SkipStyleRecalcScope<'a> {
    pub fn new<B: StyleEngineBackend>(engine: &'a StyleEngine<B>) -> Self {
        let old_value_ = engine
            .allow_skip_style_recalc_
            .replace(engine.AllowSkipStyleRecalcForScope());
        Self {
            value_: &engine.allow_skip_style_recalc_,
            old_value_,
        }
    }
}
impl Drop for SkipStyleRecalcScope<'_> {
    fn drop(&mut self) {
        self.value_.set(self.old_value_);
    }
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:139-148
pub struct DOMRemovalScope<'a> {
    value_: &'a Cell<bool>,
    old_value_: bool,
}

impl<'a> DOMRemovalScope<'a> {
    pub fn new<B: StyleEngineBackend>(engine: &'a StyleEngine<B>) -> Self {
        let old_value_ = engine.in_dom_removal_.replace(true);
        Self {
            value_: &engine.in_dom_removal_,
            old_value_,
        }
    }
}

impl Drop for DOMRemovalScope<'_> {
    fn drop(&mut self) {
        self.value_.set(self.old_value_);
    }
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:168-179
pub struct AttachScrollMarkersScope<'a> {
    value_: &'a Cell<bool>,
    old_value_: bool,
}

impl<'a> AttachScrollMarkersScope<'a> {
    pub fn new<B: StyleEngineBackend>(engine: &'a StyleEngine<B>) -> Self {
        let old_value_ = engine.in_scroll_markers_attachment_.replace(true);
        Self {
            value_: &engine.in_scroll_markers_attachment_,
            old_value_,
        }
    }
}

impl Drop for AttachScrollMarkersScope<'_> {
    fn drop(&mut self) {
        self.value_.set(self.old_value_);
    }
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:455-465
pub struct InApplyAnimationUpdateScope<'a> {
    value_: &'a Cell<bool>,
    old_value_: bool,
}

impl<'a> InApplyAnimationUpdateScope<'a> {
    pub fn new<B: StyleEngineBackend>(engine: &'a StyleEngine<B>) -> Self {
        let old_value_ = engine.in_apply_animation_update_.replace(true);
        Self {
            value_: &engine.in_apply_animation_update_,
            old_value_,
        }
    }
}

impl Drop for InApplyAnimationUpdateScope<'_> {
    fn drop(&mut self) {
        self.value_.set(self.old_value_);
    }
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:469-478
pub struct InEnsureComputedStyleScope<'a> {
    value_: &'a Cell<bool>,
    old_value_: bool,
}

impl<'a> InEnsureComputedStyleScope<'a> {
    pub fn new<B: StyleEngineBackend>(engine: &'a StyleEngine<B>) -> Self {
        let old_value_ = engine.in_ensure_computed_style_.replace(true);
        Self {
            value_: &engine.in_ensure_computed_style_,
            old_value_,
        }
    }
}

impl Drop for InEnsureComputedStyleScope<'_> {
    fn drop(&mut self) {
        self.value_.set(self.old_value_);
    }
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:181-197
// There are a few instances where nodes are marked style dirty from within
// style recalc. Such marking must stay inside the subtree being traversed.
pub struct AllowMarkStyleDirtyFromRecalcScope<'a> {
    value_: &'a Cell<bool>,
    old_value_: bool,
}

impl<'a> AllowMarkStyleDirtyFromRecalcScope<'a> {
    pub fn new<B: StyleEngineBackend>(engine: &'a StyleEngine<B>) -> Self {
        let old_value_ = engine.allow_mark_style_dirty_from_recalc_.replace(true);
        Self {
            value_: &engine.allow_mark_style_dirty_from_recalc_,
            old_value_,
        }
    }
}

impl Drop for AllowMarkStyleDirtyFromRecalcScope<'_> {
    fn drop(&mut self) {
        self.value_.set(self.old_value_);
    }
}

// cpp: third_party/blink/renderer/core/css/style_engine.h:199-215
// ::first-letter reattachment can be scheduled while rebuilding the layout
// tree only while this scope is active.
pub struct AllowMarkForReattachFromRebuildLayoutTreeScope<'a> {
    value_: &'a Cell<bool>,
    old_value_: bool,
}

impl<'a> AllowMarkForReattachFromRebuildLayoutTreeScope<'a> {
    pub fn new<B: StyleEngineBackend>(engine: &'a StyleEngine<B>) -> Self {
        let old_value_ = engine
            .allow_mark_for_reattach_from_rebuild_layout_tree_
            .replace(true);
        Self {
            value_: &engine.allow_mark_for_reattach_from_rebuild_layout_tree_,
            old_value_,
        }
    }
}

impl Drop for AllowMarkForReattachFromRebuildLayoutTreeScope<'_> {
    fn drop(&mut self) {
        self.value_.set(self.old_value_);
    }
}

#[cfg(test)]
mod active_sheet_update_tests {
    use super::*;
    struct Sheet;
    impl StyleSheetMediaQueries for Sheet {
        fn HasMediaQueryResults(&self) -> bool {
            false
        }
        fn HasMediaQueries(&self) -> bool {
            false
        }
        fn HasDynamicViewportDependentMediaQueries(&self) -> bool {
            false
        }
    }
    struct Rules(CascadeLayer);
    impl CascadeLayerRuleSet for Rules {
        fn CascadeLayers(&self) -> Option<&CascadeLayer> {
            Some(&self.0)
        }
    }
    struct Diff;
    impl RuleSetDiff<Rules> for Diff {
        fn Matches(&self, _: &Rules, _: &Rules) -> bool {
            false
        }
        fn CreateDiffRuleset(&self) -> Option<Rc<Rules>> {
            None
        }
    }
    #[test]
    fn active_sheet_diff_cascade_priority_and_invalidation_flags() {
        let root = CascadeLayer::default();
        let first_layer = root.GetOrAddSubLayer(&[AtomicString::from_str("first")]);
        let second_layer = root.GetOrAddSubLayer(&[AtomicString::from_str("second")]);
        let sheet = Rc::new(Sheet);
        let first_rules = Rc::new(Rules(root));
        let second_rules = Rc::new(Rules(CascadeLayer::default()));
        let old = vec![ActiveStyleSheet::new(
            sheet.clone(),
            Some(first_rules.clone()),
        )];
        let mut new = old.clone();
        new.push(ActiveStyleSheet::new(
            Rc::new(Sheet),
            Some(second_rules.clone()),
        ));
        let mut changed = ChangedRuleSets::default();
        assert_eq!(
            CompareActiveStyleSheets(&old, &new, &[] as &[Diff], &mut changed),
            ActiveSheetsChange::kActiveSheetsAppended
        );
        assert_eq!(changed.Values().len(), 1);
        assert!(Rc::ptr_eq(&changed.Values()[0], &second_rules));
        let mut changed = ChangedRuleSets::default();
        assert_eq!(
            CompareActiveStyleSheets(&new, &old, &[] as &[Diff], &mut changed),
            ActiveSheetsChange::kActiveSheetsChanged
        );
        assert_eq!(changed.Values().len(), 1);
        let layers = CascadeLayerMap::new(&old);
        assert_eq!(
            CompareLayerOrder(Some(&layers), Some(&first_layer), Some(&second_layer)),
            Ordering::Less
        );
        assert_eq!(CompareLayerOrder(None, None, None), Ordering::Equal);
        let name = AtomicString::from_str("--property");
        let mut cascade = AtRuleCascadeMap::new();
        let user_outer = AtRulePriority {
            is_user_style: true,
            layer_order: u16::MAX,
        };
        let author_first = AtRulePriority {
            is_user_style: false,
            layer_order: layers.GetLayerOrder(&first_layer),
        };
        let author_second = AtRulePriority {
            is_user_style: false,
            layer_order: layers.GetLayerOrder(&second_layer),
        };
        assert!(cascade.AddAndCascade(name.clone(), user_outer));
        assert!(cascade.AddAndCascade(name.clone(), author_first));
        assert!(!cascade.AddAndCascade(name.clone(), user_outer));
        assert!(cascade.AddAndCascade(name.clone(), author_second));
        assert!(cascade.AddAndCascade(name.clone(), author_second));
        assert!(!cascade.AddAndCascade(name, author_first));
        assert!(!FlagsCauseInvalidation(InvalidationMatchFlags {
            affected_by_drag: false,
            affected_by_focus_within: false,
            affected_by_hover: false,
            affected_by_active: false
        }));
        for index in 0..4 {
            assert!(FlagsCauseInvalidation(InvalidationMatchFlags {
                affected_by_drag: index == 0,
                affected_by_focus_within: index == 1,
                affected_by_hover: index == 2,
                affected_by_active: index == 3
            }));
        }
    }
}

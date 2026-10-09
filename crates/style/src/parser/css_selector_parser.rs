// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// cpp: third_party/blink/renderer/core/css/parser/css_selector_parser.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Conservative source audit (physical / effective / credited / pending):
// css_selector_parser.h: 377 / 186 / 186 / 0.
// css_selector_parser.cc: 2809 / 1855 / 1855 / 0.
// Combined pending effective source lines: 0.
// Exact credited body ranges are in crates/style/css_selector_parser_ledger.tsv.
// Effective excludes comments, blank/preprocessor/namespace lines and lines
// consisting only of braces, parentheses and semicolons. No omitted credit.
// Token-only logical/compound/shadow/argument/CSSOM paths and typed observer,
// runtime and usage/deprecation interfaces follow the Chromium control flow.
// Shared-arena helpers, forgiving-relative block entry, ident() values and
// NavigationLocation are translated. Runtime/observer/usage and mathematical
// ident() integer values are supplied through typed embedding interfaces.
// The optional ident() function's Value() remains null exactly as Chromium's
// CSSCustomIdentValue::Value(), rather than evaluating an unresolved CSS value.
#![allow(non_snake_case)]
use super::css_nesting_type::CSSNestingType;
use super::css_parser_token::{
    BlockType, CSSParserTokenType::*, HashTokenType, NumericSign, NumericValueType,
};
use super::css_parser_token_stream::{BlockGuard, CSSParserTokenStream, TokenStreamTokenizer};
use super::css_tokenizer::CSSTokenizer;
use crate::css_selector::*;
use crate::css_selector_list::CSSSelectorList;
use foundation::StringView;
use foundation::{AtomicString, CSSValueID, String};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::cell::{Cell, RefCell};
use std::ops::Range;
use std::rc::Rc;
trait SelectorAtom {
    fn ToAtomicString(&self) -> AtomicString;
}
impl SelectorAtom for StringView {
    fn ToAtomicString(&self) -> AtomicString {
        AtomicString::from_utf16(self.Span16())
    }
}

pub struct SelectorParserContext {
    pub html: bool,
    pub quirks: bool,
}
impl CSSSelectorParserContext for SelectorParserContext {
    fn ParsePseudoType(&self, name: &AtomicString, args: bool) -> PseudoType {
        CSSSelectorParser::ParsePseudoType(name, args, self)
    }
}
impl CSSSelectorAttributeContext for SelectorParserContext {
    fn IsCaseSensitiveAttribute(&self, name: &QualifiedName) -> bool {
        // cpp: html_names.cc IsCaseSensitiveAttribute. Non-enumerated values
        // retain case; only HTML enumerated attribute values fold implicitly.
        !matches!(
            name.LocalName().Utf8().as_str(),
            "accept"
                | "accept-charset"
                | "align"
                | "alink"
                | "axis"
                | "bgcolor"
                | "charset"
                | "checked"
                | "clear"
                | "codetype"
                | "color"
                | "compact"
                | "declare"
                | "defer"
                | "dir"
                | "direction"
                | "disabled"
                | "enctype"
                | "face"
                | "frame"
                | "hreflang"
                | "http-equiv"
                | "lang"
                | "language"
                | "link"
                | "media"
                | "method"
                | "multiple"
                | "nohref"
                | "noresize"
                | "noshade"
                | "nowrap"
                | "readonly"
                | "rel"
                | "rev"
                | "rules"
                | "scope"
                | "scrolling"
                | "selected"
                | "shape"
                | "target"
                | "text"
                | "type"
                | "valign"
                | "valuetype"
                | "vlink"
        )
    }
}
// Typed snapshot assembled by the embedding document's runtime registry.
// Defaults match runtime_enabled_features.json5 stable/test/experimental status;
// document-dependent gates see no Document in the standalone parser context.
#[derive(Clone, Copy)]
pub struct SelectorRuntimeFeatures {
    pub CSSMediaElementPseudos: bool,
    pub GeolocationElement: bool,
    pub UserMediaElement: bool,
    pub InstallElement: bool,
    pub CSSPseudoScrollMarkers: bool,
    pub CSSScrollMarkerTargetBeforeAfter: bool,
    pub CSSPseudoScrollButtons: bool,
    pub CSSPseudoColumn: bool,
    pub SearchTextHighlightPseudo: bool,
    pub WebMCP: bool,
    pub UnboundedElement: bool,
    pub CSSPseudoHasSlotted: bool,
    pub OverscrollGestures: bool,
    pub CustomizableCombobox: bool,
    pub FilterableSelect: bool,
    pub CSSImageAnimation: bool,
    pub MenuElements: bool,
    pub NavigationSourcePseudoClass: bool,
    pub DeclarativeSkeletons: bool,
    pub has_document: bool,
}
impl Default for SelectorRuntimeFeatures {
    fn default() -> Self {
        Self {
            CSSMediaElementPseudos: false,
            GeolocationElement: true,
            UserMediaElement: true,
            InstallElement: false,
            CSSPseudoScrollMarkers: true,
            CSSScrollMarkerTargetBeforeAfter: true,
            CSSPseudoScrollButtons: true,
            CSSPseudoColumn: true,
            SearchTextHighlightPseudo: true,
            WebMCP: false,
            UnboundedElement: true,
            CSSPseudoHasSlotted: false,
            OverscrollGestures: false,
            CustomizableCombobox: false,
            FilterableSelect: false,
            CSSImageAnimation: false,
            MenuElements: false,
            NavigationSourcePseudoClass: false,
            DeclarativeSkeletons: false,
            has_document: false,
        }
    }
}
impl CSSSelectorRuntime for SelectorRuntimeFeatures {
    fn CSSMediaElementPseudosEnabled(&self) -> bool {
        self.CSSMediaElementPseudos
    }
    fn GeolocationElementEnabled(&self) -> bool {
        self.GeolocationElement
    }
    fn UserMediaElementEnabled(&self) -> bool {
        self.UserMediaElement
    }
    fn InstallElementEnabled(&self) -> bool {
        self.InstallElement
    }
    fn CSSPseudoScrollMarkersEnabled(&self) -> bool {
        self.CSSPseudoScrollMarkers
    }
    fn CSSScrollMarkerTargetBeforeAfterEnabled(&self) -> bool {
        self.CSSScrollMarkerTargetBeforeAfter
    }
    fn CSSPseudoScrollButtonsEnabled(&self) -> bool {
        self.CSSPseudoScrollButtons
    }
    fn CSSPseudoColumnEnabled(&self) -> bool {
        self.CSSPseudoColumn
    }
    fn SearchTextHighlightPseudoEnabled(&self) -> bool {
        self.SearchTextHighlightPseudo
    }
    fn WebMCPEnabled(&self) -> bool {
        self.WebMCP
    }
    fn UnboundedElementEnabled(&self) -> bool {
        self.UnboundedElement
    }
    fn CSSPseudoHasSlottedEnabled(&self) -> bool {
        self.CSSPseudoHasSlotted
    }
    fn OverscrollGesturesEnabled(&self) -> bool {
        self.OverscrollGestures
    }
    fn CustomizableComboboxEnabled(&self) -> bool {
        self.CustomizableCombobox
    }
    fn FilterableSelectEnabled(&self) -> bool {
        self.FilterableSelect
    }
    fn CSSImageAnimationEnabled(&self) -> bool {
        self.CSSImageAnimation
    }
    fn MenuElementsEnabled(&self) -> bool {
        self.MenuElements
    }
    fn NavigationSourcePseudoClassEnabled(&self) -> bool {
        self.NavigationSourcePseudoClass
    }
    fn DeclarativeSkeletonsEnabled(&self) -> bool {
        self.DeclarativeSkeletons
    }
    fn HasDocument(&self) -> bool {
        self.has_document
    }
}
impl CSSSelectorRuntime for SelectorParserContext {
    fn CSSMediaElementPseudosEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CSSMediaElementPseudos
    }
    fn GeolocationElementEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().GeolocationElement
    }
    fn UserMediaElementEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().UserMediaElement
    }
    fn InstallElementEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().InstallElement
    }
    fn CSSPseudoScrollMarkersEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CSSPseudoScrollMarkers
    }
    fn CSSScrollMarkerTargetBeforeAfterEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CSSScrollMarkerTargetBeforeAfter
    }
    fn CSSPseudoScrollButtonsEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CSSPseudoScrollButtons
    }
    fn CSSPseudoColumnEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CSSPseudoColumn
    }
    fn SearchTextHighlightPseudoEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().SearchTextHighlightPseudo
    }
    fn WebMCPEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().WebMCP
    }
    fn UnboundedElementEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().UnboundedElement
    }
    fn CSSPseudoHasSlottedEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CSSPseudoHasSlotted
    }
    fn OverscrollGesturesEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().OverscrollGestures
    }
    fn CustomizableComboboxEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CustomizableCombobox
    }
    fn FilterableSelectEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().FilterableSelect
    }
    fn CSSImageAnimationEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().CSSImageAnimation
    }
    fn MenuElementsEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().MenuElements
    }
    fn NavigationSourcePseudoClassEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().NavigationSourcePseudoClass
    }
    fn DeclarativeSkeletonsEnabled(&self) -> bool {
        SelectorRuntimeFeatures::default().DeclarativeSkeletons
    }
    fn HasDocument(&self) -> bool {
        false
    }
}

/// Namespace lookup supplied by the stylesheet owner. Query parsing has no
/// stylesheet and cannot resolve named namespace prefixes.
pub trait CSSSelectorNamespaceContext {
    fn DefaultNamespace(&self) -> AtomicString;
    fn NamespaceURIFromPrefix(&self, prefix: &AtomicString) -> AtomicString;
}
impl<B: crate::style_sheet_contents::StyleSheetContentsBackend> CSSSelectorNamespaceContext
    for crate::style_sheet_contents::StyleSheetContents<B>
{
    fn DefaultNamespace(&self) -> AtomicString {
        self.DefaultNamespace()
    }
    fn NamespaceURIFromPrefix(&self, prefix: &AtomicString) -> AtomicString {
        self.NamespaceURIFromPrefix(prefix)
    }
}
/// Source runtime defaults; experimental callers supply their actual values.
#[derive(Clone, Copy)]
pub struct SelectorParserFeatures {
    pub logical_combination_pseudo: bool,
    pub marker_nested_pseudo_element: bool,
    pub pseudo_elements_hoverable: bool,
    pub serialize_invalid_selectors: bool,
    pub lang_extended_ranges: bool,
    pub case_sensitive_selector: bool,
    pub ident_function: bool,
    pub route_matching: bool,
}
impl Default for SelectorParserFeatures {
    fn default() -> Self {
        Self {
            logical_combination_pseudo: false,
            marker_nested_pseudo_element: true,
            pseudo_elements_hoverable: false,
            serialize_invalid_selectors: false,
            lang_extended_ranges: false,
            case_sensitive_selector: false,
            ident_function: false,
            route_matching: false,
        }
    }
}
/// The node/document and pseudo-element exposure collaborators used by CSSOM.
pub trait CSSSelectorPseudoElementContext {
    fn HasParent(&self) -> bool;
    fn Runtime(&self) -> &dyn CSSSelectorRuntime;
    fn CSSOMGetComputedStylePseudoElementRequiresColonEnabled(&self) -> bool;
    fn IsWebExposed(&self, pseudo: PseudoId) -> bool;
    fn CountPseudoElementWithoutColon(&self);
}

pub trait CSSSelectorParserObserver {
    fn ObserveSelector(&mut self, start: u32, end: u32);
}
// cpp: css_selector_parser.h:367-373.
pub fn AbortsNestedSelectorParsing(
    token_type: super::css_parser_token::CSSParserTokenType,
    semicolon_aborts_nested_selector: bool,
    nesting: CSSNestingType,
) -> bool {
    semicolon_aborts_nested_selector
        && token_type == kSemicolonToken
        && nesting != CSSNestingType::kNone
}

/// Typed accounting sink owned by the parser context's document.
/// Count/CountWebDX retain the context's recording policy; deprecation routing
/// uses the embedding document's source deprecation registry.
pub trait CSSSelectorUsageContext {
    fn IsUseCounterRecordingEnabled(&self) -> bool;
    fn IsDeprecated(&self, feature: SelectorWebFeature) -> bool;
    fn Count(&self, feature: SelectorWebFeature);
    fn CountDeprecation(&self, feature: SelectorWebFeature);
    fn CountWebDX(&self, feature: SelectorWebDXFeature);
}
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum SelectorWebFeature {
    kActiveViewTransitionPseudo,
    kCSSPseudoOpen,
    kCSSPseudoParentInScope,
    kCSSSelectorCue,
    kCSSSelectorIndirectAdjacent,
    kCSSSelectorInternalMediaControlsOverlayCastButton,
    kCSSSelectorInternalPseudoListBox,
    kCSSSelectorInternalPseudoSpatialNavigationFocus,
    kCSSSelectorNthChildOfSelector,
    kCSSSelectorPseudoAny,
    kCSSSelectorPseudoAnyLink,
    kCSSSelectorPseudoDefined,
    kCSSSelectorPseudoDir,
    kCSSSelectorPseudoFileSelectorButton,
    kCSSSelectorPseudoFocus,
    kCSSSelectorPseudoFocusVisible,
    kCSSSelectorPseudoFullScreen,
    kCSSSelectorPseudoFullScreenAncestor,
    kCSSSelectorPseudoHas,
    kCSSSelectorPseudoHasSlotted,
    kCSSSelectorPseudoHost,
    kCSSSelectorPseudoHostContext,
    kCSSSelectorPseudoIs,
    kCSSSelectorPseudoModal,
    kCSSSelectorPseudoNot,
    kCSSSelectorPseudoReadOnly,
    kCSSSelectorPseudoReadWrite,
    kCSSSelectorPseudoSlotted,
    kCSSSelectorPseudoState,
    kCSSSelectorPseudoWebkitAnyLink,
    kCSSSelectorPseudoWhere,
    kCSSSelectorUserInvalid,
    kCSSSelectorUserValid,
    kCSSSelectorWebkitCalendarPickerIndicator,
    kCSSSelectorWebkitClearButton,
    kCSSSelectorWebkitColorSwatch,
    kCSSSelectorWebkitColorSwatchWrapper,
    kCSSSelectorWebkitDateAndTimeValue,
    kCSSSelectorWebkitDatetimeEdit,
    kCSSSelectorWebkitDatetimeEditAmpmField,
    kCSSSelectorWebkitDatetimeEditDayField,
    kCSSSelectorWebkitDatetimeEditFieldsWrapper,
    kCSSSelectorWebkitDatetimeEditHourField,
    kCSSSelectorWebkitDatetimeEditMillisecondField,
    kCSSSelectorWebkitDatetimeEditMinuteField,
    kCSSSelectorWebkitDatetimeEditMonthField,
    kCSSSelectorWebkitDatetimeEditSecondField,
    kCSSSelectorWebkitDatetimeEditText,
    kCSSSelectorWebkitDatetimeEditWeekField,
    kCSSSelectorWebkitDatetimeEditYearField,
    kCSSSelectorWebkitFileUploadButton,
    kCSSSelectorWebkitInnerSpinButton,
    kCSSSelectorWebkitInputPlaceholder,
    kCSSSelectorWebkitMediaControls,
    kCSSSelectorWebkitMediaControlsCurrentTimeDisplay,
    kCSSSelectorWebkitMediaControlsEnclosure,
    kCSSSelectorWebkitMediaControlsFullscreenButton,
    kCSSSelectorWebkitMediaControlsMuteButton,
    kCSSSelectorWebkitMediaControlsOverlayEnclosure,
    kCSSSelectorWebkitMediaControlsOverlayPlayButton,
    kCSSSelectorWebkitMediaControlsPanel,
    kCSSSelectorWebkitMediaControlsPlayButton,
    kCSSSelectorWebkitMediaControlsTimeRemainingDisplay,
    kCSSSelectorWebkitMediaControlsTimeline,
    kCSSSelectorWebkitMediaControlsTimelineContainer,
    kCSSSelectorWebkitMediaControlsToggleClosedCaptionsButton,
    kCSSSelectorWebkitMediaControlsVolumeSlider,
    kCSSSelectorWebkitMediaSliderContainer,
    kCSSSelectorWebkitMediaSliderThumb,
    kCSSSelectorWebkitMediaTextTrackContainer,
    kCSSSelectorWebkitMediaTextTrackDisplay,
    kCSSSelectorWebkitMediaTextTrackRegion,
    kCSSSelectorWebkitMediaTextTrackRegionContainer,
    kCSSSelectorWebkitMeterBar,
    kCSSSelectorWebkitMeterEvenLessGoodValue,
    kCSSSelectorWebkitMeterInnerElement,
    kCSSSelectorWebkitMeterOptimumValue,
    kCSSSelectorWebkitMeterSuboptimumValue,
    kCSSSelectorWebkitProgressBar,
    kCSSSelectorWebkitProgressInnerElement,
    kCSSSelectorWebkitProgressValue,
    kCSSSelectorWebkitSearchCancelButton,
    kCSSSelectorWebkitSliderContainer,
    kCSSSelectorWebkitSliderRunnableTrack,
    kCSSSelectorWebkitSliderThumb,
    kCSSSelectorWebkitTextfieldDecorationContainer,
    kCSSSelectorWebkitUnknownPseudo,
    kCSSUnknownNamespacePrefixInSelector,
    kGetComputedStylePseudoElementWithoutColon,
    kHasBeforeOrAfterPseudoElement,
    kHasMarkerPseudoElement,
    kHasSpellingOrGrammarErrorPseudoElement,
}
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorWebDXFeature {
    kAutofill,
    kDetailsContent,
    kMediaPseudos,
    kTimeRelativeSelectors,
}

// css_parsing_utils.cc:1708-1773; css_custom_ident_value.h:36-39.
// Selector consumers call Value(), which is null for function-backed values in
// Chromium. Keep the parsed function typed until that exact operation.
#[derive(Clone)]
enum CSSCustomIdentValue {
    Literal(AtomicString),
    IdentFunction(Vec<CSSIdentComponent>),
}
#[derive(Clone)]
enum CSSIdentComponent {
    CustomIdent(CSSCustomIdentValue),
    String(AtomicString),
    Integer(SelectorIdentIntegerValue),
}
#[derive(Clone)]
pub enum SelectorIdentIntegerValue {
    Literal(f64),
    Expression(Rc<dyn CSSSelectorIdentIntegerExpression>),
}
pub trait CSSSelectorIdentIntegerExpression {
    fn HasRandomFunctions(&self) -> bool;
}
/// Only mathematical integer functions require the value-parser collaborator.
/// It receives translated tokens including the complete function component.
pub trait CSSSelectorIdentIntegerParser {
    fn ParseInteger(
        &self,
        tokens: &[super::css_parser_token::CSSParserToken],
    ) -> Option<SelectorIdentIntegerValue>;
}
impl CSSCustomIdentValue {
    fn Value(&self) -> AtomicString {
        match self {
            Self::Literal(value) => value.clone(),
            Self::IdentFunction(_) => AtomicString::default(),
        }
    }
}
// navigation_query.h:25-38; navigation_query.cc:60-80.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationLocationType {
    kLocationName,
    kUrlPattern,
    kUrl,
}
pub struct NavigationLocation {
    pub location_type: NavigationLocationType,
    pub value: AtomicString,
}
impl CSSSelectorNavigationLocation for NavigationLocation {
    fn SerializeTo(&self, output: &mut Vec<u16>) {
        use crate::css_markup::{SerializeIdentifierTo, SerializeStringTo};
        let value = String::from_utf16(self.value.utf16_units().unwrap_or(&[]));
        match self.location_type {
            NavigationLocationType::kLocationName => SerializeIdentifierTo(&value, output, false),
            NavigationLocationType::kUrlPattern | NavigationLocationType::kUrl => {
                output.extend(
                    if self.location_type == NavigationLocationType::kUrl {
                        "url("
                    } else {
                        "url-pattern("
                    }
                    .encode_utf16(),
                );
                SerializeStringTo(&value, output);
                output.push(b')' as u16);
            }
        }
    }
}
pub struct NavigationParser;
impl NavigationParser {
    // cpp: navigation_parser.cc:129-177. The BlockGuard owns quoted functions;
    // token values are decoded by the existing tokenizer, never reparsed as CSS.
    pub fn ParseLocation<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<NavigationLocation>> {
        let first = stream.Peek().clone();
        if first.GetType() == kIdentToken && first.Value().Span16().starts_with(&[45, 45]) {
            return Some(Rc::new(NavigationLocation {
                location_type: NavigationLocationType::kLocationName,
                value: stream.ConsumeIncludingWhitespace().Value().ToAtomicString(),
            }));
        }
        let location_type;
        let value;
        if first.GetType() == kUrlToken {
            location_type = NavigationLocationType::kUrl;
            value = stream.ConsumeIncludingWhitespace().Value().ToAtomicString();
        } else {
            if first.GetType() != kFunctionToken {
                return None;
            }
            location_type = match first
                .Value()
                .ToAtomicString()
                .ToAsciiLower()
                .Utf8()
                .as_str()
            {
                "url-pattern" => NavigationLocationType::kUrlPattern,
                "url" => NavigationLocationType::kUrl,
                _ => return None,
            };
            {
                let mut guard = BlockGuard::new(stream);
                guard.ConsumeWhitespace();
                if guard.Peek().GetType() != kStringToken {
                    return None;
                }
                let token = guard.ConsumeIncludingWhitespace();
                if token.GetType() == kBadStringToken || !guard.UncheckedAtEnd() {
                    return None;
                }
                value = token.Value().ToAtomicString();
            }
            stream.ConsumeWhitespace();
        }
        Some(Rc::new(NavigationLocation {
            location_type,
            value,
        }))
    }
}
#[derive(Default)]
pub struct SelectorParserOptions<'a> {
    pub namespace_context: Option<&'a dyn CSSSelectorNamespaceContext>,
    pub parent_rule_for_nesting: Option<Rc<dyn CSSSelectorParentRule>>,
    pub features: SelectorParserFeatures,
    pub ua_sheet_mode: bool,
    pub runtime_context: Option<&'a dyn CSSSelectorRuntime>,
    pub usage_context: Option<&'a dyn CSSSelectorUsageContext>,
    pub ident_integer_parser: Option<&'a dyn CSSSelectorIdentIntegerParser>,
}
// cpp: css_selector_parser.h:109-120. Flags propagate even through discarded
// forgiving arguments; :has() starts a separate accumulator for its argument.
#[derive(Default)]
struct ResultFlags {
    contains_pseudo: bool,
    contains_complex: bool,
    contains_scope_or_parent: bool,
}
impl ResultFlags {
    fn merge(&mut self, flags: &Self) {
        self.contains_pseudo |= flags.contains_pseudo;
        self.contains_complex |= flags.contains_complex;
        self.contains_scope_or_parent |= flags.contains_scope_or_parent;
    }
}

// The embedding document may supply the actual feature registry without
// changing the query parser's existing HTML/quirks context interface.
struct PseudoContext<'a> {
    context: &'a SelectorParserContext,
    runtime: Option<&'a dyn CSSSelectorRuntime>,
}
impl CSSSelectorParserContext for PseudoContext<'_> {
    fn ParsePseudoType(&self, name: &AtomicString, args: bool) -> PseudoType {
        CSSSelectorParser::ParsePseudoType(name, args, self)
    }
}
macro_rules! forward_runtime {
    ($($method:ident),*) => { $(fn $method(&self) -> bool {
        self.runtime.unwrap_or(self.context).$method()
    })* };
}
impl CSSSelectorRuntime for PseudoContext<'_> {
    forward_runtime!(
        CSSMediaElementPseudosEnabled,
        GeolocationElementEnabled,
        UserMediaElementEnabled,
        InstallElementEnabled,
        CSSPseudoScrollMarkersEnabled,
        CSSScrollMarkerTargetBeforeAfterEnabled,
        CSSPseudoScrollButtonsEnabled,
        CSSPseudoColumnEnabled,
        SearchTextHighlightPseudoEnabled,
        HasDocument,
        WebMCPEnabled,
        UnboundedElementEnabled,
        CSSPseudoHasSlottedEnabled,
        OverscrollGesturesEnabled,
        CustomizableComboboxEnabled,
        FilterableSelectEnabled,
        CSSImageAnimationEnabled,
        MenuElementsEnabled,
        NavigationSourcePseudoClassEnabled,
        DeclarativeSkeletonsEnabled
    );
}

// cpp: .cc:99-124. Reordering never touches earlier arena segments.
fn ReverseSelectors(selectors: &mut [CSSSelector]) {
    let mut i = 0;
    let mut j = selectors.len();
    while i + 1 < j {
        selectors.swap(i, j - 1);
        i += 1;
        j -= 1;
    }
}
fn MarkAsEntireComplexSelector(selectors: &mut [CSSSelector]) {
    debug_assert!(!selectors.is_empty());
    debug_assert!(selectors[..selectors.len() - 1]
        .iter()
        .all(|s| !s.IsLastInComplexSelector()));
    selectors.last_mut().unwrap().SetLastInComplexSelector(true);
}
type SelectorArena = Rc<RefCell<Vec<CSSSelector>>>;
// cpp: .h:321-363. A cloned handle keeps the same backing vector; nested
// parsing restores its length while preserving its capacity and parent prefix.
struct ResetVectorAfterScope {
    vector: SelectorArena,
    initial_size: usize,
    committed: bool,
}
impl ResetVectorAfterScope {
    fn new(vector: &SelectorArena) -> Self {
        Self {
            initial_size: vector.borrow().len(),
            vector: vector.clone(),
            committed: false,
        }
    }
    fn AddedElements(&self) -> Range<usize> {
        let size = self.vector.borrow().len();
        debug_assert!(size >= self.initial_size);
        self.initial_size..size
    }
    fn WithAddedElementsMut<R>(&self, f: impl FnOnce(&mut [CSSSelector]) -> R) -> R {
        let range = self.AddedElements();
        f(&mut self.vector.borrow_mut()[range])
    }
    fn CommitAddedElements(&mut self) -> Range<usize> {
        self.committed = true;
        self.AddedElements()
    }
}
impl Drop for ResetVectorAfterScope {
    fn drop(&mut self) {
        debug_assert!(self.vector.borrow().len() >= self.initial_size);
        if !self.committed {
            self.vector.borrow_mut().truncate(self.initial_size);
        }
    }
}
// cpp: .h:288-308. Move-only Rust scope guards restore on every return path.
struct DisallowPseudoElementsScope {
    state: Rc<Cell<bool>>,
    was_disallowed: bool,
}
impl DisallowPseudoElementsScope {
    fn new(parser: &Parser<'_>) -> Self {
        let state = parser.disallow_pseudo.clone();
        let was_disallowed = state.replace(true);
        Self {
            state,
            was_disallowed,
        }
    }
}
impl Drop for DisallowPseudoElementsScope {
    fn drop(&mut self) {
        self.state.set(self.was_disallowed);
    }
}
struct Parser<'c> {
    context: &'c SelectorParserContext,
    options: &'c SelectorParserOptions<'c>,
    inside_has: bool,
    disallow_pseudo: Rc<Cell<bool>>,
    resist_default_namespace: bool,
    ignore_default_namespace: bool,
    inside_compound_pseudo: bool,
    found_host_in_compound: bool,
    in_supports_parsing: bool,
    restricting_pseudo_element: PseudoType,
    output: SelectorArena,
    semicolon_aborts_nested_selector: bool,
}
impl<'c> Parser<'c> {
    // cpp: .cc:254-264; .h:122-127,249-285. The arena is a shared backing
    // vector, while namespace/parent/runtime/document dependencies stay typed.
    fn new_in_arena(
        context: &'c SelectorParserContext,
        options: &'c SelectorParserOptions<'c>,
        semicolon_aborts_nested_selector: bool,
        output: SelectorArena,
        disallow_pseudo: bool,
    ) -> Self {
        Self {
            context,
            options,
            inside_has: false,
            disallow_pseudo: Rc::new(Cell::new(disallow_pseudo)),
            resist_default_namespace: false,
            ignore_default_namespace: false,
            inside_compound_pseudo: false,
            found_host_in_compound: false,
            in_supports_parsing: false,
            restricting_pseudo_element: PseudoType::kPseudoUnknown,
            output,
            semicolon_aborts_nested_selector,
        }
    }
    fn new(
        context: &'c SelectorParserContext,
        options: &'c SelectorParserOptions<'c>,
        disallow_pseudo: bool,
    ) -> Self {
        Self::new_in_arena(
            context,
            options,
            false,
            Rc::new(RefCell::new(Vec::new())),
            disallow_pseudo,
        )
    }
    fn adopt_range(&self, range: Range<usize>) -> Rc<CSSSelectorList> {
        let list =
            CSSSelectorList::AdoptSelectorVector(self.output.borrow()[range.clone()].to_vec());
        self.output.borrow_mut().truncate(range.start);
        list
    }
    fn count(&self, feature: SelectorWebFeature) {
        if let Some(context) = self.options.usage_context {
            context.Count(feature);
        }
    }
    fn name<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<(AtomicString, AtomicString)> {
        let token = s.Peek().clone();
        let mut name = match token.GetType() {
            kIdentToken => {
                s.Consume();
                token.Value().ToAtomicString()
            }
            kDelimiterToken if token.Delimiter() == b'*' as u16 => {
                s.Consume();
                UniversalSelectorAtom().clone()
            }
            kDelimiterToken if token.Delimiter() == b'|' as u16 => AtomicString::from_str(""),
            _ => return None,
        };
        let mut prefix = AtomicString::default();
        if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == b'|' as u16 {
            let state = s.Save();
            s.Consume();
            prefix = if name == *UniversalSelectorAtom() {
                AtomicString::from_str("*")
            } else {
                name
            };
            let next = s.Peek().clone();
            name = match next.GetType() {
                kIdentToken => {
                    s.Consume();
                    next.Value().ToAtomicString()
                }
                kDelimiterToken if next.Delimiter() == b'*' as u16 => {
                    s.Consume();
                    UniversalSelectorAtom().clone()
                }
                _ => {
                    s.Restore(state);
                    return None;
                }
            };
        }
        Some((name, prefix))
    }
    // cpp: css_selector_parser.cc:2328-2356
    fn default_namespace(&self) -> AtomicString {
        if self.ignore_default_namespace {
            return AtomicString::from_str("*");
        }
        self.options.namespace_context.map_or_else(
            || AtomicString::from_str("*"),
            |sheet| sheet.DefaultNamespace(),
        )
    }
    fn namespace(&self, prefix: &AtomicString, attribute: bool) -> Option<AtomicString> {
        Some(if prefix.IsNull() {
            if attribute {
                AtomicString::default()
            } else {
                self.default_namespace()
            }
        } else if prefix.empty() || prefix == &AtomicString::from_str("*") {
            prefix.clone()
        } else {
            let uri = self
                .options
                .namespace_context?
                .NamespaceURIFromPrefix(prefix);
            if uri.IsNull() {
                return None;
            }
            uri
        })
    }
    // cpp: css_selector_parser.cc:746-771,782-845
    fn selector_nesting_type(selector: &CSSSelector) -> CSSNestingType {
        let nested = selector
            .SelectorList()
            .map_or(CSSNestingType::kNone, |list| {
                Self::nesting_type_for_list(&list)
            });
        selector.GetNestingType().max(nested)
    }
    fn nesting_type(selectors: &[CSSSelector]) -> CSSNestingType {
        selectors
            .iter()
            .map(Self::selector_nesting_type)
            .max()
            .unwrap_or(CSSNestingType::kNone)
    }
    fn nesting_type_for_list(list: &CSSSelectorList) -> CSSNestingType {
        list.ComplexSelectorsIncludingUnparsedInvalid()
            .map(|complex| {
                complex
                    .SimpleSelectors()
                    .map(Self::selector_nesting_type)
                    .max()
                    .unwrap_or(CSSNestingType::kNone)
            })
            .max()
            .unwrap_or(CSSNestingType::kNone)
    }
    fn implicit_anchor(&self, nesting: CSSNestingType) -> CSSSelector {
        let mut selector = match nesting {
            CSSNestingType::kNesting => {
                CSSSelector::FromParent(self.options.parent_rule_for_nesting.clone(), true)
            }
            CSSNestingType::kScope => {
                CSSSelector::FromPseudo(AtomicString::from_str("scope"), true, self.context)
            }
            _ => panic!("qualified selector invalid in function/mixin context"),
        };
        selector.SetScopeContaining(true);
        selector
    }
    // cpp: css_selector_parser.cc:463-498
    fn consume_until_comma_nesting<T: TokenStreamTokenizer>(
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> CSSNestingType {
        let mut nesting = CSSNestingType::kNone;
        let mut previous_colon = false;
        while !s.AtEnd() {
            let token = s.Peek().clone();
            if token.GetBlockType() == BlockType::kBlockStart {
                let mut block = BlockGuard::new(s);
                while !block.AtEnd() {
                    nesting = nesting.max(Self::consume_until_comma_nesting(&mut block));
                    if !block.AtEnd() {
                        block.Consume();
                    }
                }
                continue;
            }
            if token.GetType() == kCommaToken {
                break;
            }
            if token.GetType() == kDelimiterToken && token.Delimiter() == 38 {
                nesting = nesting.max(CSSNestingType::kNesting);
            }
            if previous_colon
                && token.GetType() == kIdentToken
                && token.Value().ToAtomicString().ToAsciiLower() == AtomicString::from_str("scope")
            {
                nesting = CSSNestingType::kScope;
            }
            previous_colon = token.GetType() == kColonToken;
            s.Consume();
        }
        nesting
    }
    // cpp: css_selector_parser.cc:509-524
    fn placeholder<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<CSSSelector> {
        let start = s.LookAheadOffset();
        let nesting = Self::consume_until_comma_nesting(s);
        s.EnsureLookAhead();
        let end = s.LookAheadOffset();
        if nesting == CSSNestingType::kNone && !self.options.features.serialize_invalid_selectors {
            return None;
        }
        let value = s.StringRangeAt(start, end - start);
        let units = value.Span16();
        let first = units
            .iter()
            .position(|c| !foundation::unicode::IsSpaceOrNewline(*c))
            .unwrap_or(units.len());
        let last = units
            .iter()
            .rposition(|c| !foundation::unicode::IsSpaceOrNewline(*c))
            .map_or(first, |i| i + 1);
        let mut selector = CSSSelector::default();
        selector.SetMatch(MatchType::kPseudoClass);
        selector.SetUnparsedPlaceholder(nesting, AtomicString::from_utf16(&units[first..last]));
        selector.SetLastInComplexSelector(true);
        Some(selector)
    }
    // cpp: css_selector_parser.cc:2111-2126.
    fn peek_is_combinator<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> bool {
        s.ConsumeWhitespace();
        s.Peek().GetType() == kDelimiterToken && matches!(s.Peek().Delimiter(), 43 | 126 | 62)
    }
    fn combinator<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> RelationType {
        let mut relation = RelationType::kSubSelector;
        while s.Peek().GetType() == kWhitespaceToken {
            s.Consume();
            relation = RelationType::kDescendant;
        }
        if s.Peek().GetType() == kDelimiterToken {
            let explicit = match s.Peek().Delimiter() {
                43 => Some(RelationType::kDirectAdjacent),
                126 => Some(RelationType::kIndirectAdjacent),
                62 => Some(RelationType::kChild),
                _ => None,
            };
            if let Some(r) = explicit {
                s.ConsumeIncludingWhitespace();
                return r;
            }
        }
        relation
    }
    fn list<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
        forgiving: bool,
        relative: bool,
        nesting: CSSNestingType,
        result_flags: &mut ResultFlags,
    ) -> Option<Range<usize>> {
        let mut reset_vector = ResetVectorAfterScope::new(&self.output);
        s.ConsumeWhitespace();
        let forgiving = forgiving && !self.in_supports_parsing;
        loop {
            if s.AtEnd() {
                return if forgiving {
                    if let Some(selector) = self.placeholder(s) {
                        self.output.borrow_mut().push(selector);
                    }
                    reset_vector.WithAddedElementsMut(Self::mark_invalid_if_needed);
                    Some(reset_vector.CommitAddedElements())
                } else {
                    None
                };
            }
            let state = s.Save();
            let subpos = self.output.borrow().len();
            let next = self.complex(s, relative, nesting, result_flags);
            let at_end = s.AtEnd() || matches!(s.Peek().GetType(), kCommaToken | kLeftBraceToken);
            if let Some(v) = next.filter(|_| !forgiving || at_end) {
                let _ = v; // The successful complex selector is already in the arena.
            } else if forgiving {
                self.output.borrow_mut().truncate(subpos);
                s.EnsureLookAhead();
                s.Restore(state);
                if let Some(selector) = self.placeholder(s) {
                    self.output.borrow_mut().push(selector);
                }
            } else {
                return None;
            }
            if s.AtEnd() {
                break;
            }
            if s.Peek().GetType() != kCommaToken {
                break;
            }
            s.ConsumeIncludingWhitespace();
            if s.AtEnd() {
                if forgiving {
                    if let Some(selector) = self.placeholder(s) {
                        self.output.borrow_mut().push(selector);
                    }
                    break;
                } else {
                    return None;
                }
            }
        }
        if forgiving {
            reset_vector.WithAddedElementsMut(Self::mark_invalid_if_needed);
        }
        Some(reset_vector.CommitAddedElements())
    }
    // cpp: .cc:594-639. Relative recovery discards invalid entries rather
    // than serializing placeholders; :has() keeps the source's unforgiving path.
    fn forgiving_relative_list<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
        flags: &mut ResultFlags,
    ) -> Option<Rc<CSSSelectorList>> {
        if self.in_supports_parsing {
            let range = self.list(s, false, true, CSSNestingType::kNone, flags)?;
            let list = self.adopt_range(range);
            return list.IsValid().then_some(list);
        }
        let reset_vector = ResetVectorAfterScope::new(&self.output);
        while !s.AtEnd() {
            if s.Peek().GetBlockType() != BlockType::kBlockStart {
                return None;
            }
            let mut guard = BlockGuard::new(s);
            let subpos = self.output.borrow().len();
            let selector = self.complex(&mut guard, true, CSSNestingType::kNone, flags);
            if selector.is_none() || (!guard.AtEnd() && guard.Peek().GetType() != kCommaToken) {
                self.output.borrow_mut().truncate(subpos);
                guard.SkipUntilPeekedTypeIs(&[kCommaToken]);
            }
            if !guard.AtEnd() {
                guard.ConsumeIncludingWhitespace();
            }
        }
        let range = reset_vector.AddedElements();
        if self.inside_compound_pseudo
            || self.restricting_pseudo_element != PseudoType::kPseudoUnknown
            || range.is_empty()
        {
            return None;
        }
        // The scope still owns the segment; copy it into the list before Drop.
        let selectors = self.output.borrow()[range].to_vec();
        Some(CSSSelectorList::AdoptSelectorVector(selectors))
    }
    // cpp: .cc:2310-2326. Nested :nth-child(of ...) shares the arena and
    // disallows pseudo-elements until its adopted list has been copied out.
    fn nth_child_of_selectors<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<Rc<CSSSelectorList>> {
        if s.Peek().GetType() != kIdentToken
            || s.Peek().Value().ToAtomicString().ToAsciiLower() != AtomicString::from_str("of")
        {
            return None;
        }
        s.ConsumeIncludingWhitespace();
        let _scope = DisallowPseudoElementsScope::new(self);
        let range = self.list(
            s,
            false,
            false,
            CSSNestingType::kNone,
            &mut ResultFlags::default(),
        )?;
        Some(self.adopt_range(range))
    }
    // cpp: css_selector_parser.cc:339-402,526-593.
    // All nested :is/:where/:not arguments inherit compound-only parsing.
    fn compound_list<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
        forgiving: bool,
        result_flags: &mut ResultFlags,
    ) -> Option<Range<usize>> {
        let forgiving = forgiving && !self.in_supports_parsing;
        let mut reset_vector = ResetVectorAfterScope::new(&self.output);
        let mut trailing_empty = true;
        loop {
            if s.AtEnd() {
                if !forgiving {
                    return None;
                }
                break;
            }
            let start = s.LookAheadOffset();
            let subpos = self.output.borrow().len();
            let parsed = self.compound(s, result_flags);
            s.ConsumeWhitespace();
            let at_end = s.AtEnd() || s.Peek().GetType() == kCommaToken;
            if let Some(compound) = parsed.filter(|_| !forgiving || at_end) {
                MarkAsEntireComplexSelector(&mut self.output.borrow_mut()[compound]);
            } else if forgiving {
                self.output.borrow_mut().truncate(subpos);
                s.SkipUntilPeekedTypeIs(&[kCommaToken]);
                if self.options.features.serialize_invalid_selectors {
                    s.EnsureLookAhead();
                    self.output.borrow_mut().push(Self::unparsed_range(
                        s,
                        start,
                        CSSNestingType::kNone,
                    ));
                }
            } else {
                return None;
            }
            if s.AtEnd() || !forgiving && s.Peek().GetType() != kCommaToken {
                trailing_empty = false;
                break;
            }
            s.ConsumeIncludingWhitespace();
        }
        if trailing_empty && self.options.features.serialize_invalid_selectors {
            self.output.borrow_mut().push(Self::unparsed(
                CSSNestingType::kNone,
                AtomicString::from_str(""),
            ));
        }
        if forgiving {
            reset_vector.WithAddedElementsMut(Self::mark_invalid_if_needed);
        }
        Some(reset_vector.CommitAddedElements())
    }
    fn unparsed(nesting: CSSNestingType, text: AtomicString) -> CSSSelector {
        let mut selector = CSSSelector::default();
        selector.SetMatch(MatchType::kPseudoClass);
        selector.SetUnparsedPlaceholder(nesting, text);
        selector.SetLastInComplexSelector(true);
        selector
    }
    fn unparsed_range<T: TokenStreamTokenizer>(
        s: &CSSParserTokenStream<'_, T>,
        start: u32,
        nesting: CSSNestingType,
    ) -> CSSSelector {
        let value = s.StringRangeAt(start, s.LookAheadOffset() - start);
        let units = value.Span16();
        let first = units
            .iter()
            .position(|c| !foundation::unicode::IsSpaceOrNewline(*c))
            .unwrap_or(units.len());
        let last = units
            .iter()
            .rposition(|c| !foundation::unicode::IsSpaceOrNewline(*c))
            .map_or(first, |i| i + 1);
        Self::unparsed(nesting, AtomicString::from_utf16(&units[first..last]))
    }
    // cpp: css_parsing_utils.cc:1708-1773. RestoringBlockGuard leaves invalid
    // ident() untouched, including failed integer/math-value collaborator input.
    fn custom_ident_value<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<CSSCustomIdentValue> {
        if self.options.features.ident_function
            && s.Peek().GetType() == kFunctionToken
            && s.Peek().FunctionId() == Some(CSSValueID::kIdent)
        {
            let mut guard = super::css_parser_token_stream::RestoringBlockGuard::new(s);
            guard.ConsumeWhitespace();
            let mut values = Vec::new();
            while !guard.AtEnd() {
                if let Some(value) = self.custom_ident_value(&mut guard) {
                    values.push(CSSIdentComponent::CustomIdent(value));
                    continue;
                }
                if guard.Peek().GetType() == kStringToken {
                    values.push(CSSIdentComponent::String(
                        guard.ConsumeIncludingWhitespace().Value().ToAtomicString(),
                    ));
                    continue;
                }
                let token = guard.Peek().clone();
                let integer = if token.GetType() == kNumberToken
                    && token.GetNumericValueType() == NumericValueType::kIntegerValueType
                {
                    guard.ConsumeIncludingWhitespace();
                    Some(SelectorIdentIntegerValue::Literal(token.NumericValue()))
                } else if token.GetType() == kFunctionToken {
                    if let Some(parser) = self.options.ident_integer_parser {
                        let mut tokens = Vec::new();
                        Self::take_component_tokens(&mut guard, &mut tokens);
                        let value = parser.ParseInteger(&tokens);
                        guard.ConsumeWhitespace();
                        value
                    } else {
                        None
                    }
                } else {
                    None
                }?;
                if matches!(&integer, SelectorIdentIntegerValue::Expression(expression) if expression.HasRandomFunctions())
                {
                    return None;
                }
                values.push(CSSIdentComponent::Integer(integer));
            }
            if values.is_empty() || !guard.Release() {
                return None;
            }
            drop(guard);
            s.ConsumeWhitespace();
            return Some(CSSCustomIdentValue::IdentFunction(values));
        }
        if s.Peek().GetType() != kIdentToken
            || matches!(
                s.Peek().Id(),
                CSSValueID::kInherit
                    | CSSValueID::kInitial
                    | CSSValueID::kUnset
                    | CSSValueID::kRevert
                    | CSSValueID::kRevertLayer
                    | CSSValueID::kDefault
            )
        {
            return None;
        }
        Some(CSSCustomIdentValue::Literal(
            s.ConsumeIncludingWhitespace().Value().ToAtomicString(),
        ))
    }
    fn take_component_tokens<T: TokenStreamTokenizer>(
        s: &mut CSSParserTokenStream<'_, T>,
        tokens: &mut Vec<super::css_parser_token::CSSParserToken>,
    ) {
        let token = s.Peek().clone();
        if token.GetBlockType() == BlockType::kBlockStart {
            tokens.push(token);
            let mut guard = BlockGuard::new(s);
            while !guard.AtEnd() {
                Self::take_component_tokens(&mut guard, tokens);
            }
            tokens.push(guard.Peek().clone());
        } else {
            tokens.push(s.Consume().clone());
        }
    }
    fn custom_ident<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<AtomicString> {
        self.custom_ident_value(s).map(|value| value.Value())
    }
    // cpp: css_selector_parser.cc:144-157
    fn mark_invalid_if_needed(selectors: &mut [CSSSelector]) {
        if selectors.iter().all(CSSSelector::IsUnparsedInvalid) {
            if let Some(first) = selectors.first_mut() {
                first.SetMatch(MatchType::kInvalidList);
            }
        }
    }
    // cpp: .cc:716-745,853-998. Shared arena stores compounds in parse order;
    // two reversals preserve simple-selector order and reverse complex order.
    fn complex<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
        relative: bool,
        nesting: CSSNestingType,
        result_flags: &mut ResultFlags,
    ) -> Option<Range<usize>> {
        if nesting > CSSNestingType::kNesting {
            return None;
        }
        let mut reset_vector = ResetVectorAfterScope::new(&self.output);
        let mut relation = RelationType::kSubSelector;
        let nested_relative =
            !relative && nesting != CSSNestingType::kNone && self.peek_is_combinator(s);
        if nested_relative {
            let anchor = self.implicit_anchor(nesting);
            self.output.borrow_mut().push(anchor);
            result_flags.contains_scope_or_parent = true;
            relation = self.combinator(s);
        } else if relative {
            let mut anchor = CSSSelector::default();
            anchor.SetMatch(MatchType::kPseudoClass);
            anchor.SetPseudoType(PseudoType::kPseudoRelativeAnchor);
            anchor.SetValue(AtomicString::from_str("-internal-relative-anchor"), false);
            self.output.borrow_mut().push(anchor);
            relation = ConvertRelationToRelative(self.combinator(s));
        }
        let mut previous_rightmost = false;
        let mut compounds = 0;
        loop {
            let compound_start = s.LookAheadOffset();
            let compound = match self.compound(s, result_flags) {
                Some(c) => c,
                None => {
                    s.EnsureLookAhead();
                    if relation == RelationType::kDescendant
                        && (compounds > 0 || relative || nested_relative)
                        && s.LookAheadOffset() == compound_start
                    {
                        break;
                    }
                    return None;
                }
            };
            if previous_rightmost {
                return None;
            }
            {
                let mut arena = self.output.borrow_mut();
                let selectors = &mut arena[compound];
                if compounds > 0 || relative || nested_relative {
                    selectors.last_mut()?.SetRelation(relation);
                }
                previous_rightmost = selectors.iter().any(|v| {
                    v.Match() == MatchType::kPseudoElement
                        && !(self.options.ua_sheet_mode
                            && v.GetPseudoType() == PseudoType::kPseudoWebKitCustomElement)
                });
                ReverseSelectors(selectors);
            }
            compounds += 1;
            relation = self.combinator(s);
            if !relative
                && !nested_relative
                && compounds == 1
                && relation != RelationType::kSubSelector
            {
                result_flags.contains_complex = true;
            }
            if relation == RelationType::kSubSelector {
                break;
            }
        }
        let range = reset_vector.AddedElements();
        ReverseSelectors(&mut self.output.borrow_mut()[range.clone()]);
        if !relative
            && nesting != CSSNestingType::kNone
            && Self::nesting_type(&self.output.borrow()[range.clone()]) < nesting
        {
            let anchor = self.implicit_anchor(nesting);
            self.output
                .borrow_mut()
                .last_mut()?
                .SetRelation(RelationType::kDescendant);
            self.output.borrow_mut().push(anchor);
            result_flags.contains_scope_or_parent = true;
        }
        let range = reset_vector.AddedElements();
        MarkAsEntireComplexSelector(&mut self.output.borrow_mut()[range]);
        Some(reset_vector.CommitAddedElements())
    }
    fn compound<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
        result_flags: &mut ResultFlags,
    ) -> Option<Range<usize>> {
        let old_restricting = self.restricting_pseudo_element;
        let old_host = std::mem::replace(&mut self.found_host_in_compound, false);
        let result = self.compound_inner(s, result_flags);
        self.restricting_pseudo_element = old_restricting;
        self.found_host_in_compound = old_host;
        result
    }
    fn compound_inner<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
        result_flags: &mut ResultFlags,
    ) -> Option<Range<usize>> {
        let mut reset_vector = ResetVectorAfterScope::new(&self.output);
        let name = self.name(s);
        if name.is_some() && self.restricting_pseudo_element != PseudoType::kPseudoUnknown {
            return None;
        }

        loop {
            let token = s.Peek().clone();
            let mut selector = CSSSelector::default();
            let mut local_flags = ResultFlags::default();
            match token.GetType() {
                kHashToken => {
                    if token.GetHashTokenType() != HashTokenType::kHashTokenId {
                        return None;
                    }
                    s.Consume();
                    selector.SetMatch(MatchType::kId);
                    selector.SetValue(token.Value().ToAtomicString(), self.context.quirks);
                }
                kDelimiterToken if token.Delimiter() == 46 => {
                    s.Consume();
                    let token = s.Peek().clone();
                    if token.GetType() != kIdentToken {
                        return None;
                    }
                    s.Consume();
                    selector.SetMatch(MatchType::kClass);
                    selector.SetValue(token.Value().ToAtomicString(), self.context.quirks);
                }
                kLeftBracketToken => {
                    selector = self.attribute(s)?;
                }
                kColonToken => {
                    selector = self.pseudo(s, &mut local_flags)?;
                    local_flags.contains_pseudo = true;
                }
                kDelimiterToken if token.Delimiter() == 38 => {
                    s.Consume();
                    selector = CSSSelector::FromParent(
                        self.options.parent_rule_for_nesting.clone(),
                        false,
                    );
                    local_flags.contains_scope_or_parent = true;
                    local_flags.contains_pseudo = true;
                    local_flags.contains_complex = true;
                }
                _ => break,
            }
            if !self.options.ua_sheet_mode
                && !self.simple_valid_after_pseudo(&selector, self.restricting_pseudo_element)
            {
                return None;
            }
            if selector.Match() == MatchType::kPseudoElement {
                self.restricting_pseudo_element = selector.GetPseudoType();
            }
            if local_flags.contains_scope_or_parent {
                selector.SetScopeContaining(true);
            }
            result_flags.merge(&local_flags);
            self.output.borrow_mut().push(selector);
        }
        if self.found_host_in_compound {
            let range = reset_vector.AddedElements();
            for selector in &mut self.output.borrow_mut()[range] {
                if selector.GetPseudoType() == PseudoType::kPseudoHas {
                    selector.SetHasArgumentMatchInShadowTree();
                }
            }
        }
        if reset_vector.AddedElements().is_empty() && name.is_none() {
            return None;
        }
        let ignore_namespace = self.resist_default_namespace && name.is_none() && {
            s.EnsureLookAhead();
            let state = s.Save();
            s.ConsumeWhitespace();
            let at_end = s.AtEnd();
            s.EnsureLookAhead();
            s.Restore(state);
            at_end
        };
        let default_namespace = if self.ignore_default_namespace || ignore_namespace {
            AtomicString::from_str("*")
        } else {
            self.default_namespace()
        };
        let range = reset_vector.AddedElements();
        let tag =
            self.type_selector_if_needed(name, &self.output.borrow()[range], default_namespace)?;
        if let Some(tag) = tag {
            self.output
                .borrow_mut()
                .insert(reset_vector.initial_size, tag);
        }
        reset_vector.WithAddedElementsMut(|selectors| self.split_implicit_combinator(selectors));
        Some(reset_vector.CommitAddedElements())
    }
    // cpp: css_selector_parser.cc:2354-2403
    fn type_selector_if_needed(
        &self,
        name: Option<(AtomicString, AtomicString)>,
        selectors: &[CSSSelector],
        default_namespace: AtomicString,
    ) -> Option<Option<CSSSelector>> {
        let first = selectors.first();
        let needs_combinator = first.is_some_and(|s| {
            self.implicit_combinator(s.GetPseudoType()) != RelationType::kSubSelector
        });
        if name.is_none() && default_namespace == AtomicString::from_str("*") && !needs_combinator {
            return Some(None);
        }
        let has_q_name = name.is_some();
        let (name, mut prefix) =
            name.unwrap_or_else(|| (UniversalSelectorAtom().clone(), AtomicString::default()));
        let namespace = if prefix.IsNull() {
            default_namespace.clone()
        } else {
            match self.namespace(&prefix, false) {
                Some(uri) => uri,
                None => {
                    if selectors.is_empty() {
                        self.count(SelectorWebFeature::kCSSUnknownNamespacePrefixInSelector);
                    }
                    return None;
                }
            }
        };
        if namespace == default_namespace {
            prefix = AtomicString::default();
        }
        let name = if self.context.html {
            name.ToAsciiLower()
        } else {
            name
        };
        let tag = QualifiedName::new(prefix, name, namespace);
        let is_host = first.is_some_and(|s| {
            matches!(
                s.GetPseudoType(),
                PseudoType::kPseudoHost | PseudoType::kPseudoHostContext
            )
        });
        if is_host && !has_q_name && tag.Prefix().IsNull() {
            return Some(None);
        }
        if selectors.is_empty() || tag != QualifiedName::AnyQName() || is_host || needs_combinator {
            let implicit = tag.Prefix().IsNull()
                && tag.LocalName() == UniversalSelectorAtom()
                && !is_host
                && !selectors.is_empty();
            Some(Some(CSSSelector::FromTag(tag, implicit)))
        } else {
            Some(None)
        }
    }
    // cpp: css_selector_parser.cc:65-98
    fn implicit_combinator(&self, pseudo: PseudoType) -> RelationType {
        use PseudoType::*;
        match pseudo {
            kPseudoSlotted => RelationType::kShadowSlot,
            kPseudoWebKitCustomElement
            | kPseudoBlinkInternalElement
            | kPseudoCue
            | kPseudoDetailsContent
            | kPseudoPlaceholder
            | kPseudoFileSelectorButton
            | kPseudoPicker
            | kPseudoPermissionIcon
            | kPseudoSelectListbox => RelationType::kUAShadow,
            kPseudoPart => RelationType::kShadowPart,
            kPseudoBefore | kPseudoAfter | kPseudoMarker
                if self.options.features.logical_combination_pseudo =>
            {
                RelationType::kPseudoChild
            }
            _ => RelationType::kSubSelector,
        }
    }
    // cpp: css_selector_parser.cc:2431-2473
    fn split_implicit_combinator(&self, selectors: &mut [CSSSelector]) {
        for i in 1..selectors.len() {
            let relation = self.implicit_combinator(selectors[i].GetPseudoType());
            if relation != RelationType::kSubSelector {
                selectors.rotate_left(i);
                let remaining_length = selectors.len() - i;
                let remaining = &mut selectors[..remaining_length];
                self.split_implicit_combinator(remaining);
                remaining.last_mut().unwrap().SetRelation(relation);
                break;
            }
        }
    }
    // cpp: css_selector_parser.cc:1257-1300
    fn simple_valid_after_pseudo(&self, selector: &CSSSelector, restricting: PseudoType) -> bool {
        use PseudoType::*;
        if restricting == kPseudoColumn {
            return selector.GetPseudoType() == kPseudoScrollMarker;
        }
        if restricting == kPseudoUnknown {
            return true;
        }
        if matches!(restricting, kPseudoBefore | kPseudoAfter)
            && selector.GetPseudoType() == kPseudoMarker
            && self.options.features.marker_nested_pseudo_element
        {
            return true;
        }
        if restricting == kPseudoSlotted {
            return selector.IsTreeAbidingPseudoElement();
        }
        if (restricting == kPseudoPart || IsElementBackedPseudoElement(restricting))
            && selector.IsAllowedAfterPart()
        {
            return true;
        }
        if selector.Match() != MatchType::kPseudoClass {
            return false;
        }
        if matches!(
            selector.GetPseudoType(),
            kPseudoIs | kPseudoWhere | kPseudoNot
        ) {
            return true;
        }
        self.pseudo_class_valid_after(selector.GetPseudoType(), restricting)
    }
    // cpp: css_selector_parser.cc:1150-1260
    fn pseudo_class_valid_after(&self, pseudo: PseudoType, restricting: PseudoType) -> bool {
        use PseudoType::*;
        let user_action = matches!(
            pseudo,
            kPseudoHover | kPseudoFocus | kPseudoFocusVisible | kPseudoFocusWithin | kPseudoActive
        );
        if user_action && matches!(restricting, kPseudoScrollButton | kPseudoScrollMarker) {
            return true;
        }
        match restricting {
            kPseudoResizer
            | kPseudoScrollbar
            | kPseudoScrollbarCorner
            | kPseudoScrollbarButton
            | kPseudoScrollbarThumb
            | kPseudoScrollbarTrack
            | kPseudoScrollbarTrackPiece => matches!(
                pseudo,
                kPseudoEnabled
                    | kPseudoDisabled
                    | kPseudoHover
                    | kPseudoActive
                    | kPseudoHorizontal
                    | kPseudoVertical
                    | kPseudoDecrement
                    | kPseudoIncrement
                    | kPseudoStart
                    | kPseudoEnd
                    | kPseudoDoubleButton
                    | kPseudoSingleButton
                    | kPseudoNoButton
                    | kPseudoCornerPresent
                    | kPseudoWindowInactive
            ),
            kPseudoSelection => pseudo == kPseudoWindowInactive,
            kPseudoWebKitCustomElement
            | kPseudoBlinkInternalElement
            | kPseudoFileSelectorButton => user_action,
            kPseudoViewTransitionGroup
            | kPseudoViewTransitionGroupChildren
            | kPseudoViewTransitionImagePair
            | kPseudoViewTransitionOld
            | kPseudoViewTransitionNew => pseudo == kPseudoOnlyChild,
            kPseudoSearchText => pseudo == kPseudoCurrent,
            kPseudoScrollMarkerGroup => matches!(pseudo, kPseudoFocusWithin | kPseudoHover),
            kPseudoScrollMarker => matches!(
                pseudo,
                kPseudoTargetCurrent | kPseudoTargetBefore | kPseudoTargetAfter
            ),
            kPseudoScrollButton => matches!(pseudo, kPseudoDisabled | kPseudoEnabled),
            kPseudoAfter | kPseudoBefore | kPseudoMarker => {
                pseudo == kPseudoHover && self.options.features.pseudo_elements_hoverable
            }
            _ => false,
        }
    }
    fn attribute<T: TokenStreamTokenizer>(
        &self,
        s: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<CSSSelector> {
        let mut s = BlockGuard::new(s);
        s.ConsumeWhitespace();
        let (name, prefix) = self.name(&mut s)?;
        if name == *UniversalSelectorAtom() {
            return None;
        }
        let namespace = self.namespace(&prefix, true)?;
        let name = if self.context.html {
            name.ToAsciiLower()
        } else {
            name
        };
        let q = QualifiedName::new(prefix, name, namespace);
        s.ConsumeWhitespace();
        if s.AtEnd() {
            return Some(CSSSelector::FromAttribute(
                MatchType::kAttributeSet,
                q,
                AttributeMatchType::kCaseSensitive,
                None,
                self.context,
            ));
        }
        let kind = match s.Peek().GetType() {
            kIncludeMatchToken => MatchType::kAttributeList,
            kDashMatchToken => MatchType::kAttributeHyphen,
            kPrefixMatchToken => MatchType::kAttributeBegin,
            kSuffixMatchToken => MatchType::kAttributeEnd,
            kSubstringMatchToken => MatchType::kAttributeContain,
            kDelimiterToken if s.Peek().Delimiter() == 61 => MatchType::kAttributeExact,
            _ => return None,
        };
        s.ConsumeIncludingWhitespace();
        let value = s.Peek().clone();
        if !matches!(value.GetType(), kIdentToken | kStringToken) {
            return None;
        }
        s.ConsumeIncludingWhitespace();
        let mut sensitivity = AttributeMatchType::kCaseSensitive;
        if s.Peek().GetType() == kIdentToken {
            sensitivity = match s
                .ConsumeIncludingWhitespace()
                .Value()
                .ToAtomicString()
                .ToAsciiLower()
                .Utf8()
                .as_str()
            {
                "i" => AttributeMatchType::kCaseInsensitive,
                "s" if self.options.features.case_sensitive_selector => {
                    AttributeMatchType::kCaseSensitiveAlways
                }
                _ => return None,
            };
        }
        if !s.AtEnd() {
            return None;
        }
        Some(CSSSelector::FromAttribute(
            kind,
            q,
            sensitivity,
            Some(value.Value().ToAtomicString()),
            self.context,
        ))
    }
    fn pseudo<T: TokenStreamTokenizer>(
        &mut self,
        s: &mut CSSParserTokenStream<'_, T>,
        result_flags: &mut ResultFlags,
    ) -> Option<CSSSelector> {
        debug_assert_eq!(s.Peek().GetType(), kColonToken);
        s.Consume();
        let mut colons = 1;
        if s.Peek().GetType() == kColonToken {
            s.Consume();
            colons += 1;
        }
        let token = s.Peek().clone();
        if !matches!(token.GetType(), kIdentToken | kFunctionToken) {
            return None;
        }
        let mut selector = CSSSelector::default();
        selector.SetMatch(if colons == 1 {
            MatchType::kPseudoClass
        } else {
            MatchType::kPseudoElement
        });
        selector.UpdatePseudoType(
            token.Value().ToAtomicString(),
            &PseudoContext {
                context: self.context,
                runtime: self.options.runtime_context,
            },
            token.GetType() == kFunctionToken,
            self.options.ua_sheet_mode,
        );
        let pseudo = selector.GetPseudoType();
        // cpp: css_selector_parser.cc:1628-1648.
        if selector.Match() == MatchType::kPseudoElement {
            use SelectorWebFeature::*;
            match pseudo {
                PseudoType::kPseudoBefore | PseudoType::kPseudoAfter => {
                    self.count(kHasBeforeOrAfterPseudoElement)
                }
                PseudoType::kPseudoMarker if !self.options.ua_sheet_mode => {
                    self.count(kHasMarkerPseudoElement)
                }
                PseudoType::kPseudoSpellingError | PseudoType::kPseudoGrammarError
                    if !self.options.ua_sheet_mode =>
                {
                    self.count(kHasSpellingOrGrammarErrorPseudoElement)
                }
                _ => {}
            }
        }
        if pseudo == PseudoType::kPseudoUnknown
            || self.disallow_pseudo.get() && selector.Match() == MatchType::kPseudoElement
            || self.inside_has && pseudo == PseudoType::kPseudoHas
        {
            return None;
        }
        if pseudo == PseudoType::kPseudoScope {
            result_flags.contains_scope_or_parent = true;
        }
        if token.GetType() == kIdentToken {
            s.Consume();
            if pseudo == PseudoType::kPseudoHost {
                self.found_host_in_compound = true;
            }
            return Some(selector);
        }
        let mut s = BlockGuard::new(s);
        s.ConsumeWhitespace();
        match pseudo {
            PseudoType::kPseudoIs
            | PseudoType::kPseudoWhere
            | PseudoType::kPseudoNot
            | PseudoType::kPseudoHas => {
                let old_has = self.inside_has;
                let old_resist = self.resist_default_namespace;
                self.resist_default_namespace = true;
                let disallow_scope = (!self.options.features.logical_combination_pseudo
                    || pseudo == PseudoType::kPseudoHas)
                    .then(|| DisallowPseudoElementsScope::new(self));
                if pseudo == PseudoType::kPseudoHas {
                    self.inside_has = true;
                }
                let mut has_flags = ResultFlags::default();
                let flags = if pseudo == PseudoType::kPseudoHas {
                    &mut has_flags
                } else {
                    &mut *result_flags
                };
                let list = if self.inside_compound_pseudo && pseudo != PseudoType::kPseudoHas {
                    self.compound_list(
                        &mut s,
                        matches!(pseudo, PseudoType::kPseudoIs | PseudoType::kPseudoWhere),
                        flags,
                    )
                } else {
                    self.list(
                        &mut s,
                        matches!(pseudo, PseudoType::kPseudoIs | PseudoType::kPseudoWhere),
                        pseudo == PseudoType::kPseudoHas,
                        CSSNestingType::kNone,
                        flags,
                    )
                };
                self.inside_has = old_has;
                drop(disallow_scope);
                self.resist_default_namespace = old_resist;
                let list = self.adopt_range(list?);
                if pseudo == PseudoType::kPseudoHas
                    && (self.inside_compound_pseudo
                        || self.restricting_pseudo_element != PseudoType::kPseudoUnknown)
                {
                    return None;
                }
                if matches!(pseudo, PseudoType::kPseudoNot | PseudoType::kPseudoHas)
                    && !list.IsValid()
                {
                    return None;
                }
                if pseudo == PseudoType::kPseudoHas {
                    if has_flags.contains_pseudo {
                        selector.SetContainsPseudoInsideHasPseudoClass();
                    }
                    if has_flags.contains_complex {
                        selector.SetContainsComplexLogicalCombinationsInsideHasPseudoClass();
                    }
                    result_flags.merge(&has_flags);
                }
                selector.SetSelectorList(Some(list));
            }
            // cpp: css_selector_parser.cc:1714-1745,1915-1933.
            PseudoType::kPseudoHost
            | PseudoType::kPseudoHostContext
            | PseudoType::kPseudoAny
            | PseudoType::kPseudoCue
            | PseudoType::kPseudoSlotted => {
                if matches!(
                    pseudo,
                    PseudoType::kPseudoHost | PseudoType::kPseudoHostContext
                ) {
                    self.found_host_in_compound = true;
                }
                let disallow_scope = DisallowPseudoElementsScope::new(self);
                let old_compound = std::mem::replace(&mut self.inside_compound_pseudo, true);
                let old_ignore = self.ignore_default_namespace;
                self.ignore_default_namespace |= pseudo == PseudoType::kPseudoCue;
                let parsed = if pseudo == PseudoType::kPseudoSlotted {
                    self.compound(&mut s, result_flags).map(|mut compound| {
                        MarkAsEntireComplexSelector(
                            &mut self.output.borrow_mut()[compound.clone()],
                        );
                        compound
                    })
                } else {
                    self.compound_list(&mut s, false, result_flags)
                };
                drop(disallow_scope);
                self.inside_compound_pseudo = old_compound;
                self.ignore_default_namespace = old_ignore;
                let list = self.adopt_range(parsed?);
                if !list.IsValid()
                    || matches!(
                        pseudo,
                        PseudoType::kPseudoHost | PseudoType::kPseudoHostContext
                    ) && !list.IsSingleComplexSelector()
                {
                    return None;
                }
                selector.SetSelectorList(Some(list));
            }
            // cpp: css_selector_parser.cc:1785-1803.
            PseudoType::kPseudoPicker => {
                if s.Peek().GetType() != kIdentToken
                    || s.Peek().Value().ToAtomicString().ToAsciiLower()
                        != AtomicString::from_str("select")
                {
                    return None;
                }
                selector.SetArgument(AtomicString::from_str("select"));
                s.ConsumeIncludingWhitespace();
            }
            // cpp: css_selector_parser.cc:1818-1831.
            PseudoType::kPseudoPart => {
                let mut parts = vec![];
                loop {
                    if s.Peek().GetType() != kIdentToken {
                        return None;
                    }
                    parts.push(s.ConsumeIncludingWhitespace().Value().ToAtomicString());
                    if s.AtEnd() {
                        break;
                    }
                }
                selector.SetIdentList(Some(parts));
            }
            // cpp: css_selector_parser.cc:1832-1858.
            PseudoType::kPseudoActiveViewTransitionType => {
                let mut types = vec![];
                loop {
                    if s.Peek().GetType() != kIdentToken {
                        return None;
                    }
                    types.push(s.ConsumeIncludingWhitespace().Value().ToAtomicString());
                    if s.AtEnd() {
                        break;
                    }
                    if s.Peek().GetType() != kCommaToken {
                        return None;
                    }
                    s.ConsumeIncludingWhitespace();
                    if s.AtEnd() {
                        return None;
                    }
                }
                selector.SetIdentList(Some(types));
            }
            // cpp: css_selector_parser.cc:1859-1914. Typed custom-ident Value() semantics.
            PseudoType::kPseudoViewTransitionGroup
            | PseudoType::kPseudoViewTransitionGroupChildren
            | PseudoType::kPseudoViewTransitionImagePair
            | PseudoType::kPseudoViewTransitionOld
            | PseudoType::kPseudoViewTransitionNew => {
                let mut names = vec![];
                if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == 46 {
                    names.push(UniversalSelectorAtom().clone());
                }
                if names.is_empty() {
                    if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == 42 {
                        names.push(UniversalSelectorAtom().clone());
                        s.Consume();
                    } else {
                        names.push(self.custom_ident(&mut s)?);
                    }
                }
                while !s.AtEnd() && s.Peek().GetType() != kWhitespaceToken {
                    if s.Peek().GetType() != kDelimiterToken || s.Consume().Delimiter() != 46 {
                        return None;
                    }
                    names.push(self.custom_ident(&mut s)?);
                }
                selector.SetIdentList(Some(names));
            }
            // cpp: css_selector_parser.cc:2023-2033. The source intentionally
            // leaves the '*' token for BlockGuard to skip on return.
            PseudoType::kPseudoOverscrollAreaParent => {
                if s.Peek().GetType() != kDelimiterToken || s.Peek().Delimiter() != 42 {
                    return None;
                }
                selector.SetArgument(AtomicString::from_str("*"));
                return Some(selector);
            }
            // cpp: css_selector_parser.cc:125-142,2034-2056.
            PseudoType::kPseudoScrollButton => {
                let ident = s.Peek();
                if ident.GetType() == kIdentToken {
                    if !matches!(
                        ident.Id(),
                        CSSValueID::kUp
                            | CSSValueID::kDown
                            | CSSValueID::kLeft
                            | CSSValueID::kRight
                            | CSSValueID::kBlockStart
                            | CSSValueID::kBlockEnd
                            | CSSValueID::kInlineStart
                            | CSSValueID::kInlineEnd
                    ) {
                        return None;
                    }
                    selector.SetArgument(ident.Value().ToAtomicString());
                } else if ident.GetType() == kDelimiterToken && ident.Delimiter() == 42 {
                    selector.SetArgument(AtomicString::from_str("*"));
                } else {
                    return None;
                }
                s.ConsumeIncludingWhitespace();
            }
            // cpp: css_selector_parser.cc:2057-2070.
            PseudoType::kPseudoHighlight => {
                if s.Peek().GetType() != kIdentToken {
                    return None;
                }
                selector.SetArgument(s.ConsumeIncludingWhitespace().Value().ToAtomicString());
            }
            PseudoType::kPseudoLinkTo => {
                if !self.options.features.route_matching {
                    return None;
                }
                let location = NavigationParser::ParseLocation(&mut s)?;
                selector.SetNavigationLocation(Some(location));
            }
            PseudoType::kPseudoNthChild
            | PseudoType::kPseudoNthLastChild
            | PseudoType::kPseudoNthOfType
            | PseudoType::kPseudoNthLastOfType => {
                let (a, b) = Self::anb(&mut s)?;
                s.ConsumeWhitespace();
                let mut list = None;
                if !s.AtEnd()
                    && matches!(
                        pseudo,
                        PseudoType::kPseudoNthChild | PseudoType::kPseudoNthLastChild
                    )
                {
                    list = Some(self.nth_child_of_selectors(&mut s)?);
                }
                selector.SetNth(a, b, list);
            }
            PseudoType::kPseudoLang if !self.options.features.lang_extended_ranges => {
                if s.Peek().GetType() != kIdentToken {
                    return None;
                }
                selector.SetArgumentList(Some(vec![s
                    .ConsumeIncludingWhitespace()
                    .Value()
                    .ToAtomicString()]));
            }
            PseudoType::kPseudoLang => {
                let mut arguments = vec![];
                loop {
                    let token = s.Peek().clone();
                    if !matches!(token.GetType(), kIdentToken | kStringToken) {
                        return None;
                    }
                    arguments.push(token.Value().ToAtomicString());
                    s.ConsumeIncludingWhitespace();
                    if s.AtEnd() {
                        break;
                    }
                    if s.Peek().GetType() != kCommaToken {
                        return None;
                    }
                    s.ConsumeIncludingWhitespace();
                }
                selector.SetArgumentList(Some(arguments));
            }
            PseudoType::kPseudoDir | PseudoType::kPseudoState => {
                if s.Peek().GetType() != kIdentToken {
                    return None;
                }
                selector.SetArgument(s.ConsumeIncludingWhitespace().Value().ToAtomicString());
            }
            _ => return None,
        }
        s.ConsumeWhitespace();
        if !s.AtEnd() {
            return None;
        }
        Some(selector)
    }
    // cpp: css_selector_parser.cc:2206-2307
    fn anb<T: TokenStreamTokenizer>(s: &mut CSSParserTokenStream<'_, T>) -> Option<(i32, i32)> {
        if s.AtEnd() || s.Peek().GetBlockType() != BlockType::kNotBlock {
            return None;
        }
        let token = s.Consume().clone();
        if token.GetType() == kNumberToken
            && token.GetNumericValueType() == NumericValueType::kIntegerValueType
        {
            return Some((0, token.NumericValue() as i32));
        }
        if token.GetType() == kIdentToken {
            match token
                .Value()
                .ToAtomicString()
                .ToAsciiLower()
                .Utf8()
                .as_str()
            {
                "odd" => return Some((2, 1)),
                "even" => return Some((2, 0)),
                _ => {}
            }
        }
        let (a, n) = if token.GetType() == kDelimiterToken
            && token.Delimiter() == 43
            && s.Peek().GetType() == kIdentToken
        {
            (1, s.Consume().Value().ToAtomicString().Utf8())
        } else if token.GetType() == kDimensionToken
            && token.GetNumericValueType() == NumericValueType::kIntegerValueType
        {
            (
                token.NumericValue() as i32,
                token.Value().ToAtomicString().Utf8(),
            )
        } else if token.GetType() == kIdentToken {
            let n = token.Value().ToAtomicString().Utf8();
            if let Some(n) = n.strip_prefix('-') {
                (-1, n.to_string())
            } else {
                (1, n)
            }
        } else {
            return None;
        };
        s.ConsumeWhitespace();
        let n = n.to_ascii_lowercase();
        if !n.starts_with('n') || n.len() > 1 && n.as_bytes()[1] != b'-' {
            return None;
        }
        if n.len() > 2 {
            return Some((a, n[1..].parse().ok()?));
        }
        let mut sign = if n.len() == 1 {
            NumericSign::kNoSign
        } else {
            NumericSign::kMinusSign
        };
        if sign == NumericSign::kNoSign && s.Peek().GetType() == kDelimiterToken {
            sign = match s.ConsumeIncludingWhitespace().Delimiter() {
                43 => NumericSign::kPlusSign,
                45 => NumericSign::kMinusSign,
                _ => return None,
            };
        }
        if sign == NumericSign::kNoSign && s.Peek().GetType() != kNumberToken {
            return Some((a, 0));
        }
        let b = s.Peek().clone();
        if b.GetType() != kNumberToken
            || b.GetNumericValueType() != NumericValueType::kIntegerValueType
            || (b.GetNumericSign() == NumericSign::kNoSign) == (sign == NumericSign::kNoSign)
        {
            return None;
        }
        s.Consume();
        let b = b.NumericValue() as i32;
        Some((
            a,
            if sign == NumericSign::kMinusSign {
                b.checked_neg().unwrap_or(i32::MAX)
            } else {
                b
            },
        ))
    }
}
// cpp: .cc:2474-2488. An enum represents WebFeature without an unchecked
// integer conversion; both its representation and the length are 16 bits.
struct PseudoElementFeatureMapEntry {
    key: &'static str,
    key_length: u16,
    feature: SelectorWebFeature,
}
impl PseudoElementFeatureMapEntry {
    const fn new(key: &'static str, feature: SelectorWebFeature) -> Self {
        assert!(key.len() <= u16::MAX as usize);
        Self {
            key,
            key_length: key.len() as u16,
            feature,
        }
    }
}
pub struct CSSSelectorParser;
impl CSSSelectorParser {
    // cpp: css_selector_parser.cc:992-1011.
    pub fn ParsePseudoType(
        name: &AtomicString,
        has_arguments: bool,
        runtime: &impl CSSSelectorRuntime,
    ) -> PseudoType {
        let pseudo = CSSSelector::NameToPseudoType(name, has_arguments, runtime);
        if pseudo != PseudoType::kPseudoUnknown {
            return pseudo;
        }
        if name.Utf8().starts_with("-webkit-") {
            return PseudoType::kPseudoWebKitCustomElement;
        }
        if name.Utf8().starts_with("-internal-") {
            return PseudoType::kPseudoBlinkInternalElement;
        }
        PseudoType::kPseudoUnknown
    }
    // cpp: css_selector_parser.cc:1014-1144. Node/exposure and use counters
    // are supplied by the caller; token parsing uses the same selector parser.
    pub fn ParsePseudoElement(
        text: &String,
        parent: &impl CSSSelectorPseudoElementContext,
        features: SelectorParserFeatures,
        argument: &mut AtomicString,
    ) -> PseudoId {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions {
            features,
            runtime_context: Some(parent.Runtime()),
            ..SelectorParserOptions::default()
        };
        {
            let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(text), 0);
            stream.EnsureLookAhead();
            let mut colons = 0;
            if stream.Peek().GetType() == kColonToken {
                stream.Consume();
                colons += 1;
            }
            if stream.Peek().GetType() == kColonToken {
                stream.Consume();
                colons += 1;
            }
            let token = stream.Peek().clone();
            if token.GetType() == kIdentToken {
                stream.Consume();
                if token.Value().Span16().iter().any(|c| *c > 127)
                    || stream.Peek().GetType() != kEOFToken
                {
                    return PseudoId::kPseudoIdInvalid;
                }
                let name = token.Value().ToAtomicString().ToAsciiLower();
                let pseudo = Self::ParsePseudoType(
                    &name,
                    false,
                    &PseudoContext {
                        context: &context,
                        runtime: options.runtime_context,
                    },
                );
                let id = GetPseudoId(pseudo);
                if matches!(
                    id,
                    PseudoId::kPseudoIdBefore
                        | PseudoId::kPseudoIdAfter
                        | PseudoId::kPseudoIdFirstLetter
                        | PseudoId::kPseudoIdFirstLine
                ) {
                    if colons == 0 {
                        if parent.HasParent() {
                            parent.CountPseudoElementWithoutColon();
                        }
                        if parent.CSSOMGetComputedStylePseudoElementRequiresColonEnabled() {
                            return PseudoId::kPseudoIdNone;
                        }
                    }
                    return id;
                }
                if matches!(
                    pseudo,
                    PseudoType::kPseudoWebKitCustomElement
                        | PseudoType::kPseudoBlinkInternalElement
                ) && colons == 2
                {
                    return PseudoId::kPseudoIdNone;
                }
            }
            if colons != 2 {
                return if colons == 1 {
                    PseudoId::kPseudoIdInvalid
                } else {
                    PseudoId::kPseudoIdNone
                };
            }
        }
        let mut parser = Parser::new(&context, &options, false);
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(text), 0);
        let Some(selector) = parser.pseudo(&mut stream, &mut ResultFlags::default()) else {
            return PseudoId::kPseudoIdInvalid;
        };
        if !stream.AtEnd() || selector.Match() != MatchType::kPseudoElement {
            return PseudoId::kPseudoIdInvalid;
        }
        let id = GetPseudoId(selector.GetPseudoType());
        if !parent.IsWebExposed(id) {
            return PseudoId::kPseudoIdInvalid;
        }
        match id {
            PseudoId::kPseudoIdScrollButton | PseudoId::kPseudoIdHighlight => {
                *argument = (*selector.Argument()).clone()
            }
            PseudoId::kPseudoIdViewTransitionGroup
            | PseudoId::kPseudoIdViewTransitionGroupChildren
            | PseudoId::kPseudoIdViewTransitionImagePair
            | PseudoId::kPseudoIdViewTransitionOld
            | PseudoId::kPseudoIdViewTransitionNew => {
                let names = selector.IdentList();
                if names.len() != 1 || names[0] == *UniversalSelectorAtom() {
                    return PseudoId::kPseudoIdInvalid;
                }
                *argument = names[0].clone();
            }
            _ => {}
        }
        id
    }
    // cpp: .cc:2474-2603. Typed features retain the checked 16-bit table
    // representation; source key lengths avoid allocating an UTF-8 lookup key.
    fn FeatureForWebKitCustomPseudoElement(name: &AtomicString) -> SelectorWebFeature {
        const TABLE: &[PseudoElementFeatureMapEntry] = &[
            PseudoElementFeatureMapEntry::new("cue", SelectorWebFeature::kCSSSelectorCue),
            PseudoElementFeatureMapEntry::new(
                "-internal-media-controls-overlay-cast-button",
                SelectorWebFeature::kCSSSelectorInternalMediaControlsOverlayCastButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-calendar-picker-indicator",
                SelectorWebFeature::kCSSSelectorWebkitCalendarPickerIndicator,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-clear-button",
                SelectorWebFeature::kCSSSelectorWebkitClearButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-color-swatch",
                SelectorWebFeature::kCSSSelectorWebkitColorSwatch,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-color-swatch-wrapper",
                SelectorWebFeature::kCSSSelectorWebkitColorSwatchWrapper,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-date-and-time-value",
                SelectorWebFeature::kCSSSelectorWebkitDateAndTimeValue,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEdit,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-ampm-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditAmpmField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-day-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditDayField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-fields-wrapper",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditFieldsWrapper,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-hour-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditHourField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-millisecond-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditMillisecondField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-minute-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditMinuteField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-month-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditMonthField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-second-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditSecondField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-text",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditText,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-week-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditWeekField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-datetime-edit-year-field",
                SelectorWebFeature::kCSSSelectorWebkitDatetimeEditYearField,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-file-upload-button",
                SelectorWebFeature::kCSSSelectorWebkitFileUploadButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-inner-spin-button",
                SelectorWebFeature::kCSSSelectorWebkitInnerSpinButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-input-placeholder",
                SelectorWebFeature::kCSSSelectorWebkitInputPlaceholder,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls",
                SelectorWebFeature::kCSSSelectorWebkitMediaControls,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-current-time-display",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsCurrentTimeDisplay,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-enclosure",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsEnclosure,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-fullscreen-button",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsFullscreenButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-mute-button",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsMuteButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-overlay-enclosure",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsOverlayEnclosure,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-overlay-play-button",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsOverlayPlayButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-panel",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsPanel,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-play-button",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsPlayButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-timeline",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsTimeline,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-timeline-container",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsTimelineContainer,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-time-remaining-display",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsTimeRemainingDisplay,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-toggle-closed-captions-button",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsToggleClosedCaptionsButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-controls-volume-slider",
                SelectorWebFeature::kCSSSelectorWebkitMediaControlsVolumeSlider,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-slider-container",
                SelectorWebFeature::kCSSSelectorWebkitMediaSliderContainer,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-slider-thumb",
                SelectorWebFeature::kCSSSelectorWebkitMediaSliderThumb,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-text-track-container",
                SelectorWebFeature::kCSSSelectorWebkitMediaTextTrackContainer,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-text-track-display",
                SelectorWebFeature::kCSSSelectorWebkitMediaTextTrackDisplay,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-text-track-region",
                SelectorWebFeature::kCSSSelectorWebkitMediaTextTrackRegion,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-media-text-track-region-container",
                SelectorWebFeature::kCSSSelectorWebkitMediaTextTrackRegionContainer,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-meter-bar",
                SelectorWebFeature::kCSSSelectorWebkitMeterBar,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-meter-even-less-good-value",
                SelectorWebFeature::kCSSSelectorWebkitMeterEvenLessGoodValue,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-meter-inner-element",
                SelectorWebFeature::kCSSSelectorWebkitMeterInnerElement,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-meter-optimum-value",
                SelectorWebFeature::kCSSSelectorWebkitMeterOptimumValue,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-meter-suboptimum-value",
                SelectorWebFeature::kCSSSelectorWebkitMeterSuboptimumValue,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-progress-bar",
                SelectorWebFeature::kCSSSelectorWebkitProgressBar,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-progress-inner-element",
                SelectorWebFeature::kCSSSelectorWebkitProgressInnerElement,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-progress-value",
                SelectorWebFeature::kCSSSelectorWebkitProgressValue,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-search-cancel-button",
                SelectorWebFeature::kCSSSelectorWebkitSearchCancelButton,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-slider-container",
                SelectorWebFeature::kCSSSelectorWebkitSliderContainer,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-slider-runnable-track",
                SelectorWebFeature::kCSSSelectorWebkitSliderRunnableTrack,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-slider-thumb",
                SelectorWebFeature::kCSSSelectorWebkitSliderThumb,
            ),
            PseudoElementFeatureMapEntry::new(
                "-webkit-textfield-decoration-container",
                SelectorWebFeature::kCSSSelectorWebkitTextfieldDecorationContainer,
            ),
        ];
        TABLE
            .iter()
            .find(|entry| {
                name.utf16_units().unwrap_or(&[]).len() == entry.key_length as usize
                    && name
                        .utf16_units()
                        .unwrap_or(&[])
                        .iter()
                        .zip(entry.key.as_bytes())
                        .all(|(a, b)| *a == *b as u16)
            })
            .map_or(
                SelectorWebFeature::kCSSSelectorWebkitUnknownPseudo,
                |entry| entry.feature,
            )
    }
    fn IsKnownWebKitCustomPseudoElement(name: &AtomicString) -> bool {
        Self::FeatureForWebKitCustomPseudoElement(name)
            != SelectorWebFeature::kCSSSelectorWebkitUnknownPseudo
    }
    // cpp: css_selector_parser.cc:2607-2782.
    fn record_one(
        selector: &CSSSelector,
        context: &dyn CSSSelectorUsageContext,
        nesting: CSSNestingType,
        has_visited: &mut Option<&mut bool>,
    ) {
        use PseudoType::*;
        use SelectorWebDXFeature::*;
        use SelectorWebFeature::*;
        let feature = match selector.GetPseudoType() {
            kPseudoAny => Some(kCSSSelectorPseudoAny),
            kPseudoIs => Some(kCSSSelectorPseudoIs),
            kPseudoFocusVisible => Some(kCSSSelectorPseudoFocusVisible),
            kPseudoFocus => Some(kCSSSelectorPseudoFocus),
            kPseudoAnyLink => Some(kCSSSelectorPseudoAnyLink),
            kPseudoWebkitAnyLink => Some(kCSSSelectorPseudoWebkitAnyLink),
            kPseudoWhere => Some(kCSSSelectorPseudoWhere),
            kPseudoDefined => Some(kCSSSelectorPseudoDefined),
            kPseudoSlotted => Some(kCSSSelectorPseudoSlotted),
            kPseudoHost => Some(kCSSSelectorPseudoHost),
            kPseudoHostContext => Some(kCSSSelectorPseudoHostContext),
            kPseudoFullScreenAncestor => Some(kCSSSelectorPseudoFullScreenAncestor),
            kPseudoFullScreen => Some(kCSSSelectorPseudoFullScreen),
            kPseudoListBox => Some(kCSSSelectorInternalPseudoListBox),
            kPseudoSpatialNavigationFocus => Some(kCSSSelectorInternalPseudoSpatialNavigationFocus),
            kPseudoReadOnly => Some(kCSSSelectorPseudoReadOnly),
            kPseudoReadWrite => Some(kCSSSelectorPseudoReadWrite),
            kPseudoDir => Some(kCSSSelectorPseudoDir),
            kPseudoHas => Some(kCSSSelectorPseudoHas),
            kPseudoHasSlotted => Some(kCSSSelectorPseudoHasSlotted),
            kPseudoState => Some(kCSSSelectorPseudoState),
            kPseudoUserValid => Some(kCSSSelectorUserValid),
            kPseudoUserInvalid => Some(kCSSSelectorUserInvalid),
            kPseudoModal => Some(kCSSSelectorPseudoModal),
            kPseudoFileSelectorButton => Some(kCSSSelectorPseudoFileSelectorButton),
            kPseudoActiveViewTransition => Some(kActiveViewTransitionPseudo),
            kPseudoOpen => Some(kCSSPseudoOpen),
            kPseudoNot => Some(kCSSSelectorPseudoNot),
            kPseudoWebKitCustomElement => {
                Some(Self::FeatureForWebKitCustomPseudoElement(&selector.Value()))
            }
            kPseudoNthChild if selector.SelectorList().is_some() => {
                Some(kCSSSelectorNthChildOfSelector)
            }
            kPseudoParent if nesting == CSSNestingType::kScope => Some(kCSSPseudoParentInScope),
            kPseudoVisited => {
                if let Some(visited) = has_visited.as_deref_mut() {
                    *visited = true;
                }
                None
            }
            _ => None,
        };
        let webdx = match selector.GetPseudoType() {
            kPseudoAutofill => Some(kAutofill),
            kPseudoDetailsContent => Some(kDetailsContent),
            kPseudoPastCue | kPseudoFutureCue => Some(kTimeRelativeSelectors),
            kPseudoPlaying | kPseudoPaused | kPseudoSeeking | kPseudoBuffering | kPseudoStalled
            | kPseudoMuted | kPseudoVolumeLocked => Some(kMediaPseudos),
            _ => None,
        };
        if let Some(feature) = feature {
            if context.IsDeprecated(feature) {
                context.CountDeprecation(feature);
            } else {
                context.Count(feature);
            }
        }
        if let Some(webdx) = webdx {
            context.CountWebDX(webdx);
        }
        if selector.Relation() == RelationType::kIndirectAdjacent {
            context.Count(kCSSSelectorIndirectAdjacent);
        }
        if let Some(list) = selector.SelectorList() {
            for complex in list.ComplexSelectors() {
                for simple in complex.SimpleSelectors() {
                    Self::record_one(simple, context, nesting, has_visited);
                }
            }
        }
    }
    pub fn RecordUsageAndDeprecations(
        selectors: &[CSSSelector],
        nesting: CSSNestingType,
        options: &SelectorParserOptions<'_>,
        mut has_visited: Option<&mut bool>,
    ) {
        let Some(context) = options.usage_context else {
            return;
        };
        if !context.IsUseCounterRecordingEnabled() || options.ua_sheet_mode {
            return;
        }
        for selector in selectors {
            Self::record_one(selector, context, nesting, &mut has_visited);
        }
    }

    // cpp: css_selector_parser.cc:2784-2797
    fn ContainsUnknownWebkitPseudoElements(selectors: &[CSSSelector]) -> bool {
        selectors.iter().any(|selector| {
            if selector.GetPseudoType() != PseudoType::kPseudoWebKitCustomElement {
                return false;
            }
            let value = selector.Value();
            !Self::IsKnownWebKitCustomPseudoElement(&value)
        })
    }

    /// Capability boundary for the persistent matcher. Stylesheet parsing does
    /// not call this: all selectors are parsed by the native typed parser.
    pub fn UnsupportedParsedFeature(selectors: &[CSSSelector]) -> Option<std::string::String> {
        fn unsupported(selectors: &[CSSSelector]) -> Option<std::string::String> {
            use PseudoType::*;
            for selector in selectors {
                if matches!(
                    selector.Match(),
                    MatchType::kPseudoClass | MatchType::kPseudoElement
                ) && !matches!(
                    selector.GetPseudoType(),
                    kPseudoIs
                        | kPseudoWhere
                        | kPseudoNot
                        | kPseudoHas
                        | kPseudoScope
                        | kPseudoRoot
                        | kPseudoEmpty
                        | kPseudoFirstChild
                        | kPseudoLastChild
                        | kPseudoOnlyChild
                        | kPseudoFirstOfType
                        | kPseudoLastOfType
                        | kPseudoOnlyOfType
                        | kPseudoNthChild
                        | kPseudoNthLastChild
                        | kPseudoNthOfType
                        | kPseudoNthLastOfType
                        | kPseudoBefore
                        | kPseudoAfter
                        | kPseudoFirstLetter
                        | kPseudoFirstLine
                        | kPseudoMarker
                        | kPseudoBackdrop
                        | kPseudoPlaceholder
                        | kPseudoAnyLink
                        | kPseudoLink
                        | kPseudoVisited
                        | kPseudoHover
                        | kPseudoActive
                        | kPseudoFocus
                        | kPseudoFocusVisible
                        | kPseudoFocusWithin
                        | kPseudoEnabled
                        | kPseudoDisabled
                        | kPseudoReadOnly
                        | kPseudoReadWrite
                        | kPseudoRequired
                        | kPseudoOptional
                        | kPseudoChecked
                        | kPseudoIndeterminate
                        | kPseudoPlaceholderShown
                        | kPseudoLang
                        | kPseudoDir
                        | kPseudoRelativeAnchor
                        | kPseudoParent
                        | kPseudoUnparsed
                ) {
                    return Some(selector.Value().Utf8());
                }
                if let Some(list) = selector.SelectorList() {
                    if let Some(feature) = unsupported(&list.CopySelectors()) {
                        return Some(feature);
                    }
                }
            }
            None
        }
        unsupported(selectors)
    }
    pub fn UnsupportedFeature(text: &String) -> Option<std::string::String> {
        let context = SelectorParserContext { html: true, quirks: false };
        Self::UnsupportedParsedFeature(&Self::ParseSelector(text, &context))
    }
    // cpp: css_selector_parser.cc:160-183. The returned slice is the
    // appended arena segment. Chromium leaves a committed segment in the arena
    // if the whole-stream end check then fails; preserve that distinction.
    pub fn ParseSelectorStream<'a, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &SelectorParserContext,
        nesting: CSSNestingType,
        options: &SelectorParserOptions<'_>,
        arena: &'a mut Vec<CSSSelector>,
    ) -> &'a mut [CSSSelector] {
        let initial = arena.len();
        let output = Rc::new(RefCell::new(std::mem::take(arena)));
        stream.ConsumeWhitespace();
        let mut parser = Parser::new_in_arena(context, options, false, output.clone(), false);
        let result = parser.list(stream, false, false, nesting, &mut ResultFlags::default());
        let result = result
            .filter(|_| stream.AtEnd())
            .unwrap_or(initial..initial);
        Self::RecordUsageAndDeprecations(&output.borrow()[result.clone()], nesting, options, None);
        *arena = std::mem::take(&mut *output.borrow_mut());
        &mut arena[result]
    }
    // cpp: css_selector_parser.cc:208-225. Same shared arena/consumer as ParseSelector.
    pub fn ParseScopeBoundaryStream<'a,T:TokenStreamTokenizer>(stream:&mut CSSParserTokenStream<'_,T>,context:&SelectorParserContext,nesting:CSSNestingType,options:&SelectorParserOptions<'_>,arena:&'a mut Vec<CSSSelector>)->&'a mut [CSSSelector]{
        let initial=arena.len();let output=Rc::new(RefCell::new(std::mem::take(arena)));stream.ConsumeWhitespace();let mut parser=Parser::new_in_arena(context,options,false,output.clone(),true);let result=parser.list(stream,false,false,nesting,&mut ResultFlags::default()).filter(|_|stream.AtEnd()).unwrap_or(initial..initial);Self::RecordUsageAndDeprecations(&output.borrow()[result.clone()],nesting,options,None);*arena=std::mem::take(&mut *output.borrow_mut());&mut arena[result]
    }
    // cpp: .cc:187-207,289-349. Scope rollback is shared by observer and non-observer paths.
    pub fn ConsumeSelector<'a, T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &SelectorParserContext,
        nesting: CSSNestingType,
        options: &SelectorParserOptions<'_>,
        semicolon_aborts_nested_selector: bool,
        mut observer: Option<&mut dyn CSSSelectorParserObserver>,
        arena: &'a mut Vec<CSSSelector>,
        has_visited_style: Option<&mut bool>,
    ) -> &'a mut [CSSSelector] {
        let initial = arena.len();
        let output = Rc::new(RefCell::new(std::mem::take(arena)));
        stream.ConsumeWhitespace();
        let mut parser = Parser::new_in_arena(
            context,
            options,
            semicolon_aborts_nested_selector,
            output.clone(),
            false,
        );
        let result = {
            let mut reset_vector = ResetVectorAfterScope::new(&output);
            let mut flags = ResultFlags::default();
            let mut valid = true;
            loop {
                let start = stream.LookAheadOffset();
                let selectors = parser.complex(stream, false, nesting, &mut flags);
                if selectors.is_none()
                    || !(stream.AtEnd()
                        || matches!(stream.Peek().GetType(), kLeftBraceToken | kCommaToken))
                {
                    let types = if AbortsNestedSelectorParsing(
                        kSemicolonToken,
                        parser.semicolon_aborts_nested_selector,
                        nesting,
                    ) {
                        &[kLeftBraceToken, kCommaToken, kSemicolonToken][..]
                    } else {
                        &[kLeftBraceToken, kCommaToken][..]
                    };
                    stream.SkipUntilPeekedTypeIs(types);
                    valid = false;
                    break;
                }
                let end = stream.LookAheadOffset();
                if let Some(observer) = observer.as_deref_mut() {
                    observer.ObserveSelector(start, end);
                }
                if stream.UncheckedAtEnd()
                    || stream.Peek().GetType() == kLeftBraceToken
                    || AbortsNestedSelectorParsing(
                        stream.Peek().GetType(),
                        parser.semicolon_aborts_nested_selector,
                        nesting,
                    )
                {
                    break;
                }
                debug_assert_eq!(stream.Peek().GetType(), kCommaToken);
                stream.ConsumeIncludingWhitespace();
            }
            if valid {
                reset_vector.CommitAddedElements()
            } else {
                initial..initial
            }
        };
        Self::RecordUsageAndDeprecations(
            &output.borrow()[result.clone()],
            nesting,
            options,
            has_visited_style,
        );
        *arena = std::mem::take(&mut *output.borrow_mut());
        &mut arena[result]
    }
    // cpp: .h:80-81; .cc:2206-2307.
    pub fn ConsumeANPlusB<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
    ) -> Option<(i32, i32)> {
        Parser::anb(stream)
    }
    // cpp: .h:165-166; .cc:594-639.
    pub fn ConsumeForgivingRelativeSelectorList<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &SelectorParserContext,
        options: &SelectorParserOptions<'_>,
    ) -> Option<Rc<CSSSelectorList>> {
        let mut parser = Parser::new(context, options, false);
        stream.ConsumeWhitespace();
        parser.forgiving_relative_list(stream, &mut ResultFlags::default())
    }
    pub fn ParseSelector(text: &String, context: &SelectorParserContext) -> Vec<CSSSelector> {
        Self::ParseSelectorWithOptions(
            text,
            context,
            CSSNestingType::kNone,
            &SelectorParserOptions::default(),
        )
    }
    pub fn ParseSelectorWithOptions(
        text: &String,
        context: &SelectorParserContext,
        nesting: CSSNestingType,
        options: &SelectorParserOptions<'_>,
    ) -> Vec<CSSSelector> {
        Self::parse(text, context, nesting, options, false)
    }
    pub fn ParseScopeBoundary(
        text: &String,
        context: &SelectorParserContext,
        nesting: CSSNestingType,
        options: &SelectorParserOptions<'_>,
    ) -> Vec<CSSSelector> {
        Self::parse(text, context, nesting, options, true)
    }
    // cpp: css_selector_parser.cc:230-253
    pub fn SupportsComplexSelector(
        text: &String,
        context: &SelectorParserContext,
        options: &SelectorParserOptions<'_>,
    ) -> bool {
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(text), 0);
        Self::SupportsComplexSelectorStream(&mut stream, context, options)
    }
    pub fn SupportsComplexSelectorStream<T: TokenStreamTokenizer>(
        stream: &mut CSSParserTokenStream<'_, T>,
        context: &SelectorParserContext,
        options: &SelectorParserOptions<'_>,
    ) -> bool {
        stream.ConsumeWhitespace();
        let mut parser = Parser::new(context, options, false);
        parser.in_supports_parsing = true;
        let Some(selectors) = parser.complex(
            stream,
            false,
            CSSNestingType::kNone,
            &mut ResultFlags::default(),
        ) else {
            return false;
        };
        stream.AtEnd()
            && !selectors.is_empty()
            && !Self::ContainsUnknownWebkitPseudoElements(&parser.output.borrow()[selectors])
    }

    fn parse(
        text: &String,
        context: &SelectorParserContext,
        nesting: CSSNestingType,
        options: &SelectorParserOptions<'_>,
        disallow_pseudo: bool,
    ) -> Vec<CSSSelector> {
        if nesting > CSSNestingType::kNesting {
            return Vec::new();
        }
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(text), 0);
        let mut parser = Parser::new(context, options, disallow_pseudo);
        let Some(result) = parser.list(
            &mut stream,
            false,
            false,
            nesting,
            &mut ResultFlags::default(),
        ) else {
            return vec![];
        };
        if !stream.AtEnd() {
            vec![]
        } else {
            Self::RecordUsageAndDeprecations(
                &parser.output.borrow()[result.clone()],
                nesting,
                options,
                None,
            );
            parser.output.borrow()[result].to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! enabled_runtime_methods {
        () => {
            fn CSSMediaElementPseudosEnabled(&self) -> bool {
                true
            }
            fn GeolocationElementEnabled(&self) -> bool {
                true
            }
            fn UserMediaElementEnabled(&self) -> bool {
                true
            }
            fn InstallElementEnabled(&self) -> bool {
                true
            }
            fn CSSPseudoScrollMarkersEnabled(&self) -> bool {
                true
            }
            fn CSSScrollMarkerTargetBeforeAfterEnabled(&self) -> bool {
                true
            }
            fn CSSPseudoScrollButtonsEnabled(&self) -> bool {
                true
            }
            fn CSSPseudoColumnEnabled(&self) -> bool {
                true
            }
            fn SearchTextHighlightPseudoEnabled(&self) -> bool {
                true
            }
            fn WebMCPEnabled(&self) -> bool {
                true
            }
            fn UnboundedElementEnabled(&self) -> bool {
                true
            }
            fn CSSPseudoHasSlottedEnabled(&self) -> bool {
                true
            }
            fn OverscrollGesturesEnabled(&self) -> bool {
                true
            }
            fn CustomizableComboboxEnabled(&self) -> bool {
                true
            }
            fn FilterableSelectEnabled(&self) -> bool {
                true
            }
            fn CSSImageAnimationEnabled(&self) -> bool {
                true
            }
            fn MenuElementsEnabled(&self) -> bool {
                true
            }
            fn NavigationSourcePseudoClassEnabled(&self) -> bool {
                true
            }
            fn DeclarativeSkeletonsEnabled(&self) -> bool {
                true
            }
        };
    }
    struct EnabledRuntime;
    impl CSSSelectorRuntime for EnabledRuntime {
        fn HasDocument(&self) -> bool {
            true
        }
        enabled_runtime_methods!();
    }
    #[derive(Default)]
    struct RecordingUsage {
        disabled: bool,
        counts: std::cell::RefCell<Vec<SelectorWebFeature>>,
        deprecations: std::cell::RefCell<Vec<SelectorWebFeature>>,
        webdx: std::cell::RefCell<Vec<SelectorWebDXFeature>>,
    }
    impl CSSSelectorUsageContext for RecordingUsage {
        fn IsUseCounterRecordingEnabled(&self) -> bool {
            !self.disabled
        }
        fn IsDeprecated(&self, feature: SelectorWebFeature) -> bool {
            feature == SelectorWebFeature::kCSSSelectorPseudoAny
        }
        fn Count(&self, feature: SelectorWebFeature) {
            if !self.disabled {
                self.counts.borrow_mut().push(feature);
            }
        }
        fn CountDeprecation(&self, feature: SelectorWebFeature) {
            self.deprecations.borrow_mut().push(feature);
        }
        fn CountWebDX(&self, feature: SelectorWebDXFeature) {
            self.webdx.borrow_mut().push(feature);
        }
    }
    #[test]
    fn usage_recurses_routes_deprecations_and_preserves_visited_recording_gate() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let usage = RecordingUsage::default();
        let options = SelectorParserOptions {
            usage_context: Some(&usage),
            ..SelectorParserOptions::default()
        };
        let selectors = CSSSelectorParser::ParseSelectorWithOptions(
            &String::from(":is(:visited, :autofill) ~ :-webkit-any(.a)::details-content"),
            &context,
            CSSNestingType::kNone,
            &options,
        );
        assert!(!selectors.is_empty());
        assert!(usage
            .counts
            .borrow()
            .contains(&SelectorWebFeature::kCSSSelectorPseudoIs));
        assert!(usage
            .counts
            .borrow()
            .contains(&SelectorWebFeature::kCSSSelectorIndirectAdjacent));
        assert_eq!(
            usage.deprecations.borrow().as_slice(),
            &[SelectorWebFeature::kCSSSelectorPseudoAny]
        );
        assert!(usage
            .webdx
            .borrow()
            .contains(&SelectorWebDXFeature::kAutofill));
        assert!(usage
            .webdx
            .borrow()
            .contains(&SelectorWebDXFeature::kDetailsContent));
        let mut visited = false;
        CSSSelectorParser::RecordUsageAndDeprecations(
            &selectors,
            CSSNestingType::kNone,
            &options,
            Some(&mut visited),
        );
        assert!(visited);
        let disabled = RecordingUsage {
            disabled: true,
            ..RecordingUsage::default()
        };
        let mut options = SelectorParserOptions {
            usage_context: Some(&disabled),
            ..SelectorParserOptions::default()
        };
        visited = false;
        CSSSelectorParser::RecordUsageAndDeprecations(
            &selectors,
            CSSNestingType::kNone,
            &options,
            Some(&mut visited),
        );
        assert!(!visited);
        options.usage_context = Some(&usage);
        options.ua_sheet_mode = true;
        CSSSelectorParser::RecordUsageAndDeprecations(
            &selectors,
            CSSNestingType::kNone,
            &options,
            Some(&mut visited),
        );
        assert!(!visited);
    }
    #[derive(Default)]
    struct Observer(Vec<(u32, u32)>);
    impl CSSSelectorParserObserver for Observer {
        fn ObserveSelector(&mut self, start: u32, end: u32) {
            self.0.push((start, end));
        }
    }
    #[test]
    fn stream_entry_points_keep_arena_segments_observe_ranges_and_recover_boundaries() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions::default();
        let text = String::from("  .a, .b { color: red }");
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        let mut arena = vec![CSSSelector::default()];
        let mut observer = Observer::default();
        let parsed = CSSSelectorParser::ConsumeSelector(
            &mut stream,
            &context,
            CSSNestingType::kNone,
            &options,
            false,
            Some(&mut observer),
            &mut arena,
            None,
        );
        assert_eq!(parsed.len(), 2);
        assert_eq!(arena.len(), 3);
        assert_eq!(observer.0, vec![(2, 4), (6, 9)]);
        assert_eq!(stream.Peek().GetType(), kLeftBraceToken);
        for (nesting, semicolon_abort, expected) in [
            (CSSNestingType::kNesting, true, kSemicolonToken),
            (CSSNestingType::kNesting, false, kLeftBraceToken),
            (CSSNestingType::kNone, true, kLeftBraceToken),
        ] {
            let text = String::from(".x !; .y {");
            let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
            assert!(CSSSelectorParser::ConsumeSelector(
                &mut stream,
                &context,
                nesting,
                &options,
                semicolon_abort,
                None,
                &mut arena,
                None
            )
            .is_empty());
            assert_eq!(stream.Peek().GetType(), expected);
            assert_eq!(arena.len(), 3);
        }
        let text = String::from(".a !");
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        assert!(CSSSelectorParser::ParseSelectorStream(
            &mut stream,
            &context,
            CSSNestingType::kNone,
            &options,
            &mut arena
        )
        .is_empty());
        assert_eq!(arena.len(), 4); // The list was committed before whole-stream rejection.
    }
    struct PseudoElementContext {
        requires_colon: bool,
        without_colon: std::cell::Cell<usize>,
    }
    impl CSSSelectorPseudoElementContext for PseudoElementContext {
        fn HasParent(&self) -> bool {
            true
        }
        fn Runtime(&self) -> &dyn CSSSelectorRuntime {
            &EnabledRuntime
        }
        fn CSSOMGetComputedStylePseudoElementRequiresColonEnabled(&self) -> bool {
            self.requires_colon
        }
        fn IsWebExposed(&self, id: PseudoId) -> bool {
            matches!(
                id,
                PseudoId::kPseudoIdMarker
                    | PseudoId::kPseudoIdHighlight
                    | PseudoId::kPseudoIdScrollButton
                    | PseudoId::kPseudoIdViewTransitionOld
            )
        }
        fn CountPseudoElementWithoutColon(&self) {
            self.without_colon.set(self.without_colon.get() + 1);
        }
    }
    #[test]
    fn cssom_pseudo_elements_use_legacy_colons_exposure_and_argument_rules() {
        let mut context = PseudoElementContext {
            requires_colon: false,
            without_colon: std::cell::Cell::new(0),
        };
        let features = SelectorParserFeatures::default();
        let mut argument = AtomicString::default();
        for text in ["before", ":before", "::BEFORE", r":bef\oRE"] {
            assert_eq!(
                CSSSelectorParser::ParsePseudoElement(
                    &String::from(text),
                    &context,
                    features,
                    &mut argument
                ),
                PseudoId::kPseudoIdBefore,
                "{text}"
            );
        }
        assert_eq!(context.without_colon.get(), 1);
        context.requires_colon = true;
        assert_eq!(
            CSSSelectorParser::ParsePseudoElement(
                &String::from("before"),
                &context,
                features,
                &mut argument
            ),
            PseudoId::kPseudoIdNone
        );
        for (text, id) in [
            ("marker", PseudoId::kPseudoIdNone),
            (":marker", PseudoId::kPseudoIdInvalid),
            ("::marker", PseudoId::kPseudoIdMarker),
            ("::marker ", PseudoId::kPseudoIdInvalid),
            ("::-webkit-custom", PseudoId::kPseudoIdNone),
            ("::-internal-custom", PseudoId::kPseudoIdNone),
            ("::placeholder", PseudoId::kPseudoIdInvalid),
            ("::view-transition-old(*)", PseudoId::kPseudoIdInvalid),
            (
                "::view-transition-old(hero.card)",
                PseudoId::kPseudoIdInvalid,
            ),
        ] {
            assert_eq!(
                CSSSelectorParser::ParsePseudoElement(
                    &String::from(text),
                    &context,
                    features,
                    &mut argument
                ),
                id,
                "{text}"
            );
        }
        for (text, id, arg) in [
            (
                "::highlight(custom)",
                PseudoId::kPseudoIdHighlight,
                "custom",
            ),
            ("::scroll-button(up)", PseudoId::kPseudoIdScrollButton, "up"),
            (
                "::view-transition-old(hero)",
                PseudoId::kPseudoIdViewTransitionOld,
                "hero",
            ),
        ] {
            assert_eq!(
                CSSSelectorParser::ParsePseudoElement(
                    &String::from(text),
                    &context,
                    features,
                    &mut argument
                ),
                id,
                "{text}"
            );
            assert_eq!(argument.Utf8(), arg);
        }
    }
    #[test]
    fn compound_pseudos_restrict_nested_lists_and_host_marks_has_arguments() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let parse = |text: &str| CSSSelectorParser::ParseSelector(&String::from(text), &context);
        for text in [":host(.x)", ":host-context(.x)", ":host(:is(.x, .y))"] {
            assert!(!parse(text).is_empty(), "{text}");
        }
        for text in [
            ":host(.x,.y)",
            ":host-context(.x,.y)",
            ":host(.x > .y)",
            ":host(:not(.x > .y))",
            "::slotted(.x,.y)",
            "::slotted(:has(.x))",
            ":host(:has(.x))",
            "::slotted(.x)::part(a)",
        ] {
            assert!(parse(text).is_empty(), "{text}");
        }
        assert!(!parse(":-webkit-any(.x,.y)").is_empty());
        let slotted = parse("slot::slotted(.x)::before");
        assert_eq!(slotted[0].GetPseudoType(), PseudoType::kPseudoSlotted);
        assert_eq!(slotted[1].GetPseudoType(), PseudoType::kPseudoBefore);
        assert_eq!(slotted[1].Relation(), RelationType::kShadowSlot);
        let cue = parse("video::cue(.x,.y)");
        assert_eq!(cue[0].SelectorList().unwrap().ComplexSelectors().count(), 2);
        for text in [":host:has(.x)", ":has(.x):host", ":host(.x):has(.y)"] {
            let parsed = parse(text);
            assert!(
                parsed
                    .iter()
                    .find(|s| s.GetPseudoType() == PseudoType::kPseudoHas)
                    .unwrap()
                    .HasArgumentMatchInShadowTree(),
                "{text}"
            );
        }
        assert!(!parse(":host .x:has(.y)")[0].HasArgumentMatchInShadowTree());
        let host = parse(":host(:is(.x > .y, .z))");
        let inner = host[0].SelectorList().unwrap();
        assert_eq!(
            inner
                .First()
                .unwrap()
                .SimpleSelectors()
                .next()
                .unwrap()
                .SelectorList()
                .unwrap()
                .ComplexSelectors()
                .count(),
            1
        );
    }
    #[test]
    fn supports_lists_are_unforgiving_and_unknown_webkit_fallback_is_source_behavior() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions::default();
        for text in [":is(.a, :unknown)", ":where()", ":host(:is(.a > .b,.c))"] {
            assert!(
                !CSSSelectorParser::SupportsComplexSelector(
                    &String::from(text),
                    &context,
                    &options
                ),
                "{text}"
            );
        }
        let unknown =
            CSSSelectorParser::ParseSelector(&String::from("::-webkit-unrecognized"), &context);
        assert_eq!(
            unknown[0].GetPseudoType(),
            PseudoType::kPseudoWebKitCustomElement
        );
        assert!(!CSSSelectorParser::SupportsComplexSelector(
            &String::from("::-webkit-unrecognized"),
            &context,
            &options
        ));
        assert!(
            CSSSelectorParser::ParseSelector(&String::from("::-internal-custom"), &context)
                .is_empty()
        );
        let mut ua = SelectorParserOptions::default();
        ua.ua_sheet_mode = true;
        assert!(!CSSSelectorParser::ParseSelectorWithOptions(
            &String::from("video::-webkit-custom.scrolling > .x"),
            &context,
            CSSNestingType::kNone,
            &ua
        )
        .is_empty());
    }
    #[test]
    fn functional_pseudo_arguments_and_runtime_feature_gates_follow_source() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let mut options = SelectorParserOptions::default();
        options.runtime_context = Some(&EnabledRuntime);
        let parse = |text: &str, options: &SelectorParserOptions<'_>| {
            CSSSelectorParser::ParseSelectorWithOptions(
                &String::from(text),
                &context,
                CSSNestingType::kNone,
                options,
            )
        };
        for (text, expected) in [
            ("::picker(SeLeCt)", "select"),
            ("::highlight(custom)", "custom"),
            ("::scroll-button(block-start)", "block-start"),
            ("::scroll-button(*)", "*"),
        ] {
            let selectors = parse(text, &options);
            assert_eq!(selectors[0].Argument().Utf8(), expected, "{text}");
        }
        for text in [
            "::picker(menu)",
            "::picker(select,)",
            "::highlight()",
            "::highlight(a b)",
            "::scroll-button(diagonal)",
            "::scroll-button(up down)",
            "::part()",
            "::part(a,b)",
            ":active-view-transition-type(a,)",
            ":active-view-transition-type(a b)",
            "::view-transition-old(initial)",
            "::view-transition-new(default)",
            "::view-transition-group(a.)",
        ] {
            assert!(parse(text, &options).is_empty(), "{text}");
        }
        assert_eq!(parse("::part(foo bar)", &options)[0].IdentList().len(), 2);
        assert_eq!(
            parse(":active-view-transition-type(foo, bar)", &options)[0]
                .IdentList()
                .len(),
            2
        );
        for text in [
            "::view-transition-old(hero.card.active)",
            "::view-transition-group(.card)",
            "::view-transition-new(*.card)",
        ] {
            assert!(!parse(text, &options).is_empty(), "{text}");
        }
        assert!(parse(":lang(en, fr)", &options).is_empty());
        assert!(parse(":lang(\"en\")", &options).is_empty());
        assert!(parse("[type=TEXT s]", &options).is_empty());
        options.features.lang_extended_ranges = true;
        options.features.case_sensitive_selector = true;
        assert_eq!(
            parse(":lang(en, \"fr\")", &options)[0]
                .ArgumentList()
                .unwrap()
                .len(),
            2
        );
        assert!(parse(":lang(en,)", &options).is_empty());
        assert!(!parse("[type=TEXT s]", &options).is_empty());
    }
    #[test]
    fn source_token_path_retains_escapes_lists_nth_and_invalid_combinators() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let parse = |text: &str| CSSSelectorParser::ParseSelector(&String::from(text), &context);
        let list = CSSSelectorList::AdoptSelectorVector(parse(
            "main > p.a\\+b:nth-child(2n + 1), [title='a,b' i]",
        ));
        assert!(list.IsValid());
        assert_eq!(list.ComplexSelectors().count(), 2);
        let first = list.First().unwrap();
        assert_eq!(
            first
                .SimpleSelectors()
                .find(|s| s.Match() == MatchType::kClass)
                .unwrap()
                .Value()
                .Utf8(),
            "a+b"
        );
        assert_eq!(
            first
                .SimpleSelectors()
                .find(|s| s.GetPseudoType() == PseudoType::kPseudoNthChild)
                .unwrap()
                .NthAValue(),
            2
        );
        assert!(parse("p >").is_empty());
        assert!(parse("p,").is_empty());
        assert!(parse("[foo=bar invalid]").is_empty());
        assert!(!parse(":is(.ok,:unknown)").is_empty());
        assert!(parse(":has(:has(.x))").is_empty());
        assert_eq!(parse("*")[0].Match(), MatchType::kUniversalTag);
    }
    struct Namespaces;
    impl CSSSelectorNamespaceContext for Namespaces {
        fn DefaultNamespace(&self) -> AtomicString {
            AtomicString::from_str("http://default")
        }
        fn NamespaceURIFromPrefix(&self, prefix: &AtomicString) -> AtomicString {
            match prefix.Utf8().as_str() {
                "svg" => AtomicString::from_str("http://www.w3.org/2000/svg"),
                "def" => AtomicString::from_str("http://default"),
                _ => AtomicString::default(),
            }
        }
    }
    #[test]
    fn stylesheet_namespaces_implicit_universals_and_logical_subjects() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions {
            namespace_context: Some(&Namespaces),
            ..Default::default()
        };
        let parse = |text: &str| {
            CSSSelectorParser::ParseSelectorWithOptions(
                &String::from(text),
                &context,
                CSSNestingType::kNone,
                &options,
            )
        };
        let selectors = parse("svg|Rect.hot");
        assert_eq!(selectors[0].TagQName().LocalName().Utf8(), "rect");
        assert_eq!(selectors[0].TagQName().Prefix().Utf8(), "svg");
        assert_eq!(
            selectors[0].TagQName().NamespaceURI().Utf8(),
            "http://www.w3.org/2000/svg"
        );
        let selectors = parse(".hot");
        assert_eq!(selectors[0].Match(), MatchType::kUniversalTag);
        assert!(selectors[0].IsImplicit());
        assert_eq!(
            selectors[0].TagQName().NamespaceURI().Utf8(),
            "http://default"
        );
        assert!(parse("unknown|rect").is_empty());
        assert!(parse("[unknown|id]").is_empty());
        let selectors = parse("[id][svg|href]");
        assert!(selectors[1].Attribute().NamespaceURI().IsNull());
        assert_eq!(
            selectors[2].Attribute().NamespaceURI().Utf8(),
            "http://www.w3.org/2000/svg"
        );
        assert!(parse("def|rect")[0].TagQName().Prefix().IsNull());
        assert_eq!(parse("|rect")[0].TagQName().NamespaceURI().Utf8(), "");
        assert_eq!(parse("*|rect")[0].TagQName().NamespaceURI().Utf8(), "*");
        for text in [":is(.hot)", ":where(.hot)", ":not(.hot)", ":has(.hot)"] {
            let selectors = parse(text);
            let list = selectors[1].SelectorList().unwrap();
            assert!(
                list.First()
                    .unwrap()
                    .SimpleSelectors()
                    .all(|s| s.Match() != MatchType::kUniversalTag),
                "{text}"
            );
        }
        let selectors = parse(":is(.ancestor .subject)");
        let list = selectors[1].SelectorList().unwrap();
        let tags: Vec<_> = list
            .First()
            .unwrap()
            .SimpleSelectors()
            .filter(|s| s.Match() == MatchType::kUniversalTag)
            .collect();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].TagQName().NamespaceURI().Utf8(), "http://default");
        let context = SelectorParserContext {
            html: false,
            quirks: false,
        };
        assert_eq!(
            CSSSelectorParser::ParseSelectorWithOptions(
                &String::from("svg|Rect"),
                &context,
                CSSNestingType::kNone,
                &options
            )[0]
            .TagQName()
            .LocalName()
            .Utf8(),
            "Rect"
        );
    }
    #[test]
    fn nesting_scope_anchors_and_invalid_forgiving_parent_arguments() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions::default();
        for (nesting, pseudo) in [
            (CSSNestingType::kNesting, PseudoType::kPseudoParent),
            (CSSNestingType::kScope, PseudoType::kPseudoScope),
        ] {
            for text in [".a", ".a:is(.b)", "> .a"] {
                let selectors = CSSSelectorParser::ParseSelectorWithOptions(
                    &String::from(text),
                    &context,
                    nesting,
                    &options,
                );
                let anchor = selectors.last().unwrap();
                assert_eq!(anchor.GetPseudoType(), pseudo, "{text}");
                assert!(anchor.IsImplicit());
                assert!(anchor.IsScopeContaining());
                assert_eq!(
                    selectors[selectors.len() - 2].Relation(),
                    if text.starts_with('>') {
                        RelationType::kChild
                    } else {
                        RelationType::kDescendant
                    }
                );
            }
        }
        for (nesting, text) in [
            (CSSNestingType::kNesting, ".a > :is(.b, &)"),
            (CSSNestingType::kNesting, ".a > :is(.b, !&)"),
            (CSSNestingType::kScope, ".a > :is(.b, !:SCOPE)"),
            (CSSNestingType::kScope, ".a > &"),
        ] {
            let selectors = CSSSelectorParser::ParseSelectorWithOptions(
                &String::from(text),
                &context,
                nesting,
                &options,
            );
            assert!(selectors.iter().all(|s| !s.IsImplicit()), "{text}");
            if text.contains('!') {
                assert!(CSSSelectorList::AdoptSelectorVector(selectors.clone())
                    .SelectorsText()
                    .as_str()
                    .contains('!'));
                let list = selectors
                    .iter()
                    .find_map(CSSSelector::SelectorList)
                    .unwrap();
                assert_eq!(list.ComplexSelectorsIncludingUnparsedInvalid().count(), 2);
            }
        }
        assert!(CSSSelectorParser::ParseSelector(&String::from("> .a"), &context).is_empty());
        let scopes = CSSSelectorParser::ParseScopeBoundary(
            &String::from(".a:has(> .b), :scope"),
            &context,
            CSSNestingType::kNone,
            &options,
        );
        assert!(!scopes.is_empty());
        for invalid in [".a::before", ":is(.a, ::before):not(::after)"] {
            assert!(
                CSSSelectorParser::ParseScopeBoundary(
                    &String::from(invalid),
                    &context,
                    CSSNestingType::kNone,
                    &options
                )
                .is_empty(),
                "{invalid}"
            );
        }
        let mut options = SelectorParserOptions::default();
        options.features.serialize_invalid_selectors = true;
        for text in [":is()", ":is(.a,)", ":is(.a,:unknown)"] {
            let selectors = CSSSelectorParser::ParseSelectorWithOptions(
                &String::from(text),
                &context,
                CSSNestingType::kNone,
                &options,
            );
            let list = selectors[0].SelectorList().unwrap();
            assert_eq!(list.ComputeLength(), if text == ":is()" { 1 } else { 2 });
        }
    }
    #[test]
    fn pseudo_restrictions_and_has_invalidation_flags_follow_source() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let parse = |text: &str| CSSSelectorParser::ParseSelector(&String::from(text), &context);
        for invalid in [
            ":not(::before)",
            ":nth-child(2 of ::before:hover)",
            "div ::before:hover",
            "div :has(:has(.a))",
            "::before:has(.a)",
            "::before:not(.x)",
            "::before:is(.x).class",
            "::before > .x",
        ] {
            assert!(parse(invalid).is_empty(), "{invalid}");
        }
        assert!(parse(":is(::before)")[0]
            .SelectorList()
            .unwrap()
            .First()
            .is_none());
        assert_eq!(parse("*.a").len(), 1);
        assert_eq!(parse("::before::marker").len(), 2);
        assert!(parse("::before:is(.x)")[0].SelectorList().is_none());
        let mut options = SelectorParserOptions::default();
        options.features.pseudo_elements_hoverable = true;
        assert!(!CSSSelectorParser::ParseSelectorWithOptions(
            &String::from("::before:hover"),
            &context,
            CSSNestingType::kNone,
            &options
        )
        .is_empty());
        options.features.marker_nested_pseudo_element = false;
        assert!(CSSSelectorParser::ParseSelectorWithOptions(
            &String::from("::before::marker"),
            &context,
            CSSNestingType::kNone,
            &options
        )
        .is_empty());
        assert!(!parse(":has(.a)")[0].ContainsPseudoInsideHasPseudoClass());
        assert!(parse(":has(:hover)")[0].ContainsPseudoInsideHasPseudoClass());
        assert!(parse(":has(&)")[0].ContainsComplexLogicalCombinationsInsideHasPseudoClass());
        assert!(
            parse(":has(:is(.a > .b))")[0].ContainsComplexLogicalCombinationsInsideHasPseudoClass()
        );
        assert!(!parse(":has(.a > .b)")[0].ContainsComplexLogicalCombinationsInsideHasPseudoClass());
    }
    #[test]
    fn implicit_combinators_preserve_matching_compound_order() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let parse = |text: &str| CSSSelectorParser::ParseSelector(&String::from(text), &context);
        let selectors = parse("input#x::placeholder");
        assert_eq!(selectors[0].GetPseudoType(), PseudoType::kPseudoPlaceholder);
        assert_eq!(selectors[1].Match(), MatchType::kTag);
        assert_eq!(selectors[0].Relation(), RelationType::kUAShadow);
        let selectors = parse("::placeholder");
        assert_eq!(selectors.len(), 2);
        assert!(selectors[1].IsImplicit());
        assert_eq!(selectors[0].GetPseudoType(), PseudoType::kPseudoPlaceholder);
        assert_eq!(selectors[0].Relation(), RelationType::kUAShadow);
        let mut options = SelectorParserOptions::default();
        options.features.logical_combination_pseudo = true;
        let selectors = CSSSelectorParser::ParseSelectorWithOptions(
            &String::from(".a::before::marker"),
            &context,
            CSSNestingType::kNone,
            &options,
        );
        assert_eq!(
            selectors
                .iter()
                .map(|s| s.GetPseudoType())
                .collect::<Vec<_>>(),
            vec![
                PseudoType::kPseudoMarker,
                PseudoType::kPseudoBefore,
                PseudoType::kPseudoUnknown
            ]
        );
        assert_eq!(selectors[0].Relation(), RelationType::kPseudoChild);
        assert_eq!(selectors[1].Relation(), RelationType::kPseudoChild);
        assert_eq!(selectors[2].Match(), MatchType::kClass);
        let selectors = CSSSelectorParser::ParseSelectorWithOptions(
            &String::from(":is(::before)"),
            &context,
            CSSNestingType::kNone,
            &options,
        );
        assert!(selectors[0].SelectorList().unwrap().IsValid());
    }

    #[test]
    fn supports_complex_selector_rejects_unknown_webkit_pseudos_and_lists() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions::default();
        assert!(CSSSelectorParser::SupportsComplexSelector(
            &String::from("input::-webkit-slider-thumb"),
            &context,
            &options,
        ));
        assert!(!CSSSelectorParser::SupportsComplexSelector(
            &String::from("input::-webkit-rechrom-unknown"),
            &context,
            &options,
        ));
        assert!(!CSSSelectorParser::SupportsComplexSelector(
            &String::from("div, span"),
            &context,
            &options,
        ));
        assert!(CSSSelectorParser::SupportsComplexSelector(
            &String::from("main > p:is(.a, .b)"),
            &context,
            &options,
        ));
    }
    #[test]
    fn shared_arena_scopes_retain_prefix_capacity_and_reverse_in_place() {
        let output = Rc::new(RefCell::new(Vec::with_capacity(64)));
        let mut prefix = CSSSelector::default();
        prefix.SetMatch(MatchType::kClass);
        prefix.SetValue(AtomicString::from_str("prefix"), false);
        output.borrow_mut().push(prefix);
        let backing = output.borrow().as_ptr();
        {
            let reset = ResetVectorAfterScope::new(&output);
            output.borrow_mut().push(CSSSelector::default());
            assert_eq!(reset.AddedElements(), 1..2);
        }
        assert_eq!(output.borrow().len(), 1);
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions::default();
        let mut parser = Parser::new_in_arena(&context, &options, true, output.clone(), false);
        let text = String::from(".a.b > .c:is(.d, !bad, .e) + .f");
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        let range = parser
            .list(
                &mut stream,
                false,
                false,
                CSSNestingType::kNone,
                &mut ResultFlags::default(),
            )
            .unwrap();
        assert_eq!(range.start, 1);
        assert_eq!(output.borrow().as_ptr(), backing);
        assert_eq!(output.borrow()[0].Value().Utf8(), "prefix");
        assert_eq!(output.borrow()[range.start].Value().Utf8(), "f");
        let end = output.borrow().len();
        let text = String::from(".x:not(.a, !bad)");
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        assert!(parser
            .list(
                &mut stream,
                false,
                false,
                CSSNestingType::kNone,
                &mut ResultFlags::default()
            )
            .is_none());
        assert_eq!(output.borrow().len(), end);
        assert_eq!(output.borrow().as_ptr(), backing);
        {
            let _scope = DisallowPseudoElementsScope::new(&parser);
            assert!(parser.disallow_pseudo.get());
        }
        assert!(!parser.disallow_pseudo.get());
    }
    #[test]
    fn enabled_ident_values_and_navigation_locations_use_typed_source_semantics() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let mut options = SelectorParserOptions::default();
        options.features.ident_function = true;
        options.features.route_matching = true;
        let parser = Parser::new(&context, &options, false);
        let text = String::from("ident(foo \"bar\" -2 ident(baz))");
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        let value = parser.custom_ident_value(&mut stream).unwrap();
        let CSSCustomIdentValue::IdentFunction(ref components) = value else {
            panic!("function expected");
        };
        assert_eq!(components.len(), 4);
        assert!(
            matches!(&components[0], CSSIdentComponent::CustomIdent(CSSCustomIdentValue::Literal(atom)) if atom.Utf8() == "foo")
        );
        assert!(matches!(&components[1], CSSIdentComponent::String(atom) if atom.Utf8() == "bar"));
        assert!(
            matches!(&components[2], CSSIdentComponent::Integer(SelectorIdentIntegerValue::Literal(number)) if *number == -2.0)
        );
        assert!(
            matches!(&components[3], CSSIdentComponent::CustomIdent(CSSCustomIdentValue::IdentFunction(values)) if values.len() == 1)
        );
        // Chromium's selector branch calls CSSCustomIdentValue::Value(), not
        // ComputeIdent(). Function-backed values therefore retain a null atom.
        assert!(value.Value().IsNull());
        let parsed = CSSSelectorParser::ParseSelectorWithOptions(
            &String::from("::view-transition-old(ident(hero \"x\" 2).card)"),
            &context,
            CSSNestingType::kNone,
            &options,
        );
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].IdentList()[0].IsNull());
        assert_eq!(parsed[0].IdentList()[1].Utf8(), "card");
        for invalid in ["ident()", "ident(initial)", "ident(1.5)"] {
            let text = String::from(invalid);
            let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
            assert!(
                parser.custom_ident_value(&mut stream).is_none(),
                "{invalid}"
            );
            assert_eq!(stream.Peek().FunctionId(), Some(CSSValueID::kIdent));
        }
        for (input, expected) in [
            (":link-to(--home)", ":link-to(--home)"),
            (":link-to(url(/docs))", ":link-to(url(\"/docs\"))"),
            (":link-to(URL(\"a b\"))", ":link-to(url(\"a b\"))"),
            (
                ":link-to(url-pattern(\"/docs/:id\"))",
                ":link-to(url-pattern(\"/docs/:id\"))",
            ),
        ] {
            let parsed = CSSSelectorParser::ParseSelectorWithOptions(
                &String::from(input),
                &context,
                CSSNestingType::kNone,
                &options,
            );
            assert_eq!(parsed.len(), 1, "{input}");
            assert!(parsed[0].GetNavigationLocation().is_some());
            assert_eq!(
                CSSSelectorList::AdoptSelectorVector(parsed)
                    .SelectorsText()
                    .Utf8(),
                expected
            );
        }
        for invalid in [
            ":link-to(home)",
            ":link-to(url-pattern(a))",
            ":link-to(--home extra)",
            ":link-to(url(\"x\" extra))",
        ] {
            assert!(
                CSSSelectorParser::ParseSelectorWithOptions(
                    &String::from(invalid),
                    &context,
                    CSSNestingType::kNone,
                    &options
                )
                .is_empty(),
                "{invalid}"
            );
        }
    }
    #[test]
    fn forgiving_relative_block_contract_and_default_runtime_recover_without_panics() {
        let context = SelectorParserContext {
            html: true,
            quirks: false,
        };
        let options = SelectorParserOptions::default();
        let text = String::from("relative(> .a)");
        let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
        let list = CSSSelectorParser::ConsumeForgivingRelativeSelectorList(
            &mut stream,
            &context,
            &options,
        )
        .unwrap();
        assert!(list.IsValid());
        assert!(stream.AtEnd());
        for text in ["relative(> .a !)", ".a"] {
            let text = String::from(text);
            let mut stream = CSSParserTokenStream::<CSSTokenizer>::new(StringView::from(&text), 0);
            assert!(CSSSelectorParser::ConsumeForgivingRelativeSelectorList(
                &mut stream,
                &context,
                &options
            )
            .is_none());
        }
        for text in [
            ":has(.a, :no-such)",
            ":nth-child(2n of ::before)",
            ":playing",
            ":link-to(--home)",
        ] {
            assert!(
                CSSSelectorParser::ParseSelector(&String::from(text), &context).is_empty(),
                "{text}"
            );
        }
        for text in [
            ":open",
            "::column",
            "::scroll-button(up)",
            ".tail:nth-child(2n of .item)",
        ] {
            assert!(
                !CSSSelectorParser::ParseSelector(&String::from(text), &context).is_empty(),
                "{text}"
            );
        }
    }
}

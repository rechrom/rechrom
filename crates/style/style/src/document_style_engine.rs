// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Document-owned, concrete assembly of the translated selector, declaration,
//! cascade priority and native style builder. Document roots every final style.
#![allow(non_snake_case)]
use crate::{
    css_property_name::CSSPropertyName,
    css_property_value::CSSPropertyValueBackend,
    css_property_value_set::CSSPropertyValueSetRuleHandle,
    css_selector::{CSSSelector, GetPseudoId, MatchType, PseudoType, RelationType},
    css_selector_list::CSSSelectorList,
    media_queries::{
        media_query_backend::ParseMediaQuerySet,
        production_container_query::{
            ContainerState, ContainerStyleResolver, DocumentMediaValues,
            EvaluateContainerCondition, RegisteredContainerPropertyResolver, RequiredContainerType,
        },
        MediaValuesCachedData,
    },
    parser::{
        css_parser_mode::CSSParserMode,
        css_selector_parser::{CSSSelectorParser, SelectorParserContext},
        production_property_parser::{ParseDeclarationList, PropertyParseErrorKind},
    },
    persistent_selector::{
        PersistentSelectorError, PersistentSelectorMatchSession, PersistentSelectorService,
    },
    production_css_value::{ProductionCSSValueDispatch, PropertyValue, Value},
    properties::{
        css_property::CSSProperty,
        longhand_dispatch::{LonghandApplicationError, ResolvePhysical},
    },
    resolver::{
        cascade_expansion::{self, CascadeExpansionBackend},
        cascade_map::CascadeMap,
        cascade_origin::CascadeOrigin,
        cascade_priority::CascadePriority,
        match_result::{MatchResult, MatchedPropertiesData, MatchedPropertySet},
        production_style_builder::{
            self,
            custom_properties::{
                attributes::{
                    AttributeResolutionState, AttributeResolver, PersistentAttributeSource,
                },
                ComputedVariableValue, CustomProperties, EnvironmentVariableResolver,
                ResolvePropertyValueWithContext, SubstitutionContext, VariableResolutionError,
            },
        },
    },
    valid_property_filter::ValidPropertyFilter,
};
use cssom::css_style_sheet::{CSSStyleSheet, CSSUserAgentStyleSheetRole};
use dom::{
    persistent_document::{
        CompatibilityMode, DOMNamespace, DOMNodeType, DOMOwnerHandle, PseudoContent, PseudoElement,
        ResolvedNodeStyle,
    },
    style_state::{StyleChange, StyleUpdateImpact, StyleUpdateStats},
    Document, UserInteractionState, DOM,
};
use foundation::{
    AtomicString, CSSPropertyID, DisplayAdjustmentContext, EDisplay, EOverflow, EOverflowWrap,
    EScrollbarWidth, EUserModify, EUserSelect, Persistent,
};
use layoutng_style::css::white_space::EWhiteSpace;
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder, ComputedStyleDifference},
    computed_style_base::{FieldDifference, IsAtShadowBoundary},
    computed_style_constants::{PseudoId, PseudoIdFlags},
    content_data::{ImageContentData, TextContentData},
    display_adjustment::AdjustComputedDisplayForLayout,
    style_fetched_image::{FetchedImage, StyleFetchedImage},
    style_image::StyleImage,
};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeSet, HashMap, HashSet},
    fmt,
    rc::Rc,
    sync::Arc,
};

use crate::element_rule_collector::SearchTextRequest;

#[cfg(test)]
#[path = "document_style_selector_service_test.rs"]
mod selector_service_tests;

#[cfg(test)]
#[path = "document_style_custom_property_ownership_test.rs"]
mod custom_property_ownership_tests;

// cpp: style_cascade.cc:2351-2390,2924-2930.  Environment lookup is supplied
// by the document host, while the style bits always belong to the current
// builder.  Keeping these per-cascade flags here prevents an environment owner
// from retaining or mutating a builder after the call returns.
struct CascadeEnvironment<'a> {
    upstream: Option<&'a dyn EnvironmentVariableResolver>,
    has_env: Cell<bool>,
    has_safe_area_bottom: Cell<bool>,
}

impl<'a> CascadeEnvironment<'a> {
    fn new(upstream: Option<&'a dyn EnvironmentVariableResolver>) -> Self {
        Self {
            upstream,
            has_env: Cell::new(false),
            has_safe_area_bottom: Cell::new(false),
        }
    }
    fn ApplyState(&self, builder: &mut ComputedStyleBuilder) {
        if self.has_env.get() {
            builder.SetHasEnv();
        }
        if self.has_safe_area_bottom.get() {
            builder.SetHasEnvSafeAreaInsetBottom();
        }
    }
}

impl EnvironmentVariableResolver for CascadeEnvironment<'_> {
    fn ViewportSegmentsEnabled(&self) -> bool {
        self.upstream
            .is_some_and(EnvironmentVariableResolver::ViewportSegmentsEnabled)
    }
    fn ResolveEnvironmentVariable(
        &self,
        name: &foundation::AtomicString,
        indices: &[u32],
    ) -> Option<Rc<crate::production_css_value::CSSVariableData>> {
        self.has_env.set(true);
        if name == &foundation::AtomicString::from_str("safe-area-inset-bottom") {
            self.has_safe_area_bottom.set(true);
        }
        self.upstream
            .and_then(|source| source.ResolveEnvironmentVariable(name, indices))
    }
    fn HasEnvironmentVariableReference(&self) -> bool {
        self.has_env.get()
    }
}

struct CascadeAttributeState {
    has_attr: Cell<bool>,
}

struct DocumentURLImageResolver<'a> {
    document: &'a Document,
    originating_index: usize,
}

impl production_style_builder::URLImageResolver for DocumentURLImageResolver<'_> {
    fn SVGResourceScope(
        &self,
    ) -> Option<layoutng_style::style::reference_clip_path_operation::SvgClipResourceScope> {
        Some(self.document.SVGResourceScope(self.originating_index))
    }

    fn ResolveMaskSource(
        &self,
        value: &crate::production_css_value::CSSImageValue,
    ) -> Result<std::ptr::NonNull<StyleImage>, LonghandApplicationError> {
        use layoutng_style::style::style_mask_source_image::StyleMaskSourceImage;
        let local = value.url_data.unresolved_url.starts_with('#') || {
            let parsed = url::Url::parse(value.url_data.resolved_url.as_ref()).ok();
            let document = url::Url::parse(self.document.DocumentURL()).ok();
            match (parsed, document) {
                (Some(mut a), Some(mut b)) => {
                    a.set_fragment(None);
                    b.set_fragment(None);
                    a == b
                }
                _ => false,
            }
        };
        crate::resolver::element_style_resources::LoadMaskSourceBranch(
            local,
            || {
                let image = foundation::MakeGarbageCollected(StyleMaskSourceImage::Local(
                    value.url_data.clone(),
                    self.document.SVGResourceScope(self.originating_index),
                ));
                Ok(std::ptr::NonNull::new(image.cast()).unwrap())
            },
            || {
                let fetched = self.ResolveImage(value)?;
                let image = foundation::MakeGarbageCollected(StyleMaskSourceImage::Fetched(
                    value.url_data.clone(),
                    fetched.Image().as_ptr(),
                ));
                Ok(std::ptr::NonNull::new(image.cast()).unwrap())
            },
        )
    }

    fn ResolveImage(
        &self,
        value: &crate::production_css_value::CSSImageValue,
    ) -> Result<production_style_builder::FetchedImageBinding, LonghandApplicationError> {
        self.ResolveImageWithResolution(value, 0.0)
    }

    fn ResolveImageWithResolution(
        &self,
        value: &crate::production_css_value::CSSImageValue,
        override_resolution: f32,
    ) -> Result<production_style_builder::FetchedImageBinding, LonghandApplicationError> {
        let source = value.url_data.resolved_url.Utf8();
        let resource = self.document.ImageResourceFor(&source);
        let id = resource.map_or(0, |resource| resource.id);
        let natural_size = resource.map(|resource| {
            foundation::gfx::SizeF::new(
                (resource.natural_width * resource.resolution_scale) as f32,
                (resource.natural_height * resource.resolution_scale) as f32,
            )
        });
        let resource_resolution = resource.map_or(1.0, |resource| resource.resolution_scale as f32);
        let is_document_image =
            resource.is_some_and(|resource| self.document.ImageResourceIsDocument(resource.id));
        if std::env::var_os("BROWSER_TRACE_STYLE_IMAGES").is_some() {
            eprintln!("style-image-resolved source={source} resource_id={id}");
        }
        let fetched = foundation::MakeGarbageCollected(StyleFetchedImage::new(
            value.url_data.clone(),
            id,
            natural_size,
            resource_resolution,
            override_resolution,
            is_document_image,
            resource.map_or_else(Default::default, |resource| resource.orientation),
            self.document.ImageResourceFailedFor(&source),
        ));
        let image = std::ptr::NonNull::new(fetched.cast::<StyleImage>()).unwrap();
        if let Some(id) = std::num::NonZeroU64::new(id) {
            unsafe { production_style_builder::FetchedImageBinding::FromResource(id, image) }
        } else {
            unsafe { production_style_builder::FetchedImageBinding::FromUnpublished(image) }
        }
    }
}

impl CascadeAttributeState {
    fn new() -> Self {
        Self {
            has_attr: Cell::new(false),
        }
    }
    fn ApplyState(&self, builder: &mut ComputedStyleBuilder) {
        if self.has_attr.get() {
            builder.SetHasAttrFunction();
        }
    }
}

impl AttributeResolutionState for CascadeAttributeState {
    fn SetHasAttrFunction(&self) {
        self.has_attr.set(true);
    }
    fn HasAttrFunction(&self) -> bool {
        self.has_attr.get()
    }
}

#[derive(Debug)]
pub enum DocumentStyleError {
    WrongDocument,
    Selector(PersistentSelectorError),
    Property {
        node: usize,
        property: CSSPropertyID,
        offset: u32,
        operation: &'static str,
        source_file: &'static str,
        source_line: u32,
    },
    Application {
        node: usize,
        error: LonghandApplicationError,
    },
    CascadeIndexOverflow,
    PropertyRegistration(crate::property_registry::PropertyRegistryError),
    Unsupported(&'static str),
}
impl fmt::Display for DocumentStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "style update: {self:?}")
    }
}
impl std::error::Error for DocumentStyleError {}
struct MatchedRule {
    /// DefaultPseudoElementStyle is admitted only for an actual pseudo style
    /// request; its defaults do not advertise author pseudos on the origin.
    user_agent_pseudo_sheet: bool,
    specificity: u32,
    proximity: u32,
    order: usize,
    /// Exact entry emitted by SortAndTransferMatchedRules.
    transferred: Option<crate::resolver::match_result::MatchedProperties>,
}

struct CollectedRules {
    rules: Vec<MatchedRule>,
}

struct ResolvedPseudoOutput {
    pseudo: PseudoElement,
    custom_properties: Rc<CustomProperties>,
}

fn ConsumeProductionRuleCollection(
    stats: &mut TranslatedRuleShadowStats,
    sheets: &[(&CSSStyleSheet, CascadeOrigin)],
    rule_sets_consulted: usize,
    _tree_scope_root: Option<usize>,
    collection: crate::production_rule_matching::ProductionRuleMatchCollection,
    direct_rules: &mut Vec<MatchedRule>,
) {
    let report = collection.report;
    stats.rule_sets_consulted += rule_sets_consulted;
    stats.translated_collector_calls += report.collector_invocations;
    stats.translated_property_sets_transferred += report.transferred_property_sets;
    stats.translated_property_sets_consumed += collection.rules.len();
    stats.candidates_considered += report.candidates_considered;
    stats.selector_matches += report.selector_matches;
    for transferred in collection.rules {
        let sheet_base = sheets[..transferred.sheet_index]
            .iter()
            .map(|(sheet, _)| sheet.rules.len())
            .sum::<usize>();
        stats.typed_declarations_used += 1;
        direct_rules.push(MatchedRule {
            user_agent_pseudo_sheet: transferred.matched.data_.origin == CascadeOrigin::kUserAgent
                && sheets[transferred.sheet_index].0.UserAgentStyleSheetRole()
                    == Some(CSSUserAgentStyleSheetRole::kMarker),
            specificity: transferred.specificity,
            proximity: transferred.proximity,
            order: sheet_base + transferred.position as usize + 1,
            transferred: Some(transferred.matched),
        });
    }
}

/// RuleSet owners supplied to the translated StyleResolver entry. Chromium's
/// DefaultStyleSheets exposes distinct HTML, HTML-quirks, SVG and MathML UA
/// RuleSets and the scoped resolver supplies the active author RuleSets. CSSOM
/// retains the role on each UA sheet so this assembly can preserve those
/// owners without widening StyleEngine's public update boundary.
struct ProductionResolverRuleSets {
    html_user_agent: crate::resolver::production_style_resolver::ProductionResolverRuleSetInput,
    html_quirks_user_agent:
        Option<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>,
    forced_colors_user_agent:
        Option<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>,
    marker_user_agent:
        Option<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>,
    fullscreen_user_agent:
        Option<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>,
    svg_user_agent:
        Option<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>,
    mathml_user_agent:
        Option<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>,
    author: Rc<Vec<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>>,
}

// A style request names a pseudo on the originating element, never a DOM query
// for a synthetic element. Other pseudo families require their own lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PseudoStyleKind {
    Before,
    After,
    FirstLetter,
    Placeholder,
    Marker,
    BeforeMarker,
    AfterMarker,
}

/// `StyleResolver::ForEachUARulesForElement` selects exactly one namespace
/// default sheet before matching. Preserve that owner boundary so MathML's
/// universal rules do not leak into HTML/SVG and SVG tag rules cannot match
/// same-named elements outside the SVG namespace.
fn UserAgentSheetAppliesToDocument(
    sheet: &CSSStyleSheet,
    document: &Document,
    media: &MediaValuesCachedData,
    namespace: DOMNamespace,
) -> bool {
    match sheet.UserAgentStyleSheetRole() {
        Some(CSSUserAgentStyleSheetRole::kHTML) => namespace == DOMNamespace::kHTML,
        Some(CSSUserAgentStyleSheetRole::kHTMLQuirks) => {
            namespace == DOMNamespace::kHTML
                && document.GetCompatibilityMode() == CompatibilityMode::kQuirks
        }
        Some(CSSUserAgentStyleSheetRole::kForcedColors) => {
            namespace == DOMNamespace::kHTML
                && media.forced_colors == crate::media_queries::forced_colors::ForcedColors::kActive
        }
        // CSSDefaultStyleSheets::marker_style_sheet_ is selected by pseudo
        // request rather than originating-element namespace. Its selectors
        // restrict the sheet to ::marker (including nested marker forms).
        Some(CSSUserAgentStyleSheetRole::kMarker) => true,
        Some(CSSUserAgentStyleSheetRole::kFullscreen) => document.HasFullscreenElements(),
        Some(CSSUserAgentStyleSheetRole::kSVG) => namespace == DOMNamespace::kSVG,
        Some(CSSUserAgentStyleSheetRole::kMathML) => namespace == DOMNamespace::kMathML,
        // Untyped UA input is the HTML default supplied by standalone callers.
        // Browser-owned sheets always carry an explicit role.
        None => true,
    }
}

impl PseudoStyleKind {
    const DIRECT: [Self; 5] = [
        Self::Before,
        Self::After,
        Self::FirstLetter,
        Self::Placeholder,
        Self::Marker,
    ];
    const MATCH_TARGETS: [Self; 7] = [
        Self::Before,
        Self::After,
        Self::FirstLetter,
        Self::Placeholder,
        Self::Marker,
        Self::BeforeMarker,
        Self::AfterMarker,
    ];
    fn Id(self) -> PseudoId {
        match self {
            Self::Before => PseudoId::kPseudoIdBefore,
            Self::After => PseudoId::kPseudoIdAfter,
            Self::FirstLetter => PseudoId::kPseudoIdFirstLetter,
            Self::Placeholder => PseudoId::kPseudoIdPlaceholder,
            Self::Marker => PseudoId::kPseudoIdMarker,
            Self::BeforeMarker | Self::AfterMarker => PseudoId::kPseudoIdMarker,
        }
    }

    fn IsNestedMarker(self) -> bool {
        matches!(self, Self::BeforeMarker | Self::AfterMarker)
    }

    fn ValidPropertyFilter(self) -> ValidPropertyFilter {
        match self {
            Self::FirstLetter => ValidPropertyFilter::kFirstLetter,
            Self::Marker | Self::BeforeMarker | Self::AfterMarker => ValidPropertyFilter::kMarker,
            Self::Before | Self::After | Self::Placeholder => ValidPropertyFilter::kNoFilter,
        }
    }
}
struct StyleSelector {
    compiled_subject: Rc<CSSSelectorList>,
    pseudo: Option<PseudoStyleKind>,
    specificity: u32,
    // cpp: rule_set.cc:ExtractBestBucketingValues/AddRuleToRuleSet.
    // RuleSet indexes a selector by the strongest key in its subject compound
    // before invoking SelectorChecker.  The CSSOM projection currently owns
    // rules as source strings, so retain the same precondition beside the
    // compiled selector until the translated RuleSet backend owns production
    // sheets directly.
    bucket: SelectorBucket,
}

#[derive(Clone)]
enum SelectorBucket {
    Id(String),
    Class(String),
    Attribute(String),
    Tag(String),
    CustomPseudo(String),
    Universal,
}

fn SelectorBucketForSubject(selectors: &CSSSelectorList) -> SelectorBucket {
    let Some(subject) = selectors.First() else {
        return SelectorBucket::Universal;
    };
    SelectorBucketForComplexSubject(subject)
}

fn SelectorBucketForComplexSubject(
    subject: crate::css_selector::CSSSelectorComplex<'_>,
) -> SelectorBucket {
    let mut class_name = None;
    let mut attribute = None;
    let mut tag = None;
    for selector in subject.SimpleSelectors() {
        match selector.Match() {
            MatchType::kId => return SelectorBucket::Id(selector.Value().Utf8()),
            MatchType::kClass if class_name.is_none() => class_name = Some(selector.Value().Utf8()),
            MatchType::kAttributeExact
            | MatchType::kAttributeSet
            | MatchType::kAttributeHyphen
            | MatchType::kAttributeList
            | MatchType::kAttributeContain
            | MatchType::kAttributeBegin
            | MatchType::kAttributeEnd
                if attribute.is_none() =>
            {
                attribute = Some(selector.Attribute().LocalName().Utf8())
            }
            MatchType::kTag if tag.is_none() => tag = Some(selector.TagQName().LocalName().Utf8()),
            _ => {}
        }
        if !matches!(
            selector.Relation(),
            RelationType::kSubSelector | RelationType::kPseudoChild
        ) {
            break;
        }
    }
    class_name
        .map(SelectorBucket::Class)
        .or_else(|| attribute.map(SelectorBucket::Attribute))
        .or_else(|| tag.map(SelectorBucket::Tag))
        .unwrap_or(SelectorBucket::Universal)
}

impl SelectorBucket {
    fn Matches(&self, document: &Document, index: usize, quirks: bool) -> bool {
        let node = document.Node(index);
        match self {
            Self::CustomPseudo(expected) => document
                .ShadowPseudoId(index)
                .is_some_and(|name| name == expected),
            Self::Id(expected) => node.FindAttribute("id").is_some_and(|attribute| {
                if quirks {
                    attribute.value.eq_ignore_ascii_case(expected)
                } else {
                    attribute.value == *expected
                }
            }),
            Self::Class(expected) => node.FindAttribute("class").is_some_and(|attribute| {
                attribute.value.split_ascii_whitespace().any(|class_name| {
                    if quirks {
                        class_name.eq_ignore_ascii_case(expected)
                    } else {
                        class_name == expected
                    }
                })
            }),
            Self::Attribute(expected) => node
                .Attributes()
                .iter()
                .any(|attribute| attribute.local_name == *expected),
            Self::Tag(expected) => {
                if node.Namespace() == DOMNamespace::kHTML {
                    node.Name().eq_ignore_ascii_case(expected)
                } else {
                    node.Name() == expected
                }
            }
            Self::Universal => true,
        }
    }
}

#[derive(Default)]
struct RuleCandidateBuckets {
    ids: HashMap<String, Vec<usize>>,
    classes: HashMap<String, Vec<usize>>,
    attributes: HashMap<String, Vec<usize>>,
    tags: HashMap<String, Vec<usize>>,
    universal: Vec<usize>,
}

impl RuleCandidateBuckets {
    fn Add(&mut self, order: usize, bucket: &SelectorBucket) {
        let destination = match bucket {
            SelectorBucket::Id(value) => self.ids.entry(value.clone()).or_default(),
            SelectorBucket::Class(value) => self.classes.entry(value.clone()).or_default(),
            SelectorBucket::Attribute(value) => self.attributes.entry(value.clone()).or_default(),
            SelectorBucket::Tag(value) => self.tags.entry(value.clone()).or_default(),
            SelectorBucket::Universal => &mut self.universal,
            SelectorBucket::CustomPseudo(_) => &mut self.universal,
        };
        if destination.last() != Some(&order) {
            destination.push(order);
        }
    }
    fn Candidates(&self, document: &Document, index: usize, quirks: bool) -> Vec<usize> {
        let node = document.Node(index);
        let mut result = self.universal.clone();
        if let Some(id) = node.FindAttribute("id") {
            let key = quirks.then(|| id.value.to_ascii_lowercase());
            if let Some(rules) = self.ids.get(key.as_deref().unwrap_or(&id.value)) {
                result.extend_from_slice(rules);
            }
        }
        if let Some(classes) = node.FindAttribute("class") {
            for class_name in classes.value.split_ascii_whitespace() {
                let key = quirks.then(|| class_name.to_ascii_lowercase());
                if let Some(rules) = self.classes.get(key.as_deref().unwrap_or(class_name)) {
                    result.extend_from_slice(rules);
                }
            }
        }
        for attribute in node.Attributes() {
            if let Some(rules) = self.attributes.get(&attribute.local_name) {
                result.extend_from_slice(rules);
            }
        }
        if let Some(rules) = self.tags.get(node.Name()) {
            result.extend_from_slice(rules);
        }
        result.sort_unstable();
        result.dedup();
        result
    }
}

/// Equality-only invalidation metadata, never cascade ordering. Origin,
/// layers, rule revisions and environment are guarded by the update context.
/// Preserve duplicates and specificity/proximity; incomplete channels always
/// require ordinary resolution instead of manufacturing a matching signature.
#[derive(Clone)]
struct RetainedRuleSignature {
    native_style_complete: bool,
    variable_dependencies: Option<Rc<RetainedVariableDependencies>>,
    base_len: usize,
    values: Vec<(usize, u32, u32)>,
    complete: bool,
}
impl RetainedRuleSignature {
    fn new(values: Vec<(usize, u32, u32)>, complete: bool) -> Self {
        let base_len = values.len();
        Self::with_base(values, base_len, complete)
    }
    fn with_base(mut values: Vec<(usize, u32, u32)>, base_len: usize, complete: bool) -> Self {
        for (_, _, proximity) in &mut values {
            *proximity = crate::element_rule_collector::CanonicalMatchedRuleProximity(*proximity);
        }
        values[..base_len].sort_unstable();
        values[base_len..].sort_unstable();
        Self {
            native_style_complete: false,
            variable_dependencies: None,
            base_len,
            values,
            complete,
        }
    }
    fn Equivalent(&self, other: &Self) -> bool {
        self.complete
            && other.complete
            && self.base_len == other.base_len
            && self.values == other.values
    }
}

#[derive(Default)]
struct RetainedVariableDependencies {
    referenced: HashSet<std::string::String>,
    local: HashSet<std::string::String>,
}

struct RuleCandidateIndex {
    targets: [RuleCandidateBuckets; 8],
    // Source order is one-based in retained match signatures.
    locations: Vec<(usize, usize)>,
    // Scope/container queries may emit diagnostics at matching time. Ordinary
    // compiled matching cannot; compile diagnostics were emitted before this
    // complete warm index was installed.
    matching_may_report_diagnostics: bool,
}

/// Per-update evidence from the retained translated
/// stylesheet/RuleSet/SelectorChecker path. Ordinary standards-mode author
/// rules and UA-mode user-agent rules use its candidate and match results in
/// the real cascade.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TranslatedRuleShadowStats {
    pub incremental_style_updates: usize,
    pub rule_sets_built: usize,
    /// Matched author or user-agent rules whose retained typed declaration
    /// vector fed the real cascade. These rules never pass through declaration
    /// text parsing.
    pub typed_declarations_used: usize,
    /// Declarations rejected before entering the typed property store.
    pub untransferred_declarations: usize,
    pub elements_probed: usize,
    pub rule_sets_consulted: usize,
    /// Calls into translated ElementRuleCollector::CollectMatchingRules.
    pub translated_collector_calls: usize,
    /// Property sets emitted by translated SortAndTransferMatchedRules.
    pub translated_property_sets_transferred: usize,
    /// Transferred property sets retained in the MatchResult consumed by the
    /// production cascade, without a CSSOM projected-index reconstruction.
    pub translated_property_sets_consumed: usize,
    /// Real Update/Cascade candidates containing at least one retained typed
    /// property set. Empty results are not counted.
    pub translated_style_cascade_attempts: usize,
    /// Attempts whose translated StyleCascade produced the final native style.
    pub translated_style_cascade_applied: usize,
    /// Typed cascade attempts that failed atomically. Failed builders are never
    /// published and no second cascade implementation is invoked.
    pub translated_style_cascade_failures: usize,
    /// Document-element attempts. These are tracked separately because their
    /// initial parent and scheme-sensitive initial color are root-only inputs.
    pub translated_style_cascade_root_attempts: usize,
    /// Root attempts whose translated cascade produced the published style.
    pub translated_style_cascade_root_applied: usize,
    /// Root attempts that failed atomically.
    pub translated_style_cascade_root_failures: usize,
    /// Translated StyleCascade attempts for pseudo-element native styles.
    pub translated_style_cascade_pseudo_attempts: usize,
    /// Pseudo-element attempts whose translated cascade produced the published
    /// native ComputedStyle.
    pub translated_style_cascade_pseudo_applied: usize,
    /// Pseudo-element attempts that failed atomically.
    pub translated_style_cascade_pseudo_failures: usize,
    /// Candidates whose declarations depend on the document property registry.
    /// Registry-dependent cascades are accepted only when the typed backend owns
    /// the registry's syntax and initial-value callbacks.
    pub translated_style_cascade_registered_attempts: usize,
    pub translated_style_cascade_registered_applied: usize,
    pub translated_style_cascade_registered_failures: usize,
    /// Element cascades carrying document-owned sampled animation values.
    pub translated_style_cascade_animation_attempts: usize,
    pub translated_style_cascade_animation_applied: usize,
    pub translated_style_cascade_animation_failures: usize,
    /// Element resolutions that entered the translated
    /// StyleResolver::ApplyBaseStyleNoCache owner after the document had
    /// assembled its complete collaborator bundle.
    pub translated_style_resolver_attempts: usize,
    /// Attempts whose returned ComputedStyle became Update's published style.
    pub translated_style_resolver_applied: usize,
    /// Resolver attempts rejected after a typed owner reported a gap.
    pub translated_style_resolver_failures: usize,
    pub candidates_considered: usize,
    pub selector_matches: usize,
    pub skipped_owner_media: usize,
}

impl RuleCandidateIndex {
    fn Target(pseudo: Option<PseudoStyleKind>) -> usize {
        match pseudo {
            None => 0,
            Some(PseudoStyleKind::Before) => 1,
            Some(PseudoStyleKind::After) => 2,
            Some(PseudoStyleKind::FirstLetter) => 3,
            Some(PseudoStyleKind::Placeholder) => 4,
            Some(PseudoStyleKind::Marker) => 5,
            Some(PseudoStyleKind::BeforeMarker) => 6,
            Some(PseudoStyleKind::AfterMarker) => 7,
        }
    }
}

/// Document-lifetime dependency state written by translated
/// StyleResolver::ResolveStyle. Blink stores the unit bits on Document /
/// StyleEngine and positional flags on the affected parent nodes. Rechrom's
/// invalidation index is deliberately broader, but retaining the exact writes
/// here prevents the production backend from discarding those source side
/// effects and keeps the owner behind DocumentStyleEngine.
#[derive(Default)]
struct ResolverDependencyState {
    uses_root_relative_units: bool,
    uses_glyph_relative_units: bool,
    uses_line_height_units: bool,
    viewport_unit_flags: u32,
    uses_tree_counting_functions: bool,
    forward_positional_parents: HashSet<usize>,
    backward_positional_parents: HashSet<usize>,
    text_link_text_color: Option<foundation::Color>,
}

impl ResolverDependencyState {
    fn Merge(
        &mut self,
        metrics: &crate::resolver::production_style_resolver::ProductionBaseStyleMetrics,
    ) {
        self.uses_root_relative_units |= metrics.uses_root_relative_units;
        self.uses_glyph_relative_units |= metrics.uses_glyph_relative_units;
        self.uses_line_height_units |= metrics.uses_line_height_units;
        self.viewport_unit_flags |= metrics.viewport_unit_flags;
        self.uses_tree_counting_functions |= metrics.uses_tree_counting_functions;
        self.forward_positional_parents
            .extend(metrics.forward_positional_parents.iter().copied());
        self.backward_positional_parents
            .extend(metrics.backward_positional_parents.iter().copied());
        if let Some(color) = &metrics.text_link_text_color {
            self.text_link_text_color = Some(color.clone());
        }
    }
}

/// UA input preserves immutable stylesheet ownership across the host boundary.
/// Borrowed CSSOM values still compare by value; shared sheets are retained
/// directly, like CSSDefaultStyleSheets' owners in Blink. An edit through
/// Rc::make_mut creates a different owner while the engine retains the old one.
#[derive(Clone, Copy)]
pub enum UserAgentStyleSheetInput<'a> {
    Borrowed(&'a [CSSStyleSheet]),
    Shared(&'a [Rc<CSSStyleSheet>]),
}

impl<'a> From<&'a [CSSStyleSheet]> for UserAgentStyleSheetInput<'a> {
    fn from(sheets: &'a [CSSStyleSheet]) -> Self {
        Self::Borrowed(sheets)
    }
}
impl<'a, const N: usize> From<&'a [CSSStyleSheet; N]> for UserAgentStyleSheetInput<'a> {
    fn from(sheets: &'a [CSSStyleSheet; N]) -> Self {
        Self::Borrowed(sheets)
    }
}
impl<'a> From<&'a Vec<CSSStyleSheet>> for UserAgentStyleSheetInput<'a> {
    fn from(sheets: &'a Vec<CSSStyleSheet>) -> Self {
        Self::Borrowed(sheets)
    }
}
impl<'a> From<&'a [Rc<CSSStyleSheet>]> for UserAgentStyleSheetInput<'a> {
    fn from(sheets: &'a [Rc<CSSStyleSheet>]) -> Self {
        Self::Shared(sheets)
    }
}
impl<'a> From<&'a Vec<Rc<CSSStyleSheet>>> for UserAgentStyleSheetInput<'a> {
    fn from(sheets: &'a Vec<Rc<CSSStyleSheet>>) -> Self {
        Self::Shared(sheets)
    }
}
impl<'a> UserAgentStyleSheetInput<'a> {
    fn len(self) -> usize {
        match self {
            Self::Borrowed(sheets) => sheets.len(),
            Self::Shared(sheets) => sheets.len(),
        }
    }
    fn sheet(self, index: usize) -> &'a CSSStyleSheet {
        match self {
            Self::Borrowed(sheets) => &sheets[index],
            Self::Shared(sheets) => &sheets[index],
        }
    }
    fn shared(self, index: usize) -> Option<&'a Rc<CSSStyleSheet>> {
        match self {
            Self::Borrowed(_) => None,
            Self::Shared(sheets) => Some(&sheets[index]),
        }
    }
}

// cpp: document.h:729,2856; document.cc:1103; style_engine.cc:StyleEngine,
// UpdateActiveStyleSheets/UpdateStyleAndLayoutTree; style_resolver.cc:ResolveStyle.
// The source's platform/page services are outside this document-only assembly.
// Unsupported declarations are retained as typed diagnostics and skipped.
// Structural errors keep all previously rooted document styles.
pub struct DocumentStyleEngine {
    document: DOMOwnerHandle,
    selectors: Rc<RefCell<PersistentSelectorService>>,
    interaction_changed_nodes: HashSet<u64>,
    interaction_changed_pseudos: BTreeSet<PseudoType>,
    interaction_invalidation_roots: std::collections::BTreeMap<PseudoType, HashSet<usize>>,
    diagnostics: Vec<DocumentStyleError>,
    active_sheets: Arc<Vec<(Rc<CSSStyleSheet>, CascadeOrigin)>>,
    /// TreeScope owner aligned with `active_sheets`; UA entries have no
    /// author scope. Retaining the previous sequence lets stylesheet removal
    /// invalidate the old ShadowRoot even after CSSOM drops the owner.
    active_sheet_scopes: Arc<Vec<Option<usize>>>,
    // Chromium keeps StyleSheetContents alive from parse through RuleSet
    // construction. CSSOM carries a type-erased transfer owner so this engine
    // can retain the translated typed sheet without exposing it across the
    // package boundary or reparsing its source projection.
    typed_sheets: HashMap<(u64, u64), Rc<crate::production_style_sheet::ProductionStyleSheet>>,
    // cpp: StyleEngine::FindStyleSheetContents/CreateSheet. Inline sheets in
    // different shadow roots share immutable parsed contents, not CSSOM owners.
    inline_sheet_contents:
        HashMap<u64, std::rc::Weak<crate::production_style_sheet::ProductionStyleSheet>>,
    shared_rule_sets: HashMap<
        (usize, bool),
        (
            std::rc::Weak<crate::production_style_sheet::ProductionStyleSheet>,
            Rc<crate::rule_set::RuleSet<crate::production_style_sheet::Backend>>,
        ),
    >,
    typed_sheet_source_parses: usize,
    translated_rule_sets:
        HashMap<(u64, u64), Rc<crate::rule_set::RuleSet<crate::production_style_sheet::Backend>>>,
    matched_properties_cache:
        crate::resolver::production_style_resolver::ProductionMatchedPropertiesCache,
    declarations: Rc<crate::resolver::production_style_resolver::ProductionDeclarationCache>,
    resolver_dependencies: ResolverDependencyState,
    // Immutable per-family font inputs are derived from this Style-owned
    // document-author registry, never from Layout or a Page-side CSS scan.
    font_feature_values: crate::font_feature_values_registry::FontFeatureValuesRegistry,
    translated_rule_set_environment: Option<MediaValuesCachedData>,
    // Blink discovers viewport-unit users by walking the live element tree
    // when the viewport dirty flags are consumed. Keep the equivalent exact
    // result set alongside the resolved styles so the Rust boundary does not
    // rescan every connected element on each native resize sample.
    viewport_dependent_elements: HashSet<usize>,
    container_dependent_elements: HashSet<usize>,
    // SVGResources retains the live clients of paint servers. Keep the same
    // sparse resident set so an ordinary style update does not rediscover SVG
    // clients by scanning every resolved HTML style in the document.
    svg_style_elements: HashSet<usize>,
    translated_rule_shadow_stats: TranslatedRuleShadowStats,
    sheet_revision: Option<u64>,
    environment: Option<(MediaValuesCachedData, CompatibilityMode)>,
    matches: HashMap<usize, RetainedRuleSignature>,
    cascade_values: HashMap<usize, HashMap<CSSPropertyID, Rc<Value>>>,
    custom_properties: HashMap<usize, Rc<CustomProperties>>,
    sheet_dependencies: Vec<SelectorDependencies>,
    // RuleFeatureSet-style metadata belongs to immutable parsed contents, not
    // its position or CSSOM attachment. Weak ownership follows the text cache.
    shared_selector_dependencies: HashMap<
        usize,
        (
            std::rc::Weak<crate::production_style_sheet::ProductionStyleSheet>,
            SelectorDependencies,
        ),
    >,
    // Source text owns a small set of compatibility/nesting variants.  Keep
    // it as the outer key so hot candidate lookups can borrow `&str` without
    // allocating a fresh String for every rule/element pair.
    style_selectors: HashMap<String, HashMap<(bool, bool), Vec<StyleSelector>>>,
    rule_candidate_index: Option<RuleCandidateIndex>,
    // cpp: StyleRuleMedia retains a parsed MediaQuerySet and RuleSet stores
    // the evaluated result. The CSSOM projection currently carries exact
    // source strings, so retain the equivalent parsed owner here rather than
    // reparsing every condition for every candidate element.
    media_query_cache: HashMap<
        String,
        Rc<
            crate::media_queries::media_query_set::MediaQuerySet<
                crate::css_numeric_literal_value::CSSNumericLiteralValue,
            >,
        >,
    >,
    container_states: HashMap<usize, ContainerState>,
    layout_container_nodes: HashSet<usize>,
    invalidated_container_nodes: HashSet<usize>,
    property_registry: crate::property_registry::PropertyRegistry,
    registered_container_properties: Option<Rc<dyn RegisteredContainerPropertyResolver>>,
    color_scheme_settings: production_style_builder::ColorSchemeSettings,
    font_metrics: Option<Rc<dyn production_style_builder::FontMetricsResolver>>,
}
/// Read-only capacities of the production custom-property owner. Map bucket
/// figures exclude control bytes/allocator rounding. Token payloads and maps
/// are deduplicated by their actual retained Rc backing, never by node count.
#[derive(Clone, Debug, Default)]
pub struct CustomPropertyMemoryDiagnostics {
    pub node_entries: usize,
    pub custom_records: usize,
    pub value_maps: usize,
    pub value_entries: usize,
    pub value_key_capacity: usize,
    pub value_bucket_payload_capacity_estimate: usize,
    pub registered_maps: usize,
    pub registered_entries: usize,
    pub registered_key_capacity: usize,
    pub variable_data_owners: usize,
    pub token_inline_capacity: usize,
}
impl DocumentStyleEngine {
    pub fn CustomPropertyMemoryDiagnostics(&self) -> CustomPropertyMemoryDiagnostics {
        let mut stats = CustomPropertyMemoryDiagnostics {
            node_entries: self.custom_properties.len(),
            ..Default::default()
        };
        let (mut records, mut maps, mut registered, mut data) = (
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
        );
        for custom in self.custom_properties.values() {
            if !records.insert(Rc::as_ptr(custom)) {
                continue;
            }
            stats.custom_records += 1;
            if maps.insert(Rc::as_ptr(&custom.values)) {
                stats.value_maps += 1;
                stats.value_entries += custom.values.len();
                stats.value_key_capacity += custom
                    .values
                    .keys()
                    .map(std::string::String::capacity)
                    .sum::<usize>();
                stats.value_bucket_payload_capacity_estimate += custom.values.capacity()
                    * std::mem::size_of::<(std::string::String, ComputedVariableValue)>();
                for value in custom.values.values() {
                    if let ComputedVariableValue::Data(value) = value {
                        if data.insert(Rc::as_ptr(value)) {
                            stats.variable_data_owners += 1;
                            stats.token_inline_capacity += value.tokens.capacity()
                                * std::mem::size_of::<crate::production_css_value::VariableToken>();
                        }
                    }
                }
            }
            if registered.insert(Rc::as_ptr(&custom.registered_values)) {
                stats.registered_maps += 1;
                stats.registered_entries += custom.registered_values.len();
                stats.registered_key_capacity += custom
                    .registered_values
                    .keys()
                    .map(std::string::String::capacity)
                    .sum::<usize>();
            }
        }
        stats
    }

    pub fn new(owner: &DOM) -> Self {
        Self {
            document: owner.GetDocument().RootHandle(),
            selectors: Default::default(),
            interaction_changed_nodes: HashSet::new(),
            interaction_changed_pseudos: BTreeSet::new(),
            interaction_invalidation_roots: std::collections::BTreeMap::new(),
            diagnostics: vec![],
            active_sheets: Default::default(),
            active_sheet_scopes: Default::default(),
            typed_sheets: HashMap::new(),
            inline_sheet_contents: HashMap::new(),
            shared_rule_sets: HashMap::new(),
            typed_sheet_source_parses: 0,
            translated_rule_sets: HashMap::new(),
            matched_properties_cache: Default::default(),
            declarations: Default::default(),
            resolver_dependencies: Default::default(),
            font_feature_values: Default::default(),
            translated_rule_set_environment: None,
            viewport_dependent_elements: HashSet::new(),
            container_dependent_elements: HashSet::new(),
            svg_style_elements: HashSet::new(),
            translated_rule_shadow_stats: Default::default(),
            sheet_revision: None,
            environment: None,
            matches: HashMap::new(),
            cascade_values: HashMap::new(),
            custom_properties: HashMap::new(),
            sheet_dependencies: vec![],
            shared_selector_dependencies: HashMap::new(),
            style_selectors: HashMap::new(),
            rule_candidate_index: None,
            media_query_cache: HashMap::new(),
            container_states: HashMap::new(),
            layout_container_nodes: HashSet::new(),
            invalidated_container_nodes: HashSet::new(),
            property_registry: Default::default(),
            registered_container_properties: None,
            color_scheme_settings: Default::default(),
            font_metrics: None,
        }
    }

    pub fn SetFontMetricsResolver(
        &mut self,
        resolver: Rc<dyn production_style_builder::FontMetricsResolver>,
    ) {
        self.font_metrics = Some(resolver);
    }
    /// Layout/scroll/anchor owners publish the complete state after their update.
    /// Descendant query matches must be revisited even when CSS/DOM are unchanged.
    pub fn SetContainerState(
        &mut self,
        owner: &mut DOM,
        node: usize,
        state: Option<ContainerState>,
    ) -> Result<bool, DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let changed = self.container_states.get(&node) != state.as_ref();
        if changed {
            if let Some(state) = state {
                self.container_states.insert(node, state);
            } else {
                self.container_states.remove(&node);
            }
            self.invalidated_container_nodes.insert(node);
        }
        Ok(changed)
    }
    /// Replace the completed layout's size observations. Scroll/anchor state
    /// stays owned by its separate publisher; unavailable observations remain
    /// unavailable rather than becoming a negative query result.
    pub fn SetContainerSizes(
        &mut self,
        owner: &mut DOM,
        sizes: impl IntoIterator<Item = (usize, f64, f64)>,
    ) -> Result<bool, DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let mut seen = HashSet::new();
        let mut changed = false;
        for (node, width, height) in sizes {
            let Some(native) = owner
                .GetDocument()
                .ResolvedStyleFor(node)
                .and_then(|s| unsafe { s.native_style.Get().as_ref() })
            else {
                continue;
            };
            seen.insert(node);
            let mut state = self
                .container_states
                .get(&node)
                .cloned()
                .unwrap_or_else(|| {
                    ContainerState::SizeOnly(width, height, native.GetWritingDirection())
                });
            state.width = Some(width);
            state.height = Some(height);
            changed |= self.SetContainerState(owner, node, Some(state))?;
        }
        let removed = self
            .layout_container_nodes
            .difference(&seen)
            .copied()
            .collect::<Vec<_>>();
        for node in removed {
            let state = self
                .container_states
                .get(&node)
                .cloned()
                .and_then(|mut state| {
                    state.width = None;
                    state.height = None;
                    (state.scroll_state_available || state.anchored_state_available)
                        .then_some(state)
                });
            changed |= self.SetContainerState(owner, node, state)?;
        }
        self.layout_container_nodes = seen;
        Ok(changed)
    }
    /// The document registration owner supplies actual syntax-computed reads.
    pub fn SetRegisteredContainerProperties(
        &mut self,
        owner: &mut DOM,
        resolver: Option<Rc<dyn RegisteredContainerPropertyResolver>>,
    ) -> Result<(), DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        self.registered_container_properties = resolver;
        owner.GetDocumentMut().StyleStateMut().all_dirty = true;
        Ok(())
    }
    /// The real page/style-engine owner supplies page color-scheme and
    /// force-dark settings; media owns the preferred scheme.
    pub fn SetColorSchemeSettings(
        &mut self,
        owner: &mut DOM,
        settings: production_style_builder::ColorSchemeSettings,
    ) -> Result<(), DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        if self.color_scheme_settings != settings {
            self.color_scheme_settings = settings;
            owner.GetDocumentMut().StyleStateMut().all_dirty = true;
        }
        Ok(())
    }
    pub fn PropertyRegistry(&self) -> &crate::property_registry::PropertyRegistry {
        &self.property_registry
    }
    /// A typed host API for CSS.registerProperty. The script binding remains
    /// separate; no stylesheet declaration is promoted to a script slot.
    pub fn RegisterProperty(
        &mut self,
        owner: &mut DOM,
        name: foundation::AtomicString,
        registration: crate::property_registry::PropertyRegistration,
    ) -> Result<(), DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        self.property_registry
            .RegisterProperty(name, registration)
            .map_err(DocumentStyleError::PropertyRegistration)?;
        owner.GetDocumentMut().StyleStateMut().all_dirty = true;
        Ok(())
    }
    pub fn Diagnostics(&self) -> &[DocumentStyleError] {
        &self.diagnostics
    }
    // cpp: Element::SetHovered/SetActive/FocusWithinStateChanged and
    // AffectedByPseudoStateChange (element.cc:6880-6998,9140-9151,13146-13192).
    // SelectorChecker's retained flags select local and relational invalidation;
    // a state change on an unrelated ancestor does not dirty the document.
    fn RecordUserActionChange(&mut self, document: &Document, id: u64, pseudo: PseudoType) {
        use crate::resolver::match_flags::MatchFlag;
        use crate::selector_checker::ElementInvalidationFlag as F;
        let Some(index) = document.FindNodeById(id) else {
            return;
        };
        let style = document
            .ResolvedStyleFor(index)
            .and_then(|style| unsafe { style.native_style.Get().as_ref() });
        let selectors = self.selectors.borrow();
        let subject = selectors.SubjectMatchFlags(index);
        let (local, children, ancestors) = match pseudo {
            PseudoType::kPseudoHover => (
                style.is_none_or(|s| s.AffectedByHover() || s.HasEffectiveAppearance())
                    || subject & MatchFlag::kAffectedByHover as u8 != 0,
                Some(F::ChildrenOrSiblingsAffectedByHover),
                Some(F::AncestorsOrSiblingsAffectedByHoverInHas),
            ),
            PseudoType::kPseudoActive => (
                style.is_none_or(|s| s.AffectedByActive() || s.HasEffectiveAppearance())
                    || subject & MatchFlag::kAffectedByActive as u8 != 0,
                Some(F::ChildrenOrSiblingsAffectedByActive),
                Some(F::AncestorsOrSiblingsAffectedByActiveInHas),
            ),
            PseudoType::kPseudoFocusWithin => (
                style.is_some_and(|s| s.AffectedByFocusWithin())
                    || subject & MatchFlag::kAffectedByFocusWithin as u8 != 0,
                Some(F::ChildrenOrSiblingsAffectedByFocusWithin),
                Some(F::AncestorsOrSiblingsAffectedByFocusInHas),
            ),
            PseudoType::kPseudoFocus => (
                true,
                Some(F::ChildrenOrSiblingsAffectedByFocus),
                Some(F::AncestorsOrSiblingsAffectedByFocusInHas),
            ),
            PseudoType::kPseudoFocusVisible => (
                true,
                Some(F::ChildrenOrSiblingsAffectedByFocusVisible),
                Some(F::AncestorsOrSiblingsAffectedByFocusVisibleInHas),
            ),
            _ => (true, None, None),
        };
        if local {
            self.interaction_changed_nodes.insert(id);
        }
        let relational = children
            .into_iter()
            .chain(ancestors)
            .any(|flag| selectors.ElementHasInvalidationFlag(index, flag));
        // Scope activation uses the separate compiled scope dependency owner.
        let scoped = self
            .translated_rule_sets
            .values()
            .any(|rules| rules.Features().scope_pseudo_states.contains(&pseudo));
        if relational || scoped {
            self.interaction_changed_pseudos.insert(pseudo);
            self.interaction_invalidation_roots
                .entry(pseudo)
                .or_default()
                .insert(index);
        }
        if browser_tracing::enabled() {
            browser_tracing::instant(
                "style_invalidation",
                "StyleUserActionInvalidation",
                &[
                    ("node_id", id as f64),
                    ("pseudo", pseudo as u16 as f64),
                    ("local", local as u8 as f64),
                    ("relational", relational as u8 as f64),
                    ("scoped", scoped as u8 as f64),
                ],
            );
        }
    }

    /// Publish the immutable interaction snapshot used by selector matching.
    /// Interaction owns transitions; the document retains matching dependency
    /// flags and schedules the corresponding local and relational invalidation.
    pub fn SetUserInteractionState(
        &mut self,
        owner: &mut DOM,
        state: UserInteractionState,
    ) -> Result<bool, DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let previous = self.selectors.borrow().interaction;
        if previous == state {
            return Ok(false);
        }
        use crate::selector_checker::SelectorCheckerBackend;
        let old_actions = {
            let mut selectors = self.selectors.borrow_mut();
            let session = selectors.StyleMatchSession(owner.GetDocument());
            let backend = session.Backend();
            ConnectedElements(owner.GetDocument())
                .map(|index| {
                    (
                        index,
                        backend.IsHovered(&index),
                        backend.IsActive(&index),
                        backend.HasFocusWithin(&index),
                    )
                })
                .collect::<Vec<_>>()
        };
        self.selectors.borrow_mut().interaction = state;
        self.interaction_changed_nodes.extend(
            [previous.focused_node_id, state.focused_node_id]
                .into_iter()
                .flatten(),
        );
        // Element::PseudoStateChanged schedules selector invalidation; it does
        // not force every ComputedStyle in the document to be rebuilt. Reuse
        // the retained rule matcher below, including its incomplete-signature
        // fallback, to decide which elements actually require recalc.
        for (old, new, pseudo) in [
            (
                previous.focused_node_id,
                state.focused_node_id,
                PseudoType::kPseudoFocus,
            ),
            (
                previous.focus_visible_node_id,
                state.focus_visible_node_id,
                PseudoType::kPseudoFocusVisible,
            ),
            (
                previous.hovered_node_id,
                state.hovered_node_id,
                PseudoType::kPseudoHover,
            ),
            (
                previous.pressed_node_id,
                state.pressed_node_id,
                PseudoType::kPseudoActive,
            ),
            (
                previous.multi_select_focused_option_id,
                state.multi_select_focused_option_id,
                PseudoType::kPseudoMultiSelectFocus,
            ),
        ] {
            if old != new {
                for id in [old, new].into_iter().flatten() {
                    self.RecordUserActionChange(owner.GetDocument(), id, pseudo);
                    if pseudo == PseudoType::kPseudoFocus {
                        for related in [
                            PseudoType::kPseudoFocusVisible,
                            PseudoType::kPseudoSpatialNavigationFocus,
                            PseudoType::kPseudoActiveOption,
                        ] {
                            self.RecordUserActionChange(owner.GetDocument(), id, related);
                        }
                    }
                }
            }
        }
        let mut transitions = Vec::new();
        {
            // Reuse the production user-action traversal, including label
            // controls, flat-tree ancestry and top-layer boundaries. Blink
            // updates local style on every element whose state actually flips.
            let mut selectors = self.selectors.borrow_mut();
            let session = selectors.StyleMatchSession(owner.GetDocument());
            let backend = session.Backend();
            for (index, hovered, active, focus_within) in old_actions {
                for (changed, pseudo) in [
                    (
                        hovered != backend.IsHovered(&index),
                        PseudoType::kPseudoHover,
                    ),
                    (
                        active != backend.IsActive(&index),
                        PseudoType::kPseudoActive,
                    ),
                    (
                        focus_within != backend.HasFocusWithin(&index),
                        PseudoType::kPseudoFocusWithin,
                    ),
                ] {
                    if changed {
                        transitions.push((owner.GetDocument().Node(index).Id(), pseudo));
                    }
                }
            }
        }
        for (id, pseudo) in transitions {
            self.RecordUserActionChange(owner.GetDocument(), id, pseudo);
        }
        Ok(true)
    }
    pub fn HasContainerQueries(&self) -> bool {
        self.active_sheets.iter().any(|(sheet, _)| {
            sheet
                .rules
                .iter()
                .any(|rule| !rule.container_conditions.is_empty())
        })
    }
    pub fn TakeDiagnostics(&mut self) -> Vec<DocumentStyleError> {
        std::mem::take(&mut self.diagnostics)
    }
    #[cfg(test)]
    pub fn TypedSheetOwnershipStats(&self) -> (usize, usize) {
        (self.typed_sheets.len(), self.typed_sheet_source_parses)
    }
    #[cfg(test)]
    pub fn TranslatedRuleShadowStats(&self) -> TranslatedRuleShadowStats {
        self.translated_rule_shadow_stats
    }
    pub fn Update<'a>(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: impl Into<UserAgentStyleSheetInput<'a>>,
    ) -> Result<StyleUpdateImpact, DocumentStyleError> {
        self.UpdateWithEnvironment(owner, media, ua, None)
    }
    pub fn UpdateWithEnvironment<'a>(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: impl Into<UserAgentStyleSheetInput<'a>>,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<StyleUpdateImpact, DocumentStyleError> {
        self.UpdateForStyleRequest(owner, media, ua.into(), environment_variables, None)
    }

    /// Element::EnsureComputedStyle: after normal style update, resolve only the
    /// missing flat-tree ancestor chain of a script-query target. Ensured styles
    /// are query caches, not layout styles, and are invalidated by normal recalc.
    pub fn EnsureComputedStyle<'a>(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: impl Into<UserAgentStyleSheetInput<'a>>,
        node: usize,
    ) -> Result<(), DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let document = owner.GetDocument();
        if !ConnectedElement(document, node) || document.ResolvedStyleFor(node).is_some() {
            return Ok(());
        }
        let mut top = node;
        while let Some(parent) = FlatTreeStyleParentElement(document, top) {
            if document.ResolvedStyleFor(parent).is_some() {
                break;
            }
            top = parent;
        }
        if FlatTreeStyleParentElement(document, top).is_none()
            && document.Node(top).Parent() != Some(document.Root())
        {
            return Ok(());
        }
        // Like Blink, callers first update the document's normal lifecycle.
        // Detached mutations may leave DOM facts pending without dirtying the
        // active style tree; the embedding need not schedule a lifecycle for them.
        debug_assert!(!document.StyleState().all_dirty);
        self.UpdateForStyleRequest(owner, media, ua.into(), None, Some(node))?;
        Ok(())
    }

    fn UpdateForStyleRequest(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: UserAgentStyleSheetInput<'_>,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        ensure_target: Option<usize>,
    ) -> Result<StyleUpdateImpact, DocumentStyleError> {
        let mut update_trace = browser_tracing::span("style", "StyleEngine.Update");
        let mut sheets_trace =
            browser_tracing::span("style_rules", "StyleEngine.UpdateActiveStyleSheets");
        self.diagnostics.clear();
        self.translated_rule_shadow_stats = Default::default();
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let state = owner.GetDocument().StyleState();
        // Chromium's StyleFetchedImage retains identity while its
        // ImageResourceContent changes underneath it, so a matched-properties
        // cache hit observes the completed or failed resource. Rechrom publishes an
        // immutable StyleFetchedImage snapshot across the style boundary.
        // Resource invalidation must therefore reject cached snapshots before
        // rebuilding the affected paint clients.
        let resource_changed = state
            .changes
            .iter()
            .any(|change| matches!(change, StyleChange::Resource(_)));
        if state.all_dirty
            || !state.dirty_style_elements.is_empty()
            || !state.inserted_style_subtrees.is_empty()
        {
            self.SynchronizeStyleSheets(owner);
        }
        let document = owner.GetDocument();
        if document.RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        // Blink's ApplyRuleSetChanges invalidates against old and new rule sets.
        // Retain exact source snapshots: removals and source-order/layer changes
        // are observable even when the DOM and declaration text are unchanged.
        let revision = document.StyleState().sheet_revision;
        let mut previous_ua = self
            .active_sheets
            .iter()
            .filter(|(_, origin)| *origin == CascadeOrigin::kUserAgent)
            .map(|(sheet, _)| sheet);
        let mut shared_checks = 0;
        let mut value_comparisons = 0;
        let ua_unchanged = (0..ua.len()).all(|index| {
            let Some(previous) = previous_ua.next() else {
                return false;
            };
            if let Some(shared) = ua.shared(index) {
                shared_checks += 1;
                Rc::ptr_eq(previous, shared)
            } else {
                value_comparisons += 1;
                previous.as_ref() == ua.sheet(index)
            }
        }) && previous_ua.next().is_none();
        sheets_trace.set("ua_shared_owner_checks", shared_checks as f64);
        sheets_trace.set("ua_value_comparisons", value_comparisons as f64);
        // cpp: StyleEngine::UpdateActiveStyleSheets / NeedsActiveStyleSheetUpdate.
        // The DOM version includes owner connectivity, order and adoption. Reuse
        // both retained outputs when none of those inputs changed.
        let (active_sheets, active_sheet_scopes) = if self.sheet_revision == Some(revision)
            && ua_unchanged
        {
            (self.active_sheets.clone(), self.active_sheet_scopes.clone())
        } else {
            let active_scope_sheets = document.ActiveStyleSheetsWithTreeScopes();
            // ActiveStyleSheets holds stylesheet owners in Blink, not copies
            // of their rule/declaration projections. Reuse immutable snapshots
            // across list changes while retaining value checks for directly
            // edited CSSOM projections and owner metadata.
            let mut trace =
                browser_tracing::span("style_rules", "StyleEngine.SnapshotActiveSheets");
            let previous = self
                .active_sheets
                .iter()
                .map(|(sheet, origin)| ((sheet.Identity(), sheet.Revision(), *origin as u8), sheet))
                .collect::<HashMap<_, _>>();
            let mut reused = 0;
            let mut copied = 0;
            let mut retain = |sheet: &CSSStyleSheet, origin: CascadeOrigin| {
                let key = (sheet.Identity(), sheet.Revision(), origin as u8);
                if let Some(previous) = previous.get(&key).filter(|previous| {
                    previous.HasCurrentSource() == sheet.HasCurrentSource()
                        && previous.as_ref() == sheet
                }) {
                    reused += 1;
                    (*previous).clone()
                } else {
                    copied += 1;
                    Rc::new(sheet.clone())
                }
            };
            let mut sheets = Vec::with_capacity(ua.len() + active_scope_sheets.len());
            let mut shared_ua = 0;
            for index in 0..ua.len() {
                let sheet = if let Some(shared) = ua.shared(index) {
                    shared_ua += 1;
                    shared.clone()
                } else {
                    retain(ua.sheet(index), CascadeOrigin::kUserAgent)
                };
                sheets.push((sheet, CascadeOrigin::kUserAgent));
            }
            for (_, sheet) in &active_scope_sheets {
                sheets.push((
                    retain(sheet, CascadeOrigin::kAuthor),
                    CascadeOrigin::kAuthor,
                ));
            }
            trace.set("reused", reused as f64);
            trace.set("copied", copied as f64);
            trace.set("shared_ua", shared_ua as f64);
            let scopes = std::iter::repeat_n(None, ua.len())
                .chain(active_scope_sheets.iter().map(|(scope, _)| Some(*scope)))
                .collect::<Vec<_>>();
            (Arc::new(sheets), Arc::new(scopes))
        };
        // Preserve old RuleSet identities across synchronization/pruning for
        // Blink's active-sheet comparison below.
        let viewport_only_environment_change = self
            .translated_rule_set_environment
            .as_ref()
            .is_some_and(|previous| MediaDiffersOnlyByViewport(previous, media));
        let old_rule_sets = (!Arc::ptr_eq(&self.active_sheets, &active_sheets)
            || viewport_only_environment_change)
            .then(|| {
                self.active_sheets
                    .iter()
                    .map(|(sheet, _)| {
                        self.translated_rule_sets
                            .get(&(sheet.Identity(), sheet.Revision()))
                            .cloned()
                    })
                    .collect::<Vec<_>>()
            });
        self.SynchronizeTypedSheets(document, &active_sheets);
        // Ordinary standards-mode author and UA-mode user-agent matching
        // consume these retained RuleSets below. Rebuild only when the sheet
        // revision or media environment changes; advanced recalc contexts
        // remain explicit compatibility-collector fallbacks.
        let retained_viewport_environment =
            self.SynchronizeTranslatedRuleSets(&active_sheets, media);
        // DOM invalidations were recorded while the previous active RuleSets
        // still owned their feature data. Keep those owners through this
        // batch, just as mutation-time PendingInvalidations do in Blink; sheet
        // synchronization may already have pruned their old revision keys.
        // New rules participate too when sheet and DOM changes are batched.
        let attribute_invalidation_rules = active_sheets
            .iter()
            .map(|(sheet, _)| {
                self.translated_rule_sets
                    .get(&(sheet.Identity(), sheet.Revision()))
                    .cloned()
            })
            .chain(old_rule_sets.iter().flatten().cloned())
            .collect::<Option<Vec<_>>>();
        drop(sheets_trace);
        let invalidation_trace =
            browser_tracing::span("style_invalidation", "StyleEngine.Invalidate");
        let global_invalidation =
            browser_tracing::span("style_invalidation", "StyleEngine.InvalidateGlobalInputs");
        let font_features_changed = self.SynchronizeFontFeatureValues(
            document,
            media,
            &active_sheets,
            active_sheet_scopes.as_slice(),
        );
        let function_registry = {
            let rule_sets = active_sheets
                .iter()
                .filter_map(|(sheet, _)| {
                    self.translated_rule_sets
                        .get(&(sheet.Identity(), sheet.Revision()))
                        .cloned()
                })
                .collect::<Vec<_>>();
            crate::production_style_sheet::ProductionFunctionRegistry::FromRuleSets(&rule_sets)
        };
        let registry_changed = self
            .property_registry
            .UpdateDeclaredProperties(&active_sheets, &DocumentMediaValues::new(document, media));
        let environment = (media.clone(), document.GetCompatibilityMode());
        // Scope activation can change when its triggering context changes.
        // Container rules carry their dependency flags on ComputedStyle,
        // including when the previous query result was false.
        let conservative_scope_invalidation = active_sheets.iter().any(|(sheet, _)| {
            sheet
                .rules
                .iter()
                .any(|rule| !rule.scope_conditions.is_empty())
        }) && (!document.StyleState().changes.is_empty()
            || !self.invalidated_container_nodes.is_empty());
        let environment_changed = self.environment.as_ref() != Some(&environment);
        let all_dirty = font_features_changed
            || registry_changed
            || document.StyleState().all_dirty
            || (environment_changed && !viewport_only_environment_change)
            || conservative_scope_invalidation;
        let sheet_changes = ChangedSheets(&self.active_sheets, &active_sheets);
        let media_rule_sets_changed =
            viewport_only_environment_change && !retained_viewport_environment;
        let rule_sets_changed = !sheet_changes.is_empty() || media_rule_sets_changed;
        if font_features_changed
            || registry_changed
            || (environment_changed && !viewport_only_environment_change)
            // cpp: StyleEngine::ApplyRuleSetChanges. MatchResult keys own
            // property-set identity, order and cascade metadata. Appending
            // ordinary document or shadow rules does not invalidate unrelated
            // entries. Global at-rule/environment/resource inputs still do.
            || sheet_changes.iter().any(|&i| {
                [&self.active_sheets, &active_sheets].into_iter()
                    .any(|sheets| sheets.get(i).is_some_and(|(sheet, _)| {
                        !sheet.layer_order.is_empty() || !sheet.font_faces.is_empty()
                            || !sheet.property_rules.is_empty()
                            || sheet.ParsedBacking::<crate::production_style_sheet::ProductionStyleSheet>()
                                .is_none_or(|typed| !typed.functions.is_empty() || !typed.mixins.is_empty())
                    }))
            })
            || environment_variables.is_some()
            || resource_changed
        {
            self.matched_properties_cache.Clear();
        } else if viewport_only_environment_change {
            self.matched_properties_cache.ClearViewportDependent();
        }
        // cpp: matched_properties_cache.cc:392-431. Blink performs this from
        // the Oilpan weak callback; the retained Rust document owner reaches
        // the same lifecycle point once per style update.
        self.matched_properties_cache.Clean();
        // cpp: StyleEngine::ApplyRuleSetChanges(TreeScope&, old, new, diffs).
        // Sheet order is local to a TreeScope. Inserting a shadow sheet can
        // shift every later slot in the flattened host projection without
        // changing any of those scopes' sheets.
        let changed_tree_scopes = self
            .active_sheet_scopes
            .iter()
            .chain(active_sheet_scopes.iter())
            .flatten()
            .copied()
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|scope| {
                let old = self
                    .active_sheets
                    .iter()
                    .zip(self.active_sheet_scopes.iter())
                    .filter(|(_, candidate)| **candidate == Some(*scope))
                    .map(|(sheet, _)| sheet);
                let new = active_sheets
                    .iter()
                    .zip(active_sheet_scopes.iter())
                    .filter(|(_, candidate)| **candidate == Some(*scope))
                    .map(|(sheet, _)| sheet);
                !old.eq(new)
            })
            .collect::<HashSet<_>>();
        let document_sheets_changed = changed_tree_scopes.contains(&document.Root())
            || !ua_unchanged
            || media_rule_sets_changed;
        if !sheet_changes.is_empty() && !document_sheets_changed {
            // This equality-only sidecar stores flattened rule positions.
            // Revoke its certificates after reindexing; existing native styles
            // stay valid. Recalc below visits only the affected shadow scopes,
            // hosts/slotted elements and exposed descendant scopes.
            self.matches.clear();
        }
        let quirks_changed = self
            .environment
            .as_ref()
            .is_none_or(|(_, mode)| *mode != environment.1);
        if quirks_changed
            || rule_sets_changed
            || self.active_sheet_scopes.as_slice() != active_sheet_scopes.as_slice()
        {
            self.rule_candidate_index = None;
        }
        // A document stylesheet may be inserted/removed without changing its
        // TreeScope owner. Shadow scopes never use the cross-revision empty case.
        // ApplyRuleSetChanges is scoped to the changed RuleSets. An unrelated
        // shadow TreeScope does not make document RuleSet invalidation
        // inexact; only inputs shared by selector/cascade evaluation do.
        let rule_set_invalidation_context_unchanged =
            (self.environment.as_ref() == Some(&environment) || viewport_only_environment_change)
                && !font_features_changed
                && !registry_changed
                && environment_variables.is_none();
        let signature_non_rule_context_unchanged = self
            .active_sheet_scopes
            .iter()
            .flatten()
            .chain(active_sheet_scopes.iter().flatten())
            .all(|scope| *scope == document.Root())
            && rule_set_invalidation_context_unchanged;
        let signature_context_unchanged = signature_non_rule_context_unchanged
            && self.active_sheet_scopes.as_slice() == active_sheet_scopes.as_slice()
            && !rule_sets_changed;
        let ordinary_signature_complete = environment_variables.is_none()
            && active_sheet_scopes
                .iter()
                .flatten()
                .all(|scope| *scope == document.Root())
            && active_sheets.iter().all(|(sheet, _)| {
                let key = (sheet.Identity(), sheet.Revision());
                self.typed_sheets
                    .get(&key)
                    .zip(self.translated_rule_sets.get(&key))
                    .is_some_and(|(typed, rule_set)| {
                        OrdinaryMatchedRuleSignaturesAreComplete(typed, rule_set)
                            && rule_set
                                .AllRuleData()
                                .iter()
                                .map(|data| Rc::as_ptr(data.Rule()) as usize)
                                .collect::<HashSet<_>>()
                                .len()
                                == sheet.rules.len()
                    })
            });
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        // This compatibility counter counts dependency scans, not native
        // RuleSet compilation. The latter is owned by shared_rule_sets above.
        let mut rule_sets_built = 0;
        let mut dependency_trace =
            browser_tracing::span("style_rules", "StyleEngine.SelectorDependencies");
        let mut retained_hits = 0;
        let mut contents_hits = 0;
        let mut legacy_parses = 0;
        self.shared_selector_dependencies
            .retain(|_, (contents, _)| contents.strong_count() != 0);
        let retained = self
            .active_sheets
            .iter()
            .zip(&self.sheet_dependencies)
            .map(|((sheet, origin), dependencies)| {
                (
                    (Rc::as_ptr(sheet) as usize, *origin as u8),
                    dependencies.clone(),
                )
            })
            .collect::<HashMap<_, _>>();
        let sheet_dependencies: Vec<_> = active_sheets
            .iter()
            .map(|(sheet, origin)| {
                if !quirks_changed {
                    if let Some(dependencies) =
                        retained.get(&(Rc::as_ptr(sheet) as usize, *origin as u8))
                    {
                        retained_hits += 1;
                        return dependencies.clone();
                    }
                }
                // CSSOM edits without a canonical backing keep their existing
                // parser/diagnostic route. Standards-mode retained contents carry
                // the exact selectors which produced this projection.
                let parsed = (!quirks && sheet.HasCurrentSource())
                    .then(|| {
                        self.typed_sheets
                            .get(&(sheet.Identity(), sheet.Revision()))
                            .cloned()
                    })
                    .flatten();
                if let Some(parsed) = parsed {
                    let key = Rc::as_ptr(&parsed) as usize;
                    if let Some((_, dependencies)) = self.shared_selector_dependencies.get(&key) {
                        contents_hits += 1;
                        return dependencies.clone();
                    }
                    rule_sets_built += 1;
                    let dependencies = SelectorDependencies::ForParsedSheet(&parsed);
                    self.shared_selector_dependencies
                        .insert(key, (Rc::downgrade(&parsed), dependencies.clone()));
                    dependencies
                } else {
                    rule_sets_built += 1;
                    legacy_parses += 1;
                    SelectorDependencies::ForSheets(document, &[(sheet.clone(), *origin)])
                }
            })
            .collect();
        dependency_trace.set("retained_hits", retained_hits as f64);
        dependency_trace.set("contents_hits", contents_hits as f64);
        dependency_trace.set("dependency_sets_built", rule_sets_built as f64);
        dependency_trace.set("legacy_parses", legacy_parses as f64);
        drop(dependency_trace);
        let mut dependencies = SelectorDependencies::default();
        for sheet in &sheet_dependencies {
            dependencies.Merge(sheet);
        }
        let sheets: Vec<_> = active_sheets
            .iter()
            .map(|(s, o)| (s.as_ref(), *o))
            .collect();
        // cpp: style_engine.cc:290-316. This is the retained equivalent of one
        // StyleSheetCollection/ScopedStyleResolver per active TreeScope. UA
        // sheets are document-wide; author sheets remain in their owner scope.
        let scoped_sheets = document
            .ActiveTreeScopes()
            .into_iter()
            .map(|scope| {
                let owned = active_sheets
                    .iter()
                    .zip(active_sheet_scopes.iter())
                    .filter(|((_, origin), candidate)| {
                        *origin == CascadeOrigin::kUserAgent || **candidate == Some(scope)
                    })
                    .map(|((sheet, origin), _)| (sheet.as_ref(), *origin))
                    .collect::<Vec<_>>();
                (scope, owned)
            })
            .collect::<HashMap<_, _>>();
        drop(global_invalidation);
        let mut dirty = HashSet::new();
        let mut inline_dirty = HashSet::new();
        let mut candidates = HashSet::new();
        let mut refreshed_matches = HashMap::new();
        let mut candidate_checks = 0;
        let mut incomplete_candidates = 0;
        if all_dirty {
            AddSubtree(document, document.Root(), &mut dirty);
        } else {
            if viewport_only_environment_change {
                // LocalFrameView::ViewportSizeChanged marks only styles which
                // consumed viewport units. Media queries whose truth value did
                // not change keep their RuleSet and unrelated ComputedStyles.
                // Layout still receives the new containing-block constraint.
                self.viewport_dependent_elements
                    .retain(|&index| ConnectedElement(document, index));
                dirty.extend(self.viewport_dependent_elements.iter().copied());
            }
            let state = document.StyleState();
            let dom_invalidation =
                browser_tracing::span("style_invalidation", "StyleEngine.InvalidateDOMChanges");
            for change in &state.changes {
                let index = match *change {
                    StyleChange::PlaceholderShown(index)
                    | StyleChange::EmptyState(index)
                    | StyleChange::Directionality(index)
                    | StyleChange::Defined(index)
                    | StyleChange::Node(index)
                    | StyleChange::AttributeValue(index)
                    | StyleChange::ControlPseudo(index, _)
                    | StyleChange::ExportParts(index)
                    | StyleChange::InlineStyle(index)
                    | StyleChange::Animation(index)
                    | StyleChange::Resource(index)
                    | StyleChange::Children(index) => index,
                };
                // cpp: StyleEngine::ShouldSkipInvalidationFor. Removed nodes
                // cannot affect the active tree; their former parent's child
                // change remains independently recorded. Reinsertion supplies
                // its own inserted subtree invalidation.
                if !document.IsConnectedToDocument(index) {
                    continue;
                }
                let candidates_before = candidates.len();
                let dirty_before = dirty.len();
                if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                    let index = match *change {
                        StyleChange::PlaceholderShown(index)
                        | StyleChange::EmptyState(index)
                        | StyleChange::Directionality(index)
                        | StyleChange::Defined(index)
                        | StyleChange::Node(index)
                        | StyleChange::AttributeValue(index)
                        | StyleChange::ControlPseudo(index, _)
                        | StyleChange::ExportParts(index)
                        | StyleChange::InlineStyle(index)
                        | StyleChange::Animation(index)
                        | StyleChange::Resource(index)
                        | StyleChange::Children(index) => index,
                    };
                    let node = document.Node(index);
                    eprintln!("style invalidation {change:?} node_id={} tag={} id={} class={} connected={}",
                        node.Id(), node.Name(), node.FindAttribute("id").map_or("", |a| &a.value),
                        node.FindAttribute("class").map_or("", |a| &a.value), document.IsConnectedToDocument(index));
                }
                match *change {
                    StyleChange::Directionality(index) => {
                        dirty.insert(index);
                        self.interaction_changed_nodes
                            .insert(document.Node(index).Id());
                        self.interaction_changed_pseudos
                            .insert(PseudoType::kPseudoDir);
                        self.interaction_invalidation_roots
                            .entry(PseudoType::kPseudoDir)
                            .or_default()
                            .insert(index);
                    }
                    StyleChange::Defined(index) => {
                        dirty.insert(index);
                        self.interaction_changed_nodes
                            .insert(document.Node(index).Id());
                        self.interaction_changed_pseudos
                            .insert(PseudoType::kPseudoDefined);
                        self.interaction_invalidation_roots
                            .entry(PseudoType::kPseudoDefined)
                            .or_default()
                            .insert(index);
                    }
                    StyleChange::ControlPseudo(index, state) => {
                        use dom::style_state::ControlPseudoState;
                        let pseudo = match state {
                            ControlPseudoState::Default => PseudoType::kPseudoDefault,
                            ControlPseudoState::HasDatalist => PseudoType::kPseudoHasDatalist,
                            ControlPseudoState::Indeterminate => PseudoType::kPseudoIndeterminate,
                        };
                        dirty.insert(index);
                        self.interaction_changed_nodes
                            .insert(document.Node(index).Id());
                        self.interaction_changed_pseudos.insert(pseudo);
                        self.interaction_invalidation_roots
                            .entry(pseudo)
                            .or_default()
                            .insert(index);
                    }
                    StyleChange::PlaceholderShown(index) => {
                        dirty.insert(index);
                        self.interaction_changed_nodes
                            .insert(document.Node(index).Id());
                        self.interaction_changed_pseudos
                            .insert(PseudoType::kPseudoPlaceholderShown);
                        self.interaction_invalidation_roots
                            .entry(PseudoType::kPseudoPlaceholderShown)
                            .or_default()
                            .insert(index);
                    }
                    StyleChange::EmptyState(parent) => {
                        use crate::selector_checker::ElementInvalidationFlag as E;
                        let empty = self
                            .selectors
                            .borrow()
                            .ElementHasInvalidationFlag(parent, E::StyleAffectedByEmpty);
                        if empty {
                            candidates.insert(parent);
                            // cpp: Element::PseudoStateChanged(kPseudoEmpty).
                            // Character data never changes element sibling
                            // indices, so nth and adjacent-child invalidation do
                            // not participate in this mutation.
                            if dependencies.sibling {
                                if let Some(grandparent) = document.Node(parent).Parent() {
                                    let siblings = document.Node(grandparent).Children();
                                    if let Some(position) =
                                        siblings.iter().position(|&index| index == parent)
                                    {
                                        for &sibling in &siblings[position + 1..] {
                                            AddSubtree(document, sibling, &mut candidates);
                                        }
                                    }
                                }
                            }
                            if dependencies.has {
                                self.InvalidateHasAncestors(document, parent, &mut candidates);
                            }
                        }
                    }
                    StyleChange::AttributeValue(index) => {
                        dirty.insert(index);
                        InvalidateSlotAssignmentForNode(document, index, &mut dirty);
                        // HTML href changes also change link pseudo states,
                        // independently of CSS [href] attribute dependencies.
                        if document
                            .StyleState()
                            .attribute_changes
                            .get(&index)
                            .is_some_and(|change| change.names.contains("href"))
                            && matches!(document.Node(index).Name(), "a" | "area" | "link")
                        {
                            self.interaction_changed_nodes
                                .insert(document.Node(index).Id());
                            for pseudo in [
                                PseudoType::kPseudoAnyLink,
                                PseudoType::kPseudoWebkitAnyLink,
                                PseudoType::kPseudoLink,
                                PseudoType::kPseudoVisited,
                            ] {
                                self.interaction_changed_pseudos.insert(pseudo);
                                self.interaction_invalidation_roots
                                    .entry(pseudo)
                                    .or_default()
                                    .insert(index);
                            }
                        }
                        if !self.InvalidateAttributeRelations(
                            document,
                            index,
                            attribute_invalidation_rules.as_deref(),
                            &mut candidates,
                        ) {
                            self.InvalidateRelations(
                                document,
                                index,
                                &dependencies,
                                &mut candidates,
                            );
                        } else if dependencies.has {
                            self.InvalidateHasForAttribute(document, index, &mut candidates);
                        }
                    }
                    StyleChange::Node(index) => {
                        dirty.insert(index);
                        InvalidateSlotAssignmentForNode(document, index, &mut dirty);
                        self.InvalidateRelations(document, index, &dependencies, &mut candidates);
                    }
                    StyleChange::ExportParts(index) => {
                        dirty.insert(index);
                        self.InvalidateRelations(document, index, &dependencies, &mut candidates);
                        // cpp: style_engine.cc:2041-2056. Blink schedules the
                        // global part invalidation set on the host. In this
                        // retained owner, that is the attached shadow subtree
                        // whose elements can be forwarded through this map.
                        if dependencies.part {
                            if let Some(shadow_root) = document.ShadowRoot(index) {
                                AddSubtree(document, shadow_root, &mut dirty);
                            }
                        }
                    }
                    StyleChange::InlineStyle(index) => {
                        if state.independent_inline_styles.contains(&index) {
                            inline_dirty.insert(index);
                        } else {
                            dirty.insert(index);
                        }
                        if dependencies.inline_selector {
                            // Attribute selectors can upgrade this subject to a
                            // full recalc; compare their retained signature too.
                            candidates.insert(index);
                            if !self.InvalidateAttributeRelations(
                                document,
                                index,
                                attribute_invalidation_rules.as_deref(),
                                &mut candidates,
                            ) {
                                self.InvalidateRelations(
                                    document,
                                    index,
                                    &dependencies,
                                    &mut candidates,
                                );
                            }
                        }
                    }
                    StyleChange::Animation(index) => {
                        dirty.insert(index);
                    }
                    // Chromium's StyleFetchedImage observes its mutable
                    // ImageResourceContent in place. Rechrom publishes an
                    // immutable fetched-image snapshot across this package
                    // boundary, so replace only the affected paint client when
                    // the resource owner publishes its id, geometry or error.
                    StyleChange::Resource(index) => {
                        dirty.insert(index);
                    }
                    StyleChange::Children(parent) => {
                        use crate::selector_checker::{
                            ContainerInvalidationFlag as C, ElementInvalidationFlag as E,
                        };
                        let selectors = self.selectors.borrow();
                        let affected = |flag| selectors.ContainerHasInvalidationFlag(parent, flag);
                        let forward = affected(C::ChildrenAffectedByForwardPositionalRules);
                        let backward = affected(C::ChildrenAffectedByBackwardPositionalRules);
                        let first = affected(C::ChildrenAffectedByFirstChildRules);
                        let last = affected(C::ChildrenAffectedByLastChildRules);
                        let adjacent = affected(C::ChildrenAffectedByDirectAdjacentRules)
                            || affected(C::ChildrenAffectedByIndirectAdjacentRules);
                        let empty =
                            selectors.ElementHasInvalidationFlag(parent, E::StyleAffectedByEmpty);
                        drop(selectors);
                        candidates.insert(parent);
                        InvalidateSlotAssignmentForChildren(document, parent, &mut dirty);
                        // Child-state subjects such as :empty can be on the
                        // left side of a sibling combinator.
                        if dependencies.sibling && empty {
                            if let Some(grandparent) = document.Node(parent).Parent() {
                                let siblings = document.Node(grandparent).Children();
                                if let Some(position) = siblings.iter().position(|&i| i == parent) {
                                    for &sibling in &siblings[position + 1..] {
                                        AddSubtree(document, sibling, &mut candidates);
                                    }
                                }
                            }
                        }
                        // cpp: ContainerNode::CheckForSiblingStyleChanges.
                        // End insertion/removal has no succeeding sibling, so
                        // forward positional and adjacent rules cannot change
                        // existing siblings. Preserve that boundary fact.
                        if let Some(Some(boundaries)) = state.child_change_boundaries.get(&parent) {
                            for boundary in boundaries {
                                let siblings = document.Node(parent).Children();
                                // cpp: StyleEngine::ScheduleNthPseudoInvalidations.
                                // Visit only the sibling range whose index or
                                // reverse index changed. A positional ancestor
                                // may affect descendants, so each range root is
                                // traversed without widening to the parent.
                                if forward {
                                    if let Some(after) = boundary.after {
                                        if let Some(position) =
                                            siblings.iter().position(|&index| index == after)
                                        {
                                            for &sibling in &siblings[position..] {
                                                AddSubtree(document, sibling, &mut candidates);
                                            }
                                        }
                                    }
                                }
                                if backward {
                                    if let Some(before) = boundary.before {
                                        if let Some(position) =
                                            siblings.iter().position(|&index| index == before)
                                        {
                                            for &sibling in &siblings[..=position] {
                                                AddSubtree(document, sibling, &mut candidates);
                                            }
                                        }
                                    }
                                }
                                if first && boundary.before.is_none() {
                                    if let Some(after) = boundary.after {
                                        AddSubtree(document, after, &mut candidates);
                                    }
                                }
                                if last && boundary.after.is_none() {
                                    if let Some(before) = boundary.before {
                                        AddSubtree(document, before, &mut candidates);
                                    }
                                }
                                if adjacent {
                                    if let Some(after) = boundary.after {
                                        if let Some(position) =
                                            siblings.iter().position(|&i| i == after)
                                        {
                                            for &sibling in &siblings[position..] {
                                                AddSubtree(document, sibling, &mut candidates);
                                            }
                                        }
                                    }
                                }
                            }
                        } else if forward || backward || first || last || adjacent {
                            AddSubtree(document, parent, &mut candidates);
                        }
                        if browser_tracing::enabled() {
                            browser_tracing::instant(
                                "style_invalidation",
                                "StyleChildrenInvalidation",
                                &[
                                    ("parent_id", document.Node(parent).Id() as f64),
                                    ("forward", forward as u8 as f64),
                                    ("backward", backward as u8 as f64),
                                    ("first", first as u8 as f64),
                                    ("last", last as u8 as f64),
                                    ("adjacent", adjacent as u8 as f64),
                                    ("empty", empty as u8 as f64),
                                    ("candidates", candidates.len() as f64),
                                ],
                            );
                        }
                        if dependencies.has {
                            self.InvalidateHasAncestors(document, parent, &mut candidates);
                        }
                    }
                }
                if browser_tracing::enabled()
                    && (candidates.len() != candidates_before || dirty.len() != dirty_before)
                {
                    let index = match *change {
                        StyleChange::PlaceholderShown(index)
                        | StyleChange::EmptyState(index)
                        | StyleChange::Directionality(index)
                        | StyleChange::Defined(index)
                        | StyleChange::Node(index)
                        | StyleChange::AttributeValue(index)
                        | StyleChange::ControlPseudo(index, _)
                        | StyleChange::ExportParts(index)
                        | StyleChange::InlineStyle(index)
                        | StyleChange::Animation(index)
                        | StyleChange::Resource(index)
                        | StyleChange::Children(index) => index,
                    };
                    let node = document.Node(index);
                    let attributes = state.attribute_changes.get(&index);
                    browser_tracing::instant_with_text(
                        "style_invalidation",
                        "StyleChangeInvalidation",
                        &format!(
                            "{change:?} tag={} id={} attrs={:?}",
                            node.Name(),
                            node.FindAttribute("id").map_or("", |a| &a.value),
                            attributes.map(|a| &a.names)
                        ),
                        &[
                            ("node_id", node.Id() as f64),
                            ("candidates_before", candidates_before as f64),
                            ("candidates_after", candidates.len() as f64),
                            (
                                "candidates_added",
                                candidates.len().saturating_sub(candidates_before) as f64,
                            ),
                            (
                                "dirty_added",
                                dirty.len().saturating_sub(dirty_before) as f64,
                            ),
                        ],
                    );
                }
            }
            for &index in &state.inserted_style_subtrees {
                AddSubtree(document, index, &mut dirty);
            }
            drop(dom_invalidation);
            // Rule-set invalidation checks both previously matched rules and
            // current matches. The former survives simultaneous DOM mutations.
            if rule_sets_changed && document_sheets_changed {
                let mut changed_sheets_trace = browser_tracing::span(
                    "style_invalidation",
                    "StyleEngine.InvalidateChangedSheets",
                );
                let precise_changes = old_rule_sets.as_ref().and_then(|old_rules| {
                    ChangedRuleSetSheetIdentities(
                        &self.active_sheets,
                        &active_sheets,
                        old_rules,
                        &self.translated_rule_sets,
                    )
                });
                changed_sheets_trace.set("old_rule_sets", old_rule_sets.is_some() as u8 as f64);
                changed_sheets_trace.set("precise_changes", precise_changes.is_some() as u8 as f64);
                changed_sheets_trace.set(
                    "changed_rule_sets",
                    precise_changes.as_ref().map_or(0, HashSet::len) as f64,
                );
                changed_sheets_trace.set(
                    "signature_context",
                    signature_non_rule_context_unchanged as u8 as f64,
                );
                changed_sheets_trace.set(
                    "rule_set_context",
                    rule_set_invalidation_context_unchanged as u8 as f64,
                );
                changed_sheets_trace.set("ua_unchanged", ua_unchanged as u8 as f64);
                let mut old_sheets_changed = false;
                let mut old_highlight_pseudos = HashSet::new();
                let native_changed_rules = precise_changes.as_ref().and_then(|changed| {
                    if !rule_set_invalidation_context_unchanged || !ua_unchanged {
                        return None;
                    }
                    let mut rules = Vec::new();
                    let mut media_diff_sheets = HashSet::new();
                    for (position, (sheet, _)) in self.active_sheets.iter().enumerate() {
                        if changed.contains(&(sheet.Identity(), sheet.Revision())) {
                            let scope = self
                                .active_sheet_scopes
                                .get(position)
                                .copied()
                                .flatten()
                                .unwrap_or_else(|| document.Root());
                            let typed = sheet
                                .ParsedBacking::<crate::production_style_sheet::ProductionStyleSheet>()
                                .or_else(|| {
                                    self.typed_sheets
                                        .get(&(sheet.Identity(), sheet.Revision()))
                                        .cloned()
                                })?;
                            if !typed.functions.is_empty() || !typed.mixins.is_empty() { return None; }
                            let old = old_rule_sets.as_ref()?.get(position)?.as_ref()?;
                            old_sheets_changed = true;
                            // Highlights contribute originating-style metadata,
                            // not ordinary transferred declarations/signatures.
                            // Preserve that positive old-match evidence too.
                            for data in old.AllRuleData() {
                                for selector in data.Selector().SimpleSelectors() {
                                    let pseudo = GetPseudoId(selector.GetPseudoType());
                                    if layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(pseudo) {
                                        old_highlight_pseudos.insert(pseudo);
                                    }
                                }
                            }
                            let key = (sheet.Identity(), sheet.Revision());
                            if media_rule_sets_changed {
                                if let Some(current) = self.translated_rule_sets.get(&key) {
                                    // cpp: RuleSetDiff::CreateDiffRuleset. A
                                    // media change invalidates selectors that
                                    // entered or left the active RuleSet, not
                                    // every selector in the owning sheet.
                                    let difference = ProductionRuleSetDifference(old, current);
                                    // The media attribute on <style>/<link> is
                                    // owner metadata, outside StyleSheetContents.
                                    // Its truth-value can change while the
                                    // retained RuleSet remains byte-for-byte
                                    // identical. Chromium toggles that complete
                                    // ActiveStyleSheet entry; use the owning
                                    // RuleSet when the inner diff is empty.
                                    rules.push((
                                        scope,
                                        if difference.AllRuleData().is_empty() {
                                            old.clone()
                                        } else {
                                            difference
                                        },
                                    ));
                                    media_diff_sheets.insert(key);
                                    continue;
                                }
                            }
                            rules.push((scope, old.clone()));
                        }
                    }
                    for (position, (sheet, _)) in active_sheets.iter().enumerate() {
                        let key = (sheet.Identity(), sheet.Revision());
                        if changed.contains(&key) && !media_diff_sheets.contains(&key) {
                            let scope = active_sheet_scopes
                                .get(position)
                                .copied()
                                .flatten()
                                .unwrap_or_else(|| document.Root());
                            let typed = self.typed_sheets.get(&key)?;
                            if !typed.functions.is_empty() || !typed.mixins.is_empty() { return None; }
                            let ruleset = self.translated_rule_sets.get(&key)?;
                            if !rules.iter().any(|(old_scope, old)| {
                                *old_scope == scope && Rc::ptr_eq(old, ruleset)
                            }) {
                                rules.push((scope, ruleset.clone()));
                            }
                        }
                    }
                    Some(rules)
                });
                changed_sheets_trace.set(
                    "native_changed_rules",
                    native_changed_rules.is_some() as u8 as f64,
                );
                changed_sheets_trace.set(
                    "native_rule_sets",
                    native_changed_rules.as_ref().map_or(0, Vec::len) as f64,
                );
                let dirty_before_changed_sheets = dirty.len();
                let mut native_invalidated_count = 0usize;
                let mut signature_invalidated_count = 0usize;
                let mut missing_old_match_count = 0usize;
                // Blink runs ApplyRuleSetInvalidationForTreeScope in element
                // traversal order and keeps SelectorFilter's ancestor stack in
                // step with that walk. Preserve the same order here; iterating
                // a HashSet rebuilds the ancestor filter for almost every node.
                let elements = ConnectedElements(document).collect::<Vec<_>>();
                let native_invalidated = native_changed_rules.as_ref().and_then(|rules| {
                    // ApplyRuleSetChanges is scoped. A shadow sheet containing
                    // :host, ::slotted or another tree-scope selector cannot be
                    // checked by the document-scope invalidator, but it also
                    // cannot justify invalidating the entire document. Keep the
                    // precise production matcher for document rules and confine
                    // the compatibility fallback to each changed shadow scope.
                    let document_root = document.Root();
                    let document_rules = rules
                        .iter()
                        .filter(|(scope, _)| *scope == document_root)
                        .map(|(_, rules)| rules.clone())
                        .collect::<Vec<_>>();
                    let shadow_scopes = rules
                        .iter()
                        .filter_map(|(scope, _)| (*scope != document_root).then_some(*scope))
                        .collect::<HashSet<_>>();
                    changed_sheets_trace.set("shadow_fallback_scopes", shadow_scopes.len() as f64);
                    let mut invalidated = if document_rules.is_empty() {
                        HashSet::new()
                    } else {
                        crate::production_rule_matching::InvalidateProductionRuleSets(
                            &document_rules,
                            document,
                            &mut self.selectors.borrow_mut(),
                            &elements
                                .iter()
                                .copied()
                                .filter(|index| !dirty.contains(index))
                                .collect::<Vec<_>>(),
                        )?
                    };
                    InvalidateChangedTreeScopes(document, &shadow_scopes, &mut invalidated);
                    Some(invalidated)
                });
                if let Some(invalidated) = native_invalidated {
                    native_invalidated_count = invalidated.len();
                    dirty.extend(invalidated);
                    // Numeric positions are only a projection of sheet-owned
                    // rules. Reindex unchanged sheets by retained identity and
                    // revision; never compare their old/new numeric positions.
                    // Preserve positive old matches for a later removal batched
                    // with a DOM mutation, even if this update changed no style.
                    let mut new_offsets = HashMap::new();
                    let mut offset = 1;
                    for (sheet, _) in active_sheets.iter() {
                        new_offsets
                            .entry((sheet.Identity(), sheet.Revision()))
                            .or_insert_with(std::collections::VecDeque::new)
                            .push_back(offset);
                        offset += sheet.rules.len();
                    }
                    let mut positions = vec![None];
                    for (sheet, _) in self.active_sheets.iter() {
                        let start = new_offsets
                            .get_mut(&(sheet.Identity(), sheet.Revision()))
                            .and_then(|offsets| offsets.pop_front());
                        // A media-query result rebuilds the active RuleSet, but
                        // it does not reorder the CSSStyleSheet's source rules.
                        // The diff RuleSet above already invalidated elements
                        // matching rules which entered or left the active set.
                        // Keep every other retained signature at the same
                        // source position. Treating the whole owning sheet as
                        // reindexed turns a single breakpoint rule into a full
                        // document recalc on large pages.
                        let changed = !media_rule_sets_changed
                            && precise_changes
                                .as_ref()
                                .unwrap()
                                .contains(&(sheet.Identity(), sheet.Revision()));
                        positions.extend((0..sheet.rules.len()).map(|local| {
                            (!changed)
                                .then_some(start)
                                .flatten()
                                .map(|start| start + local)
                        }));
                    }
                    self.matches.retain(|index, signature| {
                        if signature
                            .values
                            .iter()
                            .any(|(order, _, _)| positions.get(*order).copied().flatten().is_none())
                        {
                            signature_invalidated_count += usize::from(dirty.insert(*index));
                            return false;
                        }
                        for (order, _, _) in &mut signature.values {
                            *order = positions[*order].unwrap();
                        }
                        signature.values[..signature.base_len].sort_unstable();
                        signature.values[signature.base_len..].sort_unstable();
                        true
                    });
                    if old_sheets_changed {
                        for &index in &elements {
                            if dirty.contains(&index) {
                                continue;
                            }
                            // The DOM adapter batches mutation facts. Losing a
                            // current selector match cannot revoke a committed
                            // old match, as mutation-time pending invalidation
                            // already protects it in Blink. Missing provenance
                            // must conservatively recalc; stable node IDs alone
                            // never prove that an old rule did not apply.
                            // An element with no committed ComputedStyle has
                            // no old rule effects to retain. Chromium still
                            // tests the changed RuleSets on that element, but
                            // does not manufacture a recalc for missing style.
                            let old_style = document.ResolvedStyleFor(index);
                            // A pure viewport media change has no selector-state
                            // mutation to preserve. The diff RuleSet above
                            // tested both the entering and leaving rules against
                            // every connected element, matching Blink's
                            // ApplyRuleSetChanges path. A missing Rechrom
                            // equality sidecar is therefore not evidence for a
                            // recalc. Keep the conservative old-match fallback
                            // only when DOM changes were batched with the rule
                            // change and the current tree cannot prove the old
                            // selector state by itself.
                            let missing_old_match = (!media_rule_sets_changed
                                || !document.StyleState().changes.is_empty())
                                && old_style.is_some()
                                && !self.matches.contains_key(&index);
                            let old_highlight = old_style
                                .and_then(|style| unsafe { style.native_style.Get().as_ref() })
                                .is_some_and(|style| {
                                    old_highlight_pseudos
                                        .iter()
                                        .any(|pseudo| style.HasPseudoElementStyle(*pseudo))
                                });
                            if missing_old_match || old_highlight {
                                missing_old_match_count += usize::from(dirty.insert(index));
                            }
                        }
                    }
                } else {
                    let mut fallback_trace = browser_tracing::span(
                        "style_invalidation",
                        "StyleEngine.InvalidateFallbackRules",
                    );
                    fallback_trace.set("elements", elements.len() as f64);
                    // Production RuleSet invalidation is the only retained
                    // selector-feature proof. If it cannot represent this
                    // sheet change, schedule one conservative scope recalc.
                    // A second complete old/new selector match before that
                    // recalc has no Chromium counterpart and doubles the hot
                    // path on late stylesheet arrival.
                    fallback_trace.set("full_scope", 1.0);
                    AddSubtree(document, document.Root(), &mut dirty);
                    self.matches.clear();
                }
                changed_sheets_trace.set(
                    "dirty_before_changed_sheets",
                    dirty_before_changed_sheets as f64,
                );
                changed_sheets_trace.set("native_invalidated", native_invalidated_count as f64);
                changed_sheets_trace
                    .set("signature_invalidated", signature_invalidated_count as f64);
                changed_sheets_trace.set(
                    "missing_old_match_invalidated",
                    missing_old_match_count as f64,
                );
                changed_sheets_trace.set("dirty_after_changed_sheets", dirty.len() as f64);
                browser_tracing::instant(
                    "style_invalidation",
                    "StyleChangedSheetInvalidationOutcome",
                    &[
                        ("dirty_before", dirty_before_changed_sheets as f64),
                        ("native_invalidated", native_invalidated_count as f64),
                        ("signature_invalidated", signature_invalidated_count as f64),
                        (
                            "missing_old_match_invalidated",
                            missing_old_match_count as f64,
                        ),
                        ("dirty_after", dirty.len() as f64),
                    ],
                );
                // Rule order in the retained match signatures changed. Refresh
                // signatures for candidates instead of comparing unlike revisions.
            }
            // cpp: style_engine.cc:3225-3231. A ShadowRoot sheet collection
            // update invalidates its scoped subtree plus the exact elements
            // reached through :host and ::slotted. Outer-scope ::part rules
            // can reach into descendant shadow scopes, so a changed ancestor
            // scope also visits those scoped subtrees.
            InvalidateChangedTreeScopes(document, &changed_tree_scopes, &mut dirty);
            if !self.interaction_changed_nodes.is_empty() {
                // RuleSet's compiled features select relational subjects.
                // Local user-action state was compared through the production
                // selector backend above; unrelated elements keep their style
                // even when a sheet cannot certify a complete match signature.
                let mut buckets = Vec::new();
                let mut whole_scope = false;
                let mut pending =
                    crate::invalidation::pending_invalidations::PendingInvalidations::default();
                let mut invalidation_dom =
                    crate::persistent_invalidation::PersistentInvalidationDom::new(document);
                for (sheet, _) in active_sheets.iter() {
                    let Some(rules) = self
                        .translated_rule_sets
                        .get(&(sheet.Identity(), sheet.Revision()))
                    else {
                        whole_scope = true;
                        continue;
                    };
                    for pseudo in &self.interaction_changed_pseudos {
                        let Some(set) = rules.Features().pseudo_state_invalidation.get(pseudo)
                        else {
                            continue;
                        };
                        let unscoped_descendants =
                            !rules.Features().scope_pseudo_states.contains(pseudo)
                                && !rules
                                    .Features()
                                    .non_descendant_pseudo_states
                                    .contains(pseudo);
                        if unscoped_descendants {
                            assert!(
                                set.borrow().IsDescendantInvalidationSet()
                                    && !set.borrow().InvalidatesNth()
                            );
                            // cpp: StyleEngine::PseudoStateChangedForElement ->
                            // PendingInvalidations::ScheduleInvalidationSetsForNode.
                            // WholeSubtreeInvalid belongs to this changed element,
                            // never to the complete document.
                            if let Some(roots) = self.interaction_invalidation_roots.get(pseudo) {
                                for &root in roots {
                                    pending.ScheduleInvalidationSetsForNode(
                                        &crate::invalidation::invalidation_set::InvalidationLists {
                                            descendants: vec![set.clone()],
                                            siblings: Vec::new(),
                                        },
                                        &root,
                                        &mut invalidation_dom,
                                    );
                                }
                            }
                            continue;
                        }
                        let set = set.borrow();
                        whole_scope |= set.WholeSubtreeInvalid();
                        buckets.extend(set.Ids().into_iter().map(|s| SelectorBucket::Id(s.Utf8())));
                        buckets.extend(
                            set.Classes()
                                .into_iter()
                                .map(|s| SelectorBucket::Class(s.Utf8())),
                        );
                        buckets.extend(
                            set.Attributes()
                                .into_iter()
                                .map(|s| SelectorBucket::Attribute(s.Utf8())),
                        );
                        buckets.extend(
                            set.TagNames()
                                .into_iter()
                                .map(|s| SelectorBucket::Tag(s.Utf8())),
                        );
                        buckets.extend(
                            set.CustomPseudoNames()
                                .into_iter()
                                .map(|s| SelectorBucket::CustomPseudo(s.Utf8())),
                        );
                    }
                }
                {
                    use crate::invalidation::invalidation_dom::InvalidationDom;
                    let root = document.Root();
                    let element = invalidation_dom.document_element(&root);
                    crate::invalidation::style_invalidator::StyleInvalidator::new(
                        pending.GetPendingInvalidationMap(),
                    )
                    .Invalidate(&root, element, &mut invalidation_dom);
                    for (index, change) in invalidation_dom.changes {
                        if change == crate::invalidation::invalidation_dom::StyleChangeType::SubtreeStyleChange {
                            AddSubtree(document, index, &mut candidates);
                        } else {
                            candidates.insert(index);
                        }
                    }
                }
                browser_tracing::instant(
                    "style_invalidation",
                    "StyleInteractionInvalidation",
                    &[
                        ("whole_scope", whole_scope as u8 as f64),
                        ("buckets", buckets.len() as f64),
                        ("candidates", candidates.len() as f64),
                        (
                            "pseudo",
                            self.interaction_changed_pseudos
                                .iter()
                                .next()
                                .map_or(0.0, |p| *p as u16 as f64),
                        ),
                    ],
                );
                let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
                for index in ConnectedElements(document) {
                    if whole_scope
                        || buckets
                            .iter()
                            .any(|bucket| bucket.Matches(document, index, quirks))
                    {
                        candidates.insert(index);
                    }
                    // Element's own native/control state must be refreshed
                    // even when its ordinary matched-rule signature is equal.
                    if self
                        .interaction_changed_nodes
                        .contains(&document.Node(index).Id())
                    {
                        dirty.insert(index);
                    }
                }
            }
            let _trace =
                browser_tracing::span("style_invalidation", "StyleEngine.InvalidateCandidates");
            for index in candidates {
                if dirty.contains(&index) || !ConnectedElement(document, index) {
                    continue;
                }
                candidate_checks += 1;
                incomplete_candidates += usize::from(
                    self.matches
                        .get(&index)
                        .is_some_and(|signature| !signature.complete),
                );
                let element_scope = document
                    .TreeScopeRoot(document.CorrespondingSVGElement(index).unwrap_or(index));
                let element_sheets = scoped_sheets
                    .get(&element_scope)
                    .map_or(&[][..], Vec::as_slice);
                if self.CandidateRequiresRecalc(
                    document,
                    index,
                    media,
                    element_sheets,
                    environment_variables,
                    signature_context_unchanged,
                    signature_non_rule_context_unchanged,
                    ordinary_signature_complete,
                ) {
                    dirty.insert(index);
                }
            }
        }
        // cpp: StyleRecalcChange::RecalcContainerQueryDependent and
        // ShouldUpdatePseudoElement. Keep container-dependent subjects in the
        // retained traversal when DOM/style changes can affect container
        // selection or computed query values. Other elements are not dirtied
        // merely because a sheet contains @container. A size/state publication
        // starts at its owning container and reaches flat-tree descendants.
        let container_invalidation = browser_tracing::span(
            "style_invalidation",
            "StyleEngine.InvalidateContainerDependents",
        );
        let mut container_dependents = 0;
        let container_selection_changed = !document.StyleState().changes.is_empty();
        self.container_dependent_elements
            .retain(|&index| ConnectedElement(document, index));
        if !all_dirty
            && (container_selection_changed || !self.invalidated_container_nodes.is_empty())
        {
            for &index in &self.container_dependent_elements {
                if !ConnectedElement(document, index) {
                    continue;
                }
                let in_changed_container = container_selection_changed || {
                    let mut ancestor = Some(index);
                    let mut affected = false;
                    while let Some(node) = ancestor {
                        if self.invalidated_container_nodes.contains(&node) {
                            affected = true;
                            break;
                        }
                        ancestor = FlatTreeStyleParentElement(document, node);
                    }
                    affected
                };
                if in_changed_container {
                    dirty.insert(index);
                    container_dependents += 1;
                }
            }
        }
        // cpp: Node::SetNeedsStyleRecalc -> SVGElement::SetNeedsStyleRecalcForInstances.
        // Source dirtiness reaches its corresponding instances. Unrelated
        // lifecycle work must not dirty every <use> shadow tree. Host inheritance
        // propagates through the normal flat-tree traversal, and changed sheets
        // already invalidate their exposed TreeScopes above.
        let mut svg_instance_dependents = 0;
        if !all_dirty {
            for index in document.SVGInstanceElements() {
                if ConnectedElement(document, index)
                    && document
                        .CorrespondingSVGElement(index)
                        .is_some_and(|source| dirty.contains(&source))
                    && dirty.insert(index)
                {
                    svg_instance_dependents += 1;
                }
            }
        }
        // Keep independently modified declarations separate until every other
        // invalidation source has upgraded its subjects to ordinary recalc.
        let incremental_style_candidates = inline_dirty
            .difference(&dirty)
            .copied()
            .collect::<HashSet<_>>();
        dirty.extend(inline_dirty);
        dirty.retain(|&index| ConnectedElement(document, index));
        drop(container_invalidation);
        if browser_tracing::enabled() {
            let state = document.StyleState();
            browser_tracing::instant(
                "style_invalidation",
                "StyleRecalcInvalidation",
                &[
                    ("all_dirty", all_dirty as u8 as f64),
                    ("font_features_changed", font_features_changed as u8 as f64),
                    ("registry_changed", registry_changed as u8 as f64),
                    ("environment_changed", environment_changed as u8 as f64),
                    (
                        "conservative_scope_invalidation",
                        conservative_scope_invalidation as u8 as f64,
                    ),
                    (
                        "state_all_dirty",
                        document.StyleState().all_dirty as u8 as f64,
                    ),
                    ("container_dependents", container_dependents as f64),
                    ("svg_instance_dependents", svg_instance_dependents as f64),
                    ("candidate_checks", candidate_checks as f64),
                    ("incomplete_candidates", incomplete_candidates as f64),
                    (
                        "ordinary_signature_complete",
                        ordinary_signature_complete as u8 as f64,
                    ),
                    (
                        "signature_context_unchanged",
                        signature_context_unchanged as u8 as f64,
                    ),
                    ("dirty_elements", dirty.len() as f64),
                    (
                        "interaction_nodes",
                        self.interaction_changed_nodes.len() as f64,
                    ),
                    (
                        "interaction_pseudos",
                        self.interaction_changed_pseudos.len() as f64,
                    ),
                    ("changes", state.changes.len() as f64),
                    (
                        "animation_changes",
                        state
                            .changes
                            .iter()
                            .filter(|c| matches!(c, StyleChange::Animation(_)))
                            .count() as f64,
                    ),
                    (
                        "node_changes",
                        state
                            .changes
                            .iter()
                            .filter(|c| {
                                matches!(c, StyleChange::Node(_) | StyleChange::AttributeValue(_))
                            })
                            .count() as f64,
                    ),
                    ("sheet_changes", sheet_changes.len() as f64),
                ],
            );
        }
        let mut ensure_path = HashSet::new();
        if let Some(target) = ensure_target {
            let mut ancestor = Some(target);
            while let Some(index) = ancestor {
                if document.ResolvedStyleFor(index).is_some() {
                    break;
                }
                ensure_path.insert(index);
                ancestor = FlatTreeStyleParentElement(document, index);
            }
            dirty.clear();
            dirty.extend(ensure_path.iter().copied());
        }
        // ChildNeedsStyleRecalc equivalent: enter only paths to dirty elements;
        // inherited differences can extend traversal during style resolution.
        let mut visit = HashSet::new();
        for &index in &dirty {
            let mut ancestor = Some(index);
            while let Some(index) = ancestor {
                if !visit.insert(index) {
                    break;
                }
                ancestor = FlatTreeStyleParentElement(document, index);
            }
            // Inactive fallback and unassigned light-DOM roots intentionally
            // have no flat-tree parent, but the single retained traversal must
            // still enter their DOM/shadow carrier path to resolve them from
            // initial style. Assigned nodes are reached through their slot.
            let mut carrier = Some(index);
            while let Some(index) = carrier {
                visit.insert(index);
                carrier = document
                    .AssignedSlot(index)
                    .or_else(|| document.ParentOrShadowHostElement(index));
            }
        }
        browser_tracing::instant(
            "style_invalidation",
            "StyleInvalidationOutcome",
            &[
                ("dirty_elements", dirty.len() as f64),
                ("visit_elements", visit.len() as f64),
            ],
        );
        if visit.is_empty() {
            self.invalidated_container_nodes.clear();
            self.interaction_changed_nodes.clear();
            self.interaction_changed_pseudos.clear();
            self.interaction_invalidation_roots.clear();
            self.sheet_revision = Some(revision);
            self.active_sheets = active_sheets;
            self.active_sheet_scopes = active_sheet_scopes;
            self.environment = Some(environment);
            self.sheet_dependencies = sheet_dependencies;
            self.matches.extend(refreshed_matches);
            FinishUpdate(
                owner.GetDocumentMut(),
                StyleUpdateStats {
                    rule_sets_built,
                    ..Default::default()
                },
                StyleUpdateImpact::default(),
            );
            return Ok(StyleUpdateImpact::default());
        }
        let production_resolver_rule_sets = self
            .ProductionResolverRuleSets(document, media, &sheets, active_sheet_scopes.as_slice())
            .ok_or(DocumentStyleError::Unsupported(
                "production-resolver-rule-set-bundle",
            ))?;
        drop(invalidation_trace);
        let _recalc_trace = browser_tracing::span("style", "StyleEngine.RecalcStyle");
        // The batch owns a bounded set of immutable preparation records.
        // No DOM/session state or previous stylesheet revision survives here.
        let rule_collections = Rc::new(
            crate::resolver::production_style_resolver::ProductionRuleCollectionCache::default(),
        );
        let resolver_batch =
            Rc::new(crate::resolver::production_style_resolver::ProductionResolverBatch::default());
        // MatchResult carries the exact highlight names which matched each
        // originating element. No document-wide selector inventory is needed.
        // Build all immutable styles before publishing. The translated
        // StyleResolver owns Blink's initial-style lifecycle; this engine only
        // supplies its immutable host environment through a private adapter.
        let initial_root = crate::resolver::production_style_resolver::InitialStyleForDocument(
            media,
            self.color_scheme_settings,
        );
        let initial_root_ref = unsafe { &*initial_root.Get() };
        // Styles are large boundary projections. Avoid repeatedly relocating
        // all earlier results while accumulating the known dirty subjects.
        let mut pending = Vec::<(usize, ResolvedNodeStyle)>::with_capacity(dirty.len());
        let mut pending_reattach = HashSet::new();
        let mut cleared = Vec::new();
        let mut native = HashMap::<usize, Persistent<ComputedStyle>>::new();
        let mut stack = vec![(
            document.Root(),
            None,
            false,
            false,
            false,
            false,
            false,
            false,
        )];
        let mut stats = StyleUpdateStats {
            rule_sets_built,
            ..Default::default()
        };
        let mut custom_pass_through_nodes = 0usize;
        let mut impact = StyleUpdateImpact::default();
        let mut pending_matches = refreshed_matches;
        let mut pending_values = HashMap::new();
        let mut pending_custom_properties = HashMap::new();
        let mut pending_container_dependencies = Vec::new();
        let mut custom_properties = HashMap::<usize, Rc<CustomProperties>>::new();
        let mut root_font = media.em_size;
        while let Some((
            index,
            parent_index,
            ancestor_none,
            inherited,
            inherited_custom,
            explicit,
            subtree,
            _at_shadow_boundary,
        )) = stack.pop()
        {
            let node = document.Node(index);
            let element = node.Type() == DOMNodeType::kElement;
            let element_scope =
                document.TreeScopeRoot(document.CorrespondingSVGElement(index).unwrap_or(index));
            let element_sheets = scoped_sheets
                .get(&element_scope)
                .map_or(&[][..], Vec::as_slice);
            let mut next_parent = parent_index;
            let mut none = ancestor_none;
            let mut child_inherited = inherited;
            let mut child_custom = inherited_custom;
            let mut child_explicit = explicit;
            let mut child_subtree = subtree;
            if element {
                let old = document.ResolvedStyleFor(index);
                // Element::RecalcOwnStyle only resolves with a stored parent
                // style. A display:none parent normally stores no style; its
                // descendants lose old/ensured styles without selector matching.
                if ensure_target.is_none()
                    && parent_index.is_some_and(|parent| !native.contains_key(&parent))
                {
                    if let Some(old) = old {
                        if !unsafe { old.native_style.Get().as_ref() }
                            .is_some_and(|style| style.IsEnsuredInDisplayNone())
                        {
                            impact.Merge(StyleUpdateImpact::TREE);
                        }
                        stats.changed_nodes += 1;
                        child_subtree = true;
                    }
                    cleared.push(index);
                    next_parent = Some(index);
                    child_inherited = false;
                    child_custom = false;
                    child_explicit = false;
                } else {
                    let inherited_custom_properties = parent_index
                        .and_then(|parent| custom_properties.get(&parent))
                        .cloned();
                    let old_native =
                        old.and_then(|style| unsafe { style.native_style.Get().as_ref() });
                    let old_custom = self.custom_properties.get(&index).cloned();
                    let old_parent_custom =
                        parent_index.and_then(|parent| self.custom_properties.get(&parent));
                    let custom_pass_through_base = inherited_custom
                        && !dirty.contains(&index)
                        && !inherited
                        && !explicit
                        && !subtree
                        && ensure_target.is_none()
                        && self.property_registry.IsEmpty()
                        && inherited_custom_properties.is_some()
                        && self
                            .matches
                            .get(&index)
                            .is_some_and(|signature| signature.native_style_complete);
                    // A variable-free node's computed map is exactly its
                    // parent's map. A variable-using node can also pass the
                    // update through when none of the changed parent names was
                    // read by its previous complete cascade. In that case keep
                    // local winning declarations and patch only inherited
                    // names, avoiding selector matching and the full cascade.
                    let pass_through_custom_properties = custom_pass_through_base
                        .then(|| {
                            let parent = inherited_custom_properties.as_ref()?;
                            let style = old_native?;
                            if !NativeStyleHasVariableDependencies(style) {
                                return Some(parent.clone());
                            }
                            if NativePseudoStyleHasVariableDependencies(style) {
                                return None;
                            }
                            let old = old_custom.as_ref()?;
                            let old_parent = old_parent_custom?;
                            let dependencies =
                                self.matches.get(&index)?.variable_dependencies.as_ref()?;
                            let changed = ChangedCustomPropertyNames(old_parent, parent);
                            if changed
                                .iter()
                                .any(|name| dependencies.referenced.contains(name))
                            {
                                return None;
                            }
                            Some(PatchInheritedCustomProperties(
                                old,
                                parent,
                                &changed,
                                &dependencies.local,
                            ))
                        })
                        .flatten();
                    let pass_through_custom = pass_through_custom_properties.is_some();
                    let must_resolve = if ensure_target.is_some() {
                        ensure_path.contains(&index)
                    } else {
                        dirty.contains(&index)
                            || inherited
                            || inherited_custom && !pass_through_custom
                            || subtree
                            || old.is_none_or(|s| {
                                s.native_style.Get().is_null()
                                    || unsafe { &*s.native_style.Get() }.IsEnsuredInDisplayNone()
                            })
                            || (explicit
                                && old.is_none_or(|s| s.has_explicit_inheritance != Some(false)))
                    };
                    if !must_resolve {
                        let old = old.unwrap();
                        native.insert(index, old.native_style.clone());
                        let old_custom = old_custom.unwrap_or_default();
                        let next_custom = if pass_through_custom {
                            pass_through_custom_properties
                                .expect("custom pass-through computed a retained map")
                        } else {
                            old_custom.clone()
                        };
                        child_custom = pass_through_custom
                            && !CustomPropertiesEquivalent(&old_custom, &next_custom);
                        if pass_through_custom {
                            custom_pass_through_nodes += 1;
                            pending_custom_properties.insert(index, next_custom.clone());
                        }
                        custom_properties.insert(index, next_custom);
                        let old_native = unsafe { &*old.native_style.Get() };
                        none |= old_native.Display() == EDisplay::kNone
                            && !UsesSVGHiddenContainer(document, index, old_native);
                        if parent_index.is_none() {
                            root_font = unsafe { &*old.native_style.Get() }
                                .GetFontDescription()
                                .SpecifiedSize();
                        }
                        next_parent = Some(index);
                        child_inherited = false;
                        child_explicit = false;
                    } else {
                        stats.resolved_nodes += 1;
                        let incremental_style = incremental_style_candidates.contains(&index)
                            && ensure_target.is_none()
                            && !inherited
                            && !explicit
                            && !subtree
                            && !resource_changed
                            && environment_variables.is_none()
                            && self
                                .matches
                                .get(&index)
                                .is_some_and(|signature| signature.native_style_complete)
                            && old.is_some_and(|old| {
                                !unsafe { &*old.native_style.Get() }.IsEnsuredInDisplayNone()
                            });
                        if !incremental_style {
                            self.selectors.borrow().ResetSubjectMatchFlags(index);
                        }
                        let parent_handle = parent_index.and_then(|p| native.get(&p)).cloned();
                        let parent = parent_handle.as_ref().map(|style| unsafe { &*style.Get() });
                        let root_handle = document
                            .Node(document.Root())
                            .Children()
                            .iter()
                            .copied()
                            .find(|&candidate| {
                                document.Node(candidate).Type() == DOMNodeType::kElement
                            })
                            .and_then(|root_index| native.get(&root_index))
                            .cloned();
                        let root_style = root_handle.as_ref().map(|style| unsafe { &*style.Get() });
                        let inherited_custom = parent_index
                            .and_then(|parent| custom_properties.get(&parent))
                            .cloned();
                        // cpp: StyleResolver::ApplyBaseStyleNoCache
                        // (style_resolver.cc:1736-1944). HTML, SVG and MathML
                        // each retain the namespace-specific default RuleSet that
                        // MatchUARules selects. This is the sole base-style path:
                        // its typed result owns native style and custom properties.
                        let rule_sets = &production_resolver_rule_sets;
                        self.translated_rule_shadow_stats.elements_probed += 1;
                        self.translated_rule_shadow_stats
                            .translated_style_resolver_attempts += 1;
                        let resolver_environment = CascadeEnvironment::new(environment_variables);
                        let resolver_attribute_source =
                            PersistentAttributeSource::new(document, index, true);
                        let resolver_attribute_state = CascadeAttributeState::new();
                        let resolver_attributes = AttributeResolver::new(
                            &resolver_attribute_source,
                            &resolver_attribute_state,
                            false,
                        );
                        let resolver_substitution = SubstitutionContext {
                            environment: Some(&resolver_environment),
                            attributes: Some(&resolver_attributes),
                        };
                        let resolver_images = DocumentURLImageResolver {
                            document,
                            originating_index: index,
                        };
                        let registered_container_properties = self
                            .registered_container_properties
                            .as_deref()
                            .unwrap_or(&self.property_registry);
                        let container_runtime = crate::resolver::production_container_runtime::ProductionContainerRuntime {
                                document,
                                media,
                                states: &self.container_states,
                                stored_custom: &self.custom_properties,
                                current_styles: Some(&native),
                                current_custom: Some(&custom_properties),
                                environment: environment_variables,
                                registered: Some(registered_container_properties),
                            };
                        let mut collaborators = crate::resolver::production_style_resolver::ProductionBaseStyleCollaborators::ordinary(root_font);
                        collaborators.rule_collections = Some(rule_collections.clone());
                        collaborators.resolver_batch = Some(resolver_batch.clone());
                        collaborators.declarations = Some(self.declarations.clone());
                        collaborators.html_quirks_user_agent_rules =
                            rule_sets.html_quirks_user_agent.clone();
                        collaborators.forced_colors_user_agent_rules =
                            rule_sets.forced_colors_user_agent.clone();
                        collaborators.pseudo_element_user_agent_rules =
                            rule_sets.marker_user_agent.clone();
                        collaborators.fullscreen_user_agent_rules =
                            rule_sets.fullscreen_user_agent.clone();
                        collaborators.root_style = root_handle.clone();
                        collaborators.color_scheme_settings = self.color_scheme_settings;
                        collaborators.text_link_text_color =
                            self.resolver_dependencies.text_link_text_color;
                        collaborators.is_document_element = parent.is_none();
                        collaborators.subject_node = Some(index);
                        collaborators.is_ensuring_style = ensure_target.is_some();
                        if incremental_style {
                            collaborators.incremental_style = Some(
                                    crate::resolver::production_style_resolver::ProductionIncrementalStyle {
                                        custom_properties: self.custom_properties.get(&index).cloned().unwrap_or_default(),
                                        matched_signatures: &self.matches[&index].values[..self.matches[&index].base_len],
                                    });
                        }
                        collaborators.container_runtime = Some(&container_runtime);
                        collaborators.parent_custom_properties = inherited_custom.as_deref();
                        collaborators.substitution_context = resolver_substitution;
                        collaborators.image_resolver = Some(&resolver_images);
                        collaborators.property_registry = Some(&self.property_registry);
                        collaborators.function_registry = Some(&function_registry);
                        collaborators.font_metrics = self.font_metrics.as_deref();
                        let animation_samples = document.AnimationStyles(document.Node(index).Id());
                        let animation_interpolations = if animation_samples.is_empty() {
                            Ok(None)
                        } else {
                            crate::resolver::production_style_cascade::ParseAnimationInterpolations(
                                animation_samples,
                                DocumentParserMode(document),
                            )
                            .map(Some)
                        };
                        let animation_interpolations = match animation_interpolations {
                            Ok(interpolations) => interpolations,
                            Err(failures) => {
                                self.translated_rule_shadow_stats
                                    .translated_style_resolver_failures += 1;
                                if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                                    eprintln!(
                                            "translated StyleResolver interpolation failure node={index}: {failures:?}"
                                        );
                                }
                                return Err(DocumentStyleError::Unsupported(
                                    "animation-sample-parse",
                                ));
                            }
                        };
                        let transition_samples =
                            document.TransitionStyles(document.Node(index).Id());
                        let transition_interpolations = if transition_samples.is_empty() {
                            Ok(None)
                        } else {
                            crate::resolver::production_style_cascade::ParseTransitionInterpolations(
                                    transition_samples,
                                    DocumentParserMode(document),
                                )
                                .map(Some)
                        };
                        let transition_interpolations = match transition_interpolations {
                            Ok(interpolations) => interpolations,
                            Err(failures) => {
                                self.translated_rule_shadow_stats
                                    .translated_style_resolver_failures += 1;
                                if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                                    eprintln!(
                                            "translated StyleResolver transition failure node={index}: {failures:?}"
                                        );
                                }
                                return Err(DocumentStyleError::Unsupported(
                                    "transition-sample-parse",
                                ));
                            }
                        };
                        collaborators.animations = animation_interpolations.as_ref();
                        collaborators.transitions = transition_interpolations.as_ref();
                        collaborators.matched_properties_cache =
                            Some(self.matched_properties_cache.clone());
                        collaborators.selectors = Some(self.selectors.clone());
                        collaborators.interaction = self.selectors.borrow().interaction;
                        let translated_output = match crate::resolver::production_style_resolver::ApplyBaseStyleForElement(
                                crate::resolver::production_style_resolver::ProductionBaseStyleInputs {
                                    matching: crate::resolver::production_style_resolver::ProductionRuleMatchingInputs {
                                        document,
                                        element_index: index,
                                        media,
                                        html_user_agent_rules: rule_sets.html_user_agent.clone(),
                                        svg_user_agent_rules: rule_sets.svg_user_agent.clone(),
                                        mathml_user_agent_rules: rule_sets.mathml_user_agent.clone(),
                                        author_rule_sets: rule_sets.author.clone(),
                                    },
                                    parent: parent_handle.clone(),
                                    collaborators,
                                },
                            ) {
                                Ok(output) => {
                                    let mut translated = ComputedStyleBuilder::from_style(unsafe {
                                        &*output.style.Get()
                                    });
                                    resolver_environment.ApplyState(&mut translated);
                                    resolver_attribute_state.ApplyState(&mut translated);
                                    let metrics = &output.metrics;
                                    self.translated_rule_shadow_stats.incremental_style_updates += metrics.incremental_style_updates;
                                    self.resolver_dependencies.Merge(metrics);
                                    self.translated_rule_shadow_stats.translated_collector_calls +=
                                        metrics.collector_calls;
                                    self.translated_rule_shadow_stats.candidates_considered +=
                                        metrics.candidates_considered;
                                    self.translated_rule_shadow_stats.selector_matches +=
                                        metrics.selector_matches;
                                    self.translated_rule_shadow_stats
                                        .translated_property_sets_transferred +=
                                        metrics.transferred_property_sets;
                                    self.translated_rule_shadow_stats
                                        .translated_property_sets_consumed +=
                                        metrics.transferred_property_sets;
                                    self.translated_rule_shadow_stats.typed_declarations_used +=
                                        metrics.transferred_property_sets;
                                    if metrics.matched_property_sets != 0
                                        || metrics.has_animations
                                    {
                                        self.translated_rule_shadow_stats
                                            .translated_style_cascade_attempts += 1;
                                        self.translated_rule_shadow_stats
                                            .translated_style_cascade_applied += 1;
                                        if parent.is_none() {
                                            self.translated_rule_shadow_stats
                                                .translated_style_cascade_root_attempts += 1;
                                            self.translated_rule_shadow_stats
                                                .translated_style_cascade_root_applied += 1;
                                        }
                                        if metrics.registry_dependent {
                                            self.translated_rule_shadow_stats
                                                .translated_style_cascade_registered_attempts += 1;
                                            self.translated_rule_shadow_stats
                                                .translated_style_cascade_registered_applied += 1;
                                        }
                                        if metrics.has_animations {
                                            self.translated_rule_shadow_stats
                                                .translated_style_cascade_animation_attempts += 1;
                                            self.translated_rule_shadow_stats
                                                .translated_style_cascade_animation_applied += 1;
                                        }
                                    }
                                    let translated_output = (
                                        translated,
                                        output.custom_properties,
                                        output.matched_signatures,
                                        output.variable_dependencies_complete.then(|| {
                                            Rc::new(RetainedVariableDependencies {
                                                referenced: output.referenced_custom_properties,
                                                local: output.local_custom_properties,
                                            })
                                        }),
                                    );
                                    self.translated_rule_shadow_stats
                                        .translated_style_resolver_applied += 1;
                                    translated_output
                                }
                                Err(failures) => {
                                    self.translated_rule_shadow_stats
                                        .translated_style_resolver_failures += 1;
                                    if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                                        eprintln!(
                                            "translated StyleResolver failure node={index}: {failures:?}"
                                        );
                                    }
                                    return Err(DocumentStyleError::Unsupported(
                                        "production-style-resolver",
                                    ));
                                }
                            };
                        let (
                            mut builder,
                            computed_custom,
                            mut matched_signatures,
                            variable_dependencies,
                        ) = translated_output;
                        let values = HashMap::new();
                        let base_signature_len = matched_signatures.len();
                        ApplyTextControlInnerEditorCustomStyle(document, index, &mut builder);
                        // cpp: MatchResult::SetHasPseudoElementStyle and
                        // StyleResolver::ApplyBaseStyleNoCache (1928). Public pseudo bits
                        // describe matched rules, independently of generated boxes.
                        let mut generated_pseudo_trace = browser_tracing::span(
                            "style_resolve",
                            "StyleEngine.ResolveGeneratedPseudos",
                        );
                        generated_pseudo_trace.set("node_id", node.Id() as f64);
                        let mut pseudo_rules = Vec::new();
                        let mut pseudo_flags = PseudoIdFlags::default();
                        for (pseudo_index, kind) in
                            PseudoStyleKind::MATCH_TARGETS.into_iter().enumerate()
                        {
                            let can_match = match kind {
                                PseudoStyleKind::Marker => matches!(
                                    builder.Display(),
                                    EDisplay::kListItem
                                        | EDisplay::kInlineListItem
                                        | EDisplay::kFlowRootListItem
                                        | EDisplay::kInlineFlowRootListItem
                                ),
                                PseudoStyleKind::Placeholder => {
                                    node.IsHTMLElement("input") || node.IsHTMLElement("textarea")
                                }
                                PseudoStyleKind::BeforeMarker | PseudoStyleKind::AfterMarker => {
                                    let origin = if kind == PseudoStyleKind::BeforeMarker {
                                        PseudoStyleKind::Before
                                    } else {
                                        PseudoStyleKind::After
                                    };
                                    builder.HasPseudoElementStyle(origin.Id())
                                }
                                _ => builder.HasPseudoElementStyle(kind.Id()),
                            };
                            if !can_match {
                                continue;
                            }
                            let rules = self.CollectRules(
                                document,
                                index,
                                media,
                                element_sheets,
                                &scoped_sheets,
                                Some(kind),
                                Some(&native),
                                Some(&custom_properties),
                                environment_variables,
                            )?;
                            matched_signatures.extend(
                                rules
                                    .rules
                                    .iter()
                                    .filter(|rule| rule.order != 0 && rule.order != usize::MAX)
                                    .map(|rule| {
                                        (
                                            rule.order,
                                            rule.specificity | (((pseudo_index + 1) as u32) << 24),
                                            rule.proximity,
                                        )
                                    }),
                            );
                            // cpp: StyleResolver::ForEachUARulesForElement.
                            // Default marker rules are requested for the generated
                            // marker itself. Counting them on every ordinary
                            // element creates a marker bit with no cached style,
                            // making even equal styles differ as kPseudoElementStyle.
                            if !kind.IsNestedMarker()
                                && rules.rules.iter().any(|rule| !rule.user_agent_pseudo_sheet)
                            {
                                pseudo_flags.MaybeSet(kind.Id());
                            }
                            pseudo_rules.push((kind, rules));
                        }
                        // MatchResult carries generated and highlight pseudo bits.
                        // Merge those typed flags with the pseudo styles resolved
                        // below without replacing either source of metadata.
                        for value in PseudoId::kFirstPublicPseudoId.value()
                            ..=PseudoId::kLastTrackedPublicPseudoId.value()
                        {
                            let pseudo_id = PseudoId::from_bits(value);
                            if builder.HasPseudoElementStyle(pseudo_id) {
                                pseudo_flags.MaybeSet(pseudo_id);
                            }
                        }
                        builder.SetPseudoElementStyles(pseudo_flags.Bits());
                        // cpp: StyleAdjuster::AdjustStyleForDisplay. The final computed
                        // display depends on the layout parent: flex/grid children are
                        // blockified in addition to roots, floats and out-of-flow boxes.
                        // Keep this in StyleEngine so Layout consumes the same immutable
                        // ComputedStyle object that Paint and invalidation observe.
                        let display_parent = parent.unwrap_or(initial_root_ref);
                        AdjustComputedDisplayForLayout(
                            &mut builder,
                            display_parent,
                            &DisplayAdjustmentContext {
                                is_root: parent.is_none(),
                                ..Default::default()
                            },
                        );
                        // cpp: Element::RecalcHighlightStyles (element.cc:6747-6828)
                        // and StyleResolver::StyleForHighlightPseudoElement
                        // (style_resolver.cc:10642-10652). Highlight pseudos are
                        // native cached styles on the originating ComputedStyle;
                        // they are not generated PseudoElement boxes. Each request
                        // inherits highlight properties from the parent's matching
                        // highlight style and initializes the remaining properties
                        // from this element's ordinary originating style.
                        self.ResolveFontFeatureValues(&mut builder);
                        if ensure_target.is_some() {
                            builder.SetIsEnsuredInDisplayNone();
                        }
                        let ordinary_style =
                            Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
                        let originating_style = unsafe { &*ordinary_style.Get() };
                        let mut style = ordinary_style.clone();
                        {
                            let mut requests = Vec::<(
                                PseudoId,
                                Option<AtomicString>,
                                SearchTextRequest,
                                *const ComputedStyle,
                            )>::new();
                            let parent_highlights = parent.map(|style| style.HighlightData());
                            // cpp: Element::RecalcHighlightStyles. The ordinary
                            // collector marks highlights matching this element;
                            // a document-wide rule inventory is not that match.
                            // Unmatched elements retain inherited HighlightData.
                            let has_highlight = |id| originating_style.HasPseudoElementStyle(id);
                            if has_highlight(PseudoId::kPseudoIdSelection) {
                                requests.push((
                                    PseudoId::kPseudoIdSelection,
                                    None,
                                    SearchTextRequest::kNone,
                                    parent_highlights
                                        .map_or(std::ptr::null(), |data| data.Selection()),
                                ));
                            }
                            if has_highlight(PseudoId::kPseudoIdSearchText) {
                                requests.push((
                                    PseudoId::kPseudoIdSearchText,
                                    None,
                                    SearchTextRequest::kCurrent,
                                    parent_highlights
                                        .map_or(std::ptr::null(), |data| data.SearchTextCurrent()),
                                ));
                                requests.push((
                                    PseudoId::kPseudoIdSearchText,
                                    None,
                                    SearchTextRequest::kNotCurrent,
                                    parent_highlights.map_or(std::ptr::null(), |data| {
                                        data.SearchTextNotCurrent()
                                    }),
                                ));
                            }
                            if has_highlight(PseudoId::kPseudoIdTargetText) {
                                requests.push((
                                    PseudoId::kPseudoIdTargetText,
                                    None,
                                    SearchTextRequest::kNone,
                                    parent_highlights
                                        .map_or(std::ptr::null(), |data| data.TargetText()),
                                ));
                            }
                            if has_highlight(PseudoId::kPseudoIdSpellingError) {
                                requests.push((
                                    PseudoId::kPseudoIdSpellingError,
                                    None,
                                    SearchTextRequest::kNone,
                                    parent_highlights
                                        .map_or(std::ptr::null(), |data| data.SpellingError()),
                                ));
                            }
                            if has_highlight(PseudoId::kPseudoIdGrammarError) {
                                requests.push((
                                    PseudoId::kPseudoIdGrammarError,
                                    None,
                                    SearchTextRequest::kNone,
                                    parent_highlights
                                        .map_or(std::ptr::null(), |data| data.GrammarError()),
                                ));
                            }
                            if has_highlight(PseudoId::kPseudoIdHighlight) {
                                if let Some(names) =
                                    unsafe { originating_style.CustomHighlightNames().as_ref() }
                                {
                                    for name in names.iter() {
                                        requests.push((
                                            PseudoId::kPseudoIdHighlight,
                                            Some(name.clone()),
                                            SearchTextRequest::kNone,
                                            parent_highlights.map_or(std::ptr::null(), |data| {
                                                data.CustomHighlight(name)
                                            }),
                                        ));
                                    }
                                }
                            }

                            if !requests.is_empty() {
                                {
                                    let rule_sets = &production_resolver_rule_sets;
                                    let mut highlight_builder =
                                        ComputedStyleBuilder::from_style(originating_style);
                                    // Keep every produced style rooted until the
                                    // completed originating style owns the Member
                                    // edges in StyleHighlightData.
                                    let mut highlight_roots = Vec::with_capacity(requests.len());
                                    for (pseudo_id, argument, search_text, highlight_parent) in
                                        requests
                                    {
                                        let resolver_environment =
                                            CascadeEnvironment::new(environment_variables);
                                        let resolver_attribute_source =
                                            PersistentAttributeSource::new(document, index, true);
                                        let resolver_attribute_state = CascadeAttributeState::new();
                                        let resolver_attributes = AttributeResolver::new(
                                            &resolver_attribute_source,
                                            &resolver_attribute_state,
                                            false,
                                        );
                                        let resolver_substitution = SubstitutionContext {
                                            environment: Some(&resolver_environment),
                                            attributes: Some(&resolver_attributes),
                                        };
                                        let resolver_images = DocumentURLImageResolver {
                                            document,
                                            originating_index: index,
                                        };
                                        let root_handle = root_style.map(|root| {
                                            Persistent::from_ptr(
                                                root as *const ComputedStyle as *mut ComputedStyle,
                                            )
                                        });
                                        let highlight_parent =
                                            (!highlight_parent.is_null()).then(|| {
                                                Persistent::from_ptr(highlight_parent.cast_mut())
                                            });
                                        let registered_container_properties = self
                                            .registered_container_properties
                                            .as_deref()
                                            .unwrap_or(&self.property_registry);
                                        let container_runtime = crate::resolver::production_container_runtime::ProductionContainerRuntime {
                                        document,
                                        media,
                                        states: &self.container_states,
                                        stored_custom: &self.custom_properties,
                                        current_styles: Some(&native),
                                        current_custom: Some(&custom_properties),
                                        environment: environment_variables,
                                        registered: Some(registered_container_properties),
                                    };
                                        let mut collaborators = crate::resolver::production_style_resolver::ProductionBaseStyleCollaborators::ordinary(root_font);
                                        collaborators.rule_collections =
                                            Some(rule_collections.clone());
                                        collaborators.resolver_batch = Some(resolver_batch.clone());
                                        collaborators.declarations =
                                            Some(self.declarations.clone());
                                        collaborators.html_quirks_user_agent_rules =
                                            rule_sets.html_quirks_user_agent.clone();
                                        collaborators.forced_colors_user_agent_rules =
                                            rule_sets.forced_colors_user_agent.clone();
                                        collaborators.pseudo_element_user_agent_rules =
                                            rule_sets.marker_user_agent.clone();
                                        collaborators.fullscreen_user_agent_rules =
                                            rule_sets.fullscreen_user_agent.clone();
                                        collaborators.root_style = root_handle;
                                        collaborators.color_scheme_settings =
                                            self.color_scheme_settings;
                                        collaborators.text_link_text_color =
                                            self.resolver_dependencies.text_link_text_color;
                                        collaborators.pseudo_id = pseudo_id;
                                        collaborators.pseudo_argument = argument.clone();
                                        collaborators.search_text = search_text;
                                        collaborators.originating_style =
                                            Some(ordinary_style.clone());
                                        collaborators.subject_node = Some(index);
                                        collaborators.container_runtime = Some(&container_runtime);
                                        // Highlight custom properties inherit from
                                        // the originating element, per the source
                                        // highlight cascade path; highlight color
                                        // properties inherit through `parent`.
                                        collaborators.parent_custom_properties =
                                            Some(computed_custom.as_ref());
                                        collaborators.substitution_context = resolver_substitution;
                                        collaborators.image_resolver = Some(&resolver_images);
                                        collaborators.property_registry =
                                            Some(&self.property_registry);
                                        collaborators.function_registry = Some(&function_registry);
                                        collaborators.font_metrics = self.font_metrics.as_deref();
                                        collaborators.matched_properties_cache =
                                            Some(self.matched_properties_cache.clone());
                                        collaborators.selectors = Some(self.selectors.clone());
                                        collaborators.interaction =
                                            self.selectors.borrow().interaction;
                                        match crate::resolver::production_style_resolver::ApplyBaseStyleForElement(
                                        crate::resolver::production_style_resolver::ProductionBaseStyleInputs {
                                            matching: crate::resolver::production_style_resolver::ProductionRuleMatchingInputs {
                                                document,
                                                element_index: index,
                                                media,
                                                html_user_agent_rules: rule_sets.html_user_agent.clone(),
                                                svg_user_agent_rules: rule_sets.svg_user_agent.clone(),
                                                mathml_user_agent_rules: rule_sets.mathml_user_agent.clone(),
                                                author_rule_sets: rule_sets.author.clone(),
                                            },
                                            parent: highlight_parent,
                                            collaborators,
                                        },
                                    ) {
                                        Ok(output) => {
                                            self.resolver_dependencies.Merge(&output.metrics);
                                            self.translated_rule_shadow_stats
                                                .translated_collector_calls +=
                                                output.metrics.collector_calls;
                                            self.translated_rule_shadow_stats
                                                .candidates_considered +=
                                                output.metrics.candidates_considered;
                                            self.translated_rule_shadow_stats.selector_matches +=
                                                output.metrics.selector_matches;
                                            // A global typed candidate need not
                                            // match this originating element.
                                            // Chromium only marks/recalculates
                                            // the pseudo when its MatchResult
                                            // contains declarations.
                                            if output.metrics.matched_property_sets == 0 {
                                                continue;
                                            }
                                            let mut resolved_highlight =
                                                ComputedStyleBuilder::from_style(unsafe {
                                                    &*output.style.Get()
                                                });
                                            resolver_environment
                                                .ApplyState(&mut resolved_highlight);
                                            resolver_attribute_state
                                                .ApplyState(&mut resolved_highlight);
                                            self.ResolveFontFeatureValues(&mut resolved_highlight);
                                            let resolved_highlight = Rc::new(Persistent::from_ptr(
                                                resolved_highlight.TakeStyle()
                                                    as *mut ComputedStyle,
                                            ));
                                            let pointer = resolved_highlight.Get();
                                            match (pseudo_id, search_text, argument.as_ref()) {
                                                (PseudoId::kPseudoIdSelection, _, _) => highlight_builder
                                                    .AccessHighlightData()
                                                    .SetSelection(pointer),
                                                (PseudoId::kPseudoIdSearchText, SearchTextRequest::kCurrent, _) => highlight_builder
                                                    .AccessHighlightData()
                                                    .SetSearchTextCurrent(pointer),
                                                (PseudoId::kPseudoIdSearchText, SearchTextRequest::kNotCurrent, _) => highlight_builder
                                                    .AccessHighlightData()
                                                    .SetSearchTextNotCurrent(pointer),
                                                (PseudoId::kPseudoIdTargetText, _, _) => highlight_builder
                                                    .AccessHighlightData()
                                                    .SetTargetText(pointer),
                                                (PseudoId::kPseudoIdSpellingError, _, _) => highlight_builder
                                                    .AccessHighlightData()
                                                    .SetSpellingError(pointer),
                                                (PseudoId::kPseudoIdGrammarError, _, _) => highlight_builder
                                                    .AccessHighlightData()
                                                    .SetGrammarError(pointer),
                                                (PseudoId::kPseudoIdHighlight, _, Some(name)) => highlight_builder
                                                    .AccessHighlightData()
                                                    .SetCustomHighlight(name, pointer),
                                                _ => unreachable!("invalid highlight style request"),
                                            }
                                            matched_signatures.extend(output.matched_signatures);
                                            highlight_roots.push(resolved_highlight);
                                        }
                                        Err(failures) => {
                                            self.translated_rule_shadow_stats
                                                .translated_style_resolver_failures += 1;
                                            if std::env::var_os(
                                                "RECHROM_DEBUG_STYLE_RESOLVER",
                                            )
                                            .is_some()
                                            {
                                                eprintln!(
                                                    "translated highlight StyleResolver failure node={index} pseudo={pseudo_id:?}: {failures:?}"
                                                );
                                            }
                                            return Err(DocumentStyleError::Unsupported(
                                                "production-highlight-style-resolver",
                                            ));
                                        }
                                    }
                                    }
                                    style = Persistent::from_ptr(
                                        highlight_builder.TakeStyle() as *mut ComputedStyle
                                    );
                                    drop(highlight_roots);
                                }
                            }
                        }
                        let mut signature = RetainedRuleSignature::with_base(
                            matched_signatures,
                            base_signature_len,
                            ordinary_signature_complete
                                && OrdinarySignatureElement(document, index),
                        );
                        signature.native_style_complete = true;
                        signature.variable_dependencies = variable_dependencies;
                        pending_matches.insert(index, signature);
                        let final_style = unsafe { &*style.Get() };
                        if parent.is_none() {
                            root_font = final_style.GetFontDescription().SpecifiedSize()
                        }
                        if ensure_target.is_none()
                            && !ShouldStoreComputedStyle(document, index, final_style)
                        {
                            if let Some(old) = old {
                                if !unsafe { old.native_style.Get().as_ref() }
                                    .is_some_and(|style| style.IsEnsuredInDisplayNone())
                                {
                                    impact.Merge(StyleUpdateImpact::TREE);
                                }
                                stats.changed_nodes += 1;
                                child_subtree = true;
                            }
                            cleared.push(index);
                            none = true;
                            next_parent = Some(index);
                            child_inherited = false;
                            child_custom = false;
                            child_explicit = false;
                        } else {
                            let own_none = final_style.Display() == EDisplay::kNone;
                            // cpp: SVGGElement::CreateLayoutObject. display:none g
                            // attaches LayoutSVGHiddenContainer; that object suppresses
                            // drawing while keeping descendant resource objects alive.
                            let svg_hidden_container =
                                UsesSVGHiddenContainer(document, index, final_style);
                            none |= own_none && !svg_hidden_container;
                            let mut resolved = ResolvedNodeStyle::default();
                            resolved.own_display_contents =
                                final_style.Display() == EDisplay::kContents;
                            resolved.display_contents = resolved.own_display_contents;
                            resolved.own_generates_box = svg_hidden_container
                                || (!own_none && !resolved.own_display_contents);
                            resolved.generates_box = !none && resolved.own_generates_box;
                            ProjectForLayoutBoundary(final_style, &mut resolved)?;
                            resolved.has_explicit_inheritance =
                                Some(final_style.HasExplicitInheritance());
                            resolved.SetNativeStyle(style.clone());
                            let compare_trace = browser_tracing::span(
                                "style_diff",
                                "StyleEngine.CompareResolvedStyle",
                            );
                            let old_native =
                                old.and_then(|s| unsafe { s.native_style.Get().as_ref() });
                            // cpp: computed_style.cc:407-500; element.cc:5562-5583.
                            // Propagation is decided by the translated native style
                            // difference, not by a hand-maintained projection subset.
                            let native_difference = old_native
                                .map_or(ComputedStyleDifference::kDescendantAffecting, |old| {
                                    style_diff::ComputeDifference(old, final_style)
                                });
                            let old_custom = self.custom_properties.get(&index);
                            let custom_changed = old_custom.is_none_or(|old| {
                                !CustomPropertiesEquivalent(old, &computed_custom)
                            });
                            // A child's highlight style inherits from the matching
                            // highlight style on its flat-tree parent. That dependency
                            // is separate from ordinary CSS inheritance and therefore
                            // must explicitly schedule descendants when the native
                            // highlight cache changes.
                            let highlight_changed = old_native.is_none_or(|old| {
                                old.HighlightData() != final_style.HighlightData()
                            });
                            let suppression_changed = old
                                .is_none_or(|s| s.generates_box != resolved.generates_box)
                                || old_native.is_some_and(|old| {
                                    (old.Display() == EDisplay::kNone) != own_none
                                });
                            drop(compare_trace);
                            if browser_tracing::enabled() {
                                browser_tracing::instant(
                                    "style_diff",
                                    "StylePropagation",
                                    &[
                                        ("node_id", node.Id() as f64),
                                        ("ancestor_display_none", ancestor_none as u8 as f64),
                                        ("had_style", old_native.is_some() as u8 as f64),
                                        ("native_difference", native_difference as u8 as f64),
                                        ("custom_changed", custom_changed as u8 as f64),
                                        ("had_custom", old_custom.is_some() as u8 as f64),
                                        (
                                            "old_custom_values",
                                            old_custom.map_or(0, |value| value.values.len()) as f64,
                                        ),
                                        ("new_custom_values", computed_custom.values.len() as f64),
                                        (
                                            "old_registered_values",
                                            old_custom
                                                .map_or(0, |value| value.registered_values.len())
                                                as f64,
                                        ),
                                        (
                                            "new_registered_values",
                                            computed_custom.registered_values.len() as f64,
                                        ),
                                        ("highlight_changed", highlight_changed as u8 as f64),
                                        ("suppression_changed", suppression_changed as u8 as f64),
                                        (
                                            "initial_data_changed",
                                            old_native.is_none_or(|old| {
                                                old.InitialData() != final_style.InitialData()
                                            }) as u8
                                                as f64,
                                        ),
                                    ],
                                );
                            }
                            child_inherited = native_difference
                                >= ComputedStyleDifference::kIndependentInherited
                                || highlight_changed;
                            child_custom = custom_changed;
                            child_subtree |= suppression_changed
                                || native_difference
                                    == ComputedStyleDifference::kDescendantAffecting;
                            // cpp: StyleResolver::ResolveStyle(StyleRequest),
                            // Element::GetCachedPseudoElementStyle. Pseudos inherit from
                            // their originating element and have a separate match result.
                            let before_has_marker = pseudo_rules.iter().any(|(kind, rules)| {
                                *kind == PseudoStyleKind::BeforeMarker && !rules.rules.is_empty()
                            });
                            let after_has_marker = pseudo_rules.iter().any(|(kind, rules)| {
                                *kind == PseudoStyleKind::AfterMarker && !rules.rules.is_empty()
                            });
                            let mut before_custom = None;
                            let mut after_custom = None;
                            for (kind, rules) in pseudo_rules
                                .iter()
                                .filter(|(kind, _)| !kind.IsNestedMarker())
                            {
                                let output = self.ResolvePseudoStyle(
                                    document,
                                    index,
                                    *kind,
                                    &resolved,
                                    &computed_custom,
                                    initial_root_ref,
                                    root_font,
                                    media,
                                    environment_variables,
                                    &rules,
                                    Some(&function_registry),
                                    !none,
                                    match kind {
                                        PseudoStyleKind::Before => before_has_marker,
                                        PseudoStyleKind::After => after_has_marker,
                                        _ => false,
                                    },
                                    Some(&production_resolver_rule_sets),
                                    &rule_collections,
                                    &resolver_batch,
                                    &native,
                                    &custom_properties,
                                    root_style,
                                )?;
                                match kind {
                                    PseudoStyleKind::Before => {
                                        if let Some(output) = output {
                                            before_custom = Some(output.custom_properties);
                                            resolved.before = Some(output.pseudo);
                                        }
                                    }
                                    PseudoStyleKind::After => {
                                        if let Some(output) = output {
                                            after_custom = Some(output.custom_properties);
                                            resolved.after = Some(output.pseudo);
                                        }
                                    }
                                    PseudoStyleKind::FirstLetter => {
                                        resolved.first_letter = output.map(|output| output.pseudo)
                                    }
                                    PseudoStyleKind::Placeholder => {
                                        resolved.placeholder = output.map(|output| output.pseudo)
                                    }
                                    PseudoStyleKind::Marker => {
                                        resolved.marker = output.map(|output| output.pseudo)
                                    }
                                    PseudoStyleKind::BeforeMarker
                                    | PseudoStyleKind::AfterMarker => {
                                        unreachable!(
                                            "nested markers resolve after their originating pseudo"
                                        )
                                    }
                                }
                            }
                            for kind in
                                [PseudoStyleKind::BeforeMarker, PseudoStyleKind::AfterMarker]
                            {
                                let Some((_, rules)) =
                                    pseudo_rules.iter().find(|(target, _)| *target == kind)
                                else {
                                    continue;
                                };
                                let (parent_pseudo, parent_custom, old_marker) = match kind {
                                    PseudoStyleKind::BeforeMarker => (
                                        resolved.before.as_ref(),
                                        before_custom.as_ref(),
                                        old.and_then(|style| style.before.as_ref())
                                            .and_then(|pseudo| pseudo.marker.as_deref()),
                                    ),
                                    PseudoStyleKind::AfterMarker => (
                                        resolved.after.as_ref(),
                                        after_custom.as_ref(),
                                        old.and_then(|style| style.after.as_ref())
                                            .and_then(|pseudo| pseudo.marker.as_deref()),
                                    ),
                                    _ => unreachable!(),
                                };
                                let (Some(parent_pseudo), Some(parent_custom)) =
                                    (parent_pseudo, parent_custom)
                                else {
                                    continue;
                                };
                                let marker = self.ResolveNestedMarkerStyle(
                                    document,
                                    index,
                                    kind,
                                    parent_pseudo,
                                    parent_custom,
                                    old_marker,
                                    initial_root_ref,
                                    root_font,
                                    media,
                                    environment_variables,
                                    rules,
                                    Some(&function_registry),
                                    !none,
                                    &production_resolver_rule_sets,
                                    &rule_collections,
                                    &resolver_batch,
                                    &native,
                                    &custom_properties,
                                    root_style,
                                )?;
                                match kind {
                                    PseudoStyleKind::BeforeMarker => {
                                        resolved.before.as_mut().unwrap().marker =
                                            marker.map(Box::new)
                                    }
                                    PseudoStyleKind::AfterMarker => {
                                        resolved.after.as_mut().unwrap().marker =
                                            marker.map(Box::new)
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            generated_pseudo_trace.set("requested", pseudo_rules.len() as f64);
                            drop(generated_pseudo_trace);
                            let mut finalize_trace = browser_tracing::span(
                                "style_resolve",
                                "StyleEngine.FinalizeResolvedStyle",
                            );
                            // The preliminary difference controls inheritance. Compare
                            // again after cache population so pseudo content changes
                            // participate in the native public-pseudo difference.
                            let final_difference = {
                                let _diff_trace = browser_tracing::span(
                                    "style_diff",
                                    "ComputedStyle.ComputeFinalDifference",
                                );
                                if pseudo_rules.is_empty() {
                                    // No generated-pseudo cache was touched since the
                                    // preliminary comparison; the same immutable inputs
                                    // cannot produce a different native style difference.
                                    native_difference
                                } else {
                                    old_native.map_or(
                                        ComputedStyleDifference::kDescendantAffecting,
                                        |old| style_diff::ComputeDifference(old, final_style),
                                    )
                                }
                            };
                            let equal = final_difference == ComputedStyleDifference::kEqual
                        // Generated style equality deliberately excludes these
                        // derived flags. A real page/preference update still
                        // publishes their newly computed native values.
                        && old_native.is_some_and(|old| {
                            // A query cache must become an ordinary style even
                            // when its physical box stays suppressed (SVG g).
                            old.IsEnsuredInDisplayNone() == final_style.IsEnsuredInDisplayNone()
                                && old.DarkColorScheme() == final_style.DarkColorScheme()
                                && old.ColorSchemeForced() == final_style.ColorSchemeForced()
                                && old.ColorSchemeFlagsIsNormal() == final_style.ColorSchemeFlagsIsNormal()
                                && old.DependsOnSizeContainerQueries() == final_style.DependsOnSizeContainerQueries()
                                && old.FirstLineDependsOnSizeContainerQueries() == final_style.FirstLineDependsOnSizeContainerQueries()
                                && old.HighlightsDependOnSizeContainerQueries() == final_style.HighlightsDependOnSizeContainerQueries()
                                && old.DependsOnStyleContainerQueries() == final_style.DependsOnStyleContainerQueries()
                                && old.DependsOnScrollStateContainerQueries() == final_style.DependsOnScrollStateContainerQueries()
                                && old.DependsOnAnchoredContainerQueries() == final_style.DependsOnAnchoredContainerQueries()
                                && old.HighlightPseudoElementStylesDependOnContainerUnits() == final_style.HighlightPseudoElementStylesDependOnContainerUnits()
                        })
                        // ::placeholder uses an internal PseudoId and is not
                        // covered by ComputedStyle's public-pseudo flag loop.
                        // Its form-control lifecycle still owns a distinct cache.
                        && old_native.is_some_and(|old| PseudoStyleKind::DIRECT.into_iter().all(|kind| {
                            old.GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                                == final_style.GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                        }))
                        // ApplyComputedStyleDiff keeps forced descendant
                        // traversal separate from value equality. child_subtree
                        // already carries that traversal below; it must not
                        // force publishing an unchanged style on every child.
                        && old.is_some_and(|old| {
                            // Normalize only this unpublished result, and only
                            // after native equality has been established. A
                            // changed style needs no cloned comparison payload.
                            // Equal styles retain the committed native owner;
                            // a projection mismatch restores the new owner.
                            resolved.SetNativeStyle(old.native_style.clone());
                            if old == &resolved { true } else {
                                resolved.SetNativeStyle(style.clone());
                                false
                            }
                        });
                            finalize_trace.set("equal", equal as u8 as f64);
                            child_explicit = !equal;
                            pending_values.insert(index, values);
                            pending_custom_properties.insert(index, computed_custom.clone());
                            custom_properties.insert(index, computed_custom);
                            // A root font change also invalidates rem users outside ordinary
                            // inheritance. Until unit dependency flags are retained, visit
                            // the connected document for that source-defined dependency.
                            if parent.is_none()
                                && old_native.is_some_and(|old| {
                                    old.GetFontDescription().SpecifiedSize()
                                        != final_style.GetFontDescription().SpecifiedSize()
                                })
                            {
                                child_subtree = true;
                            }
                            // Equality above already retained the committed owner.
                            // Retain the native dependency facts for the next targeted
                            // container recalc; no arena-wide style/flag scan is needed.
                            pending_container_dependencies.push((
                                index,
                                ContainerQueryDependent(unsafe { &*resolved.native_style.Get() }),
                            ));
                            native.insert(index, resolved.native_style.clone());
                            // `equal` already proved full projection equality. Avoid
                            // traversing the same large boundary payload a second time.
                            if !equal && old != Some(&resolved) {
                                stats.changed_nodes += 1;
                                if ensure_target.is_none() {
                                    let (change, reattach) =
                                        StyleImpact(document, index, old, &resolved);
                                    impact.Merge(change);
                                    if reattach {
                                        pending_reattach.insert(node.Id());
                                    }
                                }
                                pending.push((index, resolved));
                            }
                            next_parent = Some(index);
                            drop(finalize_trace);
                        }
                    }
                }
            }
            let mut push_child = |child: usize,
                                  parent: Option<usize>,
                                  ancestor_none: bool,
                                  inherited: bool,
                                  inherited_custom: bool,
                                  explicit: bool,
                                  subtree: bool,
                                  at_shadow_boundary: bool| {
                let should_visit = if ensure_target.is_some() {
                    visit.contains(&child)
                } else {
                    inherited || inherited_custom || explicit || subtree || visit.contains(&child)
                };
                if should_visit {
                    stack.push((
                        child,
                        parent,
                        ancestor_none,
                        inherited,
                        inherited_custom,
                        explicit,
                        subtree,
                        at_shadow_boundary,
                    ));
                }
            };
            if element && document.ShadowRoot(index).is_some() {
                // cpp: flat_tree_traversal.cc:104-123,161-191. Light-DOM
                // children of a shadow host are never direct flat children.
                // Assigned nodes are entered by their slot; unassigned roots
                // still receive style, but have no style parent.
                for &child in node.Children().iter().rev() {
                    if document.AssignedSlot(child).is_none() {
                        push_child(
                            child,
                            None,
                            false,
                            false,
                            false,
                            false,
                            child_subtree,
                            false,
                        );
                    }
                }
                let shadow_root = document.ShadowRoot(index).unwrap();
                for &child in document.Node(shadow_root).Children().iter().rev() {
                    push_child(
                        child,
                        next_parent,
                        none,
                        child_inherited,
                        child_custom,
                        child_explicit,
                        child_subtree,
                        true,
                    );
                }
            } else if element && document.IsSlotSupportingAssignment(index) {
                let assigned = AssignedNodesForSlot(document, index);
                if assigned.is_empty() {
                    for &child in node.Children().iter().rev() {
                        push_child(
                            child,
                            next_parent,
                            none,
                            child_inherited,
                            child_custom,
                            child_explicit,
                            child_subtree,
                            false,
                        );
                    }
                } else {
                    // Fallback content remains styled while inactive, but its
                    // root has no flat-tree parent and starts from initial.
                    for &child in node.Children().iter().rev() {
                        push_child(
                            child,
                            None,
                            false,
                            false,
                            false,
                            false,
                            child_subtree,
                            false,
                        );
                    }
                    for child in assigned.into_iter().rev() {
                        push_child(
                            child,
                            next_parent,
                            none,
                            child_inherited,
                            child_custom,
                            child_explicit,
                            child_subtree,
                            false,
                        );
                    }
                }
            } else {
                for &child in node.Children().iter().rev() {
                    push_child(
                        child,
                        next_parent,
                        none,
                        child_inherited,
                        child_custom,
                        child_explicit,
                        child_subtree,
                        false,
                    );
                }
            }
        }
        drop(sheets);
        // Remove obsolete resource styles before resolving paint-server clients;
        // the pending batch must never fall back to a discarded document style.
        for &index in &cleared {
            owner.GetDocumentMut().ClearResolvedStyle(index);
        }
        if ensure_target.is_none() {
            let document = owner.GetDocument();
            self.svg_style_elements.retain(|index| {
                document.ResolvedStyleFor(*index).is_some()
                    && document.Node(*index).Namespace() == DOMNamespace::kSVG
            });
            for &(index, _) in &pending {
                if document.Node(index).Namespace() == DOMNamespace::kSVG {
                    self.svg_style_elements.insert(index);
                } else {
                    self.svg_style_elements.remove(&index);
                }
            }
            ResolveSVGPaintServers(
                document,
                &self.svg_style_elements,
                &mut pending,
                &mut stats,
                &mut impact,
                &mut pending_reattach,
            );
        }
        self.sheet_revision = Some(revision);
        self.active_sheets = active_sheets;
        self.active_sheet_scopes = active_sheet_scopes;
        self.environment = Some(environment);
        self.sheet_dependencies = sheet_dependencies;
        self.matches.extend(pending_matches);
        self.cascade_values.extend(pending_values);
        self.custom_properties.extend(pending_custom_properties);
        self.invalidated_container_nodes.clear();
        self.interaction_changed_nodes.clear();
        self.interaction_changed_pseudos.clear();
        self.interaction_invalidation_roots.clear();
        for (index, dependent) in pending_container_dependencies {
            if dependent {
                self.container_dependent_elements.insert(index);
            } else {
                self.container_dependent_elements.remove(&index);
            }
        }
        let document = owner.GetDocumentMut();
        let _publish_trace = browser_tracing::span("style_commit", "StyleEngine.PublishStyles");
        for index in cleared {
            self.matches.remove(&index);
            self.cascade_values.remove(&index);
            self.custom_properties.remove(&index);
            self.viewport_dependent_elements.remove(&index);
            self.container_dependent_elements.remove(&index);
        }
        {
            let mut commit = DocumentResolvedStyleCommit {
                document,
                viewport_dependent_elements: &mut self.viewport_dependent_elements,
            };
            style_commit::CommitResolvedStyles(&mut commit, pending);
        }
        document
            .StyleStateMut()
            .layout_reattach_nodes
            .extend(pending_reattach);
        FinishUpdate(document, stats, impact);
        update_trace.set(
            "translated_resolver_attempts",
            self.translated_rule_shadow_stats
                .translated_style_resolver_attempts as f64,
        );
        update_trace.set(
            "translated_resolver_applied",
            self.translated_rule_shadow_stats
                .translated_style_resolver_applied as f64,
        );
        update_trace.set(
            "translated_resolver_failures",
            self.translated_rule_shadow_stats
                .translated_style_resolver_failures as f64,
        );
        update_trace.set(
            "translated_candidates_considered",
            self.translated_rule_shadow_stats.candidates_considered as f64,
        );
        update_trace.set(
            "translated_selector_matches",
            self.translated_rule_shadow_stats.selector_matches as f64,
        );
        update_trace.set(
            "custom_pass_through_nodes",
            custom_pass_through_nodes as f64,
        );
        Ok(impact)
    }
    fn SynchronizeTypedSheets(
        &mut self,
        document: &Document,
        sheets: &[(Rc<CSSStyleSheet>, CascadeOrigin)],
    ) {
        let mut trace = browser_tracing::span("style_rules", "StyleEngine.SynchronizeTypedSheets");
        let mut inline_hits = 0;
        let active_keys = sheets
            .iter()
            .map(|(sheet, _)| (sheet.Identity(), sheet.Revision()))
            .filter(|(identity, _)| *identity != 0)
            .collect::<HashSet<_>>();
        // Old canonical contents must stay alive until new attachments have
        // consulted the weak text cache, including replacing all old owners.
        for (sheet, origin) in sheets {
            let key = (sheet.Identity(), sheet.Revision());
            let Some(expected_mode) = ExpectedSheetParserMode(document, *origin) else {
                continue;
            };
            if key.0 == 0
                || self
                    .typed_sheets
                    .get(&key)
                    .is_some_and(|typed| typed.contents.ParserContext().Mode() == expected_mode)
            {
                continue;
            }
            // owner_media_conditions is attachment metadata, outside the
            // parsed StyleSheetContents just as LinkStyle's media owner is in
            // Blink. RuleSet consumers must continue evaluating that outer
            // condition from CSSOM rather than treating this owner as merged.
            let transferred = sheet
                .ParsedBacking::<crate::production_style_sheet::ProductionStyleSheet>()
                .filter(|parsed| {
                    parsed.source.Utf8() == sheet.SourceText()
                        && parsed.contents.ParserContext().Mode() == expected_mode
                });
            let parsed = if let Some(transferred) = transferred {
                transferred
            } else {
                // A directly edited CSSOM projection deliberately invalidates
                // its source provenance. Do not build a typed owner from stale
                // text. The update fails at the required typed RuleSet boundary
                // until the CSSOM mutation API supplies a newly parsed revision.
                if !sheet.HasCurrentSource() {
                    continue;
                }
                self.typed_sheet_source_parses += 1;
                let source = foundation::String::FromUtf8(sheet.SourceText().as_bytes());
                Rc::new(crate::production_style_sheet::ParseStyleSheet(
                    &source,
                    expected_mode,
                ))
            };
            let parsed = if document
                .FindNodeById(sheet.owner_node_id)
                .is_some_and(|index| document.Node(index).IsHTMLElement("style"))
            {
                // Parser-delivered sheets already carry a typed backing and
                // bypass SynchronizeStyleSheets. They must consume the same
                // document-owned contents cache as DOM text replacements.
                let (parsed, hit) = self.InternInlineSheetContents(parsed);
                inline_hits += usize::from(hit);
                parsed
            } else {
                parsed
            };
            if parsed.diagnostics.iter().any(|diagnostic| {
                diagnostic.kind == crate::production_style_sheet::DiagnosticKind::Unsupported
            }) {
                self.diagnostics.push(DocumentStyleError::Unsupported(
                    "typed stylesheet contains unsupported syntax",
                ));
            }
            self.typed_sheets.insert(key, parsed);
        }
        self.typed_sheets.retain(|key, _| active_keys.contains(key));
        trace.set("inline_hits", inline_hits as f64);
    }

    // cpp: StyleEngine::FindStyleSheetContents / AddStyleSheetContents. CSSOM
    // attachment identity stays distinct; only immutable parse contents share.
    fn InternInlineSheetContents(
        &mut self,
        parsed: Rc<crate::production_style_sheet::ProductionStyleSheet>,
    ) -> (
        Rc<crate::production_style_sheet::ProductionStyleSheet>,
        bool,
    ) {
        if !parsed.contents.IsCacheableForStyleElement() {
            return (parsed, false);
        }
        let source = parsed.source.Utf8();
        let key = InlineSheetContentsKey(&source);
        if let Some(cached) = self
            .inline_sheet_contents
            .get(&key)
            .and_then(std::rc::Weak::upgrade)
            .filter(|cached| {
                cached.contents.IsCacheableForStyleElement()
                    && cached.source.Utf8() == source
                    && cached.contents.ParserContext() == parsed.contents.ParserContext()
            })
        {
            return (cached, true);
        }
        self.inline_sheet_contents
            .insert(key, Rc::downgrade(&parsed));
        (parsed, false)
    }
    fn SynchronizeTranslatedRuleSets(
        &mut self,
        sheets: &[(Rc<CSSStyleSheet>, CascadeOrigin)],
        media: &MediaValuesCachedData,
    ) -> bool {
        let environment_changed = self.translated_rule_set_environment.as_ref() != Some(media);
        let viewport_only_change = self
            .translated_rule_set_environment
            .as_ref()
            .is_some_and(|previous| MediaDiffersOnlyByViewport(previous, media));
        let evaluator =
            crate::production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(media);
        let forced_colors_evaluator =
            crate::production_style_sheet::ProductionMediaQueryEvaluator::ForMediaType(
                "ua-forced-colors",
                media,
            );
        let owner_media_unchanged = viewport_only_change
            && self
                .translated_rule_set_environment
                .as_ref()
                .is_some_and(|previous| {
                    let previous_evaluator = crate::production_style_sheet::
                        ProductionMediaQueryEvaluator::FromCachedData(previous);
                    sheets.iter().all(|(sheet, _)| {
                        sheet.owner_media_conditions.iter().all(|condition| {
                            let queries = self
                                .media_query_cache
                                .entry(condition.clone())
                                .or_insert_with(|| Rc::new(ParseMediaQuerySet(condition)))
                                .clone();
                            previous_evaluator.EvalSet(queries.as_ref())
                                == evaluator.EvalSet(queries.as_ref())
                        })
                    })
                });
        let retained_viewport_environment = viewport_only_change
            && owner_media_unchanged
            && sheets.iter().all(|(sheet, _)| {
                let key = (sheet.Identity(), sheet.Revision());
                let Some(rules) = self.translated_rule_sets.get(&key) else {
                    return false;
                };
                let medium = if sheet.UserAgentStyleSheetRole()
                    == Some(CSSUserAgentStyleSheetRole::kForcedColors)
                {
                    &forced_colors_evaluator
                } else {
                    &evaluator
                };
                !rules.DidMediaQueryResultsChange(medium)
            });
        if environment_changed && !retained_viewport_environment {
            if viewport_only_change && owner_media_unchanged {
                // cpp: StyleSheetContents::EnsureRuleSet. A viewport change
                // rebuilds only RuleSets whose cached media-query result
                // changed. Other sheets retain both RuleSet and routing
                // identity, so ApplyRuleSetChanges can invalidate against the
                // exact old/new pair instead of treating the document as new.
                let changed = sheets
                    .iter()
                    .filter_map(|(sheet, _)| {
                        let key = (sheet.Identity(), sheet.Revision());
                        let rules = self.translated_rule_sets.get(&key)?;
                        let medium = if sheet.UserAgentStyleSheetRole()
                            == Some(CSSUserAgentStyleSheetRole::kForcedColors)
                        {
                            &forced_colors_evaluator
                        } else {
                            &evaluator
                        };
                        rules.DidMediaQueryResultsChange(medium).then_some(key)
                    })
                    .collect::<Vec<_>>();
                for key in changed {
                    self.translated_rule_sets.remove(&key);
                }
                // Entries for the rebuilt sheets are environment-specific.
                // Retained translated RuleSets above remain authoritative;
                // lazily repopulate the shared lookup from this environment.
                self.shared_rule_sets.clear();
            } else {
                self.translated_rule_sets.clear();
                self.shared_rule_sets.clear();
            }
        }
        if environment_changed {
            self.translated_rule_set_environment = Some(media.clone());
        }
        let active_keys = sheets
            .iter()
            .map(|(sheet, _)| (sheet.Identity(), sheet.Revision()))
            .filter(|(identity, _)| *identity != 0)
            .collect::<HashSet<_>>();
        self.translated_rule_sets
            .retain(|key, _| active_keys.contains(key));
        self.shared_rule_sets
            .retain(|_, (contents, _)| contents.strong_count() != 0);
        for (sheet, _) in sheets {
            let key = (sheet.Identity(), sheet.Revision());
            if key.0 == 0 || self.translated_rule_sets.contains_key(&key) {
                continue;
            }
            // A directly edited CSSOM clone intentionally drops its typed
            // provenance. Do not build a RuleSet from stale source. The
            // required typed resolver rejects that update.
            let Some(typed) = self.typed_sheets.get(&key) else {
                continue;
            };
            let contents_key = (
                Rc::as_ptr(typed) as usize,
                sheet.UserAgentStyleSheetRole() == Some(CSSUserAgentStyleSheetRole::kForcedColors),
            );
            if let Some((_, rules)) = self.shared_rule_sets.get(&contents_key) {
                self.translated_rule_sets.insert(key, rules.clone());
                continue;
            }
            let medium = if sheet.UserAgentStyleSheetRole()
                == Some(CSSUserAgentStyleSheetRole::kForcedColors)
            {
                &forced_colors_evaluator
            } else {
                &evaluator
            };
            let rule_set = Rc::new(typed.BuildRuleSet(medium));
            self.shared_rule_sets
                .insert(contents_key, (Rc::downgrade(typed), rule_set.clone()));
            self.translated_rule_sets.insert(key, rule_set);
            self.translated_rule_shadow_stats.rule_sets_built += 1;
        }
        retained_viewport_environment
    }

    // cpp: ScopedStyleResolver::AddFontFeatureValuesRules, CSSFontSelector::
    // GetFontData, and StyleEngine::ApplyRuleSetChanges. At the pinned source
    // revision these aliases are document-author scoped; shadow/user-sheet
    // table fusion remains a source TODO. Owner media is outside RuleSet.
    fn SynchronizeFontFeatureValues(
        &mut self,
        document: &Document,
        media: &MediaValuesCachedData,
        sheets: &[(Rc<CSSStyleSheet>, CascadeOrigin)],
        sheet_scopes: &[Option<usize>],
    ) -> bool {
        let values = DocumentMediaValues::new(document, media);
        let evaluator = crate::media_queries::media_query_backend::ForMediaValues(&values);
        let mut active = Vec::new();
        for (index, (sheet, origin)) in sheets.iter().enumerate() {
            if *origin != CascadeOrigin::kAuthor
                || sheet_scopes.get(index).copied().flatten() != Some(document.Root())
            {
                continue;
            }
            if !sheet.owner_media_conditions.iter().all(|condition| {
                let query = self
                    .media_query_cache
                    .entry(condition.clone())
                    .or_insert_with(|| Rc::new(ParseMediaQuerySet(condition)))
                    .clone();
                evaluator.Eval(query.as_ref())
            }) {
                continue;
            }
            // A CSSOM projection without authoritative backing cannot provide
            // feature-value rules. SynchronizeTypedSheets deliberately refuses
            // stale source; retain that boundary instead of reparsing it here.
            if let Some(rules) = self
                .translated_rule_sets
                .get(&(sheet.Identity(), sheet.Revision()))
            {
                active.push(crate::active_style_sheets::ActiveStyleSheet::new(
                    Rc::new(()),
                    Some(rules.clone()),
                ));
            }
        }
        let layers = crate::cascade_layer_map::CascadeLayerMap::new(&active);
        let mut next = crate::font_feature_values_registry::FontFeatureValuesRegistry::default();
        for sheet in &active {
            next.AddRules(
                sheet.rule_set.as_ref().unwrap().FontFeatureValuesRules(),
                Some(&layers),
            );
        }
        let changed = self.font_feature_values != next;
        self.font_feature_values = next;
        changed
    }

    // CSSFontSelector resolves a copy of the description independently for each
    // requested CSS family before selecting a face. The immutable family map
    // expresses that semantic boundary without exposing Style's registry or
    // live callbacks to FontEngine. The symbolic alternates remain canonical.
    fn ResolveFontFeatureValues(&self, builder: &mut ComputedStyleBuilder) {
        // cpp: StyleBuilderConverter::ConvertFontVariantAlternates. There is
        // no family-dependent work when the computed value has no symbolic
        // alternates. Avoid cloning and restaging FontDescription for every
        // ordinary element on the document traversal.
        let Some(alternates) = builder
            .GetFontDescription()
            .FontVariantAlternatesValue()
            .cloned()
        else {
            return;
        };
        let mut description = builder.GetFontDescription().clone();
        let mut resolved = Vec::new();
        let mut family = Some(description.Family());
        while let Some(current) = family {
            let storage = self
                .font_feature_values
                .FontFeatureValuesForFamily(current.FamilyName());
            let alternate = alternates.Resolve(
                |name| storage.map_or_else(Vec::new, |s| s.ResolveStylistic(name)),
                |name| storage.map_or_else(Vec::new, |s| s.ResolveStyleset(name)),
                |name| storage.map_or_else(Vec::new, |s| s.ResolveCharacterVariant(name)),
                |name| storage.map_or_else(Vec::new, |s| s.ResolveSwash(name)),
                |name| storage.map_or_else(Vec::new, |s| s.ResolveOrnaments(name)),
                |name| storage.map_or_else(Vec::new, |s| s.ResolveAnnotation(name)),
            );
            resolved.push((
                current.FamilyName().clone(),
                alternate.GetResolvedFontFeatures().clone(),
            ));
            family = unsafe { current.Next().as_ref() };
        }
        description.SetResolvedFontFeaturesForFamilies(resolved.into());
        production_style_builder::StageFontDescription(builder, &description);
    }

    /// Assemble the exact active RuleSet owners consumed by
    /// StyleResolver::MatchAllRules. Owner-level media is outside
    /// StyleSheetContents in Chromium, so it is filtered here before the
    /// retained RuleSet enters the resolver.
    fn ProductionResolverRuleSets(
        &mut self,
        document: &Document,
        media: &MediaValuesCachedData,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
        sheet_scopes: &[Option<usize>],
    ) -> Option<ProductionResolverRuleSets> {
        let values = DocumentMediaValues::new(document, media);
        let evaluator = crate::media_queries::media_query_backend::ForMediaValues(&values);
        let mut html_user_agent = None;
        let mut html_quirks_user_agent = None;
        let mut forced_colors_user_agent = None;
        let mut marker_user_agent = None;
        let mut fullscreen_user_agent = None;
        let mut svg_user_agent = None;
        let mut mathml_user_agent = None;
        let mut author = Vec::new();
        let mut sheet_bases = HashMap::<Option<usize>, usize>::new();
        for (sheet_index, &(sheet, origin)) in sheets.iter().enumerate() {
            let tree_scope_root = match origin {
                CascadeOrigin::kUserAgent => None,
                // ActiveStyleSheetsWithTreeScopes expands one constructed
                // sheet once per adopting TreeScope. Preserve that expanded
                // owner here; CSSStyleSheet's owner node cannot represent a
                // constructed sheet adopted by several scopes.
                CascadeOrigin::kAuthor => sheet_scopes.get(sheet_index).copied().flatten(),
                _ => None,
            };
            let sheet_base = sheet_bases.entry(tree_scope_root).or_default();
            let current_sheet_base = *sheet_base;
            *sheet_base += sheet.rules.len();
            if !sheet.owner_media_conditions.iter().all(|condition| {
                let query = self
                    .media_query_cache
                    .entry(condition.clone())
                    .or_insert_with(|| Rc::new(ParseMediaQuerySet(condition)))
                    .clone();
                evaluator.Eval(query.as_ref())
            }) {
                self.translated_rule_shadow_stats.skipped_owner_media += 1;
                continue;
            }
            let key = (sheet.Identity(), sheet.Revision());
            let typed = self.typed_sheets.get(&key)?;
            let rule_set = self.translated_rule_sets.get(&key)?.clone();
            let expected_mode = ExpectedSheetParserMode(document, origin)?;
            if typed.contents.ParserContext().Mode() != expected_mode {
                return None;
            }
            self.translated_rule_shadow_stats.rule_sets_consulted += 1;
            let implicit_scope_trigger = tree_scope_root
                .and_then(|scope| {
                    document.AdoptedStyleSheetImplicitScopeTrigger(sheet.Identity(), scope)
                })
                .or_else(|| {
                    document
                        .FindNodeById(sheet.owner_node_id)
                        .and_then(|owner| document.ParentOrShadowHostElement(owner))
                });
            let input =
                crate::resolver::production_style_resolver::ProductionResolverRuleSetInput {
                    rule_set,
                    implicit_scope_trigger,
                    sheet_base: current_sheet_base,
                    tree_scope_root,
                };
            match origin {
                CascadeOrigin::kUserAgent => {
                    let slot = match sheet
                        .UserAgentStyleSheetRole()
                        .unwrap_or(CSSUserAgentStyleSheetRole::kHTML)
                    {
                        CSSUserAgentStyleSheetRole::kHTML => &mut html_user_agent,
                        CSSUserAgentStyleSheetRole::kHTMLQuirks => &mut html_quirks_user_agent,
                        CSSUserAgentStyleSheetRole::kForcedColors => &mut forced_colors_user_agent,
                        CSSUserAgentStyleSheetRole::kMarker => &mut marker_user_agent,
                        CSSUserAgentStyleSheetRole::kFullscreen => &mut fullscreen_user_agent,
                        CSSUserAgentStyleSheetRole::kSVG => &mut svg_user_agent,
                        CSSUserAgentStyleSheetRole::kMathML => &mut mathml_user_agent,
                    };
                    // Chromium owns one aggregate RuleSet for each of these
                    // roles. Duplicate role owners are an invalid bundle; do
                    // not publish a partially assembled translated cascade.
                    if slot.replace(input).is_some() {
                        return None;
                    }
                }
                CascadeOrigin::kAuthor => author.push(input),
                _ => unreachable!("origin was checked above"),
            }
        }
        let html_user_agent = html_user_agent.unwrap_or_else(|| {
            crate::resolver::production_style_resolver::ProductionResolverRuleSetInput::plain(
                Rc::new(crate::rule_set::RuleSet::new()),
            )
        });
        Some(ProductionResolverRuleSets {
            html_user_agent,
            html_quirks_user_agent,
            forced_colors_user_agent,
            marker_user_agent,
            fullscreen_user_agent,
            svg_user_agent,
            mathml_user_agent,
            author: Rc::new(author),
        })
    }
    // Source's invalidation traversal is separate from the style resolver.
    // Relations widen candidates; retained matched-rule signatures decide which
    // candidates need a cascade. This includes losing a selector match.
    // cpp: StyleEngine::{Class,Id,Attribute}ChangedForElement ->
    // PendingInvalidations -> StyleInvalidator. DOM supplies changed keys,
    // RuleSet supplies retained subject sets. HTML state/presentation and
    // cross-tree changes keep the existing conservative route.
    fn InvalidateAttributeRelations(
        &self,
        document: &Document,
        index: usize,
        rule_sets: Option<&[RetainedRuleSet]>,
        candidates: &mut HashSet<usize>,
    ) -> bool {
        use crate::invalidation::{
            invalidation_dom::{InvalidationDom, StyleChangeType},
            invalidation_set::InvalidationLists,
        };
        use crate::production_style_sheet::AttributeInvalidationKey as Key;
        let Some(change) = document.StyleState().attribute_changes.get(&index) else {
            return false;
        };
        if document.GetCompatibilityMode() == CompatibilityMode::kQuirks
            // HTML control/flat-tree attributes also drive implicit pseudo
            // state, so retain their conservative route. Ordinary metadata
            // attributes use the same compiled attribute sets as class/id;
            // an unrecognized HTML attribute cannot become a whole-subtree
            // invalidation merely because it has no special DOM semantics.
            || change.names.iter().any(|name| matches!(name.as_str(),
                "disabled"|"checked"|"selected"|"readonly"|"required"|"multiple"|"size"|"type"
                |"value"|"min"|"max"|"step"|"pattern"|"minlength"|"maxlength"|"placeholder"
                |"dir"|"lang"|"contenteditable"|"inert"|"popover"|"open"
                |"form"|"name"|"slot"|"part"|"exportparts"|"tabindex"|"autofocus"|"is"))
        {
            return false;
        }
        let keys = change
            .names
            .iter()
            .map(|s| Key::Attribute(AtomicString::from_str(s)))
            .chain(
                change
                    .classes
                    .iter()
                    .map(|s| Key::Class(AtomicString::from_str(s))),
            )
            .chain(
                change
                    .ids
                    .iter()
                    .map(|s| Key::Id(AtomicString::from_str(s))),
            )
            .collect::<Vec<_>>();
        let mut work = Vec::new();
        let Some(rule_sets) = rule_sets else {
            return false;
        };
        // `hidden` is a local presentation hint (HTMLElement::CollectStyleFor-
        // PresentationAttribute). The caller dirties this element; native style
        // difference propagates display/inherited changes. Only compiled
        // selectors such as [hidden] .child invalidate additional subjects.
        for rules in rule_sets {
            for key in &keys {
                let Some(set) = rules.Features().attribute_invalidation.get(key) else {
                    continue;
                };
                if set.cross_tree {
                    return false;
                }
                let root = if set.siblings {
                    document.ParentOrShadowHostElement(index).unwrap_or(index)
                } else {
                    index
                };
                work.push((root, set.set.clone()));
            }
        }
        if work.is_empty() {
            return true;
        }
        let mut dom = crate::persistent_invalidation::PersistentInvalidationDom::new(document);
        let mut pending =
            crate::invalidation::pending_invalidations::PendingInvalidations::default();
        for (root, set) in work {
            pending.ScheduleInvalidationSetsForNode(
                &InvalidationLists {
                    descendants: vec![set],
                    siblings: Vec::new(),
                },
                &root,
                &mut dom,
            );
        }
        let root = document.Root();
        let element = dom.document_element(&root);
        crate::invalidation::style_invalidator::StyleInvalidator::new(
            pending.GetPendingInvalidationMap(),
        )
        .Invalidate(&root, element, &mut dom);
        for (index, change) in dom.changes {
            if change == StyleChangeType::SubtreeStyleChange {
                AddSubtree(document, index, candidates);
            } else {
                candidates.insert(index);
            }
        }
        true
    }
    fn InvalidateRelations(
        &self,
        document: &Document,
        index: usize,
        dependencies: &SelectorDependencies,
        candidates: &mut HashSet<usize>,
    ) {
        let state = document.StyleState();
        let before = candidates.len();
        if dependencies.descendant && state.descendant_sensitive_nodes.contains(&index) {
            AddSubtree(document, index, candidates);
        }
        if (dependencies.sibling || dependencies.positional)
            && state.sibling_sensitive_nodes.contains(&index)
        {
            if let Some(parent) = document.Node(index).Parent() {
                AddSubtree(document, parent, candidates);
            }
        }
        if browser_tracing::enabled() && candidates.len().saturating_sub(before) > 32 {
            browser_tracing::instant_with_text(
                "style_invalidation",
                "StyleRelationInvalidation",
                document.Node(index).Name(),
                &[
                    ("node_id", document.Node(index).Id() as f64),
                    (
                        "parent_id",
                        document
                            .Node(index)
                            .Parent()
                            .map_or(0, |p| document.Node(p).Id()) as f64,
                    ),
                    ("added", candidates.len().saturating_sub(before) as f64),
                    ("descendant_rules", dependencies.descendant as u8 as f64),
                    ("sibling_rules", dependencies.sibling as u8 as f64),
                    ("positional_rules", dependencies.positional as u8 as f64),
                    (
                        "sibling_sensitive",
                        state.sibling_sensitive_nodes.contains(&index) as u8 as f64,
                    ),
                ],
            );
        }
        if dependencies.has {
            self.InvalidateHasForAttribute(document, index, candidates);
        }
    }
    // cpp: PseudoHasInvalidationTraversalContext::ForAttributeOrPseudoStateChange
    // and StyleEngine::InvalidateAncestorsOrSiblingsAffectedByHas (1580-1606,
    // 1708-1752). SelectorChecker retains the same ancestor and sibling flags.
    fn InvalidateHasForAttribute(
        &self,
        document: &Document,
        changed: usize,
        candidates: &mut HashSet<usize>,
    ) {
        use crate::selector_checker::ElementInvalidationFlag as F;
        let selectors = self.selectors.borrow();
        let flag = |index, flag| selectors.ElementHasInvalidationFlag(index, flag);
        let previous = |index| {
            let parent = document.Node(index).Parent()?;
            let siblings = document.Node(parent).Children();
            let position = siblings.iter().position(|&i| i == index)?;
            siblings[..position]
                .iter()
                .rev()
                .copied()
                .find(|&i| document.Node(i).Type() == DOMNodeType::kElement)
        };
        let mut to_parent = flag(changed, F::AncestorsOrAncestorSiblingsAffectedByHas);
        let parent = document
            .Node(changed)
            .Parent()
            .filter(|&i| document.Node(i).Type() == DOMNodeType::kElement);
        let mut shadow_host = if to_parent && parent.is_none() {
            document.ParentOrShadowHostElement(changed)
        } else {
            None
        };
        let mut current = if to_parent { parent } else { None };
        if selectors.SiblingsAffectedByHasFlags(changed) != 0 {
            if let Some(sibling) = previous(changed) {
                current = Some(sibling);
                shadow_host = None;
            }
        }
        let invalidate = |index, candidates: &mut HashSet<usize>| {
            if flag(index, F::AffectedBySubjectHas) {
                candidates.insert(index);
            }
            if flag(index, F::AffectedByNonSubjectHas) {
                self.InvalidateNonSubjectHas(document, index, candidates);
            }
        };
        if flag(changed, F::AffectedByLogicalCombinationsInHas) {
            invalidate(changed, candidates);
        }
        while let Some(index) = current {
            to_parent |= flag(index, F::AncestorsOrAncestorSiblingsAffectedByHas);
            invalidate(index, candidates);
            if selectors.SiblingsAffectedByHasFlags(index) != 0 {
                if let Some(sibling) = previous(index) {
                    current = Some(sibling);
                    continue;
                }
            }
            if !to_parent {
                return;
            }
            current = document
                .Node(index)
                .Parent()
                .filter(|&i| document.Node(i).Type() == DOMNodeType::kElement);
            if current.is_none() {
                shadow_host = document.ParentOrShadowHostElement(index);
            }
            to_parent = false;
        }
        if let Some(host) = shadow_host {
            invalidate(host, candidates);
        }
    }

    fn InvalidateHasAncestors(
        &self,
        document: &Document,
        index: usize,
        candidates: &mut HashSet<usize>,
    ) {
        // cpp: ScheduleInvalidationsForHasPseudoAffectedByInsertion/Removal.
        // DOM batches child-list facts by parent, so conservatively inspect
        // every retained child as a possible previous sibling. The selector
        // flags still gate the source traversal; unrelated parent mutations
        // must never promote the entire document to style recalc.
        use crate::selector_checker::ElementInvalidationFlag as F;
        for changed in std::iter::once(index).chain(document.Node(index).Children().iter().copied())
        {
            if document.Node(changed).Type() != DOMNodeType::kElement {
                continue;
            }
            let selectors = self.selectors.borrow();
            let subject = selectors.ElementHasInvalidationFlag(changed, F::AffectedBySubjectHas);
            let non_subject =
                selectors.ElementHasInvalidationFlag(changed, F::AffectedByNonSubjectHas);
            drop(selectors);
            if subject {
                candidates.insert(changed);
            }
            if non_subject {
                self.InvalidateNonSubjectHas(document, changed, candidates);
            }
            self.InvalidateHasForAttribute(document, changed, candidates);
        }
    }

    // cpp: StyleEngine::InvalidateElementAffectedByHas. Non-subject Has
    // schedules the compiled kPseudoHas descendant sets on the affected
    // anchor, rather than promoting the complete document to style recalc.
    fn InvalidateNonSubjectHas(
        &self,
        document: &Document,
        anchor: usize,
        candidates: &mut HashSet<usize>,
    ) {
        use crate::invalidation::{
            invalidation_dom::{InvalidationDom, StyleChangeType},
            invalidation_set::InvalidationLists,
        };
        let mut descendants = Vec::new();
        for (sheet_index, (sheet, _)) in self.active_sheets.iter().enumerate() {
            let Some(rules) = self
                .translated_rule_sets
                .get(&(sheet.Identity(), sheet.Revision()))
            else {
                browser_tracing::instant(
                    "style_invalidation",
                    "StyleHasInvalidationFallback",
                    &[
                        ("anchor_id", document.Node(anchor).Id() as f64),
                        ("sheet", sheet_index as f64),
                        ("reason", 1.0),
                    ],
                );
                AddSubtree(document, document.Root(), candidates);
                return;
            };
            let features = rules.Features();
            if features.non_descendant_has_invalidation
                || features
                    .scope_pseudo_states
                    .contains(&PseudoType::kPseudoHas)
            {
                browser_tracing::instant(
                    "style_invalidation",
                    "StyleHasInvalidationFallback",
                    &[
                        ("anchor_id", document.Node(anchor).Id() as f64),
                        ("sheet", sheet_index as f64),
                        (
                            "reason",
                            if features.non_descendant_has_invalidation {
                                2.0
                            } else {
                                3.0
                            },
                        ),
                    ],
                );
                AddSubtree(document, document.Root(), candidates);
                return;
            }
            if let Some(set) = features
                .pseudo_state_invalidation
                .get(&PseudoType::kPseudoHas)
            {
                descendants.push(set.clone());
            }
        }
        if descendants.is_empty() {
            return;
        }
        let mut dom = crate::persistent_invalidation::PersistentInvalidationDom::new(document);
        let mut pending =
            crate::invalidation::pending_invalidations::PendingInvalidations::default();
        pending.ScheduleInvalidationSetsForNode(
            &InvalidationLists {
                descendants,
                siblings: Vec::new(),
            },
            &anchor,
            &mut dom,
        );
        let root = document.Root();
        let element = dom.document_element(&root);
        crate::invalidation::style_invalidator::StyleInvalidator::new(
            pending.GetPendingInvalidationMap(),
        )
        .Invalidate(&root, element, &mut dom);
        for (index, change) in dom.changes {
            if change == StyleChangeType::SubtreeStyleChange {
                AddSubtree(document, index, candidates);
            } else {
                candidates.insert(index);
            }
        }
    }
    fn BuildRuleCandidateIndex(
        &mut self,
        document: &Document,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
    ) -> RuleCandidateIndex {
        let mut trace = browser_tracing::span("style_rules", "StyleEngine.BuildRuleCandidateIndex");
        trace.set("sheets", sheets.len() as f64);
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let mut index = RuleCandidateIndex {
            targets: std::array::from_fn(|_| RuleCandidateBuckets::default()),
            locations: Vec::new(),
            matching_may_report_diagnostics: false,
        };
        let mut order = 0;
        let mut indexed_rules = 0usize;
        let mut indexed_selectors = 0usize;
        for (sheet_index, &(sheet, _)) in sheets.iter().enumerate() {
            for (rule_index, rule) in sheet.rules.iter().enumerate() {
                order += 1;
                // locations stays aligned with document source order. Retained
                // signatures and cascade ordering use the same rule identity.
                index.locations.push((sheet_index, rule_index));
                indexed_rules += 1;
                index.matching_may_report_diagnostics |=
                    !rule.scope_conditions.is_empty() || !rule.container_conditions.is_empty();
                let variant = (quirks, !rule.scope_conditions.is_empty());
                self.EnsureCompiledSelectors(&rule.selector_text, &rule.scope_conditions, quirks);
                if let Some(selectors) = self
                    .style_selectors
                    .get(rule.selector_text.as_str())
                    .and_then(|variants| variants.get(&variant))
                {
                    indexed_selectors += selectors.len();
                    for selector in selectors {
                        index.targets[RuleCandidateIndex::Target(selector.pseudo)]
                            .Add(order, &selector.bucket);
                    }
                }
            }
        }
        trace.set("rules", order as f64);
        trace.set("indexed_rules", indexed_rules as f64);
        trace.set("indexed_selectors", indexed_selectors as f64);
        index
    }
    fn CandidateRequiresRecalc(
        &mut self,
        document: &Document,
        index: usize,
        media: &MediaValuesCachedData,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        context_unchanged: bool,
        non_rule_context_unchanged: bool,
        ordinary_complete: bool,
    ) -> bool {
        if document.ResolvedStyleFor(index).is_none_or(|resolved| {
            unsafe { resolved.native_style.Get().as_ref() }
                .is_some_and(|style| style.IsEnsuredInDisplayNone())
        }) {
            return true;
        }
        // ApplyRuleSetChanges already invalidated this candidate. If its old
        // rule signature cannot be compared in the new context, no matching
        // prepass can prove reuse: normal style resolution is required. Keep
        // the empty/empty exception below, but do not match and discard rules
        // merely to discover the context mismatch a second time.
        let Some(old) = self.matches.get(&index) else {
            return true;
        };
        if !context_unchanged && !(non_rule_context_unchanged && old.values.is_empty()) {
            return true;
        }
        if context_unchanged
            && environment_variables.is_none()
            && OrdinarySignatureElement(document, index)
            && self.matches.get(&index).is_none_or(|old| !old.complete)
            && self
                .rule_candidate_index
                .as_ref()
                .is_some_and(|index| !index.matching_may_report_diagnostics)
        {
            // Full resolution is already required without a complete retained
            // signature. Do not build and discard a prepass result; the native
            // resolver still runs.
            return true;
        }
        let current = self.MatchingRules(
            document,
            index,
            media,
            sheets,
            None,
            None,
            environment_variables,
        );
        let current = RetainedRuleSignature::new(
            current,
            ordinary_complete && OrdinarySignatureElement(document, index),
        );
        self.matches.get(&index).is_none_or(|old| {
            // Numeric rule IDs never compare across revisions. The only safe
            // exception is no matched rules in either certified flat sheet;
            // there are then neither rule values nor global sheet inputs.
            let comparable = context_unchanged
                || (non_rule_context_unchanged
                    && old.values.is_empty()
                    && current.values.is_empty());
            !comparable || !old.Equivalent(&current)
        })
    }

    fn MatchingRules(
        &mut self,
        document: &Document,
        index: usize,
        media: &MediaValuesCachedData,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
        native: Option<&HashMap<usize, Persistent<ComputedStyle>>>,
        custom: Option<&HashMap<usize, Rc<CustomProperties>>>,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Vec<(usize, u32, u32)> {
        let selector_session = self.selectors.borrow_mut().StyleMatchSession(document);
        let values = DocumentMediaValues::new(document, media);
        let evaluator = crate::media_queries::media_query_backend::ForMediaValues(&values);
        let mut matches = vec![];
        let document_scope = document.TreeScopeRoot(index) == document.Root();
        if document_scope && self.rule_candidate_index.is_none() {
            self.rule_candidate_index = Some(self.BuildRuleCandidateIndex(document, sheets));
        }
        let had_index = document_scope && self.rule_candidate_index.is_some();
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let work = if had_index {
            let candidate_index = self.rule_candidate_index.as_ref().unwrap();
            let mut orders = Vec::new();
            for target in &candidate_index.targets {
                orders.extend(target.Candidates(document, index, quirks));
            }
            orders.sort_unstable();
            orders.dedup();
            orders
                .into_iter()
                .map(|order| {
                    let (sheet, rule) = candidate_index.locations[order - 1];
                    (order, sheet, rule)
                })
                .collect::<Vec<_>>()
        } else {
            let mut work = Vec::new();
            let mut order = 0;
            for (sheet_index, &(sheet, _)) in sheets.iter().enumerate() {
                for rule_index in 0..sheet.rules.len() {
                    order += 1;
                    work.push((order, sheet_index, rule_index));
                }
            }
            work
        };
        for (order, sheet_index, rule_index) in work {
            let (sheet, origin) = sheets[sheet_index];
            if origin == CascadeOrigin::kUserAgent
                && !UserAgentSheetAppliesToDocument(
                    sheet,
                    document,
                    media,
                    document.Node(index).Namespace(),
                )
            {
                continue;
            }
            let rule = &sheet.rules[rule_index];
            if !ContainerConditionsMatch(
                document,
                index,
                media,
                &rule.container_conditions,
                &self.container_states,
                Some(
                    self.registered_container_properties
                        .as_deref()
                        .unwrap_or(&self.property_registry),
                ),
                &self.custom_properties,
                native,
                custom,
                environment_variables,
                &mut self.diagnostics,
            ) {
                continue;
            }
            let media_matches = rule.media_conditions.iter().all(|condition| {
                let query = self
                    .media_query_cache
                    .entry(condition.clone())
                    .or_insert_with(|| Rc::new(ParseMediaQuerySet(condition)))
                    .clone();
                evaluator.Eval(query.as_ref())
            });
            if media_matches {
                for (target, pseudo) in std::iter::once(None)
                    .chain(PseudoStyleKind::MATCH_TARGETS.into_iter().map(Some))
                    .enumerate()
                {
                    if !self.CompiledRuleMayMatchElement(
                        document,
                        index,
                        &rule.selector_text,
                        &rule.scope_conditions,
                        pseudo,
                    ) {
                        continue;
                    }
                    if let Some((specificity, proximity)) = self.MatchingRuleSpecificity(
                        document,
                        index,
                        &rule.selector_text,
                        pseudo,
                        &rule.scope_conditions,
                        sheet.owner_node_id,
                        Some(&selector_session),
                    ) {
                        matches.push((order, specificity | ((target as u32) << 24), proximity));
                    }
                }
            }
        }
        matches
    }
    // cpp: HTMLStyleElement::ProcessStyleSheet; StyleEngine::UpdateActiveStyleSheets.
    // CSSOM owns the parsed sheets; owner-node replacement preserves source order.
    fn SynchronizeStyleSheets(&mut self, owner: &mut DOM) {
        let mut trace = browser_tracing::span("style_rules", "StyleEngine.SynchronizeStyleSheets");
        self.inline_sheet_contents
            .retain(|_, contents| contents.strong_count() != 0);
        let document = owner.GetDocument();
        let parser_context = crate::production_style_sheet::InlineStyleSheetParserContext();
        // StyleElement::Process and StyleSheetCandidateNode changes are already
        // recorded by DOM. Discover new candidates only in inserted subtrees;
        // text/media/type edits revisit their existing style owner directly.
        let state = document.StyleState();
        let mut stack = if state.all_dirty {
            vec![document.Root()]
        } else {
            state
                .inserted_style_subtrees
                .iter()
                .copied()
                .chain(
                    state
                        .dirty_style_elements
                        .iter()
                        .filter_map(|&id| document.FindNodeById(id)),
                )
                .filter(|&index| document.IsConnectedToDocument(index))
                .collect()
        };
        let mut visited = HashSet::new();
        let mut sheets = vec![];
        while let Some(index) = stack.pop() {
            if !visited.insert(index) {
                continue;
            }
            let node = document.Node(index);
            if node.IsHTMLElement("style") && owner.NeedsStyleSheetParsing(node.Id()) {
                fn text(document: &Document, index: usize, result: &mut String) {
                    let node = document.Node(index);
                    if node.Type() == DOMNodeType::kText {
                        result.push_str(node.Data())
                    }
                    for &child in node.Children() {
                        text(document, child, result)
                    }
                }
                let mut css = String::new();
                text(document, index, &mut css);
                let valid_type = node
                    .FindAttribute("type")
                    .is_none_or(|a| a.value.is_empty() || a.value.eq_ignore_ascii_case("text/css"));
                let source = if valid_type { css.as_str() } else { "" };
                let key = InlineSheetContentsKey(source);
                let cached = self
                    .inline_sheet_contents
                    .get(&key)
                    .and_then(std::rc::Weak::upgrade)
                    .filter(|contents| {
                        contents.contents.IsCacheableForStyleElement()
                            && contents.source.Utf8() == source
                            && contents.contents.ParserContext() == &parser_context
                    });
                let mut sheet = if let Some(contents) = cached {
                    let mut projection =
                        crate::production_style_sheet_projection::ProjectStyleSheet(&contents);
                    projection.AttachParsedBacking(source, contents);
                    projection
                } else {
                    let sheet = crate::ParseCSS(source);
                    if let Some(contents) =
                        sheet.ParsedBacking::<crate::production_style_sheet::ProductionStyleSheet>()
                    {
                        self.InternInlineSheetContents(contents);
                    }
                    sheet
                };
                sheet.owner_node_id = node.Id();
                if let Some(media) = node.FindAttribute("media") {
                    if !media.value.is_empty() {
                        // The media attribute belongs to the stylesheet owner,
                        // outside its retained StyleSheetContents/RuleSet.
                        // Keep that boundary explicit so the typed collector evaluates it exactly once.
                        sheet.owner_media_conditions.push(media.value.clone());
                    }
                }
                sheets.push(sheet);
            }
            for &child in node.Children().iter().rev() {
                stack.push(child)
            }
            // cpp: style_engine.cc:290-316,648-725. Shadow-root candidate
            // nodes feed that root's own StyleSheetCollection; the root is
            // not an ordinary child of its host.
            if let Some(shadow_root) = document.ShadowRoot(index) {
                stack.push(shadow_root);
            }
        }
        trace.set("visited_nodes", visited.len() as f64);
        trace.set("parsed_sheets", sheets.len() as f64);
        for sheet in sheets {
            owner.GetDocumentMut().AppendStyleSheet(sheet)
        }
    }
    fn CollectRules(
        &mut self,
        document: &Document,
        index: usize,
        media: &MediaValuesCachedData,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
        scoped_sheets: &HashMap<usize, Vec<(&CSSStyleSheet, CascadeOrigin)>>,
        pseudo: Option<PseudoStyleKind>,
        native: Option<&HashMap<usize, Persistent<ComputedStyle>>>,
        custom: Option<&HashMap<usize, Rc<CustomProperties>>>,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<CollectedRules, DocumentStyleError> {
        let mut trace = browser_tracing::span("style_match", "StyleEngine.CollectRules");
        trace.set("node_id", document.Node(index).Id() as f64);
        trace.set("pseudo", pseudo.map_or(0, |kind| kind.Id().value()) as f64);
        trace.set("sheets", sheets.len() as f64);
        let node = document.Node(index);
        let values = DocumentMediaValues::new(document, media);
        let evaluator = crate::media_queries::media_query_backend::ForMediaValues(&values);
        let owner_media_rejected = sheets
            .iter()
            .enumerate()
            .filter_map(|(sheet_index, &(sheet, _))| {
                (!sheet.owner_media_conditions.iter().all(|condition| {
                    let query = self
                        .media_query_cache
                        .entry(condition.clone())
                        .or_insert_with(|| Rc::new(ParseMediaQuerySet(condition)))
                        .clone();
                    evaluator.Eval(query.as_ref())
                }))
                .then_some(sheet_index)
            })
            .collect::<HashSet<_>>();
        let mut cascade_result = MatchResult::default();
        let mut direct_rules = Vec::new();
        {
            let registered_container_properties = self
                .registered_container_properties
                .as_deref()
                .unwrap_or(&self.property_registry);
            let container_runtime =
                crate::resolver::production_container_runtime::ProductionContainerRuntime {
                    document,
                    media,
                    states: &self.container_states,
                    stored_custom: &self.custom_properties,
                    current_styles: native,
                    current_custom: custom,
                    environment: environment_variables,
                    registered: Some(registered_container_properties),
                };
            self.translated_rule_shadow_stats.elements_probed += 1;
            let pseudo_request = match pseudo {
                None => crate::production_rule_context::ProductionPseudoRequest::Element(),
                Some(PseudoStyleKind::BeforeMarker) => {
                    crate::production_rule_context::ProductionPseudoRequest::NestedPseudo(
                        PseudoId::kPseudoIdBefore,
                        PseudoId::kPseudoIdMarker,
                    )
                }
                Some(PseudoStyleKind::AfterMarker) => {
                    crate::production_rule_context::ProductionPseudoRequest::NestedPseudo(
                        PseudoId::kPseudoIdAfter,
                        PseudoId::kPseudoIdMarker,
                    )
                }
                Some(pseudo) => crate::production_rule_context::ProductionPseudoRequest::Pseudo(
                    pseudo.Id(),
                    None,
                ),
            };
            let defer_author_until_after_slotted = pseudo.is_some_and(|kind| {
                kind.Id()
                    == layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdPlaceholder
            });
            let mut direct = Vec::<(
                usize,
                CascadeOrigin,
                Rc<crate::rule_set::RuleSet<crate::production_style_sheet::Backend>>,
                Option<usize>,
            )>::new();
            for (sheet_index, &(sheet, origin)) in sheets.iter().enumerate() {
                let Some(expected_mode) = ExpectedSheetParserMode(document, origin) else {
                    continue;
                };
                let key = (sheet.Identity(), sheet.Revision());
                let (Some(typed), Some(rule_set)) = (
                    self.typed_sheets.get(&key),
                    self.translated_rule_sets.get(&key).cloned(),
                ) else {
                    continue;
                };
                if typed.contents.ParserContext().Mode() != expected_mode {
                    continue;
                }
                if owner_media_rejected.contains(&sheet_index) {
                    self.translated_rule_shadow_stats.skipped_owner_media += 1;
                    continue;
                }
                if origin == CascadeOrigin::kUserAgent
                    && !UserAgentSheetAppliesToDocument(sheet, document, media, node.Namespace())
                {
                    continue;
                }
                let scope_root = document.TreeScopeRoot(index);
                let implicit_scope_trigger =
                    if document.IsStyleSheetAdoptedByTreeScope(sheet.Identity(), scope_root) {
                        document.AdoptedStyleSheetImplicitScopeTrigger(sheet.Identity(), scope_root)
                    } else {
                        document
                            .FindNodeById(sheet.owner_node_id)
                            .and_then(|owner| document.ParentOrShadowHostElement(owner))
                    };
                direct.push((sheet_index, origin, rule_set, implicit_scope_trigger));
            }
            for origin in [CascadeOrigin::kUserAgent, CascadeOrigin::kAuthor] {
                if origin == CascadeOrigin::kAuthor
                    && (pseudo.is_none() || defer_author_until_after_slotted)
                {
                    // Element author rules are scheduled with the shadow
                    // special buckets below. For ordinary elements this lets
                    // :host::part share the same transfer batch. For a
                    // retained ::placeholder implementation element, Blink
                    // matches slotted scopes first and the originating host's
                    // outer author scope second (style_resolver.cc:1043-1048).
                    continue;
                }
                let inputs = direct
                    .iter()
                    .filter(|(_, candidate_origin, _, _)| *candidate_origin == origin)
                    .map(|(sheet_index, _, rule_set, implicit_scope_trigger)| {
                        crate::production_rule_matching::ProductionRuleSetInput {
                            sheet_index: *sheet_index,
                            rule_set: rule_set.clone(),
                            implicit_scope_trigger: *implicit_scope_trigger,
                        }
                    })
                    .collect::<Vec<_>>();
                let collection = crate::production_rule_matching::CollectProductionRuleMatchesInto(
                    &inputs,
                    document,
                    index,
                    &mut self.selectors.borrow_mut(),
                    origin,
                    &pseudo_request,
                    Some(&container_runtime),
                    &mut cascade_result,
                );
                ConsumeProductionRuleCollection(
                    &mut self.translated_rule_shadow_stats,
                    sheets,
                    inputs.len(),
                    (origin == CascadeOrigin::kAuthor).then(|| document.TreeScopeRoot(index)),
                    collection,
                    &mut direct_rules,
                );
            }

            let shadow_matching_index = if pseudo.is_none() {
                Some(index)
            } else if pseudo.is_some_and(|kind| {
                kind.Id() == layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdPlaceholder
            }) {
                document.UAShadowPseudoElement(index, "-webkit-input-placeholder")
            } else {
                None
            };
            if let Some(shadow_matching_index) = shadow_matching_index {
                // cpp: style_resolver.cc:946-955,969-1030. Part names are
                // retained across every OwnerShadowHost boundary. Each host's
                // exportparts map is applied before entering the next outer
                // TreeScope; absent maps terminate exposure.
                let part_rule_scopes = pseudo.is_none().then(|| {
                    crate::production_rule_matching::ProductionPartRuleScopes(document, index)
                });
                let current_part_names = part_rule_scopes
                    .as_ref()
                    .and_then(|scopes| scopes.first())
                    .map(|(_, names)| names.clone());

                let mut collect_scope = |
                    scope_root: usize,
                    pass: crate::production_rule_matching::ProductionShadowRulePass,
                | {
                    let scope_sheets = scoped_sheets
                        .get(&scope_root)
                        .map_or(&[][..], Vec::as_slice);
                    let mut owners = Vec::<(
                        usize,
                        Rc<crate::rule_set::RuleSet<crate::production_style_sheet::Backend>>,
                        Option<usize>,
                    )>::new();
                    for (sheet_index, &(sheet, origin)) in scope_sheets.iter().enumerate() {
                        if origin != CascadeOrigin::kAuthor
                            || !sheet.owner_media_conditions.iter().all(|condition| {
                                let query = self
                                    .media_query_cache
                                    .entry(condition.clone())
                                    .or_insert_with(|| Rc::new(ParseMediaQuerySet(condition)))
                                    .clone();
                                evaluator.Eval(query.as_ref())
                            })
                        {
                            continue;
                        }
                        let key = (sheet.Identity(), sheet.Revision());
                        let (Some(typed), Some(rule_set)) = (
                            self.typed_sheets.get(&key),
                            self.translated_rule_sets.get(&key).cloned(),
                        ) else {
                            continue;
                        };
                        if typed.contents.ParserContext().Mode() != DocumentParserMode(document) {
                            continue;
                        }
                        let implicit_scope_trigger = if document
                            .IsStyleSheetAdoptedByTreeScope(sheet.Identity(), scope_root)
                        {
                            document.AdoptedStyleSheetImplicitScopeTrigger(
                                sheet.Identity(),
                                scope_root,
                            )
                        } else {
                            document
                                .FindNodeById(sheet.owner_node_id)
                                .and_then(|owner| document.ParentOrShadowHostElement(owner))
                        };
                        owners.push((
                            sheet_index,
                            rule_set,
                            implicit_scope_trigger,
                        ));
                    }
                    let inputs = owners
                        .iter()
                        .map(
                            |(sheet_index, rule_set, implicit_scope_trigger)| {
                                crate::production_rule_matching::ProductionRuleSetInput {
                                    sheet_index: *sheet_index,
                                    rule_set: rule_set.clone(),
                                    implicit_scope_trigger: *implicit_scope_trigger,
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let collection =
                        crate::production_rule_matching::CollectProductionShadowRuleMatchesInto(
                            &inputs,
                            document,
                            shadow_matching_index,
                            &mut self.selectors.borrow_mut(),
                            scope_root,
                            &pass,
                            Some(&container_runtime),
                            &mut cascade_result,
                        );
                    ConsumeProductionRuleCollection(
                        &mut self.translated_rule_shadow_stats,
                        scope_sheets,
                        inputs.len(),
                        Some(if matches!(pass, crate::production_rule_matching::ProductionShadowRulePass::ElementScope(_)) {
                            document.TreeScopeRoot(shadow_matching_index)
                        } else {
                            scope_root
                        }),
                        collection,
                        &mut direct_rules,
                    );
                };

                // cpp: style_resolver.cc:715-802,862-879,929-1031.
                if pseudo.is_none() {
                    if let Some(shadow_root) = document.ShadowRoot(index) {
                        collect_scope(
                            shadow_root,
                            crate::production_rule_matching::ProductionShadowRulePass::Host,
                        );
                    }
                }

                // cpp: style_resolver.cc:737-802. For a retained UA shadow
                // pseudo (notably ::placeholder), resolver selection starts
                // from its originating host while selector matching remains
                // on the internal pseudo element.
                let mut slot_scopes = Vec::new();
                let mut seen_slots = HashSet::new();
                let mut slot = document.AssignedSlot(index);
                while let Some(current) = slot {
                    if !seen_slots.insert(current) {
                        break;
                    }
                    slot_scopes.push(document.TreeScopeRoot(current));
                    slot = document.AssignedSlot(current);
                }
                for scope_root in slot_scopes.into_iter().rev() {
                    collect_scope(
                        scope_root,
                        crate::production_rule_matching::ProductionShadowRulePass::Slotted,
                    );
                }

                if pseudo.is_none() {
                    let element_scope = document
                        .TreeScopeRoot(document.CorrespondingSVGElement(index).unwrap_or(index));
                    collect_scope(
                        element_scope,
                        crate::production_rule_matching::ProductionShadowRulePass::ElementScope(
                            document
                                .IsInShadowTree(index)
                                .then(|| current_part_names.clone())
                                .flatten(),
                        ),
                    );

                    for (scope_root, names) in part_rule_scopes.into_iter().flatten() {
                        collect_scope(
                            scope_root,
                            crate::production_rule_matching::ProductionShadowRulePass::Part(names),
                        );
                    }
                }

                if defer_author_until_after_slotted {
                    let inputs = direct
                        .iter()
                        .filter(|(_, origin, _, _)| *origin == CascadeOrigin::kAuthor)
                        .map(|(sheet_index, _, rule_set, implicit_scope_trigger)| {
                            crate::production_rule_matching::ProductionRuleSetInput {
                                sheet_index: *sheet_index,
                                rule_set: rule_set.clone(),
                                implicit_scope_trigger: *implicit_scope_trigger,
                            }
                        })
                        .collect::<Vec<_>>();
                    let collection =
                        crate::production_rule_matching::CollectProductionRuleMatchesInto(
                            &inputs,
                            document,
                            index,
                            &mut self.selectors.borrow_mut(),
                            CascadeOrigin::kAuthor,
                            &pseudo_request,
                            Some(&container_runtime),
                            &mut cascade_result,
                        );
                    ConsumeProductionRuleCollection(
                        &mut self.translated_rule_shadow_stats,
                        sheets,
                        inputs.len(),
                        Some(document.TreeScopeRoot(index)),
                        collection,
                        &mut direct_rules,
                    );
                }
            }
        }
        debug_assert!(
            pseudo.is_some(),
            "CollectRules is the generated-pseudo inventory path"
        );
        if direct_rules.len() > u16::MAX as usize {
            return Err(DocumentStyleError::CascadeIndexOverflow);
        }
        debug_assert_eq!(
            direct_rules.len(),
            cascade_result.GetMatchedProperties().len()
        );
        Ok(CollectedRules {
            rules: direct_rules,
        })
    }
    fn CompiledRuleHasTarget(
        &self,
        document: &Document,
        text: &str,
        scopes: &[cssom::CSSStyleScope],
        pseudo: Option<PseudoStyleKind>,
    ) -> bool {
        let variant = (
            document.GetCompatibilityMode() == CompatibilityMode::kQuirks,
            !scopes.is_empty(),
        );
        self.style_selectors
            .get(text)
            .and_then(|variants| variants.get(&variant))
            .is_some_and(|selectors| selectors.iter().any(|selector| selector.pseudo == pseudo))
    }

    fn CompiledRuleMayMatchElement(
        &self,
        document: &Document,
        index: usize,
        text: &str,
        scopes: &[cssom::CSSStyleScope],
        pseudo: Option<PseudoStyleKind>,
    ) -> bool {
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let variant = (quirks, !scopes.is_empty());
        self.style_selectors
            .get(text)
            .and_then(|variants| variants.get(&variant))
            .is_none_or(|selectors| {
                selectors
                    .iter()
                    .filter(|selector| selector.pseudo == pseudo)
                    .any(|selector| selector.bucket.Matches(document, index, quirks))
            })
    }
    fn MatchingRuleSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        text: &str,
        pseudo: Option<PseudoStyleKind>,
        scopes: &[cssom::CSSStyleScope],
        owner_node_id: u64,
        session: Option<&PersistentSelectorMatchSession<'_>>,
    ) -> Option<(u32, u32)> {
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let variant = (quirks, !scopes.is_empty());
        self.EnsureCompiledSelectors(text, scopes, quirks);
        let mut specificity = None;
        for selector in self
            .style_selectors
            .get(text)
            .unwrap()
            .get(&variant)
            .unwrap()
            .iter()
            .filter(|s| s.pseudo == pseudo)
        {
            let matched = if scopes.is_empty() {
                Ok(session
                    .map(|session| {
                        session.MatchingCompiledSpecificity(index, &selector.compiled_subject)
                    })
                    .unwrap_or_else(|| {
                        self.selectors.borrow_mut().MatchingCompiledSpecificity(
                            document,
                            index,
                            &selector.compiled_subject,
                        )
                    })
                    .map(|_| u32::MAX))
            } else {
                self.selectors
                    .borrow_mut()
                    .MatchingScopedSpecificity(
                        document,
                        index,
                        &selector.compiled_subject,
                        scopes,
                        owner_node_id,
                    )
                    .map(|m| m.map(|m| m.proximity))
            };
            match matched {
                Ok(Some(proximity)) => {
                    let candidate = (selector.specificity, proximity);
                    if specificity.is_none_or(|old: (u32, u32)| {
                        (candidate.0, std::cmp::Reverse(candidate.1))
                            > (old.0, std::cmp::Reverse(old.1))
                    }) {
                        specificity = Some(candidate);
                    }
                }
                // This is an invalidation signature probe. Unsupported selector
                // state already keeps the retained signature incomplete; it is
                // not a second style-resolution diagnostic channel.
                Err(_) => {}
                _ => {}
            }
        }
        specificity
    }
    // cpp: RuleSet::AddRuleToRuleSet. Selector compilation and bucketing are
    // independent of whether the rule's media/container condition matches the
    // current frame. Otherwise inactive responsive rules become universal
    // candidates and are reconsidered for every element on large documents.
    fn EnsureCompiledSelectors(
        &mut self,
        text: &str,
        scopes: &[cssom::CSSStyleScope],
        quirks: bool,
    ) {
        let variant = (quirks, !scopes.is_empty());
        if self
            .style_selectors
            .get(text)
            .is_some_and(|variants| variants.contains_key(&variant))
        {
            return;
        }
        let selectors =
            CSSSelectorList::AdoptSelectorVector(CSSSelectorParser::ParseSelectorWithOptions(
                &foundation::String::from(text),
                &SelectorParserContext { html: true, quirks },
                if scopes.is_empty() {
                    crate::parser::css_nesting_type::CSSNestingType::kNone
                } else {
                    crate::parser::css_nesting_type::CSSNestingType::kScope
                },
                &Default::default(),
            ));
        let mut compiled = vec![];
        for complex in selectors.ComplexSelectors() {
            let mut simple: Vec<_> = complex.SimpleSelectors().cloned().collect();
            let positions: Vec<_> = simple
                .iter()
                .enumerate()
                .filter(|(_, selector)| selector.Match() == MatchType::kPseudoElement)
                .map(|(index, _)| index)
                .collect();
            let kind = if positions.is_empty() {
                None
            } else {
                let compound_end = simple
                    .iter()
                    .position(|selector| selector.Relation() != RelationType::kSubSelector)
                    .unwrap_or(simple.len() - 1);
                let kind = match positions.as_slice() {
                    [position] if *position == compound_end => {
                        match simple[*position].GetPseudoType() {
                            PseudoType::kPseudoBefore => Some(PseudoStyleKind::Before),
                            PseudoType::kPseudoAfter => Some(PseudoStyleKind::After),
                            PseudoType::kPseudoFirstLetter => Some(PseudoStyleKind::FirstLetter),
                            PseudoType::kPseudoPlaceholder => Some(PseudoStyleKind::Placeholder),
                            PseudoType::kPseudoMarker => Some(PseudoStyleKind::Marker),
                            _ => None,
                        }
                    }
                    [originating, marker]
                        if *marker == compound_end
                            && *marker == *originating + 1
                            && simple[*marker].GetPseudoType() == PseudoType::kPseudoMarker =>
                    {
                        match simple[*originating].GetPseudoType() {
                            PseudoType::kPseudoBefore => Some(PseudoStyleKind::BeforeMarker),
                            PseudoType::kPseudoAfter => Some(PseudoStyleKind::AfterMarker),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if kind.is_none() {
                    continue;
                }
                // Remove from the tail inward. Transferring each removed
                // selector's relation preserves the combinator that connected
                // the original subject compound to its left-hand complex.
                for &position in positions.iter().rev() {
                    let removed = simple.remove(position);
                    if position > 0 {
                        simple[position - 1].SetRelation(removed.Relation());
                    }
                }
                kind
            };
            if simple.is_empty() {
                simple = CSSSelectorParser::ParseSelector(
                    &foundation::String::from("*"),
                    &SelectorParserContext { html: true, quirks },
                );
            }
            for selector in &mut simple {
                selector.SetLastInComplexSelector(false);
                selector.SetLastInSelectorList(false);
            }
            simple.last_mut().unwrap().SetLastInComplexSelector(true);
            let compiled_subject = CSSSelectorList::AdoptSelectorVector(simple);
            if crate::production_rule_matching::ProductionUnsupportedParsedFeature(
                &compiled_subject.CopySelectors(),
            )
            .is_some()
            {
                continue;
            }
            let bucket = SelectorBucketForSubject(compiled_subject.as_ref());
            compiled.push(StyleSelector {
                compiled_subject,
                pseudo: kind,
                specificity: complex.Specificity(),
                bucket,
            });
        }
        self.style_selectors
            .entry(text.to_owned())
            .or_default()
            .insert(variant, compiled);
    }
    fn ResolvePseudoStyle(
        &mut self,
        document: &Document,
        index: usize,
        kind: PseudoStyleKind,
        originating: &ResolvedNodeStyle,
        originating_custom: &Rc<CustomProperties>,
        initial: &ComputedStyle,
        root: f32,
        media: &MediaValuesCachedData,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        rules: &CollectedRules,
        function_registry: Option<&crate::production_style_sheet::ProductionFunctionRegistry>,
        has_layout_context: bool,
        has_nested_marker: bool,
        production_rule_sets: Option<&ProductionResolverRuleSets>,
        rule_collections: &Rc<
            crate::resolver::production_style_resolver::ProductionRuleCollectionCache,
        >,
        resolver_batch: &Rc<crate::resolver::production_style_resolver::ProductionResolverBatch>,
        current_styles: &HashMap<usize, Persistent<ComputedStyle>>,
        current_custom: &HashMap<usize, Rc<CustomProperties>>,
        root_style: Option<&ComputedStyle>,
    ) -> Result<Option<ResolvedPseudoOutput>, DocumentStyleError> {
        let mut trace = browser_tracing::span("style_resolve", "StyleEngine.ResolvePseudoStyle");
        trace.set("node_id", document.Node(index).Id() as f64);
        trace.set("pseudo", kind.Id().value() as f64);
        trace.set("rules", rules.rules.len() as f64);
        let node = document.Node(index);
        let parent = unsafe { &*originating.native_style.Get() };
        if !has_layout_context
            || (kind != PseudoStyleKind::Placeholder && !parent.CanGeneratePseudoElement(kind.Id()))
        {
            return Ok(None);
        }
        if kind == PseudoStyleKind::Placeholder
            && !(node.IsHTMLElement("input") || node.IsHTMLElement("textarea"))
        {
            return Ok(None);
        }
        // cpp: PseudoElement::CanGeneratePseudoElement(kPseudoIdMarker) and
        // AttachLayoutTree. A marker request is matched against its
        // originating element, but it only owns a layout object when that
        // element's final computed display is a list-item display.
        if kind == PseudoStyleKind::Marker && !parent.IsDisplayListItem() {
            return Ok(None);
        }
        if rules.rules.is_empty() {
            return Ok(None);
        }
        // Replaced elements have no ::before/::after child layout tree.
        if matches!(kind, PseudoStyleKind::Before | PseudoStyleKind::After)
            && [
                "input", "textarea", "img", "iframe", "video", "audio", "canvas", "select",
            ]
            .iter()
            .any(|name| node.IsHTMLElement(name))
        {
            self.diagnostics.push(DocumentStyleError::Unsupported(
                "generated pseudo on replaced/form element",
            ));
            return Ok(None);
        }
        // cpp: StyleResolver::StyleForPseudoElement. Generated pseudos use the
        // same RuleSet collector, cascade and matched-properties cache as the
        // originating element.
        if !rules.rules.iter().all(|rule| rule.transferred.is_some()) {
            return Err(DocumentStyleError::Unsupported(
                "production-pseudo-rule-owner",
            ));
        }
        let rule_sets = production_rule_sets.ok_or(DocumentStyleError::Unsupported(
            "production-pseudo-rule-set-bundle",
        ))?;
        self.translated_rule_shadow_stats
            .translated_style_resolver_attempts += 1;
        self.translated_rule_shadow_stats
            .translated_style_cascade_pseudo_attempts += 1;
        let resolver_environment = CascadeEnvironment::new(environment_variables);
        let resolver_attribute_source = PersistentAttributeSource::new(document, index, true);
        let resolver_attribute_state = CascadeAttributeState::new();
        let resolver_attributes =
            AttributeResolver::new(&resolver_attribute_source, &resolver_attribute_state, false);
        let resolver_substitution = SubstitutionContext {
            environment: Some(&resolver_environment),
            attributes: Some(&resolver_attributes),
        };
        let resolver_images = DocumentURLImageResolver {
            document,
            originating_index: index,
        };
        let parent_handle = originating.native_style.clone();
        let root_handle = root_style
            .map(|style| Persistent::from_ptr(style as *const ComputedStyle as *mut ComputedStyle));
        let registered_container_properties = self
            .registered_container_properties
            .as_deref()
            .unwrap_or(&self.property_registry);
        let container_runtime =
            crate::resolver::production_container_runtime::ProductionContainerRuntime {
                document,
                media,
                states: &self.container_states,
                stored_custom: &self.custom_properties,
                current_styles: Some(current_styles),
                current_custom: Some(current_custom),
                environment: environment_variables,
                registered: Some(registered_container_properties),
            };
        let mut collaborators =
            crate::resolver::production_style_resolver::ProductionBaseStyleCollaborators::ordinary(
                root,
            );
        collaborators.rule_collections = Some(rule_collections.clone());
        collaborators.resolver_batch = Some(resolver_batch.clone());
        collaborators.declarations = Some(self.declarations.clone());
        collaborators.html_quirks_user_agent_rules = rule_sets.html_quirks_user_agent.clone();
        collaborators.forced_colors_user_agent_rules = rule_sets.forced_colors_user_agent.clone();
        collaborators.pseudo_element_user_agent_rules = rule_sets.marker_user_agent.clone();
        collaborators.fullscreen_user_agent_rules = rule_sets.fullscreen_user_agent.clone();
        collaborators.root_style = root_handle;
        collaborators.color_scheme_settings = self.color_scheme_settings;
        collaborators.text_link_text_color = self.resolver_dependencies.text_link_text_color;
        collaborators.pseudo_id = kind.Id();
        collaborators.originating_pseudo_id = match kind {
            PseudoStyleKind::BeforeMarker => PseudoId::kPseudoIdBefore,
            PseudoStyleKind::AfterMarker => PseudoId::kPseudoIdAfter,
            _ => PseudoId::kPseudoIdNone,
        };
        collaborators.originating_style = Some(parent_handle.clone());
        collaborators.subject_node = Some(index);
        collaborators.container_runtime = Some(&container_runtime);
        collaborators.parent_custom_properties = Some(originating_custom.as_ref());
        collaborators.substitution_context = resolver_substitution;
        collaborators.image_resolver = Some(&resolver_images);
        collaborators.property_registry = Some(&self.property_registry);
        collaborators.function_registry = function_registry;
        collaborators.font_metrics = self.font_metrics.as_deref();
        collaborators.matched_properties_cache = Some(self.matched_properties_cache.clone());
        collaborators.selectors = Some(self.selectors.clone());
        collaborators.interaction = self.selectors.borrow().interaction;
        match crate::resolver::production_style_resolver::ApplyBaseStyleForElement(
            crate::resolver::production_style_resolver::ProductionBaseStyleInputs {
                matching:
                    crate::resolver::production_style_resolver::ProductionRuleMatchingInputs {
                        document,
                        element_index: index,
                        media,
                        html_user_agent_rules: rule_sets.html_user_agent.clone(),
                        svg_user_agent_rules: rule_sets.svg_user_agent.clone(),
                        mathml_user_agent_rules: rule_sets.mathml_user_agent.clone(),
                        author_rule_sets: rule_sets.author.clone(),
                    },
                parent: Some(parent_handle),
                collaborators,
            },
        ) {
            Ok(output) => {
                let metrics = &output.metrics;
                self.resolver_dependencies.Merge(metrics);
                self.translated_rule_shadow_stats.translated_collector_calls +=
                    metrics.collector_calls;
                self.translated_rule_shadow_stats.candidates_considered +=
                    metrics.candidates_considered;
                self.translated_rule_shadow_stats.selector_matches += metrics.selector_matches;
                self.translated_rule_shadow_stats
                    .translated_property_sets_transferred += metrics.transferred_property_sets;
                self.translated_rule_shadow_stats
                    .translated_property_sets_consumed += metrics.transferred_property_sets;
                self.translated_rule_shadow_stats.typed_declarations_used +=
                    metrics.transferred_property_sets;
                self.translated_rule_shadow_stats
                    .translated_style_resolver_applied += 1;
                self.translated_rule_shadow_stats
                    .translated_style_cascade_pseudo_applied += 1;
                let mut builder = ComputedStyleBuilder::from_style(unsafe { &*output.style.Get() });
                resolver_environment.ApplyState(&mut builder);
                resolver_attribute_state.ApplyState(&mut builder);
                if has_nested_marker {
                    builder.SetPseudoElementStyles(
                        PseudoIdFlags::from_list(&[PseudoId::kPseudoIdMarker]).Bits(),
                    );
                }
                if matches!(
                    kind,
                    PseudoStyleKind::Marker
                        | PseudoStyleKind::BeforeMarker
                        | PseudoStyleKind::AfterMarker
                ) && parent.Display() != EDisplay::kInlineListItem
                    && parent.ListStylePosition() == foundation::EListStylePosition::kOutside
                {
                    // cpp: StyleAdjuster::AdjustStyleForMarker.
                    builder.SetDisplay(EDisplay::kInlineBlock);
                    builder.SetWhiteSpace(EWhiteSpace::kPre);
                }
                self.ResolveFontFeatureValues(&mut builder);
                let mut style = Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
                let native = unsafe { &*style.Get() };
                if !has_nested_marker && !kind.IsNestedMarker() {
                    if let Some(old) = document.ResolvedStyleFor(index) {
                        let cached = unsafe { &*old.native_style.Get() }
                            .GetCachedPseudoElementStyleWithoutArgument(kind.Id());
                        if unsafe { cached.as_ref() }.is_some_and(|old| {
                            old == native && old.HasAttrFunction() == native.HasAttrFunction()
                        }) {
                            style = Persistent::from_ptr(cached as *mut ComputedStyle);
                        }
                    }
                }
                let native = unsafe { &*style.Get() };
                parent.AddCachedPseudoElementStyle(
                    style.Get(),
                    kind.Id(),
                    &foundation::g_null_atom,
                );
                if native.Display() == EDisplay::kNone
                    || (matches!(kind, PseudoStyleKind::Before | PseudoStyleKind::After)
                        && native.ContentPreventsBoxGeneration())
                    || (matches!(
                        kind,
                        PseudoStyleKind::Marker
                            | PseudoStyleKind::BeforeMarker
                            | PseudoStyleKind::AfterMarker
                    ) && native.ContentPreventsBoxGeneration())
                {
                    return Ok(None);
                }
                return Ok(Some(ResolvedPseudoOutput {
                    pseudo: Self::ProjectPseudoElement(style)?,
                    custom_properties: output.custom_properties,
                }));
            }
            Err(failures) => {
                self.translated_rule_shadow_stats
                    .translated_style_resolver_failures += 1;
                self.translated_rule_shadow_stats
                    .translated_style_cascade_pseudo_failures += 1;
                if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                    eprintln!(
                                "translated pseudo StyleResolver failure node={index} pseudo={kind:?}: {failures:?}"
                            );
                }
                return Err(DocumentStyleError::Unsupported(
                    "production-pseudo-style-resolver",
                ));
            }
        }
        unreachable!("pseudo resolver returns from both result branches")
    }

    fn ResolveNestedMarkerStyle(
        &mut self,
        document: &Document,
        index: usize,
        kind: PseudoStyleKind,
        originating: &PseudoElement,
        originating_custom: &Rc<CustomProperties>,
        _old_marker: Option<&PseudoElement>,
        initial: &ComputedStyle,
        root: f32,
        media: &MediaValuesCachedData,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        rules: &CollectedRules,
        function_registry: Option<&crate::production_style_sheet::ProductionFunctionRegistry>,
        has_layout_context: bool,
        production_rule_sets: &ProductionResolverRuleSets,
        rule_collections: &Rc<
            crate::resolver::production_style_resolver::ProductionRuleCollectionCache,
        >,
        resolver_batch: &Rc<crate::resolver::production_style_resolver::ProductionResolverBatch>,
        current_styles: &HashMap<usize, Persistent<ComputedStyle>>,
        current_custom: &HashMap<usize, Rc<CustomProperties>>,
        root_style: Option<&ComputedStyle>,
    ) -> Result<Option<PseudoElement>, DocumentStyleError> {
        let mut trace =
            browser_tracing::span("style_resolve", "StyleEngine.ResolveNestedMarkerStyle");
        trace.set("node_id", document.Node(index).Id() as f64);
        trace.set("rules", rules.rules.len() as f64);
        let parent = unsafe { &*originating.native_style.Get() };
        if !has_layout_context
            || rules.rules.is_empty()
            || !parent.CanGeneratePseudoElement(PseudoId::kPseudoIdMarker)
        {
            return Ok(None);
        }
        let mut originating_style = ResolvedNodeStyle::default();
        originating_style.SetNativeStyle(originating.native_style.clone());
        Ok(self
            .ResolvePseudoStyle(
                document,
                index,
                kind,
                &originating_style,
                originating_custom,
                initial,
                root,
                media,
                environment_variables,
                rules,
                function_registry,
                true,
                false,
                Some(production_rule_sets),
                rule_collections,
                resolver_batch,
                current_styles,
                current_custom,
                root_style,
            )?
            .map(|output| output.pseudo))
    }

    fn ProjectPseudoElement(
        style: Persistent<ComputedStyle>,
    ) -> Result<PseudoElement, DocumentStyleError> {
        let native = unsafe { &*style.Get() };
        let mut projection = ResolvedNodeStyle::default();
        ProjectForLayoutBoundary(native, &mut projection)?;
        // DOM/Layout consumes a projection of native ContentData; the native
        // content chain remains the computed value and cache-owned source.
        let mut text = String::new();
        let mut projected_content = Vec::new();
        let mut content = native.GetContentData();
        while let Some(data) = content {
            let data_ref = unsafe { &*data };
            if data_ref.IsText() {
                let value = unsafe { &*(data as *const TextContentData) }
                    .GetText()
                    .Utf8();
                text.push_str(&value);
                projected_content.push(PseudoContent::Text(value));
            } else if data_ref.IsImage() {
                let image = unsafe { &*(data as *const ImageContentData) }.GetImage();
                if let Some(image) = unsafe { image.as_ref() }.and_then(FetchedImage) {
                    let respect_orientation = native.ImageOrientation();
                    let (natural_width, natural_height) = image
                        .NaturalSize(respect_orientation)
                        .map_or((None, None), |(width, height)| (Some(width), Some(height)));
                    projected_content.push(PseudoContent::Image {
                        resource_id: image.ResourceId(),
                        shader: None,
                        natural_width,
                        natural_height,
                        resolution_scale: image.ResolutionScale(),
                    });
                } else if let Some(shader) = unsafe { image.as_ref() }.and_then(|image| {
                    layoutng::internal::boundary::native_paint_style::ExportGeneratedImageShader(
                        native, image,
                    )
                }) {
                    projected_content.push(PseudoContent::Image {
                        resource_id: 0,
                        shader: Some(shader),
                        natural_width: None,
                        natural_height: None,
                        resolution_scale: 1.0,
                    });
                }
            }
            content = data_ref.Next();
        }
        let mut pseudo = PseudoElement {
            style: projection.style,
            text,
            content: projected_content,
            display_contents: native.Display() == EDisplay::kContents,
            ..Default::default()
        };
        pseudo.SetNativeStyle(style);
        Ok(pseudo)
    }
}

// SVGResources::GetClient + LayoutSVGResourcePaintServer::ApplyShader own this
// dependency in Chromium.  Rechrom's StyleEngine freezes the same local
// resource result into its existing typed paint payload.  Scan URI consumers
// after the cascade so a changed <stop> also refreshes unchanged referencing
// shapes, matching SVGGradientElement::InvalidateDependentGradients.
fn ResolveSVGPaintServers(
    document: &Document,
    resident_svg_styles: &HashSet<usize>,
    pending: &mut Vec<(usize, ResolvedNodeStyle)>,
    stats: &mut StyleUpdateStats,
    impact: &mut StyleUpdateImpact,
    pending_reattach: &mut HashSet<u64>,
) {
    let mut trace = browser_tracing::span("style_resolve", "StyleEngine.ResolveSVGPaintServers");
    let snapshot = pending
        .iter()
        .map(|(node, style)| (*node, style))
        .collect::<crate::svg_paint_server::PendingStyles<'_>>();
    let mut updates = Vec::new();
    let mut svg_nodes = 0;
    let mut clients = 0;
    // SVGResources clients own live ComputedStyles. Pending results can
    // introduce new clients; resident results supply unchanged dependents.
    // Removed nodes have already released their style through DOM::Remove.
    for node in snapshot.keys().copied().chain(
        resident_svg_styles
            .iter()
            .copied()
            .filter(|node| !snapshot.contains_key(node)),
    ) {
        // cpp: LayoutSVGShape/Inline/Text/TransformableContainer::StyleDidChange
        // -> SVGResources::UpdatePaints. HTML's inherited fill/stroke remains
        // canonical CSS, but it has no SVG paint-server client of its own.
        if document.Node(node).Namespace() != DOMNamespace::kSVG {
            continue;
        }
        svg_nodes += 1;
        let Some(current) = snapshot
            .get(&node)
            .copied()
            .or_else(|| document.ResolvedStyleFor(node))
        else {
            continue;
        };
        let fill_reference = current.style.paint.svg_fill_reference.as_deref();
        let stroke_reference = current.style.paint.svg_stroke_reference.as_deref();
        let native = unsafe { current.native_style.Get().as_ref() };
        let marker_url =
            |resource: *mut layoutng_style::style::style_svg_resource::StyleSVGResource| {
                unsafe { resource.as_ref() }
                    .map(|resource| resource.Url().Utf8())
                    .filter(|url| !url.is_empty())
            };
        let marker_start = native.and_then(|style| marker_url(style.MarkerStartResource()));
        let marker_mid = native.and_then(|style| marker_url(style.MarkerMidResource()));
        let marker_end = native.and_then(|style| marker_url(style.MarkerEndResource()));
        let mask_definition = document.Node(node).IsElement("mask", DOMNamespace::kSVG);
        if !mask_definition
            && fill_reference.is_none()
            && stroke_reference.is_none()
            && marker_start.is_none()
            && marker_mid.is_none()
            && marker_end.is_none()
        {
            continue;
        }
        clients += 1;
        let fill_opacity = native.map_or(1.0, |style| style.FillOpacity());
        let stroke_opacity = native.map_or(1.0, |style| style.StrokeOpacity());
        let fill = fill_reference.and_then(|reference| {
            crate::svg_paint_server::Resolve(document, node, &snapshot, reference, fill_opacity)
        });
        let stroke = stroke_reference.and_then(|reference| {
            crate::svg_paint_server::Resolve(document, node, &snapshot, reference, stroke_opacity)
        });
        let mut context_paint = current.style.paint.clone();
        context_paint.svg_fill_server = fill.clone();
        context_paint.svg_stroke_server = stroke.clone();
        let markers = crate::svg_paint_server::ResolveMarkers(
            document,
            &snapshot,
            node,
            &context_paint,
            marker_start.as_deref(),
            marker_mid.as_deref(),
            marker_end.as_deref(),
            current.style.paint.svg_stroke_width,
        );
        let mask_graph = mask_definition
            .then(|| crate::svg_paint_server::ResolveMaskDefinition(document, &snapshot, node));
        if current.style.paint.svg_mask_resource == mask_graph
            && current.style.paint.svg_fill_server == fill
            && current.style.paint.svg_stroke_server == stroke
            && current.style.paint.svg_markers == markers
        {
            continue;
        }
        let mut updated = current.clone();
        updated.style.paint.svg_mask_resource = mask_graph;
        updated.style.paint.svg_fill_server = fill;
        updated.style.paint.svg_stroke_server = stroke;
        updated.style.paint.svg_markers = markers;
        let (change, reattach) =
            StyleImpact(document, node, document.ResolvedStyleFor(node), &updated);
        impact.Merge(change);
        if reattach {
            pending_reattach.insert(document.Node(node).Id());
        }
        updates.push((node, updated));
    }
    trace.set("arena_nodes", document.NodeCount() as f64);
    trace.set("svg_nodes", svg_nodes as f64);
    trace.set("clients", clients as f64);
    trace.set("updated", updates.len() as f64);
    drop(snapshot);
    if updates.is_empty() {
        return;
    }
    let mut positions = pending
        .iter()
        .enumerate()
        .map(|(position, (node, _))| (*node, position))
        .collect::<HashMap<_, _>>();
    for (node, updated) in updates {
        if let Some(&position) = positions.get(&node) {
            pending[position].1 = updated;
        } else {
            positions.insert(node, pending.len());
            pending.push((node, updated));
            stats.changed_nodes += 1;
        }
    }
}

fn ApplyTextControlInnerEditorCustomStyle(
    document: &Document,
    element: usize,
    builder: &mut ComputedStyleBuilder,
) {
    let Some(host) = document.TextControlInnerEditorHost(element) else {
        return;
    };
    let host_node = document.Node(host);
    let read_only = host_node.FindAttribute("disabled").is_some()
        || host_node.FindAttribute("readonly").is_some();
    builder.SetUserSelect(EUserSelect::kText);
    builder.SetUserModify(if read_only {
        EUserModify::kReadOnly
    } else {
        EUserModify::kReadWritePlaintextOnly
    });
    builder.SetDisplay(EDisplay::kBlock);
    builder.SetShouldIgnoreOverflowPropertyForInlineBlockBaseline();
    if !host_node.IsElement("textarea", DOMNamespace::kHTML) {
        builder.SetHasLineIfEmpty(true);
        builder.SetWhiteSpace(EWhiteSpace::kPre);
        builder.SetOverflowWrap(EOverflowWrap::kNormal);
        builder.SetOverflowX(EOverflow::kScroll);
        builder.SetOverflowY(EOverflow::kScroll);
        builder.SetScrollbarWidth(EScrollbarWidth::kNone);
        builder.SetDisplay(EDisplay::kFlowRoot);
    }
    builder.SetOpacity(if document.TextControlInnerEditorIsVisible(element) {
        1.0
    } else {
        0.0
    });
}

// cpp: layout_list_marker.cc:74-100 (LayoutListMarker::ImageBulletSize).
// Style owns image-set selection and generated-image sizing. Export the
// selected immutable image identity plus author-space intrinsic dimensions;
// LayoutReplaced reapplies effective zoom at its native boundary.
fn ProjectListMarkerImage(
    native: &ComputedStyle,
) -> Option<layoutng::internal::layout_input::ListMarkerImage> {
    let image = unsafe { native.ListStyleImage().Get().as_ref() }?;
    if image.ErrorOccurred() {
        return None;
    }
    let (resource_id, shader, resolution_scale) = if let Some(fetched) = FetchedImage(image) {
        if fetched.ResourceId() == 0 {
            // Until the resource supplies an intrinsic size and paint id, use
            // list-style-type. Resource invalidation rebuilds this projection.
            return None;
        }
        (fetched.ResourceId(), None, fetched.ResolutionScale())
    } else {
        let shader = layoutng::internal::boundary::native_paint_style::ExportGeneratedImageShader(
            native, image,
        )?;
        (0, Some(shader), 1.0)
    };

    let zoom = native.EffectiveZoom();
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }
    let side = unsafe { native.GetFont().as_ref() }
        .and_then(|font| unsafe { font.PrimaryFont().as_ref() })
        .map_or_else(
            || native.GetFontDescription().ComputedSize() / 2.0,
            |primary| primary.GetFontMetrics().Ascent() as f32 / 2.0,
        );
    let default_size = foundation::gfx::SizeF::new(side, side);
    let respect_orientation = native.ImageOrientation();
    let size = image.ImageSize(zoom, &default_size, respect_orientation);
    let natural_width = f64::from(size.width()) / f64::from(zoom);
    let natural_height = f64::from(size.height()) / f64::from(zoom);
    if !natural_width.is_finite()
        || !natural_height.is_finite()
        || natural_width <= 0.0
        || natural_height <= 0.0
    {
        return None;
    }
    Some(layoutng::internal::layout_input::ListMarkerImage {
        resource_id,
        shader,
        natural_width,
        natural_height,
        resolution_scale,
    })
}

fn ProjectForLayoutBoundary(
    native: &ComputedStyle,
    resolved: &mut ResolvedNodeStyle,
) -> Result<(), DocumentStyleError> {
    let _trace = browser_tracing::span("style_commit", "StyleEngine.ProjectForLayoutBoundary");
    let font = native.GetFontDescription();
    let mut ext = layoutng::internal::layout_input::ExtendedStyle::default();
    use layoutng::internal::layout_input::{
        Display, FloatSide, ListStylePosition as LayoutListStylePosition,
        ListStyleType as LayoutListStyleType, Overflow, Position, WhiteSpace,
    };
    resolved.style.display = match native.Display() {
        EDisplay::kInline => Display::kInline,
        EDisplay::kBlock => Display::kBlock,
        EDisplay::kInlineBlock => Display::kInlineBlock,
        EDisplay::kListItem
        | EDisplay::kInlineListItem
        | EDisplay::kFlowRootListItem
        | EDisplay::kInlineFlowRootListItem => Display::kListItem,
        EDisplay::kFlex | EDisplay::kWebkitBox => Display::kFlex,
        EDisplay::kInlineFlex | EDisplay::kWebkitInlineBox => Display::kInlineFlex,
        EDisplay::kGrid => Display::kGrid,
        EDisplay::kInlineGrid => Display::kInlineGrid,
        EDisplay::kTable => Display::kTable,
        EDisplay::kInlineTable => Display::kInlineTable,
        EDisplay::kTableRowGroup => Display::kTableSection,
        EDisplay::kTableHeaderGroup => Display::kTableHeaderGroup,
        EDisplay::kTableFooterGroup => Display::kTableFooterGroup,
        EDisplay::kTableRow => Display::kTableRow,
        EDisplay::kTableColumn => Display::kTableColumn,
        EDisplay::kTableColumnGroup => Display::kTableColumnGroup,
        EDisplay::kTableCell => Display::kTableCell,
        EDisplay::kTableCaption => Display::kTableCaption,
        EDisplay::kFlowRoot => Display::kFlowRoot,
        EDisplay::kMath => Display::kMath,
        EDisplay::kBlockMath => Display::kBlockMath,
        EDisplay::kRuby => Display::kRuby,
        EDisplay::kBlockRuby => Display::kBlockRuby,
        EDisplay::kRubyText => Display::kRubyText,
        EDisplay::kGridLanes | EDisplay::kInlineGridLanes => Display::kGridLanes,
        EDisplay::kLayoutCustom | EDisplay::kInlineLayoutCustom => Display::kCustom,
        // Box generation/contents flags carry the structural distinction;
        // SVG's display:none g uses its dedicated hidden-container object.
        EDisplay::kNone | EDisplay::kContents => Display::kInline,
    };
    resolved.style.position = match native.GetPosition() {
        foundation::EPosition::kStatic => Position::kStatic,
        foundation::EPosition::kRelative => Position::kRelative,
        foundation::EPosition::kAbsolute => Position::kAbsolute,
        foundation::EPosition::kFixed => Position::kFixed,
        foundation::EPosition::kSticky => Position::kSticky,
    };
    resolved.style.floating = match native.Floating() {
        foundation::EFloat::kNone => FloatSide::kNone,
        foundation::EFloat::kLeft => FloatSide::kLeft,
        foundation::EFloat::kRight => FloatSide::kRight,
        foundation::EFloat::kInlineStart => FloatSide::kInlineStart,
        foundation::EFloat::kInlineEnd => FloatSide::kInlineEnd,
    };
    use layoutng_style::css::white_space::EWhiteSpace;
    ext.white_space = match native.WhiteSpace() {
        EWhiteSpace::kNormal => WhiteSpace::kNormal,
        EWhiteSpace::kNowrap => WhiteSpace::kNowrap,
        EWhiteSpace::kPre => WhiteSpace::kPre,
        EWhiteSpace::kPreLine => WhiteSpace::kPreLine,
        EWhiteSpace::kPreWrap => WhiteSpace::kPreWrap,
        EWhiteSpace::kBreakSpaces => WhiteSpace::kBreakSpaces,
        value if value.bits() == 2 => WhiteSpace::kPreserveBreaksNowrap,
        value if value.bits() == 3 => WhiteSpace::kBreakSpacesNowrap,
        _ => unreachable!("white-space collapse/wrap fields exceeded their bit widths"),
    };
    let overflow = |value| match value {
        foundation::EOverflow::kVisible => Overflow::kVisible,
        foundation::EOverflow::kHidden => Overflow::kHidden,
        foundation::EOverflow::kScroll => Overflow::kScroll,
        foundation::EOverflow::kAuto | foundation::EOverflow::kOverlay => Overflow::kAuto,
        foundation::EOverflow::kClip => Overflow::kClip,
    };
    ext.overflow_x = overflow(native.OverflowX());
    ext.overflow_y = overflow(native.OverflowY());
    ext.list_style_position = match native.ListStylePosition() {
        foundation::EListStylePosition::kOutside => LayoutListStylePosition::kOutside,
        foundation::EListStylePosition::kInside => LayoutListStylePosition::kInside,
    };
    ext.list_style_image = ProjectListMarkerImage(native);
    let list_style_type = native.ListStyleType().Get();
    ext.list_style_type = if list_style_type.is_null() {
        LayoutListStyleType::kNone
    } else {
        let list_style_type = unsafe { &*list_style_type };
        if !list_style_type.IsCounterStyle() {
            LayoutListStyleType::kDisc
        } else {
            match list_style_type.GetCounterStyleName().Utf8().as_str() {
                "none" => LayoutListStyleType::kNone,
                "circle" => LayoutListStyleType::kCircle,
                "square" => LayoutListStyleType::kSquare,
                "decimal" => LayoutListStyleType::kDecimal,
                "lower-alpha" | "lower-latin" => LayoutListStyleType::kLowerAlpha,
                _ => LayoutListStyleType::kDisc,
            }
        }
    };

    // FontDescription::ComputedSize already includes EffectiveZoom. The
    // neutral layout snapshot is author-space and ZoomedStyle applies that
    // factor at the boundary exactly once.
    ext.font_size = font.SpecifiedSize() as f64;
    ext.font_weight = font.Weight().ToFloat() as f64;
    ext.font_italic = font.Style().ToFloat() != 0.0;
    ext.font_smoothing = match font.FontSmoothing() {
        font_engine::FontSmoothingMode::kNoSmoothing => {
            layoutng::internal::layout_input::FontSmoothing::kNone
        }
        font_engine::FontSmoothingMode::kAntialiased => {
            layoutng::internal::layout_input::FontSmoothing::kAntialiased
        }
        font_engine::FontSmoothingMode::kSubpixelAntialiased => {
            layoutng::internal::layout_input::FontSmoothing::kSubpixelAntialiased
        }
        font_engine::FontSmoothingMode::kAutoSmoothing => {
            layoutng::internal::layout_input::FontSmoothing::kAuto
        }
    };
    // cpp: font_description_data.cc:127-145. The computed CSS value keeps a
    // Length/percentage, while Layout consumes the used spacing resolved from
    // the current computed font size.
    let effective_zoom = native.EffectiveZoom();
    let unzoom = |value: f32| {
        if effective_zoom > 0.0 {
            value as f64 / effective_zoom as f64
        } else {
            value as f64
        }
    };
    ext.letter_spacing = unzoom(font.LetterSpacing());
    ext.word_spacing = unzoom(font.WordSpacing());
    ext.effective_zoom = effective_zoom;
    ext.zoom = native.Zoom();
    ext.has_author_background = native.HasAuthorBackground();
    ext.has_author_border = native.HasAuthorBorder();
    ext.has_author_border_radius = native.HasAuthorBorderRadius();
    let mut family = font.Family() as *const font_engine::FontFamily;
    while !family.is_null() {
        let f = unsafe { &*family };
        if !f.FamilyName().empty() {
            ext.font_families.push(f.FamilyName().Utf8())
        }
        family = f.Next()
    }

    // cpp: ComputedStyle::GetCurrentColor/BorderWidth; StyleColor::Resolve.
    // Paint's neutral projection shares these scalar results with the native
    // object used by layout; complex paint payloads stay in their own mapping.
    let current = native.GetCurrentColor(None);
    let color=|value:&layoutng_style::css::style_color::StyleColor|->Result<layoutng::internal::layout_input_types::Color,DocumentStyleError>{
        let value=value.Resolve(current,native.UsedColorScheme(),None);
        let (red,green,blue)=value.ToSRGB();
        Ok(layoutng::internal::layout_input_types::Color{red,green,blue,alpha:value.Alpha()})
    };
    resolved.style.paint.color = color(native.Color())?;
    resolved.style.paint.background_color = color(native.BackgroundColor())?;
    resolved.style.paint.respect_image_orientation = native.ImageOrientation()
        == foundation::RespectImageOrientationEnum::kRespectImageOrientation;
    crate::resolver::layout_projection::ProjectBorderRadii(native, &mut resolved.style.paint);
    crate::resolver::layout_projection::ProjectShadows(native, &mut resolved.style.paint, color)?;
    // cpp: computed_style.h SVG fill/stroke and style_transform.cc.  Layout
    // consumes a renderer-neutral projection, but the native ComputedStyle is
    // the single source of truth for both CSS declarations and SVG
    // presentation attributes.  Do not leave these fields at PaintStyleData's
    // defaults after the native cascade has resolved them.
    use layoutng_style::style::svg_paint::SVGPaintType;
    let project_svg_paint = |paint: &layoutng_style::style::svg_paint::SVGPaint,
                             opacity: f32|
     -> Result<
        Option<layoutng::internal::layout_input_types::Color>,
        DocumentStyleError,
    > {
        if paint.paint_type == SVGPaintType::kNone {
            return Ok(None);
        }
        if !paint.HasColor() {
            return Ok(None);
        }
        let mut projected = color(paint.GetColor())?;
        projected.alpha *= opacity;
        Ok(Some(projected))
    };
    resolved.style.paint.svg_fill = project_svg_paint(native.FillPaint(), native.FillOpacity())?;
    resolved.style.paint.svg_stroke =
        project_svg_paint(native.StrokePaint(), native.StrokeOpacity())?;
    resolved.style.paint.svg_fill_reference = native
        .FillPaint()
        .HasUrl()
        .then(|| native.FillPaint().GetUrl().Utf8());
    resolved.style.paint.svg_stroke_reference = native
        .StrokePaint()
        .HasUrl()
        .then(|| native.StrokePaint().GetUrl().Utf8());
    resolved.style.paint.svg_fill_current_color = native.FillPaint().HasCurrentColor();
    resolved.style.paint.svg_stroke_current_color = native.StrokePaint().HasCurrentColor();

    resolved.style.paint.transform = crate::resolver::layout_projection::ProjectTransform(native);
    resolved.style.paint.border_colors = [
        color(native.BorderTopColor())?,
        color(native.BorderRightColor())?,
        color(native.BorderBottomColor())?,
        color(native.BorderLeftColor())?,
    ];
    resolved.style.paint.outline_color = color(native.OutlineColor())?;
    resolved.style.paint.outline_width = if native.OutlineStyle() == foundation::EBorderStyle::kNone
    {
        0.0
    } else {
        unzoom(*native.OutlineWidth() as f32)
    };
    resolved.style.paint.outline_offset = unzoom(*native.OutlineOffset() as f32);
    resolved.style.paint.opacity = native.Opacity();
    resolved.style.paint.visible = native.Visibility() == foundation::EVisibility::kVisible;
    resolved.style.paint.pointer_events_none =
        native.PointerEvents() == foundation::EPointerEvents::kNone;
    resolved.style.paint.cursor = native.Cursor();
    resolved.style.paint.z_index = (!native.HasAutoZIndex()).then(|| native.ZIndex());
    resolved.style.border.top = unzoom(native.BorderTopWidth() as f32);
    resolved.style.border.right = unzoom(native.BorderRightWidth() as f32);
    resolved.style.border.bottom = unzoom(native.BorderBottomWidth() as f32);
    resolved.style.border.left = unzoom(native.BorderLeftWidth() as f32);
    fn border(
        style: foundation::EBorderStyle,
    ) -> layoutng::internal::layout_input::BorderLineStyle {
        use foundation::EBorderStyle as N;
        use layoutng::internal::layout_input::BorderLineStyle as P;
        match style {
            N::kNone => P::kNone,
            N::kHidden => P::kHidden,
            N::kSolid => P::kSolid,
            N::kDashed => P::kDashed,
            N::kDotted => P::kDotted,
            N::kDouble => P::kDouble,
            N::kGroove => P::kGroove,
            N::kRidge => P::kRidge,
            N::kInset => P::kInset,
            N::kOutset => P::kOutset,
        }
    }
    resolved.style.border_styles = [
        border(native.BorderTopStyle()),
        border(native.BorderRightStyle()),
        border(native.BorderBottomStyle()),
        border(native.BorderLeftStyle()),
    ];
    resolved.style.paint.outline_style = border(native.OutlineStyle());
    crate::resolver::layout_projection::ProjectGeometry(native, resolved, &mut ext);
    crate::resolver::layout_projection::ProjectShapeOutside(native, &mut ext);
    resolved.style.extended = Some(ext.into());
    Ok(())
}

fn DocumentParserMode(document: &Document) -> CSSParserMode {
    if document.GetCompatibilityMode() == dom::persistent_document::CompatibilityMode::kQuirks {
        CSSParserMode::kHTMLQuirksMode
    } else {
        CSSParserMode::kHTMLStandardMode
    }
}

fn ExpectedSheetParserMode(document: &Document, origin: CascadeOrigin) -> Option<CSSParserMode> {
    match origin {
        CascadeOrigin::kUserAgent => Some(CSSParserMode::kUASheetMode),
        CascadeOrigin::kAuthor => Some(DocumentParserMode(document)),
        _ => None,
    }
}

#[derive(Clone, Default)]
struct SelectorDependencies {
    descendant: bool,
    sibling: bool,
    has: bool,
    positional: bool,
    reverse_positional: bool,
    last: bool,
    inline_selector: bool,
    part: bool,
}
impl SelectorDependencies {
    fn Merge(&mut self, other: &Self) {
        self.descendant |= other.descendant;
        self.sibling |= other.sibling;
        self.has |= other.has;
        self.positional |= other.positional;
        self.reverse_positional |= other.reverse_positional;
        self.last |= other.last;
        self.inline_selector |= other.inline_selector;
        self.part |= other.part;
    }

    // cpp: RuleSet::AddRule / RuleInvalidationDataVisitor. Reuse parser-owned
    // selectors instead of serializing and reparsing a CSSOM selector string.
    fn ForParsedSheet(sheet: &crate::production_style_sheet::ProductionStyleSheet) -> Self {
        let mut result = Self::default();
        for rule in &sheet.rules {
            result.AddList(rule.rule.Selectors());
        }
        for rule in &sheet.nested_declarations {
            result.AddList(rule.rule.InnerStyleRule().Selectors());
        }
        result
    }
    fn AddList(&mut self, list: &crate::css_selector_list::CSSSelectorList) {
        for complex in list.ComplexSelectors() {
            for selector in complex.SimpleSelectors() {
                self.Add(selector);
            }
        }
    }

    fn ForSheets(document: &Document, sheets: &[(Rc<CSSStyleSheet>, CascadeOrigin)]) -> Self {
        let context = SelectorParserContext {
            html: true,
            quirks: document.GetCompatibilityMode()
                == dom::persistent_document::CompatibilityMode::kQuirks,
        };
        let mut result = Self::default();
        for (sheet, _) in sheets {
            for rule in &sheet.rules {
                for selector in CSSSelectorParser::ParseSelector(
                    &foundation::String::from(rule.selector_text.as_str()),
                    &context,
                ) {
                    result.Add(&selector);
                }
            }
        }
        result
    }
    fn Add(&mut self, selector: &CSSSelector) {
        match selector.Relation() {
            RelationType::kDescendant
            | RelationType::kChild
            | RelationType::kRelativeDescendant
            | RelationType::kRelativeChild => self.descendant = true,
            RelationType::kDirectAdjacent
            | RelationType::kIndirectAdjacent
            | RelationType::kRelativeDirectAdjacent
            | RelationType::kRelativeIndirectAdjacent => self.sibling = true,
            _ => {}
        }
        use PseudoType::*;
        match selector.GetPseudoType() {
            kPseudoHas => self.has = true,
            kPseudoNthLastChild | kPseudoNthLastOfType => {
                self.positional = true;
                self.reverse_positional = true;
            }
            kPseudoLastChild | kPseudoLastOfType | kPseudoOnlyChild | kPseudoOnlyOfType => {
                self.positional = true;
                self.last = true;
            }
            kPseudoFirstChild | kPseudoFirstOfType | kPseudoNthChild | kPseudoNthOfType => {
                self.positional = true
            }
            // Form container state depends on descendants too.
            kPseudoHasDatalist => self.has = true,
            kPseudoLang | kPseudoDisabled | kPseudoEnabled => self.descendant = true,
            kPseudoPart => self.part = true,
            _ => {}
        }
        if selector.IsAttributeSelector()
            && selector
                .Attribute()
                .LocalName()
                .Utf8()
                .eq_ignore_ascii_case("style")
        {
            self.inline_selector = true;
        }
        if selector.GetPseudoType() == PseudoType::kPseudoParent {
            if let Some(parent) = selector.ParentRule() {
                self.AddList(parent.Selectors());
            }
        }
        if let Some(list) = selector.SelectorList() {
            self.AddList(&list);
        }
    }
}
// cpp: ElementTraversal and TreeScope traversal. Historical arena entries
// have no role in active selector invalidation. Each connected shadow scope is
// entered once through its existing DOM owner, including hidden DOM elements.
fn ConnectedElements(document: &Document) -> impl Iterator<Item = usize> + '_ {
    let mut stack = document.ActiveTreeScopes();
    stack.reverse();
    std::iter::from_fn(move || loop {
        let index = stack.pop()?;
        let node = document.Node(index);
        stack.extend(node.Children().iter().rev().copied());
        if node.Type() == DOMNodeType::kElement {
            return Some(index);
        }
    })
}

fn ConnectedElement(document: &Document, index: usize) -> bool {
    if index >= document.NodeCount() || document.Node(index).Type() != DOMNodeType::kElement {
        return false;
    }
    document.IsConnectedToDocument(index)
}

// cpp: flat_tree_traversal.cc:104-123,161-191. DOM owns both named and manual
// assignment; Style consumes only the active, ordered assigned-node view.
fn AssignedNodesForSlot(document: &Document, slot: usize) -> Vec<usize> {
    document.AssignedNodes(slot)
}

fn FlatTreeStyleParentElement(document: &Document, index: usize) -> Option<usize> {
    let parent = document.Node(index).Parent()?;
    if document.Node(parent).Type() == DOMNodeType::kElement {
        if document.ShadowRoot(parent).is_some() {
            return document.AssignedSlot(index);
        }
        if document.IsSlotSupportingAssignment(parent)
            && !AssignedNodesForSlot(document, parent).is_empty()
        {
            return None;
        }
        return Some(parent);
    }
    document.ShadowHost(parent)
}

fn DirtyDistributedHost(document: &Document, host: usize, dirty: &mut HashSet<usize>) {
    let Some(shadow_root) = document.ShadowRoot(host) else {
        return;
    };
    AddSubtree(document, shadow_root, dirty);
    for &child in document.Node(host).Children() {
        AddSubtree(document, child, dirty);
    }
}

fn InvalidateSlotAssignmentForNode(document: &Document, index: usize, dirty: &mut HashSet<usize>) {
    if document.IsSlotSupportingAssignment(index) {
        if let Some(host) = document.OwnerShadowHost(index) {
            DirtyDistributedHost(document, host, dirty);
        }
        return;
    }
    let Some(parent) = document.Node(index).Parent() else {
        return;
    };
    if document.ShadowRoot(parent).is_some() {
        DirtyDistributedHost(document, parent, dirty);
    }
}

fn InvalidateSlotAssignmentForChildren(
    document: &Document,
    parent: usize,
    dirty: &mut HashSet<usize>,
) {
    if document.ShadowRoot(parent).is_some() {
        DirtyDistributedHost(document, parent, dirty);
        return;
    }
    if let Some(host) = document.OwnerShadowHost(parent) {
        // Adding/removing a slot anywhere in this shadow scope can redistribute
        // the host's light children and toggle fallback content.
        DirtyDistributedHost(document, host, dirty);
    }
}

fn AddSubtree(document: &Document, root: usize, targets: &mut HashSet<usize>) {
    let mut stack = vec![root];
    while let Some(index) = stack.pop() {
        let node = document.Node(index);
        if node.Type() == DOMNodeType::kElement {
            targets.insert(index);
        }
        stack.extend_from_slice(node.Children());
        if let Some(shadow_root) = document.ShadowRoot(index) {
            stack.push(shadow_root);
        }
    }
}

// cpp: StyleEngine::ApplyRuleSetChanges(TreeScope&, ...). A changed shadow
// scope affects its own subtree and host/slotted subjects. Ancestor ::part
// rules can also reach nested scopes, so descendants of a changed scope are
// visited without widening the fallback to unrelated document elements.
fn InvalidateChangedTreeScopes(
    document: &Document,
    changed_scopes: &HashSet<usize>,
    targets: &mut HashSet<usize>,
) {
    if changed_scopes.is_empty() {
        return;
    }
    for scope in document.ActiveTreeScopes() {
        if scope == document.Root() {
            continue;
        }
        let mut ancestor_scope = Some(scope);
        let exposed_to_change = loop {
            let Some(candidate) = ancestor_scope else {
                break false;
            };
            if changed_scopes.contains(&candidate) {
                break true;
            }
            ancestor_scope = document
                .ShadowHost(candidate)
                .map(|host| document.TreeScopeRoot(host));
        };
        if !exposed_to_change {
            continue;
        }
        AddSubtree(document, scope, targets);
        if changed_scopes.contains(&scope) {
            if let Some(host) = document.ShadowHost(scope) {
                targets.insert(host);
            }
            for candidate in ConnectedElements(document) {
                let mut slot = document.AssignedSlot(candidate);
                while let Some(current) = slot {
                    if document.TreeScopeRoot(current) == scope {
                        targets.insert(candidate);
                        break;
                    }
                    slot = document.AssignedSlot(current);
                }
            }
        }
    }
}

fn MediaDiffersOnlyByViewport(
    previous: &MediaValuesCachedData,
    current: &MediaValuesCachedData,
) -> bool {
    if previous == current {
        return false;
    }
    let mut normalized = previous.clone();
    normalized.viewport_width = current.viewport_width;
    normalized.viewport_height = current.viewport_height;
    normalized.small_viewport_width = current.small_viewport_width;
    normalized.small_viewport_height = current.small_viewport_height;
    normalized.large_viewport_width = current.large_viewport_width;
    normalized.large_viewport_height = current.large_viewport_height;
    normalized.dynamic_viewport_width = current.dynamic_viewport_width;
    normalized.dynamic_viewport_height = current.dynamic_viewport_height;
    normalized.device_width = current.device_width;
    normalized.device_height = current.device_height;
    normalized == *current
}

fn PseudoUsesViewportUnits(pseudo: &dom::persistent_document::PseudoElement) -> bool {
    unsafe { pseudo.native_style.Get().as_ref() }.is_some_and(|style| style.HasViewportUnits())
        || pseudo
            .marker
            .as_deref()
            .is_some_and(PseudoUsesViewportUnits)
}

fn ResolvedStyleUsesViewportUnits(style: &dom::persistent_document::ResolvedNodeStyle) -> bool {
    unsafe { style.native_style.Get().as_ref() }.is_some_and(|style| style.HasViewportUnits())
        || [
            style.before.as_ref(),
            style.after.as_ref(),
            style.first_letter.as_ref(),
            style.placeholder.as_ref(),
            style.marker.as_ref(),
        ]
        .into_iter()
        .flatten()
        .any(PseudoUsesViewportUnits)
}

struct DocumentResolvedStyleCommit<'a> {
    document: &'a mut Document,
    viewport_dependent_elements: &'a mut HashSet<usize>,
}

impl style_commit::ResolvedStyleCommitBackend for DocumentResolvedStyleCommit<'_> {
    type NodeId = usize;
    type ResolvedStyle = ResolvedNodeStyle;

    fn CommitResolvedStyle(&mut self, index: usize, style: ResolvedNodeStyle) {
        if ResolvedStyleUsesViewportUnits(&style) {
            self.viewport_dependent_elements.insert(index);
        } else {
            self.viewport_dependent_elements.remove(&index);
        }
        self.document.SetResolvedStyle(index, style);
    }
}

// Immutable owner metadata used by the translated active-sheet comparison.
struct SheetQueryMetadata {
    queries: bool,
    unit_flags: u32,
}
impl crate::active_style_sheets::StyleSheetMediaQueries for SheetQueryMetadata {
    fn HasMediaQueryResults(&self) -> bool {
        self.queries
    }
    fn HasMediaQueries(&self) -> bool {
        self.queries
    }
    fn HasDynamicViewportDependentMediaQueries(&self) -> bool {
        self.unit_flags
            & crate::media_queries::media_query_exp::MediaQueryExpValue::<()>::kDynamicViewport
            != 0
    }
}
type RetainedRuleSet = Rc<crate::rule_set::RuleSet<crate::production_style_sheet::Backend>>;

fn ProductionRuleSetDifference(old: &RetainedRuleSet, new: &RetainedRuleSet) -> RetainedRuleSet {
    use crate::rule_set::{RuleSet, StyleRuleIdentitySet};

    let old_identities = old
        .AllRuleData()
        .into_iter()
        .map(|data| Rc::as_ptr(data.Rule()) as usize)
        .collect::<HashSet<_>>();
    let new_identities = new
        .AllRuleData()
        .into_iter()
        .map(|data| Rc::as_ptr(data.Rule()) as usize)
        .collect::<HashSet<_>>();
    let old_only = old
        .AllRuleData()
        .into_iter()
        .filter(|data| !new_identities.contains(&(Rc::as_ptr(data.Rule()) as usize)))
        .map(|data| data.Rule().clone())
        .collect::<StyleRuleIdentitySet<_>>();
    let new_only = new
        .AllRuleData()
        .into_iter()
        .filter(|data| !old_identities.contains(&(Rc::as_ptr(data.Rule()) as usize)))
        .map(|data| data.Rule().clone())
        .collect::<StyleRuleIdentitySet<_>>();
    let mut difference = RuleSet::new();
    difference.AddFilteredRulesFromOtherSet(old, &old_only);
    difference.AddFilteredRulesFromOtherSet(new, &new_only);
    Rc::new(difference)
}

// No CSSOM RuleSetDiff is published through this contract yet. An empty diff
// list invokes Chromium's complete old/new RuleSet branch.
enum NoRuleSetDiff {}
impl
    crate::active_style_sheets::RuleSetDiff<
        crate::rule_set::RuleSet<crate::production_style_sheet::Backend>,
    > for NoRuleSetDiff
{
    fn Matches(
        &self,
        _: &crate::rule_set::RuleSet<crate::production_style_sheet::Backend>,
        _: &crate::rule_set::RuleSet<crate::production_style_sheet::Backend>,
    ) -> bool {
        match *self {}
    }
    fn CreateDiffRuleset(&self) -> Option<RetainedRuleSet> {
        match *self {}
    }
}
fn ChangedRuleSetSheetIdentities(
    old: &[(Rc<CSSStyleSheet>, CascadeOrigin)],
    new: &[(Rc<CSSStyleSheet>, CascadeOrigin)],
    old_rules: &[Option<RetainedRuleSet>],
    current_rules: &HashMap<(u64, u64), RetainedRuleSet>,
) -> Option<HashSet<(u64, u64)>> {
    use crate::active_style_sheets::{ActiveStyleSheet, ChangedRuleSets, CompareActiveStyleSheets};
    // Layer reordering retains the existing full invalidation boundary.
    if old
        .iter()
        .flat_map(|(s, o)| s.layer_order.iter().map(move |n| (o, n)))
        .ne(new
            .iter()
            .flat_map(|(s, o)| s.layer_order.iter().map(move |n| (o, n))))
    {
        return None;
    }
    let old_rules = old_rules.iter().cloned().collect::<Option<Vec<_>>>()?;
    let new_rules = new
        .iter()
        .map(|(s, _)| current_rules.get(&(s.Identity(), s.Revision())).cloned())
        .collect::<Option<Vec<_>>>()?;
    let mut owners = HashMap::<(u64, u64, u8), (Rc<CSSStyleSheet>, Rc<SheetQueryMetadata>)>::new();
    let mut entries = |sheets: &[(Rc<CSSStyleSheet>, CascadeOrigin)], rules: &[RetainedRuleSet]| {
        sheets
            .iter()
            .zip(rules)
            .map(|((sheet, origin), rules)| {
                let key = (sheet.Identity(), sheet.Revision(), *origin as u8);
                let owner = owners
                    .get(&key)
                    .filter(|(old, _)| old == sheet)
                    .map(|(_, owner)| owner.clone())
                    .unwrap_or_else(|| {
                        let metadata = Rc::new(SheetQueryMetadata {
                            queries: !sheet.owner_media_conditions.is_empty()
                                || sheet.rules.iter().any(|r| !r.media_conditions.is_empty()),
                            unit_flags: rules.Features().media_query_result_flags.unit_flags,
                        });
                        owners.insert(key, (sheet.clone(), metadata.clone()));
                        metadata
                    });
                ActiveStyleSheet::new(owner, Some(rules.clone()))
            })
            .collect::<Vec<_>>()
    };
    let old_entries = entries(old, &old_rules);
    let new_entries = entries(new, &new_rules);
    let mut changed = ChangedRuleSets::default();
    CompareActiveStyleSheets(
        &old_entries,
        &new_entries,
        &[] as &[NoRuleSetDiff],
        &mut changed,
    );
    Some(
        old.iter()
            .zip(old_rules.iter())
            .chain(new.iter().zip(new_rules.iter()))
            .filter(|(_, rules)| {
                changed
                    .Values()
                    .iter()
                    .any(|changed| Rc::ptr_eq(changed, rules))
            })
            .map(|((sheet, _), _)| (sheet.Identity(), sheet.Revision()))
            .collect(),
    )
}

fn InlineSheetContentsKey(source: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hash);
    hash.finish()
}

fn ChangedSheets(
    old: &[(Rc<CSSStyleSheet>, CascadeOrigin)],
    new: &[(Rc<CSSStyleSheet>, CascadeOrigin)],
) -> Vec<usize> {
    if std::ptr::eq(old, new) {
        return vec![];
    }
    let layers_changed = old
        .iter()
        .flat_map(|(s, o)| s.layer_order.iter().map(move |name| (o, name)))
        .collect::<Vec<_>>()
        != new
            .iter()
            .flat_map(|(s, o)| s.layer_order.iter().map(move |name| (o, name)))
            .collect::<Vec<_>>();
    (0..old.len().max(new.len()))
        .filter(|&i| layers_changed || old.get(i) != new.get(i))
        .collect()
}
// cpp: Element::ShouldStoreComputedStyle, SVGGElement/SVGMarkerElement::
// LayoutObjectIsNeeded. Resource containers still inherit and style descendants
// inside display:none <g>; SVGStopElement retains style without a layout object.
fn UsesSVGHiddenContainer(document: &Document, index: usize, style: &ComputedStyle) -> bool {
    let node = document.Node(index);
    style.Display() == EDisplay::kNone
        && node.IsElement("g", DOMNamespace::kSVG)
        && FlatTreeStyleParentElement(document, index)
            .is_some_and(|parent| document.Node(parent).Namespace() == DOMNamespace::kSVG)
}

fn ShouldStoreComputedStyle(document: &Document, index: usize, style: &ComputedStyle) -> bool {
    if style.Display() != EDisplay::kNone {
        return true;
    }
    let node = document.Node(index);
    if node.IsHTMLElement("area") {
        return true;
    }
    if node.Namespace() == DOMNamespace::kSVG {
        let has_svg_parent = FlatTreeStyleParentElement(document, index)
            .is_some_and(|parent| document.Node(parent).Namespace() == DOMNamespace::kSVG);
        return has_svg_parent && matches!(node.Name(), "g" | "marker" | "stop");
    }
    false
}

struct ReattachElement<'a> {
    document: &'a Document,
    index: usize,
}
impl layoutng_style::style::computed_style::LayoutTreeElement for ReattachElement<'_> {
    fn IsHTMLProgressElement(&self) -> bool {
        self.document.Node(self.index).IsHTMLElement("progress")
    }
    fn IsHTMLLegendElement(&self) -> bool {
        self.document.Node(self.index).IsHTMLElement("legend")
    }
    fn SupportsBaseAppearance(
        &self,
        appearance: layoutng_style::style::appearance::AppearanceValue,
    ) -> bool {
        crate::resolver::production_style_cascade::SupportsBaseAppearance(
            self.document,
            self.index,
            appearance,
        )
    }
}

fn StyleImpact(
    document: &Document,
    index: usize,
    old: Option<&ResolvedNodeStyle>,
    new: &ResolvedNodeStyle,
) -> (StyleUpdateImpact, bool) {
    let _trace = browser_tracing::span("style_diff", "ResolvedStyle.ComputeImpact");
    let Some(old) = old else {
        return (StyleUpdateImpact::TREE, true);
    };
    let (Some(old_native), Some(new_native)) =
        (unsafe { old.native_style.Get().as_ref() }, unsafe {
            new.native_style.Get().as_ref()
        })
    else {
        return (StyleUpdateImpact::TREE, true);
    };
    let reattach = ComputedStyle::NeedsReattachLayoutTree(
        &ReattachElement { document, index },
        old_native,
        new_native,
    );
    let mut pseudo_impact = StyleUpdateImpact::default();
    for (old, new) in [
        (&old.before, &new.before),
        (&old.after, &new.after),
        (&old.first_letter, &new.first_letter),
        (&old.placeholder, &new.placeholder),
    ] {
        match (old, new) {
            (None, None) => {}
            (Some(old), Some(new))
                if old.text == new.text
                    && old.display_contents == new.display_contents
                    && old.style.display == new.style.display
                    && old.style.position == new.style.position
                    && old.style.floating == new.style.floating =>
            {
                let fields = style_diff::FieldInvalidationDiff(
                    unsafe { &*old.native_style.Get() },
                    unsafe { &*new.native_style.Get() },
                );
                let layout = fields
                    & (FieldDifference::kLayout.bits() | FieldDifference::kReshape.bits())
                    != 0
                    || !old.style.LayoutEquivalent(&new.style);
                pseudo_impact.layout |= layout;
                pseudo_impact.paint |= layout || fields != 0 || old.style.paint != new.style.paint;
            }
            _ => return (StyleUpdateImpact::TREE, reattach),
        }
    }
    if reattach
        || old.generates_box != new.generates_box
        || old.own_generates_box != new.own_generates_box
        || old.own_display_contents != new.own_display_contents
        || old_native.Display() != new_native.Display()
        || old.style.display != new.style.display
        || ComputedStyle::NeedsReinsertLayoutTree(old_native, new_native)
    {
        return (StyleUpdateImpact::TREE, reattach);
    }
    let native_difference = style_diff::ComputeDifference(old_native, new_native);
    let fields = style_diff::FieldInvalidationDiff(old_native, new_native);
    // cpp: ComputedStyle::VisualInvalidationDiff, computed_style.cc:753-757.
    // A reshape changes intrinsic advances and therefore requires layout even
    // when the projected font size/family/weight fields remain unchanged.
    let layout = fields & (FieldDifference::kLayout.bits() | FieldDifference::kReshape.bits()) != 0
        || !old.style.LayoutEquivalent(&new.style);
    (
        StyleUpdateImpact {
            reattach: false,
            layout: pseudo_impact.layout || layout,
            paint: pseudo_impact.paint
                || native_difference != ComputedStyleDifference::kEqual
                || fields != 0
                || old.style.paint != new.style.paint
                || layout,
        },
        reattach,
    )
}

// cpp: StyleRecalcChange::RecalcContainerQueryDependentStyle and
// Element::RecalcStyle. Match flags, including false queries and cached
// pseudos, stay owned by the canonical style; this is only their subject set.
fn ContainerQueryDependent(style: &ComputedStyle) -> bool {
    let change =
        crate::style_recalc_change::StyleRecalcChange::default().ForceRecalcDescendantContainers();
    change.RecalcContainerQueryDependentStyle(style)
        || style.FirstLineDependsOnSizeContainerQueries()
        || style.HighlightsDependOnSizeContainerQueries()
        || PseudoStyleKind::DIRECT.into_iter().any(|kind| {
            unsafe {
                style
                    .GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                    .as_ref()
            }
            .is_some_and(|pseudo| change.RecalcContainerQueryDependentStyle(pseudo))
        })
}

/// Proves that the retained RuleSet positions are a complete ordinary-element
/// match signature. This does not classify selector support or alter matching;
/// it only decides whether a prior signature may be reused after a rule change.
fn OrdinaryMatchedRuleSignaturesAreComplete(
    sheet: &crate::production_style_sheet::ProductionStyleSheet,
    rule_set: &crate::rule_set::RuleSet<crate::production_style_sheet::Backend>,
) -> bool {
    let flat_ordinary_sheet =
        sheet.diagnostics.is_empty()
            && (0..sheet.contents.RuleCount()).all(|index| {
                sheet.contents.RuleAt(index).AsStyle().is_some_and(|rule| {
                    rule.ChildRules().is_none_or(|children| children.is_empty())
                })
            });
    if !flat_ordinary_sheet {
        return false;
    }

    let special_rules = [
        rule_set.CuePseudoRules(),
        rule_set.ScrollbarRules(),
        rule_set.ActiveViewTransitionRules(),
        rule_set.UnboundedPseudoClassRules(),
    ]
    .into_iter()
    .flatten()
    .map(|data| Rc::as_ptr(data.Rule()) as usize)
    .collect::<HashSet<_>>();

    rule_set.AllRuleData().iter().all(|data| {
        sheet.ProjectedRuleIndexForRetainedStyleRule(data.Rule())
            == Some(data.GetPosition() as usize)
            && data.Rule().Selectors().ComplexSelectors().count() == 1
            && !data.IsStartingStyle()
            && rule_set.ScopeForRule(data).is_none()
            && rule_set.ContainerQueriesForRule(data).is_none()
            && data.Rule().GetMixinParameterBindings().is_none()
            && !special_rules.contains(&(Rc::as_ptr(data.Rule()) as usize))
            && data.Selector().SimpleSelectors().all(|selector| {
                selector.Match() != MatchType::kPseudoElement
                    && selector.SelectorList().is_none()
                    && !matches!(
                        selector.GetPseudoType(),
                        PseudoType::kPseudoHost | PseudoType::kPseudoHostContext
                    )
            })
    })
}

fn OrdinarySignatureElement(document: &Document, index: usize) -> bool {
    document.Node(index).Namespace() == DOMNamespace::kHTML
        && document.TreeScopeRoot(index) == document.Root()
        && document.CorrespondingSVGElement(index).is_none()
}

fn NativeStyleHasVariableDependencies(style: &ComputedStyle) -> bool {
    let uses_variables = |candidate: &ComputedStyle| {
        candidate.HasVariableDeclaration() || candidate.HasVariableReference()
    };
    if uses_variables(style) {
        return true;
    }
    NativePseudoStyleHasVariableDependencies(style)
}

fn NativePseudoStyleHasVariableDependencies(style: &ComputedStyle) -> bool {
    let uses_variables = |candidate: &ComputedStyle| {
        candidate.HasVariableDeclaration() || candidate.HasVariableReference()
    };
    for kind in PseudoStyleKind::DIRECT {
        let pseudo = unsafe {
            style
                .GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                .as_ref()
        };
        if pseudo.is_some_and(|pseudo| {
            uses_variables(pseudo)
                || unsafe {
                    pseudo
                        .GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdMarker)
                        .as_ref()
                }
                .is_some_and(uses_variables)
        }) {
            return true;
        }
    }
    let highlights = style.HighlightData();
    for (id, pointer) in [
        (PseudoId::kPseudoIdSelection, highlights.Selection()),
        (
            PseudoId::kPseudoIdSearchText,
            highlights.SearchTextCurrent(),
        ),
        (
            PseudoId::kPseudoIdSearchText,
            highlights.SearchTextNotCurrent(),
        ),
        (PseudoId::kPseudoIdTargetText, highlights.TargetText()),
        (PseudoId::kPseudoIdSpellingError, highlights.SpellingError()),
        (PseudoId::kPseudoIdGrammarError, highlights.GrammarError()),
    ] {
        if style.HasPseudoElementStyle(id)
            && unsafe { pointer.as_ref() }.is_some_and(uses_variables)
        {
            return true;
        }
    }
    style.HasPseudoElementStyle(PseudoId::kPseudoIdHighlight)
        && unsafe { style.CustomHighlightNames().as_ref() }.is_some_and(|names| {
            names.iter().any(|name| {
                unsafe { highlights.CustomHighlight(name).as_ref() }.is_some_and(uses_variables)
            })
        })
}

fn ChangedCustomPropertyNames(
    old: &CustomProperties,
    new: &CustomProperties,
) -> HashSet<std::string::String> {
    let mut changed = HashSet::new();
    for (name, old_value) in old.values.iter() {
        if new.values.get(name).is_none_or(|new_value| {
            !crate::resolver::production_style_builder::custom_properties::ComputedVariablesEquivalent(
                old_value,
                new_value,
            )
        }) {
            changed.insert(name.clone());
        }
    }
    for name in new.values.keys() {
        if !old.values.contains_key(name) {
            changed.insert(name.clone());
        }
    }
    changed
}

fn PatchInheritedCustomProperties(
    old: &Rc<CustomProperties>,
    parent: &Rc<CustomProperties>,
    changed: &HashSet<std::string::String>,
    local: &HashSet<std::string::String>,
) -> Rc<CustomProperties> {
    if changed.is_empty() {
        return old.clone();
    }
    let mut patched = old.as_ref().clone();
    for name in changed {
        if local.contains(name) {
            continue;
        }
        match parent.values.get(name).cloned() {
            Some(value) => patched.SetData(name.clone(), value),
            None => patched.RemoveData(name),
        }
    }
    Rc::new(patched)
}

fn CustomPropertiesEquivalent(a: &CustomProperties, b: &CustomProperties) -> bool {
    if a.registered_values.len() != b.registered_values.len()
        || !a.registered_values.iter().all(|(name, value)| {
            b.registered_values
                .get(name)
                .is_some_and(|other| value.as_ref() == other.as_ref())
        })
    {
        return false;
    }
    // Registered equality remains first (including its numeric semantics).
    // Immutable unregistered maps detach through Rc::make_mut on every write.
    if Rc::ptr_eq(&a.values, &b.values) {
        return true;
    }
    if a.values.len() != b.values.len() {
        return false;
    }
    a.values.iter().all(|(name, left)| {
        let Some(right) = b.values.get(name) else {
            return false;
        };
        crate::resolver::production_style_builder::custom_properties::ComputedVariablesEquivalent(
            left, right,
        )
    })
}

fn FinishUpdate(document: &mut Document, stats: StyleUpdateStats, impact: StyleUpdateImpact) {
    let state = document.StyleStateMut();
    state.all_dirty = false;
    state.changes.clear();
    state.independent_inline_styles.clear();
    state.attribute_changes.clear();
    state.sibling_sensitive_nodes.clear();
    state.descendant_sensitive_nodes.clear();
    state.append_only_children.clear();
    state.non_append_children.clear();
    state.child_change_boundaries.clear();
    state.inserted_style_subtrees.clear();
    state.append_affected_subtrees.clear();
    state.stats = stats;
    if impact.layout {
        state.measurement_geometry_revision = state.measurement_geometry_revision.wrapping_add(1);
    }
    state.impact.Merge(impact);
}

// cpp: ElementRuleCollector::MatchContainerQuerySet; ContainerSelector::FindContainer.
// An immutable typed projection is evaluated against the closest eligible live
// ancestor. Current traversal values take precedence over last-commit caches.
fn ContainerConditionsMatch(
    document: &Document,
    subject: usize,
    media: &MediaValuesCachedData,
    conditions: &[cssom::CSSContainerQuerySet],
    states: &HashMap<usize, ContainerState>,
    registered: Option<&dyn RegisteredContainerPropertyResolver>,
    stored_custom: &HashMap<usize, Rc<CustomProperties>>,
    native: Option<&HashMap<usize, Persistent<ComputedStyle>>>,
    current_custom: Option<&HashMap<usize, Rc<CustomProperties>>>,
    environment_variables: Option<&dyn EnvironmentVariableResolver>,
    diagnostics: &mut Vec<DocumentStyleError>,
) -> bool {
    use crate::kleene_value::{KleeneOr, KleeneValue};
    conditions.iter().all(|set| {
        let mut result = KleeneValue::kFalse;
        for query in &set.queries {
            let mut ancestor = document.Node(subject).Parent();
            let mut matched = KleeneValue::kFalse;
            while let Some(index) = ancestor {
                let node = document.Node(index);
                ancestor = node.Parent();
                if node.Type() != DOMNodeType::kElement {
                    continue;
                }
                let style = native
                    .and_then(|n| n.get(&index))
                    .and_then(|s| unsafe { s.Get().as_ref() })
                    .or_else(|| {
                        document
                            .ResolvedStyleFor(index)
                            .and_then(|s| unsafe { s.native_style.Get().as_ref() })
                    });
                let Some(style) = style else { continue };
                if !query.name.IsNull() && !query.name.Utf8().is_empty() {
                    let names = unsafe { style.ContainerName().Get().as_ref() };
                    if !names.is_some_and(|list| {
                        list.GetNames().iter().any(|name| {
                            unsafe { name.Get().as_ref() }
                                .is_some_and(|name| name.GetName() == &query.name)
                        })
                    }) {
                        continue;
                    }
                }
                let required = query
                    .condition
                    .as_ref()
                    .map_or(0, |c| RequiredContainerType(c, style));
                if style.ContainerType() & required != required {
                    continue;
                }
                let Some(condition) = &query.condition else {
                    matched = KleeneValue::kTrue;
                    break;
                };
                let empty = CustomProperties::default();
                let custom = current_custom
                    .and_then(|c| c.get(&index))
                    .or_else(|| stored_custom.get(&index))
                    .map(Rc::as_ref)
                    .unwrap_or(&empty);
                let values = DocumentMediaValues::ForContainer(
                    document,
                    index,
                    media,
                    style,
                    states.get(&index),
                );
                let mut root_index = index;
                let mut parent = node.Parent();
                while let Some(index) = parent {
                    parent = document.Node(index).Parent();
                    if document.Node(index).Type() == DOMNodeType::kElement {
                        root_index = index;
                    }
                }
                let root = native
                    .and_then(|n| n.get(&root_index))
                    .and_then(|s| unsafe { s.Get().as_ref() })
                    .or_else(|| {
                        document
                            .ResolvedStyleFor(root_index)
                            .and_then(|s| unsafe { s.native_style.Get().as_ref() })
                    })
                    .unwrap_or(style);
                let values = values.WithRootStyle(root);
                let environment = CascadeEnvironment::new(environment_variables);
                let attribute_source = PersistentAttributeSource::new(
                    document,
                    index,
                    node.Namespace() == DOMNamespace::kHTML,
                );
                let attribute_state = CascadeAttributeState::new();
                let attributes = AttributeResolver::new(&attribute_source, &attribute_state, false);
                let unsupported = Cell::new(None);
                let parent_custom = node
                    .Parent()
                    .and_then(|parent| {
                        current_custom
                            .and_then(|c| c.get(&parent))
                            .or_else(|| stored_custom.get(&parent))
                    })
                    .map(Rc::as_ref);
                let resolver = ContainerStyleResolver {
                    node: index,
                    values: &values,
                    custom,
                    parent_custom,
                    substitution: SubstitutionContext {
                        environment: Some(&environment),
                        attributes: Some(&attributes),
                    },
                    registered,
                    unsupported: &unsupported,
                };
                matched = EvaluateContainerCondition(condition, &resolver);
                if let Some(operation) = unsupported.get() {
                    diagnostics.push(DocumentStyleError::Unsupported(operation))
                }
                break;
            }
            result = KleeneOr(result, matched);
            if result == KleeneValue::kTrue {
                break;
            }
        }
        result == KleeneValue::kTrue
    })
}

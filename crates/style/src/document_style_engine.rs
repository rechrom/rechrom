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
        media_query_backend::{MediaQueryValueBackend, ParseMediaQuerySet},
        media_query_evaluator::MediaQueryEvaluator,
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
    computed_style_base::{ComputedStyleBase, FieldDifference, IsAtShadowBoundary},
    computed_style_constants::{PseudoId, PseudoIdFlags},
    content_data::{ImageContentData, TextContentData},
    display_adjustment::AdjustComputedDisplayForLayout,
    style_fetched_image::{FetchedImage, StyleFetchedImage},
    style_image::StyleImage,
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
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
        &self, value: &crate::production_css_value::CSSImageValue,
    ) -> Result<std::ptr::NonNull<StyleImage>, LonghandApplicationError> {
        use layoutng_style::style::style_mask_source_image::StyleMaskSourceImage;
        let local = value.url_data.unresolved_url.starts_with('#') || {
            let parsed = url::Url::parse(value.url_data.resolved_url.as_ref()).ok();
            let document = url::Url::parse(self.document.DocumentURL()).ok();
            match (parsed, document) {
                (Some(mut a), Some(mut b)) => { a.set_fragment(None); b.set_fragment(None); a == b },
                _ => false,
            }
        };
        crate::resolver::element_style_resources::LoadMaskSourceBranch(local, || {
            let image = foundation::MakeGarbageCollected(StyleMaskSourceImage::Local(
                value.url_data.clone(), self.document.SVGResourceScope(self.originating_index),
            ));
            Ok(std::ptr::NonNull::new(image.cast()).unwrap())
        }, || {
            let fetched = self.ResolveImage(value)?;
            let image = foundation::MakeGarbageCollected(StyleMaskSourceImage::Fetched(
                value.url_data.clone(), fetched.Image().as_ptr(),
            ));
            Ok(std::ptr::NonNull::new(image.cast()).unwrap())
        })
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
    specificity: u32,
    proximity: u32,
    order: usize,
    origin: CascadeOrigin,
    inline: bool,
    layer: u16,
    valid_property_filter: ValidPropertyFilter,
    retained_properties: Option<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>>,
    properties: Rc<Vec<PropertyValue>>,
    /// Author declarations retain the TreeScope selected by
    /// StyleResolver::MatchAllRules. MatchResult numbers these scopes in the
    /// order they are encountered, so a compatibility-entry merge must
    /// register the same scope again before re-adding the declaration.
    tree_scope_root: Option<usize>,
    /// Exact entry emitted by SortAndTransferMatchedRules. Compatibility-only
    /// entries leave this empty and are appended during the final merge.
    transferred: Option<crate::resolver::match_result::MatchedProperties>,
}

struct CollectedRules {
    rules: Vec<MatchedRule>,
    /// The exact aggregate populated by ElementRuleCollector for the direct
    /// ordinary path, or by the explicit compatibility collector otherwise.
    match_result: MatchResult,
}

struct ResolvedPseudoOutput {
    pseudo: PseudoElement,
    custom_properties: Rc<CustomProperties>,
}

fn ConsumeProductionRuleCollection(
    stats: &mut TranslatedRuleShadowStats,
    sheets: &[(&CSSStyleSheet, CascadeOrigin)],
    rule_sets_consulted: usize,
    authoritative_rules: usize,
    tree_scope_root: Option<usize>,
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
    stats.authoritative_rules += authoritative_rules;
    stats.skipped_starting_style += report.skipped_starting_style;
    stats.skipped_pseudo += report.skipped_pseudo;
    stats.skipped_unsupported_selector += report.skipped_unsupported_selector;
    stats.skipped_scope += report.skipped_scope;
    stats.skipped_container_query += report.skipped_container_query;
    for transferred in collection.rules {
        let sheet_base = sheets[..transferred.sheet_index]
            .iter()
            .map(|(sheet, _)| sheet.rules.len())
            .sum::<usize>();
        let properties = Rc::new(
            DocumentCascadeExpansionBackend.Properties(transferred.matched.properties.as_ref()),
        );
        stats.typed_declarations_used += 1;
        direct_rules.push(MatchedRule {
            specificity: transferred.specificity,
            proximity: transferred.proximity,
            order: sheet_base + transferred.position as usize + 1,
            origin: transferred.matched.data_.origin,
            inline: transferred.matched.data_.is_inline_style(),
            layer: transferred.matched.data_.layer_order,
            valid_property_filter: ValidPropertyFilter::kNoFilter,
            retained_properties: None,
            properties,
            tree_scope_root,
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
    author: Vec<crate::resolver::production_style_resolver::ProductionResolverRuleSetInput>,
}

#[derive(Default)]
struct ProductionHighlightRuleCandidates {
    pseudos: PseudoIdFlags,
    custom_names: Vec<AtomicString>,
}

impl ProductionResolverRuleSets {
    /// Collect the finite request space from retained typed selectors once per
    /// update. Actual element matching remains wholly owned by
    /// ElementRuleCollector; this list only avoids probing names which cannot
    /// occur in any active RuleSet.
    fn HighlightRuleCandidates(&self) -> ProductionHighlightRuleCandidates {
        let mut candidates = ProductionHighlightRuleCandidates::default();
        let mut custom_names = HashSet::new();
        let inputs = std::iter::once(&self.html_user_agent)
            .chain(self.html_quirks_user_agent.iter())
            .chain(self.forced_colors_user_agent.iter())
            .chain(self.marker_user_agent.iter())
            .chain(self.fullscreen_user_agent.iter())
            .chain(self.svg_user_agent.iter())
            .chain(self.mathml_user_agent.iter())
            .chain(self.author.iter());
        for input in inputs {
            for data in input.rule_set.AllRuleData() {
                for selector in data.Selector().SimpleSelectors() {
                    if selector.Match() != MatchType::kPseudoElement {
                        continue;
                    }
                    let pseudo_id = GetPseudoId(selector.GetPseudoType());
                    if !layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(
                        pseudo_id,
                    ) {
                        continue;
                    }
                    candidates.pseudos.MaybeSet(pseudo_id);
                    if pseudo_id == PseudoId::kPseudoIdHighlight {
                        custom_names.insert((&*selector.Argument()).clone());
                    }
                }
            }
        }
        candidates.custom_names.extend(custom_names);
        candidates
    }
}

/// Compatibility declaration storage for sources that do not yet arrive as a
/// parser-owned CSSPropertyValueSet (inline style and SVG presentation
/// attributes). Production stylesheet rules use their original retained set.
struct LegacyMatchedPropertySet(Rc<Vec<PropertyValue>>);

impl MatchedPropertySet for LegacyMatchedPropertySet {
    fn AsAny(&self) -> &dyn std::any::Any {
        self
    }
    fn Equals(&self, other: &dyn MatchedPropertySet) -> bool {
        other.AsAny().downcast_ref::<Self>().is_some_and(|other| {
            Rc::ptr_eq(&self.0, &other.0) || self.0.as_slice() == other.0.as_slice()
        })
    }
    fn GetHash(&self) -> u32 {
        self.0.iter().fold(0, |hash, property| {
            foundation::HashInts(
                hash,
                foundation::HashInts(property.PropertyID() as u32, property.Value().Hash()),
            )
        })
    }
    fn ModifiedSinceHashing(&self) -> bool {
        false
    }
}

struct DocumentCascadeExpansionBackend;

impl CascadeExpansionBackend for DocumentCascadeExpansionBackend {
    type Dispatch = ProductionCSSValueDispatch;
    type Document = ();

    fn Properties(&self, set: &dyn MatchedPropertySet) -> Vec<PropertyValue> {
        if let Some(set) = set
            .AsAny()
            .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
        {
            return match set {
                CSSPropertyValueSetRuleHandle::Immutable(set) => set.Properties().to_vec(),
                CSSPropertyValueSetRuleHandle::Mutable(set) => set.borrow().Properties().to_vec(),
            };
        }
        set.AsAny()
            .downcast_ref::<LegacyMatchedPropertySet>()
            .expect("document cascade property set must be retained or compatibility storage")
            .0
            .as_ref()
            .clone()
    }

    fn CustomPropertyMetadata(
        &self,
        _: &foundation::AtomicString,
        _: &Self::Document,
    ) -> CSSProperty {
        *CSSProperty::Get(CSSPropertyID::kVariable)
    }

    fn IsAffectedByAll(&self, property: &CSSProperty) -> bool {
        <ProductionCSSValueDispatch as CSSPropertyValueBackend>::IsAffectedByAll(
            property.PropertyID(),
        )
    }
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
/// default sheet before matching. Keep the compatibility collector on that
/// same owner boundary when a translated trial falls back; otherwise MathML's
/// universal rules would leak into HTML/SVG and SVG tag rules could match
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
        // Untyped UA inputs predate the browser-owned role metadata. The
        // translated resolver rejects that bundle atomically; compatibility
        // preserves the former caller contract for such external inputs.
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
    Universal,
}

fn SelectorBucketForSubject(selectors: &CSSSelectorList) -> SelectorBucket {
    let Some(subject) = selectors.First() else {
        return SelectorBucket::Universal;
    };
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
        if selector.Relation() != RelationType::kSubSelector {
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

struct RuleCandidateIndex {
    targets: [RuleCandidateBuckets; 8],
    // Source order is one-based in retained match signatures.
    locations: Vec<(usize, usize)>,
}

/// Per-update evidence from the retained translated
/// stylesheet/RuleSet/SelectorChecker path. Ordinary standards-mode author
/// rules and UA-mode user-agent rules use its candidate and match results in
/// the real cascade.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TranslatedRuleShadowStats {
    pub rule_sets_built: usize,
    /// Matched author or user-agent rules whose retained typed declaration
    /// vector fed the real cascade. These rules never pass through declaration
    /// text parsing.
    pub typed_declarations_used: usize,
    /// Rules that deliberately stayed on the compatibility parser because
    /// their parser mode or diagnostics cannot yet be transferred losslessly.
    pub legacy_declaration_fallbacks: usize,
    pub elements_probed: usize,
    pub rule_sets_consulted: usize,
    /// Calls into translated ElementRuleCollector::CollectMatchingRules.
    pub translated_collector_calls: usize,
    /// Property sets emitted by translated SortAndTransferMatchedRules.
    pub translated_property_sets_transferred: usize,
    /// Transferred property sets retained in the MatchResult consumed by the
    /// production cascade, without a CSSOM projected-index reconstruction.
    pub translated_property_sets_consumed: usize,
    /// Real Update/Cascade candidates containing at least one translated
    /// property set. Empty and compatibility-only results are not counted.
    pub translated_style_cascade_attempts: usize,
    /// Attempts whose translated StyleCascade produced the final native style.
    pub translated_style_cascade_applied: usize,
    /// Candidates rejected by the all-typed storage gate or by a typed backend
    /// fallback and wholly reapplied by the existing document cascade. Trial
    /// builders are never published.
    pub translated_style_cascade_fallbacks: usize,
    /// Document-element attempts. These are tracked separately because their
    /// initial parent and scheme-sensitive initial color are root-only inputs.
    pub translated_style_cascade_root_attempts: usize,
    /// Root attempts whose translated cascade produced the published style.
    pub translated_style_cascade_root_applied: usize,
    /// Root attempts wholly discarded after a typed backend fallback.
    pub translated_style_cascade_root_fallbacks: usize,
    /// Translated StyleCascade attempts for pseudo-element native styles.
    pub translated_style_cascade_pseudo_attempts: usize,
    /// Pseudo-element attempts whose translated cascade produced the published
    /// native ComputedStyle.
    pub translated_style_cascade_pseudo_applied: usize,
    /// Pseudo-element attempts wholly discarded after a typed backend fallback.
    pub translated_style_cascade_pseudo_fallbacks: usize,
    /// Candidates whose declarations depend on the document property registry.
    /// These remain whole-cascade fallbacks until the translated backend owns
    /// the registry's syntax and initial-value callbacks.
    pub translated_style_cascade_registered_attempts: usize,
    pub translated_style_cascade_registered_applied: usize,
    pub translated_style_cascade_registered_fallbacks: usize,
    /// Element cascades carrying document-owned sampled animation values.
    pub translated_style_cascade_animation_attempts: usize,
    pub translated_style_cascade_animation_applied: usize,
    pub translated_style_cascade_animation_fallbacks: usize,
    /// Element resolutions that entered the translated
    /// StyleResolver::ApplyBaseStyleNoCache owner after the document had
    /// assembled its complete collaborator bundle.
    pub translated_style_resolver_attempts: usize,
    /// Attempts whose returned ComputedStyle became Update's published style.
    pub translated_style_resolver_applied: usize,
    /// Whole resolver trials discarded after a typed owner reported a gap.
    pub translated_style_resolver_fallbacks: usize,
    pub candidates_considered: usize,
    pub selector_matches: usize,
    pub authoritative_rules: usize,
    pub skipped_starting_style: usize,
    pub skipped_pseudo: usize,
    pub skipped_unsupported_selector: usize,
    pub skipped_scope: usize,
    pub skipped_container_query: usize,
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

// cpp: document.h:729,2856; document.cc:1103; style_engine.cc:StyleEngine,
// UpdateActiveStyleSheets/UpdateStyleAndLayoutTree; style_resolver.cc:ResolveStyle.
// The source's platform/page services are outside this document-only assembly.
// Unsupported declarations are retained as typed diagnostics and skipped.
// Structural errors keep all previously rooted document styles.
pub struct DocumentStyleEngine {
    document: DOMOwnerHandle,
    selectors: Rc<RefCell<PersistentSelectorService>>,
    diagnostics: Vec<DocumentStyleError>,
    active_sheets: Arc<Vec<(CSSStyleSheet, CascadeOrigin)>>,
    /// TreeScope owner aligned with `active_sheets`; UA entries have no
    /// author scope. Retaining the previous sequence lets stylesheet removal
    /// invalidate the old ShadowRoot even after CSSOM drops the owner.
    active_sheet_scopes: Arc<Vec<Option<usize>>>,
    // Chromium keeps StyleSheetContents alive from parse through RuleSet
    // construction. CSSOM carries a type-erased transfer owner so this engine
    // can retain the translated typed sheet without exposing it across the
    // package boundary or reparsing its source projection.
    typed_sheets: HashMap<(u64, u64), Rc<crate::production_style_sheet::ProductionStyleSheet>>,
    typed_sheet_fallback_parses: usize,
    translated_rule_sets:
        HashMap<(u64, u64), Rc<crate::rule_set::RuleSet<crate::production_style_sheet::Backend>>>,
    translated_rule_routing:
        HashMap<(u64, u64), Rc<crate::production_rule_matching::ProductionRuleRouting>>,
    matched_properties_cache:
        crate::resolver::production_style_resolver::ProductionMatchedPropertiesCache,
    resolver_dependencies: ResolverDependencyState,
    // Immutable per-family font inputs are derived from this Style-owned
    // document-author registry, never from Layout or a Page-side CSS scan.
    font_feature_values: crate::font_feature_values_registry::FontFeatureValuesRegistry,
    translated_rule_set_environment: Option<MediaValuesCachedData>,
    translated_rule_shadow_stats: TranslatedRuleShadowStats,
    sheet_revision: Option<u64>,
    environment: Option<(MediaValuesCachedData, CompatibilityMode)>,
    matches: HashMap<usize, Vec<(usize, u32, u32)>>,
    cascade_values: HashMap<usize, HashMap<CSSPropertyID, Rc<Value>>>,
    custom_properties: HashMap<usize, Rc<CustomProperties>>,
    sheet_dependencies: Vec<SelectorDependencies>,
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
    // StyleRule owns a parsed CSSPropertyValueSet in Chromium. The current
    // CSSOM boundary still exposes exact declaration source, so preserve the
    // same parse-once lifetime here until that boundary carries the typed
    // owner directly.
    declaration_cache: HashMap<(String, u8), Rc<Vec<PropertyValue>>>,
    container_states: HashMap<usize, ContainerState>,
    layout_container_nodes: HashSet<usize>,
    property_registry: crate::property_registry::PropertyRegistry,
    registered_container_properties: Option<Rc<dyn RegisteredContainerPropertyResolver>>,
    color_scheme_settings: production_style_builder::ColorSchemeSettings,
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
        let mut stats = CustomPropertyMemoryDiagnostics { node_entries: self.custom_properties.len(), ..Default::default() };
        let (mut records, mut maps, mut registered, mut data) = (HashSet::new(), HashSet::new(), HashSet::new(), HashSet::new());
        for custom in self.custom_properties.values() {
            if !records.insert(Rc::as_ptr(custom)) { continue; }
            stats.custom_records += 1;
            if maps.insert(Rc::as_ptr(&custom.values)) {
                stats.value_maps += 1;
                stats.value_entries += custom.values.len();
                stats.value_key_capacity += custom.values.keys().map(std::string::String::capacity).sum::<usize>();
                stats.value_bucket_payload_capacity_estimate += custom.values.capacity() * std::mem::size_of::<(std::string::String, ComputedVariableValue)>();
                for value in custom.values.values() {
                    if let ComputedVariableValue::Data(value) = value {
                        if data.insert(Rc::as_ptr(value)) {
                            stats.variable_data_owners += 1;
                            stats.token_inline_capacity += value.tokens.capacity() * std::mem::size_of::<crate::production_css_value::VariableToken>();
                        }
                    }
                }
            }
            if registered.insert(Rc::as_ptr(&custom.registered_values)) {
                stats.registered_maps += 1;
                stats.registered_entries += custom.registered_values.len();
                stats.registered_key_capacity += custom.registered_values.keys().map(std::string::String::capacity).sum::<usize>();
            }
        }
        stats
    }

    pub fn new(owner: &DOM) -> Self {
        Self {
            document: owner.GetDocument().RootHandle(),
            selectors: Default::default(),
            diagnostics: vec![],
            active_sheets: Default::default(),
            active_sheet_scopes: Default::default(),
            typed_sheets: HashMap::new(),
            typed_sheet_fallback_parses: 0,
            translated_rule_sets: HashMap::new(),
            translated_rule_routing: HashMap::new(),
            matched_properties_cache: Default::default(),
            resolver_dependencies: Default::default(),
            font_feature_values: Default::default(),
            translated_rule_set_environment: None,
            translated_rule_shadow_stats: Default::default(),
            sheet_revision: None,
            environment: None,
            matches: HashMap::new(),
            cascade_values: HashMap::new(),
            custom_properties: HashMap::new(),
            sheet_dependencies: vec![],
            style_selectors: HashMap::new(),
            rule_candidate_index: None,
            media_query_cache: HashMap::new(),
            declaration_cache: HashMap::new(),
            container_states: HashMap::new(),
            layout_container_nodes: HashSet::new(),
            property_registry: Default::default(),
            registered_container_properties: None,
            color_scheme_settings: Default::default(),
        }
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
            owner.GetDocumentMut().StyleStateMut().all_dirty = true;
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
    /// Publish the immutable interaction snapshot used by selector matching.
    /// cpp: Document::FocusedElement; HTMLOptionElement::IsMultiSelectFocused.
    /// Interaction owns event transitions; StyleEngine only retains the value
    /// for one update and invalidates the affected selector result broadly.
    pub fn SetUserInteractionState(
        &mut self,
        owner: &mut DOM,
        state: UserInteractionState,
    ) -> Result<bool, DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        if self.selectors.borrow().interaction == state {
            return Ok(false);
        }
        self.selectors.borrow_mut().interaction = state;
        owner.GetDocumentMut().StyleStateMut().all_dirty = true;
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
        (self.typed_sheets.len(), self.typed_sheet_fallback_parses)
    }
    #[cfg(test)]
    pub fn TranslatedRuleShadowStats(&self) -> TranslatedRuleShadowStats {
        self.translated_rule_shadow_stats
    }
    pub fn Update(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: &[CSSStyleSheet],
    ) -> Result<StyleUpdateImpact, DocumentStyleError> {
        self.UpdateWithEnvironment(owner, media, ua, None)
    }
    pub fn UpdateWithEnvironment(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: &[CSSStyleSheet],
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<StyleUpdateImpact, DocumentStyleError> {
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
            || state
                .changes
                .iter()
                .any(|c| matches!(c, StyleChange::Children(_)))
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
        let ua_unchanged = self
            .active_sheets
            .iter()
            .filter(|(_, origin)| *origin == CascadeOrigin::kUserAgent)
            .map(|(s, _)| s)
            .eq(ua.iter());
        let active_scope_sheets = document.ActiveStyleSheetsWithTreeScopes();
        let active_sheets = if self.sheet_revision == Some(revision) && ua_unchanged {
            self.active_sheets.clone()
        } else {
            Arc::new(
                ua.iter()
                    .cloned()
                    .map(|s| (s, CascadeOrigin::kUserAgent))
                    .chain(
                        active_scope_sheets
                            .iter()
                            .map(|(_, sheet)| (*sheet).clone())
                            .map(|s| (s, CascadeOrigin::kAuthor)),
                    )
                    .collect::<Vec<_>>(),
            )
        };
        let active_sheet_scopes = Arc::new(
            std::iter::repeat_n(None, ua.len())
                .chain(active_scope_sheets.iter().map(|(scope, _)| Some(*scope)))
                .collect::<Vec<_>>(),
        );
        self.SynchronizeTypedSheets(&active_sheets);
        // Ordinary standards-mode author and UA-mode user-agent matching
        // consume these retained RuleSets below. Rebuild only when the sheet
        // revision or media environment changes; advanced recalc contexts
        // remain explicit compatibility-collector fallbacks.
        self.SynchronizeTranslatedRuleSets(&active_sheets, media);
        let font_features_changed = self.SynchronizeFontFeatureValues(
            document,
            media,
            &active_sheets,
            active_sheet_scopes.as_slice(),
        );
        let function_registry = {
            let evaluator =
                crate::production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(media);
            let typed_sheets = active_sheets
                .iter()
                .filter_map(|(sheet, _)| {
                    self.typed_sheets
                        .get(&(sheet.Identity(), sheet.Revision()))
                        .map(Rc::as_ref)
                })
                .collect::<Vec<_>>();
            crate::production_style_sheet::ProductionFunctionRegistry::FromStyleSheets(
                &typed_sheets,
                &evaluator,
            )
        };
        let registry_changed = self
            .property_registry
            .UpdateDeclaredProperties(&active_sheets, &DocumentMediaValues::new(document, media));
        let environment = (media.clone(), document.GetCompatibilityMode());
        let all_dirty = font_features_changed || registry_changed || document.StyleState().all_dirty || self.environment.as_ref() != Some(&environment)
            // Container ancestry, scope activation and computed custom values are query inputs.
            // Until per-container dependency sets are retained, any DOM/style
            // mutation revisits conditional-rule subjects against current ancestors.
            || (active_sheets.iter().any(|(s,_)|s.rules.iter().any(|r|!r.container_conditions.is_empty() || !r.scope_conditions.is_empty()))
                && !document.StyleState().changes.is_empty());
        let sheet_changes = ChangedSheets(&self.active_sheets, &active_sheets);
        if font_features_changed
            || registry_changed
            || self.environment.as_ref() != Some(&environment)
            || !sheet_changes.is_empty()
            || environment_variables.is_some()
            || resource_changed
        {
            self.matched_properties_cache.Clear();
        }
        // cpp: matched_properties_cache.cc:392-431. Blink performs this from
        // the Oilpan weak callback; the retained Rust document owner reaches
        // the same lifecycle point once per style update.
        self.matched_properties_cache.Clean();
        let changed_tree_scopes = sheet_changes
            .iter()
            .flat_map(|&index| {
                [
                    self.active_sheet_scopes.get(index).copied().flatten(),
                    active_sheet_scopes.get(index).copied().flatten(),
                ]
            })
            .flatten()
            .collect::<HashSet<_>>();
        let quirks_changed = self
            .environment
            .as_ref()
            .is_none_or(|(_, mode)| *mode != environment.1);
        if quirks_changed || !sheet_changes.is_empty() {
            self.rule_candidate_index = None;
        }
        let mut rule_sets_built = 0;
        let sheet_dependencies: Vec<_> = active_sheets
            .iter()
            .enumerate()
            .map(|(index, sheet)| {
                if !quirks_changed && self.active_sheets.get(index) == Some(sheet) {
                    if let Some(retained) = self.sheet_dependencies.get(index) {
                        return retained.clone();
                    }
                }
                rule_sets_built += 1;
                SelectorDependencies::ForSheets(document, std::slice::from_ref(sheet))
            })
            .collect();
        let mut dependencies = SelectorDependencies::default();
        for sheet in &sheet_dependencies {
            dependencies.Merge(sheet);
        }
        let sheets: Vec<_> = active_sheets.iter().map(|(s, o)| (s, *o)).collect();
        // cpp: style_engine.cc:290-316. This is the retained equivalent of one
        // StyleSheetCollection/ScopedStyleResolver per active TreeScope. UA
        // sheets are document-wide; author sheets remain in their owner scope.
        let scoped_sheets = document
            .ActiveTreeScopes()
            .into_iter()
            .map(|scope| {
                let owned = ua
                    .iter()
                    .map(|sheet| (sheet, CascadeOrigin::kUserAgent))
                    .chain(
                        active_scope_sheets
                            .iter()
                            .filter(move |(candidate, _)| *candidate == scope)
                            .map(|(_, sheet)| (*sheet, CascadeOrigin::kAuthor)),
                    )
                    .collect::<Vec<_>>();
                (scope, owned)
            })
            .collect::<HashMap<_, _>>();
        let mut dirty = HashSet::new();
        let mut candidates = HashSet::new();
        let mut refreshed_matches = HashMap::new();
        if all_dirty {
            AddSubtree(document, document.Root(), &mut dirty);
        } else {
            let state = document.StyleState();
            for change in &state.changes {
                match *change {
                    StyleChange::Node(index) | StyleChange::Attribute(index, _) => {
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
                        dirty.insert(index);
                        if dependencies.inline_selector {
                            self.InvalidateRelations(
                                document,
                                index,
                                &dependencies,
                                &mut candidates,
                            );
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
                        candidates.insert(parent);
                        InvalidateSlotAssignmentForChildren(document, parent, &mut dirty);
                        // Child-state subjects such as :empty can be on the
                        // left side of a sibling combinator.
                        if dependencies.sibling {
                            if let Some(grandparent) = document.Node(parent).Parent() {
                                AddSubtree(document, grandparent, &mut candidates);
                            }
                        }
                        if state.non_append_children.contains(&parent)
                            && (dependencies.positional || dependencies.sibling)
                        {
                            AddSubtree(document, parent, &mut candidates);
                        } else if dependencies.reverse_positional {
                            AddSubtree(document, parent, &mut candidates);
                        } else if dependencies.last {
                            for &index in &state.append_affected_subtrees {
                                AddSubtree(document, index, &mut candidates);
                            }
                        }
                        if dependencies.has {
                            self.InvalidateHasAncestors(document, parent, &mut candidates);
                        }
                    }
                }
            }
            for &index in &state.inserted_style_subtrees {
                AddSubtree(document, index, &mut dirty);
            }
            // Rule-set invalidation checks both previously matched rules and
            // current matches. The former survives simultaneous DOM mutations.
            if !sheet_changes.is_empty() {
                let mut changed_selectors = HashSet::new();
                for &i in &sheet_changes {
                    for source in [&self.active_sheets, &active_sheets] {
                        if let Some((sheet, _)) = source.get(i) {
                            changed_selectors
                                .extend(sheet.rules.iter().map(|r| r.selector_text.clone()));
                        }
                    }
                }
                let old_selectors: Vec<_> = self
                    .active_sheets
                    .iter()
                    .flat_map(|(s, _)| s.rules.iter().map(|r| r.selector_text.clone()))
                    .collect();
                let current_selectors: Vec<_> = active_sheets
                    .iter()
                    .flat_map(|(s, _)| s.rules.iter().map(|r| &r.selector_text))
                    .collect();
                let mut elements = HashSet::new();
                AddSubtree(document, document.Root(), &mut elements);
                for index in elements {
                    let element_scope = document.TreeScopeRoot(document.CorrespondingSVGElement(index).unwrap_or(index));
                    let element_sheets = scoped_sheets
                        .get(&element_scope)
                        .map_or(&[][..], Vec::as_slice);
                    let matches = self.MatchingRules(
                        document,
                        index,
                        media,
                        element_sheets,
                        None,
                        None,
                        environment_variables,
                    );
                    let was_matched = self.matches.get(&index).is_some_and(|matches| {
                        matches.iter().any(|(order, _, _)| {
                            old_selectors
                                .get(order - 1)
                                .is_some_and(|s| changed_selectors.contains(s))
                        })
                    });
                    let is_matched = matches.iter().any(|(order, _, _)| {
                        current_selectors
                            .get(order - 1)
                            .is_some_and(|selector| changed_selectors.contains(*selector))
                    });
                    refreshed_matches.insert(index, matches);
                    if was_matched || is_matched {
                        dirty.insert(index);
                    }
                }
                // Rule order in the retained match signatures changed. Refresh
                // signatures for candidates instead of comparing unlike revisions.
            }
            // cpp: style_engine.cc:3225-3231. A ShadowRoot sheet collection
            // update invalidates its scoped subtree plus the exact elements
            // reached through :host and ::slotted. Outer-scope ::part rules
            // can reach into descendant shadow scopes, so a changed ancestor
            // scope also visits those scoped subtrees.
            if !changed_tree_scopes.is_empty() {
                for scope in document.ActiveTreeScopes() {
                    if scope == document.Root() {
                        continue;
                    }
                    let mut ancestor_scope = Some(scope);
                    let exposed_to_change = loop {
                        let Some(candidate) = ancestor_scope else {
                            break false;
                        };
                        if changed_tree_scopes.contains(&candidate) {
                            break true;
                        }
                        ancestor_scope = document
                            .ShadowHost(candidate)
                            .map(|host| document.TreeScopeRoot(host));
                    };
                    if !exposed_to_change {
                        continue;
                    }
                    AddSubtree(document, scope, &mut dirty);
                    if changed_tree_scopes.contains(&scope) {
                        if let Some(host) = document.ShadowHost(scope) {
                            dirty.insert(host);
                        }
                        for candidate in 0..document.NodeCount() {
                            if document.Node(candidate).Type() != DOMNodeType::kElement {
                                continue;
                            }
                            let mut slot = document.AssignedSlot(candidate);
                            while let Some(current) = slot {
                                if document.TreeScopeRoot(current) == scope {
                                    dirty.insert(candidate);
                                    break;
                                }
                                slot = document.AssignedSlot(current);
                            }
                        }
                    }
                }
            }
            for index in candidates {
                if dirty.contains(&index) || !ConnectedElement(document, index) {
                    continue;
                }
                let element_scope = document.TreeScopeRoot(document.CorrespondingSVGElement(index).unwrap_or(index));
                let element_sheets = scoped_sheets
                    .get(&element_scope)
                    .map_or(&[][..], Vec::as_slice);
                let matches = self.MatchingRules(
                    document,
                    index,
                    media,
                    element_sheets,
                    None,
                    None,
                    environment_variables,
                );
                if self.matches.get(&index) != Some(&matches) {
                    dirty.insert(index);
                }
            }
        }
        // SVGElement::InvalidateInstances: cloned declarations match within
        // the instance tree but use their corresponding element's RuleSets.
        // Conservatively revisit connected instances at each style lifecycle,
        // including CSSOM, ancestor selector and host-inheritance changes.
        for index in 0..document.NodeCount() {
            if document.CorrespondingSVGElement(index).is_some()
                && ConnectedElement(document, index)
            {
                dirty.insert(index);
            }
        }
        dirty.retain(|&index| ConnectedElement(document, index));
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
        if visit.is_empty() {
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
        let production_resolver_rule_sets = self.ProductionResolverRuleSets(
            document,
            media,
            &sheets,
            active_sheet_scopes.as_slice(),
        );
        let production_highlight_candidates = production_resolver_rule_sets
            .as_ref()
            .map(ProductionResolverRuleSets::HighlightRuleCandidates)
            .unwrap_or_default();
        // Build all immutable styles before publishing. The translated
        // StyleResolver owns Blink's initial-style lifecycle; this engine only
        // supplies its immutable host environment through a private adapter.
        let initial_root = crate::resolver::production_style_resolver::InitialStyleForDocument(
            media,
            self.color_scheme_settings,
        );
        let initial_root_ref = unsafe { &*initial_root.Get() };
        let mut pending = Vec::<(usize, ResolvedNodeStyle)>::new();
        let mut native = HashMap::<usize, Persistent<ComputedStyle>>::new();
        let mut stack = vec![(document.Root(), None, false, false, false, false, false)];
        let mut stats = StyleUpdateStats {
            rule_sets_built,
            ..Default::default()
        };
        let mut impact = StyleUpdateImpact::default();
        let mut pending_matches = refreshed_matches;
        let mut pending_values = HashMap::new();
        let mut pending_custom_properties = HashMap::new();
        let mut custom_properties = HashMap::<usize, Rc<CustomProperties>>::new();
        let mut root_font = media.em_size;
        while let Some((
            index,
            parent_index,
            ancestor_none,
            inherited,
            explicit,
            subtree,
            at_shadow_boundary,
        )) = stack.pop()
        {
            let node = document.Node(index);
            let element = node.Type() == DOMNodeType::kElement;
            let element_scope = document.TreeScopeRoot(document.CorrespondingSVGElement(index).unwrap_or(index));
            let element_sheets = scoped_sheets
                .get(&element_scope)
                .map_or(&[][..], Vec::as_slice);
            let mut next_parent = parent_index;
            let mut none = ancestor_none;
            let mut child_inherited = inherited;
            let mut child_explicit = explicit;
            let mut child_subtree = subtree;
            if element {
                let old = document.ResolvedStyleFor(index);
                let must_resolve = dirty.contains(&index)
                    || inherited
                    || subtree
                    || old.is_none_or(|s| s.native_style.Get().is_null())
                    || (explicit && old.is_none_or(|s| s.has_explicit_inheritance != Some(false)));
                if !must_resolve {
                    let old = old.unwrap();
                    native.insert(index, old.native_style.clone());
                    custom_properties.insert(
                        index,
                        self.custom_properties
                            .get(&index)
                            .cloned()
                            .unwrap_or_default(),
                    );
                    none |= unsafe { &*old.native_style.Get() }.Display() == EDisplay::kNone;
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
                    let parent = parent_index
                        .and_then(|p| native.get(&p))
                        .map(|p| unsafe { &*p.Get() });
                    let root_style = document
                        .Node(document.Root())
                        .Children()
                        .iter()
                        .copied()
                        .find(|&candidate| document.Node(candidate).Type() == DOMNodeType::kElement)
                        .and_then(|root_index| native.get(&root_index))
                        .map(|style| unsafe { &*style.Get() });
                    let mut builder = if let Some(parent) = parent {
                        ComputedStyleBuilder::from_initial_and_parent(
                            initial_root_ref,
                            parent,
                            if at_shadow_boundary {
                                IsAtShadowBoundary::kAtShadowBoundary
                            } else {
                                IsAtShadowBoundary::kNotAtShadowBoundary
                            },
                        )
                    } else {
                        ComputedStyleBuilder::from_style(initial_root_ref)
                    };
                    let inherited_custom = parent_index
                        .and_then(|parent| custom_properties.get(&parent))
                        .cloned();
                    // cpp: StyleResolver::ApplyBaseStyleNoCache
                    // (style_resolver.cc:1736-1944). A complete owner
                    // bundle enters the translated resolver before the
                    // compatibility collector/cascade. HTML, SVG and MathML
                    // each retain the namespace-specific default RuleSet that
                    // MatchUARules selects. Its typed result owns
                    // both native style and computed custom properties, so a
                    // successful MatchAllRules -> StyleCascade operation is
                    // published without replaying either phase. Any typed
                    // fallback discards the whole trial, then the compatibility
                    // path below computes both outputs atomically.
                    let mut translated_output = None;
                    if let Some(rule_sets) = production_resolver_rule_sets.as_ref() {
                        if matches!(
                            node.Namespace(),
                            DOMNamespace::kHTML | DOMNamespace::kSVG | DOMNamespace::kMathML
                        ) {
                            self.translated_rule_shadow_stats
                                .translated_style_resolver_attempts += 1;
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
                            let parent_handle = parent.map(|style| {
                                Persistent::from_ptr(
                                    style as *const ComputedStyle as *mut ComputedStyle,
                                )
                            });
                            let root_handle = root_style.map(|style| {
                                Persistent::from_ptr(
                                    style as *const ComputedStyle as *mut ComputedStyle,
                                )
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
                            collaborators.html_quirks_user_agent_rules =
                                rule_sets.html_quirks_user_agent.clone();
                            collaborators.forced_colors_user_agent_rules =
                                rule_sets.forced_colors_user_agent.clone();
                            collaborators.fullscreen_user_agent_rules =
                                rule_sets.fullscreen_user_agent.clone();
                            collaborators.root_style = root_handle;
                            collaborators.color_scheme_settings = self.color_scheme_settings;
                            collaborators.text_link_text_color =
                                self.resolver_dependencies.text_link_text_color;
                            collaborators.is_document_element = parent.is_none();
                            collaborators.subject_node = Some(index);
                            collaborators.container_runtime = Some(&container_runtime);
                            collaborators.parent_custom_properties = inherited_custom.as_deref();
                            collaborators.substitution_context = resolver_substitution;
                            collaborators.image_resolver = Some(&resolver_images);
                            collaborators.property_registry = Some(&self.property_registry);
                            collaborators.function_registry = Some(&function_registry);
                            let animation_samples =
                                document.AnimationStyles(document.Node(index).Id());
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
                                Err(fallbacks) => {
                                    self.translated_rule_shadow_stats
                                        .translated_style_resolver_fallbacks += 1;
                                    if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                                        eprintln!(
                                            "translated StyleResolver interpolation fallback node={index}: {fallbacks:?}"
                                        );
                                    }
                                    None
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
                                Err(fallbacks) => {
                                    self.translated_rule_shadow_stats
                                        .translated_style_resolver_fallbacks += 1;
                                    if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                                        eprintln!(
                                            "translated StyleResolver transition fallback node={index}: {fallbacks:?}"
                                        );
                                    }
                                    None
                                }
                            };
                            collaborators.animations = animation_interpolations.as_ref();
                            collaborators.transitions = transition_interpolations.as_ref();
                            collaborators.matched_properties_cache =
                                Some(self.matched_properties_cache.clone());
                            collaborators.selectors = Some(self.selectors.clone());
                            collaborators.interaction = self.selectors.borrow().interaction;
                            let may_apply = (animation_samples.is_empty()
                                || animation_interpolations.is_some())
                                && (transition_samples.is_empty()
                                    || transition_interpolations.is_some());
                            if may_apply {
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
                                    parent: parent_handle,
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
                                    translated_output = Some((
                                        translated,
                                        output.custom_properties,
                                        output.matched_signatures,
                                    ));
                                    self.translated_rule_shadow_stats
                                        .translated_style_resolver_applied += 1;
                                }
                                Err(fallbacks) => {
                                    self.translated_rule_shadow_stats
                                        .translated_style_resolver_fallbacks += 1;
                                    if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                                        eprintln!(
                                            "translated StyleResolver fallback node={index}: {fallbacks:?}"
                                        );
                                    }
                                }
                            }
                            }
                        }
                    }
                    let translated_resolver_applied = translated_output.is_some();
                    let (values, computed_custom, mut matched_signatures) = if let Some((
                        translated,
                        computed_custom,
                        signatures,
                    )) =
                        translated_output
                    {
                        builder = translated;
                        (HashMap::new(), computed_custom, signatures)
                    } else {
                        let rules = self.CollectRules(
                            document,
                            index,
                            media,
                            element_sheets,
                            &scoped_sheets,
                            None,
                            Some(&native),
                            Some(&custom_properties),
                            environment_variables,
                        )?;
                        let signatures = rules
                            .rules
                            .iter()
                            .filter(|rule| rule.order != 0 && rule.order != usize::MAX)
                            .map(|rule| (rule.order, rule.specificity, rule.proximity))
                            .collect();
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
                        let (values, computed_custom) = Self::Cascade(
                            document,
                            index,
                            &rules,
                            &mut builder,
                            parent,
                            root_style,
                            inherited_custom.as_ref(),
                            root_font,
                            media,
                            environment_variables,
                            Some(&container_runtime),
                            &self.property_registry,
                            Some(&function_registry),
                            self.color_scheme_settings,
                            self.resolver_dependencies.text_link_text_color,
                            &mut self.diagnostics,
                            &mut self.translated_rule_shadow_stats,
                        )?;
                        (values, computed_custom, signatures)
                    };
                    ApplyTextControlInnerEditorCustomStyle(document, index, &mut builder);
                    // cpp: MatchResult::SetHasPseudoElementStyle and
                    // StyleResolver::ApplyBaseStyleNoCache (1928). Public pseudo bits
                    // describe matched rules, independently of generated boxes.
                    let mut pseudo_rules = Vec::new();
                    let mut pseudo_flags = PseudoIdFlags::default();
                    for (pseudo_index, kind) in
                        PseudoStyleKind::MATCH_TARGETS.into_iter().enumerate()
                    {
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
                        if !kind.IsNestedMarker() && !rules.rules.is_empty() {
                            pseudo_flags.MaybeSet(kind.Id());
                        }
                        pseudo_rules.push((kind, rules));
                    }
                    // The translated MatchResult also carries non-box pseudo
                    // metadata (highlight names and highlight pseudo bits).
                    // Replacing it with the compatibility collector's four
                    // generated-pseudo bits loses Element::RecalcHighlightStyles'
                    // input. Compatibility owns the bits only when the complete
                    // translated resolver declined this element.
                    if translated_resolver_applied {
                        // The translated resolver already populated flags for
                        // rules it owns. Merge compatibility-owned pseudos
                        // (including a StyleRule routed here because it also
                        // contains ::before::marker) without discarding those
                        // production flags.
                        for value in PseudoId::kFirstPublicPseudoId.value()
                            ..=PseudoId::kLastTrackedPublicPseudoId.value()
                        {
                            let pseudo_id = PseudoId::from_bits(value);
                            if builder.HasPseudoElementStyle(pseudo_id) {
                                pseudo_flags.MaybeSet(pseudo_id);
                            }
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
                    if !translated_resolver_applied {
                        // The compatibility cascade publishes the same native
                        // style and must run the same StyleAdjuster branch.
                        crate::resolver::style_adjuster::AdjustStyleForZIndex(
                            &mut builder,
                            display_parent,
                        );
                    }
                    // cpp: Element::RecalcHighlightStyles (element.cc:6747-6828)
                    // and StyleResolver::StyleForHighlightPseudoElement
                    // (style_resolver.cc:10642-10652). Highlight pseudos are
                    // native cached styles on the originating ComputedStyle;
                    // they are not generated PseudoElement boxes. Each request
                    // inherits highlight properties from the parent's matching
                    // highlight style and initializes the remaining properties
                    // from this element's ordinary originating style.
                    self.ResolveFontFeatureValues(&mut builder);
                    let ordinary_style =
                        Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
                    let originating_style = unsafe { &*ordinary_style.Get() };
                    let mut style = ordinary_style.clone();
                    if production_resolver_rule_sets.is_some() {
                        let mut requests = Vec::<(
                            PseudoId,
                            Option<AtomicString>,
                            SearchTextRequest,
                            *const ComputedStyle,
                        )>::new();
                        let parent_highlights = parent.map(|style| style.HighlightData());
                        if production_highlight_candidates
                            .pseudos
                            .Has(PseudoId::kPseudoIdSelection)
                        {
                            requests.push((
                                PseudoId::kPseudoIdSelection,
                                None,
                                SearchTextRequest::kNone,
                                parent_highlights.map_or(std::ptr::null(), |data| data.Selection()),
                            ));
                        }
                        if production_highlight_candidates
                            .pseudos
                            .Has(PseudoId::kPseudoIdSearchText)
                        {
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
                                parent_highlights
                                    .map_or(std::ptr::null(), |data| data.SearchTextNotCurrent()),
                            ));
                        }
                        if production_highlight_candidates
                            .pseudos
                            .Has(PseudoId::kPseudoIdTargetText)
                        {
                            requests.push((
                                PseudoId::kPseudoIdTargetText,
                                None,
                                SearchTextRequest::kNone,
                                parent_highlights
                                    .map_or(std::ptr::null(), |data| data.TargetText()),
                            ));
                        }
                        if production_highlight_candidates
                            .pseudos
                            .Has(PseudoId::kPseudoIdSpellingError)
                        {
                            requests.push((
                                PseudoId::kPseudoIdSpellingError,
                                None,
                                SearchTextRequest::kNone,
                                parent_highlights
                                    .map_or(std::ptr::null(), |data| data.SpellingError()),
                            ));
                        }
                        if production_highlight_candidates
                            .pseudos
                            .Has(PseudoId::kPseudoIdGrammarError)
                        {
                            requests.push((
                                PseudoId::kPseudoIdGrammarError,
                                None,
                                SearchTextRequest::kNone,
                                parent_highlights
                                    .map_or(std::ptr::null(), |data| data.GrammarError()),
                            ));
                        }
                        if production_highlight_candidates
                            .pseudos
                            .Has(PseudoId::kPseudoIdHighlight)
                        {
                            for name in &production_highlight_candidates.custom_names {
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

                        if !requests.is_empty() {
                            if let Some(rule_sets) = production_resolver_rule_sets.as_ref() {
                                let mut highlight_builder =
                                    ComputedStyleBuilder::from_style(originating_style);
                                let mut resolved_custom_names =
                                    foundation::HashSet::<AtomicString>::new();
                                // Keep every produced style rooted until the
                                // completed originating style owns the Member
                                // edges in StyleHighlightData.
                                let mut highlight_roots = Vec::with_capacity(requests.len());
                                for (pseudo_id, argument, search_text, highlight_parent) in requests
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
                                    let highlight_parent = (!highlight_parent.is_null())
                                        .then(|| Persistent::from_ptr(highlight_parent.cast_mut()));
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
                                    collaborators.html_quirks_user_agent_rules =
                                        rule_sets.html_quirks_user_agent.clone();
                                    collaborators.forced_colors_user_agent_rules =
                                        rule_sets.forced_colors_user_agent.clone();
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
                                    collaborators.originating_style = Some(ordinary_style.clone());
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
                                    collaborators.property_registry = Some(&self.property_registry);
                                    collaborators.function_registry = Some(&function_registry);
                                    collaborators.matched_properties_cache =
                                        Some(self.matched_properties_cache.clone());
                                    collaborators.selectors = Some(self.selectors.clone());
                                    collaborators.interaction = self.selectors.borrow().interaction;
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
                                            if !translated_resolver_applied {
                                                pseudo_flags.MaybeSet(pseudo_id);
                                                if let Some(name) = argument.as_ref() {
                                                    resolved_custom_names.insert(name.clone());
                                                }
                                            }
                                            matched_signatures.extend(output.matched_signatures);
                                            highlight_roots.push(resolved_highlight);
                                        }
                                        Err(fallbacks) => {
                                            self.translated_rule_shadow_stats
                                                .translated_style_resolver_fallbacks += 1;
                                            if std::env::var_os(
                                                "RECHROM_DEBUG_STYLE_RESOLVER",
                                            )
                                            .is_some()
                                            {
                                                eprintln!(
                                                    "translated highlight StyleResolver fallback node={index} pseudo={pseudo_id:?}: {fallbacks:?}"
                                                );
                                            }
                                        }
                                    }
                                }
                                if !translated_resolver_applied {
                                    highlight_builder.SetPseudoElementStyles(pseudo_flags.Bits());
                                    if !resolved_custom_names.is_empty() {
                                        highlight_builder
                                            .SetCustomHighlightNames(&resolved_custom_names);
                                    }
                                }
                                style = Persistent::from_ptr(
                                    highlight_builder.TakeStyle() as *mut ComputedStyle
                                );
                                drop(highlight_roots);
                            }
                        }
                    }
                    pending_matches.insert(index, matched_signatures);
                    let final_style = unsafe { &*style.Get() };
                    if parent.is_none() {
                        root_font = final_style.GetFontDescription().SpecifiedSize()
                    }
                    let own_none = final_style.Display() == EDisplay::kNone;
                    none |= own_none;
                    let mut resolved = ResolvedNodeStyle::default();
                    resolved.own_display_contents = final_style.Display() == EDisplay::kContents;
                    resolved.display_contents = resolved.own_display_contents;
                    resolved.own_generates_box = !own_none && !resolved.own_display_contents;
                    resolved.generates_box = !none && resolved.own_generates_box;
                    ProjectForLayoutBoundary(final_style, &mut resolved)?;
                    resolved.has_explicit_inheritance = Some(final_style.HasExplicitInheritance());
                    resolved.SetNativeStyle(style.clone());
                    let old_native = old.and_then(|s| unsafe { s.native_style.Get().as_ref() });
                    // cpp: computed_style.cc:407-500; element.cc:5562-5583.
                    // Propagation is decided by the translated native style
                    // difference, not by a hand-maintained projection subset.
                    let native_difference = old_native
                        .map_or(ComputedStyleDifference::kDescendantAffecting, |old| {
                            ComputedStyle::ComputeDifference(old, final_style)
                        });
                    let custom_changed = self
                        .custom_properties
                        .get(&index)
                        .is_none_or(|old| !CustomPropertiesEquivalent(old, &computed_custom));
                    // A child's highlight style inherits from the matching
                    // highlight style on its flat-tree parent. That dependency
                    // is separate from ordinary CSS inheritance and therefore
                    // must explicitly schedule descendants when the native
                    // highlight cache changes.
                    let highlight_changed = old_native
                        .is_none_or(|old| old.HighlightData() != final_style.HighlightData());
                    let suppression_changed = old
                        .is_none_or(|s| s.generates_box != resolved.generates_box)
                        || old_native
                            .is_some_and(|old| (old.Display() == EDisplay::kNone) != own_none);
                    child_inherited = native_difference
                        >= ComputedStyleDifference::kIndependentInherited
                        || custom_changed
                        || highlight_changed;
                    child_subtree |= suppression_changed
                        || native_difference == ComputedStyleDifference::kDescendantAffecting;
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
                            PseudoStyleKind::BeforeMarker | PseudoStyleKind::AfterMarker => {
                                unreachable!(
                                    "nested markers resolve after their originating pseudo"
                                )
                            }
                        }
                    }
                    for kind in [PseudoStyleKind::BeforeMarker, PseudoStyleKind::AfterMarker] {
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
                        )?;
                        match kind {
                            PseudoStyleKind::BeforeMarker => {
                                resolved.before.as_mut().unwrap().marker = marker.map(Box::new)
                            }
                            PseudoStyleKind::AfterMarker => {
                                resolved.after.as_mut().unwrap().marker = marker.map(Box::new)
                            }
                            _ => unreachable!(),
                        }
                    }
                    let mut normalized = resolved.clone();
                    if let Some(old) = old {
                        normalized.SetNativeStyle(old.native_style.clone());
                    }
                    // The preliminary difference controls inheritance. Compare
                    // again after cache population so pseudo content changes
                    // participate in the native public-pseudo difference.
                    let final_difference = old_native
                        .map_or(ComputedStyleDifference::kDescendantAffecting, |old| {
                            ComputedStyle::ComputeDifference(old, final_style)
                        });
                    let equal = final_difference == ComputedStyleDifference::kEqual
                        // Generated style equality deliberately excludes these
                        // derived flags. A real page/preference update still
                        // publishes their newly computed native values.
                        && old_native.is_some_and(|old| {
                            old.DarkColorScheme() == final_style.DarkColorScheme()
                                && old.ColorSchemeForced() == final_style.ColorSchemeForced()
                                && old.ColorSchemeFlagsIsNormal() == final_style.ColorSchemeFlagsIsNormal()
                        })
                        // ::placeholder uses an internal PseudoId and is not
                        // covered by ComputedStyle's public-pseudo flag loop.
                        // Its form-control lifecycle still owns a distinct cache.
                        && old_native.is_some_and(|old| PseudoStyleKind::DIRECT.into_iter().all(|kind| {
                            old.GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                                == final_style.GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                        }))
                        && !subtree
                        && old == Some(&normalized);
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
                    if equal {
                        // Keep pointer identity and existing Arc for equal styles.
                        let old = old.unwrap();
                        resolved.SetNativeStyle(old.native_style.clone());
                    }
                    native.insert(index, resolved.native_style.clone());
                    if old != Some(&resolved) {
                        stats.changed_nodes += 1;
                        impact.Merge(StyleImpact(old, &resolved));
                        pending.push((index, resolved));
                    }
                    next_parent = Some(index);
                }
            }
            let mut push_child = |child: usize,
                                  parent: Option<usize>,
                                  ancestor_none: bool,
                                  inherited: bool,
                                  explicit: bool,
                                  subtree: bool,
                                  at_shadow_boundary: bool| {
                if inherited || explicit || subtree || visit.contains(&child) {
                    stack.push((
                        child,
                        parent,
                        ancestor_none,
                        inherited,
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
                        push_child(child, None, false, false, false, child_subtree, false);
                    }
                }
                let shadow_root = document.ShadowRoot(index).unwrap();
                for &child in document.Node(shadow_root).Children().iter().rev() {
                    push_child(
                        child,
                        next_parent,
                        none,
                        child_inherited,
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
                            child_explicit,
                            child_subtree,
                            false,
                        );
                    }
                } else {
                    // Fallback content remains styled while inactive, but its
                    // root has no flat-tree parent and starts from initial.
                    for &child in node.Children().iter().rev() {
                        push_child(child, None, false, false, false, child_subtree, false);
                    }
                    for child in assigned.into_iter().rev() {
                        push_child(
                            child,
                            next_parent,
                            none,
                            child_inherited,
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
                        child_explicit,
                        child_subtree,
                        false,
                    );
                }
            }
        }
        ResolveSVGPaintServers(document, &mut pending, &mut stats, &mut impact);
        drop(sheets);
        self.sheet_revision = Some(revision);
        self.active_sheets = active_sheets;
        self.active_sheet_scopes = active_sheet_scopes;
        self.environment = Some(environment);
        self.sheet_dependencies = sheet_dependencies;
        self.matches.extend(pending_matches);
        self.cascade_values.extend(pending_values);
        self.custom_properties.extend(pending_custom_properties);
        let document = owner.GetDocumentMut();
        for (index, style) in pending {
            document.SetResolvedStyle(index, style)
        }
        FinishUpdate(document, stats, impact);
        Ok(impact)
    }
    fn SynchronizeTypedSheets(&mut self, sheets: &[(CSSStyleSheet, CascadeOrigin)]) {
        let active_keys = sheets
            .iter()
            .map(|(sheet, _)| (sheet.Identity(), sheet.Revision()))
            .filter(|(identity, _)| *identity != 0)
            .collect::<HashSet<_>>();
        self.typed_sheets.retain(|key, _| active_keys.contains(key));
        for (sheet, _) in sheets {
            let key = (sheet.Identity(), sheet.Revision());
            if key.0 == 0 || self.typed_sheets.contains_key(&key) {
                continue;
            }
            // owner_media_conditions is attachment metadata, outside the
            // parsed StyleSheetContents just as LinkStyle's media owner is in
            // Blink. RuleSet consumers must continue evaluating that outer
            // condition from CSSOM rather than treating this owner as merged.
            let transferred = sheet
                .ParsedBacking::<crate::production_style_sheet::ProductionStyleSheet>()
                .filter(|parsed| parsed.source.Utf8() == sheet.SourceText());
            let parsed = if let Some(transferred) = transferred {
                transferred
            } else {
                // A directly edited CSSOM projection deliberately invalidates
                // its source provenance. Do not build a typed owner from stale
                // text; the compatibility path remains authoritative until the
                // CSSOM mutation API supplies a newly parsed revision.
                if !sheet.HasCurrentSource() {
                    continue;
                }
                self.typed_sheet_fallback_parses += 1;
                let source = foundation::String::FromUtf8(sheet.SourceText().as_bytes());
                Rc::new(crate::production_style_sheet::ParseStyleSheet(
                    &source,
                    crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
                ))
            };
            self.typed_sheets.insert(key, parsed);
        }
    }
    fn SynchronizeTranslatedRuleSets(
        &mut self,
        sheets: &[(CSSStyleSheet, CascadeOrigin)],
        media: &MediaValuesCachedData,
    ) {
        if self.translated_rule_set_environment.as_ref() != Some(media) {
            self.translated_rule_sets.clear();
            self.translated_rule_routing.clear();
            self.translated_rule_set_environment = Some(media.clone());
        }
        let active_keys = sheets
            .iter()
            .map(|(sheet, _)| (sheet.Identity(), sheet.Revision()))
            .filter(|(identity, _)| *identity != 0)
            .collect::<HashSet<_>>();
        self.translated_rule_sets
            .retain(|key, _| active_keys.contains(key));
        self.translated_rule_routing
            .retain(|key, _| active_keys.contains(key));
        let evaluator =
            crate::production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(media);
        let forced_colors_evaluator =
            crate::production_style_sheet::ProductionMediaQueryEvaluator::ForMediaType(
                "ua-forced-colors",
                media,
            );
        for (sheet, _) in sheets {
            let key = (sheet.Identity(), sheet.Revision());
            if key.0 == 0 || self.translated_rule_sets.contains_key(&key) {
                continue;
            }
            // A directly edited CSSOM clone intentionally drops its typed
            // provenance. Do not build a RuleSet from stale source; the legacy
            // path remains authoritative for that update.
            let Some(typed) = self.typed_sheets.get(&key) else {
                continue;
            };
            let medium = if sheet.UserAgentStyleSheetRole()
                == Some(CSSUserAgentStyleSheetRole::kForcedColors)
            {
                &forced_colors_evaluator
            } else {
                &evaluator
            };
            let rule_set = Rc::new(typed.BuildRuleSet(medium));
            let routing = Rc::new(
                crate::production_rule_matching::ProductionRuleRouting::Build(&rule_set, typed),
            );
            self.translated_rule_sets.insert(key, rule_set);
            self.translated_rule_routing.insert(key, routing);
            self.translated_rule_shadow_stats.rule_sets_built += 1;
        }
    }

    // cpp: ScopedStyleResolver::AddFontFeatureValuesRules, CSSFontSelector::
    // GetFontData, and StyleEngine::ApplyRuleSetChanges. At the pinned source
    // revision these aliases are document-author scoped; shadow/user-sheet
    // table fusion remains a source TODO. Owner media is outside RuleSet.
    fn SynchronizeFontFeatureValues(
        &mut self,
        document: &Document,
        media: &MediaValuesCachedData,
        sheets: &[(CSSStyleSheet, CascadeOrigin)],
        sheet_scopes: &[Option<usize>],
    ) -> bool {
        let values = DocumentMediaValues::new(document, media);
        let evaluator = MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(&values);
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
        let mut description = builder.GetFontDescription().clone();
        let mut resolved = Vec::new();
        if let Some(alternates) = description.FontVariantAlternatesValue() {
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
        let evaluator = MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(&values);
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
                continue;
            }
            let key = (sheet.Identity(), sheet.Revision());
            let typed = self.typed_sheets.get(&key)?;
            let rule_set = self.translated_rule_sets.get(&key)?.clone();
            let routing = self.translated_rule_routing.get(&key)?.clone();
            let expected_mode = match origin {
                CascadeOrigin::kUserAgent => CSSParserMode::kUASheetMode,
                CascadeOrigin::kAuthor
                    if DocumentParserMode(document) == CSSParserMode::kHTMLStandardMode =>
                {
                    CSSParserMode::kHTMLStandardMode
                }
                // The current production host deliberately rejects quirks
                // author sheets until its declaration and selector contexts
                // both retain that document mode.
                CascadeOrigin::kAuthor => return None,
                _ => return None,
            };
            if typed.contents.ParserContext().Mode() != expected_mode {
                return None;
            }
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
                    routing: Some(routing),
                    tree_scope_root,
                };
            match origin {
                CascadeOrigin::kUserAgent => {
                    let slot = match sheet.UserAgentStyleSheetRole()? {
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
            author,
        })
    }
    // Source's invalidation traversal is separate from the style resolver.
    // Relations widen candidates; retained matched-rule signatures decide which
    // candidates need a cascade. This includes losing a selector match.
    fn InvalidateRelations(
        &self,
        document: &Document,
        index: usize,
        dependencies: &SelectorDependencies,
        candidates: &mut HashSet<usize>,
    ) {
        let state = document.StyleState();
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
        if dependencies.has {
            self.InvalidateHasAncestors(document, index, candidates);
        }
    }
    fn InvalidateHasAncestors(
        &self,
        document: &Document,
        index: usize,
        candidates: &mut HashSet<usize>,
    ) {
        let mut ancestor = Some(index);
        while let Some(index) = ancestor {
            // :has subjects can themselves be ancestors of a final target, or
            // precede it through a sibling relation. Exact matching filters it.
            AddSubtree(document, index, candidates);
            ancestor = document.Node(index).Parent();
        }
    }
    fn BuildRuleCandidateIndex(
        &mut self,
        document: &Document,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
    ) -> RuleCandidateIndex {
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let mut index = RuleCandidateIndex {
            targets: std::array::from_fn(|_| RuleCandidateBuckets::default()),
            locations: Vec::new(),
        };
        let mut order = 0;
        for (sheet_index, &(sheet, _)) in sheets.iter().enumerate() {
            for (rule_index, rule) in sheet.rules.iter().enumerate() {
                order += 1;
                index.locations.push((sheet_index, rule_index));
                let variant = (quirks, !rule.scope_conditions.is_empty());
                self.EnsureCompiledSelectors(&rule.selector_text, &rule.scope_conditions, quirks);
                if let Some(selectors) = self
                    .style_selectors
                    .get(rule.selector_text.as_str())
                    .and_then(|variants| variants.get(&variant))
                {
                    for selector in selectors {
                        index.targets[RuleCandidateIndex::Target(selector.pseudo)]
                            .Add(order, &selector.bucket);
                    }
                }
            }
        }
        index
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
        let evaluator = MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(&values);
        let mut matches = vec![];
        let document_scope = document.TreeScopeRoot(index) == document.Root();
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
        if !had_index && document_scope {
            self.rule_candidate_index = Some(self.BuildRuleCandidateIndex(document, sheets));
        }
        matches
    }
    // cpp: HTMLStyleElement::ProcessStyleSheet; StyleEngine::UpdateActiveStyleSheets.
    // CSSOM owns the parsed sheets; owner-node replacement preserves source order.
    fn SynchronizeStyleSheets(&mut self, owner: &mut DOM) {
        let document = owner.GetDocument();
        let mut stack = vec![document.Root()];
        let mut sheets = vec![];
        while let Some(index) = stack.pop() {
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
                let mut sheet = crate::ParseCSS(if valid_type { &css } else { "" });
                sheet.owner_node_id = node.Id();
                if let Some(media) = node.FindAttribute("media") {
                    if !media.value.is_empty() {
                        // The media attribute belongs to the stylesheet owner,
                        // outside its retained StyleSheetContents/RuleSet.
                        // Keep that boundary explicit so both translated and
                        // compatibility collectors evaluate it exactly once.
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
        let node = document.Node(index);
        let values = DocumentMediaValues::new(document, media);
        let evaluator = MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(&values);
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
        let mut live_routing =
            HashMap::<usize, Rc<crate::production_rule_matching::ProductionRuleRouting>>::new();
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
            let pseudo_request = pseudo.map_or_else(
                crate::production_rule_context::ProductionPseudoRequest::Element,
                |pseudo| {
                    crate::production_rule_context::ProductionPseudoRequest::Pseudo(
                        pseudo.Id(),
                        None,
                    )
                },
            );
            let defer_author_until_after_slotted = pseudo.is_some_and(|kind| {
                kind.Id()
                    == layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdPlaceholder
            });
            let mut direct = Vec::<(
                usize,
                CascadeOrigin,
                Rc<crate::rule_set::RuleSet<crate::production_style_sheet::Backend>>,
                Rc<crate::production_rule_matching::ProductionRuleRouting>,
                Option<usize>,
            )>::new();
            for (sheet_index, &(sheet, origin)) in sheets.iter().enumerate() {
                if owner_media_rejected.contains(&sheet_index) {
                    self.translated_rule_shadow_stats.skipped_owner_media += 1;
                    continue;
                }
                if origin == CascadeOrigin::kUserAgent
                    && !UserAgentSheetAppliesToDocument(sheet, document, media, node.Namespace())
                {
                    continue;
                }
                let expected_mode = match origin {
                    CascadeOrigin::kUserAgent => CSSParserMode::kUASheetMode,
                    CascadeOrigin::kAuthor
                        if DocumentParserMode(document) == CSSParserMode::kHTMLStandardMode =>
                    {
                        CSSParserMode::kHTMLStandardMode
                    }
                    _ => continue,
                };
                let key = (sheet.Identity(), sheet.Revision());
                let (Some(typed), Some(rule_set), Some(routing)) = (
                    self.typed_sheets.get(&key),
                    self.translated_rule_sets.get(&key).cloned(),
                    self.translated_rule_routing.get(&key).cloned(),
                ) else {
                    continue;
                };
                if typed.contents.ParserContext().Mode() != expected_mode {
                    continue;
                }
                live_routing.insert(sheet_index, routing.clone());
                let scope_root = document.TreeScopeRoot(index);
                let implicit_scope_trigger =
                    if document.IsStyleSheetAdoptedByTreeScope(sheet.Identity(), scope_root) {
                        document.AdoptedStyleSheetImplicitScopeTrigger(sheet.Identity(), scope_root)
                    } else {
                        document
                            .FindNodeById(sheet.owner_node_id)
                            .and_then(|owner| document.ParentOrShadowHostElement(owner))
                    };
                direct.push((
                    sheet_index,
                    origin,
                    rule_set,
                    routing,
                    implicit_scope_trigger,
                ));
            }
            for origin in [CascadeOrigin::kUserAgent, CascadeOrigin::kAuthor] {
                // ProductionPseudoRequest carries one PseudoId. Treating
                // ::before::marker as a request for ::marker would match the
                // originating element's marker bucket and give it the wrong
                // cache owner. The compatibility selector below recognizes
                // the complete two-pseudo chain and keeps the declarations on
                // the generated ::before/::after style instead.
                if pseudo.is_some_and(PseudoStyleKind::IsNestedMarker) {
                    continue;
                }
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
                    .filter(|(_, candidate_origin, _, _, _)| *candidate_origin == origin)
                    .map(
                        |(sheet_index, _, rule_set, routing, implicit_scope_trigger)| {
                            crate::production_rule_matching::ProductionRuleSetInput {
                                sheet_index: *sheet_index,
                                rule_set: rule_set.clone(),
                                routing: routing.as_ref(),
                                implicit_scope_trigger: *implicit_scope_trigger,
                            }
                        },
                    )
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
                let authoritative_rules = inputs
                    .iter()
                    .map(|input| input.routing.AuthoritativeRuleCount())
                    .sum::<usize>();
                ConsumeProductionRuleCollection(
                    &mut self.translated_rule_shadow_stats,
                    sheets,
                    inputs.len(),
                    authoritative_rules,
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
                        Rc<crate::production_rule_matching::ProductionRuleRouting>,
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
                        let (Some(typed), Some(rule_set), Some(routing)) = (
                            self.typed_sheets.get(&key),
                            self.translated_rule_sets.get(&key).cloned(),
                            self.translated_rule_routing.get(&key).cloned(),
                        ) else {
                            continue;
                        };
                        if typed.contents.ParserContext().Mode()
                            != CSSParserMode::kHTMLStandardMode
                        {
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
                            routing,
                            implicit_scope_trigger,
                        ));
                    }
                    let inputs = owners
                        .iter()
                        .map(
                            |(sheet_index, rule_set, routing, implicit_scope_trigger)| {
                                crate::production_rule_matching::ProductionRuleSetInput {
                                    sheet_index: *sheet_index,
                                    rule_set: rule_set.clone(),
                                    routing: routing.as_ref(),
                                    implicit_scope_trigger: *implicit_scope_trigger,
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let authoritative_rules = inputs
                        .iter()
                        .map(|input| input.routing.AuthoritativeRuleCount())
                        .sum::<usize>();
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
                        authoritative_rules,
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
                    let element_scope = document.TreeScopeRoot(document.CorrespondingSVGElement(index).unwrap_or(index));
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
                        .filter(|(_, origin, _, _, _)| *origin == CascadeOrigin::kAuthor)
                        .map(
                            |(sheet_index, _, rule_set, routing, implicit_scope_trigger)| {
                                crate::production_rule_matching::ProductionRuleSetInput {
                                    sheet_index: *sheet_index,
                                    rule_set: rule_set.clone(),
                                    routing: routing.as_ref(),
                                    implicit_scope_trigger: *implicit_scope_trigger,
                                }
                            },
                        )
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
                    let authoritative_rules = inputs
                        .iter()
                        .map(|input| input.routing.AuthoritativeRuleCount())
                        .sum::<usize>();
                    ConsumeProductionRuleCollection(
                        &mut self.translated_rule_shadow_stats,
                        sheets,
                        inputs.len(),
                        authoritative_rules,
                        Some(document.TreeScopeRoot(index)),
                        collection,
                        &mut direct_rules,
                    );
                }
            }
        }
        let selector_session = self.selectors.borrow_mut().StyleMatchSession(document);
        let mut result = vec![];
        let mut layer_names = HashMap::<(u8, String), u16>::new();
        for &(sheet, origin) in sheets {
            if origin == CascadeOrigin::kUserAgent
                && !UserAgentSheetAppliesToDocument(sheet, document, media, node.Namespace())
            {
                continue;
            }
            for name in &sheet.layer_order {
                let next = layer_names.len() + 1;
                if next >= u16::MAX as usize {
                    return Err(DocumentStyleError::CascadeIndexOverflow);
                }
                layer_names
                    .entry((origin as u8, name.clone()))
                    .or_insert(next as u16);
            }
        }
        // The retained candidate index is built for the document resolver's
        // sheet sequence. Shadow TreeScopes have independent sheet indices and
        // must iterate their own scoped RuleSets.
        let document_scope = document.TreeScopeRoot(index) == document.Root();
        let had_index = document_scope && self.rule_candidate_index.is_some();
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let mut work = if had_index {
            let candidate_index = self.rule_candidate_index.as_ref().unwrap();
            candidate_index.targets[RuleCandidateIndex::Target(pseudo)]
                .Candidates(document, index, quirks)
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
        work.sort_unstable();
        work.dedup();
        for (order, sheet_index, rule_index) in work {
            let (sheet, origin) = sheets[sheet_index];
            if origin == CascadeOrigin::kUserAgent
                && !UserAgentSheetAppliesToDocument(sheet, document, media, node.Namespace())
            {
                continue;
            }
            let rule = &sheet.rules[rule_index];
            if owner_media_rejected.contains(&sheet_index) {
                continue;
            }
            let translated_owns_rule = !pseudo.is_some_and(PseudoStyleKind::IsNestedMarker)
                && live_routing
                    .get(&sheet_index)
                    .is_some_and(|routing| routing.IsAuthoritativeProjectedRule(rule_index));
            if translated_owns_rule {
                continue;
            }
            // The ordinary collection pass compiles every active rule and
            // records which pseudo target its selector addresses. A later
            // pseudo pass must not run ordinary selectors through media,
            // container and selector matching again. Blink's RuleSet keeps
            // these buckets separately for the same reason.
            if !had_index
                && pseudo.is_some()
                && !self.CompiledRuleHasTarget(
                    document,
                    &rule.selector_text,
                    &rule.scope_conditions,
                    pseudo,
                )
            {
                continue;
            }
            // cpp: RuleSet::AddRuleToRuleSet and
            // ElementRuleCollector::CollectMatchingRules.  Reject rules
            // whose strongest subject key is absent before evaluating
            // media/container state or entering SelectorChecker.
            if !had_index
                && !self.CompiledRuleMayMatchElement(
                    document,
                    index,
                    &rule.selector_text,
                    &rule.scope_conditions,
                    pseudo,
                )
            {
                continue;
            }
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
            if !media_matches {
                continue;
            }
            let (specificity, proximity) = {
                let Some(matched) = self.MatchingRuleSpecificity(
                    document,
                    index,
                    &rule.selector_text,
                    pseudo,
                    &rule.scope_conditions,
                    sheet.owner_node_id,
                    Some(&selector_session),
                ) else {
                    continue;
                };
                matched
            };
            let mode = if origin == CascadeOrigin::kUserAgent {
                CSSParserMode::kUASheetMode
            } else {
                DocumentParserMode(document)
            };
            // ParseCSS retained the production parser's CSSPropertyValueSet.
            // Author sheets in standards mode can feed that typed owner into
            // the real cascade directly. UA declarations require their UA
            // parser mode, quirks declarations require their document mode,
            // and unsupported diagnostics still need the legacy error detail;
            // those cases remain explicit compatibility fallbacks.
            let typed_properties = self
                .typed_sheets
                .get(&(sheet.Identity(), sheet.Revision()))
                .filter(|typed| typed.contents.ParserContext().Mode() == mode)
                .and_then(|typed| {
                    Some((
                        typed.TypedDeclarationsForProjectedRule(rule_index)?,
                        typed.RetainedDeclarationsForProjectedRule(rule_index)?,
                    ))
                });
            let (properties, retained_properties) =
                if let Some((properties, retained)) = typed_properties {
                    self.translated_rule_shadow_stats.typed_declarations_used += 1;
                    (properties, Some(retained))
                } else {
                    self.translated_rule_shadow_stats
                        .legacy_declaration_fallbacks += 1;
                    let declaration_key = (rule.declaration_text.clone(), mode as u8);
                    let properties =
                        if let Some(properties) = self.declaration_cache.get(&declaration_key) {
                            properties.clone()
                        } else {
                            let properties = Rc::new(Parse(
                                document,
                                index,
                                &rule.declaration_text,
                                mode,
                                &mut self.diagnostics,
                            ));
                            self.declaration_cache
                                .insert(declaration_key, properties.clone());
                            properties
                        };
                    (properties, None)
                };
            let layer = if rule.layer_name.is_empty() {
                u16::MAX
            } else {
                *layer_names
                    .get(&(origin as u8, rule.layer_name.clone()))
                    .ok_or(DocumentStyleError::Unsupported(
                        "CSSOM layer missing source order",
                    ))?
            };
            result.push(MatchedRule {
                specificity,
                proximity,
                order,
                origin,
                inline: false,
                layer,
                // RuleData applies pseudo valid-property filters only to
                // non-UA origins; Chromium's marker.css may set its complete
                // internal default set while author ::marker stays restricted
                // to the CSS Pseudo-Elements property subset.
                valid_property_filter: if origin == CascadeOrigin::kUserAgent {
                    ValidPropertyFilter::kNoFilter
                } else {
                    pseudo.map_or(ValidPropertyFilter::kNoFilter, |kind| {
                        kind.ValidPropertyFilter()
                    })
                },
                retained_properties,
                properties,
                tree_scope_root: (origin == CascadeOrigin::kAuthor)
                    .then(|| document.TreeScopeRoot(index)),
                transferred: None,
            });
        }
        if document_scope && !had_index {
            self.rule_candidate_index = Some(self.BuildRuleCandidateIndex(document, sheets));
        }
        // cpp: svg/svg_element.cc:574-592 and
        // svg/svg_transformable_element.cc:45-53. SVG presentation
        // attributes are author-cascade input below ordinary author rules.
        if pseudo.is_none() && document.Node(index).Namespace() == DOMNamespace::kSVG {
            let properties =
                crate::svg_presentation_attributes::CollectPresentationAttributes(document, index);
            if !properties.is_empty() {
                result.push(MatchedRule {
                    specificity: 0,
                    proximity: 0,
                    order: 0,
                    origin: CascadeOrigin::kAuthorPresentationalHint,
                    inline: false,
                    layer: u16::MAX,
                    valid_property_filter: ValidPropertyFilter::kNoFilter,
                    retained_properties: None,
                    properties: Rc::new(properties),
                    tree_scope_root: None,
                    transferred: None,
                });
            }
        }
        if let Some(inline) = document
            .Node(index)
            .FindAttribute("style")
            .filter(|_| pseudo.is_none())
        {
            result.push(MatchedRule {
                specificity: u32::MAX,
                proximity: u32::MAX,
                order: usize::MAX,
                origin: CascadeOrigin::kAuthor,
                inline: true,
                layer: u16::MAX,
                valid_property_filter: ValidPropertyFilter::kNoFilter,
                retained_properties: None,
                properties: Rc::new(Parse(
                    document,
                    index,
                    &inline.value,
                    DocumentParserMode(document),
                    &mut self.diagnostics,
                )),
                tree_scope_root: Some(document.TreeScopeRoot(index)),
                transferred: None,
            })
        }
        if result.is_empty() {
            if direct_rules.len() > u16::MAX as usize {
                return Err(DocumentStyleError::CascadeIndexOverflow);
            }
            debug_assert_eq!(
                direct_rules.len(),
                cascade_result.GetMatchedProperties().len()
            );
            // The ordinary author/typed-UA case reaches Cascade with the
            // exact MatchResult object populated by SortAndTransferMatchedRules.
            return Ok(CollectedRules {
                rules: direct_rules,
                match_result: cascade_result,
            });
        }
        result.extend(direct_rules);
        // cpp: ElementRuleCollector::SortMatchedRules and MatchResult::AddMatchedProperties.
        // Merge only the explicitly retained compatibility contexts into the
        // collector's transferred entries. The direct entries keep their exact
        // property-set, bindings and match-data ownership; no selector text or
        // projected CSSOM rule is replayed to create cascade input.
        result.sort_by_key(|r| {
            (
                r.origin as u8,
                r.layer,
                r.specificity,
                u16::MAX - r.proximity.min(u16::MAX as u32) as u16,
                r.order,
            )
        });
        if result.len() > u16::MAX as usize {
            return Err(DocumentStyleError::CascadeIndexOverflow);
        }
        cascade_result.Reset();
        let mut author_scope = None;
        for rule in &result {
            if rule.origin == CascadeOrigin::kAuthor && author_scope != rule.tree_scope_root {
                // Reset clears MatchResult's tree-scope table.  Register the
                // exact owner again when sorted compatibility entries force a
                // rebuild of the aggregate result.
                let root = rule
                    .tree_scope_root
                    .unwrap_or_else(|| document.TreeScopeRoot(index));
                cascade_result.BeginAddingAuthorRulesForTreeScope(Rc::new(
                    crate::production_element_rule_collector::ProductionTreeScope(root),
                ));
                author_scope = Some(root);
            }
            if let Some(transferred) = &rule.transferred {
                cascade_result.AddMatchedProperties(
                    transferred.properties.clone(),
                    transferred.mixin_parameter_bindings.clone(),
                    transferred.data_,
                );
            } else {
                AddRuleToMatchResult(&mut cascade_result, rule)?;
            }
        }
        Ok(CollectedRules {
            rules: result,
            match_result: cascade_result,
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
                Err(error) => self.diagnostics.push(DocumentStyleError::Selector(error)),
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
                    self.diagnostics.push(DocumentStyleError::Unsupported(
                        "pseudo style request or pseudo descendants",
                    ));
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
            if let Some(feature) =
                crate::production_rule_matching::ProductionUnsupportedParsedFeature(
                    &compiled_subject.CopySelectors(),
                )
            {
                self.diagnostics.push(DocumentStyleError::Selector(
                    PersistentSelectorError::Unsupported(format!(
                        "selector backend pending: :{feature}"
                    )),
                ));
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
    ) -> Result<Option<ResolvedPseudoOutput>, DocumentStyleError> {
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
        let mut builder = ComputedStyleBuilder::from_initial_and_parent(
            initial,
            parent,
            IsAtShadowBoundary::kNotAtShadowBoundary,
        );
        builder.SetStyleType(kind.Id());
        if has_nested_marker {
            builder.SetPseudoElementStyles(
                PseudoIdFlags::from_list(&[PseudoId::kPseudoIdMarker]).Bits(),
            );
        }
        let (_, computed_custom) = Self::Cascade(
            document,
            index,
            rules,
            &mut builder,
            Some(parent),
            None,
            Some(originating_custom),
            root,
            media,
            environment_variables,
            None,
            &self.property_registry,
            function_registry,
            self.color_scheme_settings,
            self.resolver_dependencies.text_link_text_color,
            &mut self.diagnostics,
            &mut self.translated_rule_shadow_stats,
        )?;
        if kind == PseudoStyleKind::Marker
            && parent.Display() != EDisplay::kInlineListItem
            && parent.ListStylePosition() == foundation::EListStylePosition::kOutside
        {
            // cpp: StyleAdjuster::AdjustStyleForMarker. Outside markers form
            // an inline-block, preserve their trailing separator space and
            // defer size-dependent outside margins to list layout.
            builder.SetDisplay(EDisplay::kInlineBlock);
            builder.SetWhiteSpace(EWhiteSpace::kPre);
        }
        AdjustComputedDisplayForLayout(&mut builder, parent, &DisplayAdjustmentContext::default());
        crate::resolver::style_adjuster::AdjustStyleForZIndex(&mut builder, parent);
        self.ResolveFontFeatureValues(&mut builder);
        let mut style = Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
        let native = unsafe { &*style.Get() };
        // cpp: Element::GetCachedPseudoElementStyle (element.cc:10376-10458).
        // Reuse is based on the complete native style, including content and
        // inherited fields, rather than the layout projection or cascade values.
        if let Some(old) = document.ResolvedStyleFor(index) {
            let cached = unsafe { &*old.native_style.Get() }
                .GetCachedPseudoElementStyleWithoutArgument(kind.Id());
            if unsafe { cached.as_ref() }.is_some_and(|old| {
                old == native && old.HasAttrFunction() == native.HasAttrFunction()
            }) {
                style = Persistent::from_ptr(cached as *mut ComputedStyle);
            }
        }
        let native = unsafe { &*style.Get() };
        // Retain computed style before box suppression. Chromium also retains
        // attr-dependent suppressed styles in SetAssociatedPseudoElement so
        // attribute changes invalidate their native cached style.
        parent.AddCachedPseudoElementStyle(style.Get(), kind.Id(), &foundation::g_null_atom);
        if native.Display() == EDisplay::kNone
            || (matches!(kind, PseudoStyleKind::Before | PseudoStyleKind::After)
                && native.ContentPreventsBoxGeneration())
            || (kind == PseudoStyleKind::Marker && native.ContentPreventsBoxGeneration())
        {
            return Ok(None);
        }
        Ok(Some(ResolvedPseudoOutput {
            pseudo: Self::ProjectPseudoElement(style)?,
            custom_properties: computed_custom,
        }))
    }

    fn ResolveNestedMarkerStyle(
        &mut self,
        document: &Document,
        index: usize,
        originating: &PseudoElement,
        originating_custom: &Rc<CustomProperties>,
        old_marker: Option<&PseudoElement>,
        initial: &ComputedStyle,
        root: f32,
        media: &MediaValuesCachedData,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        rules: &CollectedRules,
        function_registry: Option<&crate::production_style_sheet::ProductionFunctionRegistry>,
        has_layout_context: bool,
    ) -> Result<Option<PseudoElement>, DocumentStyleError> {
        let parent = unsafe { &*originating.native_style.Get() };
        if !has_layout_context
            || rules.rules.is_empty()
            || !parent.CanGeneratePseudoElement(PseudoId::kPseudoIdMarker)
        {
            return Ok(None);
        }
        let mut builder = ComputedStyleBuilder::from_initial_and_parent(
            initial,
            parent,
            IsAtShadowBoundary::kNotAtShadowBoundary,
        );
        builder.SetStyleType(PseudoId::kPseudoIdMarker);
        Self::Cascade(
            document,
            index,
            rules,
            &mut builder,
            Some(parent),
            None,
            Some(originating_custom),
            root,
            media,
            environment_variables,
            None,
            &self.property_registry,
            function_registry,
            self.color_scheme_settings,
            self.resolver_dependencies.text_link_text_color,
            &mut self.diagnostics,
            &mut self.translated_rule_shadow_stats,
        )?;
        if parent.Display() != EDisplay::kInlineListItem
            && parent.ListStylePosition() == foundation::EListStylePosition::kOutside
        {
            builder.SetDisplay(EDisplay::kInlineBlock);
            builder.SetWhiteSpace(EWhiteSpace::kPre);
        }
        AdjustComputedDisplayForLayout(&mut builder, parent, &DisplayAdjustmentContext::default());
        crate::resolver::style_adjuster::AdjustStyleForZIndex(&mut builder, parent);
        self.ResolveFontFeatureValues(&mut builder);
        let mut style = Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
        let native = unsafe { &*style.Get() };
        if let Some(old) =
            old_marker.and_then(|marker| unsafe { marker.native_style.Get().as_ref() })
        {
            if old == native && old.HasAttrFunction() == native.HasAttrFunction() {
                style = Persistent::from_ptr(old as *const ComputedStyle as *mut ComputedStyle);
            }
        }
        let native = unsafe { &*style.Get() };
        // The generated list item is the marker's originating element. Its
        // native style owns the marker cache and therefore the invalidation
        // edge, matching Element::GetCachedPseudoElementStyle for nested
        // generated pseudos.
        parent.AddCachedPseudoElementStyle(
            style.Get(),
            PseudoId::kPseudoIdMarker,
            &foundation::g_null_atom,
        );
        if native.Display() == EDisplay::kNone || native.ContentPreventsBoxGeneration() {
            return Ok(None);
        }
        Ok(Some(Self::ProjectPseudoElement(style)?))
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
    fn Cascade(
        document: &Document,
        node: usize,
        collected: &CollectedRules,
        builder: &mut ComputedStyleBuilder,
        parent: Option<&ComputedStyle>,
        root_style: Option<&ComputedStyle>,
        parent_custom: Option<&Rc<CustomProperties>>,
        root: f32,
        media: &MediaValuesCachedData,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        container_runtime: Option<
            &crate::resolver::production_container_runtime::ProductionContainerRuntime<'_>,
        >,
        registry: &crate::property_registry::PropertyRegistry,
        function_registry: Option<&crate::production_style_sheet::ProductionFunctionRegistry>,
        color_scheme_settings: production_style_builder::ColorSchemeSettings,
        text_link_text_color: Option<foundation::Color>,
        diagnostics: &mut Vec<DocumentStyleError>,
        translated_stats: &mut TranslatedRuleShadowStats,
    ) -> Result<(HashMap<CSSPropertyID, Rc<Value>>, Rc<CustomProperties>), DocumentStyleError> {
        let rules = collected.rules.as_slice();
        let mut applied = HashMap::new();
        // cpp: StyleResolverState::SetComputedStyleFlagsFromAuthorFlags.
        // These bits are semantic input to LayoutTheme: an author background
        // or border suppresses native button decoration. They must be derived
        // from the author match result, not guessed later from final colors.
        let author_flags = rules
            .iter()
            .filter(|rule| {
                matches!(
                    rule.origin,
                    CascadeOrigin::kAuthor | CascadeOrigin::kAuthorPresentationalHint
                )
            })
            .flat_map(|rule| rule.properties.iter())
            .filter(|property| property.PropertyID() != CSSPropertyID::kVariable)
            .fold(0, |flags, property| {
                flags | CSSProperty::Get(property.PropertyID()).GetFlags()
            });
        if author_flags & CSSProperty::kBackground != 0 {
            builder.SetHasAuthorBackground();
        }
        if author_flags & CSSProperty::kBorder != 0 {
            builder.SetHasAuthorBorder();
        }
        if author_flags & CSSProperty::kBorderRadius != 0 {
            builder.SetHasAuthorBorderRadius();
        }
        // cpp: style_resolver.cc:593-600,1767-1778. The document element has
        // a scheme-sensitive initial color at the lowest UA cascade priority.
        // Pseudo styles use this same node index with a parent style, so they
        // must keep normal color inheritance instead of resetting it here.
        let is_document_element = parent.is_none()
            && document
                .Node(document.Root())
                .Children()
                .iter()
                .copied()
                .find(|&index| document.Node(index).Type() == DOMNodeType::kElement)
                == Some(node);
        // Feed the parser-owned declaration sets through Blink's translated
        // MatchResult and cascade_expansion before applying computed values.
        // This preserves the StyleEngine boundary while removing the document
        // engine's former declaration-priority reimplementation.
        let match_result = &collected.match_result;
        let custom_winners =
            WinningCustomProperties(rules, match_result, builder.GetWritingDirection())?;
        let environment = CascadeEnvironment::new(environment_variables);
        let attribute_source = PersistentAttributeSource::new(
            document,
            node,
            document.Node(node).Namespace() == DOMNamespace::kHTML,
        );
        let attribute_state = CascadeAttributeState::new();
        // RuntimeEnabledFeatures::CSSArgumentGrammar is "test" in the
        // corresponding Chromium source, so the stable document path keeps
        // its legacy attr() first-argument grammar. The typed resolver still
        // owns both branches and can be enabled by the host later.
        let attributes = AttributeResolver::new(&attribute_source, &attribute_state, false);
        let substitution = SubstitutionContext {
            environment: Some(&environment),
            attributes: Some(&attributes),
        };
        let image_resolver = DocumentURLImageResolver {
            document,
            originating_index: node,
        };
        let compute_custom = |font: f32| {
            let length = |n, u| {
                production_style_builder::Pixels(CSSPropertyID::kVariable, n, u, font, root, media)
                    .map_err(|_| {
                        VariableResolutionError::Unsupported(
                            "registered property length conversion",
                        )
                    })
            };
            // Computed custom properties inherit by shared identity in Blink.
            // A non-inherited registered property only requires a new map at
            // the boundary where the parent differs from its initial value.
            // Once reset, descendants without their own declaration retain
            // that immutable map just like Blink's StyleInheritedVariables.
            if custom_winners.is_empty() {
                if let Some(parent) = parent_custom.filter(|parent| {
                    registry.Registrations().all(|(name, registration)| {
                        if registration.inherits {
                            return true;
                        }
                        let expected_value = match registration.initial.as_ref() {
                            Some(initial) => {
                                let Ok(value) = crate::property_registry::ComputeRegisteredValue(
                                    initial, &length,
                                ) else {
                                    return false;
                                };
                                Some(value)
                            }
                            None => None,
                        };
                        let typed_matches =
                            match (expected_value.as_ref(), parent.registered_values.get(name)) {
                                (Some(expected), Some(actual)) => {
                                    expected.as_ref() == actual.as_ref()
                                }
                                (None, None) => true,
                                _ => false,
                            };
                        if !typed_matches {
                            return false;
                        }
                        let expected_data = expected_value
                            .as_ref()
                            .zip(registration.initial_data.as_ref())
                            .map(|(value, data)| {
                                crate::property_registry::RegisteredVariableData(value, data)
                            });
                        match (expected_data.as_ref(), parent.values.get(name)) {
                            (Some(expected), Some(ComputedVariableValue::Data(actual))) => {
                                Rc::ptr_eq(expected, actual)
                                    || expected.original_text == actual.original_text
                                        && expected.features == actual.features
                                        && expected.is_animation_tainted
                                            == actual.is_animation_tainted
                                        && expected.is_attr_tainted == actual.is_attr_tainted
                            }
                            (None, None | Some(ComputedVariableValue::Invalid)) => true,
                            _ => false,
                        }
                    })
                }) {
                    return parent.clone();
                }
                if parent_custom.is_none() && registry.IsEmpty() {
                    return Rc::new(CustomProperties::default());
                }
            }
            Rc::new(CustomProperties::ComputeWithRegistry(
                parent_custom.map(Rc::as_ref),
                custom_winners.iter().copied(),
                substitution,
                Some(registry),
                &length,
            ))
        };
        let mut custom = compute_custom(builder.GetFontDescription().ComputedSize());
        let debug_style_class = std::env::var("RECHROM_DEBUG_STYLE_CLASS").ok();
        let debug_style = debug_style_class.as_deref().is_some_and(|needle| {
            document
                .Node(node)
                .FindAttribute("class")
                .is_some_and(|classes| {
                    classes
                        .value
                        .split_ascii_whitespace()
                        .any(|class_name| class_name.contains(needle))
                })
        });
        if debug_style {
            eprintln!(
                "STYLE_DEBUG node={node} tag={} rules={}",
                document.Node(node).Name(),
                rules.len()
            );
            for name in [
                "--brand-button-subtle-borderColor-rest",
                "--base-color-scale-gray-2",
                "--brand-button-subtle-bgColor-rest",
                "--brand-borderWidth-thin",
            ] {
                eprintln!(
                    "STYLE_DEBUG custom {name}={:?}",
                    custom.Get(name).map(|data| data.Serialize().Utf8())
                );
            }
            for rule in rules {
                for property in rule.properties.iter() {
                    eprintln!(
                        "STYLE_DEBUG declaration {:?}={} origin={:?}",
                        property.PropertyID(),
                        property.Value().CssText().Utf8(),
                        rule.origin
                    );
                }
            }
        }
        for (_, error) in &custom.diagnostics {
            if let VariableResolutionError::Unsupported(operation) = error {
                diagnostics.push(DocumentStyleError::Unsupported(operation));
            }
        }
        // The direct collector's parser-owned property sets can now continue
        // through the translated StyleCascade itself.  Run it against an
        // isolated native-style snapshot: a typed backend fallback discards
        // that whole trial and leaves the existing cascade below as the sole
        // writer.  Selector matching and MatchResult construction are never
        // replayed. The adapter inserts the document element's source-defined
        // initial-color block into this same MatchResult and gives the root an
        // InitialStyleForElement parent before StyleCascade applies.
        let has_translated_property_set = match_result.GetMatchedProperties().iter().any(|entry| {
            entry
                .properties
                .AsAny()
                .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
                .is_some()
        });
        let all_property_sets_translated =
            match_result.GetMatchedProperties().iter().all(|entry| {
                entry
                    .properties
                    .AsAny()
                    .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
                    .is_some()
            });
        let is_pseudo = builder.StyleType() != PseudoId::kPseudoIdNone;
        let animation_samples = (!is_pseudo)
            .then(|| document.AnimationStyles(document.Node(node).Id()))
            .filter(|effects| !effects.is_empty());
        let (animation_interpolations, animation_parse_failed) = match animation_samples {
            Some(effects) => {
                match crate::resolver::production_style_cascade::ParseAnimationInterpolations(
                    effects,
                    DocumentParserMode(document),
                ) {
                    Ok(map) => (Some(map), false),
                    Err(fallbacks) => {
                        if fallbacks.iter().any(|fallback| matches!(fallback,
                        crate::resolver::production_style_cascade::ProductionCascadeFallback::UnsupportedFeature(operation)
                            if *operation == "animation-sample-parse"
                    )) {
                        diagnostics.push(DocumentStyleError::Unsupported("animation-sample-parse"));
                    }
                        (None, true)
                    }
                }
            }
            None => (None, false),
        };
        let transition_samples = (!is_pseudo)
            .then(|| document.TransitionStyles(document.Node(node).Id()))
            .filter(|effects| !effects.is_empty());
        let (transition_interpolations, transition_parse_failed) = match transition_samples {
            Some(effects) => {
                match crate::resolver::production_style_cascade::ParseTransitionInterpolations(
                    effects,
                    DocumentParserMode(document),
                ) {
                    Ok(map) => (Some(map), false),
                    Err(fallbacks) => {
                        if fallbacks.iter().any(|fallback| matches!(fallback,
                            crate::resolver::production_style_cascade::ProductionCascadeFallback::UnsupportedFeature(operation)
                                if *operation == "transition-sample-parse"
                        )) {
                            diagnostics
                                .push(DocumentStyleError::Unsupported("transition-sample-parse"));
                        }
                        (None, true)
                    }
                }
            }
            None => (None, false),
        };
        let has_animations = animation_samples.is_some() || transition_samples.is_some();
        let registry_dependent = !registry.IsEmpty()
            && rules
                .iter()
                .flat_map(|rule| rule.properties.iter())
                .any(|property| {
                    property.PropertyID() == CSSPropertyID::kVariable
                        || property.Value().IsUnparsedDeclaration()
                });
        if has_translated_property_set || has_animations {
            translated_stats.translated_style_cascade_attempts += 1;
            if is_document_element {
                translated_stats.translated_style_cascade_root_attempts += 1;
            }
            if is_pseudo {
                translated_stats.translated_style_cascade_pseudo_attempts += 1;
            }
            if registry_dependent {
                translated_stats.translated_style_cascade_registered_attempts += 1;
            }
            if has_animations {
                translated_stats.translated_style_cascade_animation_attempts += 1;
            }
            if !all_property_sets_translated || animation_parse_failed || transition_parse_failed {
                translated_stats.translated_style_cascade_fallbacks += 1;
                if is_document_element {
                    translated_stats.translated_style_cascade_root_fallbacks += 1;
                }
                if is_pseudo {
                    translated_stats.translated_style_cascade_pseudo_fallbacks += 1;
                }
                if registry_dependent {
                    translated_stats.translated_style_cascade_registered_fallbacks += 1;
                }
                if has_animations {
                    translated_stats.translated_style_cascade_animation_fallbacks += 1;
                }
            } else {
                let base = Persistent::from_ptr(builder.CloneStyle() as *mut ComputedStyle);
                let parent_handle = parent.map(|parent| {
                    Persistent::from_ptr(parent as *const ComputedStyle as *mut ComputedStyle)
                });
                let root_handle = root_style.map(|style| {
                    Persistent::from_ptr(style as *const ComputedStyle as *mut ComputedStyle)
                });
                let pseudo_id = builder.StyleType();
                match crate::resolver::production_style_cascade::ApplyMatchResult(
                    crate::resolver::production_style_cascade::ProductionCascadeInputs {
                        document: Some(document),
                        media: media.clone(),
                        base,
                        parent: parent_handle.clone(),
                        root_style: root_handle,
                        match_result: match_result.clone(),
                        root_font_size: root,
                        color_scheme_settings,
                        text_link_text_color,
                        is_document_element,
                        pseudo_id,
                        originating_style: (pseudo_id != PseudoId::kPseudoIdNone)
                            .then_some(parent_handle)
                            .flatten(),
                        subject_node: Some(node),
                        container_runtime,
                        custom_properties: Some(custom.as_ref()),
                        parent_custom_properties: parent_custom.map(Rc::as_ref),
                        substitution_context: substitution,
                        image_resolver: Some(&image_resolver),
                        property_registry: Some(registry),
                        function_registry,
                        animations: animation_interpolations.as_ref(),
                        transitions: transition_interpolations.as_ref(),
                    },
                ) {
                    Ok(output) => {
                        *builder =
                            ComputedStyleBuilder::from_style(unsafe { &*output.style.Get() });
                        translated_stats.translated_style_cascade_applied += 1;
                        if is_document_element {
                            translated_stats.translated_style_cascade_root_applied += 1;
                        }
                        if is_pseudo {
                            translated_stats.translated_style_cascade_pseudo_applied += 1;
                        }
                        if registry_dependent {
                            translated_stats.translated_style_cascade_registered_applied += 1;
                        }
                        if has_animations {
                            translated_stats.translated_style_cascade_animation_applied += 1;
                        }
                        environment.ApplyState(builder);
                        attribute_state.ApplyState(builder);
                        // `cascade_values` is currently retained only as an
                        // internal write-only cache; the canonical public result
                        // is this native ComputedStyle. If that cache gains a
                        // reader, export resolved winners from StyleCascade rather
                        // than reconstructing them here.
                        return Ok((HashMap::new(), output.custom_properties));
                    }
                    Err(fallbacks) => {
                        if std::env::var_os("RECHROM_DEBUG_STYLE_CASCADE").is_some() {
                            eprintln!(
                                "translated StyleCascade fallback node={node} root={is_document_element} pseudo={pseudo_id:?}: {fallbacks:?}"
                            );
                        }
                        translated_stats.translated_style_cascade_fallbacks += 1;
                        if is_document_element {
                            translated_stats.translated_style_cascade_root_fallbacks += 1;
                        }
                        if is_pseudo {
                            translated_stats.translated_style_cascade_pseudo_fallbacks += 1;
                        }
                        if registry_dependent {
                            translated_stats.translated_style_cascade_registered_fallbacks += 1;
                        }
                        if has_animations {
                            translated_stats.translated_style_cascade_animation_fallbacks += 1;
                        }
                    }
                }
            }
        }
        // cpp: style_cascade.cc:AnalyzeMatchResult/ApplyHighPriority/ApplyRemaining.
        // Direction-aware properties are mapped only after direction/writing-mode.
        for high in [true, false] {
            // Font-relative registered substitution data belongs to this
            // element's computed font, and remains absolute on inheritance.
            if !high && registry.Registrations().next().is_some() {
                custom = compute_custom(builder.GetFontDescription().ComputedSize());
            }
            let map = CollectProductionCascadeMap(match_result, builder.GetWritingDirection());
            let mut ids = map
                .NativeBitset()
                .begin()
                .filter(|id| {
                    matches!(
                        id,
                        CSSPropertyID::kDirection
                            | CSSPropertyID::kWritingMode
                            | CSSPropertyID::kZoom
                            | CSSPropertyID::kFontSize
                            | CSSPropertyID::kFontWeight
                            | CSSPropertyID::kFontStyle
                            | CSSPropertyID::kFontFamily
                    ) == high
                })
                .collect::<Vec<_>>();
            if !high && is_document_element {
                // ColorScheme sorts before Color. Schedule the native initial
                // color even without a declaration, then let any real winner
                // override it. No synthetic CSS value crosses a crate boundary.
                if !ids.contains(&CSSPropertyID::kColor) {
                    ids.push(CSSPropertyID::kColor);
                }
            }
            ids.sort_by_key(|id| match id {
                CSSPropertyID::kDirection => 0,
                CSSPropertyID::kWritingMode => 1,
                // cpp: style_cascade.cc:625-638 applies zoom before the
                // high-priority font properties because it changes their
                // conversion data and computed metrics.
                CSSPropertyID::kZoom => 2,
                CSSPropertyID::kFontFamily => 3,
                CSSPropertyID::kFontSize => 4,
                _ => *id as i32 + 10,
            });
            for id in ids {
                let name = CSSPropertyName::new(id);
                let mut priority = map.At(&name);
                if id == CSSPropertyID::kColor && is_document_element {
                    let color = builder.InitialColorForColorScheme();
                    builder.SetColor(&color);
                    builder.SetColorIsInherited(false);
                    builder.SetColorIsCurrentColor(false);
                    if !priority.HasOrigin() {
                        continue;
                    }
                }
                let mut value;
                let mut source_id;
                loop {
                    let source_property =
                        &rules[priority.GetRuleIndex()].properties[priority.GetDeclarationIndex()];
                    source_id = source_property.PropertyID();
                    value = source_property.ValueRef();
                    let fallback = if value.IsRevertValue() {
                        let origin = match priority.GetOrigin() {
                            CascadeOrigin::kAuthor | CascadeOrigin::kAuthorPresentationalHint => {
                                CascadeOrigin::kUser
                            }
                            CascadeOrigin::kUser => CascadeOrigin::kUserAgent,
                            _ => CascadeOrigin::kNone,
                        };
                        map.FindForOrigin(&name, origin)
                    } else if value.IsRevertLayerValue() {
                        map.FindRevertLayer(&name, priority.ForLayerComparison())
                    } else {
                        break;
                    };
                    if let Some(next) = fallback {
                        priority = *next
                    } else {
                        value = crate::production_css_value::wide(foundation::CSSValueID::kUnset)
                            .unwrap();
                        break;
                    }
                }
                let value = match ResolvePropertyValueWithContext(
                    source_id,
                    &value,
                    custom.as_ref(),
                    substitution,
                ) {
                    Ok(value) => value,
                    Err(error) => {
                        diagnostics.push(DocumentStyleError::Property {
                            node,
                            property: error.property,
                            offset: error.offset,
                            operation: error.operation,
                            source_file: error.source_file,
                            source_line: error.source_line,
                        });
                        continue;
                    }
                };
                if debug_style {
                    eprintln!(
                        "STYLE_DEBUG winner target={id:?} source={source_id:?} value={}",
                        value.CssText().Utf8()
                    );
                }
                if let Err(error) =
                    production_style_builder::ApplyWithImageResolverAndColorSchemeSettings(
                        id,
                        builder,
                        parent,
                        &value,
                        root,
                        media,
                        &image_resolver,
                        color_scheme_settings,
                    )
                {
                    diagnostics.push(DocumentStyleError::Application { node, error });
                } else {
                    applied.insert(id, value);
                }
            }
        }
        if debug_style {
            eprintln!(
                "STYLE_DEBUG final border_top_style={:?} border_top_width={} author_border={} font_smoothing={:?} font_size={} font_weight={} zoom={} effective_zoom={}",
                builder.BorderTopStyle(),
                builder.BorderTopWidth(),
                builder.HasAuthorBorder(),
                builder.GetFontDescription().FontSmoothing(),
                builder.GetFontDescription().ComputedSize(),
                builder.GetFontDescription().Weight().ToFloat(),
                builder.Zoom(),
                builder.EffectiveZoom(),
            );
        }
        environment.ApplyState(builder);
        attribute_state.ApplyState(builder);
        Ok((applied, custom))
    }
}

// SVGResources::GetClient + LayoutSVGResourcePaintServer::ApplyShader own this
// dependency in Chromium.  Rechrom's StyleEngine freezes the same local
// resource result into its existing typed paint payload.  Scan URI consumers
// after the cascade so a changed <stop> also refreshes unchanged referencing
// shapes, matching SVGGradientElement::InvalidateDependentGradients.
fn ResolveSVGPaintServers(
    document: &Document,
    pending: &mut Vec<(usize, ResolvedNodeStyle)>,
    stats: &mut StyleUpdateStats,
    impact: &mut StyleUpdateImpact,
) {
    let snapshot = pending
        .iter()
        .map(|(node, style)| (*node, style.clone()))
        .collect::<HashMap<_, _>>();
    let mut positions = pending
        .iter()
        .enumerate()
        .map(|(position, (node, _))| (*node, position))
        .collect::<HashMap<_, _>>();
    for node in 0..document.NodeCount() {
        let Some(current) = snapshot
            .get(&node)
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
        if !mask_definition && fill_reference.is_none()
            && stroke_reference.is_none()
            && marker_start.is_none()
            && marker_mid.is_none()
            && marker_end.is_none()
        {
            continue;
        }
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
        let mask_graph = mask_definition.then(|| crate::svg_paint_server::ResolveMaskDefinition(document, &snapshot, node));
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
        impact.Merge(StyleImpact(document.ResolvedStyleFor(node), &updated));
        if let Some(&position) = positions.get(&node) {
            pending[position].1 = updated;
        } else {
            positions.insert(node, pending.len());
            pending.push((node, updated));
            stats.changed_nodes += 1;
        }
    }
}

fn AddRuleToMatchResult(
    result: &mut MatchResult,
    rule: &MatchedRule,
) -> Result<(), DocumentStyleError> {
    if rule.properties.len() > u16::MAX as usize {
        return Err(DocumentStyleError::CascadeIndexOverflow);
    }
    let properties: Rc<dyn MatchedPropertySet> = match &rule.retained_properties {
        Some(properties) => properties.clone(),
        None => Rc::new(LegacyMatchedPropertySet(rule.properties.clone())),
    };
    let mut data = MatchedPropertiesData::default();
    data.origin = rule.origin;
    data.layer_order = rule.layer;
    data.set_is_inline_style(rule.inline);
    data.set_valid_property_filter(rule.valid_property_filter as u8);
    result.AddMatchedProperties(properties, None, data);
    Ok(())
}

fn CollectProductionCascadeMap(
    result: &MatchResult,
    direction: foundation::WritingDirectionMode,
) -> CascadeMap {
    let map = RefCell::new(CascadeMap::new());
    let backend = DocumentCascadeExpansionBackend;
    for (index, matched) in result.GetMatchedProperties().iter().enumerate() {
        cascade_expansion::ExpandCascade(
            matched,
            &(),
            index,
            &backend,
            |priority, name| map.borrow_mut().AddCustom(name.clone(), priority),
            |priority, id| {
                // cpp: StyleCascade::ResolveSurrogate. CascadeMap contains
                // physical properties only and asserts this invariant.
                let id = ResolvePhysical(id, direction);
                debug_assert!(
                    !CSSProperty::Get(id).IsSurrogate(),
                    "production cascade left surrogate property unresolved: {id:?}"
                );
                map.borrow_mut().Add(id, priority);
            },
        );
    }
    map.into_inner()
}

// cpp: style_cascade.cc:AnalyzeMatchResult/ResolveCustomProperty.
// Select custom-property winners with the same origin/layer ordering as native
// properties before var() substitution computes the inherited custom map.
fn WinningCustomProperties<'a>(
    rules: &'a [MatchedRule],
    match_result: &MatchResult,
    direction: foundation::WritingDirectionMode,
) -> Result<Vec<&'a PropertyValue>, DocumentStyleError> {
    let map = CollectProductionCascadeMap(match_result, direction);
    let mut names = map.GetCustomMap().keys().cloned().collect::<Vec<_>>();
    names.sort_by_key(|name| name.Utf8());
    let mut winners = Vec::with_capacity(names.len());
    for custom_name in names {
        let name = CSSPropertyName::custom(custom_name);
        let mut priority = map.At(&name);
        loop {
            let property =
                &rules[priority.GetRuleIndex()].properties[priority.GetDeclarationIndex()];
            let value = property.ValueRef();
            let fallback = if value.IsRevertValue() {
                let origin = match priority.GetOrigin() {
                    CascadeOrigin::kAuthor | CascadeOrigin::kAuthorPresentationalHint => {
                        CascadeOrigin::kUser
                    }
                    CascadeOrigin::kUser => CascadeOrigin::kUserAgent,
                    _ => CascadeOrigin::kNone,
                };
                map.FindForOrigin(&name, origin)
            } else if value.IsRevertLayerValue() {
                map.FindRevertLayer(&name, priority.ForLayerComparison())
            } else {
                winners.push(property);
                break;
            };
            let Some(next) = fallback else {
                // Custom properties inherit by default. With no lower origin,
                // revert resolves to the inherited/initial state represented by
                // the absence of a specified winner.
                break;
            };
            priority = *next;
        }
    }
    Ok(winners)
}

fn Parse(
    _document: &Document,
    node: usize,
    text: &str,
    mode: CSSParserMode,
    diagnostics: &mut Vec<DocumentStyleError>,
) -> Vec<PropertyValue> {
    let parsed = ParseDeclarationList(&foundation::String::from(text), mode);
    for e in parsed
        .errors
        .iter()
        .filter(|e| e.kind == PropertyParseErrorKind::Unsupported)
    {
        diagnostics.push(DocumentStyleError::Property {
            node,
            property: e.property,
            offset: e.offset,
            operation: e.operation,
            source_file: e.source_file,
            source_line: e.source_line,
        });
    }
    parsed.properties
}

// The layout boundary rebinds the native font to ConstraintSpace's font faces.
// It must receive the computed description from the same native style.
// cpp: text_control_inner_elements.cc:133-222. The retained inner editor is a
// real DOM owner with a custom-style callback. It is intentionally applied
// after the ordinary cascade because Blink's callback constructs the final
// editing style from the host rather than exposing the implementation element
// to author declarations.
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
        // No LayoutObject is constructed for either value; own_generates_box
        // and display_contents preserve their structural distinction.
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
    resolved.style.extended = Some(ext);
    Ok(())
}

fn DocumentParserMode(document: &Document) -> CSSParserMode {
    if document.GetCompatibilityMode() == dom::persistent_document::CompatibilityMode::kQuirks {
        CSSParserMode::kHTMLQuirksMode
    } else {
        CSSParserMode::kHTMLStandardMode
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

    fn ForSheets(document: &Document, sheets: &[(CSSStyleSheet, CascadeOrigin)]) -> Self {
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
        if let Some(list) = selector.SelectorList() {
            for complex in list.ComplexSelectors() {
                for selector in complex.SimpleSelectors() {
                    self.Add(selector);
                }
            }
        }
    }
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
fn ChangedSheets(
    old: &[(CSSStyleSheet, CascadeOrigin)],
    new: &[(CSSStyleSheet, CascadeOrigin)],
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
fn StyleImpact(old: Option<&ResolvedNodeStyle>, new: &ResolvedNodeStyle) -> StyleUpdateImpact {
    let Some(old) = old else {
        return StyleUpdateImpact::TREE;
    };
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
                let fields = ComputedStyleBase::FieldInvalidationDiff(
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
            _ => return StyleUpdateImpact::TREE,
        }
    }
    if old.generates_box != new.generates_box
        || old.own_generates_box != new.own_generates_box
        || old.own_display_contents != new.own_display_contents
        || old.style.display != new.style.display
        || old.style.position != new.style.position
        || old.style.floating != new.style.floating
    {
        return StyleUpdateImpact::TREE;
    }
    let old_native = unsafe { &*old.native_style.Get() };
    let new_native = unsafe { &*new.native_style.Get() };
    let native_difference = ComputedStyle::ComputeDifference(old_native, new_native);
    let fields = ComputedStyleBase::FieldInvalidationDiff(old_native, new_native);
    // cpp: ComputedStyle::VisualInvalidationDiff, computed_style.cc:753-757.
    // A reshape changes intrinsic advances and therefore requires layout even
    // when the projected font size/family/weight fields remain unchanged.
    let layout = fields & (FieldDifference::kLayout.bits() | FieldDifference::kReshape.bits()) != 0
        || !old.style.LayoutEquivalent(&new.style);
    StyleUpdateImpact {
        reattach: false,
        layout: pseudo_impact.layout || layout,
        paint: pseudo_impact.paint
            || native_difference != ComputedStyleDifference::kEqual
            || fields != 0
            || old.style.paint != new.style.paint
            || layout,
    }
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
    if a.values.len() != b.values.len() {
        return false;
    }
    a.values.iter().all(|(name, left)| {
        let Some(right) = b.values.get(name) else {
            return false;
        };
        crate::resolver::production_style_builder::custom_properties::ComputedVariablesEquivalent(left, right)
    })
}

fn FinishUpdate(document: &mut Document, stats: StyleUpdateStats, impact: StyleUpdateImpact) {
    let state = document.StyleStateMut();
    state.all_dirty = false;
    state.changes.clear();
    state.sibling_sensitive_nodes.clear();
    state.descendant_sensitive_nodes.clear();
    state.append_only_children.clear();
    state.non_append_children.clear();
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

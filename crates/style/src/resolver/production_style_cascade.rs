// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Production adapter for the translated `StyleCascade::Apply` path.
#![allow(non_snake_case, dead_code, unused_variables)]

use crate::css_math_expression_node::{
    CSSMathLengthResolver, CalculationResultCategory, MathError,
};
use crate::css_math_function_value::ValueRange;
use crate::css_primitive_value::{ConversionToCanonicalUnitsScaleFactor, UnitType};
use crate::css_value_clamping_utils::CSSValueClampingUtils;
use crate::kleene_value::KleeneValue;
use crate::{
    css_property_name::CSSPropertyName,
    css_property_value::{CSSPropertyValue, CSSPropertyValueBackend},
    css_property_value_set::{CSSPropertyValueSetRuleHandle, ImmutableCSSPropertyValueSet},
    css_syntax_definition::CSSSyntaxDefinition,
    css_value::{CSSValueDispatch, CSSValuePairSubclass, CSSValuePayload},
    media_queries::{
        conditional_exp_node::ConditionalExpNodeVisitor,
        media_query_evaluator::MediaQueryEvaluator,
        media_query_exp::{MediaQueryExp, MediaQueryExpBounds, MediaQueryOperator},
        media_values_cached::MediaValuesCached,
        MediaValuesCachedData,
    },
    parser::{
        css_if_parser::CSSIfParser,
        css_parser_context::{CSSParserContext, StrictCSSParserContext},
        css_parser_local_context::CSSParserLocalContext,
        css_parser_mode::CSSParserMode,
        css_parser_token::CSSParserToken,
        css_parser_token_stream::CSSParserTokenStream,
        css_tokenizer::CSSTokenizer,
        production_property_parser::{
            ParseCustomProperty, ParseDeclarationList, VariableTokenReplay,
        },
    },
    production_css_value::{self, CSSVariableData, ProductionCSSValueDispatch},
    production_style_sheet::{self, Backend as ProductionSheetBackend, QuerySet},
    properties::{
        css_property::{CSSProperty, Flags},
        longhand_dispatch::ResolvePhysical,
    },
    property_registry::PropertyRegistry,
    resolver::{
        cascade_expansion::{self, CascadeExpansionBackend},
        cascade_filter::CascadeFilter,
        cascade_origin::CascadeOrigin,
        cascade_priority::CascadePriority,
        match_result::{
            MatchResult, MatchedProperties, MatchedPropertiesData, MatchedPropertySet,
            MixinParameterBindings, TreeScope,
        },
        production_container_runtime::{
            ApplyContainerUnitDependencies, ProductionContainerChange, ProductionContainerRuntime,
            ProductionContainerSelector,
        },
        production_style_builder::{
            self,
            custom_properties::{
                attributes::{AttributeType, ConsumeAttributeType, ParseAttributeValue},
                CustomProperties, SubstitutionContext, VariableResolutionError,
            },
            URLImageResolver,
        },
        style_cascade::{FunctionRuleKind, IfConditionVisitor, StyleCascade, StyleCascadeBackend},
        style_resolver::StyleResolverBackend,
        style_resolver_state::{
            ComputedStyleHandle, ResolverLengthConversionData, ResolverURIValue, ResolverValue,
            StyleResolverRequestType, StyleResolverState, StyleResolverStateBackend,
            StyleResolverStateFontBuilder, ValueResolutionMode,
        },
    },
    ColorSchemeFlags, PreferredColorScheme,
};
use crate::{
    if_condition::{IfCondition, IfConditionEvaluator},
    production_container_parser::{ContainerCondition, ContainerSet},
};
use dom::persistent_document::{DOMInputType, DOMNamespace};
use font_engine::{FontDescription, LayoutLocale};
use foundation::{
    gfx, AtomicString, CSSBitset, CSSPropertyID, CSSValueID, EInsideLink, EOrder, EOverflow,
    EScrollbarWidth, EUserModify, Length, LengthValueRange, Persistent, String, StringView,
    WritingDirectionMode, WritingMode,
};
use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::{
    css::style_color::StyleColor,
    style::{
        computed_style::{ComputedStyle, ComputedStyleBuilder},
        computed_style_base::IsAtShadowBoundary,
        computed_style_constants::PseudoId,
        computed_style_initial_values::ComputedStyleInitialValues,
        default_anchor_data::DefaultAnchorData,
        display_style::DisplayStyle,
        font_size_style::FontSizeStyle,
        gap_data_list::GapDataList,
        position_area::PositionAreaOffsets,
        style_image::StyleImage,
    },
};
use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::{BTreeMap, HashMap},
    rc::Rc,
    sync::Arc,
};

type ParserContext<B> = CSSParserContext<<B as StyleCascadeBackend>::ParserPlatform>;
type TokenStream<'a> = CSSParserTokenStream<'a, CSSTokenizer>;
type Variable<B> = Rc<<B as StyleCascadeBackend>::VariableData>;
type VariableMap<B> = HashMap<AtomicString, Option<Variable<B>>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProductionCascadeFallback {
    UnsupportedFeature(&'static str),
    UnsupportedProperty(CSSPropertyID),
    InvalidProperty(CSSPropertyID),
}

/// Style-private representation of Blink's ActiveInterpolationsMap.  The
/// document/animation boundary supplies already sampled replacement
/// declarations; this owner retains their production-parser CSSValues and
/// lets the translated StyleCascade establish animation/transition priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProductionInterpolationKind {
    Invalidatable,
    Transition,
}

pub(crate) struct ProductionActiveInterpolations {
    property: CSSPropertyName,
    kind: ProductionInterpolationKind,
    // Low-to-high composite order. The current cross-package sample contract
    // can represent replacement samples only, so the last effect is the value
    // produced by InvalidatableInterpolation::ApplyStack.
    values: Vec<Rc<production_css_value::Value>>,
}

#[derive(Clone, Default)]
pub(crate) struct ProductionActiveInterpolationsMap {
    entries: Vec<(CSSPropertyName, Rc<ProductionActiveInterpolations>)>,
}

impl ProductionActiveInterpolationsMap {
    fn FromEffectValues(
        effects: impl IntoIterator<Item = Vec<CSSPropertyValue<ProductionCSSValueDispatch>>>,
        kind: ProductionInterpolationKind,
    ) -> Self {
        let mut stacks = Vec::<(CSSPropertyName, Vec<Rc<production_css_value::Value>>)>::new();
        for effect in effects {
            // A keyframe effect contributes at most one sampled replacement
            // per expanded longhand. Later declarations in the effect win.
            let mut effect_values =
                Vec::<(CSSPropertyName, Rc<production_css_value::Value>)>::new();
            for property in effect {
                let name = property.Name();
                if let Some((_, value)) = effect_values
                    .iter_mut()
                    .find(|(candidate, _)| candidate == &name)
                {
                    *value = property.ValueRef();
                } else {
                    effect_values.push((name, property.ValueRef()));
                }
            }
            for (name, value) in effect_values {
                if let Some((_, values)) =
                    stacks.iter_mut().find(|(candidate, _)| candidate == &name)
                {
                    values.push(value);
                } else {
                    stacks.push((name, vec![value]));
                }
            }
        }
        Self {
            entries: stacks
                .into_iter()
                .map(|(property, values)| {
                    let interpolations = Rc::new(ProductionActiveInterpolations {
                        property: property.clone(),
                        kind,
                        values,
                    });
                    (property, interpolations)
                })
                .collect(),
        }
    }

    #[cfg(test)]
    fn FromParsedProperties(
        properties: Vec<CSSPropertyValue<ProductionCSSValueDispatch>>,
        kind: ProductionInterpolationKind,
    ) -> Self {
        Self::FromEffectValues([properties], kind)
    }

    pub(crate) fn IsEmpty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Converts the existing document-owned animation samples to the typed owner
/// consumed by StyleCascade. No CSSOM projection is treated as a computed
/// value: every declaration is accepted again by the production parser.
pub(crate) fn ParseAnimationInterpolations(
    effects: &BTreeMap<u64, Vec<cssom::CSSDeclaration>>,
    mode: CSSParserMode,
) -> Result<ProductionActiveInterpolationsMap, Vec<ProductionCascadeFallback>> {
    ParseSampledInterpolations(
        effects,
        mode,
        ProductionInterpolationKind::Invalidatable,
        "animation-sample-parse",
    )
}

/// Parses the document's already-sampled transition declarations into the
/// second source-defined cascade bucket. The producer owns timing and value
/// sampling; this adapter only restores parser-owned values and transition
/// priority.
pub(crate) fn ParseTransitionInterpolations(
    effects: &BTreeMap<u64, Vec<cssom::CSSDeclaration>>,
    mode: CSSParserMode,
) -> Result<ProductionActiveInterpolationsMap, Vec<ProductionCascadeFallback>> {
    ParseSampledInterpolations(
        effects,
        mode,
        ProductionInterpolationKind::Transition,
        "transition-sample-parse",
    )
}

fn ParseSampledInterpolations(
    effects: &BTreeMap<u64, Vec<cssom::CSSDeclaration>>,
    mode: CSSParserMode,
    kind: ProductionInterpolationKind,
    parse_fallback: &'static str,
) -> Result<ProductionActiveInterpolationsMap, Vec<ProductionCascadeFallback>> {
    let mut parsed_effects = Vec::with_capacity(effects.len());
    let mut fallbacks = Vec::new();
    for declarations in effects.values() {
        let mut properties = Vec::new();
        for declaration in declarations {
            let mut source = declaration.property.clone();
            source.push(':');
            source.push_str(&declaration.value);
            if declaration.important {
                source.push_str(" !important");
            }
            source.push(';');
            let parsed = ParseDeclarationList(&foundation::String::from(source), mode);
            if !parsed.errors.is_empty() || parsed.properties.is_empty() {
                if !fallbacks.contains(&ProductionCascadeFallback::UnsupportedFeature(
                    parse_fallback,
                )) {
                    fallbacks.push(ProductionCascadeFallback::UnsupportedFeature(
                        parse_fallback,
                    ));
                }
            } else {
                properties.extend(parsed.properties);
            }
        }
        parsed_effects.push(properties);
    }
    if !fallbacks.is_empty() {
        return Err(fallbacks);
    }
    Ok(ProductionActiveInterpolationsMap::FromEffectValues(
        parsed_effects,
        kind,
    ))
}

/// Complete private input carried from `DocumentStyleEngine` into one
/// translated `StyleCascade::Apply` attempt. The public StyleEngine boundary
/// stays unchanged; borrowed document collaborators are available only for
/// the duration of this all-or-fallback trial.
pub(crate) struct ProductionCascadeInputs<'a> {
    /// The retained DOM owner used by element-specific cascade callbacks.
    /// `subject_node` is an arena index in this document; keeping the pair
    /// inside Style avoids projecting HTML element behavior into CSS text.
    pub document: Option<&'a dom::Document>,
    pub media: MediaValuesCachedData,
    pub base: ComputedStyleHandle,
    pub parent: Option<ComputedStyleHandle>,
    pub root_style: Option<ComputedStyleHandle>,
    pub match_result: MatchResult,
    pub root_font_size: f32,
    pub color_scheme_settings: production_style_builder::ColorSchemeSettings,
    pub system_colors: super::production_style_resolver::ProductionSystemColorContext<'a>,
    pub text_link_text_color: Option<foundation::Color>,
    pub is_document_element: bool,
    pub pseudo_id: PseudoId,
    pub originating_style: Option<ComputedStyleHandle>,
    pub subject_node: Option<usize>,
    pub container_runtime: Option<&'a ProductionContainerRuntime<'a>>,
    pub custom_properties: Option<&'a CustomProperties>,
    pub parent_custom_properties: Option<&'a CustomProperties>,
    pub substitution_context: SubstitutionContext<'a>,
    pub image_resolver: Option<&'a dyn URLImageResolver>,
    pub property_registry: Option<&'a PropertyRegistry>,
    pub function_registry: Option<&'a production_style_sheet::ProductionFunctionRegistry>,
    pub animations: Option<&'a ProductionActiveInterpolationsMap>,
    pub transitions: Option<&'a ProductionActiveInterpolationsMap>,
    pub font_metrics: Option<&'a dyn production_style_builder::FontMetricsResolver>,
}

/// The two values produced by one source `StyleCascade::Apply` operation.
/// Chromium stores custom-property data inside `ComputedStyle`; this port
/// keeps that data in the document-owned side table, so it must cross the
/// adapter boundary beside the native style without replaying the cascade.
pub(crate) struct ProductionCascadeOutput {
    pub style: ComputedStyleHandle,
    pub custom_properties: Rc<CustomProperties>,
    pub matched_property_sets: usize,
    pub registry_dependent: bool,
    pub has_animations: bool,
    pub important_set: Option<CSSBitset>,
    pub inline_style_lost: bool,
    pub rejected_legacy_overlapping: bool,
    pub length_conversion_flags: u32,
}

impl ProductionCascadeOutput {
    pub(crate) fn Get(&self) -> *mut ComputedStyle {
        self.style.Get()
    }
}

impl std::ops::Deref for ProductionCascadeOutput {
    type Target = ComputedStyleHandle;
    fn deref(&self) -> &Self::Target {
        &self.style
    }
}

#[derive(Default)]
pub(crate) struct ProductionLengthData;
impl ResolverLengthConversionData<ProductionStyleCascadeBackend<'_>> for ProductionLengthData {
    fn SetFontSizes(&mut self, _: ()) {}
    fn SetZoom(&mut self, _: f32) {}
    fn SetLineHeightSize(&mut self, _: ()) {}
    fn SetAnchorData(&mut self, _: ()) {}
    fn SubtractScrollbars(&mut self, _: &gfx::Size) {}
}
#[derive(Default)]
pub(crate) struct ProductionFontBuilder;
impl StyleResolverStateFontBuilder for ProductionFontBuilder {
    fn CreateFont(&mut self, _: &mut ComputedStyleBuilder, _: Option<&ComputedStyle>) {}
    fn DidChangeEffectiveZoom(&mut self) {}
    fn DidChangeWritingMode(&mut self) {}
    fn DidChangeTextSizeAdjust(&mut self) {}
    fn DidChangeTextOrientation(&mut self) {}
}
#[derive(Clone)]
pub(crate) struct ProductionElement {
    node: Option<usize>,
    parent: Option<ComputedStyleHandle>,
    root: Option<ComputedStyleHandle>,
    pseudo_id: PseudoId,
}

// cpp: html_parser_idioms.cc:174-224. Select's retained `size_` owner uses
// ParseHTMLNonNegativeInteger; only the exact value one changes UsesMenuList
// for a multiple select, but keep the source parser's sign/overflow behavior.
fn ParseHTMLNonNegativeInteger(value: &str) -> Option<u32> {
    let bytes = value.as_bytes();
    let mut offset = 0;
    while bytes
        .get(offset)
        .is_some_and(|byte| matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' '))
    {
        offset += 1;
    }
    let negative = match bytes.get(offset) {
        Some(b'+') => {
            offset += 1;
            false
        }
        Some(b'-') => {
            offset += 1;
            true
        }
        _ => false,
    };
    let mut parsed = 0u32;
    let mut saw_digit = false;
    while let Some(byte @ b'0'..=b'9') = bytes.get(offset) {
        saw_digit = true;
        parsed = parsed
            .checked_mul(10)?
            .checked_add(u32::from(*byte - b'0'))?;
        offset += 1;
    }
    if !saw_digit || (negative && parsed != 0) {
        return None;
    }
    Some(parsed)
}

fn HasNullNamespaceHTMLAttribute(node: &dom::persistent_document::DOMNode, name: &str) -> bool {
    node.Attributes()
        .iter()
        .any(|attribute| attribute.namespace_uri.is_empty() && attribute.local_name == name)
}

// cpp: html_select_element.cc:219-234 UpdateUsesMenuList.
fn SelectUsesMenuList(node: &dom::persistent_document::DOMNode) -> bool {
    let multiple = HasNullNamespaceHTMLAttribute(node, "multiple");
    let size = node
        .Attributes()
        .iter()
        .find(|attribute| attribute.namespace_uri.is_empty() && attribute.local_name == "size")
        .and_then(|attribute| ParseHTMLNonNegativeInteger(&attribute.value))
        .unwrap_or(0);
    if multiple {
        HasNullNamespaceHTMLAttribute(node, "size") && size == 1
    } else {
        size <= 1
    }
}

// cpp: element.cc:14142-14163 and the HTML element/type overrides of
// SupportsBaseAppearanceInternal at the pinned Chromium revision.
pub(crate) fn SupportsBaseAppearance(
    document: &dom::Document,
    element: usize,
    appearance: AppearanceValue,
) -> bool {
    if !matches!(
        appearance,
        AppearanceValue::kBase | AppearanceValue::kBaseSelect
    ) {
        return false;
    }
    let node = document.Node(element);
    if node.Namespace() != DOMNamespace::kHTML {
        return false;
    }

    if node.IsHTMLElement("select") {
        if appearance == AppearanceValue::kBase
            && !foundation::RuntimeEnabledFeatures::AppearanceBaseEnabled()
        {
            return false;
        }
        // CustomizableSelectMultiplePopup is experimental and disabled by the
        // fixed runtime-feature owner, matching blink-feature() in html.css.
        let customizable_multiple_popup = false;
        let multiple = HasNullNamespaceHTMLAttribute(node, "multiple");
        return customizable_multiple_popup || !(SelectUsesMenuList(node) && multiple);
    }

    if appearance != AppearanceValue::kBase
        || !foundation::RuntimeEnabledFeatures::AppearanceBaseEnabled()
    {
        return false;
    }
    if node.IsHTMLElement("button")
        || node.IsHTMLElement("textarea")
        || node.IsHTMLElement("progress")
    {
        return true;
    }
    if node.IsHTMLElement("datalist") {
        // CustomizableCombobox depends on AppearanceBase and is experimental
        // at the fixed revision; Rechrom's runtime owner leaves it disabled.
        return false;
    }
    document.InputType(element).is_some_and(|input_type| {
        input_type.IsTextFieldInputType()
            || matches!(
                input_type,
                DOMInputType::kButton
                    | DOMInputType::kReset
                    | DOMInputType::kSubmit
                    | DOMInputType::kCheckbox
                    | DOMInputType::kRadio
                    | DOMInputType::kColor
            )
    })
}

// cpp: css_pending_system_font_value.cc:25-32;
// layout_theme_font_provider_default.cc:34-56; each ConvertFont* pending
// branch in style_builder_converter.cc. Chromium's non-Windows provider uses
// Arial and subtracts two points at 96dpi for the three WebKit control fonts.
fn ResolveSystemFontLonghand(
    property: CSSPropertyID,
    system_font: CSSValueID,
    document: &dom::Document,
) -> Result<Rc<production_css_value::Value>, ()> {
    #[cfg(target_os = "windows")]
    {
        // The Windows provider reads live FontCache menu/status/caption data;
        // no such owner is retained by this browser target yet.
        let _ = (property, system_font, document);
        return Err(());
    }
    #[cfg(not(target_os = "windows"))]
    {
        use CSSPropertyID::*;
        let value = match property {
            kFontFamily => production_css_value::list(
                vec![Rc::new(production_css_value::Value::new(
                    CSSValuePayload::kFontFamilyClass(production_css_value::CSSFontFamilyValue(
                        foundation::String::from("Arial"),
                    )),
                ))],
                production_css_value::ListSeparator::Comma,
            ),
            kFontSize => {
                // cpp: layout_theme_font_provider.cc:52-64 and
                // font_size_functions.cc:221-240.  Medium has scale factor
                // one: zero means the provider's 16px fallback and all other
                // out-of-table values are clamped by minimum logical size 1.
                let configured_size = document.DefaultFontSize();
                let mut size = if configured_size == 0.0 {
                    16.0
                } else {
                    configured_size.max(1.0)
                };
                if matches!(
                    system_font,
                    CSSValueID::kWebkitMiniControl
                        | CSSValueID::kWebkitSmallControl
                        | CSSValueID::kWebkitControl
                ) {
                    size -= 2.0 / 72.0 * 96.0;
                }
                // Keep a negative result: FontBuilder::SetSize rejects it and
                // therefore preserves the inherited size, as Chromium does.
                production_css_value::numeric(size as f64, UnitType::kPixels)
            }
            kFontOpticalSizing | kFontKerning => {
                production_css_value::identifier(CSSValueID::kAuto)
            }
            kFontSizeAdjust => production_css_value::identifier(CSSValueID::kNone),
            kFontStyle
            | kFontVariantLigatures
            | kFontVariantCaps
            | kFontVariantNumeric
            | kFontVariantEastAsian
            | kFontVariantAlternates
            | kFontVariantPosition
            | kFontVariantEmoji
            | kFontWeight
            | kFontStretch
            | kLineHeight
            | kFontFeatureSettings
            | kFontVariationSettings
            | kFontLanguageOverride => production_css_value::identifier(CSSValueID::kNormal),
            _ => return Err(()),
        };
        Ok(value)
    }
}
#[derive(Clone)]
pub(crate) struct ProductionRequest {
    pseudo_id: PseudoId,
    originating_style: Option<ComputedStyleHandle>,
}
impl Default for ProductionRequest {
    fn default() -> Self {
        Self {
            pseudo_id: PseudoId::kPseudoIdNone,
            originating_style: None,
        }
    }
}
#[derive(Default)]
struct ProductionScope;
impl TreeScope for ProductionScope {}
#[derive(Default)]
pub(crate) struct ProductionCascadeResolver {
    filter: CascadeFilter,
    flags: Flags,
    author_flags: Flags,
    rejected_flags: Flags,
    current: Option<CSSPropertyName>,
    shorthand_cache: Option<(
        Rc<production_css_value::Value>,
        Vec<(CSSPropertyID, Rc<production_css_value::Value>)>,
    )>,
}
pub(crate) struct ProductionResolverLock(Option<CSSPropertyName>);

struct ProductionCascadeExpansionBackend<'a> {
    property_registry: Option<&'a PropertyRegistry>,
}
impl CascadeExpansionBackend for ProductionCascadeExpansionBackend<'_> {
    type Dispatch = ProductionCSSValueDispatch;
    type Document = ();
    fn Properties(&self, set: &dyn MatchedPropertySet) -> Vec<CSSPropertyValue<Self::Dispatch>> {
        let Some(set) = set
            .AsAny()
            .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
        else {
            return Vec::new();
        };
        match set {
            CSSPropertyValueSetRuleHandle::Immutable(set) => set.Properties().to_vec(),
            CSSPropertyValueSetRuleHandle::Mutable(set) => set.borrow().Properties().to_vec(),
        }
    }
    fn CustomPropertyMetadata(&self, name: &AtomicString, _: &()) -> CSSProperty {
        let inherited = self
            .property_registry
            .and_then(|registry| registry.Registration(&name.Utf8()))
            .map_or(true, |registration| registration.Inherits());
        CSSProperty::new(
            CSSPropertyID::kVariable,
            CSSProperty::kProperty
                | CSSProperty::kLonghand
                | if inherited {
                    CSSProperty::kInherited
                } else {
                    0
                }
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kNotLegacyOverlapping,
            ' ',
        )
    }
    fn IsAffectedByAll(&self, property: &CSSProperty) -> bool {
        ProductionCSSValueDispatch::IsAffectedByAll(property.PropertyID())
    }
}

pub(crate) struct ProductionStyleCascadeBackend<'a> {
    document: Option<&'a dom::Document>,
    media: MediaValuesCachedData,
    root_font_size: f32,
    color_scheme_settings: production_style_builder::ColorSchemeSettings,
    system_colors: super::production_style_resolver::ProductionSystemColorContext<'a>,
    text_link_text_color: Option<foundation::Color>,
    initial: ComputedStyleHandle,
    language: &'static AtomicString,
    locale: Option<Arc<LayoutLocale>>,
    transparent_image: OnceCell<Rc<ResolverValue<Self>>>,
    document_element: OnceCell<Rc<ProductionElement>>,
    document_element_style: OnceCell<ComputedStyleHandle>,
    custom_properties: RefCell<CustomProperties>,
    parent_custom_properties: Option<&'a CustomProperties>,
    substitution_context: SubstitutionContext<'a>,
    image_resolver: Option<&'a dyn URLImageResolver>,
    property_registry: Option<&'a PropertyRegistry>,
    function_registry: Option<&'a production_style_sheet::ProductionFunctionRegistry>,
    container_runtime: Option<&'a ProductionContainerRuntime<'a>>,
    font_metrics: Option<&'a dyn production_style_builder::FontMetricsResolver>,
    fallback: RefCell<Vec<ProductionCascadeFallback>>,
}
impl<'a> ProductionStyleCascadeBackend<'a> {
    pub(crate) fn new(
        document: Option<&'a dom::Document>,
        media: MediaValuesCachedData,
        root_font_size: f32,
        color_scheme_settings: production_style_builder::ColorSchemeSettings,
        system_colors: super::production_style_resolver::ProductionSystemColorContext<'a>,
        text_link_text_color: Option<foundation::Color>,
        custom_properties: Option<&'a CustomProperties>,
        parent_custom_properties: Option<&'a CustomProperties>,
        substitution_context: SubstitutionContext<'a>,
        image_resolver: Option<&'a dyn URLImageResolver>,
        property_registry: Option<&'a PropertyRegistry>,
        function_registry: Option<&'a production_style_sheet::ProductionFunctionRegistry>,
        container_runtime: Option<&'a ProductionContainerRuntime<'a>>,
        font_metrics: Option<&'a dyn production_style_builder::FontMetricsResolver>,
    ) -> Self {
        let language = Box::leak(Box::new(AtomicString::default()));
        // cpp: ComputedStyleBuilder's inherited/non-inherited variable data is
        // initialized with the rest of the style.  The Rust port owns those
        // maps beside ComputedStyle, so establish the equivalent base before
        // StyleCascade starts resolving declarations.  A supplied map remains
        // a compatibility seed for direct adapter callers; the production
        // StyleResolver path leaves it absent and receives the map written by
        // this cascade.
        let custom_properties = custom_properties.cloned().unwrap_or_else(|| {
            let font = media.em_size;
            let length = |number, unit| {
                production_style_builder::Pixels(
                    CSSPropertyID::kVariable,
                    number,
                    unit,
                    font,
                    root_font_size,
                    &media,
                )
                .map_err(|_| {
                    VariableResolutionError::Unsupported("registered property length conversion")
                })
            };
            CustomProperties::ComputeWithRegistry(
                parent_custom_properties,
                std::iter::empty::<&production_css_value::PropertyValue>(),
                substitution_context,
                property_registry,
                &length,
            )
        });
        Self {
            document,
            media,
            root_font_size,
            color_scheme_settings,
            system_colors,
            text_link_text_color,
            initial: Persistent::from_ptr(
                ComputedStyle::GetInitialStyleSingleton() as *mut ComputedStyle
            ),
            locale: LayoutLocale::GetShared(&language),
            language,
            transparent_image: OnceCell::new(),
            document_element: OnceCell::new(),
            document_element_style: OnceCell::new(),
            custom_properties: RefCell::new(custom_properties),
            parent_custom_properties,
            substitution_context,
            image_resolver,
            property_registry,
            function_registry,
            container_runtime,
            font_metrics,
            fallback: RefCell::new(Vec::new()),
        }
    }
    fn style(handle: &ComputedStyleHandle) -> &ComputedStyle {
        unsafe { &*handle.Get() }
    }
    fn unsupported(&self, operation: &'static str) {
        let fallback = ProductionCascadeFallback::UnsupportedFeature(operation);
        let mut fallbacks = self.fallback.borrow_mut();
        if !fallbacks.contains(&fallback) {
            fallbacks.push(fallback);
        }
    }
    fn unset(&self) -> Rc<ResolverValue<Self>> {
        production_css_value::wide(CSSValueID::kUnset)
            .expect("unset is a concrete production CSSValue")
    }
    pub(crate) fn take_fallbacks(&self) -> Vec<ProductionCascadeFallback> {
        std::mem::take(&mut *self.fallback.borrow_mut())
    }

    fn take_custom_properties(&self) -> Rc<CustomProperties> {
        Rc::new(std::mem::take(&mut *self.custom_properties.borrow_mut()))
    }

    fn DefaultCustomProperties(
        &self,
        parent: Option<&CustomProperties>,
        font_size: f32,
    ) -> CustomProperties {
        let length = |number, unit| {
            production_style_builder::Pixels(
                CSSPropertyID::kVariable,
                number,
                unit,
                font_size,
                self.root_font_size,
                &self.media,
            )
            .map_err(|_| {
                VariableResolutionError::Unsupported("registered property length conversion")
            })
        };
        CustomProperties::ComputeWithRegistry(
            parent,
            std::iter::empty::<&production_css_value::PropertyValue>(),
            self.substitution_context,
            self.property_registry,
            &length,
        )
    }

    fn CopyCustomPropertyFrom(&self, name: &str, source: Option<&CustomProperties>) {
        let mut custom = self.custom_properties.borrow_mut();
        match source.and_then(|source| source.values.get(name)).cloned() {
            Some(value) => {
                custom.SetData(name.to_owned(), value);
            }
            None => {
                custom.RemoveData(name);
            }
        }
        match source
            .and_then(|source| source.registered_values.get(name))
            .cloned()
        {
            Some(value) => {
                custom.SetRegistered(name.to_owned(), value);
            }
            None => {
                custom.RemoveRegistered(name);
            }
        }
    }

    // cpp: StyleBuilder::ApplyPhysicalProperty(CustomProperty). Chromium
    // writes custom-property data into ComputedStyleBuilder. This port keeps
    // the same data in a side table, but the write still occurs at the exact
    // translated StyleCascade callback so recursive var() lookup observes it.
    fn ApplyCustomProperty(
        &self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
    ) {
        let name = name.ToAtomicString().Utf8();
        let font_size = state.StyleBuilder().GetFontDescription().ComputedSize();
        if value.IsInitialValue() {
            let initial = self.DefaultCustomProperties(None, font_size);
            self.CopyCustomPropertyFrom(&name, Some(&initial));
            return;
        }
        if value.IsInheritedValue() || value.IsUnsetValue() && property.IsInherited() {
            self.CopyCustomPropertyFrom(&name, self.parent_custom_properties);
            return;
        }
        if value.IsUnsetValue() {
            let initial = self.DefaultCustomProperties(None, font_size);
            self.CopyCustomPropertyFrom(&name, Some(&initial));
            return;
        }
        if value.IsCyclicVariableValue() || value.IsInvalidVariableValue() {
            if self
                .property_registry
                .and_then(|registry| registry.Registration(&name))
                .is_none()
            {
                self.custom_properties.borrow_mut().SetData(
                    name,
                    if value.IsCyclicVariableValue() {
                        production_style_builder::custom_properties::ComputedVariableValue::Cyclic
                    } else {
                        production_style_builder::custom_properties::ComputedVariableValue::Invalid
                    },
                );
            }
            return;
        }
        let CSSValuePayload::kUnparsedDeclarationClass(unparsed) = value.Payload() else {
            self.unsupported("custom-property-application-value");
            return;
        };
        let data = unparsed.data.clone();
        let Some(registration) = self
            .property_registry
            .and_then(|registry| registry.Registration(&name))
        else {
            let mut custom = self.custom_properties.borrow_mut();
            custom.SetData(
                name.clone(),
                production_style_builder::custom_properties::ComputedVariableValue::Data(data),
            );
            custom.RemoveRegistered(&name);
            return;
        };
        let typed = match registration.Syntax().ParseTokens(&data) {
            Ok(value) => value,
            Err(crate::css_syntax_definition::SyntaxValueError::Invalid) => return,
            Err(crate::css_syntax_definition::SyntaxValueError::Unsupported(_)) => {
                self.unsupported("registered-property-syntax-consumer");
                return;
            }
        };
        let length = |number, unit| {
            production_style_builder::Pixels(
                CSSPropertyID::kVariable,
                number,
                unit,
                font_size,
                self.root_font_size,
                &self.media,
            )
            .map_err(|_| {
                VariableResolutionError::Unsupported("registered property length conversion")
            })
        };
        let typed = match crate::property_registry::ComputeRegisteredValue(&typed, &length) {
            Ok(value) => value,
            Err(VariableResolutionError::Invalid | VariableResolutionError::Cyclic) => return,
            Err(VariableResolutionError::TooLarge) => return,
            Err(VariableResolutionError::Unsupported(operation)) => {
                self.unsupported(operation);
                return;
            }
        };
        let data = crate::property_registry::RegisteredVariableData(&typed, &data);
        let mut custom = self.custom_properties.borrow_mut();
        custom.SetRegistered(name.clone(), typed);
        custom.SetData(
            name,
            production_style_builder::custom_properties::ComputedVariableValue::Data(data),
        );
    }

    // cpp: style_cascade.cc:897-934. Chromium's interpolation object carries
    // its PropertyHandle into CSSInterpolationEnvironment. The production
    // adapter retains the same handle beside the parser-owned sampled value,
    // resolves substitutions through this StyleCascade, then writes the
    // physical property through the canonical native style builder.
    fn ApplySampledInterpolation(
        &self,
        interpolations: &ProductionActiveInterpolations,
        mut value: Rc<production_css_value::Value>,
        origin: CascadeOrigin,
        state: &StyleResolverState<'_, Self>,
        cascade: &mut StyleCascade<'_, '_, Self>,
        resolver: &mut ProductionCascadeResolver,
    ) {
        let name = &interpolations.property;
        if name.IsCustomProperty() {
            // cpp: css_default_interpolation_type.cc:58-76 and
            // custom_property.cc:107-207.  A sampled custom property is
            // animation-tainted before it is resolved and installed in the
            // current ComputedStyleBuilder.  This port keeps that builder data
            // in the per-cascade CustomProperties side table, so clone only
            // the sampled token owner and publish it through the same
            // ApplyCustomProperty callback used by ordinary declarations.
            //
            // This callback is intentionally re-entrant.  While resolving a
            // longhand containing var(--x), ResolveVarInto calls
            // LookupAndApply(--x); if animation/transition priority wins,
            // StyleCascade reaches this callback before it asks
            // BuilderVariableData for --x.  The dependent longhand therefore
            // observes the sampled value during this same Apply operation.
            if let CSSValuePayload::kUnparsedDeclarationClass(unparsed) = value.Payload() {
                let mut data = (*unparsed.data).clone();
                data.is_animation_tainted = true;
                value = production_css_value::unparsed_with_base(
                    data,
                    unparsed.mode,
                    unparsed.base_url.clone(),
                );
            }
        }
        let Some(value) = cascade.Resolve(name, value, None, None, origin, resolver) else {
            self.unsupported("cyclic-interpolation-value");
            return;
        };
        if name.IsCustomProperty() {
            let property = self.CustomPropertyMetadata(name.ToAtomicString(), &());
            self.ApplyCustomProperty(name, &property, state, &value);
            return;
        }
        let logical = *CSSProperty::Get(name.Id());
        let property = if logical.IsSurrogate() {
            self.SurrogateFor(&logical, state.StyleBuilder().GetWritingDirection())
        } else {
            logical
        };
        let physical_name = property.GetCSSPropertyName();
        self.ApplyPhysicalProperty(&physical_name, &property, state, &value);
    }

    fn numeric_variable_data(text: &String) -> Option<Rc<CSSVariableData>> {
        let property = ParseCustomProperty(
            "--rechrom-numeric-coercion",
            text,
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .ok()?;
        let CSSValuePayload::kUnparsedDeclarationClass(value) = property.Value().Payload() else {
            return None;
        };
        Some(value.data.clone())
    }

    fn math_length_resolver<'s>(
        &'s self,
        state: &StyleResolverState<'_, Self>,
    ) -> impl CSSMathLengthResolver + 's {
        let font = state.StyleBuilder().GetFontDescription().ComputedSize();
        // cpp: style_cascade.cc:2701-2702 uses Unzoomed(). The source length
        // is therefore reconstructed at zoom 1 before it is converted back to
        // a CSS primitive value.
        production_style_builder::MathLengthResolver(
            CSSPropertyID::kInvalid,
            font,
            self.root_font_size,
            1.0,
            &self.media,
            Some(production_style_builder::CurrentComputedLineHeight(
                &state.StyleBuilder(),
            )),
            self.document_element_style.get().map(|style| {
                let style = ProductionStyleCascadeBackend::style(style);
                style.ComputedLineHeight() / style.EffectiveZoom()
            }),
        )
    }

    fn primitive_math_value<'v>(
        value: &'v ResolverValue<Self>,
    ) -> Option<&'v crate::css_math_function_value::CSSMathFunctionValue> {
        match value.Payload() {
            CSSValuePayload::kMathFunctionClass(value) => Some(value),
            _ => None,
        }
    }

    fn primitive_literal<'v>(
        value: &'v ResolverValue<Self>,
    ) -> Option<&'v crate::css_numeric_literal_value::CSSNumericLiteralValue> {
        match value.Payload() {
            CSSValuePayload::kNumericLiteralClass(value) => Some(value),
            _ => None,
        }
    }

    fn compute_primitive(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
        operation: &'static str,
    ) -> f64 {
        let result = if let Some(value) = Self::primitive_literal(value) {
            Some(value.DoubleValue() * ConversionToCanonicalUnitsScaleFactor(value.GetType()))
        } else if let Some(value) = Self::primitive_math_value(value) {
            let mut resolver = self.math_length_resolver(state);
            value.ComputeValue(&mut resolver, None).ok()
        } else {
            None
        };
        result
            .map(CSSValueClampingUtils::ClampDouble)
            .unwrap_or_else(|| {
                self.unsupported(operation);
                0.0
            })
    }

    fn compute_ident_component(value: &ResolverValue<Self>) -> Option<std::string::String> {
        match value.Payload() {
            CSSValuePayload::kCustomIdentClass(value) => {
                if value.property == CSSPropertyID::kInvalid {
                    Some(value.name.Utf8())
                } else {
                    Some(crate::css_property_names::GetPropertyName(value.property).to_owned())
                }
            }
            CSSValuePayload::kStringClass(value) => Some(value.0.Utf8()),
            CSSValuePayload::kNumericLiteralClass(value) => {
                Some(format!("{}", value.DoubleValue()))
            }
            CSSValuePayload::kFunctionClass(value) if value.function_id == CSSValueID::kIdent => {
                let mut result = std::string::String::new();
                for item in &value.arguments.values {
                    result.push_str(&Self::compute_ident_component(item)?);
                }
                Some(result)
            }
            _ => None,
        }
    }
}

// cpp: css_parsing_utils.cc:10483-10516. This intentionally accepts only a
// calc() block (or a top-level sequence) made from dimension literals and
// binary '+'/'-'. It is the safe-area fast-path predicate, not a CSS math
// evaluator.
fn IsSimpleLengthSumStream<T: crate::parser::css_parser_token_stream::TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> bool {
    use crate::parser::css_parser_token::CSSParserTokenType::*;
    while !stream.AtEnd() {
        match stream.Peek().GetType() {
            kDelimiterToken if matches!(stream.Peek().Delimiter(), 43 | 45) => {
                stream.ConsumeIncludingWhitespace();
            }
            kDimensionToken => {
                stream.ConsumeIncludingWhitespace();
            }
            _ => break,
        }
    }
    stream.AtEnd()
}

fn IsSimpleLengthSum(text: &String) -> bool {
    use crate::parser::{css_parser_token::BlockType, css_parser_token_stream::BlockGuard};
    let mut stream = TokenStream::new(StringView::from(text), 0);
    if stream.Peek().GetBlockType() == BlockType::kBlockStart {
        if stream.Peek().FunctionId() != Some(CSSValueID::kCalc) {
            return false;
        }
        let mut guard = BlockGuard::new(&mut stream);
        guard.ConsumeWhitespace();
        IsSimpleLengthSumStream(&mut guard)
    } else {
        IsSimpleLengthSumStream(&mut stream)
    }
}

// cpp: css_parsing_utils.cc:10443-10481. Walk each nested block while keeping
// the source spelling check for the environment variable name case-sensitive.
fn ContainsSafeAreaInsetBottom<T: crate::parser::css_parser_token_stream::TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> bool {
    use crate::parser::{
        css_parser_token::{BlockType, CSSParserTokenType::kIdentToken},
        css_parser_token_stream::{BlockGuard, RestoringBlockGuard},
    };
    while !stream.AtEnd() {
        if stream.Peek().GetBlockType() == BlockType::kBlockStart {
            if stream.Peek().FunctionId() == Some(CSSValueID::kEnv) {
                let found = {
                    let mut guard = RestoringBlockGuard::new(stream);
                    guard.ConsumeWhitespace();
                    guard.Peek().GetType() == kIdentToken
                        && guard.Peek().Value().ToString() == String::from("safe-area-inset-bottom")
                };
                if found {
                    return true;
                }
            }
            let mut guard = BlockGuard::new(stream);
            if ContainsSafeAreaInsetBottom(&mut guard) {
                return true;
            }
            continue;
        }
        stream.Consume();
    }
    false
}

thread_local! {
    // cpp: style_resolver.cc:593-602. Source keeps this declaration block in a
    // process cache; the Rust production path keeps the same immutable typed
    // property-set identity per style thread.
    static DOCUMENT_ELEMENT_USER_AGENT_DECLARATIONS:
        Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>> = {
            let property = CSSPropertyValue::new(
                &CSSPropertyName::new(CSSPropertyID::kColor),
                production_css_value::initial_color(),
                false,
                false,
                0,
                false,
            );
            CSSPropertyValueSetRuleHandle::FromImmutable(
                ImmutableCSSPropertyValueSet::Create(
                    &[property],
                    CSSParserMode::kHTMLStandardMode,
                    false,
                ),
            )
        };
}

fn PrependDocumentElementUserAgentDeclarations(result: &mut MatchResult) {
    let declarations: Rc<dyn MatchedPropertySet> =
        DOCUMENT_ELEMENT_USER_AGENT_DECLARATIONS.with(Clone::clone);
    let mut data = MatchedPropertiesData::default();
    data.origin = CascadeOrigin::kUserAgent;
    result.PrependMatchedProperties(declarations, None, data);
}

/// Applies an already-collected Blink `MatchResult` through the translated
/// `StyleCascade::Apply` implementation.  The caller keeps ownership of rule
/// matching and the public `StyleEngine` boundary; this adapter owns only the
/// resolver state needed by the cascade.
pub(crate) fn ApplyMatchResult(
    inputs: ProductionCascadeInputs<'_>,
) -> Result<ProductionCascadeOutput, Vec<ProductionCascadeFallback>> {
    ApplyMatchResultWithFilter(inputs, CascadeFilter::new())
}

/// StyleResolver's MISU branch uses the same native property application as
/// StyleCascade, without matching/cascading rules again. The caller has already
/// checked the source eligibility guards and owns the previous style builder.
pub(crate) fn ApplyInlineProperties(
    inputs: ProductionCascadeInputs<'_>,
    builder: &mut ComputedStyleBuilder,
    properties: &[(CSSPropertyName, Rc<production_css_value::Value>)],
) -> Result<u32, Vec<ProductionCascadeFallback>> {
    let backend = ProductionStyleCascadeBackend::new(
        inputs.document,
        inputs.media.clone(),
        inputs.root_font_size,
        inputs.color_scheme_settings,
        inputs.system_colors,
        inputs.text_link_text_color,
        inputs.custom_properties,
        inputs.parent_custom_properties,
        inputs.substitution_context,
        inputs.image_resolver,
        inputs.property_registry,
        inputs.function_registry,
        inputs.container_runtime,
        inputs.font_metrics,
    );
    if let Some(root) = inputs.root_style.clone() {
        let _ = backend.document_element_style.set(root);
    }
    let element = Rc::new(ProductionElement {
        node: inputs.subject_node,
        parent: inputs.parent.clone(),
        root: inputs.root_style.clone(),
        pseudo_id: inputs.pseudo_id,
    });
    let request = ProductionRequest {
        pseudo_id: inputs.pseudo_id,
        originating_style: None,
    };
    let state = StyleResolverState::new(&backend, &(), element, None, Some(&request));
    state.CreateNewStyle(
        ProductionStyleCascadeBackend::style(&inputs.base),
        ProductionStyleCascadeBackend::style(inputs.parent.as_ref().unwrap_or(&inputs.base)),
        None,
    );
    std::mem::swap(&mut *state.StyleBuilderMut(), builder);
    for (name, value) in properties {
        let logical = CSSProperty::Get(name.Id());
        let property = if logical.IsSurrogate() {
            backend.SurrogateFor(logical, state.StyleBuilder().GetWritingDirection())
        } else {
            *logical
        };
        backend.ApplyPhysicalProperty(
            &CSSPropertyName::new(property.PropertyID()),
            &property,
            &state,
            value,
        );
    }
    let flags = state.TakeLengthConversionFlags();
    std::mem::swap(&mut *state.StyleBuilderMut(), builder);
    let fallbacks = backend.take_fallbacks();
    if fallbacks.is_empty() {
        Ok(flags)
    } else {
        Err(fallbacks)
    }
}

pub(crate) fn ApplyMatchResultWithFilter(
    mut inputs: ProductionCascadeInputs<'_>,
    filter: CascadeFilter,
) -> Result<ProductionCascadeOutput, Vec<ProductionCascadeFallback>> {
    let matched_property_sets = inputs.match_result.GetMatchedProperties().len();
    let registry_dependent = inputs
        .property_registry
        .is_some_and(|registry| !registry.IsEmpty())
        && inputs
            .match_result
            .GetMatchedProperties()
            .iter()
            .any(|matched| {
                matched
                    .properties
                    .AsAny()
                    .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
                    .is_some_and(|set| {
                        let properties = match set {
                            CSSPropertyValueSetRuleHandle::Immutable(set) => set.Properties(),
                            CSSPropertyValueSetRuleHandle::Mutable(set) => {
                                return set.borrow().Properties().iter().any(|property| {
                                    property.PropertyID() == CSSPropertyID::kVariable
                                        || property.Value().IsUnparsedDeclaration()
                                });
                            }
                        };
                        properties.iter().any(|property| {
                            property.PropertyID() == CSSPropertyID::kVariable
                                || property.Value().IsUnparsedDeclaration()
                        })
                    })
            });
    let has_animations = inputs.animations.is_some_and(|map| !map.IsEmpty())
        || inputs.transitions.is_some_and(|map| !map.IsEmpty());
    let backend = ProductionStyleCascadeBackend::new(
        inputs.document,
        inputs.media.clone(),
        inputs.root_font_size,
        inputs.color_scheme_settings,
        inputs.system_colors,
        inputs.text_link_text_color,
        inputs.custom_properties,
        inputs.parent_custom_properties,
        inputs.substitution_context,
        inputs.image_resolver,
        inputs.property_registry,
        inputs.function_registry,
        inputs.container_runtime,
        inputs.font_metrics,
    );
    let document = ();
    if let Some(root_style) = inputs.root_style.clone() {
        let _ = backend.document_element_style.set(root_style);
    }
    // cpp: style_resolver.cc:1796-1802. The root element has no DOM parent,
    // but cascade inheritance and relative-value resolution use the translated
    // InitialStyleForElement result as its parent.
    let parent = if inputs.is_document_element && inputs.parent.is_none() {
        Some(super::production_style_resolver::InitialStyleForDocument(
            &inputs.media,
            inputs.color_scheme_settings,
        ))
    } else {
        inputs.parent.clone()
    };
    let element = Rc::new(ProductionElement {
        node: inputs.subject_node,
        parent: parent.clone(),
        root: inputs.root_style.clone(),
        pseudo_id: inputs.pseudo_id,
    });
    if inputs.is_document_element {
        let _ = backend.document_element.set(element.clone());
        PrependDocumentElementUserAgentDeclarations(&mut inputs.match_result);
    }
    // cpp: StyleResolverState::StyleResolverState.  A pseudo cascade is
    // identified by the StyleRequest, while `element` remains the originating
    // element.  This keeps pseudo inheritance and pseudo-sensitive cascade
    // branches on the translated state path instead of inferring them from the
    // final builder after the cascade has started.
    let request = ProductionRequest {
        pseudo_id: inputs.pseudo_id,
        originating_style: inputs.originating_style.clone().or_else(|| {
            (inputs.pseudo_id != PseudoId::kPseudoIdNone)
                .then(|| parent.clone())
                .flatten()
        }),
    };
    let state = StyleResolverState::new(&backend, &document, element, None, Some(&request));
    let inherited_from = parent.as_ref().unwrap_or(&inputs.base);
    state.CreateNewStyle(
        ProductionStyleCascadeBackend::style(&inputs.base),
        ProductionStyleCascadeBackend::style(inherited_from),
        None,
    );
    // cpp: StyleResolver::ResolveStyle/ResolvePseudoElementStyle assigns the
    // request's pseudo type after CreateNewStyle. The builder's
    // initial/non-inherited copy intentionally does not carry this field.
    if inputs.pseudo_id != PseudoId::kPseudoIdNone {
        state.StyleBuilderMut().SetStyleType(inputs.pseudo_id);
    }
    let (important_set, inline_style_lost) = {
        let mut cascade = StyleCascade::new(&state, &backend);
        *cascade.MutableMatchResult() = inputs.match_result;
        if let Some(animations) = inputs.animations.filter(|map| !map.IsEmpty()) {
            cascade.AddInterpolations(animations, CascadeOrigin::kAnimation);
        }
        if let Some(transitions) = inputs.transitions.filter(|map| !map.IsEmpty()) {
            cascade.AddInterpolations(transitions, CascadeOrigin::kTransition);
        }
        cascade.Apply(filter);
        let inline_style_lost = cascade.InlineStyleLost();
        (cascade.ReleaseImportantSet(), inline_style_lost)
    };
    let fallbacks = backend.take_fallbacks();
    if !fallbacks.is_empty() {
        return Err(fallbacks);
    }
    let rejected_legacy_overlapping = state.RejectedLegacyOverlapping();
    let length_conversion_flags = state.TakeLengthConversionFlags();
    let style = state.TakeStyle().ok_or_else(|| {
        vec![ProductionCascadeFallback::UnsupportedFeature(
            "cascade-produced-null-style",
        )]
    })?;
    Ok(ProductionCascadeOutput {
        style,
        custom_properties: backend.take_custom_properties(),
        matched_property_sets,
        registry_dependent,
        has_animations,
        important_set,
        inline_style_lost,
        rejected_legacy_overlapping,
        length_conversion_flags,
    })
}

impl StyleResolverBackend for ProductionStyleCascadeBackend<'_> {
    type Document = ();
    type ComputedStyle = ComputedStyleHandle;
    type ComputedStyleBuilder = ComputedStyleBuilder;
    type MatchedPropertiesCache = ();
    type SelectorFilter = ();
    type StyleRuleUsageTracker = ();
    type StyleEngine = ();
    type LocalFrame = ();
    type LocalFrameView = ();
    type Settings = ();
    type FontDescription = FontDescription;
    type LayoutLocale = Option<Arc<LayoutLocale>>;
    type StyleInitialData = ();
    type TapHighlightColor = StyleColor;
    type RuleSet = ();
    type RuleSetGroup = ();
    fn NewRuleSetGroup(&self, rule_set_group_index: u32) -> Self::RuleSetGroup {
        ()
    }
    fn NewMatchedPropertiesCache(&self) -> Self::MatchedPropertiesCache {
        ()
    }
    fn NewSelectorFilter(&self) -> Self::SelectorFilter {
        ()
    }
    fn ClearMatchedPropertiesCache(&self, cache: &mut Self::MatchedPropertiesCache) {
        ()
    }
    fn ClearViewportDependent(&self, cache: &mut Self::MatchedPropertiesCache) {
        ()
    }
    fn GetStyleEngine<'d>(&self, document: &'d Self::Document) -> &'d Self::StyleEngine {
        document
    }
    fn GetFrame<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::LocalFrame> {
        Some(document)
    }
    fn View<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::LocalFrameView> {
        Some(document)
    }
    fn GetSettings<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::Settings> {
        None
    }
    fn Printing(&self, document: &Self::Document) -> bool {
        false
    }
    fn VisuallyOrdered(&self, document: &Self::Document) -> bool {
        false
    }
    fn InForcedColorsMode(&self, document: &Self::Document) -> bool {
        self.media.forced_colors == crate::media_queries::forced_colors::ForcedColors::kActive
    }
    fn InDesignMode(&self, document: &Self::Document) -> bool {
        false
    }
    fn ContentLanguage<'d>(&self, document: &'d Self::Document) -> &'d AtomicString {
        self.language
    }
    fn LayoutZoomFactor(&self, frame: &Self::LocalFrame) -> f32 {
        1.0
    }
    fn MediaType<'v>(&self, view: &'v Self::LocalFrameView) -> &'v str {
        "screen"
    }
    fn UpdateActiveStyle(&self, engine: &Self::StyleEngine) {
        ()
    }
    fn GetPageColorSchemes(&self, engine: &Self::StyleEngine) -> ColorSchemeFlags {
        self.color_scheme_settings.page_color_schemes
    }
    fn GetPreferredColorScheme(&self, engine: &Self::StyleEngine) -> PreferredColorScheme {
        self.media.preferred_color_scheme
    }
    fn GetForceDarkModeEnabled(&self, engine: &Self::StyleEngine) -> bool {
        self.color_scheme_settings.force_dark
    }
    fn MaybeCreateAndGetInitialData<'e>(
        &self,
        engine: &'e Self::StyleEngine,
    ) -> Option<&'e Self::StyleInitialData> {
        None
    }
    fn PreferDefaultScrollbarStylesEnabled(&self) -> bool {
        false
    }
    fn GetPrefersDefaultScrollbarStyles(&self, settings: &Self::Settings) -> bool {
        false
    }
    fn GetInitialStyleSingleton(&self) -> &Self::ComputedStyle {
        &self.initial
    }
    fn NewComputedStyleBuilder(&self, initial: &Self::ComputedStyle) -> Self::ComputedStyleBuilder {
        ComputedStyleBuilder::from_style(Self::style(initial))
    }
    fn NewComputedStyleBuilderInheritingFrom(
        &self,
        initial: &Self::ComputedStyle,
        parent: &Self::ComputedStyle,
    ) -> Self::ComputedStyleBuilder {
        ComputedStyleBuilder::from_initial_and_parent(
            Self::style(initial),
            Self::style(parent),
            IsAtShadowBoundary::kNotAtShadowBoundary,
        )
    }
    fn TakeStyle(&self, builder: Self::ComputedStyleBuilder) -> Self::ComputedStyle {
        {
            let mut builder = builder;
            Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle)
        }
    }
    fn ClampLineWidth(&self, value: f32) -> i32 {
        crate::resolver::style_builder_converter::StyleBuilderConverter::ClampLineWidth(
            value as f64,
        )
    }
    fn SetBorderTopWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32) {
        builder.SetBorderTopWidthOwned(width)
    }
    fn SetBorderRightWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32) {
        builder.SetBorderRightWidthOwned(width)
    }
    fn SetBorderBottomWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32) {
        builder.SetBorderBottomWidthOwned(width)
    }
    fn SetBorderLeftWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32) {
        builder.SetBorderLeftWidthOwned(width)
    }
    fn SetOutlineWidth(&self, builder: &mut Self::ComputedStyleBuilder, width: i32) {
        builder.SetOutlineWidthOwned(width)
    }
    fn SetColumnRuleWidthInternal(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        width: GapDataList<i32>,
    ) {
        builder.SetColumnRuleWidthInternal(&width)
    }
    fn SetRowRuleWidthInternal(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        width: GapDataList<i32>,
    ) {
        builder.SetRowRuleWidth(&width)
    }
    fn SetRtlOrdering(&self, builder: &mut Self::ComputedStyleBuilder, ordering: EOrder) {
        builder.SetRtlOrdering(ordering)
    }
    fn SetZoom(&self, builder: &mut Self::ComputedStyleBuilder, zoom: f32) {
        builder.SetZoom(zoom)
    }
    fn SetEffectiveZoom(&self, builder: &mut Self::ComputedStyleBuilder, zoom: f32) {
        {
            builder.SetEffectiveZoom(zoom);
        }
    }
    fn SetInForcedColorsMode(&self, builder: &mut Self::ComputedStyleBuilder, enabled: bool) {
        builder.SetInForcedColorsMode(enabled)
    }
    fn InitialTapHighlightColor(&self) -> Self::TapHighlightColor {
        ComputedStyleInitialValues::InitialTapHighlightColor()
    }
    fn SetTapHighlightColor(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        color: Self::TapHighlightColor,
    ) {
        builder.SetTapHighlightColorOwned(color)
    }
    fn SetUsedColorScheme(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        page_schemes: ColorSchemeFlags,
        preferred: PreferredColorScheme,
        force_dark: bool,
    ) {
        {
            let preferred = match preferred {
                PreferredColorScheme::kDark => {
                    layoutng_style::style::color_scheme::mojom::blink::PreferredColorScheme::kDark
                }
                PreferredColorScheme::kLight => {
                    layoutng_style::style::color_scheme::mojom::blink::PreferredColorScheme::kLight
                }
            };
            builder.SetUsedColorScheme(page_schemes, preferred, force_dark);
        }
    }
    fn GetFontDescription(&self, builder: &Self::ComputedStyleBuilder) -> Self::FontDescription {
        builder.GetFontDescription().clone()
    }
    fn GetLayoutLocale(&self, language: &AtomicString) -> &Self::LayoutLocale {
        &self.locale
    }
    fn SetLocale(&self, description: &mut Self::FontDescription, locale: &Self::LayoutLocale) {
        description.SetLocale(locale.clone())
    }
    fn SetFontDescription(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        description: Self::FontDescription,
    ) {
        production_style_builder::StageFontDescription(builder, &description)
    }
    fn SetUserModify(&self, builder: &mut Self::ComputedStyleBuilder, modify: EUserModify) {
        builder.SetUserModify(modify)
    }
    fn CreateInitialFont(
        &self,
        document: &Self::Document,
        builder: &mut Self::ComputedStyleBuilder,
    ) {
        {
            let mut d = builder.GetFontDescription().clone();
            d.SetKeywordSize(4);
            d.SetGenericFamily(
                font_engine::fonts::font_description::GenericFamilyType::kStandardFamily,
            );
            d.SetSpecifiedSize(self.media.em_size);
            d.SetComputedSize(self.media.em_size);
            d.SetAdjustedSize(self.media.em_size);
            production_style_builder::StageFontDescription(builder, &d);
        }
    }
    fn SetInitialData(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        initial_data: &Self::StyleInitialData,
    ) {
        ()
    }
    fn SetPrefersDefaultScrollbarStyles(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: bool,
    ) {
        builder.SetPrefersDefaultScrollbarStyles(value)
    }
}

impl StyleResolverStateBackend for ProductionStyleCascadeBackend<'_> {
    type Element = ProductionElement;
    type PseudoElement = ProductionElement;
    type ElementResolveContext = Rc<ProductionElement>;
    type StyleRecalcContext = ();
    type StyleRequest = ProductionRequest;
    type AnchorEvaluator = ();
    type AnimationUpdate = ();
    type FontBuilder = ProductionFontBuilder;
    type ElementStyleResources = ();
    type SVGResource = ();
    type ValueDispatch = ProductionCSSValueDispatch;
    type LengthConversionData = ProductionLengthData;
    type FontSizes = ();
    type LineHeightSize = ();
    type ViewportSize = ();
    type ContainerSizes = ();
    type AnchorData = ();
    fn DefaultStyleRequest(&self) -> Self::StyleRequest {
        ProductionRequest::default()
    }
    fn RequestParentOverride(&self, request: &Self::StyleRequest) -> Option<ComputedStyleHandle> {
        None
    }
    fn RequestLayoutParentOverride(
        &self,
        request: &Self::StyleRequest,
    ) -> Option<ComputedStyleHandle> {
        None
    }
    fn RequestOriginatingElementStyle(
        &self,
        request: &Self::StyleRequest,
    ) -> Option<ComputedStyleHandle> {
        request.originating_style.clone()
    }
    fn RequestStyledElement(&self, request: &Self::StyleRequest) -> Option<Rc<Self::Element>> {
        None
    }
    fn RequestPseudoId(&self, request: &Self::StyleRequest) -> PseudoId {
        request.pseudo_id
    }
    fn RequestPseudoArgument(&self, request: &Self::StyleRequest) -> AtomicString {
        AtomicString::default()
    }
    fn RequestType(&self, request: &Self::StyleRequest) -> StyleResolverRequestType {
        StyleResolverRequestType::kForComputedStyle
    }
    fn RequestCanTriggerAnimations(&self, request: &Self::StyleRequest) -> bool {
        false
    }
    fn RecalcOldStyle(&self, context: &Self::StyleRecalcContext) -> Option<ComputedStyleHandle> {
        None
    }
    fn RecalcSizeContainer(&self, context: &Self::StyleRecalcContext) -> Option<Rc<Self::Element>> {
        None
    }
    fn RecalcAnchorEvaluator(
        &self,
        context: &Self::StyleRecalcContext,
    ) -> Option<Rc<Self::AnchorEvaluator>> {
        None
    }
    fn NewElementResolveContext(&self, element: Rc<Self::Element>) -> Self::ElementResolveContext {
        element
    }
    fn ContextElement(&self, context: &Self::ElementResolveContext) -> Rc<Self::Element> {
        context.clone()
    }
    fn ContextUltimateOriginatingElementOrSelf(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Rc<Self::Element> {
        context.clone()
    }
    fn ContextParentElement(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<Rc<Self::Element>> {
        None
    }
    fn ContextRootElementStyle(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<ComputedStyleHandle> {
        context.root.clone()
    }
    fn ContextParentStyle(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<ComputedStyleHandle> {
        context.parent.clone()
    }
    fn ContextLayoutParentStyle(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<ComputedStyleHandle> {
        context.parent.clone()
    }
    fn ContextElementLinkState(&self, context: &Self::ElementResolveContext) -> EInsideLink {
        EInsideLink::kNotInsideLink
    }
    fn ContextPseudoElement(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<Rc<Self::PseudoElement>> {
        None
    }
    fn GetStyledPseudoElement(
        &self,
        element: &Rc<Self::Element>,
        id: PseudoId,
        argument: &AtomicString,
    ) -> Option<Rc<Self::Element>> {
        Some(Rc::new(ProductionElement {
            node: element.node,
            parent: element.parent.clone(),
            root: element.root.clone(),
            pseudo_id: id,
        }))
    }
    fn ElementAsPseudoElement(
        &self,
        element: &Rc<Self::Element>,
    ) -> Option<Rc<Self::PseudoElement>> {
        (element.pseudo_id != PseudoId::kPseudoIdNone).then(|| element.clone())
    }
    fn PseudoElementAsElement(&self, element: Rc<Self::PseudoElement>) -> Rc<Self::Element> {
        element
    }
    fn ElementPseudoIdForStyling(&self, element: &Self::Element) -> PseudoId {
        element.pseudo_id
    }
    fn ElementIsLink(&self, element: &Self::Element) -> bool {
        false
    }
    fn ElementIsInCanvasSubtree(&self, element: &Self::Element) -> bool {
        false
    }
    fn FlatTreeParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>> {
        None
    }
    fn LayoutObjectIsNeeded(&self, element: &Self::Element, display: DisplayStyle<'_>) -> bool {
        true
    }
    fn IsAnimatingDisplayProperty(&self, element: &Self::Element) -> bool {
        false
    }
    fn DocumentDevicePixelRatio(&self, document: &Self::Document) -> f32 {
        self.media.device_pixel_ratio
    }
    fn DocumentInQuirksMode(&self, document: &Self::Document) -> bool {
        false
    }
    fn DocumentElementComputedStyle(
        &self,
        document: &Self::Document,
    ) -> Option<ComputedStyleHandle> {
        self.document_element_style.get().cloned()
    }
    fn DocumentViewportSize(&self, document: &Self::Document) -> Self::ViewportSize {
        ()
    }
    fn ViewportSizeFromDocumentLayoutView(&self, document: &Self::Document) -> Self::ViewportSize {
        ()
    }
    fn NewAnimationUpdate(&self) -> Self::AnimationUpdate {
        ()
    }
    fn ClearAnimationUpdate(&self, update: &mut Self::AnimationUpdate) {
        ()
    }
    fn NewFontBuilder(&self, document: &Self::Document) -> Self::FontBuilder {
        ProductionFontBuilder
    }
    fn FontDescriptionFromStyle(&self, style: &ComputedStyle) -> Self::FontDescription {
        style.GetFontDescription().clone()
    }
    fn NewElementStyleResources(
        &self,
        element: &Self::Element,
        device_pixel_ratio: f32,
    ) -> Self::ElementStyleResources {
        ()
    }
    fn ResourcesUpdateLengthConversionData(
        &self,
        resources: &mut Self::ElementStyleResources,
        data: &Rc<RefCell<Self::LengthConversionData>>,
    ) {
        ()
    }
    fn ResourcesLoadPendingResources(
        &self,
        resources: &mut Self::ElementStyleResources,
        builder: &mut ComputedStyleBuilder,
        data: &Self::LengthConversionData,
    ) {
        ()
    }
    fn ResourcesGetStyleImage(
        &self,
        resources: &mut Self::ElementStyleResources,
        property: CSSPropertyID,
        value: Rc<ResolverValue<Self>>,
    ) -> Option<Rc<StyleImage>> {
        None
    }
    fn ResourcesGetSVGResource(
        &self,
        resources: &mut Self::ElementStyleResources,
        property: CSSPropertyID,
        value: &ResolverURIValue<Self>,
    ) -> Option<Rc<Self::SVGResource>> {
        None
    }
    fn URIIsLocal(&self, value: &ResolverURIValue<Self>, document: &Self::Document) -> bool {
        false
    }
    fn NewElementLengthConversionData(
        &self,
        element: &Self::Element,
    ) -> Self::LengthConversionData {
        ProductionLengthData
    }
    fn NewFontSizes(
        &self,
        font: FontSizeStyle<'_>,
        root_style: Option<&ComputedStyle>,
    ) -> Self::FontSizes {
        ()
    }
    fn NewLineHeightSize(
        &self,
        font: FontSizeStyle<'_>,
        root_style: Option<&ComputedStyle>,
    ) -> Self::LineHeightSize {
        ()
    }
    fn NewContainerSizes(&self, element: Option<Rc<Self::Element>>) -> Self::ContainerSizes {
        ()
    }
    fn NewAnchorData(
        &self,
        evaluator: Option<Rc<Self::AnchorEvaluator>>,
        default: DefaultAnchorData,
        offsets: Option<PositionAreaOffsets>,
    ) -> Self::AnchorData {
        ()
    }
    fn NewBuilderLengthConversionData(
        &self,
        builder: &ComputedStyleBuilder,
        parent: Option<&ComputedStyle>,
        root: Option<&ComputedStyle>,
        viewport: Self::ViewportSize,
        containers: Self::ContainerSizes,
        anchors: Self::AnchorData,
        zoom: f32,
        flags: Rc<Cell<u32>>,
        element: Rc<Self::Element>,
    ) -> Self::LengthConversionData {
        ProductionLengthData
    }
    fn NewUnzoomedLengthConversionData(
        &self,
        writing_mode: WritingMode,
        fonts: Self::FontSizes,
        line_height: Self::LineHeightSize,
        viewport: Self::ViewportSize,
        containers: Self::ContainerSizes,
        anchors: Self::AnchorData,
        zoom: f32,
        flags: Rc<Cell<u32>>,
        element: Rc<Self::Element>,
    ) -> Self::LengthConversionData {
        ProductionLengthData
    }
    fn ComputePositionAreaOffsetsForLayout(
        &self,
        evaluator: &Self::AnchorEvaluator,
        default: DefaultAnchorData,
    ) -> Option<PositionAreaOffsets> {
        None
    }
    fn AnchorContainerWritingDirection(
        &self,
        evaluator: &Self::AnchorEvaluator,
    ) -> WritingDirectionMode {
        WritingDirectionMode::new(WritingMode::kHorizontalTb, foundation::TextDirection::kLtr)
    }
    fn LightDarkFirst(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSLightDarkValuePair,
    ) -> Rc<ResolverValue<Self>> {
        value.AsValuePair().first.clone()
    }
    fn LightDarkSecond(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSLightDarkValuePair,
    ) -> Rc<ResolverValue<Self>> {
        value.AsValuePair().second.clone()
    }
    fn IdentifierValueID(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSIdentifierValue,
    ) -> CSSValueID {
        value.0
    }
    fn CreateIdentifierValue(&self, id: CSSValueID) -> Rc<ResolverValue<Self>> {
        production_css_value::identifier(id)
    }
    fn NewColorImageValue(&self, color: Rc<ResolverValue<Self>>) -> Rc<ResolverValue<Self>> {
        {
            self.unsupported("color-image");
            self.unset()
        }
    }
    fn TransparentImageCache(&self) -> &OnceCell<Rc<ResolverValue<Self>>> {
        &self.transparent_image
    }
    fn ResolveGradientValuesIfNeeded(
        &self,
        value: Rc<ResolverValue<Self>>,
        state: &StyleResolverState<'_, Self>,
        mode: ValueResolutionMode,
    ) -> Rc<ResolverValue<Self>> {
        value
    }
    fn ResolveImageSetValuesIfNeeded(
        &self,
        value: Rc<ResolverValue<Self>>,
        state: &StyleResolverState<'_, Self>,
        mode: ValueResolutionMode,
    ) -> Rc<ResolverValue<Self>> {
        value
    }
    fn ResolveCrossfadeValuesIfNeeded(
        &self,
        value: Rc<ResolverValue<Self>>,
        state: &StyleResolverState<'_, Self>,
        mode: ValueResolutionMode,
    ) -> Rc<ResolverValue<Self>> {
        value
    }
}

impl StyleCascadeBackend for ProductionStyleCascadeBackend<'_> {
    type VariableData = CSSVariableData;
    type Syntax = CSSSyntaxDefinition;
    type ParserPlatform = production_style_sheet::Backend;
    type StyleRuleFunction = production_style_sheet::ProductionFunctionHandle;
    type Rule = crate::style_rule::StyleRuleBase<production_style_sheet::Backend>;
    type RuleGroup = Vec<Rc<Self::Rule>>;
    type ContainerQuerySet = ContainerSet;
    type ResolverCycleNode = CSSPropertyName;
    type ResolverLock = ProductionResolverLock;
    type AnchorScope = ();
    type AttrType = AttributeType;
    type ShadowRoot = ();
    type ContainerQuery = crate::production_rule_context::ProductionContainerQuery;
    type ContainerSelector = ProductionContainerSelector;
    type ContainerChange = ProductionContainerChange;
    type MediaQueryFeatureExpNode = MediaQueryExp<
        production_css_value::Value,
        production_css_value::CSSUnparsedDeclarationValue,
    >;
    type ConditionalExpNode = IfCondition;
    type NavigationExpNode = ();
    type NavigationTest = ();
    type MediaQuerySet = QuerySet;
    type CascadeResolver = ProductionCascadeResolver;
    type ActiveInterpolationsMap = ProductionActiveInterpolationsMap;
    type ActiveInterpolations = ProductionActiveInterpolations;
    type PropertyHandle = CSSPropertyName;
    type ScrollbarGutter = ();
    fn VariableText(&self, data: &Self::VariableData) -> String {
        data.Serialize()
    }
    fn VariableFeatures(&self, data: &Self::VariableData) -> u32 {
        data.features as u32
    }
    fn VariableNeedsResolution(&self, data: &Self::VariableData) -> bool {
        data.NeedsVariableResolution()
    }
    fn VariableAnimationTainted(&self, data: &Self::VariableData) -> bool {
        data.is_animation_tainted
    }
    fn VariableAttrTainted(&self, data: &Self::VariableData) -> bool {
        data.is_attr_tainted
    }
    fn ExtractVariableFeatures(&self, token: &CSSParserToken) -> u32 {
        CSSVariableData::ExtractFeatures(token) as u32
    }
    fn NewVariableData(
        &self,
        text: String,
        animation_tainted: bool,
        attr_tainted: bool,
        features: u32,
    ) -> Variable<Self> {
        if let Ok(property) = ParseCustomProperty(
            "--rechrom-cascade-value",
            &text,
            false,
            CSSParserMode::kHTMLStandardMode,
        ) {
            if let CSSValuePayload::kUnparsedDeclarationClass(value) = property.Value().Payload() {
                let mut data = (*value.data).clone();
                data.features = features as u8;
                data.is_animation_tainted = animation_tainted;
                data.is_attr_tainted = attr_tainted;
                return Rc::new(data);
            }
        }
        self.unsupported("variable-data-tokenization");
        Rc::new(CSSVariableData {
            tokens: Vec::new(),
            original_text: text,
            features: features as u8,
            is_animation_tainted: animation_tainted,
            is_attr_tainted: attr_tainted,
        })
    }
    fn StripTrailingWhitespaceAndComments(&self, text: StringView) -> StringView {
        text
    }
    fn UnparsedData(&self, value: &ResolverValue<Self>) -> Variable<Self> {
        match value.Payload() {
            CSSValuePayload::kUnparsedDeclarationClass(v) => v.data.clone(),
            _ => {
                self.unsupported("unparsed-data-type");
                Rc::new(CSSVariableData {
                    tokens: Vec::new(),
                    original_text: String::default(),
                    features: 0,
                    is_animation_tainted: false,
                    is_attr_tainted: false,
                })
            }
        }
    }
    fn UnparsedParserContext(
        &self,
        value: &ResolverValue<Self>,
    ) -> Option<Rc<ParserContext<Self>>> {
        match value.Payload() {
            CSSValuePayload::kUnparsedDeclarationClass(v) => Some(Rc::new(CSSParserContext::<
                production_style_sheet::Backend,
            >::new(
                v.base_url.clone(),
                true,
                None,
                v.mode,
                String::from(""),
                false,
                crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
                None,
                None,
                crate::css_resource_fetch_restriction::ResourceFetchRestriction::kNone,
            ))),
            _ => None,
        }
    }
    fn StrictParserContext(&self, document: &Self::Document) -> Rc<ParserContext<Self>> {
        StrictCSSParserContext::<production_style_sheet::Backend>(
            crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
        )
    }
    fn PendingShorthand(
        &self,
        value: &ResolverValue<Self>,
    ) -> (CSSPropertyID, Rc<ResolverValue<Self>>) {
        match value.Payload() {
            CSSValuePayload::kPendingSubstitutionValueClass(v) => (v.shorthand, v.value.clone()),
            _ => {
                self.unsupported("pending-shorthand-type");
                (CSSPropertyID::kAll, self.unset())
            }
        }
    }
    fn NewUnparsedValue(
        &self,
        data: Variable<Self>,
        context: Option<Rc<ParserContext<Self>>>,
    ) -> Rc<ResolverValue<Self>> {
        let mode = context
            .as_ref()
            .map(|context| context.Mode())
            .unwrap_or(CSSParserMode::kHTMLStandardMode);
        Rc::new(ResolverValue::<Self>::new(
            CSSValuePayload::kUnparsedDeclarationClass(
                production_css_value::CSSUnparsedDeclarationValue {
                    data,
                    mode,
                    base_url: context
                        .as_ref()
                        .map(|context| context.BaseURL().clone())
                        .unwrap_or(production_css_value::ProductionURL::Null),
                },
            ),
        ))
    }
    fn NewCyclicVariableValue(&self) -> Rc<ResolverValue<Self>> {
        self.unset()
    }
    fn NewInvalidVariableValue(&self) -> Rc<ResolverValue<Self>> {
        self.unset()
    }
    fn ConsumeCSSWideKeyword(
        &self,
        stream: &mut TokenStream<'_>,
        context: &ParserContext<Self>,
    ) -> Option<Rc<ResolverValue<Self>>> {
        let text = stream.RemainingText().ToString().Utf8();
        let keyword = match text.trim().to_ascii_lowercase().as_str() {
            "inherit" => CSSValueID::kInherit,
            "initial" => CSSValueID::kInitial,
            "unset" => CSSValueID::kUnset,
            "revert" => CSSValueID::kRevert,
            "revert-layer" => CSSValueID::kRevertLayer,
            "revert-rule" => CSSValueID::kRevertRule,
            _ => return None,
        };
        production_css_value::wide(keyword)
    }
    fn ParseSingleValue(
        &self,
        id: CSSPropertyID,
        stream: &mut TokenStream<'_>,
        context: &ParserContext<Self>,
    ) -> Option<Rc<ResolverValue<Self>>> {
        crate::parser::production_property_parser::ParsePropertyWithContext(
            id,
            &stream.RemainingText().ToString(),
            false,
            context,
        )
        .ok()?
        .into_iter()
        .find(|property| property.PropertyID() == id)
        .map(|property| property.ValueRef())
    }
    fn ParseShorthand(
        &self,
        id: CSSPropertyID,
        stream: &mut TokenStream<'_>,
        context: Option<&ParserContext<Self>>,
    ) -> Option<Vec<(CSSPropertyID, Rc<ResolverValue<Self>>)>> {
        let strict;
        let context = if let Some(context) = context {
            context
        } else {
            strict = StrictCSSParserContext::<production_style_sheet::Backend>(
                crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
            );
            strict.as_ref()
        };
        crate::parser::production_property_parser::ParsePropertyWithContext(
            id,
            &stream.RemainingText().ToString(),
            false,
            context,
        )
        .ok()
        .map(|properties| {
            properties
                .into_iter()
                .map(|property| (property.PropertyID(), property.ValueRef()))
                .collect()
        })
    }
    fn ShorthandCacheValue(
        &self,
        resolver: &Self::CascadeResolver,
    ) -> Option<Rc<ResolverValue<Self>>> {
        resolver
            .shorthand_cache
            .as_ref()
            .map(|(value, _)| value.clone())
    }
    fn ShorthandCacheProperties(
        &self,
        resolver: &Self::CascadeResolver,
    ) -> Vec<(CSSPropertyID, Rc<ResolverValue<Self>>)> {
        resolver
            .shorthand_cache
            .as_ref()
            .map_or_else(Vec::new, |(_, properties)| properties.clone())
    }
    fn SetShorthandCache(
        &self,
        resolver: &mut Self::CascadeResolver,
        value: Rc<ResolverValue<Self>>,
        properties: Vec<(CSSPropertyID, Rc<ResolverValue<Self>>)>,
    ) {
        resolver.shorthand_cache = Some((value, properties));
    }
    fn PropertyCycleNode(&self, name: &CSSPropertyName) -> Self::ResolverCycleNode {
        name.clone()
    }
    fn FunctionCycleNode(&self, function: &Self::StyleRuleFunction) -> Self::ResolverCycleNode {
        CSSPropertyName::new(CSSPropertyID::kAll)
    }
    fn LocalCycleNode(
        &self,
        name: &AtomicString,
        function: Option<&Self::StyleRuleFunction>,
    ) -> Self::ResolverCycleNode {
        CSSPropertyName::custom(name.clone())
    }
    fn ResolverDetectCycle(
        &self,
        resolver: &mut Self::CascadeResolver,
        node: &Self::ResolverCycleNode,
    ) -> bool {
        resolver.current.as_ref() == Some(node)
    }
    fn ResolverInCycle(&self, resolver: &Self::CascadeResolver) -> bool {
        false
    }
    fn ResolverLock(
        &self,
        resolver: &mut Self::CascadeResolver,
        node: Self::ResolverCycleNode,
    ) -> Self::ResolverLock {
        ProductionResolverLock(resolver.current.replace(node))
    }
    fn ResolverUnlock(&self, resolver: &mut Self::CascadeResolver, lock: Self::ResolverLock) {
        resolver.current = lock.0
    }
    fn ResolverAllowSubstitution(
        &self,
        resolver: &Self::CascadeResolver,
        data: Option<&Self::VariableData>,
    ) -> bool {
        // cpp: cascade_resolver.cc:20-28. Animation-tainted variables may be
        // composed through other custom properties, but must not feed an
        // animation-affecting native property. `current` is the production
        // adapter's equivalent of CascadeResolver::CurrentProperty(): the
        // outer longhand lock is restored after a re-entrant custom-property
        // interpolation has populated the retained variable owner.
        if !data.is_some_and(|data| data.is_animation_tainted) {
            return true;
        }
        let Some(current) = resolver.current.as_ref() else {
            return true;
        };
        current.IsCustomProperty() || !CSSProperty::Get(current.Id()).IsAnimationProperty()
    }
    fn ResolverNextFunctionInvocationCount(&self, resolver: &mut Self::CascadeResolver) -> usize {
        0
    }
    fn ResolverFunctionInvocationCount(&self, resolver: &Self::CascadeResolver) -> usize {
        0
    }
    fn ResolverRandomValueCount(&self, resolver: &Self::CascadeResolver) -> usize {
        0
    }
    fn ResolverSetRandomValueCount(&self, resolver: &mut Self::CascadeResolver, count: usize) {
        ()
    }
    fn PropertyRegistrationSyntax(
        &self,
        document: &Self::Document,
        name: &AtomicString,
    ) -> Option<Rc<Self::Syntax>> {
        self.property_registry
            .and_then(|registry| registry.Registration(&name.Utf8()))
            .map(|registration| Rc::new(registration.Syntax().clone()))
    }
    fn SyntaxIsUniversal(&self, syntax: &Self::Syntax) -> bool {
        syntax.IsUniversal()
    }
    fn SyntaxParse(
        &self,
        syntax: &Self::Syntax,
        text: String,
        context: &ParserContext<Self>,
        local: &mut CSSParserLocalContext,
        animation_tainted: bool,
    ) -> Option<Rc<ResolverValue<Self>>> {
        let source = String::from(format!("--registered:{}", text.Utf8()).as_str());
        let parsed = ParseDeclarationList(&source, context.Mode());
        let data = parsed.properties.into_iter().find_map(|property| {
            let CSSValuePayload::kUnparsedDeclarationClass(value) = property.Value().Payload()
            else {
                return None;
            };
            Some(value.data.clone())
        })?;
        syntax.ParseTokens(&data).ok()
    }
    fn ConvertRegisteredPropertyValue(
        &self,
        state: &StyleResolverState<'_, Self>,
        value: Rc<ResolverValue<Self>>,
        context: &ParserContext<Self>,
    ) -> Rc<ResolverValue<Self>> {
        value
    }
    fn ConvertRegisteredPropertyVariableData(
        &self,
        value: &ResolverValue<Self>,
        animation_tainted: bool,
        attr_tainted: bool,
    ) -> Variable<Self> {
        let original = match value.Payload() {
            CSSValuePayload::kUnparsedDeclarationClass(v) => v.data.clone(),
            _ => {
                let source =
                    String::from(format!("--registered:{}", value.CssText().Utf8()).as_str());
                ParseDeclarationList(&source, CSSParserMode::kHTMLStandardMode)
                    .properties
                    .into_iter()
                    .find_map(|property| {
                        let CSSValuePayload::kUnparsedDeclarationClass(value) =
                            property.Value().Payload()
                        else {
                            return None;
                        };
                        Some(value.data.clone())
                    })
                    .unwrap_or_else(|| {
                        Rc::new(CSSVariableData {
                            tokens: Vec::new(),
                            original_text: value.CssText(),
                            features: 0,
                            is_animation_tainted: animation_tainted,
                            is_attr_tainted: attr_tainted,
                        })
                    })
            }
        };
        crate::property_registry::RegisteredVariableData(value, &original)
    }
    fn BuilderVariableData(
        &self,
        state: &StyleResolverState<'_, Self>,
        name: &AtomicString,
        inherited: bool,
    ) -> Option<Variable<Self>> {
        self.custom_properties.borrow().Get(&name.Utf8()).cloned()
    }
    fn BuilderInitialData(
        &self,
        state: &StyleResolverState<'_, Self>,
    ) -> Option<Rc<Self::StyleInitialData>> {
        self.property_registry.map(|_| Rc::new(()))
    }
    fn InitialDataVariableData(
        &self,
        initial: &Self::StyleInitialData,
        name: &AtomicString,
    ) -> Option<Variable<Self>> {
        self.property_registry
            .and_then(|registry| registry.Registration(&name.Utf8()))
            .and_then(|registration| {
                registration
                    .Initial()
                    .zip(registration.initial_data.as_ref())
            })
            .map(|(value, data)| crate::property_registry::RegisteredVariableData(value, data))
    }
    fn ParentSetChildHasExplicitInheritance(&self, state: &StyleResolverState<'_, Self>) {
        ()
    }
    fn ParentVariableData(
        &self,
        state: &StyleResolverState<'_, Self>,
        name: &AtomicString,
    ) -> Option<Variable<Self>> {
        self.parent_custom_properties
            .and_then(|custom| custom.Get(&name.Utf8()).cloned())
    }
    fn MarkPropertyRegistryReferenced(&self, document: &Self::Document, name: &AtomicString) {
        if let Some(registry) = self.property_registry {
            registry.MarkReferenced(&name.Utf8());
        }
    }
    fn IsValidVariableName(&self, name: StringView) -> bool {
        name.ToString().Utf8().starts_with("--")
    }
    fn ConsumeIdentFunction(
        &self,
        stream: &mut TokenStream<'_>,
        context: &ParserContext<Self>,
    ) -> Rc<ResolverValue<Self>> {
        use crate::parser::{
            css_parser_token::{CSSParserTokenType::*, NumericValueType},
            css_parser_token_stream::RestoringBlockGuard,
        };
        if stream.Peek().FunctionId() != Some(CSSValueID::kIdent) {
            return self.unset();
        }
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        let mut values = Vec::new();
        while !guard.AtEnd() {
            let token = guard.Peek().clone();
            let value = if token.GetType() == kIdentToken
                && production_css_value::wide(token.Id()).is_none()
                && token.Id() != CSSValueID::kDefault
            {
                guard.ConsumeIncludingWhitespace();
                Some(production_css_value::custom_ident(
                    &token.Value().ToString(),
                    CSSPropertyID::kInvalid,
                ))
            } else if token.GetType() == kStringToken {
                guard.ConsumeIncludingWhitespace();
                Some(production_css_value::string(token.Value().ToString()))
            } else if token.GetType() == kNumberToken
                && token.GetNumericValueType() == NumericValueType::kIntegerValueType
            {
                guard.ConsumeIncludingWhitespace();
                Some(production_css_value::numeric(
                    token.NumericValue(),
                    UnitType::kInteger,
                ))
            } else if token.FunctionId() == Some(CSSValueID::kIdent) {
                let nested = self.ConsumeIdentFunction(&mut guard, context);
                matches!(
                    nested.Payload(),
                    CSSValuePayload::kFunctionClass(value)
                        if value.function_id == CSSValueID::kIdent
                )
                .then_some(nested)
            } else {
                None
            };
            let Some(value) = value else {
                return self.unset();
            };
            values.push(value);
        }
        if values.is_empty() || !guard.Release() {
            return self.unset();
        }
        drop(guard);
        stream.ConsumeWhitespace();
        Rc::new(ResolverValue::<Self>::new(CSSValuePayload::kFunctionClass(
            production_css_value::CSSFunctionValue {
                function_id: CSSValueID::kIdent,
                arguments: production_css_value::CSSValueList::new(
                    values,
                    production_css_value::ListSeparator::Space,
                ),
            },
        )))
    }
    fn ComputeIdent(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> AtomicString {
        Self::compute_ident_component(value)
            .map(|value| AtomicString::from_str(&value))
            .unwrap_or_default()
    }
    fn CSSFunctionsEnabled(&self) -> bool {
        self.function_registry.is_some()
    }
    fn FlipRevertPropertyID(&self, value: &ResolverValue<Self>) -> CSSPropertyID {
        CSSPropertyID::kAll
    }
    fn FlipRevertedValue(
        &self,
        from: CSSPropertyID,
        value: Rc<ResolverValue<Self>>,
        flip: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> Rc<ResolverValue<Self>> {
        value
    }
    fn EnterAnchorScope(
        &self,
        property: CSSPropertyID,
        state: &StyleResolverState<'_, Self>,
    ) -> Self::AnchorScope {
        ()
    }
    fn MathHasInvalidAnchorFunctions(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> bool {
        false
    }
    fn LeaveAnchorScope(&self, scope: Self::AnchorScope) {
        ()
    }
    fn MixinParent(
        &self,
        bindings: &dyn MixinParameterBindings,
    ) -> Option<Rc<dyn MixinParameterBindings>> {
        let Some(bindings) = bindings
            .AsAny()
            .downcast_ref::<production_style_sheet::ProductionMixinParameterBindings>(
        ) else {
            self.unsupported("mixin-owner");
            return None;
        };
        bindings
            .Parent()
            .map(|parent| parent as Rc<dyn MixinParameterBindings>)
    }
    fn MixinParameters(
        &self,
        bindings: &dyn MixinParameterBindings,
    ) -> Vec<(
        AtomicString,
        Option<Variable<Self>>,
        Option<Variable<Self>>,
        Rc<Self::Syntax>,
    )> {
        let Some(bindings) = bindings
            .AsAny()
            .downcast_ref::<production_style_sheet::ProductionMixinParameterBindings>(
        ) else {
            self.unsupported("mixin-owner");
            return Vec::new();
        };
        bindings
            .Bindings()
            .iter()
            .map(|(name, binding)| {
                (
                    AtomicString::from_utf16(name.Span16().unwrap_or_default()),
                    binding.argument.clone(),
                    binding.default_value.clone(),
                    Rc::new(binding.syntax.clone()),
                )
            })
            .collect()
    }
    fn MixinBaseLocals(&self, bindings: &dyn MixinParameterBindings) -> VariableMap<Self> {
        let Some(bindings) = bindings
            .AsAny()
            .downcast_ref::<production_style_sheet::ProductionMixinParameterBindings>(
        ) else {
            self.unsupported("mixin-owner");
            return HashMap::new();
        };
        bindings
            .BaseLocals()
            .iter()
            .map(|(name, value)| {
                (
                    AtomicString::from_utf16(name.Span16().unwrap_or_default()),
                    Some(value.clone()),
                )
            })
            .collect()
    }
    fn MixinConditionalLocals(
        &self,
        bindings: &dyn MixinParameterBindings,
    ) -> Vec<(
        AtomicString,
        Vec<(Rc<Self::ContainerQuerySet>, Variable<Self>)>,
    )> {
        let Some(bindings) = bindings
            .AsAny()
            .downcast_ref::<production_style_sheet::ProductionMixinParameterBindings>(
        ) else {
            self.unsupported("mixin-owner");
            return Vec::new();
        };
        bindings
            .ConditionalLocals()
            .iter()
            .map(|(name, candidates)| {
                (
                    AtomicString::from_utf16(name.Span16().unwrap_or_default()),
                    candidates
                        .iter()
                        .map(|candidate| {
                            (candidate.container_queries.clone(), candidate.data.clone())
                        })
                        .collect(),
                )
            })
            .collect()
    }
    fn FindFunctionAcrossScopes(
        &self,
        document: &Self::Document,
        name: &AtomicString,
        scope: Option<&dyn TreeScope>,
    ) -> Option<(Rc<Self::StyleRuleFunction>, Rc<dyn TreeScope>)> {
        let Some(registry) = self.function_registry else {
            self.unsupported("css-function-owner");
            return None;
        };
        registry.FunctionForName(name).map(|function| {
            (
                Rc::new(function),
                Rc::new(ProductionScope) as Rc<dyn TreeScope>,
            )
        })
    }
    fn FunctionName(&self, function: &Self::StyleRuleFunction) -> AtomicString {
        function.Name().clone()
    }
    fn FunctionParameters(
        &self,
        function: &Self::StyleRuleFunction,
    ) -> Vec<(AtomicString, Rc<Self::Syntax>, Option<Variable<Self>>)> {
        function
            .GetParameters()
            .iter()
            .map(|parameter| {
                (
                    AtomicString::from_utf16(parameter.name.Span16().unwrap_or_default()),
                    Rc::new(parameter.r#type.clone()),
                    parameter.default_value.clone(),
                )
            })
            .collect()
    }
    fn FunctionReturnType(&self, function: &Self::StyleRuleFunction) -> Rc<Self::Syntax> {
        Rc::new(function.GetReturnType().clone())
    }
    fn ConsumeFunctionArguments(
        &self,
        stream: &mut TokenStream<'_>,
        parameter_count: usize,
    ) -> Vec<String> {
        use crate::parser::{
            css_parser_token::CSSParserTokenType::{kCommaToken, kLeftBraceToken},
            css_parser_token_stream::BlockGuard,
        };
        let mut arguments = Vec::new();
        while arguments.len() < parameter_count {
            stream.ConsumeWhitespace();
            if stream.AtEnd() || (!arguments.is_empty() && stream.Peek().GetType() != kCommaToken) {
                break;
            }
            if stream.Peek().GetType() == kCommaToken {
                stream.ConsumeIncludingWhitespace();
            }
            let argument = if stream.Peek().GetType() == kLeftBraceToken {
                let mut guard = BlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let start = guard.LookAheadOffset();
                guard.SkipUntilPeekedTypeIs(&[]);
                let end = guard.LookAheadOffset();
                guard.StringRangeAt(start, end - start).ToString()
            } else {
                let start = stream.LookAheadOffset();
                stream.SkipUntilPeekedTypeIs(&[kCommaToken]);
                let end = stream.LookAheadOffset();
                stream.StringRangeAt(start, end - start).ToString()
            };
            if argument.empty() {
                break;
            }
            arguments.push(argument);
        }
        arguments
    }
    fn FunctionAsGroup(&self, function: &Self::StyleRuleFunction) -> Rc<Self::RuleGroup> {
        Rc::new(function.ChildRules().clone())
    }
    fn RuleChildren(&self, group: &Self::RuleGroup) -> Vec<Rc<Self::Rule>> {
        group.clone()
    }
    fn FunctionRuleKind(&self, rule: &Self::Rule) -> FunctionRuleKind {
        match rule {
            crate::style_rule::StyleRuleBase::FunctionDeclarations(_) => {
                FunctionRuleKind::Declarations
            }
            crate::style_rule::StyleRuleBase::Supports(_) => FunctionRuleKind::Supports,
            crate::style_rule::StyleRuleBase::Media(_) => FunctionRuleKind::Media,
            crate::style_rule::StyleRuleBase::Container(_) => FunctionRuleKind::Container,
            crate::style_rule::StyleRuleBase::Navigation(_) => FunctionRuleKind::Navigation,
            _ => FunctionRuleKind::Other,
        }
    }
    fn FunctionRuleGroup(&self, rule: &Self::Rule) -> Rc<Self::RuleGroup> {
        let children = match rule {
            crate::style_rule::StyleRuleBase::Supports(rule) => rule.ChildRules(),
            crate::style_rule::StyleRuleBase::Media(rule) => rule.ChildRules(),
            crate::style_rule::StyleRuleBase::Container(rule) => rule.ChildRules(),
            crate::style_rule::StyleRuleBase::Navigation(rule) => rule.ChildRules(),
            _ => {
                self.unsupported("css-function-rule-group");
                return Rc::new(Vec::new());
            }
        };
        Rc::new(children.clone())
    }
    fn FunctionRuleDeclarations(
        &self,
        rule: &Self::Rule,
    ) -> Vec<(CSSPropertyName, Variable<Self>)> {
        let crate::style_rule::StyleRuleBase::FunctionDeclarations(declarations) = rule else {
            self.unsupported("css-function-declarations-kind");
            return Vec::new();
        };
        let properties = declarations.Properties();
        let collect = |values: &[CSSPropertyValue<ProductionCSSValueDispatch>]| {
            values
                .iter()
                .filter_map(|property| {
                    let CSSValuePayload::kUnparsedDeclarationClass(value) =
                        property.Value().Payload()
                    else {
                        self.unsupported("css-function-declaration-value");
                        return None;
                    };
                    Some((property.Name(), value.data.clone()))
                })
                .collect()
        };
        match properties.as_ref() {
            CSSPropertyValueSetRuleHandle::Immutable(set) => collect(set.Properties()),
            CSSPropertyValueSetRuleHandle::Mutable(set) => collect(set.borrow().Properties()),
        }
    }
    fn SupportsConditionIsSupported(&self, rule: &Self::Rule) -> bool {
        match rule {
            crate::style_rule::StyleRuleBase::Supports(rule) => rule.ConditionIsSupported(),
            _ => {
                self.unsupported("functional-supports-kind");
                false
            }
        }
    }
    fn EvaluateFunctionalMediaRule(&self, rule: &Self::Rule, document: &Self::Document) -> bool {
        match rule {
            crate::style_rule::StyleRuleBase::Media(rule) => {
                rule.MediaQueries().is_none_or(|set| {
                    let values = MediaValuesCached::new(&self.media);
                    MediaQueryEvaluator::ForMediaValues(&values).Eval(set)
                })
            }
            _ => {
                self.unsupported("functional-media-kind");
                false
            }
        }
    }
    fn ContainerRuleQueries(&self, rule: &Self::Rule) -> Rc<Self::ContainerQuerySet> {
        match rule {
            crate::style_rule::StyleRuleBase::Container(rule) => {
                Rc::new(rule.GetContainerQuerySet().clone())
            }
            _ => {
                self.unsupported("functional-container-kind");
                Rc::new(ContainerSet::new(Vec::new()))
            }
        }
    }
    fn EvaluateNavigationRule(&self, rule: &Self::Rule, document: &Self::Document) -> bool {
        self.unsupported("functional-navigation");
        false
    }
    fn AttributeCycleNode(&self, name: &AtomicString) -> Self::ResolverCycleNode {
        CSSPropertyName::custom(name.clone())
    }
    fn CSSArgumentGrammarEnabled(&self) -> bool {
        self.substitution_context
            .attributes
            .is_some_and(|attributes| attributes.CSSArgumentGrammarEnabled())
    }
    fn ConsumeAttrType(&self, stream: &mut TokenStream<'_>) -> Option<Self::AttrType> {
        ConsumeAttributeType(stream)
    }
    fn DefaultAttrType(&self) -> Self::AttrType {
        AttributeType::RawString
    }
    fn AttrTypeIsSyntax(&self, attr_type: &Self::AttrType) -> bool {
        // The stable production path admits the legacy literal/unit grammar.
        // CSS syntax definitions remain explicitly gated at document assembly.
        false
    }
    fn ParseAttrType(
        &self,
        attr_type: &Self::AttrType,
        value: String,
        context: &ParserContext<Self>,
        local: &mut CSSParserLocalContext,
    ) -> Option<Rc<ResolverValue<Self>>> {
        match ParseAttributeValue(&value, *attr_type) {
            Ok(Some(data)) => Some(self.NewUnparsedValue(data, None)),
            Ok(None) => None,
            Err(VariableResolutionError::Unsupported(operation)) => {
                self.unsupported(operation);
                None
            }
            Err(_) => None,
        }
    }
    fn ElementLowercaseIfNecessary(
        &self,
        element: &Self::Element,
        name: AtomicString,
    ) -> AtomicString {
        // PersistentAttributeSource already owns the HTML lowercasing policy.
        name
    }
    fn ElementGetAttributeNS(
        &self,
        element: &Self::Element,
        namespace: &AtomicString,
        name: &AtomicString,
    ) -> String {
        if !namespace.IsNull() {
            return String::default();
        }
        let Some(attributes) = self.substitution_context.attributes else {
            self.unsupported("attr-owner");
            return String::default();
        };
        attributes.SetHasAttrFunction();
        attributes.GetAttribute(name)
    }
    fn ParseVariableDeclarationValue(
        &self,
        value: String,
        animation_tainted: bool,
        context: &ParserContext<Self>,
    ) -> Option<Variable<Self>> {
        let property = ParseCustomProperty(
            "--rechrom-variable-declaration",
            &value,
            false,
            context.Mode(),
        )
        .ok()?;
        let CSSValuePayload::kUnparsedDeclarationClass(value) = property.Value().Payload() else {
            return None;
        };
        let mut data = (*value.data).clone();
        data.is_animation_tainted |= animation_tainted;
        Some(Rc::new(data))
    }
    fn ElementSupportsBaseAppearance(
        &self,
        element: &Self::Element,
        appearance: AppearanceValue,
    ) -> bool {
        // Element::SupportsBaseAppearance returns false immediately for every
        // non-base appearance. This is the common first-argument path for the
        // UA sheet and must not turn a valid -internal-auto-base() value into
        // an adapter fallback.
        if !matches!(
            appearance,
            AppearanceValue::kBase | AppearanceValue::kBaseSelect
        ) {
            return false;
        }
        let (Some(document), Some(node)) = (self.document, element.node) else {
            self.unsupported("base-appearance-owner");
            return false;
        };
        SupportsBaseAppearance(document, node, appearance)
    }
    fn SetNeedsToUpdateComplexSafeAreaConstraints(&self, document: &Self::Document) {
        // The EnvironmentVariableResolver is the typed owner of this state;
        // resolving safe-area-inset-bottom through it records the dependency.
    }
    fn ElementTreeScopeRootShadowRoot(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::ShadowRoot>> {
        // ProductionElement represents a document-tree element. Shadow-tree
        // elements stay on the typed fallback path before this adapter.
        None
    }
    fn ShadowRootIsUserAgent(&self, root: &Self::ShadowRoot) -> bool {
        self.unsupported("shadow-root");
        false
    }
    fn ResolveEnvironmentVariable(
        &self,
        document: &Self::Document,
        name: &AtomicString,
        indices: Vec<u32>,
        record_metrics: bool,
    ) -> Option<Variable<Self>> {
        let Some(environment) = self.substitution_context.environment else {
            self.unsupported("environment-variable-owner");
            return None;
        };
        environment.ResolveEnvironmentVariable(name, &indices)
    }
    fn ContainerQueries(&self, queries: &Self::ContainerQuerySet) -> Vec<Rc<Self::ContainerQuery>> {
        queries.Queries().to_vec()
    }
    fn QuerySelector(&self, query: &Self::ContainerQuery) -> Rc<Self::ContainerSelector> {
        Rc::new(ProductionContainerSelector::FromQuery(query))
    }
    fn SelectorSelectsAnyContainer(&self, selector: &Self::ContainerSelector) -> bool {
        if selector.HasUnknownFeature() {
            self.unsupported("container-query-unknown-feature");
        }
        selector.SelectsAnyContainer()
    }
    fn QuerySetDependencyFlags(&self, query: &Self::ContainerQuery, result: &mut MatchResult) {
        ProductionContainerSelector::FromQuery(query).SetDependencyFlags(result)
    }
    fn QueryDetermineStartingElement(
        &self,
        element: &Self::Element,
        pseudo: layoutng_style::style::computed_style_constants::PseudoId,
        selector: &Self::ContainerSelector,
        nearest_size_container: Option<&Self::Element>,
    ) -> Option<Rc<Self::Element>> {
        let Some(runtime) = self.container_runtime else {
            self.unsupported("container-query-owner");
            return None;
        };
        let Some(subject) = element.node else {
            self.unsupported("container-query-subject");
            return None;
        };
        runtime
            .DetermineStartingElement(
                subject,
                pseudo,
                selector,
                nearest_size_container.and_then(|element| element.node),
            )
            .map(|node| {
                Rc::new(ProductionElement {
                    node: Some(node),
                    parent: None,
                    root: element.root.clone(),
                    pseudo_id: PseudoId::kPseudoIdNone,
                })
            })
    }
    fn QueryFindContainer(
        &self,
        starting: Option<&Self::Element>,
        selector: &Self::ContainerSelector,
        scope: Option<&dyn TreeScope>,
    ) -> Option<Rc<Self::Element>> {
        let Some(runtime) = self.container_runtime else {
            self.unsupported("container-query-owner");
            return None;
        };
        let node = runtime.FindContainer(starting.and_then(|element| element.node), selector)?;
        Some(Rc::new(ProductionElement {
            node: Some(node),
            parent: None,
            root: None,
            pseudo_id: PseudoId::kPseudoIdNone,
        }))
    }
    fn NearestContainerChange(&self) -> Self::ContainerChange {
        ProductionContainerChange::NearestContainer
    }
    fn DescendantContainersChange(&self) -> Self::ContainerChange {
        ProductionContainerChange::DescendantContainers
    }
    fn QueryEvalAndAdd(
        &self,
        container: &Self::Element,
        query: &Self::ContainerQuery,
        change: Self::ContainerChange,
        result: &mut MatchResult,
    ) -> bool {
        let Some(runtime) = self.container_runtime else {
            self.unsupported("container-query-owner");
            return false;
        };
        let Some(node) = container.node else {
            self.unsupported("container-query-container");
            return false;
        };
        match runtime.Evaluate(node, query) {
            Ok(evaluation) => {
                ApplyContainerUnitDependencies(evaluation.unit_flags, result);
                if evaluation.value == KleeneValue::kUnknown {
                    self.unsupported("container-query-state-unavailable");
                    false
                } else {
                    evaluation.value == KleeneValue::kTrue
                }
            }
            Err(operation) => {
                self.unsupported(operation);
                false
            }
        }
    }
    fn NumericSyntaxParse(
        &self,
        text: String,
        context: &ParserContext<Self>,
        local: &mut CSSParserLocalContext,
        animation_tainted: bool,
        attr_tainted: bool,
    ) -> Option<Rc<ResolverValue<Self>>> {
        let data = Self::numeric_variable_data(&text)?;
        match CSSSyntaxDefinition::CreateNumericSyntax().ParseTokens(&data) {
            Ok(value) => Some(value),
            Err(crate::css_syntax_definition::SyntaxValueError::Invalid) => None,
            Err(crate::css_syntax_definition::SyntaxValueError::Unsupported(_)) => {
                let mut stream =
                    CSSParserTokenStream::FromTokenizer(VariableTokenReplay::new(&data));
                stream.ConsumeWhitespace();
                let expression =
                    match crate::css_math_expression_node::ConsumeMathFunction(&mut stream) {
                        Ok(expression) => expression,
                        Err(MathError::UnsupportedFunction) => {
                            self.unsupported("numeric-syntax-unsupported-function");
                            return None;
                        }
                        Err(MathError::UnsupportedTypedArithmetic) => {
                            self.unsupported("numeric-syntax-unsupported-typed-arithmetic");
                            return None;
                        }
                        Err(_) => return None,
                    };
                if !stream.AtEnd()
                    || !matches!(
                        expression.Category(),
                        CalculationResultCategory::Number
                            | CalculationResultCategory::Length
                            | CalculationResultCategory::Percent
                            | CalculationResultCategory::Angle
                            | CalculationResultCategory::Time
                            | CalculationResultCategory::Resolution
                    )
                {
                    return None;
                }
                Some(production_css_value::math(expression, ValueRange::All))
            }
        }
    }
    fn ResolverCurrentPropertyName(
        &self,
        resolver: &Self::CascadeResolver,
    ) -> Option<CSSPropertyName> {
        resolver.current.clone()
    }
    fn PrimitiveIsCalculated(&self, value: &ResolverValue<Self>) -> bool {
        value.IsMathFunctionValue()
    }
    fn PrimitiveIsPx(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_literal(value).is_some_and(|value| value.GetType() == UnitType::kPixels)
            || Self::primitive_math_value(value)
                .is_some_and(|value| value.Category() == CalculationResultCategory::Length)
    }
    fn PrimitiveIsPercentage(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_literal(value).is_some_and(|value| value.GetType() == UnitType::kPercentage)
            || Self::primitive_math_value(value)
                .is_some_and(|value| value.Category() == CalculationResultCategory::Percent)
    }
    fn PrimitiveIsLength(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_literal(value).is_some_and(|value| value.IsLength())
            || Self::primitive_math_value(value)
                .is_some_and(|value| value.Category() == CalculationResultCategory::Length)
    }
    fn PrimitiveIsResolvableBeforeLayout(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_math_value(value)
            .is_none_or(|value| value.Category() != CalculationResultCategory::LengthFunction)
    }
    fn PrimitiveIsNumber(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_literal(value).is_some_and(|value| value.IsNumber())
            || Self::primitive_math_value(value)
                .is_some_and(|value| value.Category() == CalculationResultCategory::Number)
    }
    fn PrimitiveIsAngle(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_literal(value).is_some_and(|value| {
            matches!(
                value.GetType(),
                UnitType::kDegrees | UnitType::kRadians | UnitType::kGradians | UnitType::kTurns
            )
        }) || Self::primitive_math_value(value)
            .is_some_and(|value| value.Category() == CalculationResultCategory::Angle)
    }
    fn PrimitiveIsTime(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_literal(value).is_some_and(|value| {
            matches!(
                value.GetType(),
                UnitType::kSeconds | UnitType::kMilliseconds
            )
        }) || Self::primitive_math_value(value)
            .is_some_and(|value| value.Category() == CalculationResultCategory::Time)
    }
    fn PrimitiveIsResolution(&self, value: &ResolverValue<Self>) -> bool {
        Self::primitive_literal(value).is_some_and(|value| value.IsResolution())
            || Self::primitive_math_value(value)
                .is_some_and(|value| value.Category() == CalculationResultCategory::Resolution)
    }
    fn PrimitiveConvertToUnzoomedLength(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> Length {
        let mut resolver = self.math_length_resolver(state);
        let result = if let Some(value) = Self::primitive_literal(value) {
            if value.GetType() == UnitType::kPercentage {
                Some(Length::Percent(CSSValueClampingUtils::ClampLength(
                    value.DoubleValue(),
                )))
            } else if value.IsLength() {
                resolver
                    .ComputeLength(value.DoubleValue(), value.GetType())
                    .ok()
                    .map(|pixels| Length::Fixed(CSSValueClampingUtils::ClampLength(pixels)))
            } else {
                None
            }
        } else if let Some(value) = Self::primitive_math_value(value) {
            value.ConvertToLength(&mut resolver).ok()
        } else {
            None
        };
        result.unwrap_or_else(|| {
            self.unsupported("primitive-length-conversion-context");
            Length::Fixed(0.0)
        })
    }
    fn PrimitiveCreateFromLength(&self, length: Length, zoom: f32) -> Rc<ResolverValue<Self>> {
        if length.IsFixed() {
            production_css_value::numeric(length.Pixels() as f64 / zoom as f64, UnitType::kPixels)
        } else if length.IsPercent() {
            production_css_value::numeric(length.PercentValue() as f64, UnitType::kPercentage)
        } else if length.IsCalculated() {
            match crate::css_math_expression_node::CSSMathExpressionNode::FromCalculationValue(
                &length.GetCalculationValue(),
                zoom,
            ) {
                Ok(expression) => production_css_value::math(
                    expression,
                    if length.GetCalculationValue().GetValueRange()
                        == LengthValueRange::kNonNegative
                    {
                        ValueRange::NonNegative
                    } else {
                        ValueRange::All
                    },
                ),
                Err(_) => {
                    self.unsupported("primitive-from-length-unsupported-calculation-node");
                    self.unset()
                }
            }
        } else {
            self.unsupported("primitive-from-length-unsupported-calculation-node");
            self.unset()
        }
    }
    fn PrimitiveComputeNumber(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64 {
        let percentage = self.PrimitiveIsPercentage(value);
        let value = self.compute_primitive(value, state, "primitive-number");
        if percentage {
            value / 100.0
        } else {
            value
        }
    }
    fn PrimitiveComputeDegrees(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64 {
        CSSValueClampingUtils::ClampAngle(self.compute_primitive(value, state, "primitive-degrees"))
    }
    fn PrimitiveComputeSeconds(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64 {
        CSSValueClampingUtils::ClampTime(self.compute_primitive(value, state, "primitive-seconds"))
    }
    fn PrimitiveComputeDotsPerPixel(
        &self,
        value: &ResolverValue<Self>,
        state: &StyleResolverState<'_, Self>,
    ) -> f64 {
        self.compute_primitive(value, state, "primitive-resolution")
    }
    fn NewNumericLiteralValue(&self, value: f64, unit: UnitType) -> Rc<ResolverValue<Self>> {
        production_css_value::numeric(value, unit)
    }
    fn FeatureBounds(
        &self,
        feature: &Self::MediaQueryFeatureExpNode,
    ) -> MediaQueryExpBounds<ResolverValue<Self>> {
        feature.Bounds().clone()
    }
    fn FeatureName(&self, feature: &Self::MediaQueryFeatureExpNode) -> AtomicString {
        feature.MediaFeature().clone()
    }
    fn FeatureReferenceValue(
        &self,
        feature: &Self::MediaQueryFeatureExpNode,
    ) -> Rc<ResolverValue<Self>> {
        let value = feature.ReferenceValue();
        Rc::new(ResolverValue::<Self>::new(
            CSSValuePayload::kUnparsedDeclarationClass(
                production_css_value::CSSUnparsedDeclarationValue {
                    data: value.data.clone(),
                    mode: value.mode,
                    base_url: value.base_url.clone(),
                },
            ),
        ))
    }
    fn EvalStyleRange(
        &self,
        reference: &ResolverValue<Self>,
        bound: &ResolverValue<Self>,
        op: MediaQueryOperator,
        left: bool,
    ) -> KleeneValue {
        crate::media_queries::media_query_evaluator::EvalStyleRangeValues(
            reference, bound, op, left,
        )
    }
    fn VariableEqualsIgnoringAttrTainting(
        &self,
        left: &Self::VariableData,
        right: &Self::VariableData,
    ) -> bool {
        left.original_text == right.original_text
            && left.is_animation_tainted == right.is_animation_tainted
    }
    fn ConsumeIfCondition(
        &self,
        context: &ParserContext<Self>,
        stream: &mut TokenStream<'_>,
    ) -> Rc<Self::ConditionalExpNode> {
        let mut parser = CSSIfParser::new(context);
        parser.ConsumeIfCondition(stream).unwrap_or_else(|| {
            self.unsupported("if-condition-parse");
            Rc::new(IfCondition::Unknown(stream.RemainingText().ToString()))
        })
    }
    fn EvaluateIfExpression(
        &self,
        expression: &Self::ConditionalExpNode,
        visitor: &mut dyn IfConditionVisitor<Self>,
    ) -> KleeneValue {
        struct Evaluator<'a, 'b> {
            visitor: &'a mut dyn IfConditionVisitor<ProductionStyleCascadeBackend<'b>>,
        }
        impl IfConditionEvaluator for Evaluator<'_, '_> {
            fn EvaluateMedia(&mut self, query: &QuerySet) -> KleeneValue {
                self.visitor.EvaluateMediaQuerySet(query)
            }
            fn EvaluateStyle(&mut self, query: &ContainerCondition) -> KleeneValue {
                struct StyleVisitor<'a, 'b> {
                    visitor: &'a mut dyn IfConditionVisitor<ProductionStyleCascadeBackend<'b>>,
                }
                impl
                    ConditionalExpNodeVisitor<
                        production_css_value::Value,
                        production_css_value::CSSUnparsedDeclarationValue,
                    > for StyleVisitor<'_, '_>
                {
                    fn EvaluateMediaQueryFeatureExpNode(
                        &mut self,
                        feature: &MediaQueryExp<
                            production_css_value::Value,
                            production_css_value::CSSUnparsedDeclarationValue,
                        >,
                    ) -> KleeneValue {
                        self.visitor.EvaluateMediaQueryFeatureExpNode(feature)
                    }
                }
                query.Evaluate(&mut StyleVisitor {
                    visitor: self.visitor,
                })
            }
        }
        expression.Evaluate(&mut Evaluator { visitor })
    }
    fn NavigationNodeTest(&self, node: &Self::NavigationExpNode) -> Rc<Self::NavigationTest> {
        self.unsupported("navigation-query");
        Rc::new(())
    }
    fn SetNeedsStyleUpdateOnNavigation(&self, document: &Self::Document) {
        self.unsupported("navigation-update")
    }
    fn EvaluateFunctionalNavigationQuery(
        &self,
        document: &Self::Document,
        test: &Self::NavigationTest,
    ) -> bool {
        self.unsupported("navigation-query");
        false
    }
    fn EvaluateFunctionalMediaQuery(
        &self,
        document: &Self::Document,
        query: &Self::MediaQuerySet,
    ) -> bool {
        let values = MediaValuesCached::new(&self.media);
        MediaQueryEvaluator::ForMediaValues(&values).Eval(query)
    }
    fn IsSimpleSum(&self, text: String) -> bool {
        IsSimpleLengthSum(&text)
    }
    fn NewCascadeResolver(&self, filter: CascadeFilter) -> Self::CascadeResolver {
        ProductionCascadeResolver {
            filter,
            ..Default::default()
        }
    }
    fn ResolverFilter(&self, resolver: &Self::CascadeResolver) -> CascadeFilter {
        resolver.filter
    }
    fn ResolverRejects(
        &self,
        resolver: &mut Self::CascadeResolver,
        property: &CSSProperty,
    ) -> bool {
        if resolver.filter.Accepts(property) {
            false
        } else {
            resolver.rejected_flags |= property.GetFlags();
            true
        }
    }
    fn ResolverCollectFlags(
        &self,
        resolver: &mut Self::CascadeResolver,
        property: &CSSProperty,
        origin: CascadeOrigin,
    ) {
        {
            let flags = property.GetFlags();
            resolver.flags |= flags;
            if origin == CascadeOrigin::kAuthor {
                resolver.author_flags |= flags;
            }
        }
    }
    fn ResolverFlags(&self, resolver: &Self::CascadeResolver) -> Flags {
        resolver.flags
    }
    fn ResolverAuthorFlags(&self, resolver: &Self::CascadeResolver) -> Flags {
        resolver.author_flags
    }
    fn ResolverRejectedFlags(&self, resolver: &Self::CascadeResolver) -> Flags {
        resolver.rejected_flags
    }
    fn CustomPropertyMetadata(
        &self,
        name: &AtomicString,
        document: &Self::Document,
    ) -> CSSProperty {
        let inherited = self
            .property_registry
            .and_then(|registry| registry.Registration(&name.Utf8()))
            .map_or(true, |registration| registration.Inherits());
        CSSProperty::new(
            CSSPropertyID::kVariable,
            CSSProperty::kProperty
                | CSSProperty::kLonghand
                | if inherited {
                    CSSProperty::kInherited
                } else {
                    0
                }
                | CSSProperty::kNotVisited
                | CSSProperty::kNotAnimation
                | CSSProperty::kNotLegacyOverlapping,
            ' ',
        )
    }
    fn SurrogateFor(&self, property: &CSSProperty, direction: WritingDirectionMode) -> CSSProperty {
        *CSSProperty::Get(ResolvePhysical(property.PropertyID(), direction))
    }
    fn ExpandCascade(
        &self,
        properties: &MatchedProperties,
        document: &Self::Document,
        index: usize,
    ) -> Vec<(CascadePriority, CSSPropertyName)> {
        if properties
            .properties
            .AsAny()
            .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
            .is_none()
        {
            self.unsupported("property-value-set-type");
            return Vec::new();
        }
        {
            let out = RefCell::new(Vec::new());
            cascade_expansion::ExpandCascade(
                properties,
                document,
                index,
                &ProductionCascadeExpansionBackend {
                    property_registry: self.property_registry,
                },
                |priority, name| {
                    out.borrow_mut()
                        .push((priority, CSSPropertyName::custom(name.clone())))
                },
                |priority, id| out.borrow_mut().push((priority, CSSPropertyName::new(id))),
            );
            out.into_inner()
        }
    }
    fn PropertyValueAt(
        &self,
        set: &dyn MatchedPropertySet,
        declaration: usize,
    ) -> Rc<ResolverValue<Self>> {
        {
            let Some(set) = set
                .AsAny()
                .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
            else {
                self.unsupported("property-value-set-type");
                return self.unset();
            };
            match set {
                CSSPropertyValueSetRuleHandle::Immutable(set) => set
                    .Properties()
                    .get(declaration)
                    .map(CSSPropertyValue::ValueRef),
                CSSPropertyValueSetRuleHandle::Mutable(set) => set
                    .borrow()
                    .Properties()
                    .get(declaration)
                    .map(CSSPropertyValue::ValueRef),
            }
            .unwrap_or_else(|| {
                self.unsupported("property-value-index");
                self.unset()
            })
        }
    }
    fn InterpolationEntries(
        &self,
        map: &Self::ActiveInterpolationsMap,
    ) -> Vec<(CSSPropertyName, Rc<Self::ActiveInterpolations>)> {
        map.entries.clone()
    }
    fn NativePropertyHandle(&self, property: &CSSProperty) -> Self::PropertyHandle {
        property.GetCSSPropertyName()
    }
    fn CustomPropertyHandle(&self, name: &AtomicString) -> Self::PropertyHandle {
        CSSPropertyName::custom(name.clone())
    }
    fn FindInterpolation(
        &self,
        map: &Self::ActiveInterpolationsMap,
        handle: &Self::PropertyHandle,
    ) -> Option<Rc<Self::ActiveInterpolations>> {
        map.entries
            .iter()
            .find(|(name, _)| name == handle)
            .map(|(_, interpolations)| interpolations.clone())
    }
    fn FirstInterpolationIsInvalidatable(
        &self,
        interpolations: &Self::ActiveInterpolations,
    ) -> bool {
        interpolations.kind == ProductionInterpolationKind::Invalidatable
    }
    fn ApplyInvalidatableInterpolationStack(
        &self,
        interpolations: &Self::ActiveInterpolations,
        state: &StyleResolverState<'_, Self>,
        cascade: &mut StyleCascade<'_, '_, Self>,
        resolver: &mut Self::CascadeResolver,
    ) {
        if interpolations.kind != ProductionInterpolationKind::Invalidatable {
            self.unsupported("invalidatable-interpolation-kind");
            return;
        }
        // The existing AnimationStyleSample boundary carries the result of
        // sampling each effect, not keyframe interpolation objects. It can
        // therefore express Blink's replace composite only. ApplyStack's
        // result for that representable subset is the highest composite-order
        // sample, i.e. the last effect in the document's BTreeMap order.
        let Some(value) = interpolations.values.last().cloned() else {
            self.unsupported("empty-invalidatable-interpolation-stack");
            return;
        };
        self.ApplySampledInterpolation(
            interpolations,
            value,
            CascadeOrigin::kAnimation,
            state,
            cascade,
            resolver,
        );
    }
    fn ApplyTransitionInterpolation(
        &self,
        interpolations: &Self::ActiveInterpolations,
        state: &StyleResolverState<'_, Self>,
        cascade: &mut StyleCascade<'_, '_, Self>,
        resolver: &mut Self::CascadeResolver,
    ) {
        if interpolations.kind != ProductionInterpolationKind::Transition
            || interpolations.values.len() != 1
        {
            self.unsupported("transition-interpolation-stack");
            return;
        }
        self.ApplySampledInterpolation(
            interpolations,
            interpolations.values[0].clone(),
            CascadeOrigin::kTransition,
            state,
            cascade,
            resolver,
        );
    }
    fn ApplyPhysicalProperty(
        &self,
        name: &CSSPropertyName,
        property: &CSSProperty,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
    ) {
        {
            if name.IsCustomProperty() || property.PropertyID() == CSSPropertyID::kVariable {
                self.ApplyCustomProperty(name, property, state, value);
                return;
            }
            let parent = state.ParentStyle();
            let parent = parent.as_ref().map(|p| Self::style(p));
            // cpp: style_resolver_state.cc:451-469 ResolveLightDarkPair.
            // Production longhand application is the native owner below the
            // translated cascade, so select the retained pair only after the
            // cascade-affecting color-scheme pass has established this
            // element's UsedColorScheme.
            let resolved_light_dark = match value.Payload() {
                CSSValuePayload::kLightDarkValuePairClass(pair) => {
                    let selected = if state.StyleBuilder().UsedColorScheme()
                        == layoutng_style::style::color_scheme::mojom::blink::ColorScheme::kLight
                    {
                        pair.AsValuePair().first.clone()
                    } else {
                        pair.AsValuePair().second.clone()
                    };
                    Some(state.ResolveLightDarkPair(selected))
                }
                _ => None,
            };
            let value = resolved_light_dark.as_deref().unwrap_or(value);
            // cpp: longhands_custom.cc Color::ApplyValue resolves system
            // colors through StyleResolverState before storing the retained
            // keyword and platform color. Keep that document-owned decision
            // at the cascade boundary: the generic property converter cannot
            // resolve `fieldtext`, `canvastext`, or accent-color keywords.
            if property.PropertyID() == CSSPropertyID::kColor {
                if let CSSValuePayload::kIdentifierClass(identifier) = value.Payload() {
                    if StyleColor::IsSystemColorIncludingDeprecatedKeyword(identifier.0) {
                        let unresolved = StyleColor::from_keyword(identifier.0);
                        let used_color_scheme = {
                            let builder = state.StyleBuilder();
                            builder.UsedColorScheme()
                        };
                        if let Some(color) =
                            super::production_style_resolver::ResolveProductionSystemColor(
                                &unresolved,
                                used_color_scheme,
                                self.system_colors,
                            )
                        {
                            let mut builder = state.StyleBuilderMut();
                            builder.SetColor(&color);
                            builder.SetColorIsInherited(false);
                            builder.SetColorIsCurrentColor(false);
                            return;
                        }
                    }
                }
            }
            // cpp: generated longhands.cc BackgroundImage::ApplyValue and
            // MaskImage::ApplyValue call StyleResolverState::ResolveForImage
            // for every layer. A light-dark() image is nested inside the
            // comma-separated CSSValueList, unlike a color-valued longhand's
            // top-level pair, so preserve that per-layer translated step here.
            let resolved_image_layers = if matches!(
                property.PropertyID(),
                CSSPropertyID::kBackgroundImage | CSSPropertyID::kMaskImage
            ) {
                if let CSSValuePayload::kValueListClass(list) = value.Payload() {
                    Some(production_css_value::list(
                        list.values
                            .iter()
                            .cloned()
                            .map(|image| state.ResolveGradients(image))
                            .collect(),
                        list.separator,
                    ))
                } else {
                    None
                }
            } else {
                None
            };
            let value = resolved_image_layers.as_deref().unwrap_or(value);
            let resolved_system_font = match value.Payload() {
                CSSValuePayload::kPendingSystemFontValueClass(system) => {
                    let Some(document) = self.document else {
                        self.unsupported("system-font-document-owner");
                        return;
                    };
                    match ResolveSystemFontLonghand(
                        property.PropertyID(),
                        system.system_font_id,
                        document,
                    ) {
                        Ok(value) => Some(value),
                        Err(_) => {
                            self.unsupported("system-font-platform-owner");
                            return;
                        }
                    }
                }
                _ => None,
            };
            let value = resolved_system_font.as_deref().unwrap_or(value);
            // cpp: longhands_custom.cc:8480-8499 Resize::ApplyValue. The
            // internal textarea value is deliberately resolved from the
            // retained document setting at application time and never mapped
            // to a constant in the generic keyword converter.
            if property.PropertyID() == CSSPropertyID::kResize
                && matches!(
                    value.Payload(),
                    CSSValuePayload::kIdentifierClass(identifier)
                        if identifier.0 == CSSValueID::kInternalTextareaAuto
                )
            {
                let Some(document) = self.document else {
                    self.unsupported("textarea-resize-settings-owner");
                    return;
                };
                state
                    .StyleBuilderMut()
                    .SetResize(if document.TextAreasAreResizable() {
                        foundation::EResize::kBoth
                    } else {
                        foundation::EResize::kNone
                    });
                return;
            }
            let result = if let Some(images) = self.image_resolver {
                production_style_builder::ApplyWithImageResolverColorSchemeAndRootStyle(
                    property.PropertyID(),
                    &mut state.StyleBuilderMut(),
                    parent,
                    value,
                    self.root_font_size,
                    &self.media,
                    images,
                    self.color_scheme_settings,
                    self.document_element_style
                        .get()
                        .map(ProductionStyleCascadeBackend::style),
                    self.text_link_text_color,
                    self.font_metrics,
                )
            } else {
                production_style_builder::ApplyWithColorSchemeSettingsAndRootStyle(
                    property.PropertyID(),
                    &mut state.StyleBuilderMut(),
                    parent,
                    value,
                    self.root_font_size,
                    &self.media,
                    self.color_scheme_settings,
                    self.document_element_style
                        .get()
                        .map(ProductionStyleCascadeBackend::style),
                    self.text_link_text_color,
                    self.font_metrics,
                )
            };
            if let Err(error) = result {
                if std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                    eprintln!(
                        "translated longhand fallback property={:?} class={:?} value={}",
                        property.PropertyID(),
                        value.GetClassType(),
                        value.CssText().Utf8()
                    );
                }
                use crate::properties::longhand_dispatch::LonghandApplicationError;
                self.fallback.borrow_mut().push(match error {
                    LonghandApplicationError::Unsupported(id) => {
                        ProductionCascadeFallback::UnsupportedProperty(id)
                    }
                    LonghandApplicationError::InvalidValue(id) => {
                        ProductionCascadeFallback::InvalidProperty(id)
                    }
                });
            }
        }
    }
    fn EnsureScopedValue(
        &self,
        value: Rc<ResolverValue<Self>>,
        scope: Option<&dyn TreeScope>,
    ) -> Rc<ResolverValue<Self>> {
        value
    }
    fn NewUnsetValue(&self) -> Rc<ResolverValue<Self>> {
        self.unset()
    }
    fn MathValueHasAnchorFunctions(&self, value: &ResolverValue<Self>) -> bool {
        // The translated CSSMathExpressionNode currently has no anchor or
        // anchor-size node variant: those functions remain parser-level typed
        // Unsupported and cannot be present in this payload. For every math
        // value admitted by the production parser the source predicate is
        // therefore exactly false.
        false
    }
    fn UnparsedContainsSafeAreaInsetBottom(&self, value: &ResolverValue<Self>) -> bool {
        let CSSValuePayload::kUnparsedDeclarationClass(value) = value.Payload() else {
            return false;
        };
        let mut stream = CSSParserTokenStream::FromTokenizer(VariableTokenReplay::new(&value.data));
        ContainsSafeAreaInsetBottom(&mut stream)
    }
    fn DocumentTreeScope(&self, document: &Self::Document) -> Rc<dyn TreeScope> {
        Rc::new(ProductionScope)
    }
    fn DocumentElement(&self, document: &Self::Document) -> Option<Rc<Self::Element>> {
        self.document_element.get().cloned()
    }
    fn StandardizedBrowserZoomEnabled(&self, document: &Self::Document) -> bool {
        false
    }
    fn SmallerViewportUnitsEnabled(&self) -> bool {
        false
    }
    fn BuilderScrollbarGutter(
        &self,
        builder: &layoutng_style::style::computed_style::ComputedStyleBuilder,
    ) -> Self::ScrollbarGutter {
        ()
    }
    fn ComputeScrollbarWidthsForViewportUnits(
        &self,
        document: &Self::Document,
        overflow_x: EOverflow,
        overflow_y: EOverflow,
        gutter: Self::ScrollbarGutter,
        width: EScrollbarWidth,
        writing_mode: WritingMode,
    ) -> gfx::Size {
        // cpp: paint_layer_scrollable_area.cc:1698-1735. Rechrom's current
        // browser host configures overlay scrollbars, so the source algorithm
        // returns an empty size before consulting overflow/gutter/thickness.
        // The style package has no non-overlay viewport input; such a host must
        // add that typed value at the StyleEngine boundary before this branch
        // can produce a non-zero subtraction.
        gfx::Size::default()
    }
    fn BorderImageLonghands(&self) -> &[CSSPropertyID] {
        &[
            CSSPropertyID::kBorderImageSource,
            CSSPropertyID::kBorderImageSlice,
            CSSPropertyID::kBorderImageWidth,
            CSSPropertyID::kBorderImageOutset,
            CSSPropertyID::kBorderImageRepeat,
        ]
    }
    fn CSSLineClampEnabled(&self) -> bool {
        false
    }
    fn CSSLineClampAsShorthandEnabled(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        css_syntax_component::{CSSSyntaxComponent, CSSSyntaxRepeat, CSSSyntaxType},
        css_value::CSSValuePayload,
        parser::{
            css_parser_mode::CSSParserMode,
            production_property_parser::{ParseCustomProperty, ParseDeclarationList},
        },
        production_css_value::PropertyValue,
        resolver::{
            match_result::MatchedPropertiesData,
            production_style_builder::custom_properties::{
                AttributeVariableResolver, EnvironmentVariableResolver,
            },
        },
    };
    use foundation::{EDisplay, EPosition};

    struct UnsupportedPropertySet;
    impl MatchedPropertySet for UnsupportedPropertySet {
        fn AsAny(&self) -> &dyn std::any::Any {
            self
        }
        fn Equals(&self, other: &dyn MatchedPropertySet) -> bool {
            other.AsAny().is::<Self>()
        }
        fn GetHash(&self) -> u32 {
            0
        }
        fn ModifiedSinceHashing(&self) -> bool {
            false
        }
    }

    struct MissingMixinOwner;
    impl MixinParameterBindings for MissingMixinOwner {
        fn AsAny(&self) -> &dyn std::any::Any {
            self
        }
        fn Equals(&self, other: &dyn MixinParameterBindings) -> bool {
            other.AsAny().is::<Self>()
        }
        fn GetHash(&self) -> u32 {
            0x6d69_7869
        }
    }

    fn media() -> MediaValuesCachedData {
        MediaValuesCachedData {
            media_type: String::from("screen"),
            viewport_width: 800.0,
            viewport_height: 600.0,
            large_viewport_width: 800.0,
            large_viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        }
    }

    fn ordinary_author_result(source: &str) -> MatchResult {
        let sheet = production_style_sheet::ParseStyleSheet(
            &String::from(source),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let declarations = sheet
            .RetainedDeclarationsForProjectedRule(0)
            .expect("first rule retains typed declarations");
        let mut result = MatchResult::default();
        result.BeginAddingAuthorRulesForTreeScope(Rc::new(ProductionScope));
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kAuthor;
        result.AddMatchedProperties(declarations, None, data);
        result
    }

    fn parsed_properties(source: &str) -> Vec<PropertyValue> {
        let parsed = ParseDeclarationList(&String::from(source), CSSParserMode::kHTMLStandardMode);
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(!parsed.properties.is_empty());
        parsed.properties
    }

    fn inputs<'a>(
        media: MediaValuesCachedData,
        base: ComputedStyleHandle,
        match_result: MatchResult,
    ) -> ProductionCascadeInputs<'a> {
        ProductionCascadeInputs {
            document: None,
            media,
            base,
            parent: None,
            root_style: None,
            match_result,
            root_font_size: 16.0,
            color_scheme_settings: production_style_builder::ColorSchemeSettings::default(),
            system_colors:
                super::super::production_style_resolver::ProductionSystemColorContext::default(),
            text_link_text_color: None,
            is_document_element: false,
            pseudo_id: PseudoId::kPseudoIdNone,
            originating_style: None,
            subject_node: None,
            container_runtime: None,
            custom_properties: None,
            parent_custom_properties: None,
            substitution_context: SubstitutionContext::default(),
            image_resolver: None,
            property_registry: None,
            function_registry: None,
            animations: None,
            transitions: None,
            font_metrics: None,
        }
    }

    fn variable_data(text: &str) -> Rc<CSSVariableData> {
        let property = ParseCustomProperty(
            "--fixture",
            &String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let CSSValuePayload::kUnparsedDeclarationClass(value) = property.Value().Payload() else {
            panic!("fixture custom property must retain tokens")
        };
        value.data.clone()
    }

    struct TestEnvironment(Rc<CSSVariableData>);
    impl EnvironmentVariableResolver for TestEnvironment {
        fn ViewportSegmentsEnabled(&self) -> bool {
            false
        }
        fn ResolveEnvironmentVariable(
            &self,
            name: &AtomicString,
            indices: &[u32],
        ) -> Option<Rc<CSSVariableData>> {
            (name == &AtomicString::from_str("fixture-size") && indices.is_empty())
                .then(|| self.0.clone())
        }
    }

    struct TestAttributes;
    impl AttributeVariableResolver for TestAttributes {
        fn CSSArgumentGrammarEnabled(&self) -> bool {
            false
        }
        fn SetHasAttrFunction(&self) {}
        fn GetAttribute(&self, local_name: &AtomicString) -> String {
            (local_name == &AtomicString::from_str("data-size"))
                .then(|| String::from("43"))
                .unwrap_or_default()
        }
    }

    #[test]
    fn translated_apply_writes_ordinary_properties_to_native_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let style = ApplyMatchResult(inputs(
            media,
            initial,
            ordinary_author_result(
                "div { display:block; position:absolute; width:25px; opacity:.5 }",
            ),
        ))
        .expect("ordinary declarations require no compatibility fallback");
        let style = ProductionStyleCascadeBackend::style(&style);
        assert_eq!(style.Display(), EDisplay::kBlock);
        assert_eq!(style.GetPosition(), EPosition::kAbsolute);
        assert_eq!(style.Width().Pixels(), 25.0);
        assert_eq!(style.Opacity(), 0.5);
    }

    #[test]
    fn invalidatable_interpolation_stack_uses_animation_origin_and_effect_order() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let animations = ProductionActiveInterpolationsMap::FromEffectValues(
            [
                parsed_properties("width:40px;opacity:.4"),
                parsed_properties("width:80px;opacity:.8"),
            ],
            ProductionInterpolationKind::Invalidatable,
        );
        let mut request = inputs(
            media,
            initial,
            ordinary_author_result("div { width:10px; opacity:.2!important }"),
        );
        request.animations = Some(&animations);
        let style = ApplyMatchResult(request)
            .expect("replace animation samples are a complete typed interpolation stack");
        let style = ProductionStyleCascadeBackend::style(&style);
        assert_eq!(style.Width().Pixels(), 80.0);
        assert_eq!(style.Opacity(), 0.2, "author !important beats animation");
    }

    #[test]
    fn transition_interpolation_callback_beats_important_author_declaration() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let transitions = ProductionActiveInterpolationsMap::FromParsedProperties(
            parsed_properties("width:70px"),
            ProductionInterpolationKind::Transition,
        );
        let mut request = inputs(
            media,
            initial,
            ordinary_author_result("div { width:10px!important }"),
        );
        request.transitions = Some(&transitions);
        let style = ApplyMatchResult(request)
            .expect("single typed transition interpolation should apply without fallback");
        assert_eq!(
            ProductionStyleCascadeBackend::style(&style)
                .Width()
                .Pixels(),
            70.0
        );
    }

    #[test]
    fn animated_custom_property_is_visible_to_dependent_longhand_in_same_cascade() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let animations = ProductionActiveInterpolationsMap::FromParsedProperties(
            parsed_properties("--size:37px"),
            ProductionInterpolationKind::Invalidatable,
        );
        let mut request = inputs(
            media.clone(),
            initial.clone(),
            ordinary_author_result("div { --size:11px; width:var(--size) }"),
        );
        request.animations = Some(&animations);
        let output = ApplyMatchResult(request)
            .expect("sampled custom property stays in the translated cascade");
        assert_eq!(
            ProductionStyleCascadeBackend::style(&output)
                .Width()
                .Pixels(),
            37.0
        );
        let animated = output
            .custom_properties
            .Get("--size")
            .expect("sampled custom property is retained");
        assert_eq!(animated.Serialize().Utf8(), "37px");
        assert!(animated.is_animation_tainted);

        let transitions = ProductionActiveInterpolationsMap::FromParsedProperties(
            parsed_properties("--size:59px"),
            ProductionInterpolationKind::Transition,
        );
        let mut request = inputs(
            media,
            initial,
            ordinary_author_result("div { --size:11px; width:var(--size) }"),
        );
        request.animations = Some(&animations);
        request.transitions = Some(&transitions);
        let output = ApplyMatchResult(request)
            .expect("transition custom property stays in the translated cascade");
        assert_eq!(
            ProductionStyleCascadeBackend::style(&output)
                .Width()
                .Pixels(),
            59.0,
            "transition priority wins before var() substitution"
        );
        let transitioned = output
            .custom_properties
            .Get("--size")
            .expect("transitioned custom property is retained");
        assert_eq!(transitioned.Serialize().Utf8(), "59px");
        assert!(transitioned.is_animation_tainted);
    }

    #[test]
    fn missing_mixin_owner_returns_whole_cascade_typed_fallback() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let sheet = production_style_sheet::ParseStyleSheet(
            &String::from("div { width:var(--mixin-width, 10px) }"),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let declarations = sheet
            .RetainedDeclarationsForProjectedRule(0)
            .expect("rule retains typed declaration owner");
        let mut result = MatchResult::default();
        result.BeginAddingAuthorRulesForTreeScope(Rc::new(ProductionScope));
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kAuthor;
        result.AddMatchedProperties(declarations, Some(Rc::new(MissingMixinOwner)), data);

        let applied = ApplyMatchResult(inputs(media, initial, result));
        assert!(matches!(
            applied,
            Err(ref fallbacks)
                if fallbacks.contains(&ProductionCascadeFallback::UnsupportedFeature(
                    "mixin-owner"
                ))
        ));
    }

    #[test]
    fn document_update_consumes_sampled_animations_through_style_cascade() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
            "#target { width:10px; height:10px; opacity:.2!important }",
        ));
        let target = (0..owner.GetDocument().NodeCount())
            .find(|&index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "target")
            })
            .unwrap();
        let target_id = owner.GetDocument().Node(target).Id();
        owner.GetDocumentMut().SetAnimationStyle(
            target_id,
            1,
            crate::ParseCSSDeclarationList("width:40px;height:40px;opacity:.4"),
        );
        owner.GetDocumentMut().SetAnimationStyle(
            target_id,
            2,
            crate::ParseCSSDeclarationList("width:80px;height:80px;opacity:.8"),
        );
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        let style = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };
        let stats = engine.TranslatedRuleShadowStats();
        assert_eq!(style.Width().Pixels(), 80.0, "{stats:?}");
        assert_eq!(style.Height().Pixels(), 80.0, "{stats:?}");
        assert_eq!(
            style.Opacity(),
            0.2,
            "author !important beats animation: {stats:?}"
        );
        assert_eq!(stats.translated_style_cascade_animation_attempts, 1);
        assert_eq!(stats.translated_style_cascade_animation_applied, 1);
        assert_eq!(stats.translated_style_cascade_animation_fallbacks, 0);
    }

    #[test]
    fn translated_apply_reuses_typed_custom_env_and_attr_owners() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let environment = TestEnvironment(variable_data("41px"));
        let attributes = TestAttributes;
        let mut request = inputs(
            media,
            initial,
            ordinary_author_result(
                "div { --fixture-width:37px; width:var(--fixture-width); \
                 height:env(fixture-size); min-width:attr(data-size px) }",
            ),
        );
        request.substitution_context = SubstitutionContext {
            environment: Some(&environment),
            attributes: Some(&attributes),
        };
        let output = ApplyMatchResult(request)
            .expect("typed substitution owners should avoid compatibility fallback");
        let computed = output
            .custom_properties
            .Get("--fixture-width")
            .expect("the same translated cascade must publish its custom-property map");
        assert_eq!(computed.Serialize().Utf8(), "37px");
        let style = ProductionStyleCascadeBackend::style(&output);
        assert_eq!(style.Width().Pixels(), 37.0);
        assert_eq!(style.Height().Pixels(), 41.0);
        assert_eq!(style.MinWidth().Pixels(), 43.0);
        assert!(style.HasVariableReference());
        assert!(style.HasEnv());
        assert!(style.HasAttrFunction());
    }

    fn length_registry(name: &str, inherits: bool, initial: &str) -> PropertyRegistry {
        let syntax = CSSSyntaxDefinition::new(vec![CSSSyntaxComponent::new(
            CSSSyntaxType::kLength,
            &String::default(),
            CSSSyntaxRepeat::kNone,
        )]);
        let registration = crate::property_registry::PropertyRegistration::new(
            syntax,
            inherits,
            Some(variable_data(initial)),
        )
        .expect("fixture registration must be valid");
        let mut registry = PropertyRegistry::default();
        registry
            .RegisterProperty(AtomicString::from_str(name), registration)
            .expect("fixture name is registered once");
        registry
    }

    #[test]
    fn translated_cascade_computes_registered_custom_after_font_size() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let registry = length_registry("--size", false, "5px");
        let mut request = inputs(
            media,
            initial,
            ordinary_author_result("div { font-size:10px; --size:2em; width:var(--size) }"),
        );
        request.property_registry = Some(&registry);
        let output = ApplyMatchResult(request)
            .expect("registered custom property must stay on the translated cascade");
        let style = ProductionStyleCascadeBackend::style(&output);
        assert_eq!(style.GetFontDescription().ComputedSize(), 10.0);
        assert_eq!(style.Width().Pixels(), 20.0);
        assert_eq!(
            output
                .custom_properties
                .Get("--size")
                .expect("registered value is retained")
                .Serialize()
                .Utf8(),
            "20px"
        );
    }

    #[test]
    fn translated_cascade_keeps_registered_noninherit_initial_and_wide_keywords() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let registry = length_registry("--size", false, "5px");
        let parent_declarations = ParseDeclarationList(
            &String::from("--size:33px;--shared:7px"),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parent_declarations.errors.is_empty());
        let parent = CustomProperties::ComputeWithRegistry(
            None,
            parent_declarations.properties.iter(),
            SubstitutionContext::default(),
            Some(&registry),
            &|number, unit| {
                production_style_builder::Pixels(
                    CSSPropertyID::kVariable,
                    number,
                    unit,
                    16.0,
                    16.0,
                    &media,
                )
                .map_err(|_| VariableResolutionError::Invalid)
            },
        );
        let inherited_shared = parent.Get("--shared").unwrap().clone();
        for (declaration, expected) in [
            ("", "5px"),
            ("--size:initial", "5px"),
            ("--size:unset", "5px"),
            ("--size:inherit", "33px"),
        ] {
            let initial = super::super::production_style_resolver::InitialStyleForDocument(
                &media,
                production_style_builder::ColorSchemeSettings::default(),
            );
            let source = format!("div {{ {declaration}; width:var(--size) }}");
            let mut request = inputs(media.clone(), initial, ordinary_author_result(&source));
            request.parent_custom_properties = Some(&parent);
            request.property_registry = Some(&registry);
            let output = ApplyMatchResult(request)
                .unwrap_or_else(|fallbacks| panic!("unexpected fallback: {fallbacks:?}"));
            assert_eq!(
                output
                    .custom_properties
                    .Get("--size")
                    .unwrap()
                    .Serialize()
                    .Utf8(),
                expected
            );
            assert!(Rc::ptr_eq(
                output.custom_properties.Get("--shared").unwrap(),
                &inherited_shared
            ));
        }
    }

    #[test]
    fn translated_cascade_returns_cyclic_custom_map_atomically() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let output = ApplyMatchResult(inputs(
            media,
            initial,
            ordinary_author_result("div { --a:var(--b); --b:var(--a); width:var(--a,19px) }"),
        ))
        .expect("custom-property cycles are a computed result, not a partial fallback");
        assert_eq!(
            ProductionStyleCascadeBackend::style(&output)
                .Width()
                .Pixels(),
            19.0
        );
        assert!(output.custom_properties.Get("--a").is_none());
        assert!(["--a", "--b"].into_iter().all(|name| {
            !matches!(
                output.custom_properties.values.get(name),
                Some(production_style_builder::custom_properties::ComputedVariableValue::Data(_))
            )
        }));
    }

    #[test]
    fn translated_apply_resolves_if_supports_media_and_style_conditions() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let custom_declarations = ParseDeclarationList(
            &String::from("--theme:dark"),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(custom_declarations.errors.is_empty());
        let custom = CustomProperties::Compute(None, custom_declarations.properties.iter());
        let mut request = inputs(
            media,
            initial,
            ordinary_author_result(
                "div { width:if(media(width > 500px):31px;else:7px); \
                 height:if(supports(display:block):41px;else:9px); \
                 min-width:if(style(--theme:dark):43px;else:11px) }",
            ),
        );
        request.custom_properties = Some(&custom);
        let style = ApplyMatchResult(request)
            .expect("typed if() condition owners should avoid compatibility fallback");
        let style = ProductionStyleCascadeBackend::style(&style);
        assert_eq!(style.Width().Pixels(), 31.0);
        assert_eq!(style.Height().Pixels(), 41.0);
        assert_eq!(style.MinWidth().Pixels(), 43.0);
        assert!(style.AffectedByFunctionalMedia());
    }

    #[test]
    fn document_update_uses_translated_if_media_and_supports_path() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        let source = "#target { --theme:dark; \
             width:if(media(width > 500px):53px;else:13px); \
             height:if(supports(display:block):59px;else:17px); \
             min-width:if(style(--theme:dark):61px;else:19px) }";
        let sheet = crate::ParseCSS(source);
        let typed = sheet
            .ParsedBacking::<production_style_sheet::ProductionStyleSheet>()
            .expect("ParseCSS retains the typed stylesheet owner");
        assert!(typed.diagnostics.is_empty(), "{:?}", typed.diagnostics);
        assert!(typed.RetainedDeclarationsForProjectedRule(0).is_some());
        assert!(typed.TypedDeclarationsForProjectedRule(0).is_some());
        let rule_set = typed.BuildRuleSet(
            &production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(&media),
        );
        assert_eq!(rule_set.AllRuleData().len(), 1);
        let routing = crate::production_rule_matching::ProductionRuleRouting::Build(
            &rule_set,
            typed.as_ref(),
        );
        assert_eq!(routing.AuthoritativeRuleCount(), 1);
        owner.GetDocumentMut().AppendStyleSheet(sheet);
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        let target = (0..owner.GetDocument().NodeCount())
            .find(|&index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == "target")
            })
            .unwrap();
        let style = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };
        let stats = engine.TranslatedRuleShadowStats();
        assert!(
            style.Width().IsFixed(),
            "width={:?} {stats:?}",
            style.Width()
        );
        assert!(
            style.Height().IsFixed(),
            "height={:?} {stats:?}",
            style.Height()
        );
        assert!(
            style.MinWidth().IsFixed(),
            "min-width={:?} {stats:?}",
            style.MinWidth()
        );
        assert_eq!(style.Width().Pixels(), 53.0);
        assert_eq!(style.Height().Pixels(), 59.0);
        assert_eq!(style.MinWidth().Pixels(), 61.0);
        assert!(style.AffectedByFunctionalMedia());
        assert!(stats.translated_style_cascade_applied > 0, "{stats:?}");
        assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    }

    #[test]
    fn document_function_registry_resolves_through_translated_style_cascade() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let sheet = production_style_sheet::ParseStyleSheet(
            &String::from(
                "@function --size(--base type(<length>): 4px) returns type(<length>) {\
                     result: var(--base);\
                 }\
                 .target { width: --size(23px); }",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let registry = production_style_sheet::ProductionFunctionRegistry::FromStyleSheets(
            &[&sheet],
            &production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(&media),
        );
        let declarations = sheet
            .RetainedDeclarationsForProjectedRule(0)
            .expect("style rule retains typed declarations");
        let mut result = MatchResult::default();
        result.BeginAddingAuthorRulesForTreeScope(Rc::new(ProductionScope));
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kAuthor;
        result.AddMatchedProperties(declarations, None, data);
        let base = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let mut cascade_inputs = inputs(media, base, result);
        cascade_inputs.function_registry = Some(&registry);
        let style = ApplyMatchResult(cascade_inputs)
            .expect("registered CSS function should resolve without compatibility fallback");
        let style = ProductionStyleCascadeBackend::style(&style);
        assert!(style.Width().IsFixed(), "width={:?}", style.Width());
        assert_eq!(style.Width().Pixels(), 23.0);
        assert!(style.AffectedByCSSFunction());
    }

    #[test]
    fn document_update_resolves_registered_css_function_without_fallback() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
            "@function --size(--base type(<length>): 4px) returns type(<length>) {\
                 result: var(--base);\
             }\
             #target { width: --size(29px); }",
        ));
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        let target = (0..owner.GetDocument().NodeCount())
            .find(|&index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == "target")
            })
            .unwrap();
        let style = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };
        let stats = engine.TranslatedRuleShadowStats();
        assert!(
            style.Width().IsFixed(),
            "width={:?} {stats:?}",
            style.Width()
        );
        assert_eq!(style.Width().Pixels(), 29.0);
        assert!(style.AffectedByCSSFunction());
        assert!(stats.translated_style_cascade_applied > 0, "{stats:?}");
        assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    }

    #[test]
    fn translated_mixin_bindings_resolve_arguments_through_style_cascade() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let sheet = production_style_sheet::ParseStyleSheet(
            &String::from(
                "@mixin --box(--edge type(<length>): 3px) {\
                     @result { width: var(--edge); }\
                 }\
                 .target { @apply --box(31px); }",
            ),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let rule_set = sheet.BuildRuleSet(
            &production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(&media),
        );
        let expanded = rule_set
            .AllRuleData()
            .iter()
            .find_map(|data| {
                data.Rule()
                    .GetMixinParameterBindings()
                    .map(|bindings| (data.Rule().Properties(), bindings.clone()))
            })
            .expect("@apply expansion retains its parser-owned mixin bindings");
        let mut result = MatchResult::default();
        result.BeginAddingAuthorRulesForTreeScope(Rc::new(ProductionScope));
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kAuthor;
        result.AddMatchedProperties(
            expanded.0,
            Some(expanded.1 as Rc<dyn MixinParameterBindings>),
            data,
        );
        let base = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let style = ApplyMatchResult(inputs(media, base, result))
            .expect("mixin argument substitution should stay on translated StyleCascade");
        let style = ProductionStyleCascadeBackend::style(&style);
        assert!(style.Width().IsFixed(), "width={:?}", style.Width());
        assert_eq!(style.Width().Pixels(), 31.0);
    }

    #[test]
    fn document_update_resolves_apply_mixin_through_translated_owners() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
            "@mixin --box(--edge type(<length>): 3px) {\
                 @result { width: var(--edge); }\
             }\
             #target { @apply --box(37px); }",
        ));
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        let target = (0..owner.GetDocument().NodeCount())
            .find(|&index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == "target")
            })
            .unwrap();
        let style = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };
        let stats = engine.TranslatedRuleShadowStats();
        assert!(
            style.Width().IsFixed(),
            "width={:?} {stats:?}",
            style.Width()
        );
        assert_eq!(style.Width().Pixels(), 37.0);
        assert!(stats.translated_style_cascade_applied > 0, "{stats:?}");
        assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    }

    #[test]
    fn functional_condition_rule_callbacks_keep_typed_rule_owners() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let backend = ProductionStyleCascadeBackend::new(
            None,
            media,
            16.0,
            production_style_builder::ColorSchemeSettings::default(),
            super::super::production_style_resolver::ProductionSystemColorContext::default(),
            None,
            None,
            None,
            SubstitutionContext::default(),
            None,
            None,
            None,
            None,
        );

        let supports = crate::style_rule::StyleRuleBase::<ProductionSheetBackend>::Supports(
            crate::style_rule::StyleRuleSupports::new(String::from("display:block"), true, vec![]),
        );
        assert_eq!(
            backend.FunctionRuleKind(&supports),
            FunctionRuleKind::Supports
        );
        assert!(backend.SupportsConditionIsSupported(&supports));

        let media_text = String::from("(width > 500px)");
        let media_queries = crate::parser::media_query_parser::MediaQueryParser::new(
            crate::media_queries::media_query_backend::MediaQueryFeatureFlags::default(),
        )
        .ParseMediaQuerySet(StringView::from(&media_text));
        let media_rule = crate::style_rule::StyleRuleBase::<ProductionSheetBackend>::Media(
            crate::style_rule::StyleRuleMedia::new(Some(Rc::new(media_queries)), vec![]),
        );
        assert_eq!(
            backend.FunctionRuleKind(&media_rule),
            FunctionRuleKind::Media
        );
        assert!(backend.EvaluateFunctionalMediaRule(&media_rule, &()));

        let context = StrictCSSParserContext::<ProductionSheetBackend>(
            crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
        );
        let container_text = String::from("(width > 400px)");
        let container_queries = crate::parser::container_query_parser::ContainerQueryParser::new(
            crate::production_container_parser::ProductionContainerBackend::new(&context),
        )
        .ParseContainerQuerySetString(&container_text)
        .expect("production container parser retains the typed query set");
        let container_rule = crate::style_rule::StyleRuleBase::<ProductionSheetBackend>::Container(
            crate::style_rule::StyleRuleContainer::new(Rc::clone(&container_queries), vec![]),
        );
        assert_eq!(
            backend.FunctionRuleKind(&container_rule),
            FunctionRuleKind::Container
        );
        let callback_queries = backend.ContainerRuleQueries(&container_rule);
        assert_eq!(callback_queries.ToString(), container_queries.ToString());
        assert!(backend.take_fallbacks().is_empty());
    }

    #[test]
    fn translated_apply_matches_document_style_engine_for_ordinary_properties() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let source = "#target { display:block; position:absolute; width:25px; opacity:.5 }";
        let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
        owner
            .GetDocumentMut()
            .AppendStyleSheet(crate::ParseCSS(source));
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        let target = (0..owner.GetDocument().NodeCount())
            .find(|&index| {
                owner
                    .GetDocument()
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "target")
            })
            .unwrap();
        let expected = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };

        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let actual = ApplyMatchResult(inputs(media, initial, ordinary_author_result(source)))
            .expect("ordinary declarations require no compatibility fallback");
        let actual = ProductionStyleCascadeBackend::style(&actual);
        assert_eq!(actual.Display(), expected.Display());
        assert_eq!(actual.GetPosition(), expected.GetPosition());
        assert_eq!(actual.Width(), expected.Width());
        assert_eq!(actual.Opacity(), expected.Opacity());
    }

    #[test]
    fn unsupported_match_storage_returns_typed_fallback_instead_of_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let mut result = MatchResult::default();
        let mut data = MatchedPropertiesData::default();
        data.origin = CascadeOrigin::kUserAgent;
        result.AddMatchedProperties(Rc::new(UnsupportedPropertySet), None, data);
        let applied = ApplyMatchResult(inputs(media, initial, result));
        assert!(matches!(
            applied,
            Err(ref fallbacks)
                if fallbacks == &[ProductionCascadeFallback::UnsupportedFeature(
                    "property-value-set-type"
                )]
        ));
    }

    #[test]
    fn numeric_syntax_and_primitive_callbacks_use_typed_css_values() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let backend = ProductionStyleCascadeBackend::new(
            None,
            media,
            16.0,
            production_style_builder::ColorSchemeSettings::default(),
            super::super::production_style_resolver::ProductionSystemColorContext::default(),
            None,
            None,
            None,
            SubstitutionContext::default(),
            None,
            None,
            None,
            None,
        );
        let document = ();
        let element = Rc::new(ProductionElement {
            node: None,
            parent: None,
            root: None,
            pseudo_id: PseudoId::kPseudoIdNone,
        });
        let state = StyleResolverState::new(&backend, &document, element, None, None);
        state.CreateNewStyle(
            ProductionStyleCascadeBackend::style(&initial),
            ProductionStyleCascadeBackend::style(&initial),
            None,
        );
        let context = StrictCSSParserContext::<production_style_sheet::Backend>(
            crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
        );
        let parse = |source: &str| {
            let mut local = CSSParserLocalContext::CreateWithoutPropertyForSyntaxParsing();
            backend
                .NumericSyntaxParse(String::from(source), &context, &mut local, false, false)
                .unwrap_or_else(|| panic!("numeric syntax must parse {source}"))
        };

        let length = parse("calc(1in + 24px)");
        assert!(backend.PrimitiveIsCalculated(&length));
        assert!(backend.PrimitiveIsLength(&length));
        let length = backend.PrimitiveConvertToUnzoomedLength(&length, &state);
        assert!(length.IsFixed());
        assert_eq!(length.Pixels(), 120.0);
        let length = backend.PrimitiveCreateFromLength(length, 1.0);
        assert!(matches!(
            length.Payload(),
            CSSValuePayload::kNumericLiteralClass(value)
                if value.GetType() == UnitType::kPixels && value.DoubleValue() == 120.0
        ));

        let number = parse("calc(.25 + .5)");
        assert!(backend.PrimitiveIsNumber(&number));
        assert_eq!(backend.PrimitiveComputeNumber(&number, &state), 0.75);
        let angle = parse("calc(.5turn + 90deg)");
        assert!(backend.PrimitiveIsAngle(&angle));
        assert_eq!(backend.PrimitiveComputeDegrees(&angle, &state), 270.0);
        let time = parse("calc(500ms + .5s)");
        assert!(backend.PrimitiveIsTime(&time));
        assert_eq!(backend.PrimitiveComputeSeconds(&time, &state), 1.0);
        let resolution = parse("calc(96dpi + 1dppx)");
        assert!(backend.PrimitiveIsResolution(&resolution));
        assert_eq!(
            backend.PrimitiveComputeDotsPerPixel(&resolution, &state),
            2.0
        );
        assert!(backend.take_fallbacks().is_empty());
    }

    #[test]
    fn calculated_length_round_trip_and_style_range_stay_typed() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let backend = ProductionStyleCascadeBackend::new(
            None,
            media,
            16.0,
            production_style_builder::ColorSchemeSettings::default(),
            super::super::production_style_resolver::ProductionSystemColorContext::default(),
            None,
            None,
            None,
            SubstitutionContext::default(),
            None,
            None,
            None,
            None,
        );
        let mixed = foundation::CalculationValue::new(
            foundation::PixelsAndPercent::new(12.0, 25.0, true, true),
            LengthValueRange::kAll,
        );
        let value = backend.PrimitiveCreateFromLength(Length::from_calculation_value(mixed), 1.0);
        assert!(matches!(
            value.Payload(),
            CSSValuePayload::kMathFunctionClass(value)
                if value.Category() == CalculationResultCategory::LengthFunction
        ));

        let reference = production_css_value::numeric(2.54, UnitType::kCentimeters);
        let bound = production_css_value::numeric(96.0, UnitType::kPixels);
        assert_eq!(
            backend.EvalStyleRange(&reference, &bound, MediaQueryOperator::kEq, false,),
            KleeneValue::kFalse,
            "StyleCascade canonicalizes units before calling EvalStyleRange"
        );
        let reference = production_css_value::numeric(96.0, UnitType::kPixels);
        assert_eq!(
            backend.EvalStyleRange(&reference, &bound, MediaQueryOperator::kEq, false,),
            KleeneValue::kTrue
        );

        let reference_data = variable_data("96px");
        let bound_value = production_css_value::unparsed(
            (*variable_data("100px")).clone(),
            CSSParserMode::kHTMLStandardMode,
        );
        let bounds = MediaQueryExpBounds::new(
            &crate::media_queries::media_query_exp::MediaQueryExpComparison::default(),
            &crate::media_queries::media_query_exp::MediaQueryExpComparison::new(
                &crate::media_queries::media_query_exp::MediaQueryExpValue::Value(bound_value),
                MediaQueryOperator::kLt,
            ),
        );
        let feature = MediaQueryExp::CreateStyleRange(
            Rc::new(production_css_value::CSSUnparsedDeclarationValue {
                data: reference_data.clone(),
                mode: CSSParserMode::kHTMLStandardMode,
                base_url: production_css_value::ProductionURL::Null,
            }),
            &bounds,
            true,
        )
        .unwrap();
        assert!(backend.FeatureBounds(&feature) == bounds);
        assert!(matches!(
            backend.FeatureReferenceValue(&feature).Payload(),
            CSSValuePayload::kUnparsedDeclarationClass(value)
                if Rc::ptr_eq(&value.data, &reference_data)
        ));
        assert!(backend.take_fallbacks().is_empty());
    }

    #[test]
    fn safe_area_simple_sum_matches_source_token_shape() {
        let _heap = foundation::LayoutHeapScope::new();
        let backend = ProductionStyleCascadeBackend::new(
            None,
            media(),
            16.0,
            production_style_builder::ColorSchemeSettings::default(),
            super::super::production_style_resolver::ProductionSystemColorContext::default(),
            None,
            None,
            None,
            SubstitutionContext::default(),
            None,
            None,
            None,
            None,
        );
        assert!(backend.IsSimpleSum(String::from("calc(1px + 2em - 3ch)")));
        assert!(backend.IsSimpleSum(String::from("1px + 2em")));
        assert!(!backend.IsSimpleSum(String::from("min(1px, 2px)")));
        assert!(!backend.IsSimpleSum(String::from("calc(1px * 2)")));
        assert!(!backend.IsSimpleSum(String::from("calc(1%)")));
        let contains = |text| {
            let value = production_css_value::unparsed(
                (*variable_data(text)).clone(),
                CSSParserMode::kHTMLStandardMode,
            );
            backend.UnparsedContainsSafeAreaInsetBottom(&value)
        };
        assert!(contains("env(safe-area-inset-bottom)"));
        assert!(contains("max(1px, env(safe-area-inset-bottom, 2px))"));
        assert!(!contains("env(safe-area-inset-top)"));
        assert!(!contains("env(SAFE-area-inset-bottom)"));
        assert!(backend.take_fallbacks().is_empty());
    }

    #[test]
    fn unparsed_pending_shorthand_and_ident_function_keep_typed_owners() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = media();
        let initial = super::super::production_style_resolver::InitialStyleForDocument(
            &media,
            production_style_builder::ColorSchemeSettings::default(),
        );
        let backend = ProductionStyleCascadeBackend::new(
            None,
            media,
            16.0,
            production_style_builder::ColorSchemeSettings::default(),
            super::super::production_style_resolver::ProductionSystemColorContext::default(),
            None,
            None,
            None,
            SubstitutionContext::default(),
            None,
            None,
            None,
            None,
        );
        let document = ();
        let state = StyleResolverState::new(
            &backend,
            &document,
            Rc::new(ProductionElement {
                node: None,
                parent: None,
                root: None,
                pseudo_id: PseudoId::kPseudoIdNone,
            }),
            None,
            None,
        );
        state.CreateNewStyle(
            ProductionStyleCascadeBackend::style(&initial),
            ProductionStyleCascadeBackend::style(&initial),
            None,
        );
        let context = StrictCSSParserContext::<production_style_sheet::Backend>(
            crate::parser::css_parser_context::SecureContextMode::kInsecureContext,
        );
        let data = backend
            .ParseVariableDeclarationValue(String::from("var(--edge, 2px)"), true, &context)
            .expect("declaration-value grammar retains tokenizer records");
        assert!(!data.tokens.is_empty());
        assert!(data.NeedsVariableResolution());
        assert!(data.is_animation_tainted);

        let unparsed = backend.NewUnparsedValue(data.clone(), Some(context.clone()));
        assert!(Rc::ptr_eq(&backend.UnparsedData(&unparsed), &data));
        assert_eq!(
            backend.UnparsedParserContext(&unparsed).unwrap().Mode(),
            context.Mode()
        );
        let pending =
            production_css_value::pending_substitution(CSSPropertyID::kBorder, unparsed.clone());
        let (shorthand, retained) = backend.PendingShorthand(&pending);
        assert_eq!(shorthand, CSSPropertyID::kBorder);
        assert!(Rc::ptr_eq(&retained, &unparsed));

        let source = String::from("ident(foo \"-\" 7 ident(bar))");
        let mut stream = TokenStream::new(StringView::from(&source), 0);
        let ident = backend.ConsumeIdentFunction(&mut stream, &context);
        assert!(stream.AtEnd());
        assert_eq!(backend.ComputeIdent(&ident, &state).Utf8(), "foo-7bar");

        let source = String::from("1px,{var(--x,2px)},rgb(1,2,3)");
        let mut stream = TokenStream::new(StringView::from(&source), 0);
        assert_eq!(
            backend.ConsumeFunctionArguments(&mut stream, 3),
            vec![
                String::from("1px"),
                String::from("var(--x,2px)"),
                String::from("rgb(1,2,3)"),
            ]
        );
        assert!(stream.AtEnd());
        assert!(!backend.CSSFunctionsEnabled());
        assert!(backend.take_fallbacks().is_empty());

        assert!(backend
            .FindFunctionAcrossScopes(
                &document,
                &AtomicString::from_str("--missing-function"),
                None,
            )
            .is_none());
        assert_eq!(
            backend.take_fallbacks(),
            vec![ProductionCascadeFallback::UnsupportedFeature(
                "css-function-owner"
            )]
        );
    }
}

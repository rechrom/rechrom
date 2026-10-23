// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Production host adapter for the translated `StyleResolver` lifecycle.
//!
//! `StyleResolverBackend` deliberately models Blink's document-owned resolver,
//! so even its initial-style slice names the later rule/cache collaborators.
//! Rechrom keeps those collaborators behind `StyleEngine`; this adapter binds
//! the host inputs that the initial-style slice actually consumes without
//! exporting the translated resolver object across that boundary.
#![allow(non_snake_case)]

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

use font_engine::{FontDescription, FontSelectionValue, FontVariantEmoji, LayoutLocale};
use foundation::{
    AtomicString, CSSBitset as NativeCSSBitset, CSSPropertyID, CSSValueID,
    DisplayAdjustmentContext, EDisplay, EForcedColorAdjust, EInlineBlockBaselineEdge, EInsideLink,
    EInteractivity, EOrder, EOverflow, EPosition, EUserModify, Length, Member, Persistent,
    PhysicalOffset, TextDirection, Vector,
};
use layoutng_style::{
    css::style_color::StyleColor,
    style::{
        appearance::AppearanceValue,
        color_scheme::mojom::blink::ColorScheme,
        computed_style::{ComputedStyle, ComputedStyleBuilder},
        computed_style_base::IsAtShadowBoundary,
        computed_style_constants::PseudoId,
        computed_style_initial_values::ComputedStyleInitialValues,
        display_adjustment::AdjustComputedDisplayForLayout,
        gap_data_list::GapDataList,
        style_base_data::StyleBaseData as NativeStyleBaseData,
    },
};

use crate::{
    active_style_sheets::ActiveStyleSheet,
    cascade_layer_map::CascadeLayerMap,
    css_primitive_value::UnitType,
    css_property_value_set::{
        CSSPropertyValueSetRuleAdapter, CSSPropertyValueSetRuleHandle, MutableCSSPropertyValueSet,
    },
    css_value::CSSValuePayload,
    element_rule_collector::{ElementRuleCollector, MatchRequest, RuleSetGroup, SearchTextRequest},
    media_queries::{forced_colors::ForcedColors, MediaValuesCachedData},
    persistent_selector::PersistentSelectorService,
    production_css_value::{self, ProductionCSSValueDispatch},
    production_element_rule_collector::{
        ProductionElementResolveContext, ProductionElementRuleCollectorRuntime,
        ProductionStyleRecalcContext, ProductionTreeScope,
    },
    production_rule_context::ProductionPseudoRequest,
    production_style_sheet::Backend,
    properties::css_property::CSSProperty,
    property_registry::PropertyRegistry,
    resolver::{
        cascade_filter::CascadeFilter,
        cascade_origin::CascadeOrigin,
        match_flags::MatchFlag,
        match_result::MatchResult,
        matched_properties_cache::{
            AdditionalHash, Entry as MatchedPropertiesCacheEntry, MatchedPropertiesCache,
            OwnedKey as MatchedPropertiesCacheKey,
        },
        production_container_runtime::ProductionContainerRuntime,
        production_style_builder::custom_properties::{CustomProperties, SubstitutionContext},
        production_style_builder::{self, ColorSchemeSettings, URLImageResolver},
        production_style_cascade::{
            ApplyMatchResultWithFilter, ProductionActiveInterpolationsMap,
            ProductionCascadeFallback, ProductionCascadeInputs, SupportsBaseAppearance,
        },
        style_adjuster::{ElementType, ElementTypeForCache as StyleAdjusterElementTypeForCache},
        style_builder_converter::StyleBuilderConverter,
        style_resolver::{
            ResolverAnimationProperty, ResolverLengthFlag, ResolverMatchResultFlag,
            ResolverMathMLKind, ResolverMathMLProperty, ResolverRecalcFlag, ResolverRuntimeFeature,
            ResolverStyleFlag, ResolverVisitedProperty, StyleResolver, StyleResolverBackend,
            StyleResolverBaseStyleBackend, StyleResolverDeclarationCache,
            StyleResolverRuleMatchingBackend,
        },
    },
    rule_set::RuleSet,
    ColorSchemeFlags, PreferredColorScheme,
};
use dom::{
    persistent_document::{
        CompatibilityMode, DOMInputType, DOMNamespace, DOMNodeType, DOMPartNamesMap,
        DOMShadowRootMode,
    },
    Document, UserInteractionState,
};

// cpp: layout_theme.cc:84-125 AutoAppearanceFor and the concrete
// InputTypeView::AutoAppearance overrides. Temporal controls select a
// platform/runtime view (chooser-only or multiple-fields); until that owner is
// retained, they reject the translated trial instead of guessing.
fn ProductionAutoAppearanceFor(document: &Document, element: usize) -> Result<AppearanceValue, ()> {
    let node = document.Node(element);
    if node.Namespace() != DOMNamespace::kHTML {
        return Ok(AppearanceValue::kNone);
    }
    Ok(match node.Name() {
        "button" => AppearanceValue::kButton,
        "meter" => AppearanceValue::kMeter,
        "progress" => AppearanceValue::kProgressBar,
        "textarea" => AppearanceValue::kTextArea,
        "select" => {
            if document.SelectUsesMenuList(element) {
                AppearanceValue::kMenulist
            } else {
                AppearanceValue::kListbox
            }
        }
        "input" => match document.InputType(element).ok_or(())? {
            DOMInputType::kButton | DOMInputType::kReset | DOMInputType::kSubmit => {
                AppearanceValue::kPushButton
            }
            DOMInputType::kCheckbox => AppearanceValue::kCheckbox,
            DOMInputType::kRadio => AppearanceValue::kRadio,
            DOMInputType::kColor => AppearanceValue::kSquareButton,
            DOMInputType::kRange => AppearanceValue::kSliderHorizontal,
            DOMInputType::kSearch => AppearanceValue::kSearchField,
            DOMInputType::kEmail
            | DOMInputType::kPassword
            | DOMInputType::kTelephone
            | DOMInputType::kURL
            | DOMInputType::kText
            | DOMInputType::kNumber => AppearanceValue::kTextField,
            DOMInputType::kDate
            | DOMInputType::kDateTimeLocal
            | DOMInputType::kMonth
            | DOMInputType::kTime
            | DOMInputType::kWeek => return Err(()),
            DOMInputType::kFile | DOMInputType::kHidden | DOMInputType::kImage => {
                AppearanceValue::kNone
            }
        },
        _ => AppearanceValue::kNone,
    })
}

// cpp: layout_theme.cc:194-365,485-570. Common platform-neutral theme
// adjustment for ordinary retained HTML form-control owners.
fn ApplyProductionThemeAppearance(
    document: &Document,
    element: usize,
    builder: &mut ComputedStyleBuilder,
) -> Result<(), ()> {
    use AppearanceValue::*;
    let original = builder.Appearance();
    if original == kNone {
        builder.SetEffectiveAppearance(kNone);
        return Ok(());
    }
    let auto = ProductionAutoAppearanceFor(document, element)?;
    let mut appearance = if original == auto {
        original
    } else {
        match original {
            kNone
            | kMediaSlider
            | kMediaSliderThumb
            | kMediaVolumeSlider
            | kMediaVolumeSliderThumb
            | kMediaControl => original,
            kBaseSelect | kBase => {
                if SupportsBaseAppearance(document, element, original) {
                    original
                } else {
                    auto
                }
            }
            kAuto
            | kCheckbox
            | kRadio
            | kPushButton
            | kSquareButton
            | kInnerSpinButton
            | kListbox
            | kMenulist
            | kMeter
            | kProgressBar
            | kSliderHorizontal
            | kSliderThumbHorizontal
            | kSearchField
            | kSearchFieldCancelButton
            | kTextArea => auto,
            kButton => {
                if matches!(auto, kPushButton | kSquareButton) {
                    original
                } else {
                    auto
                }
            }
            kMenulistButton => {
                if auto == kMenulist {
                    original
                } else {
                    auto
                }
            }
            kSliderVertical => {
                if auto == kSliderHorizontal {
                    original
                } else {
                    auto
                }
            }
            kSliderThumbVertical => {
                if auto == kSliderThumbHorizontal {
                    original
                } else {
                    auto
                }
            }
            kTextField => {
                if document.InputType(element) == Some(DOMInputType::kSearch) {
                    original
                } else {
                    auto
                }
            }
        }
    };
    let styled = match appearance {
        kPushButton | kSquareButton | kButton | kProgressBar | kMeter => {
            builder.HasAuthorBackground() || builder.HasAuthorBorder()
        }
        kMenulist | kSearchField | kTextArea | kTextField => {
            builder.HasAuthorBackground()
                || builder.HasAuthorBorder()
                || !builder.BoxShadow().is_null()
        }
        _ => false,
    };
    if styled {
        appearance = if appearance == kMenulist {
            kMenulistButton
        } else {
            kNone
        };
    }
    builder.SetEffectiveAppearance(appearance);

    if matches!(
        builder.Display(),
        EDisplay::kInline
            | EDisplay::kInlineTable
            | EDisplay::kTableRowGroup
            | EDisplay::kTableHeaderGroup
            | EDisplay::kTableFooterGroup
            | EDisplay::kTableRow
            | EDisplay::kTableColumnGroup
            | EDisplay::kTableColumn
            | EDisplay::kTableCell
            | EDisplay::kTableCaption
    ) {
        builder.SetDisplay(EDisplay::kInlineBlock);
    } else if matches!(builder.Display(), EDisplay::kListItem | EDisplay::kTable) {
        builder.SetDisplay(EDisplay::kBlock);
    }

    match appearance {
        kCheckbox | kRadio => {
            builder.ResetPaddingTop();
            builder.ResetPaddingRight();
            builder.ResetPaddingBottom();
            builder.ResetPaddingLeft();
            builder.ResetBorderImage();
            builder.ResetBorderTopStyle();
            builder.ResetBorderTopWidth();
            builder.ResetBorderTopColor();
            builder.ResetBorderRightStyle();
            builder.ResetBorderRightWidth();
            builder.ResetBorderRightColor();
            builder.ResetBorderBottomStyle();
            builder.ResetBorderBottomWidth();
            builder.ResetBorderBottomColor();
            builder.ResetBorderLeftStyle();
            builder.ResetBorderLeftWidth();
            builder.ResetBorderLeftColor();
            builder.ResetBorderTopLeftRadius();
            builder.ResetBorderTopRightRadius();
            builder.ResetBorderBottomLeftRadius();
            builder.ResetBorderBottomRightRadius();
            builder.SetShouldIgnoreOverflowPropertyForInlineBlockBaseline();
            builder.SetInlineBlockBaselineEdge(EInlineBlockBaselineEdge::kBorderBox);
        }
        kPushButton => builder.SetLineHeight(&ComputedStyleInitialValues::InitialLineHeight()),
        kMenulist | kMenulistButton => builder.ResetLineHeight(),
        kInnerSpinButton
        | kSliderThumbHorizontal
        | kSliderThumbVertical
        | kSearchFieldCancelButton => return Err(()),
        _ => {}
    }

    if builder.HasBaseAppearance() && SupportsBaseAppearance(document, element, original) {
        builder.SetInBaseAppearance(true);
    }
    if builder.InBaseAppearance()
        && !builder.HasBaseAppearance()
        && (SupportsBaseAppearance(document, element, kBase)
            || SupportsBaseAppearance(document, element, kBaseSelect))
    {
        builder.SetInBaseAppearance(false);
    }
    Ok(())
}

/// Typed document/platform boundary consumed by Chromium's forced-colors
/// adjustment. Platform packages can retain their actual provider privately
/// and expose only the resolved native color for one system keyword.
pub(crate) trait ProductionSystemColorResolver {
    fn ResolveSystemColor(
        &self,
        keyword: CSSValueID,
        scheme: ColorScheme,
        can_expose_accent_color: bool,
    ) -> foundation::Color;
}

#[derive(Clone, Copy, Default)]
pub(crate) struct ProductionSystemColorContext<'a> {
    pub resolver: Option<&'a dyn ProductionSystemColorResolver>,
    pub can_expose_accent_color: bool,
}

// cpp: layout_theme.cc:626-743 DefaultSystemColor. These entries are the
// platform-independent portion needed by the forced-colors UA sheet and the
// three internal forced-color initial values. Platform-dependent selection,
// accent and search colors require the typed resolver above.
fn DefaultForcedSystemColor(keyword: CSSValueID, scheme: ColorScheme) -> Option<foundation::Color> {
    use CSSValueID::*;
    let dark = scheme == ColorScheme::kDark;
    let rgba = match keyword {
        kActivetext => 0xFFFF0000,
        kButtonborder | kActiveborder | kInactiveborder | kThreeddarkshadow | kThreedhighlight
        | kThreedlightshadow | kThreedshadow | kWindowframe => {
            if dark {
                0xFF6B6B6B
            } else {
                0xFF767676
            }
        }
        kButtonface | kButtonhighlight | kButtonshadow | kThreedface => {
            if dark {
                0xFF6B6B6B
            } else {
                0xFFEFEFEF
            }
        }
        kButtontext | kCanvastext | kCaptiontext | kInfotext | kMenutext | kWindowtext
        | kFieldtext | kText => {
            if dark {
                0xFFFFFFFF
            } else {
                0xFF000000
            }
        }
        kCanvas | kActivecaption | kAppworkspace | kBackground | kInactivecaption
        | kInfobackground | kMenu | kScrollbar | kWindow => {
            if dark {
                0xFF121212
            } else {
                0xFFFFFFFF
            }
        }
        kField => {
            if dark {
                0xFF3B3B3B
            } else {
                0xFFFFFFFF
            }
        }
        kGraytext | kInactivecaptiontext => 0xFF808080,
        kLinktext => {
            if dark {
                0xFF9E9EFF
            } else {
                0xFF0000EE
            }
        }
        kMark => 0xFFFFFF00,
        kMarktext => 0xFF000000,
        kVisitedtext => {
            if dark {
                0xFFD0ADF0
            } else {
                0xFF551A8B
            }
        }
        _ => return None,
    };
    Some(foundation::Color::FromRGBA32(rgba))
}

pub(crate) fn ResolveProductionSystemColor(
    color: &StyleColor,
    scheme: ColorScheme,
    system_colors: ProductionSystemColorContext<'_>,
) -> Option<StyleColor> {
    // The initial internal forced colors may already contain a resolved
    // numeric value while retaining their system keyword. Chromium's member
    // implementation deliberately reads `color_keyword_` directly here.
    let keyword = color.GetSystemColorKeywordForResolution();
    let resolved = system_colors
        .resolver
        .map(|resolver| {
            resolver.ResolveSystemColor(keyword, scheme, system_colors.can_expose_accent_color)
        })
        .or_else(|| DefaultForcedSystemColor(keyword, scheme))?;
    Some(StyleColor::from_color_and_keyword(resolved, keyword))
}

fn ResolveForcedSystemColor(
    color: &StyleColor,
    scheme: ColorScheme,
    system_colors: ProductionSystemColorContext<'_>,
) -> Option<StyleColor> {
    ResolveProductionSystemColor(color, scheme, system_colors)
}

// cpp: style_adjuster.cc:1043-1102 AdjustForForcedColorsMode. This lives in
// the production adapter because the public StyleEngine boundary owns media
// state while the translated generic StyleAdjuster requires Blink Document,
// Page and LayoutTheme types that are intentionally private to their packages.
fn AdjustProductionStyleForForcedColors(
    builder: &mut ComputedStyleBuilder,
    media: &MediaValuesCachedData,
    system_colors: ProductionSystemColorContext<'_>,
) -> Result<(), ()> {
    if !builder.InForcedColorsMode() || builder.ForcedColorAdjust() != EForcedColorAdjust::kAuto {
        return Ok(());
    }

    builder.SetTextShadow(Member::from_ptr(
        ComputedStyleInitialValues::InitialTextShadow().unwrap_or(std::ptr::null_mut()),
    ));
    builder.SetBoxShadow(Member::from_ptr(
        ComputedStyleInitialValues::InitialBoxShadow().unwrap_or(std::ptr::null_mut()),
    ));
    let schemes: Vector<AtomicString> = vec![
        AtomicString::from_str("light"),
        AtomicString::from_str("dark"),
    ];
    builder.SetColorScheme(&schemes);
    builder.SetScrollbarColor(Member::from_ptr(
        ComputedStyleInitialValues::InitialScrollbarColor().unwrap_or(std::ptr::null_mut()),
    ));
    if builder.AccentColor().IsAutoColor()
        || builder.ShouldForceColor(builder.AccentColor().ToStyleColor())
    {
        builder.SetAccentColor(&ComputedStyleInitialValues::InitialAccentColor());
    }
    if !builder.HasUrlBackgroundImage() {
        builder.ClearBackgroundImage();
    }

    let scheme = if media.preferred_color_scheme == PreferredColorScheme::kDark {
        ColorScheme::kDark
    } else {
        ColorScheme::kLight
    };
    if builder.InternalForcedBackgroundColor().IsSystemColor() {
        let color = ResolveForcedSystemColor(
            builder.InternalForcedBackgroundColor(),
            scheme,
            system_colors,
        )
        .ok_or(())?;
        builder.SetInternalForcedBackgroundColor(&color);
    }
    // EmojiMonochromeRendering is stable at the pinned Chromium revision.
    let variant = builder.GetFontDescription().VariantEmoji();
    if matches!(
        variant,
        FontVariantEmoji::kNormalVariantEmoji | FontVariantEmoji::kUnicodeVariantEmoji
    ) {
        builder.SetFontVariantEmoji(FontVariantEmoji::kTextVariantEmoji);
    }
    if builder.InternalForcedColor().IsSystemColor() {
        let color = ResolveForcedSystemColor(builder.InternalForcedColor(), scheme, system_colors)
            .ok_or(())?;
        builder.SetInternalForcedColor(&color);
    }
    if builder.InternalForcedVisitedColor().IsSystemColor() {
        let color =
            ResolveForcedSystemColor(builder.InternalForcedVisitedColor(), scheme, system_colors)
                .ok_or(())?;
        builder.SetInternalForcedVisitedColor(&color);
    }
    Ok(())
}

/// The immutable host values required by Blink's initial-style construction.
/// It is private to Style: callers continue to interact with `StyleEngine`.
struct ProductionInitialStyleHost<'a> {
    media: &'a MediaValuesCachedData,
    color_scheme: ColorSchemeSettings,
    content_language: AtomicString,
    locale: Option<Arc<LayoutLocale>>,
    initial: Persistent<ComputedStyle>,
}

impl<'a> ProductionInitialStyleHost<'a> {
    fn new(media: &'a MediaValuesCachedData, color_scheme: ColorSchemeSettings) -> Self {
        let content_language = AtomicString::default();
        Self {
            media,
            color_scheme,
            locale: LayoutLocale::GetShared(&content_language),
            content_language,
            initial: Persistent::from_ptr(
                ComputedStyle::GetInitialStyleSingleton() as *mut ComputedStyle
            ),
        }
    }

    fn style(style: &Persistent<ComputedStyle>) -> &ComputedStyle {
        // Persistent owns the GC root for every value passed through the
        // translated resolver backend.
        unsafe { &*style.Get() }
    }
}

impl StyleResolverBackend for ProductionInitialStyleHost<'_> {
    type Document = Self;
    type ComputedStyle = Persistent<ComputedStyle>;
    type ComputedStyleBuilder = ComputedStyleBuilder;
    type MatchedPropertiesCache = ();
    type SelectorFilter = ();
    type StyleRuleUsageTracker = ();
    type StyleEngine = Self;
    type LocalFrame = Self;
    type LocalFrameView = Self;
    type Settings = ();
    type FontDescription = FontDescription;
    type LayoutLocale = Option<Arc<LayoutLocale>>;
    type StyleInitialData = ();
    type TapHighlightColor = StyleColor;
    type RuleSet = ();
    type RuleSetGroup = ();

    fn NewRuleSetGroup(&self, _rule_set_group_index: u32) -> Self::RuleSetGroup {}
    fn NewMatchedPropertiesCache(&self) -> Self::MatchedPropertiesCache {}
    fn NewSelectorFilter(&self) -> Self::SelectorFilter {}
    fn ClearMatchedPropertiesCache(&self, _cache: &mut Self::MatchedPropertiesCache) {}
    fn ClearViewportDependent(&self, _cache: &mut Self::MatchedPropertiesCache) {}

    fn GetStyleEngine<'d>(&self, document: &'d Self::Document) -> &'d Self::StyleEngine {
        document
    }
    fn GetFrame<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::LocalFrame> {
        Some(document)
    }
    fn View<'d>(&self, document: &'d Self::Document) -> Option<&'d Self::LocalFrameView> {
        Some(document)
    }
    fn GetSettings<'d>(&self, _document: &'d Self::Document) -> Option<&'d Self::Settings> {
        None
    }
    fn Printing(&self, document: &Self::Document) -> bool {
        document
            .media
            .media_type
            .Utf8()
            .eq_ignore_ascii_case("print")
    }
    fn VisuallyOrdered(&self, _document: &Self::Document) -> bool {
        false
    }
    fn InForcedColorsMode(&self, document: &Self::Document) -> bool {
        document.media.forced_colors == ForcedColors::kActive
    }
    fn InDesignMode(&self, _document: &Self::Document) -> bool {
        false
    }
    fn ContentLanguage<'d>(&self, document: &'d Self::Document) -> &'d AtomicString {
        &document.content_language
    }
    fn LayoutZoomFactor(&self, frame: &Self::LocalFrame) -> f32 {
        frame.media.layout_zoom_factor
    }
    fn MediaType<'v>(&self, view: &'v Self::LocalFrameView) -> &'v str {
        &view.media.media_type
    }
    fn UpdateActiveStyle(&self, _engine: &Self::StyleEngine) {}
    fn GetPageColorSchemes(&self, engine: &Self::StyleEngine) -> ColorSchemeFlags {
        engine.color_scheme.page_color_schemes
    }
    fn GetPreferredColorScheme(&self, engine: &Self::StyleEngine) -> PreferredColorScheme {
        engine.media.preferred_color_scheme
    }
    fn GetForceDarkModeEnabled(&self, engine: &Self::StyleEngine) -> bool {
        engine.color_scheme.force_dark
    }
    fn MaybeCreateAndGetInitialData<'e>(
        &self,
        _engine: &'e Self::StyleEngine,
    ) -> Option<&'e Self::StyleInitialData> {
        None
    }
    fn PreferDefaultScrollbarStylesEnabled(&self) -> bool {
        false
    }
    fn GetPrefersDefaultScrollbarStyles(&self, _settings: &Self::Settings) -> bool {
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
    fn TakeStyle(&self, mut builder: Self::ComputedStyleBuilder) -> Self::ComputedStyle {
        Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle)
    }
    fn ClampLineWidth(&self, value: f32) -> i32 {
        StyleBuilderConverter::ClampLineWidth(value as f64)
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
        builder.SetEffectiveZoom(zoom);
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
        let preferred = match preferred {
            PreferredColorScheme::kDark => {
                layoutng_style::style::color_scheme::mojom::blink::PreferredColorScheme::kDark
            }
            PreferredColorScheme::kLight => {
                layoutng_style::style::color_scheme::mojom::blink::PreferredColorScheme::kLight
            }
        };
        builder.SetUsedColorScheme(page_schemes, preferred, force_dark)
    }
    fn GetFontDescription(&self, builder: &Self::ComputedStyleBuilder) -> Self::FontDescription {
        builder.GetFontDescription().clone()
    }
    fn GetLayoutLocale(&self, _language: &AtomicString) -> &Self::LayoutLocale {
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
        let mut description = builder.GetFontDescription().clone();
        description.SetKeywordSize(4);
        description.SetGenericFamily(
            font_engine::fonts::font_description::GenericFamilyType::kStandardFamily,
        );
        description.SetSpecifiedSize(document.media.em_size);
        let size = document.media.em_size * self.LayoutZoomFactor(document);
        description.SetComputedSize(size);
        description.SetAdjustedSize(size);
        production_style_builder::StageFontDescription(builder, &description)
    }
    fn SetInitialData(
        &self,
        _builder: &mut Self::ComputedStyleBuilder,
        _initial_data: &Self::StyleInitialData,
    ) {
    }
    fn SetPrefersDefaultScrollbarStyles(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        value: bool,
    ) {
        builder.SetPrefersDefaultScrollbarStyles(value)
    }
}

#[derive(Clone)]
struct ProductionResolverTreeScope {
    root: usize,
    parent: Option<Box<ProductionResolverTreeScope>>,
}

#[derive(Clone)]
struct ProductionResolverShadowRoot {
    root: usize,
    mode: DOMShadowRootMode,
    scope: ProductionResolverTreeScope,
    host: Box<ProductionResolverElement>,
}

#[derive(Clone)]
struct ProductionResolverSlot {
    index: usize,
    scope: ProductionResolverTreeScope,
    assigned_slot: Option<Box<ProductionResolverSlot>>,
}

#[derive(Clone)]
struct ProductionResolverElement {
    index: usize,
    namespace: AtomicString,
    shadow_pseudo_id: AtomicString,
    media_type: foundation::String,
    tree_scope: ProductionResolverTreeScope,
    hosted_shadow_root: Option<Box<ProductionResolverShadowRoot>>,
    containing_shadow_root: Option<Box<ProductionResolverShadowRoot>>,
    owner_shadow_host: Option<Box<ProductionResolverElement>>,
    assigned_slot: Option<Box<ProductionResolverSlot>>,
    part: Option<crate::persistent_selector::PersistentPartNames>,
    part_names_map: Option<DOMPartNamesMap>,
    corresponding_element: Option<Box<ProductionResolverElement>>,
}

impl ProductionResolverTreeScope {
    fn ForNode(document: &Document, node: usize) -> Self {
        let root = document.TreeScopeRoot(node);
        let parent = document
            .ShadowHost(root)
            .map(|host| Box::new(Self::ForNode(document, host)));
        Self { root, parent }
    }
}

impl ProductionResolverSlot {
    fn ForNode(document: &Document, node: usize) -> Self {
        Self {
            index: node,
            scope: ProductionResolverTreeScope::ForNode(document, node),
            assigned_slot: document
                .AssignedSlot(node)
                .map(|slot| Box::new(Self::ForNode(document, slot))),
        }
    }
}

impl ProductionResolverShadowRoot {
    fn ForRoot(document: &Document, root: usize, media_type: &foundation::String) -> Self {
        let host = document
            .ShadowHost(root)
            .expect("retained ShadowRoot must have a host");
        Self {
            root,
            mode: document
                .ShadowRootMode(root)
                .expect("retained ShadowRoot must have a mode"),
            scope: ProductionResolverTreeScope::ForNode(document, root),
            // Do not recursively retain the same hosted root on its host.
            // All outward owner/slot/part relations remain available.
            host: Box::new(ProductionResolverElement::ForNode(
                document, host, media_type, false,
            )),
        }
    }
}

impl ProductionResolverElement {
    fn ForNode(
        document: &Document,
        index: usize,
        media_type: &foundation::String,
        include_hosted_shadow_root: bool,
    ) -> Self {
        let part = document
            .Node(index)
            .FindAttribute("part")
            .filter(|attribute| attribute.namespace_uri.is_empty())
            .map(|attribute| {
                crate::persistent_selector::PersistentPartNames::FromNames(
                    attribute
                        .value
                        .split(|character| {
                            matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ')
                        })
                        .filter(|name| !name.is_empty())
                        .map(AtomicString::from_str),
                )
            });
        Self {
            index,
            namespace: NamespaceURIFor(document.Node(index).Namespace()),
            shadow_pseudo_id: AtomicString::from_str(document.ShadowPseudoId(index).unwrap_or("")),
            media_type: media_type.clone(),
            tree_scope: ProductionResolverTreeScope::ForNode(document, index),
            hosted_shadow_root: include_hosted_shadow_root
                .then(|| document.ShadowRoot(index))
                .flatten()
                .map(|root| {
                    Box::new(ProductionResolverShadowRoot::ForRoot(
                        document, root, media_type,
                    ))
                }),
            containing_shadow_root: document.ContainingShadowRoot(index).map(|root| {
                Box::new(ProductionResolverShadowRoot::ForRoot(
                    document, root, media_type,
                ))
            }),
            owner_shadow_host: document
                .OwnerShadowHost(index)
                .map(|host| Box::new(Self::ForNode(document, host, media_type, true))),
            assigned_slot: document
                .AssignedSlot(index)
                .map(|slot| Box::new(ProductionResolverSlot::ForNode(document, slot))),
            part,
            part_names_map: document.PartNamesMap(index).cloned(),
            corresponding_element: document
                .CorrespondingSVGElement(index)
                .map(|source| Box::new(Self::ForNode(document, source, media_type, false))),
        }
    }
}

#[derive(Clone)]
enum ProductionResolverRulePass {
    Normal,
    Host,
    Slotted,
    Part(crate::persistent_selector::PersistentPartNames),
    ElementScope(Option<crate::persistent_selector::PersistentPartNames>),
}

struct ProductionResolverState {
    element: ProductionResolverElement,
    pseudo: bool,
    pseudo_id: PseudoId,
    builder: ComputedStyleBuilder,
    parent: Option<Rc<Persistent<ComputedStyle>>>,
    layout_parent: Option<Rc<Persistent<ComputedStyle>>>,
    originating_style: Option<Rc<Persistent<ComputedStyle>>>,
    output: Option<Rc<Persistent<ComputedStyle>>>,
    had_no_matched_properties: bool,
    conditionally_affects_animations: bool,
    has_tree_scoped_reference: bool,
    rejected_legacy_overlapping: bool,
    length_conversion_flags: u32,
    incremental_properties: Vec<(
        crate::css_property_name::CSSPropertyName,
        Rc<production_css_value::Value>,
    )>,
}

/// Source `StyleRequest` owner. Selector matching keeps using the compact
/// pseudo request, while ResolveBaseStyle retains explicit parent handles for
/// the complete no-cache operation.
#[derive(Clone)]
struct ProductionStyleRequest {
    pseudo: ProductionPseudoRequest,
    parent_override: Option<Persistent<ComputedStyle>>,
    layout_parent_override: Option<Persistent<ComputedStyle>>,
}

impl ProductionStyleRequest {
    fn new(pseudo: ProductionPseudoRequest) -> Self {
        Self {
            pseudo,
            parent_override: None,
            layout_parent_override: None,
        }
    }
}

/// Chromium keeps this cache on the document's long-lived StyleResolver.  The
/// production adapter creates a short-lived resolver host per element, so the
/// StyleEngine retains this shared owner and each host borrows the same cache.
#[derive(Clone, Default)]
pub(crate) struct ProductionCachedVariables {
    parent: CustomProperties,
    output: Rc<CustomProperties>,
}

#[derive(Clone)]
pub(crate) struct ProductionMatchedPropertiesCache(
    Rc<
        RefCell<
            MatchedPropertiesCache<StyleAdjusterElementTypeForCache, ProductionCachedVariables>,
        >,
    >,
);

impl Default for ProductionMatchedPropertiesCache {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(MatchedPropertiesCache::new())))
    }
}

impl ProductionMatchedPropertiesCache {
    pub(crate) fn Clear(&self) {
        self.0.borrow_mut().Clear();
    }

    pub(crate) fn Clean(&self) {
        static PROFILE_RULES: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let _trace = (*PROFILE_RULES.get_or_init(|| {
            std::env::var_os("BROWSER_PROFILE_STYLE_RULES").is_some_and(|value| value == "1")
        }))
        .then(|| browser_tracing::span("style", "StyleResolver.CleanMatchedPropertiesCache"));
        self.0.borrow_mut().CleanMatchedPropertiesCache();
    }
}

#[derive(Default)]
struct ProductionBaseStyleCascade {
    result: Rc<RefCell<MatchResult>>,
    state: Option<std::ptr::NonNull<ProductionResolverState>>,
    important_set: Option<Rc<NativeCSSBitset>>,
    animations: Option<ProductionActiveInterpolationsMap>,
    transitions: Option<ProductionActiveInterpolationsMap>,
    inline_style_lost: bool,
}

/// Typed view of the active sampled effects currently owned by the document.
/// The DOM boundary supplies sampled declarations rather than Blink's full
/// `ElementAnimations`, so lifecycle methods expose only facts represented by
/// that owner and leave update/compositor state absent.
struct ProductionElementAnimations {
    has_animations: bool,
    has_transitions: bool,
}

struct ProductionResolverCollector<'d> {
    document: &'d Document,
    element: ProductionResolverElement,
    selectors: Rc<RefCell<PersistentSelectorService>>,
    result: Rc<RefCell<MatchResult>>,
    pending_inputs: Vec<ProductionResolverRuleSetInput>,
    pending_passes: Vec<ProductionResolverRulePass>,
    scope: Option<Rc<ProductionTreeScope>>,
    matching_ua: bool,
    pseudo: ProductionPseudoRequest,
    recalc: ProductionStyleRecalcContext,
    inside_link: EInsideLink,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProductionRuleMatchingFallback {
    Unsupported(&'static str),
    Cascade(ProductionCascadeFallback),
}

/// One active stylesheet RuleSet plus the exact element that owns an implicit
/// @scope for that CSSStyleSheet client. Keeping the trigger beside the RuleSet
/// preserves ScopedStyleResolver::ImplicitScopeTrigger across the private
/// StyleEngine/StyleResolver boundary.
#[derive(Clone)]
pub(crate) struct ProductionResolverRuleSetInput {
    pub rule_set: Rc<RuleSet<Backend>>,
    pub implicit_scope_trigger: Option<usize>,
    /// Number of style rules in preceding active sheets. This is retained only
    /// for DocumentStyleEngine's matched-rule invalidation signature; cascade
    /// order remains owned by ElementRuleCollector/MatchResult.
    pub sheet_base: usize,
    pub routing: Option<Rc<crate::production_rule_matching::ProductionRuleRouting>>,
    /// TreeScope whose ScopedStyleResolver owns this CSSStyleSheet. `None` is
    /// the document scope for legacy direct callers.
    pub tree_scope_root: Option<usize>,
}

impl ProductionResolverRuleSetInput {
    pub(crate) fn plain(rule_set: Rc<RuleSet<Backend>>) -> Self {
        Self {
            rule_set,
            implicit_scope_trigger: None,
            sheet_base: 0,
            routing: None,
            tree_scope_root: None,
        }
    }
}

// cpp: ScopedStyleResolver::RebuildCascadeLayerMap / rule_set_groups_.
// A recalc batch shares only immutable rule preparation. Live selector,
// container, scope activation and match-result state stay in each collector.
#[derive(Default)]
pub(crate) struct ProductionRuleCollectionCache {
    entries: RefCell<Vec<Rc<ProductionRuleCollection>>>,
}
struct ProductionRuleCollection {
    origin: CascadeOrigin,
    inputs: Vec<ProductionResolverRuleSetInput>,
    groups: Vec<RuleSetGroup<Backend>>,
    layers: Option<Rc<CascadeLayerMap>>,
    implicit_roots: Rc<HashMap<usize, Vec<usize>>>,
}
impl ProductionRuleCollection {
    fn new(inputs: &[ProductionResolverRuleSetInput], origin: CascadeOrigin) -> Self {
        let layers = (origin == CascadeOrigin::kAuthor).then(|| {
            let active = inputs
                .iter()
                .map(|input| ActiveStyleSheet::new(Rc::new(()), Some(input.rule_set.clone())))
                .collect::<Vec<_>>();
            Rc::new(CascadeLayerMap::new(&active))
        });
        let groups = inputs
            .chunks(RuleSetGroup::<Backend>::kRulesetsRoom)
            .enumerate()
            .map(|(index, chunk)| {
                let mut group = RuleSetGroup::new(index as u32);
                for input in chunk {
                    group.AddRuleSet(input.rule_set.clone());
                }
                group
            })
            .collect();
        let implicit_roots = Rc::new(
            crate::production_rule_matching::ProductionImplicitScopeRoots(
                inputs
                    .iter()
                    .map(|input| (input.rule_set.as_ref(), input.implicit_scope_trigger)),
            ),
        );
        Self {
            origin,
            inputs: inputs.to_vec(),
            groups,
            layers,
            implicit_roots,
        }
    }
    fn Matches(&self, inputs: &[ProductionResolverRuleSetInput], origin: CascadeOrigin) -> bool {
        self.origin == origin
            && self.inputs.len() == inputs.len()
            && self.inputs.iter().zip(inputs).all(|(old, new)| {
                Rc::ptr_eq(&old.rule_set, &new.rule_set)
                    && old.implicit_scope_trigger == new.implicit_scope_trigger
                    && old.tree_scope_root == new.tree_scope_root
                    && old.sheet_base == new.sheet_base
            })
    }
}
impl ProductionRuleCollectionCache {
    fn Prepare(
        &self,
        inputs: &[ProductionResolverRuleSetInput],
        origin: CascadeOrigin,
    ) -> (Rc<ProductionRuleCollection>, bool) {
        if let Some(prepared) = self
            .entries
            .borrow()
            .iter()
            .find(|entry| entry.Matches(inputs, origin))
        {
            return (prepared.clone(), true);
        }
        let prepared = Rc::new(ProductionRuleCollection::new(inputs, origin));
        self.entries.borrow_mut().push(prepared.clone());
        (prepared, false)
    }
}

type InlineDeclarations = Rc<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>>;

/// Route element-owned inline declarations and retain immutable resolver
/// defaults. Neither owner belongs to a temporary per-element matching host;
/// inline values live with DOM ElementData, not a resolver source-text cache.
pub(crate) struct ProductionDeclarationCache {
    defaults: StyleResolverDeclarationCache<
        Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>,
    >,
}

impl Default for ProductionDeclarationCache {
    fn default() -> Self {
        Self {
            defaults: StyleResolverDeclarationCache::new(),
        }
    }
}

impl ProductionDeclarationCache {
    fn Resolve(
        &self,
        document: &Document,
        index: usize,
        mode: crate::parser::css_parser_mode::CSSParserMode,
    ) -> InlineDeclarations {
        crate::production_style_sheet_projection::ResolveInlineStyle(
            &document.Node(index).InlineStyle(),
            mode,
        )
        .Declarations()
    }
}

/// The document-owned collaborators consumed when the translated resolver
/// reaches `StyleCascade::Apply`. This is deliberately the same ownership
/// boundary as [`ProductionCascadeInputs`]: the resolver gathers rules and
/// initializes state, while the existing cascade adapter remains the sole
/// owner of substitution, registered-property, image and interpolation
/// semantics.
pub(crate) struct ProductionIncrementalStyle<'a> {
    pub custom_properties: Rc<CustomProperties>,
    pub matched_signatures: &'a [(usize, u32, u32)],
}

pub(crate) struct ProductionBaseStyleCollaborators<'a> {
    /// Supplemental quirks-mode HTML UA owner. It remains parser-owned and is
    /// selected by StyleResolver::ForEachUARulesForElement through the same
    /// package-private collaborator bundle as the rest of the resolver state.
    pub html_quirks_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    pub forced_colors_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    pub fullscreen_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    pub root_style: Option<Persistent<ComputedStyle>>,
    pub root_font_size: f32,
    pub color_scheme_settings: ColorSchemeSettings,
    /// Borrowed native system-color owner used only while resolving this
    /// style. `None` selects Chromium's canonical LayoutTheme fallback.
    pub system_colors: ProductionSystemColorContext<'a>,
    /// Document::TextLinkColors custom text color. StyleResolver publishes
    /// the body's resolved current color here; UA quirks colors consume it.
    pub text_link_text_color: Option<foundation::Color>,
    pub is_document_element: bool,
    pub is_ensuring_style: bool,
    pub incremental_style: Option<ProductionIncrementalStyle<'a>>,
    pub pseudo_id: PseudoId,
    pub pseudo_argument: Option<AtomicString>,
    pub search_text: SearchTextRequest,
    pub originating_style: Option<Persistent<ComputedStyle>>,
    pub subject_node: Option<usize>,
    pub container_runtime: Option<&'a ProductionContainerRuntime<'a>>,
    pub custom_properties: Option<&'a CustomProperties>,
    pub parent_custom_properties: Option<&'a CustomProperties>,
    pub substitution_context: SubstitutionContext<'a>,
    pub image_resolver: Option<&'a dyn URLImageResolver>,
    pub property_registry: Option<&'a PropertyRegistry>,
    pub function_registry: Option<&'a crate::production_style_sheet::ProductionFunctionRegistry>,
    /// Document font service used by CSSLengthResolver for ch/ex/cap/ic and
    /// their root-relative forms. Chromium carries this through
    /// StyleResolverState's conversion data; keep the same owner on the
    /// translated resolver path instead of falling back to the compatibility
    /// cascade for font-relative lengths.
    pub font_metrics: Option<&'a dyn production_style_builder::FontMetricsResolver>,
    pub animations: Option<&'a ProductionActiveInterpolationsMap>,
    pub transitions: Option<&'a ProductionActiveInterpolationsMap>,
    pub matched_properties_cache: Option<ProductionMatchedPropertiesCache>,
    pub rule_collections: Option<Rc<ProductionRuleCollectionCache>>,
    /// Resolver-owned immutable RuleSet sentinels and the UA RuleSetGroup
    /// cache shared by every element in one style-update traversal.
    ///
    /// cpp: StyleResolver owns these for its document lifetime. Keeping this
    /// owner outside the per-element host preserves that reuse without
    /// exposing resolver internals across the Style package boundary.
    pub resolver_batch: Option<Rc<ProductionResolverBatch>>,
    pub declarations: Option<Rc<ProductionDeclarationCache>>,
    /// Document-owned selector handles and compiled-query cache. Each match
    /// still creates a fresh session/backend with live DOM and scope state.
    /// cpp: StyleEngine retains its StyleResolver; per-element collectors
    /// reference the document's existing node owners rather than rebuilding
    /// handles for every node in every ResolveStyle call.
    pub selectors: Option<Rc<RefCell<PersistentSelectorService>>>,
    /// Document-owned live interaction state consumed by selector matching.
    /// cpp: SelectorChecker::MatchInternalPseudoClass delegates
    /// `:-internal-multi-select-focus` to HTMLOptionElement's retained state;
    /// Rechrom retains the equivalent active option in the document's
    /// interaction owner and publishes one snapshot per style update.
    pub interaction: UserInteractionState,
}

impl ProductionBaseStyleCollaborators<'_> {
    pub(crate) fn ordinary(root_font_size: f32) -> Self {
        Self {
            html_quirks_user_agent_rules: None,
            forced_colors_user_agent_rules: None,
            fullscreen_user_agent_rules: None,
            root_style: None,
            root_font_size,
            color_scheme_settings: ColorSchemeSettings::default(),
            system_colors: ProductionSystemColorContext::default(),
            text_link_text_color: None,
            is_document_element: false,
            is_ensuring_style: false,
            incremental_style: None,
            pseudo_id: PseudoId::kPseudoIdNone,
            pseudo_argument: None,
            search_text: SearchTextRequest::kNone,
            originating_style: None,
            subject_node: None,
            container_runtime: None,
            custom_properties: None,
            parent_custom_properties: None,
            substitution_context: SubstitutionContext::default(),
            image_resolver: None,
            property_registry: None,
            function_registry: None,
            font_metrics: None,
            animations: None,
            transitions: None,
            matched_properties_cache: None,
            rule_collections: None,
            resolver_batch: None,
            declarations: None,
            selectors: None,
            interaction: UserInteractionState::default(),
        }
    }
}

/// The part of Chromium's document-owned StyleResolver that is invariant
/// while walking a style-update batch. Per-element state remains in
/// `ProductionRuleMatchingHost`; UA RuleSetGroup construction and empty
/// fallback owners are retained once for the whole traversal.
pub(crate) struct ProductionResolverBatch {
    empty_rules: Rc<RuleSet<Backend>>,
    media_controls_rules: Rc<RuleSet<Backend>>,
    forced_colors_media_controls_rules: Rc<RuleSet<Backend>>,
    ua_group_cache: Rc<RefCell<Vec<(u32, RuleSetGroup<Backend>)>>>,
    mathml_namespace: AtomicString,
}

impl Default for ProductionResolverBatch {
    fn default() -> Self {
        Self {
            empty_rules: Rc::new(RuleSet::new()),
            media_controls_rules: Rc::new(RuleSet::new()),
            forced_colors_media_controls_rules: Rc::new(RuleSet::new()),
            ua_group_cache: Rc::new(RefCell::new(Vec::new())),
            mathml_namespace: AtomicString::from_str("http://www.w3.org/1998/Math/MathML"),
        }
    }
}

/// Document-owned adapter for the first production slice of translated
/// StyleResolver rule matching. The rule sets and selector service remain the
/// existing parser/DOM owners; this host only binds them to required callbacks.
struct ProductionRuleMatchingHost<'d> {
    initial: ProductionInitialStyleHost<'d>,
    document: &'d Document,
    author_rule_sets: Rc<Vec<ProductionResolverRuleSetInput>>,
    selectors: Rc<RefCell<PersistentSelectorService>>,
    html_user_agent_rules: ProductionResolverRuleSetInput,
    svg_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    mathml_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    html_quirks_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    fullscreen_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    empty_rules: Rc<RuleSet<Backend>>,
    /// cpp: css_default_style_sheets.cc:288-293,347-363.
    ///
    /// CSSDefaultStyleSheets owns distinct media-control RuleSets even before
    /// the modules layer installs its UAStyleSheetLoader. Rechrom does not yet
    /// retain that modules resource owner, so these start empty while still
    /// preserving StyleResolver::MatchUARules' normal/forced-colors selection
    /// and separate UA-origin transfer. Empty here describes only the current
    /// contents of these independent sets; it does not stand in for the
    /// missing media-controls resource loader or internal control DOM.
    media_controls_rules: Rc<RuleSet<Backend>>,
    forced_colors_media_controls_rules: Rc<RuleSet<Backend>>,
    /// Supplemental forced-colors UA rules, built with Chromium's dedicated
    /// `ua-forced-colors` media-type evaluator.
    forced_color_rules: Option<ProductionResolverRuleSetInput>,
    declarations: Rc<ProductionDeclarationCache>,
    ua_group_cache: Rc<RefCell<Vec<(u32, RuleSetGroup<Backend>)>>>,
    mathml_namespace: AtomicString,
    cascade: ProductionBaseStyleCollaborators<'d>,
    cascade_custom_output: RefCell<Option<Rc<CustomProperties>>>,
    base_style_metrics: RefCell<ProductionBaseStyleMetrics>,
    matched_signatures: RefCell<Vec<(usize, u32, u32)>>,
    fallbacks: RefCell<Vec<ProductionRuleMatchingFallback>>,
    matched_properties_cache: ProductionMatchedPropertiesCache,
    // ComputedStyleBuilder stores StyleBaseData as a raw Oilpan Member. Keep
    // the production GC root alive until StateTakeStyle transfers that Member
    // into the final rooted ComputedStyle.
    base_data_roots: RefCell<Vec<Persistent<NativeStyleBaseData>>>,
}

impl<'d> ProductionRuleMatchingHost<'d> {
    fn new(
        document: &'d Document,
        media: &'d MediaValuesCachedData,
        html_user_agent_rules: ProductionResolverRuleSetInput,
        author_rule_sets: Vec<ProductionResolverRuleSetInput>,
    ) -> Self {
        let collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        Self::new_with_collaborators(
            document,
            media,
            html_user_agent_rules,
            author_rule_sets,
            collaborators,
        )
    }

    fn new_with_collaborators(
        document: &'d Document,
        media: &'d MediaValuesCachedData,
        html_user_agent_rules: ProductionResolverRuleSetInput,
        author_rule_sets: Vec<ProductionResolverRuleSetInput>,
        cascade: ProductionBaseStyleCollaborators<'d>,
    ) -> Self {
        Self::new_with_user_agent_rule_sets(
            document,
            media,
            html_user_agent_rules,
            None,
            None,
            Rc::new(author_rule_sets),
            cascade,
        )
    }

    fn new_with_user_agent_rule_sets(
        document: &'d Document,
        media: &'d MediaValuesCachedData,
        html_user_agent_rules: ProductionResolverRuleSetInput,
        svg_user_agent_rules: Option<ProductionResolverRuleSetInput>,
        mathml_user_agent_rules: Option<ProductionResolverRuleSetInput>,
        author_rule_sets: Rc<Vec<ProductionResolverRuleSetInput>>,
        cascade: ProductionBaseStyleCollaborators<'d>,
    ) -> Self {
        let matched_properties_cache = cascade.matched_properties_cache.clone().unwrap_or_default();
        let selectors = cascade.selectors.clone().unwrap_or_default();
        selectors.borrow_mut().interaction = cascade.interaction;
        let resolver_batch = cascade.resolver_batch.clone().unwrap_or_default();
        Self {
            initial: ProductionInitialStyleHost::new(media, cascade.color_scheme_settings),
            document,
            author_rule_sets,
            selectors,
            html_user_agent_rules,
            svg_user_agent_rules,
            mathml_user_agent_rules,
            html_quirks_user_agent_rules: cascade.html_quirks_user_agent_rules.clone(),
            fullscreen_user_agent_rules: cascade.fullscreen_user_agent_rules.clone(),
            empty_rules: resolver_batch.empty_rules.clone(),
            media_controls_rules: resolver_batch.media_controls_rules.clone(),
            forced_colors_media_controls_rules: resolver_batch
                .forced_colors_media_controls_rules
                .clone(),
            forced_color_rules: cascade.forced_colors_user_agent_rules.clone(),
            declarations: cascade.declarations.clone().unwrap_or_default(),
            ua_group_cache: resolver_batch.ua_group_cache.clone(),
            mathml_namespace: resolver_batch.mathml_namespace.clone(),
            cascade,
            cascade_custom_output: RefCell::new(None),
            base_style_metrics: RefCell::new(ProductionBaseStyleMetrics::default()),
            matched_signatures: RefCell::new(Vec::new()),
            fallbacks: RefCell::new(Vec::new()),
            matched_properties_cache,
            base_data_roots: RefCell::new(Vec::new()),
        }
    }

    fn unsupported(&self, operation: &'static str) {
        let value = ProductionRuleMatchingFallback::Unsupported(operation);
        let mut fallbacks = self.fallbacks.borrow_mut();
        if !fallbacks.contains(&value) {
            fallbacks.push(value);
        }
    }

    /// Element::AddPropertyToPresentationAttributeStyle(String).
    ///
    /// Presentation attributes are parsed as the CSS longhand they synthesize
    /// and retained in the same mutable property-set owner that
    /// StyleResolver::MatchPresentationalHints transfers at author-hint
    /// origin. Invalid attribute values are ignored by the source parser.
    fn AddPresentationAttributeProperty(
        &self,
        properties: &Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>,
        property: CSSPropertyID,
        value: &str,
    ) {
        let Ok(parsed) = crate::parser::production_property_parser::ParseProperty(
            property,
            &foundation::String::from(value),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        ) else {
            return;
        };
        let CSSPropertyValueSetRuleHandle::Mutable(properties) = properties.as_ref() else {
            self.unsupported("base:immutable-presentation-attribute-style");
            return;
        };
        let mut properties = properties.borrow_mut();
        for parsed_property in parsed {
            properties.SetLonghandProperty(parsed_property);
        }
    }

    /// MathMLElement::CollectStyleForPresentationAttribute(scriptlevel).
    ///
    /// Blink first validates the unsigned payload and then lets the CSS
    /// math-depth parser consume either the absolute integer or add(integer).
    fn MathMLScriptLevelPropertyValue(value: &str) -> Option<String> {
        let bytes = value.as_bytes();
        if bytes
            .first()
            .is_some_and(|byte| *byte == b'+' || *byte == b'-')
        {
            // Preserve the fixed source's validation slice: after detecting a
            // sign it validates the final code unit before constructing add()
            // from the original attribute text. The property parser remains
            // the authority for the complete signed integer.
            let last = value.chars().next_back()?;
            last.to_digit(10)?;
            Some(format!("add({value})"))
        } else {
            value.parse::<u32>().ok()?;
            Some(value.to_owned())
        }
    }

    /// MathMLElement::ParseMathLength.
    ///
    /// The source returns the parsed CSSPrimitiveValue to both presentation
    /// hint collection and custom-style conversion. Keeping that primitive as
    /// the shared owner avoids reparsing the element-specific attributes with a
    /// broader width/height grammar.
    fn MathMLAttributePrimitiveValue(
        &self,
        element: &ProductionResolverElement,
        attribute_name: &str,
        allow_percentages: bool,
        nonnegative: bool,
    ) -> Option<Rc<production_css_value::Value>> {
        let attribute = self
            .document
            .Node(element.index)
            .FindAttribute(attribute_name)?;
        let parsed = crate::parser::production_property_parser::ParseProperty(
            CSSPropertyID::kTop,
            &foundation::String::from(attribute.value.as_str()),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        )
        .ok()?;
        let [property] = parsed.as_slice() else {
            return None;
        };
        let CSSValuePayload::kNumericLiteralClass(value) = property.Value().Payload() else {
            // ParseMathLength rejects CSSMathFunctionValue after parsing.
            return None;
        };
        if nonnegative && value.DoubleValue() < 0.0 {
            return None;
        }
        if value.GetType() == UnitType::kPercentage && !allow_percentages {
            return None;
        }
        Some(property.ValueRef())
    }

    fn SetMathMLLengthPresentationAttribute(
        &self,
        properties: &Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>,
        property: CSSPropertyID,
        value: Rc<production_css_value::Value>,
    ) {
        let CSSPropertyValueSetRuleHandle::Mutable(properties) = properties.as_ref() else {
            self.unsupported("base:immutable-presentation-attribute-style");
            return;
        };
        properties
            .borrow_mut()
            .SetLonghandPropertyValue(property, value);
    }

    /// MathMLElement::ParseMathLength / AddMathLengthToComputedStyle.
    ///
    /// Blink parses these attributes with the strict CSS length-percentage
    /// grammar, rejects calculated values, then converts the retained primitive
    /// with the resolver state's CSSToLengthConversionData. Reuse the production
    /// longhand parser for the same token grammar and the existing typed length
    /// resolver for font/root/viewport/zoom conversion. Invalid attributes are
    /// ignored; a valid unit whose conversion owner is not available rejects the
    /// whole translated trial through `unsupported`.
    fn MathMLAttributeLength(
        &self,
        element: &ProductionResolverElement,
        attribute_name: &str,
        allow_percentages: bool,
        nonnegative: bool,
        state: &mut ProductionResolverState,
    ) -> Option<Length> {
        let primitive = self.MathMLAttributePrimitiveValue(
            element,
            attribute_name,
            allow_percentages,
            nonnegative,
        )?;
        let CSSValuePayload::kNumericLiteralClass(value) = primitive.Payload() else {
            unreachable!("MathMLAttributePrimitiveValue retains only numeric literals")
        };
        let number = value.DoubleValue();
        let unit = value.GetType();
        if unit == UnitType::kPercentage {
            return Some(Length::Percent(number));
        }

        let builder = &mut state.builder;
        let mut resolver = production_style_builder::MathLengthResolver(
            CSSPropertyID::kInvalid,
            builder.GetFontDescription().ComputedSize(),
            self.cascade.root_font_size,
            builder.EffectiveZoom(),
            self.initial.media,
            Some(production_style_builder::CurrentComputedLineHeight(builder)),
            self.cascade.root_style.as_ref().map(|style| {
                let style = unsafe { &*style.Get() };
                style.ComputedLineHeight() / style.EffectiveZoom()
            }),
        );
        let pixels = match crate::css_math_expression_node::CSSMathLengthResolver::ComputeLength(
            &mut resolver,
            number,
            unit,
        ) {
            Ok(value) => value,
            Err(_) => {
                self.unsupported("base:mathml-length-conversion");
                return None;
            }
        };
        match unit {
            UnitType::kEms | UnitType::kQuirkyEms => builder.SetHasEmUnits(),
            UnitType::kRems => builder.SetHasRootRelativeUnits(),
            UnitType::kViewportWidth
            | UnitType::kViewportHeight
            | UnitType::kViewportMin
            | UnitType::kViewportMax
            | UnitType::kSmallViewportWidth
            | UnitType::kSmallViewportHeight
            | UnitType::kLargeViewportWidth
            | UnitType::kLargeViewportHeight => builder.SetHasStaticViewportUnits(),
            UnitType::kDynamicViewportWidth | UnitType::kDynamicViewportHeight => {
                builder.SetHasDynamicViewportUnits()
            }
            _ => {}
        }
        Some(Length::Fixed(
            crate::css_value_clamping_utils::CSSValueClampingUtils::ClampLength(pixels),
        ))
    }

    fn take_fallbacks(&self) -> Vec<ProductionRuleMatchingFallback> {
        std::mem::take(&mut *self.fallbacks.borrow_mut())
    }

    fn style(style: &Persistent<ComputedStyle>) -> &ComputedStyle {
        unsafe { &*style.Get() }
    }
}

impl StyleResolverBackend for ProductionRuleMatchingHost<'_> {
    type Document = ProductionResolverElement;
    type ComputedStyle = Persistent<ComputedStyle>;
    type ComputedStyleBuilder = ComputedStyleBuilder;
    type MatchedPropertiesCache = ProductionMatchedPropertiesCache;
    type SelectorFilter = ();
    type StyleRuleUsageTracker = ();
    type StyleEngine = ProductionResolverElement;
    type LocalFrame = ProductionResolverElement;
    type LocalFrameView = ProductionResolverElement;
    type Settings = ();
    type FontDescription = FontDescription;
    type LayoutLocale = Option<Arc<LayoutLocale>>;
    type StyleInitialData = ();
    type TapHighlightColor = StyleColor;
    type RuleSet = RuleSet<Backend>;
    type RuleSetGroup = RuleSetGroup<Backend>;

    fn NewRuleSetGroup(&self, index: u32) -> Self::RuleSetGroup {
        RuleSetGroup::new(index)
    }
    fn NewMatchedPropertiesCache(&self) -> Self::MatchedPropertiesCache {
        self.matched_properties_cache.clone()
    }
    fn NewSelectorFilter(&self) -> Self::SelectorFilter {}
    fn ClearMatchedPropertiesCache(&self, cache: &mut Self::MatchedPropertiesCache) {
        cache.Clear()
    }
    fn ClearViewportDependent(&self, cache: &mut Self::MatchedPropertiesCache) {
        cache.0.borrow_mut().ClearViewportDependent()
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
    fn GetSettings<'d>(&self, _: &'d Self::Document) -> Option<&'d Self::Settings> {
        None
    }
    fn Printing(&self, _: &Self::Document) -> bool {
        self.initial
            .media
            .media_type
            .Utf8()
            .eq_ignore_ascii_case("print")
    }
    fn VisuallyOrdered(&self, _: &Self::Document) -> bool {
        false
    }
    fn InForcedColorsMode(&self, _: &Self::Document) -> bool {
        self.initial.media.forced_colors == ForcedColors::kActive
    }
    fn InDesignMode(&self, _: &Self::Document) -> bool {
        false
    }
    fn ContentLanguage<'d>(&self, _: &'d Self::Document) -> &'d AtomicString {
        &foundation::atomic_string::g_null_atom
    }
    fn LayoutZoomFactor(&self, _: &Self::LocalFrame) -> f32 {
        self.initial.media.layout_zoom_factor
    }
    fn MediaType<'v>(&self, view: &'v Self::LocalFrameView) -> &'v str {
        &view.media_type
    }
    fn UpdateActiveStyle(&self, _: &Self::StyleEngine) {}
    fn GetPageColorSchemes(&self, _: &Self::StyleEngine) -> ColorSchemeFlags {
        self.initial.color_scheme.page_color_schemes
    }
    fn GetPreferredColorScheme(&self, _: &Self::StyleEngine) -> PreferredColorScheme {
        self.initial.media.preferred_color_scheme
    }
    fn GetForceDarkModeEnabled(&self, _: &Self::StyleEngine) -> bool {
        self.initial.color_scheme.force_dark
    }
    fn MaybeCreateAndGetInitialData<'e>(
        &self,
        _: &'e Self::StyleEngine,
    ) -> Option<&'e Self::StyleInitialData> {
        None
    }
    fn PreferDefaultScrollbarStylesEnabled(&self) -> bool {
        false
    }
    fn GetPrefersDefaultScrollbarStyles(&self, _: &Self::Settings) -> bool {
        false
    }
    fn GetInitialStyleSingleton(&self) -> &Self::ComputedStyle {
        &self.initial.initial
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
    fn TakeStyle(&self, mut builder: Self::ComputedStyleBuilder) -> Self::ComputedStyle {
        Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle)
    }
    fn ClampLineWidth(&self, value: f32) -> i32 {
        StyleBuilderConverter::ClampLineWidth(value as f64)
    }
    fn SetBorderTopWidth(&self, b: &mut Self::ComputedStyleBuilder, v: i32) {
        b.SetBorderTopWidthOwned(v)
    }
    fn SetBorderRightWidth(&self, b: &mut Self::ComputedStyleBuilder, v: i32) {
        b.SetBorderRightWidthOwned(v)
    }
    fn SetBorderBottomWidth(&self, b: &mut Self::ComputedStyleBuilder, v: i32) {
        b.SetBorderBottomWidthOwned(v)
    }
    fn SetBorderLeftWidth(&self, b: &mut Self::ComputedStyleBuilder, v: i32) {
        b.SetBorderLeftWidthOwned(v)
    }
    fn SetOutlineWidth(&self, b: &mut Self::ComputedStyleBuilder, v: i32) {
        b.SetOutlineWidthOwned(v)
    }
    fn SetColumnRuleWidthInternal(&self, b: &mut Self::ComputedStyleBuilder, v: GapDataList<i32>) {
        b.SetColumnRuleWidthInternal(&v)
    }
    fn SetRowRuleWidthInternal(&self, b: &mut Self::ComputedStyleBuilder, v: GapDataList<i32>) {
        b.SetRowRuleWidth(&v)
    }
    fn SetRtlOrdering(&self, b: &mut Self::ComputedStyleBuilder, v: EOrder) {
        b.SetRtlOrdering(v)
    }
    fn SetZoom(&self, b: &mut Self::ComputedStyleBuilder, v: f32) {
        b.SetZoom(v)
    }
    fn SetEffectiveZoom(&self, b: &mut Self::ComputedStyleBuilder, v: f32) {
        b.SetEffectiveZoom(v);
    }
    fn SetInForcedColorsMode(&self, b: &mut Self::ComputedStyleBuilder, v: bool) {
        b.SetInForcedColorsMode(v)
    }
    fn InitialTapHighlightColor(&self) -> Self::TapHighlightColor {
        ComputedStyleInitialValues::InitialTapHighlightColor()
    }
    fn SetTapHighlightColor(&self, b: &mut Self::ComputedStyleBuilder, v: Self::TapHighlightColor) {
        b.SetTapHighlightColorOwned(v)
    }
    fn SetUsedColorScheme(
        &self,
        b: &mut Self::ComputedStyleBuilder,
        schemes: ColorSchemeFlags,
        preferred: PreferredColorScheme,
        force_dark: bool,
    ) {
        let preferred = match preferred {
            PreferredColorScheme::kDark => {
                layoutng_style::style::color_scheme::mojom::blink::PreferredColorScheme::kDark
            }
            PreferredColorScheme::kLight => {
                layoutng_style::style::color_scheme::mojom::blink::PreferredColorScheme::kLight
            }
        };
        b.SetUsedColorScheme(schemes, preferred, force_dark)
    }
    fn GetFontDescription(&self, b: &Self::ComputedStyleBuilder) -> Self::FontDescription {
        b.GetFontDescription().clone()
    }
    fn GetLayoutLocale(&self, _: &AtomicString) -> &Self::LayoutLocale {
        &self.initial.locale
    }
    fn SetLocale(&self, d: &mut Self::FontDescription, l: &Self::LayoutLocale) {
        d.SetLocale(l.clone())
    }
    fn SetFontDescription(&self, b: &mut Self::ComputedStyleBuilder, d: Self::FontDescription) {
        production_style_builder::StageFontDescription(b, &d)
    }
    fn SetUserModify(&self, b: &mut Self::ComputedStyleBuilder, v: EUserModify) {
        b.SetUserModify(v)
    }
    fn CreateInitialFont(&self, _: &Self::Document, b: &mut Self::ComputedStyleBuilder) {
        let mut d = b.GetFontDescription().clone();
        d.SetKeywordSize(4);
        d.SetGenericFamily(
            font_engine::fonts::font_description::GenericFamilyType::kStandardFamily,
        );
        d.SetSpecifiedSize(self.initial.media.em_size);
        d.SetComputedSize(self.initial.media.em_size);
        d.SetAdjustedSize(self.initial.media.em_size);
        production_style_builder::StageFontDescription(b, &d)
    }
    fn SetInitialData(&self, _: &mut Self::ComputedStyleBuilder, _: &Self::StyleInitialData) {}
    fn SetPrefersDefaultScrollbarStyles(&self, b: &mut Self::ComputedStyleBuilder, v: bool) {
        b.SetPrefersDefaultScrollbarStyles(v)
    }
}

impl<'d> StyleResolverRuleMatchingBackend for ProductionRuleMatchingHost<'d> {
    type Element = ProductionResolverElement;
    type TreeScope = ProductionResolverTreeScope;
    type ShadowRoot = ProductionResolverShadowRoot;
    type HTMLSlotElement = ProductionResolverSlot;
    type ScopedStyleResolver = ProductionResolverTreeScope;
    type ElementRuleCollector = ProductionResolverCollector<'d>;
    type StyleResolverState = ProductionResolverState;
    type CSSStyleSheet = ();
    type TextTrack = ();
    type DOMTokenList = crate::persistent_selector::PersistentPartNames;
    type TokenSet = crate::persistent_selector::PersistentPartNames;
    type PartNames = crate::persistent_selector::PersistentPartNames;
    type PartNamesMap = DOMPartNamesMap;
    type RootNode = ProductionResolverTreeScope;
    type CSSPropertyValueSet = Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>;
    type ScopedRuleTreeScope = ();

    fn PseudoElementUltimateOriginatingElement<'e>(
        &self,
        e: &'e Self::Element,
    ) -> &'e Self::Element {
        e
    }
    fn ElementTreeScope<'e>(&self, element: &'e Self::Element) -> &'e Self::TreeScope {
        &element.tree_scope
    }
    fn ParentTreeScope<'t>(&self, scope: &'t Self::TreeScope) -> Option<&'t Self::TreeScope> {
        scope.parent.as_deref()
    }
    fn TreeScopeResolver<'t>(
        &self,
        scope: &'t Self::TreeScope,
    ) -> Option<&'t Self::ScopedStyleResolver> {
        (scope.root == self.document.Root()
            || self
                .author_rule_sets
                .iter()
                .any(|input| input.tree_scope_root.unwrap_or(self.document.Root()) == scope.root))
        .then_some(scope)
    }
    fn ResolverTreeScope<'r>(
        &self,
        resolver: &'r Self::ScopedStyleResolver,
    ) -> &'r Self::TreeScope {
        resolver
    }
    fn TreeScopeRoot<'t>(&self, scope: &'t Self::TreeScope) -> &'t Self::RootNode {
        scope
    }
    fn IsHTMLElement(&self, e: &Self::Element) -> bool {
        self.document.Node(e.index).Namespace() == DOMNamespace::kHTML
    }
    fn IsSVGElement(&self, e: &Self::Element) -> bool {
        self.document.Node(e.index).Namespace() == DOMNamespace::kSVG
    }
    fn IsVTTElement(&self, _: &Self::Element) -> bool {
        false
    }
    fn IsVTTCueBackgroundBox(&self, _: &Self::Element) -> bool {
        false
    }
    fn IsPseudoElement(&self, _: &Self::Element) -> bool {
        false
    }
    fn IsStyledElement(&self, e: &Self::Element) -> bool {
        self.document.Node(e.index).Type() == DOMNodeType::kElement
    }
    fn IsInShadowTree(&self, element: &Self::Element) -> bool {
        element.containing_shadow_root.is_some()
    }
    fn IsMediaElement(&self, element: &Self::Element) -> bool {
        self.document.IsMediaElement(element.index)
    }
    fn NamespaceURI<'e>(&self, e: &'e Self::Element) -> &'e AtomicString {
        &e.namespace
    }
    fn MathMLNamespaceURI(&self) -> &AtomicString {
        &self.mathml_namespace
    }
    fn CorrespondingElement<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::Element> {
        element.corresponding_element.as_deref()
    }
    fn ShadowPseudoId<'e>(&self, e: &'e Self::Element) -> &'e AtomicString {
        &e.shadow_pseudo_id
    }
    fn ElementPseudoIdForStyling(
        &self,
        _: &Self::Element,
    ) -> layoutng_style::style::computed_style_constants::PseudoId {
        layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdNone
    }
    fn PseudoIdForShadowElementName(
        &self,
        name: &AtomicString,
    ) -> layoutng_style::style::computed_style_constants::PseudoId {
        // cpp: html/shadow/shadow_element_utils.cc:71-95. Keep this mapping
        // limited to the six UA-shadow names Chromium promotes to PseudoId;
        // media-control names deliberately remain kPseudoIdNone.
        match name.Utf8().as_str() {
            "-webkit-input-placeholder" => PseudoId::kPseudoIdPlaceholder,
            "-webkit-file-upload-button" => PseudoId::kPseudoIdFileSelectorButton,
            "details-content" => PseudoId::kPseudoIdDetailsContent,
            "permission-icon" => PseudoId::kPseudoIdPermissionIcon,
            "picker(select)" => PseudoId::kPseudoIdPickerSelect,
            "select-listbox" => PseudoId::kPseudoIdSelectListbox,
            _ => PseudoId::kPseudoIdNone,
        }
    }
    fn IsTransitionPseudoElement(
        &self,
        _: layoutng_style::style::computed_style_constants::PseudoId,
    ) -> bool {
        false
    }
    fn GetShadowRoot<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::ShadowRoot> {
        element.hosted_shadow_root.as_deref()
    }
    fn ContainingShadowRoot<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::ShadowRoot> {
        element.containing_shadow_root.as_deref()
    }
    fn ShadowRootResolver<'s>(
        &self,
        root: &'s Self::ShadowRoot,
    ) -> Option<&'s Self::ScopedStyleResolver> {
        self.author_rule_sets
            .iter()
            .any(|input| input.tree_scope_root.unwrap_or(self.document.Root()) == root.scope.root)
            .then_some(&root.scope)
    }
    fn ShadowRootIsUserAgent(&self, root: &Self::ShadowRoot) -> bool {
        root.mode == DOMShadowRootMode::kUserAgent
    }
    fn ShadowRootHost<'s>(&self, root: &'s Self::ShadowRoot) -> &'s Self::Element {
        &root.host
    }
    fn OwnerShadowHost<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::Element> {
        element.owner_shadow_host.as_deref()
    }
    fn AssignedSlot<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::HTMLSlotElement> {
        element.assigned_slot.as_deref()
    }
    fn SlotAssignedSlot<'s>(
        &self,
        slot: &'s Self::HTMLSlotElement,
    ) -> Option<&'s Self::HTMLSlotElement> {
        slot.assigned_slot.as_deref()
    }
    fn SlotTreeScope<'s>(&self, slot: &'s Self::HTMLSlotElement) -> &'s Self::TreeScope {
        &slot.scope
    }
    fn VTTElementTrack<'e>(&self, _: &'e Self::Element) -> Option<&'e Self::TextTrack> {
        None
    }
    fn VTTCueBackgroundBoxTrack<'e>(&self, _: &'e Self::Element) -> Option<&'e Self::TextTrack> {
        None
    }
    fn TextTrackCSSStyleSheets<'t>(&self, _: &'t Self::TextTrack) -> &'t [Rc<Self::CSSStyleSheet>] {
        &[]
    }
    fn ElementDocument<'e>(&self, element: &'e Self::Element) -> &'e Self::Document {
        element
    }
    fn EnsureVTTOriginatingElement<'s>(&self, engine: &'s Self::StyleEngine) -> &'s Self::Element {
        self.unsupported("vtt-originating-element");
        engine
    }
    fn RuleSetForSheet(
        &self,
        _: &Self::StyleEngine,
        _: &Self::CSSStyleSheet,
    ) -> Option<Rc<Self::RuleSet>> {
        self.unsupported("vtt-rule-set");
        None
    }
    fn GetPart<'e>(&self, element: &'e Self::Element) -> Option<&'e Self::DOMTokenList> {
        element.part.as_ref()
    }
    fn PartLength(&self, part: &Self::DOMTokenList) -> usize {
        part.Len()
    }
    fn PartTokenSet<'p>(&self, part: &'p Self::DOMTokenList) -> &'p Self::TokenSet {
        part
    }
    fn NewPartNames(&self, tokens: &Self::TokenSet) -> Self::PartNames {
        tokens.clone()
    }
    fn HasPartNamesMap(&self, element: &Self::Element) -> bool {
        element
            .part_names_map
            .as_ref()
            .is_some_and(|map| !map.IsEmpty())
    }
    fn PartNamesMap<'e>(&self, element: &'e Self::Element) -> &'e Self::PartNamesMap {
        element
            .part_names_map
            .as_ref()
            .expect("HasPartNamesMap guards PartNamesMap")
    }
    fn PushPartNamesMap(&self, names: &mut Self::PartNames, map: &Self::PartNamesMap) {
        names.PushMap(map)
    }
    fn InlineStyle(&self, e: &Self::Element) -> Option<Rc<Self::CSSPropertyValueSet>> {
        self.document.Node(e.index).FindAttribute("style")?;
        // cpp: CSSParserImpl::ParseInlineStyleDeclarationForElement
        // (css_parser_impl.cc:370-387). The source uses standard mode only for
        // HTML elements outside quirks mode; every other element uses quirks.
        let mode = if self.IsHTMLElement(e)
            && self.document.GetCompatibilityMode() != CompatibilityMode::kQuirks
        {
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode
        } else {
            crate::parser::css_parser_mode::CSSParserMode::kHTMLQuirksMode
        };
        Some(self.declarations.Resolve(self.document, e.index, mode))
    }
    fn PresentationAttributeStyle(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::CSSPropertyValueSet>> {
        let node = self.document.Node(element.index);
        if node.Namespace() == DOMNamespace::kHTML
            && !node.Attributes().iter().any(|attribute| {
                attribute.namespace_uri.is_empty() && attribute.local_name == "hidden"
            })
            && self
                .document
                .DirectionalityPresentationalHint(element.index)
                .is_none()
        {
            return None;
        }
        let properties = self.NewMutableCSSPropertyValueSet(
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        );
        // Chromium HTMLElement::CollectStyleForPresentationAttribute,
        // html_element.cc:412-420 (6c1d401fcca5e1b0030563a90c2f2bba168e0c15).
        // hidden is an HTML presentation hint, not an unoverrideable UA rule
        // or an attachment override. MatchPresentationalHints supplies its
        // cascade origin; canonical ComputedStyle owns the resulting display.
        if node.Namespace() == DOMNamespace::kHTML {
            if let Some(hidden) = node.Attributes().iter().find(|attribute| {
                attribute.namespace_uri.is_empty() && attribute.local_name == "hidden"
            }) {
                let (property, value) = if hidden.value.eq_ignore_ascii_case("until-found") {
                    (CSSPropertyID::kContentVisibility, CSSValueID::kHidden)
                } else {
                    (CSSPropertyID::kDisplay, CSSValueID::kNone)
                };
                self.SetLonghandProperty(properties.as_ref(), property, value);
            }
        }
        if node.Namespace() == DOMNamespace::kSVG {
            // SVGElement::CollectStyleForPresentationAttribute feeds the same
            // retained property-set owner as other presentational hints. Do
            // not bypass MatchPresentationalHints or repair fill/stroke after
            // the cascade: author rules must override these declarations.
            if let CSSPropertyValueSetRuleHandle::Mutable(set) = properties.as_ref().as_ref() {
                let mut set = set.borrow_mut();
                for property in crate::svg_presentation_attributes::CollectPresentationAttributes(
                    self.document,
                    element.index,
                ) {
                    set.SetLonghandProperty(property);
                }
            }
        } else if node.Namespace() == DOMNamespace::kMathML {
            // cpp: mathml/mathml_element.cc:36-124. These attributes synthesize
            // ordinary CSS declarations; their cascade position is owned by
            // MatchPresentationalHints rather than by MathML post-processing.
            if let Some(value) = node
                .FindAttribute("dir")
                .map(|attribute| attribute.value.as_str())
            {
                let direction = if value.eq_ignore_ascii_case("ltr") {
                    Some(CSSValueID::kLtr)
                } else if value.eq_ignore_ascii_case("rtl") {
                    Some(CSSValueID::kRtl)
                } else {
                    None
                };
                if let Some(direction) = direction {
                    self.SetLonghandProperty(
                        properties.as_ref(),
                        CSSPropertyID::kDirection,
                        direction,
                    );
                }
            }
            if let Some(value) = node
                .FindAttribute("mathsize")
                .map(|attribute| attribute.value.as_str())
            {
                let lower = value.to_ascii_lowercase();
                let disallowed = lower == "medium"
                    || lower.ends_with("large")
                    || lower.ends_with("small")
                    || lower == "smaller"
                    || lower == "larger"
                    || lower == "math";
                if !disallowed {
                    self.AddPresentationAttributeProperty(
                        properties.as_ref(),
                        CSSPropertyID::kFontSize,
                        value,
                    );
                }
            }
            for (attribute_name, property) in [
                ("mathbackground", CSSPropertyID::kBackgroundColor),
                ("mathcolor", CSSPropertyID::kColor),
            ] {
                if let Some(value) = node
                    .FindAttribute(attribute_name)
                    .map(|attribute| attribute.value.as_str())
                {
                    self.AddPresentationAttributeProperty(properties.as_ref(), property, value);
                }
            }
            if let Some(value) = node
                .FindAttribute("scriptlevel")
                .and_then(|attribute| Self::MathMLScriptLevelPropertyValue(&attribute.value))
            {
                self.AddPresentationAttributeProperty(
                    properties.as_ref(),
                    CSSPropertyID::kMathDepth,
                    &value,
                );
            }
            if let Some(value) = node
                .FindAttribute("displaystyle")
                .map(|attribute| attribute.value.as_str())
            {
                let math_style = if value.eq_ignore_ascii_case("false") {
                    Some(CSSValueID::kCompact)
                } else if value.eq_ignore_ascii_case("true") {
                    Some(CSSValueID::kNormal)
                } else {
                    None
                };
                if let Some(math_style) = math_style {
                    self.SetLonghandProperty(
                        properties.as_ref(),
                        CSSPropertyID::kMathStyle,
                        math_style,
                    );
                }
            }
            // cpp: mathml/mathml_token_element.cc:28-48. The legacy
            // mathvariant presentation hint is deliberately limited to
            // mathvariant=normal on <mi>.
            if node.Name() == "mi"
                && node
                    .FindAttribute("mathvariant")
                    .is_some_and(|attribute| attribute.value.eq_ignore_ascii_case("normal"))
            {
                self.SetLonghandProperty(
                    properties.as_ref(),
                    CSSPropertyID::kTextTransform,
                    CSSValueID::kNone,
                );
            }

            // cpp: mathml/mathml_space_element.cc:25-65 and
            // mathml/mathml_padded_element.cc:62-84. These element-specific
            // hints reuse ParseMathLength's strict, non-calculated,
            // non-percentage, nonnegative primitive. mspace height is the sum
            // of its height and depth attributes when both are valid.
            match node.Name() {
                "mspace" => {
                    if let Some(width) =
                        self.MathMLAttributePrimitiveValue(element, "width", false, true)
                    {
                        self.SetMathMLLengthPresentationAttribute(
                            properties.as_ref(),
                            CSSPropertyID::kWidth,
                            width,
                        );
                    }
                    let height = self.MathMLAttributePrimitiveValue(element, "height", false, true);
                    let depth = self.MathMLAttributePrimitiveValue(element, "depth", false, true);
                    match (height, depth) {
                        (Some(_), Some(_)) => {
                            let height = &node
                                .FindAttribute("height")
                                .expect("validated MathML height attribute exists")
                                .value;
                            let depth = &node
                                .FindAttribute("depth")
                                .expect("validated MathML depth attribute exists")
                                .value;
                            self.AddPresentationAttributeProperty(
                                properties.as_ref(),
                                CSSPropertyID::kHeight,
                                &format!("calc({height} + {depth})"),
                            );
                        }
                        (Some(value), None) | (None, Some(value)) => {
                            self.SetMathMLLengthPresentationAttribute(
                                properties.as_ref(),
                                CSSPropertyID::kHeight,
                                value,
                            );
                        }
                        (None, None) => {}
                    }
                }
                "mpadded" => {
                    if let Some(width) =
                        self.MathMLAttributePrimitiveValue(element, "width", false, true)
                    {
                        self.SetMathMLLengthPresentationAttribute(
                            properties.as_ref(),
                            CSSPropertyID::kWidth,
                            width,
                        );
                    }
                }
                _ => {}
            }
        } else if let Some(direction) = self
            .document
            .DirectionalityPresentationalHint(element.index)
        {
            self.SetLonghandProperty(
                properties.as_ref(),
                CSSPropertyID::kDirection,
                match direction {
                    TextDirection::kLtr => CSSValueID::kLtr,
                    TextDirection::kRtl => CSSValueID::kRtl,
                },
            );
        }
        (!self.PropertyValueSetIsEmpty(properties.as_ref())).then_some(properties)
    }
    fn AdditionalPresentationAttributeStyle(
        &self,
        _: &Self::Element,
    ) -> Option<Rc<Self::CSSPropertyValueSet>> {
        None
    }
    fn HasDirectionAuto(&self, e: &Self::Element) -> bool {
        self.document.HasDirectionAuto(e.index)
    }
    fn CachedDirectionality(&self, e: &Self::Element) -> TextDirection {
        self.document.CachedDirectionality(e.index)
    }
    fn AnimatedSMILStyleProperties(
        &self,
        _: &Self::Element,
    ) -> Option<Rc<Self::CSSPropertyValueSet>> {
        None
    }
    fn StateGetElement<'s>(&self, state: &'s Self::StyleResolverState) -> &'s Self::Element {
        &state.element
    }
    fn StateIsForPseudoElement(&self, state: &Self::StyleResolverState) -> bool {
        state.pseudo
    }

    fn DeclarationCache(&self) -> &StyleResolverDeclarationCache<Self::CSSPropertyValueSet> {
        &self.declarations.defaults
    }
    fn NewMutableCSSPropertyValueSet(
        &self,
        mode: crate::parser::css_parser_mode::CSSParserMode,
    ) -> Rc<Self::CSSPropertyValueSet> {
        let mutable = Rc::new(RefCell::new(MutableCSSPropertyValueSet::new(mode)));
        Rc::new(CSSPropertyValueSetRuleHandle::FromMutable(mutable))
    }
    fn PropertyValueSetIsEmpty(&self, set: &Self::CSSPropertyValueSet) -> bool {
        match set.as_ref() {
            CSSPropertyValueSetRuleHandle::Immutable(v) => v.Properties().is_empty(),
            CSSPropertyValueSetRuleHandle::Mutable(v) => v.borrow().Properties().is_empty(),
        }
    }
    fn SetLonghandProperty(
        &self,
        set: &Self::CSSPropertyValueSet,
        property: CSSPropertyID,
        value: CSSValueID,
    ) {
        let CSSPropertyValueSetRuleHandle::Mutable(set) = set.as_ref() else {
            self.unsupported("immutable-declaration-cache");
            return;
        };
        set.borrow_mut()
            .SetLonghandPropertyValue(property, production_css_value::identifier(value));
    }

    fn ClearMatchedRules(&self, collector: &mut Self::ElementRuleCollector) {
        collector.pending_inputs.clear();
        collector.pending_passes.clear();
    }
    fn CollectorPseudoId(
        &self,
        collector: &Self::ElementRuleCollector,
    ) -> layoutng_style::style::computed_style_constants::PseudoId {
        collector.pseudo.id
    }
    fn BeginAddingAuthorRulesForTreeScope(
        &self,
        collector: &mut Self::ElementRuleCollector,
        scope: &Self::TreeScope,
    ) {
        let scope = Rc::new(ProductionTreeScope(scope.root));
        collector.scope = Some(scope.clone());
        // Keep MatchResult's tree-scope table in lockstep with Chromium's
        // StyleResolver call. Inline style can be the only author input, so
        // delaying this until a scoped RuleSet is transferred leaves its
        // tree_order pointing at an empty table.
        collector
            .result
            .borrow_mut()
            .BeginAddingAuthorRulesForTreeScope(scope);
    }
    fn NewScopedRuleTreeScope(
        &self,
        _: &mut Self::ElementRuleCollector,
        _: &Self::TreeScope,
    ) -> Self::ScopedRuleTreeScope {
    }
    fn CollectMatchingShadowHostRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        collector: &mut Self::ElementRuleCollector,
    ) {
        if collector.pending_inputs.is_empty() {
            collector.pending_inputs.extend(
                self.author_rule_sets
                    .iter()
                    .filter(|input| {
                        input.tree_scope_root.unwrap_or(self.document.Root()) == resolver.root
                    })
                    .cloned(),
            );
        }
        collector
            .pending_passes
            .push(ProductionResolverRulePass::Host);
    }
    fn CollectMatchingSlottedRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        collector: &mut Self::ElementRuleCollector,
    ) {
        if collector.pending_inputs.is_empty() {
            collector.pending_inputs.extend(
                self.author_rule_sets
                    .iter()
                    .filter(|input| {
                        input.tree_scope_root.unwrap_or(self.document.Root()) == resolver.root
                    })
                    .cloned(),
            );
        }
        collector
            .pending_passes
            .push(ProductionResolverRulePass::Slotted);
    }
    fn CollectMatchingPartPseudoRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        collector: &mut Self::ElementRuleCollector,
        names: &Self::PartNames,
    ) {
        if collector.pending_inputs.is_empty() {
            collector.pending_inputs.extend(
                self.author_rule_sets
                    .iter()
                    .filter(|input| {
                        input.tree_scope_root.unwrap_or(self.document.Root()) == resolver.root
                    })
                    .cloned(),
            );
        }
        collector
            .pending_passes
            .push(ProductionResolverRulePass::Part(names.clone()));
    }
    fn CollectMatchingElementScopeRules(
        &self,
        resolver: &Self::ScopedStyleResolver,
        _: &Self::RootNode,
        collector: &mut Self::ElementRuleCollector,
        names: Option<&Self::PartNames>,
    ) {
        if collector.pending_inputs.is_empty() {
            collector.pending_inputs.extend(
                self.author_rule_sets
                    .iter()
                    .filter(|input| {
                        input.tree_scope_root.unwrap_or(self.document.Root()) == resolver.root
                    })
                    .cloned(),
            );
        }
        collector
            .pending_passes
            .push(ProductionResolverRulePass::ElementScope(names.cloned()));
    }
    fn SortAndTransferMatchedRules(
        &self,
        collector: &mut Self::ElementRuleCollector,
        origin: CascadeOrigin,
        _: bool,
        _: Option<&Self::StyleRuleUsageTracker>,
    ) {
        if collector.pending_inputs.is_empty() {
            return;
        }
        static PROFILE_RULES: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let profile = *PROFILE_RULES.get_or_init(|| {
            std::env::var_os("BROWSER_PROFILE_STYLE_RULES").is_some_and(|value| value == "1")
        });
        let setup_trace =
            profile.then(|| browser_tracing::span("style", "StyleResolver.SetupCollector"));
        let mut preparation_trace =
            browser_tracing::span("style", "StyleResolver.PrepareRuleCollection");
        let (prepared, cache_hit) = if let Some(cache) = self.cascade.rule_collections.as_ref() {
            cache.Prepare(&collector.pending_inputs, origin)
        } else {
            (
                Rc::new(ProductionRuleCollection::new(
                    &collector.pending_inputs,
                    origin,
                )),
                false,
            )
        };
        preparation_trace.set("cache_hit", cache_hit as u8 as f64);
        preparation_trace.set("rule_sets", collector.pending_inputs.len() as f64);
        drop(preparation_trace);
        let session = collector
            .selectors
            .borrow_mut()
            .StyleMatchSession(collector.document);
        let runtime = ProductionElementRuleCollectorRuntime::with_production_inputs(
            &session,
            prepared.layers.clone(),
            prepared.implicit_roots.clone(),
            self.cascade.container_runtime,
        );
        let mut matching_element = collector.element.index;
        let mut matching_pseudo = collector.pseudo.clone();
        if collector.pseudo.id == PseudoId::kPseudoIdPlaceholder {
            let Some(placeholder) = collector
                .document
                .UAShadowPseudoElement(collector.element.index, "-webkit-input-placeholder")
            else {
                self.unsupported("base:missing-placeholder-ua-shadow-owner");
                collector.pending_inputs.clear();
                collector.pending_passes.clear();
                return;
            };
            matching_element = placeholder;
            // ::placeholder is represented by an actual UA-shadow Element in
            // Blink. The outer StyleRequest still owns the resulting pseudo
            // style type; the nested ElementRuleCollector matches that actual
            // element as a normal element.
            matching_pseudo = ProductionPseudoRequest::Element();
        }
        let context = ProductionElementResolveContext {
            element: matching_element,
        };
        let recalc = collector.recalc.clone();
        let mut result = collector.result.borrow_mut();
        let first_result = result.GetMatchedProperties().len();
        let selector_filter = runtime.SelectorFilterForElement(context.element);
        let mut actual = ElementRuleCollector::<Backend, _>::new(
            &runtime,
            &context,
            &recalc,
            &selector_filter,
            &mut result,
            collector.inside_link,
        );
        actual.SetPseudoElementStyleRequest(&matching_pseudo);
        actual.SetMatchingUARules(collector.matching_ua);
        drop(setup_trace);
        let mut matching_trace =
            profile.then(|| browser_tracing::span("style", "StyleResolver.MatchRuleBuckets"));
        let mut collector_calls = 0usize;
        for group in &prepared.groups {
            let scope = collector.scope.as_ref().map(|scope| Rc::new(scope.0));
            let request = MatchRequest::new(group, scope, None);
            for pass in &collector.pending_passes {
                match pass {
                    ProductionResolverRulePass::Normal => {
                        actual.CollectMatchingRules(&request, None)
                    }
                    ProductionResolverRulePass::Host => {
                        actual.CollectMatchingShadowHostRules(&request)
                    }
                    ProductionResolverRulePass::Slotted => {
                        actual.CollectMatchingSlottedRules(&request)
                    }
                    ProductionResolverRulePass::Part(names) => {
                        actual.CollectMatchingPartPseudoRules(&request, Some(names))
                    }
                    ProductionResolverRulePass::ElementScope(names) => {
                        actual.CollectMatchingRules(&request, None);
                        if let Some(names) = names {
                            actual.CollectMatchingPartPseudoRules(&request, Some(names));
                        }
                    }
                }
                collector_calls += 1;
            }
        }
        let candidates_considered = actual.CandidatesConsidered();
        let selector_matches = actual.MatchedRulesForTest().len();
        if let Some(trace) = matching_trace.as_mut() {
            trace.set("node_index", context.element as f64);
            trace.set("candidates", candidates_considered as f64);
            trace.set("matches", selector_matches as f64);
            trace.set("collector_calls", collector_calls as f64);
        }
        drop(matching_trace);
        let _transfer_trace =
            profile.then(|| browser_tracing::span("style", "StyleResolver.TransferMatchedRules"));
        let inputs = &collector.pending_inputs[..];
        let incomplete_match = actual.MatchedRulesForTest().iter().any(|matched| {
            let Some(input) = inputs.get(matched.StyleSheetIndex()) else {
                return true;
            };
            let Some(routing) = input.routing.as_deref() else {
                return false;
            };
            let fallback = routing.RequiresCompatibilityFallback(matched.Rule());
            if fallback && std::env::var_os("RECHROM_DEBUG_STYLE_RESOLVER").is_some() {
                eprintln!(
                    "translated compatibility rule node={} selector={}",
                    collector.element.index,
                    matched.Rule().SelectorsText().Utf8(),
                );
            }
            fallback
        });
        if incomplete_match {
            self.unsupported("base:compatibility-rule-match");
        }
        actual.RetainMatchedRules(|matched| {
            inputs.get(matched.StyleSheetIndex()).is_some_and(|input| {
                input
                    .routing
                    .as_deref()
                    .is_none_or(|routing| routing.IsAuthoritativeRetainedRule(matched.Rule()))
            })
        });
        actual.SortAndTransferMatchedRules(origin, false, None);
        let transferred_property_sets =
            actual.MatchedResult().GetMatchedProperties().len() - first_result;
        {
            let mut metrics = self.base_style_metrics.borrow_mut();
            metrics.collector_calls += collector_calls;
            metrics.candidates_considered += candidates_considered;
            metrics.selector_matches += selector_matches;
            metrics.transferred_property_sets += transferred_property_sets;
        }
        self.matched_signatures.borrow_mut().extend(
            actual.MatchedRulesForTest().iter().filter_map(|rule| {
                inputs.get(rule.StyleSheetIndex()).map(|input| {
                    (
                        input.sheet_base + rule.PositionInStyleSheet() as usize + 1,
                        rule.Specificity(),
                        rule.Proximity(),
                    )
                })
            }),
        );
        collector.pending_inputs.clear();
        collector.pending_passes.clear();
    }
    fn AddElementStyleProperties(
        &self,
        collector: &mut Self::ElementRuleCollector,
        properties: Option<&Self::CSSPropertyValueSet>,
        origin: CascadeOrigin,
        is_cacheable: bool,
        is_inline_style: bool,
    ) {
        let Some(properties) = properties else { return };
        let mut data = crate::resolver::match_result::MatchedPropertiesData::default();
        data.origin = origin;
        data.set_is_inline_style(is_inline_style);
        collector
            .result
            .borrow_mut()
            .AddMatchedProperties(properties.clone(), None, data);
        if !is_cacheable {
            collector.result.borrow_mut().SetIsCacheable(false);
        }
    }
    fn AddTryStyleProperties(&self, _: &mut Self::ElementRuleCollector) {}
    fn AddTryTacticsStyleProperties(&self, _: &mut Self::ElementRuleCollector) {}
    fn CollectMatchingUserRules(&self, _: &Self::StyleEngine, _: &mut Self::ElementRuleCollector) {}
    fn SetMatchingUARules(&self, collector: &mut Self::ElementRuleCollector, value: bool) {
        collector.matching_ua = value
    }
    fn CollectMatchingRules(
        &self,
        collector: &mut Self::ElementRuleCollector,
        group: &Self::RuleSetGroup,
        _: Option<&Self::TreeScope>,
        _: Option<&Self::Element>,
        _: Option<&Self::PartNames>,
    ) {
        let scope_root = collector.scope.as_ref().map(|scope| scope.0);
        for rules in group.RuleSets() {
            if let Some(input) = std::iter::once(&self.html_user_agent_rules)
                .chain(self.html_quirks_user_agent_rules.iter())
                .chain(self.forced_color_rules.iter())
                .chain(self.fullscreen_user_agent_rules.iter())
                .chain(self.svg_user_agent_rules.iter())
                .chain(self.mathml_user_agent_rules.iter())
                .find(|input| Rc::ptr_eq(rules, &input.rule_set))
            {
                collector.pending_inputs.push(input.clone());
            } else if let Some(input) = self.author_rule_sets.iter().find(|input| {
                Rc::ptr_eq(rules, &input.rule_set)
                    && scope_root.is_none_or(|scope| {
                        input.tree_scope_root.unwrap_or(self.document.Root()) == scope
                    })
            }) {
                collector.pending_inputs.push(input.clone());
            }
        }
        if !group.RuleSets().is_empty() {
            collector
                .pending_passes
                .push(ProductionResolverRulePass::Normal);
        }
    }

    fn DefaultHtmlStyle(&self) -> Rc<Self::RuleSet> {
        self.html_user_agent_rules.rule_set.clone()
    }
    fn DefaultSVGStyle(&self) -> Rc<Self::RuleSet> {
        self.svg_user_agent_rules
            .as_ref()
            .map(|input| input.rule_set.clone())
            .unwrap_or_else(|| {
                self.unsupported("svg-user-agent-rules");
                self.empty_rules.clone()
            })
    }
    fn DefaultMathMLStyle(&self) -> Rc<Self::RuleSet> {
        self.mathml_user_agent_rules
            .as_ref()
            .map(|input| input.rule_set.clone())
            .unwrap_or_else(|| {
                self.unsupported("mathml-user-agent-rules");
                self.empty_rules.clone()
            })
    }
    fn DefaultFullscreenStyle(&self) -> Rc<Self::RuleSet> {
        self.fullscreen_user_agent_rules
            .as_ref()
            .map(|input| input.rule_set.clone())
            .unwrap_or_else(|| {
                self.unsupported("fullscreen-user-agent-rules");
                self.empty_rules.clone()
            })
    }
    fn DefaultHtmlQuirksStyle(&self) -> Rc<Self::RuleSet> {
        self.html_quirks_user_agent_rules
            .as_ref()
            .map(|input| input.rule_set.clone())
            .unwrap_or_else(|| {
                self.unsupported("quirks-user-agent-rules");
                self.empty_rules.clone()
            })
    }
    fn DefaultViewSourceStyle(&self) -> Rc<Self::RuleSet> {
        self.unsupported("view-source-user-agent-rules");
        self.empty_rules.clone()
    }
    fn DefaultForcedColorStyle(&self) -> Rc<Self::RuleSet> {
        self.forced_color_rules
            .as_ref()
            .map(|input| input.rule_set.clone())
            .unwrap_or_else(|| {
                self.unsupported("forced-colors-user-agent-rules");
                self.empty_rules.clone()
            })
    }
    fn DefaultJSONDocumentStyle(&self) -> Rc<Self::RuleSet> {
        self.unsupported("json-user-agent-rules");
        self.empty_rules.clone()
    }
    fn DefaultPseudoElementStyleOrNull(&self) -> Option<Rc<Self::RuleSet>> {
        None
    }
    fn DefaultForcedColorsMediaControlsStyle(&self) -> Rc<Self::RuleSet> {
        self.forced_colors_media_controls_rules.clone()
    }
    fn DefaultMediaControlsStyle(&self) -> Rc<Self::RuleSet> {
        self.media_controls_rules.clone()
    }
    fn HasFullscreenElements(&self) -> bool {
        self.document.HasFullscreenElements()
    }
    fn InQuirksMode(&self, _: &Self::Document) -> bool {
        self.document.GetCompatibilityMode() == CompatibilityMode::kQuirks
    }
    fn IsViewSource(&self, _: &Self::Document) -> bool {
        false
    }
    fn IsJSONDocument(&self, _: &Self::Document) -> bool {
        false
    }
    fn ActiveViewTransitionStyle(
        &self,
        _: &Self::StyleEngine,
        _: &Self::Element,
    ) -> Option<Rc<Self::RuleSet>> {
        None
    }
    fn RuleSetGroupCache(&self) -> &RefCell<Vec<(u32, Self::RuleSetGroup)>> {
        &self.ua_group_cache
    }
    fn RuleSetGroupAddRuleSet(&self, group: &mut Self::RuleSetGroup, rules: &Rc<Self::RuleSet>) {
        group.AddRuleSet(rules.clone())
    }
    fn RuleSetGroupIsFull(&self, group: &Self::RuleSetGroup) -> bool {
        group.IsFull()
    }
    fn RuleSetGroupIsEmpty(&self, group: &Self::RuleSetGroup) -> bool {
        group.IsEmpty()
    }
    fn RuleSetGroupAssertEqualTo(
        &self,
        group: &Self::RuleSetGroup,
        reference: &Self::RuleSetGroup,
    ) {
        debug_assert_eq!(group.RuleSets().len(), reference.RuleSets().len());
        debug_assert!(group
            .RuleSets()
            .iter()
            .zip(reference.RuleSets())
            .all(|(a, b)| Rc::ptr_eq(a, b)));
    }
    fn SetZIndex(&self, b: &mut Self::ComputedStyleBuilder, v: i32) {
        b.SetZIndex(v)
    }
    fn SetForcesStackingContext(&self, b: &mut Self::ComputedStyleBuilder, v: bool) {
        b.SetForcesStackingContext(v)
    }
    fn SetDisplay(&self, b: &mut Self::ComputedStyleBuilder, v: EDisplay) {
        b.SetDisplay(v)
    }
    fn SetPosition(&self, b: &mut Self::ComputedStyleBuilder, v: EPosition) {
        b.SetPosition(v)
    }
    fn SetOverflowX(&self, b: &mut Self::ComputedStyleBuilder, v: EOverflow) {
        b.SetOverflowX(v)
    }
    fn SetOverflowY(&self, b: &mut Self::ComputedStyleBuilder, v: EOverflow) {
        b.SetOverflowY(v)
    }
    fn ApplyVisionDeficiencyStyle(
        &self,
        _: &Self::StyleEngine,
        _: &mut Self::ComputedStyleBuilder,
    ) {
    }
}

impl StyleResolverBaseStyleBackend for ProductionRuleMatchingHost<'_> {
    type StyleRequest = ProductionStyleRequest;
    type StyleRecalcContext = ProductionStyleRecalcContext;
    type StyleCascade = ProductionBaseStyleCascade;
    type MatchResult = MatchResult;
    type CacheKey = MatchedPropertiesCacheKey;
    type CacheEntry =
        MatchedPropertiesCacheEntry<StyleAdjusterElementTypeForCache, ProductionCachedVariables>;
    type ElementTypeForCache = StyleAdjusterElementTypeForCache;
    type CSSPropertyValue = crate::css_property_value::CSSPropertyValue<ProductionCSSValueDispatch>;
    type CSSValue = Rc<production_css_value::Value>;
    type LengthConversionFlags = u32;
    type StyleBaseData = Persistent<NativeStyleBaseData>;
    type CSSBitset = NativeCSSBitset;
    type ElementAnimations = ProductionElementAnimations;
    type AnimationData = ();
    type ActiveInterpolationsMap =
        crate::resolver::production_style_cascade::ProductionActiveInterpolationsMap;
    type AnchorEvaluator = ();
    type ContainerNode = ProductionResolverElement;
    type StyleRule = ();
    type CustomHighlightNames = std::collections::HashSet<AtomicString>;
    type CurrentColor = foundation::Color;
    fn RetainElement(&self, element: &Self::Element) -> Rc<Self::Element> {
        Rc::new(element.clone())
    }
    fn ElementComputedStyle(&self, element: &Self::Element) -> Option<Rc<Self::ComputedStyle>> {
        self.document
            .ResolvedStyleFor(element.index)
            .map(|resolved| Rc::new(resolved.native_style.clone()))
    }
    fn ParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>> {
        self.document.Node(element.index).Parent().map(|index| {
            Rc::new(ProductionResolverElement::ForNode(
                self.document,
                index,
                &element.media_type,
                true,
            ))
        })
    }
    fn ParentElementOrDocumentFragment(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::ContainerNode>> {
        self.ParentElement(element)
    }
    fn ElementTreeRoot(&self, element: &Self::Element) -> Rc<Self::ContainerNode> {
        Rc::new(element.clone())
    }
    fn DocumentElement(&self, _document: &Self::Document) -> Option<Rc<Self::Element>> {
        self.document
            .Node(self.document.Root())
            .Children()
            .iter()
            .copied()
            .find(|&index| self.document.Node(index).Type() == DOMNodeType::kElement)
            .map(|index| {
                Rc::new(ProductionResolverElement::ForNode(
                    self.document,
                    index,
                    &self.initial.media.media_type,
                    true,
                ))
            })
    }
    fn IsLink(&self, element: &Self::Element) -> bool {
        let node = self.document.Node(element.index);
        matches!(
            node.Name().to_ascii_lowercase().as_str(),
            "a" | "area" | "link"
        ) && node.FindAttribute("href").is_some()
    }
    fn IsBodyElement(&self, element: &Self::Element) -> bool {
        self.document
            .Node(element.index)
            .Name()
            .eq_ignore_ascii_case("body")
    }
    fn IsMathMLElement(&self, element: &Self::Element) -> bool {
        self.document.Node(element.index).Namespace() == DOMNamespace::kMathML
    }
    fn IsAnchorOrAreaElement(&self, element: &Self::Element) -> bool {
        matches!(
            self.document
                .Node(element.index)
                .Name()
                .to_ascii_lowercase()
                .as_str(),
            "a" | "area"
        )
    }
    fn IsAtShadowBoundary(&self, element: &Self::Element) -> bool {
        element
            .containing_shadow_root
            .as_ref()
            .is_some_and(|root| self.document.Node(element.index).Parent() == Some(root.root))
    }
    fn IsInlineIndependentStyleChange(&self, element: &Self::Element) -> bool {
        self.cascade.incremental_style.is_some() && self.cascade.subject_node == Some(element.index)
    }
    fn HasCustomStyleCallbacks(&self, element: &Self::Element) -> bool {
        // cpp: SVGElement, HTMLFormControlElement, HTMLFrameOwnerElement,
        // HTMLPlugInElement and the specialized HTML constructors set this bit.
        // UA implementation elements and image fallback also have callbacks.
        let node = self.document.Node(element.index);
        node.Namespace() == DOMNamespace::kSVG
            || self.document.ShadowPseudoId(element.index).is_some()
            || node.Namespace() == DOMNamespace::kHTML
                && matches!(
                    node.Name(),
                    "button"
                        | "fieldset"
                        | "input"
                        | "output"
                        | "select"
                        | "textarea"
                        | "meter"
                        | "progress"
                        | "canvas"
                        | "frameset"
                        | "frame"
                        | "iframe"
                        | "object"
                        | "embed"
                        | "audio"
                        | "video"
                        | "img"
                        | "permission"
                        | "geolocation"
                        | "usermedia"
                )
    }
    fn HasContenteditableAttribute(&self, element: &Self::Element) -> bool {
        self.document
            .Node(element.index)
            .FindAttribute("contenteditable")
            .is_some()
    }
    fn ImplicitAnchorElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>> {
        None
    }
    fn GetCascadeFilter(&self, element: &Self::Element) -> CascadeFilter {
        let filter = CascadeFilter::new();
        if layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(
            self.cascade.pseudo_id,
        ) {
            filter.Add(CSSProperty::kValidForHighlight)
        } else {
            filter
        }
    }
    fn MarkChildrenAffectedByForwardPositionalRules(&self, node: &Self::ContainerNode) {
        self.base_style_metrics
            .borrow_mut()
            .forward_positional_parents
            .insert(node.index);
    }
    fn MarkChildrenAffectedByBackwardPositionalRules(&self, node: &Self::ContainerNode) {
        self.base_style_metrics
            .borrow_mut()
            .backward_positional_parents
            .insert(node.index);
    }
    fn SetUsesTreeCountingFunctions(&self, engine: &Self::StyleEngine) {
        self.base_style_metrics
            .borrow_mut()
            .uses_tree_counting_functions = true;
    }
    fn SetUsesRootRelativeUnits(&self, engine: &Self::StyleEngine, value: bool) {
        self.base_style_metrics
            .borrow_mut()
            .uses_root_relative_units |= value;
    }
    fn SetUsesGlyphRelativeUnits(&self, engine: &Self::StyleEngine, value: bool) {
        self.base_style_metrics
            .borrow_mut()
            .uses_glyph_relative_units |= value;
    }
    fn SetUsesLineHeightUnits(&self, engine: &Self::StyleEngine, value: bool) {
        self.base_style_metrics.borrow_mut().uses_line_height_units |= value;
    }
    fn AddViewportUnitFlags(&self, document: &Self::Document, flags: u32) {
        self.base_style_metrics.borrow_mut().viewport_unit_flags |= flags;
    }
    fn SetTextLinkTextColor(&self, document: &Self::Document, color: Self::CurrentColor) {
        self.base_style_metrics.borrow_mut().text_link_text_color = Some(color);
    }
    fn ActiveModalDialog(&self, document: &Self::Document) -> Option<Rc<Self::Element>> {
        None
    }
    fn FullscreenElementFrom(&self, _document: &Self::Document) -> Option<Rc<Self::Element>> {
        self.document.FullscreenElement().map(|element| {
            Rc::new(ProductionResolverElement::ForNode(
                self.document,
                element,
                &self.initial.media.media_type,
                true,
            ))
        })
    }
    fn RuntimeFeatureEnabled(&self, feature: ResolverRuntimeFeature) -> bool {
        false
    }
    fn NewStyleRequest(&self) -> Self::StyleRequest {
        ProductionStyleRequest::new(if self.cascade.pseudo_id == PseudoId::kPseudoIdNone {
            ProductionPseudoRequest::Element()
        } else {
            let mut request = ProductionPseudoRequest::Pseudo(
                self.cascade.pseudo_id,
                self.cascade.pseudo_argument.clone(),
            );
            request.search_text = self.cascade.search_text;
            request
        })
    }
    fn SetRequestParentOverride(
        &self,
        request: &mut Self::StyleRequest,
        style: Option<&Self::ComputedStyle>,
    ) {
        request.parent_override = style.cloned()
    }
    fn SetRequestLayoutParentOverride(
        &self,
        request: &mut Self::StyleRequest,
        style: Option<&Self::ComputedStyle>,
    ) {
        request.layout_parent_override = style.cloned()
    }
    fn RequestIsPseudoStyle(&self, request: &Self::StyleRequest) -> bool {
        request.pseudo.id != PseudoId::kPseudoIdNone
    }
    fn RequestPseudoId(&self, request: &Self::StyleRequest) -> PseudoId {
        request.pseudo.id
    }
    fn RequestUAOnly(&self, request: &Self::StyleRequest) -> bool {
        false
    }
    fn RequestExcludesSMIL(&self, request: &Self::StyleRequest) -> bool {
        true
    }
    fn NewStyleRecalcContext(&self) -> Self::StyleRecalcContext {
        let mut context =
            self.cascade
                .subject_node
                .map_or_else(ProductionStyleRecalcContext::default, |node| {
                    let size_container = self.cascade.container_runtime.and_then(|runtime| {
                        runtime.NearestSizeContainerForStyleRequest(node, self.cascade.pseudo_id)
                    });
                    ProductionStyleRecalcContext::ForStyleRequest(
                        self.document,
                        node,
                        self.cascade.pseudo_id,
                        size_container,
                    )
                });
        context.can_use_incremental_style = self.cascade.incremental_style.is_some();
        context
    }
    fn RecalcFlag(&self, context: &Self::StyleRecalcContext, flag: ResolverRecalcFlag) -> bool {
        match flag {
            ResolverRecalcFlag::EnsuringStyle => self.cascade.is_ensuring_style,
            ResolverRecalcFlag::CanUseIncrementalStyle => context.can_use_incremental_style,
            _ => false,
        }
    }
    fn NewStyleResolverState(
        &self,
        document: &Self::Document,
        element: &Self::Element,
        context: Option<&Self::StyleRecalcContext>,
        request: &Self::StyleRequest,
    ) -> Self::StyleResolverState {
        ProductionResolverState {
            element: element.clone(),
            pseudo: self.RequestIsPseudoStyle(request),
            pseudo_id: self.RequestPseudoId(request),
            builder: ComputedStyleBuilder::from_style(Self::style(&self.initial.initial)),
            parent: request.parent_override.clone().map(Rc::new),
            layout_parent: request
                .layout_parent_override
                .clone()
                .or_else(|| request.parent_override.clone())
                .map(Rc::new),
            originating_style: None,
            output: None,
            had_no_matched_properties: false,
            conditionally_affects_animations: false,
            has_tree_scoped_reference: false,
            rejected_legacy_overlapping: false,
            length_conversion_flags: 0,
            incremental_properties: Vec::new(),
        }
    }
    fn NewStyleCascade(&self, state: &mut Self::StyleResolverState) -> Self::StyleCascade {
        ProductionBaseStyleCascade {
            result: Rc::new(RefCell::new(MatchResult::default())),
            state: Some(std::ptr::NonNull::from(state)),
            important_set: None,
            animations: None,
            transitions: None,
            inline_style_lost: false,
        }
    }
    fn StateElement(&self, state: &Self::StyleResolverState) -> Rc<Self::Element> {
        Rc::new(state.element.clone())
    }
    fn StateAnimatingElement(&self, state: &Self::StyleResolverState) -> Option<Rc<Self::Element>> {
        (!state.pseudo
            && (self.cascade.animations.is_some_and(|map| !map.IsEmpty())
                || self.cascade.transitions.is_some_and(|map| !map.IsEmpty())))
        .then(|| Rc::new(state.element.clone()))
    }
    fn StatePseudoElement(&self, state: &Self::StyleResolverState) -> Option<Rc<Self::Element>> {
        None
    }
    fn StateStyledElement(&self, state: &Self::StyleResolverState) -> Option<Rc<Self::Element>> {
        Some(Rc::new(state.element.clone()))
    }
    fn StateParentStyle(
        &self,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::ComputedStyle>> {
        state.parent.clone()
    }
    fn StateLayoutParentStyle(
        &self,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::ComputedStyle>> {
        state.layout_parent.clone()
    }
    fn StateOriginatingElementStyle(
        &self,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::ComputedStyle>> {
        state.originating_style.clone()
    }
    fn StateSetParentStyle(
        &self,
        state: &mut Self::StyleResolverState,
        style: Rc<Self::ComputedStyle>,
    ) {
        state.parent = Some(style)
    }
    fn StateSetLayoutParentStyle(
        &self,
        state: &mut Self::StyleResolverState,
        style: Rc<Self::ComputedStyle>,
    ) {
        state.layout_parent = Some(style)
    }
    fn StateIsForHighlight(&self, state: &Self::StyleResolverState) -> bool {
        layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(state.pseudo_id)
    }
    fn StateHadNoMatchedProperties(&self, state: &Self::StyleResolverState) -> bool {
        state.had_no_matched_properties
    }
    fn StateSetHadNoMatchedProperties(&self, state: &mut Self::StyleResolverState) {
        state.had_no_matched_properties = true
    }
    fn StateCanTriggerAnimations(&self, state: &Self::StyleResolverState) -> bool {
        false
    }
    fn StateCanAffectAnimations(&self, state: &Self::StyleResolverState) -> bool {
        state.conditionally_affects_animations || state.builder.CanAffectAnimations()
    }
    fn StateAffectsCompositorSnapshots(&self, state: &Self::StyleResolverState) -> bool {
        false
    }
    fn StateInsideLink(&self, state: &Self::StyleResolverState) -> EInsideLink {
        let inherited = state
            .parent
            .as_ref()
            .map_or(EInsideLink::kNotInsideLink, |parent| {
                Self::style(parent).InsideLink()
            });
        if !state.pseudo && self.IsLink(&state.element) {
            // The retained document does not yet expose Blink's
            // VisitedLinkState owner. A real link still has the exact
            // unvisited state instead of being treated as outside a link;
            // descendants inherit it through their parent ComputedStyle.
            EInsideLink::kInsideUnvisitedLink
        } else {
            inherited
        }
    }
    fn StateStyleBuilder<'s>(
        &self,
        state: &'s Self::StyleResolverState,
    ) -> &'s Self::ComputedStyleBuilder {
        &state.builder
    }
    fn StateStyleBuilderMut<'s>(
        &self,
        state: &'s mut Self::StyleResolverState,
    ) -> &'s mut Self::ComputedStyleBuilder {
        &mut state.builder
    }
    fn StateCreateNewClonedStyle(
        &self,
        state: &mut Self::StyleResolverState,
        style: &Self::ComputedStyle,
    ) {
        state.builder = ComputedStyleBuilder::from_style(Self::style(style))
    }
    fn StateCreateNewStyle(
        &self,
        state: &mut Self::StyleResolverState,
        initial: &Self::ComputedStyle,
        parent: &Self::ComputedStyle,
        at_shadow_boundary: bool,
    ) {
        state.builder = ComputedStyleBuilder::from_initial_and_parent(
            Self::style(initial),
            Self::style(parent),
            if at_shadow_boundary {
                IsAtShadowBoundary::kAtShadowBoundary
            } else {
                IsAtShadowBoundary::kNotAtShadowBoundary
            },
        )
    }
    fn StateTakeStyle(&self, state: &mut Self::StyleResolverState) -> Rc<Self::ComputedStyle> {
        if state.output.is_none() {
            state.output = Some(Rc::new(Persistent::from_ptr(
                state.builder.TakeStyle() as *mut ComputedStyle
            )));
        }
        state.output.as_ref().unwrap().clone()
    }
    fn StateLoadPendingResources(&self, state: &mut Self::StyleResolverState) {
        // Ordinary typed declarations in this slice have no pending resources.
    }
    fn StateUpdateFont(&self, state: &mut Self::StyleResolverState) {
        // StyleCascade::Apply performs the production font update.
    }
    fn StateSetTreeScopedReference(&self, state: &mut Self::StyleResolverState) {
        // cpp: style_resolver_state.h:268-269. This state bit is deliberately
        // separate from ComputedStyle: Blink consumes it while deciding
        // whether the matched-properties cache can safely retain the result.
        state.has_tree_scoped_reference = true;
    }
    fn StateSetConditionallyAffectsAnimations(&self, state: &mut Self::StyleResolverState) {
        // cpp: style_resolver_state.h:225-237; style_resolver_state.cc:541-544.
        state.conditionally_affects_animations = true;
    }
    fn StateSetComputedStyleFlagsFromAuthorFlags(
        &self,
        state: &mut Self::StyleResolverState,
        flags: u64,
    ) {
        if !state.incremental_properties.is_empty() {
            let _trace =
                browser_tracing::span("style", "StyleResolver.ApplyInlineStyleIncrementally");
            let properties = std::mem::take(&mut state.incremental_properties);
            let result = crate::resolver::production_style_cascade::ApplyInlineProperties(
                ProductionCascadeInputs {
                    document: Some(self.document),
                    media: self.initial.media.clone(),
                    base: self.initial.initial.clone(),
                    parent: state.parent.as_deref().cloned(),
                    root_style: self.cascade.root_style.clone(),
                    match_result: MatchResult::default(),
                    root_font_size: self.cascade.root_font_size,
                    color_scheme_settings: self.cascade.color_scheme_settings,
                    system_colors: self.cascade.system_colors,
                    text_link_text_color: self.cascade.text_link_text_color,
                    is_document_element: false,
                    pseudo_id: self.cascade.pseudo_id,
                    originating_style: self.cascade.originating_style.clone(),
                    subject_node: self.cascade.subject_node,
                    container_runtime: self.cascade.container_runtime,
                    custom_properties: None,
                    parent_custom_properties: self.cascade.parent_custom_properties,
                    substitution_context: self.cascade.substitution_context,
                    image_resolver: self.cascade.image_resolver,
                    property_registry: self.cascade.property_registry,
                    function_registry: self.cascade.function_registry,
                    animations: None,
                    transitions: None,
                    font_metrics: self.cascade.font_metrics,
                },
                &mut state.builder,
                &properties,
            );
            match result {
                Ok(length_flags) => {
                    state.length_conversion_flags |= length_flags;
                    let old = self
                        .cascade
                        .incremental_style
                        .as_ref()
                        .expect("MISU owns previous outputs");
                    *self.cascade_custom_output.borrow_mut() = Some(old.custom_properties.clone());
                    self.matched_signatures
                        .borrow_mut()
                        .extend_from_slice(old.matched_signatures);
                    self.base_style_metrics
                        .borrow_mut()
                        .incremental_style_updates += 1;
                }
                Err(errors) => self.fallbacks.borrow_mut().extend(
                    errors
                        .into_iter()
                        .map(ProductionRuleMatchingFallback::Cascade),
                ),
            }
        }
        // cpp: style_resolver_state.cc:546-567.
        if flags & CSSProperty::kBackground != 0 {
            state.builder.SetHasAuthorBackground();
        }
        if flags & CSSProperty::kBorder != 0 {
            state.builder.SetHasAuthorBorder();
        }
        if flags & CSSProperty::kBorderRadius != 0 {
            state.builder.SetHasAuthorBorderRadius();
        }
        let inside_link = self.StateInsideLink(state);
        if (inside_link != EInsideLink::kInsideVisitedLink
            && flags & CSSProperty::kHighlightColors != 0)
            || (inside_link == EInsideLink::kInsideVisitedLink
                && flags & CSSProperty::kVisitedHighlightColors != 0)
        {
            state.builder.SetHasAuthorHighlightColors();
        }
    }
    fn StateRejectedLegacyOverlapping(&self, state: &Self::StyleResolverState) -> bool {
        state.rejected_legacy_overlapping
    }
    fn StyleFlag(&self, style: &Self::ComputedStyle, flag: ResolverStyleFlag) -> bool {
        let style = Self::style(style);
        match flag {
            ResolverStyleFlag::AnchorFunctions => style.HasAnchorFunctions(),
            ResolverStyleFlag::AnchorEvaluator => style.HasAnchorEvaluator(),
            ResolverStyleFlag::RootRelativeUnits => style.HasRootRelativeUnits(),
            ResolverStyleFlag::GlyphRelativeUnits => style.HasGlyphRelativeUnits(),
            ResolverStyleFlag::LineHeightRelativeUnits => style.HasLineHeightRelativeUnits(),
            ResolverStyleFlag::FontRelativeUnits => style.HasFontRelativeUnits(),
            ResolverStyleFlag::SiblingFunctions => style.HasSiblingFunctions(),
            ResolverStyleFlag::InlineStyleLostCascade => style.InlineStyleLostCascade(),
            ResolverStyleFlag::ExplicitInheritance => style.HasExplicitInheritance(),
            ResolverStyleFlag::CanAffectAnimations => style.CanAffectAnimations(),
            ResolverStyleFlag::OriginalDisplayInlineType => style.IsOriginalDisplayInlineType(),
            ResolverStyleFlag::EmUnits => style.HasEmUnits(),
            ResolverStyleFlag::StaticViewportUnits => style.HasStaticViewportUnits(),
            ResolverStyleFlag::DynamicViewportUnits => style.HasDynamicViewportUnits(),
            ResolverStyleFlag::ContainerRelativeValue => style.HasContainerRelativeValue(),
            ResolverStyleFlag::LogicalDirectionRelativeUnits => {
                style.HasLogicalDirectionRelativeUnits()
            }
            ResolverStyleFlag::ElementDependentRandomFunctions => {
                style.HasElementDependentRandomFunctions()
            }
            ResolverStyleFlag::EnsuredInDisplayNone => style.IsEnsuredInDisplayNone(),
            ResolverStyleFlag::SkipsContents => style.SkipsContents(),
            ResolverStyleFlag::IsLink => style.IsLink(),
            ResolverStyleFlag::HTMLInert => style.IsHTMLInert(),
            ResolverStyleFlag::HTMLInertIsInherited => style.IsHTMLInertIsInherited(),
            ResolverStyleFlag::CSSInert => style.IsCSSInert(),
            ResolverStyleFlag::CSSInertIsInherited => style.IsCSSInertIsInherited(),
            ResolverStyleFlag::NonUniversalHighlightPseudoStyles => {
                style.HasNonUniversalHighlightPseudoStyles()
            }
            ResolverStyleFlag::NonUaHighlightPseudoStyles => style.HasNonUaHighlightPseudoStyles(),
            ResolverStyleFlag::HighlightsDependOnSizeContainerQueries => {
                style.HighlightsDependOnSizeContainerQueries()
            }
            ResolverStyleFlag::AffectedByDrag => style.AffectedByDrag(),
            ResolverStyleFlag::AffectedByFocusWithin => style.AffectedByFocusWithin(),
            ResolverStyleFlag::AffectedByHover => style.AffectedByHover(),
            ResolverStyleFlag::AffectedByActive => style.AffectedByActive(),
            ResolverStyleFlag::StartingStyle => style.IsStartingStyle(),
            ResolverStyleFlag::DependsOnSizeContainerQueries => {
                style.DependsOnSizeContainerQueries()
            }
            ResolverStyleFlag::DependsOnStyleContainerQueries => {
                style.DependsOnStyleContainerQueries()
            }
            ResolverStyleFlag::DependsOnScrollStateContainerQueries => {
                style.DependsOnScrollStateContainerQueries()
            }
            ResolverStyleFlag::DependsOnAnchoredContainerQueries => {
                style.DependsOnAnchoredContainerQueries()
            }
            ResolverStyleFlag::FirstLineDependsOnSizeContainerQueries => {
                style.FirstLineDependsOnSizeContainerQueries()
            }
        }
    }
    fn BuilderFlag(&self, builder: &Self::ComputedStyleBuilder, flag: ResolverStyleFlag) -> bool {
        match flag {
            ResolverStyleFlag::AnchorFunctions => builder.HasAnchorFunctions(),
            ResolverStyleFlag::AnchorEvaluator => builder.HasAnchorEvaluator(),
            ResolverStyleFlag::RootRelativeUnits => builder.HasRootRelativeUnits(),
            ResolverStyleFlag::GlyphRelativeUnits => builder.HasGlyphRelativeUnits(),
            ResolverStyleFlag::LineHeightRelativeUnits => builder.HasLineHeightRelativeUnits(),
            ResolverStyleFlag::FontRelativeUnits => {
                builder.HasEmUnits()
                    || builder.HasRootRelativeUnits()
                    || builder.HasGlyphRelativeUnits()
            }
            ResolverStyleFlag::SiblingFunctions => builder.HasSiblingFunctions(),
            ResolverStyleFlag::InlineStyleLostCascade => builder.InlineStyleLostCascade(),
            ResolverStyleFlag::ExplicitInheritance => builder.HasExplicitInheritance(),
            ResolverStyleFlag::CanAffectAnimations => builder.CanAffectAnimations(),
            ResolverStyleFlag::OriginalDisplayInlineType => builder.IsOriginalDisplayInlineType(),
            ResolverStyleFlag::EmUnits => builder.HasEmUnits(),
            ResolverStyleFlag::StaticViewportUnits => {
                builder.ViewportUnitFlags()
                    & layoutng_style::style::computed_style_constants::ViewportUnitFlag::kStatic
                        as u32
                    != 0
            }
            ResolverStyleFlag::DynamicViewportUnits => {
                builder.ViewportUnitFlags()
                    & layoutng_style::style::computed_style_constants::ViewportUnitFlag::kDynamic
                        as u32
                    != 0
            }
            ResolverStyleFlag::ContainerRelativeValue => builder.HasContainerRelativeValue(),
            ResolverStyleFlag::LogicalDirectionRelativeUnits => {
                builder.HasLogicalDirectionRelativeUnits()
            }
            ResolverStyleFlag::ElementDependentRandomFunctions => {
                builder.HasElementDependentRandomFunctions()
            }
            ResolverStyleFlag::EnsuredInDisplayNone => builder.IsEnsuredInDisplayNone(),
            ResolverStyleFlag::SkipsContents => builder.SkipsContents(),
            ResolverStyleFlag::IsLink => builder.IsLink(),
            ResolverStyleFlag::HTMLInert => builder.IsHTMLInert(),
            ResolverStyleFlag::HTMLInertIsInherited => builder.IsHTMLInertIsInherited(),
            ResolverStyleFlag::CSSInert => builder.IsCSSInert(),
            ResolverStyleFlag::CSSInertIsInherited => builder.IsCSSInertIsInherited(),
            ResolverStyleFlag::NonUniversalHighlightPseudoStyles => {
                builder.HasNonUniversalHighlightPseudoStyles()
            }
            ResolverStyleFlag::NonUaHighlightPseudoStyles => {
                builder.HasNonUaHighlightPseudoStyles()
            }
            ResolverStyleFlag::HighlightsDependOnSizeContainerQueries => {
                builder.HighlightsDependOnSizeContainerQueries()
            }
            ResolverStyleFlag::AffectedByDrag => builder.AffectedByDrag(),
            ResolverStyleFlag::AffectedByFocusWithin => builder.AffectedByFocusWithin(),
            ResolverStyleFlag::AffectedByHover => builder.AffectedByHover(),
            ResolverStyleFlag::AffectedByActive => builder.AffectedByActive(),
            ResolverStyleFlag::StartingStyle => builder.IsStartingStyle(),
            ResolverStyleFlag::DependsOnSizeContainerQueries => {
                builder.DependsOnSizeContainerQueries()
            }
            ResolverStyleFlag::DependsOnStyleContainerQueries => {
                builder.DependsOnStyleContainerQueries()
            }
            ResolverStyleFlag::DependsOnScrollStateContainerQueries => {
                builder.DependsOnScrollStateContainerQueries()
            }
            ResolverStyleFlag::DependsOnAnchoredContainerQueries => {
                builder.DependsOnAnchoredContainerQueries()
            }
            ResolverStyleFlag::FirstLineDependsOnSizeContainerQueries => {
                builder.FirstLineDependsOnSizeContainerQueries()
            }
        }
    }
    fn SetBuilderFlag(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        flag: ResolverStyleFlag,
        value: bool,
    ) {
        match flag {
            ResolverStyleFlag::InlineStyleLostCascade => builder.SetInlineStyleLostCascade(value),
            ResolverStyleFlag::OriginalDisplayInlineType => {
                builder.SetIsOriginalDisplayInlineType(value)
            }
            ResolverStyleFlag::SkipsContents => builder.SetSkipsContents(value),
            ResolverStyleFlag::HTMLInert => builder.SetIsHTMLInert(value),
            ResolverStyleFlag::HTMLInertIsInherited => builder.SetIsHTMLInertIsInherited(value),
            ResolverStyleFlag::CSSInert => builder.SetIsCSSInert(value),
            ResolverStyleFlag::CSSInertIsInherited => builder.SetIsCSSInertIsInherited(value),
            ResolverStyleFlag::NonUniversalHighlightPseudoStyles => {
                builder.SetHasNonUniversalHighlightPseudoStyles(value)
            }
            ResolverStyleFlag::NonUaHighlightPseudoStyles => {
                builder.SetHasNonUaHighlightPseudoStyles(value)
            }
            ResolverStyleFlag::HighlightsDependOnSizeContainerQueries => {
                builder.SetHighlightsDependOnSizeContainerQueries(value)
            }
            ResolverStyleFlag::DependsOnSizeContainerQueries => {
                builder.SetDependsOnSizeContainerQueries(value)
            }
            ResolverStyleFlag::DependsOnStyleContainerQueries => {
                builder.SetDependsOnStyleContainerQueries(value)
            }
            ResolverStyleFlag::DependsOnScrollStateContainerQueries => {
                builder.SetDependsOnScrollStateContainerQueries(value)
            }
            ResolverStyleFlag::DependsOnAnchoredContainerQueries => {
                builder.SetDependsOnAnchoredContainerQueries(value)
            }
            ResolverStyleFlag::FirstLineDependsOnSizeContainerQueries => {
                builder.SetFirstLineDependsOnSizeContainerQueries(value)
            }
            ResolverStyleFlag::AnchorFunctions if value => builder.SetHasAnchorFunctions(),
            ResolverStyleFlag::AnchorEvaluator if value => builder.SetHasAnchorEvaluator(),
            ResolverStyleFlag::RootRelativeUnits if value => builder.SetHasRootRelativeUnits(),
            ResolverStyleFlag::GlyphRelativeUnits if value => builder.SetHasGlyphRelativeUnits(),
            ResolverStyleFlag::LineHeightRelativeUnits if value => {
                builder.SetHasLineHeightRelativeUnits()
            }
            ResolverStyleFlag::SiblingFunctions if value => builder.SetHasSiblingFunctions(),
            ResolverStyleFlag::ExplicitInheritance if value => builder.SetHasExplicitInheritance(),
            ResolverStyleFlag::CanAffectAnimations if value => builder.SetCanAffectAnimations(),
            ResolverStyleFlag::EmUnits if value => builder.SetHasEmUnits(),
            ResolverStyleFlag::StaticViewportUnits if value => builder.SetHasStaticViewportUnits(),
            ResolverStyleFlag::DynamicViewportUnits if value => {
                builder.SetHasDynamicViewportUnits()
            }
            ResolverStyleFlag::ContainerRelativeValue if value => {
                builder.SetHasContainerRelativeValue()
            }
            ResolverStyleFlag::LogicalDirectionRelativeUnits if value => {
                builder.SetHasLogicalDirectionRelativeUnits()
            }
            ResolverStyleFlag::ElementDependentRandomFunctions if value => {
                builder.SetHasElementDependentRandomFunctions()
            }
            ResolverStyleFlag::EnsuredInDisplayNone if value => builder.SetIsEnsuredInDisplayNone(),
            ResolverStyleFlag::IsLink if value => builder.SetIsLink(),
            ResolverStyleFlag::AffectedByDrag if value => builder.SetAffectedByDrag(),
            ResolverStyleFlag::AffectedByFocusWithin if value => builder.SetAffectedByFocusWithin(),
            ResolverStyleFlag::AffectedByHover if value => builder.SetAffectedByHover(),
            ResolverStyleFlag::AffectedByActive if value => builder.SetAffectedByActive(),
            ResolverStyleFlag::StartingStyle if value => builder.SetIsStartingStyle(),
            ResolverStyleFlag::FontRelativeUnits => {
                // This is derived from the em/root/glyph flags.
            }
            _ if !value => {}
            _ => self.unsupported("base:computed-style-flag"),
        }
    }
    fn BuilderDisplay(&self, builder: &Self::ComputedStyleBuilder) -> EDisplay {
        builder.Display()
    }
    fn StyleDisplay(&self, style: &Self::ComputedStyle) -> EDisplay {
        Self::style(style).Display()
    }
    fn IsDisplayInlineType(&self, display: EDisplay) -> bool {
        matches!(
            display,
            EDisplay::kInline
                | EDisplay::kInlineBlock
                | EDisplay::kInlineFlex
                | EDisplay::kInlineGrid
        )
    }
    fn StyleInsideLink(&self, style: &Self::ComputedStyle) -> EInsideLink {
        Self::style(style).InsideLink()
    }
    fn SetInsideLink(&self, builder: &mut Self::ComputedStyleBuilder, inside_link: EInsideLink) {
        builder.SetInsideLink(inside_link)
    }
    fn SetStyleType(&self, builder: &mut Self::ComputedStyleBuilder, pseudo_id: PseudoId) {
        builder.SetStyleType(pseudo_id)
    }
    fn BuilderStyleType(&self, builder: &Self::ComputedStyleBuilder) -> PseudoId {
        builder.StyleType()
    }
    fn CopyHighlightPropertiesFrom(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        originating: &Self::ComputedStyle,
    ) {
        builder.CopyHighlightPropertiesFrom(Self::style(originating));
    }
    fn BuilderPositionAnchorIsName(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        builder.PositionAnchor().IsName()
    }
    fn StylePositionAnchorIsName(&self, style: &Self::ComputedStyle) -> bool {
        Self::style(style).PositionAnchor().IsName()
    }
    fn BuilderHasPositionTryFallbacks(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        !builder.GetPositionTryFallbacks().Get().is_null()
    }
    fn StyleHasPositionTryFallbacks(&self, style: &Self::ComputedStyle) -> bool {
        !Self::style(style).GetPositionTryFallbacks().Get().is_null()
    }
    fn BuilderInteractivity(&self, builder: &Self::ComputedStyleBuilder) -> EInteractivity {
        builder.Interactivity()
    }
    fn BuilderInteractivityIsInherited(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        builder.InteractivityIsInherited()
    }
    fn BuilderCurrentColor(&self, builder: &Self::ComputedStyleBuilder) -> Self::CurrentColor {
        builder.GetCurrentColor()
    }
    fn BuilderViewportUnitFlags(&self, builder: &Self::ComputedStyleBuilder) -> u32 {
        builder.ViewportUnitFlags()
    }
    fn BuilderCloneStyle(&self, builder: &Self::ComputedStyleBuilder) -> Rc<Self::ComputedStyle> {
        Rc::new(Persistent::from_ptr(
            builder.CloneStyle() as *mut ComputedStyle
        ))
    }
    fn StyleBaseData(&self, style: &Self::ComputedStyle) -> Option<Rc<Self::StyleBaseData>> {
        let data = Self::style(style).BaseData();
        (!data.is_null()).then(|| Rc::new(Persistent::from_ptr(data)))
    }
    fn BuilderBaseData(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Option<Rc<Self::StyleBaseData>> {
        let data = builder.BaseData();
        (!data.is_null()).then(|| Rc::new(Persistent::from_ptr(data)))
    }
    fn SetBaseData(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        data: Option<Rc<Self::StyleBaseData>>,
    ) {
        let pointer = data
            .as_deref()
            .map_or(std::ptr::null_mut(), |data| data.Get());
        if let Some(data) = data {
            self.base_data_roots
                .borrow_mut()
                .push(data.as_ref().clone());
        }
        builder.SetBaseData(pointer)
    }
    fn BaseComputedStyle(&self, data: &Self::StyleBaseData) -> Option<Rc<Self::ComputedStyle>> {
        let style = unsafe { &*data.Get() }.GetBaseComputedStyle().cast_mut();
        (!style.is_null()).then(|| Rc::new(Persistent::from_ptr(style)))
    }
    fn BaseImportantSet(&self, data: &Self::StyleBaseData) -> Option<Rc<Self::CSSBitset>> {
        let important = unsafe { &*data.Get() }.GetBaseImportantSet();
        (!important.is_null()).then(|| Rc::new(unsafe { &*important }.clone()))
    }
    fn NewStyleBaseData(
        &self,
        style: Rc<Self::ComputedStyle>,
        important: Option<Rc<Self::CSSBitset>>,
    ) -> Rc<Self::StyleBaseData> {
        Rc::new(Persistent::from_ptr(NativeStyleBaseData::Create(
            style.Get(),
            important.map(|set| Box::new(set.as_ref().clone())),
        )))
    }
    fn BuilderBaseComputedStyle(
        &self,
        builder: &Self::ComputedStyleBuilder,
    ) -> Rc<Self::ComputedStyle> {
        let base = builder.GetBaseComputedStyle().cast_mut();
        if base.is_null() {
            self.BuilderCloneStyle(builder)
        } else {
            Rc::new(Persistent::from_ptr(base))
        }
    }
    fn SetVisitedPropertyFromUnvisited(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        property: ResolverVisitedProperty,
    ) {
        // cpp: style_resolver.cc:3348-3367. At the boundary where an
        // element first enters a visited link, initialize each inherited
        // visited slot from the already-computed unvisited value.
        match property {
            ResolverVisitedProperty::Color => {
                let value = builder.Color().clone();
                builder.SetInternalVisitedColorOwned(value)
            }
            ResolverVisitedProperty::CaretColor => {
                let value = builder.CaretColor().clone();
                builder.SetInternalVisitedCaretColorOwned(value)
            }
            ResolverVisitedProperty::FillPaint => {
                let value = builder.FillPaint().clone();
                builder.SetInternalVisitedFillPaintOwned(value)
            }
            ResolverVisitedProperty::StrokePaint => {
                let value = builder.StrokePaint().clone();
                builder.SetInternalVisitedStrokePaintOwned(value)
            }
            ResolverVisitedProperty::TextEmphasisColor => {
                let value = builder.TextEmphasisColor().clone();
                builder.SetInternalVisitedTextEmphasisColorOwned(value)
            }
            ResolverVisitedProperty::TextFillColor => {
                let value = builder.TextFillColor().clone();
                builder.SetInternalVisitedTextFillColorOwned(value)
            }
            ResolverVisitedProperty::TextStrokeColor => {
                let value = builder.TextStrokeColor().clone();
                builder.SetInternalVisitedTextStrokeColorOwned(value)
            }
        }
    }
    fn StyleUserModify(&self, style: &Self::ComputedStyle) -> EUserModify {
        Self::style(style).UserModify()
    }
    fn StyleEffectiveZoom(&self, style: &Self::ComputedStyle) -> f32 {
        Self::style(style).EffectiveZoom()
    }
    fn StyleInheritedVariablesHash(&self, style: &Self::ComputedStyle) -> u32 {
        // The native style retains other inherited fields. Variables belong
        // to the typed style-side owner and join the same source MPC key.
        self.cascade
            .parent_custom_properties
            .map_or(0, CustomProperties::CacheHash)
    }
    fn StyleInheritedBitFieldsHash(&self, style: &Self::ComputedStyle) -> u32 {
        Self::style(style).HashInheritedBitFields()
    }
    fn StyleFontComputedSize(&self, style: &Self::ComputedStyle) -> f32 {
        Self::style(style).GetFontDescription().ComputedSize()
    }
    fn StyleSetChildHasExplicitInheritance(&self, style: &Self::ComputedStyle) {
        Self::style(style).SetChildHasExplicitInheritance()
    }
    fn PropertySetProperties<'p>(
        &self,
        set: &'p Self::CSSPropertyValueSet,
    ) -> &'p [Self::CSSPropertyValue] {
        match set.as_ref() {
            CSSPropertyValueSetRuleHandle::Immutable(set) => set.Properties(),
            CSSPropertyValueSetRuleHandle::Mutable(_) => {
                // Incremental application needs a borrow guard whose lifetime
                // spans the source loop. The production resolver freezes every
                // parsed inline declaration before it reaches this path, so a
                // mutable handle is a distinct, unsupported external owner.
                self.unsupported("base:mutable-property-set-properties");
                &[]
            }
        }
    }
    fn PropertyID(&self, property: &Self::CSSPropertyValue) -> CSSPropertyID {
        property.PropertyID()
    }
    fn PropertyName(
        &self,
        property: &Self::CSSPropertyValue,
    ) -> crate::css_property_name::CSSPropertyName {
        property.Name()
    }
    fn PropertyValue<'p>(&self, property: &'p Self::CSSPropertyValue) -> &'p Self::CSSValue {
        property.ValueHandle()
    }
    fn ValueIsUnparsedDeclaration(&self, value: &Self::CSSValue) -> bool {
        value.IsUnparsedDeclaration()
    }
    fn ValueIsPendingSubstitution(&self, value: &Self::CSSValue) -> bool {
        value.IsPendingSubstitutionValue()
    }
    fn ValueIsCascadeDependentKeyword(&self, value: &Self::CSSValue) -> bool {
        value.IsCascadeDependentKeyword()
    }
    fn ValueIsMathFunction(&self, value: &Self::CSSValue) -> bool {
        value.IsMathFunctionValue()
    }
    fn MathValueHasAnchorFunctions(&self, value: &Self::CSSValue) -> bool {
        // The fixed-commit production parser has no CSSMathExpressionNode
        // variant for anchor()/anchor-size(). Such syntax is rejected before
        // a math CSSValue reaches StyleResolver, matching the cascade backend.
        false
    }
    fn TakeLengthConversionFlags(
        &self,
        state: &mut Self::StyleResolverState,
    ) -> Self::LengthConversionFlags {
        std::mem::take(&mut state.length_conversion_flags)
    }
    fn LengthFlagsEmpty(&self, flags: &Self::LengthConversionFlags) -> bool {
        *flags == 0
    }
    fn HasLengthFlag(&self, flags: &Self::LengthConversionFlags, flag: ResolverLengthFlag) -> bool {
        let bit = match flag {
            ResolverLengthFlag::Em => 1 << 0,
            ResolverLengthFlag::RootFontRelative => 1 << 1,
            ResolverLengthFlag::GlyphRelative => 1 << 2,
            ResolverLengthFlag::Viewport => 1 << 3,
            ResolverLengthFlag::SmallLargeViewport => 1 << 4,
            ResolverLengthFlag::DynamicViewport => 1 << 5,
            ResolverLengthFlag::ContainerRelative => 1 << 6,
            ResolverLengthFlag::TreeScopedReference => 1 << 7,
            ResolverLengthFlag::LogicalDirectionRelative => 1 << 8,
            ResolverLengthFlag::AnchorRelative => 1 << 9,
            ResolverLengthFlag::LhRelative => 1 << 14,
            ResolverLengthFlag::RlhRelative => 1 << 15,
            ResolverLengthFlag::SiblingRelative => 1 << 19,
            ResolverLengthFlag::ElementDependentRandom => 1 << 20,
        };
        flags & bit != 0
    }
    fn MathMLKind(&self, element: &Self::Element) -> ResolverMathMLKind {
        if self.document.Node(element.index).Namespace() != DOMNamespace::kMathML {
            return ResolverMathMLKind::Other;
        }
        match self.document.Node(element.index).Name() {
            "mspace" => ResolverMathMLKind::Space,
            "mpadded" => ResolverMathMLKind::Padded,
            "mfrac" => ResolverMathMLKind::Fraction,
            "mo" => ResolverMathMLKind::Operator,
            _ => ResolverMathMLKind::Other,
        }
    }
    fn StateAnchorEvaluator(
        &self,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::AnchorEvaluator>> {
        None
    }
    fn ComputeAnchorCenterOffsets(
        &self,
        evaluator: &Self::AnchorEvaluator,
        builder: &Self::ComputedStyleBuilder,
    ) -> Option<PhysicalOffset> {
        self.unsupported("base:ComputeAnchorCenterOffsets");
        Default::default()
    }
    fn SetAnchorCenterOffset(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        offset: Option<PhysicalOffset>,
    ) {
        builder.SetAnchorCenterOffsetOwned(offset)
    }
    fn EnsureUAStyleForElement(&self, engine: &Self::StyleEngine, element: &Self::Element) {
        // The caller supplied the typed UA RuleSet owner.
    }
    fn EnsureUAStyleForPseudoElement(&self, engine: &Self::StyleEngine, pseudo_id: PseudoId) {
        if !matches!(
            pseudo_id,
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
            self.unsupported("base:EnsureUAStyleForPseudoElement");
        }
    }
    fn NewCollectorForCascade(
        &self,
        state: &Self::StyleResolverState,
        context: &Self::StyleRecalcContext,
        selector_filter: &mut Self::SelectorFilter,
        cascade: &mut Self::StyleCascade,
    ) -> Self::ElementRuleCollector {
        if self.cascade.incremental_style.is_some() && !state.pseudo {
            self.selectors
                .borrow()
                .ResetSubjectMatchFlags(state.element.index);
        }
        ProductionResolverCollector {
            document: self.document,
            element: state.element.clone(),
            selectors: self.selectors.clone(),
            result: cascade.result.clone(),
            pending_inputs: Vec::new(),
            pending_passes: Vec::new(),
            scope: None,
            matching_ua: false,
            pseudo: ProductionPseudoRequest::Element(),
            recalc: context.clone(),
            inside_link: self.StateInsideLink(state),
        }
    }
    fn SetPseudoElementStyleRequest(
        &self,
        collector: &mut Self::ElementRuleCollector,
        request: &Self::StyleRequest,
    ) {
        collector.pseudo = request.pseudo.clone()
    }
    fn CascadeMatchResult(&self, cascade: &Self::StyleCascade) -> Rc<Self::MatchResult> {
        Rc::new(cascade.result.borrow().clone())
    }
    fn NewMatchResult(&self) -> Self::MatchResult {
        self.unsupported("base:NewMatchResult");
        Default::default()
    }
    fn NewCollectorForResult(
        &self,
        state: &Self::StyleResolverState,
        context: &Self::StyleRecalcContext,
        selector_filter: &mut Self::SelectorFilter,
        result: &mut Self::MatchResult,
    ) -> Self::ElementRuleCollector {
        self.unsupported("base:new-collector-for-external-result");
        ProductionResolverCollector {
            document: self.document,
            element: state.element.clone(),
            selectors: self.selectors.clone(),
            result: Rc::new(RefCell::new(result.clone())),
            pending_inputs: Vec::new(),
            pending_passes: Vec::new(),
            scope: None,
            matching_ua: false,
            pseudo: ProductionPseudoRequest::Element(),
            recalc: context.clone(),
            inside_link: self.StateInsideLink(state),
        }
    }
    fn AddCascadeMatchedProperties(
        &self,
        cascade: &mut Self::StyleCascade,
        properties: &Self::CSSPropertyValueSet,
        origin: CascadeOrigin,
    ) {
        let mut data = crate::resolver::match_result::MatchedPropertiesData::default();
        data.origin = origin;
        cascade
            .result
            .borrow_mut()
            .AddMatchedProperties(properties.clone(), None, data);
    }
    fn NewInitialColorValue(&self) -> Self::CSSValue {
        production_css_value::initial_color()
    }
    fn NewInheritedValue(&self) -> Self::CSSValue {
        production_css_value::wide(CSSValueID::kInherit).expect("inherit is a production CSSValue")
    }
    fn NewIdentifierValue(&self, value: CSSValueID) -> Self::CSSValue {
        production_css_value::identifier(value)
    }
    fn SetProperty(
        &self,
        set: &Self::CSSPropertyValueSet,
        property: CSSPropertyID,
        value: &Self::CSSValue,
        important: bool,
    ) {
        let CSSPropertyValueSetRuleHandle::Mutable(set) = set.as_ref() else {
            self.unsupported("base:immutable-declaration-cache");
            return;
        };
        set.borrow_mut()
            .SetProperty(property, value.clone(), important);
    }
    fn MatchResultHasMatchedProperties(&self, result: &Self::MatchResult) -> bool {
        !result.GetMatchedProperties().is_empty()
    }
    fn MatchResultIsCacheable(&self, result: &Self::MatchResult) -> bool {
        result.IsCacheable()
    }
    fn MatchResultFlag(&self, result: &Self::MatchResult, flag: ResolverMatchResultFlag) -> bool {
        match flag {
            ResolverMatchResultFlag::NonUniversalHighlightPseudoStyles => {
                result.HasNonUniversalHighlightPseudoStyles()
            }
            ResolverMatchResultFlag::NonUaHighlightPseudoStyles => {
                result.HasNonUaHighlightPseudoStyles()
            }
            ResolverMatchResultFlag::HighlightsDependOnSizeContainerQueries => {
                result.HighlightsDependOnSizeContainerQueries()
            }
            ResolverMatchResultFlag::SizeContainerQueries => result.DependsOnSizeContainerQueries(),
            ResolverMatchResultFlag::StyleContainerQueries => {
                result.DependsOnStyleContainerQueries()
            }
            ResolverMatchResultFlag::ScrollStateContainerQueries => {
                result.DependsOnScrollStateContainerQueries()
            }
            ResolverMatchResultFlag::AnchoredContainerQueries => {
                result.DependsOnAnchoredContainerQueries()
            }
            ResolverMatchResultFlag::FirstLineSizeContainerQueries => {
                result.FirstLineDependsOnSizeContainerQueries()
            }
            ResolverMatchResultFlag::StaticViewportUnits => result.DependsOnStaticViewportUnits(),
            ResolverMatchResultFlag::DynamicViewportUnits => result.DependsOnDynamicViewportUnits(),
            ResolverMatchResultFlag::RootUnitContainerQueries => {
                result.DependsOnRootUnitContainerQueries()
            }
            ResolverMatchResultFlag::ConditionallyAffectsAnimations => {
                result.ConditionallyAffectsAnimations()
            }
        }
    }
    fn MatchResultHasFlag(&self, result: &Self::MatchResult, flag: MatchFlag) -> bool {
        result.HasFlag(flag)
    }
    fn CustomHighlightNames<'r>(
        &self,
        result: &'r Self::MatchResult,
    ) -> &'r Self::CustomHighlightNames {
        result.CustomHighlightNames()
    }
    fn CustomHighlightNamesEmpty(&self, names: &Self::CustomHighlightNames) -> bool {
        names.is_empty()
    }
    fn SetCustomHighlightNames(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        names: &Self::CustomHighlightNames,
    ) {
        let mut retained_names = foundation::HashSet::new();
        for name in names {
            retained_names.insert(name.clone());
        }
        builder.SetCustomHighlightNames(&retained_names);
    }
    fn MatchResultPseudoElementBits(&self, result: &Self::MatchResult) -> u32 {
        result.PseudoElementStyles().Bits()
    }
    fn SetPseudoElementStyles(&self, builder: &mut Self::ComputedStyleBuilder, bits: u32) {
        builder.SetPseudoElementStyles(bits)
    }
    fn CascadeApply(&self, cascade: &mut Self::StyleCascade, filter: CascadeFilter) {
        let _trace = browser_tracing::span("style", "StyleResolver.ApplyCascade");
        let Some(mut state_ptr) = cascade.state else {
            self.unsupported("base:missing-cascade-state");
            return;
        };
        // NewStyleCascade records this pointer and ApplyBaseStyleNoCache keeps
        // the state alive and exclusively borrowed for the complete call.
        let state = unsafe { state_ptr.as_mut() };
        let retained_base_data = (!state.builder.BaseData().is_null())
            .then(|| Persistent::from_ptr(state.builder.BaseData()));
        let base = Persistent::from_ptr(state.builder.CloneStyle().cast_mut());
        let parent = state.parent.as_deref().cloned();
        let current_custom = self.cascade_custom_output.borrow().clone();
        match ApplyMatchResultWithFilter(
            ProductionCascadeInputs {
                document: Some(self.document),
                media: self.initial.media.clone(),
                base,
                parent,
                root_style: self.cascade.root_style.clone(),
                match_result: cascade.result.borrow().clone(),
                root_font_size: self.cascade.root_font_size,
                color_scheme_settings: self.cascade.color_scheme_settings,
                system_colors: self.cascade.system_colors,
                text_link_text_color: self.cascade.text_link_text_color,
                is_document_element: self.cascade.is_document_element,
                pseudo_id: self.cascade.pseudo_id,
                originating_style: self.cascade.originating_style.clone(),
                subject_node: self.cascade.subject_node,
                container_runtime: self.cascade.container_runtime,
                custom_properties: current_custom.as_deref().or(self.cascade.custom_properties),
                parent_custom_properties: self.cascade.parent_custom_properties,
                substitution_context: self.cascade.substitution_context,
                image_resolver: self.cascade.image_resolver,
                property_registry: self.cascade.property_registry,
                function_registry: self.cascade.function_registry,
                animations: cascade.animations.as_ref(),
                transitions: cascade.transitions.as_ref(),
                font_metrics: self.cascade.font_metrics,
            },
            filter,
        ) {
            Ok(output) => {
                state.builder = ComputedStyleBuilder::from_style(Self::style(&output.style));
                if let Some(base_data) = retained_base_data {
                    // The inner production cascade rebuilds its state from the
                    // current style snapshot. Chromium's outer StyleCascade
                    // mutates the existing builder, whose StyleBaseData must
                    // survive the interpolation pass unchanged.
                    state.builder.SetBaseData(base_data.Get());
                }
                *self.cascade_custom_output.borrow_mut() = Some(output.custom_properties);
                cascade.important_set = output.important_set.map(Rc::new);
                cascade.inline_style_lost = output.inline_style_lost;
                state.rejected_legacy_overlapping = output.rejected_legacy_overlapping;
                state.length_conversion_flags |= output.length_conversion_flags;
                let mut metrics = self.base_style_metrics.borrow_mut();
                metrics.matched_property_sets = output.matched_property_sets;
                metrics.registry_dependent = output.registry_dependent;
                metrics.has_animations = output.has_animations;
                metrics.cascade_applied = true;
                // ApplyBaseStyle still mutates the builder after the
                // cascade (pseudo bits, link state and resolver flags).  Build
                // the published style from that final builder in
                // StateTakeStyle instead of caching this intermediate value.
                state.output = None;
            }
            Err(cascade_fallbacks) => {
                let mut fallbacks = self.fallbacks.borrow_mut();
                for fallback in cascade_fallbacks {
                    let fallback = ProductionRuleMatchingFallback::Cascade(fallback);
                    if !fallbacks.contains(&fallback) {
                        fallbacks.push(fallback);
                    }
                }
            }
        }
    }
    fn CascadeInlineStyleLost(&self, cascade: &Self::StyleCascade) -> bool {
        cascade.inline_style_lost
    }
    fn CascadeReleaseImportantSet(
        &self,
        cascade: &mut Self::StyleCascade,
    ) -> Option<Rc<Self::CSSBitset>> {
        cascade.important_set.take()
    }
    fn CascadeAddInterpolations(
        &self,
        cascade: &mut Self::StyleCascade,
        interpolations: &Self::ActiveInterpolationsMap,
        origin: CascadeOrigin,
    ) {
        match origin {
            CascadeOrigin::kAnimation => cascade.animations = Some(interpolations.clone()),
            CascadeOrigin::kTransition => cascade.transitions = Some(interpolations.clone()),
            _ => self.unsupported("base:interpolation-origin"),
        }
    }
    fn AdjustComputedStyle(
        &self,
        state: &mut Self::StyleResolverState,
        element_if_not_pseudo: Option<&Self::Element>,
    ) {
        let Some(parent) = state.layout_parent.as_deref() else {
            self.unsupported("base:missing-layout-parent-for-adjustment");
            return;
        };
        AdjustComputedDisplayForLayout(
            &mut state.builder,
            Self::style(parent),
            &DisplayAdjustmentContext {
                is_root: self.cascade.is_document_element,
                ..Default::default()
            },
        );
        // style_adjuster.cc:1252,1261-1282: animated recascades start from
        // an adjusted base. Recompute forcing reasons instead of retaining a
        // former explicit z-index when the sampled value becomes auto.
        state.builder.SetForcesStackingContext(false);
        super::style_adjuster::AdjustStyleForZIndex(&mut state.builder, Self::style(parent));
        if self.cascade.is_document_element
            || matches!(
                state.builder.StyleType(),
                PseudoId::kPseudoIdBackdrop | PseudoId::kPseudoIdViewTransition
            )
            || element_if_not_pseudo.is_some_and(|element| {
                self.document.IsInTopLayer(element.index)
                    || self
                        .document
                        .Node(element.index)
                        .IsElement("foreignObject", DOMNamespace::kSVG)
            })
        {
            state.builder.SetForcesStackingContext(true);
        }
        // Chromium style_adjuster.cc:1314-1321: finalize each image layer's
        // shorter longhand lists before forced colors and style publication.
        // Keep repetition/culling in the canonical FillLayer owner; the
        // downstream geometry projection must consume the finalized values.
        state.builder.AdjustBackgroundLayers();
        state.builder.AdjustMaskLayers();
        if AdjustProductionStyleForForcedColors(
            &mut state.builder,
            self.initial.media,
            self.cascade.system_colors,
        )
        .is_err()
        {
            self.unsupported("forced-colors-system-color-owner");
        }
    }
    fn RunUncacheableStyleAdjustment(
        &self,
        state: &mut Self::StyleResolverState,
        element: &Self::Element,
        pseudo_or_element: Option<&Self::Element>,
        styled_element: Option<&Self::Element>,
    ) {
        // cpp: style_adjuster.cc:1423-1458. Pseudos without a concrete
        // element have no theme owner and source sets effective appearance to
        // none. Ordinary form controls use their retained DOM type owner.
        if state.builder.Appearance() == AppearanceValue::kNone || pseudo_or_element.is_none() {
            state.builder.SetEffectiveAppearance(AppearanceValue::kNone);
            return;
        }
        if ApplyProductionThemeAppearance(self.document, element.index, &mut state.builder).is_err()
        {
            self.unsupported("appearance-platform-owner");
        }
    }
    fn NewCacheKey(&self, result: &Self::MatchResult, inherited_hash: u32) -> Self::CacheKey {
        MatchedPropertiesCacheKey::new(
            result,
            AdditionalHash {
                hash: inherited_hash,
            },
        )
    }
    fn CacheKeyIsCacheable(&self, key: &Self::CacheKey) -> bool {
        key.IsCacheable()
    }
    fn GetElementTypeCacheKey(
        &self,
        layout_parent: &Self::ComputedStyle,
        element: &Self::Element,
    ) -> Self::ElementTypeForCache {
        // This adapter currently performs only the shared display adjustment
        // after insertion. Store the pre-adjusted style, exactly matching the
        // source's kIsNotElement cache mode, then rerun that adjustment on hit.
        StyleAdjusterElementTypeForCache {
            element_type: ElementType::kIsNotElement,
        }
    }
    fn UnadjustedElementType(&self) -> Self::ElementTypeForCache {
        StyleAdjusterElementTypeForCache {
            element_type: ElementType::kIsNotElement,
        }
    }
    fn CacheEntryElementType<'e>(
        &self,
        entry: &'e Self::CacheEntry,
    ) -> &'e Self::ElementTypeForCache {
        &entry.element_type
    }
    fn ElementTypeIsStyleAdjusted(&self, element_type: &Self::ElementTypeForCache) -> bool {
        element_type.CacheEntryIsStyleAdjusted()
    }
    fn CacheEntryStyle(&self, entry: &Self::CacheEntry) -> Rc<Self::ComputedStyle> {
        Rc::new(entry.computed_style.clone())
    }
    fn FindMatchedProperties(
        &self,
        cache: &mut Self::MatchedPropertiesCache,
        key: &Self::CacheKey,
        element_type: &Self::ElementTypeForCache,
        style_type: PseudoId,
        state: &Self::StyleResolverState,
    ) -> Option<Rc<Self::CacheEntry>> {
        // Animation samples are currently applied by ProductionStyleCascade
        // during the base pass. Such a result is a final animated style, not
        // Chromium's reusable animation base, and must never enter or hit MPC.
        if self.cascade.animations.is_some()
            || self.cascade.transitions.is_some()
            || self.StateIsForHighlight(state)
        {
            return None;
        }
        let empty = CustomProperties::default();
        let parent_custom = self.cascade.parent_custom_properties.unwrap_or(&empty);
        if !parent_custom.diagnostics.is_empty() {
            return None;
        }
        let parent = state.parent.as_deref()?;
        let layout_parent = state.layout_parent.as_deref()?;
        let entry = cache.0.borrow_mut().FindForNonHighlightWithStylesAndData(
            key,
            element_type,
            style_type,
            parent,
            layout_parent,
            self.IsAtShadowBoundary(&state.element),
            StyleAdjusterElementTypeForCache::CacheEntryIsStyleAdjusted,
            |data| data.parent.CacheEquivalent(parent_custom),
            // kIsNotElement entries are stored before style adjustment, so
            // source never consults StyleAdjuster::IsCacheCompatible here.
            |_, _, _, _| true,
        );
        if let Some(entry) = &entry {
            self.base_style_metrics
                .borrow_mut()
                .matched_property_cache_hits += 1;
            *self.cascade_custom_output.borrow_mut() = Some(entry.additional_data.output.clone());
        }
        entry.map(Rc::new)
    }
    fn MatchedPropertiesIsCacheable(&self, state: &Self::StyleResolverState) -> bool {
        if self.cascade.animations.is_some() || self.cascade.transitions.is_some() {
            return false;
        }
        // cpp: StyleResolverState::GetSVGResource sets HasTreeScopedReference.
        // The production resource boundary installs the canonical reference
        // directly; exclude that result here as well so MPC cannot transplant
        // an operation (including an inherited one) into a different scope.
        let has_svg_reference = state.builder.MutableClipPath().is_some_and(|operation| {
            unsafe { operation.as_ref() }.is_some_and(|operation| {
                operation.GetType()
                    == layoutng_style::style::clip_path_operation::OperationType::kReference
            })
        });
        if !MatchedPropertiesCache::<StyleAdjusterElementTypeForCache>::IsStyleCacheable(
            &state.builder,
        ) || state.has_tree_scoped_reference
            || has_svg_reference
        {
            return false;
        }
        let Some(parent) = state.parent.as_deref() else {
            return false;
        };
        if self.IsAtShadowBoundary(&state.element)
            && state.builder.UserModify() != Self::style(parent).UserModify()
        {
            return false;
        }
        if !self.GetCascadeFilter(&state.element).IsEmpty() {
            return false;
        }
        let custom_is_cacheable = self
            .cascade_custom_output
            .borrow()
            .as_ref()
            .is_some_and(|custom| custom.diagnostics.is_empty());
        custom_is_cacheable
            && self
                .cascade
                .substitution_context
                .attributes
                .is_none_or(|attributes| !attributes.HasAttrFunction())
            && self
                .cascade
                .substitution_context
                .environment
                .is_none_or(|environment| !environment.HasEnvironmentVariableReference())
    }
    fn AddMatchedProperties(
        &self,
        cache: &mut Self::MatchedPropertiesCache,
        key: &Self::CacheKey,
        element_type: &Self::ElementTypeForCache,
        style: Rc<Self::ComputedStyle>,
        parent: Option<Rc<Self::ComputedStyle>>,
        layout_parent: Option<Rc<Self::ComputedStyle>>,
        originating: Option<Rc<Self::ComputedStyle>>,
    ) {
        let (Some(parent), Some(layout_parent)) = (parent, layout_parent) else {
            return;
        };
        cache.0.borrow_mut().AddOwnedWithData(
            key,
            *element_type,
            style.as_ref().clone(),
            parent.as_ref().clone(),
            layout_parent.as_ref().clone(),
            originating.map(|style| style.as_ref().clone()),
            ProductionCachedVariables {
                parent: self
                    .cascade
                    .parent_custom_properties
                    .cloned()
                    .unwrap_or_default(),
                output: self
                    .cascade_custom_output
                    .borrow()
                    .as_ref()
                    .expect("cacheable cascade has custom output")
                    .clone(),
            },
        );
        self.base_style_metrics
            .borrow_mut()
            .matched_property_cache_adds += 1;
    }
    fn WatchedSelectorsRuleSet(&self, engine: &Self::StyleEngine) -> Option<Rc<Self::RuleSet>> {
        None
    }
    fn DocumentRulesSelectorsRuleSet(
        &self,
        engine: &Self::StyleEngine,
    ) -> Option<Rc<Self::RuleSet>> {
        // StyleEngine::DocumentRulesSelectorsRuleSet is nullable in Blink.
        // This owner has no document-rules selector registrations, so the
        // exact state is an absent optional RuleSet rather than a resolver
        // failure.
        None
    }
    fn SetMatchingRulesFromNoStyleSheet(
        &self,
        collector: &mut Self::ElementRuleCollector,
        value: bool,
    ) {
        self.unsupported("base:SetMatchingRulesFromNoStyleSheet");
        Default::default()
    }
    fn SetCollectingStyleRulesMode(&self, collector: &mut Self::ElementRuleCollector) {
        self.unsupported("base:SetCollectingStyleRulesMode");
        Default::default()
    }
    fn CompactRulesIfNeeded(&self, rules: &Self::RuleSet) {
        self.unsupported("base:CompactRulesIfNeeded");
        Default::default()
    }
    fn RuleSetGroupAddRuleSetBorrowed(
        &self,
        group: &mut Self::RuleSetGroup,
        rules: &Self::RuleSet,
    ) {
        self.unsupported("base:RuleSetGroupAddRuleSetBorrowed");
        Default::default()
    }
    fn CollectMatchingRulesInContainer(
        &self,
        collector: &mut Self::ElementRuleCollector,
        group: &Self::RuleSetGroup,
        scope: Option<&Self::ContainerNode>,
    ) {
        self.unsupported("base:CollectMatchingRulesInContainer");
        Default::default()
    }
    fn MatchedStyleRuleList(
        &self,
        collector: &Self::ElementRuleCollector,
    ) -> Option<Vec<Rc<Self::StyleRule>>> {
        self.unsupported("base:MatchedStyleRuleList");
        Default::default()
    }
    fn SelectorsText(&self, rule: &Self::StyleRule) -> foundation::String {
        self.unsupported("base:SelectorsText");
        Default::default()
    }
    fn AddCallbackSelector(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        selector: foundation::String,
    ) {
        self.unsupported("base:AddCallbackSelector");
        Default::default()
    }
    fn AddDocumentRulesSelector(
        &self,
        builder: &mut Self::ComputedStyleBuilder,
        rule: &Rc<Self::StyleRule>,
    ) {
        self.unsupported("base:AddDocumentRulesSelector");
        Default::default()
    }
    fn ElementAnimations(&self, element: &Self::Element) -> Option<Rc<Self::ElementAnimations>> {
        let animations = ProductionElementAnimations {
            has_animations: self.cascade.animations.is_some_and(|map| !map.IsEmpty()),
            has_transitions: self.cascade.transitions.is_some_and(|map| !map.IsEmpty()),
        };
        (self.cascade.subject_node == Some(element.index)
            && (animations.has_animations || animations.has_transitions))
            .then(|| Rc::new(animations))
    }
    fn ElementHasAnimations(&self, element: &Self::Element) -> bool {
        self.ElementAnimations(element).is_some()
    }
    fn ElementAnimationsIsAnimationStyleChange(
        &self,
        animations: &Self::ElementAnimations,
    ) -> bool {
        false
    }
    fn AnimatingProperty(
        &self,
        animations: &Self::ElementAnimations,
        property: ResolverAnimationProperty,
    ) -> bool {
        false
    }
    fn IsAnimatingStandardPropertiesAtDefaultPriority(
        &self,
        animations: &Self::ElementAnimations,
        important: Option<&Self::CSSBitset>,
    ) -> bool {
        false
    }
    fn BuilderHasAnimations(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        !builder.Animations().Get().is_null()
    }
    fn BuilderHasTransitions(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        !builder.Transitions().Get().is_null()
    }
    fn ScrollTimelineNameEmpty(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        builder.ScrollTimelineName().is_empty()
    }
    fn ViewTimelineNameEmpty(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        builder.ViewTimelineName().is_empty()
    }
    fn TimelineScopeIsNone(&self, builder: &Self::ComputedStyleBuilder) -> bool {
        builder.TimelineScope().IsNone()
    }
    fn CSSAnimationsHasTimelines(&self, animations: &Self::ElementAnimations) -> bool {
        false
    }
    fn AnimationUpdateEmpty(&self, state: &Self::StyleResolverState) -> bool {
        true
    }
    fn HasPreviousActiveInterpolationsForAnimations(
        &self,
        animations: &Self::ElementAnimations,
    ) -> bool {
        false
    }
    fn CurrentAnimationData(&self) -> Option<Rc<Self::AnimationData>> {
        None
    }
    fn StoreOldStyleIfNeeded(&self, data: &Self::AnimationData, element: &Self::Element) {
        self.unsupported("base:StoreOldStyleIfNeeded");
        Default::default()
    }
    fn SetPendingAnimationUpdate(
        &self,
        data: &Self::AnimationData,
        element: &Self::Element,
        state: &Self::StyleResolverState,
    ) {
        self.unsupported("base:SetPendingAnimationUpdate");
        Default::default()
    }
    fn CalculateAnimationUpdate(
        &self,
        state: &mut Self::StyleResolverState,
        animating: &Self::Element,
        originating: &Self::Element,
        resolver: &StyleResolver<'_, Self>,
        can_trigger: bool,
    ) {
        // `Document::AnimationStyles` is already the sampled animation update
        // produced by the document owner. There is no second update to
        // calculate here.
    }
    fn CalculateTransitionUpdate(
        &self,
        state: &mut Self::StyleResolverState,
        animating: &Self::Element,
        context: &Self::StyleRecalcContext,
        can_trigger: bool,
    ) {
        // `Document::TransitionStyles` is the separate retained transition
        // bucket, matching CSSAnimationUpdate's two ActiveInterpolationsMaps.
        // The producer has already sampled it; this host only publishes it.
    }
    fn AnimationUpdateHasActiveInterpolations(&self, state: &Self::StyleResolverState) -> bool {
        self.cascade.animations.is_some_and(|map| !map.IsEmpty())
            || self.cascade.transitions.is_some_and(|map| !map.IsEmpty())
    }
    fn ActiveInterpolationsForAnimations(
        &self,
        state: &Self::StyleResolverState,
    ) -> Rc<Self::ActiveInterpolationsMap> {
        Rc::new(self.cascade.animations.cloned().unwrap_or_default())
    }
    fn ActiveInterpolationsForTransitions(
        &self,
        state: &Self::StyleResolverState,
    ) -> Rc<Self::ActiveInterpolationsMap> {
        Rc::new(self.cascade.transitions.cloned().unwrap_or_default())
    }
    fn IsHighlightPseudoElement(&self, pseudo_id: PseudoId) -> bool {
        layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(pseudo_id)
    }
    fn CancelCompositedAnimationsAffectingProperties(
        &self,
        animations: &Self::ElementAnimations,
        important: &Self::CSSBitset,
    ) {
        // Sampled DOM declarations have no compositor animation owner.
    }
    fn CalculateCompositorAnimationUpdate(
        &self,
        state: &mut Self::StyleResolverState,
        animating: &Self::Element,
        originating: &Self::Element,
        base_style: &Self::ComputedStyle,
        parent: Option<&Self::ComputedStyle>,
        viewport_resized: bool,
        affects_snapshots: bool,
    ) {
        // The retained sample owner has no compositor update payload.
    }
    fn SnapshotCompositorKeyframes(
        &self,
        animating: &Self::Element,
        state: &Self::StyleResolverState,
        base_style: &Self::ComputedStyle,
        parent: Option<&Self::ComputedStyle>,
    ) {
        // The retained sample owner has no compositor keyframe payload.
    }
    fn UpdateAnimationFlags(
        &self,
        animating: &Self::Element,
        state: &mut Self::StyleResolverState,
    ) {
        // Current-animation compositor flags require the missing compositor
        // effect owner; sampled declarations do not imply those flags.
    }

    fn ApplyScopedProperty(
        &self,
        name: crate::css_property_name::CSSPropertyName,
        state: &mut Self::StyleResolverState,
        value: &Self::CSSValue,
        _scope: &Self::TreeScope,
    ) {
        state.incremental_properties.push((name, value.clone()));
    }
    fn AddMathPropertyIfNeeded(
        &self,
        element: &Self::Element,
        state: &mut Self::StyleResolverState,
        property: ResolverMathMLProperty,
    ) {
        use ResolverMathMLProperty::*;
        let (attribute, allow_percentages, nonnegative) = match property {
            Baseline => ("height", false, true),
            PaddedDepth => ("depth", false, true),
            PaddedLSpace => ("lspace", false, true),
            PaddedVOffset => ("voffset", false, false),
            FractionBarThickness => ("linethickness", true, false),
            LSpace => ("lspace", true, false),
            RSpace => ("rspace", true, false),
            MinSize => ("minsize", true, false),
            MaxSize => ("maxsize", true, false),
        };
        let Some(length) =
            self.MathMLAttributeLength(element, attribute, allow_percentages, nonnegative, state)
        else {
            return;
        };
        match property {
            Baseline => state.builder.SetMathBaselineOwned(length),
            PaddedDepth => state.builder.SetMathPaddedDepthOwned(length),
            PaddedLSpace | LSpace => state.builder.SetMathLSpaceOwned(length),
            PaddedVOffset => state.builder.SetMathPaddedVOffsetOwned(length),
            FractionBarThickness => state.builder.SetMathFractionBarThicknessOwned(length),
            RSpace => state.builder.SetMathRSpaceOwned(length),
            MinSize => state.builder.SetMathMinSizeOwned(length),
            MaxSize => state.builder.SetMathMaxSizeOwned(length),
        }
    }
    fn CalculateTimelineUpdate(
        &self,
        _state: &mut Self::StyleResolverState,
        _animating: &Self::Element,
    ) {
        self.unsupported("base:CalculateTimelineUpdate");
    }
}

/// Package-private owner bundle for the production `MatchAllRules` slice.
/// Rule sets stay parser-owned and the live DOM stays borrowed for the whole
/// translated matching operation.
pub(crate) struct ProductionRuleMatchingInputs<'a> {
    pub document: &'a Document,
    pub element_index: usize,
    pub media: &'a MediaValuesCachedData,
    pub html_user_agent_rules: ProductionResolverRuleSetInput,
    pub svg_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    pub mathml_user_agent_rules: Option<ProductionResolverRuleSetInput>,
    pub author_rule_sets: Rc<Vec<ProductionResolverRuleSetInput>>,
}

pub(crate) struct ProductionBaseStyleInputs<'a> {
    pub matching: ProductionRuleMatchingInputs<'a>,
    pub parent: Option<Persistent<ComputedStyle>>,
    pub collaborators: ProductionBaseStyleCollaborators<'a>,
}

pub(crate) struct ProductionResolveBaseStyleInputs<'a> {
    pub matching: ProductionRuleMatchingInputs<'a>,
    pub parent: Persistent<ComputedStyle>,
    pub layout_parent: Persistent<ComputedStyle>,
    pub collaborators: ProductionBaseStyleCollaborators<'a>,
}

/// Result of one translated MatchAllRules -> StyleCascade ->
/// ApplyBaseStyleNoCache operation. Custom properties are returned beside the
/// native style because the public DocumentStyleEngine owns their inheritance
/// side table.
pub(crate) struct ProductionBaseStyleOutput {
    pub style: Rc<Persistent<ComputedStyle>>,
    pub custom_properties: Rc<CustomProperties>,
    pub matched_signatures: Vec<(usize, u32, u32)>,
    pub metrics: ProductionBaseStyleMetrics,
}

#[derive(Clone, Default)]
pub(crate) struct ProductionBaseStyleMetrics {
    pub incremental_style_updates: usize,
    pub collector_calls: usize,
    pub candidates_considered: usize,
    pub selector_matches: usize,
    pub transferred_property_sets: usize,
    pub matched_property_sets: usize,
    pub registry_dependent: bool,
    pub has_animations: bool,
    pub cascade_applied: bool,
    pub matched_property_cache_hits: usize,
    pub matched_property_cache_adds: usize,
    pub uses_root_relative_units: bool,
    pub uses_glyph_relative_units: bool,
    pub uses_line_height_units: bool,
    pub viewport_unit_flags: u32,
    pub uses_tree_counting_functions: bool,
    pub forward_positional_parents: HashSet<usize>,
    pub backward_positional_parents: HashSet<usize>,
    pub text_link_text_color: Option<foundation::Color>,
}

impl ProductionBaseStyleOutput {
    pub(crate) fn Get(&self) -> *mut ComputedStyle {
        self.style.Get()
    }
}

impl AsRef<Persistent<ComputedStyle>> for ProductionBaseStyleOutput {
    fn as_ref(&self) -> &Persistent<ComputedStyle> {
        self.style.as_ref()
    }
}

fn NamespaceURIFor(namespace: DOMNamespace) -> AtomicString {
    AtomicString::from_str(match namespace {
        DOMNamespace::kHTML => "http://www.w3.org/1999/xhtml",
        DOMNamespace::kSVG => "http://www.w3.org/2000/svg",
        DOMNamespace::kMathML => "http://www.w3.org/1998/Math/MathML",
        DOMNamespace::kNone => "",
    })
}

/// Executes the translated `StyleResolver::MatchAllRules` body while routing
/// selector checks and transfer through `ProductionElementRuleCollectorRuntime`.
/// Unsupported document features are reported after the trial, so callers
/// never consume a partially matched result as if it were complete.
pub(crate) fn MatchAllRulesForElement(
    inputs: ProductionRuleMatchingInputs<'_>,
) -> Result<MatchResult, Vec<ProductionRuleMatchingFallback>> {
    if inputs.element_index >= inputs.document.NodeCount()
        || inputs.document.Node(inputs.element_index).Type() != DOMNodeType::kElement
    {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "non-element-match-target",
        )]);
    }
    let element = ProductionResolverElement::ForNode(
        inputs.document,
        inputs.element_index,
        &inputs.media.media_type,
        true,
    );
    let host = ProductionRuleMatchingHost::new_with_user_agent_rule_sets(
        inputs.document,
        inputs.media,
        inputs.html_user_agent_rules,
        inputs.svg_user_agent_rules,
        inputs.mathml_user_agent_rules,
        inputs.author_rule_sets,
        ProductionBaseStyleCollaborators::ordinary(inputs.media.em_size),
    );
    let state = ProductionResolverState {
        element: element.clone(),
        pseudo: false,
        pseudo_id: PseudoId::kPseudoIdNone,
        builder: ComputedStyleBuilder::from_style(ProductionRuleMatchingHost::style(
            &host.initial.initial,
        )),
        parent: None,
        layout_parent: None,
        originating_style: None,
        output: None,
        had_no_matched_properties: false,
        conditionally_affects_animations: false,
        has_tree_scoped_reference: false,
        rejected_legacy_overlapping: false,
        length_conversion_flags: 0,
        incremental_properties: Vec::new(),
    };
    let mut collector = ProductionResolverCollector {
        document: inputs.document,
        element: element.clone(),
        selectors: host.selectors.clone(),
        result: Rc::new(RefCell::new(MatchResult::default())),
        pending_inputs: Vec::new(),
        pending_passes: Vec::new(),
        scope: None,
        matching_ua: false,
        pseudo: ProductionPseudoRequest::Element(),
        recalc: ProductionStyleRecalcContext::ForElement(
            inputs.document,
            inputs.element_index,
            host.cascade
                .container_runtime
                .and_then(|runtime| runtime.NearestSizeContainer(inputs.element_index)),
        ),
        inside_link: if host.IsLink(&element) {
            EInsideLink::kInsideUnvisitedLink
        } else {
            EInsideLink::kNotInsideLink
        },
    };
    StyleResolver::new(&element, &host).MatchAllRulesForProduction(&state, &mut collector, false);
    let fallbacks = host.take_fallbacks();
    if fallbacks.is_empty() {
        Ok(collector.result.borrow().clone())
    } else {
        Err(fallbacks)
    }
}

pub(crate) fn ApplyBaseStyleForElement(
    inputs: ProductionBaseStyleInputs<'_>,
) -> Result<ProductionBaseStyleOutput, Vec<ProductionRuleMatchingFallback>> {
    let ProductionBaseStyleInputs {
        matching: inputs,
        parent,
        mut collaborators,
    } = inputs;
    if inputs.element_index >= inputs.document.NodeCount()
        || inputs.document.Node(inputs.element_index).Type() != DOMNodeType::kElement
    {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "non-element-style-target",
        )]);
    }
    let mut trace = browser_tracing::span("style", "StyleResolver.ResolveElement");
    trace.set(
        "node_id",
        inputs.document.Node(inputs.element_index).Id() as f64,
    );
    trace.set("pseudo", collaborators.pseudo_id.value() as f64);
    trace.set(
        "parent_custom_properties",
        collaborators
            .parent_custom_properties
            .map_or(0, |p| p.values.len()) as f64,
    );
    let node = inputs.document.Node(inputs.element_index);
    if node.Namespace() == DOMNamespace::kNone {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "base:unsupported-element-namespace",
        )]);
    }
    if parent.is_none()
        && !collaborators.is_document_element
        && !layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(
            collaborators.pseudo_id,
        )
    {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "base:missing-parent-owner",
        )]);
    }
    // DOM animation samples must first be parsed into the same typed
    // interpolation owner used by ProductionCascadeInputs.  Refuse an
    // incomplete bundle rather than resolving a non-animated base and
    // publishing it as the final result.
    if collaborators.pseudo_id == PseudoId::kPseudoIdNone
        && ((!inputs.document.AnimationStyles(node.Id()).is_empty()
            && collaborators.animations.is_none())
            || (!inputs.document.TransitionStyles(node.Id()).is_empty()
                && collaborators.transitions.is_none()))
    {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "base:missing-interpolation-owner",
        )]);
    }
    // cpp: StyleResolver::ApplyBaseStyleNoCache supports a StyleRequest for an
    // originating element. Before/after use the translated virtual-pseudo
    // path, while first-letter uses the source dynamic-pseudo result because
    // CSSLogicalCombinationPseudo is disabled at the audited Chromium commit.
    // Placeholder is matched against its retained UA-shadow implementation
    // element while the outer request retains the originating-element owner.
    if collaborators.pseudo_id != PseudoId::kPseudoIdNone
        && !matches!(
            collaborators.pseudo_id,
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
        )
    {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "base:unsupported-pseudo-request",
        )]);
    }
    if layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(
        collaborators.pseudo_id,
    ) && collaborators.originating_style.is_none()
    {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "base:missing-highlight-originating-style",
        )]);
    }
    // The BaseStyle entry is element-specific, so the cascade subject is not
    // an additional document lookup.  Callers may still override it for the
    // originating element of a pseudo request.
    collaborators
        .subject_node
        .get_or_insert(inputs.element_index);
    let element = ProductionResolverElement::ForNode(
        inputs.document,
        inputs.element_index,
        &inputs.media.media_type,
        true,
    );
    let host = ProductionRuleMatchingHost::new_with_user_agent_rule_sets(
        inputs.document,
        inputs.media,
        inputs.html_user_agent_rules,
        inputs.svg_user_agent_rules,
        inputs.mathml_user_agent_rules,
        inputs.author_rule_sets,
        collaborators,
    );
    let request = host.NewStyleRequest();
    let context = host.NewStyleRecalcContext();
    let mut state = host.NewStyleResolverState(&element, &element, Some(&context), &request);
    state.parent = parent.map(Rc::new);
    state.layout_parent = state.parent.clone();
    state.originating_style = host.cascade.originating_style.clone().map(Rc::new);
    let mut cascade = host.NewStyleCascade(&mut state);
    let style = StyleResolver::new(&element, &host)
        .ResolveStyleForProduction(&element, &context, &request, &mut state, &mut cascade)
        .expect("production element resolution retains a style");
    {
        let metrics = host.base_style_metrics.borrow();
        trace.set(
            "incremental_style_updates",
            metrics.incremental_style_updates as f64,
        );
        trace.set("cache_hits", metrics.matched_property_cache_hits as f64);
        trace.set("cache_adds", metrics.matched_property_cache_adds as f64);
        trace.set("collector_calls", metrics.collector_calls as f64);
        trace.set(
            "candidates_considered",
            metrics.candidates_considered as f64,
        );
    }
    let fallbacks = host.take_fallbacks();
    trace.set("fallbacks", fallbacks.len() as f64);
    if !fallbacks.is_empty() {
        return Err(fallbacks);
    }
    let custom_properties = host
        .cascade_custom_output
        .borrow_mut()
        .take()
        .ok_or_else(|| {
            vec![ProductionRuleMatchingFallback::Unsupported(
                "base:missing-custom-properties-output",
            )]
        })?;
    let matched_signatures = std::mem::take(&mut *host.matched_signatures.borrow_mut());
    let metrics = host.base_style_metrics.borrow().clone();
    if let Some(fonts) = host.cascade.font_metrics {
        fonts.WillUseFont(unsafe { &*style.Get() }.GetFontDescription());
    }
    Ok(ProductionBaseStyleOutput {
        style,
        custom_properties,
        matched_signatures,
        metrics,
    })
}

/// Production caller for Chromium's `StyleResolver::ResolveBaseStyle` entry.
/// Animation code uses this only when it owns both inheritance and layout
/// parents; the returned style is the unanimated value later retained by
/// `StyleBaseData`. Source `ApplyBaseStyleNoCache` bypasses animation-base
/// reuse but still participates in the matched-properties cache.
pub(crate) fn ResolveBaseStyleForElement(
    inputs: ProductionResolveBaseStyleInputs<'_>,
) -> Result<ProductionBaseStyleOutput, Vec<ProductionRuleMatchingFallback>> {
    let ProductionResolveBaseStyleInputs {
        matching: inputs,
        parent,
        layout_parent,
        mut collaborators,
    } = inputs;
    if inputs.element_index >= inputs.document.NodeCount()
        || inputs.document.Node(inputs.element_index).Type() != DOMNodeType::kElement
    {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "non-element-style-target",
        )]);
    }
    let node = inputs.document.Node(inputs.element_index);
    if node.Namespace() != DOMNamespace::kHTML {
        return Err(vec![ProductionRuleMatchingFallback::Unsupported(
            "base:non-html-element",
        )]);
    }
    collaborators
        .subject_node
        .get_or_insert(inputs.element_index);
    let element = ProductionResolverElement::ForNode(
        inputs.document,
        inputs.element_index,
        &inputs.media.media_type,
        true,
    );
    let host = ProductionRuleMatchingHost::new_with_user_agent_rule_sets(
        inputs.document,
        inputs.media,
        inputs.html_user_agent_rules,
        inputs.svg_user_agent_rules,
        inputs.mathml_user_agent_rules,
        inputs.author_rule_sets,
        collaborators,
    );
    let context = host.NewStyleRecalcContext();
    let style = StyleResolver::new(&element, &host).ResolveBaseStyle(
        &element,
        Some(&parent),
        Some(&layout_parent),
        &context,
    );
    let fallbacks = host.take_fallbacks();
    if !fallbacks.is_empty() {
        return Err(fallbacks);
    }
    let custom_properties = host
        .cascade_custom_output
        .borrow_mut()
        .take()
        .ok_or_else(|| {
            vec![ProductionRuleMatchingFallback::Unsupported(
                "base:missing-custom-properties-output",
            )]
        })?;
    let matched_signatures = std::mem::take(&mut *host.matched_signatures.borrow_mut());
    let metrics = host.base_style_metrics.borrow().clone();
    Ok(ProductionBaseStyleOutput {
        style,
        custom_properties,
        matched_signatures,
        metrics,
    })
}

/// Compatibility name for direct tests and callers that do not retain a
/// cache owner. `ProductionBaseStyleCollaborators::ordinary` creates a fresh
/// owner, so these calls remain effectively no-cache while sharing the same
/// source lifecycle as document-owned resolution.
pub(crate) fn ApplyBaseStyleNoCacheForElement(
    inputs: ProductionBaseStyleInputs<'_>,
) -> Result<ProductionBaseStyleOutput, Vec<ProductionRuleMatchingFallback>> {
    ApplyBaseStyleForElement(inputs)
}

/// Runs the source-aligned `StyleResolver::InitialStyleForElement` path for
/// the standalone document host and returns the canonical native style root.
pub(crate) fn InitialStyleForDocument(
    media: &MediaValuesCachedData,
    color_scheme: ColorSchemeSettings,
) -> Persistent<ComputedStyle> {
    let host = ProductionInitialStyleHost::new(media, color_scheme);
    StyleResolver::new(&host, &host).InitialStyleForElement()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        css_value::CSSValuePayload,
        parser::css_parser_mode::CSSParserMode,
        parser::production_property_parser::{ParseCustomProperty, ParseDeclarationList},
        production_css_value::CSSVariableData,
        production_style_sheet::{ParseStyleSheet, ProductionMediaQueryEvaluator},
        resolver::{
            production_style_builder::custom_properties::{
                AttributeVariableResolver, EnvironmentVariableResolver, SubstitutionContext,
            },
            production_style_cascade::{
                ApplyMatchResult, ParseAnimationInterpolations, ProductionCascadeInputs,
            },
        },
    };

    fn element_with_id(document: &Document, id: &str) -> usize {
        (0..document.NodeCount())
            .find(|&index| {
                document
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == id)
            })
            .expect("fixture element must exist")
    }

    fn rule_set(css: &str, media: &MediaValuesCachedData) -> ProductionResolverRuleSetInput {
        let parsed = ParseStyleSheet(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        ProductionResolverRuleSetInput::plain(Rc::new(
            parsed.BuildRuleSet(&ProductionMediaQueryEvaluator::FromCachedData(media)),
        ))
    }

    fn ua_rule_set(css: &str, media: &MediaValuesCachedData) -> ProductionResolverRuleSetInput {
        let parsed = ParseStyleSheet(&foundation::String::from(css), CSSParserMode::kUASheetMode);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        ProductionResolverRuleSetInput::plain(Rc::new(
            parsed.BuildRuleSet(&ProductionMediaQueryEvaluator::FromCachedData(media)),
        ))
    }

    #[test]
    fn production_element_and_pseudo_hosts_share_selector_handles_but_not_sessions() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><div id=first></div><div id=second></div>",
        );
        let document = owner.GetDocument();
        let first = element_with_id(document, "first");
        let second = element_with_id(document, "second");
        let selectors = Rc::new(RefCell::new(PersistentSelectorService::default()));
        let original = selectors.borrow_mut().StyleMatchSession(document).Backend();
        let first_handle = original.handle(first);
        let second_handle = original.handle(second);
        for (subject, pseudo) in [
            (first, PseudoId::kPseudoIdNone),
            (second, PseudoId::kPseudoIdNone),
            (first, PseudoId::kPseudoIdBefore),
            (second, PseudoId::kPseudoIdSelection),
        ] {
            let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
            collaborators.subject_node = Some(subject);
            collaborators.pseudo_id = pseudo;
            collaborators.selectors = Some(selectors.clone());
            let host = ProductionRuleMatchingHost::new_with_collaborators(
                document,
                &media,
                ua_rule_set("", &media),
                Vec::new(),
                collaborators,
            );
            assert!(Rc::ptr_eq(&selectors, &host.selectors));
            let current = host
                .selectors
                .borrow_mut()
                .StyleMatchSession(document)
                .Backend();
            assert!(!Rc::ptr_eq(&original, &current));
            assert!(Rc::ptr_eq(&first_handle, &current.handle(first)));
            assert!(Rc::ptr_eq(&second_handle, &current.handle(second)));
        }
        let standalone =
            ProductionRuleMatchingHost::new(document, &media, ua_rule_set("", &media), Vec::new());
        assert!(!Rc::ptr_eq(&selectors, &standalone.selectors));
    }

    #[test]
    fn translated_quirks_color_uses_document_text_color_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let declaration = foundation::String::from("color:-internal-quirk-inherit");
        for mode in [
            CSSParserMode::kHTMLStandardMode,
            CSSParserMode::kHTMLQuirksMode,
        ] {
            let parsed = ParseDeclarationList(&declaration, mode);
            assert!(parsed.properties.is_empty());
            assert!(!parsed.errors.is_empty());
        }
        let ua = ParseDeclarationList(&declaration, CSSParserMode::kUASheetMode);
        assert!(ua.errors.is_empty(), "{:?}", ua.errors);
        assert!(matches!(
            ua.properties[0].Value().Payload(),
            CSSValuePayload::kIdentifierClass(value)
                if value.0 == CSSValueID::kInternalQuirkInherit
        ));

        // No doctype selects quirks mode, so StyleResolver adds the separate
        // quirks UA RuleSet. The document text owner deliberately differs
        // from the DOM parent's color to prove this is TextLinkColors rather
        // than ordinary inheritance or a keyword approximation.
        let media = MediaValuesCachedData::default();
        let owner =
            html::html_parser::ParseHTML("<body><table id=target><tr><td>x</td></tr></table>");
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let mut parent_builder =
            ComputedStyleBuilder::from_style(ProductionRuleMatchingHost::style(&initial));
        parent_builder.SetColor(&StyleColor::from_color(foundation::Color::FromRGB(4, 5, 6)));
        let parent = Persistent::from_ptr(parent_builder.TakeStyle().cast_mut());
        let expected = foundation::Color::FromRGB(1, 2, 3);
        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.html_quirks_user_agent_rules = Some(ua_rule_set(
            "table { color: -internal-quirk-inherit }",
            &media,
        ));
        collaborators.text_link_text_color = Some(expected);

        let actual = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: element_with_id(owner.GetDocument(), "target"),
                media: &media,
                html_user_agent_rules: ua_rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(Vec::new()),
            },
            parent: Some(parent),
            collaborators,
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected quirks-color fallback: {fallbacks:?}"));
        assert_eq!(
            ProductionRuleMatchingHost::style(actual.style.as_ref()).GetCurrentColor(None),
            expected
        );
    }

    #[test]
    fn translated_html_ua_quirky_em_and_auto_base_use_element_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let qem = ParseDeclarationList(
            &foundation::String::from("margin-top:1__qem"),
            CSSParserMode::kUASheetMode,
        );
        let CSSValuePayload::kNumericLiteralClass(qem) = qem.properties[0].Value().Payload() else {
            panic!("UA quirky em must remain a numeric literal")
        };
        assert_eq!(
            qem.GetType(),
            crate::css_primitive_value::UnitType::kQuirkyEms
        );
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><p id=paragraph>x</p>\
             <select id=popup><option>x</option></select>\
             <select id=multiple-popup multiple size=1><option>x</option></select>",
        );
        let ua = ua_rule_set(
            "p { margin-block-start: 1__qem } \
             select { appearance:base-select; \
               padding-left:-internal-auto-base(6px, 8px) }",
            &media,
        );
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let resolve = |id: &str| {
            ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: element_with_id(owner.GetDocument(), id),
                    media: &media,
                    html_user_agent_rules: ua.clone(),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(Vec::new()),
                },
                parent: Some(initial.clone()),
                collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
            })
            .expect("UA-only value semantics must stay on the translated resolver path")
            .style
        };

        let paragraph = resolve("paragraph");
        let paragraph = unsafe { &*paragraph.Get() };
        assert_eq!(paragraph.MarginTop().Pixels(), 16.0);
        assert!(paragraph.MarginTop().Quirk());

        let popup = resolve("popup");
        assert_eq!(unsafe { &*popup.Get() }.PaddingLeft().Pixels(), 8.0);

        // Chromium rejects base-select for the exceptional menu-list-backed
        // multiple select; -internal-auto-base() therefore keeps argument 1.
        let multiple_popup = resolve("multiple-popup");
        assert_eq!(
            unsafe { &*multiple_popup.Get() }.PaddingLeft().Pixels(),
            6.0
        );
    }

    #[test]
    fn translated_html_ua_lh_math_uses_current_computed_line_height() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML("<!doctype html><button id=target>x</button>");
        let ua = ua_rule_set(
            "button { line-height:32px; min-block-size:max(24px, 1lh) }",
            &media,
        );
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let output = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: element_with_id(owner.GetDocument(), "target"),
                media: &media,
                html_user_agent_rules: ua,
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(Vec::new()),
            },
            parent: Some(initial),
            collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
        })
        .expect("lh math must stay on the translated resolver path");
        let style = unsafe { &*output.style.Get() };
        assert_eq!(style.MinHeight().Pixels(), 32.0);
        assert!(style.HasGlyphRelativeUnits());
        assert!(style.HasLineHeightRelativeUnits());
    }

    #[test]
    fn translated_rlh_uses_document_root_computed_line_height() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><html id=root><body><div id=target>x</div></body></html>",
        );
        let author = rule_set(
            "html { line-height:40px } \
             #target { line-height:10px; min-width:1rlh; min-height:1lh }",
            &media,
        );
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let mut root_collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        root_collaborators.is_document_element = true;
        let root = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: element_with_id(owner.GetDocument(), "root"),
                media: &media,
                html_user_agent_rules: ua_rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![author.clone()]),
            },
            parent: Some(initial),
            collaborators: root_collaborators,
        })
        .expect("root line-height owner must stay on the translated path")
        .style;

        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.root_style = Some((*root).clone());
        let output = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: element_with_id(owner.GetDocument(), "target"),
                media: &media,
                html_user_agent_rules: ua_rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![author]),
            },
            parent: Some((*root).clone()),
            collaborators,
        })
        .expect("rlh must use the retained document root style");
        let style = unsafe { &*output.style.Get() };
        assert_eq!(style.MinWidth().Pixels(), 40.0);
        assert_eq!(style.MinHeight().Pixels(), 10.0);
        assert!(style.HasGlyphRelativeUnits());
        assert!(style.HasLineHeightRelativeUnits());
        assert!(style.HasRootRelativeUnits());
    }

    #[test]
    fn translated_html_ua_textarea_resize_uses_document_setting_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        assert!(ParseDeclarationList(
            &foundation::String::from("resize:-internal-textarea-auto"),
            CSSParserMode::kHTMLStandardMode,
        )
        .properties
        .is_empty());
        let ua_declaration = ParseDeclarationList(
            &foundation::String::from("resize:-internal-textarea-auto"),
            CSSParserMode::kUASheetMode,
        );
        let CSSValuePayload::kIdentifierClass(value) =
            ua_declaration.properties[0].Value().Payload()
        else {
            panic!("UA textarea resize must remain an identifier")
        };
        assert_eq!(value.0, CSSValueID::kInternalTextareaAuto);

        let resolve = |text_areas_are_resizable| {
            let mut owner =
                html::html_parser::ParseHTML("<!doctype html><textarea id=target></textarea>");
            owner
                .GetDocumentMut()
                .SetTextAreasAreResizable(text_areas_are_resizable);
            let ua = ua_rule_set("textarea { resize:-internal-textarea-auto }", &media);
            let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
            let output = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: element_with_id(owner.GetDocument(), "target"),
                    media: &media,
                    html_user_agent_rules: ua,
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(Vec::new()),
                },
                parent: Some(initial),
                collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
            })
            .expect("UA textarea resize must stay on the translated resolver path");
            unsafe { &*output.style.Get() }.Resize()
        };

        assert_eq!(resolve(true), foundation::EResize::kBoth);
        assert_eq!(resolve(false), foundation::EResize::kNone);
    }

    #[test]
    fn translated_html_ua_small_control_system_font_uses_document_setting_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let declaration = ParseDeclarationList(
            &foundation::String::from("font:-webkit-small-control"),
            CSSParserMode::kUASheetMode,
        );
        assert_eq!(
            declaration.properties.len(),
            crate::parser::production_property_metadata::ShorthandFor(CSSPropertyID::kFont).len()
        );
        assert!(declaration.properties.iter().all(|property| matches!(
            property.Value().Payload(),
            CSSValuePayload::kPendingSystemFontValueClass(value)
                if value.system_font_id == CSSValueID::kWebkitSmallControl
        )));

        let mut owner =
            html::html_parser::ParseHTML("<!doctype html><button id=target>control</button>");
        owner.GetDocumentMut().SetDefaultFontSize(18.0);
        let ua = ua_rule_set("button { font:-webkit-small-control }", &media);
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let output = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: element_with_id(owner.GetDocument(), "target"),
                media: &media,
                html_user_agent_rules: ua,
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(Vec::new()),
            },
            parent: Some(initial),
            collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
        })
        .expect("pending system font must stay on the translated resolver path");
        let font = unsafe { &*output.style.Get() }.GetFontDescription();
        assert_eq!(font.Family().FamilyName().Utf8(), "Arial");
        assert!((font.SpecifiedSize() - (18.0 - 2.0 / 72.0 * 96.0)).abs() < 0.001);
        assert_eq!(font.Weight().ToFloat(), 400.0);
    }

    #[test]
    fn translated_html_ua_auto_appearance_uses_form_control_and_theme_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html>\
             <button id=button>x</button>\
             <input id=submit type=submit>\
             <input id=checkbox type=checkbox>\
             <input id=search type=search>\
             <input id=text type=text>\
             <input id=styled type=text>\
             <textarea id=textarea></textarea>\
             <select id=menu><option>x</option></select>\
             <select id=list multiple><option>x</option></select>\
             <meter id=meter></meter><progress id=progress></progress>",
        );
        let ua = ua_rule_set(
            "button,input,textarea,select,meter,progress { appearance:auto }\
             input[type=checkbox] { padding:9px }",
            &media,
        );
        let author = rule_set("#styled { border-style:solid; border-width:1px }", &media);
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let resolve = |id: &str| {
            ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: element_with_id(owner.GetDocument(), id),
                    media: &media,
                    html_user_agent_rules: ua.clone(),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(vec![author.clone()]),
                },
                parent: Some(initial.clone()),
                collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
            })
            .expect("ordinary form theme adjustment must stay on the translated path")
            .style
        };
        for (id, expected) in [
            ("button", AppearanceValue::kButton),
            ("submit", AppearanceValue::kPushButton),
            ("checkbox", AppearanceValue::kCheckbox),
            ("search", AppearanceValue::kSearchField),
            ("text", AppearanceValue::kTextField),
            ("textarea", AppearanceValue::kTextArea),
            ("menu", AppearanceValue::kMenulist),
            ("list", AppearanceValue::kListbox),
            ("meter", AppearanceValue::kMeter),
            ("progress", AppearanceValue::kProgressBar),
        ] {
            let style = resolve(id);
            let style = unsafe { &*style.Get() };
            assert_eq!(style.EffectiveAppearance(), expected, "{id}");
            assert_eq!(style.Display(), EDisplay::kInlineBlock, "{id}");
        }
        let checkbox = resolve("checkbox");
        let checkbox = unsafe { &*checkbox.Get() };
        assert_eq!(checkbox.PaddingTop().Pixels(), 0.0);
        assert!(checkbox.ShouldIgnoreOverflowPropertyForInlineBlockBaseline());
        assert_eq!(
            checkbox.InlineBlockBaselineEdge(),
            EInlineBlockBaselineEdge::kBorderBox
        );
        assert_eq!(
            unsafe { &*resolve("styled").Get() }.EffectiveAppearance(),
            AppearanceValue::kNone
        );
    }

    #[test]
    fn translated_html_ua_textfield_decoration_container_uses_retained_shadow_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><input id=target type=search value=fixture>",
        );
        let document = owner.GetDocument();
        let host = element_with_id(document, "target");
        let container = document
            .TextControlContainerElement(host)
            .expect("search input must retain its text-field decoration container");
        assert_eq!(
            document.ShadowPseudoId(container),
            Some("-webkit-textfield-decoration-container")
        );

        // cpp: html.css input::-webkit-textfield-decoration-container. This
        // is a real UA-shadow implementation element, so the source selector
        // crosses kUAShadow to the input host and applies its retained typed
        // declaration set to the container itself.
        let ua = ua_rule_set(
            "input::-webkit-textfield-decoration-container { \
                display:flex !important; align-items:center; \
                -webkit-user-modify:read-only !important; content:none !important; \
                writing-mode:inherit !important \
            }",
            &media,
        );
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let output = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document,
                element_index: container,
                media: &media,
                html_user_agent_rules: ua,
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(Vec::new()),
            },
            parent: Some(initial),
            collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
        })
        .unwrap_or_else(|fallbacks| {
            panic!("retained text-field decoration container must stay translated: {fallbacks:?}")
        });
        let actual = unsafe { &*output.style.Get() };
        assert_eq!(actual.Display(), EDisplay::kFlex);
        assert_eq!(
            actual.AlignItems().GetPosition(),
            layoutng_style::style::computed_style_constants::ItemPosition::kCenter
        );
        assert_eq!(actual.UserModify(), EUserModify::kReadOnly);
        assert!(unsafe {
            (&*actual
                .GetContentData()
                .expect("content:none must retain native NoneContentData"))
                .IsNone()
        });
    }

    #[test]
    fn translated_mathml_custom_length_properties_reach_computed_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><math>\
             <mspace id=space width=4px height=2em depth=3px></mspace>\
             <mspace id=invalid width=50% height=calc(2px) depth=-1px></mspace>\
             <mpadded id=padded width=6px height=3px depth=4px lspace=5px voffset=-6px></mpadded>\
             <mfrac id=fraction linethickness=25%><mi>x</mi><mi>y</mi></mfrac>\
             <mo id=operator lspace=1px rspace=2% minsize=3px maxsize=4px>+</mo>\
             </math>",
        );
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let empty = rule_set("", &media);
        let resolve = |id: &str| {
            ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: element_with_id(owner.GetDocument(), id),
                    media: &media,
                    html_user_agent_rules: empty.clone(),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: Some(empty.clone()),
                    author_rule_sets: Rc::new(Vec::new()),
                },
                parent: Some(initial.clone()),
                collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
            })
            .expect("supported MathML custom properties stay on the translated resolver path")
            .style
        };

        let space = resolve("space");
        let space = unsafe { &*space.Get() };
        assert_eq!(space.GetMathBaseline().Pixels(), 32.0);
        assert_eq!(space.Width().Pixels(), 4.0);
        assert_eq!(space.Height().Pixels(), 35.0);

        let invalid = resolve("invalid");
        let invalid = unsafe { &*invalid.Get() };
        assert!(invalid.Width().IsAuto());
        assert!(invalid.Height().IsAuto());

        let padded = resolve("padded");
        let padded = unsafe { &*padded.Get() };
        assert_eq!(padded.Width().Pixels(), 6.0);
        assert_eq!(padded.GetMathBaseline().Pixels(), 3.0);
        assert_eq!(padded.GetMathPaddedDepth().Pixels(), 4.0);
        assert_eq!(padded.GetMathLSpace().Pixels(), 5.0);
        assert_eq!(padded.GetMathPaddedVOffset().Pixels(), -6.0);

        let fraction = resolve("fraction");
        let fraction = unsafe { &*fraction.Get() };
        assert_eq!(fraction.GetMathFractionBarThickness().PercentValue(), 25.0);

        let operator = resolve("operator");
        let operator = unsafe { &*operator.Get() };
        assert_eq!(operator.GetMathLSpace().Pixels(), 1.0);
        assert_eq!(operator.GetMathRSpace().PercentValue(), 2.0);
        assert_eq!(operator.GetMathMinSize().Pixels(), 3.0);
        assert_eq!(operator.GetMathMaxSize().Pixels(), 4.0);
    }

    #[test]
    fn translated_nested_ua_declarations_preserve_button_alignment() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML("<!doctype html><button id=target>x</button>");
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        for css in [
            "button {text-align:center;color:red}",
            "button {color:red; @supports (display:block) {text-align:center}}",
            "button {color:red; @supports not blink-feature(AppearanceBase) {text-align:center}}",
        ] {
            let output = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: element_with_id(owner.GetDocument(), "target"),
                    media: &media,
                    html_user_agent_rules: ua_rule_set(css, &media),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(Vec::new()),
                },
                parent: Some(initial.clone()),
                collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
            })
            .unwrap();
            assert_eq!(
                unsafe { &*output.style.Get() }.GetTextAlign(),
                foundation::ETextAlign::kCenter,
                "{css}"
            );
        }
    }

    #[test]
    fn translated_svg_presentation_attributes_reach_native_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            r##"<!doctype html><svg><path id="shape" fill="none" stroke="#123456"
                stroke-width="2" stroke-linecap="round" fill-rule="evenodd"
                transform="translate(3 4)" d="M0 0L10 10"/></svg>"##,
        );
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let empty = rule_set("", &media);
        let output = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: element_with_id(owner.GetDocument(), "shape"),
                media: &media,
                html_user_agent_rules: empty.clone(),
                svg_user_agent_rules: Some(empty),
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(Vec::new()),
            },
            parent: Some(initial),
            collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
        })
        .expect("SVG hints must stay on the translated resolver path");
        let style = unsafe { &*output.style.Get() };
        assert!(
            !style.FillPaint().IsColor(),
            "fill=none must suppress the initial black fill"
        );
        assert!(style.StrokePaint().IsColor());
        assert_eq!(
            style.StrokePaint().GetColor().GetColor().Param0(),
            0x12 as f32
        );
        assert_eq!(style.StrokeWidth().length().Pixels(), 2.0);
        assert_eq!(style.CapStyle(), foundation::LineCap::kRoundCap);
        assert_eq!(style.Transform().size(), 1);
    }

    #[test]
    fn translated_mathml_presentation_attributes_reach_cascade_and_computed_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><math>\
             <mstyle id=attributes mathsize=20px mathcolor=#123456 \
               mathbackground=#abcdef scriptlevel=3 displaystyle=false dir=rtl></mstyle>\
             <mi id=variant mathvariant=normal>x</mi>\
             <mi id=relative scriptlevel=+2>y</mi>\
             </math>",
        );
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let html_ua = rule_set("", &media);
        let mathml_ua = rule_set(
            "mstyle { font-size:7px; color:red; background-color:black; \
               math-depth:1; math-style:normal; direction:ltr } \
             mi { text-transform:math-auto }",
            &media,
        );
        let resolve = |id: &str| {
            ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: element_with_id(owner.GetDocument(), id),
                    media: &media,
                    html_user_agent_rules: html_ua.clone(),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: Some(mathml_ua.clone()),
                    author_rule_sets: Rc::new(Vec::new()),
                },
                parent: Some(initial.clone()),
                collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
            })
            .expect("MathML presentation hints stay on the translated resolver path")
            .style
        };

        let attributes = resolve("attributes");
        let attributes = unsafe { &*attributes.Get() };
        assert_eq!(attributes.GetFontDescription().ComputedSize(), 20.0);
        assert_eq!(
            attributes.GetCurrentColor(None),
            foundation::Color::FromRGB(0x12, 0x34, 0x56)
        );
        assert_eq!(
            attributes.BackgroundColor().GetColor(),
            foundation::Color::FromRGB(0xab, 0xcd, 0xef)
        );
        assert_eq!(attributes.MathDepth(), 3);
        assert_eq!(attributes.MathStyle(), foundation::EMathStyle::kCompact);
        assert_eq!(attributes.Direction(), TextDirection::kRtl);

        let variant = resolve("variant");
        let variant = unsafe { &*variant.Get() };
        assert_eq!(variant.TextTransform(), foundation::ETextTransform::kNone);

        let relative = resolve("relative");
        let relative = unsafe { &*relative.Get() };
        assert_eq!(relative.MathDepth(), 2);
    }

    fn variable_data(text: &str) -> Rc<CSSVariableData> {
        let property = ParseCustomProperty(
            "--fixture",
            &foundation::String::from(text),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .expect("fixture custom property must parse");
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
        fn GetAttribute(&self, local_name: &AtomicString) -> foundation::String {
            (local_name == &AtomicString::from_str("data-size"))
                .then(|| foundation::String::from("43"))
                .unwrap_or_default()
        }
    }

    #[test]
    fn translated_initial_style_keeps_device_scale_out_of_layout_zoom() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            device_pixel_ratio: 2.0,
            em_size: 19.0,
            forced_colors: ForcedColors::kActive,
            ..Default::default()
        };
        let style = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let style = unsafe { &*style.Get() };
        assert_eq!(style.Zoom(), 1.0);
        assert_eq!(style.EffectiveZoom(), 1.0);
        assert!(style.InForcedColorsMode());
        assert_eq!(style.GetFontDescription().SpecifiedSize(), 19.0);
        assert_eq!(style.GetFontDescription().ComputedSize(), 19.0);
    }

    #[test]
    fn translated_state_retains_tree_animation_and_author_metadata() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        let target = element_with_id(owner.GetDocument(), "target");
        let host = ProductionRuleMatchingHost::new(
            owner.GetDocument(),
            &media,
            rule_set("", &media),
            Vec::new(),
        );
        let element = ProductionResolverElement::ForNode(
            owner.GetDocument(),
            target,
            &media.media_type,
            true,
        );
        let request = host.NewStyleRequest();
        let context = host.NewStyleRecalcContext();
        let mut state = host.NewStyleResolverState(&element, &element, Some(&context), &request);

        host.StateSetTreeScopedReference(&mut state);
        assert!(state.has_tree_scoped_reference);

        assert!(!host.StateCanAffectAnimations(&state));
        host.StateSetConditionallyAffectsAnimations(&mut state);
        assert!(host.StateCanAffectAnimations(&state));

        host.StateSetComputedStyleFlagsFromAuthorFlags(
            &mut state,
            CSSProperty::kBackground
                | CSSProperty::kBorder
                | CSSProperty::kBorderRadius
                | CSSProperty::kHighlightColors,
        );
        assert!(state.builder.HasAuthorBackground());
        assert!(state.builder.HasAuthorBorder());
        assert!(state.builder.HasAuthorBorderRadius());
        assert!(state.builder.HasAuthorHighlightColors());
        assert!(host.take_fallbacks().is_empty());
    }

    #[test]
    fn translated_matched_properties_cache_hits_and_rejects_stateful_results() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><main><div id=first class=hit></div><div id=second class=hit></div></main>",
        );
        let first = element_with_id(owner.GetDocument(), "first");
        let second = element_with_id(owner.GetDocument(), "second");
        let parent = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let author = rule_set(".hit { width:37px; color:rgb(1 2 3) }", &media);
        let cache = ProductionMatchedPropertiesCache::default();
        let resolve = |target, animations: Option<&ProductionActiveInterpolationsMap>| {
            let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
            collaborators.matched_properties_cache = Some(cache.clone());
            collaborators.animations = animations;
            ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: target,
                    media: &media,
                    html_user_agent_rules: rule_set("", &media),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(vec![author.clone()]),
                },
                parent: Some(parent.clone()),
                collaborators,
            })
            .unwrap_or_else(|fallbacks| panic!("unexpected cache fallback: {fallbacks:?}"))
        };

        let first_output = resolve(first, None);
        assert_eq!(first_output.metrics.matched_property_cache_adds, 1);
        assert_eq!(first_output.metrics.matched_property_cache_hits, 0);
        let second_output = resolve(second, None);
        assert_eq!(second_output.metrics.matched_property_cache_hits, 1);
        assert_eq!(
            ProductionRuleMatchingHost::style(second_output.as_ref()).Width(),
            ProductionRuleMatchingHost::style(first_output.as_ref()).Width()
        );

        // The same native parent can carry a different typed variable owner.
        // Cache hits must restore computed output, and a variable mutation
        // must miss without relying on native pointer identity or hash alone.
        let variable_author = rule_set(
            ".hit { width:var(--width); --derived:var(--width) }",
            &media,
        );
        let mut variables = CustomProperties::default();
        variables.SetData("--width".into(), crate::resolver::production_style_builder::custom_properties::ComputedVariableValue::Data(variable_data("37px")));
        let resolve_variables = |target, variables: &CustomProperties| {
            let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
            collaborators.matched_properties_cache = Some(cache.clone());
            collaborators.parent_custom_properties = Some(variables);
            ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: target,
                    media: &media,
                    html_user_agent_rules: rule_set("", &media),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(vec![variable_author.clone()]),
                },
                parent: Some(parent.clone()),
                collaborators,
            })
            .expect("variables retain the typed resolver and MPC")
        };
        let first_variable = resolve_variables(first, &variables);
        assert_eq!(first_variable.metrics.matched_property_cache_adds, 1);
        let second_variable = resolve_variables(second, &variables);
        assert_eq!(second_variable.metrics.matched_property_cache_hits, 1);
        assert_eq!(
            unsafe { &*second_variable.style.Get() }.Width().Pixels(),
            37.0
        );
        assert_eq!(
            second_variable
                .custom_properties
                .Get("--derived")
                .unwrap()
                .Serialize()
                .Utf8(),
            "37px"
        );
        let old_hash = variables.CacheHash();
        variables.SetData("--width".into(), crate::resolver::production_style_builder::custom_properties::ComputedVariableValue::Data(variable_data("41px")));
        assert_ne!(variables.CacheHash(), old_hash);
        let changed_variable = resolve_variables(second, &variables);
        assert_eq!(changed_variable.metrics.matched_property_cache_hits, 0);
        assert_eq!(
            unsafe { &*changed_variable.style.Get() }.Width().Pixels(),
            41.0
        );
        assert_eq!(
            changed_variable
                .custom_properties
                .Get("--derived")
                .unwrap()
                .Serialize()
                .Utf8(),
            "41px"
        );

        let animations = ProductionActiveInterpolationsMap::default();
        let animated = resolve(second, Some(&animations));
        assert_eq!(animated.metrics.matched_property_cache_hits, 0);
        assert_eq!(animated.metrics.matched_property_cache_adds, 0);

        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.matched_properties_cache = Some(cache.clone());
        let host = ProductionRuleMatchingHost::new_with_collaborators(
            owner.GetDocument(),
            &media,
            rule_set("", &media),
            vec![author],
            collaborators,
        );
        let element =
            ProductionResolverElement::ForNode(owner.GetDocument(), first, &media.media_type, true);
        let request = host.NewStyleRequest();
        let context = host.NewStyleRecalcContext();
        let mut state = host.NewStyleResolverState(&element, &element, Some(&context), &request);
        state.parent = Some(Rc::new(parent.clone()));
        state.layout_parent = state.parent.clone();
        *host.cascade_custom_output.borrow_mut() = Some(Rc::new(CustomProperties::default()));
        host.StateSetTreeScopedReference(&mut state);
        assert!(!host.MatchedPropertiesIsCacheable(&state));
    }

    #[test]
    fn translated_match_all_rules_matches_document_engine_for_ordinary_author_rules() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            ..Default::default()
        };
        let css = "#target.hit { display:block; position:absolute; width:25px; opacity:.5 } \
                   #target.miss { width:99px }";
        let mut owner =
            html::html_parser::ParseHTML("<!doctype html><div id=target class=hit></div>");
        owner
            .GetDocumentMut()
            .AppendStyleSheet(crate::ParseCSS(css));
        let target = element_with_id(owner.GetDocument(), "target");
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        let expected = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };
        let expected_values = (
            expected.Display(),
            expected.GetPosition(),
            expected.Width(),
            expected.Opacity(),
        );

        let matched = MatchAllRulesForElement(ProductionRuleMatchingInputs {
            document: owner.GetDocument(),
            element_index: target,
            media: &media,
            html_user_agent_rules: rule_set("", &media),
            svg_user_agent_rules: None,
            mathml_user_agent_rules: None,
            author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
        })
        .expect("ordinary document matching must not use a compatibility fallback");
        assert_eq!(matched.GetMatchedProperties().len(), 1);

        let base = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let actual = ApplyMatchResult(ProductionCascadeInputs {
            document: None,
            media: media.clone(),
            base,
            parent: None,
            root_style: None,
            match_result: matched,
            root_font_size: 16.0,
            color_scheme_settings: ColorSchemeSettings::default(),
            system_colors: ProductionSystemColorContext::default(),
            text_link_text_color: None,
            is_document_element: false,
            pseudo_id: layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdNone,
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
        })
        .expect("ordinary declarations must apply through translated StyleCascade");
        let actual = unsafe { &*actual.Get() };
        assert_eq!(actual.Display(), expected_values.0);
        assert_eq!(actual.GetPosition(), expected_values.1);
        assert_eq!(actual.Width(), expected_values.2);
        assert_eq!(actual.Opacity(), expected_values.3);
    }

    #[test]
    fn translated_match_all_rules_retains_parser_owned_inline_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let mut owner =
            html::html_parser::ParseHTML("<!doctype html><div id=target style='width:17px'></div>");
        let target = element_with_id(owner.GetDocument(), "target");
        let result = MatchAllRulesForElement(ProductionRuleMatchingInputs {
            document: owner.GetDocument(),
            element_index: target,
            media: &media,
            html_user_agent_rules: rule_set("", &media),
            svg_user_agent_rules: None,
            mathml_user_agent_rules: None,
            author_rule_sets: Rc::new(Vec::new()),
        })
        .expect("translated MatchAllRules must own inline declarations");
        assert_eq!(result.GetMatchedProperties().len(), 1);
        assert!(result.GetMatchedProperties()[0].data_.is_inline_style());
        let properties = result.GetMatchedProperties()[0]
            .properties
            .AsAny()
            .downcast_ref::<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>()
            .expect("inline declaration retains translated property-set owner");
        let properties = match properties {
            CSSPropertyValueSetRuleHandle::Immutable(properties) => properties.Properties(),
            CSSPropertyValueSetRuleHandle::Mutable(properties) => {
                panic!("inline parser unexpectedly returned mutable set: {properties:p}")
            }
        };
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].PropertyID(), CSSPropertyID::kWidth);

        // Each production request constructs a fresh host. The document
        // collaborator must retain declaration identity across those requests,
        // without reusing it after an attribute, parser-mode or document change.
        let declarations = Rc::new(ProductionDeclarationCache::default());
        let inline = |document: &Document| {
            let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
            collaborators.declarations = Some(declarations.clone());
            let host = ProductionRuleMatchingHost::new_with_collaborators(
                document,
                &media,
                rule_set("", &media),
                Vec::new(),
                collaborators,
            );
            let element =
                ProductionResolverElement::ForNode(document, target, &media.media_type, true);
            host.InlineStyle(&element)
                .expect("inline declaration must exist")
        };
        let first = inline(owner.GetDocument());
        assert!(Rc::ptr_eq(&first, &inline(owner.GetDocument())));
        let mut attribute = owner
            .GetDocument()
            .Node(target)
            .FindAttribute("style")
            .unwrap()
            .clone();
        attribute.value = "width:23px".into();
        owner.GetDocumentMut().SetAttribute(target, attribute);
        let changed = inline(owner.GetDocument());
        assert!(!Rc::ptr_eq(&first, &changed));
        assert!(Rc::ptr_eq(&changed, &inline(owner.GetDocument())));
        // A CSSOM write transfers its parsed property-set owner in the same
        // mutation as the attribute, before any reentrant style request.
        let current = owner.GetDocument().Node(target).InlineStyle();
        let edited = match crate::production_style_sheet_projection::SetInlineCSSDeclaration(
            &current, "width", "24px", "", true,
        ).unwrap() {
            crate::production_style_sheet_projection::CSSInlineDeclarationMutation::Changed(value)
            | crate::production_style_sheet_projection::CSSInlineDeclarationMutation::ChangedIndependent(value) => value,
            _ => panic!("the inline width must change"),
        };
        let parsed = edited
            .ParsedBacking::<crate::production_style_sheet_projection::ParsedInlineStyle>()
            .unwrap();
        let mut mutation = dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kSetAttribute,
            target_node_id: owner.GetDocument().Node(target).Id(),
            name: "style".into(),
            value: edited.SourceText().into(),
            inline_style: Some(edited),
            ..Default::default()
        };
        owner.ApplyMutation(&mutation).unwrap();
        assert!(
            Rc::ptr_eq(&parsed.Declarations(), &inline(owner.GetDocument())),
            "the resolver must use the exact CSSOM property-set owner"
        );
        mutation.value = "width:23px".into();
        mutation.inline_style = None;
        owner.ApplyMutation(&mutation).unwrap();
        let changed = inline(owner.GetDocument());
        let quirks =
            declarations.Resolve(owner.GetDocument(), target, CSSParserMode::kHTMLQuirksMode);
        assert!(!Rc::ptr_eq(&changed, &quirks));
        owner
            .GetDocumentMut()
            .RemoveAttributeDefault(target, "style");
        assert_eq!(
            owner.GetDocument().Node(target).InlineStyle().SourceText(),
            ""
        );
        assert!(!Rc::ptr_eq(
            &changed,
            &declarations.Resolve(
                owner.GetDocument(),
                target,
                CSSParserMode::kHTMLStandardMode,
            )
        ));
        let other = html::html_parser::ParseHTML("<!doctype html><div style='width:23px'></div>");
        let replacement = declarations.Resolve(
            other.GetDocument(),
            target,
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(!Rc::ptr_eq(&changed, &replacement));
    }

    #[test]
    fn translated_match_all_rules_uses_document_old_style_for_starting_style() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let mut owner =
            html::html_parser::ParseHTML("<!doctype html><div id=target class=hit></div>");
        let target = element_with_id(owner.GetDocument(), "target");
        let css = "@starting-style{.hit{font-size:31px}}.hit{width:7px}";

        let initial = MatchAllRulesForElement(ProductionRuleMatchingInputs {
            document: owner.GetDocument(),
            element_index: target,
            media: &media,
            html_user_agent_rules: rule_set("", &media),
            svg_user_agent_rules: None,
            mathml_user_agent_rules: None,
            author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
        })
        .expect("initial resolver pass must own @starting-style");
        assert_eq!(initial.GetMatchedProperties().len(), 2);
        assert!(initial.HasFlag(crate::resolver::match_flags::MatchFlag::kAffectedByStartingStyle));

        owner
            .GetDocumentMut()
            .SetResolvedStyle(target, Default::default());
        let subsequent = MatchAllRulesForElement(ProductionRuleMatchingInputs {
            document: owner.GetDocument(),
            element_index: target,
            media: &media,
            html_user_agent_rules: rule_set("", &media),
            svg_user_agent_rules: None,
            mathml_user_agent_rules: None,
            author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
        })
        .expect("subsequent resolver pass must reject @starting-style");
        assert_eq!(subsequent.GetMatchedProperties().len(), 1);
        assert!(
            !subsequent.HasFlag(crate::resolver::match_flags::MatchFlag::kAffectedByStartingStyle)
        );
    }

    #[test]
    fn translated_apply_base_style_no_cache_matches_document_engine_for_ordinary_element() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            ..Default::default()
        };
        let css = "#target.hit { display:block; position:absolute; width:25px; opacity:.5 }";
        let mut owner = html::html_parser::ParseHTML(
            "<!doctype html><div id=target class=hit style='width:33px'></div>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(crate::ParseCSS(css));
        let target = element_with_id(owner.GetDocument(), "target");
        let mut engine = crate::StyleEngine::new(&owner);
        engine.Update(&mut owner, &media, &[]).unwrap();
        let expected = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };
        let expected_values = (
            expected.Display(),
            expected.GetPosition(),
            expected.Width(),
            expected.Opacity(),
        );
        let parent = owner
            .GetDocument()
            .Node(target)
            .Parent()
            .and_then(|parent| {
                owner
                    .GetDocument()
                    .ResolvedStyleFor(parent)
                    .map(|resolved| resolved.native_style.clone())
            });
        let actual = ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: target,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
            },
            parent,
            collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected base-style fallbacks: {fallbacks:?}"));
        let actual = ProductionRuleMatchingHost::style(actual.as_ref());
        assert_eq!(actual.Display(), expected_values.0);
        assert_eq!(actual.GetPosition(), expected_values.1);
        assert_eq!(actual.Width(), expected_values.2);
        assert_eq!(actual.Opacity(), expected_values.3);
    }

    #[test]
    fn translated_apply_base_style_routes_supported_pseudo_requests_through_collector() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        };
        let css = ".host { font-size:99px } \
                   .host::before { font-size:21px } \
                   .host::after { font-size:22px } \
                   .host::first-letter { font-size:23px }";

        for (pseudo_id, expected_size) in [
            (PseudoId::kPseudoIdBefore, 21.0),
            (PseudoId::kPseudoIdAfter, 22.0),
            (PseudoId::kPseudoIdFirstLetter, 23.0),
        ] {
            let owner = html::html_parser::ParseHTML(
                "<!doctype html><div id=target class=host>fixture</div>",
            );
            let target = element_with_id(owner.GetDocument(), "target");
            let originating = InitialStyleForDocument(&media, ColorSchemeSettings::default());
            let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
            collaborators.root_style = Some(originating.clone());
            collaborators.pseudo_id = pseudo_id;
            collaborators.originating_style = Some(originating.clone());
            let actual = ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: target,
                    media: &media,
                    html_user_agent_rules: rule_set("", &media),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
                },
                parent: Some(originating.clone()),
                collaborators,
            })
            .unwrap_or_else(|fallbacks| {
                panic!("unexpected {pseudo_id:?} base-style fallbacks: {fallbacks:?}")
            });
            let actual = ProductionRuleMatchingHost::style(actual.as_ref());
            assert_eq!(actual.StyleType(), pseudo_id);
            assert_eq!(actual.GetFontDescription().ComputedSize(), expected_size);
        }
    }

    #[test]
    fn translated_pseudo_starting_style_uses_the_pseudo_old_style_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            em_size: 16.0,
            ..Default::default()
        };
        let css = "@starting-style{.host::first-letter{font-size:31px}}\
                   .host::first-letter{opacity:.5}";
        let mut owner =
            html::html_parser::ParseHTML("<!doctype html><div id=target class=host>fixture</div>");
        let target = element_with_id(owner.GetDocument(), "target");
        // The originating element already has an old style, but its newly
        // generated ::first-letter does not. Chromium keys the starting-style
        // gate to the request's old style, so the starting declaration applies.
        owner
            .GetDocumentMut()
            .SetResolvedStyle(target, Default::default());
        let originating = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let resolve = |document: &Document| {
            let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
            collaborators.pseudo_id = PseudoId::kPseudoIdFirstLetter;
            collaborators.originating_style = Some(originating.clone());
            ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document,
                    element_index: target,
                    media: &media,
                    html_user_agent_rules: rule_set("", &media),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
                },
                parent: Some(originating.clone()),
                collaborators,
            })
            .unwrap_or_else(|fallbacks| panic!("unexpected pseudo fallbacks: {fallbacks:?}"))
        };
        let initial = resolve(owner.GetDocument());
        let initial = ProductionRuleMatchingHost::style(initial.as_ref());
        assert_eq!(initial.GetFontDescription().ComputedSize(), 31.0);
        assert_eq!(initial.Opacity(), 0.5);

        let mut old = dom::persistent_document::ResolvedNodeStyle::default();
        old.first_letter = Some(Default::default());
        owner.GetDocumentMut().SetResolvedStyle(target, old);
        let subsequent = resolve(owner.GetDocument());
        let subsequent = ProductionRuleMatchingHost::style(subsequent.as_ref());
        assert_eq!(subsequent.GetFontDescription().ComputedSize(), 16.0);
        assert_eq!(subsequent.Opacity(), 0.5);
    }

    #[test]
    fn translated_apply_base_style_matches_retained_ua_shadow_placeholder() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><input id=target class=host placeholder=fixture>",
        );
        let target = element_with_id(owner.GetDocument(), "target");
        let originating = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.pseudo_id = PseudoId::kPseudoIdPlaceholder;
        collaborators.originating_style = Some(originating.clone());
        let actual = ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: target,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set(
                    ".other::placeholder{font-size:44px}\
                     .host::first-letter{font-size:43px}\
                     .host::placeholder{font-size:30px;font-style:italic}",
                    &media,
                )]),
            },
            parent: Some(originating.clone()),
            collaborators,
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected placeholder fallbacks: {fallbacks:?}"));
        let actual = ProductionRuleMatchingHost::style(actual.as_ref());
        assert_eq!(actual.StyleType(), PseudoId::kPseudoIdPlaceholder);
        assert_eq!(actual.GetFontDescription().ComputedSize(), 30.0);
        assert_eq!(
            actual.GetFontDescription().Style(),
            FontSelectionValue::from_int(14)
        );
    }

    #[test]
    fn retained_ua_shadow_names_and_media_host_reach_source_callbacks() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner =
            html::html_parser::ParseHTML("<!doctype html><input id=target placeholder=fixture>");
        let target = element_with_id(owner.GetDocument(), "target");
        let placeholder = owner
            .GetDocument()
            .UAShadowPseudoElement(target, "-webkit-input-placeholder")
            .expect("placeholder must retain its UA-shadow implementation element");
        let element = ProductionResolverElement::ForNode(
            owner.GetDocument(),
            placeholder,
            &media.media_type,
            true,
        );
        let host = ProductionRuleMatchingHost::new(
            owner.GetDocument(),
            &media,
            rule_set("", &media),
            Vec::new(),
        );
        assert_eq!(
            host.PseudoIdForShadowElementName(host.ShadowPseudoId(&element)),
            PseudoId::kPseudoIdPlaceholder
        );
        for (name, expected) in [
            (
                "-webkit-file-upload-button",
                PseudoId::kPseudoIdFileSelectorButton,
            ),
            ("details-content", PseudoId::kPseudoIdDetailsContent),
            ("permission-icon", PseudoId::kPseudoIdPermissionIcon),
            ("picker(select)", PseudoId::kPseudoIdPickerSelect),
            ("select-listbox", PseudoId::kPseudoIdSelectListbox),
        ] {
            assert_eq!(
                host.PseudoIdForShadowElementName(&AtomicString::from_str(name)),
                expected
            );
        }
        assert_eq!(
            host.PseudoIdForShadowElementName(&AtomicString::from_str(
                "-webkit-media-controls-play-button"
            )),
            PseudoId::kPseudoIdNone
        );

        let mut media_owner =
            html::html_parser::ParseHTML("<!doctype html><video id=media></video>");
        let media_host = element_with_id(media_owner.GetDocument(), "media");
        let control = {
            let document = media_owner.GetDocumentMut();
            let root = document.AttachShadowRoot(media_host, DOMShadowRootMode::kUserAgent);
            let control = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
            document.AppendChild(root, control);
            control
        };
        assert!(media_owner.GetDocument().IsMediaElement(media_host));
        for forced_colors in [ForcedColors::kNone, ForcedColors::kActive] {
            let media = MediaValuesCachedData {
                forced_colors,
                ..media.clone()
            };
            MatchAllRulesForElement(ProductionRuleMatchingInputs {
                document: media_owner.GetDocument(),
                element_index: control,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(Vec::new()),
            })
            .unwrap_or_else(|fallbacks| {
                panic!("media UA-shadow rule-set selection fell back: {fallbacks:?}")
            });
        }
    }

    #[test]
    fn non_html_base_style_reaches_specialized_ua_owner_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner = html::html_parser::ParseHTML(
            "<!doctype html><svg id=svg></svg><math id=math><mi>x</mi></math>",
        );
        let parent = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        for (id, expected) in [
            ("svg", "svg-user-agent-rules"),
            ("math", "mathml-user-agent-rules"),
        ] {
            let target = element_with_id(owner.GetDocument(), id);
            let result = ApplyBaseStyleForElement(ProductionBaseStyleInputs {
                matching: ProductionRuleMatchingInputs {
                    document: owner.GetDocument(),
                    element_index: target,
                    media: &media,
                    html_user_agent_rules: rule_set("", &media),
                    svg_user_agent_rules: None,
                    mathml_user_agent_rules: None,
                    author_rule_sets: Rc::new(Vec::new()),
                },
                parent: Some(parent.clone()),
                collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
            });
            let fallbacks = match result {
                Ok(_) => panic!("missing specialized UA resources must keep the trial atomic"),
                Err(fallbacks) => fallbacks,
            };
            assert_eq!(
                fallbacks,
                vec![ProductionRuleMatchingFallback::Unsupported(expected)]
            );
            assert!(
                !fallbacks.contains(&ProductionRuleMatchingFallback::Unsupported(
                    "base:unsupported-element-namespace"
                ))
            );
        }
    }

    #[test]
    fn translated_apply_base_style_reuses_custom_env_and_attr_collaborators() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        };
        let css = "#target { --fixture-width:37px; width:var(--fixture-width); \
                   height:env(fixture-size); min-width:attr(data-size px) }";
        let owner =
            html::html_parser::ParseHTML("<!doctype html><div id=target data-size=43></div>");
        let target = element_with_id(owner.GetDocument(), "target");
        let environment = TestEnvironment(variable_data("41px"));
        let attributes = TestAttributes;
        let parent = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.root_style = Some(parent.clone());
        collaborators.substitution_context = SubstitutionContext {
            environment: Some(&environment),
            attributes: Some(&attributes),
        };

        let output = ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: target,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
            },
            parent: Some(parent),
            collaborators,
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected collaborator fallback: {fallbacks:?}"));
        assert_eq!(
            output
                .custom_properties
                .Get("--fixture-width")
                .expect("base style must return the custom map from its own cascade")
                .Serialize()
                .Utf8(),
            "37px"
        );
        let actual = ProductionRuleMatchingHost::style(output.as_ref());
        assert_eq!(actual.Width().Pixels(), 37.0);
        assert_eq!(actual.Height().Pixels(), 41.0);
        assert_eq!(actual.MinWidth().Pixels(), 43.0);
        assert!(actual.HasVariableReference());
        assert!(actual.HasEnv());
        assert!(actual.HasAttrFunction());
    }

    #[test]
    fn translated_apply_base_style_reuses_animation_interpolation_owner() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        };
        let css = "#target { width:10px; opacity:.2!important }";
        let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        let target = element_with_id(owner.GetDocument(), "target");
        let target_id = owner.GetDocument().Node(target).Id();
        owner.GetDocumentMut().SetAnimationStyle(
            target_id,
            1,
            crate::ParseCSSDeclarationList("width:40px;opacity:.4"),
        );
        owner.GetDocumentMut().SetAnimationStyle(
            target_id,
            2,
            crate::ParseCSSDeclarationList("width:80px;opacity:.8"),
        );
        let animations = ParseAnimationInterpolations(
            owner.GetDocument().AnimationStyles(target_id),
            CSSParserMode::kHTMLStandardMode,
        )
        .expect("fixture animation samples must retain typed values");
        let parent = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.root_style = Some(parent.clone());
        collaborators.animations = Some(&animations);

        let actual = ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: target,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
            },
            parent: Some(parent),
            collaborators,
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected interpolation fallback: {fallbacks:?}"));
        let actual = ProductionRuleMatchingHost::style(actual.as_ref());
        assert_eq!(actual.Width().Pixels(), 80.0);
        assert_eq!(actual.Opacity(), 0.2, "author !important beats animation");
        let base_data = actual.BaseData();
        assert!(!base_data.is_null(), "animated style retains StyleBaseData");
        let base_data = unsafe { &*base_data };
        let base_style = unsafe { &*base_data.GetBaseComputedStyle() };
        assert_eq!(base_style.Width().Pixels(), 10.0);
        assert_eq!(base_style.Opacity(), 0.2);
        let important = base_data.GetBaseImportantSet();
        assert!(!important.is_null());
        assert!(unsafe { &*important }.Has(CSSPropertyID::kOpacity));
    }

    #[test]
    fn animated_z_index_auto_recomputes_context_and_retains_unanimated_base() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        };
        let css = "#target { position:relative; z-index:1 }";
        let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        let target = element_with_id(owner.GetDocument(), "target");
        let target_id = owner.GetDocument().Node(target).Id();
        owner.GetDocumentMut().SetAnimationStyle(
            target_id,
            1,
            crate::ParseCSSDeclarationList("z-index:2"),
        );
        owner.GetDocumentMut().SetAnimationStyle(
            target_id,
            2,
            crate::ParseCSSDeclarationList("z-index:auto"),
        );
        let animations = ParseAnimationInterpolations(
            owner.GetDocument().AnimationStyles(target_id),
            CSSParserMode::kHTMLStandardMode,
        )
        .expect("fixture animation samples must retain typed values");
        let parent = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.root_style = Some(parent.clone());
        collaborators.animations = Some(&animations);

        let actual = ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: target,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
            },
            parent: Some(parent),
            collaborators,
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected interpolation fallback: {fallbacks:?}"));
        let actual = ProductionRuleMatchingHost::style(actual.as_ref());
        assert!(actual.AllowsZIndex());
        assert!(actual.HasAutoZIndex());
        assert!(!actual.ForcesStackingContext());
        let base_data = unsafe { &*actual.BaseData() };
        let base_style = unsafe { &*base_data.GetBaseComputedStyle() };
        assert_eq!(base_style.ZIndex(), 1);
        assert!(base_style.AllowsZIndex());
        assert!(base_style.ForcesStackingContext());
    }

    #[test]
    fn translated_resolve_base_style_uses_typed_parent_overrides() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData::default();
        let owner =
            html::html_parser::ParseHTML("<!doctype html><main><div id=target></div></main>");
        let target = element_with_id(owner.GetDocument(), "target");
        let initial = InitialStyleForDocument(&media, ColorSchemeSettings::default());

        let mut parent_builder =
            ComputedStyleBuilder::from_style(ProductionRuleMatchingHost::style(&initial));
        parent_builder.SetColor(&StyleColor::from_color(foundation::Color::FromRGB(1, 2, 3)));
        let parent = Persistent::from_ptr(parent_builder.TakeStyle().cast_mut());

        let mut layout_parent_builder =
            ComputedStyleBuilder::from_style(ProductionRuleMatchingHost::style(&initial));
        layout_parent_builder.SetDisplay(EDisplay::kFlex);
        let layout_parent = Persistent::from_ptr(layout_parent_builder.TakeStyle().cast_mut());

        let output = ResolveBaseStyleForElement(ProductionResolveBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: target,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set("#target { display:inline }", &media)]),
            },
            parent,
            layout_parent,
            collaborators: ProductionBaseStyleCollaborators::ordinary(media.em_size),
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected ResolveBaseStyle fallback: {fallbacks:?}"));

        let actual = ProductionRuleMatchingHost::style(output.as_ref());
        assert_eq!(
            actual.GetCurrentColor(None),
            foundation::Color::FromRGB(1, 2, 3)
        );
        assert_eq!(actual.Display(), EDisplay::kBlock);
        assert!(actual.BaseData().is_null());
        assert_eq!(output.metrics.matched_property_cache_hits, 0);
        assert_eq!(output.metrics.matched_property_cache_adds, 1);
    }

    #[test]
    fn production_resolver_applies_chromium_forced_colors_adjustment() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            forced_colors: ForcedColors::kActive,
            preferred_color_scheme: PreferredColorScheme::kDark,
            em_size: 16.0,
            ..Default::default()
        };
        let owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
        let target = element_with_id(owner.GetDocument(), "target");
        let parent = InitialStyleForDocument(&media, ColorSchemeSettings::default());
        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(media.em_size);
        collaborators.forced_colors_user_agent_rules = Some(rule_set("", &media));
        let output = ResolveBaseStyleForElement(ProductionResolveBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: target,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set(
                    "#target { box-shadow:1px 2px 3px red; \
                     text-shadow:1px 2px blue; accent-color:red; \
                     color-scheme:dark; background-image:linear-gradient(red, blue) }",
                    &media,
                )]),
            },
            parent: parent.clone(),
            layout_parent: parent,
            collaborators,
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected forced-colors fallback: {fallbacks:?}"));
        let style = ProductionRuleMatchingHost::style(output.as_ref());

        assert!(style.InForcedColorsMode());
        assert_eq!(style.ForcedColorAdjust(), EForcedColorAdjust::kAuto);
        assert!(style.BoxShadow().is_null());
        assert!(style.TextShadow().is_null());
        assert!(style.ScrollbarColor().is_null());
        assert!(style.AccentColor().IsAutoColor());
        assert!(!style.HasBackgroundImage());
        assert_eq!(
            style
                .ColorScheme()
                .iter()
                .map(|scheme| scheme.Utf8())
                .collect::<Vec<_>>(),
            vec!["light", "dark"]
        );
        assert_eq!(
            style.InternalForcedBackgroundColor().GetColor(),
            foundation::Color::FromRGBA32(0xFF121212)
        );
        assert_eq!(
            style.InternalForcedColor().GetColor(),
            foundation::Color::FromRGBA32(0xFFFFFFFF)
        );
        assert_eq!(
            style.InternalForcedVisitedColor().GetColor(),
            foundation::Color::FromRGBA32(0xFFFFFFFF)
        );
    }

    #[test]
    fn translated_apply_base_style_accepts_document_element_collaborators() {
        let _heap = foundation::LayoutHeapScope::new();
        let media = MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        };
        let owner = html::html_parser::ParseHTML("<!doctype html><body></body>");
        let root = owner
            .GetDocument()
            .Node(owner.GetDocument().Root())
            .Children()
            .iter()
            .copied()
            .find(|&index| owner.GetDocument().Node(index).Type() == DOMNodeType::kElement)
            .expect("fixture document element must exist");
        let css = "html { display:block; font-size:20px; width:2rem }";
        let mut collaborators = ProductionBaseStyleCollaborators::ordinary(16.0);
        collaborators.is_document_element = true;

        let actual = ApplyBaseStyleNoCacheForElement(ProductionBaseStyleInputs {
            matching: ProductionRuleMatchingInputs {
                document: owner.GetDocument(),
                element_index: root,
                media: &media,
                html_user_agent_rules: rule_set("", &media),
                svg_user_agent_rules: None,
                mathml_user_agent_rules: None,
                author_rule_sets: Rc::new(vec![rule_set(css, &media)]),
            },
            parent: None,
            collaborators,
        })
        .unwrap_or_else(|fallbacks| panic!("unexpected document-element fallback: {fallbacks:?}"));
        let actual = ProductionRuleMatchingHost::style(actual.as_ref());
        assert_eq!(actual.GetFontDescription().ComputedSize(), 20.0);
        assert_eq!(actual.Width().Pixels(), 32.0);
    }
}

/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003-2011 Apple Inc. and Chromium contributors.
 * GNU Library General Public License version 2 or later; see COPYING.LIB.
 */
// cpp: third_party/blink/renderer/core/css/css_default_style_sheets.h
// cpp: third_party/blink/renderer/core/css/css_default_style_sheets.cc
// Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   css_default_style_sheets.h:  227 / 124 / 105 / 19 / 0.
//   css_default_style_sheets.cc: 762 / 490 / 412 / 78 / 0.
// Effective excludes copyright/comments/blank/preprocessor/include/namespace
// lines and lines consisting only of braces/parentheses/semicolons. Every
// production declaration and statement in these two files is mapped.
// Header omissions: 37-42,45-46,56-57,122-126,147,155,160,162 (forward/access,
// Oilpan/deleted/default loader boilerplate, Trace and debug-only verification).
// Implementation omissions: 106 leak annotation; VerifyUniversalRuleCount
// body 203-266 and its calls 170,293,372,613,632; debug assertions 477-479,
// 599-600; Trace 723-755. Only effective lines within these ranges are counted.
// StyleSheetContents, CSSParser, RuleSet, MixinMap and RuleSetGroup are reused
// directly. Platform resource text/theme/settings/DOM and the existing
// RuleSetBackend::MediaQueryEvaluator owner remain required typed dependencies.
// This source version has screen, ua-forced-colors and frame evaluators; there
// is no separate print RuleSet/evaluator path in css_default_style_sheets.h/cc.
#![allow(non_snake_case, non_camel_case_types)]

use crate::element_rule_collector::{ElementRuleCollectorBackend, RuleSetGroup};
use crate::parser::css_parser::{CSSParser, CSSParserBackend};
use crate::parser::css_parser_context::{CSSParserContext, SecureContextMode};
use crate::parser::css_parser_mode::{CSSDeferPropertyParsing, CSSParserMode};
use crate::properties::css_property::CSSProperty;
use crate::rule_set::{RuleSet, RuleSetIngestionBackend};
use crate::style_sheet_contents::{
    MixinMap, RuleSetHandle, StyleSheetContents, StyleSheetContentsBackend,
};
use foundation::{CSSPropertyID, String};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UAResource {
    Html,
    Quirks,
    Svg,
    Mathml,
    CapabilityElement,
    ScrollButton,
    ScrollMarker,
    Overscroll,
    Marker,
    Transition,
    Skeleton,
    Fullscreen,
    ThemeForcedColors,
    ViewSource,
    JsonDocument,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UAElementClass {
    SVG,
    MathML,
    Video,
    Audio,
    Capability,
    UserMedia,
    Camera,
    Microphone,
    Geolocation,
    Install,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UAFeature {
    UserMediaElement,
    CameraAndMicrophoneElements,
    GeolocationElement,
    InstallElement,
}
#[derive(Clone, Copy)]
enum NamespaceType {
    kHTML,
    kMathML,
    kSVG,
    kMediaControls,
}

/// Actual platform resources/theme/settings, DOM and media evaluator owners.
/// RuleSet and StyleSheetContents algorithms are invoked directly below.
pub trait CSSDefaultStyleSheetsBackend:
    StyleSheetContentsBackend + RuleSetIngestionBackend + ElementRuleCollectorBackend
{
    type Settings;
    type ColorParser: CSSParserBackend;
    fn UncompressResourceAsASCIIString(resource: UAResource) -> String;
    fn ExtraDefaultStyleSheet() -> String;
    fn ExtraFullscreenStyleSheet() -> String;
    fn IsTopChromeWebUI() -> bool;
    fn MediaEvaluatorForType(media_type: &str) -> Self::MediaQueryEvaluator;
    fn MediaEvaluatorForElementFrame(element: &Self::Element) -> Self::MediaQueryEvaluator;
    fn ElementIs(element: &Self::Element, class: UAElementClass) -> bool;
    fn FeatureEnabled(element: &Self::Element, feature: UAFeature) -> bool;
    fn ForcedColorsEnabled() -> bool;
    fn DocumentSettings(element: &Self::Element) -> Option<Rc<Self::Settings>>;
    fn TextTrackWindowColor(settings: &Self::Settings) -> String;
    fn TextTrackWindowRadius(settings: &Self::Settings) -> String;
    fn TextTrackBackgroundColor(settings: &Self::Settings) -> String;
    fn TextTrackFontFamily(settings: &Self::Settings) -> String;
    fn TextTrackFontStyle(settings: &Self::Settings) -> String;
    fn TextTrackFontVariant(settings: &Self::Settings) -> String;
    fn TextTrackTextColor(settings: &Self::Settings) -> String;
    fn TextTrackTextShadow(settings: &Self::Settings) -> String;
    fn TextTrackTextSize(settings: &Self::Settings) -> String;
    fn NewColor() -> <Self::ColorParser as CSSParserBackend>::Color;
    fn ColorAlpha(color: &<Self::ColorParser as CSSParserBackend>::Color) -> f32;
    fn IsViewSource(document: &Self::Document) -> bool;
    fn IsJSONDocument(document: &Self::Document) -> bool;
    fn MergeFeatures(target: &mut Self::RuleFeatureSet, other: &Self::RuleFeatureSet);
}

pub trait UAStyleSheetLoader {
    fn GetUAStyleSheet(&mut self) -> String;
}

// Rendering state is thread-affine, matching the surrounding CSS/DOM modules.
// Registries retain the source singleton/evaluators and their object identity.
thread_local! {
    static INSTANCES: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new());
    static EVALUATORS: RefCell<HashMap<(TypeId, u8), Box<dyn Any>>> = RefCell::new(HashMap::new());
}

pub struct CSSDefaultStyleSheets<B: CSSDefaultStyleSheetsBackend> {
    default_html_style_: RuleSetHandle<B>,
    default_mathml_style_: RuleSetHandle<B>,
    default_svg_style_: RuleSetHandle<B>,
    default_html_quirks_style_: RuleSetHandle<B>,
    default_view_source_style_: Option<RuleSetHandle<B>>,
    default_forced_color_style_: Option<RuleSetHandle<B>>,
    default_pseudo_element_style_: Option<RuleSetHandle<B>>,
    default_media_controls_style_: RuleSetHandle<B>,
    default_fullscreen_style_: RuleSetHandle<B>,
    default_json_document_style_: Option<RuleSetHandle<B>>,
    default_forced_colors_media_controls_style_: Option<RuleSetHandle<B>>,
    default_style_sheet_: Rc<StyleSheetContents<B>>,
    quirks_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    svg_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    mathml_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    media_controls_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    permission_element_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    text_track_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    fullscreen_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    marker_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    scroll_button_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    scroll_marker_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    overscroll_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    forced_colors_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    view_source_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    json_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    default_view_transition_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    skeleton_style_sheet_: Option<Rc<StyleSheetContents<B>>>,
    media_controls_style_sheet_loader_: Option<Box<dyn UAStyleSheetLoader>>,
    rule_set_group_cache_: Vec<(u32, RuleSetGroup<B>)>,
}

fn StrCat(strings: &[String]) -> String {
    let mut text = Vec::new();
    for s in strings {
        text.extend_from_slice(s.Span16().unwrap_or_default());
    }
    String::from_utf16(&text)
}
fn MaybeRemoveCSSImportant(value: String) -> String {
    let suffix: Vec<_> = " !important".encode_utf16().collect();
    let units = value.Span16().unwrap_or_default();
    if units.ends_with(&suffix) {
        String::from_utf16(&units[..units.len() - suffix.len()])
    } else {
        value
    }
}
fn AddTextTrackCSSProperties(builder: &mut Vec<u16>, property: CSSPropertyID, value: String) {
    builder.extend(CSSProperty::Get(property).GetPropertyName().encode_utf16());
    builder.extend(": ".encode_utf16());
    builder.extend_from_slice(value.Span16().unwrap_or_default());
    builder.extend("; ".encode_utf16());
}

impl<B: CSSDefaultStyleSheetsBackend> CSSDefaultStyleSheets<B>
where
    Self: 'static,
    B::MediaQueryEvaluator: 'static,
{
    pub fn Instance() -> Rc<RefCell<Self>> {
        INSTANCES.with(|instances| {
            let mut instances = instances.borrow_mut();
            let entry = instances
                .entry(TypeId::of::<B>())
                .or_insert_with(|| Box::new(Rc::new(RefCell::new(Self::new()))));
            entry
                .downcast_ref::<Rc<RefCell<Self>>>()
                .expect("UA stylesheet singleton type")
                .clone()
        })
    }
    pub fn Init() {
        Self::Instance();
    }
    fn Evaluator(key: u8, media_type: &str) -> Rc<B::MediaQueryEvaluator> {
        EVALUATORS.with(|evaluators| {
            let mut evaluators = evaluators.borrow_mut();
            evaluators
                .entry((TypeId::of::<B>(), key))
                .or_insert_with(|| Box::new(Rc::new(B::MediaEvaluatorForType(media_type))))
                .downcast_ref::<Rc<B::MediaQueryEvaluator>>()
                .expect("UA media evaluator type")
                .clone()
        })
    }
    pub fn ScreenEval() -> Rc<B::MediaQueryEvaluator> {
        Self::Evaluator(0, "screen")
    }
    fn ForcedColorsEval() -> Rc<B::MediaQueryEvaluator> {
        Self::Evaluator(1, "ua-forced-colors")
    }
    pub fn ParseUASheet(text: &String) -> Rc<StyleSheetContents<B>> {
        let context = Rc::new(CSSParserContext::<B::Platform>::FromMode(
            CSSParserMode::kUASheetMode,
            SecureContextMode::kInsecureContext,
            None,
        ));
        let sheet = StyleSheetContents::<B>::new(context, String::default(), None);
        sheet.ParseString(text, true, CSSDeferPropertyParsing::kNo);
        sheet
    }
    fn NewRuleSet() -> RuleSetHandle<B> {
        Rc::new(RefCell::new(RuleSet::<B>::new()))
    }
    fn AddRules(
        set: &RuleSetHandle<B>,
        sheet: &StyleSheetContents<B>,
        medium: &B::MediaQueryEvaluator,
    ) {
        set.borrow_mut()
            .AddRulesFromSheet(sheet, medium, &MixinMap::<B>::new(), None, None);
    }
    fn Compact(set: &RuleSetHandle<B>) {
        set.borrow_mut().CompactRulesIfNeeded();
    }
    pub fn new() -> Self {
        let default_rules = StrCat(&[
            B::UncompressResourceAsASCIIString(UAResource::Html),
            B::ExtraDefaultStyleSheet(),
        ]);
        let default_style_sheet_ = Self::ParseUASheet(&default_rules);
        let quirks_style_sheet_ = if !B::IsTopChromeWebUI() {
            Some(Self::ParseUASheet(&B::UncompressResourceAsASCIIString(
                UAResource::Quirks,
            )))
        } else {
            None
        };
        let mut result = Self {
            default_html_style_: Self::NewRuleSet(),
            default_mathml_style_: Self::NewRuleSet(),
            default_svg_style_: Self::NewRuleSet(),
            default_html_quirks_style_: Self::NewRuleSet(),
            default_media_controls_style_: Self::NewRuleSet(),
            default_fullscreen_style_: Self::NewRuleSet(),
            default_view_source_style_: None,
            default_forced_color_style_: None,
            default_pseudo_element_style_: None,
            default_json_document_style_: None,
            default_forced_colors_media_controls_style_: None,
            default_style_sheet_,
            quirks_style_sheet_,
            svg_style_sheet_: None,
            mathml_style_sheet_: None,
            media_controls_style_sheet_: None,
            permission_element_style_sheet_: None,
            text_track_style_sheet_: None,
            fullscreen_style_sheet_: None,
            marker_style_sheet_: None,
            scroll_button_style_sheet_: None,
            scroll_marker_style_sheet_: None,
            overscroll_style_sheet_: None,
            forced_colors_style_sheet_: None,
            view_source_style_sheet_: None,
            json_style_sheet_: None,
            default_view_transition_style_sheet_: None,
            skeleton_style_sheet_: None,
            media_controls_style_sheet_loader_: None,
            rule_set_group_cache_: vec![],
        };
        result.PopulateDefaultStyles();
        result
    }
    fn InitializeDefaultStyles(&mut self) {
        self.default_html_style_ = Self::NewRuleSet();
        self.default_mathml_style_ = Self::NewRuleSet();
        self.default_svg_style_ = Self::NewRuleSet();
        self.default_html_quirks_style_ = Self::NewRuleSet();
        self.default_media_controls_style_ = Self::NewRuleSet();
        self.default_fullscreen_style_ = Self::NewRuleSet();
        self.default_forced_color_style_ = None;
        self.default_pseudo_element_style_ = None;
        self.default_forced_colors_media_controls_style_ = None;
        self.PopulateDefaultStyles();
    }
    fn PopulateDefaultStyles(&mut self) {
        let medium = Self::ScreenEval();
        Self::AddRules(
            &self.default_html_style_,
            &self.default_style_sheet_,
            &medium,
        );
        if let Some(quirks) = self.quirks_style_sheet_.as_ref() {
            Self::AddRules(&self.default_html_quirks_style_, quirks, &medium);
        }
        Self::Compact(&self.default_html_style_);
        Self::Compact(&self.default_html_quirks_style_);
        assert!(
            self.default_html_style_
                .borrow()
                .ViewTransitionRules()
                .is_empty(),
            "@view-transition is not implemented for the UA stylesheet"
        );
    }
    pub fn PrepareForLeakDetection(&mut self) {
        self.Reset();
    }
    fn Reset(&mut self) {
        self.svg_style_sheet_ = None;
        self.mathml_style_sheet_ = None;
        self.media_controls_style_sheet_ = None;
        self.text_track_style_sheet_ = None;
        self.forced_colors_style_sheet_ = None;
        self.fullscreen_style_sheet_ = None;
        self.marker_style_sheet_ = None;
        self.scroll_button_style_sheet_ = None;
        self.scroll_marker_style_sheet_ = None;
        self.overscroll_style_sheet_ = None;
        self.permission_element_style_sheet_ = None;
        self.view_source_style_sheet_ = None;
        self.json_style_sheet_ = None;
        self.default_view_transition_style_sheet_ = None;
        self.skeleton_style_sheet_ = None;
        self.default_style_sheet_ = Self::ParseUASheet(&StrCat(&[
            B::UncompressResourceAsASCIIString(UAResource::Html),
            B::ExtraDefaultStyleSheet(),
        ]));
        self.InitializeDefaultStyles();
        self.default_view_source_style_ = None;
        self.rule_set_group_cache_.clear();
        // Chromium does not clear default_json_document_style_ on this path.
    }
    pub fn ResetTextTrackStyleSheet(&mut self) {
        if self.text_track_style_sheet_.is_none() {
            return;
        }
        self.text_track_style_sheet_ = None;
        self.default_media_controls_style_ = Self::NewRuleSet();
        if let Some(sheet) = self.media_controls_style_sheet_.as_ref() {
            Self::AddRules(
                &self.default_media_controls_style_,
                sheet,
                &Self::ScreenEval(),
            );
        }
        Self::Compact(&self.default_media_controls_style_);
        if self.default_forced_colors_media_controls_style_.is_some() {
            let set = Self::NewRuleSet();
            if let Some(sheet) = self.media_controls_style_sheet_.as_ref() {
                Self::AddRules(&set, sheet, &Self::ForcedColorsEval());
            }
            Self::Compact(&set);
            self.default_forced_colors_media_controls_style_ = Some(set);
        }
        self.rule_set_group_cache_.clear();
    }
    pub fn DefaultViewSourceStyle(&mut self) -> RuleSetHandle<B> {
        if self.default_view_source_style_.is_none() {
            let set = Self::NewRuleSet();
            let sheet =
                Self::ParseUASheet(&B::UncompressResourceAsASCIIString(UAResource::ViewSource));
            Self::AddRules(&set, &sheet, &Self::ScreenEval());
            Self::Compact(&set);
            self.view_source_style_sheet_ = Some(sheet);
            self.default_view_source_style_ = Some(set);
        }
        self.default_view_source_style_.as_ref().unwrap().clone()
    }
    pub fn DefaultJSONDocumentStyle(&mut self) -> RuleSetHandle<B> {
        if self.default_json_document_style_.is_none() {
            let sheet = Self::ParseUASheet(&B::UncompressResourceAsASCIIString(
                UAResource::JsonDocument,
            ));
            let set = Self::NewRuleSet();
            Self::AddRules(&set, &sheet, &Self::ScreenEval());
            Self::Compact(&set);
            self.json_style_sheet_ = Some(sheet);
            self.default_json_document_style_ = Some(set);
        }
        self.default_json_document_style_.as_ref().unwrap().clone()
    }
    fn AddRulesToDefaultStyleSheets(
        &mut self,
        sheet: &StyleSheetContents<B>,
        namespace: NamespaceType,
    ) {
        let set = match namespace {
            NamespaceType::kHTML => &self.default_html_style_,
            NamespaceType::kMathML => &self.default_mathml_style_,
            NamespaceType::kSVG => &self.default_svg_style_,
            NamespaceType::kMediaControls => &self.default_media_controls_style_,
        };
        Self::AddRules(set, sheet, &Self::ScreenEval());
        Self::Compact(set);
        if let Some(forced) = self.default_forced_color_style_.as_ref() {
            let set: &RuleSetHandle<B> = match namespace {
                NamespaceType::kMediaControls => self
                    .default_forced_colors_media_controls_style_
                    .get_or_insert_with(Self::NewRuleSet),
                _ => forced,
            };
            Self::AddRules(set, sheet, &Self::ForcedColorsEval());
            Self::Compact(set);
        }
    }
    pub fn EnsureDefaultStyleSheetsForElement(&mut self, element: &B::Element) -> bool {
        let mut changed = false;
        if B::ElementIs(element, UAElementClass::SVG) && self.svg_style_sheet_.is_none() {
            let sheet = Self::ParseUASheet(&B::UncompressResourceAsASCIIString(UAResource::Svg));
            self.AddRulesToDefaultStyleSheets(&sheet, NamespaceType::kSVG);
            self.svg_style_sheet_ = Some(sheet);
            changed = true;
        }
        if B::ElementIs(element, UAElementClass::MathML) && self.mathml_style_sheet_.is_none() {
            let sheet = Self::ParseUASheet(&B::UncompressResourceAsASCIIString(UAResource::Mathml));
            self.AddRulesToDefaultStyleSheets(&sheet, NamespaceType::kMathML);
            self.mathml_style_sheet_ = Some(sheet);
            changed = true;
        }
        if self.media_controls_style_sheet_.is_none()
            && self.HasMediaControlsStyleSheetLoader()
            && (B::ElementIs(element, UAElementClass::Video)
                || B::ElementIs(element, UAElementClass::Audio))
        {
            let text = self
                .media_controls_style_sheet_loader_
                .as_mut()
                .unwrap()
                .GetUAStyleSheet();
            let sheet = Self::ParseUASheet(&text);
            self.AddRulesToDefaultStyleSheets(&sheet, NamespaceType::kMediaControls);
            self.media_controls_style_sheet_ = Some(sheet);
            changed = true;
        }
        if self.permission_element_style_sheet_.is_none()
            && B::ElementIs(element, UAElementClass::Capability)
        {
            assert!(
                (B::FeatureEnabled(element, UAFeature::UserMediaElement)
                    && B::ElementIs(element, UAElementClass::UserMedia))
                    || (B::FeatureEnabled(element, UAFeature::CameraAndMicrophoneElements)
                        && (B::ElementIs(element, UAElementClass::Camera)
                            || B::ElementIs(element, UAElementClass::Microphone)))
                    || (B::FeatureEnabled(element, UAFeature::GeolocationElement)
                        && B::ElementIs(element, UAElementClass::Geolocation))
                    || (B::FeatureEnabled(element, UAFeature::InstallElement)
                        && B::ElementIs(element, UAElementClass::Install))
            );
            let sheet = Self::ParseUASheet(&B::UncompressResourceAsASCIIString(
                UAResource::CapabilityElement,
            ));
            self.AddRulesToDefaultStyleSheets(&sheet, NamespaceType::kHTML);
            self.permission_element_style_sheet_ = Some(sheet);
            changed = true;
        }
        if self.text_track_style_sheet_.is_none() && B::ElementIs(element, UAElementClass::Video) {
            if let Some(settings) = B::DocumentSettings(element) {
                let mut builder: Vec<u16> = "video::cue { ".encode_utf16().collect();
                let mut color = B::NewColor();
                if CSSParser::<B::ColorParser>::ParseColor(
                    &mut color,
                    &MaybeRemoveCSSImportant(B::TextTrackWindowColor(&settings)),
                    true,
                ) && B::ColorAlpha(&color) > 0.0
                {
                    AddTextTrackCSSProperties(
                        &mut builder,
                        CSSPropertyID::kBackgroundColor,
                        B::TextTrackWindowColor(&settings),
                    );
                    AddTextTrackCSSProperties(
                        &mut builder,
                        CSSPropertyID::kBorderRadius,
                        B::TextTrackWindowRadius(&settings),
                    );
                } else {
                    AddTextTrackCSSProperties(
                        &mut builder,
                        CSSPropertyID::kBackgroundColor,
                        B::TextTrackBackgroundColor(&settings),
                    );
                }
                AddTextTrackCSSProperties(
                    &mut builder,
                    CSSPropertyID::kFontFamily,
                    B::TextTrackFontFamily(&settings),
                );
                AddTextTrackCSSProperties(
                    &mut builder,
                    CSSPropertyID::kFontStyle,
                    B::TextTrackFontStyle(&settings),
                );
                AddTextTrackCSSProperties(
                    &mut builder,
                    CSSPropertyID::kFontVariant,
                    B::TextTrackFontVariant(&settings),
                );
                AddTextTrackCSSProperties(
                    &mut builder,
                    CSSPropertyID::kColor,
                    B::TextTrackTextColor(&settings),
                );
                AddTextTrackCSSProperties(
                    &mut builder,
                    CSSPropertyID::kTextShadow,
                    B::TextTrackTextShadow(&settings),
                );
                AddTextTrackCSSProperties(
                    &mut builder,
                    CSSPropertyID::kFontSize,
                    B::TextTrackTextSize(&settings),
                );
                builder.extend(" } ".encode_utf16());
                let sheet = Self::ParseUASheet(&String::from_utf16(&builder));
                self.AddRulesToDefaultStyleSheets(&sheet, NamespaceType::kMediaControls);
                self.text_track_style_sheet_ = Some(sheet);
                changed = true;
            }
        }
        if changed {
            self.rule_set_group_cache_.clear();
        }
        changed
    }
    pub fn EnsureDefaultStyleSheetsForPseudoElement(&mut self, pseudo: PseudoId) -> bool {
        let (resource, slot, invalidate) = match pseudo {
            PseudoId::kPseudoIdScrollButtonBlockStart
            | PseudoId::kPseudoIdScrollButtonInlineStart
            | PseudoId::kPseudoIdScrollButtonInlineEnd
            | PseudoId::kPseudoIdScrollButtonBlockEnd => (
                UAResource::ScrollButton,
                &mut self.scroll_button_style_sheet_,
                true,
            ),
            PseudoId::kPseudoIdScrollMarker => (
                UAResource::ScrollMarker,
                &mut self.scroll_marker_style_sheet_,
                true,
            ),
            PseudoId::kPseudoIdOverscrollAreaParent => (
                UAResource::Overscroll,
                &mut self.overscroll_style_sheet_,
                false,
            ),
            PseudoId::kPseudoIdMarker => (UAResource::Marker, &mut self.marker_style_sheet_, false),
            PseudoId::kPseudoIdViewTransition
            | PseudoId::kPseudoIdViewTransitionGroup
            | PseudoId::kPseudoIdViewTransitionGroupChildren
            | PseudoId::kPseudoIdViewTransitionImagePair
            | PseudoId::kPseudoIdViewTransitionOld
            | PseudoId::kPseudoIdViewTransitionNew => (
                UAResource::Transition,
                &mut self.default_view_transition_style_sheet_,
                false,
            ),
            PseudoId::kPseudoIdSkeleton => {
                (UAResource::Skeleton, &mut self.skeleton_style_sheet_, false)
            }
            _ => return false,
        };
        if slot.is_some() {
            return false;
        }
        let sheet = Self::ParseUASheet(&B::UncompressResourceAsASCIIString(resource));
        *slot = Some(sheet.clone());
        let set = self
            .default_pseudo_element_style_
            .get_or_insert_with(Self::NewRuleSet);
        Self::AddRules(set, &sheet, &Self::ScreenEval());
        Self::Compact(set);
        if invalidate {
            self.rule_set_group_cache_.clear();
        }
        true
    }
    pub fn SetMediaControlsStyleSheetLoader(
        &mut self,
        loader: Option<Box<dyn UAStyleSheetLoader>>,
    ) {
        let old = std::mem::replace(&mut self.media_controls_style_sheet_loader_, loader);
        drop(old);
    }
    pub fn HasMediaControlsStyleSheetLoader(&self) -> bool {
        self.media_controls_style_sheet_loader_.is_some()
    }
    pub fn EnsureDefaultStyleSheetForFullscreen(&mut self, element: &B::Element) {
        if self.fullscreen_style_sheet_.is_some() {
            return;
        }
        let rules = StrCat(&[
            B::UncompressResourceAsASCIIString(UAResource::Fullscreen),
            B::ExtraFullscreenStyleSheet(),
        ]);
        let sheet = Self::ParseUASheet(&rules);
        self.fullscreen_style_sheet_ = Some(sheet.clone());
        Self::AddRules(
            &self.default_fullscreen_style_,
            &sheet,
            &B::MediaEvaluatorForElementFrame(element),
        );
        Self::Compact(&self.default_fullscreen_style_);
    }
    pub fn RebuildFullscreenRuleSetIfMediaQueriesChanged(&mut self, element: &B::Element) {
        let Some(sheet) = self.fullscreen_style_sheet_.as_ref() else {
            return;
        };
        let medium = B::MediaEvaluatorForElementFrame(element);
        if !self
            .default_fullscreen_style_
            .borrow()
            .DidMediaQueryResultsChange(&medium)
        {
            return;
        }
        let set = Self::NewRuleSet();
        Self::AddRules(&set, sheet, &medium);
        Self::Compact(&set);
        self.default_fullscreen_style_ = set;
        self.rule_set_group_cache_.clear();
    }
    pub fn EnsureDefaultStyleSheetForForcedColors(&mut self) -> bool {
        if self.forced_colors_style_sheet_.is_some() {
            return false;
        }
        let rules = if B::ForcedColorsEnabled() {
            B::UncompressResourceAsASCIIString(UAResource::ThemeForcedColors)
        } else {
            String::default()
        };
        let sheet = Self::ParseUASheet(&rules);
        self.forced_colors_style_sheet_ = Some(sheet.clone());
        let set = self
            .default_forced_color_style_
            .get_or_insert_with(Self::NewRuleSet);
        let medium = Self::ForcedColorsEval();
        Self::AddRules(set, &self.default_style_sheet_, &medium);
        Self::AddRules(set, &sheet, &medium);
        if let Some(svg) = self.svg_style_sheet_.as_ref() {
            Self::AddRules(set, svg, &medium);
        }
        Self::Compact(set);
        if let Some(media) = self.media_controls_style_sheet_.as_ref() {
            assert!(self.default_forced_colors_media_controls_style_.is_none());
            let set = Self::NewRuleSet();
            Self::AddRules(&set, media, &medium);
            Self::Compact(&set);
            self.default_forced_colors_media_controls_style_ = Some(set);
        }
        true
    }
    pub fn CollectFeaturesTo(&mut self, document: &B::Document, features: &mut B::RuleFeatureSet) {
        self.ForEachRuleFeatureSet(document, false, |other, _| {
            B::MergeFeatures(features, other)
        });
    }
    pub fn ForEachRuleFeatureSet(
        &mut self,
        document: &B::Document,
        each_sheet: bool,
        mut func: impl FnMut(&B::RuleFeatureSet, Option<&Rc<StyleSheetContents<B>>>),
    ) {
        {
            let rules = self.default_html_style_.borrow();
            func(rules.Features(), Some(&self.default_style_sheet_));
            if each_sheet {
                if let Some(sheet) = self.permission_element_style_sheet_.as_ref() {
                    func(rules.Features(), Some(sheet));
                }
            }
        }
        {
            let rules = self.default_media_controls_style_.borrow();
            func(rules.Features(), self.media_controls_style_sheet_.as_ref());
            if each_sheet {
                if let Some(sheet) = self.text_track_style_sheet_.as_ref() {
                    func(rules.Features(), Some(sheet));
                }
            }
        }
        func(
            self.default_mathml_style_.borrow().Features(),
            self.mathml_style_sheet_.as_ref(),
        );
        func(
            self.default_fullscreen_style_.borrow().Features(),
            self.fullscreen_style_sheet_.as_ref(),
        );
        if B::IsViewSource(document) {
            let rules = self.DefaultViewSourceStyle();
            func(
                rules.borrow().Features(),
                self.view_source_style_sheet_.as_ref(),
            );
        }
        if B::IsJSONDocument(document) {
            let rules = self.DefaultJSONDocumentStyle();
            func(rules.borrow().Features(), self.json_style_sheet_.as_ref());
        }
    }
    pub fn DefaultHtmlStyle(&self) -> RuleSetHandle<B> {
        self.default_html_style_.clone()
    }
    pub fn DefaultMathMLStyle(&self) -> RuleSetHandle<B> {
        self.default_mathml_style_.clone()
    }
    pub fn DefaultSVGStyle(&self) -> RuleSetHandle<B> {
        self.default_svg_style_.clone()
    }
    pub fn DefaultHtmlQuirksStyle(&self) -> RuleSetHandle<B> {
        self.default_html_quirks_style_.clone()
    }
    pub fn DefaultMediaControlsStyle(&self) -> RuleSetHandle<B> {
        self.default_media_controls_style_.clone()
    }
    pub fn DefaultFullscreenStyle(&self) -> RuleSetHandle<B> {
        self.default_fullscreen_style_.clone()
    }
    pub fn DefaultForcedColorStyle(&self) -> Option<RuleSetHandle<B>> {
        self.default_forced_color_style_.clone()
    }
    pub fn DefaultPseudoElementStyleOrNull(&self) -> Option<RuleSetHandle<B>> {
        self.default_pseudo_element_style_.clone()
    }
    pub fn DefaultForcedColorsMediaControlsStyle(&self) -> Option<RuleSetHandle<B>> {
        self.default_forced_colors_media_controls_style_.clone()
    }
    pub fn DefaultStyleSheet(&self) -> Rc<StyleSheetContents<B>> {
        self.default_style_sheet_.clone()
    }
    pub fn QuirksStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.quirks_style_sheet_.clone()
    }
    pub fn SvgStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.svg_style_sheet_.clone()
    }
    pub fn MathmlStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.mathml_style_sheet_.clone()
    }
    pub fn MediaControlsStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.media_controls_style_sheet_.clone()
    }
    pub fn FullscreenStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.fullscreen_style_sheet_.clone()
    }
    pub fn MarkerStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.marker_style_sheet_.clone()
    }
    pub fn ScrollButtonStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.scroll_button_style_sheet_.clone()
    }
    pub fn ScrollMarkerStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.scroll_marker_style_sheet_.clone()
    }
    pub fn OverscrollStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.overscroll_style_sheet_.clone()
    }
    pub fn ForcedColorsStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.forced_colors_style_sheet_.clone()
    }
    pub fn DefaultViewTransitionStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.default_view_transition_style_sheet_.clone()
    }
    pub fn SkeletonStyleSheet(&self) -> Option<Rc<StyleSheetContents<B>>> {
        self.skeleton_style_sheet_.clone()
    }
    pub fn RuleSetGroupCache(&mut self) -> &mut Vec<(u32, RuleSetGroup<B>)> {
        &mut self.rule_set_group_cache_
    }
}

pub struct TestingScope<B: CSSDefaultStyleSheetsBackend>(PhantomData<B>)
where
    CSSDefaultStyleSheets<B>: 'static,
    B::MediaQueryEvaluator: 'static;
impl<B: CSSDefaultStyleSheetsBackend> TestingScope<B>
where
    CSSDefaultStyleSheets<B>: 'static,
    B::MediaQueryEvaluator: 'static,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
impl<B: CSSDefaultStyleSheetsBackend> Drop for TestingScope<B>
where
    CSSDefaultStyleSheets<B>: 'static,
    B::MediaQueryEvaluator: 'static,
{
    fn drop(&mut self) {
        CSSDefaultStyleSheets::<B>::Instance().borrow_mut().Reset();
    }
}

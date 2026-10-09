// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Production media-query literal consumer and typed source feature dispatch.
//! Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Literal control-flow mapping: media_query_exp.cc:61-126,145-179,193-197,
//! 254-370,535-562,568-572,580-605. These branches consume actual typed literals;
//! their CSSMathFunctionValue dependencies remain pending rather than mapped.
//! Runtime gates follow runtime_enabled_features.json5 stable-status defaults.
//! Pending dependencies: CSSMathFunctionValue/Consume{Integer,Number,Length,Resolution}
//! math-function branches; custom declarations, anchored/container style functions.
//! Evaluation source ranges/counts are in media_query_evaluator.rs and the
//! exact media_query_evaluator_ledger.tsv. Typed enum dispatch replaces the
//! global function map; new container axes/state/video/custom-media branches
//! use required MediaValues reads and real StyleRuleCustomMedia ownership.
//! Container style/fallback control flow lives in media_query_container.rs;
//! concrete owners/coercion adapters and telemetry omissions are tracked there.
//! Unknown AST stays Kleene Unknown; there is no fallback string parser.
use super::device_posture_provider::DevicePostureType;
use super::display_mode::DisplayMode as DisplayModeValue;
use super::media_query_evaluator::*;
use super::media_query_exp::*;
use super::media_query_set::MediaQuerySet;
use super::media_values::MediaValues;
use super::scripting::Scripting as ScriptingValue;
use super::web_preferences::OutputDeviceUpdateAbilityType;
use super::window_show_state::WindowShowState;
use crate::css_numeric_literal_value::CSSNumericLiteralValue;
use crate::css_primitive_value::UnitType;
use crate::kleene_value::KleeneValue;
use crate::parser::css_parser_token::{CSSParserTokenType::*, NumericValueType};
use crate::parser::css_parser_token_stream::{CSSParserTokenStream, TokenStreamTokenizer};
use crate::parser::media_query_parser::{
    MediaQueryParser, MediaQueryParserBackend, MediaQueryRuntimeFeatures,
};
use crate::resolver::media_query_result::MediaQueryResultFlags;
use foundation::{AtomicString, CSSValueID, StringView};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct MediaQueryFeatureFlags {
    pub custom_media: bool,
    pub scrolled_container_queries: bool,
    pub prefers_reduced_data: bool,
    pub forced_colors: bool,
    pub navigation_controls: bool,
    pub origin_trials_sample_api: bool,
    pub viewport_segments: bool,
    pub device_posture: bool,
    pub inverted_colors: bool,
    pub additional_windowing_controls: bool,
}
impl Default for MediaQueryFeatureFlags {
    fn default() -> Self {
        Self {
            custom_media: false,
            scrolled_container_queries: false,
            prefers_reduced_data: false,
            forced_colors: true,
            navigation_controls: false,
            origin_trials_sample_api: false,
            viewport_segments: true,
            device_posture: true,
            inverted_colors: false,
            additional_windowing_controls: false,
        }
    }
}
impl MediaQueryRuntimeFeatures for MediaQueryFeatureFlags {
    fn CSSCustomMediaEnabled(&self) -> bool {
        self.custom_media
    }
    fn CSSScrolledContainerQueriesEnabled(&self) -> bool {
        self.scrolled_container_queries
    }
    fn PrefersReducedDataEnabled(&self) -> bool {
        self.prefers_reduced_data
    }
    fn ForcedColorsEnabled(&self) -> bool {
        self.forced_colors
    }
    fn MediaQueryNavigationControlsEnabled(&self) -> bool {
        self.navigation_controls
    }
    fn OriginTrialsSampleAPIEnabled(&self) -> bool {
        self.origin_trials_sample_api
    }
    fn ViewportSegmentsEnabled(&self) -> bool {
        self.viewport_segments
    }
    fn DevicePostureEnabled(&self) -> bool {
        self.device_posture
    }
    fn InvertedColorsEnabled(&self) -> bool {
        self.inverted_colors
    }
    fn DesktopPWAsAdditionalWindowingControlsEnabled(&self) -> bool {
        self.additional_windowing_controls
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Feature {
    Width,
    Height,
    InlineSize,
    BlockSize,
    DeviceWidth,
    DeviceHeight,
    AspectRatio,
    DeviceAspectRatio,
    Resolution,
    DevicePixelRatio,
    Color,
    ColorIndex,
    Monochrome,
    Grid,
    Transform3d,
    Orientation,
    Hover,
    AnyHover,
    Pointer,
    AnyPointer,
    ColorGamut,
    PrefersColorScheme,
    PrefersContrast,
    ReducedMotion,
    ReducedData,
    ReducedTransparency,
    ForcedColors,
    NavigationControls,
    Scan,
    DisplayMode,
    DisplayState,
    Resizable,
    DynamicRange,
    VideoDynamicRange,
    Stuck,
    Snapped,
    Scrollable,
    Scrolled,
    HorizontalSegments,
    VerticalSegments,
    OverflowInline,
    OverflowBlock,
    Update,
    DevicePosture,
    Scripting,
    InvertedColors,
    OriginTrial,
}
fn FeatureForName(name: &AtomicString) -> Option<(Feature, MediaQueryOperator)> {
    use Feature::*;
    use MediaQueryOperator::*;
    // Chromium's generated CSS_MEDIAQUERY_NAMES_FOR_EACH_MEDIAFEATURE table.
    Some(match name.Utf8().as_str() {
        "width" => (Width, kNone),
        "min-width" => (Width, kGe),
        "max-width" => (Width, kLe),
        "height" => (Height, kNone),
        "min-height" => (Height, kGe),
        "max-height" => (Height, kLe),
        "inline-size" => (InlineSize, kNone),
        "min-inline-size" => (InlineSize, kGe),
        "max-inline-size" => (InlineSize, kLe),
        "block-size" => (BlockSize, kNone),
        "min-block-size" => (BlockSize, kGe),
        "max-block-size" => (BlockSize, kLe),
        "device-width" => (DeviceWidth, kNone),
        "min-device-width" => (DeviceWidth, kGe),
        "max-device-width" => (DeviceWidth, kLe),
        "device-height" => (DeviceHeight, kNone),
        "min-device-height" => (DeviceHeight, kGe),
        "max-device-height" => (DeviceHeight, kLe),
        "aspect-ratio" => (AspectRatio, kNone),
        "min-aspect-ratio" => (AspectRatio, kGe),
        "max-aspect-ratio" => (AspectRatio, kLe),
        "device-aspect-ratio" => (DeviceAspectRatio, kNone),
        "min-device-aspect-ratio" => (DeviceAspectRatio, kGe),
        "max-device-aspect-ratio" => (DeviceAspectRatio, kLe),
        "resolution" => (Resolution, kNone),
        "min-resolution" => (Resolution, kGe),
        "max-resolution" => (Resolution, kLe),
        "-webkit-device-pixel-ratio" => (DevicePixelRatio, kNone),
        "-webkit-min-device-pixel-ratio" => (DevicePixelRatio, kGe),
        "-webkit-max-device-pixel-ratio" => (DevicePixelRatio, kLe),
        "color" => (Color, kNone),
        "min-color" => (Color, kGe),
        "max-color" => (Color, kLe),
        "color-index" => (ColorIndex, kNone),
        "min-color-index" => (ColorIndex, kGe),
        "max-color-index" => (ColorIndex, kLe),
        "monochrome" => (Monochrome, kNone),
        "min-monochrome" => (Monochrome, kGe),
        "max-monochrome" => (Monochrome, kLe),
        "grid" => (Grid, kNone),
        "-webkit-transform-3d" => (Transform3d, kNone),
        "orientation" => (Orientation, kNone),
        "hover" => (Hover, kNone),
        "any-hover" => (AnyHover, kNone),
        "pointer" => (Pointer, kNone),
        "any-pointer" => (AnyPointer, kNone),
        "color-gamut" => (ColorGamut, kNone),
        "prefers-color-scheme" => (PrefersColorScheme, kNone),
        "prefers-contrast" => (PrefersContrast, kNone),
        "prefers-reduced-motion" => (ReducedMotion, kNone),
        "prefers-reduced-data" => (ReducedData, kNone),
        "prefers-reduced-transparency" => (ReducedTransparency, kNone),
        "forced-colors" => (ForcedColors, kNone),
        "navigation-controls" => (NavigationControls, kNone),
        "scan" => (Scan, kNone),
        "display-mode" => (DisplayMode, kNone),
        "display-state" => (DisplayState, kNone),
        "resizable" => (Resizable, kNone),
        "dynamic-range" => (DynamicRange, kNone),
        "video-dynamic-range" => (VideoDynamicRange, kNone),
        "stuck" => (Stuck, kNone),
        "snapped" => (Snapped, kNone),
        "scrollable" => (Scrollable, kNone),
        "scrolled" => (Scrolled, kNone),
        "horizontal-viewport-segments" => (HorizontalSegments, kNone),
        "vertical-viewport-segments" => (VerticalSegments, kNone),
        "overflow-inline" => (OverflowInline, kNone),
        "overflow-block" => (OverflowBlock, kNone),
        "update" => (Update, kNone),
        "device-posture" => (DevicePosture, kNone),
        "scripting" => (Scripting, kNone),
        "inverted-colors" => (InvertedColors, kNone),
        "origin-trial-test" => (OriginTrial, kNone),
        _ => return None,
    })
}
fn ValidIdent(feature: Feature, id: CSSValueID) -> bool {
    use CSSValueID::*;
    use Feature::*;
    match feature {
        Orientation => matches!(id, kPortrait | kLandscape),
        Pointer | AnyPointer => matches!(id, kNone | kCoarse | kFine),
        Hover | AnyHover => matches!(id, kNone | kHover),
        Scan => matches!(id, kInterlace | kProgressive),
        ColorGamut => matches!(id, kSRGB | kP3 | kRec2020),
        PrefersColorScheme => matches!(id, kDark | kLight),
        PrefersContrast => matches!(id, kNoPreference | kMore | kLess | kCustom),
        ReducedMotion | ReducedData | ReducedTransparency => matches!(id, kNoPreference | kReduce),
        ForcedColors => matches!(id, kNone | kActive),
        NavigationControls => matches!(id, kNone | kBackButton),
        DevicePosture => matches!(id, kContinuous | kFolded),
        OverflowInline => matches!(id, kNone | kScroll),
        OverflowBlock => matches!(id, kNone | kScroll | kPaged),
        Update => matches!(id, kNone | kFast | kSlow),
        Scripting => matches!(id, kNone | kEnabled | kInitialOnly),
        InvertedColors => matches!(id, kNone | kInverted),
        DynamicRange | VideoDynamicRange => matches!(id, kStandard | kHigh),
        Stuck => matches!(
            id,
            kNone
                | kTop
                | kLeft
                | kBottom
                | kRight
                | kBlockStart
                | kBlockEnd
                | kInlineStart
                | kInlineEnd
        ),
        Snapped => matches!(id, kNone | kX | kY | kBlock | kInline | kBoth),
        Scrollable | Scrolled => matches!(
            id,
            kNone
                | kTop
                | kLeft
                | kBottom
                | kRight
                | kBlockStart
                | kBlockEnd
                | kInlineStart
                | kInlineEnd
                | kX
                | kY
                | kBlock
                | kInline
        ),
        DisplayMode => matches!(
            id,
            kFullscreen
                | kUnframed
                | kStandalone
                | kMinimalUi
                | kWindowControlsOverlay
                | kBrowser
                | kTabbed
                | kPictureInPicture
        ),
        DisplayState => matches!(id, kFullscreen | kNormal | kMinimized | kMaximized),
        Resizable => matches!(id, kTrue | kFalse),
        _ => false,
    }
}
fn ConsumeNumeric<T: TokenStreamTokenizer>(
    stream: &mut CSSParserTokenStream<'_, T>,
) -> Option<CSSNumericLiteralValue> {
    let token = stream.Peek();
    let unit = match token.GetType() {
        kNumberToken => {
            if token.GetNumericValueType() == NumericValueType::kIntegerValueType {
                UnitType::kInteger
            } else {
                UnitType::kNumber
            }
        }
        kDimensionToken => token.GetUnitType(),
        _ => return None,
    };
    let value = CSSNumericLiteralValue::Create(token.NumericValue(), unit);
    if !value.IsNumber() && !value.IsLength() && !value.IsResolution() {
        return None;
    }
    stream.ConsumeIncludingWhitespace();
    Some(value)
}
impl MediaQueryParserBackend for MediaQueryFeatureFlags {
    type Value = CSSNumericLiteralValue;
    type UnparsedValue = CSSNumericLiteralValue;
    fn ConsumeValue<T: TokenStreamTokenizer>(
        &mut self,
        name: &AtomicString,
        stream: &mut CSSParserTokenStream<'_, T>,
        _: bool,
    ) -> Option<MediaQueryExpValue<Self::Value>> {
        use Feature::*;
        let (feature, prefix) = FeatureForName(name)?;
        if matches!(feature, ReducedData) && !self.prefers_reduced_data
            || matches!(feature, ForcedColors) && !self.forced_colors
            || matches!(feature, NavigationControls) && !self.navigation_controls
            || matches!(feature, DevicePosture) && !self.device_posture
            || matches!(feature, InvertedColors) && !self.inverted_colors
            || matches!(feature, DisplayState | Resizable) && !self.additional_windowing_controls
            || matches!(feature, HorizontalSegments | VerticalSegments) && !self.viewport_segments
        {
            return None;
        }
        if stream.Peek().GetType() == kIdentToken {
            let id = stream.ConsumeIncludingWhitespace().Id();
            return (prefix == MediaQueryOperator::kNone && ValidIdent(feature, id))
                .then(|| MediaQueryExpValue::FromId(id));
        }
        let value = ConsumeNumeric(stream)?;
        if matches!(feature, AspectRatio | DeviceAspectRatio) {
            if value.DoubleValue() < 0.0 {
                return None;
            }
            let denominator = if stream.Peek().GetType() == kDelimiterToken
                && stream.Peek().Delimiter() == b'/' as u16
            {
                stream.ConsumeIncludingWhitespace();
                let denominator = ConsumeNumeric(stream)?;
                if !denominator.IsNumber() || denominator.DoubleValue() < 0.0 {
                    return None;
                }
                denominator
            } else {
                CSSNumericLiteralValue::Create(1.0, UnitType::kNumber)
            };
            let numerator = if value.DoubleValue() == 0.0 && denominator.DoubleValue() == 0.0 {
                CSSNumericLiteralValue::Create(1.0, UnitType::kNumber)
            } else {
                value
            };
            return Some(MediaQueryExpValue::FromRatio(
                Rc::new(numerator),
                Rc::new(denominator),
            ));
        }
        let valid = match feature {
            Width | Height | InlineSize | BlockSize | DeviceWidth | DeviceHeight => {
                value.IsLength() || value.IsNumber() && value.DoubleValue() == 0.0
            }
            Resolution => value.IsResolution() && value.DoubleValue() >= 0.0,
            Color | ColorIndex | Monochrome | HorizontalSegments | VerticalSegments => {
                value.IsInteger()
            }
            Grid => value.IsInteger() && matches!(value.DoubleValue(), 0.0 | 1.0),
            DevicePixelRatio | Transform3d => value.IsNumber(),
            _ => false,
        };
        valid.then(|| MediaQueryExpValue::FromValue(Rc::new(value)))
    }
    fn UseCountRangeSyntax(&mut self) { /* Source telemetry remains pending. */
    }
}
pub fn ParseMediaQuerySet(text: &str) -> MediaQuerySet<CSSNumericLiteralValue> {
    MediaQueryParser::new(MediaQueryFeatureFlags::default())
        .ParseMediaQuerySet(StringView::from(text))
}
// Custom-media cycles are removed by the rule owner before evaluation, as in
// Chromium. Entries retain the translated StyleRuleCustomMedia owners.
pub type CustomMediaRulesMap = std::collections::HashMap<
    AtomicString,
    Rc<crate::style_rule::StyleRuleCustomMedia<crate::production_style_sheet::Backend>>,
>;

pub struct MediaQueryValueBackend<'a> {
    values: &'a dyn MediaValues,
    features: MediaQueryFeatureFlags,
}
impl<'a> MediaQueryEvaluator<'a, MediaQueryValueBackend<'a>> {
    pub fn ForMediaValues(values: &'a dyn MediaValues) -> Self {
        Self::ForMediaValuesWithFeatures(values, MediaQueryFeatureFlags::default())
    }
    pub fn ForMediaValuesWithFeatures(
        values: &'a dyn MediaValues,
        features: MediaQueryFeatureFlags,
    ) -> Self {
        Self::FromMediaValues(values, MediaQueryValueBackend { values, features })
    }
}
impl MediaQueryEvaluationBackend for MediaQueryValueBackend<'_> {
    type Value = CSSNumericLiteralValue;
    type UnparsedValue = CSSNumericLiteralValue;
    type ResultFlags = MediaQueryResultFlags;
    type CustomMediaRulesMap = CustomMediaRulesMap;
    fn EvalFeature(
        &self,
        feature: &MediaQueryExp<Self::Value>,
        flags: Option<&mut Self::ResultFlags>,
        custom_medias: Option<&CustomMediaRulesMap>,
    ) -> KleeneValue {
        let v = self.values;
        assert!(v.HasValues());
        if (feature.IsWidthDependent() && v.Width().is_none())
            || (feature.IsHeightDependent() && v.Height().is_none())
            || (feature.IsInlineSizeDependent() && v.InlineSize().is_none())
            || (feature.IsBlockSizeDependent() && v.BlockSize().is_none())
        {
            return KleeneValue::kUnknown;
        }
        // cpp: media_query_evaluator.cc:1662-1674,1717-1741.
        if self.features.custom_media
            && feature.IsCustomMedia()
            && feature.MediaFeature().length() >= 3
            && feature.MediaFeature().at(0) == 45
            && feature.MediaFeature().at(1) == 45
        {
            let Some(rule) = custom_medias.and_then(|rules| rules.get(feature.MediaFeature()))
            else {
                return KleeneValue::kUnknown;
            };
            if rule.IsBooleanValue() {
                return if rule.GetBooleanValue() {
                    KleeneValue::kTrue
                } else {
                    KleeneValue::kFalse
                };
            }
            let evaluator = MediaQueryEvaluator::ForMediaValuesWithFeatures(v, self.features);
            return evaluator.EvalCustomMediaSet(rule.GetMediaQueryValue(), flags, custom_medias);
        }
        if feature.IsCustomMedia() {
            return KleeneValue::kUnknown;
        }
        let Some((kind, prefix)) = FeatureForName(feature.MediaFeature()) else {
            return KleeneValue::kFalse;
        };
        if matches!(
            kind,
            Feature::Stuck | Feature::Snapped | Feature::Scrollable | Feature::Scrolled
        ) && !v.HasScrollState()
        {
            return KleeneValue::kUnknown;
        }
        let bounds = feature.Bounds();
        let mut result = true;
        if !bounds.IsRange() || bounds.right.IsValid() {
            result &= EvalLiteral(
                kind,
                &bounds.right.value,
                if prefix != MediaQueryOperator::kNone {
                    prefix
                } else {
                    bounds.right.op
                },
                v,
            );
        }
        if bounds.left.IsValid() {
            result &= EvalLiteral(kind, &bounds.left.value, ReverseOperator(bounds.left.op), v);
        }
        if let Some(flags) = flags {
            flags.is_viewport_dependent |= feature.IsViewportDependent();
            flags.is_device_dependent |= feature.IsDeviceDependent();
            flags.unit_flags |=
                LiteralUnitFlags(&bounds.left.value) | LiteralUnitFlags(&bounds.right.value);
        }
        if result {
            KleeneValue::kTrue
        } else {
            KleeneValue::kFalse
        }
    }
}
fn LiteralUnitFlags(value: &MediaQueryExpValue<CSSNumericLiteralValue>) -> u32 {
    use UnitType::*;
    match value {
        MediaQueryExpValue::Value(value) => match value.GetType() {
            kEms | kExs | kChs | kIcs | kCaps => MediaQueryExpValue::<()>::kFontRelative,
            kRems | kRexs | kRchs | kRics | kRcaps => MediaQueryExpValue::<()>::kRootRelative,
            kLhs => MediaQueryExpValue::<()>::kLineHeightRelative,
            kRlhs => MediaQueryExpValue::<()>::kRootRelative,
            kDynamicViewportWidth
            | kDynamicViewportHeight
            | kDynamicViewportInlineSize
            | kDynamicViewportBlockSize
            | kDynamicViewportMin
            | kDynamicViewportMax => MediaQueryExpValue::<()>::kDynamicViewport,
            kViewportWidth
            | kViewportHeight
            | kViewportInlineSize
            | kViewportBlockSize
            | kViewportMin
            | kViewportMax
            | kSmallViewportWidth
            | kSmallViewportHeight
            | kSmallViewportInlineSize
            | kSmallViewportBlockSize
            | kSmallViewportMin
            | kSmallViewportMax
            | kLargeViewportWidth
            | kLargeViewportHeight
            | kLargeViewportInlineSize
            | kLargeViewportBlockSize
            | kLargeViewportMin
            | kLargeViewportMax => MediaQueryExpValue::<()>::kStaticViewport,
            kContainerWidth | kContainerHeight | kContainerInlineSize | kContainerBlockSize
            | kContainerMin | kContainerMax => MediaQueryExpValue::<()>::kContainer,
            _ => 0,
        },
        _ => 0,
    }
}
fn EvalLiteral(
    kind: Feature,
    value: &MediaQueryExpValue<CSSNumericLiteralValue>,
    op: MediaQueryOperator,
    v: &dyn MediaValues,
) -> bool {
    use CSSValueID::*;
    use Feature::*;
    let id = match value {
        MediaQueryExpValue::Id(id) => Some(*id),
        _ => None,
    };
    let number = match value {
        MediaQueryExpValue::Value(value) if value.IsNumber() => Some(value.DoubleValue()),
        _ => None,
    };
    let boolean = !value.IsValid();
    let print = v.MediaType().Utf8().eq_ignore_ascii_case("print");
    match kind {
        Width | Height | InlineSize | BlockSize | DeviceWidth | DeviceHeight => {
            let actual = match kind {
                Width => v.Width().unwrap(),
                Height => v.Height().unwrap(),
                InlineSize => v.InlineSize().unwrap(),
                BlockSize => v.BlockSize().unwrap(),
                DeviceWidth => v.DeviceWidth() as f64,
                _ => v.DeviceHeight() as f64,
            };
            if boolean {
                return if matches!(kind, DeviceWidth | DeviceHeight) {
                    true
                } else {
                    actual != 0.0
                };
            }
            match value {
                MediaQueryExpValue::Value(value) if value.IsLength() => CompareDoubleValue(
                    actual,
                    v.ComputeLength(value.DoubleValue(), value.GetType()),
                    op,
                ),
                MediaQueryExpValue::Value(value) if value.IsNumber() => {
                    let query = value.DoubleValue() as i32;
                    (!v.StrictMode() || query == 0) && CompareDoubleValue(actual, query as f64, op)
                }
                _ => false,
            }
        }
        AspectRatio | DeviceAspectRatio => {
            if boolean {
                return true;
            }
            let (width, height) = if kind == AspectRatio {
                (v.Width().unwrap() as i32, v.Height().unwrap() as i32)
            } else {
                (v.DeviceWidth(), v.DeviceHeight())
            };
            match value {
                MediaQueryExpValue::Ratio(ratio) => CompareDoubleValue(
                    width as f64 * ratio.1.DoubleValue(),
                    height as f64
                        * if ratio.0.IsLength() {
                            v.ComputeLength(ratio.0.DoubleValue(), ratio.0.GetType())
                        } else if ratio.0.IsResolution() {
                            ratio.0.ComputeDotsPerPixel()
                        } else {
                            ratio.0.DoubleValue()
                        },
                    op,
                ),
                _ => false,
            }
        }
        Orientation => {
            if let Some(id) = id {
                if v.Width().unwrap() > v.Height().unwrap() {
                    id == kLandscape
                } else {
                    id == kPortrait
                }
            } else {
                v.Width().unwrap() >= 0.0 && v.Height().unwrap() >= 0.0
            }
        }
        Resolution | DevicePixelRatio => {
            let actual = if print {
                300.0_f32 / 96.0
            } else if v.MediaType().Utf8().eq_ignore_ascii_case("screen") {
                v.DevicePixelRatio()
            } else {
                0.0
            };
            if boolean {
                return actual != 0.0;
            }
            match value {
                MediaQueryExpValue::Value(value)
                    if kind == DevicePixelRatio && value.IsNumber() =>
                {
                    CompareValue(actual, value.DoubleValue() as f32, op)
                }
                MediaQueryExpValue::Value(value) if kind == Resolution && value.IsResolution() => {
                    let query = value.ComputeDotsPerPixel() as f32;
                    if value.GetType() == UnitType::kDotsPerCentimeter {
                        CompareValue(
                            (0.5 + 100.0 * actual).floor() / 100.0,
                            (0.5 + 100.0 * query).floor() / 100.0,
                            op,
                        )
                    } else {
                        CompareValue(actual, query, op)
                    }
                }
                _ => false,
            }
        }
        Color | ColorIndex | Monochrome | Transform3d => {
            let actual = match kind {
                Color => v.ColorBitsPerComponent(),
                ColorIndex => 0,
                Monochrome => v.MonochromeBitsPerComponent(),
                _ => i32::from(v.ThreeDEnabled()),
            };
            if boolean {
                actual != 0
            } else {
                number.is_some_and(|query| CompareValue(actual, (query as f32) as i32, op))
            }
        }
        Grid => number.is_some_and(|query| CompareValue((query as f32) as i32, 0, op)),
        Hover => HoverMediaFeatureEval(value, op, v),
        AnyHover => AnyHoverMediaFeatureEval(value, op, v),
        Pointer => PointerMediaFeatureEval(value, op, v),
        AnyPointer => AnyPointerMediaFeatureEval(value, op, v),
        ColorGamut => ColorGamutMediaFeatureEval(value, op, v),
        PrefersColorScheme => PrefersColorSchemeMediaFeatureEval(value, op, v),
        PrefersContrast => PrefersContrastMediaFeatureEval(value, op, v),
        ReducedMotion => PrefersReducedMotionMediaFeatureEval(value, op, v),
        ReducedData => PrefersReducedDataMediaFeatureEval(value, op, v),
        ReducedTransparency => PrefersReducedTransparencyMediaFeatureEval(value, op, v),
        ForcedColors => ForcedColorsMediaFeatureEval(value, op, v),
        NavigationControls => NavigationControlsMediaFeatureEval(value, op, v),
        Scan => {
            v.MediaType().Utf8().eq_ignore_ascii_case("tv") && (boolean || id == Some(kProgressive))
        }
        DynamicRange | VideoDynamicRange => {
            id == Some(kStandard) || id == Some(kHigh) && v.DeviceSupportsHDR()
        }
        Stuck => StuckMediaFeatureEval(value, op, v),
        Snapped => SnappedMediaFeatureEval(value, op, v),
        Scrollable => ScrollableMediaFeatureEval(value, op, v),
        Scrolled => ScrolledMediaFeatureEval(value, op, v),
        DisplayMode => {
            boolean
                || id.is_some_and(|id| {
                    v.DisplayMode()
                        == match id {
                            kFullscreen => DisplayModeValue::kFullscreen,
                            kUnframed => DisplayModeValue::kUnframed,
                            kStandalone => DisplayModeValue::kStandalone,
                            kMinimalUi => DisplayModeValue::kMinimalUi,
                            kWindowControlsOverlay => DisplayModeValue::kWindowControlsOverlay,
                            kBrowser => DisplayModeValue::kBrowser,
                            kTabbed => DisplayModeValue::kTabbed,
                            kPictureInPicture => DisplayModeValue::kPictureInPicture,
                            _ => unreachable!(),
                        }
                })
        }
        DisplayState => {
            boolean
                || match id {
                    Some(kFullscreen) => v.WindowShowState() == WindowShowState::kFullscreen,
                    Some(kMaximized) => v.WindowShowState() == WindowShowState::kMaximized,
                    Some(kMinimized) => v.WindowShowState() == WindowShowState::kMinimized,
                    Some(kNormal) => matches!(
                        v.WindowShowState(),
                        WindowShowState::kDefault
                            | WindowShowState::kInactive
                            | WindowShowState::kNormal
                    ),
                    _ => false,
                }
        }
        Resizable => boolean || id == Some(if v.Resizable() { kTrue } else { kFalse }),
        HorizontalSegments | VerticalSegments => {
            boolean
                || number.is_some_and(|query| {
                    CompareValue(
                        if kind == HorizontalSegments {
                            v.GetHorizontalViewportSegments()
                        } else {
                            v.GetVerticalViewportSegments()
                        },
                        query as i32,
                        op,
                    )
                })
        }
        OverflowInline => {
            if boolean {
                !print
            } else {
                if print {
                    id == Some(kNone)
                } else {
                    id == Some(kScroll)
                }
            }
        }
        OverflowBlock => boolean || !print && id == Some(kScroll) || print && id == Some(kPaged),
        Update => {
            if boolean {
                !print
            } else {
                match id {
                    Some(kNone) => print,
                    Some(kFast) => {
                        !print
                            && v.OutputDeviceUpdateAbilityType()
                                == OutputDeviceUpdateAbilityType::kFastType
                    }
                    Some(kSlow) => {
                        !print
                            && v.OutputDeviceUpdateAbilityType()
                                == OutputDeviceUpdateAbilityType::kSlowType
                    }
                    _ => false,
                }
            }
        }
        DevicePosture => {
            boolean
                || id
                    == Some(if v.GetDevicePosture() == DevicePostureType::kFolded {
                        kFolded
                    } else {
                        kContinuous
                    })
        }
        Scripting => {
            if boolean {
                v.GetScripting() == ScriptingValue::kEnabled
            } else {
                id == Some(match v.GetScripting() {
                    ScriptingValue::kNone => kNone,
                    ScriptingValue::kInitialOnly => kInitialOnly,
                    ScriptingValue::kEnabled => kEnabled,
                })
            }
        }
        InvertedColors => {
            if boolean {
                v.InvertedColors()
            } else {
                id == Some(if v.InvertedColors() { kInverted } else { kNone })
            }
        }
        OriginTrial => OriginTrialTestMediaFeatureEval(value, op, v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_queries::web_preferences::{HoverType, PointerType};
    use crate::media_queries::{MediaValuesCached, MediaValuesCachedData, PreferredColorScheme};
    fn values() -> MediaValuesCached {
        MediaValuesCached::new(&MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            small_viewport_width: 800.0,
            small_viewport_height: 600.0,
            large_viewport_width: 800.0,
            large_viewport_height: 600.0,
            dynamic_viewport_width: 800.0,
            dynamic_viewport_height: 600.0,
            device_width: 1920,
            device_height: 1080,
            device_pixel_ratio: 2.0,
            media_type: foundation::String::from("screen"),
            preferred_color_scheme: PreferredColorScheme::kDark,
            primary_pointer_type: PointerType::kPointerFineType,
            available_pointer_types: PointerType::kPointerFineType as i32,
            primary_hover_type: HoverType::kHoverHoverType,
            available_hover_types: HoverType::kHoverHoverType as i32,
            scripting: ScriptingValue::kEnabled,
            ..Default::default()
        })
    }
    #[test]
    fn production_media_literals_ranges_preferences_and_unknowns() {
        let values = values();
        let evaluator = MediaQueryEvaluator::ForMediaValues(&values);
        for (text, expected) in [
            ("screen", true),
            ("print", false),
            ("not print", true),
            ("(min-width: 800px)", true),
            ("(max-width: 799px)", false),
            ("(799px < width <= 800px)", true),
            ("(800px < width)", false),
            ("(width >= 50em)", true),
            ("(width: 100vw)", true),
            ("(width: 100dvw)", true),
            ("(width: 100cqw)", true),
            ("(device-width: 1920px)", true),
            ("(orientation: landscape)", true),
            ("(aspect-ratio: 4/3)", true),
            ("(device-aspect-ratio: 16/9)", true),
            ("(resolution: 192dpi)", true),
            ("(-webkit-min-device-pixel-ratio: 2)", true),
            ("(prefers-color-scheme: dark)", true),
            ("(hover: hover) and (pointer: fine)", true),
            ("(any-hover: hover) and (any-pointer: coarse)", false),
            ("(scripting: enabled)", true),
            ("(color >= 8)", true),
            ("(unknown: 1)", false),
            ("not (unknown: 1)", false),
            ("not (width: 1foo)", false),
            ("(width: 2)", false),
            ("(grid: 0)", true),
            ("(grid: 1)", false),
            ("(color: 1.5)", false),
            ("(resolution: -1dppx)", false),
            ("(color-index: 0)", true),
            ("(overflow-inline: scroll)", true),
            ("(update: fast)", true),
            ("(display-mode: browser)", true),
        ] {
            assert_eq!(
                evaluator.Eval(&ParseMediaQuerySet(text)),
                expected,
                "{text}"
            );
        }
    }
    #[test]
    fn production_media_results_record_units_and_viewport_dependencies() {
        let values = values();
        let evaluator = MediaQueryEvaluator::ForMediaValues(&values);
        let mut flags = MediaQueryResultFlags::default();
        assert!(evaluator.EvalSet(
            &ParseMediaQuerySet("(width >= 10em) and (height: 100dvh)"),
            Some(&mut flags),
            None
        ));
        assert!(flags.is_viewport_dependent);
        assert_eq!(
            flags.unit_flags,
            MediaQueryExpValue::<()>::kFontRelative | MediaQueryExpValue::<()>::kDynamicViewport
        );
        let mut flags = MediaQueryResultFlags::default();
        assert!(evaluator.EvalSet(
            &ParseMediaQuerySet("(device-width: 1920px)"),
            Some(&mut flags),
            None
        ));
        assert!(flags.is_device_dependent);
    }
    #[test]
    fn production_media_values_are_container_specific_and_updateable() {
        let mut values = values();
        let queries = ParseMediaQuerySet("(min-width: 600px)");
        assert!(MediaQueryEvaluator::ForMediaValues(&values).Eval(&queries));
        values.OverrideViewportDimensions(300.0, 600.0);
        assert!(!MediaQueryEvaluator::ForMediaValues(&values).Eval(&queries));
        assert!(MediaQueryEvaluator::ForMediaValues(&values)
            .Eval(&ParseMediaQuerySet("(orientation: portrait)")));
    }
    #[test]
    fn custom_media_boolean_nested_or_unknown_and_dependency_flags_match_source() {
        use crate::style_rule::StyleRuleCustomMedia;
        let features = MediaQueryFeatureFlags {
            custom_media: true,
            ..Default::default()
        };
        let parse =
            |text: &str| MediaQueryParser::new(features).ParseMediaQuerySet(StringView::from(text));
        let values = values();
        let evaluator = MediaQueryEvaluator::ForMediaValuesWithFeatures(&values, features);
        let mut rules = CustomMediaRulesMap::new();
        for (name, value) in [("--yes", true), ("--no", false)] {
            rules.insert(
                AtomicString::from_str(name),
                Rc::new(StyleRuleCustomMedia::FromBoolean(
                    AtomicString::from_str(name),
                    value,
                )),
            );
        }
        for (name, text) in [
            ("--unknown-or-false", "(--missing), (--no)"),
            ("--unknown-or-true", "(--missing), (--yes)"),
            ("--sized", "(width >= 10em)"),
        ] {
            rules.insert(
                AtomicString::from_str(name),
                Rc::new(StyleRuleCustomMedia::FromMediaQuery(
                    AtomicString::from_str(name),
                    Rc::new(parse(text)),
                )),
            );
        }
        for (text, expected) in [
            ("(--yes)", KleeneValue::kTrue),
            ("(--no)", KleeneValue::kFalse),
            ("(--missing)", KleeneValue::kUnknown),
            ("not (--missing)", KleeneValue::kUnknown),
            ("(--unknown-or-false)", KleeneValue::kUnknown),
            ("(--unknown-or-true)", KleeneValue::kTrue),
        ] {
            let query = parse(text);
            assert_eq!(
                evaluator.EvalQuery(&query.QueryVector()[0], None, Some(&rules)),
                expected,
                "{text}"
            );
        }
        let mut flags = MediaQueryResultFlags::default();
        assert!(evaluator.EvalSet(&parse("(--sized)"), Some(&mut flags), Some(&rules)));
        assert!(flags.is_viewport_dependent);
        assert_eq!(flags.unit_flags, MediaQueryExpValue::<()>::kFontRelative);
        assert_eq!(
            evaluator.EvalQuery(&parse("(--yes)").QueryVector()[0], None, None),
            KleeneValue::kUnknown
        );
    }
    #[test]
    fn video_dynamic_range_follows_source_dynamic_range_capability() {
        for hdr in [false, true] {
            let values = MediaValuesCached::new(&MediaValuesCachedData {
                device_supports_hdr: hdr,
                ..Default::default()
            });
            let evaluator = MediaQueryEvaluator::ForMediaValues(&values);
            assert!(evaluator.Eval(&ParseMediaQuerySet("(video-dynamic-range: standard)")));
            assert_eq!(
                evaluator.Eval(&ParseMediaQuerySet("(video-dynamic-range: high)")),
                hdr
            );
            assert_eq!(
                evaluator.Eval(&ParseMediaQuerySet("(video-dynamic-range: high)")),
                evaluator.Eval(&ParseMediaQuerySet("(dynamic-range: high)"))
            );
        }
    }
}

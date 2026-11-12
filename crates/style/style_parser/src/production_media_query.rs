// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Canonical production media query parser backend. Moved with its original
//! typed CSS values from Style so HTMLSourceElement can own the parsed set
//! without introducing a DOM -> Style dependency. Chromium revision 6c1d401f,
//! media_query_exp.cc:58-371,491-606; media_query_parser.cc:137-163.
use crate::{
    css_numeric_literal_value::CSSNumericLiteralValue,
    css_parser_token::{CSSParserTokenType::*, NumericValueType},
    css_parser_token_stream::{CSSParserTokenStream, TokenStreamTokenizer},
    css_primitive_value::UnitType,
    media_query_exp::*,
    media_query_parser::{MediaQueryParser, MediaQueryParserBackend, MediaQueryRuntimeFeatures},
    media_query_set::MediaQuerySet,
};
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
pub enum Feature {
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
pub fn FeatureForName(name: &AtomicString) -> Option<(Feature, MediaQueryOperator)> {
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
    if stream.Peek().GetType() == kFunctionToken {
        let expression = crate::css_math_expression_node::ConsumeMathFunction(stream).ok()?;
        return Some(CSSNumericLiteralValue::CreateMath(
            expression,
            crate::css_math_function_value::ValueRange::All,
        ));
    }
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
        let integer_feature = matches!(
            feature,
            Color | ColorIndex | Monochrome | Grid | HorizontalSegments | VerticalSegments
        );
        let value = ConsumeNumeric(stream)?;
        // Blink tries ConsumeInteger before ConsumeNumber/Length/Resolution.
        // A number-category math function therefore carries the integer target
        // range even for features that later accept a general number.
        let range = if integer_feature || value.MathValue().is_some() && value.IsNumber() {
            crate::css_math_function_value::ValueRange::Integer
        } else {
            crate::css_math_function_value::ValueRange::All
        };
        let value = value.WithValueRange(range);
        if matches!(feature, AspectRatio | DeviceAspectRatio) {
            if value.GetValueIfKnown().is_some_and(|value| value < 0.0) {
                return None;
            }
            let denominator = if stream.Peek().GetType() == kDelimiterToken
                && stream.Peek().Delimiter() == b'/' as u16
            {
                stream.ConsumeIncludingWhitespace();
                let denominator = ConsumeNumeric(stream)?
                    .WithValueRange(crate::css_math_function_value::ValueRange::NonNegative);
                if !denominator.IsNumber()
                    || denominator
                        .GetValueIfKnown()
                        .is_some_and(|value| value < 0.0)
                {
                    return None;
                }
                denominator
            } else {
                CSSNumericLiteralValue::Create(1.0, UnitType::kNumber)
            };
            let numerator = if value.GetValueIfKnown() == Some(0.0)
                && denominator.GetValueIfKnown() == Some(0.0)
            {
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
                value.IsLength() || value.IsNumber() && value.GetValueIfKnown() == Some(0.0)
            }
            Resolution => {
                value.IsResolution() && !value.GetValueIfKnown().is_some_and(|value| value < 0.0)
            }
            Color | ColorIndex | Monochrome | HorizontalSegments | VerticalSegments => {
                value.IsInteger()
            }
            Grid => {
                value.IsInteger()
                    && value
                        .GetValueIfKnown()
                        .is_none_or(|value| matches!(value, 0.0 | 1.0))
            }
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

/// HTMLSourceElement::CreateMediaQueryList, html_source_element.cc:70-82.
/// An empty media attribute has no list; nonempty media compiles exactly once.
pub type HTMLSourceMediaQuerySet = MediaQuerySet<CSSNumericLiteralValue>;
pub fn CompileHTMLSourceMediaQuery(text: &str) -> Option<Rc<HTMLSourceMediaQuerySet>> {
    (!text.is_empty()).then(|| Rc::new(ParseMediaQuerySet(text)))
}

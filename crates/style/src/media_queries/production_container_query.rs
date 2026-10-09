// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Document owners for the translated container feature evaluator. Snapshots
//! are published by the layout/scroll/anchor owner, never inferred from viewport.
use super::{
    color_space_gamut::ColorSpaceGamut,
    device_posture_provider::DevicePostureType,
    display_mode::DisplayMode,
    forced_colors::ForcedColors,
    navigation_controls::NavigationControls,
    preferred_color_scheme::PreferredColorScheme,
    preferred_contrast::PreferredContrast,
    scripting::Scripting,
    web_preferences::{HoverType, OutputDeviceUpdateAbilityType, PointerType},
    window_show_state::WindowShowState,
};
use super::{
    container_state::*, media_query_container::*, media_values::MediaValues, MediaValuesCached,
    MediaValuesCachedData,
};
use crate::{
    css_numeric_literal_value::{CSSNumericLiteralValue, IsLength},
    css_primitive_value::UnitType,
    css_value::CSSValuePayload,
    parser::css_parser_token::CSSParserTokenType::*,
    production_css_value::{CSSUnparsedDeclarationValue, CSSVariableData, Value},
    resolver::production_style_builder::custom_properties::{
        CustomProperties, SubstitutionContext,
    },
};
use foundation::{
    AtomicString, CSSValueID, IsHorizontalWritingMode, String, WritingDirectionMode, WritingMode,
};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    computed_style_constants::TryTactic,
    position_area::{PositionArea, PositionAreaRegion},
};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, PartialEq)]
pub struct ContainerFallback {
    pub name: Option<AtomicString>,
    pub tactics: [TryTactic; 3],
    pub area: PositionArea,
}
impl ContainerFallback {
    pub fn None() -> Self {
        Self {
            name: None,
            tactics: [TryTactic::kNone; 3],
            area: PositionArea::default(),
        }
    }
}
/// The owner supplies all state together. Missing snapshot means unavailable
/// layout state, which the condition visitor reports as Unknown.
#[derive(Clone, PartialEq)]
pub struct ContainerState {
    pub width: Option<f64>,
    pub scroll_state_available: bool,
    pub anchored_state_available: bool,
    pub height: Option<f64>,
    pub stuck_horizontal: ContainerStuckPhysical,
    pub stuck_vertical: ContainerStuckPhysical,
    pub stuck_inline: ContainerStuckLogical,
    pub stuck_block: ContainerStuckLogical,
    pub snapped: ContainerSnappedFlags,
    pub scrollable_horizontal: ContainerScrollableFlags,
    pub scrollable_vertical: ContainerScrollableFlags,
    pub scrollable_inline: ContainerScrollableFlags,
    pub scrollable_block: ContainerScrollableFlags,
    pub scrolled_horizontal: ContainerScrolled,
    pub scrolled_vertical: ContainerScrolled,
    pub scrolled_inline: ContainerScrolled,
    pub scrolled_block: ContainerScrolled,
    pub anchored_fallback: ContainerFallback,
    pub abs_container_direction: WritingDirectionMode,
}
impl ContainerState {
    /// Publishing size does not synthesize scroll or anchor observations.
    pub fn SizeOnly(width: f64, height: f64, direction: WritingDirectionMode) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
            scroll_state_available: false,
            anchored_state_available: false,
            stuck_horizontal: ContainerStuckPhysical::kNo,
            stuck_vertical: ContainerStuckPhysical::kNo,
            stuck_inline: ContainerStuckLogical::kNo,
            stuck_block: ContainerStuckLogical::kNo,
            snapped: 0,
            scrollable_horizontal: 0,
            scrollable_vertical: 0,
            scrollable_inline: 0,
            scrollable_block: 0,
            scrolled_horizontal: ContainerScrolled::kNone,
            scrolled_vertical: ContainerScrolled::kNone,
            scrolled_inline: ContainerScrolled::kNone,
            scrolled_block: ContainerScrolled::kNone,
            anchored_fallback: ContainerFallback::None(),
            abs_container_direction: direction,
        }
    }
}
/// Registered properties are computed by their document registration owner.
/// These required methods retain syntax/initial/inheritance semantics outside
/// the evaluator and supply its real CSSValueFromComputedStyle read.
pub struct RegisteredContainerPropertyResult {
    pub value: Option<Rc<Value>>,
    pub conversion_flags: u32,
    pub has_random: bool,
    pub unsupported: Option<&'static str>,
}
pub trait RegisteredContainerPropertyResolver {
    fn IsRegistered(&self, name: &AtomicString) -> bool;
    fn ComputeValue(
        &self,
        node: usize,
        name: &AtomicString,
        specified: Option<&Value>,
        custom: &CustomProperties,
        context: SubstitutionContext<'_>,
        values: &DocumentMediaValues<'_>,
    ) -> RegisteredContainerPropertyResult;
    fn ComputedValue(
        &self,
        node: usize,
        name: &AtomicString,
        custom: &CustomProperties,
    ) -> Option<Rc<Value>>;
}

pub struct DocumentMediaValues<'a> {
    document: &'a dom::Document,
    cached: MediaValuesCached,
    container: Option<usize>,
    style: Option<&'a ComputedStyle>,
    root_style: Option<&'a ComputedStyle>,
    state: Option<&'a ContainerState>,
}
impl<'a> DocumentMediaValues<'a> {
    pub fn new(document: &'a dom::Document, media: &MediaValuesCachedData) -> Self {
        Self {
            document,
            cached: MediaValuesCached::new(media),
            container: None,
            style: None,
            root_style: None,
            state: None,
        }
    }
    pub fn ForContainer(
        document: &'a dom::Document,
        index: usize,
        media: &MediaValuesCachedData,
        style: &'a ComputedStyle,
        state: Option<&'a ContainerState>,
    ) -> Self {
        let mut media = media.clone();
        media.em_size = style.GetFontDescription().SpecifiedSize();
        Self {
            document,
            cached: MediaValuesCached::new(&media),
            container: Some(index),
            style: Some(style),
            root_style: Some(style),
            state,
        }
    }
    pub fn WithRootStyle(mut self, root_style: &'a ComputedStyle) -> Self {
        self.root_style = Some(root_style);
        self
    }
    pub fn ContainerElement(&self) -> Option<usize> {
        self.container
    }
}
impl super::media_query_evaluator::MediaQueryFrameValues for DocumentMediaValues<'_> {
    // cpp: media_query_evaluator.cc:115. A live document host owns these reads.
    fn CreateDynamicIfFrameExists(&self) -> Option<&dyn MediaValues> {
        Some(self)
    }
}

impl MediaValues for DocumentMediaValues<'_> {
    fn DeviceWidth(&self) -> i32 {
        self.cached.DeviceWidth()
    }
    fn DeviceHeight(&self) -> i32 {
        self.cached.DeviceHeight()
    }
    fn DevicePixelRatio(&self) -> f32 {
        self.cached.DevicePixelRatio()
    }
    fn DeviceSupportsHDR(&self) -> bool {
        self.cached.DeviceSupportsHDR()
    }
    fn ColorBitsPerComponent(&self) -> i32 {
        self.cached.ColorBitsPerComponent()
    }
    fn MonochromeBitsPerComponent(&self) -> i32 {
        self.cached.MonochromeBitsPerComponent()
    }
    fn InvertedColors(&self) -> bool {
        self.cached.InvertedColors()
    }
    fn PrimaryPointerType(&self) -> PointerType {
        self.cached.PrimaryPointerType()
    }
    fn AvailablePointerTypes(&self) -> i32 {
        self.cached.AvailablePointerTypes()
    }
    fn PrimaryHoverType(&self) -> HoverType {
        self.cached.PrimaryHoverType()
    }
    fn OutputDeviceUpdateAbilityType(&self) -> OutputDeviceUpdateAbilityType {
        self.cached.OutputDeviceUpdateAbilityType()
    }
    fn AvailableHoverTypes(&self) -> i32 {
        self.cached.AvailableHoverTypes()
    }
    fn ThreeDEnabled(&self) -> bool {
        self.cached.ThreeDEnabled()
    }
    fn MediaType(&self) -> String {
        self.cached.MediaType()
    }
    fn DisplayMode(&self) -> DisplayMode {
        self.cached.DisplayMode()
    }
    fn WindowShowState(&self) -> WindowShowState {
        self.cached.WindowShowState()
    }
    fn Resizable(&self) -> bool {
        self.cached.Resizable()
    }
    fn StrictMode(&self) -> bool {
        self.cached.StrictMode()
    }
    fn HasValues(&self) -> bool {
        self.cached.HasValues()
    }
    fn ColorGamut(&self) -> ColorSpaceGamut {
        self.cached.ColorGamut()
    }
    fn GetPreferredColorScheme(&self) -> PreferredColorScheme {
        self.cached.GetPreferredColorScheme()
    }
    fn GetPreferredContrast(&self) -> PreferredContrast {
        self.cached.GetPreferredContrast()
    }
    fn PrefersReducedMotion(&self) -> bool {
        self.cached.PrefersReducedMotion()
    }
    fn PrefersReducedData(&self) -> bool {
        self.cached.PrefersReducedData()
    }
    fn PrefersReducedTransparency(&self) -> bool {
        self.cached.PrefersReducedTransparency()
    }
    fn GetForcedColors(&self) -> ForcedColors {
        self.cached.GetForcedColors()
    }
    fn GetNavigationControls(&self) -> NavigationControls {
        self.cached.GetNavigationControls()
    }
    fn GetHorizontalViewportSegments(&self) -> i32 {
        self.cached.GetHorizontalViewportSegments()
    }
    fn GetVerticalViewportSegments(&self) -> i32 {
        self.cached.GetVerticalViewportSegments()
    }
    fn GetDevicePosture(&self) -> DevicePostureType {
        self.cached.GetDevicePosture()
    }
    fn GetScripting(&self) -> Scripting {
        self.cached.GetScripting()
    }
    fn ViewportWidth(&self) -> f64 {
        self.cached.ViewportWidth()
    }
    fn ViewportHeight(&self) -> f64 {
        self.cached.ViewportHeight()
    }
    fn GetDocument(&self) -> Option<&dom::Document> {
        Some(self.document)
    }
    fn HasScrollState(&self) -> bool {
        self.state.is_some_and(|state| state.scroll_state_available)
    }
    fn HasAnchoredState(&self) -> bool {
        self.state
            .is_some_and(|state| state.anchored_state_available)
    }
    fn GetWritingMode(&self) -> WritingMode {
        self.style.map_or_else(
            || self.cached.GetWritingMode(),
            |s| s.GetWritingDirection().GetWritingMode(),
        )
    }
    fn Width(&self) -> Option<f64> {
        if self.container.is_some() {
            self.state.and_then(|s| s.width)
        } else {
            Some(self.cached.ViewportWidth())
        }
    }
    fn Height(&self) -> Option<f64> {
        if self.container.is_some() {
            self.state.and_then(|s| s.height)
        } else {
            Some(self.cached.ViewportHeight())
        }
    }
    fn ComputeLength(&self, value: f64, unit: UnitType) -> f64 {
        use UnitType::*;
        let fallback = || self.cached.ComputeLength(value, unit);
        if let Some(style) = self.style {
            let root = self.root_style.unwrap_or(style);
            let font_style = if matches!(unit, kRems | kRexs | kRchs | kRics | kRcaps | kRlhs) {
                root
            } else {
                style
            };
            let font = unsafe { &*font_style.GetFont() };
            let em = font_style.GetFontDescription().SpecifiedSize() as f64;
            let zoom = font_style.EffectiveZoom() as f64;
            let factor = match unit {
                kEms | kRems | kQuirkyEms => Some(em),
                kExs | kRexs => Some(
                    unsafe { font.PrimaryFont().as_ref() }
                        .filter(|d| d.GetFontMetrics().HasXHeight())
                        .map_or(em / 2.0, |d| d.GetFontMetrics().XHeight() as f64 / zoom),
                ),
                kChs | kRchs => Some(
                    unsafe { font.PrimaryFontWithDigitZero().as_ref() }
                        .map_or(0.0, |d| d.GetFontMetrics().ZeroWidth() as f64 / zoom),
                ),
                kIcs | kRics => Some(
                    unsafe { font.PrimaryFontWithCjkWater().as_ref() }
                        .and_then(|d| *d.IdeographicInlineSize())
                        .map_or(em, |v| v as f64 / zoom),
                ),
                kCaps | kRcaps => Some(
                    unsafe { font.PrimaryFont().as_ref() }
                        .map_or(0.0, |d| d.GetFontMetrics().CapHeight() as f64 / zoom),
                ),
                kLhs | kRlhs => Some(font_style.ComputedLineHeight() as f64 / zoom),
                _ => None,
            };
            if let Some(factor) = factor {
                return value * factor;
            }
        }
        let width = self.Width();
        let height = self.Height();
        let horizontal = IsHorizontalWritingMode(self.GetWritingMode());
        let inline = if horizontal { width } else { height };
        let block = if horizontal { height } else { width };
        let axis = match unit {
            kContainerWidth => width,
            kContainerHeight => height,
            kContainerInlineSize => inline,
            kContainerBlockSize => block,
            kContainerMin => inline.zip(block).map(|(a, b)| a.min(b)),
            kContainerMax => inline.zip(block).map(|(a, b)| a.max(b)),
            _ => return fallback(),
        };
        axis.map_or_else(fallback, |size| value * size / 100.0)
    }
    fn StuckHorizontal(&self) -> ContainerStuckPhysical {
        self.state
            .expect("container state must be available before evaluation")
            .stuck_horizontal
    }
    fn StuckVertical(&self) -> ContainerStuckPhysical {
        self.state
            .expect("container state must be available before evaluation")
            .stuck_vertical
    }
    fn StuckInline(&self) -> ContainerStuckLogical {
        self.state
            .expect("container state must be available before evaluation")
            .stuck_inline
    }
    fn StuckBlock(&self) -> ContainerStuckLogical {
        self.state
            .expect("container state must be available before evaluation")
            .stuck_block
    }
    fn SnappedFlags(&self) -> ContainerSnappedFlags {
        self.state
            .expect("container state must be available before evaluation")
            .snapped
    }
    fn ScrollableHorizontal(&self) -> ContainerScrollableFlags {
        self.state
            .expect("container state must be available before evaluation")
            .scrollable_horizontal
    }
    fn ScrollableVertical(&self) -> ContainerScrollableFlags {
        self.state
            .expect("container state must be available before evaluation")
            .scrollable_vertical
    }
    fn ScrollableInline(&self) -> ContainerScrollableFlags {
        self.state
            .expect("container state must be available before evaluation")
            .scrollable_inline
    }
    fn ScrollableBlock(&self) -> ContainerScrollableFlags {
        self.state
            .expect("container state must be available before evaluation")
            .scrollable_block
    }
    fn ScrolledHorizontal(&self) -> ContainerScrolled {
        self.state
            .expect("container state must be available before evaluation")
            .scrolled_horizontal
    }
    fn ScrolledVertical(&self) -> ContainerScrolled {
        self.state
            .expect("container state must be available before evaluation")
            .scrolled_vertical
    }
    fn ScrolledInline(&self) -> ContainerScrolled {
        self.state
            .expect("container state must be available before evaluation")
            .scrolled_inline
    }
    fn ScrolledBlock(&self) -> ContainerScrolled {
        self.state
            .expect("container state must be available before evaluation")
            .scrolled_block
    }
}

pub struct ContainerStyleResolver<'a> {
    pub node: usize,
    pub values: &'a DocumentMediaValues<'a>,
    pub custom: &'a CustomProperties,
    pub parent_custom: Option<&'a CustomProperties>,
    pub substitution: SubstitutionContext<'a>,
    pub registered: Option<&'a dyn RegisteredContainerPropertyResolver>,
    pub unsupported: &'a Cell<Option<&'static str>>,
}
pub struct ContainerNumericContext<'a> {
    values: &'a DocumentMediaValues<'a>,
    custom: &'a CustomProperties,
    substitution: SubstitutionContext<'a>,
    flags: u32,
    unsupported: &'a Cell<Option<&'static str>>,
}
impl ContainerNumericContext<'_> {
    // cpp: style_cascade.cc:2634-2731. Resolved tokenizer output is consumed
    // directly through the same typed math parser used by property consumers.
    fn Coerce(&mut self, data: &CSSVariableData) -> Option<CSSNumericLiteralValue> {
        let significant = data
            .tokens
            .iter()
            .filter(|t| !matches!(t.token.GetType(), kWhitespaceToken | kCommentToken))
            .collect::<Vec<_>>();
        let resolved = if significant.len() == 1
            && significant[0].token.GetType() == kIdentToken
            && significant[0]
                .token
                .Value()
                .ToString()
                .Utf8()
                .starts_with("--")
        {
            self.custom
                .Get(&significant[0].token.Value().ToString().Utf8())?
                .as_ref()
                .clone()
        } else {
            match self.custom.SubstituteWithContext(data,self.substitution) {
            Ok(data)=>data,Err(crate::resolver::production_style_builder::custom_properties::VariableResolutionError::Unsupported(op))=>{
                self.unsupported.set(Some(op));return None},Err(_)=>return None,
        }
        };
        let mut stream =
            crate::parser::css_parser_token_stream::CSSParserTokenStream::FromTokenizer(
                crate::parser::production_property_parser::VariableTokenReplay::new(&resolved),
            );
        stream.ConsumeWhitespace();
        if stream.Peek().GetType() == kFunctionToken {
            let expression = match crate::css_math_expression_node::ConsumeMathFunction(&mut stream)
            {
                Ok(expression) => expression,
                Err(crate::css_math_expression_node::MathError::UnsupportedFunction) => {
                    self.unsupported
                        .set(Some("CSSMathExpressionNode extended math functions"));
                    return None;
                }
                Err(crate::css_math_expression_node::MathError::UnsupportedTypedArithmetic) => {
                    self.unsupported
                        .set(Some("CSSMathType typed exponent arithmetic"));
                    return None;
                }
                Err(_) => return None,
            };
            if !stream.AtEnd() {
                return None;
            }
            // CSSSyntaxDefinition::CreateNumericSyntax excludes frequency.
            if expression.Category()
                == crate::css_math_expression_node::CalculationResultCategory::Frequency
            {
                return None;
            }
            let unit = expression.CanonicalUnit()?;
            let number = expression
                .ComputeValue(
                    &mut |v, u| {
                        self.CanonicalNumeric(v, u)
                            .map(|n| n.DoubleValue())
                            .ok_or(crate::css_math_expression_node::MathError::MissingLengthContext)
                    },
                    None,
                )
                .ok()?;
            // StyleCascade resolves math through CSSPrimitiveValue, including
            // its NaN censoring; range coercion itself admits all finite signs.
            let number = if number.is_nan() { 0.0 } else { number };
            return Some(CSSNumericLiteralValue::Create(number, unit));
        }
        let token = stream.Peek().clone();
        let unit = match token.GetType() {
            kNumberToken => UnitType::kNumber,
            kPercentageToken => UnitType::kPercentage,
            kDimensionToken => token.GetUnitType(),
            _ => return None,
        };
        stream.ConsumeIncludingWhitespace();
        if !stream.AtEnd() {
            return None;
        }
        self.CanonicalNumeric(token.NumericValue(), unit)
    }
    fn CanonicalNumeric(&mut self, number: f64, unit: UnitType) -> Option<CSSNumericLiteralValue> {
        use StyleQueryConversionFlags as F;
        use UnitType::*;
        let flags = match unit {
            kEms | kQuirkyEms => F::EM,
            kExs | kChs | kIcs | kCaps => F::GLYPH,
            kRems | kRexs | kRchs | kRics | kRcaps => F::ROOT_FONT,
            kLhs => F::LINE_HEIGHT,
            kRlhs => F::ROOT_LINE_HEIGHT,
            kViewportWidth | kViewportHeight | kViewportInlineSize | kViewportBlockSize
            | kViewportMin | kViewportMax => F::VIEWPORT,
            kSmallViewportWidth
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
            | kLargeViewportMax => F::SMALL_LARGE_VIEWPORT,
            kDynamicViewportWidth
            | kDynamicViewportHeight
            | kDynamicViewportInlineSize
            | kDynamicViewportBlockSize
            | kDynamicViewportMin
            | kDynamicViewportMax => F::DYNAMIC_VIEWPORT,
            kContainerWidth | kContainerHeight | kContainerInlineSize | kContainerBlockSize
            | kContainerMin | kContainerMax => F::CONTAINER,
            _ => 0,
        };
        self.flags |= flags;
        let (number, unit) = if IsLength(unit) {
            (self.values.ComputeLength(number, unit), kPixels)
        } else {
            match unit {
                kNumber | kInteger => (number, kNumber),
                kPercentage => (number, kPercentage),
                kDegrees => (number, kDegrees),
                kRadians => (number * 180.0 / std::f64::consts::PI, kDegrees),
                kGradians => (number * 0.9, kDegrees),
                kTurns => (number * 360.0, kDegrees),
                kMilliseconds => (number / 1000.0, kSeconds),
                kSeconds => (number, kSeconds),
                kDotsPerInch => (number / 96.0, kDotsPerPixel),
                kDotsPerCentimeter => (number * 2.54 / 96.0, kDotsPerPixel),
                kDotsPerPixel | kX => (number, kDotsPerPixel),
                _ => return None,
            }
        };
        Some(CSSNumericLiteralValue::Create(number, unit))
    }
}
impl StyleRangeContext<Value, CSSUnparsedDeclarationValue> for ContainerNumericContext<'_> {
    fn CoerceReference(
        &mut self,
        value: &CSSUnparsedDeclarationValue,
    ) -> Option<CSSNumericLiteralValue> {
        self.Coerce(&value.data)
    }
    fn CoerceBound(&mut self, value: &Value) -> Option<CSSNumericLiteralValue> {
        // cpp: media_query_evaluator.cc:1817-1820,1835-1838.
        match value.Payload() {
            CSSValuePayload::kUnparsedDeclarationClass(value) => self.Coerce(&value.data),
            CSSValuePayload::kNumericLiteralClass(value) => {
                self.CanonicalNumeric(value.DoubleValue(), value.GetType())
            }
            CSSValuePayload::kMathFunctionClass(value) => {
                let unit = value.expression.CanonicalUnit()?;
                let number = value
                    .ComputeValue(
                        &mut |v, u| {
                            self.CanonicalNumeric(v, u).map(|n| n.DoubleValue()).ok_or(
                                crate::css_math_expression_node::MathError::MissingLengthContext,
                            )
                        },
                        None,
                    )
                    .ok()?;
                Some(CSSNumericLiteralValue::Create(number, unit))
            }
            _ => None,
        }
    }
    fn TakeLengthConversionFlags(&mut self) -> u32 {
        std::mem::take(&mut self.flags)
    }
}
impl<'a> StyleQueryResolver<Value, CSSUnparsedDeclarationValue> for ContainerStyleResolver<'a> {
    type ComputedValue = Value;
    type VariableData = CSSVariableData;
    type RangeContext = ContainerNumericContext<'a>;
    fn RangeQueriesEnabled(&self) -> bool {
        true
    }
    fn PrepareStyleRange(&self) -> Self::RangeContext {
        ContainerNumericContext {
            values: self.values,
            custom: self.custom,
            substitution: self.substitution,
            flags: 0,
            unsupported: self.unsupported,
        }
    }
    fn ReferenceHasRandomFunctions(&self, value: &CSSUnparsedDeclarationValue) -> bool {
        value.data.features & CSSVariableData::HAS_RANDOM_FUNCTIONS != 0
    }
    fn ValueHasRandomFunctions(&self, value: &Value) -> bool {
        value.HasRandomFunctions()
    }
    fn IsCascadeDependentKeyword(&self, value: &Value) -> bool {
        value.IsCascadeDependentKeyword()
    }
    // cpp: StyleResolver::ComputeValue and media_query_evaluator.cc:1887-1921.
    fn ComputeValue(
        &self,
        name: &AtomicString,
        specified: Option<&Value>,
    ) -> StyleQueryComputedResult<Value, CSSVariableData> {
        if let Some(registry) = self.registered.filter(|r| r.IsRegistered(name)) {
            let result = registry.ComputeValue(
                self.node,
                name,
                specified,
                self.custom,
                self.substitution,
                self.values,
            );
            if let Some(operation) = result.unsupported {
                self.unsupported.set(Some(operation));
            }
            return StyleQueryComputedResult {
                value: StyleQueryComputedValue::Registered(result.value),
                conversion_flags: result.conversion_flags,
                has_random: result.has_random,
            };
        }
        let data=specified.and_then(|value|match value.Payload(){
            CSSValuePayload::kUnparsedDeclarationClass(value)=>match self.custom.SubstituteWithContext(&value.data,self.substitution){
                Ok(data)=>Some(Rc::new(data)),Err(crate::resolver::production_style_builder::custom_properties::VariableResolutionError::Unsupported(op))=>{
                    self.unsupported.set(Some(op));None},Err(_)=>None},
            CSSValuePayload::kInitialClass(_)=>None,
            _=>{self.unsupported.set(Some("StyleResolver custom property specified value class"));None},
        });
        let has_random = data
            .as_ref()
            .is_some_and(|d| self.VariableHasRandomFunctions(d));
        StyleQueryComputedResult {
            value: if data.is_some() {
                StyleQueryComputedValue::Unparsed(data)
            } else {
                StyleQueryComputedValue::Registered(None)
            },
            conversion_flags: 0,
            has_random,
        }
    }
    fn ComputedVariableData(&self, name: &AtomicString) -> Option<Rc<CSSVariableData>> {
        self.custom.Get(&name.Utf8()).cloned()
    }
    fn VariableHasRandomFunctions(&self, data: &CSSVariableData) -> bool {
        data.features & CSSVariableData::HAS_RANDOM_FUNCTIONS != 0
    }
    fn EqualsIgnoringAttrTainting(&self, a: &CSSVariableData, b: &CSSVariableData) -> bool {
        a.original_text == b.original_text
    }
    fn ComputedPropertyValue(&self, name: &AtomicString) -> Option<Rc<Value>> {
        if let Some(registry) = self.registered.filter(|r| r.IsRegistered(name)) {
            return registry.ComputedValue(self.node, name, self.custom);
        }
        self.custom.Get(&name.Utf8()).map(|data| {
            crate::production_css_value::unparsed(
                data.as_ref().clone(),
                crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
            )
        })
    }
}
impl FallbackQueryResolver<Value> for ContainerStyleResolver<'_> {
    type Fallback = ContainerFallback;
    fn AnchoredFallback(&self) -> Self::Fallback {
        self.values
            .state
            .expect("anchored state checked by condition visitor")
            .anchored_fallback
            .clone()
    }
    fn IsNone(&self, v: &Self::Fallback) -> bool {
        v.name.is_none() && v.tactics[0] == TryTactic::kNone && v.area.IsNone()
    }
    fn PositionAreaIsNone(&self, v: &Self::Fallback) -> bool {
        v.area.IsNone()
    }
    fn ToPhysicalPositionArea(&self, v: &Self::Fallback) -> Self::Fallback {
        ContainerFallback {
            name: None,
            tactics: [TryTactic::kNone; 3],
            area: v.area.ToPhysical(
                &self.values.state.unwrap().abs_container_direction,
                &self.values.style.unwrap().GetWritingDirection(),
            ),
        }
    }
    fn ConvertSinglePositionTryFallback(&self, value: &Value, allow_any: bool) -> Self::Fallback {
        use crate::resolver::style_builder_converter::{
            ExtractPositionAreaSpan, IsRepeatedPositionAreaValue,
        };
        let identifier = |v: &Value| match v.Payload() {
            CSSValuePayload::kIdentifierClass(v) => Some(v.0),
            _ => None,
        };
        let area = if let Some(id) = identifier(value) {
            if id == CSSValueID::kNone {
                PositionArea::default()
            } else {
                let (a, b) = ExtractPositionAreaSpan(id);
                let (c, d) = if IsRepeatedPositionAreaValue(id) {
                    (a, b)
                } else {
                    let r = if allow_any {
                        PositionAreaRegion::kAny
                    } else {
                        PositionAreaRegion::kAll
                    };
                    (r, r)
                };
                PositionArea::new(a, b, c, d)
            }
        } else if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
            let (a, b) =
                ExtractPositionAreaSpan(identifier(&pair.first).expect("parsed position-area"));
            let (c, d) =
                ExtractPositionAreaSpan(identifier(&pair.second).expect("parsed position-area"));
            PositionArea::new(a, b, c, d)
        } else {
            let mut result = ContainerFallback::None();
            let mut index = 0;
            let CSSValuePayload::kValueListClass(list) = value.Payload() else {
                self.unsupported
                    .set(Some("position-try fallback value class"));
                return result;
            };
            for value in &list.values {
                match value.Payload() {
                    CSSValuePayload::kCustomIdentClass(name) => {
                        result.name = Some(name.name.clone())
                    }
                    CSSValuePayload::kIdentifierClass(id) => {
                        if index >= 3 {
                            self.unsupported.set(Some("position-try tactic count"));
                            return result;
                        }
                        result.tactics[index] = match id.0 {
                            CSSValueID::kFlipBlock => TryTactic::kFlipBlock,
                            CSSValueID::kFlipInline => TryTactic::kFlipInline,
                            CSSValueID::kFlipStart => TryTactic::kFlipStart,
                            _ => {
                                self.unsupported.set(Some("position-try tactic"));
                                return result;
                            }
                        };
                        index += 1
                    }
                    _ => {
                        self.unsupported
                            .set(Some("position-try fallback list value"));
                        return result;
                    }
                }
            }
            return result;
        };
        ContainerFallback {
            name: None,
            tactics: [TryTactic::kNone; 3],
            area,
        }
    }
    // cpp: position_try_fallbacks.cc:15-27. Tree scopes are ignored for queries.
    fn Matches(&self, a: &Self::Fallback, b: &Self::Fallback) -> bool {
        a.name == b.name && a.tactics == b.tactics && a.area.Matches(&b.area)
    }
}

use super::{
    media_query_backend::MediaQueryValueBackend,
    media_query_evaluator::MediaQueryEvaluator,
    media_query_exp::{
        MediaQueryExp, MediaQueryExpBounds, MediaQueryExpComparison, MediaQueryExpValue,
        MediaQueryOperator,
    },
};
use crate::kleene_value::{KleeneAnd, KleeneNot, KleeneOr, KleeneValue};
use crate::production_container_projection::{
    RestoreContainerOperand, RestoreContainerVariableData,
};
use cssom::{
    CSSContainerCondition as Condition, CSSContainerFeature as Feature,
    CSSContainerFunction as Function, CSSContainerOperand as Operand,
    CSSContainerOperator as Operator,
};
fn Op(op: Operator) -> MediaQueryOperator {
    match op {
        Operator::None => MediaQueryOperator::kNone,
        Operator::Equal => MediaQueryOperator::kEq,
        Operator::Less => MediaQueryOperator::kLt,
        Operator::LessEqual => MediaQueryOperator::kLe,
        Operator::Greater => MediaQueryOperator::kGt,
        Operator::GreaterEqual => MediaQueryOperator::kGe,
    }
}
fn ValueOperand(value: &Operand) -> Option<MediaQueryExpValue<Value>> {
    Some(match value {
        Operand::Invalid => MediaQueryExpValue::Invalid,
        Operand::Identifier(id) => MediaQueryExpValue::Id(*id),
        Operand::Ratio(a, b) => MediaQueryExpValue::Ratio(Rc::new((
            RestoreContainerOperand(a)?,
            RestoreContainerOperand(b)?,
        ))),
        _ => MediaQueryExpValue::Value(RestoreContainerOperand(value)?),
    })
}
fn NumericOperand(value: &Operand) -> Option<MediaQueryExpValue<CSSNumericLiteralValue>> {
    Some(match ValueOperand(value)? {
        MediaQueryExpValue::Invalid => MediaQueryExpValue::Invalid,
        MediaQueryExpValue::Id(id) => MediaQueryExpValue::Id(id),
        MediaQueryExpValue::Value(v) => match v.Payload() {
            CSSValuePayload::kNumericLiteralClass(v) => {
                MediaQueryExpValue::Value(Rc::new(v.clone()))
            }
            _ => return None,
        },
        MediaQueryExpValue::Ratio(pair) => {
            let CSSValuePayload::kNumericLiteralClass(a) = pair.0.Payload() else {
                return None;
            };
            let CSSValuePayload::kNumericLiteralClass(b) = pair.1.Payload() else {
                return None;
            };
            MediaQueryExpValue::Ratio(Rc::new((Rc::new(a.clone()), Rc::new(b.clone()))))
        }
    })
}
/// Returns the selector's union of required container axes, using each candidate
/// ancestor's own writing mode. Style queries select ordinary element ancestors.
pub fn RequiredContainerType(condition: &Condition, style: &ComputedStyle) -> u32 {
    let horizontal = IsHorizontalWritingMode(style.GetWritingDirection().GetWritingMode());
    match condition {
        Condition::Feature(f) => match f
            .name
            .Utf8()
            .trim_start_matches("min-")
            .trim_start_matches("max-")
        {
            "width" => {
                if horizontal {
                    1
                } else {
                    2
                }
            }
            "height" => {
                if horizontal {
                    2
                } else {
                    1
                }
            }
            "inline-size" => 1,
            "block-size" => 2,
            "aspect-ratio" | "orientation" => 3,
            "stuck" | "snapped" | "scrollable" | "scrolled" => 4,
            "fallback" => 8,
            _ => 0,
        },
        Condition::Function(Function::ScrollState, c) => RequiredContainerType(c, style) | 4,
        Condition::Function(Function::Anchored, c) => RequiredContainerType(c, style) | 8,
        Condition::Function(_, c) | Condition::Not(c) | Condition::Nested(c) => {
            RequiredContainerType(c, style)
        }
        Condition::And(a, b) | Condition::Or(a, b) => {
            RequiredContainerType(a, style) | RequiredContainerType(b, style)
        }
        _ => 0,
    }
}
pub fn EvaluateContainerCondition(
    condition: &Condition,
    resolver: &ContainerStyleResolver<'_>,
) -> KleeneValue {
    match condition {
        Condition::Unknown(_) => KleeneValue::kUnknown,
        Condition::Unsupported(_) => {
            resolver
                .unsupported
                .set(Some("container condition projection value dependency"));
            KleeneValue::kUnknown
        }
        Condition::Not(c) => KleeneNot(EvaluateContainerCondition(c, resolver)),
        Condition::Nested(c) | Condition::Function(_, c) => EvaluateContainerCondition(c, resolver),
        Condition::And(a, b) => {
            let a = EvaluateContainerCondition(a, resolver);
            if a == KleeneValue::kFalse {
                a
            } else {
                KleeneAnd(a, EvaluateContainerCondition(b, resolver))
            }
        }
        Condition::Or(a, b) => {
            let a = EvaluateContainerCondition(a, resolver);
            if a == KleeneValue::kTrue {
                a
            } else {
                KleeneOr(a, EvaluateContainerCondition(b, resolver))
            }
        }
        Condition::Feature(f) => EvaluateContainerFeature(f, resolver),
    }
}
fn EvaluateContainerFeature(
    feature: &Feature,
    resolver: &ContainerStyleResolver<'_>,
) -> KleeneValue {
    let result = if feature.reference.is_some()
        || feature.name.Utf8().starts_with("--")
        || feature.name == "fallback"
    {
        if feature.name == "fallback"
            && !resolver
                .values
                .state
                .is_some_and(|s| s.anchored_state_available)
        {
            return KleeneValue::kUnknown;
        }
        let Some(left) = ValueOperand(&feature.left.value) else {
            resolver
                .unsupported
                .set(Some("container style operand projection"));
            return KleeneValue::kUnknown;
        };
        let Some(right) = ValueOperand(&feature.right.value) else {
            resolver
                .unsupported
                .set(Some("container style operand projection"));
            return KleeneValue::kUnknown;
        };
        // Style operands are CSSValues even for CSS-wide keywords. Ordinary
        // size/state identifiers and anchored(none) retain the ID alternative.
        let style_operand = |value: MediaQueryExpValue<Value>, operand: &Operand| {
            if feature.name != "fallback" && matches!(value, MediaQueryExpValue::Id(_)) {
                MediaQueryExpValue::Value(
                    RestoreContainerOperand(operand).expect("typed style CSSValue"),
                )
            } else {
                value
            }
        };
        let left = style_operand(left, &feature.left.value);
        let right = style_operand(right, &feature.right.value);
        let bounds = MediaQueryExpBounds::new(
            &MediaQueryExpComparison::new(&left, Op(feature.left.operator)),
            &MediaQueryExpComparison::new(&right, Op(feature.right.operator)),
        );
        if feature.name == "fallback" {
            if EvalFallbackFeature(&right, resolver) {
                KleeneValue::kTrue
            } else {
                KleeneValue::kFalse
            }
        } else {
            let exp = if let Some(reference) = &feature.reference {
                MediaQueryExp::CreateStyleRange(
                    Rc::new(CSSUnparsedDeclarationValue {
                        data: Rc::new(RestoreContainerVariableData(reference)),
                        mode: crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
                    }),
                    &bounds,
                    resolver.RangeQueriesEnabled(),
                )
                .expect("stable style range enabled")
            } else {
                MediaQueryExp::CreateWithBounds(&feature.name, &bounds)
            };
            EvalStyleFeature(&exp, None, resolver)
        }
    } else {
        if resolver.values.state.is_none()
            || (matches!(
                feature.name.Utf8().as_str(),
                "stuck" | "snapped" | "scrollable" | "scrolled"
            ) && !resolver
                .values
                .state
                .is_some_and(|s| s.scroll_state_available))
        {
            return KleeneValue::kUnknown;
        }
        let Some(left) = NumericOperand(&feature.left.value) else {
            resolver
                .unsupported
                .set(Some("container size numeric operand"));
            return KleeneValue::kUnknown;
        };
        let Some(right) = NumericOperand(&feature.right.value) else {
            resolver
                .unsupported
                .set(Some("container size numeric operand"));
            return KleeneValue::kUnknown;
        };
        let bounds = MediaQueryExpBounds::new(
            &MediaQueryExpComparison::new(&left, Op(feature.left.operator)),
            &MediaQueryExpComparison::new(&right, Op(feature.right.operator)),
        );
        MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(resolver.values).EvalNode(
            &super::conditional_exp_node::ConditionalExpNode::Feature(
                MediaQueryExp::CreateWithBounds(&feature.name, &bounds),
            ),
            None,
            None,
        )
    };
    if resolver.unsupported.get().is_some() {
        KleeneValue::kUnknown
    } else {
        result
    }
}

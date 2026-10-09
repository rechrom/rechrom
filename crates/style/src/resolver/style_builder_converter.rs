// cpp: third_party/blink/renderer/core/css/resolver/style_builder_converter.{h,cc}
// Source ledger (Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15):
// Source root: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
// Effective excludes only blanks/comments, retaining syntax and preprocessing.
// mapped + omitted + pending = effective. Batch 1 added 1488 mapped lines;
// batch 2 added 1501; final batch adds 933. Production pending is zero.
// Total physical=4951 effective=4279 mapped=3922 omitted=357 pending=0.
// Pure includes/namespace/forward/static-only/access labels, debug assertions,
// metrics and h240 ConvertColumnRuleWidth (no definition) are omitted.
// All source conversion decisions, loops, recursion, validation, units, scopes,
// colors, transforms, registered values, timeline and position-area logic map.
// Required hooks only access genuine foreign subclasses/DOM/parser/platform
// and construct objects owned by those classes; none has a default body.
// IsRepeatedPositionAreaValue additionally maps css_parsing_utils.cc:10351-10372,
// without adding those external helper lines to this source-pair ledger.
// style_builder_converter.h: physical=616 effective=527 mapped=444 omitted=83 pending=0.
// h mapped source ranges: 102-128,136-154,157-178,180-479,482-591,595-606,611-613.
// h omitted effective source lines: 27-28,30,32-75,77,79-96,98-99,101,129,132-133,135,240,480,550-551,578,588,596,598,614,616.
// h pending effective source lines: none.
// style_builder_converter.cc: physical=4335 effective=3752 mapped=3478 omitted=274 pending=0.
// cc mapped source ranges: 118-1635,1637-3308,3310-4333.
// cc omitted effective source lines: 26,28-31,33-114,116,121,156,191,200,206,275,285,309,325,362-365,373,384-386,469-470,473-475,507-509,541,546,548,550-552,559-563,703,1001,1006,1013,1019-1020,1095,1266,1274,1327,1372-1373,1389,1494-1498,1509-1510,1516-1517,1523-1524,1527,1549,1554,1591,1606,1627,1632,1644,1666,1672-1673,1686,1711-1712,1726,1740-1741,1829-1830,1861-1862,1915,1919-1920,1924-1926,1932,1946,2089,2126,2233,2241,2248,2268,2270,2279,2281,2300,2316,2353,2355,2358,2402,2407,2410,2427,2447,2453,2490,2569,2584-2587,2738,2753,2801,2853,2855,2858,2869,2871,2874,2885,2887,2893,2895-2896,3024,3089,3105,3123,3125,3129,3131,3158,3181,3249,3292-3294,3297,3356,3360,3380,3385,3405,3418,3423,3455,3459,3475,3481,3696,3701,3704,3719-3720,3723,3735-3736,3766-3767,3796-3797,3826,3828,3831,3845-3848,3901,3909,3915-3916,3988,3991,4257,4274,4304,4311,4318-4319,4335.
// cc pending effective source lines: none.
#![allow(non_snake_case)]
use super::style_resolver_state::{ResolverValue, StyleResolverState, StyleResolverStateBackend};
use font_engine::fonts::{
    font_description::{FontVariantCaps, GenericFamilyType, Kerning, LigaturesState},
    font_family::{FontFamily, FontFamilyType, SharedFontFamily},
    font_optical_sizing::OpticalSizing,
    font_palette::{FontPalette, KeywordPaletteName},
    font_selection_types::*,
    font_size_adjust::{FontSizeAdjust, Metric, ValueType},
    font_variant_alternates::FontVariantAlternates,
    font_variant_east_asian::*,
    font_variant_emoji::FontVariantEmoji,
    font_variant_numeric::*,
    opentype::font_settings::*,
};
use foundation::{
    AtomicString, CSSPropertyID, CSSValueID, Color, ColorSpace, DynamicRangeLimit,
    DynamicRangeLimitKind, HueInterpolationMethod, Length, LengthBox,
};
use layoutng_style::css::css_reflection_direction::CSSReflectionDirection;
use layoutng_style::style::color_scheme::mojom::blink::ColorScheme;
use layoutng_style::style::{
    basic_shapes::BasicShape,
    computed_style_constants::*,
    computed_style_initial_values::ComputedStyleInitialValues,
    grid_position::GridPosition,
    grid_track_size::{GridTrackSize, GridTrackSizeType},
    nine_piece_image::NinePieceImage,
    style_border_shape::StyleBorderShape,
    style_content_alignment_data::StyleContentAlignmentData,
    style_flex_wrap_data::StyleFlexWrapData,
    style_reflection::StyleReflection,
    style_self_alignment_data::StyleSelfAlignmentData,
};
use std::{collections::BTreeMap, sync::Arc};

/// Required reads from real CSS subclasses and operations belonging to foreign
/// font, shape, filter and DOM owners. Converter decisions stay below.
pub trait StyleBuilderConverterBackend: StyleResolverStateBackend {
    type FontSize;
    type FamilyDescription;
    type VariantLigatures;
    type FontVariantPosition;
    fn NewFontVariantPosition(&self, id: CSSValueID) -> Self::FontVariantPosition;
    type SVGStyleResource;
    type ClipPathOperation;
    type FilterOperations;
    type OffscreenFont;
    type GridTemplateAreas;
    type GridAreaMap;
    type TextLinkColors;
    fn Identifier(&self, value: &ResolverValue<Self>) -> Option<CSSValueID>;
    fn List<'a>(&self, value: &'a ResolverValue<Self>) -> Option<Vec<&'a ResolverValue<Self>>>;
    fn Pair<'a>(
        &self,
        value: &'a ResolverValue<Self>,
    ) -> Option<(&'a ResolverValue<Self>, &'a ResolverValue<Self>)>;
    fn CustomIdent(&self, value: &ResolverValue<Self>) -> Option<AtomicString>;
    fn ComputeIdent(
        &self,
        state: &StyleResolverState<Self>,
        value: &ResolverValue<Self>,
    ) -> AtomicString;
    fn StringValue(&self, value: &ResolverValue<Self>) -> Option<AtomicString>;
    fn IsPendingSystemFont(&self, value: &ResolverValue<Self>) -> bool;
    fn IsPercentage(&self, value: &ResolverValue<Self>) -> bool;
    fn IsNumber(&self, value: &ResolverValue<Self>) -> bool;
    fn IsFlex(&self, value: &ResolverValue<Self>) -> bool;
    fn IsLength(&self, value: &ResolverValue<Self>) -> bool;
    fn IsCalculated(&self, value: &ResolverValue<Self>) -> bool;
    fn IsFontRelativeLength(&self, value: &ResolverValue<Self>) -> bool;
    fn IsRem(&self, value: &ResolverValue<Self>) -> bool;
    fn Number(&self, data: &Self::LengthConversionData, value: &ResolverValue<Self>) -> f64;
    fn Percentage(&self, data: &Self::LengthConversionData, value: &ResolverValue<Self>) -> f64;
    fn Degrees(&self, data: &Self::LengthConversionData, value: &ResolverValue<Self>) -> f64;
    fn CanonicalUnit(&self, data: &Self::LengthConversionData, value: &ResolverValue<Self>) -> f64;
    fn ComputeLength(&self, data: &Self::LengthConversionData, value: &ResolverValue<Self>) -> f64;
    fn PrimitiveLength(
        &self,
        data: &Self::LengthConversionData,
        value: &ResolverValue<Self>,
    ) -> Length;
    fn ComputeInteger(&self, data: &Self::LengthConversionData, value: &ResolverValue<Self>)
        -> i32;
    fn ZoomedPixels(&self, data: &Self::LengthConversionData, pixels: f64) -> f64;
    fn EffectiveZoom(&self, state: &StyleResolverState<Self>) -> f32;
    fn ReflectParts<'a>(
        &self,
        value: &'a ResolverValue<Self>,
    ) -> (
        CSSReflectionDirection,
        Option<&'a ResolverValue<Self>>,
        Option<&'a ResolverValue<Self>>,
    );
    fn MapNinePieceImage(
        &self,
        state: &StyleResolverState<Self>,
        property: CSSPropertyID,
        value: &ResolverValue<Self>,
        image: &mut NinePieceImage,
    );
    fn DynamicRangeMix<'a>(
        &self,
        value: &'a ResolverValue<Self>,
    ) -> Option<Vec<(&'a ResolverValue<Self>, &'a ResolverValue<Self>)>>;
    fn IsURI(&self, value: &ResolverValue<Self>) -> bool;
    fn IsBasicShape(&self, value: &ResolverValue<Self>) -> bool;
    fn URIString(&self, value: &ResolverValue<Self>) -> foundation::String;
    fn SVGResource(
        &self,
        state: &StyleResolverState<Self>,
        property: CSSPropertyID,
        value: &ResolverValue<Self>,
    ) -> Self::SVGResource;
    fn NewSVGStyleResource(
        &self,
        resource: Self::SVGResource,
        url: foundation::String,
    ) -> Self::SVGStyleResource;
    fn BasicShapeForValue(
        &self,
        state: &StyleResolverState<Self>,
        value: &ResolverValue<Self>,
    ) -> *mut dyn BasicShape;
    fn GeometryBox(&self, value: &ResolverValue<Self>) -> GeometryBox;
    fn NewBorderShape(
        &self,
        outer: *mut dyn BasicShape,
        inner: *mut dyn BasicShape,
        outer_box: GeometryBox,
        inner_box: GeometryBox,
    ) -> StyleBorderShape;
    fn NewShapeClipPath(
        &self,
        shape: *mut dyn BasicShape,
        geometry: GeometryBox,
    ) -> Self::ClipPathOperation;
    fn NewGeometryClipPath(&self, geometry: GeometryBox) -> Self::ClipPathOperation;
    fn NewReferenceClipPath(
        &self,
        url: foundation::String,
        resource: Self::SVGResource,
    ) -> Self::ClipPathOperation;
    fn Quad<'a>(&self, value: &'a ResolverValue<Self>) -> [&'a ResolverValue<Self>; 4];
    fn CreateFilterOperations(
        &self,
        state: &StyleResolverState<Self>,
        value: &ResolverValue<Self>,
        property: CSSPropertyID,
    ) -> Self::FilterOperations;
    fn CreateOffscreenFilterOperations(
        &self,
        value: &ResolverValue<Self>,
        font: Option<&Self::OffscreenFont>,
    ) -> Self::FilterOperations;
    fn FlexWrapMode(&self, value: &ResolverValue<Self>) -> FlexWrapMode;
    fn FontFamilyValue(&self, value: &ResolverValue<Self>) -> Option<AtomicString>;
    fn GenericFontFamilyName(
        &self,
        state: &StyleResolverState<Self>,
        family: GenericFamilyType,
    ) -> AtomicString;
    fn HasSettings(&self, state: &StyleResolverState<Self>) -> bool;
    fn SetFamilyTreeScopeToDocument(&self, state: &StyleResolverState<Self>);
    fn ResolveSystemFontFamily(&self, value: &ResolverValue<Self>) -> AtomicString;
    fn LegacyMacSystemFontFamily(&self) -> Option<AtomicString>;
    fn NewFamilyDescription(
        &self,
        generic: GenericFamilyType,
        family: FontFamily,
    ) -> Self::FamilyDescription;
    fn FontFeature(
        &self,
        data: &Self::LengthConversionData,
        value: &ResolverValue<Self>,
    ) -> (u32, i32);
    fn FontVariation<'a>(&self, value: &'a ResolverValue<Self>) -> (u32, &'a ResolverValue<Self>);
    fn InitialVariationSettings(&self) -> Option<FontVariationSettings>;
    fn PaletteMix<'a>(
        &self,
        value: &'a ResolverValue<Self>,
    ) -> Option<(
        &'a ResolverValue<Self>,
        &'a ResolverValue<Self>,
        Option<&'a ResolverValue<Self>>,
        Option<&'a ResolverValue<Self>>,
        ColorSpace,
        Option<HueInterpolationMethod>,
    )>;
    fn NormalizeColorMixPercentages(
        &self,
        data: &Self::LengthConversionData,
        first: Option<&ResolverValue<Self>>,
        second: Option<&ResolverValue<Self>>,
    ) -> Option<(f64, f64)>;
    fn ParentMathDepth(&self, state: &StyleResolverState<Self>) -> i32;
    fn MathDepth(&self, state: &StyleResolverState<Self>) -> i32;
    fn ParentMathConstants(
        &self,
        state: &StyleResolverState<Self>,
    ) -> Option<(Option<f32>, Option<f32>)>;
    fn ParentFontSize(&self, state: &StyleResolverState<Self>) -> Option<Self::FontSize>;
    fn SizeParts(&self, size: &Self::FontSize) -> (f32, bool);
    fn NewFontSize(&self, keyword: i32, size: f32, absolute: bool) -> Self::FontSize;
    fn FontSizeKeyword(&self, id: CSSValueID) -> Option<i32>;
    fn SmallerFontSize(&self, size: Self::FontSize) -> Self::FontSize;
    fn LargerFontSize(&self, size: Self::FontSize) -> Self::FontSize;
    fn ResolveSystemFontSize(
        &self,
        state: &StyleResolverState<Self>,
        value: &ResolverValue<Self>,
    ) -> f32;
    fn EvaluateFontSizeCalc(
        &self,
        data: &Self::LengthConversionData,
        value: &ResolverValue<Self>,
        parent: f32,
    ) -> f32;
    fn SetHasGlyphRelativeUnits(&self, state: &StyleResolverState<Self>);
    fn InitialSizeAdjust(&self) -> FontSizeAdjust;
    fn SizeAdjustMetric(&self, value: &ResolverValue<Self>) -> Metric;
    fn ObliqueRange<'a>(
        &self,
        value: &'a ResolverValue<Self>,
    ) -> Option<(&'a ResolverValue<Self>, Vec<&'a ResolverValue<Self>>)>;
    fn ObliqueZeroAsNormalEnabled(&self) -> bool;
    fn ParentWeight(&self, state: &StyleResolverState<Self>) -> FontSelectionValue;
    fn BolderWeight(&self, parent: FontSelectionValue) -> FontSelectionValue;
    fn LighterWeight(&self, parent: FontSelectionValue) -> FontSelectionValue;
    fn NewLigatures(
        &self,
        common: LigaturesState,
        discretionary: LigaturesState,
        historical: LigaturesState,
        contextual: LigaturesState,
    ) -> Self::VariantLigatures;
    fn NewAlternates(&self) -> FontVariantAlternates;
    fn Alternate<'a>(
        &self,
        value: &'a ResolverValue<Self>,
    ) -> Option<(CSSValueID, Vec<&'a ResolverValue<Self>>)>;
    fn SetAlternate(
        &self,
        target: &mut FontVariantAlternates,
        function: CSSValueID,
        aliases: Vec<AtomicString>,
    );
    fn SetHistoricalForms(&self, target: &mut FontVariantAlternates);
    fn AlternatesIsNormal(&self, target: &FontVariantAlternates) -> bool;
    fn ItemPosition(&self, value: &ResolverValue<Self>) -> ItemPosition;
    fn OverflowAlignment(&self, value: &ResolverValue<Self>) -> OverflowAlignment;
    fn ContentDistributionIds(
        &self,
        value: &ResolverValue<Self>,
    ) -> (CSSValueID, CSSValueID, CSSValueID);
    fn DistributionFromIdentifier(&self, id: CSSValueID) -> ContentDistributionType;
    fn ContentPositionFromIdentifier(&self, id: CSSValueID) -> ContentPosition;
    fn OverflowFromIdentifier(&self, id: CSSValueID) -> OverflowAlignment;
    fn GridTemplateAreaParts(&self, value: &ResolverValue<Self>) -> (Self::GridAreaMap, u32, u32);
    fn NewGridTemplateAreas(
        &self,
        areas: Self::GridAreaMap,
        rows: u32,
        columns: u32,
    ) -> Self::GridTemplateAreas;
    fn GridMaxTracks(&self) -> i32;
    fn FunctionType(&self, value: &ResolverValue<Self>) -> CSSValueID;
    fn IdentifierFlags(&self, value: &ResolverValue<Self>) -> u32;
    fn TextColor(&self, colors: &Self::TextLinkColors, scheme: ColorScheme) -> Color;
    fn LinkColor(&self, colors: &Self::TextLinkColors, scheme: ColorScheme) -> Color;
    fn VisitedLinkColor(&self, colors: &Self::TextLinkColors, scheme: ColorScheme) -> Color;
    fn ActiveLinkColor(&self, colors: &Self::TextLinkColors, scheme: ColorScheme) -> Color;
    fn FocusRingColor(&self, scheme: ColorScheme) -> Color;
}

pub struct StyleBuilderConverter;
pub struct StyleBuilderConverterBase;
impl StyleBuilderConverter {
    // cc:199-221
    pub fn ConvertBoxReflect<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<StyleReflection> {
        if b.Identifier(v).is_some() {
            return None;
        }
        let (direction, offset, mask) = b.ReflectParts(v);
        let mut result = StyleReflection::default();
        result.SetDirection(direction);
        if let Some(offset) = offset {
            result.SetOffset(&Self::ConvertLength(b, s, offset));
        }
        if let Some(mask) = mask {
            let mut image = NinePieceImage::MaskDefaults();
            b.MapNinePieceImage(s, CSSPropertyID::kWebkitBoxReflect, mask, &mut image);
            result.SetMask(&image);
        }
        Some(result)
    }
    // cc:223-274
    pub fn ConvertDynamicRangeLimit<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> DynamicRangeLimit {
        if let Some(mix) = b.DynamicRangeMix(v) {
            let (mut standard, mut constrained, mut sum) = (0.0, 0.0, 0.0);
            for (limit, percentage) in mix {
                let limit = Self::ConvertDynamicRangeLimit(b, s, limit);
                let fraction = 0.01
                    * (b.Percentage(&s.CssToLengthConversionData(), percentage) as f32)
                        .clamp(0.0, 100.0);
                standard += fraction * limit.standard_mix;
                constrained += fraction * limit.constrained_high_mix;
                sum += fraction;
            }
            if sum == 0.0 {
                return DynamicRangeLimit::new(DynamicRangeLimitKind::kHigh);
            }
            return DynamicRangeLimit::from_mix(standard / sum, constrained / sum);
        }
        DynamicRangeLimit::new(match b.Identifier(v) {
            Some(CSSValueID::kStandard) => DynamicRangeLimitKind::kStandard,
            Some(CSSValueID::kConstrained) => DynamicRangeLimitKind::kConstrainedHigh,
            _ => DynamicRangeLimitKind::kHigh,
        })
    }
    // cc:276-341
    pub fn ConvertElementReference<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        property: CSSPropertyID,
    ) -> Option<B::SVGStyleResource> {
        if b.Identifier(v).is_some() {
            None
        } else {
            Some(b.NewSVGStyleResource(b.SVGResource(s, property, v), b.URIString(v)))
        }
    }
    fn BasicShapeAndCoordBoxForValue<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        default: GeometryBox,
    ) -> (*mut dyn BasicShape, GeometryBox) {
        if let Some((shape, geometry)) = b.Pair(v) {
            (b.BasicShapeForValue(s, shape), b.GeometryBox(geometry))
        } else {
            (b.BasicShapeForValue(s, v), default)
        }
    }
    pub fn ConvertBorderShape<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<StyleBorderShape> {
        if let Some(id) = b.Identifier(v) {
            assert_eq!(id, CSSValueID::kNone);
            return None;
        }
        if let Some(list) = b.List(v) {
            if list.len() == 2 {
                let (o, ob) =
                    Self::BasicShapeAndCoordBoxForValue(b, s, list[0], GeometryBox::kBorderBox);
                let (i, ib) =
                    Self::BasicShapeAndCoordBoxForValue(b, s, list[1], GeometryBox::kPaddingBox);
                return Some(b.NewBorderShape(o, i, ob, ib));
            }
        }
        let (o, ob) = Self::BasicShapeAndCoordBoxForValue(b, s, v, GeometryBox::kHalfBorderBox);
        Some(b.NewBorderShape(o, o, ob, ob))
    }
    pub fn ConvertClip<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> LengthBox {
        let q = b.Quad(v);
        LengthBox::new(
            Self::ConvertLengthOrAuto(b, s, q[0]),
            Self::ConvertLengthOrAuto(b, s, q[1]),
            Self::ConvertLengthOrAuto(b, s, q[2]),
            Self::ConvertLengthOrAuto(b, s, q[3]),
        )
    }
    // cc:354-394
    pub fn ConvertClipPath<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<B::ClipPathOperation> {
        if let Some(list) = b.List(v) {
            if b.IsBasicShape(list[0]) {
                let geometry = if list.len() == 2 && b.Identifier(list[1]).is_some() {
                    b.GeometryBox(list[1])
                } else {
                    GeometryBox::kBorderBox
                };
                return Some(b.NewShapeClipPath(b.BasicShapeForValue(s, list[0]), geometry));
            }
            return Some(b.NewGeometryClipPath(b.GeometryBox(list[0])));
        }
        if b.IsURI(v) {
            return Some(b.NewReferenceClipPath(
                b.URIString(v),
                b.SVGResource(s, CSSPropertyID::kClipPath, v),
            ));
        }
        None
    }
    pub fn ConvertFilterOperations<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        p: CSSPropertyID,
    ) -> B::FilterOperations {
        b.CreateFilterOperations(s, v, p)
    }
    pub fn ConvertOffscreenFilterOperations<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
        f: Option<&B::OffscreenFont>,
    ) -> B::FilterOperations {
        b.CreateOffscreenFilterOperations(v, f)
    }
    pub fn ConvertFlexWrapData<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> StyleFlexWrapData {
        let (mut mode, mut balanced) = (FlexWrapMode::kNowrap, false);
        for item in b.List(v).unwrap_or_else(|| vec![v]) {
            if b.Identifier(item) == Some(CSSValueID::kBalance) {
                balanced = true;
            } else {
                mode = b.FlexWrapMode(item);
            }
        }
        if balanced && mode == FlexWrapMode::kNowrap {
            mode = FlexWrapMode::kWrap;
        }
        StyleFlexWrapData::with_balance(mode, balanced)
    }
    pub fn ConvertFontFamily<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> B::FamilyDescription {
        b.SetFamilyTreeScopeToDocument(s);
        StyleBuilderConverterBase::ConvertFontFamily(b, s, v, b.HasSettings(s))
    }
    pub fn ConvertFontKerning<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Kerning {
        if b.IsPendingSystemFont(v) {
            return Kerning::kAutoKerning;
        }
        match b.Identifier(v) {
            Some(CSSValueID::kAuto) => Kerning::kAutoKerning,
            Some(CSSValueID::kNormal) => Kerning::kNormalKerning,
            Some(CSSValueID::kNone) => Kerning::kNoneKerning,
            _ => unreachable!("parsed font kerning"),
        }
    }
    pub fn ConvertFontVariantPosition<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> B::FontVariantPosition {
        let id = if b.IsPendingSystemFont(v) {
            CSSValueID::kNormal
        } else {
            b.Identifier(v).expect("font position")
        };
        match id {
            CSSValueID::kNormal | CSSValueID::kSub | CSSValueID::kSuper => {
                b.NewFontVariantPosition(id)
            }
            _ => unreachable!("font position"),
        }
    }
    pub fn ConvertFontVariantEmoji<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> FontVariantEmoji {
        use FontVariantEmoji::*;
        if b.IsPendingSystemFont(v) {
            return kNormalVariantEmoji;
        }
        match b.Identifier(v) {
            Some(CSSValueID::kNormal) => kNormalVariantEmoji,
            Some(CSSValueID::kText) => kTextVariantEmoji,
            Some(CSSValueID::kEmoji) => kEmojiVariantEmoji,
            Some(CSSValueID::kUnicode) => kUnicodeVariantEmoji,
            _ => unreachable!("parsed emoji"),
        }
    }
    pub fn ConvertFontOpticalSizing<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> OpticalSizing {
        if b.IsPendingSystemFont(v) {
            return OpticalSizing::kAutoOpticalSizing;
        }
        match b.Identifier(v) {
            Some(CSSValueID::kAuto) => OpticalSizing::kAutoOpticalSizing,
            Some(CSSValueID::kNone) => OpticalSizing::kNoneOpticalSizing,
            _ => unreachable!("parsed optical sizing"),
        }
    }
    pub fn ConvertFontFeatureSettings<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> FontFeatureSettings {
        StyleBuilderConverterBase::ConvertFontFeatureSettings(b, &s.CssToLengthConversionData(), v)
    }
    pub fn ConvertFontVariationSettings<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<FontVariationSettings> {
        if b.Identifier(v) == Some(CSSValueID::kNormal) || b.IsPendingSystemFont(v) {
            return b.InitialVariationSettings();
        }
        let mut axes = BTreeMap::new();
        for item in b.List(v).expect("variation list") {
            let (tag, value) = b.FontVariation(item);
            axes.insert(tag, b.Number(&s.CssToLengthConversionData(), value) as f32);
        }
        let mut settings = FontSettings::default();
        for (tag, value) in axes {
            settings.Append(FontVariationAxis::new(tag, value));
        }
        Some(Arc::new(settings))
    }
    pub fn ConvertFontLanguageOverride<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> AtomicString {
        if b.Identifier(v) == Some(CSSValueID::kNormal) {
            AtomicString::default()
        } else {
            b.StringValue(v).unwrap_or_default()
        }
    }
    pub fn ConvertFontPalette<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<Arc<FontPalette>> {
        StyleBuilderConverterBase::ConvertFontPalette(b, &s.CssToLengthConversionData(), v)
    }
    // cc:855-917
    pub fn MathScriptScaleFactor<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
    ) -> f32 {
        let (mut a, mut z) = (b.ParentMathDepth(s), b.MathDepth(s));
        if a == z {
            return 1.0;
        }
        let invert = z < a;
        if invert {
            std::mem::swap(&mut a, &mut z);
        }
        let default: f32 = 0.71;
        let mut exponent = z - a;
        let mut scale = 1.0;
        if let Some((script, scriptscript)) = b.ParentMathConstants(s) {
            let script = script.filter(|n| *n != 0.0).unwrap_or(default);
            let scriptscript = scriptscript
                .filter(|n| *n != 0.0)
                .unwrap_or(default * default);
            if a <= 0 && z >= 2 {
                scale *= scriptscript;
                exponent -= 2;
            } else if a == 1 {
                scale *= scriptscript / script;
                exponent -= 1;
            } else if z == 1 {
                scale *= script;
                exponent -= 1;
            }
        }
        scale *= default.powi(exponent);
        if invert {
            1.0 / scale
        } else {
            scale
        }
    }
    pub fn ConvertFontSize<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> B::FontSize {
        let parent = b
            .ParentFontSize(s)
            .unwrap_or_else(|| b.NewFontSize(0, 0.0, false));
        if b.Identifier(v) == Some(CSSValueID::kMath) {
            let (size, absolute) = b.SizeParts(&parent);
            let scale = Self::MathScriptScaleFactor(b, s);
            b.SetHasGlyphRelativeUnits(s);
            return b.NewFontSize(0, scale * size, absolute);
        }
        StyleBuilderConverterBase::ConvertFontSize(b, s, v, &s.FontSizeConversionData(), parent)
    }
    pub fn ConvertFontSizeAdjust<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> FontSizeAdjust {
        StyleBuilderConverterBase::ConvertFontSizeAdjust(b, s, v)
    }
    pub fn ConvertFontStretchKeyword<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Option<FontSelectionValue> {
        Some(match b.Identifier(v)? {
            CSSValueID::kUltraCondensed => kUltraCondensedWidthValue,
            CSSValueID::kExtraCondensed => kExtraCondensedWidthValue,
            CSSValueID::kCondensed => kCondensedWidthValue,
            CSSValueID::kSemiCondensed => kSemiCondensedWidthValue,
            CSSValueID::kNormal => kNormalWidthValue,
            CSSValueID::kSemiExpanded => kSemiExpandedWidthValue,
            CSSValueID::kExpanded => kExpandedWidthValue,
            CSSValueID::kExtraExpanded => kExtraExpandedWidthValue,
            CSSValueID::kUltraExpanded => kUltraExpandedWidthValue,
            _ => return None,
        })
    }
    pub fn ConvertFontStretch<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> FontSelectionValue {
        StyleBuilderConverterBase::ConvertFontStretch(b, &s.CssToLengthConversionData(), v)
    }
    pub fn ConvertFontStyle<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> FontSelectionValue {
        StyleBuilderConverterBase::ConvertFontStyle(b, &s.CssToLengthConversionData(), v)
    }
    pub fn ConvertFontWeight<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> FontSelectionValue {
        StyleBuilderConverterBase::ConvertFontWeight(
            b,
            &s.CssToLengthConversionData(),
            v,
            b.ParentWeight(s),
        )
    }
    pub fn ConvertFontVariantCaps<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> FontVariantCaps {
        StyleBuilderConverterBase::ConvertFontVariantCaps(b, v)
    }
    pub fn ConvertFontVariantLigatures<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> B::VariantLigatures {
        use LigaturesState::*;
        let mut states = [kNormalLigaturesState; 4];
        if let Some(list) = b.List(v) {
            for item in list {
                let (index, state) = match b.Identifier(item) {
                    Some(CSSValueID::kNoCommonLigatures) => (0, kDisabledLigaturesState),
                    Some(CSSValueID::kCommonLigatures) => (0, kEnabledLigaturesState),
                    Some(CSSValueID::kNoDiscretionaryLigatures) => (1, kDisabledLigaturesState),
                    Some(CSSValueID::kDiscretionaryLigatures) => (1, kEnabledLigaturesState),
                    Some(CSSValueID::kNoHistoricalLigatures) => (2, kDisabledLigaturesState),
                    Some(CSSValueID::kHistoricalLigatures) => (2, kEnabledLigaturesState),
                    Some(CSSValueID::kNoContextual) => (3, kDisabledLigaturesState),
                    Some(CSSValueID::kContextual) => (3, kEnabledLigaturesState),
                    _ => unreachable!("ligatures list"),
                };
                states[index] = state;
            }
        } else if !b.IsPendingSystemFont(v) && b.Identifier(v) == Some(CSSValueID::kNone) {
            states = [kDisabledLigaturesState; 4];
        }
        b.NewLigatures(states[0], states[1], states[2], states[3])
    }
    pub fn ConvertFontVariantNumeric<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> FontVariantNumeric {
        let mut result = FontVariantNumeric::default();
        if b.Identifier(v).is_some() || b.IsPendingSystemFont(v) {
            return result;
        }
        for item in b.List(v).expect("numeric list") {
            match b.Identifier(item) {
                Some(CSSValueID::kLiningNums) => {
                    result.SetNumericFigure(NumericFigure::kLiningNums)
                }
                Some(CSSValueID::kOldstyleNums) => {
                    result.SetNumericFigure(NumericFigure::kOldstyleNums)
                }
                Some(CSSValueID::kProportionalNums) => {
                    result.SetNumericSpacing(NumericSpacing::kProportionalNums)
                }
                Some(CSSValueID::kTabularNums) => {
                    result.SetNumericSpacing(NumericSpacing::kTabularNums)
                }
                Some(CSSValueID::kDiagonalFractions) => {
                    result.SetNumericFraction(NumericFraction::kDiagonalFractions)
                }
                Some(CSSValueID::kStackedFractions) => {
                    result.SetNumericFraction(NumericFraction::kStackedFractions)
                }
                Some(CSSValueID::kOrdinal) => result.SetOrdinal(Ordinal::kOrdinalOn),
                Some(CSSValueID::kSlashedZero) => {
                    result.SetSlashedZero(SlashedZero::kSlashedZeroOn)
                }
                _ => unreachable!("numeric feature"),
            }
        }
        result
    }
    pub fn ConvertFontVariantAlternates<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Option<Arc<FontVariantAlternates>> {
        if b.Identifier(v).is_some() || b.IsPendingSystemFont(v) {
            return None;
        }
        let mut result = b.NewAlternates();
        for item in b.List(v).expect("alternate list") {
            if let Some((function, aliases)) = b.Alternate(item) {
                match function {
                    CSSValueID::kStylistic
                    | CSSValueID::kSwash
                    | CSSValueID::kOrnaments
                    | CSSValueID::kAnnotation => {
                        let alias = FirstEntryAsAtomicString(b, &aliases);
                        b.SetAlternate(&mut result, function, vec![alias]);
                    }
                    CSSValueID::kStyleset | CSSValueID::kCharacterVariant => b.SetAlternate(
                        &mut result,
                        function,
                        ValueListToAtomicStringVector(b, &aliases),
                    ),
                    _ => unreachable!("alternate function"),
                }
            }
            if b.Identifier(item).is_some() {
                b.SetHistoricalForms(&mut result);
            }
        }
        if b.AlternatesIsNormal(&result) {
            None
        } else {
            Some(Arc::new(result))
        }
    }
    pub fn ConvertFontVariantEastAsian<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> FontVariantEastAsian {
        let mut result = FontVariantEastAsian::default();
        if b.Identifier(v).is_some() || b.IsPendingSystemFont(v) {
            return result;
        }
        for item in b.List(v).expect("east asian list") {
            match b.Identifier(item) {
                Some(CSSValueID::kJis78) => result.SetForm(EastAsianForm::kJis78),
                Some(CSSValueID::kJis83) => result.SetForm(EastAsianForm::kJis83),
                Some(CSSValueID::kJis90) => result.SetForm(EastAsianForm::kJis90),
                Some(CSSValueID::kJis04) => result.SetForm(EastAsianForm::kJis04),
                Some(CSSValueID::kSimplified) => result.SetForm(EastAsianForm::kSimplified),
                Some(CSSValueID::kTraditional) => result.SetForm(EastAsianForm::kTraditional),
                Some(CSSValueID::kFullWidth) => result.SetWidth(EastAsianWidth::kFullWidth),
                Some(CSSValueID::kProportionalWidth) => {
                    result.SetWidth(EastAsianWidth::kProportionalWidth)
                }
                Some(CSSValueID::kRuby) => result.SetRuby(true),
                _ => unreachable!("east asian feature"),
            }
        }
        result
    }
    pub fn ConvertSelfOrDefaultAlignmentData<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> StyleSelfAlignmentData {
        let mut result = ComputedStyleInitialValues::InitialAlignSelf();
        if let Some((first, second)) = b.Pair(v) {
            match b.Identifier(first) {
                Some(CSSValueID::kLegacy) => {
                    result.SetPositionType(ItemPositionType::kLegacy);
                    result.SetPosition(b.ItemPosition(second));
                }
                Some(CSSValueID::kFirst) => result.SetPosition(ItemPosition::kBaseline),
                Some(CSSValueID::kLast) => result.SetPosition(ItemPosition::kLastBaseline),
                _ => {
                    result.SetOverflow(b.OverflowAlignment(first));
                    result.SetPosition(b.ItemPosition(second));
                }
            }
        } else {
            result.SetPosition(b.ItemPosition(v));
        }
        result
    }
    pub fn ConvertContentAlignmentData<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> StyleContentAlignmentData {
        let mut result = ComputedStyleInitialValues::InitialContentAlignment();
        let (distribution, position, overflow) = b.ContentDistributionIds(v);
        if foundation::IsValidCSSValueID(distribution) {
            result.SetDistribution(b.DistributionFromIdentifier(distribution));
        }
        if foundation::IsValidCSSValueID(position) {
            result.SetPosition(b.ContentPositionFromIdentifier(position));
        }
        if foundation::IsValidCSSValueID(overflow) {
            result.SetOverflow(b.OverflowFromIdentifier(overflow));
        }
        result
    }
    pub fn ConvertGridAutoFlow<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> GridAutoFlow {
        let list = b.List(v).unwrap_or_else(|| vec![v]);
        let second = if list.len() == 2 {
            b.Identifier(list[1])
        } else {
            None
        };
        match b.Identifier(list[0]) {
            Some(CSSValueID::kRow) => {
                if second == Some(CSSValueID::kDense) {
                    GridAutoFlow::kAutoFlowRowDense
                } else {
                    GridAutoFlow::kAutoFlowRow
                }
            }
            Some(CSSValueID::kColumn) => {
                if second == Some(CSSValueID::kDense) {
                    GridAutoFlow::kAutoFlowColumnDense
                } else {
                    GridAutoFlow::kAutoFlowColumn
                }
            }
            Some(CSSValueID::kDense) => {
                if second == Some(CSSValueID::kColumn) {
                    GridAutoFlow::kAutoFlowColumnDense
                } else {
                    GridAutoFlow::kAutoFlowRowDense
                }
            }
            _ => unreachable!("grid flow"),
        }
    }
    pub fn ConvertGridPosition<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> GridPosition {
        let mut position = GridPosition::default();
        if let Some(name) = b.CustomIdent(v) {
            position.SetNamedGridArea(&name);
            return position;
        }
        if b.Identifier(v).is_some() {
            return position;
        }
        let list = b.List(v).expect("grid position list");
        let mut i = 0;
        let span = b.Identifier(list[0]) == Some(CSSValueID::kSpan);
        if span {
            i += 1;
        }
        let mut number = 1;
        let mut name = AtomicString::default();
        if let Some(value) = list.get(i) {
            if b.IsNumber(value) {
                let n = b.ComputeInteger(&s.CssToLengthConversionData(), value);
                if span {
                    number = n.clamp(1, b.GridMaxTracks());
                } else if n != 0 {
                    number = n.clamp(-b.GridMaxTracks(), b.GridMaxTracks());
                }
                i += 1;
            }
        }
        if let Some(value) = list.get(i) {
            if let Some(ident) = b.CustomIdent(value) {
                name = ident;
            }
        }
        if span {
            position.SetSpanPosition(number, &name);
        } else {
            position.SetExplicitPosition(number, &name);
        }
        position
    }
    pub fn ConvertGridTemplateAreas<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Option<B::GridTemplateAreas> {
        if b.Identifier(v).is_some() {
            None
        } else {
            {
                let (areas, rows, columns) = b.GridTemplateAreaParts(v);
                Some(b.NewGridTemplateAreas(areas, rows, columns))
            }
        }
    }
    pub fn ConvertGridTrackSize<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> GridTrackSize {
        if v.IsPrimitiveValue() || b.Identifier(v).is_some() {
            return GridTrackSize::from_length(&ConvertGridTrackBreadth(b, s, v));
        }
        let list = b.List(v).expect("grid sizing function");
        if b.FunctionType(v) == CSSValueID::kFitContent {
            GridTrackSize::new(
                &ConvertGridTrackBreadth(b, s, list[0]),
                GridTrackSizeType::kFitContentTrackSizing,
            )
        } else {
            GridTrackSize::new_minmax(
                &ConvertGridTrackBreadth(b, s, list[0]),
                &ConvertGridTrackBreadth(b, s, list[1]),
            )
        }
    }
    pub fn ConvertLength<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        b.PrimitiveLength(&s.CssToLengthConversionData(), v)
    }
    pub fn ConvertLengthOrAuto<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        if b.Identifier(v) == Some(CSSValueID::kAuto) {
            Length::Auto().clone()
        } else {
            Self::ConvertLength(b, s, v)
        }
    }
    pub fn ConvertInteger<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> i32 {
        b.ComputeInteger(&s.CssToLengthConversionData(), v)
    }
    pub fn ConvertNoneOrCustomIdentUnscoped<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> AtomicString {
        if b.Identifier(v).is_some() {
            AtomicString::default()
        } else {
            Self::ConvertCustomIdentUnscoped(b, s, v)
        }
    }
    pub fn ConvertCustomIdentUnscoped<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> AtomicString {
        b.ComputeIdent(s, v)
    }
    // Header template numeric results retain generic target clamping semantics.
    pub fn ConvertComputedLength<B: StyleBuilderConverterBackend, T: ConversionNumber>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> T {
        T::FromComputed(b.ComputeLength(&s.CssToLengthConversionData(), v))
    }
    pub fn ConvertLineWidth<B: StyleBuilderConverterBackend, T: ConversionNumber>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> T {
        let result = if let Some(id) = b.Identifier(v) {
            let pixels = match id {
                CSSValueID::kThin => 1.0,
                CSSValueID::kMedium => 3.0,
                CSSValueID::kThick => 5.0,
                _ => unreachable!("line width"),
            };
            b.ZoomedPixels(&s.CssToLengthConversionData(), pixels)
        } else {
            b.ComputeLength(&s.CssToLengthConversionData(), v)
        };
        let zoomed = b.EffectiveZoom(s) as f64 * result;
        if zoomed > 0.0 && zoomed < 1.0 {
            T::FromComputed(1.0)
        } else {
            T::RoundAndClamp(result)
        }
    }
    pub fn ConvertPositionLength<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        zero: CSSValueID,
        hundred: CSSValueID,
    ) -> Length {
        if let Some((first, second)) = b.Pair(v) {
            let length = Self::ConvertLength(b, s, second);
            if b.Identifier(first) == Some(zero) {
                return length;
            }
            return length.SubtractFromOneHundredPercent();
        }
        if let Some(id) = b.Identifier(v) {
            return Length::Percent(if id == zero {
                0
            } else if id == hundred {
                100
            } else if id == CSSValueID::kCenter {
                50
            } else {
                unreachable!("position keyword")
            });
        }
        Self::ConvertLength(b, s, v)
    }
    pub fn ConvertFlags<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
        zero: CSSValueID,
    ) -> u32 {
        if b.Identifier(v) == Some(zero) {
            return 0;
        }
        Self::ConvertFlagsFromBits(b.List(v).expect("flags list").iter().map(|value| b.IdentifierFlags(value)))
    }
    // style_builder_converter.h:490-502. Shared native flag accumulation for
    // the generic converter and typed production values.
    pub fn ConvertFlagsFromBits(bits: impl IntoIterator<Item = u32>) -> u32 {
        bits.into_iter().fold(0, |flags, value| flags | value)
    }
    pub fn ConvertString<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> AtomicString {
        b.StringValue(v).unwrap_or_default()
    }
    pub fn ConvertIntegerOrNone<B: StyleBuilderConverterBackend, const NONE: i32>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> i32 {
        if v.IsPrimitiveValue() {
            Self::ConvertInteger(b, s, v)
        } else {
            NONE
        }
    }
}
/// Target-type behavior of ComputeLength/ClampTo/RoundForImpreciseConversion.
/// Implementors must supply the real destination numeric representation.
pub trait ConversionNumber {
    fn FromComputed(value: f64) -> Self;
    fn RoundAndClamp(value: f64) -> Self;
}
impl ConversionNumber for f64 {
    fn FromComputed(v: f64) -> Self {
        v
    }
    fn RoundAndClamp(v: f64) -> Self {
        v
    }
}
impl ConversionNumber for f32 {
    fn FromComputed(v: f64) -> Self {
        v as f32
    }
    fn RoundAndClamp(v: f64) -> Self {
        v.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
    }
}
impl ConversionNumber for i32 {
    fn FromComputed(v: f64) -> Self {
        v as i32
    }
    fn RoundAndClamp(v: f64) -> Self {
        {
            let adjusted = v + if v < 0.0 { -0.01 } else { 0.01 };
            if adjusted < i32::MIN as f64 || adjusted > i32::MAX as f64 {
                0
            } else {
                adjusted as i32
            }
        }
    }
}

impl StyleBuilderConverterBase {
    pub fn ConvertFontFamily<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        has_builder: bool,
    ) -> B::FamilyDescription {
        if b.IsPendingSystemFont(v) {
            return b.NewFamilyDescription(
                GenericFamilyType::kNoFamily,
                FontFamily::new(
                    b.ResolveSystemFontFamily(v),
                    FontFamilyType::kFamilyName,
                    None,
                ),
            );
        }
        let mut generic = GenericFamilyType::kNoFamily;
        let mut name = AtomicString::default();
        let mut family_type = FontFamilyType::kFamilyName;
        let mut next = None;
        let mut has_value = false;
        for value in b.List(v).expect("font families").into_iter().rev() {
            let Some((family, next_name)) = ConvertFontFamilyName(b, s, value, has_builder) else {
                continue;
            };
            let is_generic =
                family != GenericFamilyType::kNoFamily || b.Identifier(value).is_some();
            if has_value {
                next = Some(SharedFontFamily::Create(name, family_type, next));
            }
            name = next_name;
            family_type = if is_generic {
                FontFamilyType::kGenericFamily
            } else {
                FontFamilyType::kFamilyName
            };
            has_value = true;
            if let Some(legacy) = b.LegacyMacSystemFontFamily() {
                if b.FontFamilyValue(value).is_some() && name == legacy {
                    name = AtomicString::from_str("system-ui");
                }
            }
            if generic == GenericFamilyType::kNoFamily {
                generic = family;
            }
        }
        b.NewFamilyDescription(generic, FontFamily::new(name, family_type, next))
    }
    pub fn ConvertFontFeatureSettings<B: StyleBuilderConverterBackend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
    ) -> FontFeatureSettings {
        let mut settings = FontSettings::default();
        if b.Identifier(v) == Some(CSSValueID::kNormal) || b.IsPendingSystemFont(v) {
            return Arc::new(settings);
        }
        let mut features = BTreeMap::new();
        for value in b.List(v).expect("font features") {
            let (tag, value) = b.FontFeature(data, value);
            features.insert(tag, value);
        }
        for (tag, value) in features {
            settings.Append(FontFeature::new(tag, value));
        }
        Arc::new(settings)
    }
    pub fn ConvertFontPalette<B: StyleBuilderConverterBackend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
    ) -> Option<Arc<FontPalette>> {
        match b.Identifier(v) {
            Some(CSSValueID::kNormal) => return None,
            Some(CSSValueID::kDark) => {
                return Some(FontPalette::CreateKeyword(KeywordPaletteName::kDarkPalette))
            }
            Some(CSSValueID::kLight) => {
                return Some(FontPalette::CreateKeyword(
                    KeywordPaletteName::kLightPalette,
                ))
            }
            _ => {}
        }
        if let Some(name) = b.CustomIdent(v) {
            return Some(FontPalette::CreateCustom(name));
        }
        Self::ConvertPaletteMix(b, data, v)
    }
    pub fn ConvertPaletteMix<B: StyleBuilderConverterBackend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
    ) -> Option<Arc<FontPalette>> {
        let (first, second, p1, p2, space, hue) = b.PaletteMix(v)?;
        let first = Self::ConvertFontPalette(b, data, first).unwrap_or_else(FontPalette::Create);
        let second = Self::ConvertFontPalette(b, data, second).unwrap_or_else(FontPalette::Create);
        let (normalized, alpha) = b.NormalizeColorMixPercentages(data, p1, p2)?;
        let (a, z) = match (p1, p2) {
            (Some(a), Some(z)) => (b.Percentage(data, a), b.Percentage(data, z)),
            (Some(a), None) => {
                let a = b.Percentage(data, a);
                (a, 100.0 - a)
            }
            (None, Some(z)) => {
                let z = b.Percentage(data, z);
                (100.0 - z, z)
            }
            (None, None) => (50.0, 50.0),
        };
        Some(FontPalette::Mix(
            first, second, a, z, normalized, alpha, space, hue,
        ))
    }
    pub fn ConvertFontSize<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        data: &B::LengthConversionData,
        parent: B::FontSize,
    ) -> B::FontSize {
        if let Some(id) = b.Identifier(v) {
            if let Some(keyword) = b.FontSizeKeyword(id) {
                return b.NewFontSize(keyword, 0.0, false);
            }
            return match id {
                CSSValueID::kSmaller => b.SmallerFontSize(parent),
                CSSValueID::kLarger => b.LargerFontSize(parent),
                _ => unreachable!("font size keyword"),
            };
        }
        if b.IsPendingSystemFont(v) {
            return b.NewFontSize(0, b.ResolveSystemFontSize(s, v), true);
        }
        let (size, absolute) = b.SizeParts(&parent);
        if b.IsPercentage(v) {
            return b.NewFontSize(0, b.Percentage(data, v) as f32 * size / 100.0, absolute);
        }
        let absolute =
            absolute || v.IsMathFunctionValue() || !b.IsFontRelativeLength(v) || b.IsRem(v);
        let computed = if b.IsLength(v) {
            b.ComputeLength(data, v) as f32
        } else if b.IsCalculated(v) {
            b.EvaluateFontSizeCalc(data, v, size)
        } else {
            unreachable!("font size numeric")
        };
        b.NewFontSize(0, computed, absolute)
    }
    pub fn ConvertFontSizeAdjust<B: StyleBuilderConverterBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> FontSizeAdjust {
        if b.Identifier(v) == Some(CSSValueID::kNone) || b.IsPendingSystemFont(v) {
            return b.InitialSizeAdjust();
        }
        if b.Identifier(v) == Some(CSSValueID::kFromFont) {
            return FontSizeAdjust::with_type(-1.0, ValueType::kFromFont);
        }
        if v.IsPrimitiveValue() {
            return FontSizeAdjust::new(b.Number(&s.CssToLengthConversionData(), v) as f32);
        }
        let (first, second) = b.Pair(v).expect("size adjust pair");
        let metric = b.SizeAdjustMetric(first);
        if second.IsPrimitiveValue() {
            FontSizeAdjust::with_metric(
                b.Number(&s.CssToLengthConversionData(), second) as f32,
                metric,
            )
        } else {
            FontSizeAdjust::with_metric_and_type(-1.0, metric, ValueType::kFromFont)
        }
    }
    pub fn ConvertFontStretch<B: StyleBuilderConverterBackend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
    ) -> FontSelectionValue {
        if v.IsPrimitiveValue() && b.IsPercentage(v) {
            return FontSelectionValue::from_double(b.Percentage(data, v));
        }
        if let Some(keyword) = StyleBuilderConverter::ConvertFontStretchKeyword(b, v) {
            return keyword;
        }
        if b.IsPendingSystemFont(v) {
            return kNormalWidthValue;
        }
        unreachable!("parsed font stretch")
    }
    pub fn ConvertFontStyle<B: StyleBuilderConverterBackend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
    ) -> FontSelectionValue {
        if let Some(id) = b.Identifier(v) {
            return match id {
                CSSValueID::kItalic | CSSValueID::kOblique => kItalicSlopeValue,
                CSSValueID::kNormal => kNormalSlopeValue,
                _ => unreachable!("font style"),
            };
        }
        if b.IsPendingSystemFont(v) {
            return kNormalSlopeValue;
        }
        if let Some((style, angles)) = b.ObliqueRange(v) {
            assert!(angles.len() < 2);
            if let Some(angle) = angles.first() {
                let degrees = b.Degrees(data, angle);
                if degrees == 0.0 && b.ObliqueZeroAsNormalEnabled() {
                    return kNormalSlopeValue;
                }
                return FontSelectionValue::from_double(degrees);
            }
            match b.Identifier(style) {
                Some(CSSValueID::kNormal) => return kNormalSlopeValue,
                Some(CSSValueID::kItalic | CSSValueID::kOblique) => return kItalicSlopeValue,
                _ => {}
            }
        }
        unreachable!("font oblique range")
    }
    pub fn ConvertFontWeight<B: StyleBuilderConverterBackend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
        parent: FontSelectionValue,
    ) -> FontSelectionValue {
        if v.IsPrimitiveValue() && b.IsNumber(v) {
            return FontSelectionValue::from_double(b.Number(data, v).clamp(1.0, 1000.0));
        }
        if b.IsPendingSystemFont(v) {
            return kNormalWeightValue;
        }
        match b.Identifier(v) {
            Some(CSSValueID::kNormal) => kNormalWeightValue,
            Some(CSSValueID::kBold) => kBoldWeightValue,
            Some(CSSValueID::kBolder) => b.BolderWeight(parent),
            Some(CSSValueID::kLighter) => b.LighterWeight(parent),
            _ => unreachable!("font weight"),
        }
    }
    pub fn ConvertFontVariantCaps<B: StyleBuilderConverterBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> FontVariantCaps {
        use FontVariantCaps::*;
        if b.IsPendingSystemFont(v) {
            return kCapsNormal;
        }
        match b.Identifier(v) {
            Some(CSSValueID::kSmallCaps) => kSmallCaps,
            Some(CSSValueID::kAllSmallCaps) => kAllSmallCaps,
            Some(CSSValueID::kPetiteCaps) => kPetiteCaps,
            Some(CSSValueID::kAllPetiteCaps) => kAllPetiteCaps,
            Some(CSSValueID::kUnicase) => kUnicase,
            Some(CSSValueID::kTitlingCaps) => kTitlingCaps,
            _ => kCapsNormal,
        }
    }
}
fn ConvertGenericFamily(id: CSSValueID) -> GenericFamilyType {
    match id {
        CSSValueID::kWebkitBody => GenericFamilyType::kWebkitBodyFamily,
        CSSValueID::kSerif => GenericFamilyType::kSerifFamily,
        CSSValueID::kSansSerif => GenericFamilyType::kSansSerifFamily,
        CSSValueID::kCursive => GenericFamilyType::kCursiveFamily,
        CSSValueID::kFantasy => GenericFamilyType::kFantasyFamily,
        CSSValueID::kMonospace => GenericFamilyType::kMonospaceFamily,
        _ => GenericFamilyType::kNoFamily,
    }
}
fn ConvertFontFamilyName<B: StyleBuilderConverterBackend>(
    b: &B,
    s: &StyleResolverState<B>,
    v: &ResolverValue<B>,
    has_builder: bool,
) -> Option<(GenericFamilyType, AtomicString)> {
    if let Some(name) = b.FontFamilyValue(v) {
        return Some((GenericFamilyType::kNoFamily, name));
    }
    let mut family = GenericFamilyType::kNoFamily;
    let mut name = AtomicString::default();
    if has_builder {
        let id = b.Identifier(v).expect("font family identifier");
        family = ConvertGenericFamily(id);
        if family != GenericFamilyType::kNoFamily {
            name = b.GenericFontFamilyName(s, family);
        } else if id == CSSValueID::kSystemUi {
            name = AtomicString::from_str("system-ui");
        } else if id == CSSValueID::kMath {
            name = AtomicString::from_str("math");
        }
        if name.length() == 0 {
            return None;
        }
    }
    if name.IsNull() {
        None
    } else {
        Some((family, name))
    }
}
fn ConvertGridTrackBreadth<B: StyleBuilderConverterBackend>(
    b: &B,
    s: &StyleResolverState<B>,
    v: &ResolverValue<B>,
) -> Length {
    if v.IsPrimitiveValue() && b.IsFlex(v) {
        return Length::Flex(b.CanonicalUnit(&s.CssToLengthConversionData(), v) as f32);
    }
    match b.Identifier(v) {
        Some(CSSValueID::kMinContent) => Length::MinContent().clone(),
        Some(CSSValueID::kMaxContent) => Length::MaxContent().clone(),
        _ => StyleBuilderConverter::ConvertLengthOrAuto(b, s, v),
    }
}
fn ValueListToAtomicStringVector<B: StyleBuilderConverterBackend>(
    b: &B,
    values: &[&ResolverValue<B>],
) -> Vec<AtomicString> {
    values
        .iter()
        .map(|v| b.CustomIdent(v).expect("custom identifier"))
        .collect()
}
fn FirstEntryAsAtomicString<B: StyleBuilderConverterBackend>(
    b: &B,
    values: &[&ResolverValue<B>],
) -> AtomicString {
    b.CustomIdent(values[0]).expect("custom identifier")
}
pub fn IsQuirkOrLinkOrFocusRingColor(id: CSSValueID) -> bool {
    matches!(
        id,
        CSSValueID::kInternalQuirkInherit
            | CSSValueID::kWebkitLink
            | CSSValueID::kWebkitActivelink
            | CSSValueID::kWebkitFocusRingColor
    )
}
pub fn ResolveQuirkOrLinkOrFocusRingColor<B: StyleBuilderConverterBackend>(
    b: &B,
    id: CSSValueID,
    colors: &B::TextLinkColors,
    scheme: ColorScheme,
    visited: bool,
) -> Color {
    match id {
        CSSValueID::kInternalQuirkInherit => b.TextColor(colors, scheme),
        CSSValueID::kWebkitLink => {
            if visited {
                b.VisitedLinkColor(colors, scheme)
            } else {
                b.LinkColor(colors, scheme)
            }
        }
        CSSValueID::kWebkitActivelink => b.ActiveLinkColor(colors, scheme),
        CSSValueID::kWebkitFocusRingColor => b.FocusRingColor(scheme),
        _ => unreachable!("quirk color"),
    }
}
pub fn ConvertNoneOrCustomIdentListUnscoped<B: StyleBuilderConverterBackend>(
    b: &B,
    s: &StyleResolverState<B>,
    v: &ResolverValue<B>,
) -> Vec<AtomicString> {
    b.List(v)
        .expect("custom ident list")
        .iter()
        .map(|v| StyleBuilderConverter::ConvertNoneOrCustomIdentUnscoped(b, s, v))
        .collect()
}

use foundation::{
    EBorderStyle, EInsideLink, LayoutUnit, LengthPoint, LengthSize, Member, ScopedCSSName,
    ScopedCSSNameList, StyleInitialLetter, StyleNameScope, StyleNameScopeType,
    TextDecorationThickness, TreeScope,
};
use layoutng_style::{
    css::{
        style_auto_color::StyleAutoColor, style_caret_color::StyleCaretColor,
        style_color::StyleColor,
    },
    style::{
        computed_grid_track_list::ComputedGridTrackList,
        flow_tolerance::FlowTolerance,
        gap_data::{GapData, GapValue, ValueRepeater},
        gap_data_list::GapDataList,
        grid_lanes_direction::{GridLanesDirection, GridLanesOrientation},
        grid_track_list::{AutoRepeatType, GridAxisType, GridTrackList, GridTrackRepeatType},
        named_grid_lines_map::NamedGridLinesMap,
        ordered_named_grid_lines::{NamedGridLine, OrderedNamedGridLines},
        shadow_data::{ShadowData, ShadowStyle},
        shadow_list::ShadowList,
        shape_value::ShapeValue,
        style_hyphenate_limit_chars::StyleHyphenateLimitChars,
        style_interest_delay::StyleInterestDelay,
        style_offset_rotation::StyleOffsetRotation,
        style_position_anchor::{StylePositionAnchor, Type as AnchorType},
        superellipse::Superellipse,
        svg_paint::{SVGPaint, SVGPaintType},
        text_decoration_inset::TextDecorationInset,
        text_size_adjust::TextSizeAdjust,
        transform_origin::TransformOrigin,
    },
};
// cpp: style_builder_converter.cc:1846-1895. One native conversion shared by
// the source backend and production typed value bridge.
pub fn ConvertGridLanesDirectionIdentifiers(values: &[CSSValueID]) -> Option<GridLanesDirection> {
    if values == [CSSValueID::kNormal] { return Some(GridLanesDirection::default()); }
    if values.is_empty() || values.len() > 3 { return None; }
    let orientation = match values[0] {
        CSSValueID::kRow => GridLanesOrientation::kRow,
        CSSValueID::kColumn => GridLanesOrientation::kColumn,
        _ => return None,
    };
    let (mut fill, mut track) = (false, false);
    for &value in &values[1..] {
        match value {
            CSSValueID::kFillReverse if !fill => fill = true,
            CSSValueID::kTrackReverse if !track => track = true,
            _ => return None,
        }
    }
    Some(GridLanesDirection::new(orientation, fill, track))
}

/// Second batch external owners. These APIs only read actual subclasses or
/// create/update objects owned by grid/style/GC/DOM/Color/TransformBuilder.
pub trait StyleBuilderConverterBatch2Backend: StyleBuilderConverterBackend {
    type TabSize;
    type ColorProvider;
    type UnresolvedColor;
    fn IsGridLineNames(&self, v: &ResolverValue<Self>) -> bool;
    fn GridAutoRepeat(&self, v: &ResolverValue<Self>) -> Option<CSSValueID>;
    fn GridIntegerRepeat(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
    ) -> Option<u32>;
    fn ScrollMarkerPosition(&self, v: &ResolverValue<Self>) -> ScrollMarkerPosition;
    fn ScrollMarkerMode(&self, v: &ResolverValue<Self>) -> ScrollMarkerMode;
    fn TimeSeconds(&self, data: &Self::LengthConversionData, v: &ResolverValue<Self>) -> f64;
    fn SuperellipseParam<'a>(&self, v: &'a ResolverValue<Self>) -> &'a ResolverValue<Self>;
    fn NumericLiteralDouble(&self, v: &ResolverValue<Self>) -> Option<f64>;
    fn MathKnownValue(&self, v: &ResolverValue<Self>) -> Option<f64>;
    fn NewTabSize(&self, value: f32, is_spaces: bool) -> Self::TabSize;
    fn FrameTextZoom(&self, s: &StyleResolverState<Self>) -> Option<f32>;
    fn TextSizeAdjust(&self, s: &StyleResolverState<Self>) -> TextSizeAdjust;
    fn TextSizeAdjustEnabled(&self, s: &StyleResolverState<Self>) -> bool;
    fn CopyWithAdjustedZoom(
        &self,
        data: &Self::LengthConversionData,
        multiplier: f32,
    ) -> Self::LengthConversionData;
    fn ComputedFontSize(&self, s: &StyleResolverState<Self>) -> f32;
    fn CalcLength(&self, data: &Self::LengthConversionData, v: &ResolverValue<Self>) -> Length;
    fn ValueForLength(&self, value: &Length, maximum: LayoutUnit) -> LayoutUnit;
    fn PrimitiveNumberOrPercentage(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
    ) -> f32;
    fn PopulatedTreeScope(&self, v: &ResolverValue<Self>) -> *const TreeScope;
    fn IsScopedKeyword(&self, v: &ResolverValue<Self>) -> bool;
    fn AllocateScopedName(&self, name: ScopedCSSName) -> Member<ScopedCSSName>;
    fn AllocateScopedNameList(
        &self,
        names: Vec<Option<Member<ScopedCSSName>>>,
    ) -> *const ScopedCSSNameList;
    fn PositionVisibility(&self, v: &ResolverValue<Self>) -> PositionVisibility;
    fn IsQuirkyEms(&self, v: &ResolverValue<Self>) -> bool;
    fn AddQuotePair(
        &self,
        quotes: &mut foundation::ScopedRefPtr<foundation::QuotesData>,
        first: foundation::String,
        last: foundation::String,
    );
    fn StringContents(&self, v: &ResolverValue<Self>) -> foundation::String;
    fn GapRepeat<'a>(
        &self,
        v: &'a ResolverValue<Self>,
    ) -> Option<(
        Vec<&'a ResolverValue<Self>>,
        Option<&'a ResolverValue<Self>>,
    )>;
    fn AllocateGapRepeater<T: GapValue>(
        &self,
        values: Vec<T>,
        count: Option<u32>,
    ) -> *mut ValueRepeater<T>;
    fn BorderStyle(&self, v: &ResolverValue<Self>) -> EBorderStyle;
    fn ShadowParts<'a>(
        &self,
        v: &'a ResolverValue<Self>,
    ) -> (
        &'a ResolverValue<Self>,
        &'a ResolverValue<Self>,
        Option<&'a ResolverValue<Self>>,
        Option<&'a ResolverValue<Self>>,
        Option<CSSValueID>,
        Option<&'a ResolverValue<Self>>,
    );
    fn BlackTextLinkColors(&self) -> Self::TextLinkColors;
    fn AllocateShadowList(&self, shadows: Vec<ShadowData>) -> ShadowList;
    fn ShapeBox(&self, v: &ResolverValue<Self>) -> ShapeBox;
    fn StyleImageForShape(
        &self,
        s: &StyleResolverState<Self>,
        v: &ResolverValue<Self>,
    ) -> *mut layoutng_style::style::style_image::StyleImage;
    fn DocumentTreeScope(&self, s: &StyleResolverState<Self>) -> *const TreeScope;
    fn NumericColor(&self, v: &ResolverValue<Self>) -> Option<Color>;
    fn ResolveNonFiniteColor(&self, color: &mut Color);
    fn ColorFromKeyword(
        &self,
        id: CSSValueID,
        scheme: ColorScheme,
        provider: Option<&Self::ColorProvider>,
        expose_accent: bool,
    ) -> Color;
    fn IsSystemColorKeyword(&self, id: CSSValueID) -> bool;
    fn ColorMix<'a>(
        &self,
        v: &'a ResolverValue<Self>,
    ) -> Option<(
        &'a ResolverValue<Self>,
        &'a ResolverValue<Self>,
        ColorSpace,
        Option<HueInterpolationMethod>,
    )>;
    fn ColorMixNormalizedPercentages(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
    ) -> (f64, f64);
    fn NewUnresolvedColorMix(
        &self,
        space: ColorSpace,
        hue: Option<HueInterpolationMethod>,
        first: StyleColor,
        second: StyleColor,
        amount: f64,
        alpha: f64,
    ) -> Self::UnresolvedColor;
    fn RelativeColorOrigin<'a>(
        &self,
        v: &'a ResolverValue<Self>,
    ) -> Option<&'a ResolverValue<Self>>;
    fn NewUnresolvedRelativeColor(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
        origin: StyleColor,
    ) -> Self::UnresolvedColor;
    fn ContrastColor<'a>(&self, v: &'a ResolverValue<Self>) -> Option<&'a ResolverValue<Self>>;
    fn NewUnresolvedContrastColor(&self, param: StyleColor) -> Self::UnresolvedColor;
    fn AlphaColorOrigin<'a>(&self, v: &'a ResolverValue<Self>) -> Option<&'a ResolverValue<Self>>;
    fn NewUnresolvedAlphaColor(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
        origin: StyleColor,
    ) -> Self::UnresolvedColor;
    fn ResolveUnresolvedColor(&self, unresolved: &Self::UnresolvedColor, current: Color) -> Color;
    fn StyleColorFromUnresolved(&self, unresolved: Self::UnresolvedColor) -> StyleColor;
    fn CSSUnresolvedColor(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
    ) -> Option<Color>;
    fn LightDarkPair<'a>(
        &self,
        v: &'a ResolverValue<Self>,
    ) -> Option<(&'a ResolverValue<Self>, &'a ResolverValue<Self>)>;
    fn IsLegacyColorSpace(&self, space: ColorSpace) -> bool;
    fn ConvertColorSpace(&self, color: &mut Color, space: ColorSpace, resolve_missing: bool);
    fn UsedColorScheme(&self, s: &StyleResolverState<Self>) -> ColorScheme;
    fn DocumentTextLinkColors(&self, s: &StyleResolverState<Self>) -> Self::TextLinkColors;
    fn ColorProviderForPainting(
        &self,
        s: &StyleResolverState<Self>,
        scheme: ColorScheme,
    ) -> Option<Self::ColorProvider>;
    fn IsInWebAppScope(&self, s: &StyleResolverState<Self>) -> bool;
    fn IsInitialProfile(&self, s: &StyleResolverState<Self>) -> bool;
    fn NewSVGPaint(
        &self,
        kind: SVGPaintType,
        resource: Option<Self::SVGStyleResource>,
        color: Option<StyleColor>,
    ) -> SVGPaint;
    fn TextBoxEdgeType(&self, v: &ResolverValue<Self>) -> TextBoxEdgeType;
    fn TextEmphasisAutoEnabled(&self) -> bool;
    fn ComputeEms(&self, data: &Self::LengthConversionData, value: f32) -> f32;
    fn TextUnderlinePositionFlag(&self, v: &ResolverValue<Self>) -> TextUnderlinePosition;
    fn CreateTransformOperations(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
    ) -> foundation::TransformOperations;
}

impl StyleBuilderConverter {
    // cpp: style_builder_converter.cc:1660-1680,1826-1833. The native plain
    // track construction is shared by both generic and production value owners.
    pub fn ConvertGridTrackSizeListFromSizes(sizes: &[GridTrackSize]) -> GridTrackList {
        let mut result = GridTrackList::default();
        result.AddRepeaterDefault(&sizes.to_vec());
        result
    }
    pub fn ConvertPlainGridTrackListFromSizes(sizes: &[GridTrackSize]) -> ComputedGridTrackList {
        let mut result = ComputedGridTrackList::new(&GridTrackList::default(), AutoRepeatType::kNoAutoRepeat);
        for size in sizes {
            result.GetMutableTrackList().AddRepeaterDefault(&vec![size.clone()]);
        }
        result
    }
    // cc:1660-1680
    pub fn ConvertGridTrackSizeList<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> GridTrackList {
        let Some(list) = b.List(v) else {
            return GridTrackList::from_default_track_size(&GridTrackSize::from_length(
                Length::Auto(),
            ));
        };
        let tracks = list
            .into_iter()
            .map(|v| Self::ConvertGridTrackSize(b, s, v))
            .collect::<Vec<_>>();
        Self::ConvertGridTrackSizeListFromSizes(&tracks)
    }
    // cc:1682-1833: repeater selection, line-name expansion and insertion points.
    pub fn ConvertGridTrackList<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<ComputedGridTrackList> {
        if b.Identifier(v).is_some() {
            return None;
        }
        let mut result =
            ComputedGridTrackList::new(&GridTrackList::default(), AutoRepeatType::kNoAutoRepeat);
        let values = b.List(v).expect("grid track list");
        if values.iter().all(|value| !b.IsGridLineNames(value)
            && b.Identifier(value) != Some(CSSValueID::kSubgrid)
            && b.GridAutoRepeat(value).is_none()
            && b.GridIntegerRepeat(&s.CssToLengthConversionData(),value).is_none()) {
            let sizes = values.iter().map(|value| Self::ConvertGridTrackSize(b,s,value)).collect::<Vec<_>>();
            return Some(Self::ConvertPlainGridTrackListFromSizes(&sizes));
        }
        let mut index = 0;
        let mut line = 0;
        let subgrid = b.Identifier(values[0]) == Some(CSSValueID::kSubgrid);
        if subgrid {
            result.SetGridAxisType(GridAxisType::kSubgriddedAxis);
            result
                .GetMutableTrackList()
                .SetAxisType(GridAxisType::kSubgriddedAxis);
            index += 1;
        }
        for value in &values[index..] {
            if let Some(kind) = b.GridAutoRepeat(value) {
                let kind = if kind == CSSValueID::kAutoFill {
                    AutoRepeatType::kAutoFill
                } else {
                    AutoRepeatType::kAutoFit
                };
                result.SetAutoRepeatType(kind);
                let mut sizes = vec![];
                let mut auto_line = 0;
                for entry in b.List(value).expect("auto repeat list") {
                    if b.IsGridLineNames(entry) {
                        ConvertGridLineNames(
                            b,
                            entry,
                            auto_line,
                            result.GetMutableAutoRepeatNamedGridLines(),
                            false,
                            false,
                        );
                        ConvertOrderedGridLineNames(
                            b,
                            entry,
                            auto_line,
                            result.GetMutableOrderedAutoRepeatNamedGridLines(),
                            false,
                            false,
                        );
                        if subgrid {
                            auto_line += 1;
                        }
                    } else {
                        auto_line += 1;
                        sizes.push(Self::ConvertGridTrackSize(b, s, entry));
                    }
                }
                result.GetMutableTrackList().AddRepeater(
                    &sizes,
                    if kind == AutoRepeatType::kAutoFill {
                        GridTrackRepeatType::kAutoFill
                    } else {
                        GridTrackRepeatType::kAutoFit
                    },
                    1,
                    auto_line,
                );
                result.SetAutoRepeatInsertionPoint(line);
                line += 1;
                continue;
            }
            if let Some(repetitions) = b.GridIntegerRepeat(&s.CssToLengthConversionData(), value) {
                let entries = b.List(value).expect("integer repeat list");
                let mut name_count = 0;
                for repetition in 0..repetitions {
                    for entry in &entries {
                        let count = ConvertGridLineNameOrTrackSize(
                            b,
                            entry,
                            &mut result,
                            &mut line,
                            true,
                            repetition == 0,
                        );
                        if repetition == 0 {
                            name_count += count;
                        }
                    }
                }
                let sizes = if subgrid {
                    vec![]
                } else {
                    entries
                        .iter()
                        .filter(|v| !b.IsGridLineNames(v))
                        .map(|v| Self::ConvertGridTrackSize(b, s, v))
                        .collect()
                };
                result.GetMutableTrackList().AddRepeater(
                    &sizes,
                    GridTrackRepeatType::kInteger,
                    repetitions,
                    if subgrid { name_count } else { 1 },
                );
                continue;
            }
            ConvertGridLineNameOrTrackSize(b, value, &mut result, &mut line, false, false);
            if !b.IsGridLineNames(value) {
                result
                    .GetMutableTrackList()
                    .AddRepeaterDefault(&vec![Self::ConvertGridTrackSize(b, s, value)]);
            } else if subgrid {
                result.GetMutableTrackList().AddRepeaterDefault(&vec![]);
            }
        }
        Some(result)
    }
    pub fn ConvertFlowTolerance<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> FlowTolerance {
        if let Some(id) = b.Identifier(v) {
            FlowTolerance::from_keyword(id)
        } else {
            FlowTolerance::from_length(&Self::ConvertLength(b, s, v))
        }
    }
    pub fn ConvertGridLanesDirection<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> GridLanesDirection {
        if let Some(id) = b.Identifier(v) {
            assert_eq!(id, CSSValueID::kNormal);
            return ConvertGridLanesDirectionIdentifiers(&[id]).expect("normal direction");
        }
        let list = b.List(v).expect("grid lanes list");
        let identifiers = list.iter().map(|v| b.Identifier(v).expect("grid lanes identifier")).collect::<Vec<_>>();
        ConvertGridLanesDirectionIdentifiers(&identifiers).expect("valid grid lanes direction")
    }
    pub fn ConvertScrollMarkerGroup<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Option<ScrollMarkerGroup> {
        if b.Identifier(v) == Some(CSSValueID::kNone) {
            return None;
        }
        if b.Identifier(v).is_some() {
            return Some(ScrollMarkerGroup::new_links(b.ScrollMarkerPosition(v)));
        }
        let (first, second) = b.Pair(v).expect("marker pair");
        Some(ScrollMarkerGroup::new(
            b.ScrollMarkerPosition(first),
            b.ScrollMarkerMode(second),
        ))
    }

    pub fn ConvertHyphenateLimitChars<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleHyphenateLimitChars {
        if b.Identifier(v).is_some() {
            return StyleHyphenateLimitChars::default();
        }
        let list = b.List(v).expect("hyphen limits");
        let mut values = [0; 3];
        for (i, value) in list.into_iter().enumerate() {
            values[i] = if value.IsPrimitiveValue() {
                Self::ConvertInteger(b, s, value) as u32
            } else if b.Identifier(value).is_some() {
                0
            } else {
                unreachable!("hyphen limit")
            };
        }
        StyleHyphenateLimitChars::new(values[0], values[1], values[2])
    }
    pub fn ConvertInterestDelayValue<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleInterestDelay {
        if b.Identifier(v).is_some() {
            StyleInterestDelay::default()
        } else {
            StyleInterestDelay::new(Self::ConvertTimeValue(b, s, v))
        }
    }
    pub fn ConvertTimeValue<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> f64 {
        b.TimeSeconds(&s.CssToLengthConversionData(), v)
    }
    pub fn ClampLineWidth(width: f64) -> i32 {
        if width > 0.0 && width < 1.0 {
            return 1;
        }
        width.floor().clamp(0.0, LayoutUnit::Max().ToInt() as f64) as i32
    }
    pub fn ConvertBorderWidth<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> i32 {
        let pixels = if let Some(id) = b.Identifier(v) {
            let n = match id {
                CSSValueID::kThin => 1.0,
                CSSValueID::kMedium => 3.0,
                CSSValueID::kThick => 5.0,
                _ => unreachable!("border width"),
            };
            b.ZoomedPixels(&s.CssToLengthConversionData(), n)
        } else {
            b.ComputeLength(&s.CssToLengthConversionData(), v) as f32 as f64
        };
        Self::ClampLineWidth(pixels)
    }
    pub fn ConvertOutlineOffset<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> i32 {
        let n = b.ComputeLength(&s.CssToLengthConversionData(), v);
        let absolute = n.abs();
        if absolute > 0.0 && absolute < 1.0 {
            return if n > 0.0 { 1 } else { -1 };
        }
        let integral = absolute
            .floor()
            .clamp(0.0, LayoutUnit::Max().ToInt() as f64) as i32;
        if n < 0.0 {
            -integral
        } else {
            integral
        }
    }
    pub fn ConvertCornerShape<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Superellipse {
        if let Some(id) = b.Identifier(v) {
            return match id {
                CSSValueID::kBevel => Superellipse::Bevel(),
                CSSValueID::kNotch => Superellipse::Notch(),
                CSSValueID::kRound => Superellipse::Round(),
                CSSValueID::kSquircle => Superellipse::Squircle(),
                CSSValueID::kScoop => Superellipse::Scoop(),
                CSSValueID::kSquare => Superellipse::Square(),
                _ => unreachable!("corner keyword"),
            };
        }
        let param = b.SuperellipseParam(v);
        let n = if let Some(n) = b.NumericLiteralDouble(param) {
            n
        } else if param.IsMathFunctionValue() {
            b.MathKnownValue(param)
                .unwrap_or_else(|| b.Number(&s.CssToLengthConversionData(), param))
        } else {
            b.Number(&s.CssToLengthConversionData(), param)
        };
        Superellipse::new(n)
    }
    pub fn ConvertLayoutUnit<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> LayoutUnit {
        LayoutUnit::Clamp(Self::ConvertComputedLength::<B, f32>(b, s, v) as f64)
    }
    pub fn ConvertGapLength<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<Length> {
        if b.Identifier(v) == Some(CSSValueID::kNormal) {
            None
        } else {
            Some(Self::ConvertLength(b, s, v))
        }
    }
    pub fn ConvertGapDecorationInsetLength<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        if b.Identifier(v) == Some(CSSValueID::kOverlapJoin) {
            Length::from_type(foundation::LengthType::kOverlapJoin)
        } else {
            Self::ConvertLength(b, s, v)
        }
    }
    pub fn ConvertUnzoomedLength<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> layoutng_style::style::unzoomed_length::UnzoomedLength {
        layoutng_style::style::unzoomed_length::UnzoomedLength::new(
            &b.PrimitiveLength(&s.UnzoomedLengthConversionData(), v),
        )
    }
    pub fn ConvertZoom<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> f32 {
        if b.Identifier(v) == Some(CSSValueID::kNormal) {
            return ComputedStyleInitialValues::InitialZoom();
        }
        if v.IsPrimitiveValue() {
            let data = s.CssToLengthConversionData();
            if b.IsPercentage(v) {
                let n = b.Percentage(&data, v) as f32;
                return if n != 0.0 { n / 100.0 } else { 1.0 };
            }
            if b.IsNumber(v) {
                let n = b.Number(&data, v) as f32;
                return if n != 0.0 { n } else { 1.0 };
            }
        }
        unreachable!("zoom")
    }
    pub fn ConvertLengthOrNone<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<Length> {
        if b.Identifier(v).is_some() {
            None
        } else {
            Some(Self::ConvertLength(b, s, v))
        }
    }
    pub fn ConvertLengthSizing<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        let Some(id) = b.Identifier(v) else {
            return Self::ConvertLength(b, s, v);
        };
        match id {
            CSSValueID::kMinContent | CSSValueID::kWebkitMinContent => Length::MinContent().clone(),
            CSSValueID::kMaxContent | CSSValueID::kWebkitMaxContent => Length::MaxContent().clone(),
            CSSValueID::kStretch | CSSValueID::kWebkitFillAvailable => Length::Stretch().clone(),
            CSSValueID::kFitContent | CSSValueID::kWebkitFitContent => Length::FitContent().clone(),
            CSSValueID::kContent => Length::Content().clone(),
            CSSValueID::kAuto => Length::Auto().clone(),
            _ => unreachable!("sizing length"),
        }
    }
    pub fn ConvertLengthMaxSizing<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        if b.Identifier(v) == Some(CSSValueID::kNone) {
            Length::None().clone()
        } else {
            Self::ConvertLengthSizing(b, s, v)
        }
    }
    pub fn ConvertLengthOrTabSpaces<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> B::TabSize {
        let data = s.CssToLengthConversionData();
        let spaces = b.IsNumber(v);
        b.NewTabSize(
            if spaces {
                b.Number(&data, v) as f32
            } else {
                b.ComputeLength(&data, v) as f32
            },
            spaces,
        )
    }
    pub fn ConvertLineHeight<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        if v.IsPrimitiveValue() {
            let data = AdjustedZoomConversionData(b, s);
            if b.IsLength(v) {
                return Length::Fixed(b.ComputeLength(&data, v) as f32);
            }
            if b.IsNumber(v) {
                return Length::Percent(
                    (b.Number(&data, v) * 100.0).clamp(-(f32::MAX as f64), f32::MAX as f64) as f32,
                );
            }
            let size = b.ComputedFontSize(s);
            if b.IsPercentage(v) {
                return Length::Fixed(size * (b.Percentage(&data, v) as i32) as f32 / 100.0);
            }
            if b.IsCalculated(v) {
                return Length::from_layout_unit(
                    b.ValueForLength(&b.CalcLength(&data, v), LayoutUnit::from_f32(size)),
                    foundation::LengthType::kFixed,
                );
            }
        }
        ComputedStyleInitialValues::InitialLineHeight()
    }
    pub fn ConvertNumberOrPercentage<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> f32 {
        b.PrimitiveNumberOrPercentage(&s.CssToLengthConversionData(), v)
    }
    pub fn ConvertPathLength<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        if b.Identifier(v).is_some() {
            Length::None().clone()
        } else {
            Self::ConvertLength(b, s, v)
        }
    }
    pub fn ConvertAlpha<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> f32 {
        Self::ConvertNumberOrPercentage(b, s, v).clamp(0.0, 1.0)
    }
    pub fn ConvertCustomIdent<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Member<ScopedCSSName> {
        s.SetHasTreeScopedReference();
        b.AllocateScopedName(ScopedCSSName::new(
            &Self::ConvertCustomIdentUnscoped(b, s, v),
            b.PopulatedTreeScope(v),
        ))
    }
    pub fn ConvertNoneOrCustomIdent<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<Member<ScopedCSSName>> {
        if b.Identifier(v).is_some() {
            None
        } else {
            Some(Self::ConvertCustomIdent(b, s, v))
        }
    }
    pub fn ConvertNormalOrCustomIdent<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<Member<ScopedCSSName>> {
        if b.Identifier(v).is_some() {
            None
        } else {
            Some(Self::ConvertCustomIdent(b, s, v))
        }
    }
    pub fn ConvertPositionAnchor<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StylePositionAnchor {
        if let Some(id) = b.Identifier(v) {
            return StylePositionAnchor::new_type(match id {
                CSSValueID::kAuto => AnchorType::kAuto,
                CSSValueID::kNone => AnchorType::kNone,
                CSSValueID::kNormal => AnchorType::kNormal,
                _ => unreachable!("position anchor"),
            });
        }
        StylePositionAnchor::new_name(Self::ConvertCustomIdent(b, s, v))
    }
    pub fn ConvertPositionVisibility<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> PositionVisibility {
        let mut flags = PositionVisibility::kAlways;
        for value in b.List(v).unwrap_or_else(|| vec![v]) {
            flags |= b.PositionVisibility(value);
        }
        flags
    }

    pub fn ConvertAnchorName<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<*const ScopedCSSNameList> {
        if b.Identifier(v).is_some() {
            return None;
        }
        Some(
            b.AllocateScopedNameList(
                b.List(v)
                    .expect("anchor list")
                    .into_iter()
                    .map(|v| Some(Self::ConvertCustomIdent(b, s, v)))
                    .collect(),
            ),
        )
    }
    pub fn ConvertNameScope<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleNameScope {
        assert!(v.IsScopedValue());
        if b.IsScopedKeyword(v) {
            assert_eq!(b.Identifier(v), Some(CSSValueID::kAll));
            s.SetHasTreeScopedReference();
            return StyleNameScope::new(
                StyleNameScopeType::kAll,
                b.PopulatedTreeScope(v),
                std::ptr::null(),
            );
        }
        if let Some(id) = b.Identifier(v) {
            assert_eq!(id, CSSValueID::kNone);
            return StyleNameScope::default();
        }
        assert!(v.IsBaseValueList());
        let names = b
            .List(v)
            .expect("scope list")
            .into_iter()
            .map(|v| Some(Self::ConvertCustomIdent(b, s, v)))
            .collect();
        StyleNameScope::new(
            StyleNameScopeType::kNames,
            std::ptr::null(),
            b.AllocateScopedNameList(names),
        )
    }
    pub fn ConvertAnchorScope<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleNameScope {
        Self::ConvertNameScope(b, s, v)
    }
    pub fn ConvertInitialLetter<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleInitialLetter {
        if b.Identifier(v).is_some() {
            return StyleInitialLetter::Normal();
        }
        let list = b.List(v).expect("initial letter list");
        let size = b.Number(&s.CssToLengthConversionData(), list[0]) as f32;
        if list.len() == 1 {
            return StyleInitialLetter::with_size(size);
        }
        if let Some(id) = b.Identifier(list[1]) {
            return match id {
                CSSValueID::kDrop => StyleInitialLetter::Drop(size),
                CSSValueID::kRaise => StyleInitialLetter::Raise(size),
                _ => unreachable!("sink type"),
            };
        }
        if list[1].IsPrimitiveValue() {
            return StyleInitialLetter::with_sink(
                size,
                b.Number(&s.CssToLengthConversionData(), list[1]) as i32,
            );
        }
        StyleInitialLetter::Normal()
    }
    pub fn ConvertOffsetRotate<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleOffsetRotation {
        Self::ConvertOffsetRotateWithResolver(b, &s.CssToLengthConversionData(), v)
    }
    pub fn ConvertOffsetRotateWithResolver<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
    ) -> StyleOffsetRotation {
        let mut result = StyleOffsetRotation::new(0.0, OffsetRotationType::kFixed);
        if b.Identifier(v).is_some() {
            result.r#type = OffsetRotationType::kAuto;
            return result;
        }
        for value in b.List(v).expect("offset rotation list") {
            if b.Identifier(value) == Some(CSSValueID::kAuto) {
                result.r#type = OffsetRotationType::kAuto;
            } else if b.Identifier(value) == Some(CSSValueID::kReverse) {
                result.r#type = OffsetRotationType::kAuto;
                result.angle =
                    (result.angle as f64 + 180.0).clamp(-(f32::MAX as f64), f32::MAX as f64) as f32;
            } else {
                result.angle = (result.angle as f64 + b.Degrees(data, value))
                    .clamp(-(f32::MAX as f64), f32::MAX as f64)
                    as f32;
            }
        }
        result
    }
    pub fn ConvertPosition<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> LengthPoint {
        let (first, second) = b.Pair(v).expect("position pair");
        LengthPoint::new(
            &Self::ConvertPositionLength(b, s, first, CSSValueID::kLeft, CSSValueID::kRight),
            &Self::ConvertPositionLength(b, s, second, CSSValueID::kTop, CSSValueID::kBottom),
        )
    }
    pub fn ConvertPositionOrAuto<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> LengthPoint {
        if b.Pair(v).is_some() {
            Self::ConvertPosition(b, s, v)
        } else {
            LengthPoint::new(Length::Auto(), Length::Auto())
        }
    }
    pub fn ConvertOffsetPosition<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> LengthPoint {
        if b.Pair(v).is_some() {
            Self::ConvertPosition(b, s, v)
        } else if b.Identifier(v) == Some(CSSValueID::kAuto) {
            LengthPoint::new(Length::Auto(), Length::Auto())
        } else {
            LengthPoint::new(&Length::None(), &Length::None())
        }
    }
    pub fn ConvertPerspective<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> f32 {
        if b.Identifier(v) == Some(CSSValueID::kNone) {
            ComputedStyleInitialValues::InitialPerspective()
        } else {
            (b.ComputeLength(&s.CssToLengthConversionData(), v) as f32).max(0.0)
        }
    }
    pub fn ConvertPaintOrder<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> EPaintOrder {
        let Some(list) = b.List(v) else {
            return EPaintOrder::kPaintOrderNormal;
        };
        match b.Identifier(list[0]) {
            Some(CSSValueID::kFill) => {
                if list.len() > 1 {
                    EPaintOrder::kPaintOrderFillMarkersStroke
                } else {
                    EPaintOrder::kPaintOrderFillStrokeMarkers
                }
            }
            Some(CSSValueID::kStroke) => {
                if list.len() > 1 {
                    EPaintOrder::kPaintOrderStrokeMarkersFill
                } else {
                    EPaintOrder::kPaintOrderStrokeFillMarkers
                }
            }
            Some(CSSValueID::kMarkers) => {
                if list.len() > 1 {
                    EPaintOrder::kPaintOrderMarkersStrokeFill
                } else {
                    EPaintOrder::kPaintOrderMarkersFillStroke
                }
            }
            _ => unreachable!("paint order"),
        }
    }
    pub fn ConvertQuirkyLength<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        let mut result = Self::ConvertLengthOrAuto(b, s, v);
        result.SetQuirk(b.IsQuirkyEms(v));
        result
    }
    pub fn ConvertQuotes<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Option<foundation::ScopedRefPtr<foundation::QuotesData>> {
        if let Some(list) = b.List(v) {
            let mut result = foundation::QuotesData::Create();
            for pair in list.chunks(2) {
                b.AddQuotePair(
                    &mut result,
                    b.StringContents(pair[0]),
                    b.StringContents(pair[1]),
                );
            }
            return Some(result);
        }
        if b.Identifier(v) == Some(CSSValueID::kNone) {
            Some(foundation::QuotesData::Create())
        } else {
            None
        }
    }
    pub fn ConvertRadius<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> LengthSize {
        let (first, second) = b.Pair(v).expect("radius pair");
        LengthSize::new(
            &Self::ConvertLength(b, s, first),
            &Self::ConvertLength(b, s, second),
        )
    }
    pub fn ConvertGapDecorationColorDataList<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        visited: bool,
    ) -> GapDataList<StyleColor> {
        if let Some(list) = b.List(v) {
            let single = list.len() == 1 && b.GapRepeat(list[0]).is_none();
            if !single && s.InsideLink() != EInsideLink::kNotInsideLink {
                return GapDataList::DefaultGapColorDataList();
            }
        }
        ConvertGapDecorationDataList(b, s, v, |v| Self::ConvertStyleColor(b, s, v, visited))
    }
    pub fn ConvertGapDecorationWidthDataList<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> GapDataList<i32> {
        ConvertGapDecorationDataList(b, s, v, |v| {
            Self::ConvertBorderWidth(b, s, v).clamp(0, u16::MAX as i32)
        })
    }
    pub fn ConvertGapDecorationStyleDataList<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> GapDataList<EBorderStyle> {
        ConvertGapDecorationDataList(b, s, v, |v| b.BorderStyle(v))
    }
    pub fn ConvertShadow<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        data: &B::LengthConversionData,
        s: Option<&StyleResolverState<B>>,
        v: &ResolverValue<B>,
    ) -> ShadowData {
        let (x, y, blur, spread, style, color) = b.ShadowParts(v);
        let offset = foundation::gfx::Vector2dF::new(
            b.ComputeLength(data, x) as f32,
            b.ComputeLength(data, y) as f32,
        );
        let blur = blur.map_or(0.0, |v| b.ComputeLength(data, v) as f32);
        let spread = spread.map_or(0.0, |v| b.ComputeLength(data, v) as f32);
        let style = if style == Some(CSSValueID::kInset) {
            ShadowStyle::kInset
        } else {
            ShadowStyle::kNormal
        };
        let mut result = StyleColor::CurrentColor();
        if let Some(color) = color {
            if let Some(s) = s {
                result = Self::ConvertStyleColor(b, s, color, false);
            } else {
                let colors = b.BlackTextLinkColors();
                let context = ResolveColorValueContext {
                    length_resolver: data,
                    text_link_colors: &colors,
                    used_color_scheme: ColorScheme::kLight,
                    color_provider: None,
                    can_expose_accent_color: false,
                    for_visited_link: false,
                };
                result = ResolveColorValue(b, color, &context);
                if !result.IsAbsoluteColor() {
                    result = StyleColor::from_color(Color::kBlack);
                }
            }
        }
        ShadowData::new(offset, blur, spread, style, result, 1.0)
    }
    pub fn ConvertShadowList<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<ShadowList> {
        if b.Identifier(v).is_some() {
            return None;
        }
        let shadows = b
            .List(v)
            .expect("shadows list")
            .into_iter()
            .map(|v| Self::ConvertShadow(b, &s.CssToLengthConversionData(), Some(s), v))
            .collect();
        Some(b.AllocateShadowList(shadows))
    }
    pub fn ConvertShapeValue<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<ShapeValue> {
        if b.Identifier(v).is_some() {
            return None;
        }
        if v.IsImageValue() || v.IsImageGeneratorValue() || v.IsImageSetValue() {
            return Some(ShapeValue::from_image(b.StyleImageForShape(s, v)));
        }
        let (mut shape, mut shape_box) = (None, None);
        for value in b.List(v).expect("shape list") {
            if b.IsBasicShape(value) {
                shape = Some(b.BasicShapeForValue(s, value));
            } else {
                shape_box = Some(b.ShapeBox(value));
            }
        }
        if let Some(shape) = shape {
            return Some(unsafe {
                ShapeValue::from_shape(shape, shape_box.unwrap_or(ShapeBox::kMarginBox))
            });
        }
        Some(ShapeValue::from_box(shape_box.expect("shape box CHECK")))
    }
    pub fn ConvertSpacing<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        if b.Identifier(v) == Some(CSSValueID::kNormal) {
            Length::Fixed(0)
        } else {
            Self::ConvertLength(b, s, v)
        }
    }
    pub fn ConvertStrokeDasharray<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<SVGDashArray> {
        b.List(v).map(|values| {
            values
                .into_iter()
                .map(|v| Self::ConvertLength(b, s, v))
                .collect()
        })
    }

    pub fn ConvertViewTransitionGroup<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleViewTransitionGroup {
        if let Some(id) = b.Identifier(v) {
            return match id {
                CSSValueID::kNearest => StyleViewTransitionGroup::Nearest(),
                CSSValueID::kNormal => StyleViewTransitionGroup::Normal(),
                CSSValueID::kContain => StyleViewTransitionGroup::Contain(),
                _ => unreachable!("transition group"),
            };
        }
        let name = Self::ConvertCustomIdent(b, s, v);
        StyleViewTransitionGroup::Create(unsafe { (&*name.Get()).GetName() })
    }

    pub fn ConvertViewTransitionName<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<StyleViewTransitionName> {
        s.SetHasTreeScopedReference();
        if let Some(id) = b.Identifier(v) {
            return match id {
                CSSValueID::kNone => None,
                CSSValueID::kAuto => Some(StyleViewTransitionName::new_with_type(
                    StyleViewTransitionNameType::kAuto,
                    b.DocumentTreeScope(s),
                )),
                CSSValueID::kMatchElement => Some(StyleViewTransitionName::new_with_type(
                    StyleViewTransitionNameType::kMatchElement,
                    b.DocumentTreeScope(s),
                )),
                _ => unreachable!("transition name"),
            };
        }
        let name = Self::ConvertCustomIdent(b, s, v);
        let name = unsafe { &*name.Get() };
        Some(StyleViewTransitionName::new_with_custom_name(
            name.GetName(),
            name.GetTreeScope(),
        ))
    }

    pub fn ConvertViewTransitionClass<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<*const ScopedCSSNameList> {
        if b.Identifier(v).is_some() {
            return None;
        }
        Some(
            b.AllocateScopedNameList(
                b.List(v)
                    .expect("transition class list")
                    .into_iter()
                    .map(|v| Self::ConvertNoneOrCustomIdent(b, s, v))
                    .collect(),
            ),
        )
    }
    pub fn ConvertOverscrollArea<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<*const ScopedCSSNameList> {
        if b.Identifier(v).is_some() {
            return None;
        }
        Some(
            b.AllocateScopedNameList(
                b.List(v)
                    .expect("overscroll list")
                    .into_iter()
                    .map(|v| Some(Self::ConvertCustomIdent(b, s, v)))
                    .collect(),
            ),
        )
    }
    pub fn ConvertOverscrollPosition<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<Member<ScopedCSSName>> {
        if b.Identifier(v).is_some() {
            None
        } else {
            Some(Self::ConvertCustomIdent(b, s, v))
        }
    }
    pub fn ConvertStyleColor<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        visited: bool,
    ) -> StyleColor {
        let scheme = b.UsedColorScheme(s);
        let colors = b.DocumentTextLinkColors(s);
        let provider = b.ColorProviderForPainting(s, scheme);
        let data = s.CssToLengthConversionData();
        ResolveColorValue(
            b,
            v,
            &ResolveColorValueContext {
                length_resolver: &data,
                text_link_colors: &colors,
                used_color_scheme: scheme,
                color_provider: provider.as_ref(),
                can_expose_accent_color: b.IsInWebAppScope(s) && b.IsInitialProfile(s),
                for_visited_link: visited,
            },
        )
    }
    pub fn ConvertStyleAutoColor<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        visited: bool,
    ) -> StyleAutoColor {
        if b.Identifier(v) == Some(CSSValueID::kAuto) {
            StyleAutoColor::AutoColor()
        } else {
            StyleAutoColor::new(Self::ConvertStyleColor(b, s, v, visited))
        }
    }
    pub fn ConvertStyleCaretColor<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        visited: bool,
    ) -> StyleCaretColor {
        if let Some(list) = b.List(v) {
            StyleCaretColor::new(
                Self::ConvertStyleAutoColor(b, s, list[0], visited),
                Self::ConvertStyleAutoColor(b, s, list[1], visited),
            )
        } else {
            StyleCaretColor::new(
                Self::ConvertStyleAutoColor(b, s, v, visited),
                StyleAutoColor::AutoColor(),
            )
        }
    }
    pub fn ConvertSVGPaint<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        _visited: bool,
        property: CSSPropertyID,
    ) -> SVGPaint {
        let mut local = v;
        let mut resource = None;
        if let Some(list) = b.List(v) {
            resource = Self::ConvertElementReference(b, s, list[0], property);
            local = list[1];
        }
        if b.IsURI(local) {
            return b.NewSVGPaint(
                SVGPaintType::kUri,
                Self::ConvertElementReference(b, s, local, property),
                None,
            );
        }
        if let Some(id) = b.Identifier(local) {
            match id {
                CSSValueID::kNone => {
                    return b.NewSVGPaint(
                        if resource.is_none() {
                            SVGPaintType::kNone
                        } else {
                            SVGPaintType::kUriNone
                        },
                        resource,
                        None,
                    )
                }
                CSSValueID::kContextFill => {
                    return b.NewSVGPaint(SVGPaintType::kContextFill, resource, None)
                }
                CSSValueID::kContextStroke => {
                    return b.NewSVGPaint(SVGPaintType::kContextStroke, resource, None)
                }
                _ => {}
            }
        }
        let color = Self::ConvertStyleColor(b, s, local, false);
        b.NewSVGPaint(
            if resource.is_none() {
                SVGPaintType::kColor
            } else {
                SVGPaintType::kUriColor
            },
            resource,
            Some(color),
        )
    }
    pub fn ConvertTextBoxEdge<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> TextBoxEdge {
        if b.Identifier(v).is_some() {
            TextBoxEdge::from_over(b.TextBoxEdgeType(v))
        } else {
            let list = b.List(v).expect("text box edges");
            TextBoxEdge::new(b.TextBoxEdgeType(list[0]), b.TextBoxEdgeType(list[1]))
        }
    }

    pub fn ConvertTextDecorationThickness<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> TextDecorationThickness {
        if b.Identifier(v) == Some(CSSValueID::kFromFont) {
            TextDecorationThickness::from_keyword(CSSValueID::kFromFont)
        } else {
            TextDecorationThickness::new(&Self::ConvertLengthOrAuto(b, s, v))
        }
    }
    pub fn ConvertTextDecorationInset<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> TextDecorationInset {
        if b.Identifier(v).is_some() {
            return TextDecorationInset::new(Length::Auto(), Length::Auto());
        }
        let (first, second) = b.Pair(v).expect("decoration inset");
        TextDecorationInset::new(
            &Self::ConvertLength(b, s, first),
            &Self::ConvertLength(b, s, second),
        )
    }
    pub fn ConvertTextTextEmphasisPosition<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> TextEmphasisPosition {
        let list = b.List(v).expect("emphasis list");
        let first = b.Identifier(list[0]).unwrap();
        if list.len() < 2 {
            if b.TextEmphasisAutoEnabled() && first == CSSValueID::kAuto {
                return TextEmphasisPosition::kAuto;
            }
            return if first == CSSValueID::kUnder {
                TextEmphasisPosition::kUnderRight
            } else {
                TextEmphasisPosition::kOverRight
            };
        }
        match (first, b.Identifier(list[1]).unwrap()) {
            (CSSValueID::kOver, CSSValueID::kRight) => TextEmphasisPosition::kOverRight,
            (CSSValueID::kOver, CSSValueID::kLeft) => TextEmphasisPosition::kOverLeft,
            (CSSValueID::kUnder, CSSValueID::kRight) => TextEmphasisPosition::kUnderRight,
            (CSSValueID::kUnder, CSSValueID::kLeft) => TextEmphasisPosition::kUnderLeft,
            _ => TextEmphasisPosition::kOverRight,
        }
    }
    pub fn ConvertTextStrokeWidth<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> f32 {
        if let Some(id) = b.Identifier(v) {
            if foundation::IsValidCSSValueID(id) {
                let multiplier = Self::ConvertLineWidth::<B, f32>(b, s, v);
                return b.ComputeEms(&s.CssToLengthConversionData(), multiplier / 48.0);
            }
        }
        b.ComputeLength(&s.CssToLengthConversionData(), v) as f32
    }
    pub fn ConvertTextSizeAdjust<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> TextSizeAdjust {
        match b.Identifier(v) {
            Some(CSSValueID::kNone) => TextSizeAdjust::AdjustNone(),
            Some(CSSValueID::kAuto) => TextSizeAdjust::AdjustAuto(),
            _ => {
                TextSizeAdjust::new(b.Percentage(&s.CssToLengthConversionData(), v) as f32 / 100.0)
            }
        }
    }
    pub fn ConvertTextUnderlinePosition<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> TextUnderlinePosition {
        let mut flags = TextUnderlinePosition::kAuto;
        for value in b.List(v).unwrap_or_else(|| vec![v]) {
            flags |= b.TextUnderlinePositionFlag(value);
        }
        flags
    }

    pub fn ConvertTextUnderlineOffset<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Length {
        Self::ConvertLengthOrAuto(b, s, v)
    }
    pub fn ConvertTransformOperations<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> foundation::TransformOperations {
        b.CreateTransformOperations(&s.CssToLengthConversionData(), v)
    }
    pub fn ConvertTransformOrigin<B: StyleBuilderConverterBatch2Backend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> TransformOrigin {
        let list = b.List(v).expect("transform origin list");
        let z = if list.len() == 3 {
            Self::ConvertComputedLength::<B, f32>(b, s, list[2])
        } else {
            0.0
        };
        TransformOrigin::new(
            &Self::ConvertPositionLength(b, s, list[0], CSSValueID::kLeft, CSSValueID::kRight),
            &Self::ConvertPositionLength(b, s, list[1], CSSValueID::kTop, CSSValueID::kBottom),
            z,
        )
    }
}

fn ConvertGridLineNames<B: StyleBuilderConverterBatch2Backend>(
    b: &B,
    v: &ResolverValue<B>,
    index: u32,
    names: &mut NamedGridLinesMap,
    _in_repeat: bool,
    _first: bool,
) {
    for entry in b.List(v).expect("line names list") {
        let name = b.CustomIdent(entry).expect("line name");
        let name = foundation::String::from_utf16(name.utf16_units().expect("non-null line name"));
        names.entry(name).or_default().push(index);
    }
}
fn ConvertOrderedGridLineNames<B: StyleBuilderConverterBatch2Backend>(
    b: &B,
    v: &ResolverValue<B>,
    index: u32,
    names: &mut OrderedNamedGridLines,
    in_repeat: bool,
    first: bool,
) {
    for entry in b.List(v).expect("line names list") {
        let name = b.CustomIdent(entry).expect("line name");
        names
            .entry(index as usize)
            .or_default()
            .push(NamedGridLine::new(&name, in_repeat, first));
    }
}
fn ConvertGridLineNameOrTrackSize<B: StyleBuilderConverterBatch2Backend>(
    b: &B,
    v: &ResolverValue<B>,
    result: &mut ComputedGridTrackList,
    line: &mut u32,
    in_repeat: bool,
    first: bool,
) -> u32 {
    if b.IsGridLineNames(v) {
        ConvertGridLineNames(
            b,
            v,
            *line,
            result.GetMutableNamedGridLines(),
            in_repeat,
            first,
        );
        ConvertOrderedGridLineNames(
            b,
            v,
            *line,
            result.GetMutableOrderedNamedGridLines(),
            in_repeat,
            first,
        );
        if result.IsSubgriddedAxis() {
            *line += 1;
            result
                .GetMutableTrackList()
                .IncrementNonAutoRepeatLineCount();
        }
        1
    } else {
        *line += 1;
        0
    }
}
fn AdjustedZoomConversionData<B: StyleBuilderConverterBatch2Backend>(
    b: &B,
    s: &StyleResolverState<B>,
) -> B::LengthConversionData {
    let mut multiplier = b.EffectiveZoom(s);
    if let Some(zoom) = b.FrameTextZoom(s) {
        multiplier *= zoom;
    }
    let adjust = b.TextSizeAdjust(s);
    if !adjust.IsAuto() && b.TextSizeAdjustEnabled(s) {
        multiplier *= adjust.Multiplier();
    }
    b.CopyWithAdjustedZoom(&s.CssToLengthConversionData(), multiplier)
}
fn ConvertGapDecorationDataList<B: StyleBuilderConverterBatch2Backend, T: GapValue>(
    b: &B,
    s: &StyleResolverState<B>,
    v: &ResolverValue<B>,
    convert: impl Fn(&ResolverValue<B>) -> T,
) -> GapDataList<T> {
    let Some(list) = b.List(v) else {
        return GapDataList::from_value(&convert(v));
    };
    let mut output = foundation::HeapVector::default();
    for value in list {
        let data = if let Some((repeat_values, repetitions)) = b.GapRepeat(value) {
            let values = repeat_values.into_iter().map(&convert).collect();
            let count =
                repetitions.map(|v| b.ComputeInteger(&s.CssToLengthConversionData(), v) as u32);
            GapData::from_repeater(b.AllocateGapRepeater(values, count))
        } else {
            GapData::from_value(convert(value))
        };
        output.emplace_back(data);
    }
    GapDataList::from_vector(output)
}

/// Source ResolveColorValueContext, with real value/DOM owner types and no
/// implicit default environment. Offscreen callers supply explicit black links.
pub struct ResolveColorValueContext<'a, B: StyleBuilderConverterBatch2Backend> {
    pub length_resolver: &'a B::LengthConversionData,
    pub text_link_colors: &'a B::TextLinkColors,
    pub used_color_scheme: ColorScheme,
    pub color_provider: Option<&'a B::ColorProvider>,
    pub can_expose_accent_color: bool,
    pub for_visited_link: bool,
}
fn ResolveColorValueImpl<B: StyleBuilderConverterBatch2Backend>(
    b: &B,
    v: &ResolverValue<B>,
    context: &ResolveColorValueContext<'_, B>,
) -> StyleColor {
    if let Some(mut color) = b.NumericColor(v) {
        b.ResolveNonFiniteColor(&mut color);
        return StyleColor::from_color(color);
    }
    if let Some(id) = b.Identifier(v) {
        if id == CSSValueID::kCurrentcolor {
            return StyleColor::CurrentColor();
        }
        if IsQuirkOrLinkOrFocusRingColor(id) {
            return StyleColor::from_color(ResolveQuirkOrLinkOrFocusRingColor(
                b,
                id,
                context.text_link_colors,
                context.used_color_scheme,
                context.for_visited_link,
            ));
        }
        let color = b.ColorFromKeyword(
            id,
            context.used_color_scheme,
            context.color_provider,
            context.can_expose_accent_color,
        );
        return if b.IsSystemColorKeyword(id) {
            StyleColor::from_color_and_keyword(color, id)
        } else {
            StyleColor::from_color(color)
        };
    }
    if let Some((first, second, space, hue)) = b.ColorMix(v) {
        let first = ResolveColorValueImpl(b, first, context);
        let second = ResolveColorValueImpl(b, second, context);
        let (amount, alpha) = b.ColorMixNormalizedPercentages(context.length_resolver, v);
        let absolute = first.IsAbsoluteColor() && second.IsAbsoluteColor();
        let unresolved = b.NewUnresolvedColorMix(space, hue, first, second, amount, alpha);
        return if absolute {
            StyleColor::from_color(b.ResolveUnresolvedColor(&unresolved, Color::default()))
        } else {
            b.StyleColorFromUnresolved(unresolved)
        };
    }
    if let Some(origin) = b.RelativeColorOrigin(v) {
        let origin = ResolveColorValueImpl(b, origin, context);
        let absolute = origin.IsAbsoluteColor();
        let unresolved = b.NewUnresolvedRelativeColor(context.length_resolver, v, origin);
        return if absolute {
            StyleColor::from_color(b.ResolveUnresolvedColor(&unresolved, Color::default()))
        } else {
            b.StyleColorFromUnresolved(unresolved)
        };
    }
    if let Some(param) = b.ContrastColor(v) {
        let param = ResolveColorValueImpl(b, param, context);
        let absolute = param.IsAbsoluteColor();
        let unresolved = b.NewUnresolvedContrastColor(param);
        return if absolute {
            StyleColor::from_color(b.ResolveUnresolvedColor(&unresolved, Color::default()))
        } else {
            b.StyleColorFromUnresolved(unresolved)
        };
    }
    if let Some(origin) = b.AlphaColorOrigin(v) {
        let origin = ResolveColorValueImpl(b, origin, context);
        let absolute = origin.IsAbsoluteColor();
        let unresolved = b.NewUnresolvedAlphaColor(context.length_resolver, v, origin);
        return if absolute {
            StyleColor::from_color(b.ResolveUnresolvedColor(&unresolved, Color::default()))
        } else {
            b.StyleColorFromUnresolved(unresolved)
        };
    }
    if let Some(color) = b.CSSUnresolvedColor(context.length_resolver, v) {
        return StyleColor::from_color(color);
    }
    let (first, second) = b.LightDarkPair(v).expect("parsed light-dark pair");
    ResolveColorValueImpl(
        b,
        if context.used_color_scheme == ColorScheme::kLight {
            first
        } else {
            second
        },
        context,
    )
}
pub fn ResolveColorValue<B: StyleBuilderConverterBatch2Backend>(
    b: &B,
    v: &ResolverValue<B>,
    context: &ResolveColorValueContext<'_, B>,
) -> StyleColor {
    if let Some((first, second)) = b.LightDarkPair(v) {
        return ResolveColorValue(
            b,
            if context.used_color_scheme == ColorScheme::kLight {
                first
            } else {
                second
            },
            context,
        );
    }
    let mut result = ResolveColorValueImpl(b, v, context);
    if (b.RelativeColorOrigin(v).is_some() || b.ColorMix(v).is_some()) && result.IsAbsoluteColor() {
        let mut color = result.GetColor();
        if b.IsLegacyColorSpace(color.GetColorSpace()) {
            let missing = color.GetColorSpace() != ColorSpace::kSRGBLegacy;
            b.ConvertColorSpace(&mut color, ColorSpace::kSRGB, missing);
            result = StyleColor::from_color(color);
        }
    }
    result
}

use layoutng_style::style::{
    scroll_marker_group::{ScrollMarkerGroup, ScrollMarkerMode, ScrollMarkerPosition},
    style_view_transition_group::StyleViewTransitionGroup,
    style_view_transition_name::{StyleViewTransitionName, StyleViewTransitionNameType},
    svg_dash_array::SVGDashArray,
    text_box_edge::{TextBoxEdge, TextBoxEdgeType},
};

use crate::{
    css_primitive_value::UnitType,
    parser::css_parser_context::{CSSParserContext, CSSParserContextPlatform},
};
use foundation::{
    transform_operations::{
        RotateTransformOperation, Rotation, ScaleTransformOperation, TranslateTransformOperation,
    },
    EAspectRatioType, RespectImageOrientationEnum, RubyPosition, StyleAspectRatio,
    TransformOperationType,
};
use layoutng_style::{
    css::color_scheme_flags::{ColorSchemeFlag, ColorSchemeFlags},
    style::{
        max_lines_data::MaxLinesData,
        offset_path_operation::OffsetPathOperation,
        position_area::{PositionArea, PositionAreaRegion},
        position_try_fallbacks::PositionTryFallback,
        scroll_snap_data::cc,
        style_intrinsic_length::{StyleIntrinsicLength, StyleIntrinsicLengthOptions},
        style_overflow_clip_margin::{ReferenceBox, StyleOverflowClipMargin},
        style_path::StylePath,
        style_scrollbar_color::StyleScrollbarColor,
        style_timeline_scope::{StyleTimelineScope, StyleTimelineScopeType},
        text_fit::{TextFit, TextFitTarget, TextFitType},
        text_overflow_data::{TextOverflowData, TextOverflowType},
        timeline_inset::TimelineInset,
    },
};
use std::rc::Rc;
/// Required interfaces to real value subclasses, parser/platform and allocation
/// owners used by the final batch. Recursion and conversion selection stay Rust.
pub trait StyleBuilderConverterFinalBackend: StyleBuilderConverterBatch2Backend {
    type ConverterParserPlatform: CSSParserContextPlatform;
    type RegisteredVariableData;
    fn SnapAxis(&self, v: &ResolverValue<Self>) -> cc::SnapAxis;
    fn SnapStrictness(&self, v: &ResolverValue<Self>) -> cc::SnapStrictness;
    fn SnapAlignment(&self, v: &ResolverValue<Self>) -> cc::SnapAlignment;
    fn Axis(&self, data: &Self::LengthConversionData, v: &ResolverValue<Self>) -> (f64, f64, f64);
    fn PathValue(&self, v: &ResolverValue<Self>) -> Option<*mut StylePath>;
    fn CoordBox(&self, v: &ResolverValue<Self>) -> CoordBox;
    fn NewReferenceOffsetPath(
        &self,
        url: foundation::String,
        resource: Self::SVGResource,
        coord: CoordBox,
    ) -> Rc<dyn OffsetPathOperation>;
    fn NewShapeOffsetPath(
        &self,
        shape: *mut dyn BasicShape,
        coord: CoordBox,
    ) -> Rc<dyn OffsetPathOperation>;
    fn NewCoordBoxOffsetPath(&self, coord: CoordBox) -> Rc<dyn OffsetPathOperation>;
    fn ShareValue(&self, v: &ResolverValue<Self>) -> Rc<ResolverValue<Self>>;
    fn NewFunctionValue(
        &self,
        id: CSSValueID,
        children: Vec<Rc<ResolverValue<Self>>>,
    ) -> Rc<ResolverValue<Self>>;
    fn NewListWithSeparatorFrom(
        &self,
        old: &ResolverValue<Self>,
        children: Vec<Rc<ResolverValue<Self>>>,
    ) -> Rc<ResolverValue<Self>>;
    fn IsPx(&self, v: &ResolverValue<Self>) -> bool;
    fn IsResolvableBeforeLayout(&self, v: &ResolverValue<Self>) -> bool;
    fn IsAngle(&self, v: &ResolverValue<Self>) -> bool;
    fn IsTime(&self, v: &ResolverValue<Self>) -> bool;
    fn IsResolution(&self, v: &ResolverValue<Self>) -> bool;
    fn DotsPerPixel(&self, data: &Self::LengthConversionData, v: &ResolverValue<Self>) -> f64;
    fn Unzoomed(&self, data: &Self::LengthConversionData) -> Self::LengthConversionData;
    fn NewPrimitiveFromLength(&self, length: Length, zoom: f32) -> Rc<ResolverValue<Self>>;
    fn NewNumericLiteral(&self, number: f64, unit: UnitType) -> Rc<ResolverValue<Self>>;
    fn IsColorKeyword(&self, id: CSSValueID) -> bool;
    fn DocumentLinkColors(&self, document: &Self::Document) -> Self::TextLinkColors;
    fn DocumentColorProvider(
        &self,
        document: &Self::Document,
        scheme: ColorScheme,
    ) -> Option<Self::ColorProvider>;
    fn DocumentInWebAppScope(&self, document: &Self::Document) -> bool;
    fn DocumentInitialProfile(&self, document: &Self::Document) -> bool;
    fn StyleColorToCSSValue(&self, color: StyleColor) -> Rc<ResolverValue<Self>>;
    fn ComputedURIValue(
        &self,
        v: &ResolverValue<Self>,
        url: &<Self::ConverterParserPlatform as CSSParserContextPlatform>::URL,
        charset: &<Self::ConverterParserPlatform as CSSParserContextPlatform>::TextEncoding,
    ) -> Rc<ResolverValue<Self>>;
    fn ResolveCustomIdentValue(
        &self,
        data: &Self::LengthConversionData,
        v: &ResolverValue<Self>,
    ) -> Rc<ResolverValue<Self>>;
    fn EmptyFontSizes(&self) -> Self::FontSizes;
    fn EmptyLineHeightSize(&self) -> Self::LineHeightSize;
    fn RegisteredViewportSize(&self, document: &Self::Document) -> Self::ViewportSize;
    fn EmptyContainerSizes(&self) -> Self::ContainerSizes;
    fn EmptyAnchorData(&self) -> Self::AnchorData;
    fn NewRegisteredLengthConversionData(
        &self,
        writing: foundation::WritingMode,
        fonts: Self::FontSizes,
        line: Self::LineHeightSize,
        viewport: Self::ViewportSize,
        containers: Self::ContainerSizes,
        anchors: Self::AnchorData,
        zoom: f32,
        flags: Rc<std::cell::Cell<u32>>,
        element: Option<&Self::Element>,
    ) -> Self::LengthConversionData;
    fn ElementSheetParserContext(
        &self,
        document: &Self::Document,
    ) -> Option<Rc<CSSParserContext<Self::ConverterParserPlatform>>>;
    fn NewRegisteredVariableData(
        &self,
        text: foundation::String,
        animation_taint: bool,
        attr_taint: bool,
        has_references: bool,
    ) -> Rc<Self::RegisteredVariableData>;
    fn Ratio<'a>(
        &self,
        v: &'a ResolverValue<Self>,
    ) -> Option<(&'a ResolverValue<Self>, &'a ResolverValue<Self>)>;
    fn RubyPosition(&self, v: &ResolverValue<Self>) -> RubyPosition;
    fn ScrollbarGutter(&self, v: &ResolverValue<Self>) -> ScrollbarGutter;
    fn TimelineAxis(&self, v: &ResolverValue<Self>) -> TimelineAxis;
    fn TryTactic(&self, v: &ResolverValue<Self>) -> TryTactic;
    fn PrimitiveUInt16(&self, data: &Self::LengthConversionData, v: &ResolverValue<Self>) -> u16;
}
impl StyleBuilderConverter {
    pub fn ConvertSnapType<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> cc::ScrollSnapType {
        let mut result = ComputedStyleInitialValues::InitialScrollSnapType();
        if let Some((first, second)) = b.Pair(v) {
            result.is_none = false;
            result.axis = b.SnapAxis(first);
            result.strictness = b.SnapStrictness(second);
            return result;
        }
        if b.Identifier(v) == Some(CSSValueID::kNone) {
            result.is_none = true;
            return result;
        }
        result.is_none = false;
        result.axis = b.SnapAxis(v);
        result
    }
    pub fn ConvertSnapAlign<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> cc::ScrollSnapAlign {
        let mut result = ComputedStyleInitialValues::InitialScrollSnapAlign();
        if let Some((first, second)) = b.Pair(v) {
            result.alignment_block = b.SnapAlignment(first);
            result.alignment_inline = b.SnapAlignment(second);
        } else {
            result.alignment_block = b.SnapAlignment(v);
            result.alignment_inline = result.alignment_block;
        }
        result
    }
    pub fn ConvertTranslate<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<TranslateTransformOperation> {
        if b.Identifier(v).is_some() {
            return None;
        }
        let list = b.List(v).expect("translate list");
        let x = Self::ConvertLength(b, s, list[0]);
        let y = if list.len() >= 2 {
            Self::ConvertLength(b, s, list[1])
        } else {
            Length::Fixed(0)
        };
        let z = if list.len() == 3 {
            b.ComputeLength(&s.CssToLengthConversionData(), list[2])
        } else {
            0.0
        };
        Some(TranslateTransformOperation::new(
            x,
            y,
            z,
            TransformOperationType::kTranslate3D,
        ))
    }
    pub fn ConvertRotation<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        data: &B::LengthConversionData,
        v: &ResolverValue<B>,
    ) -> Rotation {
        if b.Identifier(v).is_some() {
            return Rotation::new([0.0, 0.0, 1.0], 0.0);
        }
        let list = b.List(v).expect("rotation list");
        let (x, y, z) = if list.len() == 2 {
            b.Axis(data, list[0])
        } else {
            (0.0, 0.0, 1.0)
        };
        Rotation::new(
            [x as f32, y as f32, z as f32],
            b.Degrees(data, list[list.len() - 1]),
        )
    }
    pub fn ConvertRotate<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<RotateTransformOperation> {
        if b.Identifier(v).is_some() {
            return None;
        }
        let rotation = Self::ConvertRotation(b, &s.CssToLengthConversionData(), v);
        let axis = rotation.Axis();
        Some(RotateTransformOperation::new_3d(
            axis[0] as f64,
            axis[1] as f64,
            axis[2] as f64,
            rotation.Angle(),
            TransformOperationType::kRotate3D,
        ))
    }
    pub fn ConvertScale<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<ScaleTransformOperation> {
        if b.Identifier(v).is_some() {
            return None;
        }
        let list = b.List(v).expect("scale list");
        let data = s.CssToLengthConversionData();
        let x = b.Number(&data, list[0]);
        let y = if list.len() >= 2 {
            b.Number(&data, list[1])
        } else {
            x
        };
        let z = if list.len() == 3 {
            b.Number(&data, list[2])
        } else {
            1.0
        };
        Some(ScaleTransformOperation::new(
            x,
            y,
            z,
            TransformOperationType::kScale3D,
        ))
    }
    pub fn ConvertImageOrientation<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> RespectImageOrientationEnum {
        if b.Identifier(v) == Some(CSSValueID::kNone) {
            RespectImageOrientationEnum::kDoNotRespectImageOrientation
        } else {
            RespectImageOrientationEnum::kRespectImageOrientation
        }
    }
    pub fn ConvertPathOrNone<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Option<*mut StylePath> {
        b.PathValue(v)
    }
    pub fn ConvertOffsetPath<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<Rc<dyn OffsetPathOperation>> {
        if b.Identifier(v).is_some() {
            return None;
        }
        let list = b.List(v).expect("offset path list");
        if b.Identifier(list[0]).is_some() {
            return Some(b.NewCoordBoxOffsetPath(b.CoordBox(list[0])));
        }
        let coord = if list.len() == 2 {
            b.CoordBox(list[list.len() - 1])
        } else {
            CoordBox::kBorderBox
        };
        Some(ConvertOffsetPathValueToOperation(b, s, list[0], coord))
    }
    pub fn ConvertObjectViewBox<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<*mut dyn BasicShape> {
        if !v.IsBasicShapeInsetValue() && !v.IsBasicShapeRectValue() && !v.IsBasicShapeXYWHValue() {
            None
        } else {
            Some(b.BasicShapeForValue(s, v))
        }
    }
    pub fn ConvertRegisteredPropertyInitialValue<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        document: &B::Document,
        v: &ResolverValue<B>,
    ) -> Rc<ResolverValue<B>> {
        let data = b.NewRegisteredLengthConversionData(
            foundation::WritingMode::kHorizontalTb,
            b.EmptyFontSizes(),
            b.EmptyLineHeightSize(),
            b.RegisteredViewportSize(document),
            b.EmptyContainerSizes(),
            b.EmptyAnchorData(),
            1.0,
            Rc::new(std::cell::Cell::new(0)),
            None,
        );
        let context = b.ElementSheetParserContext(document);
        ComputeRegisteredPropertyValue(b, document, None, &data, v, context.as_deref())
    }
    pub fn ConvertRegisteredPropertyValue<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        context: Option<&CSSParserContext<B::ConverterParserPlatform>>,
    ) -> Rc<ResolverValue<B>> {
        ComputeRegisteredPropertyValue(
            b,
            s.GetDocument(),
            Some(s),
            &s.CssToLengthConversionData(),
            v,
            context,
        )
    }
    pub fn ConvertRegisteredPropertyVariableData<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
        animation_taint: bool,
        attr_taint: bool,
    ) -> Rc<B::RegisteredVariableData> {
        b.NewRegisteredVariableData(v.CssText(), animation_taint, attr_taint, false)
    }
    pub fn ConvertAspectRatio<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleAspectRatio {
        if b.Identifier(v) == Some(CSSValueID::kAuto) {
            return StyleAspectRatio::new(
                EAspectRatioType::kAuto,
                foundation::gfx::SizeF::default(),
            );
        }
        let list = b.List(v).expect("ratio list");
        let kind = if list.len() == 1 {
            EAspectRatioType::kRatio
        } else {
            EAspectRatioType::kAutoAndRatio
        };
        let ratio = b
            .Ratio(list[0])
            .or_else(|| b.Ratio(list[1]))
            .expect("ratio value");
        StyleAspectRatio::new(
            kind,
            foundation::gfx::SizeF::new(
                b.Number(&s.CssToLengthConversionData(), ratio.0) as f32,
                b.Number(&s.CssToLengthConversionData(), ratio.1) as f32,
            ),
        )
    }
    pub fn ConvertInternalAlignContentBlock<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> bool {
        b.Identifier(v) == Some(CSSValueID::kCenter)
    }
    pub fn ConvertInternalEmptyLineHeight<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> bool {
        b.Identifier(v) == Some(CSSValueID::kFabricated)
    }
    pub fn ConvertPage<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> AtomicString {
        b.CustomIdent(v).unwrap_or_default()
    }
    pub fn ConvertRubyPosition<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> RubyPosition {
        if let Some(id) = b.Identifier(v) {
            return if id == CSSValueID::kBefore {
                RubyPosition::kOver
            } else if id == CSSValueID::kAfter {
                RubyPosition::kUnder
            } else {
                b.RubyPosition(v)
            };
        }
        unreachable!("ruby position")
    }
    pub fn ConvertScrollbarColor<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<StyleScrollbarColor> {
        if b.Identifier(v) == Some(CSSValueID::kAuto) {
            return None;
        }
        let list = b.List(v).expect("scrollbar colors");
        Some(StyleScrollbarColor::new(
            Self::ConvertStyleColor(b, s, list[0], false),
            Self::ConvertStyleColor(b, s, list[list.len() - 1], false),
        ))
    }
    pub fn ConvertScrollbarGutter<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> ScrollbarGutter {
        let mut flags = ScrollbarGutter::kScrollbarGutterAuto;
        for value in b.List(v).unwrap_or_else(|| vec![v]) {
            flags |= b.ScrollbarGutter(value);
        }
        flags
    }
    pub fn ConvertContainerName<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<*const ScopedCSSNameList> {
        if b.Identifier(v).is_some() {
            return None;
        }
        Some(
            b.AllocateScopedNameList(
                b.List(v)
                    .expect("container names")
                    .into_iter()
                    .map(|v| Self::ConvertNoneOrCustomIdent(b, s, v))
                    .collect(),
            ),
        )
    }
    pub fn ConvertIntrinsicDimension<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleIntrinsicLength {
        if let Some(list) = b.List(v) {
            return StyleIntrinsicLength::new(
                &Self::ConvertLengthOrNone(b, s, list[1]),
                StyleIntrinsicLengthOptions { has_auto: true },
            );
        }
        StyleIntrinsicLength::new(
            &Self::ConvertLengthOrNone(b, s, v),
            StyleIntrinsicLengthOptions { has_auto: false },
        )
    }
    pub fn ExtractColorSchemes<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        list: &ResolverValue<B>,
        mut names: Option<&mut Vec<AtomicString>>,
    ) -> ColorSchemeFlags {
        let mut flags = ColorSchemeFlag::kNormal as u8;
        for value in b.List(list).expect("color schemes") {
            if let Some(name) = b.CustomIdent(value) {
                if let Some(names) = names.as_deref_mut() {
                    names.push(name);
                }
            } else if let Some(id) = b.Identifier(value) {
                if let Some(names) = names.as_deref_mut() {
                    names.push(AtomicString::from_utf16(
                        foundation::StringView::from(&value.CssText()).Span16(),
                    ));
                }
                flags |= match id {
                    CSSValueID::kDark => ColorSchemeFlag::kDark as u8,
                    CSSValueID::kLight => ColorSchemeFlag::kLight as u8,
                    CSSValueID::kOnly => ColorSchemeFlag::kOnly as u8,
                    _ => 0,
                };
            } else {
                unreachable!("color scheme entry")
            }
        }
        flags
    }
    pub fn ConvertOverflowClipMargin<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Option<StyleOverflowClipMargin> {
        let list = b.List(v).expect("overflow clip margin list");
        let (mut reference, mut length) = (None, None);
        if let Some(id) = b.Identifier(list[0]) {
            reference = Some(id);
        } else {
            length = Some(list[0]);
        }
        if list.len() > 1 {
            length = Some(list[1]);
        }
        let reference = match reference {
            None | Some(CSSValueID::kPaddingBox) => ReferenceBox::kPaddingBox,
            Some(CSSValueID::kBorderBox) => ReferenceBox::kBorderBox,
            Some(CSSValueID::kContentBox) => ReferenceBox::kContentBox,
            _ => unreachable!("overflow clip reference box"),
        };
        let margin = length.map_or(LayoutUnit::default(), |v| Self::ConvertLayoutUnit(b, s, v));
        Some(StyleOverflowClipMargin::new(reference, margin))
    }
    pub fn ConvertViewTimelineAxis<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> Vec<TimelineAxis> {
        b.List(v)
            .expect("timeline axes")
            .into_iter()
            .map(|v| b.TimelineAxis(v))
            .collect()
    }
    pub fn ConvertSingleTimelineInset<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> TimelineInset {
        let (first, second) = b.Pair(v).expect("timeline inset pair");
        TimelineInset::new(
            &Self::ConvertLengthOrAuto(b, s, first),
            &Self::ConvertLengthOrAuto(b, s, second),
        )
    }
    pub fn ConvertViewTimelineInset<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Vec<TimelineInset> {
        b.List(v)
            .expect("timeline insets")
            .into_iter()
            .map(|v| Self::ConvertSingleTimelineInset(b, s, v))
            .collect()
    }
    pub fn ConvertViewTimelineName<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> Vec<AtomicString> {
        ConvertNoneOrCustomIdentListUnscoped(b, s, v)
    }
    pub fn ConvertTimelineScope<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleTimelineScope {
        if let Some(id) = b.Identifier(v) {
            return StyleTimelineScope::new(
                if id == CSSValueID::kNone {
                    StyleTimelineScopeType::kNone
                } else {
                    StyleTimelineScopeType::kAll
                },
                vec![],
            );
        }
        StyleTimelineScope::new(
            StyleTimelineScopeType::kNames,
            b.List(v)
                .expect("timeline scope names")
                .into_iter()
                .map(|v| Self::ConvertCustomIdentUnscoped(b, s, v))
                .collect(),
        )
    }
    pub fn ConvertPositionArea<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
        allow_any: bool,
    ) -> PositionArea {
        if let Some(id) = b.Identifier(v) {
            if id == CSSValueID::kNone {
                return PositionArea::default();
            }
            let (start, end) = ExtractPositionAreaSpan(id);
            if IsRepeatedPositionAreaValue(id) {
                return PositionArea::new(start, end, start, end);
            }
            let second = if allow_any {
                PositionAreaRegion::kAny
            } else {
                PositionAreaRegion::kAll
            };
            return PositionArea::new(start, end, second, second);
        }
        let (first, second) = b.Pair(v).expect("position area pair");
        let (a, z) = ExtractPositionAreaSpan(b.Identifier(first).unwrap());
        let (c, d) = ExtractPositionAreaSpan(b.Identifier(second).unwrap());
        PositionArea::new(a, z, c, d)
    }
    pub fn ConvertSinglePositionTryFallback<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
        allow_any: bool,
    ) -> PositionTryFallback {
        if b.Pair(v).is_some() || b.Identifier(v).is_some() {
            return PositionTryFallback::from_position_area(Self::ConvertPositionArea(
                b, v, allow_any,
            ));
        }
        let mut name = Member::default();
        let mut tactics = [TryTactic::kNone; 3];
        let mut index = 0;
        for value in b.List(v).expect("try fallback list") {
            if b.CustomIdent(value).is_some() {
                name = Self::ConvertCustomIdent(b, s, value);
                continue;
            }
            assert!(index < tactics.len());
            tactics[index] = b.TryTactic(value);
            index += 1;
        }
        PositionTryFallback::from_name(name, tactics)
    }
    pub fn ConvertTextFit<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> TextFit {
        let list = b.List(v).expect("text fit list");
        let kind = match b.Identifier(list[0]) {
            Some(CSSValueID::kNone) => TextFitType::kNone,
            Some(CSSValueID::kGrow) => TextFitType::kGrow,
            _ => TextFitType::kShrink,
        };
        let mut index = 1;
        let mut target = TextFitTarget::kConsistent;
        if index < list.len() {
            if let Some(id) = b.Identifier(list[index]) {
                target = match id {
                    CSSValueID::kConsistent => TextFitTarget::kConsistent,
                    CSSValueID::kPerLine => TextFitTarget::kPerLine,
                    _ => TextFitTarget::kPerLineAll,
                };
                index += 1;
            }
        }
        let mut limit = None;
        if list.len() > index {
            if list[index].IsPrimitiveValue() {
                limit =
                    Some(b.Percentage(&s.CssToLengthConversionData(), list[index]) as f32 / 100.0);
            }
        }
        TextFit::new(kind, target, limit)
    }
    pub fn ConvertTextOverflow<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        v: &ResolverValue<B>,
    ) -> TextOverflowData {
        if b.StringValue(v).is_some() {
            return TextOverflowData::from_string(b.StringContents(v));
        }
        TextOverflowData::from_type(if b.Identifier(v) == Some(CSSValueID::kEllipsis) {
            TextOverflowType::kEllipsis
        } else {
            TextOverflowType::kClip
        })
    }
    pub fn ConvertMaxLines<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> MaxLinesData {
        if b.Identifier(v).is_some() {
            return MaxLinesData::new(0, true);
        }
        let (num, auto) = if let Some((first, _)) = b.Pair(v) {
            (first, true)
        } else {
            (v, false)
        };
        MaxLinesData::new(b.PrimitiveUInt16(&s.CssToLengthConversionData(), num), auto)
    }
    pub fn ConvertTriggerScope<B: StyleBuilderConverterFinalBackend>(
        b: &B,
        s: &StyleResolverState<B>,
        v: &ResolverValue<B>,
    ) -> StyleNameScope {
        Self::ConvertNameScope(b, s, v)
    }
}

fn ConvertOffsetPathValueToOperation<B: StyleBuilderConverterFinalBackend>(
    b: &B,
    s: &StyleResolverState<B>,
    v: &ResolverValue<B>,
    coord: CoordBox,
) -> Rc<dyn OffsetPathOperation> {
    if b.IsURI(v) {
        b.NewReferenceOffsetPath(
            b.URIString(v),
            b.SVGResource(s, CSSPropertyID::kOffsetPath, v),
            coord,
        )
    } else {
        b.NewShapeOffsetPath(b.BasicShapeForValue(s, v), coord)
    }
}
fn ComputeColorValue<B: StyleBuilderConverterFinalBackend>(
    b: &B,
    data: &B::LengthConversionData,
    v: &ResolverValue<B>,
    document: &B::Document,
    scheme: ColorScheme,
) -> Rc<ResolverValue<B>> {
    let colors = b.DocumentLinkColors(document);
    let provider = b.DocumentColorProvider(document, scheme);
    let context = ResolveColorValueContext {
        length_resolver: data,
        text_link_colors: &colors,
        used_color_scheme: scheme,
        color_provider: provider.as_ref(),
        can_expose_accent_color: b.DocumentInWebAppScope(document)
            && b.DocumentInitialProfile(document),
        for_visited_link: false,
    };
    b.StyleColorToCSSValue(ResolveColorValue(b, v, &context))
}
fn ComputeRegisteredPropertyValue<B: StyleBuilderConverterFinalBackend>(
    b: &B,
    document: &B::Document,
    s: Option<&StyleResolverState<B>>,
    data: &B::LengthConversionData,
    v: &ResolverValue<B>,
    context: Option<&CSSParserContext<B::ConverterParserPlatform>>,
) -> Rc<ResolverValue<B>> {
    if v.IsFunctionValue() {
        let values = b
            .List(v)
            .expect("registered function list")
            .into_iter()
            .map(|v| ComputeRegisteredPropertyValue(b, document, s, data, v, context))
            .collect();
        return b.NewFunctionValue(b.FunctionType(v), values);
    }
    if let Some(values) = b.List(v) {
        let computed = values
            .into_iter()
            .map(|v| ComputeRegisteredPropertyValue(b, document, s, data, v, context))
            .collect();
        return b.NewListWithSeparatorFrom(v, computed);
    }
    if v.IsPrimitiveValue() {
        if !b.IsCalculated(v) && (b.IsPx(v) || b.IsPercentage(v)) {
            return b.ShareValue(v);
        }
        if b.IsLength(v) || b.IsPercentage(v) || !b.IsResolvableBeforeLayout(v) {
            let length = b.PrimitiveLength(&b.Unzoomed(data), v);
            return b.NewPrimitiveFromLength(length, 1.0);
        }
        if b.IsNumber(v) && b.IsCalculated(v) {
            return b.NewNumericLiteral(b.Number(data, v), UnitType::kNumber);
        }
        if b.IsAngle(v) {
            return b.NewNumericLiteral(b.Degrees(data, v), UnitType::kDegrees);
        }
        if b.IsTime(v) {
            return b.NewNumericLiteral(b.TimeSeconds(data, v), UnitType::kSeconds);
        }
        if b.IsResolution(v) {
            return b.NewNumericLiteral(b.DotsPerPixel(data, v), UnitType::kDotsPerPixel);
        }
    }
    let scheme = s.map_or(ColorScheme::kLight, |s| b.UsedColorScheme(s));
    if let Some(id) = b.Identifier(v) {
        if id == CSSValueID::kCurrentcolor {
            return b.ShareValue(v);
        }
        if b.IsColorKeyword(id) {
            return ComputeColorValue(b, data, v, document, scheme);
        }
    }
    if b.IsURI(v) {
        if let Some(context) = context {
            return b.ComputedURIValue(v, context.BaseURL(), context.Charset());
        }
        let url = B::ConverterParserPlatform::NullURL();
        let charset = B::ConverterParserPlatform::EmptyTextEncoding();
        return b.ComputedURIValue(v, &url, &charset);
    }
    if let Some((first, second)) = b.LightDarkPair(v) {
        return ComputeRegisteredPropertyValue(
            b,
            document,
            s,
            data,
            if scheme == ColorScheme::kLight {
                first
            } else {
                second
            },
            context,
        );
    }
    if v.IsAlphaColorValue()
        || v.IsColorMixValue()
        || v.IsRelativeColorValue()
        || v.IsContrastColorValue()
        || v.IsUnresolvedColorValue()
    {
        return ComputeColorValue(b, data, v, document, scheme);
    }
    if v.IsCustomIdentValue() {
        return b.ResolveCustomIdentValue(data, v);
    }
    b.ShareValue(v)
}
pub(crate) fn IsRepeatedPositionAreaValue(id: CSSValueID) -> bool {
    matches!(
        id,
        CSSValueID::kSpanAll
            | CSSValueID::kCenter
            | CSSValueID::kStart
            | CSSValueID::kEnd
            | CSSValueID::kSpanStart
            | CSSValueID::kSpanEnd
            | CSSValueID::kSelfStart
            | CSSValueID::kSelfEnd
            | CSSValueID::kSpanSelfStart
            | CSSValueID::kSpanSelfEnd
            | CSSValueID::kAny
    )
}

pub(crate) fn ExtractPositionAreaSpan(id: CSSValueID) -> (PositionAreaRegion, PositionAreaRegion) {
    match id {
        CSSValueID::kSpanAll => (PositionAreaRegion::kAll, PositionAreaRegion::kAll),
        CSSValueID::kCenter => (PositionAreaRegion::kCenter, PositionAreaRegion::kCenter),
        CSSValueID::kLeft => (PositionAreaRegion::kLeft, PositionAreaRegion::kLeft),
        CSSValueID::kRight => (PositionAreaRegion::kRight, PositionAreaRegion::kRight),
        CSSValueID::kSpanLeft => (PositionAreaRegion::kLeft, PositionAreaRegion::kCenter),
        CSSValueID::kSpanRight => (PositionAreaRegion::kCenter, PositionAreaRegion::kRight),
        CSSValueID::kXStart => (PositionAreaRegion::kXStart, PositionAreaRegion::kXStart),
        CSSValueID::kXEnd => (PositionAreaRegion::kXEnd, PositionAreaRegion::kXEnd),
        CSSValueID::kSpanXStart => (PositionAreaRegion::kXStart, PositionAreaRegion::kCenter),
        CSSValueID::kSpanXEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kXEnd),
        CSSValueID::kSelfXStart => (
            PositionAreaRegion::kSelfXStart,
            PositionAreaRegion::kSelfXStart,
        ),
        CSSValueID::kSelfXEnd => (PositionAreaRegion::kSelfXEnd, PositionAreaRegion::kSelfXEnd),
        CSSValueID::kSpanSelfXStart => {
            (PositionAreaRegion::kSelfXStart, PositionAreaRegion::kCenter)
        }
        CSSValueID::kSpanSelfXEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kSelfXEnd),
        CSSValueID::kTop => (PositionAreaRegion::kTop, PositionAreaRegion::kTop),
        CSSValueID::kBottom => (PositionAreaRegion::kBottom, PositionAreaRegion::kBottom),
        CSSValueID::kSpanTop => (PositionAreaRegion::kTop, PositionAreaRegion::kCenter),
        CSSValueID::kSpanBottom => (PositionAreaRegion::kCenter, PositionAreaRegion::kBottom),
        CSSValueID::kYStart => (PositionAreaRegion::kYStart, PositionAreaRegion::kYStart),
        CSSValueID::kYEnd => (PositionAreaRegion::kYEnd, PositionAreaRegion::kYEnd),
        CSSValueID::kSpanYStart => (PositionAreaRegion::kYStart, PositionAreaRegion::kCenter),
        CSSValueID::kSpanYEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kYEnd),
        CSSValueID::kSelfYStart => (
            PositionAreaRegion::kSelfYStart,
            PositionAreaRegion::kSelfYStart,
        ),
        CSSValueID::kSelfYEnd => (PositionAreaRegion::kSelfYEnd, PositionAreaRegion::kSelfYEnd),
        CSSValueID::kSpanSelfYStart => {
            (PositionAreaRegion::kSelfYStart, PositionAreaRegion::kCenter)
        }
        CSSValueID::kSpanSelfYEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kSelfYEnd),
        CSSValueID::kBlockStart => (
            PositionAreaRegion::kBlockStart,
            PositionAreaRegion::kBlockStart,
        ),
        CSSValueID::kBlockEnd => (PositionAreaRegion::kBlockEnd, PositionAreaRegion::kBlockEnd),
        CSSValueID::kSpanBlockStart => {
            (PositionAreaRegion::kBlockStart, PositionAreaRegion::kCenter)
        }
        CSSValueID::kSpanBlockEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kBlockEnd),
        CSSValueID::kSelfBlockStart => (
            PositionAreaRegion::kSelfBlockStart,
            PositionAreaRegion::kSelfBlockStart,
        ),
        CSSValueID::kSelfBlockEnd => (
            PositionAreaRegion::kSelfBlockEnd,
            PositionAreaRegion::kSelfBlockEnd,
        ),
        CSSValueID::kSpanSelfBlockStart => (
            PositionAreaRegion::kSelfBlockStart,
            PositionAreaRegion::kCenter,
        ),
        CSSValueID::kSpanSelfBlockEnd => (
            PositionAreaRegion::kCenter,
            PositionAreaRegion::kSelfBlockEnd,
        ),
        CSSValueID::kInlineStart => (
            PositionAreaRegion::kInlineStart,
            PositionAreaRegion::kInlineStart,
        ),
        CSSValueID::kInlineEnd => (
            PositionAreaRegion::kInlineEnd,
            PositionAreaRegion::kInlineEnd,
        ),
        CSSValueID::kSpanInlineStart => (
            PositionAreaRegion::kInlineStart,
            PositionAreaRegion::kCenter,
        ),
        CSSValueID::kSpanInlineEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kInlineEnd),
        CSSValueID::kSelfInlineStart => (
            PositionAreaRegion::kSelfInlineStart,
            PositionAreaRegion::kSelfInlineStart,
        ),
        CSSValueID::kSelfInlineEnd => (
            PositionAreaRegion::kSelfInlineEnd,
            PositionAreaRegion::kSelfInlineEnd,
        ),
        CSSValueID::kSpanSelfInlineStart => (
            PositionAreaRegion::kSelfInlineStart,
            PositionAreaRegion::kCenter,
        ),
        CSSValueID::kSpanSelfInlineEnd => (
            PositionAreaRegion::kCenter,
            PositionAreaRegion::kSelfInlineEnd,
        ),
        CSSValueID::kStart => (PositionAreaRegion::kStart, PositionAreaRegion::kStart),
        CSSValueID::kEnd => (PositionAreaRegion::kEnd, PositionAreaRegion::kEnd),
        CSSValueID::kSpanStart => (PositionAreaRegion::kStart, PositionAreaRegion::kCenter),
        CSSValueID::kSpanEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kEnd),
        CSSValueID::kSelfStart => (
            PositionAreaRegion::kSelfStart,
            PositionAreaRegion::kSelfStart,
        ),
        CSSValueID::kSelfEnd => (PositionAreaRegion::kSelfEnd, PositionAreaRegion::kSelfEnd),
        CSSValueID::kSpanSelfStart => (PositionAreaRegion::kSelfStart, PositionAreaRegion::kCenter),
        CSSValueID::kSpanSelfEnd => (PositionAreaRegion::kCenter, PositionAreaRegion::kSelfEnd),
        CSSValueID::kAny => (PositionAreaRegion::kAny, PositionAreaRegion::kAny),
        _ => unreachable!("position area keyword"),
    }
}

#![allow(non_snake_case)]

#[path = "clip_path_native.rs"]
mod clip_path_native;
#[path = "mask_image_native.rs"]
mod mask_image_native;

use std::cell::Cell;
use std::marker::PhantomData;
use std::ptr::NonNull;

use foundation::{
    gfx, AtomicString, BlendMode, CSSBitset, CSSPropertyID, CalculationValue,
    DisplayAdjustmentContext, EAspectRatioType, EBlockEllipsis, EBorderCollapse, EBorderStyle,
    EBoxDecorationBreak, EBoxSizing, EBreakBetween, EBreakInside, ECaptionSide, EClear,
    EColumnFill, EColumnSpan, EColumnWrap, EContinue, EDisplay, EEmptyCells, EFieldSizing,
    EFlexDirection, EFloat, EInlineBlockBaselineEdge, EIsolation, EListStylePosition, EMathShift,
    EMathStyle, EObjectFit, EOverflow, EOverflowWrap, EPosition, ERubyAlign, ERubyOverhang,
    EScrollbarWidth, ETableLayout, ETextAlign, ETextAlignLast, ETextCombine,
    ETextDecorationSkipInk, ETextDecorationStyle, ETextOrientation, ETextTransform, EWordBreak,
    HeapVector, Hyphens as NativeHyphens, LayoutUnit, Length, LengthBox, LengthPoint,
    LengthValueRange, LineBreak as NativeLineBreak, MakeGarbageCollected, Member,
    PhysicalDirection, PixelsAndPercent, RubyPosition as NativeRubyPosition, ScopedCSSName,
    ScopedCSSNameList, String as BlinkString, StyleAspectRatio, StyleInitialLetter, StyleNameScope,
    StyleNameScopeType, TabSize, TabSizeValueType, TextDecorationLine, TextDecorationThickness,
    TextDirection as NativeTextDirection, TextWrapMode as NativeTextWrapMode,
    TextWrapStyle as NativeTextWrapStyle, TransformOperations, UnicodeBidi as NativeUnicodeBidi,
    Vector, WritingDirectionMode, WritingMode as NativeWritingMode,
};

use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::css::style_color::StyleColor;
use layoutng_style::css::white_space::EWhiteSpace;
use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::style::computed_grid_template_areas::ComputedGridTemplateAreas;
use layoutng_style::style::computed_grid_track_list::ComputedGridTrackList;
use layoutng_style::style::computed_style::{
    ComputedStyle as NativeComputedStyle, ComputedStyleBuilder,
};
use layoutng_style::style::computed_style_constants::{
    Containment, ContentDistributionType, ContentPosition, EMarginTrim, EVerticalAlign,
    FlexWrapMode, GridAutoFlow, ItemPosition, ItemPositionType,
    OverflowAlignment as NativeOverflowAlignment, PseudoId,
    ScrollbarGutter as NativeScrollbarGutter, ShapeBox,
};
use layoutng_style::style::display_adjustment::AdjustComputedDisplayForLayout;
use layoutng_style::style::filter_operation::{
    BasicColorMatrixFilterOperation, BasicComponentTransferFilterOperation, BlurFilterOperation,
    DropShadowFilterOperation, FilterOperation, OperationType,
};
use layoutng_style::style::filter_operations::FilterOperations;
use layoutng_style::style::gap_data_list::GapDataList;
use layoutng_style::style::grid_area::{GridArea, GridSpan, NamedGridAreaMap};
use layoutng_style::style::grid_position::GridPosition as NativeGridPosition;
use layoutng_style::style::grid_track_list::{
    AutoRepeatType, GridAxisType, GridTrackList, GridTrackRepeatType,
};
use layoutng_style::style::grid_track_size::{GridTrackSize, GridTrackSizeType};
use layoutng_style::style::list_style_type_data::ListStyleTypeData;
use layoutng_style::style::max_lines_data::MaxLinesData;
use layoutng_style::style::named_grid_lines_map::NamedGridLinesMap;
use layoutng_style::style::shadow_data::{ShadowData, ShadowStyle};
use layoutng_style::style::shape_value::ShapeValue;
use layoutng_style::style::style_content_alignment_data::StyleContentAlignmentData;
use layoutng_style::style::style_flex_wrap_data::StyleFlexWrapData;
use layoutng_style::style::style_intrinsic_length::{
    StyleIntrinsicLength, StyleIntrinsicLengthOptions,
};
use layoutng_style::style::style_overflow_clip_margin::{
    ReferenceBox as NativeOverflowClipReferenceBox, StyleOverflowClipMargin,
};
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;
use layoutng_style::style::style_trigger_scope::StyleTriggerScope;
use layoutng_style::style::style_will_change_data::StyleWillChangeData;
use layoutng_style::style::text_indent_flags::TextIndentFlags;

use crate::internal::constraint_space::{
    ConstraintSpace as NativeConstraintSpace, FragmentationType,
};
use crate::internal::constraint_space_builder::ConstraintSpaceBuilder;
use crate::internal::layout_font_resolver::{NativeFontRequest, NativeFontResolver};
use crate::internal::layout_input::{
    AlignItems, BorderLineStyle, BoxDecorationBreak, BoxSizing, BreakRule, CaptionSide, ClearSide,
    ColumnWrap, ComputedStyle, ConstraintSpace, ContainIntrinsicLength, ContentAlignment, Display,
    EmptyCells, ExtendedStyle, FieldSizing, FlexBasisSizing, FlexDirection, FloatSide, GridLine,
    GridLineKind, GridRepeatKind, GridTemplateAreasInput, GridTrack, GridTrackBreadth,
    GridTrackBreadthKind, GridTrackListInput, Hyphens, InitialLetterSink, IntrinsicSizing,
    JustifyContent, LineBreak, ListStylePosition, ListStyleType, MarginTrim, MathShift, MathStyle,
    NativeNodeConstructionData, NodeKind, ObjectFit, Overflow, OverflowAlignment,
    OverflowClipReferenceBox, OverflowWrap, Position, RubyAlign, RubyOverhang, RubyPosition,
    ScrollbarGutter, ScrollbarWidth, SelfAlignment, ShapeCoordinate, ShapeOutsideKind, ShapePoint,
    ShapeRadiusKind, ShapeReferenceBox, TextAlign, TextAlignLast, TextCombine, TextDecorationStyle,
    TextDirection, TextOrientation, TextTransform, TextWrapMode, TextWrapStyle, UnicodeBidi,
    VerticalAlign, WhiteSpace, WordBreak, WritingMode,
};
use crate::internal::paint_input::{PaintBlendMode, PaintFilterType, PaintTransformOperationKind};

// Object constructors owned by foundation/style-values remain typed links.
unsafe extern "Rust" {
    fn FoundationTextTransformFromBits(bits: u32) -> ETextTransform;
    fn RoundForImpreciseConversionI16(value: f64) -> i16;
    fn NativeInitialLetterOmitted(size: f32) -> StyleInitialLetter;
    fn NativeInitialLetterInteger(size: f32, sink: i32) -> StyleInitialLetter;
    fn NativeInitialLetterDrop(size: f32) -> StyleInitialLetter;
    fn NativeInitialLetterRaise(size: f32) -> StyleInitialLetter;
}

// cpp: layoutng/internal/boundary/native_input.cc:1449-1462
fn NativeTriggerScopeAll() -> StyleTriggerScope {
    StyleNameScope::new(StyleNameScopeType::kAll, std::ptr::null(), std::ptr::null())
}

fn NativeTriggerScopeNames(names: &[AtomicString]) -> StyleTriggerScope {
    let mut entries = HeapVector::new();
    for name in names {
        let scoped = MakeGarbageCollected(ScopedCSSName::new(name, std::ptr::null()));
        entries.push_back(Member::from_ptr(scoped));
    }
    let list = MakeGarbageCollected(ScopedCSSNameList::new(entries));
    StyleNameScope::new(StyleNameScopeType::kNames, std::ptr::null(), list)
}

// cpp: layoutng/internal/boundary/native_input.cc:28-31
thread_local! {
    static CURRENT_FONT_RESOLVER: Cell<Option<NonNull<dyn NativeFontResolver>>> = Cell::new(None);
}

// cpp: layoutng/internal/boundary/native_input.h:25-37
pub struct NativeFontResolverScope<'a> {
    previous_: Option<NonNull<dyn NativeFontResolver>>,
    resolver_lifetime_: PhantomData<&'a mut dyn NativeFontResolver>,
}

impl<'a> NativeFontResolverScope<'a> {
    // cpp: layoutng/internal/boundary/native_input.cc:33-36
    pub fn new(resolver: &'a mut dyn NativeFontResolver) -> Self {
        // The scope owns the borrow for its whole lifetime. The thread-local
        // slot cannot express that lifetime, so erase it only while the scope
        // is alive and restore the previous pointer on drop.
        let resolver = NonNull::from(resolver);
        let resolver = unsafe {
            std::mem::transmute::<
                NonNull<dyn NativeFontResolver + 'a>,
                NonNull<dyn NativeFontResolver>,
            >(resolver)
        };
        let previous_ = CURRENT_FONT_RESOLVER.with(|current| current.replace(Some(resolver)));
        Self {
            previous_,
            resolver_lifetime_: PhantomData,
        }
    }

    // Rust-only boundary helper: the standalone tree builder must call
    // PrepareNativeStyle through this same resolver while its TLS scope lives.
    // The caller owns the stable resolver allocation and drops this scope
    // before it can drop or move that allocation.
    pub(crate) unsafe fn new_from_raw(
        resolver: *mut dyn NativeFontResolver,
    ) -> NativeFontResolverScope<'static> {
        let resolver = NonNull::new(resolver).expect("native font resolver");
        let previous_ = CURRENT_FONT_RESOLVER.with(|current| current.replace(Some(resolver)));
        NativeFontResolverScope {
            previous_,
            resolver_lifetime_: PhantomData,
        }
    }
}

impl Drop for NativeFontResolverScope<'_> {
    // cpp: layoutng/internal/boundary/native_input.cc:38-41
    fn drop(&mut self) {
        CURRENT_FONT_RESOLVER.with(|current| {
            assert!(current.get().is_some(), "current_font_resolver");
            current.set(self.previous_);
        });
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:43-46
pub fn CurrentNativeFontResolver() -> *mut dyn NativeFontResolver {
    CURRENT_FONT_RESOLVER.with(|current| current.get().expect("current_font_resolver").as_ptr())
}

// cpp: layoutng/internal/boundary/native_input.cc:49-51
#[derive(Debug)]
pub(crate) struct InvalidLayoutInput(String);

impl std::fmt::Display for InvalidLayoutInput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for InvalidLayoutInput {}

#[allow(non_snake_case)]
fn Invalid(field: &str) -> ! {
    std::panic::panic_any(InvalidLayoutInput(format!("Invalid layout input: {field}")))
}

// cpp: layoutng/internal/boundary/native_input.cc:55-60
#[allow(non_snake_case)]
fn Number(value: f64, field: &str, nonnegative: bool) -> f64 {
    if !value.is_finite()
        || (nonnegative && value < 0.0)
        || value.abs() > LayoutUnit::Max().ToDouble()
    {
        Invalid(field);
    }
    value
}

// cpp: layoutng/internal/boundary/native_input.cc:61-63
#[allow(non_snake_case)]
fn Pixels(value: f64, field: &str, nonnegative: bool) -> Length {
    Length::Fixed(Number(value, field, nonnegative))
}

// cpp: layoutng/internal/boundary/native_input.cc:65-77
#[derive(Clone, Copy)]
enum NativeLengthValueRange {
    All,
    NonNegative,
}

// cpp: layoutng/internal/boundary/native_input.cc:65-77
fn MakeCalculatedLengthFromInput(
    pixels: f32,
    percent: f32,
    has_pixels: bool,
    has_percent: bool,
    range: NativeLengthValueRange,
) -> Length {
    let value = PixelsAndPercent::new(pixels, percent, has_pixels, has_percent);
    let range = match range {
        NativeLengthValueRange::All => LengthValueRange::kAll,
        NativeLengthValueRange::NonNegative => LengthValueRange::kNonNegative,
    };
    Length::from_calculation_value(CalculationValue::new(value, range))
}

#[allow(non_snake_case)]
fn CalculatedLength(
    pixels: Option<f64>,
    percent: Option<f64>,
    field: &str,
    range: NativeLengthValueRange,
) -> Length {
    if pixels.is_none() && percent.is_none() {
        Invalid(field);
    }
    if let Some(value) = pixels {
        Number(value, field, false);
    }
    if let Some(value) = percent {
        Number(value, field, false);
    }
    {
        MakeCalculatedLengthFromInput(
            pixels.unwrap_or(0.0) as f32,
            percent.unwrap_or(0.0) as f32,
            pixels.is_some(),
            percent.is_some(),
            range,
        )
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:79-88
#[allow(non_snake_case)]
fn LengthPercentage(pixels: f64, percentage: f64, field: &str) -> Length {
    Number(pixels, field, false);
    Number(percentage, field, false);
    if percentage == 0.0 {
        return Length::Fixed(pixels);
    }
    if pixels == 0.0 {
        return Length::Percent(percentage);
    }
    CalculatedLength(
        Some(pixels),
        Some(percentage),
        field,
        NativeLengthValueRange::All,
    )
}

// cpp: layoutng/internal/boundary/native_input.cc:90-101
#[allow(non_snake_case)]
fn Dimension(px: Option<f64>, percent: Option<f64>, calculated: bool, field: &str) -> Length {
    if calculated {
        return CalculatedLength(px, percent, field, NativeLengthValueRange::NonNegative);
    }
    if px.is_some() && percent.is_some() {
        Invalid(field);
    }
    if let Some(value) = px {
        return Pixels(value, field, true);
    }
    if let Some(value) = percent {
        return Length::Percent(Number(value, field, true));
    }
    Length::Auto().clone()
}

// cpp: layoutng/internal/boundary/native_input.cc:103-123
#[allow(non_snake_case)]
fn FlexBasis(style: &ComputedStyle, extra: &ExtendedStyle) -> Length {
    if extra.flex_basis_sizing == FlexBasisSizing::kAuto {
        return Dimension(
            style.flex_basis,
            extra.flex_basis_percent,
            extra.flex_basis_calculated,
            "flex_basis",
        );
    }
    if style.flex_basis.is_some()
        || extra.flex_basis_percent.is_some()
        || extra.flex_basis_calculated
    {
        Invalid("conflicting flex_basis sizing");
    }
    match extra.flex_basis_sizing {
        FlexBasisSizing::kAuto => Invalid("flex_basis_sizing"),
        FlexBasisSizing::kContent => Length::Content(),
        FlexBasisSizing::kMinContent => Length::MinContent().clone(),
        FlexBasisSizing::kMaxContent => Length::MaxContent().clone(),
        FlexBasisSizing::kFitContent => Length::FitContent().clone(),
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:124-127
#[allow(non_snake_case)]
fn Count(value: u32, maximum: u32, field: &str) -> u32 {
    if value == 0 || value > maximum {
        Invalid(field);
    }
    value
}

// cpp: layoutng/internal/boundary/native_input.cc:129-139
#[allow(non_snake_case)]
fn BorderWidth(width: f64) -> i32 {
    Number(width, "border", true);
    if width > 0.0 && width < 1.0 {
        return 1;
    }
    width.floor().clamp(0.0, LayoutUnit::Max().ToInt() as f64) as i32
}

// cpp: layoutng/internal/boundary/native_input.cc:140-147
fn ConvertWritingMode(value: WritingMode) -> NativeWritingMode {
    match value {
        WritingMode::kHorizontalTb => NativeWritingMode::kHorizontalTb,
        WritingMode::kVerticalRl => NativeWritingMode::kVerticalRl,
        WritingMode::kVerticalLr => NativeWritingMode::kVerticalLr,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:149-155
fn ConvertTextDirection(value: TextDirection) -> NativeTextDirection {
    match value {
        TextDirection::kLtr => NativeTextDirection::kLtr,
        TextDirection::kRtl => NativeTextDirection::kRtl,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:157-165
fn ConvertShapeReferenceBox(value: ShapeReferenceBox) -> ShapeBox {
    match value {
        ShapeReferenceBox::kMarginBox => ShapeBox::kMarginBox,
        ShapeReferenceBox::kBorderBox => ShapeBox::kBorderBox,
        ShapeReferenceBox::kPaddingBox => ShapeBox::kPaddingBox,
        ShapeReferenceBox::kContentBox => ShapeBox::kContentBox,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:167-174
#[allow(non_snake_case)]
fn ShapeMarginLength(value: &ShapeCoordinate) -> Length {
    Number(value.pixels, "shape_margin", false);
    Number(value.percentage, "shape_margin", false);
    if value.percentage == 0.0 {
        return Pixels(value.pixels, "shape_margin", true);
    }
    CalculatedLength(
        Some(value.pixels),
        Some(value.percentage),
        "shape_margin",
        NativeLengthValueRange::NonNegative,
    )
}

// cpp: layoutng/internal/boundary/native_input.cc:176-185
fn ConvertObjectFit(value: ObjectFit) -> EObjectFit {
    match value {
        ObjectFit::kFill => EObjectFit::kFill,
        ObjectFit::kContain => EObjectFit::kContain,
        ObjectFit::kCover => EObjectFit::kCover,
        ObjectFit::kNone => EObjectFit::kNone,
        ObjectFit::kScaleDown => EObjectFit::kScaleDown,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:187-195
#[allow(non_snake_case)]
fn PositionLength(fraction: f64, offset: f64, field: &str) -> Length {
    Number(fraction, field, false);
    Number(offset, field, false);
    let percent = (fraction * 100.0) as f32;
    if offset == 0.0 {
        return Length::Percent(percent as f64);
    }
    {
        MakeCalculatedLengthFromInput(
            offset as f32,
            percent,
            true,
            true,
            NativeLengthValueRange::All,
        )
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:197-213
#[allow(non_snake_case)]
fn EdgeLength(
    pixels: f64,
    percent: Option<f64>,
    automatic: bool,
    calculated: bool,
    field: &str,
    nonnegative: bool,
) -> Length {
    if automatic {
        if percent.is_some() || calculated {
            Invalid(field);
        }
        return Length::Auto().clone();
    }
    if calculated {
        return CalculatedLength(
            Some(pixels),
            percent,
            field,
            if nonnegative {
                NativeLengthValueRange::NonNegative
            } else {
                NativeLengthValueRange::All
            },
        );
    }
    if let Some(value) = percent {
        return Length::Percent(Number(value, field, nonnegative));
    }
    Pixels(pixels, field, nonnegative)
}

fn MarginLength(
    pixels: f64,
    percent: Option<f64>,
    automatic: bool,
    calculated: bool,
    quirk: bool,
    field: &str,
) -> Length {
    let mut length = EdgeLength(pixels, percent, automatic, calculated, field, false);
    length.SetQuirk(quirk);
    length
}

// cpp: layoutng/internal/boundary/native_input.cc:215-225
#[allow(non_snake_case)]
fn InsetLength(pixels: Option<f64>, percent: Option<f64>, calculated: bool, field: &str) -> Length {
    if calculated {
        return CalculatedLength(pixels, percent, field, NativeLengthValueRange::All);
    }
    if pixels.is_some() && percent.is_some() {
        Invalid(field);
    }
    if let Some(value) = percent {
        return Length::Percent(Number(value, field, false));
    }
    if let Some(value) = pixels {
        return Pixels(value, field, false);
    }
    Length::Auto().clone()
}

// cpp: layoutng/internal/boundary/native_input.cc:227-239
#[allow(non_snake_case)]
fn OptionalDimension(
    pixels: Option<f64>,
    percent: Option<f64>,
    calculated: bool,
    field: &str,
    none_default: bool,
) -> Length {
    if calculated {
        return CalculatedLength(pixels, percent, field, NativeLengthValueRange::NonNegative);
    }
    if pixels.is_some() && percent.is_some() {
        Invalid(field);
    }
    if let Some(value) = pixels {
        return Pixels(value, field, true);
    }
    if let Some(value) = percent {
        return Length::Percent(Number(value, field, true));
    }
    if none_default {
        Length::None()
    } else {
        Length::Auto().clone()
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:241-262
#[allow(non_snake_case)]
fn IntrinsicDimension(
    pixels: Option<f64>,
    percent: Option<f64>,
    calculated: bool,
    sizing: IntrinsicSizing,
    field: &str,
    optional: bool,
    none_default: bool,
) -> Length {
    if sizing == IntrinsicSizing::kAuto {
        return if optional {
            OptionalDimension(pixels, percent, calculated, field, none_default)
        } else {
            Dimension(pixels, percent, calculated, field)
        };
    }
    if pixels.is_some() || percent.is_some() || calculated {
        Invalid(&format!("conflicting {field} sizing"));
    }
    match sizing {
        IntrinsicSizing::kAuto => Invalid(field),
        IntrinsicSizing::kMinContent => Length::MinContent().clone(),
        IntrinsicSizing::kMaxContent => Length::MaxContent().clone(),
        IntrinsicSizing::kFitContent => Length::FitContent().clone(),
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:264-278
#[allow(non_snake_case)]
fn VerticalAlignLength(extra: &ExtendedStyle) -> Length {
    if extra.vertical_align_calculated {
        return CalculatedLength(
            extra.vertical_align_length,
            extra.vertical_align_percent,
            "vertical_align",
            NativeLengthValueRange::All,
        );
    }
    if extra.vertical_align_length.is_some() && extra.vertical_align_percent.is_some() {
        Invalid("vertical_align");
    }
    if let Some(value) = extra.vertical_align_length {
        return Pixels(value, "vertical_align", false);
    }
    if let Some(value) = extra.vertical_align_percent {
        return Length::Percent(Number(value, "vertical_align", false));
    }
    Invalid("vertical_align length")
}

// cpp: layoutng/internal/boundary/native_input.cc:280-289
#[allow(non_snake_case)]
fn TextIndentLength(extra: &ExtendedStyle) -> Length {
    if extra.text_indent_calculated {
        return CalculatedLength(
            Some(extra.text_indent),
            extra.text_indent_percent,
            "text_indent",
            NativeLengthValueRange::All,
        );
    }
    if let Some(value) = extra.text_indent_percent {
        return Length::Percent(Number(value, "text_indent", false));
    }
    Pixels(extra.text_indent, "text_indent", false)
}

// cpp: layoutng/internal/boundary/native_input.cc:291-307
#[allow(non_snake_case)]
fn GapLength(
    pixels: Option<f64>,
    percent: Option<f64>,
    calculated: bool,
    fallback: f64,
    field: &str,
) -> Option<Length> {
    if calculated {
        return Some(CalculatedLength(
            pixels,
            percent,
            field,
            NativeLengthValueRange::NonNegative,
        ));
    }
    if pixels.is_some() && percent.is_some() {
        Invalid(field);
    }
    if let Some(value) = percent {
        return Some(Length::Percent(Number(value, field, true)));
    }
    if let Some(value) = pixels {
        return Some(Pixels(value, field, true));
    }
    // Zero in the compact compatibility input represents CSS normal.
    if fallback == 0.0 {
        return None;
    }
    Some(Pixels(fallback, field, true))
}

// cpp: layoutng/internal/boundary/native_input.cc:309-323
fn ConvertBorderLineStyle(value: BorderLineStyle) -> EBorderStyle {
    match value {
        BorderLineStyle::kNone => EBorderStyle::kNone,
        BorderLineStyle::kHidden => EBorderStyle::kHidden,
        BorderLineStyle::kSolid => EBorderStyle::kSolid,
        BorderLineStyle::kDashed => EBorderStyle::kDashed,
        BorderLineStyle::kDotted => EBorderStyle::kDotted,
        BorderLineStyle::kDouble => EBorderStyle::kDouble,
        BorderLineStyle::kGroove => EBorderStyle::kGroove,
        BorderLineStyle::kRidge => EBorderStyle::kRidge,
        BorderLineStyle::kInset => EBorderStyle::kInset,
        BorderLineStyle::kOutset => EBorderStyle::kOutset,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:325-356
fn ConvertDisplay(value: Display) -> EDisplay {
    match value {
        Display::kTableSection => EDisplay::kTableRowGroup,
        Display::kTableHeaderGroup => EDisplay::kTableHeaderGroup,
        Display::kTableFooterGroup => EDisplay::kTableFooterGroup,
        Display::kCustom => EDisplay::kLayoutCustom,
        Display::kBlock => EDisplay::kBlock,
        Display::kFlex => EDisplay::kFlex,
        Display::kInlineFlex => EDisplay::kInlineFlex,
        Display::kGrid => EDisplay::kGrid,
        Display::kInlineGrid => EDisplay::kInlineGrid,
        Display::kInline => EDisplay::kInline,
        Display::kTable => EDisplay::kTable,
        Display::kInlineTable => EDisplay::kInlineTable,
        Display::kTableRow => EDisplay::kTableRow,
        Display::kTableCell => EDisplay::kTableCell,
        Display::kFlowRoot => EDisplay::kFlowRoot,
        Display::kInlineBlock => EDisplay::kInlineBlock,
        Display::kTableCaption => EDisplay::kTableCaption,
        Display::kTableColumnGroup => EDisplay::kTableColumnGroup,
        Display::kTableColumn => EDisplay::kTableColumn,
        Display::kGridLanes => EDisplay::kGridLanes,
        Display::kListItem => EDisplay::kListItem,
        Display::kMath => EDisplay::kMath,
        Display::kBlockMath => EDisplay::kBlockMath,
        Display::kRuby => EDisplay::kRuby,
        Display::kBlockRuby => EDisplay::kBlockRuby,
        Display::kRubyText => EDisplay::kRubyText,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:358-366
fn ConvertFlexDirection(value: FlexDirection) -> EFlexDirection {
    match value {
        FlexDirection::kRow => EFlexDirection::kRow,
        FlexDirection::kColumn => EFlexDirection::kColumn,
        FlexDirection::kRowReverse => EFlexDirection::kRowReverse,
        FlexDirection::kColumnReverse => EFlexDirection::kColumnReverse,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:368-383
fn ConvertAlignItems(value: AlignItems) -> ItemPosition {
    match value {
        AlignItems::kNormal => ItemPosition::kNormal,
        AlignItems::kStretch => ItemPosition::kStretch,
        AlignItems::kBaseline => ItemPosition::kBaseline,
        AlignItems::kLastBaseline => ItemPosition::kLastBaseline,
        AlignItems::kCenter => ItemPosition::kCenter,
        AlignItems::kStart => ItemPosition::kStart,
        AlignItems::kEnd => ItemPosition::kEnd,
        AlignItems::kSelfStart => ItemPosition::kSelfStart,
        AlignItems::kSelfEnd => ItemPosition::kSelfEnd,
        AlignItems::kFlexStart => ItemPosition::kFlexStart,
        AlignItems::kFlexEnd => ItemPosition::kFlexEnd,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:385-395
fn ConvertOverflowAlignment(value: OverflowAlignment) -> NativeOverflowAlignment {
    match value {
        OverflowAlignment::kDefault => NativeOverflowAlignment::kDefault,
        OverflowAlignment::kUnsafe => NativeOverflowAlignment::kUnsafe,
        OverflowAlignment::kSafe => NativeOverflowAlignment::kSafe,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:397-406
fn ConvertPosition(value: Position) -> EPosition {
    match value {
        Position::kStatic => EPosition::kStatic,
        Position::kAbsolute => EPosition::kAbsolute,
        Position::kRelative => EPosition::kRelative,
        Position::kFixed => EPosition::kFixed,
        Position::kSticky => EPosition::kSticky,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:408-417
fn ConvertFloatSide(value: FloatSide) -> EFloat {
    match value {
        FloatSide::kNone => EFloat::kNone,
        FloatSide::kLeft => EFloat::kLeft,
        FloatSide::kRight => EFloat::kRight,
        FloatSide::kInlineStart => EFloat::kInlineStart,
        FloatSide::kInlineEnd => EFloat::kInlineEnd,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:419-429
fn ConvertClearSide(value: ClearSide) -> EClear {
    match value {
        ClearSide::kNone => EClear::kNone,
        ClearSide::kLeft => EClear::kLeft,
        ClearSide::kRight => EClear::kRight,
        ClearSide::kInlineStart => EClear::kInlineStart,
        ClearSide::kInlineEnd => EClear::kInlineEnd,
        ClearSide::kBoth => EClear::kBoth,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:431-440
fn ConvertOverflow(value: Overflow) -> EOverflow {
    match value {
        Overflow::kVisible => EOverflow::kVisible,
        Overflow::kHidden => EOverflow::kHidden,
        Overflow::kScroll => EOverflow::kScroll,
        Overflow::kAuto => EOverflow::kAuto,
        Overflow::kClip => EOverflow::kClip,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:442-453
fn ConvertWhiteSpace(value: WhiteSpace) -> EWhiteSpace {
    match value {
        WhiteSpace::kNormal => EWhiteSpace::kNormal,
        WhiteSpace::kNowrap => EWhiteSpace::kNowrap,
        WhiteSpace::kPre => EWhiteSpace::kPre,
        WhiteSpace::kPreLine => EWhiteSpace::kPreLine,
        WhiteSpace::kPreWrap => EWhiteSpace::kPreWrap,
        WhiteSpace::kBreakSpaces => EWhiteSpace::kBreakSpaces,
        WhiteSpace::kPreserveBreaksNowrap => EWhiteSpace::from_bits(2),
        WhiteSpace::kBreakSpacesNowrap => EWhiteSpace::from_bits(3),
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:454-469
fn ConvertTextAlign(value: TextAlign) -> ETextAlign {
    match value {
        TextAlign::kStart => ETextAlign::kStart,
        TextAlign::kEnd => ETextAlign::kEnd,
        TextAlign::kLeft => ETextAlign::kLeft,
        TextAlign::kRight => ETextAlign::kRight,
        TextAlign::kCenter => ETextAlign::kCenter,
        TextAlign::kJustify => ETextAlign::kJustify,
        TextAlign::kMatchParent => ETextAlign::kMatchParent,
        TextAlign::kWebkitLeft => ETextAlign::kWebkitLeft,
        TextAlign::kWebkitRight => ETextAlign::kWebkitRight,
        TextAlign::kWebkitCenter => ETextAlign::kWebkitCenter,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:470-483
fn ConvertTextAlignLast(value: TextAlignLast) -> ETextAlignLast {
    match value {
        TextAlignLast::kAuto => ETextAlignLast::kAuto,
        TextAlignLast::kStart => ETextAlignLast::kStart,
        TextAlignLast::kEnd => ETextAlignLast::kEnd,
        TextAlignLast::kLeft => ETextAlignLast::kLeft,
        TextAlignLast::kRight => ETextAlignLast::kRight,
        TextAlignLast::kCenter => ETextAlignLast::kCenter,
        TextAlignLast::kJustify => ETextAlignLast::kJustify,
        TextAlignLast::kMatchParent => ETextAlignLast::kMatchParent,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:485-499
fn ConvertVerticalAlign(value: VerticalAlign) -> EVerticalAlign {
    match value {
        VerticalAlign::kBaseline => EVerticalAlign::kBaseline,
        VerticalAlign::kMiddle => EVerticalAlign::kMiddle,
        VerticalAlign::kSub => EVerticalAlign::kSub,
        VerticalAlign::kSuper => EVerticalAlign::kSuper,
        VerticalAlign::kTextTop => EVerticalAlign::kTextTop,
        VerticalAlign::kTextBottom => EVerticalAlign::kTextBottom,
        VerticalAlign::kTop => EVerticalAlign::kTop,
        VerticalAlign::kBottom => EVerticalAlign::kBottom,
        VerticalAlign::kBaselineMiddle => EVerticalAlign::kBaselineMiddle,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:501-509
fn ConvertOverflowWrap(value: OverflowWrap) -> EOverflowWrap {
    match value {
        OverflowWrap::kNormal => EOverflowWrap::kNormal,
        OverflowWrap::kBreakWord => EOverflowWrap::kBreakWord,
        OverflowWrap::kAnywhere => EOverflowWrap::kAnywhere,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:510-520
fn ConvertWordBreak(value: WordBreak) -> EWordBreak {
    match value {
        WordBreak::kNormal => EWordBreak::kNormal,
        WordBreak::kBreakAll => EWordBreak::kBreakAll,
        WordBreak::kKeepAll => EWordBreak::kKeepAll,
        WordBreak::kAutoPhrase => EWordBreak::kAutoPhrase,
        WordBreak::kBreakWord => EWordBreak::kBreakWord,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:521-531
fn ConvertLineBreak(value: LineBreak) -> NativeLineBreak {
    match value {
        LineBreak::kAuto => NativeLineBreak::kAuto,
        LineBreak::kLoose => NativeLineBreak::kLoose,
        LineBreak::kNormal => NativeLineBreak::kNormal,
        LineBreak::kStrict => NativeLineBreak::kStrict,
        LineBreak::kAnywhere => NativeLineBreak::kAnywhere,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:532-540
fn ConvertHyphens(value: Hyphens) -> NativeHyphens {
    match value {
        Hyphens::kNone => NativeHyphens::kNone,
        Hyphens::kManual => NativeHyphens::kManual,
        Hyphens::kAuto => NativeHyphens::kAuto,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:541-549
fn ConvertTextOrientation(value: TextOrientation) -> ETextOrientation {
    match value {
        TextOrientation::kMixed => ETextOrientation::kMixed,
        TextOrientation::kUpright => ETextOrientation::kUpright,
        TextOrientation::kSideways => ETextOrientation::kSideways,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:550-557
fn ConvertTextCombine(value: TextCombine) -> ETextCombine {
    match value {
        TextCombine::kNone => ETextCombine::kNone,
        TextCombine::kAll => ETextCombine::kAll,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:558-564
fn ConvertTextTransform(value: TextTransform) -> ETextTransform {
    if value.0 & !63 != 0 {
        Invalid("text_transform");
    }
    unsafe { FoundationTextTransformFromBits(value.0) }
}

// cpp: layoutng/internal/boundary/native_input.cc:565-572
fn ConvertRubyPosition(value: RubyPosition) -> NativeRubyPosition {
    match value {
        RubyPosition::kOver => NativeRubyPosition::kOver,
        RubyPosition::kUnder => NativeRubyPosition::kUnder,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:573-582
fn ConvertRubyAlign(value: RubyAlign) -> ERubyAlign {
    match value {
        RubyAlign::kCenter => ERubyAlign::kCenter,
        RubyAlign::kStart => ERubyAlign::kStart,
        RubyAlign::kSpaceBetween => ERubyAlign::kSpaceBetween,
        RubyAlign::kSpaceAround => ERubyAlign::kSpaceAround,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:583-591
fn ConvertRubyOverhang(value: RubyOverhang) -> ERubyOverhang {
    match value {
        RubyOverhang::kNone => ERubyOverhang::kNone,
        RubyOverhang::kAuto => ERubyOverhang::kAuto,
        RubyOverhang::kSpaces => ERubyOverhang::kSpaces,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:592-599
fn ConvertEmptyCells(value: EmptyCells) -> EEmptyCells {
    match value {
        EmptyCells::kHide => EEmptyCells::kHide,
        EmptyCells::kShow => EEmptyCells::kShow,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:600-611
fn ConvertMarginTrim(value: MarginTrim) -> u32 {
    let bits = value.0;
    let mut output = EMarginTrim::kMarginTrimNone.value() as u32;
    if bits & MarginTrim::kBlockStart.0 != 0 {
        output |= EMarginTrim::kMarginTrimBlockStart.value() as u32;
    }
    if bits & MarginTrim::kBlockEnd.0 != 0 {
        output |= EMarginTrim::kMarginTrimBlockEnd.value() as u32;
    }
    if bits & !MarginTrim::kBlock.0 != 0 {
        Invalid("margin_trim");
    }
    output
}

// cpp: layoutng/internal/boundary/native_input.cc:612-621
fn ConvertTextWrapStyle(value: TextWrapStyle) -> NativeTextWrapStyle {
    match value {
        TextWrapStyle::kAuto => NativeTextWrapStyle::kAuto,
        TextWrapStyle::kPretty => NativeTextWrapStyle::kPretty,
        TextWrapStyle::kBalance => NativeTextWrapStyle::kBalance,
        TextWrapStyle::kStable => NativeTextWrapStyle::kStable,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:622-634
fn ConvertUnicodeBidi(value: UnicodeBidi) -> NativeUnicodeBidi {
    match value {
        UnicodeBidi::kNormal => NativeUnicodeBidi::kNormal,
        UnicodeBidi::kEmbed => NativeUnicodeBidi::kEmbed,
        UnicodeBidi::kIsolate => NativeUnicodeBidi::kIsolate,
        UnicodeBidi::kOverride => NativeUnicodeBidi::kBidiOverride,
        UnicodeBidi::kIsolateOverride => NativeUnicodeBidi::kIsolateOverride,
        UnicodeBidi::kPlaintext => NativeUnicodeBidi::kPlaintext,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:635-653
fn ConvertJustifyContent(value: JustifyContent) -> StyleContentAlignmentData {
    use ContentDistributionType as D;
    use ContentPosition as P;
    let (position, distribution) = match value {
        JustifyContent::kNormal => (P::kNormal, D::kDefault),
        JustifyContent::kStart => (P::kStart, D::kDefault),
        JustifyContent::kEnd => (P::kEnd, D::kDefault),
        JustifyContent::kFlexStart => (P::kFlexStart, D::kDefault),
        JustifyContent::kFlexEnd => (P::kFlexEnd, D::kDefault),
        JustifyContent::kCenter => (P::kCenter, D::kDefault),
        JustifyContent::kLeft => (P::kLeft, D::kDefault),
        JustifyContent::kRight => (P::kRight, D::kDefault),
        JustifyContent::kSpaceBetween => (P::kNormal, D::kSpaceBetween),
        JustifyContent::kSpaceAround => (P::kNormal, D::kSpaceAround),
        JustifyContent::kSpaceEvenly => (P::kNormal, D::kSpaceEvenly),
    };
    StyleContentAlignmentData::new_default_overflow(position, distribution)
}

// cpp: layoutng/internal/boundary/native_input.cc:654-679
fn ConvertContentAlignment(value: ContentAlignment) -> StyleContentAlignmentData {
    use ContentDistributionType as D;
    use ContentPosition as P;
    let (position, distribution) = match value {
        ContentAlignment::kNormal => (P::kNormal, D::kDefault),
        ContentAlignment::kBaseline => (P::kBaseline, D::kDefault),
        ContentAlignment::kLastBaseline => (P::kLastBaseline, D::kDefault),
        ContentAlignment::kStart => (P::kStart, D::kDefault),
        ContentAlignment::kEnd => (P::kEnd, D::kDefault),
        ContentAlignment::kFlexStart => (P::kFlexStart, D::kDefault),
        ContentAlignment::kFlexEnd => (P::kFlexEnd, D::kDefault),
        ContentAlignment::kCenter => (P::kCenter, D::kDefault),
        ContentAlignment::kLeft => (P::kLeft, D::kDefault),
        ContentAlignment::kRight => (P::kRight, D::kDefault),
        ContentAlignment::kSpaceBetween => (P::kNormal, D::kSpaceBetween),
        ContentAlignment::kSpaceAround => (P::kNormal, D::kSpaceAround),
        ContentAlignment::kSpaceEvenly => (P::kNormal, D::kSpaceEvenly),
        ContentAlignment::kStretch => (P::kNormal, D::kStretch),
    };
    StyleContentAlignmentData::new_default_overflow(position, distribution)
}

// cpp: layoutng/internal/boundary/native_input.cc:680-701
fn ConvertSelfAlignment(value: SelfAlignment) -> StyleSelfAlignmentData {
    let position = match value {
        SelfAlignment::kAuto => ItemPosition::kAuto,
        SelfAlignment::kNormal => ItemPosition::kNormal,
        SelfAlignment::kStretch => ItemPosition::kStretch,
        SelfAlignment::kBaseline => ItemPosition::kBaseline,
        SelfAlignment::kLastBaseline => ItemPosition::kLastBaseline,
        SelfAlignment::kCenter => ItemPosition::kCenter,
        SelfAlignment::kStart => ItemPosition::kStart,
        SelfAlignment::kEnd => ItemPosition::kEnd,
        SelfAlignment::kSelfStart => ItemPosition::kSelfStart,
        SelfAlignment::kSelfEnd => ItemPosition::kSelfEnd,
        SelfAlignment::kFlexStart => ItemPosition::kFlexStart,
        SelfAlignment::kFlexEnd => ItemPosition::kFlexEnd,
        SelfAlignment::kLeft => ItemPosition::kLeft,
        SelfAlignment::kRight => ItemPosition::kRight,
    };
    StyleSelfAlignmentData::new_nonlegacy(position, NativeOverflowAlignment::kDefault)
}

// cpp: layoutng/internal/boundary/native_input.cc:702-711
fn ConvertBreakRule(value: BreakRule, context: FragmentationType) -> EBreakBetween {
    match value {
        BreakRule::kAuto => EBreakBetween::kAuto,
        BreakRule::kAvoid => EBreakBetween::kAvoid,
        BreakRule::kAlways => {
            if context == FragmentationType::kFragmentColumn {
                EBreakBetween::kColumn
            } else {
                EBreakBetween::kPage
            }
        }
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:713-722
fn AutoMargin(builder: &mut ComputedStyleBuilder, side: PhysicalDirection) {
    match side {
        PhysicalDirection::kUp => builder.SetMarginTop(&Length::Auto()),
        PhysicalDirection::kRight => builder.SetMarginRight(&Length::Auto()),
        PhysicalDirection::kDown => builder.SetMarginBottom(&Length::Auto()),
        PhysicalDirection::kLeft => builder.SetMarginLeft(&Length::Auto()),
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:723-748
fn TrackBreadth(breadth: &GridTrackBreadth, allow_flex: bool, field: &str) -> Length {
    match breadth.kind {
        GridTrackBreadthKind::kAuto => Length::Auto().clone(),
        GridTrackBreadthKind::kFixed => Pixels(breadth.pixels, field, true),
        GridTrackBreadthKind::kPercentage => {
            Length::Percent(Number(breadth.percentage, field, true))
        }
        GridTrackBreadthKind::kCalculated => CalculatedLength(
            Some(breadth.pixels),
            Some(breadth.percentage),
            field,
            NativeLengthValueRange::NonNegative,
        ),
        GridTrackBreadthKind::kMinContent => Length::MinContent().clone(),
        GridTrackBreadthKind::kMaxContent => Length::MaxContent().clone(),
        GridTrackBreadthKind::kFlex => {
            if !allow_flex || !breadth.pixels.is_finite() || breadth.pixels <= 0.0 {
                Invalid(field);
            }
            Length::Flex(breadth.pixels as f32)
        }
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:749-763
fn TrackSize(track: &GridTrack) -> GridTrackSize {
    let minimum = TrackBreadth(&track.minimum, false, "grid minimum");
    let maximum = TrackBreadth(&track.maximum, true, "grid maximum");
    if track.fit_content {
        if !minimum.IsAuto() || maximum.IsFlex() || maximum.HasAutoOrContentOrIntrinsic() {
            Invalid("grid fit-content");
        }
        return GridTrackSize::new(&maximum, GridTrackSizeType::kFitContentTrackSizing);
    }
    if minimum == maximum {
        return GridTrackSize::from_length(&minimum);
    }
    GridTrackSize::new_minmax(&minimum, &maximum)
}

// cpp: layoutng/internal/boundary/native_input.cc:764-777
fn RepeatType(kind: GridRepeatKind) -> GridTrackRepeatType {
    match kind {
        GridRepeatKind::kNone => GridTrackRepeatType::kNoRepeat,
        GridRepeatKind::kInteger => GridTrackRepeatType::kInteger,
        GridRepeatKind::kAutoFill => GridTrackRepeatType::kAutoFill,
        GridRepeatKind::kAutoFit => GridTrackRepeatType::kAutoFit,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:778-786
fn AddNamedLine(map: &mut NamedGridLinesMap, input: &str, position: u32) {
    if input.is_empty() {
        Invalid("grid line name");
    }
    let name = BlinkString::FromUtf8(input.as_bytes());
    map.entry(name).or_default().push(position);
}

// cpp: layoutng/internal/boundary/native_input.cc:787-791
fn SortNamedLines(map: &mut NamedGridLinesMap) {
    for value in map.values_mut() {
        value.sort();
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:792-867
fn Tracks(tracks: &GridTrackListInput, fixed: &[f64], subgrid: bool) -> *mut ComputedGridTrackList {
    if (!tracks.empty() && !fixed.is_empty()) || (subgrid && (!tracks.empty() || !fixed.is_empty()))
    {
        Invalid("conflicting grid track definitions");
    }
    let result_ptr = MakeGarbageCollected(ComputedGridTrackList::default());
    let result = unsafe { &mut *result_ptr };
    if subgrid {
        result.SetGridAxisType(GridAxisType::kSubgriddedAxis);
        result
            .GetMutableTrackList()
            .SetAxisType(GridAxisType::kSubgriddedAxis);
        return result_ptr;
    }
    for line in &tracks.line_names {
        AddNamedLine(result.GetMutableNamedGridLines(), &line.name, line.position);
    }
    let mut conceptual_line = 0u32;
    for repeater in &tracks.repeaters {
        if repeater.tracks.is_empty()
            || repeater.count == 0
            || (repeater.kind != GridRepeatKind::kInteger && repeater.count != 1)
        {
            Invalid("grid repeater");
        }
        let mut sizes = Vector::<GridTrackSize>::default();
        for track in &repeater.tracks {
            sizes.push(TrackSize(track));
        }
        if !repeater.line_names.is_empty() && repeater.line_names.len() != repeater.tracks.len() + 1
        {
            Invalid("grid repeater line names");
        }
        if !repeater.line_names.is_empty() {
            if repeater.kind == GridRepeatKind::kAutoFill
                || repeater.kind == GridRepeatKind::kAutoFit
            {
                for (line, names) in repeater.line_names.iter().enumerate() {
                    for name in names {
                        AddNamedLine(
                            result.GetMutableAutoRepeatNamedGridLines(),
                            name,
                            line as u32,
                        );
                    }
                }
            } else {
                let repetitions = if repeater.kind == GridRepeatKind::kInteger {
                    repeater.count
                } else {
                    1
                };
                for repetition in 0..repetitions {
                    for (line, names) in repeater.line_names.iter().enumerate() {
                        for name in names {
                            let position = conceptual_line
                                .wrapping_add(repetition.wrapping_mul(repeater.tracks.len() as u32))
                                .wrapping_add(line as u32);
                            AddNamedLine(result.GetMutableNamedGridLines(), name, position);
                        }
                    }
                }
            }
        }
        if !result.GetMutableTrackList().AddRepeaterWithCount(
            &sizes,
            RepeatType(repeater.kind),
            repeater.count,
        ) {
            Invalid("grid track count");
        }
        if repeater.kind == GridRepeatKind::kAutoFill {
            result.SetAutoRepeatType(AutoRepeatType::kAutoFill);
        } else if repeater.kind == GridRepeatKind::kAutoFit {
            result.SetAutoRepeatType(AutoRepeatType::kAutoFit);
        }
        conceptual_line = conceptual_line.wrapping_add(
            if repeater.kind == GridRepeatKind::kAutoFill
                || repeater.kind == GridRepeatKind::kAutoFit
            {
                1
            } else {
                (repeater.tracks.len() as u32).wrapping_mul(repeater.count)
            },
        );
    }
    for line in &tracks.line_names {
        if line.position > conceptual_line {
            Invalid("grid line position");
        }
    }
    SortNamedLines(result.GetMutableNamedGridLines());
    SortNamedLines(result.GetMutableAutoRepeatNamedGridLines());
    if !fixed.is_empty() {
        let mut sizes = Vector::<GridTrackSize>::default();
        for value in fixed {
            sizes.push(GridTrackSize::from_length(&Pixels(
                *value,
                "grid track",
                true,
            )));
        }
        if !result.GetMutableTrackList().AddRepeaterDefault(&sizes) {
            Invalid("grid track count");
        }
    }
    if result.GetTrackList().HasAutoRepeater() {
        result.SetAutoRepeatInsertionPoint(result.GetTrackList().TrackCountBeforeAutoRepeat());
    }
    result_ptr
}

// cpp: layoutng/internal/boundary/native_input.cc:868-881
fn AutoTracks(tracks: &GridTrackListInput) -> GridTrackList {
    let mut result = GridTrackList::default();
    if tracks.repeaters.len() != 1
        || tracks.repeaters[0].kind != GridRepeatKind::kNone
        || tracks.repeaters[0].count != 1
    {
        Invalid("grid auto tracks cannot contain repeaters");
    }
    let mut sizes = Vector::<GridTrackSize>::default();
    for track in &tracks.repeaters[0].tracks {
        sizes.push(TrackSize(track));
    }
    if !sizes.is_empty() && !result.AddRepeaterDefault(&sizes) {
        Invalid("grid auto track count");
    }
    result
}

// cpp: layoutng/internal/boundary/native_input.cc:882-908
fn GridPosition(input: &GridLine, field: &str) -> NativeGridPosition {
    let mut result = NativeGridPosition::default();
    // Rust's public String input is already valid UTF-8. Preserve the C++
    // empty-name and number guards before creating the atomic name.
    let name = if input.name.is_empty() {
        AtomicString::default()
    } else {
        AtomicString::from_utf16(&input.name.encode_utf16().collect::<Vec<_>>())
    };
    match input.kind {
        GridLineKind::kAuto => {
            if input.number != 0 || !input.name.is_empty() {
                Invalid(field);
            }
        }
        GridLineKind::kExplicit => {
            if input.number == 0 {
                Invalid(field);
            }
            result.SetExplicitPosition(input.number, &name);
        }
        GridLineKind::kSpan => {
            if input.number < 1 {
                Invalid(field);
            }
            result.SetSpanPosition(input.number, &name);
        }
        GridLineKind::kNamedArea => {
            if input.number != 0 || input.name.is_empty() {
                Invalid(field);
            }
            result.SetNamedGridArea(&name);
        }
    }
    result
}

// cpp: layoutng/internal/boundary/native_input.cc:909-936
fn GridTemplateAreas(input: &GridTemplateAreasInput) -> *mut ComputedGridTemplateAreas {
    if input.row_count == 0
        || input.column_count == 0
        || input.row_count > i32::MAX as u32
        || input.column_count > i32::MAX as u32
    {
        Invalid("grid template area dimensions");
    }
    let mut areas = NamedGridAreaMap::default();
    for area in &input.areas {
        if area.name.is_empty()
            || area.row_start >= area.row_end
            || area.column_start >= area.column_end
            || area.row_end > input.row_count
            || area.column_end > input.column_count
        {
            Invalid("grid template area");
        }
        let name = BlinkString::FromUtf8(area.name.as_bytes());
        if areas.contains_key(&name) {
            Invalid("duplicate grid template area");
        }
        let rows =
            GridSpan::UntranslatedDefiniteGridSpan(area.row_start as i32, area.row_end as i32);
        let columns = GridSpan::UntranslatedDefiniteGridSpan(
            area.column_start as i32,
            area.column_end as i32,
        );
        areas.insert(name, GridArea::new(&rows, &columns));
    }
    MakeGarbageCollected(ComputedGridTemplateAreas::new(
        &areas,
        input.row_count,
        input.column_count,
    ))
}

// cpp: layoutng/internal/boundary/native_input.cc:947-1051
fn BeginNativeStyle(input: &ComputedStyle, extra: &ExtendedStyle) -> ComputedStyleBuilder {
    let initial = unsafe { &*NativeComputedStyle::GetInitialStyleSingleton() };
    let mut builder = ComputedStyleBuilder::from_style(initial);
    let direction = WritingDirectionMode::new(
        ConvertWritingMode(input.writing_mode),
        ConvertTextDirection(input.direction),
    );
    builder.SetWritingMode(direction.GetWritingMode());
    builder.SetDirection(direction.Direction());
    builder.SetObjectFit(ConvertObjectFit(input.paint.object_fit));
    let object_position_x = PositionLength(
        input.paint.object_position.x,
        input.paint.object_position_offset.x,
        "object_position.x",
    );
    let object_position_y = PositionLength(
        input.paint.object_position.y,
        input.paint.object_position_offset.y,
        "object_position.y",
    );
    builder.SetObjectPosition(&LengthPoint::new(&object_position_x, &object_position_y));
    builder.SetMathDepth(extra.math_depth as i16);
    builder.SetMathStyle(if extra.math_style == MathStyle::kCompact {
        EMathStyle::kCompact
    } else {
        EMathStyle::kNormal
    });
    builder.SetMathShift(if extra.math_shift == MathShift::kCompact {
        EMathShift::kCompact
    } else {
        EMathShift::kNormal
    });
    if let Some(value) = extra.math_baseline {
        builder.SetMathBaseline(&Length::Fixed(value));
    }
    if let Some(value) = extra.math_padded_depth {
        builder.SetMathPaddedDepth(&Length::Fixed(value));
    }
    if let Some(value) = extra.math_lspace {
        builder.SetMathLSpace(&Length::Fixed(value));
    }
    if let Some(value) = extra.math_padded_voffset {
        builder.SetMathPaddedVOffset(&Length::Fixed(value));
    }
    if let Some(value) = extra.math_fraction_bar_thickness {
        builder.SetMathFractionBarThickness(&Length::Fixed(value));
    }
    if let Some(value) = extra.math_rspace {
        builder.SetMathRSpace(&Length::Fixed(value));
    }
    if let Some(value) = extra.math_min_size {
        builder.SetMathMinSize(&Length::Fixed(value));
    }
    if let Some(value) = extra.math_max_size {
        builder.SetMathMaxSize(&Length::Fixed(value));
    }
    if extra.has_author_background {
        builder.SetHasAuthorBackground();
    }
    if extra.has_author_border {
        builder.SetHasAuthorBorder();
    }
    if extra.has_author_border_radius {
        builder.SetHasAuthorBorderRadius();
    }
    if extra.effective_appearance == AppearanceValue::kCheckbox
        || extra.effective_appearance == AppearanceValue::kRadio
    {
        builder.SetShouldIgnoreOverflowPropertyForInlineBlockBaseline();
        builder.SetInlineBlockBaselineEdge(EInlineBlockBaselineEdge::kBorderBox);
    }
    if !matches!(
        extra.effective_appearance,
        AppearanceValue::kNone
            | AppearanceValue::kCheckbox
            | AppearanceValue::kRadio
            | AppearanceValue::kButton
            | AppearanceValue::kListbox
            | AppearanceValue::kMediaControl
            | AppearanceValue::kMenulist
            | AppearanceValue::kMenulistButton
            | AppearanceValue::kMeter
            | AppearanceValue::kProgressBar
            | AppearanceValue::kSearchField
            | AppearanceValue::kTextField
            | AppearanceValue::kTextArea
            | AppearanceValue::kInnerSpinButton
            | AppearanceValue::kMediaSlider
            | AppearanceValue::kMediaSliderThumb
            | AppearanceValue::kMediaVolumeSlider
            | AppearanceValue::kMediaVolumeSliderThumb
            | AppearanceValue::kPushButton
            | AppearanceValue::kSquareButton
            | AppearanceValue::kSliderHorizontal
            | AppearanceValue::kSliderThumbHorizontal
            | AppearanceValue::kSliderThumbVertical
            | AppearanceValue::kSearchFieldCancelButton
            | AppearanceValue::kSliderVertical
            | AppearanceValue::kBaseSelect
            | AppearanceValue::kBase
    ) {
        Invalid("effective_appearance must be resolved");
    }
    builder.SetEffectiveAppearance(extra.effective_appearance);
    if !extra.effective_zoom.is_finite() || extra.effective_zoom <= 0.0 {
        Invalid("effective_zoom");
    }
    builder.SetEffectiveZoom(extra.effective_zoom);
    builder.SetZoom(extra.zoom);
    builder.SetFieldSizing(match extra.field_sizing {
        FieldSizing::kFixed => EFieldSizing::kFixed,
        FieldSizing::kContent => EFieldSizing::kContent,
    });
    builder.SetBoxSizing(match extra.box_sizing {
        BoxSizing::kContentBox => EBoxSizing::kContentBox,
        BoxSizing::kBorderBox => EBoxSizing::kBorderBox,
    });
    builder.SetDisplay(ConvertDisplay(input.display));
    let is_inline = builder.IsDisplayInlineType();
    builder.SetIsOriginalDisplayInlineType(is_inline);
    builder
}

// cpp: layoutng/internal/boundary/native_input.cc:1052-1078
fn ApplyNativeStyleListAndPosition(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
    extra: &ExtendedStyle,
) {
    let list_style_name = match extra.list_style_type {
        ListStyleType::kDisc => Some(&foundation::keywords::kDisc),
        ListStyleType::kCircle => Some(&foundation::keywords::kCircle),
        ListStyleType::kSquare => Some(&foundation::keywords::kSquare),
        ListStyleType::kDecimal => Some(&foundation::keywords::kDecimal),
        ListStyleType::kLowerAlpha => Some(&foundation::keywords::kLowerAlpha),
        ListStyleType::kNone => None,
    };
    let list_style_type = match list_style_name {
        Some(name) => Member::from_ptr(ListStyleTypeData::CreateCounterStyle(
            name,
            std::ptr::null(),
        )),
        None => Member::default(),
    };
    builder.SetListStyleTypeOwned(list_style_type);
    builder.SetListStylePosition(match extra.list_style_position {
        ListStylePosition::kOutside => EListStylePosition::kOutside,
        ListStylePosition::kInside => EListStylePosition::kInside,
    });
    builder.SetPosition(ConvertPosition(input.position));
    builder.SetFloating(ConvertFloatSide(input.floating));
}

// cpp: layoutng/internal/boundary/native_input.cc:1079-1135
fn ApplyNativeStyleShape(builder: &mut ComputedStyleBuilder, extra: &ExtendedStyle) {
    if let Some(shape) = &extra.shape_outside {
        let validate_coordinate = |coordinate: &ShapeCoordinate, field: &str| {
            Number(coordinate.pixels, field, false);
            Number(coordinate.percentage, field, false);
        };
        let validate_point = |point: &ShapePoint, field: &str| {
            validate_coordinate(&point.x, field);
            validate_coordinate(&point.y, field);
        };
        match shape.kind {
            ShapeOutsideKind::kReferenceBox => {
                if !shape.contour.is_empty() {
                    Invalid("shape_outside reference contour");
                }
            }
            ShapeOutsideKind::kPolygon => {
                if shape.contour.len() < 3 {
                    Invalid("shape_outside polygon");
                }
            }
            ShapeOutsideKind::kCircle | ShapeOutsideKind::kEllipse | ShapeOutsideKind::kInset => {
                if !shape.contour.is_empty() {
                    Invalid("shape_outside non-polygon contour");
                }
            }
            ShapeOutsideKind::kImage => {
                if shape.resource_id == 0 || !shape.contour.is_empty() {
                    Invalid("shape_outside image resource");
                }
            }
        }
        for point in &shape.contour {
            validate_point(point, "shape_outside contour");
        }
        validate_point(&shape.center, "shape_outside center");
        for radius in [&shape.radius_x, &shape.radius_y] {
            match radius.kind {
                ShapeRadiusKind::kLengthPercentage
                | ShapeRadiusKind::kClosestSide
                | ShapeRadiusKind::kFarthestSide
                | ShapeRadiusKind::kClosestCorner
                | ShapeRadiusKind::kFarthestCorner => {}
            }
            validate_coordinate(&radius.value, "shape_outside radius");
        }
        for inset in &shape.insets {
            validate_coordinate(inset, "shape_outside inset");
        }
        for radius in &shape.corner_radii {
            validate_point(radius, "shape_outside corner radius");
        }
        builder.SetShapeOutsideOwned(Member::from_ptr(MakeGarbageCollected(
            ShapeValue::from_box(ConvertShapeReferenceBox(shape.reference_box)),
        )));
    }
    builder.SetShapeMargin(&ShapeMarginLength(&extra.shape_margin));
    builder.SetShapeImageThreshold(Number(
        extra.shape_image_threshold,
        "shape_image_threshold",
        false,
    ) as f32);
}

// cpp: layoutng/internal/boundary/native_input.cc:1136-1201
fn ApplyNativeStyleBoxEdges(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
    extra: &ExtendedStyle,
) {
    builder.SetTopOwned(InsetLength(
        input.top,
        extra.inset_percentages[0],
        extra.inset_calculated[0],
        "top",
    ));
    builder.SetRightOwned(InsetLength(
        input.right,
        extra.inset_percentages[1],
        extra.inset_calculated[1],
        "right",
    ));
    builder.SetBottomOwned(InsetLength(
        input.bottom,
        extra.inset_percentages[2],
        extra.inset_calculated[2],
        "bottom",
    ));
    builder.SetLeftOwned(InsetLength(
        input.left,
        extra.inset_percentages[3],
        extra.inset_calculated[3],
        "left",
    ));

    builder.SetWidthOwned(IntrinsicDimension(
        input.width,
        extra.width_percent,
        extra.width_calculated,
        extra.width_sizing,
        "width",
        false,
        false,
    ));
    builder.SetHeightOwned(IntrinsicDimension(
        input.height,
        extra.height_percent,
        extra.height_calculated,
        extra.height_sizing,
        "height",
        false,
        false,
    ));
    builder.SetMinWidthOwned(IntrinsicDimension(
        input.min_width,
        extra.min_width_percent,
        extra.min_width_calculated,
        extra.min_width_sizing,
        "min_width",
        true,
        false,
    ));
    builder.SetMaxWidthOwned(IntrinsicDimension(
        input.max_width,
        extra.max_width_percent,
        extra.max_width_calculated,
        extra.max_width_sizing,
        "max_width",
        true,
        true,
    ));
    builder.SetMinHeightOwned(IntrinsicDimension(
        input.min_height,
        extra.min_height_percent,
        extra.min_height_calculated,
        extra.min_height_sizing,
        "min_height",
        true,
        false,
    ));
    builder.SetMaxHeightOwned(IntrinsicDimension(
        input.max_height,
        extra.max_height_percent,
        extra.max_height_calculated,
        extra.max_height_sizing,
        "max_height",
        true,
        true,
    ));

    builder.SetMarginTop(&MarginLength(
        input.margin.top,
        extra.margin_percentages[0],
        extra.margin_auto[0],
        extra.margin_calculated[0],
        extra.margin_quirks[0],
        "margin_top",
    ));
    builder.SetPaddingTop(&EdgeLength(
        input.padding.top,
        extra.padding_percentages[0],
        false,
        extra.padding_calculated[0],
        "padding_top",
        true,
    ));
    builder.SetBorderTopWidthOwned(BorderWidth(input.border.top));
    builder.SetBorderTopStyle(ConvertBorderLineStyle(input.border_styles[0]));

    builder.SetMarginRight(&MarginLength(
        input.margin.right,
        extra.margin_percentages[1],
        extra.margin_auto[1],
        extra.margin_calculated[1],
        extra.margin_quirks[1],
        "margin_right",
    ));
    builder.SetPaddingRight(&EdgeLength(
        input.padding.right,
        extra.padding_percentages[1],
        false,
        extra.padding_calculated[1],
        "padding_right",
        true,
    ));
    builder.SetBorderRightWidthOwned(BorderWidth(input.border.right));
    builder.SetBorderRightStyle(ConvertBorderLineStyle(input.border_styles[1]));

    builder.SetMarginBottom(&MarginLength(
        input.margin.bottom,
        extra.margin_percentages[2],
        extra.margin_auto[2],
        extra.margin_calculated[2],
        extra.margin_quirks[2],
        "margin_bottom",
    ));
    builder.SetPaddingBottom(&EdgeLength(
        input.padding.bottom,
        extra.padding_percentages[2],
        false,
        extra.padding_calculated[2],
        "padding_bottom",
        true,
    ));
    builder.SetBorderBottomWidthOwned(BorderWidth(input.border.bottom));
    builder.SetBorderBottomStyle(ConvertBorderLineStyle(input.border_styles[2]));

    builder.SetMarginLeft(&MarginLength(
        input.margin.left,
        extra.margin_percentages[3],
        extra.margin_auto[3],
        extra.margin_calculated[3],
        extra.margin_quirks[3],
        "margin_left",
    ));
    builder.SetPaddingLeft(&EdgeLength(
        input.padding.left,
        extra.padding_percentages[3],
        false,
        extra.padding_calculated[3],
        "padding_left",
        true,
    ));
    builder.SetBorderLeftWidthOwned(BorderWidth(input.border.left));
    builder.SetBorderLeftStyle(ConvertBorderLineStyle(input.border_styles[3]));

    builder.SetRowGapOwned(GapLength(
        extra.row_gap,
        extra.row_gap_percent,
        extra.row_gap_calculated,
        input.gap,
        "row_gap",
    ));
    builder.SetColumnGapOwned(GapLength(
        extra.column_gap,
        extra.column_gap_percent,
        extra.column_gap_calculated,
        input.gap,
        "column_gap",
    ));
}

// cpp: layoutng/internal/boundary/native_input.cc:1202-1213
fn ApplyNativeColumnRule(builder: &mut ComputedStyleBuilder, input: &ComputedStyle) {
    let paint = &input.paint;
    if paint.column_rule_width > 0.0
        && paint.column_rule_style != BorderLineStyle::kNone
        && paint.column_rule_color.alpha > 0.0
    {
        let color = &paint.column_rule_color;
        builder.SetColumnRuleWidth(&GapDataList::from_value(&BorderWidth(
            paint.column_rule_width,
        )));
        builder.SetColumnRuleStyle(&GapDataList::from_value(&ConvertBorderLineStyle(
            paint.column_rule_style,
        )));
        let native_color =
            foundation::Color::FromRGBAFloat(color.red, color.green, color.blue, color.alpha);
        builder.SetColumnRuleColor(&GapDataList::from_value(&StyleColor::from_color(
            native_color,
        )));
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:1214-1248
fn ApplyNativeStyleFlexAlignment(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
    extra: &ExtendedStyle,
) {
    builder.SetFlexDirection(ConvertFlexDirection(input.flex_direction));
    builder.SetFlexGrow(Number(input.flex_grow as f64, "flex_grow", true) as f32);
    builder.SetFlexShrink(Number(input.flex_shrink as f64, "flex_shrink", true) as f32);
    builder.SetFlexBasisOwned(FlexBasis(input, extra));
    builder.SetFlexWrapOwned(StyleFlexWrapData::new(if extra.wrap_reverse {
        FlexWrapMode::kWrapReverse
    } else if input.flex_wrap {
        FlexWrapMode::kWrap
    } else {
        FlexWrapMode::kNowrap
    }));
    builder.SetAlignItemsOwned(StyleSelfAlignmentData::new_nonlegacy(
        ConvertAlignItems(input.align_items),
        ConvertOverflowAlignment(extra.align_items_overflow),
    ));
    let mut justify_content = ConvertJustifyContent(extra.justify_content);
    justify_content.SetOverflow(ConvertOverflowAlignment(extra.justify_content_overflow));
    builder.SetJustifyContentOwned(justify_content);
    if let Some(align_content) = extra.align_content {
        let mut value = ConvertContentAlignment(align_content);
        value.SetOverflow(ConvertOverflowAlignment(extra.align_content_overflow));
        builder.SetAlignContentOwned(value);
    }
    builder.SetAlignContentBlockCenter(extra.align_content_block_center);
    if let Some(align_self) = extra.align_self {
        let mut value = ConvertSelfAlignment(align_self);
        value.SetOverflow(ConvertOverflowAlignment(extra.align_self_overflow));
        builder.SetAlignSelfOwned(value);
    }
    if let Some(justify_items) = extra.justify_items {
        let mut value = ConvertSelfAlignment(justify_items);
        value.SetOverflow(ConvertOverflowAlignment(extra.justify_items_overflow));
        if extra.justify_items_legacy {
            value.SetPositionType(ItemPositionType::kLegacy);
        }
        builder.SetJustifyItemsOwned(value);
    }
    if let Some(justify_self) = extra.justify_self {
        let mut value = ConvertSelfAlignment(justify_self);
        value.SetOverflow(ConvertOverflowAlignment(extra.justify_self_overflow));
        builder.SetJustifySelfOwned(value);
    }
    builder.SetOrder(extra.order);
}

// cpp: layoutng/internal/boundary/native_input.cc:1249-1311
fn ApplyNativeStyleMulticolAndInitialLetter(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
    extra: &ExtendedStyle,
) {
    Count(input.column_count, u16::MAX as u32, "column_count");
    if input.column_count > 1 || extra.explicit_column_count {
        builder.SetColumnCount(input.column_count as u16);
    }
    if let Some(width) = extra.column_width {
        let width = Number(width, "column_width", true);
        if width <= 0.0 {
            Invalid("column_width");
        }
        builder.SetColumnWidth(width as f32);
    }
    builder.SetColumnSpan(if extra.column_span_all {
        EColumnSpan::kAll
    } else {
        EColumnSpan::kNone
    });
    builder.SetColumnFill(if extra.column_fill_balance {
        EColumnFill::kBalance
    } else {
        EColumnFill::kAuto
    });
    builder.SetColumnWrap(match extra.column_wrap {
        ColumnWrap::kAuto => EColumnWrap::kAuto,
        ColumnWrap::kNowrap => EColumnWrap::kNowrap,
        ColumnWrap::kWrap => EColumnWrap::kWrap,
    });
    if extra.initial_letter.size != 0.0 {
        let size = Number(extra.initial_letter.size, "initial_letter size", true) as f32;
        if size < 1.0 {
            Invalid("initial_letter size");
        }
        let initial_letter = match extra.initial_letter.sink_type {
            InitialLetterSink::kOmitted => {
                if extra.initial_letter.sink != 0 {
                    Invalid("initial_letter omitted sink");
                }
                unsafe { NativeInitialLetterOmitted(size) }
            }
            InitialLetterSink::kInteger => {
                if extra.initial_letter.sink == 0 || extra.initial_letter.sink > i32::MAX as u32 {
                    Invalid("initial_letter sink");
                }
                unsafe { NativeInitialLetterInteger(size, extra.initial_letter.sink as i32) }
            }
            InitialLetterSink::kDrop => {
                if extra.initial_letter.sink != 0 {
                    Invalid("initial_letter drop sink");
                }
                unsafe { NativeInitialLetterDrop(size) }
            }
            InitialLetterSink::kRaise => {
                if extra.initial_letter.sink != 0 {
                    Invalid("initial_letter raise sink");
                }
                unsafe { NativeInitialLetterRaise(size) }
            }
        };
        builder.SetInitialLetterOwned(initial_letter);
    } else if extra.initial_letter.sink != 0
        || extra.initial_letter.sink_type != InitialLetterSink::kOmitted
    {
        Invalid("initial_letter normal");
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:1312-1359
fn ApplyNativeStyleGridTableAndBreaks(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
    extra: &ExtendedStyle,
    context: FragmentationType,
) {
    builder.SetGridTemplateColumnsOwned(Member::from_ptr(Tracks(
        &extra.grid_columns,
        &input.grid_template_columns,
        extra.subgrid_columns,
    )));
    builder.SetGridTemplateRowsOwned(Member::from_ptr(Tracks(
        &extra.grid_rows,
        &[],
        extra.subgrid_rows,
    )));
    if let Some(areas) = &extra.grid_template_areas {
        builder.SetGridTemplateAreasOwned(Member::from_ptr(GridTemplateAreas(areas)));
    }
    if !extra.grid_auto_columns.empty() {
        builder.SetGridAutoColumnsOwned(AutoTracks(&extra.grid_auto_columns));
    }
    if !extra.grid_auto_rows.empty() {
        builder.SetGridAutoRowsOwned(AutoTracks(&extra.grid_auto_rows));
    }
    builder.SetGridAutoFlow(if extra.grid_auto_flow_column {
        if extra.grid_auto_flow_dense {
            GridAutoFlow::kAutoFlowColumnDense
        } else {
            GridAutoFlow::kAutoFlowColumn
        }
    } else if extra.grid_auto_flow_dense {
        GridAutoFlow::kAutoFlowRowDense
    } else {
        GridAutoFlow::kAutoFlowRow
    });
    builder.SetGridColumnStartOwned(GridPosition(&extra.grid_column_start, "grid_column_start"));
    builder.SetGridColumnEndOwned(GridPosition(&extra.grid_column_end, "grid_column_end"));
    builder.SetGridRowStartOwned(GridPosition(&extra.grid_row_start, "grid_row_start"));
    builder.SetGridRowEndOwned(GridPosition(&extra.grid_row_end, "grid_row_end"));
    builder.SetTableLayout(if extra.table_layout_fixed {
        ETableLayout::kFixed
    } else {
        ETableLayout::kAuto
    });
    builder.SetBorderCollapse(if extra.border_collapse {
        EBorderCollapse::kCollapse
    } else {
        EBorderCollapse::kSeparate
    });
    builder.SetCaptionSide(if extra.caption_side == CaptionSide::kBottom {
        ECaptionSide::kBottom
    } else {
        ECaptionSide::kTop
    });
    let horizontal_spacing = unsafe {
        RoundForImpreciseConversionI16(Number(extra.border_spacing, "border_spacing", true))
    };
    builder.SetHorizontalBorderSpacing(horizontal_spacing);
    let vertical_spacing = unsafe {
        RoundForImpreciseConversionI16(Number(
            extra
                .vertical_border_spacing
                .unwrap_or(extra.border_spacing),
            "vertical_border_spacing",
            true,
        ))
    };
    builder.SetVerticalBorderSpacing(vertical_spacing);
    builder.SetBreakBefore(ConvertBreakRule(extra.break_before, context));
    builder.SetBreakAfter(ConvertBreakRule(extra.break_after, context));
    builder.SetBreakInside(match extra.break_inside {
        BreakRule::kAuto => EBreakInside::kAuto,
        BreakRule::kAvoid => EBreakInside::kAvoid,
        BreakRule::kAlways => Invalid("break_inside (always is not a valid inside rule)"),
    });
    builder.SetWidows(Count(extra.widows, i16::MAX as u32, "widows") as i16);
    builder.SetOrphans(Count(extra.orphans, i16::MAX as u32, "orphans") as i16);
    builder.SetClear(ConvertClearSide(extra.clear_side));
    builder.SetOverflowX(ConvertOverflow(extra.overflow_x));
    builder.SetOverflowY(ConvertOverflow(extra.overflow_y));
}

// cpp: layoutng/internal/boundary/native_input.cc:1478-1534
fn ApplyNativeTextDecoration(builder: &mut ComputedStyleBuilder, input: &ComputedStyle) {
    let decoration = &input.paint.text_decoration;
    let mut lines = TextDecorationLine::kNone;
    if decoration.underline {
        lines |= TextDecorationLine::kUnderline;
    }
    if decoration.overline {
        lines |= TextDecorationLine::kOverline;
    }
    if decoration.line_through {
        lines |= TextDecorationLine::kLineThrough;
    }
    builder.SetTextDecorationLine(lines);
    builder.SetTextDecorationStyle(match decoration.style {
        TextDecorationStyle::kSolid => ETextDecorationStyle::kSolid,
        TextDecorationStyle::kDouble => ETextDecorationStyle::kDouble,
        TextDecorationStyle::kDotted => ETextDecorationStyle::kDotted,
        TextDecorationStyle::kDashed => ETextDecorationStyle::kDashed,
        TextDecorationStyle::kWavy => ETextDecorationStyle::kWavy,
    });
    let color = decoration.color.unwrap_or(input.paint.color);
    builder.SetTextDecorationColorOwned(StyleColor::from_color(foundation::Color::FromRGBAFloat(
        color.red,
        color.green,
        color.blue,
        color.alpha,
    )));
    let thickness = decoration
        .thickness
        .map_or_else(|| Length::Auto().clone(), Length::Fixed);
    builder.SetTextDecorationThicknessOwned(TextDecorationThickness::new(&thickness));
    let underline_offset = if decoration.underline_offset_auto {
        Length::Auto().clone()
    } else {
        Length::Fixed(decoration.underline_offset)
    };
    builder.SetTextUnderlineOffsetOwned(underline_offset);
    builder.SetTextDecorationSkipInk(if decoration.skip_ink {
        ETextDecorationSkipInk::kAuto
    } else {
        ETextDecorationSkipInk::kNone
    });
}

fn ApplyNativeStyleText(
    builder: &mut ComputedStyleBuilder,
    input: &ComputedStyle,
    extra: &ExtendedStyle,
) {
    ApplyNativeTextDecoration(builder, input);
    builder.SetWhiteSpace(ConvertWhiteSpace(extra.white_space));
    match extra.text_wrap_mode {
        TextWrapMode::kFromWhiteSpace => {}
        TextWrapMode::kWrap => builder.SetTextWrapMode(NativeTextWrapMode::kWrap),
        TextWrapMode::kNowrap => builder.SetTextWrapMode(NativeTextWrapMode::kNowrap),
    }
    builder.SetTextWrapStyle(ConvertTextWrapStyle(extra.text_wrap_style));
    builder.SetTextAlign(ConvertTextAlign(extra.text_align));
    builder.SetTextAlignLast(ConvertTextAlignLast(extra.text_align_last));
    if extra.vertical_align_length.is_some()
        || extra.vertical_align_percent.is_some()
        || extra.vertical_align_calculated
    {
        builder.SetVerticalAlignLength(&VerticalAlignLength(extra));
    } else {
        builder.SetVerticalAlign(ConvertVerticalAlign(extra.vertical_align));
    }
    builder.SetUnicodeBidi(ConvertUnicodeBidi(extra.unicode_bidi));
    if extra.line_height.is_some() && extra.line_height_percent.is_some() {
        Invalid("line_height");
    }
    if let Some(height) = extra.line_height {
        builder.SetLineHeight(&Pixels(height, "line_height", true));
    } else if let Some(percent) = extra.line_height_percent {
        builder.SetLineHeight(&Length::Percent(Number(percent, "line_height", true)));
    }
    builder.SetTextIndentOwned(TextIndentLength(extra));
    let mut text_indent_flags = TextIndentFlags::kDefault;
    if extra.text_indent_each_line {
        text_indent_flags |= TextIndentFlags::kEachLine;
    }
    if extra.text_indent_hanging {
        text_indent_flags |= TextIndentFlags::kHanging;
    }
    builder.SetTextIndentFlags(text_indent_flags);
    builder.SetOverflowWrap(ConvertOverflowWrap(extra.overflow_wrap));
    builder.SetWordBreak(ConvertWordBreak(extra.word_break));
    builder.SetLineBreak(ConvertLineBreak(extra.line_break));
    builder.SetHyphens(ConvertHyphens(extra.hyphens));
    let tab_size = Number(extra.tab_size, "tab_size", true);
    if tab_size < 0.0 {
        Invalid("tab_size");
    }
    builder.SetTabSize(&TabSize::new(
        tab_size as f32,
        if extra.tab_size_is_length {
            TabSizeValueType::kLength
        } else {
            TabSizeValueType::kSpace
        },
    ));
    builder.SetTextOrientation(ConvertTextOrientation(extra.text_orientation));
    builder.SetTextCombine(ConvertTextCombine(extra.text_combine));
    builder.SetTextTransform(ConvertTextTransform(extra.text_transform));
    builder.SetRubyPosition(ConvertRubyPosition(extra.ruby_position));
    builder.SetRubyAlign(ConvertRubyAlign(extra.ruby_align));
    builder.SetRubyOverhang(ConvertRubyOverhang(extra.ruby_overhang));
    builder.SetEmptyCells(ConvertEmptyCells(extra.empty_cells));
    builder.SetMarginTrim(ConvertMarginTrim(extra.margin_trim));
    if extra.line_clamp != 0 {
        builder.SetMaxLinesOwned(MaxLinesData::new(
            Count(extra.line_clamp, u16::MAX as u32, "line_clamp") as u16,
            false,
        ));
        builder.SetContinue(EContinue::kCollapse);
        builder.SetLineClampInternalBlockEllipsis(EBlockEllipsis::kEllipsis);
    }
}

// Copy actual resolved CSS paint values into the native style. The native
// LayerTypeRequired then reads the same style fields as Chromium; a sidecar
// painting value alone does not establish native stacking-context semantics.
// cpp: core/style/computed_style.h:3345-3348
// cpp: core/style/computed_style.cc:2913-2949
fn ApplyNativeStylePaintGrouping(builder: &mut ComputedStyleBuilder, input: &ComputedStyle) {
    if let Some(clip) = input.css_clip {
        let side = |value: Option<f64>| {
            value
                .map(Length::Fixed)
                .unwrap_or_else(|| Length::Auto().clone())
        };
        builder.SetClip(&LengthBox::new(
            side(clip.top),
            side(clip.right),
            side(clip.bottom),
            side(clip.left),
        ));
    } else {
        builder.SetHasAutoClip();
    }
    builder.SetClipPath(
        input
            .paint
            .clip_path
            .as_ref()
            .map(clip_path_native::NativeClipPath),
    );
    if !input.paint.opacity.is_finite() {
        Invalid("opacity");
    }
    builder.SetOpacity(input.paint.opacity);
    builder.SetIsolation(if input.paint.isolate_blending {
        EIsolation::kIsolate
    } else {
        EIsolation::kAuto
    });
    builder.SetBlendMode(match input.paint.blend_mode {
        PaintBlendMode::kNormal => BlendMode::kNormal,
        PaintBlendMode::kMultiply => BlendMode::kMultiply,
        PaintBlendMode::kScreen => BlendMode::kScreen,
        PaintBlendMode::kOverlay => BlendMode::kOverlay,
        PaintBlendMode::kDarken => BlendMode::kDarken,
        PaintBlendMode::kLighten => BlendMode::kLighten,
        PaintBlendMode::kColorDodge => BlendMode::kColorDodge,
        PaintBlendMode::kColorBurn => BlendMode::kColorBurn,
        PaintBlendMode::kHardLight => BlendMode::kHardLight,
        PaintBlendMode::kSoftLight => BlendMode::kSoftLight,
        PaintBlendMode::kDifference => BlendMode::kDifference,
        PaintBlendMode::kExclusion => BlendMode::kExclusion,
        PaintBlendMode::kHue => BlendMode::kHue,
        PaintBlendMode::kSaturation => BlendMode::kSaturation,
        PaintBlendMode::kColor => BlendMode::kColor,
        PaintBlendMode::kLuminosity => BlendMode::kLuminosity,
        PaintBlendMode::kPlusLighter => BlendMode::kPlusLighter,
    });
    // Preserve each supported CSS filter's real native class/payload.
    // cpp: core/css/resolver/filter_operation_resolver.cc:307-351
    if !input.paint.filters.is_empty() {
        let mut filters = FilterOperations::new();
        for filter in &input.paint.filters {
            let operation: *mut FilterOperation = match filter.r#type {
                PaintFilterType::kGrayscale
                | PaintFilterType::kSepia
                | PaintFilterType::kSaturate
                | PaintFilterType::kHueRotate => {
                    let type_ = match filter.r#type {
                        PaintFilterType::kGrayscale => OperationType::kGrayscale,
                        PaintFilterType::kSepia => OperationType::kSepia,
                        PaintFilterType::kSaturate => OperationType::kSaturate,
                        _ => OperationType::kHueRotate,
                    };
                    MakeGarbageCollected(BasicColorMatrixFilterOperation::new(
                        Number(
                            filter.amount,
                            "filter amount",
                            filter.r#type != PaintFilterType::kHueRotate,
                        ),
                        type_,
                    ))
                    .cast()
                }
                PaintFilterType::kInvert
                | PaintFilterType::kOpacity
                | PaintFilterType::kBrightness
                | PaintFilterType::kContrast => {
                    let type_ = match filter.r#type {
                        PaintFilterType::kInvert => OperationType::kInvert,
                        PaintFilterType::kOpacity => OperationType::kOpacity,
                        PaintFilterType::kBrightness => OperationType::kBrightness,
                        _ => OperationType::kContrast,
                    };
                    MakeGarbageCollected(BasicComponentTransferFilterOperation::new(
                        Number(filter.amount, "filter amount", true),
                        type_,
                    ))
                    .cast()
                }
                PaintFilterType::kBlur => {
                    let deviation = Length::Fixed(Number(filter.amount, "filter blur", true));
                    MakeGarbageCollected(BlurFilterOperation::new(&deviation)).cast()
                }
                PaintFilterType::kDropShadow => {
                    let color = foundation::Color::FromRGBAFloat(
                        filter.color.red,
                        filter.color.green,
                        filter.color.blue,
                        filter.color.alpha,
                    );
                    let shadow = ShadowData::new(
                        gfx::Vector2dF::new(
                            Number(filter.offset.x, "filter shadow x", false) as f32,
                            Number(filter.offset.y, "filter shadow y", false) as f32,
                        ),
                        Number(filter.blur_radius, "filter shadow blur", true) as f32,
                        0.0,
                        ShadowStyle::kNormal,
                        StyleColor::from_color(color),
                        1.0,
                    );
                    MakeGarbageCollected(DropShadowFilterOperation::new(shadow)).cast()
                }
            };
            filters
                .OperationsMut()
                .push_back(Member::from_ptr(operation));
        }
        builder.SetFilterOwned(filters);
    }
    if let Some(index) = input.paint.z_index {
        builder.SetZIndex(index);
    }
    if input.paint.will_change_transform {
        let mut values = Vector::new();
        values.push(AtomicString::from_utf16(
            &"transform".encode_utf16().collect::<Vec<_>>(),
        ));
        let ids = CSSBitset::FromList(&[CSSPropertyID::kTransform]);
        let will_change =
            MakeGarbageCollected(StyleWillChangeData::new(values, ids, false, true, true));
        builder.SetWillChange(Member::from_ptr(will_change));
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:1360-1399
fn ApplyNativeStyleTransforms(builder: &mut ComputedStyleBuilder, input: &ComputedStyle) {
    if let Some(transform) = &input.paint.transform {
        let mut operations = TransformOperations::default();
        for operation in &transform.operations {
            match operation.kind {
                PaintTransformOperationKind::kMatrix => {
                    for value in operation.matrix.values {
                        if !value.is_finite() {
                            Invalid("transform matrix");
                        }
                    }
                    operations.PushMatrix3D(gfx::Transform::ColMajor(&operation.matrix.values));
                }
                PaintTransformOperationKind::kTranslate => {
                    let x = LengthPercentage(
                        operation.pixels.x,
                        operation.percentages.x,
                        "transform translate x",
                    );
                    let y = LengthPercentage(
                        operation.pixels.y,
                        operation.percentages.y,
                        "transform translate y",
                    );
                    operations.PushTranslate(x, y);
                }
                PaintTransformOperationKind::kTranslate3D => {
                    let x = LengthPercentage(
                        operation.pixels.x,
                        operation.percentages.x,
                        "transform translate3d x",
                    );
                    let y = LengthPercentage(
                        operation.pixels.y,
                        operation.percentages.y,
                        "transform translate3d y",
                    );
                    operations.PushTranslate3D(
                        x,
                        y,
                        Number(operation.z, "transform translate3d z", false),
                    );
                }
            }
        }
        builder.SetTransformOwned(operations);
    }
    if let Some(origin) = &input.paint.transform_origin {
        builder.SetTransformOriginX(&LengthPercentage(
            origin.pixels.x,
            origin.percentages.x,
            "transform_origin.x",
        ));
        builder.SetTransformOriginY(&LengthPercentage(
            origin.pixels.y,
            origin.percentages.y,
            "transform_origin.y",
        ));
        builder.SetTransformOriginZ(Number(origin.z, "transform_origin.z", false) as f32);
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:1400-1448
fn ApplyNativeStyleOverflowAndScrollbars(
    builder: &mut ComputedStyleBuilder,
    extra: &ExtendedStyle,
) {
    if let Some(margin) = &extra.overflow_clip_margin {
        let reference_box = match margin.reference_box {
            OverflowClipReferenceBox::kBorderBox => NativeOverflowClipReferenceBox::kBorderBox,
            OverflowClipReferenceBox::kPaddingBox => NativeOverflowClipReferenceBox::kPaddingBox,
            OverflowClipReferenceBox::kContentBox => NativeOverflowClipReferenceBox::kContentBox,
        };
        builder.SetOverflowClipMarginOwned(Some(StyleOverflowClipMargin::new(
            reference_box,
            LayoutUnit::from_f64(Number(margin.margin, "overflow_clip_margin", true)),
        )));
    }
    builder.SetBoxDecorationBreak(match extra.box_decoration_break {
        BoxDecorationBreak::kSlice => EBoxDecorationBreak::kSlice,
        BoxDecorationBreak::kClone => EBoxDecorationBreak::kClone,
    });
    builder.SetScrollbarWidth(match extra.scrollbar_width {
        ScrollbarWidth::kAuto => EScrollbarWidth::kAuto,
        ScrollbarWidth::kThin => EScrollbarWidth::kThin,
        ScrollbarWidth::kNone => EScrollbarWidth::kNone,
    });
    builder.SetPrefersDefaultScrollbarStyles(extra.prefers_default_scrollbar_styles);
    builder.SetScrollbarGutter(match extra.scrollbar_gutter {
        ScrollbarGutter::kAuto => NativeScrollbarGutter::kScrollbarGutterAuto.value() as u32,
        ScrollbarGutter::kStable => NativeScrollbarGutter::kScrollbarGutterStable.value() as u32,
        ScrollbarGutter::kStableBothEdges => {
            (NativeScrollbarGutter::kScrollbarGutterStable.value()
                | NativeScrollbarGutter::kScrollbarGutterBothEdges.value()) as u32
        }
    });
}

// cpp: layoutng/internal/boundary/native_input.cc:1449-1462
fn ApplyNativeTriggerScope(builder: &mut ComputedStyleBuilder, extra: &ExtendedStyle) {
    if extra.trigger_scope_all && !extra.trigger_scope_names.is_empty() {
        Invalid("trigger scope cannot combine all with names");
    }
    if extra.trigger_scope_all {
        builder.SetTriggerScopeOwned(NativeTriggerScopeAll());
    } else if !extra.trigger_scope_names.is_empty() {
        let mut names = Vec::with_capacity(extra.trigger_scope_names.len());
        for value in &extra.trigger_scope_names {
            if value.is_empty() {
                Invalid("trigger_scope_names");
            }
            names.push(AtomicString::from_utf16(
                &value.encode_utf16().collect::<Vec<_>>(),
            ));
        }
        builder.SetTriggerScopeOwned(NativeTriggerScopeNames(&names));
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:1463-1477
fn ApplyNativeStyleContainmentAndAspect(builder: &mut ComputedStyleBuilder, extra: &ExtendedStyle) {
    let contain = (if extra.layout_containment {
        Containment::kContainsLayout.value()
    } else {
        0
    }) | (if extra.size_containment {
        Containment::kContainsSize.value()
    } else {
        0
    });
    builder.SetContain(contain as u32);
    let intrinsic_length = |value: &ContainIntrinsicLength, field: &str| {
        let length = value.length.map(|pixels| Pixels(pixels, field, true));
        StyleIntrinsicLength::new(
            &length,
            StyleIntrinsicLengthOptions {
                has_auto: value.has_auto,
            },
        )
    };
    builder.SetContainIntrinsicWidthOwned(intrinsic_length(
        &extra.contain_intrinsic_width,
        "contain_intrinsic_width",
    ));
    builder.SetContainIntrinsicHeightOwned(intrinsic_length(
        &extra.contain_intrinsic_height,
        "contain_intrinsic_height",
    ));
    if let Some(ratio) = extra.aspect_ratio {
        builder.SetAspectRatioOwned(StyleAspectRatio::new(
            EAspectRatioType::kRatio,
            gfx::SizeF::new(Number(ratio, "aspect_ratio", true) as f32, 1.0),
        ));
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:1535-1589
fn FinishNativeStyle(
    builder: &mut ComputedStyleBuilder,
    node: &NativeNodeConstructionData,
    extra: &ExtendedStyle,
    fonts: &mut dyn NativeFontResolver,
    layout_parent_style: *const NativeComputedStyle,
    layout_parent_kind: NodeKind,
) -> *const NativeComputedStyle {
    if let Some(element) = &node.element {
        if !element.custom_layout_name.is_empty() {
            builder.SetDisplayLayoutCustomNameOwned(AtomicString::from_utf16(
                &element
                    .custom_layout_name
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            ));
        }
    }
    let mut display_context = DisplayAdjustmentContext::default();
    display_context.is_root = layout_parent_style.is_null();
    display_context.should_be_inlinified = layout_parent_kind != NodeKind::kFieldset;
    if node.kind != NodeKind::kText {
        let parent = if layout_parent_style.is_null() {
            unsafe { &*NativeComputedStyle::GetInitialStyleSingleton() }
        } else {
            unsafe { &*layout_parent_style }
        };
        AdjustComputedDisplayForLayout(builder, parent, &display_context);
    }
    // core/css/resolver/style_adjuster.cc:1289-1298. Text decorations are
    // propagated through the layout-parent chain as applied decorating boxes;
    // they are not ordinary inherited CSS values.
    if !layout_parent_style.is_null()
        && !builder.IsAtomicInlineDisplayType()
        && !builder.IsFloating()
        && !builder.HasOutOfFlowPosition()
        && builder.Display() != EDisplay::kRubyText
    {
        builder.SetBaseTextDecorationData(Member::from_ptr(
            unsafe { &*layout_parent_style }.AppliedTextDecorationData(),
        ));
    }
    let effective_direction =
        WritingDirectionMode::new(builder.GetWritingMode(), builder.Direction());
    if extra.auto_margin_inline_start {
        AutoMargin(builder, effective_direction.InlineStart());
    }
    if extra.auto_margin_inline_end {
        AutoMargin(builder, effective_direction.InlineEnd());
    }
    let font_writing_mode = match builder.GetWritingMode() {
        NativeWritingMode::kHorizontalTb => WritingMode::kHorizontalTb,
        NativeWritingMode::kVerticalRl => WritingMode::kVerticalRl,
        NativeWritingMode::kVerticalLr => WritingMode::kVerticalLr,
        _ => Invalid("font writing mode outside public input contract"),
    };
    Number(extra.font_size, "font_size", true);
    if !extra.font_weight.is_finite() || extra.font_weight < 1.0 || extra.font_weight > 1000.0 {
        Invalid("font_weight");
    }
    Number(extra.letter_spacing, "letter_spacing", false);
    Number(extra.word_spacing, "word_spacing", false);
    // The public Rust strings are valid UTF-8 by construction.
    for family in &extra.font_families {
        if family.is_empty() {
            Invalid("font_family UTF-8");
        }
    }
    let request = NativeFontRequest::new_auto(
        extra.font_size,
        extra.letter_spacing,
        extra.word_spacing,
        font_writing_mode,
        &extra.language,
        &extra.font_families,
        extra.font_weight,
        extra.font_italic,
        builder.ComputeFontOrientation(),
    );
    let font = fonts.Resolve(&request);
    builder.SetFont(Member::from_ptr(font as *mut _));
    if let Some(element) = &node.element {
        if element.first_letter_pseudo {
            if node.kind != NodeKind::kBox || layout_parent_style.is_null() {
                Invalid("first_letter_pseudo requires a box with a parent");
            }
            builder.SetStyleType(PseudoId::kPseudoIdFirstLetter);
            let floating = builder.IsFloating();
            builder.SetDisplay(if floating {
                EDisplay::kBlock
            } else {
                EDisplay::kInline
            });
            builder.SetContainerFont(Member::from_ptr(unsafe { &*layout_parent_style }.GetFont()));
        }
    }
    builder.TakeStyle()
}

// cpp: layoutng/internal/boundary/native_input.h:39-50
// cpp: layoutng/internal/boundary/native_input.cc:939-946
pub fn PrepareNativeStyle(
    node: &NativeNodeConstructionData,
    resolver: &mut dyn NativeFontResolver,
    context: FragmentationType,
    layout_parent_style: *const NativeComputedStyle,
    layout_parent_kind: NodeKind,
) -> *const NativeComputedStyle {
    // StyleEngine already produced Chromium's immutable ComputedStyle.  The
    // layout-input projection is still retained for DOM/paint metadata, but
    // layout must consume the original style object rather than translating
    // that projection back into a second style.
    if !node.style.native_style.is_null() {
        return node.style.native_style;
    }
    // Like Blink's CSSToLengthConversionData, zoom fixed CSS lengths before
    // passing them to layout. The DOM style remains in CSS units.
    let zoomed = crate::internal::css_zoom::ZoomedStyle(&node.style);
    let input = zoomed.as_ref();
    let defaults = ExtendedStyle::default();
    let extra = input.extended.as_ref().unwrap_or(&defaults);
    let mut builder = BeginNativeStyle(input, extra);
    ApplyNativeStyleListAndPosition(&mut builder, input, extra);
    ApplyNativeStyleShape(&mut builder, extra);
    ApplyNativeStyleBoxEdges(&mut builder, input, extra);
    ApplyNativeColumnRule(&mut builder, input);
    ApplyNativeStyleFlexAlignment(&mut builder, input, extra);
    ApplyNativeStyleMulticolAndInitialLetter(&mut builder, input, extra);
    ApplyNativeStyleGridTableAndBreaks(&mut builder, input, extra, context);
    ApplyNativeStyleTransforms(&mut builder, input);
    ApplyNativeStylePaintGrouping(&mut builder, input);
    mask_image_native::ApplyNativeMaskImages(&mut builder, input);
    ApplyNativeStyleOverflowAndScrollbars(&mut builder, extra);
    ApplyNativeTriggerScope(&mut builder, extra);
    ApplyNativeStyleContainmentAndAspect(&mut builder, extra);
    ApplyNativeStyleText(&mut builder, input, extra);
    FinishNativeStyle(
        &mut builder,
        node,
        extra,
        resolver,
        layout_parent_style,
        layout_parent_kind,
    )
}

pub fn PrepareNativeStyleDefault(
    input: &NativeNodeConstructionData,
    resolver: &mut dyn NativeFontResolver,
) -> *const NativeComputedStyle {
    PrepareNativeStyle(
        input,
        resolver,
        FragmentationType::kFragmentPage,
        std::ptr::null(),
        NodeKind::kBox,
    )
}

#[cfg(test)]
mod style_engine_style_reuse_tests {
    use super::*;
    use crate::internal::layout_font_resolver::NativeFontRequest;
    use font_engine::Font;

    struct UnusedFontResolver;

    impl NativeFontResolver for UnusedFontResolver {
        fn Resolve(&mut self, _: &NativeFontRequest<'_>) -> &mut Font {
            panic!("a StyleEngine-produced ComputedStyle must bypass input font reconstruction")
        }
    }

    #[test]
    fn prepare_native_style_reuses_style_engine_object_identity() {
        let _heap = foundation::LayoutHeapScope::new();
        let native = NativeComputedStyle::GetInitialStyleSingleton();
        let mut input = NativeNodeConstructionData::default();
        input.style.native_style = native;
        let actual = PrepareNativeStyle(
            &input,
            &mut UnusedFontResolver,
            FragmentationType::kFragmentPage,
            std::ptr::null(),
            NodeKind::kBox,
        );
        assert_eq!(actual, native);
    }
}

// cpp: layoutng/internal/boundary/native_input.h:52-57
// cpp: layoutng/internal/boundary/native_input.cc:1591-1618
pub fn PrepareNativeConstraints(
    root_style: &ComputedStyle,
    space: &ConstraintSpace,
    contains_annotations: bool,
) -> NativeConstraintSpace {
    let direction = WritingDirectionMode::new(
        ConvertWritingMode(root_style.writing_mode),
        ConvertTextDirection(root_style.direction),
    );
    let width = LayoutUnit::from_f64(Number(space.available_size.width, "available width", true));
    let height = LayoutUnit::from_f64(Number(
        space.available_size.height,
        "available height",
        true,
    ));
    let mut available = if direction.IsHorizontal() {
        LogicalSize::new(width, height)
    } else {
        LogicalSize::new(height, width)
    };
    if let Some(block_size) = space.fragmentainer_block_size {
        // In paged layout the initial containing block uses the page area.
        available.block_size =
            LayoutUnit::from_f64(Number(block_size, "fragmentainer block size", true));
    }
    let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
        direction.GetWritingMode(),
        direction,
        true,
        true,
        false,
    );
    builder.SetAvailableSize(available);
    builder.SetPercentageResolutionSize(available);
    builder.SetIsFixedInlineSize(true);
    builder.SetIsFixedBlockSize(true);
    builder.SetContainsAnnotations(contains_annotations);
    builder.ToConstraintSpace()
}

pub fn PrepareNativeConstraintsDefault(
    root_style: &ComputedStyle,
    space: &ConstraintSpace,
) -> NativeConstraintSpace {
    PrepareNativeConstraints(root_style, space, false)
}

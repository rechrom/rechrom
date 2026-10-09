#![allow(non_camel_case_types, non_upper_case_globals)]

use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;

use super::form_control_types::{AutofillState, FormControlType};
use super::layout_input_types::{Color, ControlThemeMetrics, IntSize, ScrollbarThemeMetrics};

// The public input declarations and their in-header behavior map from
// layout_input.h. Referenced paint and custom-layout interfaces are supplied
// by their owning packages or later files in this package.

// cpp: layoutng/internal/layout_input.h:24-29
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    kBlock,
    kFlex,
    kInlineFlex,
    kGrid,
    kInlineGrid,
    kInline,
    kTable,
    kInlineTable,
    kTableRow,
    kTableCell,
    kFlowRoot,
    kInlineBlock,
    kTableSection,
    kTableHeaderGroup,
    kTableFooterGroup,
    kTableCaption,
    kTableColumnGroup,
    kTableColumn,
    kGridLanes,
    kCustom,
    kListItem,
    kMath,
    kBlockMath,
    kRuby,
    kBlockRuby,
    kRubyText,
}

// cpp: layoutng/internal/layout_input.h:30-43
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    kRow,
    kColumn,
    kRowReverse,
    kColumnReverse,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    kNormal,
    kStretch,
    kBaseline,
    kLastBaseline,
    kCenter,
    kStart,
    kEnd,
    kSelfStart,
    kSelfEnd,
    kFlexStart,
    kFlexEnd,
}

// cpp: layoutng/internal/layout_input.h:44-53
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    kBox,
    kText,
    kLineBreak,
    kWordBreak,
    kReplaced,
    kFieldset,
    kLegend,
    kFormControl,
    kFrameSet,
    kFrame,
    kListItem,
    kListMarker,
    kRuby,
    kRubyBase,
    kRubyAnnotation,
    kSvgRoot,
    kSvgGroup,
    kSvgForeignObject,
    kSvgText,
    kSvgInline,
    kSvgTSpan,
    kSvgTextPath,
    kSvgShape,
    kMathRow,
    kMathFraction,
    kMathSquareRoot,
    kMathRoot,
    kMathPadded,
    kMathSpace,
    kMathToken,
    kMathOperator,
    kMathSub,
    kMathSup,
    kMathSubSup,
    kMathUnder,
    kMathOver,
    kMathUnderOver,
    kMathTableCell,
    kMathMultiscripts,
    kMathPrescripts,
    kMathNone,
    kMathContainer,
    kSliderThumb,
    kMarquee,
    kMathTable,
}

// cpp: layoutng/internal/layout_input.h:54-57
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    kLtr,
    kRtl,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WritingMode {
    kHorizontalTb,
    kVerticalRl,
    kVerticalLr,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    kStatic,
    kAbsolute,
    kRelative,
    kFixed,
    kSticky,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatSide {
    kNone,
    kLeft,
    kRight,
    kInlineStart,
    kInlineEnd,
}

// cpp: layoutng/internal/layout_input.h:59-64
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

// cpp: layoutng/internal/layout_input.h:66-74
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrinsicSizing {
    kAuto,
    kMinContent,
    kMaxContent,
    kFitContent,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexBasisSizing {
    kAuto,
    kContent,
    kMinContent,
    kMaxContent,
    kFitContent,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowAlignment {
    kDefault,
    kUnsafe,
    kSafe,
}

// cpp: layoutng/internal/layout_input.h:75-119
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    kNormal,
    kStart,
    kEnd,
    kFlexStart,
    kFlexEnd,
    kCenter,
    kLeft,
    kRight,
    kSpaceBetween,
    kSpaceAround,
    kSpaceEvenly,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentAlignment {
    kNormal,
    kBaseline,
    kLastBaseline,
    kStart,
    kEnd,
    kFlexStart,
    kFlexEnd,
    kCenter,
    kLeft,
    kRight,
    kSpaceBetween,
    kSpaceAround,
    kSpaceEvenly,
    kStretch,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfAlignment {
    kAuto,
    kNormal,
    kStretch,
    kBaseline,
    kLastBaseline,
    kCenter,
    kStart,
    kEnd,
    kSelfStart,
    kSelfEnd,
    kFlexStart,
    kFlexEnd,
    kLeft,
    kRight,
}

// cpp: layoutng/internal/layout_input.h:120-172
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhiteSpace {
    kNormal,
    kNowrap,
    kPre,
    kPreLine,
    kPreWrap,
    kBreakSpaces,
    // CSS Text exposes white-space-collapse and text-wrap-mode separately.
    // These two combinations have no legacy white-space keyword spelling but
    // remain valid computed values.
    kPreserveBreaksNowrap,
    kBreakSpacesNowrap,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    kStart,
    kEnd,
    kLeft,
    kRight,
    kCenter,
    kJustify,
    kMatchParent,
    kWebkitLeft,
    kWebkitRight,
    kWebkitCenter,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlignLast {
    kAuto,
    kStart,
    kEnd,
    kLeft,
    kRight,
    kCenter,
    kJustify,
    kMatchParent,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    kBaseline,
    kMiddle,
    kSub,
    kSuper,
    kTextTop,
    kTextBottom,
    kTop,
    kBottom,
    kBaselineMiddle,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowWrap {
    kNormal,
    kBreakWord,
    kAnywhere,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordBreak {
    kNormal,
    kBreakAll,
    kKeepAll,
    kAutoPhrase,
    kBreakWord,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineBreak {
    kAuto,
    kLoose,
    kNormal,
    kStrict,
    kAnywhere,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hyphens {
    kNone,
    kManual,
    kAuto,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOrientation {
    kMixed,
    kUpright,
    kSideways,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSmoothing {
    kAuto,
    kNone,
    kAntialiased,
    kSubpixelAntialiased,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextCombine {
    kNone,
    kAll,
}

// C++ permits OR results that are not named enumerators, so keep the raw bits.
// cpp: layoutng/internal/layout_input.h:173-187
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextTransform(pub u32);
impl TextTransform {
    pub const kNone: Self = Self(0);
    pub const kCapitalize: Self = Self(1 << 0);
    pub const kUppercase: Self = Self(1 << 1);
    pub const kLowercase: Self = Self(1 << 2);
    pub const kFullWidth: Self = Self(1 << 3);
    pub const kFullSizeKana: Self = Self(1 << 4);
    pub const kMathAuto: Self = Self(1 << 5);
}
impl std::ops::BitOr for TextTransform {
    type Output = Self;
    fn bitor(self, right: Self) -> Self {
        Self(self.0 | right.0)
    }
}

// cpp: layoutng/internal/layout_input.h:182-204
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathStyle {
    kNormal,
    kCompact,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathShift {
    kNormal,
    kCompact,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RubyPosition {
    kOver,
    kUnder,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RubyAlign {
    kCenter,
    kStart,
    kSpaceBetween,
    kSpaceAround,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RubyOverhang {
    kNone,
    kAuto,
    kSpaces,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrapMode {
    kFromWhiteSpace,
    kWrap,
    kNowrap,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrapStyle {
    kAuto,
    kPretty,
    kBalance,
    kStable,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgLengthAdjust {
    kUnknown,
    kSpacing,
    kSpacingAndGlyphs,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnicodeBidi {
    kNormal,
    kEmbed,
    kOverride,
    kIsolate,
    kIsolateOverride,
    kPlaintext,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakRule {
    kAuto,
    kAvoid,
    kAlways,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnWrap {
    kAuto,
    kNowrap,
    kWrap,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitialLetterSink {
    kOmitted,
    kInteger,
    kDrop,
    kRaise,
}

// cpp: layoutng/internal/layout_input.h:205-211
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitialLetter {
    pub size: f64,
    pub sink_type: InitialLetterSink,
    pub sink: u32,
}
impl Default for InitialLetter {
    fn default() -> Self {
        Self {
            size: 0.0,
            sink_type: InitialLetterSink::kOmitted,
            sink: 0,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:212-247
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeReferenceBox {
    kMarginBox,
    kBorderBox,
    kPaddingBox,
    kContentBox,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShapeCoordinate {
    pub pixels: f64,
    pub percentage: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShapePoint {
    pub x: ShapeCoordinate,
    pub y: ShapeCoordinate,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeOutsideKind {
    kReferenceBox,
    kPolygon,
    kCircle,
    kEllipse,
    kInset,
    kImage,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeRadiusKind {
    kLengthPercentage,
    kClosestSide,
    kFarthestSide,
    kClosestCorner,
    kFarthestCorner,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeRadius {
    pub kind: ShapeRadiusKind,
    pub value: ShapeCoordinate,
}
impl Default for ShapeRadius {
    fn default() -> Self {
        Self {
            kind: ShapeRadiusKind::kClosestSide,
            value: ShapeCoordinate::default(),
        }
    }
}

// cpp: layoutng/internal/layout_input.h:249-266
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeOutsideGeometry {
    pub kind: ShapeOutsideKind,
    pub reference_box: ShapeReferenceBox,
    pub contour: Vec<ShapePoint>,
    pub center: ShapePoint,
    pub radius_x: ShapeRadius,
    pub radius_y: ShapeRadius,
    pub insets: [ShapeCoordinate; 4],
    pub corner_radii: [ShapePoint; 4],
    pub resource_id: u64,
}
impl Default for ShapeOutsideGeometry {
    fn default() -> Self {
        Self {
            kind: ShapeOutsideKind::kReferenceBox,
            reference_box: ShapeReferenceBox::kMarginBox,
            contour: Vec::new(),
            center: ShapePoint {
                x: ShapeCoordinate {
                    pixels: 0.0,
                    percentage: 50.0,
                },
                y: ShapeCoordinate {
                    pixels: 0.0,
                    percentage: 50.0,
                },
            },
            radius_x: ShapeRadius::default(),
            radius_y: ShapeRadius::default(),
            insets: [ShapeCoordinate::default(); 4],
            corner_radii: [ShapePoint::default(); 4],
            resource_id: 0,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:267-300
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearSide {
    kNone,
    kLeft,
    kRight,
    kInlineStart,
    kInlineEnd,
    kBoth,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptionSide {
    kTop,
    kBottom,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptyCells {
    kHide,
    kShow,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MarginTrim(pub u8);
impl MarginTrim {
    pub const kNone: Self = Self(0);
    pub const kBlockStart: Self = Self(1 << 0);
    pub const kBlockEnd: Self = Self(1 << 1);
    pub const kBlock: Self = Self(3);
}
impl std::ops::BitOr for MarginTrim {
    type Output = Self;
    fn bitor(self, right: Self) -> Self {
        Self(self.0 | right.0)
    }
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxDecorationBreak {
    kSlice,
    kClone,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    kVisible,
    kHidden,
    kScroll,
    kAuto,
    kClip,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowClipReferenceBox {
    kBorderBox,
    kPaddingBox,
    kContentBox,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverflowClipMargin {
    pub reference_box: OverflowClipReferenceBox,
    pub margin: f64,
}
impl Default for OverflowClipMargin {
    fn default() -> Self {
        Self {
            reference_box: OverflowClipReferenceBox::kPaddingBox,
            margin: 0.0,
        }
    }
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarWidth {
    kAuto,
    kThin,
    kNone,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarGutter {
    kAuto,
    kStable,
    kStableBothEdges,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldSizing {
    kFixed,
    kContent,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxSizing {
    kContentBox,
    kBorderBox,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStyleType {
    kDisc,
    kCircle,
    kSquare,
    kDecimal,
    kLowerAlpha,
    kNone,
}

// cpp: layoutng/internal/layout_input.h:301-303
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListStylePosition {
    kOutside,
    kInside,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompatibilityMode {
    kStandards,
    kLimitedQuirks,
    kQuirks,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentRole {
    kNone,
    kDocumentElement,
    kBody,
    kDocumentElementAndBody,
}

// cpp: layoutng/internal/layout_input.h:304-326
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridTrackBreadthKind {
    kAuto,
    kFixed,
    kPercentage,
    kCalculated,
    kMinContent,
    kMaxContent,
    kFlex,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridTrackBreadth {
    pub kind: GridTrackBreadthKind,
    pub pixels: f64,
    pub percentage: f64,
}
impl Default for GridTrackBreadth {
    fn default() -> Self {
        Self {
            kind: GridTrackBreadthKind::kAuto,
            pixels: 0.0,
            percentage: 0.0,
        }
    }
}
impl GridTrackBreadth {
    #[allow(non_snake_case)]
    pub fn Fixed(value: f64) -> Self {
        Self {
            kind: GridTrackBreadthKind::kFixed,
            pixels: value,
            ..Self::default()
        }
    }
    #[allow(non_snake_case)]
    pub fn Flex(value: f64) -> Self {
        Self {
            kind: GridTrackBreadthKind::kFlex,
            pixels: value,
            ..Self::default()
        }
    }
}

// cpp: layoutng/internal/layout_input.h:328-343
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GridTrack {
    pub minimum: GridTrackBreadth,
    pub maximum: GridTrackBreadth,
    pub fit_content: bool,
}
impl GridTrack {
    #[allow(non_snake_case)]
    pub fn Fixed(value: f64) -> Self {
        let breadth = GridTrackBreadth::Fixed(value);
        Self {
            minimum: breadth,
            maximum: breadth,
            ..Self::default()
        }
    }
    #[allow(non_snake_case)]
    pub fn Fraction(value: f64) -> Self {
        Self {
            maximum: GridTrackBreadth::Flex(value),
            ..Self::default()
        }
    }
}

// cpp: layoutng/internal/layout_input.h:345-354
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridRepeatKind {
    kNone,
    kInteger,
    kAutoFill,
    kAutoFit,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GridTrackRepeater {
    pub kind: GridRepeatKind,
    pub count: u32,
    pub tracks: Vec<GridTrack>,
    pub line_names: Vec<Vec<String>>,
}
impl Default for GridTrackRepeater {
    fn default() -> Self {
        Self {
            kind: GridRepeatKind::kNone,
            count: 1,
            tracks: Vec::new(),
            line_names: Vec::new(),
        }
    }
}

// cpp: layoutng/internal/layout_input.h:356-378
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridNamedLineInput {
    pub name: String,
    pub position: u32,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridTrackListInput {
    pub repeaters: Vec<GridTrackRepeater>,
    pub line_names: Vec<GridNamedLineInput>,
}
impl GridTrackListInput {
    pub fn from_tracks(tracks: &[GridTrack]) -> Self {
        let mut result = Self::default();
        if !tracks.is_empty() {
            result.repeaters.push(GridTrackRepeater {
                tracks: tracks.to_vec(),
                ..GridTrackRepeater::default()
            });
        }
        result
    }
    pub fn empty(&self) -> bool {
        self.repeaters.is_empty()
    }
    pub fn clear(&mut self) {
        self.repeaters.clear();
    }
}
impl From<Vec<GridTrack>> for GridTrackListInput {
    fn from(tracks: Vec<GridTrack>) -> Self {
        Self::from_tracks(&tracks)
    }
}

// cpp: layoutng/internal/layout_input.h:380-397
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridLineKind {
    kAuto,
    kExplicit,
    kSpan,
    kNamedArea,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridLine {
    pub kind: GridLineKind,
    pub number: i32,
    pub name: String,
}
impl Default for GridLineKind {
    fn default() -> Self {
        Self::kAuto
    }
}
impl GridLine {
    #[allow(non_snake_case)]
    pub fn Explicit(value: i32) -> Self {
        Self {
            kind: GridLineKind::kExplicit,
            number: value,
            ..Self::default()
        }
    }
    #[allow(non_snake_case)]
    pub fn Span(value: i32) -> Self {
        Self {
            kind: GridLineKind::kSpan,
            number: value,
            ..Self::default()
        }
    }
}

// cpp: layoutng/internal/layout_input.h:399-413
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridTemplateArea {
    pub name: String,
    pub row_start: u32,
    pub row_end: u32,
    pub column_start: u32,
    pub column_end: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GridTemplateAreasInput {
    pub row_count: u32,
    pub column_count: u32,
    pub areas: Vec<GridTemplateArea>,
}

// cpp: layoutng/internal/layout_input.h:415-423
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ContainIntrinsicLength {
    pub has_auto: bool,
    pub length: Option<f64>,
}
// cpp: layoutng/internal/layout_input.h:425-618
#[derive(Clone, Debug, PartialEq)]
pub struct ExtendedStyle {
    pub effective_appearance: AppearanceValue,
    pub has_author_background: bool,
    pub has_author_border: bool,
    pub has_author_border_radius: bool,
    pub has_author_outline: bool,
    /// The element's own CSS zoom; effective_zoom includes its ancestors.
    pub zoom: f32,
    pub effective_zoom: f32,
    pub timeline_trigger_names: Vec<Option<String>>,
    pub trigger_scope_all: bool,
    pub trigger_scope_names: Vec<String>,
    pub scrollbar_width: ScrollbarWidth,
    pub scrollbar_gutter: ScrollbarGutter,
    pub prefers_default_scrollbar_styles: bool,
    pub width_percent: Option<f64>,
    pub height_percent: Option<f64>,
    pub min_width_percent: Option<f64>,
    pub max_width_percent: Option<f64>,
    pub min_height_percent: Option<f64>,
    pub max_height_percent: Option<f64>,
    pub flex_basis_percent: Option<f64>,
    pub width_calculated: bool,
    pub height_calculated: bool,
    pub min_width_calculated: bool,
    pub max_width_calculated: bool,
    pub min_height_calculated: bool,
    pub max_height_calculated: bool,
    pub flex_basis_calculated: bool,
    pub flex_basis_sizing: FlexBasisSizing,
    pub aspect_ratio: Option<f64>,
    pub width_sizing: IntrinsicSizing,
    pub height_sizing: IntrinsicSizing,
    pub min_width_sizing: IntrinsicSizing,
    pub max_width_sizing: IntrinsicSizing,
    pub min_height_sizing: IntrinsicSizing,
    pub max_height_sizing: IntrinsicSizing,
    pub justify_content: JustifyContent,
    pub align_content: Option<ContentAlignment>,
    pub align_content_block_center: bool,
    pub align_self: Option<SelfAlignment>,
    pub justify_items: Option<SelfAlignment>,
    pub justify_self: Option<SelfAlignment>,
    pub align_items_overflow: OverflowAlignment,
    pub justify_content_overflow: OverflowAlignment,
    pub align_content_overflow: OverflowAlignment,
    pub align_self_overflow: OverflowAlignment,
    pub justify_items_overflow: OverflowAlignment,
    pub justify_self_overflow: OverflowAlignment,
    pub justify_items_legacy: bool,
    pub inset_percentages: [Option<f64>; 4],
    pub inset_calculated: [bool; 4],
    pub order: i32,
    pub wrap_reverse: bool,
    pub margin_percentages: [Option<f64>; 4],
    pub padding_percentages: [Option<f64>; 4],
    pub margin_calculated: [bool; 4],
    pub padding_calculated: [bool; 4],
    pub margin_auto: [bool; 4],
    /// Blink's UA-only `__qem` marker. Quirky margins participate in the
    /// special body/heading/list margin-collapse rules in quirks mode.
    pub margin_quirks: [bool; 4],
    pub auto_margin_inline_start: bool,
    pub auto_margin_inline_end: bool,
    pub row_gap: Option<f64>,
    pub column_gap: Option<f64>,
    pub row_gap_percent: Option<f64>,
    pub column_gap_percent: Option<f64>,
    pub row_gap_calculated: bool,
    pub column_gap_calculated: bool,
    pub grid_columns: GridTrackListInput,
    pub grid_rows: GridTrackListInput,
    pub grid_auto_columns: GridTrackListInput,
    pub grid_auto_rows: GridTrackListInput,
    pub grid_template_areas: Option<GridTemplateAreasInput>,
    pub subgrid_columns: bool,
    pub subgrid_rows: bool,
    pub grid_auto_flow_column: bool,
    pub grid_auto_flow_dense: bool,
    pub grid_column_start: GridLine,
    pub grid_column_end: GridLine,
    pub grid_row_start: GridLine,
    pub grid_row_end: GridLine,
    pub table_layout_fixed: bool,
    pub border_collapse: bool,
    pub caption_side: CaptionSide,
    pub border_spacing: f64,
    pub vertical_border_spacing: Option<f64>,
    pub column_span_all: bool,
    pub explicit_column_count: bool,
    pub column_width: Option<f64>,
    pub column_fill_balance: bool,
    pub column_wrap: ColumnWrap,
    pub initial_letter: InitialLetter,
    pub shape_outside: Option<ShapeOutsideGeometry>,
    pub shape_margin: ShapeCoordinate,
    pub shape_image_threshold: f64,
    pub break_before: BreakRule,
    pub break_after: BreakRule,
    pub break_inside: BreakRule,
    pub widows: u32,
    pub orphans: u32,
    pub clear_side: ClearSide,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub overflow_clip_margin: Option<OverflowClipMargin>,
    pub layout_containment: bool,
    pub size_containment: bool,
    pub contain_intrinsic_width: ContainIntrinsicLength,
    pub contain_intrinsic_height: ContainIntrinsicLength,
    pub white_space: WhiteSpace,
    pub text_align: TextAlign,
    pub text_align_last: TextAlignLast,
    pub vertical_align: VerticalAlign,
    pub vertical_align_length: Option<f64>,
    pub vertical_align_percent: Option<f64>,
    pub vertical_align_calculated: bool,
    pub unicode_bidi: UnicodeBidi,
    pub font_size: f64,
    pub font_families: Vec<String>,
    pub font_weight: f64,
    pub font_italic: bool,
    pub font_smoothing: FontSmoothing,
    pub line_height: Option<f64>,
    pub line_height_percent: Option<f64>,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub text_indent: f64,
    pub text_indent_percent: Option<f64>,
    pub text_indent_calculated: bool,
    pub text_indent_each_line: bool,
    pub text_indent_hanging: bool,
    pub overflow_wrap: OverflowWrap,
    pub word_break: WordBreak,
    pub line_break: LineBreak,
    pub hyphens: Hyphens,
    pub tab_size: f64,
    pub tab_size_is_length: bool,
    pub text_orientation: TextOrientation,
    pub text_combine: TextCombine,
    pub text_transform: TextTransform,
    pub math_depth: i32,
    pub math_style: MathStyle,
    pub math_shift: MathShift,
    pub math_baseline: Option<f64>,
    pub math_padded_depth: Option<f64>,
    pub math_lspace: Option<f64>,
    pub math_padded_voffset: Option<f64>,
    pub math_fraction_bar_thickness: Option<f64>,
    pub math_rspace: Option<f64>,
    pub math_min_size: Option<f64>,
    pub math_max_size: Option<f64>,
    pub font_size_math: bool,
    pub text_wrap_mode: TextWrapMode,
    pub text_wrap_style: TextWrapStyle,
    pub ruby_position: RubyPosition,
    pub ruby_align: RubyAlign,
    pub ruby_overhang: RubyOverhang,
    pub line_clamp: u32,
    pub language: String,
    pub field_sizing: FieldSizing,
    pub box_sizing: BoxSizing,
    pub list_style_type: ListStyleType,
    pub list_style_position: ListStylePosition,
    pub empty_cells: EmptyCells,
    pub margin_trim: MarginTrim,
    pub box_decoration_break: BoxDecorationBreak,
}
impl Default for ExtendedStyle {
    // cpp: layoutng/internal/layout_input.h:425-618
    fn default() -> Self {
        Self {
            effective_appearance: AppearanceValue::kNone,
            has_author_background: false,
            has_author_border: false,
            has_author_border_radius: false,
            has_author_outline: false,
            zoom: 1.0,
            effective_zoom: 1.0,
            timeline_trigger_names: Vec::new(),
            trigger_scope_all: false,
            trigger_scope_names: Vec::new(),
            scrollbar_width: ScrollbarWidth::kAuto,
            scrollbar_gutter: ScrollbarGutter::kAuto,
            prefers_default_scrollbar_styles: false,
            width_percent: None,
            height_percent: None,
            min_width_percent: None,
            max_width_percent: None,
            min_height_percent: None,
            max_height_percent: None,
            flex_basis_percent: None,
            width_calculated: false,
            height_calculated: false,
            min_width_calculated: false,
            max_width_calculated: false,
            min_height_calculated: false,
            max_height_calculated: false,
            flex_basis_calculated: false,
            flex_basis_sizing: FlexBasisSizing::kAuto,
            aspect_ratio: None,
            width_sizing: IntrinsicSizing::kAuto,
            height_sizing: IntrinsicSizing::kAuto,
            min_width_sizing: IntrinsicSizing::kAuto,
            max_width_sizing: IntrinsicSizing::kAuto,
            min_height_sizing: IntrinsicSizing::kAuto,
            max_height_sizing: IntrinsicSizing::kAuto,
            justify_content: JustifyContent::kNormal,
            align_content: None,
            align_content_block_center: false,
            align_self: None,
            justify_items: None,
            justify_self: None,
            align_items_overflow: OverflowAlignment::kDefault,
            justify_content_overflow: OverflowAlignment::kDefault,
            align_content_overflow: OverflowAlignment::kDefault,
            align_self_overflow: OverflowAlignment::kDefault,
            justify_items_overflow: OverflowAlignment::kDefault,
            justify_self_overflow: OverflowAlignment::kDefault,
            justify_items_legacy: false,
            inset_percentages: [None; 4],
            inset_calculated: [false; 4],
            order: 0,
            wrap_reverse: false,
            margin_percentages: [None; 4],
            padding_percentages: [None; 4],
            margin_calculated: [false; 4],
            padding_calculated: [false; 4],
            margin_auto: [false; 4],
            margin_quirks: [false; 4],
            auto_margin_inline_start: false,
            auto_margin_inline_end: false,
            row_gap: None,
            column_gap: None,
            row_gap_percent: None,
            column_gap_percent: None,
            row_gap_calculated: false,
            column_gap_calculated: false,
            grid_columns: GridTrackListInput::default(),
            grid_rows: GridTrackListInput::default(),
            grid_auto_columns: GridTrackListInput::default(),
            grid_auto_rows: GridTrackListInput::default(),
            grid_template_areas: None,
            subgrid_columns: false,
            subgrid_rows: false,
            grid_auto_flow_column: false,
            grid_auto_flow_dense: false,
            grid_column_start: GridLine::default(),
            grid_column_end: GridLine::default(),
            grid_row_start: GridLine::default(),
            grid_row_end: GridLine::default(),
            table_layout_fixed: false,
            border_collapse: false,
            caption_side: CaptionSide::kTop,
            border_spacing: 0.0,
            vertical_border_spacing: None,
            column_span_all: false,
            explicit_column_count: false,
            column_width: None,
            column_fill_balance: true,
            column_wrap: ColumnWrap::kAuto,
            initial_letter: InitialLetter::default(),
            shape_outside: None,
            shape_margin: ShapeCoordinate::default(),
            shape_image_threshold: 0.0,
            break_before: BreakRule::kAuto,
            break_after: BreakRule::kAuto,
            break_inside: BreakRule::kAuto,
            widows: 2,
            orphans: 2,
            clear_side: ClearSide::kNone,
            overflow_x: Overflow::kVisible,
            overflow_y: Overflow::kVisible,
            overflow_clip_margin: None,
            layout_containment: false,
            size_containment: false,
            contain_intrinsic_width: ContainIntrinsicLength::default(),
            contain_intrinsic_height: ContainIntrinsicLength::default(),
            white_space: WhiteSpace::kNormal,
            text_align: TextAlign::kStart,
            text_align_last: TextAlignLast::kAuto,
            vertical_align: VerticalAlign::kBaseline,
            vertical_align_length: None,
            vertical_align_percent: None,
            vertical_align_calculated: false,
            unicode_bidi: UnicodeBidi::kNormal,
            font_size: 16.0,
            font_families: Vec::new(),
            font_weight: 400.0,
            font_italic: false,
            font_smoothing: FontSmoothing::kAuto,
            line_height: None,
            line_height_percent: None,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            text_indent: 0.0,
            text_indent_percent: None,
            text_indent_calculated: false,
            text_indent_each_line: false,
            text_indent_hanging: false,
            overflow_wrap: OverflowWrap::kNormal,
            word_break: WordBreak::kNormal,
            line_break: LineBreak::kAuto,
            hyphens: Hyphens::kManual,
            tab_size: 8.0,
            tab_size_is_length: false,
            text_orientation: TextOrientation::kMixed,
            text_combine: TextCombine::kNone,
            text_transform: TextTransform::kNone,
            math_depth: 0,
            math_style: MathStyle::kNormal,
            math_shift: MathShift::kNormal,
            math_baseline: None,
            math_padded_depth: None,
            math_lspace: None,
            math_padded_voffset: None,
            math_fraction_bar_thickness: None,
            math_rspace: None,
            math_min_size: None,
            math_max_size: None,
            font_size_math: false,
            text_wrap_mode: TextWrapMode::kFromWhiteSpace,
            text_wrap_style: TextWrapStyle::kAuto,
            ruby_position: RubyPosition::kOver,
            ruby_align: RubyAlign::kSpaceAround,
            ruby_overhang: RubyOverhang::kAuto,
            line_clamp: 0,
            language: String::new(),
            field_sizing: FieldSizing::kFixed,
            box_sizing: BoxSizing::kContentBox,
            list_style_type: ListStyleType::kDisc,
            list_style_position: ListStylePosition::kOutside,
            empty_cells: EmptyCells::kShow,
            margin_trim: MarginTrim::kNone,
            box_decoration_break: BoxDecorationBreak::kSlice,
        }
    }
}
impl ExtendedStyle {
    // cpp: layoutng/internal/layout_input.h:547-547
    #[allow(non_snake_case)]
    pub fn SetOverflow(&mut self, value: Overflow) {
        self.overflow_y = value;
        self.overflow_x = value;
    }
}

// cpp: layoutng/internal/layout_input.h:620-621
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathTokenKind {
    kIdentifier,
    kNumber,
    kText,
    kString,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathOperatorForm {
    kPrefix,
    kInfix,
    kPostfix,
}

// cpp: layoutng/internal/layout_input.h:623-633
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameDimensionType {
    kRelative,
    kPercentage,
    kAbsolute,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameDimension {
    pub value: f64,
    pub r#type: FrameDimensionType,
}
impl Default for FrameDimension {
    fn default() -> Self {
        Self {
            value: 0.0,
            r#type: FrameDimensionType::kRelative,
        }
    }
}
impl FrameDimension {
    pub fn from_weight(weight: f64) -> Self {
        Self {
            value: weight,
            ..Self::default()
        }
    }
    pub fn new(value: f64, dimension_type: FrameDimensionType) -> Self {
        Self {
            value,
            r#type: dimension_type,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:635-639
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Offset {
    pub x: f64,
    pub y: f64,
}

// cpp: layoutng/internal/layout_input.h:641-653
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderLineStyle {
    kNone,
    kHidden,
    kSolid,
    kDashed,
    kDotted,
    kDouble,
    kGroove,
    kRidge,
    kInset,
    kOutset,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectFit {
    kFill,
    kContain,
    kCover,
    kNone,
    kScaleDown,
}

// cpp: layoutng/internal/layout_input.h:655-677
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintShadow {
    pub offset: Offset,
    pub blur_radius: f64,
    pub spread: f64,
    pub color: Color,
    pub inset: bool,
}
impl Default for PaintShadow {
    fn default() -> Self {
        Self {
            offset: Offset::default(),
            blur_radius: 0.0,
            spread: 0.0,
            color: Color::default(),
            inset: false,
        }
    }
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDecorationStyle {
    kSolid,
    kDouble,
    kDotted,
    kDashed,
    kWavy,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextDecorationPaint {
    pub underline: bool,
    pub overline: bool,
    pub line_through: bool,
    pub color: Option<Color>,
    pub style: TextDecorationStyle,
    pub thickness: Option<f64>,
    pub underline_offset: f64,
    pub underline_offset_auto: bool,
    pub skip_ink: bool,
}
impl Default for TextDecorationPaint {
    fn default() -> Self {
        Self {
            underline: false,
            overline: false,
            line_through: false,
            color: None,
            style: TextDecorationStyle::kSolid,
            thickness: None,
            underline_offset: 0.0,
            underline_offset_auto: true,
            skip_ink: true,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:679-685
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformMatrix {
    pub values: [f64; 16],
}
impl Default for TransformMatrix {
    fn default() -> Self {
        Self {
            values: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

// cpp: layoutng/internal/layout_input.h:687-702
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintPathVerb {
    kMoveTo,
    kLineTo,
    kQuadraticTo,
    kConicTo,
    kCubicTo,
    kClose,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintPathCommand {
    pub verb: PaintPathVerb,
    pub control1: Offset,
    pub control2: Offset,
    pub point: Offset,
    pub conic_weight: f64,
}
impl Default for PaintPathCommand {
    fn default() -> Self {
        Self {
            verb: PaintPathVerb::kMoveTo,
            control1: Offset::default(),
            control2: Offset::default(),
            point: Offset::default(),
            conic_weight: 1.0,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:704-718
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SvgShapeGeometry {
    kPath,
    kRectangle,
    kEllipse,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SvgShapeData {
    pub geometry: SvgShapeGeometry,
    pub bounds_offset: Offset,
    pub bounds_width: f64,
    pub bounds_height: f64,
    pub rectangle_radius_x: f64,
    pub rectangle_radius_y: f64,
    pub path: Vec<PaintPathCommand>,
    pub local_transform: Option<TransformMatrix>,
}
impl Default for SvgShapeData {
    fn default() -> Self {
        Self {
            geometry: SvgShapeGeometry::kPath,
            bounds_offset: Offset::default(),
            bounds_width: 0.0,
            bounds_height: 0.0,
            rectangle_radius_x: 0.0,
            rectangle_radius_y: 0.0,
            path: Vec::new(),
            local_transform: None,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:720-732
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SvgViewBoxData {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub preserve_none: bool,
    pub slice: bool,
    pub align_x: i8,
    pub align_y: i8,
}

// cpp: layoutng/internal/layout_input.h:734-754
// The concrete PaintStyleData is defined by paint_input.h in this package.
// Mutable dereference detaches shared storage, as the C++ accessors do.
#[derive(Clone, PartialEq)]
pub struct PaintStyle {
    pub(super) data: std::sync::Arc<super::paint_input::PaintStyleData>,
}

// cpp: layoutng/internal/layout_input.h:756-767
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverscrollType {
    kNone,
    kTransform,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewportGeometry {
    pub size: IntSize,
    pub root_scroller_visible_size: Option<IntSize>,
    pub transition_snapshot_size: Option<IntSize>,
    pub overscroll_type: OverscrollType,
}
impl Default for ViewportGeometry {
    fn default() -> Self {
        Self {
            size: IntSize::default(),
            root_scroller_visible_size: None,
            transition_snapshot_size: None,
            overscroll_type: OverscrollType::kNone,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:769-778
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentLockState {
    pub is_locked: bool,
    pub should_layout_children: bool,
}
impl ContentLockState {
    pub const fn new(locked: bool, layout_children: bool) -> Self {
        Self {
            is_locked: locked,
            should_layout_children: layout_children,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:780-789
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFieldSizing {
    pub preferred_size: i32,
    pub includes_decoration: bool,
}
impl Default for TextFieldSizing {
    fn default() -> Self {
        Self {
            preferred_size: 20,
            includes_decoration: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextAreaSizing {
    pub columns: u32,
    pub rows: u32,
}
impl Default for TextAreaSizing {
    fn default() -> Self {
        Self {
            columns: 20,
            rows: 2,
        }
    }
}
// cpp: layoutng/internal/layout_input.h:791-942
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SvgTextPathData {
    pub points: Vec<Offset>,
    pub start_offset: f64,
    pub reverse_direction: bool,
}
#[derive(Clone)]
pub struct ElementData {
    pub used_canvas_transform: Option<TransformMatrix>,
    pub content_lock: Option<ContentLockState>,
    pub remembered_inline_size: Option<f64>,
    pub remembered_block_size: Option<f64>,
    pub nowrap_attribute: bool,
    pub image_map_area: bool,
    pub html_image: bool,
    pub html_ordered_or_unordered_list: bool,
    pub first_letter_pseudo: bool,
    pub viewport_defining: bool,
    pub root_editable: bool,
    pub implicit_anchor_id: u64,
    pub scroll_offset: Offset,
    pub root_scroller_visible_size: Option<IntSize>,
    pub natural_width: Option<f64>,
    pub natural_height: Option<f64>,
    pub natural_aspect_ratio: Option<f64>,
    pub image_device_pixel_ratio: f64,
    pub replaced_respects_css_overflow: bool,
    pub row_span: u32,
    pub column_span: u32,
    pub stretchy: Option<bool>,
    pub symmetric: Option<bool>,
    pub large_op: Option<bool>,
    pub movable_limits: Option<bool>,
    pub accent: Option<bool>,
    pub accent_under: Option<bool>,
    pub math_token_kind: MathTokenKind,
    pub math_operator_form: Option<MathOperatorForm>,
    pub frame_rows: Vec<FrameDimension>,
    pub frame_columns: Vec<FrameDimension>,
    pub frame_row_deltas: Vec<i32>,
    pub frame_column_deltas: Vec<i32>,
    pub frame_border_thickness: Option<i32>,
    pub frame_border: Option<bool>,
    pub frame_no_resize: bool,
    pub frame_has_border_color: bool,
    pub svg_x: Vec<f64>,
    pub svg_y: Vec<f64>,
    pub svg_dx: Vec<f64>,
    pub svg_dy: Vec<f64>,
    pub svg_rotate: Vec<f64>,
    pub svg_shape: Option<SvgShapeData>,
    pub svg_view_box: Option<SvgViewBoxData>,
    pub svg_text_length: Option<f64>,
    pub svg_text_path: Option<SvgTextPathData>,
    pub svg_length_adjust: SvgLengthAdjust,
    pub custom_layout_name: String,
    pub custom_layout: Option<std::sync::Arc<dyn layoutng_custom::custom_layout_api::CustomLayout>>,
    pub form_control_type: Option<FormControlType>,
    pub select_uses_menu_list: Option<bool>,
    pub control_checked: bool,
    pub control_indeterminate: bool,
    pub control_disabled: bool,
    pub control_read_only: bool,
    pub control_hovered: bool,
    pub control_active: bool,
    pub control_focused: bool,
    pub control_value_ratio: Option<f64>,
    pub supports_base_appearance: bool,
    pub selected_file_count: u32,
    pub file_no_file_label: Option<String>,
    pub file_upload_button: bool,
    pub text_field_sizing: Option<TextFieldSizing>,
    pub text_area_sizing: Option<TextAreaSizing>,
    pub text_control_inner_editor: bool,
    pub text_control_container: bool,
    pub text_control_spin_button: bool,
    pub text_control_placeholder: bool,
    pub autofill_state: AutofillState,
    pub range_value_ratio: Option<f64>,
    pub control_host_ancestor: Option<u32>,
    pub marquee_horizontal: bool,
    pub document_role: DocumentRole,
}
impl PartialEq for ElementData {
    fn eq(&self, other: &Self) -> bool {
        (self.used_canvas_transform == other.used_canvas_transform)
            && (self.content_lock == other.content_lock)
            && (self.remembered_inline_size == other.remembered_inline_size)
            && (self.remembered_block_size == other.remembered_block_size)
            && (self.nowrap_attribute == other.nowrap_attribute)
            && (self.image_map_area == other.image_map_area)
            && (self.html_image == other.html_image)
            && (self.html_ordered_or_unordered_list == other.html_ordered_or_unordered_list)
            && (self.first_letter_pseudo == other.first_letter_pseudo)
            && (self.viewport_defining == other.viewport_defining)
            && (self.root_editable == other.root_editable)
            && (self.implicit_anchor_id == other.implicit_anchor_id)
            && (self.scroll_offset == other.scroll_offset)
            && (self.root_scroller_visible_size == other.root_scroller_visible_size)
            && (self.natural_width == other.natural_width)
            && (self.natural_height == other.natural_height)
            && (self.natural_aspect_ratio == other.natural_aspect_ratio)
            && (self.image_device_pixel_ratio == other.image_device_pixel_ratio)
            && (self.replaced_respects_css_overflow == other.replaced_respects_css_overflow)
            && (self.row_span == other.row_span)
            && (self.column_span == other.column_span)
            && (self.stretchy == other.stretchy)
            && (self.symmetric == other.symmetric)
            && (self.large_op == other.large_op)
            && (self.movable_limits == other.movable_limits)
            && (self.accent == other.accent)
            && (self.accent_under == other.accent_under)
            && (self.math_token_kind == other.math_token_kind)
            && (self.math_operator_form == other.math_operator_form)
            && (self.frame_rows == other.frame_rows)
            && (self.frame_columns == other.frame_columns)
            && (self.frame_row_deltas == other.frame_row_deltas)
            && (self.frame_column_deltas == other.frame_column_deltas)
            && (self.frame_border_thickness == other.frame_border_thickness)
            && (self.frame_border == other.frame_border)
            && (self.frame_no_resize == other.frame_no_resize)
            && (self.frame_has_border_color == other.frame_has_border_color)
            && (self.svg_x == other.svg_x)
            && (self.svg_y == other.svg_y)
            && (self.svg_dx == other.svg_dx)
            && (self.svg_dy == other.svg_dy)
            && (self.svg_rotate == other.svg_rotate)
            && (self.svg_shape == other.svg_shape)
            && (self.svg_view_box == other.svg_view_box)
            && (self.svg_text_length == other.svg_text_length)
            && (self.svg_text_path == other.svg_text_path)
            && (self.svg_length_adjust == other.svg_length_adjust)
            && (self.custom_layout_name == other.custom_layout_name)
            && (match (&self.custom_layout, &other.custom_layout) {
                (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            })
            && (self.form_control_type == other.form_control_type)
            && (self.select_uses_menu_list == other.select_uses_menu_list)
            && (self.control_checked == other.control_checked)
            && (self.control_indeterminate == other.control_indeterminate)
            && (self.control_disabled == other.control_disabled)
            && (self.control_read_only == other.control_read_only)
            && (self.control_hovered == other.control_hovered)
            && (self.control_active == other.control_active)
            && (self.control_focused == other.control_focused)
            && (self.control_value_ratio == other.control_value_ratio)
            && (self.supports_base_appearance == other.supports_base_appearance)
            && (self.selected_file_count == other.selected_file_count)
            && (self.file_no_file_label == other.file_no_file_label)
            && (self.file_upload_button == other.file_upload_button)
            && (self.text_field_sizing == other.text_field_sizing)
            && (self.text_area_sizing == other.text_area_sizing)
            && (self.text_control_inner_editor == other.text_control_inner_editor)
            && (self.text_control_container == other.text_control_container)
            && (self.text_control_spin_button == other.text_control_spin_button)
            && (self.text_control_placeholder == other.text_control_placeholder)
            && (self.autofill_state == other.autofill_state)
            && (self.range_value_ratio == other.range_value_ratio)
            && (self.control_host_ancestor == other.control_host_ancestor)
            && (self.marquee_horizontal == other.marquee_horizontal)
            && (self.document_role == other.document_role)
    }
}

impl Default for ElementData {
    // cpp: layoutng/internal/layout_input.h:791-942
    fn default() -> Self {
        Self {
            used_canvas_transform: None,
            content_lock: None,
            remembered_inline_size: None,
            remembered_block_size: None,
            nowrap_attribute: false,
            image_map_area: false,
            html_image: false,
            html_ordered_or_unordered_list: false,
            first_letter_pseudo: false,
            viewport_defining: false,
            root_editable: false,
            implicit_anchor_id: 0,
            scroll_offset: Offset::default(),
            root_scroller_visible_size: None,
            natural_width: None,
            natural_height: None,
            natural_aspect_ratio: None,
            image_device_pixel_ratio: 1.0,
            replaced_respects_css_overflow: false,
            row_span: 1,
            column_span: 1,
            stretchy: None,
            symmetric: None,
            large_op: None,
            movable_limits: None,
            accent: None,
            accent_under: None,
            math_token_kind: MathTokenKind::kIdentifier,
            math_operator_form: None,
            frame_rows: Vec::new(),
            frame_columns: Vec::new(),
            frame_row_deltas: Vec::new(),
            frame_column_deltas: Vec::new(),
            frame_border_thickness: None,
            frame_border: None,
            frame_no_resize: false,
            frame_has_border_color: false,
            svg_x: Vec::new(),
            svg_y: Vec::new(),
            svg_dx: Vec::new(),
            svg_dy: Vec::new(),
            svg_rotate: Vec::new(),
            svg_shape: None,
            svg_view_box: None,
            svg_text_length: None,
            svg_text_path: None,
            svg_length_adjust: SvgLengthAdjust::kSpacing,
            custom_layout_name: String::new(),
            custom_layout: None,
            form_control_type: None,
            select_uses_menu_list: None,
            control_checked: false,
            control_indeterminate: false,
            control_disabled: false,
            control_read_only: false,
            control_hovered: false,
            control_active: false,
            control_focused: false,
            control_value_ratio: None,
            supports_base_appearance: false,
            selected_file_count: 0,
            file_no_file_label: None,
            file_upload_button: false,
            text_field_sizing: None,
            text_area_sizing: None,
            text_control_inner_editor: false,
            text_control_container: false,
            text_control_spin_button: false,
            text_control_placeholder: false,
            autofill_state: AutofillState::kNotFilled,
            range_value_ratio: None,
            control_host_ancestor: None,
            marquee_horizontal: true,
            document_role: DocumentRole::kNone,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:944-986
/// Resolved CSS 2.1 `clip: rect(...)` sides in CSS pixels. `None` on an
/// individual side represents the legacy `auto` keyword; `ComputedStyle`'s
/// outer `Option` distinguishes the whole-property `clip: auto` value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CssClipRect {
    pub top: Option<f64>,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub left: Option<f64>,
}

#[derive(Clone, PartialEq)]
pub struct ComputedStyle {
    /// The Chromium `ComputedStyle` produced by StyleEngine for this
    /// projection.  DOM-to-layout metadata below remains the renderer-neutral
    /// projection consumed by the Rust paint boundary; layout itself reuses
    /// this exact immutable style object instead of reconstructing one.
    pub native_style: *const NativeComputedStyle,
    pub display: Display,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,
    pub flex_basis: Option<f64>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub min_width: Option<f64>,
    pub max_width: Option<f64>,
    pub margin: Edges,
    pub border: Edges,
    pub border_styles: [BorderLineStyle; 4],
    pub padding: Edges,
    pub gap: f64,
    pub flex_direction: FlexDirection,
    pub flex_wrap: bool,
    pub align_items: AlignItems,
    pub direction: TextDirection,
    pub writing_mode: WritingMode,
    pub position: Position,
    pub left: Option<f64>,
    pub top: Option<f64>,
    pub right: Option<f64>,
    pub bottom: Option<f64>,
    pub css_clip: Option<CssClipRect>,
    pub floating: FloatSide,
    pub column_count: u32,
    pub grid_template_columns: Vec<f64>,
    pub extended: Option<ExtendedStyle>,
    pub paint: PaintStyle,
}
impl ComputedStyle {
    /// Compare the inputs consumed by layout. Paint data may require paint or
    /// paint-property invalidation, but changing its value does not ordinarily
    /// invalidate geometry. Keep the few transitions which change the native
    /// box/containing-block model in this implementation.
    pub fn LayoutEquivalent(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }
        // Adding/removing a background can change whether an inline needs a
        // physical box fragment, even though its CSS dimensions are unchanged.
        if (self.paint.background_color.alpha > 0.0) != (other.paint.background_color.alpha > 0.0) {
            return false;
        }
        // Blink treats ordinary transform changes, including adding/removing
        // the transform property node, as paint-property updates. LayoutBox
        // only escalates them to layout for anchor-positioning fragments whose
        // propagated anchor geometry depends on the transform. Rechrom does
        // not yet expose that anchor dependency, so making every transform
        // animation establish/remove a layout boundary is both more
        // conservative than Chromium and turns staggered animation completion
        // into repeated full-document layouts.
        //
        // Filter presence still changes the native layer/containing-block
        // model represented by this translated layout tree.
        if self.paint.filters.is_empty() != other.paint.filters.is_empty() {
            return false;
        }
        let mut normalized = self.clone();
        normalized.paint = other.paint.clone();
        // The native pointer is an identity/revision signal for the complete
        // input comparison above.  It is not independently a geometry change:
        // the projected fields decide LayoutEquivalent until native style
        // difference calculation owns this boundary end-to-end.
        normalized.native_style = other.native_style;
        normalized == *other
    }
}

#[cfg(test)]
mod computed_style_layout_equivalence_tests {
    use super::*;
    use crate::internal::paint_input::{PaintTransform, PaintTransformOperation};

    #[test]
    fn opacity_and_existing_transform_value_are_paint_only() {
        let mut old = ComputedStyle::default();
        let mut new = old.clone();
        new.paint.opacity = 0.4;
        assert!(old.LayoutEquivalent(&new));

        old.paint.transform = Some(PaintTransform {
            operations: vec![PaintTransformOperation::default()],
        });
        new = old.clone();
        new.paint.transform.as_mut().unwrap().operations[0].pixels.x = 42.0;
        assert!(old.LayoutEquivalent(&new));
    }

    #[test]
    fn transform_property_nodes_are_paint_only() {
        let old = ComputedStyle::default();
        let mut transformed = old.clone();
        transformed.paint.transform = Some(PaintTransform::default());
        assert!(old.LayoutEquivalent(&transformed));
    }

    #[test]
    fn native_layer_and_inline_box_transitions_require_layout() {
        let old = ComputedStyle::default();
        let mut painted_background = old.clone();
        painted_background.paint.background_color.alpha = 1.0;
        assert!(!old.LayoutEquivalent(&painted_background));

        let mut filtered = old.clone();
        filtered.paint.filters.push(Default::default());
        assert!(!old.LayoutEquivalent(&filtered));
    }
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            native_style: std::ptr::null(),
            display: Display::kBlock,
            width: None,
            height: None,
            min_height: None,
            max_height: None,
            flex_basis: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            min_width: None,
            max_width: None,
            margin: Edges::default(),
            border: Edges::default(),
            border_styles: [BorderLineStyle::kSolid; 4],
            padding: Edges::default(),
            gap: 0.0,
            flex_direction: FlexDirection::kRow,
            flex_wrap: false,
            align_items: AlignItems::kNormal,
            direction: TextDirection::kLtr,
            writing_mode: WritingMode::kHorizontalTb,
            position: Position::kStatic,
            left: None,
            top: None,
            right: None,
            bottom: None,
            css_clip: None,
            floating: FloatSide::kNone,
            column_count: 1,
            grid_template_columns: Vec::new(),
            extended: None,
            paint: PaintStyle::default(),
        }
    }
}

// cpp: layoutng/internal/layout_input.h:988-1002
#[derive(Clone, Default, PartialEq)]
pub struct NativeNodeConstructionData {
    pub id: u64,
    pub kind: NodeKind,
    pub style_generated: bool,
    pub style: ComputedStyle,
    pub text: String,
    pub element: Option<ElementData>,
    pub debug_name: String,
    pub first_line_style: Option<ComputedStyle>,
}
impl Default for NodeKind {
    fn default() -> Self {
        Self::kBox
    }
}

// cpp: layoutng/internal/layout_input.h:1006-1010
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

// cpp: layoutng/internal/layout_input.h:1018-1023
pub use font_engine::FontVariation;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontUnicodeRange {
    pub start: u32,
    pub end: u32,
}
impl Default for FontUnicodeRange {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0x10ffff,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:1031-1052
#[derive(Clone, Debug, PartialEq)]
pub struct FontFace {
    pub family: String,
    pub native_family: String,
    pub metrics_family: String,
    pub weight: f64,
    pub italic: bool,
    // Shared immutable bytes: aliases and layout snapshots must not copy font files.
    pub bytes: font_engine::SharedFontBytes,
    pub face_index: u32,
    pub variations: Vec<FontVariation>,
    pub unicode_ranges: Vec<FontUnicodeRange>,
}
impl Default for FontFace {
    fn default() -> Self {
        Self {
            family: String::new(),
            native_family: String::new(),
            metrics_family: String::new(),
            weight: 400.0,
            italic: false,
            bytes: font_engine::SharedFontBytes::default(),
            face_index: 0,
            variations: Vec::new(),
            unicode_ranges: Vec::new(),
        }
    }
}

// cpp: layoutng/internal/layout_input.h:1054-1064
#[derive(Clone, Debug, PartialEq)]
pub struct PaintImage {
    pub id: u64,
    pub revision: u64,
    pub width: u32,
    pub height: u32,
    pub resolution_scale: f64,
    /// Immutable bitmap pixels or a renderer-neutral document recording.
    /// Layout reads only identity and intrinsic dimensions.
    pub content: image_resource::PaintImageContent,
}
impl Default for PaintImage {
    fn default() -> Self {
        Self {
            id: 0,
            revision: 0,
            width: 0,
            height: 0,
            resolution_scale: 1.0,
            content: Default::default(),
        }
    }
}
impl PaintImage {
    pub fn BitmapPixels(&self) -> Option<&std::sync::Arc<Vec<u8>>> {
        match &self.content {
            image_resource::PaintImageContent::Bitmap(pixels) => Some(pixels),
            image_resource::PaintImageContent::Document(_) => None,
        }
    }

    pub fn IsDocumentImage(&self) -> bool {
        matches!(self.content, image_resource::PaintImageContent::Document(_))
    }
}

// cpp: layoutng/internal/layout_input.h:1066-1075
#[allow(non_snake_case)]
pub trait HyphenationProvider {
    fn LastHyphenLocation(&self, locale: &str, word: &[u16], before_index: usize) -> usize;
}

// cpp: layoutng/internal/layout_input.h:1077-1088
#[allow(non_snake_case)]
pub trait PhraseBreakProvider {
    fn NextBreakLocation(
        &self,
        locale: &str,
        text: &[u16],
        from_index: usize,
        range_end: usize,
    ) -> usize;
}

// cpp: layoutng/internal/layout_input.h:1090-1099
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextOffsetMapping {
    pub source: usize,
    pub target: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextTransformResult {
    pub text: Vec<u16>,
    pub offset_map: Vec<TextOffsetMapping>,
}

// cpp: layoutng/internal/layout_input.h:1101-1112
#[allow(non_snake_case)]
pub trait TextTransformProvider {
    fn Transform(
        &self,
        transform: TextTransform,
        locale: &str,
        text: &[u16],
        previous_character: u16,
    ) -> TextTransformResult;
}

// cpp: layoutng/internal/layout_input.h:1114-1145
#[derive(Clone)]
pub struct ConstraintSpace {
    pub available_size: Size,
    pub fragmentainer_block_size: Option<f64>,
    pub compatibility_mode: CompatibilityMode,
    pub printing: bool,
    pub vertical_scroll_enforced: bool,
    pub scrollbar_theme: Option<ScrollbarThemeMetrics>,
    pub minimum_font_physical_size: Option<f32>,
    pub device_pixel_ratio: f64,
    pub canvas_draw_element_enabled: bool,
    pub viewport: Option<ViewportGeometry>,
    pub control_theme: ControlThemeMetrics,
    pub hyphenation: Option<std::sync::Arc<dyn HyphenationProvider>>,
    pub phrase_break: Option<std::sync::Arc<dyn PhraseBreakProvider>>,
    pub text_transform: Option<std::sync::Arc<dyn TextTransformProvider>>,
    pub font_backend_factory:
        Option<std::sync::Arc<dyn font_engine::fonts::font_backend::FontBackendFactory>>,
    pub fonts: Vec<FontFace>,
    pub images: Vec<PaintImage>,
}
impl Default for ConstraintSpace {
    fn default() -> Self {
        Self {
            available_size: Size::default(),
            fragmentainer_block_size: None,
            compatibility_mode: CompatibilityMode::kStandards,
            printing: false,
            vertical_scroll_enforced: false,
            scrollbar_theme: None,
            minimum_font_physical_size: None,
            device_pixel_ratio: 1.0,
            canvas_draw_element_enabled: false,
            viewport: None,
            control_theme: ControlThemeMetrics::default(),
            hyphenation: None,
            phrase_break: None,
            text_transform: None,
            font_backend_factory: None,
            fonts: Vec::new(),
            images: Vec::new(),
        }
    }
}

#[cfg(test)]
mod shared_image_pixels_tests {
    use super::{ConstraintSpace, PaintImage};
    use std::sync::Arc;

    #[test]
    fn catalog_clones_share_admitted_pixels_and_keep_them_alive() {
        let bytes = vec![1, 2, 3, 255];
        let decoded_pointer = bytes.as_ptr();
        let image = PaintImage {
            id: 1,
            width: 1,
            height: 1,
            content: image_resource::PaintImageContent::Bitmap(bytes.into()),
            ..Default::default()
        };
        let image_pixels = image.BitmapPixels().unwrap();
        assert_eq!(
            decoded_pointer,
            image_pixels.as_ptr(),
            "admission moves the decoded allocation"
        );
        let mut original = ConstraintSpace::default();
        original.images.push(image);
        let clone = original.clone();
        let paint_catalog = original.images.clone();
        assert!(Arc::ptr_eq(
            original.images[0].BitmapPixels().unwrap(),
            clone.images[0].BitmapPixels().unwrap()
        ));
        assert!(Arc::ptr_eq(
            original.images[0].BitmapPixels().unwrap(),
            paint_catalog[0].BitmapPixels().unwrap()
        ));
        drop(original);
        drop(clone);
        assert_eq!(
            decoded_pointer,
            paint_catalog[0].BitmapPixels().unwrap().as_ptr()
        );
        assert_eq!(
            paint_catalog[0].BitmapPixels().unwrap().as_slice(),
            &[1, 2, 3, 255]
        );
    }
}

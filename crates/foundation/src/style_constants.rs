// C++: src/foundation/style_values/style/computed_style_base_constants.h
// uint8_t enum declarations only. Unsigned flag enums are in style_flags.rs;
// debug ostream definitions remain pending.
#![allow(non_camel_case_types, non_upper_case_globals)]

// C++: src/foundation/style_values/style/computed_style_base_constants.h:31-45
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EAlignmentBaseline {
    #[default]
    kBaseline,
    kMiddle,
    kAuto,
    kAlphabetic,
    kBeforeEdge,
    kAfterEdge,
    kCentral,
    kTextBeforeEdge,
    kTextAfterEdge,
    kIdeographic,
    kHanging,
    kMathematical,
}
impl EAlignmentBaseline {
    pub const kMaxEnumValue: Self = Self::kMathematical;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:47-51
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBackfaceVisibility {
    #[default]
    kHidden,
    kVisible,
}
impl EBackfaceVisibility {
    pub const kMaxEnumValue: Self = Self::kVisible;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:53-58
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBaselineSource {
    #[default]
    kAuto,
    kFirst,
    kLast,
}
impl EBaselineSource {
    pub const kMaxEnumValue: Self = Self::kLast;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:60-64
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBlockEllipsis {
    #[default]
    kEllipsis,
    kNoEllipsis,
}
impl EBlockEllipsis {
    pub const kMaxEnumValue: Self = Self::kNoEllipsis;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:66-70
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBorderCollapse {
    #[default]
    kCollapse,
    kSeparate,
}
impl EBorderCollapse {
    pub const kMaxEnumValue: Self = Self::kSeparate;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:72-84
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBorderStyle {
    #[default]
    kNone,
    kHidden,
    kInset,
    kGroove,
    kOutset,
    kRidge,
    kDotted,
    kDashed,
    kSolid,
    kDouble,
}
impl EBorderStyle {
    pub const kMaxEnumValue: Self = Self::kDouble;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:86-93
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBoxAlignment {
    #[default]
    kBaseline,
    kCenter,
    kStretch,
    kStart,
    kEnd,
}
impl EBoxAlignment {
    pub const kMaxEnumValue: Self = Self::kEnd;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:95-99
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBoxDecorationBreak {
    #[default]
    kClone,
    kSlice,
}
impl EBoxDecorationBreak {
    pub const kMaxEnumValue: Self = Self::kSlice;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:101-105
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBoxDirection {
    #[default]
    kNormal,
    kReverse,
}
impl EBoxDirection {
    pub const kMaxEnumValue: Self = Self::kReverse;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:107-111
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBoxOrient {
    #[default]
    kHorizontal,
    kVertical,
}
impl EBoxOrient {
    pub const kMaxEnumValue: Self = Self::kVertical;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:113-119
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBoxPack {
    #[default]
    kCenter,
    kJustify,
    kStart,
    kEnd,
}
impl EBoxPack {
    pub const kMaxEnumValue: Self = Self::kEnd;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:121-125
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBoxSizing {
    #[default]
    kBorderBox,
    kContentBox,
}
impl EBoxSizing {
    pub const kMaxEnumValue: Self = Self::kContentBox;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:127-139
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBreakBetween {
    #[default]
    kLeft,
    kRight,
    kAuto,
    kAvoid,
    kColumn,
    kAvoidPage,
    kPage,
    kRecto,
    kVerso,
    kAvoidColumn,
}
impl EBreakBetween {
    pub const kMaxEnumValue: Self = Self::kAvoidColumn;
}

// cpp: foundation/style_values/style/computed_style_base_constants.h:127-139
impl TryFrom<u8> for EBreakBetween {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::kLeft),
            1 => Ok(Self::kRight),
            2 => Ok(Self::kAuto),
            3 => Ok(Self::kAvoid),
            4 => Ok(Self::kColumn),
            5 => Ok(Self::kAvoidPage),
            6 => Ok(Self::kPage),
            7 => Ok(Self::kRecto),
            8 => Ok(Self::kVerso),
            9 => Ok(Self::kAvoidColumn),
            _ => Err(()),
        }
    }
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:141-147
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBreakInside {
    #[default]
    kAuto,
    kAvoid,
    kAvoidPage,
    kAvoidColumn,
}
impl EBreakInside {
    pub const kMaxEnumValue: Self = Self::kAvoidColumn;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:149-154
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EBufferedRendering {
    #[default]
    kAuto,
    kStatic,
    kDynamic,
}
impl EBufferedRendering {
    pub const kMaxEnumValue: Self = Self::kDynamic;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:156-160
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ECaptionSide {
    #[default]
    kTop,
    kBottom,
}
impl ECaptionSide {
    pub const kMaxEnumValue: Self = Self::kBottom;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:162-166
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ECaretAnimation {
    #[default]
    kAuto,
    kManual,
}
impl ECaretAnimation {
    pub const kMaxEnumValue: Self = Self::kManual;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:168-174
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ECaretShape {
    #[default]
    kBlock,
    kAuto,
    kBar,
    kUnderscore,
}
impl ECaretShape {
    pub const kMaxEnumValue: Self = Self::kUnderscore;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:176-184
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EClear {
    #[default]
    kNone,
    kLeft,
    kRight,
    kInlineStart,
    kInlineEnd,
    kBoth,
}
impl EClear {
    pub const kMaxEnumValue: Self = Self::kBoth;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:186-191
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EColorInterpolation {
    #[default]
    kAuto,
    kSRGB,
    kLinearrgb,
}
impl EColorInterpolation {
    pub const kMaxEnumValue: Self = Self::kLinearrgb;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:193-198
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EColorRendering {
    #[default]
    kAuto,
    kOptimizespeed,
    kOptimizequality,
}
impl EColorRendering {
    pub const kMaxEnumValue: Self = Self::kOptimizequality;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:200-204
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EColumnFill {
    #[default]
    kAuto,
    kBalance,
}
impl EColumnFill {
    pub const kMaxEnumValue: Self = Self::kBalance;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:206-210
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EColumnSpan {
    #[default]
    kNone,
    kAll,
}
impl EColumnSpan {
    pub const kMaxEnumValue: Self = Self::kAll;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:212-217
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EColumnWrap {
    #[default]
    kAuto,
    kNowrap,
    kWrap,
}
impl EColumnWrap {
    pub const kMaxEnumValue: Self = Self::kWrap;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:219-224
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EContentVisibility {
    #[default]
    kHidden,
    kAuto,
    kVisible,
}
impl EContentVisibility {
    pub const kMaxEnumValue: Self = Self::kVisible;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:226-231
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EContinue {
    #[default]
    kNormal,
    kCollapse,
    kWebkitLegacy,
}
impl EContinue {
    pub const kMaxEnumValue: Self = Self::kWebkitLegacy;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:233-271
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ECursor {
    #[default]
    kNone,
    kCopy,
    kAuto,
    kCrosshair,
    kDefault,
    kPointer,
    kMove,
    kVerticalText,
    kCell,
    kContextMenu,
    kAlias,
    kProgress,
    kNoDrop,
    kNotAllowed,
    kZoomIn,
    kZoomOut,
    kEResize,
    kNeResize,
    kNwResize,
    kNResize,
    kSeResize,
    kSwResize,
    kSResize,
    kWResize,
    kEwResize,
    kNsResize,
    kNeswResize,
    kNwseResize,
    kColResize,
    kRowResize,
    kText,
    kWait,
    kHelp,
    kAllScroll,
    kGrab,
    kGrabbing,
}
impl ECursor {
    pub const kMaxEnumValue: Self = Self::kGrabbing;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:273-310
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EDisplay {
    #[default]
    kInline,
    kBlock,
    kListItem,
    kInlineBlock,
    kTable,
    kInlineTable,
    kTableRowGroup,
    kTableHeaderGroup,
    kTableFooterGroup,
    kTableRow,
    kTableColumnGroup,
    kTableColumn,
    kTableCell,
    kTableCaption,
    kWebkitBox,
    kWebkitInlineBox,
    kFlex,
    kInlineFlex,
    kGrid,
    kInlineGrid,
    kContents,
    kFlowRoot,
    kNone,
    kLayoutCustom,
    kInlineLayoutCustom,
    kMath,
    kBlockMath,
    kInlineListItem,
    kFlowRootListItem,
    kInlineFlowRootListItem,
    kRuby,
    kBlockRuby,
    kRubyText,
    kGridLanes,
    kInlineGridLanes,
}
impl EDisplay {
    pub const kMaxEnumValue: Self = Self::kInlineGridLanes;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:312-326
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EDominantBaseline {
    #[default]
    kMiddle,
    kAuto,
    kAlphabetic,
    kCentral,
    kTextBeforeEdge,
    kTextAfterEdge,
    kIdeographic,
    kHanging,
    kMathematical,
    kUseScript,
    kNoChange,
    kResetSize,
}
impl EDominantBaseline {
    pub const kMaxEnumValue: Self = Self::kResetSize;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:328-333
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EDraggableRegionMode {
    #[default]
    kNone,
    kMove,
    kNoDrag,
}
impl EDraggableRegionMode {
    pub const kMaxEnumValue: Self = Self::kNoDrag;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:335-339
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EEmptyCells {
    #[default]
    kHide,
    kShow,
}
impl EEmptyCells {
    pub const kMaxEnumValue: Self = Self::kShow;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:341-345
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EFieldSizing {
    #[default]
    kFixed,
    kContent,
}
impl EFieldSizing {
    pub const kMaxEnumValue: Self = Self::kContent;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:347-353
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EFlexDirection {
    #[default]
    kRow,
    kRowReverse,
    kColumn,
    kColumnReverse,
}
impl EFlexDirection {
    pub const kMaxEnumValue: Self = Self::kColumnReverse;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:355-362
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EFloat {
    #[default]
    kNone,
    kLeft,
    kRight,
    kInlineStart,
    kInlineEnd,
}
impl EFloat {
    pub const kMaxEnumValue: Self = Self::kInlineEnd;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:364-369
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EForcedColorAdjust {
    #[default]
    kNone,
    kAuto,
    kPreserveParentColor,
}
impl EForcedColorAdjust {
    pub const kMaxEnumValue: Self = Self::kPreserveParentColor;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:371-378
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EFrameSizing {
    #[default]
    kAuto,
    kContentWidth,
    kContentHeight,
    kContentBlockSize,
    kContentInlineSize,
}
impl EFrameSizing {
    pub const kMaxEnumValue: Self = Self::kContentInlineSize;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:380-384
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EGridLanesPack {
    #[default]
    kNormal,
    kDense,
}
impl EGridLanesPack {
    pub const kMaxEnumValue: Self = Self::kDense;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:386-392
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EImageRendering {
    #[default]
    kAuto,
    kPixelated,
    kCrispEdges,
    kWebkitOptimizeContrast,
}
impl EImageRendering {
    pub const kMaxEnumValue: Self = Self::kWebkitOptimizeContrast;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:394-399
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EInlineBlockBaselineEdge {
    #[default]
    kMarginBox,
    kBorderBox,
    kContentBox,
}
impl EInlineBlockBaselineEdge {
    pub const kMaxEnumValue: Self = Self::kContentBox;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:401-406
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EInsideLink {
    #[default]
    kNotInsideLink,
    kInsideUnvisitedLink,
    kInsideVisitedLink,
}
impl EInsideLink {
    pub const kMaxEnumValue: Self = Self::kInsideVisitedLink;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:408-412
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EInteractivity {
    #[default]
    kAuto,
    kInert,
}
impl EInteractivity {
    pub const kMaxEnumValue: Self = Self::kInert;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:414-418
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EInternalOverscrollContainer {
    #[default]
    kNone,
    kAuto,
}
impl EInternalOverscrollContainer {
    pub const kMaxEnumValue: Self = Self::kAuto;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:420-424
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EInternalOverscrollPosition {
    #[default]
    kNone,
    kAuto,
}
impl EInternalOverscrollPosition {
    pub const kMaxEnumValue: Self = Self::kAuto;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:426-430
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EInternalUnbounded {
    #[default]
    kNone,
    kActive,
}
impl EInternalUnbounded {
    pub const kMaxEnumValue: Self = Self::kActive;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:432-436
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EInterpolateSize {
    #[default]
    kNumericOnly,
    kAllowKeywords,
}
impl EInterpolateSize {
    pub const kMaxEnumValue: Self = Self::kAllowKeywords;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:438-442
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EIsolation {
    #[default]
    kAuto,
    kIsolate,
}
impl EIsolation {
    pub const kMaxEnumValue: Self = Self::kIsolate;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:444-448
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EListStylePosition {
    #[default]
    kOutside,
    kInside,
}
impl EListStylePosition {
    pub const kMaxEnumValue: Self = Self::kInside;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:450-454
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EMaskType {
    #[default]
    kAlpha,
    kLuminance,
}
impl EMaskType {
    pub const kMaxEnumValue: Self = Self::kLuminance;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:456-460
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EMathShift {
    #[default]
    kNormal,
    kCompact,
}
impl EMathShift {
    pub const kMaxEnumValue: Self = Self::kCompact;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:462-466
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EMathStyle {
    #[default]
    kNormal,
    kCompact,
}
impl EMathStyle {
    pub const kMaxEnumValue: Self = Self::kCompact;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:468-472
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EMaxContentSizing {
    #[default]
    kAuto,
    kShrinkToFit,
}
impl EMaxContentSizing {
    pub const kMaxEnumValue: Self = Self::kShrinkToFit;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:474-481
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EObjectFit {
    #[default]
    kNone,
    kContain,
    kCover,
    kFill,
    kScaleDown,
}
impl EObjectFit {
    pub const kMaxEnumValue: Self = Self::kScaleDown;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:483-487
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOrder {
    #[default]
    kLogical,
    kVisual,
}
impl EOrder {
    pub const kMaxEnumValue: Self = Self::kVisual;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:489-493
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOriginTrialTestProperty {
    #[default]
    kNone,
    kNormal,
}
impl EOriginTrialTestProperty {
    pub const kMaxEnumValue: Self = Self::kNormal;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:495-503
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOverflow {
    #[default]
    kHidden,
    kAuto,
    kVisible,
    kOverlay,
    kScroll,
    kClip,
}
impl EOverflow {
    pub const kMaxEnumValue: Self = Self::kClip;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:505-510
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOverflowAnchor {
    #[default]
    kNone,
    kAuto,
    kVisible,
}
impl EOverflowAnchor {
    pub const kMaxEnumValue: Self = Self::kVisible;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:512-517
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOverflowWrap {
    #[default]
    kNormal,
    kBreakWord,
    kAnywhere,
}
impl EOverflowWrap {
    pub const kMaxEnumValue: Self = Self::kAnywhere;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:519-523
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOverlay {
    #[default]
    kNone,
    kAuto,
}
impl EOverlay {
    pub const kMaxEnumValue: Self = Self::kAuto;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:525-531
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOverscrollBehavior {
    #[default]
    kNone,
    kAuto,
    kContain,
    kChain,
}
impl EOverscrollBehavior {
    pub const kMaxEnumValue: Self = Self::kChain;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:533-539
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EOverscrollContainerType {
    #[default]
    kNone,
    kAuto,
    kOverlay,
    kPush,
}
impl EOverscrollContainerType {
    pub const kMaxEnumValue: Self = Self::kPush;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:541-546
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EPageMarginSafety {
    #[default]
    kNone,
    kClamp,
    kAdd,
}
impl EPageMarginSafety {
    pub const kMaxEnumValue: Self = Self::kAdd;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:548-561
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EPointerEvents {
    #[default]
    kNone,
    kAll,
    kAuto,
    kVisible,
    kVisiblepainted,
    kVisiblefill,
    kVisiblestroke,
    kPainted,
    kFill,
    kStroke,
    kBoundingBox,
}
impl EPointerEvents {
    pub const kMaxEnumValue: Self = Self::kBoundingBox;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:563-570
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EPosition {
    #[default]
    kAbsolute,
    kFixed,
    kRelative,
    kStatic,
    kSticky,
}
impl EPosition {
    pub const kMaxEnumValue: Self = Self::kSticky;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:572-579
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EPositionTryOrder {
    #[default]
    kNormal,
    kMostWidth,
    kMostHeight,
    kMostBlockSize,
    kMostInlineSize,
}
impl EPositionTryOrder {
    pub const kMaxEnumValue: Self = Self::kMostInlineSize;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:581-585
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EPrintColorAdjust {
    #[default]
    kEconomy,
    kExact,
}
impl EPrintColorAdjust {
    pub const kMaxEnumValue: Self = Self::kExact;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:587-596
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EReadingFlow {
    #[default]
    kNormal,
    kFlexVisual,
    kFlexFlow,
    kGridRows,
    kGridColumns,
    kGridOrder,
    kSourceOrder,
}
impl EReadingFlow {
    pub const kMaxEnumValue: Self = Self::kSourceOrder;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:598-606
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EResize {
    #[default]
    kNone,
    kInline,
    kBlock,
    kBoth,
    kHorizontal,
    kVertical,
}
impl EResize {
    pub const kMaxEnumValue: Self = Self::kVertical;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:608-614
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ERubyAlign {
    #[default]
    kCenter,
    kStart,
    kSpaceBetween,
    kSpaceAround,
}
impl ERubyAlign {
    pub const kMaxEnumValue: Self = Self::kSpaceAround;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:616-621
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ERubyOverhang {
    #[default]
    kNone,
    kAuto,
    kSpaces,
}
impl ERubyOverhang {
    pub const kMaxEnumValue: Self = Self::kSpaces;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:623-627
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ERuleOverlap {
    #[default]
    kRowOverColumn,
    kColumnOverRow,
}
impl ERuleOverlap {
    pub const kMaxEnumValue: Self = Self::kColumnOverRow;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:629-633
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EScrollAxisLock {
    #[default]
    kNone,
    kAuto,
}
impl EScrollAxisLock {
    pub const kMaxEnumValue: Self = Self::kAuto;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:635-639
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EScrollInitialTarget {
    #[default]
    kNone,
    kNearest,
}
impl EScrollInitialTarget {
    pub const kMaxEnumValue: Self = Self::kNearest;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:641-645
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EScrollSnapStop {
    #[default]
    kNormal,
    kAlways,
}
impl EScrollSnapStop {
    pub const kMaxEnumValue: Self = Self::kAlways;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:647-651
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EScrollTargetGroup {
    #[default]
    kNone,
    kAuto,
}
impl EScrollTargetGroup {
    pub const kMaxEnumValue: Self = Self::kAuto;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:653-658
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EScrollbarWidth {
    #[default]
    kNone,
    kAuto,
    kThin,
}
impl EScrollbarWidth {
    pub const kMaxEnumValue: Self = Self::kThin;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:660-666
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EShapeRendering {
    #[default]
    kAuto,
    kOptimizespeed,
    kGeometricprecision,
    kCrispedges,
}
impl EShapeRendering {
    pub const kMaxEnumValue: Self = Self::kCrispedges;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:668-676
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ESpeak {
    #[default]
    kNone,
    kNormal,
    kSpellOut,
    kDigits,
    kLiteralPunctuation,
    kNoPunctuation,
}
impl ESpeak {
    pub const kMaxEnumValue: Self = Self::kNoPunctuation;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:678-682
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETableLayout {
    #[default]
    kAuto,
    kFixed,
}
impl ETableLayout {
    pub const kMaxEnumValue: Self = Self::kFixed;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:684-696
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextAlign {
    #[default]
    kLeft,
    kRight,
    kCenter,
    kJustify,
    kMatchParent,
    kWebkitLeft,
    kWebkitRight,
    kWebkitCenter,
    kStart,
    kEnd,
}
impl ETextAlign {
    pub const kMaxEnumValue: Self = Self::kEnd;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:698-708
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextAlignLast {
    #[default]
    kLeft,
    kRight,
    kCenter,
    kJustify,
    kMatchParent,
    kAuto,
    kStart,
    kEnd,
}
impl ETextAlignLast {
    pub const kMaxEnumValue: Self = Self::kEnd;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:710-715
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextAnchor {
    #[default]
    kMiddle,
    kStart,
    kEnd,
}
impl ETextAnchor {
    pub const kMaxEnumValue: Self = Self::kEnd;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:717-721
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextAutospace {
    #[default]
    kNormal,
    kNoAutospace,
}
impl ETextAutospace {
    pub const kMaxEnumValue: Self = Self::kNoAutospace;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:723-729
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextBoxTrim {
    #[default]
    kNone,
    kTrimBoth,
    kTrimEnd,
    kTrimStart,
}
impl ETextBoxTrim {
    pub const kMaxEnumValue: Self = Self::kTrimStart;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:731-735
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextCombine {
    #[default]
    kNone,
    kAll,
}
impl ETextCombine {
    pub const kMaxEnumValue: Self = Self::kAll;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:737-742
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextDecorationSkipInk {
    #[default]
    kNone,
    kAll,
    kAuto,
}
impl ETextDecorationSkipInk {
    pub const kMaxEnumValue: Self = Self::kAuto;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:744-751
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextDecorationStyle {
    #[default]
    kDotted,
    kDashed,
    kSolid,
    kDouble,
    kWavy,
}
impl ETextDecorationStyle {
    pub const kMaxEnumValue: Self = Self::kWavy;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:753-759
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextOrientation {
    #[default]
    kMixed,
    kSideways,
    kSidewaysRight,
    kUpright,
}
impl ETextOrientation {
    pub const kMaxEnumValue: Self = Self::kUpright;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:761-767
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETextSecurity {
    #[default]
    kNone,
    kDisc,
    kCircle,
    kSquare,
}
impl ETextSecurity {
    pub const kMaxEnumValue: Self = Self::kSquare;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:812-819
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETransformBox {
    #[default]
    kBorderBox,
    kContentBox,
    kFillBox,
    kViewBox,
    kStrokeBox,
}
impl ETransformBox {
    pub const kMaxEnumValue: Self = Self::kStrokeBox;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:821-825
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ETransformStyle3D {
    #[default]
    kFlat,
    kPreserve3d,
}
impl ETransformStyle3D {
    pub const kMaxEnumValue: Self = Self::kPreserve3d;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:827-832
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EUserDrag {
    #[default]
    kNone,
    kAuto,
    kElement,
}
impl EUserDrag {
    pub const kMaxEnumValue: Self = Self::kElement;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:834-839
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EUserModify {
    #[default]
    kReadOnly,
    kReadWrite,
    kReadWritePlaintextOnly,
}
impl EUserModify {
    pub const kMaxEnumValue: Self = Self::kReadWritePlaintextOnly;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:841-848
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EUserSelect {
    #[default]
    kNone,
    kAll,
    kAuto,
    kText,
    kContain,
}
impl EUserSelect {
    pub const kMaxEnumValue: Self = Self::kContain;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:850-854
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EVectorEffect {
    #[default]
    kNone,
    kNonScalingStroke,
}
impl EVectorEffect {
    pub const kMaxEnumValue: Self = Self::kNonScalingStroke;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:856-860
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EViewTransitionScope {
    #[default]
    kNone,
    kAll,
}
impl EViewTransitionScope {
    pub const kMaxEnumValue: Self = Self::kAll;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:862-867
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EVisibility {
    #[default]
    kHidden,
    kVisible,
    kCollapse,
}
impl EVisibility {
    pub const kMaxEnumValue: Self = Self::kCollapse;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:869-876
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EWordBreak {
    #[default]
    kNormal,
    kBreakAll,
    kKeepAll,
    kAutoPhrase,
    kBreakWord,
}
impl EWordBreak {
    pub const kMaxEnumValue: Self = Self::kBreakWord;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:878-883
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Hyphens {
    #[default]
    kNone,
    kAuto,
    kManual,
}
impl Hyphens {
    pub const kMaxEnumValue: Self = Self::kManual;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:885-893
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LineBreak {
    #[default]
    kNormal,
    kAuto,
    kLoose,
    kStrict,
    kAfterWhiteSpace,
    kAnywhere,
}
impl LineBreak {
    pub const kMaxEnumValue: Self = Self::kAnywhere;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:895-899
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RubyPosition {
    #[default]
    kOver,
    kUnder,
}
impl RubyPosition {
    pub const kMaxEnumValue: Self = Self::kUnder;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:901-906
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleBreak {
    #[default]
    kNone,
    kNormal,
    kIntersection,
}
impl RuleBreak {
    pub const kMaxEnumValue: Self = Self::kIntersection;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:908-914
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleVisibilityItems {
    #[default]
    kAll,
    kNormal,
    kAround,
    kBetween,
}
impl RuleVisibilityItems {
    pub const kMaxEnumValue: Self = Self::kBetween;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:999-1003
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextEmphasisFill {
    #[default]
    kFilled,
    kOpen,
}
impl TextEmphasisFill {
    pub const kMaxEnumValue: Self = Self::kOpen;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:1005-1015
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextEmphasisMark {
    #[default]
    kNone,
    kAuto,
    kDot,
    kCircle,
    kDoubleCircle,
    kTriangle,
    kSesame,
    kCustom,
}
impl TextEmphasisMark {
    pub const kMaxEnumValue: Self = Self::kCustom;
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:1017-1021
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextWrapMode {
    #[default]
    kNowrap,
    kWrap,
}
impl TextWrapMode {
    pub const kMaxEnumValue: Self = Self::kWrap;

    pub const fn value(self) -> u8 {
        self as u8
    }

    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0 => Self::kNowrap,
            1 => Self::kWrap,
            _ => panic!("invalid TextWrapMode bit pattern"),
        }
    }
}

// C++: src/foundation/style_values/style/computed_style_base_constants.h:1023-1029
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextWrapStyle {
    #[default]
    kAuto,
    kPretty,
    kBalance,
    kStable,
}
impl TextWrapStyle {
    pub const kMaxEnumValue: Self = Self::kStable;
}

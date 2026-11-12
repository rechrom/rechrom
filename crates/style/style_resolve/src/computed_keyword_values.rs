// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Direct CSSIdentifierValue::Create(style.Getter()) source virtuals.
//! Generated from fixed Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! longhands_custom.cc and the same css_properties.json5 keyword mappings.
//! Enum matches are typed and exhaustive; no enum ordinals or CSS text are read.
use crate::production_css_value::{self as values, Value};
use foundation::{CSSPropertyID as P, CSSValueID as V};
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;
pub fn KeywordValue(property: P, style: &ComputedStyle) -> Option<Rc<Value>> {
    let id = match property {
        // longhands_custom.cc:212-218.
        P::kAlignmentBaseline => match style.AlignmentBaseline() {
            foundation::EAlignmentBaseline::kBaseline => V::kBaseline,
            foundation::EAlignmentBaseline::kMiddle => V::kMiddle,
            foundation::EAlignmentBaseline::kAuto => V::kAuto,
            foundation::EAlignmentBaseline::kAlphabetic => V::kAlphabetic,
            foundation::EAlignmentBaseline::kBeforeEdge => V::kBeforeEdge,
            foundation::EAlignmentBaseline::kAfterEdge => V::kAfterEdge,
            foundation::EAlignmentBaseline::kCentral => V::kCentral,
            foundation::EAlignmentBaseline::kTextBeforeEdge => V::kTextBeforeEdge,
            foundation::EAlignmentBaseline::kTextAfterEdge => V::kTextAfterEdge,
            foundation::EAlignmentBaseline::kIdeographic => V::kIdeographic,
            foundation::EAlignmentBaseline::kHanging => V::kHanging,
            foundation::EAlignmentBaseline::kMathematical => V::kMathematical,
        },
        // longhands_custom.cc:1234-1240.
        P::kBaselineSource => match style.BaselineSource() {
            foundation::EBaselineSource::kAuto => V::kAuto,
            foundation::EBaselineSource::kFirst => V::kFirst,
            foundation::EBaselineSource::kLast => V::kLast,
        },
        // longhands_custom.cc:1396-1402.
        P::kBorderBottomStyle => match style.BorderBottomStyle() {
            foundation::EBorderStyle::kNone => V::kNone,
            foundation::EBorderStyle::kHidden => V::kHidden,
            foundation::EBorderStyle::kInset => V::kInset,
            foundation::EBorderStyle::kGroove => V::kGroove,
            foundation::EBorderStyle::kOutset => V::kOutset,
            foundation::EBorderStyle::kRidge => V::kRidge,
            foundation::EBorderStyle::kDotted => V::kDotted,
            foundation::EBorderStyle::kDashed => V::kDashed,
            foundation::EBorderStyle::kSolid => V::kSolid,
            foundation::EBorderStyle::kDouble => V::kDouble,
        },
        // longhands_custom.cc:1630-1636.
        P::kBorderLeftStyle => match style.BorderLeftStyle() {
            foundation::EBorderStyle::kNone => V::kNone,
            foundation::EBorderStyle::kHidden => V::kHidden,
            foundation::EBorderStyle::kInset => V::kInset,
            foundation::EBorderStyle::kGroove => V::kGroove,
            foundation::EBorderStyle::kOutset => V::kOutset,
            foundation::EBorderStyle::kRidge => V::kRidge,
            foundation::EBorderStyle::kDotted => V::kDotted,
            foundation::EBorderStyle::kDashed => V::kDashed,
            foundation::EBorderStyle::kSolid => V::kSolid,
            foundation::EBorderStyle::kDouble => V::kDouble,
        },
        // longhands_custom.cc:1717-1723.
        P::kBorderRightStyle => match style.BorderRightStyle() {
            foundation::EBorderStyle::kNone => V::kNone,
            foundation::EBorderStyle::kHidden => V::kHidden,
            foundation::EBorderStyle::kInset => V::kInset,
            foundation::EBorderStyle::kGroove => V::kGroove,
            foundation::EBorderStyle::kOutset => V::kOutset,
            foundation::EBorderStyle::kRidge => V::kRidge,
            foundation::EBorderStyle::kDotted => V::kDotted,
            foundation::EBorderStyle::kDashed => V::kDashed,
            foundation::EBorderStyle::kSolid => V::kSolid,
            foundation::EBorderStyle::kDouble => V::kDouble,
        },
        // longhands_custom.cc:1822-1828.
        P::kBorderTopStyle => match style.BorderTopStyle() {
            foundation::EBorderStyle::kNone => V::kNone,
            foundation::EBorderStyle::kHidden => V::kHidden,
            foundation::EBorderStyle::kInset => V::kInset,
            foundation::EBorderStyle::kGroove => V::kGroove,
            foundation::EBorderStyle::kOutset => V::kOutset,
            foundation::EBorderStyle::kRidge => V::kRidge,
            foundation::EBorderStyle::kDotted => V::kDotted,
            foundation::EBorderStyle::kDashed => V::kDashed,
            foundation::EBorderStyle::kSolid => V::kSolid,
            foundation::EBorderStyle::kDouble => V::kDouble,
        },
        // longhands_custom.cc:2042-2048.
        P::kBreakAfter => match style.BreakAfter() {
            foundation::EBreakBetween::kLeft => V::kLeft,
            foundation::EBreakBetween::kRight => V::kRight,
            foundation::EBreakBetween::kAuto => V::kAuto,
            foundation::EBreakBetween::kAvoid => V::kAvoid,
            foundation::EBreakBetween::kColumn => V::kColumn,
            foundation::EBreakBetween::kAvoidPage => V::kAvoidPage,
            foundation::EBreakBetween::kPage => V::kPage,
            foundation::EBreakBetween::kRecto => V::kRecto,
            foundation::EBreakBetween::kVerso => V::kVerso,
            foundation::EBreakBetween::kAvoidColumn => V::kAvoidColumn,
        },
        // longhands_custom.cc:2050-2056.
        P::kBreakBefore => match style.BreakBefore() {
            foundation::EBreakBetween::kLeft => V::kLeft,
            foundation::EBreakBetween::kRight => V::kRight,
            foundation::EBreakBetween::kAuto => V::kAuto,
            foundation::EBreakBetween::kAvoid => V::kAvoid,
            foundation::EBreakBetween::kColumn => V::kColumn,
            foundation::EBreakBetween::kAvoidPage => V::kAvoidPage,
            foundation::EBreakBetween::kPage => V::kPage,
            foundation::EBreakBetween::kRecto => V::kRecto,
            foundation::EBreakBetween::kVerso => V::kVerso,
            foundation::EBreakBetween::kAvoidColumn => V::kAvoidColumn,
        },
        // longhands_custom.cc:2058-2064.
        P::kBreakInside => match style.BreakInside() {
            foundation::EBreakInside::kAuto => V::kAuto,
            foundation::EBreakInside::kAvoid => V::kAvoid,
            foundation::EBreakInside::kAvoidPage => V::kAvoidPage,
            foundation::EBreakInside::kAvoidColumn => V::kAvoidColumn,
        },
        // longhands_custom.cc:2066-2072.
        P::kBufferedRendering => match style.BufferedRendering() {
            foundation::EBufferedRendering::kAuto => V::kAuto,
            foundation::EBufferedRendering::kStatic => V::kStatic,
            foundation::EBufferedRendering::kDynamic => V::kDynamic,
        },
        // longhands_custom.cc:2074-2080.
        P::kCaptionSide => match style.CaptionSide() {
            foundation::ECaptionSide::kTop => V::kTop,
            foundation::ECaptionSide::kBottom => V::kBottom,
        },
        // longhands_custom.cc:2082-2088.
        P::kCaretAnimation => match style.CaretAnimation() {
            foundation::ECaretAnimation::kAuto => V::kAuto,
            foundation::ECaretAnimation::kManual => V::kManual,
        },
        // longhands_custom.cc:2125-2131.
        P::kCaretShape => match style.CaretShape() {
            foundation::ECaretShape::kBlock => V::kBlock,
            foundation::ECaretShape::kAuto => V::kAuto,
            foundation::ECaretShape::kBar => V::kBar,
            foundation::ECaretShape::kUnderscore => V::kUnderscore,
        },
        // longhands_custom.cc:2191-2197.
        P::kClear => match style.Clear() {
            foundation::EClear::kNone => V::kNone,
            foundation::EClear::kLeft => V::kLeft,
            foundation::EClear::kRight => V::kRight,
            foundation::EClear::kInlineStart => V::kInlineStart,
            foundation::EClear::kInlineEnd => V::kInlineEnd,
            foundation::EClear::kBoth => V::kBoth,
        },
        // longhands_custom.cc:2343-2349.
        P::kClipRule => match style.ClipRule() {
            foundation::WindRule::RULE_NONZERO => V::kNonzero,
            foundation::WindRule::RULE_EVENODD => V::kEvenodd,
        },
        // longhands_custom.cc:2432-2438.
        P::kColorInterpolation => match style.ColorInterpolation() {
            foundation::EColorInterpolation::kAuto => V::kAuto,
            foundation::EColorInterpolation::kSRGB => V::kSRGB,
            foundation::EColorInterpolation::kLinearrgb => V::kLinearrgb,
        },
        // longhands_custom.cc:2440-2446.
        P::kColorInterpolationFilters => match style.ColorInterpolationFilters() {
            foundation::EColorInterpolation::kAuto => V::kAuto,
            foundation::EColorInterpolation::kSRGB => V::kSRGB,
            foundation::EColorInterpolation::kLinearrgb => V::kLinearrgb,
        },
        // longhands_custom.cc:2448-2454.
        P::kColorRendering => match style.ColorRendering() {
            foundation::EColorRendering::kAuto => V::kAuto,
            foundation::EColorRendering::kOptimizespeed => V::kOptimizespeed,
            foundation::EColorRendering::kOptimizequality => V::kOptimizequality,
        },
        // longhands_custom.cc:2589-2595.
        P::kColumnFill => match style.GetColumnFill() {
            foundation::EColumnFill::kAuto => V::kAuto,
            foundation::EColumnFill::kBalance => V::kBalance,
        },
        // longhands_custom.cc:2597-2603.
        P::kColumnWrap => match style.ColumnWrap() {
            foundation::EColumnWrap::kAuto => V::kAuto,
            foundation::EColumnWrap::kNowrap => V::kNowrap,
            foundation::EColumnWrap::kWrap => V::kWrap,
        },
        // longhands_custom.cc:3602-3608.
        P::kDirection => match style.Direction() {
            foundation::TextDirection::kLtr => V::kLtr,
            foundation::TextDirection::kRtl => V::kRtl,
        },
        // longhands_custom.cc:3984-3990.
        P::kDominantBaseline => match style.DominantBaseline() {
            foundation::EDominantBaseline::kMiddle => V::kMiddle,
            foundation::EDominantBaseline::kAuto => V::kAuto,
            foundation::EDominantBaseline::kAlphabetic => V::kAlphabetic,
            foundation::EDominantBaseline::kCentral => V::kCentral,
            foundation::EDominantBaseline::kTextBeforeEdge => V::kTextBeforeEdge,
            foundation::EDominantBaseline::kTextAfterEdge => V::kTextAfterEdge,
            foundation::EDominantBaseline::kIdeographic => V::kIdeographic,
            foundation::EDominantBaseline::kHanging => V::kHanging,
            foundation::EDominantBaseline::kMathematical => V::kMathematical,
            foundation::EDominantBaseline::kUseScript => V::kUseScript,
            foundation::EDominantBaseline::kNoChange => V::kNoChange,
            foundation::EDominantBaseline::kResetSize => V::kResetSize,
        },
        // longhands_custom.cc:4104-4110.
        P::kEmptyCells => match style.EmptyCells() {
            foundation::EEmptyCells::kHide => V::kHide,
            foundation::EEmptyCells::kShow => V::kShow,
        },
        // longhands_custom.cc:4154-4160.
        P::kFillRule => match style.FillRule() {
            foundation::WindRule::RULE_NONZERO => V::kNonzero,
            foundation::WindRule::RULE_EVENODD => V::kEvenodd,
        },
        // longhands_custom.cc:4210-4216.
        P::kFlexDirection => match style.FlexDirection() {
            foundation::EFlexDirection::kRow => V::kRow,
            foundation::EFlexDirection::kRowReverse => V::kRowReverse,
            foundation::EFlexDirection::kColumn => V::kColumn,
            foundation::EFlexDirection::kColumnReverse => V::kColumnReverse,
        },
        // longhands_custom.cc:4830-4836.
        P::kForcedColorAdjust => match style.ForcedColorAdjust() {
            foundation::EForcedColorAdjust::kNone => V::kNone,
            foundation::EForcedColorAdjust::kAuto => V::kAuto,
            foundation::EForcedColorAdjust::kPreserveParentColor => V::kPreserveParentColor,
        },
        // longhands_custom.cc:4838-4844.
        P::kFieldSizing => match style.FieldSizing() {
            foundation::EFieldSizing::kFixed => V::kFixed,
            foundation::EFieldSizing::kContent => V::kContent,
        },
        // longhands_custom.cc:4846-4852.
        P::kFrameSizing => match style.FrameSizing() {
            foundation::EFrameSizing::kAuto => V::kAuto,
            foundation::EFrameSizing::kContentWidth => V::kContentWidth,
            foundation::EFrameSizing::kContentHeight => V::kContentHeight,
            foundation::EFrameSizing::kContentBlockSize => V::kContentBlockSize,
            foundation::EFrameSizing::kContentInlineSize => V::kContentInlineSize,
        },
        // longhands_custom.cc:5083-5089.
        P::kGridLanesPack => match style.GridLanesPack() {
            foundation::EGridLanesPack::kNormal => V::kNormal,
            foundation::EGridLanesPack::kDense => V::kDense,
        },
        // longhands_custom.cc:5322-5328.
        P::kHyphens => match style.GetHyphens() {
            foundation::Hyphens::kNone => V::kNone,
            foundation::Hyphens::kAuto => V::kAuto,
            foundation::Hyphens::kManual => V::kManual,
        },
        // longhands_custom.cc:5349-5355.
        P::kImageAnimation => match style.ImageAnimation() {
            foundation::ImageAnimationEnum::kNormal => V::kNormal,
            foundation::ImageAnimationEnum::kRunning => V::kRunning,
            foundation::ImageAnimationEnum::kPaused => V::kPaused,
            foundation::ImageAnimationEnum::kStopped => V::kStopped,
        },
        // longhands_custom.cc:5357-5363.
        P::kImageRendering => match style.ImageRendering() {
            foundation::EImageRendering::kAuto => V::kAuto,
            foundation::EImageRendering::kPixelated => V::kPixelated,
            foundation::EImageRendering::kCrispEdges => V::kCrispEdges,
            foundation::EImageRendering::kWebkitOptimizeContrast => V::kWebkitOptimizeContrast,
        },
        // longhands_custom.cc:5624-5630.
        P::kColumnRuleBreak => match style.ColumnRuleBreak() {
            foundation::RuleBreak::kNone => V::kNone,
            foundation::RuleBreak::kNormal => V::kNormal,
            foundation::RuleBreak::kIntersection => V::kIntersection,
        },
        // longhands_custom.cc:5632-5638.
        P::kRowRuleBreak => match style.RowRuleBreak() {
            foundation::RuleBreak::kNone => V::kNone,
            foundation::RuleBreak::kNormal => V::kNormal,
            foundation::RuleBreak::kIntersection => V::kIntersection,
        },
        // longhands_custom.cc:5824-5830.
        P::kColumnRuleVisibilityItems => match style.ColumnRuleVisibilityItems() {
            foundation::RuleVisibilityItems::kAll => V::kAll,
            foundation::RuleVisibilityItems::kNormal => V::kNormal,
            foundation::RuleVisibilityItems::kAround => V::kAround,
            foundation::RuleVisibilityItems::kBetween => V::kBetween,
        },
        // longhands_custom.cc:5832-5838.
        P::kRowRuleVisibilityItems => match style.RowRuleVisibilityItems() {
            foundation::RuleVisibilityItems::kAll => V::kAll,
            foundation::RuleVisibilityItems::kNormal => V::kNormal,
            foundation::RuleVisibilityItems::kAround => V::kAround,
            foundation::RuleVisibilityItems::kBetween => V::kBetween,
        },
        // longhands_custom.cc:6140-6146.
        P::kInteractivity => match style.Interactivity() {
            foundation::EInteractivity::kAuto => V::kAuto,
            foundation::EInteractivity::kInert => V::kInert,
        },
        // longhands_custom.cc:6148-6154.
        P::kInterpolateSize => match style.InterpolateSize() {
            foundation::EInterpolateSize::kNumericOnly => V::kNumericOnly,
            foundation::EInterpolateSize::kAllowKeywords => V::kAllowKeywords,
        },
        // longhands_custom.cc:6156-6162.
        P::kIsolation => match style.Isolation() {
            foundation::EIsolation::kAuto => V::kAuto,
            foundation::EIsolation::kIsolate => V::kIsolate,
        },
        // longhands_custom.cc:6325-6331.
        P::kLineBreak => match style.GetLineBreak() {
            foundation::LineBreak::kNormal => V::kNormal,
            foundation::LineBreak::kAuto => V::kAuto,
            foundation::LineBreak::kLoose => V::kLoose,
            foundation::LineBreak::kStrict => V::kStrict,
            foundation::LineBreak::kAfterWhiteSpace => V::kAfterWhiteSpace,
            foundation::LineBreak::kAnywhere => V::kAnywhere,
        },
        // longhands_custom.cc:6333-6339.
        P::kContinue => match style.Continue() {
            foundation::EContinue::kNormal => V::kNormal,
            foundation::EContinue::kCollapse => V::kCollapse,
            foundation::EContinue::kWebkitLegacy => V::kWebkitLegacy,
        },
        // longhands_custom.cc:6396-6402.
        P::kBlockEllipsis => match style.BlockEllipsis() {
            foundation::EBlockEllipsis::kEllipsis => V::kEllipsis,
            foundation::EBlockEllipsis::kNoEllipsis => V::kNoEllipsis,
        },
        // longhands_custom.cc:6602-6608.
        P::kListStylePosition => match style.ListStylePosition() {
            foundation::EListStylePosition::kOutside => V::kOutside,
            foundation::EListStylePosition::kInside => V::kInside,
        },
        // longhands_custom.cc:7028-7034.
        P::kMaskType => match style.MaskType() {
            foundation::EMaskType::kAlpha => V::kAlpha,
            foundation::EMaskType::kLuminance => V::kLuminance,
        },
        // longhands_custom.cc:7045-7051.
        P::kMathShift => match style.MathShift() {
            foundation::EMathShift::kNormal => V::kNormal,
            foundation::EMathShift::kCompact => V::kCompact,
        },
        // longhands_custom.cc:7053-7059.
        P::kMathStyle => match style.MathStyle() {
            foundation::EMathStyle::kNormal => V::kNormal,
            foundation::EMathStyle::kCompact => V::kCompact,
        },
        // longhands_custom.cc:7095-7101.
        P::kMaxContentSizing => match style.MaxContentSizing() {
            foundation::EMaxContentSizing::kAuto => V::kAuto,
            foundation::EMaxContentSizing::kShrinkToFit => V::kShrinkToFit,
        },
        // longhands_custom.cc:7236-7242.
        P::kMixBlendMode => match style.GetBlendMode() {
            foundation::BlendMode::kNormal => V::kNormal,
            foundation::BlendMode::kMultiply => V::kMultiply,
            foundation::BlendMode::kScreen => V::kScreen,
            foundation::BlendMode::kOverlay => V::kOverlay,
            foundation::BlendMode::kDarken => V::kDarken,
            foundation::BlendMode::kLighten => V::kLighten,
            foundation::BlendMode::kColorDodge => V::kColorDodge,
            foundation::BlendMode::kColorBurn => V::kColorBurn,
            foundation::BlendMode::kHardLight => V::kHardLight,
            foundation::BlendMode::kSoftLight => V::kSoftLight,
            foundation::BlendMode::kDifference => V::kDifference,
            foundation::BlendMode::kExclusion => V::kExclusion,
            foundation::BlendMode::kHue => V::kHue,
            foundation::BlendMode::kSaturation => V::kSaturation,
            foundation::BlendMode::kColor => V::kColor,
            foundation::BlendMode::kLuminosity => V::kLuminosity,
            foundation::BlendMode::kPlusLighter => V::kPlusLighter,
        },
        // longhands_custom.cc:7244-7250.
        P::kObjectFit => match style.GetObjectFit() {
            foundation::EObjectFit::kNone => V::kNone,
            foundation::EObjectFit::kContain => V::kContain,
            foundation::EObjectFit::kCover => V::kCover,
            foundation::EObjectFit::kFill => V::kFill,
            foundation::EObjectFit::kScaleDown => V::kScaleDown,
        },
        // longhands_custom.cc:7447-7453.
        P::kOriginTrialTestProperty => match style.OriginTrialTestProperty() {
            foundation::EOriginTrialTestProperty::kNone => V::kNone,
            foundation::EOriginTrialTestProperty::kNormal => V::kNormal,
        },
        // longhands_custom.cc:7622-7628.
        P::kOverflowAnchor => match style.OverflowAnchor() {
            foundation::EOverflowAnchor::kNone => V::kNone,
            foundation::EOverflowAnchor::kAuto => V::kAuto,
            foundation::EOverflowAnchor::kVisible => V::kVisible,
        },
        // longhands_custom.cc:7722-7728.
        P::kOverflowWrap => match style.OverflowWrap() {
            foundation::EOverflowWrap::kNormal => V::kNormal,
            foundation::EOverflowWrap::kBreakWord => V::kBreakWord,
            foundation::EOverflowWrap::kAnywhere => V::kAnywhere,
        },
        // longhands_custom.cc:7730-7736.
        P::kOverflowX => match style.OverflowX() {
            foundation::EOverflow::kHidden => V::kHidden,
            foundation::EOverflow::kAuto => V::kAuto,
            foundation::EOverflow::kVisible => V::kVisible,
            foundation::EOverflow::kOverlay => V::kOverlay,
            foundation::EOverflow::kScroll => V::kScroll,
            foundation::EOverflow::kClip => V::kClip,
        },
        // longhands_custom.cc:7769-7775.
        P::kOverflowY => match style.OverflowY() {
            foundation::EOverflow::kHidden => V::kHidden,
            foundation::EOverflow::kAuto => V::kAuto,
            foundation::EOverflow::kVisible => V::kVisible,
            foundation::EOverflow::kOverlay => V::kOverlay,
            foundation::EOverflow::kScroll => V::kScroll,
            foundation::EOverflow::kClip => V::kClip,
        },
        // longhands_custom.cc:7809-7815.
        P::kOverscrollContainerType => match style.OverscrollContainerType() {
            foundation::EOverscrollContainerType::kNone => V::kNone,
            foundation::EOverscrollContainerType::kAuto => V::kAuto,
            foundation::EOverscrollContainerType::kOverlay => V::kOverlay,
            foundation::EOverscrollContainerType::kPush => V::kPush,
        },
        // longhands_custom.cc:7818-7824.
        P::kInternalOverscrollContainer => match style.InternalOverscrollContainer() {
            foundation::EInternalOverscrollContainer::kNone => V::kNone,
            foundation::EInternalOverscrollContainer::kAuto => V::kAuto,
        },
        // longhands_custom.cc:7827-7833.
        P::kInternalOverscrollPosition => match style.InternalOverscrollPosition() {
            foundation::EInternalOverscrollPosition::kNone => V::kNone,
            foundation::EInternalOverscrollPosition::kAuto => V::kAuto,
        },
        // longhands_custom.cc:7836-7842.
        P::kInternalUnbounded => match style.InternalUnbounded() {
            foundation::EInternalUnbounded::kNone => V::kNone,
            foundation::EInternalUnbounded::kActive => V::kActive,
        },
        // longhands_custom.cc:7844-7850.
        P::kOverscrollBehaviorX => match style.OverscrollBehaviorX() {
            foundation::EOverscrollBehavior::kNone => V::kNone,
            foundation::EOverscrollBehavior::kAuto => V::kAuto,
            foundation::EOverscrollBehavior::kContain => V::kContain,
            foundation::EOverscrollBehavior::kChain => V::kChain,
        },
        // longhands_custom.cc:7852-7858.
        P::kOverscrollBehaviorY => match style.OverscrollBehaviorY() {
            foundation::EOverscrollBehavior::kNone => V::kNone,
            foundation::EOverscrollBehavior::kAuto => V::kAuto,
            foundation::EOverscrollBehavior::kContain => V::kContain,
            foundation::EOverscrollBehavior::kChain => V::kChain,
        },
        // longhands_custom.cc:8159-8165.
        P::kViewTransitionScope => match style.ViewTransitionScope() {
            foundation::EViewTransitionScope::kNone => V::kNone,
            foundation::EViewTransitionScope::kAll => V::kAll,
        },
        // longhands_custom.cc:8329-8335.
        P::kPointerEvents => match style.PointerEvents() {
            foundation::EPointerEvents::kNone => V::kNone,
            foundation::EPointerEvents::kAll => V::kAll,
            foundation::EPointerEvents::kAuto => V::kAuto,
            foundation::EPointerEvents::kVisible => V::kVisible,
            foundation::EPointerEvents::kVisiblepainted => V::kVisiblepainted,
            foundation::EPointerEvents::kVisiblefill => V::kVisiblefill,
            foundation::EPointerEvents::kVisiblestroke => V::kVisiblestroke,
            foundation::EPointerEvents::kPainted => V::kPainted,
            foundation::EPointerEvents::kFill => V::kFill,
            foundation::EPointerEvents::kStroke => V::kStroke,
            foundation::EPointerEvents::kBoundingBox => V::kBoundingBox,
        },
        // longhands_custom.cc:8380-8386.
        P::kPositionTryOrder => match style.PositionTryOrder() {
            foundation::EPositionTryOrder::kNormal => V::kNormal,
            foundation::EPositionTryOrder::kMostWidth => V::kMostWidth,
            foundation::EPositionTryOrder::kMostHeight => V::kMostHeight,
            foundation::EPositionTryOrder::kMostBlockSize => V::kMostBlockSize,
            foundation::EPositionTryOrder::kMostInlineSize => V::kMostInlineSize,
        },
        // longhands_custom.cc:8455-8461.
        P::kReadingFlow => match style.ReadingFlow() {
            foundation::EReadingFlow::kNormal => V::kNormal,
            foundation::EReadingFlow::kFlexVisual => V::kFlexVisual,
            foundation::EReadingFlow::kFlexFlow => V::kFlexFlow,
            foundation::EReadingFlow::kGridRows => V::kGridRows,
            foundation::EReadingFlow::kGridColumns => V::kGridColumns,
            foundation::EReadingFlow::kGridOrder => V::kGridOrder,
            foundation::EReadingFlow::kSourceOrder => V::kSourceOrder,
        },
        // longhands_custom.cc:8472-8478.
        P::kResize => match style.Resize() {
            foundation::EResize::kNone => V::kNone,
            foundation::EResize::kInline => V::kInline,
            foundation::EResize::kBlock => V::kBlock,
            foundation::EResize::kBoth => V::kBoth,
            foundation::EResize::kHorizontal => V::kHorizontal,
            foundation::EResize::kVertical => V::kVertical,
        },
        // longhands_custom.cc:8634-8640.
        P::kRuleOverlap => match style.RuleOverlap() {
            foundation::ERuleOverlap::kRowOverColumn => V::kRowOverColumn,
            foundation::ERuleOverlap::kColumnOverRow => V::kColumnOverRow,
        },
        // longhands_custom.cc:8749-8755.
        P::kScrollTargetGroup => match style.ScrollTargetGroup() {
            foundation::EScrollTargetGroup::kNone => V::kNone,
            foundation::EScrollTargetGroup::kAuto => V::kAuto,
        },
        // longhands_custom.cc:8930-8936.
        P::kScrollAxisLock => match style.ScrollAxisLock() {
            foundation::EScrollAxisLock::kNone => V::kNone,
            foundation::EScrollAxisLock::kAuto => V::kAuto,
        },
        // longhands_custom.cc:9119-9125.
        P::kScrollSnapStop => match style.ScrollSnapStop() {
            foundation::EScrollSnapStop::kNormal => V::kNormal,
            foundation::EScrollSnapStop::kAlways => V::kAlways,
        },
        // longhands_custom.cc:9170-9176.
        P::kScrollInitialTarget => match style.ScrollInitialTarget() {
            foundation::EScrollInitialTarget::kNone => V::kNone,
            foundation::EScrollInitialTarget::kNearest => V::kNearest,
        },
        // longhands_custom.cc:9319-9325.
        P::kShapeRendering => match style.ShapeRendering() {
            foundation::EShapeRendering::kAuto => V::kAuto,
            foundation::EShapeRendering::kOptimizespeed => V::kOptimizespeed,
            foundation::EShapeRendering::kGeometricprecision => V::kGeometricprecision,
            foundation::EShapeRendering::kCrispedges => V::kCrispEdges,
        },
        // longhands_custom.cc:9486-9492.
        P::kSpeak => match style.Speak() {
            foundation::ESpeak::kNone => V::kNone,
            foundation::ESpeak::kNormal => V::kNormal,
            foundation::ESpeak::kSpellOut => V::kSpellOut,
            foundation::ESpeak::kDigits => V::kDigits,
            foundation::ESpeak::kLiteralPunctuation => V::kLiteralPunctuation,
            foundation::ESpeak::kNoPunctuation => V::kNoPunctuation,
        },
        // longhands_custom.cc:9624-9630.
        P::kStrokeLinecap => match style.CapStyle() {
            foundation::LineCap::kButtCap => V::kButt,
            foundation::LineCap::kRoundCap => V::kRound,
            foundation::LineCap::kSquareCap => V::kSquare,
        },
        // longhands_custom.cc:9632-9638.
        P::kStrokeLinejoin => match style.JoinStyle() {
            foundation::LineJoin::kMiterJoin => V::kMiter,
            foundation::LineJoin::kRoundJoin => V::kRound,
            foundation::LineJoin::kBevelJoin => V::kBevel,
        },
        // longhands_custom.cc:9688-9694.
        P::kContentVisibility => match style.ContentVisibility() {
            foundation::EContentVisibility::kHidden => V::kHidden,
            foundation::EContentVisibility::kAuto => V::kAuto,
            foundation::EContentVisibility::kVisible => V::kVisible,
        },
        // longhands_custom.cc:9725-9731.
        P::kTableLayout => match style.TableLayout() {
            foundation::ETableLayout::kAuto => V::kAuto,
            foundation::ETableLayout::kFixed => V::kFixed,
        },
        // longhands_custom.cc:9733-9739.
        P::kTextAlign => match style.GetTextAlign() {
            foundation::ETextAlign::kLeft => V::kLeft,
            foundation::ETextAlign::kRight => V::kRight,
            foundation::ETextAlign::kCenter => V::kCenter,
            foundation::ETextAlign::kJustify => V::kJustify,
            foundation::ETextAlign::kMatchParent => V::kMatchParent,
            foundation::ETextAlign::kWebkitLeft => V::kWebkitLeft,
            foundation::ETextAlign::kWebkitRight => V::kWebkitRight,
            foundation::ETextAlign::kWebkitCenter => V::kWebkitCenter,
            foundation::ETextAlign::kStart => V::kStart,
            foundation::ETextAlign::kEnd => V::kEnd,
        },
        // longhands_custom.cc:9777-9783.
        P::kTextAlignLast => match style.TextAlignLast() {
            foundation::ETextAlignLast::kLeft => V::kLeft,
            foundation::ETextAlignLast::kRight => V::kRight,
            foundation::ETextAlignLast::kCenter => V::kCenter,
            foundation::ETextAlignLast::kJustify => V::kJustify,
            foundation::ETextAlignLast::kMatchParent => V::kMatchParent,
            foundation::ETextAlignLast::kAuto => V::kAuto,
            foundation::ETextAlignLast::kStart => V::kStart,
            foundation::ETextAlignLast::kEnd => V::kEnd,
        },
        // longhands_custom.cc:9785-9791.
        P::kTextAnchor => match style.TextAnchor() {
            foundation::ETextAnchor::kMiddle => V::kMiddle,
            foundation::ETextAnchor::kStart => V::kStart,
            foundation::ETextAnchor::kEnd => V::kEnd,
        },
        // longhands_custom.cc:9793-9799.
        P::kTextAutospace => match style.TextAutospace() {
            foundation::ETextAutospace::kNormal => V::kNormal,
            foundation::ETextAutospace::kNoAutospace => V::kNoAutospace,
        },
        // longhands_custom.cc:9824-9830.
        P::kTextBoxTrim => match style.TextBoxTrim() {
            foundation::ETextBoxTrim::kNone => V::kNone,
            foundation::ETextBoxTrim::kTrimBoth => V::kTrimBoth,
            foundation::ETextBoxTrim::kTrimEnd => V::kTrimEnd,
            foundation::ETextBoxTrim::kTrimStart => V::kTrimStart,
        },
        // longhands_custom.cc:9832-9838.
        P::kTextCombineUpright => match style.TextCombine() {
            foundation::ETextCombine::kNone => V::kNone,
            foundation::ETextCombine::kAll => V::kAll,
        },
        // longhands_custom.cc:10100-10106.
        P::kTextJustify => match style.GetTextJustify() {
            foundation::TextJustify::kAuto => V::kAuto,
            foundation::TextJustify::kNone => V::kNone,
            foundation::TextJustify::kInterCharacter => V::kInterCharacter,
            foundation::TextJustify::kInterWord => V::kInterWord,
        },
        // longhands_custom.cc:10124-10130.
        P::kTextOrientation => match style.GetTextOrientation() {
            foundation::ETextOrientation::kMixed => V::kMixed,
            foundation::ETextOrientation::kSideways => V::kSideways,
            foundation::ETextOrientation::kSidewaysRight => V::kSidewaysRight,
            foundation::ETextOrientation::kUpright => V::kUpright,
        },
        // longhands_custom.cc:10462-10468.
        P::kTransformBox => match style.TransformBox() {
            foundation::ETransformBox::kBorderBox => V::kBorderBox,
            foundation::ETransformBox::kContentBox => V::kContentBox,
            foundation::ETransformBox::kFillBox => V::kFillBox,
            foundation::ETransformBox::kViewBox => V::kViewBox,
            foundation::ETransformBox::kStrokeBox => V::kStrokeBox,
        },
        // longhands_custom.cc:10752-10758.
        P::kUnicodeBidi => match style.GetUnicodeBidi() {
            foundation::UnicodeBidi::kNormal => V::kNormal,
            foundation::UnicodeBidi::kEmbed => V::kEmbed,
            foundation::UnicodeBidi::kBidiOverride => V::kBidiOverride,
            foundation::UnicodeBidi::kIsolate => V::kIsolate,
            foundation::UnicodeBidi::kPlaintext => V::kPlaintext,
            foundation::UnicodeBidi::kIsolateOverride => V::kIsolateOverride,
        },
        // longhands_custom.cc:10760-10766.
        P::kUserSelect => match style.UserSelect() {
            foundation::EUserSelect::kNone => V::kNone,
            foundation::EUserSelect::kAll => V::kAll,
            foundation::EUserSelect::kAuto => V::kAuto,
            foundation::EUserSelect::kText => V::kText,
            foundation::EUserSelect::kContain => V::kContain,
        },
        // longhands_custom.cc:10768-10774.
        P::kVectorEffect => match style.VectorEffect() {
            foundation::EVectorEffect::kNone => V::kNone,
            foundation::EVectorEffect::kNonScalingStroke => V::kNonScalingStroke,
        },
        // longhands_custom.cc:10943-10949.
        P::kVisibility => match style.Visibility() {
            foundation::EVisibility::kHidden => V::kHidden,
            foundation::EVisibility::kVisible => V::kVisible,
            foundation::EVisibility::kCollapse => V::kCollapse,
        },
        // longhands_custom.cc:11086-11092.
        P::kWebkitBoxAlign => match style.BoxAlign() {
            foundation::EBoxAlignment::kBaseline => V::kBaseline,
            foundation::EBoxAlignment::kCenter => V::kCenter,
            foundation::EBoxAlignment::kStretch => V::kStretch,
            foundation::EBoxAlignment::kStart => V::kStart,
            foundation::EBoxAlignment::kEnd => V::kEnd,
        },
        // longhands_custom.cc:11103-11109.
        P::kWebkitBoxDirection => match style.BoxDirection() {
            foundation::EBoxDirection::kNormal => V::kNormal,
            foundation::EBoxDirection::kReverse => V::kReverse,
        },
        // longhands_custom.cc:11137-11143.
        P::kWebkitBoxOrient => match style.BoxOrient() {
            foundation::EBoxOrient::kHorizontal => V::kHorizontal,
            foundation::EBoxOrient::kVertical => V::kVertical,
        },
        // longhands_custom.cc:11145-11151.
        P::kWebkitBoxPack => match style.BoxPack() {
            foundation::EBoxPack::kCenter => V::kCenter,
            foundation::EBoxPack::kJustify => V::kJustify,
            foundation::EBoxPack::kStart => V::kStart,
            foundation::EBoxPack::kEnd => V::kEnd,
        },
        // longhands_custom.cc:11245-11251.
        P::kWebkitLineBreak => match style.GetLineBreak() {
            foundation::LineBreak::kNormal => V::kNormal,
            foundation::LineBreak::kAuto => V::kAuto,
            foundation::LineBreak::kLoose => V::kLoose,
            foundation::LineBreak::kStrict => V::kStrict,
            foundation::LineBreak::kAfterWhiteSpace => V::kAfterWhiteSpace,
            foundation::LineBreak::kAnywhere => V::kAnywhere,
        },
        // longhands_custom.cc:11720-11726.
        P::kPrintColorAdjust => match style.PrintColorAdjust() {
            foundation::EPrintColorAdjust::kEconomy => V::kEconomy,
            foundation::EPrintColorAdjust::kExact => V::kExact,
        },
        // longhands_custom.cc:11738-11744.
        P::kRubyAlign => match style.RubyAlign() {
            foundation::ERubyAlign::kCenter => V::kCenter,
            foundation::ERubyAlign::kStart => V::kStart,
            foundation::ERubyAlign::kSpaceBetween => V::kSpaceBetween,
            foundation::ERubyAlign::kSpaceAround => V::kSpaceAround,
        },
        // longhands_custom.cc:11765-11771.
        P::kRubyOverhang => match style.RubyOverhang() {
            foundation::ERubyOverhang::kNone => V::kNone,
            foundation::ERubyOverhang::kAuto => V::kAuto,
            foundation::ERubyOverhang::kSpaces => V::kSpaces,
        },
        // longhands_custom.cc:11804-11810.
        P::kRubyPosition => match style.GetRubyPosition() {
            foundation::RubyPosition::kOver => V::kOver,
            foundation::RubyPosition::kUnder => V::kUnder,
        },
        // longhands_custom.cc:12158-12164.
        P::kWebkitTextSecurity => match style.TextSecurity() {
            foundation::ETextSecurity::kNone => V::kNone,
            foundation::ETextSecurity::kDisc => V::kDisc,
            foundation::ETextSecurity::kCircle => V::kCircle,
            foundation::ETextSecurity::kSquare => V::kSquare,
        },
        // longhands_custom.cc:12346-12352.
        P::kWebkitUserDrag => match style.UserDrag() {
            foundation::EUserDrag::kNone => V::kNone,
            foundation::EUserDrag::kAuto => V::kAuto,
            foundation::EUserDrag::kElement => V::kElement,
        },
        // longhands_custom.cc:12354-12360.
        P::kWebkitUserModify => match style.UserModify() {
            foundation::EUserModify::kReadOnly => V::kReadOnly,
            foundation::EUserModify::kReadWrite => V::kReadWrite,
            foundation::EUserModify::kReadWritePlaintextOnly => V::kReadWritePlaintextOnly,
        },
        // longhands_custom.cc:12362-12368.
        P::kWebkitWritingMode => match style.GetWritingMode() {
            foundation::WritingMode::kHorizontalTb => V::kHorizontalTb,
            foundation::WritingMode::kVerticalRl => V::kVerticalRl,
            foundation::WritingMode::kVerticalLr => V::kVerticalLr,
            foundation::WritingMode::kSidewaysRl => V::kSidewaysRl,
            foundation::WritingMode::kSidewaysLr => V::kSidewaysLr,
        },
        // longhands_custom.cc:12379-12385.
        P::kTextWrapMode => match style.GetTextWrapMode() {
            foundation::TextWrapMode::kNowrap => V::kNowrap,
            foundation::TextWrapMode::kWrap => V::kWrap,
        },
        // longhands_custom.cc:12387-12393.
        P::kTextWrapStyle => match style.GetTextWrapStyle() {
            foundation::TextWrapStyle::kAuto => V::kAuto,
            foundation::TextWrapStyle::kPretty => V::kPretty,
            foundation::TextWrapStyle::kBalance => V::kBalance,
            foundation::TextWrapStyle::kStable => V::kStable,
        },
        // longhands_custom.cc:12593-12599.
        P::kWordBreak => match style.WordBreak() {
            foundation::EWordBreak::kNormal => V::kNormal,
            foundation::EWordBreak::kBreakAll => V::kBreakAll,
            foundation::EWordBreak::kKeepAll => V::kKeepAll,
            foundation::EWordBreak::kAutoPhrase => V::kAutoPhrase,
            foundation::EWordBreak::kBreakWord => V::kBreakWord,
        },
        // longhands_custom.cc:12627-12633.
        P::kWritingMode => match style.GetWritingMode() {
            foundation::WritingMode::kHorizontalTb => V::kHorizontalTb,
            foundation::WritingMode::kVerticalRl => V::kVerticalRl,
            foundation::WritingMode::kVerticalLr => V::kVerticalLr,
            foundation::WritingMode::kSidewaysRl => V::kSidewaysRl,
            foundation::WritingMode::kSidewaysLr => V::kSidewaysLr,
        },
        _ => return None,
    };
    Some(values::identifier(id))
}

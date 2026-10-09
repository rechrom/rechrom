// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:22-1581,1586-1608
// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.cc:59-3414,3423-5210
// CssPropertyID's ExecutionContext-sensitive lookup (.cc:3416-3421) remains
// untranslated until the real parser/runtime exposure context exists.
// Name/JS-name/AtomicString data below comes from the actual simple getter
// bodies in generated properties/longhands.cc and properties/shorthands.cc.

#![allow(non_upper_case_globals)]

pub use foundation::css_property_id::*;
use foundation::AtomicString;
use std::sync::OnceLock;

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:851-971
pub const kCSSPropertyAliasList: [CSSPropertyID; 119] = [
    CSSPropertyID::kAliasWebkitAppearance,
    CSSPropertyID::kAliasWebkitAppRegion,
    CSSPropertyID::kAliasWebkitMaskClip,
    CSSPropertyID::kAliasWebkitMaskComposite,
    CSSPropertyID::kAliasWebkitMaskImage,
    CSSPropertyID::kAliasWebkitMaskOrigin,
    CSSPropertyID::kAliasWebkitMaskRepeat,
    CSSPropertyID::kAliasWebkitMaskSize,
    CSSPropertyID::kAliasWebkitBorderEndColor,
    CSSPropertyID::kAliasWebkitBorderEndStyle,
    CSSPropertyID::kAliasWebkitBorderEndWidth,
    CSSPropertyID::kAliasWebkitBorderStartColor,
    CSSPropertyID::kAliasWebkitBorderStartStyle,
    CSSPropertyID::kAliasWebkitBorderStartWidth,
    CSSPropertyID::kAliasWebkitBorderBeforeColor,
    CSSPropertyID::kAliasWebkitBorderBeforeStyle,
    CSSPropertyID::kAliasWebkitBorderBeforeWidth,
    CSSPropertyID::kAliasWebkitBorderAfterColor,
    CSSPropertyID::kAliasWebkitBorderAfterStyle,
    CSSPropertyID::kAliasWebkitBorderAfterWidth,
    CSSPropertyID::kAliasWebkitMarginEnd,
    CSSPropertyID::kAliasWebkitMarginStart,
    CSSPropertyID::kAliasWebkitMarginBefore,
    CSSPropertyID::kAliasWebkitMarginAfter,
    CSSPropertyID::kAliasWebkitPaddingEnd,
    CSSPropertyID::kAliasWebkitPaddingStart,
    CSSPropertyID::kAliasWebkitPaddingBefore,
    CSSPropertyID::kAliasWebkitPaddingAfter,
    CSSPropertyID::kAliasWebkitLogicalWidth,
    CSSPropertyID::kAliasWebkitLogicalHeight,
    CSSPropertyID::kAliasWebkitMinLogicalWidth,
    CSSPropertyID::kAliasWebkitMinLogicalHeight,
    CSSPropertyID::kAliasWebkitMaxLogicalWidth,
    CSSPropertyID::kAliasWebkitMaxLogicalHeight,
    CSSPropertyID::kAliasWebkitPrintColorAdjust,
    CSSPropertyID::kAliasWebkitBorderAfter,
    CSSPropertyID::kAliasWebkitBorderBefore,
    CSSPropertyID::kAliasWebkitBorderEnd,
    CSSPropertyID::kAliasWebkitBorderStart,
    CSSPropertyID::kAliasWebkitMask,
    CSSPropertyID::kAliasWebkitMaskPosition,
    CSSPropertyID::kAliasEpubCaptionSide,
    CSSPropertyID::kAliasEpubTextCombine,
    CSSPropertyID::kAliasEpubTextEmphasis,
    CSSPropertyID::kAliasEpubTextEmphasisColor,
    CSSPropertyID::kAliasEpubTextEmphasisStyle,
    CSSPropertyID::kAliasEpubTextOrientation,
    CSSPropertyID::kAliasEpubTextTransform,
    CSSPropertyID::kAliasEpubWordBreak,
    CSSPropertyID::kAliasEpubWritingMode,
    CSSPropertyID::kAliasWebkitAlignContent,
    CSSPropertyID::kAliasWebkitAlignItems,
    CSSPropertyID::kAliasWebkitAlignSelf,
    CSSPropertyID::kAliasWebkitAnimation,
    CSSPropertyID::kAliasWebkitAnimationDelay,
    CSSPropertyID::kAliasWebkitAnimationDirection,
    CSSPropertyID::kAliasWebkitAnimationDuration,
    CSSPropertyID::kAliasWebkitAnimationFillMode,
    CSSPropertyID::kAliasWebkitAnimationIterationCount,
    CSSPropertyID::kAliasWebkitAnimationName,
    CSSPropertyID::kAliasWebkitAnimationPlayState,
    CSSPropertyID::kAliasWebkitAnimationTimingFunction,
    CSSPropertyID::kAliasWebkitBackfaceVisibility,
    CSSPropertyID::kAliasWebkitBackgroundClip,
    CSSPropertyID::kAliasWebkitBackgroundOrigin,
    CSSPropertyID::kAliasWebkitBackgroundSize,
    CSSPropertyID::kAliasWebkitBorderBottomLeftRadius,
    CSSPropertyID::kAliasWebkitBorderBottomRightRadius,
    CSSPropertyID::kAliasWebkitBorderRadius,
    CSSPropertyID::kAliasWebkitBorderTopLeftRadius,
    CSSPropertyID::kAliasWebkitBorderTopRightRadius,
    CSSPropertyID::kAliasWebkitBoxShadow,
    CSSPropertyID::kAliasWebkitBoxSizing,
    CSSPropertyID::kAliasWebkitClipPath,
    CSSPropertyID::kAliasWebkitColumnCount,
    CSSPropertyID::kAliasWebkitColumnGap,
    CSSPropertyID::kAliasWebkitColumnRule,
    CSSPropertyID::kAliasWebkitColumnRuleColor,
    CSSPropertyID::kAliasWebkitColumnRuleStyle,
    CSSPropertyID::kAliasWebkitColumnRuleWidth,
    CSSPropertyID::kAliasWebkitColumnSpan,
    CSSPropertyID::kAliasWebkitColumnWidth,
    CSSPropertyID::kAliasWebkitColumns,
    CSSPropertyID::kAliasWebkitFilter,
    CSSPropertyID::kAliasWebkitFlex,
    CSSPropertyID::kAliasWebkitFlexBasis,
    CSSPropertyID::kAliasWebkitFlexDirection,
    CSSPropertyID::kAliasWebkitFlexFlow,
    CSSPropertyID::kAliasWebkitFlexGrow,
    CSSPropertyID::kAliasWebkitFlexShrink,
    CSSPropertyID::kAliasWebkitFlexWrap,
    CSSPropertyID::kAliasWebkitFontFeatureSettings,
    CSSPropertyID::kAliasWebkitHyphenateCharacter,
    CSSPropertyID::kAliasWebkitJustifyContent,
    CSSPropertyID::kAliasWebkitOpacity,
    CSSPropertyID::kAliasWebkitOrder,
    CSSPropertyID::kAliasWebkitPerspective,
    CSSPropertyID::kAliasWebkitPerspectiveOrigin,
    CSSPropertyID::kAliasWebkitShapeImageThreshold,
    CSSPropertyID::kAliasWebkitShapeMargin,
    CSSPropertyID::kAliasWebkitShapeOutside,
    CSSPropertyID::kAliasWebkitTextEmphasis,
    CSSPropertyID::kAliasWebkitTextEmphasisColor,
    CSSPropertyID::kAliasWebkitTextEmphasisPosition,
    CSSPropertyID::kAliasWebkitTextEmphasisStyle,
    CSSPropertyID::kAliasWebkitTextSizeAdjust,
    CSSPropertyID::kAliasWebkitTransform,
    CSSPropertyID::kAliasWebkitTransformOrigin,
    CSSPropertyID::kAliasWebkitTransformStyle,
    CSSPropertyID::kAliasWebkitTransition,
    CSSPropertyID::kAliasWebkitTransitionDelay,
    CSSPropertyID::kAliasWebkitTransitionDuration,
    CSSPropertyID::kAliasWebkitTransitionProperty,
    CSSPropertyID::kAliasWebkitTransitionTimingFunction,
    CSSPropertyID::kAliasWebkitUserSelect,
    CSSPropertyID::kAliasWordWrap,
    CSSPropertyID::kAliasGridColumnGap,
    CSSPropertyID::kAliasGridRowGap,
    CSSPropertyID::kAliasGridGap,
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:973-1468
pub const kCSSComputableProperties: [CSSPropertyID; 494] = [
    CSSPropertyID::kAccentColor,
    CSSPropertyID::kAlignContent,
    CSSPropertyID::kAlignItems,
    CSSPropertyID::kAlignSelf,
    CSSPropertyID::kAlignmentBaseline,
    CSSPropertyID::kAnchorName,
    CSSPropertyID::kAnchorScope,
    CSSPropertyID::kAnimationComposition,
    CSSPropertyID::kAnimationDelay,
    CSSPropertyID::kAnimationDirection,
    CSSPropertyID::kAnimationDuration,
    CSSPropertyID::kAnimationFillMode,
    CSSPropertyID::kAnimationIterationCount,
    CSSPropertyID::kAnimationName,
    CSSPropertyID::kAnimationPlayState,
    CSSPropertyID::kAnimationRangeEnd,
    CSSPropertyID::kAnimationRangeStart,
    CSSPropertyID::kAnimationTimeline,
    CSSPropertyID::kAnimationTimingFunction,
    CSSPropertyID::kAnimationTrigger,
    CSSPropertyID::kAppRegion,
    CSSPropertyID::kAppearance,
    CSSPropertyID::kAspectRatio,
    CSSPropertyID::kBackdropFilter,
    CSSPropertyID::kBackfaceVisibility,
    CSSPropertyID::kBackgroundAttachment,
    CSSPropertyID::kBackgroundBlendMode,
    CSSPropertyID::kBackgroundClip,
    CSSPropertyID::kBackgroundColor,
    CSSPropertyID::kBackgroundImage,
    CSSPropertyID::kBackgroundOrigin,
    CSSPropertyID::kBackgroundPosition,
    CSSPropertyID::kBackgroundRepeat,
    CSSPropertyID::kBackgroundSize,
    CSSPropertyID::kBaselineShift,
    CSSPropertyID::kBaselineSource,
    CSSPropertyID::kBlockEllipsis,
    CSSPropertyID::kBlockSize,
    CSSPropertyID::kBorderBlockEndColor,
    CSSPropertyID::kBorderBlockEndStyle,
    CSSPropertyID::kBorderBlockEndWidth,
    CSSPropertyID::kBorderBlockStartColor,
    CSSPropertyID::kBorderBlockStartStyle,
    CSSPropertyID::kBorderBlockStartWidth,
    CSSPropertyID::kBorderBottomColor,
    CSSPropertyID::kBorderBottomLeftRadius,
    CSSPropertyID::kBorderBottomRightRadius,
    CSSPropertyID::kBorderBottomStyle,
    CSSPropertyID::kBorderBottomWidth,
    CSSPropertyID::kBorderCollapse,
    CSSPropertyID::kBorderEndEndRadius,
    CSSPropertyID::kBorderEndStartRadius,
    CSSPropertyID::kBorderImageOutset,
    CSSPropertyID::kBorderImageRepeat,
    CSSPropertyID::kBorderImageSlice,
    CSSPropertyID::kBorderImageSource,
    CSSPropertyID::kBorderImageWidth,
    CSSPropertyID::kBorderInlineEndColor,
    CSSPropertyID::kBorderInlineEndStyle,
    CSSPropertyID::kBorderInlineEndWidth,
    CSSPropertyID::kBorderInlineStartColor,
    CSSPropertyID::kBorderInlineStartStyle,
    CSSPropertyID::kBorderInlineStartWidth,
    CSSPropertyID::kBorderLeftColor,
    CSSPropertyID::kBorderLeftStyle,
    CSSPropertyID::kBorderLeftWidth,
    CSSPropertyID::kBorderRightColor,
    CSSPropertyID::kBorderRightStyle,
    CSSPropertyID::kBorderRightWidth,
    CSSPropertyID::kBorderShape,
    CSSPropertyID::kBorderStartEndRadius,
    CSSPropertyID::kBorderStartStartRadius,
    CSSPropertyID::kBorderTopColor,
    CSSPropertyID::kBorderTopLeftRadius,
    CSSPropertyID::kBorderTopRightRadius,
    CSSPropertyID::kBorderTopStyle,
    CSSPropertyID::kBorderTopWidth,
    CSSPropertyID::kBottom,
    CSSPropertyID::kBoxDecorationBreak,
    CSSPropertyID::kBoxShadow,
    CSSPropertyID::kBoxSizing,
    CSSPropertyID::kBreakAfter,
    CSSPropertyID::kBreakBefore,
    CSSPropertyID::kBreakInside,
    CSSPropertyID::kBufferedRendering,
    CSSPropertyID::kCaptionSide,
    CSSPropertyID::kCaretAnimation,
    CSSPropertyID::kCaretColor,
    CSSPropertyID::kCaretShape,
    CSSPropertyID::kClear,
    CSSPropertyID::kClip,
    CSSPropertyID::kClipPath,
    CSSPropertyID::kClipRule,
    CSSPropertyID::kColor,
    CSSPropertyID::kColorInterpolation,
    CSSPropertyID::kColorInterpolationFilters,
    CSSPropertyID::kColorRendering,
    CSSPropertyID::kColorScheme,
    CSSPropertyID::kColumnCount,
    CSSPropertyID::kColumnFill,
    CSSPropertyID::kColumnGap,
    CSSPropertyID::kColumnHeight,
    CSSPropertyID::kColumnRuleBreak,
    CSSPropertyID::kColumnRuleColor,
    CSSPropertyID::kColumnRuleInsetCapEnd,
    CSSPropertyID::kColumnRuleInsetCapStart,
    CSSPropertyID::kColumnRuleInsetJunctionEnd,
    CSSPropertyID::kColumnRuleInsetJunctionStart,
    CSSPropertyID::kColumnRuleStyle,
    CSSPropertyID::kColumnRuleVisibilityItems,
    CSSPropertyID::kColumnRuleWidth,
    CSSPropertyID::kColumnSpan,
    CSSPropertyID::kColumnWidth,
    CSSPropertyID::kColumnWrap,
    CSSPropertyID::kContain,
    CSSPropertyID::kContainIntrinsicBlockSize,
    CSSPropertyID::kContainIntrinsicHeight,
    CSSPropertyID::kContainIntrinsicInlineSize,
    CSSPropertyID::kContainIntrinsicSize,
    CSSPropertyID::kContainIntrinsicWidth,
    CSSPropertyID::kContainerName,
    CSSPropertyID::kContainerType,
    CSSPropertyID::kContent,
    CSSPropertyID::kContentVisibility,
    CSSPropertyID::kContinue,
    CSSPropertyID::kCornerBottomLeftShape,
    CSSPropertyID::kCornerBottomRightShape,
    CSSPropertyID::kCornerEndEndShape,
    CSSPropertyID::kCornerEndStartShape,
    CSSPropertyID::kCornerStartEndShape,
    CSSPropertyID::kCornerStartStartShape,
    CSSPropertyID::kCornerTopLeftShape,
    CSSPropertyID::kCornerTopRightShape,
    CSSPropertyID::kCounterIncrement,
    CSSPropertyID::kCounterReset,
    CSSPropertyID::kCounterSet,
    CSSPropertyID::kCursor,
    CSSPropertyID::kCx,
    CSSPropertyID::kCy,
    CSSPropertyID::kD,
    CSSPropertyID::kDirection,
    CSSPropertyID::kDisplay,
    CSSPropertyID::kDominantBaseline,
    CSSPropertyID::kDynamicRangeLimit,
    CSSPropertyID::kEmptyCells,
    CSSPropertyID::kFieldSizing,
    CSSPropertyID::kFill,
    CSSPropertyID::kFillOpacity,
    CSSPropertyID::kFillRule,
    CSSPropertyID::kFilter,
    CSSPropertyID::kFlexBasis,
    CSSPropertyID::kFlexDirection,
    CSSPropertyID::kFlexGrow,
    CSSPropertyID::kFlexLineCount,
    CSSPropertyID::kFlexShrink,
    CSSPropertyID::kFlexWrap,
    CSSPropertyID::kFloat,
    CSSPropertyID::kFloodColor,
    CSSPropertyID::kFloodOpacity,
    CSSPropertyID::kFlowTolerance,
    CSSPropertyID::kFontFamily,
    CSSPropertyID::kFontFeatureSettings,
    CSSPropertyID::kFontKerning,
    CSSPropertyID::kFontLanguageOverride,
    CSSPropertyID::kFontOpticalSizing,
    CSSPropertyID::kFontPalette,
    CSSPropertyID::kFontSize,
    CSSPropertyID::kFontSizeAdjust,
    CSSPropertyID::kFontStretch,
    CSSPropertyID::kFontStyle,
    CSSPropertyID::kFontSynthesisSmallCaps,
    CSSPropertyID::kFontSynthesisStyle,
    CSSPropertyID::kFontSynthesisWeight,
    CSSPropertyID::kFontVariant,
    CSSPropertyID::kFontVariantAlternates,
    CSSPropertyID::kFontVariantCaps,
    CSSPropertyID::kFontVariantEastAsian,
    CSSPropertyID::kFontVariantEmoji,
    CSSPropertyID::kFontVariantLigatures,
    CSSPropertyID::kFontVariantNumeric,
    CSSPropertyID::kFontVariantPosition,
    CSSPropertyID::kFontVariationSettings,
    CSSPropertyID::kFontWeight,
    CSSPropertyID::kForcedColorAdjust,
    CSSPropertyID::kFrameSizing,
    CSSPropertyID::kGridAutoColumns,
    CSSPropertyID::kGridAutoFlow,
    CSSPropertyID::kGridAutoRows,
    CSSPropertyID::kGridColumnEnd,
    CSSPropertyID::kGridColumnStart,
    CSSPropertyID::kGridLanesDirection,
    CSSPropertyID::kGridLanesPack,
    CSSPropertyID::kGridRowEnd,
    CSSPropertyID::kGridRowStart,
    CSSPropertyID::kGridTemplateAreas,
    CSSPropertyID::kGridTemplateColumns,
    CSSPropertyID::kGridTemplateRows,
    CSSPropertyID::kHangingPunctuation,
    CSSPropertyID::kHeight,
    CSSPropertyID::kHyphenateCharacter,
    CSSPropertyID::kHyphenateLimitChars,
    CSSPropertyID::kHyphens,
    CSSPropertyID::kImageAnimation,
    CSSPropertyID::kImageOrientation,
    CSSPropertyID::kImageRendering,
    CSSPropertyID::kInitialLetter,
    CSSPropertyID::kInlineSize,
    CSSPropertyID::kInsetBlockEnd,
    CSSPropertyID::kInsetBlockStart,
    CSSPropertyID::kInsetInlineEnd,
    CSSPropertyID::kInsetInlineStart,
    CSSPropertyID::kInteractivity,
    CSSPropertyID::kInterestDelayEnd,
    CSSPropertyID::kInterestDelayStart,
    CSSPropertyID::kInterpolateSize,
    CSSPropertyID::kIsolation,
    CSSPropertyID::kJustifyContent,
    CSSPropertyID::kJustifyItems,
    CSSPropertyID::kJustifySelf,
    CSSPropertyID::kLeft,
    CSSPropertyID::kLetterSpacing,
    CSSPropertyID::kLightingColor,
    CSSPropertyID::kLineBreak,
    CSSPropertyID::kLineClamp,
    CSSPropertyID::kLineHeight,
    CSSPropertyID::kListStyleImage,
    CSSPropertyID::kListStylePosition,
    CSSPropertyID::kListStyleType,
    CSSPropertyID::kMarginBlockEnd,
    CSSPropertyID::kMarginBlockStart,
    CSSPropertyID::kMarginBottom,
    CSSPropertyID::kMarginInlineEnd,
    CSSPropertyID::kMarginInlineStart,
    CSSPropertyID::kMarginLeft,
    CSSPropertyID::kMarginRight,
    CSSPropertyID::kMarginTop,
    CSSPropertyID::kMarginTrim,
    CSSPropertyID::kMarkerEnd,
    CSSPropertyID::kMarkerMid,
    CSSPropertyID::kMarkerStart,
    CSSPropertyID::kMaskClip,
    CSSPropertyID::kMaskComposite,
    CSSPropertyID::kMaskImage,
    CSSPropertyID::kMaskMode,
    CSSPropertyID::kMaskOrigin,
    CSSPropertyID::kMaskPosition,
    CSSPropertyID::kMaskRepeat,
    CSSPropertyID::kMaskSize,
    CSSPropertyID::kMaskType,
    CSSPropertyID::kMathDepth,
    CSSPropertyID::kMathShift,
    CSSPropertyID::kMathStyle,
    CSSPropertyID::kMaxBlockSize,
    CSSPropertyID::kMaxContentSizing,
    CSSPropertyID::kMaxHeight,
    CSSPropertyID::kMaxInlineSize,
    CSSPropertyID::kMaxLines,
    CSSPropertyID::kMaxWidth,
    CSSPropertyID::kMinBlockSize,
    CSSPropertyID::kMinHeight,
    CSSPropertyID::kMinInlineSize,
    CSSPropertyID::kMinWidth,
    CSSPropertyID::kMixBlendMode,
    CSSPropertyID::kObjectFit,
    CSSPropertyID::kObjectPosition,
    CSSPropertyID::kObjectViewBox,
    CSSPropertyID::kOffsetAnchor,
    CSSPropertyID::kOffsetDistance,
    CSSPropertyID::kOffsetPath,
    CSSPropertyID::kOffsetPosition,
    CSSPropertyID::kOffsetRotate,
    CSSPropertyID::kOpacity,
    CSSPropertyID::kOrder,
    CSSPropertyID::kOrphans,
    CSSPropertyID::kOutlineColor,
    CSSPropertyID::kOutlineOffset,
    CSSPropertyID::kOutlineStyle,
    CSSPropertyID::kOutlineWidth,
    CSSPropertyID::kOverflowAnchor,
    CSSPropertyID::kOverflowBlock,
    CSSPropertyID::kOverflowClipMargin,
    CSSPropertyID::kOverflowInline,
    CSSPropertyID::kOverflowWrap,
    CSSPropertyID::kOverflowX,
    CSSPropertyID::kOverflowY,
    CSSPropertyID::kOverlay,
    CSSPropertyID::kOverscrollBehaviorBlock,
    CSSPropertyID::kOverscrollBehaviorInline,
    CSSPropertyID::kOverscrollBehaviorX,
    CSSPropertyID::kOverscrollBehaviorY,
    CSSPropertyID::kOverscrollContainerType,
    CSSPropertyID::kPaddingBlockEnd,
    CSSPropertyID::kPaddingBlockStart,
    CSSPropertyID::kPaddingBottom,
    CSSPropertyID::kPaddingInlineEnd,
    CSSPropertyID::kPaddingInlineStart,
    CSSPropertyID::kPaddingLeft,
    CSSPropertyID::kPaddingRight,
    CSSPropertyID::kPaddingTop,
    CSSPropertyID::kPaintOrder,
    CSSPropertyID::kPathLength,
    CSSPropertyID::kPerspective,
    CSSPropertyID::kPerspectiveOrigin,
    CSSPropertyID::kPointerEvents,
    CSSPropertyID::kPosition,
    CSSPropertyID::kPositionAnchor,
    CSSPropertyID::kPositionArea,
    CSSPropertyID::kPositionTryFallbacks,
    CSSPropertyID::kPositionTryOrder,
    CSSPropertyID::kPositionVisibility,
    CSSPropertyID::kPrintColorAdjust,
    CSSPropertyID::kQuotes,
    CSSPropertyID::kR,
    CSSPropertyID::kReadingFlow,
    CSSPropertyID::kReadingOrder,
    CSSPropertyID::kResize,
    CSSPropertyID::kRight,
    CSSPropertyID::kRotate,
    CSSPropertyID::kRowGap,
    CSSPropertyID::kRowRuleBreak,
    CSSPropertyID::kRowRuleColor,
    CSSPropertyID::kRowRuleInsetCapEnd,
    CSSPropertyID::kRowRuleInsetCapStart,
    CSSPropertyID::kRowRuleInsetJunctionEnd,
    CSSPropertyID::kRowRuleInsetJunctionStart,
    CSSPropertyID::kRowRuleStyle,
    CSSPropertyID::kRowRuleVisibilityItems,
    CSSPropertyID::kRowRuleWidth,
    CSSPropertyID::kRubyAlign,
    CSSPropertyID::kRubyOverhang,
    CSSPropertyID::kRubyPosition,
    CSSPropertyID::kRuleOverlap,
    CSSPropertyID::kRx,
    CSSPropertyID::kRy,
    CSSPropertyID::kScale,
    CSSPropertyID::kScrollAxisLock,
    CSSPropertyID::kScrollBehavior,
    CSSPropertyID::kScrollInitialTarget,
    CSSPropertyID::kScrollMarginBlockEnd,
    CSSPropertyID::kScrollMarginBlockStart,
    CSSPropertyID::kScrollMarginBottom,
    CSSPropertyID::kScrollMarginInlineEnd,
    CSSPropertyID::kScrollMarginInlineStart,
    CSSPropertyID::kScrollMarginLeft,
    CSSPropertyID::kScrollMarginRight,
    CSSPropertyID::kScrollMarginTop,
    CSSPropertyID::kScrollMarkerGroup,
    CSSPropertyID::kScrollPaddingBlockEnd,
    CSSPropertyID::kScrollPaddingBlockStart,
    CSSPropertyID::kScrollPaddingBottom,
    CSSPropertyID::kScrollPaddingInlineEnd,
    CSSPropertyID::kScrollPaddingInlineStart,
    CSSPropertyID::kScrollPaddingLeft,
    CSSPropertyID::kScrollPaddingRight,
    CSSPropertyID::kScrollPaddingTop,
    CSSPropertyID::kScrollSnapAlign,
    CSSPropertyID::kScrollSnapStop,
    CSSPropertyID::kScrollSnapType,
    CSSPropertyID::kScrollTargetGroup,
    CSSPropertyID::kScrollTimelineAxis,
    CSSPropertyID::kScrollTimelineName,
    CSSPropertyID::kScrollbarColor,
    CSSPropertyID::kScrollbarGutter,
    CSSPropertyID::kScrollbarWidth,
    CSSPropertyID::kShapeImageThreshold,
    CSSPropertyID::kShapeMargin,
    CSSPropertyID::kShapeOutside,
    CSSPropertyID::kShapeRendering,
    CSSPropertyID::kSpeak,
    CSSPropertyID::kStopColor,
    CSSPropertyID::kStopOpacity,
    CSSPropertyID::kStroke,
    CSSPropertyID::kStrokeDasharray,
    CSSPropertyID::kStrokeDashoffset,
    CSSPropertyID::kStrokeLinecap,
    CSSPropertyID::kStrokeLinejoin,
    CSSPropertyID::kStrokeMiterlimit,
    CSSPropertyID::kStrokeOpacity,
    CSSPropertyID::kStrokeWidth,
    CSSPropertyID::kTabSize,
    CSSPropertyID::kTableLayout,
    CSSPropertyID::kTextAlign,
    CSSPropertyID::kTextAlignLast,
    CSSPropertyID::kTextAnchor,
    CSSPropertyID::kTextAutospace,
    CSSPropertyID::kTextBoxEdge,
    CSSPropertyID::kTextBoxTrim,
    CSSPropertyID::kTextCombineUpright,
    CSSPropertyID::kTextDecoration,
    CSSPropertyID::kTextDecorationColor,
    CSSPropertyID::kTextDecorationInset,
    CSSPropertyID::kTextDecorationLine,
    CSSPropertyID::kTextDecorationSkipInk,
    CSSPropertyID::kTextDecorationSkipSpaces,
    CSSPropertyID::kTextDecorationStyle,
    CSSPropertyID::kTextDecorationThickness,
    CSSPropertyID::kTextEmphasisColor,
    CSSPropertyID::kTextEmphasisPosition,
    CSSPropertyID::kTextEmphasisStyle,
    CSSPropertyID::kTextFit,
    CSSPropertyID::kTextIndent,
    CSSPropertyID::kTextJustify,
    CSSPropertyID::kTextOrientation,
    CSSPropertyID::kTextOverflow,
    CSSPropertyID::kTextRendering,
    CSSPropertyID::kTextShadow,
    CSSPropertyID::kTextSizeAdjust,
    CSSPropertyID::kTextSpacingTrim,
    CSSPropertyID::kTextTransform,
    CSSPropertyID::kTextUnderlineOffset,
    CSSPropertyID::kTextUnderlinePosition,
    CSSPropertyID::kTextWrapMode,
    CSSPropertyID::kTextWrapStyle,
    CSSPropertyID::kTimelineScope,
    CSSPropertyID::kTimelineTriggerActivationRangeEnd,
    CSSPropertyID::kTimelineTriggerActivationRangeStart,
    CSSPropertyID::kTimelineTriggerActiveRangeEnd,
    CSSPropertyID::kTimelineTriggerActiveRangeStart,
    CSSPropertyID::kTimelineTriggerName,
    CSSPropertyID::kTimelineTriggerSource,
    CSSPropertyID::kTop,
    CSSPropertyID::kTouchAction,
    CSSPropertyID::kTransform,
    CSSPropertyID::kTransformBox,
    CSSPropertyID::kTransformOrigin,
    CSSPropertyID::kTransformStyle,
    CSSPropertyID::kTransitionBehavior,
    CSSPropertyID::kTransitionDelay,
    CSSPropertyID::kTransitionDuration,
    CSSPropertyID::kTransitionProperty,
    CSSPropertyID::kTransitionTimingFunction,
    CSSPropertyID::kTranslate,
    CSSPropertyID::kTriggerScope,
    CSSPropertyID::kUnicodeBidi,
    CSSPropertyID::kUserSelect,
    CSSPropertyID::kVectorEffect,
    CSSPropertyID::kVerticalAlign,
    CSSPropertyID::kViewTimelineAxis,
    CSSPropertyID::kViewTimelineInset,
    CSSPropertyID::kViewTimelineName,
    CSSPropertyID::kViewTransitionClass,
    CSSPropertyID::kViewTransitionGroup,
    CSSPropertyID::kViewTransitionName,
    CSSPropertyID::kViewTransitionScope,
    CSSPropertyID::kVisibility,
    CSSPropertyID::kWhiteSpaceCollapse,
    CSSPropertyID::kWidows,
    CSSPropertyID::kWidth,
    CSSPropertyID::kWillChange,
    CSSPropertyID::kWindowDrag,
    CSSPropertyID::kWordBreak,
    CSSPropertyID::kWordSpacing,
    CSSPropertyID::kWritingMode,
    CSSPropertyID::kX,
    CSSPropertyID::kY,
    CSSPropertyID::kZIndex,
    CSSPropertyID::kZoom,
    CSSPropertyID::kWebkitBorderHorizontalSpacing,
    CSSPropertyID::kWebkitBorderImage,
    CSSPropertyID::kWebkitBorderVerticalSpacing,
    CSSPropertyID::kWebkitBoxAlign,
    CSSPropertyID::kWebkitBoxDecorationBreak,
    CSSPropertyID::kWebkitBoxDirection,
    CSSPropertyID::kWebkitBoxFlex,
    CSSPropertyID::kWebkitBoxOrdinalGroup,
    CSSPropertyID::kWebkitBoxOrient,
    CSSPropertyID::kWebkitBoxPack,
    CSSPropertyID::kWebkitBoxReflect,
    CSSPropertyID::kWebkitFontSmoothing,
    CSSPropertyID::kWebkitLineBreak,
    CSSPropertyID::kAlternativeWebkitLineClampLonghand,
    CSSPropertyID::kWebkitLineClamp,
    CSSPropertyID::kWebkitLocale,
    CSSPropertyID::kWebkitMaskBoxImage,
    CSSPropertyID::kWebkitMaskBoxImageOutset,
    CSSPropertyID::kWebkitMaskBoxImageRepeat,
    CSSPropertyID::kWebkitMaskBoxImageSlice,
    CSSPropertyID::kWebkitMaskBoxImageSource,
    CSSPropertyID::kWebkitMaskBoxImageWidth,
    CSSPropertyID::kWebkitMaskPositionX,
    CSSPropertyID::kWebkitMaskPositionY,
    CSSPropertyID::kWebkitRtlOrdering,
    CSSPropertyID::kWebkitRubyPosition,
    CSSPropertyID::kWebkitTapHighlightColor,
    CSSPropertyID::kWebkitTextCombine,
    CSSPropertyID::kWebkitTextDecorationsInEffect,
    CSSPropertyID::kWebkitTextFillColor,
    CSSPropertyID::kWebkitTextOrientation,
    CSSPropertyID::kWebkitTextSecurity,
    CSSPropertyID::kWebkitTextStrokeColor,
    CSSPropertyID::kWebkitTextStrokeWidth,
    CSSPropertyID::kWebkitUserDrag,
    CSSPropertyID::kWebkitUserModify,
    CSSPropertyID::kWebkitWritingMode,
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:1470-1500
pub const kCSSIncludesCurrentColorProperties: [CSSPropertyID; 29] = [
    CSSPropertyID::kAccentColor,
    CSSPropertyID::kBackdropFilter,
    CSSPropertyID::kBackgroundColor,
    CSSPropertyID::kBackgroundImage,
    CSSPropertyID::kBorderBottomColor,
    CSSPropertyID::kBorderImageSource,
    CSSPropertyID::kBorderLeftColor,
    CSSPropertyID::kBorderRightColor,
    CSSPropertyID::kBorderTopColor,
    CSSPropertyID::kBoxShadow,
    CSSPropertyID::kCaretColor,
    CSSPropertyID::kColumnRuleColor,
    CSSPropertyID::kFill,
    CSSPropertyID::kFilter,
    CSSPropertyID::kFloodColor,
    CSSPropertyID::kLightingColor,
    CSSPropertyID::kListStyleImage,
    CSSPropertyID::kMaskImage,
    CSSPropertyID::kOutlineColor,
    CSSPropertyID::kRowRuleColor,
    CSSPropertyID::kStopColor,
    CSSPropertyID::kStroke,
    CSSPropertyID::kTextDecorationColor,
    CSSPropertyID::kTextEmphasisColor,
    CSSPropertyID::kTextShadow,
    CSSPropertyID::kWebkitBoxReflect,
    CSSPropertyID::kWebkitMaskBoxImageSource,
    CSSPropertyID::kWebkitTapHighlightColor,
    CSSPropertyID::kWebkitTextStrokeColor,
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:1529-1540
const _: () = {
    assert!((1usize << kCSSPropertyIDBitLength) > kLastUnresolvedCSSProperty as usize);
    assert!(CSSPropertyID::kColorScheme as i32 == kFirstHighPriorityCSSProperty as i32);
    assert!(CSSPropertyID::kZoom as i32 == kLastHighPriorityCSSProperty as i32);
    assert!(kLastHighPriorityCSSProperty as i32 - kFirstHighPriorityCSSProperty as i32 == 41);
};

// cpp: third_party/blink/renderer/core/css/hash_tools.h:30-39
// Rust stores a reference into the immutable string pool instead of an offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Property {
    pub name: &'static str,
    pub id_and_exposed_bit: i32,
}
pub const kNotKnownExposedPropertyBit: i32 = 0x8000;

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.cc:68-234
const ASSO_VALUES: [u16; 257] = [
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 10, 91, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 14, 221, 11, 14, 10, 783, 68, 1096, 10, 1422, 410, 10, 64, 11, 10, 11, 21, 10, 14, 12,
    231, 1633, 389, 539, 1381, 870, 10, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
    6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641, 6641,
];

fn property_hash_function(name: &[u8]) -> usize {
    let mut hval = 0;
    for index in (0..name.len()).rev() {
        let byte = name[index] as usize;
        let offset = usize::from(index == 15);
        hval += ASSO_VALUES[byte + offset] as usize;
    }
    hval
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.cc:236-1883,1897-2720
pub const PROPERTY_WORD_LIST: [Property; 821] = [
    Property {
        name: "r",
        id_and_exposed_bit: CSSPropertyID::kR as i32,
    },
    Property {
        name: "d",
        id_and_exposed_bit: CSSPropertyID::kD as i32,
    },
    Property {
        name: "top",
        id_and_exposed_bit: CSSPropertyID::kTop as i32,
    },
    Property {
        name: "all",
        id_and_exposed_bit: CSSPropertyID::kAll as i32,
    },
    Property {
        name: "src",
        id_and_exposed_bit: CSSPropertyID::kSrc as i32,
    },
    Property {
        name: "pad",
        id_and_exposed_bit: CSSPropertyID::kPad as i32,
    },
    Property {
        name: "clip",
        id_and_exposed_bit: CSSPropertyID::kClip as i32,
    },
    Property {
        name: "port",
        id_and_exposed_bit: CSSPropertyID::kPort as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "color",
        id_and_exposed_bit: CSSPropertyID::kColor as i32,
    },
    Property {
        name: "order",
        id_and_exposed_bit: CSSPropertyID::kOrder as i32,
    },
    Property {
        name: "clear",
        id_and_exposed_bit: CSSPropertyID::kClear as i32,
    },
    Property {
        name: "inset",
        id_and_exposed_bit: CSSPropertyID::kInset as i32,
    },
    Property {
        name: "scale",
        id_and_exposed_bit: CSSPropertyID::kScale as i32,
    },
    Property {
        name: "corner",
        id_and_exposed_bit: CSSPropertyID::kCorner as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "rotate",
        id_and_exposed_bit: CSSPropertyID::kRotate as i32,
    },
    Property {
        name: "content",
        id_and_exposed_bit: CSSPropertyID::kContent as i32,
    },
    Property {
        name: "contain",
        id_and_exposed_bit: CSSPropertyID::kContain as i32,
    },
    Property {
        name: "pattern",
        id_and_exposed_bit: CSSPropertyID::kPattern as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "protocol",
        id_and_exposed_bit: CSSPropertyID::kProtocol as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "position",
        id_and_exposed_bit: CSSPropertyID::kPosition as i32,
    },
    Property {
        name: "gap",
        id_and_exposed_bit: CSSPropertyID::kGap as i32,
    },
    Property {
        name: "direction",
        id_and_exposed_bit: CSSPropertyID::kDirection as i32,
    },
    Property {
        name: "container",
        id_and_exposed_bit: CSSPropertyID::kContainer as i32,
    },
    Property {
        name: "isolation",
        id_and_exposed_bit: CSSPropertyID::kIsolation as i32,
    },
    Property {
        name: "grid",
        id_and_exposed_bit: CSSPropertyID::kGrid as i32,
    },
    Property {
        name: "page",
        id_and_exposed_bit: CSSPropertyID::kPage as i32,
    },
    Property {
        name: "corner-top",
        id_and_exposed_bit: CSSPropertyID::kCornerTop as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "translate",
        id_and_exposed_bit: CSSPropertyID::kTranslate as i32,
    },
    Property {
        name: "stop-color",
        id_and_exposed_bit: CSSPropertyID::kStopColor as i32,
    },
    Property {
        name: "range",
        id_and_exposed_bit: CSSPropertyID::kRange as i32,
    },
    Property {
        name: "transition",
        id_and_exposed_bit: CSSPropertyID::kTransition as i32,
    },
    Property {
        name: "appearance",
        id_and_exposed_bit: CSSPropertyID::kAppearance as i32,
    },
    Property {
        name: "caret-color",
        id_and_exposed_bit: CSSPropertyID::kCaretColor as i32,
    },
    Property {
        name: "paint-order",
        id_and_exposed_bit: CSSPropertyID::kPaintOrder as i32,
    },
    Property {
        name: "inset-inline",
        id_and_exposed_bit: CSSPropertyID::kInsetInline as i32,
    },
    Property {
        name: "accent-color",
        id_and_exposed_bit: CSSPropertyID::kAccentColor as i32,
    },
    Property {
        name: "caption-side",
        id_and_exposed_bit: CSSPropertyID::kCaptionSide as i32,
    },
    Property {
        name: "aspect-ratio",
        id_and_exposed_bit: CSSPropertyID::kAspectRatio as i32,
    },
    Property {
        name: "padding",
        id_and_exposed_bit: CSSPropertyID::kPadding as i32,
    },
    Property {
        name: "place-content",
        id_and_exposed_bit: CSSPropertyID::kPlaceContent as i32,
    },
    Property {
        name: "position-area",
        id_and_exposed_bit: CSSPropertyID::kPositionArea as i32,
    },
    Property {
        name: "initial-letter",
        id_and_exposed_bit: CSSPropertyID::kInitialLetter as i32,
    },
    Property {
        name: "corner-end-end",
        id_and_exposed_bit: CSSPropertyID::kCornerEndEnd as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "animation",
        id_and_exposed_bit: CSSPropertyID::kAnimation as i32,
    },
    Property {
        name: "grid-area",
        id_and_exposed_bit: CSSPropertyID::kGridArea as i32,
    },
    Property {
        name: "line-clamp",
        id_and_exposed_bit: CSSPropertyID::kLineClamp as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "app-region",
        id_and_exposed_bit: CSSPropertyID::kAppRegion as i32,
    },
    Property {
        name: "inset-inline-end",
        id_and_exposed_bit: CSSPropertyID::kInsetInlineEnd as i32,
    },
    Property {
        name: "grid-lanes",
        id_and_exposed_bit: CSSPropertyID::kGridLanes as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "corner-start-end",
        id_and_exposed_bit: CSSPropertyID::kCornerStartEnd as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "place-items",
        id_and_exposed_bit: CSSPropertyID::kPlaceItems as i32,
    },
    Property {
        name: "margin",
        id_and_exposed_bit: CSSPropertyID::kMargin as i32,
    },
    Property {
        name: "corner-inline-end",
        id_and_exposed_bit: CSSPropertyID::kCornerInlineEnd as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "padding-top",
        id_and_exposed_bit: CSSPropertyID::kPaddingTop as i32,
    },
    Property {
        name: "scroll-snap-stop",
        id_and_exposed_bit: CSSPropertyID::kScrollSnapStop as i32,
    },
    Property {
        name: "align-content",
        id_and_exposed_bit: CSSPropertyID::kAlignContent as i32,
    },
    Property {
        name: "reading-order",
        id_and_exposed_bit: CSSPropertyID::kReadingOrder as i32,
    },
    Property {
        name: "timeline-scope",
        id_and_exposed_bit: CSSPropertyID::kTimelineScope as i32,
    },
    Property {
        name: "grid-gap",
        id_and_exposed_bit: CSSPropertyID::kAliasGridGap as i32,
    },
    Property {
        name: "container-name",
        id_and_exposed_bit: CSSPropertyID::kContainerName as i32,
    },
    Property {
        name: "scroll-timeline",
        id_and_exposed_bit: CSSPropertyID::kScrollTimeline as i32,
    },
    Property {
        name: "letter-spacing",
        id_and_exposed_bit: CSSPropertyID::kLetterSpacing as i32,
    },
    Property {
        name: "padding-inline",
        id_and_exposed_bit: CSSPropertyID::kPaddingInline as i32,
    },
    Property {
        name: "color-rendering",
        id_and_exposed_bit: CSSPropertyID::kColorRendering as i32,
    },
    Property {
        name: "scroll-padding",
        id_and_exposed_bit: CSSPropertyID::kScrollPadding as i32,
    },
    Property {
        name: "margin-top",
        id_and_exposed_bit: CSSPropertyID::kMarginTop as i32,
    },
    Property {
        name: "caret-animation",
        id_and_exposed_bit: CSSPropertyID::kCaretAnimation as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "page-orientation",
        id_and_exposed_bit: CSSPropertyID::kPageOrientation as i32,
    },
    Property {
        name: "align-items",
        id_and_exposed_bit: CSSPropertyID::kAlignItems as i32,
    },
    Property {
        name: "margin-inline",
        id_and_exposed_bit: CSSPropertyID::kMarginInline as i32,
    },
    Property {
        name: "scroll-margin",
        id_and_exposed_bit: CSSPropertyID::kScrollMargin as i32,
    },
    Property {
        name: "trigger-scope",
        id_and_exposed_bit: CSSPropertyID::kTriggerScope as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "grid-template",
        id_and_exposed_bit: CSSPropertyID::kGridTemplate as i32,
    },
    Property {
        name: "rule",
        id_and_exposed_bit: CSSPropertyID::kRule as i32,
    },
    Property {
        name: "animation-name",
        id_and_exposed_bit: CSSPropertyID::kAnimationName as i32,
    },
    Property {
        name: "border",
        id_and_exposed_bit: CSSPropertyID::kBorder as i32,
    },
    Property {
        name: "animation-range",
        id_and_exposed_bit: CSSPropertyID::kAnimationRange as i32,
    },
    Property {
        name: "grid-lanes-direction",
        id_and_exposed_bit: CSSPropertyID::kGridLanesDirection as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "margin-trim",
        id_and_exposed_bit: CSSPropertyID::kMarginTrim as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "padding-inline-start",
        id_and_exposed_bit: CSSPropertyID::kPaddingInlineStart as i32,
    },
    Property {
        name: "cursor",
        id_and_exposed_bit: CSSPropertyID::kCursor as i32,
    },
    Property {
        name: "result",
        id_and_exposed_bit: CSSPropertyID::kResult as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "margin-inline-end",
        id_and_exposed_bit: CSSPropertyID::kMarginInlineEnd as i32,
    },
    Property {
        name: "outline",
        id_and_exposed_bit: CSSPropertyID::kOutline as i32,
    },
    Property {
        name: "scroll-margin-top",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginTop as i32,
    },
    Property {
        name: "image-orientation",
        id_and_exposed_bit: CSSPropertyID::kImageOrientation as i32,
    },
    Property {
        name: "quotes",
        id_and_exposed_bit: CSSPropertyID::kQuotes as i32,
    },
    Property {
        name: "continue",
        id_and_exposed_bit: CSSPropertyID::kContinue as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "clip-rule",
        id_and_exposed_bit: CSSPropertyID::kClipRule as i32,
    },
    Property {
        name: "border-top",
        id_and_exposed_bit: CSSPropertyID::kBorderTop as i32,
    },
    Property {
        name: "rule-color",
        id_and_exposed_bit: CSSPropertyID::kRuleColor as i32,
    },
    Property {
        name: "scroll-margin-inline",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginInline as i32,
    },
    Property {
        name: "rule-inset",
        id_and_exposed_bit: CSSPropertyID::kRuleInset as i32,
    },
    Property {
        name: "bottom",
        id_and_exposed_bit: CSSPropertyID::kBottom as i32,
    },
    Property {
        name: "image-rendering",
        id_and_exposed_bit: CSSPropertyID::kImageRendering as i32,
    },
    Property {
        name: "grid-template-areas",
        id_and_exposed_bit: CSSPropertyID::kGridTemplateAreas as i32,
    },
    Property {
        name: "image-animation",
        id_and_exposed_bit: CSSPropertyID::kImageAnimation as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "border-color",
        id_and_exposed_bit: CSSPropertyID::kBorderColor as i32,
    },
    Property {
        name: "animation-composition",
        id_and_exposed_bit: CSSPropertyID::kAnimationComposition as i32,
    },
    Property {
        name: "timeline-trigger",
        id_and_exposed_bit: CSSPropertyID::kTimelineTrigger as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "counter-set",
        id_and_exposed_bit: CSSPropertyID::kCounterSet as i32,
    },
    Property {
        name: "user-select",
        id_and_exposed_bit: CSSPropertyID::kUserSelect as i32,
    },
    Property {
        name: "scroll-padding-bottom",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingBottom as i32,
    },
    Property {
        name: "border-inline",
        id_and_exposed_bit: CSSPropertyID::kBorderInline as i32,
    },
    Property {
        name: "base-palette",
        id_and_exposed_bit: CSSPropertyID::kBasePalette as i32,
    },
    Property {
        name: "columns",
        id_and_exposed_bit: CSSPropertyID::kColumns as i32,
    },
    Property {
        name: "outline-color",
        id_and_exposed_bit: CSSPropertyID::kOutlineColor as i32,
    },
    Property {
        name: "counter-reset",
        id_and_exposed_bit: CSSPropertyID::kCounterReset as i32,
    },
    Property {
        name: "scroll-margin-inline-end",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginInlineEnd as i32,
    },
    Property {
        name: "scrollbar-color",
        id_and_exposed_bit: CSSPropertyID::kScrollbarColor as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "rule-inset-end",
        id_and_exposed_bit: CSSPropertyID::kRuleInsetEnd as i32,
    },
    Property {
        name: "rule-inset-cap",
        id_and_exposed_bit: CSSPropertyID::kRuleInsetCap as i32,
    },
    Property {
        name: "border-collapse",
        id_and_exposed_bit: CSSPropertyID::kBorderCollapse as i32,
    },
    Property {
        name: "border-top-color",
        id_and_exposed_bit: CSSPropertyID::kBorderTopColor as i32,
    },
    Property {
        name: "border-inline-end",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineEnd as i32,
    },
    Property {
        name: "scroll-margin-inline-start",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginInlineStart as i32,
    },
    Property {
        name: "column-span",
        id_and_exposed_bit: CSSPropertyID::kColumnSpan as i32,
    },
    Property {
        name: "corner-end-start",
        id_and_exposed_bit: CSSPropertyID::kCornerEndStart as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "corner-bottom",
        id_and_exposed_bit: CSSPropertyID::kCornerBottom as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "scroll-timeline-name",
        id_and_exposed_bit: CSSPropertyID::kScrollTimelineName as i32,
    },
    Property {
        name: "animation-range-end",
        id_and_exposed_bit: CSSPropertyID::kAnimationRangeEnd as i32,
    },
    Property {
        name: "inset-inline-start",
        id_and_exposed_bit: CSSPropertyID::kInsetInlineStart as i32,
    },
    Property {
        name: "border-inline-color",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineColor as i32,
    },
    Property {
        name: "corner-start-start",
        id_and_exposed_bit: CSSPropertyID::kCornerStartStart as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "unicode-range",
        id_and_exposed_bit: CSSPropertyID::kUnicodeRange as i32,
    },
    Property {
        name: "color-interpolation",
        id_and_exposed_bit: CSSPropertyID::kColorInterpolation as i32,
    },
    Property {
        name: "border-spacing",
        id_and_exposed_bit: CSSPropertyID::kBorderSpacing as i32,
    },
    Property {
        name: "corner-inline-start",
        id_and_exposed_bit: CSSPropertyID::kCornerInlineStart as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "animation-range-start",
        id_and_exposed_bit: CSSPropertyID::kAnimationRangeStart as i32,
    },
    Property {
        name: "column-gap",
        id_and_exposed_bit: CSSPropertyID::kColumnGap as i32,
    },
    Property {
        name: "timeline-trigger-name",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerName as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "grid-column",
        id_and_exposed_bit: CSSPropertyID::kGridColumn as i32,
    },
    Property {
        name: "border-image",
        id_and_exposed_bit: CSSPropertyID::kBorderImage as i32,
    },
    Property {
        name: "border-inline-end-color",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineEndColor as i32,
    },
    Property {
        name: "counter-increment",
        id_and_exposed_bit: CSSPropertyID::kCounterIncrement as i32,
    },
    Property {
        name: "dominant-baseline",
        id_and_exposed_bit: CSSPropertyID::kDominantBaseline as i32,
    },
    Property {
        name: "speak",
        id_and_exposed_bit: CSSPropertyID::kSpeak as i32,
    },
    Property {
        name: "stroke",
        id_and_exposed_bit: CSSPropertyID::kStroke as i32,
    },
    Property {
        name: "corner-bottom-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerBottomShape as i32,
    },
    Property {
        name: "scroll-padding-top",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingTop as i32,
    },
    Property {
        name: "padding-bottom",
        id_and_exposed_bit: CSSPropertyID::kPaddingBottom as i32,
    },
    Property {
        name: "animation-direction",
        id_and_exposed_bit: CSSPropertyID::kAnimationDirection as i32,
    },
    Property {
        name: "grid-column-end",
        id_and_exposed_bit: CSSPropertyID::kGridColumnEnd as i32,
    },
    Property {
        name: "speak-as",
        id_and_exposed_bit: CSSPropertyID::kSpeakAs as i32,
    },
    Property {
        name: "mask",
        id_and_exposed_bit: CSSPropertyID::kMask as i32,
    },
    Property {
        name: "scroll-initial-target",
        id_and_exposed_bit: CSSPropertyID::kScrollInitialTarget as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "row-gap",
        id_and_exposed_bit: CSSPropertyID::kRowGap as i32,
    },
    Property {
        name: "margin-bottom",
        id_and_exposed_bit: CSSPropertyID::kMarginBottom as i32,
    },
    Property {
        name: "marker",
        id_and_exposed_bit: CSSPropertyID::kMarker as i32,
    },
    Property {
        name: "base-url",
        id_and_exposed_bit: CSSPropertyID::kBaseUrl as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "grid-row",
        id_and_exposed_bit: CSSPropertyID::kGridRow as i32,
    },
    Property {
        name: "grid-column-start",
        id_and_exposed_bit: CSSPropertyID::kGridColumnStart as i32,
    },
    Property {
        name: "border-image-repeat",
        id_and_exposed_bit: CSSPropertyID::kBorderImageRepeat as i32,
    },
    Property {
        name: "x",
        id_and_exposed_bit: CSSPropertyID::kX as i32,
    },
    Property {
        name: "margin-inline-start",
        id_and_exposed_bit: CSSPropertyID::kMarginInlineStart as i32,
    },
    Property {
        name: "scroll-target-group",
        id_and_exposed_bit: CSSPropertyID::kScrollTargetGroup as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "rx",
        id_and_exposed_bit: CSSPropertyID::kRx as i32,
    },
    Property {
        name: "cx",
        id_and_exposed_bit: CSSPropertyID::kCx as i32,
    },
    Property {
        name: "grid-column-gap",
        id_and_exposed_bit: CSSPropertyID::kAliasGridColumnGap as i32,
    },
    Property {
        name: "stroke-linecap",
        id_and_exposed_bit: CSSPropertyID::kStrokeLinecap as i32,
    },
    Property {
        name: "mask-clip",
        id_and_exposed_bit: CSSPropertyID::kMaskClip as i32,
    },
    Property {
        name: "unicode-bidi",
        id_and_exposed_bit: CSSPropertyID::kUnicodeBidi as i32,
    },
    Property {
        name: "marker-end",
        id_and_exposed_bit: CSSPropertyID::kMarkerEnd as i32,
    },
    Property {
        name: "grid-row-end",
        id_and_exposed_bit: CSSPropertyID::kGridRowEnd as i32,
    },
    Property {
        name: "word-spacing",
        id_and_exposed_bit: CSSPropertyID::kWordSpacing as i32,
    },
    Property {
        name: "border-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderRadius as i32,
    },
    Property {
        name: "mask-repeat",
        id_and_exposed_bit: CSSPropertyID::kMaskRepeat as i32,
    },
    Property {
        name: "marker-start",
        id_and_exposed_bit: CSSPropertyID::kMarkerStart as i32,
    },
    Property {
        name: "scroll-margin-bottom",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginBottom as i32,
    },
    Property {
        name: "grid-row-start",
        id_and_exposed_bit: CSSPropertyID::kGridRowStart as i32,
    },
    Property {
        name: "baseline-source",
        id_and_exposed_bit: CSSPropertyID::kBaselineSource as i32,
    },
    Property {
        name: "mask-position",
        id_and_exposed_bit: CSSPropertyID::kMaskPosition as i32,
    },
    Property {
        name: "column-rule",
        id_and_exposed_bit: CSSPropertyID::kColumnRule as i32,
    },
    Property {
        name: "mask-mode",
        id_and_exposed_bit: CSSPropertyID::kMaskMode as i32,
    },
    Property {
        name: "border-bottom",
        id_and_exposed_bit: CSSPropertyID::kBorderBottom as i32,
    },
    Property {
        name: "marker-mid",
        id_and_exposed_bit: CSSPropertyID::kMarkerMid as i32,
    },
    Property {
        name: "grid-template-columns",
        id_and_exposed_bit: CSSPropertyID::kGridTemplateColumns as i32,
    },
    Property {
        name: "writing-mode",
        id_and_exposed_bit: CSSPropertyID::kWritingMode as i32,
    },
    Property {
        name: "rule-inset-start",
        id_and_exposed_bit: CSSPropertyID::kRuleInsetStart as i32,
    },
    Property {
        name: "column-count",
        id_and_exposed_bit: CSSPropertyID::kColumnCount as i32,
    },
    Property {
        name: "grid-row-gap",
        id_and_exposed_bit: CSSPropertyID::kAliasGridRowGap as i32,
    },
    Property {
        name: "grid-lanes-pack",
        id_and_exposed_bit: CSSPropertyID::kGridLanesPack as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "mask-origin",
        id_and_exposed_bit: CSSPropertyID::kMaskOrigin as i32,
    },
    Property {
        name: "timeline-trigger-source",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerSource as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "border-inline-start",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineStart as i32,
    },
    Property {
        name: "text-indent",
        id_and_exposed_bit: CSSPropertyID::kTextIndent as i32,
    },
    Property {
        name: "transition-duration",
        id_and_exposed_bit: CSSPropertyID::kTransitionDuration as i32,
    },
    Property {
        name: "mask-composite",
        id_and_exposed_bit: CSSPropertyID::kMaskComposite as i32,
    },
    Property {
        name: "scrollbar-gutter",
        id_and_exposed_bit: CSSPropertyID::kScrollbarGutter as i32,
    },
    Property {
        name: "column-rule-color",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleColor as i32,
    },
    Property {
        name: "border-end-end-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderEndEndRadius as i32,
    },
    Property {
        name: "border-bottom-color",
        id_and_exposed_bit: CSSPropertyID::kBorderBottomColor as i32,
    },
    Property {
        name: "mask-image",
        id_and_exposed_bit: CSSPropertyID::kMaskImage as i32,
    },
    Property {
        name: "scroll-padding-block",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingBlock as i32,
    },
    Property {
        name: "row-rule",
        id_and_exposed_bit: CSSPropertyID::kRowRule as i32,
    },
    Property {
        name: "max-lines",
        id_and_exposed_bit: CSSPropertyID::kMaxLines as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "grid-template-rows",
        id_and_exposed_bit: CSSPropertyID::kGridTemplateRows as i32,
    },
    Property {
        name: "border-start-end-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderStartEndRadius as i32,
    },
    Property {
        name: "text-decoration",
        id_and_exposed_bit: CSSPropertyID::kTextDecoration as i32,
    },
    Property {
        name: "text-align",
        id_and_exposed_bit: CSSPropertyID::kTextAlign as i32,
    },
    Property {
        name: "border-inline-start-color",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineStartColor as i32,
    },
    Property {
        name: "text-orientation",
        id_and_exposed_bit: CSSPropertyID::kTextOrientation as i32,
    },
    Property {
        name: "scroll-axis-lock",
        id_and_exposed_bit: CSSPropertyID::kScrollAxisLock as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "line-break",
        id_and_exposed_bit: CSSPropertyID::kLineBreak as i32,
    },
    Property {
        name: "text-spacing",
        id_and_exposed_bit: CSSPropertyID::kTextSpacing as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "scroll-padding-block-end",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingBlockEnd as i32,
    },
    Property {
        name: "inset-block",
        id_and_exposed_bit: CSSPropertyID::kInsetBlock as i32,
    },
    Property {
        name: "text-rendering",
        id_and_exposed_bit: CSSPropertyID::kTextRendering as i32,
    },
    Property {
        name: "grid-auto-columns",
        id_and_exposed_bit: CSSPropertyID::kGridAutoColumns as i32,
    },
    Property {
        name: "row-rule-color",
        id_and_exposed_bit: CSSPropertyID::kRowRuleColor as i32,
    },
    Property {
        name: "break-inside",
        id_and_exposed_bit: CSSPropertyID::kBreakInside as i32,
    },
    Property {
        name: "row-rule-inset",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInset as i32,
    },
    Property {
        name: "scroll-padding-block-start",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingBlockStart as i32,
    },
    Property {
        name: "text-align-last",
        id_and_exposed_bit: CSSPropertyID::kTextAlignLast as i32,
    },
    Property {
        name: "block-ellipsis",
        id_and_exposed_bit: CSSPropertyID::kBlockEllipsis as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "border-top-width",
        id_and_exposed_bit: CSSPropertyID::kBorderTopWidth as i32,
    },
    Property {
        name: "animation-iteration-count",
        id_and_exposed_bit: CSSPropertyID::kAnimationIterationCount as i32,
    },
    Property {
        name: "column-wrap",
        id_and_exposed_bit: CSSPropertyID::kColumnWrap as i32,
    },
    Property {
        name: "inset-block-end",
        id_and_exposed_bit: CSSPropertyID::kInsetBlockEnd as i32,
    },
    Property {
        name: "corner-block-end",
        id_and_exposed_bit: CSSPropertyID::kCornerBlockEnd as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "row-rule-inset-cap",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetCap as i32,
    },
    Property {
        name: "inset-block-start",
        id_and_exposed_bit: CSSPropertyID::kInsetBlockStart as i32,
    },
    Property {
        name: "grid-auto-rows",
        id_and_exposed_bit: CSSPropertyID::kGridAutoRows as i32,
    },
    Property {
        name: "fill",
        id_and_exposed_bit: CSSPropertyID::kFill as i32,
    },
    Property {
        name: "padding-block",
        id_and_exposed_bit: CSSPropertyID::kPaddingBlock as i32,
    },
    Property {
        name: "left",
        id_and_exposed_bit: CSSPropertyID::kLeft as i32,
    },
    Property {
        name: "font",
        id_and_exposed_bit: CSSPropertyID::kFont as i32,
    },
    Property {
        name: "row-rule-inset-start",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetStart as i32,
    },
    Property {
        name: "widows",
        id_and_exposed_bit: CSSPropertyID::kWidows as i32,
    },
    Property {
        name: "text-decoration-line",
        id_and_exposed_bit: CSSPropertyID::kTextDecorationLine as i32,
    },
    Property {
        name: "float",
        id_and_exposed_bit: CSSPropertyID::kFloat as i32,
    },
    Property {
        name: "filter",
        id_and_exposed_bit: CSSPropertyID::kFilter as i32,
    },
    Property {
        name: "text-decoration-color",
        id_and_exposed_bit: CSSPropertyID::kTextDecorationColor as i32,
    },
    Property {
        name: "row-rule-inset-cap-end",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetCapEnd as i32,
    },
    Property {
        name: "text-decoration-inset",
        id_and_exposed_bit: CSSPropertyID::kTextDecorationInset as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "margin-block",
        id_and_exposed_bit: CSSPropertyID::kMarginBlock as i32,
    },
    Property {
        name: "page-break-inside",
        id_and_exposed_bit: CSSPropertyID::kPageBreakInside as i32,
    },
    Property {
        name: "word-wrap",
        id_and_exposed_bit: CSSPropertyID::kAliasWordWrap as i32,
    },
    Property {
        name: "padding-block-end",
        id_and_exposed_bit: CSSPropertyID::kPaddingBlockEnd as i32,
    },
    Property {
        name: "-internal-unbounded",
        id_and_exposed_bit: CSSPropertyID::kInternalUnbounded as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "row-rule-inset-cap-start",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetCapStart as i32,
    },
    Property {
        name: "scroll-timeline-axis",
        id_and_exposed_bit: CSSPropertyID::kScrollTimelineAxis as i32,
    },
    Property {
        name: "place-self",
        id_and_exposed_bit: CSSPropertyID::kPlaceSelf as i32,
    },
    Property {
        name: "corner-left",
        id_and_exposed_bit: CSSPropertyID::kCornerLeft as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "flood-color",
        id_and_exposed_bit: CSSPropertyID::kFloodColor as i32,
    },
    Property {
        name: "margin-block-end",
        id_and_exposed_bit: CSSPropertyID::kMarginBlockEnd as i32,
    },
    Property {
        name: "size",
        id_and_exposed_bit: CSSPropertyID::kSize as i32,
    },
    Property {
        name: "font-palette",
        id_and_exposed_bit: CSSPropertyID::kFontPalette as i32,
    },
    Property {
        name: "text-autospace",
        id_and_exposed_bit: CSSPropertyID::kTextAutospace as i32,
    },
    Property {
        name: "border-end-start-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderEndStartRadius as i32,
    },
    Property {
        name: "resize",
        id_and_exposed_bit: CSSPropertyID::kResize as i32,
    },
    Property {
        name: "transform",
        id_and_exposed_bit: CSSPropertyID::kTransform as i32,
    },
    Property {
        name: "border-start-start-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderStartStartRadius as i32,
    },
    Property {
        name: "corner-top-left",
        id_and_exposed_bit: CSSPropertyID::kCornerTopLeft as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "rule-break",
        id_and_exposed_bit: CSSPropertyID::kRuleBreak as i32,
    },
    Property {
        name: "scroll-marker-group",
        id_and_exposed_bit: CSSPropertyID::kScrollMarkerGroup as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "window-drag",
        id_and_exposed_bit: CSSPropertyID::kWindowDrag as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "align-self",
        id_and_exposed_bit: CSSPropertyID::kAlignSelf as i32,
    },
    Property {
        name: "border-block",
        id_and_exposed_bit: CSSPropertyID::kBorderBlock as i32,
    },
    Property {
        name: "zoom",
        id_and_exposed_bit: CSSPropertyID::kZoom as i32,
    },
    Property {
        name: "padding-left",
        id_and_exposed_bit: CSSPropertyID::kPaddingLeft as i32,
    },
    Property {
        name: "border-image-outset",
        id_and_exposed_bit: CSSPropertyID::kBorderImageOutset as i32,
    },
    Property {
        name: "inline-size",
        id_and_exposed_bit: CSSPropertyID::kInlineSize as i32,
    },
    Property {
        name: "scroll-margin-block",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginBlock as i32,
    },
    Property {
        name: "border-block-end",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockEnd as i32,
    },
    Property {
        name: "mix-blend-mode",
        id_and_exposed_bit: CSSPropertyID::kMixBlendMode as i32,
    },
    Property {
        name: "background",
        id_and_exposed_bit: CSSPropertyID::kBackground as i32,
    },
    Property {
        name: "margin-left",
        id_and_exposed_bit: CSSPropertyID::kMarginLeft as i32,
    },
    Property {
        name: "-internal-forced-color",
        id_and_exposed_bit: CSSPropertyID::kInternalForcedColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "text-wrap",
        id_and_exposed_bit: CSSPropertyID::kTextWrap as i32,
    },
    Property {
        name: "text-underline-position",
        id_and_exposed_bit: CSSPropertyID::kTextUnderlinePosition as i32,
    },
    Property {
        name: "corner-block-start",
        id_and_exposed_bit: CSSPropertyID::kCornerBlockStart as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "scroll-margin-block-end",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginBlockEnd as i32,
    },
    Property {
        name: "padding-inline-end",
        id_and_exposed_bit: CSSPropertyID::kPaddingInlineEnd as i32,
    },
    Property {
        name: "animation-fill-mode",
        id_and_exposed_bit: CSSPropertyID::kAnimationFillMode as i32,
    },
    Property {
        name: "font-optical-sizing",
        id_and_exposed_bit: CSSPropertyID::kFontOpticalSizing as i32,
    },
    Property {
        name: "transition-delay",
        id_and_exposed_bit: CSSPropertyID::kTransitionDelay as i32,
    },
    Property {
        name: "border-block-end-color",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockEndColor as i32,
    },
    Property {
        name: "scroll-margin-block-start",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginBlockStart as i32,
    },
    Property {
        name: "background-clip",
        id_and_exposed_bit: CSSPropertyID::kBackgroundClip as i32,
    },
    Property {
        name: "transform-origin",
        id_and_exposed_bit: CSSPropertyID::kTransformOrigin as i32,
    },
    Property {
        name: "-internal-align-content-block",
        id_and_exposed_bit: CSSPropertyID::kInternalAlignContentBlock as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "border-block-color",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockColor as i32,
    },
    Property {
        name: "background-color",
        id_and_exposed_bit: CSSPropertyID::kBackgroundColor as i32,
    },
    Property {
        name: "min-inline-size",
        id_and_exposed_bit: CSSPropertyID::kMinInlineSize as i32,
    },
    Property {
        name: "fill-rule",
        id_and_exposed_bit: CSSPropertyID::kFillRule as i32,
    },
    Property {
        name: "scroll-padding-left",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingLeft as i32,
    },
    Property {
        name: "word-break",
        id_and_exposed_bit: CSSPropertyID::kWordBreak as i32,
    },
    Property {
        name: "border-left",
        id_and_exposed_bit: CSSPropertyID::kBorderLeft as i32,
    },
    Property {
        name: "padding-block-start",
        id_and_exposed_bit: CSSPropertyID::kPaddingBlockStart as i32,
    },
    Property {
        name: "-epub-writing-mode",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubWritingMode as i32,
    },
    Property {
        name: "text-wrap-mode",
        id_and_exposed_bit: CSSPropertyID::kTextWrapMode as i32,
    },
    Property {
        name: "-webkit-order",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitOrder as i32,
    },
    Property {
        name: "animation-trigger",
        id_and_exposed_bit: CSSPropertyID::kAnimationTrigger as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "margin-block-start",
        id_and_exposed_bit: CSSPropertyID::kMarginBlockStart as i32,
    },
    Property {
        name: "-webkit-locale",
        id_and_exposed_bit: CSSPropertyID::kWebkitLocale as i32,
    },
    Property {
        name: "search",
        id_and_exposed_bit: CSSPropertyID::kSearch as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "column-fill",
        id_and_exposed_bit: CSSPropertyID::kColumnFill as i32,
    },
    Property {
        name: "tab-size",
        id_and_exposed_bit: CSSPropertyID::kTabSize as i32,
    },
    Property {
        name: "border-left-color",
        id_and_exposed_bit: CSSPropertyID::kBorderLeftColor as i32,
    },
    Property {
        name: "orphans",
        id_and_exposed_bit: CSSPropertyID::kOrphans as i32,
    },
    Property {
        name: "inherits",
        id_and_exposed_bit: CSSPropertyID::kInherits as i32,
    },
    Property {
        name: "clip-path",
        id_and_exposed_bit: CSSPropertyID::kClipPath as i32,
    },
    Property {
        name: "-webkit-appearance",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAppearance as i32,
    },
    Property {
        name: "-epub-word-break",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubWordBreak as i32,
    },
    Property {
        name: "-epub-text-orientation",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubTextOrientation as i32,
    },
    Property {
        name: "right",
        id_and_exposed_bit: CSSPropertyID::kRight as i32,
    },
    Property {
        name: "caret-shape",
        id_and_exposed_bit: CSSPropertyID::kCaretShape as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "corner-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerShape as i32,
    },
    Property {
        name: "anchor-scope",
        id_and_exposed_bit: CSSPropertyID::kAnchorScope as i32,
    },
    Property {
        name: "border-block-start",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockStart as i32,
    },
    Property {
        name: "-webkit-animation",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimation as i32,
    },
    Property {
        name: "hostname",
        id_and_exposed_bit: CSSPropertyID::kHostname as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "pathname",
        id_and_exposed_bit: CSSPropertyID::kPathname as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "position-anchor",
        id_and_exposed_bit: CSSPropertyID::kPositionAnchor as i32,
    },
    Property {
        name: "anchor-name",
        id_and_exposed_bit: CSSPropertyID::kAnchorName as i32,
    },
    Property {
        name: "color-scheme",
        id_and_exposed_bit: CSSPropertyID::kColorScheme as i32,
    },
    Property {
        name: "corner-right",
        id_and_exposed_bit: CSSPropertyID::kCornerRight as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-align-content",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAlignContent as i32,
    },
    Property {
        name: "scroll-snap-align",
        id_and_exposed_bit: CSSPropertyID::kScrollSnapAlign as i32,
    },
    Property {
        name: "color-interpolation-filters",
        id_and_exposed_bit: CSSPropertyID::kColorInterpolationFilters as i32,
    },
    Property {
        name: "background-repeat",
        id_and_exposed_bit: CSSPropertyID::kBackgroundRepeat as i32,
    },
    Property {
        name: "border-block-start-color",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockStartColor as i32,
    },
    Property {
        name: "-internal-forced-border-color",
        id_and_exposed_bit: CSSPropertyID::kInternalForcedBorderColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "flow-tolerance",
        id_and_exposed_bit: CSSPropertyID::kFlowTolerance as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "corner-end-end-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerEndEndShape as i32,
    },
    Property {
        name: "-internal-forced-outline-color",
        id_and_exposed_bit: CSSPropertyID::kInternalForcedOutlineColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "shape-rendering",
        id_and_exposed_bit: CSSPropertyID::kShapeRendering as i32,
    },
    Property {
        name: "text-decoration-skip-spaces",
        id_and_exposed_bit: CSSPropertyID::kTextDecorationSkipSpaces as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "background-position",
        id_and_exposed_bit: CSSPropertyID::kBackgroundPosition as i32,
    },
    Property {
        name: "-webkit-margin-start",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMarginStart as i32,
    },
    Property {
        name: "corner-start-end-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerStartEndShape as i32,
    },
    Property {
        name: "shape-margin",
        id_and_exposed_bit: CSSPropertyID::kShapeMargin as i32,
    },
    Property {
        name: "corner-inline-end-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerInlineEndShape as i32,
    },
    Property {
        name: "-webkit-animation-direction",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationDirection as i32,
    },
    Property {
        name: "-webkit-animation-name",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationName as i32,
    },
    Property {
        name: "reading-flow",
        id_and_exposed_bit: CSSPropertyID::kReadingFlow as i32,
    },
    Property {
        name: "-webkit-padding-end",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitPaddingEnd as i32,
    },
    Property {
        name: "flex",
        id_and_exposed_bit: CSSPropertyID::kFlex as i32,
    },
    Property {
        name: "lighting-color",
        id_and_exposed_bit: CSSPropertyID::kLightingColor as i32,
    },
    Property {
        name: "padding-right",
        id_and_exposed_bit: CSSPropertyID::kPaddingRight as i32,
    },
    Property {
        name: "text-box",
        id_and_exposed_bit: CSSPropertyID::kTextBox as i32,
    },
    Property {
        name: "row-rule-break",
        id_and_exposed_bit: CSSPropertyID::kRowRuleBreak as i32,
    },
    Property {
        name: "font-kerning",
        id_and_exposed_bit: CSSPropertyID::kFontKerning as i32,
    },
    Property {
        name: "box-decoration-break",
        id_and_exposed_bit: CSSPropertyID::kBoxDecorationBreak as i32,
    },
    Property {
        name: "prefix",
        id_and_exposed_bit: CSSPropertyID::kPrefix as i32,
    },
    Property {
        name: "-webkit-padding-start",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitPaddingStart as i32,
    },
    Property {
        name: "background-blend-mode",
        id_and_exposed_bit: CSSPropertyID::kBackgroundBlendMode as i32,
    },
    Property {
        name: "y",
        id_and_exposed_bit: CSSPropertyID::kY as i32,
    },
    Property {
        name: "margin-right",
        id_and_exposed_bit: CSSPropertyID::kMarginRight as i32,
    },
    Property {
        name: "transition-timing-function",
        id_and_exposed_bit: CSSPropertyID::kTransitionTimingFunction as i32,
    },
    Property {
        name: "text-fit",
        id_and_exposed_bit: CSSPropertyID::kTextFit as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "ry",
        id_and_exposed_bit: CSSPropertyID::kRy as i32,
    },
    Property {
        name: "cy",
        id_and_exposed_bit: CSSPropertyID::kCy as i32,
    },
    Property {
        name: "mask-size",
        id_and_exposed_bit: CSSPropertyID::kMaskSize as i32,
    },
    Property {
        name: "-webkit-border-start",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderStart as i32,
    },
    Property {
        name: "-webkit-columns",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumns as i32,
    },
    Property {
        name: "scroll-padding-right",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingRight as i32,
    },
    Property {
        name: "types",
        id_and_exposed_bit: CSSPropertyID::kTypes as i32,
    },
    Property {
        name: "border-shape",
        id_and_exposed_bit: CSSPropertyID::kBorderShape as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "touch-action",
        id_and_exposed_bit: CSSPropertyID::kTouchAction as i32,
    },
    Property {
        name: "-webkit-line-clamp",
        id_and_exposed_bit: CSSPropertyID::kWebkitLineClamp as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "column-rule-inset",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInset as i32,
    },
    Property {
        name: "opacity",
        id_and_exposed_bit: CSSPropertyID::kOpacity as i32,
    },
    Property {
        name: "flex-direction",
        id_and_exposed_bit: CSSPropertyID::kFlexDirection as i32,
    },
    Property {
        name: "display",
        id_and_exposed_bit: CSSPropertyID::kDisplay as i32,
    },
    Property {
        name: "shape-outside",
        id_and_exposed_bit: CSSPropertyID::kShapeOutside as i32,
    },
    Property {
        name: "text-box-trim",
        id_and_exposed_bit: CSSPropertyID::kTextBoxTrim as i32,
    },
    Property {
        name: "row-rule-inset-junction",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetJunction as i32,
    },
    Property {
        name: "z-index",
        id_and_exposed_bit: CSSPropertyID::kZIndex as i32,
    },
    Property {
        name: "text-box-edge",
        id_and_exposed_bit: CSSPropertyID::kTextBoxEdge as i32,
    },
    Property {
        name: "-webkit-column-span",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnSpan as i32,
    },
    Property {
        name: "-webkit-user-select",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitUserSelect as i32,
    },
    Property {
        name: "fallback",
        id_and_exposed_bit: CSSPropertyID::kFallback as i32,
    },
    Property {
        name: "-webkit-border-start-color",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderStartColor as i32,
    },
    Property {
        name: "border-right",
        id_and_exposed_bit: CSSPropertyID::kBorderRight as i32,
    },
    Property {
        name: "list-style",
        id_and_exposed_bit: CSSPropertyID::kListStyle as i32,
    },
    Property {
        name: "column-rule-break",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleBreak as i32,
    },
    Property {
        name: "column-rule-inset-end",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetEnd as i32,
    },
    Property {
        name: "column-rule-inset-cap",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetCap as i32,
    },
    Property {
        name: "system",
        id_and_exposed_bit: CSSPropertyID::kSystem as i32,
    },
    Property {
        name: "position-try",
        id_and_exposed_bit: CSSPropertyID::kPositionTry as i32,
    },
    Property {
        name: "break-after",
        id_and_exposed_bit: CSSPropertyID::kBreakAfter as i32,
    },
    Property {
        name: "stop-opacity",
        id_and_exposed_bit: CSSPropertyID::kStopOpacity as i32,
    },
    Property {
        name: "row-rule-inset-junction-end",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetJunctionEnd as i32,
    },
    Property {
        name: "text-transform",
        id_and_exposed_bit: CSSPropertyID::kTextTransform as i32,
    },
    Property {
        name: "column-rule-inset-start",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetStart as i32,
    },
    Property {
        name: "width",
        id_and_exposed_bit: CSSPropertyID::kWidth as i32,
    },
    Property {
        name: "container-type",
        id_and_exposed_bit: CSSPropertyID::kContainerType as i32,
    },
    Property {
        name: "-webkit-align-items",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAlignItems as i32,
    },
    Property {
        name: "border-top-left-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderTopLeftRadius as i32,
    },
    Property {
        name: "interest-delay",
        id_and_exposed_bit: CSSPropertyID::kInterestDelay as i32,
    },
    Property {
        name: "corner-top-right",
        id_and_exposed_bit: CSSPropertyID::kCornerTopRight as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "row-rule-inset-junction-start",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetJunctionStart as i32,
    },
    Property {
        name: "column-rule-inset-cap-end",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetCapEnd as i32,
    },
    Property {
        name: "empty-cells",
        id_and_exposed_bit: CSSPropertyID::kEmptyCells as i32,
    },
    Property {
        name: "backdrop-filter",
        id_and_exposed_bit: CSSPropertyID::kBackdropFilter as i32,
    },
    Property {
        name: "-webkit-animation-duration",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationDuration as i32,
    },
    Property {
        name: "corner-end-start-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerEndStartShape as i32,
    },
    Property {
        name: "position-try-order",
        id_and_exposed_bit: CSSPropertyID::kPositionTryOrder as i32,
    },
    Property {
        name: "row-rule-inset-end",
        id_and_exposed_bit: CSSPropertyID::kRowRuleInsetEnd as i32,
    },
    Property {
        name: "column-rule-inset-cap-start",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetCapStart as i32,
    },
    Property {
        name: "corner-start-start-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerStartStartShape as i32,
    },
    Property {
        name: "-webkit-mask",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMask as i32,
    },
    Property {
        name: "block-size",
        id_and_exposed_bit: CSSPropertyID::kBlockSize as i32,
    },
    Property {
        name: "corner-inline-start-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerInlineStartShape as i32,
    },
    Property {
        name: "grid-auto-flow",
        id_and_exposed_bit: CSSPropertyID::kGridAutoFlow as i32,
    },
    Property {
        name: "white-space",
        id_and_exposed_bit: CSSPropertyID::kWhiteSpace as i32,
    },
    Property {
        name: "animation-delay",
        id_and_exposed_bit: CSSPropertyID::kAnimationDelay as i32,
    },
    Property {
        name: "border-right-color",
        id_and_exposed_bit: CSSPropertyID::kBorderRightColor as i32,
    },
    Property {
        name: "interest-delay-start",
        id_and_exposed_bit: CSSPropertyID::kInterestDelayStart as i32,
    },
    Property {
        name: "max-inline-size",
        id_and_exposed_bit: CSSPropertyID::kMaxInlineSize as i32,
    },
    Property {
        name: "forced-color-adjust",
        id_and_exposed_bit: CSSPropertyID::kForcedColorAdjust as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "offset",
        id_and_exposed_bit: CSSPropertyID::kOffset as i32,
    },
    Property {
        name: "min-width",
        id_and_exposed_bit: CSSPropertyID::kMinWidth as i32,
    },
    Property {
        name: "page-break-after",
        id_and_exposed_bit: CSSPropertyID::kPageBreakAfter as i32,
    },
    Property {
        name: "-webkit-animation-iteration-count",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationIterationCount as i32,
    },
    Property {
        name: "flex-basis",
        id_and_exposed_bit: CSSPropertyID::kFlexBasis as i32,
    },
    Property {
        name: "-webkit-mask-size",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaskSize as i32,
    },
    Property {
        name: "will-change",
        id_and_exposed_bit: CSSPropertyID::kWillChange as i32,
    },
    Property {
        name: "animation-play-state",
        id_and_exposed_bit: CSSPropertyID::kAnimationPlayState as i32,
    },
    Property {
        name: "-webkit-border-radius",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderRadius as i32,
    },
    Property {
        name: "-webkit-user-drag",
        id_and_exposed_bit: CSSPropertyID::kWebkitUserDrag as i32,
    },
    Property {
        name: "-webkit-mask-repeat",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaskRepeat as i32,
    },
    Property {
        name: "-webkit-mask-position",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaskPosition as i32,
    },
    Property {
        name: "min-block-size",
        id_and_exposed_bit: CSSPropertyID::kMinBlockSize as i32,
    },
    Property {
        name: "text-decoration-skip-ink",
        id_and_exposed_bit: CSSPropertyID::kTextDecorationSkipInk as i32,
    },
    Property {
        name: "flex-line-count",
        id_and_exposed_bit: CSSPropertyID::kFlexLineCount as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-mask-composite",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaskComposite as i32,
    },
    Property {
        name: "-webkit-column-rule",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnRule as i32,
    },
    Property {
        name: "offset-rotate",
        id_and_exposed_bit: CSSPropertyID::kOffsetRotate as i32,
    },
    Property {
        name: "-webkit-column-count",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnCount as i32,
    },
    Property {
        name: "rule-style",
        id_and_exposed_bit: CSSPropertyID::kRuleStyle as i32,
    },
    Property {
        name: "scroll-padding-inline",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingInline as i32,
    },
    Property {
        name: "origin-trial-test-property",
        id_and_exposed_bit: CSSPropertyID::kOriginTrialTestProperty as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "transform-box",
        id_and_exposed_bit: CSSPropertyID::kTransformBox as i32,
    },
    Property {
        name: "offset-position",
        id_and_exposed_bit: CSSPropertyID::kOffsetPosition as i32,
    },
    Property {
        name: "border-style",
        id_and_exposed_bit: CSSPropertyID::kBorderStyle as i32,
    },
    Property {
        name: "symbols",
        id_and_exposed_bit: CSSPropertyID::kSymbols as i32,
    },
    Property {
        name: "animation-timeline",
        id_and_exposed_bit: CSSPropertyID::kAnimationTimeline as i32,
    },
    Property {
        name: "offset-distance",
        id_and_exposed_bit: CSSPropertyID::kOffsetDistance as i32,
    },
    Property {
        name: "break-before",
        id_and_exposed_bit: CSSPropertyID::kBreakBefore as i32,
    },
    Property {
        name: "font-language-override",
        id_and_exposed_bit: CSSPropertyID::kFontLanguageOverride as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "font-size",
        id_and_exposed_bit: CSSPropertyID::kFontSize as i32,
    },
    Property {
        name: "outline-style",
        id_and_exposed_bit: CSSPropertyID::kOutlineStyle as i32,
    },
    Property {
        name: "text-anchor",
        id_and_exposed_bit: CSSPropertyID::kTextAnchor as i32,
    },
    Property {
        name: "white-space-collapse",
        id_and_exposed_bit: CSSPropertyID::kWhiteSpaceCollapse as i32,
    },
    Property {
        name: "perspective",
        id_and_exposed_bit: CSSPropertyID::kPerspective as i32,
    },
    Property {
        name: "-webkit-column-rule-color",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnRuleColor as i32,
    },
    Property {
        name: "scroll-padding-inline-end",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingInlineEnd as i32,
    },
    Property {
        name: "box-sizing",
        id_and_exposed_bit: CSSPropertyID::kBoxSizing as i32,
    },
    Property {
        name: "negative",
        id_and_exposed_bit: CSSPropertyID::kNegative as i32,
    },
    Property {
        name: "-webkit-writing-mode",
        id_and_exposed_bit: CSSPropertyID::kWebkitWritingMode as i32,
    },
    Property {
        name: "scroll-padding-inline-start",
        id_and_exposed_bit: CSSPropertyID::kScrollPaddingInlineStart as i32,
    },
    Property {
        name: "pointer-events",
        id_and_exposed_bit: CSSPropertyID::kPointerEvents as i32,
    },
    Property {
        name: "flex-wrap",
        id_and_exposed_bit: CSSPropertyID::kFlexWrap as i32,
    },
    Property {
        name: "override-colors",
        id_and_exposed_bit: CSSPropertyID::kOverrideColors as i32,
    },
    Property {
        name: "object-position",
        id_and_exposed_bit: CSSPropertyID::kObjectPosition as i32,
    },
    Property {
        name: "ascent-override",
        id_and_exposed_bit: CSSPropertyID::kAscentOverride as i32,
    },
    Property {
        name: "rule-width",
        id_and_exposed_bit: CSSPropertyID::kRuleWidth as i32,
    },
    Property {
        name: "navigation",
        id_and_exposed_bit: CSSPropertyID::kNavigation as i32,
    },
    Property {
        name: "list-style-position",
        id_and_exposed_bit: CSSPropertyID::kListStylePosition as i32,
    },
    Property {
        name: "border-width",
        id_and_exposed_bit: CSSPropertyID::kBorderWidth as i32,
    },
    Property {
        name: "interpolate-size",
        id_and_exposed_bit: CSSPropertyID::kInterpolateSize as i32,
    },
    Property {
        name: "text-emphasis",
        id_and_exposed_bit: CSSPropertyID::kTextEmphasis as i32,
    },
    Property {
        name: "field-sizing",
        id_and_exposed_bit: CSSPropertyID::kFieldSizing as i32,
    },
    Property {
        name: "outline-width",
        id_and_exposed_bit: CSSPropertyID::kOutlineWidth as i32,
    },
    Property {
        name: "border-inline-end-style",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineEndStyle as i32,
    },
    Property {
        name: "flex-grow",
        id_and_exposed_bit: CSSPropertyID::kFlexGrow as i32,
    },
    Property {
        name: "vertical-align",
        id_and_exposed_bit: CSSPropertyID::kVerticalAlign as i32,
    },
    Property {
        name: "page-break-before",
        id_and_exposed_bit: CSSPropertyID::kPageBreakBefore as i32,
    },
    Property {
        name: "scrollbar-width",
        id_and_exposed_bit: CSSPropertyID::kScrollbarWidth as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "scroll-margin-left",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginLeft as i32,
    },
    Property {
        name: "line-gap-override",
        id_and_exposed_bit: CSSPropertyID::kLineGapOverride as i32,
    },
    Property {
        name: "background-position-x",
        id_and_exposed_bit: CSSPropertyID::kBackgroundPositionX as i32,
    },
    Property {
        name: "column-width",
        id_and_exposed_bit: CSSPropertyID::kColumnWidth as i32,
    },
    Property {
        name: "border-block-style",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockStyle as i32,
    },
    Property {
        name: "frame-sizing",
        id_and_exposed_bit: CSSPropertyID::kFrameSizing as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "text-emphasis-color",
        id_and_exposed_bit: CSSPropertyID::kTextEmphasisColor as i32,
    },
    Property {
        name: "transition-behavior",
        id_and_exposed_bit: CSSPropertyID::kTransitionBehavior as i32,
    },
    Property {
        name: "animation-duration",
        id_and_exposed_bit: CSSPropertyID::kAnimationDuration as i32,
    },
    Property {
        name: "-webkit-box-pack",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxPack as i32,
    },
    Property {
        name: "-internal-font-size-delta",
        id_and_exposed_bit: CSSPropertyID::kInternalFontSizeDelta as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-filter",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFilter as i32,
    },
    Property {
        name: "background-size",
        id_and_exposed_bit: CSSPropertyID::kBackgroundSize as i32,
    },
    Property {
        name: "text-emphasis-position",
        id_and_exposed_bit: CSSPropertyID::kTextEmphasisPosition as i32,
    },
    Property {
        name: "outline-offset",
        id_and_exposed_bit: CSSPropertyID::kOutlineOffset as i32,
    },
    Property {
        name: "border-inline-end-width",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineEndWidth as i32,
    },
    Property {
        name: "stroke-opacity",
        id_and_exposed_bit: CSSPropertyID::kStrokeOpacity as i32,
    },
    Property {
        name: "mask-type",
        id_and_exposed_bit: CSSPropertyID::kMaskType as i32,
    },
    Property {
        name: "border-image-slice",
        id_and_exposed_bit: CSSPropertyID::kBorderImageSlice as i32,
    },
    Property {
        name: "corner-block-end-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerBlockEndShape as i32,
    },
    Property {
        name: "alignment-baseline",
        id_and_exposed_bit: CSSPropertyID::kAlignmentBaseline as i32,
    },
    Property {
        name: "table-layout",
        id_and_exposed_bit: CSSPropertyID::kTableLayout as i32,
    },
    Property {
        name: "-internal-overscroll-position",
        id_and_exposed_bit: CSSPropertyID::kInternalOverscrollPosition as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-text-combine",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextCombine as i32,
    },
    Property {
        name: "ruby-position",
        id_and_exposed_bit: CSSPropertyID::kRubyPosition as i32,
    },
    Property {
        name: "-internal-overscroll-container",
        id_and_exposed_bit: CSSPropertyID::kInternalOverscrollContainer as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "background-image",
        id_and_exposed_bit: CSSPropertyID::kBackgroundImage as i32,
    },
    Property {
        name: "-webkit-column-break-inside",
        id_and_exposed_bit: CSSPropertyID::kWebkitColumnBreakInside as i32,
    },
    Property {
        name: "-webkit-mask-image",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaskImage as i32,
    },
    Property {
        name: "ruby-align",
        id_and_exposed_bit: CSSPropertyID::kRubyAlign as i32,
    },
    Property {
        name: "rule-overlap",
        id_and_exposed_bit: CSSPropertyID::kRuleOverlap as i32,
    },
    Property {
        name: "stroke-linejoin",
        id_and_exposed_bit: CSSPropertyID::kStrokeLinejoin as i32,
    },
    Property {
        name: "syntax",
        id_and_exposed_bit: CSSPropertyID::kSyntax as i32,
    },
    Property {
        name: "-webkit-border-vertical-spacing",
        id_and_exposed_bit: CSSPropertyID::kWebkitBorderVerticalSpacing as i32,
    },
    Property {
        name: "border-image-width",
        id_and_exposed_bit: CSSPropertyID::kBorderImageWidth as i32,
    },
    Property {
        name: "initial-value",
        id_and_exposed_bit: CSSPropertyID::kInitialValue as i32,
    },
    Property {
        name: "font-stretch",
        id_and_exposed_bit: CSSPropertyID::kFontStretch as i32,
    },
    Property {
        name: "stroke-width",
        id_and_exposed_bit: CSSPropertyID::kStrokeWidth as i32,
    },
    Property {
        name: "corner-bottom-left",
        id_and_exposed_bit: CSSPropertyID::kCornerBottomLeft as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-epub-text-transform",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubTextTransform as i32,
    },
    Property {
        name: "border-inline-style",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineStyle as i32,
    },
    Property {
        name: "-webkit-transform",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransform as i32,
    },
    Property {
        name: "-internal-forced-background-color",
        id_and_exposed_bit: CSSPropertyID::kInternalForcedBackgroundColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-rtl-ordering",
        id_and_exposed_bit: CSSPropertyID::kWebkitRtlOrdering as i32,
    },
    Property {
        name: "corner-top-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerTopShape as i32,
    },
    Property {
        name: "-epub-caption-side",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubCaptionSide as i32,
    },
    Property {
        name: "border-top-right-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderTopRightRadius as i32,
    },
    Property {
        name: "corner-left-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerLeftShape as i32,
    },
    Property {
        name: "-webkit-margin-end",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMarginEnd as i32,
    },
    Property {
        name: "hanging-punctuation",
        id_and_exposed_bit: CSSPropertyID::kHangingPunctuation as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "border-inline-start-style",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineStartStyle as i32,
    },
    Property {
        name: "-webkit-margin-before",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMarginBefore as i32,
    },
    Property {
        name: "column-rule-style",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleStyle as i32,
    },
    Property {
        name: "stroke-miterlimit",
        id_and_exposed_bit: CSSPropertyID::kStrokeMiterlimit as i32,
    },
    Property {
        name: "row-rule-style",
        id_and_exposed_bit: CSSPropertyID::kRowRuleStyle as i32,
    },
    Property {
        name: "text-shadow",
        id_and_exposed_bit: CSSPropertyID::kTextShadow as i32,
    },
    Property {
        name: "-webkit-text-stroke",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextStroke as i32,
    },
    Property {
        name: "-webkit-transform-origin",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransformOrigin as i32,
    },
    Property {
        name: "-webkit-padding-after",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitPaddingAfter as i32,
    },
    Property {
        name: "max-width",
        id_and_exposed_bit: CSSPropertyID::kMaxWidth as i32,
    },
    Property {
        name: "border-image-source",
        id_and_exposed_bit: CSSPropertyID::kBorderImageSource as i32,
    },
    Property {
        name: "-webkit-animation-fill-mode",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationFillMode as i32,
    },
    Property {
        name: "timeline-trigger-active-range",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerActiveRange as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-border-end",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderEnd as i32,
    },
    Property {
        name: "corner-top-left-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerTopLeftShape as i32,
    },
    Property {
        name: "corner-block-start-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerBlockStartShape as i32,
    },
    Property {
        name: "border-inline-start-width",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineStartWidth as i32,
    },
    Property {
        name: "-webkit-text-stroke-color",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextStrokeColor as i32,
    },
    Property {
        name: "view-timeline",
        id_and_exposed_bit: CSSPropertyID::kViewTimeline as i32,
    },
    Property {
        name: "-webkit-border-before",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderBefore as i32,
    },
    Property {
        name: "max-block-size",
        id_and_exposed_bit: CSSPropertyID::kMaxBlockSize as i32,
    },
    Property {
        name: "timeline-trigger-active-range-end",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerActiveRangeEnd as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "timeline-trigger-activation-range",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerActivationRange as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "row-rule-width",
        id_and_exposed_bit: CSSPropertyID::kRowRuleWidth as i32,
    },
    Property {
        name: "text-decoration-style",
        id_and_exposed_bit: CSSPropertyID::kTextDecorationStyle as i32,
    },
    Property {
        name: "-webkit-mask-position-x",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskPositionX as i32,
    },
    Property {
        name: "hash",
        id_and_exposed_bit: CSSPropertyID::kHash as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "baseline-shift",
        id_and_exposed_bit: CSSPropertyID::kBaselineShift as i32,
    },
    Property {
        name: "-webkit-border-end-color",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderEndColor as i32,
    },
    Property {
        name: "timeline-trigger-active-range-start",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerActiveRangeStart as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "text-spacing-trim",
        id_and_exposed_bit: CSSPropertyID::kTextSpacingTrim as i32,
    },
    Property {
        name: "-webkit-border-before-color",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderBeforeColor as i32,
    },
    Property {
        name: "font-style",
        id_and_exposed_bit: CSSPropertyID::kFontStyle as i32,
    },
    Property {
        name: "timeline-trigger-activation-range-end",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerActivationRangeEnd as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "view-timeline-inset",
        id_and_exposed_bit: CSSPropertyID::kViewTimelineInset as i32,
    },
    Property {
        name: "background-attachment",
        id_and_exposed_bit: CSSPropertyID::kBackgroundAttachment as i32,
    },
    Property {
        name: "border-bottom-style",
        id_and_exposed_bit: CSSPropertyID::kBorderBottomStyle as i32,
    },
    Property {
        name: "fill-opacity",
        id_and_exposed_bit: CSSPropertyID::kFillOpacity as i32,
    },
    Property {
        name: "rule-inset-junction",
        id_and_exposed_bit: CSSPropertyID::kRuleInsetJunction as i32,
    },
    Property {
        name: "font-feature-settings",
        id_and_exposed_bit: CSSPropertyID::kFontFeatureSettings as i32,
    },
    Property {
        name: "font-display",
        id_and_exposed_bit: CSSPropertyID::kFontDisplay as i32,
    },
    Property {
        name: "timeline-trigger-activation-range-start",
        id_and_exposed_bit: CSSPropertyID::kTimelineTriggerActivationRangeStart as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "flood-opacity",
        id_and_exposed_bit: CSSPropertyID::kFloodOpacity as i32,
    },
    Property {
        name: "height",
        id_and_exposed_bit: CSSPropertyID::kHeight as i32,
    },
    Property {
        name: "-webkit-margin-after",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMarginAfter as i32,
    },
    Property {
        name: "view-transition-scope",
        id_and_exposed_bit: CSSPropertyID::kViewTransitionScope as i32,
    },
    Property {
        name: "box-shadow",
        id_and_exposed_bit: CSSPropertyID::kBoxShadow as i32,
    },
    Property {
        name: "view-transition-class",
        id_and_exposed_bit: CSSPropertyID::kViewTransitionClass as i32,
    },
    Property {
        name: "scroll-snap-type",
        id_and_exposed_bit: CSSPropertyID::kScrollSnapType as i32,
    },
    Property {
        name: "math-depth",
        id_and_exposed_bit: CSSPropertyID::kMathDepth as i32,
    },
    Property {
        name: "line-height",
        id_and_exposed_bit: CSSPropertyID::kLineHeight as i32,
    },
    Property {
        name: "interest-delay-end",
        id_and_exposed_bit: CSSPropertyID::kInterestDelayEnd as i32,
    },
    Property {
        name: "-webkit-padding-before",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitPaddingBefore as i32,
    },
    Property {
        name: "path-length",
        id_and_exposed_bit: CSSPropertyID::kPathLength as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "transition-property",
        id_and_exposed_bit: CSSPropertyID::kTransitionProperty as i32,
    },
    Property {
        name: "view-transition-name",
        id_and_exposed_bit: CSSPropertyID::kViewTransitionName as i32,
    },
    Property {
        name: "column-rule-width",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleWidth as i32,
    },
    Property {
        name: "suffix",
        id_and_exposed_bit: CSSPropertyID::kSuffix as i32,
    },
    Property {
        name: "transform-style",
        id_and_exposed_bit: CSSPropertyID::kTransformStyle as i32,
    },
    Property {
        name: "text-combine-upright",
        id_and_exposed_bit: CSSPropertyID::kTextCombineUpright as i32,
    },
    Property {
        name: "text-decoration-thickness",
        id_and_exposed_bit: CSSPropertyID::kTextDecorationThickness as i32,
    },
    Property {
        name: "min-height",
        id_and_exposed_bit: CSSPropertyID::kMinHeight as i32,
    },
    Property {
        name: "-webkit-border-after",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderAfter as i32,
    },
    Property {
        name: "border-right-style",
        id_and_exposed_bit: CSSPropertyID::kBorderRightStyle as i32,
    },
    Property {
        name: "font-weight",
        id_and_exposed_bit: CSSPropertyID::kFontWeight as i32,
    },
    Property {
        name: "-webkit-flex",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFlex as i32,
    },
    Property {
        name: "border-block-end-style",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockEndStyle as i32,
    },
    Property {
        name: "list-style-image",
        id_and_exposed_bit: CSSPropertyID::kListStyleImage as i32,
    },
    Property {
        name: "text-wrap-style",
        id_and_exposed_bit: CSSPropertyID::kTextWrapStyle as i32,
    },
    Property {
        name: "-webkit-border-after-color",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderAfterColor as i32,
    },
    Property {
        name: "animation-timing-function",
        id_and_exposed_bit: CSSPropertyID::kAnimationTimingFunction as i32,
    },
    Property {
        name: "-webkit-border-horizontal-spacing",
        id_and_exposed_bit: CSSPropertyID::kWebkitBorderHorizontalSpacing as i32,
    },
    Property {
        name: "border-block-width",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockWidth as i32,
    },
    Property {
        name: "-webkit-clip-path",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitClipPath as i32,
    },
    Property {
        name: "object-fit",
        id_and_exposed_bit: CSSPropertyID::kObjectFit as i32,
    },
    Property {
        name: "-webkit-animation-timing-function",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationTimingFunction as i32,
    },
    Property {
        name: "view-timeline-name",
        id_and_exposed_bit: CSSPropertyID::kViewTimelineName as i32,
    },
    Property {
        name: "contain-intrinsic-size",
        id_and_exposed_bit: CSSPropertyID::kContainIntrinsicSize as i32,
    },
    Property {
        name: "border-bottom-left-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderBottomLeftRadius as i32,
    },
    Property {
        name: "-epub-text-emphasis",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubTextEmphasis as i32,
    },
    Property {
        name: "border-block-end-width",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockEndWidth as i32,
    },
    Property {
        name: "-webkit-opacity",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitOpacity as i32,
    },
    Property {
        name: "text-underline-offset",
        id_and_exposed_bit: CSSPropertyID::kTextUnderlineOffset as i32,
    },
    Property {
        name: "-webkit-flex-direction",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFlexDirection as i32,
    },
    Property {
        name: "border-top-style",
        id_and_exposed_bit: CSSPropertyID::kBorderTopStyle as i32,
    },
    Property {
        name: "font-variant",
        id_and_exposed_bit: CSSPropertyID::kFontVariant as i32,
    },
    Property {
        name: "stroke-dasharray",
        id_and_exposed_bit: CSSPropertyID::kStrokeDasharray as i32,
    },
    Property {
        name: "-webkit-column-gap",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnGap as i32,
    },
    Property {
        name: "background-origin",
        id_and_exposed_bit: CSSPropertyID::kBackgroundOrigin as i32,
    },
    Property {
        name: "flex-flow",
        id_and_exposed_bit: CSSPropertyID::kFlexFlow as i32,
    },
    Property {
        name: "-webkit-line-break",
        id_and_exposed_bit: CSSPropertyID::kWebkitLineBreak as i32,
    },
    Property {
        name: "descent-override",
        id_and_exposed_bit: CSSPropertyID::kDescentOverride as i32,
    },
    Property {
        name: "-epub-text-emphasis-color",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubTextEmphasisColor as i32,
    },
    Property {
        name: "-webkit-text-fill-color",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextFillColor as i32,
    },
    Property {
        name: "contain-intrinsic-inline-size",
        id_and_exposed_bit: CSSPropertyID::kContainIntrinsicInlineSize as i32,
    },
    Property {
        name: "view-transition-group",
        id_and_exposed_bit: CSSPropertyID::kViewTransitionGroup as i32,
    },
    Property {
        name: "border-left-style",
        id_and_exposed_bit: CSSPropertyID::kBorderLeftStyle as i32,
    },
    Property {
        name: "-webkit-transition",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransition as i32,
    },
    Property {
        name: "font-variant-caps",
        id_and_exposed_bit: CSSPropertyID::kFontVariantCaps as i32,
    },
    Property {
        name: "-webkit-border-bottom-left-radius",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderBottomLeftRadius as i32,
    },
    Property {
        name: "-webkit-shape-margin",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitShapeMargin as i32,
    },
    Property {
        name: "size-adjust",
        id_and_exposed_bit: CSSPropertyID::kSizeAdjust as i32,
    },
    Property {
        name: "math-style",
        id_and_exposed_bit: CSSPropertyID::kMathStyle as i32,
    },
    Property {
        name: "font-variant-position",
        id_and_exposed_bit: CSSPropertyID::kFontVariantPosition as i32,
    },
    Property {
        name: "corner-right-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerRightShape as i32,
    },
    Property {
        name: "column-height",
        id_and_exposed_bit: CSSPropertyID::kColumnHeight as i32,
    },
    Property {
        name: "-webkit-box-decoration-break",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxDecorationBreak as i32,
    },
    Property {
        name: "-internal-visited-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-app-region",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAppRegion as i32,
    },
    Property {
        name: "font-variant-east-asian",
        id_and_exposed_bit: CSSPropertyID::kFontVariantEastAsian as i32,
    },
    Property {
        name: "border-block-start-style",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockStartStyle as i32,
    },
    Property {
        name: "-webkit-animation-delay",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationDelay as i32,
    },
    Property {
        name: "-webkit-transform-origin-x",
        id_and_exposed_bit: CSSPropertyID::kWebkitTransformOriginX as i32,
    },
    Property {
        name: "corner-top-right-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerTopRightShape as i32,
    },
    Property {
        name: "-webkit-box-orient",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxOrient as i32,
    },
    Property {
        name: "-webkit-flex-basis",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFlexBasis as i32,
    },
    Property {
        name: "background-position-y",
        id_and_exposed_bit: CSSPropertyID::kBackgroundPositionY as i32,
    },
    Property {
        name: "font-variation-settings",
        id_and_exposed_bit: CSSPropertyID::kFontVariationSettings as i32,
    },
    Property {
        name: "-internal-visited-caret-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedCaretColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-column-break-after",
        id_and_exposed_bit: CSSPropertyID::kWebkitColumnBreakAfter as i32,
    },
    Property {
        name: "-internal-forced-visited-color",
        id_and_exposed_bit: CSSPropertyID::kInternalForcedVisitedColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-box-direction",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxDirection as i32,
    },
    Property {
        name: "-webkit-animation-play-state",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAnimationPlayState as i32,
    },
    Property {
        name: "-webkit-border-top-left-radius",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderTopLeftRadius as i32,
    },
    Property {
        name: "-webkit-box-reflect",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxReflect as i32,
    },
    Property {
        name: "offset-path",
        id_and_exposed_bit: CSSPropertyID::kOffsetPath as i32,
    },
    Property {
        name: "border-block-start-width",
        id_and_exposed_bit: CSSPropertyID::kBorderBlockStartWidth as i32,
    },
    Property {
        name: "offset-anchor",
        id_and_exposed_bit: CSSPropertyID::kOffsetAnchor as i32,
    },
    Property {
        name: "-webkit-align-self",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitAlignSelf as i32,
    },
    Property {
        name: "-webkit-logical-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitLogicalWidth as i32,
    },
    Property {
        name: "-epub-text-combine",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubTextCombine as i32,
    },
    Property {
        name: "font-variant-numeric",
        id_and_exposed_bit: CSSPropertyID::kFontVariantNumeric as i32,
    },
    Property {
        name: "border-left-width",
        id_and_exposed_bit: CSSPropertyID::kBorderLeftWidth as i32,
    },
    Property {
        name: "-webkit-border-start-style",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderStartStyle as i32,
    },
    Property {
        name: "overflow",
        id_and_exposed_bit: CSSPropertyID::kOverflow as i32,
    },
    Property {
        name: "scroll-margin-right",
        id_and_exposed_bit: CSSPropertyID::kScrollMarginRight as i32,
    },
    Property {
        name: "font-variant-alternates",
        id_and_exposed_bit: CSSPropertyID::kFontVariantAlternates as i32,
    },
    Property {
        name: "-webkit-text-emphasis",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTextEmphasis as i32,
    },
    Property {
        name: "perspective-origin",
        id_and_exposed_bit: CSSPropertyID::kPerspectiveOrigin as i32,
    },
    Property {
        name: "-webkit-flex-grow",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFlexGrow as i32,
    },
    Property {
        name: "flex-shrink",
        id_and_exposed_bit: CSSPropertyID::kFlexShrink as i32,
    },
    Property {
        name: "-webkit-print-color-adjust",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitPrintColorAdjust as i32,
    },
    Property {
        name: "list-style-type",
        id_and_exposed_bit: CSSPropertyID::kListStyleType as i32,
    },
    Property {
        name: "max-height",
        id_and_exposed_bit: CSSPropertyID::kMaxHeight as i32,
    },
    Property {
        name: "-webkit-transition-duration",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransitionDuration as i32,
    },
    Property {
        name: "overflow-inline",
        id_and_exposed_bit: CSSPropertyID::kOverflowInline as i32,
    },
    Property {
        name: "-webkit-column-break-before",
        id_and_exposed_bit: CSSPropertyID::kWebkitColumnBreakBefore as i32,
    },
    Property {
        name: "-webkit-border-image",
        id_and_exposed_bit: CSSPropertyID::kWebkitBorderImage as i32,
    },
    Property {
        name: "-webkit-border-start-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderStartWidth as i32,
    },
    Property {
        name: "-internal-visited-outline-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedOutlineColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-text-emphasis-color",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTextEmphasisColor as i32,
    },
    Property {
        name: "-internal-visited-border-top-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderTopColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-border-bottom-right-radius",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderBottomRightRadius as i32,
    },
    Property {
        name: "-webkit-box-align",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxAlign as i32,
    },
    Property {
        name: "-webkit-text-emphasis-position",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTextEmphasisPosition as i32,
    },
    Property {
        name: "border-right-width",
        id_and_exposed_bit: CSSPropertyID::kBorderRightWidth as i32,
    },
    Property {
        name: "-webkit-ruby-position",
        id_and_exposed_bit: CSSPropertyID::kWebkitRubyPosition as i32,
    },
    Property {
        name: "-webkit-transform-origin-z",
        id_and_exposed_bit: CSSPropertyID::kWebkitTransformOriginZ as i32,
    },
    Property {
        name: "corner-bottom-right",
        id_and_exposed_bit: CSSPropertyID::kCornerBottomRight as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-perspective",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitPerspective as i32,
    },
    Property {
        name: "-webkit-mask-clip",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaskClip as i32,
    },
    Property {
        name: "-internal-visited-border-inline-end-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderInlineEndColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "position-try-fallbacks",
        id_and_exposed_bit: CSSPropertyID::kPositionTryFallbacks as i32,
    },
    Property {
        name: "-webkit-flex-wrap",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFlexWrap as i32,
    },
    Property {
        name: "-internal-visited-stroke",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedStroke as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-mask-position-y",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskPositionY as i32,
    },
    Property {
        name: "overlay",
        id_and_exposed_bit: CSSPropertyID::kOverlay as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-internal-visited-border-inline-start-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderInlineStartColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "scroll-behavior",
        id_and_exposed_bit: CSSPropertyID::kScrollBehavior as i32,
    },
    Property {
        name: "font-family",
        id_and_exposed_bit: CSSPropertyID::kFontFamily as i32,
    },
    Property {
        name: "-webkit-column-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnWidth as i32,
    },
    Property {
        name: "max-content-sizing",
        id_and_exposed_bit: CSSPropertyID::kMaxContentSizing as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "math-shift",
        id_and_exposed_bit: CSSPropertyID::kMathShift as i32,
    },
    Property {
        name: "-webkit-mask-origin",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaskOrigin as i32,
    },
    Property {
        name: "-webkit-border-top-right-radius",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderTopRightRadius as i32,
    },
    Property {
        name: "-webkit-column-rule-style",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnRuleStyle as i32,
    },
    Property {
        name: "contain-intrinsic-width",
        id_and_exposed_bit: CSSPropertyID::kContainIntrinsicWidth as i32,
    },
    Property {
        name: "interactivity",
        id_and_exposed_bit: CSSPropertyID::kInteractivity as i32,
    },
    Property {
        name: "corner-bottom-left-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerBottomLeftShape as i32,
    },
    Property {
        name: "dynamic-range-limit",
        id_and_exposed_bit: CSSPropertyID::kDynamicRangeLimit as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-perspective-origin",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitPerspectiveOrigin as i32,
    },
    Property {
        name: "column-rule-inset-junction",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetJunction as i32,
    },
    Property {
        name: "contain-intrinsic-block-size",
        id_and_exposed_bit: CSSPropertyID::kContainIntrinsicBlockSize as i32,
    },
    Property {
        name: "-webkit-text-orientation",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextOrientation as i32,
    },
    Property {
        name: "-webkit-font-smoothing",
        id_and_exposed_bit: CSSPropertyID::kWebkitFontSmoothing as i32,
    },
    Property {
        name: "-webkit-column-rule-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitColumnRuleWidth as i32,
    },
    Property {
        name: "column-rule-inset-junction-end",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetJunctionEnd as i32,
    },
    Property {
        name: "print-color-adjust",
        id_and_exposed_bit: CSSPropertyID::kPrintColorAdjust as i32,
    },
    Property {
        name: "column-rule-inset-junction-start",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleInsetJunctionStart as i32,
    },
    Property {
        name: "-internal-visited-column-rule-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedColumnRuleColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-internal-visited-border-bottom-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderBottomColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "page-margin-safety",
        id_and_exposed_bit: CSSPropertyID::kPageMarginSafety as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "overflow-wrap",
        id_and_exposed_bit: CSSPropertyID::kOverflowWrap as i32,
    },
    Property {
        name: "border-inline-width",
        id_and_exposed_bit: CSSPropertyID::kBorderInlineWidth as i32,
    },
    Property {
        name: "overflow-clip-margin",
        id_and_exposed_bit: CSSPropertyID::kOverflowClipMargin as i32,
    },
    Property {
        name: "vector-effect",
        id_and_exposed_bit: CSSPropertyID::kVectorEffect as i32,
    },
    Property {
        name: "visibility",
        id_and_exposed_bit: CSSPropertyID::kVisibility as i32,
    },
    Property {
        name: "-webkit-font-feature-settings",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFontFeatureSettings as i32,
    },
    Property {
        name: "-webkit-text-security",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextSecurity as i32,
    },
    Property {
        name: "-internal-visited-text-decoration-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedTextDecorationColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-mask-box-image",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskBoxImage as i32,
    },
    Property {
        name: "-webkit-box-shadow",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBoxShadow as i32,
    },
    Property {
        name: "font-synthesis",
        id_and_exposed_bit: CSSPropertyID::kFontSynthesis as i32,
    },
    Property {
        name: "overflow-x",
        id_and_exposed_bit: CSSPropertyID::kOverflowX as i32,
    },
    Property {
        name: "-internal-visited-fill",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedFill as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-text-size-adjust",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTextSizeAdjust as i32,
    },
    Property {
        name: "text-size-adjust",
        id_and_exposed_bit: CSSPropertyID::kTextSizeAdjust as i32,
    },
    Property {
        name: "-webkit-text-decorations-in-effect",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextDecorationsInEffect as i32,
    },
    Property {
        name: "-webkit-mask-box-image-slice",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskBoxImageSlice as i32,
    },
    Property {
        name: "text-overflow",
        id_and_exposed_bit: CSSPropertyID::kTextOverflow as i32,
    },
    Property {
        name: "additive-symbols",
        id_and_exposed_bit: CSSPropertyID::kAdditiveSymbols as i32,
    },
    Property {
        name: "-webkit-transform-style",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransformStyle as i32,
    },
    Property {
        name: "-webkit-mask-box-image-repeat",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskBoxImageRepeat as i32,
    },
    Property {
        name: "position-visibility",
        id_and_exposed_bit: CSSPropertyID::kPositionVisibility as i32,
    },
    Property {
        name: "overscroll-container-type",
        id_and_exposed_bit: CSSPropertyID::kOverscrollContainerType as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "text-emphasis-style",
        id_and_exposed_bit: CSSPropertyID::kTextEmphasisStyle as i32,
    },
    Property {
        name: "overscroll-behavior",
        id_and_exposed_bit: CSSPropertyID::kOverscrollBehavior as i32,
    },
    Property {
        name: "-webkit-transform-origin-y",
        id_and_exposed_bit: CSSPropertyID::kWebkitTransformOriginY as i32,
    },
    Property {
        name: "-webkit-background-clip",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBackgroundClip as i32,
    },
    Property {
        name: "overflow-block",
        id_and_exposed_bit: CSSPropertyID::kOverflowBlock as i32,
    },
    Property {
        name: "border-bottom-right-radius",
        id_and_exposed_bit: CSSPropertyID::kBorderBottomRightRadius as i32,
    },
    Property {
        name: "border-bottom-width",
        id_and_exposed_bit: CSSPropertyID::kBorderBottomWidth as i32,
    },
    Property {
        name: "-webkit-tap-highlight-color",
        id_and_exposed_bit: CSSPropertyID::kWebkitTapHighlightColor as i32,
    },
    Property {
        name: "font-synthesis-small-caps",
        id_and_exposed_bit: CSSPropertyID::kFontSynthesisSmallCaps as i32,
    },
    Property {
        name: "overscroll-behavior-inline",
        id_and_exposed_bit: CSSPropertyID::kOverscrollBehaviorInline as i32,
    },
    Property {
        name: "-webkit-logical-height",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitLogicalHeight as i32,
    },
    Property {
        name: "-webkit-border-end-style",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderEndStyle as i32,
    },
    Property {
        name: "-webkit-background-origin",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBackgroundOrigin as i32,
    },
    Property {
        name: "-webkit-user-modify",
        id_and_exposed_bit: CSSPropertyID::kWebkitUserModify as i32,
    },
    Property {
        name: "-webkit-flex-flow",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFlexFlow as i32,
    },
    Property {
        name: "view-timeline-axis",
        id_and_exposed_bit: CSSPropertyID::kViewTimelineAxis as i32,
    },
    Property {
        name: "hyphens",
        id_and_exposed_bit: CSSPropertyID::kHyphens as i32,
    },
    Property {
        name: "-webkit-border-before-style",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderBeforeStyle as i32,
    },
    Property {
        name: "buffered-rendering",
        id_and_exposed_bit: CSSPropertyID::kBufferedRendering as i32,
    },
    Property {
        name: "-internal-visited-border-block-end-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderBlockEndColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-internal-visited-background-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBackgroundColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-text-stroke-width",
        id_and_exposed_bit: CSSPropertyID::kWebkitTextStrokeWidth as i32,
    },
    Property {
        name: "font-size-adjust",
        id_and_exposed_bit: CSSPropertyID::kFontSizeAdjust as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-mask-box-image-source",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskBoxImageSource as i32,
    },
    Property {
        name: "-webkit-mask-box-image-outset",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskBoxImageOutset as i32,
    },
    Property {
        name: "-internal-visited-border-block-start-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderBlockStartColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-box-ordinal-group",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxOrdinalGroup as i32,
    },
    Property {
        name: "-webkit-border-end-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderEndWidth as i32,
    },
    Property {
        name: "-internal-visited-text-stroke-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedTextStrokeColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-perspective-origin-x",
        id_and_exposed_bit: CSSPropertyID::kWebkitPerspectiveOriginX as i32,
    },
    Property {
        name: "-webkit-border-before-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderBeforeWidth as i32,
    },
    Property {
        name: "-internal-visited-border-left-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderLeftColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "rule-visibility-items",
        id_and_exposed_bit: CSSPropertyID::kRuleVisibilityItems as i32,
    },
    Property {
        name: "-webkit-border-after-style",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderAfterStyle as i32,
    },
    Property {
        name: "-webkit-transition-timing-function",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransitionTimingFunction as i32,
    },
    Property {
        name: "contain-intrinsic-height",
        id_and_exposed_bit: CSSPropertyID::kContainIntrinsicHeight as i32,
    },
    Property {
        name: "-webkit-border-after-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBorderAfterWidth as i32,
    },
    Property {
        name: "-webkit-shape-outside",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitShapeOutside as i32,
    },
    Property {
        name: "justify-content",
        id_and_exposed_bit: CSSPropertyID::kJustifyContent as i32,
    },
    Property {
        name: "font-variant-ligatures",
        id_and_exposed_bit: CSSPropertyID::kFontVariantLigatures as i32,
    },
    Property {
        name: "-epub-text-emphasis-style",
        id_and_exposed_bit: CSSPropertyID::kAliasEpubTextEmphasisStyle as i32,
    },
    Property {
        name: "justify-items",
        id_and_exposed_bit: CSSPropertyID::kJustifyItems as i32,
    },
    Property {
        name: "-webkit-flex-shrink",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitFlexShrink as i32,
    },
    Property {
        name: "stroke-dashoffset",
        id_and_exposed_bit: CSSPropertyID::kStrokeDashoffset as i32,
    },
    Property {
        name: "overflow-anchor",
        id_and_exposed_bit: CSSPropertyID::kOverflowAnchor as i32,
    },
    Property {
        name: "-internal-empty-line-height",
        id_and_exposed_bit: CSSPropertyID::kInternalEmptyLineHeight as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-box-flex",
        id_and_exposed_bit: CSSPropertyID::kWebkitBoxFlex as i32,
    },
    Property {
        name: "-webkit-transition-delay",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransitionDelay as i32,
    },
    Property {
        name: "overscroll-behavior-x",
        id_and_exposed_bit: CSSPropertyID::kOverscrollBehaviorX as i32,
    },
    Property {
        name: "-internal-visited-text-fill-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedTextFillColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "font-variant-emoji",
        id_and_exposed_bit: CSSPropertyID::kFontVariantEmoji as i32,
    },
    Property {
        name: "-webkit-transition-property",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTransitionProperty as i32,
    },
    Property {
        name: "-internal-visited-border-right-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedBorderRightColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "overscroll-behavior-block",
        id_and_exposed_bit: CSSPropertyID::kOverscrollBehaviorBlock as i32,
    },
    Property {
        name: "corner-bottom-right-shape",
        id_and_exposed_bit: CSSPropertyID::kCornerBottomRightShape as i32,
    },
    Property {
        name: "row-rule-visibility-items",
        id_and_exposed_bit: CSSPropertyID::kRowRuleVisibilityItems as i32,
    },
    Property {
        name: "-webkit-min-logical-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMinLogicalWidth as i32,
    },
    Property {
        name: "overflow-y",
        id_and_exposed_bit: CSSPropertyID::kOverflowY as i32,
    },
    Property {
        name: "-webkit-box-sizing",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBoxSizing as i32,
    },
    Property {
        name: "-webkit-text-emphasis-style",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitTextEmphasisStyle as i32,
    },
    Property {
        name: "shape-image-threshold",
        id_and_exposed_bit: CSSPropertyID::kShapeImageThreshold as i32,
    },
    Property {
        name: "-webkit-background-size",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBackgroundSize as i32,
    },
    Property {
        name: "text-justify",
        id_and_exposed_bit: CSSPropertyID::kTextJustify as i32,
    },
    Property {
        name: "-internal-visited-text-emphasis-color",
        id_and_exposed_bit: CSSPropertyID::kInternalVisitedTextEmphasisColor as i32
            | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "object-view-box",
        id_and_exposed_bit: CSSPropertyID::kObjectViewBox as i32,
    },
    Property {
        name: "-webkit-perspective-origin-y",
        id_and_exposed_bit: CSSPropertyID::kWebkitPerspectiveOriginY as i32,
    },
    Property {
        name: "-webkit-shape-image-threshold",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitShapeImageThreshold as i32,
    },
    Property {
        name: "justify-self",
        id_and_exposed_bit: CSSPropertyID::kJustifySelf as i32,
    },
    Property {
        name: "ruby-overhang",
        id_and_exposed_bit: CSSPropertyID::kRubyOverhang as i32 | kNotKnownExposedPropertyBit,
    },
    Property {
        name: "-webkit-max-logical-width",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaxLogicalWidth as i32,
    },
    Property {
        name: "content-visibility",
        id_and_exposed_bit: CSSPropertyID::kContentVisibility as i32,
    },
    Property {
        name: "font-synthesis-style",
        id_and_exposed_bit: CSSPropertyID::kFontSynthesisStyle as i32,
    },
    Property {
        name: "hyphenate-character",
        id_and_exposed_bit: CSSPropertyID::kHyphenateCharacter as i32,
    },
    Property {
        name: "backface-visibility",
        id_and_exposed_bit: CSSPropertyID::kBackfaceVisibility as i32,
    },
    Property {
        name: "overscroll-behavior-y",
        id_and_exposed_bit: CSSPropertyID::kOverscrollBehaviorY as i32,
    },
    Property {
        name: "-webkit-mask-box-image-width",
        id_and_exposed_bit: CSSPropertyID::kWebkitMaskBoxImageWidth as i32,
    },
    Property {
        name: "hyphenate-limit-chars",
        id_and_exposed_bit: CSSPropertyID::kHyphenateLimitChars as i32,
    },
    Property {
        name: "-webkit-min-logical-height",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMinLogicalHeight as i32,
    },
    Property {
        name: "-webkit-justify-content",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitJustifyContent as i32,
    },
    Property {
        name: "font-synthesis-weight",
        id_and_exposed_bit: CSSPropertyID::kFontSynthesisWeight as i32,
    },
    Property {
        name: "column-rule-visibility-items",
        id_and_exposed_bit: CSSPropertyID::kColumnRuleVisibilityItems as i32,
    },
    Property {
        name: "-webkit-max-logical-height",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitMaxLogicalHeight as i32,
    },
    Property {
        name: "-webkit-hyphenate-character",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitHyphenateCharacter as i32,
    },
    Property {
        name: "-webkit-backface-visibility",
        id_and_exposed_bit: CSSPropertyID::kAliasWebkitBackfaceVisibility as i32,
    },
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.cc:2722-3389
const LOOKUP: [i16; 6641] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, -1, -1, -1, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, 3, 4, -1, -1, -1, 5, -1, -1, 6, 7, -1, -1, -1, -1, -1,
    -1, -1, 8, -1, -1, 9, 10, -1, 11, -1, 12, -1, -1, 13, -1, -1, -1, -1, -1, 14, -1, -1, -1, -1,
    -1, -1, -1, -1, 15, -1, 16, 17, -1, -1, -1, 18, -1, -1, -1, 19, -1, -1, -1, -1, 20, -1, -1, -1,
    -1, 21, 22, -1, 23, 24, 25, -1, 26, -1, 27, 28, -1, -1, -1, -1, 29, 30, -1, 31, -1, 32, -1, -1,
    -1, 33, -1, -1, -1, -1, -1, -1, 34, 35, -1, -1, -1, -1, -1, -1, 36, 37, -1, -1, -1, 38, 39, -1,
    -1, 40, -1, -1, -1, -1, 41, 42, -1, -1, -1, 43, -1, -1, -1, 44, 45, -1, -1, -1, 46, -1, -1, -1,
    -1, 47, 48, -1, -1, -1, 49, 50, 51, 52, -1, -1, -1, -1, -1, -1, 53, -1, -1, -1, -1, -1, -1, 54,
    -1, -1, -1, -1, -1, -1, -1, 55, 56, -1, 57, -1, 58, -1, -1, 59, -1, -1, -1, 60, 61, 62, 63, -1,
    64, -1, -1, 65, -1, -1, 66, -1, -1, -1, -1, -1, -1, -1, -1, 67, 68, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 69, -1, -1, 70, -1, 71, 72, -1, -1, -1, -1, -1, 73, -1, -1, -1,
    74, -1, -1, -1, -1, -1, -1, -1, -1, -1, 75, -1, -1, -1, 76, -1, -1, 77, 78, 79, -1, 80, 81, -1,
    -1, -1, -1, -1, 82, 83, -1, 84, 85, 86, -1, -1, -1, -1, -1, -1, -1, 87, -1, -1, -1, -1, -1, -1,
    88, -1, -1, -1, -1, 89, -1, -1, -1, 90, 91, -1, -1, -1, -1, 92, 93, 94, 95, 96, -1, -1, -1, 97,
    98, -1, 99, -1, 100, 101, -1, -1, -1, 102, 103, 104, -1, -1, 105, -1, -1, -1, 106, -1, -1, -1,
    -1, -1, 107, -1, -1, -1, -1, -1, -1, 108, -1, -1, 109, -1, 110, 111, 112, -1, -1, -1, -1, -1,
    -1, -1, 113, -1, -1, -1, -1, -1, -1, -1, 114, -1, -1, -1, 115, -1, 116, 117, -1, -1, 118, 119,
    -1, -1, 120, -1, -1, 121, 122, -1, -1, -1, 123, -1, -1, -1, -1, -1, -1, 124, 125, -1, -1, 126,
    127, -1, -1, -1, -1, -1, -1, 128, -1, -1, -1, -1, -1, -1, -1, 129, -1, -1, -1, -1, -1, -1, -1,
    130, 131, -1, 132, 133, 134, -1, 135, -1, -1, -1, 136, -1, -1, -1, -1, -1, -1, 137, -1, -1, -1,
    138, -1, -1, -1, -1, -1, -1, -1, -1, 139, -1, 140, -1, 141, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 142, -1, -1, 143, -1, -1, -1, -1, 144, -1, -1, -1, -1, 145, -1, -1, -1, -1, 146, -1, -1,
    -1, 147, -1, 148, -1, 149, 150, -1, -1, -1, 151, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 152, 153, 154, -1, -1, -1, -1, 155, -1, -1, -1, 156, 157, -1, 158, 159, 160, -1, -1, -1,
    -1, -1, -1, -1, 161, 162, -1, -1, 163, -1, -1, -1, -1, -1, 164, -1, -1, -1, -1, -1, 165, 166,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 167, -1, 168, 169, -1, -1, 170, -1, -1, -1, 171, -1,
    -1, -1, -1, -1, -1, -1, 172, -1, 173, -1, -1, -1, 174, -1, 175, 176, 177, 178, -1, -1, 179, -1,
    180, -1, -1, 181, -1, -1, -1, 182, -1, -1, -1, 183, -1, -1, 184, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 185, -1, -1, -1, 186, -1, -1, -1, -1, -1, -1, -1, -1, 187, -1, -1, 188, -1,
    -1, 189, -1, 190, -1, -1, -1, 191, -1, 192, 193, 194, -1, 195, -1, -1, -1, -1, -1, -1, 196, -1,
    197, -1, -1, -1, 198, 199, -1, -1, 200, -1, -1, 201, 202, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 203, -1, -1, -1, -1, -1, 204, -1, 205, -1, -1, -1, -1, 206, -1, -1, -1, -1, -1, -1,
    -1, 207, -1, 208, -1, 209, -1, -1, 210, -1, -1, 211, -1, -1, -1, 212, -1, -1, -1, -1, 213, -1,
    -1, -1, -1, 214, -1, 215, -1, -1, -1, -1, -1, 216, -1, 217, -1, -1, 218, 219, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 220, -1, -1, -1, -1, -1, -1,
    -1, -1, 221, -1, -1, -1, -1, -1, -1, 222, 223, 224, 225, 226, 227, -1, -1, -1, -1, -1, -1, -1,
    -1, 228, 229, -1, 230, -1, -1, -1, -1, -1, 231, -1, 232, -1, -1, -1, 233, -1, 234, -1, -1, -1,
    -1, -1, 235, -1, -1, -1, 236, -1, -1, -1, 237, 238, -1, 239, -1, -1, -1, -1, -1, -1, -1, 240,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 241, -1, -1, 242, -1, -1, -1, 243, 244, -1, 245,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 246, 247, -1, -1, -1, -1, 248, -1, -1, -1,
    249, -1, -1, -1, -1, -1, -1, -1, -1, -1, 250, -1, -1, -1, 251, 252, 253, -1, -1, -1, -1, -1,
    254, 255, -1, 256, 257, -1, -1, -1, -1, -1, -1, 258, -1, -1, -1, -1, -1, -1, 259, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 260, -1, 261, -1, -1, -1, -1, -1, -1, 262, -1, 263, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 264, -1, -1, -1, -1, -1, -1, -1, -1, 265, -1, -1, 266, -1, 267,
    268, -1, -1, -1, 269, 270, -1, -1, -1, -1, 271, -1, -1, -1, -1, -1, -1, -1, -1, -1, 272, -1,
    -1, -1, -1, -1, -1, -1, -1, 273, -1, 274, -1, -1, 275, -1, -1, -1, -1, 276, -1, -1, -1, -1, -1,
    -1, 277, 278, -1, 279, -1, -1, -1, 280, -1, -1, -1, -1, 281, 282, -1, -1, 283, -1, -1, -1, -1,
    -1, 284, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 285, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 286, -1, 287, -1, 288, -1, -1, -1, -1, 289, -1, -1, -1, -1, -1, -1, 290, -1, -1,
    291, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 292, 293, 294, -1, -1, -1, -1, -1, -1, -1, -1,
    295, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 296, -1, -1, -1, -1,
    297, 298, 299, -1, -1, -1, 300, -1, -1, -1, -1, -1, -1, 301, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 302, -1, 303, -1, -1, -1, 304, -1, -1, -1, 305, 306, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 307, -1, -1, -1, -1, 308, 309, -1, -1, -1, -1, -1, -1, -1, 310,
    -1, -1, 311, -1, 312, 313, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    314, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 315, -1, -1, -1, -1, 316, -1, 317, -1, -1, -1, -1,
    318, -1, -1, 319, -1, -1, -1, 320, -1, -1, -1, 321, -1, -1, 322, 323, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 324, -1, -1, -1, -1, 325, -1, 326, -1, 327, -1, -1, -1, -1, -1, 328, -1,
    329, -1, 330, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 331, -1, 332, 333, -1, -1, -1, 334, 335,
    336, 337, -1, 338, -1, -1, -1, 339, -1, 340, -1, -1, -1, -1, 341, -1, 342, 343, 344, -1, -1,
    -1, -1, -1, 345, -1, -1, -1, 346, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 347, 348, -1,
    349, -1, -1, -1, 350, 351, -1, -1, 352, 353, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 354, 355, -1, -1, -1, -1, -1, 356, -1, -1, -1, 357,
    358, -1, 359, -1, -1, -1, -1, -1, -1, -1, 360, -1, 361, -1, -1, -1, -1, -1, -1, -1, 362, 363,
    364, -1, -1, -1, 365, -1, 366, -1, -1, 367, -1, -1, -1, 368, 369, 370, -1, 371, 372, -1, -1,
    -1, -1, 373, -1, -1, -1, -1, 374, -1, -1, 375, -1, 376, -1, -1, -1, -1, -1, -1, 377, -1, -1,
    378, 379, 380, -1, -1, -1, -1, -1, 381, -1, -1, 382, -1, 383, -1, 384, -1, -1, 385, -1, -1, -1,
    -1, -1, -1, -1, -1, 386, 387, -1, 388, 389, -1, -1, 390, 391, -1, 392, -1, -1, -1, -1, 393, -1,
    -1, -1, 394, -1, -1, -1, 395, -1, -1, 396, -1, -1, -1, -1, 397, -1, 398, -1, -1, -1, -1, -1,
    -1, -1, 399, -1, -1, -1, 400, 401, -1, 402, -1, -1, -1, -1, -1, 403, -1, 404, -1, -1, -1, 405,
    406, -1, -1, -1, -1, -1, 407, -1, -1, -1, -1, -1, -1, -1, 408, 409, -1, 410, -1, -1, -1, -1,
    411, 412, -1, -1, -1, -1, -1, -1, -1, 413, -1, -1, -1, 414, -1, -1, -1, -1, 415, -1, 416, -1,
    417, -1, -1, 418, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 419, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 420, -1, 421, -1, -1, -1, -1, -1, 422, 423, -1, -1, -1, -1, -1, -1, -1, -1,
    424, 425, 426, -1, -1, -1, -1, -1, 427, -1, -1, -1, -1, 428, 429, -1, -1, -1, -1, -1, 430, -1,
    -1, -1, -1, -1, -1, 431, 432, -1, -1, 433, -1, -1, -1, -1, 434, -1, 435, -1, 436, -1, 437, -1,
    438, 439, -1, -1, 440, 441, -1, -1, -1, -1, -1, 442, -1, -1, -1, -1, 443, 444, -1, -1, -1, 445,
    -1, -1, -1, -1, -1, 446, 447, -1, -1, 448, 449, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 450, -1, -1, -1, -1, 451, -1, -1, 452, -1, 453, 454, -1, 455, -1, -1, -1, -1,
    -1, 456, -1, 457, -1, -1, -1, -1, 458, -1, -1, 459, 460, -1, -1, -1, -1, -1, -1, 461, -1, -1,
    -1, -1, -1, 462, 463, -1, -1, -1, -1, -1, -1, -1, -1, 464, -1, -1, -1, 465, -1, -1, -1, -1,
    466, -1, -1, 467, 468, -1, -1, -1, 469, -1, -1, 470, -1, -1, -1, -1, 471, -1, -1, -1, -1, -1,
    -1, -1, -1, 472, -1, -1, -1, -1, -1, -1, 473, -1, -1, -1, -1, -1, -1, -1, -1, 474, -1, 475, -1,
    -1, -1, -1, 476, 477, -1, -1, -1, 478, -1, -1, -1, -1, -1, 479, -1, -1, -1, -1, -1, 480, -1,
    -1, -1, -1, -1, -1, -1, 481, -1, -1, -1, -1, -1, -1, -1, 482, 483, -1, -1, -1, -1, -1, -1, 484,
    485, 486, -1, -1, -1, -1, -1, 487, -1, -1, 488, 489, -1, 490, -1, 491, -1, 492, -1, -1, 493,
    -1, 494, -1, 495, -1, 496, -1, -1, -1, -1, -1, -1, 497, 498, -1, -1, -1, 499, -1, -1, -1, 500,
    -1, -1, -1, -1, -1, -1, -1, -1, 501, -1, -1, 502, 503, 504, 505, -1, -1, -1, -1, -1, 506, -1,
    -1, -1, -1, -1, -1, 507, -1, -1, -1, -1, -1, 508, -1, -1, -1, -1, -1, 509, -1, 510, -1, -1,
    511, 512, 513, -1, -1, -1, -1, -1, -1, -1, -1, 514, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 515, -1, -1, 516, -1, -1, -1, -1, -1, -1, -1, -1, 517, -1, -1, -1, 518, -1,
    -1, -1, -1, -1, 519, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 520, -1, -1, -1,
    521, -1, -1, -1, 522, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 523,
    -1, -1, -1, -1, -1, 524, 525, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    526, -1, -1, 527, -1, -1, -1, -1, 528, -1, -1, -1, -1, -1, -1, -1, 529, 530, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 531, 532, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 533, -1,
    534, -1, -1, 535, 536, -1, 537, 538, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 539, -1, -1, 540, -1, 541, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 542, -1, -1, -1,
    543, -1, 544, -1, 545, -1, 546, -1, -1, -1, -1, -1, 547, 548, -1, -1, -1, -1, 549, 550, -1, -1,
    -1, -1, -1, -1, -1, 551, -1, -1, -1, -1, -1, 552, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    553, 554, 555, 556, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 557, 558, -1, 559, -1,
    -1, 560, 561, -1, -1, -1, 562, 563, -1, -1, -1, -1, 564, -1, -1, -1, -1, -1, 565, -1, -1, 566,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 567, -1, -1, -1, 568, -1,
    -1, 569, -1, 570, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 571, -1, -1,
    -1, 572, -1, -1, 573, -1, -1, 574, 575, -1, 576, -1, -1, -1, 577, -1, 578, -1, 579, -1, -1, -1,
    -1, 580, -1, -1, -1, -1, -1, -1, 581, -1, 582, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    583, -1, -1, -1, -1, -1, 584, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 585, -1, -1,
    -1, 586, -1, -1, 587, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 588, -1, -1, -1, -1, -1, -1, 589,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 590, -1, -1, -1, -1, -1, -1, -1, -1, -1, 591, -1,
    -1, -1, 592, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 593, 594, -1, 595, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 596,
    -1, 597, -1, 598, -1, -1, -1, -1, -1, -1, -1, -1, 599, 600, 601, -1, -1, 602, -1, 603, -1, 604,
    -1, -1, 605, -1, 606, -1, 607, -1, 608, -1, -1, -1, -1, -1, -1, -1, 609, 610, -1, -1, 611, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 612, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 613, -1, -1, -1, -1, 614, -1, -1, -1, -1, -1, 615, -1, -1, 616, 617, -1, -1, -1,
    618, -1, -1, -1, -1, -1, -1, 619, -1, 620, -1, -1, -1, 621, -1, -1, -1, -1, -1, -1, 622, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 623, -1, 624, -1, -1, 625, -1, -1, -1, 626, -1, -1, -1, -1, -1,
    -1, -1, -1, 627, 628, -1, -1, -1, -1, 629, -1, -1, -1, 630, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 631, -1, 632, -1, -1, -1, -1, 633, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    634, -1, -1, 635, -1, -1, 636, -1, -1, -1, -1, -1, -1, 637, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 638, 639, -1, -1, 640, 641, -1, -1, 642, -1, -1, -1, -1, -1, -1, 643, -1, -1, -1,
    644, -1, -1, -1, -1, -1, -1, -1, -1, -1, 645, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 646, -1, -1, 647, -1, 648, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 649, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 650, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 651, -1,
    -1, -1, -1, -1, -1, -1, 652, -1, -1, -1, -1, -1, -1, -1, -1, 653, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 654, -1, -1, -1, 655, 656, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 657, -1, -1, -1, -1, -1, 658, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 659, 660, -1, -1, 661, 662, -1, 663, -1, 664, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 665, 666, -1, -1, -1, -1, -1, -1, 667, 668, -1, -1, -1, -1,
    -1, -1, 669, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 670, 671, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 672, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 673, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, 674, -1, -1, -1, -1, -1, -1, -1, 675, -1, -1, 676, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, 677, -1, -1, 678, -1, 679, -1, -1, -1, -1, -1, 680, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 681, -1, -1, -1, -1, 682, -1, -1, -1, -1, -1, -1, -1, -1, 683,
    -1, -1, -1, -1, 684, -1, -1, 685, -1, -1, -1, -1, 686, 687, -1, -1, -1, -1, -1, -1, 688, -1,
    -1, -1, -1, -1, 689, -1, -1, -1, -1, 690, -1, -1, -1, -1, -1, -1, -1, -1, 691, -1, 692, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 693, -1, 694, -1, -1, -1, 695,
    -1, -1, 696, -1, -1, 697, -1, -1, -1, -1, -1, -1, -1, -1, -1, 698, -1, -1, -1, -1, 699, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 700, -1, 701, -1, -1, -1, -1, -1,
    702, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 703, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 704, 705, -1, -1, -1, -1, -1, -1, 706, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 707, -1, -1, -1, -1, -1, 708, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 709, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 710, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 711, -1, -1, -1, -1, -1, 712, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 713, -1, 714, 715, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 716, -1, -1, -1, 717, -1, -1, -1, -1, -1, 718, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 719, 720, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 721, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 722, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 723, -1, -1, 724, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 725, -1, -1, 726, -1, -1, -1, -1, -1, -1, -1,
    -1, 727, -1, -1, -1, -1, -1, -1, 728, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 729, -1, -1, 730,
    731, 732, -1, 733, -1, -1, -1, -1, -1, -1, -1, 734, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 735, -1, -1, -1, -1, 736, -1, -1, -1, 737, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 738, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 739, -1, 740, 741, -1, -1,
    -1, -1, -1, -1, -1, 742, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 743, -1, -1, 744, -1, -1, -1, -1, -1, -1, -1, 745, -1, 746, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 747, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, 748, 749, -1, 750, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 751, 752, 753, -1,
    -1, -1, -1, -1, -1, -1, -1, 754, -1, -1, -1, -1, -1, -1, -1, -1, -1, 755, -1, -1, -1, -1, -1,
    -1, -1, -1, 756, -1, -1, -1, -1, -1, -1, -1, 757, 758, -1, -1, -1, -1, -1, -1, -1, -1, -1, 759,
    -1, 760, -1, -1, 761, -1, -1, 762, -1, -1, -1, -1, -1, -1, -1, -1, 763, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, 764, -1, -1, -1, -1, -1, -1, -1, 765, -1, -1, -1, -1, -1, -1, 766, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 767, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 768, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    769, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    770, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 771, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 772, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, 773, -1, -1, -1, -1, -1, 774, -1, -1, -1, -1, -1, -1, -1, -1, -1, 775, -1, -1, -1, -1, -1,
    -1, 776, -1, -1, -1, -1, -1, 777, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 778, -1, -1, -1, -1, -1, 779, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 780, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, 781, -1, -1, 782, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 783, 784,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 785, -1, -1, -1, 786, -1, -1, -1, -1, -1, 787, -1, -1, -1,
    -1, -1, 788, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, 789, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 790, -1, -1, -1, 791, 792, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 793, -1, -1, 794, 795, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 796, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 797, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 798, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 799, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 800, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 801, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 802, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 803, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, 804, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, 805, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, 806, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 807, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 808, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 809, -1, -1, -1, -1, -1, -1, 810, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 811, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 812, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 813, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 814, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 815, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 816, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, 817, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 818, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, 819, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 820,
];

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.cc:1886-1895,3391-3414
pub fn FindProperty(name: &[u8]) -> Option<&'static Property> {
    if (1..=43).contains(&name.len()) {
        let key = property_hash_function(name);
        if key <= 6640 {
            let index = LOOKUP[key];
            if index >= 0 {
                let entry = &PROPERTY_WORD_LIST[index as usize];
                // The source first-byte/strncmp/NUL check requires exact bytes
                // and length. Case folding belongs to its caller.
                if name == entry.name.as_bytes() {
                    return Some(entry);
                }
            }
        }
    }
    None
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.cc:5087-5210
const ALIAS_LOOKUP_TABLE: [u16; 119] = [
    9, 86, 299, 565, 300, 302, 303, 304, 126, 127, 128, 129, 130, 131, 110, 111, 112, 107, 108,
    109, 290, 291, 288, 287, 355, 356, 353, 352, 254, 106, 316, 314, 311, 308, 375, 575, 576, 584,
    585, 651, 652, 154, 544, 693, 472, 474, 40, 480, 560, 41, 65, 66, 67, 568, 74, 75, 76, 77, 78,
    79, 80, 84, 90, 93, 96, 100, 114, 115, 589, 142, 143, 148, 149, 160, 165, 167, 595, 170, 175,
    177, 178, 179, 601, 216, 630, 217, 218, 631, 219, 221, 222, 13, 245, 274, 329, 330, 367, 368,
    433, 434, 435, 693, 472, 473, 474, 7, 494, 496, 497, 699, 499, 500, 501, 502, 508, 341, 167,
    386, 635,
];

pub fn ResolveCSSPropertyAlias(value: i32) -> i32 {
    ALIAS_LOOKUP_TABLE[(value - 707) as usize] as i32
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:1575-1581
pub fn ResolveCSSPropertyID(id: CSSPropertyID) -> CSSPropertyID {
    let mut int_id = id as i32;
    if IsPropertyAlias(id) {
        int_id = ResolveCSSPropertyAlias(int_id);
    }
    ConvertToCSSPropertyID(int_id)
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:1586-1606
pub struct CSSPropertyIDList;

pub struct CSSPropertyIDListIterator {
    id_: i32,
}
impl CSSPropertyIDList {
    pub fn begin(&self) -> CSSPropertyIDListIterator {
        CSSPropertyIDListIterator {
            id_: kIntFirstCSSProperty,
        }
    }
    pub fn end(&self) -> CSSPropertyIDListIterator {
        CSSPropertyIDListIterator {
            id_: kIntLastCSSProperty + 1,
        }
    }
}
impl Iterator for CSSPropertyIDListIterator {
    type Item = CSSPropertyID;
    fn next(&mut self) -> Option<Self::Item> {
        if self.id_ == kIntLastCSSProperty + 1 {
            return None;
        }
        let id = ConvertToCSSPropertyID(self.id_);
        self.id_ += 1;
        Some(id)
    }
}
impl IntoIterator for CSSPropertyIDList {
    type Item = CSSPropertyID;
    type IntoIter = CSSPropertyIDListIterator;
    fn into_iter(self) -> Self::IntoIter {
        self.begin()
    }
}

// cpp: generated properties/longhands.cc and properties/shorthands.cc:
// GetPropertyName/GetPropertyNameAtomicString/GetJSPropertyName bodies only.
pub const PROPERTY_NAMES: [Option<&str>; 826] = [
    None,                                             // kInvalid has no initialized property object.
    Some("variable"), // cpp: third_party/blink/renderer/core/css/properties/longhands/variable.h:26-30
    Some("color-scheme"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:42-44,46-49,51-53
    Some("forced-color-adjust"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:68-70,72-75,77-79
    Some("math-depth"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:97-99,101-104,106-108
    Some("position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:129-131,133-136,138-140
    Some("position-anchor"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:158-160,162-165,167-169
    Some("text-size-adjust"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:178-180,182-185,187-189
    Some("-internal-visited-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:201-203,205-208,210-212
    Some("appearance"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:227-229,231-234,236-238
    Some("color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:256-258,260-263,265-267
    Some("direction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:282-284,286-289,291-293
    Some("font-family"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:308-310,312-315,317-319
    Some("font-feature-settings"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:331-333,335-338,340-342
    Some("font-kerning"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:366-368,370-373,375-377
    Some("font-language-override"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:401-403,405-408,410-412
    Some("font-optical-sizing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:430-432,434-437,439-441
    Some("font-palette"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:459-461,463-466,468-470
    Some("font-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:494-496,498-501,503-505
    Some("font-size-adjust"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:529-531,533-536,538-540
    Some("font-stretch"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:558-560,562-565,567-569
    Some("font-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:593-595,597-600,602-604
    Some("font-synthesis-small-caps"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:622-624,626-629,631-633
    Some("font-synthesis-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:651-653,655-658,660-662
    Some("font-synthesis-weight"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:680-682,684-687,689-691
    Some("font-variant-alternates"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:709-711,713-716,718-720
    Some("font-variant-caps"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:738-740,742-745,747-749
    Some("font-variant-east-asian"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:767-769,771-774,776-778
    Some("font-variant-emoji"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:796-798,800-803,805-807
    Some("font-variant-ligatures"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:825-827,829-832,834-836
    Some("font-variant-numeric"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:854-856,858-861,863-865
    Some("font-variant-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:883-885,887-890,892-894
    Some("font-variation-settings"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:912-914,916-919,921-923
    Some("font-weight"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:941-943,945-948,950-952
    Some("position-area"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:976-978,980-983,985-987
    Some("text-orientation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:996-998,1000-1003,1005-1007
    Some("text-rendering"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1016-1018,1020-1023,1025-1027
    Some("text-spacing-trim"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1045-1047,1049-1052,1054-1056
    Some("-webkit-font-smoothing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1074-1076,1078-1081,1083-1085
    Some("-webkit-locale"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1103-1105,1107-1110,1112-1114
    Some("-webkit-text-orientation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1129-1131,1133-1136,1138-1140
    Some("-webkit-writing-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1152-1154,1156-1159,1161-1163
    Some("writing-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1175-1177,1179-1182,1184-1186
    Some("zoom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1195-1197,1199-1202,1204-1206
    Some("-internal-forced-visited-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1218-1220,1222-1225,1227-1229
    Some("-internal-visited-background-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1247-1249,1251-1254,1256-1258
    Some("-internal-visited-border-block-end-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1287-1289,1291-1294,1296-1298
    Some("-internal-visited-border-block-start-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1331-1333,1335-1338,1340-1342
    Some("-internal-visited-border-bottom-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1375-1377,1379-1382,1384-1386
    Some("-internal-visited-border-inline-end-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1424-1426,1428-1431,1433-1435
    Some("-internal-visited-border-inline-start-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1468-1470,1472-1475,1477-1479
    Some("-internal-visited-border-left-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1512-1514,1516-1519,1521-1523
    Some("-internal-visited-border-right-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1561-1563,1565-1568,1570-1572
    Some("-internal-visited-border-top-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1610-1612,1614-1617,1619-1621
    Some("-internal-visited-caret-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1659-1661,1663-1666,1668-1670
    Some("-internal-visited-column-rule-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1693-1695,1697-1700,1702-1704
    Some("-internal-visited-fill"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1731-1733,1735-1738,1740-1742
    Some("-internal-visited-outline-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1771-1773,1775-1778,1780-1782
    Some("-internal-visited-stroke"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1805-1807,1809-1812,1814-1816
    Some("-internal-visited-text-decoration-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1845-1847,1849-1852,1854-1856
    Some("-internal-visited-text-emphasis-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1885-1887,1889-1892,1894-1896
    Some("-internal-visited-text-fill-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1925-1927,1929-1932,1934-1936
    Some("-internal-visited-text-stroke-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:1965-1967,1969-1972,1974-1976
    Some("accent-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2002-2004,2006-2009,2011-2013
    Some("additive-symbols"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2031-2033,2035-2038,2040-2042
    Some("align-content"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2051-2053,2055-2058,2060-2062
    Some("align-items"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2080-2082,2084-2087,2089-2091
    Some("align-self"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2109-2111,2113-2116,2118-2120
    Some("alignment-baseline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2138-2140,2142-2145,2147-2149
    Some("all"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2167-2169,2171-2174,2176-2178
    Some("-webkit-line-clamp"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2206-2208,2210-2213,2215-2217
    Some("anchor-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2226-2228,2230-2233,2235-2237
    Some("anchor-scope"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2255-2257,2259-2262,2264-2266
    Some("animation-composition"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2290-2292,2294-2297,2299-2301
    Some("animation-delay"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2336-2338,2340-2343,2345-2347
    Some("animation-direction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2382-2384,2386-2389,2391-2393
    Some("animation-duration"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2428-2430,2432-2435,2437-2439
    Some("animation-fill-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2474-2476,2478-2481,2483-2485
    Some("animation-iteration-count"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2520-2522,2524-2527,2529-2531
    Some("animation-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2566-2568,2570-2573,2575-2577
    Some("animation-play-state"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2612-2614,2616-2619,2621-2623
    Some("animation-range-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2658-2660,2662-2665,2667-2669
    Some("animation-range-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2709-2711,2713-2716,2718-2720
    Some("animation-timeline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2760-2762,2764-2767,2769-2771
    Some("animation-timing-function"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2811-2813,2815-2818,2820-2822
    Some("animation-trigger"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2863-2865,2867-2870,2872-2874
    Some("app-region"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2909-2911,2913-2916,2918-2920
    Some("ascent-override"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2932-2934,2936-2939,2941-2943
    Some("aspect-ratio"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2952-2954,2956-2959,2961-2963
    Some("backdrop-filter"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:2981-2983,2985-2988,2990-2992
    Some("backface-visibility"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3018-3020,3022-3025,3027-3029
    Some("background-attachment"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3047-3049,3051-3054,3056-3058
    Some("background-blend-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3117-3119,3121-3124,3126-3128
    Some("background-clip"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3187-3189,3191-3194,3196-3198
    Some("background-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3234-3236,3238-3241,3243-3245
    Some("background-image"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3271-3273,3275-3278,3280-3282
    Some("background-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3341-3343,3345-3348,3350-3352
    Some("background-position-x"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3411-3413,3415-3418,3420-3422
    Some("background-position-y"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3488-3490,3492-3495,3497-3499
    Some("background-repeat"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3565-3567,3569-3572,3574-3576
    Some("background-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3635-3637,3639-3642,3644-3646
    Some("base-palette"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3716-3718,3720-3723,3725-3727
    Some("base-url"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3742-3744,3746-3749,3751-3753
    Some("baseline-shift"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3762-3764,3766-3769,3771-3773
    Some("baseline-source"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3785-3787,3789-3792,3794-3796
    Some("block-ellipsis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3820-3822,3824-3827,3829-3831
    Some("block-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3849-3851,3853-3856,3858-3860
    Some("border-block-end-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3890-3892,3894-3897,3899-3901
    Some("border-block-end-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3931-3933,3935-3938,3940-3942
    Some("border-block-end-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:3966-3968,3970-3973,3975-3977
    Some("border-block-start-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4001-4003,4005-4008,4010-4012
    Some("border-block-start-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4042-4044,4046-4049,4051-4053
    Some("border-block-start-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4077-4079,4081-4084,4086-4088
    Some("border-bottom-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4112-4114,4116-4119,4121-4123
    Some("border-bottom-left-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4158-4160,4162-4165,4167-4169
    Some("border-bottom-right-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4207-4209,4211-4214,4216-4218
    Some("border-bottom-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4256-4258,4260-4263,4265-4267
    Some("border-bottom-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4294-4296,4298-4301,4303-4305
    Some("border-collapse"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4332-4334,4336-4339,4341-4343
    Some("border-end-end-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4364-4366,4368-4371,4373-4375
    Some("border-end-start-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4405-4407,4409-4412,4414-4416
    Some("border-image-outset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4446-4448,4450-4453,4455-4457
    Some("border-image-repeat"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4496-4498,4500-4503,4505-4507
    Some("border-image-slice"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4536-4538,4540-4543,4545-4547
    Some("border-image-source"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4576-4578,4580-4583,4585-4587
    Some("border-image-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4608-4610,4612-4615,4617-4619
    Some("border-inline-end-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4658-4660,4662-4665,4667-4669
    Some("border-inline-end-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4699-4701,4703-4706,4708-4710
    Some("border-inline-end-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4734-4736,4738-4741,4743-4745
    Some("border-inline-start-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4769-4771,4773-4776,4778-4780
    Some("border-inline-start-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4810-4812,4814-4817,4819-4821
    Some("border-inline-start-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4845-4847,4849-4852,4854-4856
    Some("border-left-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4880-4882,4884-4887,4889-4891
    Some("border-left-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4926-4928,4930-4933,4935-4937
    Some("border-left-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:4964-4966,4968-4971,4973-4975
    Some("border-right-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5002-5004,5006-5009,5011-5013
    Some("border-right-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5048-5050,5052-5055,5057-5059
    Some("border-right-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5086-5088,5090-5093,5095-5097
    Some("border-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5130-5132,5134-5137,5139-5141
    Some("border-start-end-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5164-5166,5168-5171,5173-5175
    Some("border-start-start-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5205-5207,5209-5212,5214-5216
    Some("border-top-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5246-5248,5250-5253,5255-5257
    Some("border-top-left-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5292-5294,5296-5299,5301-5303
    Some("border-top-right-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5341-5343,5345-5348,5350-5352
    Some("border-top-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5390-5392,5394-5397,5399-5401
    Some("border-top-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5428-5430,5432-5435,5437-5439
    Some("bottom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5466-5468,5470-5473,5475-5477
    Some("box-decoration-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5512-5514,5516-5519,5521-5523
    Some("box-shadow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5541-5543,5545-5548,5550-5552
    Some("box-sizing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5575-5577,5579-5582,5584-5586
    Some("break-after"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5604-5606,5608-5611,5613-5615
    Some("break-before"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5633-5635,5637-5640,5642-5644
    Some("break-inside"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5662-5664,5666-5669,5671-5673
    Some("buffered-rendering"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5691-5693,5695-5698,5700-5702
    Some("caption-side"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5720-5722,5724-5727,5729-5731
    Some("caret-animation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5758-5760,5762-5765,5767-5769
    Some("caret-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5787-5789,5791-5794,5796-5798
    Some("caret-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5822-5824,5826-5829,5831-5833
    Some("clear"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5851-5853,5855-5858,5860-5862
    Some("clip"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5880-5882,5884-5887,5889-5891
    Some("clip-path"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5921-5923,5925-5928,5930-5932
    Some("clip-rule"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5955-5957,5959-5962,5964-5966
    Some("color-interpolation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:5984-5986,5988-5991,5993-5995
    Some("color-interpolation-filters"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6013-6015,6017-6020,6022-6024
    Some("color-rendering"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6042-6044,6046-6049,6051-6053
    Some("column-count"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6071-6073,6075-6078,6080-6082
    Some("column-fill"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6113-6115,6117-6120,6122-6124
    Some("column-gap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6142-6144,6146-6149,6151-6153
    Some("column-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6182-6184,6186-6189,6191-6193
    Some("column-rule-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6229-6231,6233-6236,6238-6240
    Some("column-rule-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6258-6260,6262-6265,6267-6269
    Some("column-rule-inset-cap-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6287-6289,6291-6294,6296-6298
    Some("column-rule-inset-cap-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6321-6323,6325-6328,6330-6332
    Some("column-rule-inset-junction-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6355-6357,6359-6362,6364-6366
    Some("column-rule-inset-junction-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6389-6391,6393-6396,6398-6400
    Some("column-rule-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6423-6425,6427-6430,6432-6434
    Some("column-rule-visibility-items"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6452-6454,6456-6459,6461-6463
    Some("column-rule-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6481-6483,6485-6488,6490-6492
    Some("column-span"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6504-6506,6508-6511,6513-6515
    Some("column-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6533-6535,6537-6540,6542-6544
    Some("column-wrap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6580-6582,6584-6587,6589-6591
    Some("contain"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6609-6611,6613-6616,6618-6620
    Some("contain-intrinsic-block-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6638-6640,6642-6645,6647-6649
    Some("contain-intrinsic-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6679-6681,6683-6686,6688-6690
    Some("contain-intrinsic-inline-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6719-6721,6723-6726,6728-6730
    Some("contain-intrinsic-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6760-6762,6764-6767,6769-6771
    Some("container-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6800-6802,6804-6807,6809-6811
    Some("container-type"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6835-6837,6839-6842,6844-6846
    Some("content"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6864-6866,6868-6871,6873-6875
    Some("content-visibility"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6884-6886,6888-6891,6893-6895
    Some("continue"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6919-6921,6923-6926,6928-6930
    Some("corner-bottom-left-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6948-6950,6952-6955,6957-6959
    Some("corner-bottom-right-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:6992-6994,6996-6999,7001-7003
    Some("corner-end-end-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7036-7038,7040-7043,7045-7047
    Some("corner-end-start-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7077-7079,7081-7084,7086-7088
    Some("corner-start-end-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7118-7120,7122-7125,7127-7129
    Some("corner-start-start-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7159-7161,7163-7166,7168-7170
    Some("corner-top-left-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7200-7202,7204-7207,7209-7211
    Some("corner-top-right-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7244-7246,7248-7251,7253-7255
    Some("counter-increment"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7288-7290,7292-7295,7297-7299
    Some("counter-reset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7363-7365,7367-7370,7372-7374
    Some("counter-set"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7443-7445,7447-7450,7452-7454
    Some("cursor"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7518-7520,7522-7525,7527-7529
    Some("cx"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7538-7540,7542-7545,7547-7549
    Some("cy"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7572-7574,7576-7579,7581-7583
    Some("d"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7606-7608,7610-7613,7615-7617
    Some("descent-override"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7635-7637,7639-7642,7644-7646
    Some("display"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7655-7657,7659-7662,7664-7666
    Some("dominant-baseline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7675-7677,7679-7682,7684-7686
    Some("dynamic-range-limit"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7710-7712,7714-7717,7719-7721
    Some("empty-cells"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7739-7741,7743-7746,7748-7750
    Some("fallback"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7771-7773,7775-7778,7780-7782
    Some("field-sizing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7791-7793,7795-7798,7800-7802
    Some("fill"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7820-7822,7824-7827,7829-7831
    Some("fill-opacity"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7854-7856,7858-7861,7863-7865
    Some("fill-rule"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7889-7891,7893-7896,7898-7900
    Some("filter"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7918-7920,7922-7925,7927-7929
    Some("flex-basis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7955-7957,7959-7962,7964-7966
    Some("flex-direction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:7989-7991,7993-7996,7998-8000
    Some("flex-grow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8018-8020,8022-8025,8027-8029
    Some("flex-line-count"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8053-8055,8057-8060,8062-8064
    Some("flex-shrink"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8082-8084,8086-8089,8091-8093
    Some("flex-wrap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8111-8113,8115-8118,8120-8122
    Some("float"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8140-8142,8144-8147,8149-8151
    Some("flood-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8169-8171,8173-8176,8178-8180
    Some("flood-opacity"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8206-8208,8210-8213,8215-8217
    Some("flow-tolerance"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8247-8249,8251-8254,8256-8258
    Some("font-display"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8287-8289,8291-8294,8296-8298
    Some("frame-sizing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8313-8315,8317-8320,8322-8324
    Some("grid-auto-columns"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8342-8344,8346-8349,8351-8353
    Some("grid-auto-flow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8376-8378,8380-8383,8385-8387
    Some("grid-auto-rows"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8405-8407,8409-8412,8414-8416
    Some("grid-column-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8439-8441,8443-8446,8448-8450
    Some("grid-column-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8474-8476,8478-8481,8483-8485
    Some("grid-lanes-direction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8515-8517,8519-8522,8524-8526
    Some("grid-lanes-pack"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8550-8552,8554-8557,8559-8561
    Some("grid-row-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8579-8581,8583-8586,8588-8590
    Some("grid-row-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8614-8616,8618-8621,8623-8625
    Some("grid-template-areas"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8649-8651,8653-8656,8658-8660
    Some("grid-template-columns"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8678-8680,8682-8685,8687-8689
    Some("grid-template-rows"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8710-8712,8714-8717,8719-8721
    Some("hanging-punctuation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8748-8750,8752-8755,8757-8759
    Some("hash"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8783-8785,8787-8790,8792-8794
    Some("height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8803-8805,8807-8810,8812-8814
    Some("hostname"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8855-8857,8859-8862,8864-8866
    Some("hyphenate-character"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8875-8877,8879-8882,8884-8886
    Some("hyphenate-limit-chars"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8904-8906,8908-8911,8913-8915
    Some("hyphens"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8939-8941,8943-8946,8948-8950
    Some("image-animation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:8974-8976,8978-8981,8983-8985
    Some("image-orientation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9003-9005,9007-9010,9012-9014
    Some("image-rendering"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9032-9034,9036-9039,9041-9043
    Some("inherits"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9061-9063,9065-9068,9070-9072
    Some("initial-letter"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9081-9083,9085-9088,9090-9092
    Some("initial-value"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9116-9118,9120-9123,9125-9127
    Some("inline-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9136-9138,9140-9143,9145-9147
    Some("inset-block-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9177-9179,9181-9184,9186-9188
    Some("inset-block-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9212-9214,9216-9219,9221-9223
    Some("inset-inline-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9247-9249,9251-9254,9256-9258
    Some("inset-inline-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9282-9284,9286-9289,9291-9293
    Some("interactivity"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9317-9319,9321-9324,9326-9328
    Some("interest-delay-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9349-9351,9353-9356,9358-9360
    Some("interest-delay-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9378-9380,9382-9385,9387-9389
    Some("-internal-align-content-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9410-9412,9414-9417,9419-9421
    Some("-internal-empty-line-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9442-9444,9446-9449,9451-9453
    Some("-internal-font-size-delta"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9474-9476,9478-9481,9483-9485
    Some("-internal-forced-background-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9506-9508,9510-9513,9515-9517
    Some("-internal-forced-border-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9546-9548,9550-9553,9555-9557
    Some("-internal-forced-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9586-9588,9590-9593,9595-9597
    Some("-internal-forced-outline-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9615-9617,9619-9622,9624-9626
    Some("-internal-overscroll-container"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9658-9660,9662-9665,9667-9669
    Some("-internal-overscroll-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9693-9695,9697-9700,9702-9704
    Some("-internal-unbounded"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9728-9730,9732-9735,9737-9739
    Some("interpolate-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9757-9759,9761-9764,9766-9768
    Some("isolation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9786-9788,9790-9793,9795-9797
    Some("justify-content"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9815-9817,9819-9822,9824-9826
    Some("justify-items"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9844-9846,9848-9851,9853-9855
    Some("justify-self"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9873-9875,9877-9880,9882-9884
    Some("left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9902-9904,9906-9909,9911-9913
    Some("letter-spacing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9948-9950,9952-9955,9957-9959
    Some("lighting-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:9988-9990,9992-9995,9997-9999
    Some("line-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10025-10027,10029-10032,10034-10036
    Some("line-clamp"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10064-10066,10068-10071,10073-10075
    Some("line-gap-override"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10084-10086,10088-10091,10093-10095
    Some("line-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10104-10106,10108-10111,10113-10115
    Some("list-style-image"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10144-10146,10148-10151,10153-10155
    Some("list-style-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10181-10183,10185-10188,10190-10192
    Some("list-style-type"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10213-10215,10217-10220,10222-10224
    Some("margin-block-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10239-10241,10243-10246,10248-10250
    Some("margin-block-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10274-10276,10278-10281,10283-10285
    Some("margin-bottom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10309-10311,10313-10316,10318-10320
    Some("margin-inline-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10355-10357,10359-10362,10364-10366
    Some("margin-inline-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10390-10392,10394-10397,10399-10401
    Some("margin-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10425-10427,10429-10432,10434-10436
    Some("margin-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10471-10473,10475-10478,10480-10482
    Some("margin-top"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10517-10519,10521-10524,10526-10528
    Some("margin-trim"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10569-10571,10573-10576,10578-10580
    Some("marker-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10598-10600,10602-10605,10607-10609
    Some("marker-mid"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10624-10626,10628-10631,10633-10635
    Some("marker-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10650-10652,10654-10657,10659-10661
    Some("mask-clip"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10676-10678,10680-10683,10685-10687
    Some("mask-image"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10746-10748,10750-10753,10755-10757
    Some("mask-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10816-10818,10820-10823,10825-10827
    Some("mask-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10886-10888,10890-10893,10895-10897
    Some("mask-repeat"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:10956-10958,10960-10963,10965-10967
    Some("mask-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11026-11028,11030-11033,11035-11037
    Some("mask-type"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11107-11109,11111-11114,11116-11118
    Some("math-shift"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11136-11138,11140-11143,11145-11147
    Some("math-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11165-11167,11169-11172,11174-11176
    Some("max-block-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11194-11196,11198-11201,11203-11205
    Some("max-content-sizing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11241-11243,11245-11248,11250-11252
    Some("max-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11261-11263,11265-11268,11270-11272
    Some("max-inline-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11307-11309,11311-11314,11316-11318
    Some("max-lines"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11354-11356,11358-11361,11363-11365
    Some("max-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11383-11385,11387-11390,11392-11394
    Some("min-block-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11429-11431,11433-11436,11438-11440
    Some("min-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11464-11466,11468-11471,11473-11475
    Some("min-inline-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11510-11512,11514-11517,11519-11521
    Some("min-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11545-11547,11549-11552,11554-11556
    Some("mix-blend-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11591-11593,11595-11598,11600-11602
    Some("navigation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11620-11622,11624-11627,11629-11631
    Some("negative"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11640-11642,11644-11647,11649-11651
    Some("object-fit"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11660-11662,11664-11667,11669-11671
    Some("object-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11689-11691,11693-11696,11698-11700
    Some("object-view-box"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11723-11725,11727-11730,11732-11734
    Some("offset-anchor"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11757-11759,11761-11764,11766-11768
    Some("offset-distance"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11791-11793,11795-11798,11800-11802
    Some("offset-path"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11825-11827,11829-11832,11834-11836
    Some("offset-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11865-11867,11869-11872,11874-11876
    Some("offset-rotate"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11899-11901,11903-11906,11908-11910
    Some("opacity"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11934-11936,11938-11941,11943-11945
    Some("order"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:11969-11971,11973-11976,11978-11980
    Some("origin-trial-test-property"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12010-12012,12014-12017,12019-12021
    Some("orphans"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12039-12041,12043-12046,12048-12050
    Some("outline-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12074-12076,12078-12081,12083-12085
    Some("outline-offset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12105-12107,12109-12112,12114-12116
    Some("outline-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12139-12141,12143-12146,12148-12150
    Some("outline-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12159-12161,12163-12166,12168-12170
    Some("overflow-anchor"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12182-12184,12186-12189,12191-12193
    Some("overflow-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12211-12213,12215-12218,12220-12222
    Some("overflow-clip-margin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12246-12248,12250-12253,12255-12257
    Some("overflow-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12280-12282,12284-12287,12289-12291
    Some("overflow-wrap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12315-12317,12319-12322,12324-12326
    Some("overflow-x"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12344-12346,12348-12351,12353-12355
    Some("overflow-y"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12373-12375,12377-12380,12382-12384
    Some("overlay"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12408-12410,12412-12415,12417-12419
    Some("override-colors"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12437-12439,12441-12444,12446-12448
    Some("overscroll-behavior-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12457-12459,12461-12464,12466-12468
    Some("overscroll-behavior-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12492-12494,12496-12499,12501-12503
    Some("overscroll-behavior-x"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12527-12529,12531-12534,12536-12538
    Some("overscroll-behavior-y"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12565-12567,12569-12572,12574-12576
    Some("overscroll-container-type"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12609-12611,12613-12616,12618-12620
    Some("pad"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12638-12640,12642-12645,12647-12649
    Some("padding-block-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12658-12660,12662-12665,12667-12669
    Some("padding-block-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12693-12695,12697-12700,12702-12704
    Some("padding-bottom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12728-12730,12732-12735,12737-12739
    Some("padding-inline-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12771-12773,12775-12778,12780-12782
    Some("padding-inline-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12806-12808,12810-12813,12815-12817
    Some("padding-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12841-12843,12845-12848,12850-12852
    Some("padding-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12884-12886,12888-12891,12893-12895
    Some("padding-top"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12927-12929,12931-12934,12936-12938
    Some("page"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:12970-12972,12974-12977,12979-12981
    Some("page-margin-safety"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13005-13007,13009-13012,13014-13016
    Some("page-orientation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13034-13036,13038-13041,13043-13045
    Some("paint-order"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13063-13065,13067-13070,13072-13074
    Some("path-length"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13098-13100,13102-13105,13107-13109
    Some("pathname"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13138-13140,13142-13145,13147-13149
    Some("pattern"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13164-13166,13168-13171,13173-13175
    Some("perspective"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13184-13186,13188-13191,13193-13195
    Some("perspective-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13218-13220,13222-13225,13227-13229
    Some("pointer-events"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13252-13254,13256-13259,13261-13263
    Some("port"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13290-13292,13294-13297,13299-13301
    Some("position-try-fallbacks"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13310-13312,13314-13317,13319-13321
    Some("position-try-order"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13342-13344,13346-13349,13351-13353
    Some("position-visibility"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13371-13373,13375-13378,13380-13382
    Some("prefix"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13400-13402,13404-13407,13409-13411
    Some("print-color-adjust"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13420-13422,13424-13427,13429-13431
    Some("protocol"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13455-13457,13459-13462,13464-13466
    Some("quotes"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13475-13477,13479-13482,13484-13486
    Some("r"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13504-13506,13508-13511,13513-13515
    Some("range"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13538-13540,13542-13545,13547-13549
    Some("reading-flow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13558-13560,13562-13565,13567-13569
    Some("reading-order"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13587-13589,13591-13594,13596-13598
    Some("resize"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13622-13624,13626-13629,13631-13633
    Some("result"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13654-13656,13658-13661,13663-13665
    Some("right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13674-13676,13678-13681,13683-13685
    Some("rotate"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13720-13722,13724-13727,13729-13731
    Some("row-gap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13749-13751,13753-13756,13758-13760
    Some("row-rule-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13789-13791,13793-13796,13798-13800
    Some("row-rule-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13818-13820,13822-13825,13827-13829
    Some("row-rule-inset-cap-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13847-13849,13851-13854,13856-13858
    Some("row-rule-inset-cap-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13881-13883,13885-13888,13890-13892
    Some("row-rule-inset-junction-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13915-13917,13919-13922,13924-13926
    Some("row-rule-inset-junction-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13949-13951,13953-13956,13958-13960
    Some("row-rule-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:13983-13985,13987-13990,13992-13994
    Some("row-rule-visibility-items"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14012-14014,14016-14019,14021-14023
    Some("row-rule-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14041-14043,14045-14048,14050-14052
    Some("ruby-align"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14064-14066,14068-14071,14073-14075
    Some("ruby-overhang"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14099-14101,14103-14106,14108-14110
    Some("ruby-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14128-14130,14132-14135,14137-14139
    Some("rule-overlap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14157-14159,14161-14164,14166-14168
    Some("rx"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14186-14188,14190-14193,14195-14197
    Some("ry"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14220-14222,14224-14227,14229-14231
    Some("scale"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14254-14256,14258-14261,14263-14265
    Some("scroll-axis-lock"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14289-14291,14293-14296,14298-14300
    Some("scroll-behavior"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14318-14320,14322-14325,14327-14329
    Some("scroll-initial-target"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14353-14355,14357-14360,14362-14364
    Some("scroll-margin-block-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14382-14384,14386-14389,14391-14393
    Some("scroll-margin-block-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14417-14419,14421-14424,14426-14428
    Some("scroll-margin-bottom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14452-14454,14456-14459,14461-14463
    Some("scroll-margin-inline-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14495-14497,14499-14502,14504-14506
    Some("scroll-margin-inline-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14530-14532,14534-14537,14539-14541
    Some("scroll-margin-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14565-14567,14569-14572,14574-14576
    Some("scroll-margin-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14608-14610,14612-14615,14617-14619
    Some("scroll-margin-top"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14651-14653,14655-14658,14660-14662
    Some("scroll-marker-group"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14700-14702,14704-14707,14709-14711
    Some("scroll-padding-block-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14729-14731,14733-14736,14738-14740
    Some("scroll-padding-block-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14770-14772,14774-14777,14779-14781
    Some("scroll-padding-bottom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14811-14813,14815-14818,14820-14822
    Some("scroll-padding-inline-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14860-14862,14864-14867,14869-14871
    Some("scroll-padding-inline-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14901-14903,14905-14908,14910-14912
    Some("scroll-padding-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14942-14944,14946-14949,14951-14953
    Some("scroll-padding-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:14991-14993,14995-14998,15000-15002
    Some("scroll-padding-top"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15040-15042,15044-15047,15049-15051
    Some("scroll-snap-align"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15089-15091,15093-15096,15098-15100
    Some("scroll-snap-stop"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15118-15120,15122-15125,15127-15129
    Some("scroll-snap-type"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15147-15149,15151-15154,15156-15158
    Some("scroll-target-group"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15182-15184,15186-15189,15191-15193
    Some("scroll-timeline-axis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15211-15213,15215-15218,15220-15222
    Some("scroll-timeline-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15240-15242,15244-15247,15249-15251
    Some("scrollbar-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15275-15277,15279-15282,15284-15286
    Some("scrollbar-gutter"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15304-15306,15308-15311,15313-15315
    Some("scrollbar-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15339-15341,15343-15346,15348-15350
    Some("search"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15374-15376,15378-15381,15383-15385
    Some("shape-image-threshold"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15394-15396,15398-15401,15403-15405
    Some("shape-margin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15429-15431,15433-15436,15438-15440
    Some("shape-outside"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15463-15465,15467-15470,15472-15474
    Some("shape-rendering"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15497-15499,15501-15504,15506-15508
    Some("size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15526-15528,15530-15533,15535-15537
    Some("size-adjust"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15546-15548,15550-15553,15555-15557
    Some("speak"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15566-15568,15570-15573,15575-15577
    Some("speak-as"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15595-15597,15599-15602,15604-15606
    Some("src"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15615-15617,15619-15622,15624-15626
    Some("stop-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15635-15637,15639-15642,15644-15646
    Some("stop-opacity"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15672-15674,15676-15679,15681-15683
    Some("stroke"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15707-15709,15711-15714,15716-15718
    Some("stroke-dasharray"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15741-15743,15745-15748,15750-15752
    Some("stroke-dashoffset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15775-15777,15779-15782,15784-15786
    Some("stroke-linecap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15809-15811,15813-15816,15818-15820
    Some("stroke-linejoin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15838-15840,15842-15845,15847-15849
    Some("stroke-miterlimit"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15867-15869,15871-15874,15876-15878
    Some("stroke-opacity"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15896-15898,15900-15903,15905-15907
    Some("stroke-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15931-15933,15935-15938,15940-15942
    Some("suffix"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15965-15967,15969-15972,15974-15976
    Some("symbols"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:15985-15987,15989-15992,15994-15996
    Some("syntax"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16005-16007,16009-16012,16014-16016
    Some("system"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16025-16027,16029-16032,16034-16036
    Some("tab-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16045-16047,16049-16052,16054-16056
    Some("table-layout"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16079-16081,16083-16086,16088-16090
    Some("text-align"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16108-16110,16112-16115,16117-16119
    Some("text-align-last"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16134-16136,16138-16141,16143-16145
    Some("text-anchor"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16163-16165,16167-16170,16172-16174
    Some("text-autospace"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16192-16194,16196-16199,16201-16203
    Some("text-box-edge"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16221-16223,16225-16228,16230-16232
    Some("text-box-trim"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16250-16252,16254-16257,16259-16261
    Some("text-combine-upright"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16279-16281,16283-16286,16288-16290
    Some("text-decoration-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16308-16310,16312-16315,16317-16319
    Some("text-decoration-inset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16351-16353,16355-16358,16360-16362
    Some("text-decoration-line"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16385-16387,16389-16392,16394-16396
    Some("text-decoration-skip-ink"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16414-16416,16418-16421,16423-16425
    Some("text-decoration-skip-spaces"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16449-16451,16453-16456,16458-16460
    Some("text-decoration-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16478-16480,16482-16485,16487-16489
    Some("text-decoration-thickness"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16507-16509,16511-16514,16516-16518
    Some("text-emphasis-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16541-16543,16545-16548,16550-16552
    Some("text-emphasis-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16578-16580,16582-16585,16587-16589
    Some("text-emphasis-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16607-16609,16611-16614,16616-16618
    Some("text-fit"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16633-16635,16637-16640,16642-16644
    Some("text-indent"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16668-16670,16672-16675,16677-16679
    Some("text-justify"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16699-16701,16703-16706,16708-16710
    Some("text-overflow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16728-16730,16732-16735,16737-16739
    Some("text-shadow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16757-16759,16761-16764,16766-16768
    Some("text-transform"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16791-16793,16795-16798,16800-16802
    Some("text-underline-offset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16823-16825,16827-16830,16832-16834
    Some("text-underline-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16857-16859,16861-16864,16866-16868
    Some("text-wrap-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16886-16888,16890-16893,16895-16897
    Some("text-wrap-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16915-16917,16919-16922,16924-16926
    Some("timeline-scope"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16944-16946,16948-16951,16953-16955
    Some("timeline-trigger-activation-range-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:16979-16981,16983-16986,16988-16990
    Some("timeline-trigger-activation-range-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17036-17038,17040-17043,17045-17047
    Some("timeline-trigger-active-range-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17093-17095,17097-17100,17102-17104
    Some("timeline-trigger-active-range-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17150-17152,17154-17157,17159-17161
    Some("timeline-trigger-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17207-17209,17211-17214,17216-17218
    Some("timeline-trigger-source"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17259-17261,17263-17266,17268-17270
    Some("top"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17305-17307,17309-17312,17314-17316
    Some("touch-action"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17351-17353,17355-17358,17360-17362
    Some("transform"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17380-17382,17384-17387,17389-17391
    Some("transform-box"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17420-17422,17424-17427,17429-17431
    Some("transform-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17449-17451,17453-17456,17458-17460
    Some("transform-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17483-17485,17487-17490,17492-17494
    Some("transition-behavior"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17512-17514,17516-17519,17521-17523
    Some("transition-delay"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17558-17560,17562-17565,17567-17569
    Some("transition-duration"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17604-17606,17608-17611,17613-17615
    Some("transition-property"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17650-17652,17654-17657,17659-17661
    Some("transition-timing-function"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17696-17698,17700-17703,17705-17707
    Some("translate"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17742-17744,17746-17749,17751-17753
    Some("trigger-scope"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17782-17784,17786-17789,17791-17793
    Some("types"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17817-17819,17821-17824,17826-17828
    Some("unicode-bidi"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17837-17839,17841-17844,17846-17848
    Some("unicode-range"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17866-17868,17870-17873,17875-17877
    Some("user-select"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17886-17888,17890-17893,17895-17897
    Some("vector-effect"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17915-17917,17919-17922,17924-17926
    Some("vertical-align"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17944-17946,17948-17951,17953-17955
    Some("view-timeline-axis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17967-17969,17971-17974,17976-17978
    Some("view-timeline-inset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:17996-17998,18000-18003,18005-18007
    Some("view-timeline-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18030-18032,18034-18037,18039-18041
    Some("view-transition-class"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18059-18061,18063-18066,18068-18070
    Some("view-transition-group"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18088-18090,18092-18095,18097-18099
    Some("view-transition-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18117-18119,18121-18124,18126-18128
    Some("view-transition-scope"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18146-18148,18150-18153,18155-18157
    Some("visibility"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18175-18177,18179-18182,18184-18186
    Some("-webkit-border-horizontal-spacing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18207-18209,18211-18214,18216-18218
    Some("-webkit-border-image"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18241-18243,18245-18248,18250-18252
    Some("-webkit-border-vertical-spacing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18273-18275,18277-18280,18282-18284
    Some("-webkit-box-align"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18307-18309,18311-18314,18316-18318
    Some("-webkit-box-decoration-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18336-18338,18340-18343,18345-18347
    Some("-webkit-box-direction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18359-18361,18363-18366,18368-18370
    Some("-webkit-box-flex"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18388-18390,18392-18395,18397-18399
    Some("-webkit-box-ordinal-group"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18417-18419,18421-18424,18426-18428
    Some("-webkit-box-orient"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18452-18454,18456-18459,18461-18463
    Some("-webkit-box-pack"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18481-18483,18485-18488,18490-18492
    Some("-webkit-box-reflect"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18510-18512,18514-18517,18519-18521
    Some("-webkit-line-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18544-18546,18548-18551,18553-18555
    Some("-webkit-line-clamp"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18578-18580,18582-18585,18587-18589
    Some("-webkit-mask-box-image-outset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18607-18609,18611-18614,18616-18618
    Some("-webkit-mask-box-image-repeat"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18657-18659,18661-18664,18666-18668
    Some("-webkit-mask-box-image-slice"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18697-18699,18701-18704,18706-18708
    Some("-webkit-mask-box-image-source"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18739-18741,18743-18746,18748-18750
    Some("-webkit-mask-box-image-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18771-18773,18775-18778,18780-18782
    Some("-webkit-mask-position-x"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18823-18825,18827-18830,18832-18834
    Some("-webkit-mask-position-y"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18900-18902,18904-18907,18909-18911
    Some("-webkit-perspective-origin-x"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:18977-18979,18981-18984,18986-18988
    Some("-webkit-perspective-origin-y"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19003-19005,19007-19010,19012-19014
    Some("-webkit-rtl-ordering"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19029-19031,19033-19036,19038-19040
    Some("-webkit-ruby-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19061-19063,19065-19068,19070-19072
    Some("-webkit-tap-highlight-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19084-19086,19088-19091,19093-19095
    Some("-webkit-text-combine"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19119-19121,19123-19126,19128-19130
    Some("-webkit-text-decorations-in-effect"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19142-19144,19146-19149,19151-19153
    Some("-webkit-text-fill-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19171-19173,19175-19178,19180-19182
    Some("-webkit-text-security"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19208-19210,19212-19215,19217-19219
    Some("-webkit-text-stroke-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19237-19239,19241-19244,19246-19248
    Some("-webkit-text-stroke-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19274-19276,19278-19281,19283-19285
    Some("-webkit-transform-origin-x"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19308-19310,19312-19315,19317-19319
    Some("-webkit-transform-origin-y"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19334-19336,19338-19341,19343-19345
    Some("-webkit-transform-origin-z"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19360-19362,19364-19367,19369-19371
    Some("-webkit-user-drag"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19386-19388,19390-19393,19395-19397
    Some("-webkit-user-modify"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19415-19417,19419-19422,19424-19426
    Some("white-space-collapse"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19444-19446,19448-19451,19453-19455
    Some("widows"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19473-19475,19477-19480,19482-19484
    Some("width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19508-19510,19512-19515,19517-19519
    Some("will-change"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19554-19556,19558-19561,19563-19565
    Some("window-drag"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19580-19582,19584-19587,19589-19591
    Some("word-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19606-19608,19610-19613,19615-19617
    Some("word-spacing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19632-19634,19636-19639,19641-19643
    Some("x"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19672-19674,19676-19679,19681-19683
    Some("y"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19706-19708,19710-19713,19715-19717
    Some("z-index"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19740-19742,19744-19747,19749-19751
    Some("mask-composite"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19776-19778,19780-19783,19785-19787
    Some("line-clamp"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:48-50,52-55,57-59
    Some("-webkit-line-clamp"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:74-76,78-81,83-85
    Some("animation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:94-96,98-101,103-105
    Some("animation-range"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:114-116,118-121,123-125
    Some("background"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:134-136,138-141,143-145
    Some("background-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:154-156,158-161,163-165
    Some("border"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:174-176,178-181,183-185
    Some("border-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:194-196,198-201,203-205
    Some("border-block-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:214-216,218-221,223-225
    Some("border-block-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:234-236,238-241,243-245
    Some("border-block-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:269-271,273-276,278-280
    Some("border-block-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:304-306,308-311,313-315
    Some("border-block-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:324-326,328-331,333-335
    Some("border-bottom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:344-346,348-351,353-355
    Some("border-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:373-375,377-380,382-384
    Some("border-image"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:393-395,397-400,402-404
    Some("border-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:413-415,417-420,422-424
    Some("border-inline-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:433-435,437-440,442-444
    Some("border-inline-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:453-455,457-460,462-464
    Some("border-inline-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:488-490,492-495,497-499
    Some("border-inline-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:523-525,527-530,532-534
    Some("border-inline-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:543-545,547-550,552-554
    Some("border-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:563-565,567-570,572-574
    Some("border-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:592-594,596-599,601-603
    Some("border-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:612-614,616-619,621-623
    Some("border-spacing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:641-643,645-648,650-652
    Some("border-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:661-663,665-668,670-672
    Some("border-top"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:681-683,685-688,690-692
    Some("border-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:710-712,714-717,719-721
    Some("column-rule"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:730-732,734-737,739-741
    Some("column-rule-inset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:750-752,754-757,759-761
    Some("column-rule-inset-cap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:770-772,774-777,779-781
    Some("column-rule-inset-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:790-792,794-797,799-801
    Some("column-rule-inset-junction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:810-812,814-817,819-821
    Some("column-rule-inset-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:830-832,834-837,839-841
    Some("columns"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:850-852,854-857,859-861
    Some("contain-intrinsic-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:870-872,874-877,879-881
    Some("container"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:890-892,894-897,899-901
    Some("corner"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:916-918,920-923,925-927
    Some("corner-block-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:942-944,946-949,951-953
    Some("corner-block-end-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:962-964,966-969,971-973
    Some("corner-block-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:988-990,992-995,997-999
    Some("corner-block-start-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1008-1010,1012-1015,1017-1019
    Some("corner-bottom"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1034-1036,1038-1041,1043-1045
    Some("corner-bottom-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1060-1062,1064-1067,1069-1071
    Some("corner-bottom-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1086-1088,1090-1093,1095-1097
    Some("corner-bottom-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1106-1108,1110-1113,1115-1117
    Some("corner-end-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1132-1134,1136-1139,1141-1143
    Some("corner-end-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1158-1160,1162-1165,1167-1169
    Some("corner-inline-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1184-1186,1188-1191,1193-1195
    Some("corner-inline-end-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1204-1206,1208-1211,1213-1215
    Some("corner-inline-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1230-1232,1234-1237,1239-1241
    Some("corner-inline-start-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1250-1252,1254-1257,1259-1261
    Some("corner-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1276-1278,1280-1283,1285-1287
    Some("corner-left-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1296-1298,1300-1303,1305-1307
    Some("corner-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1322-1324,1326-1329,1331-1333
    Some("corner-right-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1342-1344,1346-1349,1351-1353
    Some("corner-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1362-1364,1366-1369,1371-1373
    Some("corner-start-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1388-1390,1392-1395,1397-1399
    Some("corner-start-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1414-1416,1418-1421,1423-1425
    Some("corner-top"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1440-1442,1444-1447,1449-1451
    Some("corner-top-left"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1466-1468,1470-1473,1475-1477
    Some("corner-top-right"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1492-1494,1496-1499,1501-1503
    Some("corner-top-shape"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1512-1514,1516-1519,1521-1523
    Some("flex"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1532-1534,1536-1539,1541-1543
    Some("flex-flow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1552-1554,1556-1559,1561-1563
    Some("font"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1572-1574,1576-1579,1581-1583
    Some("font-synthesis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1592-1594,1596-1599,1601-1603
    Some("font-variant"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1612-1614,1616-1619,1621-1623
    Some("gap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1632-1634,1636-1639,1641-1643
    Some("grid"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1652-1654,1656-1659,1661-1663
    Some("grid-area"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1672-1674,1676-1679,1681-1683
    Some("grid-column"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1692-1694,1696-1699,1701-1703
    Some("grid-lanes"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1718-1720,1722-1725,1727-1729
    Some("grid-row"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1738-1740,1742-1745,1747-1749
    Some("grid-template"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1758-1760,1762-1765,1767-1769
    Some("inset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1778-1780,1782-1785,1787-1789
    Some("inset-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1798-1800,1802-1805,1807-1809
    Some("inset-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1818-1820,1822-1825,1827-1829
    Some("interest-delay"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1838-1840,1842-1845,1847-1849
    Some("list-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1858-1860,1862-1865,1867-1869
    Some("margin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1878-1880,1882-1885,1887-1889
    Some("margin-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1898-1900,1902-1905,1907-1909
    Some("margin-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1918-1920,1922-1925,1927-1929
    Some("marker"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1938-1940,1942-1945,1947-1949
    Some("mask"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1958-1960,1962-1965,1967-1969
    Some("mask-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1978-1980,1982-1985,1987-1989
    Some("offset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:1998-2000,2002-2005,2007-2009
    Some("outline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2018-2020,2022-2025,2027-2029
    Some("overflow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2038-2040,2042-2045,2047-2049
    Some("overscroll-behavior"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2058-2060,2062-2065,2067-2069
    Some("padding"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2078-2080,2082-2085,2087-2089
    Some("padding-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2098-2100,2102-2105,2107-2109
    Some("padding-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2118-2120,2122-2125,2127-2129
    Some("page-break-after"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2138-2140,2142-2145,2147-2149
    Some("page-break-before"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2158-2160,2162-2165,2167-2169
    Some("page-break-inside"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2178-2180,2182-2185,2187-2189
    Some("place-content"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2198-2200,2202-2205,2207-2209
    Some("place-items"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2218-2220,2222-2225,2227-2229
    Some("place-self"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2238-2240,2242-2245,2247-2249
    Some("position-try"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2258-2260,2262-2265,2267-2269
    Some("row-rule"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2278-2280,2282-2285,2287-2289
    Some("row-rule-inset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2298-2300,2302-2305,2307-2309
    Some("row-rule-inset-cap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2318-2320,2322-2325,2327-2329
    Some("row-rule-inset-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2338-2340,2342-2345,2347-2349
    Some("row-rule-inset-junction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2358-2360,2362-2365,2367-2369
    Some("row-rule-inset-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2378-2380,2382-2385,2387-2389
    Some("rule"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2398-2400,2402-2405,2407-2409
    Some("rule-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2418-2420,2422-2425,2427-2429
    Some("rule-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2438-2440,2442-2445,2447-2449
    Some("rule-inset"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2458-2460,2462-2465,2467-2469
    Some("rule-inset-cap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2478-2480,2482-2485,2487-2489
    Some("rule-inset-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2498-2500,2502-2505,2507-2509
    Some("rule-inset-junction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2518-2520,2522-2525,2527-2529
    Some("rule-inset-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2538-2540,2542-2545,2547-2549
    Some("rule-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2558-2560,2562-2565,2567-2569
    Some("rule-visibility-items"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2578-2580,2582-2585,2587-2589
    Some("rule-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2598-2600,2602-2605,2607-2609
    Some("scroll-margin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2618-2620,2622-2625,2627-2629
    Some("scroll-margin-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2638-2640,2642-2645,2647-2649
    Some("scroll-margin-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2658-2660,2662-2665,2667-2669
    Some("scroll-padding"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2678-2680,2682-2685,2687-2689
    Some("scroll-padding-block"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2698-2700,2702-2705,2707-2709
    Some("scroll-padding-inline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2718-2720,2722-2725,2727-2729
    Some("scroll-timeline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2738-2740,2742-2745,2747-2749
    Some("text-box"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2758-2760,2762-2765,2767-2769
    Some("text-decoration"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2778-2780,2782-2785,2787-2789
    Some("text-emphasis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2798-2800,2802-2805,2807-2809
    Some("text-spacing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2824-2826,2828-2831,2833-2835
    Some("text-wrap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2844-2846,2848-2851,2853-2855
    Some("timeline-trigger"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2870-2872,2874-2877,2879-2881
    Some("timeline-trigger-activation-range"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2896-2898,2900-2903,2905-2907
    Some("timeline-trigger-active-range"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2922-2924,2926-2929,2931-2933
    Some("transition"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2942-2944,2946-2949,2951-2953
    Some("view-timeline"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2962-2964,2966-2969,2971-2973
    Some("-webkit-column-break-after"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:2982-2984,2986-2989,2991-2993
    Some("-webkit-column-break-before"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3002-3004,3006-3009,3011-3013
    Some("-webkit-column-break-inside"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3022-3024,3026-3029,3031-3033
    Some("-webkit-mask-box-image"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3042-3044,3046-3049,3051-3053
    Some("-webkit-text-stroke"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3062-3064,3066-3069,3071-3073
    Some("white-space"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3082-3084,3086-3089,3091-3093
    Some("-webkit-appearance"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19846-19848,19850-19853,19855-19857
    Some("-webkit-app-region"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19863-19865,19867-19870,19872-19874
    Some("-webkit-mask-clip"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19880-19882,19884-19887,19889-19891
    Some("-webkit-mask-composite"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19897-19899,19901-19904,19906-19908
    Some("-webkit-mask-image"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19914-19916,19918-19921,19923-19925
    Some("-webkit-mask-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19931-19933,19935-19938,19940-19942
    Some("-webkit-mask-repeat"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19948-19950,19952-19955,19957-19959
    Some("-webkit-mask-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19965-19967,19969-19972,19974-19976
    Some("-webkit-border-end-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19982-19984,19986-19989,19991-19993
    Some("-webkit-border-end-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:19999-20001,20003-20006,20008-20010
    Some("-webkit-border-end-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20016-20018,20020-20023,20025-20027
    Some("-webkit-border-start-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20033-20035,20037-20040,20042-20044
    Some("-webkit-border-start-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20050-20052,20054-20057,20059-20061
    Some("-webkit-border-start-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20067-20069,20071-20074,20076-20078
    Some("-webkit-border-before-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20084-20086,20088-20091,20093-20095
    Some("-webkit-border-before-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20101-20103,20105-20108,20110-20112
    Some("-webkit-border-before-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20118-20120,20122-20125,20127-20129
    Some("-webkit-border-after-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20135-20137,20139-20142,20144-20146
    Some("-webkit-border-after-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20152-20154,20156-20159,20161-20163
    Some("-webkit-border-after-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20169-20171,20173-20176,20178-20180
    Some("-webkit-margin-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20186-20188,20190-20193,20195-20197
    Some("-webkit-margin-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20203-20205,20207-20210,20212-20214
    Some("-webkit-margin-before"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20220-20222,20224-20227,20229-20231
    Some("-webkit-margin-after"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20237-20239,20241-20244,20246-20248
    Some("-webkit-padding-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20254-20256,20258-20261,20263-20265
    Some("-webkit-padding-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20271-20273,20275-20278,20280-20282
    Some("-webkit-padding-before"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20288-20290,20292-20295,20297-20299
    Some("-webkit-padding-after"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20305-20307,20309-20312,20314-20316
    Some("-webkit-logical-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20322-20324,20326-20329,20331-20333
    Some("-webkit-logical-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20339-20341,20343-20346,20348-20350
    Some("-webkit-min-logical-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20356-20358,20360-20363,20365-20367
    Some("-webkit-min-logical-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20373-20375,20377-20380,20382-20384
    Some("-webkit-max-logical-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20390-20392,20394-20397,20399-20401
    Some("-webkit-max-logical-height"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20407-20409,20411-20414,20416-20418
    Some("-webkit-print-color-adjust"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20424-20426,20428-20431,20433-20435
    Some("-webkit-border-after"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3102-3104,3106-3109,3111-3113
    Some("-webkit-border-before"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3119-3121,3123-3126,3128-3130
    Some("-webkit-border-end"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3136-3138,3140-3143,3145-3147
    Some("-webkit-border-start"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3153-3155,3157-3160,3162-3164
    Some("-webkit-mask"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3170-3172,3174-3177,3179-3181
    Some("-webkit-mask-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3187-3189,3191-3194,3196-3198
    Some("-epub-caption-side"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20441-20443,20445-20448,20450-20452
    Some("-epub-text-combine"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20458-20460,20462-20465,20467-20469
    Some("-epub-text-emphasis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3204-3206,3208-3211,3213-3215
    Some("-epub-text-emphasis-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20475-20477,20479-20482,20484-20486
    Some("-epub-text-emphasis-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20492-20494,20496-20499,20501-20503
    Some("-epub-text-orientation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20509-20511,20513-20516,20518-20520
    Some("-epub-text-transform"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20526-20528,20530-20533,20535-20537
    Some("-epub-word-break"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20543-20545,20547-20550,20552-20554
    Some("-epub-writing-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20560-20562,20564-20567,20569-20571
    Some("-webkit-align-content"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20577-20579,20581-20584,20586-20588
    Some("-webkit-align-items"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20594-20596,20598-20601,20603-20605
    Some("-webkit-align-self"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20611-20613,20615-20618,20620-20622
    Some("-webkit-animation"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3221-3223,3225-3228,3230-3232
    Some("-webkit-animation-delay"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20628-20630,20632-20635,20637-20639
    Some("-webkit-animation-direction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20645-20647,20649-20652,20654-20656
    Some("-webkit-animation-duration"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20662-20664,20666-20669,20671-20673
    Some("-webkit-animation-fill-mode"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20679-20681,20683-20686,20688-20690
    Some("-webkit-animation-iteration-count"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20696-20698,20700-20703,20705-20707
    Some("-webkit-animation-name"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20713-20715,20717-20720,20722-20724
    Some("-webkit-animation-play-state"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20730-20732,20734-20737,20739-20741
    Some("-webkit-animation-timing-function"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20747-20749,20751-20754,20756-20758
    Some("-webkit-backface-visibility"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20764-20766,20768-20771,20773-20775
    Some("-webkit-background-clip"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20781-20783,20785-20788,20790-20792
    Some("-webkit-background-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20798-20800,20802-20805,20807-20809
    Some("-webkit-background-size"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20815-20817,20819-20822,20824-20826
    Some("-webkit-border-bottom-left-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20832-20834,20836-20839,20841-20843
    Some("-webkit-border-bottom-right-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20849-20851,20853-20856,20858-20860
    Some("-webkit-border-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3238-3240,3242-3245,3247-3249
    Some("-webkit-border-top-left-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20866-20868,20870-20873,20875-20877
    Some("-webkit-border-top-right-radius"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20883-20885,20887-20890,20892-20894
    Some("-webkit-box-shadow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20900-20902,20904-20907,20909-20911
    Some("-webkit-box-sizing"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20917-20919,20921-20924,20926-20928
    Some("-webkit-clip-path"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20934-20936,20938-20941,20943-20945
    Some("-webkit-column-count"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20951-20953,20955-20958,20960-20962
    Some("-webkit-column-gap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20968-20970,20972-20975,20977-20979
    Some("-webkit-column-rule"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3255-3257,3259-3262,3264-3266
    Some("-webkit-column-rule-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:20985-20987,20989-20992,20994-20996
    Some("-webkit-column-rule-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21002-21004,21006-21009,21011-21013
    Some("-webkit-column-rule-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21019-21021,21023-21026,21028-21030
    Some("-webkit-column-span"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21036-21038,21040-21043,21045-21047
    Some("-webkit-column-width"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21053-21055,21057-21060,21062-21064
    Some("-webkit-columns"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3272-3274,3276-3279,3281-3283
    Some("-webkit-filter"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21070-21072,21074-21077,21079-21081
    Some("-webkit-flex"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3289-3291,3293-3296,3298-3300
    Some("-webkit-flex-basis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21087-21089,21091-21094,21096-21098
    Some("-webkit-flex-direction"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21104-21106,21108-21111,21113-21115
    Some("-webkit-flex-flow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3306-3308,3310-3313,3315-3317
    Some("-webkit-flex-grow"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21121-21123,21125-21128,21130-21132
    Some("-webkit-flex-shrink"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21138-21140,21142-21145,21147-21149
    Some("-webkit-flex-wrap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21155-21157,21159-21162,21164-21166
    Some("-webkit-font-feature-settings"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21172-21174,21176-21179,21181-21183
    Some("-webkit-hyphenate-character"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21189-21191,21193-21196,21198-21200
    Some("-webkit-justify-content"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21206-21208,21210-21213,21215-21217
    Some("-webkit-opacity"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21223-21225,21227-21230,21232-21234
    Some("-webkit-order"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21240-21242,21244-21247,21249-21251
    Some("-webkit-perspective"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21257-21259,21261-21264,21266-21268
    Some("-webkit-perspective-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21274-21276,21278-21281,21283-21285
    Some("-webkit-shape-image-threshold"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21291-21293,21295-21298,21300-21302
    Some("-webkit-shape-margin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21308-21310,21312-21315,21317-21319
    Some("-webkit-shape-outside"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21325-21327,21329-21332,21334-21336
    Some("-webkit-text-emphasis"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3323-3325,3327-3330,3332-3334
    Some("-webkit-text-emphasis-color"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21342-21344,21346-21349,21351-21353
    Some("-webkit-text-emphasis-position"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21359-21361,21363-21366,21368-21370
    Some("-webkit-text-emphasis-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21376-21378,21380-21383,21385-21387
    Some("-webkit-text-size-adjust"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21393-21395,21397-21400,21402-21404
    Some("-webkit-transform"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21410-21412,21414-21417,21419-21421
    Some("-webkit-transform-origin"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21427-21429,21431-21434,21436-21438
    Some("-webkit-transform-style"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21444-21446,21448-21451,21453-21455
    Some("-webkit-transition"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3340-3342,3344-3347,3349-3351
    Some("-webkit-transition-delay"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21461-21463,21465-21468,21470-21472
    Some("-webkit-transition-duration"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21478-21480,21482-21485,21487-21489
    Some("-webkit-transition-property"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21495-21497,21499-21502,21504-21506
    Some("-webkit-transition-timing-function"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21512-21514,21516-21519,21521-21523
    Some("-webkit-user-select"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21529-21531,21533-21536,21538-21540
    Some("word-wrap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21546-21548,21550-21553,21555-21557
    Some("grid-column-gap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21563-21565,21567-21570,21572-21574
    Some("grid-row-gap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/longhands.cc:21580-21582,21584-21587,21589-21591
    Some("grid-gap"), // cpp: out/Min/gen/third_party/blink/renderer/core/css/properties/shorthands.cc:3357-3359,3361-3364,3366-3368
];

pub const JS_PROPERTY_NAMES: [Option<&str>; 826] = [
    None, // Invalid has no object; Variable inherits NOTREACHED().
    None, // Invalid has no object; Variable inherits NOTREACHED().
    Some("colorScheme"),
    Some("forcedColorAdjust"),
    Some("mathDepth"),
    Some("position"),
    Some("positionAnchor"),
    Some("textSizeAdjust"),
    Some("internalVisitedColor"),
    Some("appearance"),
    Some("color"),
    Some("direction"),
    Some("fontFamily"),
    Some("fontFeatureSettings"),
    Some("fontKerning"),
    Some("fontLanguageOverride"),
    Some("fontOpticalSizing"),
    Some("fontPalette"),
    Some("fontSize"),
    Some("fontSizeAdjust"),
    Some("fontStretch"),
    Some("fontStyle"),
    Some("fontSynthesisSmallCaps"),
    Some("fontSynthesisStyle"),
    Some("fontSynthesisWeight"),
    Some("fontVariantAlternates"),
    Some("fontVariantCaps"),
    Some("fontVariantEastAsian"),
    Some("fontVariantEmoji"),
    Some("fontVariantLigatures"),
    Some("fontVariantNumeric"),
    Some("fontVariantPosition"),
    Some("fontVariationSettings"),
    Some("fontWeight"),
    Some("positionArea"),
    Some("textOrientation"),
    Some("textRendering"),
    Some("textSpacingTrim"),
    Some("webkitFontSmoothing"),
    Some("webkitLocale"),
    Some("webkitTextOrientation"),
    Some("webkitWritingMode"),
    Some("writingMode"),
    Some("zoom"),
    Some("internalForcedVisitedColor"),
    Some("internalVisitedBackgroundColor"),
    Some("internalVisitedBorderBlockEndColor"),
    Some("internalVisitedBorderBlockStartColor"),
    Some("internalVisitedBorderBottomColor"),
    Some("internalVisitedBorderInlineEndColor"),
    Some("internalVisitedBorderInlineStartColor"),
    Some("internalVisitedBorderLeftColor"),
    Some("internalVisitedBorderRightColor"),
    Some("internalVisitedBorderTopColor"),
    Some("internalVisitedCaretColor"),
    Some("internalVisitedColumnRuleColor"),
    Some("internalVisitedFill"),
    Some("internalVisitedOutlineColor"),
    Some("internalVisitedStroke"),
    Some("internalVisitedTextDecorationColor"),
    Some("internalVisitedTextEmphasisColor"),
    Some("internalVisitedTextFillColor"),
    Some("internalVisitedTextStrokeColor"),
    Some("accentColor"),
    Some("additiveSymbols"),
    Some("alignContent"),
    Some("alignItems"),
    Some("alignSelf"),
    Some("alignmentBaseline"),
    Some("all"),
    Some("webkitLineClamp"),
    Some("anchorName"),
    Some("anchorScope"),
    Some("animationComposition"),
    Some("animationDelay"),
    Some("animationDirection"),
    Some("animationDuration"),
    Some("animationFillMode"),
    Some("animationIterationCount"),
    Some("animationName"),
    Some("animationPlayState"),
    Some("animationRangeEnd"),
    Some("animationRangeStart"),
    Some("animationTimeline"),
    Some("animationTimingFunction"),
    Some("animationTrigger"),
    Some("appRegion"),
    Some("ascentOverride"),
    Some("aspectRatio"),
    Some("backdropFilter"),
    Some("backfaceVisibility"),
    Some("backgroundAttachment"),
    Some("backgroundBlendMode"),
    Some("backgroundClip"),
    Some("backgroundColor"),
    Some("backgroundImage"),
    Some("backgroundOrigin"),
    Some("backgroundPositionX"),
    Some("backgroundPositionY"),
    Some("backgroundRepeat"),
    Some("backgroundSize"),
    Some("basePalette"),
    Some("baseUrl"),
    Some("baselineShift"),
    Some("baselineSource"),
    Some("blockEllipsis"),
    Some("blockSize"),
    Some("borderBlockEndColor"),
    Some("borderBlockEndStyle"),
    Some("borderBlockEndWidth"),
    Some("borderBlockStartColor"),
    Some("borderBlockStartStyle"),
    Some("borderBlockStartWidth"),
    Some("borderBottomColor"),
    Some("borderBottomLeftRadius"),
    Some("borderBottomRightRadius"),
    Some("borderBottomStyle"),
    Some("borderBottomWidth"),
    Some("borderCollapse"),
    Some("borderEndEndRadius"),
    Some("borderEndStartRadius"),
    Some("borderImageOutset"),
    Some("borderImageRepeat"),
    Some("borderImageSlice"),
    Some("borderImageSource"),
    Some("borderImageWidth"),
    Some("borderInlineEndColor"),
    Some("borderInlineEndStyle"),
    Some("borderInlineEndWidth"),
    Some("borderInlineStartColor"),
    Some("borderInlineStartStyle"),
    Some("borderInlineStartWidth"),
    Some("borderLeftColor"),
    Some("borderLeftStyle"),
    Some("borderLeftWidth"),
    Some("borderRightColor"),
    Some("borderRightStyle"),
    Some("borderRightWidth"),
    Some("borderShape"),
    Some("borderStartEndRadius"),
    Some("borderStartStartRadius"),
    Some("borderTopColor"),
    Some("borderTopLeftRadius"),
    Some("borderTopRightRadius"),
    Some("borderTopStyle"),
    Some("borderTopWidth"),
    Some("bottom"),
    Some("boxDecorationBreak"),
    Some("boxShadow"),
    Some("boxSizing"),
    Some("breakAfter"),
    Some("breakBefore"),
    Some("breakInside"),
    Some("bufferedRendering"),
    Some("captionSide"),
    Some("caretAnimation"),
    Some("caretColor"),
    Some("caretShape"),
    Some("clear"),
    Some("clip"),
    Some("clipPath"),
    Some("clipRule"),
    Some("colorInterpolation"),
    Some("colorInterpolationFilters"),
    Some("colorRendering"),
    Some("columnCount"),
    Some("columnFill"),
    Some("columnGap"),
    Some("columnHeight"),
    Some("columnRuleBreak"),
    Some("columnRuleColor"),
    Some("columnRuleInsetCapEnd"),
    Some("columnRuleInsetCapStart"),
    Some("columnRuleInsetJunctionEnd"),
    Some("columnRuleInsetJunctionStart"),
    Some("columnRuleStyle"),
    Some("columnRuleVisibilityItems"),
    Some("columnRuleWidth"),
    Some("columnSpan"),
    Some("columnWidth"),
    Some("columnWrap"),
    Some("contain"),
    Some("containIntrinsicBlockSize"),
    Some("containIntrinsicHeight"),
    Some("containIntrinsicInlineSize"),
    Some("containIntrinsicWidth"),
    Some("containerName"),
    Some("containerType"),
    Some("content"),
    Some("contentVisibility"),
    Some("continue"),
    Some("cornerBottomLeftShape"),
    Some("cornerBottomRightShape"),
    Some("cornerEndEndShape"),
    Some("cornerEndStartShape"),
    Some("cornerStartEndShape"),
    Some("cornerStartStartShape"),
    Some("cornerTopLeftShape"),
    Some("cornerTopRightShape"),
    Some("counterIncrement"),
    Some("counterReset"),
    Some("counterSet"),
    Some("cursor"),
    Some("cx"),
    Some("cy"),
    Some("d"),
    Some("descentOverride"),
    Some("display"),
    Some("dominantBaseline"),
    Some("dynamicRangeLimit"),
    Some("emptyCells"),
    Some("fallback"),
    Some("fieldSizing"),
    Some("fill"),
    Some("fillOpacity"),
    Some("fillRule"),
    Some("filter"),
    Some("flexBasis"),
    Some("flexDirection"),
    Some("flexGrow"),
    Some("flexLineCount"),
    Some("flexShrink"),
    Some("flexWrap"),
    Some("float"),
    Some("floodColor"),
    Some("floodOpacity"),
    Some("flowTolerance"),
    Some("fontDisplay"),
    Some("frameSizing"),
    Some("gridAutoColumns"),
    Some("gridAutoFlow"),
    Some("gridAutoRows"),
    Some("gridColumnEnd"),
    Some("gridColumnStart"),
    Some("gridLanesDirection"),
    Some("gridLanesPack"),
    Some("gridRowEnd"),
    Some("gridRowStart"),
    Some("gridTemplateAreas"),
    Some("gridTemplateColumns"),
    Some("gridTemplateRows"),
    Some("hangingPunctuation"),
    Some("hash"),
    Some("height"),
    Some("hostname"),
    Some("hyphenateCharacter"),
    Some("hyphenateLimitChars"),
    Some("hyphens"),
    Some("imageAnimation"),
    Some("imageOrientation"),
    Some("imageRendering"),
    Some("inherits"),
    Some("initialLetter"),
    Some("initialValue"),
    Some("inlineSize"),
    Some("insetBlockEnd"),
    Some("insetBlockStart"),
    Some("insetInlineEnd"),
    Some("insetInlineStart"),
    Some("interactivity"),
    Some("interestDelayEnd"),
    Some("interestDelayStart"),
    Some("internalAlignContentBlock"),
    Some("internalEmptyLineHeight"),
    Some("internalFontSizeDelta"),
    Some("internalForcedBackgroundColor"),
    Some("internalForcedBorderColor"),
    Some("internalForcedColor"),
    Some("internalForcedOutlineColor"),
    Some("internalOverscrollContainer"),
    Some("internalOverscrollPosition"),
    Some("internalUnbounded"),
    Some("interpolateSize"),
    Some("isolation"),
    Some("justifyContent"),
    Some("justifyItems"),
    Some("justifySelf"),
    Some("left"),
    Some("letterSpacing"),
    Some("lightingColor"),
    Some("lineBreak"),
    Some("lineClamp"),
    Some("lineGapOverride"),
    Some("lineHeight"),
    Some("listStyleImage"),
    Some("listStylePosition"),
    Some("listStyleType"),
    Some("marginBlockEnd"),
    Some("marginBlockStart"),
    Some("marginBottom"),
    Some("marginInlineEnd"),
    Some("marginInlineStart"),
    Some("marginLeft"),
    Some("marginRight"),
    Some("marginTop"),
    Some("marginTrim"),
    Some("markerEnd"),
    Some("markerMid"),
    Some("markerStart"),
    Some("maskClip"),
    Some("maskImage"),
    Some("maskMode"),
    Some("maskOrigin"),
    Some("maskRepeat"),
    Some("maskSize"),
    Some("maskType"),
    Some("mathShift"),
    Some("mathStyle"),
    Some("maxBlockSize"),
    Some("maxContentSizing"),
    Some("maxHeight"),
    Some("maxInlineSize"),
    Some("maxLines"),
    Some("maxWidth"),
    Some("minBlockSize"),
    Some("minHeight"),
    Some("minInlineSize"),
    Some("minWidth"),
    Some("mixBlendMode"),
    Some("navigation"),
    Some("negative"),
    Some("objectFit"),
    Some("objectPosition"),
    Some("objectViewBox"),
    Some("offsetAnchor"),
    Some("offsetDistance"),
    Some("offsetPath"),
    Some("offsetPosition"),
    Some("offsetRotate"),
    Some("opacity"),
    Some("order"),
    Some("originTrialTestProperty"),
    Some("orphans"),
    Some("outlineColor"),
    Some("outlineOffset"),
    Some("outlineStyle"),
    Some("outlineWidth"),
    Some("overflowAnchor"),
    Some("overflowBlock"),
    Some("overflowClipMargin"),
    Some("overflowInline"),
    Some("overflowWrap"),
    Some("overflowX"),
    Some("overflowY"),
    Some("overlay"),
    Some("overrideColors"),
    Some("overscrollBehaviorBlock"),
    Some("overscrollBehaviorInline"),
    Some("overscrollBehaviorX"),
    Some("overscrollBehaviorY"),
    Some("overscrollContainerType"),
    Some("pad"),
    Some("paddingBlockEnd"),
    Some("paddingBlockStart"),
    Some("paddingBottom"),
    Some("paddingInlineEnd"),
    Some("paddingInlineStart"),
    Some("paddingLeft"),
    Some("paddingRight"),
    Some("paddingTop"),
    Some("page"),
    Some("pageMarginSafety"),
    Some("pageOrientation"),
    Some("paintOrder"),
    Some("pathLength"),
    Some("pathname"),
    Some("pattern"),
    Some("perspective"),
    Some("perspectiveOrigin"),
    Some("pointerEvents"),
    Some("port"),
    Some("positionTryFallbacks"),
    Some("positionTryOrder"),
    Some("positionVisibility"),
    Some("prefix"),
    Some("printColorAdjust"),
    Some("protocol"),
    Some("quotes"),
    Some("r"),
    Some("range"),
    Some("readingFlow"),
    Some("readingOrder"),
    Some("resize"),
    Some("result"),
    Some("right"),
    Some("rotate"),
    Some("rowGap"),
    Some("rowRuleBreak"),
    Some("rowRuleColor"),
    Some("rowRuleInsetCapEnd"),
    Some("rowRuleInsetCapStart"),
    Some("rowRuleInsetJunctionEnd"),
    Some("rowRuleInsetJunctionStart"),
    Some("rowRuleStyle"),
    Some("rowRuleVisibilityItems"),
    Some("rowRuleWidth"),
    Some("rubyAlign"),
    Some("rubyOverhang"),
    Some("rubyPosition"),
    Some("ruleOverlap"),
    Some("rx"),
    Some("ry"),
    Some("scale"),
    Some("scrollAxisLock"),
    Some("scrollBehavior"),
    Some("scrollInitialTarget"),
    Some("scrollMarginBlockEnd"),
    Some("scrollMarginBlockStart"),
    Some("scrollMarginBottom"),
    Some("scrollMarginInlineEnd"),
    Some("scrollMarginInlineStart"),
    Some("scrollMarginLeft"),
    Some("scrollMarginRight"),
    Some("scrollMarginTop"),
    Some("scrollMarkerGroup"),
    Some("scrollPaddingBlockEnd"),
    Some("scrollPaddingBlockStart"),
    Some("scrollPaddingBottom"),
    Some("scrollPaddingInlineEnd"),
    Some("scrollPaddingInlineStart"),
    Some("scrollPaddingLeft"),
    Some("scrollPaddingRight"),
    Some("scrollPaddingTop"),
    Some("scrollSnapAlign"),
    Some("scrollSnapStop"),
    Some("scrollSnapType"),
    Some("scrollTargetGroup"),
    Some("scrollTimelineAxis"),
    Some("scrollTimelineName"),
    Some("scrollbarColor"),
    Some("scrollbarGutter"),
    Some("scrollbarWidth"),
    Some("search"),
    Some("shapeImageThreshold"),
    Some("shapeMargin"),
    Some("shapeOutside"),
    Some("shapeRendering"),
    Some("size"),
    Some("sizeAdjust"),
    Some("speak"),
    Some("speakAs"),
    Some("src"),
    Some("stopColor"),
    Some("stopOpacity"),
    Some("stroke"),
    Some("strokeDasharray"),
    Some("strokeDashoffset"),
    Some("strokeLinecap"),
    Some("strokeLinejoin"),
    Some("strokeMiterlimit"),
    Some("strokeOpacity"),
    Some("strokeWidth"),
    Some("suffix"),
    Some("symbols"),
    Some("syntax"),
    Some("system"),
    Some("tabSize"),
    Some("tableLayout"),
    Some("textAlign"),
    Some("textAlignLast"),
    Some("textAnchor"),
    Some("textAutospace"),
    Some("textBoxEdge"),
    Some("textBoxTrim"),
    Some("textCombineUpright"),
    Some("textDecorationColor"),
    Some("textDecorationInset"),
    Some("textDecorationLine"),
    Some("textDecorationSkipInk"),
    Some("textDecorationSkipSpaces"),
    Some("textDecorationStyle"),
    Some("textDecorationThickness"),
    Some("textEmphasisColor"),
    Some("textEmphasisPosition"),
    Some("textEmphasisStyle"),
    Some("textFit"),
    Some("textIndent"),
    Some("textJustify"),
    Some("textOverflow"),
    Some("textShadow"),
    Some("textTransform"),
    Some("textUnderlineOffset"),
    Some("textUnderlinePosition"),
    Some("textWrapMode"),
    Some("textWrapStyle"),
    Some("timelineScope"),
    Some("timelineTriggerActivationRangeEnd"),
    Some("timelineTriggerActivationRangeStart"),
    Some("timelineTriggerActiveRangeEnd"),
    Some("timelineTriggerActiveRangeStart"),
    Some("timelineTriggerName"),
    Some("timelineTriggerSource"),
    Some("top"),
    Some("touchAction"),
    Some("transform"),
    Some("transformBox"),
    Some("transformOrigin"),
    Some("transformStyle"),
    Some("transitionBehavior"),
    Some("transitionDelay"),
    Some("transitionDuration"),
    Some("transitionProperty"),
    Some("transitionTimingFunction"),
    Some("translate"),
    Some("triggerScope"),
    Some("types"),
    Some("unicodeBidi"),
    Some("unicodeRange"),
    Some("userSelect"),
    Some("vectorEffect"),
    Some("verticalAlign"),
    Some("viewTimelineAxis"),
    Some("viewTimelineInset"),
    Some("viewTimelineName"),
    Some("viewTransitionClass"),
    Some("viewTransitionGroup"),
    Some("viewTransitionName"),
    Some("viewTransitionScope"),
    Some("visibility"),
    Some("webkitBorderHorizontalSpacing"),
    Some("webkitBorderImage"),
    Some("webkitBorderVerticalSpacing"),
    Some("webkitBoxAlign"),
    Some("webkitBoxDecorationBreak"),
    Some("webkitBoxDirection"),
    Some("webkitBoxFlex"),
    Some("webkitBoxOrdinalGroup"),
    Some("webkitBoxOrient"),
    Some("webkitBoxPack"),
    Some("webkitBoxReflect"),
    Some("webkitLineBreak"),
    Some("webkitLineClamp"),
    Some("webkitMaskBoxImageOutset"),
    Some("webkitMaskBoxImageRepeat"),
    Some("webkitMaskBoxImageSlice"),
    Some("webkitMaskBoxImageSource"),
    Some("webkitMaskBoxImageWidth"),
    Some("webkitMaskPositionX"),
    Some("webkitMaskPositionY"),
    Some("webkitPerspectiveOriginX"),
    Some("webkitPerspectiveOriginY"),
    Some("webkitRtlOrdering"),
    Some("webkitRubyPosition"),
    Some("webkitTapHighlightColor"),
    Some("webkitTextCombine"),
    Some("webkitTextDecorationsInEffect"),
    Some("webkitTextFillColor"),
    Some("webkitTextSecurity"),
    Some("webkitTextStrokeColor"),
    Some("webkitTextStrokeWidth"),
    Some("webkitTransformOriginX"),
    Some("webkitTransformOriginY"),
    Some("webkitTransformOriginZ"),
    Some("webkitUserDrag"),
    Some("webkitUserModify"),
    Some("whiteSpaceCollapse"),
    Some("widows"),
    Some("width"),
    Some("willChange"),
    Some("windowDrag"),
    Some("wordBreak"),
    Some("wordSpacing"),
    Some("x"),
    Some("y"),
    Some("zIndex"),
    Some("maskComposite"),
    Some("lineClamp"),
    Some("webkitLineClamp"),
    Some("animation"),
    Some("animationRange"),
    Some("background"),
    Some("backgroundPosition"),
    Some("border"),
    Some("borderBlock"),
    Some("borderBlockColor"),
    Some("borderBlockEnd"),
    Some("borderBlockStart"),
    Some("borderBlockStyle"),
    Some("borderBlockWidth"),
    Some("borderBottom"),
    Some("borderColor"),
    Some("borderImage"),
    Some("borderInline"),
    Some("borderInlineColor"),
    Some("borderInlineEnd"),
    Some("borderInlineStart"),
    Some("borderInlineStyle"),
    Some("borderInlineWidth"),
    Some("borderLeft"),
    Some("borderRadius"),
    Some("borderRight"),
    Some("borderSpacing"),
    Some("borderStyle"),
    Some("borderTop"),
    Some("borderWidth"),
    Some("columnRule"),
    Some("columnRuleInset"),
    Some("columnRuleInsetCap"),
    Some("columnRuleInsetEnd"),
    Some("columnRuleInsetJunction"),
    Some("columnRuleInsetStart"),
    Some("columns"),
    Some("containIntrinsicSize"),
    Some("container"),
    Some("corner"),
    Some("cornerBlockEnd"),
    Some("cornerBlockEndShape"),
    Some("cornerBlockStart"),
    Some("cornerBlockStartShape"),
    Some("cornerBottom"),
    Some("cornerBottomLeft"),
    Some("cornerBottomRight"),
    Some("cornerBottomShape"),
    Some("cornerEndEnd"),
    Some("cornerEndStart"),
    Some("cornerInlineEnd"),
    Some("cornerInlineEndShape"),
    Some("cornerInlineStart"),
    Some("cornerInlineStartShape"),
    Some("cornerLeft"),
    Some("cornerLeftShape"),
    Some("cornerRight"),
    Some("cornerRightShape"),
    Some("cornerShape"),
    Some("cornerStartEnd"),
    Some("cornerStartStart"),
    Some("cornerTop"),
    Some("cornerTopLeft"),
    Some("cornerTopRight"),
    Some("cornerTopShape"),
    Some("flex"),
    Some("flexFlow"),
    Some("font"),
    Some("fontSynthesis"),
    Some("fontVariant"),
    Some("gap"),
    Some("grid"),
    Some("gridArea"),
    Some("gridColumn"),
    Some("gridLanes"),
    Some("gridRow"),
    Some("gridTemplate"),
    Some("inset"),
    Some("insetBlock"),
    Some("insetInline"),
    Some("interestDelay"),
    Some("listStyle"),
    Some("margin"),
    Some("marginBlock"),
    Some("marginInline"),
    Some("marker"),
    Some("mask"),
    Some("maskPosition"),
    Some("offset"),
    Some("outline"),
    Some("overflow"),
    Some("overscrollBehavior"),
    Some("padding"),
    Some("paddingBlock"),
    Some("paddingInline"),
    Some("pageBreakAfter"),
    Some("pageBreakBefore"),
    Some("pageBreakInside"),
    Some("placeContent"),
    Some("placeItems"),
    Some("placeSelf"),
    Some("positionTry"),
    Some("rowRule"),
    Some("rowRuleInset"),
    Some("rowRuleInsetCap"),
    Some("rowRuleInsetEnd"),
    Some("rowRuleInsetJunction"),
    Some("rowRuleInsetStart"),
    Some("rule"),
    Some("ruleBreak"),
    Some("ruleColor"),
    Some("ruleInset"),
    Some("ruleInsetCap"),
    Some("ruleInsetEnd"),
    Some("ruleInsetJunction"),
    Some("ruleInsetStart"),
    Some("ruleStyle"),
    Some("ruleVisibilityItems"),
    Some("ruleWidth"),
    Some("scrollMargin"),
    Some("scrollMarginBlock"),
    Some("scrollMarginInline"),
    Some("scrollPadding"),
    Some("scrollPaddingBlock"),
    Some("scrollPaddingInline"),
    Some("scrollTimeline"),
    Some("textBox"),
    Some("textDecoration"),
    Some("textEmphasis"),
    Some("textSpacing"),
    Some("textWrap"),
    Some("timelineTrigger"),
    Some("timelineTriggerActivationRange"),
    Some("timelineTriggerActiveRange"),
    Some("transition"),
    Some("viewTimeline"),
    Some("webkitColumnBreakAfter"),
    Some("webkitColumnBreakBefore"),
    Some("webkitColumnBreakInside"),
    Some("webkitMaskBoxImage"),
    Some("webkitTextStroke"),
    Some("whiteSpace"),
    Some("webkitAppearance"),
    Some("webkitAppRegion"),
    Some("webkitMaskClip"),
    Some("webkitMaskComposite"),
    Some("webkitMaskImage"),
    Some("webkitMaskOrigin"),
    Some("webkitMaskRepeat"),
    Some("webkitMaskSize"),
    Some("webkitBorderEndColor"),
    Some("webkitBorderEndStyle"),
    Some("webkitBorderEndWidth"),
    Some("webkitBorderStartColor"),
    Some("webkitBorderStartStyle"),
    Some("webkitBorderStartWidth"),
    Some("webkitBorderBeforeColor"),
    Some("webkitBorderBeforeStyle"),
    Some("webkitBorderBeforeWidth"),
    Some("webkitBorderAfterColor"),
    Some("webkitBorderAfterStyle"),
    Some("webkitBorderAfterWidth"),
    Some("webkitMarginEnd"),
    Some("webkitMarginStart"),
    Some("webkitMarginBefore"),
    Some("webkitMarginAfter"),
    Some("webkitPaddingEnd"),
    Some("webkitPaddingStart"),
    Some("webkitPaddingBefore"),
    Some("webkitPaddingAfter"),
    Some("webkitLogicalWidth"),
    Some("webkitLogicalHeight"),
    Some("webkitMinLogicalWidth"),
    Some("webkitMinLogicalHeight"),
    Some("webkitMaxLogicalWidth"),
    Some("webkitMaxLogicalHeight"),
    Some("webkitPrintColorAdjust"),
    Some("webkitBorderAfter"),
    Some("webkitBorderBefore"),
    Some("webkitBorderEnd"),
    Some("webkitBorderStart"),
    Some("webkitMask"),
    Some("webkitMaskPosition"),
    Some("epubCaptionSide"),
    Some("epubTextCombine"),
    Some("epubTextEmphasis"),
    Some("epubTextEmphasisColor"),
    Some("epubTextEmphasisStyle"),
    Some("epubTextOrientation"),
    Some("epubTextTransform"),
    Some("epubWordBreak"),
    Some("epubWritingMode"),
    Some("webkitAlignContent"),
    Some("webkitAlignItems"),
    Some("webkitAlignSelf"),
    Some("webkitAnimation"),
    Some("webkitAnimationDelay"),
    Some("webkitAnimationDirection"),
    Some("webkitAnimationDuration"),
    Some("webkitAnimationFillMode"),
    Some("webkitAnimationIterationCount"),
    Some("webkitAnimationName"),
    Some("webkitAnimationPlayState"),
    Some("webkitAnimationTimingFunction"),
    Some("webkitBackfaceVisibility"),
    Some("webkitBackgroundClip"),
    Some("webkitBackgroundOrigin"),
    Some("webkitBackgroundSize"),
    Some("webkitBorderBottomLeftRadius"),
    Some("webkitBorderBottomRightRadius"),
    Some("webkitBorderRadius"),
    Some("webkitBorderTopLeftRadius"),
    Some("webkitBorderTopRightRadius"),
    Some("webkitBoxShadow"),
    Some("webkitBoxSizing"),
    Some("webkitClipPath"),
    Some("webkitColumnCount"),
    Some("webkitColumnGap"),
    Some("webkitColumnRule"),
    Some("webkitColumnRuleColor"),
    Some("webkitColumnRuleStyle"),
    Some("webkitColumnRuleWidth"),
    Some("webkitColumnSpan"),
    Some("webkitColumnWidth"),
    Some("webkitColumns"),
    Some("webkitFilter"),
    Some("webkitFlex"),
    Some("webkitFlexBasis"),
    Some("webkitFlexDirection"),
    Some("webkitFlexFlow"),
    Some("webkitFlexGrow"),
    Some("webkitFlexShrink"),
    Some("webkitFlexWrap"),
    Some("webkitFontFeatureSettings"),
    Some("webkitHyphenateCharacter"),
    Some("webkitJustifyContent"),
    Some("webkitOpacity"),
    Some("webkitOrder"),
    Some("webkitPerspective"),
    Some("webkitPerspectiveOrigin"),
    Some("webkitShapeImageThreshold"),
    Some("webkitShapeMargin"),
    Some("webkitShapeOutside"),
    Some("webkitTextEmphasis"),
    Some("webkitTextEmphasisColor"),
    Some("webkitTextEmphasisPosition"),
    Some("webkitTextEmphasisStyle"),
    Some("webkitTextSizeAdjust"),
    Some("webkitTransform"),
    Some("webkitTransformOrigin"),
    Some("webkitTransformStyle"),
    Some("webkitTransition"),
    Some("webkitTransitionDelay"),
    Some("webkitTransitionDuration"),
    Some("webkitTransitionProperty"),
    Some("webkitTransitionTimingFunction"),
    Some("webkitUserSelect"),
    Some("wordWrap"),
    Some("gridColumnGap"),
    Some("gridRowGap"),
    Some("gridGap"),
];

pub fn GetPropertyName(id: CSSPropertyID) -> &'static str {
    PROPERTY_NAMES[id as usize].expect("property has no name")
}

pub fn GetJSPropertyName(id: CSSPropertyID) -> &'static str {
    JS_PROPERTY_NAMES[id as usize].expect("property has no JS name")
}

// Each generated getter has its own DEFINE_STATIC_LOCAL. Preserve per-property
// lazy initialization, so asking for one name does not intern the whole table.
static PROPERTY_ATOMIC_NAMES: [OnceLock<AtomicString>; 826] = [const { OnceLock::new() }; 826];

pub fn GetPropertyNameAtomicString(id: CSSPropertyID) -> &'static AtomicString {
    PROPERTY_ATOMIC_NAMES[id as usize].get_or_init(|| AtomicString::from_str(GetPropertyName(id)))
}

// cpp: out/Min/gen/third_party/blink/public/mojom/use_counter/metrics/css_property_id.mojom-data-view.h:43-1764
// Several telemetry labels intentionally share the same integer (notably 0).
// A transparent numeric type preserves those aliases, unlike a Rust enum,
// which cannot have duplicate discriminants. This maps only the enum value
// boundary; Mojo serialization and wire validation remain in their own layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CSSSampleId(pub i32);

impl CSSSampleId {
    pub const kInvalid: Self = Self(0);
    pub const kInternalAlignContentBlock: Self = Self(0);
    pub const kInternalAlignSelfBlock: Self = Self(0);
    pub const kInternalEmptyLineHeight: Self = Self(0);
    pub const kInternalVisitedBackgroundColor: Self = Self(0);
    pub const kInternalVisitedBorderBlockEndColor: Self = Self(0);
    pub const kInternalVisitedBorderBlockStartColor: Self = Self(0);
    pub const kInternalVisitedBorderBottomColor: Self = Self(0);
    pub const kInternalVisitedBorderInlineEndColor: Self = Self(0);
    pub const kInternalVisitedBorderInlineStartColor: Self = Self(0);
    pub const kInternalVisitedBorderLeftColor: Self = Self(0);
    pub const kInternalVisitedBorderRightColor: Self = Self(0);
    pub const kInternalVisitedBorderTopColor: Self = Self(0);
    pub const kInternalVisitedCaretColor: Self = Self(0);
    pub const kInternalVisitedColor: Self = Self(0);
    pub const kInternalVisitedColumnRuleColor: Self = Self(0);
    pub const kInternalVisitedFill: Self = Self(0);
    pub const kInternalVisitedOutlineColor: Self = Self(0);
    pub const kInternalVisitedStroke: Self = Self(0);
    pub const kInternalVisitedTextDecorationColor: Self = Self(0);
    pub const kInternalVisitedTextEmphasisColor: Self = Self(0);
    pub const kInternalVisitedTextFillColor: Self = Self(0);
    pub const kInternalVisitedTextStrokeColor: Self = Self(0);
    pub const kInternalFontSizeDelta: Self = Self(0);
    pub const kInternalForcedBackgroundColor: Self = Self(0);
    pub const kInternalForcedBorderColor: Self = Self(0);
    pub const kInternalForcedColor: Self = Self(0);
    pub const kInternalForcedOutlineColor: Self = Self(0);
    pub const kInternalForcedVisitedColor: Self = Self(0);
    pub const kInternalOverflowBlock: Self = Self(0);
    pub const kInternalOverflowInline: Self = Self(0);
    pub const kInternalOverscrollContainer: Self = Self(0);
    pub const kInternalOverscrollPosition: Self = Self(0);
    pub const kInternalUnbounded: Self = Self(0);
    pub const kTotalPagesMeasured: Self = Self(1);
    pub const kColor: Self = Self(2);
    pub const kDirection: Self = Self(3);
    pub const kDisplay: Self = Self(4);
    pub const kFont: Self = Self(5);
    pub const kFontFamily: Self = Self(6);
    pub const kFontSize: Self = Self(7);
    pub const kFontStyle: Self = Self(8);
    pub const kFontVariant: Self = Self(9);
    pub const kFontWeight: Self = Self(10);
    pub const kTextRendering: Self = Self(11);
    pub const kAliasWebkitFontFeatureSettings: Self = Self(12);
    pub const kFontKerning: Self = Self(13);
    pub const kWebkitFontSmoothing: Self = Self(14);
    pub const kFontVariantLigatures: Self = Self(15);
    pub const kWebkitLocale: Self = Self(16);
    pub const kWebkitTextOrientation: Self = Self(17);
    pub const kWebkitWritingMode: Self = Self(18);
    pub const kZoom: Self = Self(19);
    pub const kLineHeight: Self = Self(20);
    pub const kBackground: Self = Self(21);
    pub const kBackgroundAttachment: Self = Self(22);
    pub const kBackgroundClip: Self = Self(23);
    pub const kBackgroundColor: Self = Self(24);
    pub const kBackgroundImage: Self = Self(25);
    pub const kBackgroundOrigin: Self = Self(26);
    pub const kBackgroundPosition: Self = Self(27);
    pub const kBackgroundPositionX: Self = Self(28);
    pub const kBackgroundPositionY: Self = Self(29);
    pub const kBackgroundRepeat: Self = Self(30);
    pub const kBackgroundSize: Self = Self(33);
    pub const kBorder: Self = Self(34);
    pub const kBorderBottom: Self = Self(35);
    pub const kBorderBottomColor: Self = Self(36);
    pub const kBorderBottomLeftRadius: Self = Self(37);
    pub const kBorderBottomRightRadius: Self = Self(38);
    pub const kBorderBottomStyle: Self = Self(39);
    pub const kBorderBottomWidth: Self = Self(40);
    pub const kBorderCollapse: Self = Self(41);
    pub const kBorderColor: Self = Self(42);
    pub const kBorderImage: Self = Self(43);
    pub const kBorderImageOutset: Self = Self(44);
    pub const kBorderImageRepeat: Self = Self(45);
    pub const kBorderImageSlice: Self = Self(46);
    pub const kBorderImageSource: Self = Self(47);
    pub const kBorderImageWidth: Self = Self(48);
    pub const kBorderLeft: Self = Self(49);
    pub const kBorderLeftColor: Self = Self(50);
    pub const kBorderLeftStyle: Self = Self(51);
    pub const kBorderLeftWidth: Self = Self(52);
    pub const kBorderRadius: Self = Self(53);
    pub const kBorderRight: Self = Self(54);
    pub const kBorderRightColor: Self = Self(55);
    pub const kBorderRightStyle: Self = Self(56);
    pub const kBorderRightWidth: Self = Self(57);
    pub const kBorderSpacing: Self = Self(58);
    pub const kBorderStyle: Self = Self(59);
    pub const kBorderTop: Self = Self(60);
    pub const kBorderTopColor: Self = Self(61);
    pub const kBorderTopLeftRadius: Self = Self(62);
    pub const kBorderTopRightRadius: Self = Self(63);
    pub const kBorderTopStyle: Self = Self(64);
    pub const kBorderTopWidth: Self = Self(65);
    pub const kBorderWidth: Self = Self(66);
    pub const kBottom: Self = Self(67);
    pub const kBoxShadow: Self = Self(68);
    pub const kBoxSizing: Self = Self(69);
    pub const kCaptionSide: Self = Self(70);
    pub const kClear: Self = Self(71);
    pub const kClip: Self = Self(72);
    pub const kAliasWebkitClipPath: Self = Self(73);
    pub const kContent: Self = Self(74);
    pub const kCounterIncrement: Self = Self(75);
    pub const kCounterReset: Self = Self(76);
    pub const kCursor: Self = Self(77);
    pub const kEmptyCells: Self = Self(78);
    pub const kFloat: Self = Self(79);
    pub const kFontStretch: Self = Self(80);
    pub const kHeight: Self = Self(81);
    pub const kImageRendering: Self = Self(82);
    pub const kLeft: Self = Self(83);
    pub const kLetterSpacing: Self = Self(84);
    pub const kListStyle: Self = Self(85);
    pub const kListStyleImage: Self = Self(86);
    pub const kListStylePosition: Self = Self(87);
    pub const kListStyleType: Self = Self(88);
    pub const kMargin: Self = Self(89);
    pub const kMarginBottom: Self = Self(90);
    pub const kMarginLeft: Self = Self(91);
    pub const kMarginRight: Self = Self(92);
    pub const kMarginTop: Self = Self(93);
    pub const kMaxHeight: Self = Self(94);
    pub const kMaxWidth: Self = Self(95);
    pub const kMinHeight: Self = Self(96);
    pub const kMinWidth: Self = Self(97);
    pub const kOpacity: Self = Self(98);
    pub const kOrphans: Self = Self(99);
    pub const kOutline: Self = Self(100);
    pub const kOutlineColor: Self = Self(101);
    pub const kOutlineOffset: Self = Self(102);
    pub const kOutlineStyle: Self = Self(103);
    pub const kOutlineWidth: Self = Self(104);
    pub const kOverflow: Self = Self(105);
    pub const kOverflowWrap: Self = Self(106);
    pub const kOverflowX: Self = Self(107);
    pub const kOverflowY: Self = Self(108);
    pub const kPadding: Self = Self(109);
    pub const kPaddingBottom: Self = Self(110);
    pub const kPaddingLeft: Self = Self(111);
    pub const kPaddingRight: Self = Self(112);
    pub const kPaddingTop: Self = Self(113);
    pub const kPage: Self = Self(114);
    pub const kPageBreakAfter: Self = Self(115);
    pub const kPageBreakBefore: Self = Self(116);
    pub const kPageBreakInside: Self = Self(117);
    pub const kPointerEvents: Self = Self(118);
    pub const kPosition: Self = Self(119);
    pub const kQuotes: Self = Self(120);
    pub const kResize: Self = Self(121);
    pub const kRight: Self = Self(122);
    pub const kSize: Self = Self(123);
    pub const kSrc: Self = Self(124);
    pub const kSpeak: Self = Self(125);
    pub const kTableLayout: Self = Self(126);
    pub const kTabSize: Self = Self(127);
    pub const kTextAlign: Self = Self(128);
    pub const kTextDecoration: Self = Self(129);
    pub const kTextIndent: Self = Self(130);
    pub const kTextOverflow: Self = Self(136);
    pub const kTextShadow: Self = Self(142);
    pub const kTextTransform: Self = Self(143);
    pub const kTop: Self = Self(149);
    pub const kTransition: Self = Self(150);
    pub const kTransitionDelay: Self = Self(151);
    pub const kTransitionDuration: Self = Self(152);
    pub const kTransitionProperty: Self = Self(153);
    pub const kTransitionTimingFunction: Self = Self(154);
    pub const kUnicodeBidi: Self = Self(155);
    pub const kUnicodeRange: Self = Self(156);
    pub const kVerticalAlign: Self = Self(157);
    pub const kVisibility: Self = Self(158);
    pub const kWhiteSpace: Self = Self(159);
    pub const kWidows: Self = Self(160);
    pub const kWidth: Self = Self(161);
    pub const kWordBreak: Self = Self(162);
    pub const kWordSpacing: Self = Self(163);
    pub const kAliasWordWrap: Self = Self(164);
    pub const kZIndex: Self = Self(165);
    pub const kAliasWebkitAnimation: Self = Self(166);
    pub const kAliasWebkitAnimationDelay: Self = Self(167);
    pub const kAliasWebkitAnimationDirection: Self = Self(168);
    pub const kAliasWebkitAnimationDuration: Self = Self(169);
    pub const kAliasWebkitAnimationFillMode: Self = Self(170);
    pub const kAliasWebkitAnimationIterationCount: Self = Self(171);
    pub const kAliasWebkitAnimationName: Self = Self(172);
    pub const kAliasWebkitAnimationPlayState: Self = Self(173);
    pub const kAliasWebkitAnimationTimingFunction: Self = Self(174);
    pub const kAliasWebkitAppearance: Self = Self(175);
    pub const kAliasWebkitBackfaceVisibility: Self = Self(177);
    pub const kAliasWebkitBackgroundClip: Self = Self(178);
    pub const kAliasWebkitBackgroundOrigin: Self = Self(180);
    pub const kAliasWebkitBackgroundSize: Self = Self(181);
    pub const kAliasWebkitBorderAfter: Self = Self(182);
    pub const kAliasWebkitBorderAfterColor: Self = Self(183);
    pub const kAliasWebkitBorderAfterStyle: Self = Self(184);
    pub const kAliasWebkitBorderAfterWidth: Self = Self(185);
    pub const kAliasWebkitBorderBefore: Self = Self(186);
    pub const kAliasWebkitBorderBeforeColor: Self = Self(187);
    pub const kAliasWebkitBorderBeforeStyle: Self = Self(188);
    pub const kAliasWebkitBorderBeforeWidth: Self = Self(189);
    pub const kAliasWebkitBorderEnd: Self = Self(190);
    pub const kAliasWebkitBorderEndColor: Self = Self(191);
    pub const kAliasWebkitBorderEndStyle: Self = Self(192);
    pub const kAliasWebkitBorderEndWidth: Self = Self(193);
    pub const kWebkitBorderHorizontalSpacing: Self = Self(195);
    pub const kWebkitBorderImage: Self = Self(196);
    pub const kAliasWebkitBorderRadius: Self = Self(197);
    pub const kAliasWebkitBorderStart: Self = Self(198);
    pub const kAliasWebkitBorderStartColor: Self = Self(199);
    pub const kAliasWebkitBorderStartStyle: Self = Self(200);
    pub const kAliasWebkitBorderStartWidth: Self = Self(201);
    pub const kWebkitBorderVerticalSpacing: Self = Self(202);
    pub const kWebkitBoxAlign: Self = Self(203);
    pub const kWebkitBoxDirection: Self = Self(204);
    pub const kWebkitBoxFlex: Self = Self(205);
    pub const kWebkitBoxOrdinalGroup: Self = Self(208);
    pub const kWebkitBoxOrient: Self = Self(209);
    pub const kWebkitBoxPack: Self = Self(210);
    pub const kWebkitBoxReflect: Self = Self(211);
    pub const kAliasWebkitBoxShadow: Self = Self(212);
    pub const kWebkitColumnBreakAfter: Self = Self(215);
    pub const kWebkitColumnBreakBefore: Self = Self(216);
    pub const kWebkitColumnBreakInside: Self = Self(217);
    pub const kAliasWebkitColumnCount: Self = Self(218);
    pub const kAliasWebkitColumnGap: Self = Self(219);
    pub const kAliasWebkitColumnRule: Self = Self(221);
    pub const kAliasWebkitColumnRuleColor: Self = Self(222);
    pub const kAliasWebkitColumnRuleStyle: Self = Self(223);
    pub const kAliasWebkitColumnRuleWidth: Self = Self(224);
    pub const kAliasWebkitColumnSpan: Self = Self(225);
    pub const kAliasWebkitColumnWidth: Self = Self(226);
    pub const kAliasWebkitColumns: Self = Self(227);
    pub const kAlignContent: Self = Self(230);
    pub const kAlignItems: Self = Self(231);
    pub const kAlignSelf: Self = Self(232);
    pub const kFlex: Self = Self(233);
    pub const kFlexBasis: Self = Self(234);
    pub const kFlexDirection: Self = Self(235);
    pub const kFlexFlow: Self = Self(236);
    pub const kFlexGrow: Self = Self(237);
    pub const kFlexShrink: Self = Self(238);
    pub const kFlexWrap: Self = Self(239);
    pub const kJustifyContent: Self = Self(240);
    pub const kGridTemplateColumns: Self = Self(242);
    pub const kGridTemplateRows: Self = Self(243);
    pub const kGridColumnStart: Self = Self(244);
    pub const kGridColumnEnd: Self = Self(245);
    pub const kGridRowStart: Self = Self(246);
    pub const kGridRowEnd: Self = Self(247);
    pub const kGridColumn: Self = Self(248);
    pub const kGridRow: Self = Self(249);
    pub const kGridAutoFlow: Self = Self(250);
    pub const kAliasWebkitHyphenateCharacter: Self = Self(252);
    pub const kWebkitLineBreak: Self = Self(259);
    pub const kWebkitLineClamp: Self = Self(260);
    pub const kAliasWebkitLogicalWidth: Self = Self(263);
    pub const kAliasWebkitLogicalHeight: Self = Self(264);
    pub const kWebkitMarginAfterCollapse: Self = Self(265);
    pub const kWebkitMarginBeforeCollapse: Self = Self(266);
    pub const kWebkitMarginBottomCollapse: Self = Self(267);
    pub const kWebkitMarginTopCollapse: Self = Self(268);
    pub const kWebkitMarginCollapse: Self = Self(269);
    pub const kAliasWebkitMarginAfter: Self = Self(270);
    pub const kAliasWebkitMarginBefore: Self = Self(271);
    pub const kAliasWebkitMarginEnd: Self = Self(272);
    pub const kAliasWebkitMarginStart: Self = Self(273);
    pub const kAliasWebkitMask: Self = Self(280);
    pub const kWebkitMaskBoxImage: Self = Self(281);
    pub const kWebkitMaskBoxImageOutset: Self = Self(282);
    pub const kWebkitMaskBoxImageRepeat: Self = Self(283);
    pub const kWebkitMaskBoxImageSlice: Self = Self(284);
    pub const kWebkitMaskBoxImageSource: Self = Self(285);
    pub const kWebkitMaskBoxImageWidth: Self = Self(286);
    pub const kAliasWebkitMaskClip: Self = Self(287);
    pub const kAliasWebkitMaskComposite: Self = Self(288);
    pub const kAliasWebkitMaskImage: Self = Self(289);
    pub const kAliasWebkitMaskOrigin: Self = Self(290);
    pub const kAliasWebkitMaskPosition: Self = Self(291);
    pub const kWebkitMaskPositionX: Self = Self(292);
    pub const kWebkitMaskPositionY: Self = Self(293);
    pub const kAliasWebkitMaskRepeat: Self = Self(294);
    pub const kAliasWebkitMaskSize: Self = Self(297);
    pub const kAliasWebkitMaxLogicalWidth: Self = Self(298);
    pub const kAliasWebkitMaxLogicalHeight: Self = Self(299);
    pub const kAliasWebkitMinLogicalWidth: Self = Self(300);
    pub const kAliasWebkitMinLogicalHeight: Self = Self(301);
    pub const kOrder: Self = Self(303);
    pub const kAliasWebkitPaddingAfter: Self = Self(304);
    pub const kAliasWebkitPaddingBefore: Self = Self(305);
    pub const kAliasWebkitPaddingEnd: Self = Self(306);
    pub const kAliasWebkitPaddingStart: Self = Self(307);
    pub const kAliasWebkitPerspective: Self = Self(308);
    pub const kAliasWebkitPerspectiveOrigin: Self = Self(309);
    pub const kWebkitPerspectiveOriginX: Self = Self(310);
    pub const kWebkitPerspectiveOriginY: Self = Self(311);
    pub const kAliasWebkitPrintColorAdjust: Self = Self(312);
    pub const kWebkitRtlOrdering: Self = Self(313);
    pub const kWebkitRubyPosition: Self = Self(314);
    pub const kWebkitTextCombine: Self = Self(315);
    pub const kWebkitTextDecorationsInEffect: Self = Self(316);
    pub const kAliasWebkitTextEmphasis: Self = Self(317);
    pub const kAliasWebkitTextEmphasisColor: Self = Self(318);
    pub const kAliasWebkitTextEmphasisPosition: Self = Self(319);
    pub const kAliasWebkitTextEmphasisStyle: Self = Self(320);
    pub const kWebkitTextFillColor: Self = Self(321);
    pub const kWebkitTextSecurity: Self = Self(322);
    pub const kWebkitTextStroke: Self = Self(323);
    pub const kWebkitTextStrokeColor: Self = Self(324);
    pub const kWebkitTextStrokeWidth: Self = Self(325);
    pub const kAliasWebkitTransform: Self = Self(326);
    pub const kAliasWebkitTransformOrigin: Self = Self(327);
    pub const kWebkitTransformOriginX: Self = Self(328);
    pub const kWebkitTransformOriginY: Self = Self(329);
    pub const kWebkitTransformOriginZ: Self = Self(330);
    pub const kAliasWebkitTransformStyle: Self = Self(331);
    pub const kAliasWebkitTransition: Self = Self(332);
    pub const kAliasWebkitTransitionDelay: Self = Self(333);
    pub const kAliasWebkitTransitionDuration: Self = Self(334);
    pub const kAliasWebkitTransitionProperty: Self = Self(335);
    pub const kAliasWebkitTransitionTimingFunction: Self = Self(336);
    pub const kWebkitUserDrag: Self = Self(337);
    pub const kWebkitUserModify: Self = Self(338);
    pub const kAliasWebkitUserSelect: Self = Self(339);
    pub const kShapeOutside: Self = Self(347);
    pub const kShapeMargin: Self = Self(348);
    pub const kClipPath: Self = Self(355);
    pub const kClipRule: Self = Self(356);
    pub const kMask: Self = Self(357);
    pub const kFilter: Self = Self(359);
    pub const kFloodColor: Self = Self(360);
    pub const kFloodOpacity: Self = Self(361);
    pub const kLightingColor: Self = Self(362);
    pub const kStopColor: Self = Self(363);
    pub const kStopOpacity: Self = Self(364);
    pub const kColorInterpolation: Self = Self(365);
    pub const kColorInterpolationFilters: Self = Self(366);
    pub const kColorRendering: Self = Self(368);
    pub const kFill: Self = Self(369);
    pub const kFillOpacity: Self = Self(370);
    pub const kFillRule: Self = Self(371);
    pub const kMarker: Self = Self(372);
    pub const kMarkerEnd: Self = Self(373);
    pub const kMarkerMid: Self = Self(374);
    pub const kMarkerStart: Self = Self(375);
    pub const kMaskType: Self = Self(376);
    pub const kShapeRendering: Self = Self(377);
    pub const kStroke: Self = Self(378);
    pub const kStrokeDasharray: Self = Self(379);
    pub const kStrokeDashoffset: Self = Self(380);
    pub const kStrokeLinecap: Self = Self(381);
    pub const kStrokeLinejoin: Self = Self(382);
    pub const kStrokeMiterlimit: Self = Self(383);
    pub const kStrokeOpacity: Self = Self(384);
    pub const kStrokeWidth: Self = Self(385);
    pub const kAlignmentBaseline: Self = Self(386);
    pub const kBaselineShift: Self = Self(387);
    pub const kDominantBaseline: Self = Self(388);
    pub const kTextAnchor: Self = Self(392);
    pub const kVectorEffect: Self = Self(393);
    pub const kWritingMode: Self = Self(394);
    pub const kTextDecorationLine: Self = Self(401);
    pub const kTextDecorationStyle: Self = Self(402);
    pub const kTextDecorationColor: Self = Self(403);
    pub const kTextAlignLast: Self = Self(404);
    pub const kTextUnderlinePosition: Self = Self(405);
    pub const kMaxZoom: Self = Self(406);
    pub const kMinZoom: Self = Self(407);
    pub const kOrientation: Self = Self(408);
    pub const kUserZoom: Self = Self(409);
    pub const kAliasWebkitAppRegion: Self = Self(412);
    pub const kAliasWebkitFilter: Self = Self(413);
    pub const kWebkitBoxDecorationBreak: Self = Self(414);
    pub const kWebkitTapHighlightColor: Self = Self(415);
    pub const kBufferedRendering: Self = Self(416);
    pub const kGridAutoRows: Self = Self(417);
    pub const kGridAutoColumns: Self = Self(418);
    pub const kBackgroundBlendMode: Self = Self(419);
    pub const kMixBlendMode: Self = Self(420);
    pub const kTouchAction: Self = Self(421);
    pub const kGridArea: Self = Self(422);
    pub const kGridTemplateAreas: Self = Self(423);
    pub const kAnimation: Self = Self(424);
    pub const kAnimationDelay: Self = Self(425);
    pub const kAnimationDirection: Self = Self(426);
    pub const kAnimationDuration: Self = Self(427);
    pub const kAnimationFillMode: Self = Self(428);
    pub const kAnimationIterationCount: Self = Self(429);
    pub const kAnimationName: Self = Self(430);
    pub const kAnimationPlayState: Self = Self(431);
    pub const kAnimationTimingFunction: Self = Self(432);
    pub const kObjectFit: Self = Self(433);
    pub const kPaintOrder: Self = Self(434);
    pub const kMaskSourceType: Self = Self(435);
    pub const kIsolation: Self = Self(436);
    pub const kObjectPosition: Self = Self(437);
    pub const kShapeImageThreshold: Self = Self(439);
    pub const kColumnFill: Self = Self(440);
    pub const kTextJustify: Self = Self(441);
    pub const kJustifySelf: Self = Self(443);
    pub const kScrollBehavior: Self = Self(444);
    pub const kWillChange: Self = Self(445);
    pub const kTransform: Self = Self(446);
    pub const kTransformOrigin: Self = Self(447);
    pub const kTransformStyle: Self = Self(448);
    pub const kPerspective: Self = Self(449);
    pub const kPerspectiveOrigin: Self = Self(450);
    pub const kBackfaceVisibility: Self = Self(451);
    pub const kGridTemplate: Self = Self(452);
    pub const kGrid: Self = Self(453);
    pub const kAll: Self = Self(454);
    pub const kJustifyItems: Self = Self(455);
    pub const kX: Self = Self(461);
    pub const kY: Self = Self(462);
    pub const kRx: Self = Self(463);
    pub const kRy: Self = Self(464);
    pub const kFontSizeAdjust: Self = Self(465);
    pub const kCx: Self = Self(466);
    pub const kCy: Self = Self(467);
    pub const kR: Self = Self(468);
    pub const kAliasEpubCaptionSide: Self = Self(469);
    pub const kAliasEpubTextCombine: Self = Self(470);
    pub const kAliasEpubTextEmphasis: Self = Self(471);
    pub const kAliasEpubTextEmphasisColor: Self = Self(472);
    pub const kAliasEpubTextEmphasisStyle: Self = Self(473);
    pub const kAliasEpubTextOrientation: Self = Self(474);
    pub const kAliasEpubTextTransform: Self = Self(475);
    pub const kAliasEpubWordBreak: Self = Self(476);
    pub const kAliasEpubWritingMode: Self = Self(477);
    pub const kAliasWebkitAlignContent: Self = Self(478);
    pub const kAliasWebkitAlignItems: Self = Self(479);
    pub const kAliasWebkitAlignSelf: Self = Self(480);
    pub const kAliasWebkitBorderBottomLeftRadius: Self = Self(481);
    pub const kAliasWebkitBorderBottomRightRadius: Self = Self(482);
    pub const kAliasWebkitBorderTopLeftRadius: Self = Self(483);
    pub const kAliasWebkitBorderTopRightRadius: Self = Self(484);
    pub const kAliasWebkitBoxSizing: Self = Self(485);
    pub const kAliasWebkitFlex: Self = Self(486);
    pub const kAliasWebkitFlexBasis: Self = Self(487);
    pub const kAliasWebkitFlexDirection: Self = Self(488);
    pub const kAliasWebkitFlexFlow: Self = Self(489);
    pub const kAliasWebkitFlexGrow: Self = Self(490);
    pub const kAliasWebkitFlexShrink: Self = Self(491);
    pub const kAliasWebkitFlexWrap: Self = Self(492);
    pub const kAliasWebkitJustifyContent: Self = Self(493);
    pub const kAliasWebkitOpacity: Self = Self(494);
    pub const kAliasWebkitOrder: Self = Self(495);
    pub const kAliasWebkitShapeImageThreshold: Self = Self(496);
    pub const kAliasWebkitShapeMargin: Self = Self(497);
    pub const kAliasWebkitShapeOutside: Self = Self(498);
    pub const kScrollSnapType: Self = Self(499);
    pub const kTranslate: Self = Self(504);
    pub const kRotate: Self = Self(505);
    pub const kScale: Self = Self(506);
    pub const kImageOrientation: Self = Self(507);
    pub const kBackdropFilter: Self = Self(508);
    pub const kTextCombineUpright: Self = Self(509);
    pub const kTextOrientation: Self = Self(510);
    pub const kAliasGridColumnGap: Self = Self(511);
    pub const kAliasGridRowGap: Self = Self(512);
    pub const kAliasGridGap: Self = Self(513);
    pub const kFontFeatureSettings: Self = Self(514);
    pub const kVariable: Self = Self(515);
    pub const kFontDisplay: Self = Self(516);
    pub const kContain: Self = Self(517);
    pub const kD: Self = Self(518);
    pub const kBreakAfter: Self = Self(520);
    pub const kBreakBefore: Self = Self(521);
    pub const kBreakInside: Self = Self(522);
    pub const kColumnCount: Self = Self(523);
    pub const kColumnGap: Self = Self(524);
    pub const kColumnRule: Self = Self(525);
    pub const kColumnRuleColor: Self = Self(526);
    pub const kColumnRuleStyle: Self = Self(527);
    pub const kColumnRuleWidth: Self = Self(528);
    pub const kColumnSpan: Self = Self(529);
    pub const kColumnWidth: Self = Self(530);
    pub const kColumns: Self = Self(531);
    pub const kFontVariantCaps: Self = Self(533);
    pub const kHyphens: Self = Self(534);
    pub const kFontVariantNumeric: Self = Self(535);
    pub const kTextSizeAdjust: Self = Self(536);
    pub const kAliasWebkitTextSizeAdjust: Self = Self(537);
    pub const kOverflowAnchor: Self = Self(538);
    pub const kUserSelect: Self = Self(539);
    pub const kOffsetDistance: Self = Self(540);
    pub const kOffsetPath: Self = Self(541);
    pub const kOffset: Self = Self(543);
    pub const kOffsetAnchor: Self = Self(544);
    pub const kOffsetPosition: Self = Self(545);
    pub const kCaretColor: Self = Self(547);
    pub const kOffsetRotate: Self = Self(548);
    pub const kFontVariationSettings: Self = Self(549);
    pub const kInlineSize: Self = Self(550);
    pub const kBlockSize: Self = Self(551);
    pub const kMinInlineSize: Self = Self(552);
    pub const kMinBlockSize: Self = Self(553);
    pub const kMaxInlineSize: Self = Self(554);
    pub const kMaxBlockSize: Self = Self(555);
    pub const kLineBreak: Self = Self(556);
    pub const kPlaceContent: Self = Self(557);
    pub const kPlaceItems: Self = Self(558);
    pub const kTransformBox: Self = Self(559);
    pub const kPlaceSelf: Self = Self(560);
    pub const kScrollSnapAlign: Self = Self(561);
    pub const kScrollPadding: Self = Self(562);
    pub const kScrollPaddingTop: Self = Self(563);
    pub const kScrollPaddingRight: Self = Self(564);
    pub const kScrollPaddingBottom: Self = Self(565);
    pub const kScrollPaddingLeft: Self = Self(566);
    pub const kScrollPaddingBlock: Self = Self(567);
    pub const kScrollPaddingBlockStart: Self = Self(568);
    pub const kScrollPaddingBlockEnd: Self = Self(569);
    pub const kScrollPaddingInline: Self = Self(570);
    pub const kScrollPaddingInlineStart: Self = Self(571);
    pub const kScrollPaddingInlineEnd: Self = Self(572);
    pub const kScrollMargin: Self = Self(573);
    pub const kScrollMarginTop: Self = Self(574);
    pub const kScrollMarginRight: Self = Self(575);
    pub const kScrollMarginBottom: Self = Self(576);
    pub const kScrollMarginLeft: Self = Self(577);
    pub const kScrollMarginBlock: Self = Self(578);
    pub const kScrollMarginBlockStart: Self = Self(579);
    pub const kScrollMarginBlockEnd: Self = Self(580);
    pub const kScrollMarginInline: Self = Self(581);
    pub const kScrollMarginInlineStart: Self = Self(582);
    pub const kScrollMarginInlineEnd: Self = Self(583);
    pub const kScrollSnapStop: Self = Self(584);
    pub const kOverscrollBehavior: Self = Self(585);
    pub const kOverscrollBehaviorX: Self = Self(586);
    pub const kOverscrollBehaviorY: Self = Self(587);
    pub const kFontVariantEastAsian: Self = Self(588);
    pub const kTextDecorationSkipInk: Self = Self(589);
    pub const kScrollCustomization: Self = Self(590);
    pub const kRowGap: Self = Self(591);
    pub const kGap: Self = Self(592);
    pub const kViewportFit: Self = Self(593);
    pub const kMarginBlockStart: Self = Self(594);
    pub const kMarginBlockEnd: Self = Self(595);
    pub const kMarginInlineStart: Self = Self(596);
    pub const kMarginInlineEnd: Self = Self(597);
    pub const kPaddingBlockStart: Self = Self(598);
    pub const kPaddingBlockEnd: Self = Self(599);
    pub const kPaddingInlineStart: Self = Self(600);
    pub const kPaddingInlineEnd: Self = Self(601);
    pub const kBorderBlockEndColor: Self = Self(602);
    pub const kBorderBlockEndStyle: Self = Self(603);
    pub const kBorderBlockEndWidth: Self = Self(604);
    pub const kBorderBlockStartColor: Self = Self(605);
    pub const kBorderBlockStartStyle: Self = Self(606);
    pub const kBorderBlockStartWidth: Self = Self(607);
    pub const kBorderInlineEndColor: Self = Self(608);
    pub const kBorderInlineEndStyle: Self = Self(609);
    pub const kBorderInlineEndWidth: Self = Self(610);
    pub const kBorderInlineStartColor: Self = Self(611);
    pub const kBorderInlineStartStyle: Self = Self(612);
    pub const kBorderInlineStartWidth: Self = Self(613);
    pub const kBorderBlockStart: Self = Self(614);
    pub const kBorderBlockEnd: Self = Self(615);
    pub const kBorderInlineStart: Self = Self(616);
    pub const kBorderInlineEnd: Self = Self(617);
    pub const kMarginBlock: Self = Self(618);
    pub const kMarginInline: Self = Self(619);
    pub const kPaddingBlock: Self = Self(620);
    pub const kPaddingInline: Self = Self(621);
    pub const kBorderBlockColor: Self = Self(622);
    pub const kBorderBlockStyle: Self = Self(623);
    pub const kBorderBlockWidth: Self = Self(624);
    pub const kBorderInlineColor: Self = Self(625);
    pub const kBorderInlineStyle: Self = Self(626);
    pub const kBorderInlineWidth: Self = Self(627);
    pub const kBorderBlock: Self = Self(628);
    pub const kBorderInline: Self = Self(629);
    pub const kInsetBlockStart: Self = Self(630);
    pub const kInsetBlockEnd: Self = Self(631);
    pub const kInsetBlock: Self = Self(632);
    pub const kInsetInlineStart: Self = Self(633);
    pub const kInsetInlineEnd: Self = Self(634);
    pub const kInsetInline: Self = Self(635);
    pub const kInset: Self = Self(636);
    pub const kColorScheme: Self = Self(637);
    pub const kOverflowInline: Self = Self(638);
    pub const kOverflowBlock: Self = Self(639);
    pub const kForcedColorAdjust: Self = Self(640);
    pub const kInherits: Self = Self(641);
    pub const kInitialValue: Self = Self(642);
    pub const kSyntax: Self = Self(643);
    pub const kOverscrollBehaviorInline: Self = Self(644);
    pub const kOverscrollBehaviorBlock: Self = Self(645);
    pub const kFontOpticalSizing: Self = Self(647);
    pub const kContainIntrinsicBlockSize: Self = Self(648);
    pub const kContainIntrinsicHeight: Self = Self(649);
    pub const kContainIntrinsicInlineSize: Self = Self(650);
    pub const kContainIntrinsicSize: Self = Self(651);
    pub const kContainIntrinsicWidth: Self = Self(652);
    pub const kOriginTrialTestProperty: Self = Self(654);
    pub const kMathStyle: Self = Self(656);
    pub const kAspectRatio: Self = Self(657);
    pub const kAppearance: Self = Self(658);
    pub const kRubyPosition: Self = Self(660);
    pub const kTextUnderlineOffset: Self = Self(661);
    pub const kContentVisibility: Self = Self(662);
    pub const kTextDecorationThickness: Self = Self(663);
    pub const kPageOrientation: Self = Self(664);
    pub const kAnimationTimeline: Self = Self(665);
    pub const kCounterSet: Self = Self(666);
    pub const kSource: Self = Self(667);
    pub const kStart: Self = Self(668);
    pub const kEnd: Self = Self(669);
    pub const kTimeRange: Self = Self(670);
    pub const kScrollbarGutter: Self = Self(671);
    pub const kAscentOverride: Self = Self(672);
    pub const kDescentOverride: Self = Self(673);
    pub const kAdvanceOverride: Self = Self(674);
    pub const kLineGapOverride: Self = Self(675);
    pub const kMathShift: Self = Self(676);
    pub const kMathDepth: Self = Self(677);
    pub const kOverflowClipMargin: Self = Self(679);
    pub const kScrollbarWidth: Self = Self(680);
    pub const kSystem: Self = Self(681);
    pub const kNegative: Self = Self(682);
    pub const kPrefix: Self = Self(683);
    pub const kSuffix: Self = Self(684);
    pub const kRange: Self = Self(685);
    pub const kPad: Self = Self(686);
    pub const kFallback: Self = Self(687);
    pub const kSymbols: Self = Self(688);
    pub const kAdditiveSymbols: Self = Self(689);
    pub const kSpeakAs: Self = Self(690);
    pub const kBorderStartStartRadius: Self = Self(691);
    pub const kBorderStartEndRadius: Self = Self(692);
    pub const kBorderEndStartRadius: Self = Self(693);
    pub const kBorderEndEndRadius: Self = Self(694);
    pub const kAccentColor: Self = Self(695);
    pub const kSizeAdjust: Self = Self(696);
    pub const kContainerName: Self = Self(697);
    pub const kContainerType: Self = Self(698);
    pub const kContainer: Self = Self(699);
    pub const kFontSynthesisWeight: Self = Self(700);
    pub const kFontSynthesisStyle: Self = Self(701);
    pub const kAppRegion: Self = Self(702);
    pub const kFontSynthesisSmallCaps: Self = Self(703);
    pub const kFontSynthesis: Self = Self(704);
    pub const kTextEmphasis: Self = Self(705);
    pub const kTextEmphasisColor: Self = Self(706);
    pub const kTextEmphasisPosition: Self = Self(707);
    pub const kTextEmphasisStyle: Self = Self(708);
    pub const kFontPalette: Self = Self(709);
    pub const kBasePalette: Self = Self(710);
    pub const kOverrideColors: Self = Self(711);
    pub const kViewTransitionName: Self = Self(712);
    pub const kObjectViewBox: Self = Self(713);
    pub const kObjectOverflow: Self = Self(714);
    pub const kAnchorName: Self = Self(719);
    pub const kPositionFallback: Self = Self(720);
    pub const kPopoverShowDelay: Self = Self(722);
    pub const kPopoverHideDelay: Self = Self(723);
    pub const kHyphenateCharacter: Self = Self(724);
    pub const kScrollTimeline: Self = Self(725);
    pub const kScrollTimelineName: Self = Self(726);
    pub const kScrollTimelineAxis: Self = Self(727);
    pub const kViewTimeline: Self = Self(728);
    pub const kViewTimelineAxis: Self = Self(729);
    pub const kViewTimelineInset: Self = Self(730);
    pub const kViewTimelineName: Self = Self(731);
    pub const kInitialLetter: Self = Self(733);
    pub const kHyphenateLimitChars: Self = Self(734);
    pub const kAnimationDelayStart: Self = Self(735);
    pub const kAnimationDelayEnd: Self = Self(736);
    pub const kFontVariantPosition: Self = Self(737);
    pub const kFontVariantAlternates: Self = Self(738);
    pub const kBaselineSource: Self = Self(739);
    pub const kAnimationRange: Self = Self(740);
    pub const kAnimationRangeStart: Self = Self(741);
    pub const kAnimationRangeEnd: Self = Self(742);
    pub const kAnimationComposition: Self = Self(743);
    pub const kTopLayer: Self = Self(744);
    pub const kTextWrap: Self = Self(746);
    pub const kTextBoxTrim: Self = Self(747);
    pub const kOverlay: Self = Self(748);
    pub const kWhiteSpaceCollapse: Self = Self(749);
    pub const kScrollTimelineAttachment: Self = Self(750);
    pub const kViewTimelineAttachment: Self = Self(751);
    pub const kScrollInitialTarget: Self = Self(761);
    pub const kTimelineScope: Self = Self(762);
    pub const kScrollbarColor: Self = Self(763);
    pub const kPositionFallbackBounds: Self = Self(765);
    pub const kTransitionBehavior: Self = Self(766);
    pub const kTextAutospace: Self = Self(767);
    pub const kNavigation: Self = Self(768);
    pub const kDynamicRangeLimit: Self = Self(769);
    pub const kFieldSizing: Self = Self(770);
    pub const kTextSpacingTrim: Self = Self(771);
    pub const kMaskImage: Self = Self(772);
    pub const kMaskClip: Self = Self(773);
    pub const kMaskSize: Self = Self(774);
    pub const kMaskOrigin: Self = Self(775);
    pub const kTextSpacing: Self = Self(776);
    pub const kMaskRepeat: Self = Self(777);
    pub const kMaskComposite: Self = Self(778);
    pub const kMaskPosition: Self = Self(779);
    pub const kMaskMode: Self = Self(780);
    pub const kViewTransitionClass: Self = Self(782);
    pub const kPositionTryOrder: Self = Self(783);
    pub const kPositionTry: Self = Self(785);
    pub const kTextBoxEdge: Self = Self(786);
    pub const kReadingFlow: Self = Self(787);
    pub const kPositionAnchor: Self = Self(788);
    pub const kPositionVisibility: Self = Self(789);
    pub const kTypes: Self = Self(790);
    pub const kLineClamp: Self = Self(791);
    pub const kFontVariantEmoji: Self = Self(792);
    pub const kScrollMarkerGroup: Self = Self(793);
    pub const kAnchorScope: Self = Self(794);
    pub const kRubyAlign: Self = Self(795);
    pub const kBoxDecorationBreak: Self = Self(796);
    pub const kPositionTryFallbacks: Self = Self(797);
    pub const kInterpolateSize: Self = Self(798);
    pub const kViewTransitionGroup: Self = Self(799);
    pub const kPositionArea: Self = Self(800);
    pub const kTextBox: Self = Self(801);
    pub const kTextWrapMode: Self = Self(805);
    pub const kTextWrapStyle: Self = Self(806);
    pub const kFlowTolerance: Self = Self(808);
    pub const kCaretAnimation: Self = Self(809);
    pub const kInteractivity: Self = Self(811);
    pub const kGridLanesDirection: Self = Self(813);
    pub const kResult: Self = Self(816);
    pub const kInterestDelayStart: Self = Self(823);
    pub const kInterestDelayEnd: Self = Self(824);
    pub const kAnimationTriggerRange: Self = Self(825);
    pub const kAnimationTriggerExitRange: Self = Self(826);
    pub const kInterestDelay: Self = Self(827);
    pub const kAnimationTrigger: Self = Self(828);
    pub const kColumnRuleBreak: Self = Self(829);
    pub const kRowRuleBreak: Self = Self(830);
    pub const kCornerBottomLeftShape: Self = Self(831);
    pub const kCornerBottomRightShape: Self = Self(832);
    pub const kCornerTopLeftShape: Self = Self(833);
    pub const kCornerTopRightShape: Self = Self(834);
    pub const kCornerStartStartShape: Self = Self(835);
    pub const kCornerStartEndShape: Self = Self(836);
    pub const kCornerEndStartShape: Self = Self(837);
    pub const kCornerEndEndShape: Self = Self(838);
    pub const kReadingOrder: Self = Self(839);
    pub const kColumnRuleInset: Self = Self(840);
    pub const kRowRuleInset: Self = Self(841);
    pub const kCornerShape: Self = Self(842);
    pub const kScrollTargetGroup: Self = Self(843);
    pub const kRowRuleStyle: Self = Self(844);
    pub const kRowRuleWidth: Self = Self(846);
    pub const kRowRuleColor: Self = Self(847);
    pub const kPrintColorAdjust: Self = Self(848);
    pub const kColumnHeight: Self = Self(849);
    pub const kColumnWrap: Self = Self(850);
    pub const kRuleColor: Self = Self(851);
    pub const kRuleWidth: Self = Self(852);
    pub const kRuleStyle: Self = Self(853);
    pub const kCaretShape: Self = Self(854);
    pub const kRowRule: Self = Self(855);
    pub const kRule: Self = Self(858);
    pub const kCornerTopShape: Self = Self(860);
    pub const kCornerRightShape: Self = Self(861);
    pub const kCornerBottomShape: Self = Self(862);
    pub const kCornerLeftShape: Self = Self(863);
    pub const kCornerBlockStartShape: Self = Self(864);
    pub const kCornerBlockEndShape: Self = Self(865);
    pub const kCornerInlineStartShape: Self = Self(866);
    pub const kCornerInlineEndShape: Self = Self(867);
    pub const kBorderShape: Self = Self(869);
    pub const kTimelineTriggerName: Self = Self(871);
    pub const kTimelineTriggerBehavior: Self = Self(872);
    pub const kTimelineTriggerSource: Self = Self(877);
    pub const kTimelineTrigger: Self = Self(878);
    pub const kGridLanes: Self = Self(879);
    pub const kContinue: Self = Self(880);
    pub const kMaxLines: Self = Self(881);
    pub const kBlockEllipsis: Self = Self(882);
    pub const kOverscrollArea: Self = Self(883);
    pub const kOverscrollPosition: Self = Self(884);
    pub const kFontLanguageOverride: Self = Self(885);
    pub const kRubyOverhang: Self = Self(886);
    pub const kRuleBreak: Self = Self(887);
    pub const kRuleInset: Self = Self(888);
    pub const kColumnRuleVisibilityItems: Self = Self(889);
    pub const kRowRuleVisibilityItems: Self = Self(890);
    pub const kColumnRuleInsetCapStart: Self = Self(891);
    pub const kColumnRuleInsetCapEnd: Self = Self(892);
    pub const kColumnRuleInsetJunctionStart: Self = Self(893);
    pub const kColumnRuleInsetJunctionEnd: Self = Self(894);
    pub const kRowRuleInsetCapStart: Self = Self(895);
    pub const kRowRuleInsetCapEnd: Self = Self(896);
    pub const kRowRuleInsetJunctionStart: Self = Self(897);
    pub const kRowRuleInsetJunctionEnd: Self = Self(898);
    pub const kTriggerScope: Self = Self(901);
    pub const kProtocol: Self = Self(902);
    pub const kHostname: Self = Self(903);
    pub const kPort: Self = Self(904);
    pub const kPathname: Self = Self(905);
    pub const kSearch: Self = Self(906);
    pub const kHash: Self = Self(907);
    pub const kBaseUrl: Self = Self(908);
    pub const kPattern: Self = Self(909);
    pub const kColumnRuleInsetCap: Self = Self(910);
    pub const kRowRuleInsetCap: Self = Self(911);
    pub const kColumnRuleInsetJunction: Self = Self(912);
    pub const kRowRuleInsetJunction: Self = Self(913);
    pub const kRuleInsetCap: Self = Self(914);
    pub const kRuleInsetJunction: Self = Self(915);
    pub const kPageMarginSafety: Self = Self(916);
    pub const kTimelineTriggerActiveRangeStart: Self = Self(919);
    pub const kTimelineTriggerActiveRangeEnd: Self = Self(920);
    pub const kTimelineTriggerActiveRange: Self = Self(922);
    pub const kGridLanesPack: Self = Self(923);
    pub const kTextFit: Self = Self(924);
    pub const kViewTransitionScope: Self = Self(925);
    pub const kRuleVisibilityItems: Self = Self(926);
    pub const kFrameSizing: Self = Self(927);
    pub const kImageAnimation: Self = Self(928);
    pub const kMarginTrim: Self = Self(929);
    pub const kTimelineTriggerActivationRangeStart: Self = Self(930);
    pub const kTimelineTriggerActivationRangeEnd: Self = Self(931);
    pub const kTimelineTriggerActivationRange: Self = Self(932);
    pub const kRuleOverlap: Self = Self(933);
    pub const kColumnRuleInsetStart: Self = Self(934);
    pub const kColumnRuleInsetEnd: Self = Self(935);
    pub const kRowRuleInsetStart: Self = Self(936);
    pub const kRowRuleInsetEnd: Self = Self(937);
    pub const kRuleInsetStart: Self = Self(938);
    pub const kRuleInsetEnd: Self = Self(939);
    pub const kPathLength: Self = Self(940);
    pub const kHangingPunctuation: Self = Self(941);
    pub const kFlexLineCount: Self = Self(942);
    pub const kTextDecorationSkipSpaces: Self = Self(943);
    pub const kCorner: Self = Self(944);
    pub const kScrollAxisLock: Self = Self(945);
    pub const kCornerTopLeft: Self = Self(946);
    pub const kCornerTopRight: Self = Self(947);
    pub const kCornerBottomLeft: Self = Self(948);
    pub const kCornerBottomRight: Self = Self(949);
    pub const kCornerStartStart: Self = Self(950);
    pub const kCornerStartEnd: Self = Self(951);
    pub const kCornerEndStart: Self = Self(952);
    pub const kCornerEndEnd: Self = Self(953);
    pub const kCornerTop: Self = Self(954);
    pub const kCornerRight: Self = Self(955);
    pub const kCornerBottom: Self = Self(956);
    pub const kCornerLeft: Self = Self(957);
    pub const kCornerBlockStart: Self = Self(958);
    pub const kCornerBlockEnd: Self = Self(959);
    pub const kCornerInlineStart: Self = Self(960);
    pub const kCornerInlineEnd: Self = Self(961);
    pub const kWindowDrag: Self = Self(962);
    pub const kMaxContentSizing: Self = Self(963);
    pub const kTextDecorationInset: Self = Self(964);
    pub const kOverscrollContainerType: Self = Self(965);
    pub const kMinValue: Self = Self(0);
    pub const kMaxValue: Self = Self(965);
}

// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.h:1608
// cpp: out/Min/gen/third_party/blink/renderer/core/css/css_property_names.cc:3423-5085
pub fn GetCSSSampleId(id: CSSPropertyID) -> CSSSampleId {
    match id {
        CSSPropertyID::kColorScheme => CSSSampleId::kColorScheme,
        CSSPropertyID::kForcedColorAdjust => CSSSampleId::kForcedColorAdjust,
        CSSPropertyID::kMathDepth => CSSSampleId::kMathDepth,
        CSSPropertyID::kPosition => CSSSampleId::kPosition,
        CSSPropertyID::kPositionAnchor => CSSSampleId::kPositionAnchor,
        CSSPropertyID::kTextSizeAdjust => CSSSampleId::kTextSizeAdjust,
        CSSPropertyID::kInternalVisitedColor => CSSSampleId::kInternalVisitedColor,
        CSSPropertyID::kAppearance => CSSSampleId::kAppearance,
        CSSPropertyID::kColor => CSSSampleId::kColor,
        CSSPropertyID::kDirection => CSSSampleId::kDirection,
        CSSPropertyID::kFontFamily => CSSSampleId::kFontFamily,
        CSSPropertyID::kFontFeatureSettings => CSSSampleId::kFontFeatureSettings,
        CSSPropertyID::kFontKerning => CSSSampleId::kFontKerning,
        CSSPropertyID::kFontLanguageOverride => CSSSampleId::kFontLanguageOverride,
        CSSPropertyID::kFontOpticalSizing => CSSSampleId::kFontOpticalSizing,
        CSSPropertyID::kFontPalette => CSSSampleId::kFontPalette,
        CSSPropertyID::kFontSize => CSSSampleId::kFontSize,
        CSSPropertyID::kFontSizeAdjust => CSSSampleId::kFontSizeAdjust,
        CSSPropertyID::kFontStretch => CSSSampleId::kFontStretch,
        CSSPropertyID::kFontStyle => CSSSampleId::kFontStyle,
        CSSPropertyID::kFontSynthesisSmallCaps => CSSSampleId::kFontSynthesisSmallCaps,
        CSSPropertyID::kFontSynthesisStyle => CSSSampleId::kFontSynthesisStyle,
        CSSPropertyID::kFontSynthesisWeight => CSSSampleId::kFontSynthesisWeight,
        CSSPropertyID::kFontVariantAlternates => CSSSampleId::kFontVariantAlternates,
        CSSPropertyID::kFontVariantCaps => CSSSampleId::kFontVariantCaps,
        CSSPropertyID::kFontVariantEastAsian => CSSSampleId::kFontVariantEastAsian,
        CSSPropertyID::kFontVariantEmoji => CSSSampleId::kFontVariantEmoji,
        CSSPropertyID::kFontVariantLigatures => CSSSampleId::kFontVariantLigatures,
        CSSPropertyID::kFontVariantNumeric => CSSSampleId::kFontVariantNumeric,
        CSSPropertyID::kFontVariantPosition => CSSSampleId::kFontVariantPosition,
        CSSPropertyID::kFontVariationSettings => CSSSampleId::kFontVariationSettings,
        CSSPropertyID::kFontWeight => CSSSampleId::kFontWeight,
        CSSPropertyID::kPositionArea => CSSSampleId::kPositionArea,
        CSSPropertyID::kTextOrientation => CSSSampleId::kTextOrientation,
        CSSPropertyID::kTextRendering => CSSSampleId::kTextRendering,
        CSSPropertyID::kTextSpacingTrim => CSSSampleId::kTextSpacingTrim,
        CSSPropertyID::kWebkitFontSmoothing => CSSSampleId::kWebkitFontSmoothing,
        CSSPropertyID::kWebkitLocale => CSSSampleId::kWebkitLocale,
        CSSPropertyID::kWebkitTextOrientation => CSSSampleId::kWebkitTextOrientation,
        CSSPropertyID::kWebkitWritingMode => CSSSampleId::kWebkitWritingMode,
        CSSPropertyID::kWritingMode => CSSSampleId::kWritingMode,
        CSSPropertyID::kZoom => CSSSampleId::kZoom,
        CSSPropertyID::kInternalForcedVisitedColor => CSSSampleId::kInternalForcedVisitedColor,
        CSSPropertyID::kInternalVisitedBackgroundColor => {
            CSSSampleId::kInternalVisitedBackgroundColor
        }
        CSSPropertyID::kInternalVisitedBorderBlockEndColor => {
            CSSSampleId::kInternalVisitedBorderBlockEndColor
        }
        CSSPropertyID::kInternalVisitedBorderBlockStartColor => {
            CSSSampleId::kInternalVisitedBorderBlockStartColor
        }
        CSSPropertyID::kInternalVisitedBorderBottomColor => {
            CSSSampleId::kInternalVisitedBorderBottomColor
        }
        CSSPropertyID::kInternalVisitedBorderInlineEndColor => {
            CSSSampleId::kInternalVisitedBorderInlineEndColor
        }
        CSSPropertyID::kInternalVisitedBorderInlineStartColor => {
            CSSSampleId::kInternalVisitedBorderInlineStartColor
        }
        CSSPropertyID::kInternalVisitedBorderLeftColor => {
            CSSSampleId::kInternalVisitedBorderLeftColor
        }
        CSSPropertyID::kInternalVisitedBorderRightColor => {
            CSSSampleId::kInternalVisitedBorderRightColor
        }
        CSSPropertyID::kInternalVisitedBorderTopColor => {
            CSSSampleId::kInternalVisitedBorderTopColor
        }
        CSSPropertyID::kInternalVisitedCaretColor => CSSSampleId::kInternalVisitedCaretColor,
        CSSPropertyID::kInternalVisitedColumnRuleColor => {
            CSSSampleId::kInternalVisitedColumnRuleColor
        }
        CSSPropertyID::kInternalVisitedFill => CSSSampleId::kInternalVisitedFill,
        CSSPropertyID::kInternalVisitedOutlineColor => CSSSampleId::kInternalVisitedOutlineColor,
        CSSPropertyID::kInternalVisitedStroke => CSSSampleId::kInternalVisitedStroke,
        CSSPropertyID::kInternalVisitedTextDecorationColor => {
            CSSSampleId::kInternalVisitedTextDecorationColor
        }
        CSSPropertyID::kInternalVisitedTextEmphasisColor => {
            CSSSampleId::kInternalVisitedTextEmphasisColor
        }
        CSSPropertyID::kInternalVisitedTextFillColor => CSSSampleId::kInternalVisitedTextFillColor,
        CSSPropertyID::kInternalVisitedTextStrokeColor => {
            CSSSampleId::kInternalVisitedTextStrokeColor
        }
        CSSPropertyID::kAccentColor => CSSSampleId::kAccentColor,
        CSSPropertyID::kAdditiveSymbols => CSSSampleId::kAdditiveSymbols,
        CSSPropertyID::kAlignContent => CSSSampleId::kAlignContent,
        CSSPropertyID::kAlignItems => CSSSampleId::kAlignItems,
        CSSPropertyID::kAlignSelf => CSSSampleId::kAlignSelf,
        CSSPropertyID::kAlignmentBaseline => CSSSampleId::kAlignmentBaseline,
        CSSPropertyID::kAll => CSSSampleId::kAll,
        CSSPropertyID::kAlternativeWebkitLineClampLonghand => CSSSampleId::kWebkitLineClamp,
        CSSPropertyID::kAnchorName => CSSSampleId::kAnchorName,
        CSSPropertyID::kAnchorScope => CSSSampleId::kAnchorScope,
        CSSPropertyID::kAnimationComposition => CSSSampleId::kAnimationComposition,
        CSSPropertyID::kAnimationDelay => CSSSampleId::kAnimationDelay,
        CSSPropertyID::kAnimationDirection => CSSSampleId::kAnimationDirection,
        CSSPropertyID::kAnimationDuration => CSSSampleId::kAnimationDuration,
        CSSPropertyID::kAnimationFillMode => CSSSampleId::kAnimationFillMode,
        CSSPropertyID::kAnimationIterationCount => CSSSampleId::kAnimationIterationCount,
        CSSPropertyID::kAnimationName => CSSSampleId::kAnimationName,
        CSSPropertyID::kAnimationPlayState => CSSSampleId::kAnimationPlayState,
        CSSPropertyID::kAnimationRangeEnd => CSSSampleId::kAnimationRangeEnd,
        CSSPropertyID::kAnimationRangeStart => CSSSampleId::kAnimationRangeStart,
        CSSPropertyID::kAnimationTimeline => CSSSampleId::kAnimationTimeline,
        CSSPropertyID::kAnimationTimingFunction => CSSSampleId::kAnimationTimingFunction,
        CSSPropertyID::kAnimationTrigger => CSSSampleId::kAnimationTrigger,
        CSSPropertyID::kAppRegion => CSSSampleId::kAppRegion,
        CSSPropertyID::kAscentOverride => CSSSampleId::kAscentOverride,
        CSSPropertyID::kAspectRatio => CSSSampleId::kAspectRatio,
        CSSPropertyID::kBackdropFilter => CSSSampleId::kBackdropFilter,
        CSSPropertyID::kBackfaceVisibility => CSSSampleId::kBackfaceVisibility,
        CSSPropertyID::kBackgroundAttachment => CSSSampleId::kBackgroundAttachment,
        CSSPropertyID::kBackgroundBlendMode => CSSSampleId::kBackgroundBlendMode,
        CSSPropertyID::kBackgroundClip => CSSSampleId::kBackgroundClip,
        CSSPropertyID::kBackgroundColor => CSSSampleId::kBackgroundColor,
        CSSPropertyID::kBackgroundImage => CSSSampleId::kBackgroundImage,
        CSSPropertyID::kBackgroundOrigin => CSSSampleId::kBackgroundOrigin,
        CSSPropertyID::kBackgroundPositionX => CSSSampleId::kBackgroundPositionX,
        CSSPropertyID::kBackgroundPositionY => CSSSampleId::kBackgroundPositionY,
        CSSPropertyID::kBackgroundRepeat => CSSSampleId::kBackgroundRepeat,
        CSSPropertyID::kBackgroundSize => CSSSampleId::kBackgroundSize,
        CSSPropertyID::kBasePalette => CSSSampleId::kBasePalette,
        CSSPropertyID::kBaseUrl => CSSSampleId::kBaseUrl,
        CSSPropertyID::kBaselineShift => CSSSampleId::kBaselineShift,
        CSSPropertyID::kBaselineSource => CSSSampleId::kBaselineSource,
        CSSPropertyID::kBlockEllipsis => CSSSampleId::kBlockEllipsis,
        CSSPropertyID::kBlockSize => CSSSampleId::kBlockSize,
        CSSPropertyID::kBorderBlockEndColor => CSSSampleId::kBorderBlockEndColor,
        CSSPropertyID::kBorderBlockEndStyle => CSSSampleId::kBorderBlockEndStyle,
        CSSPropertyID::kBorderBlockEndWidth => CSSSampleId::kBorderBlockEndWidth,
        CSSPropertyID::kBorderBlockStartColor => CSSSampleId::kBorderBlockStartColor,
        CSSPropertyID::kBorderBlockStartStyle => CSSSampleId::kBorderBlockStartStyle,
        CSSPropertyID::kBorderBlockStartWidth => CSSSampleId::kBorderBlockStartWidth,
        CSSPropertyID::kBorderBottomColor => CSSSampleId::kBorderBottomColor,
        CSSPropertyID::kBorderBottomLeftRadius => CSSSampleId::kBorderBottomLeftRadius,
        CSSPropertyID::kBorderBottomRightRadius => CSSSampleId::kBorderBottomRightRadius,
        CSSPropertyID::kBorderBottomStyle => CSSSampleId::kBorderBottomStyle,
        CSSPropertyID::kBorderBottomWidth => CSSSampleId::kBorderBottomWidth,
        CSSPropertyID::kBorderCollapse => CSSSampleId::kBorderCollapse,
        CSSPropertyID::kBorderEndEndRadius => CSSSampleId::kBorderEndEndRadius,
        CSSPropertyID::kBorderEndStartRadius => CSSSampleId::kBorderEndStartRadius,
        CSSPropertyID::kBorderImageOutset => CSSSampleId::kBorderImageOutset,
        CSSPropertyID::kBorderImageRepeat => CSSSampleId::kBorderImageRepeat,
        CSSPropertyID::kBorderImageSlice => CSSSampleId::kBorderImageSlice,
        CSSPropertyID::kBorderImageSource => CSSSampleId::kBorderImageSource,
        CSSPropertyID::kBorderImageWidth => CSSSampleId::kBorderImageWidth,
        CSSPropertyID::kBorderInlineEndColor => CSSSampleId::kBorderInlineEndColor,
        CSSPropertyID::kBorderInlineEndStyle => CSSSampleId::kBorderInlineEndStyle,
        CSSPropertyID::kBorderInlineEndWidth => CSSSampleId::kBorderInlineEndWidth,
        CSSPropertyID::kBorderInlineStartColor => CSSSampleId::kBorderInlineStartColor,
        CSSPropertyID::kBorderInlineStartStyle => CSSSampleId::kBorderInlineStartStyle,
        CSSPropertyID::kBorderInlineStartWidth => CSSSampleId::kBorderInlineStartWidth,
        CSSPropertyID::kBorderLeftColor => CSSSampleId::kBorderLeftColor,
        CSSPropertyID::kBorderLeftStyle => CSSSampleId::kBorderLeftStyle,
        CSSPropertyID::kBorderLeftWidth => CSSSampleId::kBorderLeftWidth,
        CSSPropertyID::kBorderRightColor => CSSSampleId::kBorderRightColor,
        CSSPropertyID::kBorderRightStyle => CSSSampleId::kBorderRightStyle,
        CSSPropertyID::kBorderRightWidth => CSSSampleId::kBorderRightWidth,
        CSSPropertyID::kBorderShape => CSSSampleId::kBorderShape,
        CSSPropertyID::kBorderStartEndRadius => CSSSampleId::kBorderStartEndRadius,
        CSSPropertyID::kBorderStartStartRadius => CSSSampleId::kBorderStartStartRadius,
        CSSPropertyID::kBorderTopColor => CSSSampleId::kBorderTopColor,
        CSSPropertyID::kBorderTopLeftRadius => CSSSampleId::kBorderTopLeftRadius,
        CSSPropertyID::kBorderTopRightRadius => CSSSampleId::kBorderTopRightRadius,
        CSSPropertyID::kBorderTopStyle => CSSSampleId::kBorderTopStyle,
        CSSPropertyID::kBorderTopWidth => CSSSampleId::kBorderTopWidth,
        CSSPropertyID::kBottom => CSSSampleId::kBottom,
        CSSPropertyID::kBoxDecorationBreak => CSSSampleId::kBoxDecorationBreak,
        CSSPropertyID::kBoxShadow => CSSSampleId::kBoxShadow,
        CSSPropertyID::kBoxSizing => CSSSampleId::kBoxSizing,
        CSSPropertyID::kBreakAfter => CSSSampleId::kBreakAfter,
        CSSPropertyID::kBreakBefore => CSSSampleId::kBreakBefore,
        CSSPropertyID::kBreakInside => CSSSampleId::kBreakInside,
        CSSPropertyID::kBufferedRendering => CSSSampleId::kBufferedRendering,
        CSSPropertyID::kCaptionSide => CSSSampleId::kCaptionSide,
        CSSPropertyID::kCaretAnimation => CSSSampleId::kCaretAnimation,
        CSSPropertyID::kCaretColor => CSSSampleId::kCaretColor,
        CSSPropertyID::kCaretShape => CSSSampleId::kCaretShape,
        CSSPropertyID::kClear => CSSSampleId::kClear,
        CSSPropertyID::kClip => CSSSampleId::kClip,
        CSSPropertyID::kClipPath => CSSSampleId::kClipPath,
        CSSPropertyID::kClipRule => CSSSampleId::kClipRule,
        CSSPropertyID::kColorInterpolation => CSSSampleId::kColorInterpolation,
        CSSPropertyID::kColorInterpolationFilters => CSSSampleId::kColorInterpolationFilters,
        CSSPropertyID::kColorRendering => CSSSampleId::kColorRendering,
        CSSPropertyID::kColumnCount => CSSSampleId::kColumnCount,
        CSSPropertyID::kColumnFill => CSSSampleId::kColumnFill,
        CSSPropertyID::kColumnGap => CSSSampleId::kColumnGap,
        CSSPropertyID::kColumnHeight => CSSSampleId::kColumnHeight,
        CSSPropertyID::kColumnRuleBreak => CSSSampleId::kColumnRuleBreak,
        CSSPropertyID::kColumnRuleColor => CSSSampleId::kColumnRuleColor,
        CSSPropertyID::kColumnRuleInsetCapEnd => CSSSampleId::kColumnRuleInsetCapEnd,
        CSSPropertyID::kColumnRuleInsetCapStart => CSSSampleId::kColumnRuleInsetCapStart,
        CSSPropertyID::kColumnRuleInsetJunctionEnd => CSSSampleId::kColumnRuleInsetJunctionEnd,
        CSSPropertyID::kColumnRuleInsetJunctionStart => CSSSampleId::kColumnRuleInsetJunctionStart,
        CSSPropertyID::kColumnRuleStyle => CSSSampleId::kColumnRuleStyle,
        CSSPropertyID::kColumnRuleVisibilityItems => CSSSampleId::kColumnRuleVisibilityItems,
        CSSPropertyID::kColumnRuleWidth => CSSSampleId::kColumnRuleWidth,
        CSSPropertyID::kColumnSpan => CSSSampleId::kColumnSpan,
        CSSPropertyID::kColumnWidth => CSSSampleId::kColumnWidth,
        CSSPropertyID::kColumnWrap => CSSSampleId::kColumnWrap,
        CSSPropertyID::kContain => CSSSampleId::kContain,
        CSSPropertyID::kContainIntrinsicBlockSize => CSSSampleId::kContainIntrinsicBlockSize,
        CSSPropertyID::kContainIntrinsicHeight => CSSSampleId::kContainIntrinsicHeight,
        CSSPropertyID::kContainIntrinsicInlineSize => CSSSampleId::kContainIntrinsicInlineSize,
        CSSPropertyID::kContainIntrinsicWidth => CSSSampleId::kContainIntrinsicWidth,
        CSSPropertyID::kContainerName => CSSSampleId::kContainerName,
        CSSPropertyID::kContainerType => CSSSampleId::kContainerType,
        CSSPropertyID::kContent => CSSSampleId::kContent,
        CSSPropertyID::kContentVisibility => CSSSampleId::kContentVisibility,
        CSSPropertyID::kContinue => CSSSampleId::kContinue,
        CSSPropertyID::kCornerBottomLeftShape => CSSSampleId::kCornerBottomLeftShape,
        CSSPropertyID::kCornerBottomRightShape => CSSSampleId::kCornerBottomRightShape,
        CSSPropertyID::kCornerEndEndShape => CSSSampleId::kCornerEndEndShape,
        CSSPropertyID::kCornerEndStartShape => CSSSampleId::kCornerEndStartShape,
        CSSPropertyID::kCornerStartEndShape => CSSSampleId::kCornerStartEndShape,
        CSSPropertyID::kCornerStartStartShape => CSSSampleId::kCornerStartStartShape,
        CSSPropertyID::kCornerTopLeftShape => CSSSampleId::kCornerTopLeftShape,
        CSSPropertyID::kCornerTopRightShape => CSSSampleId::kCornerTopRightShape,
        CSSPropertyID::kCounterIncrement => CSSSampleId::kCounterIncrement,
        CSSPropertyID::kCounterReset => CSSSampleId::kCounterReset,
        CSSPropertyID::kCounterSet => CSSSampleId::kCounterSet,
        CSSPropertyID::kCursor => CSSSampleId::kCursor,
        CSSPropertyID::kCx => CSSSampleId::kCx,
        CSSPropertyID::kCy => CSSSampleId::kCy,
        CSSPropertyID::kD => CSSSampleId::kD,
        CSSPropertyID::kDescentOverride => CSSSampleId::kDescentOverride,
        CSSPropertyID::kDisplay => CSSSampleId::kDisplay,
        CSSPropertyID::kDominantBaseline => CSSSampleId::kDominantBaseline,
        CSSPropertyID::kDynamicRangeLimit => CSSSampleId::kDynamicRangeLimit,
        CSSPropertyID::kEmptyCells => CSSSampleId::kEmptyCells,
        CSSPropertyID::kFallback => CSSSampleId::kFallback,
        CSSPropertyID::kFieldSizing => CSSSampleId::kFieldSizing,
        CSSPropertyID::kFill => CSSSampleId::kFill,
        CSSPropertyID::kFillOpacity => CSSSampleId::kFillOpacity,
        CSSPropertyID::kFillRule => CSSSampleId::kFillRule,
        CSSPropertyID::kFilter => CSSSampleId::kFilter,
        CSSPropertyID::kFlexBasis => CSSSampleId::kFlexBasis,
        CSSPropertyID::kFlexDirection => CSSSampleId::kFlexDirection,
        CSSPropertyID::kFlexGrow => CSSSampleId::kFlexGrow,
        CSSPropertyID::kFlexLineCount => CSSSampleId::kFlexLineCount,
        CSSPropertyID::kFlexShrink => CSSSampleId::kFlexShrink,
        CSSPropertyID::kFlexWrap => CSSSampleId::kFlexWrap,
        CSSPropertyID::kFloat => CSSSampleId::kFloat,
        CSSPropertyID::kFloodColor => CSSSampleId::kFloodColor,
        CSSPropertyID::kFloodOpacity => CSSSampleId::kFloodOpacity,
        CSSPropertyID::kFlowTolerance => CSSSampleId::kFlowTolerance,
        CSSPropertyID::kFontDisplay => CSSSampleId::kFontDisplay,
        CSSPropertyID::kFrameSizing => CSSSampleId::kFrameSizing,
        CSSPropertyID::kGridAutoColumns => CSSSampleId::kGridAutoColumns,
        CSSPropertyID::kGridAutoFlow => CSSSampleId::kGridAutoFlow,
        CSSPropertyID::kGridAutoRows => CSSSampleId::kGridAutoRows,
        CSSPropertyID::kGridColumnEnd => CSSSampleId::kGridColumnEnd,
        CSSPropertyID::kGridColumnStart => CSSSampleId::kGridColumnStart,
        CSSPropertyID::kGridLanesDirection => CSSSampleId::kGridLanesDirection,
        CSSPropertyID::kGridLanesPack => CSSSampleId::kGridLanesPack,
        CSSPropertyID::kGridRowEnd => CSSSampleId::kGridRowEnd,
        CSSPropertyID::kGridRowStart => CSSSampleId::kGridRowStart,
        CSSPropertyID::kGridTemplateAreas => CSSSampleId::kGridTemplateAreas,
        CSSPropertyID::kGridTemplateColumns => CSSSampleId::kGridTemplateColumns,
        CSSPropertyID::kGridTemplateRows => CSSSampleId::kGridTemplateRows,
        CSSPropertyID::kHangingPunctuation => CSSSampleId::kHangingPunctuation,
        CSSPropertyID::kHash => CSSSampleId::kHash,
        CSSPropertyID::kHeight => CSSSampleId::kHeight,
        CSSPropertyID::kHostname => CSSSampleId::kHostname,
        CSSPropertyID::kHyphenateCharacter => CSSSampleId::kHyphenateCharacter,
        CSSPropertyID::kHyphenateLimitChars => CSSSampleId::kHyphenateLimitChars,
        CSSPropertyID::kHyphens => CSSSampleId::kHyphens,
        CSSPropertyID::kImageAnimation => CSSSampleId::kImageAnimation,
        CSSPropertyID::kImageOrientation => CSSSampleId::kImageOrientation,
        CSSPropertyID::kImageRendering => CSSSampleId::kImageRendering,
        CSSPropertyID::kInherits => CSSSampleId::kInherits,
        CSSPropertyID::kInitialLetter => CSSSampleId::kInitialLetter,
        CSSPropertyID::kInitialValue => CSSSampleId::kInitialValue,
        CSSPropertyID::kInlineSize => CSSSampleId::kInlineSize,
        CSSPropertyID::kInsetBlockEnd => CSSSampleId::kInsetBlockEnd,
        CSSPropertyID::kInsetBlockStart => CSSSampleId::kInsetBlockStart,
        CSSPropertyID::kInsetInlineEnd => CSSSampleId::kInsetInlineEnd,
        CSSPropertyID::kInsetInlineStart => CSSSampleId::kInsetInlineStart,
        CSSPropertyID::kInteractivity => CSSSampleId::kInteractivity,
        CSSPropertyID::kInterestDelayEnd => CSSSampleId::kInterestDelayEnd,
        CSSPropertyID::kInterestDelayStart => CSSSampleId::kInterestDelayStart,
        CSSPropertyID::kInternalAlignContentBlock => CSSSampleId::kInternalAlignContentBlock,
        CSSPropertyID::kInternalEmptyLineHeight => CSSSampleId::kInternalEmptyLineHeight,
        CSSPropertyID::kInternalFontSizeDelta => CSSSampleId::kInternalFontSizeDelta,
        CSSPropertyID::kInternalForcedBackgroundColor => {
            CSSSampleId::kInternalForcedBackgroundColor
        }
        CSSPropertyID::kInternalForcedBorderColor => CSSSampleId::kInternalForcedBorderColor,
        CSSPropertyID::kInternalForcedColor => CSSSampleId::kInternalForcedColor,
        CSSPropertyID::kInternalForcedOutlineColor => CSSSampleId::kInternalForcedOutlineColor,
        CSSPropertyID::kInternalOverscrollContainer => CSSSampleId::kInternalOverscrollContainer,
        CSSPropertyID::kInternalOverscrollPosition => CSSSampleId::kInternalOverscrollPosition,
        CSSPropertyID::kInternalUnbounded => CSSSampleId::kInternalUnbounded,
        CSSPropertyID::kInterpolateSize => CSSSampleId::kInterpolateSize,
        CSSPropertyID::kIsolation => CSSSampleId::kIsolation,
        CSSPropertyID::kJustifyContent => CSSSampleId::kJustifyContent,
        CSSPropertyID::kJustifyItems => CSSSampleId::kJustifyItems,
        CSSPropertyID::kJustifySelf => CSSSampleId::kJustifySelf,
        CSSPropertyID::kLeft => CSSSampleId::kLeft,
        CSSPropertyID::kLetterSpacing => CSSSampleId::kLetterSpacing,
        CSSPropertyID::kLightingColor => CSSSampleId::kLightingColor,
        CSSPropertyID::kLineBreak => CSSSampleId::kLineBreak,
        CSSPropertyID::kLineClamp => CSSSampleId::kLineClamp,
        CSSPropertyID::kLineGapOverride => CSSSampleId::kLineGapOverride,
        CSSPropertyID::kLineHeight => CSSSampleId::kLineHeight,
        CSSPropertyID::kListStyleImage => CSSSampleId::kListStyleImage,
        CSSPropertyID::kListStylePosition => CSSSampleId::kListStylePosition,
        CSSPropertyID::kListStyleType => CSSSampleId::kListStyleType,
        CSSPropertyID::kMarginBlockEnd => CSSSampleId::kMarginBlockEnd,
        CSSPropertyID::kMarginBlockStart => CSSSampleId::kMarginBlockStart,
        CSSPropertyID::kMarginBottom => CSSSampleId::kMarginBottom,
        CSSPropertyID::kMarginInlineEnd => CSSSampleId::kMarginInlineEnd,
        CSSPropertyID::kMarginInlineStart => CSSSampleId::kMarginInlineStart,
        CSSPropertyID::kMarginLeft => CSSSampleId::kMarginLeft,
        CSSPropertyID::kMarginRight => CSSSampleId::kMarginRight,
        CSSPropertyID::kMarginTop => CSSSampleId::kMarginTop,
        CSSPropertyID::kMarginTrim => CSSSampleId::kMarginTrim,
        CSSPropertyID::kMarkerEnd => CSSSampleId::kMarkerEnd,
        CSSPropertyID::kMarkerMid => CSSSampleId::kMarkerMid,
        CSSPropertyID::kMarkerStart => CSSSampleId::kMarkerStart,
        CSSPropertyID::kMaskClip => CSSSampleId::kMaskClip,
        CSSPropertyID::kMaskImage => CSSSampleId::kMaskImage,
        CSSPropertyID::kMaskMode => CSSSampleId::kMaskMode,
        CSSPropertyID::kMaskOrigin => CSSSampleId::kMaskOrigin,
        CSSPropertyID::kMaskRepeat => CSSSampleId::kMaskRepeat,
        CSSPropertyID::kMaskSize => CSSSampleId::kMaskSize,
        CSSPropertyID::kMaskType => CSSSampleId::kMaskType,
        CSSPropertyID::kMathShift => CSSSampleId::kMathShift,
        CSSPropertyID::kMathStyle => CSSSampleId::kMathStyle,
        CSSPropertyID::kMaxBlockSize => CSSSampleId::kMaxBlockSize,
        CSSPropertyID::kMaxContentSizing => CSSSampleId::kMaxContentSizing,
        CSSPropertyID::kMaxHeight => CSSSampleId::kMaxHeight,
        CSSPropertyID::kMaxInlineSize => CSSSampleId::kMaxInlineSize,
        CSSPropertyID::kMaxLines => CSSSampleId::kMaxLines,
        CSSPropertyID::kMaxWidth => CSSSampleId::kMaxWidth,
        CSSPropertyID::kMinBlockSize => CSSSampleId::kMinBlockSize,
        CSSPropertyID::kMinHeight => CSSSampleId::kMinHeight,
        CSSPropertyID::kMinInlineSize => CSSSampleId::kMinInlineSize,
        CSSPropertyID::kMinWidth => CSSSampleId::kMinWidth,
        CSSPropertyID::kMixBlendMode => CSSSampleId::kMixBlendMode,
        CSSPropertyID::kNavigation => CSSSampleId::kNavigation,
        CSSPropertyID::kNegative => CSSSampleId::kNegative,
        CSSPropertyID::kObjectFit => CSSSampleId::kObjectFit,
        CSSPropertyID::kObjectPosition => CSSSampleId::kObjectPosition,
        CSSPropertyID::kObjectViewBox => CSSSampleId::kObjectViewBox,
        CSSPropertyID::kOffsetAnchor => CSSSampleId::kOffsetAnchor,
        CSSPropertyID::kOffsetDistance => CSSSampleId::kOffsetDistance,
        CSSPropertyID::kOffsetPath => CSSSampleId::kOffsetPath,
        CSSPropertyID::kOffsetPosition => CSSSampleId::kOffsetPosition,
        CSSPropertyID::kOffsetRotate => CSSSampleId::kOffsetRotate,
        CSSPropertyID::kOpacity => CSSSampleId::kOpacity,
        CSSPropertyID::kOrder => CSSSampleId::kOrder,
        CSSPropertyID::kOriginTrialTestProperty => CSSSampleId::kOriginTrialTestProperty,
        CSSPropertyID::kOrphans => CSSSampleId::kOrphans,
        CSSPropertyID::kOutlineColor => CSSSampleId::kOutlineColor,
        CSSPropertyID::kOutlineOffset => CSSSampleId::kOutlineOffset,
        CSSPropertyID::kOutlineStyle => CSSSampleId::kOutlineStyle,
        CSSPropertyID::kOutlineWidth => CSSSampleId::kOutlineWidth,
        CSSPropertyID::kOverflowAnchor => CSSSampleId::kOverflowAnchor,
        CSSPropertyID::kOverflowBlock => CSSSampleId::kOverflowBlock,
        CSSPropertyID::kOverflowClipMargin => CSSSampleId::kOverflowClipMargin,
        CSSPropertyID::kOverflowInline => CSSSampleId::kOverflowInline,
        CSSPropertyID::kOverflowWrap => CSSSampleId::kOverflowWrap,
        CSSPropertyID::kOverflowX => CSSSampleId::kOverflowX,
        CSSPropertyID::kOverflowY => CSSSampleId::kOverflowY,
        CSSPropertyID::kOverlay => CSSSampleId::kOverlay,
        CSSPropertyID::kOverrideColors => CSSSampleId::kOverrideColors,
        CSSPropertyID::kOverscrollBehaviorBlock => CSSSampleId::kOverscrollBehaviorBlock,
        CSSPropertyID::kOverscrollBehaviorInline => CSSSampleId::kOverscrollBehaviorInline,
        CSSPropertyID::kOverscrollBehaviorX => CSSSampleId::kOverscrollBehaviorX,
        CSSPropertyID::kOverscrollBehaviorY => CSSSampleId::kOverscrollBehaviorY,
        CSSPropertyID::kOverscrollContainerType => CSSSampleId::kOverscrollContainerType,
        CSSPropertyID::kPad => CSSSampleId::kPad,
        CSSPropertyID::kPaddingBlockEnd => CSSSampleId::kPaddingBlockEnd,
        CSSPropertyID::kPaddingBlockStart => CSSSampleId::kPaddingBlockStart,
        CSSPropertyID::kPaddingBottom => CSSSampleId::kPaddingBottom,
        CSSPropertyID::kPaddingInlineEnd => CSSSampleId::kPaddingInlineEnd,
        CSSPropertyID::kPaddingInlineStart => CSSSampleId::kPaddingInlineStart,
        CSSPropertyID::kPaddingLeft => CSSSampleId::kPaddingLeft,
        CSSPropertyID::kPaddingRight => CSSSampleId::kPaddingRight,
        CSSPropertyID::kPaddingTop => CSSSampleId::kPaddingTop,
        CSSPropertyID::kPage => CSSSampleId::kPage,
        CSSPropertyID::kPageMarginSafety => CSSSampleId::kPageMarginSafety,
        CSSPropertyID::kPageOrientation => CSSSampleId::kPageOrientation,
        CSSPropertyID::kPaintOrder => CSSSampleId::kPaintOrder,
        CSSPropertyID::kPathLength => CSSSampleId::kPathLength,
        CSSPropertyID::kPathname => CSSSampleId::kPathname,
        CSSPropertyID::kPattern => CSSSampleId::kPattern,
        CSSPropertyID::kPerspective => CSSSampleId::kPerspective,
        CSSPropertyID::kPerspectiveOrigin => CSSSampleId::kPerspectiveOrigin,
        CSSPropertyID::kPointerEvents => CSSSampleId::kPointerEvents,
        CSSPropertyID::kPort => CSSSampleId::kPort,
        CSSPropertyID::kPositionTryFallbacks => CSSSampleId::kPositionTryFallbacks,
        CSSPropertyID::kPositionTryOrder => CSSSampleId::kPositionTryOrder,
        CSSPropertyID::kPositionVisibility => CSSSampleId::kPositionVisibility,
        CSSPropertyID::kPrefix => CSSSampleId::kPrefix,
        CSSPropertyID::kPrintColorAdjust => CSSSampleId::kPrintColorAdjust,
        CSSPropertyID::kProtocol => CSSSampleId::kProtocol,
        CSSPropertyID::kQuotes => CSSSampleId::kQuotes,
        CSSPropertyID::kR => CSSSampleId::kR,
        CSSPropertyID::kRange => CSSSampleId::kRange,
        CSSPropertyID::kReadingFlow => CSSSampleId::kReadingFlow,
        CSSPropertyID::kReadingOrder => CSSSampleId::kReadingOrder,
        CSSPropertyID::kResize => CSSSampleId::kResize,
        CSSPropertyID::kResult => CSSSampleId::kResult,
        CSSPropertyID::kRight => CSSSampleId::kRight,
        CSSPropertyID::kRotate => CSSSampleId::kRotate,
        CSSPropertyID::kRowGap => CSSSampleId::kRowGap,
        CSSPropertyID::kRowRuleBreak => CSSSampleId::kRowRuleBreak,
        CSSPropertyID::kRowRuleColor => CSSSampleId::kRowRuleColor,
        CSSPropertyID::kRowRuleInsetCapEnd => CSSSampleId::kRowRuleInsetCapEnd,
        CSSPropertyID::kRowRuleInsetCapStart => CSSSampleId::kRowRuleInsetCapStart,
        CSSPropertyID::kRowRuleInsetJunctionEnd => CSSSampleId::kRowRuleInsetJunctionEnd,
        CSSPropertyID::kRowRuleInsetJunctionStart => CSSSampleId::kRowRuleInsetJunctionStart,
        CSSPropertyID::kRowRuleStyle => CSSSampleId::kRowRuleStyle,
        CSSPropertyID::kRowRuleVisibilityItems => CSSSampleId::kRowRuleVisibilityItems,
        CSSPropertyID::kRowRuleWidth => CSSSampleId::kRowRuleWidth,
        CSSPropertyID::kRubyAlign => CSSSampleId::kRubyAlign,
        CSSPropertyID::kRubyOverhang => CSSSampleId::kRubyOverhang,
        CSSPropertyID::kRubyPosition => CSSSampleId::kRubyPosition,
        CSSPropertyID::kRuleOverlap => CSSSampleId::kRuleOverlap,
        CSSPropertyID::kRx => CSSSampleId::kRx,
        CSSPropertyID::kRy => CSSSampleId::kRy,
        CSSPropertyID::kScale => CSSSampleId::kScale,
        CSSPropertyID::kScrollAxisLock => CSSSampleId::kScrollAxisLock,
        CSSPropertyID::kScrollBehavior => CSSSampleId::kScrollBehavior,
        CSSPropertyID::kScrollInitialTarget => CSSSampleId::kScrollInitialTarget,
        CSSPropertyID::kScrollMarginBlockEnd => CSSSampleId::kScrollMarginBlockEnd,
        CSSPropertyID::kScrollMarginBlockStart => CSSSampleId::kScrollMarginBlockStart,
        CSSPropertyID::kScrollMarginBottom => CSSSampleId::kScrollMarginBottom,
        CSSPropertyID::kScrollMarginInlineEnd => CSSSampleId::kScrollMarginInlineEnd,
        CSSPropertyID::kScrollMarginInlineStart => CSSSampleId::kScrollMarginInlineStart,
        CSSPropertyID::kScrollMarginLeft => CSSSampleId::kScrollMarginLeft,
        CSSPropertyID::kScrollMarginRight => CSSSampleId::kScrollMarginRight,
        CSSPropertyID::kScrollMarginTop => CSSSampleId::kScrollMarginTop,
        CSSPropertyID::kScrollMarkerGroup => CSSSampleId::kScrollMarkerGroup,
        CSSPropertyID::kScrollPaddingBlockEnd => CSSSampleId::kScrollPaddingBlockEnd,
        CSSPropertyID::kScrollPaddingBlockStart => CSSSampleId::kScrollPaddingBlockStart,
        CSSPropertyID::kScrollPaddingBottom => CSSSampleId::kScrollPaddingBottom,
        CSSPropertyID::kScrollPaddingInlineEnd => CSSSampleId::kScrollPaddingInlineEnd,
        CSSPropertyID::kScrollPaddingInlineStart => CSSSampleId::kScrollPaddingInlineStart,
        CSSPropertyID::kScrollPaddingLeft => CSSSampleId::kScrollPaddingLeft,
        CSSPropertyID::kScrollPaddingRight => CSSSampleId::kScrollPaddingRight,
        CSSPropertyID::kScrollPaddingTop => CSSSampleId::kScrollPaddingTop,
        CSSPropertyID::kScrollSnapAlign => CSSSampleId::kScrollSnapAlign,
        CSSPropertyID::kScrollSnapStop => CSSSampleId::kScrollSnapStop,
        CSSPropertyID::kScrollSnapType => CSSSampleId::kScrollSnapType,
        CSSPropertyID::kScrollTargetGroup => CSSSampleId::kScrollTargetGroup,
        CSSPropertyID::kScrollTimelineAxis => CSSSampleId::kScrollTimelineAxis,
        CSSPropertyID::kScrollTimelineName => CSSSampleId::kScrollTimelineName,
        CSSPropertyID::kScrollbarColor => CSSSampleId::kScrollbarColor,
        CSSPropertyID::kScrollbarGutter => CSSSampleId::kScrollbarGutter,
        CSSPropertyID::kScrollbarWidth => CSSSampleId::kScrollbarWidth,
        CSSPropertyID::kSearch => CSSSampleId::kSearch,
        CSSPropertyID::kShapeImageThreshold => CSSSampleId::kShapeImageThreshold,
        CSSPropertyID::kShapeMargin => CSSSampleId::kShapeMargin,
        CSSPropertyID::kShapeOutside => CSSSampleId::kShapeOutside,
        CSSPropertyID::kShapeRendering => CSSSampleId::kShapeRendering,
        CSSPropertyID::kSize => CSSSampleId::kSize,
        CSSPropertyID::kSizeAdjust => CSSSampleId::kSizeAdjust,
        CSSPropertyID::kSpeak => CSSSampleId::kSpeak,
        CSSPropertyID::kSpeakAs => CSSSampleId::kSpeakAs,
        CSSPropertyID::kSrc => CSSSampleId::kSrc,
        CSSPropertyID::kStopColor => CSSSampleId::kStopColor,
        CSSPropertyID::kStopOpacity => CSSSampleId::kStopOpacity,
        CSSPropertyID::kStroke => CSSSampleId::kStroke,
        CSSPropertyID::kStrokeDasharray => CSSSampleId::kStrokeDasharray,
        CSSPropertyID::kStrokeDashoffset => CSSSampleId::kStrokeDashoffset,
        CSSPropertyID::kStrokeLinecap => CSSSampleId::kStrokeLinecap,
        CSSPropertyID::kStrokeLinejoin => CSSSampleId::kStrokeLinejoin,
        CSSPropertyID::kStrokeMiterlimit => CSSSampleId::kStrokeMiterlimit,
        CSSPropertyID::kStrokeOpacity => CSSSampleId::kStrokeOpacity,
        CSSPropertyID::kStrokeWidth => CSSSampleId::kStrokeWidth,
        CSSPropertyID::kSuffix => CSSSampleId::kSuffix,
        CSSPropertyID::kSymbols => CSSSampleId::kSymbols,
        CSSPropertyID::kSyntax => CSSSampleId::kSyntax,
        CSSPropertyID::kSystem => CSSSampleId::kSystem,
        CSSPropertyID::kTabSize => CSSSampleId::kTabSize,
        CSSPropertyID::kTableLayout => CSSSampleId::kTableLayout,
        CSSPropertyID::kTextAlign => CSSSampleId::kTextAlign,
        CSSPropertyID::kTextAlignLast => CSSSampleId::kTextAlignLast,
        CSSPropertyID::kTextAnchor => CSSSampleId::kTextAnchor,
        CSSPropertyID::kTextAutospace => CSSSampleId::kTextAutospace,
        CSSPropertyID::kTextBoxEdge => CSSSampleId::kTextBoxEdge,
        CSSPropertyID::kTextBoxTrim => CSSSampleId::kTextBoxTrim,
        CSSPropertyID::kTextCombineUpright => CSSSampleId::kTextCombineUpright,
        CSSPropertyID::kTextDecorationColor => CSSSampleId::kTextDecorationColor,
        CSSPropertyID::kTextDecorationInset => CSSSampleId::kTextDecorationInset,
        CSSPropertyID::kTextDecorationLine => CSSSampleId::kTextDecorationLine,
        CSSPropertyID::kTextDecorationSkipInk => CSSSampleId::kTextDecorationSkipInk,
        CSSPropertyID::kTextDecorationSkipSpaces => CSSSampleId::kTextDecorationSkipSpaces,
        CSSPropertyID::kTextDecorationStyle => CSSSampleId::kTextDecorationStyle,
        CSSPropertyID::kTextDecorationThickness => CSSSampleId::kTextDecorationThickness,
        CSSPropertyID::kTextEmphasisColor => CSSSampleId::kTextEmphasisColor,
        CSSPropertyID::kTextEmphasisPosition => CSSSampleId::kTextEmphasisPosition,
        CSSPropertyID::kTextEmphasisStyle => CSSSampleId::kTextEmphasisStyle,
        CSSPropertyID::kTextFit => CSSSampleId::kTextFit,
        CSSPropertyID::kTextIndent => CSSSampleId::kTextIndent,
        CSSPropertyID::kTextJustify => CSSSampleId::kTextJustify,
        CSSPropertyID::kTextOverflow => CSSSampleId::kTextOverflow,
        CSSPropertyID::kTextShadow => CSSSampleId::kTextShadow,
        CSSPropertyID::kTextTransform => CSSSampleId::kTextTransform,
        CSSPropertyID::kTextUnderlineOffset => CSSSampleId::kTextUnderlineOffset,
        CSSPropertyID::kTextUnderlinePosition => CSSSampleId::kTextUnderlinePosition,
        CSSPropertyID::kTextWrapMode => CSSSampleId::kTextWrapMode,
        CSSPropertyID::kTextWrapStyle => CSSSampleId::kTextWrapStyle,
        CSSPropertyID::kTimelineScope => CSSSampleId::kTimelineScope,
        CSSPropertyID::kTimelineTriggerActivationRangeEnd => {
            CSSSampleId::kTimelineTriggerActivationRangeEnd
        }
        CSSPropertyID::kTimelineTriggerActivationRangeStart => {
            CSSSampleId::kTimelineTriggerActivationRangeStart
        }
        CSSPropertyID::kTimelineTriggerActiveRangeEnd => {
            CSSSampleId::kTimelineTriggerActiveRangeEnd
        }
        CSSPropertyID::kTimelineTriggerActiveRangeStart => {
            CSSSampleId::kTimelineTriggerActiveRangeStart
        }
        CSSPropertyID::kTimelineTriggerName => CSSSampleId::kTimelineTriggerName,
        CSSPropertyID::kTimelineTriggerSource => CSSSampleId::kTimelineTriggerSource,
        CSSPropertyID::kTop => CSSSampleId::kTop,
        CSSPropertyID::kTouchAction => CSSSampleId::kTouchAction,
        CSSPropertyID::kTransform => CSSSampleId::kTransform,
        CSSPropertyID::kTransformBox => CSSSampleId::kTransformBox,
        CSSPropertyID::kTransformOrigin => CSSSampleId::kTransformOrigin,
        CSSPropertyID::kTransformStyle => CSSSampleId::kTransformStyle,
        CSSPropertyID::kTransitionBehavior => CSSSampleId::kTransitionBehavior,
        CSSPropertyID::kTransitionDelay => CSSSampleId::kTransitionDelay,
        CSSPropertyID::kTransitionDuration => CSSSampleId::kTransitionDuration,
        CSSPropertyID::kTransitionProperty => CSSSampleId::kTransitionProperty,
        CSSPropertyID::kTransitionTimingFunction => CSSSampleId::kTransitionTimingFunction,
        CSSPropertyID::kTranslate => CSSSampleId::kTranslate,
        CSSPropertyID::kTriggerScope => CSSSampleId::kTriggerScope,
        CSSPropertyID::kTypes => CSSSampleId::kTypes,
        CSSPropertyID::kUnicodeBidi => CSSSampleId::kUnicodeBidi,
        CSSPropertyID::kUnicodeRange => CSSSampleId::kUnicodeRange,
        CSSPropertyID::kUserSelect => CSSSampleId::kUserSelect,
        CSSPropertyID::kVectorEffect => CSSSampleId::kVectorEffect,
        CSSPropertyID::kVerticalAlign => CSSSampleId::kVerticalAlign,
        CSSPropertyID::kViewTimelineAxis => CSSSampleId::kViewTimelineAxis,
        CSSPropertyID::kViewTimelineInset => CSSSampleId::kViewTimelineInset,
        CSSPropertyID::kViewTimelineName => CSSSampleId::kViewTimelineName,
        CSSPropertyID::kViewTransitionClass => CSSSampleId::kViewTransitionClass,
        CSSPropertyID::kViewTransitionGroup => CSSSampleId::kViewTransitionGroup,
        CSSPropertyID::kViewTransitionName => CSSSampleId::kViewTransitionName,
        CSSPropertyID::kViewTransitionScope => CSSSampleId::kViewTransitionScope,
        CSSPropertyID::kVisibility => CSSSampleId::kVisibility,
        CSSPropertyID::kWebkitBorderHorizontalSpacing => {
            CSSSampleId::kWebkitBorderHorizontalSpacing
        }
        CSSPropertyID::kWebkitBorderImage => CSSSampleId::kWebkitBorderImage,
        CSSPropertyID::kWebkitBorderVerticalSpacing => CSSSampleId::kWebkitBorderVerticalSpacing,
        CSSPropertyID::kWebkitBoxAlign => CSSSampleId::kWebkitBoxAlign,
        CSSPropertyID::kWebkitBoxDecorationBreak => CSSSampleId::kWebkitBoxDecorationBreak,
        CSSPropertyID::kWebkitBoxDirection => CSSSampleId::kWebkitBoxDirection,
        CSSPropertyID::kWebkitBoxFlex => CSSSampleId::kWebkitBoxFlex,
        CSSPropertyID::kWebkitBoxOrdinalGroup => CSSSampleId::kWebkitBoxOrdinalGroup,
        CSSPropertyID::kWebkitBoxOrient => CSSSampleId::kWebkitBoxOrient,
        CSSPropertyID::kWebkitBoxPack => CSSSampleId::kWebkitBoxPack,
        CSSPropertyID::kWebkitBoxReflect => CSSSampleId::kWebkitBoxReflect,
        CSSPropertyID::kWebkitLineBreak => CSSSampleId::kWebkitLineBreak,
        CSSPropertyID::kWebkitLineClamp => CSSSampleId::kWebkitLineClamp,
        CSSPropertyID::kWebkitMaskBoxImageOutset => CSSSampleId::kWebkitMaskBoxImageOutset,
        CSSPropertyID::kWebkitMaskBoxImageRepeat => CSSSampleId::kWebkitMaskBoxImageRepeat,
        CSSPropertyID::kWebkitMaskBoxImageSlice => CSSSampleId::kWebkitMaskBoxImageSlice,
        CSSPropertyID::kWebkitMaskBoxImageSource => CSSSampleId::kWebkitMaskBoxImageSource,
        CSSPropertyID::kWebkitMaskBoxImageWidth => CSSSampleId::kWebkitMaskBoxImageWidth,
        CSSPropertyID::kWebkitMaskPositionX => CSSSampleId::kWebkitMaskPositionX,
        CSSPropertyID::kWebkitMaskPositionY => CSSSampleId::kWebkitMaskPositionY,
        CSSPropertyID::kWebkitPerspectiveOriginX => CSSSampleId::kWebkitPerspectiveOriginX,
        CSSPropertyID::kWebkitPerspectiveOriginY => CSSSampleId::kWebkitPerspectiveOriginY,
        CSSPropertyID::kWebkitRtlOrdering => CSSSampleId::kWebkitRtlOrdering,
        CSSPropertyID::kWebkitRubyPosition => CSSSampleId::kWebkitRubyPosition,
        CSSPropertyID::kWebkitTapHighlightColor => CSSSampleId::kWebkitTapHighlightColor,
        CSSPropertyID::kWebkitTextCombine => CSSSampleId::kWebkitTextCombine,
        CSSPropertyID::kWebkitTextDecorationsInEffect => {
            CSSSampleId::kWebkitTextDecorationsInEffect
        }
        CSSPropertyID::kWebkitTextFillColor => CSSSampleId::kWebkitTextFillColor,
        CSSPropertyID::kWebkitTextSecurity => CSSSampleId::kWebkitTextSecurity,
        CSSPropertyID::kWebkitTextStrokeColor => CSSSampleId::kWebkitTextStrokeColor,
        CSSPropertyID::kWebkitTextStrokeWidth => CSSSampleId::kWebkitTextStrokeWidth,
        CSSPropertyID::kWebkitTransformOriginX => CSSSampleId::kWebkitTransformOriginX,
        CSSPropertyID::kWebkitTransformOriginY => CSSSampleId::kWebkitTransformOriginY,
        CSSPropertyID::kWebkitTransformOriginZ => CSSSampleId::kWebkitTransformOriginZ,
        CSSPropertyID::kWebkitUserDrag => CSSSampleId::kWebkitUserDrag,
        CSSPropertyID::kWebkitUserModify => CSSSampleId::kWebkitUserModify,
        CSSPropertyID::kWhiteSpaceCollapse => CSSSampleId::kWhiteSpaceCollapse,
        CSSPropertyID::kWidows => CSSSampleId::kWidows,
        CSSPropertyID::kWidth => CSSSampleId::kWidth,
        CSSPropertyID::kWillChange => CSSSampleId::kWillChange,
        CSSPropertyID::kWindowDrag => CSSSampleId::kWindowDrag,
        CSSPropertyID::kWordBreak => CSSSampleId::kWordBreak,
        CSSPropertyID::kWordSpacing => CSSSampleId::kWordSpacing,
        CSSPropertyID::kX => CSSSampleId::kX,
        CSSPropertyID::kY => CSSSampleId::kY,
        CSSPropertyID::kZIndex => CSSSampleId::kZIndex,
        CSSPropertyID::kMaskComposite => CSSSampleId::kMaskComposite,
        CSSPropertyID::kAlternativeLineClampShorthand => CSSSampleId::kLineClamp,
        CSSPropertyID::kAlternativeWebkitLineClampShorthand => CSSSampleId::kWebkitLineClamp,
        CSSPropertyID::kAnimation => CSSSampleId::kAnimation,
        CSSPropertyID::kAnimationRange => CSSSampleId::kAnimationRange,
        CSSPropertyID::kBackground => CSSSampleId::kBackground,
        CSSPropertyID::kBackgroundPosition => CSSSampleId::kBackgroundPosition,
        CSSPropertyID::kBorder => CSSSampleId::kBorder,
        CSSPropertyID::kBorderBlock => CSSSampleId::kBorderBlock,
        CSSPropertyID::kBorderBlockColor => CSSSampleId::kBorderBlockColor,
        CSSPropertyID::kBorderBlockEnd => CSSSampleId::kBorderBlockEnd,
        CSSPropertyID::kBorderBlockStart => CSSSampleId::kBorderBlockStart,
        CSSPropertyID::kBorderBlockStyle => CSSSampleId::kBorderBlockStyle,
        CSSPropertyID::kBorderBlockWidth => CSSSampleId::kBorderBlockWidth,
        CSSPropertyID::kBorderBottom => CSSSampleId::kBorderBottom,
        CSSPropertyID::kBorderColor => CSSSampleId::kBorderColor,
        CSSPropertyID::kBorderImage => CSSSampleId::kBorderImage,
        CSSPropertyID::kBorderInline => CSSSampleId::kBorderInline,
        CSSPropertyID::kBorderInlineColor => CSSSampleId::kBorderInlineColor,
        CSSPropertyID::kBorderInlineEnd => CSSSampleId::kBorderInlineEnd,
        CSSPropertyID::kBorderInlineStart => CSSSampleId::kBorderInlineStart,
        CSSPropertyID::kBorderInlineStyle => CSSSampleId::kBorderInlineStyle,
        CSSPropertyID::kBorderInlineWidth => CSSSampleId::kBorderInlineWidth,
        CSSPropertyID::kBorderLeft => CSSSampleId::kBorderLeft,
        CSSPropertyID::kBorderRadius => CSSSampleId::kBorderRadius,
        CSSPropertyID::kBorderRight => CSSSampleId::kBorderRight,
        CSSPropertyID::kBorderSpacing => CSSSampleId::kBorderSpacing,
        CSSPropertyID::kBorderStyle => CSSSampleId::kBorderStyle,
        CSSPropertyID::kBorderTop => CSSSampleId::kBorderTop,
        CSSPropertyID::kBorderWidth => CSSSampleId::kBorderWidth,
        CSSPropertyID::kColumnRule => CSSSampleId::kColumnRule,
        CSSPropertyID::kColumnRuleInset => CSSSampleId::kColumnRuleInset,
        CSSPropertyID::kColumnRuleInsetCap => CSSSampleId::kColumnRuleInsetCap,
        CSSPropertyID::kColumnRuleInsetEnd => CSSSampleId::kColumnRuleInsetEnd,
        CSSPropertyID::kColumnRuleInsetJunction => CSSSampleId::kColumnRuleInsetJunction,
        CSSPropertyID::kColumnRuleInsetStart => CSSSampleId::kColumnRuleInsetStart,
        CSSPropertyID::kColumns => CSSSampleId::kColumns,
        CSSPropertyID::kContainIntrinsicSize => CSSSampleId::kContainIntrinsicSize,
        CSSPropertyID::kContainer => CSSSampleId::kContainer,
        CSSPropertyID::kCorner => CSSSampleId::kCorner,
        CSSPropertyID::kCornerBlockEnd => CSSSampleId::kCornerBlockEnd,
        CSSPropertyID::kCornerBlockEndShape => CSSSampleId::kCornerBlockEndShape,
        CSSPropertyID::kCornerBlockStart => CSSSampleId::kCornerBlockStart,
        CSSPropertyID::kCornerBlockStartShape => CSSSampleId::kCornerBlockStartShape,
        CSSPropertyID::kCornerBottom => CSSSampleId::kCornerBottom,
        CSSPropertyID::kCornerBottomLeft => CSSSampleId::kCornerBottomLeft,
        CSSPropertyID::kCornerBottomRight => CSSSampleId::kCornerBottomRight,
        CSSPropertyID::kCornerBottomShape => CSSSampleId::kCornerBottomShape,
        CSSPropertyID::kCornerEndEnd => CSSSampleId::kCornerEndEnd,
        CSSPropertyID::kCornerEndStart => CSSSampleId::kCornerEndStart,
        CSSPropertyID::kCornerInlineEnd => CSSSampleId::kCornerInlineEnd,
        CSSPropertyID::kCornerInlineEndShape => CSSSampleId::kCornerInlineEndShape,
        CSSPropertyID::kCornerInlineStart => CSSSampleId::kCornerInlineStart,
        CSSPropertyID::kCornerInlineStartShape => CSSSampleId::kCornerInlineStartShape,
        CSSPropertyID::kCornerLeft => CSSSampleId::kCornerLeft,
        CSSPropertyID::kCornerLeftShape => CSSSampleId::kCornerLeftShape,
        CSSPropertyID::kCornerRight => CSSSampleId::kCornerRight,
        CSSPropertyID::kCornerRightShape => CSSSampleId::kCornerRightShape,
        CSSPropertyID::kCornerShape => CSSSampleId::kCornerShape,
        CSSPropertyID::kCornerStartEnd => CSSSampleId::kCornerStartEnd,
        CSSPropertyID::kCornerStartStart => CSSSampleId::kCornerStartStart,
        CSSPropertyID::kCornerTop => CSSSampleId::kCornerTop,
        CSSPropertyID::kCornerTopLeft => CSSSampleId::kCornerTopLeft,
        CSSPropertyID::kCornerTopRight => CSSSampleId::kCornerTopRight,
        CSSPropertyID::kCornerTopShape => CSSSampleId::kCornerTopShape,
        CSSPropertyID::kFlex => CSSSampleId::kFlex,
        CSSPropertyID::kFlexFlow => CSSSampleId::kFlexFlow,
        CSSPropertyID::kFont => CSSSampleId::kFont,
        CSSPropertyID::kFontSynthesis => CSSSampleId::kFontSynthesis,
        CSSPropertyID::kFontVariant => CSSSampleId::kFontVariant,
        CSSPropertyID::kGap => CSSSampleId::kGap,
        CSSPropertyID::kGrid => CSSSampleId::kGrid,
        CSSPropertyID::kGridArea => CSSSampleId::kGridArea,
        CSSPropertyID::kGridColumn => CSSSampleId::kGridColumn,
        CSSPropertyID::kGridLanes => CSSSampleId::kGridLanes,
        CSSPropertyID::kGridRow => CSSSampleId::kGridRow,
        CSSPropertyID::kGridTemplate => CSSSampleId::kGridTemplate,
        CSSPropertyID::kInset => CSSSampleId::kInset,
        CSSPropertyID::kInsetBlock => CSSSampleId::kInsetBlock,
        CSSPropertyID::kInsetInline => CSSSampleId::kInsetInline,
        CSSPropertyID::kInterestDelay => CSSSampleId::kInterestDelay,
        CSSPropertyID::kListStyle => CSSSampleId::kListStyle,
        CSSPropertyID::kMargin => CSSSampleId::kMargin,
        CSSPropertyID::kMarginBlock => CSSSampleId::kMarginBlock,
        CSSPropertyID::kMarginInline => CSSSampleId::kMarginInline,
        CSSPropertyID::kMarker => CSSSampleId::kMarker,
        CSSPropertyID::kMask => CSSSampleId::kMask,
        CSSPropertyID::kMaskPosition => CSSSampleId::kMaskPosition,
        CSSPropertyID::kOffset => CSSSampleId::kOffset,
        CSSPropertyID::kOutline => CSSSampleId::kOutline,
        CSSPropertyID::kOverflow => CSSSampleId::kOverflow,
        CSSPropertyID::kOverscrollBehavior => CSSSampleId::kOverscrollBehavior,
        CSSPropertyID::kPadding => CSSSampleId::kPadding,
        CSSPropertyID::kPaddingBlock => CSSSampleId::kPaddingBlock,
        CSSPropertyID::kPaddingInline => CSSSampleId::kPaddingInline,
        CSSPropertyID::kPageBreakAfter => CSSSampleId::kPageBreakAfter,
        CSSPropertyID::kPageBreakBefore => CSSSampleId::kPageBreakBefore,
        CSSPropertyID::kPageBreakInside => CSSSampleId::kPageBreakInside,
        CSSPropertyID::kPlaceContent => CSSSampleId::kPlaceContent,
        CSSPropertyID::kPlaceItems => CSSSampleId::kPlaceItems,
        CSSPropertyID::kPlaceSelf => CSSSampleId::kPlaceSelf,
        CSSPropertyID::kPositionTry => CSSSampleId::kPositionTry,
        CSSPropertyID::kRowRule => CSSSampleId::kRowRule,
        CSSPropertyID::kRowRuleInset => CSSSampleId::kRowRuleInset,
        CSSPropertyID::kRowRuleInsetCap => CSSSampleId::kRowRuleInsetCap,
        CSSPropertyID::kRowRuleInsetEnd => CSSSampleId::kRowRuleInsetEnd,
        CSSPropertyID::kRowRuleInsetJunction => CSSSampleId::kRowRuleInsetJunction,
        CSSPropertyID::kRowRuleInsetStart => CSSSampleId::kRowRuleInsetStart,
        CSSPropertyID::kRule => CSSSampleId::kRule,
        CSSPropertyID::kRuleBreak => CSSSampleId::kRuleBreak,
        CSSPropertyID::kRuleColor => CSSSampleId::kRuleColor,
        CSSPropertyID::kRuleInset => CSSSampleId::kRuleInset,
        CSSPropertyID::kRuleInsetCap => CSSSampleId::kRuleInsetCap,
        CSSPropertyID::kRuleInsetEnd => CSSSampleId::kRuleInsetEnd,
        CSSPropertyID::kRuleInsetJunction => CSSSampleId::kRuleInsetJunction,
        CSSPropertyID::kRuleInsetStart => CSSSampleId::kRuleInsetStart,
        CSSPropertyID::kRuleStyle => CSSSampleId::kRuleStyle,
        CSSPropertyID::kRuleVisibilityItems => CSSSampleId::kRuleVisibilityItems,
        CSSPropertyID::kRuleWidth => CSSSampleId::kRuleWidth,
        CSSPropertyID::kScrollMargin => CSSSampleId::kScrollMargin,
        CSSPropertyID::kScrollMarginBlock => CSSSampleId::kScrollMarginBlock,
        CSSPropertyID::kScrollMarginInline => CSSSampleId::kScrollMarginInline,
        CSSPropertyID::kScrollPadding => CSSSampleId::kScrollPadding,
        CSSPropertyID::kScrollPaddingBlock => CSSSampleId::kScrollPaddingBlock,
        CSSPropertyID::kScrollPaddingInline => CSSSampleId::kScrollPaddingInline,
        CSSPropertyID::kScrollTimeline => CSSSampleId::kScrollTimeline,
        CSSPropertyID::kTextBox => CSSSampleId::kTextBox,
        CSSPropertyID::kTextDecoration => CSSSampleId::kTextDecoration,
        CSSPropertyID::kTextEmphasis => CSSSampleId::kTextEmphasis,
        CSSPropertyID::kTextSpacing => CSSSampleId::kTextSpacing,
        CSSPropertyID::kTextWrap => CSSSampleId::kTextWrap,
        CSSPropertyID::kTimelineTrigger => CSSSampleId::kTimelineTrigger,
        CSSPropertyID::kTimelineTriggerActivationRange => {
            CSSSampleId::kTimelineTriggerActivationRange
        }
        CSSPropertyID::kTimelineTriggerActiveRange => CSSSampleId::kTimelineTriggerActiveRange,
        CSSPropertyID::kTransition => CSSSampleId::kTransition,
        CSSPropertyID::kViewTimeline => CSSSampleId::kViewTimeline,
        CSSPropertyID::kWebkitColumnBreakAfter => CSSSampleId::kWebkitColumnBreakAfter,
        CSSPropertyID::kWebkitColumnBreakBefore => CSSSampleId::kWebkitColumnBreakBefore,
        CSSPropertyID::kWebkitColumnBreakInside => CSSSampleId::kWebkitColumnBreakInside,
        CSSPropertyID::kWebkitMaskBoxImage => CSSSampleId::kWebkitMaskBoxImage,
        CSSPropertyID::kWebkitTextStroke => CSSSampleId::kWebkitTextStroke,
        CSSPropertyID::kWhiteSpace => CSSSampleId::kWhiteSpace,
        CSSPropertyID::kAliasWebkitAppearance => CSSSampleId::kAliasWebkitAppearance,
        CSSPropertyID::kAliasWebkitAppRegion => CSSSampleId::kAliasWebkitAppRegion,
        CSSPropertyID::kAliasWebkitMaskClip => CSSSampleId::kAliasWebkitMaskClip,
        CSSPropertyID::kAliasWebkitMaskComposite => CSSSampleId::kAliasWebkitMaskComposite,
        CSSPropertyID::kAliasWebkitMaskImage => CSSSampleId::kAliasWebkitMaskImage,
        CSSPropertyID::kAliasWebkitMaskOrigin => CSSSampleId::kAliasWebkitMaskOrigin,
        CSSPropertyID::kAliasWebkitMaskRepeat => CSSSampleId::kAliasWebkitMaskRepeat,
        CSSPropertyID::kAliasWebkitMaskSize => CSSSampleId::kAliasWebkitMaskSize,
        CSSPropertyID::kAliasWebkitBorderEndColor => CSSSampleId::kAliasWebkitBorderEndColor,
        CSSPropertyID::kAliasWebkitBorderEndStyle => CSSSampleId::kAliasWebkitBorderEndStyle,
        CSSPropertyID::kAliasWebkitBorderEndWidth => CSSSampleId::kAliasWebkitBorderEndWidth,
        CSSPropertyID::kAliasWebkitBorderStartColor => CSSSampleId::kAliasWebkitBorderStartColor,
        CSSPropertyID::kAliasWebkitBorderStartStyle => CSSSampleId::kAliasWebkitBorderStartStyle,
        CSSPropertyID::kAliasWebkitBorderStartWidth => CSSSampleId::kAliasWebkitBorderStartWidth,
        CSSPropertyID::kAliasWebkitBorderBeforeColor => CSSSampleId::kAliasWebkitBorderBeforeColor,
        CSSPropertyID::kAliasWebkitBorderBeforeStyle => CSSSampleId::kAliasWebkitBorderBeforeStyle,
        CSSPropertyID::kAliasWebkitBorderBeforeWidth => CSSSampleId::kAliasWebkitBorderBeforeWidth,
        CSSPropertyID::kAliasWebkitBorderAfterColor => CSSSampleId::kAliasWebkitBorderAfterColor,
        CSSPropertyID::kAliasWebkitBorderAfterStyle => CSSSampleId::kAliasWebkitBorderAfterStyle,
        CSSPropertyID::kAliasWebkitBorderAfterWidth => CSSSampleId::kAliasWebkitBorderAfterWidth,
        CSSPropertyID::kAliasWebkitMarginEnd => CSSSampleId::kAliasWebkitMarginEnd,
        CSSPropertyID::kAliasWebkitMarginStart => CSSSampleId::kAliasWebkitMarginStart,
        CSSPropertyID::kAliasWebkitMarginBefore => CSSSampleId::kAliasWebkitMarginBefore,
        CSSPropertyID::kAliasWebkitMarginAfter => CSSSampleId::kAliasWebkitMarginAfter,
        CSSPropertyID::kAliasWebkitPaddingEnd => CSSSampleId::kAliasWebkitPaddingEnd,
        CSSPropertyID::kAliasWebkitPaddingStart => CSSSampleId::kAliasWebkitPaddingStart,
        CSSPropertyID::kAliasWebkitPaddingBefore => CSSSampleId::kAliasWebkitPaddingBefore,
        CSSPropertyID::kAliasWebkitPaddingAfter => CSSSampleId::kAliasWebkitPaddingAfter,
        CSSPropertyID::kAliasWebkitLogicalWidth => CSSSampleId::kAliasWebkitLogicalWidth,
        CSSPropertyID::kAliasWebkitLogicalHeight => CSSSampleId::kAliasWebkitLogicalHeight,
        CSSPropertyID::kAliasWebkitMinLogicalWidth => CSSSampleId::kAliasWebkitMinLogicalWidth,
        CSSPropertyID::kAliasWebkitMinLogicalHeight => CSSSampleId::kAliasWebkitMinLogicalHeight,
        CSSPropertyID::kAliasWebkitMaxLogicalWidth => CSSSampleId::kAliasWebkitMaxLogicalWidth,
        CSSPropertyID::kAliasWebkitMaxLogicalHeight => CSSSampleId::kAliasWebkitMaxLogicalHeight,
        CSSPropertyID::kAliasWebkitPrintColorAdjust => CSSSampleId::kAliasWebkitPrintColorAdjust,
        CSSPropertyID::kAliasWebkitBorderAfter => CSSSampleId::kAliasWebkitBorderAfter,
        CSSPropertyID::kAliasWebkitBorderBefore => CSSSampleId::kAliasWebkitBorderBefore,
        CSSPropertyID::kAliasWebkitBorderEnd => CSSSampleId::kAliasWebkitBorderEnd,
        CSSPropertyID::kAliasWebkitBorderStart => CSSSampleId::kAliasWebkitBorderStart,
        CSSPropertyID::kAliasWebkitMask => CSSSampleId::kAliasWebkitMask,
        CSSPropertyID::kAliasWebkitMaskPosition => CSSSampleId::kAliasWebkitMaskPosition,
        CSSPropertyID::kAliasEpubCaptionSide => CSSSampleId::kAliasEpubCaptionSide,
        CSSPropertyID::kAliasEpubTextCombine => CSSSampleId::kAliasEpubTextCombine,
        CSSPropertyID::kAliasEpubTextEmphasis => CSSSampleId::kAliasEpubTextEmphasis,
        CSSPropertyID::kAliasEpubTextEmphasisColor => CSSSampleId::kAliasEpubTextEmphasisColor,
        CSSPropertyID::kAliasEpubTextEmphasisStyle => CSSSampleId::kAliasEpubTextEmphasisStyle,
        CSSPropertyID::kAliasEpubTextOrientation => CSSSampleId::kAliasEpubTextOrientation,
        CSSPropertyID::kAliasEpubTextTransform => CSSSampleId::kAliasEpubTextTransform,
        CSSPropertyID::kAliasEpubWordBreak => CSSSampleId::kAliasEpubWordBreak,
        CSSPropertyID::kAliasEpubWritingMode => CSSSampleId::kAliasEpubWritingMode,
        CSSPropertyID::kAliasWebkitAlignContent => CSSSampleId::kAliasWebkitAlignContent,
        CSSPropertyID::kAliasWebkitAlignItems => CSSSampleId::kAliasWebkitAlignItems,
        CSSPropertyID::kAliasWebkitAlignSelf => CSSSampleId::kAliasWebkitAlignSelf,
        CSSPropertyID::kAliasWebkitAnimation => CSSSampleId::kAliasWebkitAnimation,
        CSSPropertyID::kAliasWebkitAnimationDelay => CSSSampleId::kAliasWebkitAnimationDelay,
        CSSPropertyID::kAliasWebkitAnimationDirection => {
            CSSSampleId::kAliasWebkitAnimationDirection
        }
        CSSPropertyID::kAliasWebkitAnimationDuration => CSSSampleId::kAliasWebkitAnimationDuration,
        CSSPropertyID::kAliasWebkitAnimationFillMode => CSSSampleId::kAliasWebkitAnimationFillMode,
        CSSPropertyID::kAliasWebkitAnimationIterationCount => {
            CSSSampleId::kAliasWebkitAnimationIterationCount
        }
        CSSPropertyID::kAliasWebkitAnimationName => CSSSampleId::kAliasWebkitAnimationName,
        CSSPropertyID::kAliasWebkitAnimationPlayState => {
            CSSSampleId::kAliasWebkitAnimationPlayState
        }
        CSSPropertyID::kAliasWebkitAnimationTimingFunction => {
            CSSSampleId::kAliasWebkitAnimationTimingFunction
        }
        CSSPropertyID::kAliasWebkitBackfaceVisibility => {
            CSSSampleId::kAliasWebkitBackfaceVisibility
        }
        CSSPropertyID::kAliasWebkitBackgroundClip => CSSSampleId::kAliasWebkitBackgroundClip,
        CSSPropertyID::kAliasWebkitBackgroundOrigin => CSSSampleId::kAliasWebkitBackgroundOrigin,
        CSSPropertyID::kAliasWebkitBackgroundSize => CSSSampleId::kAliasWebkitBackgroundSize,
        CSSPropertyID::kAliasWebkitBorderBottomLeftRadius => {
            CSSSampleId::kAliasWebkitBorderBottomLeftRadius
        }
        CSSPropertyID::kAliasWebkitBorderBottomRightRadius => {
            CSSSampleId::kAliasWebkitBorderBottomRightRadius
        }
        CSSPropertyID::kAliasWebkitBorderRadius => CSSSampleId::kAliasWebkitBorderRadius,
        CSSPropertyID::kAliasWebkitBorderTopLeftRadius => {
            CSSSampleId::kAliasWebkitBorderTopLeftRadius
        }
        CSSPropertyID::kAliasWebkitBorderTopRightRadius => {
            CSSSampleId::kAliasWebkitBorderTopRightRadius
        }
        CSSPropertyID::kAliasWebkitBoxShadow => CSSSampleId::kAliasWebkitBoxShadow,
        CSSPropertyID::kAliasWebkitBoxSizing => CSSSampleId::kAliasWebkitBoxSizing,
        CSSPropertyID::kAliasWebkitClipPath => CSSSampleId::kAliasWebkitClipPath,
        CSSPropertyID::kAliasWebkitColumnCount => CSSSampleId::kAliasWebkitColumnCount,
        CSSPropertyID::kAliasWebkitColumnGap => CSSSampleId::kAliasWebkitColumnGap,
        CSSPropertyID::kAliasWebkitColumnRule => CSSSampleId::kAliasWebkitColumnRule,
        CSSPropertyID::kAliasWebkitColumnRuleColor => CSSSampleId::kAliasWebkitColumnRuleColor,
        CSSPropertyID::kAliasWebkitColumnRuleStyle => CSSSampleId::kAliasWebkitColumnRuleStyle,
        CSSPropertyID::kAliasWebkitColumnRuleWidth => CSSSampleId::kAliasWebkitColumnRuleWidth,
        CSSPropertyID::kAliasWebkitColumnSpan => CSSSampleId::kAliasWebkitColumnSpan,
        CSSPropertyID::kAliasWebkitColumnWidth => CSSSampleId::kAliasWebkitColumnWidth,
        CSSPropertyID::kAliasWebkitColumns => CSSSampleId::kAliasWebkitColumns,
        CSSPropertyID::kAliasWebkitFilter => CSSSampleId::kAliasWebkitFilter,
        CSSPropertyID::kAliasWebkitFlex => CSSSampleId::kAliasWebkitFlex,
        CSSPropertyID::kAliasWebkitFlexBasis => CSSSampleId::kAliasWebkitFlexBasis,
        CSSPropertyID::kAliasWebkitFlexDirection => CSSSampleId::kAliasWebkitFlexDirection,
        CSSPropertyID::kAliasWebkitFlexFlow => CSSSampleId::kAliasWebkitFlexFlow,
        CSSPropertyID::kAliasWebkitFlexGrow => CSSSampleId::kAliasWebkitFlexGrow,
        CSSPropertyID::kAliasWebkitFlexShrink => CSSSampleId::kAliasWebkitFlexShrink,
        CSSPropertyID::kAliasWebkitFlexWrap => CSSSampleId::kAliasWebkitFlexWrap,
        CSSPropertyID::kAliasWebkitFontFeatureSettings => {
            CSSSampleId::kAliasWebkitFontFeatureSettings
        }
        CSSPropertyID::kAliasWebkitHyphenateCharacter => {
            CSSSampleId::kAliasWebkitHyphenateCharacter
        }
        CSSPropertyID::kAliasWebkitJustifyContent => CSSSampleId::kAliasWebkitJustifyContent,
        CSSPropertyID::kAliasWebkitOpacity => CSSSampleId::kAliasWebkitOpacity,
        CSSPropertyID::kAliasWebkitOrder => CSSSampleId::kAliasWebkitOrder,
        CSSPropertyID::kAliasWebkitPerspective => CSSSampleId::kAliasWebkitPerspective,
        CSSPropertyID::kAliasWebkitPerspectiveOrigin => CSSSampleId::kAliasWebkitPerspectiveOrigin,
        CSSPropertyID::kAliasWebkitShapeImageThreshold => {
            CSSSampleId::kAliasWebkitShapeImageThreshold
        }
        CSSPropertyID::kAliasWebkitShapeMargin => CSSSampleId::kAliasWebkitShapeMargin,
        CSSPropertyID::kAliasWebkitShapeOutside => CSSSampleId::kAliasWebkitShapeOutside,
        CSSPropertyID::kAliasWebkitTextEmphasis => CSSSampleId::kAliasWebkitTextEmphasis,
        CSSPropertyID::kAliasWebkitTextEmphasisColor => CSSSampleId::kAliasWebkitTextEmphasisColor,
        CSSPropertyID::kAliasWebkitTextEmphasisPosition => {
            CSSSampleId::kAliasWebkitTextEmphasisPosition
        }
        CSSPropertyID::kAliasWebkitTextEmphasisStyle => CSSSampleId::kAliasWebkitTextEmphasisStyle,
        CSSPropertyID::kAliasWebkitTextSizeAdjust => CSSSampleId::kAliasWebkitTextSizeAdjust,
        CSSPropertyID::kAliasWebkitTransform => CSSSampleId::kAliasWebkitTransform,
        CSSPropertyID::kAliasWebkitTransformOrigin => CSSSampleId::kAliasWebkitTransformOrigin,
        CSSPropertyID::kAliasWebkitTransformStyle => CSSSampleId::kAliasWebkitTransformStyle,
        CSSPropertyID::kAliasWebkitTransition => CSSSampleId::kAliasWebkitTransition,
        CSSPropertyID::kAliasWebkitTransitionDelay => CSSSampleId::kAliasWebkitTransitionDelay,
        CSSPropertyID::kAliasWebkitTransitionDuration => {
            CSSSampleId::kAliasWebkitTransitionDuration
        }
        CSSPropertyID::kAliasWebkitTransitionProperty => {
            CSSSampleId::kAliasWebkitTransitionProperty
        }
        CSSPropertyID::kAliasWebkitTransitionTimingFunction => {
            CSSSampleId::kAliasWebkitTransitionTimingFunction
        }
        CSSPropertyID::kAliasWebkitUserSelect => CSSSampleId::kAliasWebkitUserSelect,
        CSSPropertyID::kAliasWordWrap => CSSSampleId::kAliasWordWrap,
        CSSPropertyID::kAliasGridColumnGap => CSSSampleId::kAliasGridColumnGap,
        CSSPropertyID::kAliasGridRowGap => CSSSampleId::kAliasGridRowGap,
        CSSPropertyID::kAliasGridGap => CSSSampleId::kAliasGridGap,
        CSSPropertyID::kVariable => CSSSampleId::kVariable,
        CSSPropertyID::kInvalid => CSSSampleId::kInvalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_names_aliases_and_alternatives_remain_distinct() {
        for entry in PROPERTY_WORD_LIST.iter() {
            let found = FindProperty(entry.name.as_bytes()).expect(entry.name);
            assert_eq!(found, entry);
            let id_value = entry.id_and_exposed_bit & !kNotKnownExposedPropertyBit;
            assert!((2..=825).contains(&id_value));
            let id = unsafe { std::mem::transmute::<i32, CSSPropertyID>(id_value) };
            assert_eq!(GetPropertyName(id), entry.name);
            let resolved = ResolveCSSPropertyID(id);
            assert!((2..=706).contains(&(resolved as i32)));
        }
        assert_eq!(
            ResolveCSSPropertyID(CSSPropertyID::kAliasWebkitAppearance),
            CSSPropertyID::kAppearance
        );
        assert_eq!(
            ResolveCSSPropertyID(CSSPropertyID::kAliasWordWrap),
            CSSPropertyID::kOverflowWrap
        );
        assert_eq!(
            GetPropertyName(CSSPropertyID::kAlternativeWebkitLineClampLonghand),
            "-webkit-line-clamp"
        );
        assert_eq!(
            GetJSPropertyName(CSSPropertyID::kBackgroundColor),
            "backgroundColor"
        );
        assert_eq!(GetPropertyName(CSSPropertyID::kVariable), "variable");
        assert_eq!(
            GetCSSSampleId(CSSPropertyID::kInternalVisitedColor),
            CSSSampleId::kInvalid
        );
        assert_ne!(
            GetCSSSampleId(CSSPropertyID::kAliasWebkitAppearance),
            GetCSSSampleId(CSSPropertyID::kAppearance)
        );
        assert!(std::ptr::eq(
            GetPropertyNameAtomicString(CSSPropertyID::kColor),
            GetPropertyNameAtomicString(CSSPropertyID::kColor)
        ));
    }

    #[test]
    fn generated_hash_requires_exact_bytes_after_lookup() {
        for text in [
            "",
            "Color",
            "colro",
            "color\0",
            "-not-a-css-property",
            "--custom",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            assert!(FindProperty(text.as_bytes()).is_none(), "{text:?}");
        }
        let mut high_byte_at_shifted_hash_slot = *b"background-color";
        high_byte_at_shifted_hash_slot[15] = 0xff;
        assert!(FindProperty(&high_byte_at_shifted_hash_slot).is_none());
        let ids: Vec<_> = CSSPropertyIDList.into_iter().collect();
        assert_eq!(ids.len(), 705);
        assert_eq!(ids.first(), Some(&kFirstCSSProperty));
        assert_eq!(ids.last(), Some(&kLastCSSProperty));
    }
}

#![allow(non_snake_case)]

use foundation::{
    kIndefiniteSize, CalcSizeKeywordBehavior, EBorderCollapse, EBoxSizing, EPageMarginSafety,
    ETextAlign, IsLtr, IsParallelWritingMode, LayoutUnit, Length, LengthType,
    MinimumValueForLength, PhysicalSize, TextDirection, ValueForLength, WritingDirectionMode,
    WritingMode,
};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_geometry::geometry::box_strut::{BoxStrut, LineBoxStrut, PhysicalBoxStrut};
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize, ToPhysicalSize};
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;
use super::constraint_space::{AutoSizeBehavior, ConstraintSpace};
use super::fragmentation_utils::IsBreakInside;
use super::layout_input_node::MinMaxSizesFloatInput;
use super::layout_object::LayoutObject;
use super::layout_pass_scope::LayoutPassScope;
use super::min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
use super::table_node::TableNode;
use std::cell::Cell;

// The foundation length owner supplies the EvaluationInput-aware overload.
// Its evaluator resolves nested calc-size keywords in this layoutng module.
unsafe extern "Rust" {
    fn FoundationMinimumValueForLengthWithIntrinsicEvaluator(
        length: &Length,
        percentage_resolution_size: LayoutUnit,
        evaluator: &mut dyn FnMut(&Length) -> LayoutUnit,
        calc_size_keyword_behavior: CalcSizeKeywordBehavior,
    ) -> LayoutUnit;
}

// cpp: layoutng/internal/length_utils.h:28-45
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SizeType {
    kContent,
    kIntrinsic,
}

pub type MinMaxSizesFunctionRef<'a> = &'a dyn Fn(SizeType) -> MinMaxSizesResult;
pub type BlockSizeFunctionRef<'a> = &'a dyn Fn(SizeType) -> LayoutUnit;

// cpp: layoutng/internal/length_utils.h:48-52
pub fn NeedMinMaxSize(style: &ComputedStyle) -> bool {
    style.LogicalWidth().HasContentOrIntrinsic()
        || style.LogicalMinWidth().HasContentOrIntrinsic()
        || style.LogicalMaxWidth().HasContentOrIntrinsic()
}

// cpp: layoutng/internal/length_utils.h:69-82
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthTypeInternal {
    kMin,
    kMain,
    kMax,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitContentMode {
    kNormal,
    kMinContribution,
    kMaxContribution,
}

// cpp: layoutng/internal/length_utils.h:383-389
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacedSizeMode {
    kNormal,
    kIgnoreInlineLengths,
    kIgnoreBlockLengths,
}

// cpp: layoutng/internal/length_utils.h:290-293
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferredSizesMode {
    kNormal,
    kIgnore,
}

// cpp: layoutng/internal/length_utils_calculations.cc:13-144
// cpp: layoutng/internal/length_utils.h:86-98
pub fn ResolveInlineLengthInternal(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
    original_length: &Length,
    auto_length: Option<&Length>,
    length_type: LengthTypeInternal,
    fit_content_mode: FitContentMode,
    override_available_size: LayoutUnit,
    calc_size_keyword_behavior: CalcSizeKeywordBehavior,
) -> LayoutUnit {
    debug_assert_eq!(constraint_space.GetWritingMode(), style.GetWritingMode());
    let length = if original_length.IsAuto() {
        auto_length.unwrap_or(original_length)
    } else {
        original_length
    };
    match length.GetType() {
        LengthType::kStretch => {
            let available_size = if override_available_size == kIndefiniteSize {
                constraint_space.AvailableSize().inline_size
            } else {
                override_available_size
            };
            if available_size == kIndefiniteSize {
                return kIndefiniteSize;
            }
            debug_assert!(available_size >= LayoutUnit::default());
            let margins = ComputeMarginsForSelf(constraint_space, style);
            let ignored = constraint_space.IgnoreMarginsForStretch();
            border_padding.InlineSum().max(
                available_size
                    - if ignored.inline_start {
                        LayoutUnit::default()
                    } else {
                        margins.inline_start
                    }
                    - if ignored.inline_end {
                        LayoutUnit::default()
                    } else {
                        margins.inline_end
                    },
            )
        }
        LengthType::kPercent | LengthType::kFixed | LengthType::kCalculated => {
            let mut percentage_resolution_size = constraint_space.PercentageResolutionInlineSize();
            if length.HasPercent() && percentage_resolution_size == kIndefiniteSize {
                if length_type != LengthTypeInternal::kMin {
                    return kIndefiniteSize;
                }
                percentage_resolution_size = LayoutUnit::default();
            }
            let mut evaluated_indefinite = false;
            let mut evaluator = |length_to_evaluate: &Length| {
                let mut result = ResolveInlineLengthInternal(
                    constraint_space,
                    style,
                    border_padding,
                    min_max_sizes_func,
                    length_to_evaluate,
                    auto_length,
                    length_type,
                    fit_content_mode,
                    override_available_size,
                    calc_size_keyword_behavior,
                );
                if result == kIndefiniteSize {
                    evaluated_indefinite = true;
                    return kIndefiniteSize;
                }
                if style.BoxSizing() == EBoxSizing::kContentBox {
                    result -= border_padding.InlineSum();
                }
                debug_assert!(result >= LayoutUnit::default());
                result
            };
            let mut value = unsafe {
                FoundationMinimumValueForLengthWithIntrinsicEvaluator(
                    length,
                    percentage_resolution_size,
                    &mut evaluator,
                    calc_size_keyword_behavior,
                )
            };
            if evaluated_indefinite {
                return kIndefiniteSize;
            }
            if style.BoxSizing() == EBoxSizing::kBorderBox {
                value = border_padding.InlineSum().max(value);
            } else {
                value += border_padding.InlineSum();
            }
            value
        }
        LengthType::kContent | LengthType::kMaxContent => {
            min_max_sizes_func(SizeType::kContent).sizes.max_size
        }
        LengthType::kMinContent => min_max_sizes_func(SizeType::kContent).sizes.min_size,
        LengthType::kMinIntrinsic => min_max_sizes_func(SizeType::kIntrinsic).sizes.min_size,
        LengthType::kFitContent => {
            let available_size = if override_available_size == kIndefiniteSize {
                constraint_space.AvailableSize().inline_size
            } else {
                override_available_size
            };
            if available_size == kIndefiniteSize {
                return match fit_content_mode {
                    FitContentMode::kNormal => match length_type {
                        LengthTypeInternal::kMin => {
                            min_max_sizes_func(SizeType::kContent).sizes.min_size
                        }
                        LengthTypeInternal::kMain => kIndefiniteSize,
                        LengthTypeInternal::kMax => {
                            min_max_sizes_func(SizeType::kContent).sizes.max_size
                        }
                    },
                    FitContentMode::kMinContribution => {
                        min_max_sizes_func(SizeType::kContent).sizes.min_size
                    }
                    FitContentMode::kMaxContribution => {
                        min_max_sizes_func(SizeType::kContent).sizes.max_size
                    }
                };
            }
            debug_assert!(available_size >= LayoutUnit::default());
            let margins = ComputeMarginsForSelf(constraint_space, style);
            min_max_sizes_func(SizeType::kContent)
                .sizes
                .ShrinkToFit((available_size - margins.InlineSum()).ClampNegativeToZero())
        }
        LengthType::kAuto if length_type == LengthTypeInternal::kMin => border_padding.InlineSum(),
        LengthType::kAuto | LengthType::kNone => kIndefiniteSize,
        LengthType::kFlex => panic!("flex length should only be used for grid"),
        LengthType::kOverlapJoin => {
            panic!("overlap-join length should only be used for gap decoration insets")
        }
    }
}

// cpp: layoutng/internal/length_utils_calculations.cc:146-263
// cpp: layoutng/internal/length_utils.h:100-109
pub fn ResolveBlockLengthInternal(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    original_length: &Length,
    auto_length: Option<&Length>,
    length_type: LengthTypeInternal,
    override_available_size: LayoutUnit,
    override_percentage_resolution_size: Option<LayoutUnit>,
    block_size_func: BlockSizeFunctionRef<'_>,
) -> LayoutUnit {
    debug_assert_eq!(constraint_space.GetWritingMode(), style.GetWritingMode());
    let length = if original_length.IsAuto() {
        auto_length.unwrap_or(original_length)
    } else {
        original_length
    };
    match length.GetType() {
        LengthType::kStretch => {
            let available_size = if override_available_size == kIndefiniteSize {
                constraint_space.AvailableSize().block_size
            } else {
                override_available_size
            };
            if available_size == kIndefiniteSize {
                return if length_type == LengthTypeInternal::kMain {
                    block_size_func(SizeType::kContent)
                } else {
                    kIndefiniteSize
                };
            }
            debug_assert!(available_size >= LayoutUnit::default());
            let margins = ComputeMarginsForSelf(constraint_space, style);
            let ignored = constraint_space.IgnoreMarginsForStretch();
            border_padding.BlockSum().max(
                available_size
                    - if ignored.block_start {
                        LayoutUnit::default()
                    } else {
                        margins.block_start
                    }
                    - if ignored.block_end {
                        LayoutUnit::default()
                    } else {
                        margins.block_end
                    },
            )
        }
        LengthType::kPercent | LengthType::kFixed | LengthType::kCalculated => {
            let mut percentage_resolution_size = override_percentage_resolution_size
                .unwrap_or_else(|| constraint_space.PercentageResolutionBlockSize());
            if length.HasPercent() && percentage_resolution_size == kIndefiniteSize {
                match length_type {
                    LengthTypeInternal::kMin => percentage_resolution_size = LayoutUnit::default(),
                    LengthTypeInternal::kMain => return block_size_func(SizeType::kContent),
                    LengthTypeInternal::kMax => return kIndefiniteSize,
                }
            }
            let mut evaluated_indefinite = false;
            let mut evaluator = |length_to_evaluate: &Length| {
                let mut result = ResolveBlockLengthInternal(
                    constraint_space,
                    style,
                    border_padding,
                    length_to_evaluate,
                    auto_length,
                    length_type,
                    override_available_size,
                    override_percentage_resolution_size,
                    block_size_func,
                );
                if result == kIndefiniteSize {
                    evaluated_indefinite = true;
                    return kIndefiniteSize;
                }
                if style.BoxSizing() == EBoxSizing::kContentBox {
                    result -= border_padding.BlockSum();
                }
                debug_assert!(result >= LayoutUnit::default());
                result
            };
            let mut value = unsafe {
                FoundationMinimumValueForLengthWithIntrinsicEvaluator(
                    length,
                    percentage_resolution_size,
                    &mut evaluator,
                    CalcSizeKeywordBehavior::kAsSpecified,
                )
            };
            if evaluated_indefinite {
                return kIndefiniteSize;
            }
            if style.BoxSizing() == EBoxSizing::kBorderBox {
                value = border_padding.BlockSum().max(value);
            } else {
                value += border_padding.BlockSum();
            }
            value
        }
        LengthType::kContent
        | LengthType::kMinContent
        | LengthType::kMaxContent
        | LengthType::kMinIntrinsic
        | LengthType::kFitContent => {
            let intrinsic_size = block_size_func(if length.IsMinIntrinsic() {
                SizeType::kIntrinsic
            } else {
                SizeType::kContent
            });
            if intrinsic_size != kIndefiniteSize && !constraint_space.HasBlockFragmentation() {
                debug_assert!(intrinsic_size >= border_padding.BlockSum());
            }
            intrinsic_size
        }
        LengthType::kAuto if length_type == LengthTypeInternal::kMin => border_padding.BlockSum(),
        LengthType::kAuto | LengthType::kNone => kIndefiniteSize,
        LengthType::kFlex => panic!("flex length should only be used for grid"),
        LengthType::kOverlapJoin => {
            panic!("overlap-join length should only be used for gap decoration insets")
        }
    }
}

// cpp: layoutng/internal/length_utils.h:112-126
pub fn ResolveMinInlineLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
    length: &Length,
    auto_length: Option<&Length>,
    override_available_size: LayoutUnit,
    fit_content_mode: FitContentMode,
) -> LayoutUnit {
    let result = ResolveInlineLengthInternal(
        constraint_space,
        style,
        border_padding,
        min_max_sizes_func,
        length,
        auto_length,
        LengthTypeInternal::kMin,
        fit_content_mode,
        override_available_size,
        CalcSizeKeywordBehavior::kAsSpecified,
    );
    if result == kIndefiniteSize {
        border_padding.InlineSum()
    } else {
        result
    }
}

// cpp: layoutng/internal/length_utils.h:129-142
pub fn ResolveMaxInlineLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
    length: &Length,
    override_available_size: LayoutUnit,
    fit_content_mode: FitContentMode,
) -> LayoutUnit {
    let result = ResolveInlineLengthInternal(
        constraint_space,
        style,
        border_padding,
        min_max_sizes_func,
        length,
        None,
        LengthTypeInternal::kMax,
        fit_content_mode,
        override_available_size,
        CalcSizeKeywordBehavior::kAsSpecified,
    );
    if result == kIndefiniteSize {
        LayoutUnit::Max()
    } else {
        result
    }
}

// cpp: layoutng/internal/length_utils.h:145-159
pub fn ResolveMainInlineLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
    length: &Length,
    auto_length: Option<&Length>,
    override_available_size: LayoutUnit,
    calc_size_keyword_behavior: CalcSizeKeywordBehavior,
) -> LayoutUnit {
    ResolveInlineLengthInternal(
        constraint_space,
        style,
        border_padding,
        min_max_sizes_func,
        length,
        auto_length,
        LengthTypeInternal::kMain,
        FitContentMode::kNormal,
        override_available_size,
        calc_size_keyword_behavior,
    )
}

// cpp: layoutng/internal/length_utils.h:162-175
pub fn ResolveInitialMinBlockLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    length: &Length,
    override_available_size: LayoutUnit,
) -> LayoutUnit {
    let result = ResolveBlockLengthInternal(
        constraint_space,
        style,
        border_padding,
        length,
        Some(&Length::Auto()),
        LengthTypeInternal::kMin,
        override_available_size,
        None,
        &|_| kIndefiniteSize,
    );
    if result == kIndefiniteSize {
        border_padding.BlockSum()
    } else {
        result
    }
}

// cpp: layoutng/internal/length_utils.h:176-190
pub fn ResolveMinBlockLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    block_size_func: BlockSizeFunctionRef<'_>,
    length: &Length,
    auto_length: Option<&Length>,
    override_available_size: LayoutUnit,
    override_percentage_resolution_size: Option<LayoutUnit>,
) -> LayoutUnit {
    let result = ResolveBlockLengthInternal(
        constraint_space,
        style,
        border_padding,
        length,
        auto_length,
        LengthTypeInternal::kMin,
        override_available_size,
        override_percentage_resolution_size,
        block_size_func,
    );
    if result == kIndefiniteSize {
        border_padding.BlockSum()
    } else {
        result
    }
}

// cpp: layoutng/internal/length_utils.h:193-206
pub fn ResolveInitialMaxBlockLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    length: &Length,
    override_available_size: LayoutUnit,
) -> LayoutUnit {
    let result = ResolveBlockLengthInternal(
        constraint_space,
        style,
        border_padding,
        length,
        Some(&Length::Auto()),
        LengthTypeInternal::kMax,
        override_available_size,
        None,
        &|_| kIndefiniteSize,
    );
    if result == kIndefiniteSize {
        LayoutUnit::Max()
    } else {
        result
    }
}

// cpp: layoutng/internal/length_utils.h:207-221
pub fn ResolveMaxBlockLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    length: &Length,
    block_size_func: BlockSizeFunctionRef<'_>,
    override_available_size: LayoutUnit,
    override_percentage_resolution_size: Option<LayoutUnit>,
) -> LayoutUnit {
    let result = ResolveBlockLengthInternal(
        constraint_space,
        style,
        border_padding,
        length,
        Some(&Length::Auto()),
        LengthTypeInternal::kMax,
        override_available_size,
        override_percentage_resolution_size,
        block_size_func,
    );
    if result == kIndefiniteSize {
        LayoutUnit::Max()
    } else {
        result
    }
}

// cpp: layoutng/internal/length_utils.h:224-237
pub fn ResolveMainBlockLengthWithIntrinsicSize(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    length: &Length,
    auto_length: Option<&Length>,
    intrinsic_size: LayoutUnit,
    override_available_size: LayoutUnit,
) -> LayoutUnit {
    ResolveBlockLengthInternal(
        constraint_space,
        style,
        border_padding,
        length,
        auto_length,
        LengthTypeInternal::kMain,
        override_available_size,
        None,
        &|_| intrinsic_size,
    )
}

// cpp: layoutng/internal/length_utils.h:239-251
pub fn ResolveMainBlockLength(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    border_padding: &BoxStrut,
    length: &Length,
    auto_length: Option<&Length>,
    block_size_func: BlockSizeFunctionRef<'_>,
    override_available_size: LayoutUnit,
) -> LayoutUnit {
    ResolveBlockLengthInternal(
        constraint_space,
        style,
        border_padding,
        length,
        auto_length,
        LengthTypeInternal::kMain,
        override_available_size,
        None,
        block_size_func,
    )
}

// cpp: layoutng/internal/length_utils_calculations.cc:265-277
// cpp: layoutng/internal/length_utils.h:55-59
pub fn InlineSizeFromAspectRatio(
    border_padding: &BoxStrut,
    aspect_ratio: &LogicalSize,
    box_sizing: EBoxSizing,
    mut block_size: LayoutUnit,
) -> LayoutUnit {
    if box_sizing == EBoxSizing::kBorderBox {
        return border_padding
            .InlineSum()
            .max(block_size.MulDiv(aspect_ratio.inline_size, aspect_ratio.block_size));
    }
    block_size -= border_padding.BlockSum();
    block_size.MulDiv(aspect_ratio.inline_size, aspect_ratio.block_size)
        + border_padding.InlineSum()
}

// cpp: layoutng/internal/length_utils_calculations.cc:279-292
// cpp: layoutng/internal/length_utils.h:60-64
pub fn BlockSizeFromAspectRatio(
    border_padding: &BoxStrut,
    aspect_ratio: &LogicalSize,
    box_sizing: EBoxSizing,
    mut inline_size: LayoutUnit,
) -> LayoutUnit {
    debug_assert!(inline_size >= border_padding.InlineSum());
    if box_sizing == EBoxSizing::kBorderBox {
        return border_padding
            .BlockSum()
            .max(inline_size.MulDiv(aspect_ratio.block_size, aspect_ratio.inline_size));
    }
    inline_size -= border_padding.InlineSum();
    inline_size.MulDiv(aspect_ratio.block_size, aspect_ratio.inline_size)
        + border_padding.BlockSum()
}

// cpp: layoutng/internal/length_utils_calculations.cc:294-313
// cpp: layoutng/internal/length_utils.h:409-412
pub fn ResolveUsedColumnCountFromValues(
    computed_count: i32,
    computed_size: LayoutUnit,
    used_gap: LayoutUnit,
    available_size: LayoutUnit,
) -> i32 {
    if computed_size == kIndefiniteSize {
        return if computed_count == 0 {
            1
        } else {
            computed_count
        };
    }
    debug_assert!(computed_size > LayoutUnit::default());
    let count_from_width = ((available_size + used_gap) / (computed_size + used_gap))
        .ToInt()
        .max(1);
    if computed_count == 0 {
        count_from_width
    } else {
        computed_count.min(count_from_width).max(1)
    }
}

// cpp: layoutng/internal/length_utils_calculations.cc:315-325
// cpp: layoutng/internal/length_utils.h:413-414
pub fn ResolveUsedColumnCount(style: &ComputedStyle, available_size: LayoutUnit) -> i32 {
    let computed_size = if style.HasAutoColumnWidth() {
        kIndefiniteSize
    } else {
        LayoutUnit::from_signed(1).max(LayoutUnit::from_f32(style.ColumnWidth()))
    };
    let gap = ResolveColumnGapForMulticol(style, available_size);
    let computed_count = if style.HasAutoColumnCount() {
        0
    } else {
        i32::from(style.ColumnCount())
    };
    ResolveUsedColumnCountFromValues(computed_count, computed_size, gap, available_size)
}

// cpp: layoutng/internal/length_utils_calculations.cc:327-335
// cpp: layoutng/internal/length_utils.h:418-421
pub fn ResolveUsedColumnInlineSizeFromValues(
    computed_count: i32,
    computed_size: LayoutUnit,
    used_gap: LayoutUnit,
    available_size: LayoutUnit,
) -> LayoutUnit {
    let used_count =
        ResolveUsedColumnCountFromValues(computed_count, computed_size, used_gap, available_size);
    (((available_size + used_gap) / used_count) - used_gap).max(LayoutUnit::default())
}

// cpp: layoutng/internal/length_utils_calculations.cc:337-350
// cpp: layoutng/internal/length_utils.h:422-423
pub fn ResolveUsedColumnInlineSize(
    style: &ComputedStyle,
    available_size: LayoutUnit,
) -> LayoutUnit {
    debug_assert!(style.SpecifiesColumns());
    let computed_size = if style.HasAutoColumnWidth() {
        kIndefiniteSize
    } else {
        LayoutUnit::from_signed(1).max(LayoutUnit::from_f32(style.ColumnWidth()))
    };
    let computed_count = if style.HasAutoColumnCount() {
        0
    } else {
        i32::from(style.ColumnCount())
    };
    let gap = ResolveColumnGapForMulticol(style, available_size);
    ResolveUsedColumnInlineSizeFromValues(computed_count, computed_size, gap, available_size)
}

// cpp: layoutng/internal/length_utils_calculations.cc:352-358
// cpp: layoutng/internal/length_utils.h:428-429
pub fn ResolveColumnGapLength(
    style: &ComputedStyle,
    available_size: LayoutUnit,
) -> Option<LayoutUnit> {
    style
        .ColumnGap()
        .as_ref()
        .map(|gap| MinimumValueForLength(gap, available_size.ClampIndefiniteToZero()))
}

// cpp: layoutng/internal/length_utils_calculations.cc:360-364
// cpp: layoutng/internal/length_utils.h:431-432
pub fn ResolveColumnGapForMulticol(
    style: &ComputedStyle,
    available_size: LayoutUnit,
) -> LayoutUnit {
    ResolveColumnGapLength(style, available_size)
        .unwrap_or_else(|| LayoutUnit::from_signed(style.GetFontDescription().ComputedPixelSize()))
}

// cpp: layoutng/internal/length_utils_calculations.cc:366-372
// cpp: layoutng/internal/length_utils.h:437-438
pub fn ResolveRowGapLength(
    style: &ComputedStyle,
    available_size: LayoutUnit,
) -> Option<LayoutUnit> {
    style
        .RowGap()
        .as_ref()
        .map(|gap| MinimumValueForLength(gap, available_size.ClampIndefiniteToZero()))
}

// cpp: layoutng/internal/length_utils_calculations.cc:374-378
// cpp: layoutng/internal/length_utils.h:440-441
pub fn ColumnInlineProgression(style: &ComputedStyle, available_size: LayoutUnit) -> LayoutUnit {
    ResolveUsedColumnInlineSize(style, available_size)
        + ResolveColumnGapForMulticol(style, available_size)
}

// cpp: layoutng/internal/length_utils_calculations.cc:380-415
// cpp: layoutng/internal/length_utils.h:445-447
pub fn AdjustMarginsForPaperEdge(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    margins: &mut BoxStrut,
) {
    let safety = style.GetPageMarginSafety();
    if safety == EPageMarginSafety::kNone {
        return;
    }
    let sides = constraint_space.PaperEdgeAdjacentSides();
    if sides.IsEmpty() {
        return;
    }
    let safe_inset = constraint_space.SafePrintableInset();
    let adjust = |margin: &mut LayoutUnit| {
        if safety == EPageMarginSafety::kClamp {
            *margin = (*margin).max(safe_inset);
        } else {
            debug_assert_eq!(safety, EPageMarginSafety::kAdd);
            *margin += safe_inset;
        }
    };
    if sides.inline_start && !style.MarginInlineStart().IsAuto() {
        adjust(&mut margins.inline_start);
    }
    if sides.inline_end && !style.MarginInlineEnd().IsAuto() {
        adjust(&mut margins.inline_end);
    }
    if sides.block_start && !style.MarginBlockStart().IsAuto() {
        adjust(&mut margins.block_start);
    }
    if sides.block_end && !style.MarginBlockEnd().IsAuto() {
        adjust(&mut margins.block_end);
    }
}

// cpp: layoutng/internal/length_utils_calculations.cc:417-432
// cpp: layoutng/internal/length_utils.h:450-452
pub fn ComputePhysicalMarginsForSize(
    style: &ComputedStyle,
    percentage_resolution_size: PhysicalSize,
) -> PhysicalBoxStrut {
    if !style.MayHaveMargin() {
        return PhysicalBoxStrut::default();
    }
    PhysicalBoxStrut::new(
        MinimumValueForLength(style.MarginTop(), percentage_resolution_size.height),
        MinimumValueForLength(style.MarginRight(), percentage_resolution_size.width),
        MinimumValueForLength(style.MarginBottom(), percentage_resolution_size.height),
        MinimumValueForLength(style.MarginLeft(), percentage_resolution_size.width),
    )
}

// cpp: layoutng/internal/length_utils.h:456-474
pub fn ComputePhysicalMargins(
    style: &ComputedStyle,
    percentage_resolution_size: LogicalSize,
) -> PhysicalBoxStrut {
    if !style.MayHaveMargin() {
        return PhysicalBoxStrut::default();
    }
    let physical_size = ToPhysicalSize(
        percentage_resolution_size.ClampIndefiniteToZero(),
        style.GetWritingMode(),
    );
    ComputePhysicalMarginsForSize(style, physical_size)
}

// cpp: layoutng/internal/length_utils.h:477-483
pub fn ComputePhysicalMarginsForSpace(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
) -> PhysicalBoxStrut {
    ComputePhysicalMargins(
        style,
        constraint_space.MarginPaddingPercentageResolutionSize(),
    )
}

// cpp: layoutng/internal/length_utils_calculations.cc:434-443
// cpp: layoutng/internal/length_utils.h:481-483
pub fn ComputeMarginsFor(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    compute_for: &ConstraintSpace,
) -> BoxStrut {
    if !style.MayHaveMargin() || constraint_space.IsAnonymous() {
        return BoxStrut::default();
    }
    ComputePhysicalMargins(
        style,
        constraint_space.MarginPaddingPercentageResolutionSize(),
    )
    .ConvertToLogical(compute_for.GetWritingDirection())
}

// cpp: layoutng/internal/length_utils.h:486-490
pub fn ComputeMarginsForSize(
    style: &ComputedStyle,
    percentage_resolution_size: LogicalSize,
    container_writing_direction: WritingDirectionMode,
) -> BoxStrut {
    ComputePhysicalMargins(style, percentage_resolution_size)
        .ConvertToLogical(container_writing_direction)
}

// cpp: layoutng/internal/length_utils.h:493-503
pub fn ComputeMarginsForInlineSize(
    style: &ComputedStyle,
    percentage_resolution_inline_size: LayoutUnit,
    container_writing_direction: WritingDirectionMode,
) -> BoxStrut {
    let size = LogicalSize::new(
        percentage_resolution_inline_size,
        percentage_resolution_inline_size,
    );
    ComputePhysicalMargins(style, size).ConvertToLogical(container_writing_direction)
}

// cpp: layoutng/internal/length_utils.h:506-511
pub fn ComputeMarginsForDirection(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
    container_writing_direction: WritingDirectionMode,
) -> BoxStrut {
    ComputePhysicalMarginsForSpace(constraint_space, style)
        .ConvertToLogical(container_writing_direction)
}

// cpp: layoutng/internal/length_utils.h:514-529
pub fn ComputeMarginsForSelf(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
) -> BoxStrut {
    if !style.MayHaveMargin() || constraint_space.IsAnonymous() {
        return BoxStrut::default();
    }
    let mut margins = ComputePhysicalMarginsForSpace(constraint_space, style)
        .ConvertToLogical(style.GetWritingDirection());
    if style.GetPageMarginSafety() != EPageMarginSafety::kNone {
        AdjustMarginsForPaperEdge(constraint_space, style, &mut margins);
    }
    margins
}

// cpp: layoutng/internal/length_utils.h:533-542
pub fn ComputeLineMarginsForSelf(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
) -> LineBoxStrut {
    if !style.MayHaveMargin() || constraint_space.IsAnonymous() {
        return LineBoxStrut::default();
    }
    ComputePhysicalMargins(
        style,
        constraint_space.MarginPaddingPercentageResolutionSize(),
    )
    .ConvertToLineLogical(style.GetWritingDirection())
}

// cpp: layoutng/internal/length_utils.h:546-557
pub fn ComputeLineMarginsForVisualContainer(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
) -> LineBoxStrut {
    if !style.MayHaveMargin() || constraint_space.IsAnonymous() {
        return LineBoxStrut::default();
    }
    ComputePhysicalMargins(
        style,
        constraint_space.MarginPaddingPercentageResolutionSize(),
    )
    .ConvertToLineLogical(WritingDirectionMode::new(
        constraint_space.GetWritingMode(),
        TextDirection::kLtr,
    ))
}

// cpp: layoutng/internal/length_utils.h:564-567
pub fn ComputeLineBorders(style: &ComputedStyle) -> LineBoxStrut {
    LineBoxStrut::from_box(
        &ComputeBordersForInline(style),
        style.IsFlippedLinesWritingMode(),
    )
}

// cpp: layoutng/internal/length_utils.h:574-578
pub fn ComputeLinePadding(
    constraint_space: &ConstraintSpace,
    style: &ComputedStyle,
) -> LineBoxStrut {
    LineBoxStrut::from_box(
        &ComputePadding(constraint_space, style),
        style.IsFlippedLinesWritingMode(),
    )
}

// cpp: layoutng/internal/length_utils_calculations.cc:445-472
// cpp: layoutng/internal/length_utils.h:571-572
pub fn ComputePadding(constraint_space: &ConstraintSpace, style: &ComputedStyle) -> BoxStrut {
    if !style.MayHavePadding() || constraint_space.IsAnonymous() {
        return BoxStrut::default();
    }
    if style.IsDisplayTable() && style.BorderCollapse() == EBorderCollapse::kCollapse {
        return BoxStrut::default();
    }
    let percentage_resolution_size = constraint_space
        .MarginPaddingPercentageResolutionSize()
        .ClampIndefiniteToZero();
    BoxStrut::new(
        MinimumValueForLength(
            style.PaddingInlineStart(),
            percentage_resolution_size.inline_size,
        ),
        MinimumValueForLength(
            style.PaddingInlineEnd(),
            percentage_resolution_size.inline_size,
        ),
        MinimumValueForLength(
            style.PaddingBlockStart(),
            percentage_resolution_size.block_size,
        ),
        MinimumValueForLength(
            style.PaddingBlockEnd(),
            percentage_resolution_size.block_size,
        ),
    )
}

// cpp: layoutng/internal/length_utils_calculations.cc:474-493
// cpp: layoutng/internal/length_utils.h:593-598
pub fn ResolveInlineAutoMargins(
    style: &ComputedStyle,
    container_style: &ComputedStyle,
    available_inline_size: LayoutUnit,
    inline_size: LayoutUnit,
    margins: &mut BoxStrut,
) {
    let available_space = available_inline_size - (inline_size + margins.InlineSum());
    let is_start_auto = style.MarginInlineStartUsing(container_style).IsAuto();
    let is_end_auto = style.MarginInlineEndUsing(container_style).IsAuto();
    if is_start_auto && is_end_auto {
        margins.inline_start = (available_space / 2i32).ClampNegativeToZero();
        margins.inline_end = available_inline_size - inline_size - margins.inline_start;
    } else if is_start_auto {
        margins.inline_start = available_space.ClampNegativeToZero();
    } else if is_end_auto {
        margins.inline_end = available_inline_size - inline_size - margins.inline_start;
    }
}

// cpp: layoutng/internal/length_utils_calculations.cc:495-513
// cpp: layoutng/internal/length_utils.h:603-607
pub fn ResolveAutoMargins(
    start_length: &Length,
    end_length: &Length,
    mut additional_space: LayoutUnit,
    start_result: &mut LayoutUnit,
    end_result: &mut LayoutUnit,
) {
    if start_length.IsAuto() {
        if end_length.IsAuto() {
            *start_result = additional_space / 2;
            additional_space -= *start_result;
        } else {
            *start_result = additional_space;
        }
    }
    if end_length.IsAuto() {
        *end_result = additional_space;
    }
}

// cpp: layoutng/internal/length_utils_calculations.cc:515-528
// cpp: layoutng/internal/length_utils.h:612-619
pub fn ResolveAutoMarginsForStrut(
    inline_start_length: &Length,
    inline_end_length: &Length,
    block_start_length: &Length,
    block_end_length: &Length,
    additional_inline_space: LayoutUnit,
    additional_block_space: LayoutUnit,
    margins: &mut BoxStrut,
) {
    ResolveAutoMargins(
        inline_start_length,
        inline_end_length,
        additional_inline_space,
        &mut margins.inline_start,
        &mut margins.inline_end,
    );
    ResolveAutoMargins(
        block_start_length,
        block_end_length,
        additional_block_space,
        &mut margins.block_start,
        &mut margins.block_end,
    );
}

// cpp: layoutng/internal/length_utils_calculations.cc:530-577
// cpp: layoutng/internal/length_utils.h:622-624
pub fn LineOffsetForTextAlign(
    mut text_align: ETextAlign,
    direction: TextDirection,
    space_left: LayoutUnit,
) -> LayoutUnit {
    let is_ltr = IsLtr(direction);
    if matches!(
        text_align,
        ETextAlign::kStart | ETextAlign::kJustify | ETextAlign::kMatchParent
    ) {
        text_align = if is_ltr {
            ETextAlign::kLeft
        } else {
            ETextAlign::kRight
        };
    } else if text_align == ETextAlign::kEnd {
        text_align = if is_ltr {
            ETextAlign::kRight
        } else {
            ETextAlign::kLeft
        };
    }
    match text_align {
        ETextAlign::kLeft | ETextAlign::kWebkitLeft => {
            if is_ltr {
                LayoutUnit::default()
            } else {
                space_left.ClampPositiveToZero()
            }
        }
        ETextAlign::kRight | ETextAlign::kWebkitRight => {
            if !is_ltr || space_left > LayoutUnit::default() {
                space_left
            } else {
                LayoutUnit::default()
            }
        }
        ETextAlign::kCenter | ETextAlign::kWebkitCenter => {
            if is_ltr || space_left > LayoutUnit::default() {
                (space_left / 2i32).ClampNegativeToZero()
            } else {
                space_left
            }
        }
        _ => unreachable!("text-align should have been resolved"),
    }
}

// cpp: layoutng/internal/length_utils_calculations.cc:579-591
// cpp: layoutng/internal/length_utils.h:648-648
pub fn ShrinkLogicalSize(mut size: LogicalSize, insets: &BoxStrut) -> LogicalSize {
    if size.inline_size != kIndefiniteSize {
        size.inline_size = (size.inline_size - insets.InlineSum()).ClampNegativeToZero();
    }
    if size.block_size != kIndefiniteSize {
        size.block_size = (size.block_size - insets.BlockSum()).ClampNegativeToZero();
    }
    size
}

// cpp: layoutng/internal/length_utils.cc:1182-1201
// cpp: layoutng/internal/length_utils.h:651-655
pub fn CalculateChildAvailableSize(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_box_size: LogicalSize,
    border_scrollbar_padding: &BoxStrut,
) -> LogicalSize {
    let mut child_available_size = ShrinkLogicalSize(border_box_size, border_scrollbar_padding);
    if space.IsAnonymous()
        || (node.IsAnonymousBlockFlow() && child_available_size.block_size == kIndefiniteSize)
    {
        child_available_size.block_size = space.AvailableSize().block_size;
    }
    child_available_size
}

// cpp: layoutng/internal/length_utils.cc:1203-1217
fn AdjustChildPercentageSize(
    node: &BlockNode,
    mut child_percentage_size: LogicalSize,
    parent_percentage_block_size: LayoutUnit,
) -> LogicalSize {
    if child_percentage_size.block_size == kIndefiniteSize
        && node.UseParentPercentageResolutionBlockSizeForChildren()
    {
        child_percentage_size.block_size = parent_percentage_block_size;
    }
    child_percentage_size
}

// cpp: layoutng/internal/length_utils.cc:1219-1237
// cpp: layoutng/internal/length_utils.h:659-662
pub fn CalculateChildPercentageSize(
    space: &ConstraintSpace,
    node: &BlockNode,
    child_available_size: LogicalSize,
) -> LogicalSize {
    if space.IsAnonymous() || node.IsAnonymousBlockFlow() {
        return LogicalSize::new(
            child_available_size.inline_size,
            space.PercentageResolutionBlockSize(),
        );
    }
    if space.IsTableCellChild() {
        return child_available_size;
    }
    AdjustChildPercentageSize(
        node,
        child_available_size,
        space.PercentageResolutionBlockSize(),
    )
}

// cpp: layoutng/internal/length_utils.cc:1239-1276
// cpp: layoutng/internal/length_utils.h:666-671
pub fn CalculateReplacedChildPercentageSize(
    space: &ConstraintSpace,
    node: &BlockNode,
    child_available_size: LogicalSize,
    border_scrollbar_padding: &BoxStrut,
    border_padding: &BoxStrut,
) -> LogicalSize {
    if space.IsAnonymous() || node.IsAnonymousBlockFlow() {
        return LogicalSize::new(
            child_available_size.inline_size,
            space.ReplacedChildPercentageResolutionBlockSize(),
        );
    }
    if space.IsTableCellChild() {
        return child_available_size;
    }
    if space.IsTableCell() && node.Style().LogicalHeight().IsFixed() {
        let block_size = ComputeBlockSizeForFragmentInternal(
            space,
            node,
            border_padding,
            kIndefiniteSize,
            kIndefiniteSize,
            kIndefiniteSize,
        );
        debug_assert_ne!(block_size, kIndefiniteSize);
        return LogicalSize::new(
            child_available_size.inline_size,
            (block_size - border_scrollbar_padding.BlockSum()).ClampNegativeToZero(),
        );
    }
    AdjustChildPercentageSize(
        node,
        child_available_size,
        space.ReplacedChildPercentageResolutionBlockSize(),
    )
}

// cpp: layoutng/internal/length_utils.cc:1378-1393
// cpp: layoutng/internal/length_utils.h:750-754
pub fn AddScrollbarFreeze(
    scrollbars_before: &BoxStrut,
    scrollbars_after: &BoxStrut,
    writing_direction: WritingDirectionMode,
    freeze_horizontal: &mut bool,
    freeze_vertical: &mut bool,
) {
    let before = scrollbars_before.ConvertToPhysical(writing_direction);
    let after = scrollbars_after.ConvertToPhysical(writing_direction);
    *freeze_horizontal |= (before.top == LayoutUnit::default()
        && after.top != LayoutUnit::default())
        || (before.bottom == LayoutUnit::default() && after.bottom != LayoutUnit::default());
    *freeze_vertical |= (before.left == LayoutUnit::default()
        && after.left != LayoutUnit::default())
        || (before.right == LayoutUnit::default() && after.right != LayoutUnit::default());
}

// cpp: layoutng/internal/length_utils.cc:1070-1077
// cpp: layoutng/internal/length_utils.h:581-581
pub fn ComputeScrollbarsForNonAnonymous(node: &BlockNode) -> BoxStrut {
    let style = node.Style();
    if !style.IsScrollContainer() && style.IsScrollbarGutterAuto() {
        return BoxStrut::default();
    }
    unsafe { &*node.GetLayoutBox() }.ComputeLogicalScrollbars()
}

// cpp: layoutng/internal/length_utils.cc:1026-1033
fn ComputeBordersInternal(style: &ComputedStyle) -> BoxStrut {
    PhysicalBoxStrut::FromInts(
        style.BorderTopWidth(),
        style.BorderRightWidth(),
        style.BorderBottomWidth(),
        style.BorderLeftWidth(),
    )
    .ConvertToLogical(style.GetWritingDirection())
}

// cpp: layoutng/internal/length_utils.cc:1035-1056
// cpp: layoutng/internal/length_utils.h:558-558
pub fn ComputeBorders(constraint_space: &ConstraintSpace, node: &BlockNode) -> BoxStrut {
    if constraint_space.IsAnonymous() {
        return BoxStrut::default();
    }
    if constraint_space.IsTableCell() {
        return constraint_space.TableCellBorders();
    }
    if node.IsTable() {
        let algorithms = LayoutPassScope::Algorithms();
        let borders = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.table_support.borders
        };
        let borders = borders.expect("table border support was not assembled");
        let table = TableNode::new(node.GetLayoutBox());
        let result = borders(&table);
        debug_assert!(!result.is_null());
        return unsafe { *result };
    }
    ComputeBordersInternal(node.Style())
}

// cpp: layoutng/internal/length_utils.cc:1058-1068
// cpp: layoutng/internal/length_utils.h:560-560
pub fn ComputeBordersForInline(style: &ComputedStyle) -> BoxStrut {
    ComputeBordersInternal(style)
}

// cpp: layoutng/internal/length_utils.h:562-562
pub fn ComputeNonCollapsedTableBorders(style: &ComputedStyle) -> BoxStrut {
    ComputeBordersInternal(style)
}

// cpp: layoutng/internal/length_utils.h:569-569
pub fn ComputeBordersForTest(style: &ComputedStyle) -> BoxStrut {
    ComputeBordersInternal(style)
}

// cpp: layoutng/internal/length_utils.h:583-591
pub fn ComputeScrollbars(space: &ConstraintSpace, node: &BlockNode) -> BoxStrut {
    if space.IsAnonymous() {
        BoxStrut::default()
    } else {
        ComputeScrollbarsForNonAnonymous(node)
    }
}

// cpp: layoutng/internal/length_utils.cc:1080-1093
// cpp: layoutng/internal/length_utils.h:377-381
pub fn CalculateDefaultBlockSize(
    space: &ConstraintSpace,
    node: &BlockNode,
    break_token: *const BlockBreakToken,
    border_scrollbar_padding: &BoxStrut,
) -> LayoutUnit {
    if node.IsQuirkyAndFillsViewport() && !IsBreakInside(break_token) {
        let mut block_size = space.AvailableSize().block_size;
        block_size -= ComputeMarginsForSelf(space, node.Style()).BlockSum();
        return block_size
            .ClampNegativeToZero()
            .max(border_scrollbar_padding.BlockSum());
    }
    kIndefiniteSize
}

// cpp: layoutng/internal/length_utils.cc:1313-1346
// cpp: layoutng/internal/length_utils.h:732-734
pub fn CalculateMinMaxSizesIgnoringChildren(
    node: &BlockNode,
    border_scrollbar_padding: &BoxStrut,
) -> Option<MinMaxSizesResult> {
    let mut sizes = MinMaxSizes::default();
    sizes += border_scrollbar_padding.InlineSum();
    let override_size = node.OverrideIntrinsicContentInlineSize();
    if override_size != kIndefiniteSize {
        sizes += override_size;
        return Some(MinMaxSizesResult::new(sizes, false));
    }
    let default_size = node.DefaultIntrinsicContentInlineSize();
    if default_size != kIndefiniteSize {
        sizes += default_size;
        if node.IsTextArea() {
            sizes -= ComputeScrollbarsForNonAnonymous(node).InlineSum();
        }
        return Some(MinMaxSizesResult::new(sizes, false));
    }
    if node.ShouldApplyInlineSizeContainment() || !node.FirstChild().is_non_null() {
        return Some(MinMaxSizesResult::new(sizes, false));
    }
    None
}

// cpp: layoutng/internal/length_utils.cc:1348-1376
// cpp: layoutng/internal/length_utils.h:739-742
pub fn CalculateIntrinsicBlockSizeIgnoringChildren(
    node: &BlockNode,
    border_scrollbar_padding: &BoxStrut,
    children_have_geometry: bool,
) -> LayoutUnit {
    let override_size = node.OverrideIntrinsicContentBlockSize();
    if override_size != kIndefiniteSize {
        return override_size + border_scrollbar_padding.BlockSum();
    }
    let default_size = node.DefaultIntrinsicContentBlockSize(children_have_geometry);
    if default_size != kIndefiniteSize {
        if node.IsTextArea() {
            return default_size - ComputeScrollbarsForNonAnonymous(node).BlockSum()
                + border_scrollbar_padding.BlockSum();
        }
        return default_size + border_scrollbar_padding.BlockSum();
    }
    if node.ShouldApplyBlockSizeContainment() {
        return border_scrollbar_padding.BlockSum();
    }
    kIndefiniteSize
}

// cpp: layoutng/internal/length_utils.cc:1278-1311
// cpp: layoutng/internal/length_utils.h:676-682
pub fn ClampIntrinsicBlockSize(
    space: &ConstraintSpace,
    node: &BlockNode,
    break_token: *const BlockBreakToken,
    border_scrollbar_padding: &BoxStrut,
    mut current_intrinsic_block_size: LayoutUnit,
    body_margin_block_sum: Option<LayoutUnit>,
) -> LayoutUnit {
    debug_assert!(!node.IsTable());
    let intrinsic_block_size =
        CalculateIntrinsicBlockSizeIgnoringChildren(node, border_scrollbar_padding, true);
    if intrinsic_block_size != kIndefiniteSize {
        return intrinsic_block_size;
    }

    let style = node.Style();
    if !IsBreakInside(break_token)
        && node.IsQuirkyAndFillsViewport()
        && style.LogicalHeight().IsAuto()
        && space.AvailableSize().block_size != kIndefiniteSize
    {
        debug_assert_eq!(
            node.IsBody() && !node.CreatesNewFormattingContext(),
            body_margin_block_sum.is_some()
        );
        let margin_sum =
            body_margin_block_sum.unwrap_or_else(|| ComputeMarginsForSelf(space, style).BlockSum());
        current_intrinsic_block_size = current_intrinsic_block_size
            .max((space.AvailableSize().block_size - margin_sum).ClampNegativeToZero());
    }
    current_intrinsic_block_size
}

// cpp: layoutng/internal/length_utils.cc:423-443
// cpp: layoutng/internal/length_utils.h:270-274
pub fn ComputeTransferredMinMaxInlineSizes(
    ratio: &LogicalSize,
    block_min_max: &MinMaxSizes,
    border_padding: &BoxStrut,
    sizing: EBoxSizing,
) -> MinMaxSizes {
    debug_assert!(!ratio.IsEmpty());
    let mut transferred = MinMaxSizes {
        min_size: LayoutUnit::default(),
        max_size: LayoutUnit::Max(),
    };
    if block_min_max.min_size > LayoutUnit::default() {
        transferred.min_size =
            InlineSizeFromAspectRatio(border_padding, ratio, sizing, block_min_max.min_size);
    }
    if block_min_max.max_size != LayoutUnit::Max() {
        transferred.max_size =
            InlineSizeFromAspectRatio(border_padding, ratio, sizing, block_min_max.max_size);
    }
    transferred.max_size = transferred.max_size.max(transferred.min_size);
    transferred
}

// cpp: layoutng/internal/length_utils.cc:445-463
// cpp: layoutng/internal/length_utils.h:275-278
pub fn ComputeTransferredMinMaxBlockSizes(
    ratio: &LogicalSize,
    inline_min_max: &MinMaxSizes,
    border_padding: &BoxStrut,
    sizing: EBoxSizing,
) -> MinMaxSizes {
    let mut transferred = MinMaxSizes {
        min_size: LayoutUnit::default(),
        max_size: LayoutUnit::Max(),
    };
    if inline_min_max.min_size > LayoutUnit::default() {
        transferred.min_size =
            BlockSizeFromAspectRatio(border_padding, ratio, sizing, inline_min_max.min_size);
    }
    if inline_min_max.max_size != LayoutUnit::Max() {
        transferred.max_size =
            BlockSizeFromAspectRatio(border_padding, ratio, sizing, inline_min_max.max_size);
    }
    transferred.max_size = transferred.max_size.max(transferred.min_size);
    transferred
}

// cpp: layoutng/internal/length_utils.cc:379-392
// cpp: layoutng/internal/length_utils.h:257-260
pub fn ComputeInitialMinMaxBlockSizes(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    override_available_size: LayoutUnit,
) -> MinMaxSizes {
    let style = node.Style();
    let mut sizes = MinMaxSizes {
        min_size: ResolveInitialMinBlockLength(
            space,
            style,
            border_padding,
            style.LogicalMinHeight(),
            override_available_size,
        ),
        max_size: ResolveInitialMaxBlockLength(
            space,
            style,
            border_padding,
            style.LogicalMaxHeight(),
            override_available_size,
        ),
    };
    sizes.max_size = sizes.max_size.max(sizes.min_size);
    sizes
}

// cpp: layoutng/internal/length_utils.cc:394-421
// cpp: layoutng/internal/length_utils.h:262-268
pub fn ComputeMinMaxBlockSizes(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    auto_min_length: Option<&Length>,
    block_size_func: BlockSizeFunctionRef<'_>,
    override_available_size: LayoutUnit,
) -> MinMaxSizes {
    let style = node.Style();
    let mut sizes = MinMaxSizes {
        min_size: ResolveMinBlockLength(
            space,
            style,
            border_padding,
            block_size_func,
            style.LogicalMinHeight(),
            auto_min_length,
            override_available_size,
            None,
        ),
        max_size: ResolveMaxBlockLength(
            space,
            style,
            border_padding,
            style.LogicalMaxHeight(),
            block_size_func,
            override_available_size,
            None,
        ),
    };
    if auto_min_length.is_some() && style.LogicalMinHeight().HasAuto() {
        sizes.min_size = sizes.min_size.min(sizes.max_size);
    }
    if node.IsTable() {
        sizes.EncompassValue(block_size_func(SizeType::kIntrinsic));
    }
    sizes.max_size = sizes.max_size.max(sizes.min_size);
    sizes
}

// cpp: layoutng/internal/length_utils.cc:465-484
// cpp: layoutng/internal/length_utils.h:285-288
pub fn ComputeMinMaxInlineSizesFromAspectRatio(
    constraint_space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
) -> MinMaxSizes {
    let style = node.Style();
    debug_assert!(!style.AspectRatio().IsAuto());
    let block_min_max =
        ComputeInitialMinMaxBlockSizes(constraint_space, node, border_padding, kIndefiniteSize);
    ComputeTransferredMinMaxInlineSizes(
        &style.LogicalAspectRatio(),
        &block_min_max,
        border_padding,
        style.BoxSizingForAspectRatio(),
    )
}

// cpp: layoutng/internal/length_utils.cc:486-531
// cpp: layoutng/internal/length_utils.h:295-303
pub fn ComputeMinMaxInlineSizes(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    auto_min_length: Option<&Length>,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
    transferred_sizes_mode: TransferredSizesMode,
    fit_content_mode: FitContentMode,
    override_available_size: LayoutUnit,
) -> MinMaxSizes {
    let style = node.Style();
    let mut sizes = MinMaxSizes {
        min_size: ResolveMinInlineLength(
            space,
            style,
            border_padding,
            min_max_sizes_func,
            style.LogicalMinWidth(),
            auto_min_length,
            override_available_size,
            fit_content_mode,
        ),
        max_size: ResolveMaxInlineLength(
            space,
            style,
            border_padding,
            min_max_sizes_func,
            style.LogicalMaxWidth(),
            override_available_size,
            fit_content_mode,
        ),
    };
    if auto_min_length.is_some() && style.LogicalMinWidth().HasAuto() {
        sizes.min_size = sizes.min_size.min(sizes.max_size);
    }
    if transferred_sizes_mode == TransferredSizesMode::kNormal
        && !style.AspectRatio().IsAuto()
        && style.LogicalWidth().HasAuto()
        && space.InlineAutoBehavior() != AutoSizeBehavior::kStretchExplicit
    {
        let transferred_sizes =
            ComputeMinMaxInlineSizesFromAspectRatio(space, node, border_padding);
        sizes.min_size = sizes
            .min_size
            .max(transferred_sizes.min_size.min(sizes.max_size));
        sizes.max_size = sizes.max_size.min(transferred_sizes.max_size);
    }
    if node.IsTable() {
        sizes.EncompassValue(min_max_sizes_func(SizeType::kIntrinsic).sizes.min_size);
    }
    sizes.max_size = sizes.max_size.max(sizes.min_size);
    sizes
}

// cpp: layoutng/internal/length_utils.cc:249-321
// cpp: layoutng/internal/length_utils.h:323-327
pub fn ComputeInlineSizeForFragmentInternal(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
) -> LayoutUnit {
    let style = node.Style();
    let logical_width = style.LogicalWidth();
    let may_apply_aspect_ratio = !style.AspectRatio().IsAuto()
        && !(style.LogicalHeight().HasAuto()
            && space.BlockAutoBehavior() != AutoSizeBehavior::kStretchExplicit)
        && ComputeBlockSizeForFragment(
            space,
            node,
            border_padding,
            kIndefiniteSize,
            kIndefiniteSize,
            kIndefiniteSize,
        ) != kIndefiniteSize;
    let auto_length = if space.AvailableSize().inline_size == kIndefiniteSize {
        Length::MinContent()
    } else if space.InlineAutoBehavior() == AutoSizeBehavior::kStretchExplicit {
        Length::Stretch()
    } else if may_apply_aspect_ratio {
        Length::FitContent()
    } else if space.InlineAutoBehavior() == AutoSizeBehavior::kStretchImplicit {
        Length::Stretch()
    } else {
        debug_assert_eq!(space.InlineAutoBehavior(), AutoSizeBehavior::kFitContent);
        Length::FitContent()
    };
    let apply_automatic_min_size = !style.IsOverflowValueScrollableInline()
        && may_apply_aspect_ratio
        && (logical_width.HasContentOrIntrinsic()
            || (logical_width.HasAuto() && auto_length.HasContentOrIntrinsic()));
    let extent = ResolveMainInlineLength(
        space,
        style,
        border_padding,
        min_max_sizes_func,
        logical_width,
        Some(&auto_length),
        kIndefiniteSize,
        CalcSizeKeywordBehavior::kAsSpecified,
    );
    let auto_min_length = apply_automatic_min_size.then(Length::MinIntrinsic);
    ComputeMinMaxInlineSizes(
        space,
        node,
        border_padding,
        auto_min_length,
        min_max_sizes_func,
        TransferredSizesMode::kNormal,
        FitContentMode::kNormal,
        kIndefiniteSize,
    )
    .ClampSizeToMinAndMax(extent)
}

// cpp: layoutng/internal/length_utils.cc:323-344
// cpp: layoutng/internal/length_utils.h:329-333
pub fn ComputeInlineSizeForFragment(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
) -> LayoutUnit {
    if space.IsFixedInlineSize() || space.IsAnonymous() {
        return space.AvailableSize().inline_size;
    }
    if node.IsTable() {
        let algorithms = LayoutPassScope::Algorithms();
        let inline_size = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.table_support.inline_size
        };
        let inline_size = inline_size.expect("table intrinsic-size support was not assembled");
        let table = TableNode::new(node.GetLayoutBox());
        return inline_size(&table, space, border_padding);
    }
    ComputeInlineSizeForFragmentInternal(space, node, border_padding, min_max_sizes_func)
}

// cpp: layoutng/internal/length_utils.h:341-355
pub fn ComputeInlineSizeForFragmentWithOverride(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    override_min_max_sizes_for_test: Option<&MinMaxSizes>,
) -> LayoutUnit {
    let min_max_sizes_func = |type_: SizeType| {
        if let Some(sizes) = override_min_max_sizes_for_test {
            return MinMaxSizesResult::new(*sizes, false);
        }
        node.ComputeMinMaxSizes(
            space.GetWritingMode(),
            type_,
            space,
            MinMaxSizesFloatInput::default(),
        )
    };
    ComputeInlineSizeForFragment(space, node, border_padding, &min_max_sizes_func)
}

// cpp: layoutng/internal/length_utils.cc:346-377
// cpp: layoutng/internal/length_utils.h:361-365
pub fn ComputeUsedInlineSizeForTableFragment(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    table_grid_min_max_sizes: &MinMaxSizes,
) -> LayoutUnit {
    debug_assert!(!space.IsFixedInlineSize());
    let min_max_sizes_func = |type_: SizeType| {
        let style = node.Style();
        if !style.AspectRatio().IsAuto() && type_ == SizeType::kContent {
            let block_size = ComputeBlockSizeForFragment(
                space,
                node,
                border_padding,
                kIndefiniteSize,
                kIndefiniteSize,
                kIndefiniteSize,
            );
            if block_size != kIndefiniteSize {
                let inline_size = InlineSizeFromAspectRatio(
                    border_padding,
                    &style.LogicalAspectRatio(),
                    style.BoxSizingForAspectRatio(),
                    block_size,
                );
                return MinMaxSizesResult::new(
                    MinMaxSizes {
                        min_size: inline_size,
                        max_size: inline_size,
                    },
                    false,
                );
            }
        }
        MinMaxSizesResult::new(*table_grid_min_max_sizes, false)
    };
    ComputeInlineSizeForFragmentInternal(space, node, border_padding, &min_max_sizes_func)
}

// cpp: layoutng/internal/length_utils.cc:533-626
fn ComputeBlockSizeForFragmentInternal(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    intrinsic_size: LayoutUnit,
    inline_size: LayoutUnit,
    override_available_size: LayoutUnit,
) -> LayoutUnit {
    let style = node.Style();
    if space.IsRestrictedBlockSizeTableCellChild() {
        return ResolveInitialMinBlockLength(
            space,
            style,
            border_padding,
            style.LogicalMinHeight(),
            override_available_size,
        );
    }
    let logical_height = style.LogicalHeight();
    let has_aspect_ratio = !style.AspectRatio().IsAuto();
    let may_apply_aspect_ratio = has_aspect_ratio && inline_size != kIndefiniteSize;
    let auto_length = if space.AvailableSize().block_size == kIndefiniteSize {
        Length::FitContent()
    } else if space.BlockAutoBehavior() == AutoSizeBehavior::kStretchExplicit {
        Length::Stretch()
    } else if may_apply_aspect_ratio {
        Length::FitContent()
    } else if space.BlockAutoBehavior() == AutoSizeBehavior::kStretchImplicit {
        Length::Stretch()
    } else {
        debug_assert_eq!(space.BlockAutoBehavior(), AutoSizeBehavior::kFitContent);
        Length::FitContent()
    };
    let apply_automatic_min_size = intrinsic_size != kIndefiniteSize
        && !style.IsOverflowValueScrollableBlock()
        && may_apply_aspect_ratio
        && (logical_height.HasContentOrIntrinsic()
            || (logical_height.HasAuto() && auto_length.HasContentOrIntrinsic()));
    let block_size_func = |type_: SizeType| {
        if type_ == SizeType::kContent && has_aspect_ratio && inline_size != kIndefiniteSize {
            BlockSizeFromAspectRatio(
                border_padding,
                &style.LogicalAspectRatio(),
                style.BoxSizingForAspectRatio(),
                inline_size,
            )
        } else {
            intrinsic_size
        }
    };
    let extent = ResolveMainBlockLength(
        space,
        style,
        border_padding,
        logical_height,
        Some(&auto_length),
        &block_size_func,
        override_available_size,
    );
    if extent == kIndefiniteSize {
        debug_assert_eq!(intrinsic_size, kIndefiniteSize);
        return extent;
    }
    let auto_min_length = apply_automatic_min_size.then(Length::MinIntrinsic);
    let mut min_max = ComputeMinMaxBlockSizes(
        space,
        node,
        border_padding,
        auto_min_length,
        &block_size_func,
        override_available_size,
    );
    if space.MinBlockSizeShouldEncompassIntrinsicSize() && intrinsic_size != kIndefiniteSize {
        min_max.EncompassValue(intrinsic_size.min(min_max.max_size));
    }
    min_max.ClampSizeToMinAndMax(extent)
}

// cpp: layoutng/internal/length_utils.cc:628-655
// cpp: layoutng/internal/length_utils.h:315-321
pub fn ComputeBlockSizeForFragment(
    constraint_space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    intrinsic_size: LayoutUnit,
    inline_size: LayoutUnit,
    override_available_size: LayoutUnit,
) -> LayoutUnit {
    debug_assert!(override_available_size == kIndefiniteSize || node.IsTable());
    if constraint_space.IsFixedBlockSize() {
        let block_size = if override_available_size == kIndefiniteSize {
            constraint_space.AvailableSize().block_size
        } else {
            override_available_size
        };
        if constraint_space.MinBlockSizeShouldEncompassIntrinsicSize() {
            return intrinsic_size.max(block_size);
        }
        return block_size;
    }
    if constraint_space.IsTableCell() && intrinsic_size != kIndefiniteSize {
        return intrinsic_size;
    }
    if constraint_space.IsAnonymous() {
        return intrinsic_size;
    }
    ComputeBlockSizeForFragmentInternal(
        constraint_space,
        node,
        border_padding,
        intrinsic_size,
        inline_size,
        override_available_size,
    )
}

// cpp: layoutng/internal/length_utils.cc:657-672
// cpp: layoutng/internal/length_utils.h:367-373
pub fn ComputeInitialBlockSizeForFragment(
    space: &ConstraintSpace,
    node: &BlockNode,
    border_padding: &BoxStrut,
    intrinsic_size: LayoutUnit,
    inline_size: LayoutUnit,
    override_available_size: LayoutUnit,
) -> LayoutUnit {
    if space.IsInitialBlockSizeIndefinite() {
        return intrinsic_size;
    }
    ComputeBlockSizeForFragment(
        space,
        node,
        border_padding,
        intrinsic_size,
        inline_size,
        override_available_size,
    )
}

// cpp: layoutng/internal/length_utils.cc:674-688
fn ComputeDefaultNaturalSize(node: &BlockNode) -> LogicalSize {
    let style = node.Style();
    let mut natural_size =
        PhysicalSize::new(LayoutUnit::from_signed(300), LayoutUnit::from_signed(150));
    natural_size.ScaleFloat(style.EffectiveZoom());
    ToLogicalSize(natural_size, style.GetWritingMode())
}

// cpp: layoutng/internal/length_utils.cc:692-738
fn ComputeNormalizedNaturalSize(
    node: &BlockNode,
    border_padding: &BoxStrut,
    box_sizing: EBoxSizing,
    aspect_ratio: &LogicalSize,
) -> Option<LogicalSize> {
    let mut intrinsic_inline = None;
    let mut intrinsic_block = None;
    node.IntrinsicSize(&mut intrinsic_inline, &mut intrinsic_block);

    if let Some(size) = intrinsic_inline.as_mut() {
        *size += border_padding.InlineSum();
    } else if aspect_ratio.IsEmpty() {
        intrinsic_inline =
            Some(ComputeDefaultNaturalSize(node).inline_size + border_padding.InlineSum());
    }

    if let Some(size) = intrinsic_block.as_mut() {
        *size += border_padding.BlockSum();
    } else if aspect_ratio.IsEmpty() {
        intrinsic_block =
            Some(ComputeDefaultNaturalSize(node).block_size + border_padding.BlockSum());
    }

    if intrinsic_inline.is_none() {
        if let Some(block_size) = intrinsic_block {
            debug_assert!(!aspect_ratio.IsEmpty());
            intrinsic_inline = Some(InlineSizeFromAspectRatio(
                border_padding,
                aspect_ratio,
                box_sizing,
                block_size,
            ));
        }
    }
    if let Some(inline_size) = intrinsic_inline {
        if intrinsic_block.is_none() || !aspect_ratio.IsEmpty() {
            debug_assert!(!aspect_ratio.IsEmpty());
            intrinsic_block = Some(BlockSizeFromAspectRatio(
                border_padding,
                aspect_ratio,
                box_sizing,
                inline_size,
            ));
        }
    }

    debug_assert_eq!(intrinsic_inline.is_some(), intrinsic_block.is_some());
    intrinsic_inline
        .zip(intrinsic_block)
        .map(|(inline_size, block_size)| LogicalSize::new(inline_size, block_size))
}

// cpp: layoutng/internal/length_utils.cc:740-959
fn ComputeReplacedSizeInternal(
    node: &BlockNode,
    space: &ConstraintSpace,
    border_padding: &BoxStrut,
    mode: ReplacedSizeMode,
) -> LogicalSize {
    debug_assert!(node.IsReplaced());
    let style = node.Style();
    let box_sizing = style.BoxSizingForAspectRatio();
    let aspect_ratio = node.GetReplacedAspectRatio();
    let natural_size =
        ComputeNormalizedNaturalSize(node, border_padding, box_sizing, &aspect_ratio);
    let block_length = style.LogicalHeight();
    let block_size_func = |_: SizeType| -> LayoutUnit {
        if aspect_ratio.IsEmpty() {
            return natural_size
                .expect("replaced element without natural size")
                .block_size;
        }
        if mode == ReplacedSizeMode::kNormal {
            return ComputeReplacedSizeInternal(
                node,
                space,
                border_padding,
                ReplacedSizeMode::kIgnoreBlockLengths,
            )
            .block_size;
        }
        natural_size.map_or(kIndefiniteSize, |size| size.block_size)
    };

    let mut replaced_block = None;
    let block_min_max_sizes = if mode == ReplacedSizeMode::kIgnoreBlockLengths {
        MinMaxSizes {
            min_size: LayoutUnit::default(),
            max_size: LayoutUnit::Max(),
        }
    } else {
        let min_max_percentage_resolution_size = if unsafe { &*node.GetLayoutBox() }
            .InQuirksModeForLayout()
            && !node.IsOutOfFlowPositioned()
        {
            space.AvailableSize().block_size
        } else {
            space.PercentageResolutionBlockSize()
        };
        let mut sizes = MinMaxSizes {
            min_size: ResolveMinBlockLength(
                space,
                style,
                border_padding,
                &block_size_func,
                style.LogicalMinHeight(),
                None,
                kIndefiniteSize,
                Some(min_max_percentage_resolution_size),
            ),
            max_size: ResolveMaxBlockLength(
                space,
                style,
                border_padding,
                style.LogicalMaxHeight(),
                &block_size_func,
                kIndefiniteSize,
                Some(min_max_percentage_resolution_size),
            ),
        };
        sizes.max_size = sizes.max_size.max(sizes.min_size);
        if space.IsFixedBlockSize() {
            let size = space.AvailableSize().block_size;
            debug_assert!(size >= LayoutUnit::default());
            replaced_block = Some(size);
        } else {
            let auto_block_length = if space.IsBlockAutoBehaviorStretch() {
                Length::Stretch()
            } else {
                Length::FitContent()
            };
            let block_size = ResolveMainBlockLength(
                space,
                style,
                border_padding,
                block_length,
                Some(&auto_block_length),
                &block_size_func,
                kIndefiniteSize,
            );
            if block_size != kIndefiniteSize {
                debug_assert!(block_size >= LayoutUnit::default());
                replaced_block = Some(sizes.ClampSizeToMinAndMax(block_size));
            }
        }
        sizes
    };

    let transferred_min_max_sizes = if aspect_ratio.IsEmpty() {
        MinMaxSizes {
            min_size: LayoutUnit::default(),
            max_size: LayoutUnit::Max(),
        }
    } else {
        ComputeTransferredMinMaxInlineSizes(
            &aspect_ratio,
            &block_min_max_sizes,
            border_padding,
            box_sizing,
        )
    };
    let inline_length = style.LogicalWidth();
    let min_max_sizes_func = |_: SizeType| -> MinMaxSizesResult {
        let size = if aspect_ratio.IsEmpty() {
            natural_size
                .expect("replaced element without natural size")
                .inline_size
        } else if let Some(replaced_block) = replaced_block {
            InlineSizeFromAspectRatio(border_padding, &aspect_ratio, box_sizing, replaced_block)
        } else if let Some(natural_size) = natural_size {
            debug_assert_ne!(mode, ReplacedSizeMode::kIgnoreInlineLengths);
            if mode == ReplacedSizeMode::kNormal {
                ComputeReplacedSizeInternal(
                    node,
                    space,
                    border_padding,
                    ReplacedSizeMode::kIgnoreInlineLengths,
                )
                .inline_size
            } else {
                natural_size.inline_size
            }
        } else {
            kIndefiniteSize
        };
        MinMaxSizesResult::new(
            MinMaxSizes {
                min_size: size,
                max_size: size,
            },
            false,
        )
    };

    let mut replaced_inline = None;
    let mut inline_min_max_sizes = if mode == ReplacedSizeMode::kIgnoreInlineLengths {
        transferred_min_max_sizes
    } else {
        let mut sizes = MinMaxSizes {
            min_size: ResolveMinInlineLength(
                space,
                style,
                border_padding,
                &min_max_sizes_func,
                style.LogicalMinWidth(),
                None,
                kIndefiniteSize,
                FitContentMode::kNormal,
            ),
            max_size: ResolveMaxInlineLength(
                space,
                style,
                border_padding,
                &min_max_sizes_func,
                style.LogicalMaxWidth(),
                kIndefiniteSize,
                FitContentMode::kNormal,
            ),
        };
        sizes.max_size = sizes.max_size.max(sizes.min_size);
        if space.IsFixedInlineSize() {
            let size = space.AvailableSize().inline_size;
            debug_assert!(size >= LayoutUnit::default());
            replaced_inline = Some(size);
        } else {
            let auto_length = if space.IsInlineAutoBehaviorStretch() {
                Length::Stretch()
            } else {
                Length::FitContent()
            };
            let inline_size = ResolveMainInlineLength(
                space,
                style,
                border_padding,
                &min_max_sizes_func,
                inline_length,
                Some(&auto_length),
                kIndefiniteSize,
                CalcSizeKeywordBehavior::kAsSpecified,
            );
            if inline_size != kIndefiniteSize {
                debug_assert!(inline_size >= LayoutUnit::default());
                replaced_inline = Some(sizes.ClampSizeToMinAndMax(inline_size));
            }
        }
        if replaced_inline.is_none() {
            sizes.min_size = sizes
                .min_size
                .max(transferred_min_max_sizes.min_size.min(sizes.max_size));
            sizes.max_size = sizes.max_size.min(transferred_min_max_sizes.max_size);
        }
        sizes
    };

    if let (Some(inline), Some(block)) = (replaced_inline, replaced_block) {
        return LogicalSize::new(inline, block);
    }
    let stretch_fit = || -> LayoutUnit {
        if space.AvailableSize().inline_size != kIndefiniteSize {
            return ResolveMainInlineLength(
                space,
                style,
                border_padding,
                &|_| panic!("stretch cannot request intrinsic min/max size"),
                &Length::Stretch(),
                None,
                kIndefiniteSize,
                CalcSizeKeywordBehavior::kAsSpecified,
            );
        }
        if inline_length.HasPercent() {
            return ComputeDefaultNaturalSize(node).inline_size + border_padding.InlineSum();
        }
        border_padding.InlineSum()
    };
    if natural_size.is_none() && replaced_inline.is_none() && replaced_block.is_none() {
        replaced_inline = Some(inline_min_max_sizes.ClampSizeToMinAndMax(stretch_fit()));
    }
    if let Some(inline) = replaced_inline {
        debug_assert!(replaced_block.is_none());
        debug_assert!(natural_size.is_some() || !aspect_ratio.IsEmpty());
        let block = if aspect_ratio.IsEmpty() {
            natural_size.expect("natural size required").block_size
        } else {
            BlockSizeFromAspectRatio(border_padding, &aspect_ratio, box_sizing, inline)
        };
        replaced_block = Some(block_min_max_sizes.ClampSizeToMinAndMax(block));
        return LogicalSize::new(inline, replaced_block.expect("block size resolved"));
    }
    if let Some(block) = replaced_block {
        debug_assert!(replaced_inline.is_none());
        debug_assert!(natural_size.is_some() || !aspect_ratio.IsEmpty());
        let inline = if aspect_ratio.IsEmpty() {
            natural_size.expect("natural size required").inline_size
        } else {
            InlineSizeFromAspectRatio(border_padding, &aspect_ratio, box_sizing, block)
        };
        replaced_inline = Some(inline_min_max_sizes.ClampSizeToMinAndMax(inline));
        return LogicalSize::new(replaced_inline.expect("inline size resolved"), block);
    }
    let natural_size = natural_size.expect("natural size required for both unresolved axes");
    LogicalSize::new(
        inline_min_max_sizes.ClampSizeToMinAndMax(natural_size.inline_size),
        block_min_max_sizes.ClampSizeToMinAndMax(natural_size.block_size),
    )
}

// cpp: layoutng/internal/length_utils.cc:961-1024
// cpp: layoutng/internal/length_utils.h:400-404
pub fn ComputeReplacedSize(
    node: &BlockNode,
    space: &ConstraintSpace,
    border_padding: &BoxStrut,
    mode: ReplacedSizeMode,
) -> LogicalSize {
    debug_assert!(node.IsReplaced());
    let box_ = unsafe { &*node.GetLayoutBox() };
    if !box_.IsSVGRoot() || !box_.IsDocumentElement() {
        return ComputeReplacedSizeInternal(node, space, border_padding, mode);
    }
    let algorithms = LayoutPassScope::Algorithms();
    let root_sizing_info = if algorithms.is_null() {
        None
    } else {
        unsafe { &*algorithms }.svg_support.root_sizing_info
    }
    .expect("SVG layout module is not installed");
    // LayoutBox begins with its LayoutObject base through LayoutBoxModelObject.
    let svg_root = root_sizing_info(unsafe { &*(box_ as *const _ as *const LayoutObject) });
    let container_size = PhysicalSize::new(svg_root.container_width, svg_root.container_height);
    if !container_size.IsEmpty() {
        let mut size = ToLogicalSize(container_size, node.Style().GetWritingMode());
        size.inline_size += border_padding.InlineSum();
        size.block_size += border_padding.BlockSum();
        return size;
    }
    if svg_root.embedded_through_frame {
        let mut size = space.AvailableSize();
        let initial_size = node.InitialContainingBlockSize();
        size.block_size = if node.Style().IsHorizontalWritingMode() {
            initial_size.height
        } else {
            initial_size.width
        };
        return size;
    }
    let mut size = ComputeReplacedSizeInternal(node, space, border_padding, mode);
    if node.Style().LogicalWidth().HasPercent() {
        let factor = svg_root.logical_size_scale_factor;
        if factor != 1.0 {
            size.inline_size = LayoutUnit::from_f32(size.inline_size * factor as f32);
        }
    }
    let logical_height = node.Style().LogicalHeight();
    if logical_height.HasPercent() {
        let view = box_.View();
        let mut height = ValueForLength(
            logical_height,
            unsafe { &*view }.ViewLogicalHeightForPercentages(),
        );
        let factor = svg_root.logical_size_scale_factor;
        if factor != 1.0 {
            height = LayoutUnit::from_f32(height * factor as f32);
        }
        size.block_size = height;
    }
    size
}

// cpp: layoutng/internal/length_utils.cc:1095-1167
// cpp: layoutng/internal/length_utils.h:626-631
pub fn CalculateInitialFragmentGeometryWithMinMaxSizes(
    space: &ConstraintSpace,
    node: &BlockNode,
    break_token: *const BlockBreakToken,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
    is_intrinsic: bool,
) -> FragmentGeometry {
    let style = node.Style();
    if node.IsFrameSet() {
        if node.IsParentNGFrameSet() {
            let size = space.AvailableSize();
            debug_assert_ne!(size.inline_size, kIndefiniteSize);
            debug_assert_ne!(size.block_size, kIndefiniteSize);
            debug_assert!(space.IsFixedInlineSize());
            debug_assert!(space.IsFixedBlockSize());
            return FragmentGeometry {
                border_box_size: size,
                ..Default::default()
            };
        }
        let size = node.InitialContainingBlockSize();
        return FragmentGeometry {
            border_box_size: ToLogicalSize(size, style.GetWritingMode()),
            ..Default::default()
        };
    }
    let border = ComputeBorders(space, node);
    let padding = ComputePadding(space, style);
    let mut scrollbar = ComputeScrollbars(space, node);
    let border_padding = border + padding;
    let border_scrollbar_padding = border_padding + scrollbar;
    if node.IsReplaced() {
        let border_box_size = ComputeReplacedSize(
            node,
            space,
            &border_padding,
            if is_intrinsic {
                ReplacedSizeMode::kIgnoreInlineLengths
            } else {
                ReplacedSizeMode::kNormal
            },
        );
        return FragmentGeometry {
            border_box_size,
            border,
            scrollbar,
            padding,
        };
    }
    let inline_size = if is_intrinsic {
        kIndefiniteSize
    } else {
        ComputeInlineSizeForFragment(space, node, &border_padding, min_max_sizes_func)
    };
    if inline_size != kIndefiniteSize
        && inline_size < border_scrollbar_padding.InlineSum()
        && scrollbar.InlineSum() != LayoutUnit::default()
        && !space.IsAnonymous()
    {
        let content_box_inline_size = inline_size - border_padding.InlineSum();
        if scrollbar.InlineSum() > content_box_inline_size {
            if scrollbar.inline_start != LayoutUnit::default()
                && scrollbar.inline_end != LayoutUnit::default()
            {
                scrollbar.inline_start = content_box_inline_size / 2;
                scrollbar.inline_end = content_box_inline_size - scrollbar.inline_start;
            } else if scrollbar.inline_end != LayoutUnit::default() {
                debug_assert_eq!(scrollbar.inline_start, LayoutUnit::default());
                scrollbar.inline_end = content_box_inline_size;
            } else {
                debug_assert_ne!(scrollbar.inline_start, LayoutUnit::default());
                scrollbar.inline_start = content_box_inline_size;
            }
        }
    }
    let default_block_size =
        CalculateDefaultBlockSize(space, node, break_token, &border_scrollbar_padding);
    let block_size = ComputeInitialBlockSizeForFragment(
        space,
        node,
        &border_padding,
        default_block_size,
        inline_size,
        kIndefiniteSize,
    );
    FragmentGeometry {
        border_box_size: LogicalSize::new(inline_size, block_size),
        border,
        scrollbar,
        padding,
    }
}

// cpp: layoutng/internal/length_utils.cc:1169-1180
// cpp: layoutng/internal/length_utils.h:641-646
pub fn CalculateInitialFragmentGeometry(
    space: &ConstraintSpace,
    node: &BlockNode,
    break_token: *const BlockBreakToken,
    is_intrinsic: bool,
) -> FragmentGeometry {
    let min_max_sizes_func = |type_: SizeType| {
        node.ComputeMinMaxSizes(
            space.GetWritingMode(),
            type_,
            space,
            MinMaxSizesFloatInput::default(),
        )
    };
    CalculateInitialFragmentGeometryWithMinMaxSizes(
        space,
        node,
        break_token,
        &min_max_sizes_func,
        is_intrinsic,
    )
}

// cpp: layoutng/internal/length_utils.cc:36-77
fn ComputeMinAndMaxContentContributionForReplaced(
    child: &BlockNode,
    space: &ConstraintSpace,
) -> MinMaxSizesResult {
    let style = child.Style();
    let border_padding = ComputeBorders(space, child) + ComputePadding(space, style);
    let inline_size =
        ComputeReplacedSize(child, space, &border_padding, ReplacedSizeMode::kNormal).inline_size;
    let mut result = MinMaxSizes {
        min_size: inline_size,
        max_size: inline_size,
    };
    if style.LogicalWidth().HasPercent() || style.LogicalMaxWidth().HasPercent() {
        result.min_size = ResolveMinInlineLength(
            space,
            style,
            &border_padding,
            &|_| {
                let size = border_padding.InlineSum();
                MinMaxSizesResult::new(
                    MinMaxSizes {
                        min_size: size,
                        max_size: size,
                    },
                    false,
                )
            },
            style.LogicalMinWidth(),
            None,
            kIndefiniteSize,
            FitContentMode::kNormal,
        );
    }
    let depends_on_block_constraints = style.LogicalHeight().MayHavePercentDependence()
        || style.LogicalMinHeight().MayHavePercentDependence()
        || style.LogicalMaxHeight().MayHavePercentDependence()
        || (style.LogicalHeight().HasAuto() && space.IsBlockAutoBehaviorStretch());
    MinMaxSizesResult::new(result, depends_on_block_constraints)
}

// cpp: layoutng/internal/length_utils.cc:79-178
// cpp: layoutng/internal/length_utils.h:684-688
pub fn ComputeMinAndMaxContentContributionInternal(
    parent_writing_mode: WritingMode,
    child: &BlockNode,
    space: &ConstraintSpace,
    original_min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
) -> MinMaxSizesResult {
    let style = child.Style();
    let border_padding = ComputeBorders(space, child) + ComputePadding(space, style);
    if !IsParallelWritingMode(parent_writing_mode, style.GetWritingMode()) {
        let block_size = ComputeBlockSizeForFragment(
            space,
            child,
            &border_padding,
            kIndefiniteSize,
            kIndefiniteSize,
            kIndefiniteSize,
        );
        if block_size == kIndefiniteSize
            || style.LogicalMinHeight().HasContentOrIntrinsic()
            || style.LogicalMaxHeight().HasContentOrIntrinsic()
            || child.IsTable()
        {
            return original_min_max_sizes_func(SizeType::kContent);
        }
        return MinMaxSizesResult::new(
            MinMaxSizes {
                min_size: block_size,
                max_size: block_size,
            },
            false,
        );
    }

    let depends_on_block_constraints = Cell::new(false);
    let applied_aspect_ratio = Cell::new(false);
    let min_max_sizes_func = |type_: SizeType| {
        let result = original_min_max_sizes_func(type_);
        depends_on_block_constraints
            .set(depends_on_block_constraints.get() || result.depends_on_block_constraints);
        applied_aspect_ratio.set(applied_aspect_ratio.get() || result.applied_aspect_ratio);
        result
    };
    debug_assert_eq!(space.AvailableSize().inline_size, kIndefiniteSize);
    let main_length = style.LogicalWidth();
    let extent = ResolveMainInlineLength(
        space,
        style,
        &border_padding,
        &min_max_sizes_func,
        main_length,
        Some(&Length::FitContent()),
        kIndefiniteSize,
        CalcSizeKeywordBehavior::kAsSpecified,
    );
    let mut sizes = if extent == kIndefiniteSize {
        min_max_sizes_func(SizeType::kContent).sizes
    } else {
        MinMaxSizes {
            min_size: extent,
            max_size: extent,
        }
    };
    if main_length.IsCalculated()
        && (main_length.HasAuto() || main_length.HasFitContent() || main_length.HasStretch())
    {
        sizes.min_size = ResolveMainInlineLength(
            space,
            style,
            &border_padding,
            &min_max_sizes_func,
            main_length,
            Some(&Length::MinContent()),
            kIndefiniteSize,
            CalcSizeKeywordBehavior::kAsAuto,
        );
        sizes.max_size = ResolveMainInlineLength(
            space,
            style,
            &border_padding,
            &min_max_sizes_func,
            main_length,
            Some(&Length::MaxContent()),
            kIndefiniteSize,
            CalcSizeKeywordBehavior::kAsAuto,
        );
    }
    let auto_min_length = (!style.IsOverflowValueScrollableInline() && applied_aspect_ratio.get())
        .then(Length::MinIntrinsic);
    if style.LogicalMinWidth().HasFitContent() || style.LogicalMaxWidth().HasFitContent() {
        let min_sizes = ComputeMinMaxInlineSizes(
            space,
            child,
            &border_padding,
            auto_min_length,
            &min_max_sizes_func,
            TransferredSizesMode::kNormal,
            FitContentMode::kMinContribution,
            kIndefiniteSize,
        );
        let max_sizes = ComputeMinMaxInlineSizes(
            space,
            child,
            &border_padding,
            auto_min_length,
            &min_max_sizes_func,
            TransferredSizesMode::kNormal,
            FitContentMode::kMaxContribution,
            kIndefiniteSize,
        );
        sizes.min_size = min_sizes.ClampSizeToMinAndMax(sizes.min_size);
        sizes.max_size = max_sizes.ClampSizeToMinAndMax(sizes.max_size);
    } else {
        let min_max_sizes = ComputeMinMaxInlineSizes(
            space,
            child,
            &border_padding,
            auto_min_length,
            &min_max_sizes_func,
            TransferredSizesMode::kNormal,
            FitContentMode::kNormal,
            kIndefiniteSize,
        );
        sizes.Constrain(min_max_sizes.max_size);
        sizes.EncompassValue(min_max_sizes.min_size);
    }
    MinMaxSizesResult::new(sizes, depends_on_block_constraints.get())
}

// cpp: layoutng/internal/length_utils.cc:180-201
// cpp: layoutng/internal/length_utils.h:702-706
pub fn ComputeMinAndMaxContentContribution(
    parent_style: &ComputedStyle,
    child: &BlockNode,
    space: &ConstraintSpace,
    float_input: MinMaxSizesFloatInput,
) -> MinMaxSizesResult {
    let parent_writing_mode = parent_style.GetWritingMode();
    let child_writing_mode = child.Style().GetWritingMode();
    if IsParallelWritingMode(parent_writing_mode, child_writing_mode) && child.IsReplaced() {
        return ComputeMinAndMaxContentContributionForReplaced(child, space);
    }
    let min_max_sizes_func =
        |type_: SizeType| child.ComputeMinMaxSizes(parent_writing_mode, type_, space, float_input);
    ComputeMinAndMaxContentContributionInternal(
        parent_writing_mode,
        child,
        space,
        &min_max_sizes_func,
    )
}

// cpp: layoutng/internal/length_utils.cc:203-220
// cpp: layoutng/internal/length_utils.h:711-713
pub fn ComputeMinAndMaxContentContributionForSelf(
    child: &BlockNode,
    space: &ConstraintSpace,
) -> MinMaxSizesResult {
    debug_assert!(child.CreatesNewFormattingContext());
    let writing_mode = child.Style().GetWritingMode();
    if child.IsReplaced() {
        return ComputeMinAndMaxContentContributionForReplaced(child, space);
    }
    let min_max_sizes_func = |type_: SizeType| {
        child.ComputeMinMaxSizes(writing_mode, type_, space, MinMaxSizesFloatInput::default())
    };
    ComputeMinAndMaxContentContributionInternal(writing_mode, child, space, &min_max_sizes_func)
}

// cpp: layoutng/internal/length_utils.cc:222-233
// cpp: layoutng/internal/length_utils.h:716-719
pub fn ComputeMinAndMaxContentContributionForSelfWithCallback(
    child: &BlockNode,
    space: &ConstraintSpace,
    min_max_sizes_func: MinMaxSizesFunctionRef<'_>,
) -> MinMaxSizesResult {
    debug_assert!(child.CreatesNewFormattingContext());
    if child.IsReplaced() {
        ComputeMinAndMaxContentContributionForReplaced(child, space)
    } else {
        ComputeMinAndMaxContentContributionInternal(
            child.Style().GetWritingMode(),
            child,
            space,
            min_max_sizes_func,
        )
    }
}

// cpp: layoutng/internal/length_utils.cc:235-247
// cpp: layoutng/internal/length_utils.h:722-725
pub fn ComputeMinAndMaxContentContributionForTest(
    parent_writing_mode: WritingMode,
    child: &BlockNode,
    space: &ConstraintSpace,
    min_max_sizes: &MinMaxSizes,
) -> MinMaxSizes {
    let min_max_sizes_func = |_: SizeType| MinMaxSizesResult::new(*min_max_sizes, false);
    ComputeMinAndMaxContentContributionInternal(
        parent_writing_mode,
        child,
        space,
        &min_max_sizes_func,
    )
    .sizes
}

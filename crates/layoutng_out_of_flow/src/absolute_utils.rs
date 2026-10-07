#![allow(non_snake_case, non_camel_case_types)]

use crate::oof_dimensions::LogicalOofDimensions;
use foundation::{
    kIndefiniteSize, CalcSizeKeywordBehavior, EOverflow, IsParallelWritingMode, LayoutUnit, Length,
    LogicalToLogical, MinimumValueForLength, PhysicalSize, PhysicalToLogical,
    RuntimeEnabledFeatures, WritingDirectionMode,
};
use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space::{AutoSizeBehavior, ConstraintSpace};
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng_assembly::internal::fragmentation_utils::SetupSpaceBuilderForFragmentationFromSpace;
use layoutng_assembly::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng_assembly::internal::length_utils::{
    ComputeInitialMinMaxBlockSizes, ComputeMinMaxInlineSizes, FitContentMode,
    ResolveMainBlockLength, ResolveMainInlineLength, SizeType, TransferredSizesMode,
};
use layoutng_assembly::layout_result::LayoutResult;
use layoutng_assembly::logical_fragment::LogicalFragment;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_geometry::geometry::static_position::{
    BlockEdge, InlineEdge, LogicalAlignmentDirection, LogicalStaticPosition,
};
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{ItemPosition, OverflowAlignment};
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;
use std::cell::Cell;

// cpp: layoutng_out_of_flow/absolute_utils.h:27-32
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalOofInsets {
    pub inline_start: Option<LayoutUnit>,
    pub inline_end: Option<LayoutUnit>,
    pub block_start: Option<LayoutUnit>,
    pub block_end: Option<LayoutUnit>,
}

// cpp: layoutng_out_of_flow/absolute_utils.h:34-42
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalAlignment {
    pub inline_alignment: StyleSelfAlignmentData,
    pub block_alignment: StyleSelfAlignmentData,
}

impl Default for LogicalAlignment {
    fn default() -> Self {
        let normal = StyleSelfAlignmentData::new_nonlegacy(
            ItemPosition::kNormal,
            OverflowAlignment::kDefault,
        );
        Self {
            inline_alignment: normal,
            block_alignment: normal,
        }
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.h:49-54
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalAnchorCenterPosition {
    pub inline_offset: Option<LayoutUnit>,
    pub block_offset: Option<LayoutUnit>,
}

// cpp: layoutng_out_of_flow/absolute_utils.h:87-87
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsetBias {
    kStart,
    kEnd,
    kEqual,
}

// cpp: layoutng_out_of_flow/absolute_utils.h:66-106
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InsetModifiedContainingBlock {
    pub available_size: LogicalSize,
    pub inline_start: LayoutUnit,
    pub inline_end: LayoutUnit,
    pub block_start: LayoutUnit,
    pub block_end: LayoutUnit,
    pub has_auto_inline_inset: bool,
    pub has_auto_block_inset: bool,
    pub inline_has_default_alignment_overflow: bool,
    pub block_has_default_alignment_overflow: bool,
    pub inline_inset_bias: InsetBias,
    pub block_inset_bias: InsetBias,
    pub inline_safe_inset_bias: Option<InsetBias>,
    pub block_safe_inset_bias: Option<InsetBias>,
    pub inline_default_inset_bias: Option<InsetBias>,
    pub block_default_inset_bias: Option<InsetBias>,
}

impl InsetModifiedContainingBlock {
    // cpp: layoutng_out_of_flow/absolute_utils.h:108-110
    pub fn InlineEndOffset(&self) -> LayoutUnit {
        self.available_size.inline_size - self.inline_end
    }

    // cpp: layoutng_out_of_flow/absolute_utils.h:111-113
    pub fn BlockEndOffset(&self) -> LayoutUnit {
        self.available_size.block_size - self.block_end
    }

    // cpp: layoutng_out_of_flow/absolute_utils.h:114-116
    pub fn InlineSize(&self) -> LayoutUnit {
        self.available_size.inline_size - self.inline_start - self.inline_end
    }

    // cpp: layoutng_out_of_flow/absolute_utils.h:117-119
    pub fn BlockSize(&self) -> LayoutUnit {
        self.available_size.block_size - self.block_start - self.block_end
    }

    // cpp: layoutng_out_of_flow/absolute_utils.h:120-120
    pub fn Size(&self) -> LogicalSize {
        LogicalSize::new(self.InlineSize(), self.BlockSize())
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.h:56-59
// cpp: layoutng_out_of_flow/absolute_utils.cc:542-564
pub fn ComputeAnchorCenterPosition(
    style: &ComputedStyle,
    alignment: &LogicalAlignment,
    available_size: LogicalSize,
) -> LogicalAnchorCenterPosition {
    let Some(physical_offset) = style.AnchorCenterOffset() else {
        return LogicalAnchorCenterPosition::default();
    };
    let converter =
        WritingModeConverter::from_logical_size(style.GetWritingDirection(), available_size);
    let offset = converter.ToLogicalOffset(physical_offset.clone(), PhysicalSize::default());
    LogicalAnchorCenterPosition {
        inline_offset: (alignment.inline_alignment.GetPosition() == ItemPosition::kAnchorCenter)
            .then_some(offset.inline_offset),
        block_offset: (alignment.block_alignment.GetPosition() == ItemPosition::kAnchorCenter)
            .then_some(offset.block_offset),
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.h:61-64
// cpp: layoutng_out_of_flow/absolute_utils.cc:456-493
pub fn ComputeOutOfFlowInsets(
    style: &ComputedStyle,
    available_size: &LogicalSize,
    alignment: &LogicalAlignment,
) -> LogicalOofInsets {
    let force_insets_to_zero = |position: ItemPosition| -> bool {
        style.PositionAreaOffsets().is_some()
            || (style.AnchorCenterOffset().is_some() && position == ItemPosition::kAnchorCenter)
    };
    let force_inline_insets_to_zero =
        force_insets_to_zero(alignment.inline_alignment.GetPosition());
    let force_block_insets_to_zero = force_insets_to_zero(alignment.block_alignment.GetPosition());

    let resolve_inset = |length: &foundation::Length,
                         available_size: LayoutUnit,
                         force_inset_to_zero: bool|
     -> Option<LayoutUnit> {
        if !length.IsAuto() {
            return Some(MinimumValueForLength(length, available_size));
        }
        if force_inset_to_zero {
            return Some(LayoutUnit::default());
        }
        None
    };
    let insets = PhysicalToLogical::new(
        style.GetWritingDirection(),
        style.Top(),
        style.Right(),
        style.Bottom(),
        style.Left(),
    );
    LogicalOofInsets {
        inline_start: resolve_inset(
            insets.InlineStart(),
            available_size.inline_size,
            force_inline_insets_to_zero,
        ),
        inline_end: resolve_inset(
            insets.InlineEnd(),
            available_size.inline_size,
            force_inline_insets_to_zero,
        ),
        block_start: resolve_inset(
            insets.BlockStart(),
            available_size.block_size,
            force_block_insets_to_zero,
        ),
        block_end: resolve_inset(
            insets.BlockEnd(),
            available_size.block_size,
            force_block_insets_to_zero,
        ),
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.h:44-47
// cpp: layoutng_out_of_flow/absolute_utils.cc:495-540
pub fn ComputeAlignment(
    style: &ComputedStyle,
    container_writing_direction: WritingDirectionMode,
    self_writing_direction: WritingDirectionMode,
) -> LogicalAlignment {
    let mut align_normal_behavior =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kNormal, OverflowAlignment::kDefault);
    let mut justify_normal_behavior =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kNormal, OverflowAlignment::kDefault);
    if style.PositionAreaOffsets().is_some() {
        let position_area = style
            .GetPositionArea()
            .ToPhysical(&container_writing_direction, &self_writing_direction);
        debug_assert!(!position_area.IsNone());
        (align_normal_behavior, justify_normal_behavior) =
            position_area.AlignJustifySelfFromPhysical(container_writing_direction);
        let is_auto = PhysicalToLogical::new(
            container_writing_direction,
            style.Top().IsAuto(),
            style.Right().IsAuto(),
            style.Bottom().IsAuto(),
            style.Left().IsAuto(),
        );
        if !is_auto.BlockStart() && is_auto.BlockEnd() {
            align_normal_behavior = StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kStart,
                OverflowAlignment::kUnsafe,
            );
        } else if is_auto.BlockStart() && !is_auto.BlockEnd() {
            align_normal_behavior = StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kEnd,
                OverflowAlignment::kUnsafe,
            );
        }
        if !is_auto.InlineStart() && is_auto.InlineEnd() {
            justify_normal_behavior = StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kStart,
                OverflowAlignment::kUnsafe,
            );
        } else if is_auto.InlineStart() && !is_auto.InlineEnd() {
            justify_normal_behavior = StyleSelfAlignmentData::new_nonlegacy(
                ItemPosition::kEnd,
                OverflowAlignment::kUnsafe,
            );
        }
    }
    let is_parallel = IsParallelWritingMode(
        container_writing_direction.GetWritingMode(),
        self_writing_direction.GetWritingMode(),
    );
    if is_parallel {
        LogicalAlignment {
            inline_alignment: style.ResolvedJustifySelf(&justify_normal_behavior, std::ptr::null()),
            block_alignment: style.ResolvedAlignSelf(&align_normal_behavior, std::ptr::null()),
        }
    } else {
        LogicalAlignment {
            inline_alignment: style.ResolvedAlignSelf(&align_normal_behavior, std::ptr::null()),
            block_alignment: style.ResolvedJustifySelf(&justify_normal_behavior, std::ptr::null()),
        }
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:27-37
fn GetStaticPositionInlineInsetBias(inline_edge: InlineEdge) -> InsetBias {
    match inline_edge {
        InlineEdge::kInlineStart => InsetBias::kStart,
        InlineEdge::kInlineCenter => InsetBias::kEqual,
        InlineEdge::kInlineEnd => InsetBias::kEnd,
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:39-49
fn GetStaticPositionBlockInsetBias(block_edge: BlockEdge) -> InsetBias {
    match block_edge {
        BlockEdge::kBlockStart => InsetBias::kStart,
        BlockEdge::kBlockCenter => InsetBias::kEqual,
        BlockEdge::kBlockEnd => InsetBias::kEnd,
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:51-106
fn GetAlignmentInsetBias(
    alignment: &StyleSelfAlignmentData,
    container_writing_direction: WritingDirectionMode,
    self_writing_direction: WritingDirectionMode,
    is_justify_axis: bool,
    out_has_default_alignment_overflow: &mut bool,
    out_default_inset_bias: &mut InsetBias,
    out_safe_inset_bias: &mut Option<InsetBias>,
) -> InsetBias {
    let bias = LogicalToLogical::new(
        self_writing_direction,
        container_writing_direction,
        InsetBias::kStart,
        InsetBias::kEnd,
        InsetBias::kStart,
        InsetBias::kEnd,
    );
    *out_default_inset_bias = if is_justify_axis {
        bias.InlineStart()
    } else {
        bias.BlockStart()
    };
    if alignment.Overflow() == OverflowAlignment::kSafe {
        *out_safe_inset_bias = Some(if is_justify_axis {
            bias.InlineStart()
        } else {
            bias.BlockStart()
        });
    }
    if alignment.Overflow() == OverflowAlignment::kDefault
        && alignment.GetPosition() != ItemPosition::kNormal
    {
        *out_has_default_alignment_overflow = true;
    }
    match alignment.GetPosition() {
        ItemPosition::kStart
        | ItemPosition::kFlexStart
        | ItemPosition::kBaseline
        | ItemPosition::kStretch
        | ItemPosition::kNormal => {
            if is_justify_axis {
                bias.InlineStart()
            } else {
                bias.BlockStart()
            }
        }
        ItemPosition::kAnchorCenter | ItemPosition::kCenter => InsetBias::kEqual,
        ItemPosition::kEnd | ItemPosition::kFlexEnd | ItemPosition::kLastBaseline => {
            if is_justify_axis {
                bias.InlineEnd()
            } else {
                bias.BlockEnd()
            }
        }
        ItemPosition::kSelfStart => InsetBias::kStart,
        ItemPosition::kSelfEnd => InsetBias::kEnd,
        ItemPosition::kLeft => {
            debug_assert!(is_justify_axis);
            if container_writing_direction.IsLtr() {
                bias.InlineStart()
            } else {
                bias.InlineEnd()
            }
        }
        ItemPosition::kRight => {
            debug_assert!(is_justify_axis);
            if container_writing_direction.IsRtl() {
                bias.InlineStart()
            } else {
                bias.InlineEnd()
            }
        }
        ItemPosition::kLegacy | ItemPosition::kAuto => {
            panic!("legacy or auto alignment is unreachable here")
        }
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:108-124
fn ResizeIMCBInOneAxis(
    inset_bias: InsetBias,
    amount: LayoutUnit,
    inset_start: &mut LayoutUnit,
    inset_end: &mut LayoutUnit,
) {
    match inset_bias {
        InsetBias::kStart => *inset_end += amount,
        InsetBias::kEnd => *inset_start += amount,
        InsetBias::kEqual => {
            *inset_start += amount / 2;
            *inset_end += amount / 2;
        }
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:126-194
fn ComputeUnclampedIMCBInOneAxis(
    available_size: LayoutUnit,
    inset_start: Option<LayoutUnit>,
    inset_end: Option<LayoutUnit>,
    is_static_alignment_parallel: bool,
    static_position_offset: LayoutUnit,
    static_position_inset_bias: InsetBias,
    alignment_inset_bias: InsetBias,
    default_inset_bias: InsetBias,
    safe_inset_bias: Option<InsetBias>,
    alt_safe_inset_bias: Option<InsetBias>,
    imcb_start_out: &mut LayoutUnit,
    imcb_end_out: &mut LayoutUnit,
    inset_bias_out: &mut InsetBias,
    safe_inset_bias_out: &mut Option<InsetBias>,
    default_inset_bias_out: &mut Option<InsetBias>,
) {
    debug_assert_ne!(available_size, kIndefiniteSize);
    if inset_start.is_none() && inset_end.is_none() {
        match static_position_inset_bias {
            InsetBias::kStart => {
                *imcb_start_out = static_position_offset;
                *imcb_end_out = LayoutUnit::default();
            }
            InsetBias::kEqual => {
                let half_size = static_position_offset.min(available_size - static_position_offset);
                *imcb_start_out = static_position_offset - half_size;
                *imcb_end_out = available_size - static_position_offset - half_size;
            }
            InsetBias::kEnd => {
                *imcb_end_out = available_size - static_position_offset;
                *imcb_start_out = LayoutUnit::default();
            }
        }
        *inset_bias_out = static_position_inset_bias;
        *safe_inset_bias_out = if is_static_alignment_parallel {
            safe_inset_bias
        } else {
            alt_safe_inset_bias
        };
    } else {
        *imcb_start_out = inset_start.unwrap_or_default();
        *imcb_end_out = inset_end.unwrap_or_default();
        if inset_start.is_none() || inset_end.is_none() {
            *inset_bias_out = if inset_start.is_some() {
                InsetBias::kStart
            } else {
                InsetBias::kEnd
            };
        } else {
            *inset_bias_out = alignment_inset_bias;
            *safe_inset_bias_out = safe_inset_bias;
            *default_inset_bias_out = Some(default_inset_bias);
        }
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:196-252
fn ComputeUnclampedIMCB(
    available_size: &LogicalSize,
    alignment: &LogicalAlignment,
    insets: &LogicalOofInsets,
    static_position: &LogicalStaticPosition,
    _style: &ComputedStyle,
    container_writing_direction: WritingDirectionMode,
    self_writing_direction: WritingDirectionMode,
) -> InsetModifiedContainingBlock {
    let mut imcb = InsetModifiedContainingBlock {
        available_size: *available_size,
        inline_start: LayoutUnit::default(),
        inline_end: LayoutUnit::default(),
        block_start: LayoutUnit::default(),
        block_end: LayoutUnit::default(),
        has_auto_inline_inset: insets.inline_start.is_none() || insets.inline_end.is_none(),
        has_auto_block_inset: insets.block_start.is_none() || insets.block_end.is_none(),
        inline_has_default_alignment_overflow: false,
        block_has_default_alignment_overflow: false,
        inline_inset_bias: InsetBias::kStart,
        block_inset_bias: InsetBias::kStart,
        inline_safe_inset_bias: None,
        block_safe_inset_bias: None,
        inline_default_inset_bias: None,
        block_default_inset_bias: None,
    };
    let is_parallel = IsParallelWritingMode(
        container_writing_direction.GetWritingMode(),
        self_writing_direction.GetWritingMode(),
    );
    let mut inline_default_inset_bias = InsetBias::kStart;
    let mut inline_safe_inset_bias = None;
    let inline_alignment_inset_bias = GetAlignmentInsetBias(
        &alignment.inline_alignment,
        container_writing_direction,
        self_writing_direction,
        is_parallel,
        &mut imcb.inline_has_default_alignment_overflow,
        &mut inline_default_inset_bias,
        &mut inline_safe_inset_bias,
    );
    let mut block_default_inset_bias = InsetBias::kStart;
    let mut block_safe_inset_bias = None;
    let block_alignment_inset_bias = GetAlignmentInsetBias(
        &alignment.block_alignment,
        container_writing_direction,
        self_writing_direction,
        !is_parallel,
        &mut imcb.block_has_default_alignment_overflow,
        &mut block_default_inset_bias,
        &mut block_safe_inset_bias,
    );
    let is_static_alignment_parallel =
        static_position.align_self_direction == LogicalAlignmentDirection::kBlock;
    ComputeUnclampedIMCBInOneAxis(
        available_size.inline_size,
        insets.inline_start,
        insets.inline_end,
        is_static_alignment_parallel,
        static_position.offset.inline_offset,
        GetStaticPositionInlineInsetBias(static_position.inline_edge),
        inline_alignment_inset_bias,
        inline_default_inset_bias,
        inline_safe_inset_bias,
        block_safe_inset_bias,
        &mut imcb.inline_start,
        &mut imcb.inline_end,
        &mut imcb.inline_inset_bias,
        &mut imcb.inline_safe_inset_bias,
        &mut imcb.inline_default_inset_bias,
    );
    ComputeUnclampedIMCBInOneAxis(
        available_size.block_size,
        insets.block_start,
        insets.block_end,
        is_static_alignment_parallel,
        static_position.offset.block_offset,
        GetStaticPositionBlockInsetBias(static_position.block_edge),
        block_alignment_inset_bias,
        block_default_inset_bias,
        block_safe_inset_bias,
        inline_safe_inset_bias,
        &mut imcb.block_start,
        &mut imcb.block_end,
        &mut imcb.block_inset_bias,
        &mut imcb.block_safe_inset_bias,
        &mut imcb.block_default_inset_bias,
    );
    imcb
}

// cpp: layoutng_out_of_flow/absolute_utils.h:126-133
// cpp: layoutng_out_of_flow/absolute_utils.cc:566-604
pub fn ComputeInsetModifiedContainingBlock(
    node: &BlockNode,
    available_size: &LogicalSize,
    alignment: &LogicalAlignment,
    insets: &LogicalOofInsets,
    static_position: &LogicalStaticPosition,
    container_writing_direction: WritingDirectionMode,
    self_writing_direction: WritingDirectionMode,
) -> InsetModifiedContainingBlock {
    let mut imcb = ComputeUnclampedIMCB(
        available_size,
        alignment,
        insets,
        static_position,
        node.Style(),
        container_writing_direction,
        self_writing_direction,
    );
    if imcb.InlineSize() < LayoutUnit::default() {
        ResizeIMCBInOneAxis(
            imcb.inline_default_inset_bias
                .unwrap_or(imcb.inline_inset_bias),
            imcb.InlineSize(),
            &mut imcb.inline_start,
            &mut imcb.inline_end,
        );
    }
    if imcb.BlockSize() < LayoutUnit::default() {
        ResizeIMCBInOneAxis(
            imcb.block_default_inset_bias
                .unwrap_or(imcb.block_inset_bias),
            imcb.BlockSize(),
            &mut imcb.block_start,
            &mut imcb.block_end,
        );
    }
    if node.IsTable() {
        if imcb.InlineSize() > available_size.inline_size {
            ResizeIMCBInOneAxis(
                imcb.inline_default_inset_bias
                    .unwrap_or(imcb.inline_inset_bias),
                imcb.InlineSize() - available_size.inline_size,
                &mut imcb.inline_start,
                &mut imcb.inline_end,
            );
        }
        if imcb.BlockSize() > available_size.block_size {
            ResizeIMCBInOneAxis(
                imcb.block_default_inset_bias
                    .unwrap_or(imcb.block_inset_bias),
                imcb.BlockSize() - available_size.block_size,
                &mut imcb.block_start,
                &mut imcb.block_end,
            );
        }
    }
    imcb
}

// cpp: layoutng_out_of_flow/absolute_utils.h:139-146
// cpp: layoutng_out_of_flow/absolute_utils.cc:606-617
pub fn ComputeIMCBForPositionFallback(
    available_size: &LogicalSize,
    alignment: &LogicalAlignment,
    insets: &LogicalOofInsets,
    static_position: &LogicalStaticPosition,
    style: &ComputedStyle,
    container_writing_direction: WritingDirectionMode,
    self_writing_direction: WritingDirectionMode,
) -> InsetModifiedContainingBlock {
    ComputeUnclampedIMCB(
        available_size,
        alignment,
        insets,
        static_position,
        style,
        container_writing_direction,
        self_writing_direction,
    )
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:254-321
fn ComputeMargins(
    margin_percentage_resolution_size: LogicalSize,
    imcb_size: LayoutUnit,
    margin_start_length: &Length,
    margin_end_length: &Length,
    size: LayoutUnit,
    has_auto_inset: bool,
    has_anchor_positioning: bool,
    is_start_dominant: bool,
    is_block_direction: bool,
    margin_start_out: &mut LayoutUnit,
    margin_end_out: &mut LayoutUnit,
) -> bool {
    let mut margin_start = if !margin_start_length.IsAuto() {
        Some(MinimumValueForLength(
            margin_start_length,
            margin_percentage_resolution_size.inline_size,
        ))
    } else {
        None
    };
    let mut margin_end = if !margin_end_length.IsAuto() {
        Some(MinimumValueForLength(
            margin_end_length,
            margin_percentage_resolution_size.inline_size,
        ))
    } else {
        None
    };
    if has_auto_inset || has_anchor_positioning || (margin_start.is_some() && margin_end.is_some())
    {
        *margin_start_out = margin_start.unwrap_or_default();
        *margin_end_out = margin_end.unwrap_or_default();
        return false;
    }
    let free_space =
        imcb_size - size - margin_start.unwrap_or_default() - margin_end.unwrap_or_default();
    if margin_start.is_none() && margin_end.is_none() {
        if free_space > LayoutUnit::default() || is_block_direction {
            margin_start = Some(free_space / 2);
            margin_end = Some(free_space - margin_start.unwrap());
        } else if is_start_dominant {
            margin_start = Some(LayoutUnit::default());
            margin_end = Some(free_space);
        } else {
            margin_start = Some(free_space);
            margin_end = Some(LayoutUnit::default());
        }
    } else if margin_start.is_none() {
        margin_start = Some(free_space);
    } else if margin_end.is_none() {
        margin_end = Some(free_space);
    }
    *margin_start_out = margin_start.expect("auto margin resolved");
    *margin_end_out = margin_end.expect("auto margin resolved");
    true
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:323-425
#[allow(clippy::too_many_arguments)]
fn ComputeInsets(
    available_size: LayoutUnit,
    overflow_limit_start: LayoutUnit,
    overflow_limit_end: LayoutUnit,
    original_imcb_start: LayoutUnit,
    original_imcb_end: LayoutUnit,
    has_default_alignment_overflow: bool,
    inset_bias: InsetBias,
    safe_inset_bias: Option<InsetBias>,
    default_inset_bias: Option<InsetBias>,
    margin_start: LayoutUnit,
    margin_end: LayoutUnit,
    size: LayoutUnit,
    anchor_center_offset: Option<LayoutUnit>,
    inset_start_out: &mut LayoutUnit,
    inset_end_out: &mut LayoutUnit,
) {
    debug_assert_ne!(available_size, kIndefiniteSize);
    let margin_box_size = margin_start + size + margin_end;
    let mut imcb_start = original_imcb_start;
    let mut imcb_end = original_imcb_end;
    if let Some(anchor_center_offset) = anchor_center_offset {
        let half_size = if safe_inset_bias.unwrap_or(InsetBias::kStart) == InsetBias::kStart {
            anchor_center_offset - imcb_start
        } else {
            available_size - anchor_center_offset - imcb_end
        };
        imcb_start = anchor_center_offset - half_size;
        imcb_end = available_size - anchor_center_offset - half_size;
    }
    let mut free_space = available_size - imcb_start - imcb_end - margin_box_size;
    let mut bias = inset_bias;
    let apply_safe_bias = safe_inset_bias.is_some() && free_space < LayoutUnit::default();
    if apply_safe_bias {
        free_space = LayoutUnit::default();
        bias = safe_inset_bias.expect("safe inset bias");
    }
    ResizeIMCBInOneAxis(bias, free_space, &mut imcb_start, &mut imcb_end);
    if has_default_alignment_overflow && default_inset_bias.is_some() && !apply_safe_bias {
        let use_imcb = margin_box_size <= available_size - original_imcb_start - original_imcb_end;
        let adjust_start = |start: &mut LayoutUnit, end: &mut LayoutUnit| {
            let safe_start = if use_imcb {
                original_imcb_start
            } else {
                original_imcb_start.min(overflow_limit_start)
            };
            if *start < safe_start {
                *end += *start - safe_start;
                *start = safe_start;
            }
        };
        let adjust_end = |start: &mut LayoutUnit, end: &mut LayoutUnit| {
            let safe_end = if use_imcb {
                original_imcb_end
            } else {
                original_imcb_end.min(overflow_limit_end)
            };
            if *end < safe_end {
                *start += *end - safe_end;
                *end = safe_end;
            }
        };
        if default_inset_bias == Some(InsetBias::kStart) {
            adjust_end(&mut imcb_start, &mut imcb_end);
            adjust_start(&mut imcb_start, &mut imcb_end);
        } else {
            adjust_start(&mut imcb_start, &mut imcb_end);
            adjust_end(&mut imcb_start, &mut imcb_end);
        }
    }
    *inset_start_out = imcb_start + margin_start;
    *inset_end_out = imcb_end + margin_end;
}

// cpp: layoutng_out_of_flow/absolute_utils.cc:427-452
fn CanComputeBlockSizeWithoutLayout(
    node: &BlockNode,
    block_auto_size_behavior: AutoSizeBehavior,
) -> bool {
    if node.IsTable() {
        return false;
    }
    let style = node.Style();
    if style.LogicalHeight().HasContentOrIntrinsic()
        || style.LogicalMinHeight().HasContentOrIntrinsic()
        || style.LogicalMaxHeight().HasContentOrIntrinsic()
    {
        return false;
    }
    if !style.LogicalHeight().HasAuto() {
        return true;
    }
    match block_auto_size_behavior {
        AutoSizeBehavior::kFitContent => false,
        AutoSizeBehavior::kStretchExplicit => true,
        AutoSizeBehavior::kStretchImplicit => style.AspectRatio().IsAuto(),
    }
}

// cpp: layoutng_out_of_flow/absolute_utils.h:158-172
// cpp: layoutng_out_of_flow/absolute_utils.cc:619-753
#[allow(clippy::too_many_arguments)]
pub fn ComputeOofInlineDimensions(
    node: &BlockNode,
    _break_token: *const BlockBreakToken,
    style: &ComputedStyle,
    space: &ConstraintSpace,
    imcb: &InsetModifiedContainingBlock,
    anchor_center_position: &LogicalAnchorCenterPosition,
    _alignment: &LogicalAlignment,
    border_padding: &BoxStrut,
    replaced_size: Option<LogicalSize>,
    overflow_limit_insets: &BoxStrut,
    inline_auto_size_behavior: AutoSizeBehavior,
    block_auto_size_behavior: AutoSizeBehavior,
    container_writing_direction: WritingDirectionMode,
    dimensions: &mut LogicalOofDimensions,
) -> bool {
    debug_assert!(imcb.InlineSize() >= LayoutUnit::default());
    let depends_on_min_max_sizes = Cell::new(false);
    let min_max_sizes_func = |type_: SizeType| {
        debug_assert!(!node.IsReplaced());
        depends_on_min_max_sizes.set(true);
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            style.GetWritingMode(),
            style.GetWritingDirection(),
            true,
            false,
            false,
        );
        builder.SetAvailableSize(imcb.Size());
        builder.SetPercentageResolutionSize(space.PercentageResolutionSize());
        builder.SetBlockAutoBehavior(block_auto_size_behavior);
        node.ComputeMinMaxSizes(
            style.GetWritingMode(),
            type_,
            &builder.ToConstraintSpace(),
            MinMaxSizesFloatInput::default(),
        )
    };
    let inline_size = if let Some(replaced_size) = replaced_size {
        debug_assert!(node.IsReplaced());
        replaced_size.inline_size
    } else {
        let may_apply_aspect_ratio = || -> bool {
            if style.AspectRatio().IsAuto() {
                return false;
            }
            let auto_length = if block_auto_size_behavior == AutoSizeBehavior::kFitContent {
                Length::FitContent()
            } else {
                Length::Stretch()
            };
            ResolveMainBlockLength(
                space,
                style,
                border_padding,
                style.LogicalHeight(),
                Some(auto_length),
                &|_| kIndefiniteSize,
                imcb.BlockSize(),
            ) != kIndefiniteSize
        };
        let mut apply_automatic_min_size = false;
        let auto_length = match inline_auto_size_behavior {
            AutoSizeBehavior::kFitContent => {
                if may_apply_aspect_ratio()
                    && style.OverflowInlineDirection() == EOverflow::kVisible
                {
                    apply_automatic_min_size = true;
                }
                Length::FitContent()
            }
            AutoSizeBehavior::kStretchExplicit => Length::Stretch(),
            AutoSizeBehavior::kStretchImplicit => {
                let is_block_explicit = !style.LogicalHeight().HasAuto()
                    || block_auto_size_behavior == AutoSizeBehavior::kStretchExplicit;
                if is_block_explicit && may_apply_aspect_ratio() {
                    if style.OverflowInlineDirection() == EOverflow::kVisible {
                        apply_automatic_min_size = true;
                    }
                    Length::FitContent()
                } else {
                    Length::Stretch()
                }
            }
        };
        let main_inline_size = ResolveMainInlineLength(
            space,
            style,
            border_padding,
            &min_max_sizes_func,
            style.LogicalWidth(),
            Some(auto_length),
            imcb.InlineSize(),
            CalcSizeKeywordBehavior::kAsSpecified,
        );
        let min_max_inline_sizes = ComputeMinMaxInlineSizes(
            space,
            node,
            border_padding,
            if apply_automatic_min_size {
                Some(Length::MinIntrinsic())
            } else {
                None
            },
            &min_max_sizes_func,
            TransferredSizesMode::kNormal,
            FitContentMode::kNormal,
            imcb.InlineSize(),
        );
        min_max_inline_sizes.ClampSizeToMinAndMax(main_inline_size)
    };
    dimensions.size.inline_size = inline_size;
    let is_margin_start_dominant = LogicalToLogical::new(
        container_writing_direction,
        style.GetWritingDirection(),
        true,
        false,
        true,
        false,
    )
    .InlineStart();
    let is_block_direction = !IsParallelWritingMode(
        container_writing_direction.GetWritingMode(),
        style.GetWritingMode(),
    );
    let has_anchor_positioning =
        style.PositionAreaOffsets().is_some() || anchor_center_position.inline_offset.is_some();
    let applied_auto_margins = ComputeMargins(
        space.MarginPaddingPercentageResolutionSize(),
        imcb.InlineSize(),
        style.MarginInlineStart(),
        style.MarginInlineEnd(),
        inline_size,
        imcb.has_auto_inline_inset,
        has_anchor_positioning,
        is_margin_start_dominant,
        is_block_direction,
        &mut dimensions.margins.inline_start,
        &mut dimensions.margins.inline_end,
    );
    if applied_auto_margins {
        dimensions.inset.inline_start = imcb.inline_start + dimensions.margins.inline_start;
        dimensions.inset.inline_end = imcb.inline_end + dimensions.margins.inline_end;
    } else {
        ComputeInsets(
            space.AvailableSize().inline_size,
            overflow_limit_insets.inline_start,
            overflow_limit_insets.inline_end,
            imcb.inline_start,
            imcb.inline_end,
            imcb.inline_has_default_alignment_overflow,
            imcb.inline_inset_bias,
            imcb.inline_safe_inset_bias,
            imcb.inline_default_inset_bias,
            dimensions.margins.inline_start,
            dimensions.margins.inline_end,
            inline_size,
            anchor_center_position.inline_offset,
            &mut dimensions.inset.inline_start,
            &mut dimensions.inset.inline_end,
        );
    }
    depends_on_min_max_sizes.get()
}

// cpp: layoutng_out_of_flow/absolute_utils.h:176-190
// cpp: layoutng_out_of_flow/absolute_utils.cc:755-874
#[allow(clippy::too_many_arguments)]
pub fn ComputeOofBlockDimensions(
    node: &BlockNode,
    break_token: *const BlockBreakToken,
    style: &ComputedStyle,
    space: &ConstraintSpace,
    imcb: &InsetModifiedContainingBlock,
    anchor_center_position: &LogicalAnchorCenterPosition,
    _alignment: &LogicalAlignment,
    border_padding: &BoxStrut,
    replaced_size: Option<LogicalSize>,
    overflow_limit_insets: &BoxStrut,
    block_auto_size_behavior: AutoSizeBehavior,
    container_writing_direction: WritingDirectionMode,
    dimensions: &mut LogicalOofDimensions,
) -> *const LayoutResult {
    debug_assert!(imcb.BlockSize() >= LayoutUnit::default());
    let mut result: *const LayoutResult = std::ptr::null();
    let block_size = if let Some(replaced_size) = replaced_size {
        debug_assert!(node.IsReplaced());
        replaced_size.block_size
    } else if CanComputeBlockSizeWithoutLayout(node, block_auto_size_behavior) {
        debug_assert!(!node.IsTable());
        let main_block_size = ResolveMainBlockLength(
            space,
            style,
            border_padding,
            style.LogicalHeight(),
            Some(Length::Stretch()),
            &|_| kIndefiniteSize,
            imcb.BlockSize(),
        );
        let min_max_block_sizes =
            ComputeInitialMinMaxBlockSizes(space, node, border_padding, imcb.BlockSize());
        min_max_block_sizes.ClampSizeToMinAndMax(main_block_size)
    } else {
        debug_assert_ne!(dimensions.size.inline_size, kIndefiniteSize);
        let force_orthogonal_writing_mode_root = !IsParallelWritingMode(
            container_writing_direction.GetWritingMode(),
            style.GetWritingMode(),
        );
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            style.GetWritingMode(),
            style.GetWritingDirection(),
            true,
            true,
            force_orthogonal_writing_mode_root,
        );
        builder.SetAvailableSize(LogicalSize::new(
            dimensions.size.inline_size,
            imcb.BlockSize(),
        ));
        builder.SetIsFixedInlineSize(true);
        builder.SetPercentageResolutionSize(space.PercentageResolutionSize());
        if space.IsHiddenForPaint() {
            builder.SetIsHiddenForPaint(true);
        }
        builder.SetBlockAutoBehavior(block_auto_size_behavior);
        if space.IsInitialColumnBalancingPass() {
            SetupSpaceBuilderForFragmentationFromSpace(
                space,
                node,
                LayoutUnit::default(),
                space.FragmentainerBlockSize(),
                false,
                &mut builder,
            );
        }
        let _disable_side_effects = if !break_token.is_null()
            && !unsafe { &*break_token }.IsBreakBefore()
            && RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
        {
            Some(DisableLayoutSideEffectsScope::new())
        } else {
            None
        };
        result = node.Layout(
            &builder.ToConstraintSpace(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        LogicalFragment::new(
            style.GetWritingDirection(),
            unsafe { &*result }.GetPhysicalFragment(),
        )
        .BlockSize()
    };
    dimensions.size.block_size = block_size;
    let is_margin_start_dominant = LogicalToLogical::new(
        container_writing_direction,
        style.GetWritingDirection(),
        true,
        false,
        true,
        false,
    )
    .BlockStart();
    let is_block_direction = IsParallelWritingMode(
        container_writing_direction.GetWritingMode(),
        style.GetWritingMode(),
    );
    let has_anchor_positioning =
        style.PositionAreaOffsets().is_some() || anchor_center_position.block_offset.is_some();
    let applied_auto_margins = ComputeMargins(
        space.MarginPaddingPercentageResolutionSize(),
        imcb.BlockSize(),
        style.MarginBlockStart(),
        style.MarginBlockEnd(),
        block_size,
        imcb.has_auto_block_inset,
        has_anchor_positioning,
        is_margin_start_dominant,
        is_block_direction,
        &mut dimensions.margins.block_start,
        &mut dimensions.margins.block_end,
    );
    if applied_auto_margins {
        dimensions.inset.block_start = imcb.block_start + dimensions.margins.block_start;
        dimensions.inset.block_end = imcb.block_end + dimensions.margins.block_end;
    } else {
        ComputeInsets(
            space.AvailableSize().block_size,
            overflow_limit_insets.block_start,
            overflow_limit_insets.block_end,
            imcb.block_start,
            imcb.block_end,
            imcb.block_has_default_alignment_overflow,
            imcb.block_inset_bias,
            imcb.block_safe_inset_bias,
            imcb.block_default_inset_bias,
            dimensions.margins.block_start,
            dimensions.margins.block_end,
            block_size,
            anchor_center_position.block_offset,
            &mut dimensions.inset.block_start,
            &mut dimensions.inset.block_end,
        );
    }
    result
}

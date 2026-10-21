#![allow(non_snake_case)]

use foundation::{
    kIndefiniteSize, AtomicString, DynamicTo, EBoxDecorationBreak, EBreakBetween, EBreakInside,
    EClear, LayoutUnit, MakeGarbageCollected, Member, PhysicalOffset, PhysicalSize, To, Visitor,
    WritingDirectionMode,
};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::break_token::BreakToken;
use layoutng_fragment_tree::break_token_algorithm_data::BreakTokenAlgorithmData;
use layoutng_fragment_tree::fragment_builder::FragmentBuilder;
use layoutng_fragment_tree::layout_result::{EStatus, LayoutResult};
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::box_sides::LogicalBoxSides;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize, ToPhysicalSize};
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

use super::block_node::BlockNode;
use super::break_appeal::BreakAppeal;
use super::column_spanner_path::ColumnSpannerPath;
use super::constraint_space::{AutoSizeBehavior, ConstraintSpace, FragmentationType};
use super::constraint_space_builder::ConstraintSpaceBuilder;
use super::early_break::{BreakType, EarlyBreak};
use super::layout_box::LayoutBox;
use super::layout_input_node::LayoutInputNode;
use super::layout_node_metadata::Element;
use super::layout_object::LayoutObject;
use super::length_utils::ComputeBlockSizeForFragment;
use super::space_utils::SetTextBoxTrimOnChildSpaceBuilder;

// cpp: layoutng/internal/fragmentation_utils.h:27-37
#[repr(C)]
pub struct FlexColumnBreakInfo {
    pub column_intrinsic_block_size: LayoutUnit,
    pub early_break: Member<EarlyBreak>,
    pub break_after: EBreakBetween,
}

impl Default for FlexColumnBreakInfo {
    fn default() -> Self {
        Self {
            column_intrinsic_block_size: LayoutUnit::default(),
            early_break: Member::default(),
            break_after: EBreakBetween::kAuto,
        }
    }
}

impl FlexColumnBreakInfo {
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.early_break);
    }
}

// cpp: layoutng/internal/fragmentation_utils.cc:29-60
fn FragmentainerBreakPrecedence(value: EBreakBetween) -> i32 {
    match value {
        EBreakBetween::kAuto => 0,
        EBreakBetween::kAvoidColumn => 1,
        EBreakBetween::kAvoidPage => 2,
        EBreakBetween::kAvoid => 3,
        EBreakBetween::kColumn => 4,
        EBreakBetween::kPage => 5,
        EBreakBetween::kLeft
        | EBreakBetween::kRight
        | EBreakBetween::kRecto
        | EBreakBetween::kVerso => 6,
        _ => unreachable!("unsupported break-between value"),
    }
}

// cpp: layoutng/internal/fragmentation_utils.cc:62-79
fn ShouldCloneBlockStartBorderPadding(builder: &BoxFragmentBuilder) -> bool {
    if builder.Node().Style().BoxDecorationBreak() != EBoxDecorationBreak::kClone {
        return false;
    }
    let previous_break_token = builder.PreviousBreakToken();
    if previous_break_token.is_null() {
        return true;
    }
    let previous_break_token = unsafe { &*previous_break_token };
    if previous_break_token.MonolithicOverflow() != LayoutUnit::default() {
        let space_left = FragmentainerSpaceLeft(builder, false);
        if space_left < builder.BorderScrollbarPadding().BlockSum() {
            return false;
        }
    }
    !previous_break_token.IsAtBlockEnd()
}

// cpp: layoutng/internal/fragmentation_utils.h:49-50
// cpp: layoutng/internal/fragmentation_utils.cc:83-89
pub fn JoinFragmentainerBreakValues(first: EBreakBetween, second: EBreakBetween) -> EBreakBetween {
    if FragmentainerBreakPrecedence(second) >= FragmentainerBreakPrecedence(first) {
        return second;
    }
    first
}

// cpp: layoutng/internal/fragmentation_utils.h:54
// cpp: layoutng/internal/fragmentation_utils.cc:91-107
pub fn IsForcedBreakValue(space: &ConstraintSpace, value: EBreakBetween) -> bool {
    if space.ShouldIgnoreForcedBreaks() {
        return false;
    }
    if value == EBreakBetween::kColumn {
        return space.BlockFragmentationType() == FragmentationType::kFragmentColumn;
    }
    if matches!(
        value,
        EBreakBetween::kLeft
            | EBreakBetween::kPage
            | EBreakBetween::kRecto
            | EBreakBetween::kRight
            | EBreakBetween::kVerso
    ) {
        return space.BlockFragmentationType() == FragmentationType::kFragmentPage;
    }
    false
}

// cpp: layoutng/internal/fragmentation_utils.h:58-59
pub trait AvoidBreakProperty: PartialEq + Copy {
    const AVOID: Self;
    const AVOID_COLUMN: Self;
    const AVOID_PAGE: Self;
}

impl AvoidBreakProperty for EBreakBetween {
    const AVOID: Self = Self::kAvoid;
    const AVOID_COLUMN: Self = Self::kAvoidColumn;
    const AVOID_PAGE: Self = Self::kAvoidPage;
}

impl AvoidBreakProperty for EBreakInside {
    const AVOID: Self = Self::kAvoid;
    const AVOID_COLUMN: Self = Self::kAvoidColumn;
    const AVOID_PAGE: Self = Self::kAvoidPage;
}

// cpp: layoutng/internal/fragmentation_utils.cc:109-129
pub fn IsAvoidBreakValue<Property: AvoidBreakProperty>(
    space: &ConstraintSpace,
    value: Property,
) -> bool {
    if value == Property::AVOID {
        return space.HasBlockFragmentation();
    }
    if value == Property::AVOID_COLUMN {
        return space.BlockFragmentationType() == FragmentationType::kFragmentColumn;
    }
    if value == Property::AVOID_PAGE {
        return space.BlockFragmentationType() == FragmentationType::kFragmentPage;
    }
    false
}

// cpp: layoutng/internal/fragmentation_utils.h:63-65
pub fn IsBreakInside(token: *const BlockBreakToken) -> bool {
    !token.is_null() && !unsafe { &*token }.IsBreakBefore() && !unsafe { &*token }.IsRepeated()
}

// cpp: layoutng/internal/fragmentation_utils.h:73-77
pub fn InvolvedInBlockFragmentation(
    space: &ConstraintSpace,
    previous_break_token: *const BlockBreakToken,
) -> bool {
    space.HasBlockFragmentation() || IsBreakInside(previous_break_token)
}

// cpp: layoutng/internal/fragmentation_utils.h:78-81
pub fn InvolvedInBlockFragmentationForBuilder(builder: &BoxFragmentBuilder) -> bool {
    InvolvedInBlockFragmentation(builder.GetConstraintSpace(), builder.PreviousBreakToken())
}

// cpp: layoutng/internal/fragmentation_utils.h:85-89
pub fn FragmentIndex(incoming_break_token: *const BlockBreakToken) -> usize {
    if !incoming_break_token.is_null() && !unsafe { &*incoming_break_token }.IsBreakBefore() {
        return unsafe { &*incoming_break_token }.SequenceNumber() as usize + 1;
    }
    0
}

// cpp: layoutng/internal/fragmentation_utils.h:94-96
// cpp: layoutng/internal/fragmentation_utils.cc:131-176
pub fn CalculateBreakBetweenValue(
    child: LayoutInputNode,
    result: &LayoutResult,
    builder: &BoxFragmentBuilder,
) -> EBreakBetween {
    if child.IsInline() {
        return builder.JoinedBreakBetweenValue(result.InitialBreakBefore());
    }
    let mut box_fragment: *const PhysicalBoxFragment = std::ptr::null();
    if result.Status() == EStatus::kSuccess {
        box_fragment = DynamicTo::<PhysicalBoxFragment>(
            result.GetPhysicalFragment() as *const PhysicalFragment
        );
        assert!(!box_fragment.is_null());
        if !unsafe { &*box_fragment }.IsFirstForNode() {
            return EBreakBetween::kAuto;
        }
    }
    let mut break_before =
        JoinFragmentainerBreakValues(child.Style().BreakBefore(), result.InitialBreakBefore());
    break_before = builder.JoinedBreakBetweenValue(break_before);
    let space = builder.GetConstraintSpace();
    if space.IsPaginated() && !box_fragment.is_null() && !IsForcedBreakValue(space, break_before) {
        let mut current_name = builder.PageName().clone();
        if current_name.IsNull() {
            current_name = space.PageName();
        }
        let child_page_name = PageNameForChildFragment(builder, unsafe { &*box_fragment });
        if child_page_name != current_name {
            return EBreakBetween::kPage;
        }
    }
    break_before
}

// cpp: layoutng/internal/fragmentation_utils.h:103-104
// cpp: layoutng/internal/fragmentation_utils.cc:178-187
pub fn PageNameForChildFragment(
    builder: &BoxFragmentBuilder,
    child: &PhysicalBoxFragment,
) -> AtomicString {
    let propagated_name = child.PropagatedPageName();
    if !propagated_name.IsNull() {
        return propagated_name;
    }
    let local_name = child.Style().Page();
    if !local_name.IsNull() {
        return local_name.clone();
    }
    builder.GetConstraintSpace().PageName()
}

// cpp: layoutng/internal/fragmentation_utils.h:108
// cpp: layoutng/internal/fragmentation_utils.cc:189-197
pub fn ShouldAvoidBreakInside(space: &ConstraintSpace, result: &LayoutResult) -> bool {
    let fragment = result.GetPhysicalFragment();
    if fragment.IsMonolithic() {
        return true;
    }
    fragment.IsBox() && IsAvoidBreakValue(space, fragment.Style().BreakInside())
}

// cpp: layoutng/internal/fragmentation_utils.h:120-123
// cpp: layoutng/internal/fragmentation_utils.cc:199-213
pub fn IsBreakableAtStartOfResumedContainerWithResult(
    space: &ConstraintSpace,
    result: &LayoutResult,
    builder: &BoxFragmentBuilder,
) -> bool {
    if result.Status() != EStatus::kSuccess {
        return false;
    }
    let mut is_first_for_node = true;
    let box_fragment =
        DynamicTo::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const PhysicalFragment);
    if !box_fragment.is_null() {
        is_first_for_node = unsafe { &*box_fragment }.IsFirstForNode();
    }
    IsBreakableAtStartOfResumedContainer(space, builder, is_first_for_node)
}

// cpp: layoutng/internal/fragmentation_utils.h:125-127
// cpp: layoutng/internal/fragmentation_utils.cc:215-220
pub fn IsBreakableAtStartOfResumedContainer(
    space: &ConstraintSpace,
    builder: &BoxFragmentBuilder,
    is_first_for_node: bool,
) -> bool {
    space.MinBreakAppeal() != BreakAppeal::kBreakAppealLastResort
        && IsBreakInside(builder.PreviousBreakToken())
        && is_first_for_node
}

// cpp: layoutng/internal/fragmentation_utils.h:130-134
// cpp: layoutng/internal/fragmentation_utils.cc:222-234
pub fn CalculateBreakAppealBeforeChild(
    space: &ConstraintSpace,
    child: LayoutInputNode,
    result: &LayoutResult,
    builder: &BoxFragmentBuilder,
    has_container_separation: bool,
) -> BreakAppeal {
    let breakable_at_start_of_container =
        IsBreakableAtStartOfResumedContainerWithResult(space, result, builder);
    let break_between = CalculateBreakBetweenValue(child, result, builder);
    CalculateBreakAppealBeforeStatus(
        space,
        result.Status(),
        break_between,
        has_container_separation,
        breakable_at_start_of_container,
    )
}

// cpp: layoutng/internal/fragmentation_utils.h:135-140
// cpp: layoutng/internal/fragmentation_utils.cc:236-273
pub fn CalculateBreakAppealBeforeStatus(
    space: &ConstraintSpace,
    status: EStatus,
    break_between: EBreakBetween,
    has_container_separation: bool,
    breakable_at_start_of_container: bool,
) -> BreakAppeal {
    debug_assert!(status == EStatus::kSuccess || status == EStatus::kOutOfFragmentainerSpace);
    let mut break_appeal = BreakAppeal::kBreakAppealPerfect;
    if !has_container_separation && status == EStatus::kSuccess {
        if !breakable_at_start_of_container {
            return BreakAppeal::kBreakAppealLastResort;
        }
        break_appeal = space.MinBreakAppeal();
    }
    if IsAvoidBreakValue(space, break_between) {
        break_appeal = std::cmp::min(break_appeal, BreakAppeal::kBreakAppealViolatingBreakAvoid);
    }
    break_appeal
}

// cpp: layoutng/internal/fragmentation_utils.h:146-149
// cpp: layoutng/internal/fragmentation_utils.cc:275-309
pub fn CalculateBreakAppealInside(
    space: &ConstraintSpace,
    result: &LayoutResult,
    hypothetical_appeal: Option<BreakAppeal>,
) -> BreakAppeal {
    if result.HasForcedBreak() {
        return BreakAppeal::kBreakAppealPerfect;
    }
    let physical_fragment = result.GetPhysicalFragment();
    let break_token = DynamicTo::<BlockBreakToken>(physical_fragment.GetBreakToken());
    let (mut appeal, consider_break_inside_avoidance) =
        if let Some(hypothetical_appeal) = hypothetical_appeal {
            debug_assert!(break_token.is_null());
            (hypothetical_appeal, true)
        } else {
            (result.GetBreakAppeal(), IsBreakInside(break_token))
        };
    if consider_break_inside_avoidance
        && appeal > BreakAppeal::kBreakAppealViolatingBreakAvoid
        && IsAvoidBreakValue(space, physical_fragment.Style().BreakInside())
    {
        appeal = BreakAppeal::kBreakAppealViolatingBreakAvoid;
    }
    appeal
}

pub fn CalculateBreakAppealInsideDefault(
    space: &ConstraintSpace,
    result: &LayoutResult,
) -> BreakAppeal {
    CalculateBreakAppealInside(space, result, None)
}

// cpp: layoutng/internal/fragmentation_utils.h:350-373
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakStatus {
    kContinue,
    kBrokeBefore,
    kNeedsEarlierBreak,
    kDisableFragmentation,
}

// cpp: layoutng/internal/fragmentation_utils.h:154-156
pub fn ClampedToValidFragmentainerCapacity(length: LayoutUnit) -> LayoutUnit {
    std::cmp::max(length, LayoutUnit::from_signed(1))
}

// cpp: layoutng/internal/fragmentation_utils.h:162-171
pub fn ClampedToValidFragmentainerCapacityForBuilder(
    builder: &BoxFragmentBuilder,
    length: LayoutUnit,
    is_for_children: bool,
) -> LayoutUnit {
    let mut minimum = LayoutUnit::from_signed(1);
    if builder.ShouldCloneBoxEndDecorations() && !is_for_children {
        minimum += builder.BorderScrollbarPadding().BlockSum();
    }
    std::cmp::max(length, minimum)
}

// cpp: layoutng/internal/fragmentation_utils.h:185-201
pub fn FragmentainerCapacity(builder: &BoxFragmentBuilder, is_for_children: bool) -> LayoutUnit {
    let space = builder.GetConstraintSpace();
    if !space.HasKnownFragmentainerBlockSize() {
        return kIndefiniteSize;
    }
    let mut size = space.FragmentainerBlockSize();
    if builder.Style().BoxDecorationBreak() == EBoxDecorationBreak::kClone && is_for_children {
        size -= builder.BorderScrollbarPadding().block_start;
        if builder.ShouldCloneBoxEndDecorations() {
            size -= builder.BorderScrollbarPadding().block_end;
        }
    }
    ClampedToValidFragmentainerCapacityForBuilder(builder, size, is_for_children)
}

// cpp: layoutng/internal/fragmentation_utils.h:210-233
pub fn FragmentainerOffset(builder: &BoxFragmentBuilder, is_for_children: bool) -> LayoutUnit {
    let space = builder.GetConstraintSpace();
    if !space.HasBlockFragmentation() {
        return LayoutUnit::default();
    }
    let mut offset = space.FragmentainerOffset();
    if builder.Style().BoxDecorationBreak() == EBoxDecorationBreak::kClone && is_for_children {
        let break_token = builder.PreviousBreakToken();
        if break_token.is_null() || !unsafe { &*break_token }.IsAtBlockEnd() {
            offset -= builder.BorderScrollbarPadding().block_start;
        }
    }
    offset
}

pub fn FragmentainerOffsetDefault(builder: &BoxFragmentBuilder) -> LayoutUnit {
    FragmentainerOffset(builder, true)
}

// cpp: layoutng/internal/fragmentation_utils.h:247-255
pub fn FragmentainerSpaceLeft(builder: &BoxFragmentBuilder, is_for_children: bool) -> LayoutUnit {
    if !builder
        .GetConstraintSpace()
        .HasKnownFragmentainerBlockSize()
    {
        return kIndefiniteSize;
    }
    let capacity = FragmentainerCapacity(builder, is_for_children);
    let offset = FragmentainerOffset(builder, is_for_children);
    (capacity - offset).ClampNegativeToZero()
}

// cpp: layoutng/internal/fragmentation_utils.h:262-268
pub fn FragmentainerOffsetAtBfc(space: &ConstraintSpace) -> LayoutUnit {
    space.FragmentainerOffset() - space.ExpectedBfcBlockOffset()
}

pub fn FragmentainerOffsetAtBfcForBuilder(builder: &BoxFragmentBuilder) -> LayoutUnit {
    FragmentainerOffsetDefault(builder) - builder.GetConstraintSpace().ExpectedBfcBlockOffset()
}

// cpp: layoutng/internal/fragmentation_utils.h:276-295
pub fn AdjustMarginsForFragmentation(
    break_token: *const BlockBreakToken,
    box_strut: &mut BoxStrut,
) {
    if break_token.is_null() {
        return;
    }
    let break_token = unsafe { &*break_token };
    if !break_token.IsBreakBefore()
        || (!break_token.IsForcedBreak() && !break_token.InputNode().IsFloating())
    {
        box_strut.block_start = LayoutUnit::default();
    }
    if break_token.IsAtBlockEnd() {
        box_strut.block_end = LayoutUnit::default();
    }
}

// cpp: layoutng/internal/fragmentation_utils.h:337-346
pub fn ClonedBlockStartDecoration(builder: &BoxFragmentBuilder) -> LayoutUnit {
    let break_token = builder.PreviousBreakToken();
    if builder.Style().BoxDecorationBreak() == EBoxDecorationBreak::kClone
        && IsBreakInside(break_token)
        && !unsafe { &*break_token }.IsAtBlockEnd()
    {
        return builder.BorderScrollbarPadding().block_start;
    }
    LayoutUnit::default()
}

// cpp: layoutng/internal/fragmentation_utils.h:301-306
// cpp: layoutng/internal/fragmentation_utils.cc:311-367
pub fn SetupSpaceBuilderForFragmentationFromSpace(
    parent_space: &ConstraintSpace,
    child: &LayoutInputNode,
    offset: LayoutUnit,
    block_size: LayoutUnit,
    requires_content_before_breaking: bool,
    builder: &mut ConstraintSpaceBuilder,
) {
    debug_assert!(parent_space.HasBlockFragmentation());
    if child.IsMonolithic() && !child.IsInline() {
        builder.SetShouldPropagateChildBreakValues(false);
        return;
    }
    builder.SetFragmentainerBlockSize(block_size);
    builder.SetFragmentainerOffset(offset);
    if offset <= LayoutUnit::default() {
        builder.SetIsAtFragmentainerStart();
    }
    builder.SetFragmentationType(parent_space.BlockFragmentationType());
    builder.SetShouldPropagateChildBreakValues(true);
    debug_assert!(
        !requires_content_before_breaking || !parent_space.IsInitialColumnBalancingPass()
    );
    builder.SetRequiresContentBeforeBreaking(requires_content_before_breaking);
    if parent_space.IsInsideBalancedColumns() {
        builder.SetIsInsideBalancedColumns();
    }
    builder.SetIsInsideBreakAvoid(parent_space.IsInsideBreakAvoid());
    if (parent_space.IsInitialColumnBalancingPass() && child.IsOutOfFlowPositioned())
        || parent_space.ShouldIgnoreForcedBreaks()
    {
        builder.SetShouldIgnoreForcedBreaks();
    }
    builder.SetMinBreakAppeal(parent_space.MinBreakAppeal());
    if parent_space.IsPaginated() {
        let page_name = child.PageName();
        if !page_name.IsNull() {
            builder.SetPageName(&page_name);
        } else {
            builder.SetPageName(&parent_space.PageName());
        }
    }
}

// cpp: layoutng/internal/fragmentation_utils.h:311-315
// cpp: layoutng/internal/fragmentation_utils.cc:369-387
pub fn SetupSpaceBuilderForFragmentationFromBuilder(
    parent_builder: &BoxFragmentBuilder,
    child: &LayoutInputNode,
    offset_delta: LayoutUnit,
    builder: &mut ConstraintSpaceBuilder,
) {
    let block_size = FragmentainerCapacity(parent_builder, true);
    let block_offset = FragmentainerOffset(parent_builder, true) + offset_delta;
    SetupSpaceBuilderForFragmentationFromSpace(
        parent_builder.GetConstraintSpace(),
        child,
        block_offset,
        block_size,
        parent_builder.RequiresContentBeforeBreaking(),
        builder,
    );
    if IsAvoidBreakValue(
        parent_builder.GetConstraintSpace(),
        parent_builder.Style().BreakInside(),
    ) {
        builder.SetIsInsideBreakAvoid(true);
    }
}

// cpp: layoutng/internal/fragmentation_utils.h:319-323
pub fn SetupFragmentBuilderForFragmentation(
    space: &ConstraintSpace,
    node: BlockNode,
    token: *const BlockBreakToken,
    builder: &mut BoxFragmentBuilder,
) {
    SetupFragmentBuilderForFragmentationForInputNode(space, &node.base, token, builder)
}

// cpp: layoutng/internal/fragmentation_utils.cc:389-510
pub fn SetupFragmentBuilderForFragmentationForInputNode(
    space: &ConstraintSpace,
    node: &LayoutInputNode,
    token: *const BlockBreakToken,
    builder: &mut BoxFragmentBuilder,
) {
    debug_assert!(space.HasBlockFragmentation() || !token.is_null());
    debug_assert!(!node.IsMonolithic() || space.IsAnonymous());
    builder.SetIsMonolithic(
        !space.IsAnonymous() && space.IsBlockFragmentationForcedOff() && !IsBreakInside(token),
    );

    let mut sequence_number = 0;
    if !token.is_null() && !unsafe { &*token }.IsBreakBefore() {
        sequence_number = unsafe { &*token }.SequenceNumber() + 1;
        builder.SetIsFirstForNode(false);
    }
    let space_left = FragmentainerSpaceLeft(builder, false);
    let clone_box_start_decorations = ShouldCloneBlockStartBorderPadding(builder);
    let mut clone_box_end_decorations = clone_box_start_decorations;
    if clone_box_start_decorations {
        builder.UpdateBorderPaddingForClonedBoxDecorations();
    }

    if space.HasBlockFragmentation()
        && !space.IsAnonymous()
        && !space.IsInitialColumnBalancingPass()
    {
        let mut requires_content_before_breaking = space.RequiresContentBeforeBreaking();
        if !node.IsTable() && builder.InitialBorderBoxSize().inline_size != kIndefiniteSize {
            let block_node = BlockNode::from(node.clone());
            let max_block_size = ComputeBlockSizeForFragment(
                space,
                &block_node,
                builder.BorderPadding(),
                LayoutUnit::Max(),
                builder.InitialBorderBoxSize().inline_size,
                kIndefiniteSize,
            );
            debug_assert!(space.HasKnownFragmentainerBlockSize());
            if max_block_size != LayoutUnit::Max() {
                let mut previously_consumed_block_size = LayoutUnit::default();
                if !token.is_null() {
                    previously_consumed_block_size = unsafe { &*token }.ConsumedBlockSize();
                }
                if max_block_size - previously_consumed_block_size <= space_left {
                    builder.SetIsKnownToFitInFragmentainer(true);
                    clone_box_end_decorations = false;
                    if builder.MustStayInCurrentFragmentainer() {
                        requires_content_before_breaking = true;
                    }
                }
            }
        }
        if clone_box_end_decorations {
            builder.SetShouldCloneBoxEndDecorations(true);
            builder.SetShouldPreventBreakBeforeBlockEndDecorations(true);
        }
        builder.SetRequiresContentBeforeBreaking(requires_content_before_breaking);
    }
    builder.SetSequenceNumber(sequence_number);

    if IsBreakInside(token) && !clone_box_start_decorations && !node.IsTable() {
        builder.ClearBorderScrollbarPaddingBlockStart();
    }
    if space.IsInitialColumnBalancingPass() {
        let unbreakable = *builder.BorderScrollbarPadding();
        builder.PropagateTallestUnbreakableBlockSize(unbreakable.block_start);
        builder.PropagateTallestUnbreakableBlockSize(unbreakable.block_end);
    }
}

// cpp: layoutng/internal/fragmentation_utils.h:328
// cpp: layoutng/internal/fragmentation_utils.cc:512-515
pub fn ShouldIncludeBlockStartBorderPadding(builder: &BoxFragmentBuilder) -> bool {
    !IsBreakInside(builder.PreviousBreakToken()) || ShouldCloneBlockStartBorderPadding(builder)
}

// cpp: layoutng/internal/fragmentation_utils.h:333
// cpp: layoutng/internal/fragmentation_utils.cc:517-535
pub fn ShouldIncludeBlockEndBorderPadding(builder: &BoxFragmentBuilder) -> bool {
    let previous_break_token = builder.PreviousBreakToken();
    if !previous_break_token.is_null() && unsafe { &*previous_break_token }.IsAtBlockEnd() {
        return false;
    }
    if !builder.ShouldBreakInside()
        || builder.IsKnownToFitInFragmentainer()
        || builder.ShouldCloneBoxEndDecorations()
    {
        return true;
    }
    if builder.GetConstraintSpace().IsNewFormattingContext() {
        return false;
    }
    !builder.HasInflowChildBreakInside()
}

// cpp: layoutng/internal/fragmentation_utils.h:393
// cpp: layoutng/internal/fragmentation_utils.cc:537-812
pub fn FinishFragmentation(builder: &mut BoxFragmentBuilder) -> BreakStatus {
    let node = builder.Node();
    let space = builder.GetConstraintSpace() as *const ConstraintSpace;
    let mut space_left = FragmentainerSpaceLeft(builder, false);
    let previous_break_token = builder.PreviousBreakToken();
    let mut previously_consumed_block_size = LayoutUnit::default();
    if !previous_break_token.is_null() && !unsafe { &*previous_break_token }.IsBreakBefore() {
        previously_consumed_block_size = unsafe { &*previous_break_token }.ConsumedBlockSize();
    }
    let is_past_end =
        !previous_break_token.is_null() && unsafe { &*previous_break_token }.IsAtBlockEnd();
    let fragments_total_block_size = builder.FragmentsTotalBlockSize();
    let desired_block_size =
        (fragments_total_block_size - previously_consumed_block_size).ClampNegativeToZero();
    let desired_intrinsic_block_size = builder.IntrinsicBlockSize();
    let mut final_block_size = desired_block_size;
    let trailing_border_padding = builder.BorderScrollbarPadding().block_end;
    let mut subtractable_border_padding = LayoutUnit::default();
    if !builder.ShouldPreventBreakBeforeBlockEndDecorations()
        && (desired_block_size > trailing_border_padding
            || (!previous_break_token.is_null()
                && unsafe { &*previous_break_token }.MonolithicOverflow() != LayoutUnit::default()))
    {
        subtractable_border_padding = trailing_border_padding;
    }
    if space_left != kIndefiniteSize {
        space_left = std::cmp::max(
            space_left,
            desired_intrinsic_block_size - subtractable_border_padding,
        );
    }
    if builder.FoundColumnSpanner() {
        builder.SetDidBreakSelf();
    }
    if is_past_end {
        final_block_size = LayoutUnit::default();
    } else if builder.FoundColumnSpanner() {
        final_block_size = (std::cmp::min(final_block_size, desired_intrinsic_block_size)
            - trailing_border_padding)
            .ClampNegativeToZero();
    } else if space_left != kIndefiniteSize
        && desired_block_size > space_left
        && unsafe { &*space }.HasBlockFragmentation()
    {
        debug_assert!(desired_intrinsic_block_size >= trailing_border_padding);
        debug_assert!(desired_block_size >= trailing_border_padding);
        let modified_intrinsic_block_size = std::cmp::max(
            space_left,
            desired_intrinsic_block_size - subtractable_border_padding,
        );
        builder.SetIntrinsicBlockSize(modified_intrinsic_block_size);
        final_block_size = std::cmp::min(
            desired_block_size - subtractable_border_padding,
            modified_intrinsic_block_size,
        );
        if final_block_size < desired_block_size {
            builder.SetDidBreakSelf();
        }
    }
    let mut sides = LogicalBoxSides::default();
    if previously_consumed_block_size != LayoutUnit::default()
        && (node.Style().BoxDecorationBreak() == EBoxDecorationBreak::kSlice || is_past_end)
    {
        sides.block_start = false;
    }
    if (builder.DidBreakSelf() && !builder.ShouldCloneBoxEndDecorations()) || is_past_end {
        sides.block_end = false;
    }
    builder.SetLogicalSidesToInclude(sides);
    builder.SetConsumedBlockSize(previously_consumed_block_size + final_block_size);
    builder.SetFragmentBlockSize(final_block_size);
    if !unsafe { &*space }.HasBlockFragmentation() {
        return BreakStatus::kContinue;
    }
    let mut was_broken_by_child = builder.HasInflowChildBreakInside();
    if !was_broken_by_child && unsafe { &*space }.IsNewFormattingContext() {
        was_broken_by_child = builder
            .GetExclusionSpace()
            .NeedsClearancePastFragmentainer(EClear::kBoth);
    }
    if space_left == kIndefiniteSize || builder.FoundColumnSpanner() {
        if !was_broken_by_child || is_past_end {
            builder.SetIsAtBlockEnd();
        }
        return BreakStatus::kContinue;
    }
    if final_block_size == LayoutUnit::default()
        && !previous_break_token.is_null()
        && unsafe { &*previous_break_token }.MonolithicOverflow() != LayoutUnit::default()
    {
        let remaining_overflow = unsafe { &*previous_break_token }.MonolithicOverflow()
            - FragmentainerCapacity(builder, false);
        if remaining_overflow > LayoutUnit::default() {
            builder.ReserveSpaceForMonolithicOverflow(remaining_overflow);
        }
    }
    if builder.ShouldBreakInside() {
        if is_past_end {
            builder.SetIsAtBlockEnd();
            debug_assert_eq!(final_block_size, LayoutUnit::default());
        } else if desired_block_size <= space_left
            && (!was_broken_by_child || builder.IsKnownToFitInFragmentainer())
        {
            if node.HasNonVisibleBlockOverflow() && builder.ShouldBreakInside() {
                return BreakStatus::kDisableFragmentation;
            }
            builder.SetIsAtBlockEnd();
        }
        if builder.IsAtBlockEnd() {
            builder.SetConsumedBlockSize(
                previously_consumed_block_size + std::cmp::max(final_block_size, space_left),
            );
        } else if !builder.ShouldCloneBoxEndDecorations() {
            sides.block_end = false;
            builder.SetLogicalSidesToInclude(sides);
        }
        return BreakStatus::kContinue;
    }
    if desired_block_size > space_left {
        if previously_consumed_block_size == LayoutUnit::default() {
            let geometry = builder.InitialFragmentGeometry();
            let block_start_unbreakable_space = geometry.border.block_start
                + geometry.scrollbar.block_start
                + geometry.padding.block_start;
            if space_left < block_start_unbreakable_space {
                builder.ClampBreakAppeal(BreakAppeal::kBreakAppealLastResort);
            }
        }
        if unsafe { &*space }.BlockFragmentationType() == FragmentationType::kFragmentColumn
            && !unsafe { &*space }.IsInitialColumnBalancingPass()
        {
            builder.PropagateSpaceShortage(Some(desired_block_size - space_left));
        }
        if desired_block_size <= desired_intrinsic_block_size {
            if builder.HasEarlyBreak() {
                return BreakStatus::kNeedsEarlierBreak;
            }
            builder.ClampBreakAppeal(BreakAppeal::kBreakAppealLastResort);
        }
        return BreakStatus::kContinue;
    }
    builder.SetIsAtBlockEnd();
    BreakStatus::kContinue
}

// cpp: layoutng/internal/fragmentation_utils.h:396
// cpp: layoutng/internal/fragmentation_utils.cc:814-859
pub fn FinishFragmentationForFragmentainer(builder: &mut BoxFragmentBuilder) -> BreakStatus {
    debug_assert!(builder.IsFragmentainerBoxType());
    let previous_break_token = builder.PreviousBreakToken();
    let mut consumed_block_size = if previous_break_token.is_null() {
        LayoutUnit::default()
    } else {
        unsafe { &*previous_break_token }.ConsumedBlockSize()
    };
    if builder
        .GetConstraintSpace()
        .HasKnownFragmentainerBlockSize()
    {
        let block_size = builder.GetConstraintSpace().FragmentainerBlockSize();
        let capacity = FragmentainerCapacity(builder, false);
        builder.SetFragmentBlockSize(block_size);
        consumed_block_size += capacity;
        builder.SetConsumedBlockSize(consumed_block_size);
        if !previous_break_token.is_null()
            && unsafe { &*previous_break_token }.MonolithicOverflow() != LayoutUnit::default()
        {
            let remaining_overflow = unsafe { &*previous_break_token }.MonolithicOverflow()
                - FragmentainerCapacity(builder, false);
            if remaining_overflow > LayoutUnit::default() {
                builder.ReserveSpaceForMonolithicOverflow(remaining_overflow);
            }
        }
    } else {
        let total = builder.FragmentsTotalBlockSize();
        builder.SetFragmentBlockSize(total);
        builder.SetConsumedBlockSize(total + consumed_block_size);
    }
    if builder.IsEmptySpannerParent() && builder.HasOutOfFlowFragmentainerDescendants() {
        builder.SetIsEmptySpannerParent(false);
    }
    BreakStatus::kContinue
}

// cpp: layoutng/internal/fragmentation_utils.h:401-403
// cpp: layoutng/internal/fragmentation_utils.cc:861-884
pub fn HasBreakOpportunityBeforeNextChild(
    fragment: &PhysicalFragment,
    incoming_break_token: *const BreakToken,
) -> bool {
    if fragment.IsBox() {
        let block_break_token = DynamicTo::<BlockBreakToken>(incoming_break_token);
        debug_assert!(incoming_break_token.is_null() || !block_break_token.is_null());
        return block_break_token.is_null() || !unsafe { &*block_break_token }.IsAtBlockEnd();
    }
    debug_assert!(fragment.IsLineBox());
    let logical_fragment = LogicalFragment::new(fragment.Style().GetWritingDirection(), fragment);
    logical_fragment.BlockSize() != LayoutUnit::default()
}

// cpp: layoutng/internal/fragmentation_utils.h:427-435
pub fn BreakBeforeChildIfNeeded(
    child: LayoutInputNode,
    result: &LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    has_container_separation: bool,
    builder: &mut BoxFragmentBuilder,
) -> BreakStatus {
    BreakBeforeChildIfNeededFull(
        child,
        result,
        block_offset,
        capacity,
        has_container_separation,
        builder,
        false,
        std::ptr::null_mut(),
    )
}

// cpp: layoutng/internal/fragmentation_utils.cc:886-933
pub fn BreakBeforeChildIfNeededFull(
    child: LayoutInputNode,
    result: &LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    has_container_separation: bool,
    builder: *mut BoxFragmentBuilder,
    is_row_item: bool,
    flex_column_break_info: *mut FlexColumnBreakInfo,
) -> BreakStatus {
    let builder_ref = unsafe { &mut *builder };
    let space = builder_ref.GetConstraintSpace() as *const ConstraintSpace;
    debug_assert!(unsafe { &*space }.HasBlockFragmentation());
    if has_container_separation && !is_row_item {
        let break_between = CalculateBreakBetweenValue(child.clone(), result, builder_ref);
        if IsForcedBreakValue(unsafe { &*space }, break_between) {
            BreakBeforeChild(
                child,
                result,
                block_offset,
                capacity,
                Some(BreakAppeal::kBreakAppealPerfect),
                true,
                builder,
                None,
            );
            return BreakStatus::kBrokeBefore;
        }
    }
    let appeal_before = CalculateBreakAppealBeforeChild(
        unsafe { &*space },
        child.clone(),
        result,
        builder_ref,
        has_container_separation,
    );
    if MovePastBreakpointFull(
        unsafe { &*space },
        child.clone(),
        result,
        block_offset,
        capacity,
        appeal_before,
        builder,
        is_row_item,
        flex_column_break_info,
    ) {
        return BreakStatus::kContinue;
    }
    if !AttemptSoftBreak(
        child,
        result,
        block_offset,
        capacity,
        appeal_before,
        builder,
        None,
        flex_column_break_info,
    ) {
        return BreakStatus::kNeedsEarlierBreak;
    }
    BreakStatus::kBrokeBefore
}

// cpp: layoutng/internal/fragmentation_utils.h:440-448
// cpp: layoutng/internal/fragmentation_utils.cc:935-968
pub fn BreakBeforeChild(
    child: LayoutInputNode,
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: Option<BreakAppeal>,
    is_forced_break: bool,
    builder: *mut BoxFragmentBuilder,
    block_size_override: Option<LayoutUnit>,
) {
    debug_assert!(!result.is_null() || block_size_override.is_some());
    if !result.is_null() && unsafe { &*result }.Status() == EStatus::kSuccess {
        let physical_fragment = unsafe { &*result }.GetPhysicalFragment();
        if physical_fragment.IsBox() {
            let box_fragment =
                DynamicTo::<PhysicalBoxFragment>(physical_fragment as *const PhysicalFragment);
            debug_assert!(!box_fragment.is_null() && unsafe { &*box_fragment }.IsFirstForNode());
        }
    }
    let builder_ref = unsafe { &mut *builder };
    let space = builder_ref.GetConstraintSpace() as *const ConstraintSpace;
    if unsafe { &*space }.HasKnownFragmentainerBlockSize() {
        let base = std::ops::DerefMut::deref_mut(builder_ref) as *mut FragmentBuilder;
        PropagateSpaceShortage(result, block_offset, capacity, base, block_size_override);
    }
    if !result.is_null() && unsafe { &*space }.ShouldPropagateChildBreakValues() && !is_forced_break
    {
        builder_ref.PropagateChildBreakValues(unsafe { &*result });
    }
    builder_ref.AddBreakBeforeChild(child, appeal, is_forced_break, LogicalOffset::default());
}

pub fn BreakBeforeChildDefault(
    child: LayoutInputNode,
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: Option<BreakAppeal>,
    is_forced_break: bool,
    builder: *mut BoxFragmentBuilder,
) {
    BreakBeforeChild(
        child,
        result,
        block_offset,
        capacity,
        appeal,
        is_forced_break,
        builder,
        None,
    )
}

// cpp: layoutng/internal/fragmentation_utils.h:455-457
// cpp: layoutng/internal/fragmentation_utils.cc:970-984
pub fn CalculateUnbreakableBlockSize(
    space: &ConstraintSpace,
    result: &LayoutResult,
    block_offset: LayoutUnit,
) -> LayoutUnit {
    let mut block_size = BlockSizeForFragmentation(result, space.GetWritingDirection());
    if block_offset < LayoutUnit::default() {
        block_size += block_offset;
    }
    block_size
}

// cpp: layoutng/internal/fragmentation_utils.h:465-470
// cpp: layoutng/internal/fragmentation_utils.cc:986-1005
pub fn PropagateSpaceShortage(
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    builder: *mut FragmentBuilder,
    block_size_override: Option<LayoutUnit>,
) {
    let builder_ref = unsafe { &mut *builder };
    let space = builder_ref.GetConstraintSpace() as *const ConstraintSpace;
    if unsafe { &*space }.BlockFragmentationType() != FragmentationType::kFragmentColumn {
        return;
    }
    let space_shortage = CalculateSpaceShortage(
        unsafe { &*space },
        result,
        block_offset,
        capacity,
        block_size_override,
    );
    if space_shortage > LayoutUnit::default() {
        builder_ref.PropagateSpaceShortage(Some(space_shortage));
    }
}

pub fn PropagateSpaceShortageDefault(
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    builder: *mut FragmentBuilder,
) {
    PropagateSpaceShortage(result, block_offset, capacity, builder, None)
}

// cpp: layoutng/internal/fragmentation_utils.h:476-481
// cpp: layoutng/internal/fragmentation_utils.cc:1007-1044
pub fn CalculateSpaceShortage(
    space: &ConstraintSpace,
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    block_size_override: Option<LayoutUnit>,
) -> LayoutUnit {
    debug_assert!(space.HasKnownFragmentainerBlockSize());
    debug_assert!(!result.is_null() || block_size_override.is_some());
    debug_assert_eq!(
        space.BlockFragmentationType(),
        FragmentationType::kFragmentColumn
    );
    if let Some(block_size) = block_size_override {
        return block_offset + block_size - capacity;
    }
    let result = unsafe { &*result };
    if let Some(space_shortage) = result.MinimalSpaceShortage() {
        space_shortage
    } else {
        if result.Status() != EStatus::kSuccess {
            return kIndefiniteSize;
        }
        let child_block_size = BlockSizeForFragmentation(result, space.GetWritingDirection());
        block_offset + child_block_size - capacity
    }
}

pub fn CalculateSpaceShortageDefault(
    space: &ConstraintSpace,
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
) -> LayoutUnit {
    CalculateSpaceShortage(space, result, block_offset, capacity, None)
}

// cpp: layoutng/internal/fragmentation_utils.h:483-484
// cpp: layoutng/internal/fragmentation_utils.cc:1046-1057
pub fn UpdateMinimalSpaceShortage(
    space_shortage: Option<LayoutUnit>,
    minimal_space_shortage: &mut LayoutUnit,
) {
    let Some(space_shortage) = space_shortage else {
        return;
    };
    if space_shortage <= LayoutUnit::default() {
        return;
    }
    if *minimal_space_shortage == kIndefiniteSize {
        *minimal_space_shortage = space_shortage;
    } else {
        *minimal_space_shortage = std::cmp::min(*minimal_space_shortage, space_shortage);
    }
}

// cpp: layoutng/internal/fragmentation_utils.h:492-500
pub fn MovePastBreakpoint(
    space: &ConstraintSpace,
    child: LayoutInputNode,
    result: &LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: BreakAppeal,
    builder: &mut BoxFragmentBuilder,
) -> bool {
    MovePastBreakpointFull(
        space,
        child,
        result,
        block_offset,
        capacity,
        appeal,
        builder,
        false,
        std::ptr::null_mut(),
    )
}

// cpp: layoutng/internal/fragmentation_utils.h:503-510
pub fn MovePastBreakpointWithoutChild(
    space: &ConstraintSpace,
    result: &LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: BreakAppeal,
    builder: &mut BoxFragmentBuilder,
) -> bool {
    MovePastBreakpointWithoutChildFull(
        space,
        result,
        block_offset,
        capacity,
        appeal,
        builder,
        false,
        std::ptr::null_mut(),
    )
}

// cpp: layoutng/internal/fragmentation_utils.cc:1059-1133
pub fn MovePastBreakpointFull(
    space: &ConstraintSpace,
    child: LayoutInputNode,
    result: &LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: BreakAppeal,
    builder: *mut BoxFragmentBuilder,
    is_row_item: bool,
    flex_column_break_info: *mut FlexColumnBreakInfo,
) -> bool {
    if result.Status() != EStatus::kSuccess {
        debug_assert_eq!(result.Status(), EStatus::kOutOfFragmentainerSpace);
        debug_assert!(child.IsInline());
        return false;
    }
    if child.IsBlock() {
        let box_fragment = DynamicTo::<PhysicalBoxFragment>(
            result.GetPhysicalFragment() as *const PhysicalFragment
        );
        debug_assert!(!box_fragment.is_null());
        if !unsafe { &*box_fragment }.IsFirstForNode() {
            return true;
        }
        if !builder.is_null() {
            let clear_type = unsafe { &*child.GetLayoutBox() }
                .StyleRef()
                .ClearWithDirection(space.Direction());
            if unsafe { &mut *builder }
                .GetExclusionSpace()
                .NeedsClearancePastFragmentainer(clear_type)
            {
                return false;
            }
        }
    }
    if space.IsInitialColumnBalancingPass()
        && !builder.is_null()
        && ShouldAvoidBreakInside(space, result)
    {
        let block_size = CalculateUnbreakableBlockSize(space, result, block_offset);
        unsafe { &mut *builder }.PropagateTallestUnbreakableBlockSize(block_size);
    }
    let move_past = MovePastBreakpointWithoutChildFull(
        space,
        result,
        block_offset,
        capacity,
        appeal,
        builder,
        is_row_item,
        flex_column_break_info,
    );
    if move_past && !builder.is_null() && child.IsBlock() && !is_row_item {
        UpdateEarlyBreakAtBlockChild(
            BlockNode::from(child.clone()),
            result,
            appeal,
            builder,
            flex_column_break_info,
        );
    }
    move_past
}

// cpp: layoutng/internal/fragmentation_utils.cc:1135-1239
pub fn MovePastBreakpointWithoutChildFull(
    space: &ConstraintSpace,
    result: &LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: BreakAppeal,
    builder: *mut BoxFragmentBuilder,
    is_row_item: bool,
    flex_column_break_info: *mut FlexColumnBreakInfo,
) -> bool {
    debug_assert_eq!(result.Status(), EStatus::kSuccess);
    if !space.HasKnownFragmentainerBlockSize() {
        return true;
    }
    let physical_fragment = result.GetPhysicalFragment();
    let _fragment = LogicalFragment::new(space.GetWritingDirection(), physical_fragment);
    let break_token = DynamicTo::<BlockBreakToken>(physical_fragment.GetBreakToken());
    let space_left = capacity - block_offset;
    let refuse_break_before = space_left >= capacity
        && (builder.is_null()
            || !IsBreakableAtStartOfResumedContainerWithResult(space, result, unsafe {
                &*builder
            }));
    let mut must_break_before = false;
    if space_left < LayoutUnit::default() {
        must_break_before = true;
    } else if space_left == LayoutUnit::default() {
        must_break_before = result.GetColumnSpannerPath().is_null()
            && IsBreakInside(break_token)
            && !unsafe { &*break_token }.IsAtBlockEnd();
    }
    if must_break_before {
        debug_assert!(!refuse_break_before);
        return false;
    }
    let block_size = BlockSizeForFragmentation(result, space.GetWritingDirection());
    let appeal_inside = CalculateBreakAppealInsideDefault(space, result);
    let mut move_past = refuse_break_before;
    if !move_past {
        if block_size <= space_left {
            if IsBreakInside(break_token) || appeal_inside < BreakAppeal::kBreakAppealPerfect {
                if appeal_inside >= appeal {
                    if !flex_column_break_info.is_null() {
                        let earlier = unsafe { &*flex_column_break_info }.early_break.Get();
                        if earlier.is_null()
                            || appeal_inside >= unsafe { &*earlier }.GetBreakAppeal()
                        {
                            move_past = true;
                        }
                    } else if builder.is_null()
                        || !unsafe { &*builder }.HasEarlyBreak()
                        || appeal_inside >= unsafe { &*builder }.GetEarlyBreak().GetBreakAppeal()
                    {
                        move_past = true;
                    }
                }
            } else {
                move_past = true;
            }
        } else if appeal == BreakAppeal::kBreakAppealLastResort
            && !builder.is_null()
            && unsafe { &*builder }.RequiresContentBeforeBreaking()
        {
            unsafe { &mut *builder }.SetIsBlockSizeForFragmentationClamped();
            move_past = true;
        }
    }
    if move_past {
        if !builder.is_null() && block_size > space_left {
            let base =
                std::ops::DerefMut::deref_mut(unsafe { &mut *builder }) as *mut FragmentBuilder;
            PropagateSpaceShortage(result, block_offset, capacity, base, None);
        }
        return true;
    }
    false
}

// cpp: layoutng/internal/fragmentation_utils.h:516-521
// cpp: layoutng/internal/fragmentation_utils.cc:1240-1306
pub fn UpdateEarlyBreakAtBlockChild(
    child: BlockNode,
    result: &LayoutResult,
    appeal: BreakAppeal,
    builder: *mut BoxFragmentBuilder,
    flex_column_break_info: *mut FlexColumnBreakInfo,
) {
    let break_token = DynamicTo::<BlockBreakToken>(result.GetPhysicalFragment().GetBreakToken());
    let mut appeal_inside = BreakAppeal::kBreakAppealLastResort;
    let breakpoint = result.GetEarlyBreak();
    if !breakpoint.is_null() {
        debug_assert!(!IsBreakInside(break_token));
        let space = unsafe { &*builder }.GetConstraintSpace();
        appeal_inside = CalculateBreakAppealInside(
            space,
            result,
            Some(unsafe { &*breakpoint }.GetBreakAppeal()),
        );
        if !flex_column_break_info.is_null() {
            let info = unsafe { &mut *flex_column_break_info };
            let earlier = info.early_break.Get();
            if earlier.is_null()
                || unsafe { &*earlier }.GetBreakAppeal() <= unsafe { &*breakpoint }.GetBreakAppeal()
            {
                let parent_break = MakeGarbageCollected(EarlyBreak::from_block_with_child(
                    child.clone(),
                    appeal_inside,
                    breakpoint,
                ));
                info.early_break = Member::from_ptr(parent_break);
            }
        } else if !unsafe { &*builder }.HasEarlyBreak()
            || unsafe { &*builder }.GetEarlyBreak().GetBreakAppeal()
                <= unsafe { &*breakpoint }.GetBreakAppeal()
        {
            let parent_break = MakeGarbageCollected(EarlyBreak::from_block_with_child(
                child.clone(),
                appeal_inside,
                breakpoint,
            ));
            unsafe { &mut *builder }.SetEarlyBreak(parent_break);
        }
    }
    if appeal <= appeal_inside {
        return;
    }
    if !flex_column_break_info.is_null() {
        let info = unsafe { &mut *flex_column_break_info };
        let earlier = info.early_break.Get();
        if !earlier.is_null() && unsafe { &*earlier }.GetBreakAppeal() > appeal {
            return;
        }
        info.early_break =
            Member::from_ptr(MakeGarbageCollected(EarlyBreak::from_block(child, appeal)));
        return;
    }
    if unsafe { &*builder }.HasEarlyBreak()
        && unsafe { &*builder }.GetEarlyBreak().GetBreakAppeal() > appeal
    {
        return;
    }
    unsafe { &mut *builder }
        .SetEarlyBreak(MakeGarbageCollected(EarlyBreak::from_block(child, appeal)));
}

pub fn UpdateEarlyBreakAtBlockChildDefault(
    child: BlockNode,
    result: &LayoutResult,
    appeal: BreakAppeal,
    builder: *mut BoxFragmentBuilder,
) {
    UpdateEarlyBreakAtBlockChild(child, result, appeal, builder, std::ptr::null_mut())
}

// cpp: layoutng/internal/fragmentation_utils.h:529-537
// cpp: layoutng/internal/fragmentation_utils.cc:1307-1345
pub fn AttemptSoftBreak(
    child: LayoutInputNode,
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: BreakAppeal,
    builder: *mut BoxFragmentBuilder,
    block_size_override: Option<LayoutUnit>,
    flex_column_break_info: *mut FlexColumnBreakInfo,
) -> bool {
    debug_assert!(!result.is_null() || block_size_override.is_some());
    let found_earlier_break = if !flex_column_break_info.is_null() {
        let earlier = unsafe { &*flex_column_break_info }.early_break.Get();
        !earlier.is_null() && unsafe { &*earlier }.GetBreakAppeal() > appeal
    } else {
        let builder_ref = unsafe { &*builder };
        builder_ref.HasEarlyBreak() && builder_ref.GetEarlyBreak().GetBreakAppeal() > appeal
    };
    if found_earlier_break {
        let base = std::ops::DerefMut::deref_mut(unsafe { &mut *builder }) as *mut FragmentBuilder;
        PropagateSpaceShortage(result, block_offset, capacity, base, block_size_override);
        return false;
    }
    BreakBeforeChild(
        child,
        result,
        block_offset,
        capacity,
        Some(appeal),
        false,
        builder,
        block_size_override,
    );
    true
}

pub fn AttemptSoftBreakDefault(
    child: LayoutInputNode,
    result: *const LayoutResult,
    block_offset: LayoutUnit,
    capacity: LayoutUnit,
    appeal: BreakAppeal,
    builder: *mut BoxFragmentBuilder,
) -> bool {
    AttemptSoftBreak(
        child,
        result,
        block_offset,
        capacity,
        appeal,
        builder,
        None,
        std::ptr::null_mut(),
    )
}

// cpp: layoutng/internal/fragmentation_utils.h:543-544
// cpp: layoutng/internal/fragmentation_utils.cc:1346-1357
pub fn EnterEarlyBreakInChild(child: &BlockNode, early_break: &EarlyBreak) -> *const EarlyBreak {
    if early_break.Type() != BreakType::kBlock || !early_break.GetBlockNode().EqualsBlockNode(child)
    {
        return std::ptr::null();
    }
    debug_assert!(!early_break.BreakInside().is_null());
    early_break.BreakInside()
}

// cpp: layoutng/internal/fragmentation_utils.h:548-550
// cpp: layoutng/internal/fragmentation_utils.cc:1359-1367
pub fn IsEarlyBreakTarget(
    early_break: &EarlyBreak,
    builder: &BoxFragmentBuilder,
    child: &LayoutInputNode,
) -> bool {
    if early_break.Type() == BreakType::kLine {
        debug_assert!(child.IsInline() || child.IsFlexItem());
        return early_break.LineNumber() == builder.LineCount();
    }
    early_break.IsBreakBefore() && early_break.GetBlockNode().EqualsLayoutInputNode(child)
}

// cpp: layoutng/internal/fragmentation_utils.h:554-564
pub fn FollowColumnSpannerPath(
    path: *const ColumnSpannerPath,
    child: &BlockNode,
) -> *const ColumnSpannerPath {
    if path.is_null() {
        return std::ptr::null();
    }
    let next_step = unsafe { &*path }.Child();
    if !next_step.is_null()
        && unsafe { &*next_step }.GetBlockNode().GetLayoutBox() == child.GetLayoutBox()
    {
        return next_step;
    }
    std::ptr::null()
}

// cpp: layoutng/internal/fragmentation_utils.h:581-588
pub fn AdjustedMarginAfterFinalChildFragment(
    builder: &BoxFragmentBuilder,
    block_offset: LayoutUnit,
    block_end_margin: LayoutUnit,
) -> LayoutUnit {
    let space_left = FragmentainerSpaceLeft(builder, true) - block_offset;
    std::cmp::min(block_end_margin, space_left.ClampNegativeToZero())
}

// cpp: layoutng/internal/fragmentation_utils.h:568-575
// cpp: layoutng/internal/fragmentation_utils.cc:1369-1403
pub fn CreateConstraintSpaceForFragmentainer(
    parent_space: &ConstraintSpace,
    fragmentation_type: FragmentationType,
    fragmentainer_size: LogicalSize,
    percentage_resolution_size: LogicalSize,
    balance_columns: bool,
    min_break_appeal: BreakAppeal,
    builder: *const BoxFragmentBuilder,
) -> ConstraintSpace {
    let mut space_builder =
        ConstraintSpaceBuilder::new(parent_space, parent_space.GetWritingDirection(), true);
    space_builder.SetAvailableSize(fragmentainer_size);
    space_builder.SetPercentageResolutionSize(percentage_resolution_size);
    space_builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
    space_builder.SetFragmentationType(fragmentation_type);
    space_builder.SetShouldPropagateChildBreakValues(true);
    space_builder.SetFragmentainerBlockSize(fragmentainer_size.block_size);
    space_builder.SetIsAnonymous(true);
    if fragmentation_type == FragmentationType::kFragmentColumn {
        space_builder.SetIsInColumnBfc();
    }
    if balance_columns {
        debug_assert_eq!(fragmentation_type, FragmentationType::kFragmentColumn);
        space_builder.SetIsInsideBalancedColumns();
    }
    space_builder.SetIsInsideBreakAvoid(false);
    space_builder.SetMinBreakAppeal(min_break_appeal);
    space_builder.SetBaselineAlgorithmType(parent_space.GetBaselineAlgorithmType());
    if !builder.is_null() && unsafe { &*builder }.ShouldTextBoxTrim() {
        SetTextBoxTrimOnChildSpaceBuilder(unsafe { &*builder }, &mut space_builder);
    }
    space_builder.ToConstraintSpace()
}

pub fn CreateConstraintSpaceForFragmentainerDefault(
    parent_space: &ConstraintSpace,
    fragmentation_type: FragmentationType,
    fragmentainer_size: LogicalSize,
    percentage_resolution_size: LogicalSize,
    balance_columns: bool,
    min_break_appeal: BreakAppeal,
) -> ConstraintSpace {
    CreateConstraintSpaceForFragmentainer(
        parent_space,
        fragmentation_type,
        fragmentainer_size,
        percentage_resolution_size,
        balance_columns,
        min_break_appeal,
        std::ptr::null(),
    )
}

// cpp: layoutng/internal/fragmentation_utils.h:595
// The body is already mapped from fragmentation_utils.cc in LayoutBox's owner.
pub fn FindPreviousBreakToken(fragment: &PhysicalBoxFragment) -> *const BlockBreakToken {
    super::layout_box::FindPreviousBreakToken(fragment)
}

// cpp: layoutng/internal/fragmentation_utils.h:599-600
// cpp: layoutng/internal/fragmentation_utils.cc:1436-1441
pub fn GetFirstFragmentBreakTokenData(
    fragment: &PhysicalBoxFragment,
) -> *const BreakTokenAlgorithmData {
    let box_ = To::<LayoutBox>(fragment.GetLayoutObject());
    let first_fragment = unsafe { &*box_ }.GetPhysicalFragment(0);
    let first_break_token = unsafe { &*first_fragment }.GetBreakToken();
    if first_break_token.is_null() {
        std::ptr::null()
    } else {
        unsafe { &*first_break_token }.TokenData()
    }
}

// cpp: layoutng/internal/fragmentation_utils.h:603
// cpp: layoutng/internal/fragmentation_utils.cc:1443-1447
pub fn BoxFragmentIndex(fragment: &PhysicalBoxFragment) -> usize {
    debug_assert!(!fragment.IsInlineBox());
    let token = FindPreviousBreakToken(fragment);
    if token.is_null() {
        0
    } else {
        unsafe { &*token }.SequenceNumber() as usize + 1
    }
}

// cpp: layoutng/internal/fragmentation_utils.h:609-611
// cpp: layoutng/internal/fragmentation_utils.cc:1449-1475
pub fn OffsetInStitchedFragments(
    fragment: &PhysicalBoxFragment,
    size: *mut PhysicalSize,
) -> PhysicalOffset {
    let writing_direction = fragment.Style().GetWritingDirection();
    let box_ = To::<LayoutBox>(fragment.GetLayoutObject());
    let box_ = unsafe { &*box_ };
    let first_fragment = unsafe { &*box_.GetPhysicalFragment(0) };
    let first_break_token = first_fragment.GetBreakToken();
    let mut fragment_block_offset = LayoutUnit::default();
    if first_break_token.is_null() || !unsafe { &*first_break_token }.IsRepeated() {
        let previous_break_token = FindPreviousBreakToken(fragment);
        if !previous_break_token.is_null() {
            fragment_block_offset = unsafe { &*previous_break_token }.ConsumedBlockSize();
        }
    }
    let stitched_logical_size = LogicalSize::new(
        LogicalFragment::new(writing_direction, fragment).InlineSize(),
        box_.StitchedBlockSize(),
    );
    let stitched_physical_size =
        ToPhysicalSize(stitched_logical_size, writing_direction.GetWritingMode());
    if !size.is_null() {
        unsafe { *size = stitched_physical_size };
    }
    let offset = LogicalOffset::new(LayoutUnit::default(), fragment_block_offset);
    WritingModeConverter::new(writing_direction, stitched_physical_size)
        .ToPhysicalOffset(offset, fragment.Size())
}

pub fn OffsetInStitchedFragmentsDefault(fragment: &PhysicalBoxFragment) -> PhysicalOffset {
    OffsetInStitchedFragments(fragment, std::ptr::null_mut())
}

// cpp: layoutng/internal/fragmentation_utils.h:617-619
// cpp: layoutng/internal/fragmentation_utils.cc:1476-1514
pub fn BlockSizeForFragmentation(
    result: &LayoutResult,
    direction: WritingDirectionMode,
) -> LayoutUnit {
    let mut block_size = result.BlockSizeForFragmentation();
    if block_size == kIndefiniteSize {
        let writing_mode = direction.GetWritingMode();
        let logical_size = ToLogicalSize(result.GetPhysicalFragment().Size(), writing_mode);
        block_size = logical_size.block_size;
        block_size -= result.TrimBlockEndBy().unwrap_or_default();
        let box_fragment = DynamicTo::<PhysicalBoxFragment>(
            result.GetPhysicalFragment() as *const PhysicalFragment
        );
        if !box_fragment.is_null() && unsafe { &*box_fragment }.IsFloating() {
            let margins = unsafe { &*box_fragment }
                .Margins()
                .ConvertToLogical(direction);
            block_size += margins.BlockSum();
        }
    }
    block_size += result.AnnotationBlockOffsetAdjustment();
    let annotation_overflow = result.AnnotationOverflow();
    if annotation_overflow > LayoutUnit::default() {
        block_size += annotation_overflow;
    }
    block_size
}

// cpp: layoutng/internal/fragmentation_utils.h:629
// cpp: layoutng/internal/fragmentation_utils.cc:1515-1520
pub fn CanPaintMultipleFragmentsForFragment(fragment: &PhysicalBoxFragment) -> bool {
    if !fragment.IsCSSBox() {
        return true;
    }
    let object = fragment.GetLayoutObject();
    debug_assert!(!object.is_null());
    CanPaintMultipleFragmentsForObject(unsafe { &*object })
}

// cpp: layoutng/internal/fragmentation_utils.h:630
// cpp: layoutng/internal/fragmentation_utils.cc:1522-1571
pub fn CanPaintMultipleFragmentsForObject(object: &LayoutObject) -> bool {
    let box_ = DynamicTo::<LayoutBox>(object as *const LayoutObject);
    if box_.is_null() {
        return true;
    }
    let box_ = unsafe { &*box_ };
    debug_assert!(box_.PhysicalFragmentCount() > 0);
    if !box_.IsMonolithic() {
        return true;
    }
    if box_.IsScrollContainer() && !object.IsPrintingForLayout() {
        return false;
    }
    if box_.IsLayoutReplaced() {
        if box_.IsLayoutImage() && !box_.IsMedia() {
            return true;
        }
        if box_.IsSVGRoot() {
            return true;
        }
        return false;
    }
    let element = DynamicTo::<Element>(box_.GetNode());
    if !element.is_null() && unsafe { &*element }.IsFormControlElement() {
        return false;
    }
    true
}

// The C++ vector-traits macro at line 634 has no Rust memory-management
// equivalent: Vec moves/drops FlexColumnBreakInfo through its normal traits.

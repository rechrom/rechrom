#![allow(non_snake_case)]

use foundation::{
    kIndefiniteSize, EClear, LayoutUnit, Length, LogicalToLogical, MarginStrut,
    RuntimeEnabledFeatures, To, WritingDirectionMode,
};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::layout_result::{EStatus, LayoutResult};
use layoutng_fragment_tree::logical_box_fragment::LogicalBoxFragment;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::static_position::{BlockEdge, InlineEdge};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{
    ContentDistributionType, ContentPosition, EVerticalAlign, ItemPosition, OverflowAlignment,
};
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;

use super::block_node::BlockNode;
use super::constraint_space::ConstraintSpace;
use super::layout_alignment_utils::BlockContentAlignment;

// cpp: layoutng/internal/layout_utils.cc:20-108
pub fn ResolveContentAlignment(
    style: &ComputedStyle,
    behave_like_table_cell: bool,
) -> BlockContentAlignment {
    let alignment = style.AlignContent();
    let mut position = alignment.GetPosition();
    let mut overflow = alignment.Overflow();
    match alignment.Distribution() {
        ContentDistributionType::kDefault => {}
        ContentDistributionType::kSpaceBetween | ContentDistributionType::kStretch => {
            position = ContentPosition::kFlexStart;
        }
        ContentDistributionType::kSpaceAround | ContentDistributionType::kSpaceEvenly => {
            overflow = OverflowAlignment::kSafe;
            position = ContentPosition::kCenter;
        }
    }
    if position == ContentPosition::kLastBaseline {
        overflow = OverflowAlignment::kSafe;
        position = ContentPosition::kEnd;
    }
    if overflow == OverflowAlignment::kDefault {
        overflow = OverflowAlignment::kSafe;
    }
    let is_safe = overflow == OverflowAlignment::kSafe;
    match position {
        ContentPosition::kCenter => {
            if is_safe {
                BlockContentAlignment::kSafeCenter
            } else {
                BlockContentAlignment::kUnsafeCenter
            }
        }
        ContentPosition::kEnd | ContentPosition::kFlexEnd => {
            if is_safe {
                BlockContentAlignment::kSafeEnd
            } else {
                BlockContentAlignment::kUnsafeEnd
            }
        }
        ContentPosition::kNormal => {
            if !behave_like_table_cell {
                return BlockContentAlignment::kStart;
            }
            match style.VerticalAlign() {
                EVerticalAlign::kTop => BlockContentAlignment::kStart,
                EVerticalAlign::kBaselineMiddle
                | EVerticalAlign::kSub
                | EVerticalAlign::kSuper
                | EVerticalAlign::kTextTop
                | EVerticalAlign::kTextBottom
                | EVerticalAlign::kLength
                | EVerticalAlign::kBaseline => BlockContentAlignment::kBaseline,
                EVerticalAlign::kMiddle => {
                    if RuntimeEnabledFeatures::LayoutTableCellAlignmentSafeEnabled() {
                        BlockContentAlignment::kSafeCenter
                    } else {
                        BlockContentAlignment::kUnsafeCenter
                    }
                }
                EVerticalAlign::kBottom => {
                    if RuntimeEnabledFeatures::LayoutTableCellAlignmentSafeEnabled() {
                        BlockContentAlignment::kSafeEnd
                    } else {
                        BlockContentAlignment::kUnsafeEnd
                    }
                }
            }
        }
        ContentPosition::kStart | ContentPosition::kFlexStart => BlockContentAlignment::kStart,
        ContentPosition::kBaseline => BlockContentAlignment::kBaseline,
        ContentPosition::kLastBaseline | ContentPosition::kLeft | ContentPosition::kRight => {
            unreachable!("unsupported resolved content position")
        }
    }
}

// cpp: layoutng/internal/layout_utils.cc:733-736
pub fn ComputeContentAlignmentForTableCell(style: &ComputedStyle) -> BlockContentAlignment {
    ResolveContentAlignment(style, true)
}

// cpp: layoutng/internal/layout_utils.cc:738-786
pub fn InlineStaticPositionEdge(
    oof_node: &BlockNode,
    justify_items_style: *const ComputedStyle,
    parent_writing_direction: WritingDirectionMode,
    should_swap_inline_axis: bool,
) -> InlineEdge {
    assert!(oof_node.IsOutOfFlowPositioned());
    let normal_value_behavior =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kStart, OverflowAlignment::kDefault);
    let align_self = oof_node
        .Style()
        .ResolvedJustifySelf(&normal_value_behavior, justify_items_style)
        .GetPosition();
    match align_self {
        ItemPosition::kEnd
        | ItemPosition::kFlexEnd
        | ItemPosition::kLastBaseline
        | ItemPosition::kRight => {
            if should_swap_inline_axis {
                InlineEdge::kInlineStart
            } else {
                InlineEdge::kInlineEnd
            }
        }
        ItemPosition::kAnchorCenter | ItemPosition::kCenter => InlineEdge::kInlineCenter,
        ItemPosition::kBaseline
        | ItemPosition::kFlexStart
        | ItemPosition::kLeft
        | ItemPosition::kStart
        | ItemPosition::kStretch => {
            if should_swap_inline_axis {
                InlineEdge::kInlineEnd
            } else {
                InlineEdge::kInlineStart
            }
        }
        ItemPosition::kSelfStart | ItemPosition::kSelfEnd => {
            let logical = LogicalToLogical::new(
                oof_node.Style().GetWritingDirection(),
                parent_writing_direction,
                InlineEdge::kInlineStart,
                InlineEdge::kInlineEnd,
                InlineEdge::kInlineStart,
                InlineEdge::kInlineEnd,
            );
            if align_self == ItemPosition::kSelfStart {
                logical.InlineStart()
            } else {
                logical.InlineEnd()
            }
        }
        ItemPosition::kAuto | ItemPosition::kLegacy | ItemPosition::kNormal => {
            unreachable!("resolved justify-self cannot be auto, legacy, or normal")
        }
    }
}

// cpp: layoutng/internal/layout_utils.cc:788-829
pub fn BlockStaticPositionEdge(
    oof_node: &BlockNode,
    align_items_style: *const ComputedStyle,
    parent_writing_direction: WritingDirectionMode,
) -> BlockEdge {
    assert!(oof_node.IsOutOfFlowPositioned());
    let normal_value_behavior =
        StyleSelfAlignmentData::new_nonlegacy(ItemPosition::kStart, OverflowAlignment::kDefault);
    let align_self = oof_node
        .Style()
        .ResolvedAlignSelf(&normal_value_behavior, align_items_style)
        .GetPosition();
    match align_self {
        ItemPosition::kEnd | ItemPosition::kFlexEnd | ItemPosition::kLastBaseline => {
            BlockEdge::kBlockEnd
        }
        ItemPosition::kAnchorCenter | ItemPosition::kCenter => BlockEdge::kBlockCenter,
        ItemPosition::kBaseline
        | ItemPosition::kFlexStart
        | ItemPosition::kStart
        | ItemPosition::kStretch => BlockEdge::kBlockStart,
        ItemPosition::kSelfEnd | ItemPosition::kSelfStart => {
            let logical = LogicalToLogical::new(
                oof_node.Style().GetWritingDirection(),
                parent_writing_direction,
                BlockEdge::kBlockStart,
                BlockEdge::kBlockEnd,
                BlockEdge::kBlockStart,
                BlockEdge::kBlockEnd,
            );
            if align_self == ItemPosition::kSelfStart {
                logical.BlockStart()
            } else {
                logical.BlockEnd()
            }
        }
        ItemPosition::kAuto
        | ItemPosition::kLeft
        | ItemPosition::kRight
        | ItemPosition::kLegacy
        | ItemPosition::kNormal => {
            unreachable!("resolved align-self cannot be auto, left, right, legacy, or normal")
        }
    }
}

// cpp: layoutng/internal/layout_utils.cc:112-117
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LengthResolveType {
    kMinSize,
    kMaxSize,
    kMainSize,
}

// cpp: layoutng/internal/layout_utils.cc:119-153
fn InlineLengthMayChange(
    style: &ComputedStyle,
    length: &Length,
    resolve_type: LengthResolveType,
    new_space: &ConstraintSpace,
    old_space: &ConstraintSpace,
    _layout_result: &LayoutResult,
) -> bool {
    debug_assert_eq!(
        new_space.InlineAutoBehavior(),
        old_space.InlineAutoBehavior()
    );
    let is_unspecified = (length.HasAuto() && resolve_type != LengthResolveType::kMinSize)
        || length.HasFitContent()
        || length.HasStretch();
    if is_unspecified
        && style.MayHaveMargin()
        && (style.MarginInlineStart().HasPercent() || style.MarginInlineEnd().HasPercent())
        && new_space.PercentageResolutionInlineSize() != old_space.PercentageResolutionInlineSize()
    {
        return true;
    }
    if is_unspecified
        && new_space.AvailableSize().inline_size != old_space.AvailableSize().inline_size
    {
        return true;
    }
    if length.MayHavePercentDependence()
        && new_space.PercentageResolutionInlineSize() != old_space.PercentageResolutionInlineSize()
    {
        return true;
    }
    false
}

// cpp: layoutng/internal/layout_utils.cc:155-167
fn BlockLengthMayChange(
    length: &Length,
    new_space: &ConstraintSpace,
    old_space: &ConstraintSpace,
) -> bool {
    debug_assert_eq!(new_space.BlockAutoBehavior(), old_space.BlockAutoBehavior());
    if (length.HasStretch() || (length.HasAuto() && new_space.IsBlockAutoBehaviorStretch()))
        && new_space.AvailableSize().block_size != old_space.AvailableSize().block_size
    {
        return true;
    }
    false
}

// cpp: layoutng/internal/layout_utils.cc:169-204
fn BlockSizeMayChange(
    node: &BlockNode,
    new_space: &ConstraintSpace,
    old_space: &ConstraintSpace,
    layout_result: &LayoutResult,
) -> bool {
    debug_assert_eq!(new_space.IsFixedBlockSize(), old_space.IsFixedBlockSize());
    debug_assert_eq!(
        new_space.IsInitialBlockSizeIndefinite(),
        old_space.IsInitialBlockSizeIndefinite()
    );
    debug_assert_eq!(new_space.BlockAutoBehavior(), old_space.BlockAutoBehavior());
    debug_assert_eq!(new_space.IsTableCellChild(), old_space.IsTableCellChild());
    debug_assert_eq!(
        new_space.IsRestrictedBlockSizeTableCellChild(),
        old_space.IsRestrictedBlockSizeTableCellChild()
    );
    if node.IsQuirkyAndFillsViewport() {
        return true;
    }
    if new_space.IsFixedBlockSize() {
        if new_space.AvailableSize().block_size != old_space.AvailableSize().block_size {
            return true;
        }
    } else {
        let style = node.Style();
        if BlockLengthMayChange(style.LogicalHeight(), new_space, old_space)
            || BlockLengthMayChange(style.LogicalMinHeight(), new_space, old_space)
            || BlockLengthMayChange(style.LogicalMaxHeight(), new_space, old_space)
        {
            return true;
        }
        if layout_result
            .GetPhysicalFragment()
            .DependsOnPercentageBlockSize()
            && new_space.PercentageResolutionBlockSize()
                != old_space.PercentageResolutionBlockSize()
        {
            return true;
        }
    }
    false
}

// cpp: layoutng/internal/layout_utils.cc:209-258
fn SizeMayChange(
    node: &BlockNode,
    new_space: &ConstraintSpace,
    old_space: &ConstraintSpace,
    layout_result: &LayoutResult,
) -> bool {
    debug_assert_eq!(new_space.IsFixedInlineSize(), old_space.IsFixedInlineSize());
    debug_assert_eq!(new_space.BlockAutoBehavior(), old_space.BlockAutoBehavior());
    let style = node.Style();
    if new_space.IsFixedInlineSize() {
        if new_space.AvailableSize().inline_size != old_space.AvailableSize().inline_size {
            return true;
        }
    } else if InlineLengthMayChange(
        style,
        style.LogicalWidth(),
        LengthResolveType::kMainSize,
        new_space,
        old_space,
        layout_result,
    ) || InlineLengthMayChange(
        style,
        style.LogicalMinWidth(),
        LengthResolveType::kMinSize,
        new_space,
        old_space,
        layout_result,
    ) || InlineLengthMayChange(
        style,
        style.LogicalMaxWidth(),
        LengthResolveType::kMaxSize,
        new_space,
        old_space,
        layout_result,
    ) {
        return true;
    }
    if style.MayHavePadding()
        && new_space.PercentageResolutionInlineSize() != old_space.PercentageResolutionInlineSize()
        && (style.PaddingTop().HasPercent()
            || style.PaddingRight().HasPercent()
            || style.PaddingBottom().HasPercent()
            || style.PaddingLeft().HasPercent())
    {
        return true;
    }
    BlockSizeMayChange(node, new_space, old_space, layout_result)
}

// cpp: layoutng/internal/layout_utils.cc:269-472
fn CalculateSizeBasedLayoutCacheStatusWithGeometry(
    node: &BlockNode,
    fragment_geometry: &FragmentGeometry,
    layout_result: &LayoutResult,
    new_space: &ConstraintSpace,
    old_space: &ConstraintSpace,
) -> LayoutCacheStatus {
    let style = node.Style();
    let physical_fragment =
        unsafe { &*To::<PhysicalBoxFragment>(layout_result.GetPhysicalFragment() as *const _) };
    let fragment = LogicalBoxFragment::new(style.GetWritingDirection(), physical_fragment);
    if fragment_geometry.border_box_size.inline_size != fragment.InlineSize() {
        return LayoutCacheStatus::kNeedsLayout;
    }
    if style.MayHavePadding() && fragment_geometry.padding != fragment.Padding() {
        return LayoutCacheStatus::kNeedsLayout;
    }
    if node.IsTable() {
        if !new_space.AreBlockSizeConstraintsEqual(old_space)
            || BlockSizeMayChange(node, new_space, old_space, layout_result)
        {
            return LayoutCacheStatus::kNeedsLayout;
        }
        return LayoutCacheStatus::kHit;
    }

    let mut block_size = fragment_geometry.border_box_size.block_size;
    let is_initial_block_size_indefinite = block_size == kIndefiniteSize;
    if is_initial_block_size_indefinite {
        let intrinsic_block_size = {
            if !physical_fragment.IsFirstForNode() || !physical_fragment.GetBreakToken().is_null() {
                kIndefiniteSize
            } else if physical_fragment.IsFragmentationContextRoot()
                && style.LogicalMaxHeight().HasPercentOrStretch()
                && style.LogicalHeight().HasAuto()
            {
                kIndefiniteSize
            } else if (old_space.IsFixedBlockSize()
                || (old_space.IsBlockAutoBehaviorStretch() && style.LogicalHeight().HasAuto()))
                && (node.IsFlexibleBox()
                    || node.IsGrid()
                    || node.IsGridLanes()
                    || node.IsFieldsetContainer())
            {
                kIndefiniteSize
            } else if physical_fragment.DependsOnPercentageBlockSize()
                && new_space.PercentageResolutionBlockSize()
                    != old_space.PercentageResolutionBlockSize()
                && (node.IsFlexibleBox() || node.IsGrid() || node.IsGridLanes())
            {
                kIndefiniteSize
            } else {
                layout_result.IntrinsicBlockSize()
            }
        };
        block_size = super::length_utils::ComputeBlockSizeForFragment(
            new_space,
            node,
            &(fragment_geometry.border + fragment_geometry.padding),
            intrinsic_block_size,
            fragment_geometry.border_box_size.inline_size,
            kIndefiniteSize,
        );
        if block_size == kIndefiniteSize {
            return LayoutCacheStatus::kNeedsLayout;
        }
    }
    if block_size != fragment.BlockSize() {
        return LayoutCacheStatus::kNeedsLayout;
    }

    let has_descendant_that_depends_on_percentage_block_size =
        layout_result.HasDescendantThatDependsOnPercentageBlockSize();
    let is_old_initial_block_size_indefinite = layout_result.IsInitialBlockSizeIndefinite();
    if is_old_initial_block_size_indefinite != is_initial_block_size_indefinite
        && (node.IsFlexibleBox()
            || node.IsGrid()
            || node.IsGridLanes()
            || has_descendant_that_depends_on_percentage_block_size)
    {
        return LayoutCacheStatus::kNeedsLayout;
    }
    if has_descendant_that_depends_on_percentage_block_size
        && is_initial_block_size_indefinite
        && physical_fragment.DependsOnPercentageBlockSize()
    {
        debug_assert!(is_old_initial_block_size_indefinite);
        if new_space.PercentageResolutionBlockSize() != old_space.PercentageResolutionBlockSize() {
            return LayoutCacheStatus::kNeedsLayout;
        }
    }

    if new_space.IsTableCell() {
        debug_assert!(old_space.IsTableCell());
        if ComputeContentAlignmentForTableCell(style) == BlockContentAlignment::kBaseline {
            let new_alignment_baseline = new_space.TableCellAlignmentBaseline();
            let old_alignment_baseline = old_space.TableCellAlignmentBaseline();
            if new_alignment_baseline.is_none() && old_alignment_baseline.is_none() {
                return LayoutCacheStatus::kHit;
            }
            if new_alignment_baseline.is_none() && old_alignment_baseline.is_some() {
                return LayoutCacheStatus::kNeedsLayout;
            }
            if old_alignment_baseline.is_none() {
                return if new_alignment_baseline == physical_fragment.FirstBaseline() {
                    LayoutCacheStatus::kHit
                } else {
                    LayoutCacheStatus::kNeedsLayout
                };
            }
            if new_alignment_baseline != old_alignment_baseline {
                return LayoutCacheStatus::kNeedsLayout;
            }
        }
    }
    LayoutCacheStatus::kHit
}

// cpp: layoutng/internal/layout_utils.cc:474-498
fn IntrinsicSizeWillChange(
    node: &BlockNode,
    break_token: *const BlockBreakToken,
    cached_layout_result: &LayoutResult,
    new_space: &ConstraintSpace,
    fragment_geometry: *mut Option<FragmentGeometry>,
) -> bool {
    let style = node.Style();
    if new_space.IsInlineAutoBehaviorStretch() && !super::length_utils::NeedMinMaxSize(style) {
        return false;
    }
    debug_assert!(!fragment_geometry.is_null());
    let geometry = unsafe { &mut *fragment_geometry };
    if geometry.is_none() {
        *geometry = Some(super::length_utils::CalculateInitialFragmentGeometry(
            new_space,
            node,
            break_token,
            false,
        ));
    }
    let inline_size = LogicalFragment::new(
        style.GetWritingDirection(),
        cached_layout_result.GetPhysicalFragment(),
    )
    .InlineSize();
    geometry
        .as_ref()
        .expect("initial fragment geometry computed")
        .border_box_size
        .inline_size
        != inline_size
}

// cpp: layoutng/internal/layout_utils.h:19-30
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutCacheStatus {
    kHit,
    kNeedsLayout,
    kNeedsSimplifiedLayout,
    kCanReuseLines,
}

// cpp: layoutng/internal/layout_utils.h:32-45
// cpp: layoutng/internal/layout_utils.cc:502-541
pub fn CalculateSizeBasedLayoutCacheStatus(
    node: &BlockNode,
    break_token: *const BlockBreakToken,
    cached_layout_result: &LayoutResult,
    new_space: &ConstraintSpace,
    fragment_geometry: *mut Option<FragmentGeometry>,
) -> LayoutCacheStatus {
    debug_assert_eq!(cached_layout_result.Status(), EStatus::kSuccess);
    debug_assert!(!fragment_geometry.is_null());
    let old_space = cached_layout_result.GetConstraintSpaceForCaching();
    if !new_space.MaySkipLayout(old_space) {
        return LayoutCacheStatus::kNeedsLayout;
    }
    if new_space.AreInlineSizeConstraintsEqual(old_space)
        && new_space.AreBlockSizeConstraintsEqual(old_space)
    {
        if IntrinsicSizeWillChange(
            node,
            break_token,
            cached_layout_result,
            new_space,
            fragment_geometry,
        ) {
            return LayoutCacheStatus::kNeedsLayout;
        }
        if new_space.AreSizesEqual(old_space) {
            return LayoutCacheStatus::kHit;
        }
        if !SizeMayChange(node, new_space, old_space, cached_layout_result) {
            return LayoutCacheStatus::kHit;
        }
    }
    let geometry = unsafe { &mut *fragment_geometry };
    if geometry.is_none() {
        *geometry = Some(super::length_utils::CalculateInitialFragmentGeometry(
            new_space,
            node,
            break_token,
            false,
        ));
    }
    CalculateSizeBasedLayoutCacheStatusWithGeometry(
        node,
        geometry
            .as_ref()
            .expect("initial fragment geometry computed"),
        cached_layout_result,
        new_space,
        old_space,
    )
}

// cpp: layoutng/internal/layout_utils.h:47-63
// cpp: layoutng/internal/layout_utils.cc:543-731
pub fn MaySkipLayoutWithinBlockFormattingContext(
    cached_layout_result: &LayoutResult,
    new_space: &ConstraintSpace,
    bfc_block_offset: *mut Option<LayoutUnit>,
    block_offset_delta: *mut LayoutUnit,
    end_margin_strut: *mut MarginStrut,
) -> bool {
    debug_assert_eq!(cached_layout_result.Status(), EStatus::kSuccess);
    debug_assert!(!bfc_block_offset.is_null());
    debug_assert!(!block_offset_delta.is_null());
    debug_assert!(!end_margin_strut.is_null());
    let old_space = cached_layout_result.GetConstraintSpaceForCaching();
    let is_margin_strut_equal = old_space.GetMarginStrut() == new_space.GetMarginStrut();
    let old_clearance_offset = old_space.ClearanceOffset();
    let new_clearance_offset = new_space.ClearanceOffset();
    let is_pushed_by_floats = cached_layout_result.IsPushedByFloats();
    if is_pushed_by_floats {
        debug_assert!(old_space.HasFloats());
        if !is_margin_strut_equal {
            return false;
        }
        if cached_layout_result.BfcBlockOffset() != Some(old_space.ClearanceOffset()) {
            return false;
        }
        if old_clearance_offset - old_space.GetBfcOffset().block_offset
            > new_clearance_offset - new_space.GetBfcOffset().block_offset
        {
            return false;
        }
    }
    if cached_layout_result.SubtreeModifiedMarginStrut() && !is_margin_strut_equal {
        return false;
    }
    let physical_fragment = unsafe {
        &*To::<PhysicalBoxFragment>(cached_layout_result.GetPhysicalFragment() as *const _)
    };
    if physical_fragment.MayHaveDescendantAboveBlockStart()
        && (old_space.HasFloats() || new_space.HasFloats())
    {
        return false;
    }
    if cached_layout_result.IsSelfCollapsing() {
        if is_pushed_by_floats {
            return false;
        }
        let old_expected = old_space.ExpectedBfcBlockOffset();
        let new_expected = new_space.ExpectedBfcBlockOffset();
        if physical_fragment.HasAdjoiningObjectDescendants() {
            if old_expected
                < old_space
                    .GetExclusionSpace()
                    .ClearanceOffsetIncludingInitialLetter(EClear::kBoth)
                || new_expected
                    < new_space
                        .GetExclusionSpace()
                        .ClearanceOffsetIncludingInitialLetter(EClear::kBoth)
            {
                return false;
            }
        }
        unsafe { *block_offset_delta = new_expected - old_expected };
        unsafe { *bfc_block_offset = new_space.ForcedBfcBlockOffset() };
        if !cached_layout_result.SubtreeModifiedMarginStrut() {
            unsafe { *end_margin_strut = new_space.GetMarginStrut() };
        } else {
            debug_assert!(is_margin_strut_equal);
        }
        return true;
    }

    debug_assert!(unsafe { *bfc_block_offset }.is_some());
    debug_assert_eq!(
        old_space.AncestorHasClearancePastAdjoiningFloats(),
        new_space.AncestorHasClearancePastAdjoiningFloats()
    );
    let ancestor_has_clearance_past_adjoining_floats =
        new_space.AncestorHasClearancePastAdjoiningFloats();
    if ancestor_has_clearance_past_adjoining_floats {
        debug_assert!(old_space.ForcedBfcBlockOffset().is_some());
        debug_assert!(new_space.ForcedBfcBlockOffset().is_some());
        debug_assert_eq!(old_space.ForcedBfcBlockOffset(), Some(old_clearance_offset));
        debug_assert_eq!(new_space.ForcedBfcBlockOffset(), Some(new_clearance_offset));
    } else if old_space.ForcedBfcBlockOffset() != new_space.ForcedBfcBlockOffset() {
        return false;
    }
    if unsafe { *bfc_block_offset }.expect("regular block has BFC offset")
        < old_space
            .GetExclusionSpace()
            .ClearanceOffsetIncludingInitialLetter(EClear::kBoth)
    {
        return false;
    }
    if is_pushed_by_floats || ancestor_has_clearance_past_adjoining_floats {
        debug_assert_eq!(unsafe { *bfc_block_offset }, Some(old_clearance_offset));
        unsafe { *block_offset_delta = new_clearance_offset - old_clearance_offset };
        unsafe { *bfc_block_offset = Some(new_clearance_offset) };
    } else if is_margin_strut_equal {
        unsafe {
            *block_offset_delta =
                new_space.GetBfcOffset().block_offset - old_space.GetBfcOffset().block_offset;
            *bfc_block_offset = Some(
                (*bfc_block_offset).expect("regular block has BFC offset") + *block_offset_delta,
            );
        }
    } else {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!cached_layout_result.SubtreeModifiedMarginStrut());
            let old_bfc_block_offset =
                old_space.GetBfcOffset().block_offset + old_space.GetMarginStrut().Sum();
            debug_assert_eq!(unsafe { *bfc_block_offset }, Some(old_bfc_block_offset));
        }
        let new_bfc_block_offset =
            new_space.GetBfcOffset().block_offset + new_space.GetMarginStrut().Sum();
        unsafe {
            *block_offset_delta =
                new_bfc_block_offset - (*bfc_block_offset).expect("regular block has BFC offset");
            *bfc_block_offset = Some(
                (*bfc_block_offset).expect("regular block has BFC offset") + *block_offset_delta,
            );
        }
    }
    if unsafe { *bfc_block_offset }.expect("regular block has BFC offset")
        < new_space
            .GetExclusionSpace()
            .ClearanceOffsetIncludingInitialLetter(EClear::kBoth)
    {
        return false;
    }
    true
}

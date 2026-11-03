#![allow(non_snake_case)]

use foundation::{kIndefiniteSize, EClear, LayoutUnit, Length, MarginStrut, To};
use layout_fragment::block_break_token::BlockBreakToken;
use layout_fragment::layout_result::{EStatus, LayoutResult};
use layout_fragment::logical_box_fragment::LogicalBoxFragment;
use layout_fragment::logical_fragment::LogicalFragment;
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_style::style::computed_style::ComputedStyle;

use super::super::block_node::BlockNode;
use super::super::constraint_space::ConstraintSpace;
use super::super::layout_alignment_utils::{
    BlockContentAlignment, ComputeContentAlignmentForTableCell,
};

// Alignment and static-position policy live in layout_alignment_utils. This
// module owns only cache-admission work.
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
        block_size = super::super::length_utils::ComputeBlockSizeForFragment(
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
    if new_space.IsInlineAutoBehaviorStretch() && !super::super::length_utils::NeedMinMaxSize(style)
    {
        return false;
    }
    debug_assert!(!fragment_geometry.is_null());
    let geometry = unsafe { &mut *fragment_geometry };
    if geometry.is_none() {
        *geometry = Some(
            super::super::length_utils::CalculateInitialFragmentGeometry(
                new_space,
                node,
                break_token,
                false,
            ),
        );
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

struct SizeBasedLayoutCacheDecision<'a> {
    node: &'a BlockNode,
    break_token: *const BlockBreakToken,
    cached_layout_result: &'a LayoutResult,
    new_space: &'a ConstraintSpace,
    old_space: &'a ConstraintSpace,
    fragment_geometry: *mut Option<FragmentGeometry>,
}

impl layout_reuse::LayoutCacheDecisionBackend for SizeBasedLayoutCacheDecision<'_> {
    fn may_skip_layout(&self) -> bool {
        self.new_space.MaySkipLayout(self.old_space)
    }

    fn inline_size_constraints_equal(&self) -> bool {
        self.new_space.AreInlineSizeConstraintsEqual(self.old_space)
    }

    fn block_size_constraints_equal(&self) -> bool {
        self.new_space.AreBlockSizeConstraintsEqual(self.old_space)
    }

    fn intrinsic_size_will_change(&mut self) -> bool {
        IntrinsicSizeWillChange(
            self.node,
            self.break_token,
            self.cached_layout_result,
            self.new_space,
            self.fragment_geometry,
        )
    }

    fn sizes_equal(&self) -> bool {
        self.new_space.AreSizesEqual(self.old_space)
    }

    fn size_may_change(&self) -> bool {
        SizeMayChange(
            self.node,
            self.new_space,
            self.old_space,
            self.cached_layout_result,
        )
    }

    fn calculate_with_geometry(&mut self) -> LayoutCacheStatus {
        let geometry = unsafe { &mut *self.fragment_geometry };
        if geometry.is_none() {
            *geometry = Some(
                super::super::length_utils::CalculateInitialFragmentGeometry(
                    self.new_space,
                    self.node,
                    self.break_token,
                    false,
                ),
            );
        }
        CalculateSizeBasedLayoutCacheStatusWithGeometry(
            self.node,
            geometry
                .as_ref()
                .expect("initial fragment geometry computed"),
            self.cached_layout_result,
            self.new_space,
            self.old_space,
        )
    }
}

pub use layout_reuse::LayoutCacheStatus;

// cpp: layoutng/internal/layout_utils.h:32-45
// cpp: layoutng/internal/layout_utils.cc:502-541
#[inline(never)]
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
    layout_reuse::calculate_size_based_layout_cache_status(&mut SizeBasedLayoutCacheDecision {
        node,
        break_token,
        cached_layout_result,
        new_space,
        old_space,
        fragment_geometry,
    })
}

// cpp: layoutng/internal/layout_utils.h:47-63
// cpp: layoutng/internal/layout_utils.cc:543-731
#[inline(never)]
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
    let physical_fragment = unsafe {
        &*To::<PhysicalBoxFragment>(cached_layout_result.GetPhysicalFragment() as *const _)
    };
    if !cached_layout_result.IsSelfCollapsing() {
        debug_assert_eq!(
            old_space.AncestorHasClearancePastAdjoiningFloats(),
            new_space.AncestorHasClearancePastAdjoiningFloats()
        );
    }
    let Some(adjustment) = layout_reuse::calculate_block_formatting_context_reuse(
        &layout_reuse::BlockFormattingContextReuseInput {
            cached_result_bfc_block_offset: cached_layout_result.BfcBlockOffset(),
            current_bfc_block_offset: unsafe { *bfc_block_offset },
            is_pushed_by_floats: cached_layout_result.IsPushedByFloats(),
            subtree_modified_margin_strut: cached_layout_result.SubtreeModifiedMarginStrut(),
            may_have_descendant_above_block_start: physical_fragment
                .MayHaveDescendantAboveBlockStart(),
            is_self_collapsing: cached_layout_result.IsSelfCollapsing(),
            has_adjoining_object_descendants: physical_fragment.HasAdjoiningObjectDescendants(),
            old_has_floats: old_space.HasFloats(),
            new_has_floats: new_space.HasFloats(),
            old_margin_strut: old_space.GetMarginStrut(),
            new_margin_strut: new_space.GetMarginStrut(),
            old_clearance_offset: old_space.ClearanceOffset(),
            new_clearance_offset: new_space.ClearanceOffset(),
            old_bfc_block_offset: old_space.GetBfcOffset().block_offset,
            new_bfc_block_offset: new_space.GetBfcOffset().block_offset,
            old_expected_bfc_block_offset: old_space.ExpectedBfcBlockOffset(),
            new_expected_bfc_block_offset: new_space.ExpectedBfcBlockOffset(),
            old_forced_bfc_block_offset: old_space.ForcedBfcBlockOffset(),
            new_forced_bfc_block_offset: new_space.ForcedBfcBlockOffset(),
            ancestor_has_clearance_past_adjoining_floats: new_space
                .AncestorHasClearancePastAdjoiningFloats(),
            old_exclusion_clearance: old_space
                .GetExclusionSpace()
                .ClearanceOffsetIncludingInitialLetter(EClear::kBoth),
            new_exclusion_clearance: new_space
                .GetExclusionSpace()
                .ClearanceOffsetIncludingInitialLetter(EClear::kBoth),
        },
    ) else {
        return false;
    };
    unsafe {
        *bfc_block_offset = adjustment.bfc_block_offset;
        *block_offset_delta = adjustment.block_offset_delta;
        if let Some(updated_end_margin_strut) = adjustment.end_margin_strut {
            *end_margin_strut = updated_end_margin_strut;
        }
    }
    true
}

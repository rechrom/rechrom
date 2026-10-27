//! Native LayoutNG adapter for the complete cached-layout-result lookup.
//!
//! This file is compiled by `layoutng` at its reciprocal assembly boundary,
//! not by the standalone `layout_reuse` crate. It lives with the reuse owner
//! so the hot decision tree has a package-qualified sampling symbol while the
//! standalone crate remains independent of native LayoutObject types.

#![allow(non_snake_case)]

use foundation::{DynamicTo, EClear, LayoutUnit, MakeGarbageCollected, To};
use layout_fragment::block_break_token::BlockBreakToken;
use layout_fragment::layout_result::{EStatus, LayoutResult};
use layout_fragment::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;

use crate::internal::block_node::BlockNode;
use crate::internal::break_appeal::BreakAppeal;
use crate::internal::column_spanner_path::ColumnSpannerPath;
use crate::internal::constraint_space::{ConstraintSpace, LayoutResultCacheSlot};
use crate::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use crate::internal::early_break::EarlyBreak;
use crate::internal::fragmentation_utils::{FragmentainerOffsetAtBfc, IsBreakInside};
use crate::internal::layout_block::LayoutBlock;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_utils::{
    CalculateSizeBasedLayoutCacheStatus, LayoutCacheStatus,
    MaySkipLayoutWithinBlockFormattingContext,
};

/// Owns the complete native cache lookup decision tree. The LayoutBox method
/// is intentionally a thin ABI-shaped adapter to this out-of-line function.
#[inline(never)]
pub fn cached_layout_result(
    box_: &mut LayoutBox,
    new_space: &ConstraintSpace,
    break_token: *const BlockBreakToken,
    early_break: *const EarlyBreak,
    column_spanner_path: *const ColumnSpannerPath,
    initial_fragment_geometry: *mut Option<FragmentGeometry>,
    out_cache_status: &mut LayoutCacheStatus,
) -> *const LayoutResult {
    box_.CheckIsNotDestroyed();
    *out_cache_status = LayoutCacheStatus::kNeedsLayout;
    if box_.SelfNeedsFullLayout() || box_.ShouldSkipLayoutCache() || !early_break.is_null() {
        return std::ptr::null();
    }

    let use_layout_cache_slot =
        new_space.CacheSlot() == LayoutResultCacheSlot::kLayout && box_.HasCachedLayoutResults();
    let mut cached_layout_result = if use_layout_cache_slot {
        box_.GetCachedLayoutResult(break_token)
    } else {
        box_.GetCachedMeasureResult(new_space, initial_fragment_geometry)
    };
    if cached_layout_result.is_null() {
        return std::ptr::null();
    }
    let cached = unsafe { &*cached_layout_result };
    debug_assert_eq!(cached.Status(), EStatus::kSuccess);
    let mut cache_status = LayoutCacheStatus::kHit;
    let physical_fragment =
        unsafe { &*To::<PhysicalBoxFragment>(cached.GetPhysicalFragment() as *const _) };
    let fragment_break_token = physical_fragment.GetBreakToken();
    if (!fragment_break_token.is_null() && unsafe { &*fragment_break_token }.IsRepeated())
        || (!break_token.is_null() && unsafe { &*break_token }.IsRepeated())
    {
        return std::ptr::null();
    }

    let is_blocked_by_display_lock = box_.ChildLayoutBlockedByDisplayLock();
    let child_needs_layout = !is_blocked_by_display_lock && box_.ChildNeedsFullLayout();
    if box_.NeedsSimplifiedLayoutOnly() {
        cache_status = LayoutCacheStatus::kNeedsSimplifiedLayout;
    } else if child_needs_layout {
        if !box_.ChildrenInline() || !physical_fragment.HasItems() || !use_layout_cache_slot {
            return std::ptr::null();
        }
        if physical_fragment.NeedsOOFPositionedInfoPropagation()
            || !cached.GetExclusionSpace().IsEmpty()
            || new_space.HasFloats()
            || physical_fragment.HasMovedChildren()
        {
            return std::ptr::null();
        }
        cache_status = LayoutCacheStatus::kCanReuseLines;
    }

    let node = BlockNode::new(box_ as *mut LayoutBox);
    let mut size_cache_status = LayoutCacheStatus::kHit;
    if use_layout_cache_slot {
        size_cache_status = CalculateSizeBasedLayoutCacheStatus(
            &node,
            break_token,
            cached,
            new_space,
            initial_fragment_geometry,
        );
    }
    if size_cache_status == LayoutCacheStatus::kNeedsLayout {
        return std::ptr::null();
    }
    if cached.HasOrthogonalFallbackSizeDescendant()
        && unsafe { &mut *box_.View() }.AffectedByResizedInitialContainingBlock(cached)
    {
        return std::ptr::null();
    }
    if !physical_fragment.ChildrenValid()
        && (size_cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
            || cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout)
    {
        return std::ptr::null();
    }
    if size_cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
        && cache_status == LayoutCacheStatus::kHit
    {
        cache_status = LayoutCacheStatus::kNeedsSimplifiedLayout;
    }
    if cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout {
        if box_.IsLayoutReplaced()
            || (!use_layout_cache_slot && box_.GetCachedLayoutResult(break_token).is_null())
        {
            return std::ptr::null();
        }
    }
    if !break_token.is_null()
        && unsafe { &*break_token }.IsBreakBefore()
        && cached.GetBreakAppeal() < BreakAppeal::kBreakAppealPerfect
    {
        return std::ptr::null();
    }

    let bfc_line_offset = new_space.GetBfcOffset().line_offset;
    let mut bfc_block_offset = cached.BfcBlockOffset();
    let mut block_offset_delta = LayoutUnit::default();
    let mut end_margin_strut = cached.EndMarginStrut();
    let is_fragmented = IsBreakInside(break_token)
        || !physical_fragment.GetBreakToken().is_null()
        || box_.PhysicalFragmentCount() > 1;
    let old_space = cached.GetConstraintSpaceForCaching();
    let are_bfc_offsets_equal = new_space.GetBfcOffset() == old_space.GetBfcOffset()
        && new_space.ExpectedBfcBlockOffset() == old_space.ExpectedBfcBlockOffset()
        && new_space.ForcedBfcBlockOffset() == old_space.ForcedBfcBlockOffset();
    let is_margin_strut_equal = new_space.GetMarginStrut() == old_space.GetMarginStrut();
    let is_exclusion_space_equal = new_space.GetExclusionSpace() == old_space.GetExclusionSpace();
    let is_clearance_offset_equal = new_space.ClearanceOffset() == old_space.ClearanceOffset();
    let is_new_formatting_context = physical_fragment.IsFormattingContextRoot();

    if !is_new_formatting_context
        && (!are_bfc_offsets_equal
            || !is_exclusion_space_equal
            || !is_margin_strut_equal
            || !is_clearance_offset_equal)
    {
        debug_assert!(!box_.CreatesNewFormattingContext());
        if cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
            || cache_status == LayoutCacheStatus::kCanReuseLines
        {
            return std::ptr::null();
        }
        debug_assert_eq!(cache_status, LayoutCacheStatus::kHit);
        if !MaySkipLayoutWithinBlockFormattingContext(
            cached,
            new_space,
            &mut bfc_block_offset,
            &mut block_offset_delta,
            &mut end_margin_strut,
        ) {
            return std::ptr::null();
        }
    }

    if new_space.HasBlockFragmentation() {
        debug_assert!(old_space.HasBlockFragmentation());
        if cache_status != LayoutCacheStatus::kHit
            || physical_fragment.HasNestedMulticolsWithOOFs()
            || physical_fragment.HasOutOfFlowFragmentChild()
            || !column_spanner_path.is_null()
            || !cached.GetColumnSpannerPath().is_null()
        {
            return std::ptr::null();
        }
        if new_space.FragmentainerBlockSize() != old_space.FragmentainerBlockSize()
            || new_space.FragmentainerOffset() != old_space.FragmentainerOffset()
        {
            if is_fragmented
                || cached.MinimalSpaceShortage().is_some()
                || physical_fragment.IsFragmentationContextRoot()
                || cached.IsBlockSizeForFragmentationClamped()
                || cached.IsTruncatedByFragmentationLine()
            {
                return std::ptr::null();
            }

            let do_floats_cross_fragmentation_line = || {
                let result_exclusion_space = cached.GetExclusionSpace();
                if result_exclusion_space != old_space.GetExclusionSpace() {
                    let block_end_offset = FragmentainerOffsetAtBfc(new_space)
                        + result_exclusion_space.ClearanceOffset(EClear::kBoth);
                    if block_end_offset > new_space.FragmentainerBlockSize() {
                        return true;
                    }
                }
                false
            };

            if bfc_block_offset.is_none() && cached.IsSelfCollapsing() {
                if old_space.IsInitialColumnBalancingPass() || do_floats_cross_fragmentation_line()
                {
                    return std::ptr::null();
                }
            } else {
                if physical_fragment.IsInlineFormattingContext()
                    && !is_new_formatting_context
                    && do_floats_cross_fragmentation_line()
                {
                    return std::ptr::null();
                }
                let block_size_for_fragmentation = cached.BlockSizeForFragmentation();
                let block_end_offset = FragmentainerOffsetAtBfc(new_space)
                    + bfc_block_offset.unwrap_or_default()
                    + block_size_for_fragmentation;
                if block_end_offset > new_space.FragmentainerBlockSize() {
                    return std::ptr::null();
                }
            }
            if old_space.IsInitialColumnBalancingPass() {
                if physical_fragment.HasOutOfFlowInFragmentainerSubtree() {
                    return std::ptr::null();
                }
                let block = DynamicTo::<LayoutBlock>(box_ as *mut LayoutBox as *mut LayoutObject);
                if !block.is_null() && unsafe { &*block }.IsFragmentationContextRoot() {
                    return std::ptr::null();
                }
            }
        }
    }

    if is_fragmented {
        if cached.GetExclusionSpace().HasFragmentainerBreak()
            || cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
        {
            return std::ptr::null();
        }
    }
    *out_cache_status = cache_status;
    if cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
        || cache_status == LayoutCacheStatus::kCanReuseLines
    {
        return cached_layout_result;
    }
    physical_fragment.CheckType();
    debug_assert_eq!(*out_cache_status, LayoutCacheStatus::kHit);
    if use_layout_cache_slot && !is_blocked_by_display_lock && box_.NeedsScrollableOverflowRecalc()
    {
        #[cfg(debug_assertions)]
        let cloned_cached_layout_result = LayoutResult::CloneWithPostLayoutFragments(cached);
        if !DisableLayoutSideEffectsScope::IsDisabled() {
            box_.RecalcScrollableOverflow();
        }
        cached_layout_result = box_.GetCachedLayoutResult(break_token);
        #[cfg(debug_assertions)]
        unsafe { &*cloned_cached_layout_result }
            .CheckSameForSimplifiedLayout(unsafe { &*cached_layout_result }, false);
    }

    let cached = unsafe { &*cached_layout_result };
    if box_.IsTableRow() {
        cached.ReplaceTableRowDataForCaching(
            new_space.CloneTableDataHandle(),
            new_space.TableRowIndex(),
        );
    }
    if are_bfc_offsets_equal && is_exclusion_space_equal && is_margin_strut_equal {
        cached
            .GetExclusionSpace()
            .MoveAndUpdateDerivedGeometry(new_space.GetExclusionSpace());
        return cached_layout_result;
    }
    MakeGarbageCollected(LayoutResult::copy_with_new_space(
        cached,
        new_space,
        &end_margin_strut,
        bfc_line_offset,
        bfc_block_offset,
        block_offset_delta,
    ))
}

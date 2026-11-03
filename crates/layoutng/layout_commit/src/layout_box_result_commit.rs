use super::*;

// cpp: layoutng/internal/layout_box_fragment_data.cc:42-66
#[inline(never)]
fn check_did_add_fragment(
    box_: &LayoutBox,
    new_fragment: &PhysicalBoxFragment,
    new_fragment_index: usize,
) {
    #[cfg(debug_assertions)]
    {
        if new_fragment.HasItems() {
            debug_assert!(box_.ChildrenInline());
        }
        for (index, fragment) in box_.PhysicalFragments().into_iter().enumerate() {
            debug_assert_eq!(fragment.IsFirstForNode(), index == 0);
            let items = fragment.Items();
            if !items.is_null() {
                unsafe { &*items }.CheckAllItemsAreValid();
            }
            if index == new_fragment_index {
                break;
            }
        }
    }
    #[cfg(not(debug_assertions))]
    let _ = (box_, new_fragment, new_fragment_index);
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:245-290
#[inline(never)]
pub(super) fn LayoutBoxSetLayoutResult(
    box_: &mut LayoutBox,
    result: *const LayoutResult,
    index: usize,
) {
    box_.CheckIsNotDestroyed();
    let result_ref = unsafe { &*result };
    debug_assert_eq!(result_ref.Status(), EStatus::kSuccess);
    let box_fragment = To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _);
    let box_fragment = unsafe { &*box_fragment };
    if index != usize::MAX && box_.layout_results_.len() > index {
        if box_.layout_results_.len() > index + 1 {
            let break_token = box_fragment.GetBreakToken();
            if break_token.is_null()
                || unsafe { &*break_token }.IsCausedByColumnSpanner()
                || box_fragment.IsFragmentationContextRoot()
            {
                if box_fragment.IsInlineFormattingContext() {
                    FragmentItems::ClearAssociatedFragments(
                        box_ as *mut LayoutBox as *mut LayoutObject,
                    );
                }
                LayoutBoxShrinkLayoutResults(box_, index + 1);
            }
        }
        LayoutBoxReplaceLayoutResult(box_, result, index);
        return;
    }
    debug_assert!(index == box_.layout_results_.len() || index == usize::MAX);
    LayoutBoxAppendLayoutResult(box_, result);
    if box_fragment.GetBreakToken().is_null() {
        LayoutBoxFinalizeLayoutResults(box_);
    }
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:292-300
#[inline(never)]
pub(super) fn LayoutBoxAppendLayoutResult(box_: &mut LayoutBox, result: *const LayoutResult) {
    box_.CheckIsNotDestroyed();
    let fragment = To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment() as *const _);
    debug_assert!(!DisableLayoutSideEffectsScope::IsDisabled());
    box_.layout_results_
        .push(Member::from_ptr(result as *mut LayoutResult));
    box_.InvalidateCachedGeometry();
    check_did_add_fragment(box_, unsafe { &*fragment }, usize::MAX);
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:302-341
#[inline(never)]
pub(super) fn LayoutBoxReplaceLayoutResult(
    box_: &mut LayoutBox,
    result: *const LayoutResult,
    index: usize,
) {
    box_.CheckIsNotDestroyed();
    debug_assert!(index <= box_.layout_results_.len());
    let old_result = box_.layout_results_[index].Get();
    if old_result == result as *mut LayoutResult {
        return;
    }
    let fragment = To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment() as *const _);
    let fragment = unsafe { &*fragment };
    let old_fragment = unsafe { &*old_result }.GetPhysicalFragment();
    let got_new_fragment = old_fragment as *const _ != fragment as *const _ as *const _;
    if got_new_fragment {
        if box_.HasFragmentItems() {
            FragmentItems::ClearAssociatedFragments(box_ as *mut LayoutBox as *mut LayoutObject);
        }
        if box_.layout_results_.len() > 1 && fragment.Size() != old_fragment.Size() {
            box_.SetShouldDoFullPaintInvalidation();
        }
    }
    debug_assert!(!DisableLayoutSideEffectsScope::IsDisabled());
    box_.layout_results_[index] = Member::from_ptr(result as *mut LayoutResult);
    box_.InvalidateCachedGeometry();
    check_did_add_fragment(box_, fragment, index);
    if got_new_fragment && fragment.GetBreakToken().is_null() {
        debug_assert_eq!(index, box_.layout_results_.len() - 1);
        LayoutBoxFinalizeLayoutResults(box_);
    }
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:343-356
#[inline(never)]
pub(super) fn LayoutBoxFinalizeLayoutResults(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    debug_assert!(!box_.layout_results_.is_empty());
    let last = box_
        .layout_results_
        .last()
        .expect("layout result required")
        .Get();
    debug_assert!(unsafe { &*last }
        .GetPhysicalFragment()
        .GetBreakToken()
        .is_null());
    #[cfg(feature = "expensive_dchecks")]
    box_.CheckMayHaveFragmentItems();
    if box_.HasFragmentItems() {
        let results = &box_.layout_results_ as *const LayoutResultList;
        let block_flow = DynamicTo::<LayoutBlockFlow>(box_ as *mut LayoutBox as *mut LayoutObject);
        debug_assert!(!block_flow.is_null());
        FragmentItems::FinalizeAfterLayout(unsafe { &*results }, unsafe { &mut *block_flow });
    }
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:358-387
#[inline(never)]
pub(super) fn LayoutBoxRebuildFragmentTreeSpine(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    debug_assert!(box_.PhysicalFragmentCount() != 0);
    let mut container = box_ as *mut LayoutBox;
    while !container.is_null()
        && unsafe { &*container }.PhysicalFragmentCount() != 0
        && !unsafe { &*container }.NeedsLayout()
    {
        let current = unsafe { &mut *container };
        for result in current.layout_results_.iter_mut() {
            let cloned = LayoutResult::CloneWithPostLayoutFragments(unsafe { &*result.Get() });
            *result = Member::from_ptr(cloned as *mut LayoutResult);
        }
        let measure_cache = current.measure_cache_.Get();
        if !measure_cache.is_null() {
            unsafe { &mut *measure_cache }.Clear();
        }
        container = current.ContainingNGBox();
    }
    if !container.is_null() && unsafe { &*container }.NeedsLayout() {
        unsafe { &mut *container }.SetHasBrokenSpine();
    }
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:389-397
#[inline(never)]
pub(super) fn LayoutBoxShrinkLayoutResults(box_: &mut LayoutBox, results_to_keep: usize) {
    box_.CheckIsNotDestroyed();
    debug_assert!(box_.layout_results_.len() >= results_to_keep);
    debug_assert!(!DisableLayoutSideEffectsScope::IsDisabled());
    box_.layout_results_.truncate(results_to_keep);
    box_.InvalidateCachedGeometry();
}

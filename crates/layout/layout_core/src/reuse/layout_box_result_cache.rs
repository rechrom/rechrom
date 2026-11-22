use super::*;

// cpp: layoutng/internal/layout_box_fragment_data.cc:186-207
#[inline(never)]
pub(super) fn LayoutBoxAddMeasureLayoutResult(box_: &mut LayoutBox, result: *const LayoutResult) {
    box_.CheckIsNotDestroyed();
    let result_ref = unsafe { &*result };
    if result_ref.Status() != EStatus::kSuccess {
        return;
    }
    if result_ref.GetConstraintSpaceForCaching().CacheSlot() != LayoutResultCacheSlot::kMeasure {
        return;
    }
    let box_fragment = To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _);
    debug_assert!(unsafe { &*box_fragment }.IsOnlyForNode());
    if box_.measure_cache_.Get().is_null() {
        box_.measure_cache_ = Member::from_ptr(MakeGarbageCollected(MeasureCache::default()));
    }
    let measure_cache = unsafe { &mut *box_.measure_cache_.Get() };
    if box_.NeedsLayout() && !box_.NeedsSimplifiedLayoutOnly() {
        measure_cache.Clear();
    }
    measure_cache.Add(result);
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:209-243
#[inline(never)]
pub(super) fn LayoutBoxSetCachedLayoutResult(
    box_: &mut LayoutBox,
    result: *const LayoutResult,
    index: usize,
) {
    box_.CheckIsNotDestroyed();
    let result_ref = unsafe { &*result };
    if result_ref.GetConstraintSpaceForCaching().CacheSlot() == LayoutResultCacheSlot::kMeasure {
        debug_assert!(result_ref.GetPhysicalFragment().GetBreakToken().is_null());
        let box_fragment = To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _);
        debug_assert!(unsafe { &*box_fragment }.IsOnlyForNode());
        debug_assert_eq!(index, 0);
        LayoutBoxAddMeasureLayoutResult(box_, result);
        if box_.IsTableCell() {
            unsafe { InvalidateLayoutResultCacheAfterMeasureProvider(box_) };
        }
    } else if box_.NeedsLayout() && !box_.NeedsSimplifiedLayoutOnly() {
        let measure_cache = box_.measure_cache_.Get();
        if !measure_cache.is_null() {
            unsafe { &mut *measure_cache }.Clear();
        }
    }
    let measure_cache = box_.measure_cache_.Get();
    if !measure_cache.is_null() {
        unsafe { &*measure_cache }.SetFragmentChildrenInvalid(result);
    }
    box_.SetLayoutResult(result, index);
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:412-422
#[inline(never)]
pub(super) fn LayoutBoxGetCachedLayoutResult(
    box_: &LayoutBox,
    break_token: *const BlockBreakToken,
) -> *const LayoutResult {
    box_.CheckIsNotDestroyed();
    let index = FragmentIndex(break_token);
    if index >= box_.layout_results_.len() {
        return std::ptr::null();
    }
    let result = box_.layout_results_[index].Get();
    debug_assert!(
        !unsafe { &*result }
            .GetPhysicalFragment()
            .IsLayoutObjectDestroyedOrMoved()
            || box_.BeingDestroyed()
    );
    result
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:424-450
#[inline(never)]
pub(super) fn LayoutBoxGetCachedMeasureResult(
    box_: &LayoutBox,
    space: &ConstraintSpace,
    fragment_geometry: *mut Option<FragmentGeometry>,
) -> *const LayoutResult {
    box_.CheckIsNotDestroyed();
    let measure_cache = box_.measure_cache_.Get();
    if measure_cache.is_null() {
        return std::ptr::null();
    }
    if !box_.layout_results_.is_empty() {
        let first_fragment = unsafe { &*box_.GetPhysicalFragment(0) };
        if !first_fragment.GetBreakToken().is_null() {
            return std::ptr::null();
        }
    }
    debug_assert!(!fragment_geometry.is_null());
    unsafe { &mut *measure_cache }.Find(
        &BlockNode::new(box_ as *const LayoutBox as *mut LayoutBox),
        space,
        unsafe { &mut *fragment_geometry },
    )
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:452-455
#[inline(never)]
pub(super) fn LayoutBoxGetSingleCachedLayoutResult(box_: &LayoutBox) -> *const LayoutResult {
    debug_assert!(box_.layout_results_.len() <= 1);
    LayoutBoxGetCachedLayoutResult(box_, std::ptr::null())
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:457-460
#[inline(never)]
pub(super) fn LayoutBoxGetSingleCachedMeasureResultForTesting(
    box_: &LayoutBox,
) -> *const LayoutResult {
    box_.CheckIsNotDestroyed();
    let measure_cache = box_.measure_cache_.Get();
    if measure_cache.is_null() {
        std::ptr::null()
    } else {
        unsafe { &*measure_cache }.GetLastForTesting()
    }
}

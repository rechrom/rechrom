use super::*;

// cpp: layoutng/internal/layout_box_fragment_data.cc:119-131
#[inline(never)]
pub(super) fn LayoutBoxDisassociatePhysicalFragments(box_: &mut LayoutBox) {
    box_.CheckIsNotDestroyed();
    if box_.FirstInlineFragmentItemIndex() != 0 {
        FragmentItems::LayoutObjectWillBeDestroyed(box_);
        box_.ClearFirstInlineFragmentItemIndex();
    }
    let measure_cache = box_.measure_cache_.Get();
    if !measure_cache.is_null() {
        unsafe { &*measure_cache }.LayoutObjectWillBeDestroyed();
    }
    for result in box_.layout_results_.iter() {
        unsafe { &*result.Get() }
            .GetPhysicalFragment()
            .LayoutObjectWillBeDestroyed();
    }
}

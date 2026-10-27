// Scrollable-overflow recomputation reads and mutates resident post-layout
// fragments. Keep physical/inline recursion and per-fragment calculation in a
// fragment-owned sampling domain.

use super::*;

// cpp: layoutng/internal/layout_box_core_services.cc:39-52
#[inline(never)]
fn RecalcFragmentChildren(
    result: &mut RecalcScrollableOverflowResult,
    fragment: &PhysicalBoxFragment,
) {
    for child in fragment.PostLayoutChildren().iter() {
        let child_fragment = child.get();
        if !unsafe { &*child_fragment }.GetLayoutObject().is_null() {
            let box_fragment = DynamicTo::<PhysicalBoxFragment>(child_fragment);
            if !box_fragment.is_null() {
                let owner = unsafe { &*box_fragment }.MutableOwnerLayoutBox();
                if !owner.is_null() {
                    let child_result = unsafe { &mut *owner }.RecalcScrollableOverflow();
                    result.Unite(&child_result);
                }
            }
        } else {
            let fragmentainer = DynamicTo::<PhysicalBoxFragment>(child_fragment);
            if !fragmentainer.is_null() {
                RecalcFragmentChildren(result, unsafe { &*fragmentainer });
            }
        }
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:153-189
#[inline(never)]
pub fn LayoutBoxRecalcScrollableOverflowNG(box_: &mut LayoutBox) -> RecalcScrollableOverflowResult {
    box_.CheckIsNotDestroyed();
    if box_.NeedsLayout() {
        return RecalcScrollableOverflowResult::default();
    }
    let mut child_result = RecalcScrollableOverflowResult::default();
    if box_.ChildNeedsScrollableOverflowRecalc() {
        child_result = box_.RecalcChildScrollableOverflowNG();
    }
    let recalculate =
        box_.SelfNeedsScrollableOverflowRecalc() || child_result.scrollable_overflow_changed;
    let mut rebuild = child_result.rebuild_fragment_tree;
    let mut changed = false;
    if rebuild || recalculate {
        for result in box_.GetLayoutResults() {
            let result = unsafe { &*result.Get() };
            let fragment = result.GetPhysicalFragment();
            let fragment = unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
            if recalculate {
                let old_overflow = fragment.ScrollableOverflow();
                let fragmented = result
                    .GetConstraintSpaceForCaching()
                    .HasBlockFragmentation();
                let new_overflow =
                    ScrollableOverflowCalculator::RecalculateScrollableOverflowForFragment(
                        fragment, fragmented,
                    );
                if old_overflow != new_overflow {
                    fragment
                        .GetMutableForStyleRecalc()
                        .SetScrollableOverflow(new_overflow);
                    changed = true;
                    rebuild = true;
                }
            }
        }
        box_.SetScrollableOverflowFromLayoutResults();
    }
    changed =
        changed && !box_.ShouldApplyLayoutContainment() && !box_.ShouldClipOverflowAlongBothAxis();
    RecalcScrollableOverflowResult {
        scrollable_overflow_changed: changed,
        rebuild_fragment_tree: rebuild,
    }
}

// cpp: layoutng/internal/layout_box_core_services.cc:191-215
#[inline(never)]
pub fn LayoutBoxRecalcChildScrollableOverflowNG(
    box_: &mut LayoutBox,
) -> RecalcScrollableOverflowResult {
    box_.CheckIsNotDestroyed();
    debug_assert!(box_.ChildNeedsScrollableOverflowRecalc());
    box_.ClearChildNeedsScrollableOverflowRecalc();
    #[cfg(debug_assertions)]
    let _scope = AllowPostLayoutScope::new();
    let mut result = RecalcScrollableOverflowResult::default();
    for layout_result in box_.GetLayoutResults() {
        let layout_result = unsafe { &*layout_result.Get() };
        let fragment = layout_result.GetPhysicalFragment();
        let fragment = unsafe { &*(fragment as *const _ as *const PhysicalBoxFragment) };
        if fragment.HasItems() {
            let mut cursor = InlineCursor::from_fragment(fragment);
            while cursor.IsNotNull() {
                let item = cursor.Current().Item();
                let child = unsafe { &*item }.PostLayoutBoxFragment();
                if !child.is_null() && unsafe { &*(*child).GetLayoutObject() }.IsBox() {
                    let owner = unsafe { &*child }.MutableOwnerLayoutBox();
                    let child_result = unsafe { &mut *owner }.RecalcScrollableOverflow();
                    result.Unite(&child_result);
                }
                cursor.MoveToNext();
            }
        }
        RecalcFragmentChildren(&mut result, fragment);
    }
    result
}

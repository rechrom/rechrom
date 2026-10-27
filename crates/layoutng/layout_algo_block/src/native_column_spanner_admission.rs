use super::*;

// cpp: layoutng/internal/layout_box_geometry.cc:404-414
#[inline(never)]
pub(super) fn LayoutBoxIsValidColumnSpannerInTree(box_: &LayoutBox, style: &ComputedStyle) -> bool {
    box_.CheckIsNotDestroyed();
    if box_.Parent().is_null()
        || !box_.IsInsideMulticol()
        || !LayoutBoxIsSelfValidColumnSpanner(box_, style)
    {
        return false;
    }
    LayoutBoxDoesAncestryAllowColumnSpanner(box_)
}

// cpp: layoutng/internal/layout_box_geometry.cc:416-433
pub(super) fn LayoutBoxIsSelfValidColumnSpanner(box_: &LayoutBox, style: &ComputedStyle) -> bool {
    box_.CheckIsNotDestroyed();
    if style.GetColumnSpan() != EColumnSpan::kAll {
        return false;
    }
    if box_.ShouldBeHandledAsInline(style)
        || box_.ShouldBeHandledAsFloating(style)
        || box_.ToPositionedState(style.GetPosition()) == PositionedState::kIsOutOfFlowPositioned
    {
        return false;
    }
    true
}

// cpp: layoutng/internal/layout_box_geometry.cc:435-449
#[inline(never)]
pub(super) fn LayoutBoxDoesAncestryAllowColumnSpanner(box_: &LayoutBox) -> bool {
    box_.CheckIsNotDestroyed();
    debug_assert!(box_.IsInsideMulticol());
    let parent = box_.Parent();
    let mut ancestor = if parent.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe { &*parent }.EnclosingBox()
    };
    while !ancestor.is_null() {
        let ancestor_ref = unsafe { &*ancestor };
        if ancestor_ref.IsMulticolContainer() {
            return true;
        }
        if LayoutBoxShouldPreventColumnSpannerDescendants(ancestor_ref) {
            return false;
        }
        ancestor = ancestor_ref.ContainingBlock() as *mut LayoutBox;
    }
    false
}

// cpp: layoutng/internal/layout_box_geometry.cc:451-478
pub(super) fn LayoutBoxShouldPreventColumnSpannerDescendants(box_: &LayoutBox) -> bool {
    box_.CheckIsNotDestroyed();
    if LayoutBoxIsSelfValidColumnSpanner(box_, box_.StyleRef()) {
        return true;
    }
    let block_flow = DynamicTo::<LayoutBlockFlow>(box_ as *const LayoutBox as *const LayoutObject);
    if block_flow.is_null() {
        return true;
    }
    let block_flow = unsafe { &*block_flow };
    if block_flow.IsMonolithic()
        || block_flow.CreatesNewFormattingContext()
        || block_flow.CanContainFixedPositionObjects()
    {
        return true;
    }
    debug_assert_ne!(box_.StyleRef().GetColumnSpan(), EColumnSpan::kAll);
    false
}

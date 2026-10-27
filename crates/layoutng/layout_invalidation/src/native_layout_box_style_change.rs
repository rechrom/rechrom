use super::*;

// cpp: layoutng/internal/layout_box_lifecycle.cc:52-68
#[inline(never)]
pub(super) fn LayoutBoxStyleWillChange(
    box_: &mut LayoutBox,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    style_change_context: &mut StyleChangeContext,
) {
    box_.CheckIsNotDestroyed();
    if !old_style.is_null() {
        if diff.NeedsFullLayout()
            && !box_.Parent().is_null()
            && unsafe { &*old_style }.GetPosition() != new_style.GetPosition()
        {
            box_.MarkContainerChainForLayout();
        }
        style_change_context.did_prevent_spanner_descendants =
            box_.IsInsideMulticol() && box_.ShouldPreventColumnSpannerDescendants();
    }
    box_.model_object_base_mut()
        .StyleWillChange(diff, old_style, new_style, style_change_context);
}

// cpp: layoutng/internal/layout_box_lifecycle.cc:70-99
#[inline(never)]
pub(super) fn LayoutBoxStyleDidChange(
    box_: &mut LayoutBox,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    style_change_context: &StyleChangeContext,
) {
    box_.CheckIsNotDestroyed();
    box_.model_object_base_mut()
        .StyleDidChange(diff, old_style, new_style, style_change_context);
    // Chromium layout_box.cc:852-909: changing a grid item's placement or
    // order invalidates its container's persistent placement cache.
    if !old_style.is_null() {
        let old = unsafe { &*old_style };
        let grid_changed = old.GridColumnStart() != new_style.GridColumnStart()
            || old.GridColumnEnd() != new_style.GridColumnEnd()
            || old.GridRowStart() != new_style.GridRowStart()
            || old.GridRowEnd() != new_style.GridRowEnd()
            || old.Order() != new_style.Order()
            || old.HasOutOfFlowPosition() != new_style.HasOutOfFlowPosition();
        if grid_changed {
            let container = box_.ContainingBlock();
            if !container.is_null() && unsafe { &*container }.IsLayoutGrid() {
                if !old.HasOutOfFlowPosition() || !new_style.HasOutOfFlowPosition() {
                    unsafe { &mut *container }.SetGridPlacementDirty(true);
                }
                unsafe { &mut *container }.SetNeedsLayoutAndIntrinsicWidthsRecalc(
                    &raw const layout_invalidation_reason::kStyleChange,
                );
            }
        }
    }
    let axes = box_.ComputeOverflowClipAxes();
    box_.SetOverflowClipAxes(axes);

    if !old_style.is_null()
        && box_.IsInLayoutNGInlineFormattingContext()
        && box_.IsInline()
        && unsafe { &*old_style }.Direction() != new_style.Direction()
    {
        box_.SetNeedsCollectInlines();
    }
    if !old_style.is_null() {
        let should_prevent_now =
            box_.IsInsideMulticol() && box_.ShouldPreventColumnSpannerDescendants();
        if style_change_context.did_prevent_spanner_descendants != should_prevent_now {
            let factories = LayoutObjectFactoryScope::Objects();
            let invalidate = if factories.is_null() {
                None
            } else {
                unsafe { &*factories }.invalidate_column_spanners
            };
            let Some(invalidate) = invalidate else {
                std::panic::panic_any(UnsupportedLayout::new(
                    "multicol layout module is not installed",
                ));
            };
            invalidate(box_);
        }
    }
    if diff.transform_changed() && box_.TransformsChangeMayRequireLayout() {
        let reason = unsafe { &layout_invalidation_reason::kStyleChange as *const _ };
        box_.SetNeedsLayoutAndFullPaintInvalidation(reason);
    }
}

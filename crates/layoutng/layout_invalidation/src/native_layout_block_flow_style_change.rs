use super::*;

// cpp: layoutng/internal/layout_block_flow.cc:605-624
#[inline(never)]
pub(super) fn LayoutBlockFlowStyleDidChange(
    block: &mut LayoutBlockFlow,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    style_change_context: &StyleChangeContext,
) {
    block.CheckIsNotDestroyed();
    block
        .block_
        .StyleDidChange(diff, old_style, new_style, style_change_context);
    if diff.NeedsFullLayout() || old_style.is_null() {
        block.UpdateForMulticol();
    }
    if !old_style.is_null()
        && unsafe { &*old_style }.CanGeneratePseudoElement(PseudoId::kPseudoIdColumn)
            != new_style.CanGeneratePseudoElement(PseudoId::kPseudoIdColumn)
    {
        block.SetNeedsLayout(&raw const layout_invalidation_reason::kStyleChange);
    }
    let factories = LayoutObjectFactoryScope::Objects();
    if diff.needs_reshape() && !factories.is_null() {
        if let Some(update_inline_style) = unsafe { &*factories }.update_inline_style {
            update_inline_style(block, old_style, new_style);
        }
    }
}

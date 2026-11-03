use super::*;

// cpp: layoutng/internal/layout_block.cc:340-379
#[inline(never)]
pub(super) fn LayoutBlockStyleDidChange(
    block: &mut LayoutBlock,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    style_change_context: &StyleChangeContext,
) {
    block.CheckIsNotDestroyed();
    let update_svg_text = diff.transform_changed() && block.HasSVGTextDescendants();
    let factories = LayoutObjectFactoryScope::Objects();
    if update_svg_text
        && (factories.is_null()
            || unsafe { &*factories }.svg_block_text_scale.is_none()
            || unsafe { &*factories }.update_svg_block_style.is_none())
    {
        std::panic::panic_any(UnsupportedLayout::new("SVG layout module is not installed"));
    }
    let old_squared_scale = if update_svg_text {
        unsafe { &*factories }.svg_block_text_scale.unwrap()(block)
    } else {
        1.0
    };
    block
        .box_
        .StyleDidChange(diff, old_style, new_style, style_change_context);
    if !old_style.is_null() && !block.Parent().is_null() {
        if unsafe { &*old_style }.GetPosition() != new_style.GetPosition()
            && new_style.GetPosition() != EPosition::kStatic
        {
            let cb = block.ContainingBlock();
            if !cb.is_null() {
                unsafe { &mut *cb }.RemovePositionedObjects(block as *mut _ as *mut LayoutObject);
            }
        }
    }
    block.PropagateStyleToAnonymousChildren();
    if update_svg_text {
        unsafe { &*factories }.update_svg_block_style.unwrap()(
            block,
            old_squared_scale,
            old_style,
            new_style,
        );
    }
}

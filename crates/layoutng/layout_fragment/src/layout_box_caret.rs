//! Source: core/layout/layout_box.cc:3138-3183.
use super::*;

#[allow(non_snake_case)]
pub(super) fn LayoutBoxLocalCaretRectProvider(
    owner: &LayoutBox,
    caret_offset: i32,
    _shape: CaretShape,
) -> PhysicalRect {
    owner.CheckIsNotDestroyed();
    let font = unsafe { owner.StyleRef().GetFont().as_ref() };
    let primary = font.and_then(|font| unsafe { font.PrimaryFont().as_ref() });
    let height = primary.map_or(LayoutUnit::default(), |primary| {
        LayoutUnit::from_f64(f64::from(primary.GetFontMetrics().Height()))
    });
    let caret_size = LogicalSize::new(owner.BarCaretWidth(), height);
    let apply_border_padding = unsafe { owner.GetNode().as_ref() }.is_some_and(|node| {
        let editing = node.InputEditingFacts();
        !editing.ignores_content && !editing.is_html_table
    });
    let direction = owner.StyleRef().GetWritingDirection();
    let mut offset = LogicalOffset::default();
    let mut content_inline_size = owner.LogicalWidth();
    if apply_border_padding {
        let border_padding =
            (owner.BorderOutsets() + owner.PaddingOutsets()).ConvertToLogical(direction);
        offset.inline_offset = border_padding.inline_start;
        offset.block_offset = border_padding.block_start;
        content_inline_size -= border_padding.InlineSum();
    }
    if caret_offset != 0 {
        offset.inline_offset += content_inline_size - caret_size.inline_size;
    }
    WritingModeConverter::new(direction, owner.StitchedSize()).ToPhysicalRect(LogicalRect {
        offset,
        size: caret_size,
    })
}

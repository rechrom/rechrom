//! Source: core/layout/layout_block.cc:507-525.
use super::*;

#[allow(non_snake_case)]
pub(super) fn LayoutBlockLocalCaretRect(
    owner: &LayoutBlock,
    offset: i32,
    shape: CaretShape,
) -> PhysicalRect {
    owner.CheckIsNotDestroyed();
    if unsafe { owner.FirstChild().as_ref() }.is_some_and(|child| !child.IsPseudoElement())
        || owner.IsInlineBoxWrapperActuallyChild()
    {
        return owner.box_.LocalCaretRect(offset, shape);
    }
    let size = owner.StitchedSize();
    let inline_size = if owner.StyleRef().IsHorizontalWritingMode() {
        size.width
    } else {
        size.height
    };
    let rect = owner.LocalCaretRectForEmptyElement(inline_size, owner.TextIndentOffset(), shape);
    owner.CreateWritingModeConverter().ToPhysicalRect(rect)
}

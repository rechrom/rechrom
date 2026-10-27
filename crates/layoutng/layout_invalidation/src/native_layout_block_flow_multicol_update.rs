use super::*;

// cpp: layoutng/internal/layout_block_flow.h:97-99
// cpp: layoutng/internal/layout_block_flow.cc:650-707
#[inline(never)]
pub(super) fn LayoutBlockFlowUpdateForMulticol(block: &mut LayoutBlockFlow) {
    block.CheckIsNotDestroyed();
    let mut should_be_multicol = block.StyleRef().SpecifiesColumns() && block.AllowsColumns();
    if should_be_multicol && block.IsFieldset() {
        should_be_multicol = false;
    }
    if should_be_multicol {
        let element = DynamicTo::<Element>(block.GetNode());
        if !element.is_null()
            && unsafe { &*element }.IsFormControlElement()
            && unsafe { &*element }.GetElementType() != ElementType::kHTMLButtonElement
        {
            should_be_multicol = false;
        }
    }
    if should_be_multicol == block.IsMulticolContainer() {
        return;
    }

    block.SetIsMulticolContainer(should_be_multicol);

    let inside_multicol = should_be_multicol || block.IsInsideMulticol();
    let mut child = block.FirstChild();
    while !child.is_null() {
        unsafe { &mut *child }.SetIsInsideMulticolIncludingDescendants(inside_multicol);
        child = unsafe { &*child }.NextSibling();
    }

    if should_be_multicol {
        block.MakeChildrenNonInlineDefault();
    } else {
        block.MakeChildrenInlineIfPossible();
    }
}

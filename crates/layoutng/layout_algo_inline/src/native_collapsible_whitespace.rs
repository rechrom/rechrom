//! Native collapsible-whitespace predecessor lookup.
//!
//! Included by `layout_object_builder` at the reciprocal assembly boundary.
//! Keeping the sibling walk out of the generic tree facade gives inline text
//! preparation a package-qualified sampling frame without a per-sibling trace.

use super::*;

#[inline(never)]
pub(super) fn needs_collapsible_whitespace(tree: &LayoutObjectTree, parent: &LayoutObject) -> bool {
    let storage = unsafe { &*tree.storage_ };
    let mut previous = if storage.updating {
        storage
            .last_child
            .get(&(parent as *const _ as *mut _))
            .map_or(std::ptr::null_mut(), |node| {
                unsafe { &**node }.GetLayoutObject()
            })
    } else {
        parent.SlowLastChild()
    };
    while !previous.is_null() {
        let sibling = unsafe { &*previous };
        if sibling.IsFloatingOrOutOfFlowPositioned() {
            previous = sibling.PreviousSibling();
            continue;
        }
        if sibling.IsText() {
            let text = unsafe { &*DynamicTo::<LayoutText>(previous) }.TransformedText();
            if text.empty() {
                return false;
            }
            let last = text.Span16().expect("nonempty text")[(text.length() - 1) as usize];
            return last != b' ' as u16
                && last != b'\t' as u16
                && last != b'\r' as u16
                && last != b'\n' as u16
                && last != b'\x0c' as u16;
        }
        return sibling.IsInline() && !sibling.IsBR();
    }
    parent.IsLayoutInline()
}

// C++: layoutng_inline/text_combine_tree.h/.cc.
#![allow(non_snake_case)]

use foundation::{DynamicTo, To};
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::layout_text_combine::LayoutTextCombine;

// cpp: layoutng_inline/text_combine_tree.h:5-8
// cpp: layoutng_inline/text_combine_tree.cc:10-46
pub fn InsertInlineTextChild(
    parent: &mut LayoutObject,
    child: &mut LayoutObject,
    before_child: *mut LayoutObject,
) -> bool {
    let children = parent.VirtualChildren();
    debug_assert!(!children.is_null());
    if parent.IsLayoutTextCombine() {
        debug_assert!(LayoutTextCombine::ShouldBeParentOf(child));
        child.SetStyle(parent.StyleRef() as *const _);
        unsafe { &mut *children }.InsertChildNodeDefault(parent, child, before_child);
        return true;
    }
    if parent.IsHorizontalTypographicMode() || !LayoutTextCombine::ShouldBeParentOf(child) {
        return false;
    }

    if !before_child.is_null() {
        if unsafe { &*before_child }.IsLayoutTextCombine() {
            let first = unsafe { &*before_child }.SlowFirstChild();
            unsafe { &mut *before_child }.AddChild(child, first);
        } else {
            let previous =
                DynamicTo::<LayoutTextCombine>(unsafe { &*before_child }.PreviousSibling());
            if !previous.is_null() {
                unsafe { &mut *previous }.AddChildDefault(child);
            } else {
                let anonymous = LayoutTextCombine::CreateAnonymous(To::<LayoutText>(child));
                unsafe { &mut *children }.InsertChildNodeDefault(
                    parent,
                    anonymous as *mut LayoutObject,
                    before_child,
                );
            }
        }
    } else {
        let last = DynamicTo::<LayoutTextCombine>(parent.SlowLastChild());
        if !last.is_null() {
            unsafe { &mut *last }.AddChildDefault(child);
        } else {
            let anonymous = LayoutTextCombine::CreateAnonymous(To::<LayoutText>(child));
            unsafe { &mut *children }
                .AppendChildNodeDefault(parent, anonymous as *mut LayoutObject);
        }
    }
    true
}

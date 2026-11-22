// C++: layoutng_inline/layout_inline_data.cc.
#![allow(non_snake_case)]

use foundation::Visitor;
use layout_core::internal::layout_inline::LayoutInline;
use layout_core::internal::layout_node_metadata::Element;

// cpp: layoutng_inline/layout_inline_data.cc:14-16
pub fn LayoutInlineNew(element: *mut Element) -> LayoutInline {
    let mut result = LayoutInline::new_base_for_inline(element);
    result.SetChildrenInline(true);
    result
}

// cpp: layoutng_inline/layout_inline_data.cc:18-21
pub fn LayoutInlineTrace(this: &LayoutInline, visitor: &mut Visitor<'_>) {
    this.ChildrenForInline().Trace(visitor);
    this.ModelObjectForInline().Trace(visitor);
}

// cpp: layoutng_inline/layout_inline_data.cc:23-28
pub fn LayoutInlineClearFirstInlineFragmentItemIndex(this: &mut LayoutInline) {
    this.CheckIsNotDestroyed();
    assert!(this.IsInLayoutNGInlineFormattingContext());
    this.SetFirstFragmentItemIndexForInline(0);
}

// cpp: layoutng_inline/layout_inline_data.cc:30-35
pub fn LayoutInlineSetFirstInlineFragmentItemIndex(
    this: &mut LayoutInline,
    index: usize,
) {
    this.CheckIsNotDestroyed();
    assert!(this.IsInLayoutNGInlineFormattingContext());
    debug_assert_ne!(index, 0);
    this.SetFirstFragmentItemIndexForInline(index);
}

// cpp: layoutng_inline/layout_inline_data.cc:37-42
pub fn LayoutInlineInLayoutNGInlineFormattingContextWillChange(
    this: &mut LayoutInline,
    _value: bool,
) {
    this.CheckIsNotDestroyed();
    if this.IsInLayoutNGInlineFormattingContext() {
        this.ClearFirstInlineFragmentItemIndex();
    }
}

//! Exact native LayoutText fact consumed by ApplyStyleCommand.
use super::*;
impl LayoutNode<'_> {
    /// apply_style_command.cc:1349-1350. Borrow this resident LayoutText;
    /// no DOM text or copied text-facts substitute participates in the query.
    pub fn EditingIsAllCollapsibleWhitespaceText(&self) -> bool {
        self.object.IsText()
            && unsafe {
                foundation::To::<crate::internal::layout_text::LayoutText>(self.object).as_ref()
            }
            .is_some_and(|text| text.IsAllCollapsibleWhitespace())
    }
}

unsafe extern "Rust" {
    fn LayoutEditingListItemValue(object: *const LayoutObject) -> Option<i32>;
}
impl LayoutNode<'_> {
    pub fn EditingIsListItem(&self) -> bool {
        self.object.IsLayoutListItem()
    }
    pub fn EditingListItemValue(&self) -> Option<i32> {
        unsafe { LayoutEditingListItemValue(self.object) }
    }
    pub fn EditingInlineAlwaysCreateLineBoxes(&self) -> Option<bool> {
        unsafe {
            foundation::DynamicTo::<crate::internal::layout_inline::LayoutInline>(self.object)
                .as_ref()
        }
        .map(|inline| inline.AlwaysCreateLineBoxes())
    }
}

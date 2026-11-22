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

impl LayoutNode<'_> {
    pub fn EditingIsListItem(&self) -> bool {
        self.object.IsLayoutListItem()
    }
    /// break_blockquote_command.cc:51-62. LayoutListItem and its lazily
    /// resolved ordinal belong to the list package (registry slot
    /// `list_support.list_item_value`); a LayoutListItem exists only when
    /// that package is installed.
    pub fn EditingListItemValue(&self) -> Option<i32> {
        if !self.object.IsLayoutListItem() {
            return None;
        }
        let value = crate::internal::layout_pass_scope::LayoutPassScope::ResidentListSupport()
            .list_item_value
            .unwrap_or_else(|| {
                std::panic::panic_any(foundation::UnsupportedLayout::new(
                    "list layout module is not installed",
                ))
            });
        value(self.object)
    }
    pub fn EditingInlineAlwaysCreateLineBoxes(&self) -> Option<bool> {
        unsafe {
            foundation::DynamicTo::<crate::internal::layout_inline::LayoutInline>(self.object)
                .as_ref()
        }
        .map(|inline| inline.AlwaysCreateLineBoxes())
    }
}

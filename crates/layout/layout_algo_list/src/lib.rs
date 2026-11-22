// Translated Chromium identifiers keep their C++ spelling.
#![allow(non_snake_case, non_camel_case_types)]

pub mod registration;
pub mod layout_inline_list_item;
pub mod layout_inside_list_marker;
pub mod layout_list_item;
pub mod layout_outside_list_marker;
pub mod list_item_ordinal;
pub mod list_marker;

/// break_blockquote_command.cc:51-62. This package owns the actual list-item
/// class and its lazily resolved ordinal; base editing borrows only its scalar
/// through the `list_support.list_item_value` registry slot.
pub fn LayoutEditingListItemValue(
    object: *const layout_base::internal::layout_object::LayoutObject,
) -> Option<i32> {
    let object = unsafe { object.as_ref() }?;
    unsafe { foundation::DynamicTo::<layout_list_item::LayoutListItem>(object).as_ref() }
        .map(|item| item.Value())
}

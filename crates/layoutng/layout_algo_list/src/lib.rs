#![allow(non_snake_case)]

pub mod assembly;
pub mod layout_inline_list_item;
pub mod layout_inside_list_marker;
pub mod layout_list_item;
pub mod layout_outside_list_marker;
pub mod list_item_ordinal;
pub mod list_marker;

/// break_blockquote_command.cc:51-62. This package owns the actual list-item
/// class and its lazily resolved ordinal; assembly borrows only its scalar.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutEditingListItemValue(
    object: *const layoutng_assembly::internal::layout_object::LayoutObject,
) -> Option<i32> {
    let object = unsafe { object.as_ref() }?;
    unsafe { foundation::DynamicTo::<layout_list_item::LayoutListItem>(object).as_ref() }
        .map(|item| item.Value())
}

// C++: layoutng_inline/inline_items_data_algorithm.cc.
// The data structs belong to //src/layoutng_fragment_tree and //src/layoutng;
// this package supplies their out-of-line method bodies at the assembly edge.
#![allow(non_snake_case)]

use foundation::TextOffsetMap;
use layoutng::internal::inline_item::InlineItemType;
use layoutng::internal::inline_node_data::InlineNodeData;
use layoutng::internal::layout_input::TextDirection;
use layoutng_fragment_tree::inline_items_data::{
    InlineItemsData, InlineItemsDataWithOffsetMap, OpenTagItems,
};

// cpp: layoutng_inline/inline_items_data_algorithm.cc:10-13
#[no_mangle]
pub extern "Rust" fn InlineNodeDataDisableBidi(data: &mut InlineNodeData) {
    data.is_bidi_enabled_ = false;
    data.SetBaseDirection(TextDirection::kLtr);
}

// cpp: layoutng_inline/inline_items_data_algorithm.cc:15-28
#[no_mangle]
pub extern "Rust" fn InlineItemsDataGetOpenTagItems(
    data: &InlineItemsData,
    start_index: u32,
    size: u32,
    open_items: &mut OpenTagItems,
) {
    assert!(size <= data.items.size());
    let end = start_index
        .checked_add(size)
        .expect("inline item range overflow");
    assert!(end <= data.items.size());
    for item_ptr in &data.items[start_index as usize..end as usize] {
        let item = unsafe { &*item_ptr.Get() };
        if item.Type() == InlineItemType::kOpenTag {
            open_items.push_back(*item_ptr);
        } else if item.Type() == InlineItemType::kCloseTag {
            open_items.pop();
        }
    }
}

// cpp: layoutng_inline/inline_items_data_algorithm.cc:30-36
#[cfg(debug_assertions)]
#[no_mangle]
pub extern "Rust" fn InlineItemsDataCheckConsistency(data: &InlineItemsData) {
    for item in &data.items {
        unsafe { &*item.Get() }.CheckTextType(&data.text_content);
    }
}

// cpp: layoutng_inline/inline_items_data_algorithm.cc:38-45
#[no_mangle]
pub extern "Rust" fn InlineItemsDataOffsetMap(data: &InlineItemsData) -> &Option<TextOffsetMap> {
    if data.IsWithOffsetMap() {
        let with_offset =
            unsafe { &*(data as *const InlineItemsData as *const InlineItemsDataWithOffsetMap) };
        return &with_offset.offset_map;
    }
    static EMPTY: Option<TextOffsetMap> = None;
    &EMPTY
}

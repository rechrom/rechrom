use layoutng_fragment_tree::fragment_items_builder::FragmentItemWithOffset;

// cpp: layoutng/internal/fragment_item_with_offset_fwd.h:8-10
// HeapVector's inline capacity is an allocation detail; callers can reserve it.
pub type FragmentItemWithOffsetList = Vec<FragmentItemWithOffset>;

pub fn fragment_item_with_offset_list() -> FragmentItemWithOffsetList {
    Vec::with_capacity(128)
}

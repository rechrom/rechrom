// The ordinal state is owned by LayoutListItem. Its DOM-order calculation is
// still pending translation from list_item_ordinal.h.
// cpp: layoutng_list/list_item_ordinal.h:112-119
#[derive(Default)]
pub struct ListItemOrdinal {
    pub value: i32,
    pub explicit_value: Option<i32>,
    pub needs_update: bool,
}

impl ListItemOrdinal {
    // cpp: layoutng_list/list_item_ordinal_core.cc:9-9
    pub fn new() -> Self {
        Self {
            needs_update: true,
            ..Self::default()
        }
    }

    // cpp: layoutng_list/list_item_ordinal.h:38-39
    // This declaration has no definition in the supplied source tree.
    pub fn Value(&self, node: &layoutng_assembly::internal::layout_node_metadata::Node) -> i32 {
        unsafe { ListItemOrdinalValue(self, node) }
    }

    // cpp: layoutng_list/list_item_ordinal.h:62-63
    pub fn ItemInsertedOrRemoved(
        item: *const layoutng_assembly::internal::layout_object::LayoutObject,
    ) {
        unsafe { ListItemOrdinalItemInsertedOrRemoved(item) }
    }
}

unsafe extern "Rust" {
    fn ListItemOrdinalValue(
        ordinal: &ListItemOrdinal,
        node: &layoutng_assembly::internal::layout_node_metadata::Node,
    ) -> i32;
    fn ListItemOrdinalItemInsertedOrRemoved(
        item: *const layoutng_assembly::internal::layout_object::LayoutObject,
    );
}

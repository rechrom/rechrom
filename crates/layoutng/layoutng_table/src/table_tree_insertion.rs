#![allow(non_snake_case)]
use crate::layout_table::LayoutTable;
use foundation::EDisplay;
use layoutng_assembly::internal::layout_object::LayoutObject;
// cpp: layoutng_table/table_tree_insertion.cc:10-26
fn RequiresAnonymousTableWrappers(parent: &LayoutObject, child: &LayoutObject) -> bool {
    if child.IsLayoutTableCol() {
        let column_in_column_group =
            child.StyleRef().Display() == EDisplay::kTableColumn && parent.IsLayoutTableCol();
        return !parent.IsTable() && !column_in_column_group;
    }
    if child.IsTableCaption() || child.IsTableSection() {
        return !parent.IsTable();
    }
    if child.IsTableRow() {
        return !parent.IsTableSection();
    }
    if child.IsTableCell() {
        return !parent.IsTableRow();
    }
    false
}
// cpp: layoutng_table/table_tree_insertion.h:6-8
// cpp: layoutng_table/table_tree_insertion.cc:30-49
pub fn InsertTableChild(
    parent: &mut LayoutObject,
    child: &mut LayoutObject,
    before: *mut LayoutObject,
) {
    let children = parent.VirtualChildren();
    debug_assert!(!children.is_null());
    if !RequiresAnonymousTableWrappers(parent, child) {
        unsafe { &mut *children }.InsertChildNode(parent, child, before, true);
        return;
    }
    let after = if before.is_null() {
        unsafe { &*children }.LastChild()
    } else {
        unsafe { &*before }.PreviousSibling()
    };
    let table =
        if !after.is_null() && unsafe { &*after }.IsAnonymous() && unsafe { &*after }.IsTable() {
            after
        } else {
            let table = LayoutTable::CreateAnonymousWithParent(parent).cast::<LayoutObject>();
            unsafe { &mut *children }.InsertChildNode(parent, table, before, true);
            table
        };
    unsafe { &mut *table }.AddChild(child, std::ptr::null_mut());
}

//! Native LayoutTable/Section/Row/Cell facts read by GridFocusgroupStructureInfo.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/page/grid_focusgroup_structure_info.cc:31-486.
#![allow(non_snake_case)]
use crate::{
    layout_table::LayoutTable, layout_table_cell::LayoutTableCell,
    layout_table_row::LayoutTableRow, layout_table_section::LayoutTableSection,
};
use foundation::{table_focusgroup::TableFocusgroupFacts as Facts, To};
use layoutng_assembly::internal::layout_object::LayoutObject;
fn Object<T>(pointer: *mut T) -> Option<*const LayoutObject> {
    (!pointer.is_null()).then_some(pointer.cast::<LayoutObject>() as *const LayoutObject)
}
pub(crate) fn Read(object: &LayoutObject) -> Option<Facts<*const LayoutObject>> {
    let pointer = object as *const LayoutObject;
    if object.IsTable() {
        let table = unsafe { &*To::<LayoutTable>(pointer) };
        Some(Facts::Table {
            columns: table.EffectiveColumnCount(),
            first_section: Object(table.FirstSection()),
            last_section: Object(table.LastSection()),
        })
    } else if object.IsTableSection() {
        let section = unsafe { &*To::<LayoutTableSection>(pointer) };
        let table = unsafe { section.Table().as_ref() }?;
        Some(Facts::Section {
            previous: Object(table.PreviousSection(section)),
            next: Object(table.NextSection(section)),
            first_row: Object(section.FirstRow()),
            last_row: Object(section.LastRow()),
        })
    } else if object.IsTableRow() {
        let row = unsafe { &*To::<LayoutTableRow>(pointer) };
        Some(Facts::Row {
            section: Object(row.Section())?,
            previous: Object(row.PreviousRow()),
            next: Object(row.NextRow()),
            first_cell: Object(row.FirstCell()),
            last_cell: Object(row.LastCell()),
        })
    } else if object.IsTableCell() {
        let cell = unsafe { &*To::<LayoutTableCell>(pointer) };
        Some(Facts::Cell {
            row: Object(cell.Row())?,
            previous: Object(cell.PreviousCell()),
            column: cell.AbsoluteColumnIndex(),
            col_span: cell.ColSpan(),
            row_span: cell.ResolvedRowSpan(),
        })
    } else {
        None
    }
}

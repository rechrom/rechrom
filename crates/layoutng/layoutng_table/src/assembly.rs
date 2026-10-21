#![allow(non_snake_case)]
use crate::{
    layout_table::LayoutTable, layout_table_caption::LayoutTableCaption,
    layout_table_cell::LayoutTableCell, layout_table_column::LayoutTableColumn,
    layout_table_row::LayoutTableRow, layout_table_section::LayoutTableSection,
};
use foundation::{EDisplay, MakeGarbageCollected, To};
use layoutng_assembly::internal::{
    layout_node_metadata::{Element, Node},
    layout_object::LayoutObject,
    layout_object_factory_set::LayoutObjectFactorySet,
};
use layoutng_assembly::{
    internal::{algorithm_entry::NativeAlgorithmEntry, layout_algorithm_set::TableLayoutSupport},
    layout_assembly::LayoutAssembly,
};
use layoutng_style::style::computed_style::ComputedStyle;
// cpp: layoutng_table/assembly.cc:22-44
fn CreateTableObject(node: &mut Node, style: &ComputedStyle) -> *mut LayoutObject {
    let element = To::<Element>(node);
    match style.Display() {
        EDisplay::kTable | EDisplay::kInlineTable => {
            MakeGarbageCollected(LayoutTable::new(element)).cast()
        }
        EDisplay::kTableRowGroup | EDisplay::kTableHeaderGroup | EDisplay::kTableFooterGroup => {
            MakeGarbageCollected(LayoutTableSection::new(element)).cast()
        }
        EDisplay::kTableRow => MakeGarbageCollected(LayoutTableRow::new(element)).cast(),
        EDisplay::kTableColumnGroup | EDisplay::kTableColumn => {
            MakeGarbageCollected(LayoutTableColumn::new(element)).cast()
        }
        EDisplay::kTableCell => MakeGarbageCollected(LayoutTableCell::new(element)).cast(),
        EDisplay::kTableCaption => MakeGarbageCollected(LayoutTableCaption::new(element)).cast(),
        other => panic!("CreateTableObject is unreachable for {other:?}"),
    }
}
// Object-only installation is restricted to the object-construction oracle.
// Production installs the source algorithms and objects together.
// cpp: layoutng_table/assembly.cc:62-63
pub(crate) fn InstallTableObjects(objects: &mut LayoutObjectFactorySet) {
    objects.table = Some(CreateTableObject);
    objects.insert_table_child = Some(crate::table_tree_insertion::InsertTableChild);
    objects.table_virtuals = Some(crate::table_object_virtuals::Virtuals());
}

// cpp: layoutng_table/assembly.h:3-3
// cpp: layoutng_table/assembly.cc:61-72
pub fn InstallTableAlgorithm(assembly: &mut LayoutAssembly) {
    InstallTableObjects(&mut assembly.objects);
    assembly.algorithms.table =
        NativeAlgorithmEntry::<crate::table_layout_algorithm::TableLayoutAlgorithm>();
    assembly.algorithms.table_row =
        NativeAlgorithmEntry::<crate::table_row_layout_algorithm::TableRowLayoutAlgorithm>();
    assembly.algorithms.table_section = NativeAlgorithmEntry::<
        crate::table_section_layout_algorithm::TableSectionLayoutAlgorithm,
    >();
    assembly.algorithms.table_support = TableLayoutSupport {
        inline_size: Some(crate::table_node::TableNodeComputeTableInlineSizeFromTable),
        caption_block_size: Some(crate::table_node::TableNodeComputeCaptionBlockSizeFromTable),
        borders: Some(crate::table_node::TableNodeGetTableBordersStrutFromTable),
        finalize_cell: Some(FinalizeCell),
    };
}
// Rust carries the C++ builder pointer through the callback boundary unchanged.
// cpp: layoutng_table/assembly.cc:71-71
fn FinalizeCell(
    size: foundation::LayoutUnit,
    builder: *mut layoutng_assembly::box_fragment_builder::BoxFragmentBuilder,
) {
    crate::table_row_measurement::FinalizeTableCellLayout(size, unsafe { &mut *builder });
}

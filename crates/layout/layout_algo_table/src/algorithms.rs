//! Out-of-line members of base-declared types whose Chromium bodies live in
//! core/layout/table/ (R7a, crates/layout/PACKAGE_MAP.md §7). Base declares the
//! types because the fragment tree (collapsed TableBorders), LayoutInputNode
//! and the TableLayoutSupport slots name them; Rust cannot add inherent
//! methods to them from this crate, so each family is an extension trait.
//! Every table module imports this prelude, so the sources keep Chromium's
//! member-call syntax and the calls stay direct, statically dispatched calls
//! inside this crate. Only table code calls these members; base reaches the
//! few table behaviours it needs through registry slots
//! (`TableLayoutSupport`, `TableObjectVirtuals`).
#![allow(non_snake_case)]

use std::sync::Arc;

use foundation::{LayoutUnit, WritingDirectionMode};
use layout_assembly::internal::{
    block_node::BlockNode,
    constraint_space::ConstraintSpace,
    layout_input_node::LayoutInputNode,
    table_borders::{EdgeSide, EdgeSource, TableBorders},
    table_layout_algorithm_types::{
        CellInlineConstraint, Column, Columns, Section, TableGroupedChildren,
        TableGroupedChildrenIterator, TableTypes,
    },
    table_node::TableNode,
};
use layout_geometry::geometry::box_strut::BoxStrut;
use layout_style::style::computed_style::ComputedStyle;

use crate::table_borders as borders;
use crate::table_layout_algorithm_types as types;

// cpp: layoutng/internal/table_node.h:20-41
// cpp: layoutng_table/table_node.cc:13-81
pub trait TableNodeAlgorithms {
    fn GetTableBordersStrut(&self) -> &BoxStrut;
    fn GetTableBorders(&self) -> *const TableBorders;
    fn ComputeCaptionBlockSize(&self, space: &ConstraintSpace) -> LayoutUnit;
    fn GetColumnConstraints(
        &self,
        grouped_children: &TableGroupedChildren,
        border_padding: &BoxStrut,
    ) -> Option<Arc<Columns>>;
    fn ComputeTableInlineSize(&self, space: &ConstraintSpace, border_padding: &BoxStrut)
        -> LayoutUnit;
    fn AllowColumnPercentages(&self, is_layout_pass: bool) -> bool;
}

impl TableNodeAlgorithms for TableNode {
    fn GetTableBordersStrut(&self) -> &BoxStrut {
        let borders = crate::table_node::TableNodeGetTableBordersStrutFromTable(self);
        debug_assert!(!borders.is_null());
        unsafe { &*borders }
    }

    fn GetTableBorders(&self) -> *const TableBorders {
        crate::table_node::TableNodeGetTableBordersFromTable(self)
    }

    fn ComputeCaptionBlockSize(&self, space: &ConstraintSpace) -> LayoutUnit {
        crate::table_node::TableNodeComputeCaptionBlockSizeFromTable(self, space)
    }

    fn GetColumnConstraints(
        &self,
        grouped_children: &TableGroupedChildren,
        border_padding: &BoxStrut,
    ) -> Option<Arc<Columns>> {
        crate::table_node::TableNodeGetColumnConstraintsFromTable(
            self,
            grouped_children,
            border_padding,
        )
    }

    fn ComputeTableInlineSize(
        &self,
        space: &ConstraintSpace,
        border_padding: &BoxStrut,
    ) -> LayoutUnit {
        crate::table_node::TableNodeComputeTableInlineSizeFromTable(self, space, border_padding)
    }

    fn AllowColumnPercentages(&self, is_layout_pass: bool) -> bool {
        crate::table_node::TableNodeAllowColumnPercentagesFromTable(self, is_layout_pass)
    }
}

// cpp: layoutng/internal/layout_input_node.h:117,123-127
// Bodies: layoutng_table/layout_table_section.cc, layout_table_column.cc,
// layout_table_cell.cc (they cast to the table LayoutObjects).
pub trait LayoutInputNodeTableAlgorithms {
    fn IsEmptyTableSection(&self) -> bool;
    fn TableColumnSpan(&self) -> u32;
    fn TableCellColspan(&self) -> u32;
    fn TableCellRowspan(&self) -> u32;
}

impl LayoutInputNodeTableAlgorithms for LayoutInputNode {
    fn IsEmptyTableSection(&self) -> bool {
        crate::layout_table_section::LayoutInputNodeIsEmptyTableSectionFromTable(self)
    }

    fn TableColumnSpan(&self) -> u32 {
        crate::layout_table_column::LayoutInputNodeTableColumnSpanFromTable(self)
    }

    fn TableCellColspan(&self) -> u32 {
        crate::layout_table_cell::LayoutInputNodeTableCellColspanFromTable(self)
    }

    fn TableCellRowspan(&self) -> u32 {
        crate::layout_table_cell::LayoutInputNodeTableCellRowspanFromTable(self)
    }
}

// cpp: layoutng/internal/table_borders.h:62-71,175-198,261-287
// cpp: layoutng_table/table_borders.cc
pub trait TableBordersAlgorithms: Sized {
    fn ComputeTableBorders(table: &BlockNode) -> *const Self;
    fn new(table_border: &BoxStrut, is_collapsed: bool) -> Self;
    #[cfg(debug_assertions)]
    fn DumpEdges(&mut self) -> foundation::String;
    #[cfg(debug_assertions)]
    fn Equals(&self, other: &Self) -> bool;
    fn CellBorder(
        &self,
        cell: &BlockNode,
        row: u32,
        column: u32,
        section: u32,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut;
    fn CellPaddingForMeasure(
        &self,
        cell_style: &ComputedStyle,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut;
    fn UpdateTableBorder(&mut self, table_row_count: u32, table_column_count: u32);
    fn MergeBorders(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        colspan: u32,
        source_style: &ComputedStyle,
        source: EdgeSource,
        box_order: u32,
        table_writing_direction: WritingDirectionMode,
        section_index: u32,
    );
    // Rust has no default arguments; this preserves the source's kNotFound
    // section-index default without changing the MergeBorders call order.
    fn MergeBordersWithoutSection(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        colspan: u32,
        source_style: &ComputedStyle,
        source: EdgeSource,
        box_order: u32,
        table_writing_direction: WritingDirectionMode,
    ) {
        self.MergeBorders(
            start_row,
            start_column,
            rowspan,
            colspan,
            source_style,
            source,
            box_order,
            table_writing_direction,
            u32::MAX,
        );
    }
    fn GetCellBorders(&self, row: u32, column: u32, rowspan: u32, colspan: u32) -> BoxStrut;
    fn EnsureCellColumnFits(&mut self, cell_column: u32);
    fn EnsureCellRowFits(&mut self, cell_row: u32);
    fn MergeRowAxisBorder(
        &mut self,
        start_row: u32,
        start_column: u32,
        colspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    );
    fn MergeColumnAxisBorder(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    );
    fn MarkInnerBordersAsDoNotFill(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        colspan: u32,
    );
}

impl TableBordersAlgorithms for TableBorders {
    fn ComputeTableBorders(table: &BlockNode) -> *const Self {
        borders::TableBordersComputeFromTable(table)
    }

    fn new(table_border: &BoxStrut, is_collapsed: bool) -> Self {
        borders::TableBordersConstructFromTable(table_border, is_collapsed)
    }

    #[cfg(debug_assertions)]
    fn DumpEdges(&mut self) -> foundation::String {
        borders::TableBordersDumpEdgesFromTable(self)
    }

    #[cfg(debug_assertions)]
    fn Equals(&self, other: &Self) -> bool {
        borders::TableBordersEqualFromTable(self, other)
    }

    fn CellBorder(
        &self,
        cell: &BlockNode,
        row: u32,
        column: u32,
        section: u32,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut {
        borders::TableBordersCellBorderFromTable(
            self,
            cell,
            row,
            column,
            section,
            table_writing_direction,
        )
    }

    fn CellPaddingForMeasure(
        &self,
        cell_style: &ComputedStyle,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut {
        borders::TableBordersCellPaddingForMeasureFromTable(
            self,
            cell_style,
            table_writing_direction,
        )
    }

    fn UpdateTableBorder(&mut self, table_row_count: u32, table_column_count: u32) {
        borders::TableBordersUpdateTableBorderFromTable(self, table_row_count, table_column_count)
    }

    fn MergeBorders(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        colspan: u32,
        source_style: &ComputedStyle,
        source: EdgeSource,
        box_order: u32,
        table_writing_direction: WritingDirectionMode,
        section_index: u32,
    ) {
        borders::TableBordersMergeBordersFromTable(
            self,
            start_row,
            start_column,
            rowspan,
            colspan,
            source_style,
            source,
            box_order,
            table_writing_direction,
            section_index,
        )
    }

    fn GetCellBorders(&self, row: u32, column: u32, rowspan: u32, colspan: u32) -> BoxStrut {
        borders::TableBordersGetCellBordersFromTable(self, row, column, rowspan, colspan)
    }

    fn EnsureCellColumnFits(&mut self, cell_column: u32) {
        borders::TableBordersEnsureCellColumnFitsFromTable(self, cell_column)
    }

    fn EnsureCellRowFits(&mut self, cell_row: u32) {
        borders::TableBordersEnsureCellRowFitsFromTable(self, cell_row)
    }

    fn MergeRowAxisBorder(
        &mut self,
        start_row: u32,
        start_column: u32,
        colspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    ) {
        borders::TableBordersMergeRowAxisBorderFromTable(
            self,
            start_row,
            start_column,
            colspan,
            source_style,
            box_order,
            side,
        )
    }

    fn MergeColumnAxisBorder(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    ) {
        borders::TableBordersMergeColumnAxisBorderFromTable(
            self,
            start_row,
            start_column,
            rowspan,
            source_style,
            box_order,
            side,
        )
    }

    fn MarkInnerBordersAsDoNotFill(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        colspan: u32,
    ) {
        borders::TableBordersMarkInnerBordersAsDoNotFillFromTable(
            self,
            start_row,
            start_column,
            rowspan,
            colspan,
        )
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:206-221
// cpp: layoutng_table/table_layout_algorithm_types.cc:86-236
pub trait TableTypesAlgorithms {
    fn CreateColumn(
        style: &ComputedStyle,
        default_inline_size: Option<LayoutUnit>,
        is_table_fixed: bool,
    ) -> Column;
    fn CreateCellInlineConstraint(
        node: &BlockNode,
        table_writing_direction: WritingDirectionMode,
        is_fixed_layout: bool,
        cell_border: &BoxStrut,
        cell_padding: &BoxStrut,
    ) -> CellInlineConstraint;
    fn CreateSection(
        node: &LayoutInputNode,
        start_row: u32,
        row_count: u32,
        block_size: LayoutUnit,
        treat_as_tbody: bool,
    ) -> Section;
}

impl TableTypesAlgorithms for TableTypes {
    fn CreateColumn(
        style: &ComputedStyle,
        default_inline_size: Option<LayoutUnit>,
        is_table_fixed: bool,
    ) -> Column {
        types::TableTypesCreateColumnFromTable(style, default_inline_size, is_table_fixed)
    }

    fn CreateCellInlineConstraint(
        node: &BlockNode,
        table_writing_direction: WritingDirectionMode,
        is_fixed_layout: bool,
        cell_border: &BoxStrut,
        cell_padding: &BoxStrut,
    ) -> CellInlineConstraint {
        types::TableTypesCreateCellInlineConstraintFromTable(
            node,
            table_writing_direction,
            is_fixed_layout,
            cell_border,
            cell_padding,
        )
    }

    fn CreateSection(
        node: &LayoutInputNode,
        start_row: u32,
        row_count: u32,
        block_size: LayoutUnit,
        treat_as_tbody: bool,
    ) -> Section {
        types::TableTypesCreateSectionFromTable(
            node,
            start_row,
            row_count,
            block_size,
            treat_as_tbody,
        )
    }
}

// cpp: layoutng_table/table_layout_algorithm_types.cc:238-262
pub trait CellInlineConstraintAlgorithms {
    fn Encompass(&mut self, other: &CellInlineConstraint);
}

impl CellInlineConstraintAlgorithms for CellInlineConstraint {
    fn Encompass(&mut self, other: &CellInlineConstraint) {
        types::TableCellInlineConstraintEncompassFromTable(self, other)
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:103-103
// cpp: layoutng_table/table_layout_algorithm_types.cc:264-306
pub trait ColumnAlgorithms {
    fn Encompass(&mut self, cell: &Option<CellInlineConstraint>);
}

impl ColumnAlgorithms for Column {
    fn Encompass(&mut self, cell: &Option<CellInlineConstraint>) {
        types::TableColumnEncompassFromTable(self, cell)
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:240-267
// cpp: layoutng_table/table_layout_algorithm_types.cc:308-343
pub trait TableGroupedChildrenAlgorithms {
    fn new(table: &BlockNode) -> Self;
    fn begin(&self) -> TableGroupedChildrenIterator;
    fn end(&self) -> TableGroupedChildrenIterator;
}

impl TableGroupedChildrenAlgorithms for TableGroupedChildren {
    fn new(table: &BlockNode) -> Self {
        types::TableGroupedChildrenConstructFromTable(table)
    }

    fn begin(&self) -> TableGroupedChildrenIterator {
        types::TableGroupedChildrenIteratorConstructFromTable(self, false)
    }

    fn end(&self) -> TableGroupedChildrenIterator {
        types::TableGroupedChildrenIteratorConstructFromTable(self, true)
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:271-298
// cpp: layoutng_table/table_layout_algorithm_types.cc:361-437
pub trait TableGroupedChildrenIteratorAlgorithms {
    fn new(grouped_children: &TableGroupedChildren, is_end: bool) -> Self;
    fn Increment(&mut self) -> &mut Self;
    fn Decrement(&mut self) -> &mut Self;
    fn Dereference(&self) -> BlockNode;
    fn Equals(&self, rhs: &Self) -> bool;
}

impl TableGroupedChildrenIteratorAlgorithms for TableGroupedChildrenIterator {
    fn new(grouped_children: &TableGroupedChildren, is_end: bool) -> Self {
        types::TableGroupedChildrenIteratorConstructFromTable(grouped_children, is_end)
    }

    fn Increment(&mut self) -> &mut Self {
        types::TableGroupedChildrenIteratorIncrementFromTable(self);
        self
    }

    fn Decrement(&mut self) -> &mut Self {
        types::TableGroupedChildrenIteratorDecrementFromTable(self);
        self
    }

    fn Dereference(&self) -> BlockNode {
        types::TableGroupedChildrenIteratorDereferenceFromTable(self)
    }

    fn Equals(&self, rhs: &Self) -> bool {
        types::TableGroupedChildrenIteratorEqualFromTable(self, rhs)
    }
}

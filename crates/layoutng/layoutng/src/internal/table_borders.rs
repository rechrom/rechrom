#![allow(non_snake_case)]

use foundation::{
    EBorderStyle, HeapVector, LayoutUnit, Member, String as BlinkString, Vector, Visitor,
    WritingDirectionMode,
};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;

// Bodies for these non-inline members are owned by //src/layoutng_table.
unsafe extern "Rust" {
    fn TableBordersComputeFromTable(table: &BlockNode) -> *const TableBorders;
    fn TableBordersConstructFromTable(table_border: &BoxStrut, is_collapsed: bool) -> TableBorders;
    #[cfg(debug_assertions)]
    fn TableBordersDumpEdgesFromTable(borders: &mut TableBorders) -> BlinkString;
    #[cfg(debug_assertions)]
    fn TableBordersEqualFromTable(left: &TableBorders, right: &TableBorders) -> bool;
    fn TableBordersCellBorderFromTable(
        borders: &TableBorders,
        cell: &BlockNode,
        row: u32,
        column: u32,
        section: u32,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut;
    fn TableBordersCellPaddingForMeasureFromTable(
        borders: &TableBorders,
        cell_style: &ComputedStyle,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut;
    fn TableBordersUpdateTableBorderFromTable(
        borders: &mut TableBorders,
        table_row_count: u32,
        table_column_count: u32,
    );
    fn TableBordersMergeBordersFromTable(
        borders: &mut TableBorders,
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
    fn TableBordersGetCellBordersFromTable(
        borders: &TableBorders,
        row: u32,
        column: u32,
        rowspan: u32,
        colspan: u32,
    ) -> BoxStrut;
    fn TableBordersEnsureCellColumnFitsFromTable(borders: &mut TableBorders, cell_column: u32);
    fn TableBordersEnsureCellRowFitsFromTable(borders: &mut TableBorders, cell_row: u32);
    fn TableBordersMergeRowAxisBorderFromTable(
        borders: &mut TableBorders,
        start_row: u32,
        start_column: u32,
        colspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    );
    fn TableBordersMergeColumnAxisBorderFromTable(
        borders: &mut TableBorders,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    );
    fn TableBordersMarkInnerBordersAsDoNotFillFromTable(
        borders: &mut TableBorders,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        colspan: u32,
    );
}

// cpp: layoutng/internal/table_borders.h:73-77
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeSource {
    kNone,
    kCell,
    kRow,
    kSection,
    kColumn,
    kTable,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeSide {
    kTop,
    kRight,
    kBottom,
    kLeft,
    kDoNotFill,
}

// cpp: layoutng/internal/table_borders.h:79-92
#[repr(C)]
pub struct Edge {
    pub style: Member<ComputedStyle>,
    pub edge_side: EdgeSide,
    pub box_order: u32,
}

impl Edge {
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.style);
    }
}

// cpp: layoutng/internal/table_borders.h:160-165
pub type Edges = HeapVector<Edge>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Section {
    pub start_row: u32,
    pub row_count: u32,
}

// cpp: layoutng/internal/table_borders.h:60-71
// cpp: layoutng/internal/table_borders.h:289-297
pub struct TableBorders {
    pub edges_: Edges,
    pub sections_: Vector<Section>,
    pub edges_per_row_: u32,
    pub table_border_: BoxStrut,
    pub last_column_index_: u32,
    pub is_collapsed_: bool,
}

impl TableBorders {
    // cpp: layoutng/internal/table_borders.h:62-66
    pub fn ComputeTableBorders(table: &BlockNode) -> *const Self {
        unsafe { TableBordersComputeFromTable(table) }
    }

    pub fn new(table_border: &BoxStrut, is_collapsed: bool) -> Self {
        unsafe { TableBordersConstructFromTable(table_border, is_collapsed) }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.edges_);
    }

    // cpp: layoutng/internal/table_borders.h:68-71
    #[cfg(debug_assertions)]
    pub fn DumpEdges(&mut self) -> BlinkString {
        unsafe { TableBordersDumpEdgesFromTable(self) }
    }

    #[cfg(debug_assertions)]
    pub fn Equals(&self, other: &Self) -> bool {
        unsafe { TableBordersEqualFromTable(self, other) }
    }

    // cpp: layoutng/internal/table_borders.h:94-110
    // Rust uses distinct names for the source's static and instance overloads.
    pub fn BorderWidthForStyle(style: *const ComputedStyle, edge_side: EdgeSide) -> LayoutUnit {
        if style.is_null() {
            return LayoutUnit::default();
        }
        let style = unsafe { &*style };
        match edge_side {
            EdgeSide::kLeft => LayoutUnit::from_signed(style.BorderLeftWidth()),
            EdgeSide::kRight => LayoutUnit::from_signed(style.BorderRightWidth()),
            EdgeSide::kTop => LayoutUnit::from_signed(style.BorderTopWidth()),
            EdgeSide::kBottom => LayoutUnit::from_signed(style.BorderBottomWidth()),
            EdgeSide::kDoNotFill => LayoutUnit::default(),
        }
    }

    // cpp: layoutng/internal/table_borders.h:112-135
    pub fn BorderStyleForStyle(style: *const ComputedStyle, edge_side: EdgeSide) -> EBorderStyle {
        if style.is_null() {
            return EBorderStyle::kNone;
        }
        let style = unsafe { &*style };
        let border_style = match edge_side {
            EdgeSide::kLeft => style.BorderLeftStyle(),
            EdgeSide::kRight => style.BorderRightStyle(),
            EdgeSide::kTop => style.BorderTopStyle(),
            EdgeSide::kBottom => style.BorderBottomStyle(),
            EdgeSide::kDoNotFill => EBorderStyle::kNone,
        };
        ComputedStyle::CollapsedBorderStyle(border_style)
    }

    // cpp: layoutng/internal/table_borders.h:137-144
    pub fn HasBorder(style: *const ComputedStyle) -> bool {
        if style.is_null() {
            return false;
        }
        let style = unsafe { &*style };
        style.BorderLeftStyle() != EBorderStyle::kNone
            || style.BorderRightStyle() != EBorderStyle::kNone
            || style.BorderTopStyle() != EBorderStyle::kNone
            || style.BorderBottomStyle() != EBorderStyle::kNone
    }

    // cpp: layoutng/internal/table_borders.h:146-158
    pub fn BorderWidth(&self, edge_index: u32) -> LayoutUnit {
        let edge = &self.edges_[edge_index as usize];
        Self::BorderWidthForStyle(edge.style.Get(), edge.edge_side)
    }

    pub fn BorderStyle(&self, edge_index: u32) -> EBorderStyle {
        let edge = &self.edges_[edge_index as usize];
        Self::BorderStyleForStyle(edge.style.Get(), edge.edge_side)
    }

    pub fn BoxOrder(&self, edge_index: u32) -> u32 {
        self.edges_[edge_index as usize].box_order
    }

    // cpp: layoutng/internal/table_borders.h:167-173
    pub fn IsEmpty(&self) -> bool {
        self.edges_.is_empty()
    }

    pub fn IsCollapsed(&self) -> bool {
        self.is_collapsed_
    }

    pub fn EdgesPerRow(&self) -> u32 {
        self.edges_per_row_
    }

    pub fn TableBorder(&self) -> &BoxStrut {
        &self.table_border_
    }

    // cpp: layoutng/internal/table_borders.h:175-198
    pub fn CellBorder(
        &self,
        cell: &BlockNode,
        row: u32,
        column: u32,
        section: u32,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut {
        unsafe {
            TableBordersCellBorderFromTable(
                self,
                cell,
                row,
                column,
                section,
                table_writing_direction,
            )
        }
    }

    pub fn CellPaddingForMeasure(
        &self,
        cell_style: &ComputedStyle,
        table_writing_direction: WritingDirectionMode,
    ) -> BoxStrut {
        unsafe {
            TableBordersCellPaddingForMeasureFromTable(self, cell_style, table_writing_direction)
        }
    }

    pub fn UpdateTableBorder(&mut self, table_row_count: u32, table_column_count: u32) {
        unsafe { TableBordersUpdateTableBorderFromTable(self, table_row_count, table_column_count) }
    }

    pub fn MergeBorders(
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
        unsafe {
            TableBordersMergeBordersFromTable(
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
    }

    // Rust has no default arguments; this preserves the source's kNotFound
    // section-index default without changing the MergeBorders call order.
    pub fn MergeBordersWithoutSection(
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

    // cpp: layoutng/internal/table_borders.h:200-210
    pub fn AddSection(&mut self, start_row: u32, row_count: u32) {
        self.sections_.push(Section {
            start_row,
            row_count,
        });
    }

    pub fn GetSection(&self, section_index: u32) -> Section {
        self.sections_[section_index as usize]
    }

    pub fn SetLastColumnIndex(&mut self, last_column_index: u32) {
        self.last_column_index_ = last_column_index;
    }

    // cpp: layoutng/internal/table_borders.h:212-216
    pub fn begin(&self) -> *const Edge {
        self.edges_.as_ptr()
    }

    pub fn end(&self) -> *const Edge {
        unsafe { self.edges_.as_ptr().add(self.edges_.len()) }
    }

    pub fn EdgeCount(&self) -> u32 {
        self.edges_.len() as u32
    }

    // cpp: layoutng/internal/table_borders.h:218-228
    pub fn CanPaint(&self, edge_index: u32) -> bool {
        if !self.HasEdgeAtIndex(edge_index) {
            return false;
        }
        let border_style = self.BorderStyle(edge_index);
        if border_style == EBorderStyle::kNone || border_style == EBorderStyle::kHidden {
            return false;
        }
        if self.BorderWidth(edge_index) == LayoutUnit::default() {
            return false;
        }
        true
    }

    // cpp: layoutng/internal/table_borders.h:230-232
    pub fn HasEdgeAtIndex(&self, edge_index: u32) -> bool {
        (edge_index as usize) < self.edges_.len()
            && !self.edges_[edge_index as usize].style.Get().is_null()
    }

    // cpp: layoutng/internal/table_borders.h:235-241
    pub fn CanPaintWithOffset(&self, edge_index: u32, index_offset: i32) -> bool {
        (index_offset >= 0 || (index_offset < 0 && edge_index >= index_offset.unsigned_abs()))
            && (edge_index.wrapping_add(index_offset as u32) as usize) < self.edges_.len()
            && !self.edges_[edge_index.wrapping_add(index_offset as u32) as usize]
                .style
                .Get()
                .is_null()
    }

    // cpp: layoutng/internal/table_borders.h:244-247
    pub fn ClampColspan(&self, column: u32, colspan: u32) -> u32 {
        debug_assert!(self.last_column_index_ >= column);
        std::cmp::min(colspan, self.last_column_index_.wrapping_sub(column))
    }

    // cpp: layoutng/internal/table_borders.h:250-259
    pub fn ClampRowspan(&self, section_index: u32, table_row_index: u32, rowspan: u32) -> u32 {
        if rowspan <= 1 {
            return rowspan;
        }
        debug_assert!((section_index as usize) < self.sections_.len());
        let section = self.sections_[section_index as usize];
        std::cmp::min(
            rowspan,
            section
                .row_count
                .wrapping_sub(table_row_index.wrapping_sub(section.start_row)),
        )
    }

    // cpp: layoutng/internal/table_borders.h:261-287
    pub fn GetCellBorders(&self, row: u32, column: u32, rowspan: u32, colspan: u32) -> BoxStrut {
        unsafe { TableBordersGetCellBordersFromTable(self, row, column, rowspan, colspan) }
    }

    pub fn EnsureCellColumnFits(&mut self, cell_column: u32) {
        unsafe { TableBordersEnsureCellColumnFitsFromTable(self, cell_column) }
    }

    pub fn EnsureCellRowFits(&mut self, cell_row: u32) {
        unsafe { TableBordersEnsureCellRowFitsFromTable(self, cell_row) }
    }

    pub fn MergeRowAxisBorder(
        &mut self,
        start_row: u32,
        start_column: u32,
        colspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    ) {
        unsafe {
            TableBordersMergeRowAxisBorderFromTable(
                self,
                start_row,
                start_column,
                colspan,
                source_style,
                box_order,
                side,
            )
        }
    }

    pub fn MergeColumnAxisBorder(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        source_style: &ComputedStyle,
        box_order: u32,
        side: EdgeSide,
    ) {
        unsafe {
            TableBordersMergeColumnAxisBorderFromTable(
                self,
                start_row,
                start_column,
                rowspan,
                source_style,
                box_order,
                side,
            )
        }
    }

    pub fn MarkInnerBordersAsDoNotFill(
        &mut self,
        start_row: u32,
        start_column: u32,
        rowspan: u32,
        colspan: u32,
    ) {
        unsafe {
            TableBordersMarkInnerBordersAsDoNotFillFromTable(
                self,
                start_row,
                start_column,
                rowspan,
                colspan,
            )
        }
    }
}

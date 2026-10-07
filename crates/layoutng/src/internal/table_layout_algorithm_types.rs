#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use foundation::{HeapVector, LayoutUnit, Vector, Visitor, WritingDirectionMode};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;
use super::layout_input_node::LayoutInputNode;
use super::min_max_sizes::MinMaxSizes;

// Non-inline member bodies are owned by //src/layoutng_table.
unsafe extern "Rust" {
    fn TableCellInlineConstraintEncompassFromTable(
        constraint: &mut CellInlineConstraint,
        other: &CellInlineConstraint,
    );
    fn TableColumnEncompassFromTable(column: &mut Column, cell: &Option<CellInlineConstraint>);
    fn TableTypesCreateColumnFromTable(
        style: &ComputedStyle,
        default_inline_size: Option<LayoutUnit>,
        is_table_fixed: bool,
    ) -> Column;
    fn TableTypesCreateCellInlineConstraintFromTable(
        node: &BlockNode,
        table_writing_direction: WritingDirectionMode,
        is_fixed_layout: bool,
        cell_border: &BoxStrut,
        cell_padding: &BoxStrut,
    ) -> CellInlineConstraint;
    fn TableTypesCreateSectionFromTable(
        node: &LayoutInputNode,
        start_row: u32,
        row_count: u32,
        block_size: LayoutUnit,
        treat_as_tbody: bool,
    ) -> Section;
    fn TableGroupedChildrenConstructFromTable(table: &BlockNode) -> TableGroupedChildren;
    fn TableGroupedChildrenTraceFromTable(children: &TableGroupedChildren, visitor: &mut Visitor);
    fn TableGroupedChildrenIteratorConstructFromTable(
        grouped_children: &TableGroupedChildren,
        is_end: bool,
    ) -> TableGroupedChildrenIterator;
    fn TableGroupedChildrenIteratorIncrementFromTable(iterator: &mut TableGroupedChildrenIterator);
    fn TableGroupedChildrenIteratorDecrementFromTable(iterator: &mut TableGroupedChildrenIterator);
    fn TableGroupedChildrenIteratorDereferenceFromTable(
        iterator: &TableGroupedChildrenIterator,
    ) -> BlockNode;
    fn TableGroupedChildrenIteratorEqualFromTable(
        left: &TableGroupedChildrenIterator,
        right: &TableGroupedChildrenIterator,
    ) -> bool;
    fn TableGroupedChildrenIteratorAdvanceForwardFromTable(
        iterator: &mut TableGroupedChildrenIterator,
    );
    fn TableGroupedChildrenIteratorAdvanceBackwardFromTable(
        iterator: &mut TableGroupedChildrenIterator,
    );
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:24-28
pub struct TableTypes;

impl TableTypes {
    // The source constexpr is a LayoutUnit; its external Rust owner does not
    // yet expose a const constructor. Preserve its value and type as a method.
    pub fn kTableMaxInlineSize() -> LayoutUnit {
        LayoutUnit::from_signed(1_000_000)
    }

    // cpp: layoutng/internal/table_layout_algorithm_types.h:206-221
    pub fn CreateColumn(
        style: &ComputedStyle,
        default_inline_size: Option<LayoutUnit>,
        is_table_fixed: bool,
    ) -> Column {
        unsafe { TableTypesCreateColumnFromTable(style, default_inline_size, is_table_fixed) }
    }

    pub fn CreateCellInlineConstraint(
        node: &BlockNode,
        table_writing_direction: WritingDirectionMode,
        is_fixed_layout: bool,
        cell_border: &BoxStrut,
        cell_padding: &BoxStrut,
    ) -> CellInlineConstraint {
        unsafe {
            TableTypesCreateCellInlineConstraintFromTable(
                node,
                table_writing_direction,
                is_fixed_layout,
                cell_border,
                cell_padding,
            )
        }
    }

    pub fn CreateSection(
        node: &LayoutInputNode,
        start_row: u32,
        row_count: u32,
        block_size: LayoutUnit,
        treat_as_tbody: bool,
    ) -> Section {
        unsafe {
            TableTypesCreateSectionFromTable(node, start_row, row_count, block_size, treat_as_tbody)
        }
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:30-42
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CellInlineConstraint {
    pub min_inline_size: LayoutUnit,
    pub max_inline_size: LayoutUnit,
    pub percent: Option<f32>,
    pub percent_border_padding: LayoutUnit,
    pub is_constrained: bool,
}

impl CellInlineConstraint {
    pub fn Encompass(&mut self, other: &Self) {
        unsafe { TableCellInlineConstraintEncompassFromTable(self, other) }
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:44-56
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ColspanCell {
    pub cell_inline_constraint: CellInlineConstraint,
    pub start_column: u32,
    pub span: u32,
}

impl ColspanCell {
    pub fn new(
        cell_inline_constraint: &CellInlineConstraint,
        start_column: u32,
        span: u32,
    ) -> Self {
        Self {
            cell_inline_constraint: *cell_inline_constraint,
            start_column,
            span,
        }
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:58-101
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct Column {
    pub min_inline_size: Option<LayoutUnit>,
    pub max_inline_size: Option<LayoutUnit>,
    pub percent: Option<f32>,
    pub percent_border_padding: LayoutUnit,
    pub is_constrained: bool,
    pub is_collapsed: bool,
    pub is_table_fixed: bool,
    pub is_mergeable: bool,
}

impl Column {
    pub fn new(
        min_inline_size: Option<LayoutUnit>,
        max_inline_size: Option<LayoutUnit>,
        percent: Option<f32>,
        percent_border_padding: LayoutUnit,
        is_constrained: bool,
        is_collapsed: bool,
        is_table_fixed: bool,
        is_mergeable: bool,
    ) -> Self {
        Self {
            min_inline_size,
            max_inline_size,
            percent,
            percent_border_padding,
            is_constrained,
            is_collapsed,
            is_table_fixed,
            is_mergeable,
        }
    }

    // cpp: layoutng/internal/table_layout_algorithm_types.h:103-103
    pub fn Encompass(&mut self, cell: &Option<CellInlineConstraint>) {
        unsafe { TableColumnEncompassFromTable(self, cell) }
    }

    // cpp: layoutng/internal/table_layout_algorithm_types.h:104-110
    pub fn ResolvePercentInlineSize(
        &self,
        percentage_resolution_inline_size: LayoutUnit,
    ) -> LayoutUnit {
        let percent = self.percent.expect("percentage column size is required");
        std::cmp::max(
            self.min_inline_size.unwrap_or_default(),
            LayoutUnit::from_f32(percent * percentage_resolution_inline_size.ToFloat() / 100.0)
                + self.percent_border_padding,
        )
    }

    // cpp: layoutng/internal/table_layout_algorithm_types.h:111-113
    pub fn IsFixed(&self) -> bool {
        self.is_constrained && self.percent.is_none() && self.max_inline_size.is_some()
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:116-139
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CellBlockConstraint {
    pub min_block_size: LayoutUnit,
    pub borders: BoxStrut,
    pub column_index: u32,
    pub effective_rowspan: u32,
    pub is_constrained: bool,
    pub has_descendant_that_depends_on_percentage_block_size: bool,
}

impl CellBlockConstraint {
    pub fn new(
        min_block_size: LayoutUnit,
        borders: BoxStrut,
        column_index: u32,
        effective_rowspan: u32,
        is_constrained: bool,
        has_descendant_that_depends_on_percentage_block_size: bool,
    ) -> Self {
        Self {
            min_block_size,
            borders,
            column_index,
            effective_rowspan,
            is_constrained,
            has_descendant_that_depends_on_percentage_block_size,
        }
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:141-180
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RowspanCell {
    pub start_row: u32,
    pub effective_rowspan: u32,
    pub min_block_size: LayoutUnit,
}

impl RowspanCell {
    pub fn new(start_row: u32, effective_rowspan: u32, min_block_size: LayoutUnit) -> Self {
        Self {
            start_row,
            effective_rowspan,
            min_block_size,
        }
    }

    pub fn LessThan(&self, rhs: &Self) -> bool {
        let is_enclosed = |inner: &Self, outer: &Self| {
            inner.start_row >= outer.start_row
                && inner.start_row.wrapping_add(inner.effective_rowspan)
                    <= outer.start_row.wrapping_add(outer.effective_rowspan)
        };
        if self.start_row == rhs.start_row && self.effective_rowspan == rhs.effective_rowspan {
            return self.min_block_size > rhs.min_block_size;
        }
        if is_enclosed(self, rhs) {
            return true;
        }
        if is_enclosed(rhs, self) {
            return false;
        }
        self.start_row < rhs.start_row
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:182-194
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Row {
    pub block_size: LayoutUnit,
    pub start_cell_index: u32,
    pub cell_count: u32,
    pub baseline: Option<LayoutUnit>,
    pub percent: Option<f32>,
    pub is_constrained: bool,
    pub has_rowspan_start: bool,
    pub is_collapsed: bool,
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:196-204
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Section {
    pub start_row: u32,
    pub row_count: u32,
    pub block_size: LayoutUnit,
    pub percent: Option<f32>,
    pub is_constrained: bool,
    pub is_tbody: bool,
    pub needs_redistribution: bool,
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:223-233
// The scoped_refptr ownership of RefCountedData<Vector<Column>> is mapped at
// call sites to Arc<Columns>; this record keeps the source's data member.
pub struct Columns {
    pub data: Vector<Column>,
}

pub type CellInlineConstraints = Vector<Option<CellInlineConstraint>>;
pub type ColspanCells = Vector<ColspanCell>;
pub type Caption = MinMaxSizes;
pub type CellBlockConstraints = Vector<CellBlockConstraint>;
pub type RowspanCells = Vector<RowspanCell>;
pub type Rows = Vector<Row>;
pub type Sections = Vector<Section>;

// cpp: layoutng/internal/table_layout_algorithm_types.h:240-267
#[repr(C)]
pub struct TableGroupedChildren {
    pub captions: HeapVector<BlockNode>,
    pub columns: HeapVector<BlockNode>,
    pub header: BlockNode,
    pub bodies: HeapVector<BlockNode>,
    pub footer: BlockNode,
}

impl TableGroupedChildren {
    pub fn new(table: &BlockNode) -> Self {
        unsafe { TableGroupedChildrenConstructFromTable(table) }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        unsafe { TableGroupedChildrenTraceFromTable(self, visitor) }
    }

    pub fn begin(&self) -> TableGroupedChildrenIterator {
        TableGroupedChildrenIterator::new(self, false)
    }

    pub fn end(&self) -> TableGroupedChildrenIterator {
        TableGroupedChildrenIterator::new(self, true)
    }
}

impl Drop for TableGroupedChildren {
    // cpp: layoutng/internal/table_layout_algorithm_types.h:245-249
    fn drop(&mut self) {
        self.captions.clear();
        self.columns.clear();
        self.bodies.clear();
    }
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:274-274
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentSection {
    kNone,
    kHead,
    kBody,
    kFoot,
    kEnd,
}

// cpp: layoutng/internal/table_layout_algorithm_types.h:271-298
#[repr(C)]
pub struct TableGroupedChildrenIterator {
    pub grouped_children_: *const TableGroupedChildren,
    pub current_section_: CurrentSection,
    pub body_vector_: *const HeapVector<BlockNode>,
    pub position_: u32,
}

impl TableGroupedChildrenIterator {
    pub fn new(grouped_children: &TableGroupedChildren, is_end: bool) -> Self {
        unsafe { TableGroupedChildrenIteratorConstructFromTable(grouped_children, is_end) }
    }

    pub fn Increment(&mut self) -> &mut Self {
        unsafe { TableGroupedChildrenIteratorIncrementFromTable(self) };
        self
    }

    pub fn Decrement(&mut self) -> &mut Self {
        unsafe { TableGroupedChildrenIteratorDecrementFromTable(self) };
        self
    }

    pub fn Dereference(&self) -> BlockNode {
        unsafe { TableGroupedChildrenIteratorDereferenceFromTable(self) }
    }

    pub fn Equals(&self, rhs: &Self) -> bool {
        unsafe { TableGroupedChildrenIteratorEqualFromTable(self, rhs) }
    }

    // cpp: layoutng/internal/table_layout_algorithm_types.h:286-286
    pub fn TreatAsTBody(&self) -> bool {
        self.current_section_ == CurrentSection::kBody
    }

    // cpp: layoutng/internal/table_layout_algorithm_types.h:289-290
    fn AdvanceForwardToNonEmptySection(&mut self) {
        unsafe { TableGroupedChildrenIteratorAdvanceForwardFromTable(self) }
    }

    fn AdvanceBackwardToNonEmptySection(&mut self) {
        unsafe { TableGroupedChildrenIteratorAdvanceBackwardFromTable(self) }
    }
}

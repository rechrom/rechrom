#![allow(non_snake_case)]
use crate::layout_table_column_visitor::{ColumnVisitor, VisitLayoutTableColumn};
pub use crate::table_size_distribution::*;
use foundation::{HeapVector, Vector};
use layoutng_assembly::internal::{
    block_node::BlockNode,
    layout_object::LayoutObject,
    table_layout_algorithm_types::{Column, Columns, TableTypes},
};

// cpp: layoutng_table/table_layout_utils.h:111-132
#[derive(Clone, Copy)]
pub struct ColspanCell {
    pub column_start: u32,
    pub span: u32,
    pub remaining_rows: u32,
}

// cpp: layoutng_table/table_layout_utils.h:140-160
#[derive(Default)]
pub struct RowBaselineTabulator {
    max_cell_ascent: Option<foundation::LayoutUnit>,
    max_cell_descent: Option<foundation::LayoutUnit>,
    max_cell_baseline_depends_on_percentage_block_descendant: bool,
    fallback_cell_descent: Option<foundation::LayoutUnit>,
    fallback_cell_depends_on_percentage_block_descendant: bool,
}
impl RowBaselineTabulator {
    // cpp: layoutng_table/table_layout_utils.cc:1680-1714
    pub fn ProcessCell(
        &mut self,
        fragment: &layoutng_assembly::logical_box_fragment::LogicalBoxFragment<'_>,
        align: layoutng_assembly::internal::layout_alignment_utils::BlockContentAlignment,
        rowspanned: bool,
        percentage_descendant: bool,
    ) {
        use foundation::LayoutUnit;
        use layoutng_assembly::internal::layout_alignment_utils::BlockContentAlignment;
        if align == BlockContentAlignment::kBaseline
            && fragment.HasDescendantsForTablePart()
            && fragment.FirstBaseline().is_some()
        {
            self.max_cell_baseline_depends_on_percentage_block_descendant |= percentage_descendant;
            let baseline = fragment.FirstBaseline().unwrap();
            self.max_cell_ascent = Some(std::cmp::max(
                self.max_cell_ascent.unwrap_or(LayoutUnit::Min()),
                baseline,
            ));
            if rowspanned {
                self.max_cell_descent.get_or_insert(LayoutUnit::default());
            } else {
                self.max_cell_descent = Some(std::cmp::max(
                    self.max_cell_descent.unwrap_or(LayoutUnit::Min()),
                    fragment.BlockSize() - baseline,
                ));
            }
        }
        if self.max_cell_ascent.is_none() {
            self.fallback_cell_depends_on_percentage_block_descendant |= percentage_descendant;
            self.fallback_cell_descent = Some(std::cmp::min(
                self.fallback_cell_descent.unwrap_or(LayoutUnit::Max()),
                fragment.Padding().block_end + fragment.Borders().block_end,
            ));
        }
    }
    // cpp: layoutng_table/table_layout_utils.cc:1716-1723
    pub fn ComputeRowBlockSize(
        &self,
        max_cell_block_size: foundation::LayoutUnit,
    ) -> foundation::LayoutUnit {
        if let Some(ascent) = self.max_cell_ascent {
            std::cmp::max(max_cell_block_size, ascent + self.max_cell_descent.unwrap())
        } else {
            max_cell_block_size
        }
    }
    // cpp: layoutng_table/table_layout_utils.cc:1725-1733
    pub fn ComputeBaseline(
        &self,
        row_block_size: foundation::LayoutUnit,
    ) -> foundation::LayoutUnit {
        if let Some(ascent) = self.max_cell_ascent {
            ascent
        } else if let Some(descent) = self.fallback_cell_descent {
            (row_block_size - descent).ClampNegativeToZero()
        } else {
            foundation::LayoutUnit::default()
        }
    }
    // cpp: layoutng_table/table_layout_utils.cc:1735-1741
    pub fn BaselineDependsOnPercentageBlockDescendant(&self) -> bool {
        if self.max_cell_ascent.is_some() {
            self.max_cell_baseline_depends_on_percentage_block_descendant
        } else if self.fallback_cell_descent.is_some() {
            self.fallback_cell_depends_on_percentage_block_descendant
        } else {
            false
        }
    }
}

// cpp: layoutng_table/table_layout_utils.cc:1363-1424
pub fn SetupTableCellConstraintSpaceBuilder(
    table_direction: foundation::WritingDirectionMode,
    cell: &BlockNode,
    borders: &layoutng_geometry::geometry::box_strut::BoxStrut,
    locations: &Vector<layoutng_assembly::internal::table_column_location::TableColumnLocation>,
    block_size: foundation::LayoutUnit,
    percentage_inline_size: foundation::LayoutUnit,
    baseline: Option<foundation::LayoutUnit>,
    start: u32,
    initial_indefinite: bool,
    table_size_specified: bool,
    collapsed: bool,
    cache: layoutng_assembly::internal::constraint_space::LayoutResultCacheSlot,
    builder: &mut layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder,
) {
    use foundation::{kIndefiniteSize, EEmptyCells, IsParallelWritingMode};
    use layoutng_geometry::geometry::logical_size::LogicalSize;
    let style = cell.Style();
    let end = std::cmp::min(
        start + cell.TableCellColspan() - 1,
        locations.len() as u32 - 1,
    ) as usize;
    let inline_size =
        locations[end].offset + locations[end].size - locations[start as usize].offset;
    let hidden = locations[start as usize..=end]
        .iter()
        .all(|v| v.is_collapsed);
    builder.SetIsTableCell(true);
    if !IsParallelWritingMode(table_direction.GetWritingMode(), style.GetWritingMode()) {
        let icb = cell.InitialContainingBlockSize();
        builder.SetOrthogonalFallbackInlineSize(if table_direction.IsHorizontal() {
            icb.height
        } else {
            icb.width
        });
    }
    builder.SetAvailableSize(LogicalSize::new(inline_size, block_size));
    builder.SetIsFixedInlineSize(true);
    if block_size != kIndefiniteSize {
        builder.SetIsFixedBlockSize(true);
    }
    builder.SetIsInitialBlockSizeIndefinite(initial_indefinite);
    builder.SetPercentageResolutionSize(LogicalSize::new(percentage_inline_size, kIndefiniteSize));
    builder.SetTableCellBorders(borders, style.GetWritingDirection(), table_direction);
    builder.SetTableCellAlignmentBaseline(baseline);
    builder.SetTableCellColumnIndex(start);
    builder
        .SetIsRestrictedBlockSizeTableCell(table_size_specified || style.LogicalHeight().IsFixed());
    builder.SetIsHiddenForPaint(hidden);
    builder.SetIsTableCellWithCollapsedBorders(collapsed);
    builder.SetHideTableCellIfEmpty(!collapsed && style.EmptyCells() == EEmptyCells::kHide);
    builder.SetCacheSlot(cache);
}
#[derive(Default)]
pub struct ColspanCellTabulator {
    current_column: u32,
    colspanned_cells: Vector<ColspanCell>,
}
impl ColspanCellTabulator {
    pub fn CurrentColumn(&self) -> u32 {
        self.current_column
    }
    // cpp: layoutng_table/table_layout_utils.cc:1642-1644
    pub fn StartRow(&mut self) {
        self.current_column = 0;
    }
    // cpp: layoutng_table/table_layout_utils.cc:1647-1660
    pub fn EndRow(&mut self) {
        for cell in &mut self.colspanned_cells {
            cell.remaining_rows = cell.remaining_rows.wrapping_sub(1);
        }
        self.colspanned_cells
            .retain(|cell| cell.remaining_rows != 0);
        self.colspanned_cells
            .sort_unstable_by_key(|cell| cell.column_start);
    }
    // cpp: layoutng_table/table_layout_utils.cc:1663-1670
    pub fn FindNextFreeColumn(&mut self) {
        for cell in &self.colspanned_cells {
            let end = cell.column_start.wrapping_add(cell.span);
            if cell.column_start <= self.current_column && end > self.current_column {
                self.current_column = end;
            }
        }
    }
    // cpp: layoutng_table/table_layout_utils.cc:1672-1678
    pub fn ProcessCell(&mut self, cell: &BlockNode) {
        let colspan = cell.TableCellColspan();
        let rowspan = cell.TableCellRowspan();
        if rowspan > 1 {
            self.colspanned_cells.push(ColspanCell {
                column_start: self.current_column,
                span: colspan,
                remaining_rows: rowspan,
            });
        }
        self.current_column = self.current_column.wrapping_add(colspan);
    }
}

// cpp: layoutng_table/table_layout_utils.h:164-173
pub const fn ComputeMaxColumn(current_column: u32, colspan: u32, is_fixed_layout: bool) -> u32 {
    current_column.wrapping_add(if is_fixed_layout { colspan } else { 1 })
}

// cpp: layoutng_table/table_layout_utils.cc:336-383
pub struct ColumnConstraintsBuilder<'a> {
    columns: &'a mut Columns,
    is_fixed_layout: bool,
    colgroup_constraint: Option<Column>,
}
impl<'a> ColumnConstraintsBuilder<'a> {
    pub fn new(columns: &'a mut Columns, is_fixed_layout: bool) -> Self {
        Self {
            columns,
            is_fixed_layout,
            colgroup_constraint: None,
        }
    }
}
impl ColumnVisitor for ColumnConstraintsBuilder<'_> {
    fn VisitCol(&mut self, column: &BlockNode, _start: u32, span: u32) {
        let default = if !self.is_fixed_layout {
            self.colgroup_constraint.and_then(|v| v.max_inline_size)
        } else {
            None
        };
        let constraint = TableTypes::CreateColumn(column.Style(), default, self.is_fixed_layout);
        for _ in 0..span {
            self.columns.data.push(constraint);
        }
        unsafe { &mut *column.GetLayoutBox() }.ClearNeedsLayout();
    }
    fn EnterColgroup(&mut self, colgroup: &BlockNode, _start: u32) {
        self.colgroup_constraint = Some(TableTypes::CreateColumn(
            colgroup.Style(),
            None,
            self.is_fixed_layout,
        ));
    }
    fn LeaveColgroup(&mut self, colgroup: &BlockNode, _start: u32, span: u32, has_children: bool) {
        if !has_children {
            for _ in 0..span {
                self.columns.data.push(self.colgroup_constraint.unwrap());
            }
        }
        self.colgroup_constraint = None;
        unsafe { &mut *colgroup.GetLayoutBox() }.ClearNeedsLayout();
        ClearColumnChildrenNeedsLayout(colgroup);
    }
}

// cpp: layoutng_table/layout_table_column.cc:141-148
// The children are accessed through the source native virtual child list.
fn ClearColumnChildrenNeedsLayout(column: &BlockNode) {
    let object = unsafe { &*column.GetLayoutBox().cast::<LayoutObject>() };
    object.CheckIsNotDestroyed();
    debug_assert!(object.IsLayoutTableCol());
    let mut child = object.SlowFirstChild();
    while !child.is_null() {
        let object = unsafe { &mut *child };
        object.ClearNeedsLayout();
        child = object.NextSibling();
    }
}

// cpp: layoutng_table/table_layout_utils.cc:386-393
pub fn ComputeColumnElementConstraints(
    columns: &HeapVector<BlockNode>,
    is_fixed_layout: bool,
    constraints: &mut Columns,
) {
    VisitLayoutTableColumn(
        columns,
        u32::MAX,
        &mut ColumnConstraintsBuilder::new(constraints, is_fixed_layout),
    );
}
// cpp: layoutng_table/table_layout_utils.cc:1428-1447
pub fn ComputeMaximumNonMergeableColumnCount(
    columns: &HeapVector<BlockNode>,
    is_fixed_layout: bool,
) -> u32 {
    let mut constraints = Columns {
        data: Vector::new(),
    };
    ComputeColumnElementConstraints(columns, is_fixed_layout, &mut constraints);
    if constraints.data.is_empty() {
        return 0;
    }
    let mut index = constraints.data.len() - 1;
    while index > 0 && constraints.data[index].is_mergeable {
        index -= 1;
    }
    if index == 0 && constraints.data[0].is_mergeable {
        0
    } else {
        (index + 1) as u32
    }
}

// cpp: layoutng_table/table_layout_utils.cc:395-454
pub fn ComputeSectionInlineConstraints(
    section: &BlockNode,
    fixed: bool,
    first_section: bool,
    direction: foundation::WritingDirectionMode,
    borders: &layoutng_assembly::internal::table_borders::TableBorders,
    section_index: u32,
    row_index: &mut u32,
    cells: &mut layoutng_assembly::internal::table_layout_algorithm_types::CellInlineConstraints,
    colspan: &mut layoutng_assembly::internal::table_layout_algorithm_types::ColspanCells,
) {
    let mut tabulator = ColspanCellTabulator::default();
    let mut first_row = true;
    let mut row = BlockNode::from(section.FirstChild());
    while row.is_non_null() {
        tabulator.StartRow();
        let mut cell = BlockNode::from(row.FirstChild());
        while cell.is_non_null() {
            tabulator.FindNextFreeColumn();
            let span = cell.TableCellColspan();
            let ignore = fixed && (!first_section || !first_row);
            let max_column = ComputeMaxColumn(tabulator.CurrentColumn(), span, fixed) as usize;
            if max_column >= cells.len() {
                cells.resize(max_column, None);
            }
            if !ignore {
                let border = borders.CellBorder(
                    &cell,
                    *row_index,
                    tabulator.CurrentColumn(),
                    section_index,
                    direction,
                );
                let padding = borders.CellPaddingForMeasure(cell.Style(), direction);
                let constraint = TableTypes::CreateCellInlineConstraint(
                    &cell, direction, fixed, &border, &padding,
                );
                if span == 1 {
                    let slot = &mut cells[tabulator.CurrentColumn() as usize];
                    if let Some(existing) = slot {
                        existing.Encompass(&constraint);
                    } else {
                        *slot = Some(constraint);
                    }
                } else {
                    colspan.push(
                        layoutng_assembly::internal::table_layout_algorithm_types::ColspanCell::new(
                            &constraint,
                            tabulator.CurrentColumn(),
                            span,
                        ),
                    );
                }
            }
            tabulator.ProcessCell(&cell);
            cell = BlockNode::from(cell.NextSibling());
        }
        first_row = false;
        *row_index = row_index.wrapping_add(1);
        tabulator.EndRow();
        row = BlockNode::from(row.NextSibling());
    }
}
// cpp: layoutng_table/table_layout_utils.cc:1450-1485
pub fn ComputeColumnConstraints(
    table: &BlockNode,
    grouped: &layoutng_assembly::internal::table_layout_algorithm_types::TableGroupedChildren,
    borders: &layoutng_assembly::internal::table_borders::TableBorders,
    _border_padding: &layoutng_geometry::geometry::box_strut::BoxStrut,
) -> std::sync::Arc<Columns> {
    let style = table.Style();
    let fixed = style.IsFixedTableLayout();
    let (mut cells, mut colspan) = (Vector::new(), Vector::new());
    let mut columns = Columns {
        data: Vector::new(),
    };
    ComputeColumnElementConstraints(&grouped.columns, fixed, &mut columns);
    let mut first = true;
    let (mut row_index, mut section_index) = (0, 0);
    let mut it = grouped.begin();
    let end = grouped.end();
    while !it.Equals(&end) {
        let section = it.Dereference();
        if !section.IsEmptyTableSection() {
            ComputeSectionInlineConstraints(
                &section,
                fixed,
                first,
                style.GetWritingDirection(),
                borders,
                section_index,
                &mut row_index,
                &mut cells,
                &mut colspan,
            );
            first = false;
        }
        section_index += 1;
        it.Increment();
    }
    ApplyCellConstraintsToColumnConstraints(
        &cells,
        style.TableBorderSpacing().inline_size,
        fixed,
        &mut colspan,
        &mut columns,
    );
    std::sync::Arc::new(columns)
}

pub use crate::table_row_measurement::{
    ComputeSectionMinimumRowBlockSizes, FinalizeTableCellLayout,
};

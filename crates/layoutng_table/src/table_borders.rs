#![allow(non_snake_case)]
use crate::{
    layout_table_column_visitor::{ColumnVisitor, VisitLayoutTableColumn},
    table_layout_utils::{
        ColspanCellTabulator, ComputeMaxColumn, ComputeMaximumNonMergeableColumnCount,
    },
};
use foundation::{
    EBorderCollapse, EBorderStyle, HeapVector, LayoutUnit, MakeGarbageCollected, Member,
    PhysicalToLogical, Vector, WritingDirectionMode,
};
use layoutng_assembly::internal::{
    block_node::BlockNode,
    constraint_space_builder::ConstraintSpaceBuilder,
    length_utils::{ComputeBorders, ComputeNonCollapsedTableBorders, ComputePadding},
    table_borders::{Edge, EdgeSide, EdgeSource, TableBorders},
    table_layout_algorithm_types::TableGroupedChildren,
};
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_table/table_borders.cc:22-44
fn IsSourceMoreSpecificThanEdge(style: EBorderStyle, width: LayoutUnit, edge: &Edge) -> bool {
    if edge.edge_side == EdgeSide::kDoNotFill {
        return false;
    }
    if edge.style.Get().is_null() || style == EBorderStyle::kHidden {
        return true;
    }
    let current = TableBorders::BorderStyleForStyle(edge.style.Get(), edge.edge_side);
    if current == EBorderStyle::kHidden {
        return false;
    }
    let current_width = TableBorders::BorderWidthForStyle(edge.style.Get(), edge.edge_side);
    if width < current_width {
        return false;
    }
    if width > current_width {
        return true;
    }
    style as i32 > current as i32
}

// cpp: layoutng_table/table_borders.cc:46-76
struct ColBordersMarker<'a> {
    rows: u32,
    order: u32,
    direction: WritingDirectionMode,
    borders: &'a mut TableBorders,
}
impl ColumnVisitor for ColBordersMarker<'_> {
    fn VisitCol(&mut self, column: &BlockNode, start: u32, span: u32) {
        for i in 0..span {
            self.borders.MergeBordersWithoutSection(
                0,
                start + i,
                self.rows,
                1,
                column.Style(),
                EdgeSource::kColumn,
                self.order,
                self.direction,
            );
        }
    }
    fn EnterColgroup(&mut self, _: &BlockNode, _: u32) {}
    fn LeaveColgroup(&mut self, _: &BlockNode, _: u32, _: u32, _: bool) {}
}
// cpp: layoutng_table/table_borders.cc:79-108
struct ColgroupBordersMarker<'a> {
    rows: u32,
    order: u32,
    direction: WritingDirectionMode,
    borders: &'a mut TableBorders,
}
impl ColumnVisitor for ColgroupBordersMarker<'_> {
    fn VisitCol(&mut self, _: &BlockNode, _: u32, _: u32) {}
    fn EnterColgroup(&mut self, _: &BlockNode, _: u32) {}
    fn LeaveColgroup(&mut self, column: &BlockNode, start: u32, span: u32, _: bool) {
        self.borders.MergeBordersWithoutSection(
            0,
            start,
            self.rows,
            span,
            column.Style(),
            EdgeSource::kColumn,
            self.order,
            self.direction,
        );
    }
}
fn sections(grouped: &TableGroupedChildren) -> Vec<BlockNode> {
    let mut iterator = grouped.begin();
    let end = grouped.end();
    let mut result = Vec::new();
    while !iterator.Equals(&end) {
        result.push(iterator.Dereference());
        iterator.Increment();
    }
    result
}

// cpp: layoutng_table/table_borders.cc:113-246
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersComputeFromTable(table: &BlockNode) -> *const TableBorders {
    let style = table.Style();
    let collapsed = style.BorderCollapse() == EBorderCollapse::kCollapse;
    let pointer = MakeGarbageCollected(TableBorders::new(
        &ComputeNonCollapsedTableBorders(style),
        collapsed,
    ));
    if !collapsed {
        return pointer;
    }
    let borders = unsafe { &mut *pointer };
    let grouped = TableGroupedChildren::new(table);
    let direction = style.GetWritingDirection();
    let mut order = 0_u32;
    let mut column_count =
        ComputeMaximumNonMergeableColumnCount(&grouped.columns, style.IsFixedTableLayout());
    let mut row_index = 0_u32;
    let mut multispan = false;
    let sections = sections(&grouped);
    for section in &sections {
        let start = row_index;
        let mut tabulator = ColspanCellTabulator::default();
        let mut row = BlockNode::from(section.FirstChild());
        while row.is_non_null() {
            tabulator.StartRow();
            let mut cell = BlockNode::from(row.FirstChild());
            while cell.is_non_null() {
                tabulator.FindNextFreeColumn();
                let colspan = cell.TableCellColspan();
                multispan |= cell.TableCellRowspan() > 1 || colspan > 1;
                column_count = column_count.max(ComputeMaxColumn(
                    tabulator.CurrentColumn(),
                    colspan,
                    style.IsFixedTableLayout(),
                ));
                if !multispan {
                    order = order.wrapping_add(1);
                    borders.MergeBordersWithoutSection(
                        row_index,
                        tabulator.CurrentColumn(),
                        cell.TableCellRowspan(),
                        colspan,
                        cell.Style(),
                        EdgeSource::kCell,
                        order,
                        direction,
                    );
                }
                tabulator.ProcessCell(&cell);
                cell = BlockNode::from(cell.NextSibling());
            }
            tabulator.EndRow();
            row_index = row_index.wrapping_add(1);
            row = BlockNode::from(row.NextSibling());
        }
        borders.AddSection(start, row_index.wrapping_sub(start));
    }
    borders.SetLastColumnIndex(column_count);
    let row_count = row_index;
    row_index = 0;
    if multispan {
        for (section_index, section) in sections.iter().enumerate() {
            let mut tabulator = ColspanCellTabulator::default();
            let mut row = BlockNode::from(section.FirstChild());
            while row.is_non_null() {
                tabulator.StartRow();
                let mut cell = BlockNode::from(row.FirstChild());
                while cell.is_non_null() {
                    tabulator.FindNextFreeColumn();
                    order = order.wrapping_add(1);
                    borders.MergeBorders(
                        row_index,
                        tabulator.CurrentColumn(),
                        cell.TableCellRowspan(),
                        cell.TableCellColspan(),
                        cell.Style(),
                        EdgeSource::kCell,
                        order,
                        direction,
                        section_index as u32,
                    );
                    tabulator.ProcessCell(&cell);
                    cell = BlockNode::from(cell.NextSibling());
                }
                tabulator.EndRow();
                row_index = row_index.wrapping_add(1);
                row = BlockNode::from(row.NextSibling());
            }
        }
    }
    row_index = 0;
    for section in &sections {
        let mut row = BlockNode::from(section.FirstChild());
        while row.is_non_null() {
            order = order.wrapping_add(1);
            borders.MergeBordersWithoutSection(
                row_index,
                0,
                1,
                column_count,
                row.Style(),
                EdgeSource::kRow,
                order,
                direction,
            );
            row_index = row_index.wrapping_add(1);
            row = BlockNode::from(row.NextSibling());
        }
    }
    for (index, section) in sections.iter().enumerate() {
        let info = borders.GetSection(index as u32);
        order = order.wrapping_add(1);
        borders.MergeBordersWithoutSection(
            info.start_row,
            0,
            info.row_count,
            column_count,
            section.Style(),
            EdgeSource::kSection,
            order,
            direction,
        );
    }
    order = order.wrapping_add(1);
    VisitLayoutTableColumn(
        &grouped.columns,
        column_count,
        &mut ColBordersMarker {
            rows: row_count,
            order,
            direction,
            borders,
        },
    );
    order = order.wrapping_add(1);
    VisitLayoutTableColumn(
        &grouped.columns,
        column_count,
        &mut ColgroupBordersMarker {
            rows: row_count,
            order,
            direction,
            borders,
        },
    );
    order = order.wrapping_add(1);
    borders.MergeBordersWithoutSection(
        0,
        0,
        row_count,
        column_count,
        style,
        EdgeSource::kTable,
        order,
        direction,
    );
    borders.UpdateTableBorder(row_count, column_count);
    pointer
}

// cpp: layoutng_table/table_borders.cc:248-250
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersConstructFromTable(
    border: &BoxStrut,
    collapsed: bool,
) -> TableBorders {
    TableBorders {
        edges_: HeapVector::new(),
        sections_: Vector::new(),
        edges_per_row_: 0,
        table_border_: *border,
        last_column_index_: u32::MAX,
        is_collapsed_: collapsed,
    }
}

// cpp: layoutng_table/table_borders.cc:253-292
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersDumpEdgesFromTable(
    borders: &mut TableBorders,
) -> foundation::String {
    if borders.edges_per_row_ == 0 {
        return foundation::String::from("No edges");
    }
    let mut output = String::new();
    let row_count = borders.edges_.len() / borders.edges_per_row_ as usize;
    for row in 0..row_count {
        for i in 0..borders.edges_per_row_ as usize {
            let edge = &borders.edges_[row * borders.edges_per_row_ as usize + i];
            output.push(if !edge.style.Get().is_null() {
                match edge.edge_side {
                    EdgeSide::kTop => '-',
                    EdgeSide::kBottom => '_',
                    EdgeSide::kLeft => '[',
                    EdgeSide::kRight => ']',
                    EdgeSide::kDoNotFill => '?',
                }
            } else if edge.edge_side == EdgeSide::kDoNotFill {
                'X'
            } else {
                '.'
            });
            if i & 1 != 0 {
                output.push(' ');
            }
        }
        output.push('\n');
    }
    foundation::String::from(output)
}
// cpp: layoutng_table/table_borders.cc:294-309
#[cfg(debug_assertions)]
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersEqualFromTable(left: &TableBorders, right: &TableBorders) -> bool {
    left.edges_.len() == right.edges_.len()
        && left
            .edges_
            .iter()
            .zip(&*right.edges_)
            .all(|(a, b)| a.edge_side == b.edge_side && a.box_order == b.box_order)
        && left.sections_.len() == right.sections_.len()
        && left
            .sections_
            .iter()
            .zip(&right.sections_)
            .all(|(a, b)| a.start_row == b.start_row && a.row_count == b.row_count)
        && left.edges_per_row_ == right.edges_per_row_
        && left.table_border_ == right.table_border_
        && left.last_column_index_ == right.last_column_index_
        && left.is_collapsed_ == right.is_collapsed_
}

// cpp: layoutng_table/table_borders.cc:313-366
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersGetCellBordersFromTable(
    borders: &TableBorders,
    row: u32,
    column: u32,
    rowspan: u32,
    colspan: u32,
) -> BoxStrut {
    let mut result = BoxStrut::default();
    let count = borders.edges_per_row_;
    if count == 0 {
        return result;
    }
    debug_assert_eq!(borders.edges_.len() % count as usize, 0);
    if column * 2 >= count || row as usize >= borders.edges_.len() / count as usize {
        return result;
    }
    let width = |index| {
        if borders.CanPaint(index) {
            borders.BorderWidth(index)
        } else {
            LayoutUnit::default()
        }
    };
    let first_start = row * count + column * 2;
    let first_end = first_start + colspan * 2;
    for i in 0..rowspan {
        let start = first_start + i * count;
        result.inline_start = result.inline_start.max(width(start));
        if start as usize >= borders.edges_.len() {
            break;
        }
        result.inline_end = result.inline_end.max(width(first_end + i * count));
    }
    let start_column = column * 2 + 1;
    for i in 0..colspan {
        let current = start_column + i * 2;
        if current >= count {
            break;
        }
        let start = row * count + current;
        result.block_start = result.block_start.max(width(start));
        result.block_end = result.block_end.max(width(start + rowspan * count));
    }
    debug_assert!(borders.is_collapsed_);
    result.block_start = LayoutUnit::FromRawValue(result.block_start.RawValue() / 2);
    result.block_end = LayoutUnit::FromRawValue(result.block_end.RawValue() / 2);
    result.inline_start = LayoutUnit::FromRawValue(result.inline_start.RawValue() / 2);
    result.inline_end = LayoutUnit::FromRawValue(result.inline_end.RawValue() / 2);
    result
}

// cpp: layoutng_table/table_borders.cc:368-378
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersUpdateTableBorderFromTable(
    borders: &mut TableBorders,
    rows: u32,
    columns: u32,
) {
    debug_assert!(borders.is_collapsed_);
    if borders.edges_per_row_ == 0 {
        borders.table_border_ = BoxStrut::default();
        return;
    }
    debug_assert!((columns + 1) * 2 >= borders.edges_per_row_);
    borders.table_border_ = borders.GetCellBorders(0, 0, rows, columns);
}

// cpp: layoutng_table/table_borders.cc:380-395
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersCellBorderFromTable(
    borders: &TableBorders,
    cell: &BlockNode,
    row: u32,
    column: u32,
    section: u32,
    direction: WritingDirectionMode,
) -> BoxStrut {
    if borders.is_collapsed_ {
        return borders.GetCellBorders(
            row,
            column,
            borders.ClampRowspan(section, row, cell.TableCellRowspan()),
            borders.ClampColspan(column, cell.TableCellColspan()),
        );
    }
    ComputeBorders(
        &ConstraintSpaceBuilder::new_without_parent_space(
            direction.GetWritingMode(),
            direction,
            false,
            true,
            false,
        )
        .ToConstraintSpace(),
        cell,
    )
}
// cpp: layoutng_table/table_borders.cc:399-410
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersCellPaddingForMeasureFromTable(
    _borders: &TableBorders,
    style: &ComputedStyle,
    direction: WritingDirectionMode,
) -> BoxStrut {
    if !style.MayHavePadding() {
        return BoxStrut::default();
    }
    ComputePadding(
        &ConstraintSpaceBuilder::new_without_parent_space(
            direction.GetWritingMode(),
            direction,
            false,
            true,
            false,
        )
        .ToConstraintSpace(),
        style,
    )
}

// cpp: layoutng_table/table_borders.cc:412-482
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersMergeBordersFromTable(
    borders: &mut TableBorders,
    row: u32,
    column: u32,
    rowspan: u32,
    colspan: u32,
    style: &ComputedStyle,
    source: EdgeSource,
    order: u32,
    direction: WritingDirectionMode,
    section: u32,
) {
    debug_assert!(borders.is_collapsed_);
    if rowspan == 0 || colspan == 0 {
        return;
    }
    let colspan = borders.ClampColspan(column, colspan);
    let rowspan = if source == EdgeSource::kCell {
        borders.ClampRowspan(section, row, rowspan)
    } else {
        rowspan
    };
    let inner = source == EdgeSource::kCell && (rowspan > 1 || colspan > 1);
    if inner {
        borders.EnsureCellColumnFits(column + colspan - 1);
        borders.EnsureCellRowFits(row + rowspan - 1);
    } else {
        let sides = PhysicalToLogical::new(
            direction,
            style.BorderTopStyle(),
            style.BorderRightStyle(),
            style.BorderBottomStyle(),
            style.BorderLeftStyle(),
        );
        if sides.InlineStart() == EBorderStyle::kNone
            && sides.InlineEnd() == EBorderStyle::kNone
            && sides.BlockStart() == EBorderStyle::kNone
            && sides.BlockEnd() == EBorderStyle::kNone
        {
            return;
        }
        borders.EnsureCellColumnFits(
            if sides.InlineEnd() == EBorderStyle::kNone
                && sides.BlockStart() == EBorderStyle::kNone
                && sides.BlockEnd() == EBorderStyle::kNone
            {
                column
            } else {
                column + colspan - 1
            },
        );
        borders.EnsureCellRowFits(
            if sides.InlineStart() == EBorderStyle::kNone
                && sides.InlineEnd() == EBorderStyle::kNone
                && sides.BlockEnd() == EBorderStyle::kNone
            {
                row
            } else {
                row + rowspan - 1
            },
        );
    }
    let sides = PhysicalToLogical::new(
        direction,
        EdgeSide::kTop,
        EdgeSide::kRight,
        EdgeSide::kBottom,
        EdgeSide::kLeft,
    );
    borders.MergeRowAxisBorder(row, column, colspan, style, order, sides.BlockStart());
    borders.MergeRowAxisBorder(
        row + rowspan,
        column,
        colspan,
        style,
        order,
        sides.BlockEnd(),
    );
    borders.MergeColumnAxisBorder(row, column, rowspan, style, order, sides.InlineStart());
    borders.MergeColumnAxisBorder(
        row,
        column + colspan,
        rowspan,
        style,
        order,
        sides.InlineEnd(),
    );
    if inner {
        borders.MarkInnerBordersAsDoNotFill(row, column, rowspan, colspan);
    }
}

// cpp: layoutng_table/table_borders.cc:484-508
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersMergeRowAxisBorderFromTable(
    borders: &mut TableBorders,
    row: u32,
    column: u32,
    colspan: u32,
    style: &ComputedStyle,
    order: u32,
    side: EdgeSide,
) {
    let border_style = TableBorders::BorderStyleForStyle(style, side);
    if border_style == EBorderStyle::kNone {
        return;
    }
    let width = TableBorders::BorderWidthForStyle(style, side);
    let start = borders.edges_per_row_ * row + column * 2 + 1;
    let end = start + colspan * 2;
    for index in (start..end).step_by(2) {
        let edge = &mut borders.edges_[index as usize];
        if IsSourceMoreSpecificThanEdge(border_style, width, edge) {
            edge.style = Member::from_ptr(style as *const _ as *mut _);
            edge.edge_side = side;
            edge.box_order = order;
        }
    }
}
// cpp: layoutng_table/table_borders.cc:510-534
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersMergeColumnAxisBorderFromTable(
    borders: &mut TableBorders,
    row: u32,
    column: u32,
    rowspan: u32,
    style: &ComputedStyle,
    order: u32,
    side: EdgeSide,
) {
    let border_style = TableBorders::BorderStyleForStyle(style, side);
    if border_style == EBorderStyle::kNone {
        return;
    }
    let width = TableBorders::BorderWidthForStyle(style, side);
    let start = borders.edges_per_row_ * row + column * 2;
    let end = start + rowspan * borders.edges_per_row_;
    for index in (start..end).step_by(borders.edges_per_row_ as usize) {
        let edge = &mut borders.edges_[index as usize];
        if IsSourceMoreSpecificThanEdge(border_style, width, edge) {
            edge.style = Member::from_ptr(style as *const _ as *mut _);
            edge.edge_side = side;
            edge.box_order = order;
        }
    }
}

// cpp: layoutng_table/table_borders.cc:538-569
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersMarkInnerBordersAsDoNotFillFromTable(
    borders: &mut TableBorders,
    row: u32,
    column: u32,
    rowspan: u32,
    colspan: u32,
) {
    let start = column * 2 + 2;
    let end = start + (colspan - 1) * 2;
    if start != end {
        for r in row..row + rowspan {
            let offset = r * borders.edges_per_row_;
            for index in (offset + start..offset + end).step_by(2) {
                let edge = &mut borders.edges_[index as usize];
                if edge.style.Get().is_null() {
                    edge.edge_side = EdgeSide::kDoNotFill;
                }
            }
        }
    }
    let start = column * 2 + 1;
    let end = start + colspan * 2;
    for r in row + 1..row + rowspan {
        let offset = r * borders.edges_per_row_;
        for index in (offset + start..offset + end).step_by(2) {
            let edge = &mut borders.edges_[index as usize];
            if edge.style.Get().is_null() {
                edge.edge_side = EdgeSide::kDoNotFill;
            }
        }
    }
}
fn empty_edge() -> Edge {
    Edge {
        style: Member::default(),
        edge_side: EdgeSide::kTop,
        box_order: 0,
    }
}

// cpp: layoutng_table/table_borders.cc:572-600
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersEnsureCellColumnFitsFromTable(
    borders: &mut TableBorders,
    column: u32,
) {
    let desired = (column + 2) * 2;
    if desired <= borders.edges_per_row_ {
        return;
    }
    let rows = if borders.edges_per_row_ == 0 {
        1
    } else {
        borders.edges_.len() / borders.edges_per_row_ as usize
    };
    borders
        .edges_
        .resize_with(rows * desired as usize, empty_edge);
    for row in (1..rows).rev() {
        for edge in (0..desired).rev() {
            let new = row * desired as usize + edge as usize;
            if edge < borders.edges_per_row_ {
                let old = row * borders.edges_per_row_ as usize + edge as usize;
                let old = &borders.edges_[old];
                let copy = Edge {
                    style: old.style.clone(),
                    edge_side: old.edge_side,
                    box_order: old.box_order,
                };
                borders.edges_[new] = copy;
            } else {
                borders.edges_[new].style = Member::default();
                borders.edges_[new].edge_side = EdgeSide::kTop;
            }
        }
    }
    for edge in borders.edges_per_row_..desired {
        borders.edges_[edge as usize].style = Member::default();
        borders.edges_[edge as usize].edge_side = EdgeSide::kTop;
    }
    borders.edges_per_row_ = desired;
}
// cpp: layoutng_table/table_borders.cc:604-611
#[unsafe(no_mangle)]
pub extern "Rust" fn TableBordersEnsureCellRowFitsFromTable(borders: &mut TableBorders, row: u32) {
    debug_assert_ne!(borders.edges_per_row_, 0);
    let count = borders.edges_.len() / borders.edges_per_row_ as usize;
    let desired = row + 2;
    if desired as usize <= count {
        return;
    }
    borders.edges_.resize_with(
        desired as usize * borders.edges_per_row_ as usize,
        empty_edge,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{LayoutHeapScope, TextDirection, WritingMode};
    use layoutng_style::style::computed_style::ComputedStyleBuilder;
    fn style(v: u32) -> EBorderStyle {
        match v {
            0 => EBorderStyle::kNone,
            1 => EBorderStyle::kHidden,
            2 => EBorderStyle::kInset,
            3 => EBorderStyle::kGroove,
            4 => EBorderStyle::kOutset,
            5 => EBorderStyle::kRidge,
            6 => EBorderStyle::kDotted,
            7 => EBorderStyle::kDashed,
            8 => EBorderStyle::kSolid,
            9 => EBorderStyle::kDouble,
            _ => panic!("frozen border style"),
        }
    }
    fn writing_mode(v: usize) -> WritingMode {
        match v {
            0 => WritingMode::kHorizontalTb,
            1 => WritingMode::kVerticalRl,
            2 => WritingMode::kVerticalLr,
            3 => WritingMode::kSidewaysRl,
            4 => WritingMode::kSidewaysLr,
            _ => panic!("frozen writing mode"),
        }
    }
    fn dump_box(out: &mut Vec<f64>, b: &BoxStrut) {
        for value in [b.inline_start, b.inline_end, b.block_start, b.block_end] {
            out.push(value.ToDouble());
        }
    }
    #[test]
    fn collapsed_border_specificity_grid_growth_and_spans_match_cpp() {
        crate::native_test_thread::run(border_grid_body);
    }
    fn border_grid_body() {
        let _heap = LayoutHeapScope::new();
        let inputs = include_str!("../../../artifacts/cpp-reference/table-border-grid-input.tsv")
            .lines()
            .collect::<Vec<_>>();
        let expected =
            include_str!("../../../artifacts/cpp-reference/table-border-grid-results.tsv")
                .lines()
                .collect::<Vec<_>>();
        assert_eq!(inputs.len(), 1000);
        assert_eq!(expected.len(), inputs.len());
        for (input, expected) in inputs.iter().zip(expected) {
            let fields = input.split('\t').collect::<Vec<_>>();
            let rows = fields[1].parse::<u32>().unwrap();
            let columns = fields[2].parse::<u32>().unwrap();
            let direction = WritingDirectionMode::new(
                writing_mode(fields[3].parse().unwrap()),
                if fields[4] == "0" {
                    TextDirection::kLtr
                } else {
                    TextDirection::kRtl
                },
            );
            let mut borders = TableBorders::new(&BoxStrut::default(), true);
            borders.AddSection(0, rows);
            borders.SetLastColumnIndex(columns);
            let mut styles = Vec::new();
            for operation in fields[5].split('|') {
                let p = operation
                    .split(',')
                    .map(|s| s.parse::<u32>().unwrap())
                    .collect::<Vec<_>>();
                let mut builder = ComputedStyleBuilder::from_style(unsafe {
                    &*ComputedStyle::GetInitialStyleSingleton()
                });
                builder.SetBorderTopStyle(style(p[6]));
                builder.SetBorderRightStyle(style(p[7]));
                builder.SetBorderBottomStyle(style(p[8]));
                builder.SetBorderLeftStyle(style(p[9]));
                builder.SetBorderTopWidthOwned(p[10] as i32);
                builder.SetBorderRightWidthOwned(p[11] as i32);
                builder.SetBorderBottomWidthOwned(p[12] as i32);
                builder.SetBorderLeftWidthOwned(p[13] as i32);
                let style = builder.TakeStyle();
                styles.push(style);
                let source = match p[4] {
                    0 => EdgeSource::kNone,
                    1 => EdgeSource::kCell,
                    2 => EdgeSource::kRow,
                    3 => EdgeSource::kSection,
                    4 => EdgeSource::kColumn,
                    5 => EdgeSource::kTable,
                    _ => panic!("frozen source"),
                };
                borders.MergeBorders(
                    p[0],
                    p[1],
                    p[2],
                    p[3],
                    unsafe { &*style },
                    source,
                    p[5],
                    direction,
                    0,
                );
            }
            borders.UpdateTableBorder(rows, columns);
            let mut out = vec![borders.EdgesPerRow() as f64, borders.EdgeCount() as f64];
            dump_box(&mut out, borders.TableBorder());
            for (index, edge) in borders.edges_.iter().enumerate() {
                out.extend([
                    styles
                        .iter()
                        .position(|&s| s == edge.style.Get())
                        .map_or(0, |i| i + 1) as f64,
                    edge.edge_side as i32 as f64,
                    edge.box_order as f64,
                    borders.BorderWidth(index as u32).ToDouble(),
                    borders.BorderStyle(index as u32) as i32 as f64,
                    borders.CanPaint(index as u32) as u8 as f64,
                ]);
            }
            for row in 0..rows {
                for col in 0..columns {
                    dump_box(&mut out, &borders.GetCellBorders(row, col, 1, 1));
                }
            }
            let mut expected = expected.split('\t');
            assert_eq!(expected.next().unwrap(), fields[0]);
            let expected = expected
                .map(|s| s.parse::<f64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(out, expected, "border case {}", fields[0]);
        }
    }
}

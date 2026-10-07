#![allow(non_snake_case)]
use crate::table_layout_utils::{
    ColspanCellTabulator, DistributeRowspanCellToRows, DistributeSectionFixedBlockSizeToRows,
    RowBaselineTabulator, SetupTableCellConstraintSpaceBuilder,
};
use foundation::{
    kIndefiniteSize, EBoxSizing, EVisibility, IsParallelWritingMode, LayoutUnit, To, Vector,
};
use layoutng_assembly::{
    box_fragment_builder::BoxFragmentBuilder,
    internal::{
        block_node::BlockNode,
        constraint_space::LayoutResultCacheSlot,
        constraint_space_builder::ConstraintSpaceBuilder,
        fragmentation_utils::IsBreakInside,
        layout_alignment_utils::{BlockContentAlignment, ComputeContentAlignmentForTableCell},
        length_utils::ComputePadding,
        table_borders::TableBorders,
        table_column_location::TableColumnLocation,
        table_layout_algorithm_types::{
            CellBlockConstraint, CellBlockConstraints, Row, Rows, RowspanCell, RowspanCells,
            Sections, TableTypes,
        },
    },
    logical_box_fragment::LogicalBoxFragment,
    physical_box_fragment::PhysicalBoxFragment,
};

// cpp: layoutng_table/table_layout_utils.cc:171-333
fn ComputeMinimumRowBlockSize(
    row_count: &mut impl FnMut() -> u32,
    row: &BlockNode,
    percentage_inline_size: LayoutUnit,
    table_block_size_specified: bool,
    columns: &Vector<TableColumnLocation>,
    borders: &TableBorders,
    start_row: u32,
    row_index: u32,
    section_index: u32,
    section_collapsed: bool,
    cells: &mut CellBlockConstraints,
    rowspans: &mut RowspanCells,
    tabulator: &mut ColspanCellTabulator,
) -> Row {
    let direction = row.Style().GetWritingDirection();
    let collapsed = borders.IsCollapsed();
    let mut max_cell_block_size = LayoutUnit::default();
    let mut row_percent: Option<f32> = None;
    let mut constrained = false;
    let mut has_rowspan_start = false;
    let start_cell = cells.len() as u32;
    let mut baselines = RowBaselineTabulator::default();
    let mut cell = BlockNode::from(row.FirstChild());
    while cell.is_non_null() {
        tabulator.FindNextFreeColumn();
        let style = cell.Style();
        let cell_direction = style.GetWritingDirection();
        let cell_borders = borders.CellBorder(
            &cell,
            row_index,
            tabulator.CurrentColumn(),
            section_index,
            direction,
        );
        let mut rowspan = cell.TableCellRowspan();
        if rowspan > 1 {
            rowspan = std::cmp::min(row_count() - (row_index - start_row), rowspan);
        }
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            direction.GetWritingMode(),
            cell_direction,
            true,
            true,
            false,
        );
        SetupTableCellConstraintSpaceBuilder(
            direction,
            &cell,
            &cell_borders,
            columns,
            kIndefiniteSize,
            percentage_inline_size,
            None,
            tabulator.CurrentColumn(),
            true,
            table_block_size_specified,
            collapsed,
            LayoutResultCacheSlot::kMeasure,
            &mut builder,
        );
        let space = builder.ToConstraintSpace();
        let result =
            unsafe { &*cell.Layout(&space, std::ptr::null(), std::ptr::null(), std::ptr::null()) };
        let fragment = LogicalBoxFragment::new(direction, unsafe {
            &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment())
        });
        let specified = if IsParallelWritingMode(direction.GetWritingMode(), style.GetWritingMode())
        {
            style.LogicalHeight()
        } else {
            style.LogicalWidth()
        };
        let percentage_descendant = result.HasDescendantThatDependsOnPercentageBlockSize();
        let effective_rowspan = rowspan > 1;
        let constraint = CellBlockConstraint::new(
            fragment.BlockSize(),
            cell_borders,
            tabulator.CurrentColumn(),
            rowspan,
            specified.IsFixed(),
            percentage_descendant,
        );
        tabulator.ProcessCell(&cell);
        cells.push(constraint);
        constrained |= constraint.is_constrained && !effective_rowspan;
        baselines.ProcessCell(
            &fragment,
            ComputeContentAlignmentForTableCell(style),
            effective_rowspan,
            percentage_descendant,
        );
        let mut css_size = None;
        let mut css_percent = None;
        if specified.IsPercent() {
            css_percent = Some(specified.PercentValue());
        } else if specified.IsFixed() {
            let border_padding = (cell_borders + ComputePadding(&space, style)).BlockSum();
            let size = LayoutUnit::from_f32(specified.Pixels());
            css_size = Some(
                if unsafe { &*cell.GetLayoutBox() }.InQuirksModeForLayout()
                    || style.BoxSizing() == EBoxSizing::kBorderBox
                {
                    std::cmp::max(border_padding, size)
                } else {
                    border_padding + size
                },
            );
        }
        if !effective_rowspan {
            if css_size.is_some() || css_percent.is_some() {
                constrained = true;
            }
            if let Some(p) = css_percent {
                row_percent = Some(row_percent.unwrap_or(0.0).max(p));
            }
            max_cell_block_size = max_cell_block_size
                .max(constraint.min_block_size)
                .max(css_size.unwrap_or_default());
        } else {
            has_rowspan_start = true;
            let mut min_size = constraint.min_block_size;
            if let Some(size) = css_size {
                min_size = min_size.max(size);
            }
            rowspans.push(RowspanCell::new(row_index, rowspan, min_size));
        }
        cell = BlockNode::from(cell.NextSibling());
    }
    let specified = row.Style().LogicalHeight();
    if specified.IsPercent() {
        constrained = true;
        row_percent = Some(row_percent.unwrap_or(0.0).max(specified.PercentValue()));
    } else if specified.IsFixed() {
        constrained = true;
        max_cell_block_size = max_cell_block_size.max(LayoutUnit::from_f32(specified.Pixels()));
    }
    let size = baselines.ComputeRowBlockSize(max_cell_block_size);
    let baseline = if !baselines.BaselineDependsOnPercentageBlockDescendant() {
        Some(baselines.ComputeBaseline(size))
    } else {
        None
    };
    Row {
        block_size: size,
        start_cell_index: start_cell,
        cell_count: cells.len() as u32 - start_cell,
        baseline,
        percent: row_percent,
        is_constrained: constrained,
        has_rowspan_start,
        is_collapsed: section_collapsed || row.Style().Visibility() == EVisibility::kCollapse,
    }
}

// cpp: layoutng_table/table_layout_utils.h:79-91
// cpp: layoutng_table/table_layout_utils.cc:1487-1573
pub fn ComputeSectionMinimumRowBlockSizes(
    section: &BlockNode,
    percentage_inline_size: LayoutUnit,
    table_block_size_specified: bool,
    columns: &Vector<TableColumnLocation>,
    borders: &TableBorders,
    spacing: LayoutUnit,
    section_index: u32,
    treat_as_tbody: bool,
    sections: &mut Sections,
    rows: &mut Rows,
    cells: &mut CellBlockConstraints,
) {
    let mut count = None;
    let mut row_count = || {
        if count.is_none() {
            let mut c = 0;
            let mut row = BlockNode::from(section.FirstChild());
            while row.is_non_null() {
                c += 1;
                row = BlockNode::from(row.NextSibling());
            }
            count = Some(c);
        }
        count.unwrap()
    };
    let start = rows.len() as u32;
    let mut current = start;
    let mut rowspans = Vector::new();
    let mut size = LayoutUnit::default();
    let mut tabulator = ColspanCellTabulator::default();
    let mut total_percent = 0.0_f32;
    let mut row = BlockNode::from(section.FirstChild());
    while row.is_non_null() {
        tabulator.StartRow();
        let mut constraint = ComputeMinimumRowBlockSize(
            &mut row_count,
            &row,
            percentage_inline_size,
            table_block_size_specified,
            columns,
            borders,
            start,
            current,
            section_index,
            section.Style().Visibility() == EVisibility::kCollapse,
            cells,
            &mut rowspans,
            &mut tabulator,
        );
        current += 1;
        if let Some(percent) = constraint.percent {
            let percent = (100.0 - total_percent).min(percent);
            constraint.percent = Some(percent);
            total_percent += percent;
        }
        rows.push(constraint);
        size += constraint.block_size;
        tabulator.EndRow();
        row = BlockNode::from(row.NextSibling());
    }
    rowspans.sort_by(|a, b| {
        if a.LessThan(b) {
            std::cmp::Ordering::Less
        } else if b.LessThan(a) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    for span in &rowspans {
        DistributeRowspanCellToRows(span, spacing, rows);
    }
    let spacing_count = if current == start {
        0
    } else {
        current - start - 1
    };
    size += spacing * spacing_count;
    let specified = section.Style().LogicalHeight();
    if specified.IsFixed() {
        let fixed = LayoutUnit::from_f32(specified.Pixels());
        if fixed > size {
            DistributeSectionFixedBlockSizeToRows(
                start,
                current - start,
                fixed,
                spacing,
                fixed,
                rows,
            );
            size = fixed;
        }
    }
    sections.push(TableTypes::CreateSection(
        section,
        start,
        current - start,
        size,
        treat_as_tbody,
    ));
}

// cpp: layoutng_table/table_layout_utils.h:94-95
// cpp: layoutng_table/table_layout_utils.cc:1575-1640
pub fn FinalizeTableCellLayout(intrinsic_block_size: LayoutUnit, builder: &mut BoxFragmentBuilder) {
    let node = builder.Node();
    let space = builder.GetConstraintSpace();
    let has_inflow_children = !builder.Children().is_empty();
    if !space.IsHiddenForPaint() {
        let hide = space.HideTableCellIfEmpty() && !has_inflow_children;
        builder.SetIsHiddenForPaint(hide);
    }
    let space = builder.GetConstraintSpace();
    let collapsed = space.IsTableCellWithCollapsedBorders();
    let column_index = space.TableCellColumnIndex();
    builder.SetHasCollapsedBorders(collapsed);
    builder.SetIsTablePart();
    builder.SetTableCellColumnIndex(column_index);
    if IsBreakInside(builder.PreviousBreakToken()) {
        return;
    }
    let mut free = builder.FragmentBlockSize() - intrinsic_block_size;
    let alignment = ComputeContentAlignmentForTableCell(builder.Style());
    if alignment == BlockContentAlignment::kSafeCenter
        || alignment == BlockContentAlignment::kSafeEnd
    {
        free = free.ClampNegativeToZero();
    }
    match alignment {
        BlockContentAlignment::kStart => {}
        BlockContentAlignment::kBaseline => {
            if builder.FirstBaseline().is_none() || node.ShouldApplyLayoutContainment() {
                builder.SetBaselines(
                    intrinsic_block_size - builder.BorderScrollbarPadding().block_end,
                );
            }
            if has_inflow_children {
                if let Some(baseline) = builder.GetConstraintSpace().TableCellAlignmentBaseline() {
                    builder.MoveChildrenInDirection(
                        baseline - builder.FirstBaseline().unwrap(),
                        true,
                        None,
                    );
                }
            }
        }
        BlockContentAlignment::kSafeCenter | BlockContentAlignment::kUnsafeCenter => {
            builder.MoveChildrenInDirection(free / 2, true, None)
        }
        BlockContentAlignment::kSafeEnd | BlockContentAlignment::kUnsafeEnd => {
            builder.MoveChildrenInDirection(free, true, None)
        }
    }
}

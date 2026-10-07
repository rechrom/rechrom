#![allow(non_snake_case)]
use crate::{
    layout_table_column::LayoutTableColumn,
    layout_table_column_visitor::ColumnVisitor,
    table_layout_utils::{
        ComputeCellBlockSize, ComputeGridInlineMinMax,
        SynchronizeAssignableTableInlineSizeAndColumns,
    },
};
use foundation::{
    kIndefiniteSize, EBorderCollapse, HeapVector, LayoutUnit, Member, To, Traceable, Vector,
    Visitor,
};
use layoutng_assembly::{
    block_break_token::BlockBreakToken,
    box_fragment_builder::BoxFragmentBuilder,
    internal::{
        block_node::BlockNode,
        constraint_space::{AutoSizeBehavior, ConstraintSpace},
        constraint_space_builder::ConstraintSpaceBuilder,
        constraint_space_builder_style::MinMaxConstraintSpaceBuilder,
        disable_layout_side_effects_scope::DisableLayoutSideEffectsScope,
        early_break::EarlyBreak,
        fragmentation_utils::{
            AdjustMarginsForFragmentation, InvolvedInBlockFragmentationForBuilder,
            SetupFragmentBuilderForFragmentation, SetupSpaceBuilderForFragmentationFromSpace,
        },
        layout_input_node::MinMaxSizesFloatInput,
        length_utils::{
            ComputeMarginsFor, ComputeMarginsForInlineSize, ComputeMinAndMaxContentContribution,
            ComputeUsedInlineSizeForTableFragment, ResolveInlineAutoMargins,
        },
        min_max_sizes::MinMaxSizes,
        space_utils::SetOrthogonalFallbackInlineSizeIfNeeded,
        table_column_location::TableColumnLocation,
        table_constraint_space_data::{self, TableConstraintSpaceData},
        table_fragment_data::{TableColumnGeometries, TableColumnGeometry},
        table_layout_algorithm_types::{
            CellBlockConstraints, Columns, Rows, Sections, TableGroupedChildren,
        },
        table_node::TableNode,
    },
    layout_result::{EStatus, LayoutResult},
    logical_fragment::LogicalFragment,
};
use layoutng_geometry::geometry::{
    box_strut::BoxStrut, fragment_geometry::FragmentGeometry, logical_size::LogicalSize,
};
use layoutng_style::style::computed_style::ComputedStyle;
use std::sync::Arc;
// cpp: layoutng_table/table_layout_algorithm.h:21-33
pub struct TableCaptionResult {
    pub node: BlockNode,
    pub layout_result: Member<LayoutResult>,
    pub margins: BoxStrut,
}
impl Traceable for TableCaptionResult {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.node);
        visitor.Trace(&self.layout_result);
    }
}
// cpp: layoutng_table/table_layout_algorithm.cc:40-61
pub(crate) fn ComputeCaptionConstraint(
    space: &ConstraintSpace,
    style: &ComputedStyle,
    grouped: &TableGroupedChildren,
) -> MinMaxSizes {
    let mut sizes = MinMaxSizes::default();
    for caption in &grouped.captions {
        let mut builder = MinMaxConstraintSpaceBuilder::new(space, style, &caption.base, true);
        builder.SetAvailableBlockSize(kIndefiniteSize);
        let caption_space = builder.ToConstraintSpace();
        let mut contribution = ComputeMinAndMaxContentContribution(
            style,
            caption,
            &caption_space,
            MinMaxSizesFloatInput::default(),
        )
        .sizes;
        contribution += ComputeMarginsFor(&caption_space, caption.Style(), space).InlineSum();
        sizes.Encompass(&contribution);
    }
    sizes
}
// cpp: layoutng_table/table_layout_algorithm.cc:63-92
pub(crate) fn CreateCaptionConstraintSpace(
    space: &ConstraintSpace,
    style: &ComputedStyle,
    caption: &BlockNode,
    available: LogicalSize,
    offset: Option<LayoutUnit>,
) -> ConstraintSpace {
    let mut builder =
        ConstraintSpaceBuilder::new(space, caption.Style().GetWritingDirection(), true);
    SetOrthogonalFallbackInlineSizeIfNeeded(style, caption.base.clone(), &mut builder);
    builder.SetAvailableSize(available);
    builder.SetPercentageResolutionSize(available);
    builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
    if let Some(offset) = offset {
        if space.HasBlockFragmentation() {
            SetupSpaceBuilderForFragmentationFromSpace(
                space,
                &caption.base,
                space.FragmentainerOffset() + offset,
                space.FragmentainerBlockSize(),
                false,
                &mut builder,
            );
        }
    }
    builder.ToConstraintSpace()
}
// cpp: layoutng_table/table_layout_algorithm.cc:94-118
pub(crate) fn LayoutCaption(
    space: &ConstraintSpace,
    style: &ComputedStyle,
    inline_size: LayoutUnit,
    caption_space: &ConstraintSpace,
    caption: &BlockNode,
    mut margins: BoxStrut,
    token: *const BlockBreakToken,
    early: *const EarlyBreak,
) -> TableCaptionResult {
    let result = caption.Layout(caption_space, token, early, std::ptr::null());
    debug_assert_eq!(unsafe { &*result }.Status(), EStatus::kSuccess);
    let fragment = LogicalFragment::new(
        space.GetWritingDirection(),
        unsafe { &*result }.GetPhysicalFragment(),
    );
    ResolveInlineAutoMargins(
        caption.Style(),
        style,
        inline_size,
        fragment.InlineSize(),
        &mut margins,
    );
    TableCaptionResult {
        node: caption.clone(),
        layout_result: Member::from_ptr(result.cast_mut()),
        margins,
    }
}
// cpp: layoutng_table/table_layout_algorithm.cc:120-130
pub(crate) fn ComputeCaptionMargins(
    space: &ConstraintSpace,
    caption: &BlockNode,
    inline_size: LayoutUnit,
    token: *const BlockBreakToken,
) -> BoxStrut {
    let mut margins =
        ComputeMarginsForInlineSize(caption.Style(), inline_size, space.GetWritingDirection());
    AdjustMarginsForFragmentation(token, &mut margins);
    margins
}
// cpp: layoutng_table/table_layout_algorithm.cc:132-173
pub(crate) fn ComputeCaptionFragments(
    builder: &BoxFragmentBuilder,
    style: &ComputedStyle,
    grouped: &TableGroupedChildren,
    mut captions: Option<&mut HeapVector<TableCaptionResult>>,
    block_size: &mut LayoutUnit,
) {
    let space = builder.GetConstraintSpace();
    let inline_size = builder.InlineSize();
    let available = LogicalSize::new(inline_size, kIndefiniteSize);
    for caption in &grouped.captions {
        let margins = ComputeCaptionMargins(space, caption, inline_size, std::ptr::null());
        let caption_space = CreateCaptionConstraintSpace(space, style, caption, available, None);
        let _disable = if (captions.is_none() && !unsafe { &*caption.GetLayoutBox() }.NeedsLayout())
            || InvolvedInBlockFragmentationForBuilder(builder)
        {
            Some(DisableLayoutSideEffectsScope::new())
        } else {
            None
        };
        let result = LayoutCaption(
            space,
            style,
            inline_size,
            &caption_space,
            caption,
            margins,
            std::ptr::null(),
            std::ptr::null(),
        );
        let fragment = LogicalFragment::new(
            space.GetWritingDirection(),
            unsafe { &*result.layout_result.Get() }.GetPhysicalFragment(),
        );
        *block_size += fragment.BlockSize() + result.margins.BlockSum();
        if let Some(captions) = captions.as_mut() {
            captions.push(result);
        }
    }
}
// cpp: layoutng_table/table_layout_algorithm.cc:175-194
pub(crate) fn ComputeUndistributableTableSpace(
    columns: &Columns,
    border_padding: LayoutUnit,
    spacing: LayoutUnit,
) -> LayoutUnit {
    let mut count = 2;
    let mut first = true;
    for column in &columns.data {
        if !column.is_mergeable {
            if first {
                first = false;
            } else {
                count += 1;
            }
        }
    }
    border_padding + spacing * count
}
// cpp: layoutng_table/table_layout_algorithm.cc:196-224
pub(crate) fn ComputeEmptyTableInlineSize(
    space: &ConstraintSpace,
    style: &ComputedStyle,
    assignable: LayoutUnit,
    undistributable: LayoutUnit,
    caption: &MinMaxSizes,
    border_padding: &BoxStrut,
    collapsed: bool,
) -> LayoutUnit {
    if space.IsFixedInlineSize()
        || space.IsInlineAutoBehaviorStretch()
        || !style.LogicalWidth().IsAuto()
        || !style.LogicalMinWidth().IsAuto()
    {
        return assignable + undistributable;
    }
    if caption.min_size != LayoutUnit::default() {
        return caption.min_size.max(border_padding.InlineSum());
    }
    if collapsed {
        return LayoutUnit::default();
    }
    assignable + border_padding.InlineSum()
}
// cpp: layoutng_table/table_layout_algorithm.cc:226-263
pub(crate) fn ComputeAssignableTableInlineSize(
    table: &TableNode,
    space: &ConstraintSpace,
    columns: &Columns,
    caption: &MinMaxSizes,
    undistributable: LayoutUnit,
    border_padding: &BoxStrut,
    fixed: bool,
) -> LayoutUnit {
    if space.IsFixedInlineSize() {
        return (space.AvailableSize().inline_size - undistributable).ClampNegativeToZero();
    }
    let grid = ComputeGridInlineMinMax(table, columns, undistributable, fixed, true);
    let used = ComputeUsedInlineSizeForTableFragment(space, table, border_padding, &grid);
    debug_assert!(used >= grid.min_size);
    let assignable = used.max(caption.min_size) - undistributable;
    debug_assert!(assignable >= LayoutUnit::default());
    assignable
}
// cpp: layoutng_table/table_layout_algorithm.cc:265-304
pub(crate) fn ComputeLocationsFromColumns(
    columns: &Columns,
    sizes: &Vector<LayoutUnit>,
    spacing: LayoutUnit,
    shrink: bool,
    locations: &mut Vector<TableColumnLocation>,
) -> bool {
    let mut collapsed = false;
    locations.resize(
        columns.data.len(),
        TableColumnLocation {
            offset: LayoutUnit::default(),
            size: LayoutUnit::default(),
            is_collapsed: false,
        },
    );
    if locations.is_empty() {
        return collapsed;
    }
    let mut first = true;
    let mut offset = spacing;
    for (i, column) in columns.data.iter().enumerate() {
        collapsed |= column.is_collapsed;
        let location = &mut locations[i];
        if (column.is_mergeable
            && (sizes[i] == kIndefiniteSize || sizes[i] == LayoutUnit::default()))
            || (shrink && column.is_collapsed)
        {
            location.offset = offset;
            location.size = LayoutUnit::default();
            location.is_collapsed = true;
        } else {
            if first {
                first = false;
            } else {
                offset += spacing;
            }
            location.offset = offset;
            location.size = if sizes[i] != kIndefiniteSize {
                sizes[i]
            } else {
                LayoutUnit::default()
            };
            location.is_collapsed = false;
            offset += location.size;
        }
    }
    collapsed
}
// cpp: layoutng_table/table_layout_algorithm.cc:306-365
pub(crate) fn CreateConstraintSpaceData(
    style: &ComputedStyle,
    locations: &Vector<TableColumnLocation>,
    sections: &Sections,
    rows: &Rows,
    cells: &CellBlockConstraints,
    spacing: LogicalSize,
) -> Arc<TableConstraintSpaceData> {
    let specified = !style.LogicalHeight().HasAuto();
    let mut data = TableConstraintSpaceData {
        table_writing_direction: style.GetWritingDirection(),
        table_border_spacing: spacing,
        is_table_block_size_specified: specified,
        has_collapsed_borders: style.BorderCollapse() == EBorderCollapse::kCollapse,
        column_locations: locations.clone(),
        ..Default::default()
    };
    data.sections.reserve(sections.len());
    for s in sections {
        data.sections
            .push(table_constraint_space_data::Section::new(
                s.start_row,
                s.row_count,
            ));
    }
    data.rows.reserve(rows.len());
    for r in rows {
        data.rows.push(table_constraint_space_data::Row::new(
            r.block_size,
            r.start_cell_index,
            r.cell_count,
            r.baseline,
            r.is_collapsed,
        ));
    }
    data.cells.reserve(cells.len());
    for section in sections {
        for row_index in section.start_row..section.start_row + section.row_count {
            let row = &rows[row_index as usize];
            for cell_index in row.start_cell_index..row.start_cell_index + row.cell_count {
                let cell = &cells[cell_index as usize];
                let size = ComputeCellBlockSize(cell, rows, row_index, &spacing, specified);
                let rowspan_size = if cell.effective_rowspan > 1 {
                    size.block_size
                } else {
                    kIndefiniteSize
                };
                data.cells.push(table_constraint_space_data::Cell::new(
                    cell.borders,
                    rowspan_size,
                    cell.column_index,
                    size.is_initial_block_size_indefinite,
                    cell.has_descendant_that_depends_on_percentage_block_size,
                ));
            }
        }
    }
    Arc::new(data)
}
// cpp: layoutng_table/table_layout_algorithm.cc:367-447
pub(crate) struct ColumnGeometriesBuilder<'a> {
    pub geometries: TableColumnGeometries,
    pub locations: &'a Vector<TableColumnLocation>,
    pub table_column_block_size: LayoutUnit,
}
impl ColumnVisitor for ColumnGeometriesBuilder<'_> {
    fn VisitCol(&mut self, column: &BlockNode, start: u32, span: u32) {
        let end = start + span - 1;
        debug_assert!((end as usize) < self.locations.len());
        let size = self.locations[end as usize].offset + self.locations[end as usize].size
            - self.locations[start as usize].offset;
        self.geometries.push(TableColumnGeometry::new(
            start,
            span,
            self.locations[start as usize].offset - self.locations[0].offset,
            size,
            column.base.clone(),
        ));
    }
    fn EnterColgroup(&mut self, _column: &BlockNode, _start: u32) {}
    fn LeaveColgroup(&mut self, column: &BlockNode, start: u32, span: u32, _has_children: bool) {
        if span == 0 {
            return;
        }
        let end = start + span - 1;
        let size = self.locations[end as usize].offset + self.locations[end as usize].size
            - self.locations[start as usize].offset;
        self.geometries.push(TableColumnGeometry::new(
            start,
            span,
            self.locations[start as usize].offset - self.locations[0].offset,
            size,
            column.base.clone(),
        ));
    }
}
impl ColumnGeometriesBuilder<'_> {
    pub fn Sort(&mut self) {
        fn less(a: &TableColumnGeometry, b: &TableColumnGeometry) -> bool {
            if a.node.IsTableCol() && b.node.IsTableCol() {
                return a.start_column < b.start_column;
            }
            if a.node.IsTableColgroup() {
                if b.node.IsTableColgroup() {
                    return a.start_column < b.start_column;
                }
                if a.start_column <= b.start_column && a.start_column + a.span > b.start_column {
                    return true;
                }
                a.start_column < b.start_column
            } else {
                debug_assert!(b.node.IsTableColgroup());
                if b.start_column <= a.start_column && b.start_column + b.span > a.start_column {
                    return false;
                }
                b.start_column >= a.start_column
            }
        }
        self.geometries.sort_unstable_by(|a, b| {
            if less(a, b) {
                std::cmp::Ordering::Less
            } else if less(b, a) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        for (index, column) in self.geometries.iter().enumerate() {
            unsafe { &mut *To::<LayoutTableColumn>(column.node.GetLayoutBox()) }
                .SetColumnIndex(index);
        }
    }
}
// cpp: layoutng_table/table_layout_algorithm.cc:449-455
pub(crate) fn ComputeTableSizeFromColumns(
    locations: &Vector<TableColumnLocation>,
    border_padding: &BoxStrut,
    spacing: LogicalSize,
) -> LayoutUnit {
    let last = locations.last().unwrap();
    last.offset + last.size + border_padding.InlineSum() + spacing.inline_size
}
// cpp: layoutng_table/table_layout_algorithm.cc:461-464
#[derive(Clone, Copy)]
pub(crate) struct TableBoxExtent {
    pub start: LayoutUnit,
    pub end: LayoutUnit,
}
// cpp: layoutng_table/table_layout_algorithm.cc:468-473
pub(crate) fn BeginTableBoxLayout(start: LayoutUnit, border_start: LayoutUnit) -> TableBoxExtent {
    TableBoxExtent {
        start,
        end: start + border_start,
    }
}
// cpp: layoutng_table/table_layout_algorithm.cc:477-495
pub(crate) fn EndTableBoxLayout(
    border_end: LayoutUnit,
    spacing: LayoutUnit,
    minimum: LayoutUnit,
    extent: &mut TableBoxExtent,
    inflation: &mut LayoutUnit,
) -> LayoutUnit {
    debug_assert!(extent.start <= extent.end);
    extent.end += spacing + border_end;
    let sections = extent.end - extent.start;
    let grid = sections.max(minimum);
    extent.end = extent.start + grid;
    *inflation = grid - sections;
    extent.end
}
// cpp: layoutng_table/table_layout_utils.h:29-31
// cpp: layoutng_table/table_layout_algorithm.cc:499-545
pub fn ComputeTableInlineSize(
    table: &TableNode,
    space: &ConstraintSpace,
    border_padding: &BoxStrut,
) -> LayoutUnit {
    let style = table.Style();
    let fixed = style.IsFixedTableLayout();
    let spacing = style.TableBorderSpacing();
    let grouped = TableGroupedChildren::new(table);
    let borders = unsafe { &*table.GetTableBorders() };
    let columns = table
        .GetColumnConstraints(&grouped, border_padding)
        .unwrap();
    let caption = ComputeCaptionConstraint(space, style, &grouped);
    let undistributable =
        ComputeUndistributableTableSpace(&columns, border_padding.InlineSum(), spacing.inline_size);
    let assignable = ComputeAssignableTableInlineSize(
        table,
        space,
        &columns,
        &caption,
        undistributable,
        border_padding,
        fixed,
    );
    if columns.data.is_empty() {
        return ComputeEmptyTableInlineSize(
            space,
            style,
            assignable,
            undistributable,
            &caption,
            border_padding,
            borders.IsCollapsed(),
        );
    }
    let sizes = SynchronizeAssignableTableInlineSizeAndColumns(assignable, fixed, &columns);
    let mut locations = Vector::new();
    ComputeLocationsFromColumns(&columns, &sizes, spacing.inline_size, true, &mut locations);
    ComputeTableSizeFromColumns(&locations, border_padding, spacing).max(caption.min_size)
}
// cpp: layoutng_table/table_layout_utils.h:32-34
// cpp: layoutng_table/table_layout_algorithm.cc:547-565
pub fn ComputeTableCaptionBlockSize(
    table: &TableNode,
    geometry: &FragmentGeometry,
    space: &ConstraintSpace,
) -> LayoutUnit {
    let mut builder = BoxFragmentBuilder::new(
        table.base.base.clone(),
        table.Style(),
        space,
        space.GetWritingDirection(),
        std::ptr::null(),
    );
    builder.SetIsNewFormattingContext(space.IsNewFormattingContext());
    builder.SetInitialFragmentGeometry(geometry);
    if space.HasBlockFragmentation() {
        SetupFragmentBuilderForFragmentation(
            space,
            table.base.clone(),
            std::ptr::null(),
            &mut builder,
        );
    }
    let grouped = TableGroupedChildren::new(table);
    let mut size = LayoutUnit::default();
    ComputeCaptionFragments(&builder, table.Style(), &grouped, None, &mut size);
    size
}

#![allow(non_snake_case)]
use crate::{
    layout_table_column_visitor::VisitLayoutTableColumn,
    table_break_token_data::TableBreakTokenData,
    table_layout_support::*,
    table_layout_utils::{
        ComputeGridInlineMinMax, ComputeSectionMinimumRowBlockSizes,
        DistributeTableBlockSizeToSections, SynchronizeAssignableTableInlineSizeAndColumns,
    },
};
use foundation::{kIndefiniteSize, DynamicTo, HeapVector, LayoutUnit, Vector};
use layoutng_assembly::{
    block_break_token::BlockBreakToken,
    box_fragment_builder::BoxFragmentBuilder,
    internal::{
        algorithm_entry::NativeAlgorithm,
        fragmentation_utils::IsBreakInside,
        layout_algorithm::{
            LayoutAlgorithm, LayoutAlgorithmParams, RelayoutAlgorithm, RelayoutType,
        },
        layout_input_node::MinMaxSizesFloatInput,
        length_utils::{ComputeBlockSizeForFragment, ResolveMainBlockLengthWithIntrinsicSize},
        min_max_sizes::{MinMaxSizes, MinMaxSizesResult},
        table_borders::TableBorders,
        table_column_location::TableColumnLocation,
        table_fragment_data::CollapsedTableBordersGeometry,
        table_layout_algorithm_types::{
            CellBlockConstraints, Rows, Sections, TableGroupedChildren, TableTypes,
        },
        table_node::TableNode,
    },
    layout_result::{EStatus, LayoutResult},
};
use layoutng_geometry::geometry::{
    box_strut::BoxStrut, logical_rect::LogicalRect, logical_size::LogicalSize,
};
use std::ops::{Deref, DerefMut};
// cpp: layoutng_table/table_layout_algorithm.h:36-89
#[repr(C)]
pub struct TableLayoutAlgorithm {
    pub(crate) base: LayoutAlgorithm<TableNode, BoxFragmentBuilder, BlockBreakToken>,
    pub(crate) total_table_min_block_size_: LayoutUnit,
    pub(crate) is_known_to_be_last_table_box_: bool,
}
impl Deref for TableLayoutAlgorithm {
    type Target = LayoutAlgorithm<TableNode, BoxFragmentBuilder, BlockBreakToken>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for TableLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
const _: () = assert!(std::mem::offset_of!(TableLayoutAlgorithm, base) == 0);
impl TableLayoutAlgorithm {
    // cpp: layoutng_table/table_layout_algorithm.h:39-40,80-87
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        Self {
            base: LayoutAlgorithm::from_params(params),
            total_table_min_block_size_: LayoutUnit::default(),
            is_known_to_be_last_table_box_: false,
        }
    }
    // cpp: layoutng_table/table_layout_algorithm.cc:567-577
    pub fn SetupRelayoutData(&mut self, previous: &Self, mode: RelayoutType) {
        self.base.SetupRelayoutData(&previous.base, mode);
        self.is_known_to_be_last_table_box_ = if mode == RelayoutType::kRelayoutAsLastTableBox {
            true
        } else {
            previous.is_known_to_be_last_table_box_
        };
    }
    // cpp: layoutng_table/table_layout_algorithm.cc:579-710
    pub fn Layout(&mut self) -> *const LayoutResult {
        if self.is_known_to_be_last_table_box_ {
            self.base
                .container_builder_
                .SetShouldCloneBoxEndDecorations(false);
            self.base
                .container_builder_
                .SetShouldPreventBreakBeforeBlockEndDecorations(true);
        }
        let node = self.Node().clone();
        let style = node.Style();
        let space = unsafe { &*(self.GetConstraintSpace() as *const _) };
        let fixed = style.IsFixedTableLayout();
        let spacing = style.TableBorderSpacing();
        let grouped = TableGroupedChildren::new(&node);
        let borders = unsafe { &*node.GetTableBorders() };
        let border_padding = *self.base.container_builder_.BorderPadding();
        let columns = node
            .GetColumnConstraints(&grouped, &border_padding)
            .unwrap();
        let caption = ComputeCaptionConstraint(space, style, &grouped);
        let undistributable = ComputeUndistributableTableSpace(
            &columns,
            border_padding.InlineSum(),
            spacing.inline_size,
        );
        let assignable = ComputeAssignableTableInlineSize(
            &node,
            space,
            &columns,
            &caption,
            undistributable,
            &border_padding,
            fixed,
        );
        let sizes = SynchronizeAssignableTableInlineSizeAndColumns(assignable, fixed, &columns);
        let mut locations = Vector::new();
        let mut collapsed = ComputeLocationsFromColumns(
            &columns,
            &sizes,
            spacing.inline_size,
            false,
            &mut locations,
        );
        let empty = locations.is_empty();
        let size_before_collapse = if empty {
            ComputeEmptyTableInlineSize(
                space,
                style,
                assignable,
                undistributable,
                &caption,
                &border_padding,
                borders.IsCollapsed(),
            )
        } else {
            ComputeTableSizeFromColumns(&locations, &border_padding, spacing)
        };
        let mut captions = HeapVector::new();
        let mut captions_size = LayoutUnit::default();
        ComputeCaptionFragments(
            &self.base.container_builder_,
            style,
            &grouped,
            Some(&mut captions),
            &mut captions_size,
        );
        let mut rows = Vector::new();
        let mut cells = Vector::new();
        let mut sections = Vector::new();
        let mut minimum_grid_size = LayoutUnit::default();
        self.ComputeRows(
            size_before_collapse - border_padding.InlineSum(),
            &grouped,
            &locations,
            borders,
            spacing,
            &border_padding,
            captions_size,
            &mut rows,
            &mut cells,
            &mut sections,
            &mut minimum_grid_size,
        );
        if collapsed {
            collapsed = ComputeLocationsFromColumns(
                &columns,
                &sizes,
                spacing.inline_size,
                true,
                &mut locations,
            );
        }
        #[cfg(debug_assertions)]
        {
            if !collapsed {
                debug_assert!(
                    (size_before_collapse - self.base.container_builder_.InlineSize()).Abs()
                        < LayoutUnit::from_signed(1)
                );
            } else if space.IsFixedInlineSize() {
                let size = ComputeTableSizeFromColumns(&locations, &border_padding, spacing)
                    .max(caption.min_size);
                debug_assert!(size <= self.base.container_builder_.InlineSize());
            } else {
                let size = ComputeTableSizeFromColumns(&locations, &border_padding, spacing)
                    .max(caption.min_size);
                debug_assert!(
                    (size - self.base.container_builder_.InlineSize()).Abs()
                        < LayoutUnit::from_signed(1)
                );
            }
        }
        let result = self.GenerateFragment(
            self.base.container_builder_.InlineSize(),
            minimum_grid_size,
            &grouped,
            &locations,
            &rows,
            &cells,
            &sections,
            &captions,
            borders,
            if empty {
                LogicalSize::default()
            } else {
                spacing
            },
        );
        let result_ref = unsafe { &*result };
        if result_ref.Status() == EStatus::kNeedsRelayoutAsLastTableBox {
            debug_assert!(!self.is_known_to_be_last_table_box_);
            return self
                .base
                .RelayoutDefault::<Self>(RelayoutType::kRelayoutAsLastTableBox);
        }
        if result_ref.Status() == EStatus::kNeedsEarlierBreak {
            debug_assert!(!self.is_known_to_be_last_table_box_);
            return self
                .base
                .RelayoutAndBreakEarlierDefault::<Self>(unsafe { &*result_ref.GetEarlyBreak() });
        }
        result
    }
    // cpp: layoutng_table/table_layout_algorithm.cc:715-745
    pub fn ComputeMinMaxSizes(&mut self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        let node = self.Node();
        let style = node.Style();
        let fixed = style.IsFixedTableLayout();
        let spacing = style.TableBorderSpacing();
        let border_padding = self.base.container_builder_.BorderPadding();
        let grouped = TableGroupedChildren::new(node);
        let columns = node.GetColumnConstraints(&grouped, border_padding).unwrap();
        let caption = ComputeCaptionConstraint(self.GetConstraintSpace(), style, &grouped);
        let undistributable = ComputeUndistributableTableSpace(
            &columns,
            border_padding.InlineSum(),
            spacing.inline_size,
        );
        let grid = ComputeGridInlineMinMax(node, &columns, undistributable, fixed, false);
        let mut sizes = MinMaxSizes {
            min_size: grid.min_size.max(caption.min_size),
            max_size: grid.max_size.max(caption.min_size),
        };
        if fixed && style.LogicalWidth().HasPercent() {
            sizes.max_size = TableTypes::kTableMaxInlineSize();
        }
        debug_assert!(sizes.min_size <= sizes.max_size);
        MinMaxSizesResult {
            sizes,
            depends_on_block_constraints: false,
            applied_aspect_ratio: false,
        }
    }
    // cpp: layoutng_table/table_layout_algorithm.cc:748-852
    fn ComputeRows(
        &mut self,
        grid_inline_size: LayoutUnit,
        grouped: &TableGroupedChildren,
        locations: &Vector<TableColumnLocation>,
        borders: &TableBorders,
        spacing: LogicalSize,
        border_padding: &BoxStrut,
        captions_size: LayoutUnit,
        rows: &mut Rows,
        cells: &mut CellBlockConstraints,
        sections: &mut Sections,
        minimum_grid_size: &mut LayoutUnit,
    ) {
        debug_assert!(rows.is_empty() && cells.is_empty());
        let token = self.GetBreakToken();
        let data = if token.is_null() {
            std::ptr::null()
        } else {
            DynamicTo::<TableBreakTokenData>(unsafe { &*token }.TokenData())
        };
        if let Some(data) = unsafe { data.as_ref() } {
            debug_assert!(IsBreakInside(token));
            *rows = data.rows.clone();
            *cells = data.cell_block_constraints.clone();
            *sections = data.sections.clone();
            self.total_table_min_block_size_ = data.total_table_min_block_size;
        } else {
            debug_assert_eq!(self.total_table_min_block_size_, LayoutUnit::default());
            let specified = !self.Style().LogicalHeight().IsAuto();
            let mut index = 0;
            let mut it = grouped.begin();
            while !it.Equals(&grouped.end()) {
                ComputeSectionMinimumRowBlockSizes(
                    &it.Dereference(),
                    grid_inline_size,
                    specified,
                    locations,
                    borders,
                    spacing.block_size,
                    index,
                    it.TreatAsTBody(),
                    sections,
                    rows,
                    cells,
                );
                index += 1;
                self.total_table_min_block_size_ += sections.last().unwrap().block_size;
                it.Increment();
            }
        }
        let space = self.GetConstraintSpace();
        let style = self.Style();
        let node = self.Node();
        let css_size = if space.IsInitialBlockSizeIndefinite() && !space.IsFixedBlockSize() {
            kIndefiniteSize
        } else {
            let min = style.LogicalMinHeight();
            let intrinsic = if min.HasAuto()
                || ResolveMainBlockLengthWithIntrinsicSize(
                    space,
                    style,
                    border_padding,
                    min,
                    None,
                    kIndefiniteSize,
                    kIndefiniteSize,
                ) == kIndefiniteSize
            {
                kIndefiniteSize
            } else {
                border_padding.BlockSum()
            };
            let available = if space.AvailableSize().block_size != kIndefiniteSize {
                (space.AvailableSize().block_size - captions_size).ClampNegativeToZero()
            } else {
                kIndefiniteSize
            };
            ComputeBlockSizeForFragment(
                space,
                node,
                border_padding,
                intrinsic,
                grid_inline_size,
                available,
            )
        };
        let empty_quirks = unsafe { &*node.GetLayoutBox() }.InQuirksModeForLayout()
            && grouped.begin().Equals(&grouped.end());
        if css_size != kIndefiniteSize && !empty_quirks {
            *minimum_grid_size = css_size;
            let distributable = (css_size - border_padding.BlockSum()).max(LayoutUnit::default());
            if distributable > self.total_table_min_block_size_ {
                DistributeTableBlockSizeToSections(
                    spacing.block_size,
                    distributable,
                    sections,
                    rows,
                );
            }
        }
        let count = rows.len();
        for row in rows {
            if row.is_collapsed {
                if *minimum_grid_size != LayoutUnit::default() {
                    *minimum_grid_size -= row.block_size;
                    if count > 1 {
                        *minimum_grid_size -= spacing.block_size;
                    }
                }
                row.block_size = LayoutUnit::default();
            }
        }
    }
    // cpp: layoutng_table/table_layout_algorithm.cc:857-897
    pub(crate) fn ComputeTableSpecificFragmentData(
        &mut self,
        grouped: &TableGroupedChildren,
        locations: &Vector<TableColumnLocation>,
        _rows: &Rows,
        borders: &TableBorders,
        grid: LogicalRect,
        block_size: LayoutUnit,
    ) {
        let builder = &mut self.base.container_builder_;
        builder.SetTableGridRect(grid);
        builder.SetTableColumnCount(locations.len() as u32);
        builder.SetHasCollapsedBorders(borders.IsCollapsed());
        if !grouped.columns.is_empty() {
            let mut geometry = ColumnGeometriesBuilder {
                geometries: Vector::new(),
                locations,
                table_column_block_size: block_size,
            };
            VisitLayoutTableColumn(&grouped.columns, locations.len() as u32, &mut geometry);
            geometry.Sort();
            builder.SetTableColumnGeometries(geometry.geometries);
        }
        if !borders.IsEmpty() {
            let mut geometry = Box::new(CollapsedTableBordersGeometry::default());
            for column in locations {
                geometry.columns.push(column.offset);
            }
            debug_assert!(!locations.is_empty());
            let last = locations.last().unwrap();
            geometry.columns.push(last.offset + last.size);
            debug_assert!(borders.EdgesPerRow() as usize / 2 <= geometry.columns.len());
            builder.SetTableCollapsedBorders(borders);
            builder.SetTableCollapsedBordersGeometry(geometry);
        }
    }
}
impl NativeAlgorithm for TableLayoutAlgorithm {
    fn new(params: &LayoutAlgorithmParams) -> Self {
        Self::new(params)
    }
    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }
    fn compute_min_max_sizes(&mut self, input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes(input)
    }
}
impl RelayoutAlgorithm<TableNode> for TableLayoutAlgorithm {
    fn base(&self) -> &LayoutAlgorithm<TableNode, BoxFragmentBuilder, BlockBreakToken> {
        &self.base
    }
    fn base_mut(&mut self) -> &mut LayoutAlgorithm<TableNode, BoxFragmentBuilder, BlockBreakToken> {
        &mut self.base
    }
    unsafe fn from_base<'a>(
        base: &'a LayoutAlgorithm<TableNode, BoxFragmentBuilder, BlockBreakToken>,
    ) -> &'a Self {
        unsafe { &*(base as *const _ as *const Self) }
    }
    fn setup_relayout_data(&mut self, previous: &Self, mode: RelayoutType) {
        self.SetupRelayoutData(previous, mode);
    }
}

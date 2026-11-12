//! LayoutTableCell paint invalidation during PrePaint.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/table_cell_paint_invalidator.{h,cc}`.

use foundation::{IsFullPaintInvalidationReason, PaintInvalidationReason};
use layoutng_assembly::fragment_tree::FragmentPaintData;
use layoutng_assembly::internal::layout_input::{Display, Offset};

use crate::block_paint_invalidator::BlockPaintInvalidator;
use crate::paint_invalidator::PaintInvalidatorContext;

pub struct TableCellPaintInvalidator;

impl TableCellPaintInvalidator {
    /// Source: `table_cell_paint_invalidator.cc:41-42`.
    pub fn CellGeometryChanged(
        fragment: &FragmentPaintData,
        previous: Option<&FragmentPaintData>,
        old_paint_offset: Option<Offset>,
        new_paint_offset: Offset,
    ) -> bool {
        old_paint_offset.is_some_and(|old| old != new_paint_offset)
            || crate::box_paint_invalidator::PreviousGeometry(fragment, previous)
                .is_some_and(|old| old.size != StitchedSize(fragment))
    }

    /// Source: `table_cell_paint_invalidator.cc:17-32`. The caller performs
    /// the row/section/table walk in source order while unwinding the physical
    /// fragment tree after the cell has been visited.
    pub fn InvalidateContainerForCellGeometryChange(container: &mut FragmentPaintData) {
        let current = container
            .paint
            .display_item_client_invalidation_reason
            .unwrap_or(PaintInvalidationReason::kNone);
        if !IsFullPaintInvalidationReason(current) && PaintInvalidationReason::kLayout > current {
            container.paint.display_item_client_invalidation_reason =
                Some(PaintInvalidationReason::kLayout);
        }
    }

    /// Source: `table_cell_paint_invalidator.cc:36-69` for the cell's own
    /// BlockPaintInvalidator call. Ancestor invalidation is applied separately
    /// because immutable fragment ownership makes it an upward result.
    pub fn ComputePaintInvalidationReason(
        fragment: &FragmentPaintData,
        previous: Option<&FragmentPaintData>,
        old_paint_offset: Option<Offset>,
        new_paint_offset: Offset,
        context: PaintInvalidatorContext,
        reason: PaintInvalidationReason,
    ) -> PaintInvalidationReason {
        BlockPaintInvalidator::ComputePaintInvalidationReason(
            fragment,
            previous,
            old_paint_offset,
            new_paint_offset,
            context,
            reason,
        )
    }

    pub fn ContainerPaintsCellGeometry(container: &FragmentPaintData) -> bool {
        match container.paint.display {
            Display::kTableRow
            | Display::kTableSection
            | Display::kTableHeaderGroup
            | Display::kTableFooterGroup => container.paint.has_background,
            Display::kTable | Display::kInlineTable => {
                container.paint.has_background
                    || container.paint.has_collapsed_borders
                    || container.paint.table.as_ref().is_some_and(|table| {
                        table.columns.iter().any(|column| {
                            column.style.background_color().alpha > 0.0
                                || column.style.has_background_images()
                        })
                    })
            }
            _ => false,
        }
    }

    /// Applies the next row/section/table step while unwinding. Returns
    /// whether the geometry-change result must continue toward another
    /// ancestor; the table is the last container in Chromium's source order.
    pub fn InvalidateAncestorAndPropagate(container: &mut FragmentPaintData) -> bool {
        let propagate = !matches!(
            container.paint.display,
            Display::kTable | Display::kInlineTable
        );
        if Self::ContainerPaintsCellGeometry(container) {
            Self::InvalidateContainerForCellGeometryChange(container);
        }
        propagate
    }
}

fn StitchedSize(fragment: &FragmentPaintData) -> layoutng_assembly::internal::layout_input::Size {
    fragment
        .paint
        .stitched_decoration
        .as_ref()
        .map_or(fragment.size, |decoration| decoration.size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::internal::layout_input::Size;

    #[test]
    fn cell_size_change_is_geometry_change() {
        let mut old = FragmentPaintData::default();
        old.size = Size {
            width: 40.0,
            height: 20.0,
        };
        let mut current = old.clone();
        current.size.width = 50.0;
        assert!(TableCellPaintInvalidator::CellGeometryChanged(
            &current,
            Some(&old),
            Some(Offset::default()),
            Offset::default(),
        ));
    }

    #[test]
    fn container_gets_layout_reason_unless_already_fully_invalidated() {
        let mut row = FragmentPaintData::default();
        row.paint.display = Display::kTableRow;
        row.paint.has_background = true;
        TableCellPaintInvalidator::InvalidateContainerForCellGeometryChange(&mut row);
        assert_eq!(
            row.paint.display_item_client_invalidation_reason,
            Some(PaintInvalidationReason::kLayout)
        );

        row.paint.display_item_client_invalidation_reason = Some(PaintInvalidationReason::kStyle);
        TableCellPaintInvalidator::InvalidateContainerForCellGeometryChange(&mut row);
        assert_eq!(
            row.paint.display_item_client_invalidation_reason,
            Some(PaintInvalidationReason::kStyle)
        );
    }

    #[test]
    fn row_and_section_propagate_but_table_finishes_the_ancestor_walk() {
        for display in [Display::kTableRow, Display::kTableSection] {
            let mut container = FragmentPaintData::default();
            container.paint.display = display;
            assert!(TableCellPaintInvalidator::InvalidateAncestorAndPropagate(
                &mut container
            ));
        }
        let mut table = FragmentPaintData::default();
        table.paint.display = Display::kTable;
        assert!(!TableCellPaintInvalidator::InvalidateAncestorAndPropagate(
            &mut table
        ));
    }
}

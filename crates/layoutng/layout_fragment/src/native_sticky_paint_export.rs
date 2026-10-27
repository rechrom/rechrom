//! Native sticky-position paint constraint projection.
//!
//! Kept separate from sticky offset recomputation so profiles distinguish
//! native constraint updates from fragment payload export.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:2137-2173
// Keep the matrix offset and the compositor's recorded constraint basis from
// the same native sticky update, for full export and retained scroll refresh.
#[inline(never)]
pub(super) fn ExportStickyPaintProperties(
    object: *const LayoutObject,
    output: &mut PaintProperties,
) {
    output.sticky_offset = Offset::default();
    output.sticky_constraint = None;
    if output.position == Position::kSticky {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if !box_model.is_null() {
            let box_model = unsafe { &*box_model };
            let constraints = box_model.StickyConstraints();
            let offset = box_model.StickyPositionOffset();
            output.sticky_offset = Offset {
                x: Number(offset.left),
                y: Number(offset.top),
            };
            let display_client_id = |object: *const LayoutObject| {
                if object.is_null() {
                    0
                } else {
                    unsafe { &*(object as *const DisplayItemClient) }.Id() as u64
                }
            };
            let nearest_sticky_box_client_id = display_client_id(
                constraints.NearestStickyLayerShiftingStickyBox() as *const LayoutObject,
            );
            let nearest_containing_block_client_id = display_client_id(
                constraints.NearestStickyLayerShiftingContainingBlock() as *const LayoutObject,
            );
            let export_axis = |axis: PhysicalAxis| {
                let data = constraints.AxisData(axis);
                if data.is_null() {
                    return None;
                }
                let data = unsafe { &*data };
                Some(StickyAxisPaintData {
                    min_inset: data.min_inset.map(Number),
                    max_inset: data.max_inset.map(Number),
                    scroll_container_relative_containing_block_offset: Number(
                        data.scroll_container_relative_containing_block_range.offset,
                    ),
                    scroll_container_relative_containing_block_size: Number(
                        data.scroll_container_relative_containing_block_range.size,
                    ),
                    scroll_container_relative_sticky_box_offset: Number(
                        data.scroll_container_relative_sticky_box_range.offset,
                    ),
                    scroll_container_relative_sticky_box_size: Number(
                        data.scroll_container_relative_sticky_box_range.size,
                    ),
                    constraining_offset: Number(data.constraining_range.offset),
                    constraining_size: Number(data.constraining_range.size),
                    scroll_container_client_id: display_client_id(
                        data.containing_scroll_container.Get() as *const LayoutObject,
                    ),
                    nearest_sticky_box_client_id,
                    nearest_containing_block_client_id,
                    current_sticky_offset: Number(data.sticky_offset),
                    current_total_sticky_box_offset: Number(data.total_sticky_box_sticky_offset),
                    current_total_containing_block_offset: Number(
                        data.total_containing_block_sticky_offset,
                    ),
                })
            };
            let constraint = StickyPositionPaintData {
                owner_client_id: output.display_item_client_id,
                horizontal: export_axis(PhysicalAxis::kHorizontal),
                vertical: export_axis(PhysicalAxis::kVertical),
            };
            if constraint.horizontal.is_some() || constraint.vertical.is_some() {
                output.sticky_constraint = Some(constraint);
            }
        }
    }
}

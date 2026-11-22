#![allow(non_snake_case)]

use layout_assembly::internal::anchor_position_scroll_data::AnchorPositionScrollData;
use layout_assembly::internal::css::out_of_flow_data::ScrollOffsetPair;
use layout_assembly::internal::layout_node_metadata::Element;
use layout_assembly::internal::layout_object::LayoutObject;

// cpp: layoutng_out_of_flow/anchor_scroll_services.cc:5-13
pub fn ComputeAnchorScrollOffsets(
    anchored_element: &Element,
    anchor: &LayoutObject,
) -> ScrollOffsetPair {
    let adjustment_data = AnchorPositionScrollData::ComputeAdjustmentContainersData(
        anchored_element as *const Element,
        anchor,
    );
    ScrollOffsetPair {
        scroll_offset_for_layout: adjustment_data.accumulated_adjustment,
        scroll_offset_for_range_adjustment: adjustment_data.accumulated_range_adjustment_offset,
    }
}

#![allow(non_snake_case)]

use layout_base::layout_registry::LayoutRegistry;

use crate::float_layout_algorithm::{
    ComputeMarginBoxInlineSizeForUnpositionedFloat, PositionFloat,
};
use crate::shape_outside_algorithm::{CreateShapeOutside, UpdateShapeOutsideInfo};

// cpp: layoutng_float/assembly.h:3-5
// cpp: layoutng_float/assembly.cc:9-16
pub fn Register(registry: &mut LayoutRegistry) {
    registry.algorithms.float_support.performance_owner = Some("layout_algo_float");
    registry.algorithms.float_support.margin_box_inline_size =
        Some(ComputeMarginBoxInlineSizeForUnpositionedFloat);
    registry.algorithms.float_support.position = Some(PositionFloat);
    registry.algorithms.float_support.create_shape = Some(CreateShapeOutside);
    registry.algorithms.float_support.update_shape_outside = Some(UpdateShapeOutsideInfo);
}

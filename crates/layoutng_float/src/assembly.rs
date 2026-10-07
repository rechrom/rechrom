#![allow(non_snake_case)]

use layoutng_assembly::layout_assembly::LayoutAssembly;

use crate::float_layout_algorithm::{
    ComputeMarginBoxInlineSizeForUnpositionedFloat, PositionFloat,
};
use crate::shape_outside_algorithm::{CreateShapeOutside, UpdateShapeOutsideInfo};

// cpp: layoutng_float/assembly.h:3-5
// cpp: layoutng_float/assembly.cc:9-16
pub fn InstallFloatAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.algorithms.float_support.margin_box_inline_size =
        Some(ComputeMarginBoxInlineSizeForUnpositionedFloat);
    assembly.algorithms.float_support.position = Some(PositionFloat);
    assembly.algorithms.float_support.create_shape = Some(CreateShapeOutside);
    assembly.algorithms.float_support.update_shape_outside = Some(UpdateShapeOutsideInfo);
}

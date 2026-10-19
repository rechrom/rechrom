// Source: components/viz/service/display/renderer_utils.h:1-17
// Source: components/viz/service/display/renderer_utils.cc:1-26
#![allow(non_snake_case)]
use skia::raster::Transform;
// Source: components/viz/service/display/renderer_utils.cc:12-14. SkScalarNearlyZero's default tolerance.
fn IsScalarNearlyInteger(value: f32) -> bool {
    (value - value.round()).abs() <= 1.0 / 4096.0
}
// Source: components/viz/service/display/renderer_utils.cc:16-24. The CPU affine matrix has the constant
// perspective row [0,0,1], satisfying the last three source predicates.
pub fn IsScaleAndIntegerTranslate(matrix: Transform) -> bool {
    IsScalarNearlyInteger(matrix.tx)
        && IsScalarNearlyInteger(matrix.ty)
        && matrix.kx.abs() <= 1.0 / 4096.0
        && matrix.ky.abs() <= 1.0 / 4096.0
}

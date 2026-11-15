// cpp: foundation/blink_geometry/transforms/transform_operation.h:41-65
// The C++ discriminant belongs to TransformOperation. Keep it independent of
// the still-unconnected virtual operation objects so style can test the
// precise operation kind without importing an algorithm implementation.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TransformOperationType {
    kScaleX,
    kScaleY,
    kScale,
    kTranslateX,
    kTranslateY,
    kTranslate,
    kRotate,
    kRotateZ,
    kSkewX,
    kSkewY,
    kSkew,
    kMatrix,
    kScaleZ,
    kScale3D,
    kTranslateZ,
    kTranslate3D,
    kRotateX,
    kRotateY,
    kRotate3D,
    kMatrix3D,
    kPerspective,
    kInterpolated,
    kRotateAroundOrigin,
}

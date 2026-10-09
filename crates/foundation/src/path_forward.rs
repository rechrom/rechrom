// cpp: layoutng_style/style/basic_shapes.h:50
// This source tree only forward-declares blink::Path. Unlike pointer-facing
// declarations, GetPath returns it by value, so there is no valid Rust value
// or ABI to implement here. The uninhabited type makes that unresolved
// external return boundary explicit until the owning Path package is linked.
pub enum Path {}

// cpp: platform/geometry/path.h:70-74. This independent result does not
// instantiate the unresolved Path owner.
pub struct PointAndTangent {
    pub point: crate::gfx::PointF,
    pub tangent_in_degrees: f32,
}

//! Canonical immutable geometry Path; concrete raster backends consume these
//! platform-neutral verbs without exposing their native path owners.
pub type Path = crate::shape_path::ShapePath;

// cpp: platform/geometry/path.h:70-74. This independent result does not
// instantiate the unresolved Path owner.
pub struct PointAndTangent {
    pub point: crate::gfx::PointF,
    pub tangent_in_degrees: f32,
}

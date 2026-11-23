pub use compositor_tree::geometry::*;

/// Path projection implemented by the external Skia path type. Moved from
/// compositor/src/adapter/geometry.rs, which re-exports it.
pub trait Path {
    fn AsRRect(&self) -> Option<RRectF>;
    fn bounds(&self) -> RectF;
    fn count_points(&self) -> i32;
    fn count_verbs(&self) -> i32;
    fn is_finite(&self) -> bool;
    fn is_convex(&self) -> bool;
    fn is_volatile(&self) -> bool;
    fn is_inverse_fill_type(&self) -> bool;
}

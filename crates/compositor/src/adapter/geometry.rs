pub use compositor_tree::geometry::*;

/// Facade-owned path projection implemented by the external Skia path type.
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

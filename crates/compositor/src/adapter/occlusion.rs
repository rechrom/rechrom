pub use compositor_draw::occlusion::SkMatrix;
pub use compositor_draw::occlusion_context::*;

use super::geometry::Rect;

/// Facade-local projection for the external paint filter type. The draw core
/// consumes only the answered values through its OcclusionContext.
pub trait OcclusionFilters {
    fn HasFilterThatAffectsOpacity(&self) -> bool;
    fn HasFilterThatMovesPixels(&self) -> bool;
    fn MapRectReverse(&self, rect: Rect, transform: &SkMatrix) -> Rect;
}

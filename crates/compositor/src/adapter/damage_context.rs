pub use compositor_draw::damage_context::*;

use super::{geometry::Rect, occlusion::SkMatrix};

/// Facade-local projection for the external paint filter type. The draw core
/// receives only the computed answers through its DamageSurface boundary.
pub trait DamageFilters {
    fn HasFilterThatMovesPixels(&self) -> bool;
    fn MapRect(&self, rect: Rect, transform: &SkMatrix) -> Rect;
}

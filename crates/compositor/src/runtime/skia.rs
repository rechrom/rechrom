//! Thin bindings to public canonical Skia value algorithms.
//! The Skia region/path/rrect value backend lives in
//! layerize::commit::skia_region; only the impl-side filter facets of the
//! external FilterOperations type stay here.
pub use layerize::commit::skia_region::*;

use crate::adapter::{geometry::Rect, occlusion::SkMatrix};

// DrawFilters and the FilterOperation matrix binding moved to
// layerize::commit::skia_filters (coherence with the DrawFilters trait).
use layerize::commit::skia_filters::{filter_rect, sk_filter_rect, FilterMapTransform};
impl crate::adapter::damage_context::DamageFilters for FilterOperations {
    fn HasFilterThatMovesPixels(&self) -> bool {
        Self::HasFilterThatMovesPixels(self)
    }
    fn MapRect(&self, rect: Rect, transform: &SkMatrix) -> Rect {
        let transform = FilterMapTransform::new(transform.values);
        filter_rect(Self::MapRect(self, sk_filter_rect(rect), Some(&transform)))
    }
}
impl crate::adapter::occlusion::OcclusionFilters for FilterOperations {
    fn HasFilterThatAffectsOpacity(&self) -> bool {
        Self::HasFilterThatAffectsOpacity(self)
    }
    fn HasFilterThatMovesPixels(&self) -> bool {
        Self::HasFilterThatMovesPixels(self)
    }
    fn MapRectReverse(&self, rect: Rect, transform: &SkMatrix) -> Rect {
        let transform = FilterMapTransform::new(transform.values);
        filter_rect(Self::MapRectReverse(
            self,
            sk_filter_rect(rect),
            Some(&transform),
        ))
    }
}

// Source consumer: cc/layers/render_surface_impl.cc:242-246. The selected
// native FilterOperations owns the exact expansion algorithm; this only maps
// the external 3x3/value representations at the package boundary.
impl crate::adapter::render_surface_impl::RenderSurfaceFilters for FilterOperations {
    fn ExpandRectWithMatrix(&self, rect: Rect, matrix: &SkMatrix) -> Rect {
        let matrix = FilterMapTransform::new(matrix.values);
        filter_rect(Self::ExpandRect(self, sk_filter_rect(rect), Some(&matrix)))
    }
}

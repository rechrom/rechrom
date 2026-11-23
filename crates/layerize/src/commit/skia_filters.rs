//! The Skia binding of cc::FilterOperations used by both halves. Moved from
//! compositor/src/runtime/skia.rs: coherence requires the DrawFilters impl for
//! paint's FilterOperations beside its trait, which now lives in
//! layerize::commit. The impl half reuses the matrix and rect conversions for
//! its damage, occlusion and render-surface filter facets.
use super::{
    geometry::{Rect, Vector2dF},
    layer_value_backend::DrawFilters,
};
use foundation::gfx;
use paint::cc::paint::filter_operation::{
    FilterMapTransform as FilterMapTransformContract, FilterType,
};
use paint::cc::paint::filter_operations::FilterOperations;
use skia::src::core::{SkMatrix::SkMatrix, SkPoint::SkPoint};

/// Feature-selected native matrix used only for FilterOperation::mapVector.
/// Adapter property trees keep their source-shaped nine scalar values while
/// each production Skia path executes its own matrix implementation.
pub struct FilterMapTransform {
    matrix: SkMatrix,
}
impl FilterMapTransform {
    pub fn new(values: [f32; 9]) -> Self {
        Self {
            matrix: SkMatrix::MakeAll(
                values[0], values[1], values[2], values[3], values[4], values[5], values[6],
                values[7], values[8],
            ),
        }
    }
}
impl FilterMapTransformContract for FilterMapTransform {
    fn MapVector(&self, x: f32, y: f32) -> (f32, f32) {
        let mapped = self.matrix.mapVector(SkPoint::make(x, y));
        (mapped.x(), mapped.y())
    }
}
pub fn sk_filter_rect(rect: Rect) -> gfx::Rect {
    gfx::Rect::new(
        gfx::Point::new(rect.x, rect.y),
        gfx::Size::new(rect.width, rect.height),
    )
}
pub fn filter_rect(rect: gfx::Rect) -> Rect {
    Rect::new(rect.x(), rect.y(), rect.width(), rect.height())
}

impl DrawFilters for FilterOperations {
    fn HasReferenceFilter(&self) -> bool {
        Self::HasReferenceFilter(self)
    }
    fn HasFilterThatMovesPixels(&self) -> bool {
        Self::HasFilterThatMovesPixels(self)
    }
    fn HasBlurFilter(&self) -> bool {
        self.HasFilterOfType(FilterType::BLUR)
    }
    fn ExpandRect(&self, rect: Rect, scale: Vector2dF) -> Rect {
        let matrix = FilterMapTransform::new([scale.x(), 0., 0., 0., scale.y(), 0., 0., 0., 1.]);
        filter_rect(Self::ExpandRect(self, sk_filter_rect(rect), Some(&matrix)))
    }
}

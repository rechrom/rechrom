//! The main-half part of the external Skia value backend: the filter and path
//! value types of the property trees and the Skia path operations Blink's
//! PaintChunksToCcLayer / clip conversion delegate to Skia. Moved from
//! compositor/src/runtime/types.rs `SkiaValueBackend`, which now adds only the
//! impl-side filter bounds (render surface, occlusion, damage).
use super::{
    geometry::RectF,
    layer_value_backend::{DrawFilters, MainLayerPath},
};
use paint_model::cc::paint::paint_flags::{Cap, Join};
use paint_model::paint_clip_path::PaintPath;

/// Pure Skia-owned value types only. Policy/owner algorithms are forbidden in
/// this capability; the backend implements native filter/path operations.
pub trait MainSkiaBackend {
    type Filters: DrawFilters + Send + Sync + 'static;
    type Path: MainLayerPath + Default + Send + Sync + 'static;
    /// Compile one immutable Paint-owned path into the selected backend value.
    /// No concrete Skia path crosses Paint's package boundary.
    fn FromPaintPath(path: &PaintPath) -> Self::Path;
    /// Blink Path::StrokeBoundingRect delegates this geometry to Skia. Keeping
    /// it beside path compilation avoids introducing a second stroker in Paint.
    fn StrokeBounds(path: &PaintPath, width: f32, resolution_scale: f32) -> Option<RectF>;
    /// Path::StrokePath is a Skia geometry operation.  Paint retains the
    /// backend-neutral path and stroke inputs; the selected value backend
    /// materializes the filled outline only at the compositor boundary.
    fn StrokePath(
        path: &PaintPath,
        width: f32,
        cap: Cap,
        join: Join,
        miter_limit: f32,
        dash_intervals: &[f32],
        dash_phase: f32,
        resolution_scale: f32,
    ) -> Option<Self::Path>;
    fn StrokeBackendPath(
        path: &Self::Path,
        width: f32,
        cap: Cap,
        join: Join,
        miter_limit: f32,
        dash_intervals: &[f32],
        dash_phase: f32,
        resolution_scale: f32,
    ) -> Option<Self::Path>;
}

/// The Skia value backend cc/paint records are generic over (PaintRecord,
/// PaintFlags, DisplayItemList ops). cc/paint runs on both threads in
/// Chromium, so this is commit vocabulary; the compositor's `CpuSkiaBackend`
/// adds only the impl filter facets on top of it.
pub trait PaintSkiaBackend:
    super::region_backend::SkRegionBackend
    + super::transform::RoundedRectTransformBackend
    + MainSkiaBackend<Filters = paint_model::cc::paint::filter_operations::FilterOperations>
    + Send
    + Sync
    + 'static
{
}
impl<R> PaintSkiaBackend for R where
    R: super::region_backend::SkRegionBackend
        + super::transform::RoundedRectTransformBackend
        + MainSkiaBackend<Filters = paint_model::cc::paint::filter_operations::FilterOperations>
        + Send
        + Sync
        + 'static
{
}

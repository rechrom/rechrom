//! The main-half facet of the external Skia value backend: the property-tree
//! filter value. The region, rounded-rect and path values and the Skia path
//! operations Blink's PaintChunksToCcLayer / clip conversion delegate to Skia
//! are cc/paint's `RecordSkiaBackend` (cc_paint root); this adds the layer
//! bounds main Layers need on them. The compositor's `SkiaValueBackend` adds
//! only the impl-side filter bounds (render surface, occlusion, damage).
use super::layer_value_backend::{DrawFilters, MainLayerPath};
pub use cc_paint::skia_backend::RecordSkiaBackend;

/// Pure Skia-owned value types only. Policy/owner algorithms are forbidden in
/// this capability.
pub trait MainSkiaBackend: RecordSkiaBackend<Path: MainLayerPath> {
    type Filters: DrawFilters + Send + Sync + 'static;
}

/// The Skia value backend main paint conversion is generic over: cc/paint's
/// backend with the cc::FilterOperations filter value.
pub trait PaintSkiaBackend:
    MainSkiaBackend<Filters = paint_model::cc::paint::filter_operations::FilterOperations>
{
}
impl<R> PaintSkiaBackend for R where
    R: MainSkiaBackend<Filters = paint_model::cc::paint::filter_operations::FilterOperations>
{
}

// The production backend's main facet (the value type is cc_paint's).
impl MainSkiaBackend for cc_paint::skia_region::SkRegion {
    type Filters = paint_model::cc::paint::filter_operations::FilterOperations;
}

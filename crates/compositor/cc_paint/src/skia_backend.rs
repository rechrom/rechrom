//! The Skia value backend cc/paint records are generic over: region and
//! rounded-rect values, and the path type with the path operations Blink's
//! Path / PaintChunksToCcLayer delegate to Skia. cc/paint runs on both
//! threads in Chromium; the main half's `MainSkiaBackend` (layerize) adds
//! only the property-tree filter value on top of it.
//! Source: cc/paint/paint_op.h (SkPath operands), cc/base/region.h (SkRegion).

use compositor_tree::geometry::{RRectF, RectF};
use compositor_tree::region_backend::SkRegionBackend;
use compositor_values::transform::RoundedRectTransformBackend;
use paint_model::cc::paint::paint_flags::{Cap, Join};
use paint_model::paint_clip_path::PaintPath;

/// Path projection implemented by the external Skia path type (moved from
/// layerize::commit::geometry, which re-exports it).
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

/// Pure Skia-owned value types only. Policy/owner algorithms are forbidden in
/// this capability; the backend implements native region/path operations.
pub trait RecordSkiaBackend:
    SkRegionBackend + RoundedRectTransformBackend + Send + Sync + 'static
{
    type Path: Path
        + compositor_tree::property_types::PropertyPath
        + Default
        + Send
        + Sync
        + 'static;
    /// Compile one immutable Paint-owned path into the selected backend value.
    /// No concrete Skia path crosses Paint's package boundary.
    fn FromPaintPath(path: &PaintPath) -> Self::Path;
    /// Blink Path::StrokeBoundingRect delegates this geometry to Skia. Keeping
    /// it beside path compilation avoids introducing a second stroker in Paint.
    fn StrokeBounds(path: &PaintPath, width: f32, resolution_scale: f32) -> Option<RectF>;
    /// Path::StrokePath is a Skia geometry operation. Paint retains the
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

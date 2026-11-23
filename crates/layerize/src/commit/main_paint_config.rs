//! Construction values of the main paint services (LayerizeEngine::new):
//! the embedding's Blink/cc feature state and the Skia paint diagnostics.
//! `CpuPaintFeatures` moved from runtime/paint_services.rs and
//! `SkiaPaintDiagnostics` from compositor/src/runtime/paint/services.rs;
//! both re-export them.

/// Explicit embedding feature values. There is deliberately no Default: the
/// embedding supplies the actual enabled runtime/feature-list state.
#[derive(Clone, Copy, Debug)]
pub struct CpuPaintFeatures {
    pub cc_slimming: bool,
    pub singular_transform_fix: bool,
    pub merge_fixed_layers: bool,
    pub merge_sticky_layers: bool,
    pub raster_inducing_scroll: bool,
    pub scrolling_contents_cull_rect: bool,
    pub remove_scroll_node_workaround: bool,
    pub selection_edge_visibility_uses_full_edge: bool,
    pub canvas_draw_element: bool,
    pub expensive_dchecks: bool,
    pub paint_under_invalidation_checking: bool,
    pub layer_merge_distance_limit: i32,
}

/// Diagnostics are independent of normal Paint. Under-invalidation checking
/// is selected through CpuPaintFeatures and requires a pixel-diagnostic backend;
/// this constructor explicitly rejects it instead of returning empty pixels.
#[derive(Clone, Copy, Debug)]
pub struct SkiaPaintDiagnostics {
    pub trace: bool,
    pub verbose_level: i32,
    pub histogram: bool,
}

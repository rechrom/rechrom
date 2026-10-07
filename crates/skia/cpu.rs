//! Compatibility entry points for browser-independent CPU rendering.
pub use crate::compat::analytic_masks as analytic_aa;
pub use crate::compat::f16 as f16_surface;
pub use crate::compat::glyph_position;
pub use crate::compat::hairline;
pub use crate::compat::image_sampling;
pub use crate::compat::layer_filters;
pub use crate::compat::mask_blitter;
pub use crate::extensions::hvgl::hvgl;
pub use crate::extensions::RasterPipelineReplay as raster_pipeline;
pub use crate::src::core::SkCanvas as canvas;
#[cfg(target_os = "macos")]
pub use crate::src::core::SkMaskGamma as mask_gamma;
pub use crate::src::core::SkScan as rect_stroke;
pub use crate::src::core::SkStroke::command_stroker as stroke;
#[cfg(target_os = "macos")]
pub use crate::src::ports::SkScalerContext_mac_ct as scaler_context_mac_ct;

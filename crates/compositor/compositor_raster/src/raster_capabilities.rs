//! Raster backend capabilities, generic over the resource-format value.

pub trait RasterFormat: Copy {
    fn DefaultRgba() -> Self;
}

#[derive(Clone, Copy, Debug)]
pub struct RasterCapabilities<F: RasterFormat> {
    pub use_gpu_rasterization: bool,
    pub can_use_msaa: bool,
    pub max_texture_size: i32,
    pub tile_format: F,
    pub tile_overlay_candidate: bool,
    pub ui_rgba_format: F,
}

impl<F: RasterFormat> Default for RasterCapabilities<F> {
    fn default() -> Self {
        Self {
            use_gpu_rasterization: false,
            can_use_msaa: false,
            max_texture_size: 0,
            tile_format: F::DefaultRgba(),
            tile_overlay_candidate: false,
            ui_rgba_format: F::DefaultRgba(),
        }
    }
}

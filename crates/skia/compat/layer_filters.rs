//! Pixel-storage adapter to SkBlurEngine; no official counterpart API.

//! Renderer storage adapter for the translated SkBlurEngine.
use crate::raster::Pixmap;
pub(crate) fn blur(pixmap: &mut Pixmap, sigma_x: f32, sigma_y: f32) {
    let (width, height) = (pixmap.width(), pixmap.height());
    let mut view =
        crate::compat::pixel_view::PixelView::from_rgba(pixmap.data_mut(), width, height)
            .expect("valid renderer N32 storage");
    crate::src::core::SkBlurEngine::blur(&mut view, sigma_x, sigma_y);
}

/// Apply SkBlurEngine to caller-owned premultiplied RGBA8888 storage.
/// This is the render-pass counterpart of SkCanvas SaveLayerFilter restore.
pub fn blur_rgba(pixels: &mut [u8], width: u32, height: u32, sigma_x: f32, sigma_y: f32) -> bool {
    let Some(mut view) = crate::compat::pixel_view::PixelView::from_rgba(pixels, width, height)
    else {
        return false;
    };
    crate::src::core::SkBlurEngine::blur(&mut view, sigma_x, sigma_y);
    true
}

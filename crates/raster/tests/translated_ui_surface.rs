use compositor_core::{
    adapter::{
        geometry::Size,
        layer_tree_host_impl::{HostUICanvas, HostUISourceBitmap, HostUISurface},
    },
    runtime::native_ui::NativeUiRasterServices,
};
use raster::RasterEngine;

#[test]
fn translated_ui_source_image_draws_into_real_surface() {
    let engine = RasterEngine::new(71, 1024 * 1024);
    let source_info = engine.MakeN32Premul(Size::new(1, 1));
    let source_pixels = [17, 43, 91, 255];
    let mut bitmap = engine.NewSourceBitmap();
    bitmap.setInfo(source_info.clone(), source_info.min_row_bytes);
    bitmap.setPixels(&source_pixels);
    let image = bitmap.asImage();

    let target_info = engine.MakeN32Premul(Size::new(2, 2));
    let mut surface = engine
        .RasterSurface(target_info)
        .expect("translated UI raster surface");
    {
        let canvas = surface.getCanvas();
        canvas.clearTransparent();
        canvas.drawImage(&image, 1.0, 1.0);
    }
    let pixels = surface.peekPixels().pixels;
    assert_eq!(&pixels[..4], &[0, 0, 0, 0]);
    assert_eq!(&pixels[12..16], &source_pixels);
}

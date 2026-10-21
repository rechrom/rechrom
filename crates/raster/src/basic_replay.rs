//! Small direct display-item raster helper used by translation fixtures.

use paint::DisplayItemList;

fn byte(channel: f32) -> u8 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[allow(non_snake_case)]
pub fn RasterizeDisplayItemList(items: &DisplayItemList, width: u32, height: u32) -> Vec<u8> {
    assert!(width > 0 && height > 0);
    let width = width as usize;
    let height = height as usize;
    let mut rgba = vec![
        255;
        width
            .checked_mul(height)
            .and_then(|x| x.checked_mul(4))
            .expect("surface too large")
    ];
    for item in &items.items {
        if item.glyph_run.is_some() {
            panic!("glyph replay requires the RasterEngine/skia path");
        }
        let color = item.color;
        assert_eq!(
            color.alpha, 1.0,
            "translucent display item requires compositing"
        );
        let x0 = (item.rect.x as i32).clamp(0, width as i32) as usize;
        let y0 = (item.rect.y as i32).clamp(0, height as i32) as usize;
        let x1 = ((item.rect.x + item.rect.width) as i32).clamp(0, width as i32) as usize;
        let y1 = ((item.rect.y + item.rect.height) as i32).clamp(0, height as i32) as usize;
        let pixel = [
            byte(color.red),
            byte(color.green),
            byte(color.blue),
            byte(color.alpha),
        ];
        for y in y0..y1 {
            for x in x0..x1 {
                let offset = (y * width + x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&pixel);
            }
        }
    }
    rgba
}

#[allow(non_snake_case)]
pub fn WriteDisplayItemListPng(
    items: &DisplayItemList,
    width: u32,
    height: u32,
    path: &std::path::Path,
) -> std::io::Result<()> {
    let rgba = RasterizeDisplayItemList(items, width, height);
    std::fs::write(path, crate::EncodeRgbaPng(&rgba, width, height))
}

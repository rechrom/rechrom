//! Small direct display-item raster helper used by translation fixtures.

#[cfg(feature = "text")]
use paint::DisplayItem;
use paint::DisplayItemList;

#[cfg(feature = "text")]
unsafe extern "C" {
    fn LayoutngDrawGlyphRun(
        rgba: *mut u8,
        width: i32,
        height: i32,
        font_bytes: *const u8,
        font_length: usize,
        face_index: i32,
        font_size: f32,
        glyph_ids: *const u16,
        glyph_xy: *const f32,
        glyph_count: usize,
        origin_x: f32,
        origin_y: f32,
        red: f32,
        green: f32,
        blue: f32,
        alpha: f32,
        synthetic_bold: u8,
        synthetic_italic: u8,
        smoothing: u8,
    ) -> i32;
}

#[cfg(feature = "text")]
fn paint_glyph_run(
    buffer: &mut [u8],
    width: usize,
    height: usize,
    item: &DisplayItem,
    list: &DisplayItemList,
) {
    use layoutng_assembly::internal::layout_input::FontSmoothing;
    let run = item.glyph_run.as_ref().expect("glyph display item");
    let resources = list
        .resources
        .as_ref()
        .expect("glyph run requires font resources");
    let face = resources
        .fonts
        .get(run.font_face_index as usize)
        .expect("font face index is outside resources");
    let mut ids = Vec::with_capacity(run.glyphs.len());
    let mut xy = Vec::with_capacity(run.glyphs.len() * 2);
    for glyph in &run.glyphs {
        ids.push(u16::try_from(glyph.id).expect("Skia glyph ID exceeds u16"));
        xy.push(glyph.offset.x as f32);
        xy.push(glyph.offset.y as f32);
    }
    let smoothing = match run.font_smoothing {
        FontSmoothing::kNone => 1,
        FontSmoothing::kAntialiased => 2,
        FontSmoothing::kAuto | FontSmoothing::kSubpixelAntialiased => 0,
    };
    let ok = unsafe {
        LayoutngDrawGlyphRun(
            buffer.as_mut_ptr(),
            width as i32,
            height as i32,
            face.bytes.as_ptr(),
            face.bytes.len(),
            (face.face_index & 0xffff) as i32,
            run.font_size as f32,
            ids.as_ptr(),
            xy.as_ptr(),
            ids.len(),
            item.text_blob_origin.x as f32,
            item.text_blob_origin.y as f32,
            item.color.red,
            item.color.green,
            item.color.blue,
            item.color.alpha,
            u8::from(run.synthetic_bold),
            u8::from(run.synthetic_italic),
            smoothing,
        )
    };
    assert_eq!(ok, 1, "Skia rejected the shaped glyph run");
}

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
            #[cfg(feature = "text")]
            paint_glyph_run(&mut rgba, width, height, item, items);
            #[cfg(not(feature = "text"))]
            panic!("text replay is not installed");
            #[allow(unreachable_code)]
            continue;
        }
        let color = item.color;
        assert_eq!(
            color.alpha, 1.0,
            "translucent display item requires compositing"
        );
        skia::compat::pixmap_draw::opaque_rect::paint_rect(
            &mut rgba,
            width,
            height,
            skia::compat::commands::PaintRect {
                x: item.rect.x,
                y: item.rect.y,
                width: item.rect.width,
                height: item.rect.height,
            },
            [
                byte(color.red),
                byte(color.green),
                byte(color.blue),
                byte(color.alpha),
            ],
        );
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
    std::fs::write(path, skia::compat::png::EncodeRgbaPng(&rgba, width, height))
}

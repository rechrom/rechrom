use browser::page::{Page, PageClient};
use std::{cell::RefCell, rc::Rc};

struct Client;
impl PageClient for Client {}

fn open(url: &str, width: u32, height: u32, scale: f64) -> Page {
    let options = url_loader::DefaultURLLoaderOptions::default();
    let loader = Rc::new(RefCell::new(
        url_loader::DefaultURLLoader::new(options).unwrap(),
    ));
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(width, height);
    constraints.device_pixel_ratio = scale;
    let mut page = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(document_image::SVGImageDecoder::new(
            &assembly,
        ))),
        constraints,
        None,
        Some(Rc::new(RefCell::new(Client))),
    );
    page.Open(url, 16384, 4096).unwrap();
    while page.IsLoading() {
        page.RunTask().unwrap();
    }
    page
}

#[test]
fn probe_text_baseline() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open(
        "file:///private/tmp/text-baseline-fixture.html",
        640,
        400,
        2.0,
    );
    let frame = page.CurrentFrame().unwrap();
    {
        let owner = page.Document();
        let document = owner.GetDocument();
        for index in 0..document.NodeCount() {
            let node = document.Node(index);
            let Some(resolved) = document.ResolvedStyleFor(index) else {
                continue;
            };
            let native = unsafe { &*resolved.native_style.Get() };
            let font = unsafe { &*native.GetFont() };
            let primary = font.PrimaryFont();
            if !primary.is_null() {
                let metrics = unsafe { &*primary }.GetFontMetrics();
                eprintln!(
                    "FONT_METRICS node={} size={} float_ascent={} ascent={} float_descent={} descent={} line_spacing={}",
                    node.Id(),
                    native.ComputedFontSize(),
                    metrics.FloatAscent(),
                    metrics.Ascent(),
                    metrics.FloatDescent(),
                    metrics.Descent(),
                    metrics.LineSpacing(),
                );
            }
        }
    }
    for item in frame.display_items.items.iter() {
        if !item.glyphs.is_empty() {
            eprintln!(
                "TEXT_ITEM node={} rect={:?} origin={:?} line_top={} line_height={} size={} baseline={} glyphs={}",
                item.node_id,
                item.rect,
                item.text_blob_origin,
                item.text_line_top,
                item.text_line_height,
                item.font_size,
                item.baseline,
                item.glyphs.len()
            );
        }
    }
    let mut pixels = vec![0_u32; 1280 * 800];
    page.PaintArtifactInto(
        &frame.display_items,
        1280,
        800,
        2.0,
        &mut pixels,
        renderer::PixelFormat::Rgba8888,
        1280,
    )
    .unwrap();
    let bytes: Vec<u8> = pixels.into_iter().flat_map(u32::to_ne_bytes).collect();
    std::fs::write(
        "/private/tmp/rechrom-text-baseline.png",
        raster::EncodeRgbaPng(&bytes, 1280, 800),
    )
    .unwrap();
}

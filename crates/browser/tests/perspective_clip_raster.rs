//! Non-composited, plane-preserving perspective property-tree lowering.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct Loader(Vec<u8>);
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: self.0.clone(),
            ..Default::default()
        }))))
    }
}
fn render(perspective: &str, nested: bool) -> Vec<u32> {
    let inner = if nested {
        "transform:translate(13px,7px) rotate(13deg);"
    } else {
        ""
    };
    let html = format!(
        r#"<!doctype html><style>
        body{{margin:0;background:#89abcd}}
        .outer{{margin:20px;width:200px;height:140px;overflow:hidden;border-radius:9px}}
        .slides{{width:200px;height:140px;perspective:{perspective};perspective-origin:17.5px 8.25px;transform:translateX(0px)}}
        .slide{{width:180px;height:120px;overflow:hidden;{inner}}}
        .nested{{perspective:{perspective};width:190px;height:130px}}
        .clip{{overflow:hidden;width:170px;height:110px;border-radius:7px}}
        .content{{width:240px;height:180px;background:linear-gradient(135deg,red,blue)}}
        </style><div class="outer"><div class="slides"><div class="slide"><div class="nested"><div class="clip"><div class="content"></div></div></div></div></div></div>"#
    );
    let assembly = browser::CreateLayoutAssembly();
    let constraints = browser::CreateBrowserConstraints(320, 200);
    let decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader(html.into_bytes()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(decoder)),
        constraints,
        None,
        None,
    );
    page.Open("https://perspective-clip.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    let artifact = &page.CurrentFrame().unwrap().display_items;
    let mut pixels = vec![0; 320 * 200];
    page.PaintArtifactInto(
        artifact,
        320,
        200,
        1.0,
        &mut pixels,
        renderer_skia::PixelFormat::Rgba8888,
        320,
    )
    .unwrap();
    if let Ok(dir) = std::env::var("PERSPECTIVE_CLIP_EVIDENCE") {
        let path = std::path::Path::new(&dir);
        std::fs::create_dir_all(path).unwrap();
        let bytes: Vec<u8> = pixels
            .iter()
            .flat_map(|pixel| pixel.to_ne_bytes())
            .collect();
        std::fs::write(
            path.join(format!("{perspective}-{nested}.png")),
            raster::EncodeRgbaPng(&bytes, 320, 200),
        )
        .unwrap();
    }
    assert!(
        pixels.iter().any(|pixel| *pixel != pixels[0]),
        "must render real content"
    );
    pixels
}
#[test]
fn perspective_overflow_clips_keep_exact_planar_pixels() {
    let control = render("none", false);
    let perspective = render("1000px", false);
    assert_eq!(
        control
            .iter()
            .zip(&perspective)
            .filter(|(a, b)| a != b)
            .count(),
        0,
        "exact pixel parity"
    );
}
#[test]
fn nested_perspective_with_affine_descendant_keeps_exact_clips() {
    let control = render("none", true);
    let perspective = render("1000px", true);
    assert_eq!(
        control
            .iter()
            .zip(&perspective)
            .filter(|(a, b)| a != b)
            .count(),
        0,
        "exact pixel parity"
    );
}

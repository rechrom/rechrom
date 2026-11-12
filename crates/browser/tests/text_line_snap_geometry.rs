//! TextFragmentPainter rounds the accumulated CSS line top before device
//! projection. Fractional ancestors must therefore affect text paint geometry
//! identically at DPR 1 and DPR 2.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use layoutng_assembly::internal::layout_input::FontFace;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Operation(Option<URLResponse>);

impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader {
    padding_top: f64,
}

impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let body = format!(
            r#"<!doctype html><style>
              html,body {{ margin:0 }}
              body {{ padding-top:{}px }}
              #line {{ font:400 14px/21px SnapFixture }}
            </style><div id=line>Source aligned text</div>"#,
            self.padding_top
        );
        Ok(Box::new(Operation(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: body.into_bytes(),
            ..Default::default()
        }))))
    }
}

fn fixture(device_pixel_ratio: f64, padding_top: f64) -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(320, 160);
    constraints.device_pixel_ratio = device_pixel_ratio;
    constraints.fonts = vec![FontFace {
        family: "SnapFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader { padding_top })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        constraints,
        None,
        None,
    );
    page.Open("https://text-line-snap.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    assert!(page.CurrentFrame().is_some());
    page
}

fn assert_source_line_snap(device_pixel_ratio: f64, padding_top: f64) {
    let page = fixture(device_pixel_ratio, padding_top);
    let frame = page.CurrentFrame().expect("published text frame");
    let mut glyph_items = frame
        .display_items
        .items
        .iter()
        .filter(|item| !item.glyphs.is_empty());
    let item = glyph_items.next().expect("text glyph display item");
    assert!(glyph_items.next().is_none(), "one fixture glyph run");

    let line_top = item.text_line_top;
    assert!((line_top - padding_top).abs() <= 1.0 / 64.0);
    let expected_delta = (line_top + 0.5).floor() - line_top;
    let actual_delta = item.text_blob_origin.y - item.baseline - item.rect.y;
    assert!(
        (actual_delta - expected_delta).abs() <= 1.0 / 64.0,
        "DPR {device_pixel_ratio}: line top {line_top}, source delta {expected_delta}, recorded delta {actual_delta}"
    );

    // Font ascent is sampled and rounded in device space. Combined with the
    // CSS line-top snap, the final baseline must land on a device pixel at
    // both scale factors.
    let device_origin = item.text_blob_origin.y * device_pixel_ratio;
    assert!(
        (device_origin - device_origin.round()).abs() <= 1.0 / 64.0,
        "DPR {device_pixel_ratio}: non-integral device baseline {device_origin}"
    );
}

#[test]
fn browser_text_origin_uses_current_css_paint_space_at_dpr_one_and_two() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for scale in [1.0, 2.0] {
            assert_source_line_snap(scale, 20.5);
            assert_source_line_snap(scale, 20.0);
        }
    });
}

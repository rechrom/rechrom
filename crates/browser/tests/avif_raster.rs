//! Exercise the production DecodeEngine/image_decoder boundary, resource load,
//! and paint independently. PNG pixels are the pinned libyuv oracle, not a
//! second invocation of the Rust converter under test.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;
use browser::page::Page;
use image_decoder::{
    image_decoder::{DecodedImage, ImageDecodeInput, ImageDecoder},
    skia_image_decoder::SkiaImageDecoder,
};
use std::{
    cell::{Cell, RefCell},
    io,
    rc::Rc,
};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct Loader {
    html: Vec<u8>,
    image: Vec<u8>,
    mime: &'static str,
    image_requests: Rc<Cell<usize>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let (mime_type, body) = if request.url.ends_with("fixture.image") {
            self.image_requests.set(self.image_requests.get() + 1);
            (self.mime, self.image.clone())
        } else {
            ("text/html", self.html.clone())
        };
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: mime_type.into(),
            body,
            ..Default::default()
        }))))
    }
}
struct Decoder {
    calls: Rc<Cell<usize>>,
}
impl ImageDecoder for Decoder {
    fn Decode(&mut self, input: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
        self.calls.set(self.calls.get() + 1);
        SkiaImageDecoder.Decode(input)
    }
}
fn render(avif: bool, scale: f64, background: &str) -> Vec<u32> {
    let calls = Rc::new(Cell::new(0));
    let requests = Rc::new(Cell::new(0));
    let encoded = if avif {
        include_bytes!("../../image_decoder/tests/fixtures/avif-source/alpha.avif").as_slice()
    } else {
        include_bytes!("../../image_decoder/tests/fixtures/avif-source/alpha-libyuv.png").as_slice()
    };
    let markup=format!("<!doctype html><style>body{{margin:0;background:{background}}}img{{display:block;position:absolute;left:17.5px;top:9.25px;width:93px;height:87px}}</style><img src='fixture.image'>");
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(160, 120);
    constraints.device_pixel_ratio = scale;
    let svg = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader {
            html: markup.into_bytes(),
            image: encoded.to_vec(),
            mime: if avif { "image/avif" } else { "image/png" },
            image_requests: requests.clone(),
        })),
        Rc::new(RefCell::new(Decoder {
            calls: calls.clone(),
        })),
        Rc::new(RefCell::new(svg)),
        constraints,
        None,
        None,
    );
    page.Open("https://avif-raster.test/", 16384, 4096).unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading(), "image resource load must settle");
    assert_eq!(calls.get(), 1, "one production decoder invocation");
    assert_eq!(requests.get(), 1, "one encoded resource fetch");
    let frame = page.CurrentFrame().unwrap();
    assert!(
        !frame
            .display_items
            .resources
            .iter()
            .flat_map(|r| &r.images)
            .collect::<Vec<_>>()
            .is_empty(),
        "decoded image reaches paint resources"
    );
    let width = (160.0 * scale) as u32;
    let height = (120.0 * scale) as u32;
    let mut pixels = vec![0; width as usize * height as usize];
    page.PaintArtifactInto(
        &frame.display_items,
        width,
        height,
        scale,
        &mut pixels,
        renderer_skia::PixelFormat::Rgba8888,
        width as usize,
    )
    .unwrap();
    assert!(
        pixels.iter().any(|p| *p != pixels[0]),
        "image artwork actually paints"
    );
    if let Ok(dir) = std::env::var("AVIF_RASTER_EVIDENCE") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        let bytes: Vec<u8> = pixels.iter().flat_map(|p| p.to_ne_bytes()).collect();
        std::fs::write(
            dir.join(format!(
                "{}-{scale}-{}.png",
                if avif { "avif" } else { "png" },
                background.replace('#', "")
            )),
            raster::EncodeRgbaPng(&bytes, width, height),
        )
        .unwrap();
    }
    pixels
}
#[test]
fn avif_loads_and_paints_exactly_like_independent_rgb_oracle() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for scale in [1.0, 1.25, 2.0] {
            for background in ["white", "#24384c"] {
                let a = render(true, scale, background);
                let b = render(false, scale, background);
                let changed = a.iter().zip(&b).filter(|(a, b)| a != b).count();
                eprintln!(
                    "AVIF_RASTER scale={scale} background={background} pixels={} changed={changed}",
                    a.len()
                );
                assert_eq!(
                    changed, 0,
                    "AVIF/oracle paint mismatch scale={scale} background={background}"
                );
            }
        }
    });
}

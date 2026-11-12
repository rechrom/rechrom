//! LocalFrameView base-background ownership at the embedding boundary.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use layoutng_assembly::internal::layout_input_types::Color;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Operation(Option<URLResponse>);

impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct FixtureLoader;

impl URLLoader for FixtureLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: b"<!doctype html><style>html,body{margin:0;background:transparent}</style><body><div style='width:1px;height:1px'></div></body>".to_vec(),
            ..Default::default()
        }))))
    }
}

fn page() -> Page {
    let assembly = browser::CreateLayoutAssembly();
    Page::Create(
        Rc::new(RefCell::new(FixtureLoader)),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        browser::CreateBrowserConstraints(160, 96),
        None,
        None,
    )
}

fn first_pixel(page: &Page) -> [u8; 4] {
    let pixels = raster::RasterizePaintArtifactTransparent(
        &page.CurrentFrame().unwrap().display_items,
        160,
        96,
    );
    pixels[..4].try_into().unwrap()
}

fn open(page: &mut Page) {
    page.Apply(page_mutation::PageMutation::DOMMutation(
        dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kParseDocument,
            value: "<!doctype html><style>html,body{margin:0;background:transparent}</style><body><div style='width:1px;height:1px'></div></body>".into(),
            ..Default::default()
        },
    ))
    .unwrap();
    assert!(page.CurrentFrame().is_some());
}

#[test]
fn base_background_is_opaque_by_default_and_embedder_configurable() {
    test_thread::run(|| {
        let mut ordinary = page();
        open(&mut ordinary);
        assert_eq!(first_pixel(&ordinary), [255, 255, 255, 255]);

        let mut embedded = page();
        embedded.SetShouldPaintBaseBackgroundColor(false);
        open(&mut embedded);
        assert_eq!(first_pixel(&embedded), [0, 0, 0, 0]);

        let mut colored = page();
        colored.SetBaseBackgroundColor(Color {
            red: 0.75,
            green: 0.25,
            blue: 0.5,
            alpha: 1.0,
        });
        open(&mut colored);
        assert_eq!(first_pixel(&colored), [191, 64, 128, 255]);
        let colored_base = colored
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .find(|item| {
                item.color.red == 0.75 && item.color.green == 0.25 && item.color.blue == 0.5
            })
            .expect("custom base background draw");
        assert!(colored_base.source_overwrite);

        let mut source_over = page();
        source_over.SetBaseBackgroundColor(Color {
            red: 0.75,
            green: 0.25,
            blue: 0.5,
            alpha: 1.0,
        });
        source_over.SetShouldClearDocumentBackground(false);
        open(&mut source_over);
        let source_over_base = source_over
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .find(|item| {
                item.color.red == 0.75 && item.color.green == 0.25 && item.color.blue == 0.5
            })
            .expect("source-over base background draw");
        assert!(!source_over_base.source_overwrite);
    });
}

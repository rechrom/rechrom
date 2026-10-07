//! Frozen independent source Browser::Open evidence. The URL loader is a
//! deterministic test transport; parser, CSS, decoding, fonts, layout and paint
//! are the production Rust implementations.
use crate::script_scheduler::{ScriptLoadClient, ScriptScheduler};
use document_image::SVGImageDecoder;
use document_loader::{ResourceFetcher, ResourceFetcherClient};
use dom::DOM;
use image_decoder::skia_image_decoder::SkiaImageDecoder;
use javascript::{
    javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
};
use layoutng_assembly::internal::layout_input::{ConstraintSpace, FontFace};
use page_mutation::ResourceMutation;
use std::{cell::RefCell, io, path::PathBuf, rc::Rc};
use url_loader::{RequestDestination, URLLoadOperation, URLLoader, URLRequest, URLResponse};
use webapi::dom_bindings::DOMJavaScriptBindings;

struct Operation {
    response: Option<URLResponse>,
    remaining: usize,
}
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        if self.remaining != 0 {
            self.remaining -= 1;
            return Ok(None);
        }
        Ok(self.response.take())
    }
}
struct Loader {
    root: PathBuf,
    trace: Rc<RefCell<Vec<String>>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.trace.borrow_mut().push(format!(
            "request\t{}\t{}\t{}",
            request.destination as u8, request.url, request.referrer
        ));
        let name = request.url.rsplit('/').next().unwrap();
        if name == "missing.ttf" {
            return Err(io::Error::other("fixture font unavailable"));
        }
        if name == "unused.ttf" {
            return Err(io::Error::other("unused font requested"));
        }
        let (path, mime, final_url, remaining) = match name {
            "entry" => (
                self.root.join("page.html"),
                "text/html",
                "https://page.test/redirect/page.html".into(),
                1,
            ),
            "sheet.css" => (
                self.root.join(name),
                "text/css",
                "https://cdn.test/css/sheet.css".into(),
                2,
            ),
            "font.ttf" => (
                PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
                )),
                "font/ttf",
                request.url.clone(),
                1,
            ),
            _ => (
                self.root.join(name),
                "image/svg+xml",
                request.url.clone(),
                1,
            ),
        };
        Ok(Box::new(Operation {
            remaining,
            response: Some(URLResponse {
                body: std::fs::read(path)?,
                mime_type: mime.into(),
                final_url,
                status_code: 200,
                ..Default::default()
            }),
        }))
    }
}
struct Client {
    document: Rc<RefCell<DOM>>,
    constraints: Rc<RefCell<ConstraintSpace>>,
    trace: Rc<RefCell<Vec<String>>>,
}
impl ScriptLoadClient for Client {
    fn DidReportScriptError(&mut self, e: &javascript::javascript_runtime::JavaScriptException) {
        panic!("unexpected script error: {}", e.message);
    }
    fn DidFailResource(&mut self, url: &str, error: &str) {
        self.trace
            .borrow_mut()
            .push(format!("failure\t{url}\t{error}"));
    }
}
impl ResourceFetcherClient for Client {
    fn ApplyResourceMutation(&mut self, mutation: ResourceMutation) {
        match mutation {
            ResourceMutation::ImageResourceReady(value) => {
                let mut owner = self.document.borrow_mut();
                dom::image_resource::AddImageResource(
                    &mut owner,
                    &mut self.constraints.borrow_mut(),
                    value.source,
                    value.image,
                );
                owner.GetDocumentMut().ClearResolvedStyles();
            }
            ResourceMutation::FontResourceReady(value) => {
                self.constraints.borrow_mut().fonts.push(value.font)
            }
            ResourceMutation::ResourceLoadFailed(value) => {
                self.DidFailResource(&value.url, &value.error)
            }
        }
    }
    fn HasJavaScript(&self) -> bool {
        false
    }
    fn DispatchImageEvent(&mut self, _: u64, _: bool) {
        panic!("source no-script fixture emits no image events");
    }
}

#[test]
fn parser_resource_fetcher_and_frame_match_unchanged_cpp_browser() {
    crate::native_test_thread::run(body);
}
fn body() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../artifacts/cpp-reference/page-resources");
    let trace = Rc::new(RefCell::new(Vec::new()));
    let loader: Rc<RefCell<dyn URLLoader>> = Rc::new(RefCell::new(Loader {
        root: root.clone(),
        trace: trace.clone(),
    }));
    let response = document_loader::LoadResponse(
        &mut *loader.borrow_mut(),
        &URLRequest {
            url: "https://page.test/entry".into(),
            destination: RequestDestination::kDocument,
            ..Default::default()
        },
    )
    .unwrap();
    let current_url = response.final_url.clone();
    let document = Rc::new(RefCell::new(DOM::new()));
    let mut constraints = crate::CreateBrowserConstraints(160, 96);
    constraints.fonts = vec![FontFace {
        family: "sans-serif".into(),
        bytes: include_bytes!(
            "../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        )
        .as_slice()
        .into(),
        ..Default::default()
    }];
    let constraints = Rc::new(RefCell::new(constraints));
    let assembly = crate::CreateLayoutAssembly();
    let resources = Rc::new(ResourceFetcher::new(
        loader.clone(),
        Rc::new(RefCell::new(SkiaImageDecoder)),
        Rc::new(RefCell::new(SVGImageDecoder::new(&assembly))),
        document.clone(),
        constraints.clone(),
        current_url.clone(),
    ));
    let changed = document.clone();
    let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document.clone(),
        Box::new(move |mutation| {
            crate::dom_mutation::ApplyDOMTreeMutation(&mut changed.borrow_mut(), mutation);
        }),
    )));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(bindings.clone());
    assert!(runtime
        .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
        .Succeeded());
    let mut scheduler =
        ScriptScheduler::new(document.clone(), bindings, loader, current_url.clone());
    scheduler.InstallResourceFetcher(resources.clone()).unwrap();
    let mut client = Client {
        document: document.clone(),
        constraints: constraints.clone(),
        trace: trace.clone(),
    };
    scheduler
        .ParseDocument(
            &document_loader::DecodeText(&response).unwrap(),
            13,
            3,
            &mut runtime,
            &realm,
            &mut client,
        )
        .unwrap();
    scheduler
        .FinishStyleSheets(None, &mut runtime, &realm, &mut client)
        .unwrap();
    crate::style_services::ResolveLayoutStyles(&mut document.borrow_mut(), &constraints.borrow());
    resources.QueueReferencedImages().unwrap();
    resources.DiscardUnusedFontFaces();
    resources.StartPendingFonts();
    resources.LoadPendingImages(&mut client);
    resources.LoadPendingFonts(&mut client);
    crate::style_services::ResolveLayoutStyles(&mut document.borrow_mut(), &constraints.borrow());
    let mut layout_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
    let fragments = crate::LayoutPersistentDocument(
        &mut layout_engine,
        &mut document.borrow_mut(),
        &dom::UserInteractionState::default(),
        &constraints.borrow(),
    );
    let items = paint::paint_engine::Paint(&fragments);
    trace.borrow_mut().push(format!("url\t{current_url}"));
    for source in [
        "rect.svg",
        "missing.svg",
        "https://page.test/assets/rect.svg",
    ] {
        let owner = document.borrow();
        let line = match owner.GetDocument().ImageResourceFor(source) {
            Some(image) => format!(
                "image\t{source}\t{}\t{}\t{}\t{}",
                image.id, image.natural_width, image.natural_height, image.resolution_scale
            ),
            None => format!("image\t{source}\tmissing"),
        };
        trace.borrow_mut().push(line);
    }
    let catalog = items.resources.as_ref().unwrap();
    for font in &catalog.fonts {
        trace.borrow_mut().push(format!(
            "font\t{}\t{}\t{}\t{}",
            font.family,
            font.weight,
            u8::from(font.italic),
            font.bytes.len()
        ));
    }
    for image in &catalog.images {
        let bytes = image.BitmapPixels().map_or_else(
            || format!("document-revision-{}", image.revision),
            |pixels| {
                pixels
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            },
        );
        trace.borrow_mut().push(format!(
            "pixels\t{}\t{}\t{}\t{bytes}",
            image.id, image.width, image.height
        ));
    }
    let reference = include_str!("../../../artifacts/cpp-reference/page-resources/results.tsv");
    let pixels = renderer::pure_replay::RasterizeDisplayItemList(&items, 160, 96);
    let expected = include_bytes!("../../../artifacts/cpp-reference/page-resources/frame.rgba");
    let differing = pixels
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    std::fs::write(
        root.join("results-rust.tsv"),
        trace.borrow().join("\n") + "\n",
    )
    .unwrap();
    std::fs::write(root.join("frame-rust.rgba"), &pixels).unwrap();
    eprintln!(
        "source full-resource fixture: {differing}/{} pixels differ; resource-state-equal={}",
        expected.len() / 4,
        trace.borrow().join("\n") + "\n" == reference
    );
    assert_eq!(
        trace.borrow().join("\n") + "\n",
        reference,
        "native source requests/resource state"
    );
    assert_eq!(pixels.len(), expected.len());
    assert_eq!(differing, 0, "full resource fixture pixels differ");
}

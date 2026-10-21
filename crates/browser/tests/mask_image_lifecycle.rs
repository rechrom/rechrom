//! Mask URL resource lifecycle through the real document/Style/Layout/Paint pipeline.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::internal::{
    layout_object::LayoutObject, layout_pass_scope::LayoutObjectFactoryScope,
};
use page_mutation::PageMutation;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

#[derive(Clone, Copy)]
enum Outcome {
    Pending,
    Loaded,
    TransportError,
    DecodeError,
}
struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct ImageResponse {
    outcome: Rc<RefCell<Outcome>>,
    url: String,
    consumed: bool,
}
impl URLLoadOperation for ImageResponse {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        if self.consumed {
            return Ok(None);
        }
        let outcome = *self.outcome.borrow();
        if matches!(outcome, Outcome::Pending) {
            return Ok(None);
        }
        self.consumed = true;
        if matches!(outcome, Outcome::TransportError) {
            return Err(io::Error::other("controlled image failure"));
        }
        Ok(Some(URLResponse {
            final_url: self.url.clone(),
            status_code: 200,
            mime_type: if matches!(outcome, Outcome::DecodeError) {
                "image/png"
            } else {
                "image/svg+xml"
            }
            .into(),
            body: if matches!(outcome, Outcome::DecodeError) {
                b"broken".to_vec()
            } else {
                br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="50" height="100" fill="black"/></svg>"#.to_vec()
            },
            ..Default::default()
        }))
    }
}
struct Loader {
    html: String,
    outcome: Rc<RefCell<Outcome>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        if request.url.ends_with("fixture.svg") {
            return Ok(Box::new(ImageResponse {
                outcome: self.outcome.clone(),
                url: request.url.clone(),
                consumed: false,
            }));
        }
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: self.html.as_bytes().to_vec(),
            ..Default::default()
        }))))
    }
}
fn fixture(
    layers: &str,
    composite: &str,
    outcome: Outcome,
    pseudo: bool,
) -> (Page, Rc<RefCell<Outcome>>) {
    let outcome = Rc::new(RefCell::new(outcome));
    let selector = if pseudo { "#target::before" } else { "#target" };
    let html = format!(
        r#"<!doctype html><style>body{{margin:0;background:white}}{selector}{{content:"";display:block;width:100px;height:100px;background:red;mask-image:{layers};mask-size:100px 100px;mask-repeat:no-repeat;mask-composite:{composite}}}</style><div id="target"></div>"#
    );
    let assembly = browser::CreateLayoutAssembly();
    let constraints = browser::CreateBrowserConstraints(160, 120);
    let decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader {
            html,
            outcome: outcome.clone(),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(decoder)),
        constraints,
        None,
        None,
    );
    page.Open("https://mask.test/", 16384, 4096).unwrap();
    settle(&mut page);
    (page, outcome)
}
fn settle(page: &mut Page) {
    for _ in 0..20 {
        page.RunTask().unwrap();
    }
    assert!(
        page.CurrentFrame().is_some(),
        "pending resources still permit a frame"
    );
}
fn native(page: &Page) -> &LayoutObject {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let wanted = (0..doc.NodeCount())
        .find_map(|i| {
            doc.Node(i)
                .FindAttribute("id")
                .filter(|a| a.value == "target")
                .map(|_| doc.Node(i).Id())
        })
        .unwrap();
    let assembly = browser::CreateLayoutAssembly();
    let _scope = LayoutObjectFactoryScope::new(&assembly.objects);
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let n = current.GetNode();
        if !n.is_null() && unsafe { &*n }.InputId() == wanted {
            return current;
        }
        object = current.NextInPreOrder(root);
    }
    panic!("target layout object missing")
}
fn pixels(page: &Page) -> Vec<u32> {
    artifact_pixels(page, &page.CurrentFrame().unwrap().display_items)
}
fn artifact_pixels(
    page: &Page,
    artifact: &std::sync::Arc<paint::paint_engine::PaintArtifact>,
) -> Vec<u32> {
    let mut pixels = vec![0; 160 * 120];
    page.PaintArtifactInto(
        artifact,
        160,
        120,
        1.0,
        &mut pixels,
        renderer_skia::PixelFormat::Rgba8888,
        160,
    )
    .unwrap();
    pixels
}
fn assert_ink(page: &Page, left: [u8; 4], right: [u8; 4]) {
    let p = pixels(page);
    assert_eq!(p[25 * 160 + 25].to_ne_bytes(), left);
    assert_eq!(p[25 * 160 + 75].to_ne_bytes(), right);
}
const WHITE: [u8; 4] = [255, 255, 255, 255];
const RED: [u8; 4] = [255, 0, 0, 255];

#[test]
fn unavailable_mask_is_hidden_without_a_fake_resource() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for outcome in [
            Outcome::Pending,
            Outcome::TransportError,
            Outcome::DecodeError,
        ] {
            let (page, _) = fixture("url(fixture.svg)", "add", outcome, false);
            assert_ink(&page, WHITE, WHITE);
            let style = native(&page).StyleRef();
            let image = unsafe { &*style.MaskLayers().GetImage() };
            assert!(!image.CanRender());
            assert_eq!(image.IsLoading(), matches!(outcome, Outcome::Pending));
            assert_eq!(image.IsLoaded(), !matches!(outcome, Outcome::Pending));
            assert_eq!(image.ErrorOccurred(), !matches!(outcome, Outcome::Pending));
            assert!(page
                .CurrentFrame()
                .unwrap()
                .display_items
                .resources
                .as_ref()
                .unwrap()
                .images
                .is_empty());
        }
    });
}
#[test]
fn pending_mask_load_reveals_and_retains_native_object_and_old_artifact() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (mut page, outcome) = fixture("url(fixture.svg)", "add", Outcome::Pending, false);
        let object = native(&page) as *const LayoutObject;
        let old = page.CurrentFrame().unwrap().display_items.clone();
        let old_pixels = pixels(&page);
        assert_ink(&page, WHITE, WHITE);
        *outcome.borrow_mut() = Outcome::Loaded;
        settle(&mut page);
        assert!(!page.IsLoading());
        assert_ink(&page, RED, WHITE);
        assert_eq!(native(&page) as *const LayoutObject, object);
        assert_eq!(artifact_pixels(&page, &old), old_pixels);
        let loaded = page.CurrentFrame().unwrap().display_items.clone();
        let loaded_pixels = pixels(&page);
        let owner = page.Document();
        let doc = owner.GetDocument();
        let index = (0..doc.NodeCount())
            .find(|i| {
                doc.Node(*i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "target")
            })
            .unwrap();
        let id = doc.Node(index).Id();
        drop(owner);
        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: id,
            name: "style".into(),
            value: "mask-image:none".into(),
            ..Default::default()
        }))
        .unwrap();
        settle(&mut page);
        assert_ink(&page, RED, RED);
        assert_eq!(native(&page) as *const LayoutObject, object);
        assert_eq!(artifact_pixels(&page, &loaded), loaded_pixels);
    });
}
#[test]
fn pending_layer_suppresses_other_loaded_layers_until_failure() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (mut page, outcome) = fixture(
            "linear-gradient(black,black),url(fixture.svg)",
            "add",
            Outcome::Pending,
            false,
        );
        let object = native(&page) as *const LayoutObject;
        assert_ink(&page, WHITE, WHITE);
        *outcome.borrow_mut() = Outcome::TransportError;
        settle(&mut page);
        assert!(!page.IsLoading());
        assert_ink(&page, RED, RED);
        assert_eq!(native(&page) as *const LayoutObject, object);
    });
}
#[test]
fn pseudo_mask_failure_updates_the_actual_style_consumer() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let (mut page, outcome) = fixture(
            "linear-gradient(black,black),url(fixture.svg)",
            "add",
            Outcome::Pending,
            true,
        );
        assert_ink(&page, WHITE, WHITE);
        *outcome.borrow_mut() = Outcome::DecodeError;
        settle(&mut page);
        assert_ink(&page, RED, RED);
    });
}

#[test]
fn non_renderable_slots_keep_native_layer_position_and_compositing() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        // Chromium skips a failed/none source before selecting its operator.
        // A renderable upper source still keeps its operation when the native
        // bottom slot cannot render; it must not be promoted to bottom.
        for missing in ["none", "url(fixture.svg)"] {
            for composite in ["add", "subtract", "intersect", "exclude"] {
                let (top_missing, _) = fixture(
                    &format!("{missing},linear-gradient(black,black)"),
                    composite,
                    Outcome::DecodeError,
                    false,
                );
                assert_ink(&top_missing, RED, RED);
                let (bottom_missing, _) = fixture(
                    &format!("linear-gradient(black,black),{missing}"),
                    composite,
                    Outcome::DecodeError,
                    false,
                );
                let expected = if composite == "intersect" { WHITE } else { RED };
                assert_ink(&bottom_missing, expected, expected);
            }
        }
        let (no_mask, _) = fixture("none", "intersect", Outcome::Pending, false);
        assert_ink(&no_mask, RED, RED);
    });
}

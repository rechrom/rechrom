//! Native SVG no-layer ownership must survive the Layout to Paint boundary.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::{
        layout_input::FontFace, layout_object::LayoutObject,
        layout_pass_scope::LayoutObjectFactoryScope,
    },
};
use page_mutation::PageMutation;
use paint::paint_engine::PaintArtifact;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader {
    html: Vec<u8>,
    image: Option<Vec<u8>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let (mime_type, body) = if request.url.ends_with("fixture.svg") {
            (
                "image/svg+xml",
                self.image.as_ref().expect("fixture SVG").clone(),
            )
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

fn page_fixture(markup: &str, image: Option<&str>) -> Page {
    let html =
        format!("<!doctype html><style>body{{margin:0}}svg,img{{display:block}}</style>{markup}");
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(320, 240);
    constraints.fonts = vec![FontFace {
        family: "SvgPaintOwnershipFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader {
            html: html.into_bytes(),
            image: image.map(|svg| svg.as_bytes().to_vec()),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(decoder)),
        constraints,
        None,
        None,
    );
    page.Open("https://svg-paint-ownership.test/", 16384, 4096)
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

fn fixture(svg: &str) -> Page {
    page_fixture(svg, None)
}

fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let doc = owner.GetDocument();
    (0..doc.NodeCount())
        .find_map(|index| {
            doc.Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == name)
                .map(|_| doc.Node(index).Id())
        })
        .unwrap_or_else(|| panic!("missing DOM node {name}"))
}

fn native<'a>(page: &'a Page, name: &str) -> &'a LayoutObject {
    let wanted = id(page, name);
    let assembly = browser::CreateLayoutAssembly();
    let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == wanted {
            return current;
        }
        object = current.NextInPreOrder(root);
    }
    panic!("missing native layout object {name}")
}

fn find_fragment(node: &FragmentNode, wanted: u64) -> Option<&FragmentNode> {
    if node.node_id == wanted {
        Some(node)
    } else {
        node.children
            .iter()
            .find_map(|child| find_fragment(child, wanted))
    }
}

fn fragment<'a>(page: &'a Page, name: &str) -> &'a FragmentNode {
    find_fragment(&page.CurrentFrame().unwrap().fragments, id(page, name))
        .unwrap_or_else(|| panic!("missing published fragment {name}"))
}

fn set_attribute(page: &mut Page, name: &str, attribute: &str, value: &str) {
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetAttribute,
        target_node_id: id(page, name),
        name: attribute.into(),
        value: value.into(),
        ..Default::default()
    }))
    .unwrap();
}

const GRAPHIC: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="80"><g id="group"><rect id="back" width="50" height="50" fill="red" transform="translate(5 5)"/><path id="front" d="M10 10H40V40H10Z" fill="blue"/></g></svg>"##;

fn pixels(page: &Page, artifact: &std::sync::Arc<PaintArtifact>) -> Vec<u32> {
    let mut result = vec![0; 320 * 240];
    page.PaintArtifactInto(
        artifact,
        320,
        240,
        1.0,
        &mut result,
        renderer_skia::PixelFormat::Rgba8888,
        320,
    )
    .unwrap();
    result
}

#[test]
fn native_svg_no_layer_keeps_sibling_paint_order_and_old_frames() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(GRAPHIC);
        for name in ["group", "back", "front"] {
            assert!(native(&page, name).IsSVGChild());
            let f = fragment(&page, name);
            assert!(f.paint.paint_layer_state_is_native);
            assert_eq!(
                f.paint.paint_layer_client_id, 0,
                "SVG descendant has no native layer"
            );
            assert!(
                !paint::paint_order::ClassifyFragmentPaintLayer(f, false, false).is_paint_layer,
                "native no-layer ownership was replaced for {name}"
            );
        }
        let old = page.CurrentFrame().unwrap().display_items.clone();
        let before = pixels(&page, &old);
        assert_eq!(before[20 * 320 + 20].to_ne_bytes(), [0, 0, 255, 255]);
        set_attribute(&mut page, "back", "transform", "translate(0 0)");
        let after = pixels(&page, &page.CurrentFrame().unwrap().display_items);
        assert_eq!(after[20 * 320 + 20].to_ne_bytes(), [0, 0, 255, 255]);
        assert_ne!(
            before, after,
            "transform mutation changes exposed red edges"
        );
        assert_eq!(
            pixels(&page, &old),
            before,
            "old artifact remains immutable"
        );
        set_attribute(&mut page, "group", "transform", "translate(10 10)");
        let moved = pixels(&page, &page.CurrentFrame().unwrap().display_items);
        assert_eq!(moved[30 * 320 + 30].to_ne_bytes(), [0, 0, 255, 255]);
        assert_eq!(pixels(&page, &old), before);
    });
}

#[test]
fn fetched_svg_keeps_transformed_shapes_in_document_order() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let p = page_fixture(r#"<img id="image" src="fixture.svg">"#, Some(GRAPHIC));
        let actual = pixels(&p, &p.CurrentFrame().unwrap().display_items);
        assert_eq!(
            actual[20 * 320 + 20].to_ne_bytes(),
            [0, 0, 255, 255],
            "foreground is covered by earlier transformed rectangle"
        );
    });
}

#[test]
fn native_svg_mask_keeps_the_existing_shared_recording_scope() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r#"<svg width="80" height="80"><svg id="masked" width="60" height="60" style="mask-image:linear-gradient(black,black)"><rect width="40" height="40" fill="red"/></svg></svg>"#,
        );
        let masked = fragment(&page, "masked");
        assert!(native(&page, "masked").IsSVGChild());
        assert!(masked.paint.paint_layer_state_is_native);
        assert_eq!(masked.paint.paint_layer_client_id, 0);
        assert!(!masked.paint.style.mask_images.is_empty());
        // Mask drawing is still owned by PaintLayerPainter in the existing
        // adapter. Keep the same shared classification consumed by Interaction
        // until source SVG mask effect scopes no longer need a synthetic layer.
        let classification = paint::paint_order::ClassifyFragmentPaintLayer(masked, false, false);
        assert!(classification.is_paint_layer && classification.is_stacking_context);
        let artifact = &page.CurrentFrame().unwrap().display_items;
        assert!(artifact.items.iter().any(|item| {
            item.node_id == id(&page, "masked")
                && item.r#type == paint::paint_engine::DisplayItemType::kSaveLayerDstIn
        }));
    });
}

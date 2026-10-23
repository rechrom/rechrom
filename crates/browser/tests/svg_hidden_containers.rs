//! SVG definitions retain layout/resource identity without ordinary paint or hit targets.
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
        family: "SvgHiddenContainerFixture".into(),
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
    page.Open("https://svg-hidden-container.test/", 16384, 4096)
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

const HIDDEN: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80"><rect id="visible" width="20" height="20" fill="blue"/><defs id="defs"><g id="group"><rect id="definition" x="30" width="40" height="40" fill="red"/></g></defs><clipPath id="clip"><rect id="clip-shape" x="70" width="40" height="40" fill="red"/></clipPath></svg>"##;

#[test]
fn hidden_svg_native_kinds_keep_descendant_layout_and_empty_visual_geometry() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(HIDDEN);
        let assembly = browser::CreateLayoutAssembly();
        let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
        for (name, resource, expected) in [
            (
                "defs",
                false,
                layoutng_assembly::internal::layout_input::NodeKind::kSvgHiddenContainer,
            ),
            (
                "clip",
                true,
                layoutng_assembly::internal::layout_input::NodeKind::kSvgResourceClipper,
            ),
        ] {
            let object = native(&page, name);
            assert!(object.IsSVGHiddenContainer());
            assert!(object.IsSVGContainer());
            assert_eq!(object.IsSVGResourceContainer(), resource);
            assert!(!object.IsSVGTransformableContainer());
            assert!(object.VisualRectInLocalSVGCoordinates().IsEmpty());
            let fragment = fragment(&page, name);
            assert!(fragment.paint.svg_hidden_container);
            assert_eq!(fragment.paint.source_kind, expected);
            assert!(!fragment.children.is_empty());
        }
        for name in ["definition", "clip-shape"] {
            let object = native(&page, name);
            assert!(object.IsSVGShape());
            assert!(!object.NeedsLayout());
            assert!(fragment(&page, name).paint.svg_shape.is_some());
        }
    });
}

#[test]
fn hidden_svg_subtrees_never_paint_or_hit_and_stay_hidden_after_mutations() {
    use interaction::input_event::{InputEvent, MouseEvent, MouseEventType};
    use layoutng_assembly::internal::layout_input::Offset;
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(HIDDEN);
        let old = page.CurrentFrame().unwrap().display_items.clone();
        let old_pixels = pixels(&page, &old);
        assert_eq!(old_pixels[10 * 320 + 10].to_ne_bytes(), [0, 0, 255, 255]);
        for x in [40, 80] {
            assert_eq!(old_pixels[10 * 320 + x].to_ne_bytes(), [255, 255, 255, 255]);
            let target = page
                .Dispatch(&InputEvent::Mouse(MouseEvent {
                    r#type: MouseEventType::kMove,
                    position: Offset {
                        x: x as f64,
                        y: 10.0,
                    },
                    ..Default::default()
                }))
                .unwrap()
                .target_node_id;
            assert_ne!(target, Some(id(&page, "definition")));
            assert_ne!(target, Some(id(&page, "clip-shape")));
        }
        for name in ["defs", "group", "definition", "clip", "clip-shape"] {
            assert!(
                old.items.iter().all(|item| item.node_id != id(&page, name)),
                "ordinary paint owner escaped: {name}"
            );
        }
        set_attribute(&mut page, "definition", "width", "60");
        set_attribute(
            &mut page,
            "clip-shape",
            "style",
            "opacity:0.5;fill:green;transform:translate(5px,5px)",
        );
        assert_eq!(
            pixels(&page, &page.CurrentFrame().unwrap().display_items),
            old_pixels
        );
        assert_eq!(pixels(&page, &old), old_pixels);
        assert!(fragment(&page, "definition").paint.svg_shape.is_some());
    });
}

#[test]
fn fetched_svg_hidden_definitions_match_inline_recording() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let inline = fixture(HIDDEN);
        let fetched = page_fixture(
            r#"<img src="fixture.svg" width="120" height="80">"#,
            Some(HIDDEN),
        );
        assert_eq!(
            pixels(&inline, &inline.CurrentFrame().unwrap().display_items),
            pixels(&fetched, &fetched.CurrentFrame().unwrap().display_items),
        );
    });
}

#[test]
fn generated_pseudos_bind_only_referenced_clip_resources_in_the_originating_scope() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for referenced in [false, true] {
            let binding_rules = if referenced {
                "#before,#letter,#marker,#placeholder{clip-path:url(#resource)} #before::before{clip-path:inherit}"
            } else {
                ""
            };
            let markup = r##"
            <style>
              body { font-family: SvgHiddenContainerFixture; }
              #before::before { content: 'B'; }
              #letter::first-letter { color: red; }
              #marker::marker { content: 'M'; }
              #placeholder::placeholder { color: red; }
            </style>
            <svg width="1" height="1"><defs><clipPath id="resource"><rect width="20" height="20"/></clipPath></defs></svg>
            <div id="before"></div>
            <div id="letter">Letter</div>
            <ul><li id="marker">Item</li></ul>
            <input id="placeholder" placeholder="Hint">
        "##;
            let markup = markup.replace("</style>", &format!("{binding_rules}</style>"));
            let page = fixture(&markup);
            let assembly = browser::CreateLayoutAssembly();
            let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
            let engine = page.GetLayoutEngine();
            let root = engine.GetLayoutTree().unwrap().Root();
            let mut bindings = std::collections::HashMap::new();
            let mut object = root as *const LayoutObject;
            while let Some(current) = unsafe { object.as_ref() } {
                if let Some(node) = unsafe { current.GetNode().as_ref() } {
                    bindings.insert(node.InputId(), node.InputStyle().svg_clip_resources.clone());
                }
                object = current.NextInPreOrder(root);
            }
            for (owner, bit) in [
                ("before", 50),
                ("letter", 46),
                ("marker", 63),
                ("placeholder", 54),
            ] {
                let owner_id = id(&page, owner);
                let owner_binding = &bindings[&owner_id];
                assert_eq!(
                    owner_binding
                        .snapshot
                        .entries
                        .iter()
                        .any(|entry| entry.id == "resource"),
                    referenced,
                    "{owner} resource ownership"
                );
                let pseudo_id = owner_id | (1_u64 << bit);
                let pseudo_binding = bindings
                    .get(&pseudo_id)
                    .unwrap_or_else(|| panic!("missing generated owner for {owner}"));
                if referenced {
                    assert_eq!(pseudo_binding.scope, owner_binding.scope, "{owner} scope");
                    assert!(
                        std::sync::Arc::ptr_eq(&pseudo_binding.snapshot, &owner_binding.snapshot),
                        "{owner} lost its referenced resource catalog"
                    );
                } else {
                    assert!(
                        pseudo_binding.snapshot.entries.is_empty(),
                        "{owner} retained an unrelated resource catalog"
                    );
                }
            }
        }
    });
}

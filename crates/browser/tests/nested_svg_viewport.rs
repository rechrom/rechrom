//! Exercise nested SVG through the production Page, retained native tree,
//! fragment publication and immutable PaintArtifact boundaries.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::{
        layout_input::{FontFace, TransformMatrix},
        layout_object::LayoutObject,
        layout_pass_scope::LayoutObjectFactoryScope,
    },
};
use page_mutation::PageMutation;
use paint::{
    geometry_mapper::{MapRectToRoot, MultiplyTransforms},
    paint_engine::{DisplayItemType, PaintArtifact, PaintRect},
};
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
        family: "NestedSVGFixture".into(),
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
    page.Open("https://nested-svg.test/", 16384, 4096).unwrap();
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

#[derive(Clone, Default)]
struct ReplayState {
    transform: TransformMatrix,
    clip: Option<PaintRect>,
}

/// Inspect the existing flat recording ABI, including save/restore scope, not
/// only a shape's local bounds. Fixtures use axis-aligned rectangle clips and
/// transforms, so mapping and intersecting these rectangles is exact.
fn rectangle_draws(artifact: &PaintArtifact, wanted: u64, clipped: bool) -> Vec<PaintRect> {
    let mut state = ReplayState::default();
    let mut stack = Vec::new();
    let mut result = Vec::new();
    for item in artifact.items.iter() {
        match item.r#type {
            DisplayItemType::kSave
            | DisplayItemType::kSaveLayer
            | DisplayItemType::kSaveLayerAlpha
            | DisplayItemType::kSaveLayerBlend
            | DisplayItemType::kSaveLayerFilter
            | DisplayItemType::kSaveLayerDstIn => stack.push(state.clone()),
            DisplayItemType::kRestore => state = stack.pop().expect("balanced paint restore"),
            DisplayItemType::kConcat => {
                state.transform = MultiplyTransforms(&state.transform, &item.transform);
            }
            DisplayItemType::kClipRect => {
                let rect = MapRectToRoot(item.rect, &[state.transform]).unwrap();
                state.clip = Some(state.clip.map_or(rect, |clip| clip.intersection(rect)));
            }
            DisplayItemType::kDrawRect if item.node_id == wanted => {
                let rect = MapRectToRoot(item.rect, &[state.transform]).unwrap();
                let rect = if clipped {
                    state.clip.map_or(rect, |clip| clip.intersection(rect))
                } else {
                    rect
                };
                if !rect.is_empty() {
                    result.push(rect);
                }
            }
            _ => {}
        }
    }
    assert!(stack.is_empty(), "balanced paint save scopes");
    result
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> PaintRect {
    PaintRect {
        x,
        y,
        width,
        height,
    }
}

fn assert_rect(actual: PaintRect, expected: PaintRect) {
    for (label, actual, expected) in [
        ("x", actual.x, expected.x),
        ("y", actual.y, expected.y),
        ("width", actual.width, expected.width),
        ("height", actual.height, expected.height),
    ] {
        assert!(
            (actual - expected).abs() < 1e-6,
            "{label}: {actual}, expected {expected}"
        );
    }
}

fn assert_draw(page: &Page, name: &str, expected: PaintRect, clipped: bool) {
    let draws = rectangle_draws(
        &page.CurrentFrame().unwrap().display_items,
        id(page, name),
        clipped,
    );
    assert_eq!(draws.len(), 1, "{name}: {draws:?}");
    assert_rect(draws[0], expected);
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

#[test]
fn nested_svg_loads_through_production_page() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r#"<svg id="outer" width="200" height="100">
                <svg id="inner" x="20" y="10" width="80" height="40">
                    <rect id="shape" width="30" height="15" fill="red"/>
                </svg>
            </svg>"#,
        );
        assert!(native(&page, "outer").IsSVGRoot());
        assert!(!native(&page, "inner").IsSVGRoot());
        assert!(native(&page, "inner").IsSVGContainer());
        assert!(native(&page, "inner").IsSVGViewportContainer());
        let viewport = fragment(&page, "inner");
        assert_eq!(viewport.size.width, 80.);
        assert_eq!(viewport.size.height, 40.);
        assert_draw(&page, "shape", rect(20., 10., 30., 15.), true);
    });
}

#[test]
fn rectangle_geometry_helper_preserves_an_ordinary_svg_root() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r#"<svg id="outer" width="200" height="100" viewBox="0 0 100 50"><rect id="shape" x="10" y="5" width="20" height="10"/></svg>"#,
        );
        assert!(native(&page, "outer").IsSVGRoot());
        assert_draw(&page, "shape", rect(20., 10., 40., 20.), true);
    });
}

#[test]
fn nested_viewbox_applies_none_meet_slice_and_alignment() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (preserve, expected) in [
            ("none", rect(20., 10., 40., 20.)),
            ("xMidYMid meet", rect(40., 10., 20., 20.)),
            ("xMaxYMax meet", rect(60., 10., 20., 20.)),
            ("xMinYMin meet", rect(20., 10., 20., 20.)),
            ("xMidYMid slice", rect(20., -10., 40., 40.)),
        ] {
            let page = fixture(&format!(
                r#"<svg width="200" height="120"><svg x="20" y="10" width="80" height="40" viewBox="10 20 20 20" preserveAspectRatio="{preserve}"><rect id="shape" x="10" y="20" width="10" height="10"/></svg></svg>"#
            ));
            assert_draw(&page, "shape", expected, false);
        }
    });
}

#[test]
fn nested_percentages_use_the_nearest_viewport_user_space() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r#"<svg id="outer" width="200" height="100" viewBox="0 0 100 50"><svg id="middle" x="10%" y="20%" width="50%" height="50%"><svg id="inner" x="50%" y="50%" width="50%" height="50%"><rect id="shape" width="25" height="12.5"/></svg></svg></svg>"#,
        );
        assert!(native(&page, "middle").IsSVGViewportContainer());
        assert!(native(&page, "inner").IsSVGViewportContainer());
        assert_eq!(fragment(&page, "middle").size.width, 50.);
        assert_eq!(fragment(&page, "middle").size.height, 25.);
        assert_eq!(fragment(&page, "inner").size.width, 25.);
        assert_eq!(fragment(&page, "inner").size.height, 12.5);
        assert_draw(&page, "shape", rect(70., 45., 50., 25.), false);
        assert_draw(&page, "shape", rect(70., 45., 50., 25.), true);
    });
}

#[test]
fn nested_transform_precedes_viewport_translation_and_viewbox_mapping() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r#"<svg width="200" height="120"><g transform="translate(3 4)"><svg x="20" y="10" width="80" height="40" viewBox="0 0 40 20" transform="translate(5 7) scale(2)"><rect id="shape" width="10" height="5"/></svg></g></svg>"#,
        );
        assert_draw(&page, "shape", rect(48., 31., 40., 20.), true);
    });
}

#[test]
fn nested_overflow_clip_is_local_and_does_not_leak_to_siblings() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (overflow, expected) in [
            ("hidden", rect(20., 10., 80., 40.)),
            ("visible", rect(10., 5., 100., 60.)),
            ("auto", rect(10., 5., 100., 60.)),
            ("scroll", rect(20., 10., 80., 40.)),
        ] {
            let page = fixture(&format!(
                r#"<svg width="200" height="120"><svg id="inner" x="20" y="10" width="80" height="40" style="overflow:{overflow}"><rect id="shape" x="-10" y="-5" width="100" height="60"/></svg><rect id="sibling" x="130" y="80" width="20" height="10"/></svg>"#
            ));
            assert!(
                fragment(&page, "inner").paint.scroll_container.is_none(),
                "overflow:{overflow} must not establish CSS scroll state"
            );
            assert_draw(&page, "shape", rect(10., 5., 100., 60.), false);
            assert_draw(&page, "shape", expected, true);
            assert_draw(&page, "sibling", rect(130., 80., 20., 10.), true);
        }
    });
}

#[test]
fn zero_sized_nested_viewports_do_not_paint_descendants() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (width, height) in [("0", "40"), ("80", "0"), ("0", "0")] {
            let page = fixture(&format!(
                r#"<svg width="200" height="120"><svg x="20" y="10" width="{width}" height="{height}" style="overflow:visible"><rect id="shape" width="30" height="15"/></svg><rect id="sibling" x="130" y="80" width="20" height="10"/></svg>"#
            ));
            assert!(rectangle_draws(
                &page.CurrentFrame().unwrap().display_items,
                id(&page, "shape"),
                true
            )
            .is_empty());
            assert_draw(&page, "sibling", rect(130., 80., 20., 10.), true);
        }
    });
}

#[test]
fn svg_inside_foreign_object_is_an_outermost_root_again() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (open, close) in [
            ("", ""),
            ("<div xmlns='http://www.w3.org/1999/xhtml'>", "</div>"),
        ] {
            let page = fixture(&format!(
                r#"<svg id="outer" width="200" height="120"><foreignObject width="120" height="80">{open}<svg id="embedded" width="80" height="40"><svg id="nested" width="20" height="10"><rect id="shape" width="10" height="5"/></svg></svg>{close}</foreignObject></svg>"#
            ));
            assert!(native(&page, "outer").IsSVGRoot());
            assert!(native(&page, "embedded").IsSVGRoot());
            assert!(!native(&page, "embedded").IsSVGViewportContainer());
            assert!(native(&page, "nested").IsSVGViewportContainer());
            assert!(!native(&page, "nested").IsSVGRoot());
        }
    });
}

#[test]
fn changing_nested_geometry_updates_paint_and_keeps_old_artifacts_immutable() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r#"<svg width="240" height="160"><svg id="inner" x="20" y="10" width="80" height="40" viewBox="0 0 40 20"><rect id="shape" width="10" height="5"/></svg></svg>"#,
        );
        let shape = id(&page, "shape");
        let original = page.CurrentFrame().unwrap().display_items.clone();
        assert_draw(&page, "shape", rect(20., 10., 20., 10.), true);
        set_attribute(&mut page, "inner", "x", "50");
        set_attribute(&mut page, "inner", "y", "30");
        assert_draw(&page, "shape", rect(50., 30., 20., 10.), true);
        set_attribute(&mut page, "inner", "width", "120");
        set_attribute(&mut page, "inner", "height", "60");
        assert_draw(&page, "shape", rect(50., 30., 30., 15.), true);
        set_attribute(&mut page, "inner", "viewBox", "0 0 20 10");
        assert_draw(&page, "shape", rect(50., 30., 60., 30.), true);
        set_attribute(&mut page, "inner", "transform", "translate(5 7)");
        assert_draw(&page, "shape", rect(55., 37., 60., 30.), true);
        set_attribute(&mut page, "inner", "viewBox", "0 0 20 20");
        assert_draw(&page, "shape", rect(85., 37., 30., 15.), true);
        set_attribute(&mut page, "inner", "preserveAspectRatio", "none");
        assert_draw(&page, "shape", rect(55., 37., 60., 15.), true);
        assert_eq!(
            rectangle_draws(&original, shape, true),
            vec![rect(20., 10., 20., 10.)]
        );
        assert!(native(&page, "inner").IsSVGViewportContainer());
    });
}

#[test]
fn fetched_svg_image_publishes_a_nested_viewport_record() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = page_fixture(
            r#"<img id="image" width="200" height="100" src="fixture.svg">"#,
            Some(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"><svg x="20" y="10" width="80" height="40" viewBox="0 0 40 20"><rect width="10" height="5" fill="red"/></svg></svg>"#,
            ),
        );
        let artifact = &page.CurrentFrame().unwrap().display_items;
        let draw = artifact
            .items
            .iter()
            .find(|item| {
                item.r#type == DisplayItemType::kDrawImageRect && item.node_id == id(&page, "image")
            })
            .expect("fetched SVG is drawn as an image");
        let image = artifact
            .resources
            .as_ref()
            .unwrap()
            .images
            .iter()
            .find(|image| image.id == draw.resource_id)
            .expect("draw has a published image resource");
        let image_resource::PaintImageContent::Document(content) = &image.content else {
            panic!("SVG resource keeps its document-paint recording");
        };
        let key = draw
            .document_image_container_key
            .as_ref()
            .expect("SVG draw container key");
        let record = content
            .exact_record_for(key)
            .expect("current SVG container record");
        let record = paint::paint_engine::downcast_document_paint_record::<
            paint::paint_engine::DocumentPaintArtifactRecord,
        >(record)
        .expect("SVG PaintArtifact record");
        let shape = record
            .artifact
            .items
            .iter()
            .find(|item| {
                item.r#type == DisplayItemType::kDrawRect
                    && item.color.red == 1.
                    && item.color.green == 0.
                    && item.color.blue == 0.
            })
            .expect("nested red rectangle is recorded");
        let draws = rectangle_draws(&record.artifact, shape.node_id, true);
        assert_eq!(draws.len(), 1);
        assert_rect(draws[0], rect(20., 10., 20., 10.));
    });
}

#[test]
fn nested_viewbox_clip_stays_in_the_viewport_coordinate_system() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r#"<svg width="200" height="120"><svg x="20" y="10" width="80" height="40" viewBox="10 20 20 20" preserveAspectRatio="none"><rect id="shape" width="50" height="50"/></svg></svg>"#,
        );
        assert_draw(&page, "shape", rect(-20., -30., 200., 100.), false);
        assert_draw(&page, "shape", rect(20., 10., 80., 40.), true);
    });
}

#[test]
fn reparenting_svg_reclassifies_root_and_viewport_without_changing_dom_identity() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r#"<svg id="outer" width="200" height="120"></svg><div id="host"><svg id="moving" x="20" y="10" width="80" height="40"><rect id="shape" width="10" height="5"/></svg></div>"#,
        );
        let moving = id(&page, "moving");
        assert!(native(&page, "moving").IsSVGRoot());
        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kAppendChild,
            target_node_id: id(&page, "outer"),
            child_node_id: moving,
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(id(&page, "moving"), moving);
        assert!(native(&page, "moving").IsSVGViewportContainer());
        assert!(!native(&page, "moving").IsSVGRoot());
        assert_draw(&page, "shape", rect(20., 10., 10., 5.), true);
        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kAppendChild,
            target_node_id: id(&page, "host"),
            child_node_id: moving,
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(id(&page, "moving"), moving);
        assert!(native(&page, "moving").IsSVGRoot());
        assert!(!native(&page, "moving").IsSVGViewportContainer());
    });
}

#[test]
fn empty_nested_viewbox_suppresses_paint_until_valid_geometry_returns() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for view_box in ["0 0 0 20", "0 0 40 0", "0 0 0 0"] {
            let mut page = fixture(&format!(
                r#"<svg width="200" height="120"><svg id="inner" x="20" y="10" width="80" height="40" viewBox="{view_box}" style="overflow:visible"><rect id="shape" width="10" height="5"/></svg></svg>"#
            ));
            assert!(rectangle_draws(
                &page.CurrentFrame().unwrap().display_items,
                id(&page, "shape"),
                true
            )
            .is_empty());
            set_attribute(&mut page, "inner", "viewBox", "0 0 40 20");
            assert_draw(&page, "shape", rect(20., 10., 20., 10.), true);
        }
    });
}

#[test]
fn retained_overflow_mutation_refreshes_only_the_nested_clip_scope() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r#"<svg width="200" height="120"><svg id="inner" x="20" y="10" width="80" height="40" style="overflow:hidden"><rect id="shape" x="-10" y="-5" width="100" height="60"/></svg><rect id="sibling" x="130" y="80" width="20" height="10"/></svg>"#,
        );
        for (overflow, expected) in [
            ("hidden", rect(20., 10., 80., 40.)),
            ("visible", rect(10., 5., 100., 60.)),
            ("hidden", rect(20., 10., 80., 40.)),
        ] {
            set_attribute(&mut page, "inner", "style", &format!("overflow:{overflow}"));
            assert_draw(&page, "shape", expected, true);
            assert_draw(&page, "sibling", rect(130., 80., 20., 10.), true);
            assert!(
                fragment(&page, "inner").paint.scroll_container.is_none(),
                "overflow:{overflow} must not establish CSS scroll state"
            );
        }
    });
}

#[test]
fn retained_css_transforms_update_svg_group_and_nested_viewport_geometry() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r#"<svg width="200" height="120"><g id="group"><svg id="inner" x="20" y="10" width="80" height="40" viewBox="0 0 40 20"><rect id="shape" width="10" height="5"/></svg></g></svg>"#,
        );
        assert_draw(&page, "shape", rect(20., 10., 20., 10.), true);
        set_attribute(&mut page, "group", "style", "transform:translate(3px,4px)");
        assert_draw(&page, "shape", rect(23., 14., 20., 10.), true);
        set_attribute(
            &mut page,
            "inner",
            "style",
            "transform:translate(5px,7px) scale(2)",
        );
        assert_draw(&page, "shape", rect(48., 31., 40., 20.), true);
        set_attribute(
            &mut page,
            "group",
            "style",
            "transform:translate(11px,13px)",
        );
        assert_draw(&page, "shape", rect(56., 40., 40., 20.), true);
        set_attribute(&mut page, "inner", "style", "transform:none");
        assert_draw(&page, "shape", rect(31., 23., 20., 10.), true);
    });
}

#[test]
fn unsupported_nested_geometry_reports_explicit_layout_errors() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (attributes, expected) in [
            ("width='2em' height='40'", "SVG viewport contextual/calculated or invalid length"),
            ("width='80' height='40' style=\"offset-path:path('M0 0 L10 0');offset-distance:5px\"", "SVG viewport motion-path transform"),
            ("width='80' height='40' style='transform-box:fill-box;transform:scale(2)'", "SVG container fill-box/stroke-box transform bounds"),
        ] {
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _page = fixture(&format!(r#"<svg width="200" height="120"><svg {attributes}><rect width="10" height="5"/></svg></svg>"#));
            })).expect_err("unsupported SVG geometry must fail explicitly");
            let error = failure.downcast_ref::<foundation::UnsupportedLayout>().expect("typed UnsupportedLayout, not an unrelated invariant panic");
            assert!(error.to_string().contains(expected), "{error}");
        }
    });
}

#[test]
fn inline_svg_root_resize_does_not_reuse_an_image_container_override() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r#"<svg id="outer" width="40" height="80"><svg x="50%" y="50%" width="50%" height="50%"><rect id="shape" width="10" height="10"/></svg></svg>"#,
        );
        assert_draw(&page, "shape", rect(20., 40., 10., 10.), true);
        set_attribute(&mut page, "outer", "width", "80");
        set_attribute(&mut page, "outer", "height", "160");
        assert_draw(&page, "shape", rect(40., 80., 10., 10.), true);
    });
}

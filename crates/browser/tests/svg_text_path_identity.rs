//! SVG textPath identity and unresolved-resource lifecycle through production Page.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::internal::{
    layout_input::FontFace, layout_object::LayoutObject,
    layout_pass_scope::LayoutObjectFactoryScope,
};
use page_mutation::PageMutation;
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
        family: "SvgTextPathFixture".into(),
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

fn pixels(page: &Page) -> Vec<u32> {
    let mut pixels = vec![0; 320 * 240];
    page.PaintArtifactInto(
        &page.CurrentFrame().unwrap().display_items,
        320,
        240,
        1.0,
        &mut pixels,
        renderer_skia::PixelFormat::Rgba8888,
        320,
    )
    .unwrap();
    pixels
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

const EMPTY: &str = r#"<svg width="300" height="100"><text id="text" y="30" style="font:20px SvgTextPathFixture"></text></svg>"#;

#[test]
fn unresolved_text_path_keeps_native_identity_and_hides_glyphs() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            r##"<svg width="300" height="100"><text y="30" style="font:20px SvgTextPathFixture"><textPath id="textpath" href="#missing">HIDDEN</textPath></text></svg>"##,
        );
        assert!(native(&page, "textpath").IsSVGTextPath());
        assert!(native(&page, "textpath").IsSVGInline());
        let empty = fixture(EMPTY);
        assert_eq!(pixels(&page), pixels(&empty));
    });
}

#[test]
fn text_path_identity_is_independent_of_supported_geometry() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (path, href, expected_resolved) in [
            ("M0 30 H280", "#path", true),
            ("M0 30", "#path", false),
            ("", "#path", false),
            (
                "M337.5,337.5 m-320,0 a320,320 0 1,1 640,0 a320,320 0 1,1 -640,0",
                "#path",
                true,
            ),
            ("M0 30 H120 M160 30 H280", "#path", false),
            ("M0 30 H280", "#missing", false),
            ("M0 30 H280", "", false),
        ] {
            let page = fixture(&format!(
                r#"<svg width="300" height="100"><defs style="display:none"><path id="path" d="{path}"/></defs><text y="30" style="font:20px SvgTextPathFixture"><textPath id="textpath" href="{href}">Text<tspan> path</tspan></textPath></text></svg>"#
            ));
            let object = native(&page, "textpath");
            assert!(object.IsSVGTextPath(), "{path:?} {href:?}");
            let element = unsafe {
                &*object
                    .GetNode()
                    .cast::<layoutng_assembly::internal::layout_node_metadata::Element>()
            }
            .InputElementData()
            .as_ref()
            .unwrap();
            assert_eq!(
                element.svg_text_path.is_some(),
                expected_resolved,
                "{path:?} {href:?}"
            );
        }
    });
}

#[test]
fn retained_text_path_resolution_changes_keep_identity_and_old_frame() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(
            r##"<svg width="300" height="100"><defs style="display:none"><path id="path" d="M0 30 H280"/></defs><text style="font:20px SvgTextPathFixture"><textPath id="textpath" href="#missing">Visible</textPath></text></svg>"##,
        );
        let original = native(&page, "textpath") as *const LayoutObject;
        let hidden_pixels = pixels(&page);
        let old_frame = page.CurrentFrame().unwrap().display_items.clone();
        set_attribute(&mut page, "textpath", "href", "#path");
        page.RunTask().unwrap();
        assert_eq!(native(&page, "textpath") as *const LayoutObject, original);
        assert!(native(&page, "textpath").IsSVGTextPath());
        assert_ne!(
            pixels(&page),
            hidden_pixels,
            "resolved geometry should reveal text"
        );
        let mut old_pixels = vec![0; 320 * 240];
        page.PaintArtifactInto(
            &old_frame,
            320,
            240,
            1.0,
            &mut old_pixels,
            renderer_skia::PixelFormat::Rgba8888,
            320,
        )
        .unwrap();
        assert_eq!(old_pixels, hidden_pixels);
        set_attribute(&mut page, "path", "d", "");
        page.RunTask().unwrap();
        assert_eq!(native(&page, "textpath") as *const LayoutObject, original);
        assert_eq!(pixels(&page), hidden_pixels, "empty target hides glyphs");
        set_attribute(&mut page, "path", "d", "M0 30 H280");
        page.RunTask().unwrap();
        assert_eq!(native(&page, "textpath") as *const LayoutObject, original);
        assert_ne!(
            pixels(&page),
            hidden_pixels,
            "changed target restores glyphs"
        );
        let visible_pixels = pixels(&page);
        set_attribute(&mut page, "path", "d", "M200 180 M0 30 H280");
        page.RunTask().unwrap();
        assert_eq!(native(&page, "textpath") as *const LayoutObject, original);
        assert!(
            pixels(&page) == visible_pixels,
            "empty leading contour must not change glyph positions"
        );
        set_attribute(&mut page, "path", "d", "M200 180 M0 30");
        page.RunTask().unwrap();
        assert_eq!(native(&page, "textpath") as *const LayoutObject, original);
        assert!(
            pixels(&page) == hidden_pixels,
            "move-only target hides glyphs"
        );
        set_attribute(&mut page, "textpath", "href", "#missing");
        page.RunTask().unwrap();
        assert_eq!(native(&page, "textpath") as *const LayoutObject, original);
        assert_eq!(pixels(&page), hidden_pixels);
    });
}

#[test]
fn text_path_requires_an_eligible_svg_text_parent() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for markup in [
            r##"<svg width="300" height="100"><textPath id="invalid" href="#missing"/></svg>"##,
            r##"<svg width="300" height="100"><g><textPath id="invalid">Ignored</textPath></g></svg>"##,
            r##"<svg width="300" height="100"><a><textPath id="invalid" href="#missing"/></a></svg>"##,
            r##"<svg width="300" height="100"><text><tspan>Control<textPath id="invalid">Ignored</textPath></tspan></text></svg>"##,
            r##"<svg width="300" height="100"><text><tspan>Control<a><textPath id="invalid">Ignored</textPath></a></tspan></text></svg>"##,
            r##"<svg width="300" height="100"><text><tspan>Control<a style="display:contents"><textPath id="invalid">Ignored</textPath></a></tspan></text></svg>"##,
        ] {
            let page = fixture(markup);
            let assembly = browser::CreateLayoutAssembly();
            let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
            let engine = page.GetLayoutEngine();
            let root = engine.GetLayoutTree().unwrap().Root();
            let mut object = root as *const LayoutObject;
            let invalid = id(&page, "invalid");
            while !object.is_null() {
                let current = unsafe { &*object };
                let node = current.GetNode();
                assert!(node.is_null() || unsafe { &*node }.InputId() != invalid);
                object = current.NextInPreOrder(root);
            }
        }
        let page = fixture(
            r##"<svg width="300" height="100"><text><a><textPath id="valid" href="#missing">Hidden</textPath></a></text></svg>"##,
        );
        assert!(native(&page, "valid").IsSVGTextPath());
    });
}

#[test]
fn consecutive_moveto_text_paths_match_their_single_moveto_geometry_and_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (source, normalized) in [
            ("M200 180 M10 60 H290", "M10 60 H290"),
            (
                "M200 180 m-190 -120 m10 0 Q140 10 280 60",
                "M20 60 Q140 10 280 60",
            ),
            (
                "M100 100 M20 60 C80 10 220 110 280 60",
                "M20 60 C80 10 220 110 280 60",
            ),
            ("M100 100 M20 60 H280 V160 H20 Z", "M20 60 H280 V160 H20 Z"),
            (
                "M150 110 m-90 0 a90 90 0 1 1 180 0 a90 90 0 1 1 -180 0",
                "M60 110 a90 90 0 1 1 180 0 a90 90 0 1 1 -180 0",
            ),
        ] {
            let markup = |d: &str| {
                format!(
                    r##"<svg width="320" height="240"><defs style="display:none"><path id="path" d="{d}"/></defs><text style="font:20px SvgTextPathFixture"><textPath id="textpath" href="#path" startOffset="10%">Contour test</textPath></text></svg>"##
                )
            };
            let source_page = fixture(&markup(source));
            let normalized_page = fixture(&markup(normalized));
            let data = |page: &Page| {
                let object = native(page, "textpath");
                unsafe {
                    &*object
                        .GetNode()
                        .cast::<layoutng_assembly::internal::layout_node_metadata::Element>()
                }
                .InputElementData()
                .as_ref()
                .unwrap()
                .svg_text_path
                .clone()
            };
            assert!(data(&source_page).is_some(), "unresolved source {source}");
            assert_eq!(data(&source_page), data(&normalized_page), "{source}");
            assert_same_pixels(&source_page, &normalized_page, source);
            assert_ne!(
                pixels(&source_page),
                pixels(&fixture(EMPTY)),
                "text must be visible: {source}"
            );
        }
    });
}

#[test]
fn consecutive_moveto_shape_pixels_preserve_curves_close_and_stroke_caps() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (source, normalized) in [
            ("M300 220 M20 60 H280", "M20 60 H280"),
            (
                "M300 220 m-280 -160 m0 0 Q140 10 280 60",
                "M20 60 Q140 10 280 60",
            ),
            (
                "M300 220 M20 60 C80 10 220 110 280 60",
                "M20 60 C80 10 220 110 280 60",
            ),
            ("M300 220 M20 60 H280 V160 H20 Z", "M20 60 H280 V160 H20 Z"),
            (
                "M150 110 m-90 0 a90 90 0 1 1 180 0 a90 90 0 1 1 -180 0",
                "M60 110 a90 90 0 1 1 180 0 a90 90 0 1 1 -180 0",
            ),
            (
                "M20 60 H100 M300 220 M180 160 H280",
                "M20 60 H100 M180 160 H280",
            ),
        ] {
            for cap in ["butt", "round", "square"] {
                let markup = |d: &str| {
                    format!(
                        r#"<svg width="320" height="240"><path d="{d}" fill="none" stroke="blue" stroke-width="8" stroke-linecap="{cap}"/></svg>"#
                    )
                };
                let source_page = fixture(&markup(source));
                let normalized_page = fixture(&markup(normalized));
                assert_same_pixels(
                    &source_page,
                    &normalized_page,
                    &format!("{source}; cap={cap}"),
                );
                assert_ne!(
                    pixels(&source_page),
                    pixels(&fixture(EMPTY)),
                    "stroke must be visible: {source}"
                );
            }
        }
    });
}

#[test]
fn visible_move_only_paths_are_valid_and_paint_no_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let control = fixture(
            r#"<svg width="320" height="240"><rect x="200" y="180" width="20" height="20" fill="green"/></svg>"#,
        );
        for d in ["M20 30", "M20 30 M200 100", "M20 30 m40 40 m-20 10"] {
            for cap in ["butt", "round", "square"] {
                let page = fixture(&format!(
                    r#"<svg width="320" height="240"><path d="{d}" fill="red" stroke="blue" stroke-width="8" stroke-linecap="{cap}"/><rect x="200" y="180" width="20" height="20" fill="green"/></svg>"#
                ));
                assert_same_pixels(&page, &control, &format!("{d}; cap={cap}"));
            }
        }
    });
}

fn assert_same_pixels(actual: &Page, expected: &Page, context: &str) {
    let actual = pixels(actual);
    let expected = pixels(expected);
    let changed = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
    assert_eq!(changed, 0, "changed pixels for {context}");
}

#[test]
fn explicit_close_and_zero_length_lines_keep_their_wide_stroke_caps() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let empty = pixels(&fixture(EMPTY));
        for cap in ["butt", "round", "square"] {
            let markup = |d: &str| {
                format!(
                    r#"<svg width="320" height="240"><path d="{d}" fill="none" stroke="blue" stroke-width="8" stroke-linecap="{cap}"/></svg>"#
                )
            };
            let closed = fixture(&markup("M40 40 Z"));
            let line = fixture(&markup("M40 40 L40 40 Z"));
            let repeated = fixture(&markup("M200 200 M40 40 Z"));
            assert_same_pixels(&closed, &line, cap);
            assert_same_pixels(&repeated, &closed, cap);
            let changed = pixels(&closed)
                .iter()
                .zip(&empty)
                .filter(|(a, b)| a != b)
                .count();
            assert_eq!(changed > 0, cap != "butt", "cap={cap}; changed={changed}");
        }
    });
}

#[test]
fn consecutive_moveto_closed_fill_pixels_match_normalized_path() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let markup =
            |d: &str| format!(r#"<svg width="320" height="240"><path d="{d}" fill="red"/></svg>"#);
        let normalized = fixture(&markup("M20 40 Q150 10 280 40 V180 H20 Z"));
        let source = fixture(&markup("M300 220 M20 40 Q150 10 280 40 V180 H20 Z"));
        assert_same_pixels(&source, &normalized, "closed fill");
        assert!(pixels(&source) != pixels(&fixture(EMPTY)));
    });
}

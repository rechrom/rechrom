//! Ordinary SVG use instances retain DOM/style/layout ownership across mutations.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::internal::layout_input::FontFace;
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

const SIMPLE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80"><defs><rect id="geometry" width="20" height="20"/></defs><use id="instance" href="#geometry" x="10" y="15" fill="blue"/></svg>"##;

#[test]
fn local_use_has_real_shadow_instance_and_inherited_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let p = fixture(SIMPLE);
        let owner = p.Document();
        let doc = owner.GetDocument();
        let host = doc.FindNodeById(id(&p, "instance")).unwrap();
        let root = doc
            .ShadowRoot(host)
            .expect("use must own an instance shadow tree");
        let clone = *doc
            .Node(root)
            .Children()
            .first()
            .expect("local target instance");
        assert_eq!(
            doc.CorrespondingSVGElement(clone),
            doc.FindNodeById(id(&p, "geometry"))
        );
        assert!(
            doc.Node(host).Children().is_empty(),
            "instances are not author DOM children"
        );
        let actual = pixels(&p, &p.CurrentFrame().unwrap().display_items);
        assert_eq!(actual[20 * 320 + 15].to_ne_bytes(), [0, 0, 255, 255]);
        assert_eq!(actual[5 * 320 + 5].to_ne_bytes(), [255, 255, 255, 255]);
    });
}

fn current_pixels(p: &Page) -> Vec<u32> {
    pixels(p, &p.CurrentFrame().unwrap().display_items)
}
fn pixel(p: &Page, x: usize, y: usize) -> [u8; 4] {
    current_pixels(p)[y * 320 + x].to_ne_bytes()
}
fn replace_children(p: &mut Page, name: &str, markup: &str) {
    p.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetInnerHTML,
        target_node_id: id(p, name),
        value: markup.into(),
        ..Default::default()
    }))
    .unwrap();
}
fn instance_root(p: &Page, name: &str) -> Option<u64> {
    let owner = p.Document();
    let doc = owner.GetDocument();
    let host = doc.FindNodeById(id(p, name)).unwrap();
    doc.ShadowRoot(host)
        .and_then(|root| doc.Node(root).Children().first())
        .map(|&node| doc.Node(node).Id())
}

#[test]
fn source_geometry_href_id_replacement_refresh_without_mutating_old_frames() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg width="120" height="80"><defs id="definitions"><rect id="geometry" width="20" height="20"/><rect id="other" width="40" height="10"/></defs><use id="instance" href="#geometry" x="10" y="15" fill="blue"/></svg>"##,
        );
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let before = pixels(&p, &old);
        assert_eq!(pixel(&p, 15, 30), [0, 0, 255, 255]);
        let first = instance_root(&p, "instance");
        set_attribute(&mut p, "geometry", "height", "10");
        assert_ne!(instance_root(&p, "instance"), first);
        assert_eq!(pixel(&p, 15, 30), [255, 255, 255, 255]);
        set_attribute(&mut p, "instance", "href", "#other");
        assert_eq!(pixel(&p, 45, 20), [0, 0, 255, 255]);
        set_attribute(&mut p, "other", "id", "renamed");
        assert_eq!(instance_root(&p, "instance"), None);
        assert_eq!(pixel(&p, 45, 20), [255, 255, 255, 255]);
        replace_children(
            &mut p,
            "definitions",
            r#"<circle id="other" cx="10" cy="10" r="8"/>"#,
        );
        assert!(instance_root(&p, "instance").is_some());
        assert_eq!(pixel(&p, 20, 25), [0, 0, 255, 255]);
        replace_children(&mut p, "definitions", "");
        assert_eq!(instance_root(&p, "instance"), None);
        assert_eq!(pixel(&p, 20, 25), [255, 255, 255, 255]);
        assert_eq!(pixels(&p, &old), before);
    });
}

#[test]
fn nested_uses_and_siblings_preserve_branch_local_transform_and_color() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg width="160" height="100"><defs><rect id="box" width="10" height="10"/><g id="pair"><use href="#box"/><use href="#box" x="20"/></g></defs><use id="blue" href="#pair" transform="translate(10 10) scale(2)" fill="blue"/><use id="red" href="#pair" x="90" y="20" fill="red"/></svg>"##,
        );
        for (x, y, color) in [
            (15, 15, [0, 0, 255, 255]),
            (55, 15, [0, 0, 255, 255]),
            (92, 25, [255, 0, 0, 255]),
            (115, 25, [255, 0, 0, 255]),
        ] {
            assert_eq!(pixel(&p, x, y), color, "({x},{y})");
        }
        set_attribute(&mut p, "box", "width", "5");
        assert_eq!(pixel(&p, 25, 15), [255, 255, 255, 255]);
        assert_eq!(pixel(&p, 55, 15), [0, 0, 255, 255]);
        set_attribute(&mut p, "blue", "fill", "green");
        assert_eq!(pixel(&p, 15, 15), [0, 128, 0, 255]);
        assert_eq!(pixel(&p, 92, 25), [255, 0, 0, 255]);
    });
}

#[test]
fn instance_rules_use_source_scope_but_clone_ancestry_and_host_inheritance() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<style id="sheet">#source-parent rect{fill:red}.shape{fill:var(--ink,currentColor)}</style><svg width="140" height="80"><defs id="source-parent" color="red"><rect id="box" class="shape" width="15" height="15"/></defs><use id="one" href="#box" x="10" y="10" color="blue"/><use id="two" href="#box" x="40" y="10" style="--ink:green"/></svg>"##,
        );
        assert_eq!(
            pixel(&p, 15, 15),
            [0, 0, 255, 255],
            "source-only ancestor must not match clone"
        );
        assert_eq!(
            pixel(&p, 45, 15),
            [0, 128, 0, 255],
            "custom properties inherit from use"
        );
        let first = instance_root(&p, "one");
        set_attribute(&mut p, "one", "color", "purple");
        assert_eq!(
            instance_root(&p, "one"),
            first,
            "host-only style retains instance identity"
        );
        assert_eq!(pixel(&p, 15, 15), [128, 0, 128, 255]);
        p.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetTextContent,
            target_node_id: id(&p, "sheet"),
            value: ".shape{fill:orange}".into(),
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(pixel(&p, 15, 15), [255, 165, 0, 255]);
        assert_eq!(pixel(&p, 45, 15), [255, 165, 0, 255]);
        assert_eq!(
            instance_root(&p, "one"),
            first,
            "stylesheet changes only recascade"
        );
    });
}

#[test]
fn use_xy_css_percent_transform_and_viewport_changes_reach_native_layout() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg id="viewport" width="100" height="80"><defs><rect id="box" width="10" height="10"/></defs><use id="instance" href="#box" x="99" y="99" style="x:20%;y:10px" transform="translate(5 5) scale(2)" fill="blue"/></svg>"##,
        );
        assert_eq!(pixel(&p, 50, 30), [0, 0, 255, 255]);
        assert_eq!(pixel(&p, 30, 30), [255, 255, 255, 255]);
        let first = instance_root(&p, "instance");
        set_attribute(&mut p, "instance", "style", "x:10px;y:0px;transform:none");
        assert_eq!(pixel(&p, 15, 5), [0, 0, 255, 255]);
        assert_eq!(pixel(&p, 50, 30), [255, 255, 255, 255]);
        assert_eq!(instance_root(&p, "instance"), first);
        set_attribute(&mut p, "instance", "style", "x:20%;y:10px;transform:none");
        set_attribute(&mut p, "viewport", "width", "200");
        assert_eq!(pixel(&p, 45, 15), [0, 0, 255, 255]);
    });
}

#[test]
fn cycle_missing_wrong_kind_and_disallowed_descendants_do_not_paint() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let p = fixture(
            r##"<div id="wrong"></div><svg width="120" height="80"><defs><g id="a"><rect width="10" height="10"/><use href="#a"/></g><g id="filtered"><foreignObject width="50" height="50"><div>hidden</div></foreignObject><rect width="10" height="10"/></g></defs><use id="cyclic" href="#a" x="10" y="10" fill="blue"/><use id="missing" href="#missing"/><use id="wronguse" href="#wrong"/><use id="filtereduse" href="#filtered" x="40" y="10" fill="red"/></svg>"##,
        );
        assert_eq!(pixel(&p, 15, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&p, 45, 15), [255, 0, 0, 255]);
        assert_eq!(instance_root(&p, "missing"), None);
        assert_eq!(instance_root(&p, "wronguse"), None);
        let owner = p.Document();
        let doc = owner.GetDocument();
        assert!(doc.NodeCount() < 70, "cycle must remain bounded");
        assert!((0..doc.NodeCount())
            .all(|i| doc.CorrespondingSVGElement(i).is_none()
                || doc.Node(i).Name() != "foreignObject"));
    });
}

#[test]
fn local_href_overrides_xlink_and_inline_matches_fetched_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100" height="80"><defs><rect id="a" width="20" height="20" fill="blue"/><rect id="b" width="40" height="20" fill="red"/></defs><use id="one" xlink:href="#a" x="10" y="10"/><use id="two" href="#a" xlink:href="#b" x="50" y="10"/></svg>"##;
        let inline = fixture(SVG);
        let fetched = page_fixture(r#"<img src="fixture.svg">"#, Some(SVG));
        assert_eq!(pixel(&inline, 15, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&inline, 55, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&inline, 75, 15), [255, 255, 255, 255]);
        assert_eq!(current_pixels(&inline), current_pixels(&fetched));
    });
}

#[test]
fn unsupported_use_features_have_explicit_dom_status() {
    native_test_thread::run(|| {
        use dom::persistent_document::SVGUseInstanceStatus as Status;
        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = html::html_parser::ParseHTML(
            r##"<svg><symbol id="symbol"/><image id="image"/><switch id="switch"/><use id="external" href="asset.svg#target"/><use id="viewport" href="#symbol"/><use id="bitmap" href="#image"/><use id="conditional" href="#switch"/></svg>"##,
        );
        let doc = owner.GetDocumentMut();
        doc.UpdateSVGUseInstances();
        for (name, expected) in [
            ("external", Status::UnsupportedReference),
            ("viewport", Status::UnsupportedViewport),
            ("bitmap", Status::UnsupportedImage),
            ("conditional", Status::UnsupportedSwitch),
        ] {
            let host = (0..doc.NodeCount())
                .find(|&i| {
                    doc.Node(i)
                        .FindAttribute("id")
                        .is_some_and(|a| a.value == name)
                })
                .unwrap();
            assert_eq!(doc.SVGUseInstanceStatus(host), Some(expected));
            assert!(doc
                .Node(doc.ShadowRoot(host).unwrap())
                .Children()
                .is_empty());
        }
    });
}

#[test]
fn clip_resource_uses_instance_visibility_and_refreshes_source_style() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg width="100" height="80"><defs><rect id="box" width="20" height="20"/></defs><clipPath id="clip"><use id="clipuse" href="#box" x="10" visibility="hidden"/></clipPath><rect width="80" height="80" fill="blue" clip-path="url(#clip)"/></svg>"##,
        );
        assert_eq!(
            pixel(&p, 15, 10),
            [255, 255, 255, 255],
            "target inherits instance visibility"
        );
        set_attribute(&mut p, "box", "visibility", "visible");
        assert_eq!(
            pixel(&p, 15, 10),
            [0, 0, 255, 255],
            "explicit target visibility overrides hidden use"
        );
        set_attribute(&mut p, "clipuse", "display", "none");
        assert_eq!(pixel(&p, 15, 10), [255, 255, 255, 255]);
        set_attribute(&mut p, "clipuse", "display", "inline");
        set_attribute(&mut p, "clipuse", "href", "");
        assert_eq!(
            pixel(&p, 15, 10),
            [255, 255, 255, 255],
            "empty local href is an empty instance, not an external IRI"
        );
    });
}

#[test]
fn duplicate_id_tree_order_and_host_detachment_clear_live_instances() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg width="100" height="80"><defs id="definitions"><rect id="box" width="10" height="10" fill="blue"/><rect id="box" width="20" height="20" fill="red"/></defs><g id="hosts"><use id="instance" href="#box" x="10" y="10"/></g></svg>"##,
        );
        assert_eq!(pixel(&p, 15, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&p, 25, 15), [255, 255, 255, 255]);
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let before = pixels(&p, &old);
        replace_children(
            &mut p,
            "definitions",
            r#"<rect id="box" width="20" height="20" fill="red"/>"#,
        );
        assert_eq!(
            pixel(&p, 25, 15),
            [255, 0, 0, 255],
            "detached first ID is not a target"
        );
        replace_children(&mut p, "hosts", "");
        assert_eq!(pixel(&p, 15, 15), [255, 255, 255, 255]);
        let owner = p.Document();
        let doc = owner.GetDocument();
        assert!((0..doc.NodeCount()).all(|i| doc.CorrespondingSVGElement(i).is_none()));
        assert_eq!(pixels(&p, &old), before);
    });
}

#[test]
fn commons_exact_asset_matches_explicit_nested_instance_control() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let original = COMMONS;
        let arrow = r##"<g clip-path="url(#aa)"><path d="M-11 180v118h22V180"/><path d="m-43 185 43-75 43 75"/></g>"##;
        let fan = [45, 90, 135]
            .into_iter()
            .map(|angle| format!("<g transform='rotate({angle})'>{arrow}</g>"))
            .collect::<String>();
        let control = original
            .replace(
                r##"<use xlink:href="#ab" transform="rotate(45)" />"##,
                &format!("<g transform='rotate(45)'>{arrow}</g>"),
            )
            .replace(
                r##"<use xlink:href="#ab" transform="rotate(90)" />"##,
                &format!("<g transform='rotate(90)'>{arrow}</g>"),
            )
            .replace(
                r##"<use xlink:href="#ab" transform="rotate(135)" />"##,
                &format!("<g transform='rotate(135)'>{arrow}</g>"),
            )
            .replace(
                r##"<use xlink:href="#ac" transform="scale(-1 1)" />"##,
                &format!("<g transform='scale(-1 1)'>{fan}</g>"),
            );
        assert!(!control.contains("<use"));
        let inline = fixture(original);
        let expected = fixture(&control);
        assert_eq!(
            current_pixels(&inline),
            current_pixels(&expected),
            "same-engine structural control"
        );
        let fetched = page_fixture(r#"<img src="fixture.svg">"#, Some(original));
        let fetched_control = page_fixture(r#"<img src="fixture.svg">"#, Some(&control));
        assert_eq!(current_pixels(&fetched), current_pixels(&fetched_control));
        assert!(
            current_pixels(&inline)
                .iter()
                .filter(|p| p.to_ne_bytes() == [0, 102, 153, 255])
                .count()
                > 100
        );
    });
}

const COMMONS: &str = r###"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="47" height="47" viewBox="-306 -517 612 822" id="Commons-logo_sister"><defs><clipPath id="aa"><circle r="298" /></clipPath></defs><circle r="100" fill="#900" /><g fill="#069"><g id="ab" clip-path="url(#aa)"><path d="M-11 180v118h22V180" /><path d="m-43 185 43-75 43 75" /></g><g id="ac"><use xlink:href="#ab" transform="rotate(45)" /><use xlink:href="#ab" transform="rotate(90)" /><use xlink:href="#ab" transform="rotate(135)" /></g><use xlink:href="#ac" transform="scale(-1 1)" /><path fill="none" stroke="#069" stroke-width="84" d="M-181.02-181.02a256 256 0 1 0 362.04 0c-70.71-70.71-177.49-34.65-198.7-190.21" /><path d="M-23-515s-36 135-80 185 116-62 170-5-90-180-90-180z" /></g></svg>"###;

#[test]
fn separate_author_shadow_scopes_choose_local_targets_and_rules() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<style>.shape{fill:red}</style><svg width="0" height="0"><defs><rect id="box" width="40" height="40"/></defs></svg><div id="host"></div>"##,
        );
        let shadow_id = p.Document().GetDocument().NextNodeId();
        p.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kAttachShadowRoot,
            target_node_id: id(&p, "host"),
            child_node_id: shadow_id,
            value: "open".into(),
            name: "named".into(),
            ..Default::default()
        }))
        .unwrap();
        // Build an HTML context inside the shadow scope using namespace-aware
        // DOM mutation. ShadowRoot.innerHTML's separate parser-context defect
        // is retained in the evidence log; it is not an SVG instance feature.
        let wrapper_id = p.Document().GetDocument().NextNodeId();
        let document_id = {
            let owner = p.Document();
            let doc = owner.GetDocument();
            doc.Node(doc.Root()).Id()
        };
        p.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kCreateElement,
            target_node_id: document_id,
            child_node_id: wrapper_id,
            name: "div".into(),
            ..Default::default()
        }))
        .unwrap();
        p.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kAppendChild,
            target_node_id: shadow_id,
            child_node_id: wrapper_id,
            ..Default::default()
        }))
        .unwrap();
        p.Apply(PageMutation::DOMMutation(DOMMutation{mutation_type:DOMMutationType::kSetInnerHTML,target_node_id:wrapper_id,value:r##"<style>.shape{fill:blue}</style><svg width="100" height="80"><defs><rect id="box" class="shape" width="10" height="10"/></defs><use id="shadowuse" href="#box" x="10" y="10"/></svg>"##.into(),..Default::default()})).unwrap();
        assert_eq!(pixel(&p, 15, 15), [0, 0, 255, 255]);
        assert_eq!(
            pixel(&p, 25, 15),
            [255, 255, 255, 255],
            "document target must not leak into shadow scope"
        );
        let owner = p.Document();
        let doc = owner.GetDocument();
        let host = doc.FindNodeById(id(&p, "shadowuse")).unwrap();
        let clone = doc.SVGUseInstanceRoot(host).unwrap();
        let source = doc.CorrespondingSVGElement(clone).unwrap();
        assert_eq!(
            doc.TreeScopeRoot(source),
            doc.FindNodeById(shadow_id).unwrap()
        );
        assert_eq!(doc.SVGResourceScope(clone), doc.SVGResourceScope(source));
    });
}

#[test]
fn source_subtree_and_nested_href_changes_refresh_retained_consumers() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg width="140" height="80"><defs><rect id="a" width="10" height="10"/><rect id="b" width="20" height="20"/><g id="group"><use id="nested" href="#a"/></g></defs><use id="instance" href="#group" x="10" y="10" fill="blue"/></svg>"##,
        );
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let before = pixels(&p, &old);
        assert_eq!(pixel(&p, 25, 15), [255, 255, 255, 255]);
        set_attribute(&mut p, "nested", "href", "#b");
        assert_eq!(pixel(&p, 25, 15), [0, 0, 255, 255]);
        replace_children(&mut p, "group", r##"<use href="#a" x="40"/>"##);
        assert_eq!(pixel(&p, 15, 15), [255, 255, 255, 255]);
        assert_eq!(pixel(&p, 55, 15), [0, 0, 255, 255]);
        assert_eq!(pixels(&p, &old), before);
    });
}

#[test]
fn display_visibility_and_explicit_target_styles_recascade_instances() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(&SIMPLE.replace(
            "</svg>",
            r##"<use id="following" href="#geometry" x="50" y="15" fill="blue"/></svg>"##,
        ));
        let first = instance_root(&p, "instance");
        set_attribute(&mut p, "instance", "visibility", "hidden");
        assert_eq!(pixel(&p, 15, 20), [255, 255, 255, 255]);
        assert_eq!(instance_root(&p, "instance"), first);
        set_attribute(&mut p, "geometry", "style", "visibility:visible;fill:red");
        assert_eq!(pixel(&p, 15, 20), [255, 0, 0, 255]);
        set_attribute(&mut p, "instance", "display", "none");
        // Reusing the following sibling moves it past the removed instance.
        // Native SVG virtuals must stay installed on this attachment fast path.
        assert_eq!(pixel(&p, 55, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&p, 15, 20), [255, 255, 255, 255]);
        set_attribute(&mut p, "instance", "display", "inline");
        assert_eq!(pixel(&p, 15, 20), [255, 0, 0, 255]);
    });
}

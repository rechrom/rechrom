//! Document-owned SVG clip resources survive canonical style and frame updates.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::{
    fragment_tree::FragmentPaintData,
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

// Engine-level fixtures must supply the same HTML/SVG UA owners as Page;
// otherwise the production SVG cascade correctly declines the missing RuleSet.
fn host_styles() -> Vec<cssom::CSSStyleSheet> {
    vec![
        style::ParseUserAgentCSS(include_str!("../resources/user_agent/html.css")),
        style::ParseUserAgentCSSWithRole(
            include_str!("../resources/user_agent/svg.css"),
            cssom::CSSUserAgentStyleSheetRole::kSVG,
        ),
    ]
}

fn fixture(svg: &str) -> Page {
    page_fixture(svg, None)
}

fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let mut stack = vec![doc.Root()];
    while let Some(index) = stack.pop() {
        let node = doc.Node(index);
        if node
            .FindAttribute("id")
            .is_some_and(|attribute| attribute.value == name)
        {
            return node.Id();
        }
        stack.extend(node.Children().iter().rev().copied());
    }
    panic!("missing DOM node {name}")
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

fn find_fragment(node: &FragmentPaintData, wanted: u64) -> Option<&FragmentPaintData> {
    if node.node_id == wanted {
        Some(node)
    } else {
        node.children
            .iter()
            .find_map(|child| find_fragment(child, wanted))
    }
}

fn fragment<'a>(page: &'a Page, name: &str) -> &'a FragmentPaintData {
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

const CLIPPED: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="80"><clipPath id="clip"><rect id="geometry" width="30" height="40" fill="red"/></clipPath><g id="target" clip-path="url(#clip)"><rect width="60" height="60" fill="blue"/></g></svg>"##;

#[test]
fn canonical_clip_reference_retains_document_resource() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let p = fixture(CLIPPED);
        let clip = fragment(&p, "target")
            .paint
            .style
            .clip_path
            .as_ref()
            .expect("canonical clip");
        assert!(
            clip.reference_graph.is_some(),
            "canonical reference lost document resource"
        );
        let actual = pixels(&p, &p.CurrentFrame().unwrap().display_items);
        assert_eq!(actual[20 * 320 + 20].to_ne_bytes(), [0, 0, 255, 255]);
        assert_eq!(actual[20 * 320 + 40].to_ne_bytes(), [255, 255, 255, 255]);
    });
}

#[test]
fn unreferenced_clip_definition_has_no_ordinary_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let p = fixture(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="80"><clipPath id="unused"><rect id="geometry" width="40" height="40" fill="red"/></clipPath><rect x="50" y="50" width="10" height="10" fill="blue"/></svg>"##,
        );
        assert!(
            native(&p, "geometry").IsSVGChild(),
            "resource descendants retain native layout"
        );
        let actual = pixels(&p, &p.CurrentFrame().unwrap().display_items);
        assert_eq!(actual[20 * 320 + 20].to_ne_bytes(), [255, 255, 255, 255]);
        assert_eq!(actual[55 * 320 + 55].to_ne_bytes(), [0, 0, 255, 255]);
    });
}

fn current_pixels(page: &Page) -> Vec<u32> {
    pixels(page, &page.CurrentFrame().unwrap().display_items)
}
fn pixel(values: &[u32], x: usize, y: usize) -> [u8; 4] {
    values[y * 320 + x].to_ne_bytes()
}
fn animate(page: &mut Page, name: &str, declarations: &str, time: f64) {
    page.Apply(PageMutation::AnimationTick(page_mutation::AnimationTick {
        monotonic_time: time,
        samples: vec![page_mutation::AnimationStyleSample {
            node_id: id(page, name),
            effect_id: 7,
            declarations: style::ParseCSSDeclarationList(declarations),
            ..Default::default()
        }],
        ..Default::default()
    }))
    .unwrap();
}

#[test]
fn resource_mutations_refresh_unchanged_consumers_and_keep_old_frames() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(CLIPPED);
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let before = pixels(&p, &old);
        let graph = fragment(&p, "target")
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap()
            .reference_graph
            .clone()
            .unwrap();
        set_attribute(&mut p, "geometry", "width", "50");
        assert_eq!(pixel(&current_pixels(&p), 40, 20), [0, 0, 255, 255]);
        assert_eq!(pixels(&p, &old), before);
        assert!(!std::sync::Arc::ptr_eq(
            &graph,
            fragment(&p, "target")
                .paint
                .style
                .clip_path
                .as_ref()
                .unwrap()
                .reference_graph
                .as_ref()
                .unwrap()
        ));
        set_attribute(&mut p, "clip", "id", "renamed");
        assert!(
            fragment(&p, "target").paint.style.clip_path.is_none(),
            "missing resource is no clip"
        );
        assert_eq!(pixel(&current_pixels(&p), 55, 55), [0, 0, 255, 255]);
        set_attribute(&mut p, "renamed", "id", "clip");
        assert_eq!(pixel(&current_pixels(&p), 55, 55), [255, 255, 255, 255]);
        page_replace_children(
            &mut p,
            "clip",
            r#"<rect id="replacement" width="10" height="10"/>"#,
        );
        assert_eq!(pixel(&current_pixels(&p), 5, 5), [0, 0, 255, 255]);
        assert_eq!(pixel(&current_pixels(&p), 20, 20), [255, 255, 255, 255]);
        assert_eq!(pixels(&p, &old), before);
    });
}

fn page_replace_children(page: &mut Page, name: &str, markup: &str) {
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetInnerHTML,
        target_node_id: id(page, name),
        value: markup.into(),
        ..Default::default()
    }))
    .unwrap();
}

#[test]
fn valid_empty_missing_wrong_kind_and_fetched_clip_are_distinct() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(CLIPPED);
        page_replace_children(&mut p, "clip", "");
        let clip = fragment(&p, "target")
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap();
        assert!(clip.reference_graph.as_ref().unwrap().children.is_empty());
        assert_eq!(pixel(&current_pixels(&p), 20, 20), [255, 255, 255, 255]);
        set_attribute(&mut p, "clip", "id", "unused");
        assert!(fragment(&p, "target").paint.style.clip_path.is_none());
        assert_eq!(pixel(&current_pixels(&p), 20, 20), [0, 0, 255, 255]);
        let p = fixture(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="80"><defs><rect id="clip"/><clipPath id="clip"><rect width="10" height="10"/></clipPath></defs><rect id="target" width="60" height="60" fill="blue" clip-path="url(#clip)"/></svg>"##,
        );
        assert!(
            fragment(&p, "target").paint.style.clip_path.is_none(),
            "first ID wins before resource type check"
        );
        assert_eq!(pixel(&current_pixels(&p), 20, 20), [0, 0, 255, 255]);
        let inline = fixture(CLIPPED);
        let fetched = page_fixture(r#"<img src="fixture.svg">"#, Some(CLIPPED));
        assert_eq!(pixel(&current_pixels(&inline), 20, 20), [0, 0, 255, 255]);
        assert_eq!(current_pixels(&inline), current_pixels(&fetched));
    });
}

#[test]
fn paint_only_animation_preserves_resource_binding_and_old_artifact() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(CLIPPED);
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let before = pixels(&p, &old);
        let graph = fragment(&p, "target")
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap()
            .reference_graph
            .clone()
            .unwrap();
        animate(&mut p, "target", "opacity:0.5", 1.0);
        assert_eq!(fragment(&p, "target").paint.style.opacity, 0.5);
        let current = fragment(&p, "target")
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap()
            .reference_graph
            .as_ref()
            .unwrap();
        assert!(
            std::sync::Arc::ptr_eq(&graph, current),
            "unchanged resource binding should stay retained"
        );
        let after = current_pixels(&p);
        assert_eq!(pixel(&after, 40, 20), [255, 255, 255, 255]);
        assert_ne!(pixel(&after, 20, 20), pixel(&before, 20, 20));
        assert_eq!(pixels(&p, &old), before);
        animate(&mut p, "target", "opacity:1", 2.0);
        assert_eq!(current_pixels(&p), before);
    });
}

#[test]
fn resource_units_transforms_visibility_and_use_target_refresh() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><rect id="source" width="20" height="40"/><rect id="other" width="40" height="20"/></defs><clipPath id="clip"><use id="instance" href="#source"/></clipPath><rect id="target" width="80" height="80" fill="blue" clip-path="url(#clip)"/></svg>"##,
        );
        assert_eq!(pixel(&current_pixels(&p), 10, 30), [0, 0, 255, 255]);
        set_attribute(&mut p, "instance", "href", "#other");
        assert_eq!(pixel(&current_pixels(&p), 10, 30), [255, 255, 255, 255]);
        assert_eq!(pixel(&current_pixels(&p), 30, 10), [0, 0, 255, 255]);
        set_attribute(&mut p, "instance", "transform", "translate(20 20)");
        assert_eq!(pixel(&current_pixels(&p), 30, 30), [0, 0, 255, 255]);
        set_attribute(&mut p, "instance", "style", "transform:none");
        set_attribute(&mut p, "other", "style", "visibility:hidden");
        assert_eq!(pixel(&current_pixels(&p), 30, 30), [255, 255, 255, 255]);
        set_attribute(&mut p, "other", "style", "visibility:visible");
        assert_eq!(pixel(&current_pixels(&p), 30, 10), [0, 0, 255, 255]);
        let p = fixture(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><clipPath id="clip" clipPathUnits="objectBoundingBox"><rect width=".5" height="1"/></clipPath><rect id="target" x="10" y="10" width="60" height="40" fill="blue" clip-path="url(#clip)"/></svg>"##,
        );
        assert_eq!(pixel(&current_pixels(&p), 20, 20), [0, 0, 255, 255]);
        assert_eq!(pixel(&current_pixels(&p), 50, 20), [255, 255, 255, 255]);
    });
}

#[test]
fn targeted_layout_update_retains_binding_and_rejects_stale_version() {
    native_test_thread::run(|| {
        use dom::persistent_document::DOMAttribute;
        use layoutng_assembly::layout_engine::{LayoutEngine, LayoutMutation};
        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = html::html_parser::ParseHTML(&format!(
            "<!doctype html><body style='margin:0'>{CLIPPED}"
        ));
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        let constraints = browser::CreateBrowserConstraints(320, 240);
        let mut style_engine = style::StyleEngine::new(&owner);
        let index = |owner: &dom::DOM, name: &str| {
            (0..owner.GetDocument().NodeCount())
                .find(|&i| {
                    owner
                        .GetDocument()
                        .Node(i)
                        .FindAttribute("id")
                        .is_some_and(|attr| attr.value == name)
                })
                .unwrap()
        };
        let target = index(&owner, "target");
        let geometry = index(&owner, "geometry");
        let target_id = owner.GetDocument().Node(target).Id();
        style_engine
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        assert!(owner.UpdateSVGResources());
        engine.ApplyMutation(LayoutMutation::Constraints(&constraints));
        owner.EmitLayoutMutations(&Default::default(), |mutation| {
            engine.ApplyMutation(mutation)
        });
        engine.Layout();
        let old = engine.GetLayoutResult().unwrap().clone();
        let graph = find_fragment(&old, target_id)
            .unwrap()
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap()
            .reference_graph
            .clone()
            .unwrap();
        owner.SetAnimationStyle(target_id, 8, style::ParseCSSDeclarationList("opacity:0.5"));
        style_engine
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        assert!(!owner.UpdateSVGResources());
        assert_eq!(
            unsafe {
                &*owner
                    .GetDocument()
                    .ResolvedStyleFor(target)
                    .unwrap()
                    .native_style
                    .Get()
            }
            .Opacity(),
            0.5,
            "canonical sampled opacity"
        );
        let mut applied = false;
        assert!(owner.EmitPaintStyle(target_id, |mutation| {
            applied = engine.ApplyMutation(mutation);
        }));
        assert!(applied);
        assert!(engine.RefreshTargetedPaintData());
        let result = find_fragment(engine.GetLayoutResult().unwrap(), target_id).unwrap();
        assert!(std::sync::Arc::ptr_eq(
            &graph,
            result
                .paint
                .style
                .clip_path
                .as_ref()
                .unwrap()
                .reference_graph
                .as_ref()
                .unwrap()
        ));
        assert_eq!(result.paint.style.opacity, 0.5);
        assert_eq!(
            find_fragment(&old, target_id).unwrap().paint.style.opacity,
            1.0
        );
        owner.GetDocumentMut().SetAttribute(
            geometry,
            DOMAttribute {
                local_name: "width".into(),
                value: "50".into(),
                ..Default::default()
            },
        );
        assert!(!owner.EmitPaintStyle(target_id, |_| panic!("stale catalog must not emit")));
        style_engine
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        assert!(owner.UpdateSVGResources());
        applied = true;
        assert!(owner.EmitPaintStyle(target_id, |mutation| {
            applied = engine.ApplyMutation(mutation);
        }));
        assert!(
            !applied,
            "new resource version must use ordinary tree update"
        );
        assert!(std::sync::Arc::ptr_eq(
            &graph,
            find_fragment(&old, target_id)
                .unwrap()
                .paint
                .style
                .clip_path
                .as_ref()
                .unwrap()
                .reference_graph
                .as_ref()
                .unwrap()
        ));
    });
}

#[test]
fn canonical_inherited_reference_keeps_originating_scope_across_shadow_root() {
    native_test_thread::run(|| {
        use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMShadowRootMode};
        use layoutng_assembly::internal::paint_input::SvgClipGeometry;
        use layoutng_assembly::layout_engine::{LayoutEngine, LayoutMutation};
        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = html::html_parser::ParseHTML(
            r##"<!doctype html><body><svg width="1" height="1"><defs><clipPath id="clip"><rect width="40" height="40"/></clipPath></defs></svg><div id="host" style="width:80px;height:80px;clip-path:url(#clip)"></div>"##,
        );
        let document = owner.GetDocumentMut();
        let host = (0..document.NodeCount())
            .find(|&i| {
                document
                    .Node(i)
                    .FindAttribute("id")
                    .is_some_and(|attr| attr.value == "host")
            })
            .unwrap();
        let document_scope = document.SVGResourceScope(host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        fn add(
            document: &mut dom::Document,
            parent: usize,
            tag: &str,
            attrs: &[(&str, &str)],
        ) -> usize {
            let node = document.CreateElement(
                DOMNamespace::kSVG,
                tag.into(),
                attrs
                    .iter()
                    .map(|(name, value)| DOMAttribute {
                        local_name: (*name).into(),
                        value: (*value).into(),
                        ..Default::default()
                    })
                    .collect(),
            );
            document.AppendChild(parent, node);
            node
        }
        let svg = add(
            document,
            shadow,
            "svg",
            &[
                ("width", "80"),
                ("height", "80"),
                ("style", "clip-path:inherit"),
            ],
        );
        let clip = add(document, svg, "clipPath", &[("id", "clip")]);
        add(document, clip, "rect", &[("width", "10"), ("height", "10")]);
        let own = add(
            document,
            svg,
            "rect",
            &[
                ("width", "60"),
                ("height", "60"),
                ("clip-path", "url(#clip)"),
            ],
        );
        let inherited = add(
            document,
            svg,
            "rect",
            &[
                ("width", "60"),
                ("height", "60"),
                ("style", "clip-path:inherit"),
            ],
        );
        let own_id = document.Node(own).Id();
        let inherited_id = document.Node(inherited).Id();
        let shadow_scope = document.SVGResourceScope(own);
        let mut style_engine = style::StyleEngine::new(&owner);
        style_engine
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        owner.UpdateSVGResources();
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        engine.ApplyMutation(LayoutMutation::Constraints(
            &browser::CreateBrowserConstraints(320, 240),
        ));
        owner.EmitLayoutMutations(&Default::default(), |mutation| {
            engine.ApplyMutation(mutation)
        });
        engine.Layout();
        let fragments = engine.GetLayoutResult().unwrap();
        let own_clip = find_fragment(fragments, own_id)
            .unwrap()
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap();
        let inherited_clip = find_fragment(fragments, inherited_id)
            .unwrap()
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap();
        assert_eq!(own_clip.reference_scope, Some(shadow_scope));
        assert_eq!(inherited_clip.reference_scope, Some(document_scope));
        let width = |clip: &layoutng_assembly::internal::paint_input::ClipPathPaint| match clip
            .reference_graph
            .as_ref()
            .unwrap()
            .children[0]
            .geometry
            .as_ref()
            .unwrap()
        {
            SvgClipGeometry::kRect { width, .. } => width.value,
            _ => panic!("expected rectangle"),
        };
        assert_eq!(width(own_clip), 10.0);
        assert_eq!(width(inherited_clip), 40.0);
    });
}

#[test]
fn percentage_resource_uses_its_viewport_and_bbox_transform_order() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg id="viewport" xmlns="http://www.w3.org/2000/svg" width="100" height="120"><clipPath id="clip"><rect width="50%" height="100%"/></clipPath><rect id="target" width="80" height="40" fill="blue" clip-path="url(#clip)"/><rect id="second" y="60" width="40" height="40" fill="blue" clip-path="url(#clip)"/></svg>"##,
        );
        let resource_width = fragment(&p, "target")
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap()
            .reference_viewport
            .unwrap()
            .width;
        // Native SVG viewport dimensions are quantized to LayoutUnit (1/64px).
        assert!((resource_width - 100.0).abs() <= 1.0 / 64.0);
        assert_eq!(
            resource_width,
            fragment(&p, "second")
                .paint
                .style
                .clip_path
                .as_ref()
                .unwrap()
                .reference_viewport
                .unwrap()
                .width
        );
        let actual = current_pixels(&p);
        assert_eq!(pixel(&actual, 45, 20), [0, 0, 255, 255]);
        assert_eq!(pixel(&actual, 55, 20), [255, 255, 255, 255]);
        assert_eq!(pixel(&actual, 35, 75), [0, 0, 255, 255]);
        set_attribute(&mut p, "viewport", "width", "120");
        let resource_width = fragment(&p, "target")
            .paint
            .style
            .clip_path
            .as_ref()
            .unwrap()
            .reference_viewport
            .unwrap()
            .width;
        assert!((resource_width - 120.0).abs() <= 1.0 / 64.0);
        assert_eq!(pixel(&current_pixels(&p), 55, 20), [0, 0, 255, 255]);
        let p = fixture(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><clipPath id="clip" clipPathUnits="objectBoundingBox" transform="translate(10 0)"><rect width=".5" height="1"/></clipPath><rect id="target" width="60" height="40" fill="blue" clip-path="url(#clip)"/></svg>"##,
        );
        let actual = current_pixels(&p);
        assert_eq!(pixel(&actual, 5, 20), [255, 255, 255, 255]);
        assert_eq!(pixel(&actual, 20, 20), [0, 0, 255, 255]);
        assert_eq!(pixel(&actual, 45, 20), [255, 255, 255, 255]);
    });
}

#[test]
fn canonical_css_geometry_and_zoomed_clip_translation_refresh() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut p = fixture(
            r##"<svg xmlns="http://www.w3.org/2000/svg" style="zoom:2" width="80" height="80"><clipPath id="clip"><rect id="geometry" width="20" height="20" transform="translate(10 0)"/></clipPath><rect id="target" width="60" height="40" fill="blue" clip-path="url(#clip)"/></svg>"##,
        );
        let actual = current_pixels(&p);
        assert_eq!(pixel(&actual, 15, 15), [255, 255, 255, 255]);
        assert_eq!(pixel(&actual, 25, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&actual, 65, 15), [255, 255, 255, 255]);
        set_attribute(&mut p, "geometry", "style", "width:30px;transform:none");
        let actual = current_pixels(&p);
        assert_eq!(pixel(&actual, 5, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&actual, 55, 15), [0, 0, 255, 255]);
        assert_eq!(pixel(&actual, 65, 15), [255, 255, 255, 255]);
    });
}

#[test]
fn foundation_asset_inline_and_fetched_retains_real_clip_content() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let asset = r###"<svg xmlns="http://www.w3.org/2000/svg" width="42" height="42" viewBox="-1 -1 1026 1026" id="Wikimedia-logo_black"><clipPath id="ga"><path d="m0-33 471 471v363h82V438l471-471v1057H0" /></clipPath><g clip-path="url(#ga)"><path d="M511 224a288 288 0 1 0 2 0" /><path fill="none" stroke="#000" stroke-width="116" d="M511 102a410 410 0 1 0 2 0" /></g><path d="M511 28a158 158 0 1 0 2 0z" /></svg>"###;
        let inline = fixture(asset);
        let fetched = page_fixture(r#"<img src="fixture.svg">"#, Some(asset));
        // Keep this lifecycle regression focused on the resource's solid
        // regions. svg_edge_raster separately requires exact inline/fetched
        // pixels through record lowering, clip raster and final composition.
        // Verify the resource's semantically solid regions in both routes:
        // head/body/outer ring survive, the central clip slot stays empty,
        // and the definition never paints the surrounding canvas.
        for actual in [current_pixels(&inline), current_pixels(&fetched)] {
            for (x, y) in [(21, 7), (5, 20), (18, 29), (25, 28), (21, 38)] {
                assert_eq!(pixel(&actual, x, y), [0, 0, 0, 255]);
            }
            for (x, y) in [(1, 1), (21, 18), (21, 27), (10, 5)] {
                assert_eq!(pixel(&actual, x, y), [255, 255, 255, 255]);
            }
            let dark = actual
                .iter()
                .filter(|pixel| {
                    let p = pixel.to_ne_bytes();
                    p[0] < 100 && p[1] < 100 && p[2] < 100 && p[3] == 255
                })
                .count();
            assert!(dark > 200, "the Foundation body must not be blank: {dark}");
        }
    });
}

#[test]
fn html_paint_input_updates_retain_text_geometry_fonts_and_svg_catalog() {
    native_test_thread::run(|| {
        use layoutng_assembly::layout_engine::{LayoutEngine, LayoutMutation};
        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = html::html_parser::ParseHTML(&format!(
            "<!doctype html><style>.wide #geometry{{width:50px}}</style><body style='margin:0'><div id='label' style='color:red'>label</div>{CLIPPED}"
        ));
        let assembly = browser::CreateLayoutAssembly();
        let mut engine = LayoutEngine::new(&assembly);
        let mut constraints = browser::CreateBrowserConstraints(320, 240);
        constraints.fonts = vec![FontFace {
            family: "SvgPaintOwnershipFixture".into(),
            bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
                .as_slice()
                .into(),
            ..Default::default()
        }]
        .into();
        let mut styles = style::StyleEngine::new(&owner);
        styles
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        owner.UpdateSVGResources();
        engine.ApplyMutation(LayoutMutation::Constraints(&constraints));
        owner.EmitLayoutMutations(&Default::default(), |m| engine.ApplyMutation(m));
        engine.Layout();
        owner.DidCommitPaint();
        let old = engine.GetLayoutResult().unwrap().clone();
        let catalog = owner.GetDocument().SVGResources().unwrap().clone();
        let label = (0..owner.GetDocument().NodeCount())
            .find(|&i| {
                owner
                    .GetDocument()
                    .Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "label")
            })
            .unwrap();
        let label_id = owner.GetDocument().Node(label).Id();
        let text_id = owner
            .GetDocument()
            .Node(owner.GetDocument().Node(label).Children()[0])
            .Id();
        let glyphs = find_fragment(&old, text_id)
            .unwrap()
            .paint
            .glyph_runs
            .clone();
        let native_label = engine.GetLayoutTree().unwrap().Root();
        let mut object = native_label as *const LayoutObject;
        while unsafe { &*object }.OwnerNodeIdDefault() as u64 != label_id {
            object = unsafe { &*object }.NextInPreOrder(native_label);
            assert!(!object.is_null());
        }
        let bound_font = unsafe { &*object }.StyleRef().GetFont();
        owner.GetDocumentMut().SetAttribute(
            label,
            dom::persistent_document::DOMAttribute {
                local_name: "style".into(),
                value: "color:blue".into(),
                ..Default::default()
            },
        );
        // Unrelated HTML mutations must preserve validity before UpdateSVGResources;
        // equal rebuilt contents would hide unnecessary whole-catalog work.
        assert!(std::sync::Arc::ptr_eq(
            &catalog,
            owner.GetDocument().SVGResources().unwrap()
        ));
        owner.GetDocumentMut().SetAttribute(
            label,
            dom::persistent_document::DOMAttribute {
                local_name: "class".into(),
                value: "label".into(),
                ..Default::default()
            },
        );
        owner.GetDocumentMut().RemoveAttribute(label, "class", "");
        assert!(std::sync::Arc::ptr_eq(
            &catalog,
            owner.GetDocument().SVGResources().unwrap()
        ));
        styles
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        assert!(!owner.UpdateSVGResources());
        assert!(std::sync::Arc::ptr_eq(
            &catalog,
            owner.GetDocument().SVGResources().unwrap()
        ));
        assert!(owner.EmitPaintInputs(label_id, &Default::default(), |m| engine.ApplyMutation(m)));
        assert!(engine.RefreshTargetedPaintInputs());
        let current = engine.GetLayoutResult().unwrap();
        let text = find_fragment(current, text_id).unwrap();
        assert_eq!(text.paint.style.color.blue, 1.0);
        assert_eq!(text.paint.style.color.red, 0.0);
        assert!(text.paint.glyph_runs == glyphs);
        assert_eq!(
            text.fragment_instance_id,
            find_fragment(&old, text_id).unwrap().fragment_instance_id
        );
        assert_eq!(unsafe { &*object }.StyleRef().GetFont(), bound_font);
        assert_eq!(
            find_fragment(&old, text_id).unwrap().paint.style.color.red,
            1.0
        );
        owner.GetDocumentMut().SetAttribute(
            label,
            dom::persistent_document::DOMAttribute {
                local_name: "style".into(),
                value: "color:blue;text-decoration:underline".into(),
                ..Default::default()
            },
        );
        styles
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        owner.UpdateSVGResources();
        assert!(owner.EmitPaintInputs(label_id, &Default::default(), |m| engine.ApplyMutation(m)));
        assert!(engine.RefreshTargetedPaintInputs());
        let retained = engine.GetLayoutResult().unwrap().clone();
        let text = find_fragment(&retained, text_id).unwrap();
        assert!(
            text.paint.style.text_decoration.underline
                || text
                    .paint
                    .style
                    .applied_text_decorations
                    .iter()
                    .any(|decoration| decoration.underline)
        );
        assert!(text.paint.glyph_runs == glyphs);
        assert!(engine.PopulatePaintData());
        let exported = find_fragment(engine.GetLayoutResult().unwrap(), text_id).unwrap();
        assert!(exported.paint.style == text.paint.style);
        assert_eq!(
            exported.paint.self_ink_overflow,
            text.paint.self_ink_overflow
        );
        assert_eq!(exported.paint.ink_overflow, text.paint.ink_overflow);
        assert!(exported.paint.glyph_runs == text.paint.glyph_runs);
        // ID changes in HTML still affect first-target resolution in this scope.
        owner.GetDocumentMut().SetAttribute(
            label,
            dom::persistent_document::DOMAttribute {
                local_name: "id".into(),
                value: "renamed-label".into(),
                ..Default::default()
            },
        );
        assert!(owner.GetDocument().SVGResources().is_none());
        assert!(owner.UpdateSVGResources());
        let resources = owner.GetDocument().SVGResources().unwrap();
        assert!(!resources.entries.iter().any(|entry| entry.id == "label"));
        assert_eq!(
            resources
                .entries
                .iter()
                .find(|entry| entry.id == "renamed-label")
                .unwrap()
                .target_node_id,
            label_id
        );
        let resources = resources.clone();
        // Allocating or editing detached nodes cannot change the active catalog.
        let detached = owner.GetDocumentMut().CreateElementDefault(
            dom::persistent_document::DOMNamespace::kSVG,
            "clipPath".into(),
        );
        owner.GetDocumentMut().SetAttribute(
            detached,
            dom::persistent_document::DOMAttribute {
                local_name: "id".into(),
                value: "detached-resource".into(),
                ..Default::default()
            },
        );
        assert!(std::sync::Arc::ptr_eq(
            &resources,
            owner.GetDocument().SVGResources().unwrap()
        ));
        // HTML ancestor selectors may still change native SVG resource style.
        let body = owner.GetDocument().Node(label).Parent().unwrap();
        owner.GetDocumentMut().SetAttribute(
            body,
            dom::persistent_document::DOMAttribute {
                local_name: "class".into(),
                value: "wide".into(),
                ..Default::default()
            },
        );
        assert!(owner.GetDocument().SVGResources().is_some());
        styles
            .Update(&mut owner, &Default::default(), &host_styles())
            .unwrap();
        assert!(owner.GetDocument().SVGResources().is_none());
        assert!(owner.UpdateSVGResources());
        let resources = owner.GetDocument().SVGResources().unwrap();
        let graph = resources
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap()
            .graph
            .as_ref()
            .unwrap();
        match graph.children[0].geometry.as_ref().unwrap() {
            layoutng_assembly::internal::paint_input::SvgClipGeometry::kRect { width, .. } => {
                assert_eq!(width.value, 50.)
            }
            _ => panic!("expected rectangle resource"),
        }
    });
}

mod active_filter_inventory {
    use super::*;
    use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMShadowRootMode};
    use layoutng_assembly::internal::paint_input::{SvgFilterGraph, SvgFilterPrimitiveKind};

    fn element(
        owner: &mut dom::DOM,
        parent: Option<usize>,
        tag: &str,
        id: &str,
        dx: Option<&str>,
        href: Option<&str>,
    ) -> usize {
        let document = owner.GetDocumentMut();
        let node = document.CreateElementDefault(DOMNamespace::kSVG, tag.into());
        for (name, value) in [("id", Some(id)), ("dx", dx), ("href", href)] {
            if let Some(value) = value.filter(|value| !value.is_empty()) {
                document.SetAttribute(
                    node,
                    DOMAttribute {
                        local_name: name.into(),
                        value: value.into(),
                        ..Default::default()
                    },
                );
            }
        }
        if let Some(parent) = parent {
            document.AppendChild(parent, node);
        }
        node
    }
    fn graphs(owner: &mut dom::DOM) -> std::sync::Arc<Vec<SvgFilterGraph>> {
        use layoutng_assembly::layout_engine::{LayoutEngine, LayoutMutation};
        let mut style = style::StyleEngine::new(owner);
        style
            .Update(
                owner,
                &style::media_queries::media_values_cached::MediaValuesCachedData {
                    viewport_width: 320.,
                    viewport_height: 240.,
                    ..Default::default()
                },
                &host_styles(),
            )
            .unwrap();
        owner.UpdateSVGResources();
        let assembly = browser::CreateLayoutAssembly();
        let mut layout = LayoutEngine::new(&assembly);
        layout.ApplyMutation(LayoutMutation::Constraints(
            &browser::CreateBrowserConstraints(320, 240),
        ));
        owner.EmitLayoutMutations(&Default::default(), |mutation| {
            layout.ApplyMutation(mutation)
        });
        layout.Layout();
        layout
            .GetLayoutResult()
            .unwrap()
            .paint
            .style
            .svg_filter_graphs
            .clone()
    }
    fn offset(owner: &mut dom::DOM, id: &str) -> f64 {
        let graphs = graphs(owner);
        let graph = graphs.iter().find(|graph| graph.id == id).unwrap();
        assert!(graph.unsupported.is_none(), "{:?}", graph.unsupported);
        match graph.primitives[0].kind {
            SvgFilterPrimitiveKind::kOffset { dx, .. } => dx,
            _ => panic!("expected inherited offset"),
        }
    }
    #[test]
    fn attached_filters_and_href_follow_connection_tree_order_and_scope() {
        native_test_thread::run(|| {
            let _heap = foundation::LayoutHeapScope::new();
            let mut owner = dom::DOM::new();
            let detached = element(&mut owner, None, "filter", "base", None, None);
            element(&mut owner, Some(detached), "feOffset", "", Some("9"), None);
            let root = owner.GetDocument().Root();
            let svg = element(&mut owner, Some(root), "svg", "", None, None);
            let base = element(&mut owner, Some(svg), "filter", "base", None, None);
            element(&mut owner, Some(base), "feOffset", "", Some("3"), None);
            element(
                &mut owner,
                Some(svg),
                "filter",
                "derived",
                None,
                Some("#base"),
            );
            assert_eq!(
                graphs(&mut owner).len(),
                2,
                "detached filters are not resources"
            );
            assert_eq!(offset(&mut owner, "derived"), 3.);
            let first = element(&mut owner, None, "filter", "base", None, None);
            element(&mut owner, Some(first), "feOffset", "", Some("5"), None);
            owner.GetDocumentMut().InsertBefore(svg, first, base);
            assert_eq!(
                offset(&mut owner, "derived"),
                5.,
                "tree order, not arena order"
            );
            owner.GetDocumentMut().Remove(first);
            assert_eq!(offset(&mut owner, "derived"), 3.);
            owner.GetDocumentMut().Remove(base);
            assert!(
                graphs(&mut owner)
                    .iter()
                    .find(|graph| graph.id == "derived")
                    .unwrap()
                    .unsupported
                    .is_some(),
                "detached old ids cannot satisfy href"
            );
            owner.GetDocumentMut().AppendChild(svg, detached);
            assert_eq!(offset(&mut owner, "derived"), 9.);
            let host = owner
                .GetDocumentMut()
                .CreateElementDefault(DOMNamespace::kHTML, "x-host".into());
            owner.GetDocumentMut().AppendChild(root, host);
            let shadow = owner
                .GetDocumentMut()
                .AttachShadowRoot(host, DOMShadowRootMode::kOpen);
            let shadow_svg = element(&mut owner, Some(shadow), "svg", "", None, None);
            let shadow_base = element(&mut owner, Some(shadow_svg), "filter", "base", None, None);
            element(
                &mut owner,
                Some(shadow_base),
                "feOffset",
                "",
                Some("7"),
                None,
            );
            element(
                &mut owner,
                Some(shadow_svg),
                "filter",
                "shadow-derived",
                None,
                Some("#base"),
            );
            assert_eq!(offset(&mut owner, "shadow-derived"), 7.);
            // Default primitive regions must not crop an offset SVG shape
            // to its original object bounds; retained frames keep old graphs.
            let mut page = fixture(
                r##"<svg width="180" height="100"><defs><filter id="shift" filterUnits="userSpaceOnUse" x="0" y="0" width="180" height="100"><feOffset id="offset" dx="12"/></filter></defs><rect id="filtered" x="16" y="16" width="20" height="20" fill="red" filter="url(#shift)"/></svg>"##,
            );
            let old = page.CurrentFrame().unwrap().display_items.clone();
            assert_eq!(pixel(&current_pixels(&page), 32, 20), [255, 0, 0, 255]);
            set_attribute(&mut page, "offset", "dx", "24");
            assert_eq!(pixel(&current_pixels(&page), 54, 20), [255, 0, 0, 255]);
            assert_eq!(pixel(&current_pixels(&page), 30, 20), [255, 255, 255, 255]);
            assert_eq!(pixel(&pixels(&page, &old), 32, 20), [255, 0, 0, 255]);
            page_replace_children(
                &mut page,
                "shift",
                r#"<feOffset dx="12" result="first"/><feOffset in="first" dx="12"/>"#,
            );
            assert_eq!(pixel(&current_pixels(&page), 54, 20), [255, 0, 0, 255]);
            assert_eq!(pixel(&current_pixels(&page), 30, 20), [255, 255, 255, 255]);
            page_replace_children(&mut page, "shift", r#"<feOffset id="offset" dx="-24"/>"#);
            assert_eq!(pixel(&current_pixels(&page), 4, 20), [255, 0, 0, 255]);
            assert_eq!(pixel(&current_pixels(&page), 14, 20), [255, 255, 255, 255]);
            set_attribute(&mut page, "offset", "dy", "12");
            assert_eq!(pixel(&current_pixels(&page), 4, 32), [255, 0, 0, 255]);
            assert_eq!(pixel(&current_pixels(&page), 4, 20), [255, 255, 255, 255]);
            set_attribute(&mut page, "offset", "dy", "-24");
            assert_eq!(pixel(&current_pixels(&page), 4, 4), [255, 0, 0, 255]);
            set_attribute(&mut page, "offset", "dx", "180");
            assert_eq!(pixel(&current_pixels(&page), 20, 20), [255, 255, 255, 255]);
            set_attribute(&mut page, "offset", "dx", "12");
            set_attribute(&mut page, "offset", "dy", "0");
            set_attribute(&mut page, "filtered", "transform", "translate(10 8)");
            assert_eq!(pixel(&current_pixels(&page), 40, 28), [255, 0, 0, 255]);
            assert_eq!(pixel(&current_pixels(&page), 30, 20), [255, 255, 255, 255]);
        });
    }
}

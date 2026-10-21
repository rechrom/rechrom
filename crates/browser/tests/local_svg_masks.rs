//! Local SVG masks use scoped immutable resources, not image requests.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;
use browser::page::Page;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct Loader {
    html: String,
    requests: Rc<RefCell<Vec<String>>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, r: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.requests.borrow_mut().push(r.url.clone());
        Ok(Box::new(Response(Some(URLResponse {
            final_url: r.url.clone(),
            status_code: 200,
            mime_type: if r.url.split('#').next().unwrap().ends_with("fixture.svg") {
                "image/svg+xml"
            } else if r.url.ends_with("fixture.css") {
                "text/css"
            } else {
                "text/html"
            }
            .into(),
            body: if r.url.split('#').next().unwrap().ends_with("fixture.svg") {
                SVG.as_bytes().to_vec()
            } else if r.url.ends_with("fixture.css") {
                b"#target{mask:url(#left)}".to_vec()
            } else {
                self.html.as_bytes().to_vec()
            },
            ..Default::default()
        }))))
    }
}
fn fixture(markup: &str) -> (Page, Rc<RefCell<Vec<String>>>) {
    let requests = Rc::new(RefCell::new(Vec::new()));
    let assembly = browser::CreateLayoutAssembly();
    let constraints = browser::CreateBrowserConstraints(140, 80);
    let decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page=Page::Create(Rc::new(RefCell::new(Loader{html:format!("<!doctype html><style>body{{margin:0;background:white}}svg,img{{display:block}}</style>{markup}"),requests:requests.clone()})),Rc::new(RefCell::new(image_decoder::skia_image_decoder::SkiaImageDecoder)),Rc::new(RefCell::new(decoder)),constraints,None,None);
    page.Open("https://local-mask.test/", 16384, 4096).unwrap();
    settle(&mut page);
    (page, requests)
}
fn settle(page: &mut Page) {
    for _ in 0..50 {
        page.RunTask().unwrap();
    }
    assert!(page.CurrentFrame().is_some());
}
fn pixels(page: &Page) -> Vec<u32> {
    artifact_pixels(page, &page.CurrentFrame().unwrap().display_items)
}
fn artifact_pixels(
    page: &Page,
    artifact: &std::sync::Arc<paint::paint_engine::PaintArtifact>,
) -> Vec<u32> {
    let mut p = vec![0; 140 * 80];
    page.PaintArtifactInto(
        artifact,
        140,
        80,
        1.,
        &mut p,
        renderer_skia::PixelFormat::Rgba8888,
        140,
    )
    .unwrap();
    p
}
fn px(p: &[u32], x: usize, y: usize) -> [u8; 4] {
    p[y * 140 + x].to_ne_bytes()
}
const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="140" height="80"><mask id="left" mask-type="alpha" maskUnits="userSpaceOnUse" x="0" y="0" width="60" height="60"><rect id="shape" x="10" y="10" width="20" height="40" fill="white"/></mask><mask id="unused"><rect x="70" y="10" width="20" height="30" fill="blue"/></mask><rect id="target" x="10" y="10" width="40" height="40" fill="red" mask="url(#left)"/><rect x="110" y="10" width="10" height="10" fill="green"/></svg>"##;
#[test]
fn local_mask_never_fetches_document_fragment() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (page, requests) = fixture(SVG);
        assert_eq!(&*requests.borrow(), &["https://local-mask.test/"]);
        let owner = page.Document();
        let doc = owner.GetDocument();
        let target = (0..doc.NodeCount())
            .find(|&i| {
                doc.Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "target")
            })
            .unwrap();
        let style = unsafe { &*doc.ResolvedStyleFor(target).unwrap().native_style.Get() };
        let image = unsafe { &*style.MaskLayers().GetImage() };
        assert!(image.IsMaskSource());
        assert!(!image.IsImageResource());
        assert!(image.IsLoaded());
        assert!(!image.IsLoading());
    });
}
#[test]
fn local_mask_pixels_and_hidden_definition() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (page, _) = fixture(SVG);
        let p = pixels(&page);
        assert_eq!(px(&p, 20, 20), [255, 0, 0, 255]);
        assert_eq!(px(&p, 40, 20), [255, 255, 255, 255]);
        assert_eq!(px(&p, 80, 20), [255, 255, 255, 255]);
        assert_eq!(px(&p, 115, 15), [0, 128, 0, 255]);
    });
}
fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let d = owner.GetDocument();
    (0..d.NodeCount())
        .find_map(|i| {
            d.Node(i)
                .FindAttribute("id")
                .filter(|a| a.value == name)
                .map(|_| d.Node(i).Id())
        })
        .unwrap()
}
fn set(page: &mut Page, name: &str, attribute: &str, value: &str) {
    page.Apply(page_mutation::PageMutation::DOMMutation(
        dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kSetAttribute,
            target_node_id: id(page, name),
            name: attribute.into(),
            value: value.into(),
            ..Default::default()
        },
    ))
    .unwrap();
}
fn inner(page: &mut Page, name: &str, value: &str) {
    page.Apply(page_mutation::PageMutation::DOMMutation(
        dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kSetInnerHTML,
            target_node_id: id(page, name),
            value: value.into(),
            ..Default::default()
        },
    ))
    .unwrap();
}
fn graph(page: &Page) -> std::sync::Arc<layoutng_assembly::internal::paint_input::SvgMaskGraph> {
    page.Document()
        .GetDocument()
        .SVGResources()
        .unwrap()
        .entries
        .iter()
        .find(|e| e.id == "left")
        .unwrap()
        .mask_graph
        .clone()
        .unwrap()
}
#[test]
fn canonical_css_overrides_presentation_and_child_geometry() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (mut p, _) = fixture(&format!(
            "<style>#target{{mask:none}}#shape{{width:30px;fill:black}}</style>{SVG}"
        ));
        assert_eq!(px(&pixels(&p), 40, 20), [255, 0, 0, 255]);
        set(&mut p, "target", "style", "mask:url(#left)");
        assert_eq!(px(&pixels(&p), 35, 20), [255, 0, 0, 255]);
        assert_eq!(px(&pixels(&p), 45, 20), [255, 255, 255, 255]);
        set(&mut p, "target", "style", "mask:url(#missing)");
        assert_eq!(px(&pixels(&p), 20, 20), [255, 255, 255, 255]);
    });
}
#[test]
fn mask_type_mode_color_and_opacity_are_canonical() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (mut p, _) = fixture(&SVG.replace("fill=\"white\"", "fill=\"black\""));
        assert_eq!(px(&pixels(&p), 20, 20), [255, 0, 0, 255]);
        set(&mut p, "left", "mask-type", "luminance");
        assert_eq!(px(&pixels(&p), 20, 20), [255, 255, 255, 255]);
        set(&mut p, "target", "style", "mask-mode:alpha");
        assert_eq!(px(&pixels(&p), 20, 20), [255, 0, 0, 255]);
        set(&mut p, "shape", "style", "fill-opacity:.5");
        let c = px(&pixels(&p), 20, 20);
        assert_eq!(c[0], 255);
        assert!((127..=128).contains(&c[1]));
        assert_eq!(c[1], c[2]);
    });
}
#[test]
fn mutation_retargets_resource_and_preserves_old_frames() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (mut p, _) = fixture(SVG);
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let old_fragment = p.CurrentFrame().unwrap().fragments.clone();
        let old_graph = graph(&p);
        let before = artifact_pixels(&p, &old);
        let revision = p.Document().GetDocument().SVGResources().unwrap().version;
        set(&mut p, "shape", "width", "35");
        assert_eq!(px(&pixels(&p), 40, 20), [255, 0, 0, 255]);
        assert_eq!(artifact_pixels(&p, &old), before);
        assert!(!std::sync::Arc::ptr_eq(&old_graph, &graph(&p)));
        assert!(
            p.Document()
                .GetDocument()
                .SVGResources()
                .unwrap()
                .version
                .revision
                > revision.revision
        );
        set(&mut p, "left", "id", "renamed");
        assert_eq!(px(&pixels(&p), 20, 20), [255, 255, 255, 255]);
        set(&mut p, "renamed", "id", "left");
        inner(
            &mut p,
            "left",
            r#"<path d="M10 10H25V50H10Z" fill="white"/>"#,
        );
        assert_eq!(px(&pixels(&p), 20, 20), [255, 0, 0, 255]);
        assert_eq!(px(&pixels(&p), 40, 20), [255, 255, 255, 255]);
        assert_eq!(artifact_pixels(&p, &old), before);
        assert!(std::rc::Rc::strong_count(&old_fragment) >= 1);
    });
}
#[test]
fn empty_missing_and_first_wrong_kind_stay_mask_slots() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (mut p, _) = fixture(SVG);
        inner(&mut p, "left", "");
        assert!(graph(&p).commands.is_empty());
        assert_eq!(px(&pixels(&p), 20, 20), [255, 255, 255, 255]);
        let (p, requests) = fixture(&SVG.replace(
            "<mask id=\"left\"",
            "<defs><rect id=\"left\"/></defs><mask id=\"left\"",
        ));
        assert_eq!(px(&pixels(&p), 20, 20), [255, 255, 255, 255]);
        assert_eq!(requests.borrow().len(), 1);
    });
}
#[test]
fn local_url_identity_normalizes_fragments_and_same_document() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        for url in ["#%6ceft", "https://local-mask.test/#left"] {
            let (p, r) = fixture(&SVG.replace("url(#left)", &format!("url({url})")));
            assert_eq!(r.borrow().len(), 1, "{url}: {:?}", r.borrow());
            assert_eq!(px(&pixels(&p), 20, 20), [255, 0, 0, 255], "{url}");
        }
        let (p, r) =
            fixture(&SVG.replace("url(#left)", "url(https://local-mask.test/?other#left)"));
        assert_eq!(r.borrow().len(), 2);
        assert_eq!(px(&pixels(&p), 20, 20), [255, 255, 255, 255]);
        let (_, r) = fixture(&format!(
            "<div style='background-image:url(#left);width:10px;height:10px'></div>{SVG}"
        ));
        assert_eq!(r.borrow().len(), 2, "background fragments remain fetched");
    });
}
#[test]
fn object_bounding_box_shape_masks_and_layer_composites() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let body = r##"<svg xmlns="http://www.w3.org/2000/svg" width="140" height="80"><mask id="left" maskContentUnits="objectBoundingBox" mask-type="alpha"><path d="M0 0H.5V1H0Z" fill="white"/></mask><rect id="target" x="10" y="10" width="40" height="40" fill="red" mask="url(#left)"/></svg>"##;
        let (mut p, _) = fixture(body);
        assert_eq!(px(&pixels(&p), 20, 20), [255, 0, 0, 255]);
        assert_eq!(px(&pixels(&p), 40, 20), [255, 255, 255, 255]);
        set(
            &mut p,
            "target",
            "style",
            "mask-image:url(#left),none;mask-composite:intersect",
        );
        assert_eq!(px(&pixels(&p), 20, 20), [255, 255, 255, 255]);
        set(
            &mut p,
            "target",
            "style",
            "mask-image:url(#left),none;mask-composite:add",
        );
        assert_eq!(px(&pixels(&p), 20, 20), [255, 0, 0, 255]);
    });
}
#[test]
fn unsupported_content_retains_explicit_resource_diagnostic() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        for replacement in [
            r#"<text x="10" y="20">x</text>"#,
            r#"<path d="M10 10H30V50H10Z" transform="translate(1)"/>"#,
            r#"<rect width="50%" height="20"/>"#,
        ] {
            let result = std::panic::catch_unwind(|| {
                fixture(&SVG.replace(
                    r#"<rect id="shape" x="10" y="10" width="20" height="40" fill="white"/>"#,
                    replacement,
                ))
            });
            let error = result.err().expect("unsupported mask must reject");
            let typed = error
                .downcast_ref::<foundation::UnsupportedLayout>()
                .expect("typed unsupported mask diagnostic");
            assert!(
                typed
                    .to_string()
                    .starts_with("unsupported SVG mask resource:"),
                "{replacement}: {typed}"
            );
        }
    });
}
#[test]
fn real_icon_mask_resource_preserves_interior_ink() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let icon = include_str!("fixtures/local_mask_icon.svg");
        let mask_start = icon.find("<mask ").unwrap();
        let mask_end = icon.find("</mask>").unwrap() + 7;
        let control = format!("{}{}", &icon[..mask_start], &icon[mask_end..])
            .replace(" mask=\"url(#mask0)\"", "");
        let (p, r) = fixture(icon);
        let (q, _) = fixture(&control);
        assert_eq!(r.borrow().len(), 1);
        let actual = pixels(&p);
        let expected = pixels(&q);
        let differences = actual.iter().zip(&expected).filter(|(a, b)| a != b).count();
        eprintln!(
            "icon diagnostic changed pixels versus unmasked control: {differences}; samples {:?}",
            [(10, 2), (3, 9), (16, 9), (8, 5), (10, 9), (10, 16)].map(|(x, y)| px(&actual, x, y))
        );
        for (x, y) in [(10, 2), (3, 9), (16, 9), (10, 9)] {
            assert_eq!(px(&actual, x, y), [0, 0, 0, 255], "bulb ink at ({x},{y})");
        }
        assert_eq!(
            px(&actual, 8, 5),
            [255, 255, 255, 255],
            "hollow bulb interior"
        );
    });
}

fn host_styles() -> Vec<cssom::CSSStyleSheet> {
    vec![
        style::ParseUserAgentCSS(include_str!("../resources/user_agent/html.css")),
        style::ParseUserAgentCSSWithRole(
            include_str!("../resources/user_agent/svg.css"),
            cssom::CSSUserAgentStyleSheetRole::kSVG,
        ),
    ]
}
fn find_fragment(
    node: &layoutng_assembly::fragment_tree::FragmentNode,
    wanted: u64,
) -> Option<&layoutng_assembly::fragment_tree::FragmentNode> {
    if node.node_id == wanted {
        Some(node)
    } else {
        node.children.iter().find_map(|c| find_fragment(c, wanted))
    }
}
#[test]
fn inherited_mask_reference_keeps_originating_scope_across_shadow_root() {
    native_test_thread::run(|| {
        use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMShadowRootMode};

        use layoutng_assembly::layout_engine::{LayoutEngine, LayoutMutation};
        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = html::html_parser::ParseHTML(
            r##"<!doctype html><body><svg width="1" height="1"><defs><mask id="clip"><rect width="40" height="40"/></mask></defs></svg><div id="host" style="width:80px;height:80px;mask-image:url(#clip)"></div>"##,
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
                ("style", "mask-image:inherit"),
            ],
        );
        let clip = add(document, svg, "mask", &[("id", "clip")]);
        add(document, clip, "rect", &[("width", "10"), ("height", "10")]);
        let own = add(
            document,
            svg,
            "rect",
            &[("width", "60"), ("height", "60"), ("mask", "url(#clip)")],
        );
        let inherited = add(
            document,
            svg,
            "rect",
            &[
                ("width", "60"),
                ("height", "60"),
                ("style", "mask-image:inherit"),
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
            .mask_images[0]
            .svg_reference
            .as_ref()
            .unwrap();
        let inherited_clip = find_fragment(fragments, inherited_id)
            .unwrap()
            .paint
            .style
            .mask_images[0]
            .svg_reference
            .as_ref()
            .unwrap();
        assert_eq!(own_clip.scope, shadow_scope);
        assert_eq!(inherited_clip.scope, document_scope);
        let width = |clip: &layoutng_assembly::internal::paint_input::SvgMaskReference| match &clip
            .graph
            .as_ref()
            .unwrap()
            .commands[0]
        {
            layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawShape(shape) => {
                shape.shape.bounds_width
            }
            _ => panic!("expected canonical shape record"),
        };
        assert_eq!(width(own_clip), 10.0);
        assert_eq!(width(inherited_clip), 40.0);
    });
}
#[test]
fn mask_record_uses_canonical_stroke_fillrule_and_suppression() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let replacement = r#"<path id="ink" d="M10 10H40V40H10Z M20 20H30V30H20Z" fill="white" fill-rule="evenodd" stroke="white" stroke-width="4" stroke-linecap="round" stroke-linejoin="bevel"/><rect width="80" height="80" style="display:none"/>"#;
        let (mut p, _) = fixture(&SVG.replace(
            r#"<rect id="shape" x="10" y="10" width="20" height="40" fill="white"/>"#,
            replacement,
        ));
        let g = graph(&p);
        assert!(g.unsupported.is_none());
        let layoutng_assembly::internal::paint_input::SvgPatternCommand::DrawShape(shape) =
            &g.commands[0]
        else {
            panic!("shape record")
        };
        assert_eq!(shape.stroke_width, 4.);
        assert!(shape.fill_even_odd);
        assert_eq!(
            shape.stroke_line_cap,
            layoutng_assembly::internal::paint_input::SvgStrokeLineCap::kRound
        );
        assert_eq!(px(&pixels(&p), 25, 25), [255, 255, 255, 255]);
        set(&mut p, "ink", "style", "fill-rule:nonzero;stroke-width:8");
        assert_eq!(px(&pixels(&p), 25, 25), [255, 0, 0, 255]);
    });
}
#[test]
fn unsupported_group_mask_fails_before_empty_bounds_culling() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let html = r##"<svg width="140" height="80"><mask id="m" maskUnits="userSpaceOnUse" x="0" y="0" width="80" height="80"><text>x</text></mask><g mask="url(#m)"><rect width="40" height="40" fill="red"/></g></svg>"##;
        let error = std::panic::catch_unwind(|| fixture(html))
            .err()
            .expect("unsupported group must reject");
        assert!(error
            .downcast_ref::<foundation::UnsupportedLayout>()
            .is_some());
    });
}
#[test]
fn paint_only_mask_update_retains_viewport_and_old_artifact() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (mut p, _) = fixture(SVG);
        let old = p.CurrentFrame().unwrap().display_items.clone();
        let before = artifact_pixels(&p, &old);
        let target = id(&p, "target");
        p.Apply(page_mutation::PageMutation::AnimationTick(
            page_mutation::AnimationTick {
                monotonic_time: 1.,
                samples: vec![page_mutation::AnimationStyleSample {
                    node_id: target,
                    effect_id: 7,
                    declarations: style::ParseCSSDeclarationList("opacity:0.5"),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ))
        .unwrap();
        let c = px(&pixels(&p), 20, 20);
        assert!((127..=128).contains(&c[1]));
        let mask = &find_fragment(&p.CurrentFrame().unwrap().fragments, target)
            .unwrap()
            .paint
            .style
            .mask_images[0];
        assert!(mask.svg_reference.as_ref().unwrap().viewport.is_some());
        assert_eq!(artifact_pixels(&p, &old), before);
    });
}
#[test]
fn external_stylesheet_fragment_uses_originating_document() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let(p,r)=fixture(&format!("<base href='https://different-base.test/'><link rel='stylesheet' href='https://stylesheet.test/fixture.css'>{}",SVG.replace("mask=\"url(#left)\"","mask=\"none\"")));
        assert_eq!(
            &*r.borrow(),
            &[
                "https://local-mask.test/",
                "https://stylesheet.test/fixture.css"
            ]
        );
        assert_eq!(px(&pixels(&p), 20, 20), [255, 0, 0, 255]);
        assert_eq!(px(&pixels(&p), 40, 20), [255, 255, 255, 255]);
    });
}
#[test]
fn local_shape_mask_equals_direct_geometry_fixture() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (p, _) = fixture(SVG);
        let (q, _) = fixture(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="140" height="80"><rect x="10" y="10" width="20" height="40" fill="red"/><rect x="110" y="10" width="10" height="10" fill="green"/></svg>"#,
        );
        assert!(
            pixels(&p) == pixels(&q),
            "mask must equal intended nonempty direct geometry exactly"
        );
    });
}
#[test]
fn user_space_mask_survives_consumer_viewbox_transform() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (p, _) = fixture(&SVG.replace(
            "width=\"140\" height=\"80\"",
            "width=\"140\" height=\"80\" viewBox=\"0 0 70 40\"",
        ));
        assert_eq!(px(&pixels(&p), 40, 40), [255, 0, 0, 255]);
        assert_eq!(px(&pixels(&p), 80, 40), [255, 255, 255, 255]);
    });
}

#[test]
fn existing_pattern_mask_reads_canonical_override() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let source = r##"<svg width="140" height="80"><defs><mask id="m" maskUnits="userSpaceOnUse" x="0" y="0" width="40" height="40" mask-type="alpha"><rect width="20" height="40" fill="white"/></mask><pattern id="p" patternUnits="userSpaceOnUse" width="40" height="40"><g id="group" mask="url(#m)"><rect width="40" height="40" fill="red"/></g></pattern></defs><rect id="painted" width="40" height="40" fill="url(#p)"/></svg>"##;
        let (mut p, r) = fixture(source);
        assert_eq!(r.borrow().len(), 1);
        let has_mask = |page: &Page| {
            let fragment =
                find_fragment(&page.CurrentFrame().unwrap().fragments, id(page, "painted"))
                    .unwrap();
            fragment
                .paint
                .style
                .svg_fill_server
                .as_ref()
                .unwrap()
                .pattern
                .as_ref()
                .unwrap()
                .commands
                .iter()
                .any(|command| {
                    matches!(
                        command,
                        layoutng_assembly::internal::paint_input::SvgPatternCommand::SaveLayerMask(
                            _
                        )
                    )
                })
        };
        assert!(has_mask(&p));
        // Existing LayerTile admission rejects pattern-shaded SVG paths. This
        // resource-record regression does not broaden that separate renderer.
        let raster_error = |page: &Page| {
            let mut bytes = vec![0; 140 * 80];
            page.PaintArtifactInto(
                &page.CurrentFrame().unwrap().display_items,
                140,
                80,
                1.,
                &mut bytes,
                renderer_skia::PixelFormat::Rgba8888,
                140,
            )
            .unwrap_err()
            .to_string()
        };
        assert_eq!(raster_error(&p), "unsupported-record-resource-or-effect");
        set(&mut p, "group", "style", "mask:none");
        assert!(
            !has_mask(&p),
            "canonical none must override presentation mask"
        );
        assert_eq!(
            raster_error(&p),
            "unsupported-record-resource-or-effect",
            "unmasked control has the same pre-existing raster limit"
        );
    });
}
#[test]
fn fetched_svg_document_keeps_local_mask_and_hidden_definition() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let (p, _) = fixture(SVG);
        let (q, r) = fixture("<img src='fixture.svg' width='140' height='80'>");
        assert_eq!(r.borrow().len(), 2);
        let inline = pixels(&p);
        let fetched = pixels(&q);
        let changed: Vec<_> = inline
            .iter()
            .zip(&fetched)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| (i % 140, i / 140, a.to_ne_bytes(), b.to_ne_bytes()))
            .collect();
        eprintln!(
            "fetched-mask differences {} samples {:?}",
            changed.len(),
            &changed[..changed.len().min(40)]
        );
        if let Ok(dir) = std::env::var("LOCAL_MASK_EVIDENCE") {
            std::fs::create_dir_all(&dir).unwrap();
            for (name, values) in [("inline-mask", &inline), ("fetched-mask", &fetched)] {
                let bytes: Vec<u8> = values.iter().flat_map(|p| p.to_ne_bytes()).collect();
                std::fs::write(
                    std::path::Path::new(&dir).join(format!("{name}.png")),
                    raster::EncodeRgbaPng(&bytes, 140, 80),
                )
                .unwrap();
            }
        }
        assert!(inline == fetched);
    });
}
#[test]
fn external_svg_mask_target_selection_is_explicitly_unsupported() {
    native_test_thread::run(|| {
        let _h = foundation::LayoutHeapScope::new();
        let error=std::panic::catch_unwind(||fixture("<div style='width:40px;height:40px;background:red;mask:url(fixture.svg#left)'></div>")).err().expect("external target must reject");
        let typed = error
            .downcast_ref::<foundation::UnsupportedLayout>()
            .expect("typed external SVG diagnostic");
        assert!(typed
            .to_string()
            .contains("external SVG mask target selection"));
    });
}

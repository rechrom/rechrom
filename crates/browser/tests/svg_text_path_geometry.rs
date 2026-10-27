//! Source-aligned SVG transformed visual bounds through production Page.
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

fn circle_markup(overflow: &str, size: u32, style: &str, adjust: &str) -> String {
    let marks = (0..32)
        .map(|i| {
            if i % 2 == 0 {
                " / "
            } else {
                " <tspan>/</tspan> "
            }
        })
        .collect::<String>();
    format!(
        r##"<div style="margin-left:80px;width:200px;height:200px;overflow:{overflow}"><svg width="{size}" height="{size}" viewBox="0 0 300 300" style="margin-left:-60px"><defs style="display:none"><path id="path" d="M150 150 m-110 0 a110 110 0 1 1 220 0 a110 110 0 1 1 -220 0"/></defs><text id="text" dy="40" textLength="690" style="font:12px SvgTextPathFixture;{style}"><textPath textLength="690" lengthAdjust="{adjust}" href="#path">{marks}</textPath></text></svg></div>"##
    )
}

fn assert_inside_clip_equal(visible: &Page, clipped: &Page) {
    let left = pixels(visible);
    let right = pixels(clipped);
    let mut ink = 0;
    let mut changed = 0;
    for y in 0..200 {
        for x in 80..280 {
            let index = y * 320 + x;
            if left[index] != 0xffff_ffff {
                ink += 1;
            }
            if left[index] != right[index] {
                changed += 1;
            }
        }
    }
    assert!(
        ink > 100,
        "the reference must contain real visible text, got {ink}"
    );
    assert_eq!(
        changed, 0,
        "clipping must preserve every inside glyph pixel"
    );
}

#[test]
fn circular_text_clip_preserves_inside_glyphs() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let visible = fixture(&circle_markup("visible", 300, "", "spacing"));
        let clipped = fixture(&circle_markup("hidden", 300, "", "spacing"));
        assert_inside_clip_equal(&visible, &clipped);
    });
}

#[test]
fn fractional_viewbox_and_length_adjust_keep_svg_clip_bounds() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for size in [240, 375] {
            for adjust in ["spacing", "spacingAndGlyphs"] {
                let visible = fixture(&circle_markup("visible", size, "", adjust));
                let clipped = fixture(&circle_markup("hidden", size, "", adjust));
                assert_inside_clip_equal(&visible, &clipped);
            }
        }
    });
}

#[test]
fn stroked_and_shadowed_path_text_keep_inside_clip_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for style in [
            "stroke:black;stroke-width:2",
            "text-shadow:8px 3px 2px #555",
        ] {
            let visible = fixture(&circle_markup("visible", 300, style, "spacing"));
            let clipped = fixture(&circle_markup("hidden", 300, style, "spacing"));
            assert_inside_clip_equal(&visible, &clipped);
        }
    });
}

#[test]
fn direct_rotated_svg_text_keeps_inside_clip_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let markup = |overflow| {
            format!(
                r##"<div style="margin-left:80px;width:200px;height:200px;overflow:{overflow}"><svg width="300" height="240" style="margin-left:-60px"><text id="text" x="50" y="40" rotate="90" style="font:24px SvgTextPathFixture">WIDE <tspan>TEXT</tspan></text></svg></div>"##
            )
        };
        let visible = fixture(&markup("visible"));
        let clipped = fixture(&markup("hidden"));
        assert_inside_clip_equal(&visible, &clipped);
    });
}

#[test]
fn retained_svg_bounds_refresh_after_dy_mutation_and_preserve_old_artifact() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(&circle_markup("hidden", 300, "", "spacing"));
        let original = native(&page, "text") as *const LayoutObject;
        let before = pixels(&page);
        let old = page.CurrentFrame().unwrap().display_items.clone();
        set_attribute(&mut page, "text", "dy", "60");
        page.RunTask().unwrap();
        assert_eq!(native(&page, "text") as *const LayoutObject, original);
        assert_ne!(pixels(&page), before);
        let mut old_pixels = vec![0; 320 * 240];
        page.PaintArtifactInto(
            &old,
            320,
            240,
            1.0,
            &mut old_pixels,
            renderer_skia::PixelFormat::Rgba8888,
            320,
        )
        .unwrap();
        assert_eq!(old_pixels, before);
        let mut visible = fixture(&circle_markup("visible", 300, "", "spacing"));
        set_attribute(&mut visible, "text", "dy", "60");
        visible.RunTask().unwrap();
        assert_inside_clip_equal(&visible, &page);
    });
}

fn first_svg_data(
    node: &layoutng_assembly::fragment_tree::FragmentPaintData,
) -> Option<&layoutng_assembly::fragment_tree::SvgTextPaintData> {
    if let Some(svg) = node.paint.svg_text.as_deref() {
        return Some(svg);
    }
    node.children.iter().find_map(|child| first_svg_data(child))
}

#[test]
fn svg_parent_visual_bounds_expand_with_percentage_stroke_and_shadow() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let markup = |style| {
            format!(
                r##"<svg width="300" height="240" viewBox="0 0 300 240"><text id="text" x="20" y="80" style="font:24px SvgTextPathFixture;{style}">Bounds</text></svg>"##
            )
        };
        let plain = fixture(&markup(""));
        let stroke = fixture(&markup("stroke:black;stroke-width:2%"));
        let shadow = fixture(&markup("text-shadow:8px 3px 2px #555"));
        let rect = |page: &Page| {
            first_svg_data(&page.CurrentFrame().unwrap().fragments)
                .unwrap()
                .visual_rect
        };
        let base = rect(&plain);
        let stroked = rect(&stroke);
        let shadowed = rect(&shadow);
        let outset = ((300_f64 * 300.0 + 240.0 * 240.0) / 2.0).sqrt() * 0.02;
        assert!((stroked.size.width - base.size.width - 2.0 * outset).abs() < 0.001);
        assert!((stroked.size.height - base.size.height - 2.0 * outset).abs() < 0.001);
        assert!((shadowed.size.width - base.size.width - 11.0).abs() < 0.001);
        assert!((shadowed.size.height - base.size.height - 6.0).abs() < 0.001);
    });
}

#[test]
fn native_shadow_extents_keep_original_and_skip_inset() {
    use foundation::gfx;
    use layoutng_style::css::style_color::StyleColor;
    use layoutng_style::style::shadow_data::{ShadowData, ShadowStyle};
    use layoutng_style::style::shadow_list::{ShadowDataVector, ShadowList};
    let normal = ShadowData::new(
        gfx::Vector2dF::new(-5.0, 3.0),
        2.2,
        -1.0,
        ShadowStyle::kNormal,
        StyleColor::CurrentColor(),
        1.0,
    );
    let outsets = normal.RectOutsets();
    assert_eq!(
        [
            outsets.top(),
            outsets.right(),
            outsets.bottom(),
            outsets.left()
        ],
        [0.0, -2.0, 6.0, 8.0]
    );
    let inset = ShadowData::new(
        gfx::Vector2dF::new(100.0, 100.0),
        100.0,
        100.0,
        ShadowStyle::kInset,
        StyleColor::CurrentColor(),
        1.0,
    );
    let shadows = ShadowList::new(ShadowDataVector::from(vec![normal, inset]));
    let mut rect = gfx::RectF::new(gfx::PointF::new(10.0, 20.0), gfx::SizeF::new(30.0, 40.0));
    shadows.AdjustRectForShadow(&mut rect);
    assert_eq!(
        [rect.x(), rect.y(), rect.width(), rect.height()],
        [2.0, 20.0, 38.0, 46.0]
    );
}

//! Derived z-index eligibility must reach native layout and paint order through
//! the production Style owner, including later style changes.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use layoutng_assembly::{fragment_tree::FragmentNode, internal::layout_input::FontFace};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Operation(Option<URLResponse>);
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct FixtureLoader(Vec<u8>);
impl URLLoader for FixtureLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: self.0.clone(),
            ..Default::default()
        }))))
    }
}
struct XHR;
impl xhr_transport::XMLHttpRequestTransport for XHR {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("unexpected fixture XHR"))
    }
}

fn fixture(css: &str, children: &str) -> Page {
    let html = format!("<!doctype html><style>body{{margin:32px;font:16px/1 ButtonFixture}}{css}</style>{children}");
    let assembly = browser::CreateLayoutAssembly();
    let mut space = browser::CreateBrowserConstraints(640, 480);
    space.fonts = vec![FontFace {
        family: "ButtonFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(html.into_bytes()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        space,
        Some(browser::page::ScriptEnvironment {
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_native_stack_budget(
                    4 * 1024 * 1024,
                ),
            ),
            xhr: Box::new(XHR),
            user_agent: "button-fixture".into(),
        }),
        None,
    );
    page.Open("https://page.test/button.html", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
}

fn node_id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find(|&i| {
            document
                .Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == name)
        })
        .map(|i| document.Node(i).Id())
        .unwrap()
}
fn flags(page: &Page, name: &str) -> (bool, bool, i32) {
    let owner = page.Document();
    let document = owner.GetDocument();
    let i = (0..document.NodeCount())
        .find(|&i| document.Node(i).Id() == node_id(page, name))
        .unwrap();
    let style = document.ResolvedStyleFor(i).unwrap();
    let native = unsafe { &*style.native_style.Get() };
    (
        native.AllowsZIndex(),
        native.ForcesStackingContext(),
        native.EffectiveZIndex(),
    )
}
fn fragment(node: &FragmentNode, id: u64) -> &FragmentNode {
    fn find(node: &FragmentNode, id: u64) -> Option<&FragmentNode> {
        if node.node_id == id {
            Some(node)
        } else {
            node.children.iter().find_map(|child| find(child, id))
        }
    }
    find(node, id).unwrap()
}
fn first_background(frame: &browser::page::PageFrame, id: u64) -> usize {
    frame
        .display_items
        .items
        .iter()
        .position(|item| {
            item.node_id == id && item.r#type == paint::paint_engine::DisplayItemType::kDrawRect
        })
        .unwrap()
}
fn mutate(page: &mut Page, script: &str) {
    assert!(page.Evaluate(script, "test:stacking").unwrap().Succeeded());
    page.RunTask().unwrap();
}
const BUTTON_CSS: &str = ".wrapper{text-align:center} #front{display:inline;position:relative;z-index:1;outline:16px solid white;border:1px solid #a2a9b1;background:#f8f9fa;padding:6px 12px} #back{position:relative;width:460px;margin:-16px auto 16px;height:1px;background:#c8ccd1}";
const BUTTON_HTML: &str =
    "<div class=wrapper><button id=front>Read in your language</button></div><div id=back></div>";

#[test]
fn positioned_descendant_background_text_and_outline_cover_later_auto_layer() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(BUTTON_CSS, BUTTON_HTML);
        assert_eq!(flags(&page, "front"), (true, true, 1));
        assert_eq!(flags(&page, "back"), (true, false, 0));
        let frame = page.CurrentFrame().unwrap();
        let front = node_id(&page, "front");
        let back = node_id(&page, "back");
        let owned = fragment(&frame.fragments, front);
        assert!(owned.paint.paint_layer_is_self_painting);
        assert!(owned.paint.paint_layer_is_stacking_context);
        assert_eq!(owned.paint.paint_layer_stacking_level, 1);
        let rule = first_background(&frame, back);
        assert!(rule < first_background(&frame, front));
        let outline = frame
            .display_items
            .items
            .iter()
            .position(|item| {
                item.node_id == front
                    && item.r#type == paint::paint_engine::DisplayItemType::kStrokeRect
            })
            .unwrap();
        let glyph = frame
            .display_items
            .items
            .iter()
            .position(|item| item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun)
            .unwrap();
        assert!(rule < glyph && glyph < outline);
    });
}

#[test]
fn changing_position_z_index_and_auto_recomputes_flags_without_changing_old_frame() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(BUTTON_CSS, BUTTON_HTML);
        let front = node_id(&page, "front");
        let back = node_id(&page, "back");
        let current = page.CurrentFrame().unwrap();
        let retained = browser::page::PageFrame {
            sequence: current.sequence,
            compositor_scroll_sequence: current.compositor_scroll_sequence,
            fragments: current.fragments.clone(),
            display_items: current.display_items.clone(),
            document_image_records_complete: current.document_image_records_complete,
        };
        for (script, expected) in [
            (
                "document.getElementById('front').style.zIndex='auto'",
                (true, false, 0),
            ),
            (
                "document.getElementById('front').style.zIndex='-1'",
                (true, true, -1),
            ),
            (
                "document.getElementById('front').style.position='static'",
                (false, false, 0),
            ),
            (
                "document.getElementById('front').style.position='relative'",
                (true, true, -1),
            ),
            (
                "document.getElementById('front').style.zIndex='1'",
                (true, true, 1),
            ),
        ] {
            mutate(&mut page, script);
            assert_eq!(flags(&page, "front"), expected, "{script}");
        }
        assert_eq!(
            fragment(&retained.fragments, front)
                .paint
                .paint_layer_stacking_level,
            1
        );
        assert!(first_background(&retained, back) < first_background(&retained, front));
    });
}

#[test]
fn static_flex_and_grid_items_get_stacking_and_revert_with_parent_display() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for display in ["flex", "inline-flex", "grid", "inline-grid"] {
            let mut page = fixture(&format!("#parent{{display:{display}}} .item{{width:80px;height:20px;background:red;z-index:2}}"), "<div id=parent><div id=front class=item></div><div id=copy class=item></div></div>");
            assert_eq!(flags(&page, "front"), (true, true, 2), "{display}");
            assert_eq!(
                flags(&page, "copy"),
                (true, true, 2),
                "reused declaration {display}"
            );
            mutate(
                &mut page,
                "document.getElementById('parent').style.display='block'",
            );
            assert_eq!(
                flags(&page, "front"),
                (false, false, 0),
                "{display} -> block"
            );
            mutate(
                &mut page,
                &format!("document.getElementById('parent').style.display='{display}'"),
            );
            assert_eq!(flags(&page, "front"), (true, true, 2), "restore {display}");
        }
    });
}

#[test]
fn positioned_z_index_applies_to_all_position_schemes_but_not_static_blocks() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for position in ["static", "relative", "absolute", "fixed", "sticky"] {
            let page = fixture(
                &format!(
                    "#front{{position:{position};z-index:7;width:20px;height:20px;background:red}}"
                ),
                "<div id=front></div>",
            );
            assert_eq!(
                flags(&page, "front"),
                if position == "static" {
                    (false, false, 0)
                } else {
                    (true, true, 7)
                },
                "{position}"
            );
        }
    });
}

#[test]
fn descendant_z_index_stays_in_its_explicit_stacking_context() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture("#parent{position:relative;z-index:0} #front{position:relative;z-index:10;background:red;width:30px;height:30px} #back{position:relative;z-index:1;background:blue;width:30px;height:30px;margin-top:-30px}", "<div id=parent><div id=front></div></div><div id=back></div>");
        let frame = page.CurrentFrame().unwrap();
        assert_eq!(flags(&page, "parent"), (true, true, 0));
        assert!(
            first_background(&frame, node_id(&page, "front"))
                < first_background(&frame, node_id(&page, "back"))
        );
    });
}

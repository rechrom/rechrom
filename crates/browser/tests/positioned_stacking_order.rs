//! Derived z-index eligibility must reach native layout and paint order through
//! the production Style owner, including later style changes.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use foundation::DynamicTo;
use layoutng_assembly::{
    fragment_tree::{FragmentPaintData, LayoutResultHandle},
    internal::{
        layout_box_model_object::LayoutBoxModelObject, layout_input::FontFace,
        layout_object::LayoutObject,
    },
    layout_engine::PaintFragmentCursor,
};
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
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_max_stack_size(
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
fn layout_object(page: &Page, name: &str) -> *mut LayoutObject {
    let wanted = node_id(page, name);
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject as *mut LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == wanted {
            return object;
        }
        object = current.NextInPreOrder(root);
    }
    panic!("layout object for {name} missing")
}
fn flags(page: &Page, name: &str) -> (bool, bool, i32) {
    let owner = page.Document();
    let document = owner.GetDocument();
    let i = (0..document.NodeCount())
        .find(|&i| document.Node(i).Id() == node_id(page, name))
        .unwrap();
    let style = document.ResolvedStyleFor(i).unwrap();
    let native = unsafe { &*style.computed_style_owner.Get() };
    (
        native.AllowsZIndex(),
        native.ForcesStackingContext(),
        native.EffectiveZIndex(),
    )
}
fn fragment(handle: &LayoutResultHandle, id: u64) -> &FragmentPaintData {
    fn find(node: PaintFragmentCursor<'_>, id: u64) -> Option<PaintFragmentCursor<'_>> {
        if node.PaintData().node_id == id {
            Some(node)
        } else {
            node.Children()
                .into_iter()
                .find_map(|child| find(child, id))
        }
    }
    find(PaintFragmentCursor::FromHandle(handle), id)
        .map(PaintFragmentCursor::PaintData)
        .unwrap()
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
fn self_painting_inline_recalculates_fragment_item_overflow_without_visiting_layout_text() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(BUTTON_CSS, BUTTON_HTML);
        let inline = layout_object(&page, "front");
        assert!(unsafe { &*inline }.IsInline());
        assert!(unsafe { &*inline }.IsInLayoutNGInlineFormattingContext());
        let box_model = DynamicTo::<LayoutBoxModelObject>(inline);
        assert!(!box_model.is_null());
        assert!(unsafe { &*box_model }.HasSelfPaintingLayer());

        // Before the LayoutBoxModelObject inline-fragment dispatch existed,
        // this base traversal reached its LayoutText child and hit Chromium's
        // deliberate LayoutText::RecalcVisualOverflow NOTREACHED assertion.
        unsafe { &mut *inline }.RecalcVisualOverflow();
    });
}

#[test]
fn inline_svg_root_recalculates_through_layout_replaced_override() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            "svg{display:inline;width:24px;height:24px}",
            "<svg id=icon viewBox='0 0 24 24'><path d='M2 2h20v20H2z'/></svg>",
        );
        let svg_root = layout_object(&page, "icon");
        assert!(unsafe { &*svg_root }.IsSVGRoot());
        assert!(unsafe { &*svg_root }.IsLayoutReplaced());

        // LayoutSVGRoot derives from LayoutReplaced. Falling through to the
        // LayoutBoxModelObject override enters the inline-fragment cursor a
        // second time and recursively revisits this same SVG root.
        unsafe { &mut *svg_root }.RecalcVisualOverflow();
    });
}

#[test]
fn anchor_auto_outline_includes_its_out_of_flow_descendant_only_for_anchor() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            ".focus{display:block;position:relative;width:20px;height:20px;outline:auto;margin-bottom:8px}.child{position:absolute;left:80px;top:0;width:20px;height:10px;transform:translateX(20px)}",
            "<a id=anchor class=focus><span class=child></span></a><div id=plain class=focus><span class=child></span></div>",
        );
        let frame = page.CurrentFrame().unwrap();
        let anchor = fragment(&frame.fragments, node_id(&page, "anchor"));
        let plain = fragment(&frame.fragments, node_id(&page, "plain"));
        let anchor_overflow = anchor
            .paint
            .self_ink_overflow
            .expect("anchor self ink overflow");
        let plain_overflow = plain
            .paint
            .self_ink_overflow
            .expect("plain self ink overflow");
        let anchor_right = anchor_overflow.offset.x + anchor_overflow.size.width;
        let plain_right = plain_overflow.offset.x + plain_overflow.size.width;
        assert!(
            anchor_right > 100.0,
            "anchor focus outline must include its absolute descendant: {anchor_right}"
        );
        assert!(
            plain_right < 50.0,
            "non-anchor focus outline must omit the absolute descendant: {plain_right}"
        );
    });
}

#[test]
fn positioned_descendant_background_text_and_outline_cover_later_auto_layer() {
    test_thread::run(|| {
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
    test_thread::run(|| {
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
    test_thread::run(|| {
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
    test_thread::run(|| {
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
    test_thread::run(|| {
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

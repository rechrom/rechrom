#[path = "../src/native_test_thread.rs"]
mod native_test_thread;
use browser::page::Page;
use document_image::SVGImageDecoder;
use image_decoder::skia_image_decoder::SkiaImageDecoder;
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
struct Operation(Option<URLResponse>);
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct FixtureLoader;
impl URLLoader for FixtureLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {final_url:request.url.clone(),status_code:200,mime_type:"text/html".into(),
            body:br#"<!doctype html><style>
            body { margin:0; background:white; }
            input,textarea { display:block; box-sizing:border-box; width:300px; height:72px;
                margin:12px; border:2px solid black; padding:8px; font:20px/48px sans-serif; }
            </style><input id="edit"><input id="hint" placeholder="placeholder"><textarea id="area"></textarea>"#.to_vec(),..Default::default()}))))
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

fn editing_fixture() -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut space = browser::CreateBrowserConstraints(640, 480);
    space.fonts = vec![layoutng_assembly::internal::layout_input::FontFace {
        family: "sans-serif".into(),
        bytes: include_bytes!(
            "../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
        )
        .as_slice()
        .into(),
        ..Default::default()
    }];
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader)),
        Rc::new(RefCell::new(SkiaImageDecoder)),
        Rc::new(RefCell::new(SVGImageDecoder::new(&assembly))),
        space,
        Some(browser::page::ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_native_stack_budget(
                4 * 1024 * 1024,
            )),
            xhr: Box::new(XHR),
            user_agent: "editing-fixture".into(),
        }),
        None,
    );
    page.Open("https://page.test/page.html", 16384, 4096)
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
fn edit_id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let doc = owner.GetDocument();
    fn find(doc: &dom::Document, i: usize, name: &str) -> Option<u64> {
        let n = doc.Node(i);
        if n.FindAttribute("id").is_some_and(|a| a.value == name) {
            return Some(n.Id());
        }
        n.Children().iter().find_map(|&i| find(doc, i, name))
    }
    find(doc, doc.Root(), name).unwrap()
}
fn edit_focus(page: &mut Page, id: u64) {
    page.Dispatch(&interaction::input_event::InputEvent::Focus(
        interaction::input_event::FocusEvent {
            target_node_id: id,
            ..Default::default()
        },
    ))
    .unwrap();
}
fn edit_key(page: &mut Page, key: &str, shift: bool, control: bool) {
    use interaction::input_event::*;
    page.Dispatch(&InputEvent::Key(KeyEvent {
        key: key.into(),
        modifiers: EventModifiers {
            shift,
            control,
            ..Default::default()
        },
        ..Default::default()
    }))
    .unwrap();
}
fn edit_value(page: &Page, id: u64) -> String {
    let owner = page.Document();
    let doc = owner.GetDocument();
    doc.ControlValue(doc.FindNodeById(id).unwrap())
}
fn edit_selection(page: &Page, id: u64) -> layoutng_assembly::editing_state::Selection {
    page.GetLayoutEngine()
        .GetLayoutTree()
        .unwrap()
        .EditingState()
        .selections
        .Get(id)
        .unwrap()
}
fn edit_insert(page: &mut Page, text: &str) {
    use interaction::input_event::*;
    page.Dispatch(&InputEvent::TextInput(TextInputEvent {
        text: text.into(),
        ..Default::default()
    }))
    .unwrap();
}
fn edit_mouse(
    page: &mut Page,
    kind: interaction::input_event::MouseEventType,
    point: layoutng_assembly::internal::layout_input::Offset,
    shift: bool,
) {
    use interaction::input_event::*;
    page.Dispatch(&InputEvent::Mouse(MouseEvent {
        r#type: kind,
        position: point,
        button: MouseButton::kPrimary,
        modifiers: EventModifiers {
            shift,
            ..Default::default()
        },
        ..Default::default()
    }))
    .unwrap();
}

#[test]
fn text_control_empty_and_placeholder_caret_match_populated_baseline() {
    native_test_thread::run(|| {
        let mut page = editing_fixture();
        for name in ["edit", "hint", "area"] {
            let id = edit_id(&page, name);
            edit_focus(&mut page, id);
            let empty = page.Caret().unwrap().rect;
            edit_insert(&mut page, "W");
            edit_key(&mut page, "Home", false, false);
            let populated = page.Caret().unwrap().rect;
            assert_eq!(
                empty, populated,
                "{name}: empty and populated caret must share editor baseline"
            );
        }
    });
}

#[test]
fn text_control_hit_test_round_trips_shaped_unicode_positions_and_transforms() {
    native_test_thread::run(|| {
        use interaction::input_event::MouseEventType as M;
        use layoutng_assembly::internal::layout_input::Offset;
        let mut page = editing_fixture();
        let id = edit_id(&page, "edit");
        edit_focus(&mut page, id);
        let text = "Wi你🙂fi";
        edit_insert(&mut page, text);
        for transformed in [false, true] {
            if transformed {
                assert!(page.Evaluate("document.getElementById('edit').style.transform='translate(40px,20px) scale(1.25)'", "test:transform").unwrap().Succeeded());
            }
            let mut positions = Vec::new();
            edit_key(&mut page, "Home", false, false);
            for byte in text
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(text.len()))
            {
                let c = page.Caret().unwrap().rect;
                positions.push((
                    byte,
                    Offset {
                        x: c.x,
                        y: c.y + c.height * 0.5,
                    },
                ));
                edit_key(&mut page, "ArrowRight", false, false);
            }
            for (expected, point) in positions {
                edit_mouse(&mut page, M::kDown, point, false);
                edit_mouse(&mut page, M::kUp, point, false);
                assert_eq!(
                    edit_selection(&page, id).focus,
                    expected,
                    "click at {point:?}, transformed={transformed}"
                );
            }
        }
    });
}

#[test]
fn text_control_preedit_replaces_previous_range_and_preserves_native_selection() {
    native_test_thread::run(|| {
        use interaction::input_event::*;
        let mut page = editing_fixture();
        let id = edit_id(&page, "edit");
        edit_focus(&mut page, id);
        edit_insert(&mut page, "AB");
        edit_key(&mut page, "ArrowLeft", false, false);
        let send = |page: &mut Page, kind, data: &str, selection| {
            page.Dispatch(&InputEvent::Composition(CompositionEvent {
                r#type: kind,
                data: data.into(),
                selection,
                ..Default::default()
            }))
            .unwrap();
        };
        send(&mut page, CompositionEventType::kStart, "", None);
        send(&mut page, CompositionEventType::kUpdate, "ni", Some((1, 1)));
        assert_eq!(edit_value(&page, id), "AniB");
        assert_eq!(edit_selection(&page, id).focus, 2);
        let first = page.Caret().unwrap().rect;
        send(
            &mut page,
            CompositionEventType::kUpdate,
            "你好",
            Some((3, 3)),
        );
        assert_eq!(edit_value(&page, id), "A你好B");
        assert_eq!(edit_selection(&page, id).focus, 4);
        assert_ne!(page.Caret().unwrap().rect.x, first.x);
        send(&mut page, CompositionEventType::kUpdate, "", None);
        assert_eq!(edit_value(&page, id), "AB");
        send(&mut page, CompositionEventType::kEnd, "你", None);
        assert_eq!(edit_value(&page, id), "A你B");
        assert_eq!(edit_selection(&page, id).focus, 4);
        edit_insert(&mut page, "!");
        assert_eq!(edit_value(&page, id), "A你!B");
    });
}

#[test]
fn text_control_selection_paints_blue_and_tracks_drag_shift_and_collapse() {
    native_test_thread::run(|| {
        use interaction::input_event::MouseEventType as M;
        use layoutng_assembly::internal::layout_input::Offset;
        let mut page = editing_fixture();
        let id = edit_id(&page, "edit");
        edit_focus(&mut page, id);
        edit_insert(&mut page, "Wide text");
        let end = page.Caret().unwrap().rect;
        edit_key(&mut page, "Home", false, false);
        let start = page.Caret().unwrap().rect;
        let a = Offset {
            x: start.x,
            y: start.y + start.height * 0.5,
        };
        let b = Offset { x: end.x, y: a.y };
        edit_mouse(&mut page, M::kDown, a, false);
        edit_mouse(&mut page, M::kMove, b, false);
        edit_mouse(&mut page, M::kUp, b, false);
        assert_eq!(edit_selection(&page, id).Start(), 0);
        assert_eq!(edit_selection(&page, id).End(), 9);
        let blue_pixels = |page: &Page| {
            let mut pixels = vec![0; 640 * 480];
            raster::surface::RenderDisplayItemListIntoWindowBuffer(
                &page.CurrentFrame().unwrap().display_items,
                640,
                480,
                1.0,
                &mut pixels,
            )
            .unwrap();
            pixels.iter().filter(|p| **p & 0xffffff == 0x0066ff).count()
        };
        assert!(page.Caret().is_none());
        assert!(
            blue_pixels(&page) > 100,
            "selection must reach actual raster output"
        );
        let selected_pixels = blue_pixels(&page);
        std::thread::sleep(std::time::Duration::from_millis(550));
        page.RunTask().unwrap();
        assert_eq!(
            blue_pixels(&page),
            selected_pixels,
            "caret blinking must retain selection highlighting"
        );
        edit_key(&mut page, "ArrowLeft", false, false);
        assert_eq!(edit_selection(&page, id).focus, 0);
        assert_eq!(blue_pixels(&page), 0);
        assert!(page.Caret().is_some());
        edit_key(&mut page, "ArrowRight", true, false);
        assert!(blue_pixels(&page) > 20);
        edit_key(&mut page, "a", false, true);
        assert!(blue_pixels(&page) > 100);
        edit_insert(&mut page, "你");
        assert_eq!(edit_value(&page, id), "你");
        assert_eq!(blue_pixels(&page), 0);
    });
}

#[test]
fn text_control_multiline_hit_test_and_trailing_newline_caret() {
    native_test_thread::run(|| {
        use interaction::input_event::MouseEventType as M;
        use layoutng_assembly::internal::layout_input::Offset;
        let mut page = editing_fixture();
        let id = edit_id(&page, "area");
        assert!(page
            .Evaluate(
                "document.getElementById('area').style.height='220px'",
                "test:textarea"
            )
            .unwrap()
            .Succeeded());
        edit_focus(&mut page, id);
        let first = page.Caret().unwrap().rect;
        edit_insert(&mut page, "AB\n你D");
        let second = page.Caret().unwrap().rect;
        assert!(second.y > first.y);
        let point = Offset {
            x: second.x,
            y: second.y + second.height * 0.5,
        };
        edit_mouse(&mut page, M::kDown, point, false);
        edit_mouse(&mut page, M::kUp, point, false);
        assert_eq!(edit_selection(&page, id).focus, "AB\n你D".len());
        edit_insert(&mut page, "\n");
        let trailing = page.Caret().unwrap().rect;
        assert!(
            trailing.y > second.y,
            "trailing newline must place caret on the next line: {trailing:?} vs {second:?}"
        );
        assert_eq!(trailing.x, first.x);
    });
}

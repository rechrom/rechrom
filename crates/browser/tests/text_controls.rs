#[path = "../src/test_thread.rs"]
mod test_thread;
use browser::page::Page;
use dom::persistent_document::DOMNodeType;
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
            :root { --base-color-scale-gray-7:#59636e;
                --brand-color-text-muted:var(--base-color-scale-gray-7); }
            input::placeholder { color:var(--brand-color-text-muted); opacity:1; }
            #toolbar-flex { display:flex; align-items:center; width:640px; height:46px; padding:6px; }
            #toolbar-input { flex:1; min-width:8px; width:100%; height:30px; margin:0; padding:0 10px 0 0;
                border:0; background:transparent; color:#202124; font:14px sans-serif; outline:none; }
            #hint { display:flex; }
            #parent { color:var(--nested-color,#008c2f); font:32px/40px sans-serif; }
            </style><input id="edit"><input id="hint" placeholder="placeholder"><textarea id="area"></textarea><div id="toolbar-flex"><input id="toolbar-input" value="about:home"></div><div id="parent">First line<br>Second line</div>"#.to_vec(),..Default::default()}))))
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
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader)),
        Rc::new(RefCell::new(SkiaImageDecoder)),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        space,
        Some(browser::page::ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_max_stack_size(
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
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
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
fn placeholder_pseudo_style_reaches_the_raster_output() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = editing_fixture();
        let placeholder_text_id = edit_id(&page, "hint") | (1 << 53);
        let item = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .find(|item| {
                item.node_id == placeholder_text_id
                    && item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
                    && !item.stroke_glyphs
            })
            .expect("the visible placeholder must produce a foreground glyph run");
        assert!(
            (item.color.red - 0x59 as f32 / 255.0).abs() < 0.01
                && (item.color.green - 0x63 as f32 / 255.0).abs() < 0.01
                && (item.color.blue - 0x6e as f32 / 255.0).abs() < 0.01
                && item.color.alpha > 0.99,
            "::placeholder custom-property color must reach Paint: {:?}",
            item.color
        );

        let pixels = raster::RasterizePaintArtifactWithScale(
            &page.CurrentFrame().unwrap().display_items,
            640,
            480,
            1.0,
        );
        let rect = item.glyph_ink_rect.unwrap_or(item.rect);
        let left = rect.x.floor().max(0.0) as usize;
        let top = rect.y.floor().max(0.0) as usize;
        let right = (rect.x + rect.width).ceil().min(640.0) as usize;
        let bottom = (rect.y + rect.height).ceil().min(480.0) as usize;
        let colored = (top..bottom)
            .flat_map(|y| (left..right).map(move |x| (y * 640 + x) * 4))
            .filter(|&offset| {
                let pixel = &pixels[offset..offset + 4];
                pixel[0] < 230 || pixel[1] < 230 || pixel[2] < 230
            })
            .count();
        assert!(
            colored > 20,
            "the placeholder glyph run must survive rasterization; colored pixels={colored}, rect={rect:?}"
        );
    });
}

#[test]
fn flexed_toolbar_text_control_value_reaches_raster() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = editing_fixture();
        let id = edit_id(&page, "toolbar-input");
        let value_text_id = id | (1 << 53);
        let frame = page.CurrentFrame().unwrap();
        let item = frame
            .display_items
            .items
            .iter()
            .find(|item| {
                item.node_id == value_text_id
                    && item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
                    && !item.stroke_glyphs
            })
            .expect("the toolbar control value must produce a foreground glyph run");
        let pixels = raster::RasterizePaintArtifactWithScale(&frame.display_items, 640, 480, 1.0);
        let rect = item.glyph_ink_rect.unwrap_or(item.rect);
        let left = rect.x.floor().max(0.0) as usize;
        let top = rect.y.floor().max(0.0) as usize;
        let right = (rect.x + rect.width).ceil().min(640.0) as usize;
        let bottom = (rect.y + rect.height).ceil().min(480.0) as usize;
        let dark = (top..bottom)
            .flat_map(|y| (left..right).map(move |x| (y * 640 + x) * 4))
            .filter(|&offset| {
                pixels[offset] < 160 && pixels[offset + 1] < 160 && pixels[offset + 2] < 160
            })
            .count();
        assert!(
            dark > 10,
            "toolbar value glyphs must reach raster; dark={dark}, rect={rect:?}"
        );
    });
}

#[test]
fn paint_only_placeholder_color_change_rejects_the_cached_glyph_run() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = editing_fixture();
        let placeholder_text_id = edit_id(&page, "hint") | (1 << 53);
        let color = |page: &Page| {
            page.CurrentFrame()
                .unwrap()
                .display_items
                .items
                .iter()
                .find(|item| {
                    item.node_id == placeholder_text_id
                        && item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
                        && !item.is_shadow
                        && !item.stroke_glyphs
                })
                .expect("the visible placeholder must produce a foreground glyph run")
                .color
        };
        let before = color(&page);
        assert!(page
            .Evaluate(
                "document.documentElement.style.setProperty('--base-color-scale-gray-7','#ff0000')",
                "test:placeholder-paint-color",
            )
            .unwrap()
            .Succeeded());
        page.RunTask().unwrap();
        let after = color(&page);
        assert_ne!(after, before, "the old cached glyph run must be rejected");
        assert!(
            after.red > 0.99 && after.green < 0.01 && after.blue < 0.01 && after.alpha > 0.99,
            "the repainted placeholder must use the current custom-property color: {after:?}"
        );
        let red_pixels = |page: &Page| {
            raster::RasterizePaintArtifactWithScale(
                &page.CurrentFrame().unwrap().display_items,
                640,
                480,
                1.0,
            )
            .chunks_exact(4)
            .filter(|pixel| pixel[0] > 200 && pixel[1] < 80 && pixel[2] < 80)
            .count()
        };
        assert!(
            red_pixels(&page) > 20,
            "the current glyph color must reach raster output"
        );

        assert!(page
            .Evaluate(
                "document.getElementById('hint').style.opacity='0'",
                "test:placeholder-paint-opacity",
            )
            .unwrap()
            .Succeeded());
        page.RunTask().unwrap();
        assert_eq!(
            red_pixels(&page),
            0,
            "the current opacity must suppress the retained glyph at raster"
        );
    });
}

#[test]
fn inherited_paint_only_color_change_repaints_existing_text_child() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = editing_fixture();
        let parent = edit_id(&page, "parent");
        let texts = {
            let owner = page.Document();
            let document = owner.GetDocument();
            let node = document.FindNodeById(parent).unwrap();
            document
                .Node(node)
                .Children()
                .iter()
                .filter_map(|&child| {
                    (document.Node(child).Type() == DOMNodeType::kText)
                        .then(|| document.Node(child).Id())
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(texts.len(), 2);
        let color = |page: &Page| {
            texts
                .iter()
                .map(|text| {
                    page.CurrentFrame()
                        .unwrap()
                        .display_items
                        .items
                        .iter()
                        .find(|item| {
                            item.node_id == *text
                                && item.r#type
                                    == paint::paint_engine::DisplayItemType::kDrawGlyphRun
                                && !item.is_shadow
                                && !item.stroke_glyphs
                        })
                        .expect("nested text must produce a foreground glyph run")
                        .color
                })
                .collect::<Vec<_>>()
        };
        let before = color(&page);
        assert!(before.iter().all(|color| color.green > color.red));
        assert!(page
            .Evaluate(
                "if(getComputedStyle(document.getElementById('parent')).color!=='rgb(0, 140, 47)')throw Error('inherited color CSSOM')",
                "test:inherited-color-cssom-before",
            )
            .unwrap()
            .Succeeded());
        page.Apply(page_mutation::PageMutation::CSSOMMutation(
            page_mutation::CSSOMMutation {
                style_sheet: style::ParseCSS(":root { --nested-color:#000000 }"),
                base_url: "https://page.test/late.css".into(),
            },
        ))
        .unwrap();
        let after = color(&page);
        assert!(page
            .Evaluate(
                "if(getComputedStyle(document.getElementById('parent')).color!=='rgb(0, 0, 0)')throw Error('updated inherited color CSSOM')",
                "test:inherited-color-cssom-after",
            )
            .unwrap()
            .Succeeded());
        assert!(
            after
                .iter()
                .all(|color| color.red < 0.01 && color.green < 0.01 && color.blue < 0.01),
            "the inherited current color must replace the retained child glyph: {after:?}"
        );
    });
}

#[test]
fn hydration_replacement_retires_old_text_fragments_before_late_style() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = editing_fixture();
        let parent = edit_id(&page, "parent");
        let old_text = {
            let owner = page.Document();
            let document = owner.GetDocument();
            let node = document.FindNodeById(parent).unwrap();
            document
                .Node(node)
                .Children()
                .iter()
                .filter_map(|&child| {
                    (document.Node(child).Type() == DOMNodeType::kText)
                        .then(|| document.Node(child).Id())
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(old_text.len(), 2);
        assert!(page
            .Evaluate(
                r#"var p=document.getElementById('parent');
                   while(p.firstChild)p.removeChild(p.firstChild);
                   for(const word of ['Software ','still ','starts']){
                     const span=document.createElement('span');span.textContent=word;p.appendChild(span);
                   }
                   p.appendChild(document.createElement('br'));
                   for(const word of [' with ','you']){
                     const span=document.createElement('span');span.textContent=word;p.appendChild(span);
                   }
                   const link=document.createElement('link');link.id='late-style-owner';
                   link.rel='stylesheet';document.head.appendChild(link);"#,
                "test:hydrate-text",
            )
            .unwrap()
            .Succeeded());
        page.RunTask().unwrap();
        let mut late = style::ParseCSS("#parent { color:#000000 }");
        late.SetLinkMetadata(
            edit_id(&page, "late-style-owner"),
            false,
            false,
            String::new(),
            String::new(),
        );
        page.Apply(page_mutation::PageMutation::CSSOMMutation(
            page_mutation::CSSOMMutation {
                style_sheet: late,
                base_url: "https://page.test/hydrated.css".into(),
            },
        ))
        .unwrap();
        let resolved_colors = {
            let owner = page.Document();
            let document = owner.GetDocument();
            let parent_index = document.FindNodeById(parent).unwrap();
            std::iter::once(parent_index)
                .chain(document.Node(parent_index).Children().iter().copied())
                .filter_map(|index| {
                    document
                        .ResolvedStyleFor(index)
                        .and_then(|resolved| unsafe {
                            resolved.computed_style_owner.Get().as_ref().map(|style| {
                                (document.Node(index).Id(), style.GetCurrentColor(None))
                            })
                        })
                })
                .collect::<Vec<_>>()
        };
        let glyphs = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .items
            .iter()
            .filter(|item| {
                item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
                    && !item.stroke_glyphs
            })
            .collect::<Vec<_>>();
        assert!(
            old_text
                .iter()
                .all(|old| glyphs.iter().all(|item| item.node_id != *old)),
            "hydration must retire every display item owned by removed LayoutText"
        );
        let hydrated = glyphs
            .iter()
            .filter(|item| item.rect.y > 200.0)
            .collect::<Vec<_>>();
        assert!(
            hydrated.len() >= 5,
            "each hydrated word must paint; glyphs={}",
            hydrated.len()
        );
        assert!(
            hydrated.iter().all(|item| {
                item.color.red < 0.01 && item.color.green < 0.01 && item.color.blue < 0.01
            }),
            "late style must reach every replacement LayoutText: {:?}",
            (
                resolved_colors,
                hydrated
                    .iter()
                    .map(|item| (item.node_id, item.color))
                    .collect::<Vec<_>>()
            )
        );
    });
}

#[test]
fn text_control_hit_test_round_trips_shaped_unicode_positions_and_transforms() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
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
fn text_control_preedit_replaces_previous_range_and_preserves_source_selection() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
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
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
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
            let Some(selection_color) = page
                .CurrentFrame()
                .unwrap()
                .display_items
                .items
                .iter()
                .find(|item| {
                    item.r#type == paint::paint_engine::DisplayItemType::kDrawRect
                        && item.color.alpha > 0.0
                        && item.color.alpha < 1.0
                        && item.color.blue > item.color.green
                        && item.color.green > item.color.red
                })
                .map(|item| item.color)
            else {
                return 0;
            };
            let composite_over_white = |channel: f32| {
                ((channel * selection_color.alpha + 1.0 - selection_color.alpha) * 255.0).round()
                    as u8
            };
            let expected = [
                composite_over_white(selection_color.red),
                composite_over_white(selection_color.green),
                composite_over_white(selection_color.blue),
            ];
            let pixels = raster::RasterizePaintArtifactWithScale(
                &page.CurrentFrame().unwrap().display_items,
                640,
                480,
                1.0,
            );
            pixels
                .chunks_exact(4)
                .filter(|pixel| pixel[..3] == expected)
                .count()
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
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
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

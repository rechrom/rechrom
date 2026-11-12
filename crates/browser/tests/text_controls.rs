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
    let editing_behavior = if cfg!(target_vendor = "apple") {
        interaction::default::core::editing::editing_behavior::EditingBehaviorType::Mac
    } else if cfg!(target_os = "windows") {
        interaction::default::core::editing::editing_behavior::EditingBehaviorType::Windows
    } else if cfg!(target_os = "android") {
        interaction::default::core::editing::editing_behavior::EditingBehaviorType::Android
    } else {
        interaction::default::core::editing::editing_behavior::EditingBehaviorType::Unix
    };
    page.SetInteractionTextSettings(editing_behavior, c"en-US", cfg!(target_os = "macos"));
    page.SetSystemCursorSize(0.0, 0.0).unwrap();
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
fn edit_index(page: &Page, name: &str) -> usize {
    let owner = page.Document();
    let doc = owner.GetDocument();
    fn find(doc: &dom::Document, i: usize, name: &str) -> Option<usize> {
        let n = doc.Node(i);
        if n.FindAttribute("id").is_some_and(|a| a.value == name) {
            return Some(i);
        }
        n.Children().iter().find_map(|&i| find(doc, i, name))
    }
    find(doc, doc.Root(), name).unwrap()
}
fn edit_id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    owner.GetDocument().Node(edit_index(page, name)).Id()
}
fn text_control_placeholder_text_id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    let host = edit_index(page, name);
    let placeholder = document
        .TextControlPlaceholderElement(host)
        .expect("text control placeholder owner");
    document
        .Node(placeholder)
        .Children()
        .iter()
        .copied()
        .find(|&child| document.Node(child).Type() == DOMNodeType::kText)
        .map(|child| document.Node(child).Id())
        .expect("placeholder text owner")
}
fn text_control_value_text_id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    let host = edit_index(page, name);
    let editor = document
        .TextControlInnerEditorElement(host)
        .expect("text control editor owner");
    document
        .Node(editor)
        .Children()
        .iter()
        .copied()
        .find(|&child| document.Node(child).Type() == DOMNodeType::kText)
        .map(|child| document.Node(child).Id())
        .expect("text control value text owner")
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
    let edit_commands = if cfg!(target_os = "macos") && control && key.eq_ignore_ascii_case("a") {
        vec![interaction::default::core::editing::commands::editor_command_table::EditingCommandType::SelectAll]
    } else {
        Vec::new()
    };
    page.Dispatch(&InputEvent::Key(KeyEvent {
        key: key.into(),
        code: DomCodeForKey(key),
        key_code: LegacyKeyCode(key),
        // RenderWidgetHostViewCocoa supplies Command-key editing selectors on
        // this side channel; Blink deliberately omits them from its macOS key
        // table so the native menu command remains the single source owner.
        edit_commands,
        modifiers: EventModifiers {
            shift,
            control: control && !cfg!(target_os = "macos"),
            meta: control && cfg!(target_os = "macos"),
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

#[test]
fn first_active_publication_enables_the_focused_caret() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = editing_fixture();
        page.SetActive(true).unwrap();
        let id = edit_id(&page, "edit");
        edit_focus(&mut page, id);
        assert!(
            page.Caret().is_some_and(|caret| caret.visible),
            "the first host active=true must reach FrameSelection"
        );
    });
}

#[test]
fn mouse_refocus_reactivates_the_text_control_selection_owner() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        use interaction::input_event::MouseEventType as M;
        use layoutng_assembly::internal::layout_input::Offset;

        let mut page = editing_fixture();
        assert!(
            page.Evaluate(
                "var b=document.createElement('button');b.id='focus-button';b.textContent='button';b.style.cssText='position:absolute;left:400px;top:12px;width:120px;height:48px';document.body.appendChild(b)",
                "test:focus-button",
            )
            .unwrap()
            .Succeeded()
        );
        page.SetActive(true).unwrap();
        let edit = edit_id(&page, "edit");
        let button = edit_id(&page, "focus-button");

        edit_focus(&mut page, edit);
        edit_insert(&mut page, "first");
        for point in [Offset { x: 430.0, y: 30.0 }, Offset { x: 300.0, y: 40.0 }] {
            edit_mouse(&mut page, M::kDown, point, false);
            edit_mouse(&mut page, M::kUp, point, false);
            assert_eq!(
                page.FocusedNodeId(),
                Some(if point.x > 400.0 { button } else { edit })
            );
        }

        edit_insert(&mut page, "-second");
        assert_eq!(edit_value(&page, edit), "first-second");
        assert_eq!(edit_selection(&page, edit).focus, "first-second".len());
        assert!(page.Caret().is_some_and(|caret| caret.visible));
    });
}

#[test]
fn document_editor_insert_link_and_format_commands_use_native_undo() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = editing_fixture();
        let setup = r#"
            var old=document.getElementById('command-editor'); if(old) old.remove();
            var c=document.createElement('div'); c.id='command-editor';
            c.setAttribute('contenteditable','true'); c.innerHTML='hello';
            document.body.appendChild(c); c.focus();
            function selectContents(n, collapsed) {
                var r=document.createRange(); r.selectNodeContents(n);
                if(collapsed) r.collapse(false);
                var s=window.getSelection();s.removeAllRanges();s.addRange(r);
            }
            function check(condition, message) { if(!condition) throw new Error(message); }
            selectContents(c, false);
        "#;
        for (name, script) in [
            (
                "html",
                r#"check(document.execCommand('insertHTML',false,'<b id="inserted">world</b>'),'insertHTML accepted');check(c.textContent==='world'&&!!c.querySelector('b'),'native HTML replacement');check(document.execCommand('undo'),'undo accepted');check(c.textContent==='hello','HTML undo');"#,
            ),
            (
                "image",
                r#"check(document.execCommand('insertImage',false,'literal"quote'),'insertImage accepted');var image=c.querySelector('img');check(image&&image.getAttribute('src')==='literal"quote','literal image attribute');"#,
            ),
            (
                "horizontal rule",
                r#"check(document.execCommand('insertHorizontalRule',false,'rule"id'),'insertHorizontalRule accepted');var rule=c.querySelector('hr');check(rule&&rule.id==='rule"id','literal rule id');"#,
            ),
            (
                "range link",
                r#"check(document.execCommand('createLink',false,'https://link.test/'),'createLink accepted');var link=c.querySelector('a');check(link&&link.textContent==='hello'&&link.getAttribute('href')==='https://link.test/','range link');check(document.execCommand('undo'),'link undo accepted');check(!c.querySelector('a')&&c.textContent==='hello','link undo');"#,
            ),
            (
                "caret link",
                r#"selectContents(c,true);check(document.execCommand('createLink',false,'https://link.test/'),'caret createLink accepted');var link=c.querySelector('a');check(link&&link.textContent==='https://link.test/','caret URL label');check(window.getSelection().toString()==='https://link.test/','caret link selection');"#,
            ),
            (
                "range unlink",
                r#"c.innerHTML='<a href="/test">hello</a>';selectContents(c.firstChild,false);check(document.execCommand('unlink'),'unlink accepted');check(!c.querySelector('a')&&c.textContent==='hello','unlink keeps text');"#,
            ),
            (
                "caret unlink",
                r#"c.innerHTML='<a href="/test">hello</a>';selectContents(c.firstChild,true);check(!document.execCommand('unlink'),'caret unlink disabled');check(!!c.querySelector('a'),'source caret unlink leaves link');"#,
            ),
            (
                "remove format",
                r#"c.innerHTML='<p><a href="/test"><b>hello</b></a></p>';selectContents(c.querySelector('b'),false);check(document.execCommand('removeFormat'),'removeFormat accepted');check(!c.querySelector('b')&&!!c.querySelector('a')&&!!c.querySelector('p')&&c.textContent==='hello','removeFormat preserves link and block');"#,
            ),
            (
                "format block",
                r#"c.innerHTML='<p>hello</p>';selectContents(c.firstChild,false);check(document.execCommand('formatBlock',false,'<H2>'),'formatBlock accepted');check(!!c.querySelector('h2')&&c.textContent==='hello','formatted heading');check(document.queryCommandValue('formatBlock')==='h2','formatBlock query');check(document.execCommand('undo'),'block undo accepted');check(!!c.querySelector('p')&&!c.querySelector('h2'),'block undo');"#,
            ),
            (
                "format paragraphs",
                r#"c.innerHTML='<p>hello</p><p>world</p>';selectContents(c,false);check(document.execCommand('formatBlock',false,'pre'),'multi paragraph formatBlock');check(!!c.querySelector('pre')&&c.textContent.indexOf('hello')>=0&&c.textContent.indexOf('world')>=0,'formatted paragraph contents');"#,
            ),
            (
                "unsupported block",
                r#"check(!document.execCommand('formatBlock',false,'script'),'unsupported block rejected');check(c.textContent==='hello','unsupported block leaves selection');"#,
            ),
            (
                "input HTML",
                r#"var input=document.getElementById('edit');input.value='old';input.focus();input.setSelectionRange(0,3);check(document.execCommand('insertHTML',false,'<b>hello</b><br>world'),'input insertHTML');check(input.value==='hello world','single line input replaces inserted LF with space: '+JSON.stringify(input.value));"#,
            ),
        ] {
            let source = format!("{setup}\n{script}");
            let result = page.Evaluate(&source, "test:editor-commands").unwrap();
            assert!(
                result.Succeeded(),
                "{name}: {}",
                result
                    .exception
                    .as_ref()
                    .map(|e| e.message.as_str())
                    .unwrap_or("")
            );
        }
    });
}
fn edit_mouse(
    page: &mut Page,
    kind: interaction::input_event::MouseEventType,
    point: layoutng_assembly::internal::layout_input::Offset,
    shift: bool,
) {
    use interaction::input_event::*;
    let dragging = kind == MouseEventType::kMove;
    page.Dispatch(&InputEvent::Mouse(MouseEvent {
        r#type: kind,
        position: Offset {
            x: point.x,
            y: point.y,
        },
        // Match native/CDP ingress: a move has no changed button, while the
        // depressed-buttons mask identifies it as a left-button drag.
        button: if dragging {
            MouseButton::kNone
        } else {
            MouseButton::kPrimary
        },
        buttons: if dragging {
            MouseButton::kPrimary.ButtonsMask()
        } else {
            0
        },
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
        let placeholder_text_id = text_control_placeholder_text_id(&page, "hint");
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
        let value_text_id = text_control_value_text_id(&page, "toolbar-input");
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
        let placeholder_text_id = text_control_placeholder_text_id(&page, "hint");
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
            for utf16_offset in text
                .char_indices()
                .map(|(byte, _)| text[..byte].encode_utf16().count())
                .chain(std::iter::once(text.encode_utf16().count()))
            {
                let c = page.Caret().unwrap().rect;
                positions.push((
                    utf16_offset,
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
        assert!(page
            .Evaluate(
                "window.__compositionData=[];document.getElementById('edit').addEventListener('compositionupdate',event=>__compositionData.push(['update',event.data]));document.getElementById('edit').addEventListener('compositionend',event=>__compositionData.push(['end',event.data]))",
                "test:composition-data",
            )
            .unwrap()
            .Succeeded());
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
        // CommitText(relative_caret = 0) collapses immediately after the
        // committed text; the following insertion must therefore precede B.
        assert_eq!(edit_selection(&page, id).focus, 2);
        edit_insert(&mut page, "!");
        assert_eq!(edit_value(&page, id), "A你!B");
        send(
            &mut page,
            CompositionEventType::kUpdate,
            "hao",
            Some((3, 3)),
        );
        send(&mut page, CompositionEventType::kEnd, "好", None);
        assert_eq!(edit_value(&page, id), "A你!好B");
        let composition_data = page
            .Evaluate(
                "if(JSON.stringify(__compositionData)!==JSON.stringify([['update','ni'],['update','你好'],['update',''],['end',''],['update','hao'],['update','好'],['end','好']]))throw new Error(JSON.stringify(__compositionData))",
                "test:composition-data",
            )
            .unwrap();
        assert!(
            composition_data.Succeeded(),
            "{}",
            composition_data
                .exception
                .as_ref()
                .map(|exception| exception.message.as_str())
                .unwrap_or("")
        );
    });
}

#[test]
fn text_control_selection_paints_blue_and_tracks_drag_shift_and_collapse() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        use interaction::input_event::MouseEventType as M;
        use layoutng_assembly::internal::layout_input::Offset;
        let mut page = editing_fixture();
        page.SetActive(true).unwrap();
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
        // A real drag spans rendering opportunities. Blink recomputes hover
        // from the retained MouseEventManager during BeginMainFrame; that
        // synthetic move must preserve the pressed button and the drag owner.
        let frame_time = std::time::Instant::now();
        page.UpdateRendering(foundation::begin_frame::BeginFrameArgs {
            source_id: 1,
            sequence_number: 1,
            frame_time,
            deadline: frame_time + std::time::Duration::from_millis(16),
            interval: std::time::Duration::from_millis(16),
        })
        .unwrap();
        page.QueueFrameInput(
            interaction::input_event::InputEvent::Mouse(interaction::input_event::MouseEvent {
                r#type: M::kMove,
                position: interaction::input_event::Offset { x: b.x, y: b.y },
                button: interaction::input_event::MouseButton::kNone,
                buttons: interaction::input_event::MouseButton::kPrimary.ButtonsMask(),
                ..Default::default()
            }),
            std::time::Instant::now(),
            1,
        )
        .unwrap();
        edit_mouse(&mut page, M::kUp, b, false);
        assert_eq!(edit_selection(&page, id).Start(), 0);
        assert_eq!(edit_selection(&page, id).End(), 9);
        let blue_pixels = |page: &Page| {
            let Some((selection_color, selection_rect)) = page
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
                .map(|item| (item.color, item.rect))
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
            let exact = pixels
                .chunks_exact(4)
                .filter(|pixel| pixel[..3] == expected)
                .count();
            let _ = selection_rect;
            exact
        };
        assert!(page.Caret().is_none());
        let selected_pixels = blue_pixels(&page);
        assert!(
            selected_pixels > 100,
            "selection must reach actual raster output: {selected_pixels} exact pixels"
        );
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
        assert_eq!(edit_selection(&page, id).Start(), 0);
        assert_eq!(edit_selection(&page, id).End(), 9);
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
        assert!(
            second.y > first.y,
            "multiline caret must advance: first={first:?}, second={second:?}, value={:?}, selection={:?}",
            edit_value(&page, id),
            edit_selection(&page, id),
        );
        let point = Offset {
            x: second.x,
            y: second.y + second.height * 0.5,
        };
        edit_mouse(&mut page, M::kDown, point, false);
        edit_mouse(&mut page, M::kUp, point, false);
        assert_eq!(
            edit_selection(&page, id).focus,
            "AB\n你D".encode_utf16().count()
        );
        edit_insert(&mut page, "\n");
        let trailing = page.Caret().unwrap().rect;
        assert!(
            trailing.y > second.y,
            "trailing newline must place caret on the next line: {trailing:?} vs {second:?}"
        );
        assert_eq!(trailing.x, first.x);
    });
}

//! UA button alignment must survive the canonical cascade and subsequent
//! author/style mutations. Check both ComputedStyle and published geometry.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use foundation::ETextAlign;
use layoutng_assembly::{
    fragment_tree::FragmentPaintData,
    internal::layout_input::{FontFace, NodeKind, Offset},
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

fn fixture(author_color: bool) -> Page {
    let css_color = if author_color { "color: black;" } else { "" };
    let html = format!(
        r#"<!doctype html><style>
        body {{ margin: 0; }}
        input,button {{ display: block; width: 108px; height: 44px;
            padding: 0; border: 0; font: 17px/44px ButtonFixture; {css_color} }}
        .row {{ display: flex; align-items: center; margin-left: 30px; }}
        </style>
        <input id=submit type=submit value=Search>
        <input id=reset type=reset value=Search>
        <input id=input-button type=button value=Search>
        <button id=button>Search</button>
        <button id=left style='text-align:left'>Search</button>
        <button id=right style='text-align:right'>Search</button>
        <div class=row><button id=flex style='position:relative'> Search </button></div>"#
    );
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

fn find_fragment<'a>(
    node: &'a FragmentPaintData,
    parent: Offset,
    predicate: &impl Fn(&FragmentPaintData) -> bool,
) -> Option<(&'a FragmentPaintData, Offset)> {
    let absolute = Offset {
        x: parent.x + node.offset.x,
        y: parent.y + node.offset.y,
    };
    if predicate(node) {
        return Some((node, absolute));
    }
    node.children
        .iter()
        .find_map(|child| find_fragment(child, absolute, predicate))
}

fn assert_label_alignment(page: &Page, name: &str, align: ETextAlign) {
    let owner = page.Document();
    let document = owner.GetDocument();
    let index = (0..document.NodeCount())
        .find(|&index| {
            document
                .Node(index)
                .FindAttribute("id")
                .is_some_and(|attribute| attribute.value == name)
        })
        .unwrap();
    let style = document.ResolvedStyleFor(index).unwrap();
    assert_eq!(
        unsafe { &*style.native_style.Get() }.GetTextAlign(),
        align,
        "{name}: native alignment"
    );
    let id = document.Node(index).Id();
    let frame = page.CurrentFrame().unwrap();
    let (control, origin) = find_fragment(&frame.fragments, Offset::default(), &|fragment| {
        fragment.node_id == id
    })
    .unwrap();
    let (label, label_origin) = find_fragment(control, Offset::default(), &|fragment| {
        fragment.paint.source_kind == NodeKind::kText
    })
    .unwrap();
    // label_origin includes the control's offset, so remove that offset to
    // compare positions in the control's own content box. Padding/border=0.
    let actual = label_origin.x - control.offset.x;
    let available = control.size.width - label.size.width;
    let expected = match align {
        ETextAlign::kCenter => available / 2.0,
        ETextAlign::kRight => available,
        ETextAlign::kLeft => 0.0,
        _ => unreachable!(),
    };
    assert_eq!(control.size.width, 108.0, "{name} at {origin:?}");
    assert!((label.size.width - 52.203125).abs() <= 1.0 / 64.0);
    assert!(
        (actual - expected).abs() <= 1.0 / 64.0,
        "{name}: label starts at {actual}, expected {expected} in a 108px control"
    );
}

#[test]
fn author_text_color_preserves_ua_button_label_centering() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(true);
        for id in ["submit", "reset", "input-button", "button", "flex"] {
            assert_label_alignment(&page, id, ETextAlign::kCenter);
        }
        assert_label_alignment(&page, "left", ETextAlign::kLeft);
        assert_label_alignment(&page, "right", ETextAlign::kRight);
    });
}

#[test]
fn button_alignment_survives_color_change_and_dynamic_insertion() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture(false);
        assert_label_alignment(&page, "button", ETextAlign::kCenter);
        assert!(page
            .Evaluate(
                "document.getElementById('button').style.color='white'; \
             var b=document.createElement('button'); b.id='dynamic'; b.textContent=' Search '; \
             b.style.color='black'; document.querySelector('.row').appendChild(b);",
                "test:button-color",
            )
            .unwrap()
            .Succeeded());
        page.RunTask().unwrap();
        assert_label_alignment(&page, "button", ETextAlign::kCenter);
        assert_label_alignment(&page, "dynamic", ETextAlign::kCenter);
        assert!(page
            .Evaluate(
                "document.getElementById('dynamic').style.textAlign='right'",
                "test:button-alignment",
            )
            .unwrap()
            .Succeeded());
        page.RunTask().unwrap();
        assert_label_alignment(&page, "dynamic", ETextAlign::kRight);
    });
}

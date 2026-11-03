#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::{Page, ScriptEnvironment};
use dom::persistent_document::DOMNodeType;
use dom::Document;
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use layoutng_assembly::{
    fragment_tree::{FragmentPaintData, LayoutResultHandle},
    layout_engine::PaintFragmentCursor,
};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader;
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: br#"<!doctype html><html><head><style>
                body{margin:0;font:24px/32px sans-serif}
                #title{color:#087f2c}
                #row{position:absolute;display:flex;gap:12px;margin:20px}
                .action{display:inline-flex;align-items:center;min-height:48px;padding:0 18px;
                        border:1px solid #333;background:#eee;color:#111}
                #primary{background:#087f2c;color:white}
            </style></head><body>
                <h1 id=title>Server title</h1>
                <div id=row><button id=primary class=action><span><span>Primary action</span></span></button></div>
            </body></html>"#.to_vec(),
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

fn fixture() -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(800, 400);
    constraints.fonts = vec![layoutng_assembly::internal::layout_input::FontFace {
        family: "sans-serif".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader)),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        constraints,
        Some(ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_max_stack_size(
                4 * 1024 * 1024,
            )),
            xhr: Box::new(XHR),
            user_agent: "hydration-flex-sibling-paint-fixture".into(),
        }),
        None,
    );
    page.Open("https://hydration-flex-sibling-paint.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(page.CurrentFrame().is_some());
    page
}

fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .and_then(|attribute| (attribute.value == name).then(|| document.Node(index).Id()))
        })
        .unwrap_or_else(|| panic!("missing node #{name}"))
}

fn text_child_id(page: &Page, owner_id: u64) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    let root = document.FindNodeById(owner_id).unwrap();
    fn first_text(document: &Document, node: usize) -> Option<u64> {
        if document.Node(node).Type() == DOMNodeType::kText {
            return Some(document.Node(node).Id());
        }
        document
            .Node(node)
            .Children()
            .iter()
            .find_map(|&child| first_text(document, child))
    }
    first_text(document, root).expect("nested label must own text")
}

fn find(handle: &LayoutResultHandle, wanted: u64) -> Option<&FragmentPaintData> {
    fn walk(node: PaintFragmentCursor<'_>, wanted: u64) -> Option<PaintFragmentCursor<'_>> {
        if node.PaintData().node_id == wanted {
            Some(node)
        } else {
            node.Children()
                .into_iter()
                .find_map(|child| walk(child, wanted))
        }
    }
    walk(PaintFragmentCursor::FromHandle(handle), wanted).map(PaintFragmentCursor::PaintData)
}

#[test]
fn hydration_appended_flex_sibling_and_replaced_inline_text_enter_same_paint_artifact() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture();
        let result = page
            .Evaluate(
                r#"const title=document.getElementById('title');
                   title.replaceChildren();
                   for(const word of ['Software ','still ','starts ','with ','you']) {
                     const span=document.createElement('span'); span.textContent=word; title.appendChild(span);
                   }
                   const secondary=document.createElement('a');
                   secondary.id='secondary'; secondary.className='action';
                   const outer=document.createElement('span');
                   const label=document.createElement('span'); label.id='secondary-label';
                   label.textContent='Secondary action'; outer.appendChild(label);
                   secondary.appendChild(outer); document.getElementById('row').appendChild(secondary);"#,
                "test:hydrate-flex-sibling",
            )
            .unwrap();
        assert!(result.Succeeded());
        page.RunTask().unwrap();

        let secondary = id(&page, "secondary");
        let label_text = text_child_id(&page, id(&page, "secondary-label"));
        let frame = page.CurrentFrame().unwrap();
        assert!(
            find(&frame.fragments, secondary).is_some(),
            "the appended flex item must be owned by the current fragment tree"
        );
        assert!(
            frame.display_items.items.iter().any(|item| {
                item.node_id == secondary
                    && item.r#type == paint::paint_engine::DisplayItemType::kDrawRect
            }),
            "the appended flex item background/border must enter the paint artifact"
        );
        assert!(
            frame.display_items.items.iter().any(|item| {
                item.node_id == label_text
                    && item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
                    && !item.is_shadow
            }),
            "the appended nested label must enter the same paint artifact"
        );
    });
}

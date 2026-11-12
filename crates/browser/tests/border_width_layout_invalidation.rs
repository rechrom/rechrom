//! Effective border-width changes must invalidate retained LayoutNG geometry.
//! Source: third_party/blink/renderer/core/style/computed_style.cc
//! ComputedStyle::DiffNeedsFullLayout.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::{Page, ScriptEnvironment};
use layoutng_assembly::{
    fragment_tree::{FragmentPaintData, LayoutResultHandle},
    internal::layout_input::{FontFace, Size},
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

struct Loader;

impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: br#"<!doctype html>
                <style>
                  html,body { margin:0 }
                  .product { --bottom:1px; display:grid; width:100px }
                  .product.right.left { --bottom:0px }
                  .surface {
                    display:grid;
                    border:1px solid black;
                    border-bottom-width:var(--bottom);
                  }
                  .child { width:100px; height:100px }
                </style>
                <div id=product class=product>
                  <div id=surface class=surface><div id=child class=child></div></div>
                </div>"#
                .to_vec(),
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
        Err(io::Error::other("unexpected border fixture XHR"))
    }
}

fn fixture() -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(320, 240);
    constraints.fonts = vec![FontFace {
        family: "BorderFixture".into(),
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
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_max_stack_size(
                    4 * 1024 * 1024,
                ),
            ),
            xhr: Box::new(XHR),
            user_agent: "border-width-layout-invalidation-fixture".into(),
        }),
        None,
    );
    page.Open(
        "https://border-width-layout-invalidation.test/",
        16384,
        4096,
    )
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

fn evaluate(page: &mut Page, script: &str) {
    let result = page
        .Evaluate(script, "test:border-width-layout-invalidation")
        .unwrap();
    assert!(result.Succeeded(), "{:?}", result.exception);
}

fn node_id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == name)
                .map(|_| document.Node(index).Id())
        })
        .unwrap_or_else(|| panic!("missing fixture node {name}"))
}

fn find(handle: &LayoutResultHandle, id: u64) -> Option<&FragmentPaintData> {
    fn visit(node: PaintFragmentCursor<'_>, id: u64) -> Option<PaintFragmentCursor<'_>> {
        if node.PaintData().node_id == id {
            Some(node)
        } else {
            node.Children()
                .into_iter()
                .find_map(|child| visit(child, id))
        }
    }
    visit(PaintFragmentCursor::FromHandle(handle), id).map(PaintFragmentCursor::PaintData)
}

fn size(page: &Page, name: &str) -> Size {
    let id = node_id(page, name);
    let frame = page.CurrentFrame().expect("published fixture frame");
    if let Some(fragment) = find(&frame.fragments, id) {
        fragment.size
    } else {
        panic!("missing fixture fragment {name}")
    }
}

#[test]
fn compound_custom_property_and_direct_border_changes_relayout_retained_grid() {
    test_thread::run(|| {
        let mut page = fixture();
        assert_eq!(size(&page, "surface").height, 102.0);
        assert_eq!(size(&page, "child").height, 100.0);

        evaluate(
            &mut page,
            r#"
              (() => {
                const product = document.getElementById('product');
                const surface = document.getElementById('surface');
                product.className = 'product right left';
                const rect = surface.getBoundingClientRect();
                if (getComputedStyle(surface).borderBottomWidth !== '0px' || rect.height !== 101)
                  throw Error('compound border geometry ' + JSON.stringify(rect));
              })();
            "#,
        );
        assert_eq!(size(&page, "surface").height, 101.0);

        evaluate(
            &mut page,
            r#"
              (() => {
                const surface = document.getElementById('surface');
                surface.style.setProperty('border-bottom-width', '2px', 'important');
                const rect = surface.getBoundingClientRect();
                if (getComputedStyle(surface).borderBottomWidth !== '2px' || rect.height !== 103)
                  throw Error('direct border geometry ' + JSON.stringify(rect));
              })();
            "#,
        );
        assert_eq!(size(&page, "surface").height, 103.0);

        evaluate(
            &mut page,
            r#"
              (() => {
                const surface = document.getElementById('surface');
                surface.style.setProperty('border-bottom-width', '0px', 'important');
                const rect = surface.getBoundingClientRect();
                if (getComputedStyle(surface).borderBottomWidth !== '0px' || rect.height !== 101)
                  throw Error('restored border geometry ' + JSON.stringify(rect));
              })();
            "#,
        );
        assert_eq!(size(&page, "surface").height, 101.0);
        assert_eq!(size(&page, "child").height, 100.0);
    });
}

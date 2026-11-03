#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
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

struct Xhr;
impl xhr_transport::XMLHttpRequestTransport for Xhr {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("unexpected fixture XHR"))
    }
}

fn page() -> Page {
    let html = b"<!doctype html><style>body{margin:0}#before{height:300px}#anchor{height:60px;background:yellow}#after{height:1200px}</style><div id=before></div><div id=anchor></div><div id=after></div>";
    let assembly = browser::CreateLayoutAssembly();
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(html.to_vec()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        browser::CreateBrowserConstraints(300, 200),
        Some(browser::page::ScriptEnvironment {
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_max_stack_size(
                    4 * 1024 * 1024,
                ),
            ),
            xhr: Box::new(Xhr),
            user_agent: "scroll-anchor-fixture".into(),
        }),
        None,
    );
    page.Open("https://page.test/anchor.html", 16384, 4096)
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

fn check(page: &mut Page, expression: &str) {
    let result = page.Evaluate(expression, "scroll-anchor-fixture").unwrap();
    assert!(result.Succeeded(), "{:?}", result.exception);
}

#[test]
fn layout_growth_before_viewport_keeps_root_anchor_stable() {
    test_thread::run(|| {
        let mut page = page();
        check(
            &mut page,
            "scrollTo(0,300);if(document.getElementById('anchor').getBoundingClientRect().top!==0)throw Error('initial anchor');",
        );
        check(
            &mut page,
            "const growth=document.createElement('div');growth.style.height='120px';document.body.insertBefore(growth,document.getElementById('anchor'));",
        );
        check(
            &mut page,
            "if(scrollY!==420)throw Error('anchored offset '+scrollY);if(document.getElementById('anchor').getBoundingClientRect().top!==0)throw Error('anchor moved');",
        );
    });
}

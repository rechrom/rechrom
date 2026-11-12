//! LayoutView user-scroll policy while a document is fullscreen.
#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use interaction::input_event::Offset;
use interaction::input_event::{WheelEvent, WheelPhase};
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

fn fixture(children: &str) -> Page {
    let html = format!(
        "<!doctype html><style>html,body{{margin:0}}#fill{{height:800px}}</style>{children}"
    );
    let assembly = browser::CreateLayoutAssembly();
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(html.into_bytes()))),
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
            xhr: Box::new(XHR),
            user_agent: "fullscreen-scroll-fixture".into(),
        }),
        None,
    );
    page.Open("https://page.test/fullscreen.html", 16384, 4096)
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
    let result = page
        .Evaluate(expression, "fullscreen-scroll-fixture")
        .unwrap();
    assert!(result.Succeeded(), "{:?}", result.exception);
}

fn wheel_down(page: &mut Page) {
    page.ScrollWheelDefault(&WheelEvent {
        phase: WheelPhase::kEnded,
        position: Offset { x: 20.0, y: 100.0 },
        delta: Offset { x: 0.0, y: 120.0 },
        ..Default::default()
    })
    .unwrap();
}

#[test]
fn fullscreen_element_controls_layout_view_user_scrolling() {
    test_thread::run(|| {
        let mut page = fixture("<div id=fullscreen></div><div id=fill></div>");
        check(
            &mut page,
            "document.getElementById('fullscreen').requestFullscreen();document.body.getBoundingClientRect()",
        );
        wheel_down(&mut page);
        check(
            &mut page,
            "if(scrollY!==0)throw Error('non-root fullscreen allowed layout-view scrolling '+scrollY)",
        );

        let mut page = fixture("<div id=fill></div>");
        check(
            &mut page,
            "document.documentElement.requestFullscreen();document.body.getBoundingClientRect()",
        );
        wheel_down(&mut page);
        check(
            &mut page,
            "if(scrollY<=0)throw Error('documentElement fullscreen blocked layout-view scrolling')",
        );
    });
}

#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::{Page, ScriptEnvironment};
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
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
            body: b"<!doctype html><html><body></body></html>".to_vec(),
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
        Err(io::Error::other("unexpected resize observer XHR"))
    }
}

fn fixture() -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader)),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        browser::CreateBrowserConstraints(640, 480),
        Some(ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_max_stack_size(
                4 * 1024 * 1024,
            )),
            xhr: Box::new(XHR),
            user_agent: "resize-observer-fixture".into(),
        }),
        None,
    );
    page.Open("https://resize-observer.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
}

#[test]
fn reports_content_and_border_boxes_independently() {
    test_thread::run(|| {
        let mut page = fixture();
        let result = page
            .Evaluate(
                r#"
                const target=document.createElement('div');
                target.style.cssText='width:100.25px;height:50.5px;padding:3.5px 2.5px;border:1.25px solid black;box-sizing:content-box';
                document.body.appendChild(target);
                globalThis.resizeEntries=[];
                globalThis.resizeObserver=new ResizeObserver(entries=>resizeEntries.push(...entries));
                resizeObserver.observe(target);
                "#,
                "test:resize-observer-setup",
            )
            .unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
        for _ in 0..20 {
            page.RunTask().unwrap();
        }
        let result = page
            .Evaluate(
                r#"
                const check=(value,message)=>{if(!value)throw Error(message)};
                const near=(actual,expected,message)=>check(Math.abs(actual-expected)<.02,message+': '+actual);
                check(resizeEntries.length===1,'one initial entry');
                const entry=resizeEntries[0];
                near(entry.contentRect.width,100.25,'content rect width');
                near(entry.contentRect.height,50.5,'content rect height');
                near(entry.contentBoxSize[0].inlineSize,100.25,'content inline size');
                near(entry.contentBoxSize[0].blockSize,50.5,'content block size');
                near(entry.borderBoxSize[0].inlineSize,107.25,'border inline size');
                near(entry.borderBoxSize[0].blockSize,59.5,'border block size');
                "#,
                "test:resize-observer-assert",
            )
            .unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
    });
}

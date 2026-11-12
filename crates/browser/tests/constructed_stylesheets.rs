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
        Err(io::Error::other("unexpected constructed stylesheet XHR"))
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
            user_agent: "constructed-stylesheet-fixture".into(),
        }),
        None,
    );
    page.Open("https://constructed-stylesheet.test/", 16384, 4096)
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
fn replace_sync_and_shadow_adoption_use_the_native_sheet_path() {
    test_thread::run(|| {
        let mut page = fixture();
        let result = page
            .Evaluate(
                r#"
                const check=(value,message)=>{if(!value)throw Error(message)};
                const near=(actual,expected,message)=>check(Math.abs(actual-expected)<.05,message+': '+actual);
                const host=document.createElement('x-constructed');
                document.body.appendChild(host);
                const root=host.attachShadow({mode:'closed'});
                const target=document.createElement('div');
                target.id='target';
                target.textContent='native sheet';
                root.appendChild(target);
                const sheet=new CSSStyleSheet();
                sheet.replaceSync(':host{display:block} #target{display:block;width:123px;height:37px;background:green}');
                check(sheet.ownerNode===null,'constructed owner');
                check(sheet.cssRules.length===2,'parsed rule list');
                root.adoptedStyleSheets=[sheet];
                check(root.adoptedStyleSheets.length===1&&root.adoptedStyleSheets[0]===sheet,'wrapper identity');
                near(target.getBoundingClientRect().width,123,'adopted width');
                near(target.getBoundingClientRect().height,37,'adopted height');
                check(root.querySelectorAll('style').length===0,'fallback style node');
                sheet.replaceSync('#target{display:block;width:147px;height:41px}');
                near(target.getBoundingClientRect().width,147,'replacement width');
                near(target.getBoundingClientRect().height,41,'replacement height');
                "#,
                "test:constructed-stylesheet",
            )
            .unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
    });
}

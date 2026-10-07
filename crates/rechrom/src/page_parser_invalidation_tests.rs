//! Parsing adds nodes independently of JS DOM mutation callbacks. A previous
//! script's cached computed styles must not hide the newly parsed elements.
use crate::page::{Page, PageClient};
use javascript::{
    javascript_runtime::JavaScriptException, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
struct Loader;
struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
impl URLLoader for Loader {
    fn Load(&mut self, _: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Response(Some(URLResponse {
            body: br#"<html><head><script>
                globalThis.prime=getComputedStyle(document.documentElement).display;
            </script></head><body><div id='later' style='width:29px;height:10px'></div><script>
                globalThis.observed=getComputedStyle(document.getElementById('later')).width;
                if(observed!=='29px')throw Error('new parser node style: '+observed);
            </script></body></html>"#
                .to_vec(),
            mime_type: "text/html".into(),
            status_code: 200,
            final_url: "https://parser.test/".into(),
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
        Err(io::Error::other("no XHR expected"))
    }
}
#[derive(Default)]
struct Client(Vec<String>);
impl PageClient for Client {
    fn DidReportScriptError(&mut self, e: &JavaScriptException) {
        self.0.push(e.message.clone());
    }
}
#[test]
fn parsing_after_a_style_read_invalidates_page_styles_before_the_next_script() {
    crate::native_test_thread::run(|| {
        let assembly = crate::CreateLayoutAssembly();
        let client = Rc::new(RefCell::new(Client::default()));
        let mut page = Page::Create(
            Rc::new(RefCell::new(Loader)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(
                image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
            )),
            crate::CreateBrowserConstraints(160, 96),
            Some(crate::page::ScriptEnvironment {
                runtime: Box::new(QuickJsJavaScriptRuntime::new()),
                xhr: Box::new(XHR),
                user_agent: "test".into(),
            }),
            Some(client.clone()),
        );
        page.OpenSynchronously("https://parser.test/", 16384, 4096)
            .unwrap();

        assert!(client.borrow().0.is_empty(), "{:?}", client.borrow().0);
        assert!(page.CurrentFrame().is_some());
    });
}

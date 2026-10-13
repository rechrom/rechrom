#[path = "../src/native_test_thread.rs"]
mod native_test_thread;
use browser::page::{Page, PageClient, ScriptEnvironment};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    io,
    rc::Rc,
};
use url_loader::{
    URLLoadEvent, URLLoadOperation, URLLoader, URLRequest, URLResponse, URLResponseHead,
    URLStreamOperation,
};
type Events = Rc<RefCell<VecDeque<URLLoadEvent>>>;
type Responses = Rc<RefCell<HashMap<String, Rc<RefCell<Option<URLResponse>>>>>>;
struct Body(Events);
impl URLStreamOperation for Body {
    fn Poll(&mut self, budget: usize) -> io::Result<Option<URLLoadEvent>> {
        let mut events = self.0.borrow_mut();
        if let Some(URLLoadEvent::Data(data)) = events.front_mut() {
            if data.len() > budget {
                return Ok(Some(URLLoadEvent::Data(data.drain(..budget).collect())));
            }
        }
        Ok(events.pop_front())
    }
}
struct Resource(Rc<RefCell<Option<URLResponse>>>);
impl URLLoadOperation for Resource {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.borrow_mut().take())
    }
}
struct Loader {
    body: Events,
    resources: Responses,
}
impl URLLoader for Loader {
    fn LoadStream(&mut self, _: &URLRequest) -> io::Result<Box<dyn URLStreamOperation>> {
        Ok(Box::new(Body(self.body.clone())))
    }
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        if request.url.ends_with("/fail.css") {
            return Err(io::Error::other("fixture stylesheet failure"));
        }
        Ok(Box::new(Resource(
            self.resources
                .borrow_mut()
                .entry(request.url.clone())
                .or_default()
                .clone(),
        )))
    }
}
struct Client(Rc<Cell<usize>>);
impl PageClient for Client {
    fn DidFinishLoad(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
fn create(scripting: bool) -> (Page, Events, Responses, Rc<Cell<usize>>) {
    let body = Rc::new(RefCell::new(VecDeque::new()));
    let resources = Rc::new(RefCell::new(HashMap::new()));
    let finished = Rc::new(Cell::new(0));
    let assembly = browser::CreateLayoutAssembly();
    let scripts = scripting.then(|| ScriptEnvironment {
        runtime: Box::new(javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::new()),
        xhr: xhr_transport::CreateHTTPXMLHttpRequestTransport(&Default::default()).unwrap(),
        user_agent: "stream-test".into(),
    });
    let page = Page::Create(
        Rc::new(RefCell::new(Loader {
            body: body.clone(),
            resources: resources.clone(),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        browser::CreateBrowserConstraints(320, 200),
        scripts,
        Some(Rc::new(RefCell::new(Client(finished.clone())))),
    );
    (page, body, resources, finished)
}
fn response() -> URLLoadEvent {
    URLLoadEvent::Response(URLResponseHead {
        final_url: "https://stream.test/page".into(),
        mime_type: "text/html".into(),
        text_encoding: "utf-8".into(),
        status_code: 200,
        ..Default::default()
    })
}
fn finish(page: &mut Page) {
    for _ in 0..1000 {
        page.RunTask().unwrap();
        if !page.IsLoading() {
            return;
        }
    }
    panic!("loading failed to finish within bounded turns");
}
fn assert_script(page: &mut Page, source: &str) {
    let result = page.Evaluate(source, "assert:module-scheduling").unwrap();
    assert!(result.Succeeded(), "{:?}", result.exception);
}

#[test]
fn synchronous_scroll_queries_before_html_arrives_keep_navigation_alive() {
    crate::native_test_thread::run(|| {
        let (mut page, body, _, finished) = create(true);
        body.borrow_mut().push_back(response());
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        assert_script(
            &mut page,
            "if(window.scrollX!==0 || window.scrollY!==0) throw Error('empty viewport');",
        );
        body.borrow_mut().extend([
            URLLoadEvent::Data(
                b"<body style='margin:0'><div style='height:2000px'>late body</div></body>"
                    .to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        finish(&mut page);
        assert_eq!(finished.get(), 1);
        assert_script(&mut page, "window.scrollTo(0,100); if(window.scrollY!==100) throw Error('late body did not scroll');");
    });
}

#[test]
fn hidden_document_element_geometry_is_empty_and_can_be_restored() {
    native_test_thread::run(|| {
        let (mut page, body, _, _) = create(true);
        body.borrow_mut().extend([response(),URLLoadEvent::Data(b"<html style='display:none'><body style='margin:0'><div style='height:2000px'>content</div></body></html>".to_vec()),URLLoadEvent::Finished]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        finish(&mut page);
        assert_script(&mut page,"if(window.scrollY!==0 || document.documentElement.getBoundingClientRect().width!==0) throw Error('hidden root geometry');");
        assert_script(&mut page,"document.documentElement.style.display='block';window.scrollTo(0,100);if(window.scrollY!==100) throw Error('restored document');");
    });
}

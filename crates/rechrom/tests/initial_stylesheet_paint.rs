#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use rechrom::page::Page;
use std::{
    cell::RefCell,
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

fn page() -> (Page, Events, Responses) {
    let body = Rc::new(RefCell::new(VecDeque::new()));
    let resources = Rc::new(RefCell::new(HashMap::new()));
    let assembly = rechrom::CreateLayoutAssembly();
    let page = Page::Create(
        Rc::new(RefCell::new(Loader {
            body: body.clone(),
            resources: resources.clone(),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(document_image::SVGImageDecoder::new(
            &assembly,
        ))),
        rechrom::CreateBrowserConstraints(320, 200),
        None,
        None,
    );
    (page, body, resources)
}

fn response() -> URLLoadEvent {
    URLLoadEvent::Response(URLResponseHead {
        final_url: "https://stream.test/page".into(),
        mime_type: "text/html".into(),
        text_encoding: "utf-8".into(),
        status_code: 200,
    })
}

fn pump(page: &mut Page, condition: impl Fn(&Page) -> bool) {
    for _ in 0..100 {
        page.RunTasks(0.0).unwrap();
        if condition(page) {
            return;
        }
    }
    panic!("page condition did not become true within bounded turns");
}

#[test]
fn first_content_frame_waits_for_head_stylesheets_only() {
    native_test_thread::run(|| {
        let (mut styled, body, resources) = page();
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                b"<head><script type=importmap>{}</script>\n<link rel=stylesheet href=slow.css></head><body><p>Styled</p></body>".to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        styled.Open("https://stream.test/", 4096, 4096).unwrap();
        for _ in 0..8 {
            styled.RunTasks(0.0).unwrap();
        }
        assert!(!styled.IsRenderingReady());
        assert!(styled.CurrentFrame().is_none());
        *resources
            .borrow()
            .get("https://stream.test/slow.css")
            .unwrap()
            .borrow_mut() = Some(URLResponse {
            body: b"p { color: green }".to_vec(),
            ..Default::default()
        });
        pump(&mut styled, |page| page.CurrentFrame().is_some());
        assert_eq!(styled.Document().GetDocument().StyleSheets(None).len(), 1);

        let (mut body_sheet, body, resources) = page();
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                b"<body><link rel=stylesheet href=late.css><p>Already renderable</p>".to_vec(),
            ),
        ]);
        body_sheet.Open("https://stream.test/", 4096, 4096).unwrap();
        pump(&mut body_sheet, |page| page.CurrentFrame().is_some());
        assert!(resources
            .borrow()
            .contains_key("https://stream.test/late.css"));
        assert!(body_sheet.IsRenderingReady());

        let (mut plain, body, _) = page();
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(b"<body><p>Plain</p></body>".to_vec()),
            URLLoadEvent::Finished,
        ]);
        plain.Open("https://stream.test/", 4096, 4096).unwrap();
        pump(&mut plain, |page| page.CurrentFrame().is_some());

        let (mut failed, body, _) = page();
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                b"<head><script type=importmap>{}</script>\n<link rel=stylesheet href=fail.css></head><body><p>Fallback</p></body>".to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        failed.Open("https://stream.test/", 4096, 4096).unwrap();
        pump(&mut failed, |page| page.CurrentFrame().is_some());
        assert_eq!(failed.Document().GetDocument().StyleSheets(None).len(), 0);
    });
}

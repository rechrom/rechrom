use super::*;
use dom::DOM;
use html::html_parser::HTMLParserStatus;
use html::{HTMLParserHost, ParserElementEvent};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    io,
    rc::Rc,
};
use url_loader::{
    URLLoadEvent, URLLoadOperation, URLLoader, URLRequest, URLResponseHead, URLStreamOperation,
};
struct Host;
impl HTMLParserHost for Host {
    fn HandleParserElement(&mut self, _: ParserElementEvent<'_>) {}
}
struct Stream {
    events: Rc<RefCell<VecDeque<URLLoadEvent>>>,
    polls: Rc<Cell<usize>>,
    dropped: Rc<Cell<bool>>,
}
impl Drop for Stream {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}
impl URLStreamOperation for Stream {
    fn Poll(&mut self, max_bytes: usize) -> io::Result<Option<URLLoadEvent>> {
        self.polls.set(self.polls.get() + 1);
        let mut events = self.events.borrow_mut();
        if let Some(URLLoadEvent::Data(bytes)) = events.front_mut() {
            if bytes.len() > max_bytes {
                return Ok(Some(URLLoadEvent::Data(bytes.drain(..max_bytes).collect())));
            }
        }
        Ok(events.pop_front())
    }
}
struct Loader {
    events: Rc<RefCell<VecDeque<URLLoadEvent>>>,
    polls: Rc<Cell<usize>>,
    dropped: Rc<Cell<bool>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, _: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        panic!("main document must use streaming delivery")
    }
    fn LoadStream(&mut self, _: &URLRequest) -> io::Result<Box<dyn URLStreamOperation>> {
        Ok(Box::new(Stream {
            events: self.events.clone(),
            polls: self.polls.clone(),
            dropped: self.dropped.clone(),
        }))
    }
}
fn setup() -> (
    DocumentLoader,
    Rc<RefCell<VecDeque<URLLoadEvent>>>,
    Rc<Cell<usize>>,
    Rc<Cell<bool>>,
) {
    let events = Rc::new(RefCell::new(VecDeque::new()));
    let polls = Rc::new(Cell::new(0));
    let dropped = Rc::new(Cell::new(false));
    let backend = Rc::new(RefCell::new(Loader {
        events: events.clone(),
        polls: polls.clone(),
        dropped: dropped.clone(),
    }));
    let mut loader = DocumentLoader::new(backend);
    loader
        .StartLoading(&URLRequest {
            url: "https://stream.test/".into(),
            ..Default::default()
        })
        .unwrap();
    (loader, events, polls, dropped)
}
fn head() -> URLLoadEvent {
    URLLoadEvent::Response(URLResponseHead {
        final_url: "https://stream.test/final".into(),
        mime_type: "text/html".into(),
        text_encoding: "utf-8".into(),
        status_code: 200,
    })
}
fn text(owner: &DOM) -> String {
    fn visit(d: &dom::Document, i: usize, output: &mut String) {
        if d.Node(i).Type() == dom::persistent_document::DOMNodeType::kText {
            output.push_str(d.Node(i).Data());
        }
        for &child in d.Node(i).Children() {
            visit(d, child, output);
        }
    }
    let mut result = String::new();
    visit(owner.GetDocument(), owner.GetDocument().Root(), &mut result);
    result
}
#[test]
fn streaming_headers_dom_before_eof_split_bom_and_utf8_and_resident_identity() {
    let (mut loader, events, _, dropped) = setup();
    assert!(loader.PollResponse().unwrap().is_none());
    events.borrow_mut().push_back(head());
    assert_eq!(
        loader.PollResponse().unwrap().unwrap().final_url,
        "https://stream.test/final"
    );
    assert!(loader.PollResponse().unwrap().is_none());
    let mut owner = DOM::new();
    let root = owner.GetDocument().RootHandle();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut Host)
        .unwrap();
    events.borrow_mut().push_back(URLLoadEvent::Data(
        b"\xef\xbb\xbf<body><p>early</p>".to_vec(),
    ));
    for _ in 0..100 {
        let result = loader
            .Pump(
                owner.GetDocumentMut(),
                &mut Host,
                DocumentLoadBudget {
                    body_bytes: 1,
                    parser_tokens: 2,
                },
            )
            .unwrap();
        assert!(!result.body_finished);
        if text(&owner).contains("early") {
            break;
        }
    }
    assert!(text(&owner).contains("early"));
    assert!(!dropped.get());
    events.borrow_mut().extend([
        URLLoadEvent::Data("<p>世界😀</p></body>".as_bytes().to_vec()),
        URLLoadEvent::Finished,
    ]);
    for _ in 0..200 {
        let result = loader
            .Pump(
                owner.GetDocumentMut(),
                &mut Host,
                DocumentLoadBudget {
                    body_bytes: 1,
                    parser_tokens: 2,
                },
            )
            .unwrap();
        if result.parser.status == HTMLParserStatus::kFinished {
            assert!(result.body_finished);
            break;
        }
    }
    assert_eq!(loader.Status(), DocumentLoaderStatus::kFinished);
    assert_eq!(text(&owner), "early世界😀");
    assert_eq!(owner.GetDocument().RootHandle(), root);
    assert!(dropped.get());
}
#[test]
fn script_pause_applies_backpressure_and_eof_is_distinct_from_parser_completion() {
    let (mut loader, events, polls, dropped) = setup();
    events.borrow_mut().push_back(head());
    loader.PollResponse().unwrap();
    let mut owner = DOM::new();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut Host)
        .unwrap();
    events.borrow_mut().extend([
        URLLoadEvent::Data(b"<body>before<script>hold()</script>after".to_vec()),
        URLLoadEvent::Finished,
    ]);
    let result = loader
        .Pump(
            owner.GetDocumentMut(),
            &mut Host,
            DocumentLoadBudget {
                body_bytes: 4096,
                parser_tokens: 4096,
            },
        )
        .unwrap();
    assert!(result.body_finished);
    assert_eq!(result.parser.status, HTMLParserStatus::kWaitingForScript);
    assert_eq!(loader.Status(), DocumentLoaderStatus::kParsing);
    assert!(dropped.get());
    let count = polls.get();
    let result = loader
        .Pump(owner.GetDocumentMut(), &mut Host, 4096)
        .unwrap();
    assert_eq!(polls.get(), count);
    assert!(!result.made_progress);
    assert!(!text(&owner).contains("after"));
    loader.InsertFromScript("<b>written</b>").unwrap();
    loader.ResumeAfterScript().unwrap();
    let result = loader
        .Pump(owner.GetDocumentMut(), &mut Host, 4096)
        .unwrap();
    assert_eq!(result.parser.status, HTMLParserStatus::kFinished);
    assert!(text(&owner).contains("writtenafter"));
}
#[test]
fn yielding_does_not_read_more_body_and_cancellation_preserves_dom() {
    let (mut loader, events, polls, dropped) = setup();
    events.borrow_mut().push_back(head());
    loader.PollResponse().unwrap();
    let mut owner = DOM::new();
    let root = owner.GetDocument().RootHandle();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut Host)
        .unwrap();
    events
        .borrow_mut()
        .push_back(URLLoadEvent::Data(b"<body><p>one</p><p>two</p>".to_vec()));
    let result = loader.Pump(owner.GetDocumentMut(), &mut Host, 1).unwrap();
    assert_eq!(result.parser.status, HTMLParserStatus::kYielded);
    let count = polls.get();
    events
        .borrow_mut()
        .push_back(URLLoadEvent::Data(b"more".to_vec()));
    loader.Pump(owner.GetDocumentMut(), &mut Host, 1).unwrap();
    assert_eq!(polls.get(), count);
    loader.StopLoading();
    assert!(dropped.get());
    assert_eq!(loader.Status(), DocumentLoaderStatus::kCancelled);
    assert_eq!(owner.GetDocument().RootHandle(), root);
    assert!(loader.Pump(owner.GetDocumentMut(), &mut Host, 1).is_err());
}
#[test]
fn malformed_stream_order_and_truncated_utf8_fail_without_finishing_parser() {
    let (mut loader, events, _, dropped) = setup();
    events.borrow_mut().push_back(URLLoadEvent::Finished);
    assert!(loader.PollResponse().is_err());
    assert!(dropped.get());
    let (mut loader, events, _, _) = setup();
    events.borrow_mut().push_back(head());
    loader.PollResponse().unwrap();
    let mut owner = DOM::new();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut Host)
        .unwrap();
    events
        .borrow_mut()
        .extend([URLLoadEvent::Data(vec![0xf0, 0x9f]), URLLoadEvent::Finished]);
    assert!(loader
        .Pump(owner.GetDocumentMut(), &mut Host, 4096)
        .is_err());
    assert_eq!(loader.Status(), DocumentLoaderStatus::kFailed);
}

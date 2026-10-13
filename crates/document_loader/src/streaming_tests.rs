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
        ..Default::default()
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

#[test]
fn production_preloads_continue_input_while_script_blocks_without_mutating_dom() {
    use html::preload_scanner::PreloadCandidate;
    #[derive(Default)]
    struct PreloadHost(Vec<PreloadCandidate>);
    impl HTMLParserHost for PreloadHost {
        fn HandleParserElement(&mut self, _: ParserElementEvent<'_>) {}
        fn HandlePreloadCandidate(&mut self, candidate: &PreloadCandidate) {
            self.0.push(candidate.clone());
        }
    }
    let (mut loader, events, _, _) = setup();
    loader.EnablePreloadScanning(true);
    events.borrow_mut().push_back(head());
    loader.PollResponse().unwrap();
    let mut owner = DOM::new();
    let mut host = PreloadHost::default();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut host)
        .unwrap();
    events.borrow_mut().push_back(URLLoadEvent::Data(
        b"<body>before<script>hold()</script>".to_vec(),
    ));
    let budget = DocumentLoadBudget {
        body_bytes: 4096,
        parser_tokens: 4096,
    };
    assert_eq!(
        loader
            .Pump(owner.GetDocumentMut(), &mut host, budget)
            .unwrap()
            .parser
            .status,
        HTMLParserStatus::kWaitingForScript
    );
    let before = text(&owner);
    events.borrow_mut().extend([
        URLLoadEvent::Data("<img src='你好.png'>after</body>".as_bytes().to_vec()),
        URLLoadEvent::Finished,
    ]);
    let progress = loader
        .Pump(owner.GetDocumentMut(), &mut host, budget)
        .unwrap();
    assert!(progress.made_progress && progress.body_finished);
    assert!(!progress.parser_made_progress);
    assert_eq!(loader.Status(), DocumentLoaderStatus::kParsing);
    assert_eq!(
        text(&owner),
        before,
        "speculation cannot advance the tree builder"
    );
    assert!(host.0.iter().any(|candidate| candidate
        .attributes
        .iter()
        .any(|(name, value)| name == "src" && value == "你好.png")));
    loader.InsertFromScript("<b>written</b>").unwrap();
    loader.ResumeAfterScript().unwrap();
    let result = loader
        .Pump(owner.GetDocumentMut(), &mut host, budget)
        .unwrap();
    assert_eq!(result.parser.status, HTMLParserStatus::kFinished);
    assert!(result.parser_made_progress);
    assert_eq!(text(&owner), "beforehold()writtenafter");
}

#[test]
fn diagnostic_read_ahead_holds_eof_and_preserves_script_insertion_order() {
    let (mut loader, events, polls, _) = setup();
    loader.EnableDiagnosticReadAhead(true);
    events.borrow_mut().push_back(head());
    loader.PollResponse().unwrap();
    let mut owner = DOM::new();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut Host)
        .unwrap();
    let prefix = b"<body>before<script>hold()</script>";
    events
        .borrow_mut()
        .push_back(URLLoadEvent::Data(prefix.to_vec()));
    let budget = DocumentLoadBudget {
        body_bytes: prefix.len(),
        parser_tokens: 4096,
    };
    assert_eq!(
        loader
            .Pump(owner.GetDocumentMut(), &mut Host, budget)
            .unwrap()
            .parser
            .status,
        HTMLParserStatus::kWaitingForScript
    );
    events.borrow_mut().extend([
        URLLoadEvent::Data(
            "<p>世界😀</p><img src='later.png'>after</body>"
                .as_bytes()
                .to_vec(),
        ),
        URLLoadEvent::Finished,
    ]);
    let count = polls.get();
    let before = text(&owner);
    let progress = loader
        .Pump(owner.GetDocumentMut(), &mut Host, budget)
        .unwrap();
    assert!(polls.get() > count);
    assert!(!progress.made_progress);
    assert!(
        !progress.body_finished,
        "diagnostic EOF must not finish authoritative input"
    );
    assert_eq!(text(&owner), before, "read-ahead must not build DOM");
    loader.InsertFromScript("<b>written</b>").unwrap();
    loader.ResumeAfterScript().unwrap();
    for _ in 0..100 {
        if loader
            .Pump(
                owner.GetDocumentMut(),
                &mut Host,
                DocumentLoadBudget {
                    body_bytes: 1,
                    parser_tokens: 4,
                },
            )
            .unwrap()
            .parser
            .status
            == HTMLParserStatus::kFinished
        {
            break;
        }
    }
    assert_eq!(loader.Status(), DocumentLoaderStatus::kFinished);
    assert_eq!(text(&owner), "beforehold()written世界😀after");
}

#[test]
fn diagnostic_read_ahead_cap_restores_backpressure_and_stop_drops_queue() {
    let (mut loader, events, polls, dropped) = setup();
    loader.EnableDiagnosticReadAhead(true);
    events.borrow_mut().push_back(head());
    loader.PollResponse().unwrap();
    let mut owner = DOM::new();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut Host)
        .unwrap();
    let prefix = b"<body>before<script>hold()</script>";
    events
        .borrow_mut()
        .push_back(URLLoadEvent::Data(prefix.to_vec()));
    let budget = DocumentLoadBudget {
        body_bytes: prefix.len(),
        parser_tokens: 4096,
    };
    loader
        .Pump(owner.GetDocumentMut(), &mut Host, budget)
        .unwrap();
    events
        .borrow_mut()
        .push_back(URLLoadEvent::Data(vec![b'x'; 2 * 1024 * 1024]));
    for _ in 0..80 {
        loader
            .Pump(owner.GetDocumentMut(), &mut Host, budget)
            .unwrap();
    }
    let count = polls.get();
    loader
        .Pump(owner.GetDocumentMut(), &mut Host, budget)
        .unwrap();
    assert_eq!(
        polls.get(),
        count,
        "bounded diagnostic must resume backpressure"
    );
    assert!(!events.borrow().is_empty());
    assert_eq!(text(&owner), "beforehold()");
    loader.StopLoading();
    assert!(dropped.get());
    assert_eq!(loader.Status(), DocumentLoaderStatus::kCancelled);
}

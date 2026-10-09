use super::*;
use dom::{Document, DOM};
use html::html_parser::HTMLParserStatus;
use html::{HTMLParserHost, ParserElementEvent};
use std::{
    cell::{Cell, RefCell},
    io,
    rc::Rc,
};
use url_loader::{RequestDestination, URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Operation {
    pending: usize,
    response: Option<io::Result<URLResponse>>,
    drops: Rc<Cell<usize>>,
}
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        if self.pending > 0 {
            self.pending -= 1;
            return Ok(None);
        }
        self.response.take().transpose()
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
struct Loader {
    requests: Rc<RefCell<Vec<URLRequest>>>,
    pending: usize,
    response: Option<io::Result<URLResponse>>,
    drops: Rc<Cell<usize>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.requests.borrow_mut().push(request.clone());
        Ok(Box::new(Operation {
            pending: self.pending,
            response: self.response.take(),
            drops: self.drops.clone(),
        }))
    }
}
fn backend(
    response: io::Result<URLResponse>,
    pending: usize,
) -> (Rc<RefCell<Loader>>, Rc<Cell<usize>>) {
    let drops = Rc::new(Cell::new(0));
    (
        Rc::new(RefCell::new(Loader {
            requests: Rc::new(RefCell::new(vec![])),
            pending,
            response: Some(response),
            drops: drops.clone(),
        })),
        drops,
    )
}
#[derive(Default)]
struct Host;
impl HTMLParserHost for Host {
    fn HandleParserElement(&mut self, _: ParserElementEvent<'_>) {}
}
fn has_id(d: &Document, id: &str) -> bool {
    fn visit(d: &Document, i: usize, id: &str) -> bool {
        d.Node(i).FindAttribute("id").is_some_and(|a| a.value == id)
            || d.Node(i).Children().iter().any(|&c| visit(d, c, id))
    }
    visit(d, d.Root(), id)
}

#[test]
fn resource_completion_remains_terminal_after_consumption_and_releases_operation() {
    let (backend, drops) = backend(
        Ok(URLResponse {
            body: vec![0, 255],
            ..Default::default()
        }),
        1,
    );
    let mut resource = ResourceLoader::Start(&mut *backend.borrow_mut(), &URLRequest::default());
    assert!(!resource.Poll());
    assert!(resource.Poll());
    assert_eq!(resource.Status(), ResourceLoadStatus::kComplete);
    assert_eq!(drops.get(), 1);
    assert_eq!(
        RequireResponse(resource.TakeResult()).unwrap().body,
        [0, 255]
    );
    assert!(resource.Poll());
    assert!(resource.TakeResult().response.is_none());
    resource.Cancel();
    assert_eq!(resource.Status(), ResourceLoadStatus::kComplete);
    assert_eq!(drops.get(), 1);
}

#[test]
fn resource_failure_and_cancellation_are_terminal_without_repolling_backend() {
    let (backend, drops) = backend(Err(io::Error::other("network failure")), 0);
    let mut resource = ResourceLoader::Start(&mut *backend.borrow_mut(), &URLRequest::default());
    assert!(resource.Poll());
    assert_eq!(resource.Status(), ResourceLoadStatus::kFailed);
    assert_eq!(resource.TakeResult().error, "network failure");
    assert!(resource.Poll());
    assert_eq!(drops.get(), 1);
    let (backend, drops) = self::backend(Ok(URLResponse::default()), 10);
    let mut resource = ResourceLoader::Start(&mut *backend.borrow_mut(), &URLRequest::default());
    resource.Cancel();
    assert!(resource.Poll());
    assert_eq!(resource.Status(), ResourceLoadStatus::kCancelled);
    assert_eq!(drops.get(), 1);
    assert!(resource.TakeResult().error.contains("cancelled"));
}

#[test]
fn main_document_redirect_encoding_and_one_shot_headers_precede_parsing() {
    let (backend, drops) = backend(
        Ok(URLResponse {
            final_url: "https://cdn.test/page".into(),
            mime_type: "text/html".into(),
            text_encoding: "windows-1252".into(),
            body: vec![0x80],
            ..Default::default()
        }),
        1,
    );
    let mut loader = DocumentLoader::new(backend.clone());
    loader
        .StartLoading(&URLRequest {
            url: "https://test/page".into(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        backend.borrow().requests.borrow()[0].destination,
        RequestDestination::kDocument
    );
    assert_eq!(loader.Status(), DocumentLoaderStatus::kLoading);
    assert!(loader.PollResponse().unwrap().is_none());
    let document = loader.PollResponse().unwrap().unwrap();
    assert_eq!(document.final_url, "https://cdn.test/page");
    assert_eq!(loader.URL(), document.final_url);
    assert_eq!(loader.Status(), DocumentLoaderStatus::kResponseReady);
    assert_eq!(drops.get(), 1);
    assert!(loader.PollResponse().unwrap().is_none());
    assert!(loader
        .StartLoading(&URLRequest {
            url: "https://another.test".into(),
            ..Default::default()
        })
        .is_err());
}

#[test]
fn document_failures_and_cancelled_loads_do_not_enter_parsing() {
    let (backend, drops) = backend(
        Ok(URLResponse {
            text_encoding: "unsupported".into(),
            ..Default::default()
        }),
        0,
    );
    let mut loader = DocumentLoader::new(backend);
    loader
        .StartLoading(&URLRequest {
            url: "https://test/page".into(),
            ..Default::default()
        })
        .unwrap();
    loader.PollResponse().unwrap().unwrap();
    assert!(loader
        .BeginParsing(DOM::new().GetDocumentMut(), &mut Host)
        .is_err());
    assert_eq!(loader.Status(), DocumentLoaderStatus::kFailed);
    assert_eq!(drops.get(), 1);
    assert!(loader.PollResponse().is_err());
    assert!(loader
        .BeginParsing(DOM::new().GetDocumentMut(), &mut Host)
        .is_err());
    let (backend, drops) = self::backend(Ok(URLResponse::default()), 10);
    let mut loader = DocumentLoader::new(backend);
    loader
        .StartLoading(&URLRequest {
            url: "https://test/page".into(),
            ..Default::default()
        })
        .unwrap();
    loader.StopLoading();
    assert_eq!(loader.Status(), DocumentLoaderStatus::kCancelled);
    assert_eq!(drops.get(), 1);
    assert!(loader.PollResponse().unwrap().is_none());
    assert!(loader
        .BeginParsing(DOM::new().GetDocumentMut(), &mut Host)
        .is_err());
}

#[test]
fn resident_document_parser_keeps_identity_script_pause_and_input_between_turns() {
    let (backend, _) = backend(Ok(URLResponse::default()), 0);
    let mut loader = DocumentLoader::new(backend);
    let mut owner = DOM::new();
    let mut host = Host;
    let root = owner.GetDocument().RootHandle();
    loader
        .BeginParsing(owner.GetDocumentMut(), &mut host)
        .unwrap();
    loader
        .Append("<body><p id=early>你好</p><script>blocking()</script>\n")
        .unwrap();
    assert!(loader.Pump(owner.GetDocumentMut(), &mut host, 0).is_err());
    let status = loop {
        let status = loader
            .Pump(owner.GetDocumentMut(), &mut host, 1)
            .unwrap()
            .parser
            .status;
        if status != HTMLParserStatus::kYielded {
            break status;
        }
    };
    assert_eq!(status, HTMLParserStatus::kWaitingForScript);
    assert!(has_id(owner.GetDocument(), "early"));
    loader.Append("<div id=later>later</div>").unwrap();
    loader.FinishInput().unwrap();
    assert_eq!(
        loader
            .Pump(owner.GetDocumentMut(), &mut host, 64)
            .unwrap()
            .parser
            .status,
        HTMLParserStatus::kWaitingForScript
    );
    assert!(!has_id(owner.GetDocument(), "later"));
    loader
        .InsertFromScript("<b id=written>written</b>")
        .unwrap();
    loader.ResumeAfterScript().unwrap();
    loop {
        if loader
            .Pump(owner.GetDocumentMut(), &mut host, 1)
            .unwrap()
            .parser
            .status
            == HTMLParserStatus::kFinished
        {
            break;
        }
    }
    assert_eq!(loader.Status(), DocumentLoaderStatus::kFinished);
    assert_eq!(owner.GetDocument().RootHandle(), root);
    assert!(has_id(owner.GetDocument(), "written"));
    assert!(has_id(owner.GetDocument(), "later"));
    assert!(loader.Append("tail").is_err());
}

struct Images;
impl image_decoder::image_decoder::ImageDecoder for Images {
    fn Decode(
        &mut self,
        input: &image_decoder::image_decoder::ImageDecodeInput<'_>,
    ) -> io::Result<image_decoder::image_decoder::DecodedImage> {
        assert_eq!(input.bytes, [1, 2, 3, 255]);
        Ok(image_decoder::image_decoder::DecodedImage {
            width: 1,
            height: 1,
            rgba8: input.bytes.into(),
        })
    }
}
struct DocumentImages;
impl image_resource::DocumentImageDecoder for DocumentImages {
    fn can_decode(&self, _: &[u8], _: &str) -> bool {
        false
    }
    fn create(
        &mut self,
        _: image_resource::ImageId,
        _: std::sync::Arc<[u8]>,
        _: &str,
        _: &image_resource::ContainerKey,
    ) -> io::Result<image_resource::CreatedDocumentImage> {
        panic!("ordinary image must use image decoder")
    }
}
#[derive(Default)]
struct Client {
    mutations: Vec<page_mutation::ResourceMutation>,
}
impl ResourceFetcherClient for Client {
    fn ApplyResourceMutation(&mut self, mutation: page_mutation::ResourceMutation) {
        self.mutations.push(mutation);
    }
    fn DispatchImageEvent(&mut self, _: u64, _: bool) {
        panic!("no JavaScript event dispatch")
    }
    fn HasJavaScript(&self) -> bool {
        false
    }
}
fn fetcher(backend: Rc<RefCell<dyn URLLoader>>) -> ResourceFetcher {
    ResourceFetcher::new(
        backend,
        Rc::new(RefCell::new(Images)),
        Rc::new(RefCell::new(DocumentImages)),
        Rc::new(RefCell::new(DOM::new())),
        Rc::new(RefCell::new(
            layoutng_assembly::internal::layout_input::ConstraintSpace::default(),
        )),
        "https://page.test/root/entry".into(),
    )
}
#[test]
fn fetcher_deduplicates_image_requests_and_delivers_only_completed_mutations() {
    let (backend, drops) = backend(
        Ok(URLResponse {
            body: vec![1, 2, 3, 255],
            mime_type: "image/png".into(),
            ..Default::default()
        }),
        1,
    );
    let fetcher = fetcher(backend.clone());
    let mut client = Client::default();
    fetcher.QueueImage("image.png", None).unwrap();
    fetcher.QueueImage("image.png", None).unwrap();
    assert_eq!(backend.borrow().requests.borrow().len(), 1);
    assert_eq!(
        backend.borrow().requests.borrow()[0].url,
        "https://page.test/root/image.png"
    );
    assert_eq!(
        backend.borrow().requests.borrow()[0].referrer,
        "https://page.test/root/entry"
    );
    assert_eq!(
        backend.borrow().requests.borrow()[0].destination,
        RequestDestination::kImage
    );
    assert_eq!(fetcher.PollPendingImages(&mut client), 0);
    assert!(fetcher.HasPendingImages());
    assert!(client.mutations.is_empty());
    assert_eq!(fetcher.PollPendingImages(&mut client), 1);
    assert!(!fetcher.HasPendingImages());
    assert_eq!(client.mutations.len(), 1);
    match &client.mutations[0] {
        page_mutation::ResourceMutation::ImageResourceReady(image) => {
            assert_eq!(image.source, "image.png");
            assert_eq!(image.image.width, 1);
            assert_eq!(
                image.image.BitmapPixels().unwrap().as_slice(),
                [1, 2, 3, 255]
            );
        }
        _ => panic!("unexpected resource mutation"),
    }
    assert_eq!(fetcher.PollPendingImages(&mut client), 0);
    assert_eq!(drops.get(), 1);
}

#[test]
fn lazy_image_does_not_delay_load_and_an_eager_consumer_promotes_it() {
    let (backend, _) = backend(Ok(URLResponse::default()), 10);
    let fetcher = fetcher(backend.clone());
    fetcher
        .QueueImageWithLoadBlocking("shared.png", None, false)
        .unwrap();
    assert!(!fetcher.HasPendingImages());
    assert!(!fetcher.HasPendingLoadBlockingImages());
    assert!(backend.borrow().requests.borrow().is_empty());
    fetcher
        .QueueImageWithLoadBlocking("shared.png", None, true)
        .unwrap();
    assert!(fetcher.HasPendingImages());
    assert!(fetcher.HasPendingLoadBlockingImages());
    assert_eq!(backend.borrow().requests.borrow().len(), 1);
}
struct FailingLoader(Rc<RefCell<Vec<String>>>);
impl URLLoader for FailingLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.0.borrow_mut().push(request.url.clone());
        Err(io::Error::other("unavailable"))
    }
}
#[test]
fn font_fallback_progresses_one_poll_pass_at_a_time_and_keeps_failure_callbacks() {
    let requests = Rc::new(RefCell::new(vec![]));
    let fetcher = fetcher(Rc::new(RefCell::new(FailingLoader(requests.clone()))));
    // Resource transport fixture; stylesheet acceptance belongs to Style.
    let sheet = style::ParseCSS("@font-face { font-family: X; src: url(a.ttf),url(b.ttf); }");
    fetcher
        .QueueFontFace(&sheet.font_faces[0], "https://fonts.test/")
        .unwrap();
    let mut client = Client::default();
    assert!(fetcher.HasPendingFonts());
    assert_eq!(fetcher.PollPendingFonts(&mut client), 1);
    assert!(fetcher.HasPendingFonts());
    assert_eq!(requests.borrow().as_slice(), ["https://fonts.test/a.ttf"]);
    assert_eq!(fetcher.PollPendingFonts(&mut client), 1);
    assert!(!fetcher.HasPendingFonts());
    assert_eq!(
        requests.borrow().as_slice(),
        ["https://fonts.test/a.ttf", "https://fonts.test/b.ttf"]
    );
    assert_eq!(client.mutations.len(), 2);
    for mutation in client.mutations {
        assert!(matches!(
            mutation,
            page_mutation::ResourceMutation::ResourceLoadFailed(_)
        ));
    }
    assert_eq!(fetcher.PollPendingFonts(&mut Client::default()), 0);
}

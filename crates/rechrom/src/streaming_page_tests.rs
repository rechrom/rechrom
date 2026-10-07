use crate::page::{Page, PageClient, ScriptEnvironment};
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
    let assembly = crate::CreateLayoutAssembly();
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
        Rc::new(RefCell::new(
            image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
        )),
        crate::CreateBrowserConstraints(320, 200),
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
    })
}
fn has_id(page: &Page, id: &str) -> bool {
    fn visit(d: &dom::Document, i: usize, id: &str) -> bool {
        d.Node(i).FindAttribute("id").is_some_and(|a| a.value == id)
            || d.Node(i).Children().iter().any(|&c| visit(d, c, id))
    }
    let owner = page.Document();
    visit(owner.GetDocument(), owner.GetDocument().Root(), id)
}
fn finish(page: &mut Page) {
    for _ in 0..1000 {
        page.RunTasks(0.0).unwrap();
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
fn script_response(resources: &Responses, url: &str, source: &str) {
    *resources
        .borrow_mut()
        .entry(url.into())
        .or_default()
        .borrow_mut() = Some(URLResponse {
        final_url: url.into(),
        status_code: 200,
        mime_type: "text/javascript".into(),
        text_encoding: "utf-8".into(),
        body: source.as_bytes().to_vec(),
    });
}
fn compilation_wakes(page: &mut Page) -> std::sync::mpsc::Receiver<()> {
    let (sender, receiver) = std::sync::mpsc::channel();
    page.SetLoadingWakeCallback(std::sync::Arc::new(move || {
        let _ = sender.send(());
    }));
    receiver
}
fn pump_module_until(
    page: &mut Page,
    wakes: &std::sync::mpsc::Receiver<()>,
    mut ready: impl FnMut(&mut Page) -> bool,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        // Preserve real one-task turns while draining already-ready parser and
        // scheduler work. No elapsed-time sleep stands in for worker readiness.
        for _ in 0..40 {
            page.RunTasks(0.0).unwrap();
            if ready(page) {
                return;
            }
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        assert!(!remaining.is_zero(), "module work failed to finish");
        wakes
            .recv_timeout(remaining)
            .expect("real module completion must wake Page");
    }
}
fn script_condition(page: &mut Page, source: &str) -> bool {
    let result = page.Evaluate(source, "probe:module-scheduling").unwrap();
    assert!(result.Succeeded(), "{:?}", result.exception);
    result
        .value
        .Implementation::<javascript::quickjs_javascript_runtime::JsValue>()
        .unwrap()
        .as_boolean()
        .unwrap()
}
#[test]
fn module_graph_waits_yield_frames_and_async_defer_order_matches_browser_rules() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(true);
        let wakes = compilation_wakes(&mut page);
        body.borrow_mut().extend([response(), URLLoadEvent::Data(br#"<body><p id=early>Early</p>
            <script>globalThis.order=[]; document.addEventListener('DOMContentLoaded',()=>order.push('dom'));</script>
            <script type=module>import {x} from './dep.js'; order.push('module:'+x);</script>
            <script defer src='./defer.js'></script>
            <script async type=module>order.push('async');</script>
            <script type=module>order.push('second-inline');</script>"#.to_vec())]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        pump_module_until(&mut page, &wakes, |page| {
            script_condition(
                page,
                "typeof order!=='undefined' && order.includes('async')",
            )
        });
        assert!(page.CurrentFrame().is_some());
        assert!(page.IsLoading());
        assert_eq!(finished.get(), 0);
        assert_script(
            &mut page,
            "if(JSON.stringify(order)!=='[\"async\"]') throw Error(order);",
        );
        script_response(
            &resources,
            "https://stream.test/dep.js",
            "export const x=7;",
        );
        script_response(
            &resources,
            "https://stream.test/defer.js",
            "order.push('defer');",
        );
        for _ in 0..20 {
            page.RunTasks(0.0).unwrap();
        }
        assert_script(
            &mut page,
            "if(JSON.stringify(order)!=='[\"async\"]') throw Error('must wait for HTML end');",
        );
        body.borrow_mut().extend([
            URLLoadEvent::Data(b"<p id=late>Late</p></body>".to_vec()),
            URLLoadEvent::Finished,
        ]);
        finish(&mut page);
        assert!(has_id(&page, "late"));
        assert_eq!(finished.get(), 1);
        assert_script(&mut page, "if(JSON.stringify(order)!=='[\"async\",\"module:7\",\"defer\",\"second-inline\",\"dom\"]') throw Error(order);");
    });
}
#[test]
fn async_module_can_execute_while_css_and_deferred_scripts_are_waiting() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, _) = create(true);
        let wakes = compilation_wakes(&mut page);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                br#"<body>
            <script>globalThis.order=[];</script><link rel=stylesheet href='./slow.css'>
            <script type=module>order.push('deferred');</script>
            <script async type=module>import './async.js'; order.push('async-root');</script>
            </body>"#
                    .to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        for _ in 0..20 {
            page.RunTasks(0.0).unwrap();
        }
        script_response(
            &resources,
            "https://stream.test/async.js",
            "order.push('async-dependency');",
        );
        pump_module_until(&mut page, &wakes, |page| {
            script_condition(page, "order.includes('async-root')")
        });
        assert!(page.IsLoading());
        // Body sheets still delay deferred scripts, but do not block rendering.
        assert!(page.IsRenderingReady());
        assert!(page.CurrentFrame().is_some());
        assert_script(&mut page,"if(JSON.stringify(order)!=='[\"async-dependency\",\"async-root\"]') throw Error(order);");
        *resources.borrow()["https://stream.test/slow.css"].borrow_mut() = Some(URLResponse {
            final_url: "https://stream.test/slow.css".into(),
            mime_type: "text/css".into(),
            body: b"body{color:blue}".to_vec(),
            ..Default::default()
        });
        finish(&mut page);
        assert_script(
            &mut page,
            "if(order.join('|')!=='async-dependency|async-root|deferred') throw Error(order);",
        );
        assert!(page.CurrentFrame().is_some());
    });
}
#[test]
fn failed_dependency_does_not_execute_root_or_stall_later_deferred_scripts() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(true);
        body.borrow_mut().extend([response(),URLLoadEvent::Data(br#"<body>
            <script>globalThis.order=[]; document.addEventListener('DOMContentLoaded',()=>order.push('dom'));</script>
            <script type=module src='./bad-root.js'></script>
            <script type=module>order.push('good');</script></body>"#.to_vec()),URLLoadEvent::Finished]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        script_response(
            &resources,
            "https://stream.test/bad-root.js",
            "import './bad-dependency.js'; order.push('must-not-execute');",
        );
        for _ in 0..10 {
            page.RunTasks(0.0).unwrap();
        }
        assert!(page.IsLoading());
        assert!(page.CurrentFrame().is_some());
        script_response(
            &resources,
            "https://stream.test/bad-dependency.js",
            "export const = syntax_error;",
        );
        finish(&mut page);
        assert_eq!(finished.get(), 1);
        assert_script(
            &mut page,
            "if(order.join('|')!=='good|dom') throw Error(order);",
        );
    });
}
#[test]
fn dynamic_inline_module_after_load_yields_and_uses_captured_base_and_document_url() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, _) = create(true);
        let wakes = compilation_wakes(&mut page);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                b"<base href='https://cdn.test/pkg/'><body><p id=early>Early</p></body>".to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        finish(&mut page);
        assert_script(&mut page, "globalThis.order=[]; let s=document.createElement('script'); s.type='module'; s.textContent=\"import {x} from './dynamic.js'; order.push(x); globalThis.moduleURL=import.meta.url;\"; document.body.appendChild(s);");
        pump_module_until(&mut page, &wakes, |_| {
            resources
                .borrow()
                .contains_key("https://cdn.test/pkg/dynamic.js")
        });
        assert!(page.CurrentFrame().is_some());
        assert!(has_id(&page, "early"));
        assert_script(
            &mut page,
            "if(order.length!==0) throw Error('must await dependency');",
        );
        assert!(resources
            .borrow()
            .contains_key("https://cdn.test/pkg/dynamic.js"));
        script_response(
            &resources,
            "https://cdn.test/pkg/dynamic.js",
            "export const x=9;",
        );
        pump_module_until(&mut page, &wakes, |page| {
            script_condition(page, "order.length>0")
        });
        assert_script(&mut page,"if(order[0]!==9 || moduleURL!=='https://stream.test/page') throw Error('base/source URL');");
    });
}
#[test]
fn stopping_navigation_cancels_unfinished_module_graph_without_execution() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(true);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                b"<body><script type=module src='./main.js'></script></body>".to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        script_response(
            &resources,
            "https://stream.test/main.js",
            "import './pending.js'; globalThis.executed=true;",
        );
        for _ in 0..20 {
            page.RunTasks(0.0).unwrap();
        }
        assert!(page.IsLoading());
        assert!(page.CurrentFrame().is_some());
        page.StopLoading();
        script_response(
            &resources,
            "https://stream.test/pending.js",
            "export const x=1;",
        );
        for _ in 0..10 {
            page.RunTasks(0.0).unwrap();
        }
        assert!(!page.IsLoading());
        assert_eq!(finished.get(), 0);
        assert_script(
            &mut page,
            "if(typeof executed!=='undefined') throw Error('cancelled graph executed');",
        );
    });
}
#[test]
fn page_first_frame_precedes_body_eof_and_resize_keeps_the_live_document() {
    crate::native_test_thread::run(first_frame);
}
fn first_frame() {
    let (mut page, body, _, finished) = create(false);
    page.Open("https://stream.test/", 13, 3).unwrap();
    assert!(page.IsLoading());
    assert!(page.CurrentFrame().is_none());
    page.RunTasks(0.0).unwrap();
    assert!(page.CurrentFrame().is_none());
    body.borrow_mut().extend([
        response(),
        URLLoadEvent::Data(b"<body><p id=early>Early</p>".to_vec()),
    ]);
    for _ in 0..100 {
        page.RunTasks(0.0).unwrap();
        if has_id(&page, "early") && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(has_id(&page, "early"));
    assert!(page.CurrentFrame().is_some());
    assert!(page.IsLoading());
    assert_eq!(finished.get(), 0);
    let root = page.Document().GetDocument().RootHandle();
    let frame = page.CurrentFrame().unwrap().sequence;
    // Waiting on network input must not keep presenting identical frames.
    for _ in 0..100 {
        page.RunTasks(0.0).unwrap();
    }
    let stable = page.CurrentFrame().unwrap().sequence;
    for _ in 0..10 {
        page.RunTasks(0.0).unwrap();
    }
    assert_eq!(page.CurrentFrame().unwrap().sequence, stable);
    assert!(stable >= frame);
    page.ResizeViewport(400.0, 240.0, 1.0).unwrap();
    assert_eq!(page.Document().GetDocument().RootHandle(), root);
    body.borrow_mut().extend([
        URLLoadEvent::Data("<p id=late>世界😀</p></body>".as_bytes().to_vec()),
        URLLoadEvent::Finished,
    ]);
    finish(&mut page);
    assert!(has_id(&page, "late"));
    assert_eq!(finished.get(), 1);
    page.RunTasks(0.0).unwrap();
    assert_eq!(finished.get(), 1);
    assert_eq!(page.Document().GetDocument().RootHandle(), root);
}
#[test]
fn parser_blocking_script_yields_to_page_and_document_write_resumes_in_order() {
    crate::native_test_thread::run(blocking_script);
}
fn blocking_script() {
    let (mut page, body, resources, finished) = create(true);
    body.borrow_mut().extend([
        response(),
        URLLoadEvent::Data(
            b"<body><p id=early>Early</p><script src='slow.js'></script><p id=late>Late</p>"
                .to_vec(),
        ),
        URLLoadEvent::Finished,
    ]);
    page.Open("https://stream.test/", 4096, 4096).unwrap();
    for _ in 0..8 {
        page.RunTasks(0.0).unwrap();
    }
    assert!(has_id(&page, "early"));
    assert!(!has_id(&page, "late"));
    assert!(page.CurrentFrame().is_some());
    assert!(page.IsLoading());
    assert_eq!(finished.get(), 0);
    let root = page.Document().GetDocument().RootHandle();
    assert!(page
        .Evaluate(
            "document.getElementById('early').setAttribute('data-alive','yes')",
            "test:while-waiting"
        )
        .unwrap()
        .Succeeded());
    *resources.borrow().get("https://stream.test/slow.js").unwrap().borrow_mut() = Some(URLResponse {
        body: b"document.write('<b id=written>Written</b>');if(document.getElementById('late'))throw Error('parsed past script')".to_vec(), ..Default::default()
    });
    finish(&mut page);
    assert!(has_id(&page, "written"));
    assert!(has_id(&page, "late"));
    assert_eq!(finished.get(), 1);
    assert_eq!(page.Document().GetDocument().RootHandle(), root);
}
#[test]
fn stylesheet_completion_unblocks_rendering_without_new_main_body_data() {
    crate::native_test_thread::run(blocking_style);
}
fn blocking_style() {
    let (mut page, body, resources, finished) = create(false);
    body.borrow_mut().extend([
        response(),
        URLLoadEvent::Data(
            b"<head><link rel=stylesheet href=slow.css></head><body><p id=early>Early</p>".to_vec(),
        ),
        URLLoadEvent::Finished,
    ]);
    page.Open("https://stream.test/", 4096, 4096).unwrap();
    for _ in 0..8 {
        page.RunTasks(0.0).unwrap();
    }
    assert!(has_id(&page, "early"));
    assert!(!page.IsRenderingReady());
    assert!(page.CurrentFrame().is_none());
    assert_eq!(finished.get(), 0);
    *resources
        .borrow()
        .get("https://stream.test/slow.css")
        .unwrap()
        .borrow_mut() = Some(URLResponse {
        body: b"p { color: green }".to_vec(),
        ..Default::default()
    });
    finish(&mut page);
    assert!(page.IsRenderingReady());
    assert!(page.CurrentFrame().is_some());
    assert_eq!(finished.get(), 1);
}

#[test]
fn post_body_stylesheets_do_not_freeze_typing_or_caret() {
    crate::native_test_thread::run(|| {
        for scripting in [false, true] {
            let (mut page, body, resources, finished) = create(scripting);
            body.borrow_mut().extend([
                response(),
                URLLoadEvent::Data(b"<body><textarea id=editor></textarea>".to_vec()),
            ]);
            page.Open("https://stream.test/", 4096, 4096).unwrap();
            for _ in 0..8 {
                page.RunTasks(0.0).unwrap();
            }
            let first = page.CurrentFrame().unwrap().sequence;
            body.borrow_mut().extend([
                URLLoadEvent::Data(b"<link rel=stylesheet href=late.css></body>".to_vec()),
                URLLoadEvent::Finished,
            ]);
            for _ in 0..8 {
                page.RunTasks(0.0).unwrap();
            }
            assert!(resources
                .borrow()
                .contains_key("https://stream.test/late.css"));
            assert!(page.IsLoading());
            assert!(page.IsRenderingReady());
            assert_eq!(finished.get(), 0);
            fn editor(d: &dom::Document, i: usize) -> Option<u64> {
                if d.Node(i).IsHTMLElement("textarea") {
                    return Some(d.Node(i).Id());
                }
                d.Node(i).Children().iter().find_map(|&c| editor(d, c))
            }
            let id = {
                let owner = page.Document();
                editor(owner.GetDocument(), owner.GetDocument().Root()).unwrap()
            };
            use interaction::input_event::*;
            page.Dispatch(&InputEvent::Mouse(MouseEvent {
                r#type: MouseEventType::kDown,
                button: MouseButton::kPrimary,
                target_node_id: Some(id),
                ..Default::default()
            }))
            .unwrap();
            page.Dispatch(&InputEvent::TextInput(TextInputEvent {
                text: "hello".into(),
                ..Default::default()
            }))
            .unwrap();
            assert!(page.CurrentFrame().unwrap().sequence > first);
            assert!(page.Caret().is_some_and(|caret| caret.visible));
            {
                let owner = page.Document();
                let d = owner.GetDocument();
                assert_eq!(d.ControlValue(d.FindNodeById(id).unwrap()), "hello");
            }
            *resources
                .borrow()
                .get("https://stream.test/late.css")
                .unwrap()
                .borrow_mut() = Some(URLResponse {
                body: b"textarea { color: green }".to_vec(),
                ..Default::default()
            });
            finish(&mut page);
            assert_eq!(finished.get(), 1);
            assert!(page.Caret().is_some());
        }
    });
}

#[test]
fn real_http_first_frame_is_visible_while_server_holds_the_response_open() {
    crate::native_test_thread::run(real_http);
}
fn real_http() {
    use std::{
        io::{BufRead, Write},
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc, Arc,
        },
        time::{Duration, Instant},
    };
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}/", listener.local_addr().unwrap());
    let (release, gate) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut reader = std::io::BufReader::new(socket.try_clone().unwrap());
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
        }
        let first = "<body><p id=early>Early frame</p>";
        let last = "<p id=late>Final frame</p></body>";
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{first}", first.len() + last.len()).unwrap();
        gate.recv_timeout(Duration::from_secs(5)).unwrap();
        socket.write_all(last.as_bytes()).unwrap();
    });
    let options = url_loader::DefaultURLLoaderOptions {
        max_retries: 0,
        ..Default::default()
    };
    let assembly = crate::CreateLayoutAssembly();
    let mut page = Page::Create(
        Rc::new(RefCell::new(
            url_loader::DefaultURLLoader::new(options).unwrap(),
        )),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(
            image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
        )),
        crate::CreateBrowserConstraints(320, 200),
        None,
        None,
    );
    let wakes = Arc::new(AtomicUsize::new(0));
    let count = wakes.clone();
    page.SetLoadingWakeCallback(Arc::new(move || {
        count.fetch_add(1, Ordering::Relaxed);
    }));
    let start = Instant::now();
    page.Open(&address, 16384, 4096).unwrap();
    let deadline = start + Duration::from_secs(5);
    while page.CurrentFrame().is_none() || !has_id(&page, "early") {
        page.RunTasks(0.0).unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    eprintln!(
        "streaming-first-frame-before-eof debug_ms={:.3}",
        start.elapsed().as_secs_f64() * 1000.0
    );
    assert!(page.IsLoading());
    assert!(!has_id(&page, "late"));
    assert!(wakes.load(Ordering::Relaxed) > 0);
    release.send(()).unwrap();
    while page.IsLoading() {
        page.RunTasks(0.0).unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(has_id(&page, "late"));
    server.join().unwrap();
}

#[test]
fn real_http_module_dependency_wait_keeps_page_frames_and_tasks_running() {
    crate::native_test_thread::run(|| {
        use std::{
            io::{BufRead, BufReader, Write},
            net::TcpListener,
            sync::{
                atomic::{AtomicBool, Ordering},
                mpsc, Arc,
            },
            time::{Duration, Instant},
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}/page", listener.local_addr().unwrap());
        let (release, gate) = mpsc::channel();
        let waiting = Arc::new(AtomicBool::new(false));
        let server_waiting = waiting.clone();
        let server = std::thread::spawn(move || {
            for expected in ["/page", "/root.js", "/leaf.js"] {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                let mut request = String::new();
                reader.read_line(&mut request).unwrap();
                assert_eq!(request.split_whitespace().nth(1).unwrap(), expected);
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                }
                let (mime,source)=match expected {
                    "/page" => ("text/html", "<body><p id=early>Early frame</p><script>globalThis.ticks=0;</script><script async type=module src='/root.js'></script></body>"),
                    "/root.js" => ("text/javascript", "import {x} from './leaf.js'; globalThis.moduleAnswer=x;"),
                    _ => ("text/javascript", "export const x=42;"),
                };
                write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: {mime}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",source.len()).unwrap();
                if expected == "/leaf.js" {
                    server_waiting.store(true, Ordering::Release);
                    gate.recv_timeout(Duration::from_secs(5)).unwrap();
                }
                socket.write_all(source.as_bytes()).unwrap();
            }
        });
        let assembly = crate::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(
                url_loader::DefaultURLLoader::new(url_loader::DefaultURLLoaderOptions {
                    max_retries: 0,
                    ..Default::default()
                })
                .unwrap(),
            )),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(
                image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
            )),
            crate::CreateBrowserConstraints(320, 200),
            Some(ScriptEnvironment {
                runtime: Box::new(
                    javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::new(),
                ),
                xhr: xhr_transport::CreateHTTPXMLHttpRequestTransport(&Default::default()).unwrap(),
                user_agent: "module-stream-test".into(),
            }),
            None,
        );
        page.Open(&address, 16384, 4096).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !waiting.load(Ordering::Acquire) || page.CurrentFrame().is_none() {
            page.RunTasks(0.0).unwrap();
            assert!(
                Instant::now() < deadline,
                "dependency fetch failed to start"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(page.IsLoading());
        assert!(has_id(&page, "early"));
        assert_script(&mut page, "setTimeout(()=>ticks++,0);");
        page.RunTasks(1.0).unwrap();
        assert_script(&mut page,"if(ticks!==1 || typeof moduleAnswer!=='undefined') throw Error('page task blocked by module fetch');");
        page.ResizeViewport(360.0, 220.0, 1.0).unwrap();
        assert!(page.CurrentFrame().is_some());
        eprintln!(
            "real HTTP: frame, timer and resize completed with module dependency body withheld"
        );
        release.send(()).unwrap();
        while page.IsLoading() {
            page.RunTasks(0.0).unwrap();
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_script(
            &mut page,
            "if(moduleAnswer!==42) throw Error('dependency did not execute');",
        );
        server.join().unwrap();
    });
}

#[test]
fn resource_completion_after_load_is_polled_without_another_dom_edit() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(true);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(b"<body><p>Ready</p>".to_vec()),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        finish(&mut page);
        assert_eq!(finished.get(), 1);
        assert_script(&mut page, "globalThis.imageLoads=0;var later=new Image();later.id='later';later.onload=()=>imageLoads++;later.src='later.svg';document.body.appendChild(later);");
        // The creation frame consumes its dirty state while the transport is
        // still pending. Completion must be driven by a later ordinary turn.
        for _ in 0..3 {
            page.RunTasks(0.0).unwrap();
        }
        *resources.borrow()["https://stream.test/later.svg"].borrow_mut() = Some(URLResponse {
            final_url: "https://stream.test/later.svg".into(),
            status_code: 200,
            mime_type: "image/svg+xml".into(),
            text_encoding: "utf-8".into(),
            body: br#"<svg xmlns="http://www.w3.org/2000/svg" width="7" height="11"><rect width="7" height="11" fill="red"/></svg>"#.to_vec(),
        });
        for _ in 0..5 {
            page.RunTasks(0.0).unwrap();
        }
        assert_script(&mut page, "if(imageLoads!==1||!later.complete||later.naturalWidth!==7||later.naturalHeight!==11)throw Error('late resource did not complete');if(document.readyState!=='complete')throw Error('load restarted');");
        assert!(!page.IsLoading());
        assert_eq!(finished.get(), 1);
    });
}

#[test]
fn nonblocking_page_turn_returns_between_ready_scripts_and_timer() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(true);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                br#"<body><script>
                var order=[],turnRuns=0;
                document.addEventListener('DOMContentLoaded',()=>order.push('dom'));
                window.addEventListener('load',()=>order.push('load'));
                </script><script async src='a.js'></script><script async src='b.js'></script>
                <script defer src='c.js'></script><p>End</p>"#
                    .to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        for _ in 0..20 {
            page.RunTasks(0.0).unwrap();
        }
        assert!(page.IsLoading());
        assert_eq!(finished.get(), 0);
        for name in ["a", "b", "c"] {
            script_response(&resources, &format!("https://stream.test/{name}.js"),
                &format!("turnRuns++;order.push('{name}');Promise.resolve().then(()=>order.push('m{name}'));"));
        }
        assert_script(
            &mut page,
            "setTimeout(()=>{turnRuns++;order.push('timer')},0);",
        );
        for _ in 0..12 {
            assert_script(&mut page, "turnRuns=0;");
            page.RunTasks(0.0).unwrap();
            assert_script(&mut page, "if(turnRuns>1)throw Error('multiple script jobs in one host turn: '+turnRuns);for(var n of ['a','b','c']){var i=order.indexOf(n);if(i>=0&&order[i+1]!=='m'+n)throw Error('microtask checkpoint split')}");
        }
        assert_script(&mut page, "if(order.filter(x=>['a','b','c','timer'].includes(x)).length!==4)throw Error('script lost or duplicated');if(order.indexOf('c')>order.indexOf('dom')||order.indexOf('dom')>order.indexOf('load'))throw Error('load ordering');if(document.readyState!=='complete')throw Error('readyState');");
        assert!(!page.IsLoading());
        assert_eq!(finished.get(), 1);
    });
}

#[test]
fn resource_turn_budget_is_shared_by_loading_lifecycle_and_no_script_page() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(false);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(br#"<head><style>@font-face{font-family:StreamFont;src:url(font.ttf)}body{font-family:StreamFont}</style></head><body>Font<img src='a.svg'><img src='b.svg'>"#.to_vec()),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        for _ in 0..20 {
            page.RunTasks(0.0).unwrap();
        }
        assert!(page.CurrentFrame().is_some());
        assert!(page.IsLoading());
        for name in ["a.svg", "b.svg", "font.ttf"] {
            let font = name.ends_with("ttf");
            *resources.borrow()[&format!("https://stream.test/{name}")].borrow_mut() = Some(
                URLResponse {
                    final_url: format!("https://stream.test/{name}"),
                    status_code: 200,
                    mime_type: if font { "font/ttf" } else { "image/svg+xml" }.into(),
                    text_encoding: "utf-8".into(),
                    body: if font {
                        include_bytes!(
                            "../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
                        )
                        .to_vec()
                    } else {
                        br#"<svg xmlns="http://www.w3.org/2000/svg" width="7" height="11"><rect width="7" height="11"/></svg>"#.to_vec()
                    },
                },
            );
        }
        for remaining in [2, 1, 0] {
            page.RunTasks(0.0).unwrap();
            let waiting = resources
                .borrow()
                .values()
                .filter(|response| response.borrow().is_some())
                .count();
            assert_eq!(
                waiting, remaining,
                "loading and dirty lifecycle must share one resource completion allowance"
            );
            assert_eq!(
                finished.get(),
                0,
                "document load follows resource completion on a later turn"
            );
        }
        finish(&mut page);
        assert_eq!(finished.get(), 1);
        assert!(page
            .Document()
            .GetDocument()
            .ImageResourceFor("a.svg")
            .is_some());
        assert!(page
            .Document()
            .GetDocument()
            .ImageResourceFor("b.svg")
            .is_some());
    });
}

#[test]
fn image_completion_and_timer_have_separate_turns_with_complete_checkpoints() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(true);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(br#"<body><script>
                var order=[],turnRuns=0;
                window.addEventListener('readystatechange',()=>{if(document.readyState==='complete'){turnRuns++;order.push('complete');Promise.resolve().then(()=>order.push('mcomplete'))}},true);
                window.addEventListener('load',()=>order.push('load'));
                </script><img id=first src='first.svg'>"#.to_vec()),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        for _ in 0..20 {
            page.RunTasks(0.0).unwrap();
        }
        assert_script(&mut page, "document.getElementById('first').onload=()=>{turnRuns++;order.push('image');Promise.resolve().then(()=>order.push('mimage'))};setTimeout(()=>{turnRuns++;order.push('timer');Promise.resolve().then(()=>order.push('mtimer'))},0);");
        *resources.borrow()["https://stream.test/first.svg"].borrow_mut() = Some(URLResponse {
            final_url: "https://stream.test/first.svg".into(), status_code: 200,
            mime_type: "image/svg+xml".into(), text_encoding: "utf-8".into(),
            body: br#"<svg xmlns="http://www.w3.org/2000/svg" width="7" height="11"><rect width="7" height="11" fill="red"/></svg>"#.to_vec(),
        });
        for _ in 0..5 {
            assert_script(&mut page, "turnRuns=0;");
            page.RunTasks(0.0).unwrap();
            assert_script(&mut page, "if(turnRuns>1)throw Error('resource, load or timer jobs merged');for(var n of ['image','timer','complete']){var i=order.indexOf(n);if(i>=0&&order[i+1]!=='m'+n)throw Error('unfinished checkpoint')}");
        }
        assert_script(&mut page, "if(order.indexOf('image')<0||order.indexOf('timer')<0||order.indexOf('complete')>order.indexOf('load')||order.indexOf('mcomplete')>order.indexOf('load'))throw Error('completion/lifecycle order: '+JSON.stringify(order));");
        assert!(!page.IsLoading());
        assert_eq!(finished.get(), 1);
    });
}

// Real streamed resource completion exports fresh paint catalogs while retaining
// the immutable decoded pixel allocation across ordinary Page lifecycles.
#[test]
fn delayed_image_snapshots_share_pixels_and_match_fresh_page() {
    crate::native_test_thread::run(|| {
        let html = b"<body style='margin:0'><img id=photo src='photo.svg' style='width:40px;height:20px'><p>after</p>";
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="7" height="11"><rect width="7" height="11" fill="red"/></svg>"#;
        let (mut page, body, resources, finished) = create(false);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(html.to_vec()),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        for _ in 0..5 {
            page.RunTasks(0.0).unwrap();
        }
        assert!(page.CurrentFrame().is_some());
        assert!(page.IsLoading());
        assert_eq!(finished.get(), 0);
        *resources.borrow()["https://stream.test/photo.svg"].borrow_mut() = Some(URLResponse {
            final_url: "https://stream.test/photo.svg".into(),
            status_code: 200,
            mime_type: "image/svg+xml".into(),
            text_encoding: "utf-8".into(),
            body: svg.to_vec(),
        });
        finish(&mut page);
        let first = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .resources
            .as_ref()
            .unwrap()
            .clone();
        let bytes: usize = first.images.iter().map(|image| image.rgba8.len()).sum();
        assert_eq!(first.images.len(), 1);
        assert_eq!(bytes, 7 * 11 * 4);
        eprintln!("fixture-resource-bytes source_rgba_bytes={bytes} legacy_copy_sites_per_full_lifecycle=2 legacy_deep_copy_payload_bytes={} basis=static_layout_environment_and_export_callchain", 2*bytes);
        // A real viewport change forces another ordinary Page lifecycle. This
        // is not old-frame reuse and the pixels remain complete and immutable.
        page.ResizeViewport(321.0, 200.0, 1.0).unwrap();
        page.RunTasks(0.0).unwrap();
        let second = page
            .CurrentFrame()
            .unwrap()
            .display_items
            .resources
            .as_ref()
            .unwrap();
        assert!(
            !std::sync::Arc::ptr_eq(&first, second),
            "new lifecycle exports a new resource catalog"
        );
        assert_eq!(
            first.images[0].rgba8.as_ptr(),
            second.images[0].rgba8.as_ptr(),
            "catalogs share the admitted image pixel allocation"
        );
        assert_eq!(
            first.images[0].rgba8.as_slice(),
            second.images[0].rgba8.as_slice()
        );
        let current_pixels = renderer::pure_replay::RasterizeDisplayItemList(
            &page.CurrentFrame().unwrap().display_items,
            321,
            200,
        );
        let (mut fresh, fresh_body, fresh_resources, _) = create(false);
        fresh.ResizeViewport(321.0, 200.0, 1.0).unwrap();
        fresh_body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(html.to_vec()),
            URLLoadEvent::Finished,
        ]);
        *fresh_resources
            .borrow_mut()
            .entry("https://stream.test/photo.svg".into())
            .or_default()
            .borrow_mut() = Some(URLResponse {
            final_url: "https://stream.test/photo.svg".into(),
            status_code: 200,
            mime_type: "image/svg+xml".into(),
            text_encoding: "utf-8".into(),
            body: svg.to_vec(),
        });
        fresh.Open("https://stream.test/", 4096, 4096).unwrap();
        finish(&mut fresh);
        let fresh_pixels = renderer::pure_replay::RasterizeDisplayItemList(
            &fresh.CurrentFrame().unwrap().display_items,
            321,
            200,
        );
        assert_eq!(
            current_pixels, fresh_pixels,
            "shared storage renders the same complete final pixels as a fresh Page"
        );
        assert_eq!(finished.get(), 1);
    });
}

// A resource result is not itself the load/error event task. Check observable
// turn/checkpoint ordering and cancellation across real cached request changes.
#[test]
fn image_dom_tasks_defer_load_error_and_cancel_replaced_requests() {
    crate::native_test_thread::run(|| {
        let (mut page, body, resources, finished) = create(true);
        body.borrow_mut().extend([
            response(),
            URLLoadEvent::Data(
                br#"<body><script>
                var order=[];
                var windowCaptureLoads=0,documentCaptureLoads=0;
                window.addEventListener('load',()=>windowCaptureLoads++,true);
                document.addEventListener('load',()=>documentCaptureLoads++,true);
                window.onload=()=>order.push('window');
                </script><img id=good src='good.svg'><img id=bad src='bad.png'>"#
                    .to_vec(),
            ),
            URLLoadEvent::Finished,
        ]);
        page.Open("https://stream.test/", 4096, 4096).unwrap();
        for _ in 0..20 {
            page.RunTasks(0.0).unwrap();
        }
        assert_script(&mut page, "var good=document.getElementById('good'),bad=document.getElementById('bad');good.onload=()=>{order.push('good');Promise.resolve().then(()=>order.push('mgood'))};bad.onerror=()=>{order.push('bad');Promise.resolve().then(()=>order.push('mbad'))};");
        *resources.borrow()["https://stream.test/good.svg"].borrow_mut() = Some(URLResponse {
            final_url: "https://stream.test/good.svg".into(), status_code: 200,
            mime_type: "image/svg+xml".into(), text_encoding: "utf-8".into(),
            body: br#"<svg xmlns="http://www.w3.org/2000/svg" width="7" height="11"><rect width="7" height="11" fill="red"/></svg>"#.to_vec(),
        });
        page.RunTasks(0.0).unwrap();
        assert_script(
            &mut page,
            "if(order.length)throw Error('load fired during resource commit');",
        );
        assert_eq!(finished.get(), 0);
        page.RunTasks(0.0).unwrap();
        assert_script(
            &mut page,
            "if(order.join(',')!=='good,mgood')throw Error('load/checkpoint '+order);",
        );
        assert_script(&mut page, "if(windowCaptureLoads!==0||documentCaptureLoads!==1)throw Error('image load crossed Document to Window');");
        *resources.borrow()["https://stream.test/bad.png"].borrow_mut() = Some(URLResponse {
            final_url: "https://stream.test/bad.png".into(),
            status_code: 200,
            mime_type: "image/png".into(),
            text_encoding: "utf-8".into(),
            body: b"invalid image".to_vec(),
        });
        page.RunTasks(0.0).unwrap();
        assert_script(
            &mut page,
            "if(order.join(',')!=='good,mgood')throw Error('error fired during resource commit');",
        );
        assert_eq!(finished.get(), 0);
        page.RunTasks(0.0).unwrap();
        assert_script(
            &mut page,
            "if(order.join(',')!=='good,mgood,bad,mbad')throw Error('error/checkpoint '+order);",
        );
        finish(&mut page);
        assert_script(&mut page, "if(order.join(',')!=='good,mgood,bad,mbad,window')throw Error('document load order '+order);order=[];var changed=document.createElement('img');changed.onload=()=>{order.push('changed');Promise.resolve().then(()=>order.push('mchanged'))};changed.onerror=()=>order.push('stale-error');changed.src='good.svg';document.body.appendChild(changed);changed.src='other.svg';changed.src='good.svg';document.body.style.color='red';if(order.length)throw Error('cached load must be deferred');");
        for _ in 0..4 {
            page.RunTasks(0.0).unwrap();
        }
        assert_script(&mut page, "if(order.join(',')!=='changed,mchanged')throw Error('stale or duplicated request event '+order);");
        assert_script(&mut page, "order=[];changed.setAttribute('class','loaded');changed.style.opacity='.9';changed.setAttribute('alt','loaded image');changed.setAttribute('data-ready','1');");
        for _ in 0..4 {
            page.RunTasks(0.0).unwrap();
        }
        assert_script(
            &mut page,
            "if(order.length)throw Error('non-request attributes refired image load '+order);",
        );
        assert_script(&mut page, "if(windowCaptureLoads!==1||documentCaptureLoads!==2)throw Error('native window/image load routing');good.dispatchEvent(new Event('load',{bubbles:true}));if(windowCaptureLoads!==1||documentCaptureLoads!==3)throw Error('synthetic node load crossed to Window');window.dispatchEvent(new Event('load'));if(windowCaptureLoads!==2)throw Error('direct Window load suppressed');");
        assert_eq!(finished.get(), 1);
    });
}

// The old C++ extraction's cached-image fixture required callbacks inside
// appendChild. Replace that assertion with Blink's observable DOM task contract,
// including the public Page::Apply mutation route.
pub(crate) fn check_cached_image_external_apply_dom_tasks() {
    let (mut page, body, resources, _) = create(true);
    body.borrow_mut().extend([
        response(),
        URLLoadEvent::Data(b"<body><img id=seed src='good.svg'>".to_vec()),
        URLLoadEvent::Finished,
    ]);
    page.Open("https://stream.test/", 4096, 4096).unwrap();
    for _ in 0..20 {
        page.RunTasks(0.0).unwrap();
    }
    *resources.borrow()["https://stream.test/good.svg"].borrow_mut() = Some(URLResponse {
        final_url: "https://stream.test/good.svg".into(), status_code: 200,
        mime_type: "image/svg+xml".into(), text_encoding: "utf-8".into(),
        body: br#"<svg xmlns="http://www.w3.org/2000/svg" width="7" height="11"><rect width="7" height="11"/></svg>"#.to_vec(),
    });
    finish(&mut page);
    assert_script(&mut page, "var order=[];var host=document.createElement('img');host.onload=()=>{order.push('host');Promise.resolve().then(()=>order.push('mhost'))};host.src='good.svg';document.body.appendChild(host);order.push('returned');if(order.join(',')!=='returned')throw Error('host mutation fired load synchronously');");
    page.RunTasks(0.0).unwrap();
    assert_script(&mut page, "if(order.join(',')!=='returned,host,mhost')throw Error('host task/checkpoint '+order);order=[];");
    let (document, body, id) = {
        let owner = page.Document();
        let tree = owner.GetDocument();
        fn body_node(tree: &dom::Document, i: usize) -> Option<usize> {
            if tree.Node(i).IsHTMLElement("body") {
                return Some(i);
            }
            tree.Node(i)
                .Children()
                .iter()
                .find_map(|&child| body_node(tree, child))
        }
        let body = body_node(tree, tree.Root()).unwrap();
        (
            tree.Node(tree.Root()).Id(),
            tree.Node(body).Id(),
            tree.NextNodeId(),
        )
    };
    use dom::dom_mutation::{DOMMutation, DOMMutationType};
    use page_mutation::PageMutation;
    for mutation in [
        DOMMutation {
            mutation_type: DOMMutationType::kCreateElement,
            target_node_id: document,
            name: "img".into(),
            child_node_id: id,
            ..Default::default()
        },
        DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: id,
            name: "id".into(),
            value: "external".into(),
            ..Default::default()
        },
        DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: id,
            name: "src".into(),
            value: "good.svg".into(),
            ..Default::default()
        },
    ] {
        page.Apply(PageMutation::DOMMutation(mutation)).unwrap();
    }
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kAppendChild,
        target_node_id: body,
        child_node_id: id,
        ..Default::default()
    }))
    .unwrap();
    // getElementById only finds connected elements. The queued load task has
    // not started, so its listener can be installed after the public mutation.
    assert_script(&mut page, "if(order.length)throw Error('Page::Apply fired load synchronously');document.getElementById('external').onload=()=>{order.push('external');Promise.resolve().then(()=>order.push('mexternal'))};");
    page.RunTasks(0.0).unwrap();
    assert_script(
        &mut page,
        "if(order.join(',')!=='external,mexternal')throw Error('external task/checkpoint '+order);",
    );
}

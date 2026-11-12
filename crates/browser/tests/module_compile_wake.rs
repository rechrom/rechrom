use browser::page::{Page, PageThreads, ScriptEnvironment};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    io,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};
use url_loader::{
    URLLoadEvent, URLLoadOperation, URLLoader, URLRequest, URLResponse, URLResponseHead,
    URLStreamOperation,
};

struct Body(VecDeque<URLLoadEvent>);
impl URLStreamOperation for Body {
    fn Poll(&mut self, _budget: usize) -> io::Result<Option<URLLoadEvent>> {
        Ok(self.0.pop_front())
    }
}

struct UnusedResource;
impl URLLoadOperation for UnusedResource {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(None)
    }
}

struct ReadyResource(Option<URLResponse>);
impl URLLoadOperation for ReadyResource {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader {
    body: Option<Body>,
    modules: HashMap<String, URLResponse>,
}
impl URLLoader for Loader {
    fn LoadStream(&mut self, _: &URLRequest) -> io::Result<Box<dyn URLStreamOperation>> {
        Ok(Box::new(self.body.take().expect("one document request")))
    }

    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(match self.modules.remove(&request.url) {
            Some(response) => Box::new(ReadyResource(Some(response))),
            None => Box::new(UnusedResource),
        })
    }
}

#[derive(Default)]
struct QueuedScriptCompile(Mutex<Vec<executor::Task>>);
impl executor::Executor for QueuedScriptCompile {
    fn execute(&self, task: executor::Task) -> io::Result<()> {
        self.0.lock().unwrap().push(task);
        Ok(())
    }

    fn execute_after(&self, _delay: Duration, task: executor::Task) -> io::Result<()> {
        self.execute(task)
    }
}

#[test]
fn background_module_compile_completion_wakes_the_page_owner() {
    let body = Body(VecDeque::from([
        URLLoadEvent::Response(URLResponseHead {
            final_url: "https://stream.test/page".into(),
            mime_type: "text/html".into(),
            text_encoding: "utf-8".into(),
            status_code: 200,
            ..Default::default()
        }),
        URLLoadEvent::Data(
            b"<body><script type=module>globalThis.moduleDone=true;</script></body>".to_vec(),
        ),
        URLLoadEvent::Finished,
    ]));
    let assembly = browser::CreateLayoutAssembly();
    let scripts = ScriptEnvironment {
        runtime: Box::new(javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::new()),
        xhr: xhr_transport::CreateHTTPXMLHttpRequestTransport(&Default::default()).unwrap(),
        user_agent: "module-wake-test".into(),
    };
    let compile = Arc::new(QueuedScriptCompile::default());
    let mut threads = PageThreads::default();
    threads.scriptCompile = true;
    threads.script_compile_executor = compile.clone();
    let mut page = Page::CreateWithThreads(
        Rc::new(RefCell::new(Loader {
            body: Some(body),
            modules: HashMap::new(),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        browser::CreateBrowserConstraints(320, 200),
        Some(scripts),
        None,
        threads,
    );
    let (send, wakes) = mpsc::channel();
    page.SetLoadingWakeCallback(Arc::new(move || {
        let _ = send.send(());
    }));
    page.Open("https://stream.test/", 4096, 4096).unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !compile.0.lock().unwrap().is_empty() {
            break;
        }
    }
    let task = compile
        .0
        .lock()
        .unwrap()
        .pop()
        .expect("module compilation must be posted to the worker executor");

    // Discard parser/resource wakes. The worker completion below is now the
    // only event that can make its prepared module visible to the Page owner.
    while wakes.try_recv().is_ok() {}
    task();
    wakes
        .recv_timeout(Duration::from_secs(2))
        .expect("background module completion must post an owner task");

    // A worker wake only makes its completion observable. The graph-ready
    // transition and the later ScriptRunner execution are distinct owner tasks;
    // every phase must post the task that follows it.
    page.RunTask().unwrap();
    wakes
        .recv_timeout(Duration::from_secs(2))
        .expect("ready module root must post its ScriptRunner owner task");

    for _ in 0..4 {
        page.RunTask().unwrap();
        let result = page
            .Evaluate(
                "if(globalThis.moduleDone===true) true; else throw Error('pending')",
                "assert:module-wake",
            )
            .unwrap();
        if result.Succeeded() {
            return;
        }
        wakes
            .recv_timeout(Duration::from_secs(2))
            .expect("pending root lost its next owner task");
    }
    panic!("owner wakes did not execute module root");
}

#[test]
fn eight_deferred_module_roots_with_dependencies_drain_only_from_owner_wakes() {
    let mut html = String::from("<body>");
    let mut modules = HashMap::new();
    for root in 0..8 {
        html.push_str(&format!(
            "<script type=module>import './d{root}a.js';import './d{root}b.js';\
             if(!globalThis.moduleOrder)globalThis.moduleOrder=[];\
             moduleOrder.push({root});</script>"
        ));
        for suffix in ['a', 'b'] {
            let url = format!("https://stream.test/d{root}{suffix}.js");
            modules.insert(
                url.clone(),
                URLResponse {
                    final_url: url,
                    body: format!("export const value='{root}{suffix}';").into_bytes(),
                    mime_type: "text/javascript".into(),
                    status_code: 200,
                    ..Default::default()
                },
            );
        }
    }
    html.push_str("</body>");
    let body = Body(VecDeque::from([
        URLLoadEvent::Response(URLResponseHead {
            final_url: "https://stream.test/page".into(),
            mime_type: "text/html".into(),
            text_encoding: "utf-8".into(),
            status_code: 200,
            ..Default::default()
        }),
        URLLoadEvent::Data(html.into_bytes()),
        URLLoadEvent::Finished,
    ]));
    let assembly = browser::CreateLayoutAssembly();
    let scripts = ScriptEnvironment {
        runtime: Box::new(javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::new()),
        xhr: xhr_transport::CreateHTTPXMLHttpRequestTransport(&Default::default()).unwrap(),
        user_agent: "module-wake-test".into(),
    };
    let compile = Arc::new(QueuedScriptCompile::default());
    let mut threads = PageThreads::default();
    threads.scriptCompile = true;
    threads.script_compile_executor = compile.clone();
    let mut page = Page::CreateWithThreads(
        Rc::new(RefCell::new(Loader {
            body: Some(body),
            modules,
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        browser::CreateBrowserConstraints(320, 200),
        Some(scripts),
        None,
        threads,
    );
    let (send, wakes) = mpsc::channel();
    let wake_pending = Arc::new(AtomicBool::new(false));
    let pending = wake_pending.clone();
    page.SetLoadingWakeCallback(Arc::new(move || {
        if !pending.swap(true, Ordering::AcqRel) {
            let _ = send.send(());
        }
    }));
    page.Open("https://stream.test/", 65536, 65536).unwrap();

    // Bootstrap the in-memory main response. Once all eight root compilation
    // jobs exist, every further Page turn must have a corresponding owner wake.
    for _ in 0..100 {
        page.RunTask().unwrap();
        if compile.0.lock().unwrap().len() == 8 {
            break;
        }
    }
    assert_eq!(compile.0.lock().unwrap().len(), 8);
    while wakes.try_recv().is_ok() {}
    wake_pending.store(false, Ordering::Release);

    for _ in 0..1000 {
        let tasks = std::mem::take(&mut *compile.0.lock().unwrap());
        for task in tasks {
            task();
        }
        if !page.IsLoading() {
            break;
        }
        wakes
            .recv_timeout(Duration::from_secs(2))
            .expect("pending module/deferred work lost its owner wake");
        wake_pending.store(false, Ordering::Release);
        page.RunTask().unwrap();
    }
    assert!(
        !page.IsLoading(),
        "deferred module roots stalled before load"
    );
    let result = page
        .Evaluate(
            "if(moduleOrder.join(',')==='0,1,2,3,4,5,6,7') true; \
             else throw Error(String(moduleOrder))",
            "assert:eight-module-roots",
        )
        .unwrap();
    assert!(
        result.Succeeded(),
        "{:?}",
        result
            .exception
            .as_ref()
            .map(|exception| &exception.message)
    );
}

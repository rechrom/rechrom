use super::*;
use javascript::{javascript_runtime::*, quickjs_javascript_runtime::QuickJsJavaScriptRuntime};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    rc::Rc,
};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
type Replies = Rc<RefCell<HashMap<String, Rc<RefCell<Option<io::Result<URLResponse>>>>>>>;
struct Operation {
    reply: Rc<RefCell<Option<io::Result<URLResponse>>>>,
    drops: Rc<Cell<usize>>,
}
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        self.reply.borrow_mut().take().transpose()
    }
}
impl Drop for Operation {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}
struct Backend {
    replies: Replies,
    requests: Rc<RefCell<Vec<URLRequest>>>,
    drops: Rc<Cell<usize>>,
}
impl URLLoader for Backend {
    fn Load(&mut self, r: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.requests.borrow_mut().push(r.clone());
        Ok(Box::new(Operation {
            reply: self
                .replies
                .borrow_mut()
                .entry(r.url.clone())
                .or_default()
                .clone(),
            drops: self.drops.clone(),
        }))
    }
}
struct Host;
impl JavaScriptHostBindings for Host {
    fn GlobalNames(&self) -> Vec<String> {
        vec![]
    }
    fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
        HostResult {
            handled: false,
            ..Default::default()
        }
    }
}
fn setup() -> (
    Rc<ModuleResources>,
    QuickJsJavaScriptRuntime,
    JavaScriptRealm,
    Replies,
    Rc<RefCell<Vec<URLRequest>>>,
    Rc<Cell<usize>>,
) {
    let replies = Rc::new(RefCell::new(HashMap::new()));
    let requests = Rc::new(RefCell::new(vec![]));
    let drops = Rc::new(Cell::new(0));
    let modules = Rc::new(ModuleResources::new(Rc::new(RefCell::new(Backend {
        replies: replies.clone(),
        requests: requests.clone(),
        drops: drops.clone(),
    }))));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(Rc::new(RefCell::new(Host)));
    (modules, runtime, realm, replies, requests, drops)
}
fn respond(replies: &Replies, url: &str, final_url: &str, source: &str) {
    *replies
        .borrow_mut()
        .entry(url.into())
        .or_default()
        .borrow_mut() = Some(Ok(URLResponse {
        final_url: final_url.into(),
        body: source.as_bytes().to_vec(),
        text_encoding: "utf-8".into(),
        mime_type: "text/javascript".into(),
        status_code: 200,
    }));
}
fn assert_js(runtime: &mut QuickJsJavaScriptRuntime, realm: &JavaScriptRealm, source: &str) {
    let result = runtime.Evaluate(realm, source, "assert.js");
    assert!(result.Succeeded(), "{:?}", result.exception);
}
fn pump_until(
    modules: &ModuleResources,
    runtime: &mut QuickJsJavaScriptRuntime,
    realm: &JavaScriptRealm,
    ready: impl Fn() -> bool,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while !ready() {
        modules.Pump(runtime, realm, 16);
        assert!(
            std::time::Instant::now() < deadline,
            "module worker failed to wake/complete"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
#[test]
fn graph_loading_yields_reuses_shared_requests_and_executes_real_cyclic_records() {
    let (modules, mut runtime, realm, replies, requests, _) = setup();
    modules.StartExternalScript(1, "https://test/main.js", "https://test/page");
    assert_eq!(modules.Pump(&mut runtime, &realm, 16), 0);
    assert!(!modules.ScriptReady(1));
    assert!(modules
        .ResolveModule("./missing.js", "https://test/main.js")
        .is_err());
    assert_eq!(
        requests.borrow().len(),
        1,
        "resolver must never start a request"
    );
    respond(
        &replies,
        "https://test/main.js",
        "https://test/main.js",
        "import {a} from './a.js'; import {b} from './b.js'; globalThis.answer=a()+b();",
    );
    assert_eq!(modules.Pump(&mut runtime, &realm, 1), 1);
    assert!(
        !modules.ScriptReady(1),
        "worker must not pretend the graph is ready"
    );
    pump_until(&modules, &mut runtime, &realm, || {
        requests.borrow().len() == 3
    });
    assert_eq!(
        requests.borrow().len(),
        3,
        "all direct requests start before dependencies complete"
    );
    respond(
        &replies,
        "https://test/a.js",
        "https://test/a.js",
        "import {b} from './b.js'; export const a=()=>b()+1;",
    );
    respond(
        &replies,
        "https://test/b.js",
        "https://test/b.js",
        "import {a} from './a.js'; export const b=()=>2;",
    );
    pump_until(&modules, &mut runtime, &realm, || modules.ScriptReady(1));
    assert!(modules.ScriptReady(1));
    assert_eq!(
        requests.borrow().len(),
        3,
        "cycles/shared imports must not issue duplicate requests"
    );
    let root = modules.ScriptModule(1).unwrap();
    assert!(runtime
        .EvaluateCompiledModule(
            &realm,
            root.module.as_ref().unwrap(),
            &root.url,
            Some(modules.clone())
        )
        .Succeeded());
    assert_js(
        &mut runtime,
        &realm,
        "if(answer!==5) throw Error('cyclic graph');",
    );
    assert_eq!(requests.borrow().len(), 3, "linking must not fetch");
}
#[test]
fn inline_records_are_distinct_and_imports_follow_redirect_and_import_map() {
    let (modules, mut runtime, realm, replies, requests, _) = setup();
    modules
        .ProcessImportMap(r#"{"imports":{"pkg/":"./lib/"}}"#, "https://test/")
        .unwrap();
    modules.StartInlineScript(
        1,
        "import {x} from 'pkg/entry.js'; globalThis.one=x;",
        "https://test/page",
        "https://test/page",
    );
    modules.StartInlineScript(
        2,
        "globalThis.two=2;",
        "https://test/page",
        "https://test/page",
    );
    modules.Pump(&mut runtime, &realm, 16);
    assert!(!modules.ScriptReady(1));
    pump_until(&modules, &mut runtime, &realm, || {
        modules.ScriptReady(2) && requests.borrow().len() == 1
    });
    assert!(modules.ScriptReady(2));
    respond(
        &replies,
        "https://test/lib/entry.js",
        "https://cdn.test/pkg/main.js",
        "export {x} from './leaf.js';",
    );
    pump_until(&modules, &mut runtime, &realm, || {
        requests.borrow().len() == 2
    });
    assert_eq!(
        requests.borrow().last().unwrap().url,
        "https://cdn.test/pkg/leaf.js"
    );
    respond(
        &replies,
        "https://cdn.test/pkg/leaf.js",
        "https://cdn.test/pkg/leaf.js",
        "export const x=3;",
    );
    pump_until(&modules, &mut runtime, &realm, || modules.ScriptReady(1));
    for id in [1, 2] {
        let root = modules.ScriptModule(id).unwrap();
        let result = runtime.EvaluateCompiledModule(
            &realm,
            root.module.as_ref().unwrap(),
            &root.url,
            Some(modules.clone()),
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
    assert_js(
        &mut runtime,
        &realm,
        "if(one!==3 || two!==2) throw Error('inline identity');",
    );
    assert_eq!(requests.borrow().len(), 2);
}
#[test]
fn graph_failure_is_terminal_and_cancellation_releases_pending_operations() {
    let (modules, mut runtime, realm, replies, requests, drops) = setup();
    modules.StartExternalScript(1, "https://test/root.js", "https://test/page");
    respond(
        &replies,
        "https://test/root.js",
        "https://test/root.js",
        "import './bad.js'; globalThis.executed=true;",
    );
    pump_until(&modules, &mut runtime, &realm, || {
        requests.borrow().len() == 2
    });
    *replies.borrow()["https://test/bad.js"].borrow_mut() =
        Some(Err(io::Error::other("network failed")));
    pump_until(&modules, &mut runtime, &realm, || modules.ScriptReady(1));
    assert!(modules.ScriptReady(1));
    assert!(matches!(
        modules.ScriptModule(1),
        Err(ModuleLoadError::Resource { .. })
    ));
    for _ in 0..4 {
        modules.Pump(&mut runtime, &realm, 16);
    }
    assert_eq!(
        requests.borrow().len(),
        2,
        "failed modules remain failed without automatic retries"
    );
    modules.StartInlineScript(
        2,
        "import { from './syntax.js'",
        "https://test/page",
        "https://test/page",
    );
    pump_until(&modules, &mut runtime, &realm, || modules.ScriptReady(2));
    assert!(matches!(
        modules.ScriptModule(2),
        Err(ModuleLoadError::Compilation(_))
    ));
    modules.StartExternalScript(3, "https://test/pending.js", "https://test/page");
    assert_eq!(drops.get(), 2);
    modules.StopLoading();
    assert_eq!(drops.get(), 3);
    assert_js(
        &mut runtime,
        &realm,
        "if(typeof executed!=='undefined') throw Error('must not execute during compilation');",
    );
}

#[test]
fn rejected_module_responses_and_unmapped_imports_have_stable_terminal_errors() {
    let (modules, mut runtime, realm, replies, requests, _) = setup();
    for (id, url, mime, status) in [
        (1, "https://test/html.js", "text/html", 200),
        (2, "https://test/404.js", "text/javascript", 404),
    ] {
        modules.StartExternalScript(id, url, "https://test/page");
        *replies.borrow()[url].borrow_mut() = Some(Ok(URLResponse {
            final_url: url.into(),
            body: b"globalThis.mustNotExecute=true;".to_vec(),
            mime_type: mime.into(),
            status_code: status,
            ..Default::default()
        }));
    }
    modules.StartInlineScript(
        3,
        "import 'unmapped';",
        "https://test/page",
        "https://test/page",
    );
    pump_until(&modules, &mut runtime, &realm, || {
        (1..=3).all(|id| modules.ScriptReady(id))
    });
    assert!(matches!(
        modules.ScriptModule(1),
        Err(ModuleLoadError::Resource { .. })
    ));
    assert!(matches!(
        modules.ScriptModule(2),
        Err(ModuleLoadError::Resource { .. })
    ));
    assert!(matches!(
        modules.ScriptModule(3),
        Err(ModuleLoadError::Compilation(JavaScriptException {
            kind: JavaScriptExceptionKind::kTypeError,
            ..
        }))
    ));
    assert_eq!(requests.borrow().len(), 2);
    for _ in 0..3 {
        modules.Pump(&mut runtime, &realm, 16);
    }
    assert_eq!(requests.borrow().len(), 2);
    assert_js(
        &mut runtime,
        &realm,
        "if(typeof mustNotExecute!=='undefined') throw Error('rejected response executed');",
    );
}

#[test]
fn background_modules_snapshot_import_map_through_compile_and_link() {
    let (modules, mut runtime, realm, replies, requests, _) = setup();
    modules
        .ProcessImportMap(r#"{"imports":{"pkg":"./old.js"}}"#, "https://test/")
        .unwrap();
    modules.StartInlineScript(
        10,
        "import {x} from 'pkg';globalThis.answer=x;",
        "https://test/page",
        "https://test/page",
    );
    modules.Pump(&mut runtime, &realm, 1); // Captures old map at Begin.
    modules
        .ProcessImportMap(r#"{"imports":{"pkg":"./new.js"}}"#, "https://test/")
        .unwrap();
    pump_until(&modules, &mut runtime, &realm, || {
        requests.borrow().len() == 1
    });
    assert_eq!(requests.borrow()[0].url, "https://test/old.js");
    respond(
        &replies,
        "https://test/old.js",
        "https://test/old.js",
        "export const x=41;",
    );
    pump_until(&modules, &mut runtime, &realm, || modules.ScriptReady(10));
    let root = modules.ScriptModule(10).unwrap();
    let result = runtime.EvaluateCompiledModule(
        &realm,
        root.module.as_ref().unwrap(),
        &root.url,
        Some(modules.ResolverForScript(10)),
    );
    assert!(result.Succeeded(), "{:?}", result.exception);
    assert_js(
        &mut runtime,
        &realm,
        "if(answer!==41)throw Error('map rebound after background compile');",
    );
    assert_eq!(requests.borrow().len(), 1);
}
#[test]
fn background_module_stop_loading_discards_late_compilation() {
    let (modules, mut runtime, realm, _, requests, _) = setup();
    modules.StartInlineScript(
        1,
        "import './late.js';globalThis.mustNotRun=true;",
        "https://test/page",
        "https://test/page",
    );
    modules.Pump(&mut runtime, &realm, 1);
    assert!(!modules.ScriptReady(1));
    modules.StopLoading();
    for _ in 0..5 {
        modules.Pump(&mut runtime, &realm, 16);
    }
    assert!(modules.ScriptReady(1));
    assert!(matches!(
        modules.ScriptModule(1),
        Err(ModuleLoadError::Resource { .. })
    ));
    assert!(requests.borrow().is_empty());
    assert_js(
        &mut runtime,
        &realm,
        "if(typeof mustNotRun!=='undefined')throw Error('cancelled worker executed');",
    );
}

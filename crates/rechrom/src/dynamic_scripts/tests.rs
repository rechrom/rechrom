use super::*;
use crate::dom_mutation::ApplyDOMTreeMutation;
use dom::dom_mutation::DOMMutationType as M;
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use std::rc::Weak;
use url_loader::{URLLoadOperation, URLResponse};
struct Pending {
    polls: usize,
    response: Option<URLResponse>,
}
impl URLLoadOperation for Pending {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        self.polls += 1;
        if self.polls < 3 {
            return Ok(None);
        }
        Ok(self.response.take())
    }
}
struct Loader(Rc<RefCell<Vec<URLRequest>>>);
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.0.borrow_mut().push(request.clone());
        if request.url.ends_with("fail.js") {
            return Err(io::Error::other("original resource failure"));
        }
        Ok(Box::new(Pending{polls:0,response:Some(URLResponse {final_url:"https://cdn.test/redirected.js".into(),status_code:200,mime_type:"text/javascript".into(),body:b"order.push('external'); globalThis.loadedScriptId=document.currentScript.id; Promise.resolve().then(()=>order.push('external-job'));".to_vec(),..Default::default()})}))
    }
}
struct Transport;
impl xhr_transport::XMLHttpRequestTransport for Transport {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("no XHR in this test"))
    }
}
#[derive(Default)]
struct Client {
    executed: Vec<u64>,
    failures: Vec<String>,
    errors: Vec<String>,
}
impl ScriptLoadClient for Client {
    fn DidExecuteScript(&mut self, s: ParserScript, _: &str, _: bool) {
        self.executed.push(s.node_id);
    }
    fn DidFailResource(&mut self, _: &str, e: &str) {
        self.failures.push(e.into());
    }
    fn DidReportScriptError(&mut self, e: &javascript::javascript_runtime::JavaScriptException) {
        self.errors.push(e.message.clone());
    }
}
fn run(
    html: &str,
    verify: impl FnOnce(
        &mut QuickJsJavaScriptRuntime,
        &JavaScriptRealm,
        &Rc<DynamicScriptTasks>,
        &Rc<RefCell<Client>>,
        &Rc<RefCell<Vec<URLRequest>>>,
        &Rc<RefCell<WindowJavaScriptBindings>>,
    ),
) {
    let document = Rc::new(RefCell::new(DOM::new()));
    let service = Rc::new(RefCell::new(Weak::<DynamicScriptTasks>::new()));
    let emitted = service.clone();
    let mutation_document = document.clone();
    let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document.clone(),
        Box::new(move |mutation| {
            let mut owner = mutation_document.borrow_mut();
            ApplyDOMTreeMutation(&mut owner, mutation);
            let tree = owner.GetDocumentMut();
            let (id, scripts) = match mutation.mutation_type {
                M::kAppendChild | M::kInsertBefore => (mutation.child_node_id, true),
                M::kSetAttribute | M::kSetTextContent | M::kSetInnerHTML => (
                    mutation.target_node_id,
                    mutation.mutation_type != M::kSetInnerHTML,
                ),
                _ => return,
            };
            if let (Some(service), Some(index)) =
                (emitted.borrow().upgrade(), tree.FindNodeById(id))
            {
                service
                    .PrepareConnectedScripts(tree, index, scripts)
                    .unwrap();
            }
        }),
    )));
    let requests = Rc::new(RefCell::new(Vec::new()));
    let loader = Rc::new(RefCell::new(Loader(requests.clone())));
    let mut scheduler = crate::script_scheduler::ScriptScheduler::new(
        document.clone(),
        bindings.clone(),
        loader.clone(),
        "https://example.test/page".into(),
    );
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let dynamic = Rc::new(DynamicScriptTasks::new(
        document,
        bindings.clone(),
        loader,
        "https://example.test/page".into(),
        scheduler.StartedScripts(),
        runtime.SupportsModules(),
    ));
    scheduler.InstallDynamicScripts(&dynamic).unwrap();
    *service.borrow_mut() = Rc::downgrade(&dynamic);
    let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
        bindings,
        Box::new(Transport),
        "test".into(),
        None,
    )));
    let client = Rc::new(RefCell::new(Client::default()));
    let host = Rc::new(RefCell::new(DynamicScriptHost::new(
        window.clone(),
        dynamic.clone(),
        client.clone(),
    )));
    let realm = runtime.CreateRealm(host);
    window.borrow_mut().BindRuntime(&realm);
    for (source, name) in [
        (webapi::DOMBootstrapSource().to_owned(), "dom"),
        (WindowJavaScriptBindings::BootstrapSource(), "window"),
    ] {
        let r = runtime.Evaluate(&realm, &source, name);
        assert!(r.Succeeded(), "{:?}", r.exception);
    }
    let mut parser_client = Client::default();
    scheduler
        .ParseDocument(html, 16384, 256, &mut runtime, &realm, &mut parser_client)
        .unwrap();
    assert!(
        parser_client.errors.is_empty(),
        "{:?}",
        parser_client.errors
    );
    WindowJavaScriptBindings::RunTasks(&window, &mut runtime, &realm, 0.0, &mut |e| {
        panic!("{}", e.message)
    });
    verify(&mut runtime, &realm, &dynamic, &client, &requests, &window);
}
fn assert_js(runtime: &mut QuickJsJavaScriptRuntime, realm: &JavaScriptRealm, source: &str) {
    let r = runtime.Evaluate(
        realm,
        &format!("if (!({source})) throw Error('verification failed');"),
        "verify",
    );
    assert!(r.Succeeded(), "{:?}", r.exception);
}
#[test]
fn connected_inline_script_current_script_jobs_load_and_timer_follow_source_order() {
    run(
        r#"<html><head><script id='parser'>
        globalThis.order=['parser'];globalThis.once=0;
        const detached=document.createElement('div');const s=document.createElement('script');s.id='dynamic';
        s.textContent="once++;order.push('script');if(document.currentScript.id!=='dynamic')throw Error('currentScript');Promise.resolve().then(()=>order.push(document.currentScript===null?'job':'bad-job'));";
        s.onload=()=>order.push('load');detached.appendChild(s);
        if(once!==0)throw Error('detached executed');document.head.appendChild(detached);
        s.setAttribute('data-again','yes');document.head.appendChild(detached);
        setTimeout(()=>order.push('timer'),0);
        if(document.currentScript.id!=='parser')throw Error('parser rerun');
    </script></head></html>"#,
        |runtime, realm, dynamic, client, requests, window| {
            // The timer is a separate task after the dynamic script task.
            WindowJavaScriptBindings::RunTasks(window, runtime, realm, 0.0, &mut |e| {
                panic!("{}", e.message)
            });
            assert_js(runtime,realm,"once===1 && JSON.stringify(order)===JSON.stringify(['parser','script','job','load','timer'])");
            assert_eq!(client.borrow().executed.len(), 1);
            assert!(client.borrow().errors.is_empty());
            assert!(requests.borrow().is_empty());
            assert_eq!(dynamic.PendingLoads(), 0);
        },
    );
}
#[test]
fn dynamic_focus_listener_errors_are_reported_at_the_owning_checkpoint() {
    run(
        r#"<html><head></head><body><input id='first'><input id='second'><script>
        document.getElementById('first').onfocus=()=>{throw Error('first-focus')};
        document.getElementById('second').onfocus=()=>{throw Error('second-focus')};
        const s=document.createElement('script');
        s.textContent="document.getElementById('first').focus();Promise.resolve().then(()=>document.getElementById('second').focus());throw Error('dynamic-evaluate');";
        s.onload=()=>document.getElementById('first').focus();document.head.appendChild(s);
        </script></body></html>"#,
        |_, _, _, client, _, _| {
            let errors = &client.borrow().errors;
            assert_eq!(errors.len(), 4, "{errors:?}");
            for (actual, expected) in errors.iter().zip([
                "first-focus",
                "dynamic-evaluate",
                "second-focus",
                "first-focus",
            ]) {
                assert!(actual.contains(expected), "{actual} != {expected}");
            }
        },
    );
}
#[test]
fn base_url_delayed_external_redirect_and_resource_failure_keep_source_events() {
    run(
        r#"<html><head><base href='https://assets.test/scripts/'><script>
        globalThis.order=[];const ext=document.createElement('script');ext.id='loaded';ext.src='./ok.js';ext.onload=()=>order.push('load');document.head.appendChild(ext);
        const failure=document.createElement('script');failure.src='fail.js';failure.onload=()=>order.push('failure-load');document.head.appendChild(failure);
    </script></head></html>"#,
        |runtime, realm, dynamic, client, requests, window| {
            // First RunTasks(0) leaves the delayed response pending; poll through
            // the same production queue using the host's retained Window.
            assert_eq!(
                requests.borrow()[0].url,
                "https://assets.test/scripts/ok.js"
            );
            assert_eq!(requests.borrow()[0].referrer, "https://example.test/page");
            assert_eq!(
                requests.borrow()[0].destination,
                RequestDestination::kScript
            );
            assert!(client.borrow().errors.is_empty());
            assert_eq!(client.borrow().failures, vec!["original resource failure"]);
            assert_js(runtime, realm, "order.includes('failure-load')");
            assert_eq!(dynamic.PendingLoads(), 1);
            for _ in 0..8 {
                if dynamic.PendingLoads() == 0 {
                    break;
                }
                WindowJavaScriptBindings::RunTasks(window, runtime, realm, 0.0, &mut |e| {
                    panic!("{}", e.message)
                });
            }
            assert_eq!(dynamic.PendingLoads(), 0);
            assert_js(runtime,realm,"loadedScriptId==='loaded' && JSON.stringify(order)===JSON.stringify(['failure-load','external','external-job','load'])");
            assert_eq!(client.borrow().executed.len(), 1);
        },
    );
}
#[test]
fn innerhtml_script_and_json_nodes_are_not_executed() {
    run(
        r#"<html><head><script>
        globalThis.order=[];const box=document.createElement('div');document.head.appendChild(box);
        box.innerHTML='<script>order.push("innerHTML")<\/script>';
        const data=document.createElement('script');data.type='application/json';data.textContent='order.push("json")';document.head.appendChild(data);
    </script></head></html>"#,
        |runtime, realm, _, client, _, _| {
            assert_js(runtime, realm, "order.length===0");
            assert!(client.borrow().executed.is_empty());
        },
    );
}

#[test]
fn stale_dynamic_registration_cannot_execute_a_parser_script_twice() {
    run("<html><head><script id=parser>globalThis.parserCount=(globalThis.parserCount||0)+1;</script></head></html>",
        |runtime,realm,dynamic,client,_,window| {
            let id={
                let owner=dynamic.document.borrow();let document=owner.GetDocument();
                (0..document.NodeCount()).find_map(|index|document.Node(index).FindAttribute("id")
                    .filter(|attribute|attribute.value=="parser").map(|_|document.Node(index).Id())).unwrap()
            };
            // A mutation may have prepared this entry during the parser's load
            // wait. Its host task survives after parser execution has claimed it.
            dynamic.StartScriptLoad(id,"pending.js").unwrap();
            dynamic.registrations.borrow_mut().push_back(id);
            dynamic.EnqueuePreparedTasks(window,client.clone());
            for _ in 0..4 {
                WindowJavaScriptBindings::RunTasks(window,runtime,realm,0.0,&mut |e|panic!("{}",e.message));
            }
            assert_js(runtime,realm,"parserCount===1");
            assert!(client.borrow().executed.is_empty());
            assert!(client.borrow().errors.is_empty());
            assert_eq!(dynamic.PendingLoads(),0,"discarded task retires its resource load");
        });
}

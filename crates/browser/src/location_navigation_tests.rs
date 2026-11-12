//! Location conformance uses the complete browser layout providers.
use javascript::javascript_runtime::*;
use javascript::quickjs_javascript_runtime::{JsValue, QuickJsJavaScriptRuntime};
use std::{cell::RefCell, io, rc::Rc};
use webapi::{dom_bindings::DOMJavaScriptBindings, window_bindings::WindowJavaScriptBindings};
use xhr_transport::*;
struct MockTransport(Rc<RefCell<Vec<XMLHttpRequestData>>>);
impl XMLHttpRequestTransport for MockTransport {
    fn Start(
        &mut self,
        request: &XMLHttpRequestData,
    ) -> io::Result<Box<dyn XMLHttpRequestOperation>> {
        self.0.borrow_mut().push(request.clone());
        Err(io::Error::other("unexpected network access"))
    }
}

#[test]
#[ignore = "manual cold/warm full Page creation profile; no timing assertions"]
fn profile_full_page_creation_and_bootstrap_isolation() {
    crate::test_thread::run(|| {
        struct NoNetwork;
        impl url_loader::URLLoader for NoNetwork {
            fn Load(
                &mut self,
                _: &url_loader::URLRequest,
            ) -> io::Result<Box<dyn url_loader::URLLoadOperation>> {
                Err(io::Error::other("creation profile does not navigate"))
            }
        }
        let mut samples = Vec::new();
        for iteration in 0..5 {
            let start = std::time::Instant::now();
            let assembly = crate::CreateLayoutAssembly();
            let mut page = crate::page::Page::Create(
                Rc::new(RefCell::new(NoNetwork)),
                Rc::new(RefCell::new(
                    image_decoder::skia_image_decoder::SkiaImageDecoder,
                )),
                Rc::new(RefCell::new(crate::CreateSVGImageDecoder(&assembly))),
                crate::CreateBrowserConstraints(1280, 720),
                Some(crate::page::ScriptEnvironment {
                    runtime: Box::new(QuickJsJavaScriptRuntime::with_max_stack_size(
                        8 * 1024 * 1024,
                    )),
                    xhr: Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
                    user_agent: "Mozilla/5.0 creation-profile".into(),
                }),
                None,
            );
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            samples.push(elapsed_ms);
            eprintln!(
                "page-create-profile iteration={iteration} cold={} ms={elapsed_ms:.3}",
                iteration == 0
            );
            let result = page.Evaluate(r#"
                if (globalThis.previousRealmMarker !== undefined) throw Error('global leaked between realms');
                if (Node.prototype.previousRealmMarker !== undefined) throw Error('prototype leaked between realms');
                globalThis.previousRealmMarker = 1;
                Node.prototype.previousRealmMarker = 1;
                if (document !== window.document || typeof document.createElement !== 'function') throw Error('DOM bootstrap missing');
                if (typeof location.assign !== 'function' || typeof location.replace !== 'function') throw Error('Window bootstrap missing');
                true
            "#, "fixture:bootstrap-isolation").unwrap();
            assert!(result.Succeeded(), "{:?}", result.exception);
            assert_eq!(
                result
                    .value
                    .Implementation::<JsValue>()
                    .unwrap()
                    .as_boolean(),
                Some(true)
            );
        }
        eprintln!(
            "page-create-profile summary count={} max_ms={:.3} over16={}",
            samples.len(),
            samples.iter().copied().fold(0.0_f64, f64::max),
            samples.iter().filter(|&&ms| ms >= 16.0).count()
        );
    });
}
fn assert_script(runtime: &mut QuickJsJavaScriptRuntime, realm: &JavaScriptRealm, script: &str) {
    let result = runtime.Evaluate(realm, script, "location-test.js");
    assert!(result.Succeeded(), "{:?}", result.exception);
    assert_eq!(
        result
            .value
            .Implementation::<JsValue>()
            .unwrap()
            .as_boolean(),
        Some(true)
    );
}

#[test]
fn native_history_retains_window_state_and_restores_host_entry_owners() {
    crate::test_thread::run(|| {
        use navigation_history::{
            FrameLoadType, HistoryItemSnapshot, NavigationRequestMetadata, WebNavigationType,
        };
        use std::sync::Arc;
        struct Response(Option<url_loader::URLResponse>);
        impl url_loader::URLLoadOperation for Response {
            fn Poll(&mut self) -> io::Result<Option<url_loader::URLResponse>> {
                Ok(self.0.take())
            }
        }
        struct Loader;
        impl url_loader::URLLoader for Loader {
            fn Load(
                &mut self,
                request: &url_loader::URLRequest,
            ) -> io::Result<Box<dyn url_loader::URLLoadOperation>> {
                Ok(Box::new(Response(Some(url_loader::URLResponse {
                    final_url: request.url.clone(),
                    body:
                        b"<!doctype html><title>native history fixture</title><body>history</body>"
                            .to_vec(),
                    mime_type: "text/html".into(),
                    status_code: 200,
                    ..Default::default()
                }))))
            }
        }
        struct Client(
            Rc<RefCell<Vec<Arc<HistoryItemSnapshot>>>>,
            crate::page::PageChromeState,
        );
        impl crate::page::PageClient for Client {
            fn DidEventHandlerChromeEffect(
                &mut self,
                effect: crate::page::EventHandlerChromeEffect,
            ) {
                self.1.Apply(effect);
            }
            fn DidCommitHistory(&mut self, item: Arc<HistoryItemSnapshot>, _: FrameLoadType) {
                self.0.borrow_mut().push(item);
            }
        }
        let create = |commits: Rc<RefCell<Vec<Arc<HistoryItemSnapshot>>>>| {
            let assembly = crate::CreateLayoutAssembly();
            crate::page::Page::Create(
                Rc::new(RefCell::new(Loader)),
                Rc::new(RefCell::new(
                    image_decoder::skia_image_decoder::SkiaImageDecoder,
                )),
                Rc::new(RefCell::new(crate::CreateSVGImageDecoder(&assembly))),
                crate::CreateBrowserConstraints(160, 96),
                Some(crate::page::ScriptEnvironment {
                    runtime: Box::new(QuickJsJavaScriptRuntime::with_max_stack_size(
                        8 * 1024 * 1024,
                    )),
                    xhr: Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
                    user_agent: "native-history-test".into(),
                }),
                Some(Rc::new(RefCell::new(Client(commits, Default::default())))),
            )
        };
        let check = |page: &mut crate::page::Page, script: &str| {
            let result = page.Evaluate(script, "fixture:native-history").unwrap();
            assert!(result.Succeeded(), "{:?}", result.exception);
            assert_eq!(
                result
                    .value
                    .Implementation::<JsValue>()
                    .unwrap()
                    .as_boolean(),
                Some(true)
            );
        };
        let commits = Rc::new(RefCell::new(Vec::new()));
        let mut page = create(commits.clone());
        page.OpenSynchronously("https://history.test/start", 4096, 4096)
            .unwrap();
        let initial = commits.borrow().last().unwrap().clone();
        check(
            &mut page,
            r#"
            globalThis.savedHistory = history;
            globalThis.historyEvents = [];
            navigation.addEventListener('navigate', event => historyEvents.push([event.navigationType,event.destination.url]));
            const original = {n:1};
            history.pushState(original, '', '#one');
            original.n = 99;
            navigation.updateCurrentEntry({state:{nav:1}});
            history instanceof History && history.state.n === 1 && history.state === history.state &&
                history.length === 2 && navigation.entries().length === 2 && historyEvents[0][0] === 'push'
        "#,
        );
        let one = commits.borrow().last().unwrap().clone();
        page.CollectJavaScriptGarbage();
        check(
            &mut page,
            "history.state.n===1 && history.state===history.state",
        );
        check(
            &mut page,
            r#"
            history.pushState({n:2}, '', '#two');
            const prior = history.state;
            history.replaceState({n:3}, '', '#three');
            globalThis.popAlias = false;
            globalThis.retainedPop = null;
            addEventListener('popstate', event => {popAlias = event.state === history.state; retainedPop = event;});
            let security = false; try {history.pushState({}, '', 'https://other.test/');} catch(error) {security=error.name==='SecurityError';}
            let clone = false; try {history.replaceState(()=>{}, '');} catch(error) {clone=error.name==='DataCloneError';}
            prior.n === 2 && history.state.n === 3 && prior !== history.state &&
                history.length === 3 && historyEvents.at(-1)[0] === 'replace' && security && clone
        "#,
        );
        let three = commits.borrow().last().unwrap().clone();
        assert!(page.TraverseToHistoryEntry(&one.navigation_api_key));
        page.RunTask().unwrap();
        check(&mut page, "popAlias && history.state.n===1 && navigation.currentEntry.getState().nav===1 && history===savedHistory && retainedPop.state===history.state");
        check(
            &mut page,
            r#"
            const before = retainedPop.state;
            history.replaceState({n:4}, '', '#four');
            globalThis.savedFourth = history.state;
            before === retainedPop.state && retainedPop.state !== history.state && retainedPop.state.n === 1
        "#,
        );
        check(&mut page, "document.open();document.write('<!doctype html><title>reused window</title><body>reused</body>');document.close();history===savedHistory && history.length===3 && history.state===savedFourth");

        let metadata = NavigationRequestMetadata {
            load_type: FrameLoadType::BackForward,
            history_item: Some(one.clone()),
            back_entries: Arc::from([initial.clone()]),
            forward_entries: Arc::from([three.clone()]),
            navigation_api_state: None,
            navigation_type: WebNavigationType::BackForward,
            is_browser_initiated: true,
            had_transient_user_activation: false,
            requestor_origin: one.requestor_origin.clone(),
            propagated_text_fragment_token: None,
            internal_scroll_to_text_fragment: None,
        };
        let mut restored = create(Default::default());
        restored
            .OpenRequest(
                &url_loader::URLRequest {
                    url: one.url.clone(),
                    navigation: Some(metadata.clone()),
                    ..Default::default()
                },
                4096,
                4096,
            )
            .unwrap();
        while restored.IsLoading() {
            restored.RunTask().unwrap();
        }
        check(
            &mut restored,
            r#"
            globalThis.restoredEntries = navigation.entries();
            globalThis.restoredPopAlias = false;
            addEventListener('popstate', event => restoredPopAlias=event.state===history.state);
            history.state.n===1 && history.state===history.state && navigation.currentEntry.getState().nav===1 &&
                history.length===3 && restoredEntries.length===3 && restoredEntries[1]===navigation.currentEntry &&
                navigation.canGoBack && navigation.canGoForward
        "#,
        );
        let entries = restored
            .Evaluate(
                "JSON.stringify(navigation.entries().map(entry=>[entry.key,entry.id]))",
                "fixture:entry-identities",
            )
            .unwrap();
        assert!(entries.Succeeded(), "{:?}", entries.exception);
        let text = entries
            .value
            .Implementation::<JsValue>()
            .unwrap()
            .as_string()
            .unwrap()
            .to_string_lossy();
        assert!(text.contains(&one.navigation_api_key) && text.contains(&one.navigation_api_id));
        assert!(
            text.contains(&initial.navigation_api_key) && text.contains(&three.navigation_api_key)
        );
        assert!(restored.TraverseToHistoryEntry(&three.navigation_api_key));
        restored.RunTask().unwrap();
        check(&mut restored, "restoredPopAlias && history.state.n===3 && navigation.currentEntry===restoredEntries[2] && !navigation.canGoForward");

        let mut foreign = (*initial).clone();
        foreign.url = "https://foreign.test/".into();
        foreign.committed_origin = Some(navigation_history::RequestorOrigin::Tuple {
            scheme: "https".into(),
            host: "foreign.test".into(),
            port: 443,
        });
        let mut filtered = create(Default::default());
        filtered
            .OpenRequest(
                &url_loader::URLRequest {
                    url: one.url.clone(),
                    navigation: Some(NavigationRequestMetadata {
                        back_entries: Arc::from([initial, Arc::new(foreign)]),
                        ..metadata
                    }),
                    ..Default::default()
                },
                4096,
                4096,
            )
            .unwrap();
        while filtered.IsLoading() {
            filtered.RunTask().unwrap();
        }
        check(&mut filtered, "history.length===4 && navigation.entries().length===2 && navigation.currentEntry.index===0 && !navigation.canGoBack && navigation.canGoForward");

        // Exercise the ordinary response commit on the same Page domain.
        // Retaining the native owners proves that publication replaces them,
        // rather than rewriting the outgoing Window's Document in place.
        check(
            &mut page,
            r#"
            globalThis.outgoingWindowMarker = 42;
            Node.prototype.outgoingPrototypeMarker = 43;
            document.body.setAttribute('data-outgoing-document', 'retained');
            true
        "#,
        );
        let (outgoing_document, outgoing_window, outgoing_timing) = page.WindowOwnersForTesting();
        let outgoing_performance = outgoing_timing.EventTimingSource();
        let outgoing_form = outgoing_document
            .borrow()
            .GetDocument()
            .FormController()
            .unwrap();
        let outgoing_document_identity = outgoing_document.borrow().GetDocument().RootHandle();
        let outgoing_url = outgoing_document
            .borrow()
            .GetDocument()
            .DocumentURL()
            .to_owned();
        let outgoing_history = commits.borrow().last().unwrap().clone();
        let outgoing_sequence = page.CurrentFrame().unwrap().sequence;

        page.OpenSynchronously("https://history.test/second-document", 4096, 4096)
            .unwrap();
        let (incoming_document, incoming_window, incoming_timing) = page.WindowOwnersForTesting();
        let incoming_form = incoming_document
            .borrow()
            .GetDocument()
            .FormController()
            .unwrap();
        assert!(!Rc::ptr_eq(&outgoing_document, &incoming_document));
        assert!(!Rc::ptr_eq(&outgoing_window, &incoming_window));
        assert!(!Rc::ptr_eq(&outgoing_timing, &incoming_timing));
        assert!(!Rc::ptr_eq(
            &outgoing_performance,
            &incoming_timing.EventTimingSource()
        ));
        assert!(!Rc::ptr_eq(&outgoing_form, &incoming_form));
        {
            let outgoing = outgoing_document.borrow();
            let outgoing = outgoing.GetDocument();
            assert!(!outgoing.IsActive());
            assert!(outgoing.IsContextDestroyed());
            assert_eq!(outgoing.RootHandle(), outgoing_document_identity);
            assert_eq!(outgoing.DocumentURL(), outgoing_url);
            assert_eq!(
                outgoing
                    .Node(outgoing.BodyElement().unwrap())
                    .FindAttribute("data-outgoing-document")
                    .unwrap()
                    .value,
                "retained"
            );
        }
        assert!(incoming_document.borrow().GetDocument().IsActive());
        assert!(page.CurrentFrame().unwrap().sequence > outgoing_sequence);
        let incoming_history = commits.borrow().last().unwrap().clone();
        assert_eq!(incoming_history.url, "https://history.test/second-document");
        assert_ne!(
            incoming_history.document_sequence_number,
            outgoing_history.document_sequence_number
        );
        assert_ne!(
            incoming_history.navigation_api_key,
            outgoing_history.navigation_api_key
        );
        assert_ne!(
            incoming_history.navigation_api_id,
            outgoing_history.navigation_api_id
        );
        check(
            &mut page,
            r#"
            typeof outgoingWindowMarker === 'undefined' &&
                Node.prototype.outgoingPrototypeMarker === undefined &&
                document.title === 'native history fixture' &&
                document.body.getAttribute('data-outgoing-document') === null &&
                document === window.document && history.state === null &&
                history.length === 3 && navigation.entries().length === 3 &&
                navigation.currentEntry.index === 2 && navigation.canGoBack && !navigation.canGoForward
        "#,
        );
    });
}
#[test]
fn location_navigation_resolves_urls_and_preserves_outgoing_document() {
    let owner = Rc::new(RefCell::new(dom::DOM::new()));
    let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        owner,
        Box::new(|_| {}),
    )));
    let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
        dom,
        Box::new(MockTransport(Rc::new(RefCell::new(Vec::new())))),
        "Mozilla/5.0 TEST".into(),
        None,
    )));
    window
        .borrow_mut()
        .SetURL("https://example.test/a/b?old=1".into());
    let navigations = Rc::new(RefCell::new(Vec::new()));
    let output = navigations.clone();
    window
        .borrow_mut()
        .SetLocationNavigator(Rc::new(move |request| output.borrow_mut().push(request)));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(window.clone());
    assert!(runtime
        .Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl")
        .Succeeded());
    assert!(runtime
        .Evaluate(
            &realm,
            &WindowJavaScriptBindings::BootstrapSource(),
            "window-webidl"
        )
        .Succeeded());
    assert_script(
        &mut runtime,
        &realm,
        r#"
            const outgoing = location;
            location.assign('../next?q=hello world');
            location.replace({toString() { return '/replacement#' + location.pathname; }});
            location.href = '?href=1';
            window.location = '/window';
            document.location = '/document';
            location.assign(12);
            let missing=false; try { location.assign(); } catch(e) { missing=e instanceof TypeError; }
            let invalid=false; try { location.replace('http://[bad'); } catch(e) { invalid=e instanceof SyntaxError; }
            let conversion=false; try { location.href={toString(){throw new TypeError('conversion')}}; } catch(e) { conversion=e instanceof TypeError; }
            missing && invalid && conversion && location === outgoing &&
                location.href === 'https://example.test/a/b?old=1' && document.location === location
        "#,
    );
    let requests = navigations.borrow();
    assert_eq!(requests.len(), 6);
    let expected = [
        ("https://example.test/next?q=hello%20world", false),
        ("https://example.test/replacement#/a/b", true),
        ("https://example.test/a/b?href=1", false),
        ("https://example.test/window", false),
        ("https://example.test/document", false),
        ("https://example.test/a/12", false),
    ];
    for (request, (url, replace)) in requests.iter().zip(expected) {
        assert_eq!(request.url, url);
        assert_eq!(request.replace_history, replace);
        assert_eq!(request.referrer, "https://example.test/a/b?old=1");
    }
}

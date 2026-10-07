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
    crate::native_test_thread::run(|| {
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
                Rc::new(RefCell::new(
                    image_decoder::svg_image_decoder::SVGImageDecoder::new(&assembly),
                )),
                crate::CreateBrowserConstraints(1280, 720),
                Some(crate::page::ScriptEnvironment {
                    runtime: Box::new(QuickJsJavaScriptRuntime::with_native_stack_budget(
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

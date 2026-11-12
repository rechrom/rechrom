use dom::dom_mutation::ApplyDOMMutations;
use javascript::{
    javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
};
use std::{cell::RefCell, io, rc::Rc};
use webapi::{dom_bindings::DOMJavaScriptBindings, window_bindings::WindowJavaScriptBindings};
use xhr_transport::{
    XMLHttpRequestData, XMLHttpRequestOperation, XMLHttpRequestTransport, XMLHttpResponseData,
};

struct ImmediateOperation(Option<XMLHttpResponseData>);

impl XMLHttpRequestOperation for ImmediateOperation {
    fn Poll(&mut self) -> io::Result<Option<XMLHttpResponseData>> {
        Ok(self.0.take())
    }
}

struct RecordingTransport(Rc<RefCell<Vec<XMLHttpRequestData>>>);

impl XMLHttpRequestTransport for RecordingTransport {
    fn Start(
        &mut self,
        request: &XMLHttpRequestData,
    ) -> io::Result<Box<dyn XMLHttpRequestOperation>> {
        self.0.borrow_mut().push(request.clone());
        Ok(Box::new(ImmediateOperation(Some(XMLHttpResponseData {
            status: 201,
            final_url: "https://example.test/sync".into(),
            mime_type: "text/plain".into(),
            body: b"reply".to_vec(),
            ..Default::default()
        }))))
    }
}

#[test]
fn synchronous_xhr_blocks_on_the_existing_transport_and_returns_response() {
    let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
        "<!doctype html><html><body></body></html>",
    )));
    let changed = document.clone();
    let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document,
        Box::new(move |mutation| {
            ApplyDOMMutations(
                changed.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            );
        }),
    )));
    let requests = Rc::new(RefCell::new(Vec::new()));
    let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
        dom.clone(),
        Box::new(RecordingTransport(requests.clone())),
        "sync-xhr-test".into(),
        None,
    )));
    window
        .borrow_mut()
        .SetURL("https://example.test/page".into());

    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(window.clone());
    window.borrow_mut().BindRuntime(&realm);
    dom.borrow_mut().BindRuntime(&realm);
    for (source, name) in [
        (webapi::DOMBootstrapSource().to_owned(), "dom-webidl"),
        (webapi::WindowBootstrapSource(), "window-webidl"),
    ] {
        let result = runtime.Evaluate(&realm, &source, name);
        assert!(result.Succeeded(), "{name}: {:?}", result.exception);
    }

    let result = runtime.Evaluate(
        &realm,
        "(()=>{const xhr=new XMLHttpRequest();xhr.open('POST','/sync',false);xhr.setRequestHeader('X-Test','yes');xhr.send('body');return xhr.readyState===4&&xhr.status===201&&xhr.responseText==='reply'&&xhr.responseURL==='https://example.test/sync'})()",
        "synchronous-xhr-test.js",
    );
    assert!(result.Succeeded(), "{:?}", result.exception);
    assert_eq!(
        result
            .value
            .Implementation::<javascript::quickjs_javascript_runtime::JsValue>()
            .unwrap()
            .as_boolean(),
        Some(true)
    );

    let requests = requests.borrow();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, "https://example.test/sync");
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].body, b"body");
    assert_eq!(requests[0].headers, vec![("x-test".into(), "yes".into())]);
}

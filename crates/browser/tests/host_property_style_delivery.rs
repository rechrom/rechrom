//! Author properties and native input properties coexist through production Page.
//! Native HTML construction keeps fragment-parser namespace concerns out of this fixture.
//! https://webidl.spec.whatwg.org/#es-attributes
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::{Page, PageClient, ScriptEnvironment};
use javascript::{
    javascript_runtime::JavaScriptException, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
};
use layoutng_assembly::fragment_tree::FragmentNode;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct Loader;
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: b"<!doctype html><html><head><style>body{margin:0;font:16px/20px sans-serif}</style></head><body><x-property-styles id=host></x-property-styles></body></html>".to_vec(),
            ..Default::default()
        }))))
    }
}
struct XHR;
impl xhr_transport::XMLHttpRequestTransport for XHR {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("unexpected property fixture XHR"))
    }
}
struct ErrorClient(Rc<RefCell<Vec<String>>>);
impl PageClient for ErrorClient {
    fn DidReportScriptError(&mut self, error: &JavaScriptException) {
        self.0.borrow_mut().push(error.message.clone());
    }
}

fn evaluate(page: &mut Page, script: &str) {
    let result = page
        .Evaluate(script, "test:host-property-style-delivery")
        .unwrap();
    assert!(
        result.Succeeded(),
        "property fixture JavaScript failed: {}",
        result
            .exception
            .as_ref()
            .map(|error| error.message.as_str())
            .unwrap_or("")
    );
}

fn fixture() -> (Page, Rc<RefCell<Vec<String>>>) {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(640, 800);
    constraints.fonts = vec![layoutng_assembly::internal::layout_input::FontFace {
        family: "sans-serif".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }];
    let errors = Rc::new(RefCell::new(Vec::new()));
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader)),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(
            &assembly,
        ))),
        constraints,
        Some(ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_native_stack_budget(
                4 * 1024 * 1024,
            )),
            xhr: Box::new(XHR),
            user_agent: "host-property-style-delivery-fixture".into(),
        }),
        Some(Rc::new(RefCell::new(ErrorClient(errors.clone())))),
    );
    page.Open("https://host-property-style-delivery.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    assert!(page.CurrentFrame().is_some());
    assert!(
        errors.borrow().is_empty(),
        "load errors: {:?}",
        errors.borrow()
    );
    (page, errors)
}

fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == name)
                .map(|_| document.Node(index).Id())
        })
        .unwrap_or_else(|| panic!("missing native DOM node {name}"))
}

fn find(root: &FragmentNode, id: u64) -> Option<&FragmentNode> {
    if root.node_id == id {
        Some(root)
    } else {
        root.children.iter().find_map(|child| find(child, id))
    }
}

fn assert_control_state(page: &Page, control_id: u64, checked: bool, value: &str) {
    let owner = page.Document();
    let document = owner.GetDocument();
    let index = document
        .FindNodeById(control_id)
        .expect("native input identity retained");
    assert!(document.Node(index).IsHTMLElement("input"));
    assert_eq!(document.ControlChecked(index), checked);
    assert_eq!(document.ControlValue(index), value);
}

#[test]
fn author_checked_and_value_accessors_deliver_shadow_style_and_native_control_state() {
    native_test_thread::run(|| {
        let (mut page, errors) = fixture();
        evaluate(
            &mut page,
            r#"
            function check(value, message) { if (!value) throw new Error(message); }
            function near(actual, expected, message) {
                check(Math.abs(actual - expected) < 0.05,
                      message + ': expected ' + expected + ', got ' + actual);
            }
            var writes = [];
            const checkedState = Symbol('checked'), valueState = Symbol('value');
            class PropertyStyles extends HTMLElement {
                constructor() {
                    super();
                    this.checked = false;
                    this.value = 'initial value';
                    this.attachShadow({mode: 'open'});
                }
                get checked() { return this[checkedState]; }
                set checked(value) {
                    writes.push(['checked', this, value]);
                    this[checkedState] = value;
                    this.updateControl();
                }
                get value() { return this[valueState]; }
                set value(value) {
                    writes.push(['value', this, value]);
                    this[valueState] = value;
                    this.updateControl();
                }
                updateControl() {
                    const input = this.shadowRoot && this.shadowRoot.querySelector('input');
                    if (!input) return;
                    input.checked = this.checked;
                    input.value = this.value;
                }
                connectedCallback() {
                    const style = document.createElement('style');
                    style.id = 'fallback-style';
                    style.appendChild(document.createTextNode(
                        'button { display:block; box-sizing:border-box; width:120px; height:40px; padding:0; border:0; margin:0; }' +
                        'input { display:block; width:16px; height:16px; margin:0; }'
                    ));
                    const button = document.createElement('button');
                    button.id = 'styled-button';
                    button.appendChild(document.createTextNode('Toggle'));
                    const input = document.createElement('input');
                    input.id = 'native-control';
                    input.setAttribute('type', 'checkbox');
                    this.shadowRoot.appendChild(style);
                    this.shadowRoot.appendChild(button);
                    this.shadowRoot.appendChild(input);
                    this.updateControl();
                }
            }
            var host = document.getElementById('host');
            customElements.define('x-property-styles', PropertyStyles);
            check(host instanceof PropertyStyles, 'existing host upgraded');
            check(writes.length === 2 && writes[0][0] === 'checked' && writes[1][0] === 'value',
                  'constructor used author checked and value setters exactly once');
            check(writes.every(write => write[1] === host), 'constructor setters preserve host receiver');
            check(host.checked === false && host.value === 'initial value', 'author state retained');
            check(!host.hasAttribute('checked') && !host.hasAttribute('value'), 'author state is not a host attribute');
            var control = host.shadowRoot.querySelector('input');
            var button = host.shadowRoot.querySelector('button');
            var fallbackStyle = host.shadowRoot.querySelector('style');
            check(control instanceof HTMLInputElement && fallbackStyle instanceof HTMLStyleElement,
                  'shadow children retain native interface brands');
            check(control.checked === false && control.value === 'initial value', 'initial native control state');
            check(host.shadowRoot.querySelectorAll('style').length === 1, 'one fallback style delivered');
            var rect = button.getBoundingClientRect();
            near(rect.width, 120, 'fallback style button width');
            near(rect.height, 40, 'fallback style button height');
            check(getComputedStyle(button).width === '120px' && getComputedStyle(button).height === '40px',
                  'fallback style reaches production computed style');
        "#,
        );
        let control_id = id(&page, "native-control");
        assert_control_state(&page, control_id, false, "initial value");
        assert!(
            errors.borrow().is_empty(),
            "upgrade errors: {:?}",
            errors.borrow()
        );
        evaluate(
            &mut page,
            r#"
            host.checked = true;
            host.value = 'updated value';
            check(writes.length === 4 && writes[2][0] === 'checked' && writes[3][0] === 'value',
                  'reactive author writes each invoke one setter');
            check(writes.every(write => write[1] === host), 'reactive setters preserve host receiver');
            check(host.checked === true && host.value === 'updated value', 'reactive author state');
            check(control === host.shadowRoot.querySelector('input'), 'native input identity retained');
            check(control.checked === true && control.value === 'updated value', 'author setters update native control');
            check(host.shadowRoot.querySelectorAll('style').length === 1, 'reactive update does not duplicate fallback style');
            var updatedRect = button.getBoundingClientRect();
            near(updatedRect.width, 120, 'retained button width');
            near(updatedRect.height, 40, 'retained button height');
        "#,
        );
        assert_control_state(&page, control_id, true, "updated value");
        let button_id = id(&page, "styled-button");
        let style_id = id(&page, "fallback-style");
        let frame = page.CurrentFrame().expect("production frame is published");
        let button = find(&frame.fragments, button_id).expect("styled shadow button is published");
        assert_eq!(button.size.width, 120.0);
        assert_eq!(button.size.height, 40.0);
        assert!(
            find(&frame.fragments, control_id).is_some(),
            "native input is published in the retained fragment tree"
        );
        assert!(
            find(&frame.fragments, style_id).is_none(),
            "fallback style does not create a layout box"
        );
        assert!(
            errors.borrow().is_empty(),
            "property fixture reported JavaScript errors: {:?}",
            errors.borrow()
        );
    });
}

#[test]
fn author_event_handler_shadow_preserves_real_native_dispatch_until_explicit_removal() {
    native_test_thread::run(|| {
        let (mut page, errors) = fixture();
        evaluate(
            &mut page,
            r#"
            function check(value, message) { if (!value) throw new Error(message); }
            function equal(actual, expected, message) { check(Object.is(actual, expected), message); }
            const element = document.createElement('button'), events = [], authorWrites = [];
            document.body.appendChild(element);
            const nativeHandler = function(event) { events.push([this, event.type]); };
            element.onclick = nativeHandler;
            equal(element.onclick, nativeHandler, 'native handler getter retains function identity');
            let authorState = 'author initial';
            Object.defineProperty(element, 'onclick', {
                configurable: true,
                get() { return authorState; },
                set(value) { authorWrites.push([this, value]); authorState = value; }
            });
            const authorHandler = function() { throw new Error('author expando must not become a native listener'); };
            element.onclick = authorHandler;
            equal(element.onclick, authorHandler, 'own author getter and setter override native handler attribute');
            check(authorWrites.length === 1 && authorWrites[0][0] === element,
                  'own handler setter executes once with receiver');
            element.dispatchEvent(new Event('click'));
            check(events.length === 1 && events[0][0] === element && events[0][1] === 'click',
                  'native handler remains registered behind author shadow');
            delete element.onclick;
            equal(element.onclick, nativeHandler, 'deleting shadow exposes retained native handler');
            element.onclick = null;
            equal(element.onclick, null, 'native null assignment removes handler');
            element.dispatchEvent(new Event('click'));
            equal(events.length, 1, 'removed native handler does not dispatch');
            const text = document.createTextNode('text');
            check(!('onclick' in text), 'CharacterData does not inherit GlobalEventHandlers');
            text.onclick = authorHandler;
            equal(text.onclick, authorHandler, 'unrelated onclick property is an ordinary expando');
        "#,
        );
        assert!(
            errors.borrow().is_empty(),
            "event dispatch reported JavaScript errors: {:?}",
            errors.borrow()
        );
    });
}

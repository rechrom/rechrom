//! Generic registration-time metadata finalization delivered to production Page style/layout.
//! Native HTML style/button nodes keep fragment-parser namespace behavior out of this test.
//! https://html.spec.whatwg.org/multipage/custom-elements.html#dom-customelementregistry-define
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::{Page, ScriptEnvironment};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use layoutng_assembly::fragment_tree::FragmentNode;
use page_mutation::PageMutation;
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
            body: b"<!doctype html><html><head><style>body{margin:0;font:16px/20px sans-serif}</style></head><body><x-finalized-styles id=host data-state=ready></x-finalized-styles></body></html>".to_vec(),
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
        Err(io::Error::other("unexpected custom element fixture XHR"))
    }
}

fn evaluate(page: &mut Page, script: &str) {
    let result = page
        .Evaluate(script, "test:custom-element-style-delivery")
        .unwrap();
    assert!(
        result.Succeeded(),
        "custom element fixture JavaScript failed: {}",
        result
            .exception
            .as_ref()
            .map(|e| e.message.as_str())
            .unwrap_or("")
    );
}

fn fixture() -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(640, 800);
    constraints.fonts = vec![layoutng_assembly::internal::layout_input::FontFace {
        family: "sans-serif".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader)),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        constraints,
        Some(ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_native_stack_budget(
                4 * 1024 * 1024,
            )),
            xhr: Box::new(XHR),
            user_agent: "custom-element-style-delivery-fixture".into(),
        }),
        None,
    );
    page.Open("https://custom-element-style-delivery.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    assert!(page.CurrentFrame().is_some());
    page
}

fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|i| {
            document
                .Node(i)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == name)
                .map(|_| document.Node(i).Id())
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

#[test]
fn registration_finalizes_inherited_styles_before_connected_shadow_rendering() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            function check(value, message) { if (!value) throw new Error(message); }
            function near(actual, expected, message) {
                check(Math.abs(actual - expected) < 0.05,
                      message + ': expected ' + expected + ', got ' + actual);
            }
            var lifecycle = [];
            class RenderBase extends HTMLElement {
                constructor() {
                    super();
                    lifecycle.push('constructor');
                    this.attachShadow({mode: 'open'});
                }
                attributeChangedCallback(name, oldValue, value) {
                    lifecycle.push('attribute:' + name + ':' + value);
                }
                connectedCallback() {
                    lifecycle.push('connected');
                    if (this.shadowRoot.firstChild) return;
                    const style = document.createElement('style');
                    style.id = 'delivered-style';
                    style.appendChild(document.createTextNode(this.constructor.elementStyles.join('\n')));
                    const button = document.createElement('button');
                    button.id = 'delivered-button';
                    button.appendChild(document.createTextNode('Ready'));
                    this.shadowRoot.appendChild(style);
                    this.shadowRoot.appendChild(button);
                }
            }
            RenderBase.elementStyles = [];
            class FinalizedStyles extends RenderBase {
                static get observedAttributes() {
                    lifecycle.push('observed');
                    this.elementStyles = [this.styles];
                    return ['data-state'];
                }
            }
            FinalizedStyles.styles = 'button { display:block; box-sizing:border-box; width:120px; height:40px; padding:0; border:0; margin:0; }';
            var host = document.getElementById('host');
            check(host.isConnected && !(host instanceof FinalizedStyles), 'pre-existing connected undefined host');
            check(FinalizedStyles.elementStyles === RenderBase.elementStyles && FinalizedStyles.elementStyles.length === 0,
                  'subclass initially inherits empty metadata');
            customElements.define('x-finalized-styles', FinalizedStyles);
            check(JSON.stringify(lifecycle) === JSON.stringify(['observed', 'constructor', 'attribute:data-state:ready', 'connected']),
                  'metadata and initial reactions precede connected rendering: ' + JSON.stringify(lifecycle));
            check(host instanceof FinalizedStyles, 'definition upgraded existing host');
            var deliveredStyle = host.shadowRoot.querySelector('style');
            var button = host.shadowRoot.querySelector('button');
            check(deliveredStyle instanceof HTMLStyleElement && button instanceof HTMLButtonElement,
                  'native HTML elements retain their brands');
            check(button.isConnected && button.getRootNode({composed:true}) === document,
                  'shadow descendants use the shadow-including document root');
            check(deliveredStyle.namespaceURI === 'http://www.w3.org/1999/xhtml' && button.namespaceURI === 'http://www.w3.org/1999/xhtml',
                  'style and button use the HTML namespace');
            check(deliveredStyle.textContent === FinalizedStyles.styles, 'finalized CSS reaches shadow style node');
            check(FinalizedStyles.elementStyles.length === 1 && RenderBase.elementStyles.length === 0,
                  'finalization owns subclass metadata');
            var rect = button.getBoundingClientRect();
            near(rect.width, 120, 'shadow button width');
            near(rect.height, 40, 'shadow button height');
            check(getComputedStyle(button).width === '120px', 'computed width receives finalized stylesheet');
            check(getComputedStyle(button).height === '40px', 'computed height receives finalized stylesheet');
        "#,
        );
        let button_id = id(&page, "delivered-button");
        let frame = page.CurrentFrame().expect("production frame is published");
        let button = find(&frame.fragments, button_id).expect("shadow button is published");
        assert_eq!(button.size.width, 120.0);
        assert_eq!(button.size.height, 40.0);
        let style_id = id(&page, "delivered-style");
        assert!(
            find(&frame.fragments, style_id).is_none(),
            "style has no layout box"
        );
    });
}

// Blink EnqueueToBackupQueue posts the backup microtask when the first unscoped
// reaction arrives; a subsequently queued Promise job follows it. The next batch
// must post a fresh backup job after InvokeBackupQueue clears the previous queue.
// https://chromium.googlesource.com/chromium/src/+/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/custom/custom_element_reaction_stack.cc#86
#[test]
fn native_attribute_mutations_schedule_and_rearm_backup_reactions_at_microtask_checkpoints() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            function check(value, message) { if (!value) throw new Error(message); }
            var changes = [], checkpoints = [];
            class NativeObserved extends HTMLElement {
                static get observedAttributes() { return ['data-state']; }
                attributeChangedCallback(name, oldValue, value, namespace) {
                    changes.push([name, oldValue, value, namespace]);
                }
            }
            customElements.define('x-finalized-styles', NativeObserved);
            check(changes.length === 1 && changes[0][2] === 'ready', 'initial attribute callback');
            changes.length = 0;
        "#,
        );
        let host_id = id(&page, "host");
        for value in ["first", "second", "second"] {
            page.Apply(PageMutation::DOMMutation(DOMMutation {
                mutation_type: DOMMutationType::kSetAttribute,
                target_node_id: host_id,
                name: "data-state".into(),
                value: value.into(),
                ..Default::default()
            }))
            .unwrap();
        }
        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kRemoveAttribute,
            target_node_id: host_id,
            name: "data-state".into(),
            ..Default::default()
        }))
        .unwrap();
        evaluate(
            &mut page,
            r#"
            check(changes.length === 0, 'unscoped native mutations wait for microtask checkpoint');
            Promise.resolve().then(() => checkpoints.push(JSON.stringify(changes)));
        "#,
        );
        evaluate(
            &mut page,
            r#"
            var expected = [
                ['data-state', 'ready', 'first', null],
                ['data-state', 'first', 'second', null],
                ['data-state', 'second', 'second', null],
                ['data-state', 'second', null, null]
            ];
            check(JSON.stringify(changes) === JSON.stringify(expected), 'all native mutation argument snapshots are retained: ' + JSON.stringify(changes));
            check(checkpoints.length === 1 && checkpoints[0] === JSON.stringify(expected),
                  'already queued backup reactions run before subsequently queued promise job');
        "#,
        );
        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: host_id,
            name: "data-state".into(),
            value: "again".into(),
            ..Default::default()
        }))
        .unwrap();
        evaluate(
            &mut page,
            r#"
            check(changes.length === 4, 'next native batch remains queued before checkpoint');
            Promise.resolve().then(() => checkpoints.push(JSON.stringify(changes)));
        "#,
        );
        evaluate(
            &mut page,
            r#"
            expected.push(['data-state', null, 'again', null]);
            check(JSON.stringify(changes) === JSON.stringify(expected), 'backup scheduling rearms after previous drain');
            check(checkpoints.length === 2 && checkpoints[1] === JSON.stringify(expected), 'later checkpoint receives exactly one additional callback');
        "#,
        );
    });
}

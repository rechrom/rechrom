//! Hidden presentation hints through production Style, flat-tree attachment and Page.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::{Page, ScriptEnvironment};
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
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
            body: b"<!doctype html><html><head><style>body{margin:0;font:16px/20px sans-serif}</style></head><body></body></html>".to_vec(),
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
        Err(io::Error::other("unexpected shadow fixture XHR"))
    }
}

fn evaluate(page: &mut Page, script: &str) {
    let result = page.Evaluate(script, "test:shadow-publication").unwrap();
    assert!(
        result.Succeeded(),
        "shadow fixture JavaScript failed: {}",
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
    }];
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
            user_agent: "shadow-publication-fixture".into(),
        }),
        None,
    );
    page.Open("https://shadow-publication.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    assert!(page.CurrentFrame().is_some());
    evaluate(
        &mut page,
        r#"
        function check(value, message) { if (!value) throw new Error(message); }
        function near(actual, expected, message) {
            check(Math.abs(actual - expected) < 0.05,
                  message + ': expected ' + expected + ', got ' + actual);
        }
        function element(tag, id, css, parent, text) {
            var node = document.createElement(tag);
            if (id) node.id = id;
            if (css) node.setAttribute('style', css);
            if (text !== undefined) node.appendChild(document.createTextNode(text));
            if (parent) parent.appendChild(node);
            return node;
        }
        function rect(node) {
            var r = node.getBoundingClientRect();
            return [r.x, r.y, r.width, r.height];
        }
        function sameRect(actual, expected, message) {
            for (var i = 0; i < 4; ++i) near(actual[i], expected[i], message + '[' + i + ']');
        }
    "#,
    );
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

fn count(root: &FragmentNode, id: u64) -> usize {
    usize::from(root.node_id == id)
        + root
            .children
            .iter()
            .map(|child| count(child, id))
            .sum::<usize>()
}

fn native_display(page: &Page, name: &str) -> foundation::EDisplay {
    let owner = page.Document();
    let document = owner.GetDocument();
    let index = document.FindNodeById(id(page, name)).unwrap();
    unsafe { &*document.ResolvedStyleFor(index).unwrap().native_style.Get() }.Display()
}

#[test]
fn hidden_slot_suppresses_assigned_subtree_and_retains_control_and_old_frames() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:contents', document.body);
            var button = element('button', 'toggle', 'width:80px;height:20px', host, 'Toggle');
            button.setAttribute('slot', 'button');
            var panel = element('div', 'panel', 'width:120px;height:40px;background:red', host);
            panel.setAttribute('slot', 'panel');
            var input = element('input', 'control', 'width:20px;height:20px', panel);
            input.type = 'checkbox'; input.checked = true; input.value = 'retained';
            var root = host.attachShadow({mode:'open'});
            var buttonSlot = element('slot', 'button-slot', '', root); buttonSlot.name = 'button';
            var slot = element('slot', 'panel-slot', '', root); slot.name = 'panel';
            check(slot.namespaceURI === 'http://www.w3.org/1999/xhtml', 'native HTML slot');
            check(slot.assignedElements()[0] === panel, 'native assignment identity');
        "#,
        );
        assert_eq!(
            native_display(&page, "panel-slot"),
            foundation::EDisplay::kContents
        );
        let panel_id = id(&page, "panel");
        let control_id = id(&page, "control");
        let old = page.CurrentFrame().unwrap().fragments.clone();
        assert_eq!(find(&old, panel_id).unwrap().size.width, 120.0);
        for _ in 0..3 {
            evaluate(
                &mut page,
                "slot.hidden = true; check(slot.hasAttribute('hidden'), 'hidden reflected');",
            );
            assert_eq!(
                native_display(&page, "panel-slot"),
                foundation::EDisplay::kNone,
                "HTML hidden must enter the native computed style before attachment"
            );
            let frame = &page.CurrentFrame().unwrap().fragments;
            assert!(find(frame, panel_id).is_none());
            assert!(find(frame, control_id).is_none());
            assert!(find(frame, id(&page, "toggle")).is_some());
            evaluate(
                &mut page,
                r#"
                near(rect(panel)[2], 0, 'hidden assigned panel geometry');
                check(slot.assignedElements()[0] === panel, 'hiding does not change assignment');
                check(input.checked && input.value === 'retained', 'native control state survives');
                slot.hidden = false;
            "#,
            );
            assert_eq!(
                native_display(&page, "panel-slot"),
                foundation::EDisplay::kContents
            );
            assert_eq!(
                find(&page.CurrentFrame().unwrap().fragments, panel_id)
                    .unwrap()
                    .size
                    .width,
                120.0
            );
            assert_eq!(count(&page.CurrentFrame().unwrap().fragments, panel_id), 1);
        }
        assert_eq!(
            find(&old, panel_id).unwrap().size.width,
            120.0,
            "old published frames are immutable across hide/show"
        );
        evaluate(&mut page, "slot.setAttribute('hidden', 'false');");
        assert!(find(&page.CurrentFrame().unwrap().fragments, panel_id).is_none());
        evaluate(&mut page, "slot.style.display = 'contents';");
        assert_eq!(
            native_display(&page, "panel-slot"),
            foundation::EDisplay::kContents,
            "author style overrides hidden presentational hint"
        );
        assert!(find(&page.CurrentFrame().unwrap().fragments, panel_id).is_some());
    });
}

#[test]
fn hidden_slot_suppresses_fallback_and_ordinary_hidden_subtrees() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:block', document.body);
            var root = host.attachShadow({mode:'closed'});
            var slot = element('slot', 'fallback-slot', '', root);
            var fallback = element('div', 'fallback', 'width:70px;height:25px', slot);
            var ordinary = element('div', 'ordinary', 'width:80px;height:30px', document.body);
            var nested = element('span', 'nested', '', ordinary, 'hidden text');
            slot.hidden = true; ordinary.hidden = true;
        "#,
        );
        for name in ["fallback-slot", "ordinary"] {
            assert_eq!(
                native_display(&page, name),
                foundation::EDisplay::kNone,
                "{name}"
            );
        }
        for name in ["fallback", "ordinary", "nested"] {
            assert!(
                find(&page.CurrentFrame().unwrap().fragments, id(&page, name)).is_none(),
                "{name}"
            );
        }
        evaluate(
            &mut page,
            "slot.removeAttribute('hidden'); ordinary.removeAttribute('hidden');",
        );
        assert_eq!(
            native_display(&page, "fallback-slot"),
            foundation::EDisplay::kContents
        );
        assert_eq!(
            find(
                &page.CurrentFrame().unwrap().fragments,
                id(&page, "fallback")
            )
            .unwrap()
            .size
            .width,
            70.0
        );
        assert_eq!(
            find(
                &page.CurrentFrame().unwrap().fragments,
                id(&page, "ordinary")
            )
            .unwrap()
            .size
            .width,
            80.0
        );
    });
}

//! Author shadow/slot attachment through the production Page and QuickJS lifecycle.
//! Use native HTML createElement nodes: ShadowRoot.innerHTML's fragment namespace
//! behavior is a separate parser concern and must not hide attachment regressions.
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

// Inline owners can legitimately have no separate FragmentNode. Inspect the
// retained native tree when checking uniqueness of the file status span.
fn native_count(page: &Page, id: u64) -> usize {
    use layoutng_assembly::internal::{
        layout_object::LayoutObject, layout_pass_scope::LayoutObjectFactoryScope,
    };
    let assembly = browser::CreateLayoutAssembly();
    let _factories = LayoutObjectFactoryScope::new(&assembly.objects);
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    let mut count = 0;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == id {
            count += 1;
        }
        object = current.NextInPreOrder(root);
    }
    count
}

fn published<'a>(page: &'a Page, name: &str) -> &'a FragmentNode {
    find(&page.CurrentFrame().unwrap().fragments, id(page, name))
        .unwrap_or_else(|| panic!("native DOM node {name} has no published layout fragment"))
}

#[test]
fn open_and_closed_nested_shadow_buttons_publish_native_fragments() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var cases = [];
            for (var mode of ['open', 'closed']) {
                var host = element('x-host', mode + '-host',
                    'display:inline-block;color:rgb(19,37,73)', document.body);
                var root = host.attachShadow({mode:mode});
                var inner = element('x-inner', mode + '-inner', 'display:block', root);
                var nested = inner.attachShadow({mode:mode});
                var button = element('button', mode + '-button',
                    'display:block;box-sizing:border-box;width:120px;height:30px;padding:0;border:0;color:inherit',
                    nested, mode);
                cases.push({host:host, inner:inner, root:root, nested:nested, button:button});
                check((host.shadowRoot === null) === (mode === 'closed'), mode + ' root exposure');
            }
        "#,
        );
        for mode in ["open", "closed"] {
            let button = published(&page, &format!("{mode}-button"));
            assert_eq!(button.size.width, 120.0);
            assert_eq!(button.size.height, 30.0);
            assert_eq!(
                count(
                    &page.CurrentFrame().unwrap().fragments,
                    id(&page, &format!("{mode}-button"))
                ),
                1
            );
            assert!(published(&page, &format!("{mode}-host")).size.width >= 120.0);
        }
        evaluate(
            &mut page,
            r#"
            for (var item of cases) {
                near(rect(item.button)[2], 120, 'shadow button width');
                near(rect(item.button)[3], 30, 'shadow button height');
                check(rect(item.host)[2] >= 120, 'shadow content contributes host width');
            }
        "#,
        );
    });
}

#[test]
fn attaching_empty_shadow_root_detaches_previously_published_light_subtree() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:block', document.body);
            var light = element('div', 'light', 'width:80px;height:20px;background:red', host);
        "#,
        );
        let light_id = id(&page, "light");
        let old = page.CurrentFrame().unwrap().fragments.clone();
        assert_eq!(find(&old, light_id).unwrap().size.width, 80.0);
        evaluate(&mut page, "var root = host.attachShadow({mode:'open'});");
        assert!(
            find(&page.CurrentFrame().unwrap().fragments, light_id).is_none(),
            "attaching even an empty root must evict the old light subtree"
        );
        assert_eq!(
            find(&old, light_id).unwrap().size.width,
            80.0,
            "published old fragments remain immutable after attachment"
        );
        evaluate(
            &mut page,
            r#"
            near(rect(light)[2], 0, 'unassigned light child is excluded');
            var shadow = element('div', 'shadow', 'width:70px;height:25px;background:blue', root);
        "#,
        );
        let shadow_id = id(&page, "shadow");
        assert_eq!(published(&page, "shadow").size.width, 70.0);
        evaluate(&mut page, "root.removeChild(shadow);");
        assert!(find(&page.CurrentFrame().unwrap().fragments, shadow_id).is_none());
        assert!(find(&page.CurrentFrame().unwrap().fragments, light_id).is_none());
    });
}

#[test]
fn slots_publish_only_assigned_or_fallback_children_and_keep_slot_boxes() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:block;width:300px;color:black', document.body);
            var named = element('div', 'named', 'width:70px;height:12px', host);
            named.setAttribute('slot', 'named');
            var ordinary = element('div', 'ordinary', 'width:80px;height:13px', host);
            var text = document.createTextNode('slotted text');
            host.appendChild(text);
            var unmatched = element('div', 'unmatched', 'width:90px;height:14px', host);
            unmatched.setAttribute('slot', 'missing');
            var root = host.attachShadow({mode:'open'});
            var namedSlot = element('slot', 'named-slot', 'display:block;width:180px;color:rgb(19,37,73)', root);
            namedSlot.name = 'named';
            var inactive = element('div', 'inactive-fallback', 'width:99px;height:15px', namedSlot);
            var defaultSlot = element('slot', 'default-slot', 'display:block;width:220px', root);
            var defaultFallback = element('div', 'default-fallback', 'width:101px;height:16px', defaultSlot);
            var duplicate = element('slot', 'duplicate-slot', 'display:block;width:160px', root);
            duplicate.name = 'named';
            var duplicateFallback = element('div', 'duplicate-fallback', 'width:102px;height:17px', duplicate);
            var empty = element('slot', 'empty-slot', 'display:block;width:150px', root);
            empty.name = 'empty';
            var emptyFallback = element('div', 'empty-fallback', 'width:103px;height:18px', empty);
            check(namedSlot.assignedNodes().length === 1 && namedSlot.assignedNodes()[0] === named,
                  'named native assignment');
            check(defaultSlot.assignedNodes().length === 2 && defaultSlot.assignedNodes()[1] === text,
                  'element and text default assignment');
            check(duplicate.assignedNodes().length === 0, 'first slot wins duplicate names');
        "#,
        );
        for name in ["named", "ordinary", "duplicate-fallback", "empty-fallback"] {
            let node_id = id(&page, name);
            assert_eq!(
                count(&page.CurrentFrame().unwrap().fragments, node_id),
                1,
                "active child {name} appears exactly once"
            );
        }
        for name in ["unmatched", "inactive-fallback", "default-fallback"] {
            assert!(
                find(&page.CurrentFrame().unwrap().fragments, id(&page, name)).is_none(),
                "inactive child {name} must not be published"
            );
        }
        for (name, width) in [
            ("named-slot", 180.0),
            ("default-slot", 220.0),
            ("duplicate-slot", 160.0),
            ("empty-slot", 150.0),
        ] {
            assert_eq!(
                published(&page, name).size.width,
                width,
                "a styled slot remains a layout owner rather than being flattened away"
            );
        }
        assert!(
            published(&page, "named").paint.style.color
                == published(&page, "named-slot").paint.style.color,
            "assigned elements inherit from the active slot"
        );
        assert!(
            published(&page, "named").paint.style.color
                != published(&page, "host").paint.style.color
        );
        let text_id = {
            let owner = page.Document();
            let document = owner.GetDocument();
            (0..document.NodeCount())
                .find_map(|i| {
                    (document.Node(i).Data() == "slotted text").then_some(document.Node(i).Id())
                })
                .unwrap()
        };
        assert!(
            find(&page.CurrentFrame().unwrap().fragments, text_id).is_some(),
            "a slotted native text node reaches layout"
        );
        evaluate(
            &mut page,
            r#"
            check(rect(named)[1] < rect(ordinary)[1], 'slot order controls composed layout');
            host.removeChild(named);
            host.removeChild(ordinary);
            host.removeChild(text);
        "#,
        );
        for name in [
            "inactive-fallback",
            "default-fallback",
            "duplicate-fallback",
            "empty-fallback",
        ] {
            assert_eq!(
                count(&page.CurrentFrame().unwrap().fragments, id(&page, name)),
                1,
                "fallback {name} activates when its assignments disappear"
            );
        }
        for name in ["named", "ordinary", "unmatched"] {
            assert!(find(&page.CurrentFrame().unwrap().fragments, id(&page, name)).is_none());
        }
    });
}

#[test]
fn equal_style_slot_redistribution_updates_measurement_before_the_next_frame() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:block;width:240px', document.body);
            var light = element('div', 'light', 'width:40px;height:20px', host);
            light.setAttribute('slot', 'a');
            var sibling = element('div', 'sibling', 'width:40px;height:20px', host);
            sibling.setAttribute('slot', 'a');
            var root = host.attachShadow({mode:'open'});
            var first = element('div', 'first', 'height:80px;width:240px', root);
            var a = element('slot', 'a', 'display:block;width:200px', first); a.name = 'a';
            var second = element('div', 'second', 'height:80px;width:240px', root);
            var b = element('slot', 'b', 'display:block;width:200px', second); b.name = 'b';
        "#,
        );
        let sibling_id = id(&page, "sibling");
        let sibling_client = published(&page, "sibling").paint.display_item_client_id;
        evaluate(
            &mut page,
            r#"
            var before = rect(light);
            near(before[2], 40, 'initial slotted width');
            light.setAttribute('slot', 'b');
            var immediate = rect(light);
            near(immediate[1] - before[1], 80, 'same-task slot attribute redistribution');
            check(b.assignedNodes().indexOf(light) !== -1, 'native slot identity changes immediately');
        "#,
        );
        evaluate(
            &mut page,
            "sameRect(rect(light), immediate, 'post-frame slot redistribution');",
        );
        assert_eq!(id(&page, "sibling"), sibling_id);
        assert_eq!(
            published(&page, "sibling").paint.display_item_client_id,
            sibling_client,
            "unchanged sibling keeps its resident native layout owner"
        );
        evaluate(
            &mut page,
            r#"
            var previous = rect(light);
            b.name = 'renamed';
            near(rect(light)[2], 0, 'slot rename immediately excludes obsolete assignment');
            light.setAttribute('slot', 'renamed');
            var renamed = rect(light);
            sameRect(renamed, previous, 'renamed slot immediately reassigns');
        "#,
        );
        evaluate(
            &mut page,
            "sameRect(rect(light), renamed, 'post-frame renamed slot');",
        );
        evaluate(
            &mut page,
            r#"
            light.setAttribute('slot', 'a');
            var orderBefore = rect(light);
            near(rect(sibling)[1] - orderBefore[1], 20, 'initial assigned order');
            host.insertBefore(sibling, light);
            var reordered = rect(light);
            near(reordered[1] - orderBefore[1], 20, 'same-task host child reorder');
            near(rect(sibling)[1], orderBefore[1], 'reordered first child');
            check(a.assignedNodes()[0] === sibling && a.assignedNodes()[1] === light,
                  'assignment order follows light DOM');
        "#,
        );
        evaluate(
            &mut page,
            "sameRect(rect(light), reordered, 'post-frame assigned order');",
        );
        evaluate(
            &mut page,
            r#"
            host.removeChild(light);
            near(rect(light)[2], 0, 'removal excludes immediately');
            host.appendChild(light);
            sameRect(rect(light), reordered, 'reinsertion restores immediately');
            b.name = 'a';
            root.insertBefore(second, first);
            check(b.assignedNodes().indexOf(light) !== -1, 'slot tree order selects the first matching slot');
            var newFirst = rect(light);
            near(newFirst[1], reordered[1], 'new first wrapper receives assigned children');
        "#,
        );
        evaluate(
            &mut page,
            "sameRect(rect(light), newFirst, 'post-frame slot tree reorder');",
        );
    });
}

#[test]
fn manual_slot_assignment_moves_and_reorders_the_published_native_children() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:block;width:240px', document.body);
            var light = element('div', 'light', 'width:40px;height:20px', host);
            var other = element('div', 'other', 'width:40px;height:20px', host);
            var root = host.attachShadow({mode:'closed', slotAssignment:'manual'});
            var first = element('div', 'first', 'height:80px;width:240px', root);
            var a = element('slot', 'a', 'display:block;width:200px', first);
            var second = element('div', 'second', 'height:80px;width:240px', root);
            var b = element('slot', 'b', 'display:block;width:200px', second);
            a.assign(light, other);
        "#,
        );
        assert_eq!(
            count(&page.CurrentFrame().unwrap().fragments, id(&page, "light")),
            1
        );
        evaluate(
            &mut page,
            r#"
            var before = rect(light);
            near(before[2], 40, 'manual assignment publishes content');
            b.assign(other, light);
            var immediate = rect(light);
            near(immediate[1] - before[1], 100, 'manual redistribution and argument order');
            check(a.assignedNodes().length === 0, 'manual assignment removes previous ownership');
            check(b.assignedNodes()[0] === other && b.assignedNodes()[1] === light,
                  'manual argument order is retained');
        "#,
        );
        evaluate(
            &mut page,
            "sameRect(rect(light), immediate, 'post-frame manual assignment');",
        );
        for name in ["light", "other"] {
            assert_eq!(
                count(&page.CurrentFrame().unwrap().fragments, id(&page, name)),
                1
            );
        }
        evaluate(
            &mut page,
            "b.assign(); near(rect(light)[2], 0, 'cleared manual assignment');",
        );
        for name in ["light", "other"] {
            assert!(find(&page.CurrentFrame().unwrap().fragments, id(&page, name)).is_none());
        }
    });
}

#[test]
fn shadow_style_width_and_text_mutations_preserve_older_published_frames() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:block;color:rgb(19,37,73)', document.body);
            var root = host.attachShadow({mode:'open'});
            var box = element('div', 'box', 'width:80px;height:24px;background:red', root, 'alpha');
            var sibling = element('div', 'sibling', 'width:50px;height:10px', root);
        "#,
        );
        let box_id = id(&page, "box");
        let old_fragments = page.CurrentFrame().unwrap().fragments.clone();
        let old_artifact = page.CurrentFrame().unwrap().display_items.clone();
        let old_items = old_artifact.items.as_ref().clone();
        let old_box_paint = published(&page, "box").paint.clone();
        let old_box_size = published(&page, "box").size;
        let old_glyphs = glyph_ids(published(&page, "box"));
        assert!(!old_glyphs.is_empty());
        let box_client = published(&page, "box").paint.display_item_client_id;
        let sibling_client = published(&page, "sibling").paint.display_item_client_id;
        assert_eq!(old_box_size.width, 80.0);
        assert!(
            published(&page, "box").paint.style.color == published(&page, "host").paint.style.color,
            "host inheritance crosses its shadow root"
        );
        evaluate(
            &mut page,
            "box.style.backgroundColor = 'blue'; box.style.color = 'green';",
        );
        assert_eq!(
            published(&page, "box").size,
            old_box_size,
            "paint-only shadow mutations preserve geometry"
        );
        assert_eq!(
            published(&page, "box").paint.display_item_client_id,
            box_client
        );
        assert!(
            published(&page, "box").paint.style.background_color
                != old_box_paint.style.background_color
        );
        assert!(
            page.CurrentFrame().unwrap().display_items.items.as_ref() != &old_items,
            "shadow paint mutation reaches the published artifact"
        );
        assert!(
            find(&old_fragments, box_id).unwrap().paint == old_box_paint,
            "old fragment paint properties stay immutable"
        );
        assert!(
            old_artifact.items.as_ref() == &old_items,
            "old display items stay immutable"
        );
        evaluate(
            &mut page,
            r#"
            box.style.width = '160px';
            near(rect(box)[2], 160, 'same-task shadow width mutation');
            box.firstChild.textContent = 'beta';
        "#,
        );
        assert_eq!(published(&page, "box").size.width, 160.0);
        assert_ne!(
            glyph_ids(published(&page, "box")),
            old_glyphs,
            "mutated shadow text reaches shaping and publication"
        );
        assert_eq!(
            glyph_ids(find(&old_fragments, box_id).unwrap()),
            old_glyphs,
            "old text glyphs stay immutable"
        );
        assert_eq!(find(&old_fragments, box_id).unwrap().size.width, 80.0);
        assert!(find(&old_fragments, box_id).unwrap().paint == old_box_paint);
        assert!(old_artifact.items.as_ref() == &old_items);
        assert_eq!(
            published(&page, "sibling").paint.display_item_client_id,
            sibling_client,
            "unmodified sibling remains resident across shadow changes"
        );
        let sequence = page.CurrentFrame().unwrap().sequence;
        let retained = page.CurrentFrame().unwrap().fragments.clone();
        evaluate(&mut page, "void 0;");
        assert_eq!(
            page.CurrentFrame().unwrap().sequence,
            sequence,
            "no-op script does not republish"
        );
        assert!(Rc::ptr_eq(
            &page.CurrentFrame().unwrap().fragments,
            &retained
        ));
        evaluate(&mut page, "host.style.display = 'none';");
        assert!(find(&page.CurrentFrame().unwrap().fragments, box_id).is_none());
        evaluate(&mut page, "host.style.display = 'block';");
        assert_eq!(published(&page, "box").size.width, 160.0);
        evaluate(
            &mut page,
            "host.style.display = 'contents'; near(rect(box)[2], 160, 'display contents keeps shadow child');",
        );
        assert!(find(&page.CurrentFrame().unwrap().fragments, id(&page, "host")).is_none());
        assert_eq!(published(&page, "box").size.width, 160.0);
        evaluate(
            &mut page,
            "document.body.removeChild(host); near(rect(box)[2], 0, 'detached shadow subtree');",
        );
        assert!(find(&page.CurrentFrame().unwrap().fragments, box_id).is_none());
        evaluate(
            &mut page,
            "document.body.appendChild(host); near(rect(box)[2], 160, 'reattached shadow subtree');",
        );
        assert_eq!(published(&page, "box").size.width, 160.0);
        assert_eq!(find(&old_fragments, box_id).unwrap().size.width, 80.0);
    });
}

fn glyph_ids(root: &FragmentNode) -> Vec<u32> {
    root.paint
        .glyph_runs
        .iter()
        .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.id))
        .chain(root.children.iter().flat_map(glyph_ids))
        .collect()
}

#[test]
fn ua_control_attachment_owners_remain_single_and_stable_with_author_shadow() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var controls = [];
            var controlCSS = 'display:block;box-sizing:border-box;width:360px;height:36px;margin:0;font:16px/20px sans-serif';
            for (var type of ['file', 'text', 'search', 'range', 'color']) {
                var input = element('input', type, controlCSS, null);
                input.type = type;
                if (type === 'text' || type === 'search') input.value = type + ' value';
                if (type === 'range') input.value = '25';
                if (type === 'color') input.value = '#123456';
                document.body.appendChild(input);
                controls.push(input);
            }
            var area = element('textarea', 'textarea', controlCSS, document.body);
            area.value = 'area value'; controls.push(area);
            var select = element('select', 'select', controlCSS, document.body);
            var option = element('option', 'option', '', select, 'First choice');
            option.setAttribute('value', 'one'); controls.push(select);
            var progress = element('progress', 'progress', controlCSS, document.body); progress.setAttribute('value', '0.6');
            controls.push(progress);
            var originalRects = controls.map(rect);
            for (var i = 0; i < controls.length; ++i) {
                check(originalRects[i][2] > 0 && originalRects[i][3] > 0,
                      controls[i].id + ' has control geometry');
            }
        "#,
        );
        let names = [
            "file", "text", "search", "textarea", "range", "select", "color", "progress",
        ];
        let original: Vec<_> = names
            .iter()
            .map(|name| {
                let fragment = published(&page, name);
                (
                    fragment.node_id,
                    fragment.size,
                    fragment.paint.display_item_client_id,
                )
            })
            .collect();
        let (button_id, status_id, native_editors) = {
            let owner = page.Document();
            let document = owner.GetDocument();
            let file = document.FindNodeById(id(&page, "file")).unwrap();
            let button_id = document
                .Node(document.FileUploadButtonElement(file).unwrap())
                .Id();
            let status_id = document
                .Node(document.FileStatusElement(file).unwrap())
                .Id();
            let native_editors: Vec<_> = ["text", "search", "textarea"]
                .iter()
                .map(|name| {
                    let host = document.FindNodeById(id(&page, name)).unwrap();
                    document
                        .Node(document.TextControlInnerEditorElement(host).unwrap())
                        .Id()
                })
                .collect();
            (button_id, status_id, native_editors)
        };
        for _ in 0..2 {
            let fragments = &page.CurrentFrame().unwrap().fragments;
            assert_eq!(
                count(fragments, button_id),
                1,
                "UA file button is emitted exactly once"
            );
            assert_eq!(
                native_count(&page, status_id),
                1,
                "UA file status retains exactly one native layout owner"
            );
            for (name, native_editor) in ["text", "search", "textarea"].iter().zip(&native_editors)
            {
                assert_eq!(
                    native_count(&page, id(&page, name) | (1 << 54)),
                    1,
                    "{name} retains one specialized editor"
                );
                assert_eq!(
                    native_count(&page, id(&page, name) | (1 << 53)),
                    1,
                    "{name} retains one specialized value text owner"
                );
                assert_eq!(
                    count(fragments, *native_editor),
                    0,
                    "generic UA traversal must not duplicate {name}'s specialized editor"
                );
            }
            for (name, &(node_id, size, client)) in names.iter().zip(&original) {
                let fragment = published(&page, name);
                assert_eq!(count(fragments, node_id), 1, "one control box for {name}");
                assert_eq!(fragment.size, size, "stable {name} control dimensions");
                assert_eq!(
                    fragment.paint.display_item_client_id, client,
                    "stable {name} layout owner"
                );
            }
            evaluate(
                &mut page,
                r#"
                if (typeof authorHost === 'undefined') {
                    var authorHost = element('x-host', 'author-host', 'display:block', document.body);
                    var authorRoot = authorHost.attachShadow({mode:'open'});
                    var authorBox = element('div', 'author-box', 'width:60px;height:20px', authorRoot);
                } else {
                    authorBox.style.width = '90px';
                }
                for (var i = 0; i < controls.length; ++i)
                    sameRect(rect(controls[i]), originalRects[i], controls[i].id + ' after author attachment');
                check(document.getElementById('text').value === 'text value', 'text value preserved');
                check(document.getElementById('search').value === 'search value', 'search value preserved');
                check(area.value === 'area value', 'textarea value preserved');
                check(document.getElementById('range').value === '25', 'range value preserved');
                check(document.getElementById('color').value === '#123456', 'color value preserved');
                check(select.value === 'one', 'select value preserved');
                near(Number(progress.getAttribute('value')), 0.6, 'progress value preserved');
            "#,
            );
        }
        assert_eq!(published(&page, "author-box").size.width, 90.0);
    });
}

#[test]
fn slot_first_letter_uses_the_active_assigned_or_fallback_text() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host = element('x-host', 'host', 'display:block', document.body);
            var assigned = document.createTextNode('Active');
            host.appendChild(assigned);
            var root = host.attachShadow({mode:'open'});
            element('style', '', '', root, 'slot::first-letter { font-size:32px; color:red; }');
            var slot = element('slot', 'slot', 'display:block;width:200px', root);
            var fallback = element('span', 'fallback', '', slot, 'Zebra');
        "#,
        );
        let first_letter_id = id(&page, "slot") | (1 << 45);
        let old = page.CurrentFrame().unwrap().fragments.clone();
        let active = find(&old, first_letter_id).expect("slot first-letter is published");
        let active_glyphs: Vec<_> = active
            .paint
            .glyph_runs
            .iter()
            .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.id))
            .collect();
        assert_eq!(active_glyphs.len(), 1, "one active first-letter glyph");
        assert!(active
            .paint
            .glyph_runs
            .iter()
            .all(|run| run.font_size == 32.0));
        assert!(
            find(&old, id(&page, "fallback")).is_none(),
            "inactive fallback is excluded"
        );
        evaluate(&mut page, "host.removeChild(assigned);");
        let fallback = find(&page.CurrentFrame().unwrap().fragments, first_letter_id)
            .expect("fallback first-letter is published");
        let fallback_glyphs: Vec<_> = fallback
            .paint
            .glyph_runs
            .iter()
            .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.id))
            .collect();
        assert_eq!(fallback_glyphs.len(), 1);
        assert_ne!(
            fallback_glyphs, active_glyphs,
            "active A changes to fallback Z"
        );
        assert!(fallback
            .paint
            .glyph_runs
            .iter()
            .all(|run| run.font_size == 32.0));
        assert!(find(
            &page.CurrentFrame().unwrap().fragments,
            id(&page, "fallback")
        )
        .is_some());
        let held_glyphs: Vec<_> = find(&old, first_letter_id)
            .unwrap()
            .paint
            .glyph_runs
            .iter()
            .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.id))
            .collect();
        assert_eq!(
            held_glyphs, active_glyphs,
            "old first-letter snapshot stays immutable"
        );
    });
}

#[test]
fn slotted_whitespace_only_nodes_do_not_create_flex_or_grid_items() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            for (var display of ['flex', 'grid']) {
                var host = element('x-host', display + '-host',
                    'display:' + display + ';width:100px;grid-template-columns:40px 40px', document.body);
                var first = element('div', display + '-first', 'width:40px;height:20px', host);
                host.appendChild(document.createTextNode('\n \t '));
                var second = element('div', display + '-second', 'width:40px;height:20px', host);
                var root = host.attachShadow({mode:'open'});
                var slot = element('slot', display + '-slot', 'display:contents', root);
                check(slot.assignedNodes().length === 3, display + ' includes native whitespace assignment');
                near(rect(second)[0] - rect(first)[0], 40, display + ' has only two layout items');
                near(rect(second)[1], rect(first)[1], display + ' items share their row');
            }
        "#,
        );
        let whitespace_ids = {
            let owner = page.Document();
            let document = owner.GetDocument();
            (0..document.NodeCount())
                .filter_map(|i| {
                    (document.Node(i).Data() == "\n \t ").then_some(document.Node(i).Id())
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(whitespace_ids.len(), 2);
        for whitespace in whitespace_ids {
            assert!(
                find(&page.CurrentFrame().unwrap().fragments, whitespace).is_none(),
                "whitespace suppression follows the effective flat-tree layout parent"
            );
        }
        for display in ["flex", "grid"] {
            assert_eq!(
                count(
                    &page.CurrentFrame().unwrap().fragments,
                    id(&page, &format!("{display}-first"))
                ),
                1
            );
            assert_eq!(
                count(
                    &page.CurrentFrame().unwrap().fragments,
                    id(&page, &format!("{display}-second"))
                ),
                1
            );
            assert!(
                find(
                    &page.CurrentFrame().unwrap().fragments,
                    id(&page, &format!("{display}-slot"))
                )
                .is_none(),
                "display:contents slot has no box of its own"
            );
        }
    });
}

#[test]
#[ignore = "pre-existing meter UA pseudo selector calls pending GetComputedStyle collaborator"]
fn meter_ua_selector_collaborator_diagnostic() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(&mut page, "element('meter', 'meter', 'width:100px;height:20px', document.body).setAttribute('value', '0.4');");
        assert!(published(&page, "meter").size.width > 0.0);
    });
}

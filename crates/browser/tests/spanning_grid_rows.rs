//! Spanning named grid rows through the production Style and Layout owners.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use layoutng_assembly::{
    fragment_tree::FragmentNode,
    internal::layout_input::{FontFace, Offset},
};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Operation(Option<URLResponse>);
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct FixtureLoader(Vec<u8>);
impl URLLoader for FixtureLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: self.0.clone(),
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
        Err(io::Error::other("unexpected fixture XHR"))
    }
}

fn fixture(css: &str, children: &str) -> Page {
    let html = format!("<!doctype html><style>html{{font-size:16px}}body{{margin:0}}#grid{{display:grid;width:1180px;{css}}}</style><div id=grid>{children}</div>");
    let assembly = browser::CreateLayoutAssembly();
    let mut space = browser::CreateBrowserConstraints(1180, 757);
    space.fonts = vec![FontFace {
        family: "ButtonFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }];
    let mut page = Page::Create(
        Rc::new(RefCell::new(FixtureLoader(html.into_bytes()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        space,
        Some(browser::page::ScriptEnvironment {
            runtime: Box::new(
                javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_native_stack_budget(
                    4 * 1024 * 1024,
                ),
            ),
            xhr: Box::new(XHR),
            user_agent: "button-fixture".into(),
        }),
        None,
    );
    page.Open("https://page.test/rows.html", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() {
            break;
        }
    }
    assert!(!page.IsLoading());
    page
}

fn find_fragment<'a>(
    node: &'a FragmentNode,
    parent: Offset,
    predicate: &impl Fn(&FragmentNode) -> bool,
) -> Option<(&'a FragmentNode, Offset)> {
    let absolute = Offset {
        x: parent.x + node.offset.x,
        y: parent.y + node.offset.y,
    };
    if predicate(node) {
        return Some((node, absolute));
    }
    node.children
        .iter()
        .find_map(|child| find_fragment(child, absolute, predicate))
}

fn item(page: &Page, name: &str) -> (Offset, layoutng_assembly::internal::layout_input::Size) {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let index = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == name)
        })
        .unwrap();
    let id = doc.Node(index).Id();
    let frame = page.CurrentFrame().unwrap();
    let (fragment, offset) =
        find_fragment(&frame.fragments, Offset::default(), &|f| f.node_id == id).unwrap();
    (offset, fragment.size)
}

fn native(
    page: &Page,
    name: &str,
) -> foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle> {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let index = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == name)
        })
        .unwrap();
    doc.ResolvedStyleFor(index).unwrap().native_style.clone()
}

#[test]
fn spanning_auto_rows_use_only_the_unmet_item_contribution() {
    native_test_thread::run(|| {
        for css in ["grid-template-areas: \"a c\" \"b c\"", ""] {
            let page = fixture(&format!("grid-template-columns:200px 200px;{css}"),
                "<div id=a style='grid-column:1;grid-row:1'><div style='height:260px'></div></div><div id=b style='grid-column:1;grid-row:2'><div style='height:90px'></div></div><div id=c style='grid-column:2;grid-row:1 / 3;max-height:320px;overflow:hidden'><div style='height:400px'></div></div>");
            eprintln!(
                "grid={:?} a={:?} b={:?} c={:?}",
                item(&page, "grid"),
                item(&page, "a"),
                item(&page, "b"),
                item(&page, "c")
            );
            assert_eq!(item(&page, "a").1.height, 260.0);
            assert_eq!(item(&page, "b").0.y, 260.0);
            assert_eq!(item(&page, "grid").1.height, 350.0);
        }
    });
}

#[test]
fn named_area_spanning_auto_rows_match_explicit_lines() {
    native_test_thread::run(|| {
        let page = fixture("grid-template-columns:200px 200px;grid-template-areas:\"a c\" \"b c\"",
            "<div id=a style='grid-area:a'><div style='height:260px'></div></div><div id=b style='grid-area:b'><div style='height:90px'></div></div><div id=c style='grid-area:c;max-height:320px;overflow:hidden'><div style='height:400px'></div></div>");
        eprintln!(
            "named grid={:?} a={:?} b={:?} c={:?}",
            item(&page, "grid"),
            item(&page, "a"),
            item(&page, "b"),
            item(&page, "c")
        );
        assert_eq!(item(&page, "a").1.height, 260.0);
        assert_eq!(item(&page, "b").0.y, 260.0);
        assert_eq!(item(&page, "grid").1.height, 350.0);
    });
}

#[test]
fn named_spans_reach_existing_auto_and_intrinsic_row_sizing() {
    native_test_thread::run(|| {
        for rows in [
            "auto auto",
            "min-content min-content",
            "minmax(auto,min-content) minmax(auto,min-content)",
            "minmax(0,max-content) minmax(0,max-content)",
        ] {
            let page = fixture(&format!("grid-template-columns:200px 200px;grid-template-rows:{rows};grid-template-areas:\"a c\" \"b c\""),
                "<div id=a style='grid-area:a'><div style='height:260px'></div></div><div id=b style='grid-area:b'><div style='height:90px'></div></div><div id=c style='grid-area:c;max-height:320px;overflow:hidden'><div style='height:400px'></div></div>");
            assert_eq!(item(&page, "a").1.height, 260.0, "{rows}");
            assert_eq!(item(&page, "b").0.y, 260.0, "{rows}");
            assert_eq!(item(&page, "grid").1.height, 350.0, "{rows}");
        }
    });
}

#[test]
fn axis_shorthands_resolve_both_named_spanning_edges() {
    native_test_thread::run(|| {
        let page = fixture("grid-template-columns:100px 100px 100px;grid-template-areas:\"a c c\" \"b c c\"",
            "<div id=a style='grid-area:a'><div style='height:260px'></div></div><div id=b style='grid-area:b'><div style='height:90px'></div></div><div id=c style='grid-row:c;grid-column:c;max-height:320px;overflow:hidden'><div style='height:400px'></div></div>");
        let style = native(&page, "c");
        let style = unsafe { &*style.Get() };
        assert!(style.GridRowEnd().IsNamedGridArea());
        assert!(style.GridColumnEnd().IsNamedGridArea());
        assert_eq!(item(&page, "a").1.height, 260.0);
        assert_eq!(item(&page, "b").0.y, 260.0);
        assert_eq!(item(&page, "c").0.x, 100.0);
        assert_eq!(item(&page, "c").1.width, 200.0);
        assert_eq!(item(&page, "grid").1.height, 350.0);
    });
}

#[test]
fn custom_property_span_mutation_preserves_old_style_and_fragment_snapshots() {
    native_test_thread::run(|| {
        let mut page = fixture("grid-template-columns:200px 200px;grid-template-areas:\"a c\" \"b c\"",
            "<div id=a style='grid-area:a'><div style='height:260px'></div></div><div id=b style='grid-area:b'><div style='height:90px'></div></div><div id=c style='--area:c;grid-area:var(--area);max-height:320px;overflow:hidden'><div style='height:400px'></div></div>");
        let old = page.CurrentFrame().unwrap().fragments.clone();
        let old_style = native(&page, "c");
        assert_eq!(item(&page, "a").1.height, 260.0);
        for (value, height, named) in [
            ("c / c / auto / auto", 320.0, false),
            ("c", 260.0, true),
            ("c / c / auto / auto", 320.0, false),
            ("c", 260.0, true),
        ] {
            let result = page
                .Evaluate(
                    &format!("document.getElementById('c').setAttribute('style','--area:{value};grid-area:var(--area);max-height:320px;overflow:hidden')"),
                    "test:grid-span",
                )
                .unwrap();
            assert!(result.Succeeded(), "{:?}", result.exception);
            assert_eq!(item(&page, "a").1.height, height);
            assert_eq!(item(&page, "b").0.y, height);
            assert_eq!(
                unsafe { &*native(&page, "c").Get() }
                    .GridRowEnd()
                    .IsNamedGridArea(),
                named
            );
        }
        let owner = page.Document();
        let doc = owner.GetDocument();
        let id = (0..doc.NodeCount())
            .find(|&i| {
                doc.Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "a")
            })
            .map(|i| doc.Node(i).Id())
            .unwrap();
        let (old_item, old_offset) =
            find_fragment(&old, Offset::default(), &|f| f.node_id == id).unwrap();
        assert_eq!(old_offset.y, 0.0);
        assert_eq!(old_item.size.height, 260.0);
        assert!(unsafe { &*old_style.Get() }.GridRowEnd().IsNamedGridArea());
        assert_eq!(
            unsafe { &*old_style.Get() }
                .GridRowEnd()
                .NamedGridLine()
                .Utf8(),
            "c"
        );
    });
}

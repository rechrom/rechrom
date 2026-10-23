//! UA button alignment must survive the canonical cascade and subsequent
//! author/style mutations. Check both ComputedStyle and published geometry.
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
    let html = format!("<!doctype html><style>body{{margin:0}}#columns{{width:220px;column-count:2;column-gap:20px;{css}}}.item{{height:20px;background:red}}</style><div id=columns>{children}</div>");
    let assembly = browser::CreateLayoutAssembly();
    let mut space = browser::CreateBrowserConstraints(640, 480);
    space.fonts = vec![FontFace {
        family: "ButtonFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
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
    page.Open("https://page.test/columns.html", 16384, 4096)
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
const ITEMS: &str = "<div class=item id=a></div><div class=item id=b></div><div class=item id=c></div><div class=item id=d></div>";
#[test]
fn balanced_columns_publish_real_column_geometry() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture("", ITEMS);
        assert_eq!(item(&page, "columns").1.height, 40.0);
        for (id, x, y) in [
            ("a", 0., 0.),
            ("b", 0., 20.),
            ("c", 120., 0.),
            ("d", 120., 20.),
        ] {
            let (offset, size) = item(&page, id);
            assert_eq!(offset, Offset { x, y }, "{id}");
            assert_eq!(size.width, 100., "{id}");
            assert_eq!(size.height, 20., "{id}");
        }
    });
}

fn column_fragments(page: &Page) -> &FragmentNode {
    let owner = page.Document();
    let doc = owner.GetDocument();
    let i = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == "columns")
        })
        .unwrap();
    let id = doc.Node(i).Id();
    find_fragment(
        &page.CurrentFrame().unwrap().fragments,
        Offset::default(),
        &|f| f.node_id == id,
    )
    .unwrap()
    .0
}

#[test]
fn column_width_and_count_resolve_against_available_inline_size() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for css in [
            "column-count:auto;column-width:100px",
            "column-count:4;column-width:100px",
        ] {
            let page = fixture(css, ITEMS);
            assert_eq!(item(&page, "c").0, Offset { x: 120., y: 0. });
            assert_eq!(item(&page, "columns").1.height, 40.);
        }
    });
}

#[test]
fn empty_multicol_retains_one_anonymous_fragmentainer() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            "column-fill:auto;height:100px;width:210px;column-gap:10px",
            "",
        );
        let container = column_fragments(&page);
        assert_eq!(container.size.height, 100.);
        assert_eq!(container.children.len(), 1);
        assert_eq!(container.children[0].node_id, 0);
        assert_eq!(container.children[0].size.width, 100.);
        assert_eq!(container.children[0].size.height, 100.);
        assert!(container.children[0].children.is_empty());
    });
}

#[test]
fn fixed_height_fragments_one_block_and_allows_overflow_columns() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (height, expected) in [(150., vec![100., 50.]), (250., vec![100., 100., 50.])] {
            let page = fixture(
                "column-fill:auto;height:100px;width:210px;column-gap:10px",
                &format!("<div style='width:75%;height:{height}px'></div>"),
            );
            let container = column_fragments(&page);
            assert_eq!(container.children.len(), expected.len());
            for (i, (column, height)) in container.children.iter().zip(expected).enumerate() {
                assert_eq!(
                    column.offset,
                    Offset {
                        x: i as f64 * 110.,
                        y: 0.
                    }
                );
                assert_eq!(column.size.width, 100.);
                assert_eq!(column.size.height, 100.);
                assert_eq!(column.children[0].size.width, 75.);
                assert_eq!(column.children[0].size.height, height);
            }
        }
    });
}

#[test]
fn balancing_honors_box_edges_and_min_max_height() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (constraint, container_height, column_height, count) in [
            ("", 60., 50., 3),
            ("min-height:70px", 80., 50., 3),
            ("max-height:40px", 50., 40., 4),
        ] {
            let page=fixture(&format!("column-count:3;column-gap:10px;width:320px;border:3px solid;padding:2px;{constraint}"), "<div style='width:30px;height:150px'></div>");
            let container = column_fragments(&page);
            assert_eq!(container.size.height, container_height, "{constraint}");
            assert_eq!(container.children.len(), count, "{constraint}");
            for (i, column) in container.children.iter().enumerate() {
                assert_eq!(
                    column.offset,
                    Offset {
                        x: 5. + i as f64 * 110.,
                        y: 5.
                    }
                );
                assert_eq!(column.size.height, column_height, "{constraint}");
            }
        }
    });
}

#[test]
fn balancing_stretches_for_unbreakable_children_and_forced_breaks() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page=fixture("", "<div style='height:30px;break-inside:avoid'></div><div style='height:30px;break-inside:avoid'></div><div style='height:30px;break-inside:avoid'></div>");
        assert_eq!(item(&page, "columns").1.height, 60.);
        let page = fixture(
            "",
            "<div style='height:30px;break-after:column'></div><div style='height:50px'></div>",
        );
        assert_eq!(item(&page, "columns").1.height, 50.);
        assert_eq!(column_fragments(&page).children.len(), 2);
    });
}

#[test]
fn columns_use_logical_direction_and_subpixel_progression() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture("direction:rtl", ITEMS);
        assert_eq!(item(&page, "a").0, Offset { x: 120., y: 0. });
        assert_eq!(item(&page, "c").0, Offset { x: 0., y: 0. });
        let page = fixture(
            "width:100px;column-count:3;column-gap:0",
            "<div style='height:90px'></div>",
        );
        let columns = &column_fragments(&page).children;
        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].offset.x, 0.);
        assert_eq!(columns[1].offset.x, 33.328125);
        assert_eq!(columns[2].offset.x, 66.671875);
    });
}

#[test]
fn inline_content_fragments_in_columns() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = fixture(
            "font:16px/20px ButtonFixture",
            "one<br>two<br>three<br>four",
        );
        let columns = &column_fragments(&page).children;
        assert_eq!(columns.len(), 2);
        assert_eq!(item(&page, "columns").1.height, 40.);
        assert!(columns.iter().all(|c| !c.children.is_empty()));
    });
}

#[test]
fn column_layout_survives_style_content_and_width_mutations() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = fixture("", ITEMS);
        assert_eq!(item(&page, "c").0, Offset { x: 120., y: 0. });
        assert!(page
            .Evaluate(
                "document.getElementById('columns').style.columnCount='1'",
                "test:columns"
            )
            .unwrap()
            .Succeeded());
        page.RunTask().unwrap();
        assert_eq!(item(&page, "c").0, Offset { x: 0., y: 40. });
        assert_eq!(item(&page, "columns").1.height, 80.);
        assert!(page.Evaluate("var c=document.getElementById('columns');c.style.columnCount='auto';c.style.columnWidth='100px';c.style.width='220px';document.getElementById('d').style.height='40px'", "test:columns").unwrap().Succeeded());
        page.RunTask().unwrap();
        page.RunTask().unwrap();
        assert_eq!(item(&page, "columns").1.width, 220.);
        assert_eq!(column_fragments(&page).children.len(), 2);
        assert!(page
            .Evaluate(
                "document.getElementById('columns').style.width='100px'",
                "test:columns"
            )
            .unwrap()
            .Succeeded());
        page.RunTask().unwrap();
        assert_eq!(column_fragments(&page).children.len(), 1);
        assert_eq!(item(&page, "columns").1.height, 100.);
    });
}

#[test]
fn untranslated_column_branches_fail_explicitly() {
    native_test_thread::run(|| {
        for (css, children, reason) in [
            ("column-rule:1px solid red", ITEMS, "gap decorations"),
            ("column-wrap:wrap", ITEMS, "column wrapping"),
            (
                "",
                "<div style='column-span:all'>spanner</div>",
                "column spanners",
            ),
            (
                "",
                "<div style='column-count:2'>nested</div>",
                "nested fragmentation",
            ),
            (
                "",
                "<div style='position:absolute'>positioned</div>",
                "out-of-flow",
            ),
        ] {
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _heap = foundation::LayoutHeapScope::new();
                fixture(css, children)
            }));
            let payload = match failure {
                Err(value) => value,
                Ok(_) => panic!("{reason} silently accepted"),
            };
            let failure = payload
                .downcast_ref::<foundation::UnsupportedLayout>()
                .expect("typed capability error");
            assert!(failure.to_string().contains(reason), "{failure}");
        }
    });
}

#[test]
fn column_intrinsic_width_and_percentage_gaps_use_native_values() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        for (css, width) in [
            ("display:inline-block;width:auto", 220.),
            ("display:inline-block;width:auto;column-width:120px", 260.),
        ] {
            let page = fixture(css, "<div style='width:100px;height:80px'></div>");
            assert_eq!(item(&page, "columns").1.width, width);
        }
        let page = fixture("column-gap:10%", ITEMS);
        assert_eq!(item(&page, "a").1.width, 99.);
        assert_eq!(item(&page, "c").0.x, 121.);
    });
}

#[test]
fn ordinary_list_content_keeps_ids_across_fragmentainers() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page=fixture("font:16px/20px ButtonFixture", "<ul style='list-style:none;margin:0;padding:0'><li id=a>One</li><li id=b>Two</li><li id=c>Three</li><li id=d>Four</li></ul>");
        assert_eq!(item(&page, "a").0, Offset { x: 0., y: 0. });
        assert_eq!(item(&page, "c").0, Offset { x: 120., y: 0. });
        let columns = &column_fragments(&page).children;
        assert_eq!(columns.len(), 2);
        assert_ne!(
            columns[0].fragmentainer_instance_id,
            columns[1].fragmentainer_instance_id
        );
    });
}

#[test]
fn formatting_context_change_rechecks_real_spanner_candidates() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page=fixture("", "<div id=fc style='display:flow-root'><div style='column-span:all;height:40px'></div></div>");
        assert!(page.CurrentFrame().is_some());
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            page.Evaluate(
                "document.getElementById('fc').style.display='block'",
                "test:spanner-admission",
            )
        }));
        let payload = match failure {
            Err(value) => value,
            Ok(_) => panic!("real spanner silently accepted"),
        };
        let error = payload
            .downcast_ref::<foundation::UnsupportedLayout>()
            .expect("typed capability error");
        assert!(error.to_string().contains("column spanners"), "{error}");
    });
}

//! Integer repeat track lists through production Style and native LayoutNG.
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
    page.Open("https://page.test/responsive-tracks.html", 16384, 4096)
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
fn integer_repeat_reaches_native_tracks_and_two_row_column_flow() {
    native_test_thread::run(|| {
        let page = fixture("width:683px;grid-auto-flow:column;grid-template-columns:repeat(9,1fr);grid-template-rows:repeat(2,1fr);gap:10px", &(0..18).map(|i| format!("<div id=item{i} style='height:28px'></div>")).collect::<String>());
        let style = native(&page, "grid");
        let style = unsafe { &*style.Get() };
        assert!(
            !style.SpecifiedGridTemplateColumns().Get().is_null(),
            "repeat tracks discarded before LayoutNG"
        );
        let tracks = unsafe { &*style.SpecifiedGridTemplateColumns().Get() }.GetTrackList();
        assert_eq!(tracks.TrackCountWithoutAutoRepeat(), 9);
        assert_eq!(tracks.RepeaterCount(), 1);
        assert_eq!(tracks.RepeatCount(0, 0), 9);
        assert_eq!(item(&page, "item1").0.y, 38.0);
        assert_eq!(item(&page, "item2").0.y, 0.0);
        assert_eq!(item(&page, "grid").1.height, 66.0);
    });
}

#[test]
fn integer_repeat_with_spanning_item_preserves_four_column_geometry() {
    native_test_thread::run(|| {
        let page = fixture("width:1045px;grid-template-columns:repeat(4,1fr);gap:20px", "<div id=feature style='grid-column:1/3;grid-row:1/3;height:370px'></div><div id=card style='height:175px'></div><div style='height:175px'></div><div style='height:175px'></div>");
        assert_eq!(item(&page, "feature").1.width, 512.5);
        assert_eq!(item(&page, "card").0.x, 532.5);
    });
}

#[test]
fn repeat_custom_property_mutation_and_media_keep_native_snapshots() {
    native_test_thread::run(|| {
        let mut page = fixture("width:400px;--n:2;grid-template-columns:repeat(var(--n),minmax(0,1fr))", "<div id=item0 style='height:20px'></div><div id=item1 style='height:20px'></div><div id=item2 style='height:20px'></div><div id=item3 style='height:20px'></div>");
        let old = native(&page, "grid");
        let old_frame = page.CurrentFrame().unwrap().fragments.clone();
        assert_eq!(item(&page, "item0").1.width, 200.0);
        for (n, width) in [(4, 100.0), (2, 200.0)] {
            let result=page.Evaluate(&format!("document.getElementById('grid').style.setProperty('--n','{n}');document.getElementById('grid').getBoundingClientRect();"),"test:repeat-mutation").unwrap();
            assert!(result.Succeeded(), "{:?}", result.exception);
            assert_eq!(item(&page, "item0").1.width, width);
        }
        let result=page.Evaluate("var s=document.createElement('style');s.textContent='@media(max-width:700px){#grid{grid-template-columns:repeat(4,1fr)}}';document.head.appendChild(s);", "test:repeat-media").unwrap();
        assert!(result.Succeeded());
        page.ResizeViewport(600.0, 757.0, 1.0).unwrap();
        assert_eq!(item(&page, "item0").1.width, 100.0);
        page.ResizeViewport(1180.0, 757.0, 1.0).unwrap();
        assert_eq!(item(&page, "item0").1.width, 200.0);
        let old = unsafe { &*old.Get() };
        assert_eq!(
            unsafe { &*old.SpecifiedGridTemplateColumns().Get() }
                .GetTrackList()
                .RepeatCount(0, 0),
            2
        );
        let owner = page.Document();
        let doc = owner.GetDocument();
        let id = (0..doc.NodeCount())
            .find(|&i| {
                doc.Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "item0")
            })
            .map(|i| doc.Node(i).Id())
            .unwrap();
        assert_eq!(
            find_fragment(&old_frame, Offset::default(), &|f| f.node_id == id)
                .unwrap()
                .0
                .size
                .width,
            200.0
        );
    });
}

#[test]
fn repeat_math_counts_round_and_clamp_before_layout() {
    native_test_thread::run(|| {
        for (count, width) in [("calc(1 + 1.5)", 100.0), ("calc(0 - 4)", 300.0)] {
            let page = fixture(
                &format!("width:300px;grid-template-columns:repeat({count},1fr)"),
                "<div id=item style='height:20px'></div>",
            );
            assert_eq!(item(&page, "item").1.width, width, "{count}");
        }
    });
}

#[test]
fn repeated_line_height_units_recompute_after_inherited_metrics_change() {
    native_test_thread::run(|| {
        let mut page = fixture(
            "line-height:inherit;grid-template-columns:repeat(2,minmax(0,1lh))",
            "<div id=item style='height:20px'></div>",
        );
        for (height, width) in [(20, 20.0), (40, 40.0)] {
            let result=page.Evaluate(&format!("document.body.style.lineHeight='{height}px';document.getElementById('grid').getBoundingClientRect();"),"test:repeat-lh").unwrap();
            assert!(result.Succeeded());
            let style = native(&page, "grid");
            let style = unsafe { &*style.Get() };
            assert!(style.HasLineHeightRelativeUnits());
            assert!(style.HasGlyphRelativeUnits());
            assert_eq!(item(&page, "item").1.width, width);
        }
        let result=page.Evaluate("document.documentElement.style.lineHeight='30px';document.getElementById('grid').style.gridTemplateColumns='repeat(calc(1rlh / 10px),20px)';document.getElementById('grid').getBoundingClientRect();", "test:repeat-rlh-count").unwrap();
        assert!(result.Succeeded());
        let style = native(&page, "grid");
        let style = unsafe { &*style.Get() };
        assert!(style.HasRootRelativeUnits());
        assert!(style.HasLineHeightRelativeUnits());
        assert_eq!(
            unsafe { &*style.SpecifiedGridTemplateColumns().Get() }
                .GetTrackList()
                .RepeatCount(0, 0),
            3
        );
    });
}

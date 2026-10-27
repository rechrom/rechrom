//! Responsive intrinsic grid tracks through the production Style and Layout owners.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use layoutng_assembly::{
    fragment_tree::FragmentPaintData,
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
    node: &'a FragmentPaintData,
    parent: Offset,
    predicate: &impl Fn(&FragmentPaintData) -> bool,
) -> Option<(&'a FragmentPaintData, Offset)> {
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

// The same source-native constructor is used by document and shadow sheets.
#[test]
fn fit_content_tracks_reach_canonical_native_grid_sizes() {
    native_test_thread::run(|| {
        let page = fixture("grid-template-columns:[start] fit-content(120px) [next] auto", "<div id=first style='height:20px'><div style='width:80px;height:20px'></div></div><div id=second style='height:20px'></div>");
        let style = native(&page, "grid");
        let style = unsafe { &*style.Get() };
        let tracks = style.SpecifiedGridTemplateColumns();
        assert!(
            !tracks.Get().is_null(),
            "fit-content must reach native computed tracks"
        );
        let tracks = unsafe { &*tracks.Get() };
        let first = tracks.GetTrackList().RepeatTrackSize(0, 0);
        assert!(first.IsFitContent());
        assert_eq!(first.FitContentTrackBreadth().Pixels(), 120.0);
        assert!(first.MinTrackBreadth().IsAuto());
        assert!(first.MaxTrackBreadth().IsAuto());
        assert_eq!(
            tracks.GetNamedGridLines()[&foundation::String::from("next")],
            [1]
        );
        assert_eq!(item(&page, "second").0.x, 80.0);
    });
}

#[test]
fn fit_content_caps_max_content_but_preserves_intrinsic_minimum() {
    native_test_thread::run(|| {
        for (minimum, expected) in [("0", 120.0), ("auto", 200.0)] {
            let page = fixture("width:300px;grid-template-columns:fit-content(120px) auto", &format!("<div id=first style='min-width:{minimum}'><div style='width:200px;height:20px'></div></div><div id=second style='height:20px'></div>"));
            assert_eq!(item(&page, "second").0.x, expected, "{minimum}");
        }
    });
}

#[test]
fn fit_content_percentage_and_math_limits_use_definite_available_size() {
    native_test_thread::run(|| {
        for (limit, expected) in [("25%", 75.0), ("calc(25% + 10px)", 85.0), ("0", 0.0)] {
            let page = fixture(&format!("width:300px;grid-template-columns:fit-content({limit}) auto"), "<div id=first style='min-width:0'><div style='width:200px;height:20px'></div></div><div id=second style='height:20px'></div>");
            assert_eq!(item(&page, "second").0.x, expected, "{limit}");
        }
    });
}

#[test]
fn fit_content_rows_and_grid_shorthand_reach_native_sizing() {
    native_test_thread::run(|| {
        for css in [
            "grid-template-rows:fit-content(30px) auto;grid-template-columns:100px",
            "grid-template:fit-content(30px) auto / 100px",
        ] {
            let page = fixture(css, "<div id=first style='min-height:0'><div style='height:60px'></div></div><div id=second style='height:20px'></div>");
            let style = native(&page, "grid");
            let style = unsafe { &*style.Get() };
            let rows = unsafe { &*style.SpecifiedGridTemplateRows().Get() }.GetTrackList();
            assert!(rows.RepeatTrackSize(0, 0).IsFitContent());
            assert_eq!(item(&page, "second").0.y, 30.0, "{css}");
        }
    });
}

#[test]
fn shadow_responsive_fit_content_rule_keeps_named_spans_and_native_track_limits() {
    native_test_thread::run(|| {
        let mut page = fixture("", "");
        // The unrelated selector list also exercises production RuleSet order.
        let css = ".unused-a,.unused-b,.unused-c{color:green} #shadowgrid{display:grid;width:300px;grid-template-areas:'main action' 'optout note'} @media(width > 769px){#shadowgrid{grid-template-areas:'main action note' 'main action optout';grid-template-columns:fit-content(100%) auto fit-content(100%)}}";
        let script = format!(
            r#"
            var host=document.getElementById('grid');
            var shadow=host.attachShadow({{mode:'open'}});
            var g=document.createElement('div');g.id='shadowgrid';shadow.appendChild(g);
            for(var pair of [['main','100px;height:54px'],['action','80px;height:27px'],['note','30px;height:25px'],['optout','100px;height:17px']]){{
                var e=document.createElement('div');e.id=pair[0];e.setAttribute('style','grid-area:'+pair[0]+';width:'+pair[1]);g.appendChild(e);
            }}
            var s=document.createElement('style');s.appendChild(document.createTextNode({:?}));shadow.appendChild(s);g.getBoundingClientRect();
        "#,
            css
        );
        assert!(page
            .Evaluate(&script, "test:shadow-fit-content")
            .unwrap()
            .Succeeded());
        let style = native(&page, "shadowgrid");
        let style = unsafe { &*style.Get() };
        assert!(!style.GridTemplateAreas().Get().is_null());
        assert_eq!(unsafe { &*style.GridTemplateAreas().Get() }.column_count, 3);
        let tracks = unsafe { &*style.SpecifiedGridTemplateColumns().Get() }.GetTrackList();
        assert!(tracks.RepeatTrackSize(0, 0).IsFitContent());
        assert!(tracks.RepeatTrackSize(2, 0).IsFitContent());
        assert_eq!(item(&page, "action").0.x, 100.0);
        assert_eq!(item(&page, "note").0, Offset { x: 200.0, y: 0.0 });
        assert_eq!(item(&page, "shadowgrid").1.height, 54.0);
        page.ResizeViewport(600.0, 757.0, 1.0).unwrap();
        let resized = native(&page, "shadowgrid");
        let resized = unsafe { &*resized.Get() };
        assert_eq!(
            unsafe { &*resized.GridTemplateAreas().Get() }.column_count,
            2
        );
        page.ResizeViewport(1180.0, 757.0, 1.0).unwrap();
        assert_eq!(item(&page, "note").0, Offset { x: 200.0, y: 0.0 });
    });
}

#[test]
fn fit_content_mutation_replaces_limits_without_mutating_published_snapshots() {
    native_test_thread::run(|| {
        let mut page = fixture("width:300px;grid-template-columns:fit-content(120px) auto", "<div id=first style='min-width:0'><div style='width:200px;height:20px'></div></div><div id=second style='height:20px'></div>");
        let old_style = native(&page, "grid");
        let old_frame = page.CurrentFrame().unwrap().fragments.clone();
        assert_eq!(item(&page, "second").0.x, 120.0);
        for (limit, expected) in [("80px", 80.0), ("50%", 150.0), ("120px", 120.0)] {
            let script = format!("document.getElementById('grid').setAttribute('style','grid-template-columns:fit-content({limit}) auto');document.getElementById('second').getBoundingClientRect();");
            assert!(page
                .Evaluate(&script, "test:fit-content-mutation")
                .unwrap()
                .Succeeded());
            assert_eq!(item(&page, "second").0.x, expected);
        }
        let old_style = unsafe { &*old_style.Get() };
        let old_tracks = unsafe { &*old_style.SpecifiedGridTemplateColumns().Get() }.GetTrackList();
        assert_eq!(
            old_tracks
                .RepeatTrackSize(0, 0)
                .FitContentTrackBreadth()
                .Pixels(),
            120.0
        );
        let owner = page.Document();
        let doc = owner.GetDocument();
        let second = (0..doc.NodeCount())
            .find(|&i| {
                doc.Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "second")
            })
            .unwrap();
        let (_, old_offset) = find_fragment(&old_frame, Offset::default(), &|f| {
            f.node_id == doc.Node(second).Id()
        })
        .unwrap();
        assert_eq!(old_offset.x, 120.0);
    });
}

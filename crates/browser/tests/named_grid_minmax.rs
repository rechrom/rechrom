//! Named grid minmax tracks through the production Style and Layout owners.
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
    let html = format!("<!doctype html><style>html{{font-size:16px}}body{{margin:0}}#grid{{display:grid;width:1180px;{css}}}.item{{height:20px;background:red}}</style><div id=grid>{children}</div>");
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

const NAMED: &str = "[extended-full-start] 16px [full-start] 1fr [content-start] minmax(0,48rem) [content-end] 1fr [full-end] 16px [extended-full-end]";
const CHILDREN: &str = "<div class=item id=content style='grid-column:content'></div><div class=item id=extended style='grid-column:content / extended-full-end'></div>";

#[test]
fn minmax_is_retained_in_canonical_computed_grid_tracks() {
    native_test_thread::run(|| {
        let page = fixture(&format!("grid-template-columns:{NAMED}"), CHILDREN);
        let style = native(&page, "grid");
        let style = unsafe { &*style.Get() };
        let columns = style.SpecifiedGridTemplateColumns();
        assert!(
            !columns.Get().is_null(),
            "valid minmax track list was discarded"
        );
        let columns = unsafe { &*columns.Get() };
        assert_eq!(columns.GetTrackList().TrackCountWithoutAutoRepeat(), 5);
        assert_eq!(
            columns.GetNamedGridLines()[&foundation::String::from("content-start")],
            vec![2]
        );
        let track = columns.GetTrackList().RepeatTrackSize(2, 0);
        assert_eq!(
            track.GetType(),
            layoutng_style::style::grid_track_size::GridTrackSizeType::kMinMaxTrackSizing
        );
        assert_eq!(track.MinTrackBreadth().Pixels(), 0.0);
        assert_eq!(track.MaxTrackBreadth().Pixels(), 768.0);
    });
}

#[test]
fn named_minmax_tracks_publish_centered_and_extended_fragments() {
    native_test_thread::run(|| {
        let page = fixture(&format!("grid-template-columns:{NAMED}"), CHILDREN);
        let (offset, size) = item(&page, "content");
        assert_eq!(offset.x, 206.0);
        assert_eq!(size.width, 768.0);
        let (offset, size) = item(&page, "extended");
        assert_eq!(offset.x, 206.0);
        assert_eq!(size.width, 974.0);
    });
}

#[test]
fn custom_property_minmax_and_math_tracks_reach_geometry() {
    native_test_thread::run(|| {
        let page = fixture("--pad:max(1rem,calc(50vw - 720px + 1rem));--content:48rem;--layout:[extended-full-start] var(--pad) [full-start] 1fr [content-start] minmax(0,var(--content)) [content-end] 1fr [full-end] var(--pad) [extended-full-end];grid-template-columns:var(--layout)", CHILDREN);
        let (offset, size) = item(&page, "content");
        assert_eq!(offset.x, 206.0);
        assert_eq!(size.width, 768.0);
        let (offset, size) = item(&page, "extended");
        assert_eq!(offset.x, 206.0);
        assert_eq!(size.width, 974.0);
    });
}

#[test]
fn minmax_shrinks_to_the_available_named_grid_space() {
    native_test_thread::run(|| {
        let page = fixture(
            &format!("grid-template-columns:{NAMED};width:600px"),
            CHILDREN,
        );
        let (offset, size) = item(&page, "content");
        assert_eq!(offset.x, 16.0);
        assert_eq!(size.width, 568.0);
        let (offset, size) = item(&page, "extended");
        assert_eq!(offset.x, 16.0);
        assert_eq!(size.width, 584.0);
    });
}

#[test]
fn minmax_custom_property_mutation_relayouts_without_changing_retained_frame() {
    native_test_thread::run(|| {
        let mut page = fixture(
            &format!(
                "--content:48rem;grid-template-columns:{}",
                NAMED.replace("48rem", "var(--content)")
            ),
            CHILDREN,
        );
        let old = page.CurrentFrame().unwrap().fragments.clone();
        let old_style = native(&page, "grid");
        for (width, x) in [(400.0, 390.0), (768.0, 206.0)] {
            let result=page.Evaluate(&format!("document.getElementById('grid').style.setProperty('--content','{width}px')"),"test:grid-minmax").unwrap();
            assert!(result.Succeeded(), "{:?}", result.exception);
            let (offset, size) = item(&page, "content");
            assert_eq!(offset.x, x);
            assert_eq!(size.width, width);
        }
        let doc = page.Document();
        let doc = doc.GetDocument();
        let id = (0..doc.NodeCount())
            .find(|&i| {
                doc.Node(i)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == "content")
            })
            .map(|i| doc.Node(i).Id())
            .unwrap();
        let (old_item, old_offset) =
            find_fragment(&old, Offset::default(), &|f| f.node_id == id).unwrap();
        assert_eq!(old_offset.x, 206.0);
        assert_eq!(old_item.size.width, 768.0);
        assert_eq!(
            unsafe { &*(&*old_style.Get()).SpecifiedGridTemplateColumns().Get() }
                .GetTrackList()
                .RepeatTrackSize(2, 0)
                .MaxTrackBreadth()
                .Pixels(),
            768.0
        );
    });
}

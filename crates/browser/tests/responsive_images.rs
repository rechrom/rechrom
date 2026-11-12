#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::internal::{
    layout_object::LayoutObject, layout_pass_scope::LayoutObjectFactoryScope,
    layout_replaced::LayoutInputReplaced,
};
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
        let is_document = request.destination == url_loader::RequestDestination::kDocument;
        let (mime_type, body) = if is_document {
            (
                "text/html",
                br#"<!doctype html><style>body{margin:0}</style><img id=hero loading=lazy style="width:200px;height:100px" sizes=auto srcset="small.svg 100w, medium.svg 200w, large.svg 400w">"#
                    .to_vec(),
            )
        } else {
            (
                "image/svg+xml",
                br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"/>"#.to_vec(),
            )
        };
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: mime_type.into(),
            body,
            ..Default::default()
        }))))
    }
}

struct ContentDPRLoader;
impl URLLoader for ContentDPRLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let is_document = request.destination == url_loader::RequestDestination::kDocument;
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: if is_document {
                "text/html".into()
            } else {
                "image/svg+xml".into()
            },
            content_dpr: (!is_document).then_some(4.0),
            body: if is_document {
                br#"<!doctype html><style>body{margin:0}</style><img id=hero src=hero.svg>"#
                    .to_vec()
            } else {
                br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"/>"#.to_vec()
            },
            ..Default::default()
        }))))
    }
}

struct ScrollLazyLoader {
    requests: Rc<RefCell<Vec<String>>>,
}
impl URLLoader for ScrollLazyLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        self.requests.borrow_mut().push(request.url.to_string());
        let is_document = request.destination == url_loader::RequestDestination::kDocument;
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: if is_document {
                "text/html".into()
            } else {
                "image/svg+xml".into()
            },
            body: if is_document {
                br#"<!doctype html><style>body{margin:0;height:7000px}#lazy{position:absolute;top:5000px;width:200px;height:100px}</style><img id=lazy loading=lazy src=lazy.svg>"#.to_vec()
            } else {
                br#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100"/>"#.to_vec()
            },
            ..Default::default()
        }))))
    }
}

struct LazyXHR;
impl xhr_transport::XMLHttpRequestTransport for LazyXHR {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("unexpected fixture XHR"))
    }
}

fn image(page: &Page, id: u64) -> &LayoutInputReplaced {
    let assembly = browser::CreateLayoutAssembly();
    let _scope = LayoutObjectFactoryScope::new(&assembly.objects);
    let engine = page.GetLayoutEngine();
    let root = engine.GetLayoutTree().unwrap().Root();
    let mut object = root as *const LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == id {
            assert!(current.IsLayoutImage());
            return unsafe { &*object.cast::<LayoutInputReplaced>() };
        }
        object = current.NextInPreOrder(root);
    }
    panic!("responsive image layout object missing")
}

fn hero(page: &Page) -> (usize, u64) {
    let owner = page.Document();
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find_map(|index| {
            document
                .Node(index)
                .FindAttribute("id")
                .filter(|attribute| attribute.value == "hero")
                .map(|_| (index, document.Node(index).Id()))
        })
        .unwrap()
}

#[test]
fn root_scroll_admits_a_deferred_image_in_document_coordinates() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let assembly = browser::CreateLayoutAssembly();
        let requests = Rc::new(RefCell::new(Vec::new()));
        let mut page = Page::Create(
            Rc::new(RefCell::new(ScrollLazyLoader {
                requests: requests.clone(),
            })),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
            browser::CreateBrowserConstraints(800, 600),
            Some(browser::page::ScriptEnvironment {
                runtime: Box::new(
                    javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime::with_max_stack_size(
                        4 * 1024 * 1024,
                    ),
                ),
                xhr: Box::new(LazyXHR),
                user_agent: "lazy-scroll-fixture".into(),
            }),
            None,
        );
        page.Open("https://lazy-scroll.test/", 16384, 4096).unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if !page.IsLoading() {
                break;
            }
        }
        assert!(!page.IsLoading());
        assert!(
            !requests
                .borrow()
                .iter()
                .any(|url| url.ends_with("lazy.svg")),
            "far-offscreen lazy image must remain deferred"
        );

        let result = page
            .Evaluate("scrollTo(0,4400)", "lazy-scroll-fixture")
            .unwrap();
        assert!(result.Succeeded(), "{:?}", result.exception);
        let frame_time = std::time::Instant::now();
        page.UpdateRendering(foundation::begin_frame::BeginFrameArgs {
            source_id: 1,
            sequence_number: 1,
            frame_time,
            deadline: frame_time + std::time::Duration::from_millis(16),
            interval: std::time::Duration::from_millis(16),
        })
        .unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if requests
                .borrow()
                .iter()
                .any(|url| url.ends_with("lazy.svg"))
            {
                break;
            }
        }
        assert!(
            requests
                .borrow()
                .iter()
                .any(|url| url.ends_with("lazy.svg")),
            "scrolling the document viewport over the image must admit its request"
        );
    });
}

#[test]
fn lazy_auto_sizes_reselects_from_the_rendered_content_width() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let assembly = browser::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(Loader)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
            browser::CreateBrowserConstraints(800, 600),
            None,
            None,
        );
        page.Open("https://responsive.test/", 16384, 4096).unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if page.CurrentFrame().is_some() {
                break;
            }
        }
        let (index, id) = hero(&page);
        assert_eq!(
            page.Document().GetDocument().ResponsiveImageAutoSize(index),
            Some(200.0)
        );
        assert_eq!(
            page.Document().GetDocument().CurrentImageSource(index),
            Some("medium.svg")
        );

        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: id,
            name: "style".into(),
            value: "width:100px;height:100px".into(),
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(
            page.Document().GetDocument().ResponsiveImageAutoSize(index),
            Some(100.0)
        );
        assert_eq!(
            page.Document().GetDocument().CurrentImageSource(index),
            Some("small.svg")
        );
    });
}

#[test]
fn content_dpr_header_overrides_html_image_density() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let assembly = browser::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(ContentDPRLoader)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
            browser::CreateBrowserConstraints(800, 600),
            None,
            None,
        );
        page.Open("https://content-dpr.test/", 16384, 4096).unwrap();
        for _ in 0..40 {
            page.RunTask().unwrap();
        }
        let (index, id) = hero(&page);
        let owner = page.Document();
        let document = owner.GetDocument();
        let source = document.CurrentImageSource(index).unwrap();
        let resource = document.ImageResourceFor(source).unwrap();
        assert_eq!(document.ImageResourceContentDPR(resource.id), Some(4.0));
        assert!(document.IsUseCounted(dom::WebFeature::kClientHintsContentDPR));
        drop(owner);
        assert_eq!(image(&page, id).ImageDevicePixelRatio(), 0.25);
    });
}

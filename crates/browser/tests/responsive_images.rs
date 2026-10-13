#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use dom::dom_mutation::{DOMMutation, DOMMutationType};
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
        let (mime_type, body) = if request.destination == url_loader::RequestDestination::kDocument
        {
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
fn lazy_auto_sizes_reselects_from_the_rendered_content_width() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let assembly = browser::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(Loader)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(browser::CreateSVGImageDecoder(
                &assembly,
            ))),
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

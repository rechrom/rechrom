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
        Ok(Box::new(Response(Some(URLResponse{final_url:request.url.clone(),status_code:200,mime_type:"text/html".into(),
            body:br#"<!doctype html><style>
            body{margin:0} #container{container-type:inline-size;width:240px;height:80px;padding:20px;border:5px solid black}
            #target{width:11px;height:10px}
            @container (min-width:250px){#target{width:91px}}
            @container (max-width:249px){#target{height:37px}}
            </style><div id=container><div id=target></div></div>"#.to_vec(),..Default::default()}))))
    }
}
fn id(page: &Page, name: &str) -> u64 {
    let owner = page.Document();
    let d = owner.GetDocument();
    (0..d.NodeCount())
        .find_map(|i| {
            d.Node(i)
                .FindAttribute("id")
                .filter(|a| a.value == name)
                .map(|_| d.Node(i).Id())
        })
        .unwrap()
}
fn fragment<'a>(
    root: &'a layoutng_assembly::fragment_tree::FragmentNode,
    id: u64,
) -> &'a layoutng_assembly::fragment_tree::FragmentNode {
    if root.node_id == id {
        return root;
    }
    root.children.iter().find_map(|c| find(c, id)).unwrap()
}
fn find(
    root: &layoutng_assembly::fragment_tree::FragmentNode,
    id: u64,
) -> Option<&layoutng_assembly::fragment_tree::FragmentNode> {
    if root.node_id == id {
        Some(root)
    } else {
        root.children.iter().find_map(|c| find(c, id))
    }
}
#[test]
fn native_content_box_publication_recalculates_rules_before_page_paint() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let assembly = browser::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(Loader)),
            Rc::new(RefCell::new(
                image_decoder::skia_image_decoder::SkiaImageDecoder,
            )),
            Rc::new(RefCell::new(document_image::SVGImageDecoder::new(
                &assembly,
            ))),
            browser::CreateBrowserConstraints(600, 300),
            None,
            None,
        );
        page.Open("https://container.test/", 16384, 4096).unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if !page.IsLoading() && page.CurrentFrame().is_some() {
                break;
            }
        }
        assert!(!page.IsLoading());
        let container = id(&page, "container");
        let target = id(&page, "target");
        let frame = page.CurrentFrame().unwrap();
        assert_eq!(fragment(&frame.fragments, container).size.width, 290.0);
        assert_eq!(fragment(&frame.fragments, target).size.width, 11.0);
        assert_eq!(fragment(&frame.fragments, target).size.height, 37.0);
        let sizes = page.GetLayoutEngine().ContainerQuerySizes();
        assert_eq!(
            sizes.iter().find(|s| s.node_id == container).unwrap().width,
            240.0
        );
        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: container,
            name: "style".into(),
            value: "width:320px".into(),
            ..Default::default()
        }))
        .unwrap();
        let frame = page.CurrentFrame().unwrap();
        assert_eq!(fragment(&frame.fragments, target).size.width, 91.0);
        assert_eq!(fragment(&frame.fragments, target).size.height, 10.0);
        assert_eq!(
            page.GetLayoutEngine()
                .ContainerQuerySizes()
                .iter()
                .find(|s| s.node_id == container)
                .unwrap()
                .width,
            320.0
        );
        page.Apply(PageMutation::DOMMutation(DOMMutation {
            mutation_type: DOMMutationType::kSetAttribute,
            target_node_id: container,
            name: "style".into(),
            value: "width:220px".into(),
            ..Default::default()
        }))
        .unwrap();
        let frame = page.CurrentFrame().unwrap();
        assert_eq!(fragment(&frame.fragments, target).size.width, 11.0);
        assert_eq!(fragment(&frame.fragments, target).size.height, 37.0);
    });
}

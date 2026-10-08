use crate::page::Page;
use layoutng_assembly::internal::{
    layout_block_flow::LayoutBlockFlow, layout_object::LayoutObject,
};
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};
struct Loader(String);
struct Operation(Option<URLResponse>);
impl URLLoadOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Operation(Some(URLResponse {
            body: self.0.as_bytes().to_vec(),
            final_url: request.url.clone(),
            mime_type: "text/html".into(),
            status_code: 200,
            ..Default::default()
        }))))
    }
}
fn page(html: &str, width: u32, height: u32, scale: f64) -> Page {
    let mut constraints = crate::CreateBrowserConstraints(width, height);
    constraints.device_pixel_ratio = scale;
    let assembly = crate::CreateLayoutAssembly();
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader(html.into()))),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(document_image::SVGImageDecoder::new(
            &assembly,
        ))),
        constraints,
        None,
        None,
    );
    page.Open("https://viewport.test/", 8192, 4096).unwrap();
    while page.IsLoading() {
        page.RunTask().unwrap();
    }
    page
}
fn shape(page: &Page) -> *const font_engine::fonts::shaping::shape_result::ShapeResult {
    let owner = page.Document();
    let document = owner.GetDocument();
    let id = (0..document.NodeCount())
        .find_map(|i| {
            let node = document.Node(i);
            node.FindAttribute("id")
                .is_some_and(|a| a.value == "text")
                .then_some(node.Id())
        })
        .unwrap();
    let engine = page.GetLayoutEngine();
    let tree = engine.GetLayoutTree().unwrap();
    let root = tree.Root() as *const LayoutObject as *mut LayoutObject;
    let mut object = root;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() && unsafe { &*node }.InputId() == id {
            let flow = foundation::DynamicTo::<LayoutBlockFlow>(object);
            assert!(!flow.is_null());
            let data = unsafe { &*flow }.GetInlineNodeData();
            assert!(!data.is_null());
            return unsafe { &*data }
                .items
                .iter()
                .map(|item| unsafe { &*item.Get() }.TextShapeResult())
                .find(|p| !p.is_null())
                .unwrap();
        }
        object = current.NextInPreOrder(root);
    }
    panic!("text layout object missing")
}
fn pixels(page: &Page, width: u32, height: u32, scale: f64) -> Vec<u8> {
    raster::surface::RenderDisplayItemListToSurface(
        &page.CurrentFrame().unwrap().display_items,
        width,
        height,
        scale,
    )
    .unwrap()
    .pixels()
    .to_vec()
}
#[test]
fn viewport_geometry_reuses_shape_and_real_font_inputs_match_fresh_layout() {
    crate::native_test_thread::run(|| {
        for html in [
            "<!doctype html><style>body{margin:0}#text{margin:0;font:16px sans-serif}@media(max-width:500px){#text{font-size:24px;font-weight:700;letter-spacing:2px;word-spacing:3px}}</style><p id='text'>Words to wrap while preserving glyphs.</p>",
            "<!doctype html><style>body{margin:0}#text{margin:0;font-family:sans-serif;font-size:4vw}</style><p id='text'>Viewport sized text.</p>",
            "<!doctype html><style>body{margin:0}#text{margin:0;font:16px sans-serif}@media(max-width:500px){#text{width:80%}}</style><p id='text'>Geometry only style change.</p>",
            "<!doctype html><style>body{margin:0}#text{margin:0;font:16px sans-serif}@media(max-width:500px){#text{font-family:serif;font-style:italic;font-size:20px;zoom:1.25}}</style><p id='text'>Font family, zoom.</p>",
        ] {
            let mut live=page(html,800,240,1.0);
            let original=shape(&live);
            let _original_root=foundation::Persistent::from_ptr(original as *mut font_engine::fonts::shaping::shape_result::ShapeResult);
            live.ResizeViewport(720.0,280.0,1.0).unwrap();
            if !html.contains("4vw") {assert_eq!(shape(&live),original,"pure width change must retain prepared glyphs");}
            let fresh=page(html,720,280,1.0);
            assert_eq!(pixels(&live,720,280,1.0),pixels(&fresh,720,280,1.0),"geometry result must match fresh layout");
            drop(fresh);
            live.ResizeViewport(400.0,300.0,1.0).unwrap();
            if html.contains("width:80%") { assert_eq!(shape(&live),original,"geometry-only style changes must retain glyphs"); }
            else { assert_ne!(shape(&live),original,"media/vw typography must invalidate shaping"); }
            let fresh=page(html,400,300,1.0);
            assert_eq!(pixels(&live,400,300,1.0),pixels(&fresh,400,300,1.0),"changed font result must match fresh layout");
            drop(fresh);
            live.ResizeViewport(400.0,300.0,2.0).unwrap();
            let fresh=page(html,400,300,2.0);
            assert_eq!(pixels(&live,800,600,2.0),pixels(&fresh,800,600,2.0),"DPR change must match fresh layout");
        }
    });
}

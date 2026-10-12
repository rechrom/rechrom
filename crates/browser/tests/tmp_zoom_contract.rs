use browser::page::{Page, PageClient};
use std::{cell::RefCell, rc::Rc};
struct C;
impl PageClient for C {}
fn walk(
    n: &layoutng_assembly::fragment_tree::FragmentNode,
    p: layoutng_assembly::internal::layout_input::Offset,
) {
    let a = layoutng_assembly::internal::layout_input::Offset {
        x: p.x + n.offset.x,
        y: p.y + n.offset.y,
    };
    eprintln!(
        "ZFRAG id={} kind={:?} abs={:?} size={:?} content={:?}",
        n.node_id, n.kind, a, n.size, n.content_size
    );
    for c in &n.children {
        walk(c, a)
    }
}
#[test]
fn zoom_contract() {
    let _h = foundation::LayoutHeapScope::new();
    let loader = Rc::new(RefCell::new(
        url_loader::DefaultURLLoader::new(Default::default()).unwrap(),
    ));
    let a = browser::CreateLayoutAssembly();
    let mut s = browser::CreateBrowserConstraints(1280, 713);
    s.device_pixel_ratio = 2.0;
    let mut p = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(document_image::SVGImageDecoder::new(&a))),
        s,
        None,
        Some(Rc::new(RefCell::new(C))),
    );
    p.Open("http://127.0.0.1:9555/zoom-contract.html", 16384, 4096)
        .unwrap();
    while p.IsLoading() {
        p.RunTask().unwrap()
    }
    let f = p.CurrentFrame().unwrap();
    walk(&f.fragments, Default::default());
    let o = p.Document();
    let d = o.GetDocument();
    for i in 0..d.NodeCount() {
        let n = d.Node(i);
        if let Some(s) = d.ResolvedStyleFor(i) {
            let st = unsafe { &*s.native_style.Get() };
            eprintln!("ZSTYLE id={} name={} class={:?} zoom={} width={:?} height={:?} marginL={:?} paddingL={:?}",n.Id(),n.Name(),n.FindAttribute("class").map(|a|a.value.as_str()),st.EffectiveZoom(),st.Width(),st.Height(),st.MarginLeft(),st.PaddingLeft())
        }
    }
}

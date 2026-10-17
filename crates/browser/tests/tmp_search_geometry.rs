use browser::page::{Page, PageClient};
use layoutng_assembly::internal::layout_input::Offset;
use std::{cell::RefCell, rc::Rc};

struct Client;
impl PageClient for Client {}

fn open() -> Page {
    let loader = Rc::new(RefCell::new(
        url_loader::DefaultURLLoader::new(Default::default()).unwrap(),
    ));
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(1280, 713);
    constraints.device_pixel_ratio = 2.0;
    let mut page = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        constraints,
        None,
        Some(Rc::new(RefCell::new(Client))),
    );
    page.Open("http://127.0.0.1:9555/text-fixed-oracle.html", 16384, 4096)
        .unwrap();
    while page.IsLoading() {
        page.RunTask().unwrap();
    }
    page
}

fn walk(node: &layoutng_assembly::fragment_tree::FragmentNode, parent: Offset, wanted: &[u64]) {
    let absolute = Offset {
        x: parent.x + node.offset.x,
        y: parent.y + node.offset.y,
    };
    if true {
        eprintln!("SEARCH_FRAGMENT node={} kind={:?} relative={:?} absolute={absolute:?} size={:?} content={:?}", node.node_id, node.paint.source_kind, node.offset, node.size, node.content_size);
    }
    for child in &node.children {
        walk(child, absolute, wanted);
    }
}

#[test]
fn dump_search_geometry() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open();
    let frame = page.CurrentFrame().unwrap();
    let owner = page.Document();
    let document = owner.GetDocument();
    let mut wanted = Vec::new();
    for index in 0..document.NodeCount() {
        let node = document.Node(index);
        if node.FindAttribute("class").is_some_and(|a| {
            a.value
                .split_ascii_whitespace()
                .any(|c| c == "octicon-search" || c == "octicon-book")
        }) {
            wanted.push(node.Id());
            for child in node.Children() {
                wanted.push(document.Node(*child).Id());
            }
            let mut current = index;
            for depth in 0..5 {
                let n = document.Node(current);
                wanted.push(n.Id());
                let s = document.ResolvedStyleFor(current).unwrap();
                let native = unsafe { &*s.native_style.Get() };
                let primary = unsafe { &*native.GetFont() }.PrimaryFont();
                let font = if primary.is_null() {
                    None
                } else {
                    let p = unsafe { &*primary };
                    Some((
                        p.PlatformData().RawFont().FamilyName(),
                        p.GetFontMetrics().FloatAscent(),
                        p.GetFontMetrics().FloatDescent(),
                        p.GetFontMetrics().Ascent(),
                        p.GetFontMetrics().Descent(),
                    ))
                };
                eprintln!("SEARCH_NODE depth={depth} index={current} id={} name={} class={:?} display={:?} vertical={:?} line_height={} width={:?} height={:?} font={font:?}", n.Id(), n.Name(), n.FindAttribute("class").map(|a|a.value.as_str()), native.Display(), native.VerticalAlign(), native.ComputedLineHeight(), native.Width(), native.Height());
                let Some(parent) = n.Parent() else { break };
                current = parent;
            }
        }
    }
    walk(&frame.fragments, Offset::default(), &wanted);
    for item in frame
        .display_items
        .items
        .iter()
        .filter(|i| wanted.contains(&i.node_id))
    {
        eprintln!(
            "SEARCH_ITEM node={} type={:?} rect={:?} transform={:?} path_first={:?} path_last={:?}",
            item.node_id,
            item.r#type,
            item.rect,
            item.transform,
            item.path.first(),
            item.path.last()
        );
    }
}

use browser::page::{Page, PageClient};
use std::{cell::RefCell, rc::Rc};

struct Client;
impl PageClient for Client {}

fn fragment_geometry(
    node: &layoutng_assembly::fragment_tree::FragmentNode,
    wanted: u64,
    parent: layoutng_assembly::internal::layout_input::Offset,
) -> Option<(
    layoutng_assembly::internal::layout_input::Offset,
    layoutng_assembly::internal::layout_input::Size,
)> {
    let absolute = layoutng_assembly::internal::layout_input::Offset {
        x: parent.x + node.offset.x,
        y: parent.y + node.offset.y,
    };
    if node.node_id == wanted {
        return Some((absolute, node.size));
    }
    node.children
        .iter()
        .find_map(|child| fragment_geometry(child, wanted, absolute))
}

#[test]
fn map_pixel_diff_to_paint_records() {
    let _heap = foundation::LayoutHeapScope::new();
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
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(
            &assembly,
        ))),
        constraints,
        None,
        Some(Rc::new(RefCell::new(Client))),
    );
    page.Open(
        "http://127.0.0.1:9555/github-static-oracle-zero.html",
        16384,
        4096,
    )
    .unwrap();
    while page.IsLoading() {
        page.RunTask().unwrap();
    }
    let artifact = &page.CurrentFrame().unwrap().display_items;
    let mut map = String::new();
    for (record_index, record) in artifact.display_items.iter().enumerate() {
        let kinds = artifact.items[record.record_begin..record.record_end]
            .iter()
            .filter(|item| {
                format!("{:?}", item.r#type).starts_with("kDraw")
                    || format!("{:?}", item.r#type).starts_with("kStroke")
            })
            .map(|item| format!("{:?}:{}", item.r#type, item.node_id))
            .collect::<Vec<_>>()
            .join(",");
        map.push_str(&format!(
            "{record_index}\t{}\t{}\t{}\t{}\t{kinds}\n",
            record.visual_rect.x,
            record.visual_rect.y,
            record.visual_rect.width,
            record.visual_rect.height,
        ));
    }
    std::fs::write("/private/tmp/github-paint-record-map.tsv", map).unwrap();
    for item in artifact.items.iter().filter(|item| {
        item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
            && matches!(item.node_id, 630 | 1149 | 1368)
    }) {
        eprintln!(
            "GLYPHS node={} origin={:?} size={} {:?}",
            item.node_id,
            item.text_blob_origin,
            item.font_size,
            item.glyphs
                .iter()
                .map(|glyph| (glyph.id, glyph.offset.x, glyph.offset.y))
                .collect::<Vec<_>>()
        );
    }
    let owner = page.Document();
    let document = owner.GetDocument();
    for node_id in [630_u64, 1149, 1368] {
        eprintln!("ANCESTORS node={node_id}");
        let mut current = document.FindNodeById(node_id);
        for depth in 0..8 {
            let Some(index) = current else { break };
            let node = document.Node(index);
            eprintln!(
                " ANCESTOR depth={depth} id={} name={} class={:?} fragment={:?}",
                node.Id(),
                node.Name(),
                node.FindAttribute("class")
                    .map(|value| value.value.as_str()),
                fragment_geometry(
                    &page.CurrentFrame().unwrap().fragments,
                    node.Id(),
                    layoutng_assembly::internal::layout_input::Offset::default(),
                )
            );
            current = node.Parent();
        }
    }
    let points = [
        (685.0, 146.0),
        (270.5, 247.0),
        (233.5, 33.0),
        (621.5, 93.0),
        (1058.5, 188.5),
        (400.0, 646.5),
    ];
    for (x, y) in points {
        eprintln!("DIFF_POINT {x} {y}");
        for (record_index, record) in artifact.display_items.iter().enumerate() {
            let r = record.visual_rect;
            if x >= r.x - 2.0
                && x <= r.x + r.width + 2.0
                && y >= r.y - 2.0
                && y <= r.y + r.height + 2.0
            {
                eprintln!(
                    " RECORD {record_index} visual={r:?} range={}..{}",
                    record.record_begin, record.record_end
                );
                for (item_index, item) in artifact.items[record.record_begin..record.record_end]
                    .iter()
                    .enumerate()
                {
                    eprintln!(
                        "  ITEM {} {:?} node={} rect={:?} color={:?} font={} origin={:?} glyphs={} path={}",
                        record.record_begin + item_index,
                        item.r#type,
                        item.node_id,
                        item.rect,
                        item.color,
                        item.font_size,
                        item.text_blob_origin,
                        item.glyphs.len(),
                        item.path.len(),
                    );
                }
            }
        }
    }
}

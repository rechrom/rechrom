use browser::page::{Page, PageClient, ScriptEnvironment};
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use layoutng_assembly::layout_engine::PaintFragmentCursor;
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLStreamOperation};

fn dump_word_fragments(fragment: PaintFragmentCursor<'_>, word_ids: &[u64]) {
    let data = fragment.PaintData();
    if word_ids.contains(&data.node_id) {
        eprintln!(
            "WORD_FRAGMENT node={} kind={:?} offset={:?} layer_client={} self_painting={} transform={} establishes={}",
            data.node_id,
            data.paint.source_kind,
            data.offset,
            data.paint.paint_layer_client_id,
            data.paint.PaintLayerIsSelfPainting(),
            data.paint.style.has_transform_payload(),
            data.paint.establishes_paint_state,
        );
    }
    for child in fragment.Children() {
        dump_word_fragments(child, word_ids);
    }
}

struct RecordingLoader {
    inner: url_loader::DefaultURLLoader,
    requests: Rc<RefCell<Vec<String>>>,
}

impl URLLoader for RecordingLoader {
    fn LoadStream(&mut self, request: &URLRequest) -> std::io::Result<Box<dyn URLStreamOperation>> {
        self.requests.borrow_mut().push(request.url.clone());
        self.inner.LoadStream(request)
    }

    fn Load(&mut self, request: &URLRequest) -> std::io::Result<Box<dyn URLLoadOperation>> {
        self.requests.borrow_mut().push(request.url.clone());
        self.inner.Load(request)
    }
}

#[derive(Default)]
struct Client {
    chrome: browser::page::PageChromeState,
    errors: Rc<RefCell<Vec<String>>>,
    resource_errors: Rc<RefCell<Vec<String>>>,
}
impl PageClient for Client {
    fn DidEventHandlerChromeEffect(&mut self, effect: browser::page::EventHandlerChromeEffect) {
        self.chrome.Apply(effect);
    }
    fn DidReportScriptError(
        &mut self,
        error: &javascript::javascript_runtime::JavaScriptException,
    ) {
        self.errors.borrow_mut().push(error.message.clone());
    }

    fn DidFailResource(&mut self, url: &str, message: &str) {
        self.resource_errors
            .borrow_mut()
            .push(format!("{url}: {message}"));
    }
}

#[test]
fn git_text_metrics_probe() {
    let _heap = foundation::LayoutHeapScope::new();
    let requests = Rc::new(RefCell::new(Vec::new()));
    let options = url_loader::DefaultURLLoaderOptions::default();
    let loader = Rc::new(RefCell::new(RecordingLoader {
        inner: url_loader::DefaultURLLoader::new(options.clone()).unwrap(),
        requests: requests.clone(),
    }));
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(1280, 720);
    constraints.device_pixel_ratio = 2.0;
    let errors = Rc::new(RefCell::new(Vec::new()));
    let resource_errors = Rc::new(RefCell::new(Vec::new()));
    let mut page = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        constraints,
        Some(ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::new()),
            xhr: xhr_transport::CreateHTTPXMLHttpRequestTransport(&options).unwrap(),
            user_agent: options.user_agent,
        }),
        Some(Rc::new(RefCell::new(Client {
            chrome: Default::default(),
            errors: errors.clone(),
            resource_errors: resource_errors.clone(),
        }))),
    );
    let url = std::env::var("RECHROM_TEXT_PROBE_URL")
        .unwrap_or_else(|_| "https://github.com".to_string());
    page.Open(&url, 16384, 4096).unwrap();
    while page.IsLoading() {
        page.RunTask().unwrap();
    }
    let settle_ms = std::env::var("RECHROM_TEXT_PROBE_SETTLE_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(8_000);
    let settle_deadline = Instant::now() + Duration::from_millis(settle_ms);
    while Instant::now() < settle_deadline {
        page.RunTask().unwrap();
        std::thread::sleep(Duration::from_millis(1));
    }
    for error in errors.borrow().iter() {
        eprintln!("SCRIPT_ERROR {error}");
    }
    for error in resource_errors.borrow().iter() {
        eprintln!("RESOURCE_ERROR {error}");
    }
    for request in requests.borrow().iter().filter(|url| url.contains(".js")) {
        eprintln!("SCRIPT_REQUEST {request}");
    }

    let owner = page.Document();
    let document = owner.GetDocument();
    let word_ids: Vec<_> = (0..document.NodeCount())
        .filter_map(|index| {
            let node = document.Node(index);
            node.FindAttribute("class")
                .is_some_and(|attribute| {
                    attribute
                        .value
                        .split_ascii_whitespace()
                        .any(|v| v == "word")
                })
                .then_some(node.Id())
        })
        .collect();
    dump_word_fragments(
        PaintFragmentCursor::FromHandle(&page.CurrentFrame().unwrap().fragments),
        &word_ids,
    );
    for index in 0..document.NodeCount() {
        let node = document.Node(index);
        let Some(resolved) = document.ResolvedStyleFor(index) else {
            continue;
        };
        let style = unsafe { &*resolved.computed_style_owner.Get() };
        let size = style.ComputedFontSize();
        if !matches!(size as i32, 14 | 16 | 36 | 56) {
            continue;
        }
        let primary = unsafe { &*style.GetFont() }.PrimaryFont();
        if primary.is_null() {
            continue;
        }
        let metrics = unsafe { &*primary }.GetFontMetrics();
        eprintln!(
            "NODE id={} name={} size={} weight={} ascent={} float_ascent={} descent={} float_descent={} height={} line_spacing={}",
            node.Id(),
            node.Name(),
            size,
            style.GetFontDescription().Weight().ToFloat(),
            metrics.Ascent(),
            metrics.FloatAscent(),
            metrics.Descent(),
            metrics.FloatDescent(),
            metrics.Height(),
            metrics.LineSpacing(),
        );
    }

    let frame = page.CurrentFrame().unwrap();
    for (chunk_index, chunk) in frame.display_items.chunks.iter().enumerate() {
        for (semantic_offset, recorded) in frame
            .display_items
            .display_items
            .ItemsInRange(chunk.begin_index as usize, chunk.end_index as usize)
            .iter()
            .enumerate()
        {
            let headline = frame.display_items.items[recorded.record_begin..recorded.record_end]
                .iter()
                .any(|op| {
                    op.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
                        && op.font_size as i32 == 56
                });
            if !headline {
                continue;
            }
            eprintln!(
                "HEADLINE_CHUNK chunk={} semantic={} id={:?} chunk_bounds={:?} drawable={:?} item_visual={:?} record={}..{} has_text={} transform_id={}",
                chunk_index,
                chunk.begin_index as usize + semantic_offset,
                recorded.id,
                chunk.bounds,
                chunk.drawable_bounds,
                recorded.visual_rect,
                recorded.record_begin,
                recorded.record_end,
                recorded.record_has_text,
                chunk.properties.transform.id,
            );
            let mut transform = Some(chunk.properties.transform.clone());
            while let Some(node) = transform {
                eprintln!(
                    "  CHUNK_TRANSFORM id={} tx={} ty={} reasons={:?}",
                    node.id,
                    node.matrix.values[12],
                    node.matrix.values[13],
                    node.direct_compositing_reasons,
                );
                transform = node.parent.clone();
            }
        }
    }
    for (item_index, item) in frame.display_items.items.iter().enumerate() {
        if (50..90).contains(&item_index) {
            eprintln!(
                "RAW_ITEM index={} type={:?} node={} tx={} ty={} rect={:?}",
                item_index,
                item.r#type,
                item.node_id,
                item.transform.values[12],
                item.transform.values[13],
                item.rect,
            );
        }
    }
    for (item_index, item) in frame
        .display_items
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            !item.glyphs.is_empty()
                && item.text_blob_origin.y < 720.0
                && matches!(item.font_size as i32, 14 | 16 | 36 | 56)
        })
    {
        eprintln!(
            "GLYPH index={} node={} rect={:?} origin={:?} line_top={} line_height={} size={} baseline={} count={}",
            item_index,
            item.node_id,
            item.rect,
            item.text_blob_origin,
            item.text_line_top,
            item.text_line_height,
            item.font_size,
            item.baseline,
            item.glyphs.len(),
        );
        if item.font_size as i32 == 56 {
            for (glyph_index, glyph) in item.glyphs.iter().enumerate() {
                eprintln!(
                    "  GLYPH_POSITION index={} id={} x={} y={} canvas_rotation={}",
                    glyph_index,
                    glyph.id,
                    glyph.offset.x + item.text_blob_origin.x,
                    glyph.offset.y + item.text_blob_origin.y,
                    glyph.canvas_rotation,
                );
            }
            let mut dom_index = document.FindNodeById(item.node_id);
            while let Some(index) = dom_index {
                let dom_node = document.Node(index);
                eprintln!(
                    "  DOM id={} name={} data={:?} class={:?}",
                    dom_node.Id(),
                    dom_node.Name(),
                    dom_node.Data(),
                    dom_node
                        .FindAttribute("class")
                        .map(|attribute| attribute.value.as_str()),
                );
                dom_index = dom_node.Parent();
            }
            let chunk = frame
                .display_items
                .chunks
                .iter()
                .find(|chunk| {
                    chunk.begin_index as usize <= item_index
                        && item_index < chunk.end_index as usize
                })
                .expect("headline glyph paint chunk");
            let mut transform = Some(chunk.properties.transform.clone());
            while let Some(node) = transform {
                eprintln!(
                    "  TRANSFORM id={} tx={} ty={} reasons={:?}",
                    node.id,
                    node.matrix.values[12],
                    node.matrix.values[13],
                    node.direct_compositing_reasons,
                );
                transform = node.parent.clone();
            }
        }
    }
}

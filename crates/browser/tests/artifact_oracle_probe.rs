use browser::page::{Page, PageClient};
use std::{cell::RefCell, rc::Rc};

struct Client;
impl PageClient for Client {}

fn open(url: &str, width: u32, height: u32, scale: f64) -> Page {
    let options = url_loader::DefaultURLLoaderOptions::default();
    let loader = Rc::new(RefCell::new(
        url_loader::DefaultURLLoader::new(options).unwrap(),
    ));
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(width, height);
    constraints.device_pixel_ratio = scale;
    let mut page = Page::Create(
        loader,
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(document_image::SVGImageDecoder::new(
            &assembly,
        ))),
        constraints,
        None,
        Some(Rc::new(RefCell::new(Client))),
    );
    page.Open(url, 16384, 4096).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(25);
    while page.IsLoading() && std::time::Instant::now() < deadline {
        page.RunFor(std::time::Duration::from_millis(5)).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    page
}

fn find_fragment<'a>(
    root: &'a layoutng_assembly::fragment_tree::FragmentNode,
    id: u64,
) -> Option<&'a layoutng_assembly::fragment_tree::FragmentNode> {
    if root.node_id == id {
        Some(root)
    } else {
        root.children
            .iter()
            .find_map(|child| find_fragment(child, id))
    }
}

fn find_fragment_absolute(
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
        .find_map(|child| find_fragment_absolute(child, wanted, absolute))
}

fn dump_fragment_path(
    node: &layoutng_assembly::fragment_tree::FragmentNode,
    wanted: u64,
    parent: layoutng_assembly::internal::layout_input::Offset,
) -> bool {
    let absolute = layoutng_assembly::internal::layout_input::Offset {
        x: parent.x + node.offset.x,
        y: parent.y + node.offset.y,
    };
    let contains = node.node_id == wanted
        || node
            .children
            .iter()
            .any(|child| dump_fragment_path(child, wanted, absolute));
    if contains {
        eprintln!(
            "GITHUB_NAV_PATH node={} kind={:?} offset={:?} absolute={:?} size={:?} content={:?}",
            node.node_id, node.kind, node.offset, absolute, node.size, node.content_size
        );
    }
    contains
}

#[test]
fn probe_github_page_layout_styles() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open(
        "http://127.0.0.1:9555/github-static-oracle-zero.html",
        1280,
        713,
        2.0,
    );
    let frame = page.CurrentFrame().unwrap();
    if false {
        let mut layer_tile = layer_tile::LayerTileEngine::new(512);
        let update = layer_tile
            .UpdatePending(
                &frame.display_items,
                layer_tile::FrameConfig {
                    viewport: paint::paint_engine::PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 1280.0,
                        height: 713.0,
                    },
                    raster_scale: 2.0,
                    device_scale_factor: 2.0,
                    layout_zoom_factor: 2.0,
                    activation_scroll: None,
                    frame_time: None,
                },
            )
            .unwrap();
        for layer in &update.frame_plan.layers {
            eprintln!(
            "GITHUB_LAYER id={:?} first={} transparent={} complete={} bounds={:?} transform={} clip={} effect={} chunks={:?} records={:?}",
            layer.id, layer.is_first_layer, layer.requires_transparent_backing, layer.bounds_are_complete, layer.bounds,
            layer.properties.transform.id, layer.properties.clip.id, layer.properties.effect.id,
            layer.chunk_indices, layer.record_indices,
        );
            let code_records: Vec<_> = layer
                .record_indices
                .iter()
                .filter_map(|&index| {
                    let record = &update.frame_plan.GetRasterRecords()[index];
                    record
                        .items
                        .iter()
                        .any(|item| item.node_id == 1392)
                        .then_some((index, record.visual_rect))
                })
                .collect();
            if !code_records.is_empty() {
                let code_items: Vec<_> = layer
                    .record_indices
                    .iter()
                    .map(|&index| {
                        (
                            index,
                            update.frame_plan.GetRasterRecords()[index].bounds_are_complete,
                            update.frame_plan.GetRasterRecords()[index]
                                .items
                                .iter()
                                .map(|item| (item.r#type, item.rect, item.blur_radius))
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect();
                eprintln!(
                "GITHUB_CODE_LAYER id={:?} first={} transparent={} bounds={:?} transform={} clip={} effect={} chunks={:?} records={:?} items={:?}",
                layer.id,
                layer.is_first_layer,
                layer.requires_transparent_backing,
                layer.bounds,
                layer.properties.transform.id,
                layer.properties.clip.id,
                layer.properties.effect.id,
                layer.chunk_indices,
                code_records,
                code_items,
            );
            }
            let gradients: Vec<_> = layer
                .record_indices
                .iter()
                .flat_map(|&index| {
                    update.frame_plan.GetRasterRecords()[index]
                        .items
                        .iter()
                        .filter_map(move |item| {
                            matches!(
                                item.r#type,
                                paint::paint_engine::DisplayItemType::kDrawGradientRect
                                    | paint::paint_engine::DisplayItemType::kDrawTiledGradient
                            )
                            .then_some((
                                index,
                                item.node_id,
                                item.rect,
                            ))
                        })
                })
                .collect();
            if !gradients.is_empty() {
                eprintln!(
                "GITHUB_GRADIENT_LAYER id={:?} bounds={:?} complete={} origin={:?} translation={:?} tile_indices={:?} records={:?}",
                layer.id,
                layer.bounds,
                layer.bounds_are_complete,
                layer.raster_origin,
                layer.root_translation,
                layer.tiles.iter().map(|tile| (tile.tile_index, tile.tile_rect, tile.pixel_size)).collect::<Vec<_>>(),
                gradients,
            );
            }
        }
    }
    let owner = page.Document();
    let document = owner.GetDocument();
    if let Some(nav) = find_fragment(&frame.fragments, 1105) {
        for child in &nav.children {
            eprintln!(
                "GITHUB_NAV_CHILD node={} offset={:?} size={:?} kind={:?}",
                child.node_id, child.offset, child.size, child.kind
            );
        }
    }
    dump_fragment_path(
        &frame.fragments,
        1149,
        layoutng_assembly::internal::layout_input::Offset::default(),
    );
    for index in 0..document.NodeCount() {
        let node = document.Node(index);
        if (1105..=1157).contains(&node.Id()) {
            let style = document.ResolvedStyleFor(index);
            eprintln!(
                "GITHUB_NAV_NODE index={index} id={} name={} data={:?} class={:?} parent={:?} fragment={:?} style={:?}",
                node.Id(),
                node.Name(),
                node.Data(),
                node.FindAttribute("class").map(|attribute| attribute.value.as_str()),
                node.Parent().map(|parent| document.Node(parent).Id()),
                find_fragment(&frame.fragments, node.Id())
                    .map(|fragment| (fragment.offset, fragment.size, fragment.content_size)),
                style.map(|resolved| {
                    let native = unsafe { &*resolved.native_style.Get() };
                    (
                        (native.EffectiveZoom(), native.Display()),
                        (native.Width(), native.Height(), native.MinWidth(), native.MinHeight()),
                        (native.PaddingTop(), native.PaddingRight(), native.PaddingBottom(), native.PaddingLeft()),
                        (native.RowGap(), native.ColumnGap()),
                        (resolved.style.padding, resolved.style.gap),
                        resolved.style.extended.as_ref().map(|extra| (extra.row_gap, extra.column_gap)),
                    )
                }),
            );
        }
        if node.FindAttribute("class").is_some_and(|attribute| {
            attribute.value.contains("octicon-play")
                || attribute.value.contains("octicon-issue-opened")
        }) {
            eprintln!(
                "GITHUB_TARGET_SVG index={index} id={} class={:?} fragment={:?}",
                node.Id(),
                node.FindAttribute("class")
                    .map(|attribute| attribute.value.as_str()),
                find_fragment_absolute(
                    &frame.fragments,
                    node.Id(),
                    layoutng_assembly::internal::layout_input::Offset::default(),
                )
            );
        }
        if node
            .FindAttribute("class")
            .is_some_and(|attribute| attribute.value.contains("CounterLabel"))
        {
            let parent = node.Parent().map(|parent| document.Node(parent));
            let grandparent = parent
                .and_then(|parent| parent.Parent())
                .map(|parent| document.Node(parent));
            eprintln!("GITHUB_COUNTER index={index} id={} parent={:?} parent_fragment={:?} grandparent={:?} grandparent_fragment={:?} fragment={:?}",
                node.Id(), parent.map(|n| n.Id()), parent.and_then(|n| find_fragment(&frame.fragments, n.Id())).map(|f| (f.offset, f.size, f.content_size)),
                grandparent.map(|n| n.Id()), grandparent.and_then(|n| find_fragment(&frame.fragments, n.Id())).map(|f| (f.offset, f.size, f.content_size)),
                find_fragment(&frame.fragments, node.Id()).map(|f| (f.offset, f.size, f.content_size)));
        }
    }
    for index in 0..document.NodeCount() {
        let node = document.Node(index);
        if (1388..=1405).contains(&node.Id()) {
            let fragment = find_fragment(&frame.fragments, node.Id());
            let style = document.ResolvedStyleFor(index);
            eprintln!(
                "GITHUB_CODE_NODE index={index} id={} name={} text={:?} class={:?} parent={:?} fragment={:?} style={:?}",
                node.Id(),
                node.Name(),
                node.Data(),
                node.FindAttribute("class").map(|a| a.value.as_str()),
                node.Parent().map(|parent| document.Node(parent).Id()),
                fragment.map(|f| (f.offset, f.size, f.content_size)),
                style.map(|s| (&s.style.display, &s.style.width, &s.style.min_width, &s.style.flex_basis, s.style.flex_grow, s.style.flex_shrink)),
            );
        }
    }
    for item in frame.display_items.items.iter() {
        if (1117..=1120).contains(&(item.node_id & 0xffff)) {
            eprintln!(
                "GITHUB_ISSUE_PATH node={} type={:?} rect={:?} transform={:?} even_odd={} aa={} path={:?}",
                item.node_id,
                item.r#type,
                item.rect,
                item.transform,
                item.even_odd,
                item.antialias,
                item.path,
            );
        }
        if item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
            && (170.0..=210.0).contains(&item.text_blob_origin.y)
        {
            eprintln!(
                "GITHUB_HEADER_GLYPH node={} origin={:?} line_top={} line_height={} baseline={} glyphs={:?}",
                item.node_id,
                item.text_blob_origin,
                item.text_line_top,
                item.text_line_height,
                item.baseline,
                item.glyphs.iter().map(|glyph| glyph.id).collect::<Vec<_>>()
            );
        }
        if item.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
            && item.rect.y >= 130.0
            && item.rect.y <= 165.0
            && item.rect.x >= 500.0
            && item.rect.x <= 720.0
        {
            eprintln!(
                "GITHUB_NAV_GLYPH node={} rect={:?} origin={:?} face={} size={} smoothing={:?} bold={} italic={} variations={:?} glyphs={:?}",
                item.node_id,
                item.rect,
                item.text_blob_origin,
                item.font_face_index,
                item.font_size,
                item.font_smoothing,
                item.synthetic_bold,
                item.synthetic_italic,
                item.font_variations,
                item.glyphs.iter().map(|glyph| (glyph.id, glyph.offset, glyph.advance)).collect::<Vec<_>>(),
            );
        }
        if (item.node_id & 0xffff) != 0
            && (item.rect.x <= 635.0 && item.rect.x + item.rect.width >= 610.0)
            && (item.rect.y <= 110.0 && item.rect.y + item.rect.height >= 80.0)
        {
            eprintln!(
                "GITHUB_CRUMB_ITEM node={} type={:?} rect={:?} transform={:?} origin={:?} font_size={} line_top={} line_height={} baseline={}",
                item.node_id,
                item.r#type,
                item.rect,
                item.transform,
                item.text_blob_origin,
                item.font_size,
                item.text_line_top,
                item.text_line_height,
                item.baseline,
            );
        }
        if (1090..=1105).contains(&(item.node_id & 0xffff)) {
            eprintln!(
                "GITHUB_CRUMB_OWNER node={} type={:?} rect={:?} transform={:?} color={:?} stroke={} origin={:?}",
                item.node_id,
                item.r#type,
                item.rect,
                item.transform,
                item.color,
                item.stroke_width,
                item.text_blob_origin,
            );
        }
        if item.r#type == paint::paint_engine::DisplayItemType::kDrawBoxShadow {
            eprintln!(
                "GITHUB_SHADOW node={} rect={:?} transform={:?} blur={} spread={} inset={} color={:?} radii={:?} offset={:?}",
                item.node_id,
                item.rect,
                item.transform,
                item.blur_radius,
                item.spread,
                item.inset,
                item.color,
                item.corner_radii,
                item.shadow_offset,
            );
        }
        if (1880..=1895).contains(&(item.node_id & 0xffff)) {
            eprintln!(
                "GITHUB_SIDEBAR_NODE node={} type={:?} rect={:?} transform={:?} path_bounds=({:?},{:?})",
                item.node_id,
                item.r#type,
                item.rect,
                item.transform,
                item.path.first(),
                item.path.last(),
            );
        }
        if item.rect.x >= 950.0
            && item.rect.x <= 1000.0
            && item.rect.y >= 340.0
            && item.rect.y <= 380.0
        {
            eprintln!(
                "GITHUB_SIDEBAR_ICON node={} type={:?} rect={:?} transform={:?}",
                item.node_id, item.r#type, item.rect, item.transform
            );
        }
        if matches!(
            item.r#type,
            paint::paint_engine::DisplayItemType::kDrawGradientRect
                | paint::paint_engine::DisplayItemType::kDrawTiledGradient
        ) {
            eprintln!(
                "GITHUB_GRADIENT node={} type={:?} rect={:?} tile={:?} phase={:?} shader={:?}",
                item.node_id,
                item.r#type,
                item.rect,
                item.tile_rect,
                item.tile_phase,
                item.paint_shader.as_ref().map(|shader| (
                    shader.start,
                    shader.end,
                    shader.stops.clone(),
                    shader.transform,
                    shader.interpolate_premultiplied,
                )),
            );
        }
    }
    if document.NodeCount() > 1443 {
        let node = document.Node(1443);
        let resolved = document.ResolvedStyleFor(1443);
        eprintln!(
            "GITHUB_NODE_1444 name={} class={:?} children={:?} style={:?}",
            node.Name(),
            node.FindAttribute("class").map(|a| a.value.as_str()),
            node.Children().to_vec(),
            resolved.map(|value| {
                let native = unsafe { &*value.native_style.Get() };
                (
                    value.style.height,
                    native.Display(),
                    native.PaddingTop(),
                    native.PaddingRight(),
                    native.PaddingBottom(),
                    native.PaddingLeft(),
                    native.Height(),
                )
            }),
        );
    }
    if let Some(mut index) = (0..document.NodeCount()).find(|&index| {
        document
            .Node(index)
            .FindAttribute("placeholder")
            .is_some_and(|attribute| attribute.value == "Go to file")
    }) {
        for depth in 0..8 {
            let node = document.Node(index);
            let fragment = find_fragment(&frame.fragments, node.Id());
            let style = document.ResolvedStyleFor(index);
            eprintln!("GITHUB_GOTO depth={depth} index={index} id={} name={} class={:?} fragment={:?} style={:?}",
                node.Id(), node.Name(), node.FindAttribute("class").map(|a| a.value.as_str()),
                fragment.map(|f| (f.offset, f.size, f.content_size)),
                style.map(|s| (&s.style.display, &s.style.width)));
            if depth == 0 {
                if let Some(resolved) = style {
                    let native = unsafe { &*resolved.native_style.Get() };
                    eprintln!(
                        "GITHUB_GOTO_PAINT border={:?} bg={:?} color={:?} border_widths=({}, {}, {}, {})",
                        native.BorderTopColor().GetColor(),
                        native.BackgroundColor().GetColor(),
                        native.Color().GetColor(),
                        native.BorderTopWidth(),
                        native.BorderRightWidth(),
                        native.BorderBottomWidth(),
                        native.BorderLeftWidth(),
                    );
                    let font = unsafe { &*native.GetFont() };
                    let primary = font.PrimaryFont();
                    if !primary.is_null() {
                        let primary = unsafe { &*primary };
                        eprintln!(
                            "GITHUB_GOTO_FONT size={} avg={} max={} zero={} family={:?}",
                            native.ComputedFontSize(),
                            primary.AvgCharWidth(),
                            primary.MaxCharWidth(),
                            primary.GetFontMetrics().ZeroWidth(),
                            font.GetFontDescription().Family().FamilyName()
                        );
                    }
                }
            }
            let Some(parent) = node.Parent() else { break };
            index = parent;
        }
    }
    for wanted in [
        "prc-PageLayout-PageLayoutContent-BneH9",
        "prc-PageLayout-ContentWrapper-gR9eG",
        "prc-PageLayout-PaneWrapper-pHPop",
        "prc-PageLayout-Pane-AyzHK",
    ] {
        let index = (0..document.NodeCount())
            .find(|&index| {
                document
                    .Node(index)
                    .FindAttribute("class")
                    .is_some_and(|attribute| {
                        attribute
                            .value
                            .split_ascii_whitespace()
                            .any(|class_name| class_name == wanted)
                    })
            })
            .unwrap();
        let node = document.Node(index);
        let resolved = document.ResolvedStyleFor(index).unwrap();
        let style = &resolved.style;
        let native = unsafe { &*resolved.native_style.Get() };
        let fragment = find_fragment(&frame.fragments, node.Id());
        eprintln!(
            "GITHUB_STYLE class={wanted} id={} display={:?} width={:?} min_width={:?} max_width={:?} flex_basis={:?} grow={} shrink={} wrap={} native_display={:?} native_width={:?} native_basis={:?} native_grow={} native_shrink={} fragment={:?}",
            node.Id(),
            style.display,
            style.width,
            style.min_width,
            style.max_width,
            style.flex_basis,
            style.flex_grow,
            style.flex_shrink,
            style.flex_wrap,
            native.Display(),
            native.Width(),
            native.FlexBasis(),
            native.FlexGrow(),
            native.FlexShrink(),
            fragment.map(|f| (f.offset, f.size, f.content_size)),
        );
    }
    let mut pixels = vec![0_u32; 2560 * 1426];
    page.PaintArtifactInto(
        &frame.display_items,
        2560,
        1426,
        2.0,
        &mut pixels,
        renderer::PixelFormat::Rgba8888,
        2560,
    )
    .unwrap();
    let bytes: Vec<u8> = pixels.into_iter().flat_map(u32::to_ne_bytes).collect();
    std::fs::write(
        "/private/tmp/rechrom-github-test-frame.png",
        raster::EncodeRgbaPng(&bytes, 2560, 1426),
    )
    .unwrap();
    let single_tile = raster::pure_replay::RasterizeSourceDisplayItemListWithScale(
        &frame.display_items,
        2560,
        1426,
        2.0,
    );
    std::fs::write(
        "/private/tmp/rechrom-github-single-tile.png",
        raster::EncodeRgbaPng(&single_tile, 2560, 1426),
    )
    .unwrap();
    #[cfg(feature = "source_png")]
    {
        let native = raster::source_replay::RasterizeSourceDisplayItemListWithScale(
            &frame.display_items,
            2560,
            1426,
            2.0,
        );
        std::fs::write(
            "/private/tmp/rechrom-github-native-skia.png",
            raster::EncodeRgbaPng(&native, 2560, 1426),
        )
        .unwrap();
    }
}

#[test]
fn probe_input_intrinsic_widths() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open("http://127.0.0.1:9555/input-intrinsic.html", 1000, 300, 2.0);
    let frame = page.CurrentFrame().unwrap();
    let owner = page.Document();
    let document = owner.GetDocument();
    for index in 0..document.NodeCount() {
        let node = document.Node(index);
        let Some(id) = node.FindAttribute("id") else {
            continue;
        };
        if !id.value.starts_with('i') {
            continue;
        }
        eprintln!(
            "INPUT_INTRINSIC {} {:?}",
            id.value,
            find_fragment(&frame.fragments, node.Id()).map(|f| (f.offset, f.size, f.content_size))
        );
    }
}

#[test]
fn probe_table_row_height() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open("file:///private/tmp/table-row-height.html", 500, 300, 2.0);
    let owner = page.Document();
    let document = owner.GetDocument();
    for index in 0..document.NodeCount() {
        let node = document.Node(index);
        if node.Name() != "tr" {
            continue;
        }
        let value = document.ResolvedStyleFor(index).unwrap();
        let native = unsafe { &*value.native_style.Get() };
        eprintln!("TABLE_ROW index={index} public_height={:?} native_height={:?} padding={:?}/{:?}/{:?}/{:?}", value.style.height, native.Height(), native.PaddingTop(), native.PaddingRight(), native.PaddingBottom(), native.PaddingLeft());
    }
}

#[test]
fn probe_gradient_oracle() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open("http://127.0.0.1:9555/gradient-oracle.html", 1280, 713, 2.0);
    let frame = page.CurrentFrame().unwrap();
    for item in frame.display_items.items.iter().filter(|item| {
        matches!(
            item.r#type,
            paint::paint_engine::DisplayItemType::kDrawGradientRect
                | paint::paint_engine::DisplayItemType::kDrawTiledGradient
        )
    }) {
        eprintln!(
            "GRADIENT_ORACLE type={:?} rect={:?} tile={:?} phase={:?} shader={:?}",
            item.r#type, item.rect, item.tile_rect, item.tile_phase, item.paint_shader
        );
    }
    let mut pixels = vec![0_u32; 2560 * 1426];
    page.PaintArtifactInto(
        &frame.display_items,
        2560,
        1426,
        2.0,
        &mut pixels,
        renderer::PixelFormat::Rgba8888,
        2560,
    )
    .unwrap();
    let bytes: Vec<u8> = pixels.into_iter().flat_map(u32::to_ne_bytes).collect();
    std::fs::write(
        "/private/tmp/rechrom-gradient-oracle.png",
        raster::EncodeRgbaPng(&bytes, 2560, 1426),
    )
    .unwrap();
}

#[test]
fn probe_svg_flex_oracle() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open("file:///private/tmp/svg-flex-oracle.html", 1280, 713, 2.0);
    let frame = page.CurrentFrame().unwrap();
    fn dump_fragment(fragment: &layoutng_assembly::fragment_tree::FragmentNode, depth: usize) {
        if depth >= 2 || format!("{:?}", fragment.paint.source_kind).starts_with("kSvg") {
            eprintln!(
                "SVG_FLEX_FRAGMENT depth={depth} node={} kind={:?} offset={:?} size={:?}",
                fragment.node_id, fragment.paint.source_kind, fragment.offset, fragment.size
            );
        }
        for child in &fragment.children {
            dump_fragment(child, depth + 1);
        }
    }
    dump_fragment(&frame.fragments, 0);
    for (item_index, item) in frame.display_items.items.iter().enumerate() {
        if !item.path.is_empty() {
            eprintln!(
                "SVG_FLEX_ITEM node={} type={:?} rect={:?} transform={:?} path={:?}",
                item.node_id, item.r#type, item.rect, item.transform, item.path
            );
            for (record_index, record) in frame.display_items.display_items.iter().enumerate() {
                if item_index < record.record_begin || item_index >= record.record_end {
                    continue;
                }
                for chunk in &frame.display_items.chunks {
                    if record_index < chunk.begin_index as usize
                        || record_index >= chunk.end_index as usize
                    {
                        continue;
                    }
                    let mut transform = Some(&chunk.properties.transform);
                    while let Some(node) = transform {
                        eprintln!("SVG_FLEX_TRANSFORM id={} matrix={:?}", node.id, node.matrix);
                        transform = node.parent.as_ref();
                    }
                }
            }
        }
    }
    let mut pixels = vec![0_u32; 2560 * 1426];
    page.PaintArtifactInto(
        &frame.display_items,
        2560,
        1426,
        2.0,
        &mut pixels,
        renderer::PixelFormat::Rgba8888,
        2560,
    )
    .unwrap();
    let bytes: Vec<u8> = pixels.into_iter().flat_map(u32::to_ne_bytes).collect();
    std::fs::write(
        "/private/tmp/rechrom-svg-flex-subpixel-test.png",
        raster::EncodeRgbaPng(&bytes, 2560, 1426),
    )
    .unwrap();
    #[cfg(feature = "source_png")]
    {
        let native = raster::source_replay::RasterizeSourceDisplayItemListWithScale(
            &frame.display_items,
            2560,
            1426,
            2.0,
        );
        std::fs::write(
            "/private/tmp/rechrom-svg-flex-native-skia.png",
            raster::EncodeRgbaPng(&native, 2560, 1426),
        )
        .unwrap();
    }
}

#[test]
fn probe_github_svg_offsets() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open(
        "http://127.0.0.1:9555/github-static-oracle-zero.html",
        1280,
        713,
        2.0,
    );
    let frame = page.CurrentFrame().unwrap();
    let owner = page.Document();
    let document = owner.GetDocument();
    let wanted: Vec<_> = (0..document.NodeCount())
        .filter_map(|index| {
            let node = document.Node(index);
            node.FindAttribute("class")
                .is_some_and(|value| {
                    value
                        .value
                        .split_ascii_whitespace()
                        .any(|class| class == "octicon-pulse")
                })
                .then_some(node.Id())
        })
        .collect();
    fn walk(
        node: &layoutng_assembly::fragment_tree::FragmentNode,
        parent: layoutng_assembly::internal::layout_input::Offset,
        wanted: &[u64],
    ) {
        let absolute = layoutng_assembly::internal::layout_input::Offset {
            x: parent.x + node.offset.x,
            y: parent.y + node.offset.y,
        };
        if wanted.contains(&node.node_id)
            || format!("{:?}", node.paint.source_kind).starts_with("kSvg")
                && absolute.y > 350.0
                && absolute.y < 500.0
        {
            eprintln!("GITHUB_SVG_FRAGMENT node={} kind={:?} relative={:?} absolute={absolute:?} size={:?}", node.node_id, node.paint.source_kind, node.offset, node.size);
        }
        for child in &node.children {
            walk(child, absolute, wanted);
        }
    }
    eprintln!("GITHUB_PULSE_IDS {wanted:?}");
    walk(
        &frame.fragments,
        layoutng_assembly::internal::layout_input::Offset::default(),
        &wanted,
    );
    for item in frame
        .display_items
        .items
        .iter()
        .filter(|item| wanted.contains(&item.node_id))
    {
        eprintln!(
            "GITHUB_PULSE_ITEM node={} type={:?} rect={:?} path_first={:?}",
            item.node_id,
            item.r#type,
            item.rect,
            item.path.first()
        );
    }
}

#[test]
fn probe_pulse_inline_offsets() {
    let _heap = foundation::LayoutHeapScope::new();
    let page = open("http://127.0.0.1:9555/pulse-inline.html", 1280, 713, 2.0);
    let frame = page.CurrentFrame().unwrap();
    fn walk(
        node: &layoutng_assembly::fragment_tree::FragmentNode,
        parent: layoutng_assembly::internal::layout_input::Offset,
        depth: usize,
    ) {
        let absolute = layoutng_assembly::internal::layout_input::Offset {
            x: parent.x + node.offset.x,
            y: parent.y + node.offset.y,
        };
        eprintln!("PULSE_FRAGMENT depth={depth} node={} kind={:?} relative={:?} absolute={absolute:?} size={:?}", node.node_id, node.paint.source_kind, node.offset, node.size);
        for child in &node.children {
            walk(child, absolute, depth + 1);
        }
    }
    walk(
        &frame.fragments,
        layoutng_assembly::internal::layout_input::Offset::default(),
        0,
    );
}

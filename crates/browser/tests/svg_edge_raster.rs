//! Production SVG inline and document-image clip/raster equivalence.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::Page;
use layoutng_assembly::internal::layout_input::FontFace;
use paint::paint_engine::PaintArtifact;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct Loader {
    html: Vec<u8>,
    image: Option<Vec<u8>>,
}
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let (mime_type, body) = if request.url.ends_with("fixture.svg") {
            (
                "image/svg+xml",
                self.image.as_ref().expect("fixture SVG").clone(),
            )
        } else {
            ("text/html", self.html.clone())
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

fn page_fixture(markup: &str, image: Option<&str>) -> Page {
    page_fixture_at_scale(markup, image, 1.0)
}

fn page_fixture_at_scale(markup: &str, image: Option<&str>, scale: f64) -> Page {
    let html =
        format!("<!doctype html><style>body{{margin:0}}svg,img{{display:block}}</style>{markup}");
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(320, 200);
    constraints.device_pixel_ratio = scale;
    constraints.fonts = vec![FontFace {
        family: "SvgPaintOwnershipFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }]
    .into();
    let decoder = browser::CreateSVGImageDecoderWithConstraints(&assembly, &constraints);
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader {
            html: html.into_bytes(),
            image: image.map(|svg| svg.as_bytes().to_vec()),
        })),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(decoder)),
        constraints,
        None,
        None,
    );
    page.Open("https://svg-paint-ownership.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    assert!(page.CurrentFrame().is_some());
    page
}

const FOUNDATION: &str = r###"<svg xmlns="http://www.w3.org/2000/svg" width="42" height="42" viewBox="-1 -1 1026 1026" id="Wikimedia-logo_black"><clipPath id="ga"><path d="m0-33 471 471v363h82V438l471-471v1057H0" /></clipPath><g clip-path="url(#ga)"><path d="M511 224a288 288 0 1 0 2 0" /><path fill="none" stroke="#000" stroke-width="116" d="M511 102a410 410 0 1 0 2 0" /></g><path d="M511 28a158 158 0 1 0 2 0z" /></svg>"###;
const COMMONS: &str = r###"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="47" height="47" viewBox="-306 -517 612 822" id="Commons-logo_sister"><defs><clipPath id="aa"><circle r="298" /></clipPath></defs><circle r="100" fill="#900" /><g fill="#069"><g id="ab" clip-path="url(#aa)"><path d="M-11 180v118h22V180" /><path d="m-43 185 43-75 43 75" /></g><g id="ac"><use xlink:href="#ab" transform="rotate(45)" /><use xlink:href="#ab" transform="rotate(90)" /><use xlink:href="#ab" transform="rotate(135)" /></g><use xlink:href="#ac" transform="scale(-1 1)" /><path fill="none" stroke="#069" stroke-width="84" d="M-181.02-181.02a256 256 0 1 0 362.04 0c-70.71-70.71-177.49-34.65-198.7-190.21" /><path d="M-23-515s-36 135-80 185 116-62 170-5-90-180-90-180z" /></g></svg>"###;

fn render(page: &Page, label: &str) -> Vec<u32> {
    let artifact = &page.CurrentFrame().unwrap().display_items;
    let mut result = vec![0; 320 * 200];
    page.PaintArtifactInto(
        artifact,
        320,
        200,
        1.0,
        &mut result,
        renderer_skia::PixelFormat::Rgba8888,
        320,
    )
    .unwrap();
    if let Ok(dir) = std::env::var("SVG_EDGE_EVIDENCE") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        let bytes: Vec<u8> = result.iter().flat_map(|p| p.to_ne_bytes()).collect();
        std::fs::write(
            dir.join(format!("{label}.png")),
            raster::EncodeRgbaPng(&bytes, 320, 200),
        )
        .unwrap();
        dump_artifact(artifact, &dir.join(format!("{label}.txt")));
        for image in artifact.resources.iter().flat_map(|r| &r.images) {
            if let image_resource::PaintImageContent::Document(content) = &image.content {
                for (i, variant) in content.variants().iter().enumerate() {
                    let record = variant
                        .record
                        .as_any()
                        .downcast_ref::<paint::paint_engine::DocumentPaintArtifactRecord>()
                        .unwrap();
                    dump_artifact(
                        &record.artifact,
                        &dir.join(format!("{label}-document-{i}.txt")),
                    );
                    std::fs::write(
                        dir.join(format!("{label}-container-{i}.txt")),
                        format!(
                            "intrinsic={:?}\nrecord={:?}\nkey={:?}\n",
                            record.intrinsic_size, record.record_size, record.container_key
                        ),
                    )
                    .unwrap();
                }
            }
        }
    }
    result
}
fn dump_artifact(a: &std::sync::Arc<PaintArtifact>, path: &std::path::Path) {
    use std::fmt::Write;
    let mut dump = String::new();
    for (i, c) in a.chunks.iter().enumerate() {
        writeln!(
            dump,
            "chunk {i} records={}..{} transform={:?} clip={:?} effect={:?}",
            c.begin_index,
            c.end_index,
            c.properties.transform,
            c.properties.clip,
            c.properties.effect
        )
        .unwrap();
    }
    for (i, item) in a.items.iter().enumerate() {
        writeln!(dump, "item {i}: {}", describe(item)).unwrap();
    }
    std::fs::write(path, dump).unwrap();
}
#[test]
fn inline_and_fetched_asset_edges_share_record_raster_semantics() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut mismatches = Vec::new();
        for (name, asset, clip) in [("foundation", FOUNDATION, "ga"), ("commons", COMMONS, "aa")] {
            for unclipped in [false, true] {
                let asset = if unclipped {
                    asset.replace(&format!(" clip-path=\"url(#{clip})\""), "")
                } else {
                    asset.into()
                };
                let suffix = if unclipped { "unclipped" } else { "clipped" };
                let inline = page_fixture(&asset, None);
                let fetched = page_fixture(r#"<img src="fixture.svg">"#, Some(&asset));
                let a = render(&inline, &format!("{name}-{suffix}-inline"));
                let b = render(&fetched, &format!("{name}-{suffix}-fetched"));
                let changed = a.iter().zip(&b).filter(|(a, b)| a != b).count();
                let max = a
                    .iter()
                    .zip(&b)
                    .flat_map(|(a, b)| {
                        a.to_ne_bytes()
                            .into_iter()
                            .zip(b.to_ne_bytes())
                            .map(|(a, b)| a.abs_diff(b))
                    })
                    .max()
                    .unwrap();
                eprintln!("SVG_EDGE {name} {suffix}: changed={changed} max={max}");
                if changed != 0 {
                    mismatches.push((name, suffix, changed, max));
                }
            }
        }
        assert!(
            mismatches.is_empty(),
            "cross-route mismatches: {mismatches:?}"
        );
    });
}

fn describe(i: &paint::paint_engine::DisplayItem) -> String {
    format!("kind={:?} rect={:?} transform={:?} path={:?} color={:?} stroke={} AA={} opacity={} source={:?} resource={} container={:?}",i.r#type,i.rect,i.transform.values,i.path,i.color,i.stroke_width,i.antialias,i.opacity,i.source_rect,i.resource_id,i.document_image_container_key)
}

#[test]
fn clipped_asset_edges_match_at_device_scales_fractional_placement_and_backgrounds() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut failures = Vec::new();
        for (name, asset) in [("foundation", FOUNDATION), ("commons", COMMONS)] {
            for (scale, x, y, color) in [
                (1.0, 17.5, 8.25, "#89abcd"),
                (2.0, 0.0, 0.0, "white"),
                (2.0, 17.5, 8.25, "#89abcd"),
            ] {
                let wrapper = |content: &str| {
                    format!("<style>body{{background:{color}}}</style><div style=\"position:absolute;left:{x}px;top:{y}px\">{content}</div>")
                };
                let inline = page_fixture_at_scale(&wrapper(asset), None, scale);
                let fetched = page_fixture_at_scale(
                    &wrapper(r#"<img src="fixture.svg">"#),
                    Some(asset),
                    scale,
                );
                let width = (320.0 * scale) as u32;
                let height = (200.0 * scale) as u32;
                let render = |page: &Page| {
                    let mut pixels = vec![0; (width * height) as usize];
                    page.PaintArtifactInto(
                        &page.CurrentFrame().unwrap().display_items,
                        width,
                        height,
                        scale,
                        &mut pixels,
                        renderer_skia::PixelFormat::Rgba8888,
                        width as usize,
                    )
                    .unwrap();
                    pixels
                };
                let a = render(&inline);
                let b = render(&fetched);
                let changed = a.iter().zip(&b).filter(|(a, b)| a != b).count();
                if let Ok(dir) = std::env::var("SVG_EDGE_EVIDENCE") {
                    let dir = std::path::Path::new(&dir);
                    std::fs::create_dir_all(dir).unwrap();
                    for (label, page, pixels) in
                        [("inline", &inline, &a), ("fetched", &fetched, &b)]
                    {
                        let name = format!("{name}-dpr{scale}-at{x},{y}-{label}");
                        dump_artifact(
                            &page.CurrentFrame().unwrap().display_items,
                            &dir.join(format!("{name}.txt")),
                        );
                        let bytes: Vec<u8> = pixels.iter().flat_map(|p| p.to_ne_bytes()).collect();
                        std::fs::write(
                            dir.join(format!("{name}.png")),
                            raster::EncodeRgbaPng(&bytes, width, height),
                        )
                        .unwrap();
                    }
                }
                eprintln!("SVG_EDGE_VARIANT {name} scale={scale} origin={x},{y} background={color}: changed={changed}");
                if changed != 0 {
                    failures.push((name, scale, x, y, color, changed));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "cross-route variant mismatches: {failures:?}"
        );
    });
}

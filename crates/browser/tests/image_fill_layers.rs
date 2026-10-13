//! Multi-layer sprite geometry must reach native style and document-image replay.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::{Page, PreferredColorScheme};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use layoutng_assembly::internal::layout_input::FontFace;
use page_mutation::PageMutation;
use paint::paint_engine::{DisplayItemType, PaintRect};
use std::sync::Arc;
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
    page_fixture_with_preference(markup, image, 0)
}
fn page_fixture_with_preference(
    markup: &str,
    image: Option<&str>,
    preferred_color_scheme: u8,
) -> Page {
    let html =
        format!("<!doctype html><style>body{{margin:0}}svg,img{{display:block}}</style>{markup}");
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(320, 240);
    constraints.preferred_color_scheme = preferred_color_scheme;
    constraints.fonts = vec![FontFace {
        family: "NestedSVGFixture".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }];
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
    page.Open("https://nested-svg.test/", 16384, 4096).unwrap();
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

const SPRITE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="40" viewBox="0 0 20 40"><svg width="20" height="20"><rect width="20" height="20" fill="red"/></svg><svg y="20" width="20" height="20"><rect width="20" height="20" fill="blue"/></svg></svg>"#;

fn sprite_page(layers: &str, extra: &str) -> Page {
    page_fixture(
        &format!(
            r#"<style>body{{background:white}}
      #tile{{position:absolute;left:40px;top:40px;width:20px;height:20px;
        background-image:{layers};background-repeat:no-repeat;
        background-position:0 -20px;{extra}}}</style><div id=tile></div>"#
        ),
        Some(SPRITE),
    )
}
fn tile_id(page: &Page) -> u64 {
    let owner = page.Document();
    let doc = owner.GetDocument();
    (0..doc.NodeCount())
        .find_map(|index| {
            doc.Node(index)
                .FindAttribute("id")
                .filter(|a| a.value == "tile")
                .map(|_| doc.Node(index).Id())
        })
        .unwrap()
}
fn image_draw(page: &Page) -> &paint::paint_engine::DisplayItem {
    let wanted = tile_id(page);
    page.CurrentFrame()
        .unwrap()
        .display_items
        .items
        .iter()
        .find(|item| item.node_id == wanted && item.r#type == DisplayItemType::kDrawImageRect)
        .expect("non-repeated SVG draw")
}
fn pixels(page: &Page) -> Vec<u32> {
    let mut pixels = vec![0; 320 * 240];
    page.PaintArtifactInto(
        &page.CurrentFrame().unwrap().display_items,
        320,
        240,
        1.0,
        &mut pixels,
        renderer::PixelFormat::Rgba8888,
        320,
    )
    .unwrap();
    pixels
}
fn assert_geometry(page: &Page, size: f64, y: f64) {
    let item = image_draw(page);
    assert_eq!(
        item.rect,
        PaintRect {
            x: 40.0,
            y,
            width: size,
            height: 2.0 * size
        }
    );
    assert_eq!(
        item.source_rect,
        PaintRect {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 40.0
        }
    );
}
fn assert_tile(page: &Page, size: f64, y: f64) {
    assert_geometry(page, size, y);
    let pixels = pixels(page);
    for y in 40..60 {
        for x in 40..60 {
            assert_eq!(
                pixels[y * 320 + x].to_ne_bytes(),
                [0, 0, 255, 255],
                "blue sprite pixel ({x},{y})"
            );
        }
    }
    assert_eq!(pixels[40 * 320 + 39].to_ne_bytes(), [255, 255, 255, 255]);
    assert_eq!(pixels[60 * 320 + 40].to_ne_bytes(), [255, 255, 255, 255]);
}
#[test]
fn one_layer_sprite_crop_control() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = sprite_page("url(fixture.svg)", "");
        assert_tile(&page, 20.0, 20.0);
    });
}
#[test]
fn multilayer_svg_sprite_repeats_position_and_repeat_before_paint() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = sprite_page(
            "linear-gradient(transparent,transparent),url(fixture.svg)",
            "",
        );
        assert_tile(&page, 20.0, 20.0);
        let control = sprite_page("url(fixture.svg)", "");
        assert_eq!(
            pixels(&page),
            pixels(&control),
            "transparent first layer preserves every RGBA pixel"
        );
    });
}
#[test]
fn multilayer_svg_sprite_repeats_background_size_in_paint_geometry() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = sprite_page(
            "linear-gradient(transparent,transparent),url(fixture.svg)",
            "background-size:40px 80px;background-position:0 -40px",
        );
        assert_geometry(&page, 40.0, 0.0);
        let key = image_draw(&page)
            .document_image_container_key
            .as_ref()
            .unwrap();
        assert_eq!(key.width, 40);
        assert_eq!(key.height, 80);
        assert_tile(&page, 40.0, 0.0);
    });
}

#[test]
fn scaled_single_layer_svg_publishes_the_exact_requested_container() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = sprite_page(
            "url(fixture.svg)",
            "background-size:40px 80px;background-position:0 -40px",
        );
        let artifact = &page.CurrentFrame().unwrap().display_items;
        for (id, key) in artifact.DocumentImageContainerRequirements() {
            let resource = artifact
                .resources
                .as_ref()
                .unwrap()
                .images
                .iter()
                .find(|image| image.id == id)
                .unwrap();
            let image_resource::PaintImageContent::Document(content) = &resource.content else {
                panic!("SVG image keeps its document recording");
            };
            assert!(
                content.exact_record_for(&key).is_some(),
                "requested {key:?}; published {:?}",
                content
                    .variants()
                    .iter()
                    .map(|variant| &variant.container_key)
                    .collect::<Vec<_>>()
            );
        }
        assert_tile(&page, 40.0, 0.0);
    });
}

fn assert_fulfilled(artifact: &paint::paint_engine::PaintArtifact) {
    let requirements = artifact.DocumentImageContainerRequirements();
    assert!(
        !requirements.is_empty(),
        "fixture has a document image request"
    );
    for (id, key) in requirements {
        assert!(
            key.preferred_color_scheme_is_explicit,
            "published identity is resolved"
        );
        let resource = artifact
            .resources
            .as_ref()
            .unwrap()
            .images
            .iter()
            .find(|image| image.id == id)
            .unwrap();
        let image_resource::PaintImageContent::Document(content) = &resource.content else {
            panic!("SVG recording");
        };
        assert!(
            content.exact_record_for(&key).is_some(),
            "unfulfilled {key:?}"
        );
    }
}

fn set_tile_style(page: &mut Page, value: &str) {
    page.Apply(PageMutation::DOMMutation(DOMMutation {
        mutation_type: DOMMutationType::kSetAttribute,
        target_node_id: tile_id(page),
        name: "style".into(),
        value: value.into(),
        ..Default::default()
    }))
    .unwrap();
}

fn artifact_pixels(page: &Page, artifact: &Arc<paint::paint_engine::PaintArtifact>) -> Vec<u32> {
    let mut pixels = vec![0; 320 * 240];
    page.PaintArtifactInto(
        artifact,
        320,
        240,
        1.0,
        &mut pixels,
        renderer::PixelFormat::Rgba8888,
        320,
    )
    .unwrap();
    pixels
}

#[test]
fn resized_svg_keeps_old_artifacts_and_reuses_exact_container_variants() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = sprite_page(
            "url(fixture.svg)",
            "background-size:40px 80px;background-position:0 -40px",
        );
        assert_tile(&page, 40.0, 0.0);
        let old = page.CurrentFrame().unwrap().display_items.clone();
        let old_pixels = artifact_pixels(&page, &old);
        let old_resources = old.resources.as_ref().unwrap().clone();
        for (size, y) in [(60.0, -20.0), (10.0, 30.0), (40.0, 0.0)] {
            set_tile_style(
                &mut page,
                &format!(
                    "background-size:{size}px {}px;background-position:0 -{size}px",
                    size * 2.0
                ),
            );
            assert_geometry(&page, size, y);
            assert_fulfilled(&page.CurrentFrame().unwrap().display_items);
            let actual = pixels(&page);
            assert_eq!(actual[45 * 320 + 45].to_ne_bytes(), [0, 0, 255, 255]);
            assert_eq!(
                artifact_pixels(&page, &old),
                old_pixels,
                "retained frame pixels stay immutable"
            );
            assert!(Arc::ptr_eq(old.resources.as_ref().unwrap(), &old_resources));
        }
        assert_tile(&page, 40.0, 0.0);
        let key = image_draw(&page)
            .document_image_container_key
            .as_ref()
            .unwrap();
        let id = image_draw(&page).resource_id;
        let record = |artifact: &paint::paint_engine::PaintArtifact| {
            let resource = artifact
                .resources
                .as_ref()
                .unwrap()
                .images
                .iter()
                .find(|image| image.id == id)
                .unwrap();
            let image_resource::PaintImageContent::Document(content) = &resource.content else {
                panic!()
            };
            content.exact_record_for(key).unwrap().clone()
        };
        assert!(
            Arc::ptr_eq(
                &record(&old),
                &record(&page.CurrentFrame().unwrap().display_items)
            ),
            "returning to a published container reuses its immutable recording"
        );
    });
}

#[test]
fn one_svg_resource_supports_simultaneous_image_and_background_containers() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = page_fixture(
            r#"<img src=fixture.svg style="position:absolute;left:0;top:0;width:20px;height:40px">
          <div style="position:absolute;left:40px;top:0;width:40px;height:80px;background:url(fixture.svg) no-repeat;background-size:40px 80px"></div>
          <div style="position:absolute;left:100px;top:0;width:60px;height:120px;background:url(fixture.svg) no-repeat;background-size:60px 120px"></div>"#,
            Some(SPRITE),
        );
        let artifact = &page.CurrentFrame().unwrap().display_items;
        assert_fulfilled(artifact);
        let required = artifact.DocumentImageContainerRequirements();
        let ids: std::collections::HashSet<_> = required.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            ids.len(),
            1,
            "all clients share one fetched resource identity"
        );
        for size in [(20, 40), (40, 80), (60, 120)] {
            assert!(required
                .iter()
                .any(|(_, key)| (key.width, key.height) == size));
        }
        let actual = pixels(&page);
        for (x, size) in [(5, 20), (45, 40), (105, 60)] {
            assert_eq!(actual[5 * 320 + x].to_ne_bytes(), [255, 0, 0, 255]);
            assert_eq!(actual[(size + 5) * 320 + x].to_ne_bytes(), [0, 0, 255, 255]);
        }
    });
}

#[test]
fn svg_container_identity_resolves_host_preference_per_embedding_and_mutation() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="40" viewBox="0 0 20 40"><style>rect{fill:red}@media(prefers-color-scheme:dark){rect{fill:blue}}</style><rect width="20" height="40"/></svg>"#;
        let mut page = page_fixture(
            r#"<style>.tile{position:absolute;top:0;width:40px;height:80px;background:url(fixture.svg) no-repeat;background-size:40px 80px}</style>
          <div class=tile style="left:0"></div><div class=tile style="left:50px;color-scheme:light"></div><div class=tile style="left:100px;color-scheme:dark"></div>"#,
            Some(svg),
        );
        for dark in [false, true, false] {
            page.SetPreferredColorScheme(if dark {
                PreferredColorScheme::kDark
            } else {
                PreferredColorScheme::kLight
            });
            page.RunTask().unwrap();
            assert_fulfilled(&page.CurrentFrame().unwrap().display_items);
            let actual = pixels(&page);
            let red = [255, 0, 0, 255];
            let blue = [0, 0, 255, 255];
            for (x, color) in [(5, if dark { blue } else { red }), (55, red), (105, blue)] {
                assert_eq!(
                    actual[5 * 320 + x].to_ne_bytes(),
                    color,
                    "dark={dark},x={x}"
                );
            }
        }
    });
}

#[test]
fn scaled_tiled_svg_background_uses_container_pixels() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = page_fixture(
            r#"<div style="width:100px;height:170px;background:url(fixture.svg);background-size:40px 80px"></div>"#,
            Some(SPRITE),
        );
        assert_fulfilled(&page.CurrentFrame().unwrap().display_items);
        let actual = pixels(&page);
        for y in [5, 45, 85, 125, 165] {
            let color = if y % 80 < 40 {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            };
            for x in [5, 45, 85] {
                assert_eq!(actual[y * 320 + x].to_ne_bytes(), color);
            }
        }
    });
}

#[test]
fn svg_reflection_mask_container_is_requested_before_resource_snapshot() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = page_fixture(
            r#"<div style="width:40px;height:80px;background:red;-webkit-box-reflect:below 0 url(fixture.svg) 0 fill stretch"></div>"#,
            Some(SPRITE),
        );
        assert_fulfilled(&page.CurrentFrame().unwrap().display_items);
        // Reflection's downstream property clip is independently unsupported;
        // this regression covers its resource request and exact publication.
        let artifact = &page.CurrentFrame().unwrap().display_items;
        for chunk in &artifact.chunks {
            for filter in &chunk.properties.effect.filters {
                for layer in &filter.reflection_mask_layers {
                    if let (Some(image), Some(key)) =
                        (&layer.image, &layer.document_image_container_key)
                    {
                        let image_resource::PaintImageContent::Document(content) = &image.content
                        else {
                            panic!()
                        };
                        assert!(
                            content.exact_record_for(key).is_some(),
                            "reflection retains the fulfilled snapshot"
                        );
                    }
                }
            }
        }
    });
}

#[test]
fn svg_container_resize_without_viewbox_preserves_fixed_units_and_resolves_percentages() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let page = page_fixture(
            r#"<div style="width:40px;height:80px;background:url(fixture.svg) no-repeat;background-size:40px 80px"></div>"#,
            Some(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="40"><rect width="10" height="10" fill="red"/><svg x="50%" y="50%" width="50%" height="50%"><rect width="20" height="40" fill="blue"/></svg></svg>"#,
            ),
        );
        assert_fulfilled(&page.CurrentFrame().unwrap().display_items);
        let actual = pixels(&page);
        assert_eq!(actual[5 * 320 + 5].to_ne_bytes(), [255, 0, 0, 255]);
        assert_eq!(
            actual[5 * 320 + 15].to_ne_bytes(),
            [255, 255, 255, 255],
            "fixed user units do not stretch with the container"
        );
        assert_eq!(
            actual[45 * 320 + 25].to_ne_bytes(),
            [0, 0, 255, 255],
            "percentages resolve against the new SVG viewport"
        );
        assert_eq!(actual[35 * 320 + 25].to_ne_bytes(), [255, 255, 255, 255]);
    });
}

#[test]
fn initial_host_scheme_agrees_with_later_preference_mutations() {
    native_test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let mut page = page_fixture_with_preference(
            r#"<div style="width:40px;height:80px;background:url(fixture.svg) no-repeat;background-size:40px 80px"></div>"#,
            Some(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="40" viewBox="0 0 20 40"><style>rect{fill:red}@media(prefers-color-scheme:dark){rect{fill:blue}}</style><rect width="20" height="40"/></svg>"#,
            ),
            1,
        );
        assert_fulfilled(&page.CurrentFrame().unwrap().display_items);
        assert_eq!(pixels(&page)[5 * 320 + 5].to_ne_bytes(), [0, 0, 255, 255]);
        page.SetPreferredColorScheme(PreferredColorScheme::kLight);
        page.RunTask().unwrap();
        assert_fulfilled(&page.CurrentFrame().unwrap().display_items);
        assert_eq!(pixels(&page)[5 * 320 + 5].to_ne_bytes(), [255, 0, 0, 255]);
    });
}

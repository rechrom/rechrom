#![allow(non_snake_case)]

use dom::StyleSource;
use image_decoder::image_decoder::{DecodedImage, ImageDecodeInput, ImageDecoder};
use image_decoder::skia_image_decoder::SkiaImageDecoder;
use layoutng_assembly::fragment_tree::FragmentNode;
use layoutng_assembly::internal::boundary::assembly::InstallLayoutBoundary;
use layoutng_assembly::internal::layout_input::{
    ConstraintSpace, OverscrollType, PaintImage, Size, ViewportGeometry,
};
use layoutng_assembly::internal::layout_input_types::{
    ControlThemeMetrics, IntSize, ScrollbarThemeMetrics,
};
use layoutng_assembly::layout_assembly::LayoutAssembly;
pub use layoutng_assembly::layout_engine::LayoutEngine;
use layoutng_block::assembly::InstallBlockAlgorithm;
use layoutng_flex::assembly::InstallFlexAlgorithm;
use layoutng_float::assembly::InstallFloatAlgorithm;
use layoutng_forms::assembly::InstallFormsAlgorithm;
use layoutng_list::assembly::InstallListModule;
use layoutng_replaced::assembly::InstallReplacedAlgorithm;
use layoutng_svg::assembly::InstallSvgModule;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io;
use std::path::Path;

pub mod dom_mutation;
pub mod dynamic_scripts;
mod font_catalog;
#[cfg(feature = "source_paint")]
mod form_submission;
#[cfg(test)]
mod incremental_style_tests;
pub mod interaction_services;
#[cfg(test)]
mod native_test_thread;
#[cfg(feature = "source_paint")]
pub mod page;
#[cfg(all(test, feature = "pure_source_png"))]
mod page_parser_invalidation_tests;
#[cfg(all(test, feature = "pure_source_png"))]
mod page_resource_tests;
#[cfg(all(test, feature = "pure_source_png"))]
mod page_tests;
mod persistent_layout;
pub mod script_execution;
pub mod script_scheduler;
pub mod style_services;
#[cfg(all(test, feature = "source_paint"))]
mod svg_image_tests;
#[cfg(test)]
mod table_layout_tests;
mod text_transform;
pub use persistent_layout::LayoutPersistentDocument;
mod user_agent_styles;

// Shared translated assembly; missing native package slots retain source
// capability errors, and are not replaced with generic boxes.
pub fn CreateLayoutAssembly() -> LayoutAssembly {
    let mut assembly = LayoutAssembly::default();
    InstallBlockAlgorithm(&mut assembly);
    InstallFlexAlgorithm(&mut assembly);
    layoutng_grid::assembly::InstallGridAlgorithm(&mut assembly);
    layoutng_table::assembly::InstallTableAlgorithm(&mut assembly);
    InstallFloatAlgorithm(&mut assembly);
    InstallFormsAlgorithm(&mut assembly);
    InstallReplacedAlgorithm(&mut assembly);
    InstallListModule(&mut assembly);
    InstallSvgModule(&mut assembly);
    #[cfg(feature = "inline_layout")]
    {
        layoutng_assembly::assembly::InstallInlineAlgorithm(&mut assembly);
        layoutng_block::block_inline_layout::InstallBlockInlineSupport(&mut assembly);
        layoutng_out_of_flow::assembly::InstallOutOfFlowAlgorithm(&mut assembly);
    }
    InstallLayoutBoundary(&mut assembly);

    assembly
}

/// Host viewport, system fonts and theme inputs shared by static and live DOM layout.
pub fn CreateBrowserConstraints(width: u32, height: u32) -> ConstraintSpace {
    let started = std::env::var_os("BROWSER_PROFILE_INPUT")
        .is_some()
        .then(std::time::Instant::now);
    let mut space = ConstraintSpace::default();
    space.available_size = Size {
        width: f64::from(width),
        height: f64::from(height),
    };
    space.scrollbar_theme = Some(ScrollbarThemeMetrics {
        auto_thickness: 15,
        thin_thickness: 7,
        minimum_thumb_length: 24,
        has_buttons: true,
        uses_overlay_scrollbars: true,
        ..ScrollbarThemeMetrics::default()
    });
    space.control_theme = ControlThemeMetrics {
        checkbox: Some(IntSize {
            width: 13,
            height: 13,
        }),
        radio: Some(IntSize {
            width: 13,
            height: 13,
        }),
    };
    space.viewport = Some(ViewportGeometry {
        size: IntSize {
            width: width.try_into().expect("viewport width exceeds i32"),
            height: height.try_into().expect("viewport height exceeds i32"),
        },
        overscroll_type: OverscrollType::kTransform,
        ..ViewportGeometry::default()
    });
    space.fonts = font_catalog::DemoFonts();
    space.text_transform = Some(std::sync::Arc::new(
        text_transform::BrowserTextTransformProvider,
    ));
    #[cfg(feature = "source_png")]
    {
        space.font_backend_factory = Some(std::sync::Arc::new(
            raster::font_backend::SkiaFontBackendFactory,
        ));
    }
    if let Some(started) = started {
        eprintln!(
            "browser-constraints-profile width={width} height={height} ms={:.3}",
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
    space
}

/// First static-page profile: block elements, embedded CSS, solid colors and
/// borders. Other page features report unsupported at their owning boundary.
pub fn RenderHtml(source: &str, width: u32, height: u32) -> Vec<u8> {
    RenderWithStyleLoader(
        source,
        width,
        height,
        None,
        |_, _| {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "linked CSS requires a base path or URL",
            ))
        },
        |_, _| {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "image requires a base path or URL",
            ))
        },
        |_, _| {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "font requires a base path or URL",
            ))
        },
    )
    .expect("HTML render failed")
}

fn RenderWithStyleLoader(
    source: &str,
    width: u32,
    height: u32,
    document_url: Option<&str>,
    load_link: impl FnMut(&str, Option<&str>) -> io::Result<cssom::CSSStyleSheet>,
    load_image: impl FnMut(&str, Option<&str>) -> io::Result<DecodedImage>,
    load_font: impl FnMut(&str, Option<&str>) -> io::Result<Vec<u8>>,
) -> io::Result<Vec<u8>> {
    let fragments = LayoutWithStyleLoader(
        source,
        width,
        height,
        document_url,
        load_link,
        load_image,
        load_font,
    )?;
    Ok(RenderFragmentPng(&fragments, width, height))
}

fn RenderFragmentPng(fragments: &FragmentNode, width: u32, height: u32) -> Vec<u8> {
    #[cfg(feature = "pure_source_png")]
    let rgba = raster::pure_replay::RasterizeSourceDisplayItemList(
        &paint::paint_engine::Paint(fragments),
        width,
        height,
    );
    #[cfg(not(feature = "pure_source_png"))]
    let rgba = raster::RasterizeDisplayItemList(&paint::Paint(fragments), width, height);
    raster::EncodeRgbaPng(&rgba, width, height)
}

fn LayoutWithStyleLoader(
    source: &str,
    width: u32,
    height: u32,
    document_url: Option<&str>,
    mut load_link: impl FnMut(&str, Option<&str>) -> io::Result<cssom::CSSStyleSheet>,
    mut load_image: impl FnMut(&str, Option<&str>) -> io::Result<DecodedImage>,
    mut load_font: impl FnMut(&str, Option<&str>) -> io::Result<Vec<u8>>,
) -> io::Result<FragmentNode> {
    // The narrow static parser remains a resource-discovery adapter.  Style,
    // layout-object construction and layout all consume the resident DOM.
    let resource_document = html::Parse(source);
    let resource_base_url = match (document_url, resource_document.base_href.as_deref()) {
        (Some(url), Some(base)) => Some(document_loader::ResolveUrl(url, base)?),
        (Some(url), None) => Some(url.to_owned()),
        (None, _) => None,
    };
    let mut sheets = Vec::new();
    for style_source in &resource_document.style_sources {
        match style_source {
            StyleSource::Inline(text) => {
                let mut sheet = style::ParseCSS(text);
                if let Some(base) = resource_base_url.as_deref() {
                    document_loader::ResolveCSSStyleSheetURLs(&mut sheet, base)?;
                }
                sheets.push(sheet);
            }
            StyleSource::Link(href) => {
                sheets.push(load_link(href, resource_document.base_href.as_deref())?)
            }
        }
    }

    let mut document = html::html_parser::ParseHTML(source);
    for sheet in &sheets {
        document.GetDocumentMut().AppendStyleSheet(sheet.clone());
    }
    let assembly = CreateLayoutAssembly();
    let mut space = CreateBrowserConstraints(width, height);
    style_services::ResolveLayoutStyles(&mut document, &space);
    let used_font_families = (0..document.GetDocument().NodeCount())
        .filter_map(|node| document.GetDocument().ResolvedStyleFor(node))
        .flat_map(|style| style.style.extended.iter())
        .flat_map(|extended| extended.font_families.iter().cloned())
        .collect::<Vec<_>>();
    let web_fonts =
        document_loader::LoadUsedFontFacesForFamilies(&sheets, used_font_families, |url| {
            load_font(url, resource_document.base_href.as_deref())
        });

    // cpp: browser/browser.cc:1241-1289
    // cpp: browser/browser.cc:1359-1393
    // cpp: browser/browser.cc:1761-1771
    let mut image_sources = Vec::new();
    let mut seen_images = HashSet::new();
    for element in &resource_document.elements {
        if element.tag == "img" {
            if let Some((_, source)) = element.attributes.iter().find(|(name, _)| name == "src") {
                if !source.is_empty() && seen_images.insert(source.clone()) {
                    image_sources.push(source.clone());
                }
            }
        }
    }
    let mut images = Vec::new();
    let mut next_image_id = 1_u64;
    for source in image_sources {
        let Ok(decoded) = load_image(&source, resource_document.base_href.as_deref()) else {
            continue;
        };
        let id = next_image_id;
        next_image_id = next_image_id
            .checked_add(1)
            .ok_or_else(|| io::Error::other("image resource id space exhausted"))?;
        document.GetDocumentMut().SetImageResource(
            source,
            dom::ImageResourceMetadata {
                id,
                natural_width: f64::from(decoded.width),
                natural_height: f64::from(decoded.height),
                resolution_scale: 1.0,
            },
        );
        images.push(PaintImage {
            id,
            revision: 1,
            width: decoded.width,
            height: decoded.height,
            resolution_scale: 1.0,
            content: image_resource::PaintImageContent::Bitmap(decoded.rgba8.into()),
        });
    }
    space.fonts.extend(web_fonts);
    space.images = images;
    let mut layout = LayoutEngine::new(&assembly);
    Ok(LayoutPersistentDocument(
        &mut layout,
        &mut document,
        &dom::UserInteractionState::default(),
        &space,
    ))
}

pub fn RenderFile(path: &Path, width: u32, height: u32) -> io::Result<Vec<u8>> {
    let source = std::fs::read_to_string(path)?;
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let loader = RefCell::new(url_loader::DefaultURLLoader::new(
        url_loader::DefaultURLLoaderOptions::default(),
    )?);
    RenderWithStyleLoader(
        &source,
        width,
        height,
        None,
        |href, _base_href| {
            if href.starts_with("http://") || href.starts_with("https://") {
                let response = document_loader::LoadResponse(
                    &mut *loader.borrow_mut(),
                    &url_loader::URLRequest {
                        url: href.to_owned(),
                        destination: url_loader::RequestDestination::kStyleSheet,
                        ..Default::default()
                    },
                )?;
                let mut sheet = style::ParseCSS(&document_loader::DecodeText(&response)?);
                let base = if response.final_url.is_empty() {
                    href
                } else {
                    &response.final_url
                };
                document_loader::ResolveCSSStyleSheetURLs(&mut sheet, base)?;
                Ok(sheet)
            } else {
                Ok(style::ParseCSS(&std::fs::read_to_string(
                    directory.join(href),
                )?))
            }
        },
        |source, _base_href| {
            if source.starts_with("http://") || source.starts_with("https://") {
                let response = document_loader::LoadResponse(
                    &mut *loader.borrow_mut(),
                    &url_loader::URLRequest {
                        url: source.to_owned(),
                        destination: url_loader::RequestDestination::kImage,
                        ..Default::default()
                    },
                )?;
                DecodeRasterImage(&response.body, &response.mime_type)
            } else {
                let bytes = std::fs::read(directory.join(source))?;
                DecodeRasterImage(&bytes, "")
            }
        },
        |source, _base_href| {
            if source.starts_with("http://") || source.starts_with("https://") {
                Ok(document_loader::LoadResponse(
                    &mut *loader.borrow_mut(),
                    &url_loader::URLRequest {
                        url: source.to_owned(),
                        destination: url_loader::RequestDestination::kFont,
                        ..Default::default()
                    },
                )?
                .body)
            } else {
                std::fs::read(directory.join(source))
            }
        },
    )
}

fn DecodeRasterImage(bytes: &[u8], mime_type: &str) -> io::Result<DecodedImage> {
    if mime_type.to_ascii_lowercase().contains("svg") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SVG document image decoder is not connected",
        ));
    }
    let mut decoder = SkiaImageDecoder;
    decoder.Decode(&ImageDecodeInput { bytes, mime_type })
}

pub fn RenderUrl(url: &str, width: u32, height: u32) -> std::io::Result<Vec<u8>> {
    let fragments = LayoutUrl(url, width, height)?;
    Ok(RenderFragmentPng(&fragments, width, height))
}

/// Navigate through the persistent Page and drain tasks as the source CLI does.
#[cfg(feature = "source_paint")]
pub fn LayoutUrl(url: &str, width: u32, height: u32) -> io::Result<FragmentNode> {
    let mut page = page::OpenUrl(
        url,
        width,
        height,
        std::rc::Rc::new(RefCell::new(page::NullPageClient)),
    )?;
    page.RunTask()?;
    page.CurrentFrame()
        .map(|frame| frame.fragments.as_ref().clone())
        .ok_or_else(|| io::Error::other("page has no frame"))
}

#[cfg(not(feature = "source_paint"))]
pub fn LayoutUrl(url: &str, width: u32, height: u32) -> io::Result<FragmentNode> {
    LayoutUrlStatic(url, width, height)
}

/// Explicit static diagnostic profile, without the Page runtime or lifecycle.
pub fn LayoutUrlStatic(url: &str, width: u32, height: u32) -> io::Result<FragmentNode> {
    let loader = RefCell::new(url_loader::DefaultURLLoader::new(
        url_loader::DefaultURLLoaderOptions::default(),
    )?);
    let response = document_loader::LoadResponse(
        &mut *loader.borrow_mut(),
        &url_loader::URLRequest {
            url: url.to_owned(),
            destination: url_loader::RequestDestination::kDocument,
            ..Default::default()
        },
    )?;
    let final_url = response.final_url.clone();
    let source = document_loader::DecodeText(&response)?;
    LayoutWithStyleLoader(
        &source,
        width,
        height,
        Some(&final_url),
        |href, base_href| {
            let base = match base_href {
                Some(base_href) => document_loader::ResolveUrl(&final_url, base_href)?,
                None => final_url.clone(),
            };
            let url = document_loader::ResolveUrl(&base, href)?;
            let response = document_loader::LoadResponse(
                &mut *loader.borrow_mut(),
                &url_loader::URLRequest {
                    url: url.clone(),
                    referrer: final_url.clone(),
                    destination: url_loader::RequestDestination::kStyleSheet,
                    ..Default::default()
                },
            )?;
            let mut sheet = style::ParseCSS(&document_loader::DecodeText(&response)?);
            let base = if response.final_url.is_empty() {
                &url
            } else {
                &response.final_url
            };
            document_loader::ResolveCSSStyleSheetURLs(&mut sheet, base)?;
            Ok(sheet)
        },
        |image_source, base_href| {
            let base = match base_href {
                Some(base_href) => document_loader::ResolveUrl(&final_url, base_href)?,
                None => final_url.clone(),
            };
            let url = document_loader::ResolveUrl(&base, image_source)?;
            let response = document_loader::LoadResponse(
                &mut *loader.borrow_mut(),
                &url_loader::URLRequest {
                    url,
                    referrer: final_url.clone(),
                    destination: url_loader::RequestDestination::kImage,
                    ..Default::default()
                },
            )?;
            DecodeRasterImage(&response.body, &response.mime_type)
        },
        |font_source, base_href| {
            let base = match base_href {
                Some(base_href) => document_loader::ResolveUrl(&final_url, base_href)?,
                None => final_url.clone(),
            };
            let url = document_loader::ResolveUrl(&base, font_source)?;
            Ok(document_loader::LoadResponse(
                &mut *loader.borrow_mut(),
                &url_loader::URLRequest {
                    url,
                    referrer: final_url.clone(),
                    destination: url_loader::RequestDestination::kFont,
                    ..Default::default()
                },
            )?
            .body)
        },
    )
}

#[cfg(feature = "source_paint")]
pub fn BuildSourceDisplayListUrl(
    url: &str,
    width: u32,
    height: u32,
) -> io::Result<paint::paint_engine::PaintArtifact> {
    let mut page = page::OpenUrl(
        url,
        width,
        height,
        std::rc::Rc::new(RefCell::new(page::NullPageClient)),
    )?;
    page.RunTask()?;
    page.CurrentFrame()
        .map(|frame| (*frame.display_items).clone())
        .ok_or_else(|| io::Error::other("page has no frame"))
}

#[cfg(feature = "source_png")]
pub fn RenderUrlSourcePng(url: &str, width: u32, height: u32) -> io::Result<Vec<u8>> {
    let display_list = BuildSourceDisplayListUrl(url, width, height)?;
    let rgba = raster::source_replay::RasterizeSourceDisplayItemList(&display_list, width, height);
    Ok(raster::EncodeRgbaPng(&rgba, width, height))
}

#[cfg(feature = "pure_source_png")]
pub fn RenderUrlPureSourcePng(url: &str, width: u32, height: u32) -> io::Result<Vec<u8>> {
    let display_list = BuildSourceDisplayListUrl(url, width, height)?;
    let rgba = raster::pure_replay::RasterizeSourceDisplayItemList(&display_list, width, height);
    Ok(raster::EncodeRgbaPng(&rgba, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn svg_profile_exports_shape_fragments() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../layoutng/src/pixel_testdata/algorithm_profiles/svg.html");
        let url = format!("file://{}", fixture.display());
        let root = LayoutUrl(&url, 400, 320).unwrap();
        fn inspect(node: &FragmentNode, shapes: &mut usize) {
            if node.paint.svg_shape.is_some() {
                *shapes += 1;
            }
            for child in &node.children {
                inspect(child, shapes);
            }
        }
        let mut shapes = 0;
        inspect(&root, &mut shapes);
        assert_eq!(shapes, 2);
        #[cfg(feature = "source_paint")]
        {
            use paint::paint_engine::DisplayItemType;
            let list = paint::paint_engine::Paint(&root);
            assert!(list
                .items
                .iter()
                .any(|item| item.r#type == DisplayItemType::kDrawRect));
            assert!(list
                .items
                .iter()
                .any(|item| item.r#type == DisplayItemType::kDrawEllipse));
        }
    }

    #[cfg(feature = "pure_source_png")]
    fn generated_after_clears_nested_floats_like_cpp_reference() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pseudo-clearfix.html");
        let url = format!("file://{}", fixture.display());
        let actual = RenderUrlPureSourcePng(&url, 160, 100).unwrap();
        let reference = include_bytes!("../tests/fixtures/pseudo-clearfix.expected.png");
        assert_eq!(actual, reference);
    }

    // Live acceptance probe for the resource path. This site currently
    // redirects HTTP to HTTPS, links a relative stylesheet, and that
    // stylesheet refers to a relative image. Keep it ignored in offline CI.
    #[test]
    #[ignore = "requires access to the public webtemplate site"]
    fn live_webtemplate_redirect_relative_css_and_css_resource() {
        let mut loader =
            url_loader::DefaultURLLoader::new(url_loader::DefaultURLLoaderOptions::default())
                .unwrap();
        let document_response = document_loader::LoadResponse(
            &mut loader,
            &url_loader::URLRequest {
                url: "http://www.cjpj.de/webtemplate.htm".into(),
                destination: url_loader::RequestDestination::kDocument,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(document_response.status_code, 200);
        assert_eq!(
            document_response.final_url,
            "https://www.cjpj.de/webtemplate.htm"
        );
        let source = document_loader::DecodeText(&document_response).unwrap();
        let css_href = "./css/simpleweb.css?080526";
        assert!(source.contains(&format!("href=\"{css_href}\"")));
        let document = html::Parse(&source);
        assert!(document
            .style_sources
            .iter()
            .any(|style| matches!(style, StyleSource::Link(href) if href == css_href)));
        assert_eq!(document.style_sources.len(), 2);
        assert!(document
            .elements
            .iter()
            .any(|element| element.tag == "main"));
        let css_url = document_loader::ResolveUrl(&document_response.final_url, css_href).unwrap();
        assert_eq!(css_url, "https://www.cjpj.de/css/simpleweb.css?080526");
        let css_response = document_loader::LoadResponse(
            &mut loader,
            &url_loader::URLRequest {
                url: css_url,
                referrer: document_response.final_url,
                destination: url_loader::RequestDestination::kStyleSheet,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(css_response.status_code, 200);
        assert_eq!(css_response.mime_type, "text/css");
        let css = document_loader::DecodeText(&css_response).unwrap();
        assert!(css.contains("url(../images/extlink2.png)"));
        let image_url =
            document_loader::ResolveUrl(&css_response.final_url, "../images/extlink2.png").unwrap();
        assert_eq!(image_url, "https://www.cjpj.de/images/extlink2.png");
        let image_response = document_loader::LoadResponse(
            &mut loader,
            &url_loader::URLRequest {
                url: image_url,
                referrer: css_response.final_url,
                destination: url_loader::RequestDestination::kImage,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(image_response.status_code, 200);
        assert_eq!(image_response.mime_type, "image/png");
        assert!(!image_response.body.is_empty());
    }

    const REFERENCE: &[u8] = include_bytes!("../tests/fixtures/block-only-reference.png");
    const HTML: &str = include_str!("../tests/fixtures/block-only.html");
    const VARIANT: &str = include_str!("../tests/fixtures/block-variant.html");
    const VARIANT_REFERENCE: &[u8] =
        include_bytes!("../tests/fixtures/block-variant-reference.png");
    const DEFAULT_MARGIN: &str = include_str!("../tests/fixtures/block-default-margin.html");
    const DEFAULT_MARGIN_REFERENCE: &[u8] =
        include_bytes!("../tests/fixtures/block-default-margin-reference.png");
    const SELECTOR: &str = include_str!("../tests/fixtures/block-selector.html");
    const SELECTOR_REFERENCE: &[u8] =
        include_bytes!("../tests/fixtures/block-selector-reference.png");
    const INLINE_STYLE: &str = include_str!("../tests/fixtures/block-inline-style.html");
    const INLINE_STYLE_REFERENCE: &[u8] =
        include_bytes!("../tests/fixtures/block-inline-style-reference.png");
    const SIDE_BORDERS: &str = include_str!("../tests/fixtures/block-side-borders.html");
    const SIDE_BORDERS_REFERENCE: &[u8] =
        include_bytes!("../tests/fixtures/block-side-borders-reference.png");
    const SIZE_LIMITS: &str = include_str!("../tests/fixtures/block-size-limits.html");
    const SIZE_LIMITS_REFERENCE: &[u8] =
        include_bytes!("../tests/fixtures/block-size-limits-reference.png");
    const SOURCE_BLOCK_PROFILE: &str =
        include_str!("../../../../layoutng/src/pixel_testdata/algorithm_profiles/block.html");
    const SOURCE_BLOCK_PROFILE_REFERENCE: &[u8] =
        include_bytes!("../tests/fixtures/block-profile-reference.png");

    #[test]
    fn local_and_redirected_url_match_cpp_png_bytes() {
        crate::native_test_thread::run(local_and_redirected_url_body);
    }
    fn local_and_redirected_url_body() {
        svg_profile_exports_shape_fragments();
        #[cfg(feature = "pure_source_png")]
        generated_after_clears_nested_floats_like_cpp_reference();
        let plain = "<html><body><div style='width:40px;height:20px;background:#123456'></div></body></html>";
        let hidden = "<html><body><div style='display:none;width:200px;height:200px;background:#ff0000'></div><div style='width:40px;height:20px;background:#123456'></div></body></html>";
        let contents = "<html><body><section style='display:contents'><div style='width:40px;height:20px;background:#123456'></div></section></body></html>";
        let reference = RenderHtml(plain, 100, 80);
        assert_eq!(RenderHtml(hidden, 100, 80), reference);
        assert_eq!(RenderHtml(contents, 100, 80), reference);
        assert_eq!(RenderHtml(HTML, 400, 320), REFERENCE);
        assert_eq!(RenderHtml(VARIANT, 320, 240), VARIANT_REFERENCE);
        assert_eq!(
            RenderHtml(DEFAULT_MARGIN, 280, 180),
            DEFAULT_MARGIN_REFERENCE
        );
        assert_eq!(RenderHtml(SELECTOR, 240, 170), SELECTOR_REFERENCE);
        assert_eq!(RenderHtml(INLINE_STYLE, 260, 180), INLINE_STYLE_REFERENCE);
        assert_eq!(RenderHtml(SIDE_BORDERS, 180, 170), SIDE_BORDERS_REFERENCE);
        assert_eq!(RenderHtml(SIZE_LIMITS, 240, 230), SIZE_LIMITS_REFERENCE);
        assert_eq!(
            RenderHtml(SOURCE_BLOCK_PROFILE, 400, 320),
            SOURCE_BLOCK_PROFILE_REFERENCE
        );
        let css = HTML
            .split_once("<style>")
            .unwrap()
            .1
            .split_once("</style>")
            .unwrap()
            .0
            .to_owned();
        let linked_html = "<!doctype html><html><head><link rel=stylesheet href=../assets/site.css></head><body><div class=frame><div class=top></div><div class=bottom></div></div></body></html>";
        let linked_html_url = "<!doctype html><html><head><base href=/assets/><link rel=icon href=data:,><link rel=stylesheet href=site.css></head><body><div class=frame><div class=top></div><div class=bottom></div></div></body></html>";
        let directory =
            std::env::temp_dir().join(format!("layoutng-block-file-test-{}", std::process::id()));
        std::fs::create_dir_all(directory.join("nested")).unwrap();
        std::fs::create_dir_all(directory.join("assets")).unwrap();
        std::fs::write(directory.join("nested/page.html"), linked_html).unwrap();
        std::fs::write(directory.join("assets/site.css"), &css).unwrap();
        assert_eq!(
            RenderFile(&directory.join("nested/page.html"), 400, 320).unwrap(),
            REFERENCE
        );
        std::fs::remove_dir_all(&directory).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            for _ in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 2048];
                let size = stream.read(&mut request).unwrap();
                let request = std::str::from_utf8(&request[..size]).unwrap();
                let path = request.split_ascii_whitespace().nth(1).unwrap();
                let response = match path {
                    "/start" => "HTTP/1.1 302 Found\r\nLocation: /nested/page.html\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(),
                    "/nested/page.html" => format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{linked_html_url}", linked_html_url.len()),
                    "/assets/site.css" => format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{css}", css.len()),
                    _ => panic!("unexpected URL {path}"),
                };
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        let png = RenderUrl(&format!("http://127.0.0.1:{port}/start"), 400, 320).unwrap();
        server.join().unwrap();
        assert_eq!(png, REFERENCE);
        #[cfg(feature = "source_paint")]
        {
            let fragments = LayoutWithStyleLoader(
                HTML,
                400,
                320,
                None,
                |_, _| {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "this fixture has no linked stylesheet",
                    ))
                },
                |_, _| {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "this fixture has no image resources",
                    ))
                },
                |_, _| {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "this fixture has no font resources",
                    ))
                },
            )
            .unwrap();
            let display_list = paint::paint_engine::Paint(&fragments);
            assert!(!display_list.items.is_empty());
            assert!(!display_list.chunks.is_empty());
            assert_eq!(
                display_list.chunks.last().unwrap().end_index,
                u32::try_from(display_list.display_items.len()).unwrap()
            );
        }
    }
}

#[cfg(test)]
mod grid_layout_tests;

#[cfg(test)]
mod streaming_page_tests;

#[cfg(test)]
mod viewport_font_tests;

#[cfg(test)]
mod location_navigation_tests;

#[cfg(all(test, feature = "pure_source_png"))]
mod layer_tile_engine_tests;

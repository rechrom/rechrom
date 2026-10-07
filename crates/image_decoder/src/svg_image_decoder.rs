#![allow(non_snake_case)]

use std::ffi::c_char;
use std::io;

use dom::persistent_document::{DOMNamespace, DOMNode, PersistentDocument, DOM};
use dom::style_resolver::AddStyleSheet;
use html::html_parser::ParseHTMLBytes;
use layoutng_assembly::layout_assembly::LayoutAssembly;

use crate::image_decoder::ImageDecodeInput;

unsafe extern "C" {
    fn strtod(input: *const c_char, end: *mut *mut c_char) -> f64;
}

// cpp: image_decoder/svg_image_decoder.h:8-20
pub struct SVGImageDecoder {
    assembly_: LayoutAssembly,
}

impl SVGImageDecoder {
    // cpp: image_decoder/svg_image_decoder.h:12-13
    pub fn new(assembly: &LayoutAssembly) -> Self {
        Self {
            assembly_: *assembly,
        }
    }

    // cpp: image_decoder/svg_image_decoder.h:15-15
    // cpp: image_decoder/svg_image_decoder.cc:68-81
    pub fn CanDecode(&self, input: &ImageDecodeInput<'_>) -> bool {
        if SVGLowerASCII(input.mime_type).contains("image/svg+xml") {
            return true;
        }
        let mut offset = 0;
        while offset < input.bytes.len() && input.bytes[offset].is_ascii_whitespace() {
            offset += 1;
        }
        const SVG: &[u8] = b"<svg";
        input.bytes.len() - offset >= SVG.len()
            && SVG
                .iter()
                .zip(&input.bytes[offset..])
                .all(|(expected, actual)| *expected == actual.to_ascii_lowercase())
    }

    // cpp: image_decoder/svg_image_decoder.cc:83-110
    // The first half of Decode, shared with the source document pipeline below.
    fn PrepareDocument(&self, input: &ImageDecodeInput<'_>) -> io::Result<(DOM, i32, i32)> {
        if !self.CanDecode(input) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input is not an SVG image",
            ));
        }
        let mut document = ParseHTMLBytes(input.bytes);
        let svg =
            FindSVG(document.GetDocument(), document.GetDocument().Root()).ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "SVG image has no root svg element")
            })?;
        let svg = document.GetDocument().Node(svg);
        let mut width = NumberAttribute(svg, "width");
        let mut height = NumberAttribute(svg, "height");
        let view_box = ViewBoxSize(svg);
        if width.is_none() {
            width = view_box.map(|view_box| view_box.0);
        }
        if height.is_none() {
            height = view_box.map(|view_box| view_box.1);
        }
        let (width, height) = width.zip(height).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "SVG image has no intrinsic dimensions",
            )
        })?;
        const MAXIMUM_DIMENSION: f64 = 8192.0;
        if width > MAXIMUM_DIMENSION || height > MAXIMUM_DIMENSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "SVG image dimensions exceed the raster limit",
            ));
        }
        let pixel_width = (width.ceil() as i32).max(1);
        let pixel_height = (height.ceil() as i32).max(1);
        AddStyleSheet(
            &mut document,
            cssom::css_style_sheet::ParseCSS(concat!(
                "html,body{margin:0;padding:0;overflow:hidden;background:transparent;}",
                "html,body{width:100%;height:100%;}svg{display:block;}"
            )),
        );
        Ok((document, pixel_width, pixel_height))
    }
}

// cpp: image_decoder/svg_image_decoder.h:15-16
// cpp: image_decoder/svg_image_decoder.cc:83-128
#[cfg(feature = "source_document_images")]
impl crate::document_image_decoder::DocumentImageDecoder for SVGImageDecoder {
    fn CanDecode(&self, input: &ImageDecodeInput<'_>) -> bool {
        Self::CanDecode(self, input)
    }
    fn Decode(
        &mut self,
        input: &ImageDecodeInput<'_>,
        host_constraints: &layoutng_assembly::internal::layout_input::ConstraintSpace,
    ) -> io::Result<crate::image_decoder::DecodedImage> {
        use layoutng_assembly::{
            internal::{
                layout_input::{Size, ViewportGeometry},
                layout_input_types::IntSize,
            },
            layout_engine::LayoutEngine,
        };
        let (mut document, width, height) = self.PrepareDocument(input)?;
        dom::style_resolver::ResolveComputedStyles(
            &mut document,
            &dom::style_resolver::StyleEnvironment {
                viewport_width: Some(f64::from(width)),
                viewport_height: Some(f64::from(height)),
                resolution_dppx: Some(1.0),
                ..Default::default()
            },
            &[],
        );
        let mut constraints = host_constraints.clone();
        constraints.available_size = Size {
            width: f64::from(width),
            height: f64::from(height),
        };
        constraints.viewport = Some(ViewportGeometry {
            size: IntSize { width, height },
            ..Default::default()
        });
        let mut engine = LayoutEngine::new(&self.assembly_);
        document.EmitConstraints(&constraints, |mutation| {
            engine.ApplyMutation(mutation);
        });
        document.EmitLayoutMutations(&dom::UserInteractionState::default(), |mutation| {
            engine.ApplyMutation(mutation);
        });
        engine.Layout();
        let fragments = engine
            .TakeLayoutResult()
            .expect("successful layout publishes fragments");
        let items = paint::paint_engine::Paint(&fragments);
        let rgba8 =
            renderer::pure_replay::RasterizeDisplayItemList(&items, width as u32, height as u32);
        Ok(crate::image_decoder::DecodedImage {
            width: width as u32,
            height: height as u32,
            rgba8,
        })
    }
}

// cpp: image_decoder/svg_image_decoder.cc:23-29
fn SVGLowerASCII(input: &str) -> String {
    input.to_ascii_lowercase()
}

// cpp: image_decoder/svg_image_decoder.cc:31-36
fn FindSVG(document: &PersistentDocument, node: usize) -> Option<usize> {
    let node_ref = document.Node(node);
    if node_ref.IsElement("svg", DOMNamespace::kSVG) {
        return Some(node);
    }
    for &child in node_ref.Children() {
        if let Some(result) = FindSVG(document, child) {
            return Some(result);
        }
    }
    None
}

// cpp: image_decoder/svg_image_decoder.cc:38-51
fn NumberAttribute(node: &DOMNode, name: &str) -> Option<f64> {
    let attribute = node.FindAttribute(name)?;
    if attribute.value.is_empty() || attribute.value.contains('%') {
        return None;
    }
    let mut bytes = attribute.value.as_bytes().to_vec();
    bytes.push(0);
    let mut end: *mut c_char = std::ptr::null_mut();
    let start = bytes.as_ptr().cast::<c_char>();
    let value = unsafe { strtod(start, &mut end) };
    if end == start.cast_mut() || !value.is_finite() || value <= 0.0 {
        return None;
    }
    while unsafe { *end } != 0 && (unsafe { *end } as u8).is_ascii_whitespace() {
        end = unsafe { end.add(1) };
    }
    if unsafe { *end } != 0 {
        let suffix = unsafe { std::ffi::CStr::from_ptr(end) };
        if suffix.to_bytes() != b"px" {
            return None;
        }
    }
    Some(value)
}

// cpp: image_decoder/svg_image_decoder.cc:53-64
fn ViewBoxSize(node: &DOMNode) -> Option<(f64, f64)> {
    let attribute = node.FindAttribute("viewBox")?;
    let value = attribute.value.replace(',', " ");
    // istringstream's formatted extraction consumes a numeric prefix and
    // leaves the rest for the next extraction; split + parse would reject it.
    let mut bytes = value.into_bytes();
    bytes.push(0);
    let mut cursor = bytes.as_ptr().cast::<c_char>();
    let mut next_number = || {
        while unsafe { *cursor } != 0 && (unsafe { *cursor } as u8).is_ascii_whitespace() {
            cursor = unsafe { cursor.add(1) };
        }
        let mut end = std::ptr::null_mut();
        let number = unsafe { strtod(cursor, &mut end) };
        if end == cursor.cast_mut() {
            return None;
        }
        cursor = end;
        Some(number)
    };
    let _x = next_number()?;
    let _y = next_number()?;
    let width = next_number()?;
    let height = next_number()?;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some((width, height))
}

#![allow(non_snake_case)]

use std::collections::{BTreeMap, HashMap};
use std::ffi::c_char;
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use cssom::{CSSDeclaration, CSSKeyframesRule, CSSStyleSheet};
use dom::persistent_document::{
    DOMNamespace, DOMNode, PersistentDocument, SVGImagePreserveAspectRatio, SVGImageViewSpec, DOM,
};
use html::html_parser::ParseHTMLBytes;
use image_resource::{
    ContainerKey, CreatedDocumentImage, DocumentImage, DocumentImageEffect, DocumentImageFrame,
    DocumentImageMutation, ImageId, IntrinsicSize, MutationCause,
};
use layoutng_assembly::layout_assembly::LayoutAssembly;
use style::media_queries::media_query_backend::ParseMediaQuerySet;
use style::media_queries::media_query_evaluator::MediaQueryEvaluator;
use style::media_queries::{MediaValuesCached, MediaValuesCachedData};
use style::PreferredColorScheme;

unsafe extern "C" {
    fn strtod(input: *const c_char, end: *mut *mut c_char) -> f64;
}

fn PreferredScheme(value: u8) -> PreferredColorScheme {
    // ContainerKey uses the renderer-neutral image encoding: 0 is light and
    // 1 is dark. Blink's mojom enum orders dark before light.
    if value == 1 {
        PreferredColorScheme::kDark
    } else {
        PreferredColorScheme::kLight
    }
}

// cpp: image_decoder/svg_image_decoder.h:8-20
pub struct SVGImageDecoder {
    assembly_: LayoutAssembly,
    host_constraints_: layoutng_assembly::internal::layout_input::ConstraintSpace,
    // SVGImage's isolated document uses the same namespace-specific UA rules
    // as any other document (StyleResolver::ForEachUARulesForElement). Keep
    // the embedding's typed, parser-backed sheets rather than reconstructing
    // defaults in Layout or Paint.
    user_agent_sheets_: Vec<Rc<CSSStyleSheet>>,
}

#[derive(Clone)]
struct TimedKeyframe {
    offset: f64,
    declarations: Vec<CSSDeclaration>,
}

#[derive(Clone)]
struct CSSImageAnimation {
    node_id: u64,
    effect_id: u64,
    duration: f64,
    delay: f64,
    iterations: f64,
    easing: String,
    keyframes: Vec<TimedKeyframe>,
}

struct SVGDocumentAnimation {
    resource_id: ImageId,
    document: DOM,
    style_engine: style::StyleEngine,
    user_agent_sheets: Vec<Rc<CSSStyleSheet>>,
    engine: layoutng_assembly::layout_engine::LayoutEngine,
    paint_engine: paint::paint_engine::PaintEngine,
    constraints: layoutng_assembly::internal::layout_input::ConstraintSpace,
    intrinsic_sizing: SVGIntrinsicSizing,
    container: ContainerKey,
    containers: Vec<ContainerKey>,
    revision: u64,
    animations: Vec<CSSImageAnimation>,
    last_sample: Option<AnimationSampleBatch>,
}

// cpp: core/layout/natural_sizing_info.h NaturalSizingInfo and
// core/layout/svg/layout_svg_root.cc UnscaledNaturalSizingInfo().  An SVG can
// legitimately provide one dimension, only a viewBox ratio, or neither.  Keep
// those states distinct until ConcreteObjectSize is applied; collapsing them
// into a fixed bitmap size loses the SVG intrinsic-size contract at the image
// decoder boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SVGIntrinsicSizing {
    width: Option<f64>,
    height: Option<f64>,
    aspect_ratio: Option<(f64, f64)>,
    preserve_aspect_ratio_none: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SVGImageViewInfo {
    target: Option<usize>,
    view_spec: Option<SVGImageViewSpec>,
}

fn DecodeURLFragment(fragment: &str) -> String {
    let bytes = fragment.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b'%' && cursor + 2 < bytes.len() {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                b'A'..=b'F' => Some(byte - b'A' + 10),
                _ => None,
            };
            if let (Some(high), Some(low)) = (digit(bytes[cursor + 1]), digit(bytes[cursor + 2])) {
                decoded.push(high * 16 + low);
                cursor += 3;
                continue;
            }
        }
        decoded.push(bytes[cursor]);
        cursor += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn ParseViewBoxValue(value: &str) -> Option<[f64; 4]> {
    let values = ParseSVGNumbers(value)?;
    if values.len() != 4 || values[2] < 0.0 || values[3] < 0.0 {
        return None;
    }
    Some([values[0], values[1], values[2], values[3]])
}

fn ParsePreserveAspectRatio(value: &str) -> Option<SVGImagePreserveAspectRatio> {
    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
    let mut cursor = usize::from(parts.first() == Some(&"defer"));
    let align = *parts.get(cursor)?;
    cursor += 1;
    let mut result = SVGImagePreserveAspectRatio::default();
    if align == "none" {
        result.preserve_none = true;
    } else {
        let bytes = align.as_bytes();
        if bytes.len() != 8 || bytes[0] != b'x' || bytes[4] != b'Y' {
            return None;
        }
        result.align_x = match &bytes[1..4] {
            b"Min" => -1,
            b"Mid" => 0,
            b"Max" => 1,
            _ => return None,
        };
        result.align_y = match &bytes[5..8] {
            b"Min" => -1,
            b"Mid" => 0,
            b"Max" => 1,
            _ => return None,
        };
    }
    if let Some(meet_or_slice) = parts.get(cursor) {
        result.slice = match *meet_or_slice {
            "meet" => false,
            "slice" => true,
            _ => return None,
        };
        cursor += 1;
    }
    (cursor == parts.len()).then_some(result)
}

fn ParseSVGNumbers(value: &str) -> Option<Vec<f64>> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    let mut cursor = bytes.as_ptr().cast::<c_char>();
    let mut values = Vec::new();
    loop {
        while unsafe { *cursor } != 0
            && ((unsafe { *cursor } as u8).is_ascii_whitespace()
                || unsafe { *cursor } as u8 == b',')
        {
            cursor = unsafe { cursor.add(1) };
        }
        if unsafe { *cursor } == 0 {
            return Some(values);
        }
        let mut end = std::ptr::null_mut();
        let number = unsafe { strtod(cursor, &mut end) };
        if end == cursor.cast_mut() || !number.is_finite() {
            return None;
        }
        values.push(number);
        cursor = end;
    }
}

fn MultiplyAffine(left: [f64; 6], right: [f64; 6]) -> [f64; 6] {
    [
        left[0] * right[0] + left[2] * right[1],
        left[1] * right[0] + left[3] * right[1],
        left[0] * right[2] + left[2] * right[3],
        left[1] * right[2] + left[3] * right[3],
        left[0] * right[4] + left[2] * right[5] + left[4],
        left[1] * right[4] + left[3] * right[5] + left[5],
    ]
}

fn ParseViewTransform(value: &str) -> io::Result<Option<[f64; 6]>> {
    let mut cursor = 0;
    let mut transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let bytes = value.as_bytes();
    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b',')
        {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let name_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_alphabetic() {
            cursor += 1;
        }
        let name = &value[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if name.is_empty() || bytes.get(cursor) != Some(&b'(') {
            return Ok(None);
        }
        cursor += 1;
        let arguments_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b')' {
            cursor += 1;
        }
        if cursor == bytes.len() {
            return Ok(None);
        }
        let arguments = ParseSVGNumbers(&value[arguments_start..cursor]);
        cursor += 1;
        let Some(arguments) = arguments else {
            return Ok(None);
        };
        let operation = match name {
            "matrix" if arguments.len() == 6 => arguments.try_into().unwrap(),
            "translate" if (1..=2).contains(&arguments.len()) => [
                1.0,
                0.0,
                0.0,
                1.0,
                arguments[0],
                arguments.get(1).copied().unwrap_or(0.0),
            ],
            "scale" if (1..=2).contains(&arguments.len()) => [
                arguments[0],
                0.0,
                0.0,
                arguments.get(1).copied().unwrap_or(arguments[0]),
                0.0,
                0.0,
            ],
            "rotate" if arguments.len() == 1 || arguments.len() == 3 => {
                let angle = arguments[0].to_radians();
                let rotation = [
                    angle.cos(),
                    angle.sin(),
                    -angle.sin(),
                    angle.cos(),
                    0.0,
                    0.0,
                ];
                if arguments.len() == 1 {
                    rotation
                } else {
                    let to_origin = [1.0, 0.0, 0.0, 1.0, -arguments[1], -arguments[2]];
                    let from_origin = [1.0, 0.0, 0.0, 1.0, arguments[1], arguments[2]];
                    MultiplyAffine(from_origin, MultiplyAffine(rotation, to_origin))
                }
            }
            "skewX" if arguments.len() == 1 => {
                [1.0, 0.0, arguments[0].to_radians().tan(), 1.0, 0.0, 0.0]
            }
            "skewY" if arguments.len() == 1 => {
                [1.0, arguments[0].to_radians().tan(), 0.0, 1.0, 0.0, 0.0]
            }
            known
                if matches!(
                    known,
                    "matrix" | "translate" | "scale" | "rotate" | "skewX" | "skewY"
                ) =>
            {
                return Ok(None);
            }
            unsupported => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("unsupported svgView transform operation: {unsupported}"),
                ));
            }
        };
        transform = MultiplyAffine(transform, operation);
    }
    Ok(Some(transform))
}

fn ParseSVGViewSpecification(fragment: &str) -> io::Result<Option<SVGImageViewSpec>> {
    let Some(body) = fragment
        .strip_prefix("svgView(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let mut result = SVGImageViewSpec::default();
    let mut cursor = 0;
    while cursor < body.len() {
        while cursor < body.len()
            && (body.as_bytes()[cursor].is_ascii_whitespace() || body.as_bytes()[cursor] == b';')
        {
            cursor += 1;
        }
        if cursor == body.len() {
            break;
        }
        let name_start = cursor;
        while cursor < body.len() && body.as_bytes()[cursor].is_ascii_alphabetic() {
            cursor += 1;
        }
        let name = &body[name_start..cursor];
        if body.as_bytes().get(cursor) != Some(&b'(') {
            return Ok(None);
        }
        cursor += 1;
        let value_start = cursor;
        let mut depth = 1usize;
        while cursor < body.len() && depth != 0 {
            match body.as_bytes()[cursor] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            cursor += 1;
        }
        if depth != 0 {
            return Ok(None);
        }
        let value = &body[value_start..cursor - 1];
        match name {
            "viewBox" => {
                let Some(view_box) = ParseViewBoxValue(value) else {
                    return Ok(None);
                };
                result.view_box = Some(view_box);
            }
            "preserveAspectRatio" => {
                let Some(preserve) = ParsePreserveAspectRatio(value) else {
                    return Ok(None);
                };
                result.preserve_aspect_ratio = Some(preserve);
            }
            "transform" => {
                let Some(transform) = ParseViewTransform(value)? else {
                    return Ok(None);
                };
                result.transform = Some(transform);
            }
            // SVGViewSpec parses but deliberately ignores viewTarget.
            "viewTarget" => {}
            "zoomAndPan" if matches!(value.trim(), "magnify" | "disable") => {}
            "zoomAndPan" => return Ok(None),
            unsupported => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    format!("unsupported svgView function: {unsupported}"),
                ));
            }
        }
    }
    Ok(Some(result))
}

fn FindFragmentTarget(document: &PersistentDocument, id: &str) -> Option<usize> {
    (0..document.NodeCount()).find(|&node| {
        document.Node(node).Type() == dom::persistent_document::DOMNodeType::kElement
            && document
                .Node(node)
                .FindAttribute("id")
                .is_some_and(|attribute| attribute.value == id)
    })
}

fn CreateSVGImageViewInfo(
    document: &PersistentDocument,
    fragment: &str,
) -> io::Result<Option<SVGImageViewInfo>> {
    if fragment.is_empty() {
        return Ok(None);
    }
    let fragment = DecodeURLFragment(fragment);
    if fragment.starts_with("xywh=") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SVG spatial media fragments (xywh) are not translated",
        ));
    }
    let target = FindFragmentTarget(document, &fragment);
    let view_spec = if fragment.starts_with("svgView(") {
        ParseSVGViewSpecification(&fragment)?
    } else if target
        .is_some_and(|target| document.Node(target).IsElement("view", DOMNamespace::kSVG))
    {
        let view = document.Node(target.unwrap());
        let view_box = view
            .FindAttribute("viewBox")
            .and_then(|attribute| ParseViewBoxValue(&attribute.value));
        let preserve_aspect_ratio = view
            .FindAttribute("preserveAspectRatio")
            .and_then(|attribute| ParsePreserveAspectRatio(&attribute.value));
        Some(SVGImageViewSpec {
            view_box,
            preserve_aspect_ratio,
            transform: None,
        })
    } else {
        None
    };
    if view_spec.is_none() && target.is_none() {
        return Ok(None);
    }
    Ok(Some(SVGImageViewInfo { target, view_spec }))
}

fn ApplySVGImageViewInfo(document: &mut DOM, fragment: &str) -> io::Result<()> {
    let root = FindSVG(document.GetDocument(), document.GetDocument().Root())
        .ok_or_else(|| io::Error::other("SVG image has no root svg element"))?;
    let view_info = CreateSVGImageViewInfo(document.GetDocument(), fragment)?;
    document.GetDocumentMut().SetSVGImageViewInfo(
        root,
        view_info.and_then(|info| info.target),
        view_info.and_then(|info| info.view_spec),
    );
    Ok(())
}

fn EffectiveIntrinsicSize(
    document: &PersistentDocument,
    fragment: &str,
    mut sizing: SVGIntrinsicSizing,
) -> io::Result<IntrinsicSize> {
    // SVGImage::GetNaturalDimensions passes the active view specification's
    // viewBox to LayoutSVGRoot.  Explicit width+height remain the ratio owner;
    // otherwise the active viewBox supplies only an intrinsic ratio.
    if let Some([_, _, view_width, view_height]) = CreateSVGImageViewInfo(document, fragment)?
        .and_then(|info| info.view_spec)
        .and_then(|view| view.view_box)
    {
        if !(sizing.width.is_some() && sizing.height.is_some())
            && view_width > 0.0
            && view_height > 0.0
        {
            sizing.aspect_ratio = Some((view_width, view_height));
        }
    }
    // SVGImage::GetNaturalDimensions keeps Blink's stable compatibility
    // behavior for an authored preserveAspectRatio="none": partial intrinsic
    // dimensions are cleared after applying any override viewBox, so default
    // sizing supplies 300x150.
    if sizing.preserve_aspect_ratio_none && !(sizing.width.is_some() && sizing.height.is_some()) {
        sizing.width = None;
        sizing.height = None;
        sizing.aspect_ratio = None;
    }

    // cpp: core/layout/natural_sizing_info.cc ConcreteObjectSizeImpl(), with
    // the CSS Images default object size used by LayoutImage (300x150).
    let (width, height) = match (sizing.width, sizing.height, sizing.aspect_ratio) {
        (Some(width), Some(height), _) => (width, height),
        (Some(width), None, Some((ratio_width, ratio_height))) => {
            (width, width * ratio_height / ratio_width)
        }
        (Some(width), None, None) => (width, 150.0),
        (None, Some(height), Some((ratio_width, ratio_height))) => {
            (height * ratio_width / ratio_height, height)
        }
        (None, Some(height), None) => (300.0, height),
        (None, None, Some((ratio_width, ratio_height))) => {
            let width_at_default_height = 150.0 * ratio_width / ratio_height;
            if width_at_default_height <= 300.0 {
                (width_at_default_height, 150.0)
            } else {
                (300.0, 300.0 * ratio_height / ratio_width)
            }
        }
        (None, None, None) => (300.0, 150.0),
    };
    if !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
        || width > 8192.0
        || height > 8192.0
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "SVG image view intrinsic dimensions exceed the raster limit",
        ));
    }
    Ok(IntrinsicSize {
        width: width.ceil() as u32,
        height: height.ceil() as u32,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct AnimationSample {
    node_id: u64,
    effect_id: u64,
    declarations: Vec<CSSDeclaration>,
}

#[derive(Clone, Debug, PartialEq)]
struct AnimationSampleBatch {
    timeline_time: Duration,
    samples: Vec<AnimationSample>,
}

impl SVGImageDecoder {
    /// The embedding supplies complete host constraints and retained UA sheets,
    /// including SVG defaults. Container geometry is applied during creation.
    pub fn new_with_constraints(
        assembly: &LayoutAssembly,
        constraints: &layoutng_assembly::internal::layout_input::ConstraintSpace,
        user_agent_sheets: &[Rc<CSSStyleSheet>],
    ) -> Self {
        Self {
            assembly_: *assembly,
            host_constraints_: constraints.clone(),
            user_agent_sheets_: user_agent_sheets.to_vec(),
        }
    }

    // cpp: image_decoder/svg_image_decoder.h:15-15
    // cpp: image_decoder/svg_image_decoder.cc:68-81
    pub fn CanDecode(&self, bytes: &[u8], mime_type: &str) -> bool {
        if SVGLowerASCII(mime_type).contains("image/svg+xml") {
            return true;
        }
        let mut offset = 0;
        while offset < bytes.len() && bytes[offset].is_ascii_whitespace() {
            offset += 1;
        }
        const SVG: &[u8] = b"<svg";
        bytes.len() - offset >= SVG.len()
            && SVG
                .iter()
                .zip(&bytes[offset..])
                .all(|(expected, actual)| *expected == actual.to_ascii_lowercase())
    }

    // cpp: image_decoder/svg_image_decoder.cc:83-110
    // The first half of Decode, shared with the source document pipeline below.
    fn PrepareDocument(
        &self,
        bytes: &[u8],
        mime_type: &str,
    ) -> io::Result<(DOM, SVGIntrinsicSizing)> {
        if !self.CanDecode(bytes, mime_type) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "input is not an SVG image",
            ));
        }
        let mut document = ParseHTMLBytes(bytes);
        let svg =
            FindSVG(document.GetDocument(), document.GetDocument().Root()).ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "SVG image has no root svg element")
            })?;
        let svg = document.GetDocument().Node(svg);
        let width = NumberAttribute(svg, "width");
        let height = NumberAttribute(svg, "height");
        // LayoutSVGRoot derives a ratio from explicit dimensions only when
        // both are present.  With a missing dimension, viewBox is the ratio
        // owner but is not itself an intrinsic width or height.
        let aspect_ratio = width
            .zip(height)
            .or_else(|| ViewBoxSize(svg).filter(|(width, height)| *width > 0.0 && *height > 0.0));
        let intrinsic_sizing = SVGIntrinsicSizing {
            width,
            height,
            aspect_ratio,
            preserve_aspect_ratio_none: svg
                .FindAttribute("preserveAspectRatio")
                .and_then(|attribute| ParsePreserveAspectRatio(&attribute.value))
                .is_some_and(|preserve| preserve.preserve_none),
        };
        // The inert parser intentionally does not run Page's stylesheet
        // scheduler.  SVGImage owns an isolated document, so admit its inline
        // author sheets here before resolving the nested document.
        let style_nodes: Vec<_> = (0..document.GetDocument().NodeCount())
            .filter(|&index| {
                document
                    .GetDocument()
                    .Node(index)
                    .IsElement("style", DOMNamespace::kSVG)
                    || document.GetDocument().Node(index).IsHTMLElement("style")
            })
            .collect();
        for index in style_nodes {
            fn text(document: &PersistentDocument, index: usize, output: &mut String) {
                let node = document.Node(index);
                if node.Type() == dom::persistent_document::DOMNodeType::kText {
                    output.push_str(node.Data());
                }
                for &child in node.Children() {
                    text(document, child, output);
                }
            }
            let mut source = String::new();
            text(document.GetDocument(), index, &mut source);
            let mut sheet = style::ParseCSS(&source);
            sheet.owner_node_id = document.GetDocument().Node(index).Id();
            document.GetDocumentMut().AppendStyleSheet(sheet);
        }
        document
            .GetDocumentMut()
            .AppendStyleSheet(style::ParseCSS(concat!(
                "html,body{margin:0;padding:0;overflow:hidden;background:transparent;}",
                "html,body{width:100%;height:100%;}svg{display:block;}"
            )));
        Ok((document, intrinsic_sizing))
    }

    fn PrepareDocumentImage(
        &self,
        resource_id: ImageId,
        bytes: &[u8],
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<SVGDocumentAnimation> {
        use layoutng_assembly::internal::layout_input::{Size, ViewportGeometry};
        use layoutng_assembly::internal::layout_input_types::IntSize;

        let (mut document, intrinsic_sizing) = self.PrepareDocument(bytes, mime_type)?;
        let mut container = container.clone();
        ApplySVGImageViewInfo(&mut document, &container.view_specification)?;
        let intrinsic = EffectiveIntrinsicSize(
            document.GetDocument(),
            &container.view_specification,
            intrinsic_sizing,
        )?;
        if container.width == 0 {
            container.width = intrinsic.width;
        }
        if container.height == 0 {
            container.height = intrinsic.height;
        }
        let media = ImageMediaData(&container);
        document.GetDocumentMut().UpdateSVGUseInstances();
        let mut style_engine = style::StyleEngine::new(&document);
        style_engine
            .Update(&mut document, &media, &self.user_agent_sheets_)
            .map_err(|error| io::Error::other(error.to_string()))?;
        document.UpdateSVGResources();
        let environment = MediaValuesCached::new(&media);
        let animations = CollectAnimations(document.GetDocument(), &environment)?;
        let mut constraints = self.host_constraints_.clone();
        constraints.available_size = Size {
            width: f64::from(container.width),
            height: f64::from(container.height),
        };
        // LayoutNG records in layout-zoom coordinates. A document image's
        // container owns that coordinate space; inheriting the outer page's
        // scale while publishing an unscaled record size makes raster replay
        // apply the DPR a second time.
        let root = FindSVG(document.GetDocument(), document.GetDocument().Root())
            .expect("prepared SVG document has its root");
        constraints.document_image_container = Some(
            layoutng_assembly::internal::layout_input::DocumentImageContainer {
                root_node_id: document.GetDocument().Node(root).Id(),
                size: constraints.available_size,
            },
        );
        constraints.device_pixel_ratio = container.device_pixel_ratio();
        constraints.preferred_color_scheme = container.preferred_color_scheme;
        constraints.layout_zoom_factor = 1.0;
        constraints.viewport = Some(ViewportGeometry {
            size: IntSize {
                width: container.width as i32,
                height: container.height as i32,
            },
            ..Default::default()
        });
        Ok(SVGDocumentAnimation {
            resource_id,
            document,
            style_engine,
            user_agent_sheets: self.user_agent_sheets_.clone(),
            engine: layoutng_assembly::layout_engine::LayoutEngine::new(&self.assembly_),
            paint_engine: paint::paint_engine::PaintEngine::new(),
            constraints,
            intrinsic_sizing,
            containers: vec![container.clone()],
            container,
            revision: 0,
            animations,
            last_sample: None,
        })
    }
}

impl SVGDocumentAnimation {
    fn UpdateContainerEnvironment(&mut self) {
        use layoutng_assembly::internal::layout_input::{Size, ViewportGeometry};
        use layoutng_assembly::internal::layout_input_types::IntSize;
        self.constraints.available_size = Size {
            width: f64::from(self.container.width),
            height: f64::from(self.container.height),
        };
        self.constraints
            .document_image_container
            .as_mut()
            .expect("SVG image retains its root container input")
            .size = self.constraints.available_size;
        self.constraints.device_pixel_ratio = self.container.device_pixel_ratio();
        self.constraints.preferred_color_scheme = self.container.preferred_color_scheme;
        // The nested PaintArtifact is replayed through the outer image draw,
        // whose transform already maps record_size into the destination. Keep
        // its layout coordinate space independent from the host page's DSF.
        self.constraints.layout_zoom_factor = 1.0;
        self.constraints.viewport = Some(ViewportGeometry {
            size: IntSize {
                width: self.container.width as i32,
                height: self.container.height as i32,
            },
            ..Default::default()
        });
    }

    fn Sample(&self, elapsed: Duration) -> AnimationSampleBatch {
        let samples = self
            .animations
            .iter()
            .map(|animation| AnimationSample {
                node_id: animation.node_id,
                effect_id: animation.effect_id,
                declarations: SampleAnimation(animation, elapsed.as_secs_f64()),
            })
            .collect();
        AnimationSampleBatch {
            timeline_time: elapsed,
            samples,
        }
    }

    fn RenderSample(
        &mut self,
        sample: AnimationSampleBatch,
    ) -> io::Result<Arc<DocumentImageFrame>> {
        // SVGImage::Draw always applies (or clears) view info before lifecycle
        // update. This is required when consecutive container variants select
        // different fragments on the same retained nested document.
        ApplySVGImageViewInfo(&mut self.document, &self.container.view_specification)?;
        for animation in &sample.samples {
            self.document.GetDocumentMut().SetAnimationStyle(
                animation.node_id,
                animation.effect_id,
                animation.declarations.clone(),
            );
        }
        self.document.GetDocumentMut().UpdateSVGUseInstances();
        self.style_engine
            .Update(
                &mut self.document,
                &ImageMediaData(&self.container),
                &self.user_agent_sheets,
            )
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.document.UpdateSVGResources();
        self.document
            .EmitConstraints(&self.constraints, |mutation| {
                self.engine.ApplyMutation(mutation);
            });
        self.document
            .EmitLayoutMutations(&dom::UserInteractionState::default(), |mutation| {
                self.engine.ApplyMutation(mutation)
            });
        self.engine.Layout();
        let fragments = self
            .engine
            .TakeLayoutResult()
            .ok_or_else(|| io::Error::other("animated SVG layout produced no fragments"))?;
        self.paint_engine.Paint(&fragments, Default::default());
        let artifact = self
            .paint_engine
            .GetPaintResult()
            .ok_or_else(|| io::Error::other("animated SVG paint produced no artifact"))?
            .clone();
        let intrinsic_size = EffectiveIntrinsicSize(
            self.document.GetDocument(),
            &self.container.view_specification,
            self.intrinsic_sizing,
        )?;
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| io::Error::other("document image revision exhausted"))?;
        self.last_sample = Some(sample);
        let record = Arc::new(paint::paint_engine::DocumentPaintArtifactRecord {
            artifact,
            intrinsic_size,
            record_size: IntrinsicSize {
                width: self.container.width,
                height: self.container.height,
            },
            container_key: self.container.clone(),
        });
        Ok(Arc::new(DocumentImageFrame {
            resource_id: self.resource_id,
            revision: self.revision,
            intrinsic_size,
            container_key: self.container.clone(),
            record,
        }))
    }

    fn RenderAt(&mut self, elapsed: Duration) -> io::Result<Arc<DocumentImageFrame>> {
        let sample = self.Sample(elapsed);
        self.RenderSample(sample)
    }
}

impl DocumentImage for SVGDocumentAnimation {
    fn apply_mutation(
        &mut self,
        mutation: DocumentImageMutation,
    ) -> io::Result<Vec<DocumentImageEffect>> {
        // The isolated SVG document owns a complete Style/Layout/Paint
        // lifecycle. Its typed mutation boundary is therefore also its cppgc
        // allocation boundary; the application thread that hosts this object
        // does not need to know which mutation will allocate native style.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        match mutation {
            DocumentImageMutation::AdvanceTimeline {
                frame_time,
                begin_frame_sequence,
            } => {
                let sample = self.Sample(frame_time);
                if self
                    .last_sample
                    .as_ref()
                    .is_some_and(|previous| previous.samples == sample.samples)
                {
                    return Ok(self
                        .has_active_animation()
                        .then_some(DocumentImageEffect::RequestBeginFrame)
                        .into_iter()
                        .collect());
                }
                let containers = self.containers.clone();
                let mut effects = Vec::with_capacity(containers.len() + 1);
                for container in containers {
                    self.container = container;
                    self.UpdateContainerEnvironment();
                    let frame = self.RenderSample(sample.clone())?;
                    effects.push(DocumentImageEffect::FrameChanged {
                        frame,
                        cause: MutationCause {
                            begin_frame_sequence,
                        },
                    });
                }
                if self.has_active_animation() {
                    effects.push(DocumentImageEffect::RequestBeginFrame);
                }
                Ok(effects)
            }
            DocumentImageMutation::SetContainer { container } => {
                if self.container == container {
                    return Ok(Vec::new());
                }
                self.container = container;
                if self.container.width == 0 {
                    self.container.width = EffectiveIntrinsicSize(
                        self.document.GetDocument(),
                        &self.container.view_specification,
                        self.intrinsic_sizing,
                    )?
                    .width;
                }
                if self.container.height == 0 {
                    self.container.height = EffectiveIntrinsicSize(
                        self.document.GetDocument(),
                        &self.container.view_specification,
                        self.intrinsic_sizing,
                    )?
                    .height;
                }
                if self.containers.contains(&self.container) {
                    self.UpdateContainerEnvironment();
                    return Ok(Vec::new());
                }
                self.containers.push(self.container.clone());
                self.UpdateContainerEnvironment();
                let frame = self.RenderAt(
                    self.last_sample
                        .as_ref()
                        .map_or(Duration::ZERO, |s| s.timeline_time),
                )?;
                Ok(vec![DocumentImageEffect::FrameChanged {
                    frame,
                    cause: MutationCause {
                        begin_frame_sequence: 0,
                    },
                }])
            }
            DocumentImageMutation::SetPreferredColorScheme { scheme } => {
                if self.container.preferred_color_scheme == scheme {
                    return Ok(Vec::new());
                }
                self.container.preferred_color_scheme = scheme;
                if !self.containers.contains(&self.container) {
                    self.containers.push(self.container.clone());
                }
                self.UpdateContainerEnvironment();
                let frame = self.RenderAt(
                    self.last_sample
                        .as_ref()
                        .map_or(Duration::ZERO, |s| s.timeline_time),
                )?;
                Ok(vec![DocumentImageEffect::FrameChanged {
                    frame,
                    cause: MutationCause {
                        begin_frame_sequence: 0,
                    },
                }])
            }
        }
    }

    fn has_active_animation(&self) -> bool {
        !self.animations.is_empty()
    }
}

impl image_resource::DocumentImageDecoder for SVGImageDecoder {
    fn can_decode(&self, bytes: &[u8], mime_type: &str) -> bool {
        self.CanDecode(bytes, mime_type)
    }

    fn create(
        &mut self,
        resource_id: ImageId,
        bytes: Arc<[u8]>,
        mime_type: &str,
        container: &ContainerKey,
    ) -> io::Result<CreatedDocumentImage> {
        // Creation performs the same isolated Style/Layout/Paint lifecycle as
        // later animation mutations and may allocate native ComputedStyle.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        let mut image = self.PrepareDocumentImage(resource_id, &bytes, mime_type, container)?;
        let initial_frame = image.RenderAt(Duration::ZERO)?;
        let effects = image
            .has_active_animation()
            .then_some(DocumentImageEffect::RequestBeginFrame)
            .into_iter()
            .collect();
        Ok(CreatedDocumentImage {
            initial_frame,
            image: Box::new(image),
            effects,
        })
    }
}

#[derive(Default)]
struct AnimationShorthand {
    name: String,
    duration: f64,
    delay: f64,
    iterations: f64,
    easing: String,
}

fn Seconds(value: &str) -> Option<f64> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.trim().parse::<f64>().ok().map(|value| value / 1000.0);
    }
    value
        .strip_suffix('s')
        .and_then(|seconds| seconds.trim().parse::<f64>().ok())
}

fn ParseAnimationShorthand(value: &str) -> Option<AnimationShorthand> {
    let value = value.split(',').next()?.trim();
    if value.eq_ignore_ascii_case("none") {
        return None;
    }
    let mut result = AnimationShorthand {
        duration: 0.0,
        delay: 0.0,
        iterations: 1.0,
        easing: "ease".into(),
        ..Default::default()
    };
    let mut time_count = 0;
    for token in value.split_ascii_whitespace() {
        if let Some(seconds) = Seconds(token) {
            if time_count == 0 {
                result.duration = seconds.max(0.0);
            } else if time_count == 1 {
                result.delay = seconds;
            }
            time_count += 1;
        } else if token.eq_ignore_ascii_case("infinite") {
            result.iterations = f64::INFINITY;
        } else if let Ok(iterations) = token.parse::<f64>() {
            result.iterations = iterations.max(0.0);
        } else if matches!(
            token.to_ascii_lowercase().as_str(),
            "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out" | "step-start" | "step-end"
        ) || token.to_ascii_lowercase().starts_with("cubic-bezier(")
            || token.to_ascii_lowercase().starts_with("steps(")
        {
            result.easing = token.to_ascii_lowercase();
        } else if !matches!(
            token.to_ascii_lowercase().as_str(),
            "normal"
                | "reverse"
                | "alternate"
                | "alternate-reverse"
                | "none"
                | "forwards"
                | "backwards"
                | "both"
                | "running"
                | "paused"
        ) {
            result.name = token.to_owned();
        }
    }
    (!result.name.is_empty() && result.duration > 0.0 && result.iterations > 0.0).then_some(result)
}

fn KeyframeOffset(value: &str) -> Option<f64> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "from" => Some(0.0),
        "to" => Some(1.0),
        _ => value
            .strip_suffix('%')
            .and_then(|percent| percent.trim().parse::<f64>().ok())
            .map(|percent| (percent / 100.0).clamp(0.0, 1.0)),
    }
}

fn ExpandKeyframes(rule: &CSSKeyframesRule) -> Vec<TimedKeyframe> {
    let mut result = Vec::new();
    for keyframe in &rule.keyframes {
        for key in keyframe.key_text.split(',') {
            if let Some(offset) = KeyframeOffset(key) {
                result.push(TimedKeyframe {
                    offset,
                    declarations: keyframe.declarations.clone(),
                });
            }
        }
    }
    result.sort_by(|a, b| a.offset.total_cmp(&b.offset));
    result
}

// Chromium SVGImageForContainer supplies the container viewport, DPR and color scheme.
fn ImageMediaData(container: &ContainerKey) -> MediaValuesCachedData {
    let width = container.width as f64;
    let height = container.height as f64;
    MediaValuesCachedData {
        viewport_width: width,
        viewport_height: height,
        small_viewport_width: width,
        small_viewport_height: height,
        large_viewport_width: width,
        large_viewport_height: height,
        dynamic_viewport_width: width,
        dynamic_viewport_height: height,
        device_width: container.width as i32,
        device_height: container.height as i32,
        // SVG image documents have the same LocalFrame DPR contract as the
        // outer document: host scale multiplied by their container zoom.
        device_pixel_ratio: (container.device_pixel_ratio()
            * container.zoom().max(f64::MIN_POSITIVE)) as f32,
        layout_zoom_factor: container.zoom().max(f64::MIN_POSITIVE) as f32,
        media_type: foundation::String::from("screen"),
        preferred_color_scheme: PreferredScheme(container.preferred_color_scheme),
        scripting: style::media_queries::scripting::Scripting::kNone,
        ..Default::default()
    }
}

fn CollectAnimations(
    document: &PersistentDocument,
    environment: &MediaValuesCached,
) -> io::Result<Vec<CSSImageAnimation>> {
    let mut definitions = HashMap::new();
    let mut keyframe_rule_count = 0;
    for sheet in document.ActiveStyleSheets() {
        for rule in &sheet.keyframes {
            keyframe_rule_count += 1;
            if rule.media_conditions.iter().all(|condition| {
                MediaQueryEvaluator::ForMediaValues(environment)
                    .Eval(&ParseMediaQuerySet(condition))
            }) {
                definitions.insert(rule.name.clone(), ExpandKeyframes(rule));
            }
        }
    }
    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() && keyframe_rule_count != 0 {
        eprintln!(
            "svg-animation-profile keyframes={} definitions={}",
            keyframe_rule_count,
            definitions.len()
        );
    }
    if definitions.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    let mut selector_service = style::persistent_selector::PersistentSelectorService::default();
    let mut effect_id = 1;
    for index in 0..document.NodeCount() {
        if document.Node(index).Type() != dom::persistent_document::DOMNodeType::kElement {
            continue;
        }
        let mut shorthand = None;
        for sheet in document.ActiveStyleSheets() {
            for rule in &sheet.rules {
                if !rule.media_conditions.iter().all(|condition| {
                    MediaQueryEvaluator::ForMediaValues(environment)
                        .Eval(&ParseMediaQuerySet(condition))
                }) || !selector_service
                    .Matches(document, index, &rule.selector_text)
                    .map_err(|error| {
                        io::Error::new(
                            io::ErrorKind::Unsupported,
                            format!("SVG animation selector: {error:?}"),
                        )
                    })?
                {
                    continue;
                }
                for declaration in &rule.declarations {
                    if declaration.property.eq_ignore_ascii_case("animation")
                        || declaration
                            .property
                            .eq_ignore_ascii_case("-webkit-animation")
                    {
                        shorthand = ParseAnimationShorthand(&declaration.value);
                    }
                }
            }
        }
        let Some(animation) = shorthand else { continue };
        let Some(keyframes) = definitions
            .get(&animation.name)
            .filter(|frames| !frames.is_empty())
        else {
            continue;
        };
        result.push(CSSImageAnimation {
            node_id: document.Node(index).Id(),
            effect_id,
            duration: animation.duration,
            delay: animation.delay,
            iterations: animation.iterations,
            easing: animation.easing,
            keyframes: keyframes.clone(),
        });
        effect_id += 1;
    }
    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() && keyframe_rule_count != 0 {
        eprintln!("svg-animation-profile targets={}", result.len());
    }
    Ok(result)
}

fn Cubic(t: f64, a: f64, b: f64) -> f64 {
    3.0 * (1.0 - t) * (1.0 - t) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t
}

fn Ease(value: f64, easing: &str) -> f64 {
    let points = match easing {
        "ease" => Some((0.25, 0.1, 0.25, 1.0)),
        "ease-in" => Some((0.42, 0.0, 1.0, 1.0)),
        "ease-out" => Some((0.0, 0.0, 0.58, 1.0)),
        "ease-in-out" => Some((0.42, 0.0, 0.58, 1.0)),
        "step-start" => return 1.0,
        "step-end" => return if value < 1.0 { 0.0 } else { 1.0 },
        _ => None,
    };
    let Some((x1, y1, x2, y2)) = points else {
        return value;
    };
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..24 {
        let middle = (low + high) * 0.5;
        if Cubic(middle, x1, x2) < value {
            low = middle;
        } else {
            high = middle;
        }
    }
    Cubic((low + high) * 0.5, y1, y2)
}

fn NumericParts(value: &str) -> (Vec<f64>, Vec<String>) {
    let bytes = value.as_bytes();
    let mut numbers = Vec::new();
    let mut parts = Vec::new();
    let mut cursor = 0;
    let mut text = 0;
    while cursor < bytes.len() {
        let start = cursor;
        if matches!(bytes[cursor], b'+' | b'-') {
            cursor += 1;
        }
        let digits = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b'.' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
        }
        if cursor > digits && value[start..cursor].parse::<f64>().is_ok() {
            parts.push(value[text..start].to_owned());
            numbers.push(value[start..cursor].parse().unwrap());
            text = cursor;
        } else {
            cursor = start + 1;
        }
    }
    parts.push(value[text..].to_owned());
    (numbers, parts)
}

fn CanonicalPart(value: &str) -> String {
    value.replace("px", "").replace("deg", "")
}

fn Interpolate(a: &str, b: &str, progress: f64) -> String {
    if a == b {
        return a.to_owned();
    }
    let (an, ap) = NumericParts(a);
    let (bn, bp) = NumericParts(b);
    if an.len() != bn.len()
        || an.is_empty()
        || ap.len() != bp.len()
        || !ap
            .iter()
            .zip(&bp)
            .all(|(a, b)| CanonicalPart(a) == CanonicalPart(b))
    {
        return if progress < 0.5 { a } else { b }.to_owned();
    }
    let mut result = String::new();
    for index in 0..an.len() {
        result.push_str(&bp[index]);
        result.push_str(&format!(
            "{:.6}",
            an[index] + (bn[index] - an[index]) * progress
        ));
    }
    result.push_str(bp.last().unwrap());
    result
}

fn SampleAnimation(animation: &CSSImageAnimation, elapsed: f64) -> Vec<CSSDeclaration> {
    let active = elapsed - animation.delay;
    if active < 0.0 {
        return Vec::new();
    }
    if animation.iterations.is_finite() && active >= animation.duration * animation.iterations {
        return Vec::new();
    }
    let progress = Ease(
        (active % animation.duration) / animation.duration,
        &animation.easing,
    );
    let mut properties: BTreeMap<String, Vec<(f64, String, bool)>> = BTreeMap::new();
    for frame in &animation.keyframes {
        for declaration in &frame.declarations {
            properties
                .entry(declaration.property.clone())
                .or_default()
                .push((
                    frame.offset,
                    declaration.value.clone(),
                    declaration.important,
                ));
        }
    }
    properties
        .into_iter()
        .filter_map(|(property, mut values)| {
            values.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut left = &values[0];
            let mut right = values.last().unwrap();
            for pair in values.windows(2) {
                if progress >= pair[0].0 && progress <= pair[1].0 {
                    left = &pair[0];
                    right = &pair[1];
                    break;
                }
            }
            let local = if right.0 == left.0 {
                1.0
            } else {
                ((progress - left.0) / (right.0 - left.0)).clamp(0.0, 1.0)
            };
            Some(CSSDeclaration {
                property,
                value: Interpolate(&left.1, &right.1, local),
                important: left.2 || right.2,
            })
        })
        .collect()
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

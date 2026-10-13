//! Document-owned resolution of local SVG paint-server IRIs.
//!
//! Chromium keeps the IRI on `SVGPaint` and resolves it through
//! `LayoutSVGResourcePaintServer`; gradient elements then collect the first
//! specified value for each attribute along their href chain.  Rechrom freezes
//! the resulting backend-neutral shader at the existing Style -> Layout paint
//! boundary, while the canonical native `ComputedStyle` remains the URI owner.

use dom::{
    persistent_document::{DOMNamespace, DOMNodeType},
    Document,
};
use layoutng::internal::{
    layout_input::{
        Offset, PaintPathVerb, Size, SvgShapeData, SvgViewBoxData, TextDirection, TransformMatrix,
    },
    paint_input::{
        PaintColorStop, PaintFilterType, PaintMaskMode, PaintRadialShape, PaintShader,
        PaintShaderKind, PaintSpreadMethod, PaintStyleData, SvgClipLength, SvgClipText,
        SvgClipTextAnchor, SvgClipTextPath, SvgMarkerInstance, SvgMarkerPrimitive, SvgMaskGraph,
        SvgMaskUnits, SvgPaintServerReferenceBox, SvgPatternCommand, SvgPatternData,
        SvgPatternImage, SvgPatternReferenceFilter, SvgPatternText,
    },
};
use layoutng_style::style::svg_paint::SVGPaintType;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dom::persistent_document::ResolvedNodeStyle;

#[derive(Clone, Copy)]
enum LengthAxis {
    X,
    Y,
    Width,
    Height,
    Other,
}

#[derive(Default)]
struct CollectedGradient {
    spread: Option<PaintSpreadMethod>,
    object_bounding_box: Option<bool>,
    transform: Option<TransformMatrix>,
    stops: Option<Vec<PaintColorStop>>,
    x1: Option<String>,
    y1: Option<String>,
    x2: Option<String>,
    y2: Option<String>,
    cx: Option<String>,
    cy: Option<String>,
    radius: Option<String>,
    fx: Option<String>,
    fy: Option<String>,
    focal_radius: Option<String>,
}

fn Style<'a>(
    document: &'a Document,
    pending: &'a HashMap<usize, ResolvedNodeStyle>,
    node: usize,
) -> Option<&'a ResolvedNodeStyle> {
    pending
        .get(&node)
        .or_else(|| document.ResolvedStyleFor(node))
}

fn LocalFragment(reference: &str) -> Option<&str> {
    reference.strip_prefix('#').filter(|id| !id.is_empty())
}

fn FindById(document: &Document, origin: usize, id: &str) -> Option<usize> {
    document.FindSVGResourceById(origin, id)
}

fn ReferencedGradient(document: &Document, node: usize) -> Option<usize> {
    let href = Reference(document, node)?;
    FindById(document, node, LocalFragment(href)?)
}

fn Reference(document: &Document, node: usize) -> Option<&str> {
    let element = document.Node(node);
    element
        .FindAttribute("href")
        .or_else(|| element.FindAttribute("xlink:href"))
        .map(|attribute| attribute.value.trim())
}

fn Attribute(document: &Document, node: usize, name: &str) -> Option<String> {
    document
        .Node(node)
        .FindAttribute(name)
        .map(|attribute| attribute.value.clone())
}

fn SvgTextLength(token: &str) -> Option<SvgClipLength> {
    let token = token.trim();
    if let Some(value) = token.strip_suffix('%') {
        return value
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .map(|v| SvgClipLength {
                value: 0.0,
                percentage: v / 100.0,
            });
    }
    token
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|value| SvgClipLength {
            value,
            percentage: 0.0,
        })
}

fn SvgTextLengthList(document: &Document, node: usize, name: &str) -> Option<Vec<SvgClipLength>> {
    Attribute(document, node, name)
        .unwrap_or_default()
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|token| !token.is_empty())
        .map(SvgTextLength)
        .collect()
}

fn SvgTextNumberList(document: &Document, node: usize, name: &str) -> Option<Vec<f64>> {
    Attribute(document, node, name)
        .unwrap_or_default()
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|token| !token.is_empty())
        .map(|token| token.parse::<f64>().ok().filter(|v| v.is_finite()))
        .collect()
}

fn DirectSvgText(document: &Document, node: usize) -> String {
    let preserve = Attribute(document, node, "xml:space").as_deref() == Some("preserve");
    let text = document
        .Node(node)
        .Children()
        .iter()
        .filter_map(|&child| {
            (document.Node(child).Type() == DOMNodeType::kText).then(|| document.Node(child).Data())
        })
        .collect::<String>();
    if preserve {
        text
    } else {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

fn PatternTextPath(
    document: &Document,
    node: usize,
) -> Result<Option<SvgClipTextPath>, PatternContentError> {
    if document.Node(node).Name() != "textPath" {
        return Ok(None);
    }
    let reference = Reference(document, node).ok_or(PatternContentError::TextRequiresShaping)?;
    let id = LocalFragment(reference).ok_or(PatternContentError::TextRequiresShaping)?;
    let target = FindById(document, node, id).ok_or(PatternContentError::TextRequiresShaping)?;
    if document.Node(target).Name() != "path" {
        return Err(PatternContentError::TextRequiresShaping);
    }
    let parsed = dom::svg_path_parser::ParseSVGPathDefault(
        Attribute(document, target, "d").as_deref().unwrap_or(""),
    )
    .ok_or(PatternContentError::TextRequiresShaping)?;
    if parsed.flattened_points.len() < 2 {
        return Err(PatternContentError::TextRequiresShaping);
    }
    let matrix = Attribute(document, target, "transform")
        .as_deref()
        .and_then(ParseGradientTransform)
        .unwrap_or_default();
    let points = parsed
        .flattened_points
        .into_iter()
        .map(|point| Offset {
            x: matrix.values[0] * point.x + matrix.values[4] * point.y + matrix.values[12],
            y: matrix.values[1] * point.x + matrix.values[5] * point.y + matrix.values[13],
        })
        .collect();
    let start_offset = Attribute(document, node, "startOffset")
        .as_deref()
        .and_then(SvgTextLength)
        .unwrap_or_default();
    Ok(Some(SvgClipTextPath {
        points,
        start_offset,
        reverse_direction: Attribute(document, node, "side").as_deref() == Some("right"),
    }))
}

fn ParseGradientTransform(value: &str) -> Option<TransformMatrix> {
    type Affine = [f64; 6];
    let multiply = |left: Affine, right: Affine| {
        [
            left[0] * right[0] + left[2] * right[1],
            left[1] * right[0] + left[3] * right[1],
            left[0] * right[2] + left[2] * right[3],
            left[1] * right[2] + left[3] * right[3],
            left[0] * right[4] + left[2] * right[5] + left[4],
            left[1] * right[4] + left[3] * right[5] + left[5],
        ]
    };
    let operation = |name: &str, arguments: &[f64]| -> Option<Affine> {
        match name.to_ascii_lowercase().as_str() {
            "matrix" if arguments.len() == 6 => arguments.try_into().ok(),
            "translate" if (1..=2).contains(&arguments.len()) => Some([
                1.0,
                0.0,
                0.0,
                1.0,
                arguments[0],
                arguments.get(1).copied().unwrap_or(0.0),
            ]),
            "scale" if (1..=2).contains(&arguments.len()) => Some([
                arguments[0],
                0.0,
                0.0,
                arguments.get(1).copied().unwrap_or(arguments[0]),
                0.0,
                0.0,
            ]),
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
                    Some(rotation)
                } else {
                    let center = [1.0, 0.0, 0.0, 1.0, arguments[1], arguments[2]];
                    let inverse_center = [1.0, 0.0, 0.0, 1.0, -arguments[1], -arguments[2]];
                    Some(multiply(multiply(center, rotation), inverse_center))
                }
            }
            "skewx" if arguments.len() == 1 => {
                Some([1.0, 0.0, arguments[0].to_radians().tan(), 1.0, 0.0, 0.0])
            }
            "skewy" if arguments.len() == 1 => {
                Some([1.0, arguments[0].to_radians().tan(), 0.0, 1.0, 0.0, 0.0])
            }
            _ => None,
        }
    };
    let mut result = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut rest = value.trim();
    let mut count = 0;
    while !rest.is_empty() {
        let open = rest.find('(')?;
        let close = rest[open + 1..].find(')')? + open + 1;
        let name = rest[..open].trim();
        let arguments = rest[open + 1..close]
            .split(|character: char| character == ',' || character.is_ascii_whitespace())
            .filter(|argument| !argument.is_empty())
            .map(str::parse::<f64>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        result = multiply(result, operation(name, &arguments)?);
        count += 1;
        rest = rest[close + 1..].trim_start_matches(|character: char| {
            character == ',' || character.is_ascii_whitespace()
        });
    }
    if count == 0 {
        return None;
    }
    let mut matrix = TransformMatrix::default();
    matrix.values[0] = result[0];
    matrix.values[1] = result[1];
    matrix.values[4] = result[2];
    matrix.values[5] = result[3];
    matrix.values[12] = result[4];
    matrix.values[13] = result[5];
    Some(matrix)
}

fn Stops(
    document: &Document,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    gradient: usize,
    paint_opacity: f32,
) -> Option<Vec<PaintColorStop>> {
    let mut result = Vec::new();
    let mut previous = 0.0_f64;
    for &child in document.Node(gradient).Children() {
        if !document.Node(child).IsElement("stop", DOMNamespace::kSVG) {
            continue;
        }
        let offset = Attribute(document, child, "offset")
            .as_deref()
            .and_then(ParseOffset)
            .unwrap_or(0.0)
            .clamp(previous, 1.0);
        previous = offset;
        let style = Style(document, pending, child)?;
        let native = unsafe { style.native_style.Get().as_ref() }?;
        let resolved = native.StopColor().Resolve(
            native.GetCurrentColor(None),
            native.UsedColorScheme(),
            None,
        );
        let (red, green, blue) = resolved.ToSRGB();
        let color = layoutng::internal::layout_input_types::Color {
            red,
            green,
            blue,
            alpha: resolved.Alpha() * native.StopOpacity() * paint_opacity,
        };
        result.push(PaintColorStop {
            offset,
            color,
            source_color_space:
                layoutng::internal::paint_input::PaintColorInterpolationSpace::kSRGB,
            source_components: [red, green, blue, color.alpha],
            missing_components: 0,
            offset_length: 0.0,
            is_hint: false,
        });
    }
    (!result.is_empty()).then_some(result)
}

fn ParseOffset(value: &str) -> Option<f64> {
    let value = value.trim();
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| value / 100.0);
    }
    value.parse::<f64>().ok()
}

fn ViewBox(document: &Document, mut node: usize) -> Option<(f64, f64, f64, f64)> {
    loop {
        let current = document.Node(node);
        if current.IsElement("svg", DOMNamespace::kSVG) {
            if let Some(value) = current.FindAttribute("viewBox") {
                let values = value
                    .value
                    .split(|character: char| character.is_ascii_whitespace() || character == ',')
                    .filter(|part| !part.is_empty())
                    .map(str::parse::<f64>)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;
                if values.len() == 4 && values[2] > 0.0 && values[3] > 0.0 {
                    return Some((values[0], values[1], values[2], values[3]));
                }
            }
            let width = Attribute(document, node, "width")?.parse::<f64>().ok()?;
            let height = Attribute(document, node, "height")?.parse::<f64>().ok()?;
            if width > 0.0 && height > 0.0 {
                return Some((0.0, 0.0, width, height));
            }
        }
        node = current.Parent()?;
    }
}

fn ParseLength(
    value: &str,
    axis: LengthAxis,
    object_bounding_box: bool,
    viewport: Option<(f64, f64, f64, f64)>,
) -> Option<f64> {
    let value = value.trim();
    if let Some(percent) = value.strip_suffix('%') {
        let ratio = percent.trim().parse::<f64>().ok()? / 100.0;
        if object_bounding_box {
            return Some(ratio);
        }
        let (x, y, width, height) = viewport?;
        return Some(match axis {
            LengthAxis::X => x + ratio * width,
            LengthAxis::Y => y + ratio * height,
            LengthAxis::Width => ratio * width,
            LengthAxis::Height => ratio * height,
            LengthAxis::Other => ratio * width.hypot(height) / 2.0_f64.sqrt(),
        });
    }
    let (number, scale) = [
        ("px", 1.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("pt", 96.0 / 72.0),
        ("pc", 16.0),
    ]
    .into_iter()
    .find_map(|(suffix, scale)| value.strip_suffix(suffix).map(|number| (number, scale)))
    .unwrap_or((value, 1.0));
    let number = number.trim().parse::<f64>().ok()? * scale;
    (!object_bounding_box || scale == 1.0 && !value.ends_with("px")).then_some(number)
}

fn Collect(
    document: &Document,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    start: usize,
    kind: PaintShaderKind,
    paint_opacity: f32,
) -> CollectedGradient {
    let mut output = CollectedGradient::default();
    let mut visited = HashSet::new();
    let mut current = Some(start);
    while let Some(node) = current {
        if !visited.insert(node) {
            break;
        }
        let element = document.Node(node);
        let linear = element.IsElement("linearGradient", DOMNamespace::kSVG);
        let radial = element.IsElement("radialGradient", DOMNamespace::kSVG);
        if !linear && !radial {
            break;
        }
        if output.spread.is_none() {
            output.spread =
                Attribute(document, node, "spreadMethod").map(|value| match value.as_str() {
                    "reflect" => PaintSpreadMethod::kReflect,
                    "repeat" => PaintSpreadMethod::kRepeat,
                    _ => PaintSpreadMethod::kPad,
                });
        }
        if output.object_bounding_box.is_none() {
            output.object_bounding_box =
                Attribute(document, node, "gradientUnits").map(|value| value != "userSpaceOnUse");
        }
        if output.transform.is_none() {
            output.transform = Attribute(document, node, "gradientTransform")
                .as_deref()
                .and_then(ParseGradientTransform);
        }
        if output.stops.is_none() {
            output.stops = Stops(document, pending, node, paint_opacity);
        }
        if kind == PaintShaderKind::kLinearGradient && linear {
            output.x1 = output.x1.or_else(|| Attribute(document, node, "x1"));
            output.y1 = output.y1.or_else(|| Attribute(document, node, "y1"));
            output.x2 = output.x2.or_else(|| Attribute(document, node, "x2"));
            output.y2 = output.y2.or_else(|| Attribute(document, node, "y2"));
        }
        if kind == PaintShaderKind::kRadialGradient && radial {
            output.cx = output.cx.or_else(|| Attribute(document, node, "cx"));
            output.cy = output.cy.or_else(|| Attribute(document, node, "cy"));
            output.radius = output.radius.or_else(|| Attribute(document, node, "r"));
            output.fx = output.fx.or_else(|| Attribute(document, node, "fx"));
            output.fy = output.fy.or_else(|| Attribute(document, node, "fy"));
            output.focal_radius = output
                .focal_radius
                .or_else(|| Attribute(document, node, "fr"));
        }
        current = ReferencedGradient(document, node);
    }
    output
}

#[derive(Default)]
struct CollectedPattern {
    object_bounding_box: Option<bool>,
    content_object_bounding_box: Option<bool>,
    transform: Option<TransformMatrix>,
    x: Option<String>,
    y: Option<String>,
    width: Option<String>,
    height: Option<String>,
    view_box: Option<Option<SvgViewBoxData>>,
    preserve_aspect_ratio: Option<String>,
    content_node: Option<usize>,
}

fn ParseViewBox(value: &str) -> Option<SvgViewBoxData> {
    let values = value
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() != 4
        || !values.iter().all(|value| value.is_finite())
        || values[2] <= 0.0
        || values[3] <= 0.0
    {
        return None;
    }
    Some(SvgViewBoxData {
        x: values[0],
        y: values[1],
        width: values[2],
        height: values[3],
        ..SvgViewBoxData::default()
    })
}

fn ApplyPreserveAspectRatio(view_box: &mut SvgViewBoxData, value: Option<&str>) -> bool {
    let Some(value) = value else { return true };
    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
    let mut index = usize::from(parts.first() == Some(&"defer"));
    if index >= parts.len() {
        return false;
    }
    let align = parts[index];
    index += 1;
    if align == "none" {
        view_box.preserve_none = true;
    } else {
        let bytes = align.as_bytes();
        if bytes.len() != 8 || bytes[0] != b'x' || bytes[4] != b'Y' {
            return false;
        }
        view_box.align_x = match &bytes[1..4] {
            b"Min" => -1,
            b"Mid" => 0,
            b"Max" => 1,
            _ => return false,
        };
        view_box.align_y = match &bytes[5..8] {
            b"Min" => -1,
            b"Mid" => 0,
            b"Max" => 1,
            _ => return false,
        };
    }
    if index < parts.len() {
        view_box.slice = match parts[index] {
            "meet" => false,
            "slice" => true,
            _ => return false,
        };
        index += 1;
    }
    index == parts.len()
}

fn CollectPattern(document: &Document, start: usize) -> Option<CollectedPattern> {
    let mut output = CollectedPattern::default();
    let mut visited = HashSet::new();
    let mut current = Some(start);
    while let Some(node) = current {
        if !visited.insert(node) {
            return None;
        }
        if !document.Node(node).IsElement("pattern", DOMNamespace::kSVG) {
            return None;
        }
        if output.object_bounding_box.is_none() {
            output.object_bounding_box =
                Attribute(document, node, "patternUnits").map(|value| value != "userSpaceOnUse");
        }
        if output.content_object_bounding_box.is_none() {
            output.content_object_bounding_box = Attribute(document, node, "patternContentUnits")
                .map(|value| value == "objectBoundingBox");
        }
        if output.transform.is_none() {
            if let Some(value) = Attribute(document, node, "patternTransform") {
                output.transform = Some(ParseGradientTransform(&value)?);
            }
        }
        output.x = output.x.or_else(|| Attribute(document, node, "x"));
        output.y = output.y.or_else(|| Attribute(document, node, "y"));
        output.width = output.width.or_else(|| Attribute(document, node, "width"));
        output.height = output
            .height
            .or_else(|| Attribute(document, node, "height"));
        if output.view_box.is_none() {
            if let Some(value) = Attribute(document, node, "viewBox") {
                // Preserve malformed specified viewBox as an invalid pattern,
                // rather than inheriting a referenced element's viewBox.
                output.view_box = Some(ParseViewBox(&value));
            }
        }
        output.preserve_aspect_ratio = output
            .preserve_aspect_ratio
            .or_else(|| Attribute(document, node, "preserveAspectRatio"));
        if output.content_node.is_none() && !document.Node(node).Children().is_empty() {
            output.content_node = Some(node);
        }
        current = match Reference(document, node) {
            None => None,
            Some(reference) => {
                // External resource documents need their own lifecycle and
                // security owner. Do not reinterpret them as local patterns.
                let id = LocalFragment(reference)?;
                Some(FindById(document, node, id)?)
            }
        };
    }
    Some(output)
}

fn Multiply(left: &TransformMatrix, right: &TransformMatrix) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    for column in 0..4 {
        for row in 0..4 {
            output.values[column * 4 + row] = (0..4)
                .map(|index| left.values[index * 4 + row] * right.values[column * 4 + index])
                .sum();
        }
    }
    output
}

fn Translation(x: f64, y: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[12] = x;
    output.values[13] = y;
    output
}

fn Scale(x: f64, y: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[0] = x;
    output.values[5] = y;
    output
}

fn Rotation(angle: f64) -> TransformMatrix {
    let (sin, cos) = angle.sin_cos();
    let mut output = TransformMatrix::default();
    output.values[0] = cos;
    output.values[1] = sin;
    output.values[4] = -sin;
    output.values[5] = cos;
    output
}

fn MapPoint(matrix: &TransformMatrix, point: Offset) -> Offset {
    Offset {
        x: matrix.values[0] * point.x + matrix.values[4] * point.y + matrix.values[12],
        y: matrix.values[1] * point.x + matrix.values[5] * point.y + matrix.values[13],
    }
}

fn PatternViewportTransform(
    document: &Document,
    node: usize,
    width: f64,
    height: f64,
) -> Option<TransformMatrix> {
    let mut view_box = ParseViewBox(&Attribute(document, node, "viewBox")?)?;
    if !ApplyPreserveAspectRatio(
        &mut view_box,
        Attribute(document, node, "preserveAspectRatio").as_deref(),
    ) {
        return None;
    }
    let mut scale_x = width / view_box.width;
    let mut scale_y = height / view_box.height;
    let mut offset_x = 0.0;
    let mut offset_y = 0.0;
    if !view_box.preserve_none {
        let scale = if view_box.slice {
            scale_x.max(scale_y)
        } else {
            scale_x.min(scale_y)
        };
        scale_x = scale;
        scale_y = scale;
        let extra_x = width - view_box.width * scale;
        let extra_y = height - view_box.height * scale;
        offset_x = match view_box.align_x {
            -1 => 0.0,
            1 => extra_x,
            _ => extra_x * 0.5,
        };
        offset_y = match view_box.align_y {
            -1 => 0.0,
            1 => extra_y,
            _ => extra_y * 0.5,
        };
    }
    let mut scale = TransformMatrix::default();
    scale.values[0] = scale_x;
    scale.values[5] = scale_y;
    Some(Multiply(
        &Translation(offset_x, offset_y),
        &Multiply(&scale, &Translation(-view_box.x, -view_box.y)),
    ))
}

#[derive(Clone, Copy, Debug)]
enum PatternContentError {
    ExternalUse,
    MissingUseTarget,
    RecursiveUse,
    UseStyleRequiresInstanceCascade,
    InvalidUseViewport,
    ImageResourceLifecycle,
    TextRequiresShaping,
    ClipRequiresResourceGraph,
    MaskRequiresResourceGraph,
    ReferenceFilterRequiresSurface,
    UnsupportedElement,
}

fn PatternFilters(
    filters: &[layoutng::internal::paint_input::PaintFilterOperation],
) -> Result<Vec<layoutng::internal::paint_input::PaintFilterOperation>, PatternContentError> {
    for filter in filters {
        if filter.r#type == PaintFilterType::kReference {
            if filter.reference_error.is_some()
                || filter.reference_graph.is_none()
                || filter
                    .reference_graph
                    .as_ref()
                    .is_some_and(|graph| graph.unsupported.is_some())
            {
                return Err(PatternContentError::ReferenceFilterRequiresSurface);
            }
        } else if filter.reference_graph.is_some() || filter.reference_error.is_some() {
            return Err(PatternContentError::ReferenceFilterRequiresSurface);
        }
    }
    Ok(filters.to_vec())
}

fn PushPatternFilters(
    output: &mut Vec<SvgPatternCommand>,
    filters: &[layoutng::internal::paint_input::PaintFilterOperation],
) -> usize {
    // Nested saveLayers restore from the inside out. Push the CSS list in
    // reverse so filters still execute in authored left-to-right order.
    for filter in filters.iter().rev() {
        if filter.r#type == PaintFilterType::kReference {
            output.push(SvgPatternCommand::SaveLayerReferenceFilter(Arc::new(
                SvgPatternReferenceFilter {
                    graph: Arc::clone(
                        filter
                            .reference_graph
                            .as_ref()
                            .expect("validated SVG reference filter graph"),
                    ),
                    reference: None,
                },
            )));
        } else {
            output.push(SvgPatternCommand::SaveLayerFilter(vec![filter.clone()]));
        }
    }
    filters.len()
}

fn ValidatePatternEffects(
    paint: &layoutng::internal::paint_input::PaintStyleData,
) -> Result<(), PatternContentError> {
    PatternFilters(&paint.filters).map(|_| ())
}

fn PatternClip(
    paint: &layoutng::internal::paint_input::PaintStyleData,
) -> Result<Option<Arc<layoutng::internal::paint_input::SvgClipPathGraph>>, PatternContentError> {
    let Some(clip) = paint.clip_path.as_ref() else {
        return Ok(None);
    };
    if clip.reference_error.is_some() {
        return Err(PatternContentError::ClipRequiresResourceGraph);
    }
    clip.reference_graph
        .as_ref()
        .map(|graph| Some(Arc::clone(graph)))
        .ok_or(PatternContentError::ClipRequiresResourceGraph)
}


fn PatternMaskLength(value: Option<String>, default: &str) -> Option<SvgClipLength> {
    let value = value.as_deref().unwrap_or(default).trim();
    if let Some(percent) = value.strip_suffix('%') {
        return percent
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .map(|v| SvgClipLength {
                value: 0.0,
                percentage: v / 100.0,
            });
    }
    ParseLength(value, LengthAxis::Other, false, None).map(|value| SvgClipLength {
        value,
        percentage: 0.0,
    })
}

fn PatternMask(
    document: &Document,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    node: usize,
    paint: &layoutng::internal::paint_input::PaintStyleData,
    resolving: &mut HashSet<usize>,
) -> Result<Option<Arc<SvgMaskGraph>>, PatternContentError> {
    // Consume the same native projection as ordinary Layout/Paint. The older
    // layout-style projection intentionally omits effect/stroke/mask fields.
    let canonical = Style(document,pending,node).and_then(|style|unsafe{style.native_style.Get().as_ref()})
        .map(|style|layoutng::internal::boundary::native_paint_style::Export(style,None));
    let paint = canonical.as_ref().unwrap_or(paint);
    if paint.mask_images.is_empty() { return Ok(None); }
    if paint.mask_images.len() != 1 { return Err(PatternContentError::MaskRequiresResourceGraph); }
    let reference = paint.mask_images[0].svg_reference.as_ref()
        .ok_or(PatternContentError::MaskRequiresResourceGraph)?;
    let id = reference.id.as_str();
    if reference.scope.document_id != document.SVGResourceScope(node).document_id {return Err(PatternContentError::MaskRequiresResourceGraph);}
    let scope = document.FindNodeById(reference.scope.tree_scope_id).ok_or(PatternContentError::MaskRequiresResourceGraph)?;
    let target = document.FindSVGResourceById(scope,id).ok_or(PatternContentError::MaskRequiresResourceGraph)?;
    if !document.Node(target).IsElement("mask",DOMNamespace::kSVG) || !resolving.insert(target) {
        return Err(PatternContentError::MaskRequiresResourceGraph);
    }
    let mask_units = match Attribute(document, target, "maskUnits").as_deref() {
        Some("userSpaceOnUse") => SvgMaskUnits::kUserSpaceOnUse,
        None | Some("objectBoundingBox") => SvgMaskUnits::kObjectBoundingBox,
        _ => {
            resolving.remove(&target);
            return Err(PatternContentError::MaskRequiresResourceGraph);
        }
    };
    let content_units = match Attribute(document, target, "maskContentUnits").as_deref() {
        Some("objectBoundingBox") => SvgMaskUnits::kObjectBoundingBox,
        None | Some("userSpaceOnUse") => SvgMaskUnits::kUserSpaceOnUse,
        _ => {
            resolving.remove(&target);
            return Err(PatternContentError::MaskRequiresResourceGraph);
        }
    };
    let mode = Style(document, pending, target)
        .and_then(|style| unsafe { style.native_style.Get().as_ref() })
        .map_or(PaintMaskMode::kLuminance, |style| match style.MaskType() {
            foundation::EMaskType::kAlpha => PaintMaskMode::kAlpha,
            foundation::EMaskType::kLuminance => PaintMaskMode::kLuminance,
        });
    let result = (|| {
        Some(SvgMaskGraph {
            id: id.to_string(),
            mask_units,
            content_units,
            x: PatternMaskLength(Attribute(document, target, "x"), "-10%")?,
            y: PatternMaskLength(Attribute(document, target, "y"), "-10%")?,
            width: PatternMaskLength(Attribute(document, target, "width"), "120%")?,
            height: PatternMaskLength(Attribute(document, target, "height"), "120%")?,
            mode,
            commands: PatternPrimitives(document, pending, target, resolving, None, None).ok()?,
            content_transform: TransformMatrix::default(),
            resolved_clip: None,
            unsupported: None,
        })
    })();
    resolving.remove(&target);
    result
        .map(|graph| Some(Arc::new(graph)))
        .ok_or(PatternContentError::MaskRequiresResourceGraph)
}

fn PatternPrimitives(
    document: &Document,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    root: usize,
    resolving: &mut HashSet<usize>,
    context_paint: Option<&PaintStyleData>,
    context_bounds: Option<&SvgPaintServerReferenceBox>,
) -> Result<Vec<SvgPatternCommand>, PatternContentError> {
    PatternPrimitivesWithShapes(document,pending,root,resolving,context_paint,context_bounds,None)
}


fn PatternPrimitivesWithShapes(
    document: &Document,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    root: usize,
    resolving: &mut HashSet<usize>,
    context_paint: Option<&PaintStyleData>,
    context_bounds: Option<&SvgPaintServerReferenceBox>,
    shapes: Option<&HashMap<usize, SvgShapeData>>,
) -> Result<Vec<SvgPatternCommand>, PatternContentError> {
    fn append(
        document: &Document,
        pending: &HashMap<usize, ResolvedNodeStyle>,
        node: usize,
        parent_transform: TransformMatrix,
        resolving: &mut HashSet<usize>,
        instances: &mut HashSet<usize>,
        output: &mut Vec<SvgPatternCommand>,
        instance_root: bool,
        context_paint: Option<&PaintStyleData>,
        context_bounds: Option<&SvgPaintServerReferenceBox>,
        shapes: Option<&HashMap<usize, SvgShapeData>>,
    ) -> Result<(), PatternContentError> {
        let element = document.Node(node);
        if !matches!(element.Namespace(), DOMNamespace::kSVG) {
            return Ok(());
        }
        if Style(document,pending,node).and_then(|style| unsafe {style.native_style.Get().as_ref()})
            .is_some_and(|style| style.Display()==foundation::EDisplay::kNone) {return Ok(());}
        let local_transform = if shapes.is_some() { TransformMatrix::default() } else { Attribute(document, node, "transform")
            .as_deref()
            .and_then(ParseGradientTransform)
            .unwrap_or_default() };
        let transform = Multiply(&parent_transform, &local_transform);
        if element.Name() == "use" {
            let reference =
                Reference(document, node).ok_or(PatternContentError::MissingUseTarget)?;
            let id = LocalFragment(reference).ok_or(PatternContentError::ExternalUse)?;
            let target = FindById(document, node, id).ok_or(PatternContentError::MissingUseTarget)?;
            if !instances.insert(target) {
                return Err(PatternContentError::RecursiveUse);
            }
            let use_style = Style(document, pending, node)
                .ok_or(PatternContentError::UseStyleRequiresInstanceCascade)?;
            let target_style = Style(document, pending, target)
                .ok_or(PatternContentError::UseStyleRequiresInstanceCascade)?;
            // A translated SVGUseElement builds an instance style from the
            // corresponding element plus the <use> inheritance parent. Until
            // that StyleEngine entry exists, only reuse an already-equivalent
            // immutable paint style; guessing which inherited field won would
            // silently change fill/stroke/currentColor.
            if use_style.style.paint != target_style.style.paint {
                instances.remove(&target);
                return Err(PatternContentError::UseStyleRequiresInstanceCascade);
            }
            let x = Attribute(document, node, "x")
                .as_deref()
                .map_or(Some(0.0), |value| {
                    ParseLength(value, LengthAxis::X, false, None)
                })
                .ok_or(PatternContentError::InvalidUseViewport)?;
            let y = Attribute(document, node, "y")
                .as_deref()
                .map_or(Some(0.0), |value| {
                    ParseLength(value, LengthAxis::Y, false, None)
                })
                .ok_or(PatternContentError::InvalidUseViewport)?;
            let mut instance_transform = Multiply(&transform, &Translation(x, y));
            let target_element = document.Node(target);
            if matches!(target_element.Name(), "symbol" | "svg")
                && target_element.FindAttribute("viewBox").is_some()
            {
                let width = Attribute(document, node, "width")
                    .or_else(|| Attribute(document, target, "width"))
                    .as_deref()
                    .and_then(|value| ParseLength(value, LengthAxis::Width, false, None))
                    .ok_or(PatternContentError::InvalidUseViewport)?;
                let height = Attribute(document, node, "height")
                    .or_else(|| Attribute(document, target, "height"))
                    .as_deref()
                    .and_then(|value| ParseLength(value, LengthAxis::Height, false, None))
                    .ok_or(PatternContentError::InvalidUseViewport)?;
                if width <= 0.0 || height <= 0.0 {
                    instances.remove(&target);
                    return Err(PatternContentError::InvalidUseViewport);
                }
                let viewport = PatternViewportTransform(document, target, width, height)
                    .ok_or(PatternContentError::InvalidUseViewport)?;
                instance_transform = Multiply(&instance_transform, &viewport);
            }
            let result = append(
                document,
                pending,
                target,
                instance_transform,
                resolving,
                instances,
                output,
                true,
                context_paint,
                context_bounds,
                shapes,
            );
            instances.remove(&target);
            return result;
        }
        if let Some(shape) =
            shapes.map_or_else(|| dom::layout_mapping::layout_object_tree_builder::SVGShapeDataForResourceNode(element), |shapes| shapes.get(&node).cloned())
        {
            let Some(style) = Style(document, pending, node) else {
                return Ok(());
            };
            let native = unsafe { style.native_style.Get().as_ref() };
            if !style.style.paint.visible
                || native.is_some_and(|style| style.Display() == foundation::EDisplay::kNone)
            {
                return Ok(());
            }
            let paint = &style.style.paint;
            ValidatePatternEffects(paint)?;
            let filters = PatternFilters(&paint.filters)?;
            let clip = PatternClip(paint)?;
            let has_clip = clip.is_some();
            let mask = PatternMask(document, pending, node, paint, resolving)?;
            let has_mask = mask.is_some();
            let has_blend = paint.blend_mode
                != layoutng::internal::paint_input::PaintBlendMode::kNormal
                || paint.isolate_blending;
            let fill_opacity = native.map_or(1.0, |style| style.FillOpacity());
            let stroke_opacity = native.map_or(1.0, |style| style.StrokeOpacity());
            let mut fill_server = paint.svg_fill_reference.as_deref().and_then(|reference| {
                ResolveInner(document, node, pending, reference, fill_opacity, resolving)
            });
            let mut stroke_server = paint.svg_stroke_reference.as_deref().and_then(|reference| {
                ResolveInner(document, node, pending, reference, stroke_opacity, resolving)
            });
            let mut fill = paint.svg_fill;
            let mut stroke = paint.svg_stroke;
            let mut fill_is_context = false;
            let mut stroke_is_context = false;
            if let (Some(native), Some(context)) = (native, context_paint) {
                match native.FillPaint().paint_type {
                    SVGPaintType::kContextFill => {
                        fill_is_context = true;
                        fill = context.svg_fill;
                        fill_server = context.svg_fill_server.clone();
                    }
                    SVGPaintType::kContextStroke => {
                        fill_is_context = true;
                        fill = context.svg_stroke;
                        fill_server = context.svg_stroke_server.clone();
                    }
                    _ => {}
                }
                match native.StrokePaint().paint_type {
                    SVGPaintType::kContextFill => {
                        stroke_is_context = true;
                        stroke = context.svg_fill;
                        stroke_server = context.svg_fill_server.clone();
                    }
                    SVGPaintType::kContextStroke => {
                        stroke_is_context = true;
                        stroke = context.svg_stroke;
                        stroke_server = context.svg_stroke_server.clone();
                    }
                    _ => {}
                }
            }
            if has_blend {
                output.push(SvgPatternCommand::SaveLayerBlend(paint.blend_mode));
            }
            if paint.opacity != 1.0 {
                output.push(SvgPatternCommand::SaveLayerOpacity(paint.opacity));
            }
            let filter_layers = PushPatternFilters(output, &filters);
            if let Some(clip) = clip {
                output.push(SvgPatternCommand::SaveLayerClip(clip));
            }
            if let Some(mask) = mask {
                output.push(SvgPatternCommand::SaveLayerMask(mask));
            }
            output.push(SvgPatternCommand::DrawShape(SvgMarkerPrimitive {
                shape,
                fill,
                stroke,
                fill_server,
                stroke_server,
                fill_server_reference_box: fill_is_context
                    .then(|| context_bounds.cloned())
                    .flatten(),
                stroke_server_reference_box: stroke_is_context
                    .then(|| context_bounds.cloned())
                    .flatten(),
                stroke_width: paint.svg_stroke_width,
                stroke_dash_array: paint.svg_stroke_dash_array.clone(),
                stroke_dash_offset: paint.svg_stroke_dash_offset,
                stroke_line_cap: paint.svg_stroke_line_cap,
                stroke_line_join: paint.svg_stroke_line_join,
                stroke_miter_limit: paint.svg_stroke_miter_limit,
                fill_even_odd: paint.svg_fill_even_odd,
                non_scaling_stroke: paint.svg_non_scaling_stroke,
                antialias: paint.svg_shape_antialias,
                transform: Some(transform),
            }));
            if has_mask {
                output.push(SvgPatternCommand::Restore);
            }
            if has_clip {
                output.push(SvgPatternCommand::Restore);
            }
            for _ in 0..filter_layers {
                output.push(SvgPatternCommand::Restore);
            }
            if paint.opacity != 1.0 {
                output.push(SvgPatternCommand::Restore);
            }
            if has_blend {
                output.push(SvgPatternCommand::Restore);
            }
            return Ok(());
        }
        // Resource/definition elements inside the content subtree do not
        // render by themselves. Container graphics elements inherit their
        // already-computed style into descendants and contribute transforms.
        if matches!(
            element.Name(),
            "defs" | "pattern" | "linearGradient" | "radialGradient"
        ) {
            return Ok(());
        }
        match element.Name() {
            "image" => {
                let style = Style(document, pending, node)
                    .ok_or(PatternContentError::ImageResourceLifecycle)?;
                let native = unsafe { style.native_style.Get().as_ref() };
                if !style.style.paint.visible
                    || native.is_some_and(|style| style.Display() == foundation::EDisplay::kNone)
                {
                    return Ok(());
                }
                let href =
                    Reference(document, node).ok_or(PatternContentError::ImageResourceLifecycle)?;
                // Local fragments are graphics references, not image resource
                // fetches. They require the SVG use/instance lifecycle.
                if LocalFragment(href).is_some() {
                    return Err(PatternContentError::ImageResourceLifecycle);
                }
                let resource = document
                    .ImageResourceFor(href)
                    .ok_or(PatternContentError::ImageResourceLifecycle)?;
                if resource.id == 0 {
                    return Err(PatternContentError::ImageResourceLifecycle);
                }
                let x = Attribute(document, node, "x")
                    .as_deref()
                    .map_or(Some(0.0), |value| {
                        ParseLength(value, LengthAxis::X, false, None)
                    })
                    .ok_or(PatternContentError::ImageResourceLifecycle)?;
                let y = Attribute(document, node, "y")
                    .as_deref()
                    .map_or(Some(0.0), |value| {
                        ParseLength(value, LengthAxis::Y, false, None)
                    })
                    .ok_or(PatternContentError::ImageResourceLifecycle)?;
                let width = Attribute(document, node, "width")
                    .as_deref()
                    .and_then(|value| ParseLength(value, LengthAxis::Width, false, None))
                    .ok_or(PatternContentError::ImageResourceLifecycle)?;
                let height = Attribute(document, node, "height")
                    .as_deref()
                    .and_then(|value| ParseLength(value, LengthAxis::Height, false, None))
                    .ok_or(PatternContentError::ImageResourceLifecycle)?;
                if ![x, y, width, height].iter().all(|value| value.is_finite())
                    || width <= 0.0
                    || height <= 0.0
                {
                    return Err(PatternContentError::ImageResourceLifecycle);
                }
                let mut aspect = SvgViewBoxData::default();
                if !ApplyPreserveAspectRatio(
                    &mut aspect,
                    Attribute(document, node, "preserveAspectRatio").as_deref(),
                ) {
                    return Err(PatternContentError::ImageResourceLifecycle);
                }
                let paint = &style.style.paint;
                ValidatePatternEffects(paint)?;
                let filters = PatternFilters(&paint.filters)?;
                let filter_layers = filters.len();
                let clip = PatternClip(paint)?;
                let has_clip = clip.is_some();
                let mask = PatternMask(document, pending, node, paint, resolving)?;
                let has_mask = mask.is_some();
                let has_blend = paint.blend_mode
                    != layoutng::internal::paint_input::PaintBlendMode::kNormal
                    || paint.isolate_blending;
                if has_blend {
                    output.push(SvgPatternCommand::SaveLayerBlend(paint.blend_mode));
                }
                if paint.opacity != 1.0 {
                    output.push(SvgPatternCommand::SaveLayerOpacity(paint.opacity));
                }
                PushPatternFilters(output, &filters);
                if let Some(clip) = clip {
                    output.push(SvgPatternCommand::SaveLayerClip(clip));
                }
                if let Some(mask) = mask {
                    output.push(SvgPatternCommand::SaveLayerMask(mask));
                }
                output.push(SvgPatternCommand::DrawImage(SvgPatternImage {
                    resource_id: resource.id,
                    image_interpolation_quality: paint.image_interpolation_quality,
                    destination_offset: Offset { x, y },
                    destination_width: width,
                    destination_height: height,
                    preserve_none: aspect.preserve_none,
                    align_x: aspect.align_x,
                    align_y: aspect.align_y,
                    slice: aspect.slice,
                    transform,
                    color_scheme_flags_is_normal: paint.color_scheme_flags_is_normal,
                    used_dark_color_scheme: paint.used_dark_color_scheme,
                    ..SvgPatternImage::default()
                }));
                if has_mask {
                    output.push(SvgPatternCommand::Restore);
                }
                if has_clip {
                    output.push(SvgPatternCommand::Restore);
                }
                for _ in 0..filter_layers {
                    output.push(SvgPatternCommand::Restore);
                }
                if paint.opacity != 1.0 {
                    output.push(SvgPatternCommand::Restore);
                }
                if has_blend {
                    output.push(SvgPatternCommand::Restore);
                }
                return Ok(());
            }
            "text" | "tspan" | "textPath" => {}
            "g" | "svg" | "symbol" | "a" | "switch" => {}
            _ => return Err(PatternContentError::UnsupportedElement),
        }
        if let Some(style) = Style(document, pending, node) {
            let paint = &style.style.paint;
            if !instance_root
                && unsafe { style.native_style.Get().as_ref() }
                    .is_some_and(|style| style.Display() == foundation::EDisplay::kNone)
                || !paint.visible
            {
                return Ok(());
            }
            ValidatePatternEffects(paint)?;
        }
        let pattern_paint = Style(document, pending, node).map(|style| &style.style.paint);
        let opacity = pattern_paint.map_or(1.0, |paint| paint.opacity);
        let blend_mode = pattern_paint.map_or(
            layoutng::internal::paint_input::PaintBlendMode::kNormal,
            |paint| paint.blend_mode,
        );
        let has_blend = pattern_paint.is_some_and(|paint| {
            paint.blend_mode != layoutng::internal::paint_input::PaintBlendMode::kNormal
                || paint.isolate_blending
        });
        if has_blend {
            output.push(SvgPatternCommand::SaveLayerBlend(blend_mode));
        }
        if opacity != 1.0 {
            output.push(SvgPatternCommand::SaveLayerOpacity(opacity));
        }
        let filters = Style(document, pending, node)
            .map(|style| PatternFilters(&style.style.paint.filters))
            .transpose()?
            .unwrap_or_default();
        let filter_layers = PushPatternFilters(output, &filters);
        let clip = pattern_paint
            .map(|paint| PatternClip(paint))
            .transpose()?
            .flatten();
        let has_clip = clip.is_some();
        if let Some(clip) = clip {
            output.push(SvgPatternCommand::SaveLayerClip(clip));
        }
        let mask = pattern_paint
            .map(|paint| PatternMask(document, pending, node, paint, resolving))
            .transpose()?
            .flatten();
        let has_mask = mask.is_some();
        if let Some(mask) = mask {
            output.push(SvgPatternCommand::SaveLayerMask(mask));
        }
        if matches!(element.Name(), "text" | "tspan" | "textPath") {
            let style =
                Style(document, pending, node).ok_or(PatternContentError::TextRequiresShaping)?;
            let text = DirectSvgText(document, node);
            if !text.is_empty() {
                let native = unsafe { style.native_style.Get().as_ref() }
                    .ok_or(PatternContentError::TextRequiresShaping)?;
                let paint = &style.style.paint;
                let text_style = style
                    .style
                    .extended
                    .as_ref()
                    .ok_or(PatternContentError::TextRequiresShaping)?;
                let fill_opacity = native.FillOpacity();
                let stroke_opacity = native.StrokeOpacity();
                let fill_server = paint.svg_fill_reference.as_deref().and_then(|reference| {
                    ResolveInner(document, node, pending, reference, fill_opacity, resolving)
                });
                let stroke_server = paint.svg_stroke_reference.as_deref().and_then(|reference| {
                    ResolveInner(document, node, pending, reference, stroke_opacity, resolving)
                });
                let anchor = match native.TextAnchor() {
                    foundation::ETextAnchor::kMiddle => SvgClipTextAnchor::kMiddle,
                    foundation::ETextAnchor::kEnd => SvgClipTextAnchor::kEnd,
                    _ => SvgClipTextAnchor::kStart,
                };
                output.push(SvgPatternCommand::DrawText(SvgPatternText {
                    text: SvgClipText {
                        text,
                        x: SvgTextLengthList(document, node, "x")
                            .ok_or(PatternContentError::TextRequiresShaping)?,
                        y: SvgTextLengthList(document, node, "y")
                            .ok_or(PatternContentError::TextRequiresShaping)?,
                        dx: SvgTextLengthList(document, node, "dx")
                            .ok_or(PatternContentError::TextRequiresShaping)?,
                        dy: SvgTextLengthList(document, node, "dy")
                            .ok_or(PatternContentError::TextRequiresShaping)?,
                        rotate: SvgTextNumberList(document, node, "rotate")
                            .ok_or(PatternContentError::TextRequiresShaping)?,
                        font_families: text_style.font_families.clone(),
                        font_size: text_style.font_size,
                        font_weight: text_style.font_weight,
                        italic: text_style.font_italic,
                        letter_spacing: text_style.letter_spacing,
                        word_spacing: text_style.word_spacing,
                        rtl: style.style.direction == TextDirection::kRtl,
                        language: text_style.language.clone(),
                        anchor,
                        text_path: PatternTextPath(document, node)?,
                    },
                    paint: SvgMarkerPrimitive {
                        shape: SvgShapeData::default(),
                        fill: paint.svg_fill,
                        stroke: paint.svg_stroke,
                        fill_server,
                        stroke_server,
                        fill_server_reference_box: None,
                        stroke_server_reference_box: None,
                        stroke_width: paint.svg_stroke_width,
                        stroke_dash_array: paint.svg_stroke_dash_array.clone(),
                        stroke_dash_offset: paint.svg_stroke_dash_offset,
                        stroke_line_cap: paint.svg_stroke_line_cap,
                        stroke_line_join: paint.svg_stroke_line_join,
                        stroke_miter_limit: paint.svg_stroke_miter_limit,
                        fill_even_odd: false,
                        non_scaling_stroke: paint.svg_non_scaling_stroke,
                        antialias: paint.svg_shape_antialias,
                        transform: Some(transform),
                    },
                    reset_cursor: element.Name() == "text",
                }));
            }
        }
        for &child in element.Children() {
            append(
                document,
                pending,
                child,
                transform,
                resolving,
                instances,
                output,
                false,
                context_paint,
                context_bounds,
                shapes,
            )?;
        }
        if has_mask {
            output.push(SvgPatternCommand::Restore);
        }
        if has_clip {
            output.push(SvgPatternCommand::Restore);
        }
        for _ in 0..filter_layers {
            output.push(SvgPatternCommand::Restore);
        }
        if opacity != 1.0 {
            output.push(SvgPatternCommand::Restore);
        }
        if has_blend {
            output.push(SvgPatternCommand::Restore);
        }
        Ok(())
    }

    let mut output = Vec::new();
    let mut instances = HashSet::new();
    for &child in document.Node(root).Children() {
        append(
            document,
            pending,
            child,
            TransformMatrix::default(),
            resolving,
            &mut instances,
            &mut output,
            false,
            context_paint,
            context_bounds,
            shapes,
        )?;
    }
    Ok(output)
}

#[derive(Clone, Copy)]
enum MarkerPosition {
    Start,
    Mid,
    End,
}

#[derive(Clone, Copy)]
struct MarkerPlacement {
    position: MarkerPosition,
    point: Offset,
    angle: f64,
}

#[derive(Clone, Copy)]
struct MarkerSegment {
    start: Offset,
    end: Offset,
    start_tangent: Offset,
    end_tangent: Offset,
}

fn Subtract(left: Offset, right: Offset) -> Offset {
    Offset {
        x: left.x - right.x,
        y: left.y - right.y,
    }
}

fn NonZero(vector: Offset) -> bool {
    vector.x.abs() > 1e-12 || vector.y.abs() > 1e-12
}

fn SegmentAngle(vector: Offset) -> f64 {
    vector.y.atan2(vector.x)
}

fn BisectAngles(incoming: Offset, outgoing: Offset) -> f64 {
    let incoming = SegmentAngle(incoming);
    let outgoing = SegmentAngle(outgoing);
    let mut delta = outgoing - incoming;
    while delta > std::f64::consts::PI {
        delta -= std::f64::consts::TAU;
    }
    while delta < -std::f64::consts::PI {
        delta += std::f64::consts::TAU;
    }
    incoming + delta * 0.5
}

/// SVGMarkerData in Blink derives marker positions from path segment endpoints
/// and their incoming/outgoing tangents. Keep that geometry above Layout so
/// the immutable marker instances crossing into Paint need no DOM access.
fn MarkerPlacements(shape: &SvgShapeData) -> Vec<MarkerPlacement> {
    if shape.geometry != layoutng::internal::layout_input::SvgShapeGeometry::kPath {
        return Vec::new();
    }
    fn flush(segments: &mut Vec<MarkerSegment>, output: &mut Vec<MarkerPlacement>) {
        if segments.is_empty() {
            return;
        }
        let first = segments[0];
        let last = *segments.last().unwrap();
        let first_tangent = segments
            .iter()
            .map(|segment| segment.start_tangent)
            .find(|vector| NonZero(*vector))
            .unwrap_or_else(|| Subtract(first.end, first.start));
        output.push(MarkerPlacement {
            position: MarkerPosition::Start,
            point: first.start,
            angle: SegmentAngle(first_tangent),
        });
        for index in 1..segments.len() {
            let incoming = (0..index)
                .rev()
                .map(|i| segments[i].end_tangent)
                .find(|vector| NonZero(*vector))
                .unwrap_or_else(|| Subtract(segments[index - 1].end, segments[index - 1].start));
            let outgoing = (index..segments.len())
                .map(|i| segments[i].start_tangent)
                .find(|vector| NonZero(*vector))
                .unwrap_or_else(|| Subtract(segments[index].end, segments[index].start));
            output.push(MarkerPlacement {
                position: MarkerPosition::Mid,
                point: segments[index].start,
                angle: BisectAngles(incoming, outgoing),
            });
        }
        let last_tangent = segments
            .iter()
            .rev()
            .map(|segment| segment.end_tangent)
            .find(|vector| NonZero(*vector))
            .unwrap_or_else(|| Subtract(last.end, last.start));
        output.push(MarkerPlacement {
            position: MarkerPosition::End,
            point: last.end,
            angle: SegmentAngle(last_tangent),
        });
        segments.clear();
    }

    let mut output = Vec::new();
    let mut segments = Vec::new();
    let mut current = Offset::default();
    let mut subpath_start = Offset::default();
    let mut has_current = false;
    for command in &shape.path {
        match command.verb {
            PaintPathVerb::kMoveTo => {
                flush(&mut segments, &mut output);
                current = command.point;
                subpath_start = current;
                has_current = true;
            }
            PaintPathVerb::kLineTo if has_current => {
                let tangent = Subtract(command.point, current);
                segments.push(MarkerSegment {
                    start: current,
                    end: command.point,
                    start_tangent: tangent,
                    end_tangent: tangent,
                });
                current = command.point;
            }
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo if has_current => {
                let chord = Subtract(command.point, current);
                let start_tangent = Subtract(command.control1, current);
                let end_tangent = Subtract(command.point, command.control1);
                segments.push(MarkerSegment {
                    start: current,
                    end: command.point,
                    start_tangent: if NonZero(start_tangent) {
                        start_tangent
                    } else {
                        chord
                    },
                    end_tangent: if NonZero(end_tangent) {
                        end_tangent
                    } else {
                        chord
                    },
                });
                current = command.point;
            }
            PaintPathVerb::kCubicTo if has_current => {
                let chord = Subtract(command.point, current);
                let start_tangent = Subtract(command.control1, current);
                let end_tangent = Subtract(command.point, command.control2);
                segments.push(MarkerSegment {
                    start: current,
                    end: command.point,
                    start_tangent: if NonZero(start_tangent) {
                        start_tangent
                    } else {
                        chord
                    },
                    end_tangent: if NonZero(end_tangent) {
                        end_tangent
                    } else {
                        chord
                    },
                });
                current = command.point;
            }
            PaintPathVerb::kClose if has_current => {
                let tangent = Subtract(subpath_start, current);
                segments.push(MarkerSegment {
                    start: current,
                    end: subpath_start,
                    start_tangent: tangent,
                    end_tangent: tangent,
                });
                current = subpath_start;
                flush(&mut segments, &mut output);
                has_current = false;
            }
            _ => {}
        }
    }
    flush(&mut segments, &mut output);
    output
}

fn MarkerOrient(value: Option<&str>, automatic: f64, reverse_start: bool) -> Option<f64> {
    let value = value.unwrap_or("0").trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(automatic);
    }
    if value.eq_ignore_ascii_case("auto-start-reverse") {
        return Some(
            automatic
                + if reverse_start {
                    std::f64::consts::PI
                } else {
                    0.0
                },
        );
    }
    let (number, scale) = if let Some(number) = value.strip_suffix("deg") {
        (number, std::f64::consts::PI / 180.0)
    } else if let Some(number) = value.strip_suffix("grad") {
        (number, std::f64::consts::PI / 200.0)
    } else if let Some(number) = value.strip_suffix("rad") {
        (number, 1.0)
    } else if let Some(number) = value.strip_suffix("turn") {
        (number, std::f64::consts::TAU)
    } else {
        (value, std::f64::consts::PI / 180.0)
    };
    number
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|angle| angle.is_finite())
        .map(|angle| angle * scale)
}

fn ResolveMarkerResource(
    document: &Document,
    origin: usize,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    reference: &str,
    placement: MarkerPlacement,
    stroke_width: f64,
    context_paint: &PaintStyleData,
    context_bounds: &SvgPaintServerReferenceBox,
) -> Option<SvgMarkerInstance> {
    let target = FindById(document, origin, LocalFragment(reference)?)?;
    if !document
        .Node(target)
        .IsElement("marker", DOMNamespace::kSVG)
    {
        return None;
    }
    let mut resolving = HashSet::from([target]);
    let commands = PatternPrimitives(
        document,
        pending,
        target,
        &mut resolving,
        Some(context_paint),
        Some(context_bounds),
    )
    .ok()?;
    let mut primitives = commands
        .into_iter()
        .map(|command| match command {
            SvgPatternCommand::DrawShape(primitive) => Some(primitive),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    if primitives.is_empty() {
        return None;
    }
    let viewport = ViewBox(document, target);
    let length = |name: &str, default: &str, axis| {
        ParseLength(
            Attribute(document, target, name)
                .as_deref()
                .unwrap_or(default),
            axis,
            false,
            viewport,
        )
    };
    let width = length("markerWidth", "3", LengthAxis::Width)?;
    let height = length("markerHeight", "3", LengthAxis::Height)?;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let ref_x = length("refX", "0", LengthAxis::X)?;
    let ref_y = length("refY", "0", LengthAxis::Y)?;
    let units = match Attribute(document, target, "markerUnits").as_deref() {
        Some("userSpaceOnUse") => 1.0,
        None | Some("strokeWidth") => stroke_width,
        _ => return None,
    };
    let angle = MarkerOrient(
        Attribute(document, target, "orient").as_deref(),
        placement.angle,
        matches!(placement.position, MarkerPosition::Start),
    )?;
    let viewport_transform = if Attribute(document, target, "viewBox").is_some() {
        PatternViewportTransform(document, target, width, height)?
    } else {
        TransformMatrix::default()
    };
    let mapped_ref = MapPoint(&viewport_transform, Offset { x: ref_x, y: ref_y });
    let local = Multiply(
        &Translation(-mapped_ref.x, -mapped_ref.y),
        &viewport_transform,
    );
    for primitive in &mut primitives {
        primitive.transform = Some(Multiply(&local, &primitive.transform.unwrap_or_default()));
    }
    let transform = Multiply(
        &Translation(placement.point.x, placement.point.y),
        &Multiply(&Rotation(angle), &Scale(units, units)),
    );
    let clips_viewport = Style(document, pending, target)
        .and_then(|style| unsafe { style.native_style.Get().as_ref() })
        .is_none_or(|style| {
            style.OverflowX() != foundation::EOverflow::kVisible
                || style.OverflowY() != foundation::EOverflow::kVisible
        });
    Some(SvgMarkerInstance {
        resource_id: target as u64 + 1,
        transform,
        clip_offset: clips_viewport.then_some(Offset {
            x: -mapped_ref.x,
            y: -mapped_ref.y,
        }),
        clip_size: Size { width, height },
        primitives,
    })
}

/// Resolve marker-start/mid/end into path-ordered immutable instances. The
/// references stay canonical on ComputedStyle; this is the same document-owned
/// projection step used for SVG fill/stroke paint servers.
pub(crate) fn ResolveMarkers(
    document: &Document,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    node: usize,
    context_paint: &PaintStyleData,
    start_reference: Option<&str>,
    mid_reference: Option<&str>,
    end_reference: Option<&str>,
    stroke_width: f64,
) -> Vec<SvgMarkerInstance> {
    let Some(shape) = dom::layout_mapping::layout_object_tree_builder::SVGShapeDataForResourceNode(
        document.Node(node),
    ) else {
        return Vec::new();
    };
    let context_bounds = SvgPaintServerReferenceBox {
        offset: shape.bounds_offset,
        size: Size {
            width: shape.bounds_width,
            height: shape.bounds_height,
        },
    };
    MarkerPlacements(&shape)
        .into_iter()
        .filter_map(|placement| {
            let reference = match placement.position {
                MarkerPosition::Start => start_reference,
                MarkerPosition::Mid => mid_reference,
                MarkerPosition::End => end_reference,
            }?;
            ResolveMarkerResource(
                document,
                node,
                pending,
                reference,
                placement,
                stroke_width,
                context_paint,
                &context_bounds,
            )
        })
        .collect()
}

/// Resolve a canonical SVGPaint local IRI to the immutable Paint shader
/// transported through Layout. Unsupported resources return None so the
/// SVGPaint fallback color/none semantics remain authoritative.
pub(crate) fn Resolve(
    document: &Document,
    origin: usize,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    reference: &str,
    paint_opacity: f32,
) -> Option<PaintShader> {
    ResolveInner(
        document,
        origin,
        pending,
        reference,
        paint_opacity,
        &mut HashSet::new(),
    )
}

fn ResolveInner(
    document: &Document,
    origin: usize,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    reference: &str,
    paint_opacity: f32,
    resolving: &mut HashSet<usize>,
) -> Option<PaintShader> {
    let target = FindById(document, origin, LocalFragment(reference)?)?;
    if !resolving.insert(target) {
        return None;
    }
    let result = ResolveElement(document, pending, target, paint_opacity, resolving);
    resolving.remove(&target);
    result
}

fn ResolveElement(
    document: &Document,
    pending: &HashMap<usize, ResolvedNodeStyle>,
    target: usize,
    paint_opacity: f32,
    resolving: &mut HashSet<usize>,
) -> Option<PaintShader> {
    let element = document.Node(target);
    let kind = if element.IsElement("linearGradient", DOMNamespace::kSVG) {
        PaintShaderKind::kLinearGradient
    } else if element.IsElement("radialGradient", DOMNamespace::kSVG) {
        PaintShaderKind::kRadialGradient
    } else if element.IsElement("pattern", DOMNamespace::kSVG) {
        let collected = CollectPattern(document, target)?;
        let object_bounding_box = collected.object_bounding_box.unwrap_or(true);
        let viewport = (!object_bounding_box)
            .then(|| ViewBox(document, target))
            .flatten();
        let length = |value: Option<String>, default: &str, axis| {
            ParseLength(
                value.as_deref().unwrap_or(default),
                axis,
                object_bounding_box,
                viewport,
            )
        };
        let width = length(collected.width, "0", LengthAxis::Width)?;
        let height = length(collected.height, "0", LengthAxis::Height)?;
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        let mut view_box = match collected.view_box {
            Some(Some(view_box)) => Some(view_box),
            Some(None) => return None,
            None => None,
        };
        if let Some(view_box) = &mut view_box {
            if !ApplyPreserveAspectRatio(view_box, collected.preserve_aspect_ratio.as_deref()) {
                return None;
            }
        }
        let primitives = match collected.content_node {
            Some(content) => {
                match PatternPrimitives(document, pending, content, resolving, None, None) {
                    Ok(primitives) => primitives,
                    Err(error) => {
                        if std::env::var_os("BROWSER_TRACE_SVG_PAINT_SERVER").is_some() {
                            eprintln!(
                                "unsupported SVG pattern content resource_id={} reason={error:?}",
                                target as u64 + 1
                            );
                        }
                        return None;
                    }
                }
            }
            None => Vec::new(),
        };
        return Some(PaintShader {
            kind: PaintShaderKind::kPattern,
            resource_id: target as u64 + 1,
            tile_offset: Offset {
                x: length(collected.x, "0", LengthAxis::X)?,
                y: length(collected.y, "0", LengthAxis::Y)?,
            },
            tile_width: width,
            tile_height: height,
            unit_coordinates: object_bounding_box,
            object_bounding_box_coordinates: object_bounding_box,
            transform: collected.transform.unwrap_or_default(),
            pattern: Some(Arc::new(SvgPatternData {
                commands: primitives,
                content_units_object_bounding_box: collected
                    .content_object_bounding_box
                    .unwrap_or(false),
                view_box,
                content_transform: TransformMatrix::default(),
                opacity: paint_opacity,
            })),
            ..PaintShader::default()
        });
    } else {
        return None;
    };
    let collected = Collect(document, pending, target, kind, paint_opacity);
    let object_bounding_box = collected.object_bounding_box.unwrap_or(true);
    let viewport = (!object_bounding_box)
        .then(|| ViewBox(document, target))
        .flatten();
    let stops = collected.stops?;
    let mut output = PaintShader {
        kind,
        spread: collected.spread.unwrap_or(PaintSpreadMethod::kPad),
        stops,
        unit_coordinates: object_bounding_box,
        object_bounding_box_coordinates: object_bounding_box,
        transform: collected.transform.unwrap_or_default(),
        interpolate_premultiplied: true,
        ..PaintShader::default()
    };
    let length = |value: Option<String>, default: &str, axis| {
        ParseLength(
            value.as_deref().unwrap_or(default),
            axis,
            object_bounding_box,
            viewport,
        )
    };
    match kind {
        PaintShaderKind::kLinearGradient => {
            output.start = Offset {
                x: length(collected.x1, "0%", LengthAxis::X)?,
                y: length(collected.y1, "0%", LengthAxis::Y)?,
            };
            output.end = Offset {
                x: length(collected.x2, "100%", LengthAxis::X)?,
                y: length(collected.y2, "0%", LengthAxis::Y)?,
            };
        }
        PaintShaderKind::kRadialGradient => {
            let center = Offset {
                x: length(collected.cx, "50%", LengthAxis::X)?,
                y: length(collected.cy, "50%", LengthAxis::Y)?,
            };
            output.center = center;
            output.focal = Offset {
                x: collected
                    .fx
                    .and_then(|value| {
                        ParseLength(&value, LengthAxis::X, object_bounding_box, viewport)
                    })
                    .unwrap_or(center.x),
                y: collected
                    .fy
                    .and_then(|value| {
                        ParseLength(&value, LengthAxis::Y, object_bounding_box, viewport)
                    })
                    .unwrap_or(center.y),
            };
            output.radius = length(collected.radius, "50%", LengthAxis::Other)?;
            output.focal_radius = length(collected.focal_radius, "0%", LengthAxis::Other)?;
            output.radial_shape = PaintRadialShape::kCircle;
        }
        _ => unreachable!(),
    }
    Some(output)
}

/// LayoutSVGResourceMasker::CreatePaintRecord equivalent in the existing
/// immutable resource recorder. Uses the same command recording traversal as
/// patterns, fed canonical native shape/style inputs. Unsupported collaborators
/// are retained on the graph, never reported as an empty successful mask.
pub(crate) fn ResolveMaskDefinition(document:&Document, pending:&HashMap<usize,ResolvedNodeStyle>, target:usize) -> Arc<SvgMaskGraph> {
    let style=Style(document,pending,target).and_then(|s|unsafe{s.native_style.Get().as_ref()});
    let mut graph=SvgMaskGraph {
        id:Attribute(document,target,"id").unwrap_or_default(),
        mask_units:if Attribute(document,target,"maskUnits").as_deref()==Some("userSpaceOnUse") {SvgMaskUnits::kUserSpaceOnUse} else {SvgMaskUnits::kObjectBoundingBox},
        content_units:if Attribute(document,target,"maskContentUnits").as_deref()==Some("objectBoundingBox") {SvgMaskUnits::kObjectBoundingBox} else {SvgMaskUnits::kUserSpaceOnUse},
        x:SvgClipLength::default(),y:SvgClipLength::default(),width:SvgClipLength::default(),height:SvgClipLength::default(),
        mode:if style.is_some_and(|s|s.MaskType()==foundation::EMaskType::kAlpha) {PaintMaskMode::kAlpha} else {PaintMaskMode::kLuminance},
        commands:Vec::new(),content_transform:TransformMatrix::default(),resolved_clip:None,unsupported:None,
    };
    let result=(|| -> Result<Vec<SvgPatternCommand>,String> {
        for name in ["maskUnits","maskContentUnits"] {if Attribute(document,target,name).is_some_and(|s|s!="userSpaceOnUse" && s!="objectBoundingBox") {return Err(format!("invalid {name}"));}}
        if style.is_some_and(|s|s.ColorInterpolation()==foundation::EColorInterpolation::kLinearrgb) {return Err("linearRGB local mask color interpolation".into());}
        graph.x=PatternMaskLength(Attribute(document,target,"x"),"-10%").ok_or("mask x")?;
        graph.y=PatternMaskLength(Attribute(document,target,"y"),"-10%").ok_or("mask y")?;
        graph.width=PatternMaskLength(Attribute(document,target,"width"),"120%").ok_or("mask width")?;
        graph.height=PatternMaskLength(Attribute(document,target,"height"),"120%").ok_or("mask height")?;
        fn collect(document:&Document,pending:&HashMap<usize,ResolvedNodeStyle>,node:usize,shapes:&mut HashMap<usize,SvgShapeData>,canonical:&mut HashMap<usize,ResolvedNodeStyle>)->Result<(),String> {
            let element=document.Node(node);
            if element.Type()!=DOMNodeType::kElement {return Ok(());}
            let style=Style(document,pending,node).ok_or("unresolved mask content")?;
            let native=unsafe{style.native_style.Get().as_ref()}.ok_or("missing native mask style")?;
            if native.Display()==foundation::EDisplay::kNone {return Ok(());}
            if !style.own_generates_box {return Err("mask content without its own layout box".into());}
            let native_paint=layoutng::internal::boundary::native_paint_style::Export(native,None);
            let paint=&native_paint;
            let mut record_style=style.clone();*record_style.style.paint=native_paint.clone();canonical.insert(node,record_style);
            if !matches!(native.FillPaint().paint_type, SVGPaintType::kColor | SVGPaintType::kNone)
                || !matches!(native.StrokePaint().paint_type, SVGPaintType::kColor | SVGPaintType::kNone) {return Err("context/resource mask paint".into());}
            if paint.svg_non_scaling_stroke || native.PaintOrder()!=layoutng_style::style::computed_style_constants::EPaintOrder::kPaintOrderNormal || element.FindAttribute("pathLength").is_some() {return Err("mask non-scaling stroke, paint order or pathLength".into());}
            if !native.StrokeWidth().length().IsFixed() || !native.StrokeDashOffset().IsFixed()
                || unsafe {native.StrokeDashArray().as_ref()}.is_some_and(|a|a.iter().any(|l|!l.IsFixed())) {return Err("mask percentage/expression stroke metrics".into());}
            if native.EffectiveZoom()!=1.0 {return Err("zoomed mask content recording".into());}
            if paint.transform.is_some() || native.HasOffset() {return Err("transformed mask content recording".into());}
            if !paint.filters.is_empty() || paint.clip_path.is_some() || !paint.mask_images.is_empty() || !paint.backdrop_filters.is_empty() || paint.svg_fill_reference.is_some() || paint.svg_stroke_reference.is_some() || !native.MarkerStartResource().is_null() || !native.MarkerMidResource().is_null() || !native.MarkerEndResource().is_null() {return Err("mask content effect/paint-server collaborator".into());}
            if element.Namespace()!=DOMNamespace::kSVG {return Err("non-SVG mask content".into());}
            if element.Name()=="g" {
                if !paint.visible {return Err("hidden mask group with descendant visibility overrides".into());}
            } else {
                let shape=dom::layout_mapping::SVGMaskShape(document,node,native)?.ok_or_else(||format!("unsupported mask child {}",element.Name()))?;
                shapes.insert(node,shape);
            }
            for &child in element.Children() {collect(document,pending,child,shapes,canonical)?;}
            Ok(())
        }
        let mut shapes=HashMap::new();
        let mut canonical=pending.clone();
        for &child in document.Node(target).Children() {collect(document,pending,child,&mut shapes,&mut canonical)?;}
        PatternPrimitivesWithShapes(document,&canonical,target,&mut HashSet::new(),None,None,Some(&shapes)).map_err(|error|format!("mask content {error:?}"))
    })();
    match result {Ok(commands)=>graph.commands=commands,Err(error)=>graph.unsupported=Some(error)}
    Arc::new(graph)
}

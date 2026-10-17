//! Document-owned snapshots for local SVG <clipPath> resources.
use crate::{persistent_document::DOMNamespace, Child, Element, ParsedDocument as Document};
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use layoutng_assembly::internal::paint_input::{
    SvgClipGeometry, SvgClipLength, SvgClipPathChild, SvgClipPathGraph, SvgClipPathUnits,
    SvgClipResourceEntry, SvgClipResourceSnapshot, SvgClipResourceVersion, SvgClipText,
    SvgClipTextAnchor, SvgClipTextPath, SvgClipTransform, SvgClipTransformStep,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// Resource traversal borrows the live DOM. The static parser adapter still
// exposes Element values, so materialize only the nodes the graph reads, once.
// ID indexing and tree traversal never clone style/attributes for the arena.
enum ResourceElements<'a> {
    Parsed(&'a [Element]),
    Persistent(
        &'a crate::Document,
        std::cell::OnceCell<HashMap<usize, std::cell::OnceCell<Box<Element>>>>,
    ),
}
impl std::ops::Index<usize> for ResourceElements<'_> {
    type Output = Element;
    fn index(&self, index: usize) -> &Element {
        match self {
            Self::Parsed(elements) => &elements[index],
            Self::Persistent(document, cache) => cache.get_or_init(|| {
                // Native ID/type/style lookups borrow DOM directly. Allocate
                // adapter projection slots only when graph construction actually
                // reads an Element; HTML and SVG without resource graphs need none.
                let mut slots = HashMap::new();
                let mut pending = document.ActiveTreeScopes();
                while let Some(node) = pending.pop() {
                    slots.insert(node, std::cell::OnceCell::new());
                    pending.extend_from_slice(document.Node(node).Children());
                }
                slots
            })[&index]
                .get_or_init(|| Box::new(project_resource_element(document, index)))
                .as_ref(),
        }
    }
}
enum ResourceTexts<'a> {
    Parsed(&'a [String]),
    Persistent(&'a crate::Document),
}
impl std::ops::Index<usize> for ResourceTexts<'_> {
    type Output = str;
    fn index(&self, index: usize) -> &str {
        match self {
            Self::Parsed(texts) => &texts[index],
            Self::Persistent(document) => document.Node(index).Data(),
        }
    }
}
struct ResourceDocument<'a> {
    elements: ResourceElements<'a>,
    texts: ResourceTexts<'a>,
}
impl<'a> ResourceDocument<'a> {
    fn parsed(document: &'a Document) -> Self {
        Self {
            elements: ResourceElements::Parsed(&document.elements),
            texts: ResourceTexts::Parsed(&document.texts),
        }
    }
    fn persistent(document: &'a crate::Document) -> Self {
        Self {
            elements: ResourceElements::Persistent(document, std::cell::OnceCell::new()),
            texts: ResourceTexts::Persistent(document),
        }
    }
    fn children(&self, index: usize) -> Vec<usize> {
        match &self.elements {
            ResourceElements::Parsed(elements) => elements[index]
                .children
                .iter()
                .filter_map(|child| match child {
                    Child::Element(index) => Some(*index),
                    _ => None,
                })
                .collect(),
            ResourceElements::Persistent(document, _) => document
                .Node(index)
                .Children()
                .iter()
                .copied()
                .filter(|&child| {
                    document.Node(child).Type() == crate::persistent_document::DOMNodeType::kElement
                })
                .collect(),
        }
    }
    fn id(&self, index: usize) -> Option<&str> {
        match &self.elements {
            ResourceElements::Parsed(elements) => {
                attr(&elements[index], "id").or(elements[index].id.as_deref())
            }
            ResourceElements::Persistent(document, _) => document
                .Node(index)
                .FindAttribute("id")
                .map(|a| a.value.as_str()),
        }
    }
    fn is_clip(&self, index: usize) -> bool {
        match &self.elements {
            ResourceElements::Parsed(elements) => {
                elements[index].namespace == DOMNamespace::kSVG && elements[index].tag == "clipPath"
            }
            ResourceElements::Persistent(document, _) => document
                .Node(index)
                .IsElement("clipPath", DOMNamespace::kSVG),
        }
    }
    fn reference_scope(&self, index: usize) -> u64 {
        match &self.elements {
            ResourceElements::Persistent(document, _) => {
                document.SVGResourceScope(index).tree_scope_id
            }
            _ => 0,
        }
    }
    fn display(&self, index: usize) -> Option<&str> {
        if let ResourceElements::Persistent(document, _) = &self.elements {
            if let Some(style) = document
                .ResolvedStyleFor(index)
                .and_then(|resolved| unsafe { resolved.native_style.Get().as_ref() })
            {
                // Same computed override as project_resource_element, without
                // serializing font/geometry state just to inspect display.
                return Some(if style.Display() == foundation::EDisplay::kNone {
                    "none"
                } else {
                    "rendered"
                });
            }
        }
        style_value(&self.elements[index], "display")
    }
    fn parent(&self, index: usize) -> Option<usize> {
        match &self.elements {
            ResourceElements::Parsed(elements) => elements[index].parent,
            ResourceElements::Persistent(document, _) => document.Node(index).Parent(),
        }
    }
}

// A catalog is indexed by the first connected ID target in each TreeScope,
// before checking the target's element type. Linked use/textPath references
// resolve in the referencing element's originating scope, never arena order.
struct GraphDocument<'a> {
    document: ResourceDocument<'a>,
    reference_scopes: Vec<u64>,
    targets: HashMap<(u64, String), usize>,
    ordered_targets: Vec<(u64, String, usize, u64)>,
    computed_geometry: HashMap<usize, Result<Option<SvgClipGeometry>, String>>,
    computed_use_offsets: HashMap<usize, Result<(SvgClipLength, SvgClipLength), String>>,
    unavailable_targets: HashSet<usize>,
    mask_graphs: HashMap<usize, Arc<layoutng_assembly::internal::paint_input::SvgMaskGraph>>,
    use_instances: HashMap<usize, Result<Option<usize>, String>>,
}

impl<'a> std::ops::Deref for GraphDocument<'a> {
    type Target = ResourceDocument<'a>;
    fn deref(&self) -> &Self::Target {
        &self.document
    }
}

fn collect_tree_order(document: &ResourceDocument<'_>, index: usize, output: &mut Vec<usize>) {
    output.push(index);
    for child in document.children(index) {
        collect_tree_order(document, child, output);
    }
}

impl<'a> GraphDocument<'a> {
    fn new(document: ResourceDocument<'a>, reference_scopes: Vec<u64>) -> Self {
        Self {
            document,
            reference_scopes,
            targets: HashMap::new(),
            ordered_targets: Vec::new(),
            computed_geometry: HashMap::new(),
            computed_use_offsets: HashMap::new(),
            unavailable_targets: HashSet::new(),
            mask_graphs: HashMap::new(),
            use_instances: HashMap::new(),
        }
    }

    // SVGTreeScopeResources::ResourceForId / LocalSVGResource::TargetChanged:
    // only the first nonempty ID target in a scope owns resource availability.
    fn index_target(&mut self, index: usize, scope: u64, node_id: u64) -> bool {
        let Some(id) = self.document.id(index).filter(|id| !id.is_empty()) else {
            return false;
        };
        let key = (scope, id.to_owned());
        if let std::collections::hash_map::Entry::Vacant(entry) = self.targets.entry(key.clone()) {
            entry.insert(index);
            self.ordered_targets.push((scope, key.1, index, node_id));
            true
        } else {
            false
        }
    }

    fn snapshot(&self, document_id: u64) -> Arc<SvgClipResourceSnapshot> {
        Arc::new(SvgClipResourceSnapshot {
            version: SvgClipResourceVersion {
                document_id,
                revision: 0,
            },
            entries: self
                .ordered_targets
                .iter()
                .map(|(scope, id, index, node_id)| SvgClipResourceEntry {
                    scope_id: *scope,
                    target_node_id: *node_id,
                    id: id.clone(),
                    mask_graph: (!self.unavailable_targets.contains(index))
                        .then(|| self.mask_graphs.get(index).cloned())
                        .flatten(),
                    graph: (self.document.is_clip(*index)
                        && !self.unavailable_targets.contains(index))
                    .then(|| Arc::new(build_one(self, *index))),
                })
                .collect(),
        })
    }
}

fn attr<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}
fn number(element: &Element, name: &str, default: f64) -> f64 {
    attr(element, name)
        .and_then(|value| value.parse().ok())
        .filter(|v: &f64| v.is_finite())
        .unwrap_or(default)
}
fn numbers(value: &str) -> Result<Vec<f64>, String> {
    value
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| "invalid SVG clip number list".to_owned())
        })
        .collect()
}

fn svg_length(element: &Element, name: &str, default: f64) -> Result<SvgClipLength, String> {
    let Some(raw) = attr(element, name) else {
        return Ok(SvgClipLength {
            value: default,
            percentage: 0.0,
        });
    };
    let raw = raw.trim();
    let (number, percentage) = if let Some(value) = raw.strip_suffix('%') {
        (value, true)
    } else if let Some(value) = raw.strip_suffix("px") {
        (value, false)
    } else {
        (raw, false)
    };
    let value = number
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .ok_or_else(|| format!("unsupported SVG clip length {name}"))?;
    Ok(if percentage {
        SvgClipLength {
            value: 0.0,
            percentage: value / 100.0,
        }
    } else {
        SvgClipLength {
            value,
            percentage: 0.0,
        }
    })
}
fn svg_length_token(raw: &str) -> Result<SvgClipLength, String> {
    let raw = raw.trim();
    let (number, percentage) = raw.strip_suffix('%').map_or((raw, false), |v| (v, true));
    let value = number
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| "invalid SVG clip points list".to_owned())?;
    Ok(if percentage {
        SvgClipLength {
            value: 0.0,
            percentage: value / 100.0,
        }
    } else {
        SvgClipLength {
            value,
            percentage: 0.0,
        }
    })
}
// SVGLengthConversionData consumes canonical computed lengths. Preserve the
// unresolved linear percentage part until Paint knows the reference viewport;
// native fixed lengths carry effective zoom, while SVG resource units do not.
fn native_clip_length(value: &foundation::Length, zoom: f32) -> Result<SvgClipLength, String> {
    if value.IsAuto() {
        return Ok(SvgClipLength::default());
    }
    let pair = if value.IsFixed() || value.IsPercent() {
        value.GetPixelsAndPercent()
    } else if value.IsCalculated() && !value.GetCalculationValue().IsExpression() {
        value.GetCalculationValue().GetPixelsAndPercent()
    } else {
        return Err("unsupported computed SVG clip geometry length".into());
    };
    let length = SvgClipLength {
        value: if pair.has_explicit_pixels {
            f64::from(pair.pixels) / f64::from(zoom)
        } else {
            0.0
        },
        percentage: if pair.has_explicit_percent {
            f64::from(pair.percent) / 100.0
        } else {
            0.0
        },
    };
    if !length.value.is_finite() || !length.percentage.is_finite() {
        return Err("non-finite computed SVG clip geometry length".into());
    }
    Ok(length)
}

fn native_shape_geometry(
    tag: &str,
    style: &layoutng_style::style::computed_style::ComputedStyle,
) -> Result<Option<SvgClipGeometry>, String> {
    let zoom = style.EffectiveZoom().max(f32::MIN_POSITIVE);
    let length = |value: &foundation::Length| native_clip_length(value, zoom);
    Ok(match tag {
        "circle" => Some(SvgClipGeometry::kCircle {
            cx: length(style.Cx())?,
            cy: length(style.Cy())?,
            r: length(style.R())?,
        }),
        "ellipse" => Some(SvgClipGeometry::kEllipse {
            cx: length(style.Cx())?,
            cy: length(style.Cy())?,
            rx: length(if style.Rx().IsAuto() {
                style.Ry()
            } else {
                style.Rx()
            })?,
            ry: length(if style.Ry().IsAuto() {
                style.Rx()
            } else {
                style.Ry()
            })?,
        }),
        "rect" => Some(SvgClipGeometry::kRect {
            x: length(style.X())?,
            y: length(style.Y())?,
            width: length(style.Width())?,
            height: length(style.Height())?,
            rx: (!style.Rx().IsAuto())
                .then(|| length(style.Rx()))
                .transpose()?,
            ry: (!style.Ry().IsAuto())
                .then(|| length(style.Ry()))
                .transpose()?,
        }),
        _ => None,
    })
}

fn shape_geometry(element: &Element) -> Result<Option<SvgClipGeometry>, String> {
    Ok(match element.tag.as_str() {
        "circle" => Some(SvgClipGeometry::kCircle {
            cx: svg_length(element, "cx", 0.0)?,
            cy: svg_length(element, "cy", 0.0)?,
            r: svg_length(element, "r", 0.0)?,
        }),
        "ellipse" => Some(SvgClipGeometry::kEllipse {
            cx: svg_length(element, "cx", 0.0)?,
            cy: svg_length(element, "cy", 0.0)?,
            rx: svg_length(element, "rx", 0.0)?,
            ry: svg_length(element, "ry", 0.0)?,
        }),
        "rect" => Some(SvgClipGeometry::kRect {
            x: svg_length(element, "x", 0.0)?,
            y: svg_length(element, "y", 0.0)?,
            width: svg_length(element, "width", 0.0)?,
            height: svg_length(element, "height", 0.0)?,
            rx: attr(element, "rx")
                .map(|_| svg_length(element, "rx", 0.0))
                .transpose()?,
            ry: attr(element, "ry")
                .map(|_| svg_length(element, "ry", 0.0))
                .transpose()?,
        }),
        "polygon" | "polyline" => {
            let tokens = attr(element, "points")
                .unwrap_or("")
                .split(|c: char| c.is_ascii_whitespace() || c == ',')
                .filter(|part| !part.is_empty())
                .map(svg_length_token)
                .collect::<Result<Vec<_>, _>>()?;
            if tokens.len() % 2 != 0 {
                return Err("invalid SVG clip points list".into());
            }
            Some(SvgClipGeometry::kPolygon {
                points: tokens.chunks_exact(2).map(|p| (p[0], p[1])).collect(),
                close: element.tag == "polygon",
            })
        }
        "line" => Some(SvgClipGeometry::kLine {
            x1: svg_length(element, "x1", 0.0)?,
            y1: svg_length(element, "y1", 0.0)?,
            x2: svg_length(element, "x2", 0.0)?,
            y2: svg_length(element, "y2", 0.0)?,
        }),
        _ => None,
    })
}
fn multiply(a: SvgClipTransform, b: SvgClipTransform) -> SvgClipTransform {
    let [a0, a1, a2, a3, a4, a5] = a.values;
    let [b0, b1, b2, b3, b4, b5] = b.values;
    SvgClipTransform {
        values: [
            a0 * b0 + a2 * b1,
            a1 * b0 + a3 * b1,
            a0 * b2 + a2 * b3,
            a1 * b2 + a3 * b3,
            a0 * b4 + a2 * b5 + a4,
            a1 * b4 + a3 * b5 + a5,
        ],
        percentages: [
            a0 * b.percentages[0] + a2 * b.percentages[2] + a.percentages[0],
            a0 * b.percentages[1] + a2 * b.percentages[3] + a.percentages[1],
            a1 * b.percentages[0] + a3 * b.percentages[2] + a.percentages[2],
            a1 * b.percentages[1] + a3 * b.percentages[3] + a.percentages[3],
        ],
    }
}
fn transform(element: &Element) -> Result<SvgClipTransform, String> {
    if let Some(error) = attr(element, "data-computed-clip-transform-error") {
        return Err(error.to_owned());
    }
    // SVG2 maps the CSS transform property onto clipPath content.  The style
    // declaration wins over the presentation attribute through normal CSS
    // cascade; the persistent snapshot already contains computed style data.
    let Some(value) =
        attr(element, "data-computed-clip-transform").or_else(|| style_value(element, "transform"))
    else {
        return Ok(SvgClipTransform::default());
    };
    if let Some(arguments) = value
        .strip_prefix("matrix-affine(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let values = numbers(arguments)?;
        if values.len() != 10 {
            return Err("invalid computed CSS clip transform".into());
        }
        return Ok(SvgClipTransform {
            values: values[..6].try_into().unwrap(),
            percentages: values[6..].try_into().unwrap(),
        });
    }
    let mut rest = value.trim();
    let mut result = SvgClipTransform::default();
    while !rest.is_empty() {
        let open = rest
            .find('(')
            .ok_or_else(|| "invalid SVG clip transform".to_owned())?;
        let close = rest[open + 1..]
            .find(')')
            .map(|v| v + open + 1)
            .ok_or_else(|| "invalid SVG clip transform".to_owned())?;
        let name = rest[..open].trim();
        let values = numbers(&rest[open + 1..close])?;
        let next = match name {
            "matrix" if values.len() == 6 => SvgClipTransform {
                values: values.try_into().unwrap(),
                percentages: [0.0; 4],
            },
            "translate" if (1..=2).contains(&values.len()) => SvgClipTransform {
                values: [
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    values[0],
                    *values.get(1).unwrap_or(&0.0),
                ],
                percentages: [0.0; 4],
            },
            "scale" if (1..=2).contains(&values.len()) => SvgClipTransform {
                values: [
                    values[0],
                    0.0,
                    0.0,
                    *values.get(1).unwrap_or(&values[0]),
                    0.0,
                    0.0,
                ],
                percentages: [0.0; 4],
            },
            "rotate" if values.len() == 1 || values.len() == 3 => {
                let angle = values[0].to_radians();
                let (s, c) = angle.sin_cos();
                let rotation = SvgClipTransform {
                    values: [c, s, -s, c, 0.0, 0.0],
                    percentages: [0.0; 4],
                };
                if values.len() == 3 {
                    let to = SvgClipTransform {
                        values: [1.0, 0.0, 0.0, 1.0, values[1], values[2]],
                        percentages: [0.0; 4],
                    };
                    let from = SvgClipTransform {
                        values: [1.0, 0.0, 0.0, 1.0, -values[1], -values[2]],
                        percentages: [0.0; 4],
                    };
                    multiply(multiply(to, rotation), from)
                } else {
                    rotation
                }
            }
            "skewX" if values.len() == 1 => SvgClipTransform {
                values: [1.0, 0.0, values[0].to_radians().tan(), 1.0, 0.0, 0.0],
                percentages: [0.0; 4],
            },
            "skewY" if values.len() == 1 => SvgClipTransform {
                values: [1.0, values[0].to_radians().tan(), 0.0, 1.0, 0.0, 0.0],
                percentages: [0.0; 4],
            },
            _ => return Err(format!("unsupported SVG clip transform {name}")),
        };
        result = multiply(result, next);
        rest = rest[close + 1..].trim_start_matches(|c: char| c.is_ascii_whitespace() || c == ',');
    }
    Ok(result)
}
fn computed_css_transform(
    style: &layoutng_style::style::computed_style::ComputedStyle,
) -> Result<String, String> {
    use layoutng_style::style::style_transform::{
        ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
        ApplyTransformOrigin,
    };
    let zoom = style.EffectiveZoom().max(f32::MIN_POSITIVE);
    let apply = |width: f32, height: f32| {
        let mut matrix = foundation::gfx::Transform::default();
        let reference = foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(0.0, 0.0),
            foundation::gfx::SizeF::new(width * zoom, height * zoom),
        );
        style.ApplyTransformRectF::<()>(
            &mut matrix,
            None,
            &reference,
            ApplyTransformOperations::kIncludeTransformOperations,
            ApplyTransformOrigin::kIncludeTransformOrigin,
            ApplyMotionPath::kExcludeMotionPath,
            ApplyIndependentTransformProperties::kIncludeIndependentTransformProperties,
        );
        // TransformHelper computes against the zoomed reference box, then
        // removes effective zoom to publish SVG user-space geometry. Scaling
        // the input basis as well preserves percentage translation factors.
        if zoom != 1.0 {
            matrix.Zoom(1.0 / zoom);
        }
        matrix.GetColMajor()
    };
    let zero = apply(0.0, 0.0);
    let width = apply(1.0, 0.0);
    let height = apply(0.0, 1.0);
    for index in [0, 1, 4, 5] {
        if (zero[index] - width[index]).abs() > 1e-9 || (zero[index] - height[index]).abs() > 1e-9 {
            return Err("non-affine percentage CSS transform on SVG clip content".into());
        }
    }
    Ok(format!(
        "matrix-affine({},{},{},{},{},{},{},{},{},{})",
        zero[0],
        zero[1],
        zero[4],
        zero[5],
        zero[12],
        zero[13],
        width[12] - zero[12],
        height[12] - zero[12],
        width[13] - zero[13],
        height[13] - zero[13],
    ))
}
fn style_value<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
    let computed_name = match name {
        "clip-rule" => Some("data-computed-clip-rule"),
        "display" => Some("data-computed-display"),
        "visibility" => Some("data-computed-visibility"),
        "clip-path" => Some("data-computed-clip-path"),
        _ => None,
    };
    if let Some(value) = computed_name.and_then(|name| attr(element, name)) {
        return Some(value);
    }
    attr(element, "style")
        .or(element.inline_style.as_deref())
        .and_then(|style| {
            style.split(';').find_map(|item| {
                let (key, value) = item.split_once(':')?;
                (key.trim().eq_ignore_ascii_case(name)).then_some(value.trim())
            })
        })
        .or_else(|| attr(element, name))
}

fn line_path(points: &[Offset], close: bool) -> Vec<PaintPathCommand> {
    let mut output = Vec::new();
    if let Some(first) = points.first() {
        output.push(PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point: *first,
            ..Default::default()
        });
        for point in &points[1..] {
            output.push(PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: *point,
                ..Default::default()
            })
        }
        if close {
            output.push(PaintPathCommand {
                verb: PaintPathVerb::kClose,
                ..Default::default()
            })
        }
    }
    output
}
fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<PaintPathCommand> {
    if rx <= 0.0 || ry <= 0.0 {
        return Vec::new();
    }
    let w = std::f64::consts::FRAC_1_SQRT_2;
    let points = [
        (cx + rx, cy),
        (cx, cy + ry),
        (cx - rx, cy),
        (cx, cy - ry),
        (cx + rx, cy),
    ];
    let controls = [
        (cx + rx, cy + ry),
        (cx - rx, cy + ry),
        (cx - rx, cy - ry),
        (cx + rx, cy - ry),
    ];
    let mut out = vec![PaintPathCommand {
        verb: PaintPathVerb::kMoveTo,
        point: Offset {
            x: points[0].0,
            y: points[0].1,
        },
        ..Default::default()
    }];
    for i in 0..4 {
        out.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset {
                x: controls[i].0,
                y: controls[i].1,
            },
            point: Offset {
                x: points[i + 1].0,
                y: points[i + 1].1,
            },
            conic_weight: w,
            ..Default::default()
        })
    }
    out.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..Default::default()
    });
    out
}
fn rect_path(element: &Element) -> Vec<PaintPathCommand> {
    let x = number(element, "x", 0.0);
    let y = number(element, "y", 0.0);
    let width = number(element, "width", 0.0).max(0.0);
    let height = number(element, "height", 0.0).max(0.0);
    let rx_attr = attr(element, "rx");
    let ry_attr = attr(element, "ry");
    let mut rx = number(element, "rx", 0.0).max(0.0);
    let mut ry = number(element, "ry", 0.0).max(0.0);
    if rx_attr.is_none() {
        rx = ry
    }
    if ry_attr.is_none() {
        ry = rx
    }
    rx = rx.min(width * 0.5);
    ry = ry.min(height * 0.5);
    if rx == 0.0 || ry == 0.0 {
        return line_path(
            &[
                Offset { x, y },
                Offset { x: x + width, y },
                Offset {
                    x: x + width,
                    y: y + height,
                },
                Offset { x, y: y + height },
            ],
            true,
        );
    }
    let mut out = vec![PaintPathCommand {
        verb: PaintPathVerb::kMoveTo,
        point: Offset { x: x + rx, y },
        ..Default::default()
    }];
    let w = std::f64::consts::FRAC_1_SQRT_2;
    let line = |out: &mut Vec<PaintPathCommand>, x, y| {
        out.push(PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset { x, y },
            ..Default::default()
        })
    };
    let curve = |out: &mut Vec<PaintPathCommand>, cx, cy, x, y| {
        out.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset { x: cx, y: cy },
            point: Offset { x, y },
            conic_weight: w,
            ..Default::default()
        })
    };
    line(&mut out, x + width - rx, y);
    curve(&mut out, x + width, y, x + width, y + ry);
    line(&mut out, x + width, y + height - ry);
    curve(&mut out, x + width, y + height, x + width - rx, y + height);
    line(&mut out, x + rx, y + height);
    curve(&mut out, x, y + height, x, y + height - ry);
    line(&mut out, x, y + ry);
    curve(&mut out, x, y, x + rx, y);
    out.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..Default::default()
    });
    out
}
fn shape_commands(element: &Element) -> Result<Vec<PaintPathCommand>, String> {
    let numeric_attributes: &[&str] = match element.tag.as_str() {
        "circle" => &["cx", "cy", "r"],
        "ellipse" => &["cx", "cy", "rx", "ry"],
        "rect" => &["x", "y", "width", "height", "rx", "ry"],
        "line" => &["x1", "y1", "x2", "y2"],
        _ => &[],
    };
    if let Some(name) = numeric_attributes.iter().find(|name| {
        attr(element, name).is_some_and(|value| {
            value
                .parse::<f64>()
                .ok()
                .is_none_or(|value| !value.is_finite())
        })
    }) {
        return Err(format!("unsupported SVG clip length {name}"));
    }
    Ok(match element.tag.as_str() {
        "path" => {
            crate::svg_path_parser::ParseSVGPathDefault(attr(element, "d").unwrap_or(""))
                .ok_or_else(|| "invalid SVG clip path data".to_owned())?
                .commands
        }
        "circle" => {
            let r = number(element, "r", 0.0).max(0.0);
            ellipse(number(element, "cx", 0.0), number(element, "cy", 0.0), r, r)
        }
        "ellipse" => ellipse(
            number(element, "cx", 0.0),
            number(element, "cy", 0.0),
            number(element, "rx", 0.0).max(0.0),
            number(element, "ry", 0.0).max(0.0),
        ),
        "rect" => rect_path(element),
        "polygon" | "polyline" => {
            let values = numbers(attr(element, "points").unwrap_or(""))?;
            if values.len() % 2 != 0 {
                return Err("invalid SVG clip points list".into());
            }
            let points = values
                .chunks_exact(2)
                .map(|p| Offset { x: p[0], y: p[1] })
                .collect::<Vec<_>>();
            line_path(&points, element.tag == "polygon")
        }
        "line" => line_path(
            &[
                Offset {
                    x: number(element, "x1", 0.0),
                    y: number(element, "y1", 0.0),
                },
                Offset {
                    x: number(element, "x2", 0.0),
                    y: number(element, "y2", 0.0),
                },
            ],
            false,
        ),
        tag => return Err(format!("unsupported SVG clip child {tag}")),
    })
}
fn find_id(document: &GraphDocument<'_>, reference: usize, id: &str) -> Option<usize> {
    document
        .targets
        .get(&(
            document
                .reference_scopes
                .get(reference)
                .copied()
                .unwrap_or_else(|| document.reference_scope(reference)),
            id.to_owned(),
        ))
        .copied()
}
fn length_list(element: &Element, name: &str) -> Result<Vec<SvgClipLength>, String> {
    attr(element, name)
        .unwrap_or("")
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|token| !token.is_empty())
        .map(svg_length_token)
        .collect()
}
fn number_list(element: &Element, name: &str) -> Result<Vec<f64>, String> {
    numbers(attr(element, name).unwrap_or(""))
}
fn normalize_svg_text(text: &str, preserve: bool) -> String {
    if preserve {
        text.to_owned()
    } else {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}
fn text_path(
    document: &GraphDocument<'_>,
    reference: usize,
    element: &Element,
) -> Result<Option<SvgClipTextPath>, String> {
    if element.tag != "textPath" {
        return Ok(None);
    }
    let href = attr(element, "href")
        .or_else(|| attr(element, "xlink:href"))
        .ok_or_else(|| "SVG clip textPath missing href".to_owned())?;
    let id = href
        .strip_prefix('#')
        .ok_or_else(|| "external SVG clip textPath IRI".to_owned())?;
    let index = find_id(document, reference, id)
        .ok_or_else(|| "missing SVG clip textPath target".to_owned())?;
    let path = &document.elements[index];
    if path.namespace != DOMNamespace::kSVG || path.tag != "path" {
        return Err("SVG clip textPath target is not a path".into());
    }
    let parsed = crate::svg_path_parser::ParseSVGPathDefault(attr(path, "d").unwrap_or(""))
        .ok_or_else(|| "invalid SVG clip textPath data".to_owned())?;
    let matrix = transform(path)?;
    let points = parsed
        .flattened_points
        .into_iter()
        .map(|point| Offset {
            x: matrix.values[0] * point.x + matrix.values[2] * point.y + matrix.values[4],
            y: matrix.values[1] * point.x + matrix.values[3] * point.y + matrix.values[5],
        })
        .collect::<Vec<_>>();
    if points.len() < 2 {
        return Err("empty SVG clip textPath geometry".into());
    }
    Ok(Some(SvgClipTextPath {
        points,
        start_offset: svg_length(element, "startOffset", 0.0)?,
        reverse_direction: attr(element, "side") == Some("right"),
    }))
}
fn text_semantics(
    document: &GraphDocument<'_>,
    reference: usize,
    element: &Element,
    text: String,
) -> Result<SvgClipText, String> {
    let font_families = attr(element, "data-computed-font-family")
        .or_else(|| style_value(element, "font-family"))
        .unwrap_or("serif")
        .split(',')
        .map(|value| value.trim().trim_matches(['\'', '"']).to_owned())
        .filter(|value| !value.is_empty())
        .collect();
    let parsed = |name: &str, default: f64| {
        attr(element, name)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .unwrap_or(default)
    };
    Ok(SvgClipText {
        text,
        x: length_list(element, "x")?,
        y: length_list(element, "y")?,
        dx: length_list(element, "dx")?,
        dy: length_list(element, "dy")?,
        rotate: number_list(element, "rotate")?,
        font_families,
        font_size: parsed("data-computed-font-size", 16.0),
        font_weight: parsed("data-computed-font-weight", 400.0),
        italic: attr(element, "data-computed-font-italic") == Some("true"),
        letter_spacing: parsed("data-computed-letter-spacing", 0.0),
        word_spacing: parsed("data-computed-word-spacing", 0.0),
        rtl: attr(element, "data-computed-direction") == Some("rtl"),
        language: attr(element, "data-computed-language")
            .unwrap_or("en")
            .to_owned(),
        anchor: match attr(element, "data-computed-text-anchor")
            .or_else(|| style_value(element, "text-anchor"))
        {
            Some("middle") => SvgClipTextAnchor::kMiddle,
            Some("end") => SvgClipTextAnchor::kEnd,
            _ => SvgClipTextAnchor::kStart,
        },
        text_path: text_path(document, reference, element)?,
    })
}
fn collect_child(
    document: &GraphDocument<'_>,
    index: usize,
    stack: &mut HashSet<usize>,
    prefix: Vec<SvgClipTransformStep>,
    inherited_even_odd: bool,
    inherited_visible: bool,
    output: &mut Vec<SvgClipPathChild>,
) -> Result<(), String> {
    if !stack.insert(index) {
        return Err("recursive SVG clipPath use".into());
    }
    let element = &document.elements[index];
    if element.namespace != DOMNamespace::kSVG {
        return Err("SVG clip reference targets a non-SVG element".into());
    }
    if style_value(element, "display") == Some("none") {
        stack.remove(&index);
        return Ok(());
    }
    if style_value(element, "clip-path").is_some_and(|value| value != "none") {
        return Err("nested clip-path on clip content is unsupported".into());
    }
    let own = transform(element)?;
    let mut combined = prefix;
    combined.push(SvgClipTransformStep::kMatrix(own));
    let even_odd = match style_value(element, "clip-rule") {
        Some(value) if value.eq_ignore_ascii_case("evenodd") => true,
        Some(value) if value.eq_ignore_ascii_case("nonzero") => false,
        _ => inherited_even_odd,
    };
    let visible = match style_value(element, "visibility") {
        Some(value) if value.eq_ignore_ascii_case("hidden") => false,
        Some(value) if value.eq_ignore_ascii_case("collapse") => false,
        Some(value) if value.eq_ignore_ascii_case("visible") => true,
        _ => inherited_visible,
    };
    if element.tag == "use" {
        // The persistent document supplies the actual cloned target after
        // its instance cascade and href precedence. Only the legacy static
        // fixture adapter lacks the retained instance lifecycle.
        let target = if let Some(instance) = document.use_instances.get(&index) {
            instance.clone()?
        } else {
            let Some(href) = attr(element, "href").or_else(|| attr(element, "xlink:href")) else {
                stack.remove(&index);
                return Ok(());
            };
            let Some(id) = href.strip_prefix('#') else {
                return Err("external SVG clip use IRI".into());
            };
            find_id(document, index, id)
        };
        let Some(target) = target else {
            stack.remove(&index);
            return Ok(());
        };
        let target_element = &document.elements[target];
        if target_element.namespace != DOMNamespace::kSVG
            || !matches!(
                target_element.tag.as_str(),
                "path" | "rect" | "circle" | "ellipse" | "polygon" | "polyline" | "text"
            )
        {
            stack.remove(&index);
            return Ok(());
        }
        let (use_x, use_y) = match document.computed_use_offsets.get(&index) {
            Some(offsets) => offsets.clone()?,
            None => (
                svg_length(element, "x", 0.0)?,
                svg_length(element, "y", 0.0)?,
            ),
        };
        let mut translated = combined;
        translated.push(SvgClipTransformStep::kMatrix(SvgClipTransform {
            values: [1.0, 0.0, 0.0, 1.0, use_x.value, use_y.value],
            percentages: [use_x.percentage, 0.0, 0.0, use_y.percentage],
        }));
        collect_child(
            document, target, stack, translated, even_odd, visible, output,
        )?
    } else if matches!(element.tag.as_str(), "g" | "a" | "symbol" | "svg") {
        // LayoutSVGResourceClipper::DetermineClipStrategy admits only direct
        // shapes/text and restricted <use> targets, not group descendants.
    } else if matches!(element.tag.as_str(), "text" | "tspan" | "textPath") {
        let preserve = attr(element, "xml:space") == Some("preserve");
        for child in &element.children {
            match child {
                Child::Text(text_index) => {
                    let text = normalize_svg_text(&document.texts[*text_index], preserve);
                    if visible && !text.is_empty() {
                        output.push(SvgClipPathChild {
                            commands: Vec::new(),
                            geometry: None,
                            text: Some(text_semantics(document, index, element, text)?),
                            even_odd: false,
                            transforms: combined.clone(),
                        });
                    }
                }
                Child::Element(index) => collect_child(
                    document,
                    *index,
                    stack,
                    combined.clone(),
                    even_odd,
                    visible,
                    output,
                )?,
            }
        }
    } else {
        let geometry = match document.computed_geometry.get(&index) {
            Some(geometry) => geometry.clone()?,
            None => shape_geometry(element)?,
        };
        let commands = if geometry.is_some() {
            Vec::new()
        } else {
            shape_commands(element)?
        };
        if visible && (!commands.is_empty() || geometry.is_some()) {
            output.push(SvgClipPathChild {
                commands,
                geometry,
                text: None,
                even_odd,
                transforms: combined,
            })
        }
    }
    stack.remove(&index);
    Ok(())
}
fn build_one(document: &GraphDocument<'_>, index: usize) -> SvgClipPathGraph {
    let element = &document.elements[index];
    let (resource_transform, transform_error) = match transform(element) {
        Ok(value) => (value, None),
        Err(error) => (SvgClipTransform::default(), Some(error)),
    };
    let mut graph = SvgClipPathGraph {
        id: attr(element, "id")
            .or(element.id.as_deref())
            .unwrap_or("")
            .to_owned(),
        units: if attr(element, "clipPathUnits") == Some("objectBoundingBox") {
            SvgClipPathUnits::kObjectBoundingBox
        } else {
            SvgClipPathUnits::kUserSpaceOnUse
        },
        transform: resource_transform,
        children: Vec::new(),
        unsupported: transform_error.or_else(|| {
            style_value(element, "clip-path")
                .is_some_and(|value| value != "none")
                .then(|| "nested clip-path on clipPath resource is unsupported".into())
        }),
    };
    let local = element
        .children
        .iter()
        .any(|child| matches!(child, Child::Element(_)));
    // SVGClipPathElement owns clipPathUnits and transform; unlike gradients,
    // it has no href-based content inheritance.
    if local {
        let inherited_even_odd = matches!(
            style_value(element, "clip-rule"),
            Some(value) if value.eq_ignore_ascii_case("evenodd")
        );
        for child in &element.children {
            let Child::Element(child) = child else {
                continue;
            };
            let content = &document.elements[*child];
            if content.namespace != DOMNamespace::kSVG
                || !matches!(
                    content.tag.as_str(),
                    "circle"
                        | "ellipse"
                        | "rect"
                        | "polygon"
                        | "polyline"
                        | "line"
                        | "path"
                        | "text"
                        | "use"
                )
            {
                continue;
            }
            if let Err(error) = collect_child(
                document,
                *child,
                &mut HashSet::new(),
                Vec::new(),
                inherited_even_odd,
                true,
                &mut graph.children,
            ) {
                graph.unsupported.get_or_insert(error);
                break;
            }
        }
    }
    graph
}
fn raw_resource_display_suppressed(document: &ResourceDocument<'_>, index: usize) -> bool {
    if let ResourceElements::Persistent(owner, _) = &document.elements {
        // Element::RecalcOwnStyle has already applied parent-style absence and
        // ShouldStoreComputedStyle, including SVG's hidden-container exceptions.
        // A clip/mask below display:none <g> still has a resource layout object;
        // physical box suppression on that ancestor is not resource suppression.
        // The static DOM adapter can also be used before its first style
        // lifecycle; retain its existing raw-input projection until style is
        // ready. Page always updates style before publishing this catalog.
        if !owner.StyleState().all_dirty {
            return owner.ResolvedStyleFor(index).is_none_or(|resolved| {
                !resolved.own_generates_box
                    || unsafe { resolved.native_style.Get().as_ref() }
                        .is_some_and(|style| style.IsEnsuredInDisplayNone())
            });
        }
    }
    if document.display(index) == Some("contents") {
        return true;
    }
    let mut ancestor = Some(index);
    while let Some(node) = ancestor {
        if document.display(node) == Some("none") {
            return true;
        }
        ancestor = document.parent(node);
    }
    false
}

pub(super) fn build(document: &Document) -> Arc<SvgClipResourceSnapshot> {
    let mut catalog = GraphDocument::new(
        ResourceDocument::parsed(document),
        vec![0; document.elements.len()],
    );
    let mut order = Vec::new();
    if let Some(root) = document.root {
        collect_tree_order(&catalog.document, root, &mut order);
    } else {
        for (index, element) in document.elements.iter().enumerate() {
            if element.parent.is_none() {
                collect_tree_order(&catalog.document, index, &mut order);
            }
        }
    }
    for index in order {
        if !catalog.index_target(index, 0, index as u64 + 1) {
            continue;
        }
        let has_svg_parent = document.elements[index]
            .parent
            .is_some_and(|parent| document.elements[parent].namespace == DOMNamespace::kSVG);
        if !has_svg_parent || raw_resource_display_suppressed(&catalog.document, index) {
            catalog.unavailable_targets.insert(index);
        }
    }
    catalog.snapshot(0)
}

fn project_resource_element(document: &crate::Document, index: usize) -> Element {
    use crate::persistent_document::DOMNodeType;
    let _trace = browser_tracing::span("dom", "SVGResource.ProjectElement");
    let node = document.Node(index);
    let mut attributes = node
        .Attributes()
        .iter()
        .filter(|attribute| !attribute.local_name.starts_with("data-computed-"))
        .map(|attribute| (attribute.local_name.clone(), attribute.value.clone()))
        .collect::<Vec<_>>();
    if let Some(native) = document
        .ResolvedStyleFor(index)
        .and_then(|resolved| unsafe { resolved.native_style.Get().as_ref() })
    {
        {
            match computed_css_transform(native) {
                Ok(transform) => {
                    attributes.push(("data-computed-clip-transform".into(), transform))
                }
                Err(error) => attributes.push(("data-computed-clip-transform-error".into(), error)),
            }
        }
        let zoom = native.EffectiveZoom().max(f32::MIN_POSITIVE);
        let font = native.GetFontDescription();
        attributes.push(("data-computed-font-family".into(), font.Family().ToString()));
        attributes.push((
            "data-computed-font-size".into(),
            (native.ComputedFontSize() / zoom).to_string(),
        ));
        attributes.push((
            "data-computed-font-weight".into(),
            font.Weight().ToFloat().to_string(),
        ));
        attributes.push((
            "data-computed-font-italic".into(),
            (font.Style().ToFloat() != 0.0).to_string(),
        ));
        attributes.push((
            "data-computed-letter-spacing".into(),
            (native.LetterSpacing() / zoom).to_string(),
        ));
        attributes.push((
            "data-computed-word-spacing".into(),
            (native.WordSpacing() / zoom).to_string(),
        ));
        attributes.push((
            "data-computed-direction".into(),
            if native.IsLeftToRightDirection() {
                "ltr"
            } else {
                "rtl"
            }
            .into(),
        ));
        attributes.push(("data-computed-language".into(), native.Locale().Utf8()));
        attributes.push((
            "data-computed-text-anchor".into(),
            match native.TextAnchor() {
                foundation::ETextAnchor::kStart => "start",
                foundation::ETextAnchor::kMiddle => "middle",
                foundation::ETextAnchor::kEnd => "end",
            }
            .into(),
        ));
        // Resource descendants retain native style/layout ownership. Their
        // final cascade values are authoritative over raw inline text in
        // this immutable geometry snapshot.
        attributes
            .retain(|(name, _)| !matches!(name.as_str(), "clip-rule" | "display" | "visibility"));
        attributes.push((
            "data-computed-clip-rule".into(),
            if native.ClipRule() == foundation::WindRule::RULE_EVENODD {
                "evenodd"
            } else {
                "nonzero"
            }
            .into(),
        ));
        attributes.push((
            "data-computed-display".into(),
            if native.Display() == foundation::EDisplay::kNone {
                "none"
            } else {
                "rendered"
            }
            .into(),
        ));
        attributes.push((
            "data-computed-clip-path".into(),
            if native.ClipPath().is_some() {
                "reference-or-shape"
            } else {
                "none"
            }
            .into(),
        ));
        attributes.push((
            "data-computed-visibility".into(),
            match native.Visibility() {
                foundation::EVisibility::kVisible => "visible",
                foundation::EVisibility::kHidden => "hidden",
                foundation::EVisibility::kCollapse => "collapse",
            }
            .into(),
        ));
    }
    let children = node
        .Children()
        .iter()
        .filter_map(|child| match document.Node(*child).Type() {
            DOMNodeType::kElement => Some(Child::Element(*child)),
            DOMNodeType::kText => Some(Child::Text(*child)),
            _ => None,
        })
        .collect();
    Element {
        tag: node.Name().to_owned(),
        namespace: node.Namespace(),
        id: node.FindAttribute("id").map(|a| a.value.clone()),
        classes: Vec::new(),
        inline_style: node.FindAttribute("style").map(|a| a.value.clone()),
        attributes,
        parent: node.Parent(),
        children,
    }
}

/// Project the retained DOM into the same immutable resource snapshot used by
/// the static adapter. Arena indices stay unchanged so local fragment links
/// and mutation-driven rebuilds keep document identity semantics.
pub(super) fn build_persistent(document: &crate::Document) -> Arc<SvgClipResourceSnapshot> {
    use crate::persistent_document::DOMNodeType;

    let mut trace = browser_tracing::span("dom", "DOM.BuildSVGClipResources");
    let mut catalog = GraphDocument::new(ResourceDocument::persistent(document), Vec::new());
    // cpp: LocalSVGResource / TreeScopeResources resolve connected ID targets
    // in their TreeScope. Detached arena entries cannot contribute a resource.
    // Keep one ordered inventory for both resource inputs and ID indexing;
    // repeated layout queries must not scan the historical allocation arena.
    let scoped_order = document
        .ActiveTreeScopes()
        .into_iter()
        .map(|scope| {
            let mut order = Vec::new();
            collect_tree_order(&catalog.document, scope, &mut order);
            (scope, order)
        })
        .collect::<Vec<_>>();
    trace.set("arena_nodes", document.NodeCount() as f64);
    trace.set(
        "connected_nodes",
        scoped_order
            .iter()
            .map(|(_, order)| order.len())
            .sum::<usize>() as f64,
    );
    let mut svg_nodes = 0;
    for &index in scoped_order.iter().flat_map(|(_, order)| order.iter()) {
        let node = document.Node(index);
        if node.Namespace() != DOMNamespace::kSVG {
            continue;
        }
        svg_nodes += 1;
        if let Some(native) = document
            .ResolvedStyleFor(index)
            .and_then(|s| unsafe { s.native_style.Get().as_ref() })
        {
            if matches!(node.Name(), "rect" | "circle" | "ellipse") {
                catalog
                    .computed_geometry
                    .insert(index, native_shape_geometry(node.Name(), native));
            }
            if node.Name() == "use" {
                let zoom = native.EffectiveZoom().max(f32::MIN_POSITIVE);
                catalog.computed_use_offsets.insert(
                    index,
                    native_clip_length(native.X(), zoom)
                        .and_then(|x| native_clip_length(native.Y(), zoom).map(|y| (x, y))),
                );
            }
        }
        if node.IsElement("mask", DOMNamespace::kSVG) {
            if let Some(graph) = document
                .ResolvedStyleFor(index)
                .and_then(|style| style.style.paint.svg_mask_resource.clone())
            {
                catalog.mask_graphs.insert(index, graph);
            }
        }
        if document.Node(index).IsElement("use", DOMNamespace::kSVG) {
            use crate::persistent_document::SVGUseInstanceStatus as Status;
            let target = match document.SVGUseInstanceStatus(index) {
                Some(
                    Status::Ready
                    | Status::MissingTarget
                    | Status::DisallowedTarget
                    | Status::CyclicReference,
                ) => Ok(document.SVGUseInstanceRoot(index)),
                Some(status) => Err(format!("unsupported SVG clip use instance: {status:?}")),
                None => Err("SVG clip use instance lifecycle was not updated".into()),
            };
            catalog.use_instances.insert(index, target);
        }
    }

    trace.set("svg_nodes", svg_nodes as f64);
    for (scope, order) in scoped_order {
        let scope_id = document.Node(scope).Id();
        for index in order {
            // This inventory is already connected and scoped. Availability
            // belongs to its first ID target, not every element in the DOM.
            if document.Node(index).Type() == DOMNodeType::kElement
                && catalog.index_target(index, scope_id, document.Node(index).Id())
            {
                // SVGElement::LayoutObjectIsNeeded requires an SVG flat-tree
                // parent and an attached own box. visibility:hidden does not
                // suppress the resource container's existence.
                let has_svg_parent = document
                    .FlatTreeParentElement(index)
                    .is_some_and(|parent| document.Node(parent).Namespace() == DOMNamespace::kSVG);
                if !has_svg_parent || raw_resource_display_suppressed(&catalog.document, index) {
                    catalog.unavailable_targets.insert(index);
                }
            }
        }
    }
    trace.set("resource_targets", catalog.ordered_targets.len() as f64);
    trace.set(
        "unavailable_targets",
        catalog.unavailable_targets.len() as f64,
    );
    let snapshot = catalog.snapshot(document.SVGResourceScope(document.Root()).document_id);
    if let ResourceElements::Persistent(_, cache) = &catalog.document.elements {
        trace.set(
            "projection_slots",
            cache.get().map_or(0, |slots| slots.len()) as f64,
        );
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistent_document::{DOMAttribute, DOMShadowRootMode};

    fn attr_value(name: &str, value: &str) -> DOMAttribute {
        DOMAttribute {
            local_name: name.into(),
            value: value.into(),
            ..Default::default()
        }
    }

    fn add(
        document: &mut crate::Document,
        parent: usize,
        name: &str,
        attrs: &[(&str, &str)],
    ) -> usize {
        let node = document.CreateElement(
            DOMNamespace::kSVG,
            name.into(),
            attrs
                .iter()
                .map(|(name, value)| attr_value(name, value))
                .collect(),
        );
        document.AppendChild(parent, node);
        node
    }

    fn root(owner: &mut crate::DOM) -> usize {
        let document = owner.GetDocumentMut();
        let parent = document.Root();
        add(document, parent, "svg", &[])
    }

    fn clip_graph(
        snapshot: &SvgClipResourceSnapshot,
        scope: u64,
        id: &str,
    ) -> Arc<SvgClipPathGraph> {
        snapshot
            .entries
            .iter()
            .find(|entry| entry.scope_id == scope && entry.id == id)
            .and_then(|entry| entry.graph.clone())
            .expect("clip graph")
    }

    fn rect_width(graph: &SvgClipPathGraph) -> f64 {
        match graph
            .children
            .first()
            .and_then(|child| child.geometry.as_ref())
        {
            Some(SvgClipGeometry::kRect { width, .. }) => width.value,
            _ => panic!("expected rectangle clip"),
        }
    }

    #[test]
    fn svg_clip_catalog_selects_first_connected_tree_order_target_before_kind() {
        let mut owner = crate::DOM::new();
        let root = root(&mut owner);
        let document = owner.GetDocumentMut();
        let clip = document.CreateElement(
            DOMNamespace::kSVG,
            "clipPath".into(),
            vec![attr_value("id", "clip")],
        );
        let wrong_kind = add(document, root, "rect", &[("id", "clip")]);
        document.AppendChild(root, clip);
        add(document, clip, "rect", &[("width", "10"), ("height", "10")]);
        let snapshot = build_persistent(document);
        let entry = snapshot
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap();
        assert_eq!(entry.target_node_id, document.Node(wrong_kind).Id());
        assert!(entry.graph.is_none());
        document.Remove(wrong_kind);
        let snapshot = build_persistent(document);
        let entry = snapshot
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap();
        assert_eq!(entry.target_node_id, document.Node(clip).Id());
        assert_eq!(rect_width(entry.graph.as_ref().unwrap()), 10.0);
        document.Remove(clip);
        assert!(build_persistent(document)
            .entries
            .iter()
            .all(|entry| entry.id != "clip"));
    }

    #[test]
    fn svg_clip_catalog_resolves_use_in_its_connected_tree_scope() {
        let mut owner = crate::DOM::new();
        let root = root(&mut owner);
        let document = owner.GetDocumentMut();
        add(
            document,
            root,
            "rect",
            &[("id", "shape"), ("width", "10"), ("height", "10")],
        );
        let clip = add(document, root, "clipPath", &[("id", "clip")]);
        add(document, clip, "use", &[("href", "#shape")]);
        let host = document.CreateElementDefault(DOMNamespace::kHTML, "x-host".into());
        document.AppendChild(root, host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let shadow_svg = add(document, shadow, "svg", &[]);
        add(
            document,
            shadow_svg,
            "rect",
            &[("id", "shape"), ("width", "20"), ("height", "10")],
        );
        let shadow_clip = add(document, shadow_svg, "clipPath", &[("id", "clip")]);
        add(document, shadow_clip, "use", &[("href", "#shape")]);
        document.UpdateSVGUseInstances();
        let snapshot = build_persistent(document);
        let document_scope = document.SVGResourceScope(root).tree_scope_id;
        let shadow_scope = document.SVGResourceScope(shadow_clip).tree_scope_id;
        assert_ne!(document_scope, shadow_scope);
        assert_eq!(
            rect_width(&clip_graph(&snapshot, document_scope, "clip")),
            10.0
        );
        assert_eq!(
            rect_width(&clip_graph(&snapshot, shadow_scope, "clip")),
            20.0
        );
        document.Remove(host);
        assert!(build_persistent(document)
            .entries
            .iter()
            .all(|entry| entry.scope_id != shadow_scope));
    }

    #[test]
    fn svg_clip_catalog_ignores_groups_and_nonexistent_href_inheritance() {
        let mut owner = crate::DOM::new();
        let root = root(&mut owner);
        let document = owner.GetDocumentMut();
        let base = add(document, root, "clipPath", &[("id", "base")]);
        add(document, base, "rect", &[("width", "10"), ("height", "10")]);
        let clip = add(
            document,
            root,
            "clipPath",
            &[("id", "clip"), ("href", "#base")],
        );
        let group = add(document, clip, "g", &[]);
        add(
            document,
            group,
            "rect",
            &[("width", "20"), ("height", "20")],
        );
        add(document, clip, "use", &[("href", "#missing")]);
        document.UpdateSVGUseInstances();
        let graph = clip_graph(
            &build_persistent(document),
            document.SVGResourceScope(root).tree_scope_id,
            "clip",
        );
        assert!(graph.children.is_empty());
        assert!(graph.unsupported.is_none());
    }

    #[test]
    fn svg_clip_cache_rejects_stale_inputs_and_keeps_old_frames_immutable() {
        let mut owner = crate::DOM::new();
        let root = root(&mut owner);
        let document = owner.GetDocumentMut();
        let clip = add(document, root, "clipPath", &[("id", "clip")]);
        let rect = add(document, clip, "rect", &[("width", "10"), ("height", "10")]);
        assert!(owner.UpdateSVGResources());
        let old = owner.GetDocument().SVGResources().unwrap().clone();
        let scope = owner.GetDocument().SVGResourceScope(root).tree_scope_id;
        owner.DidCommitPaint();
        owner
            .GetDocumentMut()
            .SetAttribute(rect, attr_value("width", "20"));
        assert!(owner.GetDocument().SVGResources().is_none());
        assert!(owner.UpdateSVGResources());
        let current = owner.GetDocument().SVGResources().unwrap().clone();
        assert_eq!(current.version.revision, old.version.revision + 1);
        assert_eq!(rect_width(&clip_graph(&old, scope, "clip")), 10.0);
        assert_eq!(rect_width(&clip_graph(&current, scope, "clip")), 20.0);
        owner
            .GetDocumentMut()
            .SetAttribute(rect, attr_value("data-unused", "value"));
        assert!(!owner.UpdateSVGResources());
        assert!(Arc::ptr_eq(
            &current,
            owner.GetDocument().SVGResources().unwrap()
        ));
        owner.DidCommitPaint();
        owner
            .GetDocumentMut()
            .SetAttribute(clip, attr_value("id", "replacement"));
        assert!(owner.UpdateSVGResources());
        assert!(owner.GetStyleImpact().paint);
        assert!(!owner.GetStyleImpact().layout);
        assert!(owner
            .GetDocument()
            .SVGResources()
            .unwrap()
            .entries
            .iter()
            .all(|entry| entry.id != "clip"));
    }

    #[test]
    fn svg_clip_native_geometry_preserves_zoom_percent_and_linear_calc() {
        use foundation::{CalculationValue, Length, LengthValueRange, PixelsAndPercent};
        assert_eq!(
            native_clip_length(&Length::Fixed(30.0), 3.0).unwrap(),
            SvgClipLength {
                value: 10.0,
                percentage: 0.0
            }
        );
        assert_eq!(
            native_clip_length(&Length::Percent(40.0), 3.0).unwrap(),
            SvgClipLength {
                value: 0.0,
                percentage: 0.4
            }
        );
        let calculation = Length::from_calculation_value(CalculationValue::new(
            PixelsAndPercent::new(30.0, 40.0, true, true),
            LengthValueRange::kAll,
        ));
        assert_eq!(
            native_clip_length(&calculation, 3.0).unwrap(),
            SvgClipLength {
                value: 10.0,
                percentage: 0.4
            }
        );
        assert!(native_clip_length(Length::MinContent(), 1.0).is_err());
    }

    #[test]
    fn svg_clip_native_transform_unzooms_lengths_without_scaling_percentages() {
        use foundation::{
            CalculationValue, Length, LengthValueRange, PixelsAndPercent, TransformOperations,
        };
        use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetEffectiveZoom(3.0);
        let mut operations = TransformOperations::default();
        operations.PushTranslate(
            Length::from_calculation_value(CalculationValue::new(
                PixelsAndPercent::new(30.0, 50.0, true, true),
                LengthValueRange::kAll,
            )),
            Length::Fixed(60.0),
        );
        builder.SetTransform(&operations);
        let native = unsafe { &*builder.TakeStyle() };
        let transform = computed_css_transform(native).unwrap();
        let values = numbers(
            transform
                .strip_prefix("matrix-affine(")
                .unwrap()
                .strip_suffix(')')
                .unwrap(),
        )
        .unwrap();
        assert_eq!(&values[..6], &[1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
        assert_eq!(&values[6..], &[0.5, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn svg_clip_catalog_keeps_nested_resource_clip_explicitly_unsupported() {
        let mut owner = crate::DOM::new();
        let root = root(&mut owner);
        let document = owner.GetDocumentMut();
        let clip = add(
            document,
            root,
            "clipPath",
            &[("id", "clip"), ("clip-path", "url(#other)")],
        );
        add(document, clip, "rect", &[("width", "10"), ("height", "10")]);
        let graph = clip_graph(
            &build_persistent(document),
            document.SVGResourceScope(root).tree_scope_id,
            "clip",
        );
        assert!(graph
            .unsupported
            .as_deref()
            .is_some_and(|error| error.contains("nested clip-path")));
    }

    #[test]
    fn svg_clip_catalog_requires_layout_attachment_but_not_visible_resource() {
        let mut owner = crate::DOM::new();
        let root = root(&mut owner);
        let document = owner.GetDocumentMut();
        let clip = add(
            document,
            root,
            "clipPath",
            &[("id", "clip"), ("visibility", "hidden")],
        );
        add(
            document,
            clip,
            "rect",
            &[("width", "10"), ("height", "10"), ("visibility", "visible")],
        );
        let snapshot = build_persistent(document);
        assert!(snapshot
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap()
            .graph
            .is_some());
        document.SetAttribute(clip, attr_value("display", "none"));
        let snapshot = build_persistent(document);
        let entry = snapshot
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap();
        assert_eq!(entry.target_node_id, document.Node(clip).Id());
        assert!(entry.graph.is_none());
        document.SetAttribute(clip, attr_value("display", "contents"));
        assert!(build_persistent(document)
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap()
            .graph
            .is_none());
        document.SetAttribute(clip, attr_value("display", "inline"));
        document.SetAttribute(root, attr_value("display", "none"));
        assert!(build_persistent(document)
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap()
            .graph
            .is_none());
        document.SetAttribute(root, attr_value("display", "inline"));
        let html_parent = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        document.AppendChild(root, html_parent);
        document.AppendChild(html_parent, clip);
        assert!(build_persistent(document)
            .entries
            .iter()
            .find(|entry| entry.id == "clip")
            .unwrap()
            .graph
            .is_none());
    }
}

/// Canonical solid-shape resource geometry. Percent geometry needs a deferred
/// resource viewport and is explicit unsupported in this recording slice.
pub fn SVGMaskShape(
    document: &crate::Document,
    index: usize,
    native: &layoutng_style::style::computed_style::ComputedStyle,
) -> Result<Option<layoutng_assembly::internal::layout_input::SvgShapeData>, String> {
    use layoutng_assembly::internal::layout_input::{SvgShapeData, SvgShapeGeometry};
    let node = document.Node(index);
    if node.Name() == "path" {
        if !native.D().is_null() {
            return Err("CSS d resource geometry".into());
        }
        return Ok(super::layout_object_tree_builder::SVGShapeDataForResourceNode(node));
    }
    let fixed = |v: SvgClipLength| {
        if v.percentage == 0.0 {
            Ok(v.value)
        } else {
            Err("percentage mask child geometry".to_owned())
        }
    };
    let mut shape = SvgShapeData::default();
    match native_shape_geometry(node.Name(), native)? {
        Some(SvgClipGeometry::kRect {
            x,
            y,
            width,
            height,
            rx,
            ry,
        }) => {
            shape.geometry = SvgShapeGeometry::kRectangle;
            shape.bounds_offset = Offset {
                x: fixed(x)?,
                y: fixed(y)?,
            };
            shape.bounds_width = fixed(width)?.max(0.0);
            shape.bounds_height = fixed(height)?.max(0.0);
            shape.rectangle_radius_x = rx
                .or(ry)
                .map(fixed)
                .transpose()?
                .unwrap_or(0.0)
                .max(0.0)
                .min(shape.bounds_width / 2.0);
            shape.rectangle_radius_y = ry
                .or(rx)
                .map(fixed)
                .transpose()?
                .unwrap_or(0.0)
                .max(0.0)
                .min(shape.bounds_height / 2.0);
        }
        Some(SvgClipGeometry::kCircle { cx, cy, r }) => {
            let radius = fixed(r)?.max(0.0);
            shape.geometry = SvgShapeGeometry::kEllipse;
            shape.bounds_offset = Offset {
                x: fixed(cx)? - radius,
                y: fixed(cy)? - radius,
            };
            shape.bounds_width = 2.0 * radius;
            shape.bounds_height = 2.0 * radius;
        }
        Some(SvgClipGeometry::kEllipse { cx, cy, rx, ry }) => {
            let (rx, ry) = (fixed(rx)?.max(0.0), fixed(ry)?.max(0.0));
            shape.geometry = SvgShapeGeometry::kEllipse;
            shape.bounds_offset = Offset {
                x: fixed(cx)? - rx,
                y: fixed(cy)? - ry,
            };
            shape.bounds_width = 2.0 * rx;
            shape.bounds_height = 2.0 * ry;
        }
        _ => return Ok(None),
    }
    Ok(Some(shape))
}

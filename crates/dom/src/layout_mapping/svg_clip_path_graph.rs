//! Document-owned snapshots for local SVG <clipPath> resources.
use crate::{persistent_document::DOMNamespace, Child, Element, ParsedDocument as Document};
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use layoutng_assembly::internal::paint_input::{
    SvgClipGeometry, SvgClipLength, SvgClipPathChild, SvgClipPathGraph, SvgClipPathUnits,
    SvgClipPreserveAspectRatio, SvgClipText, SvgClipTextAnchor, SvgClipTextPath, SvgClipTransform,
    SvgClipTransformStep, SvgClipViewport,
};
use std::collections::HashSet;
use std::sync::Arc;

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
    let apply = |width: f32, height: f32| {
        let mut matrix = foundation::gfx::Transform::default();
        let reference = foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(0.0, 0.0),
            foundation::gfx::SizeF::new(width, height),
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
    attr(element, "style")
        .and_then(|style| {
            style.split(';').find_map(|item| {
                let (key, value) = item.split_once(':')?;
                (key.trim().eq_ignore_ascii_case(name)).then_some(value.trim())
            })
        })
        .or_else(|| attr(element, name))
}

fn use_viewport_transform(
    use_element: &Element,
    target: &Element,
) -> Result<Option<SvgClipViewport>, String> {
    let Some(raw_view_box) = attr(target, "viewBox") else {
        return Ok(None);
    };
    let view_box = numbers(raw_view_box)?;
    if view_box.len() != 4 || view_box[2] <= 0.0 || view_box[3] <= 0.0 {
        return Err("invalid SVG clip use viewBox".into());
    }
    let width = if attr(use_element, "width").is_some() {
        svg_length(use_element, "width", view_box[2])?
    } else {
        svg_length(target, "width", view_box[2])?
    };
    let height = if attr(use_element, "height").is_some() {
        svg_length(use_element, "height", view_box[3])?
    } else {
        svg_length(target, "height", view_box[3])?
    };
    let preserve = attr(use_element, "preserveAspectRatio")
        .or_else(|| attr(target, "preserveAspectRatio"))
        .unwrap_or("xMidYMid meet");
    let parts = preserve.split_ascii_whitespace().collect::<Vec<_>>();
    let align = parts.first().copied().unwrap_or("xMidYMid");
    let align_x = if align.contains("xMax") {
        1.0
    } else if align.contains("xMid") {
        0.5
    } else {
        0.0
    };
    let align_y = if align.contains("YMax") {
        1.0
    } else if align.contains("YMid") {
        0.5
    } else {
        0.0
    };
    let preserve = if align == "none" {
        SvgClipPreserveAspectRatio::kNone
    } else if parts.get(1) == Some(&"slice") {
        SvgClipPreserveAspectRatio::kSlice { align_x, align_y }
    } else {
        SvgClipPreserveAspectRatio::kMeet { align_x, align_y }
    };
    Ok(Some(SvgClipViewport {
        view_box: view_box.try_into().unwrap(),
        width,
        height,
        preserve,
    }))
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
fn find_id(document: &Document, id: &str) -> Option<usize> {
    document
        .elements
        .iter()
        .position(|element| attr(element, "id") == Some(id) || element.id.as_deref() == Some(id))
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
fn text_path(document: &Document, element: &Element) -> Result<Option<SvgClipTextPath>, String> {
    if element.tag != "textPath" {
        return Ok(None);
    }
    let href = attr(element, "href")
        .or_else(|| attr(element, "xlink:href"))
        .ok_or_else(|| "SVG clip textPath missing href".to_owned())?;
    let id = href
        .strip_prefix('#')
        .ok_or_else(|| "external SVG clip textPath IRI".to_owned())?;
    let index =
        find_id(document, id).ok_or_else(|| "missing SVG clip textPath target".to_owned())?;
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
    document: &Document,
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
        text_path: text_path(document, element)?,
    })
}
fn collect_child(
    document: &Document,
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
    if style_value(element, "clip-path").is_some() {
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
        let href = attr(element, "href")
            .or_else(|| attr(element, "xlink:href"))
            .ok_or_else(|| "SVG clip use missing href".to_owned())?;
        let Some(id) = href.strip_prefix('#') else {
            return Err("external SVG clip use IRI".into());
        };
        let target =
            find_id(document, id).ok_or_else(|| "missing SVG clip use target".to_owned())?;
        let use_x = svg_length(element, "x", 0.0)?;
        let use_y = svg_length(element, "y", 0.0)?;
        let mut translated = combined;
        translated.push(SvgClipTransformStep::kMatrix(SvgClipTransform {
            values: [1.0, 0.0, 0.0, 1.0, use_x.value, use_y.value],
            percentages: [use_x.percentage, 0.0, 0.0, use_y.percentage],
        }));
        let target_element = &document.elements[target];
        if matches!(target_element.tag.as_str(), "symbol" | "svg") {
            if let Some(viewport) = use_viewport_transform(element, target_element)? {
                translated.push(SvgClipTransformStep::kViewport(viewport));
            }
        }
        collect_child(
            document, target, stack, translated, even_odd, visible, output,
        )?
    } else if matches!(element.tag.as_str(), "g" | "a" | "symbol" | "svg") {
        for child in &element.children {
            match child {
                Child::Element(child) => collect_child(
                    document,
                    *child,
                    stack,
                    combined.clone(),
                    even_odd,
                    visible,
                    output,
                )?,
                Child::Text(text) if !document.texts[*text].trim().is_empty() => {
                    return Err("SVG text clip content requires mask clipping".into())
                }
                Child::Text(_) => {}
            }
        }
    } else if matches!(element.tag.as_str(), "text" | "tspan" | "textPath") {
        let preserve = attr(element, "xml:space") == Some("preserve");
        for child in &element.children {
            match child {
                Child::Text(index) => {
                    let text = normalize_svg_text(&document.texts[*index], preserve);
                    if visible && !text.is_empty() {
                        output.push(SvgClipPathChild {
                            commands: Vec::new(),
                            geometry: None,
                            text: Some(text_semantics(document, element, text)?),
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
        let geometry = shape_geometry(element)?;
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
fn build_one(document: &Document, index: usize, stack: &mut HashSet<usize>) -> SvgClipPathGraph {
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
        unsupported: transform_error,
    };
    if !stack.insert(index) {
        graph.unsupported = Some("recursive SVG clipPath href".into());
        return graph;
    }
    let local = element
        .children
        .iter()
        .any(|child| matches!(child, Child::Element(_)));
    if let Some(href) = attr(element, "href").or_else(|| attr(element, "xlink:href")) {
        if let Some(id) = href.strip_prefix('#') {
            if let Some(base) = find_id(document, id) {
                if document.elements[base].namespace != DOMNamespace::kSVG
                    || document.elements[base].tag != "clipPath"
                {
                    graph.unsupported = Some("SVG clipPath href target is not a clipPath".into())
                } else {
                    let inherited = build_one(document, base, stack);
                    if inherited.unsupported.is_some() {
                        graph.unsupported = inherited.unsupported
                    }
                    if attr(element, "clipPathUnits").is_none() {
                        graph.units = inherited.units
                    }
                    if attr(element, "transform").is_none() {
                        graph.transform = inherited.transform
                    }
                    if !local {
                        graph.children = inherited.children
                    }
                }
            } else {
                graph.unsupported = Some("missing SVG clipPath href target".into())
            }
        } else {
            graph.unsupported = Some("external SVG clipPath href".into())
        }
    }
    if local {
        let inherited_even_odd = matches!(
            style_value(element, "clip-rule"),
            Some(value) if value.eq_ignore_ascii_case("evenodd")
        );
        for child in &element.children {
            let Child::Element(child) = child else {
                continue;
            };
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
        if graph.unsupported.is_none()
            && element.children.iter().any(|child| {
                matches!(child, Child::Text(text) if !document.texts[*text].trim().is_empty())
            })
        {
            graph.unsupported = Some("SVG text clip content requires mask clipping".into())
        }
    }
    stack.remove(&index);
    graph
}
pub(super) fn build(document: &Document) -> Arc<Vec<SvgClipPathGraph>> {
    Arc::new(
        document
            .elements
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                element.namespace == DOMNamespace::kSVG && element.tag == "clipPath"
            })
            .map(|(index, _)| build_one(document, index, &mut HashSet::new()))
            .collect(),
    )
}

/// Project the retained DOM into the same immutable resource snapshot used by
/// the static adapter. Arena indices stay unchanged so local fragment links
/// and mutation-driven rebuilds keep document identity semantics.
pub(super) fn build_persistent(document: &crate::Document) -> Arc<Vec<SvgClipPathGraph>> {
    use crate::persistent_document::DOMNodeType;

    let mut snapshot = Document::default();
    let mut text_indices = vec![None; document.NodeCount()];
    for (index, slot) in text_indices.iter_mut().enumerate() {
        let node = document.Node(index);
        if node.Type() == DOMNodeType::kText {
            *slot = Some(snapshot.texts.len());
            snapshot.texts.push(node.Data().to_owned());
        }
    }
    for index in 0..document.NodeCount() {
        let node = document.Node(index);
        let mut attributes = node
            .Attributes()
            .iter()
            .map(|attribute| (attribute.local_name.clone(), attribute.value.clone()))
            .collect::<Vec<_>>();
        if let Some(native) = document
            .ResolvedStyleFor(index)
            .and_then(|resolved| unsafe { resolved.native_style.Get().as_ref() })
        {
            if native.HasTransform() {
                match computed_css_transform(native) {
                    Ok(transform) => {
                        attributes.push(("data-computed-clip-transform".into(), transform))
                    }
                    Err(error) => {
                        attributes.push(("data-computed-clip-transform-error".into(), error))
                    }
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
            // Resource children do not create normal layout objects, but their
            // final inherited/presentation values still participate in clip
            // geometry. Store the computed values in the immutable snapshot.
            attributes.retain(|(name, _)| {
                !matches!(name.as_str(), "clip-rule" | "display" | "visibility")
            });
            attributes.push((
                "clip-rule".into(),
                if native.ClipRule() == foundation::WindRule::RULE_EVENODD {
                    "evenodd"
                } else {
                    "nonzero"
                }
                .into(),
            ));
            if native.Display() == foundation::EDisplay::kNone {
                attributes.push(("display".into(), "none".into()));
            }
            attributes.push((
                "visibility".into(),
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
                DOMNodeType::kText => text_indices[*child].map(Child::Text),
                _ => None,
            })
            .collect();
        snapshot.elements.push(Element {
            tag: node.Name().to_owned(),
            namespace: node.Namespace(),
            id: node
                .FindAttribute("id")
                .map(|attribute| attribute.value.clone()),
            classes: Vec::new(),
            inline_style: node
                .FindAttribute("style")
                .map(|attribute| attribute.value.clone()),
            attributes,
            parent: node.Parent(),
            children,
        });
    }
    build(&snapshot)
}

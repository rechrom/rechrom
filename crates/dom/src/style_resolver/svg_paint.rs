//! Resolve local SVG paint resources after stop styles have completed cascading.
#![allow(non_snake_case)]
use super::number::Number;
use crate::persistent_document::DOMNamespace;
use layoutng_assembly::internal::layout_input::{ComputedStyle, Offset};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintColorStop, PaintShader, PaintShaderKind, PaintSpreadMethod,
};
use std::collections::{HashMap, HashSet};

pub(super) fn ParseReference(value: &str) -> Option<(String, Option<Color>, bool)> {
    let value = value.trim();
    if !value.get(..4)?.eq_ignore_ascii_case("url(") {
        return None;
    }
    let close = value.find(')')?;
    let mut reference = value[4..close].trim();
    if (reference.starts_with('"') && reference.ends_with('"'))
        || (reference.starts_with('\'') && reference.ends_with('\''))
    {
        reference = &reference[1..reference.len() - 1];
    }
    let fragment = reference.strip_prefix('#').filter(|id| !id.is_empty())?;
    let fallback = value[close + 1..].trim();
    let current_color = fallback.eq_ignore_ascii_case("currentcolor");
    let color = if fallback.is_empty() || fallback.eq_ignore_ascii_case("none") || current_color {
        None
    } else {
        Some(layoutng_assembly::css_color_parser::ParseCSSColor(
            fallback,
        )?)
    };
    Some((fragment.to_owned(), color, current_color))
}

trait Tree {
    fn name(&self, index: usize) -> &str;
    fn svg(&self, index: usize) -> bool;
    fn attribute(&self, index: usize, name: &str) -> Option<&str>;
    fn children(&self, index: usize) -> Vec<usize>;
    fn parent(&self, index: usize) -> Option<usize>;
}
impl Tree for crate::Document {
    fn name(&self, i: usize) -> &str {
        self.Node(i).Name()
    }
    fn svg(&self, i: usize) -> bool {
        self.Node(i).Namespace() == DOMNamespace::kSVG
    }
    fn attribute(&self, i: usize, n: &str) -> Option<&str> {
        self.Node(i).FindAttribute(n).map(|a| a.value.as_str())
    }
    fn children(&self, i: usize) -> Vec<usize> {
        self.Node(i).Children().to_vec()
    }
    fn parent(&self, i: usize) -> Option<usize> {
        self.Node(i).Parent()
    }
}
impl Tree for crate::ParsedDocument {
    fn name(&self, i: usize) -> &str {
        &self.elements[i].tag
    }
    fn svg(&self, i: usize) -> bool {
        self.elements[i].namespace == DOMNamespace::kSVG
    }
    fn attribute(&self, i: usize, n: &str) -> Option<&str> {
        self.elements[i]
            .attributes
            .iter()
            .find(|(name, _)| name == n)
            .map(|(_, value)| value.as_str())
    }
    fn children(&self, i: usize) -> Vec<usize> {
        self.elements[i]
            .children
            .iter()
            .filter_map(|c| match c {
                crate::Child::Element(i) => Some(*i),
                _ => None,
            })
            .collect()
    }
    fn parent(&self, i: usize) -> Option<usize> {
        self.elements[i].parent
    }
}

fn Ids(tree: &impl Tree, root: usize) -> HashMap<String, usize> {
    let mut ids = HashMap::new();
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        if let Some(id) = tree.attribute(i, "id") {
            ids.entry(id.to_owned()).or_insert(i);
        }
        stack.extend(tree.children(i).into_iter().rev());
    }
    ids
}

fn Resolve(
    tree: &impl Tree,
    index: usize,
    reference: &str,
    ids: &HashMap<String, usize>,
    style: impl Fn(usize) -> Option<ComputedStyle>,
) -> Option<PaintShader> {
    let server = *ids.get(reference)?;
    if !tree.svg(server) || !matches!(tree.name(server), "linearGradient" | "radialGradient") {
        return None;
    }
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut current = Some(server);
    while let Some(i) = current {
        if !seen.insert(i) {
            return None;
        }
        if !tree.svg(i) || !matches!(tree.name(i), "linearGradient" | "radialGradient") {
            break;
        }
        chain.push(i);
        current = tree
            .attribute(i, "href")
            .or_else(|| tree.attribute(i, "xlink:href"))
            .and_then(|href| href.strip_prefix('#'))
            .and_then(|id| ids.get(id))
            .copied();
    }
    let attribute = |name: &str| chain.iter().find_map(|&i| tree.attribute(i, name));
    let unit_coordinates = attribute("gradientUnits") != Some("userSpaceOnUse");
    let mut width = 300.0;
    let mut height = 150.0;
    let mut ancestor = Some(index);
    while let Some(i) = ancestor {
        if tree.svg(i) && tree.name(i) == "svg" {
            if let Some(viewbox) = tree.attribute(i, "viewBox") {
                let numbers: Vec<_> = viewbox
                    .split(|c: char| c == ',' || c.is_ascii_whitespace())
                    .filter(|s| !s.is_empty())
                    .filter_map(Number)
                    .collect();
                if numbers.len() == 4 && numbers[2] > 0.0 && numbers[3] > 0.0 {
                    width = numbers[2];
                    height = numbers[3];
                    break;
                }
            }
            width = tree.attribute(i, "width").and_then(Number).unwrap_or(width);
            height = tree
                .attribute(i, "height")
                .and_then(Number)
                .unwrap_or(height);
            break;
        }
        ancestor = tree.parent(i);
    }
    let x = if unit_coordinates { 1.0 } else { width };
    let y = if unit_coordinates { 1.0 } else { height };
    let r = if unit_coordinates {
        1.0
    } else {
        width.hypot(height) / 2.0_f64.sqrt()
    };
    let coordinate = |name: &str, default: f64, basis: f64| {
        attribute(name)
            .and_then(|v| Coordinate(v, basis))
            .unwrap_or(default)
    };
    let mut shader = PaintShader {
        kind: if tree.name(server) == "linearGradient" {
            PaintShaderKind::kLinearGradient
        } else {
            PaintShaderKind::kRadialGradient
        },
        unit_coordinates,
        object_bounding_box_coordinates: unit_coordinates,
        interpolate_premultiplied: false,
        spread: match attribute("spreadMethod") {
            Some("reflect") => PaintSpreadMethod::kReflect,
            Some("repeat") => PaintSpreadMethod::kRepeat,
            _ => PaintSpreadMethod::kPad,
        },
        ..Default::default()
    };
    shader.start = Offset {
        x: coordinate("x1", 0.0, x),
        y: coordinate("y1", 0.0, y),
    };
    shader.end = Offset {
        x: coordinate("x2", x, x),
        y: coordinate("y2", 0.0, y),
    };
    shader.center = Offset {
        x: coordinate("cx", x / 2.0, x),
        y: coordinate("cy", y / 2.0, y),
    };
    shader.focal = Offset {
        x: coordinate("fx", shader.center.x, x),
        y: coordinate("fy", shader.center.y, y),
    };
    shader.radius = coordinate("r", r / 2.0, r);
    shader.focal_radius = coordinate("fr", 0.0, r);
    if shader.radius < 0.0 || shader.focal_radius < 0.0 {
        return None;
    }
    if let Some(transform) = attribute("gradientTransform") {
        shader.transform = super::transform_parser::ParseSVGTransform(transform)?;
    }
    let stops = chain
        .iter()
        .map(|&i| {
            tree.children(i)
                .into_iter()
                .filter(|&c| tree.svg(c) && tree.name(c) == "stop")
                .collect::<Vec<_>>()
        })
        .find(|stops| !stops.is_empty())?;
    let mut previous = 0.0_f64;
    for stop in stops {
        let stop_style = style(stop)?;
        let offset = tree
            .attribute(stop, "offset")
            .and_then(|v| Coordinate(v, 1.0))
            .unwrap_or(0.0)
            .clamp(0.0, 1.0)
            .max(previous);
        previous = offset;
        let mut color = stop_style.paint.svg_stop_color;
        color.alpha *= stop_style.paint.svg_stop_opacity;
        shader.stops.push(PaintColorStop {
            offset,
            color,
            ..Default::default()
        });
    }
    Some(shader)
}
fn Coordinate(value: &str, basis: f64) -> Option<f64> {
    let value = value.trim();
    match value.strip_suffix('%') {
        Some(number) => Number(number).map(|v| v * basis / 100.0),
        None => Number(value),
    }
}

pub(super) fn ResolveDocument(document: &mut crate::Document) {
    let references: Vec<_> = (0..document.NodeCount())
        .filter_map(|i| {
            let style = document.ResolvedStyleFor(i)?;
            (style.style.paint.svg_fill_reference.is_some()
                || style.style.paint.svg_stroke_reference.is_some())
            .then(|| (i, style.clone()))
        })
        .collect();
    if references.is_empty() {
        return;
    }
    let ids = Ids(document, document.Root());
    for (i, mut resolved) in references {
        let styles = |stop| document.ResolvedStyleFor(stop).map(|r| r.style.clone());
        resolved.style.paint.svg_fill_server = resolved
            .style
            .paint
            .svg_fill_reference
            .as_ref()
            .and_then(|id| Resolve(document, i, id, &ids, styles));
        resolved.style.paint.svg_stroke_server = resolved
            .style
            .paint
            .svg_stroke_reference
            .as_ref()
            .and_then(|id| Resolve(document, i, id, &ids, styles));
        if document.ResolvedStyleFor(i) != Some(&resolved) {
            document.StyleStateMut().impact.paint = true;
            document.SetResolvedStyle(i, resolved);
        }
    }
}
pub(super) fn ResolveStatic(document: &crate::ParsedDocument, styles: &mut [ComputedStyle]) {
    if !styles
        .iter()
        .any(|s| s.paint.svg_fill_reference.is_some() || s.paint.svg_stroke_reference.is_some())
    {
        return;
    }
    let Some(root) = document.root else {
        return;
    };
    let ids = Ids(document, root);
    for i in 0..styles.len() {
        let fill = styles[i]
            .paint
            .svg_fill_reference
            .as_ref()
            .and_then(|id| Resolve(document, i, id, &ids, |stop| styles.get(stop).cloned()));
        let stroke = styles[i]
            .paint
            .svg_stroke_reference
            .as_ref()
            .and_then(|id| Resolve(document, i, id, &ids, |stop| styles.get(stop).cloned()));
        styles[i].paint.svg_fill_server = fill;
        styles[i].paint.svg_stroke_server = stroke;
    }
}

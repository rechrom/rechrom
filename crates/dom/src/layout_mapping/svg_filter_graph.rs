//! Document-owned SVG filter resource snapshots.  CSS keeps only the IRI;
//! this is the DOM -> Layout projection of the referenced SVG graph.
use crate::persistent_document::{DOMNamespace, DOMNode, DOMNodeType};
use crate::{Child, Element, ParsedDocument as Document};
use layoutng_assembly::internal::layout_input::ComputedStyle;
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::*;
use std::collections::HashSet;
use std::sync::Arc;

// A borrowed view keeps filter graph construction on its DOM resource subtree.
// In particular, do not serialize the live document and clone every computed
// style just to adapt it to the static parser's graph builder.
#[derive(Clone, Copy)]
enum FilterElement<'a> {
    Parsed(&'a Element),
    Persistent(&'a DOMNode),
}
impl<'a> FilterElement<'a> {
    fn tag(self) -> &'a str {
        match self {
            Self::Parsed(n) => &n.tag,
            Self::Persistent(n) => n.Name(),
        }
    }
    fn namespace(self) -> DOMNamespace {
        match self {
            Self::Parsed(n) => n.namespace,
            Self::Persistent(n) => n.Namespace(),
        }
    }
    fn id(self) -> Option<&'a str> {
        match self {
            Self::Parsed(n) => n.id.as_deref(),
            Self::Persistent(n) => n.FindAttribute("id").map(|a| a.value.as_str()),
        }
    }
}
enum FilterDocument<'a> {
    Parsed(&'a Document, &'a [ComputedStyle]),
    Persistent(&'a crate::Document),
}
impl<'a> FilterDocument<'a> {
    // TreeScopeResources resolves attached targets in DOM tree order. The
    // persistent arena also contains detached allocation history, which must
    // neither add filter resources nor make a local layout update scan it.
    fn indices<'b>(&'b self) -> impl Iterator<Item = usize> + 'b + use<'a, 'b> {
        let mut parsed_index = 0;
        let mut stack = match self {
            Self::Persistent(document) => document.ActiveTreeScopes(),
            Self::Parsed(..) => Vec::new(),
        };
        stack.reverse();
        std::iter::from_fn(move || match self {
            Self::Parsed(document, _) => {
                let index = parsed_index;
                parsed_index += 1;
                (index < document.elements.len()).then_some(index)
            }
            Self::Persistent(document) => loop {
                let index = stack.pop()?;
                let node = document.Node(index);
                stack.extend(node.Children().iter().rev().copied());
                if node.Type() == DOMNodeType::kElement {
                    return Some(index);
                }
            },
        })
    }
    fn element(&self, index: usize) -> FilterElement<'a> {
        match self {
            Self::Parsed(d, _) => FilterElement::Parsed(&d.elements[index]),
            Self::Persistent(d) => FilterElement::Persistent(d.Node(index)),
        }
    }
    fn style(&self, index: usize) -> Option<&'a ComputedStyle> {
        match self {
            Self::Parsed(_, s) => s.get(index),
            Self::Persistent(d) => d.ResolvedStyleFor(index).map(|s| &s.style),
        }
    }
    fn children<'b>(
        &'b self,
        element: FilterElement<'b>,
    ) -> impl Iterator<Item = usize> + 'b + use<'a, 'b> {
        let (parsed, persistent) = match element {
            FilterElement::Parsed(n) => (n.children.as_slice(), &[][..]),
            FilterElement::Persistent(n) => (&[][..], n.Children()),
        };
        parsed
            .iter()
            .filter_map(|c| match c {
                Child::Element(i) => Some(*i),
                _ => None,
            })
            .chain(persistent.iter().copied().filter(|&i| match self {
                Self::Persistent(d) => d.Node(i).Type() == DOMNodeType::kElement,
                _ => false,
            }))
    }
    fn image(&self, source: &str) -> Option<&'a crate::ImageResourceMetadata> {
        match self {
            Self::Parsed(d, _) => d.ImageResourceFor(source),
            Self::Persistent(d) => d.ImageResourceFor(source),
        }
    }
}
fn attr<'a>(element: FilterElement<'a>, name: &str) -> Option<&'a str> {
    match element {
        FilterElement::Parsed(n) => n
            .attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str()),
        FilterElement::Persistent(n) => n.FindAttribute(name).map(|a| a.value.as_str()),
    }
}
fn number(value: Option<&str>, default: f64) -> f64 {
    value
        .and_then(|value| value.trim().parse().ok())
        .filter(|v: &f64| v.is_finite())
        .unwrap_or(default)
}
fn numbers(value: Option<&str>) -> Vec<f64> {
    value
        .unwrap_or("")
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .collect()
}
fn length(value: Option<&str>) -> Option<SvgFilterLength> {
    let value = value?.trim();
    let (text, percentage) = value
        .strip_suffix('%')
        .map_or((value, false), |v| (v, true));
    text.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .map(|value| SvgFilterLength { value, percentage })
}
fn region(element: FilterElement<'_>) -> SvgFilterRegion {
    SvgFilterRegion {
        x: length(attr(element, "x")),
        y: length(attr(element, "y")),
        width: length(attr(element, "width")),
        height: length(attr(element, "height")),
    }
}
fn input(value: Option<&str>) -> SvgFilterInput {
    match value.unwrap_or("").trim() {
        "SourceGraphic" => SvgFilterInput::kSourceGraphic,
        "SourceAlpha" => SvgFilterInput::kSourceAlpha,
        "BackgroundImage" => SvgFilterInput::kBackgroundImage,
        "BackgroundAlpha" => SvgFilterInput::kBackgroundAlpha,
        "FillPaint" => SvgFilterInput::kFillPaint,
        "StrokePaint" => SvgFilterInput::kStrokePaint,
        value => SvgFilterInput::kNamed(value.to_owned()),
    }
}
fn edge_mode(value: Option<&str>) -> Result<SvgFilterEdgeMode, String> {
    match value.unwrap_or("none") {
        "none" => Ok(SvgFilterEdgeMode::kNone),
        "duplicate" => Ok(SvgFilterEdgeMode::kDuplicate),
        "wrap" => Ok(SvgFilterEdgeMode::kWrap),
        value => Err(format!("invalid SVG filter edgeMode {value}")),
    }
}
fn channel(value: Option<&str>, default: SvgFilterChannel) -> Result<SvgFilterChannel, String> {
    match value {
        None => Ok(default),
        Some("R") => Ok(SvgFilterChannel::kR),
        Some("G") => Ok(SvgFilterChannel::kG),
        Some("B") => Ok(SvgFilterChannel::kB),
        Some("A") => Ok(SvgFilterChannel::kA),
        Some(v) => Err(format!("invalid displacement channel {v}")),
    }
}
fn transfer_function(element: FilterElement<'_>) -> Result<SvgTransferFunction, String> {
    Ok(match attr(element, "type").unwrap_or("identity") {
        "identity" => SvgTransferFunction::kIdentity,
        "table" => SvgTransferFunction::kTable(numbers(attr(element, "tableValues"))),
        "discrete" => SvgTransferFunction::kDiscrete(numbers(attr(element, "tableValues"))),
        "linear" => SvgTransferFunction::kLinear {
            slope: number(attr(element, "slope"), 1.0),
            intercept: number(attr(element, "intercept"), 0.0),
        },
        "gamma" => SvgTransferFunction::kGamma {
            amplitude: number(attr(element, "amplitude"), 1.0),
            exponent: number(attr(element, "exponent"), 1.0),
            offset: number(attr(element, "offset"), 0.0),
        },
        value => return Err(format!("unsupported component transfer type {value}")),
    })
}
fn style_value<'a>(element: FilterElement<'a>, property: &str) -> Option<&'a str> {
    if let Some(value) = attr(element, property) {
        return Some(value);
    }
    attr(element, "style")?.split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;
        (name.trim().eq_ignore_ascii_case(property)).then_some(value.trim())
    })
}
fn computed_flood_color(style: &ComputedStyle) -> Option<Color> {
    let native = unsafe { style.native_style.as_ref()? };
    let current = native.GetCurrentColor(None);
    let value = native
        .FloodColor()
        .Resolve(current, native.UsedColorScheme(), None);
    let (red, green, blue) = value.ToSRGB();
    Some(Color {
        red,
        green,
        blue,
        alpha: value.Alpha() * native.FloodOpacity().clamp(0.0, 1.0),
    })
}
fn flood_color(element: FilterElement<'_>, style: Option<&ComputedStyle>) -> Result<Color, String> {
    if let Some(color) = style.and_then(computed_flood_color) {
        return Ok(color);
    }
    let raw = style_value(element, "flood-color").unwrap_or("black");
    if raw.eq_ignore_ascii_case("currentcolor") {
        return Err("feFlood currentColor requires computed SVG style".into());
    }
    let mut color = layoutng_assembly::css_color_parser::ParseCSSColor(raw)
        .ok_or_else(|| "invalid feFlood flood-color".to_owned())?;
    color.alpha *= number(style_value(element, "flood-opacity"), 1.0).clamp(0.0, 1.0) as f32;
    Ok(color)
}
fn computed_lighting_color(style: &ComputedStyle) -> Option<Color> {
    let native = unsafe { style.native_style.as_ref()? };
    let current = native.GetCurrentColor(None);
    let value = native
        .LightingColor()
        .Resolve(current, native.UsedColorScheme(), None);
    let (red, green, blue) = value.ToSRGB();
    Some(Color {
        red,
        green,
        blue,
        alpha: value.Alpha(),
    })
}
fn lighting_color(
    element: FilterElement<'_>,
    style: Option<&ComputedStyle>,
) -> Result<Color, String> {
    // SVGFilterBuilder reads the primitive's final ComputedStyle. This keeps
    // presentation attributes, author declarations, inheritance and
    // currentColor on the canonical cascade path instead of reparsing one
    // declaration from the resource element.
    if let Some(color) = style.and_then(computed_lighting_color) {
        return Ok(color);
    }
    let raw = style_value(element, "lighting-color").unwrap_or("white");
    if raw.eq_ignore_ascii_case("currentcolor") {
        return Err("SVG lighting-color currentColor requires computed primitive style".into());
    }
    layoutng_assembly::css_color_parser::ParseCSSColor(raw)
        .ok_or_else(|| "invalid SVG lighting-color".to_owned())
}
fn linear_rgb(element: FilterElement<'_>, style: Option<&ComputedStyle>, inherited: bool) -> bool {
    if let Some(native) = style.and_then(|style| unsafe { style.native_style.as_ref() }) {
        return native.ColorInterpolationFilters() == foundation::EColorInterpolation::kLinearrgb;
    }
    match style_value(element, "color-interpolation-filters") {
        Some(value) if value.eq_ignore_ascii_case("linearrgb") => true,
        Some(value) if value.eq_ignore_ascii_case("srgb") || value.eq_ignore_ascii_case("auto") => {
            false
        }
        _ => inherited,
    }
}
fn light_source(
    document: &FilterDocument<'_>,
    element: FilterElement<'_>,
) -> Option<SvgLightSource> {
    document.children(element).find_map(|index| {
        let light = document.element(index);
        match light.tag() {
            "feDistantLight" => Some(SvgLightSource::kDistant {
                azimuth: number(attr(light, "azimuth"), 0.0),
                elevation: number(attr(light, "elevation"), 0.0),
            }),
            "fePointLight" => Some(SvgLightSource::kPoint {
                x: number(attr(light, "x"), 0.0),
                y: number(attr(light, "y"), 0.0),
                z: number(attr(light, "z"), 0.0),
            }),
            "feSpotLight" => Some(SvgLightSource::kSpot {
                x: number(attr(light, "x"), 0.0),
                y: number(attr(light, "y"), 0.0),
                z: number(attr(light, "z"), 0.0),
                points_at_x: number(attr(light, "pointsAtX"), 0.0),
                points_at_y: number(attr(light, "pointsAtY"), 0.0),
                points_at_z: number(attr(light, "pointsAtZ"), 0.0),
                specular_exponent: number(attr(light, "specularExponent"), 1.0).clamp(1.0, 128.0),
                limiting_cone_angle: number(attr(light, "limitingConeAngle"), 0.0),
            }),
            _ => None,
        }
    })
}
fn primitive(document: &FilterDocument<'_>, index: usize) -> Result<SvgFilterPrimitive, String> {
    let element = document.element(index);
    let first = input(attr(element, "in"));
    let kind = match element.tag() {
        "feGaussianBlur" => {
            let values = numbers(attr(element, "stdDeviation"));
            let x = values.first().copied().unwrap_or(0.0).max(0.0);
            let y = values.get(1).copied().unwrap_or(x).max(0.0);
            SvgFilterPrimitiveKind::kGaussianBlur {
                sigma_x: x,
                sigma_y: y,
                edge_mode: edge_mode(attr(element, "edgeMode"))?,
            }
        }
        "feColorMatrix" => {
            let ty = attr(element, "type").unwrap_or("matrix");
            let kind = match ty {
                "matrix" => SvgColorMatrixType::kMatrix,
                "saturate" => SvgColorMatrixType::kSaturate,
                "hueRotate" => SvgColorMatrixType::kHueRotate,
                "luminanceToAlpha" => SvgColorMatrixType::kLuminanceToAlpha,
                _ => return Err(format!("unsupported feColorMatrix type {ty}")),
            };
            SvgFilterPrimitiveKind::kColorMatrix {
                kind,
                values: numbers(attr(element, "values")),
            }
        }
        "feOffset" => SvgFilterPrimitiveKind::kOffset {
            dx: number(attr(element, "dx"), 0.0),
            dy: number(attr(element, "dy"), 0.0),
        },
        "feFlood" => SvgFilterPrimitiveKind::kFlood {
            color: flood_color(element, document.style(index))?,
        },
        "feBlend" => {
            let mode = match attr(element, "mode").unwrap_or("normal") {
                "normal" => SvgBlendMode::kNormal,
                "multiply" => SvgBlendMode::kMultiply,
                "screen" => SvgBlendMode::kScreen,
                "darken" => SvgBlendMode::kDarken,
                "lighten" => SvgBlendMode::kLighten,
                value => return Err(format!("unsupported feBlend mode {value}")),
            };
            SvgFilterPrimitiveKind::kBlend {
                mode,
                input2: input(attr(element, "in2")),
            }
        }
        "feComposite" => {
            let operator = match attr(element, "operator").unwrap_or("over") {
                "over" => SvgCompositeOperator::kOver,
                "in" => SvgCompositeOperator::kIn,
                "out" => SvgCompositeOperator::kOut,
                "atop" => SvgCompositeOperator::kAtop,
                "xor" => SvgCompositeOperator::kXor,
                "arithmetic" => SvgCompositeOperator::kArithmetic,
                value => return Err(format!("unsupported feComposite operator {value}")),
            };
            SvgFilterPrimitiveKind::kComposite {
                operator,
                input2: input(attr(element, "in2")),
                k: [
                    number(attr(element, "k1"), 0.0),
                    number(attr(element, "k2"), 0.0),
                    number(attr(element, "k3"), 0.0),
                    number(attr(element, "k4"), 0.0),
                ],
            }
        }
        "feMerge" => {
            let mut inputs = Vec::new();
            for index in document.children(element) {
                let node = document.element(index);
                if node.tag() == "feMergeNode" {
                    inputs.push(input(attr(node, "in")));
                }
            }
            SvgFilterPrimitiveKind::kMerge { inputs }
        }
        "feComponentTransfer" => {
            let mut functions: [SvgTransferFunction; 4] =
                std::array::from_fn(|_| SvgTransferFunction::kIdentity);
            for index in document.children(element) {
                let node = document.element(index);
                let slot = match node.tag() {
                    "feFuncR" => Some(0),
                    "feFuncG" => Some(1),
                    "feFuncB" => Some(2),
                    "feFuncA" => Some(3),
                    _ => None,
                };
                if let Some(slot) = slot {
                    functions[slot] = transfer_function(node)?;
                }
            }
            SvgFilterPrimitiveKind::kComponentTransfer { functions }
        }
        "feMorphology" => {
            let radii = numbers(attr(element, "radius"));
            let x = radii.first().copied().unwrap_or(0.0).max(0.0);
            let y = radii.get(1).copied().unwrap_or(x).max(0.0);
            let operator = match attr(element, "operator").unwrap_or("erode") {
                "erode" => SvgMorphologyOperator::kErode,
                "dilate" => SvgMorphologyOperator::kDilate,
                v => return Err(format!("unsupported morphology operator {v}")),
            };
            SvgFilterPrimitiveKind::kMorphology {
                operator,
                radius_x: x,
                radius_y: y,
            }
        }
        "feDisplacementMap" => SvgFilterPrimitiveKind::kDisplacementMap {
            input2: input(attr(element, "in2")),
            scale: number(attr(element, "scale"), 0.0),
            x_channel: channel(attr(element, "xChannelSelector"), SvgFilterChannel::kA)?,
            y_channel: channel(attr(element, "yChannelSelector"), SvgFilterChannel::kA)?,
        },
        "feTurbulence" => {
            let frequency = numbers(attr(element, "baseFrequency"));
            let x = frequency.first().copied().unwrap_or(0.0).max(0.0);
            let y = frequency.get(1).copied().unwrap_or(x).max(0.0);
            SvgFilterPrimitiveKind::kTurbulence {
                base_x: x,
                base_y: y,
                octaves: number(attr(element, "numOctaves"), 1.0).clamp(1.0, 9.0) as u32,
                seed: number(attr(element, "seed"), 0.0),
                stitch: attr(element, "stitchTiles") == Some("stitch"),
                kind: match attr(element, "type").unwrap_or("turbulence") {
                    "turbulence" => SvgTurbulenceType::kTurbulence,
                    "fractalNoise" => SvgTurbulenceType::kFractalNoise,
                    v => return Err(format!("unsupported turbulence type {v}")),
                },
            }
        }
        "feTile" => SvgFilterPrimitiveKind::kTile,
        "feImage" => {
            let preserve = attr(element, "preserveAspectRatio")
                .map(str::trim)
                .unwrap_or("xMidYMid meet");
            if !matches!(preserve, "none" | "xMidYMid" | "xMidYMid meet") {
                return Err(format!(
                    "unsupported feImage preserveAspectRatio {preserve}"
                ));
            }
            let href = attr(element, "href")
                .or_else(|| attr(element, "xlink:href"))
                .ok_or_else(|| "feImage missing href".to_owned())?
                .to_owned();
            let mut local_fragment = href.starts_with('#');
            let mut resource = if local_fragment {
                None
            } else {
                document.image(&href)
            };
            if local_fragment {
                let id = href.trim_start_matches('#');
                let target = find_id(document, index, id)
                    .ok_or_else(|| "feImage local fragment is missing".to_owned())?;
                let target = document.element(target);
                if target.tag() == "image" {
                    if let Some(source) =
                        attr(target, "href").or_else(|| attr(target, "xlink:href"))
                    {
                        resource = document.image(source);
                        local_fragment = false
                    }
                } else {
                    return Err(
                        "feImage local non-image SVG subtree rasterization is unavailable".into(),
                    );
                }
            }
            SvgFilterPrimitiveKind::kImage {
                local_fragment,
                preserve_none: preserve == "none",
                resource_id: resource.map(|v| v.id),
                width: resource.map_or(0, |v| v.natural_width.max(0.0) as u32),
                height: resource.map_or(0, |v| v.natural_height.max(0.0) as u32),
                pixels: Arc::new(Vec::new()),
                href,
            }
        }
        "feConvolveMatrix" => {
            let order = numbers(attr(element, "order"));
            let ox = order.first().copied().unwrap_or(3.0).max(1.0) as u32;
            let oy = order.get(1).copied().unwrap_or(ox as f64).max(1.0) as u32;
            let kernel = numbers(attr(element, "kernelMatrix"));
            if kernel.len() != ox as usize * oy as usize {
                return Err("feConvolveMatrix kernel/order mismatch".into());
            }
            let sum: f64 = kernel.iter().sum();
            let divisor = attr(element, "divisor")
                .map(|_| number(attr(element, "divisor"), 1.0))
                .unwrap_or(if sum == 0.0 { 1.0 } else { sum });
            if divisor == 0.0 {
                return Err("feConvolveMatrix zero divisor".into());
            }
            let unit = numbers(attr(element, "kernelUnitLength"));
            let ux = unit.first().copied().unwrap_or(1.0);
            let uy = unit.get(1).copied().unwrap_or(ux);
            SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x: ox,
                order_y: oy,
                kernel,
                divisor,
                bias: number(attr(element, "bias"), 0.0),
                target_x: number(attr(element, "targetX"), (ox / 2) as f64)
                    .clamp(0.0, (ox - 1) as f64) as u32,
                target_y: number(attr(element, "targetY"), (oy / 2) as f64)
                    .clamp(0.0, (oy - 1) as f64) as u32,
                edge_mode: edge_mode(attr(element, "edgeMode")).map(|m| {
                    if m == SvgFilterEdgeMode::kNone {
                        SvgFilterEdgeMode::kDuplicate
                    } else {
                        m
                    }
                })?,
                kernel_unit_x: ux,
                kernel_unit_y: uy,
                preserve_alpha: attr(element, "preserveAlpha") == Some("true"),
            }
        }
        "feDiffuseLighting" | "feSpecularLighting" => {
            let specular = element.tag() == "feSpecularLighting";
            let unit = numbers(attr(element, "kernelUnitLength"));
            let kernel_unit_x = unit.first().copied().unwrap_or(0.0);
            let kernel_unit_y = unit.get(1).copied().unwrap_or(kernel_unit_x);
            SvgFilterPrimitiveKind::kLighting {
                lighting_type: if specular {
                    SvgLightingType::kSpecular
                } else {
                    SvgLightingType::kDiffuse
                },
                light: light_source(document, element),
                color: lighting_color(element, document.style(index))?,
                surface_scale: number(attr(element, "surfaceScale"), 1.0),
                constant: number(
                    attr(
                        element,
                        if specular {
                            "specularConstant"
                        } else {
                            "diffuseConstant"
                        },
                    ),
                    1.0,
                )
                .max(0.0),
                specular_exponent: number(attr(element, "specularExponent"), 1.0).clamp(1.0, 128.0),
                kernel_unit_x,
                kernel_unit_y,
            }
        }
        tag => return Err(format!("unsupported SVG filter primitive {tag}")),
    };
    Ok(SvgFilterPrimitive {
        input: first,
        result: attr(element, "result").unwrap_or("").to_owned(),
        region: region(element),
        linear_rgb: linear_rgb(element, document.style(index), true),
        kind,
    })
}
fn find_id(document: &FilterDocument<'_>, source: usize, id: &str) -> Option<usize> {
    match document {
        FilterDocument::Persistent(document) => {
            let source = document.CorrespondingSVGElement(source).unwrap_or(source);
            document.FindElementByIdInTreeScope(source, id)
        }
        FilterDocument::Parsed(..) => document.indices().find(|&index| {
            let element = document.element(index);
            attr(element, "id") == Some(id) || element.id() == Some(id)
        }),
    }
}

fn build_one(
    document: &FilterDocument<'_>,
    index: usize,
    stack: &mut HashSet<usize>,
) -> SvgFilterGraph {
    let element = document.element(index);
    let filter_linear_rgb = linear_rgb(element, document.style(index), true);
    let units = |name, default| match attr(element, name) {
        Some("userSpaceOnUse") => SvgFilterUnits::kUserSpaceOnUse,
        Some("objectBoundingBox") | None => default,
        Some(_) => default,
    };
    let mut graph = SvgFilterGraph {
        id: attr(element, "id")
            .or(element.id())
            .unwrap_or("")
            .to_owned(),
        filter_units: units("filterUnits", SvgFilterUnits::kObjectBoundingBox),
        primitive_units: units("primitiveUnits", SvgFilterUnits::kUserSpaceOnUse),
        region: region(element),
        primitives: Vec::new(),
        fill_paint: None,
        stroke_paint: None,
        unsupported: None,
    };
    if !stack.insert(index) {
        graph.unsupported = Some("recursive SVG filter href".into());
        return graph;
    }
    let has_local_primitives = document.children(element).any(|index| {
        let child = document.element(index);
        child.namespace() == DOMNamespace::kSVG && child.tag().starts_with("fe")
    });
    if let Some(href) = attr(element, "href").or_else(|| attr(element, "xlink:href")) {
        if let Some(id) = href.strip_prefix('#') {
            if let Some(base) = find_id(document, index, id) {
                let inherited = build_one(document, base, stack);
                if inherited.unsupported.is_some() {
                    graph.unsupported = inherited.unsupported;
                }
                if attr(element, "filterUnits").is_none() {
                    graph.filter_units = inherited.filter_units
                }
                if attr(element, "primitiveUnits").is_none() {
                    graph.primitive_units = inherited.primitive_units
                }
                if attr(element, "x").is_none() {
                    graph.region.x = inherited.region.x
                }
                if attr(element, "y").is_none() {
                    graph.region.y = inherited.region.y
                }
                if attr(element, "width").is_none() {
                    graph.region.width = inherited.region.width
                }
                if attr(element, "height").is_none() {
                    graph.region.height = inherited.region.height
                }
                if !has_local_primitives {
                    graph.primitives = inherited.primitives;
                }
            } else {
                graph.unsupported = Some("missing SVG filter href target".into());
            }
        } else {
            graph.unsupported = Some("external SVG filter href".into());
        }
    }
    for child in document.children(element) {
        let node = document.element(child);
        if node.namespace() != DOMNamespace::kSVG || !node.tag().starts_with("fe") {
            continue;
        }
        match primitive(document, child) {
            Ok(mut value) => {
                if document
                    .style(child)
                    .is_none_or(|style| style.native_style.is_null())
                    && style_value(node, "color-interpolation-filters").is_none()
                {
                    value.linear_rgb = filter_linear_rgb;
                }
                graph.primitives.push(value)
            }
            Err(error) => {
                graph.unsupported.get_or_insert(error);
            }
        }
    }
    stack.remove(&index);
    graph
}
fn build_graphs(document: &FilterDocument<'_>) -> Arc<Vec<SvgFilterGraph>> {
    let mut trace = browser_tracing::span("layout", "DOM.BuildSVGFilterGraphs");
    let graphs: Vec<_> = document
        .indices()
        .filter(|&index| {
            let element = document.element(index);
            element.namespace() == DOMNamespace::kSVG && element.tag() == "filter"
        })
        .map(|index| build_one(document, index, &mut HashSet::new()))
        .collect();
    trace.set("filters", graphs.len() as f64);
    Arc::new(graphs)
}
pub(super) fn build(document: &Document, styles: &[ComputedStyle]) -> Arc<Vec<SvgFilterGraph>> {
    build_graphs(&FilterDocument::Parsed(document, styles))
}

// cpp: FilterEffectBuilder::BuildReferenceFilter / SVGFilterBuilder::BuildGraph.
// Read resource attributes and canonical computed styles directly from their
// DOM owner. The output alone is owned by the immutable paint snapshot.
pub(super) fn build_persistent(document: &crate::Document) -> Arc<Vec<SvgFilterGraph>> {
    document
        .SVGFilterGraphs()
        .cloned()
        .unwrap_or_else(|| build_graphs(&FilterDocument::Persistent(document)))
}

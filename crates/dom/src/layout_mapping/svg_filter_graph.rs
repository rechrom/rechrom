//! Document-owned SVG filter resource snapshots.  CSS keeps only the IRI;
//! this is the DOM -> Layout projection of the referenced SVG graph.
use crate::persistent_document::{DOMNamespace, DOMNode, DOMNodeType};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::*;
use layoutng_style::style::computed_style::ComputedStyle;
use std::sync::Arc;

// A borrowed view keeps filter graph construction on its DOM resource subtree.
// In particular, do not serialize the live document and clone every computed
// style just to adapt it to the static parser's graph builder.
#[derive(Clone, Copy)]
struct FilterElement<'a>(&'a DOMNode);
impl<'a> FilterElement<'a> {
    fn tag(self) -> &'a str {
        self.0.Name()
    }
    fn namespace(self) -> DOMNamespace {
        self.0.Namespace()
    }
    fn id(self) -> Option<&'a str> {
        self.0.FindAttribute("id").map(|a| a.value.as_str())
    }
}
struct FilterDocument<'a>(&'a crate::Document);
impl<'a> FilterDocument<'a> {
    // TreeScopeResources resolves attached targets in DOM tree order. The
    // persistent arena also contains detached allocation history, which must
    // neither add filter resources nor make a local layout update scan it.
    fn indices<'b>(&'b self) -> impl Iterator<Item = usize> + 'b + use<'a, 'b> {
        let mut stack = self.0.ActiveTreeScopes();
        stack.reverse();
        std::iter::from_fn(move || loop {
            let index = stack.pop()?;
            let node = self.0.Node(index);
            stack.extend(node.Children().iter().rev().copied());
            if node.Type() == DOMNodeType::kElement {
                return Some(index);
            }
        })
    }
    fn element(&self, index: usize) -> FilterElement<'a> {
        FilterElement(self.0.Node(index))
    }
    fn style(&self, index: usize) -> Option<&'a ComputedStyle> {
        self.0
            .ResolvedStyleFor(index)
            .and_then(|style| unsafe { style.computed_style_owner.Get().as_ref() })
    }
    fn children<'b>(
        &'b self,
        element: FilterElement<'b>,
    ) -> impl Iterator<Item = usize> + 'b + use<'a, 'b> {
        element
            .0
            .Children()
            .iter()
            .copied()
            .filter(|&index| self.0.Node(index).Type() == DOMNodeType::kElement)
    }
    fn image(&self, source: &str) -> Option<&'a crate::ImageResourceMetadata> {
        self.0.ImageResourceFor(source)
    }
}
fn attr<'a>(element: FilterElement<'a>, name: &str) -> Option<&'a str> {
    element
        .0
        .FindAttribute(name)
        .map(|attribute| attribute.value.as_str())
}
fn number(value: Option<&str>, default: f64) -> f64 {
    let Some(value) = value else {
        return default;
    };
    // SVGNumber::SetValueAsString treats the literal empty string as a valid
    // zero before entering the SVG-number parser. Whitespace-only input does
    // enter that parser, fails, and is restored to the constructor initial by
    // SVGAnimatedPropertyBase::UpdateBaseValueFromAttribute.
    if value.is_empty() {
        return 0.0;
    }
    match svg_number_list(Some(value)).as_slice() {
        [value] => *value,
        _ => default,
    }
}
// Source: svg_parser_utilities.cc:42-174 and svg_number_list.cc:35-60.
// SVG number lists are transactional: one malformed token clears the entire
// value. Signs may start the next number without whitespace (for example
// `1-2`), while at most one comma is consumed as a delimiter.
fn svg_number_list(value: Option<&str>) -> Vec<f64> {
    let Some(input) = value else {
        return Vec::new();
    };
    let bytes = input.as_bytes();
    let is_svg_space = |byte| matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' ');
    let mut cursor = 0;
    let mut values = Vec::new();
    while cursor < bytes.len() {
        while cursor < bytes.len() && is_svg_space(bytes[cursor]) {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        let sign = if bytes[cursor] == b'-' {
            -1.0f32
        } else {
            1.0f32
        };
        if matches!(bytes[cursor], b'+' | b'-') {
            cursor += 1;
        }
        let integer_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        let integer_end = cursor;
        let mut digits = cursor - integer_start;
        // GenericParseNumber deliberately constructs the integer right-to-left
        // in FloatType. Preserve that operation order instead of using Rust's
        // correctly-rounded decimal parser, which differs at a few f32 edges.
        let mut integer = 0.0f32;
        let mut multiplier = 1.0f32;
        for digit in bytes[integer_start..integer_end].iter().rev() {
            integer += multiplier * f32::from(*digit - b'0');
            multiplier *= 10.0;
        }
        if !integer.is_finite() {
            return Vec::new();
        }
        let mut decimal = 0.0f32;
        if cursor < bytes.len() && bytes[cursor] == b'.' {
            cursor += 1;
            let fraction_start = cursor;
            let mut fraction = 1.0f32;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                fraction *= 0.1;
                decimal += f32::from(bytes[cursor] - b'0') * fraction;
                cursor += 1;
            }
            if cursor == fraction_start {
                return Vec::new();
            }
            digits += cursor - fraction_start;
        }
        if digits == 0 {
            return Vec::new();
        }
        let mut number = (integer + decimal) * sign;
        if cursor + 1 < bytes.len()
            && matches!(bytes[cursor], b'e' | b'E')
            && !matches!(bytes[cursor + 1], b'x' | b'm')
        {
            cursor += 1;
            let exponent_is_negative = cursor < bytes.len() && bytes[cursor] == b'-';
            if cursor < bytes.len() && matches!(bytes[cursor], b'+' | b'-') {
                cursor += 1;
            }
            let exponent_start = cursor;
            let mut exponent = 0.0f32;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                exponent *= 10.0;
                exponent += f32::from(bytes[cursor] - b'0');
                cursor += 1;
            }
            if cursor == exponent_start {
                return Vec::new();
            }
            if exponent_is_negative {
                exponent = -exponent;
            }
            if exponent > f32::MAX_10_EXP as f32 {
                return Vec::new();
            }
            if exponent != 0.0 {
                number *= 10.0f64.powf(f64::from(exponent)) as f32;
            }
        }
        if !number.is_finite() {
            return Vec::new();
        }
        values.push(f64::from(number));

        while cursor < bytes.len() && is_svg_space(bytes[cursor]) {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b',' {
            cursor += 1;
            while cursor < bytes.len() && is_svg_space(bytes[cursor]) {
                cursor += 1;
            }
        }
    }
    values
}

// Source: svg_number_optional_number.cc:64-80. Missing and malformed values
// retain the animated property's constructor initial; one value is duplicated.
fn svg_number_optional_number(value: Option<&str>) -> (f64, f64) {
    svg_number_optional_number_initial(value, 0.0)
}
fn svg_number_optional_number_initial(value: Option<&str>, initial: f64) -> (f64, f64) {
    let Some(value) = value else {
        return (initial, initial);
    };
    let values = svg_number_list(Some(value));
    match values.as_slice() {
        [value] => (*value, *value),
        [x, y] => (*x, *y),
        _ => (initial, initial),
    }
}

// Source: svg_preserve_aspect_ratio.cc:37-44,55-189. The parser commits its
// local values only after the whole attribute validates; malformed input thus
// leaves the constructor default, xMidYMid meet.
fn fe_image_preserve_aspect_ratio(value: Option<&str>) -> SvgPreserveAspectRatio {
    let default = SvgPreserveAspectRatio::default();
    let Some(value) = value else {
        return default;
    };
    if value.is_empty() {
        return default;
    }
    let bytes = value.as_bytes();
    let is_svg_space = |byte| matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' ');
    let mut start = 0;
    while start < bytes.len() && is_svg_space(bytes[start]) {
        start += 1;
    }
    let input = &value[start..];
    let (token, align) = if input.starts_with("none") {
        ("none", SvgPreserveAspectRatioAlign::kNone)
    } else {
        const ALIGNS: [(&str, SvgPreserveAspectRatioAlign); 9] = [
            ("xMinYMin", SvgPreserveAspectRatioAlign::kXMinYMin),
            ("xMidYMin", SvgPreserveAspectRatioAlign::kXMidYMin),
            ("xMaxYMin", SvgPreserveAspectRatioAlign::kXMaxYMin),
            ("xMinYMid", SvgPreserveAspectRatioAlign::kXMinYMid),
            ("xMidYMid", SvgPreserveAspectRatioAlign::kXMidYMid),
            ("xMaxYMid", SvgPreserveAspectRatioAlign::kXMaxYMid),
            ("xMinYMax", SvgPreserveAspectRatioAlign::kXMinYMax),
            ("xMidYMax", SvgPreserveAspectRatioAlign::kXMidYMax),
            ("xMaxYMax", SvgPreserveAspectRatioAlign::kXMaxYMax),
        ];
        let Some(value) = ALIGNS
            .into_iter()
            .find(|(token, _)| input.starts_with(token))
        else {
            return default;
        };
        value
    };
    let mut rest = &input[token.len()..];
    rest = rest.trim_start_matches(|character: char| {
        matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ')
    });
    let mut meet_or_slice = SvgMeetOrSlice::kMeet;
    if rest.starts_with("meet") {
        rest = &rest[4..];
    } else if rest.starts_with("slice") {
        rest = &rest[5..];
        if align != SvgPreserveAspectRatioAlign::kNone {
            meet_or_slice = SvgMeetOrSlice::kSlice;
        }
    }
    rest = rest.trim_start_matches(|character: char| {
        matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ')
    });
    if !rest.is_empty() {
        return default;
    }
    SvgPreserveAspectRatio {
        align,
        meet_or_slice,
    }
}
fn percentage_length(value: f64) -> SvgFilterLength {
    SvgFilterLength {
        value,
        percentage: true,
    }
}

// Sources: svg_length.cc:81-108,214-278 and svg_length_context.cc:163-209.
// The graph ABI can losslessly carry a resolved absolute length or a percentage.
// Context-dependent units and math expressions remain explicit unsupported
// values until the ABI also carries their font/viewport/container dependencies.
fn length(
    value: Option<&str>,
    invalid_initial: SvgFilterLength,
) -> Result<Option<SvgFilterLength>, String> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let raw = raw
        .trim_matches(|character: char| matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' '));
    if raw.is_empty() {
        return Ok(Some(invalid_initial));
    }
    let lower = raw.to_ascii_lowercase();
    if ["calc(", "min(", "max(", "clamp(", "var(", "env("]
        .into_iter()
        .any(|prefix| lower.starts_with(prefix))
    {
        return Err(format!("unsupported computed SVG filter length {raw}"));
    }
    const CONTEXT_UNITS: [&str; 36] = [
        "rem", "rex", "rch", "ric", "rlh", "cap", "rcap", "em", "ex", "ch", "ic", "lh", "vw", "vh",
        "vi", "vb", "vmin", "vmax", "svw", "svh", "svmin", "svmax", "lvw", "lvh", "lvmin", "lvmax",
        "dvw", "dvh", "dvmin", "dvmax", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
    ];
    if CONTEXT_UNITS.into_iter().any(|unit| {
        lower
            .strip_suffix(unit)
            .is_some_and(|number| css_length_number(number).is_some())
    }) {
        return Err(format!("unsupported relative SVG filter length {raw}"));
    }
    let (number, percentage, factor) = if let Some(number) = lower.strip_suffix('%') {
        (number, true, 1.0)
    } else if let Some(number) = lower.strip_suffix("px") {
        (number, false, 1.0)
    } else if let Some(number) = lower.strip_suffix("cm") {
        (number, false, 96.0 / 2.54)
    } else if let Some(number) = lower.strip_suffix("mm") {
        (number, false, 96.0 / 25.4)
    } else if let Some(number) = lower.strip_suffix('q') {
        (number, false, 96.0 / 101.6)
    } else if let Some(number) = lower.strip_suffix("in") {
        (number, false, 96.0)
    } else if let Some(number) = lower.strip_suffix("pt") {
        (number, false, 96.0 / 72.0)
    } else if let Some(number) = lower.strip_suffix("pc") {
        (number, false, 16.0)
    } else {
        (lower.as_str(), false, 1.0)
    };
    let Some(value) = css_length_number(number) else {
        return Ok(Some(invalid_initial));
    };
    let value = value * factor;
    // CSSPrimitiveValue::ConvertToLength clamps fixed lengths to the LayoutUnit
    // range. Percentages are stored as f32 by Length.
    let value = if percentage {
        value.clamp(-(f32::MAX as f64), f32::MAX as f64)
    } else {
        value.clamp(-33_554_430.0, 33_554_429.0)
    };
    Ok(Some(SvgFilterLength { value, percentage }))
}

fn css_length_number(value: &str) -> Option<f64> {
    let bytes = value.as_bytes();
    let mut cursor = 0;
    if cursor < bytes.len() && matches!(bytes[cursor], b'+' | b'-') {
        cursor += 1;
    }
    let integer_start = cursor;
    while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        cursor += 1;
    }
    let mut digits = cursor - integer_start;
    if cursor < bytes.len() && bytes[cursor] == b'.' {
        cursor += 1;
        let fraction_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor == fraction_start {
            return None;
        }
        digits += cursor - fraction_start;
    }
    if digits == 0 {
        return None;
    }
    if cursor < bytes.len() && matches!(bytes[cursor], b'e' | b'E') {
        cursor += 1;
        if cursor < bytes.len() && matches!(bytes[cursor], b'+' | b'-') {
            cursor += 1;
        }
        let exponent_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor == exponent_start {
            return None;
        }
    }
    if cursor != bytes.len() {
        return None;
    }
    value.parse::<f64>().ok().filter(|value| !value.is_nan())
}

fn region(element: FilterElement<'_>, filter_element: bool) -> Result<SvgFilterRegion, String> {
    let initials = if filter_element {
        [
            percentage_length(-10.0),
            percentage_length(-10.0),
            percentage_length(120.0),
            percentage_length(120.0),
        ]
    } else {
        [
            percentage_length(0.0),
            percentage_length(0.0),
            percentage_length(100.0),
            percentage_length(100.0),
        ]
    };
    Ok(SvgFilterRegion {
        x: length(attr(element, "x"), initials[0])?,
        y: length(attr(element, "y"), initials[1])?,
        width: length(attr(element, "width"), initials[2])?,
        height: length(attr(element, "height"), initials[3])?,
    })
}
fn input(value: Option<&str>) -> SvgFilterInput {
    match value.unwrap_or("") {
        "SourceGraphic" => SvgFilterInput::kSourceGraphic,
        "SourceAlpha" => SvgFilterInput::kSourceAlpha,
        "FillPaint" => SvgFilterInput::kFillPaint,
        "StrokePaint" => SvgFilterInput::kStrokePaint,
        value => SvgFilterInput::kNamed(value.to_owned()),
    }
}
fn edge_mode(value: Option<&str>, default: SvgFilterEdgeMode) -> SvgFilterEdgeMode {
    match value {
        Some("none") => SvgFilterEdgeMode::kNone,
        Some("duplicate") => SvgFilterEdgeMode::kDuplicate,
        Some("wrap") => SvgFilterEdgeMode::kWrap,
        _ => default,
    }
}
// Sources: svg_fe_convolve_matrix_element.cc:41-81,149-166;
// svg_integer_optional_integer.cc:64-82; svg_integer.cc:50-59. Invalid
// animated values are restored to their constructor initials by
// svg_animated_property.h:156-184 rather than poisoning the filter graph.
fn convolve_edge_mode(value: Option<&str>) -> SvgFilterEdgeMode {
    edge_mode(value, SvgFilterEdgeMode::kDuplicate)
}
fn convolve_order(value: Option<&str>) -> (u32, u32) {
    let Some(value) = value else {
        return (3, 3);
    };
    let (x, y) = svg_number_optional_number(Some(value));
    let (x, y) = (x as i32, y as i32);
    if x > 0 && y > 0 {
        (x as u32, y as u32)
    } else {
        (3, 3)
    }
}
// Source: svg_integer.cc:50-59. Only HTML spaces are stripped; an empty
// specified value is the valid integer zero, while any other parse failure is
// restored to the animated property's constructor initial.
fn svg_integer(value: Option<&str>, initial: i32) -> i32 {
    let Some(value) = value else {
        return initial;
    };
    if value.is_empty() {
        return 0;
    }
    value
        .trim_matches(|character: char| matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
        .parse()
        .unwrap_or(initial)
}
fn convolve_target(value: Option<&str>, default: i32) -> i32 {
    value.map_or(default, |value| svg_integer(Some(value), 0))
}
fn turbulence_octaves(value: Option<&str>) -> i32 {
    svg_integer(value, 1)
}
fn channel(value: Option<&str>, default: SvgFilterChannel) -> SvgFilterChannel {
    match value {
        Some("R") => SvgFilterChannel::kR,
        Some("G") => SvgFilterChannel::kG,
        Some("B") => SvgFilterChannel::kB,
        Some("A") => SvgFilterChannel::kA,
        _ => default,
    }
}
fn transfer_function(element: FilterElement<'_>) -> SvgTransferFunction {
    match attr(element, "type").unwrap_or("identity") {
        "identity" => SvgTransferFunction::kIdentity,
        "table" => SvgTransferFunction::kTable(svg_number_list(attr(element, "tableValues"))),
        "discrete" => SvgTransferFunction::kDiscrete(svg_number_list(attr(element, "tableValues"))),
        "linear" => SvgTransferFunction::kLinear {
            slope: number(attr(element, "slope"), 1.0),
            intercept: number(attr(element, "intercept"), 0.0),
        },
        "gamma" => SvgTransferFunction::kGamma {
            amplitude: number(attr(element, "amplitude"), 1.0),
            exponent: number(attr(element, "exponent"), 1.0),
            offset: number(attr(element, "offset"), 0.0),
        },
        _ => SvgTransferFunction::kIdentity,
    }
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
    let current = style.GetCurrentColor(None);
    let value = style
        .FloodColor()
        .Resolve(current, style.UsedColorScheme(), None);
    let (red, green, blue) = value.ToSRGB();
    Some(Color {
        red,
        green,
        blue,
        alpha: value.Alpha() * style.FloodOpacity().clamp(0.0, 1.0),
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
    let current = style.GetCurrentColor(None);
    let value = style
        .LightingColor()
        .Resolve(current, style.UsedColorScheme(), None);
    let (red, green, blue) = value.ToSRGB();
    Some(Color {
        red,
        green,
        blue,
        alpha: value.Alpha(),
    })
}
fn flood_color_taints_origin(style: Option<&ComputedStyle>) -> bool {
    style.is_some_and(|style| style.FloodColor().DependsOnCurrentColor())
}
fn lighting_color_taints_origin(style: Option<&ComputedStyle>) -> bool {
    style.is_some_and(|style| style.LightingColor().DependsOnCurrentColor())
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
    if let Some(style) = style {
        return style.ColorInterpolationFilters() == foundation::EColorInterpolation::kLinearrgb;
    }
    // Source: svg_filter_builder.cc:150-162. The no-layout path consults the
    // presentation-attribute set, not a hand-parsed inline style declaration.
    match attr(element, "color-interpolation-filters") {
        Some(value) if value.eq_ignore_ascii_case("linearrgb") => true,
        Some(value) if value.eq_ignore_ascii_case("srgb") || value.eq_ignore_ascii_case("auto") => {
            false
        }
        _ => inherited,
    }
}

// Sources: fe_flood.h:41-44 and svg_fe_image.h:43-45. These two effects
// deliberately ignore SVGFilterBuilder::SetOperatingInterpolationSpace and
// keep their produced color/image pixels in sRGB. Consumers still convert the
// input to their own operating space when they build the next effect.
fn primitive_linear_rgb(tag: &str, configured_linear_rgb: bool) -> bool {
    if matches!(tag, "feFlood" | "feImage") {
        false
    } else {
        configured_linear_rgb
    }
}
fn light_source(
    document: &FilterDocument<'_>,
    element: FilterElement<'_>,
) -> Option<SvgLightSource> {
    document.children(element).find_map(|index| {
        let light = document.element(index);
        if light.namespace() != DOMNamespace::kSVG {
            return None;
        }
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
                specular_exponent: number(attr(light, "specularExponent"), 1.0),
                limiting_cone_angle: number(attr(light, "limitingConeAngle"), 0.0),
            }),
            _ => None,
        }
    })
}
fn primitive(
    document: &FilterDocument<'_>,
    index: usize,
    inherited_linear_rgb: bool,
) -> Result<SvgFilterPrimitive, String> {
    let element = document.element(index);
    let first = input(attr(element, "in"));
    let kind = match element.tag() {
        "feGaussianBlur" => {
            // Source: svg_fe_gaussian_blur_element.cc:76-94. Chromium clamps
            // stdDeviation and always builds a decal blur; it does not expose
            // the proposed SVG 2 edgeMode attribute on this element.
            let (x, y) = svg_number_optional_number(attr(element, "stdDeviation"));
            let x = x.max(0.0);
            let y = y.max(0.0);
            SvgFilterPrimitiveKind::kGaussianBlur {
                sigma_x: x,
                sigma_y: y,
            }
        }
        "feColorMatrix" => {
            let ty = attr(element, "type").unwrap_or("matrix");
            let kind = match ty {
                "matrix" => SvgColorMatrixType::kMatrix,
                "saturate" => SvgColorMatrixType::kSaturate,
                "hueRotate" => SvgColorMatrixType::kHueRotate,
                "luminanceToAlpha" => SvgColorMatrixType::kLuminanceToAlpha,
                _ => SvgColorMatrixType::kMatrix,
            };
            SvgFilterPrimitiveKind::kColorMatrix {
                kind,
                values: svg_number_list(attr(element, "values")),
            }
        }
        "feOffset" => SvgFilterPrimitiveKind::kOffset {
            dx: number(attr(element, "dx"), 0.0),
            dy: number(attr(element, "dy"), 0.0),
        },
        "feDropShadow" => {
            // Source: svg_fe_drop_shadow_element.cc:37-46,102-123. Unlike
            // feGaussianBlur, all four numeric constructor initials are 2.
            let (sigma_x, sigma_y) =
                svg_number_optional_number_initial(attr(element, "stdDeviation"), 2.0);
            SvgFilterPrimitiveKind::kDropShadow {
                sigma_x: sigma_x.max(0.0),
                sigma_y: sigma_y.max(0.0),
                dx: number(attr(element, "dx"), 2.0),
                dy: number(attr(element, "dy"), 2.0),
                color: flood_color(element, document.style(index))?,
            }
        }
        "feFlood" => SvgFilterPrimitiveKind::kFlood {
            color: flood_color(element, document.style(index))?,
        },
        "feBlend" => {
            let mode = blend_mode(attr(element, "mode"));
            SvgFilterPrimitiveKind::kBlend {
                mode,
                input2: input(attr(element, "in2")),
            }
        }
        "feComposite" => {
            let operator = composite_operator(attr(element, "operator"));
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
                if node.namespace() == DOMNamespace::kSVG && node.tag() == "feMergeNode" {
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
                if node.namespace() != DOMNamespace::kSVG {
                    continue;
                }
                let slot = match node.tag() {
                    "feFuncR" => Some(0),
                    "feFuncG" => Some(1),
                    "feFuncB" => Some(2),
                    "feFuncA" => Some(3),
                    _ => None,
                };
                if let Some(slot) = slot {
                    functions[slot] = transfer_function(node);
                }
            }
            SvgFilterPrimitiveKind::kComponentTransfer { functions }
        }
        "feMorphology" => {
            let (x, y) = svg_number_optional_number(attr(element, "radius"));
            let x = x.max(0.0);
            let y = y.max(0.0);
            let operator = match attr(element, "operator").unwrap_or("erode") {
                "erode" => SvgMorphologyOperator::kErode,
                "dilate" => SvgMorphologyOperator::kDilate,
                _ => SvgMorphologyOperator::kErode,
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
            x_channel: channel(attr(element, "xChannelSelector"), SvgFilterChannel::kA),
            y_channel: channel(attr(element, "yChannelSelector"), SvgFilterChannel::kA),
        },
        "feTurbulence" => {
            let (x, y) = svg_number_optional_number(attr(element, "baseFrequency"));
            SvgFilterPrimitiveKind::kTurbulence {
                base_x: x,
                base_y: y,
                octaves: turbulence_octaves(attr(element, "numOctaves")),
                seed: number(attr(element, "seed"), 0.0),
                stitch: attr(element, "stitchTiles") == Some("stitch"),
                kind: match attr(element, "type").unwrap_or("turbulence") {
                    "turbulence" => SvgTurbulenceType::kTurbulence,
                    "fractalNoise" => SvgTurbulenceType::kFractalNoise,
                    _ => SvgTurbulenceType::kTurbulence,
                },
            }
        }
        "feTile" => SvgFilterPrimitiveKind::kTile,
        "feImage" => {
            let preserve_aspect_ratio =
                fe_image_preserve_aspect_ratio(attr(element, "preserveAspectRatio"));
            let href = attr(element, "href")
                .or_else(|| attr(element, "xlink:href"))
                .unwrap_or("")
                .to_owned();
            let local_reference = href.starts_with('#');
            let resource = if local_reference {
                None
            } else {
                document.image(&href)
            };
            // Source: svg_fe_image_element.cc:163-180. A local IRI becomes a
            // layout-subtree FEImage only when it resolves to an SVGElement.
            // Do not substitute an <image>'s external href: Chromium records
            // that element with its own geometry, transforms and style.
            let local_subtree = if local_reference {
                let id = href.trim_start_matches('#');
                find_id(document, index, id)
                    .map(|target| document.element(target).namespace() == DOMNamespace::kSVG)
                    .unwrap_or(false)
            } else {
                false
            };
            SvgFilterPrimitiveKind::kImage {
                local_subtree,
                preserve_aspect_ratio,
                resource_id: resource.map(|v| v.id),
                width: resource.map_or(0, |v| v.natural_width.max(0.0) as u32),
                height: resource.map_or(0, |v| v.natural_height.max(0.0) as u32),
                orientation: resource.map_or(Default::default(), |v| v.orientation),
                pixels: Arc::new(Vec::new()),
                href,
            }
        }
        "feConvolveMatrix" => {
            // Source: svg_fe_convolve_matrix_element.cc:219-233. Although the
            // DOM exposes kernelUnitLength, Build does not pass it into
            // FEConvolveMatrix; it is not a render-graph input in Chromium.
            let (ox, oy) = convolve_order(attr(element, "order"));
            let kernel = svg_number_list(attr(element, "kernelMatrix"));
            let divisor = attr(element, "divisor")
                .map(|_| number(attr(element, "divisor"), 1.0))
                .unwrap_or(0.0);
            SvgFilterPrimitiveKind::kConvolveMatrix {
                order_x: ox,
                order_y: oy,
                kernel,
                divisor,
                bias: number(attr(element, "bias"), 0.0),
                target_x: convolve_target(attr(element, "targetX"), (ox / 2) as i32),
                target_y: convolve_target(attr(element, "targetY"), (oy / 2) as i32),
                edge_mode: convolve_edge_mode(attr(element, "edgeMode")),
                preserve_alpha: attr(element, "preserveAlpha") == Some("true"),
            }
        }
        "feDiffuseLighting" | "feSpecularLighting" => {
            // Sources: svg_fe_diffuse_lighting_element.cc:127-150 and
            // svg_fe_specular_lighting_element.cc:139-163. kernelUnitLength is
            // a reflected DOM property but is not passed to either effect.
            let specular = element.tag() == "feSpecularLighting";
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
                ),
                specular_exponent: number(attr(element, "specularExponent"), 1.0),
            }
        }
        tag => return Err(format!("unsupported SVG filter primitive {tag}")),
    };
    let origin_tainted = match &kind {
        SvgFilterPrimitiveKind::kFlood { .. } | SvgFilterPrimitiveKind::kDropShadow { .. } => {
            flood_color_taints_origin(document.style(index))
        }
        SvgFilterPrimitiveKind::kLighting { .. } => {
            lighting_color_taints_origin(document.style(index))
        }
        SvgFilterPrimitiveKind::kImage { href, .. } => {
            href.starts_with('#')
                || document
                    .image(href)
                    .is_none_or(|image| !image.is_cors_same_origin)
        }
        _ => false,
    };
    Ok(SvgFilterPrimitive {
        input: first,
        result: attr(element, "result").unwrap_or("").to_owned(),
        region: region(element, false)?,
        // Source: svg_filter_builder.cc:197-200. A primitive without its own
        // resolved value inherits the filter element's interpolation value.
        linear_rgb: primitive_linear_rgb(
            element.tag(),
            linear_rgb(element, document.style(index), inherited_linear_rgb),
        ),
        origin_tainted,
        kind,
    })
}

// Source: svg_filter_builder.cc:182-188 and
// svg_filter_primitive_standard_attributes.h:83. This is a type predicate in
// Chromium, not a tag-prefix test: feMergeNode, feFunc*, light-source children
// and unknown `fe*` elements are not filter effects when placed directly under
// a filter element.
fn is_filter_effect(tag: &str) -> bool {
    matches!(
        tag,
        "feBlend"
            | "feColorMatrix"
            | "feComponentTransfer"
            | "feComposite"
            | "feConvolveMatrix"
            | "feDiffuseLighting"
            | "feDisplacementMap"
            | "feDropShadow"
            | "feFlood"
            | "feGaussianBlur"
            | "feImage"
            | "feMerge"
            | "feMorphology"
            | "feOffset"
            | "feSpecularLighting"
            | "feTile"
            | "feTurbulence"
    )
}
fn find_id(document: &FilterDocument<'_>, source: usize, id: &str) -> Option<usize> {
    let source = document.0.CorrespondingSVGElement(source).unwrap_or(source);
    document.0.FindElementByIdInTreeScope(source, id)
}

// Source: svg_fe_blend_element.cc:31-78. SVG Filter Effects accepts the
// complete non-Porter-Duff blend-mode set, not only the five SVG 1.1 modes.
fn blend_mode(value: Option<&str>) -> SvgBlendMode {
    match value.unwrap_or("normal") {
        "normal" => SvgBlendMode::kNormal,
        "multiply" => SvgBlendMode::kMultiply,
        "screen" => SvgBlendMode::kScreen,
        "darken" => SvgBlendMode::kDarken,
        "lighten" => SvgBlendMode::kLighten,
        "overlay" => SvgBlendMode::kOverlay,
        "color-dodge" => SvgBlendMode::kColorDodge,
        "color-burn" => SvgBlendMode::kColorBurn,
        "hard-light" => SvgBlendMode::kHardLight,
        "soft-light" => SvgBlendMode::kSoftLight,
        "difference" => SvgBlendMode::kDifference,
        "exclusion" => SvgBlendMode::kExclusion,
        "hue" => SvgBlendMode::kHue,
        "saturation" => SvgBlendMode::kSaturation,
        "color" => SvgBlendMode::kColor,
        "luminosity" => SvgBlendMode::kLuminosity,
        _ => SvgBlendMode::kNormal,
    }
}

// Source: svg_fe_composite_element.cc:35-49. `lighter` was added after SVG
// 1.1 and is intentionally not DOM-enumeration-exposed, but it is still an
// accepted attribute value and reaches FEComposite as SkBlendMode::kPlus.
fn composite_operator(value: Option<&str>) -> SvgCompositeOperator {
    match value.unwrap_or("over") {
        "over" => SvgCompositeOperator::kOver,
        "in" => SvgCompositeOperator::kIn,
        "out" => SvgCompositeOperator::kOut,
        "atop" => SvgCompositeOperator::kAtop,
        "xor" => SvgCompositeOperator::kXor,
        "arithmetic" => SvgCompositeOperator::kArithmetic,
        "lighter" => SvgCompositeOperator::kLighter,
        _ => SvgCompositeOperator::kOver,
    }
}

fn build_one(document: &FilterDocument<'_>, index: usize) -> SvgFilterGraph {
    let element = document.element(index);
    // Source: svg_filter_builder.cc:177-178. The manual external-SVG path
    // starts at `auto`; ResolveInterpolationSpace maps that to sRGB. A normal
    // layout-backed element still reads its computed initial (linearRGB).
    let filter_linear_rgb = linear_rgb(element, document.style(index), false);
    let units = |name, default| match attr(element, name) {
        Some("userSpaceOnUse") => SvgFilterUnits::kUserSpaceOnUse,
        Some("objectBoundingBox") | None => default,
        Some(_) => default,
    };
    let (filter_region, region_error) = match region(element, true) {
        Ok(region) => (region, None),
        Err(error) => (SvgFilterRegion::default(), Some(error)),
    };
    let mut graph = SvgFilterGraph {
        resource_node_id: element.0.Id(),
        id: attr(element, "id")
            .or(element.id())
            .unwrap_or("")
            .to_owned(),
        filter_units: units("filterUnits", SvgFilterUnits::kObjectBoundingBox),
        primitive_units: units("primitiveUnits", SvgFilterUnits::kUserSpaceOnUse),
        region: filter_region,
        primitives: Vec::new(),
        fill_paint: None,
        stroke_paint: None,
        unsupported: region_error,
    };
    // Source: svg_filter_builder.cc:172-205. Chromium currently builds only
    // this filter element's direct primitive children. SVGFilterElement exposes
    // SVGURIReference at the DOM surface, but filter href render inheritance is
    // not implemented (the corresponding WPT is an expected failure under
    // crbug.com/1184034), so do not synthesize inheritance here.
    for child in document.children(element) {
        let node = document.element(child);
        if node.namespace() != DOMNamespace::kSVG || !is_filter_effect(node.tag()) {
            continue;
        }
        match primitive(document, child, filter_linear_rgb) {
            Ok(value) => graph.primitives.push(value),
            Err(error) => {
                graph.unsupported.get_or_insert(error);
            }
        }
    }
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
        .map(|index| build_one(document, index))
        .collect();
    trace.set("filters", graphs.len() as f64);
    Arc::new(graphs)
}
// cpp: FilterEffectBuilder::BuildReferenceFilter / SVGFilterBuilder::BuildGraph.
// Read resource attributes and canonical computed styles directly from their
// DOM owner. The output alone is owned by the immutable paint snapshot.
pub(super) fn build_persistent(document: &crate::Document) -> Arc<Vec<SvgFilterGraph>> {
    document
        .SVGFilterGraphs()
        .cloned()
        .unwrap_or_else(|| build_graphs(&FilterDocument(document)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convolve_edge_mode_distinguishes_default_from_explicit_none() {
        assert_eq!(convolve_edge_mode(None), SvgFilterEdgeMode::kDuplicate);
        assert_eq!(convolve_edge_mode(Some("none")), SvgFilterEdgeMode::kNone);
        assert_eq!(
            convolve_edge_mode(Some("invalid")),
            SvgFilterEdgeMode::kDuplicate
        );
        assert_eq!(convolve_order(None), (3, 3));
        assert_eq!(convolve_order(Some("2 4")), (2, 4));
        assert_eq!(convolve_order(Some("2.5")), (2, 2));
        assert_eq!(convolve_order(Some("0 4")), (3, 3));
        assert_eq!(convolve_order(Some("invalid")), (3, 3));
        assert_eq!(convolve_target(Some("-1"), 4), -1);
        assert_eq!(convolve_target(Some("invalid"), 4), 0);
        assert_eq!(turbulence_octaves(None), 1);
        assert_eq!(turbulence_octaves(Some("")), 0);
        assert_eq!(turbulence_octaves(Some("0")), 0);
        assert_eq!(turbulence_octaves(Some("-2")), -2);
        assert_eq!(turbulence_octaves(Some("1.5")), 1);
        assert_eq!(turbulence_octaves(Some("invalid")), 1);
        assert_eq!(turbulence_octaves(Some("\u{00a0}2\u{00a0}")), 1);
        assert_eq!(convolve_target(Some("\t-2\r"), 4), -2);
        assert_eq!(convolve_target(Some("\u{00a0}2"), 4), 0);
    }

    #[test]
    fn blend_mode_accepts_the_full_chromium_svg_enumeration() {
        for (value, expected) in [
            ("normal", SvgBlendMode::kNormal),
            ("multiply", SvgBlendMode::kMultiply),
            ("screen", SvgBlendMode::kScreen),
            ("darken", SvgBlendMode::kDarken),
            ("lighten", SvgBlendMode::kLighten),
            ("overlay", SvgBlendMode::kOverlay),
            ("color-dodge", SvgBlendMode::kColorDodge),
            ("color-burn", SvgBlendMode::kColorBurn),
            ("hard-light", SvgBlendMode::kHardLight),
            ("soft-light", SvgBlendMode::kSoftLight),
            ("difference", SvgBlendMode::kDifference),
            ("exclusion", SvgBlendMode::kExclusion),
            ("hue", SvgBlendMode::kHue),
            ("saturation", SvgBlendMode::kSaturation),
            ("color", SvgBlendMode::kColor),
            ("luminosity", SvgBlendMode::kLuminosity),
        ] {
            assert_eq!(blend_mode(Some(value)), expected, "{value}");
        }
        assert_eq!(blend_mode(Some("plus-lighter")), SvgBlendMode::kNormal);
    }

    #[test]
    fn composite_operator_accepts_chromiums_lighter_extension() {
        assert_eq!(composite_operator(None), SvgCompositeOperator::kOver);
        assert_eq!(
            composite_operator(Some("lighter")),
            SvgCompositeOperator::kLighter
        );
        assert_eq!(
            composite_operator(Some("plus-lighter")),
            SvgCompositeOperator::kOver
        );
    }

    #[test]
    fn svg_number_lists_are_transactional_and_allow_adjacent_signs() {
        assert_eq!(svg_number_list(Some("1-2 .5e1")), vec![1.0, -2.0, 5.0]);
        assert_eq!(svg_number_list(Some("1, 2,")), vec![1.0, 2.0]);
        assert!(svg_number_list(Some("1 invalid 2")).is_empty());
        assert!(svg_number_list(Some("1,,2")).is_empty());
        assert!(svg_number_list(Some("1.")).is_empty());
        assert!(svg_number_list(Some("0e39")).is_empty());
        assert_eq!(svg_number_list(Some("1e-999")), vec![0.0]);
        assert_eq!(svg_number_optional_number(Some("3")), (3.0, 3.0));
        assert_eq!(svg_number_optional_number(Some("3 4")), (3.0, 4.0));
        assert_eq!(svg_number_optional_number(Some("3 4 5")), (0.0, 0.0));
        assert_eq!(svg_number_optional_number(Some("3 nope")), (0.0, 0.0));
        assert_eq!(number(None, 1.0), 1.0);
        assert_eq!(number(Some(""), 1.0), 0.0);
        assert_eq!(number(Some(".5"), 1.0), 0.5);
        assert_eq!(number(Some(" "), 1.0), 1.0);
        assert_eq!(number(Some("invalid"), 1.0), 1.0);
        assert_eq!(number(Some("1 2"), 1.0), 1.0);
    }

    #[test]
    fn fe_image_invalid_preserve_aspect_ratio_restores_default() {
        let default = SvgPreserveAspectRatio::default();
        assert_eq!(fe_image_preserve_aspect_ratio(None), default);
        assert_eq!(fe_image_preserve_aspect_ratio(Some("")), default);
        assert_eq!(fe_image_preserve_aspect_ratio(Some("garbage")), default);
        assert_eq!(
            fe_image_preserve_aspect_ratio(Some("none")),
            SvgPreserveAspectRatio {
                align: SvgPreserveAspectRatioAlign::kNone,
                meet_or_slice: SvgMeetOrSlice::kMeet,
            }
        );
        assert_eq!(
            fe_image_preserve_aspect_ratio(Some(" none slice ")),
            SvgPreserveAspectRatio {
                align: SvgPreserveAspectRatioAlign::kNone,
                meet_or_slice: SvgMeetOrSlice::kMeet,
            }
        );
        assert_eq!(
            fe_image_preserve_aspect_ratio(Some("xMinYMax meet")),
            SvgPreserveAspectRatio {
                align: SvgPreserveAspectRatioAlign::kXMinYMax,
                meet_or_slice: SvgMeetOrSlice::kMeet,
            }
        );
        assert_eq!(
            fe_image_preserve_aspect_ratio(Some("xMaxYMin slice")),
            SvgPreserveAspectRatio {
                align: SvgPreserveAspectRatioAlign::kXMaxYMin,
                meet_or_slice: SvgMeetOrSlice::kSlice,
            }
        );
        assert_eq!(
            fe_image_preserve_aspect_ratio(Some("xMidYMidmeet")),
            default
        );
        assert_eq!(
            fe_image_preserve_aspect_ratio(Some("xMinYMin junk")),
            default
        );
    }

    #[test]
    fn svg_filter_lengths_preserve_invalid_specification_and_units() {
        let initial = percentage_length(-10.0);
        assert_eq!(length(None, initial).unwrap(), None);
        assert_eq!(length(Some(""), initial).unwrap(), Some(initial));
        assert_eq!(length(Some("invalid"), initial).unwrap(), Some(initial));
        assert_eq!(
            length(Some("50%"), initial).unwrap(),
            Some(percentage_length(50.0))
        );
        assert_eq!(
            length(Some("12PX"), initial).unwrap(),
            Some(SvgFilterLength {
                value: 12.0,
                percentage: false,
            })
        );
        let centimeters = length(Some("2.54cm"), initial).unwrap().unwrap();
        assert!((centimeters.value - 96.0).abs() < 0.0001);
        assert_eq!(
            length(Some("1e100px"), initial).unwrap().unwrap().value,
            33_554_429.0
        );
        assert!(length(Some("1em"), initial).is_err());
        assert!(length(Some("calc(10px + 5%)"), initial).is_err());
    }

    #[test]
    fn svg_filter_input_names_are_exact_untrimmed_strings() {
        assert_eq!(input(None), SvgFilterInput::kNamed(String::new()));
        assert_eq!(input(Some("SourceAlpha")), SvgFilterInput::kSourceAlpha);
        assert_eq!(
            input(Some(" SourceAlpha ")),
            SvgFilterInput::kNamed(" SourceAlpha ".into())
        );
        assert_eq!(
            input(Some("BackgroundImage")),
            SvgFilterInput::kNamed("BackgroundImage".into())
        );
    }

    #[test]
    fn filter_effect_membership_is_an_exact_element_type_set() {
        assert!(is_filter_effect("feOffset"));
        assert!(is_filter_effect("feDropShadow"));
        assert!(!is_filter_effect("feMergeNode"));
        assert!(!is_filter_effect("feFuncR"));
        assert!(!is_filter_effect("feDistantLight"));
        assert!(!is_filter_effect("feWhatever"));
    }

    #[test]
    fn manual_color_interpolation_uses_parent_and_presentation_attribute() {
        use crate::persistent_document::{DOMAttribute, DOMNamespace};

        let mut owner = crate::DOM::new();
        let node = owner
            .GetDocumentMut()
            .CreateElementDefault(DOMNamespace::kSVG, "filter".into());
        let filter = FilterElement(owner.GetDocument().Node(node));
        // SVGFilterBuilder starts the filter element's no-layout path at auto,
        // which ResolveInterpolationSpace maps to sRGB.
        assert!(!linear_rgb(filter, None, false));
        assert!(linear_rgb(filter, None, true));

        owner.GetDocumentMut().SetAttribute(
            node,
            DOMAttribute {
                local_name: "color-interpolation-filters".into(),
                value: "linearRGB".into(),
                ..Default::default()
            },
        );
        assert!(linear_rgb(
            FilterElement(owner.GetDocument().Node(node)),
            None,
            false
        ));
        owner.GetDocumentMut().SetAttribute(
            node,
            DOMAttribute {
                local_name: "color-interpolation-filters".into(),
                value: "invalid".into(),
                ..Default::default()
            },
        );
        assert!(!linear_rgb(
            FilterElement(owner.GetDocument().Node(node)),
            None,
            false
        ));
    }

    #[test]
    fn flood_and_image_ignore_configured_operating_interpolation_space() {
        assert!(!primitive_linear_rgb("feFlood", true));
        assert!(!primitive_linear_rgb("feImage", true));
        assert!(primitive_linear_rgb("feBlend", true));
        assert!(!primitive_linear_rgb("feBlend", false));
    }
}

#![allow(non_snake_case)]

use super::{number::Number, selector::SourceSpace};
use css_parser::length_percentage_parser::{AbsoluteLengthScale, ParseLengthPercentage};
use layoutng_assembly::internal::layout_input::ComputedStyle;
use layoutng_assembly::internal::paint_input::{PaintCornerRadii, PaintCornerRadius};

// cpp: style_resolver/style_resolver.cc:1254-1273
pub(crate) fn SplitTopLevel(value: &str, delimiter: u8) -> Vec<String> {
    let bytes = value.as_bytes();
    let mut result = Vec::new();
    let mut begin = 0;
    let mut depth = 0;
    let mut quote = 0;
    for i in 0..=bytes.len() {
        let character = bytes.get(i).copied().unwrap_or(delimiter);
        if quote != 0 {
            if character == quote && (i == 0 || bytes[i - 1] != b'\\') {
                quote = 0;
            }
        } else if matches!(character, b'\'' | b'"') {
            quote = character;
        } else if character == b'(' {
            depth += 1;
        } else if character == b')' {
            depth -= 1;
        } else if character == delimiter && depth == 0 {
            result.push(
                value[begin..i]
                    .trim_matches(|c: char| c.source_space())
                    .to_owned(),
            );
            begin = i + 1;
        }
    }
    result
}

// cpp: style_resolver/style_resolver.cc:842-888
pub(crate) fn Length(text: &str, font_size: f64) -> Option<f64> {
    let value = text
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    for function in ["min(", "max(", "clamp("] {
        if let Some(inner) = value
            .strip_prefix(function)
            .and_then(|rest| rest.strip_suffix(')'))
        {
            let arguments = SplitTopLevel(inner, b',');
            if arguments.is_empty() || (function == "clamp(" && arguments.len() != 3) {
                return None;
            }
            let lengths: Vec<f64> = arguments
                .iter()
                .map(|argument| Length(argument, font_size))
                .collect::<Option<_>>()?;
            return Some(match function {
                "min(" => lengths.into_iter().reduce(f64::min)?,
                "max(" => lengths.into_iter().reduce(f64::max)?,
                _ => lengths[0].max(lengths[1].min(lengths[2])),
            });
        }
    }
    if let Some(calculated) = ParseLengthPercentage(&value, font_size) {
        if !calculated.has_percentage {
            return Some(calculated.pixels);
        }
    }
    if let Some(number) = value.strip_suffix("rem").and_then(Number) {
        return Some(number * 16.0);
    }
    if let Some(number) = value.strip_suffix("em").and_then(Number) {
        return Some(number * font_size);
    }
    for unit in ["px", "in", "cm", "mm", "pt", "pc", "q"] {
        if let Some(number) = value.strip_suffix(unit).and_then(Number) {
            return Some(number * AbsoluteLengthScale(unit.as_bytes())?);
        }
    }
    if Number(&value) == Some(0.0) {
        return Some(0.0);
    }
    None
}

// cpp: style_resolver/style_resolver.cc:905-909
fn Percentage(text: &str) -> Option<f64> {
    text.trim().strip_suffix('%')?.trim().parse::<f64>().ok()
}

// cpp: style_resolver/style_resolver.cc:2016-2021
fn ExpandFour(values: &[f64]) -> [f64; 4] {
    [
        values[0],
        *values.get(1).unwrap_or(&values[0]),
        *values.get(2).unwrap_or(&values[0]),
        *values.get(3).unwrap_or(values.get(1).unwrap_or(&values[0])),
    ]
}

// cpp: style_resolver/style_resolver.cc:2187-2195
#[derive(Clone, Copy)]
struct ParsedCornerRadii {
    pixels: PaintCornerRadii,
    percentages: PaintCornerRadii,
}

#[derive(Clone, Copy)]
struct ParsedCornerRadius {
    pixels: PaintCornerRadius,
    percentages: PaintCornerRadius,
}

// cpp: style_resolver/style_resolver.cc:2197-2233
fn ParseBorderRadii(value: &str, font_size: f64) -> Option<ParsedCornerRadii> {
    let axes = SplitTopLevel(value, b'/');
    if axes.is_empty() || axes.len() > 2 {
        return None;
    }
    let parse_axis = |axis: &str| -> Option<([f64; 4], [f64; 4])> {
        let parts: Vec<&str> = axis.split_ascii_whitespace().collect();
        if parts.is_empty() || parts.len() > 4 {
            return None;
        }
        let mut pixels = Vec::new();
        let mut percentages = Vec::new();
        for part in parts {
            if let Some(radius) = Length(part, font_size).filter(|radius| *radius >= 0.0) {
                pixels.push(radius);
                percentages.push(0.0);
            } else if let Some(radius) = Percentage(part).filter(|radius| *radius >= 0.0) {
                pixels.push(0.0);
                percentages.push(radius);
            } else {
                return None;
            }
        }
        Some((ExpandFour(&pixels), ExpandFour(&percentages)))
    };
    let horizontal = parse_axis(&axes[0])?;
    let vertical = if axes.len() == 1 {
        horizontal
    } else {
        parse_axis(&axes[1])?
    };
    let corners = |x: [f64; 4], y: [f64; 4]| PaintCornerRadii {
        top_left: PaintCornerRadius { x: x[0], y: y[0] },
        top_right: PaintCornerRadius { x: x[1], y: y[1] },
        bottom_right: PaintCornerRadius { x: x[2], y: y[2] },
        bottom_left: PaintCornerRadius { x: x[3], y: y[3] },
    };
    Some(ParsedCornerRadii {
        pixels: corners(horizontal.0, vertical.0),
        percentages: corners(horizontal.1, vertical.1),
    })
}

// cpp: style_resolver/style_resolver.cc:2235-2257
fn ParseCornerRadius(value: &str, font_size: f64) -> Option<ParsedCornerRadius> {
    let parts: Vec<&str> = value.split_ascii_whitespace().collect();
    if parts.is_empty() || parts.len() > 2 {
        return None;
    }
    let parse = |part: &str| -> Option<(f64, f64)> {
        if let Some(length) = Length(part, font_size).filter(|length| *length >= 0.0) {
            return Some((length, 0.0));
        }
        Percentage(part)
            .filter(|percentage| *percentage >= 0.0)
            .map(|percentage| (0.0, percentage))
    };
    let horizontal = parse(parts[0])?;
    let vertical = if parts.len() == 1 {
        horizontal
    } else {
        parse(parts[1])?
    };
    Some(ParsedCornerRadius {
        pixels: PaintCornerRadius {
            x: horizontal.0,
            y: vertical.0,
        },
        percentages: PaintCornerRadius {
            x: horizontal.1,
            y: vertical.1,
        },
    })
}

// cpp: style_resolver/style_resolver.cc:2259-2263
fn CurrentCornerRadii(style: &ComputedStyle) -> PaintCornerRadii {
    if let Some(radii) = style.paint.border_radii {
        return radii;
    }
    let radius = PaintCornerRadius {
        x: style.paint.border_radius,
        y: style.paint.border_radius,
    };
    PaintCornerRadii {
        top_left: radius,
        top_right: radius,
        bottom_right: radius,
        bottom_left: radius,
    }
}

// cpp: style_resolver/style_resolver.cc:6104-6131
pub fn ApplyBorderRadius(style: &mut ComputedStyle, property: &str, value: &str) {
    let font_size = style
        .extended
        .as_ref()
        .map_or(16.0, |extended| extended.font_size);
    if property == "border-radius" {
        if let Some(radii) = ParseBorderRadii(value, font_size) {
            style.paint.border_radii = Some(radii.pixels);
            style.paint.border_radii_percentages = radii.percentages;
            style.paint.border_radius = 0.0;
        }
        return;
    }
    if let Some(radius) = ParseCornerRadius(value, font_size) {
        let mut radii = CurrentCornerRadii(style);
        let mut percentages = style.paint.border_radii_percentages;
        let target = match property {
            "border-top-left-radius" => (&mut radii.top_left, &mut percentages.top_left),
            "border-top-right-radius" => (&mut radii.top_right, &mut percentages.top_right),
            "border-bottom-right-radius" => {
                (&mut radii.bottom_right, &mut percentages.bottom_right)
            }
            "border-bottom-left-radius" => (&mut radii.bottom_left, &mut percentages.bottom_left),
            _ => return,
        };
        *target.0 = radius.pixels;
        *target.1 = radius.percentages;
        style.paint.border_radii = Some(radii);
        style.paint.border_radii_percentages = percentages;
        style.paint.border_radius = 0.0;
    }
}

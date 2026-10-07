#![allow(non_snake_case)]

use super::border_radius::Length;
use super::shadow::SplitWhitespace;
use layoutng_assembly::css_color_parser::ParseCSSColor;
use layoutng_assembly::internal::layout_input::{BorderLineStyle, ComputedStyle};
use layoutng_assembly::internal::layout_input_types::Color;

// cpp: style_resolver/style_resolver.cc:1991-2003
pub(crate) fn ParseBorderLineStyle(value: &str) -> Option<BorderLineStyle> {
    Some(match value {
        "none" => BorderLineStyle::kNone,
        "hidden" => BorderLineStyle::kHidden,
        "solid" => BorderLineStyle::kSolid,
        "dashed" => BorderLineStyle::kDashed,
        "dotted" => BorderLineStyle::kDotted,
        "double" => BorderLineStyle::kDouble,
        "groove" => BorderLineStyle::kGroove,
        "ridge" => BorderLineStyle::kRidge,
        "inset" => BorderLineStyle::kInset,
        "outset" => BorderLineStyle::kOutset,
        _ => return None,
    })
}

// cpp: style_resolver/style_resolver.cc:2005-2013
pub(crate) fn ParseBorderWidth(value: &str, font_size: f64) -> Option<f64> {
    match value {
        "thin" => Some(1.0),
        "medium" => Some(3.0),
        "thick" => Some(5.0),
        _ => Length(value, font_size).filter(|width| *width >= 0.0),
    }
}

// cpp: style_resolver/style_resolver.cc:2090-2136
pub(crate) struct ParsedBorder {
    pub width: f64,
    pub style: BorderLineStyle,
    pub color: Color,
}
pub(crate) fn ParseBorder(
    value: &str,
    font_size: f64,
    current_color: Color,
) -> Option<ParsedBorder> {
    let mut result = ParsedBorder {
        width: 3.0,
        style: BorderLineStyle::kNone,
        color: current_color,
    };
    let (mut saw_width, mut saw_style, mut saw_color) = (false, false, false);
    let parts = SplitWhitespace(value);
    if parts.is_empty() {
        return None;
    }
    for part in &parts {
        if !saw_width {
            if let Some(width) = ParseBorderWidth(part, font_size) {
                result.width = width;
                saw_width = true;
                continue;
            }
        }
        if !saw_style {
            if let Some(style) = ParseBorderLineStyle(part) {
                result.style = style;
                saw_style = true;
                continue;
            }
        }
        if !saw_color {
            if let Some(color) = ParseBorderColor(part, current_color) {
                result.color = color;
                saw_color = true;
                continue;
            }
        }
        return None;
    }
    Some(result)
}

// cpp: style_resolver/style_resolver.cc:4921-4973
pub(crate) fn ApplyBorder(style: &mut ComputedStyle, property: &str, value: &str, font_size: f64) {
    let current = style.paint.color;
    match property {
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" => {
            if let Some(parsed) = ParseBorder(value, font_size, current) {
                let range = BorderSide(property).map_or(0..4, |side| side..side + 1);
                for side in range {
                    match side {
                        0 => style.border.top = parsed.width,
                        1 => style.border.right = parsed.width,
                        2 => style.border.bottom = parsed.width,
                        _ => style.border.left = parsed.width,
                    };
                    style.border_styles[side] = parsed.style;
                    style.paint.border_colors[side] = parsed.color;
                }
            }
        }
        "column-rule" | "outline" => {
            if let Some(parsed) = ParseBorder(value, font_size, current) {
                if property == "column-rule" {
                    style.paint.column_rule_width = parsed.width;
                    style.paint.column_rule_style = parsed.style;
                    style.paint.column_rule_color = parsed.color;
                } else {
                    style.paint.outline_width = parsed.width;
                    style.paint.outline_style = parsed.style;
                    style.paint.outline_color = parsed.color;
                }
            }
        }
        "column-rule-width" => {
            if let Some(width) = ParseBorderWidth(value, font_size) {
                style.paint.column_rule_width = width;
            }
        }
        "column-rule-style" => {
            if let Some(line) = ParseBorderLineStyle(value) {
                style.paint.column_rule_style = line;
            }
        }
        "column-rule-color" => {
            if let Some(color) = ParseBorderColor(value, current) {
                style.paint.column_rule_color = color;
            }
        }
        "outline-width" => {
            if let Some(width) = ParseBorderWidth(value, font_size) {
                style.paint.outline_width = width;
            }
        }
        "outline-style" => {
            if let Some(line) = ParseBorderLineStyle(value) {
                style.paint.outline_style = line;
            }
        }
        "outline-color" => {
            if let Some(color) = ParseBorderColor(value, current) {
                style.paint.outline_color = color;
            }
        }
        "outline-offset" => {
            if let Some(offset) = Length(value, font_size) {
                style.paint.outline_offset = offset;
            }
        }
        _ => {}
    }
}

// cpp: style_resolver/style_resolver.cc:2015-2022
fn ExpandFour<T: Copy>(values: &[T]) -> [T; 4] {
    [
        values[0],
        *values.get(1).unwrap_or(&values[0]),
        *values.get(2).unwrap_or(&values[0]),
        *values.get(3).unwrap_or(values.get(1).unwrap_or(&values[0])),
    ]
}

// cpp: style_resolver/style_resolver.cc:2024-2050
fn ParseFour<T: Copy>(value: &str, mut parse: impl FnMut(&str) -> Option<T>) -> Option<[T; 4]> {
    let parts = SplitWhitespace(value);
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    let parsed = parts
        .iter()
        .map(|part| parse(part))
        .collect::<Option<Vec<T>>>()?;
    Some(ExpandFour(&parsed))
}

// cpp: style_resolver/style_resolver.cc:2052-2068
fn ParseBorderColor(value: &str, current_color: Color) -> Option<Color> {
    if value == "currentcolor" {
        Some(current_color)
    } else {
        ParseCSSColor(value)
    }
}

// cpp: style_resolver/style_resolver.cc:2081-2094
pub(crate) fn BorderSide(property: &str) -> Option<usize> {
    if property.starts_with("border-top") {
        Some(0)
    } else if property.starts_with("border-right") {
        Some(1)
    } else if property.starts_with("border-bottom") {
        Some(2)
    } else if property.starts_with("border-left") {
        Some(3)
    } else {
        None
    }
}

// cpp: style_resolver/style_resolver.cc:4890-4920
pub fn ApplyBorderLonghand(style: &mut ComputedStyle, property: &str, value: &str, font_size: f64) {
    let side = BorderSide(property);
    match property {
        "border-width" => {
            if let Some(widths) = ParseFour(value, |part| ParseBorderWidth(part, font_size)) {
                style.border.top = widths[0];
                style.border.right = widths[1];
                style.border.bottom = widths[2];
                style.border.left = widths[3];
            }
        }
        "border-style" => {
            if let Some(styles) = ParseFour(value, ParseBorderLineStyle) {
                style.border_styles = styles;
            }
        }
        "border-color" => {
            let current = style.paint.color;
            if let Some(colors) = ParseFour(value, |part| ParseBorderColor(part, current)) {
                style.paint.border_colors = colors;
            }
        }
        _ if property.ends_with("-width") => {
            if let (Some(side), Some(width)) = (side, ParseBorderWidth(value, font_size)) {
                match side {
                    0 => style.border.top = width,
                    1 => style.border.right = width,
                    2 => style.border.bottom = width,
                    _ => style.border.left = width,
                }
            }
        }
        _ if property.ends_with("-style") => {
            if let (Some(side), Some(line_style)) = (side, ParseBorderLineStyle(value)) {
                style.border_styles[side] = line_style;
            }
        }
        _ if property.ends_with("-color") => {
            if let (Some(side), Some(color)) = (side, ParseBorderColor(value, style.paint.color)) {
                style.paint.border_colors[side] = color;
            }
        }
        _ => unreachable!("not a border longhand"),
    }
}

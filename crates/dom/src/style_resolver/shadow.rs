#![allow(non_snake_case)]

use super::border_radius::{Length, SplitTopLevel};
use layoutng_assembly::internal::layout_input::{ComputedStyle, Offset, PaintShadow};
use layoutng_assembly::internal::layout_input_types::Color;

// cpp: style_resolver/style_resolver.cc:50-69
pub(crate) fn SplitWhitespace(value: &str) -> Vec<String> {
    let bytes = value.as_bytes();
    let mut result = Vec::new();
    let mut begin = 0;
    let mut depth = 0;
    for i in 0..=bytes.len() {
        let character = bytes.get(i).copied().unwrap_or(b' ');
        if character == b'(' {
            depth += 1;
        } else if character == b')' {
            depth -= 1;
        }
        if depth == 0 && crate::style_resolver::selector::SourceSpace::source_space(character) {
            if i > begin {
                result.push(value[begin..i].to_owned());
            }
            begin = i + 1;
        }
    }
    result
}

// cpp: style_resolver/style_resolver.cc:2138-2185
fn ParseShadows(
    value: &str,
    font_size: f64,
    current_color: Color,
    box_shadow: bool,
) -> Option<Vec<PaintShadow>> {
    if value == "none" {
        return Some(Vec::new());
    }
    let mut shadows = Vec::new();
    for layer in SplitTopLevel(value, b',') {
        if layer.is_empty() {
            return None;
        }
        let mut shadow = PaintShadow {
            color: current_color,
            ..PaintShadow::default()
        };
        let mut saw_color = false;
        let mut saw_inset = false;
        let mut lengths = Vec::new();
        for part in SplitWhitespace(&layer) {
            if part == "inset" {
                if !box_shadow || saw_inset {
                    return None;
                }
                shadow.inset = true;
                saw_inset = true;
                continue;
            }
            if !saw_color {
                if part == "currentcolor" {
                    shadow.color = current_color;
                    saw_color = true;
                    continue;
                }
                if let Some(color) = layoutng_assembly::css_color_parser::ParseCSSColor(&part) {
                    shadow.color = color;
                    saw_color = true;
                    continue;
                }
            }
            lengths.push(Length(&part, font_size)?);
        }
        let maximum_lengths = if box_shadow { 4 } else { 3 };
        if lengths.len() < 2
            || lengths.len() > maximum_lengths
            || (lengths.len() >= 3 && lengths[2] < 0.0)
        {
            return None;
        }
        shadow.offset = Offset {
            x: lengths[0],
            y: lengths[1],
        };
        if lengths.len() >= 3 {
            shadow.blur_radius = lengths[2];
        }
        if lengths.len() == 4 {
            shadow.spread = lengths[3];
        }
        shadows.push(shadow);
    }
    Some(shadows)
}

// cpp: style_resolver/style_resolver.cc:6133-6139
pub fn ApplyShadow(style: &mut ComputedStyle, property: &str, value: &str) {
    let font_size = style
        .extended
        .as_ref()
        .map_or(16.0, |extended| extended.font_size);
    let box_shadow = property == "box-shadow";
    if let Some(shadows) = ParseShadows(value, font_size, style.paint.color, box_shadow) {
        if box_shadow {
            style.paint.box_shadows = shadows;
        } else {
            style.paint.text_shadows = shadows;
        }
    }
}

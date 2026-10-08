#![allow(non_snake_case)]

use super::border_radius::Length;
use crate::style_resolver::{number::Number, selector::SourceSpace, shadow::SplitWhitespace};
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::layout_input::ComputedStyle;

// cpp: style_resolver/style_resolver.cc:3556-3561
#[derive(Clone, Copy, Default)]
pub(crate) struct EdgeValue {
    pixels: f64,
    percentage: Option<f64>,
    automatic: bool,
    calculated: bool,
    quirk: bool,
}

// cpp: style_resolver/style_resolver.cc:3597-3621
pub(crate) fn ParseEdgeValue(input: &str, font_size: f64, margin: bool) -> Option<EdgeValue> {
    let value = input
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    if margin && value == "auto" {
        return Some(EdgeValue {
            automatic: true,
            ..EdgeValue::default()
        });
    }
    // Blink's built-in html.css uses the UA-only `__qem` unit to retain the
    // historical margin-collapse quirk on headings, paragraphs and lists.
    // Author CSS cannot spell this through a parsed author stylesheet.
    if margin {
        if let Some(number) = value.strip_suffix("__qem").and_then(Number) {
            return Some(EdgeValue {
                pixels: number * font_size,
                quirk: true,
                ..EdgeValue::default()
            });
        }
    }
    if let Some(calculated) = ParseLengthPercentage(&value, font_size) {
        return Some(EdgeValue {
            pixels: if calculated.has_pixels {
                calculated.pixels
            } else {
                0.0
            },
            percentage: calculated.has_percentage.then_some(calculated.percentage),
            calculated: true,
            ..EdgeValue::default()
        });
    }
    if let Some(percentage) = value.strip_suffix('%').and_then(Number) {
        if margin || percentage >= 0.0 {
            return Some(EdgeValue {
                percentage: Some(percentage),
                ..EdgeValue::default()
            });
        }
    }
    if let Some(pixels) = Length(&value, font_size) {
        if margin || pixels >= 0.0 {
            return Some(EdgeValue {
                pixels,
                ..EdgeValue::default()
            });
        }
    }
    None
}

// cpp: style_resolver/style_resolver.cc:3623-3644
fn ParseEdgeValues(input: &str, font_size: f64, margin: bool) -> Option<[EdgeValue; 4]> {
    let parts = SplitWhitespace(input);
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }
    let parsed: Vec<EdgeValue> = parts
        .iter()
        .map(|part| ParseEdgeValue(part, font_size, margin))
        .collect::<Option<_>>()?;
    Some([
        parsed[0],
        *parsed.get(1).unwrap_or(&parsed[0]),
        *parsed.get(2).unwrap_or(&parsed[0]),
        *parsed.get(3).unwrap_or(parsed.get(1).unwrap_or(&parsed[0])),
    ])
}

// cpp: style_resolver/style_resolver.cc:3817-3839
fn SetEdgeValue(style: &mut ComputedStyle, margin: bool, side: usize, value: EdgeValue) {
    let edge = if margin {
        &mut style.margin
    } else {
        &mut style.padding
    };
    match side {
        0 => edge.top = value.pixels,
        1 => edge.right = value.pixels,
        2 => edge.bottom = value.pixels,
        _ => edge.left = value.pixels,
    }
    let extra = style.extended.get_or_insert_with(Default::default);
    if margin {
        extra.margin_percentages[side] = value.percentage;
        extra.margin_calculated[side] = value.calculated;
        extra.margin_auto[side] = value.automatic;
        extra.margin_quirks[side] = value.quirk;
    } else {
        extra.padding_percentages[side] = value.percentage;
        extra.padding_calculated[side] = value.calculated;
    }
}

// cpp: style_resolver/style_resolver.cc:4879-4889
pub fn ApplyEdges(style: &mut ComputedStyle, property: &str, value: &str) {
    let margin = property.starts_with("margin");
    let font_size = style
        .extended
        .as_ref()
        .map_or(16.0, |extra| extra.font_size);
    if property == "margin" || property == "padding" {
        if let Some(edges) = ParseEdgeValues(value, font_size, margin) {
            for (side, edge) in edges.into_iter().enumerate() {
                SetEdgeValue(style, margin, side, edge);
            }
        }
    } else {
        let side = if property.ends_with("-top") {
            0
        } else if property.ends_with("-right") {
            1
        } else if property.ends_with("-bottom") {
            2
        } else {
            3
        };
        if let Some(edge) = ParseEdgeValue(value, font_size, margin) {
            SetEdgeValue(style, margin, side, edge);
        }
    }
}

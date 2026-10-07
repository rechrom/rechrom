#![allow(non_snake_case)]

use super::border_radius::Length;
use super::shadow::SplitWhitespace;
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::layout_input::ComputedStyle;

// cpp: style_resolver/style_resolver.cc:3563-3567
#[derive(Clone, Copy, Default)]
struct GapValue {
    pixels: Option<f64>,
    percentage: Option<f64>,
    calculated: bool,
}

// cpp: style_resolver/style_resolver.cc:3569-3587
fn ParseGapValue(input: &str, font_size: f64) -> Option<GapValue> {
    let value = input.trim().to_ascii_lowercase();
    if value == "normal" {
        return Some(GapValue::default());
    }
    if let Some(calculated) = ParseLengthPercentage(&value, font_size) {
        return Some(GapValue {
            pixels: calculated.has_pixels.then_some(calculated.pixels),
            percentage: calculated.has_percentage.then_some(calculated.percentage),
            calculated: true,
        });
    }
    if let Some(percentage) = value
        .strip_suffix('%')
        .and_then(|part| part.parse::<f64>().ok())
        .filter(|number| *number >= 0.0)
    {
        return Some(GapValue {
            percentage: Some(percentage),
            ..GapValue::default()
        });
    }
    Length(&value, font_size)
        .filter(|number| *number >= 0.0)
        .map(|pixels| GapValue {
            pixels: Some(pixels),
            ..GapValue::default()
        })
}

// cpp: style_resolver/style_resolver.cc:3589-3595
fn SetGapValue(style: &mut ComputedStyle, row: bool, value: GapValue) {
    let extra = style.extended.get_or_insert_with(Default::default);
    if row {
        extra.row_gap = value.pixels;
        extra.row_gap_percent = value.percentage;
        extra.row_gap_calculated = value.calculated;
    } else {
        extra.column_gap = value.pixels;
        extra.column_gap_percent = value.percentage;
        extra.column_gap_calculated = value.calculated;
    }
}

// cpp: style_resolver/style_resolver.cc:4988-5005
pub fn ApplyGap(style: &mut ComputedStyle, property: &str, value: &str, font_size: f64) {
    if property == "gap" {
        let parts = SplitWhitespace(value);
        if parts.len() != 1 && parts.len() != 2 {
            return;
        }
        let Some(row) = ParseGapValue(&parts[0], font_size) else {
            return;
        };
        let column = if parts.len() == 1 {
            row
        } else {
            let Some(column) = ParseGapValue(&parts[1], font_size) else {
                return;
            };
            column
        };
        style.gap = if row.pixels.is_some() && row.pixels == column.pixels {
            row.pixels.unwrap()
        } else {
            0.0
        };
        SetGapValue(style, true, row);
        SetGapValue(style, false, column);
    } else if let Some(gap) = ParseGapValue(value, font_size) {
        SetGapValue(style, property == "row-gap", gap);
    }
}

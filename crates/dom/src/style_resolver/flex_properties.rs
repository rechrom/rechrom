#![allow(non_snake_case)]

use super::border_radius::Length;
use super::shadow::SplitWhitespace;
use super::{number::Number, selector::SourceSpace};
use layoutng_assembly::internal::layout_input::{ComputedStyle, FlexBasisSizing, FlexDirection};

// cpp: style_resolver/style_resolver.cc:1903-1936
pub(crate) fn SetFlexBasis(style: &mut ComputedStyle, value: &str, font_size: f64) -> bool {
    let normalized = value
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    let value = normalized.as_str();
    style.flex_basis = None;
    let extra = style.extended.get_or_insert_with(Default::default);
    extra.flex_basis_percent = None;
    extra.flex_basis_calculated = false;
    extra.flex_basis_sizing = FlexBasisSizing::kAuto;
    match value {
        "auto" => return true,
        "content" => extra.flex_basis_sizing = FlexBasisSizing::kContent,
        "min-content" => extra.flex_basis_sizing = FlexBasisSizing::kMinContent,
        "max-content" => extra.flex_basis_sizing = FlexBasisSizing::kMaxContent,
        "fit-content" => extra.flex_basis_sizing = FlexBasisSizing::kFitContent,
        _ => {
            if let Some(length) =
                css_parser::length_percentage_parser::ParseLengthPercentage(value, font_size)
            {
                if length.has_pixels {
                    style.flex_basis = Some(length.pixels);
                }
                if length.has_percentage {
                    extra.flex_basis_percent = Some(length.percentage);
                }
                extra.flex_basis_calculated = true;
                return true;
            }
            if let Some(percent) = value
                .strip_suffix('%')
                .and_then(Number)
                .filter(|number| *number >= 0.0)
            {
                extra.flex_basis_percent = Some(percent);
                return true;
            }
            if let Some(length) = Length(value, font_size).filter(|number| *number >= 0.0) {
                style.flex_basis = Some(length);
                return true;
            }
            return false;
        }
    }
    true
}

// cpp: style_resolver/style_resolver.cc:5023-5063
pub fn ApplyFlexFlow(style: &mut ComputedStyle, value: &str) {
    let mut direction = FlexDirection::kRow;
    let mut wrap = false;
    let mut wrap_reverse = false;
    let mut saw_direction = false;
    let mut saw_wrap = false;
    let mut valid = true;
    for part in SplitWhitespace(value) {
        match part.as_str() {
            "row" if !saw_direction => {
                direction = FlexDirection::kRow;
                saw_direction = true;
            }
            "row-reverse" if !saw_direction => {
                direction = FlexDirection::kRowReverse;
                saw_direction = true;
            }
            "column" if !saw_direction => {
                direction = FlexDirection::kColumn;
                saw_direction = true;
            }
            "column-reverse" if !saw_direction => {
                direction = FlexDirection::kColumnReverse;
                saw_direction = true;
            }
            "nowrap" | "wrap" | "wrap-reverse" if !saw_wrap => {
                wrap = part != "nowrap";
                wrap_reverse = part == "wrap-reverse";
                saw_wrap = true;
            }
            _ => valid = false,
        }
    }
    if valid && (saw_direction || saw_wrap) {
        style.flex_direction = direction;
        style.flex_wrap = wrap;
        style
            .extended
            .get_or_insert_with(Default::default)
            .wrap_reverse = wrap_reverse;
    }
}

// cpp: style_resolver/style_resolver.cc:1938-1990
pub fn ApplyFlexShorthand(style: &mut ComputedStyle, value: &str, font_size: f64) -> bool {
    let normalized = value
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    let value = normalized.as_str();
    let mut candidate = style.clone();
    candidate.flex_basis = None;
    {
        let extra = candidate.extended.get_or_insert_with(Default::default);
        extra.flex_basis_percent = None;
        extra.flex_basis_calculated = false;
        extra.flex_basis_sizing = FlexBasisSizing::kAuto;
    }
    if value == "none" {
        candidate.flex_grow = 0.0;
        candidate.flex_shrink = 0.0;
        *style = candidate;
        return true;
    }
    if value == "auto" || value == "initial" {
        candidate.flex_grow = if value == "auto" { 1.0 } else { 0.0 };
        candidate.flex_shrink = 1.0;
        *style = candidate;
        return true;
    }
    let parts = SplitWhitespace(value);
    if parts.is_empty() || parts.len() > 3 {
        return false;
    }
    let grow = Number(&parts[0]).filter(|number| *number >= 0.0);
    let Some(grow) = grow else {
        if parts.len() != 1 || !SetFlexBasis(&mut candidate, &parts[0], font_size) {
            return false;
        }
        candidate.flex_grow = 1.0;
        candidate.flex_shrink = 1.0;
        *style = candidate;
        return true;
    };
    candidate.flex_grow = grow as f32;
    candidate.flex_shrink = 1.0;
    candidate
        .extended
        .get_or_insert_with(Default::default)
        .flex_basis_percent = Some(0.0);
    if parts.len() >= 2 {
        if let Some(shrink) = Number(&parts[1]).filter(|number| *number >= 0.0) {
            candidate.flex_shrink = shrink as f32;
            if parts.len() == 3 && !SetFlexBasis(&mut candidate, &parts[2], font_size) {
                return false;
            }
        } else if parts.len() != 2 || !SetFlexBasis(&mut candidate, &parts[1], font_size) {
            return false;
        }
    }
    *style = candidate;
    true
}

// cpp: style_resolver/style_resolver.cc:4986-4987,5006-5063
pub(crate) fn ApplyFlexProperty(style: &mut ComputedStyle, property: &str, raw: &str) -> bool {
    let value = raw
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    match property {
        "flex-grow" | "flex-shrink" => {
            if let Some(number) = Number(&value).filter(|n| *n >= 0.0) {
                if property == "flex-grow" {
                    style.flex_grow = number as f32;
                } else {
                    style.flex_shrink = number as f32;
                }
            }
        }
        "order" => {
            if let Some(number) = Number(&value)
                .filter(|n| n.floor() == *n && *n >= i32::MIN as f64 && *n <= i32::MAX as f64)
            {
                style.extended.get_or_insert_with(Default::default).order = number as i32;
            }
        }
        "flex-direction" => {
            if let Some(direction) = match value.as_str() {
                "row" => Some(FlexDirection::kRow),
                "column" => Some(FlexDirection::kColumn),
                "row-reverse" => Some(FlexDirection::kRowReverse),
                "column-reverse" => Some(FlexDirection::kColumnReverse),
                _ => None,
            } {
                style.flex_direction = direction;
            }
        }
        "flex-wrap" => {
            if matches!(value.as_str(), "nowrap" | "wrap" | "wrap-reverse") {
                style.flex_wrap = value != "nowrap";
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .wrap_reverse = value == "wrap-reverse";
            }
        }
        "flex-flow" => ApplyFlexFlow(style, &value),
        "flex" => {
            let font_size = style.extended.as_ref().map_or(16.0, |e| e.font_size);
            ApplyFlexShorthand(style, &value, font_size);
        }
        _ => return false,
    }
    true
}

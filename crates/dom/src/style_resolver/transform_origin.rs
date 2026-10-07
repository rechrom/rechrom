#![allow(non_snake_case)]

use super::{
    border_radius::Length, number::Number, selector::SourceSpace, shadow::SplitWhitespace,
};
use css_parser::length_percentage_parser::ParseLengthPercentage;
use layoutng_assembly::internal::{layout_input::Offset, paint_input::PaintTransformOrigin};

// cpp: style_resolver/style_resolver.cc:991-1018
// ShapeCoordinate is represented by (pixels, percentage); both components are
// retained, including calc() values, until the paint box dimensions are known.
fn ParseTransformOriginComponent(
    text: &str,
    horizontal: bool,
    font_size: f64,
) -> Option<(f64, f64)> {
    let value = text
        .trim_matches(|c: char| c.source_space())
        .to_ascii_lowercase();
    match value.as_str() {
        "center" => return Some((0.0, 50.0)),
        "left" if horizontal => return Some((0.0, 0.0)),
        "right" if horizontal => return Some((0.0, 100.0)),
        "top" if !horizontal => return Some((0.0, 0.0)),
        "bottom" if !horizontal => return Some((0.0, 100.0)),
        "left" | "right" | "top" | "bottom" => return None,
        _ => {}
    }
    if let Some(calculated) = ParseLengthPercentage(&value, font_size) {
        return Some((calculated.pixels, calculated.percentage));
    }
    if let Some(percentage) = value.strip_suffix('%').and_then(Number) {
        return Some((0.0, percentage));
    }
    Length(&value, font_size).map(|pixels| (pixels, 0.0))
}

// cpp: style_resolver/style_resolver.cc:1020-1067
pub(crate) fn ParseTransformOrigin(text: &str, font_size: f64) -> Option<PaintTransformOrigin> {
    let parts = SplitWhitespace(
        &text
            .trim_matches(|c: char| c.source_space())
            .to_ascii_lowercase(),
    );
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }
    let mut x = (0.0, 50.0);
    let mut y = (0.0, 50.0);
    if parts.len() == 1 {
        if matches!(parts[0].as_str(), "top" | "bottom") {
            y = ParseTransformOriginComponent(&parts[0], false, font_size)?;
        } else {
            x = ParseTransformOriginComponent(&parts[0], true, font_size)?;
        }
    } else {
        let vertical_first = matches!(parts[0].as_str(), "top" | "bottom")
            || (parts[0] == "center" && matches!(parts[1].as_str(), "left" | "right"));
        let first = ParseTransformOriginComponent(&parts[0], !vertical_first, font_size)?;
        let second = ParseTransformOriginComponent(&parts[1], vertical_first, font_size)?;
        if vertical_first {
            y = first;
            x = second;
        } else {
            x = first;
            y = second;
        }
    }
    let z = if parts.len() == 3 {
        Length(&parts[2], font_size)?
    } else {
        0.0
    };
    Some(PaintTransformOrigin {
        pixels: Offset { x: x.0, y: y.0 },
        percentages: Offset { x: x.1, y: y.1 },
        z,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_origin_keyword_order_and_mixed_coordinates() {
        for (input, pixels, percentages, z) in [
            ("bottom", (0.0, 0.0), (50.0, 100.0), 0.0),
            ("top right", (0.0, 0.0), (100.0, 0.0), 0.0),
            ("center left", (0.0, 0.0), (0.0, 50.0), 0.0),
            ("calc(25% + 2px) 3em 4px", (2.0, 48.0), (25.0, 0.0), 4.0),
        ] {
            let origin = ParseTransformOrigin(input, 16.0).unwrap();
            assert_eq!((origin.pixels.x, origin.pixels.y), pixels, "{input}");
            assert_eq!(
                (origin.percentages.x, origin.percentages.y),
                percentages,
                "{input}"
            );
            assert_eq!(origin.z, z);
        }
        for input in [
            "",
            "left right",
            "top bottom",
            "50% 50% 20%",
            "1px 2px 3px 4px",
        ] {
            assert!(ParseTransformOrigin(input, 16.0).is_none(), "{input}");
        }
    }
}

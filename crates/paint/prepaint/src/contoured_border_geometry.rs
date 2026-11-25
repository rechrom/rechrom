#![allow(non_snake_case)]

//! ContouredBorderGeometry entry points that read the painted fragment and
//! its style (core/paint/contoured_border_geometry.cc). The fragment-free
//! rounded-rect geometry stays in `paint_model::contoured_border_geometry`.

pub use paint_model::contoured_border_geometry::*;

use layout::fragment::ResidentBoxGeometry;
use layout::input::PaintStyle;
use layout::paint::{FragmentBoxSides, FragmentPaintData};
use paint_input_types::layout_input::{Edges, Size};
use paint_input_types::paint_input::{PaintCornerRadii, PaintCornerRadius};

// Source: ContouredBorderGeometry included-edge selection. Border widths and
// included sides come from the painted PhysicalBoxFragment.
pub fn IncludedBorderEdges(fragment: &FragmentPaintData) -> Edges {
    let border = fragment.Borders();
    let sides = fragment.BorderSides();
    Edges {
        top: if sides.top { border.top } else { 0.0 },
        right: if sides.right { border.right } else { 0.0 },
        bottom: if sides.bottom { border.bottom } else { 0.0 },
        left: if sides.left { border.left } else { 0.0 },
    }
}

// Source: ContouredBorderGeometry::CalcRadiiFor and ContouredRect::Inset
pub fn ResolveCornerRadii(
    style: &PaintStyle,
    sides: &FragmentBoxSides,
    outer_size: Size,
    inset: Edges,
) -> PaintCornerRadii {
    let mut radii = style
        .border_radii()
        .unwrap_or_else(|| UniformCornerRadii(style.border_radius));
    let percentages = &style.border_radii_percentages();
    radii.top_left.x += percentages.top_left.x * outer_size.width / 100.0;
    radii.top_left.y += percentages.top_left.y * outer_size.height / 100.0;
    radii.top_right.x += percentages.top_right.x * outer_size.width / 100.0;
    radii.top_right.y += percentages.top_right.y * outer_size.height / 100.0;
    radii.bottom_right.x += percentages.bottom_right.x * outer_size.width / 100.0;
    radii.bottom_right.y += percentages.bottom_right.y * outer_size.height / 100.0;
    radii.bottom_left.x += percentages.bottom_left.x * outer_size.width / 100.0;
    radii.bottom_left.y += percentages.bottom_left.y * outer_size.height / 100.0;
    for corner in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        ValidateRadius(corner);
        MakeDegenerateCornerSquare(corner);
    }
    for value in [inset.top, inset.right, inset.bottom, inset.left] {
        if !value.is_finite() || value < 0.0 {
            panic!("rounded rectangle inset is invalid");
        }
    }
    radii.top_left = PaintCornerRadius {
        x: (radii.top_left.x - inset.left).max(0.0),
        y: (radii.top_left.y - inset.top).max(0.0),
    };
    radii.top_right = PaintCornerRadius {
        x: (radii.top_right.x - inset.right).max(0.0),
        y: (radii.top_right.y - inset.top).max(0.0),
    };
    radii.bottom_right = PaintCornerRadius {
        x: (radii.bottom_right.x - inset.right).max(0.0),
        y: (radii.bottom_right.y - inset.bottom).max(0.0),
    };
    radii.bottom_left = PaintCornerRadius {
        x: (radii.bottom_left.x - inset.left).max(0.0),
        y: (radii.bottom_left.y - inset.bottom).max(0.0),
    };
    for corner in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        MakeDegenerateCornerSquare(corner);
    }
    if !sides.top || !sides.left {
        radii.top_left = PaintCornerRadius::default();
    }
    if !sides.top || !sides.right {
        radii.top_right = PaintCornerRadius::default();
    }
    if !sides.bottom || !sides.right {
        radii.bottom_right = PaintCornerRadius::default();
    }
    if !sides.bottom || !sides.left {
        radii.bottom_left = PaintCornerRadius::default();
    }
    Normalize(
        &mut radii,
        Size {
            width: (outer_size.width - inset.left - inset.right).max(0.0),
            height: (outer_size.height - inset.top - inset.bottom).max(0.0),
        },
    );
    radii
}


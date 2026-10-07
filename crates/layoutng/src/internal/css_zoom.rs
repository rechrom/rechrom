#![allow(non_snake_case)]
//! Conversion from CSS units to the used lengths consumed by layout and paint.
//! Corresponds to Blink's CSSToLengthConversionData::ZoomedComputedPixels.
use super::layout_input::{ComputedStyle, Edges, GridTrackBreadthKind, GridTrackListInput};
use std::borrow::Cow;

fn EdgesScale(edges: &mut Edges, zoom: f64) {
    edges.top *= zoom;
    edges.right *= zoom;
    edges.bottom *= zoom;
    edges.left *= zoom;
}
fn TracksScale(list: &mut GridTrackListInput, zoom: f64) {
    for repeat in &mut list.repeaters {
        for track in &mut repeat.tracks {
            for breadth in [&mut track.minimum, &mut track.maximum] {
                if breadth.kind != GridTrackBreadthKind::kFlex {
                    breadth.pixels *= zoom;
                }
            }
        }
    }
}
pub fn ZoomedStyle(style: &ComputedStyle) -> Cow<'_, ComputedStyle> {
    let zoom = style.extended.as_ref().map_or(1.0, |e| e.effective_zoom) as f64;
    if zoom == 1.0 {
        // Layout/paint callers only read the converted style. Unit zoom needs
        // neither conversion nor a per-fragment copy of its owned vectors.
        return Cow::Borrowed(style);
    }
    let mut output = style.clone();
    for dimension in [
        &mut output.width,
        &mut output.height,
        &mut output.min_width,
        &mut output.max_width,
        &mut output.min_height,
        &mut output.max_height,
        &mut output.flex_basis,
        &mut output.top,
        &mut output.right,
        &mut output.bottom,
        &mut output.left,
    ] {
        if let Some(px) = dimension {
            *px *= zoom;
        }
    }
    EdgesScale(&mut output.margin, zoom);
    EdgesScale(&mut output.padding, zoom);
    EdgesScale(&mut output.border, zoom);
    // Blink preserves at least one device pixel for a nonzero CSS border.
    for width in [
        &mut output.border.top,
        &mut output.border.right,
        &mut output.border.bottom,
        &mut output.border.left,
    ] {
        if *width > 0.0 {
            *width = width.max(1.0).floor();
        }
    }
    output.gap *= zoom;
    for width in &mut output.grid_template_columns {
        *width *= zoom;
    }
    if let Some(e) = &mut output.extended {
        e.font_size *= zoom;
        e.letter_spacing *= zoom;
        e.word_spacing *= zoom;
        e.text_indent *= zoom;
        e.border_spacing *= zoom;
        for dimension in [
            &mut e.row_gap,
            &mut e.column_gap,
            &mut e.line_height,
            &mut e.column_width,
            &mut e.vertical_border_spacing,
            &mut e.vertical_align_length,
            &mut e.contain_intrinsic_width.length,
            &mut e.contain_intrinsic_height.length,
            &mut e.math_baseline,
            &mut e.math_padded_depth,
            &mut e.math_lspace,
            &mut e.math_padded_voffset,
            &mut e.math_fraction_bar_thickness,
            &mut e.math_rspace,
            &mut e.math_min_size,
            &mut e.math_max_size,
        ] {
            if let Some(px) = dimension {
                *px *= zoom;
            }
        }
        if e.tab_size_is_length {
            e.tab_size *= zoom;
        }
        for list in [
            &mut e.grid_columns,
            &mut e.grid_rows,
            &mut e.grid_auto_columns,
            &mut e.grid_auto_rows,
        ] {
            TracksScale(list, zoom);
        }
        e.shape_margin.pixels *= zoom;
    }
    let paint: &mut super::paint_input::PaintStyleData = &mut output.paint;
    paint.border_radius *= zoom;
    if let Some(radii) = &mut paint.border_radii {
        for radius in [
            &mut radii.top_left,
            &mut radii.top_right,
            &mut radii.bottom_right,
            &mut radii.bottom_left,
        ] {
            radius.x *= zoom;
            radius.y *= zoom;
        }
    }
    paint.outline_width *= zoom;
    paint.outline_offset *= zoom;
    paint.column_rule_width *= zoom;
    for shadow in paint
        .box_shadows
        .iter_mut()
        .chain(paint.text_shadows.iter_mut())
    {
        shadow.offset.x *= zoom;
        shadow.offset.y *= zoom;
        shadow.blur_radius *= zoom;
        shadow.spread *= zoom;
    }
    for layer in &mut paint.background_images {
        layer.position_offset.x *= zoom;
        layer.position_offset.y *= zoom;
        if let Some(px) = &mut layer.width {
            *px *= zoom;
        }
        if let Some(px) = &mut layer.height {
            *px *= zoom;
        }
    }
    Cow::Owned(output)
}

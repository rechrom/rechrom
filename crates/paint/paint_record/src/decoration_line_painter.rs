//! Geometry shared by text-decoration overflow and recording.

#![allow(non_snake_case)]

pub use foundation::{
    DecorationGeometry, DecorationLineBounds as Bounds, StrokeStyle, WaveDefinition,
};
use layoutng_assembly::internal::paint_input::SvgStrokeLineCap;
use paint_model::cc::paint::path_effect::PathEffect;
use paint_model::styled_stroke_data::{GeometryInfo, StyledStrokeData};

use crate::{DisplayItem, DisplayItemType, PaintRect};

#[cfg(test)]
use foundation::gfx;

/// Backend-neutral DecorationLinePainter. GraphicsContext ownership is the
/// PaintContext/DisplayItemList in this package; this owner retains Chromium's
/// one-decoration dispatch and output contract.
pub struct DecorationLinePainter;

impl DecorationLinePainter {
    // Source: decoration_line_painter.cc:437-475.
    pub fn Paint(
        geometry: &DecorationGeometry,
        physical_line: PaintRect,
        item: DisplayItem,
    ) -> Vec<DisplayItem> {
        if geometry.line.width() <= 0.0 {
            return Vec::new();
        }
        match geometry.style {
            StrokeStyle::kWavyStroke => PaintWavyTextDecoration(geometry, physical_line, item)
                .into_iter()
                .collect(),
            StrokeStyle::kDottedStroke | StrokeStyle::kDashedStroke => {
                vec![DrawLineAsStroke(geometry, item)]
            }
            StrokeStyle::kSolidStroke | StrokeStyle::kDoubleStroke => {
                PaintSolidOrDouble(geometry, item)
            }
        }
    }
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:55-102.
// GraphicsContext's final DrawLine is represented by the package's retained
// DisplayItem, while all source stroke resolution remains in StyledStrokeData.
pub fn DrawLineAsStroke(geometry: &DecorationGeometry, mut item: DisplayItem) -> DisplayItem {
    let line = &geometry.line;
    let mid_y = (line.y() + (line.height() / 2.0).max(0.5)).floor();
    let mut p1 = (line.x().trunc(), mid_y);
    let mut p2 = (line.right().trunc(), mid_y);

    let mut styled_stroke = StyledStrokeData::default();
    styled_stroke.SetStyle(geometry.style);
    styled_stroke.SetThickness(geometry.Thickness());
    let thickness = geometry.Thickness().round() as i32;
    let stroke_data = styled_stroke.ConvertToStrokeData(&GeometryInfo {
        path_length: (p2.0 - p1.0) as i32,
        dash_thickness: thickness,
        closed_path: false,
    });

    if thickness % 2 != 0 {
        p1.1 += 0.5;
        p2.1 += 0.5;
    }
    if !StyledStrokeData::StrokeIsDashed(thickness as f32, styled_stroke.Style()) {
        p1.0 += thickness as f32 / 2.0;
        p2.0 -= thickness as f32 / 2.0;
    }

    item.r#type = DisplayItemType::kStrokeLine;
    item.rect = PaintRect {
        x: f64::from(p1.0),
        y: f64::from(p1.1),
        width: f64::from(p2.0 - p1.0),
        height: f64::from(p2.1 - p1.1),
    };
    item.decoration_thickness = f64::from(stroke_data.Thickness());
    item.antialias = geometry.antialias;
    let mut decoration_dash_intervals = Vec::new();
    let mut decoration_dash_offset = 0.0;
    let mut decoration_round_cap = false;
    if let Some(effect) = stroke_data.DashEffect() {
        match effect.as_ref() {
            PathEffect::Dash { intervals, phase } => {
                decoration_dash_intervals = intervals.iter().copied().map(f64::from).collect();
                decoration_dash_offset = f64::from(*phase);
            }
            PathEffect::Corner { .. } => {
                unreachable!("StyledStrokeData only resolves dash path effects")
            }
        }
        decoration_round_cap = geometry.style == StrokeStyle::kDottedStroke
            && !StyledStrokeData::StrokeIsDashed(thickness as f32, geometry.style);
    }
    item.decoration_dash_intervals = decoration_dash_intervals.clone();
    item.decoration_dash_offset = decoration_dash_offset;
    item.decoration_round_cap = decoration_round_cap;
    if item.decoration_paint_pass != crate::DecorationPaintPass::SvgStroke {
        item.stroke_width = f64::from(stroke_data.Thickness());
        item.dash_intervals = decoration_dash_intervals;
        item.dash_offset = decoration_dash_offset;
        item.round_cap = decoration_round_cap;
        if decoration_round_cap {
            item.svg_line_cap = SvgStrokeLineCap::kRound;
        }
    }
    item
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:21-23,40-45,104-123.
fn DrawLineAsRect(line: &foundation::gfx::RectF, mut item: DisplayItem) -> DisplayItem {
    if item.decoration_paint_pass == crate::DecorationPaintPass::SvgStroke {
        item.r#type = DisplayItemType::kStrokeRect;
    } else {
        item.r#type = DisplayItemType::kDrawRect;
    }
    item.rect = PaintRect {
        x: f64::from(line.x()),
        y: if item.decoration_paint_pass == crate::DecorationPaintPass::Html {
            f64::from((line.y() + 0.5).floor())
        } else {
            f64::from(line.y())
        },
        width: f64::from(line.width()),
        height: if item.decoration_paint_pass == crate::DecorationPaintPass::Html {
            f64::from(line.height().floor().max(1.0))
        } else {
            f64::from(line.height())
        },
    };
    item.antialias = item.decoration_paint_pass != crate::DecorationPaintPass::Html;
    item
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:462-473.
pub fn PaintSolidOrDouble(geometry: &DecorationGeometry, item: DisplayItem) -> Vec<DisplayItem> {
    if geometry.line.width() <= 0.0 {
        return Vec::new();
    }
    let first = DrawLineAsRect(&geometry.line, item);
    if geometry.style != StrokeStyle::kDoubleStroke {
        return vec![first];
    }
    let mut second_line = geometry.line;
    second_line.Offset(0.0, geometry.double_offset);
    let second = DrawLineAsRect(&second_line, first.clone());
    vec![first, second]
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:477-525.
// The retained compound operation is lowered to Chromium's PaintRecord tile
// shader in compositor/runtime/paint/lowering.rs. Retain the resolved wave so
// custom spelling/grammar waves are not replaced by a default wave.
pub fn PaintWavyTextDecoration(
    geometry: &DecorationGeometry,
    physical_line: PaintRect,
    mut item: DisplayItem,
) -> Option<DisplayItem> {
    if geometry.line.width() <= 0.0 {
        return None;
    }
    item.r#type = DisplayItemType::kStrokeWavyLine;
    item.rect = physical_line;
    if item.decoration_paint_pass != crate::DecorationPaintPass::SvgStroke {
        item.stroke_width = f64::from(geometry.Thickness());
    }
    item.decoration_thickness = f64::from(geometry.Thickness());
    item.antialias = true;
    item.wavy_wavelength = f64::from(geometry.wavy_wave.wavelength);
    item.wavy_control_point_distance = f64::from(geometry.wavy_wave.control_point_distance);
    item.wavy_phase = f64::from(geometry.wavy_wave.phase);
    item.wavy_offset = f64::from(geometry.wavy_offset);
    Some(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(y: f32, height: f32) -> gfx::RectF {
        gfx::RectF::new(gfx::PointF::new(2.75, y), gfx::SizeF::new(13.5, height))
    }

    #[test]
    fn solid_and_double_bounds_match_the_line_geometry() {
        let solid =
            DecorationGeometry::Make(StrokeStyle::kSolidStroke, &line(5.0, 2.0), 0.0, 0.0, None);
        assert_eq!(Bounds(&solid), solid.line);

        let above =
            DecorationGeometry::Make(StrokeStyle::kDoubleStroke, &line(5.0, 2.0), -3.0, 0.0, None);
        assert_eq!(Bounds(&above).y(), 2.0);
        assert_eq!(Bounds(&above).height(), 5.0);

        let below =
            DecorationGeometry::Make(StrokeStyle::kDoubleStroke, &line(5.0, 2.0), 3.0, 0.0, None);
        assert_eq!(Bounds(&below).y(), 5.0);
        assert_eq!(Bounds(&below).height(), 5.0);
    }

    #[test]
    fn dashed_bounds_use_chromium_integer_snapping() {
        let geometry =
            DecorationGeometry::Make(StrokeStyle::kDashedStroke, &line(4.2, 1.6), 0.0, 0.0, None);
        let bounds = Bounds(&geometry);
        assert_eq!(bounds.x(), 2.0);
        assert_eq!(bounds.y(), 4.0);
        assert_eq!(bounds.width(), 14.0);
        assert_eq!(bounds.height(), 2.0);
    }

    #[test]
    fn wavy_bounds_include_backend_equivalent_stroke_extrema() {
        let geometry =
            DecorationGeometry::Make(StrokeStyle::kWavyStroke, &line(7.0, 1.0), 0.0, 2.0, None);
        assert_eq!(geometry.wavy_wave.wavelength, 7.0);
        let bounds = Bounds(&geometry);
        assert_eq!(bounds.x(), 2.75);
        assert_eq!(bounds.y(), 7.0);
        assert_eq!(bounds.width(), 13.5);
        assert_eq!(bounds.height(), 5.0);
    }

    #[test]
    fn dashed_record_uses_styled_stroke_path_fitting() {
        let geometry = DecorationGeometry::Make(
            StrokeStyle::kDashedStroke,
            &gfx::RectF::new(gfx::PointF::new(2.75, 4.2), gfx::SizeF::new(37.5, 2.0)),
            0.0,
            0.0,
            None,
        );
        let item = DrawLineAsStroke(&geometry, DisplayItem::default());
        assert_eq!(item.r#type, DisplayItemType::kStrokeLine);
        assert_eq!(
            item.rect,
            PaintRect {
                x: 2.0,
                y: 5.0,
                width: 38.0,
                height: 0.0
            }
        );
        assert_eq!(item.stroke_width, 2.0);
        assert_eq!(item.dash_intervals, vec![6.0, 4.666666507720947]);
        assert!(!item.round_cap);
    }

    #[test]
    fn thick_dotted_record_preserves_round_caps_and_inset_endpoints() {
        let geometry = DecorationGeometry::Make(
            StrokeStyle::kDottedStroke,
            &gfx::RectF::new(gfx::PointF::new(2.0, 4.0), gfx::SizeF::new(20.0, 4.0)),
            0.0,
            0.0,
            None,
        );
        let item = DrawLineAsStroke(&geometry, DisplayItem::default());
        assert_eq!(item.rect.x, 4.0);
        assert_eq!(item.rect.width, 16.0);
        assert_eq!(item.dash_intervals, vec![0.0, 7.989999771118164]);
        assert!(item.round_cap);
        assert_eq!(item.svg_line_cap, SvgStrokeLineCap::kRound);
    }

    #[test]
    fn solid_and_double_records_are_snapped_fill_rects() {
        let solid =
            DecorationGeometry::Make(StrokeStyle::kSolidStroke, &line(4.6, 2.8), 0.0, 0.0, None);
        let solid_items = PaintSolidOrDouble(&solid, DisplayItem::default());
        assert_eq!(solid_items.len(), 1);
        assert_eq!(solid_items[0].r#type, DisplayItemType::kDrawRect);
        assert_eq!(solid_items[0].rect.y, 5.0);
        assert_eq!(solid_items[0].rect.height, 2.0);

        let double =
            DecorationGeometry::Make(StrokeStyle::kDoubleStroke, &line(4.6, 2.8), 3.8, 0.0, None);
        let double_items = PaintSolidOrDouble(&double, DisplayItem::default());
        assert_eq!(double_items.len(), 2);
        assert_eq!(double_items[0].rect.y, 5.0);
        assert_eq!(double_items[1].rect.y, 8.0);
    }

    #[test]
    fn svg_rect_passes_are_unsnapped_and_preserve_paint_style() {
        let geometry =
            DecorationGeometry::Make(StrokeStyle::kSolidStroke, &line(4.6, 2.8), 0.0, 0.0, None);
        let fill = PaintSolidOrDouble(
            &geometry,
            DisplayItem {
                decoration_paint_pass: crate::DecorationPaintPass::SvgFill,
                ..Default::default()
            },
        );
        assert_eq!(fill[0].r#type, DisplayItemType::kDrawRect);
        assert_eq!(fill[0].rect.y, 4.599999904632568);
        assert_eq!(fill[0].rect.height, 2.799999952316284);
        assert!(fill[0].antialias);

        let stroke = PaintSolidOrDouble(
            &geometry,
            DisplayItem {
                decoration_paint_pass: crate::DecorationPaintPass::SvgStroke,
                stroke_width: 3.0,
                dash_intervals: vec![2.0, 1.0],
                ..Default::default()
            },
        );
        assert_eq!(stroke[0].r#type, DisplayItemType::kStrokeRect);
        assert_eq!(stroke[0].stroke_width, 3.0);
        assert_eq!(stroke[0].dash_intervals, vec![2.0, 1.0]);
    }

    #[test]
    fn svg_stroke_keeps_both_decoration_and_svg_dash_geometry() {
        let geometry = DecorationGeometry::Make(
            StrokeStyle::kDashedStroke,
            &gfx::RectF::new(gfx::PointF::new(2.75, 4.2), gfx::SizeF::new(37.5, 2.0)),
            0.0,
            0.0,
            None,
        );
        let item = DrawLineAsStroke(
            &geometry,
            DisplayItem {
                decoration_paint_pass: crate::DecorationPaintPass::SvgStroke,
                stroke_width: 3.0,
                dash_intervals: vec![5.0, 2.0],
                dash_offset: 1.0,
                ..Default::default()
            },
        );
        assert_eq!(item.stroke_width, 3.0);
        assert_eq!(item.dash_intervals, vec![5.0, 2.0]);
        assert_eq!(item.dash_offset, 1.0);
        assert_eq!(item.decoration_thickness, 2.0);
        assert_eq!(item.decoration_dash_intervals, vec![6.0, 4.666666507720947]);
    }

    #[test]
    fn wavy_record_retains_custom_wave_and_offset_for_tile_lowering() {
        let custom = WaveDefinition {
            wavelength: 6.0,
            control_point_distance: 5.0,
            phase: -4.5,
        };
        let geometry = DecorationGeometry::Make(
            StrokeStyle::kWavyStroke,
            &line(4.0, 2.0),
            3.0,
            3.0,
            Some(&custom),
        );
        let physical = PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 0.0,
        };
        let item = PaintWavyTextDecoration(&geometry, physical, DisplayItem::default()).unwrap();
        assert_eq!(item.r#type, DisplayItemType::kStrokeWavyLine);
        assert_eq!(item.rect, physical);
        assert_eq!(item.stroke_width, 2.0);
        assert_eq!(item.wavy_wavelength, 6.0);
        assert_eq!(item.wavy_control_point_distance, 5.0);
        assert_eq!(item.wavy_phase, -4.5);
        assert_eq!(item.wavy_offset, 3.0);
    }
}

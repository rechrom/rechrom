//! Layout-owned vector geometry for CSS motion paths.
//!
//! `layoutng_style` owns typed shapes but has no concrete graphics `Path`.
//! This module is the narrow geometry owner used by `MotionPathBox`. It keeps
//! Chromium's two zoom contracts separate: computed basic-shape lengths are
//! already zoomed, while SVG `path()` data is scaled after path measurement.

#![allow(non_snake_case)]

use foundation::{gfx, FloatValueForLength, Length, PointAndTangent};
use layoutng_style::style::{
    basic_shapes::{
        BasicShape, BasicShapeCircle, BasicShapeEllipse, BasicShapeInset, BasicShapePolygon,
        BasicShapeRadius, BasicShapeWithCenterAndRadii, RadiusType, ShapeType,
    },
    style_path::StylePath,
};
use skia::{
    path::{Path, PathBuilder, Rect},
    src::core::SkContourMeasure::{
        last_contour_is_closed, path_length, point_and_tangent_at_length,
    },
};

use super::{
    boundary::native_paint_style::clip_path_export::NormalizePath,
    layout_input::{Offset, PaintPathCommand, PaintPathVerb},
};

#[derive(Clone, Copy, Default)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn from_gfx(value: &gfx::PointF) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
        }
    }

    fn from_offset(value: Offset) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
        }
    }

    fn distance(self, other: Self) -> f32 {
        (other.x - self.x).hypot(other.y - self.y)
    }
}

fn resolve(value: &Length, basis: f32) -> f32 {
    FloatValueForLength(value, basis)
}

fn corner_radius(center: Point, size: &gfx::SizeF, closest: bool) -> f32 {
    let dx = if closest {
        center.x.abs().min((size.width() - center.x).abs())
    } else {
        center.x.abs().max((size.width() - center.x).abs())
    };
    let dy = if closest {
        center.y.abs().min((size.height() - center.y).abs())
    } else {
        center.y.abs().max((size.height() - center.y).abs())
    };
    dx.hypot(dy)
}

fn circle_radius(value: &BasicShapeRadius, center: Point, size: &gfx::SizeF) -> f32 {
    let horizontal = [center.x.abs(), (size.width() - center.x).abs()];
    let vertical = [center.y.abs(), (size.height() - center.y).abs()];
    match value.GetType() {
        RadiusType::kValue => {
            let basis = size.width().hypot(size.height()) / std::f32::consts::SQRT_2;
            resolve(value.Value(), basis).max(0.0)
        }
        RadiusType::kClosestSide => horizontal
            .into_iter()
            .chain(vertical)
            .fold(f32::INFINITY, f32::min),
        RadiusType::kFarthestSide => horizontal.into_iter().chain(vertical).fold(0.0, f32::max),
        RadiusType::kClosestCorner => corner_radius(center, size, true),
        RadiusType::kFarthestCorner => corner_radius(center, size, false),
    }
}

fn ellipse_radius(value: &BasicShapeRadius, center: f32, extent: f32) -> f32 {
    match value.GetType() {
        RadiusType::kValue => resolve(value.Value(), extent).max(0.0),
        RadiusType::kClosestSide | RadiusType::kClosestCorner => {
            center.abs().min((extent - center).abs())
        }
        RadiusType::kFarthestSide | RadiusType::kFarthestCorner => {
            center.max((extent - center).abs())
        }
    }
}

fn ellipse_path(center: Point, rx: f32, ry: f32) -> Option<Path> {
    PathBuilder::from_oval(Rect::from_ltrb(
        center.x - rx,
        center.y - ry,
        center.x + rx,
        center.y + ry,
    )?)
}

fn inset_path(shape: &BasicShapeInset, size: &gfx::SizeF) -> Option<Path> {
    let left = resolve(shape.Left(), size.width());
    let right = (size.width() - resolve(shape.Right(), size.width())).max(left);
    let top = resolve(shape.Top(), size.height());
    let bottom = (size.height() - resolve(shape.Bottom(), size.height())).max(top);
    let width = right - left;
    let height = bottom - top;
    let radii = [
        shape.TopLeftRadius(),
        shape.TopRightRadius(),
        shape.BottomRightRadius(),
        shape.BottomLeftRadius(),
    ];
    let mut resolved = radii.map(|value| {
        (
            resolve(value.Width(), size.width()).max(0.0),
            resolve(value.Height(), size.height()).max(0.0),
        )
    });
    let fit = |available: f32, requested: f32| {
        if requested > available && requested > 0.0 {
            available / requested
        } else {
            1.0
        }
    };
    let factor = 1.0_f32
        .min(fit(width, resolved[0].0 + resolved[1].0))
        .min(fit(width, resolved[3].0 + resolved[2].0))
        .min(fit(height, resolved[0].1 + resolved[3].1))
        .min(fit(height, resolved[1].1 + resolved[2].1));
    for radius in &mut resolved {
        radius.0 *= factor;
        radius.1 *= factor;
    }

    const WEIGHT: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let mut builder = PathBuilder::new();
    builder.move_to(left + resolved[0].0, top);
    builder.line_to(right - resolved[1].0, top);
    builder.conic_to(right, top, right, top + resolved[1].1, WEIGHT);
    builder.line_to(right, bottom - resolved[2].1);
    builder.conic_to(right, bottom, right - resolved[2].0, bottom, WEIGHT);
    builder.line_to(left + resolved[3].0, bottom);
    builder.conic_to(left, bottom, left, bottom - resolved[3].1, WEIGHT);
    builder.line_to(left, top + resolved[0].1);
    builder.conic_to(left, top, left + resolved[0].0, top, WEIGHT);
    builder.close();
    builder.finish()
}

fn polygon_path(shape: &BasicShapePolygon, size: &gfx::SizeF) -> Option<Path> {
    let vertices: Vec<Point> = shape
        .Values()
        .as_slice()
        .chunks_exact(2)
        .map(|pair| Point {
            x: resolve(&pair[0], size.width()),
            y: resolve(&pair[1], size.height()),
        })
        .collect();
    let first = *vertices.first()?;
    let requested = resolve(shape.RoundingRadius(), 0.0);
    let mut builder = PathBuilder::new();
    if requested <= 0.5 || vertices.len() < 3 {
        builder.move_to(first.x, first.y);
        for point in vertices.iter().skip(1) {
            builder.line_to(point.x, point.y);
        }
        builder.close();
        return builder.finish();
    }

    let count = vertices.len();
    let last = vertices[count - 1];
    builder.move_to((last.x + first.x) * 0.5, (last.y + first.y) * 0.5);
    for index in 0..count {
        let previous = vertices[(index + count - 1) % count];
        let current = vertices[index];
        let next = vertices[(index + 1) % count];
        let incoming_length = previous.distance(current);
        let outgoing_length = current.distance(next);
        if incoming_length <= 0.5 || outgoing_length <= 0.5 {
            builder.line_to(current.x, current.y);
            continue;
        }
        let incoming = Point {
            x: (current.x - previous.x) / incoming_length,
            y: (current.y - previous.y) / incoming_length,
        };
        let outgoing = Point {
            x: (next.x - current.x) / outgoing_length,
            y: (next.y - current.y) / outgoing_length,
        };
        let interior = (-(incoming.x * outgoing.x + incoming.y * outgoing.y))
            .clamp(-1.0, 1.0)
            .acos();
        let tangent = (interior * 0.5).tan();
        let weight = (interior * 0.5).sin();
        if !tangent.is_finite() || tangent <= f32::EPSILON || weight >= 1.0 - f32::EPSILON {
            builder.line_to(current.x, current.y);
            continue;
        }
        let radius = requested
            .min(tangent * incoming_length * 0.5)
            .min(tangent * outgoing_length * 0.5);
        let along = radius / tangent;
        let entry = Point {
            x: current.x - incoming.x * along,
            y: current.y - incoming.y * along,
        };
        let exit = Point {
            x: current.x + outgoing.x * along,
            y: current.y + outgoing.y * along,
        };
        builder.line_to(entry.x, entry.y);
        builder.conic_to(current.x, current.y, exit.x, exit.y, weight);
    }
    builder.close();
    builder.finish()
}

fn commands_path(commands: &[PaintPathCommand]) -> Option<Path> {
    let mut builder = PathBuilder::new();
    for command in commands {
        let point = Point::from_offset(command.point);
        match command.verb {
            PaintPathVerb::kMoveTo => builder.move_to(point.x, point.y),
            PaintPathVerb::kLineTo => builder.line_to(point.x, point.y),
            PaintPathVerb::kCubicTo => {
                let control1 = Point::from_offset(command.control1);
                let control2 = Point::from_offset(command.control2);
                builder.cubic_to(
                    control1.x, control1.y, control2.x, control2.y, point.x, point.y,
                );
            }
            PaintPathVerb::kClose => builder.close(),
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                unreachable!("native SVG normalization emits only cubic curves")
            }
        }
    }
    builder.finish()
}

fn basic_shape_path(shape: &dyn BasicShape, start: Point, size: &gfx::SizeF) -> Option<Path> {
    match shape.GetType() {
        ShapeType::kBasicShapeCircleType => {
            let circle = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeCircle) };
            let center = if circle.HasExplicitCenter() {
                Point {
                    x: resolve(circle.Center().X(), size.width()),
                    y: resolve(circle.Center().Y(), size.height()),
                }
            } else {
                start
            };
            let radius = circle_radius(circle.Radius(), center, size);
            ellipse_path(center, radius, radius)
        }
        ShapeType::kBasicShapeEllipseType => {
            let ellipse = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeEllipse) };
            let center = if ellipse.HasExplicitCenter() {
                Point {
                    x: resolve(ellipse.Center().X(), size.width()),
                    y: resolve(ellipse.Center().Y(), size.height()),
                }
            } else {
                start
            };
            let rx = match ellipse.RadiusX().GetType() {
                RadiusType::kClosestCorner => corner_radius(center, size, true),
                RadiusType::kFarthestCorner => corner_radius(center, size, false),
                _ => ellipse_radius(ellipse.RadiusX(), center.x, size.width()),
            };
            let ry = match ellipse.RadiusY().GetType() {
                RadiusType::kClosestCorner => corner_radius(center, size, true),
                RadiusType::kFarthestCorner => corner_radius(center, size, false),
                _ => ellipse_radius(ellipse.RadiusY(), center.y, size.height()),
            };
            ellipse_path(center, rx, ry)
        }
        ShapeType::kBasicShapeInsetType => {
            let inset = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeInset) };
            inset_path(inset, size)
        }
        ShapeType::kBasicShapePolygonType => {
            let polygon = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapePolygon) };
            polygon_path(polygon, size)
        }
        _ => None,
    }
}

fn resolved_sample(
    path: Option<Path>,
    distance: &Length,
    scale: f32,
    path_is_author_units: bool,
) -> PointAndTangent {
    let Some(path) = path else {
        return PointAndTangent {
            point: gfx::PointF::new(0.0, 0.0),
            tangent_in_degrees: 0.0,
        };
    };
    let length = path_length(&path);
    let mut resolved = if path_is_author_units {
        resolve(distance, length * scale) / scale
    } else {
        resolve(distance, length)
    };
    if length > 0.0 {
        if last_contour_is_closed(&path) {
            resolved %= length;
            if resolved < 0.0 {
                resolved += length;
            }
        } else {
            resolved = resolved.clamp(0.0, length);
        }
    }
    let sampled = point_and_tangent_at_length(&path, resolved);
    let point_scale = if path_is_author_units { scale } else { 1.0 };
    PointAndTangent {
        point: gfx::PointF::new(sampled.x * point_scale, sampled.y * point_scale),
        tangent_in_degrees: sampled.tangent_y.atan2(sampled.tangent_x).to_degrees(),
    }
}

pub(crate) fn PointAndTangentForShape(
    shape: &dyn BasicShape,
    starting_point: &gfx::PointF,
    reference_size: &gfx::SizeF,
    distance: &Length,
    effective_zoom: f32,
) -> Option<PointAndTangent> {
    if shape.GetType() == ShapeType::kStylePathType {
        let path = unsafe { &*(shape as *const dyn BasicShape as *const StylePath) };
        return Some(resolved_sample(
            commands_path(&NormalizePath(path.ByteStream().Span())),
            distance,
            effective_zoom,
            true,
        ));
    }
    Some(resolved_sample(
        basic_shape_path(shape, Point::from_gfx(starting_point), reference_size),
        distance,
        1.0,
        false,
    ))
}

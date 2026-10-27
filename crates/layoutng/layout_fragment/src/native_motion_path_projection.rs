//! Fragment-owned vector geometry for CSS motion paths.
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
    include::core::SkPathTypes::{SkPathDirection, SkPathVerb},
    src::core::{
        SkContourMeasure::SkContourMeasureIter, SkPath::SkPath, SkPathBuilder::SkPathBuilder,
        SkPoint::SkPoint, SkRect::SkRect,
    },
};

use super::super::{
    boundary::native_paint_style::clip_path_export::NormalizePath,
    layout_input::{Offset, PaintPathCommand, PaintPathVerb},
};

#[path = "native_motion_path_geometry.rs"]
mod layout_fragment_native_motion_path_geometry;
use layout_fragment_native_motion_path_geometry::{
    MotionPathCommands, MotionPathLength, MotionPathPointAndTangentAtLength, MotionPathPolygon,
};

#[derive(Clone, Copy, Default)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy)]
struct PathPointAndTangent {
    x: f32,
    y: f32,
    tangent_x: f32,
    tangent_y: f32,
}

/// Preserve the previous Skia adapter's `PathBuilder::finish` contract: an
/// empty/move-only or non-finite path is not usable motion-path geometry.
fn finish_path(builder: &mut SkPathBuilder) -> Option<SkPath> {
    if builder.verbs().len() <= 1 || !builder.isFinite() {
        return None;
    }
    Some(builder.detach(None))
}

fn path_length(path: &SkPath) -> f32 {
    MotionPathLength(path)
}

fn point_and_tangent_at_length(path: &SkPath, distance: f32) -> PathPointAndTangent {
    MotionPathPointAndTangentAtLength(path, distance)
}

fn last_contour_is_closed(path: &SkPath) -> bool {
    path.verbs().last() == Some(&SkPathVerb::kClose)
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

fn ellipse_path(center: Point, rx: f32, ry: f32) -> Option<SkPath> {
    let left = center.x - rx;
    let top = center.y - ry;
    let right = center.x + rx;
    let bottom = center.y + ry;
    let rect = SkRect::MakeLTRB(left, top, right, bottom);
    // The previous Rect::from_ltrb rejected unsorted/non-finite coordinates
    // and finite endpoints whose width or height overflowed to infinity.
    if !rect.isFinite()
        || !rect.isSorted()
        || !(right - left).is_finite()
        || !(bottom - top).is_finite()
    {
        return None;
    }
    let mut builder = SkPathBuilder::default();
    builder.addOvalDefault(&rect, SkPathDirection::kCW);
    finish_path(&mut builder)
}

fn inset_path(shape: &BasicShapeInset, size: &gfx::SizeF) -> Option<SkPath> {
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
    let mut builder = SkPathBuilder::default();
    builder.moveToXY(left + resolved[0].0, top);
    builder.lineToXY(right - resolved[1].0, top);
    builder.conicToXY(right, top, right, top + resolved[1].1, WEIGHT);
    builder.lineToXY(right, bottom - resolved[2].1);
    builder.conicToXY(right, bottom, right - resolved[2].0, bottom, WEIGHT);
    builder.lineToXY(left + resolved[3].0, bottom);
    builder.conicToXY(left, bottom, left, bottom - resolved[3].1, WEIGHT);
    builder.lineToXY(left, top + resolved[0].1);
    builder.conicToXY(left, top, left + resolved[0].0, top, WEIGHT);
    builder.close();
    finish_path(&mut builder)
}

fn polygon_path(shape: &BasicShapePolygon, size: &gfx::SizeF) -> Option<SkPath> {
    MotionPathPolygon(shape, size)
}

fn commands_path(commands: &[PaintPathCommand]) -> Option<SkPath> {
    MotionPathCommands(commands)
}

fn basic_shape_path(shape: &dyn BasicShape, start: Point, size: &gfx::SizeF) -> Option<SkPath> {
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
    path: Option<SkPath>,
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

#[inline(never)]
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

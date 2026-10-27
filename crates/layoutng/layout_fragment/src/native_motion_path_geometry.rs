//! Native CSS motion-path geometry with fragment-qualified sampling frames.
//!
//! Included by `motion_path_geometry` at the reciprocal assembly boundary.
//! Author-sized contour, polygon, and command walks stay out of the LayoutNG
//! core facade so CPU samples identify fragment geometry directly.

use super::*;

/// Sum every nonempty contour using Skia's source contour measurement rules.
#[inline(never)]
pub(super) fn MotionPathLength(path: &SkPath) -> f32 {
    let mut contours = SkContourMeasureIter::new(path, false, 1.0);
    let mut length = 0.0;
    loop {
        let contour = contours.next();
        if !contour.is_some() {
            return length;
        }
        length += contour.length();
    }
}

/// Match Blink's `Path::PointAndNormalAtLength`: distances continue across
/// contours and an unavailable sample falls back to the first path point.
#[inline(never)]
pub(super) fn MotionPathPointAndTangentAtLength(
    path: &SkPath,
    distance: f32,
) -> PathPointAndTangent {
    let distance = if distance.is_finite() { distance } else { 0.0 };
    let mut contours = SkContourMeasureIter::new(path, false, 1.0);
    let mut contour_start = 0.0;
    loop {
        let contour = contours.next();
        if !contour.is_some() {
            break;
        }
        let contour_end = contour_start + contour.length();
        if distance <= contour_end {
            let local_distance = (distance - contour_start).clamp(0.0, contour.length());
            let mut position = SkPoint::default();
            let mut tangent = SkPoint::default();
            if contour.getPosTan(local_distance, Some(&mut position), Some(&mut tangent)) {
                return PathPointAndTangent {
                    x: position.fX,
                    y: position.fY,
                    tangent_x: tangent.fX,
                    tangent_y: tangent.fY,
                };
            }
        }
        contour_start = contour_end;
    }

    let first = path.getPoint(0);
    PathPointAndTangent {
        x: first.fX,
        y: first.fY,
        tangent_x: 1.0,
        tangent_y: 0.0,
    }
}

#[inline(never)]
pub(super) fn MotionPathPolygon(shape: &BasicShapePolygon, size: &gfx::SizeF) -> Option<SkPath> {
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
    let mut builder = SkPathBuilder::default();
    if requested <= 0.5 || vertices.len() < 3 {
        builder.moveToXY(first.x, first.y);
        for point in vertices.iter().skip(1) {
            builder.lineToXY(point.x, point.y);
        }
        builder.close();
        return finish_path(&mut builder);
    }

    let count = vertices.len();
    let last = vertices[count - 1];
    builder.moveToXY((last.x + first.x) * 0.5, (last.y + first.y) * 0.5);
    for index in 0..count {
        let previous = vertices[(index + count - 1) % count];
        let current = vertices[index];
        let next = vertices[(index + 1) % count];
        let incoming_length = previous.distance(current);
        let outgoing_length = current.distance(next);
        if incoming_length <= 0.5 || outgoing_length <= 0.5 {
            builder.lineToXY(current.x, current.y);
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
            builder.lineToXY(current.x, current.y);
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
        builder.lineToXY(entry.x, entry.y);
        builder.conicToXY(current.x, current.y, exit.x, exit.y, weight);
    }
    builder.close();
    finish_path(&mut builder)
}

#[inline(never)]
pub(super) fn MotionPathCommands(commands: &[PaintPathCommand]) -> Option<SkPath> {
    let mut builder = SkPathBuilder::default();
    for command in commands {
        let point = Point::from_offset(command.point);
        match command.verb {
            PaintPathVerb::kMoveTo => {
                builder.moveToXY(point.x, point.y);
            }
            PaintPathVerb::kLineTo => {
                builder.lineToXY(point.x, point.y);
            }
            PaintPathVerb::kCubicTo => {
                let control1 = Point::from_offset(command.control1);
                let control2 = Point::from_offset(command.control2);
                builder.cubicToXY(
                    control1.x, control1.y, control2.x, control2.y, point.x, point.y,
                );
            }
            PaintPathVerb::kClose => {
                builder.close();
            }
            PaintPathVerb::kQuadraticTo | PaintPathVerb::kConicTo => {
                unreachable!("native SVG normalization emits only cubic curves")
            }
        }
    }
    finish_path(&mut builder)
}

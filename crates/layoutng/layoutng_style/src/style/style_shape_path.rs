//! Source StyleShape::SegmentVisitor and GetPath, style_shape.cc:40-193.
//! Resolve the compiled lengths against the current reference box, then feed
//! the same source SVGPathBuilder owner as path(). No strings or sampled arcs.
use super::style_shape::{ControlPoint, ControlPointOrigin, CurveSegment, Segment, StyleShape};
use foundation::svg_path_byte_stream::NativePathSink as BasicShapePathSink;
use foundation::{
    gfx::{PointF, RectF},
    svg_path_byte_stream::{NativeSvgPathBuilder, SvgPathCommand as C, SvgPathSegmentData},
    FloatValueForLength, LengthPoint,
};
fn point(p: &LengthPoint, rect: RectF) -> PointF {
    PointF::new(
        FloatValueForLength(p.X(), rect.width()),
        FloatValueForLength(p.Y(), rect.height()),
    )
}
fn add(a: PointF, b: PointF) -> PointF {
    PointF::new(a.x() + b.x(), a.y() + b.y())
}
fn control(p: &ControlPoint, start: PointF, end: PointF, rect: RectF) -> PointF {
    let pnt = point(&p.point, rect);
    match p.origin {
        ControlPointOrigin::ReferenceBox => pnt,
        ControlPointOrigin::SegmentStart => add(pnt, start),
        ControlPointOrigin::SegmentEnd => add(pnt, end),
    }
}
fn curve<const N: usize>(
    segment: &CurveSegment<N>,
    command: C,
    builder: &mut NativeSvgPathBuilder<'_>,
    rect: RectF,
) {
    let start = builder.CurrentPoint();
    let target = point(&segment.target_point, rect);
    let end = if command.IsRelative() {
        add(start, target)
    } else {
        target
    };
    let mut data = SvgPathSegmentData::new(command.ToAbsolute());
    data.target = end;
    let p1 = control(&segment.control_points[0], start, end, rect);
    if matches!(command, C::SmoothCubicAbs | C::SmoothCubicRel) {
        data.point2 = p1;
    } else {
        data.point1 = p1;
    }
    if N == 2 {
        data.point2 = control(&segment.control_points[1], start, end, rect);
    }
    builder.EmitSegment(data);
}
pub fn BuildStyleShapePath(
    shape: &StyleShape,
    rect: RectF,
    scale: f32,
    sink: &mut dyn BasicShapePathSink,
) {
    sink.SetWindRule(shape.GetWindRule());
    let mut builder = NativeSvgPathBuilder::new(sink, scale, rect.x(), rect.y());
    let mut origin = SvgPathSegmentData::new(C::MoveAbs);
    origin.target = point(shape.GetOrigin(), rect);
    builder.EmitSegment(origin);
    for segment in shape.Segments() {
        let (command, target) = match segment {
            Segment::MoveTo(p) => (C::MoveAbs, point(p, rect)),
            Segment::MoveBy(p) => (C::MoveRel, point(p, rect)),
            Segment::LineTo(p) => (C::LineAbs, point(p, rect)),
            Segment::LineBy(p) => (C::LineRel, point(p, rect)),
            Segment::SmoothQuadraticCurveTo(p) => (C::SmoothQuadAbs, point(p, rect)),
            Segment::SmoothQuadraticCurveBy(p) => (C::SmoothQuadRel, point(p, rect)),
            Segment::HLineTo(x) => (
                C::HLineAbs,
                PointF::new(FloatValueForLength(x, rect.width()), 0.),
            ),
            Segment::HLineBy(x) => (
                C::HLineRel,
                PointF::new(FloatValueForLength(x, rect.width()), 0.),
            ),
            Segment::VLineTo(y) => (
                C::VLineAbs,
                PointF::new(0., FloatValueForLength(y, rect.height())),
            ),
            Segment::VLineBy(y) => (
                C::VLineRel,
                PointF::new(0., FloatValueForLength(y, rect.height())),
            ),
            Segment::CubicCurveTo(s) => {
                curve(s, C::CubicAbs, &mut builder, rect);
                continue;
            }
            Segment::CubicCurveBy(s) => {
                curve(s, C::CubicRel, &mut builder, rect);
                continue;
            }
            Segment::QuadraticCurveTo(s) => {
                curve(s, C::QuadAbs, &mut builder, rect);
                continue;
            }
            Segment::QuadraticCurveBy(s) => {
                curve(s, C::QuadRel, &mut builder, rect);
                continue;
            }
            Segment::SmoothCubicCurveTo(s) => {
                curve(s, C::SmoothCubicAbs, &mut builder, rect);
                continue;
            }
            Segment::SmoothCubicCurveBy(s) => {
                curve(s, C::SmoothCubicRel, &mut builder, rect);
                continue;
            }
            Segment::ArcTo(s) | Segment::ArcBy(s) => {
                let mut data = SvgPathSegmentData::new(if matches!(segment, Segment::ArcBy(_)) {
                    C::ArcRel
                } else {
                    C::ArcAbs
                });
                data.target = point(&s.target_point, rect);
                data.arc_angle = s.angle as f32;
                data.arc_large = s.large;
                data.arc_sweep = s.sweep;
                let agnostic = FloatValueForLength(
                    &s.direction_agnostic_radius,
                    rect.width().hypot(rect.height()) / 2_f32.sqrt(),
                );
                data.point1 = PointF::new(
                    FloatValueForLength(s.radius.Width(), rect.width()) + agnostic,
                    FloatValueForLength(s.radius.Height(), rect.height()) + agnostic,
                );
                builder.EmitSegment(data);
                continue;
            }
            Segment::Close => (C::Close, PointF::default()),
        };
        let mut data = SvgPathSegmentData::new(command);
        data.target = target;
        builder.EmitSegment(data);
    }
}

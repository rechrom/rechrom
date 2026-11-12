//! Native Path primitives for the shared source BasicShape path builder.
use foundation::{
    gfx::{PointF, RectF},
    WindRule,
};
use layoutng_style::style::basic_shapes::BasicShape;
use paint_model::basic_shape_path::{BasicShapePathSink, BuildBasicShapePath, InvalidBasicShape};
use skia::{
    include::core::SkPathTypes::{SkPathDirection, SkPathFillType},
    src::core::{
        SkPath::SkPath,
        SkPathBuilder::{ArcSize, SkPathBuilder},
        SkPoint::SkPoint,
        SkRRect::SkRRect,
        SkRect::SkRect,
    },
};
pub(super) struct Builder(pub(super) SkPathBuilder);
pub(super) fn point(value: PointF) -> SkPoint {
    SkPoint {
        fX: value.x(),
        fY: value.y(),
    }
}
pub(super) fn rect(value: RectF) -> SkRect {
    SkRect::MakeLTRB(value.x(), value.y(), value.right(), value.bottom())
}
impl BasicShapePathSink for Builder {
    fn SetWindRule(&mut self, rule: WindRule) {
        self.0.setFillType(if rule == WindRule::RULE_EVENODD {
            SkPathFillType::kEvenOdd
        } else {
            SkPathFillType::kWinding
        });
    }
    fn MoveTo(&mut self, p: PointF) {
        self.0.moveTo(point(p));
    }
    fn LineTo(&mut self, p: PointF) {
        self.0.lineTo(point(p));
    }
    fn QuadTo(&mut self, c: PointF, p: PointF) {
        self.0.quadTo(point(c), point(p));
    }
    fn CubicTo(&mut self, a: PointF, b: PointF, p: PointF) {
        self.0.cubicTo(point(a), point(b), point(p));
    }
    fn ArcTo(&mut self, c: PointF, n: PointF, r: f32) {
        self.0.arcToPoints(point(c), point(n), r);
    }
    fn ArcToSvg(&mut self, r: PointF, a: f32, l: bool, s: bool, p: PointF) {
        self.0.arcToSvg(
            point(r),
            a,
            if l {
                ArcSize::kLarge_ArcSize
            } else {
                ArcSize::kSmall_ArcSize
            },
            if s {
                SkPathDirection::kCW
            } else {
                SkPathDirection::kCCW
            },
            point(p),
        );
    }
    fn Close(&mut self) {
        self.0.close();
    }
    fn Oval(&mut self, r: RectF) {
        self.0.addOvalDefault(&rect(r), SkPathDirection::kCW);
    }
    fn RoundedRect(&mut self, r: RectF, c: [PointF; 4]) {
        self.0.addRRect(
            &SkRRect::MakeRectRadii(&rect(r), &c.map(point)),
            SkPathDirection::kCW,
            0,
        );
    }
}
pub(super) fn FromBasicShape(
    shape: &dyn BasicShape,
    reference: RectF,
    zoom: f32,
    path_scale: f32,
) -> Result<SkPath, InvalidBasicShape> {
    let mut builder = Builder(SkPathBuilder::new());
    BuildBasicShapePath(shape, reference, zoom, path_scale, &mut builder)?;
    Ok(builder.0.detach(None))
}

struct BorderBuilder;
impl paint_model::border_shape_path::BorderShapePathSink for BorderBuilder {
    type Path = SkPath;
    fn BasicShape(
        &self,
        shape: &dyn BasicShape,
        reference: RectF,
        zoom: f32,
    ) -> Result<SkPath, InvalidBasicShape> {
        FromBasicShape(shape, reference, zoom, 1.)
    }
    fn SegmentMasks(&self, path: &SkPath) -> u32 {
        path.getSegmentMasks()
    }
    fn StrokeUnion(&self, path: SkPath, thickness: f32, miter: f32, round: bool) -> SkPath {
        use skia::{
            include::pathops::SkPathOps::{kUnion_SkPathOp, Op},
            src::core::{
                SkPaint::{Cap, Join},
                SkStrokeRec::{InitStyle, SkStrokeRec},
            },
        };
        let mut stroke = SkStrokeRec::new(InitStyle::kFill_InitStyle);
        stroke.setStrokeStyle(thickness, false);
        stroke.setStrokeParams(
            Cap::kButt_Cap,
            if round {
                Join::kRound_Join
            } else {
                Join::kMiter_Join
            },
            miter,
        );
        let mut builder = SkPathBuilder::new();
        stroke.applyToPath(&mut builder, &path);
        Op(&path, &builder.detach(None), kUnion_SkPathOp).unwrap_or(path)
    }
}
pub(super) fn BorderShapeOuterPath(
    style: &layoutng_style::style::computed_style::ComputedStyle,
    reference: RectF,
) -> Result<SkPath, InvalidBasicShape> {
    paint_model::border_shape_path::OuterPath(style, reference, &BorderBuilder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{Length, LengthPoint};
    use layoutng_style::style::basic_shapes::{
        BasicShapeCircle, BasicShapePolygon, BasicShapeWithCenterAndRadii,
    };
    #[test]
    fn compiled_basic_shapes_keep_native_conics_and_rounded_polygon_arcs() {
        let reference = RectF::new(PointF::new(0., 0.), foundation::gfx::SizeF::new(100., 100.));
        let mut circle = BasicShapeCircle::default();
        circle.SetCenter(LengthPoint::new(
            &Length::Percent(50.),
            &Length::Percent(50.),
        ));
        let path = FromBasicShape(&circle, reference, 1., 1.).unwrap();
        assert_eq!(path.countVerbs(), 6);
        assert!(path.containsXY(50., 1.));
        assert!(!path.containsXY(90., 90.));
        let mut polygon = BasicShapePolygon::default();
        for (x, y) in [(0, 0), (100, 0), (100, 100), (0, 100)] {
            polygon.AppendPoint(&Length::Fixed(x), &Length::Fixed(y));
        }
        polygon.SetRoundingRadius(&Length::Fixed(20));
        let path = FromBasicShape(&polygon, reference, 1., 1.).unwrap();
        assert!(
            path.getSegmentMasks() & 4 != 0,
            "rounded vertices must use conic arcs"
        );
        assert!(!path.containsXY(1., 1.));
        assert!(path.containsXY(10., 10.));
        assert!(path.containsXY(50., 1.));
    }
    #[test]
    fn css_shape_command_sink_matches_native_svg_arc_primitives() {
        use foundation::{LengthPoint, LengthSize};
        use layoutng_style::style::style_shape::{ArcSegment, Segment, StyleShape};
        use paint_model::{
            basic_shape_commands::BasicShapeCommands,
            paint_clip_path::{PaintPath, PaintPathFillType},
        };
        let reference = RectF::new(
            PointF::new(17.25, 9.75),
            foundation::gfx::SizeF::new(160., 120.),
        );
        for angle in [0., 90., 37., -125.] {
            for (large, sweep) in [(false, false), (false, true), (true, false), (true, true)] {
                let shape = StyleShape::new(
                    foundation::WindRule::RULE_EVENODD,
                    LengthPoint::new(&Length::Percent(5), &Length::Percent(15)),
                    vec![
                        Segment::ArcTo(ArcSegment {
                            target_point: LengthPoint::new(
                                &Length::Percent(85),
                                &Length::Percent(75),
                            ),
                            angle,
                            radius: LengthSize::new(&Length::Fixed(40), &Length::Fixed(55)),
                            direction_agnostic_radius: Length::Fixed(0),
                            large,
                            sweep,
                        }),
                        Segment::Close,
                    ],
                );
                let native = FromBasicShape(&shape, reference, 1., 0.5).unwrap();
                let mut sink = BasicShapeCommands::default();
                BuildBasicShapePath(&shape, reference, 1., 0.5, &mut sink).unwrap();
                assert!(sink.commands.iter().any(
                    |command| command.verb == layoutng_geometry::shape_path::PathVerb::kConicTo
                ));
                let resolved = sink.Finish();
                let measure =
                    layoutng_geometry::shape_path_measure::ShapePathMeasure::new(&resolved);
                let mut contours = skia::src::core::SkContourMeasure::SkContourMeasureIter::new(
                    &native, false, 1.,
                );
                let contour = contours.next();
                assert!(contour.is_some());
                assert!((measure.Length() - contour.length()).abs() < 0.001);
                for fraction in [0., 0.1, 0.5, 0.9, 1.] {
                    let distance = contour.length() * fraction;
                    let mut point = SkPoint { fX: 0., fY: 0. };
                    let mut tangent = point;
                    assert!(contour.getPosTan(distance, Some(&mut point), Some(&mut tangent)));
                    let sampled = measure.PointAndTangent(distance);
                    assert!(
                        (sampled.point.x() - point.fX).abs() < 0.001
                            && (sampled.point.y() - point.fY).abs() < 0.001
                    );
                }
                let lowered = crate::path::FromPaintPath(&PaintPath::from_commands(
                    paint_model::basic_shape_commands::IntoPaintCommands(&resolved),
                    PaintPathFillType::EvenOdd,
                ));
                assert!(
                    native == lowered,
                    "angle={angle} large={large} sweep={sweep}"
                );
            }
        }
    }
}

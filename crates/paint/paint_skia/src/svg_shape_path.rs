//! Native platform Path retained by LayoutSVGShape, shared with hit testing.
use foundation::{
    gfx::{PointF, QuadF, RectF, SizeF},
    native_path::{NativePath, StrokeData},
    LineCap, LineJoin, WindRule,
};
use layoutng_assembly::internal::layout_input::{SvgShapeData, SvgShapeGeometry};
use skia::{
    include::core::SkPathTypes::{SkPathDirection, SkPathFillType},
    src::core::{
        SkPaint::{Cap, Join},
        SkPath::SkPath,
        SkPathBuilder::SkPathBuilder,
        SkPoint::SkPoint,
        SkRRect::SkRRect,
        SkRect::SkRect,
        SkStrokeRec::{InitStyle, SkStrokeRec},
    },
};
use std::sync::Arc;
struct SvgPath(SkPath);
fn rect(r: &SkRect) -> RectF {
    RectF::new(
        PointF::new(r.left(), r.top()),
        SizeF::new(r.width(), r.height()),
    )
}
fn rule(r: WindRule) -> SkPathFillType {
    match r {
        WindRule::RULE_NONZERO => SkPathFillType::kWinding,
        WindRule::RULE_EVENODD => SkPathFillType::kEvenOdd,
    }
}
impl NativePath for SvgPath {
    // platform/geometry/path.cc:177-230. Preserve RawIter and the source
    // exactly-two-quads conversion of a rational conic.
    fn Apply(&self, callback: &mut dyn FnMut(foundation::native_path::NativePathElement)) {
        use foundation::native_path::NativePathElement as E;
        use skia::include::core::SkPathTypes::SkPathVerb as V;
        let mut iter = skia::src::core::SkPath::RawIter::new(&self.0);
        let point = |p: SkPoint| PointF::new(p.x(), p.y());
        while let Some(record) = iter.next() {
            let p = record.fPoints;
            match record.fVerb {
                V::kMove => callback(E::Move(point(p[0]))),
                V::kLine => callback(E::Line(point(p[1]))),
                V::kQuad => callback(E::Quad(point(p[1]), point(p[2]))),
                V::kCubic => callback(E::Cubic(point(p[1]), point(p[2]), point(p[3]))),
                V::kConic => {
                    let mut quads = [SkPoint::default(); 5];
                    SkPath::ConvertConicToQuads(
                        p[0],
                        p[1],
                        p[2],
                        record.fConicWeight,
                        &mut quads,
                        1,
                    );
                    for i in 0..2 {
                        callback(E::Quad(point(quads[1 + 2 * i]), point(quads[2 + 2 * i])));
                    }
                }
                V::kClose => callback(E::Close),
            }
        }
    }

    fn IsEmpty(&self) -> bool {
        self.0.isEmpty()
    }
    fn IsLine(&self) -> bool {
        self.0.isLine(None)
    }
    fn Length(&self) -> f32 {
        let mut measure = skia::src::core::SkPathMeasure::SkPathMeasure::new(&self.0, false, 1.);
        let mut length = 0.;
        loop {
            length += measure.getLength();
            if !measure.nextContour() {
                break;
            }
        }
        length
    }
    fn Transformed(
        &self,
        transform: &foundation::blink_geometry::transforms::affine_transform::AffineTransform,
    ) -> Arc<dyn NativePath> {
        let matrix = skia::src::core::SkMatrix::SkMatrix::MakeAll(
            transform.A() as f32,
            transform.C() as f32,
            transform.E() as f32,
            transform.B() as f32,
            transform.D() as f32,
            transform.F() as f32,
            0.,
            0.,
            1.,
        );
        Arc::new(SvgPath(self.0.makeTransform(&matrix)))
    }
    fn StrokePath(
        &self,
        data: StrokeData,
        dashes: &[f32],
        offset: f32,
        precision: f32,
    ) -> Arc<dyn NativePath> {
        use skia::src::core::{
            SkMatrix::SkMatrix,
            SkPaint::{SkPaint, Style},
            SkPathUtils::FillPathWithPaint,
        };
        let mut paint = SkPaint::default();
        paint.setStyle(Style::kStroke_Style);
        paint.setStrokeWidth(data.width);
        paint.setStrokeMiter(data.miter_limit);
        paint.setStrokeCap(match data.cap {
            LineCap::kButtCap => Cap::kButt_Cap,
            LineCap::kRoundCap => Cap::kRound_Cap,
            LineCap::kSquareCap => Cap::kSquare_Cap,
        });
        paint.setStrokeJoin(match data.join {
            LineJoin::kMiterJoin => Join::kMiter_Join,
            LineJoin::kRoundJoin => Join::kRound_Join,
            LineJoin::kBevelJoin => Join::kBevel_Join,
        });
        if !dashes.is_empty() {
            paint.setPathEffect(skia::src::effects::SkDashPathEffect::Make(
                &foundation::native_path::RepeatedDashIntervals(dashes),
                offset,
            ));
        }
        let mut builder = SkPathBuilder::new();
        FillPathWithPaint(
            &self.0,
            &paint,
            &mut builder,
            None,
            &SkMatrix::Scale(precision, precision),
        );
        Arc::new(SvgPath(builder.detach(None)))
    }
    fn CountVerbs(&self) -> i32 {
        self.0.countVerbs()
    }
    fn BoundingRect(&self) -> RectF {
        rect(self.0.getBounds())
    }
    fn StrokeBoundingRect(&self, data: StrokeData) -> RectF {
        let mut stroke = SkStrokeRec::new(InitStyle::kFill_InitStyle);
        // platform/geometry/path.cc:161-165 uses reduced precision for
        // stroke bounds, then computes the tight curve extrema.
        stroke.setResScale(0.3);
        stroke.setStrokeStyle(data.width, false);
        stroke.setStrokeParams(
            match data.cap {
                LineCap::kButtCap => Cap::kButt_Cap,
                LineCap::kRoundCap => Cap::kRound_Cap,
                LineCap::kSquareCap => Cap::kSquare_Cap,
            },
            match data.join {
                LineJoin::kMiterJoin => Join::kMiter_Join,
                LineJoin::kRoundJoin => Join::kRound_Join,
                LineJoin::kBevelJoin => Join::kBevel_Join,
            },
            data.miter_limit,
        );
        let mut builder = SkPathBuilder::new();
        if !stroke.applyToPath(&mut builder, &self.0) {
            return self.BoundingRect();
        }
        rect(&builder.detach(None).computeTightBounds())
    }
    fn Contains(&self, point: PointF, fill: WindRule) -> bool {
        point.x().is_finite()
            && point.y().is_finite()
            && self
                .0
                .makeFillType(rule(fill))
                .containsXY(point.x(), point.y())
    }
    fn Intersects(&self, quad: &QuadF, fill: WindRule) -> bool {
        super::path_hit_test::path_quad_intersection(&self.0.makeFillType(rule(fill)), quad)
    }
}
pub fn CreateSvgShapePath(shape: &SvgShapeData) -> Arc<dyn NativePath> {
    let bounds = SkRect::MakeXYWH(
        shape.bounds_offset.x as f32,
        shape.bounds_offset.y as f32,
        shape.bounds_width as f32,
        shape.bounds_height as f32,
    );
    let path = match shape.geometry {
        SvgShapeGeometry::kPath => {
            super::path::FromPaintPath(&paint_model::paint_clip_path::PaintPath::from_commands(
                shape.path.clone(),
                Default::default(),
            ))
        }
        SvgShapeGeometry::kRectangle => {
            let radius = SkPoint {
                fX: shape.rectangle_radius_x as f32,
                fY: shape.rectangle_radius_y as f32,
            };
            SkPath::RRectDefault(
                &SkRRect::MakeRectRadii(&bounds, &[radius; 4]),
                SkPathDirection::kCW,
            )
        }
        SvgShapeGeometry::kEllipse => {
            let mut builder = SkPathBuilder::new();
            builder.addOvalDefault(&bounds, SkPathDirection::kCW);
            builder.detach(None)
        }
    };
    Arc::new(SvgPath(path))
}

/// platform/geometry/path_builder.cc:210-218,469-481. Keep these primitive
/// source operations on the same SkPath owner used by SVG Layout and Paint.
pub struct SkiaNativePathFactory;
impl foundation::native_path::NativePathFactory for SkiaNativePathFactory {
    fn Empty(&self) -> Arc<dyn NativePath> {
        Arc::new(SvgPath(SkPath::default()))
    }
    fn Rect(&self, origin: PointF, opposite: PointF) -> Arc<dyn NativePath> {
        let mut builder = SkPathBuilder::new();
        builder.addRect(
            &SkRect::MakeLTRB(origin.x(), origin.y(), opposite.x(), opposite.y()),
            SkPathDirection::kCW,
            0,
        );
        Arc::new(SvgPath(builder.detach(None)))
    }
    fn Ellipse(&self, center: PointF, radius_x: f32, radius_y: f32) -> Arc<dyn NativePath> {
        let mut builder = SkPathBuilder::new();
        builder.addOval(
            &SkRect::MakeLTRB(
                center.x() - radius_x,
                center.y() - radius_y,
                center.x() + radius_x,
                center.y() + radius_y,
            ),
            SkPathDirection::kCW,
            1,
        );
        Arc::new(SvgPath(builder.detach(None)))
    }
    fn Polygon(&self, points: &[PointF], fill: WindRule) -> Arc<dyn NativePath> {
        let mut builder = SkPathBuilder::new();
        if let Some(first) = points.first() {
            builder.moveTo(SkPoint::make(first.x(), first.y()));
            for point in &points[1..] {
                builder.lineTo(SkPoint::make(point.x(), point.y()));
            }
            builder.close();
        }
        builder.setFillType(rule(fill));
        Arc::new(SvgPath(builder.detach(None)))
    }
}

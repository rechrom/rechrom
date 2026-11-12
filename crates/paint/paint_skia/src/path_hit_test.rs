//! Chromium Path hit-test geometry backed by its canonical SkPath and PathOps.
// Source: third_party/blink/renderer/platform/geometry/path.cc:50-61,89-124.
use foundation::gfx::{PointF, QuadF};
use paint_model::{
    paint_clip_path::{PaintPath, PaintPathFillType},
    path_hit_test::{PaintPathHitTest, PaintPathHitTestFactory},
};
use skia::{
    include::{
        core::SkPathTypes::SkPathFillType,
        pathops::SkPathOps::{kIntersect_SkPathOp, Op},
    },
    src::core::{SkPath::SkPath, SkPathBuilder::SkPathBuilder},
};

#[derive(Default)]
pub struct SkiaPathHitTestFactory;
struct SkiaPathHitTest {
    path: SkPath,
}
impl PaintPathHitTestFactory for SkiaPathHitTestFactory {
    fn new_region(&self) -> Box<dyn paint_model::path_hit_test::PaintHitTestRegion> {
        Box::new(NativeRegion(skia::src::core::SkRegion::SkRegion::new()))
    }
    fn create(&self, path: &PaintPath) -> Box<dyn PaintPathHitTest> {
        Box::new(SkiaPathHitTest {
            path: super::path::FromPaintPath(path),
        })
    }
    fn create_basic_shape(
        &self,
        shape: &dyn layoutng_style::style::basic_shapes::BasicShape,
        reference: foundation::gfx::RectF,
        zoom: f32,
        path_scale: f32,
    ) -> Result<Box<dyn PaintPathHitTest>, paint_model::basic_shape_path::InvalidBasicShape> {
        Ok(Box::new(SkiaPathHitTest {
            path: super::basic_shape_path::FromBasicShape(shape, reference, zoom, path_scale)?,
        }))
    }
    fn create_border_shape(
        &self,
        style: &layoutng_style::style::computed_style::ComputedStyle,
        outer_reference: foundation::gfx::RectF,
    ) -> Result<Box<dyn PaintPathHitTest>, paint_model::basic_shape_path::InvalidBasicShape> {
        Ok(Box::new(SkiaPathHitTest {
            path: super::basic_shape_path::BorderShapeOuterPath(style, outer_reference)?,
        }))
    }
    fn create_contoured_rect(
        &self,
        rect: &paint_model::contoured_rect::ContouredRect,
    ) -> Box<dyn PaintPathHitTest> {
        Box::new(SkiaPathHitTest {
            path: super::contoured_rect::FromContouredRect(rect),
        })
    }
}
struct NativeRegion(skia::src::core::SkRegion::SkRegion);
fn irect(rect: &foundation::gfx::Rect) -> skia::src::core::SkRect::SkIRect {
    skia::src::core::SkRect::SkIRect::MakeLTRB(rect.x(), rect.y(), rect.right(), rect.bottom())
}
impl paint_model::path_hit_test::PaintHitTestRegion for NativeRegion {
    fn union(&mut self, rect: foundation::gfx::Rect) {
        self.0
            .op_rect(irect(&rect), skia::src::core::SkRegion::Op::Union);
    }
    fn contains(&self, rect: &foundation::gfx::Rect) -> bool {
        self.0.contains_rect(&irect(rect))
    }
}
fn fill_type(fill: PaintPathFillType) -> SkPathFillType {
    match fill {
        PaintPathFillType::Winding => SkPathFillType::kWinding,
        PaintPathFillType::EvenOdd => SkPathFillType::kEvenOdd,
        PaintPathFillType::InverseWinding => SkPathFillType::kInverseWinding,
        PaintPathFillType::InverseEvenOdd => SkPathFillType::kInverseEvenOdd,
    }
}
pub(crate) fn path_quad_intersection(path: &SkPath, quad: &QuadF) -> bool {
    fn finite(value: f32) -> f32 {
        if value.is_finite() {
            value
        } else {
            0.0
        }
    }
    let points = [quad.p1(), quad.p2(), quad.p3(), quad.p4()];
    let mut builder = SkPathBuilder::new();
    builder.moveToXY(finite(points[0].x()), finite(points[0].y()));
    for point in &points[1..] {
        builder.lineToXY(finite(point.x()), finite(point.y()));
    }
    builder.close();
    let quad_path = builder.detach(None);
    Op(path, &quad_path, kIntersect_SkPathOp).is_some_and(|intersection| !intersection.isEmpty())
}
impl PaintPathHitTest for SkiaPathHitTest {
    fn count_verbs(&self) -> i32 {
        self.path.countVerbs()
    }
    fn contains(&self, point: PointF, fill: Option<PaintPathFillType>) -> bool {
        if !point.x().is_finite() || !point.y().is_finite() {
            return false;
        }
        if let Some(fill) = fill {
            let fill = fill_type(fill);
            if self.path.getFillType() != fill {
                return self
                    .path
                    .makeFillType(fill)
                    .containsXY(point.x(), point.y());
            }
        }
        self.path.containsXY(point.x(), point.y())
    }
    fn intersects(&self, quad: &QuadF, fill: Option<PaintPathFillType>) -> bool {
        if let Some(fill) = fill {
            let fill = fill_type(fill);
            if self.path.getFillType() != fill {
                return path_quad_intersection(&self.path.makeFillType(fill), quad);
            }
        }
        path_quad_intersection(&self.path, quad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint_model::paint_clip_path::{Offset, PaintPathCommand, PaintPathVerb};

    fn command(verb: PaintPathVerb, x: f64, y: f64) -> PaintPathCommand {
        PaintPathCommand {
            verb,
            point: Offset { x, y },
            ..Default::default()
        }
    }
    fn quad(x: f32, y: f32, w: f32, h: f32) -> QuadF {
        QuadF::new(
            PointF::new(x, y),
            PointF::new(x + w, y),
            PointF::new(x + w, y + h),
            PointF::new(x, y + h),
        )
    }
    fn rectangle(x: f64, y: f64, size: f64) -> [PaintPathCommand; 5] {
        [
            command(PaintPathVerb::kMoveTo, x, y),
            command(PaintPathVerb::kLineTo, x + size, y),
            command(PaintPathVerb::kLineTo, x + size, y + size),
            command(PaintPathVerb::kLineTo, x, y + size),
            command(PaintPathVerb::kClose, 0.0, 0.0),
        ]
    }
    #[test]
    fn quad_intersection_keeps_cubic_geometry_instead_of_control_bounds() {
        let path = PaintPath::from_commands(
            vec![
                command(PaintPathVerb::kMoveTo, 0.0, 0.0),
                PaintPathCommand {
                    verb: PaintPathVerb::kCubicTo,
                    control1: Offset { x: 0.0, y: 100.0 },
                    control2: Offset { x: 100.0, y: 100.0 },
                    point: Offset { x: 100.0, y: 0.0 },
                    ..Default::default()
                },
                command(PaintPathVerb::kClose, 0.0, 0.0),
            ],
            PaintPathFillType::Winding,
        );
        let owner = SkiaPathHitTestFactory.create(&path);
        assert!(owner.intersects(&quad(49.0, 70.0, 2.0, 2.0), None));
        assert!(!owner.intersects(&quad(49.0, 90.0, 2.0, 2.0), None));
        // Every quad vertex lies outside the filled curve, but its interior
        // crosses both curve sides. Vertex-only containment cannot detect it.
        assert!(owner.intersects(&quad(-5.0, 40.0, 110.0, 2.0), None));
    }
    #[test]
    fn quad_intersection_preserves_holes_inverse_fill_and_rule_override() {
        let mut commands = rectangle(0.0, 0.0, 100.0).to_vec();
        commands.extend(rectangle(25.0, 25.0, 50.0));
        let path = PaintPath::from_commands(commands, PaintPathFillType::EvenOdd);
        let owner = SkiaPathHitTestFactory.create(&path);
        let hole = quad(40.0, 40.0, 10.0, 10.0);
        assert!(!owner.intersects(&hole, None));
        assert!(owner.intersects(&hole, Some(PaintPathFillType::Winding)));
        assert!(owner.intersects(&hole, Some(PaintPathFillType::InverseEvenOdd)));
        assert!(!owner.intersects(
            &quad(5.0, 5.0, 10.0, 10.0),
            Some(PaintPathFillType::InverseEvenOdd)
        ));
    }
    #[test]
    fn quad_intersection_keeps_rational_conic_weights() {
        let mut commands = vec![command(PaintPathVerb::kMoveTo, 0.0, 50.0)];
        for (cx, cy, x, y) in [
            (0.0, 0.0, 50.0, 0.0),
            (100.0, 0.0, 100.0, 50.0),
            (100.0, 100.0, 50.0, 100.0),
            (0.0, 100.0, 0.0, 50.0),
        ] {
            commands.push(PaintPathCommand {
                verb: PaintPathVerb::kConicTo,
                control1: Offset { x: cx, y: cy },
                point: Offset { x, y },
                conic_weight: std::f64::consts::FRAC_1_SQRT_2,
                ..Default::default()
            });
        }
        commands.push(command(PaintPathVerb::kClose, 0.0, 0.0));
        let owner = SkiaPathHitTestFactory.create(&PaintPath::from_commands(
            commands,
            PaintPathFillType::Winding,
        ));
        assert!(!owner.intersects(&quad(0.0, 0.0, 5.0, 5.0), None));
        assert!(owner.intersects(&quad(98.0, 48.0, 1.0, 1.0), None));
        assert!(owner.intersects(&quad(-5.0, 49.0, 110.0, 2.0), None));
    }
    #[test]
    fn nonfinite_quad_coordinates_follow_chromium_clamping() {
        let path = PaintPath::from_commands(
            rectangle(0.0, 0.0, 100.0).to_vec(),
            PaintPathFillType::Winding,
        );
        let owner = SkiaPathHitTestFactory.create(&path);
        let invalid = QuadF::new(
            PointF::new(f32::NAN, f32::INFINITY),
            PointF::new(10.0, 0.0),
            PointF::new(10.0, 10.0),
            PointF::new(0.0, 10.0),
        );
        assert_eq!(
            owner.intersects(&invalid, None),
            owner.intersects(&quad(0.0, 0.0, 10.0, 10.0), None)
        );
        assert!(!owner.contains(PointF::new(f32::NAN, 0.0), None));
    }
}

//! Native primitives for source PathBuilder::AddContouredRect.
use super::basic_shape_path::{point, rect, Builder};
use foundation::gfx::{PointF, RectF};
use paint_model::contoured_rect::{
    path::{BuildContouredRectPath, ContouredPathBuilder, ContouredPathSink},
    ContouredRect,
};
use skia::{
    include::{
        core::SkPathTypes::{SkPathDirection, SkPathFillType},
        pathops::SkPathOps::{kIntersect_SkPathOp, Op, Simplify},
    },
    src::core::{
        SkPath::SkPath,
        SkPathBuilder::SkPathBuilder,
        SkPathEnums::{SkPathFirstDirection, SkResolveConvexity},
        SkPathPriv::SkPathPriv,
    },
};
impl ContouredPathBuilder for Builder {
    type Path = SkPath;
    fn ConicTo(&mut self, c: PointF, p: PointF, w: f32) {
        self.0.conicTo(point(c), point(p), w);
    }
    fn Rect(&mut self, r: RectF) {
        self.0.addRect(&rect(r), SkPathDirection::kCW, 0);
    }
    fn Detach(mut self) -> SkPath {
        self.0.detach(None)
    }
}
struct Sink;
impl ContouredPathSink for Sink {
    type Path = SkPath;
    type Builder = Builder;
    fn Builder(&self) -> Builder {
        Builder(SkPathBuilder::new())
    }
    fn IntersectPaths(&self, paths: Vec<SkPath>) -> Option<SkPath> {
        let mut paths = paths.into_iter();
        let first = paths.next()?;
        let Some(second) = paths.next() else {
            // The only all-Union input of AddContouredRect is its single
            // target rectangle. Apply SkOpBuilder's exact convex/one-contour
            // branch (SkOpBuilder.cpp:148-209) without a second contour model.
            if first.isConvex() {
                let raw = SkPathPriv::Raw(&first, SkResolveConvexity::kNo)?;
                if SkPathPriv::ComputeFirstDirection(&raw) == SkPathFirstDirection::kUnknown {
                    return Some(first);
                }
            }
            let mut path = Simplify(&first)?;
            if !path.isEmpty() {
                let raw = SkPathPriv::Raw(&path, SkResolveConvexity::kNo)?;
                match SkPathPriv::ComputeFirstDirection(&raw) {
                    SkPathFirstDirection::kCW => {
                        let mut reverse = SkPathBuilder::new();
                        reverse.moveTo(path.getLastPt()?);
                        SkPathPriv::ReversePathTo(&mut reverse, &path);
                        reverse.close();
                        path = reverse.detach(None);
                    }
                    SkPathFirstDirection::kCCW => {}
                    // A simplified native rectangle cannot have multiple or
                    // unknown-direction contours unless it is empty above.
                    SkPathFirstDirection::kUnknown => return None,
                }
                path.setFillType(SkPathFillType::kWinding);
            }
            return Simplify(&path);
        };
        // Source SkOpBuilder.cpp:177-192: any Intersect operand selects this
        // ordered Op branch; no all-Union optimization applies.
        let mut result = Op(&first, &second, kIntersect_SkPathOp)?;
        for path in paths {
            result = Op(&result, &path, kIntersect_SkPathOp)?;
        }
        Some(result)
    }
}
pub(super) fn FromContouredRect(rect: &ContouredRect) -> SkPath {
    BuildContouredRectPath(rect, &Sink)
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::gfx::{OutsetsF, QuadF, SizeF};
    use paint_model::contoured_rect::{CornerCurvature, FloatRoundedRect, Radii};
    fn shape(curvature: f32) -> ContouredRect {
        let rounded = FloatRoundedRect::new(
            RectF::new(PointF::new(0., 0.), SizeF::new(100., 100.)),
            Radii([SizeF::new(20., 20.); 4]),
        );
        ContouredRect {
            rounded,
            curvature: CornerCurvature([curvature; 4]),
            origin: Some(rounded),
        }
    }
    #[test]
    fn source_superellipse_corners_and_inset_pathops_keep_curves() {
        let squircle = FromContouredRect(&shape(4.));
        assert!(squircle.getSegmentMasks() & 8 != 0);
        assert!(squircle.containsXY(5., 5.));
        let scoop = FromContouredRect(&shape(0.5));
        assert!(scoop.getSegmentMasks() & 4 != 0);
        assert!(!scoop.containsXY(5., 5.));
        assert!(scoop.containsXY(15., 15.));
        let bevel = FromContouredRect(&shape(1.));
        assert!(!bevel.containsXY(5., 5.));
        assert!(bevel.containsXY(12., 12.));
        let mut inner = shape(4.);
        inner.Inset(OutsetsF::new(5., 10., 15., 8.));
        let path = FromContouredRect(&inner);
        assert!(path.containsXY(50., 50.));
        assert!(!path.containsXY(2., 50.));
        assert!(!path.containsXY(8., 5.));
        let bounds = path.getBounds();
        let target = inner.Rect();
        assert!(
            bounds.fLeft >= target.x()
                && bounds.fTop >= target.y()
                && bounds.fRight <= target.right()
                && bounds.fBottom <= target.bottom()
        );
    }
    #[test]
    fn round_quad_tangency_keeps_source_edge_inclusive_semantics() {
        let round = shape(2.);
        let point = PointF::new(0., 20.);
        let quad = QuadF::new(point, point, point, point);
        assert!(round.rounded.IntersectsQuad(&quad));
        let point = PointF::new(0., 0.);
        assert!(!round
            .rounded
            .IntersectsQuad(&QuadF::new(point, point, point, point)));
    }
}

//! Source PathBuilder::AddContouredRect and corner path primitives.
//! platform/geometry/path_builder.cc:24-103,238-399. The cubic superellipse
//! construction below is Chromium's source formula, not a replacement fit.
use super::{ContouredRect, Corner, FloatRoundedRect};
use crate::basic_shape_path::BasicShapePathSink;
use foundation::{
    gfx::{PointF, RectF, Vector2dF},
    gfx_line_f::LineF,
};
pub trait ContouredPathBuilder: BasicShapePathSink {
    type Path;
    fn ConicTo(&mut self, control: PointF, end: PointF, weight: f32);
    fn Rect(&mut self, rect: RectF);
    fn Detach(self) -> Self::Path;
}
pub trait ContouredPathSink {
    type Path;
    type Builder: ContouredPathBuilder<Path = Self::Path>;
    fn Builder(&self) -> Self::Builder;
    /// Native SkOpBuilder input is a first Union rect followed only by
    /// Intersect operands. Preserve operand order and source failure result.
    fn IntersectPaths(&self, paths: Vec<Self::Path>) -> Option<Self::Path>;
}
fn half_corner(curvature: f32) -> [Vector2dF; 3] {
    const P: [f64; 7] = [
        1.2430920942724248,
        2.010479023614843,
        0.32922901179443753,
        0.2823023142212073,
        1.3473704261055421,
        2.9149468637949814,
        0.9106507102917086,
    ];
    debug_assert!(curvature >= 1.);
    let s = curvature.log2();
    let slope = (P[0] + (P[6] - P[0]) * 0.5 * (1. + (P[5] * (s as f64 - P[1])).tanh())) as f32;
    let base = (1. / (1. + (-(slope as f64) * (0. - P[1])).exp())) as f32;
    let logistic = (1. / (1. + (-(slope as f64) * (s as f64 - P[1])).exp())) as f32;
    let a = (logistic - base) / (1. - base);
    let b = (P[2] * (-P[3] * (s as f64).powf(P[4])).exp()) as f32;
    let h = Corner::HalfCornerForCurvature(curvature);
    [
        Vector2dF::new(a, 1.),
        Vector2dF::new(h - b, h + b),
        Vector2dF::new(h, h),
    ]
}
pub fn AddCurvedCorner<B: ContouredPathBuilder>(path: &mut B, corner: Corner) {
    if corner.IsConcave() {
        AddCurvedCorner(path, corner.Inverse());
        return;
    }
    assert!(corner.Curvature() >= 1.);
    path.LineTo(corner.Start());
    if corner.IsStraight() || corner.IsEmpty() {
        path.LineTo(corner.Outer());
        path.LineTo(corner.End());
    } else if corner.IsBevel() {
        path.LineTo(corner.End());
    } else if corner.IsRound() {
        path.ConicTo(
            corner.Outer(),
            corner.End(),
            std::f32::consts::FRAC_1_SQRT_2,
        );
    } else {
        let [a, b, c] = half_corner(corner.Curvature());
        path.CubicTo(corner.MapPoint(a), corner.MapPoint(b), corner.MapPoint(c));
        path.CubicTo(
            corner.MapPoint(Vector2dF::new(b.y(), b.x())),
            corner.MapPoint(Vector2dF::new(a.y(), a.x())),
            corner.End(),
        );
    }
}
fn single<S: ContouredPathSink>(rect: &ContouredRect, sink: &S) -> S::Path {
    let mut path = sink.Builder();
    path.MoveTo(rect.TopRightCorner().Start());
    for index in [1, 2, 3, 0] {
        AddCurvedCorner(&mut path, rect.Corner(index));
    }
    path.Close();
    path.Detach()
}
fn rounded<S: ContouredPathSink>(rect: FloatRoundedRect, sink: &S) -> S::Path {
    let mut path = sink.Builder();
    path.RoundedRect(
        rect.rect,
        rect.radii.0.map(|r| PointF::new(r.width(), r.height())),
    );
    path.Detach()
}
pub fn BuildContouredRectPath<S: ContouredPathSink>(rect: &ContouredRect, sink: &S) -> S::Path {
    let target = rect.AsRoundedRect();
    if rect.HasRoundCurvature() {
        return rounded(target, sink);
    }
    let origin = rect.GetOriginRect();
    if origin == target {
        return single(rect, sink);
    }
    if target.rect.Contains(&origin.rect) {
        fn miter(corner: Corner, edge: LineF, reference: PointF) -> PointF {
            let control = if corner.IsEmpty() {
                reference + edge.Normal()
            } else {
                corner.QuadraticControlPoint()
            };
            edge.IntersectionWith(&LineF::new(control, reference))
                .unwrap_or(reference)
        }
        let r = target.rect;
        let tl = r.origin();
        let tr = PointF::new(r.right(), r.y());
        let br = PointF::new(r.right(), r.bottom());
        let bl = PointF::new(r.x(), r.bottom());
        let edges = [
            LineF::new(tl, tr),
            LineF::new(tr, br),
            LineF::new(bl, br),
            LineF::new(tl, bl),
        ];
        let mut path = sink.Builder();
        for (step, index) in [1, 2, 3, 0].into_iter().enumerate() {
            let corner = rect.Corner(index);
            let start = miter(corner, edges[step], corner.Start());
            if step == 0 {
                path.MoveTo(start);
            } else {
                path.LineTo(start);
            }
            AddCurvedCorner(&mut path, corner);
            path.LineTo(miter(corner, edges[(step + 1) % 4], corner.End()));
        }
        path.Close();
        return path.Detach();
    }
    // Source clips the target rectangle by the exact native curved-corner
    // paths. These are PathOps intersections, never corner bounds clipping.
    let infinite = RectF::from(foundation::infinite_int_rect::InfiniteIntRect());
    let (left, top, right, bottom) = (
        infinite.x(),
        infinite.y(),
        infinite.right(),
        infinite.bottom(),
    );
    let points = [
        PointF::new(left, top),
        PointF::new(right, top),
        PointF::new(right, bottom),
        PointF::new(left, bottom),
    ];
    let mut target_path = sink.Builder();
    target_path.Rect(target.rect);
    let mut paths = vec![target_path.Detach()];
    for (step, index) in [1, 2, 3, 0].into_iter().enumerate() {
        if origin.radii.0[index].IsEmpty() {
            continue;
        }
        let mut path = sink.Builder();
        path.MoveTo(points[step]);
        AddCurvedCorner(&mut path, rect.Corner(index));
        path.LineTo(points[(step + 2) % 4]);
        path.LineTo(points[(step + 3) % 4]);
        path.Close();
        paths.push(path.Detach());
    }
    // This is the source's own SkOpBuilder resolve-failure branch.
    sink.IntersectPaths(paths)
        .unwrap_or_else(|| single(rect, sink))
}

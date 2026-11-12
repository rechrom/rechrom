//! Source ContouredRect; platform/geometry/contoured_rect.h:31-362,
//! contoured_rect.cc:24-208. Source path construction is in `path`.
pub mod border;
pub mod path;
pub mod rounded;
use foundation::gfx::{OutsetsF, PointF, QuadF, RectF, SizeF, Vector2dF};
pub use rounded::{FloatRoundedRect, Radii};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerCurvature(pub [f32; 4]);
impl Default for CornerCurvature {
    fn default() -> Self {
        Self([Self::ROUND; 4])
    }
}
impl CornerCurvature {
    pub const ROUND: f32 = 2.;
    pub const BEVEL: f32 = 1.;
    pub const SCOOP: f32 = 0.5;
    pub const STRAIGHT: f32 = 1000.;
    pub const NOTCH: f32 = 1. / Self::STRAIGHT;
    pub fn IsUniform(&self) -> bool {
        self.0[1..].iter().all(|v| *v == self.0[0])
    }
    pub fn IsRound(&self) -> bool {
        self.0[0] == Self::ROUND && self.IsUniform()
    }
    pub fn IsConvex(&self) -> bool {
        self.0.iter().all(|v| *v >= Self::BEVEL)
    }
    pub fn IsHyperellipse(&self) -> bool {
        self.0.iter().all(|v| *v >= Self::ROUND)
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corner {
    vertices: [PointF; 4],
    curvature: f32,
}
fn mul(v: Vector2dF, s: f32) -> Vector2dF {
    Vector2dF::new(v.x() * s, v.y() * s)
}
fn normal(mut v: Vector2dF) -> Vector2dF {
    v.Normalize();
    v
}
impl Corner {
    pub fn FromRect(rect: RectF, rotation: usize, curvature: f32) -> Self {
        let mut vertices = [
            rect.origin(),
            PointF::new(rect.right(), rect.y()),
            PointF::new(rect.right(), rect.bottom()),
            PointF::new(rect.x(), rect.bottom()),
        ];
        vertices.rotate_left(rotation);
        Self {
            vertices,
            curvature,
        }
    }
    pub fn new(vertices: [PointF; 4], curvature: f32) -> Self {
        Self {
            vertices,
            curvature: Self::ClampCurvature(curvature),
        }
    }
    pub fn ClampCurvature(value: f32) -> f32 {
        value.clamp(CornerCurvature::NOTCH, CornerCurvature::STRAIGHT)
    }
    pub fn Start(&self) -> PointF {
        self.vertices[0]
    }
    pub fn Outer(&self) -> PointF {
        self.vertices[1]
    }
    pub fn End(&self) -> PointF {
        self.vertices[2]
    }
    pub fn Center(&self) -> PointF {
        self.vertices[3]
    }
    pub fn Curvature(&self) -> f32 {
        self.curvature
    }
    pub fn IsStraight(&self) -> bool {
        self.curvature == CornerCurvature::STRAIGHT
    }
    pub fn IsBevel(&self) -> bool {
        self.curvature == CornerCurvature::BEVEL
    }
    pub fn IsRound(&self) -> bool {
        self.curvature == CornerCurvature::ROUND
    }
    pub fn IsNotch(&self) -> bool {
        self.curvature == CornerCurvature::NOTCH
    }
    pub fn IsConcave(&self) -> bool {
        self.curvature < 1.
    }
    pub fn IsHyperellipse(&self) -> bool {
        self.curvature >= 2.
    }
    pub fn IsZero(&self) -> bool {
        self.Start() == self.End()
    }
    pub fn IsEmpty(&self) -> bool {
        self.v1().Length() == 0. || self.v2().Length() == 0.
    }
    pub fn Inverse(&self) -> Self {
        Self::new(
            [self.Start(), self.Center(), self.End(), self.Outer()],
            1. / self.curvature,
        )
    }
    pub fn Reverse(&self) -> Self {
        Self::new(
            [self.End(), self.Outer(), self.Start(), self.Center()],
            self.curvature,
        )
    }
    pub fn BoundingBox(&self) -> RectF {
        let a = self.Start();
        let b = self.End();
        RectF::new(
            PointF::new(a.x().min(b.x()), a.y().min(b.y())),
            SizeF::new((a.x() - b.x()).abs(), (a.y() - b.y()).abs()),
        )
    }
    pub fn Intersects(&self, other: &Self) -> bool {
        self.BoundingBox().Intersects(&other.BoundingBox())
    }
    pub fn v1(&self) -> Vector2dF {
        self.Outer() - self.Start()
    }
    pub fn v2(&self) -> Vector2dF {
        self.End() - self.Outer()
    }
    pub fn v3(&self) -> Vector2dF {
        self.Center() - self.End()
    }
    pub fn v4(&self) -> Vector2dF {
        self.Start() - self.Center()
    }
    pub fn DiagonalLength(&self) -> f32 {
        (self.End() - self.Start()).Length()
    }
    pub fn HalfCornerForCurvature(value: f32) -> f32 {
        0.5_f64.powf((1. / Self::ClampCurvature(value)) as f64) as f32
    }
    pub fn CurvatureForHalfCorner(value: f32) -> f32 {
        if value >= 1. {
            CornerCurvature::STRAIGHT
        } else if value <= 0. {
            CornerCurvature::NOTCH
        } else {
            (0.5_f64.ln() / value.ln() as f64) as f32
        }
    }
    pub fn HalfCorner(&self) -> PointF {
        let h = Self::HalfCornerForCurvature(self.curvature);
        self.MapPoint(Vector2dF::new(h, h))
    }
    pub fn MapPoint(&self, p: Vector2dF) -> PointF {
        self.Center() + mul(self.v1(), p.x()) + mul(self.v4(), p.y())
    }
    pub fn QuadraticControlPoint(&self) -> PointF {
        if self.IsConcave() {
            return self.Inverse().QuadraticControlPoint();
        }
        if self.curvature >= CornerCurvature::ROUND {
            return self.Outer();
        }
        let p = 2. * Self::HalfCornerForCurvature(self.curvature) - 0.5;
        self.MapPoint(Vector2dF::new(p, p))
    }
    pub fn AlignedToOrigin(&self, origin: &Self, thickness_start: f32, thickness_end: f32) -> Self {
        if origin.IsEmpty() || self == origin {
            return *self;
        }
        let curvature = origin.curvature;
        let half = Self::HalfCornerForCurvature(curvature.clamp(0.5, 2.));
        let hull = normal(Vector2dF::new(half * 2. - 0.5, (1. - half) * 2. - 0.5));
        let offset = Vector2dF::new(hull.x(), -hull.y());
        let a = mul(normal(origin.v1()), thickness_start * offset.y());
        let b = mul(normal(origin.v2()), thickness_start * offset.x());
        let c = mul(normal(origin.v3()), thickness_end * offset.x());
        let d = mul(normal(origin.v4()), thickness_end * offset.y());
        Self::new(
            [
                origin.Start() + a + b,
                origin.Outer() + b + c,
                origin.End() + c + d,
                origin.Center() + d + a,
            ],
            curvature,
        )
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ContouredRect {
    pub rounded: FloatRoundedRect,
    pub curvature: CornerCurvature,
    pub origin: Option<FloatRoundedRect>,
}
impl ContouredRect {
    pub fn new(rounded: FloatRoundedRect) -> Self {
        Self {
            rounded,
            ..Default::default()
        }
    }
    pub fn Rect(&self) -> RectF {
        self.rounded.rect
    }
    pub fn BoundingRect(&self) -> RectF {
        self.Rect()
    }
    pub fn HasRoundCurvature(&self) -> bool {
        self.curvature.IsRound() || !self.IsRounded()
    }
    pub fn IsConvex(&self) -> bool {
        !self.IsRounded() || self.curvature.IsConvex()
    }
    pub fn IsRounded(&self) -> bool {
        self.rounded.IsRounded() || self.origin.is_some_and(|r| r.IsRounded())
    }
    pub fn IsEmpty(&self) -> bool {
        self.rounded.IsEmpty()
    }
    pub fn GetRadii(&self) -> Radii {
        self.rounded.radii
    }
    pub fn SetRadii(&mut self, radii: Radii) {
        self.rounded.radii = radii;
    }
    pub fn GetCornerCurvature(&self) -> CornerCurvature {
        self.curvature
    }
    pub fn SetCornerCurvature(&mut self, curvature: CornerCurvature) {
        self.curvature = curvature;
    }
    pub fn SetOriginRect(&mut self, origin: FloatRoundedRect) {
        self.origin = Some(origin);
    }
    pub fn GetOriginRect(&self) -> FloatRoundedRect {
        self.origin.unwrap_or(self.rounded)
    }
    pub fn IsInnerRect(&self) -> bool {
        self.origin.is_some_and(|r| r != self.rounded)
    }
    pub fn AsRoundedRect(&self) -> FloatRoundedRect {
        self.rounded
    }
    pub fn Move(&mut self, v: Vector2dF) {
        self.rounded.Move(v);
    }
    pub fn Inset(&mut self, o: OutsetsF) {
        self.rounded.Inset(o);
    }
    pub fn Outset(&mut self, o: OutsetsF) {
        self.rounded.Outset(o);
    }
    pub fn OutsetWithCornerCorrection(&mut self, o: OutsetsF) {
        self.rounded.OutsetWithCornerCorrection(o);
    }
    pub fn OutsetForShapeMargin(&mut self, o: f32) {
        self.rounded.OutsetForShapeMargin(o);
    }
    pub fn ConstrainRadii(&mut self) {
        self.rounded.ConstrainRadii();
    }
    pub fn IsRenderable(&self) -> bool {
        self.rounded.IsRenderable()
    }
    fn internal_corner(&self, index: usize) -> Corner {
        Corner::FromRect(
            self.rounded.CornerRect(index),
            (index + 3) % 4,
            self.curvature.0[index],
        )
    }
    pub fn Corner(&self, index: usize) -> Corner {
        let target = self.internal_corner(index);
        if !self.IsInnerRect() {
            return target;
        }
        let origin = self.GetOriginRect();
        let original = Self {
            rounded: origin,
            curvature: self.curvature,
            origin: None,
        }
        .internal_corner(index);
        let r = self.Rect();
        let o = origin.rect;
        let (start, end) = match index {
            0 => (r.x() - o.x(), r.y() - o.y()),
            1 => (r.y() - o.y(), o.right() - r.right()),
            2 => (o.right() - r.right(), o.bottom() - r.bottom()),
            _ => (o.bottom() - r.bottom(), r.x() - o.x()),
        };
        target.AlignedToOrigin(&original, start, end)
    }
    pub fn TopLeftCorner(&self) -> Corner {
        self.Corner(0)
    }
    pub fn TopRightCorner(&self) -> Corner {
        self.Corner(1)
    }
    pub fn BottomRightCorner(&self) -> Corner {
        self.Corner(2)
    }
    pub fn BottomLeftCorner(&self) -> Corner {
        self.Corner(3)
    }
    pub fn IntersectsQuad(
        &self,
        quad: &QuadF,
        factory: &dyn crate::path_hit_test::PaintPathHitTestFactory,
    ) -> bool {
        if self.HasRoundCurvature() {
            self.rounded.IntersectsQuad(quad)
        } else {
            factory.create_contoured_rect(self).intersects(quad, None)
        }
    }
    pub fn GetPath(
        &self,
        factory: &dyn crate::path_hit_test::PaintPathHitTestFactory,
    ) -> Box<dyn crate::path_hit_test::PaintPathHitTest> {
        factory.create_contoured_rect(self)
    }
    pub fn XInterceptsAtY(&self, y: f32) -> Option<(f32, f32)> {
        let rect = self.Rect();
        if y < rect.y() || y > rect.bottom() {
            return None;
        }
        if !self.IsRounded() {
            return Some((rect.x(), rect.right()));
        }
        fn intercept(y: f32, rect: RectF, k: f32) -> f32 {
            debug_assert!(rect.height() > 0.);
            if k == CornerCurvature::ROUND {
                return rect.width() * (1. - (y * y) / (rect.height() * rect.height())).sqrt();
            }
            if k < CornerCurvature::BEVEL {
                return rect.width() - intercept(rect.height() - y, rect, 1. / k);
            }
            rect.width() * (1. - (y / rect.height()).powf(k)).powf(1. / k)
        }
        let tl = self.rounded.CornerRect(0);
        let bl = self.rounded.CornerRect(3);
        let tr = self.rounded.CornerRect(1);
        let br = self.rounded.CornerRect(2);
        let min = if !tl.IsEmpty() && y >= tl.y() && y < tl.bottom() {
            tl.right() - intercept(tl.bottom() - y, tl, self.curvature.0[0])
        } else if !bl.IsEmpty() && y >= bl.y() && y <= bl.bottom() {
            bl.right() - intercept(y - bl.y(), bl, self.curvature.0[3])
        } else {
            rect.x()
        };
        let max = if !tr.IsEmpty() && y >= tr.y() && y <= tr.bottom() {
            tr.x() + intercept(tr.bottom() - y, tr, self.curvature.0[1])
        } else if !br.IsEmpty() && y >= br.y() && y <= br.bottom() {
            br.x() + intercept(y - br.y(), br, self.curvature.0[2])
        } else {
            rect.right()
        };
        Some((min, max))
    }
}

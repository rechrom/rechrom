// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/math_util.h:1-371
// Source: cc/base/math_util.cc:1-1128
use crate::adapter::geometry::*;

// Source: cc/base/math_util.h:48-100
#[derive(Clone, Copy, Debug, Default)]
pub struct HomogeneousCoordinate {
    pub vec: [f32; 4],
}
impl HomogeneousCoordinate {
    // Source: cc/base/math_util.h:49-55
    pub const kInfiniteCoordinate: f32 = 1_000_000.0;
    // Source: cc/base/math_util.h:57-62
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { vec: [x, y, z, w] }
    }
    // Source: cc/base/math_util.h:64-64
    pub fn ShouldBeClipped(&self) -> bool {
        self.w() <= 0.0
    }
    // Source: cc/base/math_util.h:66-80
    pub fn CartesianPoint2d(&self) -> PointF {
        if self.w() == 1.0 {
            return PointF::new(self.x(), self.y());
        }
        debug_assert!(self.w() != 0.0);
        let inv_w = 1.0 / self.w();
        PointF::new(
            cpp_clamp(
                self.x() * inv_w,
                -Self::kInfiniteCoordinate,
                Self::kInfiniteCoordinate,
            ),
            cpp_clamp(
                self.y() * inv_w,
                -Self::kInfiniteCoordinate,
                Self::kInfiniteCoordinate,
            ),
        )
    }
    // Source: cc/base/math_util.h:82-93
    pub fn CartesianPoint3dUnclamped(&self) -> Point3F {
        if self.w() == 1.0 {
            return Point3F::new(self.x(), self.y(), self.z());
        }
        debug_assert!(self.w() != 0.0);
        let inv_w = 1.0 / self.w();
        Point3F::new(self.x() * inv_w, self.y() * inv_w, self.z() * inv_w)
    }
    // Source: cc/base/math_util.h:95-95
    pub fn x(&self) -> f32 {
        self.vec[0]
    }
    pub fn y(&self) -> f32 {
        self.vec[1]
    }
    pub fn z(&self) -> f32 {
        self.vec[2]
    }
    pub fn w(&self) -> f32 {
        self.vec[3]
    }
}

// Rust equivalents of std::integral template constraints. Wrapping operations
// implement unchecked integer arithmetic without adding debug-only panics.
pub trait RoundInteger: Copy + PartialOrd + std::ops::Rem<Output = Self> {
    const ZERO: Self;
    const MIN: Self;
    const MAX: Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}
macro_rules! round_integers {
    ($($integer:ty),*) => { $(impl RoundInteger for $integer {
        const ZERO: Self = 0; const MIN: Self = <$integer>::MIN; const MAX: Self = <$integer>::MAX;
        fn add(self, other: Self) -> Self { self.wrapping_add(other) }
        fn sub(self, other: Self) -> Self { self.wrapping_sub(other) }
    })* };
}
round_integers!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
pub trait EpsilonFloat: Copy + std::ops::Sub<Output = Self> + PartialOrd {
    const EPSILON: Self;
    fn abs(self) -> Self;
}
impl EpsilonFloat for f32 {
    const EPSILON: Self = f32::EPSILON;
    fn abs(self) -> Self {
        self.abs()
    }
}
impl EpsilonFloat for f64 {
    const EPSILON: Self = f64::EPSILON;
    fn abs(self) -> Self {
        self.abs()
    }
}

pub struct MathUtil;
impl MathUtil {
    // Source: cc/base/math_util.h:108-111
    pub fn VerifyRoundup<T: RoundInteger>(n: T, mul: T) -> bool {
        mul != T::ZERO && n <= T::MAX.sub(T::MAX % mul)
    }
    // Source: cc/base/math_util.h:119-121
    pub fn UncheckedRoundUp<T: RoundInteger>(n: T, mul: T) -> T {
        Self::RoundUpInternal(n, mul)
    }
    // Source: cc/base/math_util.h:127-130
    pub fn CheckedRoundUp<T: RoundInteger>(n: T, mul: T) -> T {
        assert!(Self::VerifyRoundup(n, mul));
        Self::RoundUpInternal(n, mul)
    }
    // Source: cc/base/math_util.h:135-138
    pub fn VerifyRoundDown<T: RoundInteger>(n: T, mul: T) -> bool {
        mul != T::ZERO && n >= T::MIN.sub(T::MIN % mul)
    }
    // Source: cc/base/math_util.h:146-148
    pub fn UncheckedRoundDown<T: RoundInteger>(n: T, mul: T) -> T {
        Self::RoundDownInternal(n, mul)
    }
    // Source: cc/base/math_util.h:154-157
    pub fn CheckedRoundDown<T: RoundInteger>(n: T, mul: T) -> T {
        assert!(Self::VerifyRoundDown(n, mul));
        Self::RoundDownInternal(n, mul)
    }
    // Source: cc/base/math_util.h:160-162
    pub fn IsWithinEpsilon<T: EpsilonFloat>(a: T, b: T) -> bool {
        (a - b).abs() < T::EPSILON
    }
    // Source: cc/base/math_util.h:336-342
    fn RoundUpInternal<T: RoundInteger>(n: T, mul: T) -> T {
        let remainder = n % mul;
        if remainder == T::ZERO {
            return n;
        }
        if n > T::ZERO {
            n.add(mul).sub(remainder)
        } else {
            n.sub(remainder)
        }
    }
    // Source: cc/base/math_util.h:345-351
    fn RoundDownInternal<T: RoundInteger>(n: T, mul: T) -> T {
        let remainder = n % mul;
        if remainder == T::ZERO {
            return n;
        }
        if n > T::ZERO {
            n.sub(remainder)
        } else {
            n.sub(mul).sub(remainder)
        }
    }
}

// Source: cc/base/math_util.cc:34-57
fn ProjectHomogeneousPoint<T: Transform>(transform: &T, p: PointF) -> HomogeneousCoordinate {
    let m22 = transform.rc(2, 2) as f32;
    if !m22.is_normal() {
        return HomogeneousCoordinate::new(0.0, 0.0, 0.0, 0.0);
    }
    let z = (-(transform.rc(2, 0) * p.x() as f64
        + transform.rc(2, 1) * p.y() as f64
        + transform.rc(2, 3))
        / m22 as f64) as f32;
    if !z.is_finite() {
        return HomogeneousCoordinate::new(0.0, 0.0, 0.0, 0.0);
    }
    let mut result = HomogeneousCoordinate::new(p.x(), p.y(), z, 1.0);
    transform.TransformVector4(&mut result.vec);
    result
}
// Source: cc/base/math_util.cc:59-66
fn ProjectHomogeneousPointWithClipped<T: Transform>(
    transform: &T,
    p: PointF,
    clipped: &mut bool,
) -> HomogeneousCoordinate {
    let h = ProjectHomogeneousPoint(transform, p);
    *clipped = h.w() <= 0.0;
    h
}
// Source: cc/base/math_util.cc:68-74
fn MapHomogeneousPoint<T: Transform>(transform: &T, p: PointF) -> HomogeneousCoordinate {
    let mut result = HomogeneousCoordinate::new(p.x(), p.y(), 0.0, 1.0);
    transform.TransformVector4(&mut result.vec);
    result
}
// Source: cc/base/math_util.cc:76-81
const kStationaryPointEpsilon: f32 = 0.00001;
// Source: cc/base/math_util.cc:83-100
fn homogeneousLimitAtZero(a1: f32, w1: f32, a2: f32, w2: f32, t: f32, limit: &mut f32) {
    if (a1 * w2 / w1 / a2 - 1.0).abs() > kStationaryPointEpsilon {
        if ((1.0 - t) * a1 + t * a2) > 0.0 {
            *limit = HomogeneousCoordinate::kInfiniteCoordinate;
        } else {
            *limit = -HomogeneousCoordinate::kInfiniteCoordinate;
        }
    } else {
        *limit = a1 / w1;
    }
}
// Source: cc/base/math_util.cc:102-134
fn ComputeClippedCartesianPoint2dForEdge(
    h1: HomogeneousCoordinate,
    h2: HomogeneousCoordinate,
) -> PointF {
    debug_assert_ne!(h1.w() <= 0.0, h2.w() <= 0.0);
    let t = h1.w() / (h1.w() - h2.w());
    let mut x = 0.0;
    let mut y = 0.0;
    homogeneousLimitAtZero(h1.x(), h1.w(), h2.x(), h2.w(), t, &mut x);
    homogeneousLimitAtZero(h1.y(), h1.w(), h2.y(), h2.w(), t, &mut y);
    PointF::new(x, y)
}
// Source: cc/base/math_util.cc:136-153
fn homogeneousLimitNearZero(a1: f32, w1: f32, a2: f32, w2: f32, t: f32, limit: &mut f32) {
    if (a1 * w2 / w1 / a2 - 1.0).abs() > kStationaryPointEpsilon {
        *limit = ((1.0 - t) * a1 + t * a2) / ((1.0 - t) * w1 + t * w2);
    } else {
        *limit = a1 / w1;
    }
}
// Source: cc/base/math_util.cc:155-215
fn ComputeClippedCartesianPoint3dForEdge(
    h1: HomogeneousCoordinate,
    h2: HomogeneousCoordinate,
) -> Point3F {
    debug_assert_ne!(h1.w() <= 0.0, h2.w() <= 0.0);
    let w_diff = h1.w() - h2.w();
    let mut t = h1.w() / w_diff;
    let max_numerator = cpp_max(
        cpp_max(
            ((1.0 - t) * h1.x() + t * h2.x()).abs(),
            ((1.0 - t) * h1.y() + t * h2.y()).abs(),
        ),
        ((1.0 - t) * h1.z() + t * h2.z()).abs(),
    );
    let t_shift = max_numerator / w_diff / HomogeneousCoordinate::kInfiniteCoordinate;
    const HALF_EPSILON: f32 = f32::EPSILON / 2.0;
    debug_assert_eq!(w_diff > 0.0, t_shift > 0.0);
    if w_diff > 0.0 {
        t = cpp_max(0.0, t - (t_shift + HALF_EPSILON));
    } else {
        t = cpp_min(1.0, t - (t_shift - HALF_EPSILON));
    }
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    homogeneousLimitNearZero(h1.x(), h1.w(), h2.x(), h2.w(), t, &mut x);
    homogeneousLimitNearZero(h1.y(), h1.w(), h2.y(), h2.w(), t, &mut y);
    homogeneousLimitNearZero(h1.z(), h1.w(), h2.z(), h2.w(), t, &mut z);
    Point3F::new(x, y, z)
}
// Source: cc/base/math_util.cc:217-226
fn ExpandBoundsToIncludePoint(
    xmin: &mut f32,
    xmax: &mut f32,
    ymin: &mut f32,
    ymax: &mut f32,
    p: PointF,
) {
    *xmin = cpp_min(p.x(), *xmin);
    *xmax = cpp_max(p.x(), *xmax);
    *ymin = cpp_min(p.y(), *ymin);
    *ymax = cpp_max(p.y(), *ymax);
}
// Source: cc/base/math_util.cc:228-236
fn IsNearlyTheSame(f: f32, g: f32) -> bool {
    const EPSILON_SCALE: f32 = 0.00001;
    (f - g).abs() < EPSILON_SCALE * cpp_max(cpp_max(f.abs(), g.abs()), EPSILON_SCALE)
}
// Source: cc/base/math_util.cc:238-241
fn IsNearlyTheSamePoint2(lhs: PointF, rhs: PointF) -> bool {
    IsNearlyTheSame(lhs.x(), rhs.x()) && IsNearlyTheSame(lhs.y(), rhs.y())
}
// Source: cc/base/math_util.cc:243-247
fn IsNearlyTheSamePoint3(lhs: Point3F, rhs: Point3F) -> bool {
    IsNearlyTheSame(lhs.x(), rhs.x())
        && IsNearlyTheSame(lhs.y(), rhs.y())
        && IsNearlyTheSame(lhs.z(), rhs.z())
}
// Source: cc/base/math_util.cc:249-274
fn AddVertexToClippedQuad3d(
    new_vertex: Point3F,
    clipped_quad: &mut [Point3F; 6],
    num_vertices: &mut i32,
    need_to_clamp: &mut bool,
) {
    assert!(*num_vertices >= 0);
    if *num_vertices > 0
        && IsNearlyTheSamePoint3(clipped_quad[(*num_vertices - 1) as usize], new_vertex)
    {
        return;
    }
    assert!(*num_vertices < 6);
    clipped_quad[*num_vertices as usize] = new_vertex;
    *num_vertices += 1;
    if new_vertex.x() < -HomogeneousCoordinate::kInfiniteCoordinate
        || new_vertex.x() > HomogeneousCoordinate::kInfiniteCoordinate
        || new_vertex.y() < -HomogeneousCoordinate::kInfiniteCoordinate
        || new_vertex.y() > HomogeneousCoordinate::kInfiniteCoordinate
        || new_vertex.z() < -HomogeneousCoordinate::kInfiniteCoordinate
        || new_vertex.z() > HomogeneousCoordinate::kInfiniteCoordinate
    {
        *need_to_clamp = true;
    }
}

impl MathUtil {
    // Source: cc/base/math_util.h:172-173
    // Source: cc/base/math_util.cc:276-279
    pub fn MapEnclosingClippedRect<T: Transform>(transform: &T, src_rect: Rect) -> Rect {
        Self::MapEnclosingClippedRectIgnoringError(transform, src_rect, 0.0)
    }
    // Source: cc/base/math_util.h:174-177
    // Source: cc/base/math_util.cc:281-290
    pub fn MapEnclosingClippedRectIgnoringError<T: Transform>(
        transform: &T,
        src_rect: Rect,
        ignore_error: f32,
    ) -> Rect {
        if transform.IsIdentityOrIntegerTranslation() {
            return src_rect + ToFlooredVector2d(transform.To2dTranslation());
        }
        let mapped_rect = Self::MapClippedRect(transform, RectF::from(src_rect));
        ToEnclosingRectIgnoringError(mapped_rect, ignore_error)
    }
    // Source: cc/base/math_util.h:178-179
    // Source: cc/base/math_util.cc:292-307
    pub fn MapClippedRect<T: Transform>(transform: &T, src_rect: RectF) -> RectF {
        if transform.IsIdentityOrTranslation() {
            return src_rect + transform.To2dTranslation();
        }
        let hc0 = MapHomogeneousPoint(transform, src_rect.origin());
        let hc1 = MapHomogeneousPoint(transform, src_rect.top_right());
        let hc2 = MapHomogeneousPoint(transform, src_rect.bottom_right());
        let hc3 = MapHomogeneousPoint(transform, src_rect.bottom_left());
        Self::ComputeEnclosingClippedRect(hc0, hc1, hc2, hc3)
    }
    // Source: cc/base/math_util.h:180-181
    // Source: cc/base/math_util.cc:309-323
    pub fn ProjectEnclosingClippedRect<T: Transform>(transform: &T, src_rect: Rect) -> Rect {
        if transform.IsIdentityOrIntegerTranslation() {
            return src_rect + ToFlooredVector2d(transform.To2dTranslation());
        }
        let projected_rect = Self::ProjectClippedRect(transform, RectF::from(src_rect));
        if projected_rect.x().is_nan()
            || projected_rect.y().is_nan()
            || projected_rect.right().is_nan()
            || projected_rect.bottom().is_nan()
        {
            return Rect::default();
        }
        ToEnclosingRect(projected_rect)
    }
    // Source: cc/base/math_util.h:182-185
    // Source: cc/base/math_util.cc:325-343
    pub fn ProjectEnclosingClippedRectIgnoringError<T: Transform>(
        transform: &T,
        src_rect: Rect,
        ignore_error: f32,
    ) -> Rect {
        if transform.IsIdentityOrIntegerTranslation() {
            return src_rect + ToFlooredVector2d(transform.To2dTranslation());
        }
        let projected_rect = Self::ProjectClippedRect(transform, RectF::from(src_rect));
        if projected_rect.x().is_nan()
            || projected_rect.y().is_nan()
            || projected_rect.right().is_nan()
            || projected_rect.bottom().is_nan()
        {
            return Rect::default();
        }
        ToEnclosingRectIgnoringError(projected_rect, ignore_error)
    }
    // Source: cc/base/math_util.h:186-187
    // Source: cc/base/math_util.cc:345-358
    pub fn ProjectClippedRect<T: Transform>(transform: &T, src_rect: RectF) -> RectF {
        if transform.IsIdentityOrTranslation() {
            return src_rect + transform.To2dTranslation();
        }
        let q = QuadF::FromRect(src_rect);
        let h1 = ProjectHomogeneousPoint(transform, q.points[0]);
        let h2 = ProjectHomogeneousPoint(transform, q.points[1]);
        let h3 = ProjectHomogeneousPoint(transform, q.points[2]);
        let h4 = ProjectHomogeneousPoint(transform, q.points[3]);
        Self::ComputeEnclosingClippedRect(h1, h2, h3, h4)
    }
    // Source: cc/base/math_util.h:192-194
    // Source: cc/base/math_util.cc:360-373
    pub fn InverseMapQuadToLocalSpace<T: Transform>(
        device_transform: &T,
        device_quad: QuadF,
    ) -> QuadF {
        debug_assert!(device_transform.IsFlat());
        let inverse_device_transform = device_transform.GetCheckedInverse();
        let mut clipped = false;
        let local_quad = Self::MapQuad(&inverse_device_transform, device_quad, &mut clipped);
        local_quad
    }
    // Source: cc/base/math_util.h:198-200
    // Source: cc/base/math_util.cc:375-399
    pub fn MapEnclosedRectWith2dAxisAlignedTransform<T: Transform>(
        transform: &T,
        rect: Rect,
    ) -> Rect {
        debug_assert!(transform.Preserves2dAxisAlignment());
        debug_assert!(transform.rc(3, 3) > 0.0);
        debug_assert!(transform.rc(3, 3).is_normal());
        if transform.IsIdentityOrIntegerTranslation() {
            return rect + ToFlooredVector2d(transform.To2dTranslation());
        }
        if transform.IsIdentityOrTranslation() {
            let offset = transform.To2dTranslation();
            return ToEnclosedRect(RectF::from(rect) + offset);
        }
        let hc0 = MapHomogeneousPoint(
            transform,
            PointF::new(rect.origin().x() as f32, rect.origin().y() as f32),
        );
        let hc1 = MapHomogeneousPoint(
            transform,
            PointF::new(
                rect.bottom_right().x() as f32,
                rect.bottom_right().y() as f32,
            ),
        );
        debug_assert!(!hc0.ShouldBeClipped());
        debug_assert!(!hc1.ShouldBeClipped());
        let top_left = hc0.CartesianPoint2d();
        let bottom_right = hc1.CartesianPoint2d();
        ToEnclosedRect(BoundingRect(top_left, bottom_right))
    }

    // Source: cc/base/math_util.h:207-210
    // Source: cc/base/math_util.cc:401-620
    pub fn MapClippedQuad3d<T: Transform>(
        transform: &T,
        src_quad: QuadF,
        clipped_quad: &mut [Point3F; 6],
        num_vertices_in_clipped_quad: &mut i32,
    ) -> bool {
        // Source: cc/base/math_util.cc:409-416
        let h1 = MapHomogeneousPoint(transform, src_quad.points[0]);
        let h2 = MapHomogeneousPoint(transform, src_quad.points[1]);
        let h3 = MapHomogeneousPoint(transform, src_quad.points[2]);
        let h4 = MapHomogeneousPoint(transform, src_quad.points[3]);
        *num_vertices_in_clipped_quad = 0;
        let mut need_to_clamp = false;

        // Source: cc/base/math_util.cc:419-465
        if !h1.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                h1.CartesianPoint3dUnclamped(),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }
        if h1.ShouldBeClipped() ^ h2.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                ComputeClippedCartesianPoint3dForEdge(h1, h2),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }
        if !h2.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                h2.CartesianPoint3dUnclamped(),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }
        if h2.ShouldBeClipped() ^ h3.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                ComputeClippedCartesianPoint3dForEdge(h2, h3),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }
        if !h3.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                h3.CartesianPoint3dUnclamped(),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }
        if h3.ShouldBeClipped() ^ h4.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                ComputeClippedCartesianPoint3dForEdge(h3, h4),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }
        if !h4.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                h4.CartesianPoint3dUnclamped(),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }
        if h4.ShouldBeClipped() ^ h1.ShouldBeClipped() {
            AddVertexToClippedQuad3d(
                ComputeClippedCartesianPoint3dForEdge(h4, h1),
                clipped_quad,
                num_vertices_in_clipped_quad,
                &mut need_to_clamp,
            );
        }

        // Source: cc/base/math_util.cc:467-473
        if *num_vertices_in_clipped_quad > 2
            && IsNearlyTheSamePoint3(
                clipped_quad[0],
                clipped_quad[(*num_vertices_in_clipped_quad - 1) as usize],
            )
        {
            *num_vertices_in_clipped_quad -= 1;
        }

        // Source: cc/base/math_util.cc:475-497
        if need_to_clamp {
            let mut normal = Vector3dF::new(0.0, 0.0, 0.0);
            if *num_vertices_in_clipped_quad > 2 {
                let loop_vector =
                    clipped_quad[0] - clipped_quad[(*num_vertices_in_clipped_quad - 1) as usize];
                let mut prev_vector = loop_vector;
                for i in 1..*num_vertices_in_clipped_quad as usize {
                    let cur_vector = clipped_quad[i] - clipped_quad[i - 1];
                    normal += CrossProduct(prev_vector, cur_vector);
                    prev_vector = cur_vector;
                }
                normal += CrossProduct(prev_vector, loop_vector);
            }
            let mut clamp_by_points = false;
            let length = normal.Length();

            // Source: cc/base/math_util.cc:498-531
            if length.is_normal() {
                normal.InvScale(length);
                let mut shortest_from_zero = normal;
                shortest_from_zero.Scale(DotProduct3(
                    normal,
                    clipped_quad[0] - Point3F::new(0.0, 0.0, 0.0),
                ));
                let mut z_at_xy_zero = 0.0;
                if shortest_from_zero.x() == 0.0 && shortest_from_zero.y() == 0.0 {
                    z_at_xy_zero = shortest_from_zero.z();
                } else if shortest_from_zero.z() != 0.0 {
                    let v =
                        CrossProduct(normal, CrossProduct(Vector3dF::new(0.0, 0.0, 1.0), normal));
                    debug_assert!((normal.x() * v.y() - normal.y() * v.x()).abs() < 0.00001);
                    let r = if v.x().abs() > v.y().abs() {
                        shortest_from_zero.x() / v.x()
                    } else {
                        shortest_from_zero.y() / v.y()
                    };
                    z_at_xy_zero = shortest_from_zero.z() - v.z() * r;
                } else {
                    clamp_by_points = true;
                }

                // Source: cc/base/math_util.cc:533-556
                if !clamp_by_points {
                    const MAX_DISTANCE: f32 = 0.75 * HomogeneousCoordinate::kInfiniteCoordinate;
                    if z_at_xy_zero.abs() > MAX_DISTANCE {
                        let z_delta;
                        if z_at_xy_zero > 0.0 {
                            z_delta = MAX_DISTANCE - z_at_xy_zero;
                        } else {
                            z_delta = -MAX_DISTANCE - z_at_xy_zero;
                        }
                        for i in 0..*num_vertices_in_clipped_quad as usize {
                            clipped_quad[i].set_z(clipped_quad[i].z() + z_delta);
                        }
                        z_at_xy_zero += z_delta;
                    }

                    // Source: cc/base/math_util.cc:558-593
                    for i in 0..*num_vertices_in_clipped_quad as usize {
                        let point = &mut clipped_quad[i];
                        let mut t = 1.0;
                        let x_abs = point.x().abs();
                        if x_abs > HomogeneousCoordinate::kInfiniteCoordinate {
                            t = cpp_min(t, HomogeneousCoordinate::kInfiniteCoordinate / x_abs);
                        }
                        let y_abs = point.y().abs();
                        if y_abs > HomogeneousCoordinate::kInfiniteCoordinate {
                            t = cpp_min(t, HomogeneousCoordinate::kInfiniteCoordinate / y_abs);
                        }
                        let z = point.z();
                        if z.abs() > HomogeneousCoordinate::kInfiniteCoordinate {
                            let z_at_xy_zero_clamped = cpp_min(
                                HomogeneousCoordinate::kInfiniteCoordinate,
                                cpp_max(-HomogeneousCoordinate::kInfiniteCoordinate, z_at_xy_zero),
                            );
                            let z_offset = z - z_at_xy_zero_clamped;
                            let z_space = (if z > 0.0 {
                                HomogeneousCoordinate::kInfiniteCoordinate
                            } else {
                                -HomogeneousCoordinate::kInfiniteCoordinate
                            }) - z_at_xy_zero_clamped;
                            debug_assert_ne!(z_offset, 0.0);
                            debug_assert_ne!(z_space, 0.0);
                            debug_assert_eq!(z_offset > 0.0, z_space > 0.0);
                            t = cpp_min(t, z_space / z_offset);
                        }
                        if t != 1.0 {
                            debug_assert!(0.0 <= t && t < 1.0);
                            point.set_x(t * point.x());
                            point.set_y(t * point.y());
                            point.set_z((1.0 - t) * z_at_xy_zero + t * point.z());
                        }
                    }
                }
            } else {
                clamp_by_points = true;
            }

            // Source: cc/base/math_util.cc:599-615
            if clamp_by_points {
                for i in 0..*num_vertices_in_clipped_quad as usize {
                    let point = &mut clipped_quad[i];
                    point.set_x(cpp_clamp(
                        point.x(),
                        -HomogeneousCoordinate::kInfiniteCoordinate,
                        HomogeneousCoordinate::kInfiniteCoordinate,
                    ));
                    point.set_y(cpp_clamp(
                        point.y(),
                        -HomogeneousCoordinate::kInfiniteCoordinate,
                        HomogeneousCoordinate::kInfiniteCoordinate,
                    ));
                    point.set_z(cpp_clamp(
                        point.z(),
                        -HomogeneousCoordinate::kInfiniteCoordinate,
                        HomogeneousCoordinate::kInfiniteCoordinate,
                    ));
                }
            }
        }
        // Source: cc/base/math_util.cc:618-620
        debug_assert!(*num_vertices_in_clipped_quad <= 6);
        *num_vertices_in_clipped_quad >= 4
    }
}
impl MathUtil {
    // Source: cc/base/math_util.h:212-213
    // Source: cc/base/math_util.cc:622-639
    pub fn ComputeEnclosingRectOfVertices(vertices: &[PointF]) -> RectF {
        if vertices.len() < 2 {
            return RectF::default();
        }
        let mut xmin = f32::MAX;
        let mut xmax = -f32::MAX;
        let mut ymin = f32::MAX;
        let mut ymax = -f32::MAX;
        for vertex in vertices {
            ExpandBoundsToIncludePoint(&mut xmin, &mut xmax, &mut ymin, &mut ymax, *vertex);
        }
        RectF::FromOriginSize(
            PointF::new(xmin, ymin),
            SizeF::new(xmax - xmin, ymax - ymin),
        )
    }
    // Source: cc/base/math_util.h:214-218
    // Source: cc/base/math_util.cc:641-706
    pub fn ComputeEnclosingClippedRect(
        h1: HomogeneousCoordinate,
        h2: HomogeneousCoordinate,
        h3: HomogeneousCoordinate,
        h4: HomogeneousCoordinate,
    ) -> RectF {
        let something_clipped = h1.ShouldBeClipped()
            || h2.ShouldBeClipped()
            || h3.ShouldBeClipped()
            || h4.ShouldBeClipped();
        if !something_clipped {
            let mapped_quad = QuadF::new(
                h1.CartesianPoint2d(),
                h2.CartesianPoint2d(),
                h3.CartesianPoint2d(),
                h4.CartesianPoint2d(),
            );
            return mapped_quad.BoundingBox();
        }
        let everything_clipped = h1.ShouldBeClipped()
            && h2.ShouldBeClipped()
            && h3.ShouldBeClipped()
            && h4.ShouldBeClipped();
        if everything_clipped {
            return RectF::default();
        }
        let mut xmin = f32::MAX;
        let mut xmax = -f32::MAX;
        let mut ymin = f32::MAX;
        let mut ymax = -f32::MAX;
        if !h1.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                h1.CartesianPoint2d(),
            );
        }
        if h1.ShouldBeClipped() ^ h2.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                ComputeClippedCartesianPoint2dForEdge(h1, h2),
            );
        }
        if !h2.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                h2.CartesianPoint2d(),
            );
        }
        if h2.ShouldBeClipped() ^ h3.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                ComputeClippedCartesianPoint2dForEdge(h2, h3),
            );
        }
        if !h3.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                h3.CartesianPoint2d(),
            );
        }
        if h3.ShouldBeClipped() ^ h4.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                ComputeClippedCartesianPoint2dForEdge(h3, h4),
            );
        }
        if !h4.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                h4.CartesianPoint2d(),
            );
        }
        if h4.ShouldBeClipped() ^ h1.ShouldBeClipped() {
            ExpandBoundsToIncludePoint(
                &mut xmin,
                &mut xmax,
                &mut ymin,
                &mut ymax,
                ComputeClippedCartesianPoint2dForEdge(h4, h1),
            );
        }
        RectF::FromOriginSize(
            PointF::new(xmin, ymin),
            SizeF::new(xmax - xmin, ymax - ymin),
        )
    }
    // Source: cc/base/math_util.h:222-224
    // Source: cc/base/math_util.cc:708-732
    pub fn MapQuad<T: Transform>(transform: &T, q: QuadF, clipped: &mut bool) -> QuadF {
        if transform.IsIdentityOrTranslation() {
            let mut mapped_quad = q;
            mapped_quad += transform.To2dTranslation();
            *clipped = false;
            return mapped_quad;
        }
        let h1 = MapHomogeneousPoint(transform, q.points[0]);
        let h2 = MapHomogeneousPoint(transform, q.points[1]);
        let h3 = MapHomogeneousPoint(transform, q.points[2]);
        let h4 = MapHomogeneousPoint(transform, q.points[3]);
        *clipped = h1.ShouldBeClipped()
            || h2.ShouldBeClipped()
            || h3.ShouldBeClipped()
            || h4.ShouldBeClipped();
        QuadF::new(
            h1.CartesianPoint2d(),
            h2.CartesianPoint2d(),
            h3.CartesianPoint2d(),
            h4.CartesianPoint2d(),
        )
    }
    // Source: cc/base/math_util.h:225-227
    // Source: cc/base/math_util.cc:734-756
    pub fn MapPoint<T: Transform>(transform: &T, p: PointF, clipped: &mut bool) -> PointF {
        let h = MapHomogeneousPoint(transform, p);
        if h.w() > 0.0 {
            *clipped = false;
            return h.CartesianPoint2d();
        }
        *clipped = true;
        if h.w() == 0.0 {
            return PointF::default();
        }
        h.CartesianPoint2d()
    }
    // Source: cc/base/math_util.h:228-230
    // Source: cc/base/math_util.cc:758-771
    pub fn ProjectPoint<T: Transform>(transform: &T, p: PointF, clipped: &mut bool) -> PointF {
        let h = ProjectHomogeneousPointWithClipped(transform, p, clipped);
        if h.w() == 0.0 {
            return PointF::default();
        }
        h.CartesianPoint2d()
    }
    // Source: cc/base/math_util.h:237-239
    // Source: cc/base/math_util.cc:773-792
    pub fn ScaleRectProportional(
        input_outer_rect: RectF,
        scale_outer_rect: RectF,
        scale_inner_rect: RectF,
    ) -> RectF {
        let mut output_inner_rect = input_outer_rect;
        let scale_rect_to_input_scale_x = scale_outer_rect.width() / input_outer_rect.width();
        let scale_rect_to_input_scale_y = scale_outer_rect.height() / input_outer_rect.height();
        let top_left_diff = scale_inner_rect.origin() - scale_outer_rect.origin();
        let bottom_right_diff = scale_inner_rect.bottom_right() - scale_outer_rect.bottom_right();
        output_inner_rect.Inset(
            top_left_diff.y() / scale_rect_to_input_scale_y,
            top_left_diff.x() / scale_rect_to_input_scale_x,
            -bottom_right_diff.y() / scale_rect_to_input_scale_y,
            -bottom_right_diff.x() / scale_rect_to_input_scale_x,
        );
        output_inner_rect
    }
    // Source: cc/base/math_util.h:243-244
    // Source: cc/base/math_util.cc:794-800
    pub fn SmallestAngleBetweenVectors(v1: Vector2dF, v2: Vector2dF) -> f32 {
        let mut dot_product = DotProduct2(v1, v2) / v1.Length() as f64 / v2.Length() as f64;
        dot_product = cpp_clamp(dot_product, -1.0, 1.0);
        (dot_product.acos() * 180.0 / std::f64::consts::PI) as f32
    }
    // Source: cc/base/math_util.h:248-249
    // Source: cc/base/math_util.cc:802-808
    pub fn ProjectVector(source: Vector2dF, destination: Vector2dF) -> Vector2dF {
        let projected_length =
            (DotProduct2(source, destination) / destination.LengthSquared()) as f32;
        Vector2dF::new(
            projected_length * destination.x(),
            projected_length * destination.y(),
        )
    }
    // Source: cc/base/math_util.h:253-253
    // Source: cc/base/math_util.cc:810-818
    pub fn ScalePointByInverse(point: PointF, scale: f32) -> PointF {
        debug_assert!(scale.abs() > f32::EPSILON);
        let inv_scale_magnitude = 1.0 / cpp_max(f32::EPSILON, scale.abs());
        let mut result = point;
        result.Scale(inv_scale_magnitude.copysign(scale));
        result
    }
    // Source: cc/base/math_util.h:257-258
    // Source: cc/base/math_util.cc:820-826
    pub fn ScaleVectorByInverse(vector: Vector2dF, scale: f32) -> Vector2dF {
        debug_assert!(scale.abs() > f32::EPSILON);
        let inv_scale_magnitude = 1.0 / cpp_max(f32::EPSILON, scale.abs());
        let scale = inv_scale_magnitude.copysign(scale);
        Vector2dF::new(vector.x() * scale, vector.y() * scale)
    }
    // Source: cc/base/math_util.h:260-260
    // Source: cc/base/math_util.cc:828-850
    pub fn FromValue(raw_value: &Value, out_rect: &mut Rect) -> bool {
        let list = match raw_value {
            Value::List(list) => list,
            _ => return false,
        };
        if list.len() != 4 {
            return false;
        }
        for val in list {
            if !matches!(val, Value::Int(_)) {
                return false;
            }
        }
        let integer = |index: usize| match list[index] {
            Value::Int(value) => value,
            _ => unreachable!(),
        };
        let x = integer(0);
        let y = integer(1);
        let w = integer(2);
        let h = integer(3);
        *out_rect = Rect::new(x, y, w, h);
        true
    }
}

// The trait implements Rust's equivalent of the C++ AddToTracedValue overload
// set. Every implementation retains its own source region and statement order.
pub trait MathTraceGeometry {
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue);
}
impl MathTraceGeometry for Size {
    // Source: cc/base/math_util.cc:852-859
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginDictionary(name);
        res.SetDouble("width", self.width() as f64);
        res.SetDouble("height", self.height() as f64);
        res.EndDictionary();
    }
}
impl MathTraceGeometry for SizeF {
    // Source: cc/base/math_util.cc:861-868
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginDictionary(name);
        res.SetDouble("width", self.width() as f64);
        res.SetDouble("height", self.height() as f64);
        res.EndDictionary();
    }
}
impl MathTraceGeometry for Rect {
    // Source: cc/base/math_util.cc:870-879
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendInteger(self.x);
        res.AppendInteger(self.y);
        res.AppendInteger(self.width);
        res.AppendInteger(self.height);
        res.EndArray();
    }
}
impl MathTraceGeometry for Point {
    // Source: cc/base/math_util.cc:881-888
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendInteger(self.x());
        res.AppendInteger(self.y());
        res.EndArray();
    }
}
impl MathTraceGeometry for PointF {
    // Source: cc/base/math_util.cc:890-897
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendDouble(self.x() as f64);
        res.AppendDouble(self.y() as f64);
        res.EndArray();
    }
}
impl MathTraceGeometry for Point3F {
    // Source: cc/base/math_util.cc:899-907
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendDouble(self.x() as f64);
        res.AppendDouble(self.y() as f64);
        res.AppendDouble(self.z() as f64);
        res.EndArray();
    }
}
impl MathTraceGeometry for Vector2d {
    // Source: cc/base/math_util.cc:909-916
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendInteger(self.x);
        res.AppendInteger(self.y);
        res.EndArray();
    }
}
impl MathTraceGeometry for Vector2dF {
    // Source: cc/base/math_util.cc:918-925
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendDouble(self.x() as f64);
        res.AppendDouble(self.y() as f64);
        res.EndArray();
    }
}
impl MathTraceGeometry for QuadF {
    // Source: cc/base/math_util.cc:927-940
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendDouble(self.points[0].x() as f64);
        res.AppendDouble(self.points[0].y() as f64);
        res.AppendDouble(self.points[1].x() as f64);
        res.AppendDouble(self.points[1].y() as f64);
        res.AppendDouble(self.points[2].x() as f64);
        res.AppendDouble(self.points[2].y() as f64);
        res.AppendDouble(self.points[3].x() as f64);
        res.AppendDouble(self.points[3].y() as f64);
        res.EndArray();
    }
}
impl MathTraceGeometry for RectF {
    // Source: cc/base/math_util.cc:942-951
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendDouble(self.x() as f64);
        res.AppendDouble(self.y() as f64);
        res.AppendDouble(self.width() as f64);
        res.AppendDouble(self.height() as f64);
        res.EndArray();
    }
}
// Rust overload wrapper: Transform is a dependency trait, so a wrapper avoids
// overlapping the blanket Transform implementation with concrete gfx values.
pub struct TransformTrace<'a, T: Transform>(pub &'a T);
impl<T: Transform> MathTraceGeometry for TransformTrace<'_, T> {
    // Source: cc/base/math_util.cc:953-962
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        for row in 0..4 {
            for col in 0..4 {
                res.AppendDouble(self.0.rc(row, col));
            }
        }
        res.EndArray();
    }
}
impl MathTraceGeometry for BoxF {
    // Source: cc/base/math_util.cc:964-975
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendInteger(self.x as i32);
        res.AppendInteger(self.y as i32);
        res.AppendInteger(self.z as i32);
        res.AppendInteger(self.width as i32);
        res.AppendInteger(self.height as i32);
        res.AppendInteger(self.depth as i32);
        res.EndArray();
    }
}
impl MathTraceGeometry for RRectF {
    // Source: cc/base/math_util.cc:977-994
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendDouble(self.rect.x() as f64);
        res.AppendDouble(self.rect.y() as f64);
        res.AppendDouble(self.rect.width() as f64);
        res.AppendDouble(self.rect.height() as f64);
        res.AppendDouble(self.corner_radii[0].x() as f64);
        res.AppendDouble(self.corner_radii[0].y() as f64);
        res.AppendDouble(self.corner_radii[1].x() as f64);
        res.AppendDouble(self.corner_radii[1].y() as f64);
        res.AppendDouble(self.corner_radii[2].x() as f64);
        res.AppendDouble(self.corner_radii[2].y() as f64);
        res.AppendDouble(self.corner_radii[3].x() as f64);
        res.AppendDouble(self.corner_radii[3].y() as f64);
        res.EndArray();
    }
}
pub struct PathTrace<'a, T: Path>(pub &'a T);
impl<T: Path> MathTraceGeometry for PathTrace<'_, T> {
    // Source: cc/base/math_util.cc:996-1009
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        if let Some(rrect) = self.0.AsRRect() {
            MathUtil::AddToTracedValue(name, &rrect, res);
        } else {
            res.BeginDictionary(name);
            MathUtil::AddToTracedValue("bounds", &self.0.bounds(), res);
            res.SetInteger("num_points", self.0.count_points());
            res.SetInteger("num_verbs", self.0.count_verbs());
            res.EndDictionary();
        }
    }
}
impl MathUtil {
    // Source: cc/base/math_util.h:262-303
    pub fn AddToTracedValue<T: MathTraceGeometry>(
        name: &str,
        value: &T,
        res: &mut impl TracedValue,
    ) {
        value.AddToTracedValue(name, res);
    }
    // Source: cc/base/math_util.h:304-306
    // Source: cc/base/math_util.cc:1011-1025
    pub fn AddCornerRadiiToTracedValue(name: &str, rect: &RRectF, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendDouble(rect.corner_radii[0].x() as f64);
        res.AppendDouble(rect.corner_radii[0].y() as f64);
        res.AppendDouble(rect.corner_radii[1].x() as f64);
        res.AppendDouble(rect.corner_radii[1].y() as f64);
        res.AppendDouble(rect.corner_radii[2].x() as f64);
        res.AppendDouble(rect.corner_radii[2].y() as f64);
        res.AppendDouble(rect.corner_radii[3].x() as f64);
        res.AppendDouble(rect.corner_radii[3].y() as f64);
        res.EndArray();
    }
}
impl MathTraceGeometry for LinearGradient {
    // Source: cc/base/math_util.cc:1027-1038
    fn AddToTracedValue(&self, name: &str, res: &mut impl TracedValue) {
        res.BeginArray(name);
        res.AppendInteger(self.angle as i32);
        res.AppendInteger(self.steps.len() as i32);
        for i in 0..self.steps.len() {
            res.AppendDouble(self.steps[i].fraction as f64);
            res.AppendInteger(self.steps[i].alpha as i32);
        }
        res.EndArray();
    }
}
impl MathUtil {
    // Source: cc/base/math_util.h:313-313
    // Source: cc/base/math_util.cc:1040-1042
    pub fn AsDoubleSafely(value: f64) -> f64 {
        cpp_min(value, f64::MAX)
    }
    // Source: cc/base/math_util.cc:1044-1046
    pub fn AsFloatSafely(value: f32) -> f32 {
        cpp_min(value, f32::MAX)
    }
    // Source: cc/base/math_util.h:317-317
    // Source: cc/base/math_util.cc:1048-1055
    pub fn GetXAxis<T: Transform>(transform: &T) -> Vector3dF {
        if transform.IsScaleOrTranslation() {
            return Vector3dF::new(transform.To2dScale().x(), 0.0, 0.0);
        }
        Vector3dF::new(
            transform.rc(0, 0) as f32,
            transform.rc(1, 0) as f32,
            transform.rc(2, 0) as f32,
        )
    }
    // Source: cc/base/math_util.h:320-320
    // Source: cc/base/math_util.cc:1057-1063
    pub fn GetYAxis<T: Transform>(transform: &T) -> Vector3dF {
        if transform.IsScaleOrTranslation() {
            return Vector3dF::new(0.0, transform.To2dScale().y(), 0.0);
        }
        Vector3dF::new(
            transform.rc(0, 1) as f32,
            transform.rc(1, 1) as f32,
            transform.rc(2, 1) as f32,
        )
    }
}

// Source: cc/base/math_util.h:354-366.
// Reading and writing the CPU control register is an embedding operation. The
// translated type owns the source RAII lifetime and restores through the same
// live capability on drop.
pub struct ScopedSubnormalFloatDisabler<
    'a,
    C: crate::adapter::subnormal_float::SubnormalFloatControl,
> {
    control_: &'a mut C,
    orig_state_: Option<C::State>,
    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl<'a, C: crate::adapter::subnormal_float::SubnormalFloatControl>
    ScopedSubnormalFloatDisabler<'a, C>
{
    // Source: cc/base/math_util.cc:1065-1071.
    pub fn new(control: &'a mut C) -> Self {
        let orig_state_ = Some(control.DisableSubnormalFloats());
        Self {
            control_: control,
            orig_state_,
            _same_thread: std::marker::PhantomData,
        }
    }
}
impl<'a, C: crate::adapter::subnormal_float::SubnormalFloatControl> Drop
    for ScopedSubnormalFloatDisabler<'a, C>
{
    // Source: cc/base/math_util.cc:1073-1077.
    fn drop(&mut self) {
        if let Some(state) = self.orig_state_.take() {
            self.control_.RestoreSubnormalFloatState(state);
        }
    }
}
impl MathUtil {
    // Source: cc/base/math_util.h:322-322
    // Source: cc/base/math_util.cc:1079-1081
    pub fn IsFloatNearlyTheSame(left: f32, right: f32) -> bool {
        IsNearlyTheSame(left, right)
    }
    // Source: cc/base/math_util.h:323-324
    // Source: cc/base/math_util.cc:1083-1086
    pub fn IsNearlyTheSameForTesting(left: PointF, right: PointF) -> bool {
        IsNearlyTheSamePoint2(left, right)
    }
    // Source: cc/base/math_util.h:325-326
    // Source: cc/base/math_util.cc:1088-1091
    pub fn IsNearlyTheSamePoint3ForTesting(left: Point3F, right: Point3F) -> bool {
        IsNearlyTheSamePoint3(left, right)
    }
    // Source: cc/base/math_util.h:330-330
    // Source: cc/base/math_util.cc:1093-1098
    pub fn SkM44HasPerspective(m: &impl Matrix44) -> bool {
        m.rc(3, 0) != 0.0 || m.rc(3, 1) != 0.0 || m.rc(3, 2) != 0.0 || m.rc(3, 3) != 1.0
    }
    // Source: cc/base/math_util.h:331-331
    // Source: cc/base/math_util.cc:1100-1105
    pub fn SkM44Is2D(m: &impl Matrix44) -> bool {
        m.rc(0, 2) == 0.0
            && m.rc(1, 2) == 0.0
            && m.rc(2, 2) == 1.0
            && m.rc(2, 0) == 0.0
            && m.rc(2, 1) == 0.0
            && m.rc(2, 3) == 0.0
            && m.rc(3, 2) == 0.0
    }
    // Source: cc/base/math_util.h:332-332
    // Source: cc/base/math_util.cc:1107-1126
    pub fn SkM44Preserves2DAxisAlignment(m: &impl Matrix44) -> bool {
        if !Self::SkM44Is2D(m) || Self::SkM44HasPerspective(m) {
            return false;
        }
        if m.rc(0, 1) != 0.0 || m.rc(1, 0) != 0.0 {
            return m.rc(0, 0) == 0.0
                && m.rc(1, 1) == 0.0
                && m.rc(0, 1) != 0.0
                && m.rc(1, 0) != 0.0;
        }
        m.rc(0, 0) != 0.0 && m.rc(1, 1) != 0.0
    }
}

use compositor_core::adapter::geometry::{
    CrossProduct, DotProduct3, Point3F, PointF, QuadF, RectF, Vector3dF,
};
use compositor_core::cc::base::math_util::MathUtil;
use viz::common::quads::aggregated_render_pass::NativeTransform;

// Source: components/viz/service/display/draw_polygon.cc:20-34
const SPLIT_THRESHOLD: f32 = 0.05;
const NORMALIZED_THRESHOLD: f32 = 0.001;
fn point_interpolate(from: Point3F, to: Point3F, delta: f64) -> Point3F {
    Point3F::new(
        (from.x as f64 + (to.x - from.x) as f64 * delta) as f32,
        (from.y as f64 + (to.y - from.y) as f64 * delta) as f32,
        (from.z as f64 + (to.z - from.z) as f64 * delta) as f32,
    )
}
fn map_vector(transform: &NativeTransform, vector: Vector3dF) -> Vector3dF {
    let mut v = [vector.x, vector.y, vector.z, 0.0];
    transform.native().TransformVector4(&mut v);
    Vector3dF::new(v[0], v[1], v[2])
}
// Source: components/viz/service/display/draw_polygon.h:22-76
// The original_ref raw pointer becomes a pass-local quad index. The caller keeps
// the immutable AggregatedRenderPass alive; splitting copies only this handle.
#[derive(Debug, Default)]
pub struct DrawPolygon {
    pub points: Vec<Point3F>,
    pub normal: Vector3dF,
    pub order_index: i32,
    pub original_ref: usize,
    pub is_split: bool,
}
impl DrawPolygon {
    // Source: components/viz/service/display/draw_polygon.cc:39-56,87-87
    pub fn from_points(
        original_ref: usize,
        points: &[Point3F],
        normal: Vector3dF,
        order_index: i32,
    ) -> Self {
        Self {
            original_ref,
            points: points.to_vec(),
            normal,
            order_index,
            is_split: true,
        }
    }
    // Source: components/viz/service/display/draw_polygon.cc:58-85
    pub fn new(
        original_ref: usize,
        rect: RectF,
        transform: &NativeTransform,
        order_index: i32,
    ) -> Self {
        let mut points = [Point3F::default(); 6];
        let mut count = 0;
        MathUtil::MapClippedQuad3d(transform, QuadF::FromRect(rect), &mut points, &mut count);
        let mut polygon = Self {
            original_ref,
            points: points[..count as usize].to_vec(),
            normal: map_vector(transform, Vector3dF::new(0.0, 0.0, 1.0)),
            order_index,
            is_split: false,
        };
        polygon.construct_normal();
        polygon
    }
    // Source: components/viz/service/display/draw_polygon.cc:89-99
    pub fn create_copy(&self) -> Self {
        Self {
            points: self.points.clone(),
            normal: self.normal,
            order_index: self.order_index,
            original_ref: self.original_ref,
            ..Self::default()
        }
    }
    // Source: components/viz/service/display/draw_polygon.cc:101-125
    fn construct_normal(&mut self) {
        let mut normal = Vector3dF::default();
        let delta = self.points.len() / 2;
        for i in 1..self.points.len().saturating_sub(delta) {
            normal += CrossProduct(
                self.points[i] - self.points[0],
                self.points[i + delta] - self.points[0],
            );
        }
        let mut magnitude = normal.Length();
        if DotProduct3(self.normal, normal) < 0.0 {
            magnitude *= -1.0;
        }
        if magnitude != 0.0 && magnitude != 1.0 {
            normal.Scale(1.0 / magnitude);
        }
        self.normal = normal;
    }
    // Source: components/viz/service/display/draw_polygon.cc:127-134
    pub fn recompute_normal_for_testing(&mut self) {
        self.construct_normal();
    }
    // Source: components/viz/service/display/draw_polygon.cc:136-138
    pub fn signed_point_distance(&self, point: Point3F) -> f32 {
        DotProduct3(point - self.points[0], self.normal)
    }
    // Source: components/viz/service/display/draw_polygon.cc:140-156
    pub fn apply_transform_to_normal(&mut self, transform: &NativeTransform) {
        let inverse = transform.native().GetCheckedInverse();
        // Transpose the canonical inverse matrix, then use its normal vector
        // mapping, retaining Foundation's float-geometry clamping.
        let m = inverse.GetColMajor();
        let transposed = std::array::from_fn(|i| m[(i % 4) * 4 + i / 4]);
        self.normal = map_vector(&NativeTransform::FromColMajor(&transposed), self.normal);
        let magnitude = self.normal.Length();
        if magnitude != 0.0 && magnitude != 1.0 {
            self.normal.InvScale(magnitude);
        }
    }
    // Source: components/viz/service/display/draw_polygon.cc:158-161
    fn apply_transform(&mut self, transform: &NativeTransform) {
        for p in &mut self.points {
            let q = transform.native().MapPoint3d([p.x, p.y, p.z]);
            *p = Point3F::new(q[0], q[1], q[2]);
        }
    }
    // Source: components/viz/service/display/draw_polygon.cc:163-170
    pub fn transform_to_screen_space(&mut self, transform: &NativeTransform) {
        self.apply_transform(transform);
        self.normal = map_vector(transform, self.normal);
        self.construct_normal();
    }
    // Source: components/viz/service/display/draw_polygon.cc:172-180
    pub fn transform_to_layer_space(&mut self, transform: &NativeTransform) {
        self.apply_transform(transform);
        self.normal = Vector3dF::new(0.0, 0.0, -1.0);
    }
    // Source: components/viz/service/display/draw_polygon.cc:182-235
    pub fn split_polygon(&self, polygon: Self) -> (Option<Self>, Option<Self>, bool) {
        let normal_length_squared = self.normal.x as f64 * self.normal.x as f64
            + self.normal.y as f64 * self.normal.y as f64
            + self.normal.z as f64 * self.normal.z as f64;
        debug_assert!((normal_length_squared - 1.0).abs() <= NORMALIZED_THRESHOLD as f64);
        let n = polygon.points.len();
        let next = |i| (i + 1) % n;
        let prev = |i| (i + n - 1) % n;
        let mut distances = Vec::with_capacity(n);
        let mut positive = 0;
        let mut negative = 0;
        for p in &polygon.points {
            let mut d = self.signed_point_distance(*p);
            if d < -SPLIT_THRESHOLD {
                negative += 1;
            } else if d > SPLIT_THRESHOLD {
                positive += 1;
            } else {
                d = 0.0;
            }
            distances.push(d);
        }
        if positive == 0 && negative == 0 {
            return if polygon.order_index >= self.order_index {
                (Some(polygon), None, true)
            } else {
                (None, Some(polygon), true)
            };
        }
        if negative == 0 {
            return (Some(polygon), None, false);
        } else if positive == 0 {
            return (None, Some(polygon), false);
        }
        // Source: components/viz/service/display/draw_polygon.cc:237-289
        let mut front_begin = distances.iter().position(|d| *d > 0.0).unwrap();
        while distances[prev(front_begin)] > 0.0 {
            front_begin = prev(front_begin);
        }
        let pre_front = prev(front_begin);
        let mut back_begin = distances.iter().position(|d| *d < 0.0).unwrap();
        while distances[prev(back_begin)] < 0.0 {
            back_begin = prev(back_begin);
        }
        let pre_back = prev(back_begin);
        debug_assert!(distances[front_begin] > 0.0 && distances[pre_front] <= 0.0);
        debug_assert!(distances[back_begin] < 0.0 && distances[pre_back] >= 0.0);
        let front_intersection = point_interpolate(
            polygon.points[pre_front],
            polygon.points[front_begin],
            (distances[pre_front] / (distances[pre_front] - distances[front_begin])) as f64,
        );
        let back_intersection = point_interpolate(
            polygon.points[pre_back],
            polygon.points[back_begin],
            (distances[pre_back] / (distances[pre_back] - distances[back_begin])) as f64,
        );
        for p in [front_intersection, back_intersection] {
            debug_assert!(p.x.is_finite() && p.y.is_finite() && p.z.is_finite());
        }
        // Source: components/viz/service/display/draw_polygon.cc:291-320
        let build = |start, end, first, last| {
            let mut points = vec![first];
            let mut i = start;
            while i != end {
                points.push(polygon.points[i]);
                i = next(i);
            }
            if points.last() != Some(&last) {
                points.push(last);
            }
            let out = Self::from_points(
                polygon.original_ref,
                &points,
                polygon.normal,
                polygon.order_index,
            );
            debug_assert!(out.points.len() >= 3);
            out
        };
        (
            Some(build(
                front_begin,
                back_begin,
                front_intersection,
                back_intersection,
            )),
            Some(build(
                back_begin,
                front_begin,
                back_intersection,
                front_intersection,
            )),
            false,
        )
    }
    // Source: components/viz/service/display/draw_polygon.cc:322-352
    pub fn to_quads_2d(&self, quads: &mut Vec<QuadF>) {
        if self.points.len() <= 2 {
            return;
        }
        let point = |i: usize| PointF::new(self.points[i].x, self.points[i].y);
        let mut offset = 1;
        while offset < self.points.len() - 1 {
            let op1 = offset + 1;
            let mut op2 = offset + 2;
            if op2 >= self.points.len() {
                op2 = op1;
            }
            quads.push(QuadF::new(point(0), point(offset), point(op1), point(op2)));
            offset = op2;
        }
    }
    // Source: components/viz/service/display/draw_polygon.h:45-49
    pub fn points(&self) -> &[Point3F] {
        &self.points
    }
    pub fn normal(&self) -> Vector3dF {
        self.normal
    }
    pub fn original_ref(&self) -> usize {
        self.original_ref
    }
    pub fn order_index(&self) -> i32 {
        self.order_index
    }
    pub fn is_split(&self) -> bool {
        self.is_split
    }
}

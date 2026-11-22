//! Pure source SkContourMeasure over the shared immutable shape path.
//! Source: SkContourMeasure.cpp:147-331,360-463,465-501,549-635;
//! SkGeometry.h:430-490; SkGeometry.cpp:148-191,394-452,1358-1384.
#![allow(non_snake_case)]
use crate::shape_path::{PathPoint, PathVerb, ShapePath};
use foundation::{gfx::PointF, PointAndTangent};
const MAX_T: u32 = 0x3fffffff;
fn scalar(t: u32) -> f32 {
    t as f32 * (1_f32 / MAX_T as f32)
}
fn p(p: PathPoint) -> PointF {
    PointF::new(p.x as f32, p.y as f32)
}
fn add(a: PointF, b: PointF) -> PointF {
    PointF::new(a.x() + b.x(), a.y() + b.y())
}
fn sub(a: PointF, b: PointF) -> PointF {
    PointF::new(a.x() - b.x(), a.y() - b.y())
}
fn mul(a: PointF, s: f32) -> PointF {
    PointF::new(a.x() * s, a.y() * s)
}
fn interp(a: PointF, b: PointF, t: f32) -> PointF {
    add(a, mul(sub(b, a), t))
}
fn norm(a: PointF) -> f32 {
    let n = a.x() * a.x() + a.y() * a.y();
    if n.is_finite() {
        n.sqrt()
    } else {
        (a.x() as f64).hypot(a.y() as f64) as f32
    }
}
fn cheap(a: PointF) -> f32 {
    a.x().abs().max(a.y().abs())
}
fn quad(a: PointF, b: PointF, c: PointF, t: f32) -> PointF {
    add(
        mul(
            add(mul(add(sub(c, mul(b, 2.)), a), t), mul(sub(b, a), 2.)),
            t,
        ),
        a,
    )
}
#[derive(Clone, Copy)]
struct Curve {
    kind: PathVerb,
    points: [PointF; 4],
    weight: f32,
}
impl Curve {
    fn End(&self) -> PointF {
        self.points[match self.kind {
            PathVerb::kLineTo => 1,
            PathVerb::kCubicTo => 3,
            _ => 2,
        }]
    }
    fn Eval(&self, t: f32) -> PointF {
        let [a, b, c, d] = self.points;
        match self.kind {
            PathVerb::kLineTo => interp(a, b, t),
            PathVerb::kQuadraticTo => quad(a, b, c, t),
            PathVerb::kConicTo => {
                let bw = mul(b, self.weight);
                let numerator = quad(a, bw, c, t);
                let bb = 2. * (self.weight - 1.);
                let denominator = (-bb * t + bb) * t + 1.;
                PointF::new(numerator.x() / denominator, numerator.y() / denominator)
            }
            PathVerb::kCubicTo => {
                let aa = sub(add(d, mul(sub(b, c), 3.)), a);
                let bb = mul(add(sub(c, mul(b, 2.)), a), 3.);
                let cc = mul(sub(b, a), 3.);
                add(mul(add(mul(add(mul(aa, t), bb), t), cc), t), a)
            }
            _ => a,
        }
    }
    fn Tangent(&self, t: f32) -> PointF {
        let [a, b, c, d] = self.points;
        let tangent = match self.kind {
            PathVerb::kLineTo => sub(b, a),
            PathVerb::kQuadraticTo => {
                if (t == 0. && a == b) || (t == 1. && b == c) {
                    sub(c, a)
                } else {
                    let bb = sub(b, a);
                    mul(add(mul(sub(sub(c, b), bb), t), bb), 2.)
                }
            }
            PathVerb::kConicTo => {
                if (t == 0. && a == b) || (t == 1. && b == c) {
                    sub(c, a)
                } else {
                    let p20 = sub(c, a);
                    let cc = mul(sub(b, a), self.weight);
                    let aa = sub(mul(p20, self.weight), p20);
                    let bb = sub(sub(p20, cc), cc);
                    add(mul(add(mul(aa, t), bb), t), cc)
                }
            }
            PathVerb::kCubicTo => {
                if (t == 0. && a == b) || (t == 1. && c == d) {
                    let v = if t == 0. { sub(c, a) } else { sub(d, b) };
                    if v == PointF::default() {
                        sub(d, a)
                    } else {
                        v
                    }
                } else {
                    let aa = sub(add(d, mul(sub(b, c), 3.)), a);
                    let bb = mul(add(sub(c, mul(b, 2.)), a), 2.);
                    let cc = sub(b, a);
                    add(mul(add(mul(aa, t), bb), t), cc)
                }
            }
            _ => PointF::default(),
        };
        let length = (tangent.x() as f64).hypot(tangent.y() as f64);
        if length > 0. && length.is_finite() {
            PointF::new(
                (tangent.x() as f64 / length) as f32,
                (tangent.y() as f64 / length) as f32,
            )
        } else {
            PointF::default()
        }
    }
}
#[derive(Clone, Copy)]
struct Segment {
    distance: f32,
    curve: usize,
    t: u32,
}
#[derive(Default)]
struct Contour {
    curves: Vec<Curve>,
    segments: Vec<Segment>,
    length: f32,
}
impl Contour {
    fn Append(&mut self, curve: Curve) {
        let index = self.curves.len();
        self.curves.push(curve);
        self.Subdivide(index, curve, 0, MAX_T, curve.points[0], curve.End(), 0);
    }
    fn Subdivide(
        &mut self,
        index: usize,
        curve: Curve,
        min: u32,
        max: u32,
        first: PointF,
        last: PointF,
        depth: u32,
    ) {
        let half = (min + max) >> 1;
        let [a, b, c, d] = curve.points;
        let split = match curve.kind {
            PathVerb::kQuadraticTo => cheap(sub(mul(b, 0.5), mul(mul(add(a, c), 0.5), 0.5))) > 0.5,
            PathVerb::kCubicTo => {
                cheap(sub(b, interp(a, d, 1. / 3.))) > 0.5
                    || cheap(sub(c, interp(a, d, 2. / 3.))) > 0.5
            }
            PathVerb::kConicTo => {
                cheap(sub(
                    self.curves[index].Eval(scalar(half)),
                    mul(add(first, last), 0.5),
                )) > 0.5
            }
            _ => false,
        };
        if depth < 8 && (max - min) >> 10 != 0 && split {
            match curve.kind {
                PathVerb::kQuadraticTo => {
                    let ab = interp(a, b, 0.5);
                    let bc = interp(b, c, 0.5);
                    let mid = interp(ab, bc, 0.5);
                    let mut left = curve;
                    left.points = [a, ab, mid, d];
                    let mut right = curve;
                    right.points = [mid, bc, c, d];
                    self.Subdivide(index, left, min, half, a, mid, depth + 1);
                    self.Subdivide(index, right, half, max, mid, c, depth + 1);
                }
                PathVerb::kCubicTo => {
                    let ab = interp(a, b, 0.5);
                    let bc = interp(b, c, 0.5);
                    let cd = interp(c, d, 0.5);
                    let abc = interp(ab, bc, 0.5);
                    let bcd = interp(bc, cd, 0.5);
                    let mid = interp(abc, bcd, 0.5);
                    let mut left = curve;
                    left.points = [a, ab, abc, mid];
                    let mut right = curve;
                    right.points = [mid, bcd, cd, d];
                    self.Subdivide(index, left, min, half, a, mid, depth + 1);
                    self.Subdivide(index, right, half, max, mid, d, depth + 1);
                }
                PathVerb::kConicTo => {
                    let mid = self.curves[index].Eval(scalar(half));
                    if !mid.x().is_finite() || !mid.y().is_finite() {
                        return;
                    }
                    self.Subdivide(index, curve, min, half, first, mid, depth + 1);
                    self.Subdivide(index, curve, half, max, mid, last, depth + 1);
                }
                _ => (),
            }
        } else {
            let previous = self.length;
            self.length += norm(sub(last, first));
            if self.length > previous {
                self.segments.push(Segment {
                    distance: self.length,
                    curve: index,
                    t: max,
                });
            }
        }
    }
    fn Sample(&self, distance: f32) -> Option<(PointF, PointF)> {
        if self.segments.is_empty() || distance.is_nan() {
            return None;
        }
        let distance = distance.clamp(0., self.length);
        let index = self
            .segments
            .partition_point(|s| s.distance < distance)
            .min(self.segments.len() - 1);
        let segment = self.segments[index];
        let (mut start_t, mut start_d) = (0., 0.);
        if index > 0 {
            let previous = self.segments[index - 1];
            start_d = previous.distance;
            if previous.curve == segment.curve {
                start_t = scalar(previous.t);
            }
        }
        let t = start_t
            + (scalar(segment.t) - start_t) * (distance - start_d) / (segment.distance - start_d);
        if t.is_nan() {
            return None;
        }
        let curve = self.curves[segment.curve];
        Some((curve.Eval(t), curve.Tangent(t)))
    }
}
pub struct ShapePathMeasure {
    contours: Vec<Contour>,
    first: PointF,
    length: f32,
    closed: bool,
}
impl ShapePathMeasure {
    pub fn new(path: &ShapePath) -> Self {
        let mut result = Self {
            contours: Vec::new(),
            first: PointF::default(),
            length: 0.,
            closed: path
                .commands
                .last()
                .is_some_and(|c| c.verb == PathVerb::kClose),
        };
        let (mut current, mut start) = (PointF::default(), PointF::default());
        let mut contour = Contour::default();
        let mut first = true;
        for command in path.commands.iter() {
            let end = p(command.point);
            if [command.point, command.control1, command.control2]
                .iter()
                .any(|p| !p.x.is_finite() || !p.y.is_finite())
                || !command.conic_weight.is_finite()
            {
                return Self {
                    contours: Vec::new(),
                    first: PointF::default(),
                    length: 0.,
                    closed: false,
                };
            }
            if command.verb == PathVerb::kMoveTo {
                if !contour.segments.is_empty() {
                    result.length += contour.length;
                    result.contours.push(contour);
                    contour = Contour::default();
                }
                current = end;
                start = end;
                if first {
                    result.first = end;
                    first = false;
                }
                continue;
            }
            let (kind, points) = match command.verb {
                PathVerb::kLineTo => (
                    PathVerb::kLineTo,
                    [current, end, PointF::default(), PointF::default()],
                ),
                PathVerb::kQuadraticTo | PathVerb::kConicTo => (
                    command.verb,
                    [current, p(command.control1), end, PointF::default()],
                ),
                PathVerb::kCubicTo => (
                    PathVerb::kCubicTo,
                    [current, p(command.control1), p(command.control2), end],
                ),
                PathVerb::kClose => (
                    PathVerb::kLineTo,
                    [current, start, PointF::default(), PointF::default()],
                ),
                _ => continue,
            };
            let curve = Curve {
                kind,
                points,
                weight: command.conic_weight as f32,
            };
            current = curve.End();
            contour.Append(curve);
        }
        if !contour.segments.is_empty() {
            result.length += contour.length;
            result.contours.push(contour);
        }
        result
    }
    pub fn Length(&self) -> f32 {
        self.length
    }
    pub fn IsClosed(&self) -> bool {
        self.closed
    }
    pub fn PointAndTangent(&self, mut distance: f32) -> PointAndTangent {
        if !distance.is_finite() {
            distance = 0.;
        }
        let mut base = 0.;
        for contour in &self.contours {
            if distance <= base + contour.length {
                if let Some((point, tangent)) =
                    contour.Sample((distance - base).clamp(0., contour.length))
                {
                    return PointAndTangent {
                        point,
                        tangent_in_degrees: tangent.y().atan2(tangent.x()).to_degrees(),
                    };
                }
            }
            base += contour.length;
        }
        PointAndTangent {
            point: self.first,
            tangent_in_degrees: 0.,
        }
    }
}

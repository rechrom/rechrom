//! Backend-neutral sink for the canonical BasicShape builder. Its SVG arcs
//! use SkPathBuilder::arcToSvg (src/core/SkPathBuilder.cpp:568-694) conics;
//! Style, Layout and Paint share these immutable, platform-neutral primitives.
#![allow(non_snake_case, non_camel_case_types)]
use crate::{
    gfx::{PointF, RectF},
    svg_path_byte_stream::NativePathSink as BasicShapePathSink,
    WindRule,
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PathPoint {
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PathVerb {
    #[default]
    kMoveTo,
    kLineTo,
    kQuadraticTo,
    kConicTo,
    kCubicTo,
    kClose,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathCommand {
    pub verb: PathVerb,
    pub point: PathPoint,
    pub control1: PathPoint,
    pub control2: PathPoint,
    pub conic_weight: f64,
}
impl Default for PathCommand {
    fn default() -> Self {
        Self {
            verb: PathVerb::kMoveTo,
            point: PathPoint::default(),
            control1: PathPoint::default(),
            control2: PathPoint::default(),
            conic_weight: 1.,
        }
    }
}
#[derive(Clone)]
pub struct ShapePath {
    pub commands: std::sync::Arc<[PathCommand]>,
    pub even_odd: bool,
}
impl ShapePath {
    pub fn Bounds(&self) -> RectF {
        let mut bounds: Option<[f64; 4]> = None;
        let mut include = |p: PathPoint| {
            if let Some([left, top, right, bottom]) = bounds.as_mut() {
                *left = left.min(p.x);
                *top = top.min(p.y);
                *right = right.max(p.x);
                *bottom = bottom.max(p.y);
            } else {
                bounds = Some([p.x, p.y, p.x, p.y]);
            }
        };
        for c in self.commands.iter() {
            match c.verb {
                PathVerb::kClose => (),
                PathVerb::kMoveTo | PathVerb::kLineTo => include(c.point),
                PathVerb::kQuadraticTo | PathVerb::kConicTo => {
                    include(c.control1);
                    include(c.point);
                }
                PathVerb::kCubicTo => {
                    include(c.control1);
                    include(c.control2);
                    include(c.point);
                }
            }
        }
        let Some([left, top, right, bottom]) = bounds else {
            return RectF::default();
        };
        RectF::new(
            PointF::new(left as f32, top as f32),
            crate::gfx::SizeF::new((right - left) as f32, (bottom - top) as f32),
        )
    }
}
#[derive(Default)]
pub struct ShapePathBuilder {
    pub commands: Vec<PathCommand>,
    pub even_odd: bool,
    current: PointF,
    start: PointF,
    closed: bool,
}
fn offset(p: PointF) -> PathPoint {
    PathPoint {
        x: p.x() as f64,
        y: p.y() as f64,
    }
}
fn point(p: PathPoint) -> PointF {
    PointF::new(p.x as f32, p.y as f32)
}
fn add(a: PointF, b: PointF) -> PointF {
    PointF::new(a.x() + b.x(), a.y() + b.y())
}
fn sub(a: PointF, b: PointF) -> PointF {
    PointF::new(a.x() - b.x(), a.y() - b.y())
}
fn mul(p: PointF, s: f32) -> PointF {
    PointF::new(p.x() * s, p.y() * s)
}
fn snap(v: f32) -> f32 {
    if v.abs() <= 1. / 65536. {
        0.
    } else {
        v
    }
}
impl ShapePathBuilder {
    fn ensure_move(&mut self) {
        if self.closed {
            self.MoveTo(self.current);
        }
    }
    fn conic(&mut self, control: PointF, end: PointF, weight: f32) {
        self.ensure_move();
        if weight <= 0. {
            self.LineTo(end);
            return;
        }
        if weight == 1. {
            self.QuadTo(control, end);
            return;
        }
        if !weight.is_finite() {
            self.LineTo(control);
            self.LineTo(end);
            return;
        }
        self.commands.push(PathCommand {
            verb: PathVerb::kConicTo,
            control1: offset(control),
            point: offset(end),
            conic_weight: weight as f64,
            ..Default::default()
        });
        self.current = end;
    }
}
impl BasicShapePathSink for ShapePathBuilder {
    fn SetWindRule(&mut self, rule: WindRule) {
        self.even_odd = rule == WindRule::RULE_EVENODD;
    }
    fn MoveTo(&mut self, p: PointF) {
        self.closed = false;
        self.commands.push(PathCommand {
            point: offset(p),
            ..Default::default()
        });
        self.current = p;
        self.start = p;
    }
    fn LineTo(&mut self, p: PointF) {
        self.ensure_move();
        self.commands.push(PathCommand {
            verb: PathVerb::kLineTo,
            point: offset(p),
            ..Default::default()
        });
        self.current = p;
    }
    fn QuadTo(&mut self, c: PointF, p: PointF) {
        self.ensure_move();
        self.commands.push(PathCommand {
            verb: PathVerb::kQuadraticTo,
            control1: offset(c),
            point: offset(p),
            ..Default::default()
        });
        self.current = p;
    }
    fn CubicTo(&mut self, a: PointF, b: PointF, p: PointF) {
        self.ensure_move();
        self.commands.push(PathCommand {
            verb: PathVerb::kCubicTo,
            control1: offset(a),
            control2: offset(b),
            point: offset(p),
            ..Default::default()
        });
        self.current = p;
    }
    fn Close(&mut self) {
        if self.closed || self.commands.is_empty() {
            return;
        }
        self.closed = true;
        self.commands.push(PathCommand {
            verb: PathVerb::kClose,
            ..Default::default()
        });
        self.current = self.start;
    }
    // Source: SkPathBuilder.cpp:526-560, native tangent arc primitive.
    fn ArcTo(&mut self, p1: PointF, p2: PointF, radius: f32) {
        if radius == 0. {
            self.LineTo(p1);
            return;
        }
        let normalize = |p: PointF| {
            let (x, y) = (p.x() as f64, p.y() as f64);
            let inv = 1. / (x * x + y * y).sqrt();
            [x * inv, y * inv]
        };
        let before = normalize(sub(p1, self.current));
        let after = normalize(sub(p2, p1));
        let cosh = before[0] * after[0] + before[1] * after[1];
        let sinh = before[0] * after[1] - before[1] * after[0];
        if !before.iter().chain(after.iter()).all(|n| n.is_finite())
            || (sinh as f32).abs() <= 1. / 4096.
        {
            self.LineTo(p1);
            return;
        }
        let dist = ((radius as f64 * (1. - cosh) / sinh) as f32).abs();
        let before_end = PointF::new(
            (p1.x() as f64 - dist as f64 * before[0]) as f32,
            (p1.y() as f64 - dist as f64 * before[1]) as f32,
        );
        let after = PointF::new(after[0] as f32, after[1] as f32);
        let length = (after.x() as f64).hypot(after.y() as f64);
        let after = PointF::new(
            (after.x() as f64 * (dist as f64 / length)) as f32,
            (after.y() as f64 * (dist as f64 / length)) as f32,
        );
        self.LineTo(before_end);
        self.conic(p1, add(p1, after), ((0.5 + cosh * 0.5) as f32).sqrt());
    }
    // Source: SkPathBuilder.cpp:568-694. Keep f32 source arithmetic, radius
    // correction, <=120 degree segments, quadrant snapping and exact endpoint.
    fn ArcToSvg(&mut self, rad: PointF, angle: f32, large: bool, sweep: bool, end: PointF) {
        let src = [self.current, end];
        if rad.x() == 0. || rad.y() == 0. || src[0] == src[1] {
            self.LineTo(end);
            return;
        }
        let mut rx = rad.x().abs();
        let mut ry = rad.y().abs();
        let midpoint = mul(sub(src[0], src[1]), 0.5);
        let radians = -angle * (std::f32::consts::PI / 180.);
        let (sin, cos) = (snap(radians.sin()), snap(radians.cos()));
        let transformed = PointF::new(
            midpoint.x() * cos + midpoint.y() * -sin,
            midpoint.x() * sin + midpoint.y() * cos,
        );
        let mut radii_scale = (transformed.x() * transformed.x()) / (rx * rx)
            + (transformed.y() * transformed.y()) / (ry * ry);
        if radii_scale > 1. {
            radii_scale = radii_scale.sqrt();
            rx *= radii_scale;
            ry *= radii_scale;
        }
        let (ix, iy) = (1. / rx, 1. / ry);
        let map_unit = |p: PointF| {
            PointF::new(
                p.x() * (ix * cos) + p.y() * (ix * -sin),
                p.x() * (iy * sin) + p.y() * (iy * cos),
            )
        };
        let mut unit = [map_unit(src[0]), map_unit(src[1])];
        let mut delta = sub(unit[1], unit[0]);
        let d = delta.x() * delta.x() + delta.y() * delta.y();
        let candidate = 1. / d - 0.25;
        let mut factor = if candidate < 0. { 0. } else { candidate }.sqrt();
        if (!sweep) != large {
            factor = -factor;
        }
        delta = mul(delta, factor);
        let center = add(
            mul(add(unit[0], unit[1]), 0.5),
            PointF::new(-delta.y(), delta.x()),
        );
        unit[0] = sub(unit[0], center);
        unit[1] = sub(unit[1], center);
        let theta1 = unit[0].y().atan2(unit[0].x());
        let theta2 = unit[1].y().atan2(unit[1].x());
        let mut arc = theta2 - theta1;
        if arc < 0. && sweep {
            arc += std::f32::consts::PI * 2.;
        } else if arc > 0. && !sweep {
            arc -= std::f32::consts::PI * 2.;
        }
        if arc.abs() < std::f32::consts::PI / 1_000_000. {
            self.LineTo(end);
            return;
        }
        let segments = (arc / (2. * std::f32::consts::PI / 3.)).abs().ceil() as i32;
        let width = arc / segments as f32;
        let tangent = (0.5 * width).tan();
        if !tangent.is_finite() {
            return;
        }
        let radians = angle * (std::f32::consts::PI / 180.);
        let (sin, cos) = (snap(radians.sin()), snap(radians.cos()));
        let map = |p: PointF| {
            PointF::new(
                p.x() * (cos * rx) + p.y() * (-sin * ry),
                p.x() * (sin * rx) + p.y() * (cos * ry),
            )
        };
        let integer = |n: f32| n == n.floor();
        let expect = (std::f32::consts::PI / 2. - width.abs()).abs() <= 1. / 4096.
            && integer(rx)
            && integer(ry)
            && integer(end.x())
            && integer(end.y());
        let mut start = theta1;
        let weight = (0.5 + width.cos() * 0.5).sqrt();
        for _ in 0..segments {
            let end_theta = start + width;
            let (sin, cos) = (snap(end_theta.sin()), snap(end_theta.cos()));
            let p = add(PointF::new(cos, sin), center);
            let c = add(p, PointF::new(tangent * sin, -tangent * cos));
            let mut c = map(c);
            let mut p = map(p);
            if expect {
                c = PointF::new((c.x() + 0.5).floor(), (c.y() + 0.5).floor());
                p = PointF::new((p.x() + 0.5).floor(), (p.y() + 0.5).floor());
            }
            self.conic(c, p, weight);
            start = end_theta;
        }
        if let Some(last) = self.commands.last_mut() {
            last.point = offset(end);
        }
        self.current = end;
    }
    fn Oval(&mut self, rect: RectF) {
        self.RoundedRect(
            rect,
            [PointF::new(rect.width() / 2., rect.height() / 2.); 4],
        );
    }
    fn RoundedRect(&mut self, rect: RectF, mut corners: [PointF; 4]) {
        let mut factor = 1_f32;
        for (available, sum) in [
            (rect.width(), corners[0].x() + corners[1].x()),
            (rect.width(), corners[3].x() + corners[2].x()),
            (rect.height(), corners[0].y() + corners[3].y()),
            (rect.height(), corners[1].y() + corners[2].y()),
        ] {
            if sum > 0. {
                factor = factor.min(available / sum);
            }
        }
        if factor < 1. {
            for c in &mut corners {
                *c = PointF::new(c.x() * factor, c.y() * factor);
            }
        }
        let [tl, tr, br, bl] = corners;
        let (l, t, r, b) = (rect.x(), rect.y(), rect.right(), rect.bottom());
        let w = std::f32::consts::FRAC_1_SQRT_2;
        self.MoveTo(PointF::new(l + tl.x(), t));
        self.LineTo(PointF::new(r - tr.x(), t));
        self.conic(PointF::new(r, t), PointF::new(r, t + tr.y()), w);
        self.LineTo(PointF::new(r, b - br.y()));
        self.conic(PointF::new(r, b), PointF::new(r - br.x(), b), w);
        self.LineTo(PointF::new(l + bl.x(), b));
        self.conic(PointF::new(l, b), PointF::new(l, b - bl.y()), w);
        self.LineTo(PointF::new(l, t + tl.y()));
        self.conic(PointF::new(l, t), PointF::new(l + tl.x(), t), w);
        self.Close();
    }
}
impl ShapePathBuilder {
    pub fn Finish(self) -> ShapePath {
        ShapePath {
            commands: self.commands.into(),
            even_odd: self.even_odd,
        }
    }
}

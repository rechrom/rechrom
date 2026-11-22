//! SVGMarkerDataBuilder, svg_marker_data.cc:30-297; arc tangent normalization
//! from svg_path_parser.cc:174-277, pinned Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform,
    gfx::{PointF, RectF, Vector2dF},
    native_path::{NativePath, NativePathElement as Element},
    svg_path_byte_stream::{BuildSvgByteStream, NativePathSink},
    WindRule,
};
#[derive(Clone, Copy, PartialEq)]
pub enum MarkerType {
    Start,
    Mid,
    End,
}
#[derive(Clone, Copy)]
pub struct MarkerPosition {
    pub kind: MarkerType,
    pub origin: PointF,
    pub angle: f32,
}
fn sub(a: PointF, b: PointF) -> Vector2dF {
    Vector2dF::new(a.x() - b.x(), a.y() - b.y())
}
fn zero(v: Vector2dF) -> bool {
    v.x() == 0. && v.y() == 0.
}
fn quad(start: PointF, control: PointF, end: PointF) -> (Vector2dF, Vector2dF) {
    let mut incoming = sub(control, start);
    let mut outgoing = sub(end, control);
    if zero(incoming) {
        incoming = outgoing;
    } else if zero(outgoing) {
        outgoing = incoming;
    }
    (incoming, outgoing)
}
struct Builder {
    positions: Vec<MarkerPosition>,
    last_move: usize,
    last: Element,
    origin: PointF,
    start: PointF,
    in_slope: Vector2dF,
    out_slope: Vector2dF,
    move_out_slope: Vector2dF,
}
impl Default for Builder {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            last_move: 0,
            last: Element::Move(PointF::default()),
            origin: PointF::default(),
            start: PointF::default(),
            in_slope: Vector2dF::default(),
            out_slope: Vector2dF::default(),
            move_out_slope: Vector2dF::default(),
        }
    }
}
impl Builder {
    fn UpdateAngle(&mut self, ends_subpath: bool) {
        let closed = matches!(self.last, Element::Close);
        if closed {
            self.out_slope = self.move_out_slope;
        }
        let inbound =
            (self.in_slope.SlopeAngleRadians() * (180.0f32 / std::f32::consts::PI)) as f64;
        let outbound =
            (self.out_slope.SlopeAngleRadians() * (180.0f32 / std::f32::consts::PI)) as f64;
        let angle = if closed || (!ends_subpath && !matches!(self.last, Element::Move(_))) {
            let diff = inbound - outbound;
            let inbound = if diff > 180. || diff <= -180. {
                inbound + 360.
            } else {
                inbound
            };
            (inbound + outbound) / 2.
        } else if ends_subpath {
            inbound
        } else {
            outbound
        };
        let angle = angle.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32;
        if closed {
            self.positions[self.last_move].angle = angle;
        }
        self.positions.last_mut().unwrap().angle = angle;
    }
    fn Update(&mut self, element: Element) {
        let (position, start_tangent, end_tangent) = match element {
            Element::Cubic(a, b, end) => {
                let mut start = sub(a, self.origin);
                let mut finish = sub(end, b);
                if zero(start) {
                    (start, finish) = quad(a, b, end);
                } else if zero(finish) {
                    (start, finish) = quad(self.origin, a, b);
                }
                (end, start, finish)
            }
            Element::Quad(control, end) => {
                let (a, b) = quad(self.origin, control, end);
                (end, a, b)
            }
            Element::Move(end) | Element::Line(end) => {
                let slope = sub(end, self.origin);
                (end, slope, slope)
            }
            Element::Close => {
                let mut slope = sub(self.start, self.origin);
                if !matches!(self.last, Element::Move(_)) && zero(slope) {
                    slope = self.move_out_slope;
                }
                (self.start, slope, slope)
            }
        };
        self.out_slope = start_tangent;
        if matches!(self.last, Element::Move(_)) {
            self.move_out_slope = self.out_slope;
        }
        let starts = matches!(element, Element::Move(_));
        if !self.positions.is_empty() {
            self.UpdateAngle(starts);
        }
        self.in_slope = end_tangent;
        self.origin = position;
        if starts {
            self.start = position;
            self.last_move = self.positions.len();
        }
        self.last = element;
        let kind = if self.positions.is_empty() {
            MarkerType::Start
        } else {
            MarkerType::Mid
        };
        self.positions.push(MarkerPosition {
            kind,
            origin: self.origin,
            angle: 0.,
        });
    }
    fn Finish(mut self) -> Vec<MarkerPosition> {
        if !self.positions.is_empty() {
            self.UpdateAngle(true);
            self.positions.last_mut().unwrap().kind = MarkerType::End;
        }
        self.positions
    }
}
pub fn BuildFromPath(path: &dyn NativePath) -> Vec<MarkerPosition> {
    let mut builder = Builder::default();
    path.Apply(&mut |element| builder.Update(element));
    builder.Finish()
}
pub fn BuildFromByteStream(bytes: &[u8]) -> Vec<MarkerPosition> {
    let mut builder = Builder::default();
    BuildSvgByteStream(bytes, 1., 0., 0., &mut builder);
    builder.Finish()
}
impl NativePathSink for Builder {
    fn SetWindRule(&mut self, _: WindRule) {}
    fn MoveTo(&mut self, p: PointF) {
        self.Update(Element::Move(p));
    }
    fn LineTo(&mut self, p: PointF) {
        self.Update(Element::Line(p));
    }
    fn QuadTo(&mut self, c: PointF, p: PointF) {
        // SVGPathNormalizer::BlendPoints preserves the source float order.
        let blend = |a: PointF, b: PointF| {
            PointF::new(
                (a.x() + 2. * b.x()) * (1. / 3.),
                (a.y() + 2. * b.y()) * (1. / 3.),
            )
        };
        self.Update(Element::Cubic(blend(self.origin, c), blend(p, c), p));
    }
    fn CubicTo(&mut self, a: PointF, b: PointF, p: PointF) {
        self.Update(Element::Cubic(a, b, p));
    }
    fn ArcToSvg(&mut self, r: PointF, a: f32, large: bool, sweep: bool, end: PointF) {
        let curves = ArcCurves(self.origin, r, a, large, sweep, end);
        // MarkerPathSegmentProcessor merges all decomposed curves of one SVG
        // arc into one endpoint pair with the first/last matching tangents.
        if let (Some(first), Some(last)) = (curves.first(), curves.last()) {
            self.Update(Element::Cubic(first.0, last.1, last.2));
        } else {
            self.Update(Element::Line(end));
        }
    }
    fn Close(&mut self) {
        self.Update(Element::Close);
    }
    fn ArcTo(&mut self, _: PointF, _: PointF, _: f32) {
        unreachable!("SVG byte stream has no canvas corner arc");
    }
    fn Oval(&mut self, _: RectF) {
        unreachable!("SVG byte stream has no oval primitive");
    }
    fn RoundedRect(&mut self, _: RectF, _: [PointF; 4]) {
        unreachable!("SVG byte stream has no rounded-rect primitive");
    }
}
fn ArcCurves(
    current: PointF,
    radii: PointF,
    angle: f32,
    large: bool,
    sweep: bool,
    end: PointF,
) -> Vec<(PointF, PointF, PointF)> {
    let mut rx = radii.x().abs();
    let mut ry = radii.y().abs();
    let mut curves = Vec::new();
    if rx == 0. || ry == 0. || end == current {
        return curves;
    }
    let midpoint = PointF::new((current.x() - end.x()) * 0.5, (current.y() - end.y()) * 0.5);
    let mut transform = AffineTransform::default();
    transform.Rotate(-(angle as f64));
    let middle = transform.MapPoint(&midpoint);
    let scale = middle.x() * middle.x() / (rx * rx) + middle.y() * middle.y() / (ry * ry);
    if scale > 1. {
        rx *= scale.sqrt();
        ry *= scale.sqrt();
    }
    transform = AffineTransform::default();
    transform
        .Scale((1. / rx) as f64, (1. / ry) as f64)
        .Rotate(-(angle as f64));
    let first = transform.MapPoint(&current);
    let second = transform.MapPoint(&end);
    let mut delta = sub(second, first);
    let scale_squared =
        ((1.0f32 / (delta.x() * delta.x() + delta.y() * delta.y())) as f64 - 0.25).max(0.);
    let mut factor = scale_squared.sqrt().min(f32::MAX as f64) as f32;
    if sweep == large {
        factor = -factor;
    }
    delta = Vector2dF::new(delta.x() * factor, delta.y() * factor);
    let center = PointF::new(
        (first.x() + second.x()) * 0.5 - delta.y(),
        (first.y() + second.y()) * 0.5 + delta.x(),
    );
    let theta1 = sub(first, center).SlopeAngleRadians();
    let theta2 = sub(second, center).SlopeAngleRadians();
    let mut arc = theta2 - theta1;
    if arc < 0. && sweep {
        arc += std::f32::consts::TAU;
    } else if arc > 0. && !sweep {
        arc -= std::f32::consts::TAU;
    }
    transform = AffineTransform::default();
    transform.Rotate(angle as f64).Scale(rx as f64, ry as f64);
    let segments = (arc.abs() / (std::f32::consts::FRAC_PI_2 + 0.001)).ceil() as i32;
    for i in 0..segments {
        let start = theta1 + i as f32 * arc / segments as f32;
        let finish = theta1 + (i + 1) as f32 * arc / segments as f32;
        let t = (8. / 6.0f32) * (0.25 * (finish - start)).tan();
        if !t.is_finite() {
            return curves;
        }
        let (sin_start, cos_start) = (start.sin(), start.cos());
        let (sin_end, cos_end) = (finish.sin(), finish.cos());
        let a = PointF::new(
            cos_start - t * sin_start + center.x(),
            sin_start + t * cos_start + center.y(),
        );
        let end = PointF::new(cos_end + center.x(), sin_end + center.y());
        let b = PointF::new(end.x() + t * sin_end, end.y() - t * cos_end);
        curves.push((
            transform.MapPoint(&a),
            transform.MapPoint(&b),
            transform.MapPoint(&end),
        ));
    }
    curves
}

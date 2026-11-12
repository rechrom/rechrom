//! Source basic-shape geometry over the canonical compiled Style owner.
//! Source: core/style/basic_shapes.cc:49-370; style_path.cc:53-63.
//! The sink performs native Path primitives, including conic arcs. No shape,
//! Style, SVG byte-stream or resource graph is flattened at this boundary.
use foundation::{
    gfx::{PointF, RectF},
    FloatValueForLength, WindRule,
};
use layoutng_style::style::{basic_shapes::*, style_path::StylePath};

pub use foundation::svg_path_byte_stream::{
    BuildSvgByteStream, NativePathSink as BasicShapePathSink,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Source precondition failure: ray() has independent motion geometry and is
/// excluded by the border-shape/clip-path grammar (StyleRay::GetPath is
/// NOTREACHED in Chromium). This is not an unsupported filled basic shape.
pub enum InvalidBasicShape {
    MotionRay,
}
fn corner_radius(center: PointF, width: f32, height: f32, closest: bool) -> f32 {
    let select = if closest { f32::min } else { f32::max };
    select(center.x(), width - center.x()).hypot(select(center.y(), height - center.y()))
}
fn radius(
    value: &BasicShapeRadius,
    center: PointF,
    width: f32,
    height: f32,
    axis: Option<bool>,
) -> f32 {
    match value.GetType() {
        RadiusType::kClosestCorner => corner_radius(center, width, height, true),
        RadiusType::kFarthestCorner => corner_radius(center, width, height, false),
        RadiusType::kValue => FloatValueForLength(
            value.Value(),
            axis.map_or(width.hypot(height) / 2_f32.sqrt(), |x| {
                if x {
                    width
                } else {
                    height
                }
            }),
        ),
        RadiusType::kClosestSide | RadiusType::kFarthestSide => {
            let closest = value.GetType() == RadiusType::kClosestSide;
            let pick = if closest { f32::min } else { f32::max };
            let side =
                |c: f32, extent: f32| pick(if closest { c.abs() } else { c }, (extent - c).abs());
            axis.map_or_else(
                || pick(side(center.x(), width), side(center.y(), height)),
                |x| {
                    if x {
                        side(center.x(), width)
                    } else {
                        side(center.y(), height)
                    }
                },
            )
        }
    }
}
#[derive(Clone, Copy)]
struct Edge {
    x: f32,
    y: f32,
    length: f32,
}
impl Edge {
    fn new(a: PointF, b: PointF) -> Self {
        let (x, y) = (b.x() - a.x(), b.y() - a.y());
        let length = x.hypot(y);
        Self {
            x: x / length,
            y: y / length,
            length,
        }
    }
    fn rounding(self, next: Self, requested: f32) -> f32 {
        if requested <= 0. || self.length <= 0.5 || next.length <= 0.5 {
            return 0.;
        }
        let angle = (-(self.x * next.x + self.y * next.y) as f64)
            .clamp(-1., 1.)
            .acos();
        if angle <= f64::EPSILON || (angle - std::f64::consts::PI).abs() <= f64::EPSILON {
            return 0.;
        }
        let tan = (angle / 2.).tan();
        if !tan.is_finite() || tan <= f64::EPSILON {
            return 0.;
        }
        let radius = (requested as f64)
            .min(tan * self.length as f64 * 0.5)
            .min(tan * next.length as f64 * 0.5);
        if radius <= f64::EPSILON {
            0.
        } else {
            radius as f32
        }
    }
}
pub fn BuildBasicShapePath(
    shape: &dyn BasicShape,
    rect: RectF,
    zoom: f32,
    scale: f32,
    sink: &mut dyn BasicShapePathSink,
) -> Result<(), InvalidBasicShape> {
    let center = |shape: &dyn BasicShapeWithCenterAndRadii| {
        PointF::new(
            FloatValueForLength(shape.Center().X(), rect.width()),
            FloatValueForLength(shape.Center().Y(), rect.height()),
        )
    };
    let ellipse = |center: PointF, rx: f32, ry: f32, sink: &mut dyn BasicShapePathSink| {
        let (x, y) = (
            (center.x() + rect.x()) * scale,
            (center.y() + rect.y()) * scale,
        );
        sink.Oval(RectF::new(
            foundation::gfx::PointF::new(x - rx * scale, y - ry * scale),
            foundation::gfx::SizeF::new(2. * rx * scale, 2. * ry * scale),
        ));
    };
    match shape.GetType() {
        ShapeType::kBasicShapeCircleType => {
            let shape = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeCircle) };
            let c = center(shape);
            let r = radius(shape.Radius(), c, rect.width(), rect.height(), None);
            ellipse(c, r, r, sink);
        }
        ShapeType::kBasicShapeEllipseType => {
            let shape = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeEllipse) };
            let c = center(shape);
            ellipse(
                c,
                radius(shape.RadiusX(), c, rect.width(), rect.height(), Some(true)),
                radius(shape.RadiusY(), c, rect.width(), rect.height(), Some(false)),
                sink,
            );
        }
        ShapeType::kBasicShapePolygonType => {
            let shape = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapePolygon) };
            sink.SetWindRule(shape.GetWindRule());
            let values = shape.Values().as_slice();
            assert_eq!(values.len() % 2, 0);
            let points: Vec<_> = values
                .chunks_exact(2)
                .map(|p| {
                    PointF::new(
                        (FloatValueForLength(&p[0], rect.width()) + rect.x()) * scale,
                        (FloatValueForLength(&p[1], rect.height()) + rect.y()) * scale,
                    )
                })
                .collect();
            if points.is_empty() {
                return Ok(());
            }
            let requested = FloatValueForLength(shape.RoundingRadius(), 0.) * scale;
            if requested <= 0.5 || points.len() < 3 {
                sink.MoveTo(points[0]);
                for point in &points[1..] {
                    sink.LineTo(*point);
                }
            } else {
                let (last, first) = (*points.last().unwrap(), points[0]);
                sink.MoveTo(PointF::new(
                    0.5 * last.x() + 0.5 * first.x(),
                    0.5 * last.y() + 0.5 * first.y(),
                ));
                let mut incoming = Edge::new(last, first);
                for i in 0..points.len() {
                    let current = points[i];
                    let next = points[(i + 1) % points.len()];
                    let outgoing = Edge::new(current, next);
                    let r = incoming.rounding(outgoing, requested);
                    if r > 0. {
                        sink.ArcTo(current, next, r);
                    } else {
                        sink.LineTo(current);
                    }
                    incoming = outgoing;
                }
            }
            sink.Close();
        }
        ShapeType::kBasicShapeInsetType => {
            let shape = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeInset) };
            let left = FloatValueForLength(shape.Left(), rect.width());
            let top = FloatValueForLength(shape.Top(), rect.height());
            let output = RectF::new(
                PointF::new((rect.x() + left) * scale, (rect.y() + top) * scale),
                foundation::gfx::SizeF::new(
                    (rect.width() - left - FloatValueForLength(shape.Right(), rect.width()))
                        .max(0.)
                        * scale,
                    (rect.height() - top - FloatValueForLength(shape.Bottom(), rect.height()))
                        .max(0.)
                        * scale,
                ),
            );
            let radii = [
                shape.TopLeftRadius(),
                shape.TopRightRadius(),
                shape.BottomRightRadius(),
                shape.BottomLeftRadius(),
            ]
            .map(|r| {
                PointF::new(
                    FloatValueForLength(r.Width(), rect.width()) * scale,
                    FloatValueForLength(r.Height(), rect.height()) * scale,
                )
            });
            sink.RoundedRect(output, radii);
        }
        ShapeType::kStylePathType => {
            let shape = unsafe { &*(shape as *const dyn BasicShape as *const StylePath) };
            sink.SetWindRule(shape.GetWindRule());
            BuildSvgByteStream(
                shape.ByteStream().Span(),
                zoom * scale,
                rect.x(),
                rect.y(),
                sink,
            );
        }
        ShapeType::kStyleRayType => return Err(InvalidBasicShape::MotionRay),
        ShapeType::kStyleShapeType => {
            let shape = unsafe {
                &*(shape as *const dyn BasicShape
                    as *const layoutng_style::style::style_shape::StyleShape)
            };
            crate::style_shape_path::BuildStyleShapePath(shape, rect, scale, sink);
        }
    }
    Ok(())
}

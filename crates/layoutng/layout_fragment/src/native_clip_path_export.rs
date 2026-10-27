//! Project native Blink basic-shape ownership into fragment-owned Paint input.
//! Geometry stays unresolved until PrePaint owns the fragment reference box.
#![allow(non_snake_case)]

use foundation::{Length, WindRule};
use layoutng_style::style::{
    basic_shapes::{
        BasicShape, BasicShapeCircle, BasicShapeEllipse, BasicShapeInset, BasicShapePolygon,
        BasicShapeRadius, BasicShapeWithCenterAndRadii, RadiusType, ShapeType,
    },
    clip_path_operation::OperationType,
    computed_style::ComputedStyle,
    computed_style_constants::GeometryBox,
    reference_clip_path_operation::ReferenceClipPathOperation,
    shape_clip_path_operation::ShapeClipPathOperation,
    style_path::StylePath,
};

use crate::internal::{
    layout_input::{Offset, PaintPathCommand, PaintPathVerb},
    paint_input::{
        ClipPathCircle, ClipPathEllipse, ClipPathInset, ClipPathLength, ClipPathPaint,
        ClipPathRadius, ClipPathRadiusKind, ClipPathReferenceBox, ClipPathShapeRadius,
    },
};

fn LengthParts(value: &Length) -> (f64, f64) {
    if value.IsFixed() {
        return (value.Pixels() as f64, 0.0);
    }
    if value.IsPercent() {
        return (0.0, value.PercentValue() as f64);
    }
    if value.IsCalculated() {
        let calculation = value.GetCalculationValue();
        if !calculation.IsExpression() {
            let pair = calculation.GetPixelsAndPercent();
            return (
                pair.has_explicit_pixels
                    .then_some(pair.pixels as f64)
                    .unwrap_or(0.0),
                pair.has_explicit_percent
                    .then_some(pair.percent as f64)
                    .unwrap_or(0.0),
            );
        }
    }
    (0.0, 0.0)
}

fn ClipLength(value: &Length) -> ClipPathLength {
    let (pixels, percentage) = LengthParts(value);
    ClipPathLength { pixels, percentage }
}

fn CornerRadius(value: &foundation::LengthSize) -> ClipPathRadius {
    ClipPathRadius {
        x: ClipLength(value.Width()),
        y: ClipLength(value.Height()),
    }
}

fn ShapeRadius(value: &BasicShapeRadius) -> ClipPathShapeRadius {
    ClipPathShapeRadius {
        kind: match value.GetType() {
            RadiusType::kValue => ClipPathRadiusKind::kValue,
            RadiusType::kClosestSide => ClipPathRadiusKind::kClosestSide,
            RadiusType::kFarthestSide => ClipPathRadiusKind::kFarthestSide,
            RadiusType::kClosestCorner => ClipPathRadiusKind::kClosestCorner,
            RadiusType::kFarthestCorner => ClipPathRadiusKind::kFarthestCorner,
        },
        value: ClipLength(value.Value()),
    }
}

fn PointCommand(verb: PaintPathVerb, point: Offset) -> PaintPathCommand {
    PaintPathCommand {
        verb,
        point,
        ..PaintPathCommand::default()
    }
}

#[derive(Clone, Copy, Default)]
struct Segment {
    command: u16,
    target: Offset,
    point1: Offset,
    point2: Offset,
    arc_large: bool,
    arc_sweep: bool,
}

struct ByteStreamReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ByteStreamReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    fn has_more(&self) -> bool {
        self.position < self.bytes.len()
    }
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let end = self
            .position
            .checked_add(N)
            .expect("SVG path byte offset overflow");
        let bytes: [u8; N] = self
            .bytes
            .get(self.position..end)
            .expect("truncated native SVGPathByteStream")
            .try_into()
            .unwrap();
        self.position = end;
        bytes
    }
    fn u16(&mut self) -> u16 {
        u16::from_ne_bytes(self.take())
    }
    fn float(&mut self) -> f64 {
        f32::from_ne_bytes(self.take()) as f64
    }
    fn flag(&mut self) -> bool {
        self.take::<1>()[0] != 0
    }
    fn point(&mut self) -> Offset {
        Offset {
            x: self.float(),
            y: self.float(),
        }
    }
    fn segment(&mut self) -> Segment {
        let mut segment = Segment {
            command: self.u16(),
            ..Segment::default()
        };
        match segment.command {
            2 | 3 | 4 | 5 | 18 | 19 => segment.target = self.point(),
            12 | 13 => segment.target.x = self.float(),
            14 | 15 => segment.target.y = self.float(),
            1 => {}
            6 | 7 => {
                segment.point1 = self.point();
                segment.point2 = self.point();
                segment.target = self.point();
            }
            16 | 17 => {
                segment.point2 = self.point();
                segment.target = self.point();
            }
            8 | 9 => {
                segment.point1 = self.point();
                segment.target = self.point();
            }
            10 | 11 => {
                segment.point1 = self.point(); // radii
                segment.point2.x = self.float(); // rotation in degrees
                segment.arc_large = self.flag();
                segment.arc_sweep = self.flag();
                segment.target = self.point();
            }
            _ => panic!("invalid native SVG path segment {}", segment.command),
        }
        segment
    }
}

fn add(a: Offset, b: Offset) -> Offset {
    Offset {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}
fn sub(a: Offset, b: Offset) -> Offset {
    Offset {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
fn scale(a: Offset, value: f64) -> Offset {
    Offset {
        x: a.x * value,
        y: a.y * value,
    }
}
fn reflect(point: Offset, control: Offset) -> Offset {
    Offset {
        x: 2.0 * point.x - control.x,
        y: 2.0 * point.y - control.y,
    }
}
fn cubic(command1: Offset, command2: Offset, target: Offset) -> PaintPathCommand {
    PaintPathCommand {
        verb: PaintPathVerb::kCubicTo,
        control1: command1,
        control2: command2,
        point: target,
        ..PaintPathCommand::default()
    }
}

/// SVG 2 endpoint-to-center conversion, matching SVGPathNormalizer's cubic
/// decomposition. Each emitted span is at most 90 degrees.
fn ArcToCubics(current: Offset, segment: &Segment, output: &mut Vec<PaintPathCommand>) -> bool {
    let mut rx = segment.point1.x.abs();
    let mut ry = segment.point1.y.abs();
    if rx == 0.0 || ry == 0.0 || segment.target == current {
        return false;
    }
    let phi = segment.point2.x.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();
    let half = scale(sub(current, segment.target), 0.5);
    let x1 = cos_phi * half.x + sin_phi * half.y;
    let y1 = -sin_phi * half.x + cos_phi * half.y;
    let radii_scale = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
    if radii_scale > 1.0 {
        let factor = radii_scale.sqrt();
        rx *= factor;
        ry *= factor;
    }
    let numerator = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
    let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    if denominator == 0.0 {
        return false;
    }
    let sign = if segment.arc_large == segment.arc_sweep {
        -1.0
    } else {
        1.0
    };
    let coefficient = sign * (numerator / denominator).sqrt();
    let cx1 = coefficient * (rx * y1 / ry);
    let cy1 = coefficient * (-ry * x1 / rx);
    let center = Offset {
        x: cos_phi * cx1 - sin_phi * cy1 + (current.x + segment.target.x) * 0.5,
        y: sin_phi * cx1 + cos_phi * cy1 + (current.y + segment.target.y) * 0.5,
    };
    let unit_start = Offset {
        x: (x1 - cx1) / rx,
        y: (y1 - cy1) / ry,
    };
    let unit_end = Offset {
        x: (-x1 - cx1) / rx,
        y: (-y1 - cy1) / ry,
    };
    let mut theta = unit_start.y.atan2(unit_start.x);
    let mut sweep = (unit_start.x * unit_end.y - unit_start.y * unit_end.x)
        .atan2(unit_start.x * unit_end.x + unit_start.y * unit_end.y);
    if !segment.arc_sweep && sweep > 0.0 {
        sweep -= std::f64::consts::TAU;
    } else if segment.arc_sweep && sweep < 0.0 {
        sweep += std::f64::consts::TAU;
    }
    let count = (sweep.abs() / (std::f64::consts::FRAC_PI_2 + 0.001)).ceil() as usize;
    if count == 0 {
        return false;
    }
    let delta = sweep / count as f64;
    let map = |point: Offset| Offset {
        x: center.x + cos_phi * rx * point.x - sin_phi * ry * point.y,
        y: center.y + sin_phi * rx * point.x + cos_phi * ry * point.y,
    };
    for _ in 0..count {
        let next = theta + delta;
        let alpha = (4.0 / 3.0) * (delta * 0.25).tan();
        if !alpha.is_finite() {
            return false;
        }
        let (sin_start, cos_start) = theta.sin_cos();
        let (sin_end, cos_end) = next.sin_cos();
        output.push(cubic(
            map(Offset {
                x: cos_start - alpha * sin_start,
                y: sin_start + alpha * cos_start,
            }),
            map(Offset {
                x: cos_end + alpha * sin_end,
                y: sin_end - alpha * cos_end,
            }),
            map(Offset {
                x: cos_end,
                y: sin_end,
            }),
        ));
        theta = next;
    }
    true
}

/// Normalize relative, H/V, smooth, quadratic and arc segments into the
/// Move/Line/Cubic/Close vocabulary used by Paint and Raster.
pub(crate) fn NormalizePath(bytes: &[u8]) -> Vec<PaintPathCommand> {
    let mut reader = ByteStreamReader::new(bytes);
    let mut output = Vec::new();
    let mut current = Offset::default();
    let mut subpath = Offset::default();
    let mut control = Offset::default();
    let mut last = 0_u16;
    while reader.has_more() {
        let original = reader.segment();
        let mut segment = original;
        match original.command {
            9 => {
                segment.point1 = add(segment.point1, current);
                segment.target = add(segment.target, current);
            }
            7 => {
                segment.point1 = add(segment.point1, current);
                segment.point2 = add(segment.point2, current);
                segment.target = add(segment.target, current);
            }
            17 => {
                segment.point2 = add(segment.point2, current);
                segment.target = add(segment.target, current);
            }
            3 | 5 | 13 | 15 | 19 | 11 => segment.target = add(segment.target, current),
            12 => segment.target.y = current.y,
            14 => segment.target.x = current.x,
            1 => segment.target = subpath,
            _ => {}
        }
        match original.command {
            2 | 3 => {
                subpath = segment.target;
                output.push(PointCommand(PaintPathVerb::kMoveTo, segment.target));
            }
            4 | 5 | 12 | 13 | 14 | 15 => {
                output.push(PointCommand(PaintPathVerb::kLineTo, segment.target));
            }
            16 | 17 => {
                segment.point1 = if matches!(last, 6 | 7 | 16 | 17) {
                    reflect(current, control)
                } else {
                    current
                };
                control = segment.point2;
                output.push(cubic(segment.point1, segment.point2, segment.target));
            }
            6 | 7 => {
                control = segment.point2;
                output.push(cubic(segment.point1, segment.point2, segment.target));
            }
            18 | 19 => {
                segment.point1 = if matches!(last, 8 | 9 | 18 | 19) {
                    reflect(current, control)
                } else {
                    current
                };
                control = segment.point1;
                output.push(cubic(
                    add(scale(current, 1.0 / 3.0), scale(segment.point1, 2.0 / 3.0)),
                    add(
                        scale(segment.target, 1.0 / 3.0),
                        scale(segment.point1, 2.0 / 3.0),
                    ),
                    segment.target,
                ));
            }
            8 | 9 => {
                control = segment.point1;
                output.push(cubic(
                    add(scale(current, 1.0 / 3.0), scale(segment.point1, 2.0 / 3.0)),
                    add(
                        scale(segment.target, 1.0 / 3.0),
                        scale(segment.point1, 2.0 / 3.0),
                    ),
                    segment.target,
                ));
            }
            10 | 11 => {
                if !ArcToCubics(current, &segment, &mut output) {
                    output.push(PointCommand(PaintPathVerb::kLineTo, segment.target));
                }
            }
            1 => output.push(PaintPathCommand {
                verb: PaintPathVerb::kClose,
                ..PaintPathCommand::default()
            }),
            _ => unreachable!(),
        }
        current = segment.target;
        if !matches!(original.command, 6 | 7 | 8 | 9 | 16 | 17 | 18 | 19) {
            control = current;
        }
        last = original.command;
    }
    output
}

fn ShapeOperation(style: &ComputedStyle) -> Option<(&ShapeClipPathOperation, &dyn BasicShape)> {
    let operation = unsafe { style.ClipPath()?.as_ref()? };
    if operation.GetType() != OperationType::kShape {
        return None;
    }
    let shape_operation = unsafe {
        &*(operation as *const dyn layoutng_style::style::clip_path_operation::ClipPathOperation
            as *const ShapeClipPathOperation)
    };
    Some((shape_operation, shape_operation.GetBasicShape()))
}

pub(super) fn ExportClipPath(style: &ComputedStyle) -> Option<ClipPathPaint> {
    let raw = unsafe { style.ClipPath()?.as_ref()? };
    if raw.GetType() == OperationType::kReference {
        let reference = unsafe {
            &*(raw as *const dyn layoutng_style::style::clip_path_operation::ClipPathOperation
                as *const ReferenceClipPathOperation)
        };
        return Some(ClipPathPaint {
            reference_url: Some(reference.Url().Utf8()),
            reference_scope: reference.ResourceScope(),
            ..ClipPathPaint::default()
        });
    }
    let (operation, shape) = ShapeOperation(style)?;
    let mut output = ClipPathPaint::default();
    output.reference_box = match operation.GetGeometryBox() {
        GeometryBox::kBorderBox => ClipPathReferenceBox::kBorderBox,
        GeometryBox::kPaddingBox => ClipPathReferenceBox::kPaddingBox,
        GeometryBox::kContentBox => ClipPathReferenceBox::kContentBox,
        GeometryBox::kMarginBox => ClipPathReferenceBox::kMarginBox,
        GeometryBox::kFillBox => ClipPathReferenceBox::kFillBox,
        GeometryBox::kStrokeBox => ClipPathReferenceBox::kStrokeBox,
        GeometryBox::kViewBox => ClipPathReferenceBox::kViewBox,
        GeometryBox::kHalfBorderBox => ClipPathReferenceBox::kHalfBorderBox,
    };
    match shape.GetType() {
        ShapeType::kBasicShapeInsetType => {
            let inset = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeInset) };
            output.inset = Some(ClipPathInset {
                top: ClipLength(inset.Top()),
                right: ClipLength(inset.Right()),
                bottom: ClipLength(inset.Bottom()),
                left: ClipLength(inset.Left()),
                top_left_radius: CornerRadius(inset.TopLeftRadius()),
                top_right_radius: CornerRadius(inset.TopRightRadius()),
                bottom_right_radius: CornerRadius(inset.BottomRightRadius()),
                bottom_left_radius: CornerRadius(inset.BottomLeftRadius()),
            });
        }
        ShapeType::kBasicShapeCircleType => {
            let circle = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeCircle) };
            output.circle = Some(ClipPathCircle {
                center_x: ClipLength(circle.Center().X()),
                center_y: ClipLength(circle.Center().Y()),
                radius: ShapeRadius(circle.Radius()),
            });
        }
        ShapeType::kBasicShapeEllipseType => {
            let ellipse = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapeEllipse) };
            output.ellipse = Some(ClipPathEllipse {
                center_x: ClipLength(ellipse.Center().X()),
                center_y: ClipLength(ellipse.Center().Y()),
                radius_x: ShapeRadius(ellipse.RadiusX()),
                radius_y: ShapeRadius(ellipse.RadiusY()),
            });
        }
        ShapeType::kBasicShapePolygonType => {
            let polygon = unsafe { &*(shape as *const dyn BasicShape as *const BasicShapePolygon) };
            let values = polygon.Values();
            if !values.is_empty() {
                for (index, point) in values.as_slice().chunks_exact(2).enumerate() {
                    let (x, xp) = LengthParts(&point[0]);
                    let (y, yp) = LengthParts(&point[1]);
                    let verb = if index == 0 {
                        PaintPathVerb::kMoveTo
                    } else {
                        PaintPathVerb::kLineTo
                    };
                    output.commands.push(PointCommand(verb, Offset { x, y }));
                    output.percentage_commands.push(PointCommand(
                        verb,
                        Offset {
                            x: xp / 100.0,
                            y: yp / 100.0,
                        },
                    ));
                }
                output.commands.push(PaintPathCommand {
                    verb: PaintPathVerb::kClose,
                    ..PaintPathCommand::default()
                });
                output.percentage_commands.push(PaintPathCommand {
                    verb: PaintPathVerb::kClose,
                    ..PaintPathCommand::default()
                });
            }
            output.polygon_rounding_radius = Some(ClipLength(polygon.RoundingRadius()));
            output.even_odd = polygon.GetWindRule() == WindRule::RULE_EVENODD;
        }
        ShapeType::kStylePathType => {
            let path = unsafe { &*(shape as *const dyn BasicShape as *const StylePath) };
            output.commands = NormalizePath(path.ByteStream().Span());
            output.path_zoom = style.EffectiveZoom() as f64;
            output.even_odd = path.GetWindRule() == WindRule::RULE_EVENODD;
        }
        ShapeType::kStyleRayType | ShapeType::kStyleShapeType => return None,
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(segment: u16, floats: &[f32]) -> Vec<u8> {
        let mut output = segment.to_ne_bytes().to_vec();
        for value in floats {
            output.extend(value.to_ne_bytes());
        }
        output
    }

    #[test]
    fn normalizes_relative_smooth_and_arc_segments() {
        let mut stream = bytes(2, &[10.0, 10.0]);
        stream.extend(bytes(7, &[10.0, 0.0, 20.0, 0.0, 30.0, 10.0]));
        stream.extend(bytes(17, &[20.0, 20.0, 30.0, 10.0]));
        let mut arc = 11_u16.to_ne_bytes().to_vec();
        arc.extend(10_f32.to_ne_bytes());
        arc.extend(10_f32.to_ne_bytes());
        arc.extend(0_f32.to_ne_bytes());
        arc.extend([0, 1]);
        arc.extend(20_f32.to_ne_bytes());
        arc.extend(0_f32.to_ne_bytes());
        stream.extend(arc);
        let path = NormalizePath(&stream);
        assert_eq!(path[0].point, Offset { x: 10.0, y: 10.0 });
        assert_eq!(path[1].point, Offset { x: 40.0, y: 20.0 });
        assert_eq!(path[2].control1, Offset { x: 50.0, y: 30.0 });
        assert_eq!(path[2].point, Offset { x: 70.0, y: 30.0 });
        assert!(path[3..]
            .iter()
            .all(|command| command.verb == PaintPathVerb::kCubicTo));
        assert!((path.last().unwrap().point.x - 90.0).abs() < 1e-5);
        assert!((path.last().unwrap().point.y - 30.0).abs() < 1e-5);
    }
}

//! Canonical computed shape() segments.
//! Source: core/style/style_shape.h:32-186; style_shape.cc:28-38.
use super::basic_shapes::{BasicShape, ShapeType};
use foundation::{gfx, Length, LengthPoint, LengthSize, Path, Traceable, Visitor, WindRule};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlPointOrigin {
    SegmentStart,
    SegmentEnd,
    ReferenceBox,
}
#[derive(Clone, PartialEq)]
pub struct ControlPoint {
    pub origin: ControlPointOrigin,
    pub point: LengthPoint,
}
#[derive(Clone, PartialEq)]
pub struct CurveSegment<const N: usize> {
    pub target_point: LengthPoint,
    pub control_points: [ControlPoint; N],
}
#[derive(Clone, PartialEq)]
pub struct ArcSegment {
    pub target_point: LengthPoint,
    pub angle: f64,
    pub radius: LengthSize,
    pub direction_agnostic_radius: Length,
    pub large: bool,
    pub sweep: bool,
}
#[derive(Clone, PartialEq)]
pub enum Segment {
    MoveTo(LengthPoint),
    MoveBy(LengthPoint),
    LineTo(LengthPoint),
    LineBy(LengthPoint),
    HLineTo(Length),
    HLineBy(Length),
    VLineTo(Length),
    VLineBy(Length),
    CubicCurveTo(CurveSegment<2>),
    CubicCurveBy(CurveSegment<2>),
    QuadraticCurveTo(CurveSegment<1>),
    QuadraticCurveBy(CurveSegment<1>),
    SmoothCubicCurveTo(CurveSegment<1>),
    SmoothCubicCurveBy(CurveSegment<1>),
    SmoothQuadraticCurveTo(LengthPoint),
    SmoothQuadraticCurveBy(LengthPoint),
    ArcTo(ArcSegment),
    ArcBy(ArcSegment),
    Close,
}
pub struct StyleShape {
    wind_rule: WindRule,
    origin: LengthPoint,
    segments: Vec<Segment>,
}
impl StyleShape {
    pub fn new(wind_rule: WindRule, origin: LengthPoint, segments: Vec<Segment>) -> Self {
        Self {
            wind_rule,
            origin,
            segments,
        }
    }
    pub fn GetWindRule(&self) -> WindRule {
        self.wind_rule
    }
    pub fn GetOrigin(&self) -> &LengthPoint {
        &self.origin
    }
    pub fn Segments(&self) -> &[Segment] {
        &self.segments
    }
}
impl BasicShape for StyleShape {
    fn GetType(&self) -> ShapeType {
        ShapeType::kStyleShapeType
    }
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool {
        debug_assert!(self.IsSameType(other));
        let other = unsafe { &*(other as *const dyn BasicShape as *const Self) };
        self.wind_rule == other.wind_rule
            && self.origin == other.origin
            && self.segments == other.segments
    }
    fn GetPath(&self, rect: &gfx::RectF, _zoom: f32, scale: f32) -> Path {
        let mut builder = foundation::shape_path::ShapePathBuilder::default();
        super::style_shape_path::BuildStyleShapePath(self, *rect, scale, &mut builder);
        builder.Finish()
    }
}
impl Traceable for StyleShape {
    fn Trace(&self, _: &mut Visitor<'_>) {}
}

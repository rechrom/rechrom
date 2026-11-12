//! Shared source compiled SVG path normalization. Paint and native Layout
//! consume this same algorithm; no authored string is parsed here.
use crate::{
    gfx::{PointF, RectF},
    WindRule,
};
pub trait NativePathSink {
    fn SetWindRule(&mut self, rule: WindRule);
    fn MoveTo(&mut self, point: PointF);
    fn LineTo(&mut self, point: PointF);
    fn QuadTo(&mut self, control: PointF, point: PointF);
    fn CubicTo(&mut self, control1: PointF, control2: PointF, point: PointF);
    fn ArcTo(&mut self, corner: PointF, next: PointF, radius: f32);
    fn ArcToSvg(&mut self, radii: PointF, angle: f32, large: bool, sweep: bool, point: PointF);
    fn Close(&mut self);
    fn Oval(&mut self, rect: RectF);
    /// Corner order matches FloatRoundedRect: TL, TR, BR, BL. The native
    /// rounded-rect primitive constrains overlapping radii once.
    fn RoundedRect(&mut self, rect: RectF, radii: [PointF; 4]);
}
struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let end = self
            .position
            .checked_add(N)
            .expect("SVG byte-stream offset");
        let value = self.bytes[self.position..end]
            .try_into()
            .expect("compiled SVG byte stream");
        self.position = end;
        value
    }
    fn float(&mut self) -> f32 {
        f32::from_ne_bytes(self.take())
    }
    fn point(&mut self) -> PointF {
        PointF::new(self.float(), self.float())
    }
}
/// Source SVGPathSegType and PathSegmentData. Both compiled SVG streams and
/// computed CSS shape() segments feed the same normalization owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum SvgPathCommand {
    Close = 1,
    MoveAbs,
    MoveRel,
    LineAbs,
    LineRel,
    CubicAbs,
    CubicRel,
    QuadAbs,
    QuadRel,
    ArcAbs,
    ArcRel,
    HLineAbs,
    HLineRel,
    VLineAbs,
    VLineRel,
    SmoothCubicAbs,
    SmoothCubicRel,
    SmoothQuadAbs,
    SmoothQuadRel,
}
impl SvgPathCommand {
    pub fn IsRelative(self) -> bool {
        matches!(
            self,
            Self::MoveRel
                | Self::LineRel
                | Self::CubicRel
                | Self::QuadRel
                | Self::ArcRel
                | Self::HLineRel
                | Self::VLineRel
                | Self::SmoothCubicRel
                | Self::SmoothQuadRel
        )
    }
    pub fn ToAbsolute(self) -> Self {
        match self {
            Self::MoveRel => Self::MoveAbs,
            Self::LineRel => Self::LineAbs,
            Self::CubicRel => Self::CubicAbs,
            Self::QuadRel => Self::QuadAbs,
            Self::ArcRel => Self::ArcAbs,
            Self::HLineRel => Self::HLineAbs,
            Self::VLineRel => Self::VLineAbs,
            Self::SmoothCubicRel => Self::SmoothCubicAbs,
            Self::SmoothQuadRel => Self::SmoothQuadAbs,
            other => other,
        }
    }
}
#[derive(Clone, Copy)]
pub struct SvgPathSegmentData {
    pub command: SvgPathCommand,
    pub target: PointF,
    pub point1: PointF,
    pub point2: PointF,
    pub arc_angle: f32,
    pub arc_large: bool,
    pub arc_sweep: bool,
}
impl SvgPathSegmentData {
    pub fn new(command: SvgPathCommand) -> Self {
        Self {
            command,
            target: PointF::default(),
            point1: PointF::default(),
            point2: PointF::default(),
            arc_angle: 0.,
            arc_large: false,
            arc_sweep: false,
        }
    }
}
/// Source SVGPathBuilder::EmitSegment. Keep current/subpath/control state once
/// and send native arc/conic primitives directly to the injected path sink.
pub struct NativeSvgPathBuilder<'a> {
    sink: &'a mut dyn NativePathSink,
    current: PointF,
    start: PointF,
    control: PointF,
    previous: Option<SvgPathCommand>,
    scale: f32,
    dx: f32,
    dy: f32,
}
impl<'a> NativeSvgPathBuilder<'a> {
    pub fn new(sink: &'a mut dyn NativePathSink, scale: f32, dx: f32, dy: f32) -> Self {
        Self {
            sink,
            current: PointF::default(),
            start: PointF::default(),
            control: PointF::default(),
            previous: None,
            scale,
            dx,
            dy,
        }
    }
    pub fn CurrentPoint(&self) -> PointF {
        self.current
    }
    fn map(&self, p: PointF) -> PointF {
        PointF::new(self.dx + p.x() * self.scale, self.dy + p.y() * self.scale)
    }
    pub fn EmitSegment(&mut self, data: SvgPathSegmentData) {
        use SvgPathCommand::*;
        let origin = if data.command.IsRelative() {
            self.current
        } else {
            PointF::default()
        };
        let add = |a: PointF, b: PointF| PointF::new(a.x() + b.x(), a.y() + b.y());
        let reflect = |p: PointF, c: PointF| PointF::new(2. * p.x() - c.x(), 2. * p.y() - c.y());
        match data.command {
            Close => {
                self.sink.Close();
                self.current = self.start;
            }
            MoveAbs | MoveRel => {
                self.current = add(origin, data.target);
                self.start = self.current;
                self.sink.MoveTo(self.map(self.current));
            }
            LineAbs | LineRel => {
                self.current = add(origin, data.target);
                self.sink.LineTo(self.map(self.current));
            }
            CubicAbs | CubicRel => {
                let a = add(origin, data.point1);
                let b = add(origin, data.point2);
                self.current = add(origin, data.target);
                self.sink
                    .CubicTo(self.map(a), self.map(b), self.map(self.current));
                self.control = b;
            }
            QuadAbs | QuadRel => {
                let a = add(origin, data.point1);
                self.current = add(origin, data.target);
                self.sink.QuadTo(self.map(a), self.map(self.current));
                self.control = a;
            }
            ArcAbs | ArcRel => {
                self.current = add(origin, data.target);
                self.sink.ArcToSvg(
                    PointF::new(data.point1.x() * self.scale, data.point1.y() * self.scale),
                    data.arc_angle,
                    data.arc_large,
                    data.arc_sweep,
                    self.map(self.current),
                );
            }
            HLineAbs | HLineRel => {
                self.current = PointF::new(origin.x() + data.target.x(), self.current.y());
                self.sink.LineTo(self.map(self.current));
            }
            VLineAbs | VLineRel => {
                self.current = PointF::new(self.current.x(), origin.y() + data.target.y());
                self.sink.LineTo(self.map(self.current));
            }
            SmoothCubicAbs | SmoothCubicRel => {
                let a = if self.previous.is_some_and(|p| {
                    matches!(p, CubicAbs | CubicRel | SmoothCubicAbs | SmoothCubicRel)
                }) {
                    reflect(self.current, self.control)
                } else {
                    self.current
                };
                let b = add(origin, data.point2);
                self.current = add(origin, data.target);
                self.sink
                    .CubicTo(self.map(a), self.map(b), self.map(self.current));
                self.control = b;
            }
            SmoothQuadAbs | SmoothQuadRel => {
                let a = if self
                    .previous
                    .is_some_and(|p| matches!(p, QuadAbs | QuadRel | SmoothQuadAbs | SmoothQuadRel))
                {
                    reflect(self.current, self.control)
                } else {
                    self.current
                };
                self.current = add(origin, data.target);
                self.sink.QuadTo(self.map(a), self.map(self.current));
                self.control = a;
            }
        }
        self.previous = Some(data.command);
    }
}
// Source: SVGPathByteStreamSource::ParseSegment / SVGPathBuilder::EmitSegment.
pub fn BuildSvgByteStream(
    bytes: &[u8],
    scale: f32,
    dx: f32,
    dy: f32,
    sink: &mut dyn NativePathSink,
) {
    use SvgPathCommand::*;
    let mut input = Reader { bytes, position: 0 };
    let mut builder = NativeSvgPathBuilder::new(sink, scale, dx, dy);
    while input.position < input.bytes.len() {
        let raw = u16::from_ne_bytes(input.take());
        let command = match raw {
            1 => Close,
            2 => MoveAbs,
            3 => MoveRel,
            4 => LineAbs,
            5 => LineRel,
            6 => CubicAbs,
            7 => CubicRel,
            8 => QuadAbs,
            9 => QuadRel,
            10 => ArcAbs,
            11 => ArcRel,
            12 => HLineAbs,
            13 => HLineRel,
            14 => VLineAbs,
            15 => VLineRel,
            16 => SmoothCubicAbs,
            17 => SmoothCubicRel,
            18 => SmoothQuadAbs,
            19 => SmoothQuadRel,
            _ => panic!("invalid compiled SVG path command {raw}"),
        };
        let mut data = SvgPathSegmentData::new(command);
        match command {
            Close => (),
            MoveAbs | MoveRel | LineAbs | LineRel | SmoothQuadAbs | SmoothQuadRel => {
                data.target = input.point()
            }
            CubicAbs | CubicRel => {
                data.point1 = input.point();
                data.point2 = input.point();
                data.target = input.point();
            }
            QuadAbs | QuadRel => {
                data.point1 = input.point();
                data.target = input.point();
            }
            ArcAbs | ArcRel => {
                data.point1 = input.point();
                data.arc_angle = input.float();
                data.arc_large = input.take::<1>()[0] != 0;
                data.arc_sweep = input.take::<1>()[0] != 0;
                data.target = input.point();
            }
            HLineAbs | HLineRel => data.target = PointF::new(input.float(), 0.),
            VLineAbs | VLineRel => data.target = PointF::new(0., input.float()),
            SmoothCubicAbs | SmoothCubicRel => {
                data.point2 = input.point();
                data.target = input.point();
            }
        }
        builder.EmitSegment(data);
    }
}

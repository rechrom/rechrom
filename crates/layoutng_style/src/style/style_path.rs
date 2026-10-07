//! StylePath's native byte-stream and wind-rule payload from
//! core/style/style_path.h:19-50 and style_path.cc:49-52.
use super::basic_shapes::{BasicShape, ShapeType};
use super::svg_path_byte_stream::SVGPathByteStream;
use foundation::{gfx, Path, Traceable, Visitor, WindRule};

pub struct StylePath {
    byte_stream: SVGPathByteStream,
    wind_rule: WindRule,
}

#[allow(non_snake_case)]
impl StylePath {
    pub fn new(byte_stream: SVGPathByteStream, wind_rule: WindRule) -> Self {
        Self {
            byte_stream,
            wind_rule,
        }
    }
    pub fn ByteStream(&self) -> &SVGPathByteStream {
        &self.byte_stream
    }
    pub fn GetWindRule(&self) -> WindRule {
        self.wind_rule
    }
}

#[allow(non_snake_case)]
impl BasicShape for StylePath {
    fn GetType(&self) -> ShapeType {
        ShapeType::kStylePathType
    }
    fn IsEqualAssumingSameType(&self, other: &dyn BasicShape) -> bool {
        debug_assert!(self.IsSameType(other));
        // Blink To<StylePath> downcasts after the matching shape type check.
        let other = unsafe { &*(other as *const dyn BasicShape as *const Self) };
        self.wind_rule == other.wind_rule && self.byte_stream == other.byte_stream
    }
    fn GetPath(&self, _rect: &gfx::RectF, _zoom: f32, _path_scale: f32) -> Path {
        // The owning Path package has not been linked: foundation::Path is
        // deliberately uninhabited. Native layout only needs the real shape
        // payload and HasClipPath here; it does not call this path builder.
        unimplemented!("StylePath::GetPath requires the owning vector Path package")
    }
}

impl Traceable for StylePath {
    fn Trace(&self, _visitor: &mut Visitor<'_>) {}
}

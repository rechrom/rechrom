//! Lossless lowering at the backend boundary. Curves stay native SkPath verbs;
//! rounded rectangles use SkRRect normalization, just as Chromium PathBuilder.
use paint_model::paint_clip_path::PaintPath;
use skia::src::core::SkPath::SkPath;

pub fn FromPaintPath(path: &PaintPath) -> SkPath {
    {
        use paint_model::paint_clip_path::{PaintPathFillType, PaintPathGeometry, PaintPathVerb};
        use skia::{
            include::core::SkPathTypes::{SkPathDirection, SkPathFillType},
            src::core::{
                SkPathBuilder::SkPathBuilder, SkPoint::SkPoint, SkRRect::SkRRect, SkRect::SkRect,
            },
        };
        if let PaintPathGeometry::RoundedRect { rect, radii } = path.geometry() {
            let rect = SkRect::MakeLTRB(
                rect.x as f32,
                rect.y as f32,
                (rect.x + rect.width) as f32,
                (rect.y + rect.height) as f32,
            );
            let radii = [
                radii.top_left,
                radii.top_right,
                radii.bottom_right,
                radii.bottom_left,
            ]
            .map(|radius| SkPoint {
                fX: radius.x as f32,
                fY: radius.y as f32,
            });
            return SkPath::RRectDefault(
                &SkRRect::MakeRectRadii(&rect, &radii),
                SkPathDirection::kCW,
            );
        }
        let fill = match path.fill_type() {
            PaintPathFillType::Winding => SkPathFillType::kWinding,
            PaintPathFillType::EvenOdd => SkPathFillType::kEvenOdd,
            PaintPathFillType::InverseWinding => SkPathFillType::kInverseWinding,
            PaintPathFillType::InverseEvenOdd => SkPathFillType::kInverseEvenOdd,
        };
        let mut result = SkPathBuilder::with_fill_type(fill);
        let PaintPathGeometry::Commands(commands) = path.geometry() else {
            unreachable!("rounded path returned above")
        };
        for command in commands.iter() {
            match command.verb {
                PaintPathVerb::kMoveTo => {
                    result.moveToXY(command.point.x as f32, command.point.y as f32);
                }
                PaintPathVerb::kLineTo => {
                    result.lineToXY(command.point.x as f32, command.point.y as f32);
                }
                PaintPathVerb::kQuadraticTo => {
                    result.quadToXY(
                        command.control1.x as f32,
                        command.control1.y as f32,
                        command.point.x as f32,
                        command.point.y as f32,
                    );
                }
                PaintPathVerb::kConicTo => {
                    result.conicToXY(
                        command.control1.x as f32,
                        command.control1.y as f32,
                        command.point.x as f32,
                        command.point.y as f32,
                        command.conic_weight as f32,
                    );
                }
                PaintPathVerb::kCubicTo => {
                    result.cubicToXY(
                        command.control1.x as f32,
                        command.control1.y as f32,
                        command.control2.x as f32,
                        command.control2.y as f32,
                        command.point.x as f32,
                        command.point.y as f32,
                    );
                }
                PaintPathVerb::kClose => {
                    result.close();
                }
            }
        }
        result.detach(None)
    }
}

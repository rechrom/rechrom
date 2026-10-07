//! Convert the supported standalone clip geometry to a native shape payload.
//! ShapeClipPathOperation is the official operation; native Style.HasClipPath
//! consequently participates in the normal PaintLayer creation rules.
#![allow(non_snake_case)]
use super::LengthPercentage;
use crate::internal::layout_input::{Offset, PaintPathVerb};
use crate::internal::paint_input::ClipPathPaint;
use foundation::{MakeGarbageCollected, WindRule};
use layoutng_style::style::basic_shapes::{BasicShape, BasicShapePolygon};
use layoutng_style::style::clip_path_operation::ClipPathOperation;
use layoutng_style::style::computed_style_constants::GeometryBox;
use layoutng_style::style::shape_clip_path_operation::ShapeClipPathOperation;
use layoutng_style::style::style_path::StylePath;
use layoutng_style::style::svg_path_byte_stream::SVGPathByteStream;

fn WritePoint(bytes: &mut Vec<u8>, point: Offset) {
    for value in [point.x, point.y] {
        assert!(
            value.is_finite() && (value as f32).is_finite(),
            "native clip-path coordinate must fit a finite float"
        );
        bytes.extend_from_slice(&(value as f32).to_ne_bytes());
    }
}

/// The input adapter has already normalized relative SVG segments and
/// converted supported arcs into cubic segments. Preserve that real geometry
/// using the official SVGPathByteStreamBuilder serialization format.
fn PathByteStream(input: &ClipPathPaint) -> SVGPathByteStream {
    let mut bytes = Vec::new();
    for command in &input.commands {
        // core/svg/svg_path_data.h:31-51; builder.cc:61-112.
        let segment: u16 = match command.verb {
            PaintPathVerb::kClose => 1,
            PaintPathVerb::kMoveTo => 2,
            PaintPathVerb::kLineTo => 4,
            PaintPathVerb::kCubicTo => 6,
            PaintPathVerb::kQuadraticTo => 8,
            PaintPathVerb::kConicTo => {
                unimplemented!("native SVGPathByteStream has no rational conic segment")
            }
        };
        bytes.extend_from_slice(&segment.to_ne_bytes());
        match command.verb {
            PaintPathVerb::kClose => {}
            PaintPathVerb::kCubicTo => {
                WritePoint(&mut bytes, command.control1);
                WritePoint(&mut bytes, command.control2);
                WritePoint(&mut bytes, command.point);
            }
            PaintPathVerb::kQuadraticTo => {
                WritePoint(&mut bytes, command.control1);
                WritePoint(&mut bytes, command.point);
            }
            PaintPathVerb::kMoveTo | PaintPathVerb::kLineTo => {
                WritePoint(&mut bytes, command.point)
            }
            PaintPathVerb::kConicTo => unreachable!(),
        }
    }
    SVGPathByteStream::from_data(bytes)
}

pub(super) fn NativeClipPath(input: &ClipPathPaint) -> *mut dyn ClipPathOperation {
    assert!(
        !input.commands.is_empty(),
        "native clip-path requires actual geometry"
    );
    let rule = if input.even_odd {
        WindRule::RULE_EVENODD
    } else {
        WindRule::RULE_NONZERO
    };
    let percentage_shape = !input.percentage_commands.is_empty();
    let shape: *mut dyn BasicShape = if percentage_shape {
        // ParseClipPath currently lowers both inset() and polygon() to the same
        // closed linear point payload. Preserve their px+percentage geometry
        // with a native BasicShapePolygon; their original CSS kind was already
        // discarded by the input adapter, before this boundary.
        assert_eq!(
            input.commands.len(),
            input.percentage_commands.len(),
            "clip-path percentage geometry must have matching segments"
        );
        assert!(
            input.commands.len() >= 4
                && input.commands[0].verb == PaintPathVerb::kMoveTo
                && input.commands.last().unwrap().verb == PaintPathVerb::kClose
                && input.commands[1..input.commands.len() - 1]
                    .iter()
                    .all(|command| command.verb == PaintPathVerb::kLineTo),
            "percentage clip-path must be one supported closed polygon"
        );
        let mut polygon = BasicShapePolygon::default();
        polygon.SetWindRule(rule);
        for (command, percentage) in input.commands.iter().zip(&input.percentage_commands) {
            assert_eq!(
                command.verb, percentage.verb,
                "clip-path percentage segments must match absolute geometry"
            );
            if command.verb == PaintPathVerb::kClose {
                continue;
            }
            let x = LengthPercentage(
                command.point.x,
                100.0 * percentage.point.x,
                "clip-path polygon x",
            );
            let y = LengthPercentage(
                command.point.y,
                100.0 * percentage.point.y,
                "clip-path polygon y",
            );
            polygon.AppendPoint(&x, &y);
        }
        MakeGarbageCollected(polygon)
    } else {
        MakeGarbageCollected(StylePath::new(PathByteStream(input), rule))
    };
    // Current standalone geometry resolves against the border box; geometry
    // box keywords and reference/url clip operations are not supported inputs.
    MakeGarbageCollected(unsafe { ShapeClipPathOperation::new(shape, GeometryBox::kBorderBox) })
}

use dom::svg_path_parser::ParseSVGPathDefault;
use layoutng_assembly::internal::layout_input::PaintPathVerb;
use skia::{
    include::core::SkPathTypes::SkPathDirection,
    src::core::{
        SkPathBuilder::{ArcSize, SkPathBuilder},
        SkPoint::SkPoint,
    },
};

fn from_dom(data: &str) -> SkPathBuilder {
    let parsed = ParseSVGPathDefault(data).expect("valid SVG path");
    let mut path = SkPathBuilder::default();
    for command in parsed.commands {
        match command.verb {
            PaintPathVerb::kMoveTo => {
                path.moveToXY(command.point.x as f32, command.point.y as f32);
            }
            PaintPathVerb::kLineTo => {
                path.lineToXY(command.point.x as f32, command.point.y as f32);
            }
            PaintPathVerb::kQuadraticTo => {
                path.quadToXY(
                    command.control1.x as f32,
                    command.control1.y as f32,
                    command.point.x as f32,
                    command.point.y as f32,
                );
            }
            PaintPathVerb::kConicTo => {
                path.conicToXY(
                    command.control1.x as f32,
                    command.control1.y as f32,
                    command.point.x as f32,
                    command.point.y as f32,
                    command.conic_weight as f32,
                );
            }
            PaintPathVerb::kCubicTo => {
                path.cubicToXY(
                    command.control1.x as f32,
                    command.control1.y as f32,
                    command.control2.x as f32,
                    command.control2.y as f32,
                    command.point.x as f32,
                    command.point.y as f32,
                );
            }
            PaintPathVerb::kClose => {
                path.close();
            }
        }
    }
    path
}

fn add(a: SkPoint, b: SkPoint) -> SkPoint {
    SkPoint::make(a.fX + b.fX, a.fY + b.fY)
}

fn reference_chevron() -> SkPathBuilder {
    let mut path = SkPathBuilder::default();
    let mut current = SkPoint::make(6.22_f32, 3.22_f32);
    path.moveTo(current);

    current = add(current, SkPoint::make(1.06_f32, 0.0));
    path.arcToSvg(
        SkPoint::make(0.75_f32, 0.75_f32),
        0.0,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        current,
    );
    current = add(current, SkPoint::make(4.25_f32, 4.25_f32));
    path.lineTo(current);
    current = add(current, SkPoint::make(0.0, 1.06_f32));
    path.arcToSvg(
        SkPoint::make(0.75_f32, 0.75_f32),
        0.0,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        current,
    );
    current = add(current, SkPoint::make(-4.25_f32, 4.25_f32));
    path.lineTo(current);
    current = add(current, SkPoint::make(-1.042_f32, -0.018_f32));
    path.arcToSvg(
        SkPoint::make(0.751_f32, 0.751_f32),
        0.0,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        current,
    );
    current = add(current, SkPoint::make(-0.018_f32, -1.042_f32));
    path.arcToSvg(
        SkPoint::make(0.751_f32, 0.751_f32),
        0.0,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        current,
    );
    // Blink's GenericParseNumber<float> accumulates the decimal digits in
    // float precision. `9.94` therefore lands one ULP above the result of a
    // directly rounded Rust/C++ float literal.
    current = SkPoint::make(f32::from_bits(0x411f_0a3e), 8.0);
    path.lineTo(current);
    current = SkPoint::make(6.22_f32, 4.28_f32);
    path.lineTo(current);
    current = add(current, SkPoint::make(0.0, -1.06_f32));
    path.arcToSvg(
        SkPoint::make(0.75_f32, 0.75_f32),
        0.0,
        ArcSize::kSmall_ArcSize,
        SkPathDirection::kCW,
        current,
    );
    path.close();
    path
}

fn bits(path: &SkPathBuilder) -> Vec<String> {
    let mut output = Vec::new();
    let mut iter = path.iter();
    while let Some(record) = iter.next() {
        let points = record
            .fPoints
            .iter()
            .map(|point| format!("{:08x},{:08x}", point.fX.to_bits(), point.fY.to_bits()))
            .collect::<Vec<_>>()
            .join(";");
        output.push(format!(
            "{:?} [{}] w={:08x}",
            record.fVerb,
            points,
            record.fConicWeight.to_bits()
        ));
    }
    output
}

#[test]
fn github_chevron_matches_skia_svg_arc_builder_bit_for_bit() {
    let data = "M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.751.751 0 0 1-1.042-.018a.751.751 0 0 1-.018-1.042L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z";
    assert_eq!(bits(&from_dom(data)), bits(&reference_chevron()));
}

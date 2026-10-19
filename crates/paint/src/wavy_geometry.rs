//! Selected native DecorationLinePainter geometry. These are actual SkPath
//! values shared by the wavy tile record and its raster/metadata consumers.
#![allow(non_snake_case)]
use skia::include::core::SkPath::{SkPath, SkPathFillType, SkPathPoint, SkPathVerb};
// Source: third_party/blink/renderer/core/paint/decoration_line_painter.h:24-31.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveDefinition {
    pub wavelength: f32,
    pub control_point_distance: f32,
    pub phase: f32,
}
// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:179-194.
pub fn MakeWave(thickness: f32) -> WaveDefinition {
    // std::max(1, NaN) preserves its first argument. Native round is ties away.
    let clamped_thickness = if 1. < thickness { thickness } else { 1. };
    let wavelength = 1. + 2. * (2. * clamped_thickness + 0.5).round();
    let cp_distance = 0.5 + (3. * clamped_thickness + 0.5).round();
    WaveDefinition {
        wavelength,
        control_point_distance: cp_distance,
        phase: -wavelength,
    }
}
// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:130-146.
pub fn WavyCenterlinePath(wave: &WaveDefinition, total_width: f32, start: SkPathPoint) -> SkPath {
    let mut x = start.x + wave.phase;
    let mut points = vec![SkPathPoint { x, y: start.y }];
    let mut verbs = vec![SkPathVerb::kMove];
    let end_x = start.x + total_width + wave.wavelength;
    while x < end_x {
        points.push(SkPathPoint {
            x: x + wave.wavelength * 0.5,
            y: start.y + wave.control_point_distance,
        });
        points.push(SkPathPoint {
            x: x + wave.wavelength * 0.5,
            y: start.y - wave.control_point_distance,
        });
        points.push(SkPathPoint {
            x: x + wave.wavelength,
            y: start.y,
        });
        verbs.push(SkPathVerb::kCubic);
        x += wave.wavelength;
    }
    SkPath::Raw(&points, &verbs, &[], SkPathFillType::kWinding, false)
}
// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:173-177.
pub fn WavyPath(wave: &WaveDefinition) -> SkPath {
    WavyCenterlinePath(wave, wave.wavelength, SkPathPoint { x: 0., y: 0.5 })
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:197-213.
pub fn ComputeWavyPatternRect(
    thickness: f32,
    wave: &WaveDefinition,
    path: &SkPath,
) -> Option<[f32; 4]> {
    // platform/geometry/path.cc:161-165 uses stroke precision 0.3f.
    let bounds = path.stroke_bounds(thickness, 0.3)?;
    let top = bounds.top.floor();
    let bottom = bounds.bottom.ceil();
    Some([0., top, wave.wavelength, bottom])
}

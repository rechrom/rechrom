//! Selected DecorationLinePainter geometry in Paint-owned path commands.
#![allow(non_snake_case)]
use layoutng_assembly::internal::layout_input::{Offset, PaintPathCommand, PaintPathVerb};
use paint_model::paint_clip_path::{PaintPath, PaintPathFillType};

pub use foundation::{MakeWave, WaveDefinition, WavyPaintRect, WavyPatternBounds};

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:130-146.
pub fn WavyCenterlinePath(wave: &WaveDefinition, total_width: f32, start: Offset) -> PaintPath {
    let mut x = start.x as f32 + wave.phase;
    let mut commands = vec![PaintPathCommand {
        verb: PaintPathVerb::kMoveTo,
        point: Offset {
            x: f64::from(x),
            y: start.y,
        },
        ..PaintPathCommand::default()
    }];
    let end_x = start.x as f32 + total_width + wave.wavelength;
    while x < end_x {
        commands.push(PaintPathCommand {
            verb: PaintPathVerb::kCubicTo,
            control1: Offset {
                x: f64::from(x + wave.wavelength * 0.5),
                y: start.y + f64::from(wave.control_point_distance),
            },
            control2: Offset {
                x: f64::from(x + wave.wavelength * 0.5),
                y: start.y - f64::from(wave.control_point_distance),
            },
            point: Offset {
                x: f64::from(x + wave.wavelength),
                y: start.y,
            },
            ..PaintPathCommand::default()
        });
        x += wave.wavelength;
    }
    PaintPath::from_commands(commands, PaintPathFillType::Winding)
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:173-177.
pub fn WavyPath(wave: &WaveDefinition) -> PaintPath {
    WavyCenterlinePath(wave, wave.wavelength, Offset { x: 0.0, y: 0.5 })
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:197-213.
// The actual stroked path bounds are computed by the selected Skia backend.
pub fn WavyPatternRect(wave: &WaveDefinition, stroke_bounds: [f32; 4]) -> [f32; 4] {
    [
        0.0,
        stroke_bounds[1].floor(),
        wave.wavelength,
        stroke_bounds[3].ceil(),
    ]
}

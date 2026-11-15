//! Backend-neutral text-decoration geometry shared by Layout overflow and Paint.

#![allow(non_camel_case_types, non_snake_case)]

use crate::gfx;

// Source: third_party/blink/renderer/platform/graphics/styled_stroke_data.h:43-49.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StrokeStyle {
    #[default]
    kSolidStroke,
    kDottedStroke,
    kDashedStroke,
    kDoubleStroke,
    kWavyStroke,
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.h:24-32.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WaveDefinition {
    pub wavelength: f32,
    pub control_point_distance: f32,
    pub phase: f32,
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:179-194.
pub fn MakeWave(thickness: f32) -> WaveDefinition {
    let clamped_thickness = if 1.0 < thickness { thickness } else { 1.0 };
    let wavelength = 1.0 + 2.0 * (2.0 * clamped_thickness + 0.5).round();
    let control_point_distance = 0.5 + (3.0 * clamped_thickness + 0.5).round();
    WaveDefinition {
        wavelength,
        control_point_distance,
        phase: -wavelength,
    }
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:200-212.
pub fn WavyPatternBounds(thickness: f32, wave: &WaveDefinition) -> [f32; 4] {
    let amplitude = wave.control_point_distance * 3.0_f32.sqrt() / 6.0;
    let half_stroke = thickness * 0.5;
    let top = (0.5 - amplitude - half_stroke).floor();
    let bottom = (0.5 + amplitude + half_stroke).ceil();
    [0.0, top, wave.wavelength, bottom]
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:281-289.
pub fn WavyPaintRect(
    line: &gfx::RectF,
    wave: &WaveDefinition,
    thickness: f32,
    wavy_offset: f32,
) -> gfx::RectF {
    let bounds = WavyPatternBounds(thickness, wave);
    gfx::RectF::new(
        gfx::PointF::new(line.x() + bounds[0], line.y() + bounds[1] + wavy_offset),
        gfx::SizeF::new(line.width(), bounds[3] - bounds[1]),
    )
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.h:34-57.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecorationGeometry {
    pub style: StrokeStyle,
    pub line: gfx::RectF,
    pub double_offset: f32,
    pub wavy_offset: f32,
    pub wavy_wave: WaveDefinition,
    pub antialias: bool,
}

impl Default for DecorationGeometry {
    fn default() -> Self {
        Self {
            style: StrokeStyle::kSolidStroke,
            line: gfx::RectF::default(),
            double_offset: 0.0,
            wavy_offset: 0.0,
            wavy_wave: WaveDefinition::default(),
            antialias: false,
        }
    }
}

impl DecorationGeometry {
    // Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:391-407.
    pub fn Make(
        style: StrokeStyle,
        line: &gfx::RectF,
        double_offset: f32,
        wavy_offset: f32,
        custom_wave: Option<&WaveDefinition>,
    ) -> Self {
        let mut geometry = Self {
            style,
            line: *line,
            double_offset,
            ..Self::default()
        };
        if style == StrokeStyle::kWavyStroke {
            geometry.wavy_wave = custom_wave
                .copied()
                .unwrap_or_else(|| MakeWave(geometry.Thickness()));
            geometry.wavy_offset = wavy_offset;
        }
        geometry
    }

    pub fn Thickness(&self) -> f32 {
        self.line.height()
    }
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:47-53.
fn GetSnappedPointsForTextLine(rect: &gfx::RectF) -> ((i32, i32), (i32, i32)) {
    let mid_y = (rect.y() + (rect.height() / 2.0).max(0.5)).floor() as i32;
    ((rect.x() as i32, mid_y), (rect.right() as i32, mid_y))
}

// Source: third_party/blink/renderer/core/paint/decoration_line_painter.cc:409-435.
pub fn DecorationLineBounds(geometry: &DecorationGeometry) -> gfx::RectF {
    match geometry.style {
        StrokeStyle::kDottedStroke | StrokeStyle::kDashedStroke => {
            let thickness = geometry.Thickness().round();
            let (start, end) = GetSnappedPointsForTextLine(&geometry.line);
            gfx::RectF::new(
                gfx::PointF::new(start.0 as f32, start.1 as f32 - thickness / 2.0),
                gfx::SizeF::new((end.0 - start.0) as f32, thickness),
            )
        }
        StrokeStyle::kWavyStroke => WavyPaintRect(
            &geometry.line,
            &geometry.wavy_wave,
            geometry.Thickness(),
            geometry.wavy_offset,
        ),
        StrokeStyle::kDoubleStroke => {
            let y = geometry.line.y() + geometry.double_offset.min(0.0);
            gfx::RectF::new(
                gfx::PointF::new(geometry.line.x(), y),
                gfx::SizeF::new(
                    geometry.line.width(),
                    geometry.line.height() + geometry.double_offset.abs(),
                ),
            )
        }
        StrokeStyle::kSolidStroke => geometry.line,
    }
}

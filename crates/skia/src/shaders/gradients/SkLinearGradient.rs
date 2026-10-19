//! Rust subset and compatibility adaptation of the matching official Skia file.
//! Migrated tiny-skia algorithms retain their original arithmetic and data layout.

// Migrated unchanged in behavior from tiny-skia-0.12.0/src/shaders/linear_gradient.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use crate::path::Scalar;

use crate::raster::{ColorSpace, GradientStop, Point, Shader, SpreadMode, Transform};

use crate::compat::gradient_interpolation::{GradientHueMethod, GradientInterpolationSpace};
use crate::raster::pipeline::RasterPipelineBuilder;
use crate::raster::shaders::gradient::{
    average_gradient_color_in_space, Gradient, DEGENERATE_THRESHOLD,
};

/// A linear gradient shader.
#[derive(Clone, PartialEq, Debug)]
pub struct SkLinearGradient {
    pub(crate) base: Gradient,
}

impl SkLinearGradient {
    /// Creates a new linear gradient shader.
    ///
    /// Returns `Shader::SolidColor` when:
    /// - `stops.len()` == 1
    /// - `start` and `end` are very close
    ///
    /// Returns `None` when:
    ///
    /// - `stops` is empty
    /// - `start` == `end`
    /// - `transform` is not invertible
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        start: Point,
        end: Point,
        stops: Vec<GradientStop>,
        mode: SpreadMode,
        transform: Transform,
    ) -> Option<Shader<'static>> {
        Self::new_with_interpolation(
            start,
            end,
            stops,
            mode,
            transform,
            GradientInterpolationSpace::None,
            GradientHueMethod::Shorter,
            false,
        )
    }

    pub(crate) fn new_with_interpolation(
        start: Point,
        end: Point,
        mut stops: Vec<GradientStop>,
        mode: SpreadMode,
        transform: Transform,
        interpolation_space: GradientInterpolationSpace,
        hue_method: GradientHueMethod,
        interpolate_premultiplied: bool,
    ) -> Option<Shader<'static>> {
        if stops.is_empty() {
            return None;
        }

        if stops.len() == 1 {
            if mode != SpreadMode::kDecal {
                return Some(Shader::SolidColor(stops[0].color));
            }
            let c = stops[0].color;
            stops = alloc::vec![GradientStop::new(0., c), GradientStop::new(1., c)];
        }

        let length = (end - start).length();
        if !length.is_finite() {
            return None;
        }

        if length.is_nearly_zero_within_tolerance(DEGENERATE_THRESHOLD) {
            // Degenerate gradient, the only tricky complication is when in clamp mode,
            // the limit of the gradient approaches two half planes of solid color
            // (first and last). However, they are divided by the line perpendicular
            // to the start and end point, which becomes undefined once start and end
            // are exactly the same, so just use the end color for a stable solution.

            // Except for special circumstances of clamped gradients,
            // every gradient shape (when degenerate) can be mapped to the same fallbacks.
            // The specific shape factories must account for special clamped conditions separately,
            // this will always return the last color for clamped gradients.
            match mode {
                SpreadMode::kDecal => {
                    return Some(Shader::SolidColor(crate::raster::Color::TRANSPARENT))
                }
                SpreadMode::Pad => {
                    // Depending on how the gradient shape degenerates,
                    // there may be a more specialized fallback representation
                    // for the factories to use, but this is a reasonable default.
                    return Some(Shader::SolidColor(stops.last().unwrap().color));
                }
                SpreadMode::Reflect | SpreadMode::Repeat => {
                    // repeat and mirror are treated the same: the border colors are never visible,
                    // but approximate the final color as infinite repetitions of the colors, so
                    // it can be represented as the average color of the gradient.
                    return Some(Shader::SolidColor(average_gradient_color_in_space(
                        &stops,
                        interpolation_space,
                        hue_method,
                        interpolate_premultiplied,
                    )));
                }
            }
        }

        transform.invert()?;

        let unit_ts = points_to_unit_ts(start, end)?;
        let mut base = Gradient::new(stops, mode, transform, unit_ts);
        base.set_interpolation(interpolation_space, hue_method, interpolate_premultiplied);
        Some(Shader::LinearGradient(SkLinearGradient { base }))
    }

    pub(crate) fn is_opaque(&self) -> bool {
        self.base.is_opaque()
    }

    pub(crate) fn appendStages(&self, cs: ColorSpace, p: &mut RasterPipelineBuilder) -> bool {
        self.base.appendStages(p, cs, &|_| {}, &|_| {})
    }
}

fn points_to_unit_ts(start: Point, end: Point) -> Option<Transform> {
    let mut vec = end - start;
    let mag = vec.length();
    let inv = if mag != 0.0 { mag.invert() } else { 0.0 };

    vec.scale(inv);

    let mut ts = ts_from_sin_cos_at(-vec.y, vec.x, start.x, start.y);
    ts = ts.post_translate(-start.x, -start.y);
    ts = ts.post_scale(inv, inv);
    Some(ts)
}

fn ts_from_sin_cos_at(sin: f32, cos: f32, px: f32, py: f32) -> Transform {
    let cos_inv = 1.0 - cos;
    Transform::from_row(
        cos,
        sin,
        -sin,
        cos,
        sdot(sin, py, cos_inv, px),
        sdot(-sin, px, cos_inv, py),
    )
}

fn sdot(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * b + c * d
}

/// Compatibility spelling retained for migrated callers; use `SkLinearGradient` in the mapped API.
pub use SkLinearGradient as LinearGradient;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::Color;

    fn solid(shader: Shader<'static>) -> Color {
        match shader {
            Shader::SolidColor(color) => color,
            _ => panic!("zero-length repeating gradient must become a solid average"),
        }
    }

    #[test]
    fn zero_length_repeat_averages_in_the_requested_space() {
        let stops = || {
            vec![
                GradientStop::new_extended(
                    0.0,
                    [1.0, 0.0, 0.0, 1.0],
                    Color::from_rgba8(255, 0, 0, 255),
                    GradientInterpolationSpace::Srgb,
                    0,
                ),
                GradientStop::new_extended(
                    1.0,
                    [0.0, 0.0, 1.0, 1.0],
                    Color::from_rgba8(0, 0, 255, 255),
                    GradientInterpolationSpace::Srgb,
                    0,
                ),
            ]
        };
        let srgb = solid(
            SkLinearGradient::new_with_interpolation(
                Point::from_xy(0.0, 0.0),
                Point::from_xy(0.0, 0.0),
                stops(),
                SpreadMode::Repeat,
                Transform::identity(),
                GradientInterpolationSpace::Srgb,
                GradientHueMethod::Shorter,
                true,
            )
            .unwrap(),
        );
        let oklab = solid(
            SkLinearGradient::new_with_interpolation(
                Point::from_xy(0.0, 0.0),
                Point::from_xy(0.0, 0.0),
                stops(),
                SpreadMode::Repeat,
                Transform::identity(),
                GradientInterpolationSpace::Oklab,
                GradientHueMethod::Shorter,
                true,
            )
            .unwrap(),
        );
        assert!(oklab.red() > srgb.red() && oklab.green() > srgb.green());
    }
}

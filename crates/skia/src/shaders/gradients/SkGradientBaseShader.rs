//! Rust subset and compatibility adaptation of the matching official Skia file.
//! Migrated tiny-skia algorithms retain their original arithmetic and data layout.

// Migrated unchanged in behavior from tiny-skia-0.12.0/src/shaders/gradient.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use alloc::vec::Vec;

use crate::path::{NormalizedF32, Scalar};

use crate::raster::{Color, ColorSpace, SpreadMode, Transform};

use crate::compat::gradient_interpolation::{
    finish, prepare_pair, GradientHueMethod, GradientInterpolationSpace,
};
use crate::raster::pipeline::RasterPipelineBuilder;
use crate::raster::pipeline::{self, EvenlySpaced2StopGradientCtx, GradientColor, GradientCtx};

// The default SCALAR_NEARLY_ZERO threshold of .0024 is too big and causes regressions for svg
// gradients defined in the wild.
pub const DEGENERATE_THRESHOLD: f32 = 1.0 / (1 << 15) as f32;

pub(crate) fn average_gradient_color_in_space(
    stops: &[GradientStop],
    space: GradientInterpolationSpace,
    hue: GradientHueMethod,
    interpolate_premultiplied: bool,
) -> Color {
    debug_assert!(stops.len() >= 2);
    if space == GradientInterpolationSpace::None {
        return average_gradient_color_srgb(stops, interpolate_premultiplied);
    }
    let rgba = |stop: &GradientStop| {
        [
            stop.source_color.r,
            stop.source_color.g,
            stop.source_color.b,
            stop.source_color.a,
        ]
    };
    let pair = |left: &GradientStop, right: &GradientStop| {
        prepare_pair(
            rgba(left),
            left.source_space,
            left.missing_components,
            rgba(right),
            right.source_space,
            right.missing_components,
            space,
            hue,
            interpolate_premultiplied,
        )
    };
    let mut average = [0.0; 4];
    for index in 0..stops.len() - 1 {
        let (left, right) = pair(&stops[index], &stops[index + 1]);
        let width = stops[index + 1].position.get() - stops[index].position.get();
        for channel in 0..4 {
            average[channel] += 0.5 * width * (left[channel] + right[channel]);
        }
    }
    if stops[0].position.get() > 0.0 {
        let (color, _) = pair(&stops[0], &stops[0]);
        for channel in 0..4 {
            average[channel] += stops[0].position.get() * color[channel];
        }
    }
    let last = stops.len() - 1;
    if stops[last].position.get() < 1.0 {
        let (_, color) = pair(&stops[last], &stops[last]);
        for channel in 0..4 {
            average[channel] += (1.0 - stops[last].position.get()) * color[channel];
        }
    }
    let result = finish(average, space, interpolate_premultiplied);
    Color::from_rgba(
        result[0].clamp(0.0, 1.0),
        result[1].clamp(0.0, 1.0),
        result[2].clamp(0.0, 1.0),
        result[3].clamp(0.0, 1.0),
    )
    .unwrap()
}

fn average_gradient_color_srgb(points: &[GradientStop], premultiplied: bool) -> Color {
    let mut blend = [0.0; 4];
    let rgba = |color: Color| {
        let alpha = color.alpha();
        let scale = if premultiplied { alpha } else { 1.0 };
        [
            color.red() * scale,
            color.green() * scale,
            color.blue() * scale,
            alpha,
        ]
    };
    for index in 0..points.len() - 1 {
        let left = rgba(points[index].color);
        let right = rgba(points[index + 1].color);
        let width = points[index + 1].position.get() - points[index].position.get();
        for channel in 0..4 {
            blend[channel] += 0.5 * width * (left[channel] + right[channel]);
        }
    }
    if points[0].position.get() > 0.0 {
        let color = rgba(points[0].color);
        for channel in 0..4 {
            blend[channel] += points[0].position.get() * color[channel];
        }
    }
    let last = points.len() - 1;
    if points[last].position.get() < 1.0 {
        let color = rgba(points[last].color);
        for channel in 0..4 {
            blend[channel] += (1.0 - points[last].position.get()) * color[channel];
        }
    }
    let alpha = blend[3];
    if premultiplied && alpha != 0.0 {
        for channel in &mut blend[..3] {
            *channel /= alpha;
        }
    }
    Color::from_rgba(blend[0], blend[1], blend[2], alpha).unwrap()
}

/// A gradient point.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct GradientStop {
    pub(crate) position: NormalizedF32,
    pub(crate) color: Color,
    source_color: GradientColor,
    source_space: GradientInterpolationSpace,
    missing_components: u8,
}

impl GradientStop {
    /// Creates a new gradient point.
    ///
    /// `position` will be clamped to a 0..=1 range.
    pub fn new(position: f32, color: Color) -> Self {
        GradientStop {
            position: NormalizedF32::new_clamped(position),
            color,
            source_color: GradientColor::from(color),
            source_space: GradientInterpolationSpace::None,
            missing_components: 0,
        }
    }

    pub(crate) fn new_extended(
        position: f32,
        color: [f32; 4],
        fallback: Color,
        source_space: GradientInterpolationSpace,
        missing_components: u8,
    ) -> Self {
        Self {
            position: NormalizedF32::new_clamped(position),
            color: fallback,
            source_color: GradientColor::new(color[0], color[1], color[2], color[3]),
            source_space,
            missing_components,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SkGradientBaseShader {
    stops: Vec<GradientStop>,
    tile_mode: SpreadMode,
    pub(crate) transform: Transform,
    points_to_unit: Transform,
    pub(crate) colors_are_opaque: bool,
    has_uniform_stops: bool,
    interpolation_space: GradientInterpolationSpace,
    hue_method: GradientHueMethod,
    interpolate_premultiplied: bool,
}

impl SkGradientBaseShader {
    pub fn new(
        mut stops: Vec<GradientStop>,
        tile_mode: SpreadMode,
        transform: Transform,
        points_to_unit: Transform,
    ) -> Self {
        debug_assert!(stops.len() > 1);

        // Note: we let the caller skip the first and/or last position.
        // i.e. pos[0] = 0.3, pos[1] = 0.7
        // In these cases, we insert dummy entries to ensure that the final data
        // will be bracketed by [0, 1].
        // i.e. our_pos[0] = 0, our_pos[1] = 0.3, our_pos[2] = 0.7, our_pos[3] = 1
        let dummy_first = stops[0].position.get() != 0.0;
        let dummy_last = stops[stops.len() - 1].position.get() != 1.0;

        // Now copy over the colors, adding the dummies as needed.
        if dummy_first {
            let mut stop = stops[0];
            stop.position = NormalizedF32::ZERO;
            stops.insert(0, stop);
        }

        if dummy_last {
            let mut stop = stops[stops.len() - 1];
            stop.position = NormalizedF32::ONE;
            stops.push(stop);
        }

        let colors_are_opaque = stops.iter().all(|p| p.color.is_opaque());

        // Pin the last value to 1.0, and make sure positions are monotonic.
        let start_index = if dummy_first { 0 } else { 1 };
        let mut prev = 0.0;
        let mut has_uniform_stops = true;
        let uniform_step = stops[start_index].position.get() - prev;
        for i in start_index..stops.len() {
            let curr = if i + 1 == stops.len() {
                // The last one must be zero.
                1.0
            } else {
                stops[i].position.get().bound(prev, 1.0)
            };

            has_uniform_stops &= uniform_step.is_nearly_equal(curr - prev);
            stops[i].position = NormalizedF32::new_clamped(curr);
            prev = curr;
        }

        SkGradientBaseShader {
            stops,
            tile_mode,
            transform,
            points_to_unit,
            colors_are_opaque,
            has_uniform_stops,
            interpolation_space: GradientInterpolationSpace::None,
            hue_method: GradientHueMethod::Shorter,
            interpolate_premultiplied: false,
        }
    }

    pub fn set_interpolation(
        &mut self,
        space: GradientInterpolationSpace,
        hue: GradientHueMethod,
        premultiplied: bool,
    ) {
        self.interpolation_space = space;
        self.hue_method = hue;
        self.interpolate_premultiplied = premultiplied;
    }

    fn prepared_pair(
        &self,
        left: &GradientStop,
        right: &GradientStop,
        cs: ColorSpace,
    ) -> (GradientColor, GradientColor) {
        if self.interpolation_space == GradientInterpolationSpace::None {
            let prepare = |stop: &GradientStop| {
                let mut color = GradientColor::from(cs.expand_color(stop.color));
                // Blink creates CSS gradient shaders with premultiplied-alpha
                // interpolation even when no explicit Color 4 space is named.
                // Keep the channels premultiplied through the gradient stage;
                // adding the ordinary post-gradient premultiply would square
                // alpha and darken every translucent interval.
                if self.interpolate_premultiplied {
                    color.r *= color.a;
                    color.g *= color.a;
                    color.b *= color.a;
                }
                color
            };
            return (prepare(left), prepare(right));
        }
        let (left, right) = prepare_pair(
            [
                left.source_color.r,
                left.source_color.g,
                left.source_color.b,
                left.source_color.a,
            ],
            left.source_space,
            left.missing_components,
            [
                right.source_color.r,
                right.source_color.g,
                right.source_color.b,
                right.source_color.a,
            ],
            right.source_space,
            right.missing_components,
            self.interpolation_space,
            self.hue_method,
            self.interpolate_premultiplied,
        );
        (
            GradientColor::new(left[0], left[1], left[2], left[3]),
            GradientColor::new(right[0], right[1], right[2], right[3]),
        )
    }

    pub fn appendStages(
        &self,
        p: &mut RasterPipelineBuilder,
        cs: ColorSpace,
        appendStages_pre: &dyn Fn(&mut RasterPipelineBuilder),
        appendStages_post: &dyn Fn(&mut RasterPipelineBuilder),
    ) -> bool {
        p.push(pipeline::Stage::SeedShader);

        let ts = match self.transform.invert() {
            Some(v) => v,
            None => {
                log::warn!("failed to invert a gradient transform. Nothing will be rendered");
                return false;
            }
        };
        let ts = ts.post_concat(self.points_to_unit);
        p.push_transform(ts);

        appendStages_pre(p);

        if self.interpolation_space != GradientInterpolationSpace::None {
            p.set_force_hq_pipeline(true);
            p.ctx.gradient_interpolation_space = self.interpolation_space;
            p.ctx.gradient_interpolate_premultiplied = self.interpolate_premultiplied;
        }

        match self.tile_mode {
            SpreadMode::Reflect => {
                p.push(pipeline::Stage::ReflectX1);
            }
            SpreadMode::Repeat => {
                p.push(pipeline::Stage::RepeatX1);
            }
            SpreadMode::Pad => {
                if self.has_uniform_stops {
                    // We clamp only when the stops are evenly spaced.
                    // If not, there may be hard stops, and clamping ruins hard stops at 0 and/or 1.
                    // In that case, we must make sure we're using the general "gradient" stage,
                    // which is the only stage that will correctly handle unclamped t.
                    p.push(pipeline::Stage::PadX1);
                }
            }
        }

        // The two-stop case with stops at 0 and 1.
        if self.stops.len() == 2 {
            debug_assert!(self.has_uniform_stops);

            let (c0, c1) = self.prepared_pair(&self.stops[0], &self.stops[1], cs);

            p.ctx.evenly_spaced_2_stop_gradient = EvenlySpaced2StopGradientCtx {
                factor: GradientColor::new(c1.r - c0.r, c1.g - c0.g, c1.b - c0.b, c1.a - c0.a),
                bias: c0,
            };

            p.push(pipeline::Stage::EvenlySpaced2StopGradient);
        } else {
            // Unlike Skia, we do not support the `evenly_spaced_gradient` stage.
            // In our case, there is no performance difference.

            let mut ctx = GradientCtx::default();

            // Note: In order to handle clamps in search, the search assumes
            // a stop conceptually placed at -inf.
            // Therefore, the max number of stops is `self.points.len()+1`.
            //
            // We also need at least 16 values for lowp pipeline.
            ctx.factors.reserve((self.stops.len() + 1).max(16));
            ctx.biases.reserve((self.stops.len() + 1).max(16));

            ctx.t_values.reserve(self.stops.len() + 1);

            // Remove the dummy stops inserted by SkGradientBaseShader::new
            // because they are naturally handled by the search method.
            let (first_stop, last_stop) = if self.stops.len() > 2 {
                let first = if self.stops[0].color != self.stops[1].color {
                    0
                } else {
                    1
                };

                let len = self.stops.len();
                let last = if self.stops[len - 2].color != self.stops[len - 1].color {
                    len - 1
                } else {
                    len - 2
                };
                (first, last)
            } else {
                (0, 1)
            };

            let mut t_l = self.stops[first_stop].position.get();
            let (first_color, mut last_color) =
                self.prepared_pair(&self.stops[first_stop], &self.stops[first_stop + 1], cs);
            ctx.push_const_color(first_color);
            ctx.t_values.push(NormalizedF32::ZERO);
            // N.B. lastStop is the index of the last stop, not one after.
            for i in first_stop..last_stop {
                let t_r = self.stops[i + 1].position.get();
                let (segment_l, c_r) = self.prepared_pair(&self.stops[i], &self.stops[i + 1], cs);
                debug_assert!(t_l <= t_r);
                if t_l < t_r {
                    // For each stop we calculate a bias B and a scale factor F, such that
                    // for any t between stops n and n+1, the color we want is B[n] + F[n]*t.
                    let f = GradientColor::new(
                        (c_r.r - segment_l.r) / (t_r - t_l),
                        (c_r.g - segment_l.g) / (t_r - t_l),
                        (c_r.b - segment_l.b) / (t_r - t_l),
                        (c_r.a - segment_l.a) / (t_r - t_l),
                    );
                    ctx.factors.push(f);

                    ctx.biases.push(GradientColor::new(
                        segment_l.r - f.r * t_l,
                        segment_l.g - f.g * t_l,
                        segment_l.b - f.b * t_l,
                        segment_l.a - f.a * t_l,
                    ));

                    ctx.t_values.push(NormalizedF32::new_clamped(t_l));
                }

                t_l = t_r;
                last_color = c_r;
            }

            ctx.push_const_color(last_color);
            ctx.t_values.push(NormalizedF32::new_clamped(t_l));

            ctx.len = ctx.factors.len();

            // All lists must have the same length.
            debug_assert_eq!(ctx.factors.len(), ctx.t_values.len());
            debug_assert_eq!(ctx.biases.len(), ctx.t_values.len());

            // Will with zeros until we have enough data to fit into F32x16.
            while ctx.factors.len() < 16 {
                ctx.factors.push(GradientColor::default());
                ctx.biases.push(GradientColor::default());
            }

            p.push(pipeline::Stage::Gradient);
            p.ctx.gradient = ctx;
        }

        if self.interpolation_space != GradientInterpolationSpace::None {
            p.push(pipeline::Stage::GradientColorToSrgb);
        }
        if !self.colors_are_opaque
            && (self.interpolation_space != GradientInterpolationSpace::None
                || !self.interpolate_premultiplied)
        {
            p.push(pipeline::Stage::Premultiply);
        }

        appendStages_post(p);

        true
    }

    pub fn apply_opacity(&mut self, opacity: f32) {
        for stop in &mut self.stops {
            stop.color.apply_opacity(opacity);
            stop.source_color.a *= opacity;
        }

        self.colors_are_opaque = self.stops.iter().all(|p| p.color.is_opaque());
    }
}

/// Compatibility spelling retained for migrated callers; use `SkGradientBaseShader` in the mapped API.
pub use SkGradientBaseShader as Gradient;

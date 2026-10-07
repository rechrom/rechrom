//! Rust subset and compatibility adaptation of the matching official Skia file.
//! Migrated tiny-skia algorithms retain their original arithmetic and data layout.

// Migrated unchanged in behavior from tiny-skia-0.12.0/src/shaders/mod.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

pub(crate) use crate::src::shaders::gradients::SkGradientBaseShader as gradient;
pub(crate) use crate::src::shaders::gradients::SkLinearGradient as linear_gradient;
pub(crate) use crate::src::shaders::gradients::SkRadialGradient as radial_gradient;
pub(crate) use crate::src::shaders::gradients::SkSweepGradient as sweep_gradient;
pub(crate) use crate::src::shaders::SkImageShader as pattern;

use crate::path::{NormalizedF32, Scalar};

pub use gradient::GradientStop;
pub use linear_gradient::LinearGradient;
pub use pattern::{FilterQuality, Pattern, PixmapPaint};
pub use radial_gradient::RadialGradient;
pub use sweep_gradient::SweepGradient;

use crate::raster::{Color, ColorSpace, Transform};

use crate::raster::pipeline::RasterPipelineBuilder;

pub use super::SkTileMode::{SkTileMode, SpreadMode};

/// A shader specifies the source color(s) for what is being drawn.
///
/// If a paint has no shader, then the paint's color is used. If the paint has a
/// shader, then the shader's color(s) are use instead, but they are
/// modulated by the paint's alpha. This makes it easy to create a shader
/// once (e.g. bitmap tiling or gradient) and then change its transparency
/// without having to modify the original shader. Only the paint's alpha needs
/// to be modified.
#[derive(Clone, PartialEq, Debug)]
pub enum SkShader<'a> {
    /// A solid color shader.
    SolidColor(Color),
    /// A linear gradient shader.
    LinearGradient(LinearGradient),
    /// A radial gradient shader.
    RadialGradient(RadialGradient),
    /// A sweep gradient shader.
    SweepGradient(SweepGradient),
    /// A pattern shader.
    Pattern(Pattern<'a>),
}

impl SkShader<'_> {
    /// Checks if the shader is guaranteed to produce only opaque colors.
    pub fn is_opaque(&self) -> bool {
        match self {
            SkShader::SolidColor(c) => c.is_opaque(),
            SkShader::LinearGradient(g) => g.is_opaque(),
            // A radial gradient may have points that are "undefined" so we just assume that it is
            // not opaque.
            SkShader::RadialGradient(_) => false,
            SkShader::SweepGradient(g) => g.is_opaque(),
            SkShader::Pattern(_) => false,
        }
    }

    // Unlike Skia, we do not have is_constant, because we don't have Color shaders.

    /// If this returns false, then we draw nothing (do not fall back to shader context)
    #[must_use]
    pub(crate) fn appendStages(&self, cs: ColorSpace, p: &mut RasterPipelineBuilder) -> bool {
        match self {
            SkShader::SolidColor(color) => {
                let color = cs.expand_color(*color).premultiply();
                p.push_uniform_color(color);
                true
            }
            SkShader::LinearGradient(g) => g.appendStages(cs, p),
            SkShader::RadialGradient(g) => g.appendStages(cs, p),
            SkShader::SweepGradient(g) => g.appendStages(cs, p),
            SkShader::Pattern(patt) => patt.appendStages(cs, p),
        }
    }

    /// Transforms the shader.
    pub fn transform(&mut self, ts: Transform) {
        match self {
            SkShader::SolidColor(_) => {}
            SkShader::LinearGradient(g) => {
                g.base.transform = g.base.transform.post_concat(ts);
            }
            SkShader::RadialGradient(g) => {
                g.base.transform = g.base.transform.post_concat(ts);
            }
            SkShader::SweepGradient(g) => {
                g.base.transform = g.base.transform.post_concat(ts);
            }
            SkShader::Pattern(p) => {
                p.transform = p.transform.post_concat(ts);
            }
        }
    }

    /// Shifts shader's opacity.
    ///
    /// `opacity` will be clamped to the 0..=1 range.
    ///
    /// This is roughly the same as Skia's `SkPaint::setAlpha`.
    ///
    /// Unlike Skia, we do not support global alpha/opacity, which is in Skia
    /// is set via the alpha channel of the `SkPaint::fColor4f`.
    /// Instead, you can shift the opacity of the shader to whatever value you need.
    ///
    /// - For `SolidColor` this function will multiply `color.alpha` by `opacity`.
    /// - For gradients this function will multiply all colors by `opacity`.
    /// - For `Pattern` this function will multiply `Patter::opacity` by `opacity`.
    pub fn apply_opacity(&mut self, opacity: f32) {
        match self {
            SkShader::SolidColor(ref mut c) => {
                c.apply_opacity(opacity);
            }
            SkShader::LinearGradient(g) => {
                g.base.apply_opacity(opacity);
            }
            SkShader::RadialGradient(g) => {
                g.base.apply_opacity(opacity);
            }
            SkShader::SweepGradient(g) => {
                g.base.apply_opacity(opacity);
            }
            SkShader::Pattern(ref mut p) => {
                p.opacity = NormalizedF32::new(p.opacity.get() * opacity.bound(0.0, 1.0)).unwrap();
            }
        }
    }
}

/// Compatibility spelling retained for migrated callers; use `SkShader` in the mapped API.
pub use SkShader as Shader;

//! Rust subset and compatibility adaptation of the matching official Skia file.
//! Migrated tiny-skia algorithms retain their original arithmetic and data layout.

// Migrated unchanged in behavior from tiny-skia-0.12.0/src/shaders/pattern.rs.

// Copyright 2006 The Android Open Source Project
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::path::NormalizedF32;

use crate::raster::{ColorSpace, PixmapRef, Shader, SpreadMode, Transform};

use crate::raster::pipeline;
use crate::raster::pipeline::RasterPipelineBuilder;

#[cfg(all(not(feature = "std"), feature = "no-std-float"))]
use crate::path::NoStdFloat;

pub use crate::extensions::ShaderCompatibility::{FilterQuality, PixmapPaint};

/// A pattern shader.
///
/// Essentially a `SkImageShader`.
///
/// Unlike Skia, we do not support FilterQuality::Medium, because it involves
/// mipmap generation, which adds too much complexity.
#[derive(Clone, PartialEq, Debug)]
pub struct SkImageShader<'a> {
    pub(crate) pixmap: PixmapRef<'a>,
    quality: FilterQuality,
    spread_mode: SpreadMode,
    pub(crate) opacity: NormalizedF32,
    pub(crate) transform: Transform,
}

impl<'a> SkImageShader<'a> {
    /// Creates a new pattern shader.
    ///
    /// `opacity` will be clamped to the 0..=1 range.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        pixmap: PixmapRef<'a>,
        spread_mode: SpreadMode,
        quality: FilterQuality,
        opacity: f32,
        transform: Transform,
    ) -> Shader<'a> {
        Shader::Pattern(SkImageShader {
            pixmap,
            spread_mode,
            quality,
            opacity: NormalizedF32::new_clamped(opacity),
            transform,
        })
    }

    pub(crate) fn appendStages(&self, cs: ColorSpace, p: &mut RasterPipelineBuilder) -> bool {
        let ts = match self.transform.invert() {
            Some(v) => v,
            None => {
                log::warn!("failed to invert a pattern transform. Nothing will be rendered");
                return false;
            }
        };

        p.push(pipeline::Stage::SeedShader);

        p.push_transform(ts);

        let mut quality = self.quality;

        if ts.is_identity() || ts.is_translate() {
            quality = FilterQuality::Nearest;
        }

        if quality == FilterQuality::Bilinear {
            if ts.is_translate() {
                if ts.tx == ts.tx.trunc() && ts.ty == ts.ty.trunc() {
                    // When the matrix is just an integer translate, bilerp == nearest neighbor.
                    quality = FilterQuality::Nearest;
                }
            }
        }

        // TODO: minimizing scale via mipmap

        match quality {
            FilterQuality::Nearest => {
                p.ctx.limit_x = pipeline::TileCtx {
                    scale: self.pixmap.width() as f32,
                    inv_scale: 1.0 / self.pixmap.width() as f32,
                };

                p.ctx.limit_y = pipeline::TileCtx {
                    scale: self.pixmap.height() as f32,
                    inv_scale: 1.0 / self.pixmap.height() as f32,
                };

                match self.spread_mode {
                    SpreadMode::Pad => { /* The gather() stage will clamp for us. */ }
                    SpreadMode::Repeat => p.push(pipeline::Stage::Repeat),
                    SpreadMode::Reflect => p.push(pipeline::Stage::Reflect),
                }

                p.push(pipeline::Stage::Gather);
            }
            FilterQuality::Bilinear => {
                p.ctx.sampler = pipeline::SamplerCtx {
                    spread_mode: self.spread_mode,
                    inv_width: 1.0 / self.pixmap.width() as f32,
                    inv_height: 1.0 / self.pixmap.height() as f32,
                };
                p.push(pipeline::Stage::Bilinear);
            }
            FilterQuality::Bicubic => {
                p.ctx.sampler = pipeline::SamplerCtx {
                    spread_mode: self.spread_mode,
                    inv_width: 1.0 / self.pixmap.width() as f32,
                    inv_height: 1.0 / self.pixmap.height() as f32,
                };
                p.push(pipeline::Stage::Bicubic);

                // Bicubic filtering naturally produces out of range values on both sides of [0,1].
                p.push(pipeline::Stage::Clamp0);
                p.push(pipeline::Stage::ClampA);
            }
        }

        // Unlike Skia, we do not support global opacity and only SkImageShader allows it.
        if self.opacity != NormalizedF32::ONE {
            debug_assert_eq!(
                core::mem::size_of_val(&self.opacity),
                4,
                "alpha must be f32"
            );
            p.ctx.current_coverage = self.opacity.get();
            p.push(pipeline::Stage::Scale1Float);
        }

        if let Some(stage) = cs.expand_stage() {
            // TODO: it would be better to gamma-expand prior to sampling, but
            // there isn't currently an intermediate pipeline stage for that.
            p.push(stage);
        }

        true
    }
}

/// Compatibility spelling retained for migrated callers; use `SkImageShader` in the mapped API.
pub use SkImageShader as Pattern;

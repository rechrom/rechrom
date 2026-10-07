//! Rust subset and compatibility adaptation of the matching official Skia file.
//! Migrated tiny-skia algorithms retain their original arithmetic and data layout.

// Migrated unchanged in behavior from tiny-skia-0.12.0/src/pipeline/blitter.rs.

// Copyright 2016 Google Inc.
// Copyright 2020 Yevhenii Reizner
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::raster::{BlendMode, Color, LengthU32, Paint, PixmapRef, PremultipliedColorU8, Shader};
use crate::raster::{ALPHA_U8_OPAQUE, ALPHA_U8_TRANSPARENT};

use crate::raster::alpha_runs::AlphaRun;
use crate::raster::blitter::{Blitter, Mask};
use crate::raster::color::AlphaU8;
use crate::raster::geom::ScreenIntRect;
use crate::raster::mask::SubMaskRef;
use crate::raster::math::LENGTH_U32_ONE;
use crate::raster::pipeline::{self, RasterPipeline, RasterPipelineBuilder};
use crate::raster::pixmap::SubPixmapMut;

pub struct SkRasterPipelineBlitter<'a, 'b: 'a> {
    mask: Option<SubMaskRef<'a>>,
    pixmap_src: PixmapRef<'a>,
    pixmap: &'a mut SubPixmapMut<'b>,
    memset2d_color: Option<PremultipliedColorU8>,
    blit_anti_h_rp: RasterPipeline,
    blit_rect_rp: RasterPipeline,
    blit_mask_rp: RasterPipeline,
    // Constant-color scanline blitters do not need an A8 shader program.
    // Retain its exact recipe and compile it only if blit_mask is requested.
    mask_pipeline_recipe: Option<(Paint<'a>, BlendMode)>,
    is_mask: bool,
    // SkBlitter::Choose selects a constant N32 blitter before constructing a
    // shader pipeline. This adapter retains that choice for SrcOver in the
    // existing byte color space. Other modes and gamma/highp use the pipeline.
    solid_color: Option<[u8; 4]>,
}

impl<'a, 'b: 'a> SkRasterPipelineBlitter<'a, 'b> {
    // Local vertical-mask adapter. The caller proves this shader's input,
    // clip coverage are invariant across the run. Destination colors are
    // checked below, so overlapping mask tiles still execute exact SrcOver.
    // Execute the existing H/AntiH pipeline at its actual first pixel, including
    // its original coverage stage, then repeat that exact packed result.
    pub(crate) fn blit_uniform_in_x_h(&mut self, x: u32, y: u32, width: LengthU32, alpha: u8) {
        if alpha == 0 {
            return;
        }
        let start = self.pixmap.offset(x as usize, y as usize) * 4;
        let end = start + width.get() as usize * 4;
        let mut position = start;
        while position < end {
            let destination: [u8; 4] = self.pixmap.data[position..position + 4].try_into().unwrap();
            let pixels = self.pixmap.data[position..end]
                .chunks_exact(4)
                .position(|pixel| pixel != destination)
                .unwrap_or((end - position) / 4);
            let length = core::num::NonZeroU32::new(pixels as u32).unwrap();
            self.blit_uniform_destination_h(x + ((position - start) / 4) as u32, y, length, alpha);
            position += pixels * 4;
        }
    }

    fn blit_uniform_destination_h(&mut self, x: u32, y: u32, width: LengthU32, alpha: u8) {
        if alpha == 255 {
            self.blit_h(x, y, LENGTH_U32_ONE);
        } else {
            let mut aa = [alpha, 0];
            let mut runs = [core::num::NonZeroU16::new(1), None];
            self.blit_anti_h(x, y, &mut aa, &mut runs);
        }
        let start = self.pixmap.offset(x as usize, y as usize) * 4;
        let source: [u8; 4] = self.pixmap.data[start..start + 4].try_into().unwrap();
        crate::src::core::SkBlitRow_D32::fill_solid(
            &mut self.pixmap.data[start..start + width.get() as usize * 4],
            source,
        );
    }

    pub fn new(
        paint: &Paint<'a>,
        mask: Option<SubMaskRef<'a>>,
        pixmap: &'a mut SubPixmapMut<'b>,
    ) -> Option<Self> {
        // Make sure that `mask` has the same size as `pixmap`.
        if let Some(mask) = mask {
            if mask.size.width() != pixmap.size.width()
                || mask.size.height() != pixmap.size.height()
            {
                log::warn!("Pixmap and Mask are expected to have the same size");
                return None;
            }
        }

        // Fast-reject.
        // This is basically SkInterpretXfermode().
        match paint.blend_mode {
            // `Destination` keep the pixmap unchanged. Nothing to do here.
            BlendMode::kDst => return None,
            BlendMode::kDstIn if paint.shader.is_opaque() && paint.is_solid_color() => return None,
            _ => {}
        }

        let solid_color = if paint.blend_mode == BlendMode::kSrcOver
            && !paint.force_hq_pipeline
            && paint.colorspace == crate::raster::ColorSpace::Linear
        {
            if let Shader::SolidColor(c) = &paint.shader {
                Some(crate::src::core::SkColor::premultiply(
                    crate::include::core::SkColor::SkColor4f::new(
                        c.red(),
                        c.green(),
                        c.blue(),
                        c.alpha(),
                    ),
                ))
            } else {
                None
            }
        } else {
            None
        };

        // We can strength-reduce SourceOver into Source when opaque.
        let mut blend_mode = paint.blend_mode;
        if paint.shader.is_opaque() && blend_mode == BlendMode::kSrcOver && mask.is_none() {
            blend_mode = BlendMode::kSrc;
        }

        // When we're drawing a constant color in Source mode, we can sometimes just memset.
        let mut memset2d_color = None;
        if paint.is_solid_color() && blend_mode == BlendMode::kSrc && mask.is_none() {
            // Unlike Skia, our shader cannot be constant.
            // Therefore there is no need to run a raster pipeline to get shader's color.
            if let Shader::SolidColor(ref color) = paint.shader {
                let c = color.premultiply().to_color_u8();
                let c = pixmap
                    .format
                    .encode([c.red(), c.green(), c.blue(), c.alpha()]);
                memset2d_color = Some(PremultipliedColorU8::from_rgba_unchecked(
                    c[0], c[1], c[2], c[3],
                ));
            }
        };

        // Clear is just a transparent color memset.
        if blend_mode == BlendMode::kClear && !paint.anti_alias && mask.is_none() {
            blend_mode = BlendMode::kSrc;
            memset2d_color = Some(PremultipliedColorU8::TRANSPARENT);
        }

        // SkBlitter::Choose's constant N32 path never executes the shader H /
        // AntiH / Rect pipelines. Keep mask fallback construction unchanged.
        let blit_anti_h_rp = if solid_color.is_some() {
            RasterPipelineBuilder::new().compile()
        } else {
            let mut p = RasterPipelineBuilder::new();
            p.set_force_hq_pipeline(paint.force_hq_pipeline);
            if !paint.shader.appendStages(paint.colorspace, &mut p) {
                return None;
            }

            if mask.is_some() {
                p.push(pipeline::Stage::MaskU8);
            }

            if blend_mode.should_pre_scale_coverage() {
                p.push(pipeline::Stage::Scale1Float);
                p.push(pipeline::Stage::LoadDestination);
                if let Some(stage) = paint.colorspace.expand_dest_stage() {
                    p.push(stage);
                }
                if let Some(blend_stage) = blend_mode.to_stage() {
                    p.push(blend_stage);
                }
            } else {
                p.push(pipeline::Stage::LoadDestination);
                if let Some(stage) = paint.colorspace.expand_dest_stage() {
                    p.push(stage);
                }
                if let Some(blend_stage) = blend_mode.to_stage() {
                    p.push(blend_stage);
                }

                p.push(pipeline::Stage::Lerp1Float);
            }

            if let Some(stage) = paint.colorspace.compress_stage() {
                p.push(stage);
            }
            p.push(pipeline::Stage::Store);

            p.compile()
        };

        let blit_rect_rp = if solid_color.is_some() {
            RasterPipelineBuilder::new().compile()
        } else {
            let mut p = RasterPipelineBuilder::new();
            p.set_force_hq_pipeline(paint.force_hq_pipeline);
            if !paint.shader.appendStages(paint.colorspace, &mut p) {
                return None;
            }

            if mask.is_some() {
                p.push(pipeline::Stage::MaskU8);
            }

            if blend_mode == BlendMode::kSrcOver && mask.is_none() {
                if let Some(stage) = paint.colorspace.compress_stage() {
                    p.push(stage);
                }
                // TODO: ignore when dither_rate is non-zero
                p.push(pipeline::Stage::SourceOverRgba);
            } else {
                if blend_mode != BlendMode::kSrc {
                    p.push(pipeline::Stage::LoadDestination);
                    if let Some(blend_stage) = blend_mode.to_stage() {
                        if let Some(stage) = paint.colorspace.expand_dest_stage() {
                            p.push(stage);
                        }
                        p.push(blend_stage);
                    }
                }

                if let Some(stage) = paint.colorspace.compress_stage() {
                    p.push(stage);
                }
                p.push(pipeline::Stage::Store);
            }

            p.compile()
        };

        let mask_pipeline_recipe = solid_color.map(|_| (paint.clone(), blend_mode));
        let blit_mask_rp = if mask_pipeline_recipe.is_some() {
            RasterPipelineBuilder::new().compile()
        } else {
            Self::make_mask_pipeline(paint, blend_mode, mask.is_some())?
        };

        let pixmap_src = match paint.shader {
            Shader::Pattern(ref patt) => patt.pixmap,
            // Just a dummy one.
            _ => PixmapRef::from_bytes(&[0, 0, 0, 0], 1, 1).unwrap(),
        };

        Some(SkRasterPipelineBlitter {
            mask,
            pixmap_src,
            pixmap,
            memset2d_color,
            blit_anti_h_rp,
            blit_rect_rp,
            blit_mask_rp,
            mask_pipeline_recipe,
            is_mask: false,
            solid_color,
        })
    }

    fn make_mask_pipeline(
        paint: &Paint<'a>,
        blend_mode: BlendMode,
        masked: bool,
    ) -> Option<RasterPipeline> {
        let mut p = RasterPipelineBuilder::new();
        p.set_force_hq_pipeline(paint.force_hq_pipeline);
        if !paint.shader.appendStages(paint.colorspace, &mut p) {
            return None;
        }

        if masked {
            p.push(pipeline::Stage::MaskU8);
        }

        if blend_mode.should_pre_scale_coverage() {
            p.push(pipeline::Stage::ScaleU8);
            p.push(pipeline::Stage::LoadDestination);
            if let Some(stage) = paint.colorspace.expand_dest_stage() {
                p.push(stage);
            }
            if let Some(blend_stage) = blend_mode.to_stage() {
                p.push(blend_stage);
            }
        } else {
            p.push(pipeline::Stage::LoadDestination);
            if let Some(stage) = paint.colorspace.expand_dest_stage() {
                p.push(stage);
            }
            if let Some(blend_stage) = blend_mode.to_stage() {
                p.push(blend_stage);
            }

            p.push(pipeline::Stage::LerpU8);
        }

        if let Some(stage) = paint.colorspace.compress_stage() {
            p.push(stage);
        }
        p.push(pipeline::Stage::Store);

        Some(p.compile())
    }

    pub fn new_mask(pixmap: &'a mut SubPixmapMut<'b>) -> Option<Self> {
        let color = Color::WHITE.premultiply();

        let memset2d_color = Some(color.to_color_u8());

        let blit_anti_h_rp = {
            let mut p = RasterPipelineBuilder::new();
            p.push_uniform_color(color);
            p.push(pipeline::Stage::LoadDestinationU8);
            p.push(pipeline::Stage::Lerp1Float);
            p.push(pipeline::Stage::StoreU8);
            p.compile()
        };

        let blit_rect_rp = {
            let mut p = RasterPipelineBuilder::new();
            p.push_uniform_color(color);
            p.push(pipeline::Stage::StoreU8);
            p.compile()
        };

        let blit_mask_rp = {
            let mut p = RasterPipelineBuilder::new();
            p.push_uniform_color(color);
            p.push(pipeline::Stage::LoadDestinationU8);
            p.push(pipeline::Stage::LerpU8);
            p.push(pipeline::Stage::StoreU8);
            p.compile()
        };

        Some(SkRasterPipelineBlitter {
            mask: None,
            pixmap_src: PixmapRef::from_bytes(&[0, 0, 0, 0], 1, 1).unwrap(),
            pixmap,
            memset2d_color,
            blit_anti_h_rp,
            blit_rect_rp,
            blit_mask_rp,
            mask_pipeline_recipe: None,
            is_mask: true,
            solid_color: None,
        })
    }
    // Local row/clip adapter for SkARGB32_[Opaque/Black_]Blitter::blitAntiH.
    // AA clip coverage is combined before the underlying blitter, as in
    // SkAAClipBlitterWrapper. Raster-pipeline mask and shader stages are not
    // re-entered for each short constant-color interval.
    fn blit_solid_run(&mut self, x: u32, y: u32, width: u32, alpha: u8) -> bool {
        let Some(source) = self.solid_color else {
            return false;
        };
        if alpha == 0 || source[3] == 0 {
            return true;
        }
        let offset = self.pixmap.offset(x as usize, y as usize) * 4;
        let dst = &mut self.pixmap.data[offset..offset + width as usize * 4];
        if let Some(mask) = self.mask {
            let mask_offset = (y * mask.real_width + x) as usize;
            let row = &mask.data[mask_offset..mask_offset + width as usize];
            let mut at = 0;
            while at < row.len() {
                let coverage = row[at];
                let mut end = at + 1;
                while end < row.len() && row[end] == coverage {
                    end += 1;
                }
                let coverage = crate::src::core::SkColor::mul_div_255_round(alpha, coverage);
                crate::src::core::SkBlitter_ARGB32::blend_anti_h_format(
                    &mut dst[at * 4..end * 4],
                    source,
                    coverage,
                    self.pixmap.format,
                );
                at = end;
            }
        } else {
            crate::src::core::SkBlitter_ARGB32::blend_anti_h_format(
                dst,
                source,
                alpha,
                self.pixmap.format,
            );
        }
        true
    }
}

impl Blitter for SkRasterPipelineBlitter<'_, '_> {
    fn blit_h(&mut self, x: u32, y: u32, width: LengthU32) {
        if !self.blit_solid_run(x, y, width.get(), 255) {
            let r = ScreenIntRect::from_xywh_safe(x, y, width, LENGTH_U32_ONE);
            self.blit_rect(&r);
        }
    }

    fn blit_anti_h(&mut self, mut x: u32, y: u32, aa: &mut [AlphaU8], runs: &mut [AlphaRun]) {
        let mask_ctx = self.mask.map(|c| c.mask_ctx()).unwrap_or_default();

        let mut aa_offset = 0;
        let mut run_offset = 0;
        let mut run_opt = runs[0];
        while let Some(run) = run_opt {
            let width = LengthU32::from(run);

            if self.blit_solid_run(x, y, width.get(), aa[aa_offset]) {
                x += width.get();
                run_offset += usize::from(run.get());
                aa_offset += usize::from(run.get());
                run_opt = runs[run_offset];
                continue;
            }
            match aa[aa_offset] {
                ALPHA_U8_TRANSPARENT => {}
                ALPHA_U8_OPAQUE => {
                    self.blit_h(x, y, width);
                }
                alpha => {
                    self.blit_anti_h_rp.ctx.current_coverage = alpha as f32 * (1.0 / 255.0);

                    let rect = ScreenIntRect::from_xywh_safe(x, y, width, LENGTH_U32_ONE);
                    self.blit_anti_h_rp.run(
                        &rect,
                        pipeline::AAMaskCtx::default(),
                        mask_ctx,
                        self.pixmap_src,
                        self.pixmap,
                    );
                }
            }

            x += width.get();
            run_offset += usize::from(run.get());
            aa_offset += usize::from(run.get());
            run_opt = runs[run_offset];
        }
    }

    fn blit_v(&mut self, x: u32, y: u32, height: LengthU32, alpha: AlphaU8) {
        let bounds = ScreenIntRect::from_xywh_safe(x, y, LENGTH_U32_ONE, height);

        let mask = Mask {
            image: [alpha, alpha],
            bounds,
            row_bytes: 0, // so we reuse the 1 "row" for all of height
        };

        self.blit_mask(&mask, &bounds);
    }

    fn blit_anti_h2(&mut self, x: u32, y: u32, alpha0: AlphaU8, alpha1: AlphaU8) {
        let bounds = ScreenIntRect::from_xywh(x, y, 2, 1).unwrap();

        let mask = Mask {
            image: [alpha0, alpha1],
            bounds,
            row_bytes: 2,
        };

        self.blit_mask(&mask, &bounds);
    }

    fn blit_anti_v2(&mut self, x: u32, y: u32, alpha0: AlphaU8, alpha1: AlphaU8) {
        let bounds = ScreenIntRect::from_xywh(x, y, 1, 2).unwrap();

        let mask = Mask {
            image: [alpha0, alpha1],
            bounds,
            row_bytes: 1,
        };

        self.blit_mask(&mask, &bounds);
    }

    fn blit_rect(&mut self, rect: &ScreenIntRect) {
        if self.solid_color.is_some() {
            for y in rect.y()..rect.bottom() {
                self.blit_solid_run(rect.x(), y, rect.width(), 255);
            }
            return;
        }
        if let Some(c) = self.memset2d_color {
            if self.is_mask {
                for y in 0..rect.height() {
                    let start = self
                        .pixmap
                        .offset(rect.x() as usize, (rect.y() + y) as usize);
                    let end = start + rect.width() as usize;
                    self.pixmap.data[start..end]
                        .iter_mut()
                        .for_each(|p| *p = c.alpha());
                }
            } else {
                for y in 0..rect.height() {
                    let start = self
                        .pixmap
                        .offset(rect.x() as usize, (rect.y() + y) as usize);
                    let end = start + rect.width() as usize;
                    self.pixmap.pixels_mut()[start..end]
                        .iter_mut()
                        .for_each(|p| *p = c);
                }
            }

            return;
        }

        let mask_ctx = self.mask.map(|c| c.mask_ctx()).unwrap_or_default();

        self.blit_rect_rp.run(
            rect,
            pipeline::AAMaskCtx::default(),
            mask_ctx,
            self.pixmap_src,
            self.pixmap,
        );
    }

    fn blit_mask(&mut self, mask: &Mask, clip: &ScreenIntRect) {
        if let Some((paint, mode)) = self.mask_pipeline_recipe.take() {
            self.blit_mask_rp = Self::make_mask_pipeline(&paint, mode, self.mask.is_some())
                .expect("constant shader mask pipeline");
        }
        let aa_mask_ctx = pipeline::AAMaskCtx {
            pixels: mask.image,
            stride: mask.row_bytes,
            shift: (mask.bounds.left() + mask.bounds.top() * mask.row_bytes) as usize,
        };

        let mask_ctx = self.mask.map(|c| c.mask_ctx()).unwrap_or_default();

        self.blit_mask_rp
            .run(clip, aa_mask_ctx, mask_ctx, self.pixmap_src, self.pixmap);
    }
}

/// Compatibility spelling retained for migrated callers; use `SkRasterPipelineBlitter` in the mapped API.
pub use SkRasterPipelineBlitter as RasterPipelineBlitter;

#[cfg(test)]
mod lazy_constant_pipeline_tests {
    use super::*;
    #[test]
    fn lazy_mask_fallback_preserves_eager_program_after_scanline_blits() {
        for alpha in [0.0, 0.125, 0.5, 1.0] {
            for clipped in [false, true] {
                let mut paint = Paint::default();
                paint.set_color_rgba8(37, 91, 129, (alpha * 255.0) as u8);
                let mut clip = crate::raster::Mask::new(16, 12).unwrap();
                for (i, a) in clip.data_mut().iter_mut().enumerate() {
                    *a = [0, 17, 127, 255][i % 4];
                }
                let render = |eager: bool| {
                    let mut pixels = crate::raster::Pixmap::new(16, 12).unwrap();
                    pixels.fill(Color::WHITE);
                    {
                        let mut view = pixels.as_mut();
                        let mut sub = view.as_subpixmap();
                        let mut b = SkRasterPipelineBlitter::new(
                            &paint,
                            clipped.then(|| clip.as_submask()),
                            &mut sub,
                        )
                        .unwrap();
                        if eager {
                            if let Some((p, mode)) = b.mask_pipeline_recipe.take() {
                                b.blit_mask_rp = SkRasterPipelineBlitter::make_mask_pipeline(
                                    &p,
                                    mode,
                                    b.mask.is_some(),
                                )
                                .unwrap();
                            }
                        }
                        b.blit_h(1, 1, LengthU32::new(7).unwrap());
                        b.blit_rect(&ScreenIntRect::from_xywh(3, 3, 7, 2).unwrap());
                        if !eager {
                            assert!(b.mask_pipeline_recipe.is_some());
                        }
                        b.blit_anti_h2(4, 6, 17, 231);
                        assert!(b.mask_pipeline_recipe.is_none());
                        b.blit_anti_v2(7, 7, 127, 255);
                    }
                    pixels.take()
                };
                assert_eq!(
                    render(false),
                    render(true),
                    "alpha={alpha},clipped={clipped}"
                );
            }
        }
    }
}

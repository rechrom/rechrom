//! Software execution of immutable Viz aggregate output.
//!
//! Rasterization and tile ownership live in `raster`; surface lifetime,
//! aggregation and swap throttling live in `viz`. `Renderer` retains only the
//! backend state needed to draw each surface into a borrowed output target.

use std::{collections::HashMap, io};

use raster::RasterResourceProvider;
use viz::{AggregatedFrame, RenderPassId, SurfaceId};

use crate::render_pass::RenderPassRenderer;

#[derive(Clone, Copy, Debug)]
pub struct RenderUpdate {
    pub mode: &'static str,
    pub reason: &'static str,
    pub damage_pixels: usize,
    pub copied_bytes: usize,
}

/// Borrowed output selected by the embedding display backend.
///
/// Renderer writes pixels but never owns, presents, resizes or otherwise
/// identifies the platform surface behind this memory.
pub struct RenderTarget<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a mut [u32],
    pub format: skia::PixelFormat,
    pub row_stride: usize,
}

#[derive(Default)]
pub struct Renderer {
    surfaces: HashMap<(SurfaceId, RenderPassId), RenderPassRenderer>,
}

/// Resolves the opaque resource namespace retained by each Viz surface.
/// Implementations borrow raster services; resource pixels never enter Viz.
pub trait RasterResourceRegistry {
    fn ResourcesFor(&self, surface_id: SurfaceId) -> Option<&dyn RasterResourceProvider>;
}

pub struct SingleSurfaceResources<'a> {
    pub surface_id: SurfaceId,
    pub resources: &'a dyn RasterResourceProvider,
}

impl RasterResourceRegistry for SingleSurfaceResources<'_> {
    fn ResourcesFor(&self, surface_id: SurfaceId) -> Option<&dyn RasterResourceProvider> {
        (surface_id == self.surface_id).then_some(self.resources)
    }
}

impl Renderer {
    pub fn Invalidate(&mut self) {
        for surface in self.surfaces.values_mut() {
            surface.invalidate();
        }
        self.surfaces.clear();
    }

    pub fn InvalidateSurface(&mut self, surface_id: SurfaceId) {
        self.surfaces
            .retain(|(surface, _), _| *surface != surface_id);
    }

    pub fn Render(
        &mut self,
        frame: &AggregatedFrame,
        resources: &dyn RasterResourceRegistry,
        target: RenderTarget<'_>,
    ) -> io::Result<RenderUpdate> {
        let RenderTarget {
            width,
            height,
            pixels: buffer,
            format,
            row_stride,
        } = target;
        let output = frame.OutputRect();
        let expected = row_stride
            .checked_mul(height as usize)
            .filter(|&length| {
                output.x == 0
                    && output.y == 0
                    && output.width == width
                    && output.height == height
                    && row_stride >= width as usize
                    && buffer.len() == length
            })
            .ok_or_else(|| io::Error::other("aggregated frame does not match render target"))?;
        debug_assert_eq!(expected, buffer.len());

        let covers_output = frame
            .RenderPasses()
            .iter()
            .filter(|pass| pass.IsRoot())
            .all(|pass| {
                let destination = pass.Destination();
                destination.x == 0 && destination.width == width
            })
            && {
                let mut rows: Vec<_> = frame
                    .RenderPasses()
                    .iter()
                    .filter(|pass| pass.IsRoot())
                    .map(|pass| {
                        let destination = pass.Destination();
                        (destination.y, destination.y + destination.height as i32)
                    })
                    .collect();
                rows.sort_unstable();
                let mut end = 0i32;
                rows.into_iter().all(|(top, bottom)| {
                    let contiguous = top <= end;
                    end = end.max(bottom);
                    contiguous
                }) && end >= height as i32
            };
        if !covers_output {
            buffer.fill(match format {
                skia::PixelFormat::Bgrx8888 => 0x00ff_ffff,
                skia::PixelFormat::Rgba8888 | skia::PixelFormat::Bgra8888 => u32::MAX,
            });
        }

        let mut damage_pixels = 0usize;
        let mut copied_bytes = 0usize;
        let mut all_retained = true;
        let mut pass_outputs: HashMap<(SurfaceId, RenderPassId), Vec<u32>> = HashMap::new();
        for pass in frame.RenderPasses() {
            let destination = pass.Destination();
            let provider = resources.ResourcesFor(pass.Surface()).ok_or_else(|| {
                io::Error::other("aggregated render pass has no raster resource provider")
            })?;
            if !pass.IsRoot() {
                let length = (destination.width as usize)
                    .checked_mul(destination.height as usize)
                    .ok_or_else(|| io::Error::other("render pass is too large"))?;
                let mut scratch = vec![0u32; length];
                let local_outputs: HashMap<_, _> = pass_outputs
                    .iter()
                    .filter_map(|(&(surface, id), pixels)| {
                        (surface == pass.Surface()).then_some((id, pixels.as_slice()))
                    })
                    .collect();
                let update = self
                    .surfaces
                    .entry((pass.Surface(), pass.RenderPassId()))
                    .or_default()
                    .compose(
                        pass,
                        provider,
                        destination.width,
                        destination.height,
                        &mut scratch,
                        skia::PixelFormat::Rgba8888,
                        destination.width as usize,
                        &local_outputs,
                    )?;
                damage_pixels = damage_pixels.saturating_add(update.damage_pixels);
                copied_bytes = copied_bytes.saturating_add(update.copied_bytes);
                all_retained &= update.damage_pixels == 0;
                pass_outputs.insert((pass.Surface(), pass.RenderPassId()), scratch);
                continue;
            }
            let local_outputs: HashMap<_, _> = pass_outputs
                .iter()
                .filter_map(|(&(surface, id), pixels)| {
                    (surface == pass.Surface()).then_some((id, pixels.as_slice()))
                })
                .collect();
            let update = if destination.x == 0 && destination.width == width {
                let top = usize::try_from(destination.y)
                    .map_err(|_| io::Error::other("negative surface destination"))?;
                let bottom = top
                    .checked_add(destination.height as usize)
                    .filter(|&bottom| bottom <= height as usize)
                    .ok_or_else(|| io::Error::other("surface destination exceeds target"))?;
                let target = &mut buffer[top * row_stride..bottom * row_stride];
                self.surfaces
                    .entry((pass.Surface(), pass.RenderPassId()))
                    .or_default()
                    .compose(
                        pass,
                        provider,
                        destination.width,
                        destination.height,
                        target,
                        format,
                        row_stride,
                        &local_outputs,
                    )?
            } else {
                let length = (destination.width as usize)
                    .checked_mul(destination.height as usize)
                    .ok_or_else(|| io::Error::other("surface destination is too large"))?;
                let mut scratch = vec![0u32; length];
                let update = self
                    .surfaces
                    .entry((pass.Surface(), pass.RenderPassId()))
                    .or_default()
                    .compose(
                        pass,
                        provider,
                        destination.width,
                        destination.height,
                        &mut scratch,
                        format,
                        destination.width as usize,
                        &local_outputs,
                    )?;
                let left = usize::try_from(destination.x)
                    .map_err(|_| io::Error::other("negative surface destination"))?;
                let top = usize::try_from(destination.y)
                    .map_err(|_| io::Error::other("negative surface destination"))?;
                for (row, source) in scratch.chunks_exact(destination.width as usize).enumerate() {
                    let start = (top + row) * row_stride + left;
                    buffer[start..start + destination.width as usize].copy_from_slice(source);
                }
                update
            };
            damage_pixels = damage_pixels.saturating_add(update.damage_pixels);
            copied_bytes = copied_bytes.saturating_add(update.copied_bytes);
            all_retained &= update.damage_pixels == 0;
        }
        Ok(RenderUpdate {
            mode: "aggregated-surfaces",
            reason: if all_retained {
                "retained-root-passes"
            } else {
                "surface-damage"
            },
            damage_pixels,
            copied_bytes,
        })
    }
}

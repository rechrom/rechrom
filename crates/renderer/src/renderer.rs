//! Software execution of immutable Viz aggregate output.
//!
//! Rasterization and tile ownership live in `raster`; surface lifetime,
//! aggregation and swap throttling live in `viz`. `Renderer` retains only the
//! backend state needed to draw each surface into a borrowed output target.

use std::{
    collections::{HashMap, HashSet},
    io,
};

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

/// Renderer-owned retained storage. Capacities exclude container overhead;
/// clip products are the cache's accounted bytes, not allocator residency.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackingStats {
    pub passes: usize,
    pub composed_capacity_bytes: usize,
    pub scratch_capacity_bytes: usize,
    pub clip_product_accounted_bytes: usize,
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
    pub fn BackingStats(&self) -> BackingStats {
        let (passes, composed_capacity_bytes, scratch_capacity_bytes, clip_product_accounted_bytes) =
            self.backing_usage();
        BackingStats {
            passes,
            composed_capacity_bytes,
            scratch_capacity_bytes,
            clip_product_accounted_bytes,
        }
    }
    fn backing_usage(&self) -> (usize, usize, usize, usize) {
        self.surfaces.values().fold((0, 0, 0, 0), |sum, pass| {
            let (composed, scratch, clips) = pass.backing_usage();
            (sum.0 + 1, sum.1 + composed, sum.2 + scratch, sum.3 + clips)
        })
    }

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

        // DirectRenderer::DecideRenderPassAllocations supplies the complete
        // current pass requirements to SoftwareRenderer before drawing. Viz
        // aggregates every pass, including undamaged retained children, for
        // each represented surface. Omitted surfaces have an independent
        // lifetime and remain owned by InvalidateSurface.
        let required: HashSet<_> = frame
            .RenderPasses()
            .iter()
            .map(|pass| (pass.Surface(), pass.RenderPassId()))
            .collect();
        let represented: HashSet<_> = required.iter().map(|key| key.0).collect();
        // Admission precedes reconciliation. A missing/mismatched provider
        // cannot discard backings from a candidate which will not render.
        for pass in frame.RenderPasses() {
            let provider = resources.ResourcesFor(pass.Surface()).ok_or_else(|| {
                io::Error::other("aggregated render pass has no raster resource provider")
            })?;
            if provider.prepared_frame_id() != Some(pass.SourceFrameId()) {
                return Err(io::Error::other(
                    "aggregated render-pass raster resources are not prepared",
                ));
            }
        }
        let before = self.backing_usage();
        self.surfaces
            .retain(|key, _| !represented.contains(&key.0) || required.contains(key));
        let reconciled = self.backing_usage();

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
        let mut completed_children = HashSet::new();
        let mut temporary_output_peak = 0usize;
        for pass in frame.RenderPasses() {
            let destination = pass.Destination();
            let provider = resources.ResourcesFor(pass.Surface()).ok_or_else(|| {
                io::Error::other("aggregated render pass has no raster resource provider")
            })?;
            let key = (pass.Surface(), pass.RenderPassId());
            // SoftwareRenderer samples each child directly from its retained
            // bitmap. Remove only the destination owner while borrowing the
            // already-completed child backings; no viewport staging copy is
            // needed, and children remain immutable throughout this draw.
            let mut renderer = self.surfaces.remove(&key).unwrap_or_default();
            let result = (|| {
                let local_outputs: HashMap<_, _> = completed_children
                    .iter()
                    .filter_map(|&(surface, id)| {
                        (surface == pass.Surface())
                            .then(|| self.surfaces.get(&(surface, id)))
                            .flatten()
                            .and_then(RenderPassRenderer::backing_pixels)
                            .map(|pixels| (id, pixels))
                    })
                    .collect();
                if !pass.IsRoot() {
                    return renderer.compose_retained(
                        pass,
                        provider,
                        destination.width,
                        destination.height,
                        &local_outputs,
                    );
                }
                if destination.x == 0 && destination.width == width {
                    let top = usize::try_from(destination.y)
                        .map_err(|_| io::Error::other("negative surface destination"))?;
                    let bottom = top
                        .checked_add(destination.height as usize)
                        .filter(|&bottom| bottom <= height as usize)
                        .ok_or_else(|| io::Error::other("surface destination exceeds target"))?;
                    let target = &mut buffer[top * row_stride..bottom * row_stride];
                    renderer.compose(
                        pass,
                        provider,
                        destination.width,
                        destination.height,
                        target,
                        format,
                        row_stride,
                        &local_outputs,
                    )
                } else {
                    let length = (destination.width as usize)
                        .checked_mul(destination.height as usize)
                        .ok_or_else(|| io::Error::other("surface destination is too large"))?;
                    let mut scratch = vec![0u32; length];
                    let update = renderer.compose(
                        pass,
                        provider,
                        destination.width,
                        destination.height,
                        &mut scratch,
                        format,
                        destination.width as usize,
                        &local_outputs,
                    )?;
                    temporary_output_peak =
                        temporary_output_peak.max(scratch.capacity() * std::mem::size_of::<u32>());
                    let left = usize::try_from(destination.x)
                        .map_err(|_| io::Error::other("negative surface destination"))?;
                    let top = usize::try_from(destination.y)
                        .map_err(|_| io::Error::other("negative surface destination"))?;
                    for (row, source) in
                        scratch.chunks_exact(destination.width as usize).enumerate()
                    {
                        let start = (top + row) * row_stride + left;
                        buffer[start..start + destination.width as usize].copy_from_slice(source);
                    }
                    Ok(update)
                }
            })();
            // Restore ownership before propagating any draw/validation error.
            // A failed backing remains invalid according to compose's normal
            // lifecycle; unrelated child and surface backings remain resident.
            self.surfaces.insert(key, renderer);
            let update = result?;
            if !pass.IsRoot() {
                completed_children.insert(key);
            }
            damage_pixels = damage_pixels.saturating_add(update.damage_pixels);
            copied_bytes = copied_bytes.saturating_add(update.copied_bytes);
            all_retained &= update.damage_pixels == 0;
        }
        let after = self.backing_usage();
        browser_tracing::instant(
            "render",
            "RenderPassBackingReconciled",
            &[
                ("current_surfaces", represented.len() as f64),
                ("current_passes", required.len() as f64),
                ("evicted_passes", (before.0 - reconciled.0) as f64),
                (
                    "evicted_capacity_bytes",
                    (before.1 + before.2 - reconciled.1 - reconciled.2) as f64,
                ),
            ],
        );
        browser_tracing::instant(
            "render",
            "RenderPassBackingUsage",
            &[
                ("retained_passes", self.surfaces.len() as f64),
                ("composed_capacity_bytes", after.1 as f64),
                ("effect_scratch_capacity_bytes", after.2 as f64),
                ("clip_product_accounted_bytes", after.3 as f64),
                (
                    "temporary_output_peak_capacity_bytes",
                    temporary_output_peak as f64,
                ),
            ],
        );
        if std::env::var_os("RECHROM_RENDERER_BACKING_DIAGNOSTICS").is_some() {
            eprintln!("renderer-backings surfaces={} current={} retained={} evicted={} evicted_capacity={} composed_capacity={} scratch_capacity={} clip_accounted={} temporary_peak={}",
                represented.len(), required.len(), self.surfaces.len(), before.0 - reconciled.0,
                before.1 + before.2 - reconciled.1 - reconciled.2, after.1, after.2, after.3, temporary_output_peak);
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

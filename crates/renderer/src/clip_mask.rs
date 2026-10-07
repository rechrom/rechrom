//! Synthesized compositor clip coverage. Chromium's SynthesizedClip paints
//! the actual RRect/path in local transform space; its mask effect is DstIn.
//! Bounds remain a conservative geometry guard, never a replacement shape.
use std::io;
use std::sync::Arc;

use crate::convert::ToSkia;
use layer_tile::CompositorScrollOffset;
use paint::paint_property_tree::{ClipPaintPropertyNode, TransformPaintPropertyNode};
use skia::compat::commands::{CommandKind, DrawCommand, PaintRect, ResourceContext};

#[path = "layer_aa_clip.rs"]
mod aa_clip;
use aa_clip::AAClipRuns;

// Physical device coordinates stay separate from cropped storage coordinates.
// The coverage bytes are the original Canvas result, including all AA values.
pub(crate) struct MaskCoverage {
    pub pixels: Arc<[u8]>,
    // The actual A8 run product shares this generation on integer relocation.
    pub run_clip: Arc<AAClipRuns>,
    pub origin: (usize, usize),
    pub stride: usize,
    pub rows: Vec<(usize, usize)>,
    pub bounds: Option<(usize, usize, usize, usize)>,
}
impl MaskCoverage {
    fn new(pixels: Vec<u8>, device: DeviceBounds) -> Self {
        let stride = device.right.saturating_sub(device.left);
        let mut bounds: Option<(usize, usize, usize, usize)> = None;
        let rows = if stride == 0 {
            Vec::new()
        } else {
            pixels
                .chunks_exact(stride)
                .enumerate()
                .map(|(y, row)| {
                    if let Some(first) = row.iter().position(|&v| v != 0) {
                        let last = row.iter().rposition(|&v| v != 0).expect("nonzero mask row") + 1;
                        let left = device.left + first;
                        let right = device.left + last;
                        let top = device.top + y;
                        bounds = Some(bounds.map_or((left, top, right, top + 1), |b| {
                            (
                                b.0.min(left),
                                b.1.min(top),
                                b.2.max(right),
                                b.3.max(top + 1),
                            )
                        }));
                        (left, right)
                    } else {
                        (0, 0)
                    }
                })
                .collect()
        };
        let run_clip = Arc::new(AAClipRuns::from_alpha(&pixels, stride));
        Self {
            pixels: pixels.into(),
            run_clip,
            origin: (device.left, device.top),
            stride,
            rows,
            bounds,
        }
    }
    fn len(&self) -> usize {
        self.pixels
            .len()
            .saturating_add(
                self.rows
                    .len()
                    .saturating_mul(std::mem::size_of::<(usize, usize)>()),
            )
            .saturating_add(self.run_clip.bytes())
    }
    fn relocated(&self, origin: (usize, usize)) -> Self {
        let dx = origin.0 as i64 - self.origin.0 as i64;
        let dy = origin.1 as i64 - self.origin.1 as i64;
        let rows = self
            .rows
            .iter()
            .map(|&(left, right)| {
                if left == right {
                    (0, 0)
                } else {
                    ((left as i64 + dx) as usize, (right as i64 + dx) as usize)
                }
            })
            .collect();
        let bounds = self.bounds.map(|b| {
            (
                (b.0 as i64 + dx) as usize,
                (b.1 as i64 + dy) as usize,
                (b.2 as i64 + dx) as usize,
                (b.3 as i64 + dy) as usize,
            )
        });
        Self {
            pixels: self.pixels.clone(),
            run_clip: self.run_clip.clone(),
            origin,
            stride: self.stride,
            rows,
            bounds,
        }
    }
}
#[derive(Clone, Copy)]
struct DeviceBounds {
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
}
impl DeviceBounds {
    fn viewport(width: u32, height: u32) -> Self {
        Self {
            left: 0,
            top: 0,
            right: width as usize,
            bottom: height as usize,
        }
    }
    fn empty(self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }
    fn intersect(&mut self, b: Self) {
        self.left = self.left.max(b.left);
        self.top = self.top.max(b.top);
        self.right = self.right.min(b.right);
        self.bottom = self.bottom.min(b.bottom);
    }
}
fn affine(matrix: &[f64; 16]) -> skia::Transform {
    skia::Transform::from_row(
        matrix[0] as f32,
        matrix[1] as f32,
        matrix[4] as f32,
        matrix[5] as f32,
        matrix[12] as f32,
        matrix[13] as f32,
    )
}
fn clip_transform(
    clip: &ClipPaintPropertyNode,
    scale: f64,
    scroll_override: Option<CompositorScrollOffset>,
) -> io::Result<skia::Transform> {
    let matrix = layer_tile::compositor_transform_with_scroll(
        &clip.local_transform_space,
        scale,
        scroll_override,
    )
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "non-affine compositor mask transform",
        )
    })?;
    Ok(affine(&matrix.values))
}
fn projected_support(
    rect: PaintRect,
    transform: skia::Transform,
    width: u32,
    height: u32,
) -> DeviceBounds {
    // Match Canvas's f32 rectangle/CTM domain. The guard encloses scan-converter
    // edge coverage; it does not become a substitute for the actual clip.
    let x = rect.x as f32;
    let y = rect.y as f32;
    let right = x + rect.width as f32;
    let bottom = y + rect.height as f32;
    let mut points = [
        skia::Point::from_xy(x, y),
        skia::Point::from_xy(right, y),
        skia::Point::from_xy(x, bottom),
        skia::Point::from_xy(right, bottom),
    ];
    transform.map_points(&mut points);
    if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return DeviceBounds::viewport(width, height);
    }
    let left = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) as f64;
    let top = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) as f64;
    let right = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max) as f64;
    let bottom = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max) as f64;
    DeviceBounds {
        left: (left.floor() - 2.0).clamp(0.0, width as f64) as usize,
        top: (top.floor() - 2.0).clamp(0.0, height as f64) as usize,
        right: (right.ceil() + 2.0).clamp(0.0, width as f64) as usize,
        bottom: (bottom.ceil() + 2.0).clamp(0.0, height as f64) as usize,
    }
}
fn install_transform(
    canvas: &mut skia::cpu::canvas::Canvas,
    matrix: skia::Transform,
    device: DeviceBounds,
    resources: &ResourceContext,
) {
    // The original complete CTM is formed before subtracting the integer
    // device offset. Raster-origin metadata preserves global shader phase.
    let mut command = DrawCommand {
        r#type: CommandKind::kConcat,
        ..Default::default()
    };
    command.transform.values[0] = matrix.sx as f64;
    command.transform.values[1] = matrix.ky as f64;
    command.transform.values[4] = matrix.kx as f64;
    command.transform.values[5] = matrix.sy as f64;
    command.transform.values[12] = (matrix.tx - device.left as f32) as f64;
    command.transform.values[13] = (matrix.ty - device.top as f32) as f64;
    canvas.set_scale(1.0);
    canvas.replay_item(&command, resources);
}

struct CacheKey {
    clips: Vec<Arc<ClipPaintPropertyNode>>,
    transforms: Vec<Arc<TransformPaintPropertyNode>>,
    width: u32,
    height: u32,
    scale: u64,
    scroll_override: Option<(u64, u64)>,
}
impl CacheKey {
    fn new(
        clips: &[Arc<ClipPaintPropertyNode>],
        width: u32,
        height: u32,
        scale: f64,
        scroll_override: Option<CompositorScrollOffset>,
    ) -> Self {
        let mut transforms: Vec<Arc<TransformPaintPropertyNode>> = Vec::new();
        for clip in clips {
            let mut node = Some(clip.local_transform_space.clone());
            while let Some(transform) = node {
                node = transform.parent.clone();
                if !transforms
                    .iter()
                    .any(|old| old.lifecycle.same_node(&transform.lifecycle))
                {
                    transforms.push(transform);
                }
            }
        }
        Self {
            clips: clips.to_vec(),
            transforms,
            width,
            height,
            scale: scale.to_bits(),
            scroll_override: scroll_override
                .map(|value| (value.scroll_node_id, value.translation_y.to_bits())),
        }
    }
    fn scroll_override(&self) -> Option<CompositorScrollOffset> {
        self.scroll_override
            .map(|(scroll_node_id, translation)| CompositorScrollOffset {
                scroll_node_id,
                translation_y: f64::from_bits(translation),
            })
    }
    fn matches(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.scale == other.scale
            && self.scroll_override == other.scroll_override
            && self.clips.len() == other.clips.len()
            && self.transforms.len() == other.transforms.len()
            && self.clips.iter().zip(&other.clips).all(|(a, b)| {
                a.lifecycle.same_node(&b.lifecycle) && a.lifecycle.revision == b.lifecycle.revision
            })
            && self.transforms.iter().zip(&other.transforms).all(|(a, b)| {
                a.lifecycle.same_node(&b.lifecycle) && a.lifecycle.revision == b.lifecycle.revision
            })
    }
}
struct Entry {
    key: CacheKey,
    coverage: Arc<MaskCoverage>,
    last_used: u64,
}
fn same_local_transform(
    a: skia::Transform,
    a_origin: (usize, usize),
    b: skia::Transform,
    b_origin: (usize, usize),
) -> bool {
    let local = |m: skia::Transform, origin: (usize, usize)| {
        [
            m.sx.to_bits(),
            m.ky.to_bits(),
            m.kx.to_bits(),
            m.sy.to_bits(),
            (m.tx - origin.0 as f32).to_bits(),
            (m.ty - origin.1 as f32).to_bits(),
        ]
    };
    let dx = b_origin.0 as f64 - a_origin.0 as f64;
    let dy = b_origin.1 as f64 - a_origin.1 as f64;
    // Compare exact coefficients in the actual f32 receiver domain, not an
    // epsilon or nominal scroll offset. Integer translation preserves AA phase.
    a.tx as f64 + dx == b.tx as f64
        && a.ty as f64 + dy == b.ty as f64
        && local(a, a_origin) == local(b, b_origin)
}
#[derive(Default)]
pub(crate) struct ClipMaskCache {
    entries: Vec<Entry>,
    bytes: usize,
    clock: u64,
    hits: u64,
    relocations: u64,
    rasters: u64,
    raster_pixels: u64,
}
impl ClipMaskCache {
    const BYTE_LIMIT: usize = 128 * 1024 * 1024;
    pub(super) fn observations(&self) -> [u64; 4] {
        [
            self.hits,
            self.relocations,
            self.rasters,
            self.raster_pixels,
        ]
    }
    pub(crate) fn coverage(
        &mut self,
        clips: &[Arc<ClipPaintPropertyNode>],
        width: u32,
        height: u32,
        scale: f64,
        scroll_override: Option<CompositorScrollOffset>,
    ) -> io::Result<Arc<MaskCoverage>> {
        let key = CacheKey::new(clips, width, height, scale, scroll_override);
        self.clock = self.clock.wrapping_add(1);
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.key.matches(&key))
        {
            entry.last_used = self.clock;
            self.hits += 1;
            return Ok(entry.coverage.clone());
        }
        // SynthesizedClip's A8 receiver has no global shader/dither anchor.
        // An unchanged rect/rrect chain whose complete CTMs all move by the
        // same integer device delta has identical local coverage. Reuse that
        // product as Chromium reuses the synthesized clip layer resource;
        // viewport truncation which changes storage dimensions still rerasterizes.
        if !clips.is_empty()
            && clips.iter().all(|clip| {
                clip.rect.is_some()
                    && clip.clip_path.is_empty()
                    && clip.pixel_moving_filter.is_none()
            })
        {
            let transforms = clips
                .iter()
                .map(|clip| clip_transform(clip, scale, scroll_override))
                .collect::<io::Result<Vec<_>>>()?;
            let mut device = DeviceBounds::viewport(width, height);
            for (clip, &matrix) in clips.iter().zip(&transforms) {
                device.intersect(projected_support(
                    clip.rect.expect("checked rect").to_skia(),
                    matrix,
                    width,
                    height,
                ));
            }
            if !device.empty() {
                let origin = (device.left, device.top);
                for entry in &mut self.entries {
                    if entry.key.width != width
                        || entry.key.height != height
                        || entry.key.scale != scale.to_bits()
                        || entry.key.clips.len() != clips.len()
                        || entry.coverage.stride != device.right - device.left
                        || entry.coverage.rows.len() != device.bottom - device.top
                    {
                        continue;
                    }
                    let mut same = true;
                    for ((old, new), &matrix) in entry.key.clips.iter().zip(clips).zip(&transforms)
                    {
                        if !old.lifecycle.same_node(&new.lifecycle)
                            || old.rect != new.rect
                            || old.radii != new.radii
                            || !old.clip_path.is_empty()
                            || old.pixel_moving_filter.is_some()
                            || !same_local_transform(
                                clip_transform(old, scale, entry.key.scroll_override())?,
                                entry.coverage.origin,
                                matrix,
                                origin,
                            )
                        {
                            same = false;
                            break;
                        }
                    }
                    if same {
                        let coverage = Arc::new(entry.coverage.relocated(origin));
                        entry.key = key;
                        entry.coverage = coverage.clone();
                        entry.last_used = self.clock;
                        self.relocations += 1;
                        return Ok(coverage);
                    }
                }
            }
        }
        let coverage = Arc::new(raster_coverage(
            clips,
            width,
            height,
            scale,
            scroll_override,
        )?);
        self.rasters += 1;
        self.raster_pixels += coverage.pixels.len() as u64;
        if coverage.len() <= Self::BYTE_LIMIT {
            while self.bytes > Self::BYTE_LIMIT - coverage.len() || self.entries.len() >= 1024 {
                let oldest = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, e)| e.last_used)
                    .map(|(i, _)| i)
                    .expect("nonempty over-budget clip cache");
                self.bytes -= self.entries.swap_remove(oldest).coverage.len();
            }
            self.bytes += coverage.len();
            self.entries.push(Entry {
                key,
                coverage: coverage.clone(),
                last_used: self.clock,
            });
        }
        Ok(coverage)
    }
}

fn raster_coverage(
    clips: &[Arc<ClipPaintPropertyNode>],
    width: u32,
    height: u32,
    scale: f64,
    scroll_override: Option<CompositorScrollOffset>,
) -> io::Result<MaskCoverage> {
    let mut device = DeviceBounds::viewport(width, height);
    let transforms = clips
        .iter()
        .map(|clip| clip_transform(clip, scale, scroll_override))
        .collect::<io::Result<Vec<_>>>()?;
    for (clip, &matrix) in clips.iter().zip(&transforms) {
        if let Some(rect) = clip.rect {
            device.intersect(projected_support(rect.to_skia(), matrix, width, height));
        }
    }
    if device.empty() {
        return Ok(MaskCoverage::new(Vec::new(), device));
    }
    let width = (device.right - device.left) as u32;
    let height = (device.bottom - device.top) as u32;
    let count = (width as usize)
        .checked_mul(height as usize)
        .filter(|&n| n != 0 && n <= isize::MAX as usize / 4)
        .ok_or_else(|| io::Error::other("invalid compositor mask dimensions"))?;
    let resources = ResourceContext::default();
    let mut canvas = skia::cpu::canvas::Canvas::make_raster_direct_with_format_preserving(
        &resources,
        width,
        height,
        width as usize * 4,
        skia::raster::PixelStorage::owned(vec![0; count * 4]),
        skia::PixelFormat::Rgba8888,
        false,
    )
    .ok_or_else(|| io::Error::other("invalid compositor mask device"))?;
    canvas.set_raster_origin((device.left as i32, device.top as i32));
    for (clip, &matrix) in clips.iter().zip(&transforms) {
        install_transform(&mut canvas, matrix, device, &resources);
        if let Some(rect) = clip.rect {
            canvas.replay_item(
                &DrawCommand {
                    r#type: if clip.radii == Default::default() {
                        CommandKind::kClipRect
                    } else {
                        CommandKind::kClipRoundedRect
                    },
                    // SwitchToClip uses AA for both rects and rounded rects;
                    // SkRasterClip keeps integral rectangular clips on its BW path.
                    rect: rect.to_skia(),
                    corner_radii: clip.radii.to_skia(),
                    antialias: true,
                    ..Default::default()
                },
                &resources,
            );
        }
        if !clip.clip_path.is_empty() {
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kClipPath,
                    path: clip.clip_path.to_skia(),
                    even_odd: clip.clip_path_even_odd,
                    antialias: true,
                    ..Default::default()
                },
                &resources,
            );
        }
    }
    // The completed raster clip already owns the exact A8 plane. Avoid a
    // second white draw through it and another RGBA-to-alpha extraction pass.
    Ok(MaskCoverage::new(canvas.finish_clip_alpha(), device))
}

//! Synthesized compositor clip coverage. Chromium's SynthesizedClip paints
//! the actual RRect/path in local transform space; its mask effect is DstIn.
//! Bounds remain a conservative geometry guard, never a replacement shape.
use std::io;
use std::sync::Arc;

use layer_tile::CompositorScrollOffset;
use layoutng_assembly::internal::layout_input::{PaintPathCommand, PaintPathVerb};
use paint::paint_property_tree::{ClipPaintPropertyNode, TransformPaintPropertyNode};
use raster::convert::ToSkia;
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

#[derive(Clone, Copy)]
enum CompleteClipTransform {
    Affine(skia::Transform),
    Projective(viz::ProjectiveTransform),
}

fn complete_clip_transform(
    clip: &ClipPaintPropertyNode,
    scale: f64,
    scroll_override: Option<CompositorScrollOffset>,
) -> io::Result<CompleteClipTransform> {
    let matrix = layer_tile::compositor_transform_3d_with_scroll(
        &clip.local_transform_space,
        scale,
        scroll_override,
    )
    .map_err(|_| {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "invalid compositor mask transform",
        )
    })?;
    let m = matrix.values;
    if m.iter().any(|value| !value.is_finite()) {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "non-finite compositor mask transform",
        ));
    }
    // A z=0 clip plane only needs its induced homography. A 3D matrix whose
    // homogeneous w is constant still has an exact affine target mapping even
    // when its unused z row/column is nontrivial (for example rotateX without
    // perspective), so retain the analytic Canvas path in that case.
    if m[3] == 0.0 && m[7] == 0.0 && m[15] > 0.0 {
        let inverse_w = 1.0 / m[15];
        return Ok(CompleteClipTransform::Affine(skia::Transform::from_row(
            (m[0] * inverse_w) as f32,
            (m[1] * inverse_w) as f32,
            (m[4] * inverse_w) as f32,
            (m[5] * inverse_w) as f32,
            (m[12] * inverse_w) as f32,
            (m[13] * inverse_w) as f32,
        )));
    }
    Ok(CompleteClipTransform::Projective(
        viz::ProjectiveTransform {
            h: [m[0], m[4], m[12], m[1], m[5], m[13], m[3], m[7], m[15]],
            z: [m[2], m[6], m[14]],
            backface_hidden: false,
            backface_visible: false,
        },
    ))
}
fn projected_affine_support(
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

fn projected_support(
    rect: PaintRect,
    transform: CompleteClipTransform,
    width: u32,
    height: u32,
) -> DeviceBounds {
    let CompleteClipTransform::Projective(projective) = transform else {
        let CompleteClipTransform::Affine(affine) = transform else {
            unreachable!()
        };
        return projected_affine_support(rect, affine, width, height);
    };
    let x = rect.x;
    let y = rect.y;
    let source = [
        (x, y),
        (x + rect.width, y),
        (x + rect.width, y + rect.height),
        (x, y + rect.height),
    ];
    const NEAR: f64 = 1.0e-6;
    let mut clipped = Vec::with_capacity(6);
    for index in 0..source.len() {
        let a = source[index];
        let b = source[(index + 1) % source.len()];
        let wa = projective.source_w(a.0, a.1);
        let wb = projective.source_w(b.0, b.1);
        let a_inside = wa >= NEAR;
        let b_inside = wb >= NEAR;
        if a_inside {
            clipped.push(a);
        }
        if a_inside != b_inside {
            let t = (NEAR - wa) / (wb - wa);
            clipped.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    let mapped: Vec<_> = clipped
        .into_iter()
        .filter_map(|point| projective.map_point(point.0, point.1))
        .collect();
    if mapped.is_empty() {
        return DeviceBounds {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
    }
    let left = mapped.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let top = mapped.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let right = mapped.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let bottom = mapped.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
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
    scroll_override: Option<(u64, u64, u64)>,
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
            scroll_override: scroll_override.map(|value| {
                (
                    value.scroll_node_id,
                    value.translation_x.to_bits(),
                    value.translation_y.to_bits(),
                )
            }),
        }
    }
    fn scroll_override(&self) -> Option<CompositorScrollOffset> {
        self.scroll_override
            .map(
                |(scroll_node_id, translation_x, translation_y)| CompositorScrollOffset {
                    scroll_node_id,
                    translation_x: f64::from_bits(translation_x),
                    translation_y: f64::from_bits(translation_y),
                },
            )
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
        if let Some(error) = clips
            .iter()
            .find_map(|clip| clip.clip_path_error.as_deref())
        {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("clip-path: {error}"),
            ));
        }
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
                    && layer_tile::compositor_transform_with_scroll(
                        &clip.local_transform_space,
                        scale,
                        scroll_override,
                    )
                    .is_ok()
            })
        {
            let transforms = clips
                .iter()
                .map(|clip| clip_transform(clip, scale, scroll_override))
                .collect::<io::Result<Vec<_>>>()?;
            let mut device = DeviceBounds::viewport(width, height);
            for (clip, &matrix) in clips.iter().zip(&transforms) {
                device.intersect(projected_affine_support(
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

type LocalPoint = (f64, f64);

struct FlatPath {
    contours: Vec<Vec<LocalPoint>>,
    even_odd: bool,
}

fn projected_curve_flat_enough(
    projective: viz::ProjectiveTransform,
    first: LocalPoint,
    interior: [LocalPoint; 3],
    last: LocalPoint,
) -> bool {
    let (Some(first), Some(last)) = (
        projective.map_point(first.0, first.1),
        projective.map_point(last.0, last.1),
    ) else {
        return false;
    };
    let [Some(quarter), Some(middle), Some(three_quarters)] =
        interior.map(|point| projective.map_point(point.0, point.1))
    else {
        return false;
    };
    let dx = last.0 - first.0;
    let dy = last.1 - first.1;
    let length_squared = dx * dx + dy * dy;
    let distance_squared = [quarter, middle, three_quarters]
        .into_iter()
        .map(|point| {
            if length_squared <= 1.0e-18 {
                (point.0 - first.0).powi(2) + (point.1 - first.1).powi(2)
            } else {
                let cross = dx * (point.1 - first.1) - dy * (point.0 - first.0);
                cross * cross / length_squared
            }
        })
        .fold(0.0, f64::max);
    // Keep projected curve error below one eighth of a device pixel. The A8
    // raster below samples a 4x4 grid, so tighter subdivision cannot alter its
    // coverage decision except at the same subpixel boundary.
    distance_squared <= 0.125 * 0.125
}

fn flatten_curve(
    projective: viz::ProjectiveTransform,
    evaluate: &impl Fn(f64) -> LocalPoint,
    t0: f64,
    first: LocalPoint,
    t1: f64,
    last: LocalPoint,
    depth: u8,
    output: &mut Vec<LocalPoint>,
) {
    let middle_t = (t0 + t1) * 0.5;
    let middle = evaluate(middle_t);
    let quarter = evaluate((t0 + middle_t) * 0.5);
    let three_quarters = evaluate((middle_t + t1) * 0.5);
    if depth >= 14
        || projected_curve_flat_enough(projective, first, [quarter, middle, three_quarters], last)
    {
        output.push(last);
        return;
    }
    flatten_curve(
        projective,
        evaluate,
        t0,
        first,
        middle_t,
        middle,
        depth + 1,
        output,
    );
    flatten_curve(
        projective,
        evaluate,
        middle_t,
        middle,
        t1,
        last,
        depth + 1,
        output,
    );
}

fn flat_path(
    commands: &[PaintPathCommand],
    even_odd: bool,
    projective: viz::ProjectiveTransform,
) -> FlatPath {
    let mut contours: Vec<Vec<LocalPoint>> = Vec::new();
    let mut current = (0.0, 0.0);
    let mut start = current;
    for command in commands {
        let end = (command.point.x, command.point.y);
        match command.verb {
            PaintPathVerb::kMoveTo => {
                contours.push(vec![end]);
                current = end;
                start = end;
            }
            PaintPathVerb::kLineTo => {
                if contours.is_empty() {
                    contours.push(vec![current]);
                    start = current;
                }
                contours.last_mut().unwrap().push(end);
                current = end;
            }
            PaintPathVerb::kQuadraticTo => {
                if contours.is_empty() {
                    contours.push(vec![current]);
                    start = current;
                }
                let first = current;
                let control = (command.control1.x, command.control1.y);
                let evaluate = |t: f64| {
                    let u = 1.0 - t;
                    (
                        u * u * first.0 + 2.0 * u * t * control.0 + t * t * end.0,
                        u * u * first.1 + 2.0 * u * t * control.1 + t * t * end.1,
                    )
                };
                flatten_curve(
                    projective,
                    &evaluate,
                    0.0,
                    first,
                    1.0,
                    end,
                    0,
                    contours.last_mut().unwrap(),
                );
                current = end;
            }
            PaintPathVerb::kCubicTo => {
                if contours.is_empty() {
                    contours.push(vec![current]);
                    start = current;
                }
                let first = current;
                let control1 = (command.control1.x, command.control1.y);
                let control2 = (command.control2.x, command.control2.y);
                let evaluate = |t: f64| {
                    let u = 1.0 - t;
                    (
                        u * u * u * first.0
                            + 3.0 * u * u * t * control1.0
                            + 3.0 * u * t * t * control2.0
                            + t * t * t * end.0,
                        u * u * u * first.1
                            + 3.0 * u * u * t * control1.1
                            + 3.0 * u * t * t * control2.1
                            + t * t * t * end.1,
                    )
                };
                flatten_curve(
                    projective,
                    &evaluate,
                    0.0,
                    first,
                    1.0,
                    end,
                    0,
                    contours.last_mut().unwrap(),
                );
                current = end;
            }
            PaintPathVerb::kConicTo => {
                if contours.is_empty() {
                    contours.push(vec![current]);
                    start = current;
                }
                let first = current;
                let control = (command.control1.x, command.control1.y);
                let weight = command.conic_weight;
                let evaluate = |t: f64| {
                    let u = 1.0 - t;
                    let denominator = u * u + 2.0 * weight * u * t + t * t;
                    (
                        (u * u * first.0 + 2.0 * weight * u * t * control.0 + t * t * end.0)
                            / denominator,
                        (u * u * first.1 + 2.0 * weight * u * t * control.1 + t * t * end.1)
                            / denominator,
                    )
                };
                flatten_curve(
                    projective,
                    &evaluate,
                    0.0,
                    first,
                    1.0,
                    end,
                    0,
                    contours.last_mut().unwrap(),
                );
                current = end;
            }
            PaintPathVerb::kClose => {
                if let Some(contour) = contours.last_mut() {
                    if contour.last().copied() != Some(start) {
                        contour.push(start);
                    }
                }
                current = start;
            }
        }
    }
    FlatPath { contours, even_odd }
}

fn path_contains(path: &FlatPath, point: LocalPoint) -> bool {
    let mut winding = 0i32;
    let mut parity = false;
    for contour in &path.contours {
        if contour.len() < 2 {
            continue;
        }
        for index in 0..contour.len() {
            let a = contour[index];
            let b = contour[(index + 1) % contour.len()];
            let upward = a.1 <= point.1 && b.1 > point.1;
            let downward = b.1 <= point.1 && a.1 > point.1;
            if !upward && !downward {
                continue;
            }
            let cross = (b.0 - a.0) * (point.1 - a.1) - (point.0 - a.0) * (b.1 - a.1);
            if (upward && cross > 0.0) || (downward && cross < 0.0) {
                parity = !parity;
                winding += if upward { 1 } else { -1 };
            }
        }
    }
    if path.even_odd {
        parity
    } else {
        winding != 0
    }
}

fn rounded_rect_contains(clip: &ClipPaintPropertyNode, point: LocalPoint) -> bool {
    let Some(rect) = clip.rect else { return true };
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    if point.0 < left || point.0 > right || point.1 < top || point.1 > bottom {
        return false;
    }
    // SkRRect scales all corners together when opposing radii exceed an edge.
    // Do the same before source-space containment so projective and affine
    // synthesized clips agree on oversized CSS radii.
    let mut radii = clip.radii;
    let mut radius_scale = 1.0f64;
    for (available, sum) in [
        (rect.width, radii.top_left.x + radii.top_right.x),
        (rect.width, radii.bottom_left.x + radii.bottom_right.x),
        (rect.height, radii.top_left.y + radii.bottom_left.y),
        (rect.height, radii.top_right.y + radii.bottom_right.y),
    ] {
        if sum > 0.0 {
            radius_scale = radius_scale.min(available / sum);
        }
    }
    if radius_scale < 1.0 {
        for corner in [
            &mut radii.top_left,
            &mut radii.top_right,
            &mut radii.bottom_right,
            &mut radii.bottom_left,
        ] {
            corner.x *= radius_scale;
            corner.y *= radius_scale;
        }
    }
    let corners = [
        (left, top, radii.top_left.x, radii.top_left.y, -1.0, -1.0),
        (right, top, radii.top_right.x, radii.top_right.y, 1.0, -1.0),
        (
            right,
            bottom,
            radii.bottom_right.x,
            radii.bottom_right.y,
            1.0,
            1.0,
        ),
        (
            left,
            bottom,
            radii.bottom_left.x,
            radii.bottom_left.y,
            -1.0,
            1.0,
        ),
    ];
    for (corner_x, corner_y, radius_x, radius_y, side_x, side_y) in corners {
        if radius_x <= 0.0 || radius_y <= 0.0 {
            continue;
        }
        let center_x = corner_x - side_x * radius_x;
        let center_y = corner_y - side_y * radius_y;
        if side_x * (point.0 - center_x) > 0.0 && side_y * (point.1 - center_y) > 0.0 {
            let dx = (point.0 - center_x) / radius_x;
            let dy = (point.1 - center_y) / radius_y;
            return dx * dx + dy * dy <= 1.0;
        }
    }
    true
}

fn raster_projective_clip(
    clip: &ClipPaintPropertyNode,
    projective: viz::ProjectiveTransform,
    path: Option<(&[PaintPathCommand], bool)>,
    device: DeviceBounds,
    width: u32,
    height: u32,
) -> io::Result<Vec<u8>> {
    let count = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| io::Error::other("invalid projective clip dimensions"))?;
    let flat = path.map(|(commands, even_odd)| flat_path(commands, even_odd, projective));
    const SAMPLE_AXIS: usize = 4;
    const SAMPLE_COUNT: usize = SAMPLE_AXIS * SAMPLE_AXIS;
    const NEAR: f64 = 1.0e-6;
    let mut output = vec![0u8; count];
    for local_y in 0..height as usize {
        for local_x in 0..width as usize {
            let mut inside = 0usize;
            for sample_y in 0..SAMPLE_AXIS {
                let target_y = device.top as f64
                    + local_y as f64
                    + (sample_y as f64 + 0.5) / SAMPLE_AXIS as f64;
                for sample_x in 0..SAMPLE_AXIS {
                    let target_x = device.left as f64
                        + local_x as f64
                        + (sample_x as f64 + 0.5) / SAMPLE_AXIS as f64;
                    let Some(source) = projective.inverse_point(target_x, target_y) else {
                        continue;
                    };
                    if projective.source_w(source.0, source.1) < NEAR {
                        continue;
                    }
                    let contains = flat.as_ref().map_or_else(
                        || rounded_rect_contains(clip, source),
                        |path| path_contains(path, source),
                    );
                    inside += usize::from(contains);
                }
            }
            output[local_y * width as usize + local_x] =
                ((inside * 255 + SAMPLE_COUNT / 2) / SAMPLE_COUNT) as u8;
        }
    }
    Ok(output)
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
        .map(|clip| complete_clip_transform(clip, scale, scroll_override))
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
    let raster_one = |clip: &ClipPaintPropertyNode,
                      matrix: CompleteClipTransform,
                      path: Option<(
        &[layoutng_assembly::internal::layout_input::PaintPathCommand],
        bool,
    )>|
     -> io::Result<Vec<u8>> {
        if let CompleteClipTransform::Projective(projective) = matrix {
            return raster_projective_clip(clip, projective, path, device, width, height);
        }
        let CompleteClipTransform::Affine(matrix) = matrix else {
            unreachable!()
        };
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
        install_transform(&mut canvas, matrix, device, &resources);
        if let Some((path, even_odd)) = path {
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kClipPath,
                    path: path.to_vec().to_skia(),
                    even_odd,
                    antialias: true,
                    ..Default::default()
                },
                &resources,
            );
        } else if let Some(rect) = clip.rect {
            canvas.replay_item(
                &DrawCommand {
                    r#type: if clip.radii == Default::default() {
                        CommandKind::kClipRect
                    } else {
                        CommandKind::kClipRoundedRect
                    },
                    rect: rect.to_skia(),
                    corner_radii: clip.radii.to_skia(),
                    antialias: true,
                    ..Default::default()
                },
                &resources,
            );
        }
        Ok(canvas.finish_clip_alpha())
    };
    let mut coverage = vec![255u8; count];
    for (clip, &matrix) in clips.iter().zip(&transforms) {
        // LayoutSVGResourceClipper unions the independently filled child
        // graphics, then intersects that product with the ancestor clip.  A
        // Source-over coverage union matches the mask fallback used when
        // LayoutSVGResourceClipper cannot reduce the child graphics to one
        // SkPath. It also avoids concatenating even-odd/nonzero contours into
        // one path with the wrong shared fill rule.
        let node = if !clip.clip_path_children.is_empty() {
            let mut union = vec![0u8; count];
            for child in &clip.clip_path_children {
                let child_alpha =
                    raster_one(clip, matrix, Some((&child.commands, child.even_odd)))?;
                for (dst, src) in union.iter_mut().zip(child_alpha) {
                    let inverse = 255u16 - *dst as u16;
                    *dst = (*dst as u16 + (src as u16 * inverse + 127) / 255) as u8;
                }
            }
            union
        } else if !clip.clip_path.is_empty() {
            raster_one(
                clip,
                matrix,
                Some((&clip.clip_path, clip.clip_path_even_odd)),
            )?
        } else {
            raster_one(clip, matrix, None)?
        };
        for (dst, src) in coverage.iter_mut().zip(node) {
            *dst = (((*dst as u16) * (src as u16) + 127) / 255) as u8;
        }
    }
    Ok(MaskCoverage::new(coverage, device))
}

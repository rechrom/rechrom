//! CPU execution of immutable compositor frames. `raster` owns premultiplied
//! tile pixels; this backend resolves them and writes the borrowed output.
use crate::clip_mask::{ClipMaskCache, MaskCoverage};
use layoutng_assembly::internal::paint_input::{
    PaintBlendMode, PaintCornerRadii, PaintFilterType, PaintReflectionDirection, SvgFilterGraph,
};
use paint::paint_property_tree::{ClipPaintPropertyNode, EffectPaintPropertyNode};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use viz::{AggregatedRenderPass, DrawQuad};

pub(crate) fn effect_opacity_has_output(opacity: f32) -> bool {
    // N32 effect restore rounds every premultiplied channel independently.
    // At and below this value even 255 rounds ties-to-even to zero.
    opacity > 0.5 / 255.0
}

use crate::renderer::RenderUpdate;
use layer_tile::{TileId, TilePlacement};
use paint::paint_engine::PaintRect;
use paint::paint_property_tree::PropertyTreeState;
use raster::convert::ToSkia;
use raster::{
    RasterResource as CachedTile, RasterResourceProvider, RasterRowSupport as TileRowSupport,
    RasterStats as LayerTileRasterStats, RowSupport,
};
use viz::LayerComposition;

use skia::cpu::layer_filters::{ColorMatrixKind, RenderSurfaceFilter as EffectFilter};

#[path = "layer_clip_quad.rs"]
mod layer_clip_quad;
#[cfg(feature = "compose_work_profile")]
#[path = "layer_compose_profile.rs"]
mod layer_compose_profile;
#[path = "layer_composition_preflight.rs"]
mod layer_composition_preflight;
#[path = "layer_direct_clip.rs"]
mod layer_direct_clip;
#[path = "layer_occlusion.rs"]
mod layer_occlusion;
#[path = "layer_solid.rs"]
pub(crate) mod layer_solid;

// Bounded stack storage for software sampling spans; larger tiles use the
// existing general path. Tile geometry itself comes exclusively from FramePlan.
// LayerTreeSettings uses 512px CPU tiles on high-DPI macOS and permits
// untiled CPU resources up to the same extent.  Scratch storage is bounded by
// that backend maximum, not by the 256px default used on other platforms.
const TILE_SIZE: u32 = 512;

#[cfg(test)]
fn sampled_opaque_device_rect_for_test(
    rect: PaintRect,
    scale: f64,
    translation: (f64, f64),
) -> io::Result<Option<viz::DeviceRect>> {
    if rect.is_empty() {
        return Ok(None);
    }
    let tx = translation.0.ceil();
    let ty = translation.1.ceil();
    let x = (rect.x * scale).ceil() + tx;
    let y = (rect.y * scale).ceil() + ty;
    let right = ((rect.x + rect.width) * scale).floor() + tx - f64::from(tx != translation.0);
    let bottom = ((rect.y + rect.height) * scale).floor() + ty - f64::from(ty != translation.1);
    if ![x, y, right, bottom]
        .into_iter()
        .all(|value| value.is_finite() && value.abs() <= i32::MAX as f64)
    {
        return Err(io::Error::other("invalid opaque bounds"));
    }
    Ok((right > x && bottom > y).then_some(viz::DeviceRect::new(
        x as i32,
        y as i32,
        (right - x) as u32,
        (bottom - y) as u32,
    )))
}

/// Borrow-free execution view of one immutable aggregated render pass. It
/// contains only quad state and tile placements; surface submission state,
/// PaintArtifact and RasterTask cannot cross the renderer boundary.
struct CompositionPlan {
    frame_id: u64,
    viewport: PaintRect,
    scale: f64,
    raster_task_count: usize,
    root_damage: Option<SurfaceBounds>,
    damage_region: Vec<SurfaceBounds>,
    backing_shift: Option<(i32, i32)>,
    output_bounds: Option<SurfaceBounds>,
    transparent_root: bool,
    owner_effect: Option<Arc<EffectPaintPropertyNode>>,
    layers: Vec<CompositionLayer>,
    solid_quads: Vec<viz::SolidColorDrawQuad>,
}

struct CompositionLayer {
    properties: PropertyTreeState,
    composition: LayerComposition,
    visible_polygon: Option<viz::TargetPolygon>,
    raster_content_bounds: Option<PaintRect>,
    retained_clips: Arc<[Arc<ClipPaintPropertyNode>]>,
    compositor_scroll: Option<layer_tile::CompositorScrollOffset>,
    render_pass: Option<viz::RenderPassId>,
    tiles: Vec<TilePlacement>,
}

impl CompositionLayer {
    fn has_drawable_content(&self) -> bool {
        !self.tiles.is_empty() || self.render_pass.is_some()
    }
}

impl CompositionPlan {
    fn from_pass(pass: &AggregatedRenderPass) -> io::Result<Self> {
        let mut tiles = vec![Vec::new(); pass.SharedQuadStates().len()];
        let mut render_passes = vec![None; pass.SharedQuadStates().len()];
        let mut solid_quads = Vec::new();
        for quad in pass.Quads() {
            match quad {
                DrawQuad::Tile(quad) => {
                    pass.SharedQuadStates()
                        .get(quad.shared_quad_state_index)
                        .ok_or_else(|| io::Error::other("draw quad has no shared state"))?;
                    let resource = pass
                        .Resources()
                        .iter()
                        .find(|resource| resource.id == quad.resource_id)
                        .ok_or_else(|| io::Error::other("draw quad resource is not listed"))?;
                    if resource.tile_id != quad.tile.tile_id
                        || resource.generation != quad.tile.generation
                    {
                        return Err(io::Error::other("draw quad identity changed"));
                    }
                    tiles[quad.shared_quad_state_index].push(quad.tile.clone());
                }
                DrawQuad::RenderPass(quad) => {
                    pass.SharedQuadStates()
                        .get(quad.shared_quad_state_index)
                        .ok_or_else(|| io::Error::other("render-pass quad has no shared state"))?;
                    let slot = &mut render_passes[quad.shared_quad_state_index];
                    if slot.replace(quad.render_pass_id).is_some()
                        || !tiles[quad.shared_quad_state_index].is_empty()
                    {
                        return Err(io::Error::other(
                            "shared state mixes tile and surface quads",
                        ));
                    }
                }
                DrawQuad::SolidColor(quad) => solid_quads.push(quad.clone()),
            }
        }
        let layers = pass
            .SharedQuadStates()
            .iter()
            .zip(tiles.into_iter().zip(render_passes))
            .map(|(state, (tiles, render_pass))| CompositionLayer {
                properties: state.properties.clone(),
                composition: state.composition,
                visible_polygon: state.visible_polygon.clone(),
                raster_content_bounds: state.raster_content_bounds,
                retained_clips: state.retained_clips.clone(),
                compositor_scroll: state.compositor_scroll,
                render_pass,
                tiles,
            })
            .collect();
        let damage_region = pass
            .DamageRects()
            .iter()
            .map(|rect| compositor_damage_bounds(*rect, pass.SurfaceViewport()))
            .collect::<io::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(Self {
            frame_id: pass.SourceFrameId(),
            viewport: pass.LogicalViewport(),
            scale: pass.RasterScaleFactor(),
            raster_task_count: pass.RasterTaskCount(),
            root_damage: compositor_damage_bounds(pass.DamageRect(), pass.SurfaceViewport())?,
            damage_region,
            backing_shift: pass.BackingShift(),
            output_bounds: compositor_damage_bounds(pass.Viewport(), pass.SurfaceViewport())?,
            transparent_root: !pass.IsRoot() || pass.HasTransparentBackground(),
            owner_effect: pass.OwnerEffect().cloned(),
            layers,
            solid_quads,
        })
    }
}

#[derive(Default)]
struct ComposeCosts {
    open: std::time::Duration,
    clip: std::time::Duration,
    mask_layers: usize,
    mask_raster_tasks: usize,
    mask_reused_tiles: usize,
    direct_mask_layers: usize,
    scratch: std::time::Duration,
    blit: std::time::Duration,
    close: std::time::Duration,
    scopes: usize,
    integral: usize,
    fractional: usize,
    fractional_phase_classes: [usize; 3],
    skipped_zero: usize,
    nominal_pixels: usize,
    occluded_tiles: usize,
    occluded_pixels: usize,
    tile_blits: [std::time::Duration; 5],
    tile_pixels: [usize; 5],
    solid_tiles: usize,
    direct_clip_layers: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SurfaceBounds {
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
}

fn shift_retained_backing(
    pixels: &mut [u32],
    width: usize,
    bounds: SurfaceBounds,
    delta: (i32, i32),
) {
    if delta == (0, 0) || bounds.left >= bounds.right || bounds.top >= bounds.bottom {
        return;
    }
    let dx = delta.0 as isize;
    let dy = delta.1 as isize;
    for row in 0..bounds.bottom - bounds.top {
        let y = if dy > 0 {
            bounds.bottom - 1 - row
        } else {
            bounds.top + row
        };
        for column in 0..bounds.right - bounds.left {
            let x = if dx > 0 {
                bounds.right - 1 - column
            } else {
                bounds.left + column
            };
            let source_x = x as isize - dx;
            let source_y = y as isize - dy;
            let destination = y * width + x;
            pixels[destination] = if source_x >= bounds.left as isize
                && source_x < bounds.right as isize
                && source_y >= bounds.top as isize
                && source_y < bounds.bottom as isize
            {
                pixels[source_y as usize * width + source_x as usize]
            } else {
                0
            };
        }
    }
}

// SoftwareRenderer::DrawRenderPassQuad draws dest_visible_rect, intersected
// with the target clip. The backing's allocation extent is not quad coverage.
fn render_pass_quad_bounds(
    composition: LayerComposition,
    width: u32,
    height: u32,
) -> Option<SurfaceBounds> {
    let (mut left, mut top, mut right, mut bottom) =
        (0i64, 0i64, i64::from(width), i64::from(height));
    for clip in [composition.clip, composition.content_bounds]
        .into_iter()
        .flatten()
    {
        left = left.max(i64::from(clip.x));
        top = top.max(i64::from(clip.y));
        right = right.min(i64::from(clip.x) + i64::from(clip.width));
        bottom = bottom.min(i64::from(clip.y) + i64::from(clip.height));
    }
    (left < right && top < bottom).then_some(SurfaceBounds {
        left: left as usize,
        top: top as usize,
        right: right as usize,
        bottom: bottom as usize,
    })
}

fn compositor_damage_bounds(
    damage: viz::DeviceRect,
    viewport: viz::DeviceRect,
) -> io::Result<Option<SurfaceBounds>> {
    if damage.width == 0 || damage.height == 0 {
        return Ok(None);
    }
    if damage.x < viewport.x || damage.y < viewport.y {
        return Err(io::Error::other(
            "aggregated render-pass damage is outside viewport",
        ));
    }
    let left = usize::try_from(damage.x - viewport.x)
        .map_err(|_| io::Error::other("invalid compositor damage"))?;
    let top = usize::try_from(damage.y - viewport.y)
        .map_err(|_| io::Error::other("invalid compositor damage"))?;
    let right = left
        .checked_add(damage.width as usize)
        .ok_or_else(|| io::Error::other("invalid compositor damage"))?;
    let bottom = top
        .checked_add(damage.height as usize)
        .ok_or_else(|| io::Error::other("invalid compositor damage"))?;
    if right > viewport.width as usize || bottom > viewport.height as usize {
        return Err(io::Error::other(
            "aggregated render-pass damage is outside viewport",
        ));
    }
    Ok(Some(SurfaceBounds {
        left,
        top,
        right,
        bottom,
    }))
}
// Pixel storage can cover a subrectangle, while paint/mask coordinates remain
// global. Never substitute the destination stride/origin for the source view.
#[derive(Clone, Copy)]
struct TargetView {
    origin: (usize, usize),
    stride: usize,
}
impl TargetView {
    fn row_start(self, y: usize, x: usize) -> usize {
        (y - self.origin.1) * self.stride + (x - self.origin.0)
    }
    fn row_range(self, y: usize, left: usize, right: usize) -> std::ops::Range<usize> {
        let start = self.row_start(y, left);
        start..start + right - left
    }
}
fn union_bounds(current: &mut Option<SurfaceBounds>, next: Option<SurfaceBounds>) {
    if let Some(b) = next {
        *current = Some(current.map_or(b, |a| SurfaceBounds {
            left: a.left.min(b.left),
            top: a.top.min(b.top),
            right: a.right.max(b.right),
            bottom: a.bottom.max(b.bottom),
        }));
    }
}
fn intersect_bounds(a: SurfaceBounds, b: SurfaceBounds) -> Option<SurfaceBounds> {
    let bounds = SurfaceBounds {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    };
    (bounds.left < bounds.right && bounds.top < bounds.bottom).then_some(bounds)
}

fn compose_solid_color_quad(
    buffer: &mut [u32],
    view: TargetView,
    format: skia::PixelFormat,
    quad: &viz::SolidColorDrawQuad,
    scale: f64,
    damage: SurfaceBounds,
) {
    let rect = quad.rect;
    let visible = quad.visible_rect;
    if rect.is_empty() || visible.is_empty() || quad.color[3] == 0 {
        return;
    }
    let left = (rect.x * scale).floor().max(0.0);
    let top = (rect.y * scale).floor().max(0.0);
    let right = ((rect.x + rect.width) * scale).ceil();
    let bottom = ((rect.y + rect.height) * scale).ceil();
    let visible_left = (visible.x * scale).floor().max(0.0);
    let visible_top = (visible.y * scale).floor().max(0.0);
    let visible_right = ((visible.x + visible.width) * scale).ceil();
    let visible_bottom = ((visible.y + visible.height) * scale).ceil();
    let bounds = SurfaceBounds {
        left: left.max(visible_left) as usize,
        top: top.max(visible_top) as usize,
        right: right.min(visible_right).max(0.0) as usize,
        bottom: bottom.min(visible_bottom).max(0.0) as usize,
    };
    let Some(bounds) = intersect_bounds(bounds, damage) else {
        return;
    };
    let radius = (quad.corner_radius * scale)
        .max(0.0)
        .min((right - left).min(bottom - top) * 0.5);
    let center_x = (left + right) * 0.5;
    let cap_top = top + radius;
    let cap_bottom = bottom - radius;
    for y in bounds.top..bounds.bottom {
        let py = y as f64 + 0.5;
        let nearest_y = py.clamp(cap_top, cap_bottom.max(cap_top));
        for x in bounds.left..bounds.right {
            let px = x as f64 + 0.5;
            let coverage = if radius == 0.0 {
                1.0
            } else {
                let distance = ((px - center_x).powi(2) + (py - nearest_y).powi(2)).sqrt();
                (radius + 0.5 - distance).clamp(0.0, 1.0)
            };
            if coverage == 0.0 {
                continue;
            }
            let source = quad
                .color
                .map(|channel| (f64::from(channel) * coverage).round().clamp(0.0, 255.0) as u8);
            let index = view.row_start(y, x);
            buffer[index] = source_over_word(buffer[index], source, format);
        }
    }
}

#[derive(Clone)]
enum Scope {
    Clip(Arc<ClipPaintPropertyNode>),
    Effect(Arc<EffectPaintPropertyNode>),
    DstIn(Arc<EffectPaintPropertyNode>),
}
impl Scope {
    fn same_node(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Clip(a), Self::Clip(b)) => a.lifecycle.same_node(&b.lifecycle),
            (Self::Effect(a), Self::Effect(b)) => a.lifecycle.same_node(&b.lifecycle),
            (Self::DstIn(a), Self::DstIn(b)) => a.lifecycle.same_node(&b.lifecycle),
            _ => false,
        }
    }
    fn same_values(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Clip(a), Self::Clip(b)) => a.lifecycle.revision == b.lifecycle.revision,
            (Self::Effect(a), Self::Effect(b)) => {
                a.lifecycle.revision == b.lifecycle.revision
                    && a.opacity == b.opacity
                    && a.blend_mode == b.blend_mode
                    && a.isolates_blending == b.isolates_blending
                    && a.filters == b.filters
                    && a.filter_reference_box == b.filter_reference_box
                    && a.filter_reference_scale == b.filter_reference_scale
                    && a.backdrop_filters == b.backdrop_filters
                    && a.backdrop_filter_has_reference == b.backdrop_filter_has_reference
                    && a.backdrop_filter_clip
                        .as_ref()
                        .map(|clip| clip.lifecycle.revision)
                        == b.backdrop_filter_clip
                            .as_ref()
                            .map(|clip| clip.lifecycle.revision)
            }
            (Self::DstIn(a), Self::DstIn(b)) => {
                a.lifecycle.revision == b.lifecycle.revision && a.opacity == b.opacity
            }
            _ => false,
        }
    }
    fn opacity(&self) -> f32 {
        match self {
            Self::Clip(_) => 1.0,
            Self::Effect(effect) | Self::DstIn(effect) => effect.opacity,
        }
    }

    fn blend_mode(&self) -> PaintBlendMode {
        match self {
            Self::Effect(effect) => effect.blend_mode,
            Self::Clip(_) | Self::DstIn(_) => PaintBlendMode::kNormal,
        }
    }
}

fn color_matrix(filter: PaintFilterType, amount: f32) -> [f32; 20] {
    let kind = match filter {
        PaintFilterType::kGrayscale => ColorMatrixKind::Grayscale,
        PaintFilterType::kSepia => ColorMatrixKind::Sepia,
        PaintFilterType::kSaturate => ColorMatrixKind::Saturate,
        PaintFilterType::kHueRotate => ColorMatrixKind::HueRotate,
        PaintFilterType::kInvert => ColorMatrixKind::Invert,
        PaintFilterType::kOpacity => ColorMatrixKind::Opacity,
        PaintFilterType::kBrightness => ColorMatrixKind::Brightness,
        PaintFilterType::kContrast => ColorMatrixKind::Contrast,
        PaintFilterType::kReference
        | PaintFilterType::kBlur
        | PaintFilterType::kDropShadow
        | PaintFilterType::kBoxReflect => {
            unreachable!()
        }
    };
    skia::cpu::layer_filters::css_color_matrix(kind, amount)
}

fn svg_filter(
    effect: &EffectPaintPropertyNode,
    graph: &SvgFilterGraph,
    scale: f64,
    scroll: Option<layer_tile::CompositorScrollOffset>,
) -> io::Result<EffectFilter> {
    let reference = effect.filter_reference_box.ok_or_else(|| {
        io::Error::new(io::ErrorKind::Unsupported, "missing-filter-reference-box")
    })?;
    let matrix =
        layer_tile::compositor_transform_with_scroll(&effect.local_transform_space, scale, scroll)
            .map_err(|_| io::Error::new(io::ErrorKind::Unsupported, "filter-transform"))?;
    let sx = matrix.values[0].hypot(matrix.values[1]);
    let sy = matrix.values[4].hypot(matrix.values[5]);
    if !sx.is_finite() || !sy.is_finite() || sx <= f64::EPSILON || sy <= f64::EPSILON {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "filter-transform",
        ));
    }
    let determinant = matrix.values[0] * matrix.values[5] - matrix.values[4] * matrix.values[1];
    if !determinant.is_finite() || determinant.abs() <= f64::EPSILON {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "filter-transform",
        ));
    }
    raster::svg_filter_graph::TranslateSvgFilterGraphWithTransform(
        graph,
        reference,
        sx,
        sy,
        0.0,
        0.0,
        reference.x,
        reference.y,
        effect.filter_reference_scale,
    )
    .map(|graph| EffectFilter::AffineSvgGraph {
        graph,
        filter_to_device: [
            (matrix.values[0] / sx) as f32,
            (matrix.values[1] / sx) as f32,
            (matrix.values[4] / sy) as f32,
            (matrix.values[5] / sy) as f32,
            matrix.values[12] as f32,
            matrix.values[13] as f32,
        ],
    })
}

fn translated_filters(
    effect: &EffectPaintPropertyNode,
    filters: &[layoutng_assembly::internal::paint_input::PaintFilterOperation],
    scale: f64,
    prepare_masks: bool,
    scroll: Option<layer_tile::CompositorScrollOffset>,
) -> io::Result<Vec<EffectFilter>> {
    if filters.is_empty() {
        return Ok(Vec::new());
    }
    let matrix =
        layer_tile::compositor_transform_with_scroll(&effect.local_transform_space, scale, scroll)
            .map_err(|_| io::Error::new(io::ErrorKind::Unsupported, "filter-transform"))?;
    let sx = matrix.values[0].hypot(matrix.values[1]);
    let sy = matrix.values[4].hypot(matrix.values[5]);
    filters
        .iter()
        .map(|filter| {
            if !filter.amount.is_finite()
                || !filter.blur_radius.is_finite()
                || !filter.offset.x.is_finite()
                || !filter.offset.y.is_finite()
                || ![
                    filter.color.red,
                    filter.color.green,
                    filter.color.blue,
                    filter.color.alpha,
                ]
                .iter()
                .all(|value| value.is_finite())
            {
                return Err(io::Error::new(io::ErrorKind::Unsupported, "filter-effect"));
            }
            let amount = filter.amount as f32;
            match filter.r#type {
                PaintFilterType::kReference => {
                    if let Some(error) = &filter.reference_error {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            format!("reference-filter: {error}"),
                        ));
                    }
                    let graph = filter.reference_graph.as_ref().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::Unsupported, "missing-reference-filter-graph")
                    })?;
                    svg_filter(effect, graph, scale, scroll)
                }
                PaintFilterType::kGrayscale
                | PaintFilterType::kSepia
                | PaintFilterType::kInvert
                | PaintFilterType::kOpacity
                    if !(0.0..=1.0).contains(&filter.amount) =>
                {
                    Err(io::Error::new(io::ErrorKind::Unsupported, "filter-effect"))
                }
                PaintFilterType::kSaturate
                | PaintFilterType::kBrightness
                | PaintFilterType::kContrast
                    if filter.amount < 0.0 =>
                {
                    Err(io::Error::new(io::ErrorKind::Unsupported, "filter-effect"))
                }
                PaintFilterType::kBlur if filter.amount < 0.0 => {
                    Err(io::Error::new(io::ErrorKind::Unsupported, "filter-effect"))
                }
                PaintFilterType::kDropShadow if filter.blur_radius < 0.0 => {
                    Err(io::Error::new(io::ErrorKind::Unsupported, "filter-effect"))
                }
                PaintFilterType::kBlur => Ok(EffectFilter::Blur {
                    sigma_x: (filter.amount * sx) as f32,
                    sigma_y: (filter.amount * sy) as f32,
                }),
                PaintFilterType::kDropShadow => {
                    // FilterEffectBuilder floors the CSS offset before cc
                    // maps it through the render-surface transform.
                    let x = filter.offset.x.floor();
                    let y = filter.offset.y.floor();
                    Ok(EffectFilter::DropShadow {
                        sigma_x: (filter.blur_radius * sx) as f32,
                        sigma_y: (filter.blur_radius * sy) as f32,
                        offset_x: (matrix.values[0] * x + matrix.values[4] * y) as f32,
                        offset_y: (matrix.values[1] * x + matrix.values[5] * y) as f32,
                        color: [
                            filter.color.red,
                            filter.color.green,
                            filter.color.blue,
                            filter.color.alpha,
                        ],
                    })
                }
                PaintFilterType::kBoxReflect => {
                    if filter.reflection_mask_image.is_some()
                        || filter.reflection_offset_percentage != 0.0
                        || filter.reflection_offset_calculation.is_some()
                        || !filter.reflection_offset.is_finite()
                    {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            "box-reflect-mask",
                        ));
                    }
                    let local = match filter.reflection_direction {
                        PaintReflectionDirection::kAbove | PaintReflectionDirection::kBelow => {
                            [1.0, 0.0, 0.0, -1.0, 0.0, filter.reflection_offset]
                        }
                        PaintReflectionDirection::kLeft | PaintReflectionDirection::kRight => {
                            [-1.0, 0.0, 0.0, 1.0, filter.reflection_offset, 0.0]
                        }
                    };
                    let affine = |a: [f64; 6], b: [f64; 6]| {
                        [
                            a[0] * b[0] + a[2] * b[1],
                            a[1] * b[0] + a[3] * b[1],
                            a[0] * b[2] + a[2] * b[3],
                            a[1] * b[2] + a[3] * b[3],
                            a[0] * b[4] + a[2] * b[5] + a[4],
                            a[1] * b[4] + a[3] * b[5] + a[5],
                        ]
                    };
                    let device = [
                        matrix.values[0],
                        matrix.values[1],
                        matrix.values[4],
                        matrix.values[5],
                        matrix.values[12],
                        matrix.values[13],
                    ];
                    let determinant = device[0] * device[3] - device[2] * device[1];
                    if determinant.abs() <= f64::EPSILON {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            "box-reflect-transform",
                        ));
                    }
                    let inverse = [
                        device[3] / determinant,
                        -device[1] / determinant,
                        -device[2] / determinant,
                        device[0] / determinant,
                        (device[2] * device[5] - device[3] * device[4]) / determinant,
                        (device[1] * device[4] - device[0] * device[5]) / determinant,
                    ];
                    Ok(EffectFilter::BoxReflect {
                        matrix: affine(affine(device, local), inverse).map(|value| value as f32),
                        mask: if prepare_masks && filter.reflection_has_mask {
                            Some(skia::cpu::layer_filters::RenderReflectionMask {
                                layers: raster::CompileReflectionMaskLayers(
                                    &filter.reflection_mask_layers,
                                )?,
                                local_to_device: device.map(|value| value as f32),
                            })
                        } else {
                            None
                        },
                    })
                }
                _ => Ok(EffectFilter::ColorMatrix(color_matrix(
                    filter.r#type,
                    amount,
                ))),
            }
        })
        .collect()
}

fn effect_filters(
    scope: &Scope,
    scale: f64,
    prepare_masks: bool,
    scroll: Option<layer_tile::CompositorScrollOffset>,
) -> io::Result<Vec<EffectFilter>> {
    let Scope::Effect(effect) = scope else {
        return Ok(Vec::new());
    };
    translated_filters(effect, &effect.filters, scale, prepare_masks, scroll)
}

fn backdrop_filters(
    scope: &Scope,
    scale: f64,
    scroll: Option<layer_tile::CompositorScrollOffset>,
) -> io::Result<Vec<EffectFilter>> {
    let Scope::Effect(effect) = scope else {
        return Ok(Vec::new());
    };
    // A projected local url() is an ordinary reference operation in this
    // chain. Unresolved/external references retain their missing graph or
    // reference_error and are rejected by translated_filters below.
    translated_filters(effect, &effect.backdrop_filters, scale, false, scroll)
}

fn expanded_for_filters(
    bounds: Option<SurfaceBounds>,
    filters: &[EffectFilter],
    width: u32,
    height: u32,
) -> Option<SurfaceBounds> {
    let mut bounds = bounds?;
    let clamp_floor = |value: f32, maximum: u32| value.floor().clamp(0.0, maximum as f32) as usize;
    let clamp_ceil = |value: f32, maximum: u32| value.ceil().clamp(0.0, maximum as f32) as usize;
    for filter in filters {
        match *filter {
            EffectFilter::ColorMatrix(_) => {}
            EffectFilter::SvgGraph(_) | EffectFilter::AffineSvgGraph { .. } => {
                bounds = SurfaceBounds {
                    left: 0,
                    top: 0,
                    right: width as usize,
                    bottom: height as usize,
                };
            }
            EffectFilter::BoxReflect { .. } => {
                // Reflection can move the input to either side of the effect
                // origin after an arbitrary affine local transform.
                bounds = SurfaceBounds {
                    left: 0,
                    top: 0,
                    right: width as usize,
                    bottom: height as usize,
                };
            }
            EffectFilter::Blur { sigma_x, sigma_y } => {
                let x = (sigma_x * 3.0).ceil() as usize + 1;
                let y = (sigma_y * 3.0).ceil() as usize + 1;
                bounds.left = bounds.left.saturating_sub(x);
                bounds.top = bounds.top.saturating_sub(y);
                bounds.right = bounds.right.saturating_add(x).min(width as usize);
                bounds.bottom = bounds.bottom.saturating_add(y).min(height as usize);
            }
            EffectFilter::DropShadow {
                sigma_x,
                sigma_y,
                offset_x,
                offset_y,
                ..
            } => {
                let x = (sigma_x * 3.0).ceil() + 1.0;
                let y = (sigma_y * 3.0).ceil() + 1.0;
                let shadow = SurfaceBounds {
                    left: clamp_floor(bounds.left as f32 + offset_x - x, width),
                    top: clamp_floor(bounds.top as f32 + offset_y - y, height),
                    right: clamp_ceil(bounds.right as f32 + offset_x + x, width),
                    bottom: clamp_ceil(bounds.bottom as f32 + offset_y + y, height),
                };
                bounds = SurfaceBounds {
                    left: bounds.left.min(shadow.left),
                    top: bounds.top.min(shadow.top),
                    right: bounds.right.max(shadow.right),
                    bottom: bounds.bottom.max(shadow.bottom),
                };
            }
        }
    }
    Some(bounds)
}

fn needs_clip_mask(clip: &ClipPaintPropertyNode, scale: f64) -> bool {
    // PropertyTreeManager::SyntheticEffectType does not create an effect for
    // an axis-aligned plain rect, even at a fractional coordinate. Retained
    // compositor rects use LayerComposition's enclosing integer scissor
    // (draw_property_utils / SoftwareRenderer::SetClipRect). Clips removed
    // into a layer's PaintRecord remain AA, as in SwitchToClip/StartClip.
    // An affine transform turns a plain rect into a non-axis-aligned shape.
    // Chromium represents that as a synthesized clip effect, so preserve its
    // exact transformed edges in the same A8 coverage path as rrects/paths.
    let affine = layer_tile::compositor_transform(&clip.local_transform_space, scale)
        .map(|matrix| matrix.values[1] != 0.0 || matrix.values[4] != 0.0)
        .unwrap_or(true);
    affine
        || clip.radii != PaintCornerRadii::default()
        || clip.clip_path_error.is_some()
        || (clip.clip_path_is_empty || !clip.clip_path.is_empty())
}
fn append_clip_scopes(
    leaf: &Arc<ClipPaintPropertyNode>,
    scopes: &mut Vec<Scope>,
    seen: &mut Vec<Arc<ClipPaintPropertyNode>>,
    scale: f64,
) {
    scopes.extend(
        take_unseen_clip_masks(leaf, seen, scale)
            .into_iter()
            .map(Scope::Clip),
    );
}
fn take_unseen_clip_masks(
    leaf: &Arc<ClipPaintPropertyNode>,
    seen: &mut Vec<Arc<ClipPaintPropertyNode>>,
    scale: f64,
) -> Vec<Arc<ClipPaintPropertyNode>> {
    let mut masks = Vec::new();
    let mut chain = Vec::new();
    let mut clip = Some(leaf.clone());
    while let Some(node) = clip {
        clip = node.parent.clone();
        chain.push(node);
    }
    for clip in chain.into_iter().rev() {
        if !seen
            .iter()
            .any(|old| old.lifecycle.same_node(&clip.lifecycle))
        {
            if needs_clip_mask(&clip, scale) {
                masks.push(clip.clone());
            }
            seen.push(clip);
        }
    }
    masks
}
// Actual Mask effects require their content parent to be a render surface,
// including opacity-one parents. Derive this from all committed mask layers,
// not just visible tiles: a fully transparent/offscreen source still clears DstIn.
fn masked_content_effects(plan: &CompositionPlan) -> io::Result<Vec<Arc<EffectPaintPropertyNode>>> {
    let mut parents: Vec<Arc<EffectPaintPropertyNode>> = Vec::new();
    for layer in &plan.layers {
        let mut node = Some(layer.properties.effect.clone());
        while let Some(effect) = node {
            node = effect.parent.clone();
            if effect.is_mask {
                let parent = effect
                    .parent
                    .as_ref()
                    .ok_or_else(|| io::Error::other("mask effect has no content parent"))?;
                if !parents
                    .iter()
                    .any(|old| old.lifecycle.same_node(&parent.lifecycle))
                {
                    parents.push(parent.clone());
                }
            }
        }
    }
    Ok(parents)
}
fn composition_scopes(
    effect: &Arc<EffectPaintPropertyNode>,
    owner_effect: Option<&Arc<EffectPaintPropertyNode>>,
    retained: &[Arc<ClipPaintPropertyNode>],
    masked_parents: &[Arc<EffectPaintPropertyNode>],
    scale: f64,
) -> io::Result<(Vec<Scope>, Vec<Arc<[Arc<ClipPaintPropertyNode>]>>)> {
    let mut scopes = Vec::new();
    let mut scope_output_clips = Vec::new();
    let mut seen = Vec::new();
    let mut chain = Vec::new();
    let mut node = Some(effect.clone());
    while let Some(effect) = node {
        node = effect.parent.clone();
        chain.push(effect);
    }
    chain.reverse();
    let start = if let Some(owner) = owner_effect {
        chain
            .iter()
            .position(|effect| effect.lifecycle.same_node(&owner.lifecycle))
            .map(|index| index + 1)
            .ok_or_else(|| io::Error::other("render-pass owner is not an effect ancestor"))?
    } else {
        0
    };
    for effect in chain.into_iter().skip(start) {
        if effect.is_mask {
            // Clip the mask source inside its DstIn surface. The content parent
            // must remain the immediate restore destination, so transparent
            // source outside its output clip also clears the parent contents.
            scopes.push(Scope::DstIn(effect.clone()));
            scope_output_clips.push(Arc::from([]));
            if let Some(clip) = &effect.output_clip {
                let old_len = scopes.len();
                append_clip_scopes(clip, &mut scopes, &mut seen, scale);
                scope_output_clips.extend((old_len..scopes.len()).map(|_| Arc::from([])));
            }
        } else if effect.opacity != 1.0
            || !effect.filters.is_empty()
            || !effect.backdrop_filters.is_empty()
            || effect.backdrop_filter_has_reference
            || effect.blend_mode != PaintBlendMode::kNormal
            || effect.isolates_blending
            || masked_parents
                .iter()
                .any(|parent| parent.lifecycle.same_node(&effect.lifecycle))
        {
            // Apply output coverage on the effect's restore. Nesting the
            // blend surface inside a transparent synthesized-clip surface
            // would make mix-blend-mode sample transparency instead of the
            // real parent backdrop. A restore mask also preserves filter-then-
            // clip ordering and avoids multiplying shared AA edges twice.
            let output_clips = effect
                .output_clip
                .as_ref()
                .map(|clip| take_unseen_clip_masks(clip, &mut seen, scale))
                .unwrap_or_default();
            scopes.push(Scope::Effect(effect));
            scope_output_clips.push(Arc::from(output_clips));
        }
    }
    for clip in retained.iter().rev() {
        if !seen
            .iter()
            .any(|old| old.lifecycle.same_node(&clip.lifecycle))
        {
            if needs_clip_mask(clip, scale) {
                scopes.push(Scope::Clip(clip.clone()));
                scope_output_clips.push(Arc::from([]));
            }
            seen.push(clip.clone());
        }
    }
    debug_assert_eq!(scopes.len(), scope_output_clips.len());
    Ok((scopes, scope_output_clips))
}

fn clip_chain(leaf: &Arc<ClipPaintPropertyNode>) -> Vec<Arc<ClipPaintPropertyNode>> {
    let mut chain = Vec::new();
    let mut node = Some(leaf.clone());
    while let Some(clip) = node {
        node = clip.parent.clone();
        chain.push(clip);
    }
    chain.reverse();
    chain
}

fn backdrop_output_bounds(
    effect: &EffectPaintPropertyNode,
    width: u32,
    height: u32,
    scale: f64,
    scroll: Option<layer_tile::CompositorScrollOffset>,
) -> io::Result<Option<SurfaceBounds>> {
    let Some(leaf) = &effect.backdrop_filter_clip else {
        return Ok(None);
    };
    let mut output = SurfaceBounds {
        left: 0,
        top: 0,
        right: width as usize,
        bottom: height as usize,
    };
    for clip in clip_chain(leaf) {
        let Some(rect) = clip.rect else {
            continue;
        };
        let matrix = layer_tile::compositor_transform_with_scroll(
            &clip.local_transform_space,
            scale,
            scroll,
        )
        .map_err(|_| io::Error::new(io::ErrorKind::Unsupported, "backdrop-filter-transform"))?;
        let mut left = f64::INFINITY;
        let mut top = f64::INFINITY;
        let mut right = f64::NEG_INFINITY;
        let mut bottom = f64::NEG_INFINITY;
        for (x, y) in [
            (rect.x, rect.y),
            (rect.x + rect.width, rect.y),
            (rect.x, rect.y + rect.height),
            (rect.x + rect.width, rect.y + rect.height),
        ] {
            let px = matrix.values[0] * x + matrix.values[4] * y + matrix.values[12];
            let py = matrix.values[1] * x + matrix.values[5] * y + matrix.values[13];
            if !px.is_finite() || !py.is_finite() {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "backdrop-filter-transform",
                ));
            }
            left = left.min(px);
            top = top.min(py);
            right = right.max(px);
            bottom = bottom.max(py);
        }
        let projected = SurfaceBounds {
            left: left.floor().clamp(0.0, width as f64) as usize,
            top: top.floor().clamp(0.0, height as f64) as usize,
            right: right.ceil().clamp(0.0, width as f64) as usize,
            bottom: bottom.ceil().clamp(0.0, height as f64) as usize,
        };
        let Some(intersection) = intersect_bounds(output, projected) else {
            return Ok(None);
        };
        output = intersection;
    }
    Ok(Some(output))
}

#[inline]
fn target_word_as_rgba(mut word: u32, format: skia::PixelFormat) -> u32 {
    if format != skia::PixelFormat::Rgba8888 {
        word = (word & 0xff00_ff00) | ((word & 0xff) << 16) | ((word >> 16) & 0xff);
    }
    if format == skia::PixelFormat::Bgrx8888 {
        word |= 0xff00_0000;
    }
    word
}

fn copy_backdrop(
    destination: &mut [u32],
    destination_view: TargetView,
    destination_bounds: SurfaceBounds,
    source: &[u32],
    source_view: TargetView,
    source_bounds: SurfaceBounds,
    source_format: skia::PixelFormat,
) {
    let Some(copy) = intersect_bounds(destination_bounds, source_bounds) else {
        return;
    };
    for y in copy.top..copy.bottom {
        let source_row = source_view.row_range(y, copy.left, copy.right);
        let destination_row = destination_view.row_range(y, copy.left, copy.right);
        for (output, &input) in destination[destination_row]
            .iter_mut()
            .zip(&source[source_row])
        {
            *output = target_word_as_rgba(input, source_format);
        }
    }
}

#[inline]
fn mirror_coordinate(value: usize, start: usize, end: usize) -> usize {
    debug_assert!(start < end);
    let extent = end - start;
    let period = extent * 2;
    let offset = if value >= start {
        (value - start) % period
    } else {
        let backwards = (start - value) % period;
        if backwards == 0 {
            0
        } else {
            period - backwards
        }
    };
    if offset < extent {
        start + offset
    } else {
        end - 1 - (offset - extent)
    }
}

/// SoftwareRenderer::GetBackdropFilterShader reads only the backdrop-filter
/// output rect. When a pixel-moving filter needs samples beyond that bitmap,
/// ApplyBackdropFilterWithExactOutputSize composes an SkTileMode::kMirror crop.
/// Keep the larger scratch allocation used by our CPU filter executor, but
/// populate its input with that same mirrored output-rect backdrop.
fn copy_backdrop_mirrored(
    destination: &mut [u32],
    destination_view: TargetView,
    destination_bounds: SurfaceBounds,
    output_bounds: SurfaceBounds,
    source: &[u32],
    source_view: TargetView,
    source_bounds: SurfaceBounds,
    source_format: skia::PixelFormat,
) {
    let Some(available) = intersect_bounds(output_bounds, source_bounds) else {
        return;
    };
    for y in destination_bounds.top..destination_bounds.bottom {
        let source_y = mirror_coordinate(y, available.top, available.bottom);
        for x in destination_bounds.left..destination_bounds.right {
            let source_x = mirror_coordinate(x, available.left, available.right);
            destination[destination_view.row_start(y, x)] = target_word_as_rgba(
                source[source_view.row_start(source_y, source_x)],
                source_format,
            );
        }
    }
}

fn apply_backdrop_coverage(
    pixels: &mut [u32],
    view: TargetView,
    storage: SurfaceBounds,
    output: SurfaceBounds,
    coverage: &MaskCoverage,
) -> Option<SurfaceBounds> {
    let coverage_bounds = coverage
        .bounds
        .map(|(left, top, right, bottom)| SurfaceBounds {
            left,
            top,
            right,
            bottom,
        })?;
    let touched = intersect_bounds(output, coverage_bounds)?;
    for y in storage.top..storage.bottom {
        for x in storage.left..storage.right {
            let alpha = if x >= output.left
                && x < output.right
                && y >= output.top
                && y < output.bottom
                && y >= coverage.origin.1
                && y < coverage.origin.1 + coverage.rows.len()
                && x >= coverage.origin.0
                && x < coverage.origin.0 + coverage.stride
            {
                coverage.pixels[(y - coverage.origin.1) * coverage.stride + x - coverage.origin.0]
            } else {
                0
            };
            let index = view.row_start(y, x);
            pixels[index] = scale_mask_word(pixels[index], alpha);
        }
    }
    Some(touched)
}

struct EffectSurface {
    run: usize,
    scope: Scope,
    direct_dst_in: bool,
    direct_clip: bool,
    dst_in_root: bool,
    coverage: Option<Arc<MaskCoverage>>,
    // All intermediate surfaces retain premultiplied RGBA including alpha.
    pixels: Vec<u32>,
    view: TargetView,
    touched: Option<SurfaceBounds>,
    filters: Vec<EffectFilter>,
    // Backdrop sampling is deferred until the stable render-surface restore.
    // A BSP-fragmented effect may be suspended while unrelated parent content
    // is drawn; sampling on the first opening would miss that content.
    backdrop_filters: Vec<EffectFilter>,
    backdrop_output: Option<SurfaceBounds>,
    backdrop_coverage: Option<Arc<MaskCoverage>>,
    visible_polygon: Option<viz::TargetPolygon>,
}
struct EffectScratch {
    pixels: Vec<u32>,
    touched: Option<SurfaceBounds>,
    view: TargetView,
}

struct LayerEffectPlan {
    chain: Vec<Scope>,
    /// Nontrivial Effect::output_clip nodes applied once at surface restore.
    /// This stays parallel to `chain`; ordinary synthesized Clip scopes have
    /// an empty entry because their coverage owns a separate scope run.
    scope_output_clips: Vec<Arc<[Arc<ClipPaintPropertyNode>]>>,
    opened_runs: Vec<usize>,
    direct_mask: Option<Vec<SurfaceBounds>>,
}
impl LayerEffectPlan {
    fn has_backdrop(&self) -> bool {
        self.chain.iter().any(
            |scope| matches!(scope, Scope::Effect(effect) if !effect.backdrop_filters.is_empty() || effect.backdrop_filter_has_reference),
        )
    }
}

// A native effect owns one run even when legacy flat input reopens it. New
// compositor frames encode isolated effects as child render passes, but this
// lifecycle fallback remains required for ordinary clip/effect chains that do
// not need their own Viz surface. Bounds affect allocation only; real mask
// coverage still runs at group exit.
fn effect_allocation_plan(
    plan: &CompositionPlan,
    width: u32,
    height: u32,
    scale: f64,
) -> io::Result<(Vec<LayerEffectPlan>, Vec<Option<SurfaceBounds>>)> {
    let masked_parents = masked_content_effects(plan)?;
    let mut layers = Vec::with_capacity(plan.layers.len());
    let mut runs = Vec::new();
    let mut active: Vec<(Scope, usize)> = Vec::new();
    // Preserve one Viz render-surface identity when BSP ordering emits
    // fragments from the same Effect node in separate stretches. Ordinary
    // synthesized Clip scopes remain per-opening because they can be applied
    // independently; opacity/blend/filter/mask scopes must accumulate all
    // contributing fragments before one restore.
    let mut persistent_runs: Vec<(Scope, usize)> = Vec::new();
    for layer in &plan.layers {
        let mut effect_cursor = Some(&layer.properties.effect);
        let mut has_backdrop = false;
        while let Some(effect) = effect_cursor {
            has_backdrop |=
                !effect.backdrop_filters.is_empty() || effect.backdrop_filter_has_reference;
            effect_cursor = effect.parent.as_ref();
        }
        if !layer.has_drawable_content() && !layer.properties.effect.is_mask && !has_backdrop {
            layers.push(LayerEffectPlan {
                chain: Vec::new(),
                scope_output_clips: Vec::new(),
                opened_runs: Vec::new(),
                direct_mask: None,
            });
            continue;
        }
        let scope_effect = layer
            .render_pass
            .and_then(|_| layer.properties.effect.parent.as_ref())
            .unwrap_or(&layer.properties.effect);
        let (chain, scope_output_clips) = composition_scopes(
            scope_effect,
            plan.owner_effect.as_ref(),
            &layer.retained_clips,
            &masked_parents,
            scale,
        )?;
        if chain
            .iter()
            .any(|scope| matches!(scope, Scope::Effect(effect) if effect.opacity == 0.0))
        {
            layers.push(LayerEffectPlan {
                chain,
                scope_output_clips,
                opened_runs: Vec::new(),
                direct_mask: None,
            });
            continue;
        }
        let common = active
            .iter()
            .zip(&chain)
            .take_while(|((old, _), new)| old.same_node(new))
            .count();
        active.truncate(common);
        let mut opened_runs = Vec::with_capacity(chain.len() - common);
        for scope in chain.iter().skip(common) {
            let persistent = matches!(scope, Scope::Effect(_) | Scope::DstIn(_));
            let index = persistent_runs
                .iter()
                .find(|(known, _)| known.same_node(scope))
                .map(|(_, run)| *run)
                .unwrap_or_else(|| {
                    let run = runs.len();
                    runs.push(None);
                    if persistent {
                        persistent_runs.push((scope.clone(), run));
                    }
                    run
                });
            active.push((scope.clone(), index));
            opened_runs.push(index);
        }
        if !active.is_empty() {
            let composition = layer.composition;
            let mut bounds = None;
            if layer.render_pass.is_some() {
                bounds = render_pass_quad_bounds(composition, width, height);
            }
            for tile in &layer.tiles {
                union_bounds(
                    &mut bounds,
                    tile_composition_geometry(tile, composition, width, height)?
                        .map(|(bounds, _, _)| bounds),
                );
            }
            // A parent effect receives the already-filtered output of every
            // descendant surface.  Accumulate from the innermost scope so a
            // blur/drop-shadow chain cannot be clipped at a nested boundary.
            let mut filtered_bounds = bounds;
            for (scope, index) in active.iter().rev() {
                let backdrop = backdrop_filters(scope, scale, layer.compositor_scroll)?;
                if let Scope::Effect(effect) = scope {
                    let output = backdrop_output_bounds(
                        effect,
                        width,
                        height,
                        scale,
                        layer.compositor_scroll,
                    )?;
                    let input = expanded_for_filters(output, &backdrop, width, height);
                    union_bounds(&mut filtered_bounds, input);
                }
                let filters = effect_filters(scope, scale, false, layer.compositor_scroll)?;
                filtered_bounds = expanded_for_filters(filtered_bounds, &filters, width, height);
                union_bounds(&mut runs[*index], filtered_bounds);
            }
        }
        let direct_mask = direct_mask_partition(plan, layer, &chain, width, height)?;
        layers.push(LayerEffectPlan {
            chain,
            scope_output_clips,
            opened_runs,
            direct_mask,
        });
    }
    Ok((layers, runs))
}

// Corresponds to PropertyTreeManager's conditional kBlendModeDstIn surface:
// one actual layer with no child surface can draw its tile quads directly.
// Reuse the existing tile sampler, retaining its ownership and AA convention.
fn direct_mask_partition(
    plan: &CompositionPlan,
    layer: &CompositionLayer,
    chain: &[Scope],
    width: u32,
    height: u32,
) -> io::Result<Option<Vec<SurfaceBounds>>> {
    if plan.owner_effect.is_some() {
        return Ok(None);
    }
    let effect = &layer.properties.effect;
    if !effect.is_mask
        || effect.opacity != 1.0
        || !matches!(chain.last(), Some(Scope::DstIn(node)) if node.lifecycle.same_node(&effect.lifecycle))
        || plan
            .layers
            .iter()
            .filter(|candidate| {
                let mut node = Some(&candidate.properties.effect);
                while let Some(ancestor) = node {
                    if ancestor.lifecycle.same_node(&effect.lifecycle) {
                        return true;
                    }
                    node = ancestor.parent.as_ref();
                }
                false
            })
            .count()
            != 1
    {
        return Ok(None);
    }
    let composition = layer.composition;
    if composition.white_backing {
        return Ok(None);
    }
    layer_tile_partition(layer, composition, width, height)
}
fn layer_tile_partition(
    layer: &CompositionLayer,
    composition: LayerComposition,
    width: u32,
    height: u32,
) -> io::Result<Option<Vec<SurfaceBounds>>> {
    // Rotated/skewed tile bounding boxes overlap even though their actual
    // parallelograms do not.  Keep those layers on the general effect/clip
    // surface instead of using the rectangular direct-mask partition proof.
    if composition.projective.is_some() || !composition.transform.is_translation() {
        return Ok(None);
    }
    let mut rectangles = Vec::with_capacity(layer.tiles.len());
    for tile in &layer.tiles {
        if let Some((bounds, _, _)) = tile_composition_geometry(tile, composition, width, height)? {
            rectangles.push(bounds);
        }
    }
    rectangles.sort_unstable_by_key(|b| (b.top, b.left, b.bottom, b.right));
    // Prove a disjoint row grid in one sorted scan. Conservatively reject any
    // staggered band, even if a more general rectangle intersection could pass.
    let mut band: Option<SurfaceBounds> = None;
    for &rect in &rectangles {
        if let Some(previous) = band {
            if rect.top == previous.top {
                if rect.bottom != previous.bottom || rect.left < previous.right {
                    return Ok(None);
                }
            } else if rect.top < previous.bottom {
                return Ok(None);
            }
        }
        band = Some(rect);
    }
    Ok(Some(rectangles))
}

fn partition_covers_viewport(rectangles: &[SurfaceBounds], width: u32, height: u32) -> bool {
    let Some(first) = rectangles.first() else {
        return false;
    };
    if first.top != 0 {
        return false;
    }
    let (mut top, mut bottom, mut right) = (first.top, first.bottom, 0);
    for rect in rectangles {
        if rect.top != top {
            if right != width as usize || rect.top != bottom {
                return false;
            }
            top = rect.top;
            bottom = rect.bottom;
            right = 0;
        }
        if rect.bottom != bottom || rect.left != right {
            return false;
        }
        right = rect.right;
    }
    right == width as usize && bottom == height as usize
}

#[inline]
fn compact_solid_color(pixels: &CachedTile) -> Option<u32> {
    match pixels.row_support.draw_mode {
        layer_solid::TileDrawMode::SolidColor { premul_rgba }
            if pixels.rgba.is_empty() && pixels.bgra.is_empty() =>
        {
            Some(premul_rgba)
        }
        _ => None,
    }
}

fn valid_tile_storage(pixels: &CachedTile, pixel_size: (u32, u32)) -> io::Result<bool> {
    let bytes = tile_bytes(pixel_size)?;
    Ok(
        (pixels.rgba.len() == bytes && pixels.bgra.len() == bytes / 4)
            || compact_solid_color(pixels).is_some(),
    )
}

pub(crate) struct RenderPassRenderer {
    effect_scratch: Vec<EffectScratch>,
    // Persistent root render-pass backing; native presentation surfaces rotate.
    composed_pixels: Vec<u32>,
    composed_format: Option<skia::PixelFormat>,
    composed_valid: bool,
    composed_output_bounds: Option<SurfaceBounds>,
    clip_masks: ClipMaskCache,
    stats: LayerTileRasterStats,
}

impl Default for RenderPassRenderer {
    fn default() -> Self {
        Self {
            effect_scratch: Vec::new(),
            composed_pixels: Vec::new(),
            composed_format: None,
            composed_valid: false,
            composed_output_bounds: None,
            clip_masks: ClipMaskCache::default(),
            stats: LayerTileRasterStats::default(),
        }
    }
}

impl RenderPassRenderer {
    pub(crate) fn backing_usage(&self) -> (usize, usize, usize) {
        (
            self.composed_pixels.capacity() * std::mem::size_of::<u32>(),
            self.effect_scratch
                .iter()
                .map(|scratch| scratch.pixels.capacity() * std::mem::size_of::<u32>())
                .sum(),
            self.clip_masks.accounted_bytes(),
        )
    }

    pub(crate) fn invalidate(&mut self) {
        self.composed_pixels.clear();
        self.composed_format = None;
        self.composed_valid = false;
        self.composed_output_bounds = None;
        self.clip_masks = ClipMaskCache::default();
        self.stats = LayerTileRasterStats::default();
    }

    fn unsupported(&mut self, reason: &'static str) -> io::Result<RenderUpdate> {
        self.invalidate();
        self.stats.unsupported_reason = Some(reason);
        Err(io::Error::new(io::ErrorKind::Unsupported, reason))
    }

    fn finish_effect_group(
        &mut self,
        groups: &mut Vec<EffectSurface>,
        target: &mut [u32],
        frame_view: TargetView,
        target_format: skia::PixelFormat,
        destination_damage: SurfaceBounds,
    ) {
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::class(5);
        let mut group = groups.pop().expect("active effect group");
        // Direct quads already performed this Mask effect, including exterior
        // clearing. Keep scope ordering/identity without allocating a source.
        if group.direct_dst_in || group.direct_clip {
            return;
        }
        if let (Some(output), Some(coverage)) =
            (group.backdrop_output, group.backdrop_coverage.as_deref())
        {
            let mut backdrop = vec![0u32; group.pixels.len()];
            let storage = SurfaceBounds {
                left: group.view.origin.0,
                top: group.view.origin.1,
                right: group.view.origin.0 + group.view.stride,
                bottom: group.view.origin.1 + group.pixels.len() / group.view.stride.max(1),
            };
            if let Some(parent) = groups.last() {
                let source_bounds = SurfaceBounds {
                    left: parent.view.origin.0,
                    top: parent.view.origin.1,
                    right: parent.view.origin.0 + parent.view.stride,
                    bottom: parent.view.origin.1 + parent.pixels.len() / parent.view.stride.max(1),
                };
                copy_backdrop_mirrored(
                    &mut backdrop,
                    group.view,
                    storage,
                    output,
                    &parent.pixels,
                    parent.view,
                    source_bounds,
                    skia::PixelFormat::Rgba8888,
                );
            } else {
                copy_backdrop_mirrored(
                    &mut backdrop,
                    group.view,
                    storage,
                    output,
                    target,
                    frame_view,
                    SurfaceBounds {
                        left: 0,
                        top: 0,
                        right: frame_view.stride,
                        bottom: target.len() / frame_view.stride.max(1),
                    },
                    target_format,
                );
            }
            if !group.backdrop_filters.is_empty() && !backdrop.is_empty() {
                let bytes = unsafe {
                    std::slice::from_raw_parts_mut(
                        backdrop.as_mut_ptr().cast::<u8>(),
                        backdrop.len() * std::mem::size_of::<u32>(),
                    )
                };
                for filter in &group.backdrop_filters {
                    let ok = skia::cpu::layer_filters::apply_rgba(
                        bytes,
                        group.view.stride as u32,
                        (backdrop.len() / group.view.stride.max(1)) as u32,
                        filter,
                    );
                    debug_assert!(ok);
                }
            }
            let mut touched =
                apply_backdrop_coverage(&mut backdrop, group.view, storage, output, coverage);
            let foreground = std::mem::replace(&mut group.pixels, backdrop);
            let written = composite_effect_surface(
                &mut group.pixels,
                &foreground,
                group.touched,
                group.view,
                group.view,
                skia::PixelFormat::Rgba8888,
                1.0,
                PaintBlendMode::kNormal,
                None,
                None,
            );
            union_bounds(&mut touched, written);
            group.touched = touched;
        }
        if !group.filters.is_empty() && !group.pixels.is_empty() {
            let mut trace = browser_tracing::span("compose", "ApplySurfaceFilters");
            trace.set("storage_pixels", group.pixels.len() as f64);
            trace.set("filter_count", group.filters.len() as f64);
            trace.set(
                "touched_pixels",
                group
                    .touched
                    .map_or(0, |b| (b.right - b.left) * (b.bottom - b.top)) as f64,
            );
            let width = group.view.stride;
            let height = group.pixels.len() / width;
            // Effect surfaces store native little-endian RGBA words, the same
            // premultiplied byte layout consumed by the translated Skia
            // render-surface filter adapters. Preserve CSS list order.
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    group.pixels.as_mut_ptr().cast::<u8>(),
                    group.pixels.len() * std::mem::size_of::<u32>(),
                )
            };
            for filter in &group.filters {
                let ok = skia::cpu::layer_filters::apply_rgba_at(
                    bytes,
                    width as u32,
                    height as u32,
                    filter,
                    (group.view.origin.0 as i32, group.view.origin.1 as i32),
                );
                debug_assert!(ok);
            }
            group.touched = expanded_for_filters(
                group.touched,
                &group.filters,
                (group.view.origin.0 + width) as u32,
                (group.view.origin.1 + height) as u32,
            )
            .map(|mut bounds| {
                bounds.left = bounds.left.max(group.view.origin.0);
                bounds.top = bounds.top.max(group.view.origin.1);
                bounds
            });
        }
        if let Scope::DstIn(effect) = &group.scope {
            if group.dst_in_root {
                dst_in_effect_surface(
                    target,
                    &group.pixels,
                    Some(destination_damage),
                    group.touched,
                    frame_view,
                    group.view,
                    effect.opacity,
                );
                return;
            }
            let parent = groups
                .last_mut()
                .expect("preflight validated isolated mask destination");
            let expected = effect
                .parent
                .as_ref()
                .expect("preflight validated mask content parent");
            debug_assert!(matches!(&parent.scope, Scope::Effect(actual)
                if actual.lifecycle.same_node(&expected.lifecycle)));
            parent.touched = dst_in_effect_surface(
                &mut parent.pixels,
                &group.pixels,
                parent.touched,
                group.touched,
                parent.view,
                group.view,
                effect.opacity,
            );
        } else if let Some(parent) = groups.last_mut() {
            let written = composite_effect_surface(
                &mut parent.pixels,
                &group.pixels,
                group.touched,
                parent.view,
                group.view,
                skia::PixelFormat::Rgba8888,
                group.scope.opacity(),
                group.scope.blend_mode(),
                group.coverage.as_deref(),
                group.visible_polygon.as_ref(),
            );
            union_bounds(&mut parent.touched, written);
        } else {
            // DirectRenderer applies the render-pass scissor to every final
            // destination draw, including render-pass quads. Filter input and
            // intermediate surfaces above remain complete; clipping them
            // before filtering would discard samples outside the damage.
            let touched = group
                .touched
                .and_then(|bounds| intersect_bounds(bounds, destination_damage));
            composite_effect_surface(
                target,
                &group.pixels,
                touched,
                frame_view,
                group.view,
                target_format,
                group.scope.opacity(),
                group.scope.blend_mode(),
                group.coverage.as_deref(),
                group.visible_polygon.as_ref(),
            );
        }
        // Scratch contains no reusable rendered result: every acquisition is
        // transparent after clearing old writes. Bound retained capacity.
        let bytes = group.pixels.capacity().saturating_mul(4);
        let retained = self
            .effect_scratch
            .iter()
            .map(|p| p.pixels.capacity().saturating_mul(4))
            .fold(0usize, usize::saturating_add);
        if self.effect_scratch.len() < 4
            && bytes <= 32 * 1024 * 1024
            && retained <= 32 * 1024 * 1024 - bytes
        {
            self.effect_scratch.push(EffectScratch {
                pixels: group.pixels,
                touched: group.touched,
                view: group.view,
            });
        }
    }

    fn cleared_effect_surface(&mut self, expected: usize) -> Vec<u32> {
        if let Some(mut scratch) = self.effect_scratch.pop() {
            if scratch.pixels.len() != expected {
                scratch.pixels.resize(expected, 0);
            }
            // SkCanvas::saveLayer() starts with a transparent device.  Scratch
            // reuse is only an allocation optimization; pixels outside the
            // previous `touched` estimate may still have been written by a
            // clip/effect close, so that estimate cannot weaken the clear
            // semantics of the next layer.
            scratch.pixels.fill(0);
            scratch.pixels
        } else {
            vec![0; expected]
        }
    }

    pub(crate) fn compose(
        &mut self,
        pass: &AggregatedRenderPass,
        resources: &dyn RasterResourceProvider,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
        row_stride: usize,
        render_pass_outputs: &HashMap<viz::RenderPassId, &[u32]>,
        force_full_damage: bool,
        suppress_damage: bool,
    ) -> io::Result<RenderUpdate> {
        self.compose_with_output(
            pass,
            resources,
            width,
            height,
            Some((buffer, row_stride)),
            format,
            render_pass_outputs,
            force_full_damage,
            suppress_damage,
        )
    }

    /// Render an internal pass into its existing backing. Only root passes
    /// need a copy into caller-owned presentation storage.
    pub(crate) fn compose_retained(
        &mut self,
        pass: &AggregatedRenderPass,
        resources: &dyn RasterResourceProvider,
        width: u32,
        height: u32,
        render_pass_outputs: &HashMap<viz::RenderPassId, &[u32]>,
        force_full_damage: bool,
        suppress_damage: bool,
    ) -> io::Result<RenderUpdate> {
        self.compose_with_output(
            pass,
            resources,
            width,
            height,
            None,
            skia::PixelFormat::Rgba8888,
            render_pass_outputs,
            force_full_damage,
            suppress_damage,
        )
    }

    pub(crate) fn backing_pixels(&self) -> Option<&[u32]> {
        self.composed_valid
            .then_some(self.composed_pixels.as_slice())
    }

    fn compose_with_output(
        &mut self,
        pass: &AggregatedRenderPass,
        resources: &dyn RasterResourceProvider,
        width: u32,
        height: u32,
        output: Option<(&mut [u32], usize)>,
        format: skia::PixelFormat,
        render_pass_outputs: &HashMap<viz::RenderPassId, &[u32]>,
        force_full_damage: bool,
        suppress_damage: bool,
    ) -> io::Result<RenderUpdate> {
        let mut render_trace = browser_tracing::span("render", "LayerTilePaint");
        render_trace.set("frame_id", pass.SourceFrameId() as f64);
        render_trace.set("render_pass_id", pass.RenderPassId().0 as f64);
        render_trace.set("output_width", pass.Viewport().width as f64);
        render_trace.set("output_height", pass.Viewport().height as f64);
        render_trace.set("force_full_damage", force_full_damage as u8 as f64);
        render_trace.set("suppress_damage", suppress_damage as u8 as f64);
        render_trace.set("damage_rects", pass.DamageRects().len() as f64);
        if let Some(shift) = pass.BackingShift() {
            if shift.1 != 0 {
                render_trace.set("backing_shift_y", shift.1 as f64);
            } else {
                render_trace.set("backing_shift_x", shift.0 as f64);
            }
        }
        if let Some(effect) = pass.OwnerEffect() {
            render_trace.set("effect_id", effect.id as f64);
            render_trace.set("effect_opacity", effect.opacity as f64);
            render_trace.set(
                "effect_blend",
                (effect.blend_mode != PaintBlendMode::kNormal) as u8 as f64,
            );
            render_trace.set("effect_filters", effect.filters.len() as f64);
            render_trace.set(
                "effect_backdrop_filters",
                effect.backdrop_filters.len() as f64,
            );
            render_trace.set(
                "effect_isolates_blending",
                effect.isolates_blending as u8 as f64,
            );
        }
        if resources.prepared_frame_id() != Some(pass.SourceFrameId()) {
            return Err(io::Error::other(
                "aggregated render-pass raster resources are not prepared",
            ));
        }
        let mut plan = CompositionPlan::from_pass(pass)?;
        self.stats = resources.stats();
        if let Some((buffer, row_stride)) = output.as_ref() {
            row_stride
                .checked_mul(height as usize)
                .filter(|&len| *row_stride >= width as usize && buffer.len() == len)
                .ok_or_else(|| io::Error::other("invalid layer output storage extent"))?;
        }
        let retained_len = (width as usize)
            .checked_mul(height as usize)
            .filter(|&len| len > 0 && len <= isize::MAX as usize / 4)
            .ok_or_else(|| io::Error::other("invalid retained output storage extent"))?;
        if self.composed_pixels.len() != retained_len || self.composed_format != Some(format) {
            self.composed_pixels.resize(retained_len, 0);
            self.composed_format = Some(format);
            self.composed_valid = false;
        }
        // SoftwareRenderer's render-pass bitmap is transparent outside its
        // output_rect. Our retained storage keeps surface coordinates/stride;
        // when that rect moves, erase the old extent before drawing the new one.
        // Allocation extent never enlarges either output or damage coverage.
        let bounds_changed = self.composed_output_bounds != plan.output_bounds;
        if bounds_changed {
            if let Some(old) = self.composed_output_bounds {
                for y in old.top.min(height as usize)..old.bottom.min(height as usize) {
                    let left = old.left.min(width as usize);
                    let right = old.right.min(width as usize);
                    self.composed_pixels[y * width as usize + left..y * width as usize + right]
                        .fill(0);
                }
            }
        }
        if self.composed_valid && !bounds_changed {
            if let (Some(delta), Some(bounds)) = (plan.backing_shift, plan.output_bounds) {
                shift_retained_backing(
                    &mut self.composed_pixels,
                    width as usize,
                    bounds,
                    delta,
                );
                render_trace.set("backing_shift_x", delta.0 as f64);
                render_trace.set("backing_shift_y", delta.1 as f64);
            }
        }
        // DirectRenderer::CanSkipRenderPass requires a valid backing. Repaint
        // the full *output rect* after loss, token gaps or output-rect changes.
        if suppress_damage && self.composed_valid && !bounds_changed {
            plan.root_damage = None;
            plan.damage_region.clear();
        } else if force_full_damage || !self.composed_valid || bounds_changed {
            plan.root_damage = plan.output_bounds;
            plan.damage_region = plan.output_bounds.into_iter().collect();
        }
        plan.damage_region = plan
            .damage_region
            .into_iter()
            .filter_map(|damage| {
                plan.output_bounds
                    .and_then(|output| intersect_bounds(damage, output))
            })
            .collect();
        plan.root_damage = plan
            .damage_region
            .iter()
            .copied()
            .reduce(|a, b| SurfaceBounds {
                left: a.left.min(b.left),
                top: a.top.min(b.top),
                right: a.right.max(b.right),
                bottom: a.bottom.max(b.bottom),
            });
        // Keep one renderer-owned root render-pass backing. Native IOSurfaces
        // are a rotating presentation queue, so repainting them directly would
        // otherwise discard unchanged pixels on every animation frame.
        let mut composed = std::mem::take(&mut self.composed_pixels);
        let damage_region = std::mem::take(&mut plan.damage_region);
        let mut result = if damage_region.is_empty() {
            plan.root_damage = None;
            self.paint_frame(
                &plan,
                width,
                height,
                &mut composed,
                format,
                width as usize,
                resources.resources(),
                resources.prepared_damage_pixels(),
                render_pass_outputs,
            )
        } else {
            let mut combined = RenderUpdate {
                mode: "layer-tiles",
                reason: "damage-region",
                damage_pixels: 0,
                copied_bytes: 0,
            };
            let mut failure = None;
            for damage in damage_region {
                plan.root_damage = Some(damage);
                match self.paint_frame(
                    &plan,
                    width,
                    height,
                    &mut composed,
                    format,
                    width as usize,
                    resources.resources(),
                    resources.prepared_damage_pixels(),
                    render_pass_outputs,
                ) {
                    Ok(update) => {
                        combined.damage_pixels =
                            combined.damage_pixels.saturating_add(update.damage_pixels);
                        combined.copied_bytes =
                            combined.copied_bytes.saturating_add(update.copied_bytes);
                    }
                    Err(error) => {
                        failure = Some(error);
                        break;
                    }
                }
            }
            failure.map_or(Ok(combined), Err)
        };
        self.composed_pixels = composed;
        result = result.map(|mut update| {
            if let Some((buffer, row_stride)) = output {
                for (source, destination) in self
                    .composed_pixels
                    .chunks_exact(width as usize)
                    .zip(buffer.chunks_exact_mut(row_stride))
                {
                    destination[..width as usize].copy_from_slice(source);
                    if row_stride > width as usize {
                        destination[width as usize..].fill(0);
                    }
                }
                update.copied_bytes = retained_len * std::mem::size_of::<u32>();
            }
            update
        });
        // A failed draw may have modified part of the retained target. Never
        // expose it as a valid retained pass on the next attempt.
        self.composed_valid = result.is_ok();
        self.composed_output_bounds = plan.output_bounds;
        render_trace.set("succeeded", if result.is_ok() { 1.0 } else { 0.0 });
        if let Ok(update) = &result {
            render_trace.set("layers", self.stats.layers as f64);
            render_trace.set("tiles", self.stats.tiles as f64);
            render_trace.set("raster_tasks", self.stats.raster_tasks as f64);
            render_trace.set("reused_tiles", self.stats.reused_tiles as f64);
            render_trace.set("resident_bytes", self.stats.resident_bytes as f64);
            render_trace.set("damage_pixels", update.damage_pixels as f64);
        }
        result
    }

    fn paint_frame(
        &mut self,
        plan: &CompositionPlan,
        width: u32,
        height: u32,
        buffer: &mut [u32],
        format: skia::PixelFormat,
        row_stride: usize,
        resident: &HashMap<TileId, Arc<CachedTile>>,
        raster_damage_pixels: usize,
        render_pass_outputs: &HashMap<viz::RenderPassId, &[u32]>,
    ) -> io::Result<RenderUpdate> {
        let scale = plan.scale;
        let timing = (std::env::var_os("LAYOUTNG_LAYER_PROFILE").is_some()
            || std::env::var_os("LAYOUTNG_LAYER_REPLAY_DIAGNOSTICS").is_some())
        .then(std::time::Instant::now);
        if width == 0 || !scale.is_finite() || scale <= 0.0 || !cfg!(target_endian = "little") {
            return Err(io::Error::other("unsupported compositor target"));
        }
        let expected = row_stride
            .checked_mul(height as usize)
            .filter(|&n| n > 0 && n <= isize::MAX as usize / 4)
            .ok_or_else(|| io::Error::other("invalid compositor target storage extent"))?;
        if row_stride < width as usize || row_stride > u32::MAX as usize || buffer.len() != expected
        {
            return Err(io::Error::other("invalid compositor target storage extent"));
        }
        let expected_viewport = PaintRect {
            x: 0.0,
            y: 0.0,
            width: width as f64 / scale,
            height: height as f64 / scale,
        };
        // Target allocation and aggregation reconstruct the same CSS-space
        // viewport through a different floating-point operation order. Keep
        // the contract strict while accepting the one-ULP drift produced by
        // non-integral zoom and DPR values.
        let nearly_equal = |a: f64, b: f64| {
            a.is_finite()
                && b.is_finite()
                && (a - b).abs() <= f64::EPSILON * 8.0 * a.abs().max(b.abs()).max(1.0)
        };
        if !nearly_equal(plan.viewport.x, expected_viewport.x)
            || !nearly_equal(plan.viewport.y, expected_viewport.y)
            || !nearly_equal(plan.viewport.width, expected_viewport.width)
            || !nearly_equal(plan.viewport.height, expected_viewport.height)
        {
            return Err(io::Error::other(
                "aggregated render-pass viewport does not match target",
            ));
        }
        let child_extent = (width as usize)
            .checked_mul(height as usize)
            .ok_or_else(|| io::Error::other("child render-pass output is too large"))?;
        for layer in plan
            .layers
            .iter()
            .filter(|layer| layer.render_pass.is_some())
            .filter(|layer| effect_opacity_has_output(layer.properties.effect.opacity))
        {
            if layer.properties.effect.is_mask {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "mask-effect-cannot-own-render-pass-quad",
                ));
            }
            let render_pass = layer.render_pass.unwrap();
            if render_pass_outputs
                .get(&render_pass)
                .is_none_or(|pixels| pixels.len() != child_extent)
            {
                return Err(io::Error::other(
                    "render-pass quad references unavailable child output",
                ));
            }
        }
        let replay_trace = browser_tracing::span("render", "ValidateAggregatedRenderPass");
        if plan
            .layers
            .iter()
            .flat_map(|layer| &layer.tiles)
            .any(|tile| {
                device_coordinate(tile.tile_rect.x * tile.raster_scale).is_err()
                    || device_coordinate(tile.tile_rect.y * tile.raster_scale).is_err()
            })
        {
            return Err(io::Error::other("invalid tile pixel placement"));
        }
        let replay_done = timing.map(|start| start.elapsed());
        drop(replay_trace);
        let staged: HashMap<TileId, Arc<CachedTile>> = HashMap::new();
        let (effect_plans, effect_bounds) = effect_allocation_plan(plan, width, height, scale)?;
        let mut compose_damage = plan.root_damage;
        if let Some(mut damage) = compose_damage {
            // A backdrop output depends on parent pixels in the filter input
            // outset. Damage to any sampled pixel invalidates the complete
            // clipped backdrop output, even when that output lies beyond the
            // original root damage rectangle.
            let mut changed = true;
            while changed {
                changed = false;
                for (layer, effect_plan) in plan.layers.iter().zip(&effect_plans) {
                    for scope in &effect_plan.chain {
                        let Scope::Effect(effect) = scope else {
                            continue;
                        };
                        if effect.backdrop_filters.is_empty()
                            && !effect.backdrop_filter_has_reference
                        {
                            continue;
                        }
                        let output = backdrop_output_bounds(
                            effect,
                            width,
                            height,
                            scale,
                            layer.compositor_scroll,
                        )?;
                        let filters = backdrop_filters(scope, scale, layer.compositor_scroll)?;
                        let input = expanded_for_filters(output, &filters, width, height);
                        if input
                            .and_then(|input| intersect_bounds(input, damage))
                            .is_some()
                        {
                            if let Some(output) = output {
                                let before = damage;
                                damage = SurfaceBounds {
                                    left: damage.left.min(output.left),
                                    top: damage.top.min(output.top),
                                    right: damage.right.max(output.right),
                                    bottom: damage.bottom.max(output.bottom),
                                };
                                if damage != before {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
            compose_damage = plan
                .output_bounds
                .and_then(|output| intersect_bounds(damage, output));
        }
        let damage_pixels = compose_damage.map_or(0, |bounds| {
            (bounds.right - bounds.left) * (bounds.bottom - bounds.top)
        });
        let raster_done = replay_done;

        let observe_costs = timing.is_some() || browser_tracing::enabled();
        let mut compose_trace = browser_tracing::span("compose", "ComposeTiles");
        compose_trace.set("raster_damage_pixels", raster_damage_pixels as f64);
        compose_trace.set("output_damage_pixels", damage_pixels as f64);
        let Some(compose_damage) = compose_damage else {
            compose_trace.set("retained_root_pass", 1.0);
            self.stats.raster_tasks = plan.raster_task_count;
            self.stats.reused_tiles = plan.layers.iter().map(|layer| layer.tiles.len()).sum();
            return Ok(RenderUpdate {
                mode: "layer-tiles",
                reason: "retained-root-pass",
                damage_pixels: 0,
                copied_bytes: 0,
            });
        };
        let mut preflight_trace = browser_tracing::span("compose", "ComposePreflight");
        let white = if plan.transparent_root {
            0
        } else if format == skia::PixelFormat::Bgrx8888 {
            0x00ff_ffff
        } else {
            u32::MAX
        };
        preflight_trace.set("effect_runs", effect_bounds.len() as f64);
        let direct_clips = layer_direct_clip::eligible_runs(plan, &effect_plans, width, height)?;
        let mut visible_rects = layer_occlusion::visible_rects(
            plan,
            &effect_plans,
            &direct_clips,
            &staged,
            resident,
            width,
            height,
        )?;
        for layer in &mut visible_rects {
            for bounds in layer {
                *bounds = bounds.and_then(|bounds| intersect_bounds(bounds, compose_damage));
            }
        }
        let clip_observations = self.clip_masks.observations();
        let prepared = layer_composition_preflight::prepare(
            plan,
            &effect_plans,
            &effect_bounds,
            &direct_clips,
            &visible_rects,
            resident,
            &mut self.clip_masks,
            width,
            height,
            scale,
            observe_costs,
        )?;
        let clip_after = self.clip_masks.observations();
        preflight_trace.set("clip_prepare_ms", prepared.clip_time.as_secs_f64() * 1000.0);
        for (index, name) in [
            "clip_cache_hits",
            "clip_relocations",
            "clip_rasters",
            "clip_raster_pixels",
        ]
        .into_iter()
        .enumerate()
        {
            preflight_trace.set(
                name,
                clip_after[index].saturating_sub(clip_observations[index]) as f64,
            );
        }
        let frame_view = TargetView {
            origin: (0, 0),
            stride: row_stride,
        };
        // SoftwareRenderer leaves an opaque root framebuffer uncleared. Here
        // skip only when the actual first white-backed layer proves every
        // visible pixel will be overwritten, including fractional neighbors.
        let full_damage = compose_damage
            == (SurfaceBounds {
                left: 0,
                top: 0,
                right: width as usize,
                bottom: height as usize,
            });
        let root_clear_skipped = if full_damage && !plan.transparent_root {
            if let (Some(layer), Some(effects)) = (plan.layers.first(), effect_plans.first()) {
                let composition = layer.composition;
                composition.white_backing
                    && effects.chain.is_empty()
                    && layer.tiles.iter().all(|tile| {
                        staged
                            .get(&tile.tile_id)
                            .or_else(|| resident.get(&tile.tile_id))
                            .is_some_and(|pixels| {
                                pixels.matches(tile, composition.white_backing)
                                    && pixels.row_support.opaque
                            })
                    })
                    && layer_tile_partition(layer, composition, width, height)?.is_some_and(
                        |rectangles| partition_covers_viewport(&rectangles, width, height),
                    )
            } else {
                false
            }
        } else {
            false
        };
        drop(preflight_trace);
        let mut draw_trace = browser_tracing::span("compose", "ComposeDraw");
        let (mut integral_tiles, mut fractional_tiles) = (0usize, 0usize);
        // viz::SoftwareRenderer::BeginDrawingRenderPass draws the root pass
        // into OutputDevice::BeginPaint's canvas. All fallible admission above
        // precedes this point; keep only tile and transient effect resources.
        if root_clear_skipped {
            // Padding is observable to callers but no tile writes it.
            if row_stride > width as usize {
                for row in buffer.chunks_exact_mut(row_stride) {
                    row[width as usize..].fill(white);
                }
            }
        } else {
            for y in compose_damage.top..compose_damage.bottom {
                buffer[frame_view.row_range(y, compose_damage.left, compose_damage.right)]
                    .fill(white);
            }
        }
        let mut reused = 0usize;
        let mut groups: Vec<EffectSurface> = Vec::new();
        let mut suspended_groups: HashMap<usize, EffectSurface> = HashMap::new();
        let mut compose_costs = observe_costs.then(|| ComposeCosts {
            clip: prepared.clip_time,
            ..Default::default()
        });
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::begin();
        for (layer_index, (((layer, effect_plan), layer_visible), direct_clip)) in plan
            .layers
            .iter()
            .zip(effect_plans)
            .zip(visible_rects)
            .zip(direct_clips)
            .enumerate()
        {
            if layer.render_pass.is_some()
                && !effect_opacity_has_output(layer.properties.effect.opacity)
            {
                // The software output is N32. Do not request a child backing
                // whose final group opacity cannot contribute a single bit.
                // Renderer prunes the same quad from the reachable pass DAG.
                continue;
            }
            if layer.properties.effect.is_mask {
                if let Some(cost) = &mut compose_costs {
                    cost.mask_layers += 1;
                    cost.mask_raster_tasks += layer.tiles.len();
                }
            }
            // Keep the resident layer's identity, but a layer with no visible
            // tiles contributes no pixels or group coverage. Do not open a
            // full-size opacity/clip surface for offscreen animation layers.
            if !layer.has_drawable_content()
                && !layer.properties.effect.is_mask
                && !effect_plan.has_backdrop()
            {
                continue;
            }
            let direct_mask = effect_plan.direct_mask;
            let chain = effect_plan.chain;
            let common = groups
                .iter()
                .zip(&chain)
                .take_while(|(active, wanted)| active.scope.same_node(wanted))
                .count();
            debug_assert!(groups
                .iter()
                .zip(&chain)
                .take(common)
                .all(|(active, wanted)| active.scope.same_values(wanted)));
            if chain
                .iter()
                .any(|scope| matches!(scope, Scope::Effect(effect) if effect.opacity == 0.0))
            {
                if let Some(cost) = &mut compose_costs {
                    cost.skipped_zero += 1;
                }
                continue;
            }
            debug_assert_eq!(chain.len() - common, effect_plan.opened_runs.len());
            let close_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
            while groups.len() > common {
                let run = groups.last().expect("active effect group").run;
                if prepared.run_last_use[run] >= layer_index {
                    let group = groups.pop().expect("active effect group");
                    debug_assert!(suspended_groups.insert(run, group).is_none());
                } else {
                    self.finish_effect_group(
                        &mut groups,
                        buffer,
                        frame_view,
                        format,
                        compose_damage,
                    );
                }
            }
            if let Some(cost) = &mut compose_costs {
                cost.close += close_start.unwrap().elapsed();
            }
            let open_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
            for (scope, run) in chain.into_iter().skip(common).zip(effect_plan.opened_runs) {
                if let Some(cost) = &mut compose_costs {
                    cost.scopes += 1;
                }
                if let Some(group) = suspended_groups.remove(&run) {
                    debug_assert!(group.scope.same_node(&scope));
                    debug_assert!(group.scope.same_values(&scope));
                    groups.push(group);
                    continue;
                }
                let bounds = effect_bounds[run].unwrap_or(SurfaceBounds {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                });
                let coverage = prepared.coverage[run].clone();
                let scratch_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
                let view = TargetView {
                    origin: (bounds.left, bounds.top),
                    stride: bounds.right - bounds.left,
                };
                let len = view
                    .stride
                    .checked_mul(bounds.bottom - bounds.top)
                    .expect("preflight validated effect dimensions");
                let direct_dst_in = direct_mask.is_some() && matches!(&scope, Scope::DstIn(_));
                let dst_in_root = matches!(&scope, Scope::DstIn(effect)
                    if plan.owner_effect.as_ref().is_some_and(|owner| effect.parent.as_ref()
                        .is_some_and(|parent| parent.lifecycle.same_node(&owner.lifecycle))));
                let direct_clip = direct_clip.is_some_and(|candidate| candidate.opening_run == run);
                let filters = effect_filters(&scope, scale, true, layer.compositor_scroll)?;
                let backdrop_filters = backdrop_filters(&scope, scale, layer.compositor_scroll)?;
                let backdrop_output = match &scope {
                    Scope::Effect(effect) => backdrop_output_bounds(
                        effect,
                        width,
                        height,
                        scale,
                        layer.compositor_scroll,
                    )?,
                    Scope::Clip(_) | Scope::DstIn(_) => None,
                };
                let backdrop_coverage = match (&scope, backdrop_output) {
                    (Scope::Effect(effect), Some(_)) => effect
                        .backdrop_filter_clip
                        .as_ref()
                        .map(|clip| {
                            self.clip_masks.coverage(
                                &clip_chain(clip),
                                width,
                                height,
                                scale,
                                layer.compositor_scroll,
                            )
                        })
                        .transpose()?,
                    _ => None,
                };
                let pixels = if direct_dst_in || direct_clip {
                    Vec::new()
                } else {
                    self.cleared_effect_surface(len)
                };
                if let Some(cost) = &mut compose_costs {
                    cost.scratch += scratch_start.unwrap().elapsed();
                }
                groups.push(EffectSurface {
                    run,
                    scope,
                    coverage,
                    direct_dst_in,
                    direct_clip,
                    dst_in_root,
                    pixels,
                    view,
                    touched: None,
                    filters,
                    backdrop_filters,
                    backdrop_output,
                    backdrop_coverage,
                    visible_polygon: None,
                });
            }
            if let Some(cost) = &mut compose_costs {
                cost.open += open_start.unwrap().elapsed();
            }
            if let Some(candidate) = direct_clip {
                debug_assert!(
                    candidate.scope_index + 1 == groups.len()
                        && groups.last().is_some_and(|g| g.direct_clip)
                );
                if let Some(cost) = &mut compose_costs {
                    cost.direct_clip_layers += 1;
                }
            }
            let composition = layer.composition;
            let blit_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
            if let Some(render_pass_id) = layer.render_pass {
                let mut quad_trace = browser_tracing::span("compose", "ComposeRenderPassQuad");
                quad_trace.set("render_pass_id", render_pass_id.0 as f64);
                quad_trace.set(
                    "visible_polygon",
                    layer.visible_polygon.is_some() as u8 as f64,
                );
                quad_trace.set("source_pixels", f64::from(width) * f64::from(height));
                quad_trace.set(
                    "content_bounds_known",
                    composition.content_bounds.is_some() as u8 as f64,
                );
                let source = render_pass_outputs.get(&render_pass_id).ok_or_else(|| {
                    io::Error::other("render-pass quad references unavailable child output")
                })?;
                let expected = (width as usize)
                    .checked_mul(height as usize)
                    .ok_or_else(|| io::Error::other("child render-pass output is too large"))?;
                if source.len() != expected {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "invalid-effect-render-pass-output",
                    ));
                }
                let owner = layer.properties.effect.clone();
                let scope = Scope::Effect(owner.clone());
                let filters = effect_filters(&scope, scale, true, layer.compositor_scroll)?;
                let backdrop_filters = backdrop_filters(&scope, scale, layer.compositor_scroll)?;
                let borrows_source = filters.is_empty()
                    && backdrop_filters.is_empty()
                    && owner.backdrop_filter_clip.is_none();
                let output_clips = owner
                    .output_clip
                    .as_ref()
                    .map(clip_chain)
                    .unwrap_or_default()
                    .into_iter()
                    // The compositor already supplied the enclosing scissor.
                    // Raster coverage is only for rounded/path/transformed clips.
                    .filter(|clip| !borrows_source || needs_clip_mask(clip, scale))
                    .collect::<Vec<_>>();
                let coverage = (!output_clips.is_empty())
                    .then(|| {
                        self.clip_masks.coverage(
                            &output_clips,
                            width,
                            height,
                            scale,
                            layer.compositor_scroll,
                        )
                    })
                    .transpose()?;
                quad_trace.set(
                    "clip_mask_pixels",
                    coverage.as_ref().map_or(0, |mask| mask.pixels.len()) as f64,
                );
                let backdrop_output =
                    backdrop_output_bounds(&owner, width, height, scale, layer.compositor_scroll)?;
                let backdrop_coverage = match (&owner.backdrop_filter_clip, backdrop_output) {
                    (Some(clip), Some(_)) => Some(self.clip_masks.coverage(
                        &clip_chain(clip),
                        width,
                        height,
                        scale,
                        layer.compositor_scroll,
                    )?),
                    _ => None,
                };
                if borrows_source {
                    // The child backing is immutable and already contains its
                    // subtree's mask/effects. Like SoftwareRenderer's bitmap
                    // shader, sample it directly rather than copying a whole
                    // viewport into a transient saveLayer for every quad.
                    let source_view = TargetView {
                        origin: (0, 0),
                        stride: width as usize,
                    };
                    let touched = render_pass_quad_bounds(composition, width, height);
                    quad_trace.set("borrowed_source", 1.0);
                    quad_trace.set(
                        "draw_pixels",
                        touched.map_or(0, |b| (b.right - b.left) * (b.bottom - b.top)) as f64,
                    );
                    if let Some(parent) = groups.last_mut() {
                        let written = composite_effect_surface(
                            &mut parent.pixels,
                            source,
                            touched,
                            parent.view,
                            source_view,
                            skia::PixelFormat::Rgba8888,
                            owner.opacity,
                            owner.blend_mode,
                            coverage.as_deref(),
                            layer.visible_polygon.as_ref(),
                        );
                        union_bounds(&mut parent.touched, written);
                    } else {
                        composite_effect_surface(
                            buffer,
                            source,
                            touched.and_then(|b| intersect_bounds(b, compose_damage)),
                            frame_view,
                            source_view,
                            format,
                            owner.opacity,
                            owner.blend_mode,
                            coverage.as_deref(),
                            layer.visible_polygon.as_ref(),
                        );
                    }
                    if let Some(cost) = &mut compose_costs {
                        cost.blit += blit_start.unwrap().elapsed();
                    }
                    continue;
                }
                let quad_bounds = render_pass_quad_bounds(composition, width, height);
                quad_trace.set(
                    "quad_pixels",
                    quad_bounds.map_or(0, |b| (b.right - b.left) * (b.bottom - b.top)) as f64,
                );
                // RenderSurfaceImpl keeps a tight content_rect backing. Its
                // filters are evaluated while the parent draws the render-pass
                // quad, but that does not turn the filter input into a
                // viewport-sized bitmap. Reserve only the raw quad plus the
                // pixels an image/backdrop filter can read or produce.
                let mut storage_bounds =
                    expanded_for_filters(quad_bounds, &filters, width, height);
                if let Some(output) = backdrop_output {
                    union_bounds(
                        &mut storage_bounds,
                        expanded_for_filters(Some(output), &backdrop_filters, width, height),
                    );
                }
                let Some(storage_bounds) = storage_bounds else {
                    if let Some(cost) = &mut compose_costs {
                        cost.blit += blit_start.unwrap().elapsed();
                    }
                    continue;
                };
                let storage_width = storage_bounds.right - storage_bounds.left;
                let storage_height = storage_bounds.bottom - storage_bounds.top;
                let storage_len = storage_width
                    .checked_mul(storage_height)
                    .ok_or_else(|| io::Error::other("render-pass effect surface is too large"))?;
                quad_trace.set("storage_pixels", storage_len as f64);
                let copy_started = std::time::Instant::now();
                let mut pixels = vec![0; storage_len];
                if let Some(input) = quad_bounds.and_then(|input| {
                    intersect_bounds(input, storage_bounds)
                }) {
                    for y in input.top..input.bottom {
                        let source_start = y * width as usize + input.left;
                        let destination_start =
                            (y - storage_bounds.top) * storage_width
                                + input.left
                                - storage_bounds.left;
                        let columns = input.right - input.left;
                        pixels[destination_start..destination_start + columns]
                            .copy_from_slice(&source[source_start..source_start + columns]);
                    }
                }
                quad_trace.set(
                    "source_copy_ms",
                    copy_started.elapsed().as_secs_f64() * 1000.0,
                );
                groups.push(EffectSurface {
                    run: usize::MAX,
                    scope: scope.clone(),
                    direct_dst_in: false,
                    direct_clip: false,
                    dst_in_root: false,
                    coverage,
                    pixels,
                    view: TargetView {
                        origin: (storage_bounds.left, storage_bounds.top),
                        stride: storage_width,
                    },
                    touched: quad_bounds,
                    filters,
                    backdrop_filters,
                    backdrop_output,
                    backdrop_coverage,
                    visible_polygon: layer.visible_polygon.clone(),
                });
                self.finish_effect_group(&mut groups, buffer, frame_view, format, compose_damage);
                if let Some(cost) = &mut compose_costs {
                    cost.blit += blit_start.unwrap().elapsed();
                }
                continue;
            }
            if let Some(rectangles) = &direct_mask {
                let index = groups
                    .len()
                    .checked_sub(2)
                    .expect("preflight validated direct mask destination");
                let expected = layer
                    .properties
                    .effect
                    .parent
                    .as_ref()
                    .expect("preflight validated direct mask parent");
                debug_assert!(
                    groups.last().is_some_and(|group| group.direct_dst_in)
                        && matches!(&groups[index].scope, Scope::Effect(parent)
                        if parent.lifecycle.same_node(&expected.lifecycle))
                );
                let parent = &mut groups[index];
                parent.touched = clear_mask_exterior(
                    &mut parent.pixels,
                    parent.view,
                    parent.touched,
                    rectangles,
                );
                if let Some(cost) = &mut compose_costs {
                    cost.direct_mask_layers += 1;
                }
            }
            if composition.projective.is_some() || !composition.transform.is_translation() {
                debug_assert!(direct_mask.is_none());
                let mut sources = Vec::with_capacity(layer.tiles.len());
                for tile in &layer.tiles {
                    let pixels = staged
                        .get(&tile.tile_id)
                        .or_else(|| resident.get(&tile.tile_id))
                        .expect("preflight validated affine tile");
                    debug_assert!(pixels.matches(tile, composition.white_backing));
                    if !valid_tile_storage(pixels, tile.pixel_size)
                        .expect("preflight validated affine tile storage")
                    {
                        unreachable!("preflight validated affine tile storage");
                    }
                    if !staged.contains_key(&tile.tile_id) && pixels.raster_frame != plan.frame_id {
                        reused += 1;
                    }
                    sources.push(AffineTileSource {
                        tile,
                        pixels,
                        origin: (
                            device_coordinate(tile.tile_rect.x * tile.raster_scale)
                                .expect("preflight validated affine x"),
                            device_coordinate(tile.tile_rect.y * tile.raster_scale)
                                .expect("preflight validated affine y"),
                        ),
                        content_bounds: layer.raster_content_bounds.map(|bounds| PaintRect {
                            x: bounds.x * plan.scale,
                            y: bounds.y * plan.scale,
                            width: bounds.width * plan.scale,
                            height: bounds.height * plan.scale,
                        }),
                    });
                }
                let mut affine_bounds = None;
                for visible in layer_visible.iter().copied() {
                    union_bounds(&mut affine_bounds, visible);
                }
                let touched = if direct_clip.is_some() {
                    let clip_index = groups.len() - 1;
                    let coverage = groups[clip_index].coverage.clone();
                    match coverage {
                        Some(coverage) if clip_index > 0 => {
                            let parent = &mut groups[clip_index - 1];
                            let touched = compose_affine_layer(
                                &mut parent.pixels,
                                parent.view,
                                skia::PixelFormat::Rgba8888,
                                composition,
                                layer.visible_polygon.as_ref(),
                                &sources,
                                affine_bounds,
                                Some(&coverage),
                            )?;
                            union_bounds(&mut parent.touched, touched);
                            None
                        }
                        Some(coverage) => compose_affine_layer(
                            buffer,
                            frame_view,
                            format,
                            composition,
                            layer.visible_polygon.as_ref(),
                            &sources,
                            affine_bounds,
                            Some(&coverage),
                        )?,
                        None => None,
                    }
                } else if let Some(group) = groups.last_mut() {
                    let touched = compose_affine_layer(
                        &mut group.pixels,
                        group.view,
                        skia::PixelFormat::Rgba8888,
                        composition,
                        layer.visible_polygon.as_ref(),
                        &sources,
                        affine_bounds,
                        None,
                    )?;
                    union_bounds(&mut group.touched, touched);
                    None
                } else {
                    compose_affine_layer(
                        buffer,
                        frame_view,
                        format,
                        composition,
                        layer.visible_polygon.as_ref(),
                        &sources,
                        affine_bounds,
                        None,
                    )?
                };
                let _ = touched;
                fractional_tiles += layer.tiles.len();
                continue;
            }
            let needs_neighbors = layer.tiles.first().is_some_and(|tile| {
                let x = tile.tile_rect.x * tile.raster_scale + composition.transform.tx;
                let y = tile.tile_rect.y * tile.raster_scale + composition.transform.ty;
                x.fract() != 0.0 || y.fract() != 0.0
            });
            let mut neighbors = HashMap::with_capacity(if needs_neighbors {
                layer.tiles.len()
            } else {
                0
            });
            for tile in layer.tiles.iter().filter(|_| needs_neighbors) {
                let pixels = staged
                    .get(&tile.tile_id)
                    .or_else(|| resident.get(&tile.tile_id))
                    .expect("preflight validated resident tile");
                neighbors.insert(tile.tile_index, pixels);
            }
            for (tile, draw_visible) in layer.tiles.iter().zip(layer_visible) {
                let origin_device = (
                    device_coordinate(tile.tile_rect.x * tile.raster_scale)
                        .expect("preflight validated tile x")
                        + composition.transform.tx,
                    device_coordinate(tile.tile_rect.y * tile.raster_scale)
                        .expect("preflight validated tile y")
                        + composition.transform.ty,
                );
                let phase = (
                    origin_device.0.ceil() - origin_device.0,
                    origin_device.1.ceil() - origin_device.1,
                );
                if let Some(cost) = &mut compose_costs {
                    if phase == (0.0, 0.0) {
                        cost.integral += 1;
                    } else {
                        cost.fractional += 1;
                        let phases = [phase.0, phase.1];
                        let exact = |scale: f64| {
                            phases.iter().all(|phase| {
                                let weight = phase * scale;
                                weight.is_finite() && weight == weight.round()
                            })
                        };
                        cost.fractional_phase_classes[if exact(256.0) {
                            0
                        } else if exact(65536.0) {
                            1
                        } else {
                            2
                        }] += 1;
                    }
                    cost.nominal_pixels += tile.pixel_size.0 as usize * tile.pixel_size.1 as usize;
                }
                let adjacent = [
                    neighbors
                        .get(&(tile.tile_index.0 + 1, tile.tile_index.1))
                        .copied(),
                    neighbors
                        .get(&(tile.tile_index.0, tile.tile_index.1 + 1))
                        .copied(),
                    neighbors
                        .get(&(tile.tile_index.0 + 1, tile.tile_index.1 + 1))
                        .copied(),
                ]
                .map(|resource| resource.map(Arc::as_ref));
                let pixels = if let Some(pixels) = staged.get(&tile.tile_id) {
                    pixels
                } else {
                    let cached = resident
                        .get(&tile.tile_id)
                        .expect("preflight validated resident tile");
                    if cached.raster_frame != plan.frame_id {
                        reused += 1;
                        if layer.properties.effect.is_mask {
                            if let Some(cost) = &mut compose_costs {
                                cost.mask_reused_tiles += 1;
                            }
                        }
                    }
                    resident
                        .get(&tile.tile_id)
                        .expect("preflight validated resident tile")
                };
                debug_assert!(pixels.matches(tile, composition.white_backing));
                if let Some(cost) = &mut compose_costs {
                    if let Some((original, _, _)) =
                        tile_composition_geometry(tile, composition, width, height)
                            .expect("preflight validated tile geometry")
                    {
                        let original_area =
                            (original.right - original.left) * (original.bottom - original.top);
                        let visible_area =
                            draw_visible.map_or(0, |b| (b.right - b.left) * (b.bottom - b.top));
                        cost.occluded_pixels += original_area - visible_area;
                        if draw_visible.is_none() {
                            cost.occluded_tiles += 1;
                        }
                    }
                }
                // Keep resource/generation accounting even for hidden quads.
                // Mask quads retain their original partition and exterior clear.
                if draw_visible.is_none() {
                    continue;
                }
                // Count quads that reach a visible target, after occlusion and
                // empty direct-clip rejection. No per-pixel instrumentation.
                if direct_clip.is_none() || groups.last().is_some_and(|g| g.coverage.is_some()) {
                    if needs_neighbors {
                        fractional_tiles += 1;
                    } else {
                        integral_tiles += 1;
                    }
                }
                let tile_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
                let tile_class = if direct_mask.is_some() {
                    3
                } else if layer_solid::color_for_sampling(pixels, adjacent, phase).is_some() {
                    4
                } else if phase == (0.0, 0.0) {
                    0
                } else {
                    let horizontal = phase.0 != 0.0;
                    let vertical = phase.1 != 0.0;
                    let opaque = draw_visible
                        .is_some_and(|bounds| opaque_covers(composition, bounds))
                        || ((composition.white_backing || pixels.row_support.opaque)
                            && adjacent
                                .iter()
                                .zip([horizontal, vertical, horizontal && vertical])
                                .all(|(next, used)| {
                                    !used
                                        || next.is_none_or(|next| {
                                            next.white_backing || next.row_support.opaque
                                        })
                                }));
                    if opaque {
                        1
                    } else {
                        2
                    }
                };
                #[cfg(feature = "compose_work_profile")]
                {
                    layer_compose_profile::class(tile_class);
                    if let Some(b) = draw_visible {
                        layer_compose_profile::quad((b.right - b.left) * (b.bottom - b.top));
                    }
                }
                if direct_mask.is_some() {
                    let index = groups.len() - 2;
                    let parent = &mut groups[index];
                    dst_in_tile(
                        &mut parent.pixels,
                        parent.view,
                        parent.touched,
                        width,
                        height,
                        tile,
                        composition,
                        pixels,
                        adjacent,
                    )
                    .expect("preflight validated direct mask tile");
                } else if direct_clip.is_some() {
                    let clip_index = groups.len() - 1;
                    let coverage = groups[clip_index].coverage.clone();
                    // An empty actual clip draws nothing. Keep its scope
                    // identity but write directly to the original parent.
                    if let Some(coverage) = coverage {
                        if clip_index > 0 {
                            let parent = &mut groups[clip_index - 1];
                            let touched = layer_clip_quad::compose_clipped_tile(
                                &mut parent.pixels,
                                width,
                                height,
                                parent.view,
                                skia::PixelFormat::Rgba8888,
                                tile,
                                composition,
                                pixels,
                                adjacent,
                                draw_visible,
                                &coverage,
                            )
                            .expect("preflight validated clipped tile");
                            union_bounds(&mut parent.touched, touched);
                        } else {
                            layer_clip_quad::compose_clipped_tile(
                                buffer,
                                width,
                                height,
                                frame_view,
                                format,
                                tile,
                                composition,
                                pixels,
                                adjacent,
                                draw_visible,
                                &coverage,
                            )
                            .expect("preflight validated clipped tile");
                        }
                    }
                } else if let Some(group) = groups.last_mut() {
                    debug_assert!(!composition.white_backing);
                    let touched = compose_tile(
                        &mut group.pixels,
                        width,
                        height,
                        group.view,
                        skia::PixelFormat::Rgba8888,
                        tile,
                        composition,
                        pixels,
                        adjacent,
                        draw_visible,
                    )
                    .expect("preflight validated tile draw");
                    union_bounds(&mut group.touched, touched);
                } else {
                    compose_tile(
                        buffer,
                        width,
                        height,
                        frame_view,
                        format,
                        tile,
                        composition,
                        pixels,
                        adjacent,
                        draw_visible,
                    )
                    .expect("preflight validated tile draw");
                }
                if let Some(cost) = &mut compose_costs {
                    cost.tile_blits[tile_class] += tile_start.unwrap().elapsed();
                    if let Some(bounds) = draw_visible {
                        cost.tile_pixels[tile_class] +=
                            (bounds.right - bounds.left) * (bounds.bottom - bounds.top);
                    }
                    if tile_class == 4 {
                        cost.solid_tiles += 1;
                    }
                }
            }
            if let Some(cost) = &mut compose_costs {
                cost.blit += blit_start.unwrap().elapsed();
            }
        }
        debug_assert!(suspended_groups.is_empty());
        let close_start = compose_costs.as_ref().map(|_| std::time::Instant::now());
        while !groups.is_empty() {
            self.finish_effect_group(&mut groups, buffer, frame_view, format, compose_damage);
        }
        for quad in &plan.solid_quads {
            compose_solid_color_quad(buffer, frame_view, format, quad, scale, compose_damage);
        }
        if let Some(cost) = &mut compose_costs {
            cost.close += close_start.unwrap().elapsed();
        }
        let compose_done = timing.map(|start| start.elapsed());
        draw_trace.set("reused_tiles", reused as f64);
        draw_trace.set("integral_tiles", integral_tiles as f64);
        draw_trace.set("fractional_tiles", fractional_tiles as f64);
        draw_trace.set(
            "root_clear_skipped",
            if root_clear_skipped { 1.0 } else { 0.0 },
        );
        if let Some(cost) = &compose_costs {
            let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
            // Scratch is a subset of open. Clip preparation is preflight work.
            draw_trace.set("open_effects_ms", ms(cost.open));
            draw_trace.set("scratch_ms", ms(cost.scratch));
            draw_trace.set("blit_ms", ms(cost.blit));
            draw_trace.set("finish_effects_ms", ms(cost.close));
            browser_tracing::instant(
                "compose",
                "ComposeBlitCosts",
                &[
                    ("integral_ms", ms(cost.tile_blits[0])),
                    ("fractional_opaque_ms", ms(cost.tile_blits[1])),
                    ("fractional_blend_ms", ms(cost.tile_blits[2])),
                    ("mask_ms", ms(cost.tile_blits[3])),
                    ("solid_ms", ms(cost.tile_blits[4])),
                    ("integral_pixels", cost.tile_pixels[0] as f64),
                ],
            );
            // Trace events retain at most six numeric fields. Keep pixel work
            // separate from timings so the final classes are not truncated.
            browser_tracing::instant(
                "compose",
                "ComposeBlitPixels",
                &[
                    ("integral", cost.tile_pixels[0] as f64),
                    ("fractional_opaque", cost.tile_pixels[1] as f64),
                    ("fractional_blend", cost.tile_pixels[2] as f64),
                    ("mask", cost.tile_pixels[3] as f64),
                    ("solid", cost.tile_pixels[4] as f64),
                    ("occluded", cost.occluded_pixels as f64),
                ],
            );
            browser_tracing::instant(
                "compose",
                "ComposeEffects",
                &[
                    ("direct_clip_layers", cost.direct_clip_layers as f64),
                    ("mask_layers", cost.mask_layers as f64),
                    ("direct_mask_layers", cost.direct_mask_layers as f64),
                    ("effect_scopes", cost.scopes as f64),
                    ("finish_ms", ms(cost.close)),
                    ("scratch_ms", ms(cost.scratch)),
                ],
            );
        }
        drop(draw_trace);
        compose_trace.set("layers", self.stats.layers as f64);
        compose_trace.set("tiles", self.stats.tiles as f64);
        compose_trace.set("raster_tasks", plan.raster_task_count as f64);
        compose_trace.set("reused_tiles", reused as f64);
        compose_trace.set(
            "root_clear_skipped",
            if root_clear_skipped { 1.0 } else { 0.0 },
        );
        drop(compose_trace);
        self.stats.raster_tasks = plan.raster_task_count;
        self.stats.reused_tiles = reused;
        if let Some(start) = timing {
            let ms = |time: std::time::Duration| time.as_secs_f64() * 1000.0;
            let replay_time = replay_done.unwrap();
            let raster = raster_done.unwrap();
            let compose = compose_done.unwrap();
            eprintln!(
                "layer-tile-stages replay_ms={:.3} raster_ms={:.3} compose_ms={:.3} commit_ms={:.3} tasks={} reused={}",
                ms(replay_time),
                ms(raster - replay_time),
                ms(compose - raster),
                ms(start.elapsed() - compose),
                self.stats.raster_tasks,
                self.stats.reused_tiles
            );
            if let Some(cost) = compose_costs {
                eprintln!(
                    "layer-tile-compose-cost open_ms={:.3} clip_ms={:.3} scratch_ms={:.3} blit_ms={:.3} close_ms={:.3} scopes={} integral_tiles={} fractional_tiles={} skip_zero_layers={} nominal_tile_pixels={} phase_8bit_tiles={} phase_16bit_tiles={} phase_f64_tiles={} mask_layers={} mask_raster_tasks={} mask_reused_tiles={} direct_mask_layers={} root_clear_skipped={} occluded_tiles={} occluded_pixels={} integral_blit_ms={:.3} fractional_opaque_blit_ms={:.3} fractional_blend_blit_ms={:.3} mask_blit_ms={:.3} solid_tiles={} solid_blit_ms={:.3} direct_clip_layers={}",
                    ms(cost.open),
                    ms(cost.clip),
                    ms(cost.scratch),
                    ms(cost.blit),
                    ms(cost.close),
                    cost.scopes,
                    cost.integral,
                    cost.fractional,
                    cost.skipped_zero,
                    cost.nominal_pixels,
                    cost.fractional_phase_classes[0],
                    cost.fractional_phase_classes[1],
                    cost.fractional_phase_classes[2],
                    cost.mask_layers,
                    cost.mask_raster_tasks,
                    cost.mask_reused_tiles,
                    cost.direct_mask_layers,
                    root_clear_skipped,
                    cost.occluded_tiles,
                    cost.occluded_pixels,
                    ms(cost.tile_blits[0]),
                    ms(cost.tile_blits[1]),
                    ms(cost.tile_blits[2]),
                    ms(cost.tile_blits[3]),
                    cost.solid_tiles,
                    ms(cost.tile_blits[4]),
                    cost.direct_clip_layers
                );
            }
        }
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::end();
        Ok(RenderUpdate {
            mode: "layer-tiles",
            reason: if plan.raster_task_count == 0 {
                "reused-visible-tiles"
            } else {
                "rasterized-dirty-tiles"
            },
            damage_pixels,
            copied_bytes: 0,
        })
    }
}

fn tile_bytes(size: (u32, u32)) -> io::Result<usize> {
    (size.0 as usize)
        .checked_mul(size.1 as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|&n| size.0 > 0 && size.1 > 0 && n <= isize::MAX as usize)
        .ok_or_else(|| io::Error::other("invalid tile pixel dimensions"))
}

fn device_coordinate(value: f64) -> io::Result<f64> {
    if !value.is_finite() || value < i32::MIN as f64 || value > i32::MAX as f64 {
        return Err(io::Error::other(
            "tile has invalid or overflowing pixel placement",
        ));
    }
    Ok(value)
}

pub(crate) fn tile_row_support(rgba: &[u8], width: usize, opaque: bool) -> TileRowSupport {
    let mut support = TileRowSupport {
        rows: Vec::with_capacity(rgba.len() / (width * 4)),
        runs: Vec::new(),
        opaque: true,
        draw_mode: layer_solid::TileDrawMode::Resource,
    };
    for row in rgba.chunks_exact(width * 4) {
        if opaque {
            support.rows.push(RowSupport {
                extent: (0, width),
                opaque: true,
                runs: (0, 0),
            });
            continue;
        }
        // Most image tiles are opaque although their layer started clear.
        // Prove this once from actual pixels, instead of reading/blending the
        // destination on every cached composition. Transparent tiles stop the
        // proof at their first non-opaque row.
        if support.opaque {
            if row.chunks_exact(4).all(|pixel| pixel[3] == 255) {
                support.rows.push(RowSupport {
                    extent: (0, width),
                    opaque: true,
                    runs: (0, 0),
                });
                continue;
            }
            support.opaque = false;
        }
        let nonzero = |pixel: &[u8]| pixel != [0, 0, 0, 0];
        let Some(left) = row.chunks_exact(4).position(nonzero) else {
            support.rows.push(RowSupport {
                extent: (0, 0),
                opaque: false,
                runs: (0, 0),
            });
            continue;
        };
        let right = row.chunks_exact(4).rposition(nonzero).unwrap() + 1;
        let run_start = support.runs.len();
        let mut row_opaque = row[left * 4 + 3] == 255;
        let mut span_start = left;
        let mut previous = left;
        // Cache only long holes. Short glyph/AA holes stay in the SIMD span,
        // avoiding an allocation or kernel call per small coverage fragment.
        if right - left > 17 {
            for (x, pixel) in row.chunks_exact(4).enumerate().take(right).skip(left + 1) {
                // Reuse the existing product classification scan. A zero hole
                // inside this extent also rejects Src: copying zero must not
                // erase the underlying destination.
                row_opaque &= pixel[3] == 255;
                if nonzero(pixel) {
                    if x - previous - 1 >= 16 {
                        support.runs.push((span_start, previous + 1));
                        span_start = x;
                    }
                    previous = x;
                }
            }
        } else {
            row_opaque &= row[left * 4..right * 4]
                .chunks_exact(4)
                .all(|pixel| pixel[3] == 255);
        }
        if support.runs.len() != run_start {
            support.runs.push((span_start, right));
        }
        support.rows.push(RowSupport {
            extent: (left, right),
            opaque: row_opaque,
            runs: (run_start, support.runs.len()),
        });
    }
    support
}

// Merge real source support without allocating per composed row. Horizontal
// sampling reaches x+1; a narrow lower tile clamps its last real pixel through
// the rest of the upper tile. The true right edge still uses tile_edge_pixel.
fn for_fractional_spans<const H: bool>(
    upper: &[(usize, usize)],
    lower: &[(usize, usize)],
    width: usize,
    lower_width: usize,
    mut visit: impl FnMut(usize, usize),
) {
    let normalize = |&(left, right): &(usize, usize), clamp: bool| {
        let left = if H { left.saturating_sub(1) } else { left };
        let right = if clamp && right == lower_width {
            width
        } else {
            right.min(width)
        };
        (left < right).then_some((left, right))
    };
    let mut upper = upper
        .iter()
        .filter_map(|span| normalize(span, false))
        .peekable();
    let mut lower = lower
        .iter()
        .filter_map(|span| normalize(span, true))
        .peekable();
    let mut merged: Option<(usize, usize)> = None;
    loop {
        let next = match (upper.peek(), lower.peek()) {
            (Some(a), Some(b)) if a.0 <= b.0 => upper.next(),
            (Some(_), Some(_)) => lower.next(),
            (Some(_), None) => upper.next(),
            (None, Some(_)) => lower.next(),
            (None, None) => break,
        }
        .unwrap();
        merged = Some(match merged {
            Some(old) if next.0 <= old.1 => (old.0, old.1.max(next.1)),
            Some(old) => {
                visit(old.0, old.1);
                next
            }
            None => next,
        });
    }
    if let Some((left, right)) = merged {
        visit(left, right);
    }
}

#[cfg(test)]
mod sparse_support_test {
    use super::*;

    #[test]
    fn opaque_row_src_preserves_transparent_margins_holes_and_aa() {
        let (width, height) = (19u32, 6u32);
        let mut rgba = vec![0; width as usize * height as usize * 4];
        for y in 1..6usize {
            for x in 3..16usize {
                let a = if y == 3 && x == 3 { 127 } else { 255 };
                let at = (y * width as usize + x) * 4;
                rgba[at..at + 4].copy_from_slice(&[a / 3, a / 2, a, a]);
            }
        }
        // A short zero hole stays within the dense support extent. Its row
        // must keep SrcOver so copying zero cannot erase the parent surface.
        rgba[(2 * width as usize + 9) * 4..(2 * width as usize + 9) * 4 + 4].fill(0);
        let support = tile_row_support(&rgba, width as usize, false);
        assert!(!support.opaque);
        assert!(support.rows[1].opaque && support.rows[4].opaque);
        assert!(!support.rows[2].opaque && !support.rows[3].opaque);
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: width as f64,
            height: height as f64,
        };
        let tile = TilePlacement {
            tile_id: TileId(1),
            generation: 1,
            tile_index: (0, 0),
            tiling_rect: rect,
            tile_rect: rect,
            raster_scale: 1.0,
            pixel_size: (width, height),
            ready: true,
        };
        let pixels = CachedTile {
            raster_frame: 0,
            generation: 1,
            pixel_size: (width, height),
            raster_scale: 1.0,
            white_backing: false,
            rgba,
            row_support: support,
        };
        let composition = LayerComposition {
            transform: viz::AffineTransform {
                tx: 0.0,
                ty: 0.0,
                ..viz::AffineTransform::IDENTITY
            },
            projective: None,
            rendering_context_id: 0,
            clip: None,
            content_bounds: None,
            opaque_bounds: None,
            white_backing: false,
            grid_min: None,
        };
        let view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            let initial = if format == skia::PixelFormat::Bgrx8888 {
                0x00714a25
            } else {
                0xb8714a25
            };
            let mut actual = vec![initial; width as usize * height as usize];
            let mut expected = actual.clone();
            let expected_bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    expected.as_mut_ptr().cast::<u8>(),
                    expected.len() * 4,
                )
            };
            skia::blend_premultiplied_rgba_row(expected_bytes, &pixels.rgba, format);
            compose_tile(
                &mut actual,
                width,
                height,
                view,
                format,
                &tile,
                composition,
                &pixels,
                [None; 3],
                None,
            )
            .unwrap();
            assert_eq!(actual, expected, "opaque row Src format={format:?}");
        }
    }

    #[test]
    fn fractional_sparse_row_matches_dense_sampler_with_clamped_neighbor() {
        let tile = |row: Vec<u8>| {
            let width = row.len() / 4;
            CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: (width as u32, 1),
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&row, width, false),
                rgba: row,
                bgra: Vec::new(),
            }
        };
        let mut upper = vec![0; 96 * 4];
        for x in [0, 1, 35, 36, 82, 83, 95] {
            upper[x * 4..x * 4 + 4].copy_from_slice(&[40, 20, 10, 90]);
        }
        // RGB also participates in the exact-zero proof.
        upper[35 * 4..35 * 4 + 4].copy_from_slice(&[1, 0, 0, 0]);
        let mut lower = vec![0; 64 * 4];
        for x in [2, 3, 37, 38, 63] {
            lower[x * 4..x * 4 + 4].copy_from_slice(&[10, 30, 50, 120]);
        }
        let upper = tile(upper);
        let lower = tile(lower);
        let right = tile(vec![30, 10, 20, 80]);
        let phase = (0.375, 0.3125);
        let mut expected = vec![0xb8734a25; 96];
        let mut actual = expected.clone();
        for (x, dst) in expected.iter_mut().enumerate() {
            let a = rgba_word(&upper.rgba, x);
            let b = if x + 1 == 96 {
                rgba_word(&right.rgba, 0)
            } else {
                rgba_word(&upper.rgba, x + 1)
            };
            let c = rgba_word(&lower.rgba, x.min(63));
            // Missing diagonal neighbor clamps to the original tile, matching
            // the original complete row sampler's actual edge selection.
            let d = if x + 1 == 96 {
                rgba_word(&upper.rgba, 95)
            } else {
                rgba_word(&lower.rgba, (x + 1).min(63))
            };
            store_linear::<false>(
                dst,
                linear_word::<true, true>(a, b, c, d, phase, (1.0 - phase.0, 1.0 - phase.1)),
                skia::PixelFormat::Bgrx8888,
            );
        }
        let mut sampled = [false; 96];
        for_fractional_spans::<true>(
            upper.row_support.spans(0),
            lower.row_support.spans(0),
            96,
            64,
            |left, right| sampled[left..right].fill(true),
        );
        assert!(!sampled[10]);
        assert!(sampled[35] && sampled[80]);
        assert!(fractional_row::<true, true, false, false>(
            &mut actual,
            &upper,
            [Some(&right), Some(&lower), None],
            0,
            0,
            phase,
            (1.0 - phase.0, 1.0 - phase.1),
            skia::PixelFormat::Bgrx8888
        )
        .is_some());
        assert_eq!(actual, expected);
    }

    #[test]
    fn fractional_opaque_rows_match_scalar_srcover_with_aa_holes_and_neighbors() {
        let tile = |width: usize, height: usize| {
            let rgba: Vec<u8> = (0..height)
                .flat_map(|y| {
                    (0..width).flat_map(move |x| {
                        let a = if x < 3 || x + 3 >= width {
                            0
                        } else if y == 2 && x == 9 {
                            0
                        } else if y == 3 && x == 3 {
                            127
                        } else {
                            255
                        };
                        [a / 3, a / 2, a, a]
                    })
                })
                .collect();
            CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: (width as u32, height as u32),
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&rgba, width, false),
                rgba,
                bgra: Vec::new(),
            }
        };
        let pixels = tile(29, 6);
        let mut lower = tile(13, 1);
        // A narrow neighbor whose last real pixel is opaque clamps through
        // the upper tile's remainder. The diagonal still follows tile_edge_pixel.
        lower.rgba[10 * 4..]
            .copy_from_slice(&[85, 127, 255, 255, 85, 127, 255, 255, 85, 127, 255, 255]);
        lower.row_support = tile_row_support(&lower.rgba, 13, false);
        assert!(pixels.row_support.rows[1].opaque);
        assert!(!pixels.row_support.rows[2].opaque && !pixels.row_support.rows[3].opaque);
        let adjacent = [None, Some(&lower), None];
        fn check<const H: bool, const V: bool>(
            pixels: &CachedTile,
            adjacent: [Option<&CachedTile>; 3],
            phase: (f64, f64),
            format: skia::PixelFormat,
        ) {
            let sampler = PreparedFractionalSampler::for_tile::<H, V, false>(phase, format);
            for sy in -1..pixels.pixel_size.1 as i64 {
                let mut expected = vec![
                    if format == skia::PixelFormat::Bgrx8888 {
                        0x00714a25
                    } else {
                        0xb8714a25
                    };
                    pixels.pixel_size.0 as usize + 1
                ];
                let mut actual = expected.clone();
                for (i, dst) in expected.iter_mut().enumerate() {
                    let x = i as i64 - 1;
                    let a = signed_tile_pixel(pixels, adjacent, x, sy);
                    let b = if H {
                        signed_tile_pixel(pixels, adjacent, x + 1, sy)
                    } else {
                        a
                    };
                    let c = if V {
                        signed_tile_pixel(pixels, adjacent, x, sy + 1)
                    } else {
                        a
                    };
                    let d = if H && V {
                        signed_tile_pixel(pixels, adjacent, x + 1, sy + 1)
                    } else if V {
                        c
                    } else {
                        b
                    };
                    *dst = source_over_rgba_word(
                        *dst,
                        linear_word::<H, V>(a, b, c, d, phase, sampler.inverse),
                        format,
                    );
                }
                fractional_row_signed_execute::<H, V, false, true, true>(
                    &mut actual,
                    pixels,
                    adjacent,
                    sy,
                    -1,
                    &sampler,
                    format,
                );
                assert_eq!(
                    actual, expected,
                    "phase={phase:?} sy={sy} format={format:?}"
                );
            }
        }
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            for phase in [(0.375, 0.3125), (0.1, 0.2)] {
                check::<false, true>(&pixels, adjacent, (0.0, phase.1), format);
                check::<true, false>(&pixels, adjacent, (phase.0, 0.0), format);
                check::<true, true>(&pixels, adjacent, phase, format);
            }
        }
        if std::env::var_os("RENDERER_TIMING_OPAQUE_FRACTIONAL").is_some() {
            let pixels = tile(256, 256);
            let mut baseline = tile(256, 256);
            // Same source/support geometry, with only the new row opacity
            // proof disabled, exercises the previous production blend kernel.
            for row in &mut baseline.row_support.rows {
                row.opaque = false;
            }
            let mut output = vec![0x00714a25; 256 * 256];
            let view = TargetView {
                origin: (0, 0),
                stride: 256,
            };
            let bounds = SurfaceBounds {
                left: 0,
                top: 0,
                right: 256,
                bottom: 256,
            };
            for phase in [(0.0, 0.3125), (0.375, 0.3125)] {
                output.fill(0x00714a25);
                fractional_tile::<false>(
                    &mut output,
                    view,
                    bounds,
                    (0, 0),
                    &baseline,
                    [None; 3],
                    phase,
                    skia::PixelFormat::Bgrx8888,
                );
                let expected = output.clone();
                output.fill(0x00714a25);
                fractional_tile::<false>(
                    &mut output,
                    view,
                    bounds,
                    (0, 0),
                    &pixels,
                    [None; 3],
                    phase,
                    skia::PixelFormat::Bgrx8888,
                );
                assert!(
                    output == expected,
                    "timed tile phase={phase:?} first mismatch={:?}",
                    output.iter().zip(&expected).position(|(a, b)| a != b)
                );
                let measure = |output: &mut [u32], source: &CachedTile| {
                    output.fill(0x00714a25);
                    let started = std::time::Instant::now();
                    for _ in 0..128 {
                        std::hint::black_box(fractional_tile::<false>(
                            std::hint::black_box(output),
                            view,
                            bounds,
                            (0, 0),
                            std::hint::black_box(source),
                            [None; 3],
                            phase,
                            skia::PixelFormat::Bgrx8888,
                        ));
                    }
                    started.elapsed().as_secs_f64() * 1000.0
                };
                let mut old = [0.0f64; 3];
                let mut new = [0.0f64; 3];
                for round in 0..3 {
                    old[round] = measure(&mut output, &baseline);
                    new[round] = measure(&mut output, &pixels);
                }
                old.sort_by(f64::total_cmp);
                new.sort_by(f64::total_cmp);
                eprintln!(
                    "opaque-fractional-row-timing phase={phase:?} iterations=128 pixels=65536 baseline_ms={:.3} src_ms={:.3}",
                    old[1], new[1]
                );
            }
        }
        if std::env::var_os("RENDERER_TIMING_SCROLL_SNAPPING").is_some() {
            let pixels = tile(256, 256);
            let rect = PaintRect {
                x: 0.0,
                y: 0.0,
                width: 256.0,
                height: 256.0,
            };
            let placement = TilePlacement {
                tile_id: TileId(1),
                generation: 1,
                tile_index: (0, 0),
                tiling_rect: rect,
                tile_rect: rect,
                raster_scale: 1.0,
                pixel_size: (256, 256),
                ready: true,
            };
            let base = LayerComposition {
                transform: viz::AffineTransform {
                    tx: 0.0,
                    ty: -0.25,
                    ..viz::AffineTransform::IDENTITY
                },
                projective: None,
                rendering_context_id: 0,
                clip: None,
                content_bounds: None,
                opaque_bounds: None,
                white_backing: false,
                grid_min: Some((0, 0)),
            };
            let snapped = LayerComposition {
                transform: viz::AffineTransform {
                    tx: 0.0,
                    ty: 0.0,
                    ..viz::AffineTransform::IDENTITY
                },
                ..base
            };
            let view = TargetView {
                origin: (0, 0),
                stride: 256,
            };
            let mut output = vec![0x00714a25; 256 * 256];
            let measure = |output: &mut [u32], composition| {
                output.fill(0x00714a25);
                let started = std::time::Instant::now();
                for _ in 0..128 {
                    std::hint::black_box(
                        compose_tile(
                            std::hint::black_box(output),
                            256,
                            256,
                            view,
                            skia::PixelFormat::Bgrx8888,
                            &placement,
                            composition,
                            std::hint::black_box(&pixels),
                            [None; 3],
                            None,
                        )
                        .unwrap(),
                    );
                }
                started.elapsed().as_secs_f64() * 1000.0
            };
            let (mut fractional, mut integral) = ([0.0f64; 3], [0.0f64; 3]);
            for round in 0..3 {
                fractional[round] = measure(&mut output, base);
                integral[round] = measure(&mut output, snapped);
            }
            fractional.sort_by(f64::total_cmp);
            integral.sort_by(f64::total_cmp);
            eprintln!(
                "scroll-snapping-compose-timing iterations=128 tile_pixels=65536 fractional_ms={:.3} integral_ms={:.3}",
                fractional[1], integral[1]
            );
        }
    }
}

#[inline]
fn rgba_word(row: &[u8], x: usize) -> u32 {
    let i = x * 4;
    u32::from_le_bytes([row[i], row[i + 1], row[i + 2], row[i + 3]])
}

// Only the last column/row can enter a different tile. Keep the original
// right/down/diagonal selection and missing-neighbor clamp for those edges.
fn tile_edge_pixel(
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    x: usize,
    y: usize,
) -> u32 {
    let width = pixels.pixel_size.0 as usize;
    let height = pixels.pixel_size.1 as usize;
    let right = x >= width;
    let down = y >= height;
    let neighbor = match (right, down) {
        (true, false) => adjacent[0],
        (false, true) => adjacent[1],
        (true, true) => adjacent[2],
        _ => None,
    };
    let (source, x, y) = match neighbor {
        Some(next) => (
            next,
            if right { x - width } else { x },
            if down { y - height } else { y },
        ),
        None => (pixels, x, y),
    };
    let width = source.pixel_size.0 as usize;
    let height = source.pixel_size.1 as usize;
    if let Some(color) = compact_solid_color(source) {
        return color;
    }
    rgba_word(
        &source.rgba[y.min(height - 1) * width * 4..][..width * 4],
        x.min(width - 1),
    )
}

#[inline]
fn linear_word<const H: bool, const V: bool>(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    phase: (f64, f64),
    inverse: (f64, f64),
) -> u32 {
    let mut result = 0;
    for shift in [0, 8, 16, 24] {
        let channel = |word: u32| ((word >> shift) & 255) as f64;
        // Preserve the original f64 operation order, including round().
        let upper = if H {
            channel(a) * inverse.0 + channel(b) * phase.0
        } else {
            channel(a)
        };
        let value = if V {
            let lower = if H {
                channel(c) * inverse.0 + channel(d) * phase.0
            } else {
                channel(c)
            };
            upper * inverse.1 + lower * phase.1
        } else {
            upper
        };
        result |= (value.round().clamp(0.0, 255.0) as u32) << shift;
    }
    result
}

#[inline]
fn store_linear<const OPAQUE: bool>(
    destination: &mut u32,
    mut source: u32,
    format: skia::PixelFormat,
) {
    if OPAQUE {
        if format != skia::PixelFormat::Rgba8888 {
            source = (source & 0xff00_ff00) | ((source & 0xff) << 16) | ((source >> 16) & 0xff);
        }
        if format == skia::PixelFormat::Bgrx8888 {
            source &= 0x00ff_ffff;
        }
        *destination = source;
    } else {
        *destination = source_over_rgba_word(*destination, source, format);
    }
}

fn exact_fraction_weight(phase: f64) -> Option<u16> {
    let weight = phase * 256.0;
    (weight.is_finite() && (0.0..=256.0).contains(&weight) && weight == weight.round())
        .then_some(weight as u16)
}

// Like SkRasterPipelineBlitter's compiled shader context, these parameters
// belong to the actual tile geometry and remain unchanged across scanlines.
// Do not derive them from nominal layer translation: adding the tile origin
// can change the exact f64 phase used by the original sampler.
struct PreparedFractionalSampler {
    phase: (f64, f64),
    inverse: (f64, f64),
    fixed: Option<(u16, u16)>,
    valid: bool,
    #[cfg(target_arch = "aarch64")]
    compiled: Option<CompiledFractionalKernel>,
    #[cfg(target_arch = "aarch64")]
    compiled_src: Option<CompiledFractionalKernel>,
}
#[cfg(target_arch = "aarch64")]
type CompiledFractionalKernel =
    unsafe fn(&mut [u8], &[u8], &[u8], &PreparedFractionalSampler) -> usize;

// SkRasterPipeline::compile and SkRasterPipelineBlitter::blitRect select
// shader/blend/storage stages once, before running all spans. Keep that
// selection outside this backend's scanline/span traversal as well.
#[cfg(target_arch = "aarch64")]
unsafe fn compiled_fractional_kernel<
    const H: bool,
    const V: bool,
    const BLEND: bool,
    const SWAP: bool,
    const ZERO_X: bool,
    const FIXED: bool,
>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    sampler: &PreparedFractionalSampler,
) -> usize {
    unsafe {
        if FIXED {
            fixed_linear_neon_storage::<H, V, BLEND, SWAP, ZERO_X>(
                output,
                upper,
                lower,
                sampler.fixed.expect("compiled fixed sampler"),
            )
        } else {
            general_linear_neon_storage::<H, V, BLEND, SWAP, ZERO_X>(
                output,
                upper,
                lower,
                sampler.phase,
                sampler.inverse,
            )
        }
    }
}

#[cfg(target_arch = "aarch64")]
fn select_fractional_kernel<const H: bool, const V: bool, const BLEND: bool, const FIXED: bool>(
    format: skia::PixelFormat,
) -> CompiledFractionalKernel {
    match format {
        skia::PixelFormat::Rgba8888 => {
            compiled_fractional_kernel::<H, V, BLEND, false, false, FIXED>
        }
        skia::PixelFormat::Bgra8888 => {
            compiled_fractional_kernel::<H, V, BLEND, true, false, FIXED>
        }
        skia::PixelFormat::Bgrx8888 => compiled_fractional_kernel::<H, V, BLEND, true, true, FIXED>,
    }
}
impl PreparedFractionalSampler {
    fn new(phase: (f64, f64)) -> Self {
        Self {
            phase,
            inverse: (1.0 - phase.0, 1.0 - phase.1),
            fixed: exact_fraction_weight(phase.0).zip(exact_fraction_weight(phase.1)),
            valid: [phase.0, phase.1]
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
            #[cfg(target_arch = "aarch64")]
            compiled: None,
            #[cfg(target_arch = "aarch64")]
            compiled_src: None,
        }
    }

    fn for_tile<const H: bool, const V: bool, const OPAQUE: bool>(
        phase: (f64, f64),
        format: skia::PixelFormat,
    ) -> Self {
        let mut sampler = Self::new(phase);
        #[cfg(target_arch = "aarch64")]
        {
            sampler.compiled = Some(if sampler.fixed.is_some() {
                if OPAQUE {
                    select_fractional_kernel::<H, V, false, true>(format)
                } else {
                    select_fractional_kernel::<H, V, true, true>(format)
                }
            } else {
                if OPAQUE {
                    select_fractional_kernel::<H, V, false, false>(format)
                } else {
                    select_fractional_kernel::<H, V, true, false>(format)
                }
            });
            // A translucent tile can contain opaque scanline interiors. Compile
            // their Src storage with the same sampling arithmetic, once per tile.
            sampler.compiled_src = Some(if OPAQUE {
                sampler.compiled.unwrap()
            } else if sampler.fixed.is_some() {
                select_fractional_kernel::<H, V, false, true>(format)
            } else {
                select_fractional_kernel::<H, V, false, false>(format)
            });
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = format;
        }
        sampler
    }
}

#[inline]
fn fractional_compiled_span<const H: bool, const V: bool, const OPAQUE: bool>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> bool {
    #[cfg(not(target_arch = "aarch64"))]
    {
        return fractional_fixed_span_prepared::<H, V, OPAQUE, true>(
            output, upper, lower, first, sampler, format,
        );
    }
    #[cfg(target_arch = "aarch64")]
    {
        if output.len() > TILE_SIZE as usize || !sampler.valid {
            return false;
        }
        let bytes = output.len() * 4;
        let start = first * 4;
        let required = bytes + if H { 4 } else { 0 };
        if start + required > upper.len() || (V && start + required > lower.len()) {
            return false;
        }
        let kernel = if OPAQUE {
            sampler.compiled_src
        } else {
            sampler.compiled
        }
        .expect("prepared tile program");
        // Bounds above include the actual H neighbor. Output is a complete
        // exclusive N32 row; RGBA effects and final BGRX retain their formats.
        let done = unsafe {
            let target = std::slice::from_raw_parts_mut(output.as_mut_ptr().cast::<u8>(), bytes);
            kernel(
                target,
                &upper[start..],
                if V { &lower[start..] } else { &upper[start..] },
                sampler,
            )
        };
        #[cfg(feature = "compose_work_profile")]
        layer_compose_profile::span(output.len(), done);
        for i in done..output.len() {
            let x = first + i;
            let a = rgba_word(upper, x);
            let b = if H { rgba_word(upper, x + 1) } else { a };
            let c = if V { rgba_word(lower, x) } else { a };
            let d = if H && V {
                rgba_word(lower, x + 1)
            } else if V {
                c
            } else {
                b
            };
            store_linear::<OPAQUE>(
                &mut output[i],
                linear_word::<H, V>(a, b, c, d, sampler.phase, sampler.inverse),
                format,
            );
        }
        true
    }
}

#[inline]
fn fixed_linear_word<const H: bool, const V: bool>(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    weight: (u16, u16),
) -> u32 {
    let wx = u32::from(weight.0);
    let wy = u32::from(weight.1);
    let mut result = 0;
    for shift in [0, 8, 16, 24] {
        let channel = |word: u32| (word >> shift) & 255;
        let value = if H && V {
            let upper = channel(a) * (256 - wx) + channel(b) * wx;
            let lower = channel(c) * (256 - wx) + channel(d) * wx;
            (upper * (256 - wy) + lower * wy + 32768) >> 16
        } else if H {
            (channel(a) * (256 - wx) + channel(b) * wx + 128) >> 8
        } else {
            (channel(a) * (256 - wy) + channel(c) * wy + 128) >> 8
        };
        result |= value << shift;
    }
    result
}

// SkPMSrcOver_neon8 / SkMulDiv255Round_neon8: retain both rounded
// narrowing shifts and saturating add. Sampling hands off its actual N32
// registers, matching RasterPipeline's color -> srcover_rgba_8888 -> store.
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn src_over_neon8(
    source: core::arch::aarch64::uint8x8_t,
    destination: core::arch::aarch64::uint8x8_t,
    inverse_alpha: core::arch::aarch64::uint8x8_t,
) -> core::arch::aarch64::uint8x8_t {
    use core::arch::aarch64::*;
    unsafe {
        let product = vmull_u8(destination, inverse_alpha);
        // SkBlitRow_opts.h::SkMulDiv255Round_neon8, including both
        // rounded shifts; keep the instructions corresponding to the source.
        let scaled = vraddhn_u16(product, vrshrq_n_u16::<8>(product));
        vqadd_u8(source, scaled)
    }
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn fixed_linear_neon<const H: bool, const V: bool, const BLEND: bool>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    weight: (u16, u16),
    format: skia::PixelFormat,
) -> usize {
    // SkRasterPipelineBlitter selects swap_rb and the storage stage when
    // compiling its pipeline. Select this backend's storage before the loop.
    unsafe {
        match format {
            skia::PixelFormat::Rgba8888 => {
                fixed_linear_neon_storage::<H, V, BLEND, false, false>(output, upper, lower, weight)
            }
            skia::PixelFormat::Bgra8888 => {
                fixed_linear_neon_storage::<H, V, BLEND, true, false>(output, upper, lower, weight)
            }
            skia::PixelFormat::Bgrx8888 => {
                fixed_linear_neon_storage::<H, V, BLEND, true, true>(output, upper, lower, weight)
            }
        }
    }
}

// Keep storage-specialized loops separate so dispatch cannot reconverge
// into runtime format selects inside each four/eight-pixel batch.
#[cfg(target_arch = "aarch64")]
#[inline(never)]
unsafe fn fixed_linear_neon_storage<
    const H: bool,
    const V: bool,
    const BLEND: bool,
    const SWAP: bool,
    const ZERO_X: bool,
>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    weight: (u16, u16),
) -> usize {
    use core::arch::aarch64::*;
    unsafe fn horizontal(a: uint8x8_t, b: uint8x8_t, w: u16) -> uint16x8_t {
        unsafe { vmlaq_n_u16(vmulq_n_u16(vmovl_u8(a), 256 - w), vmovl_u8(b), w) }
    }
    unsafe fn vertical(a: uint16x8_t, b: uint16x8_t, w: u16) -> uint8x8_t {
        unsafe {
            let lo = vmlal_n_u16(vmull_n_u16(vget_low_u16(a), 256 - w), vget_low_u16(b), w);
            let hi = vmlal_n_u16(vmull_n_u16(vget_high_u16(a), 256 - w), vget_high_u16(b), w);
            vmovn_u16(vcombine_u16(vrshrn_n_u32::<16>(lo), vrshrn_n_u32::<16>(hi)))
        }
    }
    let mut offset = 0;
    // Packed unaligned loads cover four complete RGBA pixels. Caller excludes
    // the right tile edge and proves both source spans cover every load. The
    // shifted horizontal load reads exactly one extra real neighbor pixel.
    unsafe {
        // Corresponding swap_rb + store_8888 storage stages. Canonical source
        // channels and all sampling arithmetic remain unchanged above the store.
        let swap = vld1q_u8([2u8, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15].as_ptr());
        let keep_rgb = vdupq_n_u32(0x00ff_ffff);
        let repeat_alpha =
            vld1q_u8([3u8, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15].as_ptr());
        let block = |output: *mut u8, upper: *const u8, lower: *const u8| {
            let a = vld1q_u8(upper);
            let b = if H { vld1q_u8(upper.add(4)) } else { a };
            let c = if V { vld1q_u8(lower) } else { a };
            let d = if H && V { vld1q_u8(lower.add(4)) } else { c };
            let (lo, hi) = if H && V {
                (
                    vertical(
                        horizontal(vget_low_u8(a), vget_low_u8(b), weight.0),
                        horizontal(vget_low_u8(c), vget_low_u8(d), weight.0),
                        weight.1,
                    ),
                    vertical(
                        horizontal(vget_high_u8(a), vget_high_u8(b), weight.0),
                        horizontal(vget_high_u8(c), vget_high_u8(d), weight.0),
                        weight.1,
                    ),
                )
            } else {
                let (other, w) = if H { (b, weight.0) } else { (c, weight.1) };
                (
                    vrshrn_n_u16::<8>(horizontal(vget_low_u8(a), vget_low_u8(other), w)),
                    vrshrn_n_u16::<8>(horizontal(vget_high_u8(a), vget_high_u8(other), w)),
                )
            };
            let mut result = vcombine_u8(lo, hi);
            if SWAP {
                result = vqtbl1q_u8(result, swap);
            }
            if BLEND {
                let inverse = vmvnq_u8(vqtbl1q_u8(result, repeat_alpha));
                let destination = vld1q_u8(output);
                result = vcombine_u8(
                    src_over_neon8(
                        vget_low_u8(result),
                        vget_low_u8(destination),
                        vget_low_u8(inverse),
                    ),
                    src_over_neon8(
                        vget_high_u8(result),
                        vget_high_u8(destination),
                        vget_high_u8(inverse),
                    ),
                );
            }
            if ZERO_X {
                result = vreinterpretq_u8_u32(vandq_u32(vreinterpretq_u32_u8(result), keep_rgb));
            }
            vst1q_u8(output, result);
        };
        while offset + 16 <= output.len() {
            let upper = upper.as_ptr().add(offset);
            let lower = if V { lower.as_ptr().add(offset) } else { upper };
            block(output.as_mut_ptr().add(offset), upper, lower);
            offset += 16;
        }
        // SkBlitRow_opts.h::SkPMSrcOver_neon2 and its two/one-pixel tail:
        // narrow loads cover only real pixels, and a lane store writes the
        // final pixel. Sampling keeps the same arithmetic as the full block.
        let block_two =
            |a: uint8x8_t, b: uint8x8_t, c: uint8x8_t, d: uint8x8_t, destination: uint8x8_t| {
                let mut result = if H && V {
                    vertical(
                        horizontal(a, b, weight.0),
                        horizontal(c, d, weight.0),
                        weight.1,
                    )
                } else {
                    let (other, w) = if H { (b, weight.0) } else { (c, weight.1) };
                    vrshrn_n_u16::<8>(horizontal(a, other, w))
                };
                if SWAP {
                    result = vtbl1_u8(result, vget_low_u8(swap));
                }
                if BLEND {
                    let inverse = vmvn_u8(vtbl1_u8(result, vget_low_u8(repeat_alpha)));
                    result = src_over_neon8(result, destination, inverse);
                }
                if ZERO_X {
                    result = vreinterpret_u8_u32(vand_u32(
                        vreinterpret_u32_u8(result),
                        vget_low_u32(keep_rgb),
                    ));
                }
                result
            };
        if offset + 8 <= output.len() {
            let a = vld1_u8(upper.as_ptr().add(offset));
            let b = if H {
                vld1_u8(upper.as_ptr().add(offset + 4))
            } else {
                a
            };
            let c = if V {
                vld1_u8(lower.as_ptr().add(offset))
            } else {
                a
            };
            let d = if H && V {
                vld1_u8(lower.as_ptr().add(offset + 4))
            } else {
                c
            };
            let destination = if BLEND {
                vld1_u8(output.as_ptr().add(offset))
            } else {
                vdup_n_u8(0)
            };
            vst1_u8(
                output.as_mut_ptr().add(offset),
                block_two(a, b, c, d, destination),
            );
            offset += 8;
        }
        if offset < output.len() {
            let pixel =
                |ptr: *const u8| vcreate_u8(u64::from(std::ptr::read_unaligned(ptr.cast::<u32>())));
            let a = pixel(upper.as_ptr().add(offset));
            let b = if H {
                pixel(upper.as_ptr().add(offset + 4))
            } else {
                a
            };
            let c = if V {
                pixel(lower.as_ptr().add(offset))
            } else {
                a
            };
            let d = if H && V {
                pixel(lower.as_ptr().add(offset + 4))
            } else {
                c
            };
            let destination = if BLEND {
                pixel(output.as_ptr().add(offset))
            } else {
                vdup_n_u8(0)
            };
            let result = block_two(a, b, c, d, destination);
            vst1_lane_u32::<0>(
                output.as_mut_ptr().add(offset).cast::<u32>(),
                vreinterpret_u32_u8(result),
            );
            offset += 4;
        }
    }
    offset / 4
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn general_linear_neon<const H: bool, const V: bool, const BLEND: bool>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    phase: (f64, f64),
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> usize {
    unsafe {
        match format {
            skia::PixelFormat::Rgba8888 => {
                general_linear_neon_storage::<H, V, BLEND, false, false>(
                    output, upper, lower, phase, inverse,
                )
            }
            skia::PixelFormat::Bgra8888 => general_linear_neon_storage::<H, V, BLEND, true, false>(
                output, upper, lower, phase, inverse,
            ),
            skia::PixelFormat::Bgrx8888 => general_linear_neon_storage::<H, V, BLEND, true, true>(
                output, upper, lower, phase, inverse,
            ),
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(never)]
unsafe fn general_linear_neon_storage<
    const H: bool,
    const V: bool,
    const BLEND: bool,
    const SWAP: bool,
    const ZERO_X: bool,
>(
    output: &mut [u8],
    upper: &[u8],
    lower: &[u8],
    phase: (f64, f64),
    inverse: (f64, f64),
) -> usize {
    use core::arch::aarch64::*;
    #[inline(always)]
    unsafe fn pair<const H: bool, const V: bool>(
        a: uint32x2_t,
        b: uint32x2_t,
        c: uint32x2_t,
        d: uint32x2_t,
        phase: (f64, f64),
        inverse: (f64, f64),
    ) -> uint32x2_t {
        unsafe {
            let convert = |v| vcvtq_f64_u64(vmovl_u32(v));
            // Separate multiply/add intrinsics preserve linear_word's exact
            // f64 order. No phase quantization, f32 narrowing or fused multiply.
            let top = if H {
                vaddq_f64(
                    vmulq_n_f64(convert(a), inverse.0),
                    vmulq_n_f64(convert(b), phase.0),
                )
            } else {
                convert(a)
            };
            let value = if V {
                let bottom = if H {
                    vaddq_f64(
                        vmulq_n_f64(convert(c), inverse.0),
                        vmulq_n_f64(convert(d), phase.0),
                    )
                } else {
                    convert(c)
                };
                vaddq_f64(vmulq_n_f64(top, inverse.1), vmulq_n_f64(bottom, phase.1))
            } else {
                top
            };
            // Inputs and positive weights enclose [0,255]. FRINTA implements
            // the scalar positive round() tie rule before integer conversion.
            vmovn_u64(vcvtq_u64_f64(vrndaq_f64(value)))
        }
    }
    #[inline(always)]
    unsafe fn four<const H: bool, const V: bool>(
        a: uint16x4_t,
        b: uint16x4_t,
        c: uint16x4_t,
        d: uint16x4_t,
        phase: (f64, f64),
        inverse: (f64, f64),
    ) -> uint16x4_t {
        unsafe {
            let (a, b, c, d) = (vmovl_u16(a), vmovl_u16(b), vmovl_u16(c), vmovl_u16(d));
            vmovn_u32(vcombine_u32(
                pair::<H, V>(
                    vget_low_u32(a),
                    vget_low_u32(b),
                    vget_low_u32(c),
                    vget_low_u32(d),
                    phase,
                    inverse,
                ),
                pair::<H, V>(
                    vget_high_u32(a),
                    vget_high_u32(b),
                    vget_high_u32(c),
                    vget_high_u32(d),
                    phase,
                    inverse,
                ),
            ))
        }
    }
    #[inline(always)]
    unsafe fn eight<const H: bool, const V: bool>(
        a: uint8x8_t,
        b: uint8x8_t,
        c: uint8x8_t,
        d: uint8x8_t,
        phase: (f64, f64),
        inverse: (f64, f64),
    ) -> uint8x8_t {
        unsafe {
            let (a, b, c, d) = (vmovl_u8(a), vmovl_u8(b), vmovl_u8(c), vmovl_u8(d));
            vmovn_u16(vcombine_u16(
                four::<H, V>(
                    vget_low_u16(a),
                    vget_low_u16(b),
                    vget_low_u16(c),
                    vget_low_u16(d),
                    phase,
                    inverse,
                ),
                four::<H, V>(
                    vget_high_u16(a),
                    vget_high_u16(b),
                    vget_high_u16(c),
                    vget_high_u16(d),
                    phase,
                    inverse,
                ),
            ))
        }
    }
    let mut at = 0;
    // Interleaved RGBA rows are complete and caller excludes the actual tile
    // edge. Each 8-pixel load and optional right neighbor is inside its span.
    unsafe {
        while at + 32 <= output.len() {
            let a = vld4_u8(upper.as_ptr().add(at));
            let b = if H {
                vld4_u8(upper.as_ptr().add(at + 4))
            } else {
                a
            };
            let c = if V {
                vld4_u8(lower.as_ptr().add(at))
            } else {
                a
            };
            let d = if H && V {
                vld4_u8(lower.as_ptr().add(at + 4))
            } else if V {
                c
            } else {
                b
            };
            let (r, g, b, alpha) = (
                eight::<H, V>(a.0, b.0, c.0, d.0, phase, inverse),
                eight::<H, V>(a.1, b.1, c.1, d.1, phase, inverse),
                eight::<H, V>(a.2, b.2, c.2, d.2, phase, inverse),
                eight::<H, V>(a.3, b.3, c.3, d.3, phase, inverse),
            );
            let (mut r, mut b) = if SWAP { (b, r) } else { (r, b) };
            let (mut g, mut alpha) = (g, alpha);
            if BLEND {
                let inverse = vmvn_u8(alpha);
                let destination = vld4_u8(output.as_ptr().add(at));
                r = src_over_neon8(r, destination.0, inverse);
                g = src_over_neon8(g, destination.1, inverse);
                b = src_over_neon8(b, destination.2, inverse);
                alpha = src_over_neon8(alpha, destination.3, inverse);
            }
            let alpha = if ZERO_X { vdup_n_u8(0) } else { alpha };
            // Skia's NEON store_8888_ keeps channels in registers through the
            // final format permutation and interleaved pixel store.
            vst4_u8(output.as_mut_ptr().add(at), uint8x8x4_t(r, g, b, alpha));
            at += 32;
        }
    }
    at / 4
}

fn fractional_general_span_prepared<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> bool {
    if output.len() > TILE_SIZE as usize || !sampler.valid {
        return false;
    }
    let bytes = output.len() * 4;
    let start = first * 4;
    let required = bytes + if H { 4 } else { 0 };
    if start + required > upper.len() || (V && start + required > lower.len()) {
        return false;
    }
    let phase = sampler.phase;
    let inverse = sampler.inverse;
    let mut scratch = [0u32; TILE_SIZE as usize];
    let rgba = &mut scratch[..output.len()];
    #[cfg(target_arch = "aarch64")]
    // SAFETY: the checks above cover the byte output and all source neighbors;
    // valid phases keep every rounded channel within an initialized N32 pixel.
    let done = unsafe {
        let target = if OPAQUE || COMPOSITOR {
            &mut *output
        } else {
            &mut *rgba
        };
        let target = std::slice::from_raw_parts_mut(target.as_mut_ptr().cast::<u8>(), bytes);
        let lower = if V { &lower[start..] } else { &upper[start..] };
        if !OPAQUE && COMPOSITOR {
            general_linear_neon::<H, V, true>(
                target,
                &upper[start..],
                lower,
                phase,
                inverse,
                format,
            )
        } else {
            general_linear_neon::<H, V, false>(
                target,
                &upper[start..],
                lower,
                phase,
                inverse,
                if OPAQUE {
                    format
                } else {
                    skia::PixelFormat::Rgba8888
                },
            )
        }
    };
    #[cfg(not(target_arch = "aarch64"))]
    let done = 0;
    #[cfg(feature = "compose_work_profile")]
    layer_compose_profile::span(output.len(), done);
    for i in done..output.len() {
        let x = first + i;
        let a = rgba_word(upper, x);
        let b = if H { rgba_word(upper, x + 1) } else { a };
        let c = if V { rgba_word(lower, x) } else { a };
        let d = if H && V {
            rgba_word(lower, x + 1)
        } else if V {
            c
        } else {
            b
        };
        let sampled = linear_word::<H, V>(a, b, c, d, phase, inverse);
        if OPAQUE {
            store_linear::<true>(&mut output[i], sampled, format);
        } else if COMPOSITOR {
            store_linear::<false>(&mut output[i], sampled, format);
        } else {
            rgba[i] = sampled;
        }
    }
    if !OPAQUE && !COMPOSITOR {
        blend_group_row(output, rgba, format);
    }
    true
}

fn fractional_fixed_span_prepared<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> bool {
    let Some((wx, wy)) = sampler.fixed else {
        return fractional_general_span_prepared::<H, V, OPAQUE, COMPOSITOR>(
            output, upper, lower, first, sampler, format,
        );
    };
    if output.len() > TILE_SIZE as usize {
        return false;
    }
    let bytes = output.len() * 4;
    let start = first * 4;
    let required = bytes + if H { 4 } else { 0 };
    if start + required > upper.len() || (V && start + required > lower.len()) {
        return false;
    }
    // Dyadic weights make every original f64 multiply/add exact (at most a
    // 24-bit numerator). Add half the denominator only once, preserving
    // round()'s positive ties-up rule, including two-axis interpolation.
    let mut scratch = [0u32; TILE_SIZE as usize];
    let rgba = &mut scratch[..output.len()];
    #[cfg(target_arch = "aarch64")]
    let done = unsafe {
        let target = if OPAQUE || COMPOSITOR {
            &mut *output
        } else {
            &mut *rgba
        };
        let target = std::slice::from_raw_parts_mut(target.as_mut_ptr().cast::<u8>(), bytes);
        let lower = if V { &lower[start..] } else { &upper[start..] };
        if !OPAQUE && COMPOSITOR {
            fixed_linear_neon::<H, V, true>(target, &upper[start..], lower, (wx, wy), format)
        } else {
            fixed_linear_neon::<H, V, false>(
                target,
                &upper[start..],
                lower,
                (wx, wy),
                if OPAQUE {
                    format
                } else {
                    skia::PixelFormat::Rgba8888
                },
            )
        }
    };
    #[cfg(not(target_arch = "aarch64"))]
    let done = 0;
    #[cfg(feature = "compose_work_profile")]
    layer_compose_profile::span(output.len(), done);
    for i in done..output.len() {
        let x = first + i;
        let a = rgba_word(upper, x);
        let b = if H { rgba_word(upper, x + 1) } else { a };
        let c = if V { rgba_word(lower, x) } else { a };
        let d = if H && V { rgba_word(lower, x + 1) } else { c };
        let sampled = fixed_linear_word::<H, V>(a, b, c, d, (wx, wy));
        if OPAQUE {
            store_linear::<true>(&mut output[i], sampled, format);
        } else if COMPOSITOR {
            store_linear::<false>(&mut output[i], sampled, format);
        } else {
            rgba[i] = sampled;
        }
    }
    if !OPAQUE && !COMPOSITOR {
        blend_group_row(output, rgba, format);
    }
    true
}

#[cfg(test)]
fn fractional_fixed_span<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    upper: &[u8],
    lower: &[u8],
    first: usize,
    phase: (f64, f64),
    format: skia::PixelFormat,
) -> bool {
    fractional_fixed_span_prepared::<H, V, OPAQUE, COMPOSITOR>(
        output,
        upper,
        lower,
        first,
        &PreparedFractionalSampler::new(phase),
        format,
    )
}

#[cfg(test)]
mod fractional_kernel_test {
    use super::*;

    #[test]
    fn fractional_simd_matches_original_sampling_and_blending() {
        let row = |seed: u32| -> Vec<u8> {
            (0..19u32)
                .flat_map(|x| {
                    let alpha = if x % 5 == 0 {
                        0
                    } else {
                        ((x * 37 + seed) % 256) as u8
                    };
                    [
                        ((x * 19 + seed) % (u32::from(alpha) + 1)) as u8,
                        ((x * 29 + seed) % (u32::from(alpha) + 1)) as u8,
                        ((x * 43 + seed) % (u32::from(alpha) + 1)) as u8,
                        alpha,
                    ]
                })
                .collect()
        };
        let upper = row(17);
        let lower = row(61);
        // Effect close uses the final frame's zero-X invariant. Compare its
        // fused BGRX store with the original scalar SrcOver, including zero
        // source pixels and a vector tail.
        let source: Vec<u32> = upper
            .chunks_exact(4)
            .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
            .collect();
        let mut actual = vec![0x00734a25; source.len()];
        let expected: Vec<u32> = actual
            .iter()
            .zip(&source)
            .map(|(&dst, &src)| source_over_rgba_word(dst, src, skia::PixelFormat::Bgrx8888))
            .collect();
        blend_compositor_row(&mut actual, &source, skia::PixelFormat::Bgrx8888);
        assert_eq!(actual, expected);
        // Exercise fractional production BGRX with the actual zero-X proof,
        // exact transparent source pixels, SIMD blocks and a scalar tail.
        let mut upper_final = upper.clone();
        let mut lower_final = lower.clone();
        // Nine source pixels make the first eight H-filtered pixels exactly
        // zero, exercising complete fixed/general source blocks, then a tail.
        upper_final[..36].fill(0);
        lower_final[..36].fill(0);
        fn check_compositor(
            upper: &[u8],
            lower: &[u8],
            phase: (f64, f64),
            format: skia::PixelFormat,
            destination: u32,
        ) {
            let mut expected = vec![destination; 17];
            let mut actual = expected.clone();
            for (x, dst) in expected.iter_mut().enumerate() {
                let sampled = linear_word::<true, true>(
                    rgba_word(upper, x),
                    rgba_word(upper, x + 1),
                    rgba_word(lower, x),
                    rgba_word(lower, x + 1),
                    phase,
                    (1.0 - phase.0, 1.0 - phase.1),
                );
                *dst = source_over_rgba_word(*dst, sampled, format);
            }
            assert!(fractional_fixed_span::<true, true, false, true>(
                &mut actual,
                upper,
                lower,
                0,
                phase,
                format
            ));
            assert_eq!(actual, expected);
        }
        check_compositor(
            &upper_final,
            &lower_final,
            (0.375, 0.3125),
            skia::PixelFormat::Bgrx8888,
            0x00734a25,
        );
        // The general f64 sampler uses eight deinterleaved source registers;
        // cover its fused RGBA blend while retaining source zeroes and a tail.
        check_compositor(
            &upper_final,
            &lower_final,
            (0.1, 0.2),
            skia::PixelFormat::Rgba8888,
            0x80734a25,
        );
        fn check<const H: bool, const V: bool, const OPAQUE: bool>(
            upper: &[u8],
            lower: &[u8],
            phase: (f64, f64),
            format: skia::PixelFormat,
        ) {
            // Retain the original floating-point sampler and scalar blend as
            // the independent reference, including a vector tail and raw X.
            let mut expected = vec![0xb8734a25; 17];
            let mut actual = expected.clone();
            for (x, dst) in expected.iter_mut().enumerate() {
                let a = rgba_word(upper, x);
                let b = if H { rgba_word(upper, x + 1) } else { a };
                let c = if V { rgba_word(lower, x) } else { a };
                let d = if H && V {
                    rgba_word(lower, x + 1)
                } else if V {
                    c
                } else {
                    b
                };
                store_linear::<OPAQUE>(
                    dst,
                    linear_word::<H, V>(a, b, c, d, phase, (1.0 - phase.0, 1.0 - phase.1)),
                    format,
                );
            }
            assert!(fractional_fixed_span::<H, V, OPAQUE, false>(
                &mut actual,
                upper,
                lower,
                0,
                phase,
                format
            ));
            assert_eq!(
                actual, expected,
                "phase={phase:?} opaque={OPAQUE} format={format:?}"
            );
            // Production's compiled program uses the actual final-target X=0
            // invariant. Cover exact-sized source/output tails independently
            // against scalar sampling + SrcOver, rather than the vector body.
            let sampler = PreparedFractionalSampler::for_tile::<H, V, OPAQUE>(phase, format);
            for count in [1usize, 2, 3, 7, 17] {
                let destination = if format == skia::PixelFormat::Bgrx8888 {
                    0x00734a25
                } else {
                    0xb8734a25
                };
                let mut expected = vec![destination; count];
                let mut actual = expected.clone();
                for (x, dst) in expected.iter_mut().enumerate() {
                    let a = rgba_word(upper, x);
                    let b = if H { rgba_word(upper, x + 1) } else { a };
                    let c = if V { rgba_word(lower, x) } else { a };
                    let d = if H && V {
                        rgba_word(lower, x + 1)
                    } else if V {
                        c
                    } else {
                        b
                    };
                    store_linear::<OPAQUE>(
                        dst,
                        linear_word::<H, V>(a, b, c, d, phase, (1.0 - phase.0, 1.0 - phase.1)),
                        format,
                    );
                }
                let extent = (count + usize::from(H)) * 4;
                assert!(fractional_compiled_span::<H, V, OPAQUE>(
                    &mut actual,
                    &upper[..extent],
                    &lower[..extent],
                    0,
                    &sampler,
                    format
                ));
                assert_eq!(
                    actual, expected,
                    "compiled count={count} phase={phase:?} opaque={OPAQUE} format={format:?}"
                );
            }
        }
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            check::<true, false, false>(&upper, &lower, (0.375, 0.0), format);
            check::<false, true, false>(&upper, &lower, (0.0, 0.5), format);
            check::<true, true, false>(&upper, &lower, (0.375, 0.3125), format);
            check::<true, true, true>(&upper, &lower, (0.375, 0.3125), format);
            for phase in [
                (0.1, 0.2),
                (1.0 / 3.0, 7.0 / 9.0),
                (
                    f64::from_bits(0.5f64.to_bits() - 1),
                    f64::from_bits(0.5f64.to_bits() + 1),
                ),
            ] {
                check::<true, false, false>(&upper, &lower, (phase.0, 0.0), format);
                check::<false, true, false>(&upper, &lower, (0.0, phase.1), format);
                check::<true, true, false>(&upper, &lower, phase, format);
                check::<true, true, true>(&upper, &lower, phase, format);
            }
        }
    }
}

fn fractional_row_prepared<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: usize,
    sx: usize,
    sampler: &PreparedFractionalSampler,
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    fractional_row_execute::<H, V, OPAQUE, COMPOSITOR, false>(
        output, pixels, adjacent, sy, sx, sampler, inverse, format,
    )
}

fn fractional_row_execute<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
    const COMPILED: bool,
>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: usize,
    sx: usize,
    sampler: &PreparedFractionalSampler,
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    let phase = sampler.phase;
    let width = pixels.pixel_size.0 as usize;
    let height = pixels.pixel_size.1 as usize;
    let (below, below_y) = if V && sy + 1 >= height {
        match adjacent[1] {
            Some(next) => (next, 0),
            None => (pixels, height - 1),
        }
    } else {
        (pixels, if V { sy + 1 } else { sy })
    };
    let lower_width = below.pixel_size.0 as usize;
    let mut support = pixels.row_support.rows[sy].extent;
    if V {
        let mut next = below.row_support.rows[below_y].extent;
        // Clamping a narrow neighbor's last nonzero pixel can extend its
        // support through the current tile's remaining columns.
        if next.0 < next.1 && next.1 == lower_width {
            next.1 = width.max(next.1);
        }
        if next.0 < next.1 {
            support = if support.0 == support.1 {
                next
            } else {
                (support.0.min(next.0), support.1.max(next.1))
            };
        }
    }
    if H {
        if support.0 < support.1 {
            support.0 = support.0.saturating_sub(1);
        }
        if tile_edge_pixel(pixels, adjacent, width, sy) != 0
            || (V && tile_edge_pixel(pixels, adjacent, width, sy + 1) != 0)
        {
            support = if support.0 == support.1 {
                (width - 1, width)
            } else {
                (support.0.min(width - 1), support.1.max(width))
            };
        }
    }
    let first = sx.max(support.0);
    let last = (sx + output.len()).min(support.1);
    if first >= last {
        return None;
    }
    // SOLID_COLOR_MODE has no backing resource. A scalar edge path preserves
    // the same bilinear footprint when a solid meets a differently colored
    // resource; the common all-solid case was already handled as one fill.
    if compact_solid_color(pixels).is_some() || compact_solid_color(below).is_some() {
        for x in first..last {
            let a = tile_edge_pixel(pixels, adjacent, x, sy);
            let b = if H {
                tile_edge_pixel(pixels, adjacent, x + 1, sy)
            } else {
                a
            };
            let c = if V {
                tile_edge_pixel(pixels, adjacent, x, sy + 1)
            } else {
                a
            };
            let d = if H && V {
                tile_edge_pixel(pixels, adjacent, x + 1, sy + 1)
            } else if V {
                c
            } else {
                b
            };
            let sampled = linear_word::<H, V>(a, b, c, d, phase, inverse);
            store_linear::<OPAQUE>(&mut output[x - sx], sampled, format);
        }
        return Some((first - sx, last - sx));
    }
    let upper = &pixels.rgba[sy * width * 4..][..width * 4];
    let lower = &below.rgba[below_y * lower_width * 4..][..lower_width * 4];
    let interior_end = if H { last.min(width - 1) } else { last };
    // SkRasterPipelineBlitter.cpp strength-reduces SrcOver to Src for an
    // opaque source. Reuse the
    // immutable product's row proof for the complete bilinear footprint;
    // transparent margins, holes and the actual neighbor edge keep SrcOver.
    // A narrow lower tile clamps its last opaque pixel beyond its width.
    let upper_support = pixels.row_support.rows[sy];
    let below_support = below.row_support.rows[below_y];
    let opaque_span = if !OPAQUE
        && sampler.valid
        && (COMPILED || inverse == sampler.inverse)
        && upper_support.opaque
        && (!V || below_support.opaque)
    {
        let left = upper_support
            .extent
            .0
            .max(if V { below_support.extent.0 } else { 0 });
        let right = upper_support.extent.1.saturating_sub(usize::from(H)).min(
            if !V || below_support.extent.1 == lower_width {
                width
            } else {
                below_support.extent.1.saturating_sub(usize::from(H))
            },
        );
        Some((left, right))
    } else {
        None
    };
    if first < interior_end {
        let lower_support = if V {
            below.row_support.spans(below_y)
        } else {
            &[]
        };
        let mut draw_span = |first: usize, end: usize, opaque: bool| {
            if first >= end {
                return;
            }
            let input = &upper[first * 4..end * 4];
            let destination = &mut output[first - sx..end - sx];
            let processed = if opaque {
                if COMPILED {
                    fractional_compiled_span::<H, V, true>(
                        destination,
                        upper,
                        lower,
                        first,
                        sampler,
                        format,
                    )
                } else {
                    fractional_fixed_span_prepared::<H, V, true, COMPOSITOR>(
                        destination,
                        upper,
                        lower,
                        first,
                        sampler,
                        format,
                    )
                }
            } else if COMPILED {
                fractional_compiled_span::<H, V, OPAQUE>(
                    destination,
                    upper,
                    lower,
                    first,
                    sampler,
                    format,
                )
            } else {
                fractional_fixed_span_prepared::<H, V, OPAQUE, COMPOSITOR>(
                    destination,
                    upper,
                    lower,
                    first,
                    sampler,
                    format,
                )
            };
            if !processed {
                for (i, (dst, bytes)) in destination
                    .iter_mut()
                    .zip(input.chunks_exact(4))
                    .enumerate()
                {
                    let x = first + i;
                    let a = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    let b = if H { rgba_word(upper, x + 1) } else { a };
                    let c = if V {
                        rgba_word(lower, x.min(lower_width - 1))
                    } else {
                        a
                    };
                    let d = if H && V {
                        rgba_word(lower, (x + 1).min(lower_width - 1))
                    } else if V {
                        c
                    } else {
                        b
                    };
                    let sampled = linear_word::<H, V>(a, b, c, d, phase, inverse);
                    if opaque {
                        store_linear::<true>(dst, sampled, format);
                    } else {
                        store_linear::<OPAQUE>(dst, sampled, format);
                    }
                }
            }
        };
        for_fractional_spans::<H>(
            pixels.row_support.spans(sy),
            lower_support,
            width,
            lower_width,
            |span_left, span_right| {
                let first = first.max(span_left);
                let end = interior_end.min(span_right);
                if first >= end {
                    return;
                }
                if let Some((opaque_left, opaque_right)) = opaque_span {
                    let left = first.max(opaque_left);
                    let right = end.min(opaque_right);
                    if left < right {
                        draw_span(first, left, false);
                        draw_span(left, right, true);
                        draw_span(right, end, false);
                        return;
                    }
                }
                draw_span(first, end, OPAQUE);
            },
        );
    }
    if H && last == width && first < width {
        let x = width - 1;
        let a = rgba_word(upper, x);
        let b = tile_edge_pixel(pixels, adjacent, width, sy);
        let c = if V {
            rgba_word(lower, x.min(lower_width - 1))
        } else {
            a
        };
        let d = if V {
            tile_edge_pixel(pixels, adjacent, width, sy + 1)
        } else {
            b
        };
        store_linear::<OPAQUE>(
            &mut output[x - sx],
            linear_word::<H, V>(a, b, c, d, phase, inverse),
            format,
        );
    }
    // Both the SIMD interior and scalar neighbor edge only write within this
    // existing support span, including horizontal/vertical neighbor expansion.
    Some((first - sx, last - sx))
}

// Tile texture filtering includes a transparent pixel before the first
// source column/row. Keep all nonnegative spans on the existing SIMD kernel;
// only this one-pixel outer fringe needs signed scalar sampling. Missing
// right/down resources retain the original clamping contract.
fn signed_tile_pixel(
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    x: i64,
    y: i64,
) -> u32 {
    if x < 0 || y < 0 {
        return 0;
    }
    tile_edge_pixel(pixels, adjacent, x as usize, y as usize)
}

fn fractional_row_signed_execute<
    const H: bool,
    const V: bool,
    const OPAQUE: bool,
    const COMPOSITOR: bool,
    const COMPILED: bool,
>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: i64,
    sx: i64,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    debug_assert!(sx >= -1 && sy >= -1);
    if sx >= 0 && sy >= 0 {
        return fractional_row_execute::<H, V, OPAQUE, COMPOSITOR, COMPILED>(
            output,
            pixels,
            adjacent,
            sy as usize,
            sx as usize,
            sampler,
            sampler.inverse,
            format,
        );
    }
    // A negative row uses transparent upper samples and real source row zero;
    // a negative column needs just one scalar cell before the regular span.
    let count = if sy < 0 {
        output.len()
    } else {
        output.len().min(1)
    };
    let mut span = None;
    for (i, dst) in output[..count].iter_mut().enumerate() {
        let x = sx + i as i64;
        let a = signed_tile_pixel(pixels, adjacent, x, sy);
        let b = if H {
            signed_tile_pixel(pixels, adjacent, x + 1, sy)
        } else {
            a
        };
        let c = if V {
            signed_tile_pixel(pixels, adjacent, x, sy + 1)
        } else {
            a
        };
        let d = if H && V {
            signed_tile_pixel(pixels, adjacent, x + 1, sy + 1)
        } else if V {
            c
        } else {
            b
        };
        let sampled = linear_word::<H, V>(a, b, c, d, sampler.phase, sampler.inverse);
        if sampled != 0 {
            // The negative footprint contains transparency even when the
            // actual tile's complete backing is opaque.
            store_linear::<false>(dst, sampled, format);
            span = Some(span.map_or((i, i + 1), |(first, _)| (first, i + 1)));
        }
    }
    if count < output.len() {
        if let Some((first, last)) = fractional_row_execute::<H, V, OPAQUE, COMPOSITOR, COMPILED>(
            &mut output[count..],
            pixels,
            adjacent,
            sy as usize,
            (sx + count as i64) as usize,
            sampler,
            sampler.inverse,
            format,
        ) {
            span = Some(span.map_or((count + first, count + last), |(start, _)| {
                (start, count + last)
            }));
        }
    }
    span
}

fn fractional_row_signed<const OPAQUE: bool, const COMPOSITOR: bool>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: i64,
    sx: i64,
    sampler: &PreparedFractionalSampler,
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    if sampler.phase.0 == 0.0 {
        fractional_row_signed_execute::<false, true, OPAQUE, COMPOSITOR, false>(
            output, pixels, adjacent, sy, sx, sampler, format,
        )
    } else if sampler.phase.1 == 0.0 {
        fractional_row_signed_execute::<true, false, OPAQUE, COMPOSITOR, false>(
            output, pixels, adjacent, sy, sx, sampler, format,
        )
    } else {
        fractional_row_signed_execute::<true, true, OPAQUE, COMPOSITOR, false>(
            output, pixels, adjacent, sy, sx, sampler, format,
        )
    }
}

#[cfg(test)]
fn fractional_row<const H: bool, const V: bool, const OPAQUE: bool, const COMPOSITOR: bool>(
    output: &mut [u32],
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    sy: usize,
    sx: usize,
    phase: (f64, f64),
    inverse: (f64, f64),
    format: skia::PixelFormat,
) -> Option<(usize, usize)> {
    // Keep the generic raw-X and caller-supplied scalar-edge inverse contract.
    fractional_row_prepared::<H, V, OPAQUE, COMPOSITOR>(
        output,
        pixels,
        adjacent,
        sy,
        sx,
        &PreparedFractionalSampler::new(phase),
        inverse,
        format,
    )
}

// Only compose_tile enters this production path. Its final BGRX frame starts
// with X=0 and all writes preserve X=0; isolated effect targets are RGBA. Keep
// generic fractional_row/span callers on COMPOSITOR=false for arbitrary raw X.
fn fractional_tile<const OPAQUE: bool>(
    target: &mut [u32],
    view: TargetView,
    bounds: SurfaceBounds,
    origin: (i64, i64),
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    phase: (f64, f64),
    format: skia::PixelFormat,
) -> Option<SurfaceBounds> {
    if phase.0 == 0.0 {
        fractional_tile_axes::<false, true, OPAQUE>(
            target, view, bounds, origin, pixels, adjacent, phase, format,
        )
    } else if phase.1 == 0.0 {
        fractional_tile_axes::<true, false, OPAQUE>(
            target, view, bounds, origin, pixels, adjacent, phase, format,
        )
    } else {
        fractional_tile_axes::<true, true, OPAQUE>(
            target, view, bounds, origin, pixels, adjacent, phase, format,
        )
    }
}

fn fractional_tile_axes<const H: bool, const V: bool, const OPAQUE: bool>(
    target: &mut [u32],
    view: TargetView,
    bounds: SurfaceBounds,
    origin: (i64, i64),
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    phase: (f64, f64),
    format: skia::PixelFormat,
) -> Option<SurfaceBounds> {
    let sampler = PreparedFractionalSampler::for_tile::<H, V, OPAQUE>(phase, format);
    let mut touched = None;
    for y in bounds.top..bounds.bottom {
        let output = &mut target[view.row_range(y, bounds.left, bounds.right)];
        let sy = y as i64 - origin.1;
        let sx = bounds.left as i64 - origin.0;
        let span = fractional_row_signed_execute::<H, V, OPAQUE, true, true>(
            output, pixels, adjacent, sy, sx, &sampler, format,
        );
        if !OPAQUE {
            if let Some((first, last)) = span {
                union_bounds(
                    &mut touched,
                    Some(SurfaceBounds {
                        left: bounds.left + first,
                        right: bounds.left + last,
                        top: y,
                        bottom: y + 1,
                    }),
                );
            }
        }
    }
    if OPAQUE {
        Some(bounds)
    } else {
        touched
    }
}

fn projective_tile_bounds(
    projective: viz::ProjectiveTransform,
    input: [(f64, f64); 4],
) -> Option<(f64, f64, f64, f64)> {
    const NEAR: f64 = 1.0e-6;
    let mut clipped = Vec::new();
    for index in 0..4 {
        let a = input[index];
        let b = input[(index + 1) % 4];
        let wa = projective.source_w(a.0, a.1);
        let wb = projective.source_w(b.0, b.1);
        let ina = wa >= NEAR;
        let inb = wb >= NEAR;
        if ina {
            clipped.push(a);
        }
        if ina != inb {
            let t = (NEAR - wa) / (wb - wa);
            clipped.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    let mapped: Vec<_> = clipped
        .into_iter()
        .filter_map(|p| projective.map_point(p.0, p.1))
        .collect();
    (!mapped.is_empty()).then(|| {
        (
            mapped.iter().map(|p| p.0).fold(f64::INFINITY, f64::min),
            mapped.iter().map(|p| p.1).fold(f64::INFINITY, f64::min),
            mapped.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max),
            mapped.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
        )
    })
}

fn tile_composition_geometry(
    tile: &TilePlacement,
    composition: LayerComposition,
    width: u32,
    height: u32,
) -> io::Result<Option<(SurfaceBounds, (i64, i64), (f64, f64))>> {
    let source_x = device_coordinate(tile.tile_rect.x * tile.raster_scale)?;
    let source_y = device_coordinate(tile.tile_rect.y * tile.raster_scale)?;
    let transform = composition.transform;
    let (mut left, mut top, mut right, mut bottom, origin, phase) = if composition
        .projective
        .is_none()
        && transform.is_translation()
    {
        let origin_x = source_x + transform.tx;
        let origin_y = source_y + transform.ty;
        if !origin_x.is_finite()
            || !origin_y.is_finite()
            || origin_x.abs() > i32::MAX as f64
            || origin_y.abs() > i32::MAX as f64
        {
            return Err(io::Error::other("invalid composition placement"));
        }
        let x = origin_x.ceil() as i64;
        let y = origin_y.ceil() as i64;
        let phase = (x as f64 - origin_x, y as f64 - origin_y);
        let leading = (!composition.white_backing)
            .then_some(composition.grid_min)
            .flatten();
        (
            x - i64::from(phase.0 != 0.0 && leading.is_some_and(|min| tile.tile_index.0 == min.0)),
            y - i64::from(phase.1 != 0.0 && leading.is_some_and(|min| tile.tile_index.1 == min.1)),
            x + i64::from(tile.pixel_size.0),
            y + i64::from(tile.pixel_size.1),
            (x, y),
            phase,
        )
    } else {
        let x0 = source_x;
        let y0 = source_y;
        let x1 = x0 + f64::from(tile.pixel_size.0);
        let y1 = y0 + f64::from(tile.pixel_size.1);
        let (min_x, min_y, max_x, max_y) = if let Some(projective) = composition.projective {
            let Some(bounds) =
                projective_tile_bounds(projective, [(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
            else {
                return Ok(None);
            };
            bounds
        } else {
            let points = [
                transform.map_point(x0, y0),
                transform.map_point(x1, y0),
                transform.map_point(x0, y1),
                transform.map_point(x1, y1),
            ];
            (
                points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min),
                points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min),
                points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max),
                points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max),
            )
        };
        if ![min_x, min_y, max_x, max_y].into_iter().all(f64::is_finite) {
            return Err(io::Error::other("invalid affine composition placement"));
        }
        (
            min_x.floor() as i64,
            min_y.floor() as i64,
            max_x.ceil() as i64,
            max_y.ceil() as i64,
            (source_x.floor() as i64, source_y.floor() as i64),
            (0.0, 0.0),
        )
    };
    left = left.max(0);
    top = top.max(0);
    right = right.min(width as i64);
    bottom = bottom.min(height as i64);
    for clip in [composition.clip, composition.content_bounds]
        .into_iter()
        .flatten()
    {
        left = left.max(i64::from(clip.x));
        top = top.max(i64::from(clip.y));
        right = right.min(i64::from(clip.x) + i64::from(clip.width));
        bottom = bottom.min(i64::from(clip.y) + i64::from(clip.height));
    }
    if left >= right || top >= bottom {
        return Ok(None);
    }
    Ok(Some((
        SurfaceBounds {
            left: left as usize,
            top: top as usize,
            right: right as usize,
            bottom: bottom as usize,
        },
        origin,
        phase,
    )))
}

// SkSwizzler_opts.inc::RGBA_to_BGRA (370-400): an opaque Src span needs
// only source loads and destination stores, including an unaligned source.
fn copy_opaque_rgba_row(output: &mut [u32], input: &[u8], format: skia::PixelFormat) {
    debug_assert_eq!(input.len(), output.len() * 4);
    if format == skia::PixelFormat::Rgba8888 {
        // SAFETY: complete initialized N32 output, exclusively borrowed.
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(
                output.as_mut_ptr().cast::<u8>(),
                std::mem::size_of_val(output),
            )
        };
        bytes.copy_from_slice(input);
        return;
    }
    #[allow(unused_mut)]
    let mut at = 0;
    #[cfg(target_arch = "aarch64")]
    {
        use core::arch::aarch64::*;
        // SkSwizzler's 8888 channel permutation stays in packed vectors. Src
        // needs no deinterleave/reinterleave or destination load: use the same
        // packed representation as SkOpts' opaque source block.
        unsafe {
            let swap = vld1q_u8([2u8, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15].as_ptr());
            let keep = vdupq_n_u32(if format == skia::PixelFormat::Bgrx8888 {
                0x00ff_ffff
            } else {
                u32::MAX
            });
            while at + 16 <= output.len() {
                for block in 0..4usize {
                    let offset = (at + block * 4) * 4;
                    let rgba = vld1q_u8(input.as_ptr().add(offset));
                    let bgra = vreinterpretq_u8_u32(vandq_u32(
                        vreinterpretq_u32_u8(vqtbl1q_u8(rgba, swap)),
                        keep,
                    ));
                    vst1q_u8(output.as_mut_ptr().add(at + block * 4).cast::<u8>(), bgra);
                }
                at += 16;
            }
            if at + 8 <= output.len() {
                for block in 0..2usize {
                    let offset = (at + block * 4) * 4;
                    let rgba = vld1q_u8(input.as_ptr().add(offset));
                    let bgra = vreinterpretq_u8_u32(vandq_u32(
                        vreinterpretq_u32_u8(vqtbl1q_u8(rgba, swap)),
                        keep,
                    ));
                    vst1q_u8(output.as_mut_ptr().add(at + block * 4).cast::<u8>(), bgra);
                }
                at += 8;
            }
        }
    }
    let keep = if format == skia::PixelFormat::Bgrx8888 {
        0x0000_ff00
    } else {
        0xff00_ff00
    };
    for (pixel, bytes) in output[at..].iter_mut().zip(input[at * 4..].chunks_exact(4)) {
        let word = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        *pixel = (word & keep) | ((word & 0xff) << 16) | ((word >> 16) & 0xff);
    }
}

pub(crate) fn cached_bgra(rgba: &[u8]) -> Vec<u32> {
    let mut bgra = vec![0; rgba.len() / 4];
    copy_opaque_rgba_row(&mut bgra, rgba, skia::PixelFormat::Bgra8888);
    bgra
}

#[cfg(test)]
#[test]
fn opaque_copy_writes_complete_pixels_in_all_output_formats() {
    for columns in [0, 1, 7, 8, 9, 15, 16, 17, 24, 31, 32, 257] {
        // Exercise an unaligned source and vector/tail boundaries. The guard
        // words must survive the write into the middle of the target row.
        let mut source = vec![0x7b; 1 + columns * 4];
        for (i, pixel) in source[1..].chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&[i as u8, (i * 3) as u8, (i * 7) as u8, 255]);
        }
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            let mut target = vec![0x1234_5678; columns + 2];
            copy_opaque_rgba_row(&mut target[1..1 + columns], &source[1..], format);
            assert_eq!(target[0], 0x1234_5678);
            assert_eq!(target[columns + 1], 0x1234_5678);
            for (i, actual) in target[1..1 + columns].iter().enumerate() {
                let [r, g, b] = [i as u8, (i * 3) as u8, (i * 7) as u8];
                let expected = match format {
                    skia::PixelFormat::Rgba8888 => [r, g, b, 255],
                    skia::PixelFormat::Bgra8888 => [b, g, r, 255],
                    skia::PixelFormat::Bgrx8888 => [b, g, r, 0],
                };
                assert_eq!(actual.to_le_bytes(), expected);
            }
        }
    }
}

fn opaque_covers(composition: LayerComposition, bounds: SurfaceBounds) -> bool {
    composition.opaque_bounds.is_some_and(|rect| {
        rect.x as i64 <= bounds.left as i64
            && rect.y as i64 <= bounds.top as i64
            && i64::from(rect.x) + i64::from(rect.width) >= bounds.right as i64
            && i64::from(rect.y) + i64::from(rect.height) >= bounds.bottom as i64
    })
}

struct AffineTileSource<'a> {
    tile: &'a TilePlacement,
    pixels: &'a CachedTile,
    origin: (f64, f64),
    content_bounds: Option<PaintRect>,
}

fn affine_source_word(source: &AffineTileSource<'_>, x: f64, y: f64) -> Option<u32> {
    let local_x = x - source.origin.0;
    let local_y = y - source.origin.1;
    // Calls are made on the tiling's texel lattice.  Recovering the integer
    // resource index with round absorbs only floating arithmetic drift; the
    // lattice's device-space subpixel phase remains in `x`/`y`.
    let pixel_x = local_x.round();
    let pixel_y = local_y.round();
    if (local_x - pixel_x).abs() > 1.0e-6
        || (local_y - pixel_y).abs() > 1.0e-6
        || pixel_x < 0.0
        || pixel_y < 0.0
        || pixel_x >= f64::from(source.tile.pixel_size.0)
        || pixel_y >= f64::from(source.tile.pixel_size.1)
        || source.content_bounds.is_some_and(|bounds| {
            let x = x + 0.5;
            let y = y + 0.5;
            x < bounds.x
                || y < bounds.y
                || x >= bounds.x + bounds.width
                || y >= bounds.y + bounds.height
        })
    {
        return None;
    }
    if let Some(color) = compact_solid_color(source.pixels) {
        return Some(color);
    }
    let width = source.tile.pixel_size.0 as usize;
    let offset = (pixel_y as usize * width + pixel_x as usize) * 4;
    let bytes = source.pixels.rgba.get(offset..offset + 4)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn affine_sample_word(sources: &[AffineTileSource<'_>], x: f64, y: f64) -> u32 {
    sources
        .iter()
        .find_map(|source| affine_source_word(source, x, y))
        .unwrap_or(0)
}

fn affine_has_owner(
    sources: &[AffineTileSource<'_>],
    composition: LayerComposition,
    x: f64,
    y: f64,
) -> bool {
    if composition
        .projective
        .is_some_and(|projective| projective.source_w(x, y) < 1.0e-6)
    {
        return false;
    }
    sources.iter().any(|source| {
        x >= source.origin.0
            && y >= source.origin.1
            && x < source.origin.0 + f64::from(source.tile.pixel_size.0)
            && y < source.origin.1 + f64::from(source.tile.pixel_size.1)
            && source.content_bounds.is_none_or(|bounds| {
                x >= bounds.x
                    && y >= bounds.y
                    && x < bounds.x + bounds.width
                    && y < bounds.y + bounds.height
            })
    })
}

fn affine_sample_at(sources: &[AffineTileSource<'_>], source_x: f64, source_y: f64) -> u32 {
    let lattice = sources.first().map_or((0.0, 0.0), |source| {
        (
            source.origin.0.rem_euclid(1.0),
            source.origin.1.rem_euclid(1.0),
        )
    });
    let texel_x = source_x - lattice.0 - 0.5;
    let texel_y = source_y - lattice.1 - 0.5;
    let left = texel_x.floor() as i64;
    let top = texel_y.floor() as i64;
    let phase = (texel_x - left as f64, texel_y - top as f64);
    let left = left as f64 + lattice.0;
    let top = top as f64 + lattice.1;
    linear_word::<true, true>(
        affine_sample_word(sources, left, top),
        affine_sample_word(sources, left + 1.0, top),
        affine_sample_word(sources, left, top + 1.0),
        affine_sample_word(sources, left + 1.0, top + 1.0),
        phase,
        (1.0 - phase.0, 1.0 - phase.1),
    )
}

fn affine_pixel(
    sources: &[AffineTileSource<'_>],
    composition: LayerComposition,
    visible_polygon: Option<&viz::TargetPolygon>,
    x: usize,
    y: usize,
) -> u32 {
    let inverse_map = |x: f64, y: f64| match composition.projective {
        Some(projective) => projective.inverse_point(x, y),
        None => composition
            .transform
            .inverse()
            .map(|inverse| inverse.map_point(x, y)),
    };
    // A BSP cut is an ownership boundary, not a new geometric edge. Assign
    // the whole destination pixel by its center using a half-open fill rule;
    // antialiasing both complementary fragments would source-over two partial
    // pixels and leave a translucent seam along the intersection.
    if !target_polygon_contains(visible_polygon, x as f64 + 0.5, y as f64 + 0.5, false) {
        return 0;
    }
    let corners = [(0.01, 0.01), (0.99, 0.01), (0.01, 0.99), (0.99, 0.99)];
    if corners.into_iter().all(|(dx, dy)| {
        let target = (x as f64 + dx, y as f64 + dy);
        target_polygon_contains(visible_polygon, target.0, target.1, true)
            && inverse_map(target.0, target.1)
                .is_some_and(|point| affine_has_owner(sources, composition, point.0, point.1))
    }) {
        let point = inverse_map(x as f64 + 0.5, y as f64 + 0.5).unwrap();
        return affine_sample_at(sources, point.0, point.1);
    }
    // Skia anti-aliases the transformed quad independently from its bilinear
    // image shader. A fixed subpixel grid is used only on edge pixels; it also
    // retains continuous sampling across internal tile boundaries.
    let mut channels = [0u32; 4];
    for dy in [0.125, 0.375, 0.625, 0.875] {
        for dx in [0.125, 0.375, 0.625, 0.875] {
            if !target_polygon_contains(visible_polygon, x as f64 + dx, y as f64 + dy, true) {
                continue;
            }
            let Some(point) = inverse_map(x as f64 + dx, y as f64 + dy) else {
                continue;
            };
            if !affine_has_owner(sources, composition, point.0, point.1) {
                continue;
            }
            let word = affine_sample_at(sources, point.0, point.1);
            for (index, shift) in [0, 8, 16, 24].into_iter().enumerate() {
                channels[index] += (word >> shift) & 255;
            }
        }
    }
    channels
        .into_iter()
        .zip([0, 8, 16, 24])
        .fold(0, |word, (channel, shift)| {
            word | (((channel + 8) / 16) << shift)
        })
}

fn target_polygon_contains(
    polygon: Option<&viz::TargetPolygon>,
    x: f64,
    y: f64,
    antialias_edges: bool,
) -> bool {
    let Some(polygon) = polygon else {
        return true;
    };
    if polygon.points.len() < 3 || polygon.antialias_edges.len() != polygon.points.len() {
        return false;
    }
    let signed_area: f64 = (0..polygon.points.len())
        .map(|index| {
            let a = polygon.points[index];
            let b = polygon.points[(index + 1) % polygon.points.len()];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    if signed_area.abs() <= 1.0e-12 {
        return false;
    }
    let winding = signed_area.signum();
    for index in 0..polygon.points.len() {
        if polygon.antialias_edges[index] != antialias_edges {
            continue;
        }
        let a = polygon.points[index];
        let b = polygon.points[(index + 1) % polygon.points.len()];
        let cross = winding * ((b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0]));
        if cross < -1.0e-9 {
            return false;
        }
        if !antialias_edges && cross.abs() <= 1.0e-9 {
            let dx = winding * (b[0] - a[0]);
            let dy = winding * (b[1] - a[1]);
            if !(dy > 0.0 || (dy == 0.0 && dx < 0.0)) {
                return false;
            }
        }
    }
    true
}

fn clip_coverage_at(coverage: &MaskCoverage, x: usize, y: usize) -> u8 {
    if x < coverage.origin.0
        || y < coverage.origin.1
        || x >= coverage.origin.0 + coverage.stride
        || y >= coverage.origin.1 + coverage.rows.len()
    {
        return 0;
    }
    coverage.pixels[(y - coverage.origin.1) * coverage.stride + x - coverage.origin.0]
}

/// Draw one promoted affine layer as a single logical texture grid.  Selecting
/// the source tile after inverse mapping preserves filtering across retained
/// tile seams; each destination pixel is emitted once even though transformed
/// tile AABBs overlap.
fn compose_affine_layer(
    target: &mut [u32],
    view: TargetView,
    format: skia::PixelFormat,
    composition: LayerComposition,
    visible_polygon: Option<&viz::TargetPolygon>,
    sources: &[AffineTileSource<'_>],
    bounds: Option<SurfaceBounds>,
    coverage: Option<&MaskCoverage>,
) -> io::Result<Option<SurfaceBounds>> {
    let Some(bounds) = bounds else {
        return Ok(None);
    };

    if composition
        .projective
        .is_some_and(|projective| projective.backface_hidden && projective.backface_visible)
    {
        return Ok(None);
    }
    let mut touched = None;
    for y in bounds.top..bounds.bottom {
        let mut row_first = None;
        let mut row_last = 0;
        for x in bounds.left..bounds.right {
            let mut sampled = affine_pixel(sources, composition, visible_polygon, x, y);
            if let Some(coverage) = coverage {
                let alpha = clip_coverage_at(coverage, x, y);
                if alpha == 0 {
                    continue;
                }
                if alpha != 255 {
                    sampled = scale_mask_word(sampled, alpha);
                }
            }
            if sampled == 0 {
                continue;
            }
            let at = view.row_start(y, x);
            target[at] = source_over_rgba_word(target[at], sampled, format);
            row_first.get_or_insert(x);
            row_last = x + 1;
        }
        if let Some(left) = row_first {
            union_bounds(
                &mut touched,
                Some(SurfaceBounds {
                    left,
                    top: y,
                    right: row_last,
                    bottom: y + 1,
                }),
            );
        }
    }
    Ok(touched)
}

fn compose_tile(
    target: &mut [u32],
    width: u32,
    height: u32,
    view: TargetView,
    format: skia::PixelFormat,
    tile: &TilePlacement,
    composition: LayerComposition,
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
    draw_visible: Option<SurfaceBounds>,
) -> io::Result<Option<SurfaceBounds>> {
    let Some((mut bounds, (x, y), phase)) =
        tile_composition_geometry(tile, composition, width, height)?
    else {
        return Ok(None);
    };
    // DrawTileQuad maps visible_rect back to the unchanged source UVs. Keep
    // the original tile origin/phase and shorten only the destination extent.
    if let Some(visible) = draw_visible {
        bounds.left = bounds.left.max(visible.left);
        bounds.top = bounds.top.max(visible.top);
        bounds.right = bounds.right.min(visible.right);
        bounds.bottom = bounds.bottom.min(visible.bottom);
        if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
            return Ok(None);
        }
    }
    let (left, top, right, bottom) = (
        bounds.left as i64,
        bounds.top as i64,
        bounds.right as i64,
        bounds.bottom as i64,
    );
    let touched = Some(bounds);
    let has_leading_fringe = left < x || top < y;
    if !has_leading_fringe
        && layer_solid::try_compose(target, view, bounds, format, pixels, adjacent, phase)
    {
        return Ok(touched);
    }
    if !valid_tile_storage(pixels, tile.pixel_size)? {
        return Err(io::Error::other("invalid cached tile extent"));
    }
    let rgba = &pixels.rgba;
    let source_stride = tile.pixel_size.0 as usize * 4;
    let columns = (right - left) as usize;
    if phase != (0.0, 0.0) {
        // Each phase has one row kernel; only the true tile edges consult
        // adjacent pixels. Interior loads use contiguous preselected rows.
        // Missing neighbors use opaque edge clamping. Any real neighbor
        // participating in interpolation must carry the same exact proof.
        // Only actual shader samples affect opacity: H reads the right tile,
        // V reads the lower tile, and their conjunction reads the diagonal.
        let required = [
            phase.0 != 0.0,
            phase.1 != 0.0,
            phase.0 != 0.0 && phase.1 != 0.0,
        ];
        let opaque = !has_leading_fringe
            && (opaque_covers(composition, bounds)
                || ((composition.white_backing || pixels.row_support.opaque)
                    && adjacent.iter().zip(required).all(|(next, required)| {
                        !required
                            || next.is_none_or(|next| next.white_backing || next.row_support.opaque)
                    })));
        let bounds = touched.expect("nonempty tile composition");
        return Ok(if opaque {
            fractional_tile::<true>(
                target,
                view,
                bounds,
                (x, y),
                pixels,
                adjacent,
                phase,
                format,
            )
        } else {
            fractional_tile::<false>(
                target,
                view,
                bounds,
                (x, y),
                pixels,
                adjacent,
                phase,
                format,
            )
        });
    }
    if composition.white_backing || pixels.row_support.opaque || opaque_covers(composition, bounds)
    {
        // Either the backing guarantees opacity or the tile's actual pixels
        // prove it. SrcOver then equals copying/swizzling this clipped row.
        let native_bgra =
            matches!(format, skia::PixelFormat::Bgra8888) && pixels.bgra.len() == rgba.len() / 4;
        for row in top..bottom {
            let source = (row - y) as usize * source_stride + (left - x) as usize * 4;
            let destination = view.row_start(row as usize, left as usize);
            let output = &mut target[destination..destination + columns];
            if native_bgra {
                let source = source / 4;
                output.copy_from_slice(&pixels.bgra[source..source + columns]);
            } else {
                let input = &rgba[source..source + columns * 4];
                copy_opaque_rgba_row(output, input, format);
            }
        }
        return Ok(touched);
    }
    let mut written = None;
    for row in top..bottom {
        let sy = (row - y) as usize;
        let (support_left, support_right) = pixels.row_support.rows[sy].extent;
        let first = ((left - x) as usize).max(support_left);
        let last = ((right - x) as usize).min(support_right);
        if first >= last {
            continue;
        }
        // The cached runs exclude only complete RGBA-zero holes. Integer
        // placement can skip the same proven no-op pixels as interpolation.
        for &(span_left, span_right) in pixels.row_support.spans(sy) {
            let span_left = first.max(span_left);
            let span_right = last.min(span_right);
            if span_left >= span_right {
                continue;
            }
            let source = sy * source_stride + span_left * 4;
            let destination = view.row_start(row as usize, (x + span_left as i64) as usize);
            let input = &rgba[source..source + (span_right - span_left) * 4];
            let output = &mut target[destination..destination + span_right - span_left];
            // SoftwareRenderer/SkBlitRow: opaque SrcOver equals Src. The
            // immutable product proves every byte alpha in this row extent;
            // no destination read or per-frame alpha scan is needed. Filtered
            // fractional samples keep their separate complete-footprint proof.
            if pixels.row_support.rows[sy].opaque {
                if matches!(format, skia::PixelFormat::Bgra8888)
                    && pixels.bgra.len() == rgba.len() / 4
                {
                    let source = source / 4;
                    output.copy_from_slice(&pixels.bgra[source..source + output.len()]);
                } else {
                    copy_opaque_rgba_row(output, input, format);
                }
            } else {
                // SAFETY: exclusive initialized complete N32 pixel row; no
                // alias survives this little-endian byte view.
                let output_bytes = unsafe {
                    std::slice::from_raw_parts_mut(
                        output.as_mut_ptr().cast::<u8>(),
                        std::mem::size_of_val(output),
                    )
                };
                skia::blend_premultiplied_rgba_row(output_bytes, input, format);
            }
        }
        union_bounds(
            &mut written,
            Some(SurfaceBounds {
                left: (x + first as i64) as usize,
                right: (x + last as i64) as usize,
                top: row as usize,
                bottom: row as usize + 1,
            }),
        );
    }
    Ok(written)
}

#[inline]
fn source_over_word(destination: u32, source: [u8; 4], format: skia::PixelFormat) -> u32 {
    source_over_rgba_word(destination, u32::from_le_bytes(source), format)
}

#[inline]
fn source_over_rgba_word(destination: u32, mut source: u32, format: skia::PixelFormat) -> u32 {
    if source == 0 {
        return destination;
    }
    let inverse = 255 - (source >> 24);
    if format != skia::PixelFormat::Rgba8888 {
        source = (source & 0xff00_ff00) | ((source & 0xff) << 16) | ((source >> 16) & 0xff);
    }
    if inverse != 0 {
        // Same rounded div255 as multiply_255, two isolated 16-bit lanes
        // at a time. Premultiplied source-over cannot overflow a byte.
        let mut rb = (destination & 0x00ff_00ff) * inverse + 0x0080_0080;
        rb = ((rb + ((rb >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
        let mut ga = ((destination >> 8) & 0x00ff_00ff) * inverse + 0x0080_0080;
        ga = ((ga + ((ga >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
        source += rb | (ga << 8);
    }
    if format == skia::PixelFormat::Bgrx8888 {
        source &= 0x00ff_ffff;
    }
    source
}

#[inline]
fn scale_mask_word(source: u32, coverage: u8) -> u32 {
    let coverage = u32::from(coverage);
    // Two independent 16-bit lanes preserve multiply_255's rounded div255.
    // Each product + rounding adjustment stays below 65536, so no lane carries.
    let mut rb = (source & 0x00ff_00ff) * coverage + 0x0080_0080;
    rb = ((rb + ((rb >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
    let mut ga = ((source >> 8) & 0x00ff_00ff) * coverage + 0x0080_0080;
    ga = ((ga + ((ga >> 8) & 0x00ff_00ff)) >> 8) & 0x00ff_00ff;
    rb | (ga << 8)
}

fn blend_rgba_words(target: &mut [u32], source: &[u32], format: skia::PixelFormat) {
    // SAFETY: both rows contain initialized complete N32 words. Their byte
    // views have identical extents; target is exclusively borrowed and the
    // independently allocated source surface/scratch cannot alias it.
    let destination = unsafe {
        std::slice::from_raw_parts_mut(
            target.as_mut_ptr().cast::<u8>(),
            std::mem::size_of_val(target),
        )
    };
    let rgba = unsafe {
        std::slice::from_raw_parts(source.as_ptr().cast::<u8>(), std::mem::size_of_val(source))
    };
    skia::blend_premultiplied_rgba_row(destination, rgba, format);
}

fn blend_group_row(target: &mut [u32], source: &[u32], format: skia::PixelFormat) {
    if format == skia::PixelFormat::Bgrx8888 {
        // General callers can retain arbitrary raw X at an exact-zero source.
        blend_rgba_words(target, source, skia::PixelFormat::Bgra8888);
        for (dst, &src) in target.iter_mut().zip(source) {
            if src != 0 {
                *dst &= 0x00ff_ffff;
            }
        }
    } else {
        blend_rgba_words(target, source, format);
    }
}

fn blend_compositor_row(target: &mut [u32], source: &[u32], format: skia::PixelFormat) {
    #[cfg(feature = "compose_work_profile")]
    layer_compose_profile::blend(target.len());
    // Called only for final-frame/effect surfaces. Final BGRX starts with
    // X=0 and all admitted composition writes preserve X=0; effect parents
    // are RGBA. The existing BGRX SIMD store clears X in its blend pass,
    // avoiding the general helper's second complete source/destination scan.
    blend_rgba_words(target, source, format);
}

/// Blend an isolated effect surface with the same Skia raster-pipeline modes
/// used by ordinary paint replay. Non-normal effects are uncommon, so a
/// bounded row scratch keeps the compositor's retained surface format out of
/// Skia's RGBA-only Pixmap adapter without adding a second blend implementation.
fn blend_effect_row(
    target: &mut [u32],
    source: &[u32],
    format: skia::PixelFormat,
    mode: PaintBlendMode,
) {
    if mode == PaintBlendMode::kNormal {
        blend_compositor_row(target, source, format);
        return;
    }
    skia::cpu::effect_composite::blend_effect_row(
        target,
        source,
        format,
        mode.to_skia(),
        1.0,
        None,
    );
}

#[cfg(test)]
fn swap_red_blue(word: u32) -> u32 {
    (word & 0xff00_ff00) | ((word & 0x0000_00ff) << 16) | ((word & 0x00ff_0000) >> 16)
}

#[cfg(target_arch = "aarch64")]
unsafe fn scale_effect_neon(
    output: &mut [u32],
    input: &[u32],
    opacity: f32,
    coverage: Option<&[u8]>,
) -> (usize, u32) {
    use core::arch::aarch64::*;
    unsafe fn alpha_half(bytes: uint8x8_t, opacity: f32) -> uint8x8_t {
        unsafe {
            let wide = vmovl_u8(bytes);
            // FCVTN rounds the same f32 product to nearest, ties-even. Do not
            // replace opacity with an 8-bit factor or fuse it with mask alpha.
            let low = vcvtnq_u32_f32(vmulq_n_f32(
                vcvtq_f32_u32(vmovl_u16(vget_low_u16(wide))),
                opacity,
            ));
            let high = vcvtnq_u32_f32(vmulq_n_f32(
                vcvtq_f32_u32(vmovl_u16(vget_high_u16(wide))),
                opacity,
            ));
            vmovn_u16(vcombine_u16(vmovn_u32(low), vmovn_u32(high)))
        }
    }
    unsafe fn mask_half(bytes: uint8x8_t, alpha: uint8x8_t) -> uint8x8_t {
        unsafe {
            let product = vmull_u8(bytes, alpha);
            vraddhn_u16(product, vrshrq_n_u16::<8>(product))
        }
    }
    let mut at = 0;
    let mut any = 0;
    // Caller provides complete, equally sized pixel rows and coverage spans.
    // Unaligned loads/stores stay within four complete pixels; masks read only
    // their four real coverage bytes. Source and scratch are separate buffers.
    unsafe {
        let repeat = vld1q_u8([0u8, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3].as_ptr());
        let mut seen = vdupq_n_u32(0);
        while at + 4 <= input.len() {
            let mut bytes = vreinterpretq_u8_u32(vld1q_u32(input.as_ptr().add(at)));
            if opacity != 1.0 {
                bytes = vcombine_u8(
                    alpha_half(vget_low_u8(bytes), opacity),
                    alpha_half(vget_high_u8(bytes), opacity),
                );
            }
            if let Some(mask) = coverage {
                let four = std::ptr::read_unaligned(mask.as_ptr().add(at).cast::<u32>());
                let alpha = vqtbl1q_u8(vreinterpretq_u8_u32(vdupq_n_u32(four)), repeat);
                bytes = vcombine_u8(
                    mask_half(vget_low_u8(bytes), vget_low_u8(alpha)),
                    mask_half(vget_high_u8(bytes), vget_high_u8(alpha)),
                );
            }
            let words = vreinterpretq_u32_u8(bytes);
            vst1q_u32(output.as_mut_ptr().add(at), words);
            seen = vorrq_u32(seen, words);
            at += 4;
        }
        any |= vmaxvq_u32(seen);
    }
    (at, any)
}

fn scale_effect_row(
    output: &mut [u32],
    input: &[u32],
    opacity: f32,
    alpha: Option<&[u8; 256]>,
    mask: Option<&[u8]>,
) -> u32 {
    let mut at = 0;
    let mut any = 0;
    #[cfg(target_arch = "aarch64")]
    if opacity.is_finite() && (0.0..=1.0).contains(&opacity) {
        // SAFETY: all spans cover input.len() complete pixels, established by
        // the group row/chunk slices; the helper never crosses their ends.
        (at, any) = unsafe { scale_effect_neon(output, input, opacity, mask) };
    }
    for i in at..input.len() {
        let coverage = mask.map_or(255, |row| row[i]);
        let mut word = if input[i] == 0 || coverage == 0 {
            0
        } else {
            input[i]
        };
        if word != 0 {
            if let Some(alpha) = alpha {
                word = u32::from(alpha[(word & 255) as usize])
                    | (u32::from(alpha[((word >> 8) & 255) as usize]) << 8)
                    | (u32::from(alpha[((word >> 16) & 255) as usize]) << 16)
                    | (u32::from(alpha[(word >> 24) as usize]) << 24);
            }
            if coverage != 255 {
                word = scale_mask_word(word, coverage);
            }
        }
        output[i] = word;
        any |= word;
    }
    any
}

// Tile geometry is the actual clipped draw-quad partition. Clear everything
// else once, including a completely offscreen mask, before applying any quad.
fn clear_mask_exterior(
    target: &mut [u32],
    view: TargetView,
    touched: Option<SurfaceBounds>,
    rectangles: &[SurfaceBounds],
) -> Option<SurfaceBounds> {
    let bounds = touched?;
    let mut retained = None;
    for rect in rectangles {
        let overlap = SurfaceBounds {
            left: bounds.left.max(rect.left),
            top: bounds.top.max(rect.top),
            right: bounds.right.min(rect.right),
            bottom: bounds.bottom.min(rect.bottom),
        };
        if overlap.left < overlap.right && overlap.top < overlap.bottom {
            union_bounds(&mut retained, Some(overlap));
        }
    }
    let mut first = 0;
    for y in bounds.top..bounds.bottom {
        while first < rectangles.len() && rectangles[first].bottom <= y {
            first += 1;
        }
        let mut at = first;
        let mut cursor = bounds.left;
        while at < rectangles.len() && rectangles[at].top <= y {
            let rect = rectangles[at];
            let left = rect.left.clamp(bounds.left, bounds.right);
            let right = rect.right.clamp(bounds.left, bounds.right);
            if cursor < left {
                target[view.row_range(y, cursor, left)].fill(0);
            }
            cursor = cursor.max(right);
            at += 1;
        }
        if cursor < bounds.right {
            target[view.row_range(y, cursor, bounds.right)].fill(0);
        }
    }
    retained
}
fn dst_in_tile(
    target: &mut [u32],
    view: TargetView,
    touched: Option<SurfaceBounds>,
    width: u32,
    height: u32,
    tile: &TilePlacement,
    composition: LayerComposition,
    pixels: &CachedTile,
    adjacent: [Option<&CachedTile>; 3],
) -> io::Result<()> {
    if !valid_tile_storage(pixels, tile.pixel_size)? {
        return Err(io::Error::other("invalid direct mask tile extent"));
    }
    let Some(parent) = touched else {
        return Ok(());
    };
    let Some((mut bounds, (x, y), phase)) =
        tile_composition_geometry(tile, composition, width, height)?
    else {
        return Ok(());
    };
    bounds.left = bounds.left.max(parent.left);
    bounds.right = bounds.right.min(parent.right);
    bounds.top = bounds.top.max(parent.top);
    bounds.bottom = bounds.bottom.min(parent.bottom);
    if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
        return Ok(());
    }
    if phase == (0.0, 0.0) && compact_solid_color(pixels).is_some() {
        // The analyzer admits only opaque solids. DstIn(alpha=1) is identity
        // inside this tile; clear_mask_exterior already handled the outside.
        return Ok(());
    }
    let stride = tile.pixel_size.0 as usize * 4;
    let sampler = PreparedFractionalSampler::new(phase);
    for row in bounds.top..bounds.bottom {
        if phase != (0.0, 0.0) {
            let output = &mut target[view.row_range(row, bounds.left, bounds.right)];
            // Transparent sampler skips must remain A=0 for DstIn. Bounded
            // chunks preserve the same global source coordinate and neighbors.
            let sy = row as i64 - y;
            let first = bounds.left as i64 - x;
            for (chunk, output) in output.chunks_mut(256).enumerate() {
                let mut sampled = [0u32; 256];
                let sampled = &mut sampled[..output.len()];
                let sx = first + (chunk * 256) as i64;
                fractional_row_signed::<false, false>(
                    sampled,
                    pixels,
                    adjacent,
                    sy,
                    sx,
                    &sampler,
                    skia::PixelFormat::Rgba8888,
                );
                dst_in_row(output, sampled, 1.0);
            }
            continue;
        }
        let start = (row as i64 - y) as usize * stride + (bounds.left as i64 - x) as usize * 4;
        let input = &pixels.rgba[start..start + (bounds.right - bounds.left) * 4];
        let output = &mut target[view.row_range(row, bounds.left, bounds.right)];
        // Source RGBA bytes are initialized and independent of the destination.
        // Preserve the existing N32 DstIn arithmetic for either alignment.
        let (prefix, words, suffix) = unsafe { input.align_to::<u32>() };
        if prefix.is_empty() && suffix.is_empty() {
            dst_in_row(output, words, 1.0);
        } else {
            let mut scratch = [0u32; 256];
            for (dst, bytes) in output.chunks_mut(256).zip(input.chunks(256 * 4)) {
                for (word, pixel) in scratch.iter_mut().zip(bytes.chunks_exact(4)) {
                    *word = u32::from_le_bytes([pixel[0], pixel[1], pixel[2], pixel[3]]);
                }
                dst_in_row(dst, &scratch[..dst.len()], 1.0);
            }
        }
    }
    Ok(())
}

// SoftwareRenderer's Mask/kDstIn restore scales the entire isolated content
// by the source alpha, including transparent source outside the mask quads.
// Tiles first form one source surface; never apply DstIn independently per tile.
fn dst_in_effect_surface(
    target: &mut [u32],
    source: &[u32],
    target_touched: Option<SurfaceBounds>,
    source_touched: Option<SurfaceBounds>,
    target_view: TargetView,
    source_view: TargetView,
    opacity: f32,
) -> Option<SurfaceBounds> {
    let bounds = target_touched?;
    let overlap = source_touched
        .and_then(|source| {
            let clipped = SurfaceBounds {
                left: bounds.left.max(source.left),
                top: bounds.top.max(source.top),
                right: bounds.right.min(source.right),
                bottom: bounds.bottom.min(source.bottom),
            };
            (clipped.left < clipped.right && clipped.top < clipped.bottom).then_some(clipped)
        })
        .filter(|_| opacity != 0.0);
    for y in bounds.top..bounds.bottom {
        let row = &mut target[target_view.row_range(y, bounds.left, bounds.right)];
        if let Some(inside) = overlap.filter(|inside| y >= inside.top && y < inside.bottom) {
            let left = inside.left - bounds.left;
            let right = inside.right - bounds.left;
            row[..left].fill(0);
            row[right..].fill(0);
            let input = &source[source_view.row_range(y, inside.left, inside.right)];
            dst_in_row(&mut row[left..right], input, opacity);
        } else {
            row.fill(0);
        }
    }
    overlap
}
fn dst_in_row(target: &mut [u32], source: &[u32], opacity: f32) {
    #[allow(unused_mut)]
    let mut done = 0;
    #[cfg(target_arch = "aarch64")]
    if opacity == 1.0 {
        use core::arch::aarch64::*;
        // Corresponding N32 DstIn: round each destination channel * source A
        // with Skia's div255. The source is ordinary resident RGBA mask pixels.
        // SAFETY: equal complete rows, unaligned four-pixel loads stay in bounds.
        unsafe {
            let repeat =
                vld1q_u8([3u8, 3, 3, 3, 7, 7, 7, 7, 11, 11, 11, 11, 15, 15, 15, 15].as_ptr());
            while done + 4 <= target.len() {
                let dst = vreinterpretq_u8_u32(vld1q_u32(target.as_ptr().add(done)));
                let src = vreinterpretq_u8_u32(vld1q_u32(source.as_ptr().add(done)));
                let alpha = vqtbl1q_u8(src, repeat);
                let low = vmull_u8(vget_low_u8(dst), vget_low_u8(alpha));
                let high = vmull_u8(vget_high_u8(dst), vget_high_u8(alpha));
                let result = vcombine_u8(
                    vraddhn_u16(low, vrshrq_n_u16::<8>(low)),
                    vraddhn_u16(high, vrshrq_n_u16::<8>(high)),
                );
                vst1q_u32(target.as_mut_ptr().add(done), vreinterpretq_u32_u8(result));
                done += 4;
            }
        }
    }
    for (dst, &src) in target[done..].iter_mut().zip(&source[done..]) {
        let alpha = ((src >> 24) as f32 * opacity).round_ties_even() as u8;
        *dst = scale_mask_word(*dst, alpha);
    }
}

fn composite_effect_surface(
    target: &mut [u32],
    source: &[u32],
    touched: Option<SurfaceBounds>,
    target_view: TargetView,
    source_view: TargetView,
    format: skia::PixelFormat,
    opacity: f32,
    blend_mode: PaintBlendMode,
    mask: Option<&MaskCoverage>,
    visible_polygon: Option<&viz::TargetPolygon>,
) -> Option<SurfaceBounds> {
    let mut bounds = touched?;
    // Every premultiplied 8-bit channel rounds to zero at this opacity. Skia's
    // alpha-zero draw is a no-op as well; avoid walking a full render pass for
    // animation values that are nonzero only in floating-point representation.
    if !effect_opacity_has_output(opacity) {
        return None;
    }
    if let Some(mask) = mask {
        let mask_bounds = mask.bounds?;
        bounds.left = bounds.left.max(mask_bounds.0);
        bounds.top = bounds.top.max(mask_bounds.1);
        bounds.right = bounds.right.min(mask_bounds.2);
        bounds.bottom = bounds.bottom.min(mask_bounds.3);
        if bounds.left >= bounds.right || bounds.top >= bounds.bottom {
            return None;
        }
    }
    let mut trace = browser_tracing::span("compose", "CompositeEffectSurface");
    trace.set(
        "write_pixels",
        ((bounds.right - bounds.left) * (bounds.bottom - bounds.top)) as f64,
    );
    trace.set(
        "mask_pixels",
        mask.map_or(0, |mask| mask.pixels.len()) as f64,
    );
    trace.set("polygon", visible_polygon.is_some() as u8 as f64);
    trace.set("opacity", opacity as f64);
    // Same N32 restore mapping as SkCanvas: scale each premultiplied channel
    // in f32 and round ties-even, rather than quantizing opacity to an 8-bit alpha.
    let alpha: Option<[u8; 256]> = (opacity != 1.0)
        .then(|| std::array::from_fn(|v| (v as f32 * opacity).round_ties_even() as u8));
    // Fixed stack scratch, independent of viewport width or allocation pools.
    let mut scratch = [0u32; 256];
    for y in bounds.top..bounds.bottom {
        let (left, right) = mask.map_or((bounds.left, bounds.right), |mask| {
            let (left, right) = mask.rows[y - mask.origin.1];
            (bounds.left.max(left), bounds.right.min(right))
        });
        if left >= right {
            continue;
        }
        let target_range = target_view.row_range(y, left, right);
        let source_range = source_view.row_range(y, left, right);
        let coverage = mask.map(|mask| {
            let offset = (y - mask.origin.1) * mask.stride + left - mask.origin.0;
            &mask.pixels[offset..offset + right - left]
        });
        if visible_polygon.is_some() {
            for x in left..right {
                if !target_polygon_contains(visible_polygon, x as f64 + 0.5, y as f64 + 0.5, false)
                {
                    continue;
                }
                let mut word = source[source_view.row_start(y, x)];
                let mut samples = 0u16;
                for dy in [0.125, 0.375, 0.625, 0.875] {
                    for dx in [0.125, 0.375, 0.625, 0.875] {
                        samples += u16::from(target_polygon_contains(
                            visible_polygon,
                            x as f64 + dx,
                            y as f64 + dy,
                            true,
                        ));
                    }
                }
                let clip_alpha = mask.map_or(255, |mask| {
                    mask.pixels[(y - mask.origin.1) * mask.stride + x - mask.origin.0]
                });
                let polygon_alpha = ((u32::from(samples) * 255 + 8) / 16) as u8;
                let combined =
                    ((u16::from(clip_alpha) * u16::from(polygon_alpha) + 127) / 255) as u8;
                word = scale_mask_word(word, combined);
                if opacity != 1.0 {
                    let bytes = word.to_le_bytes();
                    word = u32::from_le_bytes(
                        bytes.map(|v| (v as f32 * opacity).round_ties_even() as u8),
                    );
                }
                if word >> 24 != 0 {
                    let target_index = target_view.row_start(y, x);
                    blend_effect_row(
                        std::slice::from_mut(&mut target[target_index]),
                        std::slice::from_ref(&word),
                        format,
                        blend_mode,
                    );
                }
            }
            continue;
        }
        if alpha.is_none() && coverage.is_none() {
            // A premultiplied transparent backdrop is the identity source
            // boundary for SrcOver, including CSS blend modes: with Ab=0 the
            // blend contribution from the backdrop is zero. Render surfaces
            // begin transparent, so their first covering child can copy rows
            // directly instead of running a full per-pixel blend pipeline.
            if format == skia::PixelFormat::Rgba8888
                && (target[target_range.clone()].iter().all(|&pixel| pixel == 0)
                    || (blend_mode == PaintBlendMode::kNormal
                        && source[source_range.clone()]
                            .iter()
                            .all(|&pixel| pixel & 0xff00_0000 == 0xff00_0000)))
            {
                target[target_range].copy_from_slice(&source[source_range]);
                continue;
            }
            blend_effect_row(
                &mut target[target_range],
                &source[source_range],
                format,
                blend_mode,
            );
            continue;
        }
        for (chunk, (output, input)) in target[target_range]
            .chunks_mut(256)
            .zip(source[source_range].chunks(256))
            .enumerate()
        {
            let mask = coverage.map(|row| &row[chunk * 256..chunk * 256 + input.len()]);
            if mask.is_some_and(|row| row.iter().all(|&v| v == 0)) {
                continue;
            }
            let opaque_mask = mask.is_none_or(|row| row.iter().all(|&v| v == 255));
            if alpha.is_none() && opaque_mask {
                if format == skia::PixelFormat::Rgba8888
                    && (output.iter().all(|&pixel| pixel == 0)
                        || (blend_mode == PaintBlendMode::kNormal
                            && input
                                .iter()
                                .all(|&pixel| pixel & 0xff00_0000 == 0xff00_0000)))
                {
                    output.copy_from_slice(input);
                } else {
                    blend_effect_row(output, input, format, blend_mode);
                }
                continue;
            }
            let scaled = &mut scratch[..input.len()];
            // Opacity then true DstIn, preserving the original rounding at
            // both stages. SIMD only replaces the independent channel loop.
            let any = scale_effect_row(
                scaled,
                input,
                opacity,
                alpha.as_ref(),
                (!opaque_mask).then_some(mask).flatten(),
            );
            if any != 0 {
                if format == skia::PixelFormat::Rgba8888
                    && output.iter().all(|&pixel| pixel == 0)
                {
                    output.copy_from_slice(scaled);
                } else {
                    blend_effect_row(output, scaled, format, blend_mode);
                }
            }
        }
    }
    Some(bounds)
}

#[cfg(test)]
mod effect_blend_test {
    use super::*;

    fn encode(rgba: u32, format: skia::PixelFormat) -> u32 {
        match format {
            skia::PixelFormat::Rgba8888 => rgba,
            skia::PixelFormat::Bgra8888 => swap_red_blue(rgba),
            skia::PixelFormat::Bgrx8888 => swap_red_blue(rgba) & 0x00ff_ffff,
        }
    }

    #[test]
    fn plus_lighter_effect_uses_skia_blend_for_every_output_format() {
        // Premultiplied RGBA bytes: kPlus saturates each channel independently.
        let destination = 0xc878_5030;
        let source = 0x8060_2040;
        let expected = 0xffd8_7070;
        for format in [
            skia::PixelFormat::Rgba8888,
            skia::PixelFormat::Bgra8888,
            skia::PixelFormat::Bgrx8888,
        ] {
            let mut actual = [encode(destination, format)];
            blend_effect_row(&mut actual, &[source], format, PaintBlendMode::kPlusLighter);
            let expected = if format == skia::PixelFormat::Bgrx8888 {
                // An opaque presentation target remains opaque/X=0.
                encode(expected | 0xff00_0000, format)
            } else {
                encode(expected, format)
            };
            assert_eq!(actual[0], expected, "format={format:?}");
        }
    }
}

#[inline]
fn multiply_255(channel: u8, alpha: u8) -> u16 {
    let product = channel as u16 * alpha as u16 + 128;
    (product + (product >> 8)) >> 8
}

#[cfg(all(test, target_arch = "aarch64"))]
mod effect_row_test {
    use super::*;

    #[test]
    fn effect_simd_preserves_opacity_then_mask_rounding() {
        let input: Vec<u32> = (0u32..257)
            .map(|v| {
                if v == 256 {
                    0
                } else {
                    v | ((255 - v) << 8) | (((v + 127) & 255) << 16) | (((v + 63) & 255) << 24)
                }
            })
            .collect();
        let mut output = vec![0; input.len()];
        for opacity in [
            0.0f32,
            f32::MIN_POSITIVE,
            0.001,
            0.1,
            0.25,
            0.5,
            0.73,
            0.9,
            f32::from_bits(1.0f32.to_bits() - 1),
            1.0,
        ] {
            let alpha = (opacity != 1.0)
                .then(|| std::array::from_fn(|v| (v as f32 * opacity).round_ties_even() as u8));
            for coverage in 0..=256u16 {
                let mask: Vec<u8> = (0..input.len())
                    .map(|i| (coverage as usize + i * 17) as u8)
                    .collect();
                let mask = (coverage != 256).then_some(mask.as_slice());
                let any = scale_effect_row(&mut output, &input, opacity, alpha.as_ref(), mask);
                let mut reference_any = 0u32;
                for (index, (&src, &actual)) in input.iter().zip(&output).enumerate() {
                    let mut reference = 0u32;
                    for shift in [0, 8, 16, 24] {
                        let value =
                            (((src >> shift) & 255) as f32 * opacity).round_ties_even() as u8;
                        let value = multiply_255(value, mask.map_or(255, |row| row[index]));
                        reference |= u32::from(value) << shift;
                    }
                    assert_eq!(
                        actual, reference,
                        "opacity={opacity} coverage={coverage} pixel={index}"
                    );
                    reference_any |= reference;
                }
                assert_eq!(any != 0, reference_any != 0);
            }
        }
    }
}

#[cfg(test)]
mod direct_mask_quad_test {
    use super::*;

    #[test]
    fn fractional_direct_mask_matches_isolated_source_with_holes_neighbors_and_clip() {
        let (width, height) = (112u32, 14u32);
        let composition = LayerComposition {
            transform: viz::AffineTransform {
                tx: 3.625,
                ty: 2.6875,
                ..viz::AffineTransform::IDENTITY
            },
            projective: None,
            rendering_context_id: 0,
            clip: Some(viz::DeviceRect {
                x: 5,
                y: 3,
                width: 85,
                height: 8,
            }),
            content_bounds: None,
            opaque_bounds: None,
            white_backing: false,
            grid_min: None,
        };
        let mut tiles = Vec::new();
        for index in 0..4usize {
            let (column, row) = (index % 2, index / 2);
            let rect = PaintRect {
                x: (column * 48) as f64,
                y: (row * 4) as f64,
                width: 48.0,
                height: 4.0,
            };
            let placement = TilePlacement {
                tile_id: TileId(index as u64 + 1),
                generation: 1,
                tile_index: (column as i32, row as i32),
                tiling_rect: rect,
                tile_rect: rect,
                raster_scale: 1.0,
                pixel_size: (48, 4),
                ready: true,
            };
            let rgba: Vec<u8> = (0..4usize)
                .flat_map(|y| {
                    (0..48usize).flat_map(move |x| {
                        let (gx, gy) = (column * 48 + x, row * 4 + y);
                        // Long exact-zero holes exercise support skips; fractional AA
                        // alpha and true neighbor edges remain present around them.
                        let a = if (6..26).contains(&(gx % 48)) || gy == 5 {
                            0
                        } else {
                            ((gx * 37 + gy * 53 + 91) % 256) as u8
                        };
                        [a / 3, a / 2, a, a]
                    })
                })
                .collect();
            let cached = CachedTile {
                raster_frame: 0,
                generation: 1,
                pixel_size: (48, 4),
                raster_scale: 1.0,
                white_backing: false,
                row_support: tile_row_support(&rgba, 48, false),
                rgba,
                bgra: Vec::new(),
            };
            tiles.push((placement, cached));
        }
        let adjacent = |index: usize| {
            [
                (index % 2 == 0).then(|| &tiles[index + 1].1),
                (index < 2).then(|| &tiles[index + 2].1),
                (index == 0).then(|| &tiles[3].1),
            ]
        };
        let source_view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        let mut source = vec![0; width as usize * height as usize];
        let mut source_touched = None;
        let mut rectangles = Vec::new();
        for (index, (tile, pixels)) in tiles.iter().enumerate() {
            union_bounds(
                &mut source_touched,
                compose_tile(
                    &mut source,
                    width,
                    height,
                    source_view,
                    skia::PixelFormat::Rgba8888,
                    tile,
                    composition,
                    pixels,
                    adjacent(index),
                    None,
                )
                .unwrap(),
            );
            rectangles.push(
                tile_composition_geometry(tile, composition, width, height)
                    .unwrap()
                    .unwrap()
                    .0,
            );
        }
        rectangles.sort_unstable_by_key(|b| (b.top, b.left, b.bottom, b.right));
        // Content ROI and stride differ from the source surface, with padding
        // left observable so accidental whole-row clearing is caught too.
        let view = TargetView {
            origin: (1, 1),
            stride: 109,
        };
        let touched = Some(SurfaceBounds {
            left: 1,
            top: 1,
            right: 108,
            bottom: 13,
        });
        let original: Vec<u32> = (0..109 * 12)
            .map(|i| {
                let a = ((i * 19 + 67) % 256) as u32;
                (a / 4) | ((a / 2) << 8) | ((a / 3) << 16) | (a << 24)
            })
            .collect();
        let mut expected = original.clone();
        dst_in_effect_surface(
            &mut expected,
            &source,
            touched,
            source_touched,
            view,
            source_view,
            1.0,
        );
        let mut actual = original;
        let retained = clear_mask_exterior(&mut actual, view, touched, &rectangles);
        for (index, (tile, pixels)) in tiles.iter().enumerate() {
            dst_in_tile(
                &mut actual,
                view,
                retained,
                width,
                height,
                tile,
                composition,
                pixels,
                adjacent(index),
            )
            .unwrap();
        }
        assert_eq!(actual, expected);
    }
}

#[cfg(test)]
mod opaque_quad_test {
    use super::*;

    #[test]
    fn bounded_opaque_quad_matches_src_over_with_transparent_padding_and_fractional_sampling() {
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 20.0,
        };
        let proof = PaintRect {
            x: 3.0,
            y: 2.0,
            width: 26.0,
            height: 16.0,
        };
        let tile = TilePlacement {
            tile_id: TileId(1),
            generation: 1,
            tile_index: (0, 0),
            tiling_rect: rect,
            tile_rect: rect,
            raster_scale: 1.0,
            pixel_size: (32, 20),
            ready: true,
        };
        let rgba: Vec<u8> = (0..20usize)
            .flat_map(|y| {
                (0..32usize).flat_map(move |x| {
                    if (3..29).contains(&x) && (2..18).contains(&y) {
                        [(x * 7) as u8, (y * 11) as u8, 91, 255]
                    } else {
                        [0; 4]
                    }
                })
            })
            .collect();
        let pixels = CachedTile {
            raster_frame: 0,
            generation: 1,
            pixel_size: (32, 20),
            raster_scale: 1.0,
            white_backing: false,
            row_support: tile_row_support(&rgba, 32, false),
            rgba,
        };
        assert!(
            !pixels.row_support.opaque,
            "allocation padding is transparent"
        );
        let (width, height) = (48, 32);
        let view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        for translation in [(4.0, 3.0), (4.375, 3.625), (-1.625, -0.375)] {
            for format in [
                skia::PixelFormat::Rgba8888,
                skia::PixelFormat::Bgra8888,
                skia::PixelFormat::Bgrx8888,
            ] {
                let opaque = sampled_opaque_device_rect_for_test(proof, 1.0, translation)
                    .unwrap()
                    .unwrap();
                let safe = SurfaceBounds {
                    left: opaque.x.max(0) as usize,
                    top: opaque.y.max(0) as usize,
                    right: (i64::from(opaque.x) + i64::from(opaque.width)) as usize,
                    bottom: (i64::from(opaque.y) + i64::from(opaque.height)) as usize,
                };
                let base = LayerComposition {
                    transform: viz::AffineTransform {
                        tx: translation.0,
                        ty: translation.1,
                        ..viz::AffineTransform::IDENTITY
                    },
                    projective: None,
                    rendering_context_id: 0,
                    clip: None,
                    content_bounds: None,
                    opaque_bounds: None,
                    white_backing: false,
                    grid_min: None,
                };
                let optimized = LayerComposition {
                    opaque_bounds: Some(opaque),
                    ..base
                };
                assert!(opaque_covers(optimized, safe));
                assert!(!opaque_covers(
                    optimized,
                    SurfaceBounds {
                        right: safe.right + 1,
                        ..safe
                    }
                ));
                for visible in [
                    Some(safe),
                    Some(SurfaceBounds {
                        right: safe.right + 1,
                        ..safe
                    }),
                    None,
                ] {
                    let background = if format == skia::PixelFormat::Bgrx8888 {
                        0x005f3921
                    } else {
                        0x9f5f3921
                    };
                    let mut expected = vec![background; (width * height) as usize];
                    let mut actual = expected.clone();
                    compose_tile(
                        &mut expected,
                        width,
                        height,
                        view,
                        format,
                        &tile,
                        base,
                        &pixels,
                        [None; 3],
                        visible,
                    )
                    .unwrap();
                    compose_tile(
                        &mut actual,
                        width,
                        height,
                        view,
                        format,
                        &tile,
                        optimized,
                        &pixels,
                        [None; 3],
                        visible,
                    )
                    .unwrap();
                    assert_eq!(
                        actual, expected,
                        "translation={translation:?} format={format:?}"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod backdrop_filter_test {
    use super::*;

    #[test]
    fn backdrop_filters_parent_pixels_before_group_content_and_clips_output() {
        let (width, height) = (6u32, 4u32);
        let view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        let mut parent = vec![0u32; width as usize * height as usize];
        for y in 0..height as usize {
            for x in 0..width as usize {
                parent[y * width as usize + x] = u32::from_le_bytes([
                    (20 + x * 23) as u8,
                    (35 + y * 31) as u8,
                    (170 - x * 11) as u8,
                    255,
                ]);
            }
        }
        let root = PropertyTreeState::default();
        let clip = Arc::new(ClipPaintPropertyNode {
            lifecycle: Default::default(),
            id: 9001,
            parent: Some(root.clip.clone()),
            local_transform_space: root.transform.clone(),
            rect: Some(PaintRect {
                x: 1.0,
                y: 1.0,
                width: 4.0,
                height: 2.0,
            }),
            radii: PaintCornerRadii::default(),
            clip_path: Vec::new(),
            clip_path_children: Vec::new(),
            clip_path_is_empty: false,
            clip_path_even_odd: false,
            clip_path_error: None,
            pixel_moving_filter: None,
        });
        let effect = EffectPaintPropertyNode {
            lifecycle: Default::default(),
            id: 9002,
            parent: Some(root.effect.clone()),
            local_transform_space: root.transform,
            output_clip: Some(root.clip),
            filter_reference_box: None,
            filter_reference_scale: 1.0,
            opacity: 1.0,
            blend_mode: PaintBlendMode::kNormal,
            filters: Vec::new(),
            backdrop_filters: Vec::new(),
            backdrop_filter_has_reference: false,
            backdrop_filter_clip: Some(clip.clone()),
            isolates_blending: false,
            has_mask: false,
            is_mask: false,
            direct_compositing_reasons: Vec::new(),
        };
        let output = backdrop_output_bounds(&effect, width, height, 1.0, None)
            .unwrap()
            .unwrap();
        assert_eq!(
            output,
            SurfaceBounds {
                left: 1,
                top: 1,
                right: 5,
                bottom: 3,
            }
        );
        let mut group = vec![0u32; parent.len()];
        let viewport = SurfaceBounds {
            left: 0,
            top: 0,
            right: width as usize,
            bottom: height as usize,
        };
        copy_backdrop(
            &mut group,
            view,
            viewport,
            &parent,
            view,
            viewport,
            skia::PixelFormat::Rgba8888,
        );
        let gray = EffectFilter::ColorMatrix(skia::cpu::layer_filters::css_color_matrix(
            ColorMatrixKind::Grayscale,
            1.0,
        ));
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(group.as_mut_ptr().cast::<u8>(), group.len() * 4)
        };
        assert!(skia::cpu::layer_filters::apply_rgba(
            bytes, width, height, &gray
        ));
        let mut cache = ClipMaskCache::default();
        let coverage = cache
            .coverage(&clip_chain(&clip), width, height, 1.0, None)
            .unwrap();
        let touched = apply_backdrop_coverage(&mut group, view, viewport, output, &coverage);
        assert_eq!(touched, Some(output));
        for y in 0..height as usize {
            for x in 0..width as usize {
                let pixel = group[y * width as usize + x].to_le_bytes();
                if (1..5).contains(&x) && (1..3).contains(&y) {
                    assert_eq!(pixel[0], pixel[1]);
                    assert_eq!(pixel[1], pixel[2]);
                    assert_eq!(pixel[3], 255);
                } else {
                    assert_eq!(pixel, [0, 0, 0, 0]);
                }
            }
        }
        // Group content is drawn after backdrop filtering; it is not fed back
        // through the backdrop chain as an ordinary foreground filter.
        let center = 2 * width as usize + 3;
        group[center] = source_over_rgba_word(
            group[center],
            u32::from_le_bytes([200, 0, 0, 255]),
            skia::PixelFormat::Rgba8888,
        );
        assert_eq!(group[center].to_le_bytes(), [200, 0, 0, 255]);
    }

    #[test]
    fn backdrop_blur_mirrors_the_exact_output_backdrop() {
        let (width, height) = (9u32, 9u32);
        let view = TargetView {
            origin: (0, 0),
            stride: width as usize,
        };
        let viewport = SurfaceBounds {
            left: 0,
            top: 0,
            right: width as usize,
            bottom: height as usize,
        };
        let output = SurfaceBounds {
            left: 3,
            top: 3,
            right: 6,
            bottom: 6,
        };
        let outside = u32::from_le_bytes([180, 70, 30, 255]);
        let inside = u32::from_le_bytes([0, 0, 0, 255]);
        let mut parent = vec![outside; width as usize * height as usize];
        for y in output.top..output.bottom {
            parent[view.row_range(y, output.left, output.right)].fill(inside);
        }
        let mut group = vec![0; parent.len()];
        copy_backdrop_mirrored(
            &mut group,
            view,
            viewport,
            output,
            &parent,
            view,
            viewport,
            skia::PixelFormat::Rgba8888,
        );
        let bytes = unsafe {
            std::slice::from_raw_parts_mut(group.as_mut_ptr().cast::<u8>(), group.len() * 4)
        };
        assert!(skia::cpu::layer_filters::blur_rgba(
            bytes, width, height, 2.0, 2.0
        ));
        assert!(group.iter().all(|&pixel| pixel == inside));
    }
}

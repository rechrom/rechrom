//! Backend-independent compositor scheduling and frame construction.
//!
//! `CompositorEngine` owns active/pending compositor state, coordinates
//! `layer_tile` and `raster`, and uses `FrameBuilder` to publish immutable
//! `CompositorFrame`s. A frame contains resource identities, shared quad state
//! and draw quads; this crate has no native surface, Viz aggregation, renderer,
//! window, or knowledge of which thread consumes its effects.

mod engine;
mod scroll_tree;
pub use engine::*;
pub use scroll_tree::ComputeMainThreadScrollUpdates;

use layer_tile::{
    compositor_context_transform_3d_with_scroll, compositor_transform_3d_with_scroll,
    resolved_compositor_transform_with_scroll, FramePlan, LayerId, LayerPlan, TileId,
    TilePlacement,
};
use layoutng_assembly::internal::paint_input::PaintBlendMode;
pub use paint::paint_engine::PaintRect;
pub use paint::paint_property_tree::EffectPaintPropertyNode;
use paint::paint_property_tree::{
    ClipPaintPropertyNode, PaintBackfaceVisibility, PropertyTreeState,
};
use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    sync::Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RenderPassId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl DeviceRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// A finite 2D affine mapping from raster-device coordinates to the final
/// render target.  Raster tiles already include `raster_scale`; keeping that
/// scale out of this value makes the identity transform the common fast path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineTransform {
    pub m11: f64,
    pub m12: f64,
    pub m21: f64,
    pub m22: f64,
    pub tx: f64,
    pub ty: f64,
}

impl AffineTransform {
    pub const IDENTITY: Self = Self {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub fn is_translation(self) -> bool {
        self.m11 == 1.0 && self.m12 == 0.0 && self.m21 == 0.0 && self.m22 == 1.0
    }

    pub fn map_point(self, x: f64, y: f64) -> (f64, f64) {
        (
            x * self.m11 + y * self.m21 + self.tx,
            x * self.m12 + y * self.m22 + self.ty,
        )
    }

    pub fn inverse(self) -> Option<Self> {
        let determinant = self.m11 * self.m22 - self.m12 * self.m21;
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        let inverse = Self {
            m11: self.m22 / determinant,
            m12: -self.m12 / determinant,
            m21: -self.m21 / determinant,
            m22: self.m11 / determinant,
            tx: (self.m21 * self.ty - self.m22 * self.tx) / determinant,
            ty: (self.m12 * self.tx - self.m11 * self.ty) / determinant,
        };
        [
            inverse.m11,
            inverse.m12,
            inverse.m21,
            inverse.m22,
            inverse.tx,
            inverse.ty,
        ]
        .into_iter()
        .all(f64::is_finite)
        .then_some(inverse)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectiveTransform {
    /// Homography from raster pixels on the local z=0 plane to device space.
    pub h: [f64; 9],
    /// Device-space homogeneous z numerator for depth/backface diagnostics.
    pub z: [f64; 3],
    pub backface_hidden: bool,
    /// gfx::Transform::IsBackFaceVisible for the complete accumulated 4x4.
    pub backface_visible: bool,
}

impl ProjectiveTransform {
    pub fn determinant(self) -> f64 {
        let [a, b, c, d, e, f, g, h, i] = self.h;
        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    pub fn map_point(self, x: f64, y: f64) -> Option<(f64, f64, f64)> {
        let w = self.h[6] * x + self.h[7] * y + self.h[8];
        if !w.is_finite() || w <= 0.0 {
            return None;
        }
        let px = (self.h[0] * x + self.h[1] * y + self.h[2]) / w;
        let py = (self.h[3] * x + self.h[4] * y + self.h[5]) / w;
        let depth = (self.z[0] * x + self.z[1] * y + self.z[2]) / w;
        [px, py, depth]
            .into_iter()
            .all(f64::is_finite)
            .then_some((px, py, depth))
    }

    pub fn inverse_point(self, x: f64, y: f64) -> Option<(f64, f64)> {
        let [a, b, c, d, e, f, g, h, i] = self.h;
        let determinant = self.determinant();
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        let inv = [
            (e * i - f * h) / determinant,
            (c * h - b * i) / determinant,
            (b * f - c * e) / determinant,
            (f * g - d * i) / determinant,
            (a * i - c * g) / determinant,
            (c * d - a * f) / determinant,
            (d * h - e * g) / determinant,
            (b * g - a * h) / determinant,
            (a * e - b * d) / determinant,
        ];
        let w = inv[6] * x + inv[7] * y + inv[8];
        if !w.is_finite() || w.abs() <= 1.0e-12 {
            return None;
        }
        let sx = (inv[0] * x + inv[1] * y + inv[2]) / w;
        let sy = (inv[3] * x + inv[4] * y + inv[5]) / w;
        [sx, sy].into_iter().all(f64::is_finite).then_some((sx, sy))
    }

    pub fn source_w(self, x: f64, y: f64) -> f64 {
        self.h[6] * x + self.h[7] * y + self.h[8]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerComposition {
    pub transform: AffineTransform,
    pub projective: Option<ProjectiveTransform>,
    /// Nonzero values identify one Chromium 3D sorting context. Only
    /// consecutive layers with the same id participate in one BSP run.
    pub rendering_context_id: u64,
    pub clip: Option<DeviceRect>,
    /// Output-quad limit from complete retained drawing support in target
    /// device coordinates. This never changes tile texture coordinates.
    pub content_bounds: Option<DeviceRect>,
    /// Destination cells whose complete sampling footprint is opaque.
    pub opaque_bounds: Option<DeviceRect>,
    pub white_backing: bool,
    pub grid_min: Option<(i32, i32)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TargetPolygon {
    pub points: Arc<[[f64; 2]]>,
    /// One flag for the edge from `points[i]` to the following point. BSP cut
    /// edges are hard ownership boundaries; original projected quad edges are
    /// antialiased by the existing projective sampler.
    pub antialias_edges: Arc<[bool]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompositorFrameMetadata {
    /// Successful-build sequence in this FrameBuilder/SurfaceId lifetime.
    /// Distinct from the raster source frame: impl-only updates advance it too.
    /// A replacement producer must use a new surface or invalidate its renderer.
    pub frame_token: u64,
    pub source_frame_id: u64,
    pub device_scale_factor: f64,
    /// Retained artifact units to output device pixels. This differs from
    /// device_scale_factor when Blink paints in LayoutZoomFactor coordinates.
    pub raster_scale_factor: f64,
    pub logical_viewport: PaintRect,
    pub viewport: DeviceRect,
    pub root_damage_rect: DeviceRect,
    pub raster_task_count: usize,
    pub begin_frame_ack: Option<BeginFrameAck>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeginFrameAck {
    pub source_id: u64,
    pub sequence_number: u64,
    pub has_damage: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferableResource {
    pub id: ResourceId,
    pub tile_id: TileId,
    pub generation: u64,
    pub size: (u32, u32),
}

/// State shared by adjacent tile quads from one retained layer. Keeping the
/// original property nodes preserves their stable identity across frames and
/// lets a backend lower clips/effects without consulting PaintArtifact.
#[derive(Clone)]
pub struct SharedQuadState {
    pub layer_id: LayerId,
    pub properties: PropertyTreeState,
    pub composition: LayerComposition,
    /// Target-space convex coverage produced by preserve-3d BSP splitting.
    /// The tile/resource identity and projective sampling transform remain
    /// unchanged; the renderer only discards samples outside this polygon.
    pub visible_polygon: Option<TargetPolygon>,
    pub retained_clips: Arc<[Arc<ClipPaintPropertyNode>]>,
    pub is_first_layer: bool,
    pub requires_transparent_backing: bool,
    pub raster_content_bounds: Option<PaintRect>,
    pub rect_known_to_be_opaque: PaintRect,
    pub raster_origin: (f64, f64),
    pub root_translation: (f64, f64),
    pub root_clip: Option<PaintRect>,
    pub compositor_scroll: Option<layer_tile::CompositorScrollOffset>,
}

impl fmt::Debug for SharedQuadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedQuadState")
            .field("layer_id", &self.layer_id)
            .field("composition", &self.composition)
            .field(
                "visible_polygon",
                &self
                    .visible_polygon
                    .as_ref()
                    .map(|polygon| polygon.points.len()),
            )
            .field("retained_clips", &self.retained_clips.len())
            .field("compositor_scroll", &self.compositor_scroll)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct TileDrawQuad {
    pub shared_quad_state_index: usize,
    pub resource_id: ResourceId,
    pub rect: PaintRect,
    pub visible_rect: PaintRect,
    pub texture_rect: PaintRect,
    pub nearest_neighbor: bool,
    pub tile: TilePlacement,
}

/// A child effect render surface drawn into its parent pass.  The child pass
/// stores raw descendant content; the owning effect is evaluated while this
/// quad is drawn so BSP fragments sample one already-rendered surface.
#[derive(Clone, Debug)]
pub struct RenderPassDrawQuad {
    pub shared_quad_state_index: usize,
    pub render_pass_id: RenderPassId,
}

/// Backend-neutral colored geometry composed without a raster resource.
#[derive(Clone, Debug)]
pub struct SolidColorDrawQuad {
    pub rect: PaintRect,
    pub visible_rect: PaintRect,
    /// Premultiplied RGBA.
    pub color: [u8; 4],
    pub corner_radius: f64,
}

#[derive(Clone, Debug)]
pub enum DrawQuad {
    Tile(TileDrawQuad),
    RenderPass(RenderPassDrawQuad),
    SolidColor(SolidColorDrawQuad),
}

#[derive(Clone, Debug)]
pub struct CompositorRenderPass {
    pub id: RenderPassId,
    /// Effect whose descendants are rendered by this pass. `None` identifies
    /// the root pass. The owner is applied by the parent RenderPassDrawQuad.
    pub owner_effect: Option<Arc<EffectPaintPropertyNode>>,
    pub output_rect: DeviceRect,
    pub damage_rect: DeviceRect,
    pub shared_quad_state_list: Vec<SharedQuadState>,
    pub quad_list: Vec<DrawQuad>,
}

/// Complete, immutable output of the compositor. Raster resources are named
/// by opaque ids; their pixels remain owned by the raster service.
#[derive(Clone, Debug)]
pub struct CompositorFrame {
    pub metadata: CompositorFrameMetadata,
    pub resource_list: Vec<TransferableResource>,
    /// Draw order. The final pass is the root pass.
    pub render_pass_list: Vec<CompositorRenderPass>,
}

#[derive(Clone, PartialEq)]
struct TileSnapshot {
    id: TileId,
    generation: u64,
    tile_index: (i32, i32),
    rect: PaintRect,
    tiling_rect: PaintRect,
    pixel_size: (u32, u32),
    raster_scale: f64,
}

#[derive(Clone)]
struct LayerSnapshot {
    id: LayerId,
    properties: PropertyTreeState,
    composition: LayerComposition,
    tiles: Vec<TileSnapshot>,
    output_bounds: Option<DeviceRect>,
    has_damage_sensitive_effect: bool,
    raster_content_bounds: Option<PaintRect>,
    compositor_scroll: Option<layer_tile::CompositorScrollOffset>,
    is_first_layer: bool,
    requires_transparent_backing: bool,
    raster_origin: (f64, f64),
    root_translation: (f64, f64),
    root_clip: Option<PaintRect>,
    rect_known_to_be_opaque: PaintRect,
}

impl LayerSnapshot {
    fn same_inputs(&self, other: &Self) -> bool {
        self.id == other.id
            && self.composition == other.composition
            && self.tiles == other.tiles
            && self.properties.same_values(&other.properties)
            && self.raster_content_bounds == other.raster_content_bounds
            && self.compositor_scroll == other.compositor_scroll
            && self.is_first_layer == other.is_first_layer
            && self.requires_transparent_backing == other.requires_transparent_backing
            && self.raster_origin == other.raster_origin
            && self.root_translation == other.root_translation
            && self.root_clip == other.root_clip
            && self.rect_known_to_be_opaque == other.rect_known_to_be_opaque
    }
}

// Keep ordered nested dependencies, not just a set of leaf layers: a mask,
// surface owner, or sibling reordering changes contributing content.
#[derive(Clone)]
enum CompositionSnapshot {
    Layer(LayerId, Option<TargetPolygon>),
    Surface {
        properties: PropertyTreeState,
        composition: LayerComposition,
        polygon: Option<TargetPolygon>,
        scroll: Option<layer_tile::CompositorScrollOffset>,
        children: Vec<CompositionSnapshot>,
    },
}
impl PartialEq for CompositionSnapshot {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Layer(a, b), Self::Layer(x, y)) => a == x && b == y,
            (
                Self::Surface {
                    properties: a,
                    composition: b,
                    polygon: c,
                    scroll: d,
                    children: e,
                },
                Self::Surface {
                    properties: x,
                    composition: y,
                    polygon: z,
                    scroll: w,
                    children: v,
                },
            ) => a.same_values(x) && b == y && c == z && d == w && e == v,
            _ => false,
        }
    }
}

#[derive(Clone)]
struct FrameSnapshot {
    layer_tree_id: layer_tile::LayerTreeId,
    viewport: DeviceRect,
    layers: Vec<LayerSnapshot>,
    draw_order: Vec<CompositionSnapshot>,
    logical_viewport: PaintRect,
    raster_scale: f64,
    device_scale: f64,
    owner: Option<PropertyTreeState>,
}

/// Retained, backend-independent compositor state. The owner decides which
/// thread calls it; the compositor only turns ready retained tiles into an
/// immutable frame and computes render-pass damage across submissions.
///
/// One builder is one producer lifetime for a Viz SurfaceId. Invalidate drops
/// damage history but keeps the successful-build sequence. Replacing the builder
/// requires a new surface lifetime (or Renderer::InvalidateSurface at the host
/// boundary); arbitrarily hot-swapping producers on an old surface is unsupported.
#[derive(Default)]
pub struct FrameBuilder {
    previous: Option<FrameSnapshot>,
    submission_sequence: u64,
    previous_passes: HashMap<RenderPassId, FrameSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameBuildError(pub &'static str);

impl fmt::Display for FrameBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for FrameBuildError {}

fn finite_rect(rect: PaintRect) -> bool {
    [rect.x, rect.y, rect.width, rect.height]
        .into_iter()
        .all(f64::is_finite)
        && rect.width >= 0.0
        && rect.height >= 0.0
}

fn device_rect(rect: PaintRect, scale: f64) -> Result<DeviceRect, FrameBuildError> {
    if !finite_rect(rect) || !scale.is_finite() || scale <= 0.0 {
        return Err(FrameBuildError("invalid-composition-rect"));
    }
    let left = (rect.x * scale).floor();
    let top = (rect.y * scale).floor();
    let right = ((rect.x + rect.width) * scale).ceil();
    let bottom = ((rect.y + rect.height) * scale).ceil();
    if ![left, top, right, bottom]
        .into_iter()
        .all(|value| value.is_finite() && value.abs() <= i32::MAX as f64)
    {
        return Err(FrameBuildError("composition-rect-out-of-range"));
    }
    Ok(DeviceRect::new(
        left as i32,
        top as i32,
        (right - left).max(0.0) as u32,
        (bottom - top).max(0.0) as u32,
    ))
}

fn transformed_device_rect(
    rect: PaintRect,
    scale: f64,
    transform: AffineTransform,
    projective: Option<ProjectiveTransform>,
) -> Result<DeviceRect, FrameBuildError> {
    if rect.is_empty() {
        return Ok(DeviceRect::new(0, 0, 0, 0));
    }
    let left = rect.x * scale;
    let top = rect.y * scale;
    let right = (rect.x + rect.width) * scale;
    let bottom = (rect.y + rect.height) * scale;
    let mut points = vec![(left, top), (right, top), (right, bottom), (left, bottom)];
    let mapped: Vec<(f64, f64)> = if let Some(projective) = projective {
        const NEAR: f64 = 1.0e-6;
        let mut clipped = Vec::new();
        for index in 0..points.len() {
            let a = points[index];
            let b = points[(index + 1) % points.len()];
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
        if clipped.is_empty() {
            return Ok(DeviceRect::new(0, 0, 0, 0));
        }
        clipped
            .into_iter()
            .filter_map(|p| projective.map_point(p.0, p.1).map(|q| (q.0, q.1)))
            .collect()
    } else {
        points
            .drain(..)
            .map(|p| transform.map_point(p.0, p.1))
            .collect()
    };
    if mapped.is_empty() {
        return Ok(DeviceRect::new(0, 0, 0, 0));
    }
    let min_x = mapped.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let min_y = mapped.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_x = mapped.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let max_y = mapped.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    device_rect(
        PaintRect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        },
        1.0,
    )
}

fn sampled_opaque_device_rect(
    rect: PaintRect,
    scale: f64,
    translation: (f64, f64),
) -> Result<Option<DeviceRect>, FrameBuildError> {
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
        return Err(FrameBuildError("invalid-opaque-device-bounds"));
    }
    Ok((right > x && bottom > y).then_some(DeviceRect::new(
        x as i32,
        y as i32,
        (right - x) as u32,
        (bottom - y) as u32,
    )))
}

fn retained_clips(layer: &LayerPlan) -> Arc<[Arc<ClipPaintPropertyNode>]> {
    let mut clips = Vec::new();
    let mut node = Some(layer.properties.clip.clone());
    while let Some(clip) = node {
        node = clip.parent.clone();
        clips.push(clip);
    }
    Arc::from(clips)
}

fn intersect_rect(a: DeviceRect, b: DeviceRect) -> Option<DeviceRect> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (i64::from(a.x) + i64::from(a.width)).min(i64::from(b.x) + i64::from(b.width));
    let bottom = (i64::from(a.y) + i64::from(a.height)).min(i64::from(b.y) + i64::from(b.height));
    (right > i64::from(left) && bottom > i64::from(top)).then_some(DeviceRect::new(
        left,
        top,
        (right - i64::from(left)) as u32,
        (bottom - i64::from(top)) as u32,
    ))
}

fn union_rect(target: &mut Option<DeviceRect>, rect: Option<DeviceRect>) {
    let Some(rect) = rect else { return };
    *target = Some(match *target {
        None => rect,
        Some(old) => {
            let left = old.x.min(rect.x);
            let top = old.y.min(rect.y);
            let right = (i64::from(old.x) + i64::from(old.width))
                .max(i64::from(rect.x) + i64::from(rect.width));
            let bottom = (i64::from(old.y) + i64::from(old.height))
                .max(i64::from(rect.y) + i64::from(rect.height));
            DeviceRect::new(
                left,
                top,
                (right - i64::from(left)) as u32,
                (bottom - i64::from(top)) as u32,
            )
        }
    });
}

fn damage_sensitive_effect(properties: &PropertyTreeState) -> bool {
    let mut effect = Some(&properties.effect);
    while let Some(node) = effect {
        if node.is_mask
            || node.has_mask
            || !node.filters.is_empty()
            || !node.backdrop_filters.is_empty()
            || node.backdrop_filter_has_reference
            || node.blend_mode != layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal
        {
            return true;
        }
        effect = node.parent.as_ref();
    }
    false
}

fn layer_output_bounds(
    layer: &LayerPlan,
    composition: LayerComposition,
    viewport: DeviceRect,
    scale: f64,
) -> Result<Option<DeviceRect>, FrameBuildError> {
    let mut bounds = None;
    for tile in &layer.tiles {
        let mut rect = transformed_device_rect(
            tile.tile_rect,
            scale,
            composition.transform,
            composition.projective,
        )?;
        if let Some(clip) = composition.clip {
            let Some(clipped) = intersect_rect(rect, clip) else {
                continue;
            };
            rect = clipped;
        }
        if let Some(content) = composition.content_bounds {
            let Some(clipped) = intersect_rect(rect, content) else {
                continue;
            };
            rect = clipped;
        }
        union_rect(&mut bounds, intersect_rect(rect, viewport));
    }
    Ok(bounds)
}

#[derive(Clone)]
struct SortingPolygon {
    order: usize,
    layer_index: usize,
    points: Vec<[f64; 3]>,
    antialias_edges: Vec<bool>,
    normal: [f64; 3],
}

#[derive(Clone)]
struct OrderedLayerComposition {
    layer_index: usize,
    composition: LayerComposition,
    visible_polygon: Option<TargetPolygon>,
}

#[derive(Clone)]
struct OrderedEffectSurface {
    owner: Arc<EffectPaintPropertyNode>,
    properties: PropertyTreeState,
    composition: LayerComposition,
    visible_polygon: Option<TargetPolygon>,
    representative_layer_index: usize,
    output_bounds: Option<DeviceRect>,
    units: Vec<OrderedCompositionUnit>,
}

#[derive(Clone)]
enum OrderedCompositionUnit {
    Layer(OrderedLayerComposition),
    Surface(OrderedEffectSurface),
}

fn effect_needs_render_surface(effect: &EffectPaintPropertyNode) -> bool {
    !effect.is_mask
        && (effect.opacity != 1.0
            || effect.blend_mode != PaintBlendMode::kNormal
            || effect.isolates_blending
            || effect.has_mask
            || !effect.filters.is_empty()
            || !effect.backdrop_filters.is_empty()
            || effect.backdrop_filter_has_reference)
}

fn effect_chain(layer: &LayerPlan) -> Vec<Arc<EffectPaintPropertyNode>> {
    let mut chain = Vec::new();
    let mut node = Some(layer.properties.effect.clone());
    while let Some(effect) = node {
        node = effect.parent.clone();
        chain.push(effect);
    }
    chain.reverse();
    chain
}

fn intersect_paint_rect(a: PaintRect, b: PaintRect) -> Option<PaintRect> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > left && bottom > top).then_some(PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

fn sorting_source_rect(layer: &LayerPlan) -> Option<PaintRect> {
    let mut support: Option<PaintRect> = None;
    for tile in &layer.tiles {
        let rect = tile.tile_rect;
        support = Some(match support {
            None => rect,
            Some(old) => {
                let left = old.x.min(rect.x);
                let top = old.y.min(rect.y);
                let right = (old.x + old.width).max(rect.x + rect.width);
                let bottom = (old.y + old.height).max(rect.y + rect.height);
                PaintRect {
                    x: left,
                    y: top,
                    width: right - left,
                    height: bottom - top,
                }
            }
        });
    }
    match (support, layer.raster_content_bounds) {
        (Some(support), Some(content)) => intersect_paint_rect(support, content),
        (support, None) => support,
        (None, _) => None,
    }
}

fn sorting_polygon(
    layer: &LayerPlan,
    composition: LayerComposition,
    scale: f64,
    order: usize,
    layer_index: usize,
) -> Result<Option<SortingPolygon>, FrameBuildError> {
    let Some(projective) = composition.projective else {
        return Err(FrameBuildError("preserve-3d-layer-without-plane"));
    };
    if projective.backface_hidden && projective.backface_visible {
        return Ok(None);
    }
    let Some(rect) = sorting_source_rect(layer) else {
        return Ok(None);
    };
    let left = rect.x * scale;
    let top = rect.y * scale;
    let right = (rect.x + rect.width) * scale;
    let bottom = (rect.y + rect.height) * scale;
    let source = vec![(left, top), (right, top), (right, bottom), (left, bottom)];
    const NEAR: f64 = 1.0e-6;
    let mut clipped = Vec::new();
    for index in 0..source.len() {
        let a = source[index];
        let b = source[(index + 1) % source.len()];
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
    let points: Vec<[f64; 3]> = clipped
        .into_iter()
        .filter_map(|point| {
            projective
                .map_point(point.0, point.1)
                .map(|point| [point.0, point.1, point.2])
        })
        .collect();
    if points.len() < 3 {
        return Ok(None);
    }
    // Newell's method is stable for the near-clipped convex polygon and
    // matches Viz's construction of one target-space plane per DrawPolygon.
    let mut normal = [0.0; 3];
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if !length.is_finite() || length <= 1.0e-12 {
        return Ok(None);
    }
    normal.iter_mut().for_each(|value| *value /= length);
    Ok(Some(SortingPolygon {
        order,
        layer_index,
        antialias_edges: vec![true; points.len()],
        points,
        normal,
    }))
}

fn surface_sorting_polygon(
    effect: &Arc<EffectPaintPropertyNode>,
    representative: &LayerPlan,
    bounds: DeviceRect,
    scale: f64,
    order: usize,
    unit_index: usize,
) -> Result<Option<SortingPolygon>, FrameBuildError> {
    if bounds.width == 0 || bounds.height == 0 {
        return Ok(None);
    }
    let matrix = compositor_transform_3d_with_scroll(
        &effect.local_transform_space,
        scale,
        representative.compositor_scroll,
    )
    .map_err(|_| FrameBuildError("unsupported-effect-surface-transform"))?;
    let mut backface = PaintBackfaceVisibility::Inherited;
    let mut node = Some(&effect.local_transform_space);
    while let Some(transform) = node {
        if backface == PaintBackfaceVisibility::Inherited
            && transform.backface_visibility != PaintBackfaceVisibility::Inherited
        {
            backface = transform.backface_visibility;
        }
        node = transform.parent.as_ref();
    }
    if backface == PaintBackfaceVisibility::Hidden && backface_visible(&matrix) {
        return Ok(None);
    }
    let map = |x: f64, y: f64| -> Option<[f64; 3]> {
        let w = matrix.values[3] * x + matrix.values[7] * y + matrix.values[15];
        (w.is_finite() && w.abs() >= 1.0e-6).then(|| {
            [
                (matrix.values[0] * x + matrix.values[4] * y + matrix.values[12]) / w,
                (matrix.values[1] * x + matrix.values[5] * y + matrix.values[13]) / w,
                (matrix.values[2] * x + matrix.values[6] * y + matrix.values[14]) / w,
            ]
        })
    };
    let origin = map(0.0, 0.0).ok_or(FrameBuildError("effect-surface-near-plane"))?;
    let px = map(1.0, 0.0).ok_or(FrameBuildError("effect-surface-near-plane"))?;
    let py = map(0.0, 1.0).ok_or(FrameBuildError("effect-surface-near-plane"))?;
    let ux = [px[0] - origin[0], px[1] - origin[1], px[2] - origin[2]];
    let uy = [py[0] - origin[0], py[1] - origin[1], py[2] - origin[2]];
    let mut normal = [
        ux[1] * uy[2] - ux[2] * uy[1],
        ux[2] * uy[0] - ux[0] * uy[2],
        ux[0] * uy[1] - ux[1] * uy[0],
    ];
    let length = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if !length.is_finite() || length <= 1.0e-12 || normal[2].abs() <= 1.0e-12 {
        return Ok(None);
    }
    normal.iter_mut().for_each(|value| *value /= length);
    let z_at = |x: f64, y: f64| {
        origin[2] - (normal[0] * (x - origin[0]) + normal[1] * (y - origin[1])) / normal[2]
    };
    let left = bounds.x as f64;
    let top = bounds.y as f64;
    let right = left + bounds.width as f64;
    let bottom = top + bounds.height as f64;
    Ok(Some(SortingPolygon {
        order,
        layer_index: unit_index,
        points: vec![
            [left, top, z_at(left, top)],
            [right, top, z_at(right, top)],
            [right, bottom, z_at(right, bottom)],
            [left, bottom, z_at(left, bottom)],
        ],
        antialias_edges: vec![true; 4],
        normal,
    }))
}

enum ClassifiedPolygon {
    Front(SortingPolygon),
    Back(SortingPolygon),
    Coplanar(SortingPolygon),
    Split {
        front: SortingPolygon,
        back: SortingPolygon,
    },
}

fn plane_distance(splitter: &SortingPolygon, point: [f64; 3]) -> f64 {
    let origin = splitter.points[0];
    (point[0] - origin[0]) * splitter.normal[0]
        + (point[1] - origin[1]) * splitter.normal[1]
        + (point[2] - origin[2]) * splitter.normal[2]
}

fn push_distinct(points: &mut Vec<[f64; 3]>, point: [f64; 3]) {
    const EPSILON_SQUARED: f64 = 1.0e-20;
    if points.last().is_some_and(|last| {
        (last[0] - point[0]).powi(2) + (last[1] - point[1]).powi(2) + (last[2] - point[2]).powi(2)
            <= EPSILON_SQUARED
    }) {
        return;
    }
    points.push(point);
}

fn finish_split_points(points: &mut Vec<[f64; 3]>) {
    const EPSILON_SQUARED: f64 = 1.0e-20;
    if points.len() > 1 {
        let first = points[0];
        let last = points[points.len() - 1];
        if (first[0] - last[0]).powi(2)
            + (first[1] - last[1]).powi(2)
            + (first[2] - last[2]).powi(2)
            <= EPSILON_SQUARED
        {
            points.pop();
        }
    }
}

fn point_on_segment(point: [f64; 3], a: [f64; 3], b: [f64; 3]) -> bool {
    let edge = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let length_squared = edge[0] * edge[0] + edge[1] * edge[1] + edge[2] * edge[2];
    if length_squared <= 1.0e-20 {
        return false;
    }
    let offset = [point[0] - a[0], point[1] - a[1], point[2] - a[2]];
    let t = (offset[0] * edge[0] + offset[1] * edge[1] + offset[2] * edge[2]) / length_squared;
    if !(-1.0e-9..=1.0 + 1.0e-9).contains(&t) {
        return false;
    }
    let nearest = [a[0] + edge[0] * t, a[1] + edge[1] * t, a[2] + edge[2] * t];
    let error_squared = (nearest[0] - point[0]).powi(2)
        + (nearest[1] - point[1]).powi(2)
        + (nearest[2] - point[2]).powi(2);
    error_squared <= 1.0e-16 * length_squared.max(1.0)
}

fn split_edge_flags(
    splitter: &SortingPolygon,
    original: &SortingPolygon,
    points: &[[f64; 3]],
) -> Vec<bool> {
    (0..points.len())
        .map(|index| {
            let a = points[index];
            let b = points[(index + 1) % points.len()];
            if plane_distance(splitter, a).abs() <= 1.0e-8
                && plane_distance(splitter, b).abs() <= 1.0e-8
            {
                return false;
            }
            let midpoint = [
                (a[0] + b[0]) * 0.5,
                (a[1] + b[1]) * 0.5,
                (a[2] + b[2]) * 0.5,
            ];
            original
                .points
                .iter()
                .enumerate()
                .find(|(old, &start)| {
                    point_on_segment(
                        midpoint,
                        start,
                        original.points[(*old + 1) % original.points.len()],
                    )
                })
                .is_some_and(|(old, _)| original.antialias_edges[old])
        })
        .collect()
}

fn classify_polygon(splitter: &SortingPolygon, polygon: SortingPolygon) -> ClassifiedPolygon {
    // Chromium viz::DrawPolygon uses a normalized plane thickness of 0.05.
    const SPLIT_THRESHOLD: f64 = 0.05;
    let distances: Vec<_> = polygon
        .points
        .iter()
        .copied()
        .map(|point| plane_distance(splitter, point))
        .collect();
    let mut front = false;
    let mut back = false;
    for &distance in &distances {
        front |= distance > SPLIT_THRESHOLD;
        back |= distance < -SPLIT_THRESHOLD;
    }
    if !(front && back) {
        return if front {
            ClassifiedPolygon::Front(polygon)
        } else if back {
            ClassifiedPolygon::Back(polygon)
        } else {
            ClassifiedPolygon::Coplanar(polygon)
        };
    }

    // DrawPolygon splitting retains the original quad/material identity and
    // divides only target-space coverage. Interpolate on the exact plane;
    // SPLIT_THRESHOLD is solely the robust classification thickness.
    let mut front_points = Vec::with_capacity(polygon.points.len() + 1);
    let mut back_points = Vec::with_capacity(polygon.points.len() + 1);
    for index in 0..polygon.points.len() {
        let point = polygon.points[index];
        let next = polygon.points[(index + 1) % polygon.points.len()];
        let distance = distances[index];
        let next_distance = distances[(index + 1) % distances.len()];
        if distance >= 0.0 {
            push_distinct(&mut front_points, point);
        }
        if distance <= 0.0 {
            push_distinct(&mut back_points, point);
        }
        if (distance > 0.0 && next_distance < 0.0) || (distance < 0.0 && next_distance > 0.0) {
            let t = distance / (distance - next_distance);
            let intersection = [
                point[0] + (next[0] - point[0]) * t,
                point[1] + (next[1] - point[1]) * t,
                point[2] + (next[2] - point[2]) * t,
            ];
            push_distinct(&mut front_points, intersection);
            push_distinct(&mut back_points, intersection);
        }
    }
    finish_split_points(&mut front_points);
    finish_split_points(&mut back_points);
    debug_assert!(front_points.len() >= 3 && back_points.len() >= 3);
    let front_edges = split_edge_flags(splitter, &polygon, &front_points);
    let back_edges = split_edge_flags(splitter, &polygon, &back_points);
    ClassifiedPolygon::Split {
        front: SortingPolygon {
            points: front_points,
            antialias_edges: front_edges,
            ..polygon.clone()
        },
        back: SortingPolygon {
            points: back_points,
            antialias_edges: back_edges,
            ..polygon
        },
    }
}

fn bsp_order(mut polygons: Vec<SortingPolygon>, output: &mut Vec<SortingPolygon>) {
    if polygons.is_empty() {
        return;
    }
    let root = polygons.remove(0);
    let mut front = Vec::new();
    let mut back = Vec::new();
    let mut coplanar_before = Vec::new();
    let mut coplanar_after = Vec::new();
    for polygon in polygons {
        match classify_polygon(&root, polygon) {
            ClassifiedPolygon::Front(polygon) => front.push(polygon),
            ClassifiedPolygon::Back(polygon) => back.push(polygon),
            ClassifiedPolygon::Coplanar(polygon) if polygon.order < root.order => {
                coplanar_before.push(polygon)
            }
            ClassifiedPolygon::Coplanar(polygon) => coplanar_after.push(polygon),
            ClassifiedPolygon::Split {
                front: front_fragment,
                back: back_fragment,
            } => {
                front.push(front_fragment);
                back.push(back_fragment);
            }
        }
    }
    // Viz's BSP walk treats normal.z > 0 as the camera-facing side and emits
    // the opposite subtree first, preserving paint order for coplanar planes.
    let (first, second) = if root.normal[2] > 0.0 {
        (back, front)
    } else {
        (front, back)
    };
    bsp_order(first, output);
    coplanar_before.sort_by_key(|polygon| polygon.order);
    output.extend(coplanar_before);
    output.push(root);
    coplanar_after.sort_by_key(|polygon| polygon.order);
    output.extend(coplanar_after);
    bsp_order(second, output);
}

#[derive(Clone)]
struct PendingEffectSurface {
    owner: Arc<EffectPaintPropertyNode>,
    representative_layer_index: usize,
    units: Vec<PendingCompositionUnit>,
}

#[derive(Clone)]
enum PendingCompositionUnit {
    Layer(usize),
    Surface(PendingEffectSurface),
}

fn insert_pending_layer(
    units: &mut Vec<PendingCompositionUnit>,
    effects: &[Arc<EffectPaintPropertyNode>],
    layer_index: usize,
) {
    let Some((owner, descendants)) = effects.split_first() else {
        units.push(PendingCompositionUnit::Layer(layer_index));
        return;
    };
    if let Some(PendingCompositionUnit::Surface(surface)) = units.iter_mut().find(|unit| {
        matches!(unit, PendingCompositionUnit::Surface(surface)
            if surface.owner.lifecycle.same_node(&owner.lifecycle))
    }) {
        insert_pending_layer(&mut surface.units, descendants, layer_index);
        return;
    }
    let mut surface = PendingEffectSurface {
        owner: owner.clone(),
        representative_layer_index: layer_index,
        units: Vec::new(),
    };
    insert_pending_layer(&mut surface.units, descendants, layer_index);
    units.push(PendingCompositionUnit::Surface(surface));
}

fn unit_context(plan: &FramePlan, unit: &OrderedCompositionUnit) -> u64 {
    match unit {
        // A real mask is the final DstIn child of its content effect. It is a
        // lifecycle operation, not a depth-sortable sibling: keeping it as a
        // barrier preserves content/filter completion before mask application.
        OrderedCompositionUnit::Layer(layer)
            if plan.layers[layer.layer_index].properties.effect.is_mask =>
        {
            0
        }
        OrderedCompositionUnit::Layer(layer) => layer.composition.rendering_context_id,
        OrderedCompositionUnit::Surface(surface) => surface.composition.rendering_context_id,
    }
}

fn order_direct_units(
    plan: &FramePlan,
    units: Vec<OrderedCompositionUnit>,
) -> Result<Vec<OrderedCompositionUnit>, FrameBuildError> {
    let mut ordered = Vec::new();
    let mut start = 0;
    while start < units.len() {
        let context = unit_context(plan, &units[start]);
        if context == 0 {
            ordered.push(units[start].clone());
            start += 1;
            continue;
        }
        let mut end = start + 1;
        while end < units.len() && unit_context(plan, &units[end]) == context {
            end += 1;
        }
        let mut polygons = Vec::new();
        for unit_index in start..end {
            let polygon = match &units[unit_index] {
                OrderedCompositionUnit::Layer(layer) => sorting_polygon(
                    &plan.layers[layer.layer_index],
                    layer.composition,
                    plan.config.raster_scale,
                    unit_index,
                    unit_index,
                )?,
                OrderedCompositionUnit::Surface(surface) => surface
                    .output_bounds
                    .map(|bounds| {
                        surface_sorting_polygon(
                            &surface.owner,
                            &plan.layers[surface.representative_layer_index],
                            bounds,
                            plan.config.raster_scale,
                            unit_index,
                            unit_index,
                        )
                    })
                    .transpose()?
                    .flatten(),
            };
            if let Some(polygon) = polygon {
                polygons.push(polygon);
            }
        }
        let mut sorted = Vec::new();
        bsp_order(polygons, &mut sorted);
        for polygon in sorted {
            let mut unit = units[polygon.layer_index].clone();
            let visible_polygon = Some(TargetPolygon {
                points: polygon
                    .points
                    .iter()
                    .map(|point| [point[0], point[1]])
                    .collect::<Vec<_>>()
                    .into(),
                antialias_edges: polygon.antialias_edges.into(),
            });
            match &mut unit {
                OrderedCompositionUnit::Layer(layer) => layer.visible_polygon = visible_polygon,
                OrderedCompositionUnit::Surface(surface) => {
                    surface.visible_polygon = visible_polygon
                }
            }
            ordered.push(unit);
        }
        start = end;
    }
    Ok(ordered)
}

fn ordered_unit_bounds(
    plan: &FramePlan,
    unit: &OrderedCompositionUnit,
    viewport: DeviceRect,
) -> Result<Option<DeviceRect>, FrameBuildError> {
    match unit {
        OrderedCompositionUnit::Layer(layer) => layer_output_bounds(
            &plan.layers[layer.layer_index],
            layer.composition,
            viewport,
            plan.config.raster_scale,
        ),
        OrderedCompositionUnit::Surface(surface) => Ok(surface.output_bounds),
    }
}

fn lower_pending_units(
    plan: &FramePlan,
    compositions: &[LayerComposition],
    viewport: DeviceRect,
    pending: Vec<PendingCompositionUnit>,
) -> Result<Vec<OrderedCompositionUnit>, FrameBuildError> {
    let mut units = Vec::with_capacity(pending.len());
    for pending in pending {
        match pending {
            PendingCompositionUnit::Layer(layer_index) => {
                units.push(OrderedCompositionUnit::Layer(OrderedLayerComposition {
                    layer_index,
                    composition: compositions[layer_index],
                    visible_polygon: None,
                }));
            }
            PendingCompositionUnit::Surface(surface) => {
                let representative = surface.representative_layer_index;
                let child_units = lower_pending_units(plan, compositions, viewport, surface.units)?;
                let mut output_bounds = None;
                for unit in &child_units {
                    union_rect(
                        &mut output_bounds,
                        ordered_unit_bounds(plan, unit, viewport)?,
                    );
                }
                if !surface.owner.filters.is_empty()
                    || !surface.owner.backdrop_filters.is_empty()
                    || surface.owner.backdrop_filter_has_reference
                {
                    output_bounds = Some(viewport);
                }
                let mut properties = plan.layers[representative].properties.clone();
                properties.effect = surface.owner.clone();
                properties.transform = surface.owner.local_transform_space.clone();
                let context = compositor_context_transform_3d_with_scroll(
                    &surface.owner.local_transform_space,
                    plan.config.raster_scale,
                    plan.layers[representative].compositor_scroll,
                )
                .map_err(|_| FrameBuildError("unsupported-effect-surface-transform"))?
                .map_or(0, |context| context.0);
                units.push(OrderedCompositionUnit::Surface(OrderedEffectSurface {
                    owner: surface.owner,
                    properties,
                    composition: LayerComposition {
                        transform: AffineTransform::IDENTITY,
                        projective: None,
                        rendering_context_id: context,
                        clip: None,
                        content_bounds: output_bounds,
                        opaque_bounds: None,
                        white_backing: false,
                        grid_min: None,
                    },
                    visible_polygon: None,
                    representative_layer_index: representative,
                    output_bounds,
                    units: child_units,
                }));
            }
        }
    }
    order_direct_units(plan, units)
}

fn ordered_composition_units(
    plan: &FramePlan,
) -> Result<Vec<OrderedCompositionUnit>, FrameBuildError> {
    let compositions: Vec<_> = plan
        .layers
        .iter()
        .map(|layer| LayerComposition::from_layer(layer, plan.config.raster_scale))
        .collect::<Result<_, _>>()?;
    let masked_parents: Vec<_> = plan
        .layers
        .iter()
        .flat_map(effect_chain)
        .filter(|effect| effect.is_mask)
        .filter_map(|effect| effect.parent.clone())
        .collect();
    let mut pending = Vec::new();
    for (layer_index, layer) in plan.layers.iter().enumerate() {
        let surfaces: Vec<_> = effect_chain(layer)
            .into_iter()
            .filter(|effect| {
                effect_needs_render_surface(effect)
                    || masked_parents
                        .iter()
                        .any(|parent| parent.lifecycle.same_node(&effect.lifecycle))
            })
            .collect();
        insert_pending_layer(&mut pending, &surfaces, layer_index);
    }
    let viewport = device_rect(plan.config.viewport, plan.config.raster_scale)?;
    lower_pending_units(plan, &compositions, viewport, pending)
}

fn frame_snapshot(
    plan: &FramePlan,
    viewport: DeviceRect,
    ordered: &[OrderedCompositionUnit],
) -> Result<FrameSnapshot, FrameBuildError> {
    let mut layers = Vec::with_capacity(plan.layers.len());
    for layer in &plan.layers {
        let composition = LayerComposition::from_layer(layer, plan.config.raster_scale)?;
        layers.push(LayerSnapshot {
            id: layer.id,
            properties: layer.properties.clone(),
            composition,
            tiles: layer
                .tiles
                .iter()
                .map(|tile| TileSnapshot {
                    id: tile.tile_id,
                    generation: tile.generation,
                    tile_index: tile.tile_index,
                    rect: tile.tile_rect,
                    tiling_rect: tile.tiling_rect,
                    pixel_size: tile.pixel_size,
                    raster_scale: tile.raster_scale,
                })
                .collect(),
            output_bounds: layer_output_bounds(
                layer,
                composition,
                viewport,
                plan.config.raster_scale,
            )?,
            has_damage_sensitive_effect: damage_sensitive_effect(&layer.properties),
            raster_content_bounds: layer.raster_content_bounds,
            compositor_scroll: layer.compositor_scroll,
            is_first_layer: layer.is_first_layer,
            requires_transparent_backing: layer.requires_transparent_backing,
            raster_origin: layer.raster_origin,
            root_translation: layer.root_translation,
            root_clip: layer.root_clip,
            rect_known_to_be_opaque: layer.rect_known_to_be_opaque,
        });
    }
    Ok(FrameSnapshot {
        layer_tree_id: plan.layer_tree_id,
        viewport,
        layers,
        draw_order: composition_snapshot(plan, ordered),
        logical_viewport: plan.config.viewport,
        raster_scale: plan.config.raster_scale,
        device_scale: plan.config.device_scale_factor,
        owner: None,
    })
}

fn composition_snapshot(
    plan: &FramePlan,
    units: &[OrderedCompositionUnit],
) -> Vec<CompositionSnapshot> {
    units
        .iter()
        .map(|unit| match unit {
            OrderedCompositionUnit::Layer(layer) => CompositionSnapshot::Layer(
                plan.layers[layer.layer_index].id,
                layer.visible_polygon.clone(),
            ),
            OrderedCompositionUnit::Surface(surface) => CompositionSnapshot::Surface {
                properties: surface.properties.clone(),
                composition: surface.composition,
                polygon: surface.visible_polygon.clone(),
                scroll: plan.layers[surface.representative_layer_index].compositor_scroll,
                children: composition_snapshot(plan, &surface.units),
            },
        })
        .collect()
}

fn collect_contributing_layers(units: &[OrderedCompositionUnit], layers: &mut BTreeSet<usize>) {
    for unit in units {
        match unit {
            OrderedCompositionUnit::Layer(layer) => {
                layers.insert(layer.layer_index);
            }
            OrderedCompositionUnit::Surface(surface) => {
                collect_contributing_layers(&surface.units, layers)
            }
        }
    }
}
fn effect_pass_id(owner: &EffectPaintPropertyNode) -> RenderPassId {
    RenderPassId(Arc::as_ptr(&owner.lifecycle.identity) as usize as u64)
}
fn collect_pass_snapshots(
    plan: &FramePlan,
    root: &FrameSnapshot,
    units: &[OrderedCompositionUnit],
    snapshots: &mut HashMap<RenderPassId, FrameSnapshot>,
) {
    for unit in units {
        if let OrderedCompositionUnit::Surface(surface) = unit {
            let id = effect_pass_id(&surface.owner);
            if snapshots.contains_key(&id) {
                continue;
            }
            let mut indices = BTreeSet::new();
            collect_contributing_layers(&surface.units, &mut indices);
            snapshots.insert(
                id,
                FrameSnapshot {
                    layer_tree_id: root.layer_tree_id,
                    viewport: root.viewport,
                    logical_viewport: root.logical_viewport,
                    raster_scale: root.raster_scale,
                    device_scale: root.device_scale,
                    owner: Some(surface.properties.clone()),
                    layers: indices
                        .into_iter()
                        .map(|index| root.layers[index].clone())
                        .collect(),
                    draw_order: composition_snapshot(plan, &surface.units),
                },
            );
            collect_pass_snapshots(plan, root, &surface.units, snapshots);
        }
    }
}
impl FrameSnapshot {
    fn same_space(&self, other: &Self) -> bool {
        self.layer_tree_id == other.layer_tree_id
            && self.viewport == other.viewport
            && self.logical_viewport == other.logical_viewport
            && self.raster_scale == other.raster_scale
            && self.device_scale == other.device_scale
            && match (&self.owner, &other.owner) {
                (None, None) => true,
                (Some(a), Some(b)) => a.same_values(b),
                _ => false,
            }
    }
    fn same_inputs(&self, other: &Self) -> bool {
        self.same_space(other)
            && self.draw_order == other.draw_order
            && self.layers.len() == other.layers.len()
            && self
                .layers
                .iter()
                .zip(&other.layers)
                .all(|(a, b)| a.same_inputs(b))
    }
}

fn frame_damage(previous: Option<&FrameSnapshot>, current: &FrameSnapshot) -> DeviceRect {
    let full = current.viewport;
    let Some(previous) = previous else {
        return full;
    };
    if !previous.same_space(current) {
        return full;
    }
    if previous.draw_order != current.draw_order {
        return full;
    }
    let old_by_id: std::collections::HashMap<_, _> = previous
        .layers
        .iter()
        .map(|layer| (layer.id, layer))
        .collect();
    let new_by_id: std::collections::HashMap<_, _> = current
        .layers
        .iter()
        .map(|layer| (layer.id, layer))
        .collect();
    if old_by_id.len() != previous.layers.len() || new_by_id.len() != current.layers.len() {
        return full;
    }
    let old_common: Vec<_> = previous
        .layers
        .iter()
        .filter(|layer| new_by_id.contains_key(&layer.id))
        .map(|layer| layer.id)
        .collect();
    let new_common: Vec<_> = current
        .layers
        .iter()
        .filter(|layer| old_by_id.contains_key(&layer.id))
        .map(|layer| layer.id)
        .collect();
    if old_common != new_common {
        return full;
    }
    let mut damage = None;
    for old in previous
        .layers
        .iter()
        .filter(|layer| !new_by_id.contains_key(&layer.id))
    {
        if old.has_damage_sensitive_effect {
            return full;
        }
        union_rect(&mut damage, old.output_bounds);
    }
    for new in &current.layers {
        let Some(old) = old_by_id.get(&new.id).copied() else {
            if new.has_damage_sensitive_effect {
                return full;
            }
            union_rect(&mut damage, new.output_bounds);
            continue;
        };
        let changed = !old.same_inputs(new);
        if !changed {
            continue;
        }
        if old.has_damage_sensitive_effect || new.has_damage_sensitive_effect {
            return full;
        }
        union_rect(&mut damage, old.output_bounds);
        union_rect(&mut damage, new.output_bounds);
    }
    damage.unwrap_or(DeviceRect::new(
        current.viewport.x,
        current.viewport.y,
        0,
        0,
    ))
}

fn projective_matrix_singular(
    matrix: &layoutng_assembly::internal::layout_input::TransformMatrix,
    scale: f64,
) -> bool {
    let a = matrix.values[0] / scale;
    let b = matrix.values[4] / scale;
    let c = matrix.values[12];
    let d = matrix.values[1] / scale;
    let e = matrix.values[5] / scale;
    let f = matrix.values[13];
    let g = matrix.values[3] / scale;
    let h = matrix.values[7] / scale;
    let i = matrix.values[15];
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    !det.is_finite() || det == 0.0
}

fn determinant_4x4(matrix: &layoutng_assembly::internal::layout_input::TransformMatrix) -> f64 {
    let at = |row: usize, column: usize| matrix.values[column * 4 + row];
    let determinant_3x3 = |columns: [usize; 3]| {
        let (a, b, c) = (at(1, columns[0]), at(1, columns[1]), at(1, columns[2]));
        let (d, e, f) = (at(2, columns[0]), at(2, columns[1]), at(2, columns[2]));
        let (g, h, i) = (at(3, columns[0]), at(3, columns[1]), at(3, columns[2]));
        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    };
    at(0, 0) * determinant_3x3([1, 2, 3]) - at(0, 1) * determinant_3x3([0, 2, 3])
        + at(0, 2) * determinant_3x3([0, 1, 3])
        - at(0, 3) * determinant_3x3([0, 1, 2])
}

fn backface_visible(matrix: &layoutng_assembly::internal::layout_input::TransformMatrix) -> bool {
    let m = &matrix.values;
    let determinant = determinant_4x4(matrix);
    if determinant == 0.0 {
        return false;
    }
    // Chromium gfx::Transform::IsBackFaceVisible computes the (2,2)
    // cofactor, which is the transformed normal's z component before the
    // determinant division. This deliberately differs from projected 2D
    // winding: a negative scaleX/scaleY does not expose the back face.
    let cofactor_22 = m[0] * m[5] * m[15] + m[4] * m[13] * m[3] + m[12] * m[1] * m[7]
        - m[0] * m[13] * m[7]
        - m[4] * m[1] * m[15]
        - m[12] * m[5] * m[3];
    cofactor_22 * determinant < -(f32::EPSILON as f64)
}

impl LayerComposition {
    pub fn from_layer(layer: &LayerPlan, scale: f64) -> Result<Self, FrameBuildError> {
        let (matrix, clip) = resolved_compositor_transform_with_scroll(
            &layer.properties,
            scale,
            layer.compositor_scroll,
        )
        .map_err(|_| FrameBuildError("unsupported-compositor-properties"))?;
        let affine = matrix.values.iter().enumerate().all(|(index, value)| {
            value.is_finite()
                && (matches!(index, 0 | 1 | 4 | 5 | 12 | 13)
                    || *value
                        == layoutng_assembly::internal::layout_input::TransformMatrix::default()
                            .values[index])
        });
        let mut transform_node = Some(&layer.properties.transform);
        let mut backface = PaintBackfaceVisibility::Inherited;
        let mut rendering_context_id = 0;
        while let Some(node) = transform_node {
            if node.rendering_context_id != 0 {
                if rendering_context_id == 0 {
                    rendering_context_id = node.rendering_context_id;
                } else if rendering_context_id != node.rendering_context_id {
                    return Err(FrameBuildError("nested-preserve-3d-context"));
                }
            }
            if backface == PaintBackfaceVisibility::Inherited
                && node.backface_visibility != PaintBackfaceVisibility::Inherited
            {
                backface = node.backface_visibility;
            }
            transform_node = node.parent.as_ref();
        }
        let context_transform = compositor_context_transform_3d_with_scroll(
            &layer.properties.transform,
            scale,
            layer.compositor_scroll,
        )
        .map_err(|_| FrameBuildError("unsupported-compositor-properties"))?;
        if context_transform.is_some_and(|context| context.0 != rendering_context_id) {
            return Err(FrameBuildError("inconsistent-preserve-3d-context"));
        }
        let transform = AffineTransform {
            m11: matrix.values[0] / scale,
            m12: matrix.values[1] / scale,
            m21: matrix.values[4] / scale,
            m22: matrix.values[5] / scale,
            tx: matrix.values[12],
            ty: matrix.values[13],
        };
        if [
            transform.m11,
            transform.m12,
            transform.m21,
            transform.m22,
            transform.tx,
            transform.ty,
        ]
        .into_iter()
        .any(|value| !value.is_finite() || value.abs() > (1 << 22) as f64)
            || (affine && transform.inverse().is_none())
            || projective_matrix_singular(&matrix, scale)
        {
            return Err(FrameBuildError("large-property-placement"));
        }
        let white_backing = layer.is_first_layer && !layer.requires_transparent_backing;
        let backface_visible =
            backface_visible(&context_transform.map(|context| context.1).unwrap_or(matrix));
        let projective = (!affine || rendering_context_id != 0).then_some(ProjectiveTransform {
            h: [
                matrix.values[0] / scale,
                matrix.values[4] / scale,
                matrix.values[12],
                matrix.values[1] / scale,
                matrix.values[5] / scale,
                matrix.values[13],
                matrix.values[3] / scale,
                matrix.values[7] / scale,
                matrix.values[15],
            ],
            z: [
                matrix.values[2] / scale,
                matrix.values[6] / scale,
                matrix.values[14],
            ],
            backface_hidden: backface == PaintBackfaceVisibility::Hidden,
            backface_visible,
        });
        Ok(Self {
            transform,
            projective,
            rendering_context_id,
            clip: clip.map(|rect| device_rect(rect, 1.0)).transpose()?,
            content_bounds: if white_backing {
                None
            } else {
                layer
                    .raster_content_bounds
                    .map(|rect| transformed_device_rect(rect, scale, transform, projective))
                    .transpose()?
            },
            // A transformed opaque quad is generally a parallelogram; an
            // axis-aligned bounding box is not an opacity proof.
            opaque_bounds: if projective.is_none() && transform.is_translation() {
                sampled_opaque_device_rect(
                    layer.rect_known_to_be_opaque,
                    scale,
                    (transform.tx, transform.ty),
                )?
            } else {
                None
            },
            white_backing,
            grid_min: layer
                .tiles
                .iter()
                .map(|tile| tile.tile_index)
                .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1))),
        })
    }
}

fn append_layer_quads(
    plan: &FramePlan,
    fragment: &OrderedLayerComposition,
    states: &mut Vec<SharedQuadState>,
    quads: &mut Vec<DrawQuad>,
    resources: &mut Vec<TransferableResource>,
    resource_keys: &mut BTreeSet<(TileId, u64)>,
) -> Result<(), FrameBuildError> {
    let layer = &plan.layers[fragment.layer_index];
    let composition = fragment.composition;
    let shared_quad_state_index = states.len();
    states.push(SharedQuadState {
        layer_id: layer.id,
        properties: layer.properties.clone(),
        composition,
        visible_polygon: fragment.visible_polygon.clone(),
        retained_clips: retained_clips(layer),
        is_first_layer: layer.is_first_layer,
        requires_transparent_backing: layer.requires_transparent_backing,
        raster_content_bounds: layer.raster_content_bounds,
        rect_known_to_be_opaque: layer.rect_known_to_be_opaque,
        raster_origin: layer.raster_origin,
        root_translation: layer.root_translation,
        root_clip: layer.root_clip,
        compositor_scroll: layer.compositor_scroll,
    });
    for tile in &layer.tiles {
        if !plan.TileIsReady(tile) {
            return Err(FrameBuildError("compositor-frame-references-unready-tile"));
        }
        let resource_id = ResourceId(tile.tile_id.0);
        if resource_keys.insert((tile.tile_id, tile.generation)) {
            resources.push(TransferableResource {
                id: resource_id,
                tile_id: tile.tile_id,
                generation: tile.generation,
                size: tile.pixel_size,
            });
        }
        quads.push(DrawQuad::Tile(TileDrawQuad {
            shared_quad_state_index,
            resource_id,
            rect: tile.tile_rect,
            visible_rect: tile.tile_rect,
            texture_rect: PaintRect {
                x: 0.0,
                y: 0.0,
                width: tile.pixel_size.0 as f64,
                height: tile.pixel_size.1 as f64,
            },
            nearest_neighbor: composition.transform.is_translation()
                && composition.transform.tx.fract() == 0.0
                && composition.transform.ty.fract() == 0.0,
            tile: tile.clone(),
        }));
    }
    Ok(())
}

fn append_composition_units(
    plan: &FramePlan,
    units: &[OrderedCompositionUnit],
    states: &mut Vec<SharedQuadState>,
    quads: &mut Vec<DrawQuad>,
    render_pass_list: &mut Vec<CompositorRenderPass>,
    surface_passes: &mut Vec<(Arc<EffectPaintPropertyNode>, RenderPassId)>,
    resources: &mut Vec<TransferableResource>,
    resource_keys: &mut BTreeSet<(TileId, u64)>,
    viewport: DeviceRect,
) -> Result<(), FrameBuildError> {
    for unit in units {
        match unit {
            OrderedCompositionUnit::Layer(fragment) => {
                append_layer_quads(plan, fragment, states, quads, resources, resource_keys)?
            }
            OrderedCompositionUnit::Surface(surface) => {
                let pass_id = if let Some((_, pass_id)) = surface_passes
                    .iter()
                    .find(|(owner, _)| owner.lifecycle.same_node(&surface.owner.lifecycle))
                {
                    *pass_id
                } else {
                    let pass_id = effect_pass_id(&surface.owner);
                    // Register before descending so malformed cyclic property
                    // input cannot recurse indefinitely. Valid effect trees
                    // still publish children before parents in pass order.
                    surface_passes.push((surface.owner.clone(), pass_id));
                    let mut child_states = Vec::new();
                    let mut child_quads = Vec::new();
                    append_composition_units(
                        plan,
                        &surface.units,
                        &mut child_states,
                        &mut child_quads,
                        render_pass_list,
                        surface_passes,
                        resources,
                        resource_keys,
                        viewport,
                    )?;
                    render_pass_list.push(CompositorRenderPass {
                        id: pass_id,
                        owner_effect: Some(surface.owner.clone()),
                        // Rechrom's CPU pass backing stays in target device
                        // coordinates. The surface quad carries the tight
                        // output polygon; keeping full-viewport pass storage
                        // preserves global tile/clip coordinates and filter
                        // outsets without a lossy origin translation.
                        output_rect: viewport,
                        damage_rect: viewport,
                        shared_quad_state_list: child_states,
                        quad_list: child_quads,
                    });
                    pass_id
                };
                let representative = &plan.layers[surface.representative_layer_index];
                let shared_quad_state_index = states.len();
                states.push(SharedQuadState {
                    layer_id: representative.id,
                    properties: surface.properties.clone(),
                    composition: surface.composition,
                    visible_polygon: surface.visible_polygon.clone(),
                    retained_clips: Arc::from([]),
                    is_first_layer: false,
                    requires_transparent_backing: true,
                    raster_content_bounds: None,
                    rect_known_to_be_opaque: PaintRect::default(),
                    raster_origin: (0.0, 0.0),
                    root_translation: (0.0, 0.0),
                    root_clip: None,
                    compositor_scroll: representative.compositor_scroll,
                });
                quads.push(DrawQuad::RenderPass(RenderPassDrawQuad {
                    shared_quad_state_index,
                    render_pass_id: pass_id,
                }));
            }
        }
    }
    Ok(())
}

#[allow(non_snake_case)]
impl FrameBuilder {
    pub fn Invalidate(&mut self) {
        self.previous = None;
        self.previous_passes.clear();
    }

    pub fn BuildFrame(&mut self, plan: &FramePlan) -> Result<CompositorFrame, FrameBuildError> {
        if let Some(reason) = plan.unsupported {
            return Err(FrameBuildError(match reason {
                layer_tile::UnsupportedReason::InvalidFrameGeometry => "invalid-frame-geometry",
                layer_tile::UnsupportedReason::Preserve3dContext => {
                    "unsupported-preserve-3d-context"
                }
                _ => "unsupported-frame-plan",
            }));
        }
        let viewport = device_rect(plan.config.viewport, plan.config.raster_scale)?;
        let ordered = ordered_composition_units(plan)?;
        let snapshot = frame_snapshot(plan, viewport, &ordered)?;
        let root_damage = frame_damage(self.previous.as_ref(), &snapshot);
        let mut pass_snapshots = HashMap::new();
        collect_pass_snapshots(plan, &snapshot, &ordered, &mut pass_snapshots);
        let mut shared_quad_state_list = Vec::with_capacity(plan.layers.len());
        let mut quad_list = Vec::new();
        let mut render_pass_list = Vec::new();
        let mut surface_passes: Vec<(Arc<EffectPaintPropertyNode>, RenderPassId)> = Vec::new();
        let mut resource_list = Vec::new();
        let mut resource_keys = BTreeSet::new();
        append_composition_units(
            plan,
            &ordered,
            &mut shared_quad_state_list,
            &mut quad_list,
            &mut render_pass_list,
            &mut surface_passes,
            &mut resource_list,
            &mut resource_keys,
            viewport,
        )?;
        for pass in &mut render_pass_list {
            if self
                .previous_passes
                .get(&pass.id)
                .zip(pass_snapshots.get(&pass.id))
                .is_some_and(|(old, new)| old.same_inputs(new))
            {
                pass.damage_rect = DeviceRect::new(viewport.x, viewport.y, 0, 0);
            }
        }
        let root_pass_id = RenderPassId(0);
        render_pass_list.push(CompositorRenderPass {
            id: root_pass_id,
            owner_effect: None,
            output_rect: viewport,
            damage_rect: root_damage,
            shared_quad_state_list,
            quad_list,
        });
        let frame = CompositorFrame {
            metadata: CompositorFrameMetadata {
                frame_token: self.submission_sequence.wrapping_add(1),
                source_frame_id: plan.frame_id,
                device_scale_factor: plan.config.device_scale_factor,
                raster_scale_factor: plan.config.raster_scale,
                // This is the coordinate-space viewport consumed by Viz and
                // Renderer, so it must stay in PaintArtifact/LayoutZoomFactor
                // units. CSSOM is the boundary that removes absolute zoom;
                // dividing here would mix CSS coordinates into raster replay
                // and make a DPR-zoomed artifact half the render target size.
                logical_viewport: plan.config.viewport,
                viewport,
                root_damage_rect: root_damage,
                raster_task_count: plan.raster_task_count,
                begin_frame_ack: None,
            },
            resource_list,
            render_pass_list,
        };
        self.previous = Some(snapshot);
        self.previous_passes = pass_snapshots;
        self.submission_sequence = frame.metadata.frame_token;
        Ok(frame)
    }
}

#[allow(non_snake_case)]
impl CompositorFrame {
    pub fn WithBeginFrameAck(mut self, ack: BeginFrameAck) -> Self {
        self.metadata.begin_frame_ack = Some(ack);
        self
    }

    /// Append compositor-owned overlay geometry to the root render pass.
    pub fn WithSolidColorQuad(mut self, quad: SolidColorDrawQuad, damage: DeviceRect) -> Self {
        let root = self
            .render_pass_list
            .last_mut()
            .expect("CompositorFrame always has a root render pass");
        root.quad_list.push(DrawQuad::SolidColor(quad));
        self.WithRootDamage(damage)
    }

    pub fn WithRootDamage(mut self, damage: DeviceRect) -> Self {
        let root = self
            .render_pass_list
            .last_mut()
            .expect("CompositorFrame always has a root render pass");
        root.damage_rect = union_device_rect(root.damage_rect, damage);
        self.metadata.root_damage_rect = root.damage_rect;
        self
    }
}

fn union_device_rect(a: DeviceRect, b: DeviceRect) -> DeviceRect {
    if a.width == 0 || a.height == 0 {
        return b;
    }
    if b.width == 0 || b.height == 0 {
        return a;
    }
    let left = a.x.min(b.x);
    let top = a.y.min(b.y);
    let right = (i64::from(a.x) + i64::from(a.width)).max(i64::from(b.x) + i64::from(b.width));
    let bottom = (i64::from(a.y) + i64::from(a.height)).max(i64::from(b.y) + i64::from(b.height));
    DeviceRect::new(
        left,
        top,
        (right - i64::from(left)) as u32,
        (bottom - i64::from(top)) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(x: i32) -> FrameSnapshot {
        FrameSnapshot {
            layer_tree_id: layer_tile::LayerTreeId(1),
            viewport: DeviceRect::new(0, 0, 100, 100),
            layers: vec![LayerSnapshot {
                id: LayerId(7),
                properties: PropertyTreeState::default(),
                composition: LayerComposition {
                    transform: AffineTransform {
                        tx: x as f64,
                        ..AffineTransform::IDENTITY
                    },
                    projective: None,
                    rendering_context_id: 0,
                    clip: None,
                    content_bounds: None,
                    opaque_bounds: None,
                    white_backing: false,
                    grid_min: Some((0, 0)),
                },
                tiles: vec![TileSnapshot {
                    id: TileId(11),
                    generation: 3,
                    tiling_rect: PaintRect::default(),
                    tile_index: (0, 0),
                    rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: 10.0,
                        height: 10.0,
                    },
                    pixel_size: (10, 10),
                    raster_scale: 1.0,
                }],
                output_bounds: Some(DeviceRect::new(x, 0, 10, 10)),
                has_damage_sensitive_effect: false,
                raster_content_bounds: None,
                compositor_scroll: None,
                is_first_layer: false,
                requires_transparent_backing: false,
                raster_origin: (0., 0.),
                root_translation: (0., 0.),
                root_clip: None,
                rect_known_to_be_opaque: PaintRect::default(),
            }],
            draw_order: vec![CompositionSnapshot::Layer(LayerId(7), None)],
            logical_viewport: PaintRect::default(),
            raster_scale: 1.,
            device_scale: 1.,
            owner: None,
        }
    }

    #[test]
    fn device_rect_encloses_fractional_geometry() {
        assert_eq!(
            device_rect(
                PaintRect {
                    x: 0.25,
                    y: 1.25,
                    width: 10.0,
                    height: 5.0,
                },
                2.0,
            )
            .unwrap(),
            DeviceRect::new(0, 2, 21, 11)
        );
    }

    #[test]
    fn retained_damage_is_empty_when_unchanged_and_unions_old_and_new_position() {
        let old = snapshot(5);
        assert_eq!(frame_damage(Some(&old), &old), DeviceRect::new(0, 0, 0, 0));
        let moved = snapshot(8);
        assert_eq!(
            frame_damage(Some(&old), &moved),
            DeviceRect::new(5, 0, 13, 10)
        );
    }
}

#[cfg(test)]
mod retained_damage_tests;

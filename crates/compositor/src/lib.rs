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
    resolved_compositor_properties_with_scroll, FramePlan, LayerId, LayerPlan, TileId,
    TilePlacement,
};
pub use paint::paint_engine::PaintRect;
use paint::paint_property_tree::{ClipPaintPropertyNode, PropertyTreeState};
use std::{collections::BTreeSet, fmt, sync::Arc};

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerComposition {
    pub translation: (f64, f64),
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
pub struct CompositorFrameMetadata {
    pub frame_token: u64,
    pub source_frame_id: u64,
    pub device_scale_factor: f64,
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
    SolidColor(SolidColorDrawQuad),
}

#[derive(Clone, Debug)]
pub struct CompositorRenderPass {
    pub id: RenderPassId,
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
}

#[derive(Clone)]
struct FrameSnapshot {
    viewport: DeviceRect,
    layers: Vec<LayerSnapshot>,
}

/// Retained, backend-independent compositor state. The owner decides which
/// thread calls it; the compositor only turns ready retained tiles into an
/// immutable frame and computes root render-pass damage across submissions.
#[derive(Default)]
pub struct FrameBuilder {
    previous: Option<FrameSnapshot>,
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

fn content_device_rect(
    rect: PaintRect,
    scale: f64,
    translation: (f64, f64),
) -> Result<DeviceRect, FrameBuildError> {
    if rect.is_empty() {
        return Ok(DeviceRect::new(0, 0, 0, 0));
    }
    device_rect(
        PaintRect {
            x: rect.x * scale + translation.0,
            y: rect.y * scale + translation.1,
            width: rect.width * scale,
            height: rect.height * scale,
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
        let mut rect = content_device_rect(tile.tile_rect, scale, composition.translation)?;
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

fn frame_snapshot(
    plan: &FramePlan,
    viewport: DeviceRect,
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
        });
    }
    Ok(FrameSnapshot { viewport, layers })
}

fn frame_damage(previous: Option<&FrameSnapshot>, current: &FrameSnapshot) -> DeviceRect {
    let full = current.viewport;
    let Some(previous) = previous else {
        return full;
    };
    if previous.viewport != current.viewport {
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
        let changed = old.composition != new.composition
            || old.tiles != new.tiles
            || !old.properties.same_values(&new.properties);
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

impl LayerComposition {
    pub fn from_layer(layer: &LayerPlan, scale: f64) -> Result<Self, FrameBuildError> {
        let (translation, clip) = resolved_compositor_properties_with_scroll(
            &layer.properties,
            scale,
            layer.compositor_scroll,
        )
        .map_err(|_| FrameBuildError("unsupported-compositor-properties"))?;
        if ![translation.0, translation.1]
            .into_iter()
            .all(|value| value.is_finite() && value.abs() <= (1 << 22) as f64)
        {
            return Err(FrameBuildError("large-property-placement"));
        }
        let white_backing = layer.is_first_layer && !layer.requires_transparent_backing;
        Ok(Self {
            translation,
            clip: clip.map(|rect| device_rect(rect, 1.0)).transpose()?,
            content_bounds: if white_backing {
                None
            } else {
                layer
                    .raster_content_bounds
                    .map(|rect| content_device_rect(rect, scale, translation))
                    .transpose()?
            },
            opaque_bounds: sampled_opaque_device_rect(
                layer.rect_known_to_be_opaque,
                scale,
                translation,
            )?,
            white_backing,
            grid_min: layer
                .tiles
                .iter()
                .map(|tile| tile.tile_index)
                .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1))),
        })
    }
}

#[allow(non_snake_case)]
impl FrameBuilder {
    pub fn Invalidate(&mut self) {
        self.previous = None;
    }

    pub fn BuildFrame(&mut self, plan: &FramePlan) -> Result<CompositorFrame, FrameBuildError> {
        if let Some(reason) = plan.unsupported {
            return Err(FrameBuildError(match reason {
                layer_tile::UnsupportedReason::InvalidFrameGeometry => "invalid-frame-geometry",
                _ => "unsupported-frame-plan",
            }));
        }
        let viewport = device_rect(plan.config.viewport, plan.config.raster_scale)?;
        let snapshot = frame_snapshot(plan, viewport)?;
        let root_damage = frame_damage(self.previous.as_ref(), &snapshot);
        let mut shared_quad_state_list = Vec::with_capacity(plan.layers.len());
        let mut quad_list = Vec::new();
        let mut resource_list = Vec::new();
        let mut resource_keys = BTreeSet::new();
        for layer in &plan.layers {
            let shared_quad_state_index = shared_quad_state_list.len();
            let composition = LayerComposition::from_layer(layer, plan.config.raster_scale)?;
            shared_quad_state_list.push(SharedQuadState {
                layer_id: layer.id,
                properties: layer.properties.clone(),
                composition,
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
                    resource_list.push(TransferableResource {
                        id: resource_id,
                        tile_id: tile.tile_id,
                        generation: tile.generation,
                        size: tile.pixel_size,
                    });
                }
                quad_list.push(DrawQuad::Tile(TileDrawQuad {
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
                    nearest_neighbor: composition.translation.0.fract() == 0.0
                        && composition.translation.1.fract() == 0.0,
                    tile: tile.clone(),
                }));
            }
        }
        let frame = CompositorFrame {
            metadata: CompositorFrameMetadata {
                frame_token: plan.frame_id,
                source_frame_id: plan.frame_id,
                device_scale_factor: plan.config.raster_scale,
                logical_viewport: plan.config.viewport,
                viewport,
                root_damage_rect: root_damage,
                raster_task_count: plan.raster_task_count,
                begin_frame_ack: None,
            },
            resource_list,
            render_pass_list: vec![CompositorRenderPass {
                id: RenderPassId(1),
                output_rect: viewport,
                damage_rect: root_damage,
                shared_quad_state_list,
                quad_list,
            }],
        };
        self.previous = Some(snapshot);
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
            viewport: DeviceRect::new(0, 0, 100, 100),
            layers: vec![LayerSnapshot {
                id: LayerId(7),
                properties: PropertyTreeState::default(),
                composition: LayerComposition {
                    translation: (x as f64, 0.0),
                    clip: None,
                    content_bounds: None,
                    opaque_bounds: None,
                    white_backing: false,
                    grid_min: Some((0, 0)),
                },
                tiles: vec![TileSnapshot {
                    id: TileId(11),
                    generation: 3,
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
            }],
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

#![allow(non_snake_case)]

use crate::{
    adapter::{
        CoreRangeFromRuntime, DeviceRect, DisplayResource, DisplayRootFrame, DrawReceipt,
        RootPresentation, SurfaceId, SurfacePlacement, VizError, VizOutput,
    },
    common::{
        quads::aggregated_render_pass::AggregatedRenderPass as OwnedAggregatedRenderPass,
        surfaces::{
            frame_sink_id::FrameSinkId, local_surface_id::LocalSurfaceId,
            surface_id::SurfaceId as CoreSurfaceId, surface_range::SurfaceRange,
        },
    },
    service::{
        display::{
            aggregated_frame::AggregatedFrame,
            display_damage_tracker::{DisplayDamageTracker, UnionRect},
            resolved_frame_data::{FrameValidationError, ResolvedFrameData},
        },
        surfaces::surface_manager::SurfaceManager,
    },
};
use compositor_core::{
    adapter::{
        geometry::{Rect, RectF, ToEnclosingRect},
        layer_tree_impl::Color4f,
        layer_tree_impl::ResourceId,
    },
    cc::base::math_util::MathUtil,
    runtime::frame::{QuadMaterial, RenderPassId, SharedQuadStateHandle},
    CompositorFrame, CompositorRenderPass,
};
use std::collections::{BTreeMap, BTreeSet};

// Source: components/viz/service/display/surface_aggregator.h:1-133
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExtraPassForReadbackOption {
    #[default]
    kNone,
    kAddPassForReadback,
    kAlwaysAddPass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalCapability {
    CopyOutput,
    OverlayPromotion,
    DelegatedInk,
    DisplayColorConversion,
    TrackedElementTransform,
}

#[derive(Clone, Debug, Default)]
struct ResolvedFrameRecord {
    frame_index: u32,
    root_pass_id: RenderPassId,
    valid: bool,
    pass_ids: BTreeMap<RenderPassId, u64>,
}

#[derive(Default)]
struct PrewalkResult {
    undrawn_surfaces: BTreeSet<CoreSurfaceId>,
    frame_sinks_changed: bool,
    root_damage: Rect,
    contained_surfaces: BTreeSet<CoreSurfaceId>,
    contained_frame_sinks: BTreeMap<FrameSinkId, LocalSurfaceId>,
    referenced_surfaces: BTreeSet<CoreSurfaceId>,
    surface_damage_rects: Vec<Rect>,
    pass_damage: BTreeMap<(CoreSurfaceId, RenderPassId), Rect>,
}

// Source: components/viz/service/display/surface_aggregator.cc:844-872,
//         1092-1134. A non-merged SurfaceDrawQuad becomes an RPDQ whose
// geometry is in the child root pass's content space. The copied shared state
// carries the child-content-to-parent-surface scale.
#[derive(Clone, Copy, Debug)]
struct EmbeddedSurfaceRoot {
    pass_id: u64,
    output_rect: Rect,
    extra_content_scale_x: f32,
    extra_content_scale_y: f32,
}

fn ScaleToEnclosingRect2d(rect: Rect, x_scale: f32, y_scale: f32) -> Rect {
    if x_scale == 1.0 && y_scale == 1.0 {
        return rect;
    }
    ToEnclosingRect(RectF::new(
        rect.x as f32 * x_scale,
        rect.y as f32 * y_scale,
        rect.width as f32 * x_scale,
        rect.height as f32 * y_scale,
    ))
}

#[derive(Default)]
pub struct SurfaceAggregator {
    previous_root_placements_: BTreeMap<SurfaceId, Rect>,
    previous_display_root_: Option<DisplayRootFrame>,
    previous_contained_surfaces_: BTreeSet<CoreSurfaceId>,
    previous_contained_frame_sinks_: BTreeMap<FrameSinkId, LocalSurfaceId>,
    contained_surfaces_: BTreeSet<CoreSurfaceId>,
    contained_frame_sinks_: BTreeMap<FrameSinkId, LocalSurfaceId>,
    resolved_frames_: BTreeMap<CoreSurfaceId, ResolvedFrameRecord>,
    resolved_surface_ranges_: BTreeMap<SurfaceRange, Option<CoreSurfaceId>>,
    damage_ranges_: BTreeMap<FrameSinkId, Vec<SurfaceRange>>,
    full_damage_: BTreeSet<CoreSurfaceId>,
    added_surfaces_: Vec<CoreSurfaceId>,
    removed_surfaces_: Vec<CoreSurfaceId>,
    root_surface_id_: Option<CoreSurfaceId>,
    root_damage_rect_: Rect,
    max_render_target_size_: i32,
    output_is_secure_: bool,
    take_copy_requests_: bool,
    needs_surface_damage_rect_list_: bool,
    extra_pass_for_readback_option_: ExtraPassForReadbackOption,
    prevent_merging_surfaces_to_root_pass_: bool,
    is_inside_aggregate_: bool,
    next_render_pass_id_: u64,
    display_root_pass_id_: Option<u64>,
    previous_output_rect_: Rect,
    resource_ids_: BTreeMap<(CoreSurfaceId, u32), ResourceId>,
    next_resource_id_: u32,
}

// Source: components/viz/service/display/surface_aggregator.h:134-240
impl SurfaceAggregator {
    pub fn new() -> Self {
        Self {
            take_copy_requests_: true,
            needs_surface_damage_rect_list_: true,
            next_render_pass_id_: 1,
            next_resource_id_: 1,
            ..Self::default()
        }
    }

    pub fn previous_contained_surfaces(&self) -> &BTreeSet<CoreSurfaceId> {
        &self.previous_contained_surfaces_
    }

    pub fn previous_contained_frame_sinks(&self) -> &BTreeMap<FrameSinkId, LocalSurfaceId> {
        &self.previous_contained_frame_sinks_
    }

    pub fn GetLatestFrameData<'a>(
        &self,
        manager: &'a SurfaceManager,
        surface_id: CoreSurfaceId,
    ) -> Result<Option<ResolvedFrameData<'a>>, FrameValidationError> {
        let Some(frame) = manager
            .GetSurfaceForId(surface_id)
            .and_then(|surface| surface.active_frame())
        else {
            return Ok(None);
        };
        let previous = self
            .resolved_frames_
            .get(&surface_id)
            .map_or(0, |record| record.frame_index);
        Ok(Some(ResolvedFrameData::new_with_previous(
            surface_id,
            frame,
            previous,
            self.resolved_frames_
                .get(&surface_id)
                .map(|record| record.root_pass_id.0),
        )?))
    }

    fn GetResolvedSurfaceForRange(
        &mut self,
        manager: &SurfaceManager,
        range: &SurfaceRange,
    ) -> Option<CoreSurfaceId> {
        if let Some(resolved) = self.resolved_surface_ranges_.get(range) {
            return *resolved;
        }
        let resolved = manager
            .GetLatestInFlightSurface(range)
            .map(|surface| surface.surface_id());
        self.resolved_surface_ranges_
            .insert(range.clone(), resolved);
        resolved
    }

    // SurfaceDrawQuad embedding enters this CPU Engine as SurfacePlacement for
    // root content and as SurfaceRange metadata for referenced descendants.
    fn HandleSurfacePlacement(
        &mut self,
        manager: &SurfaceManager,
        damage: &DisplayDamageTracker,
        public_id: SurfaceId,
        core_id: CoreSurfaceId,
        destination: Rect,
        result: &mut PrewalkResult,
        passes: &mut Vec<OwnedAggregatedRenderPass>,
        resources: &mut BTreeMap<u32, DisplayResource>,
        roots: &mut Vec<RootPresentation>,
        receipt: &mut DrawReceipt,
        core_to_public: &BTreeMap<CoreSurfaceId, SurfaceId>,
        copied_surfaces: &mut BTreeSet<CoreSurfaceId>,
        copy_stack: &mut BTreeSet<CoreSurfaceId>,
    ) -> Result<(), VizError> {
        let surface = manager
            .GetSurfaceForId(core_id)
            .ok_or(VizError::InvalidSurface)?;
        let frame = surface.active_frame().ok_or(VizError::InvalidSurface)?;
        let resolved = ResolvedFrameData::new_with_previous(
            core_id,
            frame,
            self.resolved_frames_
                .get(&core_id)
                .map_or(0, |record| record.frame_index),
            self.resolved_frames_
                .get(&core_id)
                .map(|record| record.root_pass_id.0),
        )
        .map_err(VizError::Frame)?;

        let surface_damage = self.PrewalkSurface(manager, damage, core_id, false, result)?;
        let source_rect = resolved.root_pass().render_pass().output_rect;
        let destination_damage = MapRectToDestination(source_rect, destination, surface_damage);
        result.root_damage.Union(destination_damage);
        self.AddSurfaceDamageToDamageList(destination_damage, result);
        self.CopyPassesOwned(
            manager,
            destination,
            frame,
            core_id,
            &resolved,
            damage,
            passes,
            resources,
            roots,
            receipt,
            core_to_public,
            true,
            copied_surfaces,
            copy_stack,
            &result.pass_damage,
        )?;
        if !receipt.0.contains(&(public_id, frame.submission)) {
            receipt.0.push((public_id, frame.submission));
        }
        Ok(())
    }

    // Source: components/viz/service/display/surface_aggregator.h:241-340
    fn PrewalkRenderPass(
        &mut self,
        manager: &SurfaceManager,
        damage_tracker: &DisplayDamageTracker,
        frame: &CompositorFrame,
        pass_index: usize,
        pass_damage: &BTreeMap<RenderPassId, Rect>,
        force_full_damage: bool,
        visiting: &mut BTreeSet<RenderPassId>,
        result: &mut PrewalkResult,
    ) -> Result<Rect, VizError> {
        let pass = &frame.render_pass_list[pass_index];
        if !visiting.insert(pass.id) {
            return Ok(Rect::default());
        }
        let mut damage = if force_full_damage {
            pass.output_rect
        } else {
            pass_damage
                .get(&pass.id)
                .copied()
                .unwrap_or(pass.damage_rect)
        };

        for quad in &pass.quad_list {
            let mut child_damage = match &quad.material {
                QuadMaterial::CompositorRenderPass { render_pass_id, .. } => {
                    let Some(child_index) = frame
                        .render_pass_list
                        .iter()
                        .position(|candidate| candidate.id == *render_pass_id)
                    else {
                        continue;
                    };
                    let damage = self.PrewalkRenderPass(
                        manager,
                        damage_tracker,
                        frame,
                        child_index,
                        pass_damage,
                        force_full_damage,
                        visiting,
                        result,
                    )?;
                    let child = &frame.render_pass_list[child_index];
                    MapRectToDestination(child.output_rect, quad.geometry.rect, damage)
                }
                QuadMaterial::Surface { surface_range, .. } => {
                    let range = CoreRangeFromRuntime(surface_range);
                    self.AddDamageRanges(std::slice::from_ref(&range));
                    if let Some(child_id) = self.GetResolvedSurfaceForRange(manager, &range) {
                        let child_root = manager
                            .GetSurfaceForId(child_id)
                            .and_then(|surface| surface.active_frame())
                            .and_then(|child| child.frame.render_pass_list.last())
                            .map(|root| root.output_rect);
                        let child_damage =
                            self.PrewalkSurface(manager, damage_tracker, child_id, false, result)?;
                        if child_id != range.end() {
                            // A fallback may introduce stretch/gutter, so Chromium
                            // conservatively damages the full SurfaceDrawQuad.
                            quad.geometry.visible_rect
                        } else if let Some(child_root) = child_root {
                            MapRectToDestination(child_root, quad.geometry.rect, child_damage)
                        } else {
                            quad.geometry.visible_rect
                        }
                    } else {
                        // Source surface_aggregator.cc:801-808,1728-1739: a
                        // missing child paints the entire default background.
                        quad.geometry.visible_rect
                    }
                }
                _ => continue,
            };
            // Chromium maps contributing render-pass damage through the RPDQ
            // into its parent target. Keeping child-local damage here makes
            // the host output claim pixels were repainted outside the region
            // DirectRenderer actually writes, which breaks rotating-buffer
            // preservation after a surface switch.
            if !child_damage.IsEmpty() {
                child_damage.Intersect(quad.geometry.visible_rect);
                if let Some(shared) = quad
                    .geometry
                    .shared_quad_state
                    .and_then(|id| pass.TrySharedQuadState(id))
                {
                    child_damage = MathUtil::MapEnclosingClippedRect(
                        &shared.quad_to_target_transform,
                        child_damage,
                    );
                    if let Some(clip) = shared.clip_rect {
                        child_damage.Intersect(clip);
                    }
                }
                damage.Union(child_damage);
            }
        }
        visiting.remove(&pass.id);
        damage.Intersect(pass.output_rect);
        Ok(damage)
    }

    fn CopyPassesOwned(
        &mut self,
        manager: &SurfaceManager,
        destination: Rect,
        frame: &crate::service::surfaces::surface::FrameData,
        surface_id: CoreSurfaceId,
        resolved: &ResolvedFrameData<'_>,
        damage: &DisplayDamageTracker,
        passes: &mut Vec<OwnedAggregatedRenderPass>,
        resources: &mut BTreeMap<u32, DisplayResource>,
        roots: &mut Vec<RootPresentation>,
        receipt: &mut DrawReceipt,
        core_to_public: &BTreeMap<CoreSurfaceId, SurfaceId>,
        present_root: bool,
        copied_surfaces: &mut BTreeSet<CoreSurfaceId>,
        copy_stack: &mut BTreeSet<CoreSurfaceId>,
        prewalk_damage: &BTreeMap<(CoreSurfaceId, RenderPassId), Rect>,
    ) -> Result<(), VizError> {
        if !copy_stack.insert(surface_id) {
            return Ok(());
        }
        if !copied_surfaces.insert(surface_id) {
            if present_root {
                if let Some(pass_id) = frame.frame.render_pass_list.last().and_then(|pass| {
                    self.resolved_frames_
                        .get(&surface_id)?
                        .pass_ids
                        .get(&pass.id)
                }) {
                    roots.push(RootPresentation {
                        pass_id: *pass_id,
                        destination,
                    });
                }
            }
            copy_stack.remove(&surface_id);
            return Ok(());
        }
        let known_ids = self
            .resolved_frames_
            .get(&surface_id)
            .map(|record| record.pass_ids.clone())
            .unwrap_or_default();
        let mut remapped_ids = known_ids;
        for pass in &frame.frame.render_pass_list {
            remapped_ids.entry(pass.id).or_insert_with(|| {
                let id = self.next_render_pass_id_.max(1);
                self.next_render_pass_id_ = id.wrapping_add(1).max(1);
                id
            });
        }
        if let Some(record) = self.resolved_frames_.get_mut(&surface_id) {
            record.pass_ids = remapped_ids.clone();
        }
        let state = damage.State(surface_id);
        let damage_continuous = state.is_none_or(|value| value.continuous);
        let mut remapped_resources = BTreeMap::new();
        for resource in &frame.frame.resource_list {
            let display_id = *self
                .resource_ids_
                .entry((surface_id, resource.id.0))
                .or_insert_with(|| {
                    let id = ResourceId(self.next_resource_id_.max(1));
                    self.next_resource_id_ = id.0.wrapping_add(1).max(1);
                    id
                });
            remapped_resources.insert(resource.id.0, display_id);
            // Source: DisplayResourceProvider::ReceiveFromChild keeps the
            // complete TransferableResource in ChildResource. Retaining only
            // RasterReadLease here dropped non-host-readable GPU SharedImages
            // before a SkiaRenderer image context could import them.
            let mut transferable = resource.clone();
            transferable.id = display_id;
            resources.insert(
                display_id.0,
                DisplayResource {
                    read_lease: resource.RasterReadLease(),
                    transferable,
                },
            );
        }
        let source_damage: BTreeMap<_, _> = frame
            .frame
            .render_pass_list
            .iter()
            .map(|pass| {
                let mut value = if self.full_damage_.contains(&surface_id) {
                    pass.output_rect
                } else {
                    damage.Damage(surface_id, pass.id, pass.damage_rect)
                };
                if let Some(contributing) = prewalk_damage.get(&(surface_id, pass.id)) {
                    value.Union(*contributing);
                }
                (pass.id, value)
            })
            .collect();

        for (index, source) in frame.frame.render_pass_list.iter().enumerate() {
            let mut nested_roots = BTreeMap::new();
            for (quad_index, quad) in source.quad_list.iter().enumerate() {
                let QuadMaterial::Surface {
                    surface_range,
                    stretch_content_to_fill_bounds,
                    ..
                } = &quad.material
                else {
                    continue;
                };
                let range = CoreRangeFromRuntime(surface_range);
                let child_root =
                    if let Some(child_id) = self.GetResolvedSurfaceForRange(manager, &range) {
                        if copy_stack.contains(&child_id) {
                            nested_roots.insert(quad_index, None);
                            continue;
                        }
                        let child_surface = manager.GetSurfaceForId(child_id);
                        let child_frame = child_surface.and_then(|surface| surface.active_frame());
                        if let Some(child_frame) = child_frame {
                            let child_resolved = ResolvedFrameData::new_with_previous(
                                child_id,
                                child_frame,
                                self.resolved_frames_
                                    .get(&child_id)
                                    .map_or(0, |record| record.frame_index),
                                self.resolved_frames_
                                    .get(&child_id)
                                    .map(|record| record.root_pass_id.0),
                            )
                            .map_err(VizError::Frame)?;
                            self.CopyPassesOwned(
                                manager,
                                Rect::default(),
                                child_frame,
                                child_id,
                                &child_resolved,
                                damage,
                                passes,
                                resources,
                                roots,
                                receipt,
                                core_to_public,
                                false,
                                copied_surfaces,
                                copy_stack,
                                prewalk_damage,
                            )?;
                            if let Some(public_id) = core_to_public.get(&child_id).copied() {
                                let pair = (public_id, child_frame.submission);
                                if !receipt.0.contains(&pair) {
                                    receipt.0.push(pair);
                                }
                            }
                            child_frame.frame.render_pass_list.last().and_then(|root| {
                                let pass_id = self
                                    .resolved_frames_
                                    .get(&child_id)?
                                    .pass_ids
                                    .get(&root.id)
                                    .copied()?;
                                let (extra_content_scale_x, extra_content_scale_y) =
                                    if *stretch_content_to_fill_bounds {
                                        (
                                            quad.geometry.rect.width as f32
                                                / root.output_rect.width.max(1) as f32,
                                            quad.geometry.rect.height as f32
                                                / root.output_rect.height.max(1) as f32,
                                        )
                                    } else {
                                        let parent_scale = frame
                                            .frame
                                            .metadata
                                            .device_scale_factor
                                            .max(f32::MIN_POSITIVE);
                                        let child_scale = child_frame
                                            .frame
                                            .metadata
                                            .device_scale_factor
                                            .max(f32::MIN_POSITIVE);
                                        let scale = parent_scale / child_scale;
                                        (scale, scale)
                                    };
                                Some(EmbeddedSurfaceRoot {
                                    pass_id,
                                    output_rect: root.output_rect,
                                    extra_content_scale_x,
                                    extra_content_scale_y,
                                })
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                nested_roots.insert(quad_index, child_root);
            }
            let mut output = source.output_rect;
            if self.max_render_target_size_ > 0 {
                output.width = output.width.min(self.max_render_target_size_);
                output.height = output.height.min(self.max_render_target_size_);
            }
            let force_full = !damage_continuous
                || (!frame.frame_drawn && RenderPassNeedsFullDamage(source, false))
                || self.full_damage_.contains(&surface_id);
            let pass_damage = SetRenderPassDamageRect(
                source,
                source_damage.get(&source.id).copied().unwrap_or_default(),
                Rect::default(),
                force_full,
            );
            let mut copy = OwnedAggregatedRenderPass::new(
                source.shared_quad_state_list.len(),
                source.quad_list.len(),
            );
            copy.SetAll(
                remapped_ids[&source.id],
                output,
                pass_damage,
                source.transform_to_root_target.clone(),
                frame.frame.metadata.content_color_usage,
                source.has_transparent_background,
                source.cache_render_pass,
                !frame.frame_drawn && source.has_damage_from_contributing_content,
                source.generate_mipmap,
            );
            copy.is_from_surface_root_pass = index + 1 == frame.frame.render_pass_list.len();
            copy.filters = source.filters.clone();
            copy.backdrop_filters = source.backdrop_filters.clone();
            copy.backdrop_filter_bounds = source.backdrop_filter_bounds.clone();
            copy.shared_quad_state_list = source.shared_quad_state_list.clone();
            copy.quad_list = source.quad_list.clone();

            // Chromium's prewalk traverses back-to-front. Damage accumulated
            // below an RPDQ invalidates its intersects_damage_under hint.
            let mut damage_under = pass_damage;
            for (quad_index, quad) in copy.quad_list.iter_mut().enumerate().rev() {
                let drawable = quad.geometry.visible_rect;
                match &mut quad.material {
                    QuadMaterial::Tile { resource_id, .. }
                    | QuadMaterial::Texture { resource_id, .. } => {
                        *resource_id = *remapped_resources
                            .get(&resource_id.0)
                            .ok_or(VizError::MissingResource)?;
                    }
                    QuadMaterial::CompositorRenderPass {
                        render_pass_id,
                        mask_resource_id,
                        intersects_damage_under,
                        ..
                    } => {
                        if mask_resource_id.0 != 0 {
                            *mask_resource_id = *remapped_resources
                                .get(&mask_resource_id.0)
                                .ok_or(VizError::MissingResource)?;
                        }
                        let source_child_id = *render_pass_id;
                        *intersects_damage_under |= drawable.Intersects(damage_under);
                        if let Some(child_damage) = source_damage.get(&source_child_id) {
                            let mut bounded = *child_damage;
                            bounded.Intersect(drawable);
                            damage_under.Union(bounded);
                        }
                        if let Some(remapped) = remapped_ids.get(&source_child_id) {
                            *render_pass_id = RenderPassId::from(*remapped);
                        }
                    }
                    QuadMaterial::Surface {
                        default_background_color,
                        ..
                    } => {
                        let color = *default_background_color;
                        quad.material = if let Some(Some(child_root)) =
                            nested_roots.get(&quad_index)
                        {
                            // Chromium keeps the RPDQ in the child root pass's
                            // content space and scales its copied SQS into the
                            // SurfaceDrawQuad destination. Keeping the parent
                            // destination geometry here loses the source rect
                            // and makes software sampling depend on DPR.
                            // Source: surface_aggregator.cc:1060-1134.
                            let inverse_x = 1.0 / child_root.extra_content_scale_x;
                            let inverse_y = 1.0 / child_root.extra_content_scale_y;
                            if let Some(state) = quad.geometry.shared_quad_state {
                                if let Some(mut shared) =
                                    copy.shared_quad_state_list.get(state.Index()).cloned()
                                {
                                    shared.quad_to_target_transform.transform_mut().Scale(
                                        child_root.extra_content_scale_x,
                                        child_root.extra_content_scale_y,
                                    );
                                    shared.quad_layer_rect = ScaleToEnclosingRect2d(
                                        shared.quad_layer_rect,
                                        inverse_x,
                                        inverse_y,
                                    );
                                    shared.visible_quad_layer_rect = ScaleToEnclosingRect2d(
                                        shared.visible_quad_layer_rect,
                                        inverse_x,
                                        inverse_y,
                                    );
                                    let state_index = copy.shared_quad_state_list.len();
                                    copy.shared_quad_state_list.push(shared);
                                    quad.geometry.shared_quad_state =
                                        Some(SharedQuadStateHandle::FromIndex(state_index));
                                }
                            }
                            quad.geometry.rect = child_root.output_rect;
                            quad.geometry.visible_rect = ScaleToEnclosingRect2d(
                                quad.geometry.visible_rect,
                                inverse_x,
                                inverse_y,
                            );
                            quad.geometry.visible_rect.Intersect(child_root.output_rect);
                            QuadMaterial::CompositorRenderPass {
                                render_pass_id: RenderPassId::from(child_root.pass_id),
                                mask_resource_id: ResourceId::default(),
                                mask_uv_rect: Default::default(),
                                mask_texture_size: Default::default(),
                                filters_scale: compositor_core::adapter::geometry::Vector2dF::new(
                                    1.0, 1.0,
                                ),
                                filters_origin: Default::default(),
                                force_anti_aliasing_off: false,
                                backdrop_filter_quality: 1.0,
                                intersects_damage_under: drawable.Intersects(damage_under),
                            }
                        } else {
                            // Source surface_aggregator.cc:1118-1139.
                            QuadMaterial::SolidColor {
                                color,
                                force_anti_aliasing_off: false,
                            }
                        };
                        damage_under.Union(drawable);
                    }
                    _ => {}
                }
            }
            if present_root
                && resolved
                    .passes()
                    .get(index)
                    .is_some_and(|pass| pass.is_root())
            {
                roots.push(RootPresentation {
                    pass_id: copy.id,
                    destination,
                });
            }
            passes.push(copy);
        }
        copy_stack.remove(&surface_id);
        Ok(())
    }

    fn ProcessAddedAndRemovedSurfaces(&mut self) {
        self.added_surfaces_ = self
            .contained_surfaces_
            .difference(&self.previous_contained_surfaces_)
            .copied()
            .collect();
        self.removed_surfaces_ = self
            .previous_contained_surfaces_
            .difference(&self.contained_surfaces_)
            .copied()
            .collect();
        let removed: BTreeSet<_> = self.removed_surfaces_.iter().copied().collect();
        self.resource_ids_
            .retain(|(surface, _), _| !removed.contains(surface));
    }

    /// Materialize the browser display root that owns embedded Surface roots.
    ///
    /// Chromium normally reaches SurfaceAggregator with a browser root
    /// Surface containing SurfaceDrawQuads for the toolbar and WebContents.
    /// Mergeable child roots are moved directly into that root pass, which
    /// DirectRenderer binds to the output surface. Rechrom carries the same
    /// embedding as `SurfacePlacement`; materialize the equivalent display
    /// root here instead of publishing each child root through an additional
    /// full-frame software copy.
    fn MaterializeDisplayRoot(
        &mut self,
        output_rect: Rect,
        has_copy_requests: bool,
        passes: &mut Vec<OwnedAggregatedRenderPass>,
        roots: &mut Vec<RootPresentation>,
        output_damage_rects: &[Rect],
        color_usage: compositor_core::adapter::layer_tree_host_impl::ContentColorUsage,
        background_color: Color4f,
        solid_color_quads: &[crate::adapter::DisplayRootSolidColorQuad],
    ) {
        if self.prevent_merging_surfaces_to_root_pass_ || has_copy_requests || roots.is_empty() {
            return;
        }
        if roots.len() == 1 && roots[0].destination == output_rect && solid_color_quads.is_empty() {
            // A single full-output Surface root is already the display root.
            // Wrapping it in an RPDQ only adds an offscreen pass and defeats
            // DirectRenderer's root-target path.
            return;
        }

        let root_passes: Option<Vec<_>> = roots
            .iter()
            .map(|root| {
                let index = passes.iter().position(|pass| pass.id == root.pass_id)?;
                let pass = &passes[index];
                let same_size = pass.output_rect.width == root.destination.width
                    && pass.output_rect.height == root.destination.height;
                let mergeable = same_size
                    && pass.transform_to_root_target.as_source().IsIdentity()
                    && !pass.quad_list.is_empty()
                    && !pass.cache_render_pass
                    && !pass.generate_mipmap
                    && compositor_core::adapter::property_types::FilterOperations::IsEmpty(
                        &pass.filters,
                    )
                    && compositor_core::adapter::property_types::FilterOperations::IsEmpty(
                        &pass.backdrop_filters,
                    )
                    && pass.backdrop_filter_bounds.is_none()
                    && pass
                        .shared_quad_state_list
                        .iter()
                        .all(|state| state.mask_filter_info.IsEmpty())
                    && pass.quad_list.iter().all(|quad| {
                        quad.geometry
                            .shared_quad_state
                            .is_some_and(|state| state.Index() < pass.shared_quad_state_list.len())
                    });
                Some((index, root.destination, mergeable))
            })
            .collect();
        let Some(root_passes) = root_passes else {
            return;
        };
        let distinct_roots: BTreeSet<_> = root_passes.iter().map(|(index, _, _)| *index).collect();
        if distinct_roots.len() != root_passes.len() {
            return;
        }
        if root_passes
            .iter()
            .any(|(index, _, mergeable)| !mergeable && passes[*index].output_rect.IsEmpty())
        {
            return;
        }

        let id = self.display_root_pass_id_.unwrap_or_else(|| {
            let id = self.next_render_pass_id_.max(1);
            self.next_render_pass_id_ = id.wrapping_add(1).max(1);
            self.display_root_pass_id_ = Some(id);
            id
        });
        let mut damage = Rect::default();
        for rect in output_damage_rects {
            damage.Union(*rect);
        }
        damage.Intersect(output_rect);
        let state_count = root_passes
            .iter()
            .map(|(index, _, mergeable)| {
                if *mergeable {
                    passes[*index].shared_quad_state_list.len()
                } else {
                    1
                }
            })
            .sum::<usize>()
            + solid_color_quads.len()
            + 1;
        let quad_count = root_passes
            .iter()
            .map(|(index, _, mergeable)| {
                if *mergeable {
                    passes[*index].quad_list.len()
                } else {
                    1
                }
            })
            .sum::<usize>()
            + solid_color_quads.len()
            + 1;
        let mut display_root = OwnedAggregatedRenderPass::new(state_count, quad_count);
        let transparent_background = background_color.alpha < 1.0;
        display_root.SetAll(
            id,
            output_rect,
            damage,
            Default::default(),
            color_usage,
            transparent_background,
            false,
            !damage.IsEmpty(),
            false,
        );
        display_root.is_from_surface_root_pass = true;

        // Aggregated quad lists are front-to-back and DirectRenderer draws
        // them in reverse. Placements are back-to-front, so append roots in
        // reverse placement order while preserving each root's quad order.
        let mut owned_passes: Vec<_> = passes.drain(..).map(Some).collect();
        for (index, destination, mergeable) in root_passes.iter().rev() {
            let pass = owned_passes[*index]
                .as_ref()
                .expect("display roots have distinct render passes");
            if !mergeable {
                let source = pass.output_rect;
                let scale_x = destination.width as f32 / source.width as f32;
                let scale_y = destination.height as f32 / source.height as f32;
                let mut state =
                    crate::common::quads::aggregated_render_pass::SharedQuadState::default();
                state
                    .quad_to_target_transform
                    .transform_mut()
                    .Scale(scale_x, scale_y);
                state
                    .quad_to_target_transform
                    .transform_mut()
                    .PostTranslate(
                        destination.x as f32 - source.x as f32 * scale_x,
                        destination.y as f32 - source.y as f32 * scale_y,
                    );
                state.quad_layer_rect = source;
                state.visible_quad_layer_rect = source;
                let mut clip = *destination;
                clip.Intersect(output_rect);
                state.clip_rect = Some(clip);
                state.are_contents_opaque = !pass.has_transparent_background;
                state.opacity = 1.0;
                let state_index = display_root.shared_quad_state_list.len();
                display_root.shared_quad_state_list.push(state);
                display_root
                    .quad_list
                    .push(compositor_core::runtime::frame::DrawQuad {
                        geometry: compositor_core::runtime::frame::QuadGeometry {
                            shared_quad_state: Some(SharedQuadStateHandle::FromIndex(state_index)),
                            rect: source,
                            visible_rect: source,
                            needs_blending: pass.has_transparent_background,
                        },
                        material: QuadMaterial::CompositorRenderPass {
                            render_pass_id: pass.RenderPassId(),
                            mask_resource_id: ResourceId::default(),
                            mask_uv_rect: Default::default(),
                            mask_texture_size: Default::default(),
                            filters_scale: compositor_core::adapter::geometry::Vector2dF::new(
                                1.0, 1.0,
                            ),
                            filters_origin: Default::default(),
                            force_anti_aliasing_off: false,
                            backdrop_filter_quality: 1.0,
                            intersects_damage_under: destination.Intersects(damage),
                        },
                    });
                continue;
            }
            let pass = owned_passes[*index]
                .take()
                .expect("mergeable roots have distinct render passes");
            let dx = destination.x - pass.output_rect.x;
            let dy = destination.y - pass.output_rect.y;
            let state_base = display_root.shared_quad_state_list.len();
            for mut state in pass.shared_quad_state_list {
                state
                    .quad_to_target_transform
                    .transform_mut()
                    .PostTranslate(dx as f32, dy as f32);
                let mut clip = state.clip_rect.unwrap_or(*destination);
                if state.clip_rect.is_some() {
                    clip.x = clip.x.saturating_add(dx);
                    clip.y = clip.y.saturating_add(dy);
                    clip.Intersect(*destination);
                }
                state.clip_rect = Some(clip);
                display_root.shared_quad_state_list.push(state);
            }
            for mut quad in pass.quad_list {
                let state = quad
                    .geometry
                    .shared_quad_state
                    .expect("mergeable root quads have shared state");
                quad.geometry.shared_quad_state =
                    Some(SharedQuadStateHandle::FromIndex(state_base + state.Index()));
                display_root.quad_list.push(quad);
            }
        }

        // BrowserView-owned backgrounds are part of the browser root frame,
        // behind renderer-owned toolbar/WebContents Surfaces but in front of
        // the widget background. This keeps fallback gutters inside Viz's
        // normal quad ordering, damage and color pipeline.
        for backdrop in solid_color_quads {
            let mut visible = backdrop.destination;
            visible.Intersect(output_rect);
            if visible.IsEmpty() {
                continue;
            }
            let mut state =
                crate::common::quads::aggregated_render_pass::SharedQuadState::default();
            state.quad_layer_rect = backdrop.destination;
            state.visible_quad_layer_rect = visible;
            state.clip_rect = Some(output_rect);
            state.are_contents_opaque = backdrop.color.alpha >= 1.0;
            state.opacity = 1.0;
            let state_index = display_root.shared_quad_state_list.len();
            display_root.shared_quad_state_list.push(state);
            display_root
                .quad_list
                .push(compositor_core::runtime::frame::DrawQuad {
                    geometry: compositor_core::runtime::frame::QuadGeometry {
                        shared_quad_state: Some(SharedQuadStateHandle::FromIndex(state_index)),
                        rect: backdrop.destination,
                        visible_rect: visible,
                        needs_blending: backdrop.color.alpha < 1.0,
                    },
                    material: QuadMaterial::SolidColor {
                        color: backdrop.color,
                        force_anti_aliasing_off: false,
                    },
                });
        }

        // SurfacePlacement is the translated form of the browser root's
        // child SurfaceDrawQuads. Its metadata supplies the root background
        // that Chromium exposes through fallback gutters. Keep the solid quad
        // last because aggregated quad lists are front-to-back.
        let mut background_state =
            crate::common::quads::aggregated_render_pass::SharedQuadState::default();
        background_state.quad_layer_rect = output_rect;
        background_state.visible_quad_layer_rect = output_rect;
        background_state.clip_rect = Some(output_rect);
        background_state.are_contents_opaque = !transparent_background;
        background_state.opacity = 1.0;
        let background_state_index = display_root.shared_quad_state_list.len();
        display_root.shared_quad_state_list.push(background_state);
        display_root
            .quad_list
            .push(compositor_core::runtime::frame::DrawQuad {
                geometry: compositor_core::runtime::frame::QuadGeometry {
                    shared_quad_state: Some(SharedQuadStateHandle::FromIndex(
                        background_state_index,
                    )),
                    rect: output_rect,
                    visible_rect: output_rect,
                    needs_blending: transparent_background,
                },
                material: QuadMaterial::SolidColor {
                    color: background_color,
                    force_anti_aliasing_off: false,
                },
            });

        passes.extend(owned_passes.into_iter().flatten());
        passes.push(display_root);
        roots.clear();
        roots.push(RootPresentation {
            pass_id: id,
            destination: output_rect,
        });
    }

    pub fn added_surfaces(&self) -> &[CoreSurfaceId] {
        &self.added_surfaces_
    }

    pub fn removed_surfaces(&self) -> &[CoreSurfaceId] {
        &self.removed_surfaces_
    }
}

// Source: components/viz/service/display/surface_aggregator.h:341-477
impl SurfaceAggregator {
    pub fn SetFullDamageForSurface(&mut self, id: CoreSurfaceId) {
        self.full_damage_.insert(id);
    }

    pub fn SetOutputIsSecure(&mut self, secure: bool) {
        self.output_is_secure_ = secure;
    }

    pub fn SetTakeCopyRequests(&mut self, value: bool) {
        self.take_copy_requests_ = value;
    }

    pub fn SetNeedsSurfaceDamageRectList(&mut self, value: bool) {
        self.needs_surface_damage_rect_list_ = value;
    }

    pub fn SetExtraPassForReadbackOption(&mut self, value: ExtraPassForReadbackOption) {
        self.extra_pass_for_readback_option_ = value;
    }

    pub fn SetPreventMergingSurfacesToRootPass(&mut self, value: bool) {
        self.prevent_merging_surfaces_to_root_pass_ = value;
    }

    pub fn SetMaxRenderTargetSize(&mut self, max_size: i32) {
        assert!(max_size >= 0);
        self.max_render_target_size_ = max_size;
    }

    pub fn root_damage_rect(&self) -> Rect {
        self.root_damage_rect_
    }

    pub fn CheckForDisplayDamage(&self, id: CoreSurfaceId) -> bool {
        self.damage_ranges_
            .get(&id.frame_sink_id())
            .is_some_and(|ranges| ranges.iter().any(|range| range.IsInRangeInclusive(id)))
            || self.previous_contained_surfaces_.contains(&id)
    }

    pub fn OnSurfaceDestroyed(&mut self, id: CoreSurfaceId) {
        assert!(!self.is_inside_aggregate_);
        self.resolved_frames_.remove(&id);
        self.full_damage_.remove(&id);
    }
}

// Source: components/viz/service/display/surface_aggregator.cc:1-146
// Chromium's MaskFilterInfoExt decides whether a child root pass may merge.
// Native CPU output retains mask/filter state in each SharedQuadState; the
// renderer consumes the copied state from the owned aggregate.
fn CanPotentiallyMergePass(opacity: f32, allow_merge: bool) -> bool {
    const OPACITY_EPSILON: f32 = 0.001;
    allow_merge && (opacity - 1.0).abs() <= OPACITY_EPSILON
}

// Source: components/viz/service/display/surface_aggregator.cc:147-267
fn CalculateClipRect(surface_clip: Option<Rect>, quad_clip: Option<Rect>) -> Option<Rect> {
    match (surface_clip, quad_clip) {
        (Some(mut parent), Some(child)) => {
            parent.Intersect(child);
            Some(parent)
        }
        (Some(parent), None) => Some(parent),
        (None, Some(child)) => Some(child),
        (None, None) => None,
    }
}

fn ComputeDrawableRectForQuad<L>(quad: &compositor_core::runtime::frame::DrawQuad<L>) -> Rect {
    let mut rect = quad.geometry.visible_rect;
    rect.Intersect(quad.geometry.rect);
    rect
}

fn MapRectToDestination(source: Rect, destination: Rect, mut damage: Rect) -> Rect {
    damage.Intersect(source);
    if damage.IsEmpty() || source.IsEmpty() || destination.IsEmpty() {
        return Rect::default();
    }
    let relative_left = (damage.x - source.x) as i64;
    let relative_top = (damage.y - source.y) as i64;
    let relative_right = (damage.right() - source.x) as i64;
    let relative_bottom = (damage.bottom() - source.y) as i64;
    let scale_floor = |value: i64, target: i32, extent: i32| -> i32 {
        (value * target as i64 / extent as i64) as i32
    };
    let scale_ceil = |value: i64, target: i32, extent: i32| -> i32 {
        ((value * target as i64 + extent as i64 - 1) / extent as i64) as i32
    };
    let left = destination.x + scale_floor(relative_left, destination.width, source.width);
    let top = destination.y + scale_floor(relative_top, destination.height, source.height);
    let right = destination.x + scale_ceil(relative_right, destination.width, source.width);
    let bottom = destination.y + scale_ceil(relative_bottom, destination.height, source.height);
    let mut mapped = Rect::FromBounds(left, top, right, bottom);
    mapped.Intersect(destination);
    mapped
}

// Source: components/viz/service/display/surface_aggregator.cc:268-423
fn RenderPassNeedsFullDamage(pass: &CompositorRenderPass, merge_state_changed: bool) -> bool {
    pass.cache_render_pass || !pass.copy_requests.is_empty() || merge_state_changed
}

fn SetRenderPassDamageRect(
    pass: &CompositorRenderPass,
    frame_damage: Rect,
    added_damage: Rect,
    force_full: bool,
) -> Rect {
    let mut damage = if force_full {
        pass.output_rect
    } else {
        UnionRect(frame_damage, added_damage)
    };
    damage.Intersect(pass.output_rect);
    damage
}

// Source: components/viz/service/display/surface_aggregator.cc:424-573
impl SurfaceAggregator {
    fn CheckFrameSinksChanged(&self, surface_id: CoreSurfaceId, result: &mut PrewalkResult) {
        result.contained_surfaces.insert(surface_id);
        let sink = surface_id.frame_sink_id();
        result.frame_sinks_changed |= !self.previous_contained_frame_sinks_.contains_key(&sink);
        let local = surface_id.local_surface_id();
        result
            .contained_frame_sinks
            .entry(sink)
            .and_modify(|current| *current = (*current).max(local))
            .or_insert(local);
    }

    fn AddSurfaceDamageToDamageList(&self, damage: Rect, result: &mut PrewalkResult) {
        if self.needs_surface_damage_rect_list_ && !damage.IsEmpty() {
            result.surface_damage_rects.push(damage);
        }
    }

    fn AddDamageRanges(&mut self, ranges: &[SurfaceRange]) {
        for range in ranges {
            self.damage_ranges_
                .entry(range.end().frame_sink_id())
                .or_default()
                .push(range.clone());
            if let Some(start) = range.start() {
                if start.frame_sink_id() != range.end().frame_sink_id() {
                    self.damage_ranges_
                        .entry(start.frame_sink_id())
                        .or_default()
                        .push(range.clone());
                }
            }
        }
    }

    // Overlay damage association is a Renderer capability. The core still
    // produces the ordered surface damage list used by that decision.
    pub fn SurfaceDamageRectListEnabled(&self) -> bool {
        self.needs_surface_damage_rect_list_
    }
}

// Source: components/viz/service/display/surface_aggregator.cc:574-660
impl SurfaceAggregator {
    fn ResolveFrameRecord(
        &mut self,
        id: CoreSurfaceId,
        frame: &crate::service::surfaces::surface::FrameData,
        valid: bool,
    ) {
        let root_pass_id = frame
            .frame
            .render_pass_list
            .last()
            .map_or(RenderPassId::default(), |pass| pass.id);
        let pass_ids = self
            .resolved_frames_
            .get(&id)
            .map(|record| record.pass_ids.clone())
            .unwrap_or_default();
        self.resolved_frames_.insert(
            id,
            ResolvedFrameRecord {
                frame_index: frame.frame_index,
                root_pass_id,
                valid,
                pass_ids,
            },
        );
    }

    pub fn ForceReleaseResourcesIfNeeded(
        &mut self,
        manager: &SurfaceManager,
        id: CoreSurfaceId,
    ) -> bool {
        let Some(surface) = manager.GetSurfaceForId(id) else {
            return false;
        };
        let Some(frame) = surface.active_frame() else {
            return false;
        };
        if frame.frame.resource_list.is_empty() {
            self.resolved_frames_.remove(&id);
        }
        true
    }

    // Source: components/viz/service/display/surface_aggregator.cc:661-750

    fn IsRootSurface(&self, id: CoreSurfaceId) -> bool {
        self.root_surface_id_ == Some(id)
    }

    fn ResetAfterAggregate(&mut self) {
        self.is_inside_aggregate_ = false;
        self.root_surface_id_ = None;
        self.resolved_surface_ranges_.clear();
        self.contained_surfaces_.clear();
        self.contained_frame_sinks_.clear();
    }
}

// Source: components/viz/service/display/surface_aggregator.cc:751-838
// Missing primary Surface ranges fall back through SurfaceManager's allocation
// group lookup. A missing fallback is represented by an absent placement and
// is rejected at the Engine boundary because this CPU path has no synthetic
// SurfaceDrawQuad background color input.
// Source: components/viz/service/display/surface_aggregator.cc:839-996
impl SurfaceAggregator {
    fn PrewalkSurface(
        &mut self,
        manager: &SurfaceManager,
        damage_tracker: &DisplayDamageTracker,
        surface_id: CoreSurfaceId,
        will_draw: bool,
        result: &mut PrewalkResult,
    ) -> Result<Rect, VizError> {
        if !result.referenced_surfaces.insert(surface_id) {
            return Ok(Rect::default());
        }
        self.CheckFrameSinksChanged(surface_id, result);

        let Some(surface) = manager.GetSurfaceForId(surface_id) else {
            result.referenced_surfaces.remove(&surface_id);
            return Ok(Rect::default());
        };
        let Some(frame) = surface.active_frame() else {
            result.referenced_surfaces.remove(&surface_id);
            return Ok(Rect::default());
        };
        let resolved = ResolvedFrameData::new_with_previous(
            surface_id,
            frame,
            self.resolved_frames_
                .get(&surface_id)
                .map_or(0, |record| record.frame_index),
            self.resolved_frames_
                .get(&surface_id)
                .map(|record| record.root_pass_id.0),
        )
        .map_err(VizError::Frame)?;

        let force_full = self.full_damage_.contains(&surface_id)
            || match self.resolved_frames_.get(&surface_id) {
                None => true,
                Some(record) if frame.frame_index == record.frame_index => false,
                Some(record) => frame.frame_index != record.frame_index.wrapping_add(1),
            };
        let root = resolved.root_pass().render_pass();
        let pass_damage: BTreeMap<_, _> = frame
            .frame
            .render_pass_list
            .iter()
            .map(|pass| {
                (
                    pass.id,
                    damage_tracker.Damage(surface_id, pass.id, pass.damage_rect),
                )
            })
            .collect();
        let mut visiting = BTreeSet::new();
        let root_index = frame.frame.render_pass_list.len() - 1;
        let mut surface_damage = self.PrewalkRenderPass(
            manager,
            damage_tracker,
            &frame.frame,
            root_index,
            &pass_damage,
            force_full,
            &mut visiting,
            result,
        )?;
        surface_damage = SetRenderPassDamageRect(
            root,
            surface_damage,
            Rect::default(),
            (!frame.frame_drawn && RenderPassNeedsFullDamage(root, false)) || force_full,
        );
        result
            .pass_damage
            .insert((surface_id, root.id), surface_damage);
        if will_draw {
            result.root_damage.Union(surface_damage);
            self.AddSurfaceDamageToDamageList(surface_damage, result);
        }
        self.AddDamageRanges(&frame.referenced_surfaces);

        // Source: components/viz/service/display/surface_aggregator.cc:997-1118
        for range in &frame.referenced_surfaces {
            let Some(child_id) = self.GetResolvedSurfaceForRange(manager, range) else {
                continue;
            };
            if result.contained_surfaces.contains(&child_id) {
                continue;
            }
            result.undrawn_surfaces.insert(child_id);
            self.PrewalkSurface(manager, damage_tracker, child_id, false, result)?;
        }
        for child_id in surface.active_referenced_surfaces() {
            if result.contained_surfaces.contains(child_id) {
                continue;
            }
            result.undrawn_surfaces.insert(*child_id);
            self.PrewalkSurface(manager, damage_tracker, *child_id, false, result)?;
        }

        self.ResolveFrameRecord(surface_id, frame, resolved.is_valid());
        result.referenced_surfaces.remove(&surface_id);
        Ok(surface_damage)
    }
}

// Source: components/viz/service/display/surface_aggregator.cc:1119-1274
// Default background/gutter emission and readback/display-transform passes need
// owned synthetic quads. They are explicit Renderer capabilities in the current
// CompositorEngine output contract. Max-target and security policy are retained
// above and are available when that output capability is injected.
// Source: components/viz/service/display/surface_aggregator.cc:1275-1430
// CopyQuadsToPass is represented by CopyPassesOwned for the CPU Engine: each
// pass, SharedQuadState and quad is copied in source order and render-pass IDs
// are remapped into the aggregate namespace. Producer resource IDs are also
// imported into Viz's global display namespace; VizOutput owns immutable read
// leases so no producer frame or Surface identity crosses into Renderer.
// Source: components/viz/service/display/surface_aggregator.cc:1431-1515
// CompositorRenderPass dependencies were validated by ResolvedFrameData and
// recursively prewalked above. Stable per-Surface source-ID mappings preserve
// aggregate IDs across frames so Renderer pass backings remain reusable.
// Source: components/viz/service/display/surface_aggregator.cc:1516-1600
// Source: components/viz/service/display/surface_aggregator.cc:1601-1750
// Damage for cached/copy passes is forced to the output rect by
// RenderPassNeedsFullDamage/SetRenderPassDamageRect. Added and removed Surface
// sets are computed by ProcessAddedAndRemovedSurfaces after every aggregation.
// Source: components/viz/service/display/surface_aggregator.cc:1751-1900
// The source recursively propagates damage through SurfaceDrawQuad and
// CompositorRenderPassDrawQuad transforms. Rechrom has the latter in
// QuadMaterial and performs the conservative visible-rect propagation in
// PrewalkRenderPass; placement remains opaque VizOutput presentation data.
// Source: components/viz/service/display/surface_aggregator.cc:1901-2050
// Per-quad damage, cache/copy expansion, root clipping, cycle detection and
// SurfaceRange selection feed the same prewalk state above. Undrawn referenced
// Surfaces are visited for lifecycle/resource tracking without adding pixels to
// root damage.
// Source: components/viz/service/display/surface_aggregator.cc:2051-2146
// CopyUndrawnSurfaces and MarkAndPropagateCopyRequestPasses terminate at the
// Renderer capability boundary. Their graph walk remains in PrewalkSurface, so
// referenced Surface lifetime and contained-set transitions are preserved.

// Source: components/viz/service/display/surface_aggregator.cc:2147-2300
impl SurfaceAggregator {
    pub fn Aggregate(
        &mut self,
        manager: &SurfaceManager,
        damage: &DisplayDamageTracker,
        ids: &BTreeMap<SurfaceId, CoreSurfaceId>,
        output_rect: DeviceRect,
        placements: &[SurfacePlacement],
        display_root: Option<&DisplayRootFrame>,
    ) -> Result<VizOutput, VizError> {
        assert!(
            !self.is_inside_aggregate_,
            "nested SurfaceAggregator::Aggregate"
        );
        self.is_inside_aggregate_ = true;
        self.damage_ranges_.clear();
        self.contained_surfaces_.clear();
        self.contained_frame_sinks_.clear();

        let result =
            self.AggregateImpl(manager, damage, ids, output_rect, placements, display_root);
        match &result {
            Ok(output) => {
                self.previous_output_rect_ = output.output_rect;
                self.previous_display_root_ = display_root.cloned();
                self.previous_root_placements_ = placements
                    .iter()
                    .map(|placement| (placement.surface_id, placement.destination))
                    .collect();
                self.root_damage_rect_ = output
                    .frame
                    .SurfaceDamageRects()
                    .iter()
                    .copied()
                    .fold(Rect::default(), UnionRect);
                self.ProcessAddedAndRemovedSurfaces();
                self.previous_contained_surfaces_ = self.contained_surfaces_.clone();
                self.previous_contained_frame_sinks_ = self.contained_frame_sinks_.clone();
                for id in &self.contained_surfaces_ {
                    self.full_damage_.remove(id);
                }
            }
            Err(_) => {
                self.root_damage_rect_ = Rect::default();
            }
        }
        self.ResetAfterAggregate();
        result
    }

    fn AggregateImpl(
        &mut self,
        manager: &SurfaceManager,
        damage: &DisplayDamageTracker,
        ids: &BTreeMap<SurfaceId, CoreSurfaceId>,
        output_rect: DeviceRect,
        placements: &[SurfacePlacement],
        display_root: Option<&DisplayRootFrame>,
    ) -> Result<VizOutput, VizError> {
        // Added, removed, and moved root Surfaces expose pixels that were
        // covered by the previous aggregate. Chromium forces damage for these
        // presentation changes so a retained output buffer redraws the root
        // underneath a dismissed popup.
        let current_root_placements: BTreeMap<_, _> = placements
            .iter()
            .map(|placement| (placement.surface_id, placement.destination))
            .collect();
        let mut placement_damage = Rect::default();
        for (&surface, &destination) in &self.previous_root_placements_ {
            if current_root_placements.get(&surface) != Some(&destination) {
                placement_damage.Union(destination);
            }
        }
        for (&surface, &destination) in &current_root_placements {
            if self.previous_root_placements_.get(&surface) != Some(&destination) {
                placement_damage.Union(destination);
            }
        }
        if self.previous_display_root_.as_ref() != display_root {
            // Browser-root solid quads are retained output content just like
            // child Surface placements. A theme/geometry change must replace
            // their old pixels even when every child Surface is unchanged.
            placement_damage.Union(output_rect);
        }
        let mut placed = BTreeSet::new();
        let mut passes = Vec::new();
        let mut resources = BTreeMap::new();
        let mut roots = Vec::new();
        let mut receipt = DrawReceipt::default();
        let core_to_public: BTreeMap<_, _> =
            ids.iter().map(|(public, core)| (*core, *public)).collect();
        let mut copied_surfaces = BTreeSet::new();
        let mut copy_stack = BTreeSet::new();
        let mut prewalk = PrewalkResult::default();
        let mut frame = AggregatedFrame::default();
        let mut display_background_color = None;

        for placement in placements {
            if placement.destination.IsEmpty() {
                return Err(VizError::InvalidPlacement);
            }
            if !placed.insert(placement.surface_id) {
                return Err(VizError::DuplicateSurface);
            }
            let core = *ids
                .get(&placement.surface_id)
                .ok_or(VizError::InvalidSurface)?;
            self.root_surface_id_.get_or_insert(core);
            let source = manager
                .GetSurfaceForId(core)
                .and_then(|surface| surface.active_frame())
                .ok_or(VizError::InvalidSurface)?;
            display_background_color.get_or_insert(source.frame.metadata.root_background_color);
            frame
                .latency_info
                .extend(source.frame.metadata.latency_info.iter().cloned());
            frame.content_color_usage = frame
                .content_color_usage
                .max(source.frame.metadata.content_color_usage);
            if source.frame.metadata.delegated_ink_metadata.is_some() {
                frame.delegated_ink_metadata = source.frame.metadata.delegated_ink_metadata.clone();
            }
            for (feature, rects) in &source.frame.metadata.tracked_element_rects.0 {
                frame
                    .tracked_element_rects
                    .0
                    .entry(*feature)
                    .or_default()
                    .extend(rects.iter().cloned());
            }
            frame.has_copy_requests |= source
                .frame
                .render_pass_list
                .iter()
                .any(|pass| !pass.copy_requests.is_empty());
            self.HandleSurfacePlacement(
                manager,
                damage,
                placement.surface_id,
                core,
                placement.destination,
                &mut prewalk,
                &mut passes,
                &mut resources,
                &mut roots,
                &mut receipt,
                &core_to_public,
                &mut copied_surfaces,
                &mut copy_stack,
            )?;
        }

        if !placement_damage.IsEmpty() {
            if !prewalk
                .surface_damage_rects
                .iter()
                .any(|damage| damage.Contains(placement_damage))
            {
                self.AddSurfaceDamageToDamageList(placement_damage, &mut prewalk);
            }
            prewalk.root_damage.Union(placement_damage);
            for root in &roots {
                let mut overlap = root.destination;
                overlap.Intersect(placement_damage);
                if overlap.IsEmpty() {
                    continue;
                }
                if let Some(pass) = passes.iter_mut().find(|pass| pass.id == root.pass_id) {
                    pass.damage_rect = pass.output_rect;
                    pass.has_damage_from_contributing_content = true;
                }
            }
        }

        self.contained_surfaces_ = prewalk.contained_surfaces.clone();
        self.contained_frame_sinks_ = prewalk.contained_frame_sinks.clone();
        // surface_damage_rect_list_ describes contributing Surface damage for
        // Viz decisions such as overlay damage. It can be larger than the
        // pixels DirectRenderer writes into the final target because child
        // render passes have their own backing stores. Rotating software
        // output buffers must preserve everything outside the mapped damage of
        // the root passes that are actually drawn into the host target.
        let mut output_damage_rects: Vec<_> = roots
            .iter()
            .filter_map(|root| {
                let pass = passes.iter().find(|pass| pass.id == root.pass_id)?;
                let damage =
                    MapRectToDestination(pass.output_rect, root.destination, pass.damage_rect);
                (!damage.IsEmpty()).then_some(damage)
            })
            .collect();
        if !placement_damage.IsEmpty() {
            // A moved or previous-size fallback root can expose display
            // background outside every current child destination. Preserve
            // that old coverage as display-root damage.
            let mut display_damage = placement_damage;
            for rect in &output_damage_rects {
                display_damage.Union(*rect);
            }
            display_damage.Intersect(output_rect);
            output_damage_rects.clear();
            if !display_damage.IsEmpty() {
                output_damage_rects.push(display_damage);
            }
        }
        // Output growth exposes pixels outside every fallback Surface. That
        // region is display-root damage even though no child root contains it.
        if !self.previous_output_rect_.IsEmpty() && self.previous_output_rect_ != output_rect {
            output_damage_rects.clear();
            output_damage_rects.push(output_rect);
        }
        let background_color = display_root.map_or_else(
            || display_background_color.expect("non-empty placements have an active frame"),
            |root| root.background_color,
        );
        let solid_color_quads =
            display_root.map_or(&[][..], |root| root.solid_color_quads.as_slice());
        self.MaterializeDisplayRoot(
            output_rect,
            frame.has_copy_requests,
            &mut passes,
            &mut roots,
            &output_damage_rects,
            frame.content_color_usage,
            background_color,
            solid_color_quads,
        );
        frame.render_pass_list = passes;
        frame.surface_damage_rect_list_ = prewalk.surface_damage_rects;
        Ok(VizOutput {
            frame,
            resources,
            receipt,
            roots,
            output_rect,
            output_damage_rects,
        })
    }
}

// Source: components/viz/service/display/surface_aggregator.cc:2301-2440
impl SurfaceAggregator {
    pub fn HasFrameAnnotator(&self) -> bool {
        false
    }

    pub fn OutputIsSecure(&self) -> bool {
        self.output_is_secure_
    }

    pub fn TakeCopyRequests(&self) -> bool {
        self.take_copy_requests_
    }

    pub fn MaxRenderTargetSize(&self) -> i32 {
        self.max_render_target_size_
    }

    pub fn ExtraPassOption(&self) -> ExtraPassForReadbackOption {
        self.extra_pass_for_readback_option_
    }

    pub fn PreventMergingSurfacesToRootPass(&self) -> bool {
        self.prevent_merging_surfaces_to_root_pass_
    }
}

// Source: components/viz/service/display/surface_aggregator.cc:2441-2503
// Delegated ink metadata and tracked-element rectangles require geometry types
// that belong to the display/overlay adapter. The aggregation core exposes them
// as ExternalCapability values and never reads platform or GPU state directly.
#[allow(dead_code)]
fn _source_equivalent_helpers_are_linked() {
    let _ = CanPotentiallyMergePass(1.0, true);
    let _ = CalculateClipRect(None, None);
    let _ = ExternalCapability::DelegatedInk;
    let _ = ComputeDrawableRectForQuad::<()>
        as fn(&compositor_core::runtime::frame::DrawQuad<()>) -> Rect;
}

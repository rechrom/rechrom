//! Surface retention and aggregation for the source-aligned compositor frame.
//!
//! Viz retains the immutable frame submitted by each producer. Aggregation
//! lends render passes and resources directly from those owners, adding only
//! the display placement and the surface namespace needed to disambiguate pass
//! and resource identities. No PaintArtifact, layer tree, tiling or pixel data
//! is copied into this boundary.

use compositor_core::{
    runtime::{
        frame::{DrawQuad, QuadMaterial, RenderPassId, SharedQuadState},
        geometry::Transform,
        metadata::CompositorFrameMetadata,
        resources::{CpuReturnedResource, CpuTransferableResource},
        skia::SkRegion,
    },
    NativeCompositorFrame, NativeCompositorRenderPass, NativeCompositorTypes,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    error::Error,
    fmt,
};

pub type DeviceRect = compositor_core::adapter::geometry::Rect;
pub type NativeTransform = Transform<SkRegion>;
pub type NativeSharedQuadState = SharedQuadState<NativeTransform>;
pub type NativeMetadata = CompositorFrameMetadata<NativeCompositorTypes>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SwapId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePlacement {
    pub surface_id: SurfaceId,
    /// Destination in the display's device-pixel coordinate space.
    pub destination: DeviceRect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VizError {
    InvalidSurface,
    DuplicateSurface,
    DuplicateRenderPass,
    DuplicateResource,
    MissingRenderPass,
    MissingSharedQuadState,
    MissingResource,
    InvalidPlacement,
    DuplicateSwap,
}

impl fmt::Display for VizError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidSurface => "invalid Viz surface id",
            Self::DuplicateSurface => "surface is placed more than once",
            Self::DuplicateRenderPass => "compositor frame has duplicate render-pass ids",
            Self::DuplicateResource => "compositor frame has duplicate resource ids",
            Self::MissingRenderPass => "render-pass quad references a missing pass",
            Self::MissingSharedQuadState => "draw quad references a missing shared quad state",
            Self::MissingResource => "draw quad references a missing resource",
            Self::InvalidPlacement => "surface placement is empty",
            Self::DuplicateSwap => "swap id is already pending",
        })
    }
}

impl Error for VizError {}

struct Surface {
    frame: NativeCompositorFrame,
    submission: u64,
    producer_token: u32,
    damage_start_token: Option<u32>,
    damage_continuous: bool,
    pending_damage: BTreeMap<RenderPassId, DeviceRect>,
}

/// One render pass in the aggregate. The producer frame remains resident in
/// `VizEngine` for the complete lifetime of this view.
#[derive(Clone, Copy)]
pub struct AggregatedRenderPass<'a> {
    surface_id: SurfaceId,
    submission: u64,
    destination: DeviceRect,
    is_root: bool,
    metadata: &'a NativeMetadata,
    pass: &'a NativeCompositorRenderPass,
    resources: &'a [CpuTransferableResource],
    damage_rect: DeviceRect,
    damage_start_token: Option<u32>,
    damage_continuous: bool,
}

impl<'a> AggregatedRenderPass<'a> {
    pub fn Surface(&self) -> SurfaceId {
        self.surface_id
    }
    pub fn Submission(&self) -> u64 {
        self.submission
    }
    pub fn Destination(&self) -> DeviceRect {
        self.destination
    }
    pub fn IsRoot(&self) -> bool {
        self.is_root
    }
    pub fn Metadata(&self) -> &'a NativeMetadata {
        self.metadata
    }
    pub fn RenderPass(&self) -> &'a NativeCompositorRenderPass {
        self.pass
    }
    pub fn RenderPassId(&self) -> RenderPassId {
        self.pass.id
    }
    pub fn SharedQuadStates(&self) -> &'a [NativeSharedQuadState] {
        &self.pass.shared_quad_state_list
    }
    pub fn Quads(&self) -> &'a [DrawQuad<<NativeCompositorTypes as compositor_core::runtime::frame::FrameValueTypes>::PictureDisplayItemList>]{
        &self.pass.quad_list
    }
    pub fn Resources(&self) -> &'a [CpuTransferableResource] {
        self.resources
    }
    pub fn DamageRect(&self) -> DeviceRect {
        self.damage_rect
    }
    pub fn DamageStartToken(&self) -> Option<u32> {
        self.damage_start_token
    }
    pub fn DamageIsContinuous(&self) -> bool {
        self.damage_continuous
    }
}

pub struct AggregatedFrame<'a> {
    output_rect: DeviceRect,
    passes: Vec<AggregatedRenderPass<'a>>,
    receipt: DrawReceipt,
}

impl<'a> AggregatedFrame<'a> {
    pub fn OutputRect(&self) -> DeviceRect {
        self.output_rect
    }
    pub fn RenderPasses(&self) -> &[AggregatedRenderPass<'a>] {
        &self.passes
    }
    pub fn Receipt(&self) -> DrawReceipt {
        self.receipt.clone()
    }
}

/// Acknowledges only the exact surface submissions that Renderer completed.
/// If an embedding chooses not to draw an aggregate, it simply drops this
/// value and pending damage remains live in Viz.
#[derive(Clone, Debug, Default)]
pub struct DrawReceipt(Vec<(SurfaceId, u64)>);

pub struct VizEngine {
    surfaces: BTreeMap<SurfaceId, Surface>,
    next_submission: u64,
    pending_swaps: BTreeSet<SwapId>,
    max_pending_swaps: usize,
}

/// Result of accepting one producer frame. Resources from the replaced frame
/// are returned only after Viz has dropped that frame's last surface owner.
/// The embedding routes these typed completions to the same producer without
/// gaining access to its resource pool, layers, tiles, or raster scheduler.
pub struct FrameSubmission {
    pub submission: u64,
    pub returned_resources: Vec<CpuReturnedResource>,
}

impl Default for VizEngine {
    fn default() -> Self {
        Self {
            surfaces: BTreeMap::new(),
            next_submission: 0,
            pending_swaps: BTreeSet::new(),
            max_pending_swaps: 1,
        }
    }
}

impl VizEngine {
    pub fn New() -> Self {
        Self::default()
    }

    /// Replaces one producer's active frame. The move-only native frame stays
    /// owned by Viz until replacement or explicit surface destruction.
    pub fn SubmitFrame(
        &mut self,
        surface_id: SurfaceId,
        frame: NativeCompositorFrame,
    ) -> Result<FrameSubmission, VizError> {
        if surface_id.0 == 0 {
            return Err(VizError::InvalidSurface);
        }
        validate_frame(&frame)?;
        self.next_submission = self.next_submission.wrapping_add(1).max(1);
        let submission = self.next_submission;
        let producer_token = frame.metadata.frame_token;
        let previous = self.surfaces.get(&surface_id);
        let damage_start_token = previous
            .and_then(|surface| surface.damage_start_token)
            .or(Some(producer_token));
        let damage_continuous = previous.is_none_or(|surface| {
            surface.damage_continuous
                && surface.producer_token.checked_add(1) == Some(producer_token)
        });
        let mut pending_damage = BTreeMap::new();
        for pass in &frame.render_pass_list {
            let damage = previous
                .and_then(|surface| {
                    let old = surface
                        .frame
                        .render_pass_list
                        .iter()
                        .find(|old| old.id == pass.id)?;
                    Some(if old.output_rect == pass.output_rect {
                        union_rect(
                            surface
                                .pending_damage
                                .get(&pass.id)
                                .copied()
                                .unwrap_or_default(),
                            pass.damage_rect,
                        )
                    } else {
                        pass.output_rect
                    })
                })
                .unwrap_or(pass.damage_rect);
            pending_damage.insert(pass.id, damage);
        }
        let replaced = self.surfaces.insert(
            surface_id,
            Surface {
                frame,
                submission,
                producer_token,
                damage_start_token,
                damage_continuous,
                pending_damage,
            },
        );
        Ok(FrameSubmission {
            submission,
            returned_resources: replaced.map_or_else(Vec::new, return_frame_resources),
        })
    }

    pub fn Aggregate<'a>(
        &'a self,
        output_rect: DeviceRect,
        placements: &[SurfacePlacement],
    ) -> Result<AggregatedFrame<'a>, VizError> {
        let mut placed = BTreeMap::new();
        let mut passes = Vec::new();
        let mut receipt = DrawReceipt::default();
        for placement in placements {
            // A surface may be partially or completely outside the display;
            // clipping it is a renderer/aggregate operation. Empty placements
            // cannot contribute and indicate a broken embedding contract.
            if placement.destination.IsEmpty() {
                return Err(VizError::InvalidPlacement);
            }
            if placed.insert(placement.surface_id, ()).is_some() {
                return Err(VizError::DuplicateSurface);
            }
            let Some(surface) = self.surfaces.get(&placement.surface_id) else {
                return Err(VizError::InvalidSurface);
            };
            receipt.0.push((placement.surface_id, surface.submission));
            let root_index = surface.frame.render_pass_list.len().checked_sub(1);
            for (index, pass) in surface.frame.render_pass_list.iter().enumerate() {
                passes.push(AggregatedRenderPass {
                    surface_id: placement.surface_id,
                    submission: surface.submission,
                    destination: placement.destination,
                    is_root: Some(index) == root_index,
                    metadata: &surface.frame.metadata,
                    pass,
                    resources: &surface.frame.resource_list,
                    damage_rect: surface
                        .pending_damage
                        .get(&pass.id)
                        .copied()
                        .unwrap_or(pass.damage_rect),
                    damage_start_token: surface.damage_start_token,
                    damage_continuous: surface.damage_continuous,
                });
            }
        }
        Ok(AggregatedFrame {
            output_rect,
            passes,
            receipt,
        })
    }

    /// Clears accumulated damage after successful rendering. A stale receipt
    /// cannot clear damage belonging to a replacement submission.
    pub fn AcknowledgeDraw(&mut self, receipt: DrawReceipt) {
        for (surface_id, submission) in receipt.0 {
            let Some(surface) = self.surfaces.get_mut(&surface_id) else {
                continue;
            };
            if surface.submission != submission {
                continue;
            }
            surface.damage_start_token = None;
            surface.damage_continuous = true;
            for damage in surface.pending_damage.values_mut() {
                *damage = DeviceRect::default();
            }
        }
    }

    /// Removes one producer surface and returns every resource still imported
    /// by its active frame. The caller must route the values to that producer.
    pub fn DestroySurface(&mut self, surface_id: SurfaceId) -> Option<Vec<CpuReturnedResource>> {
        self.surfaces
            .remove(&surface_id)
            .map(return_frame_resources)
    }

    pub fn HasActiveFrame(&self, surface_id: SurfaceId) -> bool {
        self.surfaces.contains_key(&surface_id)
    }

    pub fn CanDrawAndSwap(&self) -> bool {
        self.pending_swaps.len() < self.max_pending_swaps.max(1)
    }

    pub fn DidSubmitSwap(&mut self, swap_id: SwapId) -> Result<(), VizError> {
        if !self.pending_swaps.insert(swap_id) {
            return Err(VizError::DuplicateSwap);
        }
        Ok(())
    }

    pub fn DidReceiveSwapAck(&mut self, swap_id: SwapId) -> bool {
        self.pending_swaps.remove(&swap_id)
    }
}

fn return_frame_resources(surface: Surface) -> Vec<CpuReturnedResource> {
    surface
        .frame
        .resource_list
        .iter()
        .map(CpuTransferableResource::ToReturnedResource)
        .collect()
}

fn union_rect(lhs: DeviceRect, rhs: DeviceRect) -> DeviceRect {
    if lhs.IsEmpty() {
        return rhs;
    }
    if rhs.IsEmpty() {
        return lhs;
    }
    DeviceRect::FromBounds(
        lhs.x.min(rhs.x),
        lhs.y.min(rhs.y),
        lhs.right().max(rhs.right()),
        lhs.bottom().max(rhs.bottom()),
    )
}

fn validate_frame(frame: &NativeCompositorFrame) -> Result<(), VizError> {
    if frame.render_pass_list.is_empty() {
        return Err(VizError::MissingRenderPass);
    }
    let mut pass_ids = BTreeMap::new();
    for pass in &frame.render_pass_list {
        if pass_ids.insert(pass.id, ()).is_some() {
            return Err(VizError::DuplicateRenderPass);
        }
    }
    let mut resources = HashSet::new();
    for resource in &frame.resource_list {
        if !resources.insert(resource.id) {
            return Err(VizError::DuplicateResource);
        }
    }
    for pass in &frame.render_pass_list {
        for quad in &pass.quad_list {
            let Some(handle) = quad.geometry.shared_quad_state else {
                return Err(VizError::MissingSharedQuadState);
            };
            if pass.TrySharedQuadState(handle).is_none() {
                return Err(VizError::MissingSharedQuadState);
            }
            match quad.material {
                QuadMaterial::Tile { resource_id, .. } if !resources.contains(&resource_id) => {
                    return Err(VizError::MissingResource);
                }
                QuadMaterial::CompositorRenderPass {
                    render_pass_id,
                    mask_resource_id,
                    ..
                } => {
                    if !pass_ids.contains_key(&render_pass_id) {
                        return Err(VizError::MissingRenderPass);
                    }
                    if mask_resource_id.0 != 0 && !resources.contains(&mask_resource_id) {
                        return Err(VizError::MissingResource);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

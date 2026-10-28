//! Rechrom public adapter over the source-relative Viz owners.
#![allow(non_snake_case)]

use crate::{
    common::surfaces::{
        frame_sink_id::FrameSinkId as CoreFrameSinkId,
        local_surface_id::{EmbedToken, LocalSurfaceId as CoreLocalSurfaceId},
        surface_id::SurfaceId as CoreSurfaceId,
        surface_info::SurfaceInfo,
        surface_range::SurfaceRange,
    },
    service::{
        display::{
            display_damage_tracker::DisplayDamageTracker,
            resolved_frame_data::{FrameValidationError, ValidateFrame},
            surface_aggregator::SurfaceAggregator,
        },
        surfaces::{
            surface::{FrameData, QueueFrameResult, Surface},
            surface_manager::SurfaceManager,
        },
    },
};
use compositor_core::runtime::resources::RasterReturnedResource;
use compositor_core::{
    adapter::layer_tree_impl::ResourceId, runtime::frame::RenderPassId, NativeCompositorFrame,
    NativeRasterResourceLease,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    sync::Arc,
};

pub type DeviceRect = compositor_core::adapter::geometry::Rect;
pub use crate::common::quads::aggregated_render_pass::{
    AggregatedRenderPass, NativeSharedQuadState, NativeTransform,
};
pub use crate::service::display::aggregated_frame::AggregatedFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SwapId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePlacement {
    pub surface_id: SurfaceId,
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
    Frame(FrameValidationError),
}
impl fmt::Display for VizError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{self:?}")
    }
}
impl Error for VizError {}
pub struct FrameSubmission {
    pub submission: u64,
    pub returned_resources: Vec<RasterReturnedResource>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DrawReceipt(pub(crate) Vec<(SurfaceId, u64)>);

#[derive(Clone, Copy)]
pub(crate) struct RootPresentation {
    pub(crate) pass_id: u64,
    pub(crate) destination: DeviceRect,
}

/// The sole Viz-to-Renderer interface. Chromium's AggregatedFrame remains a
/// pure display value; resource resolution and draw acknowledgement stay
/// opaque capabilities owned by this Engine output transaction.
pub struct VizOutput {
    pub(crate) frame: AggregatedFrame,
    pub(crate) resources: BTreeMap<u32, Arc<NativeRasterResourceLease>>,
    pub(crate) receipt: DrawReceipt,
    pub(crate) roots: Vec<RootPresentation>,
    pub(crate) output_rect: DeviceRect,
}
impl VizOutput {
    pub fn Frame(&self) -> &AggregatedFrame {
        &self.frame
    }
    /// Display-stage mutation performed after aggregation and before drawing.
    /// Chromium's Display owns this interval and uses it for occlusion culling.
    pub fn FrameMut(&mut self) -> &mut AggregatedFrame {
        &mut self.frame
    }
    pub fn Resource(&self, id: ResourceId) -> Option<Arc<NativeRasterResourceLease>> {
        self.resources.get(&id.0).cloned()
    }
    pub fn RootDestination(&self, id: RenderPassId) -> Option<DeviceRect> {
        self.roots
            .iter()
            .find(|root| root.pass_id == id.0)
            .map(|root| root.destination)
    }
    pub fn OutputRect(&self) -> DeviceRect {
        self.output_rect
    }
    pub fn DamagePixels(&self) -> usize {
        self.frame
            .RenderPasses()
            .iter()
            .map(|pass| {
                let damage = pass.DamageRect();
                damage.width.max(0) as usize * damage.height.max(0) as usize
            })
            .sum()
    }
}

pub struct VizEngine {
    manager: SurfaceManager,
    aggregator: SurfaceAggregator,
    damage: DisplayDamageTracker,
    ids: BTreeMap<SurfaceId, CoreSurfaceId>,
    next_submission: u64,
    pending_swaps: BTreeSet<SwapId>,
    max_pending_swaps: usize,
}
impl Default for VizEngine {
    fn default() -> Self {
        Self {
            manager: SurfaceManager::new(),
            aggregator: SurfaceAggregator::new(),
            damage: Default::default(),
            ids: BTreeMap::new(),
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
    fn CoreId(id: SurfaceId) -> CoreSurfaceId {
        let low = id.0 as u32;
        let high = (id.0 >> 32) as u32;
        CoreSurfaceId::new(
            CoreFrameSinkId::new(high.max(1), low.max(1)),
            CoreLocalSurfaceId::new(1, 1, EmbedToken::new(0x7265_6368_726f_6d00, id.0.max(1))),
        )
    }
    pub fn SubmitFrame(
        &mut self,
        id: SurfaceId,
        frame: NativeCompositorFrame,
    ) -> Result<FrameSubmission, VizError> {
        if id.0 == 0 {
            return Err(VizError::InvalidSurface);
        }
        ValidateFrame(&frame).map_err(VizError::Frame)?;
        let core = *self.ids.entry(id).or_insert_with(|| Self::CoreId(id));
        if self.manager.GetSurfaceForId(core).is_none() {
            let root = frame
                .render_pass_list
                .last()
                .ok_or(VizError::MissingRenderPass)?;
            let size = compositor_core::adapter::geometry::Size::new(
                root.output_rect.width,
                root.output_rect.height,
            );
            let info = SurfaceInfo::new(
                core,
                frame.metadata.device_scale_factor.max(f32::MIN_POSITIVE),
                size,
            );
            self.manager.CreateSurface(info, 0).map_err(|error| {
                eprintln!(
                    "viz-surface-error operation=create public_id={} core_id={} error={error}",
                    id.0, core
                );
                VizError::InvalidSurface
            })?;
            if self.manager.GetRootSurfaceId().is_none() {
                self.manager.SetRootSurfaceId(core);
                self.damage.SetNewRootSurface(core);
            }
        }
        self.next_submission = self.next_submission.wrapping_add(1).max(1);
        let submission = self.next_submission;
        let activation_dependencies: BTreeSet<CoreSurfaceId> = frame
            .metadata
            .activation_dependencies
            .iter()
            .map(|id| {
                CoreSurfaceId::new(
                    CoreFrameSinkId::new(id.frame_sink_id.client_id, id.frame_sink_id.sink_id),
                    CoreLocalSurfaceId::new(
                        id.local_surface_id.parent_sequence_number,
                        id.local_surface_id.child_sequence_number,
                        EmbedToken::new(
                            id.local_surface_id.embed_token[0],
                            id.local_surface_id.embed_token[1],
                        ),
                    ),
                )
            })
            .collect();
        let damage_state = {
            let previous = self
                .manager
                .GetSurfaceForId(core)
                .and_then(Surface::active_frame)
                .map(|f| &f.frame);
            self.damage.UpdateFrame(core, &frame, previous)
        };
        let referenced_surfaces = frame
            .metadata
            .referenced_surfaces
            .iter()
            .map(|range| {
                let cv = |id: &compositor_core::runtime::types::SurfaceId| {
                    CoreSurfaceId::new(
                        CoreFrameSinkId::new(id.frame_sink_id.client_id, id.frame_sink_id.sink_id),
                        CoreLocalSurfaceId::new(
                            id.local_surface_id.parent_sequence_number,
                            id.local_surface_id.child_sequence_number,
                            EmbedToken::new(
                                id.local_surface_id.embed_token[0],
                                id.local_surface_id.embed_token[1],
                            ),
                        ),
                    )
                };
                SurfaceRange::new(range.start.as_ref().map(cv), cv(&range.end))
            })
            .collect();
        let data = FrameData {
            frame,
            frame_index: submission as u32,
            submission,
            producer_token: damage_state.producer_token,
            damage_start_token: damage_state.damage_start_token,
            damage_continuous: damage_state.continuous,
            pending_damage: damage_state.pass_damage.clone(),
            activation_dependencies,
            referenced_surfaces,
            acknowledged: false,
            frame_drawn: false,
            will_be_notified_of_presentation: false,
        };
        let result = self
            .manager
            .GetSurfaceForIdMutable(core)
            .expect("surface created")
            .QueueFrame(data);
        if result == QueueFrameResult::REJECTED {
            let info = self
                .manager
                .GetSurfaceForId(core)
                .map(|surface| surface.surface_info().ToString())
                .unwrap_or_else(|| "missing".into());
            eprintln!(
                "viz-surface-error operation=queue public_id={} core_id={} surface={info}",
                id.0, core
            );
            return Err(VizError::InvalidSurface);
        }
        if result == QueueFrameResult::ACCEPTED_ACTIVE {
            self.manager.SurfaceActivated(core);
        }
        let returned_resources = self
            .manager
            .GetSurfaceForIdMutable(core)
            .expect("surface created")
            .TakeReturnedResources();
        Ok(FrameSubmission {
            submission,
            returned_resources,
        })
    }
    pub fn Aggregate(
        &mut self,
        output: DeviceRect,
        placements: &[SurfacePlacement],
    ) -> Result<VizOutput, VizError> {
        self.aggregator
            .Aggregate(&self.manager, &self.damage, &self.ids, output, placements)
    }
    pub fn DidDraw(&mut self, output: VizOutput) {
        for (id, submission) in output.receipt.0 {
            let Some(core) = self.ids.get(&id).copied() else {
                continue;
            };
            if self
                .manager
                .GetSurfaceForId(core)
                .and_then(Surface::active_frame)
                .is_some_and(|f| f.submission == submission)
            {
                self.damage.DidFinishFrame(core);
                if let Some(surface) = self.manager.GetSurfaceForIdMutable(core) {
                    surface.MarkAsDrawn();
                }
            }
        }
    }
    pub fn DestroySurface(&mut self, id: SurfaceId) -> Option<Vec<RasterReturnedResource>> {
        let core = self.ids.remove(&id)?;
        self.damage.OnSurfaceMarkedForDestruction(core);
        let mut surface = self.manager.DestroySurface(core)?;
        surface.Close();
        Some(surface.TakeReturnedResources())
    }
    pub fn HasActiveFrame(&self, id: SurfaceId) -> bool {
        self.ids
            .get(&id)
            .and_then(|id| self.manager.GetSurfaceForId(*id))
            .is_some_and(Surface::has_active_frame)
    }
    pub fn CanDrawAndSwap(&self) -> bool {
        self.pending_swaps.len() < self.max_pending_swaps.max(1)
    }
    pub fn DidSubmitSwap(&mut self, id: SwapId) -> Result<(), VizError> {
        if !self.pending_swaps.insert(id) {
            Err(VizError::DuplicateSwap)
        } else {
            Ok(())
        }
    }
    pub fn DidReceiveSwapAck(&mut self, id: SwapId) -> bool {
        self.pending_swaps.remove(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compositor_core::runtime::frame::RenderPassId;

    fn frame(token: u32, damage: DeviceRect, contributing_damage: bool) -> NativeCompositorFrame {
        let rect = DeviceRect::new(0, 0, 10, 10);
        let mut pass = compositor_core::NativeCompositorRenderPass::default();
        pass.id = RenderPassId(1);
        pass.output_rect = rect;
        pass.damage_rect = damage;
        pass.has_damage_from_contributing_content = contributing_damage;
        let mut frame = NativeCompositorFrame::default();
        frame.metadata.frame_token = token;
        frame.metadata.device_scale_factor = 1.0;
        frame.render_pass_list.push(Box::new(pass));
        frame
    }

    #[test]
    fn drawn_surface_damage_is_not_replayed_when_another_surface_updates() {
        let first = SurfaceId(1);
        let second = SurfaceId(2);
        let placement = |surface_id, x| SurfacePlacement {
            surface_id,
            destination: DeviceRect::new(x, 0, 10, 10),
        };
        let mut viz = VizEngine::New();
        let full_damage = DeviceRect::new(0, 0, 10, 10);
        viz.SubmitFrame(first, frame(1, full_damage, true)).unwrap();
        let initial = viz
            .Aggregate(DeviceRect::new(0, 0, 20, 10), &[placement(first, 0)])
            .unwrap();
        assert_eq!(initial.DamagePixels(), 100);
        viz.DidDraw(initial);

        viz.SubmitFrame(second, frame(1, full_damage, true))
            .unwrap();
        let next = viz
            .Aggregate(
                DeviceRect::new(0, 0, 20, 10),
                &[placement(first, 0), placement(second, 10)],
            )
            .unwrap();
        assert_eq!(next.DamagePixels(), 100);
        assert_eq!(
            next.Frame().SurfaceDamageRects(),
            &[DeviceRect::new(10, 0, 10, 10)]
        );
        assert!(!next.Frame().RenderPasses()[0].has_damage_from_contributing_content);
        assert!(next.Frame().RenderPasses()[1].has_damage_from_contributing_content);
    }

    #[test]
    fn contributing_content_flag_preserves_partial_damage() {
        let surface = SurfaceId(1);
        let placement = SurfacePlacement {
            surface_id: surface,
            destination: DeviceRect::new(0, 0, 10, 10),
        };
        let full_damage = DeviceRect::new(0, 0, 10, 10);
        let partial_damage = DeviceRect::new(0, 0, 2, 10);
        let mut viz = VizEngine::New();
        viz.SubmitFrame(surface, frame(1, full_damage, false))
            .unwrap();
        let initial = viz
            .Aggregate(DeviceRect::new(0, 0, 10, 10), &[placement])
            .unwrap();
        viz.DidDraw(initial);

        viz.SubmitFrame(surface, frame(2, partial_damage, true))
            .unwrap();
        let output = viz
            .Aggregate(DeviceRect::new(0, 0, 10, 10), &[placement])
            .unwrap();
        assert_eq!(output.DamagePixels(), 20);
        assert_eq!(output.Frame().SurfaceDamageRects(), &[partial_damage]);
    }

    #[test]
    fn removing_overlaid_root_surface_damages_the_root_below_it() {
        let document = SurfaceId(1);
        let popup = SurfaceId(2);
        let destination = DeviceRect::new(0, 0, 10, 10);
        let placement = |surface_id| SurfacePlacement {
            surface_id,
            destination,
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(document, frame(1, destination, true))
            .unwrap();
        viz.SubmitFrame(popup, frame(1, destination, true))
            .unwrap();
        let initial = viz
            .Aggregate(
                destination,
                &[placement(document), placement(popup)],
            )
            .unwrap();
        viz.DidDraw(initial);

        let output = viz
            .Aggregate(destination, &[placement(document)])
            .unwrap();
        assert_eq!(output.DamagePixels(), 100);
        assert_eq!(output.Frame().SurfaceDamageRects(), &[destination]);
        assert_eq!(output.Frame().RenderPasses()[0].DamageRect(), destination);
    }
}

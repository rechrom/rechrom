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
use compositor_core::runtime::{
    frame::QuadMaterial,
    resources::{RasterReturnedResource, RasterTransferableResource},
    types::{
        FrameSinkId as RuntimeFrameSinkId, LocalSurfaceId as RuntimeLocalSurfaceId,
        SurfaceId as RuntimeSurfaceIdValue, SurfaceRange as RuntimeSurfaceRangeValue,
    },
};
use compositor_core::{
    adapter::layer_tree_impl::ResourceId, runtime::frame::RenderPassId, CompositorFrame,
    RasterResourceLease,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    sync::Arc,
};

pub type DeviceRect = compositor_core::adapter::geometry::Rect;
pub use crate::common::quads::aggregated_render_pass::{
    AggregatedRenderPass, SharedQuadState, Transform,
};
pub use crate::service::display::aggregated_frame::AggregatedFrame;
pub use compositor_core::adapter::layer_tree_impl::Color4f;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SwapId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePlacement {
    pub surface_id: SurfaceId,
    pub destination: DeviceRect,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayRootSolidColorQuad {
    pub destination: DeviceRect,
    pub color: Color4f,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayRootFrame {
    pub background_color: Color4f,
    /// Front-to-back, matching AggregatedRenderPass quad ordering.
    pub solid_color_quads: Vec<DisplayRootSolidColorQuad>,
}

pub(crate) fn CoreIdFromRuntime(id: &RuntimeSurfaceIdValue) -> CoreSurfaceId {
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
}

pub(crate) fn CoreRangeFromRuntime(range: &RuntimeSurfaceRangeValue) -> SurfaceRange {
    SurfaceRange::new(
        range.start.as_ref().map(CoreIdFromRuntime),
        CoreIdFromRuntime(&range.end),
    )
}

fn RuntimeIdFromCore(id: CoreSurfaceId) -> RuntimeSurfaceIdValue {
    let token = id.local_surface_id().embed_token();
    RuntimeSurfaceIdValue {
        frame_sink_id: RuntimeFrameSinkId {
            client_id: id.frame_sink_id().client_id(),
            sink_id: id.frame_sink_id().sink_id(),
        },
        local_surface_id: RuntimeLocalSurfaceId {
            parent_sequence_number: id.local_surface_id().parent_sequence_number(),
            child_sequence_number: id.local_surface_id().child_sequence_number(),
            embed_token: [token.high, token.low],
        },
    }
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
/// The exact source frame named by an aggregation draw receipt. Native
/// presentation feedback must use this identity, not the latest submission.
#[derive(Clone, Copy, Debug)]
pub struct PresentedSurfaceFrame {
    pub surface_id: SurfaceId,
    pub submission: u64,
    pub frame_token: u32,
    pub frame_id: compositor_core::adapter::begin_frame::BeginFrameId,
}
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
    pub(crate) resources: BTreeMap<u32, DisplayResource>,
    pub(crate) receipt: DrawReceipt,
    pub(crate) roots: Vec<RootPresentation>,
    pub(crate) output_rect: DeviceRect,
    pub(crate) output_damage_rects: Vec<DeviceRect>,
}

/// Viz's imported display resource. Chromium's DisplayResourceProvider keeps
/// the complete TransferableResource in ChildResource rather than projecting
/// it immediately to CPU pixels (display_resource_provider.h:250-300). The
/// optional immutable lease pins the current software snapshot; a GPU-backed
/// SharedImage remains present even when no host mapping exists.
pub(crate) struct DisplayResource {
    pub(crate) transferable: RasterTransferableResource,
    pub(crate) read_lease: Option<Arc<RasterResourceLease>>,
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
    pub fn Resource(&self, id: ResourceId) -> Option<Arc<RasterResourceLease>> {
        self.resources.get(&id.0)?.read_lease.clone()
    }
    /// Complete imported resource for GPU renderer/image-context creation.
    /// Renderer packages receive the SharedImage and its synchronization type
    /// without gaining access to the producer's pool, tile or compositor.
    pub fn TransferableResource(&self, id: ResourceId) -> Option<&RasterTransferableResource> {
        self.resources
            .get(&id.0)
            .map(|resource| &resource.transferable)
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
    pub fn OutputDamageRect(&self) -> DeviceRect {
        self.output_damage_rects
            .iter()
            .copied()
            .fold(DeviceRect::default(), |mut total, damage| {
                total.Union(damage);
                total
            })
    }
    pub fn OutputDamageRects(&self) -> &[DeviceRect] {
        &self.output_damage_rects
    }
}

pub struct VizEngine {
    manager: SurfaceManager,
    aggregator: SurfaceAggregator,
    damage: DisplayDamageTracker,
    ids: BTreeMap<SurfaceId, CoreSurfaceId>,
    frame_indices: BTreeMap<SurfaceId, u32>,
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
            frame_indices: BTreeMap::new(),
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
    /// Returns the compositor identity a parent SurfaceDrawQuad must carry for
    /// a child submitted under `id`; embedders must not duplicate Viz's private
    /// CoreSurfaceId construction.
    pub fn RuntimeSurfaceId(id: SurfaceId) -> RuntimeSurfaceIdValue {
        RuntimeIdFromCore(Self::CoreId(id))
    }

    pub fn RuntimeSurfaceRange(id: SurfaceId) -> RuntimeSurfaceRangeValue {
        RuntimeSurfaceRangeValue {
            start: None,
            end: Self::RuntimeSurfaceId(id),
        }
    }
    pub fn SubmitFrame(
        &mut self,
        id: SurfaceId,
        frame: CompositorFrame,
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
        let frame_index = *self
            .frame_indices
            .entry(id)
            .and_modify(|index| *index = index.wrapping_add(1).max(1))
            .or_insert(1);
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
        let activation_dependencies = if frame.metadata.is_handling_interaction
            || (frame.metadata.deadline.deadline_in_frames == 0
                && !frame.metadata.deadline.use_default_lower_bound_deadline)
        {
            self.manager
                .PrepareActivationDependencies(core, std::iter::empty())
        } else {
            self.manager
                .PrepareActivationDependencies(core, activation_dependencies)
        };
        let damage_state = {
            let previous = self
                .manager
                .GetSurfaceForId(core)
                .and_then(Surface::active_frame)
                .map(|f| &f.frame);
            self.damage.UpdateFrame(core, &frame, previous)
        };
        let mut referenced_surfaces: Vec<_> = frame
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
        // A direct producer can submit a SurfaceDrawQuad without passing
        // through LayerTreeHostImpl's metadata collector. The Surface graph
        // still follows every resident quad range.
        for range in frame.render_pass_list.iter().flat_map(|pass| {
            pass.quad_list
                .iter()
                .filter_map(|quad| match &quad.material {
                    QuadMaterial::Surface { surface_range, .. } => {
                        Some(CoreRangeFromRuntime(surface_range))
                    }
                    _ => None,
                })
        }) {
            if !referenced_surfaces.contains(&range) {
                referenced_surfaces.push(range);
            }
        }
        let data = FrameData {
            frame,
            frame_index,
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
        self.aggregator.Aggregate(
            &self.manager,
            &self.damage,
            &self.ids,
            output,
            placements,
            None,
        )
    }
    pub fn AggregateWithDisplayRoot(
        &mut self,
        output: DeviceRect,
        placements: &[SurfacePlacement],
        display_root: &DisplayRootFrame,
    ) -> Result<VizOutput, VizError> {
        self.aggregator.Aggregate(
            &self.manager,
            &self.damage,
            &self.ids,
            output,
            placements,
            Some(display_root),
        )
    }
    pub fn PresentationFrames(&self, output: &VizOutput) -> Vec<PresentedSurfaceFrame> {
        output
            .receipt
            .0
            .iter()
            .filter_map(|(id, submission)| {
                let core = self.ids.get(id)?;
                let active = self.manager.GetSurfaceForId(*core)?.active_frame()?;
                if active.submission != *submission {
                    return None;
                }
                Some(PresentedSurfaceFrame {
                    surface_id: *id,
                    submission: *submission,
                    frame_token: active.frame.metadata.frame_token,
                    frame_id: active.frame.metadata.begin_frame_ack.frame_id,
                })
            })
            .collect()
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
        self.frame_indices.remove(&id);
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
    use compositor_core::{
        adapter::layer_tree_impl::Color4f,
        adapter::resource_pool::{ResourcePoolMemoryDump, SharedImageTracingImportance},
        runtime::frame::{QuadMaterial, RenderPassId, SharedQuadStateHandle},
        runtime::resources::{
            CommandBufferNamespace, RasterDumpGuid, RasterResourceSource, RasterSharedImage,
            RasterSyncToken, RasterSynchronizationType, RasterTransferableResource,
            RasterWriteMapping, SharedImageBackend,
        },
    };
    use std::{io, sync::Arc};

    struct GpuLikeSharedImage {
        allocation: ResourceId,
        token: RasterSyncToken,
    }

    impl SharedImageBackend for GpuLikeSharedImage {
        fn RasterAllocationId(&self) -> ResourceId {
            self.allocation
        }

        fn creation_sync_token(&self) -> RasterSyncToken {
            self.token
        }

        fn EndImport(&self, token: &RasterSyncToken) -> RasterSyncToken {
            *token
        }

        fn UpdateDestructionSyncToken(&self, _token: &RasterSyncToken) {}

        fn OnMemoryDump(
            &self,
            _dump: &mut dyn ResourcePoolMemoryDump<RasterDumpGuid>,
            _guid: &RasterDumpGuid,
            _importance: SharedImageTracingImportance,
        ) {
        }

        fn RasterReadLease(&self) -> Option<Arc<RasterResourceLease>> {
            None
        }

        fn MapRasterWrite(self: Arc<Self>) -> io::Result<Box<dyn RasterWriteMapping>> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "GPU-like test image is not host mappable",
            ))
        }
    }

    fn frame(token: u32, damage: DeviceRect, contributing_damage: bool) -> CompositorFrame {
        let rect = DeviceRect::new(0, 0, 10, 10);
        let mut pass = compositor_core::CompositorRenderPass::default();
        pass.id = RenderPassId(1);
        pass.output_rect = rect;
        pass.damage_rect = damage;
        pass.has_damage_from_contributing_content = contributing_damage;
        let mut frame = CompositorFrame::default();
        frame.metadata.frame_token = token;
        frame.metadata.device_scale_factor = 1.0;
        frame.render_pass_list.push(Box::new(pass));
        frame
    }

    fn frame_with_gpu_like_resource(token: u32) -> CompositorFrame {
        let rect = DeviceRect::new(0, 0, 10, 10);
        let child_id = ResourceId(7);
        let mut pass = compositor_core::CompositorRenderPass::default();
        pass.id = RenderPassId(1);
        pass.output_rect = rect;
        pass.damage_rect = rect;
        let shared = pass.AppendSharedQuadState();
        {
            let state = pass.SharedQuadStateMutable(shared);
            state.quad_layer_rect = rect;
            state.visible_quad_layer_rect = rect;
            state.are_contents_opaque = true;
        }
        pass.quad_list
            .push(compositor_core::runtime::frame::DrawQuad {
                geometry: compositor_core::runtime::frame::QuadGeometry {
                    shared_quad_state: Some(shared),
                    rect,
                    visible_rect: rect,
                    needs_blending: false,
                },
                material: QuadMaterial::Tile {
                    resource_id: child_id,
                    tex_coord_rect: compositor_core::adapter::geometry::RectF::new(
                        0.0, 0.0, 10.0, 10.0,
                    ),
                    nearest_neighbor: false,
                    force_anti_aliasing_off: false,
                },
            });
        let mut sync_token = RasterSyncToken::new(
            CommandBufferNamespace::GpuChannelSharedImageInterface,
            19,
            23,
        );
        sync_token.verified_flush = true;
        let shared_image = RasterSharedImage::new(Arc::new(GpuLikeSharedImage {
            allocation: ResourceId(700),
            token: sync_token,
        }));
        let mut frame = CompositorFrame::default();
        frame.metadata.frame_token = token;
        frame.metadata.device_scale_factor = 1.0;
        frame.resource_list.push(RasterTransferableResource {
            id: child_id,
            shared_image,
            source: RasterResourceSource::TileRasterTask,
            sync_token,
            synchronization_type: RasterSynchronizationType::SyncToken,
        });
        frame.render_pass_list.push(Box::new(pass));
        frame
    }

    #[test]
    fn aggregate_retains_non_host_readable_transferable_resource_for_gpu_renderer() {
        let surface = SurfaceId(70);
        let rect = DeviceRect::new(0, 0, 10, 10);
        let mut viz = VizEngine::New();
        viz.SubmitFrame(surface, frame_with_gpu_like_resource(1))
            .unwrap();
        let output = viz
            .Aggregate(
                rect,
                &[SurfacePlacement {
                    surface_id: surface,
                    destination: rect,
                }],
            )
            .unwrap();
        let display_id = output
            .Frame()
            .RenderPasses()
            .iter()
            .flat_map(|pass| &pass.quad_list)
            .find_map(|quad| match quad.material {
                QuadMaterial::Tile { resource_id, .. } => Some(resource_id),
                _ => None,
            })
            .expect("aggregated tile resource");
        assert!(output.Resource(display_id).is_none());
        let resource = output
            .TransferableResource(display_id)
            .expect("GPU transferable survives aggregation");
        assert_eq!(resource.id, display_id);
        assert_eq!(resource.shared_image.RasterAllocationId(), ResourceId(700));
        assert_eq!(
            resource.sync_token.namespace_id,
            CommandBufferNamespace::GpuChannelSharedImageInterface
        );
        assert_eq!(resource.sync_token.command_buffer_id, 19);
        assert_eq!(resource.sync_token.release_count, 23);
        assert_eq!(resource.source, RasterResourceSource::TileRasterTask);
    }

    fn opaque_color_frame(
        token: u32,
        rect: DeviceRect,
        color: Color4f,
        background: Color4f,
    ) -> CompositorFrame {
        let mut pass = compositor_core::CompositorRenderPass::default();
        pass.id = RenderPassId(1);
        pass.output_rect = rect;
        pass.damage_rect = rect;
        pass.has_transparent_background = false;
        let shared = pass.AppendSharedQuadState();
        {
            let state = pass.SharedQuadStateMutable(shared);
            state.quad_layer_rect = rect;
            state.visible_quad_layer_rect = rect;
            state.are_contents_opaque = true;
        }
        pass.quad_list
            .push(compositor_core::runtime::frame::DrawQuad {
                geometry: compositor_core::runtime::frame::QuadGeometry {
                    shared_quad_state: Some(shared),
                    rect,
                    visible_rect: rect,
                    needs_blending: false,
                },
                material: QuadMaterial::SolidColor {
                    color,
                    force_anti_aliasing_off: false,
                },
            });
        let mut frame = CompositorFrame::default();
        frame.metadata.frame_token = token;
        frame.metadata.device_scale_factor = 1.0;
        frame.metadata.root_background_color = background;
        frame.render_pass_list.push(Box::new(pass));
        frame
    }

    fn surface_frame(
        token: u32,
        damage: DeviceRect,
        child: SurfaceId,
        child_rect: DeviceRect,
    ) -> CompositorFrame {
        let output = DeviceRect::new(0, 0, 30, 10);
        let mut frame = frame(token, damage, true);
        let pass = frame.render_pass_list.last_mut().unwrap();
        pass.output_rect = output;
        pass.has_transparent_background = false;
        let state = pass.AppendSharedQuadState();
        pass.AppendSurfaceQuad(
            state,
            child_rect,
            child_rect,
            true,
            VizEngine::RuntimeSurfaceRange(child),
            Color4f {
                red: 0.25,
                green: 0.5,
                blue: 0.75,
                alpha: 1.0,
            },
            true,
            false,
            true,
        );
        frame
    }

    #[test]
    fn nested_surface_quad_resolves_to_child_pass_and_propagates_damage() {
        let parent = SurfaceId(10);
        let child = SurfaceId(11);
        let child_rect = DeviceRect::new(5, 0, 20, 10);
        let placement = SurfacePlacement {
            surface_id: parent,
            destination: DeviceRect::new(0, 0, 30, 10),
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(child, frame(1, DeviceRect::new(0, 0, 10, 10), true))
            .unwrap();
        viz.SubmitFrame(
            parent,
            surface_frame(1, DeviceRect::new(0, 0, 30, 10), child, child_rect),
        )
        .unwrap();
        let initial = viz
            .Aggregate(DeviceRect::new(0, 0, 30, 10), &[placement])
            .unwrap();
        assert!(initial.Frame().RenderPasses().iter().any(|pass| {
            pass.quad_list
                .iter()
                .any(|quad| matches!(quad.material, QuadMaterial::CompositorRenderPass { .. }))
        }));
        assert!(initial.Frame().RenderPasses().iter().all(|pass| {
            pass.quad_list
                .iter()
                .all(|quad| !matches!(quad.material, QuadMaterial::Surface { .. }))
        }));
        viz.DidDraw(initial);

        viz.SubmitFrame(child, frame(2, DeviceRect::new(0, 0, 2, 5), true))
            .unwrap();
        let next = viz
            .Aggregate(DeviceRect::new(0, 0, 30, 10), &[placement])
            .unwrap();
        assert_eq!(next.OutputDamageRect(), DeviceRect::new(5, 0, 4, 5));
    }

    #[test]
    fn missing_nested_surface_emits_default_background_quad() {
        let parent = SurfaceId(20);
        let missing = SurfaceId(21);
        let placement = SurfacePlacement {
            surface_id: parent,
            destination: DeviceRect::new(0, 0, 30, 10),
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(
            parent,
            surface_frame(
                1,
                DeviceRect::new(0, 0, 30, 10),
                missing,
                DeviceRect::new(5, 0, 20, 10),
            ),
        )
        .unwrap();
        let output = viz
            .Aggregate(DeviceRect::new(0, 0, 30, 10), &[placement])
            .unwrap();
        assert!(output.Frame().RenderPasses().iter().any(|pass| {
            pass.quad_list.iter().any(|quad| {
                matches!(
                    quad.material,
                    QuadMaterial::SolidColor { color, .. }
                        if color == Color4f {
                            red: 0.25,
                            green: 0.5,
                            blue: 0.75,
                            alpha: 1.0,
                        }
                )
            })
        }));
    }

    #[test]
    fn late_video_surface_in_runtime_namespaces_replaces_missing_background() {
        let parent = SurfaceId(1_u64 << 63);
        let child = SurfaceId(1_u64 << 32);
        let child_rect = DeviceRect::new(5, 0, 20, 10);
        let placement = SurfacePlacement {
            surface_id: parent,
            destination: DeviceRect::new(0, 0, 30, 10),
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(
            parent,
            surface_frame(1, DeviceRect::new(0, 0, 30, 10), child, child_rect),
        )
        .unwrap();
        let missing = viz
            .Aggregate(DeviceRect::new(0, 0, 30, 10), &[placement])
            .unwrap();
        assert!(missing.Frame().RenderPasses().iter().any(|pass| {
            pass.quad_list
                .iter()
                .any(|quad| matches!(quad.material, QuadMaterial::SolidColor { .. }))
        }));
        viz.DidDraw(missing);

        viz.SubmitFrame(child, frame(1, DeviceRect::new(0, 0, 10, 10), true))
            .unwrap();
        let resolved = viz
            .Aggregate(DeviceRect::new(0, 0, 30, 10), &[placement])
            .unwrap();
        assert!(resolved.Frame().RenderPasses().iter().any(|pass| {
            pass.quad_list
                .iter()
                .any(|quad| matches!(quad.material, QuadMaterial::CompositorRenderPass { .. }))
        }));
    }

    #[test]
    fn active_video_surface_satisfies_parent_activation_dependency() {
        let parent = SurfaceId(1_u64 << 63);
        let child = SurfaceId(1_u64 << 32);
        let placement = SurfacePlacement {
            surface_id: parent,
            destination: DeviceRect::new(0, 0, 30, 10),
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(child, frame(1, DeviceRect::new(0, 0, 10, 10), true))
            .unwrap();
        let mut parent_frame = surface_frame(
            1,
            DeviceRect::new(0, 0, 30, 10),
            child,
            DeviceRect::new(5, 0, 20, 10),
        );
        parent_frame
            .metadata
            .activation_dependencies
            .push(VizEngine::RuntimeSurfaceId(child));
        viz.SubmitFrame(parent, parent_frame).unwrap();
        assert!(viz.HasActiveFrame(parent));
        let output = viz
            .Aggregate(DeviceRect::new(0, 0, 30, 10), &[placement])
            .unwrap();
        assert!(output.Frame().RenderPasses().iter().any(|pass| {
            pass.quad_list
                .iter()
                .any(|quad| matches!(quad.material, QuadMaterial::CompositorRenderPass { .. }))
        }));
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
        // The partial root and the materialized display root each draw the
        // damaged pixels; the latter also owns the uncovered output gutter.
        assert_eq!(initial.DamagePixels(), 200);
        viz.DidDraw(initial);

        viz.SubmitFrame(second, frame(1, full_damage, true))
            .unwrap();
        let next = viz
            .Aggregate(
                DeviceRect::new(0, 0, 20, 10),
                &[placement(first, 0), placement(second, 10)],
            )
            .unwrap();
        assert_eq!(next.DamagePixels(), 200);
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
        assert_eq!(output.OutputDamageRects(), &[partial_damage]);
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
        viz.SubmitFrame(popup, frame(1, destination, true)).unwrap();
        let initial = viz
            .Aggregate(destination, &[placement(document), placement(popup)])
            .unwrap();
        viz.DidDraw(initial);

        let output = viz.Aggregate(destination, &[placement(document)]).unwrap();
        assert_eq!(output.DamagePixels(), 100);
        assert_eq!(output.Frame().SurfaceDamageRects(), &[destination]);
        assert_eq!(output.OutputDamageRects(), &[destination]);
        assert_eq!(output.Frame().RenderPasses()[0].DamageRect(), destination);
    }

    #[test]
    fn fallback_gutter_still_materializes_one_display_root() {
        let toolbar = SurfaceId(31);
        let content = SurfaceId(32);
        let child_rect = DeviceRect::new(0, 0, 10, 10);
        let output_rect = DeviceRect::new(0, 0, 30, 10);
        let background = Color4f {
            red: 0.125,
            green: 0.25,
            blue: 0.5,
            alpha: 1.0,
        };
        let foreground = Color4f {
            red: 0.75,
            green: 0.5,
            blue: 0.25,
            alpha: 1.0,
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(
            toolbar,
            opaque_color_frame(1, child_rect, foreground, background),
        )
        .unwrap();
        viz.SubmitFrame(
            content,
            opaque_color_frame(1, child_rect, foreground, Color4f::default()),
        )
        .unwrap();

        let aggregate = viz
            .Aggregate(
                output_rect,
                &[
                    SurfacePlacement {
                        surface_id: toolbar,
                        destination: child_rect,
                    },
                    SurfacePlacement {
                        surface_id: content,
                        destination: DeviceRect::new(10, 0, 10, 10),
                    },
                ],
            )
            .unwrap();

        assert_eq!(aggregate.roots.len(), 1);
        assert_eq!(aggregate.roots[0].destination, output_rect);
        assert_eq!(aggregate.Frame().RenderPasses().len(), 1);
        let display_root = &aggregate.Frame().RenderPasses()[0];
        assert_eq!(display_root.OutputRect(), output_rect);
        assert!(display_root.quad_list.iter().any(|quad| {
            quad.geometry.rect == output_rect
                && matches!(quad.material, QuadMaterial::SolidColor { color, .. } if color == background)
        }));
    }

    #[test]
    fn browser_root_backdrops_are_behind_surfaces_and_keep_front_to_back_order() {
        let toolbar = SurfaceId(35);
        let child_rect = DeviceRect::new(0, 0, 10, 10);
        let output_rect = DeviceRect::new(0, 0, 20, 10);
        let child_color = Color4f {
            red: 0.9,
            green: 0.8,
            blue: 0.7,
            alpha: 1.0,
        };
        let border = Color4f {
            red: 0.1,
            green: 0.2,
            blue: 0.3,
            alpha: 1.0,
        };
        let navigation = Color4f {
            red: 0.4,
            green: 0.5,
            blue: 0.6,
            alpha: 1.0,
        };
        let tabstrip = Color4f {
            red: 0.7,
            green: 0.6,
            blue: 0.5,
            alpha: 1.0,
        };
        let background = Color4f {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let root = DisplayRootFrame {
            background_color: background,
            solid_color_quads: vec![
                DisplayRootSolidColorQuad {
                    destination: DeviceRect::new(0, 8, 20, 2),
                    color: border,
                },
                DisplayRootSolidColorQuad {
                    destination: DeviceRect::new(0, 4, 20, 6),
                    color: navigation,
                },
                DisplayRootSolidColorQuad {
                    destination: DeviceRect::new(0, 0, 20, 4),
                    color: tabstrip,
                },
            ],
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(
            toolbar,
            opaque_color_frame(1, child_rect, child_color, child_color),
        )
        .unwrap();

        let aggregate = viz
            .AggregateWithDisplayRoot(
                output_rect,
                &[SurfacePlacement {
                    surface_id: toolbar,
                    destination: child_rect,
                }],
                &root,
            )
            .unwrap();
        let display_root = aggregate.Frame().RenderPasses().last().unwrap();
        let solid_colors = display_root
            .quad_list
            .iter()
            .filter_map(|quad| match &quad.material {
                QuadMaterial::SolidColor { color, .. } => Some(*color),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(solid_colors[0], child_color);
        assert_eq!(
            &solid_colors[solid_colors.len() - 4..],
            &[border, navigation, tabstrip, background]
        );
    }

    #[test]
    fn transparent_surface_merges_into_browser_root_without_an_intermediate_pass() {
        let toolbar = SurfaceId(37);
        let rect = DeviceRect::new(0, 0, 20, 10);
        let foreground = Color4f {
            red: 0.2,
            green: 0.3,
            blue: 0.4,
            alpha: 0.5,
        };
        let background = Color4f {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let mut frame = opaque_color_frame(1, rect, foreground, Color4f::default());
        let pass = &mut frame.render_pass_list[0];
        pass.has_transparent_background = true;
        pass.SharedQuadStateMutable(SharedQuadStateHandle::FromIndex(0))
            .are_contents_opaque = false;
        pass.quad_list[0].geometry.needs_blending = true;

        let mut viz = VizEngine::New();
        viz.SubmitFrame(toolbar, frame).unwrap();
        let aggregate = viz
            .AggregateWithDisplayRoot(
                rect,
                &[SurfacePlacement {
                    surface_id: toolbar,
                    destination: rect,
                }],
                &DisplayRootFrame {
                    background_color: background,
                    solid_color_quads: vec![DisplayRootSolidColorQuad {
                        destination: DeviceRect::new(0, 0, 5, 10),
                        color: background,
                    }],
                },
            )
            .unwrap();

        assert_eq!(aggregate.Frame().RenderPasses().len(), 1);
        let display_root = &aggregate.Frame().RenderPasses()[0];
        assert!(matches!(
            display_root.quad_list[0].material,
            QuadMaterial::SolidColor { color, .. } if color == foreground
        ));
        assert!(display_root.quad_list[0].geometry.needs_blending);
        let state = display_root.quad_list[0]
            .geometry
            .shared_quad_state
            .expect("merged toolbar quad keeps its shared state");
        assert!(!display_root.SharedQuadStates()[state.Index()].are_contents_opaque);
    }

    #[test]
    fn changing_browser_root_backdrop_damages_retained_output() {
        let toolbar = SurfaceId(36);
        let output_rect = DeviceRect::new(0, 0, 10, 10);
        let placement = SurfacePlacement {
            surface_id: toolbar,
            destination: output_rect,
        };
        let background = Color4f {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let root = |red| DisplayRootFrame {
            background_color: background,
            solid_color_quads: vec![DisplayRootSolidColorQuad {
                destination: DeviceRect::new(0, 0, 10, 4),
                color: Color4f {
                    red,
                    green: 0.25,
                    blue: 0.5,
                    alpha: 1.0,
                },
            }],
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(
            toolbar,
            opaque_color_frame(1, output_rect, background, background),
        )
        .unwrap();
        let initial = viz
            .AggregateWithDisplayRoot(output_rect, &[placement], &root(0.25))
            .unwrap();
        viz.DidDraw(initial);

        let changed = viz
            .AggregateWithDisplayRoot(output_rect, &[placement], &root(0.75))
            .unwrap();
        assert_eq!(changed.OutputDamageRects(), &[output_rect]);
        assert_eq!(
            changed.Frame().RenderPasses().last().unwrap().DamageRect(),
            output_rect
        );
    }

    #[test]
    fn unmergeable_surface_is_referenced_by_display_root() {
        let surface = SurfaceId(41);
        let child_rect = DeviceRect::new(0, 0, 10, 10);
        let output_rect = DeviceRect::new(0, 0, 20, 10);
        let color = Color4f {
            red: 0.2,
            green: 0.4,
            blue: 0.6,
            alpha: 1.0,
        };
        let mut child = opaque_color_frame(1, child_rect, color, color);
        child.render_pass_list[0].cache_render_pass = true;
        let mut viz = VizEngine::New();
        viz.SubmitFrame(surface, child).unwrap();

        let aggregate = viz
            .Aggregate(
                output_rect,
                &[SurfacePlacement {
                    surface_id: surface,
                    destination: child_rect,
                }],
            )
            .unwrap();

        assert_eq!(aggregate.roots.len(), 1);
        assert_eq!(aggregate.roots[0].destination, output_rect);
        assert_eq!(aggregate.Frame().RenderPasses().len(), 2);
        let display_root = aggregate.Frame().RenderPasses().last().unwrap();
        assert!(display_root
            .quad_list
            .iter()
            .any(|quad| { matches!(quad.material, QuadMaterial::CompositorRenderPass { .. }) }));
    }

    #[test]
    fn shrinking_fallback_damages_newly_exposed_gutter() {
        let surface = SurfaceId(51);
        let frame_rect = DeviceRect::new(0, 0, 20, 10);
        let output_rect = frame_rect;
        let color = Color4f {
            red: 0.3,
            green: 0.4,
            blue: 0.5,
            alpha: 1.0,
        };
        let mut viz = VizEngine::New();
        viz.SubmitFrame(surface, opaque_color_frame(1, frame_rect, color, color))
            .unwrap();
        let initial = viz
            .Aggregate(
                output_rect,
                &[SurfacePlacement {
                    surface_id: surface,
                    destination: frame_rect,
                }],
            )
            .unwrap();
        viz.DidDraw(initial);

        let aggregate = viz
            .Aggregate(
                output_rect,
                &[SurfacePlacement {
                    surface_id: surface,
                    destination: DeviceRect::new(0, 0, 10, 10),
                }],
            )
            .unwrap();

        assert_eq!(aggregate.OutputDamageRects(), &[output_rect]);
        assert_eq!(
            aggregate
                .Frame()
                .RenderPasses()
                .last()
                .unwrap()
                .DamageRect(),
            output_rect
        );
    }
}

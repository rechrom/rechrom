#![allow(non_snake_case)]
//! Backend-independent display scheduling and surface aggregation.
//!
//! Compositor clients submit immutable [`compositor::CompositorFrame`] values
//! to persistent surfaces. `VizEngine` selects the active frame and publishes
//! an [`AggregatedFrame`] for `renderer`. This crate owns neither pixels nor a
//! clock, thread, native window, Skia context, or presentation API.

use compositor::{CompositorFrame, ResourceId};
pub use compositor::{
    DeviceRect, DrawQuad, LayerComposition, PaintRect, SharedQuadState, SolidColorDrawQuad,
    TransferableResource,
};
use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameTiming {
    pub frame_time: Instant,
    pub interval: Duration,
    pub source_deadline: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDeadlines {
    pub compositor_prepare: Instant,
    pub draw_and_swap: Instant,
}

impl FrameTiming {
    pub fn Deadlines(self) -> FrameDeadlines {
        FrameDeadlines {
            compositor_prepare: self.frame_time + self.interval / 3,
            draw_and_swap: self
                .source_deadline
                .checked_sub(self.interval / 3)
                .unwrap_or(self.frame_time)
                .max(self.frame_time),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScheduledFrame<T> {
    payload: T,
    deadlines: FrameDeadlines,
    prepared: bool,
}

impl<T: Copy> ScheduledFrame<T> {
    pub fn Payload(self) -> T {
        self.payload
    }

    pub fn IsPrepared(self) -> bool {
        self.prepared
    }
}

/// Pure Viz deadline state. The embedding runtime supplies time and executes
/// the returned work; this type owns policy without owning a clock or thread.
pub struct DisplayScheduler<T> {
    pending: Option<ScheduledFrame<T>>,
}

impl<T> Default for DisplayScheduler<T> {
    fn default() -> Self {
        Self { pending: None }
    }
}

impl<T: Copy> DisplayScheduler<T> {
    pub fn BeginFrame(&mut self, payload: T, timing: FrameTiming) -> Option<ScheduledFrame<T>> {
        self.pending.replace(ScheduledFrame {
            payload,
            deadlines: timing.Deadlines(),
            prepared: false,
        })
    }

    pub fn NeedsPrepare(&self, now: Instant) -> bool {
        self.pending
            .is_some_and(|frame| !frame.prepared && now >= frame.deadlines.compositor_prepare)
    }

    pub fn MarkPrepared(&mut self) {
        if let Some(frame) = &mut self.pending {
            frame.prepared = true;
        }
    }

    pub fn PendingPayload(&self) -> Option<T> {
        self.pending.map(|frame| frame.payload)
    }

    pub fn TakeIfDrawDue(&mut self, now: Instant) -> Option<T> {
        self.pending
            .filter(|frame| now >= frame.deadlines.draw_and_swap)
            .and_then(|_| self.pending.take())
            .map(|frame| frame.payload)
    }

    pub fn NextDeadline(&self) -> Option<Instant> {
        self.pending.map(|frame| {
            if frame.prepared {
                frame.deadlines.draw_and_swap
            } else {
                frame.deadlines.compositor_prepare
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SwapId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePlacement {
    pub surface_id: SurfaceId,
    /// Placement in the final display's device-pixel coordinate space.
    pub destination: DeviceRect,
}

/// One surface root pass after Viz has consumed the submitted
/// `CompositorFrame`. The renderer sees render-pass protocol and a resource
/// namespace, never the original surface submission or its scheduling state.
#[derive(Clone, Debug)]
pub struct AggregatedRenderPass {
    surface_id: SurfaceId,
    destination: DeviceRect,
    source_frame_id: u64,
    device_scale_factor: f64,
    logical_viewport: PaintRect,
    viewport: DeviceRect,
    damage_rect: DeviceRect,
    raster_task_count: usize,
    resource_list: Arc<[TransferableResource]>,
    shared_quad_state_list: Arc<[SharedQuadState]>,
    quad_list: Arc<[DrawQuad]>,
}

impl AggregatedRenderPass {
    pub fn Surface(&self) -> SurfaceId {
        self.surface_id
    }

    pub fn Destination(&self) -> DeviceRect {
        self.destination
    }

    pub fn SourceFrameId(&self) -> u64 {
        self.source_frame_id
    }

    pub fn DeviceScaleFactor(&self) -> f64 {
        self.device_scale_factor
    }

    pub fn LogicalViewport(&self) -> PaintRect {
        self.logical_viewport
    }

    pub fn Viewport(&self) -> DeviceRect {
        self.viewport
    }

    pub fn DamageRect(&self) -> DeviceRect {
        self.damage_rect
    }

    pub fn RasterTaskCount(&self) -> usize {
        self.raster_task_count
    }

    pub fn Resources(&self) -> &[TransferableResource] {
        &self.resource_list
    }

    pub fn SharedQuadStates(&self) -> &[SharedQuadState] {
        &self.shared_quad_state_list
    }

    pub fn Quads(&self) -> &[DrawQuad] {
        &self.quad_list
    }
}

/// Immutable renderer input produced by resolving the active display surfaces.
///
/// Chromium's SurfaceAggregator can recursively resolve SurfaceDrawQuads. The
/// current Rechrom compositor emits root tile surfaces only, so aggregation
/// resolves those roots into one ordered display frame. Surface identity stays
/// on each resolved pass solely as a resource namespace and retained-renderer
/// key; surface scheduling state does not cross this boundary.
#[derive(Clone, Debug)]
pub struct AggregatedFrame {
    output_rect: DeviceRect,
    damage_rect: DeviceRect,
    render_passes: Arc<[AggregatedRenderPass]>,
}

impl AggregatedFrame {
    pub fn OutputRect(&self) -> DeviceRect {
        self.output_rect
    }

    pub fn DamageRect(&self) -> DeviceRect {
        self.damage_rect
    }

    pub fn RenderPasses(&self) -> &[AggregatedRenderPass] {
        &self.render_passes
    }
}

#[derive(Clone, Debug)]
struct SurfaceState {
    source_frame_id: u64,
    device_scale_factor: f64,
    logical_viewport: PaintRect,
    viewport: DeviceRect,
    damage_rect: DeviceRect,
    raster_task_count: usize,
    resource_list: Arc<[TransferableResource]>,
    shared_quad_state_list: Arc<[SharedQuadState]>,
    quad_list: Arc<[DrawQuad]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VizError {
    EmptyRenderPassList,
    InvalidRootPass,
    MissingSharedQuadState,
    MissingResource,
    MissingSurface,
    InvalidSurfacePlacement,
    UnsupportedRenderPassGraph,
    DuplicateSwap,
}

impl fmt::Display for VizError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyRenderPassList => "compositor frame has no render pass",
            Self::InvalidRootPass => "compositor frame root pass metadata differs",
            Self::MissingSharedQuadState => "draw quad has no shared state",
            Self::MissingResource => "draw quad references an unlisted resource",
            Self::MissingSurface => "surface has no active compositor frame",
            Self::InvalidSurfacePlacement => "surface placement is outside display output",
            Self::UnsupportedRenderPassGraph => {
                "compositor frame contains an unsupported render-pass graph"
            }
            Self::DuplicateSwap => "swap id is already pending",
        })
    }
}

impl std::error::Error for VizError {}

fn ValidateFrame(frame: &CompositorFrame) -> Result<(), VizError> {
    let root = frame
        .render_pass_list
        .last()
        .ok_or(VizError::EmptyRenderPassList)?;
    if root.output_rect != frame.metadata.viewport
        || root.damage_rect != frame.metadata.root_damage_rect
    {
        return Err(VizError::InvalidRootPass);
    }
    let resources: BTreeSet<ResourceId> = frame
        .resource_list
        .iter()
        .map(|resource| resource.id)
        .collect();
    for quad in &root.quad_list {
        if let DrawQuad::Tile(tile) = quad {
            if root
                .shared_quad_state_list
                .get(tile.shared_quad_state_index)
                .is_none()
            {
                return Err(VizError::MissingSharedQuadState);
            }
            if !resources.contains(&tile.resource_id) {
                return Err(VizError::MissingResource);
            }
        }
    }
    Ok(())
}

/// Persistent Viz-side state for a display pipeline.
///
/// Surface frames and pending swaps survive across BeginFrames. The embedding
/// application supplies timing mutations and routes the resulting aggregate to
/// `renderer`; deployment on one or several threads does not change this API.
pub struct VizEngine {
    surfaces: HashMap<SurfaceId, SurfaceState>,
    pending_swaps: BTreeSet<SwapId>,
    max_pending_swaps: usize,
}

impl Default for VizEngine {
    fn default() -> Self {
        Self::New()
    }
}

impl VizEngine {
    pub fn New() -> Self {
        Self {
            surfaces: HashMap::new(),
            pending_swaps: BTreeSet::new(),
            max_pending_swaps: 1,
        }
    }

    pub fn SubmitFrame(
        &mut self,
        surface_id: SurfaceId,
        frame: CompositorFrame,
    ) -> Result<(), VizError> {
        ValidateFrame(&frame)?;
        if frame.render_pass_list.len() != 1 {
            return Err(VizError::UnsupportedRenderPassGraph);
        }
        let CompositorFrame {
            metadata,
            resource_list,
            mut render_pass_list,
        } = frame;
        let root = render_pass_list
            .pop()
            .ok_or(VizError::EmptyRenderPassList)?;
        self.surfaces.insert(
            surface_id,
            SurfaceState {
                source_frame_id: metadata.source_frame_id,
                device_scale_factor: metadata.device_scale_factor,
                logical_viewport: metadata.logical_viewport,
                viewport: metadata.viewport,
                damage_rect: metadata.root_damage_rect,
                raster_task_count: metadata.raster_task_count,
                resource_list: resource_list.into(),
                shared_quad_state_list: root.shared_quad_state_list.into(),
                quad_list: root.quad_list.into(),
            },
        );
        Ok(())
    }

    pub fn Aggregate(
        &self,
        output_rect: DeviceRect,
        placements: &[SurfacePlacement],
    ) -> Result<AggregatedFrame, VizError> {
        let mut render_passes = Vec::with_capacity(placements.len());
        let mut damage = DeviceRect::new(output_rect.x, output_rect.y, 0, 0);
        for placement in placements {
            let surface = self
                .surfaces
                .get(&placement.surface_id)
                .ok_or(VizError::MissingSurface)?;
            let viewport = surface.viewport;
            if placement.destination.width != viewport.width
                || placement.destination.height != viewport.height
                || !contains(output_rect, placement.destination)
            {
                return Err(VizError::InvalidSurfacePlacement);
            }
            let local_damage = surface.damage_rect;
            damage = union(
                damage,
                translate_damage(
                    local_damage,
                    viewport,
                    placement.destination.x,
                    placement.destination.y,
                )?,
            );
            render_passes.push(AggregatedRenderPass {
                surface_id: placement.surface_id,
                destination: placement.destination,
                source_frame_id: surface.source_frame_id,
                device_scale_factor: surface.device_scale_factor,
                logical_viewport: surface.logical_viewport,
                viewport: surface.viewport,
                damage_rect: surface.damage_rect,
                raster_task_count: surface.raster_task_count,
                resource_list: surface.resource_list.clone(),
                shared_quad_state_list: surface.shared_quad_state_list.clone(),
                quad_list: surface.quad_list.clone(),
            });
        }
        Ok(AggregatedFrame {
            output_rect,
            damage_rect: damage,
            render_passes: render_passes.into(),
        })
    }

    pub fn DestroySurface(&mut self, surface_id: SurfaceId) {
        self.surfaces.remove(&surface_id);
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

    pub fn HasActiveFrame(&self, surface_id: SurfaceId) -> bool {
        self.surfaces.contains_key(&surface_id)
    }
}

fn right(rect: DeviceRect) -> Option<i64> {
    i64::from(rect.x).checked_add(i64::from(rect.width))
}

fn bottom(rect: DeviceRect) -> Option<i64> {
    i64::from(rect.y).checked_add(i64::from(rect.height))
}

fn contains(outer: DeviceRect, inner: DeviceRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && right(inner).zip(right(outer)).is_some_and(|(a, b)| a <= b)
        && bottom(inner)
            .zip(bottom(outer))
            .is_some_and(|(a, b)| a <= b)
}

fn translate_damage(
    damage: DeviceRect,
    viewport: DeviceRect,
    destination_x: i32,
    destination_y: i32,
) -> Result<DeviceRect, VizError> {
    if !contains(viewport, damage) {
        return Err(VizError::InvalidRootPass);
    }
    let x = i64::from(destination_x) + i64::from(damage.x - viewport.x);
    let y = i64::from(destination_y) + i64::from(damage.y - viewport.y);
    Ok(DeviceRect::new(
        i32::try_from(x).map_err(|_| VizError::InvalidSurfacePlacement)?,
        i32::try_from(y).map_err(|_| VizError::InvalidSurfacePlacement)?,
        damage.width,
        damage.height,
    ))
}

fn union(a: DeviceRect, b: DeviceRect) -> DeviceRect {
    if a.width == 0 || a.height == 0 {
        return b;
    }
    if b.width == 0 || b.height == 0 {
        return a;
    }
    let left = a.x.min(b.x);
    let top = a.y.min(b.y);
    let right = right(a)
        .expect("validated rectangle")
        .max(right(b).expect("validated rectangle"));
    let bottom = bottom(a)
        .expect("validated rectangle")
        .max(bottom(b).expect("validated rectangle"));
    DeviceRect::new(
        left,
        top,
        u32::try_from(right - i64::from(left)).expect("validated rectangle union"),
        u32::try_from(bottom - i64::from(top)).expect("validated rectangle union"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use compositor::{CompositorFrameMetadata, CompositorRenderPass, RenderPassId};
    use paint::paint_engine::PaintRect;

    fn empty_frame(token: u64) -> CompositorFrame {
        let viewport = DeviceRect::new(0, 0, 80, 60);
        CompositorFrame {
            metadata: CompositorFrameMetadata {
                frame_token: token,
                source_frame_id: token,
                device_scale_factor: 1.0,
                logical_viewport: PaintRect {
                    x: 0.0,
                    y: 0.0,
                    width: 80.0,
                    height: 60.0,
                },
                viewport,
                root_damage_rect: viewport,
                raster_task_count: 0,
                begin_frame_ack: None,
            },
            resource_list: Vec::new(),
            render_pass_list: vec![CompositorRenderPass {
                id: RenderPassId(1),
                output_rect: viewport,
                damage_rect: viewport,
                shared_quad_state_list: Vec::new(),
                quad_list: Vec::new(),
            }],
        }
    }

    #[test]
    fn retains_active_surface_and_swap_back_pressure() {
        let mut viz = VizEngine::New();
        let surface = SurfaceId(4);
        viz.SubmitFrame(surface, empty_frame(7)).unwrap();
        let placement = SurfacePlacement {
            surface_id: surface,
            destination: DeviceRect::new(0, 0, 80, 60),
        };
        assert_eq!(
            viz.Aggregate(DeviceRect::new(0, 0, 80, 60), &[placement])
                .unwrap()
                .RenderPasses()[0]
                .SourceFrameId(),
            7
        );
        assert!(viz.CanDrawAndSwap());
        viz.DidSubmitSwap(SwapId(9)).unwrap();
        assert!(!viz.CanDrawAndSwap());
        assert!(viz.DidReceiveSwapAck(SwapId(9)));
        assert!(viz.CanDrawAndSwap());
    }

    #[test]
    fn aggregates_ordered_surfaces_and_translates_damage() {
        let mut viz = VizEngine::New();
        let toolbar = SurfaceId(1);
        let content = SurfaceId(2);
        viz.SubmitFrame(toolbar, empty_frame(10)).unwrap();
        viz.SubmitFrame(content, empty_frame(20)).unwrap();
        let frame = viz
            .Aggregate(
                DeviceRect::new(0, 0, 80, 120),
                &[
                    SurfacePlacement {
                        surface_id: toolbar,
                        destination: DeviceRect::new(0, 0, 80, 60),
                    },
                    SurfacePlacement {
                        surface_id: content,
                        destination: DeviceRect::new(0, 60, 80, 60),
                    },
                ],
            )
            .unwrap();
        assert_eq!(frame.DamageRect(), DeviceRect::new(0, 0, 80, 120));
        assert_eq!(frame.RenderPasses()[0].Surface(), toolbar);
        assert_eq!(frame.RenderPasses()[1].Surface(), content);
    }

    #[test]
    fn scheduler_owns_prepare_and_draw_deadline_policy() {
        let start = Instant::now();
        let interval = Duration::from_millis(15);
        let mut scheduler = DisplayScheduler::default();
        assert!(scheduler
            .BeginFrame(
                7u64,
                FrameTiming {
                    frame_time: start,
                    interval,
                    source_deadline: start + interval,
                },
            )
            .is_none());
        assert_eq!(
            scheduler.NextDeadline(),
            Some(start + Duration::from_millis(5))
        );
        assert!(!scheduler.NeedsPrepare(start + Duration::from_millis(4)));
        assert!(scheduler.NeedsPrepare(start + Duration::from_millis(5)));
        scheduler.MarkPrepared();
        assert_eq!(
            scheduler.NextDeadline(),
            Some(start + Duration::from_millis(10))
        );
        assert_eq!(
            scheduler.TakeIfDrawDue(start + Duration::from_millis(9)),
            None
        );
        assert_eq!(
            scheduler.TakeIfDrawDue(start + Duration::from_millis(10)),
            Some(7)
        );
    }
}

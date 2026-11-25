//! One concrete CPU compositor family. R is the external Skia value backend;
//! cc owners are canonical types, never supplied by R or copied from old core.
use super::{frame, geometry::Transform, metadata};
use crate::adapter::frame_data::FrameDataTypes;
use std::{marker::PhantomData, sync::Arc};

pub use layerize::commit::skia_value_backend::MainSkiaBackend;
/// The impl-side filter facets of the Skia value backend. The value types and
/// the path operations are the main half's `MainSkiaBackend`.
pub trait SkiaValueBackend:
    MainSkiaBackend<
    Filters: crate::adapter::render_surface_impl::RenderSurfaceFilters
                 + crate::adapter::occlusion::OcclusionFilters
                 + crate::adapter::damage_context::DamageFilters,
>
{
}
impl<T> SkiaValueBackend for T
where
    T: MainSkiaBackend,
    T::Filters: crate::adapter::render_surface_impl::RenderSurfaceFilters
        + crate::adapter::occlusion::OcclusionFilters
        + crate::adapter::damage_context::DamageFilters,
{
}
pub use layerize::commit::skia_value_backend::PaintSkiaBackend;
/// cc/paint's backend (`PaintSkiaBackend`, commit vocabulary) plus the impl
/// filter facets.
pub trait CpuSkiaBackend: PaintSkiaBackend + SkiaValueBackend {}
impl<R> CpuSkiaBackend for R where R: PaintSkiaBackend + SkiaValueBackend {}

/// This marker has no resident service/host/tree. All owner constructors receive
/// actual owned native components and all runtime services remain call-scoped.
pub struct CpuCompositorTypes<R: CpuSkiaBackend>(PhantomData<fn() -> R>);
impl<R: CpuSkiaBackend> Default for CpuCompositorTypes<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<R: CpuSkiaBackend> CpuCompositorTypes<R> {
    pub fn new() -> Self {
        Self::default()
    }
}

pub use layerize::commit::main_callbacks::{ResultTaskRunner, TaskRunner};
// viz::CopyOutputRequest is a viz value; its cc facets live with their
// traits in layerize::commit.
pub use viz_common::copy_output_request::{CopyOutputRequest, CopyOutputRequestBackend};
/// Native SurfaceId values live in compositor_frame; routing/aggregation is
/// external Viz work.
pub use compositor_frame::surface_id::{FrameSinkId, LocalSurfaceId, SurfaceId, SurfaceRange};
impl crate::adapter::layer_tree_host_impl::HostHitTestSurfaceId for SurfaceId {
    type FrameSinkId = FrameSinkId;
    fn frame_sink_id(&self) -> FrameSinkId {
        self.frame_sink_id
    }
}
/// The viz frame values this family produces (viz_common::FrameValues).
pub type CpuFrameValues<R> = frame::FrameValues<
    Transform<R>,
    <R as super::types::MainSkiaBackend>::Filters,
    <R as layerize::commit::paint::skia_backend::RecordSkiaBackend>::Path,
    CopyOutputRequest,
    super::paint::ContentDisplayItemList<R>,
>;
impl<R: CpuSkiaBackend> FrameDataTypes for CpuCompositorTypes<R> {
    type SharedQuadStateHandle = frame::SharedQuadStateHandle;
    type DrawQuadHandle = frame::DrawQuadHandle;
    type RenderPassId = frame::RenderPassId;
    type SurfaceId = SurfaceId;
    type ResourceId = crate::adapter::layer_tree_impl::ResourceId;
    type RenderPass = frame::CompositorRenderPass<CpuFrameValues<R>>;
    type BeginFrameAck = frame::BeginFrameAck;
    type TreesInVizTiming = metadata::TreesInVizTiming;
}

impl<R: CpuSkiaBackend> frame::ViewTransitionGeometry for CpuCompositorTypes<R> {
    type ViewTransitionTransform = Transform<R>;
    // Source dependency: components/viz/common/viz_utils.cc:116-135.
    fn GetViewTransitionTransform(
        shared: crate::adapter::geometry::Rect,
        output: crate::adapter::geometry::Rect,
    ) -> Transform<R> {
        let mut transform = Transform::<R>::default();
        transform
            .transform_mut()
            .Translate(shared.x as f32, shared.y as f32);
        transform.transform_mut().Scale(
            shared.width as f32 / output.width as f32,
            shared.height as f32 / output.height as f32,
        );
        transform.transform_mut().Translate(
            output.x.wrapping_neg() as f32,
            output.y.wrapping_neg() as f32,
        );
        transform
    }
}

pub use layerize::commit::paint::NodeId;
impl crate::adapter::layer_impl::LayerImplNodeId for NodeId {
    fn IsNonzero(&self) -> bool {
        self.0 != 0
    }
    fn AsInteger(&self) -> i32 {
        self.0
    }
}
pub use layerize::commit::main_callbacks::DidScrollCallback;
/// Only the native global trace effect survives until LayerImpl's destructor.
/// The injected sink is forbidden from retaining a Layer, tree, host or Page.
pub struct DeletionTrace(
    pub Arc<dyn crate::adapter::layer_impl::LayerImplDeletionTrace + Send + Sync>,
);
impl crate::adapter::layer_impl::LayerImplDeletionTrace for DeletionTrace {
    fn LayerDeleted(&self, layer_id: i32, stable_id: i32) {
        self.0.LayerDeleted(layer_id, stable_id)
    }
}

/// Native ThreadUnsafeCommitState's non-owning MutatorHost* identity (shared).
pub use layerize::commit::mutator_host::MainMutatorHostIdentity;

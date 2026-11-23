//! One concrete CPU compositor family. R is the external Skia value backend;
//! cc owners are canonical types, never supplied by R or copied from old core.
use super::{
    frame,
    geometry::{RoundedRectTransformBackend, Transform},
    metadata,
};
use crate::adapter::{
    frame_data::FrameDataTypes,
    main_layer::MainLayerCopyOutputRequest,
    property_tree::PropertyTreeCopyOutputRequest,
    region_backend::SkRegionBackend,
};
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
/// Native Viz request ownership and abort-on-drop behavior are implemented by
/// the external Viz request owner. This wrapper forwards only native fields.
pub trait CopyOutputRequestBackend:
    PropertyTreeCopyOutputRequest + MainLayerCopyOutputRequest<TaskRunner = TaskRunner> + Send + Sync
{
}
impl<T> CopyOutputRequestBackend for T where
    T: PropertyTreeCopyOutputRequest
        + MainLayerCopyOutputRequest<TaskRunner = TaskRunner>
        + Send
        + Sync
{
}
pub struct CopyOutputRequest(pub Box<dyn CopyOutputRequestBackend>);
impl PropertyTreeCopyOutputRequest for CopyOutputRequest {
    fn has_area(&self) -> bool {
        PropertyTreeCopyOutputRequest::has_area(self.0.as_ref())
    }
    fn area(&self) -> crate::adapter::geometry::Rect {
        PropertyTreeCopyOutputRequest::area(self.0.as_ref())
    }
    fn set_area(&mut self, area: crate::adapter::geometry::Rect) {
        PropertyTreeCopyOutputRequest::set_area(self.0.as_mut(), area);
    }
    fn is_scaled(&self) -> bool {
        self.0.is_scaled()
    }
    fn has_result_selection(&self) -> bool {
        self.0.has_result_selection()
    }
    fn scale_from(&self) -> crate::adapter::geometry::Vector2d {
        self.0.scale_from()
    }
    fn scale_to(&self) -> crate::adapter::geometry::Vector2d {
        self.0.scale_to()
    }
    fn SetScaleRatio(
        &mut self,
        from: crate::adapter::geometry::Vector2d,
        to: crate::adapter::geometry::Vector2d,
    ) {
        self.0.SetScaleRatio(from, to);
    }
}
impl MainLayerCopyOutputRequest for CopyOutputRequest {
    type TaskRunner = TaskRunner;
    fn has_result_task_runner(&self) -> bool {
        self.0.has_result_task_runner()
    }
    fn set_result_task_runner(&mut self, runner: TaskRunner) {
        self.0.set_result_task_runner(runner);
    }
    fn has_area(&self) -> bool {
        MainLayerCopyOutputRequest::has_area(self.0.as_ref())
    }
    fn area(&self) -> &crate::adapter::geometry::Rect {
        MainLayerCopyOutputRequest::area(self.0.as_ref())
    }
    fn set_area(&mut self, area: crate::adapter::geometry::Rect) {
        MainLayerCopyOutputRequest::set_area(self.0.as_mut(), area);
    }
    fn has_source(&self) -> bool {
        self.0.has_source()
    }
    fn source(&self) -> crate::adapter::layer_tree_impl::UnguessableToken {
        self.0.source()
    }
}
/// Native SurfaceId values live in compositor_frame; routing/aggregation is
/// external Viz work.
pub use compositor_frame::surface_id::{FrameSinkId, LocalSurfaceId, SurfaceId, SurfaceRange};
impl crate::adapter::layer_tree_host_impl::HostHitTestSurfaceId for SurfaceId {
    type FrameSinkId = FrameSinkId;
    fn frame_sink_id(&self) -> FrameSinkId {
        self.frame_sink_id
    }
}
impl<R: CpuSkiaBackend> frame::FrameValueTypes for CpuCompositorTypes<R> {
    type FrameTransform = Transform<R>;
    type FrameFilters = R::Filters;
    type FramePath = R::Path;
    type FrameCopyRequest = CopyOutputRequest;
    type PictureDisplayItemList = super::paint::ContentDisplayItemList<R>;
}
impl<R: CpuSkiaBackend> FrameDataTypes for CpuCompositorTypes<R> {
    type SharedQuadStateHandle = frame::SharedQuadStateHandle;
    type DrawQuadHandle = frame::DrawQuadHandle;
    type RenderPassId = frame::RenderPassId;
    type SurfaceId = SurfaceId;
    type ResourceId = crate::adapter::layer_tree_impl::ResourceId;
    type RenderPass = frame::CompositorRenderPass<Self>;
    type BeginFrameAck = frame::BeginFrameAck;
    type TreesInVizTiming = metadata::TreesInVizTiming;
}

impl<R: CpuSkiaBackend> frame::ViewTransitionGeometry for CpuCompositorTypes<R> {
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

/// Native ThreadUnsafeCommitState's non-owning MutatorHost* identity. The
/// actual animation owner remains in its existing subsystem and is never
/// resolved through a global registry or retained by a Layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainMutatorHostIdentity(pub u64);

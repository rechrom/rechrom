//! One concrete CPU compositor family. R is the external Skia value backend;
//! cc owners are canonical types, never supplied by R or copied from old core.
use super::{
    frame,
    geometry::{RoundedRectTransformBackend, Transform},
    metadata,
};
use crate::adapter::{
    frame_data::FrameDataTypes,
    main_layer::{MainLayerCopyOutputRequest, MainLayerPath},
    property_tree::PropertyTreeCopyOutputRequest,
    region_backend::SkRegionBackend,
};
use std::{marker::PhantomData, sync::Arc};

/// Pure Skia-owned value types only. Policy/owner algorithms are forbidden in
/// this capability; the backend implements native filter/path operations.
pub trait SkiaValueBackend {
    type Filters: crate::adapter::render_surface_impl::RenderSurfaceFilters
        + crate::adapter::draw_property::DrawFilters
        + crate::adapter::occlusion::OcclusionFilters
        + crate::adapter::damage_context::DamageFilters
        + Send
        + Sync
        + 'static;
    type Path: MainLayerPath + Default + Send + Sync + 'static;
    /// Compile one immutable Paint-owned path into the selected backend value.
    /// No concrete Skia path crosses Paint's package boundary.
    fn FromPaintPath(path: &paint::paint_clip_path::PaintPath) -> Self::Path;
    /// Blink Path::StrokeBoundingRect delegates this geometry to Skia. Keeping
    /// it beside path compilation avoids introducing a second stroker in Paint.
    fn StrokeBounds(
        path: &paint::paint_clip_path::PaintPath,
        width: f32,
        resolution_scale: f32,
    ) -> Option<crate::adapter::geometry::RectF>;
}
pub trait CpuSkiaBackend:
    SkRegionBackend
    + RoundedRectTransformBackend
    + SkiaValueBackend<Filters = paint::cc::paint::filter_operations::FilterOperations>
    + Send
    + Sync
    + 'static
{
}
impl<R> CpuSkiaBackend for R where
    R: SkRegionBackend
        + RoundedRectTransformBackend
        + SkiaValueBackend<Filters = paint::cc::paint::filter_operations::FilterOperations>
        + Send
        + Sync
        + 'static
{
}

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

/// Strong native sequenced-task-runner capability, supplied by the embedding.
/// No worker/thread/clock implementation is created by the compositor family.
pub trait ResultTaskRunner: Send + Sync {
    fn PostTask(&self, task: Box<dyn FnOnce() + Send>);
}
pub type TaskRunner = Arc<dyn ResultTaskRunner>;
/// Native Viz request ownership and abort-on-drop behavior are implemented by
/// the external Viz request owner. This wrapper forwards only native fields.
pub trait NativeCopyOutputRequest:
    PropertyTreeCopyOutputRequest + MainLayerCopyOutputRequest<TaskRunner = TaskRunner> + Send + Sync
{
}
impl<T> NativeCopyOutputRequest for T where
    T: PropertyTreeCopyOutputRequest
        + MainLayerCopyOutputRequest<TaskRunner = TaskRunner>
        + Send
        + Sync
{
}
pub struct CopyOutputRequest(pub Box<dyn NativeCopyOutputRequest>);
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
/// Native SurfaceId fields; routing/aggregation is external Viz work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameSinkId {
    pub client_id: u32,
    pub sink_id: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalSurfaceId {
    pub parent_sequence_number: u32,
    pub child_sequence_number: u32,
    pub embed_token: [u64; 2],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceId {
    pub frame_sink_id: FrameSinkId,
    pub local_surface_id: LocalSurfaceId,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceRange {
    pub start: Option<SurfaceId>,
    pub end: SurfaceId,
}
impl crate::adapter::unbounded::LocalSurfaceId for LocalSurfaceId {
    fn is_valid(&self) -> bool {
        self.parent_sequence_number != 0
            && self.child_sequence_number != 0
            && self.embed_token != [0, 0]
    }
}
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
            .native_mut()
            .Translate(shared.x as f32, shared.y as f32);
        transform.native_mut().Scale(
            shared.width as f32 / output.width as f32,
            shared.height as f32 / output.height as f32,
        );
        transform.native_mut().Translate(
            output.x.wrapping_neg() as f32,
            output.y.wrapping_neg() as f32,
        );
        transform
    }
}

/// Source dependency cc/paint/node_id.h:17-20. Native invalid NodeId is zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeId(pub i32);
impl crate::adapter::layer_impl::LayerImplNodeId for NodeId {
    fn IsNonzero(&self) -> bool {
        self.0 != 0
    }
    fn AsInteger(&self) -> i32 {
        self.0
    }
}
/// Native base::RepeatingCallback void(PointF,ElementId). Cloning shares only
/// the callback allocation, not any compositor tree or Layer state.
#[derive(Clone, Default)]
pub struct DidScrollCallback(
    pub  Option<
        Arc<
            dyn Fn(crate::adapter::geometry::PointF, crate::adapter::input_types::ElementId)
                + Send
                + Sync,
        >,
    >,
);
impl crate::adapter::main_layer::LayerDidScrollCallback for DidScrollCallback {
    fn is_null(&self) -> bool {
        self.0.is_none()
    }
    fn Run(
        &self,
        offset: crate::adapter::geometry::PointF,
        element: crate::adapter::input_types::ElementId,
    ) {
        (self.0.as_ref().expect("native null callback invocation"))(offset, element)
    }
}
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

impl LocalSurfaceId {
    // Source: components/viz/common/surfaces/local_surface_id.cc:47-61.
    pub fn IsNewerThan(&self, other: &Self) -> bool {
        self.embed_token == other.embed_token
            && self
                .child_sequence_number
                .wrapping_sub(other.child_sequence_number)
                < (1u32 << 31)
            && self
                .parent_sequence_number
                .wrapping_sub(other.parent_sequence_number)
                < (1u32 << 31)
            && (self.child_sequence_number != other.child_sequence_number
                || self.parent_sequence_number != other.parent_sequence_number)
    }
    // Source: components/viz/common/surfaces/local_surface_id.cc:68-70.
    pub fn IsSameOrNewerThan(&self, other: &Self) -> bool {
        self == other || self.IsNewerThan(other)
    }
    // Source: components/viz/common/surfaces/local_surface_id.cc:63-66.
    pub(crate) fn IsNewerThanOrEmbeddingChanged(&self, other: &Self) -> bool {
        self.IsNewerThan(other) || self.embed_token != other.embed_token
    }
}

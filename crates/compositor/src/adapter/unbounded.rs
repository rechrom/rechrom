//! Live Viz/frame-sink transport services. ClientIdentity replaces the source
//! implicit `this` address, registered by the owner and stable across Rust moves.
use super::{geometry::Size, property_tree::PropertyTreeTransform};
pub trait LocalSurfaceId: Default + Clone + PartialEq {
    fn is_valid(&self) -> bool;
}
pub trait CompositorFrame<R> {
    fn render_pass_list_is_empty(&self) -> bool;
    fn SetLastRenderPassOutputSize(&mut self, size: Size);
    fn size_in_pixels(&self) -> Size;
    fn ReturnResources(&self) -> Vec<R>;
}
pub trait FrameSink<B: UnboundedTypes> {
    fn BindToClient(&mut self, client: B::ClientIdentity) -> bool;
    fn SetLocalSurfaceId(&mut self, id: &B::LocalSurfaceId);
    fn DetachFromClient(&mut self);
    fn SubmitCompositorFrame(&mut self, frame: B::Frame, hit_test_data_changed: bool);
}
// These calls borrow the resident owner for the duration of a callback. The
// child stores no owner handle, copied facade, or back reference.
// SubmitFrame only consumes the native resource-provider callback. The full
// owner's sink-loss callback remains separate and is never manufactured while
// the resident handler is mutably borrowed.
pub trait UnboundedResourceHost<B: UnboundedTypes> {
    fn ReceiveReturnsFromParent(&mut self, resources: Vec<B::ReturnedResource>);
}
impl<B: UnboundedTypes, H: UnboundedHost<B> + ?Sized> UnboundedResourceHost<B> for &mut H {
    fn ReceiveReturnsFromParent(&mut self, resources: Vec<B::ReturnedResource>) {
        UnboundedHost::ReceiveReturnsFromParent(*self, resources);
    }
}
pub trait UnboundedHost<B: UnboundedTypes> {
    fn ReceiveReturnsFromParent(&mut self, resources: Vec<B::ReturnedResource>);
    fn DismissUnboundedFrameSink(&mut self);
}
pub trait UnboundedTypes: Sized {
    type LocalSurfaceId: LocalSurfaceId;
    type ClientIdentity: Copy;
    type Frame: CompositorFrame<Self::ReturnedResource>;
    type FrameSink: FrameSink<Self>;
    type ReturnedResource;
    type HitTestRegionList;
    type BeginFrameSource;
    type RepeatingClosure;
    type FrameTimingDetails;
    type Transform: PropertyTreeTransform;
}

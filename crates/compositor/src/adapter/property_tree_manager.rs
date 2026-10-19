//! Typed live Blink property-node and client boundary for PropertyTreeManager.
//! The translated manager owns all conversion, stack, synthesis, and ordering
//! policy; this module only exposes the actual Paint-owned resident nodes and
//! its PaintArtifactCompositor client.

use super::{geometry::Rect, input_types::ElementId, layer_tree_impl::LayerTreeTypes};
use crate::cc::input::{
    main_thread_scrolling_reason::MainThreadRepaintReasons,
    overscroll_behavior::OverscrollBehavior, scroll_snap_data::SnapContainerData,
};

pub trait PropertyTreeManagerTypes:
    super::paint_artifact_compositor::SynthesizedClipTypes<
        ScrollNode: PropertyTreeManagerScrollNode,
    > + super::picture_layer::PictureLayerTypes
{
    /// Paint's resident singleton property nodes. Handles retain the actual
    /// nodes; the compositor never materializes a parallel property tree.
    fn RootTransformNode() -> Self::TransformNode;
    fn RootClipNode() -> Self::ClipNode;
    fn RootEffectNode() -> Self::EffectNode;
    fn RootScrollNode() -> Self::ScrollNode;
    fn ScrollNodeForTransform(transform: &Self::TransformNode) -> Option<Self::ScrollNode>;

    /// Exact gfx representation primitive used by SetupRootTransformNode.
    /// It constructs a 2d uniform scale and contains no compositor policy.
    fn UniformScaleTransform(scale: f32) -> <Self as LayerTreeTypes>::Transform;
    fn SourceToDestinationProjection(
        source: &Self::TransformNode,
        destination: &Self::TransformNode,
    ) -> <Self as LayerTreeTypes>::Transform;
    fn EffectMaskElementId(effect: ElementId) -> ElementId;
    fn NewUniqueElementId() -> ElementId;
}

pub trait PropertyTreeManagerScrollNode: Clone {
    fn CcNodeId(&self, sequence: i32) -> i32;
    fn SetCcNodeId(&self, sequence: i32, id: i32);
    fn Parent(&self) -> Option<Self>;
    fn ContainerRect(&self) -> Rect;
    fn GetCompositorElementId(&self) -> ElementId;
    fn ScrollingContentsCullRect(&self) -> Rect;
    fn ContentsRect(&self) -> Rect;
    fn UserScrollableHorizontal(&self) -> bool;
    fn UserScrollableVertical(&self) -> bool;
    fn PreventViewportScrollingFromInner(&self) -> bool;
    fn MaxScrollOffsetAffectedByPageScale(&self) -> bool;
    fn OverscrollBehavior(&self) -> OverscrollBehavior;
    fn GetSnapContainerData(&self) -> Option<SnapContainerData>;
    fn PreventScrollAxisLocking(&self) -> bool;
    fn GetMainThreadRepaintReasons(&self) -> MainThreadRepaintReasons;
    fn CompositedScrollingNotPreferred(&self) -> bool;
}

pub trait PropertyTreeManagerClient<B: PropertyTreeManagerTypes> {
    fn CreateOrReuseSynthesizedClipLayer(
        &mut self,
        clip: &B::ClipNode,
        transform: &B::TransformNode,
        needs_layer: bool,
        mask_isolation_id: &mut ElementId,
        mask_effect_id: &mut ElementId,
        layer_context: &mut super::main_layer::LayerContext<'_, B>,
    ) -> &mut crate::third_party::blink::renderer::platform::graphics::compositing::paint_artifact_compositor::SynthesizedClipRef<B>;

    fn NeedsCompositedScrolling(&self, scroll_translation: &B::TransformNode) -> bool;
    fn ShouldForceMainThreadRepaint(&self, scroll_translation: &B::TransformNode) -> bool;
}

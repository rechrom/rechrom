//! PAC borrows actual native node/item values and retained record allocations.
use super::*;
use crate::{adapter::{paint_artifact_compositor::*},runtime::types::{PaintSkiaBackend,NodeId}};
use crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache;
impl<R, B> ArtifactTransformNode<B> for TransformNode<R>
where
    R: PaintSkiaBackend,
    B: PaintArtifactCompositorTypes<
        TransformNode = Self,
        CompositingReasons = CpuCompositingReasons,
    >,
{
    fn HasDirectCompositingReasons(&self) -> bool {
        self.node().HasDirectCompositingReasons()
    }
    fn NearestDirectlyCompositedAncestor(&self) -> Option<Self> {
        WithTransformCache(self, |c| c.nearest_directly_composited_ancestor())
    }
    fn RequiresCompositingForFixedPositionOnly(&self) -> bool {
        self.node().RequiresCompositingForFixedPositionOnly()
    }
    fn RequiresCompositingForStickyPositionOnly(&self) -> bool {
        self.node().RequiresCompositingForStickyPositionOnly()
    }
    fn RequiresCompositingForFixedPosition(&self) -> bool {
        self.node().RequiresCompositingForFixedPosition()
    }
    fn RequiresCompositingForStickyPosition(&self) -> bool {
        self.node().RequiresCompositingForStickyPosition()
    }
    fn ScrollTranslationState(&self) -> Self {
        WithTransformCache(self, |c| c.scroll_translation_state())
    }
    fn ClearChangedToRoot(&self, s: i32) {
        super::clear_transform_changed(self.as_arc(), self.trees(), s)
    }
    fn DirectCompositingReasonsForDebugging(&self) -> CpuCompositingReasons {
        CpuCompositingReasons::from_source(&self.node().DirectCompositingReasonsForDebugging())
    }
}
impl<R, B> ArtifactEffectNode<B> for EffectNode<R>
where
    R: PaintSkiaBackend,
    B: PaintArtifactCompositorTypes<
            TransformNode = TransformNode<R>,
            ClipNode = ClipNode<R>,
            EffectNode = Self,
            CompositingReasons = CpuCompositingReasons,
        > + crate::commit::push_properties::PushPropertiesTypes<NodeId = NodeId>,
{
    fn CanvasChildId(&self) -> NodeId {
        NodeId(self.node().CanvasChildId())
    }
    fn HasCanvasChildState(&self) -> bool {
        self.node().HasCanvasChildState()
    }
    fn CanvasChildContentTransform(&self) -> TransformNode<R> {
        self.link_transform(self.node().CanvasChildContentTransform())
    }
    fn CanvasChildContentClip(&self) -> ClipNode<R> {
        self.link_clip(self.node().CanvasChildContentClip())
    }
    fn CanvasChildContentEffect(&self) -> Self {
        Self::from_arc(self.node().CanvasChildContentEffect().clone())
    }
    fn IsInDrawableCanvasSubtree(&self) -> bool {
        self.node().IsInDrawableCanvasSubtree()
    }
    fn DrawsContent(&self) -> bool {
        self.node().DrawsContent()
    }
    fn DirectCompositingReasonsForDebugging(&self) -> CpuCompositingReasons {
        CpuCompositingReasons::from_source(&self.node().direct_compositing_reasons)
    }
}
impl<R, B> ArtifactScrollNode<B> for ScrollNode<R>
where
    R: PaintSkiaBackend,
    B: PaintArtifactCompositorTypes,
{
    fn UserScrollable(&self) -> bool {
        read(|a| self.node().UserScrollable(a))
    }
    fn CompositedScrollingPreferred(&self) -> bool {
        read(|a| self.node().GetCompositedScrollingPreference(a))
            == paint_property_tree::PaintCompositedScrollingPreference::Preferred
    }
}
impl<R, B> ArtifactPropertyState<B> for PropertyTreeState<R>
where
    R: PaintSkiaBackend,
    B: PaintArtifactCompositorTypes<ClipNode = ClipNode<R>, EffectNode = EffectNode<R>>,
{
    fn SetEffectNode(&mut self, n: EffectNode<R>) {
        self.set_effect(n)
    }
    fn SetClipNode(&mut self, n: ClipNode<R>) {
        self.set_clip(n)
    }
    fn ClearChangedToRoot(&self, s: i32) {
        self.clear_changed_to_root(s)
    }
}
macro_rules! debug_node {
    ($t:ident) => {
        impl<R> ArtifactDebugNode for $t<R> {
            fn DebugPointer(&self) -> String {
                format!("{:p}", Arc::as_ptr(self.as_arc()))
            }
            fn DebugJSONString(&self) -> String {
                serde_json::Value::String(format!("{:?}", self.node())).to_string()
            }
        }
    };
}
impl<R> ArtifactDebugNode for TransformNode<R> {
    fn DebugPointer(&self) -> String {
        format!("{:p}", Arc::as_ptr(self.as_arc()))
    }
    fn DebugJSONString(&self) -> String {
        self.node().ToString()
    }
}
impl<R> ArtifactDebugNode for ClipNode<R> {
    fn DebugPointer(&self) -> String {
        format!("{:p}", Arc::as_ptr(self.as_arc()))
    }
    fn DebugJSONString(&self) -> String {
        self.node().ToString()
    }
}
impl<R> ArtifactDebugNode for EffectNode<R> {
    fn DebugPointer(&self) -> String {
        format!("{:p}", Arc::as_ptr(self.as_arc()))
    }
    fn DebugJSONString(&self) -> String {
        self.node().ToString()
    }
}
impl<R, B> ArtifactDisplayItem<B> for DisplayItem
where
    R: PaintSkiaBackend,
    B: PaintArtifactCompositorTypes<TransformNode = TransformNode<R>> + 'static,
{
    fn IsForeignLayer(&self) -> bool {
        let t = self.id.r#type.0;
        (paint::display_item_id::DisplayItemIdType::kForeignLayerFirst.0
            ..=paint::display_item_id::DisplayItemIdType::kForeignLayerLast.0)
            .contains(&t)
    }
    fn ForeignLayerBorrow(&self) -> crate::cc::layers::layer::LayerBorrow<B> {
        self.GetForeignLayer()
            .AsAny()
            .downcast_ref::<CpuForeignLayer<B>>()
            .expect("ForeignLayerDisplayItem owner must match the active compositor backend")
            .Layer()
            .borrow()
    }
    fn ScrollTranslation(&self) -> Option<TransformNode<R>> {
        self.scroll_translation
            .as_ref()
            .map(|n| TransformNode::from_arc(n.clone()))
    }
    fn GetType(&self) -> ArtifactDisplayItemType {
        use paint::display_item_id::DisplayItemIdType as T;
        use ArtifactDisplayItemType as D;
        let t = self.id.r#type;
        if t == T::kFixedAttachmentBackground {
            return D::FixedAttachmentBackground;
        }
        if t == T::kCaret {
            return D::Caret;
        }
        if t == T::kScrollbarHorizontal {
            return D::ScrollbarHorizontal;
        }
        if t == T::kScrollbarVertical {
            return D::ScrollbarVertical;
        }
        D::Other
    }
}
impl<R, B> ArtifactCanvasChildPaintRecord<B> for CpuCanvasChildPaintRecord<R>
where
    R: PaintSkiaBackend,
    B: PaintArtifactCompositorTypes<PaintRecord = CpuPaintRecord<R>>,
{
    fn record(&self) -> &CpuPaintRecord<R> {
        &self.record
    }
    fn record_mut(&mut self) -> &mut CpuPaintRecord<R> {
        &mut self.record
    }
}
/// Diagnostic JSON is an external native output owner, lent resident layers.
pub struct CpuLayersAsJson<B: PaintArtifactCompositorTypes>(pub crate::third_party::blink::renderer::platform::graphics::compositing::layers_as_json::LayersAsJSON<B>);
impl<B> ArtifactLayersAsJson<B> for CpuLayersAsJson<B>
where
    B: PaintArtifactCompositorTypes<
        Json = serde_json::Value,
        DerivedServices = dyn super::CpuPaintServices<B>,
    >,
{
    fn AddLayer(
        &mut self,
        layer: &crate::cc::layers::layer::LayerRef<B>,
        transform: &B::TransformNode,
        client:Option<&crate::third_party::blink::renderer::platform::graphics::compositing::content_layer_client_impl::ContentLayerClientRef<B>>,
        c: &crate::adapter::main_layer::LayerContext<'_, B>,
    ) {
        self.0.AddLayer(
            layer,
            transform,
            client,
            c,
            c.DerivedServices(),
            c.DerivedServices(),
        )
    }
    fn Finalize(self) -> serde_json::Value {
        self.0.Finalize()
    }
}
pub trait CpuCanvasSnapshot<R: PaintSkiaBackend> {
    fn Snapshot(&self, child: NodeId) -> Option<CpuPaintRecord<R>>;
}

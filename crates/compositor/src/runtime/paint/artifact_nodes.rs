//! PAC borrows actual native node/item values and retained record allocations.
use super::*;
use crate::{adapter::{paint_artifact_compositor::*,pending_layer::{PendingTransformNode,PendingEffectNode}},runtime::types::{CpuSkiaBackend,CpuCompositorTypes,NodeId}};
use crate::third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache;
impl<R, B> ArtifactTransformNode<B> for TransformNode<R>
where
    R: CpuSkiaBackend,
    B: PaintArtifactCompositorTypes<
        TransformNode = Self,
        CompositingReasons = CpuCompositingReasons,
    >,
{
    fn HasDirectCompositingReasons(&self) -> bool {
        !self.node().direct_compositing_reasons.is_empty()
    }
    fn NearestDirectlyCompositedAncestor(&self) -> Option<Self> {
        WithTransformCache(self, |c| c.nearest_directly_composited_ancestor())
    }
    fn RequiresCompositingForFixedPositionOnly(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"FixedPosition")
            && self.node().direct_compositing_reasons.iter().all(|r| {
                matches!(
                    *r,
                    "FixedPosition"
                        | "UndoOverscroll"
                        | "AffectedByOuterViewportBoundsDelta"
                        | "AffectedBySafeAreaBottom"
                )
            })
    }
    fn RequiresCompositingForStickyPositionOnly(&self) -> bool {
        self.node().direct_compositing_reasons.as_slice() == ["StickyPosition"]
    }
    fn RequiresCompositingForFixedPosition(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"FixedPosition")
    }
    fn RequiresCompositingForStickyPosition(&self) -> bool {
        self.node()
            .direct_compositing_reasons
            .contains(&"StickyPosition")
    }
    fn ScrollTranslationState(&self) -> Self {
        WithTransformCache(self, |c| c.scroll_translation_state())
    }
    fn ClearChangedToRoot(&self, s: i32) {
        super::clear_transform_changed(self.as_arc(), s)
    }
    fn DirectCompositingReasonsForDebugging(&self) -> CpuCompositingReasons {
        CpuCompositingReasons::from_native(&self.node().direct_compositing_reasons)
    }
}
impl<R: CpuSkiaBackend> ArtifactEffectNode<CpuCompositorTypes<R>> for EffectNode<R> {
    fn CanvasChildId(&self) -> NodeId {
        NodeId(
            self.node()
                .compositor
                .canvas_child_state
                .as_ref()
                .map_or(0, |s| s.id),
        )
    }
    fn HasCanvasChildState(&self) -> bool {
        self.node()
            .compositor
            .canvas_child_state
            .as_ref()
            .is_some_and(|s| s.id != 0)
    }
    fn CanvasChildContentTransform(&self) -> TransformNode<R> {
        TransformNode::from_arc(
            self.node()
                .compositor
                .canvas_child_state
                .as_ref()
                .expect("source HasCanvasChildState")
                .content_state
                .transform
                .clone(),
        )
    }
    fn CanvasChildContentClip(&self) -> ClipNode<R> {
        ClipNode::from_arc(
            self.node()
                .compositor
                .canvas_child_state
                .as_ref()
                .expect("source HasCanvasChildState")
                .content_state
                .clip
                .clone(),
        )
    }
    fn CanvasChildContentEffect(&self) -> Self {
        Self::from_arc(
            self.node()
                .compositor
                .canvas_child_state
                .as_ref()
                .expect("source HasCanvasChildState")
                .content_state
                .effect
                .clone(),
        )
    }
    fn IsInDrawableCanvasSubtree(&self) -> bool {
        self.node().compositor.is_in_drawable_canvas_subtree
    }
    fn DrawsContent(&self) -> bool {
        self.HasFilter()
            || self.HasActiveFilterAnimation()
            || self.RequiresCompositingForWillChangeFilter()
            || self.MayHaveBackdropEffect()
            || self.HasValidViewTransitionElementResourceId()
            || self.HasElementCaptureId()
    }
    fn DirectCompositingReasonsForDebugging(&self) -> CpuCompositingReasons {
        CpuCompositingReasons::from_native(&self.node().direct_compositing_reasons)
    }
}
impl<R, B> ArtifactScrollNode<B> for ScrollNode<R>
where
    R: CpuSkiaBackend,
    B: PaintArtifactCompositorTypes,
{
    fn UserScrollable(&self) -> bool {
        self.node().user_scrollable_horizontal || self.node().user_scrollable_vertical
    }
    fn CompositedScrollingPreferred(&self) -> bool {
        self.node().compositor.composited_scrolling_preference
            == native::PaintCompositedScrollingPreference::Preferred
    }
}
impl<R, B> ArtifactPropertyState<B> for PropertyTreeState<R>
where
    R: CpuSkiaBackend,
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
debug_node!(TransformNode);
debug_node!(ClipNode);
debug_node!(EffectNode);
impl<R, B> ArtifactDisplayItem<B> for DisplayItem
where
    R: CpuSkiaBackend,
    B: PaintArtifactCompositorTypes<TransformNode = TransformNode<R>>,
{
    fn IsForeignLayer(&self) -> bool {
        let t = self.id.r#type.0;
        (paint::display_item_id::DisplayItemIdType::kForeignLayerFirst.0
            ..=paint::display_item_id::DisplayItemIdType::kForeignLayerLast.0)
            .contains(&t)
    }
    fn ForeignLayerBorrow(&self) -> crate::cc::layers::layer::LayerBorrow<B> {
        unreachable!("PaintInput rejects absent native foreign layer owner")
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
    R: CpuSkiaBackend,
    B: PaintArtifactCompositorTypes<PaintRecord = CpuPaintRecord<R>>,
{
    fn record(&self) -> &CpuPaintRecord<R> {
        &self.record
    }
    fn record_mut(&mut self) -> &mut CpuPaintRecord<R> {
        &mut self.record
    }
}
impl<R: CpuSkiaBackend> ArtifactPaintRecord<CpuCompositorTypes<R>> for CpuPaintRecord<R> {
    fn ops(&self) -> impl Iterator<Item = ArtifactRecordOp<'_, CpuCompositorTypes<R>>> {
        use crate::adapter::display_item_list::DisplayPaintOpBuffer;
        self.buffer()
            .into_iter()
            .flat_map(|b| b.iter())
            .map(|op| match op {
                CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => {
                    ArtifactRecordOp::DrawRecord(r)
                }
                _ => ArtifactRecordOp::Other,
            })
    }
    fn ReplaceCustomData(&self, replacements: &std::collections::BTreeMap<u32, Self>) -> Self {
        assert!(
            replacements.is_empty(),
            "PaintInput rejects unrepresented native CustomData paint operations"
        );
        self.clone()
    }
}
/// Diagnostic JSON is an external native output owner, lent resident layers.
pub struct CpuLayersAsJson<R:CpuSkiaBackend>(pub crate::third_party::blink::renderer::platform::graphics::compositing::layers_as_json::LayersAsJSON<CpuCompositorTypes<R>>);
impl<R: CpuSkiaBackend> ArtifactLayersAsJson<CpuCompositorTypes<R>> for CpuLayersAsJson<R> {
    fn AddLayer(
        &mut self,
        layer: &crate::cc::layers::layer::LayerRef<CpuCompositorTypes<R>>,
        transform: &TransformNode<R>,
        client:Option<&crate::third_party::blink::renderer::platform::graphics::compositing::content_layer_client_impl::ContentLayerClientRef<CpuCompositorTypes<R>>>,
        c: &crate::adapter::main_layer::LayerContext<'_, CpuCompositorTypes<R>>,
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
pub trait CpuCanvasSnapshot<R: CpuSkiaBackend> {
    fn Snapshot(&self, child: NodeId) -> Option<CpuPaintRecord<R>>;
}

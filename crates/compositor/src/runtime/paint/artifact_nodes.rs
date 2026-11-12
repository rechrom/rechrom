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
        super::clear_transform_changed(self.as_arc(), s)
    }
    fn DirectCompositingReasonsForDebugging(&self) -> CpuCompositingReasons {
        CpuCompositingReasons::from_source(&self.node().DirectCompositingReasonsForDebugging())
    }
}
impl<R: CpuSkiaBackend> ArtifactEffectNode<CpuCompositorTypes<R>> for EffectNode<R> {
    fn CanvasChildId(&self) -> NodeId {
        NodeId(self.node().CanvasChildId())
    }
    fn HasCanvasChildState(&self) -> bool {
        self.node().HasCanvasChildState()
    }
    fn CanvasChildContentTransform(&self) -> TransformNode<R> {
        TransformNode::from_arc(self.node().CanvasChildContentTransform().clone())
    }
    fn CanvasChildContentClip(&self) -> ClipNode<R> {
        ClipNode::from_arc(self.node().CanvasChildContentClip().clone())
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
    R: CpuSkiaBackend,
    B: PaintArtifactCompositorTypes,
{
    fn UserScrollable(&self) -> bool {
        self.node().UserScrollable()
    }
    fn CompositedScrollingPreferred(&self) -> bool {
        self.node().GetCompositedScrollingPreference()
            == paint_property_tree::PaintCompositedScrollingPreference::Preferred
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
impl<R: CpuSkiaBackend> ArtifactDisplayItem<CpuCompositorTypes<R>> for DisplayItem {
    fn IsForeignLayer(&self) -> bool {
        let t = self.id.r#type.0;
        (paint::display_item_id::DisplayItemIdType::kForeignLayerFirst.0
            ..=paint::display_item_id::DisplayItemIdType::kForeignLayerLast.0)
            .contains(&t)
    }
    fn ForeignLayerBorrow(&self) -> crate::cc::layers::layer::LayerBorrow<CpuCompositorTypes<R>> {
        self.GetForeignLayer()
            .AsAny()
            .downcast_ref::<CpuForeignLayer<R>>()
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
                CpuPaintOpRef::PaintRecord { op, .. }
                    if op.r#type == paint::paint_engine::DisplayItemType::kCustomData =>
                {
                    ArtifactRecordOp::CustomData(op.custom_data_id)
                }
                CpuPaintOpRef::Converted(CpuPaintOp::CustomData(id)) => {
                    ArtifactRecordOp::CustomData(*id)
                }
                CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => {
                    ArtifactRecordOp::DrawRecord(r)
                }
                _ => ArtifactRecordOp::Other,
            })
    }
    fn ReplaceCustomData(&self, replacements: &std::collections::BTreeMap<u32, Self>) -> Self {
        use crate::adapter::display_item_list::DisplayPaintOpBuffer;
        if replacements.is_empty() {
            return self.clone();
        }
        let Some(source) = self.buffer() else {
            return self.clone();
        };
        let mut output = CpuPaintOpBuffer::new(source.values().clone());
        for (index, op) in source.iter().enumerate() {
            let custom_data_id = match op {
                CpuPaintOpRef::PaintRecord { op, .. }
                    if op.r#type == paint::paint_engine::DisplayItemType::kCustomData =>
                {
                    Some(op.custom_data_id)
                }
                CpuPaintOpRef::Converted(CpuPaintOp::CustomData(id)) => Some(*id),
                _ => None,
            };
            if let Some(id) = custom_data_id {
                if let Some(replacement) = replacements.get(&id) {
                    output.push(CpuPaintOp::DrawRecord(replacement.clone()));
                } else {
                    output.push(CpuPaintOp::CustomData(id));
                }
                continue;
            }
            match op {
                CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(record)) => {
                    output.push(CpuPaintOp::DrawRecord(
                        record.ReplaceCustomData(replacements),
                    ));
                }
                _ => {
                    output.push(source.RetainOpAt(index));
                }
            }
        }
        output.ReleaseAsRecord()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        adapter::{
            display_item_list::DisplayPaintOpBuffer,
            paint_artifact_compositor::{ArtifactPaintRecord, ArtifactRecordOp},
        },
        runtime::{
            paint::{CpuPaintOp, CpuPaintOpBuffer, CpuPaintRecord, SkiaPaintDiagnostics},
            paint_services::{CpuPaintFeatures, CpuPaintServices},
            skia::SkRegion,
            types::CpuCompositorTypes,
        },
    };
    use std::{collections::BTreeMap, sync::Arc};

    fn values() -> Arc<dyn CpuPaintValueServices<SkRegion>> {
        CpuPaintServices::new_cpu(
            CpuPaintFeatures {
                cc_slimming: false,
                singular_transform_fix: false,
                merge_fixed_layers: false,
                merge_sticky_layers: false,
                raster_inducing_scroll: false,
                scrolling_contents_cull_rect: false,
                remove_scroll_node_workaround: false,
                selection_edge_visibility_uses_full_edge: false,
                canvas_draw_element: false,
                expensive_dchecks: false,
                paint_under_invalidation_checking: false,
                layer_merge_distance_limit: 0,
            },
            SkiaPaintDiagnostics {
                trace: false,
                verbose_level: 0,
                histogram: false,
            },
        )
        .expect("CPU Paint services")
        .PaintValues()
    }

    fn ops(
        record: &CpuPaintRecord<SkRegion>,
    ) -> impl Iterator<Item = ArtifactRecordOp<'_, CpuCompositorTypes<SkRegion>>> {
        <CpuPaintRecord<SkRegion> as ArtifactPaintRecord<CpuCompositorTypes<SkRegion>>>::ops(record)
    }

    #[test]
    fn replaces_custom_data_in_outer_and_nested_records() {
        let values = values();
        let mut replacement = CpuPaintOpBuffer::new(values.clone());
        replacement.push(CpuPaintOp::Noop);
        let replacement = replacement.ReleaseAsRecord();

        let mut nested = CpuPaintOpBuffer::new(values.clone());
        nested.push(CpuPaintOp::CustomData(7));
        let nested = nested.ReleaseAsRecord();

        let mut source = CpuPaintOpBuffer::new(values);
        source.push(CpuPaintOp::CustomData(7));
        source.push(CpuPaintOp::CustomData(8));
        source.push(CpuPaintOp::DrawRecord(nested));
        let source = source.ReleaseAsRecord();

        let replacements = BTreeMap::from([(7, replacement)]);
        let replaced = <CpuPaintRecord<SkRegion> as ArtifactPaintRecord<
            CpuCompositorTypes<SkRegion>,
        >>::ReplaceCustomData(&source, &replacements);

        let mut outer_ops = ops(&replaced);
        assert!(matches!(
            outer_ops.next(),
            Some(ArtifactRecordOp::DrawRecord(_))
        ));
        assert!(matches!(
            outer_ops.next(),
            Some(ArtifactRecordOp::CustomData(8))
        ));
        let nested = match outer_ops.next() {
            Some(ArtifactRecordOp::DrawRecord(record)) => record,
            _ => panic!("recursive DrawRecord replacement"),
        };
        assert!(outer_ops.next().is_none());
        assert!(matches!(
            ops(nested).next(),
            Some(ArtifactRecordOp::DrawRecord(_))
        ));
    }
}

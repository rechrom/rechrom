//! Native value/factory capabilities for PaintArtifactCompositor and its
//! synthesized clip client. No compositor owner, tree or layer is stored here.
use super::{
    geometry::{Rect, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::LayerTreeTypes,
    paint_chunks_to_cc_layer::PaintChunksTypes,
    paint_geometry::ChunkGeometryMapper,
};
/// This small configuration is independent of the PAC runtime owner, allowing
/// PropertyTreeManager to reference the exact canonical SynthesizedClip type.
pub trait SynthesizedClipTypes:
    PaintChunksTypes<
    Transform: super::paint_chunks_to_cc_layer::ConversionTransform,
    ChunkMapper: ChunkGeometryMapper<Transform = <Self as LayerTreeTypes>::Transform>,
    PaintPath: Send + Sync,
    DerivedServices: SynthesizedClipPaintServices<Self>,
>
{
    type SkRoundedRect: Clone + Default + PartialEq + Send + Sync;
}
/// Only construction/value/paint-op append primitives of the external backend.
/// SynthesizedClip's source branches and ordering remain in its canonical stem.
pub trait SynthesizedClipPaintServices<B: SynthesizedClipTypes> {
    fn CreateDisplayItemList(&self) -> B::ContentDisplayItemList;
    fn NewPaintFlags(&self) -> B::PaintFlags;
    fn SetAntiAlias(&self, flags: &mut B::PaintFlags, value: bool);
    fn PushDrawRRect(
        &self,
        list: &mut B::ContentDisplayItemList,
        rect: &B::SkRoundedRect,
        flags: B::PaintFlags,
    );
}
pub trait SynthesizedClipServices<B: SynthesizedClipTypes>:
    super::paint_chunks_to_cc_layer::PaintChunksServices<B>
{
    fn NewUniqueObjectElementId(&self) -> ElementId;
    fn Projection(
        &self,
        source: &B::TransformNode,
        destination: &B::TransformNode,
    ) -> <B as LayerTreeTypes>::Transform;
    fn ToSkRoundedRect(&self, rect: &B::RoundedRect) -> B::SkRoundedRect;
    fn OffsetSkRoundedRect(&self, rect: &mut B::SkRoundedRect, offset: Vector2dF);
    fn MapRect(&self, transform: &<B as LayerTreeTypes>::Transform, rect: Rect) -> Rect;
    fn PostTranslate(&self, transform: &mut <B as LayerTreeTypes>::Transform, offset: Vector2dF);
    fn MakeIdentity(&self, transform: &mut <B as LayerTreeTypes>::Transform);
    fn ClipPathEquals(&self, clip: &B::ClipNode, previous: &Option<B::PaintPath>) -> bool;
}

use super::{
    main_layer::{LayerContext, MainLayerTypes},
    pending_layer::{PendingLayerGeometry, PendingLayerJson, PendingLayerServices},
    property_tree_manager::PropertyTreeManagerTypes,
};
use crate::cc::layers::layer::{LayerDebugInfo, LayerRef};
use std::{collections::BTreeMap, hash::Hash};

pub trait PaintArtifactCompositorTypes:
    PropertyTreeManagerTypes<
    DomNodeId = <Self as MainLayerTypes>::NodeId,
    NodeId: Eq + Hash,
    ScrollNode: Eq + Hash + ArtifactScrollNode<Self>,
    TransformNode: ArtifactTransformNode<Self> + ArtifactDebugNode,
    EffectNode: ArtifactEffectNode<Self> + ArtifactDebugNode,
    ClipNode: ArtifactDebugNode,
    PropertyTreeState: ArtifactPropertyState<Self>,
    PaintChunk: ArtifactPaintChunk<Self>,
    PaintChunkSubset: ArtifactChunkSubset,
    PaintRecord: ArtifactPaintRecord<Self>,
    CanvasChildPaintRecord: ArtifactCanvasChildPaintRecord<Self>,
    DisplayItem: ArtifactDisplayItem<Self>,
    RasterInvalidationTracking: ArtifactInvalidationTracking<Self>,
>
{
    type CanvasSnapshotCallback;
    type CompositingReasons: CompositingReasons;
    type LayersAsJson: ArtifactLayersAsJson<Self>;
    type CrashKeyScope;
    type UpdateTraceScope;
}
pub trait ArtifactTransformNode<B: PaintArtifactCompositorTypes> {
    fn HasDirectCompositingReasons(&self) -> bool;
    fn NearestDirectlyCompositedAncestor(&self) -> Option<B::TransformNode>;
    fn RequiresCompositingForFixedPositionOnly(&self) -> bool;
    fn RequiresCompositingForStickyPositionOnly(&self) -> bool;
    fn RequiresCompositingForFixedPosition(&self) -> bool;
    fn RequiresCompositingForStickyPosition(&self) -> bool;
    fn ScrollTranslationState(&self) -> B::TransformNode;
    fn ClearChangedToRoot(&self, sequence: i32);
    fn DirectCompositingReasonsForDebugging(&self) -> B::CompositingReasons;
}
pub trait ArtifactEffectNode<B: PaintArtifactCompositorTypes> {
    fn CanvasChildId(&self) -> B::DomNodeId;
    fn HasCanvasChildState(&self) -> bool;
    fn CanvasChildContentTransform(&self) -> B::TransformNode;
    fn CanvasChildContentClip(&self) -> B::ClipNode;
    fn CanvasChildContentEffect(&self) -> B::EffectNode;
    fn IsInDrawableCanvasSubtree(&self) -> bool;
    fn DrawsContent(&self) -> bool;
    fn DirectCompositingReasonsForDebugging(&self) -> B::CompositingReasons;
}
pub trait ArtifactScrollNode<B: PaintArtifactCompositorTypes> {
    fn UserScrollable(&self) -> bool;
    fn CompositedScrollingPreferred(&self) -> bool;
}
pub trait ArtifactPropertyState<B: PaintArtifactCompositorTypes> {
    fn SetEffectNode(&mut self, effect: B::EffectNode);
    fn SetClipNode(&mut self, clip: B::ClipNode);
    fn ClearChangedToRoot(&self, sequence: i32);
}
pub trait ArtifactPaintChunk<B: PaintArtifactCompositorTypes> {
    fn CanMatchOldChunk(&self) -> bool;
    fn SameRawTransform(&self, other: &B::PaintChunk) -> bool;
    fn UnaliasedTransform(&self) -> B::TransformNode;
    fn UnaliasedClip(&self) -> B::ClipNode;
    fn UnaliasedEffect(&self) -> B::EffectNode;
    fn opaque_rect(&self) -> Rect;
    fn SameRawProperties(&self, other: &B::PaintChunk) -> bool;
    fn IdToString(&self) -> String;
    fn RawTransformForDebug(&self) -> B::TransformNode;
    fn RawClipForDebug(&self) -> B::ClipNode;
    fn RawEffectForDebug(&self) -> B::EffectNode;
}
pub trait ArtifactChunkSubset {
    fn ApproximateUnsharedMemoryUsage(&self) -> usize;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArtifactDisplayItemType {
    FixedAttachmentBackground,
    Caret,
    ScrollbarHorizontal,
    ScrollbarVertical,
    ForeignLayerCanvas,
    ForeignLayerDevToolsOverlay,
    ForeignLayerPlugin,
    ForeignLayerVideo,
    ForeignLayerRemoteFrame,
    ForeignLayerLinkHighlight,
    ForeignLayerViewportScroll,
    ForeignLayerViewportScrollbar,
    ForeignLayerViewTransitionContent,
    Other,
}
pub trait ArtifactDisplayItem<B: PaintArtifactCompositorTypes> {
    fn IsForeignLayer(&self) -> bool;
    fn ForeignLayerBorrow(&self) -> crate::cc::layers::layer::LayerBorrow<B>;
    fn ScrollTranslation(&self) -> Option<B::TransformNode>;
    fn GetType(&self) -> ArtifactDisplayItemType;
}
pub use compositor_layerize::{CompositingReason, CompositingReasons};
pub trait ArtifactInvalidationTracking<B: PaintArtifactCompositorTypes> {
    fn AddToLayerDebugInfo(&self, info: &mut LayerDebugInfo<B::NodeId>);
    fn ClearInvalidations(&mut self);
}
pub trait ArtifactLayersAsJson<B: PaintArtifactCompositorTypes> {
    fn AddLayer(
        &mut self,
        layer: &LayerRef<B>,
        transform: &B::TransformNode,
        client: Option<&crate::third_party::blink::renderer::platform::graphics::compositing::content_layer_client_impl::ContentLayerClientRef<B>>,
        c: &LayerContext<'_, B>,
    );
    fn Finalize(self) -> B::Json;
}

pub enum ArtifactRecordOp<'a, B: PaintArtifactCompositorTypes> {
    CustomData(u32),
    DrawRecord(&'a B::PaintRecord),
    Other,
}
pub trait ArtifactPaintRecord<B: PaintArtifactCompositorTypes>: Sized {
    fn ops(&self) -> impl Iterator<Item = ArtifactRecordOp<'_, B>>;
    fn ReplaceCustomData(&self, replacements: &BTreeMap<u32, B::PaintRecord>) -> Self;
}
pub trait ArtifactCanvasChildPaintRecord<B: PaintArtifactCompositorTypes> {
    fn record(&self) -> &B::PaintRecord;
    fn record_mut(&mut self) -> &mut B::PaintRecord;
}
pub trait PaintArtifactCompositorServices<B: PaintArtifactCompositorTypes>:
    SynthesizedClipServices<B> + PendingLayerGeometry<B>
{
    fn VlogIsOn(&self, level: i32) -> bool;
    fn NullPointerForDebug(&self) -> String;
    fn CrashKey(&self, name: &str, section: &str, value: String) -> B::CrashKeyScope;
    fn DumpWithoutCrashing(&self);
    fn CanvasDrawElementEnabledByRuntimeFlag(&self) -> bool;
    fn LayerMergeDistanceLimit(&self) -> i32;
    /// Source PropertyTreeManager's static UniqueObjectId, retained by the
    /// embedding as one native identity; no layerization/state policy.
    fn RootEffectElementId(&self) -> ElementId;
    fn CompositorElementIdFromDomNodeId(&self, id: B::DomNodeId) -> ElementId;
    fn NewUpdateTrace(&self) -> B::UpdateTraceScope;
    fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool;
    fn HistogramCounts100(&self, name: &str, value: i32);
    fn CanvasSnapshot(
        &self,
        callback: &B::CanvasSnapshotCallback,
        child: B::DomNodeId,
    ) -> Option<B::PaintRecord>;
    fn DomNodeIdFromCustomData(&self, id: u32) -> B::DomNodeId;
    fn AppendPendingLayerJson(&self, array: &mut B::JsonArray, layer: PendingLayerJson);
    fn NewLayersAsJson(
        &self,
        flags: crate::third_party::blink::renderer::platform::graphics::compositing::content_layer_client_impl::LayerTreeFlags,
    ) -> B::LayersAsJson;
    fn Descriptions(&self, reasons: &B::CompositingReasons) -> Vec<&'static str>;
    fn ShortNames(&self, reasons: &B::CompositingReasons) -> Vec<&'static str>;
    fn IsTracingRasterInvalidations(&self) -> bool;
    fn IsScrollbarElementId(&self, id: ElementId) -> bool;
    fn DebugShowLayers(&self, json: B::Json);
    fn DebugVlogLayers(&self, event: &str, json: B::Json);
}
/// Source raw pointers and global sequence value are explicit live borrows for
/// one call. The sequence is initialized by the embedding to native value 1;
/// it is not a Page/host/tree snapshot or an owned service backpointer.
pub struct PaintArtifactCompositorContext<'a, 'c, B: PaintArtifactCompositorTypes> {
    pub layer: &'a mut LayerContext<'c, B>,
    pub pending: &'a mut dyn PendingLayerServices<B>,
    pub services: &'a dyn PaintArtifactCompositorServices<B>,
    pub property_tree_sequence_number: &'a mut i32,
}

/// Native debug-only node identity and ToJSON output. Returned text is the
/// source diagnostic value, never a replacement for the resident node state.
pub trait ArtifactDebugNode {
    fn DebugPointer(&self) -> String;
    fn DebugJSONString(&self) -> String;
}

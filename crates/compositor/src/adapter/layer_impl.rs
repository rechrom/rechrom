//! Live tree siblings and external Viz/gfx/tracing capabilities for LayerImpl.
//! The resident base stores only LayerTreeId. These borrows never escape a call.
use super::{
    frame_data::FrameDataTypes,
    geometry::{Rect, Size, Vector2dF},
    input_types::ElementId,
    layer_tree_impl::{LayerTreeId, PropertyTreesFor, ResourceId},
    main_layer::{MainLayerImplTree, MainLayerTypes},
    occlusion::SkBlendMode,
    property_types::MaskFilterInfo,
};
use crate::cc::{
    debug::layer_tree_debug_state::LayerTreeDebugState, layers::layer_impl::LayerImpl,
    trees::layer_tree_settings::LayerTreeSettings,
};
/// Native generated cc::mojom::LayerType identity (cc/mojom/layer_type.mojom:8-24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerType {
    kLayer,
    kSolidColor,
    kTexture,
    kSurface,
    kPicture,
    kTileDisplay,
    kMirror,
    kHeadsUpDisplay,
    kUIResource,
    kNinePatch,
    kSolidColorScrollbar,
    kPaintedScrollbar,
    kNinePatchThumbScrollbar,
    kVideo,
    kViewTransitionContent,
}
pub trait LayerImplTypes:
    MainLayerTypes<
        NodeId: LayerImplNodeId,
        Transform: LayerImplTransform,
        Layer: LayerImplOwner<Self>,
    > + FrameDataTypes<ResourceId = ResourceId, RenderPass: LayerImplRenderPass<Self>>
{
    type SharedQuadState: LayerImplSharedQuadState<Self::Transform>;
    type DrawQuad: LayerImplDrawQuad;
    type DeletionTrace: LayerImplDeletionTrace;
    /// Pure injection into the binding's unique most-derived owner/enum.
    fn WrapLayerImpl(layer: LayerImpl<Self>) -> Self::Layer;
    /// Pure synchronous borrow factory. Split only the actual LTI/host/services
    /// fields; never cache any reference, move a tree, or implement decisions.
    /// All constructors and Sync dispatch use this same live context boundary.
    fn WithLayerImplContext<Output>(
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<Self>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, Self>,
        callback: impl FnOnce(LayerTreeId, &mut LayerImplContext<'_, Self>) -> Output,
    ) -> Output;
}
/// Source cc/layers/layer_impl.cc:133-137 TRACE_EVENT_INSTANT/TerminatingFlow.
/// This capability may retain only the external trace sink and native IDs. It
/// must not hold Layer/tree/host or expose a general logger/owner lookup.
pub trait LayerImplDeletionTrace {
    fn LayerDeleted(&self, layer_id: i32, stable_id: i32);
}
pub trait LayerImplReadTree<B: LayerImplTypes>: MainLayerImplTree<B> {
    fn id(&self) -> LayerTreeId;
    fn IsActiveTree(&self) -> bool;
    fn TreesInVizInClientProcess(&self) -> bool;
    fn IsCCSlimmingEnabled(&self) -> bool;
    fn property_trees(&self) -> &PropertyTreesFor<B>;
    fn settings(&self) -> &LayerTreeSettings;
    fn debug_state(&self) -> &LayerTreeDebugState;
    fn device_scale_factor(&self) -> f32;
    fn max_texture_size(&self) -> i32;
    fn current_page_scale_factor(&self) -> f32;
    fn LifecycleAllowsPropertyTreeAccess(&self) -> bool;
}
pub trait LayerImplTree<B: LayerImplTypes>: LayerImplReadTree<B> {
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B>;
    fn AddLayerShouldPushProperties(&mut self, layer_id: i32);
    fn set_needs_update_draw_properties(&mut self);
}
/// ScrollTree callbacks also need the actual selected receiver and remaining
/// storage. Picture/property-only contexts do not claim this unrelated ability.
pub trait LayerImplScrollTree<B: LayerImplTypes>: LayerImplTree<B> {
    fn ScrollTreeScrollBy(
        &mut self,
        current: &mut dyn super::layer_tree_impl::LayerTreeCurrentReceiver<B>,
        node_id: i32,
        scroll: Vector2dF,
    ) -> Vector2dF;
    fn DidUpdateScrollOffset(
        &mut self,
        current: &mut dyn super::layer_tree_impl::LayerTreeCurrentReceiver<B>,
        element: ElementId,
        pushed: bool,
    );
}
pub struct LayerImplScrollContext<'a, B: LayerImplTypes> {
    pub tree: &'a mut dyn LayerImplScrollTree<B>,
}
/// Only the source's live readonly tree queries. No service or mutable owner
/// is lent when native CopyPropertiesTo reads its source allocation.
pub struct LayerImplReadContext<'a, B: LayerImplTypes> {
    pub tree: &'a dyn LayerImplReadTree<B>,
}
pub trait LayerImplServices<B: LayerImplTypes>:
    super::tile_based_layer_impl::TileBasedTrace
{
    fn NextStableIdSequenceValue(&self) -> i32;
    fn DeletionTrace(&self) -> B::DeletionTrace;
    fn TraceElement(&self, event: &str, element: ElementId);
    fn ValidateResource(&self, resource: ResourceId);
    fn NewTracedValueJSON(&self) -> B::LayerTraceValue;
    fn ScaleToCeiledSize(&self, size: Size, scale: f32) -> Size;
    fn TryComputeTransform2dScaleComponents(&self, transform: &B::Transform) -> Option<Vector2dF>;
    fn ComputeApproximateMaxScale(&self, transform: &B::Transform) -> f32;
}
pub struct LayerImplContext<'a, B: LayerImplTypes> {
    pub tree: &'a mut dyn LayerImplTree<B>,
    pub services: &'a mut dyn LayerImplServices<B>,
}
pub trait LayerImplSharedQuadState<T> {
    fn opacity(&self) -> f32;
    fn SetAll(
        &mut self,
        transform: &T,
        rect: Rect,
        visible: Rect,
        mask: &MaskFilterInfo,
        clip: Option<Rect>,
        opaque: bool,
        opacity: f32,
        blend: SkBlendMode,
        sorting_context_id: i32,
        layer_id: u32,
        fast_rounded_corner: bool,
    );
}
pub trait LayerImplDrawQuad {
    fn resource_id(&self) -> ResourceId;
}
pub trait LayerImplRenderPass<B: LayerImplTypes> {
    fn AppendDebugBorderQuad(
        &mut self,
        shared: B::SharedQuadStateHandle,
        rect: Rect,
        visible: Rect,
        color: super::layer_tree_impl::Color4f,
        width: f32,
    );
}
/// Native NodeId's integer trace operations (no layer lookup).
pub trait LayerImplNodeId {
    fn IsNonzero(&self) -> bool;
    fn AsInteger(&self) -> i32;
}
pub trait LayerImplTraceValue: super::property_types::PropertyTrace {
    fn MakeDictIntoImplicitSnapshotWithCategory(
        &mut self,
        category: &str,
        base_type: &str,
        layer_type: &str,
        stable_id: i32,
    );
    fn ElementIdInto(&mut self, element: ElementId);
    fn ToFormattedJSON(&self) -> String;
}
pub trait LayerImplBenchmark<B: LayerImplTypes> {
    fn RunOnLayer(&mut self, layer: &mut LayerImpl<B>, c: &mut LayerImplContext<'_, B>);
}

/// External gfx::Transform identity query consumed by native tracing only.
pub trait LayerImplTransform: super::layer_tree_impl::LayerTreeTransform {
    fn IsIdentity(&self) -> bool;
}

/// Native LayerImpl* points to the actual most-derived owned layer. This pure
/// same-package projection exposes its resident base without an owner lookup.
pub trait LayerImplOwner<B: LayerImplTypes> {
    fn impl_base(&self) -> &LayerImpl<B>;
    fn impl_base_mut(&mut self) -> &mut LayerImpl<B>;
}
impl<B: LayerImplTypes> LayerImplOwner<B> for LayerImpl<B> {
    fn impl_base(&self) -> &LayerImpl<B> {
        self
    }
    fn impl_base_mut(&mut self) -> &mut LayerImpl<B> {
        self
    }
}

// The concrete PushDirtyProperties target context is only the current live
// tree/services split. This projection returns the source frame number directly
// from the same resident target tree, without retaining an owner reference.
impl<B: LayerImplTypes> LayerImplContext<'_, B> {
    pub fn ReadContext(&self) -> LayerImplReadContext<'_, B> {
        LayerImplReadContext { tree: self.tree }
    }
}
impl<B: LayerImplTypes> MainLayerImplTree<B> for LayerImplContext<'_, B> {
    fn source_frame_number(&self) -> i32 {
        self.tree.source_frame_number()
    }
}

/// Only the resident bookkeeping touched by source SetNeedsPushProperties and
/// NoteLayerPropertyChanged. ScrollTree callbacks can borrow these disjoint
/// leaves while the same property's ScrollTree itself is executing.
pub trait LayerImplPushContext {
    type Types: super::layer_tree_impl::LayerTreeTypes;
    fn id(&self) -> LayerTreeId;
    fn IsActiveTree(&self) -> bool;
    fn TreesInVizInClientProcess(&self) -> bool;
    fn IsCCSlimmingEnabled(&self) -> bool;
    fn AddLayerShouldPushProperties(&mut self, id: i32);
    fn set_needs_update_draw_properties(&mut self);
}
impl<B: LayerImplTypes> LayerImplPushContext for LayerImplContext<'_, B> {
    type Types = B;
    fn id(&self) -> LayerTreeId {
        self.tree.id()
    }
    fn IsActiveTree(&self) -> bool {
        self.tree.IsActiveTree()
    }
    fn TreesInVizInClientProcess(&self) -> bool {
        self.tree.TreesInVizInClientProcess()
    }
    fn IsCCSlimmingEnabled(&self) -> bool {
        self.tree.IsCCSlimmingEnabled()
    }
    fn AddLayerShouldPushProperties(&mut self, id: i32) {
        self.tree.AddLayerShouldPushProperties(id)
    }
    fn set_needs_update_draw_properties(&mut self) {
        self.tree.set_needs_update_draw_properties()
    }
}

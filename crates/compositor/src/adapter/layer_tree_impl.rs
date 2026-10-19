//! Typed live services owned outside cc/trees/layer_tree_impl. No tree, layer,
//! property-node or draw-result snapshots are constructed by these contracts.
use super::{
    geometry::{PointF, Rect, RectF, Size},
    input_types::ElementId,
    property_tree::{PropertyTreeCopyOutputRequest, PropertyTreeRenderSurface},
    property_types::{PropertyPath, ViewTransitionElementResourceId},
    region_backend::SkRegionBackend,
};
use crate::cc::{
    input::{
        event_listener_properties::{EventListenerClass, EventListenerProperties},
        scrollbar::ScrollbarOrientation,
    },
    trees::{
        layer_tree_impl::LayerTreeImpl, property_tree::PropertyTrees,
        viewport_property_ids::ViewportPropertyIds,
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerTreeId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerTreeHostId(pub u64);
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color4f {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnguessableToken(pub [u64; 2]);
impl UnguessableToken {
    pub fn is_empty(self) -> bool {
        self.0 == [0, 0]
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayTransform {
    #[default]
    None,
    FlipHorizontal,
    FlipVertical,
    Rotate90,
    Rotate180,
    Rotate270,
    FlipVerticalRotate90,
    FlipVerticalRotate270,
    Invalid,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PropertyChangeForcesCommitCriteria {
    #[default]
    kNone,
    kTransform,
    kAny,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionBoundType {
    LEFT,
    RIGHT,
    CENTER,
    HIDDEN,
    #[default]
    EMPTY,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection<T> {
    pub start: T,
    pub end: T,
}
pub trait LayerTreeSelectionBound {
    fn r#type(&self) -> SelectionBoundType;
    fn visible(&self) -> bool;
    fn edge_start(&self) -> PointF;
    fn edge_end(&self) -> PointF;
    fn SetEdgeStart(&mut self, point: PointF);
    fn SetEdgeEnd(&mut self, point: PointF);
    fn SetVisibleEdgeStart(&mut self, point: PointF);
    fn SetVisibleEdgeEnd(&mut self, point: PointF);
    fn set_type(&mut self, value: SelectionBoundType);
    fn SetEdge(&mut self, start: PointF, end: PointF);
    fn SetVisibleEdge(&mut self, start: PointF, end: PointF);
    fn set_visible(&mut self, value: bool);
}
pub use crate::cc::layers::picture_layer_impl::ImageInvalidationResult;
pub use crate::cc::trees::swap_promise::{DidNotSwapAction, DidNotSwapReason, SwapPromise};

pub type PropertyTreesFor<B> = PropertyTrees<
    <B as LayerTreeTypes>::Transform,
    <B as LayerTreeTypes>::Filters,
    <B as LayerTreeTypes>::Path,
    <B as LayerTreeTypes>::RenderSurface,
    <B as LayerTreeTypes>::CopyOutputRequest,
>;
pub trait LayerTreePictureContext<B: LayerTreeTypes>:
    super::picture_layer_impl::PictureLayerReadContext
{
    fn WithPicture<Output>(
        &mut self,
        callback: impl FnOnce(&mut super::picture_layer_impl::PictureLayerImplContext<'_, B>) -> Output,
    ) -> Output
    where
        B: super::picture_layer_impl::PictureLayerImplTypes;
}
/// Field-only, synchronous projections of the current split resident tree and
/// services. Implementations lend the real context inside the callback; they
/// never save owners, copy state, or perform LayerImpl policy here.
pub trait LayerTreeLayerContext<B: LayerTreeTypes>: LayerTreePictureContext<B> {
    fn RegisterScrollbar(&mut self, layer: &mut B::ScrollbarLayer);
    fn UnregisterScrollbar(&mut self, layer: &mut B::ScrollbarLayer);

    /// The same actual property trees used by this callback, never a cached view.
    fn WithScrollbarOpacity<Output>(
        &mut self,
        callback: impl FnOnce(
            &mut dyn super::scrollbar_layer_impl_base::ScrollbarOpacityContext<B>,
        ) -> Output,
    ) -> Output;
    fn PropertyTrees(&self) -> &PropertyTreesFor<B>;
    fn WithBase<Output>(
        &mut self,
        callback: impl FnOnce(&mut super::layer_impl::LayerImplContext<'_, B>) -> Output,
    ) -> Output
    where
        B: super::layer_impl::LayerImplTypes;
    fn WithScrollbar<Output>(
        &mut self,
        callback: impl FnOnce(
            &mut super::scrollbar_layer_impl_base::ScrollbarLayerImplContext<'_, B>,
        ) -> Output,
    ) -> Output
    where
        B: super::layer_impl::LayerImplTypes;
}
/// Readonly source-tree borrows used by native pending-to-active property moves.
/// Source layer storage remains independently mutable; no tree state is copied.
pub trait LayerTreeSourceContext<B: LayerTreeTypes> {
    fn WithBaseSource<Output>(
        &self,
        callback: impl FnOnce(&super::layer_impl::LayerImplReadContext<'_, B>) -> Output,
    ) -> Output
    where
        B: super::layer_impl::LayerImplTypes;
    fn WithPictureSource<Output>(
        &self,
        callback: impl FnOnce(&super::picture_layer_impl::PictureLayerSourceContext<'_, B>) -> Output,
    ) -> Output
    where
        B: super::picture_layer_impl::PictureLayerImplTypes;
}
pub trait LayerTreeTypes: Sized + 'static {
    type LayerSourceContext<'a>: LayerTreeSourceContext<Self>
    where
        Self: 'a;
    /// One field split of both resident trees. The source lends only readonly
    /// sibling state; target services and tracking are exclusively mutable.
    fn WithLayerSyncContext<Output>(
        source: &mut LayerTreeImpl<Self>,
        target: &mut LayerTreeImpl<Self>,
        target_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, Self>,
        id: i32,
        callback: impl FnOnce(
            &mut Self::Layer,
            &Self::LayerSourceContext<'_>,
            &mut Self::Layer,
            &mut Self::LayerContext<'_>,
        ) -> Output,
    ) -> Output;
    type LayerTraceValue: super::layer_impl::LayerImplTraceValue;
    type LayerRemaining<'a>: LayerTreeRemainingStorage<Self>
    where
        Self: 'a;
    type LayerStorage<'a>: LayerTreeLayerStorage<Self>
    where
        Self: 'a;
    type PictureLayerContext<'a>: LayerTreePictureContext<Self>
    where
        Self: 'a;
    fn WithPictureLayersContext<Output>(
        tree: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<Self>,
        host: &mut dyn super::picture_layer_impl::PictureLayerContextHost<Self>,
        callback: impl FnOnce(&mut Self::LayerStorage<'_>, &mut Self::PictureLayerContext<'_>) -> Output,
    ) -> Output;

    type LayerContext<'a>: LayerTreeLayerContext<Self>
    where
        Self: 'a;
    /// Only splits the actual layer list from its sibling tree/host/services.
    /// Source policy remains in LayerTreeImpl and the canonical derived layers.
    fn WithLayerContext<Output>(
        tree: &mut LayerTreeImpl<Self>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, Self>,
        id: i32,
        callback: impl FnOnce(&mut Self::Layer, &mut Self::LayerContext<'_>) -> Output,
    ) -> Output;
    /// Run a native most-derived destructor after its owner has been removed
    /// from the tree's LayerList. The selected tree and all remaining layers
    /// stay resident for destructor callbacks.
    fn WithDetachedLayerContext<Output>(
        tree: &mut LayerTreeImpl<Self>,
        tree_context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, Self>,
        layer: &mut Self::Layer,
        callback: impl FnOnce(&mut Self::Layer, &mut Self::LayerContext<'_>) -> Output,
    ) -> Output;
    type DamageContext<'a>: super::damage_context::DamageContext<Types = Self>
    where
        Self: 'a;
    fn MakeDamageContext<'a>(
        tree: &'a crate::cc::trees::layer_tree_impl::LayerTreeImpl<Self>,
        host: &'a dyn LayerTreeHost<Self>,
    ) -> Self::DamageContext<'a>;
    type DrawContext<'a>: super::draw_property::DrawContext<
        Types = Self,
        Transform = Self::Transform,
        Filters = Self::Filters,
        Path = Self::Path,
        Surface = Self::RenderSurface,
        CopyRequest = Self::CopyOutputRequest,
        Layer = i32,
    >
    where
        Self: 'a;
    type OcclusionContext<'a>: super::occlusion::OcclusionContext<
            Types = Self,
            Transform = Self::Transform,
            Layer = i32,
            Surface = i32,
        > + super::occlusion::OcclusionLayerTree<i32>
    where
        Self: 'a;
    fn MakeDrawContext<'a>(
        state: LayerTreeDrawState<'a, Self>,
        host: &'a mut dyn draw_host::LayerTreeDrawHost<Self>,
    ) -> Self::DrawContext<'a>;
    fn MakeOcclusionContext<'a>(
        layers: &'a Self::LayerList,
        trees: &'a PropertyTreesFor<Self>,
        surfaces: &'a [i32],
    ) -> Self::OcclusionContext<'a>;

    type CommitState: LayerTreeCommitState<Self>;
    type UnsafeCommitState: LayerTreeUnsafeCommitState<Self>;
    type Synchronizer: LayerTreeSynchronizer<Self>;
    type Transform: LayerTreeTransform + super::scrollbar::ScrollbarTransform;
    type Filters: super::draw_property::DrawFilters;
    type Path: PropertyPath;
    type RenderSurface: LayerTreeSurfaceTransform<Transform = Self::Transform>
        + LayerTreeRenderSurface<Self>
        + super::draw_property::DrawSurface<Self::Transform, Types = Self>;
    type CopyOutputRequest: PropertyTreeCopyOutputRequest;
    type RegionBackend: SkRegionBackend;
    type Layer: LayerTreeLayer<Self>;
    type PictureLayer: LayerTreePictureLayer<Self>;
    type ScrollbarLayer: LayerTreeScrollbar<Self>;
    type BeginMainFrameReason;

    type LayerList: LayerTreeLayerList<Self> + Default;
    type DisplayColorSpaces: Clone + Default + PartialEq;
    type LocalSurfaceId: Clone + Default + PartialEq;
    type SurfaceRange: Clone + Ord;
    type SwapPromise: SwapPromise<Self::CompositorFrameMetadata>;
    type CompositorFrameMetadata;
    type UIResourceRequest: LayerTreeUIResourceRequest<Self>;
    type UIResourceBitmap;
    type PresentationCallback;
    type SuccessfulPresentationCallback;
    type EventMetrics;
    type DelegatedInkMetadata;
    type ViewTransitionRequest: LayerTreeViewTransitionRequest;
    type SelectionBound: LayerTreeSelectionBound + Clone + Default;
    type DecodingMode;
    type Settings: LayerTreeSettings;
    type DebugState;
    type LayerTreeFrameSink;
    type RasterContextProvider;
    type ClientResourceProvider;
    type TileManager;
    type ImageDecodeCache;
    type ImageAnimationController;
    type FrameSorter;
    type MemoryHistory;
    type DebugRectHistory;
    type GlobalTileState;
    type VideoFrameControllerClient;
    type MutatorHost: LayerTreeMutatorHost;
    type TaskRunnerProvider;
    type PaintWorkletTracker;
    type PaintImageId: Ord;
    type PrioritizedTile;
    type AnimatedImageDriverMap;
    type DisplayItemList;
    type ContentColorUsage;
    type TargetColorParams;
}
pub trait LayerTreeSettings {
    fn enable_hit_test_opaqueness(&self) -> bool;
    fn is_layer_tree_for_ui(&self) -> bool;
    fn scrollbar_fade_delay(&self) -> super::time::TimeDelta;
    fn scrollbar_fade_duration(&self) -> super::time::TimeDelta;
    fn scrollbar_animator(&self) -> ScrollbarAnimator;
    fn scrollbar_thinning_duration(&self) -> super::time::TimeDelta;
    fn idle_thickness_scale(&self) -> f32;
    fn enable_fluent_overlay_scrollbar(&self) -> bool;
    fn minimum_occlusion_tracking_size(&self) -> Size;
    fn release_tile_resources_for_hidden_layers(&self) -> bool;
    fn commit_fractional_scroll_deltas(&self) -> bool;
    fn is_for_scalable_page(&self) -> bool;
    fn trees_in_viz_in_viz_process(&self) -> bool;
    fn scrollbar_flash_once_after_scroll_update(&self) -> bool;
    fn scrollbar_flash_after_any_scroll_update(&self) -> bool;
}
// Field-only facet of the canonical source value; feature decisions stay with
// LayerTreeSettings source methods called through the current live host.
impl LayerTreeSettings for crate::cc::trees::layer_tree_settings::LayerTreeSettings {
    fn enable_hit_test_opaqueness(&self) -> bool {
        self.enable_hit_test_opaqueness
    }
    fn is_layer_tree_for_ui(&self) -> bool {
        self.is_layer_tree_for_ui
    }
    fn scrollbar_fade_delay(&self) -> super::time::TimeDelta {
        self.scrollbar_fade_delay
    }
    fn scrollbar_fade_duration(&self) -> super::time::TimeDelta {
        self.scrollbar_fade_duration
    }
    fn scrollbar_animator(&self) -> ScrollbarAnimator {
        self.scrollbar_animator
    }
    fn scrollbar_thinning_duration(&self) -> super::time::TimeDelta {
        self.scrollbar_thinning_duration
    }
    fn enable_fluent_overlay_scrollbar(&self) -> bool {
        self.enable_fluent_overlay_scrollbar
    }
    fn idle_thickness_scale(&self) -> f32 {
        self.idle_thickness_scale
    }
    fn minimum_occlusion_tracking_size(&self) -> Size {
        self.minimum_occlusion_tracking_size
    }
    fn release_tile_resources_for_hidden_layers(&self) -> bool {
        self.release_tile_resources_for_hidden_layers
    }
    fn commit_fractional_scroll_deltas(&self) -> bool {
        self.commit_fractional_scroll_deltas
    }
    fn is_for_scalable_page(&self) -> bool {
        self.is_for_scalable_page
    }
    fn trees_in_viz_in_viz_process(&self) -> bool {
        self.trees_in_viz_in_viz_process
    }
    fn scrollbar_flash_once_after_scroll_update(&self) -> bool {
        self.scrollbar_flash_once_after_scroll_update
    }
    fn scrollbar_flash_after_any_scroll_update(&self) -> bool {
        self.scrollbar_flash_after_any_scroll_update
    }
}

pub trait LayerTreeHUD {
    fn IsAnimatingHUDContents(&self) -> bool;
}
pub trait LayerTreeLayer<B: LayerTreeTypes> {
    // Source: cc/layers/layer_impl.h; access the current native layer output.
    fn GetAllowedTouchAction(&self, rect: Rect) -> crate::cc::input::touch_action::TouchAction;

    fn bounds(&self, trees: &PropertyTreesFor<B>) -> Size;
    fn ScreenSpaceTransform(&self, trees: &PropertyTreesFor<B>) -> B::Transform;
    fn clip_tree_index(&self) -> i32;
    fn effect_tree_index(&self) -> i32;
    fn render_target_effect_tree_index(&self, trees: &PropertyTreesFor<B>) -> i32;
    fn transform_tree_index(&self) -> i32;
    fn HitTestable(&self, trees: &PropertyTreesFor<B>) -> bool;
    fn Is3dSorted(&self, trees: &PropertyTreesFor<B>) -> bool;
    fn GetSortingContextId(&self, trees: &PropertyTreesFor<B>) -> i32;
    fn OpaqueToHitTest(&self, trees: &PropertyTreesFor<B>) -> bool;
    fn IsScrollerOrScrollbar(&self, trees: &PropertyTreesFor<B>) -> bool;
    fn has_touch_action_regions(&self) -> bool;
    fn GetAllTouchActionRegions(&self) -> &crate::cc::base::region::Region<B::RegionBackend>;
    fn wheel_event_handler_region(
        &self,
    ) -> impl std::ops::Deref<Target = crate::cc::base::region::Region<B::RegionBackend>> + '_;
    fn main_thread_scroll_hit_test_region(
        &self,
    ) -> impl std::ops::Deref<Target = crate::cc::base::region::Region<B::RegionBackend>> + '_;
    fn non_composited_scroll_hit_test_rects(&self) -> Option<&[ScrollHitTestRect]>;
    fn visible_layer_rect(&self) -> Rect;
    fn ResetChangeTracking(&mut self);
    fn AsHeadsUpDisplayLayer(&self) -> Option<&dyn LayerTreeHUD> {
        None
    }
    fn id(&self) -> i32;
    fn element_id(&self) -> ElementId;
    fn SetOcclusionInContentSpace(
        &mut self,
        occlusion: crate::cc::trees::occlusion::Occlusion<B::Transform>,
    );
    fn DrawTransform(&self, trees: &PropertyTreesFor<B>) -> B::Transform;
    fn SetElementId(&mut self, id: ElementId, context: &mut B::LayerContext<'_>);
    fn ReleaseResources(&mut self, context: &mut B::LayerContext<'_>);
    fn OnPurgeMemory(&mut self, context: &mut B::LayerContext<'_>);
    fn ReleaseTileResources(&mut self, context: &mut B::LayerContext<'_>);
    fn RecreateTileResources(&mut self, context: &mut B::LayerContext<'_>);
    fn SetInInvisibleLayerTree(&mut self);
    fn DidBecomeActive(&mut self, context: &mut B::LayerContext<'_>);
    fn contributes_to_drawn_render_surface(&self) -> bool;
    fn GetAllPrioritizedTilesForTracing(
        &mut self,
        out: &mut Vec<B::PrioritizedTile>,
        context: &mut B::LayerContext<'_>,
    );
    fn AsValueInto(&self, value: &mut B::LayerTraceValue, context: &mut B::LayerContext<'_>);
    fn LayerPropertyChanged(&self, trees: &PropertyTreesFor<B>) -> bool;
    fn LayerPropertyChangedFromPropertyTrees(&self, trees: &PropertyTreesFor<B>) -> bool;
    fn NoteLayerPropertyChangedFromPropertyTrees(&mut self, context: &mut B::LayerContext<'_>);
    fn set_is_inner_viewport_scroll_layer(&mut self);
    fn AsScrollbarLayer(&self) -> Option<&B::ScrollbarLayer>;
    fn AsScrollbarLayerMut(&mut self) -> Option<&mut B::ScrollbarLayer>;
    fn AsPictureLayer(&self) -> Option<&B::PictureLayer>;
    fn AsPictureLayerMut(&mut self) -> Option<&mut B::PictureLayer>;
    /// Explicit most-derived destruction required before the Rust owner drops.
    fn Destroy(&mut self, context: &mut B::LayerContext<'_>);
}
pub trait LayerTreePictureLayer<B: LayerTreeTypes> {
    fn IdentityForDestruction(&self) -> crate::cc::layers::picture_layer_impl::PictureLayerImplId;
    fn TwinForDestruction(
        &self,
    ) -> Option<crate::cc::layers::picture_layer_impl::PictureLayerImplId>;
    fn ClearTwinForDestruction(
        &mut self,
    ) -> crate::cc::layers::picture_layer_impl::PictureLayerImplId;
    fn HasValidTilePriorities(
        &self,
        context: &dyn super::picture_layer_impl::PictureLayerReadContext,
    ) -> bool;
    fn UpdateTiles(&mut self, context: &mut B::PictureLayerContext<'_>) -> bool;
    fn ReleaseResources(&mut self, context: &mut B::PictureLayerContext<'_>);
    fn AnnotateAnimatedImages(
        &self,
        map: &mut B::AnimatedImageDriverMap,
        context: &dyn super::picture_layer_impl::PictureLayerReadContext,
    );
    fn ResetRasterScale(&mut self);
    fn CommitPendingRasterSource(
        &mut self,
        updates: &mut Vec<i32>,
        context: &mut B::PictureLayerContext<'_>,
    );
    fn RegenerateDiscardableImageMap(&mut self, context: &mut B::PictureLayerContext<'_>);
    fn InvalidateRegionForImages(
        &mut self,
        images: &std::collections::BTreeSet<B::PaintImageId>,
        context: &mut B::PictureLayerContext<'_>,
    ) -> ImageInvalidationResult;
    fn InvalidateRasterInducingScrolls(
        &mut self,
        scrolls: &std::collections::BTreeSet<ElementId>,
        context: &mut B::PictureLayerContext<'_>,
    );
}
/// Borrowed resident layer storage, excluding the three independently mutated
/// push/animated-image/worklet tracking sets. No owner, index or layer is copied.
pub trait LayerTreeLayerStorage<B: LayerTreeTypes> {
    type Iter<'a>: DoubleEndedIterator<Item = &'a B::Layer>
    where
        Self: 'a,
        B: 'a;
    type IterMut<'a>: DoubleEndedIterator<Item = &'a mut B::Layer>
    where
        Self: 'a,
        B: 'a;
    fn iter(&self) -> Self::Iter<'_>;
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
    fn LayerByIdMut(&mut self, id: i32) -> Option<&mut B::Layer>;
    /// Native picture-id map order, each mutable reference scoped to one call.
    fn ForEachPictureLayer(
        &mut self,
        callback: &mut dyn FnMut(&mut B::PictureLayer) -> std::ops::ControlFlow<()>,
    ) -> std::ops::ControlFlow<()>;
}
pub trait LayerTreeLayerList<B: LayerTreeTypes> {
    type Iter<'a>: DoubleEndedIterator<Item = &'a B::Layer>
    where
        Self: 'a,
        B: 'a;
    type IterMut<'a>: DoubleEndedIterator<Item = &'a mut B::Layer>
    where
        Self: 'a,
        B: 'a;
    type PictureIter<'a>: Iterator<Item = &'a B::PictureLayer>
    where
        Self: 'a,
        B: 'a;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn iter(&self) -> Self::Iter<'_>;
    fn iter_mut(&mut self) -> Self::IterMut<'_>;
    fn PictureLayers(&self) -> Self::PictureIter<'_>;
    fn ForEachPictureLayer(
        &mut self,
        callback: &mut dyn FnMut(&mut B::PictureLayer) -> std::ops::ControlFlow<()>,
    ) -> std::ops::ControlFlow<()>;
    fn WithScrollbarGeometry(
        &mut self,
        id: i32,
        callback: &mut dyn FnMut(
            &mut dyn LayerTreeScrollbarGeometry<B>,
            &mut std::collections::BTreeSet<i32>,
        ),
    );
    fn LayerAt(&self, index: usize) -> &B::Layer;
    fn LayerById(&self, id: i32) -> Option<&B::Layer>;
    fn LayerByIdMut(&mut self, id: i32) -> Option<&mut B::Layer>;
    fn LayerByElementId(&self, id: ElementId) -> Option<&B::Layer>;
    fn AddLayer(&mut self, layer: Box<B::Layer>);
    fn Reserve(&mut self, count: usize);
    fn ReleaseLayerForTesting(&mut self, id: i32) -> Box<B::Layer>;
    fn SetPictureLayerWithAnimatedImages(&mut self, id: i32);
    fn RemovePictureLayerWithAnimatedImages(&mut self, id: i32);
    fn HasPictureLayerWithAnimatedImages(&self, id: i32) -> bool;
    fn SetPictureLayerWithWorklet(&mut self, id: i32);
    fn RemovePictureLayerWithWorklet(&mut self, id: i32);
    fn HasPictureLayerWithWorklet(&self, id: i32) -> bool;
    fn PictureLayersWithAnimatedImages(&self) -> Self::PictureIter<'_>;
    fn PictureLayersWithWorklets(&self) -> Self::PictureIter<'_>;
    fn LayersThatShouldPushProperties(&self) -> Self::Iter<'_>;
    fn AddLayerShouldPushProperties(&mut self, id: i32);
    fn ClearLayersThatShouldPushProperties(&mut self);
    fn RemainingLayerIds(&self) -> Vec<i32>;
}
pub trait LayerTreeHost<B: LayerTreeTypes>:
    super::picture_layer_impl::PictureLayerContextHost<B>
{
    fn DisconnectPictureTwin(
        &mut self,
        id: crate::cc::layers::picture_layer_impl::PictureLayerImplId,
    ) -> crate::cc::layers::picture_layer_impl::PictureLayerImplId;
    fn WithLayerContextParts(
        &mut self,
        selected: LayerTreeId,
        callback: &mut dyn FnMut(
            &dyn super::picture_layer_impl::PictureLayerReadHost<B>,
            &mut dyn super::picture_layer_impl::PictureLayerTreeHost<B>,
            &mut dyn super::picture_layer_impl::PictureLayerImplServices<B>,
            &mut dyn super::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
            &mut dyn LayerTreeScrollbarRegistrationHost<B>,
        ),
    ) where
        B: super::picture_layer_impl::PictureLayerImplTypes;
    fn NewLayerTraceValue(&self) -> B::LayerTraceValue;
    /// External feature and diagnostic calls only, shared with the narrow draw host.
    fn DrawServices(&self) -> &dyn draw_host::LayerTreeDrawServices;

    /// Synchronously lends the real source tree back to the host while another
    /// selected tree executes its native child-to-owner callback sequence.
    fn WithBorrowedTree(
        &mut self,
        source: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        callback: &mut dyn FnMut(&mut dyn LayerTreeHost<B>),
    );

    fn MonotonicTimeNow(&self) -> super::time::TimeTicks;
    fn identity(&self) -> LayerTreeHostId;
    fn SelectionEdgeVisibilityUsesFullEdge(&self) -> bool;
    fn ViewTransitionCaptureAndDisplay(&self) -> bool;
    fn GetTreePriority(&self) -> super::scheduler::TreePriority;
    fn GetVideoFrameControllerClient(&self) -> &B::VideoFrameControllerClient;
    fn UpdateImageDecodingHints(
        &mut self,
        map: std::collections::BTreeMap<B::PaintImageId, B::DecodingMode>,
    );
    fn GetMSAASampleCountForRaster(
        &self,
        receiver: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        list: &B::DisplayItemList,
    ) -> i32;
    fn GetTargetColorParams(
        &self,
        receiver: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        usage: B::ContentColorUsage,
    ) -> B::TargetColorParams;
    fn ResetBrowserControlsAnimations(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    );
    fn SetViewTransitionContentRect(
        &mut self,
        sequence: u32,
        id: &ViewTransitionElementResourceId,
        rect: &RectF,
    );
    fn IsReadyToActivate(&self) -> bool;
    fn RequestImplSideInvalidationForRerasterTiling(&mut self);
    fn global_tile_state(&self) -> &B::GlobalTileState;
    fn paint_worklet_tracker(&mut self) -> &mut B::PaintWorkletTracker;
    fn DrawTransform(&self) -> &B::Transform;
    fn IsInSynchronousComposite(&self) -> bool;
    fn GetActivelyScrollingType(
        &self,
        receiver: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    ) -> crate::cc::input::actively_scrolling_type::ActivelyScrollingType;
    fn viewport_mobile_optimized(&self) -> bool;

    fn IsCCSlimmingEnabled(&self) -> bool;
    fn DidRegisterScrollbarLayer(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: ElementId,
        orientation: ScrollbarOrientation,
    );
    fn DidUnregisterScrollbarLayer(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: ElementId,
        orientation: ScrollbarOrientation,
    );
    fn RegisterScrollbarAnimationController(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: ElementId,
        opacity: f32,
    );

    /// Source factory uses the actual borrowed tree and an ephemeral live client.
    fn CreateScrollbarAnimationController(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: ElementId,
        opacity: f32,
    ) -> Option<Box<crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>>;
    fn ResetRequiresHighResToDraw(&mut self);
    fn TraceDidActivateLayerTree(&self, source_frame_number: i32);
    fn RequiresHighResToDraw(&self) -> bool;
    fn task_runner_provider(&self) -> &B::TaskRunnerProvider;
    fn WithLayerTreeFrameSink(&self, callback: &mut dyn FnMut(Option<&B::LayerTreeFrameSink>));
    fn max_texture_size(&self) -> i32;
    fn debug_state(&self) -> &B::DebugState;
    fn context_provider(&self) -> Option<&B::RasterContextProvider>;
    fn resource_provider(&self) -> Option<&B::ClientResourceProvider>;
    fn tile_manager(&self) -> &B::TileManager;
    fn GetImageDecodeCache(&self) -> Option<&B::ImageDecodeCache>;
    fn image_animation_controller(&self) -> Option<&B::ImageAnimationController>;
    fn frame_sorter(&self) -> &B::FrameSorter;
    fn memory_history(&self) -> &B::MemoryHistory;
    fn debug_rect_history(&self) -> &B::DebugRectHistory;
    fn ActiveTreeLayerById<'a>(
        &'a self,
        receiver: &'a crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: i32,
    ) -> Option<&'a B::Layer>;
    fn PendingTreeLayerById<'a>(
        &'a self,
        receiver: &'a crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: i32,
    ) -> Option<&'a B::Layer>;
    fn IsPinchGestureActive(&self) -> bool;
    fn CurrentBeginFrameArgs(&self) -> &super::begin_frame::BeginFrameArgs;
    fn CurrentBeginFrameInterval(&self) -> super::time::TimeDelta;
    fn viewport_rect_for_tile_priority(&self) -> Rect;
    fn raster_caps(&self) -> &crate::cc::trees::raster_capabilities::RasterCapabilities;
    fn SetNeedsRedraw(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        animation_only: bool,
        skip_if_inside_draw: bool,
    );
    fn ResourceIdForUIResource(&self, id: UIResourceId) -> ResourceId;
    fn GetUIResourceSize(&self, id: UIResourceId) -> Size;
    fn IsUIResourceOpaque(&self, id: UIResourceId) -> bool;
    fn CreateUIResource(
        &mut self,
        receiver: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: UIResourceId,
        bitmap: &B::UIResourceBitmap,
    );
    fn DeleteUIResource(
        &mut self,
        receiver: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: UIResourceId,
    );
    fn EvictedUIResourcesExist(&self) -> bool;

    fn mutator_host(&self) -> &B::MutatorHost;
    fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32);
    fn OnBrowserControlsParamsChanged(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        animate_height_changes: bool,
    );
    fn TopControlsShownRatioRange(&self) -> (f32, f32);
    fn BottomControlsShownRatioRange(&self) -> (f32, f32);
    fn DidChangeBrowserControlsPosition(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    );
    fn PendingTreeDidUpdatePageScale(&mut self);
    fn RecycleTreeDidUpdatePageScale(&mut self);
    fn OnPageScaleUpdated(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    );
    fn SetViewportDamage(&mut self, rect: Rect);
    fn external_viewport(&self) -> Rect;
    fn IsActiveTree(&self, id: LayerTreeId) -> bool;
    fn IsPendingTree(&self, id: LayerTreeId) -> bool;
    fn IsRecycleTree(&self, id: LayerTreeId) -> bool;
    fn IsSyncTree(&self, id: LayerTreeId) -> bool;
    fn HasPendingTree(&self) -> bool;
    fn HasRecycleTree(&self) -> bool;
    fn settings(&self) -> &B::Settings;
    /// Synchronous canonical settings + live feature-service queries.
    fn TreesInVizInClientProcess(&self) -> bool;
    fn TreeAnimationsInVizInVizProcess(&self) -> bool;
    fn HasLayerTreeFrameSink(&self) -> bool;
    fn IsResourcelessSoftwareDraw(&self) -> bool;
    fn HasImageAnimationController(&self) -> bool;
    fn UpdateImageAnimationStateFromDrivers(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    );
    fn DidModifyTilePriorities(&mut self, pending_update_tiles: bool);
    fn TraceLayersUpdatedCount(&self, count: usize);

    fn RequestImplSideInvalidationForRasterInducingScroll(&mut self, id: ElementId);
    fn PendingTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool);
    fn RecycleTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool);
    fn OnCanDrawStateChangedForTree(
        &mut self,
        receiver: &crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    );
    fn SetNeedsCommit(&mut self);
    fn MissingPropertyNodeForElementId(
        &self,
        mutation: crate::cc::trees::layer_tree_impl::PropertyMutation,
    );
    fn TraceDelegatedInkMetadataPush(&self, metadata: &B::DelegatedInkMetadata);
    fn OuterViewportScrollNode<'a>(
        &'a self,
        receiver: &'a crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
    ) -> Option<&'a crate::cc::trees::scroll_node::ScrollNode>;
    /// Source native pointers are borrowed only inside this synchronous callback.
    fn WithScrollbarAnimationController(
        &mut self,
        receiver: &mut crate::cc::trees::layer_tree_impl::LayerTreeImpl<B>,
        id: ElementId,
        callback: &mut dyn FnMut(Option<&mut crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>, &mut dyn crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationControllerClient<Transform=B::Transform>),
    );
    fn DidAnimateScrollOffset(&mut self);
    fn TraceImageInvalidationCounts(
        &self,
        no_images: usize,
        no_invalidation: usize,
        invalidated: usize,
    );
    fn ClearHistory(&mut self);
    fn ClearCaches(&mut self);
    fn PushMutatorPropertiesFrom(
        &mut self,
        unsafe_state: &B::UnsafeCommitState,
        trees: &PropertyTreesFor<B>,
    );
    fn CreateRenderSurface(&mut self, tree: LayerTreeId, id: ElementId) -> Box<B::RenderSurface>;
}

pub trait LayerTreeScrollbarGeometry<B: LayerTreeTypes> {
    fn is_overlay_scrollbar(&self) -> bool;
    fn thumb_thickness_scale_factor(&self) -> f32;
    fn SetThumbThicknessScaleFactor(
        &mut self,
        value: f32,
        c: &mut dyn super::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    );
    fn orientation(&self) -> ScrollbarOrientation;
    fn SetCurrentPos(
        &mut self,
        value: f32,
        c: &mut dyn super::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    );
    fn SetClipLayerLength(
        &mut self,
        value: f32,
        c: &mut dyn super::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    );
    fn SetScrollLayerLength(
        &mut self,
        value: f32,
        c: &mut dyn super::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    );
    fn SetVerticalAdjust(
        &mut self,
        value: f32,
        c: &mut dyn super::scrollbar_layer_impl_base::ScrollbarGeometryContext<B>,
    );
}
pub trait LayerTreeScrollbar<B: LayerTreeTypes>: LayerTreeScrollbarGeometry<B> {
    fn id(&self) -> i32;
    fn scroll_element_id(&self) -> ElementId;

    fn GetScrollbarAnimator(
        &self,
        style: &dyn super::scrollbar_layer_impl_base::ScrollbarGeometryStyle,
    ) -> ScrollbarAnimator;
    fn IsFluentOverlayScrollbarEnabled(
        &self,
        style: &dyn super::scrollbar_layer_impl_base::ScrollbarGeometryStyle,
    ) -> bool;
    fn SetOverlayScrollbarLayerOpacityAnimated(
        &mut self,
        opacity: f32,
        fade_out_animation: bool,
        c: &mut dyn super::scrollbar_layer_impl_base::ScrollbarOpacityContext<B>,
    );
    fn Opacity(&self, trees: &PropertyTreesFor<B>) -> f32;
    fn has_find_in_page_tickmarks(&self) -> bool;
}
/// Native storage/tracking field split during synchronous scroll callbacks.
/// The callback can mutate an actual scrollbar and the disjoint source push set.
pub trait LayerTreeScrollStore<B: LayerTreeTypes> {
    fn WithScrollbarGeometry(
        &mut self,
        id: i32,
        callback: &mut dyn FnMut(
            &mut dyn LayerTreeScrollbarGeometry<B>,
            &mut std::collections::BTreeSet<i32>,
        ),
    );
}

/// Field-level access to the live main-thread CommitState. Moves are explicit;
/// no aggregate commit/tree snapshot is fabricated at this boundary.
pub trait LayerTreeCommitState<B: LayerTreeTypes> {
    fn property_trees(&self) -> &PropertyTreesFor<B>;
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B>;
    fn next_commit_forces_redraw(&self) -> bool;
    fn next_commit_forces_recalculate_raster_scales(&self) -> bool;
    fn needs_full_tree_sync(&self) -> bool;
    fn clear_caches_on_next_commit(&self) -> bool;
    fn needs_surface_ranges_sync(&self) -> bool;
    fn TakePresentationCallbacks(&mut self) -> Vec<B::PresentationCallback>;
    fn TakeSuccessfulPresentationCallbacks(&mut self) -> Vec<B::SuccessfulPresentationCallback>;
    fn SurfaceRanges(&self) -> std::collections::BTreeSet<B::SurfaceRange>;
    fn scrollers_clobbering_active_value(&self) -> &std::collections::BTreeSet<ElementId>;
    fn picture_layer_ids_with_new_raster_source(&self) -> &std::collections::BTreeSet<i32>;
    fn TakeSwapPromises(&mut self) -> Vec<Box<B::SwapPromise>>;
    fn TakeEventMetrics(&mut self) -> Vec<B::EventMetrics>;
    fn ui_resource_request_queue(&mut self) -> Vec<B::UIResourceRequest>;
    fn hud_layer_id(&self) -> i32;
    fn background_color(&self) -> Color4f;
    fn have_scroll_event_handlers(&self) -> bool;
    fn GetEventListenerProperties(&self, class: EventListenerClass) -> EventListenerProperties;
    fn property_change_forces_commit_criteria(&self) -> PropertyChangeForcesCommitCriteria;
    fn viewport_property_ids(&self) -> ViewportPropertyIds;
    fn selection(&self) -> &crate::cc::input::layer_selection_bound::LayerSelection;
    fn page_scale_factor(&self) -> f32;
    fn min_page_scale_factor(&self) -> f32;
    fn max_page_scale_factor(&self) -> f32;
    fn page_scale_factor_limits_set(&self) -> bool;
    fn browser_controls_params(
        &self,
    ) -> &crate::cc::trees::browser_controls_params::BrowserControlsParams;
    fn overscroll_behavior(&self) -> crate::cc::input::overscroll_behavior::OverscrollBehavior;
    fn top_controls_shown_ratio(&self) -> f32;
    fn bottom_controls_shown_ratio(&self) -> f32;
    fn display_color_spaces(&self) -> &B::DisplayColorSpaces;
    fn external_page_scale_factor(&self) -> f32;
    fn painted_device_scale_factor(&self) -> f32;
    fn device_scale_factor(&self) -> f32;
    fn device_viewport_rect(&self) -> Rect;
    fn max_safe_area_inset_bottom(&self) -> f32;
    fn new_local_surface_id_request(&self) -> bool;
    fn screenshot_destination_token(&self) -> UnguessableToken;
    fn primary_main_frame_item_sequence_number(&self) -> i64;
    fn local_surface_id_from_parent(&self) -> &B::LocalSurfaceId;
    fn TakePendingPageScaleAnimation(
        &mut self,
    ) -> Option<Box<crate::cc::input::page_scale_animation::PendingPageScaleAnimation>>;
    fn force_send_metadata_request(&self) -> bool;
    fn display_transform_hint(&self) -> OverlayTransform;
    fn TakeDelegatedInkMetadata(&mut self) -> Option<Box<B::DelegatedInkMetadata>>;
    fn ViewTransitionRequests(&mut self) -> &mut Vec<Option<Box<B::ViewTransitionRequest>>>;
}
pub trait LayerTreeUnsafeCommitState<B: LayerTreeTypes> {
    fn HasRootLayer(&self) -> bool;
}
pub trait LayerTreeSynchronizer<B: LayerTreeTypes> {
    /// Page/assembly-root borrowed main-side context, never provided by the impl Host.
    type MainContext<'a, 'c>
    where
        B: 'a + 'c,
        'c: 'a;
    fn SynchronizeTrees(
        commit: &mut B::CommitState,
        unsafe_state: &B::UnsafeCommitState,
        tree: &mut LayerTreeImpl<B>,
        context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        main_context: &mut Self::MainContext<'_, '_>,
    );
    fn PushLayerProperties(
        commit: &mut B::CommitState,
        unsafe_state: &B::UnsafeCommitState,
        tree: &mut LayerTreeImpl<B>,
        context: &mut crate::cc::trees::layer_tree_impl::LayerTreeContext<'_, B>,
        main_context: &mut Self::MainContext<'_, '_>,
        discardable_image_updates: &mut Vec<i32>,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementListType {
    ACTIVE,
    PENDING,
}
pub trait LayerTreeRenderSurface<B: LayerTreeTypes>: PropertyTreeRenderSurface {
    fn SetOcclusionInContentSpace(
        &mut self,
        occlusion: crate::cc::trees::occlusion::Occlusion<B::Transform>,
    );
    fn AncestorPropertyChanged(&self, trees: &PropertyTreesFor<B>) -> bool;
    fn NoteAncestorPropertyChanged(&mut self);
}

pub trait LayerTreeMutatorHost {
    fn HasAnyAnimationTargetingProperty(
        &self,
        id: ElementId,
        property: crate::cc::trees::target_property::Type,
    ) -> bool;
    fn HasPotentiallyRunningAnimationForProperty(
        &self,
        id: ElementId,
        list: ElementListType,
        property: crate::cc::trees::target_property::Type,
    ) -> bool;
    fn MaximumScale(&self, id: ElementId, list: ElementListType) -> f32;
}

/// Only borrowed owner fields required by draw_property_utils. This view never
/// owns a layer, node, tree, raster result, or copied configuration aggregate.
pub struct LayerTreeDrawState<'a, B: LayerTreeTypes> {
    pub layers: &'a mut B::LayerList,
    pub viewport: &'a ViewportPropertyIds,
    pub page_scale:
        &'a std::rc::Rc<std::cell::RefCell<crate::cc::trees::layer_tree_impl::SyncedScale>>,
    pub device_scale: &'a f32,
    pub external_scale: &'a f32,
    pub device_viewport: &'a Rect,
    pub capture_tokens: &'a std::collections::BTreeSet<super::draw_property::ViewTransitionToken>,
    pub tree_id: LayerTreeId,
    /// The native request vector remains resident while its setter is invoked.
    pub view_transition_requests: &'a [Box<B::ViewTransitionRequest>],
}

pub trait LayerTreeSurfaceTransform {
    type Transform: super::occlusion::OcclusionTransform;
}

pub use crate::cc::trees::layer_tree_settings::ScrollbarAnimator;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct UIResourceId(pub i32);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UIResourceRequestType {
    kCreate,
    kDelete,
}
pub trait LayerTreeUIResourceRequest<B: LayerTreeTypes> {
    fn GetType(&self) -> UIResourceRequestType;
    fn GetId(&self) -> UIResourceId;
    fn GetBitmap(&self) -> &B::UIResourceBitmap;
}

pub trait LayerTreeTransform:
    super::draw_property::DrawTransform + super::occlusion::OcclusionTransform
{
    fn MapPoint3D(&self, point: super::geometry::Point3F) -> super::geometry::Point3F;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScrollbarLayerHandle {
    pub object_identity: usize,
    pub layer_id: i32,
}
pub type ScrollbarSet = std::collections::BTreeSet<ScrollbarLayerHandle>;
pub use crate::cc::layers::scroll_hit_test_rect::ScrollHitTestRect;

pub trait LayerTreeViewTransitionRequest {
    fn IsSave(&self) -> bool;
    fn token(&self) -> super::draw_property::ViewTransitionToken;
    fn sequence_id(&self) -> u32;
}

// gfx::RectF inclusive intersection is a geometry primitive, including zero
// area intersections at touching edges (ordinary Intersect excludes those).
pub fn InclusiveIntersectRectF(rect: &mut RectF, other: RectF) -> bool {
    let left = super::geometry::cpp_max(rect.x(), other.x());
    let top = super::geometry::cpp_max(rect.y(), other.y());
    let right = super::geometry::cpp_min(rect.right(), other.right());
    let bottom = super::geometry::cpp_min(rect.bottom(), other.bottom());
    if left > right || top > bottom {
        *rect = RectF::default();
        return false;
    }
    *rect = RectF::new(left, top, right - left, bottom - top);
    true
}

// components/viz/common/traced_value.cc:14-18,29-35. These are trace-output
// dictionary annotations, not rendering/data snapshots or a retained pipeline.
pub fn AppendIDRef(value: &mut impl super::property_types::PropertyTrace, id: usize) {
    value.BeginDictionaryUnnamed();
    value.SetString("id_ref", &format!("0x{id:x}"));
    value.EndDictionary();
}
pub fn MakeDictIntoImplicitSnapshot(
    value: &mut impl super::property_types::PropertyTrace,
    name: &str,
    id: usize,
) {
    value.SetString("id", &format!("{name}/0x{id:x}"));
}

// Rust borrowed field access to the canonical source CommitState. This facet
// never owns/copies the aggregate; take() corresponds to each native std::move.
impl<B: super::commit_state::CommitStateTypes> LayerTreeCommitState<B>
    for crate::cc::trees::commit_state::CommitState<B>
{
    fn property_trees(&self) -> &PropertyTreesFor<B> {
        &self.property_trees
    }
    fn property_trees_mut(&mut self) -> &mut PropertyTreesFor<B> {
        &mut self.property_trees
    }
    fn next_commit_forces_redraw(&self) -> bool {
        self.next_commit_forces_redraw
    }
    fn next_commit_forces_recalculate_raster_scales(&self) -> bool {
        self.next_commit_forces_recalculate_raster_scales
    }
    fn needs_full_tree_sync(&self) -> bool {
        self.needs_full_tree_sync
    }
    fn clear_caches_on_next_commit(&self) -> bool {
        self.clear_caches_on_next_commit
    }
    fn needs_surface_ranges_sync(&self) -> bool {
        self.needs_surface_ranges_sync
    }
    fn TakePresentationCallbacks(&mut self) -> Vec<B::PresentationCallback> {
        std::mem::take(&mut self.pending_presentation_callbacks)
    }
    fn TakeSuccessfulPresentationCallbacks(&mut self) -> Vec<B::SuccessfulPresentationCallback> {
        std::mem::take(&mut self.pending_successful_presentation_callbacks)
    }
    fn SurfaceRanges(&self) -> std::collections::BTreeSet<B::SurfaceRange> {
        Self::SurfaceRanges(self)
    }
    fn scrollers_clobbering_active_value(&self) -> &std::collections::BTreeSet<ElementId> {
        &self.scrollers_clobbering_active_value
    }
    fn picture_layer_ids_with_new_raster_source(&self) -> &std::collections::BTreeSet<i32> {
        &self.picture_layer_ids_with_new_raster_source
    }
    fn TakeSwapPromises(&mut self) -> Vec<Box<B::SwapPromise>> {
        std::mem::take(&mut self.swap_promises)
    }
    fn TakeEventMetrics(&mut self) -> Vec<B::EventMetrics> {
        std::mem::take(&mut self.event_metrics)
    }
    fn ui_resource_request_queue(&mut self) -> Vec<B::UIResourceRequest> {
        std::mem::take(&mut self.ui_resource_request_queue)
    }
    fn hud_layer_id(&self) -> i32 {
        self.hud_layer_id
    }
    fn background_color(&self) -> Color4f {
        self.background_color
    }
    fn have_scroll_event_handlers(&self) -> bool {
        self.have_scroll_event_handlers
    }
    fn GetEventListenerProperties(&self, class: EventListenerClass) -> EventListenerProperties {
        Self::GetEventListenerProperties(self, class)
    }
    fn property_change_forces_commit_criteria(&self) -> PropertyChangeForcesCommitCriteria {
        self.property_change_forces_commit_criteria
    }
    fn viewport_property_ids(&self) -> ViewportPropertyIds {
        self.viewport_property_ids
    }
    fn selection(&self) -> &crate::cc::input::layer_selection_bound::LayerSelection {
        &self.selection
    }
    fn page_scale_factor(&self) -> f32 {
        self.page_scale_factor
    }
    fn min_page_scale_factor(&self) -> f32 {
        self.min_page_scale_factor
    }
    fn max_page_scale_factor(&self) -> f32 {
        self.max_page_scale_factor
    }
    fn page_scale_factor_limits_set(&self) -> bool {
        self.page_scale_factor_limits_set
    }
    fn browser_controls_params(
        &self,
    ) -> &crate::cc::trees::browser_controls_params::BrowserControlsParams {
        &self.browser_controls_params
    }
    fn overscroll_behavior(&self) -> crate::cc::input::overscroll_behavior::OverscrollBehavior {
        self.overscroll_behavior
    }
    fn top_controls_shown_ratio(&self) -> f32 {
        self.top_controls_shown_ratio
    }
    fn bottom_controls_shown_ratio(&self) -> f32 {
        self.bottom_controls_shown_ratio
    }
    fn display_color_spaces(&self) -> &B::DisplayColorSpaces {
        &self.display_color_spaces
    }
    fn external_page_scale_factor(&self) -> f32 {
        self.external_page_scale_factor
    }
    fn painted_device_scale_factor(&self) -> f32 {
        self.painted_device_scale_factor
    }
    fn device_scale_factor(&self) -> f32 {
        self.device_scale_factor
    }
    fn device_viewport_rect(&self) -> Rect {
        self.device_viewport_rect
    }
    fn max_safe_area_inset_bottom(&self) -> f32 {
        self.max_safe_area_insets.bottom
    }
    fn new_local_surface_id_request(&self) -> bool {
        self.new_local_surface_id_request
    }
    fn screenshot_destination_token(&self) -> UnguessableToken {
        self.screenshot_destination_token
    }
    fn primary_main_frame_item_sequence_number(&self) -> i64 {
        self.primary_main_frame_item_sequence_number
    }
    fn local_surface_id_from_parent(&self) -> &B::LocalSurfaceId {
        &self.local_surface_id_from_parent
    }
    fn TakePendingPageScaleAnimation(
        &mut self,
    ) -> Option<Box<crate::cc::input::page_scale_animation::PendingPageScaleAnimation>> {
        self.pending_page_scale_animation.take()
    }
    fn force_send_metadata_request(&self) -> bool {
        self.force_send_metadata_request
    }
    fn display_transform_hint(&self) -> OverlayTransform {
        self.display_transform_hint
    }
    fn TakeDelegatedInkMetadata(&mut self) -> Option<Box<B::DelegatedInkMetadata>> {
        self.delegated_ink_metadata.take()
    }
    fn ViewTransitionRequests(&mut self) -> &mut Vec<Option<Box<B::ViewTransitionRequest>>> {
        &mut self.view_transition_requests
    }
}
impl<B: super::commit_state::CommitStateTypes> LayerTreeUnsafeCommitState<B>
    for crate::cc::trees::commit_state::ThreadUnsafeCommitState<B>
{
    fn HasRootLayer(&self) -> bool {
        self.root_layer.is_some()
    }
}

/// Live callbacks used by the source scroll/page-scale mutation call chain.
/// This subset cannot register input children and never holds their owner.
pub mod mutation {
    use super::*;
    pub trait LayerTreeScrollHost<B: LayerTreeTypes> {
        fn settings(&self) -> &B::Settings;
        fn IsCCSlimmingEnabled(&self) -> bool;
        fn TreesInVizInClientProcess(&self) -> bool;
        fn IsActiveTree(&self, id: LayerTreeId) -> bool;
        fn HasPendingTree(&self) -> bool;
        fn HasRecycleTree(&self) -> bool;
        fn RequestImplSideInvalidationForRasterInducingScroll(&mut self, id: ElementId);
        fn PendingTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool);
        fn RecycleTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool);
        fn DidAnimateScrollOffset(&mut self);
    }
    impl<B: LayerTreeTypes> LayerTreeScrollHost<B> for dyn LayerTreeHost<B> + '_ {
        fn settings(&self) -> &B::Settings {
            LayerTreeHost::settings(self)
        }
        fn IsCCSlimmingEnabled(&self) -> bool {
            LayerTreeHost::IsCCSlimmingEnabled(self)
        }
        fn TreesInVizInClientProcess(&self) -> bool {
            LayerTreeHost::TreesInVizInClientProcess(self)
        }
        fn IsActiveTree(&self, id: LayerTreeId) -> bool {
            LayerTreeHost::IsActiveTree(self, id)
        }
        fn HasPendingTree(&self) -> bool {
            LayerTreeHost::HasPendingTree(self)
        }
        fn HasRecycleTree(&self) -> bool {
            LayerTreeHost::HasRecycleTree(self)
        }
        fn RequestImplSideInvalidationForRasterInducingScroll(&mut self, id: ElementId) {
            LayerTreeHost::RequestImplSideInvalidationForRasterInducingScroll(self, id)
        }
        fn PendingTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool) {
            LayerTreeHost::PendingTreeDidUpdateScrollOffset(self, id, pushed)
        }
        fn RecycleTreeDidUpdateScrollOffset(&mut self, id: ElementId, pushed: bool) {
            LayerTreeHost::RecycleTreeDidUpdateScrollOffset(self, id, pushed)
        }
        fn DidAnimateScrollOffset(&mut self) {
            LayerTreeHost::DidAnimateScrollOffset(self)
        }
    }
    pub trait LayerTreeScaleHost<B: LayerTreeTypes>: LayerTreeScrollHost<B> {
        fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32);
        fn PendingTreeDidUpdatePageScale(&mut self);
        fn RecycleTreeDidUpdatePageScale(&mut self);
        fn OnPageScaleUpdated(&mut self, receiver: &mut LayerTreeImpl<B>);
        fn OuterViewportScrollNode<'a>(
            &'a self,
            receiver: &'a LayerTreeImpl<B>,
        ) -> Option<&'a crate::cc::trees::scroll_node::ScrollNode>;
        fn WithScrollbarAnimationController(
            &mut self,
            receiver: &mut LayerTreeImpl<B>,
            id: ElementId,
            callback:&mut dyn FnMut(Option<&mut crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>, &mut dyn crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationControllerClient<Transform=B::Transform>),
        );
    }
    impl<B: LayerTreeTypes> LayerTreeScaleHost<B> for dyn LayerTreeHost<B> + '_ {
        fn ReportUnexpectedOopifPageScaleOnce(&mut self, scale: f32) {
            LayerTreeHost::ReportUnexpectedOopifPageScaleOnce(self, scale)
        }
        fn PendingTreeDidUpdatePageScale(&mut self) {
            LayerTreeHost::PendingTreeDidUpdatePageScale(self)
        }
        fn RecycleTreeDidUpdatePageScale(&mut self) {
            LayerTreeHost::RecycleTreeDidUpdatePageScale(self)
        }
        fn OnPageScaleUpdated(&mut self, receiver: &mut LayerTreeImpl<B>) {
            LayerTreeHost::OnPageScaleUpdated(self, receiver)
        }
        fn OuterViewportScrollNode<'a>(
            &'a self,
            receiver: &'a LayerTreeImpl<B>,
        ) -> Option<&'a crate::cc::trees::scroll_node::ScrollNode> {
            LayerTreeHost::OuterViewportScrollNode(self, receiver)
        }
        fn WithScrollbarAnimationController(
            &mut self,
            receiver: &mut LayerTreeImpl<B>,
            id: ElementId,
            callback:&mut dyn FnMut(Option<&mut crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationController>, &mut dyn crate::cc::input::scrollbar_animation_controller::ScrollbarAnimationControllerClient<Transform=B::Transform>),
        ) {
            LayerTreeHost::WithScrollbarAnimationController(self, receiver, id, callback)
        }
    }
}

/// Capability split for native UpdateDrawProperties and hit-test callers. This
/// borrows original state; setters keep explicit unique host/map access. The
/// source draw context must drop before the factory's host borrow is reused.
pub mod draw_host {
    use super::*;
    pub trait LayerTreeDrawServices {
        fn SizeOopifEffectSurfacesAtExternalScale(&self) -> bool;
        fn AllowUndamagedNonrootRenderPassToSkip(&self) -> bool;
        fn ViewTransitionsNewRoundingChange(&self) -> bool;
        fn LogDoubleBlur(&self, first: &str, second: &str);
        fn LogMaxTextureExceeded(&self, maximum: i32, width: i32, height: i32);
        fn TraceCategoryEnabled(&self) -> bool;
        fn TraceRenderSurfaceReasonCount(&self, name: &str, count: usize);
        fn TraceScope(&self, name: &str, count: Option<usize>);
    }
    pub trait LayerTreeDrawHost<B: LayerTreeTypes>:
        super::super::picture_layer_impl::PictureLayerContextHost<B>
    {
        fn DrawServices(&self) -> &dyn LayerTreeDrawServices;
        fn settings(&self) -> &B::Settings;
        fn IsActiveTree(&self, id: LayerTreeId) -> bool;
        fn HasLayerTreeFrameSink(&self) -> bool;
        fn IsResourcelessSoftwareDraw(&self) -> bool;
        fn HasImageAnimationController(&self) -> bool;
        fn ViewTransitionCaptureAndDisplay(&self) -> bool;
        fn IsCCSlimmingEnabled(&self) -> bool;
        fn DrawTransform(&self) -> &B::Transform;
        fn external_viewport(&self) -> Rect;
        fn max_texture_size(&self) -> i32;
        fn SetViewTransitionContentRect(
            &mut self,
            sequence: u32,
            id: &ViewTransitionElementResourceId,
            rect: &RectF,
        );
        fn UpdateImageAnimationStateFromDrivers(&mut self, receiver: &mut LayerTreeImpl<B>);
        fn DidModifyTilePriorities(&mut self, pending_update_tiles: bool);
        fn TraceLayersUpdatedCount(&self, count: usize);
    }
    /// This single borrowed capability adapts an existing full native host. It
    /// owns no flags, trees, requests, map or callback beyond this invocation.
    pub struct LayerTreeDrawHostRef<'a, H: ?Sized>(pub &'a mut H);
    impl<B: LayerTreeTypes> super::super::picture_layer_impl::PictureLayerContextHost<B>
        for LayerTreeDrawHostRef<'_, dyn LayerTreeHost<B> + '_>
    {
        fn WithPictureContextParts(
            &mut self,
            selected: LayerTreeId,
            callback: &mut dyn FnMut(
                &dyn super::super::picture_layer_impl::PictureLayerReadHost<B>,
                &mut dyn super::super::picture_layer_impl::PictureLayerTreeHost<B>,
                &mut dyn super::super::picture_layer_impl::PictureLayerImplServices<B>,
                &mut dyn super::super::picture_layer_impl::PictureLayerImplTilingEnvironment<B>,
            ),
        ) where
            B: super::super::picture_layer_impl::PictureLayerImplTypes,
        {
            self.0.WithPictureContextParts(selected, callback)
        }
    }
    impl<B: LayerTreeTypes> LayerTreeDrawHost<B>
        for LayerTreeDrawHostRef<'_, dyn LayerTreeHost<B> + '_>
    {
        fn DrawServices(&self) -> &dyn LayerTreeDrawServices {
            self.0.DrawServices()
        }
        fn settings(&self) -> &B::Settings {
            self.0.settings()
        }
        fn IsActiveTree(&self, id: LayerTreeId) -> bool {
            self.0.IsActiveTree(id)
        }
        fn HasLayerTreeFrameSink(&self) -> bool {
            self.0.HasLayerTreeFrameSink()
        }
        fn IsResourcelessSoftwareDraw(&self) -> bool {
            self.0.IsResourcelessSoftwareDraw()
        }
        fn HasImageAnimationController(&self) -> bool {
            self.0.HasImageAnimationController()
        }
        fn ViewTransitionCaptureAndDisplay(&self) -> bool {
            self.0.ViewTransitionCaptureAndDisplay()
        }
        fn IsCCSlimmingEnabled(&self) -> bool {
            self.0.IsCCSlimmingEnabled()
        }
        fn DrawTransform(&self) -> &B::Transform {
            self.0.DrawTransform()
        }
        fn external_viewport(&self) -> Rect {
            self.0.external_viewport()
        }
        fn max_texture_size(&self) -> i32 {
            self.0.max_texture_size()
        }
        fn SetViewTransitionContentRect(
            &mut self,
            sequence: u32,
            id: &ViewTransitionElementResourceId,
            rect: &RectF,
        ) {
            self.0.SetViewTransitionContentRect(sequence, id, rect)
        }
        fn UpdateImageAnimationStateFromDrivers(&mut self, receiver: &mut LayerTreeImpl<B>) {
            self.0.UpdateImageAnimationStateFromDrivers(receiver)
        }
        fn DidModifyTilePriorities(&mut self, pending_update_tiles: bool) {
            self.0.DidModifyTilePriorities(pending_update_tiles)
        }
        fn TraceLayersUpdatedCount(&self, count: usize) {
            self.0.TraceLayersUpdatedCount(count)
        }
    }
}

/// Explicit receiver of a source LayerImpl -> owning-tree synchronous callback.
/// The receiver stays in its original resident slot; the remaining-storage view
/// cannot resolve that slot without this live argument.
pub trait LayerTreeCurrentReceiver<B: LayerTreeTypes> {
    fn id(&self) -> i32;
    fn WithScrollbarGeometry(
        &mut self,
        callback: &mut dyn FnMut(&mut dyn LayerTreeScrollbarGeometry<B>),
    );
}
pub trait LayerTreeRemainingStorage<B: LayerTreeTypes> {
    fn LayerObjectIdentity(&self, id: i32) -> Option<usize>;
    fn selected_id(&self) -> i32;
    fn WithScrollbarGeometry(
        &mut self,
        id: i32,
        callback: &mut dyn FnMut(&mut dyn LayerTreeScrollbarGeometry<B>),
    );
}

/// Source scrollbar-controller lookup consumes only these live scroll nodes.
/// Both a resident LTI and a selected-layer split expose the same native nodes.
pub trait ScrollbarControllerTree {
    fn InnerViewportScrollNode(&self) -> Option<&crate::cc::trees::scroll_node::ScrollNode>;
    fn OuterViewportScrollNode(&self) -> Option<&crate::cc::trees::scroll_node::ScrollNode>;
}
pub struct ScrollbarControllerTreeFields<'a, B: LayerTreeTypes> {
    pub property_trees: &'a PropertyTreesFor<B>,
    pub viewport: &'a crate::cc::trees::viewport_property_ids::ViewportPropertyIds,
}
impl<B: LayerTreeTypes> ScrollbarControllerTree for ScrollbarControllerTreeFields<'_, B> {
    fn InnerViewportScrollNode(&self) -> Option<&crate::cc::trees::scroll_node::ScrollNode> {
        let id = self.viewport.inner_scroll;
        (id != crate::cc::trees::property_ids::kInvalidPropertyNodeId)
            .then(|| self.property_trees.scroll_tree().Node(id))
    }
    fn OuterViewportScrollNode(&self) -> Option<&crate::cc::trees::scroll_node::ScrollNode> {
        let id = self.viewport.outer_scroll;
        (id != crate::cc::trees::property_ids::kInvalidPropertyNodeId)
            .then(|| self.property_trees.scroll_tree().Node(id))
    }
}
/// Full-lifecycle-only host callbacks. cc registration/map/opacity decisions
/// remain in canonical LTI; the client resolves actual current/rest storage.
pub trait LayerTreeScrollbarRegistrationHost<B: LayerTreeTypes> {
    fn DidRegisterScrollbarLayer(&mut self, id: ElementId, orientation: ScrollbarOrientation);
    fn DidUnregisterScrollbarLayer(
        &mut self,
        id: ElementId,
        orientation: ScrollbarOrientation,
        client: &mut dyn super::scrollbar::ScrollbarInitializationClient,
    );
    fn RegisterScrollbarAnimationController(
        &mut self,
        id: ElementId,
        opacity: f32,
        tree: &dyn ScrollbarControllerTree,
        client: &mut dyn super::scrollbar::ScrollbarInitializationClient,
    );
}

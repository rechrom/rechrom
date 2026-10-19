//! Direct resident PaintArtifact boundary for Blink PendingLayer. The subset
//! retains the Paint owner's immutable Arc and source indices; no chunk,
//! display-item, property-tree, or layer snapshot is created here.

use super::{
    geometry::{Rect, RectF},
    layer_tree_impl::Color4f,
    main_layer::MainLayerTypes,
    property_types::BlendMode,
};
use crate::cc::input::hit_test_opaqueness::HitTestOpaqueness;
use crate::cc::layers::layer::LayerRef;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PendingChunkBackgroundColor {
    pub color: Color4f,
    pub area: f32,
    pub is_solid_color: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LcdTextPreference {
    kStronglyPreferred,
    kWeaklyPreferred,
    kIgnored,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingUpcastResult<S> {
    pub upcasted_state: S,
    pub scroll_range_dependent: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PendingMappedVisualRect {
    pub rect: RectF,
    pub is_tight: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PendingLayerJson {
    pub debug_name: String,
    pub bounds: RectF,
    pub rect_known_to_be_opaque: RectF,
    pub text_known_to_be_on_opaque_background: bool,
    pub property_tree_state: String,
    pub offset_of_decomposited_transforms: super::geometry::Vector2dF,
    pub paint_chunks: String,
    pub draws_content: bool,
    pub is_solid_color: bool,
    pub hit_test_opaqueness: String,
}

pub trait PendingLayerTypes:
    MainLayerTypes
    + super::picture_layer::PictureLayerTypes<
        Recording: super::recording_source::RecordingSourceTypes<
            DisplayItemList = Self::ContentDisplayItemList,
        >,
    > + Sized
{
    type ContentDisplayItemList: super::content_layer_client_impl::ContentDisplayItemList<Self>
        + Send
        + Sync;
    type RasterInvalidator: super::content_layer_client_impl::RasterInvalidator<Self> + Send + Sync;
    type RasterInvalidationTracking: super::content_layer_client_impl::RasterInvalidationTracking<
        Self,
    >;
    type PaintRecord;
    type PaintOpBuffer: super::content_layer_client_impl::ContentPaintOpBuffer<Self>;
    type Json: super::content_layer_client_impl::ContentJson<Self>;
    type JsonArray: Clone + Send + Sync;
    type LoggingCanvas: super::content_layer_client_impl::ContentLoggingCanvas<Self>;
    type PaintArtifact: PendingPaintArtifact<Self> + Send + Sync + 'static;
    type PaintChunk: PendingPaintChunk<Self>;
    type DisplayItem: PendingDisplayItem<Self>;
    type PaintChunkSubset: PendingPaintChunkSubset<Self>;
    type PropertyTreeState: PendingPropertyTreeState<
            PaintPropertyChange = Self::PaintPropertyChange,
            TransformNode = Self::TransformNode,
            ClipNode = Self::ClipNode,
            EffectNode = Self::EffectNode,
        > + Clone
        + PartialEq;
    /// A retained identity/handle to Paint's resident transform node.
    type TransformNode: PendingTransformNode<
            Self::PaintPropertyChange,
            Matrix = <Self as super::layer_tree_impl::LayerTreeTypes>::Transform,
        > + Clone
        + Eq
        + Hash;
    type ClipNode: PendingClipNode<
            Self::TransformNode,
            Self::PropertyNodeIdentity,
            EffectNode = Self::EffectNode,
        > + Clone
        + PartialEq;
    type EffectNode: PendingEffectNode<
            Self::TransformNode,
            Self::PropertyNodeIdentity,
            Filters = <Self as super::layer_tree_impl::LayerTreeTypes>::Filters,
            Path = <Self as super::layer_tree_impl::LayerTreeTypes>::Path,
            PaintPropertyChange = Self::PaintPropertyChange,
            ClipNode = Self::ClipNode,
        > + Clone
        + PartialEq;
    type PropertyNodeIdentity: Copy + Eq + Hash;
    type PaintPropertyChange: Copy + Ord;
    type DomNodeId: Copy + Eq;
    type ClientId: Copy;
    type CanvasChildPaintRecord: Clone;
    type CanvasChildPaintState: super::content_layer_client_impl::CanvasChildPaintState
        + Clone
        + Send
        + Sync;

    fn UnchangedPaintPropertyChange() -> Self::PaintPropertyChange;
    fn ChangedOnlySimpleValues() -> Self::PaintPropertyChange;
    fn InvalidDomNodeId() -> Self::DomNodeId;
}

pub trait PendingDisplayItem<B: PendingLayerTypes> {
    fn ForeignLayerVisualRect(&self) -> RectF;
    fn ForeignLayer(&self) -> LayerRef<B>;
    fn IsScrollbar(&self) -> bool;
    fn CreateOrReuseScrollbarLayer(
        &self,
        old_layer: Option<LayerRef<B>>,
        decomposited_offset: super::geometry::Vector2dF,
    ) -> LayerRef<B>;
}

pub trait PendingLayerServices<B: PendingLayerTypes> {
    /// Constructs the canonical cc::SolidColorLayer owner. The returned base
    /// handle points at that most-derived resident object.
    fn CreateSolidColorLayer(
        &mut self,
        layer_services: &dyn super::main_layer::MainLayerServices,
    ) -> LayerRef<B>;
}

pub trait PendingPropertyTreeState {
    type PaintPropertyChange: Copy;
    type TransformNode: Clone + Eq;
    type ClipNode: Clone;
    type EffectNode: Clone;
    fn EffectRequiresCompositingForBackdropFilterMask(&self) -> bool;
    fn EffectRequiresCompositingForCanvasChild(&self) -> bool;
    fn ChangedToRoot(&self, change: Self::PaintPropertyChange) -> bool;
    fn SameEffectNode(&self, other: &Self) -> bool;
    fn EffectUnaliasedParentSameAs(&self, ancestor: &Self) -> bool;
    fn EffectIsOpacityOnly(&self) -> bool;
    fn EffectOpacity(&self) -> f32;
    fn EffectBlendMode(&self) -> BlendMode;
    fn CanUpcastWith(
        &self,
        other: &Self,
        merge_fixed_layers: bool,
        merge_sticky_layers: bool,
        is_composited_scroll: &mut dyn FnMut(&Self::TransformNode) -> bool,
    ) -> Option<PendingUpcastResult<Self>>
    where
        Self: Sized;
    fn NearestScrollTranslationNode(&self) -> Self::TransformNode;
    fn SameNearestDirectlyCompositedAncestor(&self, other: &Self) -> bool;
    fn ToDebugString(&self) -> String;
    fn TransformNode(&self) -> Self::TransformNode;
    fn ClipNode(&self) -> Self::ClipNode;
    fn EffectNode(&self) -> Self::EffectNode;
    fn SetTransformNode(&mut self, transform: Self::TransformNode);
}

pub trait PendingTransformNode<C: Copy>: Clone + Eq + Hash {
    type Matrix: Clone;
    fn CcNodeId(&self, sequence: i32) -> i32;
    fn SetCcNodeId(&self, sequence: i32, id: i32);
    fn IsRoot(&self) -> bool;
    fn UnaliasedParent(&self) -> Option<Self>;
    fn ParentUnalias(&self) -> Self;
    fn IsIdentityOr2dTranslation(&self) -> bool;
    fn HasScrollNode(&self) -> bool;
    fn HasDirectCompositingReasonsOtherThan3dTransform(&self) -> bool;
    fn FlattensInheritedTransformSameAsParent(&self) -> bool;
    fn BackfaceVisibilitySameAsParent(&self) -> bool;
    fn HasStickyConstraint(&self) -> bool;
    fn HasAnchorPositionScrollData(&self) -> bool;
    fn IsAffectedByOuterViewportBoundsDelta(&self) -> bool;
    fn IsAffectedBySafeAreaBottom(&self) -> bool;
    fn IsInSubtreeOfPageScale(&self) -> bool;
    fn RequiresCompositingForFixedToViewport(&self) -> bool;
    fn FlattensInheritedTransform(&self) -> bool;
    fn RenderingContextId(&self) -> i32;
    fn DelegatesToParentForBackface(&self) -> bool;
    fn RequiresCompositingForWillChangeTransform(&self) -> bool;
    fn IsForSVGChild(&self) -> bool;
    fn ScrollParentScrollTranslation(&self) -> Option<Self>;
    fn PropertyTreeNearestScrollTranslationNode(&self) -> Self;
    fn GetStickyConstraint(
        &self,
    ) -> Option<crate::cc::trees::sticky_position_constraint::StickyPositionConstraint>;
    fn GetAnchorPositionScrollData(
        &self,
    ) -> Option<crate::cc::trees::property_tree::AnchorPositionScrollData>;
    fn GetCompositorElementId(&self) -> super::input_types::ElementId;
    fn GetVisibleFrameElementId(&self) -> super::input_types::ElementId;
    fn IsFramePaintOffsetTranslation(&self) -> bool;
    fn IsBackfaceHidden(&self) -> bool;
    fn HasActiveTransformAnimation(&self) -> bool;
    fn TransformAnimationIsAxisAligned(&self) -> bool;
    fn PropertyTreeLowestCommonAncestor(&self, other: &Self) -> Self;
    fn Get2dTranslation(&self) -> super::geometry::Vector2dF;
    fn Matrix(&self) -> Self::Matrix;
    fn Origin(&self) -> super::geometry::Point3F;
    fn NodeChanged(&self) -> C;
    fn ScrollCompositorElementId(&self) -> super::input_types::ElementId;
    fn ScrollContainerRect(&self) -> Rect;
}

pub trait PendingClipNode<T, I>: Clone {
    type EffectNode: Clone;
    fn CcNodeId(&self, sequence: i32) -> i32;
    fn SetCcNodeId(&self, sequence: i32, id: i32);
    fn IsRoot(&self) -> bool;
    fn Identity(&self) -> I;
    fn ParentUnalias(&self) -> Self;
    fn LocalTransformSpaceUnalias(&self) -> T;
    /// Direct projection of PaintClipRect().Rect() for the property-tree
    /// builder; the rounded-rect owner remains the Paint conversion adapter.
    fn PaintClipRectRect(&self) -> RectF;
    fn LowestCommonAncestor(&self, other: &Self) -> Self;
    fn IsAncestorOf(&self, other: &Self) -> bool;
    fn PaintClipRectIsRounded(&self) -> bool;
    fn PaintClipRRect(&self) -> Option<super::geometry::RRectF>;
    fn HasClipPath(&self) -> bool;
    fn PixelMovingFilter(&self) -> Option<Self::EffectNode>;
}

pub trait PendingEffectNode<T, I>: Clone {
    type Filters;
    type Path;
    type PaintPropertyChange: Copy;
    type ClipNode: Clone;
    fn CcNodeId(&self, sequence: i32) -> i32;
    fn SetCcNodeId(&self, sequence: i32, id: i32);
    fn IsRoot(&self) -> bool;
    fn Identity(&self) -> I;
    fn ParentUnalias(&self) -> Self;
    fn LocalTransformSpaceUnalias(&self) -> T;
    fn BlendMode(&self) -> BlendMode;
    fn Opacity(&self) -> f32;
    fn HasActiveOpacityAnimation(&self) -> bool;
    fn HasFilter(&self) -> bool;
    fn RequiresCompositingForWillChangeFilter(&self) -> bool;
    fn HasActiveFilterAnimation(&self) -> bool;
    fn HasBackdropFilter(&self) -> bool;
    fn RequiresCompositingForWillChangeBackdropFilter(&self) -> bool;
    fn HasActiveBackdropFilterAnimation(&self) -> bool;
    fn HasValidViewTransitionElementResourceId(&self) -> bool;
    fn RequiresCompositingForUnboundedElement(&self) -> bool;
    fn FlattensAtLeafOf3DScene(&self) -> bool;
    fn HasElementCaptureId(&self) -> bool;
    fn LowestCommonAncestor(&self, other: &Self) -> Self;
    fn MayHaveBackdropEffect(&self) -> bool;
    fn OutputClip(&self) -> Option<Self::ClipNode>;
    fn GetCompositorElementId(&self) -> super::input_types::ElementId;
    fn NeedsEffectFor2DScaleTransform(&self) -> bool;
    fn FilterOperations(&self) -> Option<Self::Filters>;
    fn BackdropFilterOperations(&self) -> Option<Self::Filters>;
    fn PropertyTreeBackdropFilterBounds(&self) -> Option<Self::Path>;
    fn BackdropMaskElementId(&self) -> super::input_types::ElementId;
    fn NodeChanged(&self) -> Self::PaintPropertyChange;
    fn ViewTransitionElementResourceId(
        &self,
    ) -> super::property_types::ViewTransitionElementResourceId;
    fn SubtreeCaptureId(&self) -> super::property_types::SubtreeCaptureId;
}

/// Exact ui::color_utils source-over primitive. PendingLayer retains the
/// source selection and ordering algorithm; this boundary owns only the
/// external color representation's conversion and rounding semantics.
pub trait PendingLayerColor {
    fn GetResultingPaintColor(&self, foreground: Color4f, background: Color4f) -> Color4f;
}

pub trait PendingPaintArtifact<B: PendingLayerTypes> {
    fn PaintChunk(&self, index: usize) -> Option<&B::PaintChunk>;
    fn ClientDebugName(&self, client: B::ClientId) -> String;
    fn ClientOwnerNodeId(&self, client: B::ClientId) -> B::DomNodeId;
    fn PaintChunkCount(&self) -> usize;
}

pub trait PendingPaintChunk<B: PendingLayerTypes> {
    fn HasHitTestData(&self) -> bool;
    fn PropertiesUnalias(&self) -> B::PropertyTreeState;
    fn client_id(&self) -> B::ClientId;
    fn bounds(&self) -> RectF;
    fn rect_known_to_be_opaque(&self) -> RectF;
    fn background_color(&self) -> PendingChunkBackgroundColor;
    fn hit_test_opaqueness(&self) -> HitTestOpaqueness;
    fn has_text(&self) -> bool;
    fn text_known_to_be_on_opaque_background(&self) -> bool;
    fn DrawsContent(&self) -> bool;
    fn display_item_count(&self) -> usize;
    fn non_composited_scroll_translation(&self) -> Option<B::TransformNode>;
    fn Matches(&self, other: &Self) -> bool;
    fn first_display_item_is_foreign_layer_video(&self, artifact: &B::PaintArtifact) -> bool;
    fn FirstDisplayItem<'a>(&self, artifact: &'a B::PaintArtifact) -> &'a B::DisplayItem;
    fn is_moved_from_cached_subsequence(&self) -> bool;
}

pub trait PendingPaintChunkSubset<B: PendingLayerTypes>: Sized {
    /// Creates the source PaintChunkSubset from one index while retaining the
    /// exact immutable artifact owner passed by Page/PaintEngine.
    fn FromFirst(artifact: Arc<B::PaintArtifact>, first_chunk_index: usize) -> Self;
    fn artifact(&self) -> &Arc<B::PaintArtifact>;
    fn SetPaintArtifact(&mut self, artifact: Arc<B::PaintArtifact>);
    fn Merge(&mut self, guest: &Self);
    fn len(&self) -> usize;
    fn chunk(&self, index: usize) -> &B::PaintChunk;
    fn ToDebugJson(&self) -> String;
}

/// Paint's canonical GeometryMapper operation used by PendingLayer. The
/// implementation reads resident property nodes; it does not materialize a
/// compositor-owned property tree.
pub trait PendingLayerGeometry<B: PendingLayerTypes> {
    fn VisibilityLimit(&self, state: &B::PropertyTreeState) -> Option<RectF>;
    fn MightOverlapForCompositing(
        &self,
        first_bounds: RectF,
        first_state: &B::PropertyTreeState,
        second_bounds: RectF,
        second_state: &B::PropertyTreeState,
    ) -> bool;
    fn MapLocalToAncestorVisualRect(
        &self,
        source: &B::PropertyTreeState,
        destination: &B::PropertyTreeState,
        rect: RectF,
    ) -> PendingMappedVisualRect;
    fn MaximumCoveredRect(&self, first: RectF, second: RectF) -> RectF;
    fn UnionRects(&self, first: RectF, second: RectF) -> RectF;
    fn ToRoundedRect(&self, rect: RectF) -> Rect;
    fn MergeFixedLayersEnabled(&self) -> bool;
    fn MergeStickyLayersEnabled(&self) -> bool;
}

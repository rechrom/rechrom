//! Live geometry-mapper boundary used by Blink PaintArtifact layerization.
//! The Paint owner supplies its resident transform nodes and canonical mapper;
//! compositor does not clone the property tree or implement another mapper.

use super::geometry::{RectF, Vector2dF};
/// Exact external gfx value primitives consumed by native FloatClipRect.
pub trait FloatClipRectTransform {
    fn IsIdentityOr2dTranslation(&self) -> bool;
    fn To2dTranslation(&self) -> super::geometry::Vector2dF;
    fn MapRect(&self, rect: RectF) -> RectF;
}

/// Source transform-node queries. Implementations lend the actual Paint node
/// and its own cache slot; they do not implement cache/projection algorithms.
pub trait TransformCacheNode: Clone + PartialEq + Send + Sync + 'static {
    type Matrix: super::property_tree::PropertyTreeTransform + Send + Sync + 'static;
    type Weak: Clone + Send + Sync + 'static;
    fn Downgrade(&self) -> Self::Weak;
    fn Upgrade(weak: &Self::Weak) -> Self;
    fn CacheSlot(&self) -> &paint::opaque_paint_cache_slot::OpaquePaintCacheSlot;
    fn IsRoot(&self) -> bool;
    fn IsIdentity(&self) -> bool;
    fn Parent(&self) -> Option<Self>;
    fn RequiresCompositingForStickyPosition(&self) -> bool;
    fn RequiresCompositingForAnchorPosition(&self) -> bool;
    fn IsBackfaceHiddenInternal(&self, parent_hidden: bool) -> bool;
    fn HasScrollNode(&self) -> bool;
    fn ScrollParentScrollTranslation(&self) -> Option<Self>;
    fn HasDirectCompositingReasons(&self) -> bool;
    fn IsIdentityOr2dTranslation(&self) -> bool;
    fn HasActiveTransformAnimation(&self) -> bool;
    fn Get2dTranslation(&self) -> Vector2dF;
    fn MatrixWithOriginApplied(&self) -> Self::Matrix;
    fn FlattensInheritedTransform(&self) -> bool;
}

pub trait MaskGeometryMapper {
    type TransformPaintPropertyNode: ?Sized;

    fn SourceToDestinationRectFromRoot(
        &self,
        transform: &Self::TransformPaintPropertyNode,
        rect: &mut RectF,
    );
}

pub use compositor_layerize::paint_geometry::{ChunkGeometryMapper, RasterEffectOutset};

/// Facade-owned projection of an external Paint chunk. Keeping this trait in
/// the facade permits native Paint types to implement it without orphan-rule
/// coupling between the responsibility packages.
pub trait PaintChunkView<M: ChunkGeometryMapper + ?Sized> {
    fn properties_unalias(&self) -> M::PropertyTreeState;
    fn raster_effect_outset(&self) -> RasterEffectOutset;
}

/// Exact live node reads consumed by GeometryMapper; no mapping algorithm is
/// implemented by this capability. Arc values retain only native identities.
pub trait GeometryMapperNode: TransformCacheNode {
    fn RawNode(&self) -> &std::sync::Arc<paint::paint_property_tree::TransformPaintPropertyNode>;
    fn OriginIsOrigin(&self) -> bool;
    fn Matrix(&self) -> Self::Matrix;
    fn LowestCommonAncestor(&self, other: &Self) -> Self;
    fn ParentScrollTranslationNode(&self) -> Self;
    fn ScrollContentsRect(&self) -> super::geometry::Rect;
    fn ScrollContainerRect(&self) -> super::geometry::Rect;
    fn ScrollOverflowClip(
        &self,
    ) -> Option<std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>>;
}
pub trait GeometryMapperState: Clone {
    type Node: GeometryMapperNode;
    type Effect: Clone + PartialEq;
    fn Transform(&self) -> &Self::Node;
    fn Clip(&self) -> &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>;
    fn Effect(&self) -> &Self::Effect;
    fn SetClip(&mut self, clip: std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>);
    fn SetEffect(&mut self, effect: Self::Effect);
}
pub trait GeometryMapperContext {
    type State: GeometryMapperState;
    fn SingularTransformFixEnabled(&self) -> bool;
    fn RootState(&self) -> Self::State;
    fn MakeState(
        &self,
        transform: <Self::State as GeometryMapperState>::Node,
        clip: std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
        effect: <Self::State as GeometryMapperState>::Effect,
    ) -> Self::State;
    fn ClipLocalTransform(
        &self,
        clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
    ) -> <Self::State as GeometryMapperState>::Node;
    fn ClipLowestCommonAncestor(
        &self,
        first: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
        second: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
    ) -> std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>;
    fn LayoutClipRectExcludingOverlayScrollbars(&self, clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>) -> crate::third_party::blink::renderer::platform::graphics::paint::float_clip_rect::FloatClipRect;
    fn PreciseLayoutClipRect(&self, clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>) -> crate::third_party::blink::renderer::platform::graphics::paint::float_clip_rect::FloatClipRect;
    fn LayoutClipRect(&self, clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>) -> crate::third_party::blink::renderer::platform::graphics::paint::float_clip_rect::FloatClipRect;
    fn PaintClipRect(
        &self,
        clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
    ) -> RectF;
    fn ClipHasPath(
        &self,
        clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
    ) -> bool;
    fn IsForCompositeClipPathAnimation(
        &self,
        clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
    ) -> bool;
    fn PixelMovingFilter(
        &self,
        clip: &std::sync::Arc<paint::paint_property_tree::ClipPaintPropertyNode>,
    ) -> <Self::State as GeometryMapperState>::Effect;
    fn FilterLocalTransform(
        &self,
        effect: &<Self::State as GeometryMapperState>::Effect,
    ) -> <Self::State as GeometryMapperState>::Node;
    fn HasActiveFilterAnimation(
        &self,
        effect: &<Self::State as GeometryMapperState>::Effect,
    ) -> bool;
    fn FilterMapRect(
        &self,
        effect: &<Self::State as GeometryMapperState>::Effect,
        rect: super::geometry::Rect,
    ) -> super::geometry::Rect;
    fn SelfOrAncestorParticipatesInViewTransition(
        &self,
        effect: &<Self::State as GeometryMapperState>::Effect,
    ) -> bool;
}

// Source: third_party/blink/renderer/platform/graphics/visual_rect_flags.h:12-56.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum VisualRectFlag {
    kEdgeInclusive = 0,
    kIgnoreFilters = 1,
    kUseGeometryMapper = 2,
    kDontApplyMainFrameOverflowClip = 3,
    kIgnoreLocalClipPath = 4,
    kApplyRemoteViewportTransform = 5,
    kUsePreciseClipPath = 6,
    kSkipAncestorAndViewportClips = 7,
}
impl super::enum_set::EnumValue for VisualRectFlag {
    fn to_integer(self) -> i32 {
        self as i32
    }
    fn from_integer(value: i32) -> Self {
        match value {
            0 => Self::kEdgeInclusive,
            1 => Self::kIgnoreFilters,
            2 => Self::kUseGeometryMapper,
            3 => Self::kDontApplyMainFrameOverflowClip,
            4 => Self::kIgnoreLocalClipPath,
            5 => Self::kApplyRemoteViewportTransform,
            6 => Self::kUsePreciseClipPath,
            7 => Self::kSkipAncestorAndViewportClips,
            _ => panic!("invalid VisualRectFlag"),
        }
    }
}
pub type VisualRectFlags = super::enum_set::EnumSet<VisualRectFlag, 0, 7>;

/// Native gfx::Transform::MapRect overloads used by GeometryMapper's source
/// Rect template. Integer identity preserves the original full i32 endpoints.
pub trait GeometryMapperRect<M> {
    fn Mapped(&self, matrix: &M) -> Self;
}
impl<M: FloatClipRectTransform> GeometryMapperRect<M> for RectF {
    fn Mapped(&self, matrix: &M) -> Self {
        matrix.MapRect(*self)
    }
}
impl<M: FloatClipRectTransform + Default + PartialEq> GeometryMapperRect<M>
    for super::geometry::Rect
{
    fn Mapped(&self, matrix: &M) -> Self {
        // Source: ui/gfx/geometry/transform.cc:804-809.
        if *matrix == M::default() {
            return *self;
        }
        super::geometry::ToEnclosingRect(matrix.MapRect(RectF::new(
            self.x as f32,
            self.y as f32,
            self.width as f32,
            self.height as f32,
        )))
    }
}

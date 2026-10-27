use compositor_tree::geometry::{RectF, Vector2dF};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RasterEffectOutset {
    #[default]
    kNone,
    kHalfPixel,
    kWholePixel,
}

pub trait ChunkGeometryMapper {
    type PropertyTreeState: Clone + PartialEq;
    type Transform: Clone;
    type FloatClipRect: Clone;

    fn MakeTranslation(&self, offset: Vector2dF) -> Self::Transform;
    fn MakeDefaultClipRect(&self) -> Self::FloatClipRect;
    fn MakeFloatClipRect(&self, rect: RectF) -> Self::FloatClipRect;
    fn InfiniteLooseFloatClipRect(&self) -> Self::FloatClipRect;
    fn SameTransformNode(
        &self,
        first: &Self::PropertyTreeState,
        second: &Self::PropertyTreeState,
    ) -> bool;
    fn SameClipNode(
        &self,
        first: &Self::PropertyTreeState,
        second: &Self::PropertyTreeState,
    ) -> bool;
    fn SameNearestPixelMovingFilterClip(
        &self,
        first: &Self::PropertyTreeState,
        second: &Self::PropertyTreeState,
    ) -> bool;
    fn SourceToDestinationProjection(
        &self,
        source: &Self::PropertyTreeState,
        destination: &Self::PropertyTreeState,
        output: &mut Self::Transform,
    ) -> bool;
    fn PostTranslate(&self, transform: &mut Self::Transform, offset: Vector2dF);
    fn MakeIdentity(&self, transform: &mut Self::Transform);
    fn ScaleToZero(&self, transform: &mut Self::Transform);
    fn MapRect(&self, transform: &Self::Transform, rect: RectF) -> RectF;
    fn LocalToAncestorClipRect(
        &self,
        local: &Self::PropertyTreeState,
        ancestor: &Self::PropertyTreeState,
    ) -> Self::FloatClipRect;
    fn LocalToAncestorVisualRect(
        &self,
        local: &Self::PropertyTreeState,
        ancestor: &Self::PropertyTreeState,
        rect: &mut Self::FloatClipRect,
    );
    fn ClipIsInfinite(&self, clip: &Self::FloatClipRect) -> bool;
    fn ClipRect(&self, clip: &Self::FloatClipRect) -> RectF;
    fn MoveClip(&self, clip: &mut Self::FloatClipRect, offset: Vector2dF);
}

// Translation status: complete
// Copyright 2016 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:1-30;
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:1-16.
use super::{
    float_clip_rect::{FloatClipRect, InfiniteLooseFloatClipRect},
    geometry_mapper_clip_cache::{
        ClipAndTransform, ClipCacheEntry, GeometryMapperClipCache, OverlayScrollbarClipBehavior,
        WithClipCache,
    },
    geometry_mapper_transform_cache::{GeometryMapperTransformCache, WithTransformCache},
};
use crate::adapter::{
    geometry::{RectF, ToEnclosingRect},
    paint_geometry::{
        FloatClipRectTransform, GeometryMapperContext, GeometryMapperNode, GeometryMapperRect,
        GeometryMapperState, TransformCacheNode, VisualRectFlag, VisualRectFlags,
    },
    property_tree::PropertyTreeTransform,
};
use paint::paint_property_tree::ClipPaintPropertyNode;
use std::{marker::PhantomData, sync::Arc};
type Node<C> = <<C as GeometryMapperContext>::State as GeometryMapperState>::Node;
type Matrix<C> = <Node<C> as TransformCacheNode>::Matrix;
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:203-209,216-218.
#[derive(Default)]
struct ExtraProjectionResult {
    has_animation: bool,
    has_sticky_or_anchor_position: bool,
}
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:29-30.
pub struct GeometryMapper<C: GeometryMapperContext>(PhantomData<C>);
fn same_clip(a: &Arc<ClipPaintPropertyNode>, b: &Arc<ClipPaintPropertyNode>) -> bool {
    Arc::ptr_eq(a, b)
}
fn same_optional_clip(
    a: &Option<Arc<ClipPaintPropertyNode>>,
    b: &Option<Arc<ClipPaintPropertyNode>>,
) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => same_clip(a, b),
        _ => false,
    }
}
fn nearest_filter(clip: &Arc<ClipPaintPropertyNode>) -> Option<Arc<ClipPaintPropertyNode>> {
    WithClipCache(clip, |cache| cache.NearestPixelMovingFilterClip())
}
impl<C: GeometryMapperContext> GeometryMapper<C>
where
    Matrix<C>: FloatClipRectTransform,
{
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:18-46.
    fn ScrollContainerState(context: &C, scroll: &Node<C>) -> C::State {
        let root = context.RootState();
        let mut state = context.MakeState(
            scroll.Parent().expect("source scroll parent"),
            root.Clip().clone(),
            root.Effect().clone(),
        );
        if let Some(clip) = scroll.ScrollOverflowClip() {
            state.SetClip(clip.parent.clone().expect("source overflow clip parent"));
        }
        state
    }
    fn ScrollingContentsState(context: &C, scroll: &Node<C>) -> C::State {
        let root = context.RootState();
        let mut state =
            context.MakeState(scroll.clone(), root.Clip().clone(), root.Effect().clone());
        if let Some(clip) = scroll.ScrollOverflowClip() {
            state.SetClip(clip);
        }
        state
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:31-66;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:49-69.
    pub fn SourceToDestinationProjection(source: &Node<C>, destination: &Node<C>) -> Matrix<C> {
        let mut extra = ExtraProjectionResult::default();
        let mut success = false;
        Self::SourceToDestinationProjectionInternal(source, destination, &mut extra, &mut success)
    }
    pub fn SourceToDestinationProjectionTo(
        context: &C,
        source: &Node<C>,
        destination: &Node<C>,
        projection: &mut Matrix<C>,
    ) -> bool {
        let mut extra = ExtraProjectionResult::default();
        let mut success = false;
        *projection = Self::SourceToDestinationProjectionInternal(
            source,
            destination,
            &mut extra,
            &mut success,
        );
        !context.SingularTransformFixEnabled() || success
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:68-88.
    pub fn SourceToDestinationRect<R: GeometryMapperRect<Matrix<C>>>(
        source: &Node<C>,
        destination: &Node<C>,
        rect: &mut R,
    ) {
        *rect = rect.Mapped(&Self::SourceToDestinationProjection(source, destination));
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:71-195;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:211-215.
    fn SourceToDestinationProjectionInternal(
        source: &Node<C>,
        destination: &Node<C>,
        extra: &mut ExtraProjectionResult,
        success: &mut bool,
    ) -> Matrix<C> {
        *success = true;
        if source == destination {
            return Matrix::<C>::default();
        }
        if source.Parent().as_ref() == Some(destination) {
            extra.has_sticky_or_anchor_position = source.RequiresCompositingForStickyPosition()
                || source.RequiresCompositingForAnchorPosition();
            if source.IsIdentityOr2dTranslation() && source.OriginIsOrigin() {
                extra.has_animation = source.HasActiveTransformAnimation();
                return source.Matrix();
            }
        }
        if destination.IsIdentityOr2dTranslation()
            && destination.Parent().as_ref() == Some(source)
            && !destination.HasActiveTransformAnimation()
        {
            let v = destination.Get2dTranslation();
            let mut m = Matrix::<C>::default();
            m.Translate(-v.x(), -v.y());
            return m;
        }
        // Update each node before taking either cache's scoped read. An ancestor
        // may be the other node; holding its mutex during a descendant update would
        // recursively borrow the same native allocation.
        WithTransformCache(source, |_| ());
        WithTransformCache(destination, |_| ());
        let coplanar = WithTransformCache(source, |sc| {
            WithTransformCache(destination, |dc| {
                extra.has_sticky_or_anchor_position |= sc.has_sticky_or_anchor_position();
                if sc.root_of_2d_translation() == dc.root_of_2d_translation() {
                    let v = *sc.to_2d_translation_root() - *dc.to_2d_translation_root();
                    let mut m = Matrix::<C>::default();
                    m.Translate(v.x(), v.y());
                    return Some(m);
                }
                if sc.plane_root() == dc.plane_root() {
                    extra.has_animation =
                        sc.has_animation_to_plane_root() || dc.has_animation_to_plane_root();
                    if *source == dc.plane_root() {
                        return Some(dc.from_plane_root().clone());
                    }
                    if *destination == sc.plane_root() {
                        return Some(sc.to_plane_root().clone());
                    }
                    let mut m = Matrix::<C>::default();
                    dc.ApplyFromPlaneRoot(&mut m);
                    sc.ApplyToPlaneRoot(&mut m);
                    return Some(m);
                }
                None
            })
        });
        if let Some(m) = coplanar {
            return m;
        }
        WithTransformCache(source, |sc| sc.UpdateScreenTransform(source));
        WithTransformCache(destination, |dc| dc.UpdateScreenTransform(destination));
        WithTransformCache(source, |sc| {
            WithTransformCache(destination, |dc| {
                extra.has_animation = sc.has_animation_to_screen() || dc.has_animation_to_screen();
                if !dc.projection_from_screen_is_valid() {
                    *success = false;
                    return Matrix::<C>::default();
                }
                if source.IsRoot() {
                    return dc.projection_from_screen().clone();
                }
                let mut m = Matrix::<C>::default();
                dc.ApplyProjectionFromScreen(&mut m);
                sc.ApplyToScreen(&mut m);
                m.Flatten();
                m
            })
        })
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:197-213;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:89-92.
    pub fn SourceToDestinationApproximateMinimumScale(
        source: &Node<C>,
        destination: &Node<C>,
    ) -> f32 {
        if source == destination {
            return 1.;
        }
        WithTransformCache(source, |_| ());
        WithTransformCache(destination, |_| ());
        if WithTransformCache(source, |sc| {
            WithTransformCache(destination, |dc| {
                sc.root_of_2d_translation() == dc.root_of_2d_translation()
            })
        }) {
            return 1.;
        }
        let mut rect = RectF::new(0., 0., 1., 1.);
        Self::SourceToDestinationRect(source, destination, &mut rect);
        if rect.height() < rect.width() {
            rect.height()
        } else {
            rect.width()
        }
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:215-232;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:119-178.
    pub fn LocalToAncestorVisualRect(
        context: &C,
        local: &C::State,
        ancestor: &C::State,
        rect: &mut FloatClipRect,
        behavior: OverlayScrollbarClipBehavior,
        flags: VisualRectFlags,
    ) -> bool {
        Self::LocalToAncestorVisualRectInternal::<false>(
            context, local, ancestor, rect, behavior, flags,
        )
    }
    pub fn LocalToLocalRootViewportRect(
        context: &C,
        local: &C::State,
        rect: &mut FloatClipRect,
        behavior: OverlayScrollbarClipBehavior,
        flags: VisualRectFlags,
    ) -> bool {
        Self::LocalToAncestorVisualRect(context, local, &context.RootState(), rect, behavior, flags)
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:234-311;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:227-235.
    fn LocalToAncestorVisualRectInternal<const OVERLAP: bool>(
        context: &C,
        local: &C::State,
        ancestor: &C::State,
        rect: &mut FloatClipRect,
        behavior: OverlayScrollbarClipBehavior,
        flags: VisualRectFlags,
    ) -> bool {
        if local.Effect() != ancestor.Effect() {
            rect.ClearIsTight();
        }
        if local.Transform() == ancestor.Transform() && same_clip(local.Clip(), ancestor.Clip()) {
            return true;
        }
        if !flags.Has(VisualRectFlag::kIgnoreFilters)
            && !same_clip(local.Clip(), ancestor.Clip())
            && !same_optional_clip(
                &nearest_filter(local.Clip()),
                &nearest_filter(ancestor.Clip()),
            )
        {
            return Self::SlowLocalToAncestorVisualRectWithPixelMovingFilters::<OVERLAP>(
                context, local, ancestor, rect, behavior, flags,
            );
        }
        let mut extra = ExtraProjectionResult::default();
        let mut success = false;
        let projection = Self::SourceToDestinationProjectionInternal(
            local.Transform(),
            ancestor.Transform(),
            &mut extra,
            &mut success,
        );
        if !success {
            *rect = FloatClipRect::FromRect(RectF::default());
            return false;
        }
        if OVERLAP && (extra.has_animation || extra.has_sticky_or_anchor_position) {
            *rect = InfiniteLooseFloatClipRect();
        } else {
            rect.Map(&projection);
        }
        if flags.Has(VisualRectFlag::kSkipAncestorAndViewportClips) {
            return true;
        }
        let clip = Self::LocalToAncestorClipRectInternal::<OVERLAP>(
            context,
            local.Clip(),
            ancestor.Clip(),
            ancestor.Transform(),
            behavior,
            flags,
        );
        if flags.Has(VisualRectFlag::kEdgeInclusive) {
            return rect.InclusiveIntersect(&clip);
        }
        rect.Intersect(&clip);
        !rect.Rect().IsEmpty()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:313-369;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:237-243.
    fn SlowLocalToAncestorVisualRectWithPixelMovingFilters<const OVERLAP: bool>(
        context: &C,
        local: &C::State,
        ancestor: &C::State,
        rect: &mut FloatClipRect,
        behavior: OverlayScrollbarClipBehavior,
        flags: VisualRectFlags,
    ) -> bool {
        debug_assert!(!flags.Has(VisualRectFlag::kIgnoreFilters));
        let mut last = local.clone();
        last.SetEffect(ancestor.Effect().clone());
        let ancestor_filter = nearest_filter(ancestor.Clip());
        let mut filter_clip = nearest_filter(local.Clip());
        while !same_optional_clip(&filter_clip, &ancestor_filter) {
            let Some(clip) = filter_clip else {
                *rect = InfiniteLooseFloatClipRect();
                return true;
            };
            let new_state = context.MakeState(
                context.ClipLocalTransform(&clip),
                clip.clone(),
                last.Effect().clone(),
            );
            let filter = context.PixelMovingFilter(&clip);
            debug_assert!(context.FilterLocalTransform(&filter) == *new_state.Transform());
            if OVERLAP && context.HasActiveFilterAnimation(&filter) {
                *rect = InfiniteLooseFloatClipRect();
            } else {
                if !Self::LocalToAncestorVisualRectInternal::<OVERLAP>(
                    context, &last, &new_state, rect, behavior, flags,
                ) {
                    *rect = FloatClipRect::FromRect(RectF::default());
                    return false;
                }
                if !rect.IsInfinite() {
                    let mapped = context.FilterMapRect(&filter, ToEnclosingRect(*rect.Rect()));
                    *rect.RectMut() = RectF::new(
                        mapped.x as f32,
                        mapped.y as f32,
                        mapped.width as f32,
                        mapped.height as f32,
                    );
                }
            }
            last = new_state;
            let next = clip.parent.as_ref().expect("source filter clip parent");
            last.SetClip(next.clone());
            filter_clip = nearest_filter(next);
        }
        Self::LocalToAncestorVisualRectInternal::<OVERLAP>(
            context, &last, ancestor, rect, behavior, flags,
        )
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:371-409;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:93-117.
    pub fn LocalToAncestorClipRect(
        context: &C,
        local: &C::State,
        ancestor: &C::State,
        behavior: OverlayScrollbarClipBehavior,
    ) -> FloatClipRect {
        if same_clip(local.Clip(), ancestor.Clip()) {
            return FloatClipRect::default();
        }
        let mut result = Self::LocalToAncestorClipRectInternal::<false>(
            context,
            local.Clip(),
            ancestor.Clip(),
            ancestor.Transform(),
            behavior,
            VisualRectFlags::default(),
        );
        if local.Effect() != ancestor.Effect() {
            result.ClearIsTight();
        }
        result
    }
    fn GetClipRect(
        context: &C,
        clip: &Arc<ClipPaintPropertyNode>,
        behavior: OverlayScrollbarClipBehavior,
        precise: bool,
    ) -> FloatClipRect {
        let mut rect = if behavior
            == OverlayScrollbarClipBehavior::kExcludeOverlayScrollbarSizeForHitTesting
        {
            context.LayoutClipRectExcludingOverlayScrollbars(clip)
        } else if precise {
            context.PreciseLayoutClipRect(clip)
        } else {
            context.LayoutClipRect(clip)
        };
        if context.ClipHasPath(clip) {
            rect.ClearIsTight();
        }
        rect
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:411-519;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:219-225.
    fn LocalToAncestorClipRectInternal<const OVERLAP: bool>(
        context: &C,
        descendant: &Arc<ClipPaintPropertyNode>,
        ancestor: &Arc<ClipPaintPropertyNode>,
        ancestor_transform: &Node<C>,
        behavior: OverlayScrollbarClipBehavior,
        flags: VisualRectFlags,
    ) -> FloatClipRect {
        if same_clip(descendant, ancestor) {
            return FloatClipRect::default();
        }
        if descendant
            .parent
            .as_ref()
            .is_some_and(|p| same_clip(p, ancestor))
            && context.ClipLocalTransform(descendant) == *ancestor_transform
        {
            return Self::GetClipRect(
                context,
                descendant,
                behavior,
                flags.Has(VisualRectFlag::kUsePreciseClipPath),
            );
        }
        let mut clip = FloatClipRect::default();
        let mut clip_node = Some(descendant.clone());
        let mut intermediate = Vec::with_capacity(16);
        let key = ClipAndTransform::new(ancestor, ancestor_transform.RawNode(), behavior);
        while let Some(node) = clip_node.as_ref() {
            if same_clip(node, ancestor) {
                break;
            }
            // Source stores the cache entry by value when memoizing; copy its scalar
            // clip result only, never a node or property-tree snapshot.
            let mut cached = None;
            if !flags.Has(VisualRectFlag::kEdgeInclusive)
                && !(flags.Has(VisualRectFlag::kUsePreciseClipPath)
                    && context.IsForCompositeClipPathAnimation(node))
            {
                cached = WithClipCache(node, |cache| cache.GetCachedClip(&key).cloned());
            }
            if OVERLAP
                && cached
                    .as_ref()
                    .is_some_and(|c| c.has_transform_animation || c.has_sticky_transform)
            {
                cached = None;
            }
            if let Some(cached) = cached {
                clip = cached.clip_rect;
                break;
            }
            intermediate.push(node.clone());
            clip_node = node.parent.clone();
        }
        if clip_node.is_none() {
            return InfiniteLooseFloatClipRect();
        }
        for node in intermediate.iter().rev() {
            let mut extra = ExtraProjectionResult::default();
            let mut success = false;
            let projection = Self::SourceToDestinationProjectionInternal(
                &context.ClipLocalTransform(node),
                ancestor_transform,
                &mut extra,
                &mut success,
            );
            if !success {
                return FloatClipRect::FromRect(RectF::default());
            }
            if OVERLAP && (extra.has_animation || extra.has_sticky_or_anchor_position) {
                continue;
            }
            let mut mapped = Self::GetClipRect(
                context,
                node,
                behavior,
                flags.Has(VisualRectFlag::kUsePreciseClipPath),
            );
            mapped.Map(&projection);
            if flags.Has(VisualRectFlag::kEdgeInclusive) {
                clip.InclusiveIntersect(&mapped);
            } else {
                clip.Intersect(&mapped);
                if !(flags.Has(VisualRectFlag::kUsePreciseClipPath)
                    && context.IsForCompositeClipPathAnimation(node))
                {
                    WithClipCache(node, |cache| {
                        cache.SetCachedClip(&ClipCacheEntry {
                            clip_and_transform: key.clone(),
                            clip_rect: clip,
                            has_transform_animation: extra.has_animation,
                            has_sticky_transform: extra.has_sticky_or_anchor_position,
                        })
                    });
                }
            }
        }
        debug_assert!(
            flags.Has(VisualRectFlag::kEdgeInclusive)
                || (flags.Has(VisualRectFlag::kUsePreciseClipPath)
                    && context.IsForCompositeClipPathAnimation(descendant))
                || OVERLAP
                || WithClipCache(descendant, |cache| cache
                    .GetCachedClip(&key)
                    .expect("source cached descendant clip")
                    .clip_rect
                    == clip)
        );
        clip
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:521-605;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:180-183.
    pub fn MightOverlapForCompositing(
        context: &C,
        rect1: &RectF,
        state1: &C::State,
        rect2: &RectF,
        state2: &C::State,
    ) -> bool {
        if state1.Transform() == state2.Transform() && same_clip(state1.Clip(), state2.Clip()) {
            return rect1.Intersects(*rect2);
        }
        let root = context.RootState();
        let common = context.MakeState(
            state1.Transform().LowestCommonAncestor(state2.Transform()),
            context.ClipLowestCommonAncestor(state1.Clip(), state2.Clip()),
            root.Effect().clone(),
        );
        let scroll1 = WithTransformCache(state1.Transform(), |c| c.nearest_scroll_translation());
        let scroll2 = WithTransformCache(state2.Transform(), |c| c.nearest_scroll_translation());
        let mut new_state1 = state1.clone();
        let mut new_state2 = state2.clone();
        let adjust_for_clips = |scroll: &Node<C>, state: &mut C::State| -> bool {
            let mut clip = state.Clip().clone();
            while !same_clip(&clip, common.Clip()) {
                if WithTransformCache(&context.ClipLocalTransform(&clip), |c| {
                    c.nearest_scroll_translation()
                }) != *scroll
                {
                    if !same_optional_clip(&nearest_filter(state.Clip()), &nearest_filter(&clip)) {
                        return true;
                    }
                    state.SetClip(clip.parent.clone().expect("source ancestor clip"));
                    return false;
                }
                clip = clip.parent.clone().expect("source ancestor clip");
            }
            false
        };
        if adjust_for_clips(&scroll1, &mut new_state1)
            || adjust_for_clips(&scroll2, &mut new_state2)
        {
            return true;
        }
        if scroll1 == scroll2 {
            return Self::MightOverlapForCompositingInternal(
                context, &common, rect1, state1, rect2, state2,
            );
        }
        let mut new_rect1 = *rect1;
        let mut new_rect2 = *rect2;
        let scroll_lca = WithTransformCache(common.Transform(), |c| c.nearest_scroll_translation());
        let adjust = |mut scroll: Node<C>, rect: &mut RectF, state: &mut C::State| {
            while scroll != scroll_lca {
                Self::MapVisualRectAboveScrollForCompositingOverlap(context, &scroll, rect, state);
                scroll = scroll.ParentScrollTranslationNode();
            }
        };
        adjust(scroll1, &mut new_rect1, &mut new_state1);
        adjust(scroll2, &mut new_rect2, &mut new_state2);
        Self::MightOverlapForCompositingInternal(
            context,
            &common,
            &new_rect1,
            &new_state1,
            &new_rect2,
            &new_state2,
        )
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:607-630;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:245-255.
    fn MightOverlapForCompositingInternal(
        context: &C,
        common: &C::State,
        rect1: &RectF,
        state1: &C::State,
        rect2: &RectF,
        state2: &C::State,
    ) -> bool {
        let v1 = Self::VisualRectForCompositingOverlap(context, rect1, state1, common);
        let v2 = Self::VisualRectForCompositingOverlap(context, rect2, state2, common);
        v1.Intersects(v2)
    }
    fn VisualRectForCompositingOverlap(
        context: &C,
        rect: &RectF,
        local: &C::State,
        ancestor: &C::State,
    ) -> RectF {
        let mut visual = FloatClipRect::FromRect(*rect);
        Self::LocalToAncestorVisualRectInternal::<true>(
            context,
            local,
            ancestor,
            &mut visual,
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            VisualRectFlags::default(),
        );
        if let Some(limit) = Self::VisibilityLimit(context, ancestor) {
            visual.RectMut().Intersect(limit);
        }
        *visual.Rect()
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:632-658;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:257-260.
    fn MapVisualRectAboveScrollForCompositingOverlap(
        context: &C,
        scroll: &Node<C>,
        rect: &mut RectF,
        state: &mut C::State,
    ) {
        debug_assert!(
            WithTransformCache(state.Transform(), |c| c.nearest_scroll_translation()) == *scroll
        );
        debug_assert!(scroll.HasScrollNode());
        *rect = Self::VisualRectForCompositingOverlap(
            context,
            rect,
            state,
            &Self::ScrollingContentsState(context, scroll),
        );
        let content = scroll.ScrollContentsRect();
        let container = scroll.ScrollContainerRect();
        // Source gfx::Size subtraction clamps to non-negative values before SizeF.
        let width = (content.width - container.width).max(0) as f32;
        let height = (content.height - container.height).max(0) as f32;
        rect.Offset(-width, -height);
        rect.set_width(rect.width() + width);
        rect.set_height(rect.height() + height);
        rect.Intersect(RectF::new(
            container.x as f32,
            container.y as f32,
            container.width as f32,
            container.height as f32,
        ));
        *state = Self::ScrollContainerState(context, scroll);
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:660-675;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:261-275.
    pub fn LocalToAncestorVisualRectInternalForTesting(
        context: &C,
        local: &C::State,
        ancestor: &C::State,
        rect: &mut FloatClipRect,
    ) -> bool {
        Self::LocalToAncestorVisualRectInternal::<false>(
            context,
            local,
            ancestor,
            rect,
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            VisualRectFlags::default(),
        )
    }
    pub fn LocalToAncestorVisualRectInternalForCompositingOverlapForTesting(
        context: &C,
        local: &C::State,
        ancestor: &C::State,
        rect: &mut FloatClipRect,
    ) -> bool {
        Self::LocalToAncestorVisualRectInternal::<true>(
            context,
            local,
            ancestor,
            rect,
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            VisualRectFlags::default(),
        )
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:677-690;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:184-201.
    pub fn VisibilityLimit(context: &C, state: &C::State) -> Option<RectF> {
        if context.SelfOrAncestorParticipatesInViewTransition(state.Effect()) {
            return None;
        }
        if context.ClipLocalTransform(state.Clip()) == *state.Transform() {
            return Some(context.PaintClipRect(state.Clip()));
        }
        if state.Transform().HasScrollNode() {
            let rect = state.Transform().ScrollContentsRect();
            return Some(RectF::new(
                rect.x as f32,
                rect.y as f32,
                rect.width as f32,
                rect.height as f32,
            ));
        }
        None
    }
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:692-697;
    // Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:202.
    pub fn ClearCache() {
        GeometryMapperTransformCache::<Node<C>>::ClearCache();
        GeometryMapperClipCache::ClearCache();
    }
}

// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:67,118,179,210,226,236,244,256;
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:17,47-48,70,196,214,233,312,370,410,520,606,631,659,676,691.
// Declaration/definition separators have no runtime instructions.

// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:216-218.
// ForCompositingOverlap::{kNo,kYes} is the const bool specialization domain
// (false,true); Rust stable const generics cannot use an enum ADT parameter.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:118-118.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:179-179.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:210-210.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:226-226.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:236-236.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:244-244.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.h:256-256.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:47-48.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:70-70.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:196-196.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:214-214.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:233-233.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:312-312.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:370-370.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:410-410.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:520-520.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:606-606.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:631-631.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:659-659.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:676-676.
// Source: third_party/blink/renderer/platform/graphics/paint/geometry_mapper.cc:691-691.
// Namespace closure, documentation and declaration separators map to module scope.

//! Direct Blink property-state upcast and transform merge decisions.
//!
//! Source: third_party/blink/renderer/platform/graphics/paint/
//! property_tree_state.cc:17-196;
//! Source: third_party/blink/renderer/platform/graphics/paint/
//! transform_paint_property_node.cc:166-218.
//! The implementation walks the resident Paint property nodes and their
//! canonical GeometryMapper cache. It does not build a parallel ancestry tree.

use super::{ClipNode, EffectNode, PropertyTreeState, TransformNode};
use crate::{
    adapter::{
        geometry::{Rect, RectF, ToEnclosingRect},
        input_types::ElementId,
        pending_layer::{
            PendingEffectNode, PendingPropertyTreeState, PendingTransformNode, PendingUpcastResult,
        },
        property_types::BlendMode,
    },
    cc::trees::sticky_position_constraint::CanMergeResult,
    runtime::types::CpuSkiaBackend,
    third_party::blink::renderer::platform::graphics::paint::geometry_mapper_transform_cache::WithTransformCache,
};
use paint::paint_property_tree as native;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransformCompositingBoundaryType {
    NotSameBoundary,
    SameBoundary,
    ThroughMergeableComposited,
    ThroughMergeableCompositedScrollRangeDependent,
}

fn nearest_directly_composited_ancestor<R: CpuSkiaBackend>(
    node: &TransformNode<R>,
) -> Option<TransformNode<R>> {
    WithTransformCache(node, |cache| cache.nearest_directly_composited_ancestor())
}

fn nearest_composited_scroll_translation<R: CpuSkiaBackend>(
    scroll_translation: &TransformNode<R>,
    is_composited_scroll: &mut dyn FnMut(&TransformNode<R>) -> bool,
) -> Option<TransformNode<R>> {
    let mut current = scroll_translation.clone();
    loop {
        if is_composited_scroll(&current) {
            return Some(current);
        }
        let parent = current.node().parent.as_ref()?.clone();
        let parent = TransformNode::from_arc(parent);
        current = WithTransformCache(&parent, |cache| cache.nearest_scroll_translation());
    }
}

fn requires_compositing_for_fixed_position_only<R>(node: &TransformNode<R>) -> bool {
    let reasons = &node.node().direct_compositing_reasons;
    reasons.contains(&"FixedPosition")
        && reasons.iter().all(|reason| {
            matches!(
                *reason,
                "FixedPosition"
                    | "UndoOverscroll"
                    | "AffectedByOuterViewportBoundsDelta"
                    | "AffectedBySafeAreaBottom"
            )
        })
}

fn requires_compositing_for_sticky_position_only<R>(node: &TransformNode<R>) -> bool {
    node.node().direct_compositing_reasons.as_slice() == ["StickyPosition"]
}

fn can_merge_for_fixed_position<R: CpuSkiaBackend>(
    first: &TransformNode<R>,
    second: &TransformNode<R>,
) -> bool {
    first.node().direct_compositing_reasons == second.node().direct_compositing_reasons
        && requires_compositing_for_fixed_position_only(first)
        && requires_compositing_for_fixed_position_only(second)
        && first.ScrollParentScrollTranslation() == second.ScrollParentScrollTranslation()
        && first.UnaliasedParent() == second.UnaliasedParent()
}

fn source_rect(value: paint::paint_engine::PaintRect) -> Rect {
    ToEnclosingRect(RectF::new(
        value.x as f32,
        value.y as f32,
        value.width as f32,
        value.height as f32,
    ))
}

fn can_merge_for_sticky_position<R: CpuSkiaBackend>(
    first: &TransformNode<R>,
    second: &TransformNode<R>,
) -> CanMergeResult {
    if !requires_compositing_for_sticky_position_only(first)
        || !requires_compositing_for_sticky_position_only(second)
    {
        return CanMergeResult::kCannotMerge;
    }
    let Some(first_parent) = first.UnaliasedParent() else {
        return CanMergeResult::kCannotMerge;
    };
    let Some(second_parent) = second.UnaliasedParent() else {
        return CanMergeResult::kCannotMerge;
    };
    if nearest_directly_composited_ancestor(&first_parent)
        != nearest_directly_composited_ancestor(&second_parent)
        || first.PropertyTreeNearestScrollTranslationNode()
            != second.PropertyTreeNearestScrollTranslationNode()
    {
        return CanMergeResult::kCannotMerge;
    }

    let first_constraint = first.GetStickyConstraint();
    let second_constraint = second.GetStickyConstraint();
    let (Some(first_constraint), Some(second_constraint)) = (first_constraint, second_constraint)
    else {
        return if first_constraint.is_none() && second_constraint.is_none() {
            CanMergeResult::kCanAlwaysMerge
        } else {
            CanMergeResult::kCannotMerge
        };
    };

    let scroll_translation = first.PropertyTreeNearestScrollTranslationNode();
    let scroll_node = scroll_translation
        .node()
        .scroll
        .as_ref()
        .expect("source sticky nearest scroll translation");
    let scroll_element = scroll_translation.ScrollCompositorElementId();
    let can_use_scroll_range = |first: ElementId, second: ElementId| {
        first == second && (!first.IsValid() || first == scroll_element)
    };
    let scroll_range = if can_use_scroll_range(
        first_constraint.x_scroll_ancestor_element_id,
        second_constraint.x_scroll_ancestor_element_id,
    ) && can_use_scroll_range(
        first_constraint.y_scroll_ancestor_element_id,
        second_constraint.y_scroll_ancestor_element_id,
    ) {
        let contents = source_rect(scroll_node.contents_rect);
        let container = source_rect(scroll_node.container_rect);
        let mut range = source_rect(scroll_node.compositor.scrolling_contents_cull_rect);
        range.Intersect(contents);
        range = Rect::new(
            range.x.saturating_sub(contents.x),
            range.y.saturating_sub(contents.y),
            range.width.saturating_sub(container.width),
            range.height.saturating_sub(container.height),
        );
        Some(RectF::new(
            range.x as f32,
            range.y as f32,
            range.width as f32,
            range.height as f32,
        ))
    } else {
        None
    };
    first_constraint.CanMerge(&second_constraint, scroll_range)
}

fn in_same_transform_compositing_boundary<R: CpuSkiaBackend>(
    first: &TransformNode<R>,
    second: &TransformNode<R>,
    merge_fixed_layers: bool,
    merge_sticky_layers: bool,
    is_composited_scroll: &mut dyn FnMut(&TransformNode<R>) -> bool,
) -> TransformCompositingBoundaryType {
    let first_composited = nearest_directly_composited_ancestor(first);
    let second_composited = nearest_directly_composited_ancestor(second);
    if first_composited != second_composited {
        if let (Some(first_composited), Some(second_composited)) =
            (&first_composited, &second_composited)
        {
            if merge_fixed_layers
                && can_merge_for_fixed_position(first_composited, second_composited)
            {
                return TransformCompositingBoundaryType::ThroughMergeableComposited;
            }
            if merge_sticky_layers {
                return match can_merge_for_sticky_position(first_composited, second_composited) {
                    CanMergeResult::kCannotMerge => {
                        TransformCompositingBoundaryType::NotSameBoundary
                    }
                    CanMergeResult::kCanAlwaysMerge => {
                        TransformCompositingBoundaryType::ThroughMergeableComposited
                    }
                    CanMergeResult::kCanMergeWithinScrollRange => {
                        TransformCompositingBoundaryType::ThroughMergeableCompositedScrollRangeDependent
                    }
                };
            }
        }
        return TransformCompositingBoundaryType::NotSameBoundary;
    }

    let first_scroll = first.PropertyTreeNearestScrollTranslationNode();
    let second_scroll = second.PropertyTreeNearestScrollTranslationNode();
    if first_scroll == second_scroll
        || nearest_composited_scroll_translation(&first_scroll, is_composited_scroll)
            == nearest_composited_scroll_translation(&second_scroll, is_composited_scroll)
    {
        TransformCompositingBoundaryType::SameBoundary
    } else {
        TransformCompositingBoundaryType::NotSameBoundary
    }
}

fn clip_chain_in_transform_compositing_boundary<R: CpuSkiaBackend>(
    node: &ClipNode<R>,
    ancestor: &ClipNode<R>,
    transform: &TransformNode<R>,
    merge_fixed_layers: bool,
    merge_sticky_layers: bool,
    is_composited_scroll: &mut dyn FnMut(&TransformNode<R>) -> bool,
) -> bool {
    let mut current = node.clone();
    while current != *ancestor {
        let local = TransformNode::from_arc(current.node().local_transform_space.clone());
        if in_same_transform_compositing_boundary(
            transform,
            &local,
            merge_fixed_layers,
            merge_sticky_layers,
            is_composited_scroll,
        ) != TransformCompositingBoundaryType::SameBoundary
        {
            return false;
        }
        let Some(parent) = current.node().parent.as_ref() else {
            return false;
        };
        current = ClipNode::from_arc(parent.clone());
    }
    true
}

impl<R: CpuSkiaBackend> PendingPropertyTreeState for PropertyTreeState<R> {
    type PaintPropertyChange = native::PaintPropertyChangeType;
    type TransformNode = TransformNode<R>;
    type ClipNode = ClipNode<R>;
    type EffectNode = EffectNode<R>;

    fn EffectRequiresCompositingForBackdropFilterMask(&self) -> bool {
        self.effect_
            .node()
            .direct_compositing_reasons
            .contains(&"BackdropFilterMask")
    }
    fn EffectRequiresCompositingForCanvasChild(&self) -> bool {
        self.effect_
            .node()
            .direct_compositing_reasons
            .contains(&"CanvasChild")
    }
    fn ChangedToRoot(&self, change: Self::PaintPropertyChange) -> bool {
        paint::property_node_geometry::PropertyStateChanged(
            &self.native(),
            change,
            &native::PropertyTreeState::default(),
        )
    }
    fn SameEffectNode(&self, other: &Self) -> bool {
        self.effect_ == other.effect_
    }
    fn EffectUnaliasedParentSameAs(&self, ancestor: &Self) -> bool {
        ancestor
            .effect_
            .node()
            .parent
            .as_ref()
            .is_some_and(|parent| parent.lifecycle.same_node(&self.effect_.node().lifecycle))
    }
    fn EffectIsOpacityOnly(&self) -> bool {
        self.EffectBlendMode() == BlendMode::kSrcOver
            && !self.effect_.node().compositor.has_filter_info
            && !self.effect_.node().compositor.has_backdrop_filter_info
    }
    fn EffectOpacity(&self) -> f32 {
        self.effect_.node().opacity
    }
    fn EffectBlendMode(&self) -> BlendMode {
        <EffectNode<R> as PendingEffectNode<TransformNode<R>, usize>>::BlendMode(&self.effect_)
    }
    fn CanUpcastWith(
        &self,
        guest: &Self,
        merge_fixed_layers: bool,
        merge_sticky_layers: bool,
        is_composited_scroll: &mut dyn FnMut(&Self::TransformNode) -> bool,
    ) -> Option<PendingUpcastResult<Self>> {
        debug_assert_eq!(self.effect_, guest.effect_);
        let mut scroll_range_dependent = false;
        let upcast_transform = if self.transform_ == guest.transform_ {
            self.transform_.clone()
        } else {
            let boundary = in_same_transform_compositing_boundary(
                &self.transform_,
                &guest.transform_,
                merge_fixed_layers,
                merge_sticky_layers,
                is_composited_scroll,
            );
            if boundary == TransformCompositingBoundaryType::NotSameBoundary
                || self.transform_.IsBackfaceHidden() != guest.transform_.IsBackfaceHidden()
            {
                return None;
            }
            if boundary == TransformCompositingBoundaryType::SameBoundary {
                self.transform_
                    .PropertyTreeLowestCommonAncestor(&guest.transform_)
            } else {
                scroll_range_dependent = boundary
                    == TransformCompositingBoundaryType::ThroughMergeableCompositedScrollRangeDependent;
                nearest_directly_composited_ancestor(&self.transform_)
                    .expect("source mergeable composited ancestor")
            }
        };

        let upcast_clip = if self.clip_ == guest.clip_ {
            self.clip_.clone()
        } else {
            let clip = ClipNode::from_arc(paint::property_node_geometry::LowestCommonAncestor(
                self.clip_.as_arc(),
                guest.clip_.as_arc(),
            ));
            if !clip_chain_in_transform_compositing_boundary(
                &self.clip_,
                &clip,
                &upcast_transform,
                merge_fixed_layers,
                merge_sticky_layers,
                is_composited_scroll,
            ) || !clip_chain_in_transform_compositing_boundary(
                &guest.clip_,
                &clip,
                &upcast_transform,
                merge_fixed_layers,
                merge_sticky_layers,
                is_composited_scroll,
            ) {
                return None;
            }
            clip
        };

        Some(PendingUpcastResult {
            upcasted_state: Self {
                transform_: upcast_transform,
                clip_: upcast_clip,
                effect_: self.effect_.clone(),
            },
            scroll_range_dependent,
        })
    }
    fn NearestScrollTranslationNode(&self) -> Self::TransformNode {
        self.transform_.PropertyTreeNearestScrollTranslationNode()
    }
    fn SameNearestDirectlyCompositedAncestor(&self, other: &Self) -> bool {
        nearest_directly_composited_ancestor(&self.transform_)
            == nearest_directly_composited_ancestor(&other.transform_)
    }
    fn ToDebugString(&self) -> String {
        format!(
            "t:{:?} c:{:?} e:{:?}",
            self.transform_, self.clip_, self.effect_
        )
    }
    fn TransformNode(&self) -> Self::TransformNode {
        self.transform_.clone()
    }
    fn ClipNode(&self) -> Self::ClipNode {
        self.clip_.clone()
    }
    fn EffectNode(&self) -> Self::EffectNode {
        self.effect_.clone()
    }
    fn SetTransformNode(&mut self, transform: Self::TransformNode) {
        self.transform_ = transform;
    }
}

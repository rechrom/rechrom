//! Paint-effect to compositor property-tree conversion policy.

use compositor_tree::{
    effect_node::{EffectNode, RenderSurfaceReason},
    property_types::{BlendMode, FilterOperations, PropertyPath},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CcEffectType(u8);

impl CcEffectType {
    pub const kEffect: Self = Self(0);
    pub const kSyntheticForNonTrivialClip: Self = Self(1 << 0);
    pub const kSyntheticFor2dAxisAlignment: Self = Self(1 << 1);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for CcEffectType {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for CcEffectType {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alignment {
    #[default]
    kAligned,
    kMisaligned,
    kUnknown,
}

pub trait TransformAlignmentNode: Clone + PartialEq {
    fn HasActiveTransformAnimation(&self) -> bool;
    fn TransformAnimationIsAxisAligned(&self) -> bool;
    fn UnaliasedParent(&self) -> Option<Self>;
    fn LowestCommonAncestor(&self, other: &Self) -> Self;
    fn ProjectionToPreserves2dAxisAlignment(&self, destination: &Self) -> bool;
}

pub fn TransformsToAncestorHaveNonAxisAlignedActiveAnimation<N: TransformAlignmentNode>(
    descendant: &N,
    ancestor: &N,
) -> bool {
    if descendant == ancestor {
        return false;
    }
    let mut node = descendant.clone();
    while &node != ancestor {
        if node.HasActiveTransformAnimation() && !node.TransformAnimationIsAxisAligned() {
            return true;
        }
        node = node
            .UnaliasedParent()
            .expect("ancestor must be on the unaliased transform chain");
    }
    false
}

pub fn TransformsMayBe2dAxisMisaligned<N: TransformAlignmentNode>(a: &N, b: &N) -> bool {
    if a == b {
        return false;
    }
    if !a.ProjectionToPreserves2dAxisAlignment(b) {
        return true;
    }
    let lca = a.LowestCommonAncestor(b);
    TransformsToAncestorHaveNonAxisAlignedActiveAnimation(a, &lca)
        || TransformsToAncestorHaveNonAxisAlignedActiveAnimation(b, &lca)
}

#[derive(Clone, Copy, Debug)]
pub struct EffectSurfaceInputs {
    pub has_filter: bool,
    pub will_change_filter: bool,
    pub active_filter_animation: bool,
    pub has_backdrop_filter: bool,
    pub will_change_backdrop_filter: bool,
    pub active_backdrop_filter_animation: bool,
    pub blend_mode: BlendMode,
    pub has_view_transition_resource: bool,
    pub unbounded_element: bool,
    pub flattens_3d_scene: bool,
    pub has_element_capture: bool,
    pub opacity: f32,
    pub active_opacity_animation: bool,
}

pub trait EffectSurfaceSource {
    fn has_filter(&self) -> bool;
    fn will_change_filter(&self) -> bool;
    fn active_filter_animation(&self) -> bool;
    fn has_backdrop_filter(&self) -> bool;
    fn will_change_backdrop_filter(&self) -> bool;
    fn active_backdrop_filter_animation(&self) -> bool;
    fn blend_mode(&self) -> BlendMode;
    fn has_view_transition_resource(&self) -> bool;
    fn unbounded_element(&self) -> bool;
    fn flattens_3d_scene(&self) -> bool;
    fn has_element_capture(&self) -> bool;
    fn opacity(&self) -> f32;
    fn active_opacity_animation(&self) -> bool;
}

impl EffectSurfaceSource for EffectSurfaceInputs {
    fn has_filter(&self) -> bool {
        self.has_filter
    }
    fn will_change_filter(&self) -> bool {
        self.will_change_filter
    }
    fn active_filter_animation(&self) -> bool {
        self.active_filter_animation
    }
    fn has_backdrop_filter(&self) -> bool {
        self.has_backdrop_filter
    }
    fn will_change_backdrop_filter(&self) -> bool {
        self.will_change_backdrop_filter
    }
    fn active_backdrop_filter_animation(&self) -> bool {
        self.active_backdrop_filter_animation
    }
    fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }
    fn has_view_transition_resource(&self) -> bool {
        self.has_view_transition_resource
    }
    fn unbounded_element(&self) -> bool {
        self.unbounded_element
    }
    fn flattens_3d_scene(&self) -> bool {
        self.flattens_3d_scene
    }
    fn has_element_capture(&self) -> bool {
        self.has_element_capture
    }
    fn opacity(&self) -> f32 {
        self.opacity
    }
    fn active_opacity_animation(&self) -> bool {
        self.active_opacity_animation
    }
}

pub fn IsConditionalRenderSurfaceReason(reason: RenderSurfaceReason) -> bool {
    matches!(
        reason,
        RenderSurfaceReason::kBlendModeDstIn
            | RenderSurfaceReason::kOpacity
            | RenderSurfaceReason::kOpacityAnimation
    )
}

pub fn ConditionalRenderSurfaceReason(inputs: &impl EffectSurfaceSource) -> RenderSurfaceReason {
    if inputs.blend_mode() == BlendMode::kDstIn {
        return RenderSurfaceReason::kBlendModeDstIn;
    }
    if inputs.opacity() != 1.0 {
        return RenderSurfaceReason::kOpacity;
    }
    if inputs.active_opacity_animation() {
        return RenderSurfaceReason::kOpacityAnimation;
    }
    RenderSurfaceReason::kNone
}

/// Returns the first source-ordered reason that requires a render surface.
pub fn RenderSurfaceReasonForEffect(inputs: &impl EffectSurfaceSource) -> RenderSurfaceReason {
    if inputs.has_filter() || inputs.will_change_filter() {
        return RenderSurfaceReason::kFilter;
    }
    if inputs.active_filter_animation() {
        return RenderSurfaceReason::kFilterAnimation;
    }
    if inputs.has_backdrop_filter() || inputs.will_change_backdrop_filter() {
        return RenderSurfaceReason::kBackdropFilter;
    }
    if inputs.active_backdrop_filter_animation() {
        return RenderSurfaceReason::kBackdropFilterAnimation;
    }
    if !matches!(inputs.blend_mode(), BlendMode::kSrcOver | BlendMode::kDstIn) {
        return RenderSurfaceReason::kBlendMode;
    }
    if inputs.has_view_transition_resource() {
        return RenderSurfaceReason::kViewTransitionParticipant;
    }
    if inputs.unbounded_element() {
        return RenderSurfaceReason::kUnboundedElement;
    }
    if inputs.flattens_3d_scene() {
        return RenderSurfaceReason::k3dTransformFlattening;
    }
    if inputs.has_element_capture() {
        return RenderSurfaceReason::kSubtreeIsBeingCaptured;
    }
    ConditionalRenderSurfaceReason(inputs)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConditionalSurfaceStats {
    pub effect_nodes_visited: usize,
    pub scale_surfaces_promoted: usize,
    pub conditional_surfaces_removed: usize,
    pub child_surfaces_propagated: usize,
}

/// Resolves conditional render surfaces bottom-up after layer membership is
/// known. Layer adapters only collect counts/text/video flags; this package
/// owns the promotion/removal and ancestor propagation decisions.
pub fn ResolveConditionalRenderSurfaceReasons<F: FilterOperations, P: PropertyPath>(
    effects: &mut [EffectNode<F, P>],
    first_effect_id: usize,
    effect_layer_counts: &mut [i32],
    has_text: &mut [bool],
    has_video: &[bool],
) -> ConditionalSurfaceStats {
    let mut trace = browser_tracing::span(
        "compositor.layerize",
        "Compositor.Layerize.ResolveRenderSurfaces",
    );
    assert_eq!(effects.len(), effect_layer_counts.len());
    assert_eq!(effects.len(), has_text.len());
    assert_eq!(effects.len(), has_video.len());
    let mut stats = ConditionalSurfaceStats::default();
    let mut has_child_surface = vec![false; effects.len()];
    for id in (first_effect_id..effects.len()).rev() {
        stats.effect_nodes_visited += 1;
        let effect = &mut effects[id];
        if effect.render_surface_reason == RenderSurfaceReason::kNone
            && effect.needs_effect_for_2d_scale_transform
            && effect_layer_counts[id] >= 2
            && !has_text[id]
            && !has_video[id]
        {
            effect.render_surface_reason =
                RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants;
            stats.scale_surfaces_promoted += 1;
        }

        if effect_layer_counts[id] < 2
            && IsConditionalRenderSurfaceReason(effect.render_surface_reason)
            && (effect.render_surface_reason != RenderSurfaceReason::kBlendModeDstIn
                || !(has_child_surface[id] || effect.is_fast_rounded_corner))
        {
            effect.render_surface_reason = RenderSurfaceReason::kNone;
            stats.conditional_surfaces_removed += 1;
        }

        let parent_id = effect.parent_id as usize;
        debug_assert_ne!(-1, effect_layer_counts[parent_id]);
        if effect.HasRenderSurface() {
            effect_layer_counts[parent_id] += 1;
            has_child_surface[parent_id] = true;
            stats.child_surfaces_propagated += 1;
        } else {
            effect_layer_counts[parent_id] += effect_layer_counts[id];
            has_child_surface[parent_id] |= has_child_surface[id];
        }
        has_text[parent_id] |= has_text[id];

        #[cfg(debug_assertions)]
        {
            effect_layer_counts[id] = -1;
        }
    }
    trace.set("effect_nodes", stats.effect_nodes_visited as f64);
    trace.set("scale_promotions", stats.scale_surfaces_promoted as f64);
    trace.set(
        "conditional_removals",
        stats.conditional_surfaces_removed as f64,
    );
    trace.set(
        "child_surfaces_propagated",
        stats.child_surfaces_propagated as f64,
    );
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> EffectSurfaceInputs {
        EffectSurfaceInputs {
            has_filter: false,
            will_change_filter: false,
            active_filter_animation: false,
            has_backdrop_filter: false,
            will_change_backdrop_filter: false,
            active_backdrop_filter_animation: false,
            blend_mode: BlendMode::kSrcOver,
            has_view_transition_resource: false,
            unbounded_element: false,
            flattens_3d_scene: false,
            has_element_capture: false,
            opacity: 1.0,
            active_opacity_animation: false,
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestTransformNode {
        id: usize,
        parent: Option<usize>,
        active_non_axis_aligned_animation: bool,
        projection_preserves_axis_alignment: bool,
    }

    impl TransformAlignmentNode for TestTransformNode {
        fn HasActiveTransformAnimation(&self) -> bool {
            self.active_non_axis_aligned_animation
        }
        fn TransformAnimationIsAxisAligned(&self) -> bool {
            !self.active_non_axis_aligned_animation
        }
        fn UnaliasedParent(&self) -> Option<Self> {
            self.parent.map(|id| Self {
                id,
                parent: if id > 0 { Some(id - 1) } else { None },
                active_non_axis_aligned_animation: false,
                projection_preserves_axis_alignment: true,
            })
        }
        fn LowestCommonAncestor(&self, other: &Self) -> Self {
            let id = self.id.min(other.id);
            Self {
                id,
                parent: if id > 0 { Some(id - 1) } else { None },
                active_non_axis_aligned_animation: false,
                projection_preserves_axis_alignment: true,
            }
        }
        fn ProjectionToPreserves2dAxisAlignment(&self, _: &Self) -> bool {
            self.projection_preserves_axis_alignment
        }
    }

    #[test]
    fn non_axis_aligned_animation_marks_transform_chain_misaligned() {
        let root = TestTransformNode {
            id: 0,
            parent: None,
            active_non_axis_aligned_animation: false,
            projection_preserves_axis_alignment: true,
        };
        let child = TestTransformNode {
            id: 1,
            parent: Some(0),
            active_non_axis_aligned_animation: true,
            projection_preserves_axis_alignment: true,
        };
        assert!(TransformsMayBe2dAxisMisaligned(&child, &root));
    }

    #[test]
    fn filter_reason_wins_over_later_effect_reasons() {
        let mut value = inputs();
        value.has_filter = true;
        value.has_view_transition_resource = true;
        value.opacity = 0.5;
        assert_eq!(
            RenderSurfaceReasonForEffect(&value),
            RenderSurfaceReason::kFilter
        );
    }

    struct FilterOnlySource;
    impl EffectSurfaceSource for FilterOnlySource {
        fn has_filter(&self) -> bool {
            true
        }
        fn will_change_filter(&self) -> bool {
            unreachable!()
        }
        fn active_filter_animation(&self) -> bool {
            unreachable!()
        }
        fn has_backdrop_filter(&self) -> bool {
            unreachable!()
        }
        fn will_change_backdrop_filter(&self) -> bool {
            unreachable!()
        }
        fn active_backdrop_filter_animation(&self) -> bool {
            unreachable!()
        }
        fn blend_mode(&self) -> BlendMode {
            unreachable!()
        }
        fn has_view_transition_resource(&self) -> bool {
            unreachable!()
        }
        fn unbounded_element(&self) -> bool {
            unreachable!()
        }
        fn flattens_3d_scene(&self) -> bool {
            unreachable!()
        }
        fn has_element_capture(&self) -> bool {
            unreachable!()
        }
        fn opacity(&self) -> f32 {
            unreachable!()
        }
        fn active_opacity_animation(&self) -> bool {
            unreachable!()
        }
    }

    #[test]
    fn filter_short_circuits_later_effect_queries() {
        assert_eq!(
            RenderSurfaceReasonForEffect(&FilterOnlySource),
            RenderSurfaceReason::kFilter
        );
    }

    #[test]
    fn dst_in_is_a_conditional_reason() {
        let mut value = inputs();
        value.blend_mode = BlendMode::kDstIn;
        let reason = RenderSurfaceReasonForEffect(&value);
        assert_eq!(reason, RenderSurfaceReason::kBlendModeDstIn);
        assert!(IsConditionalRenderSurfaceReason(reason));
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    struct TestFilters;
    impl FilterOperations for TestFilters {
        fn IsEmpty(&self) -> bool {
            true
        }
        fn ToString(&self) -> String {
            String::new()
        }
    }

    #[test]
    fn resolves_scale_promotion_and_conditional_removal_bottom_up() {
        let mut effects = vec![EffectNode::<TestFilters, ()>::default(); 4];
        for (id, effect) in effects.iter_mut().enumerate() {
            effect.id = id as i32;
            effect.parent_id = id.saturating_sub(1) as i32;
        }
        effects[2].render_surface_reason = RenderSurfaceReason::kOpacity;
        effects[3].needs_effect_for_2d_scale_transform = true;
        let mut counts = vec![0, 0, 1, 2];
        let mut text = vec![false; 4];
        let video = vec![false; 4];
        let stats =
            ResolveConditionalRenderSurfaceReasons(&mut effects, 2, &mut counts, &mut text, &video);
        assert_eq!(
            effects[3].render_surface_reason,
            RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants
        );
        assert_eq!(
            effects[2].render_surface_reason,
            RenderSurfaceReason::kOpacity
        );
        assert_eq!(stats.scale_surfaces_promoted, 1);
        assert_eq!(stats.conditional_surfaces_removed, 0);
        assert_eq!(stats.effect_nodes_visited, 2);
    }
}

//! Recursive Paint effect-group traversal that produces pending layers.

use crate::{layer_search::MergeSearchStats, stats::LayerizeStats};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectDecompositionOutcome {
    pub decomposed: bool,
    pub rejected: bool,
    pub scroll_range_dependent_upcasts: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectCompositedTransformState {
    pub merge_fixed_layers_enabled: bool,
    pub merge_sticky_layers_enabled: bool,
    pub requires_fixed_position_only: bool,
    pub requires_sticky_position_only: bool,
}

pub trait LayerizeWalkContext {
    type Effect: Clone + PartialEq;
    type CanvasChildId: Copy;
    type Transform: Clone;

    fn root_effect(&self) -> Self::Effect;
    fn invalid_canvas_child_id(&self) -> Self::CanvasChildId;
    fn effect_canvas_child_id(&self, effect: &Self::Effect) -> Option<Self::CanvasChildId>;
    fn effect_draws_content(&self, effect: &Self::Effect) -> bool;

    fn chunk_cursor(&self) -> usize;
    fn chunk_count(&self) -> usize;
    fn chunk_effect(&self, index: usize) -> Self::Effect;
    fn update_before_layerizing_chunk(&mut self, index: usize);
    fn push_pending_layer(&mut self, index: usize, canvas_child_id: Self::CanvasChildId);
    fn advance_chunk_cursor(&mut self);

    fn pending_layer_count(&self) -> usize;
    fn last_layer_requires_own_layer(&self) -> bool;
    fn last_layer_effect_matches(&self, effect: &Self::Effect) -> bool;
    fn strict_child_along_path(
        &self,
        ancestor: &Self::Effect,
        node: &Self::Effect,
    ) -> Option<Self::Effect>;
    fn decomposite_effect(
        &mut self,
        parent_effect: &Self::Effect,
        first_layer_in_parent_group: usize,
        effect: &Self::Effect,
        layer_index: usize,
    ) -> EffectDecompositionOutcome;
    fn force_last_layer_draws_content(&mut self);

    fn last_layer_composited_transform(&self) -> Option<Self::Transform>;
    fn direct_composited_transform_state(
        &self,
        transform: &Self::Transform,
    ) -> DirectCompositedTransformState;
    fn insert_directly_composited_transform(&mut self, transform: Self::Transform) -> bool;
    fn search_merge_candidate(&mut self, first_layer_in_group: usize) -> MergeSearchStats;
}

fn LayerizeGroup<C: LayerizeWalkContext>(
    context: &mut C,
    current_group: &C::Effect,
    mut canvas_child_id: C::CanvasChildId,
    force_draws_content: bool,
    stats: &mut LayerizeStats,
) {
    if let Some(group_canvas_child) = context.effect_canvas_child_id(current_group) {
        canvas_child_id = group_canvas_child;
    }
    let first_layer_in_current_group = context.pending_layer_count();
    while context.chunk_cursor() != context.chunk_count() {
        let chunk_index = context.chunk_cursor();
        let chunk_effect = context.chunk_effect(chunk_index);
        if chunk_effect == *current_group {
            context.update_before_layerizing_chunk(chunk_index);
            context.push_pending_layer(chunk_index, canvas_child_id);
            context.advance_chunk_cursor();
            stats.input_chunks += 1;
            if context.last_layer_requires_own_layer() {
                stats.own_layer_chunks += 1;
                continue;
            }
        } else {
            let Some(subgroup) = context.strict_child_along_path(current_group, &chunk_effect)
            else {
                break;
            };
            let first_layer_in_subgroup = context.pending_layer_count();
            let subgroup_force_draws_content =
                force_draws_content || context.effect_draws_content(&subgroup);
            LayerizeGroup(
                context,
                &subgroup,
                canvas_child_id,
                subgroup_force_draws_content,
                stats,
            );
            if first_layer_in_subgroup != context.pending_layer_count().wrapping_sub(1) {
                continue;
            }
            let outcome = context.decomposite_effect(
                current_group,
                first_layer_in_current_group,
                &subgroup,
                first_layer_in_subgroup,
            );
            stats.effect_decomposition_rejections += usize::from(outcome.rejected);
            stats.scroll_range_dependent_upcasts += outcome.scroll_range_dependent_upcasts;
            if !outcome.decomposed {
                continue;
            }
            stats.decomposited_effects += 1;
        }

        debug_assert!(!context.last_layer_requires_own_layer());
        debug_assert!(context.last_layer_effect_matches(current_group));
        if force_draws_content {
            context.force_last_layer_draws_content();
        }
        if let Some(transform) = context.last_layer_composited_transform() {
            let state = context.direct_composited_transform_state(&transform);
            let participates_in_merge = (state.merge_fixed_layers_enabled
                && state.requires_fixed_position_only)
                || (state.merge_sticky_layers_enabled && state.requires_sticky_position_only);
            if !participates_in_merge && context.insert_directly_composited_transform(transform) {
                continue;
            }
        }

        let merge = context.search_merge_candidate(first_layer_in_current_group);
        stats.merge_attempts += merge.attempts;
        stats.merged_layers += usize::from(merge.merged);
        stats.overlap_barriers += usize::from(merge.overlap_barrier);
        stats.max_merge_search_depth = stats.max_merge_search_depth.max(merge.max_search_depth);
    }
}

pub fn Layerize<C: LayerizeWalkContext>(context: &mut C) -> LayerizeStats {
    let mut trace = browser_tracing::span(
        "compositor.layerize",
        "Compositor.Layerize.TraverseEffectGroups",
    );
    let mut stats = LayerizeStats::default();
    let root = context.root_effect();
    let invalid_canvas = context.invalid_canvas_child_id();
    LayerizeGroup(context, &root, invalid_canvas, false, &mut stats);
    debug_assert_eq!(context.chunk_cursor(), context.chunk_count());
    stats.output_layers = context.pending_layer_count();
    trace.set("input_chunks", stats.input_chunks as f64);
    trace.set("output_layers", stats.output_layers as f64);
    trace.set("own_layer_chunks", stats.own_layer_chunks as f64);
    trace.set("decomposited_effects", stats.decomposited_effects as f64);
    trace.set(
        "effect_decomposition_rejections",
        stats.effect_decomposition_rejections as f64,
    );
    trace.set(
        "scroll_range_dependent_upcasts",
        stats.scroll_range_dependent_upcasts as f64,
    );
    trace.set("merge_attempts", stats.merge_attempts as f64);
    trace.set("merged_layers", stats.merged_layers as f64);
    trace.set("overlap_barriers", stats.overlap_barriers as f64);
    trace.set(
        "max_merge_search_depth",
        stats.max_merge_search_depth as f64,
    );
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Context {
        chunks: Vec<i32>,
        cursor: usize,
        pending_effects: Vec<i32>,
        own_layer_by_chunk: Vec<bool>,
        forced: Vec<bool>,
    }

    impl LayerizeWalkContext for Context {
        type Effect = i32;
        type CanvasChildId = i32;
        type Transform = i32;

        fn root_effect(&self) -> Self::Effect {
            0
        }
        fn invalid_canvas_child_id(&self) -> Self::CanvasChildId {
            -1
        }
        fn effect_canvas_child_id(&self, _effect: &Self::Effect) -> Option<Self::CanvasChildId> {
            None
        }
        fn effect_draws_content(&self, _effect: &Self::Effect) -> bool {
            false
        }
        fn chunk_cursor(&self) -> usize {
            self.cursor
        }
        fn chunk_count(&self) -> usize {
            self.chunks.len()
        }
        fn chunk_effect(&self, index: usize) -> Self::Effect {
            self.chunks[index]
        }
        fn update_before_layerizing_chunk(&mut self, _index: usize) {}
        fn push_pending_layer(&mut self, index: usize, _canvas_child_id: Self::CanvasChildId) {
            self.pending_effects.push(self.chunks[index]);
            self.forced.push(false);
        }
        fn advance_chunk_cursor(&mut self) {
            self.cursor += 1;
        }
        fn pending_layer_count(&self) -> usize {
            self.pending_effects.len()
        }
        fn last_layer_requires_own_layer(&self) -> bool {
            self.own_layer_by_chunk[self.cursor - 1]
        }
        fn last_layer_effect_matches(&self, effect: &Self::Effect) -> bool {
            self.pending_effects.last() == Some(effect)
        }
        fn strict_child_along_path(
            &self,
            _ancestor: &Self::Effect,
            _node: &Self::Effect,
        ) -> Option<Self::Effect> {
            None
        }
        fn decomposite_effect(
            &mut self,
            _parent_effect: &Self::Effect,
            _first_layer_in_parent_group: usize,
            _effect: &Self::Effect,
            _layer_index: usize,
        ) -> EffectDecompositionOutcome {
            unreachable!()
        }
        fn force_last_layer_draws_content(&mut self) {
            *self.forced.last_mut().unwrap() = true;
        }
        fn last_layer_composited_transform(&self) -> Option<Self::Transform> {
            None
        }
        fn direct_composited_transform_state(
            &self,
            _transform: &Self::Transform,
        ) -> DirectCompositedTransformState {
            unreachable!()
        }
        fn insert_directly_composited_transform(&mut self, _transform: Self::Transform) -> bool {
            unreachable!()
        }
        fn search_merge_candidate(&mut self, _first_layer_in_group: usize) -> MergeSearchStats {
            if self.pending_effects.len() <= 1 {
                return MergeSearchStats::default();
            }
            self.pending_effects.pop();
            self.forced.pop();
            MergeSearchStats {
                attempts: 1,
                merged: true,
                overlap_barrier: false,
                max_search_depth: 1,
            }
        }
    }

    #[test]
    fn traversal_owns_chunk_counts_own_layers_and_merge_stats() {
        let mut context = Context {
            chunks: vec![0, 0, 0],
            cursor: 0,
            pending_effects: Vec::new(),
            own_layer_by_chunk: vec![false, true, false],
            forced: Vec::new(),
        };
        let stats = Layerize(&mut context);
        assert_eq!(context.cursor, 3);
        assert_eq!(context.pending_effects.len(), 2);
        assert_eq!(stats.input_chunks, 3);
        assert_eq!(stats.own_layer_chunks, 1);
        assert_eq!(stats.output_layers, 2);
        assert_eq!(stats.merge_attempts, 1);
        assert_eq!(stats.merged_layers, 1);
        assert_eq!(stats.max_merge_search_depth, 1);
    }
}

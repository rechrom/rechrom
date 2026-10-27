//! Compare inherited builder context before persistence/canonicalization.
//! Blink reuses native property-node slots; this builder instead creates raw
//! snapshots. Native owner + role identifies that same slot, and every typed
//! value/link must still match before a clean child can reuse its context.
//! Sources: pre_paint_tree_walk.cc:398-447 and
//! paint_property_tree_builder.cc:4731-4770. This comparison itself neither
//! mutates node identities nor produces property invalidation.
use layoutng_assembly::internal::paint_input::PaintCornerRadii;
use paint_model::{
    paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyKey,
        PaintPropertyNodeLifecycle, PropertyTreeState, ScrollPaintPropertyNode,
        TransformPaintPropertyNode,
    },
    PaintRect,
};
use std::{collections::BTreeMap, sync::Arc};

pub fn SameBuilderState(
    a: &PropertyTreeState,
    b: &PropertyTreeState,
    current_keys: &BTreeMap<u64, PaintPropertyKey>,
    prior_keys: &BTreeMap<u64, PaintPropertyKey>,
) -> bool {
    let mut compare = Compare {
        current_keys,
        prior_keys,
        memo: BTreeMap::new(),
    };
    compare.transform(&a.transform, &b.transform)
        && compare.clip(&a.clip, &b.clip)
        && compare.effect(&a.effect, &b.effect)
}

/// Chromium keeps property nodes in stable mutable owner/role slots. A value
/// change on an ancestor slot does not by itself require rebuilding clean
/// descendants; their references are rebound to the updated slot during the
/// persistence pass. This comparison admits only that case: each inherited
/// top slot must still name the same semantic owner/role entry.
pub fn SameBuilderStructure(
    a: &PropertyTreeState,
    b: &PropertyTreeState,
    current_keys: &BTreeMap<u64, PaintPropertyKey>,
    prior_keys: &BTreeMap<u64, PaintPropertyKey>,
) -> bool {
    fn same_slot<T>(
        a: &Arc<T>,
        b: &Arc<T>,
        aid: u64,
        bid: u64,
        alife: &PaintPropertyNodeLifecycle,
        blife: &PaintPropertyNodeLifecycle,
        current_keys: &BTreeMap<u64, PaintPropertyKey>,
        prior_keys: &BTreeMap<u64, PaintPropertyKey>,
    ) -> bool {
        if aid == 0 || bid == 0 {
            return aid == 0 && bid == 0 && alife.same_node(blife);
        }
        match (
            current_keys.get(&aid).or_else(|| prior_keys.get(&aid)),
            current_keys.get(&bid).or_else(|| prior_keys.get(&bid)),
        ) {
            (Some(a), Some(b)) => a == b,
            _ => Arc::ptr_eq(a, b),
        }
    }
    same_slot(
        &a.transform,
        &b.transform,
        a.transform.id,
        b.transform.id,
        &a.transform.lifecycle,
        &b.transform.lifecycle,
        current_keys,
        prior_keys,
    ) && same_slot(
        &a.clip,
        &b.clip,
        a.clip.id,
        b.clip.id,
        &a.clip.lifecycle,
        &b.clip.lifecycle,
        current_keys,
        prior_keys,
    ) && same_slot(
        &a.effect,
        &b.effect,
        a.effect.id,
        b.effect.id,
        &a.effect.lifecycle,
        &b.effect.lifecycle,
        current_keys,
        prior_keys,
    )
}

#[derive(Clone, Copy)]
enum Compared {
    Visiting,
    Done(bool),
}
type Pair = (u8, usize, usize);
struct Compare<'a> {
    current_keys: &'a BTreeMap<u64, PaintPropertyKey>,
    prior_keys: &'a BTreeMap<u64, PaintPropertyKey>,
    memo: BTreeMap<Pair, Compared>,
}
impl Compare<'_> {
    fn same_slot<T>(
        &self,
        a: &Arc<T>,
        b: &Arc<T>,
        aid: u64,
        bid: u64,
        alife: &PaintPropertyNodeLifecycle,
        blife: &PaintPropertyNodeLifecycle,
    ) -> bool {
        if aid == 0 || bid == 0 {
            return aid == 0 && bid == 0 && alife.same_node(blife);
        }
        match (
            self.current_keys
                .get(&aid)
                .or_else(|| self.prior_keys.get(&aid)),
            self.current_keys
                .get(&bid)
                .or_else(|| self.prior_keys.get(&bid)),
        ) {
            (Some(a), Some(b)) => a == b,
            // Unowned hand-built nodes have no cross-frame slot proof. The
            // same live immutable Arc is the only admissible identity there.
            _ => Arc::ptr_eq(a, b),
        }
    }
    fn start<T>(&mut self, kind: u8, a: &Arc<T>, b: &Arc<T>) -> (Pair, Option<bool>) {
        let key = (kind, Arc::as_ptr(a) as usize, Arc::as_ptr(b) as usize);
        let result = match self.memo.get(&key) {
            Some(Compared::Done(same)) => Some(*same),
            // Well-formed paint-property graphs are acyclic. Reject a cycle
            // conservatively rather than assuming equality or recursing forever.
            Some(Compared::Visiting) => Some(false),
            None => {
                self.memo.insert(key, Compared::Visiting);
                None
            }
        };
        (key, result)
    }
    fn finish(&mut self, key: Pair, result: bool) -> bool {
        self.memo.insert(key, Compared::Done(result));
        result
    }
    fn optional<T>(
        &mut self,
        a: &Option<Arc<T>>,
        b: &Option<Arc<T>>,
        compare: fn(&mut Self, &Arc<T>, &Arc<T>) -> bool,
    ) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => compare(self, a, b),
            _ => false,
        }
    }
    fn transform(
        &mut self,
        a: &Arc<TransformPaintPropertyNode>,
        b: &Arc<TransformPaintPropertyNode>,
    ) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        let (key, cached) = self.start(0, a, b);
        if let Some(result) = cached {
            return result;
        }
        // Exhaustive destructuring makes newly added semantic fields require
        // an explicit comparison here. Raw IDs/lifecycles are handled as slots.
        let TransformPaintPropertyNode {
            compositor,
            geometry_mapper_cache: _,
            scroll_parent_scroll_translation,
            lifecycle,
            id,
            parent,
            matrix,
            origin,
            flattens_inherited_transform,
            rendering_context_id,
            backface_visibility,
            scroll,
            sticky_constraint,
            direct_compositing_reasons,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && compositor == &b.compositor
            && matrix.values.iter().all(|v| v.is_finite())
            && matrix == &b.matrix
            && origin.iter().all(|v| v.is_finite())
            && origin == &b.origin
            && flattens_inherited_transform == &b.flattens_inherited_transform
            && rendering_context_id == &b.rendering_context_id
            && backface_visibility == &b.backface_visibility
            && sticky_constraint == &b.sticky_constraint
            && direct_compositing_reasons == &b.direct_compositing_reasons
            && self.optional(parent, &b.parent, Self::transform)
            && self.optional(scroll, &b.scroll, Self::scroll)
            && self.optional(
                scroll_parent_scroll_translation,
                &b.scroll_parent_scroll_translation,
                Self::transform,
            );
        self.finish(key, same)
    }
    fn clip(&mut self, a: &Arc<ClipPaintPropertyNode>, b: &Arc<ClipPaintPropertyNode>) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        let (key, cached) = self.start(1, a, b);
        if let Some(result) = cached {
            return result;
        }
        let ClipPaintPropertyNode {
            geometry_mapper_cache: _,
            layout_clip_rects,
            lifecycle,
            id,
            parent,
            local_transform_space,
            rect,
            radii,
            clip_path,
            clip_path_error,
            pixel_moving_filter,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && rect.as_ref().is_none_or(finite_rect)
            && rect == &b.rect
            && finite_radii(radii)
            && radii == &b.radii
            && layout_clip_rects == &b.layout_clip_rects
            && clip_path.as_ref().is_none_or(|path| path.is_finite())
            && clip_path == &b.clip_path
            && clip_path_error == &b.clip_path_error
            && self.optional(parent, &b.parent, Self::clip)
            && self.transform(local_transform_space, &b.local_transform_space)
            && self.optional(pixel_moving_filter, &b.pixel_moving_filter, Self::effect);
        self.finish(key, same)
    }
    fn effect(
        &mut self,
        a: &Arc<EffectPaintPropertyNode>,
        b: &Arc<EffectPaintPropertyNode>,
    ) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        let (key, cached) = self.start(2, a, b);
        if let Some(result) = cached {
            return result;
        }
        let EffectPaintPropertyNode {
            compositor,
            backdrop_filter_bounds,
            self_or_ancestor_participates_in_view_transition,
            lifecycle,
            id,
            parent,
            local_transform_space,
            output_clip,
            filter_reference_box,
            filter_reference_scale,
            opacity,
            blend_mode,
            filters,
            backdrop_filters,
            backdrop_filter_has_reference,
            backdrop_filter_clip,
            isolates_blending,
            has_mask,
            is_mask,
            direct_compositing_reasons,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && compositor == &b.compositor
            && backdrop_filter_bounds == &b.backdrop_filter_bounds
            && opacity.is_finite()
            && opacity == &b.opacity
            && self_or_ancestor_participates_in_view_transition
                == &b.self_or_ancestor_participates_in_view_transition
            && blend_mode == &b.blend_mode
            && filters.is_finite()
            && filters == &b.filters
            && filter_reference_box == &b.filter_reference_box
            && filter_reference_scale == &b.filter_reference_scale
            && backdrop_filters.is_finite()
            && backdrop_filters == &b.backdrop_filters
            && backdrop_filter_has_reference == &b.backdrop_filter_has_reference
            && self.optional(backdrop_filter_clip, &b.backdrop_filter_clip, Self::clip)
            && isolates_blending == &b.isolates_blending
            && has_mask == &b.has_mask
            && is_mask == &b.is_mask
            && direct_compositing_reasons == &b.direct_compositing_reasons
            && self.optional(parent, &b.parent, Self::effect)
            && self.transform(local_transform_space, &b.local_transform_space)
            && self.optional(output_clip, &b.output_clip, Self::clip);
        self.finish(key, same)
    }
    fn scroll(
        &mut self,
        a: &Arc<ScrollPaintPropertyNode>,
        b: &Arc<ScrollPaintPropertyNode>,
    ) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        let (key, cached) = self.start(3, a, b);
        if let Some(result) = cached {
            return result;
        }
        let ScrollPaintPropertyNode {
            compositor,
            lifecycle,
            id,
            owner_client_id,
            parent,
            overflow_clip,
            container_rect,
            contents_rect,
            minimum_scroll_offset,
            maximum_scroll_offset,
            user_scrollable_horizontal,
            user_scrollable_vertical,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && compositor == &b.compositor
            && owner_client_id == &b.owner_client_id
            && finite_rect(container_rect)
            && container_rect == &b.container_rect
            && finite_rect(contents_rect)
            && contents_rect == &b.contents_rect
            && minimum_scroll_offset == &b.minimum_scroll_offset
            && maximum_scroll_offset == &b.maximum_scroll_offset
            && user_scrollable_horizontal == &b.user_scrollable_horizontal
            && user_scrollable_vertical == &b.user_scrollable_vertical
            && self.optional(parent, &b.parent, Self::scroll)
            && self.optional(overflow_clip, &b.overflow_clip, Self::clip);
        self.finish(key, same)
    }
}
fn finite_rect(rect: &PaintRect) -> bool {
    let PaintRect {
        x,
        y,
        width,
        height,
    } = rect;
    [*x, *y, *width, *height].iter().all(|v| v.is_finite())
}
fn finite_radii(radii: &PaintCornerRadii) -> bool {
    let PaintCornerRadii {
        top_left,
        top_right,
        bottom_right,
        bottom_left,
    } = radii;
    [top_left, top_right, bottom_right, bottom_left]
        .iter()
        .all(|radius| radius.x.is_finite() && radius.y.is_finite())
}

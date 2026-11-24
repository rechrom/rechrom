//! Compare inherited builder context. PrePaint updates the resident
//! ObjectPaintProperties slot nodes in place (Chromium owner + role slots),
//! so a slot is one node: two states name the same slot exactly when they
//! name the same node entity, and every typed value/link must still match
//! before a clean child can reuse its context.
//! Sources: pre_paint_tree_walk.cc:398-447 and
//! paint_property_tree_builder.cc:4731-4770. This comparison itself neither
//! mutates node identities nor produces property invalidation.
use paint_input_types::paint_input::PaintCornerRadii;
use paint_model::{
    paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyNodeLifecycle,
        PropertyTreeState, ScrollPaintPropertyNode, ClipPaintPropertyNodeState,
        EffectPaintPropertyNodeState, ScrollPaintPropertyNodeState,
        TransformPaintPropertyNodeState, TransformPaintPropertyNode,
    },
    property_cell::PropertyAccess,
    PaintRect,
};
use std::{collections::BTreeMap, sync::Arc};

pub fn SameBuilderState(a: &PropertyTreeState, b: &PropertyTreeState) -> bool {
    // One proof for the whole comparison: the compared nodes belong to the
    // document sequence running PrePaint.
    paint_model::property_cell::with_access(|access| {
        let mut compare = Compare {
            access,
            memo: BTreeMap::new(),
        };
        compare.transform(&a.transform, &b.transform)
            && compare.clip(&a.clip, &b.clip)
            && compare.effect(&a.effect, &b.effect)
    })
}

/// The same resident slot node (Chromium's `Member` pointer compare), also
/// across the value snapshots that still share one node entity.
fn same_slot(aid: u64, bid: u64, alife: &PaintPropertyNodeLifecycle, blife: &PaintPropertyNodeLifecycle) -> bool {
    aid == bid && alife.same_node(blife)
}

/// Chromium keeps property nodes in stable mutable owner/role slots. A value
/// change on an ancestor slot does not by itself require rebuilding clean
/// descendants: they reference the same updated slot node. This comparison
/// admits only that case: each inherited top slot must be the same node.
pub fn SameBuilderStructure(a: &PropertyTreeState, b: &PropertyTreeState) -> bool {
    same_slot(a.transform.id, b.transform.id, &a.transform.lifecycle, &b.transform.lifecycle)
        && same_slot(a.clip.id, b.clip.id, &a.clip.lifecycle, &b.clip.lifecycle)
        && same_slot(a.effect.id, b.effect.id, &a.effect.lifecycle, &b.effect.lifecycle)
}

#[derive(Clone, Copy)]
enum Compared {
    Visiting,
    Done(bool),
}
type Pair = (u8, usize, usize);
struct Compare<'a> {
    access: &'a PropertyAccess,
    memo: BTreeMap<Pair, Compared>,
}
impl Compare<'_> {
    fn same_slot<T>(
        &self,
        _a: &Arc<T>,
        _b: &Arc<T>,
        aid: u64,
        bid: u64,
        alife: &PaintPropertyNodeLifecycle,
        blife: &PaintPropertyNodeLifecycle,
    ) -> bool {
        same_slot(aid, bid, alife, blife)
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
        let access = self.access;
        let (lifecycle, id) = (&a.lifecycle, &a.id);
        let TransformPaintPropertyNodeState {
            compositor,
            scroll_parent_scroll_translation,
            parent,
            matrix,
            origin,
            flattens_inherited_transform,
            rendering_context_id,
            backface_visibility,
            scroll,
            sticky_constraint,
            direct_compositing_reasons,
        } = a.state(access);
        let bs = b.state(access);
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && compositor == &bs.compositor
            && matrix.values.iter().all(|v| v.is_finite())
            && matrix == &bs.matrix
            && origin.iter().all(|v| v.is_finite())
            && origin == &bs.origin
            && flattens_inherited_transform == &bs.flattens_inherited_transform
            && rendering_context_id == &bs.rendering_context_id
            && backface_visibility == &bs.backface_visibility
            && sticky_constraint == &bs.sticky_constraint
            && direct_compositing_reasons == &bs.direct_compositing_reasons
            && self.optional(parent, &bs.parent, Self::transform)
            && self.optional(scroll, &bs.scroll, Self::scroll)
            && self.optional(
                scroll_parent_scroll_translation,
                &bs.scroll_parent_scroll_translation,
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
        let access = self.access;
        let (lifecycle, id) = (&a.lifecycle, &a.id);
        let ClipPaintPropertyNodeState {
            layout_clip_rects,
            parent,
            local_transform_space,
            rect,
            radii,
            clip_path,
            clip_path_error,
            pixel_moving_filter,
        } = a.state(access);
        let bs = b.state(access);
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && rect.as_ref().is_none_or(finite_rect)
            && rect == &bs.rect
            && finite_radii(radii)
            && radii == &bs.radii
            && layout_clip_rects == &bs.layout_clip_rects
            && clip_path.as_ref().is_none_or(|path| path.is_finite())
            && clip_path == &bs.clip_path
            && clip_path_error == &bs.clip_path_error
            && self.optional(parent, &bs.parent, Self::clip)
            && self.transform(local_transform_space, &bs.local_transform_space)
            && self.optional(pixel_moving_filter, &bs.pixel_moving_filter, Self::effect);
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
        let access = self.access;
        let (lifecycle, id) = (&a.lifecycle, &a.id);
        let EffectPaintPropertyNodeState {
            compositor,
            backdrop_filter_bounds,
            self_or_ancestor_participates_in_view_transition,
            parent,
            local_transform_space,
            output_clip,
            filter_reference_box,
            filter_viewport,
            filter_space_origin,
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
        } = a.state(access);
        let bs = b.state(access);
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && compositor == &bs.compositor
            && backdrop_filter_bounds == &bs.backdrop_filter_bounds
            && opacity.is_finite()
            && opacity == &bs.opacity
            && self_or_ancestor_participates_in_view_transition
                == &bs.self_or_ancestor_participates_in_view_transition
            && blend_mode == &bs.blend_mode
            && filters.is_finite()
            && filters == &bs.filters
            && filter_reference_box == &bs.filter_reference_box
            && filter_viewport == &bs.filter_viewport
            && filter_space_origin == &bs.filter_space_origin
            && filter_reference_scale == &bs.filter_reference_scale
            && backdrop_filters.is_finite()
            && backdrop_filters == &bs.backdrop_filters
            && backdrop_filter_has_reference == &bs.backdrop_filter_has_reference
            && self.optional(backdrop_filter_clip, &bs.backdrop_filter_clip, Self::clip)
            && isolates_blending == &bs.isolates_blending
            && has_mask == &bs.has_mask
            && is_mask == &bs.is_mask
            && direct_compositing_reasons == &bs.direct_compositing_reasons
            && self.optional(parent, &bs.parent, Self::effect)
            && self.transform(local_transform_space, &bs.local_transform_space)
            && self.optional(output_clip, &bs.output_clip, Self::clip);
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
        let access = self.access;
        let (lifecycle, id) = (&a.lifecycle, &a.id);
        let ScrollPaintPropertyNodeState {
            compositor,
            owner_client_id,
            owner_node_id,
            parent,
            overflow_clip,
            container_rect,
            contents_rect,
            minimum_scroll_offset,
            maximum_scroll_offset,
            user_scrollable_horizontal,
            user_scrollable_vertical,
        } = a.state(access);
        let bs = b.state(access);
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && compositor == &bs.compositor
            && owner_client_id == &bs.owner_client_id
            && owner_node_id == &bs.owner_node_id
            && finite_rect(container_rect)
            && container_rect == &bs.container_rect
            && finite_rect(contents_rect)
            && contents_rect == &bs.contents_rect
            && minimum_scroll_offset == &bs.minimum_scroll_offset
            && maximum_scroll_offset == &bs.maximum_scroll_offset
            && user_scrollable_horizontal == &bs.user_scrollable_horizontal
            && user_scrollable_vertical == &bs.user_scrollable_vertical
            && self.optional(parent, &bs.parent, Self::scroll)
            && self.optional(overflow_clip, &bs.overflow_clip, Self::clip);
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

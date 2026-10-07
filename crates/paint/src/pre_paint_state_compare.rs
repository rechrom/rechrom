//! Compare inherited builder context before persistence/canonicalization.
//! Blink reuses native property-node slots; this builder instead creates raw
//! snapshots. Native owner + role identifies that same slot, and every typed
//! value/link must still match before a clean child can reuse its context.
//! Sources: pre_paint_tree_walk.cc:398-447 and
//! paint_property_tree_builder.cc:4731-4770. This comparison itself neither
//! mutates node identities nor produces property invalidation.
use crate::{
    paint_property_tree::{
        ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyKey,
        PaintPropertyNodeLifecycle, PropertyTreeState, ScrollPaintPropertyNode,
        TransformPaintPropertyNode,
    },
    PaintRect,
};
use layoutng_assembly::internal::{
    layout_input::PaintPathCommand,
    paint_input::{PaintCornerRadii, PaintFilterOperation},
};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) fn SameBuilderState(
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
            lifecycle,
            id,
            parent,
            matrix,
            origin,
            scroll,
            direct_compositing_reasons,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && matrix.values.iter().all(|v| v.is_finite())
            && matrix == &b.matrix
            && origin.iter().all(|v| v.is_finite())
            && origin == &b.origin
            && direct_compositing_reasons == &b.direct_compositing_reasons
            && self.optional(parent, &b.parent, Self::transform)
            && self.optional(scroll, &b.scroll, Self::scroll);
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
            lifecycle,
            id,
            parent,
            local_transform_space,
            rect,
            radii,
            clip_path,
            clip_path_even_odd,
            pixel_moving_filter,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && rect.as_ref().is_none_or(finite_rect)
            && rect == &b.rect
            && finite_radii(radii)
            && radii == &b.radii
            && clip_path.iter().all(finite_path)
            && clip_path == &b.clip_path
            && clip_path_even_odd == &b.clip_path_even_odd
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
            lifecycle,
            id,
            parent,
            local_transform_space,
            output_clip,
            opacity,
            blend_mode,
            filters,
            isolates_blending,
            has_mask,
            is_mask,
            direct_compositing_reasons,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && opacity.is_finite()
            && opacity == &b.opacity
            && blend_mode == &b.blend_mode
            && filters.iter().all(finite_filter)
            && filters == &b.filters
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
            lifecycle,
            id,
            parent,
            overflow_clip,
            container_rect,
            contents_rect,
            user_scrollable_horizontal,
            user_scrollable_vertical,
        } = &**a;
        let same = self.same_slot(a, b, *id, b.id, lifecycle, &b.lifecycle)
            && finite_rect(container_rect)
            && container_rect == &b.container_rect
            && finite_rect(contents_rect)
            && contents_rect == &b.contents_rect
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
fn finite_path(command: &PaintPathCommand) -> bool {
    let PaintPathCommand {
        verb: _,
        control1,
        control2,
        point,
        conic_weight,
    } = command;
    [control1, control2, point]
        .iter()
        .all(|p| p.x.is_finite() && p.y.is_finite())
        && conic_weight.is_finite()
}
fn finite_filter(filter: &PaintFilterOperation) -> bool {
    let PaintFilterOperation {
        r#type: _,
        amount,
        offset,
        blur_radius,
        color,
    } = filter;
    amount.is_finite()
        && offset.x.is_finite()
        && offset.y.is_finite()
        && blur_radius.is_finite()
        && [color.red, color.green, color.blue, color.alpha]
            .iter()
            .all(|v| v.is_finite())
}

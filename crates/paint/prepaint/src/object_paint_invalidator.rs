//! Object paint invalidation during the PrePaint lifecycle.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/object_paint_invalidator.{h,cc}`.

use foundation::PaintInvalidationReason;
use layoutng_assembly::fragment_tree::{FragmentKind, FragmentPaintData};
use layoutng_assembly::internal::layout_input::Offset;
use layoutng_assembly::internal::paint_input::BackgroundBox;

use crate::paint_invalidator::PaintInvalidatorContext;

pub struct ObjectPaintInvalidator;

impl ObjectPaintInvalidator {
    /// Source: `object_paint_invalidator.cc:49-93`. Every fragment-local
    /// branch is preserved in source order.
    pub fn ComputePaintInvalidationReason(
        fragment: &FragmentPaintData,
        old_paint_offset: Option<Offset>,
        new_paint_offset: Offset,
        context: PaintInvalidatorContext,
    ) -> PaintInvalidationReason {
        let invalidation = fragment.paint.pre_paint_invalidation;
        if invalidation.visual_rect_respects_visibility
            && !invalidation.previous_visibility_visible
            && !fragment.paint.style.visible()
        {
            return PaintInvalidationReason::kNone;
        }

        if !invalidation.should_check_for_paint_invalidation && !context.NeedsSubtreeWalk() {
            return PaintInvalidationReason::kNone;
        }

        if context.HasSubtreeFullInvalidation() {
            return PaintInvalidationReason::kSubtree;
        }

        if old_paint_offset.is_some_and(|old| old != new_paint_offset) {
            return PaintInvalidationReason::kLayout;
        }

        if let Some(reason) = invalidation.full_paint_invalidation_reason {
            return reason;
        }

        if fragment.paint.InForcedColorsMode() && fragment.paint.IsLayoutBlockFlow() {
            return PaintInvalidationReason::kBackplate;
        }

        if matches!(
            fragment.paint.style.background_clip(),
            BackgroundBox::kText | BackgroundBox::kBorderAreaText
        ) {
            return PaintInvalidationReason::kBackground;
        }

        if fragment.kind == FragmentKind::kBox {
            return PaintInvalidationReason::kIncremental;
        }

        PaintInvalidationReason::kNone
    }

    /// Source: `object_paint_invalidator.cc:30-46,95-121`.
    pub fn InvalidatePaintWithComputedReason(
        fragment: &mut FragmentPaintData,
        mut reason: PaintInvalidationReason,
    ) {
        if reason == PaintInvalidationReason::kNone {
            let invalidation = fragment.paint.pre_paint_invalidation;
            if !invalidation.should_invalidate_selection || !invalidation.can_be_selection_leaf {
                return;
            }
            reason = PaintInvalidationReason::kSelection;
            if fragment.paint.selection_client_id != 0 {
                let current = fragment
                    .paint
                    .selection_client_invalidation_reason
                    .unwrap_or(PaintInvalidationReason::kNone);
                if reason > current {
                    fragment.paint.selection_client_invalidation_reason = Some(reason);
                }
                return;
            }
        }
        let current = fragment
            .paint
            .display_item_client_invalidation_reason
            .unwrap_or(PaintInvalidationReason::kNone);
        if reason > current {
            fragment.paint.display_item_client_invalidation_reason = Some(reason);
        }
        // LayoutText::InvalidateDisplayItemClients invalidates both its base
        // client and the independent selection client for ordinary reasons.
        if fragment.paint.selection_client_id != 0 {
            let current = fragment
                .paint
                .selection_client_invalidation_reason
                .unwrap_or(PaintInvalidationReason::kNone);
            if reason > current {
                fragment.paint.selection_client_invalidation_reason = Some(reason);
            }
        }
    }

    pub fn InvalidatePaint(
        fragment: &mut FragmentPaintData,
        old_paint_offset: Option<Offset>,
        new_paint_offset: Offset,
        context: PaintInvalidatorContext,
    ) -> PaintInvalidationReason {
        let reason = Self::ComputePaintInvalidationReason(
            fragment,
            old_paint_offset,
            new_paint_offset,
            context,
        );
        Self::InvalidatePaintWithComputedReason(fragment, reason);
        reason
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_full_reason_is_applied_in_prepaint_and_keeps_stronger_client_state() {
        let mut fragment = FragmentPaintData::default();
        fragment
            .paint
            .pre_paint_invalidation
            .full_paint_invalidation_reason = Some(PaintInvalidationReason::kStyle);
        fragment
            .paint
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        ObjectPaintInvalidator::InvalidatePaint(
            &mut fragment,
            None,
            Offset::default(),
            PaintInvalidatorContext::default(),
        );
        assert_eq!(
            fragment.paint.display_item_client_invalidation_reason,
            Some(PaintInvalidationReason::kStyle)
        );

        fragment.paint.display_item_client_invalidation_reason =
            Some(PaintInvalidationReason::kJustCreated);
        ObjectPaintInvalidator::InvalidatePaint(
            &mut fragment,
            None,
            Offset::default(),
            PaintInvalidatorContext::default(),
        );
        assert_eq!(
            fragment.paint.display_item_client_invalidation_reason,
            Some(PaintInvalidationReason::kJustCreated)
        );
    }

    #[test]
    fn source_order_prefers_subtree_then_layout_then_full_reason() {
        let mut fragment = FragmentPaintData::default();
        fragment
            .paint
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        fragment
            .paint
            .pre_paint_invalidation
            .full_paint_invalidation_reason = Some(PaintInvalidationReason::kStyle);

        let layout = ObjectPaintInvalidator::ComputePaintInvalidationReason(
            &fragment,
            Some(Offset::default()),
            Offset { x: 1.0, y: 0.0 },
            PaintInvalidatorContext::default(),
        );
        assert_eq!(layout, PaintInvalidationReason::kLayout);

        fragment
            .paint
            .pre_paint_invalidation
            .subtree_should_do_full_paint_invalidation = true;
        let mut context = PaintInvalidatorContext::default();
        crate::paint_invalidator::PaintInvalidator::UpdateContext(&fragment, &mut context);
        let subtree = ObjectPaintInvalidator::ComputePaintInvalidationReason(
            &fragment,
            Some(Offset::default()),
            Offset { x: 1.0, y: 0.0 },
            context,
        );
        assert_eq!(subtree, PaintInvalidationReason::kSubtree);
    }

    #[test]
    fn forced_colors_block_flow_uses_backplate_reason() {
        let mut fragment = FragmentPaintData::default();
        fragment
            .paint
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        fragment.paint.in_forced_colors_mode = true;
        fragment.paint.is_layout_block_flow = true;
        assert_eq!(
            ObjectPaintInvalidator::ComputePaintInvalidationReason(
                &fragment,
                Some(Offset::default()),
                Offset::default(),
                PaintInvalidatorContext::default(),
            ),
            PaintInvalidationReason::kBackplate
        );
    }

    #[test]
    fn selection_only_invalidation_targets_the_selection_client() {
        let mut fragment = FragmentPaintData::default();
        fragment
            .paint
            .pre_paint_invalidation
            .should_invalidate_selection = true;
        fragment.paint.pre_paint_invalidation.can_be_selection_leaf = true;
        fragment.paint.selection_client_id = 42;

        ObjectPaintInvalidator::InvalidatePaintWithComputedReason(
            &mut fragment,
            PaintInvalidationReason::kNone,
        );

        assert_eq!(
            fragment.paint.selection_client_invalidation_reason,
            Some(PaintInvalidationReason::kSelection)
        );
        assert_eq!(fragment.paint.display_item_client_invalidation_reason, None);
    }

    #[test]
    fn ordinary_text_invalidation_targets_base_and_selection_clients() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.selection_client_id = 42;

        ObjectPaintInvalidator::InvalidatePaintWithComputedReason(
            &mut fragment,
            PaintInvalidationReason::kStyle,
        );

        assert_eq!(
            fragment.paint.display_item_client_invalidation_reason,
            Some(PaintInvalidationReason::kStyle)
        );
        assert_eq!(
            fragment.paint.selection_client_invalidation_reason,
            Some(PaintInvalidationReason::kStyle)
        );
    }
}

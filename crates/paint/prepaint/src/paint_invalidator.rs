//! Paint invalidation context propagation during PrePaint.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/paint_invalidator.{h,cc}`.

use std::collections::HashSet;

use foundation::{IsFullPaintInvalidationReason, PaintInvalidationReason};
use layoutng_assembly::fragment_tree::FragmentPaintData;
use layoutng_assembly::internal::layout_input::{Display, Offset};

/// Source: `paint_invalidator.h:20-72`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintInvalidatorContext {
    subtree_flags: u8,
}

impl PaintInvalidatorContext {
    const SUBTREE_INVALIDATION_CHECKING: u8 = 1 << 0;
    const SUBTREE_FULL_INVALIDATION: u8 = 1 << 1;
    const SUBTREE_FULL_INVALIDATION_FOR_STACKED_CONTENTS: u8 = 1 << 2;
    const SUBTREE_NO_INVALIDATION: u8 = 1 << 6;

    /// Source: `paint_invalidator.h:31-36`.
    pub fn NeedsSubtreeWalk(self) -> bool {
        self.subtree_flags
            & (Self::SUBTREE_INVALIDATION_CHECKING
                | Self::SUBTREE_FULL_INVALIDATION
                | Self::SUBTREE_FULL_INVALIDATION_FOR_STACKED_CONTENTS)
            != 0
    }

    pub(crate) fn HasSubtreeFullInvalidation(self) -> bool {
        self.subtree_flags & Self::SUBTREE_FULL_INVALIDATION != 0
    }

    fn HasNoInvalidation(self) -> bool {
        self.subtree_flags & Self::SUBTREE_NO_INVALIDATION != 0
    }
}

/// Source: `paint_invalidator.h:82-110`.
#[derive(Default)]
pub struct PaintInvalidator {
    pending_delayed_paint_invalidations: HashSet<u64>,
}

impl PaintInvalidator {
    pub(crate) fn BeginPass(&mut self) {
        self.pending_delayed_paint_invalidations.clear();
    }

    /// Source: `paint_invalidator.cc:55-75`. When the complete movement is the
    /// propagated subpixel delta of a directly composited container, Chromium
    /// treats the current offset as the invalidation baseline. The compositor
    /// moves the layer without rerecording its display items.
    pub(crate) fn UpdateFromTreeBuilderContext(
        directly_composited_container_paint_offset_subpixel_delta: Offset,
        old_paint_offset: Option<Offset>,
        current_paint_offset: Offset,
    ) -> Option<Offset> {
        old_paint_offset.map(|old| {
            let movement = Offset {
                x: current_paint_offset.x - old.x,
                y: current_paint_offset.y - old.y,
            };
            if directly_composited_container_paint_offset_subpixel_delta == movement {
                current_paint_offset
            } else {
                old
            }
        })
    }

    /// Prepares the child context and reports whether this object must execute
    /// its invalidation step. Source: `paint_invalidator.cc:207-252`.
    pub fn UpdateContext(
        fragment: &FragmentPaintData,
        context: &mut PaintInvalidatorContext,
    ) -> bool {
        if fragment.paint.svg_hidden_container
            || matches!(
                fragment.paint.display,
                Display::kTableColumn | Display::kTableColumnGroup
            )
        {
            context.subtree_flags |= PaintInvalidatorContext::SUBTREE_NO_INVALIDATION;
        }

        if context.HasNoInvalidation() {
            return false;
        }

        let invalidation = fragment.paint.pre_paint_invalidation;
        if !invalidation.should_check_for_paint_invalidation && !context.NeedsSubtreeWalk() {
            return false;
        }

        if invalidation.subtree_should_do_full_paint_invalidation {
            context.subtree_flags |= PaintInvalidatorContext::SUBTREE_FULL_INVALIDATION
                | PaintInvalidatorContext::SUBTREE_FULL_INVALIDATION_FOR_STACKED_CONTENTS;
        }
        if invalidation.subtree_should_check_for_paint_invalidation {
            context.subtree_flags |= PaintInvalidatorContext::SUBTREE_INVALIDATION_CHECKING;
        }
        true
    }

    /// Source: `paint_invalidator.cc:270-276`. Chromium checks the client
    /// after object invalidation, so use the resident client reason rather
    /// than the locally computed reason.
    pub(crate) fn TrackPendingDelayedPaintInvalidation(&mut self, fragment: &FragmentPaintData) {
        let invalidation = fragment.paint.pre_paint_invalidation;
        let reason = fragment
            .paint
            .display_item_client_invalidation_reason
            .unwrap_or(PaintInvalidationReason::kNone);
        if invalidation.should_delay_full_paint_invalidation
            && (!IsFullPaintInvalidationReason(reason)
                || reason == PaintInvalidationReason::kJustCreated)
            && fragment.paint.display_item_client_id != 0
        {
            self.pending_delayed_paint_invalidations
                .insert(fragment.paint.display_item_client_id);
        }
    }

    /// Source: `paint_invalidator.cc:291-294`. The reciprocal Layout commit
    /// applies these IDs to their source LayoutObjects after Paint succeeds.
    pub fn PendingDelayedPaintInvalidations(&self) -> &HashSet<u64> {
        &self.pending_delayed_paint_invalidations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtree_full_invalidation_propagates_to_children() {
        let mut root = FragmentPaintData::default();
        root.paint
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        root.paint
            .pre_paint_invalidation
            .subtree_should_do_full_paint_invalidation = true;
        let mut context = PaintInvalidatorContext::default();
        assert!(PaintInvalidator::UpdateContext(&root, &mut context));
        assert!(context.HasSubtreeFullInvalidation());

        let child = FragmentPaintData::default();
        let mut child_context = context;
        assert!(PaintInvalidator::UpdateContext(&child, &mut child_context));
    }

    #[test]
    fn svg_hidden_container_suppresses_its_subtree() {
        let mut root = FragmentPaintData::default();
        root.paint.svg_hidden_container = true;
        root.paint
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        let mut context = PaintInvalidatorContext::default();
        assert!(!PaintInvalidator::UpdateContext(&root, &mut context));

        let mut child = FragmentPaintData::default();
        child
            .paint
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        assert!(!PaintInvalidator::UpdateContext(&child, &mut context));
    }

    #[test]
    fn delayed_full_invalidation_is_retained_only_until_a_real_full_reason() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.display_item_client_id = 41;
        fragment
            .paint
            .pre_paint_invalidation
            .should_delay_full_paint_invalidation = true;
        fragment.paint.display_item_client_invalidation_reason =
            Some(PaintInvalidationReason::kIncremental);

        let mut invalidator = PaintInvalidator::default();
        invalidator.TrackPendingDelayedPaintInvalidation(&fragment);
        assert_eq!(
            invalidator.PendingDelayedPaintInvalidations(),
            &HashSet::from([41])
        );

        invalidator.BeginPass();
        fragment.paint.display_item_client_invalidation_reason =
            Some(PaintInvalidationReason::kStyle);
        invalidator.TrackPendingDelayedPaintInvalidation(&fragment);
        assert!(invalidator.PendingDelayedPaintInvalidations().is_empty());

        fragment.paint.display_item_client_invalidation_reason =
            Some(PaintInvalidationReason::kJustCreated);
        invalidator.TrackPendingDelayedPaintInvalidation(&fragment);
        assert_eq!(
            invalidator.PendingDelayedPaintInvalidations(),
            &HashSet::from([41])
        );
    }

    #[test]
    fn composited_subpixel_movement_uses_current_offset_as_baseline() {
        let old = Offset { x: 0.25, y: 0.5 };
        let current = Offset { x: 0.5, y: 0.75 };
        assert_eq!(
            PaintInvalidator::UpdateFromTreeBuilderContext(
                Offset { x: 0.25, y: 0.25 },
                Some(old),
                current,
            ),
            Some(current)
        );
        assert_eq!(
            PaintInvalidator::UpdateFromTreeBuilderContext(
                Offset { x: 0.0, y: 0.25 },
                Some(old),
                current,
            ),
            Some(old)
        );
    }
}

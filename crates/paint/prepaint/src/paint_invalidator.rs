//! Paint invalidation context propagation during PrePaint.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/paint_invalidator.{h,cc}`.

use std::collections::HashSet;
use std::ptr::NonNull;

use foundation::{IsFullPaintInvalidationReason, PaintInvalidationReason};
use layout::fragment::{FragmentKind, PrePaintSourceObject, ResidentBoxGeometry as _};
use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use layout::input::Display;
use paint_input_types::layout_input::Offset;

/// Source: `paint_invalidator.h:20-72`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PaintInvalidatorContext {
    subtree_flags: u8,
    /// `painting_layer` (paint_invalidator.h:65): the PaintLayer that paints
    /// the visited object, inherited from the parent context and replaced by
    /// UpdatePaintingLayer. The layer is owned by its LayoutBoxModelObject
    /// for the whole walk; `None` before the first self-painting layer and
    /// in hand-built fixtures.
    painting_layer: Option<NonNull<PaintLayer>>,
}

/// The context's invalidation state; `painting_layer` is derived from the
/// visited object (DCHECK_EQ(context.painting_layer, object.PaintingLayer()),
/// paint_invalidator.cc:230), so retained-context proofs ignore it.
impl PartialEq for PaintInvalidatorContext {
    fn eq(&self, other: &Self) -> bool {
        self.subtree_flags == other.subtree_flags
    }
}

impl Eq for PaintInvalidatorContext {}

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
    /// Client identities of `pending_delayed_paint_invalidations_`; they
    /// de-duplicate the fragments of one object.
    pending_delayed_paint_invalidations: HashSet<u64>,
    /// The source LayoutObjects of those clients, re-armed after the walk.
    pending_delayed_sources: Vec<PrePaintSourceObject>,
}

impl PaintInvalidator {
    pub(crate) fn BeginPass(&mut self) {
        self.pending_delayed_paint_invalidations.clear();
        self.pending_delayed_sources.clear();
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

    /// Source: `paint_invalidator.cc:33-53`. InvalidatePaint runs it for every
    /// visited object outside a no-invalidation subtree, before the
    /// should-check gate (cc:217-225): the context's painting layer becomes
    /// this object's self-painting layer, or its PaintingLayer() for inline
    /// ruby text, and that layer records the float and descendant-outline
    /// paint phases its painter must visit.
    ///
    /// Dual write: PaintLayerPainter still reads the walk's
    /// `needs_float_phase` / `needs_outline_phase` node flags
    /// (MarkPaintPhaseNeeds); these PaintLayer bits are recorded alongside.
    /// Like Chromium they are sticky (paint_layer.h:490-507): only a new
    /// PaintLayer starts without them.
    pub(crate) fn UpdatePaintingLayer(
        cursor: PaintFragmentCursor<'_>,
        context: &mut PaintInvalidatorContext,
    ) {
        if context.HasNoInvalidation() {
            return;
        }
        // A line box is a fragment of its container, not a LayoutObject that
        // InvalidatePaint visits.
        if cursor.ViewKind() == FragmentKind::kLine {
            return;
        }
        // cc:35-42.
        if cursor.SourceHasSelfPaintingLayer() {
            context.painting_layer = cursor.OwnPaintLayer().map(NonNull::from);
        } else if cursor.SourceIsInlineRubyText() || context.painting_layer.is_none() {
            // The second condition covers a walk that starts below the
            // LayoutView, where Chromium's context would carry the ancestor's
            // layer (DCHECK_EQ at cc:230).
            context.painting_layer = cursor.PaintingLayer().map(NonNull::from);
        }
        let Some(painting_layer) = context.painting_layer else {
            return;
        };
        // The painting layer is owned by its LayoutBoxModelObject, alive for
        // the walk; its phase bits are sequence-confined interior state.
        let painting_layer = unsafe { painting_layer.as_ref() };
        // cc:44-46.
        if cursor.SourceIsFloating() {
            painting_layer.SetNeedsPaintPhaseFloat();
        }
        // cc:48-52.
        if !painting_layer.NeedsPaintPhaseDescendantOutlines()
            && !cursor.IsLayerObject(painting_layer)
            && cursor.SourceHasOutline()
        {
            painting_layer.SetNeedsPaintPhaseDescendantOutlines();
        }
    }

    /// Prepares the child context and reports whether this object must execute
    /// its invalidation step. Source: `paint_invalidator.cc:207-252`.
    pub fn UpdateContext(
        fragment: &FragmentPaintData,
        context: &mut PaintInvalidatorContext,
    ) -> bool {
        if fragment.paint.IsSVGHiddenContainer()
            || matches!(
                fragment.paint.Display(),
                Display::kTableColumn | Display::kTableColumnGroup
            )
        {
            context.subtree_flags |= PaintInvalidatorContext::SUBTREE_NO_INVALIDATION;
        }

        if context.HasNoInvalidation() {
            return false;
        }

        let invalidation = fragment.pre_paint_invalidation;
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
        let invalidation = fragment.pre_paint_invalidation;
        // The live client query is needed only for a delayed invalidation.
        if !invalidation.should_delay_full_paint_invalidation {
            return;
        }
        let reason = fragment
            .paint
            .DisplayItemClientState()
            .invalidation_reason
            .unwrap_or(PaintInvalidationReason::kNone);
        if invalidation.should_delay_full_paint_invalidation
            && (!IsFullPaintInvalidationReason(reason)
                || reason == PaintInvalidationReason::kJustCreated)
            && fragment.paint.DisplayItemClientId() != 0
        {
            if self
                .pending_delayed_paint_invalidations
                .insert(fragment.paint.DisplayItemClientId())
            {
                if let Some(source) = fragment.paint.PrePaintSourceObject() {
                    self.pending_delayed_sources.push(source);
                }
            }
        }
    }

    /// Client identities scheduled by this pass for a delayed invalidation.
    pub fn PendingDelayedPaintInvalidations(&self) -> &HashSet<u64> {
        &self.pending_delayed_paint_invalidations
    }

    /// Source: `paint_invalidator.cc:291-294`, called by
    /// PrePaintTreeWalk::WalkTree right after the walk
    /// (`pre_paint_tree_walk.cc:106-107`): the walk has cleared each visited
    /// object's flags, so re-arm the delayed targets for the next pass.
    pub(crate) fn ProcessPendingDelayedPaintInvalidations(&mut self) {
        for source in self.pending_delayed_sources.drain(..) {
            source.SetShouldDelayFullPaintInvalidation();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtree_full_invalidation_propagates_to_children() {
        let mut root = FragmentPaintData::default();
        root
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        root
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
        root
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        let mut context = PaintInvalidatorContext::default();
        assert!(!PaintInvalidator::UpdateContext(&root, &mut context));

        let mut child = FragmentPaintData::default();
        child
            
            .pre_paint_invalidation
            .should_check_for_paint_invalidation = true;
        assert!(!PaintInvalidator::UpdateContext(&child, &mut context));
    }

    #[test]
    fn delayed_full_invalidation_is_retained_only_until_a_real_full_reason() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.display_item_client_id = 41;
        fragment
            
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

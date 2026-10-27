//! LayoutObject prepaint, selection, and interaction invalidation.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2906-2911
    pub fn EffectiveAllowedTouchAction(&self) -> TouchAction {
        self.CheckIsNotDestroyed();
        if self.InsideBlockingTouchEventHandler() {
            TouchAction::kNone
        } else {
            self.StyleRef().EffectiveTouchAction()
        }
    }

    // cpp: layoutng/internal/layout_object.h:2912-2915
    pub fn HasEffectiveAllowedTouchAction(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.EffectiveAllowedTouchAction() != TouchAction::kAuto
    }

    // cpp: layoutng/internal/layout_object.h:2820-2820
    pub fn SetShouldDelayFullPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectSetShouldDelayFullPaintInvalidationProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2963-2963
    pub fn SetShouldInheritContainerTimingRoot(&mut self, inherit: bool) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectSetShouldInheritContainerTimingRootProvider(self, inherit) }
    }

    // cpp: layoutng/internal/layout_object.h:3444-3444
    // cpp: layoutng/internal/layout_object.cc:5368-5382
    pub fn ClearPaintFlags(&mut self) {
        self.CheckIsNotDestroyed();
        self.ClearPaintInvalidationFlags();
        self.bitfields_.needs_paint_property_update_ = false;
        self.bitfields_.pre_paint_subtree_walk_reasons_ = 0;

        // The translated input boundary currently exposes the display-lock
        // decision that gates child layout. A blocked child subtree must keep
        // its propagated pre-paint state for the later unlock walk, matching
        // Chromium's ChildPrePaintBlockedByDisplayLock() branch.
        if !self.ChildLayoutBlockedByDisplayLock() {
            self.bitfields_.descendant_needs_paint_property_update_ = false;
            self.bitfields_.descendant_pre_paint_subtree_walk_reasons_ = 0;
            self.bitfields_.subtree_paint_property_update_reasons_ = 0;
        }
    }

    // cpp: layoutng/internal/layout_object.h:3443-3443
    #[inline(never)]
    pub fn EnsureIsReadyForPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectEnsureIsReadyForPaintInvalidationProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1914-1914
    #[inline(never)]
    pub fn InvalidateSubtreePositionTry(&mut self, mark_style_dirty: bool) {
        unsafe { LayoutObjectInvalidateSubtreePositionTryProvider(self, mark_style_dirty) }
    }

    // cpp: layoutng/internal/layout_object.h:2761-2761
    pub fn SetShouldInvalidatePaintForHitTest(&mut self) {
        unsafe { LayoutObjectSetShouldInvalidatePaintForHitTestProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2868
    // cpp: layoutng/internal/layout_object.cc:5318-5339
    pub fn ClearPaintInvalidationFlags(&mut self) {
        self.CheckIsNotDestroyed();
        if !self.bitfields_.should_delay_full_paint_invalidation_ {
            self.bitfields_.paint_invalidation_reason_for_pre_paint_ =
                PaintInvalidationReason::kNone as u8;
            self.bitfields_.background_needs_full_paint_invalidation_ = false;
        }
        self.bitfields_.should_check_for_paint_invalidation_ = false;
        self.bitfields_.subtree_should_check_for_paint_invalidation_ = false;
        self.bitfields_.subtree_should_do_full_paint_invalidation_ = false;
        self.bitfields_
            .may_need_paint_invalidation_animated_background_image_ = false;
        self.bitfields_.should_check_layout_for_paint_invalidation_ = false;
        self.bitfields_
            .descendant_should_check_layout_for_paint_invalidation_ = false;
        self.bitfields_.should_invalidate_selection_ = false;
    }

    // cpp: layoutng/internal/layout_object.h:2802-2802
    pub fn SetMayNeedPaintInvalidationAnimatedBackgroundImage(&mut self) {
        unsafe { LayoutObjectSetMayNeedPaintInvalidationAnimatedBackgroundImageProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2821-2821
    pub fn ClearShouldDelayFullPaintInvalidation(&mut self) {
        unsafe { LayoutObjectClearShouldDelayFullPaintInvalidationProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2827-2827
    pub fn SetShouldInvalidateSelection(&mut self) {
        unsafe { LayoutObjectSetShouldInvalidateSelectionProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3561-3561
    #[inline(never)]
    pub(crate) fn InvalidateSelectedChildrenOnStyleChange(&mut self) {
        unsafe { LayoutObjectInvalidateSelectedChildrenOnStyleChangeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2882-2882
    #[inline(never)]
    pub fn InvalidateSelectionOnStyleChange(&mut self) {
        unsafe { LayoutObjectInvalidateSelectionOnStyleChangeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2891-2891
    #[inline(never)]
    pub fn SetNeedsPrePaintSubtreeWalk(&mut self, reasons: PrePaintSubtreeWalkReasons) {
        unsafe { LayoutObjectSetNeedsPrePaintSubtreeWalkProvider(self, reasons) }
    }

    // cpp: layoutng/internal/layout_object.h:2897-2897
    #[inline(never)]
    pub fn SetDescendantNeedsPrePaintSubtreeWalk(&mut self, reasons: PrePaintSubtreeWalkReasons) {
        unsafe { LayoutObjectSetDescendantNeedsPrePaintSubtreeWalkProvider(self, reasons) }
    }

    // cpp: layoutng/internal/layout_object.h:2923-2923
    #[inline(never)]
    pub fn MarkEffectiveAllowedTouchActionChanged(&mut self) {
        unsafe { LayoutObjectMarkEffectiveAllowedTouchActionChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2938-2938
    #[inline(never)]
    pub fn MarkBlockingWheelEventHandlerChanged(&mut self) {
        unsafe { LayoutObjectMarkBlockingWheelEventHandlerChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2944-2944
    #[inline(never)]
    pub fn MarkSoftNavigationContextChanged(&mut self) {
        unsafe { LayoutObjectMarkSoftNavigationContextChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2958-2958
    #[inline(never)]
    pub fn MarkContainerTimingChanged(&mut self) {
        unsafe { LayoutObjectMarkContainerTimingChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3120-3120
    #[inline(never)]
    pub fn MaybeClearIsScrollAnchorObject(&mut self) {
        unsafe { LayoutObjectMaybeClearIsScrollAnchorObjectProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3264-3264
    #[inline(never)]
    pub fn SetSVGDescendantMayHaveTransformRelatedOperations(&mut self) {
        unsafe { LayoutObjectSetSVGDescendantMayHaveTransformRelatedOperationsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3341-3341
    #[inline(never)]
    pub fn UpdateAfterReinsert(&mut self, old_style: &ComputedStyle) {
        unsafe { LayoutObjectUpdateAfterReinsertProvider(self, old_style) }
    }

    // cpp: layoutng/internal/layout_object.h:3579-3579
    #[inline(never)]
    pub(super) fn MarkSelfPaintingLayerForVisualOverflowRecalc(&mut self) {
        unsafe { LayoutObjectMarkSelfPaintingLayerForVisualOverflowRecalcProvider(self) }
    }
}

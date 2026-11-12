//! LayoutObject prepaint, selection, and interaction invalidation.

use super::*;

impl LayoutObject {
    fn FindReferencingScrollAnchors(&mut self, clear: bool) -> bool {
        let mut layer = if self.Parent().is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*self.Parent() }.EnclosingLayer()
        };
        let mut found = false;
        while !layer.is_null() {
            let area = unsafe { &*layer }.GetLayoutObject().GetScrollableArea();
            if !area.is_null() && unsafe { &*area }.ScrollAnchorRefersTo(self) {
                found = true;
                if clear {
                    unsafe { &mut *area }.NotifyScrollAnchorObjectRemoved(self);
                } else {
                    return true;
                }
            }
            layer = unsafe { &*layer }.Parent();
        }
        found
    }

    pub(crate) fn ClearScrollAnchorReferencesBeforeRemoval(&mut self) {
        if !self.bitfields_.is_scroll_anchor_object_ {
            return;
        }
        self.bitfields_.is_scroll_anchor_object_ = false;
        self.FindReferencingScrollAnchors(true);
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectMaybeClearIsScrollAnchorObjectProvider(object: *mut LayoutObject) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    if object.bitfields_.is_scroll_anchor_object_ {
        object.bitfields_.is_scroll_anchor_object_ = object.FindReferencingScrollAnchors(false);
    }
}

// cpp: layoutng/internal/layout_object.cc:5352-5365
impl LayoutObject {
    pub(crate) fn EnsureIsReadyForPaintInvalidationBase(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.NeedsLayout() || self.ChildLayoutBlockedByDisplayLock());

        if self.bitfields_.outline_may_be_affected_by_descendants_
            || self
                .bitfields_
                .previous_outline_may_be_affected_by_descendants_
        {
            self.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                PaintInvalidationReason::kOutline,
            );
        }
        self.bitfields_
            .previous_outline_may_be_affected_by_descendants_ =
            self.bitfields_.outline_may_be_affected_by_descendants_;
    }
}

// The C++ call is virtual.  Preserve that dispatch before entering the exact
// base body above or LayoutBox's background-obscuration work would be skipped.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectEnsureIsReadyForPaintInvalidationProvider(
    object: *mut LayoutObject,
) {
    let object_ref = unsafe { &mut *object };
    if object_ref.IsBox() {
        unsafe { &mut *object.cast::<LayoutBox>() }.EnsureIsReadyForPaintInvalidation();
    } else {
        object_ref.EnsureIsReadyForPaintInvalidationBase();
    }
}

// cpp: layoutng/internal/layout_object.h:2802-2802
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectSetMayNeedPaintInvalidationAnimatedBackgroundImageProvider(
    object: *mut LayoutObject,
) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    object
        .bitfields_
        .may_need_paint_invalidation_animated_background_image_ = true;
}

// cpp: layoutng/internal/layout_object.h:2821-2821
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectClearShouldDelayFullPaintInvalidationProvider(
    object: *mut LayoutObject,
) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    object.bitfields_.should_delay_full_paint_invalidation_ = false;
}

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
    // cpp: third_party/blink/renderer/core/layout/layout_object.cc:5290-5308
    pub fn SetShouldDelayFullPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(IsFullPaintInvalidationReason(
            self.PaintInvalidationReasonForPrePaint()
        ));
        if self.bitfields_.subtree_should_do_full_paint_invalidation_ {
            return;
        }

        self.bitfields_.should_delay_full_paint_invalidation_ = true;
        if !self.ShouldCheckForPaintInvalidation() {
            self.SetShouldCheckForPaintInvalidationWithoutLayoutChange();
        }
        // Chromium's FrameView schedules the following document cycle here.
        // Rechrom's package boundary carries the retained delayed bit through
        // PaintInvalidator and client_commit; that commit re-arms this
        // object after ClearPaintFlags(), which is the host scheduling signal.
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

    // Source: core/layout/layout_object.cc:5659-5671.
    #[inline(never)]
    pub(crate) fn MarkSelfPaintingLayerForVisualOverflowRecalc(&mut self) {
        self.CheckIsNotDestroyed();
        if !self.HasLayer() {
            return;
        }
        let box_model = DynamicTo::<LayoutBoxModelObject>(self);
        debug_assert!(!box_model.is_null());
        let box_model = unsafe { &mut *box_model };
        if box_model.HasSelfPaintingLayer() {
            let layer = box_model.Layer();
            debug_assert!(!layer.is_null());
            unsafe { &mut *layer }.SetNeedsVisualOverflowRecalc();
        }
    }
}

// Chromium layout_object.cc: SetShouldInvalidateSelection. Keep this body in
// the invalidation owner even though LayoutSelection calls it through the
// native LayoutObject API. The package split previously left only the extern
// declaration, so changing focus jumped through an unresolved provider.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectSetShouldInvalidateSelectionProvider(object: *mut LayoutObject) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    object.bitfields_.should_invalidate_selection_ = true;
    object.SetShouldCheckForPaintInvalidation();
    if object.IsText() {
        let selection = object.StyleRef().HighlightData().Selection();
        if unsafe { selection.as_ref() }.is_some_and(|style| {
            style.HasAppliedTextDecorations() || style.HasVisualOverflowingEffect()
        }) {
            object.InvalidateVisualOverflow();
        }
    }
}

// Chromium layout_object.cc:5218-5227.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectSetShouldInvalidatePaintForHitTestProvider(
    object: *mut LayoutObject,
) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    if (object.PaintInvalidationReasonForPrePaint() as u8) < PaintInvalidationReason::kHitTest as u8
    {
        object.SetShouldCheckForPaintInvalidationWithoutLayoutChange();
        object.bitfields_.paint_invalidation_reason_for_pre_paint_ =
            PaintInvalidationReason::kHitTest as u8;
    }
}

// Chromium layout_object.cc:5468-5512. The flags belong to the original
// LayoutObject and propagate through its live parent chain, stopping at a lock.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectSetNeedsPrePaintSubtreeWalkProvider(
    object: *mut LayoutObject,
    reasons: PrePaintSubtreeWalkReasons,
) {
    let object = unsafe { &mut *object };
    object.CheckIsNotDestroyed();
    assert_ne!(reasons.bits(), 0);
    object.bitfields_.pre_paint_subtree_walk_reasons_ |= reasons.bits();
    if object.ChildPrePaintBlockedByDisplayLock() {
        object.bitfields_.descendant_pre_paint_subtree_walk_reasons_ |= reasons.bits();
        return;
    }
    if let Some(parent) = unsafe { object.Parent().as_mut() } {
        parent.SetDescendantNeedsPrePaintSubtreeWalk(reasons);
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectSetDescendantNeedsPrePaintSubtreeWalkProvider(
    mut object: *mut LayoutObject,
    reasons: PrePaintSubtreeWalkReasons,
) {
    while let Some(current) = unsafe { object.as_mut() } {
        current.CheckIsNotDestroyed();
        if current.GetDescendantPrePaintSubtreeWalkReasons().bits() & reasons.bits()
            == reasons.bits()
        {
            break;
        }
        current
            .bitfields_
            .descendant_pre_paint_subtree_walk_reasons_ |= reasons.bits();
        if current.ChildPrePaintBlockedByDisplayLock() {
            break;
        }
        object = current.Parent();
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectMarkEffectiveAllowedTouchActionChangedProvider(
    object: *mut LayoutObject,
) {
    unsafe { &mut *object }.SetNeedsPrePaintSubtreeWalk(
        PrePaintSubtreeWalkReasons::from_reason(
            super::super::pre_paint_subtree_walk_reasons::PrePaintSubtreeWalkReason::kEffectiveAllowedTouchAction,
        ),
    );
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectMarkBlockingWheelEventHandlerChangedProvider(
    object: *mut LayoutObject,
) {
    unsafe { &mut *object }.SetNeedsPrePaintSubtreeWalk(
        PrePaintSubtreeWalkReasons::from_reason(
            super::super::pre_paint_subtree_walk_reasons::PrePaintSubtreeWalkReason::kBlockingWheelEventHandler,
        ),
    );
}

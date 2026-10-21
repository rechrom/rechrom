#![allow(non_snake_case)]

use foundation::{
    IsFullPaintInvalidationReason, IsLayoutFullPaintInvalidationReason,
    IsNonLayoutFullPaintInvalidationReason, PaintInvalidationReason,
};

use super::layout_object::LayoutObject;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2756-2757
    pub fn SetShouldDoFullPaintInvalidation(&mut self) {
        self.SetShouldDoFullPaintInvalidationWithReason(PaintInvalidationReason::kLayout);
    }

    // cpp: layoutng/internal/layout_geometry_invalidation.cc:36-42
    pub fn SetShouldDoFullPaintInvalidationWithReason(&mut self, reason: PaintInvalidationReason) {
        self.CheckIsNotDestroyed();
        debug_assert!(IsLayoutFullPaintInvalidationReason(reason));
        self.SetShouldCheckForPaintInvalidation();
        self.SetShouldDoFullPaintInvalidationWithoutLayoutChangeInternal(reason);
    }

    // cpp: layoutng/internal/layout_geometry_invalidation.cc:44-52
    pub fn SetShouldDoFullPaintInvalidationWithoutLayoutChange(
        &mut self,
        reason: PaintInvalidationReason,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert!(IsNonLayoutFullPaintInvalidationReason(reason));
        debug_assert_ne!(reason, PaintInvalidationReason::kBackground);
        self.SetShouldDoFullPaintInvalidationWithoutLayoutChangeInternal(reason);
    }

    // The pending LayoutObject record must expose these dirty bits to this
    // sibling implementation; they are its original C++ member fields.
    // cpp: layoutng/internal/layout_geometry_invalidation.cc:54-70
    pub(crate) fn SetShouldDoFullPaintInvalidationWithoutLayoutChangeInternal(
        &mut self,
        reason: PaintInvalidationReason,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert!(IsFullPaintInvalidationReason(reason));
        let was_delayed = self.bitfields_.should_delay_full_paint_invalidation_;
        self.bitfields_.should_delay_full_paint_invalidation_ = false;
        let should_upgrade_reason =
            (reason as u8) > (self.PaintInvalidationReasonForPrePaint() as u8);
        if was_delayed || should_upgrade_reason {
            self.SetShouldCheckForPaintInvalidationWithoutLayoutChange();
        }
        if should_upgrade_reason {
            self.bitfields_.paint_invalidation_reason_for_pre_paint_ = reason as u8;
            debug_assert_eq!(reason, self.PaintInvalidationReasonForPrePaint());
        }
    }

    // cpp: layoutng/internal/layout_geometry_invalidation.cc:72-94
    pub fn SetShouldCheckForPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        if self.ShouldCheckLayoutForPaintInvalidation() {
            debug_assert!(self.ShouldCheckForPaintInvalidation());
            return;
        }
        self.bitfields_.should_check_for_paint_invalidation_ = true;
        self.bitfields_.should_check_layout_for_paint_invalidation_ = true;
        let mut ancestor = self.Parent();
        while !ancestor.is_null()
            && !unsafe { &*ancestor }.DescendantShouldCheckLayoutForPaintInvalidation()
        {
            let ancestor_ref = unsafe { &mut *ancestor };
            ancestor_ref.bitfields_.should_check_for_paint_invalidation_ = true;
            ancestor_ref
                .bitfields_
                .descendant_should_check_layout_for_paint_invalidation_ = true;
            ancestor = ancestor_ref.Parent();
        }
    }

    // cpp: layoutng/internal/layout_geometry_invalidation.cc:96-108
    pub fn SetShouldCheckForPaintInvalidationWithoutLayoutChange(&mut self) {
        self.CheckIsNotDestroyed();
        if self.ShouldCheckForPaintInvalidation() {
            return;
        }
        self.bitfields_.should_check_for_paint_invalidation_ = true;
        let mut ancestor = self.Parent();
        while !ancestor.is_null() && !unsafe { &*ancestor }.ShouldCheckForPaintInvalidation() {
            let ancestor_ref = unsafe { &mut *ancestor };
            ancestor_ref.bitfields_.should_check_for_paint_invalidation_ = true;
            ancestor = ancestor_ref.Parent();
        }
    }

    // cpp: layoutng/internal/layout_geometry_invalidation.cc:110-118
    pub fn SetSubtreeShouldCheckForPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        if self.SubtreeShouldCheckForPaintInvalidation() {
            debug_assert!(self.ShouldCheckForPaintInvalidation());
            return;
        }
        self.SetShouldCheckForPaintInvalidation();
        self.bitfields_.subtree_should_check_for_paint_invalidation_ = true;
    }

    // cpp: layoutng/internal/layout_geometry_invalidation.cc:120-125
    pub fn SetSubtreeShouldDoFullPaintInvalidation(&mut self, reason: PaintInvalidationReason) {
        self.CheckIsNotDestroyed();
        self.SetShouldDoFullPaintInvalidationWithReason(reason);
        self.bitfields_.subtree_should_do_full_paint_invalidation_ = true;
    }
}

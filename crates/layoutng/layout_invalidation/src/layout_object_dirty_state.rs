#![allow(non_snake_case)]

use std::ffi::c_char;

use layoutng_style::style::computed_style::ComputedStyle;

use super::super::layout_object::{LayoutObject, MarkingBehavior};

impl LayoutObject {
    // cpp: layoutng/internal/layout_object_inlines.h:12-15
    pub fn StyleRefForLine(&self, first_line: bool) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        if first_line {
            self.FirstLineStyleRef()
        } else {
            self.StyleRef()
        }
    }

    // The source's default MarkingBehavior argument is represented by the
    // one-argument wrapper; explicit marking uses the second method.
    // cpp: layoutng/internal/layout_object_inlines.h:20-38
    pub fn SetNeedsLayout(&mut self, reason: *const c_char) {
        self.SetNeedsLayoutWithMarking(reason, MarkingBehavior::kMarkContainerChain);
    }

    pub fn SetNeedsLayoutWithMarking(
        &mut self,
        _reason: *const c_char,
        mark_parents: MarkingBehavior,
    ) {
        self.CheckIsNotDestroyed();
        #[cfg(debug_assertions)]
        debug_assert!(!self.IsSetNeedsLayoutForbidden());
        let already_needed_layout = self.SelfNeedsFullLayout();
        self.SetSelfNeedsFullLayout(true);
        self.SetNeedsOverflowRecalc();
        self.SetSubgridMinMaxSizesCacheDirty(true);
        self.SetTableColumnConstraintsDirty(true);
        if !already_needed_layout && mark_parents == MarkingBehavior::kMarkContainerChain {
            self.MarkContainerChainForLayout();
        }
    }

    // cpp: layoutng/internal/layout_object_inlines.h:40-46
    pub fn SetNeedsLayoutAndFullPaintInvalidation(&mut self, reason: *const c_char) {
        self.SetNeedsLayoutAndFullPaintInvalidationWithMarking(
            reason,
            MarkingBehavior::kMarkContainerChain,
        );
    }

    pub fn SetNeedsLayoutAndFullPaintInvalidationWithMarking(
        &mut self,
        reason: *const c_char,
        mark_parents: MarkingBehavior,
    ) {
        self.CheckIsNotDestroyed();
        self.SetNeedsLayoutWithMarking(reason, mark_parents);
        self.SetShouldDoFullPaintInvalidation();
    }

    // cpp: layoutng/internal/layout_object_inlines.h:48-53
    pub fn SetNeedsLayoutAndIntrinsicWidthsRecalc(&mut self, reason: *const c_char) {
        self.CheckIsNotDestroyed();
        self.SetNeedsLayout(reason);
        self.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkContainerChain);
    }

    // cpp: layoutng/internal/layout_object_inlines.h:55-61
    pub fn SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
        &mut self,
        reason: *const c_char,
    ) {
        self.CheckIsNotDestroyed();
        self.SetNeedsLayoutAndFullPaintInvalidation(reason);
        self.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkContainerChain);
    }

    // cpp: layoutng/internal/layout_object_hot.cc:145-155
    pub fn ClearNeedsLayoutWithoutPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetEverHadLayout();
        self.SetSelfNeedsFullLayout(false);
        if !self.ChildLayoutBlockedByDisplayLock() {
            self.SetChildNeedsFullLayout(false);
            self.SetNeedsSimplifiedLayout(false);
        }
        self.SetScrollAnchorDisablingStyleChanged(false);
        self.SetShouldSkipLayoutCache(false);
    }

    // cpp: layoutng/internal/layout_object_hot.cc:157-160
    pub fn ClearNeedsLayout(&mut self) {
        self.CheckIsNotDestroyed();
        self.ClearNeedsLayoutWithoutPaintInvalidation();
        self.SetShouldCheckForPaintInvalidation();
    }

    // cpp: layoutng/internal/layout_object_hot.cc:162-165
    pub fn ClearNeedsLayoutWithFullPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        self.ClearNeedsLayoutWithoutPaintInvalidation();
        self.SetShouldDoFullPaintInvalidation();
    }
}

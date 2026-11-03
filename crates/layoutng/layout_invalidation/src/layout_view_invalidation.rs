//! LayoutView style, resize, and explicit invalidation entry points.
//!
//! This module is loaded by the reciprocal LayoutNG assembly. Keeping these
//! entry points here prevents invalidation propagation from accumulating as
//! generic LayoutView self time in sampling profiles.

use super::*;

// LayoutResult resize tracking is a compact set insertion, so it stays
// inlineable rather than forcing a frame on every fragment result.
impl LayoutView {
    // cpp: layoutng/internal/layout_view_data.cc:100-112
    pub fn AffectedByResizedInitialContainingBlock(&mut self, result: &LayoutResult) -> bool {
        self.CheckIsNotDestroyed();
        let handled = self.initial_containing_block_resize_handled_list_.Get();
        if handled.is_null() {
            return false;
        }
        let object = result.GetPhysicalFragment().GetLayoutObject();
        debug_assert!(!object.is_null());
        unsafe { &mut *handled }.insert(Member::from_ptr(object as *mut LayoutObject))
    }

    // cpp: layoutng/internal/layout_view_data.cc:128-136
    #[inline(never)]
    pub fn SetContainsAnnotations(&mut self) {
        self.CheckIsNotDestroyed();
        if self.contains_annotations_ {
            return;
        }
        self.contains_annotations_ = true;
        self.SetNeedsLayout(&raw const layout_invalidation_reason::kStyleChange);
    }

    // cpp: layoutng/internal/layout_view_layout.cc:204-207
    #[inline(never)]
    pub fn UpdateFromStyle(&mut self) {
        self.CheckIsNotDestroyed();
        self.base_.UpdateFromStyle();
    }

    // cpp: layoutng/internal/layout_view_layout.cc:214-221
    #[inline(never)]
    pub fn StyleDidChange(
        &mut self,
        diff: layoutng_style::style::style_difference::StyleDifference,
        old_style: Option<&ComputedStyle>,
        new_style: &ComputedStyle,
        context: &super::super::layout_object::StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.base_.StyleDidChange(
            diff,
            old_style.map_or(std::ptr::null(), |style| style),
            new_style,
            context,
        );
    }

    // cpp: layoutng/internal/layout_view.h:263-265
    #[inline(never)]
    pub fn InvalidateLayoutForCounterStyleChanges(&mut self) {
        unsafe { LayoutViewInvalidateLayoutForCounterStyleChangesProvider(self) }
    }

    // cpp: layoutng/internal/layout_view.h:288-288
    #[inline(never)]
    pub fn InvalidatePaintForViewAndDescendants(&mut self) {
        unsafe { LayoutViewInvalidatePaintForViewAndDescendantsProvider(self) }
    }
}

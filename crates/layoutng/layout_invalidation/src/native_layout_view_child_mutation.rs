//! LayoutView child admission and structural mutation.

use super::*;

impl LayoutView {
    // cpp: layoutng/internal/layout_view_layout.cc:59-65
    #[inline(never)]
    pub fn AddChild(&mut self, child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        assert_ne!(
            unsafe { &*child }.StyleRef().StyleType(),
            layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdViewTransition
        );
        self.base_.AddChild(child, before_child);
    }

    // cpp: layoutng/internal/layout_view_layout.cc:67-71
    pub fn IsChildAllowed(&self, child: *const LayoutObject, _style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { &*child }.IsBox()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:73-76
    pub fn CanHaveChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
}

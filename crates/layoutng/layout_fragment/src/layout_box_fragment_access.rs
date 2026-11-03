//! Fragment-result and inline-fragment state owned by `layout_fragment`.

use super::*;

impl LayoutBox {
    #[cfg(feature = "expensive_dchecks")]
    pub fn CheckMayHaveFragmentItems(&self) {
        self.CheckIsNotDestroyed();
        if !self.MayHaveFragmentItems() {
            debug_assert!(!self.PhysicalFragments().SlowHasFragmentItems());
        }
    }

    pub fn GetLayoutResult(&self, i: usize) -> *const LayoutResult {
        self.CheckIsNotDestroyed();
        self.layout_results_[i].Get()
    }

    pub fn InvalidateCachedGeometry(&self) {
        self.CheckIsNotDestroyed();
        self.SetHasValidCachedGeometry(false);
    }

    pub fn GetPhysicalFragment(&self, i: usize) -> *const PhysicalBoxFragment {
        self.CheckIsNotDestroyed();
        let result = unsafe { &*self.layout_results_[i].Get() };
        result.GetPhysicalFragment() as *const _ as *const PhysicalBoxFragment
    }

    pub fn PhysicalFragmentCount(&self) -> usize {
        self.CheckIsNotDestroyed();
        self.layout_results_.len()
    }

    pub fn HasInlineFragments(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.first_fragment_item_index_ != 0
    }

    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        self.CheckIsNotDestroyed();
        assert!(self.IsInLayoutNGInlineFormattingContext());
        self.first_fragment_item_index_ = 0;
    }

    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        self.CheckIsNotDestroyed();
        assert!(self.IsInLayoutNGInlineFormattingContext());
        debug_assert_ne!(index, 0);
        self.first_fragment_item_index_ = index;
    }

    pub fn InLayoutNGInlineFormattingContextWillChange(&mut self, _new_value: bool) {
        self.CheckIsNotDestroyed();
        if self.IsInLayoutNGInlineFormattingContext() {
            self.ClearFirstInlineFragmentItemIndex();
        }
    }

    pub fn OutOfFlowInsetsForGetComputedStyle(&self) -> &BoxStrut {
        self.CheckIsNotDestroyed();
        let layout_results = self.GetLayoutResults();
        let first = layout_results
            .iter()
            .next()
            .expect("out-of-flow insets require a completed layout result");
        #[cfg(feature = "expensive_dchecks")]
        {
            let mut previous = unsafe { &*first.Get() };
            for result in layout_results.iter().skip(1) {
                let current = unsafe { &*result.Get() };
                debug_assert_eq!(
                    current.OutOfFlowInsetsForGetComputedStyle(),
                    previous.OutOfFlowInsetsForGetComputedStyle(),
                );
                previous = current;
            }
        }
        unsafe { &*first.Get() }.OutOfFlowInsetsForGetComputedStyle()
    }

    pub fn NeedsAnchorPositionScrollAdjustmentInX(&self) -> bool {
        self.CheckIsNotDestroyed();
        let layout_results = self.GetLayoutResults();
        let Some(first) = layout_results.iter().next() else {
            return false;
        };
        #[cfg(feature = "expensive_dchecks")]
        {
            let mut previous = unsafe { &*first.Get() };
            for result in layout_results.iter().skip(1) {
                let current = unsafe { &*result.Get() };
                debug_assert_eq!(
                    current.NeedsAnchorPositionScrollAdjustmentInX(),
                    previous.NeedsAnchorPositionScrollAdjustmentInX(),
                );
                previous = current;
            }
        }
        unsafe { &*first.Get() }.NeedsAnchorPositionScrollAdjustmentInX()
    }

    pub fn NeedsAnchorPositionScrollAdjustmentInY(&self) -> bool {
        self.CheckIsNotDestroyed();
        let layout_results = self.GetLayoutResults();
        let Some(first) = layout_results.iter().next() else {
            return false;
        };
        #[cfg(feature = "expensive_dchecks")]
        {
            let mut previous = unsafe { &*first.Get() };
            for result in layout_results.iter().skip(1) {
                let current = unsafe { &*result.Get() };
                debug_assert_eq!(
                    current.NeedsAnchorPositionScrollAdjustmentInY(),
                    previous.NeedsAnchorPositionScrollAdjustmentInY(),
                );
                previous = current;
            }
        }
        unsafe { &*first.Get() }.NeedsAnchorPositionScrollAdjustmentInY()
    }

    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        if !self.IsInLayoutNGInlineFormattingContext() {
            return 0;
        }
        self.first_fragment_item_index_
    }
}

//! Fragment-result and inline-fragment state owned by `layout_fragment`.

use super::*;

impl LayoutBox {
    // cpp: layoutng/internal/layout_box_fragment_data.cc:399-405
    #[cfg(feature = "expensive_dchecks")]
    pub fn CheckMayHaveFragmentItems(&self) {
        self.CheckIsNotDestroyed();
        if !self.MayHaveFragmentItems() {
            debug_assert!(!self.PhysicalFragments().SlowHasFragmentItems());
        }
    }

    // cpp: layoutng/internal/layout_box.h:606-606
    // cpp: layoutng/internal/layout_box_fragment_data.cc:462-465
    pub fn GetLayoutResult(&self, i: usize) -> *const LayoutResult {
        self.CheckIsNotDestroyed();
        self.layout_results_[i].Get()
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:407-410
    pub fn InvalidateCachedGeometry(&self) {
        self.CheckIsNotDestroyed();
        self.SetHasValidCachedGeometry(false);
    }

    // cpp: layoutng/internal/layout_box.h:607-611
    // cpp: layoutng/internal/layout_box_hot.cc:464-467
    pub fn GetPhysicalFragment(&self, i: usize) -> *const PhysicalBoxFragment {
        self.CheckIsNotDestroyed();
        let result = unsafe { &*self.layout_results_[i].Get() };
        result.GetPhysicalFragment() as *const _ as *const PhysicalBoxFragment
    }

    // cpp: layoutng/internal/layout_box.h:614-617
    pub fn PhysicalFragmentCount(&self) -> usize {
        self.CheckIsNotDestroyed();
        self.layout_results_.len()
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:133-136
    pub fn HasInlineFragments(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.first_fragment_item_index_ != 0
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:138-142
    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        self.CheckIsNotDestroyed();
        assert!(self.IsInLayoutNGInlineFormattingContext());
        self.first_fragment_item_index_ = 0;
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:144-149
    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        self.CheckIsNotDestroyed();
        assert!(self.IsInLayoutNGInlineFormattingContext());
        debug_assert_ne!(index, 0);
        self.first_fragment_item_index_ = index;
    }

    // cpp: layoutng/internal/layout_box_fragment_data.cc:151-155
    pub fn InLayoutNGInlineFormattingContextWillChange(&mut self, _new_value: bool) {
        self.CheckIsNotDestroyed();
        if self.IsInLayoutNGInlineFormattingContext() {
            self.ClearFirstInlineFragmentItemIndex();
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:676-689
    // cpp: layoutng/internal/layout_box_geometry.cc:691-704
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

    // cpp: layoutng/internal/layout_box_geometry.cc:676-689
    // cpp: layoutng/internal/layout_box_geometry.cc:740-754
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

    // cpp: layoutng/internal/layout_box_geometry.cc:676-689
    // cpp: layoutng/internal/layout_box_geometry.cc:756-770
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

    // cpp: layoutng/internal/layout_box.h:1363-1368
    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        if !self.IsInLayoutNGInlineFormattingContext() {
            return 0;
        }
        self.first_fragment_item_index_
    }
}

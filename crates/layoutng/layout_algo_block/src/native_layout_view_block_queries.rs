//! LayoutView root-block and paginated-layout queries.
//!
//! Page objects are block-flow objects and pagination is entered from the root
//! block algorithm. Keeping these entry points here prevents paged-layout work
//! from being attributed to the generic LayoutView record.

use super::*;

impl LayoutView {
    // cpp: layoutng/internal/layout_view_layout.cc:107-111
    pub fn RootBox(&self) -> &LayoutBox {
        self.CheckIsNotDestroyed();
        let child = DynamicTo::<LayoutBox>(self.FirstChild());
        if child.is_null() {
            unsafe { &*(self as *const LayoutView as *const LayoutBox) }
        } else {
            unsafe { &*child }
        }
    }

    // cpp: layoutng/internal/layout_view_layout.cc:223-226
    pub fn IsFragmentationContextRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputPaginated()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:228-231
    pub fn ShouldUsePaginatedLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputPaginated()
    }

    // cpp: layoutng/internal/layout_view.h:77-88
    #[inline(never)]
    pub fn CreateAnonymousPageLayoutObject(
        &mut self,
        style: &ComputedStyle,
    ) -> *mut LayoutBlockFlow {
        unsafe { LayoutViewCreateAnonymousPageLayoutObjectProvider(self, style) }
    }

    // cpp: layoutng/internal/layout_view.h:205-205
    #[inline(never)]
    pub fn NamedPageAtIndex(&self, page_index: usize) -> AtomicString {
        unsafe { LayoutViewNamedPageAtIndexProvider(self, page_index) }
    }
}

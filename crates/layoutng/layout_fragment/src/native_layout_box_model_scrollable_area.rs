//! LayoutBoxModelObject scrollable-area ownership at the assembly boundary.

use super::{
    DynamicTo, LayoutBox, LayoutBoxModelObject, LayoutObject, MakeGarbageCollected, Member,
    PaintLayerScrollableArea,
};

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_scrollable_area.cc:311-336
    #[inline(never)]
    pub fn UpdateScrollableAreaForLayout(&mut self) {
        self.CheckIsNotDestroyed();
        let requires_scrollable_area = {
            let box_ =
                DynamicTo::<LayoutBox>(self as *mut LayoutBoxModelObject as *mut LayoutObject);
            if box_.is_null() {
                false
            } else {
                let box_ = unsafe { &*box_ };
                box_.IsScrollContainer() || box_.IsOverscrollContainer() || box_.CanResize()
            }
        };
        if requires_scrollable_area == !self.scrollable_area_.Get().is_null() {
            return;
        }
        if self.scrollable_area_.Get().is_null() {
            let box_ =
                DynamicTo::<LayoutBox>(self as *mut LayoutBoxModelObject as *mut LayoutObject);
            assert!(!box_.is_null());
            self.scrollable_area_ = Member::from_ptr(MakeGarbageCollected(
                PaintLayerScrollableArea::new(unsafe { &mut *box_ }),
            ));
        } else {
            unsafe { &mut *self.scrollable_area_.Get() }.Dispose();
            self.scrollable_area_ = Member::default();
        }
    }

    // cpp: layoutng/internal/layout_scrollable_area.cc:338-344
    #[inline(never)]
    pub fn DisposeScrollableAreaForLayout(&mut self) {
        self.CheckIsNotDestroyed();
        let area = self.scrollable_area_.Get();
        if !area.is_null() {
            unsafe { &mut *area }.Dispose();
            self.scrollable_area_ = Member::default();
        }
    }
}

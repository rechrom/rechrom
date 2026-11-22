//! LayoutBoxModelObject scrollable-area ownership at the assembly boundary.

use super::{
    DynamicTo, LayoutBox, LayoutBoxModelObject, LayoutObject, MakeGarbageCollected, Member,
    PaintLayerScrollableArea,
};

impl LayoutBoxModelObject {
    // PaintLayer owns admission in Chromium. Storage remains on the existing
    // LayoutBoxModelObject side of Rechrom's Layout/Paint package boundary.
    // Source: core/paint/paint_layer.cc:905-907.
    #[inline(never)]
    pub(crate) fn CreateScrollableAreaForPaintLayer(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.scrollable_area_.Get().is_null());
        let box_ = DynamicTo::<LayoutBox>(self as *mut LayoutBoxModelObject as *mut LayoutObject);
        assert!(!box_.is_null());
        let area = MakeGarbageCollected(PaintLayerScrollableArea::new(unsafe { &mut *box_ }));
        unsafe { &mut *area }.InitializeScrollingBackgroundDisplayItemClient();
        unsafe { &mut *area }.RegisterWithFrameView();
        self.scrollable_area_ = Member::from_ptr(area);
    }

    // Package-boundary half of PaintLayer::UpdateScrollableArea()/Destroy().
    // Source: core/paint/paint_layer.cc:917-919,225-257.
    #[inline(never)]
    pub(crate) fn DisposeScrollableAreaForPaintLayer(&mut self) {
        self.CheckIsNotDestroyed();
        let area = self.scrollable_area_.Get();
        if !area.is_null() {
            unsafe { &mut *area }.Dispose();
            self.scrollable_area_ = Member::default();
        }
    }
}

//! PaintLayer lifecycle caused by LayoutBoxModelObject style invalidation.

use super::{LayoutBoxModelObject, MakeGarbageCollected, PaintLayer, PaintLayerType};

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_box_model_object.h:124-130
    // cpp: core/layout/layout_box_model_object.cc:360-367
    #[inline(never)]
    pub fn DestroyLayer(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.HasLayer() && !self.Layer().is_null());
        self.DisposeScrollableAreaForPaintLayer();
        self.SetHasLayer(false);
        // FragmentData::SetLayer owns the single PaintLayer::Destroy() call.
        self.GetMutableForPainting()
            .FirstFragment()
            .SetLayer(std::ptr::null_mut());
        self.SetNeedsPaintPropertyUpdate();
    }

    // The layer survives style changes whenever the source object continues
    // to require a layer; only a no-layer transition destroys its client.
    // cpp: core/layout/layout_box_model_object.cc:205-223,294-295
    #[inline(never)]
    pub(crate) fn UpdateLayerAfterStyleChange(&mut self) {
        let required = self.LayerTypeRequired();
        let layer = self.Layer();
        if required == PaintLayerType::kNoPaintLayer {
            if !layer.is_null() {
                self.DestroyLayer();
                if self.EverHadLayout() {
                    self.SetChildNeedsLayout();
                }
            }
        } else if layer.is_null() {
            if self.EverHadLayout() {
                self.SetChildNeedsLayout();
            }
            self.CreateLayerAfterStyleChange();
        } else {
            let was_self_painting = unsafe { &*layer }.IsSelfPaintingLayer();
            unsafe { &mut *layer }.UpdateSelfPaintingLayer();
            if was_self_painting != unsafe { &*layer }.IsSelfPaintingLayer() {
                self.SetChildNeedsLayout();
            }
        }
    }

    // cpp: layoutng/internal/layout_box_model_object.h:360-360
    // cpp: core/layout/layout_box_model_object.cc:347-357
    #[inline(never)]
    fn CreateLayerAfterStyleChange(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.HasLayer() && self.Layer().is_null());
        let owner = self as *mut LayoutBoxModelObject;
        let layer = MakeGarbageCollected(PaintLayer::new(owner));
        let mut painting = self.GetMutableForPainting();
        let first = painting.FirstFragment();
        first.EnsureId();
        first.SetLayer(layer);
        self.SetHasLayer(true);
        self.SetNeedsPaintPropertyUpdate();
    }
}

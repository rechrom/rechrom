//! PaintLayer lifecycle caused by LayoutBoxModelObject style invalidation.

use layout_style::style::computed_style::ComputedStyle;

use super::{LayoutBoxModelObject, MakeGarbageCollected, PaintLayer, PaintLayerType};

impl LayoutBoxModelObject {
    // Source: core/layout/layout_box_model_object.cc:360-367. Tree removal
    // (LayoutObject::RemoveLayers from WillBeRemovedFromTree) or
    // PaintLayer::RemoveOnlyThisLayerAfterStyleChange has already invalidated
    // the layer's former relationships.
    #[inline(never)]
    pub fn DestroyLayer(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.HasLayer() && !self.Layer().is_null());
        // Chromium's PaintLayer::Destroy disposes scrollable_area_
        // (paint_layer.cc:251-252); Rechrom's area is owned by this object.
        self.DisposeScrollableAreaForPaintLayer();
        self.SetHasLayer(false);
        // FragmentData::SetLayer owns the single PaintLayer::Destroy() call.
        self.GetMutableForPainting()
            .FirstFragment()
            .SetLayer(std::ptr::null_mut());
        // Removing a layer may affect existence of the
        // LocalBorderBoxProperties, so we need to ensure that we update paint
        // properties.
        self.SetNeedsPaintPropertyUpdate();
    }

    // The layer survives style changes whenever the source object continues
    // to require a layer; only a no-layer transition destroys its client.
    // cpp: core/layout/layout_box_model_object.cc:205-223,294-295
    #[inline(never)]
    pub(crate) fn UpdateLayerAfterStyleChange(&mut self, old_style: *const ComputedStyle) {
        let required = self.LayerTypeRequired();
        let layer = self.Layer();
        if required == PaintLayerType::kNoPaintLayer {
            if !layer.is_null() {
                // Source: core/layout/layout_box_model_object.cc:223-224.
                // Calls DestroyLayer() which clears the layer.
                PaintLayer::RemoveOnlyThisLayerAfterStyleChange(layer, old_style);
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

    // Source: core/layout/layout_box_model_object.cc:347-357.
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
        unsafe { &mut *layer }.InsertOnlyThisLayerAfterStyleChange();
        // Creating a layer may affect existence of the
        // LocalBorderBoxProperties, so we need to ensure that we update paint
        // properties.
        self.SetNeedsPaintPropertyUpdate();
    }
}

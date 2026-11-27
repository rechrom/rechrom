//! PaintLayer lifecycle caused by LayoutBoxModelObject style invalidation.

use super::{LayoutBoxModelObject, MakeGarbageCollected, PaintLayer, PaintLayerType};

fn MarkLayerParentAfterTreeChange(layer: *mut PaintLayer) {
    let parent = unsafe { &*layer }.Parent();
    if parent.is_null() {
        return;
    }
    unsafe { &mut *parent }.MarkAncestorChainForLayerTreeChange();
    if !unsafe { &*layer }.IsSelfPaintingLayer() {
        unsafe { &mut *parent }.DirtyVisibleContentStatus();
    }
}

impl LayoutBoxModelObject {
    // cpp: layoutng/internal/layout_box_model_object.h:124-130
    // cpp: core/layout/layout_box_model_object.cc:360-367
    #[inline(never)]
    pub fn DestroyLayer(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.HasLayer() && !self.Layer().is_null());
        // PaintLayer::RemoveChild marks the former ancestor chain before the
        // child is detached. Rechrom derives layer links from LayoutObject, so
        // perform the same invalidation while that relationship is still live.
        MarkLayerParentAfterTreeChange(self.Layer());
        // Source: core/paint/paint_layer.cc:774-775 (RemoveChild dirties the
        // z-order list in which the layer is contained) and 803-809
        // (RemoveOnlyThisLayerAfterStyleChange hands its paint-phase needs to
        // the enclosing self-painting layer).
        let layer = unsafe { &mut *self.Layer() };
        layer.DirtyStackingContextZOrderLists();
        if layer.IsSelfPaintingLayer() {
            let parent = layer.Parent();
            if !parent.is_null() {
                let enclosing = unsafe { &*parent }.EnclosingSelfPaintingLayer();
                if let Some(enclosing) = unsafe { enclosing.as_ref() } {
                    enclosing.MergeNeedsPaintPhaseFlagsFrom(layer);
                }
            }
        }
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
        // Chromium's InsertOnlyThisLayerAfterStyleChange() reaches AddChild(),
        // which marks descendant-dependent flags on the complete ancestor
        // chain. Our links are derived instead of stored, but the invalidation
        // is still required so the root PrePaint refresh visits this new layer.
        MarkLayerParentAfterTreeChange(layer);
        let layer = unsafe { &mut *layer };
        let parent = layer.Parent();
        if !parent.is_null() {
            // Source: core/paint/paint_layer.cc:731-735. Dirty the z-order
            // list in which we are contained.
            if layer.GetLayoutObject().IsStacked() || !layer.FirstChild().is_null() {
                layer.DirtyStackingContextZOrderLists();
            }
            // Source: core/paint/paint_layer.cc:750-754.
            if layer.NeedsCullRectUpdate() {
                unsafe { &*parent }.SetDescendantNeedsCullRectUpdate();
            } else {
                layer.SetNeedsCullRectUpdate();
            }
            // Source: core/paint/paint_layer.cc:852-857.
            if layer.IsSelfPaintingLayer() {
                let enclosing = unsafe { &*parent }.EnclosingSelfPaintingLayer();
                if let Some(enclosing) = unsafe { enclosing.as_ref() } {
                    layer.MergeNeedsPaintPhaseFlagsFrom(enclosing);
                }
            }
        }
        self.SetNeedsPaintPropertyUpdate();
    }
}

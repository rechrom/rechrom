//! The PaintLayer and LayoutObject facts PrePaint's CullRectUpdater and
//! PaintInvalidator read from the visited object.
//!
//! Source: core/paint/cull_rect_updater.cc:291-390,558-656 (the visited
//! PaintLayer, its cull-rect bits, `object.SlowFirstChild()`,
//! `object.EnclosingLayer()`) and core/paint/paint_invalidator.cc:33-53
//! (`object.HasLayer() && HasSelfPaintingLayer()`, `IsInlineRubyText()`,
//! `PaintingLayer()`, `IsFloating()`, `StyleRef().HasOutline()` and the
//! painting layer's own LayoutObject). Chromium's core/paint reads these on
//! the LayoutObject directly; the cursor lends them for the visited
//! fragment's source object. The writable PaintLayer bits are
//! sequence-confined interior state, so shared borrows suffice.
use crate::internal::{
    layout_box_model_object::LayoutBoxModelObject, layout_object::LayoutObject,
    paint_layer::PaintLayer,
};
use foundation::DynamicTo;

impl<'a> super::PaintFragmentCursor<'a> {
    fn PrePaintSourceLayoutObject(self) -> Option<&'a LayoutObject> {
        unsafe { self.source.GetLayoutObject().as_ref() }
    }

    /// `To<LayoutBoxModelObject>(object).Layer()` of the source object, None
    /// when it has no PaintLayer (layout_box_model_object.h Layer()).
    pub fn OwnPaintLayer(self) -> Option<&'a PaintLayer> {
        let object = self.PrePaintSourceLayoutObject()?;
        if !object.HasLayer() {
            return None;
        }
        let box_model = unsafe { DynamicTo::<LayoutBoxModelObject>(object).as_ref() }?;
        unsafe { box_model.Layer().as_ref() }
    }

    /// `object.HasLayer() && HasSelfPaintingLayer()` of the source object
    /// (paint_invalidator.cc:35-36).
    pub fn SourceHasSelfPaintingLayer(self) -> bool {
        self.OwnPaintLayer()
            .is_some_and(PaintLayer::IsSelfPaintingLayer)
    }

    /// LayoutObject::PaintingLayer() (layout_object.cc:1342-1370).
    pub fn PaintingLayer(self) -> Option<&'a PaintLayer> {
        unsafe { self.PrePaintSourceLayoutObject()?.PaintingLayerDefault().as_ref() }
    }

    /// LayoutObject::EnclosingLayer() (layout_object.cc).
    pub fn EnclosingPaintLayer(self) -> Option<&'a PaintLayer> {
        unsafe { self.PrePaintSourceLayoutObject()?.EnclosingLayer().as_ref() }
    }

    /// LayoutObject::IsInlineRubyText() (paint_invalidator.cc:38).
    pub fn SourceIsInlineRubyText(self) -> bool {
        self.PrePaintSourceLayoutObject()
            .is_some_and(LayoutObject::IsInlineRubyText)
    }

    /// LayoutObject::IsFloating() (paint_invalidator.cc:44).
    pub fn SourceIsFloating(self) -> bool {
        self.PrePaintSourceLayoutObject()
            .is_some_and(LayoutObject::IsFloating)
    }

    /// `object.StyleRef().HasOutline()` (paint_invalidator.cc:50).
    pub fn SourceHasOutline(self) -> bool {
        self.PrePaintSourceLayoutObject()
            .is_some_and(|object| object.StyleRef().HasOutline())
    }

    /// `object.SlowFirstChild()` is non-null (cull_rect_updater.cc:649).
    pub fn SourceHasChildren(self) -> bool {
        self.PrePaintSourceLayoutObject()
            .is_some_and(|object| !object.SlowFirstChild().is_null())
    }

    /// `object == layer.GetLayoutObject()` (paint_invalidator.cc:49).
    pub fn IsLayerObject(self, layer: &PaintLayer) -> bool {
        self.PrePaintSourceLayoutObject().is_some_and(|object| {
            std::ptr::eq(
                layer.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject,
                object,
            )
        })
    }
}

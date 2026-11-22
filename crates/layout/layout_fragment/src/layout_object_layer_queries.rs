//! LayoutObject paint-layer and visual-effect queries.

use super::*;

impl LayoutObject {
    // Source: core/layout/layout_object.cc:2316-2361. The source algorithm
    // requires FragmentData::LocalBorderBoxProperties() plus the local-frame
    // root property state, neither of which exists before Rechrom's immutable
    // Layout->Paint export. Do not infer these effects from style.
    pub fn HasDistortingVisualEffects(&self) -> bool {
        self.CheckIsNotDestroyed();
        std::panic::panic_any(foundation::UnsupportedLayout::new(
            "LayoutObject::HasDistortingVisualEffects requires source LocalBorderBoxProperties",
        ))
    }

    // Source: core/layout/layout_object.cc:2363-2385. Effective opacity is an
    // ancestor EffectPaintPropertyNode query, not the local computed opacity.
    pub fn HasNonZeroEffectiveOpacity(&self) -> bool {
        self.CheckIsNotDestroyed();
        std::panic::panic_any(foundation::UnsupportedLayout::new(
            "LayoutObject::HasNonZeroEffectiveOpacity requires source LocalBorderBoxProperties",
        ))
    }

    // Source: core/layout/layout_object.cc:1329-1340.
    #[inline(never)]
    pub fn EnclosingLayer(&self) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        let mut current = self as *const LayoutObject;
        while !current.is_null() {
            let current_ref = unsafe { &*current };
            if current_ref.HasLayer() {
                let box_model = DynamicTo::<LayoutBoxModelObject>(current);
                debug_assert!(!box_model.is_null());
                return unsafe { &*box_model }.Layer();
            }
            current = current_ref.Parent();
        }
        std::ptr::null_mut()
    }

    // Source: core/layout/layout_object.cc:1342-1370 and
    // core/layout/layout_object.h:468-470. Cross-frame OwnerLayoutObject and
    // anonymous pagination page-box fallback remain explicitly unsupported.
    #[inline(never)]
    pub fn PaintingLayer(&self, max_depth: i32) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        let mut depth = 0;
        let mut current = self as *const LayoutObject as *mut LayoutObject;
        while !current.is_null() {
            if max_depth != -1 {
                depth += 1;
                if depth > max_depth {
                    return std::ptr::null_mut();
                }
            }
            let current_ref = unsafe { &*current };
            if current_ref.HasLayer() {
                let box_model = DynamicTo::<LayoutBoxModelObject>(current);
                debug_assert!(!box_model.is_null());
                let layer = unsafe { &*box_model }.Layer();
                if !layer.is_null() && unsafe { &*layer }.IsSelfPaintingLayer() {
                    return layer;
                }
            }
            current = if current_ref.IsColumnSpanAll() {
                current_ref.ContainerForColumnSpanner()
            } else if current_ref.IsInlineRubyText() {
                current_ref.ContainingBlock() as *mut LayoutObject
            } else if current_ref.IsLayoutView() {
                std::ptr::null_mut()
            } else {
                current_ref.Parent()
            };
        }
        std::ptr::null_mut()
    }

    // Source: core/layout/layout_object.h:468-470.
    pub fn PaintingLayerDefault(&self) -> *mut PaintLayer {
        self.PaintingLayer(-1)
    }
}

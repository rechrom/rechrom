//! LayoutObject paint-layer and visual-effect queries.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:334-334
    pub fn HasDistortingVisualEffects(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectHasDistortingVisualEffectsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:337-337
    pub fn HasNonZeroEffectiveOpacity(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectHasNonZeroEffectiveOpacityProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:448-448
    #[inline(never)]
    pub fn EnclosingLayer(&self) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectEnclosingLayerProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:458-458
    #[inline(never)]
    pub fn PaintingLayer(&self, max_depth: i32) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectPaintingLayerProvider(self, max_depth) }
    }

    // cpp: layoutng/internal/layout_object.h:458-458
    pub fn PaintingLayerDefault(&self) -> *mut PaintLayer {
        self.PaintingLayer(-1)
    }
}

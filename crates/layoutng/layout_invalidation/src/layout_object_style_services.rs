//! LayoutObject element refresh and pseudo-style services.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2155-2155
    pub fn UpdateFromElementBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    #[inline(never)]
    pub fn UpdateFromElement(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectUpdateFromElement(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3491-3491
    pub(crate) fn FirstLineStyleWithoutFallback(&self) -> *const ComputedStyle {
        unsafe { LayoutObjectFirstLineStyleWithoutFallbackProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1671-1671
    pub fn GetCachedPseudoElementStyle(&self, pseudo_id: PseudoId) -> *const ComputedStyle {
        unsafe { LayoutObjectGetCachedPseudoElementStyleProvider(self, pseudo_id) }
    }

    // cpp: layoutng/internal/layout_object.h:1672-1672
    #[inline(never)]
    pub fn GetUncachedPseudoElementStyle(&self, request: &StyleRequest) -> *const ComputedStyle {
        unsafe { LayoutObjectGetUncachedPseudoElementStyleProvider(self, request) }
    }

    // cpp: layoutng/internal/layout_object.h:2218-2219
    #[inline(never)]
    pub fn SetModifiedStyleOutsideStyleRecalc(
        &mut self,
        style: *const ComputedStyle,
        apply_changes: ApplyStyleChanges,
    ) {
        unsafe {
            LayoutObjectSetModifiedStyleOutsideStyleRecalcProvider(self, style, apply_changes)
        }
    }
}

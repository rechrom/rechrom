//! Native password-view invalidation on the retained LayoutObject.
//! Source: core/html/forms/password_input_type.cc:141-145.
use super::*;

impl LayoutEngine {
    pub fn InvalidateInputCapsLockIndicatorPaint(&mut self, id: u64) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(object) = self
            .GetLayoutTree()
            .and_then(|tree| tree.OutputNode(id))
            .map(|object| object as *const LayoutObject as *mut LayoutObject)
        else {
            return false;
        };
        unsafe { &mut *object }.SetShouldDoFullPaintInvalidation();
        self.pending_paint_ids_.insert(id);
        self.RefreshSelectionPaintInputs(&[object]);
        true
    }
}

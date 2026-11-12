//! Typed document-to-native FrameView effect. Pending members and ancestor
//! paint-property invalidation stay inside the owning Layout package.
use crate::{internal::layout_view::LayoutView, layout_engine::LayoutEngine};
use foundation::DynamicTo;
impl LayoutEngine {
    /// DisplayLockContext.cc:545-555 -> LocalFrameView.cc:5713-5726.
    pub fn RemoveAllPendingPaintPropertyUpdates(&mut self) {
        let _scope = self.NativeScrollPassScope();
        let Some(tree) = self.GetLayoutTree() else {
            return;
        };
        let view = DynamicTo::<LayoutView>(tree.Root());
        if let Some(view) = unsafe { view.as_ref() } {
            view.RemoveAllPendingUpdates();
        }
    }
}

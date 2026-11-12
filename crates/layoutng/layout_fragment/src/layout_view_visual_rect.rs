//! Source core/layout/layout_view.cc:425-500. Frame-independent mapping
//! terminates at the real LayoutView. Cross-frame/visual-viewport transforms
//! require their actual frame owner; they cannot be inferred from root style.
use super::*;
use foundation::graphics_types::graphics::visual_rect_flags::VisualRectFlag;
impl LayoutView {
    pub(crate) fn MapVisualRectInternalSource(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        self.CheckIsNotDestroyed();
        let layer = self.Layer();
        if let Some(layer) = unsafe { layer.as_ref() } {
            let object = layer.GetLayoutObject();
            if !object.Transform().is_null() {
                state.ApplyTransform(
                    &object.CurrentTransform(),
                    TransformState::kFlattenTransform,
                );
            }
        }
        state.Flatten();
        if std::ptr::eq(self as *const Self as *const LayoutBoxModelObject, ancestor) {
            return true;
        }
        let viewport = self
            .InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("native frame viewport required");
        if !viewport.is_main_frame {
            std::panic::panic_any(foundation::UnsupportedLayout::new(
                "embedded frame visual-rect mapping requires LocalOwner/frame coordinate producer",
            ));
        }
        if flags.Has(VisualRectFlag::kApplyRemoteViewportTransform) {
            std::panic::panic_any(foundation::UnsupportedLayout::new(
                "visual viewport RootFrameToViewport producer is not installed",
            ));
        }
        // LocalFrameView::MapToVisualRectInRemoteRootFrame, source.cc:5242-
        // 5263: for the outermost root frame this leaves the rect unchanged.
        // Source does not apply a separate ViewRect clip on this branch.
        let rect = PhysicalRect::EnclosingRect(&state.LastPlanarQuad().BoundingBox());
        state.SetQuad(gfx::QuadF::from(gfx::RectF::from(rect)));
        true
    }
}

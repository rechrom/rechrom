//! Source core/layout/layout_inline.cc:779-824. The layout-inline owning
//! class is compiled through its existing native facade, not a second object.
use super::*;
use foundation::DynamicTo;
impl LayoutInline {
    pub(crate) fn MapVisualRectInternalSource(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        self.CheckIsNotDestroyed();
        if std::ptr::eq(&self.model_object_, ancestor) {
            return true;
        }
        let canvas = self.CanvasForDrawingLayoutObject();
        if !canvas.is_null() {
            if self.ShouldUseTransformFromContainer(canvas) {
                let mut transform = gfx::Transform::default();
                self.GetTransformFromContainer(
                    canvas,
                    PhysicalOffset::default(),
                    &mut transform,
                    std::ptr::null(),
                    None,
                );
                state.ApplyTransform(
                    &transform,
                    if self.StyleRef().Preserves3D() {
                        TransformState::kAccumulateTransform
                    } else {
                        TransformState::kFlattenTransform
                    },
                );
            }
            return unsafe { &*canvas }
                .MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags);
        }
        let container = self.Container();
        debug_assert_eq!(container, self.Parent());
        let Some(container_ref) = (unsafe { container.as_ref() }) else {
            return true;
        };
        let accumulation = if container_ref.StyleRef().Preserves3D() {
            TransformState::kAccumulateTransform
        } else {
            TransformState::kFlattenTransform
        };
        if self.IsStickyPositioned() {
            state.Move(self.StickyPositionOffset(), accumulation);
        }
        let box_ = DynamicTo::<LayoutBox>(container);
        if !box_.is_null()
            && container.cast_const() != ancestor.cast()
            && !unsafe { &*box_ }.MapContentsRectToBoxSpace(state, accumulation, self, flags)
        {
            return false;
        }
        container_ref.MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags)
    }
}

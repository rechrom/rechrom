//! Chromium layout_object.cc:2692-2764. Source TransformState slow path;
//! no fragment geometry export, AbsoluteQuads, or parallel coordinate tree.
use super::*;
use crate::internal::{layout_inline::LayoutInline, layout_view::LayoutView};

impl LayoutObject {
    pub(crate) fn MapVisualRectInternalDispatch(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        self.CheckIsNotDestroyed();
        // These are owning-class virtuals. SVG has separate source overrides;
        // it must not silently inherit HTML box geometry through this dispatch.
        if self.IsSVGChild() {
            std::panic::panic_any(foundation::UnsupportedLayout::new(
                "SVG visual-rect mapping override is not installed",
            ));
        }
        let view = DynamicTo::<LayoutView>(self as *const LayoutObject);
        if !view.is_null() {
            return unsafe { &*view }
                .MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags);
        }
        let box_ = DynamicTo::<LayoutBox>(self as *const LayoutObject);
        if !box_.is_null() {
            return unsafe { &*box_ }
                .MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags);
        }
        let inline = DynamicTo::<LayoutInline>(self as *const LayoutObject);
        if !inline.is_null() {
            return unsafe { &*inline }
                .MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags);
        }
        // Source default virtual, mainly LayoutText: incoming rect is already
        // in its parent's physical coordinate space.
        if std::ptr::eq(self, ancestor.cast()) {
            return true;
        }
        let parent = self.Parent();
        let Some(parent) = (unsafe { parent.as_ref() }) else {
            return true;
        };
        if parent.IsBox() {
            let accumulation = if parent.StyleRef().Preserves3D() && !parent.IsText() {
                TransformState::kAccumulateTransform
            } else {
                TransformState::kFlattenTransform
            };
            if !std::ptr::eq(parent, ancestor.cast())
                && !unsafe { &*DynamicTo::<LayoutBox>(parent) }.MapContentsRectToBoxSpace(
                    state,
                    accumulation,
                    self,
                    flags,
                )
            {
                return false;
            }
        }
        parent.MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags)
    }
}

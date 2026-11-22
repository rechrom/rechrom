//! Source core/layout/layout_box.cc:1794-1921,3024-3138 (pinned revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15). Existing native box geometry
//! supplies physical offsets (including writing mode and fragment placement).
use super::*;
use crate::internal::{layout_object::AncestorSkipInfo, layout_view::LayoutView};
impl LayoutBox {
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
        if !flags.Has(VisualRectFlag::kIgnoreFilters) {
            self.InflateVisualRectForFilter(state);
        }
        let canvas = self.CanvasForDrawingLayoutObject();
        if !canvas.is_null() {
            if !self.MapVisualRectToContainer(
                canvas,
                &PhysicalOffset::default(),
                ancestor.cast(),
                flags,
                state,
            ) {
                return false;
            }
            return unsafe { &*canvas }
                .MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags);
        }
        let mut skip = AncestorSkipInfo::new_with_filters(ancestor.cast(), true);
        let container = self.ContainerWithSkipInfo(&mut skip);
        let Some(container_ref) = (unsafe { container.as_ref() }) else {
            return true;
        };
        let mut offset = self.PhysicalLocation();
        if self.IsStickyPositioned() {
            offset += self.StickyPositionOffset();
        } else if self.NeedsAnchorPositionScrollAdjustment() {
            offset += self.AnchorPositionScrollTranslationOffset();
        }
        if skip.FilterSkipped() && !flags.Has(VisualRectFlag::kIgnoreFilters) {
            self.InflateVisualRectForFilterUnderContainer(state, container_ref, ancestor);
        }
        if !self.MapVisualRectToContainer(container, &offset, ancestor.cast(), flags, state) {
            return false;
        }
        if skip.AncestorSkipped() {
            assert!(!ancestor.is_null());
            let accumulation = if container_ref.StyleRef().Preserves3D() {
                TransformState::kAccumulateTransform
            } else {
                TransformState::kFlattenTransform
            };
            state.Move(
                -unsafe { &*ancestor.cast::<LayoutObject>() }.OffsetFromAncestor(container),
                accumulation,
            );
            return true;
        }
        if self.IsFixedPositioned()
            && container.cast_const() == ancestor.cast()
            && container_ref.IsLayoutView()
        {
            state.Move(
                unsafe { &*container.cast::<LayoutView>() }.OffsetForFixedPosition(),
                TransformState::kFlattenTransform,
            );
        }
        container_ref.MapToVisualRectInAncestorSpaceInternal(ancestor, state, flags)
    }
    pub(crate) fn MapVisualRectToContainerSource(
        &self,
        container: *const LayoutObject,
        offset: &PhysicalOffset,
        ancestor: *const LayoutObject,
        flags: VisualRectFlags,
        state: &mut TransformState,
    ) -> bool {
        self.CheckIsNotDestroyed();
        assert!(!container.is_null());
        let container_ref = unsafe { &*container };
        let preserve3d = container_ref.StyleRef().Preserves3D()
            && container == self.NearestAncestorForElement().cast_const();
        let accumulation = if preserve3d {
            TransformState::kAccumulateTransform
        } else {
            TransformState::kFlattenTransform
        };
        let container_box = DynamicTo::<LayoutBox>(container);
        if !self.ShouldUseTransformFromContainer(container) {
            state.Move(*offset, accumulation);
            if !container_box.is_null()
                && container != ancestor
                && !unsafe { &*container_box }.MapContentsRectToBoxSpace(
                    state,
                    accumulation,
                    self,
                    flags,
                )
            {
                return false;
            }
            return true;
        }
        // VisualRectMappingFixForExpansion is stable in pinned JSON5:6725.
        // Do not repeatedly round rotated/skewed rectangles up the chain.
        let mut transform = self.CurrentTransform();
        transform.PostTranslate(offset.left.ToFloat(), offset.top.ToFloat());
        if !container_box.is_null()
            && container != ancestor
            && unsafe { &*container_box }.ContainedContentsScroll(self)
        {
            let scroll = -unsafe { &*container_box }.ScrolledContentOffset();
            transform.PostTranslate(scroll.left.ToFloat(), scroll.top.ToFloat());
        }
        if container_ref.HasPerspective()
            && container == self.NearestAncestorForElement().cast_const()
        {
            let origin = if container_box.is_null() {
                gfx::PointF::default()
            } else {
                unsafe { &*container_box }.PerspectiveOrigin(std::ptr::null())
            };
            let mut perspective = gfx::Transform::default();
            perspective.ApplyPerspectiveDepth(container_ref.StyleRef().UsedPerspective() as f64);
            perspective.ApplyTransformOrigin(origin.x(), origin.y(), 0.0);
            transform = perspective * transform;
        }
        state.ApplyTransform(&transform, accumulation);
        if !preserve3d {
            state.Flatten();
        }
        if !container_box.is_null()
            && container != ancestor
            && container_ref.HasClipRelatedProperty()
        {
            return unsafe { &*container_box }.ApplyBoxClips(state, accumulation, flags);
        }
        true
    }
    pub(crate) fn InflateVisualRectForFilterSource(&self, state: &mut TransformState) {
        self.CheckIsNotDestroyed();
        let layer = self.Layer();
        let Some(layer) = (unsafe { layer.as_ref() }) else {
            return;
        };
        if !layer.PaintsWithFilters() {
            return;
        }
        state.Flatten();
        let rect = PhysicalRect::EnclosingRect(&state.LastPlanarQuad().BoundingBox());
        state.SetQuad(gfx::QuadF::from(gfx::RectF::from(
            layer.MapRectForFilter(&rect),
        )));
    }
    pub(crate) fn InflateVisualRectForFilterUnderContainerSource(
        &self,
        state: &mut TransformState,
        container: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
    ) {
        self.CheckIsNotDestroyed();
        state.Flatten();
        let offset = self.OffsetFromContainerDefault(container);
        state.Move(offset, TransformState::kFlattenTransform);
        let mut parent = self.Parent();
        while !parent.is_null() && !std::ptr::eq(unsafe { &*parent }, container) {
            let parent_ref = unsafe { &*parent };
            let box_ = DynamicTo::<LayoutBox>(parent);
            if !box_.is_null() {
                let offset = parent_ref.OffsetFromAncestor(container);
                state.Move(-offset, TransformState::kFlattenTransform);
                unsafe { &*box_ }.InflateVisualRectForFilter(state);
                state.Move(offset, TransformState::kFlattenTransform);
            }
            if parent.cast_const() == ancestor.cast() {
                break;
            }
            parent = parent_ref.Parent();
        }
        state.Move(-offset, TransformState::kFlattenTransform);
    }
}

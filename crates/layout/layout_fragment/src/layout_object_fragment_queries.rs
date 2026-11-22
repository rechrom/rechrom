//! LayoutObject fragment hit testing and coordinate projection.
//!
//! The concrete virtual dispatch remains assembly supplied. Traversal-capable
//! entry points are deliberately out of line so sampling profiles attribute
//! their inclusive cost to layout_fragment instead of LayoutObject core.

use super::*;

impl LayoutObject {
    #[inline(never)]
    pub fn HitTestForOcclusion(&self, rect: &PhysicalRect) -> HitTestResult {
        unsafe { LayoutObjectHitTestForOcclusionProvider(self, rect) }
    }

    #[inline(never)]
    pub fn HitTestAllPhases(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
    ) -> bool {
        unsafe { LayoutObjectHitTestAllPhasesProvider(self, result, location, accumulated_offset) }
    }

    /// Link generated anonymous content to its canonical DOM PseudoElement.
    /// This is an owner identity, not an additional hit-test tree or geometry.
    pub fn WithHitTestNodeOwner(&mut self, owner: Option<u64>) -> &mut Self {
        if let Some(node) = unsafe { self.node_.Get().as_mut() } {
            node.SetInputHitTestNodeOwner(owner);
        }
        self
    }
    pub fn NodeForHitTest(&self) -> *mut Node {
        // Source: core/layout/layout_object.cc:4547-4590. Generated content
        // resolves through its existing anonymous LayoutObject ancestry.
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        if !node.is_null() {
            return node;
        }
        let mut parent = self.Parent();
        let Some(parent_object) = (unsafe { parent.as_ref() }) else {
            return std::ptr::null_mut();
        };
        if !matches!(
            parent_object.StyleRef().StyleType(),
            PseudoId::kPseudoIdAfter
                | PseudoId::kPseudoIdBefore
                | PseudoId::kPseudoIdFirstLetter
                | PseudoId::kPseudoIdInterestButton
                | PseudoId::kPseudoIdMarker
                | PseudoId::kPseudoIdScrollButton
                | PseudoId::kPseudoIdScrollButtonBlockStart
                | PseudoId::kPseudoIdScrollButtonInlineStart
                | PseudoId::kPseudoIdScrollButtonInlineEnd
                | PseudoId::kPseudoIdScrollButtonBlockEnd
                | PseudoId::kPseudoIdScrollMarker
        ) {
            return std::ptr::null_mut();
        }
        while let Some(object) = unsafe { parent.as_ref() } {
            let node = object.GetNode();
            if !node.is_null() {
                return node;
            }
            parent = object.Parent();
        }
        std::ptr::null_mut()
    }

    pub fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutObjectUpdateHitTestResultProvider(self, result, offset) }
    }

    #[inline(never)]
    pub fn NodeAtPoint(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        if self.IsSVGHiddenContainer() {
            return false;
        }
        unsafe {
            LayoutObjectNodeAtPointProvider(self, result, location, accumulated_offset, phase)
        }
    }

    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpacePhysical(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut PhysicalRect,
        flags: VisualRectFlags,
    ) -> bool {
        self.CheckIsNotDestroyed();
        let mut float_rect = gfx::RectF::from(*rect);
        let intersects = self.MapToVisualRectInAncestorSpaceFloat(ancestor, &mut float_rect, flags);
        *rect = PhysicalRect::EnclosingRect(&float_rect);
        intersects
    }

    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpaceFloat(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
    ) -> bool {
        self.CheckIsNotDestroyed();
        // GeometryMapper acceleration is not enabled at this boundary. The
        // source TransformState slow path retains the same geometry semantics.
        let mut state = TransformState::new(
            TransformState::kApplyTransformDirection,
            gfx::QuadF::from(*rect),
        );
        let intersects = self.MapToVisualRectInAncestorSpaceInternal(ancestor, &mut state, flags);
        state.Flatten();
        *rect = state.LastPlanarQuad().BoundingBox();
        intersects
    }

    #[inline(never)]
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        self.MapVisualRectInternalDispatch(ancestor, state, flags)
    }

    #[inline(never)]
    pub fn GetPropertyContainer(
        &self,
        skip_info: *mut AncestorSkipInfo,
        state: *mut PropertyTreeStateOrAlias,
        flags: VisualRectFlags,
    ) -> *const LayoutObject {
        unsafe { LayoutObjectGetPropertyContainerProvider(self, skip_info, state, flags) }
    }

    pub fn LocalCaretRect(&self, offset: i32, shape: CaretShape) -> PhysicalRect {
        // Source virtual dispatch: layout_block.cc:507-525,
        // layout_inline.cc:253-286, layout_box.cc:3138-3183. Text and the
        // base LayoutObject return empty rectangles (layout_text.cc:715-718,
        // layout_object.cc:4170-4173).
        self.CheckIsNotDestroyed();
        if let Some(block) = unsafe { foundation::DynamicTo::<LayoutBlock>(self).as_ref() } {
            block.LocalCaretRect(offset, shape)
        } else if let Some(inline) = unsafe {
            foundation::DynamicTo::<crate::internal::layout_inline::LayoutInline>(self).as_ref()
        } {
            inline.LocalCaretRect(offset, shape)
        } else if let Some(box_) = unsafe { foundation::DynamicTo::<LayoutBox>(self).as_ref() } {
            box_.LocalCaretRect(offset, shape)
        } else {
            PhysicalRect::default()
        }
    }

    // This is a fixed-cost hit-test admission predicate. Keep it inlineable so
    // object-by-object hit testing does not manufacture a frame per candidate.
    pub fn VisibleToHitTestRequest(&self, request: &HitTestRequest) -> bool {
        unsafe { LayoutObjectVisibleToHitTestRequestProvider(self, request) }
    }

    #[inline(never)]
    pub fn MapLocalToAncestor(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    ) {
        unsafe { LayoutObjectMapLocalToAncestorProvider(self, ancestor, state, mode) }
    }

    // This is a specifically optimized caller-side path, so the dispatch
    // wrapper remains inlineable despite being physically fragment-owned.
    pub(crate) fn MapToVisualRectInAncestorSpaceInternalFastPath(
        &self,
        ancestor_or_null_for_viewport: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
        intersects: &mut bool,
    ) -> bool {
        let _ = (ancestor_or_null_for_viewport, rect, flags, intersects);
        // Required canonical Paint-property owner acceleration is deliberately
        // not enabled; callers use the fully translated source slow path.
        false
    }
}

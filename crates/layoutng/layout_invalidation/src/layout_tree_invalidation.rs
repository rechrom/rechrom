#![allow(non_snake_case)]

use foundation::RuntimeEnabledFeatures;

use crate::internal::layout_object::{LayoutObject, MarkingBehavior};

// Concrete grid and inline-fragment owners supply these virtual results when
// the class hierarchy is assembled.
unsafe extern "Rust" {
    fn DispatchLayoutObjectIsLayoutGridOrGridLanes(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVG(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGRoot(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGText(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGInline(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGForeignObject(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectHasInlineFragments(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectInlineFormattingContextWillChange(
        object: *mut LayoutObject,
        new_value: bool,
    );
    fn DispatchLayoutObjectInLayoutNGInlineFormattingContextWillChange(
        object: *mut LayoutObject,
        value: bool,
    );
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:1874-1874
    pub fn MarkContainerChainForLayout(&mut self) {
        self.MarkContainerChainForLayoutWithSchedule(true);
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:232-260
    #[inline(never)]
    pub fn MarkContainerChainForLayoutWithSchedule(&mut self, _schedule_relayout: bool) {
        self.CheckIsNotDestroyed();
        layout_invalidation::propagate_container_chain_for_layout(
            &mut LayoutObjectInvalidationTree,
            self,
        );
    }

    pub fn InLayoutNGInlineFormattingContextWillChangeBase(&self, _new_value: bool) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:3403-3405
    #[inline(never)]
    pub fn InLayoutNGInlineFormattingContextWillChange(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectInLayoutNGInlineFormattingContextWillChange(self, value) }
    }
}

// cpp: layoutng/internal/layout_tree_invalidation.cc:38-64
#[inline(never)]
fn ShouldInvalidateBeyond(object: &LayoutObject) -> bool {
    if object.IsLayoutInline() || object.IsText() {
        return true;
    }
    if object.IsTableRow() || object.IsTableSection() {
        return true;
    }
    if unsafe { DispatchLayoutObjectIsLayoutGridOrGridLanes(object) } {
        let style = object.StyleRef();
        if style.GridTemplateColumns().IsSubgriddedAxis()
            || style.GridTemplateRows().IsSubgriddedAxis()
        {
            return true;
        }
    }
    false
}

pub(crate) struct LayoutObjectInvalidationTree;

impl layout_invalidation::AnchorInvalidationTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn parent(&self, node: Self::Node) -> Option<Self::Node> {
        let parent = unsafe { &*node }.Parent();
        (!parent.is_null()).then_some(parent)
    }

    fn may_contain_anchor(&self, node: Self::Node) -> bool {
        unsafe { &*node }.MayContainAnchor()
    }

    fn mark_may_contain_anchor(&mut self, node: Self::Node) {
        unsafe { &mut *node }.SetSelfMayContainAnchor();
    }
}

impl layout_invalidation::InlineCollectionInvalidationTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn parent(&self, node: Self::Node) -> Option<Self::Node> {
        let parent = unsafe { &*node }.Parent();
        (!parent.is_null()).then_some(parent)
    }

    fn needs_collect_inlines(&self, node: Self::Node) -> bool {
        unsafe { &*node }.NeedsCollectInlines()
    }

    fn mark_needs_collect_inlines(&mut self, node: Self::Node) {
        unsafe { &mut *node }.SetNeedsCollectInlinesFlag(true);
    }

    fn is_layout_inline(&self, node: Self::Node) -> bool {
        unsafe { &*node }.IsLayoutInline()
    }
}

impl layout_invalidation::IntrinsicWidthInvalidationTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn container(&self, node: Self::Node) -> Option<Self::Node> {
        let container = unsafe { &*node }.Container();
        (!container.is_null()).then_some(container)
    }

    fn intrinsic_widths_dirty(&self, node: Self::Node) -> bool {
        unsafe { &*node }.IntrinsicLogicalWidthsDirty()
    }

    fn should_invalidate_beyond(&self, node: Self::Node) -> bool {
        ShouldInvalidateBeyond(unsafe { &*node })
    }

    fn is_layout_view(&self, node: Self::Node) -> bool {
        unsafe { &*node }.IsLayoutView()
    }

    fn mark_intrinsic_widths_dirty(&mut self, node: Self::Node) {
        unsafe { &mut *node }
            .bitfields_
            .layout_invalidation_
            .set_intrinsic_logical_widths_dirty(true);
    }

    fn is_out_of_flow_positioned(&self, node: Self::Node) -> bool {
        unsafe { &*node }.StyleRef().HasOutOfFlowPosition()
    }
}

impl layout_invalidation::ContainerChainInvalidationTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn container(&self, node: Self::Node) -> Option<Self::Node> {
        let container = unsafe { &*node }.Container();
        (!container.is_null()).then_some(container)
    }

    fn containing_block(&self, node: Self::Node) -> Option<Self::Node> {
        let containing_block = unsafe { &*node }.ContainingBlock() as *mut LayoutObject;
        (!containing_block.is_null()).then_some(containing_block)
    }

    fn self_needs_full_layout(&self, node: Self::Node) -> bool {
        unsafe { &*node }.SelfNeedsFullLayout()
    }

    fn child_needs_full_layout(&self, node: Self::Node) -> bool {
        unsafe { &*node }.ChildNeedsFullLayout()
    }

    fn mark_child_needs_full_layout(&mut self, node: Self::Node) {
        unsafe { &mut *node }.SetChildNeedsFullLayout(true);
    }

    fn needs_simplified_layout(&self, node: Self::Node) -> bool {
        unsafe { &*node }.NeedsSimplifiedLayout()
    }

    fn needs_simplified_layout_only(&self, node: Self::Node) -> bool {
        unsafe { &*node }.NeedsSimplifiedLayoutOnly()
    }

    fn mark_needs_simplified_layout(&mut self, node: Self::Node) {
        unsafe { &mut *node }.SetNeedsSimplifiedLayout(true);
    }

    fn is_text_or_svg_child(&self, node: Self::Node) -> bool {
        unsafe { &*node }.IsTextOrSVGChild()
    }

    fn is_out_of_flow_positioned(&self, node: Self::Node) -> bool {
        unsafe { &*node }.StyleRef().HasOutOfFlowPosition()
    }
}

impl layout_invalidation::MulticolDescendantTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn is_inside_multicol(&self, node: Self::Node) -> bool {
        unsafe { &*node }.IsInsideMulticol()
    }

    fn set_inside_multicol(&mut self, node: Self::Node, value: bool) {
        unsafe { &mut *node }.SetIsInsideMulticol(value);
    }

    fn is_multicol_container(&self, node: Self::Node) -> bool {
        unsafe { &*node }.IsMulticolContainer()
    }

    fn next_in_preorder(&self, node: Self::Node, root: Self::Node) -> Option<Self::Node> {
        let next = unsafe { &*node }.NextInPreOrder(unsafe { &*root });
        (!next.is_null()).then_some(next)
    }

    fn next_after_children(&self, node: Self::Node, root: Self::Node) -> Option<Self::Node> {
        let next = unsafe { &*node }.NextInPreOrderAfterChildren(unsafe { &*root });
        (!next.is_null()).then_some(next)
    }
}

impl layout_invalidation::SubtreeChangeListenerTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn listener_registered(&self, node: Self::Node) -> bool {
        unsafe { &*node }
            .bitfields_
            .subtree_change_listener_registered_
    }

    fn set_listener_registered(&mut self, node: Self::Node, value: bool) {
        unsafe { &mut *node }
            .bitfields_
            .subtree_change_listener_registered_ = value;
    }

    fn first_child(&self, node: Self::Node) -> Option<Self::Node> {
        let child = unsafe { &*node }.SlowFirstChild();
        (!child.is_null()).then_some(child)
    }

    fn next_sibling(&self, node: Self::Node) -> Option<Self::Node> {
        let sibling = unsafe { &*node }.NextSibling();
        (!sibling.is_null()).then_some(sibling)
    }
}

impl layout_invalidation::PaintInvalidationTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn parent(&self, node: Self::Node) -> Option<Self::Node> {
        let parent = unsafe { &*node }.Parent();
        (!parent.is_null()).then_some(parent)
    }

    fn should_check_for_paint_invalidation(&self, node: Self::Node) -> bool {
        unsafe { &*node }.ShouldCheckForPaintInvalidation()
    }

    fn should_check_layout_for_paint_invalidation(&self, node: Self::Node) -> bool {
        unsafe { &*node }.ShouldCheckLayoutForPaintInvalidation()
    }

    fn descendant_should_check_layout_for_paint_invalidation(&self, node: Self::Node) -> bool {
        unsafe { &*node }.DescendantShouldCheckLayoutForPaintInvalidation()
    }

    fn mark_layout_paint_invalidation(&mut self, node: Self::Node) {
        let object = unsafe { &mut *node };
        object.bitfields_.should_check_for_paint_invalidation_ = true;
        object
            .bitfields_
            .should_check_layout_for_paint_invalidation_ = true;
    }

    fn mark_descendant_layout_paint_invalidation(&mut self, node: Self::Node) {
        let object = unsafe { &mut *node };
        object.bitfields_.should_check_for_paint_invalidation_ = true;
        object
            .bitfields_
            .descendant_should_check_layout_for_paint_invalidation_ = true;
    }

    fn mark_paint_invalidation(&mut self, node: Self::Node) {
        unsafe { &mut *node }
            .bitfields_
            .should_check_for_paint_invalidation_ = true;
    }
}

impl layout_invalidation::SpannerParentInvalidationTree for LayoutObjectInvalidationTree {
    type Node = *mut LayoutObject;

    fn parent(&self, node: Self::Node) -> Option<Self::Node> {
        let parent = unsafe { &*node }.Parent();
        (!parent.is_null()).then_some(parent)
    }

    fn containing_block(&self, node: Self::Node) -> Option<Self::Node> {
        let containing_block = unsafe { &*node }.ContainingBlock() as *mut LayoutObject;
        (!containing_block.is_null()).then_some(containing_block)
    }

    fn container(&self, node: Self::Node) -> Option<Self::Node> {
        let container = unsafe { &*node }.Container();
        (!container.is_null()).then_some(container)
    }

    fn mark_child_layout_only(&mut self, node: Self::Node) {
        unsafe { &mut *node }.SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkOnlyThis);
    }

    fn mark_child_layout_chain(&mut self, node: Self::Node) {
        unsafe { &mut *node }.SetChildNeedsLayout();
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:1166-1177
    pub fn IsSVGChild(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVG(self) && !DispatchLayoutObjectIsSVGRoot(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1206-1209
    pub fn IsSVGInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGInline(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1218-1221
    pub fn IsSVGForeignObject(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGForeignObject(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1230-1233
    pub fn IsSVGText(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGText(self) }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:66-71
    pub fn MarkMayContainAnchor(&mut self) {
        layout_invalidation::propagate_may_contain_anchor(&mut LayoutObjectInvalidationTree, self);
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:73-89
    pub fn SetIsInsideMulticolIncludingDescendants(&mut self, inside_multicol: bool) {
        self.CheckIsNotDestroyed();
        layout_invalidation::propagate_inside_multicol(
            &mut LayoutObjectInvalidationTree,
            self,
            inside_multicol,
        );
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:91-103
    pub fn RegisterSubtreeChangeListenerOnDescendants(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        layout_invalidation::propagate_subtree_change_listener(
            &mut LayoutObjectInvalidationTree,
            self,
            value,
        );
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:105-112
    pub fn NotifyOfSubtreeChange(&mut self) -> bool {
        self.CheckIsNotDestroyed();
        if !self.bitfields_.subtree_change_listener_registered_
            || self.bitfields_.notified_of_subtree_change_
        {
            return false;
        }
        self.bitfields_.notified_of_subtree_change_ = true;
        true
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:114-119
    pub fn HandleSubtreeModifications(&mut self) {
        self.CheckIsNotDestroyed();
        if self.ConsumesSubtreeChangeNotification() {
            self.SubtreeDidChange();
        }
        self.bitfields_.notified_of_subtree_change_ = false;
    }

    // cpp: layoutng/internal/layout_object.h:2121-2121
    pub fn SubtreeDidChange(&self) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2124-2127
    pub fn ConsumesSubtreeChangeNotification(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.consumes_subtree_change_notification_
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:121-135
    pub fn SetNeedsCollectInlines(&mut self) {
        self.CheckIsNotDestroyed();
        if self.NeedsCollectInlines() {
            return;
        }
        if self.IsSVGChild()
            && !self.IsSVGText()
            && !self.IsSVGInline()
            && !self.IsSVGInlineText()
            && !self.IsSVGForeignObject()
        {
            return;
        }
        self.SetNeedsCollectInlinesFlag(true);
        let parent = self.Parent();
        if !parent.is_null() {
            unsafe { &mut *parent }.SetChildNeedsCollectInlines();
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:137-154
    pub fn SetChildNeedsCollectInlines(&mut self) {
        self.CheckIsNotDestroyed();
        layout_invalidation::propagate_needs_collect_inlines(
            &mut LayoutObjectInvalidationTree,
            self,
        );
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:156-188
    pub fn MarkParentForSpannerOrOutOfFlowPositionedChange(&mut self) {
        self.CheckIsNotDestroyed();
        #[cfg(debug_assertions)]
        debug_assert!(!self.bitfields_.set_needs_layout_forbidden_);
        let object = self.Parent();
        if object.is_null() {
            return;
        }
        if RuntimeEnabledFeatures::LayoutOOFCollectInlinesFixEnabled() {
            unsafe { &mut *object }.SetChildNeedsCollectInlines();
        } else {
            unsafe { &mut *object }.SetNeedsCollectInlines();
        }
        layout_invalidation::propagate_spanner_or_oof_parent_change(
            &mut LayoutObjectInvalidationTree,
            self,
        );
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:190-200
    pub fn SetIntrinsicLogicalWidthsDirty(&mut self, mark_parents: MarkingBehavior) {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .layout_invalidation_
            .mark_intrinsic_logical_widths_dirty();
        if mark_parents == MarkingBehavior::kMarkContainerChain
            && (self.IsText() || !self.StyleRef().HasOutOfFlowPosition())
        {
            self.InvalidateContainerIntrinsicLogicalWidths();
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:202-205
    pub fn ClearIntrinsicLogicalWidthsDirty(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .layout_invalidation_
            .clear_intrinsic_logical_widths_dirty();
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:207-227
    pub fn InvalidateContainerIntrinsicLogicalWidths(&self) {
        self.CheckIsNotDestroyed();
        layout_invalidation::propagate_intrinsic_widths_dirty(
            &mut LayoutObjectInvalidationTree,
            self as *const LayoutObject as *mut LayoutObject,
        );
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:229-240
    pub fn SetChildNeedsLayoutWithBehavior(&mut self, mark_parents: MarkingBehavior) {
        self.CheckIsNotDestroyed();
        #[cfg(debug_assertions)]
        debug_assert!(!self.bitfields_.set_needs_layout_forbidden_);
        let already_needed_layout = self.ChildNeedsFullLayout();
        self.SetNeedsOverflowRecalc();
        self.SetChildNeedsFullLayout(true);
        if !already_needed_layout && mark_parents == MarkingBehavior::kMarkContainerChain {
            self.MarkContainerChainForLayout();
        }
    }

    pub fn SetChildNeedsLayout(&mut self) {
        self.SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkContainerChain);
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:242-252
    pub fn MarkNeedsSimplifiedLayout(&mut self) {
        self.CheckIsNotDestroyed();
        let already_needed_layout = self.NeedsSimplifiedLayout();
        self.SetNeedsSimplifiedLayout(true);
        #[cfg(debug_assertions)]
        debug_assert!(!self.bitfields_.set_needs_layout_forbidden_);
        if !already_needed_layout {
            self.MarkContainerChainForLayout();
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:268-279
    pub fn SetIsInLayoutNGInlineFormattingContext(&mut self, new_value: bool) {
        self.CheckIsNotDestroyed();
        if self.IsInLayoutNGInlineFormattingContext() == new_value {
            return;
        }
        unsafe { DispatchLayoutObjectInlineFormattingContextWillChange(self, new_value) };
        debug_assert!(!unsafe { DispatchLayoutObjectHasInlineFragments(self) });
        self.bitfields_.is_in_layout_ng_inline_formatting_context_ = new_value;
        debug_assert!(!unsafe { DispatchLayoutObjectHasInlineFragments(self) });
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:281-284
    pub fn SetNeedsCollectInlinesFlag(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .layout_invalidation_
            .set_needs_collect_inlines(value);
    }

    // cpp: layoutng/internal/layout_object.h:1438-1441
    pub fn NeedsCollectInlines(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.layout_invalidation_.needs_collect_inlines()
    }

    // cpp: layoutng/internal/layout_object.h:1449-1452
    pub fn IntrinsicLogicalWidthsDirty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .layout_invalidation_
            .intrinsic_logical_widths_dirty()
    }

    // cpp: layoutng/internal/layout_object.h:1137-1145
    pub fn IsInsideMulticol(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_inside_multicol_
    }

    pub fn SetIsInsideMulticol(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_inside_multicol_ = value;
    }

    // cpp: layoutng/internal/layout_object.h:3320-3327
    pub fn IsMulticolContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_multicol_container_
    }

    pub fn SetIsMulticolContainer(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_multicol_container_ = value;
    }
}

#![allow(non_snake_case)]

use foundation::RuntimeEnabledFeatures;

use super::layout_object::{LayoutObject, MarkingBehavior};

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
}

// cpp: layoutng/internal/layout_tree_invalidation.cc:38-64
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
        let mut runner: *mut LayoutObject = self;
        while !runner.is_null() && !unsafe { &*runner }.MayContainAnchor() {
            let current = unsafe { &mut *runner };
            current.SetSelfMayContainAnchor();
            runner = current.Parent();
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:73-89
    pub fn SetIsInsideMulticolIncludingDescendants(&mut self, inside_multicol: bool) {
        self.CheckIsNotDestroyed();
        let mut object: *mut LayoutObject = self;
        while !object.is_null() {
            let current = unsafe { &mut *object };
            let was_inside_multicol = current.IsInsideMulticol();
            current.SetIsInsideMulticol(inside_multicol);
            object = if inside_multicol == was_inside_multicol || current.IsMulticolContainer() {
                current.NextInPreOrderAfterChildren(self)
            } else {
                current.NextInPreOrder(self)
            };
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:91-103
    pub fn RegisterSubtreeChangeListenerOnDescendants(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        if self.bitfields_.subtree_change_listener_registered_ == value {
            return;
        }
        self.bitfields_.subtree_change_listener_registered_ = value;
        let mut current = self.SlowFirstChild();
        while !current.is_null() {
            let child = unsafe { &mut *current };
            child.RegisterSubtreeChangeListenerOnDescendants(value);
            current = child.NextSibling();
        }
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
        let mut object: *mut LayoutObject = self;
        loop {
            let current = unsafe { &mut *object };
            if current.NeedsCollectInlines() {
                break;
            }
            current.SetNeedsCollectInlinesFlag(true);
            if !current.IsLayoutInline() {
                break;
            }
            object = current.Parent();
            if object.is_null() {
                break;
            }
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:156-188
    pub fn MarkParentForSpannerOrOutOfFlowPositionedChange(&mut self) {
        self.CheckIsNotDestroyed();
        #[cfg(debug_assertions)]
        debug_assert!(!self.bitfields_.set_needs_layout_forbidden_);
        let mut object = self.Parent();
        if object.is_null() {
            return;
        }
        if RuntimeEnabledFeatures::LayoutOOFCollectInlinesFixEnabled() {
            unsafe { &mut *object }.SetChildNeedsCollectInlines();
        } else {
            unsafe { &mut *object }.SetNeedsCollectInlines();
        }
        let containing_block = self.ContainingBlock() as *mut LayoutObject;
        while object != containing_block {
            let current = unsafe { &mut *object };
            current.SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkOnlyThis);
            object = current.Container();
        }
        if !object.is_null() {
            unsafe { &mut *object }.SetChildNeedsLayout();
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:190-200
    pub fn SetIntrinsicLogicalWidthsDirty(&mut self, mark_parents: MarkingBehavior) {
        self.CheckIsNotDestroyed();
        self.bitfields_.intrinsic_logical_widths_dirty_ = true;
        self.bitfields_
            .intrinsic_logical_widths_depends_on_block_constraints_ = true;
        self.bitfields_.indefinite_intrinsic_logical_widths_dirty_ = true;
        self.bitfields_.definite_intrinsic_logical_widths_dirty_ = true;
        if mark_parents == MarkingBehavior::kMarkContainerChain
            && (self.IsText() || !self.StyleRef().HasOutOfFlowPosition())
        {
            self.InvalidateContainerIntrinsicLogicalWidths();
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:202-205
    pub fn ClearIntrinsicLogicalWidthsDirty(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.intrinsic_logical_widths_dirty_ = false;
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:207-227
    pub fn InvalidateContainerIntrinsicLogicalWidths(&self) {
        self.CheckIsNotDestroyed();
        let mut object = self.Container();
        while !object.is_null()
            && (!unsafe { &*object }.IntrinsicLogicalWidthsDirty()
                || ShouldInvalidateBeyond(unsafe { &*object }))
        {
            let current = unsafe { &mut *object };
            let container = current.Container();
            if container.is_null() && !current.IsLayoutView() {
                break;
            }
            current.bitfields_.intrinsic_logical_widths_dirty_ = true;
            if current.StyleRef().HasOutOfFlowPosition() {
                break;
            }
            object = container;
        }
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
        self.bitfields_.needs_collect_inlines_ = value;
    }

    // cpp: layoutng/internal/layout_object.h:1438-1441
    pub fn NeedsCollectInlines(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.needs_collect_inlines_
    }

    // cpp: layoutng/internal/layout_object.h:1449-1452
    pub fn IntrinsicLogicalWidthsDirty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.intrinsic_logical_widths_dirty_
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

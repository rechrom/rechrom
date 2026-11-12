//! StyleEngine::DetachedFromParent and DetachLayoutTreeScope, h150-166,
//! h758-772; cc4866-4899. Page routes only small native/DOM facts.
use foundation::layout_detach::{
    LayoutDetachParentFlags, LayoutDetachParentMutation, LayoutDetachedParent,
};
use std::{cell::Cell, rc::Rc};
#[derive(Default)]
pub(crate) struct StyleDetachState {
    in_scope: Cell<bool>,
    parent_for_detached_subtree: Cell<Option<u64>>,
}
pub struct StyleDetachScope {
    state: Rc<StyleDetachState>,
    old_value: bool,
    finished: bool,
}
pub struct StyleDetachCompletion {
    pub mark_ancestors_for: Option<u64>,
    pub invalidate_pending_svg_resources: bool,
}
impl StyleDetachState {
    pub(crate) fn Clear(&self) {
        self.parent_for_detached_subtree.set(None);
        self.in_scope.set(false);
    }
    pub(crate) fn Begin(self: &Rc<Self>) -> StyleDetachScope {
        let old_value = self.in_scope.replace(true);
        StyleDetachScope {
            state: self.clone(),
            old_value,
            finished: false,
        }
    }
    pub(crate) fn DetachedFromParent(&self, parent: LayoutDetachedParent) {
        if self.in_scope.get() {
            self.parent_for_detached_subtree.set(parent.element);
        }
    }
}
impl StyleDetachScope {
    pub fn ParentForDetachedSubtree(&self) -> Option<u64> {
        self.state.parent_for_detached_subtree.get()
    }
    pub fn PlanParentUpdate(
        &self,
        flags: LayoutDetachParentFlags,
        may_have_flat_tree_children: bool,
    ) -> LayoutDetachParentMutation {
        assert!(self.ParentForDetachedSubtree().is_some());
        LayoutDetachParentMutation {
            set_whitespace_children_may_change: may_have_flat_tree_children
                && !flags.whitespace_children_may_change,
            notify_of_subtree_change: !flags.was_notified_of_subtree_change,
        }
    }
    pub fn Finish(
        mut self,
        update: Option<LayoutDetachParentMutation>,
        notified: Option<bool>,
    ) -> StyleDetachCompletion {
        let parent = self.state.parent_for_detached_subtree.take();
        let mark_ancestors_for = match (parent, update, notified) {
            (Some(parent), Some(update), Some(notified))
                if update.set_whitespace_children_may_change || notified =>
            {
                Some(parent)
            }
            (Some(_), Some(_), Some(_)) | (None, None, None) => None,
            _ => panic!("detach scope completes its actual surviving parent"),
        };
        self.state.in_scope.set(self.old_value);
        self.finished = true;
        StyleDetachCompletion {
            mark_ancestors_for,
            invalidate_pending_svg_resources: true,
        }
    }
}
impl Drop for StyleDetachScope {
    fn drop(&mut self) {
        if !self.finished {
            self.state.parent_for_detached_subtree.set(None);
            self.state.in_scope.set(self.old_value);
        }
    }
}

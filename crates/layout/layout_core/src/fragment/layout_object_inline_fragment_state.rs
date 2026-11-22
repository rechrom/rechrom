//! LayoutObject retained inline-fragment bookkeeping dispatch.
//!
//! These queries and mutations execute per object while fragment items are
//! built or traversed, so they remain inlineable. Traversal frames belong to
//! the fragment-item and cursor owners, not to this scalar dispatch surface.

use super::*;

impl LayoutObject {
    pub fn IsFragmentLessBoxBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsFragmentLessBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsFragmentLessBox(self) }
    }

    pub fn HasInlineFragmentsBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn HasInlineFragments(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectHasInlineFragments(self) }
    }

    pub fn FirstInlineFragmentItemIndexBase(&self) -> usize {
        self.CheckIsNotDestroyed();
        0
    }

    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectFirstInlineFragmentItemIndex(self) }
    }

    pub fn ClearFirstInlineFragmentItemIndexBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectClearFirstInlineFragmentItemIndex(self) }
    }

    pub fn SetFirstInlineFragmentItemIndexBase(&mut self, _index: usize) {
        self.CheckIsNotDestroyed();
    }

    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectSetFirstInlineFragmentItemIndex(self, index) }
    }
}

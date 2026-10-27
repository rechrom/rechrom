//! LayoutObject retained inline-fragment bookkeeping dispatch.
//!
//! These queries and mutations execute per object while fragment items are
//! built or traversed, so they remain inlineable. Traversal frames belong to
//! the fragment-item and cursor owners, not to this scalar dispatch surface.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:1994-1997
    pub fn IsFragmentLessBoxBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:1994-1997
    pub fn IsFragmentLessBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsFragmentLessBox(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2001-2004
    pub fn HasInlineFragmentsBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:2001-2004
    pub fn HasInlineFragments(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectHasInlineFragments(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2011-2014
    pub fn FirstInlineFragmentItemIndexBase(&self) -> usize {
        self.CheckIsNotDestroyed();
        0
    }

    // cpp: layoutng/internal/layout_object.h:2011-2014
    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectFirstInlineFragmentItemIndex(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2015-2015
    pub fn ClearFirstInlineFragmentItemIndexBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2015-2015
    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectClearFirstInlineFragmentItemIndex(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2016-2016
    pub fn SetFirstInlineFragmentItemIndexBase(&mut self, _index: usize) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2016-2016
    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectSetFirstInlineFragmentItemIndex(self, index) }
    }
}

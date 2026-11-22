//! LayoutObject visual-overflow and paint invalidation dispatch.

use super::*;
use crate::internal::layout_text::LayoutText;

impl LayoutObject {
    // cpp: layout_object.cc:4304-4335; node.cc:1981-1984. The optional
    // value is the source StyleEngine::DetachedFromParent notification, not
    // a callback into Style or DOM while this native borrow is held.
    #[inline(never)]
    pub fn DestroyAndCleanupAnonymousWrappers(
        &mut self,
        performing_reattach: bool,
    ) -> Option<foundation::layout_detach::LayoutDetachedParent> {
        self.CheckIsNotDestroyed();
        let mut destroy_root = self as *mut LayoutObject;
        let mut parent = self.Parent();
        while !parent.is_null() && unsafe { &*parent }.IsAnonymous() {
            let grandparent = unsafe { &*parent }.Parent();
            if !grandparent.is_null() && unsafe { &*grandparent }.IsFieldset() {
                break;
            }
            if !unsafe { &*destroy_root }.PreviousSibling().is_null()
                || !unsafe { &*destroy_root }.NextSibling().is_null()
            {
                break;
            }
            destroy_root = parent;
            parent = unsafe { &*parent }.Parent();
        }
        let notice = if !performing_reattach && !parent.is_null() {
            while unsafe { &*parent }.IsAnonymous() {
                parent = unsafe { &*parent }.Parent();
            }
            let node = unsafe { &*parent }.GetNode();
            Some(foundation::layout_detach::LayoutDetachedParent {
                element: unsafe { node.as_ref() }
                    .filter(|node| node.IsElementNode())
                    .map(|node| node.InputId()),
            })
        } else {
            None
        };
        unsafe { &mut *destroy_root }.Destroy();
        notice
    }

    // cpp: layoutng/internal/layout_object.h:2108-2108
    #[inline(never)]
    pub fn InvalidateVisualOverflow(&mut self) {
        // Source: core/layout/layout_object.cc:2289-2302.
        self.CheckIsNotDestroyed();
        if self.IsInLayoutNGInlineFormattingContext() && !self.NeedsLayout() {
            let text = DynamicTo::<LayoutText>(self as *mut LayoutObject);
            if !text.is_null() {
                unsafe { &mut *text }.InvalidateVisualOverflow();
            }
        }
        let layer = self.PaintingLayerDefault();
        if !layer.is_null() {
            unsafe { &mut *layer }.SetNeedsVisualOverflowRecalc();
        }
        self.SetShouldCheckForPaintInvalidation();
    }

    // Base classes without line boxes intentionally perform no propagation.
    // cpp: layoutng/internal/layout_object.h:2197-2197
    pub fn DirtyLinesFromChangedChild(&mut self, _child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2834-2834
    #[inline(never)]
    pub fn InvalidatePaint(&self, context: &PaintInvalidatorContext) {
        unsafe { LayoutObjectInvalidatePaintProvider(self, context) }
    }

    // cpp: layoutng/internal/layout_object.h:2843-2843
    #[inline(never)]
    pub fn InvalidateDisplayItemClients(&self, reason: PaintInvalidationReason) {
        unsafe { LayoutObjectInvalidateDisplayItemClientsProvider(self, reason) }
    }
}

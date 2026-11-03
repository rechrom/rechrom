//! LayoutObject selection state and retained-fragment geometry queries.
//!
//! Selection rectangles may aggregate retained geometry, so that entry point
//! keeps a fragment-owned sampling frame. Fixed-cost state and virtual-policy
//! predicates remain inlineable because they are called per object.

use super::*;
use crate::internal::layout_pass_scope::LayoutObjectFactoryScope;
use crate::internal::layout_text::LayoutText;
use foundation::UnsupportedLayout;

impl LayoutObject {
    pub fn CanUpdateSelectionOnRootLineBoxes(&self) -> bool {
        unsafe { LayoutObjectCanUpdateSelectionOnRootLineBoxesProvider(self) }
    }

    #[inline(never)]
    pub fn AbsoluteSelectionRect(&self) -> PhysicalRect {
        unsafe { LayoutObjectAbsoluteSelectionRectProvider(self) }
    }

    pub fn CanBeSelectionLeaf(&self) -> bool {
        self.CheckIsNotDestroyed();
        if !self.SlowFirstChild().is_null()
            || self.StyleRef().Visibility() != EVisibility::kVisible
            || self.HasLockedAncestorPreventingPaint()
        {
            return false;
        }
        self.CanBeSelectionLeafInternal()
    }

    // cpp: core/layout/layout_object.cc:2935-2945
    pub fn IsSelected(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.GetSelectionState() != SelectionState::kNone {
            return true;
        }
        let text = DynamicTo::<LayoutText>(self);
        if text.is_null() || !unsafe { &*text }.IsTextFragment() {
            return false;
        }
        let factories = LayoutObjectFactoryScope::Objects();
        let is_selected = unsafe { factories.as_ref() }
            .and_then(|factories| factories.is_text_fragment_selected)
            .unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "inline text-fragment selection query is not installed",
                ))
            });
        is_selected(unsafe { &*text })
    }

    // cpp: core/layout/layout_object.cc:2947-2950
    pub fn IsSelectable(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().IsSelectable()
    }
}

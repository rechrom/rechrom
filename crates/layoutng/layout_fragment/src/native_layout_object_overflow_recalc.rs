use super::*;

unsafe extern "Rust" {
    fn LayoutObjectRecalcNormalFlowChildVisualOverflowIfNeededProvider(object: *mut LayoutObject);
}

impl LayoutObject {
    // Base virtual implementation. Child calls retain derived dispatch.
    // cpp: layoutng/internal/layout_object_core_services.cc:280-292
    pub fn RecalcScrollableOverflow(&mut self) -> RecalcScrollableOverflowResult {
        LayoutObjectRecalcScrollableOverflow(self)
    }

    // Base virtual implementation. Child calls retain derived dispatch.
    // cpp: layoutng/internal/layout_object_core_services.cc:294-300
    pub fn RecalcVisualOverflow(&mut self) {
        LayoutObjectRecalcVisualOverflow(self)
    }

    // cpp: layoutng/internal/layout_object.h:2113-2113
    #[inline(never)]
    pub fn RecalcNormalFlowChildVisualOverflowIfNeeded(&mut self) {
        unsafe { LayoutObjectRecalcNormalFlowChildVisualOverflowIfNeededProvider(self) }
    }
}

// Base virtual implementation. Child calls retain derived dispatch.
// cpp: layoutng/internal/layout_object_core_services.cc:280-292
#[inline(never)]
pub(super) fn LayoutObjectRecalcScrollableOverflow(
    object: &mut LayoutObject,
) -> RecalcScrollableOverflowResult {
    object.CheckIsNotDestroyed();
    object.ClearSelfNeedsScrollableOverflowRecalc();
    if !object.ChildNeedsScrollableOverflowRecalc() {
        return RecalcScrollableOverflowResult::default();
    }
    object.ClearChildNeedsScrollableOverflowRecalc();
    let mut changed = false;
    let mut child = object.SlowFirstChild();
    while !child.is_null() {
        changed |= unsafe { DispatchLayoutObjectRecalcScrollableOverflow(child) }
            .scrollable_overflow_changed;
        child = unsafe { &*child }.NextSibling();
    }
    RecalcScrollableOverflowResult {
        scrollable_overflow_changed: changed,
        rebuild_fragment_tree: false,
    }
}

// Base virtual implementation. Child calls retain derived dispatch.
// cpp: layoutng/internal/layout_object_core_services.cc:294-300
#[inline(never)]
pub(super) fn LayoutObjectRecalcVisualOverflow(object: &mut LayoutObject) {
    object.CheckIsNotDestroyed();
    let mut child = object.SlowFirstChild();
    while !child.is_null() {
        unsafe { DispatchLayoutObjectRecalcVisualOverflow(child) };
        child = unsafe { &*child }.NextSibling();
    }
}

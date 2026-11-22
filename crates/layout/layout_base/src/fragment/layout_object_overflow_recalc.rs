use super::*;

impl LayoutObject {
    // Base virtual implementation. Child calls retain derived dispatch.
    pub fn RecalcScrollableOverflow(&mut self) -> RecalcScrollableOverflowResult {
        LayoutObjectRecalcScrollableOverflow(self)
    }

    // Virtual entry point. The assembled hierarchy selects the concrete
    // override exactly as LayoutObject's C++ vtable does.
    // Source: core/layout/layout_object.h:2170.
    pub fn RecalcVisualOverflow(&mut self) {
        unsafe { DispatchLayoutObjectRecalcVisualOverflow(self) }
    }

    // Explicit base implementation for derived overrides.
    // Source: core/layout/layout_object.cc:2268-2278.
    pub fn RecalcVisualOverflowBase(&mut self) {
        LayoutObjectRecalcVisualOverflow(self)
    }

    // Source: core/layout/layout_object.cc:2280-2287.
    #[inline(never)]
    pub fn RecalcNormalFlowChildVisualOverflowIfNeeded(&mut self) {
        self.CheckIsNotDestroyed();
        if self.IsOutOfFlowPositioned() {
            return;
        }
        if self.HasLayer() {
            let box_model = DynamicTo::<LayoutBoxModelObject>(self as *mut LayoutObject);
            debug_assert!(!box_model.is_null());
            if !box_model.is_null() && unsafe { &*box_model }.HasSelfPaintingLayer() {
                return;
            }
        }
        self.RecalcVisualOverflow();
    }
}

// Base virtual implementation. Child calls retain derived dispatch.
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

// Source: core/layout/layout_object.cc:2268-2278.
#[inline(never)]
pub(super) fn LayoutObjectRecalcVisualOverflow(object: &mut LayoutObject) {
    object.CheckIsNotDestroyed();
    let mut child = object.SlowFirstChild();
    while !child.is_null() {
        let current = unsafe { &*child };
        if current.HasLayer() {
            let box_model = DynamicTo::<LayoutBoxModelObject>(child);
            debug_assert!(!box_model.is_null());
            if !box_model.is_null() && unsafe { &*box_model }.HasSelfPaintingLayer() {
                child = current.NextSibling();
                continue;
            }
        }
        unsafe { DispatchLayoutObjectRecalcVisualOverflow(child) };
        child = current.NextSibling();
    }
}

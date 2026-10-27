// cpp: layoutng/internal/layout_object.h:117-120
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum MarkingBehavior {
    kMarkOnlyThis,
    kMarkContainerChain,
}

/// Layout-only dirty state retained by each native LayoutObject.
///
/// Propagation still walks the private object tree in the core assembly. The
/// state itself is isolated here so invalidation policy no longer shares the
/// complete LayoutObject bitfield record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutInvalidationState {
    self_needs_full_layout: bool,
    child_needs_full_layout: bool,
    needs_simplified_layout: bool,
    intrinsic_logical_widths_dirty: bool,
    intrinsic_logical_widths_depends_on_block_constraints: bool,
    indefinite_intrinsic_logical_widths_dirty: bool,
    definite_intrinsic_logical_widths_dirty: bool,
    needs_collect_inlines: bool,
}

impl Default for LayoutInvalidationState {
    fn default() -> Self {
        Self {
            self_needs_full_layout: false,
            child_needs_full_layout: false,
            needs_simplified_layout: false,
            intrinsic_logical_widths_dirty: false,
            intrinsic_logical_widths_depends_on_block_constraints: true,
            indefinite_intrinsic_logical_widths_dirty: true,
            definite_intrinsic_logical_widths_dirty: true,
            needs_collect_inlines: false,
        }
    }
}

impl LayoutInvalidationState {
    pub fn self_needs_full_layout(&self) -> bool {
        self.self_needs_full_layout
    }

    pub fn set_self_needs_full_layout(&mut self, value: bool) {
        self.self_needs_full_layout = value;
    }

    pub fn child_needs_full_layout(&self) -> bool {
        self.child_needs_full_layout
    }

    pub fn set_child_needs_full_layout(&mut self, value: bool) {
        self.child_needs_full_layout = value;
    }

    pub fn needs_simplified_layout(&self) -> bool {
        self.needs_simplified_layout
    }

    pub fn set_needs_simplified_layout(&mut self, value: bool) {
        self.needs_simplified_layout = value;
    }

    pub fn intrinsic_logical_widths_dirty(&self) -> bool {
        self.intrinsic_logical_widths_dirty
    }

    pub fn set_intrinsic_logical_widths_dirty(&mut self, value: bool) {
        self.intrinsic_logical_widths_dirty = value;
    }

    pub fn intrinsic_logical_widths_depends_on_block_constraints(&self) -> bool {
        self.intrinsic_logical_widths_depends_on_block_constraints
    }

    pub fn set_intrinsic_logical_widths_depends_on_block_constraints(&mut self, value: bool) {
        self.intrinsic_logical_widths_depends_on_block_constraints = value;
    }

    pub fn indefinite_intrinsic_logical_widths_dirty(&self) -> bool {
        self.indefinite_intrinsic_logical_widths_dirty
    }

    pub fn set_indefinite_intrinsic_logical_widths_dirty(&mut self, value: bool) {
        self.indefinite_intrinsic_logical_widths_dirty = value;
    }

    pub fn definite_intrinsic_logical_widths_dirty(&self) -> bool {
        self.definite_intrinsic_logical_widths_dirty
    }

    pub fn set_definite_intrinsic_logical_widths_dirty(&mut self, value: bool) {
        self.definite_intrinsic_logical_widths_dirty = value;
    }

    pub fn needs_collect_inlines(&self) -> bool {
        self.needs_collect_inlines
    }

    pub fn set_needs_collect_inlines(&mut self, value: bool) {
        self.needs_collect_inlines = value;
    }

    pub fn needs_layout(&self) -> bool {
        self.self_needs_full_layout || self.child_needs_full_layout || self.needs_simplified_layout
    }

    pub fn needs_simplified_layout_only(&self) -> bool {
        self.needs_simplified_layout
            && !self.self_needs_full_layout
            && !self.child_needs_full_layout
    }

    pub fn mark_intrinsic_logical_widths_dirty(&mut self) {
        self.intrinsic_logical_widths_dirty = true;
        self.intrinsic_logical_widths_depends_on_block_constraints = true;
        self.indefinite_intrinsic_logical_widths_dirty = true;
        self.definite_intrinsic_logical_widths_dirty = true;
    }

    pub fn clear_intrinsic_logical_widths_dirty(&mut self) {
        self.intrinsic_logical_widths_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_layout_dominates_the_simplified_only_state() {
        let mut state = LayoutInvalidationState::default();
        state.set_needs_simplified_layout(true);
        assert!(state.needs_simplified_layout_only());
        assert!(state.needs_layout());

        state.set_child_needs_full_layout(true);
        assert!(!state.needs_simplified_layout_only());
        assert!(state.needs_layout());
    }

    #[test]
    fn intrinsic_width_invalidation_marks_both_constraint_variants() {
        let mut state = LayoutInvalidationState::default();
        state.clear_intrinsic_logical_widths_dirty();
        state.set_indefinite_intrinsic_logical_widths_dirty(false);
        state.set_definite_intrinsic_logical_widths_dirty(false);

        state.mark_intrinsic_logical_widths_dirty();
        assert!(state.intrinsic_logical_widths_dirty());
        assert!(state.intrinsic_logical_widths_depends_on_block_constraints());
        assert!(state.indefinite_intrinsic_logical_widths_dirty());
        assert!(state.definite_intrinsic_logical_widths_dirty());
    }
}

#![allow(non_snake_case)]

use crate::{UserInteractionState, DOM};
use layoutng_assembly::{
    editing_state::LayoutEditingState,
    internal::layout_input::{
        CompatibilityMode as LayoutCompatibilityMode, ConstraintSpace, Offset,
    },
    layout_engine::LayoutMutation,
};
use std::rc::Rc;

// DOM owns style/input state. Native tree ownership and application of these
// synchronous mutation events belong to the Page's LayoutEngine.
impl DOM {
    pub fn EmitLayoutMutations(
        &mut self,
        interaction: &UserInteractionState,
        receive: impl FnMut(LayoutMutation<'_>),
    ) {
        super::EmitLayoutMutations(self, interaction, receive);
    }

    /// Emit the resolved style for one element whose animation sample contains
    /// only non-geometric paint properties. LayoutEngine still validates the
    /// resident native object before accepting the targeted update.
    pub fn EmitPaintStyle(
        &self,
        node_id: u64,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) -> bool {
        let document = self.GetDocument();
        let Some(index) = document.FindNodeById(node_id) else {
            return false;
        };
        let Some(resolved) = document.ResolvedStyleFor(index) else {
            return false;
        };
        receive(LayoutMutation::PaintStyle {
            node_id,
            style: &resolved.style,
        });
        true
    }

    /// Return whether a live DOM node was found and a scroll mutation emitted.
    /// A receiver can separately inspect its native engine's application result.
    pub fn EmitScrollOffset(
        &mut self,
        node_id: u64,
        offset: Offset,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) -> bool {
        if !self.SetScrollOffset(node_id, offset) {
            return false;
        }
        receive(LayoutMutation::ScrollOffset { node_id, offset });
        true
    }

    pub fn EmitEditing(&self, node_id: u64, mut receive: impl FnMut(LayoutMutation<'_>)) -> bool {
        if self.GetDocument().FindNodeById(node_id).is_none() {
            return false;
        }
        receive(LayoutMutation::Editing { node_id });
        true
    }

    pub fn EmitEditingState(
        &self,
        state: Rc<LayoutEditingState>,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) {
        receive(LayoutMutation::EditingState(state));
    }

    pub fn EmitConstraints(
        &self,
        space: &ConstraintSpace,
        mut receive: impl FnMut(LayoutMutation<'_>),
    ) {
        // Document mode is parser-owned semantic state, while viewport and
        // platform inputs come from the Page host.  Resolve them at this DOM →
        // Layout boundary so callers cannot accidentally lay out a quirks
        // document with the host's default standards-mode constraint.
        let mut resolved = space.clone();
        resolved.compatibility_mode = match self.GetDocument().GetCompatibilityMode() {
            crate::persistent_document::CompatibilityMode::kNoQuirks => {
                LayoutCompatibilityMode::kStandards
            }
            crate::persistent_document::CompatibilityMode::kLimitedQuirks => {
                LayoutCompatibilityMode::kLimitedQuirks
            }
            crate::persistent_document::CompatibilityMode::kQuirks => {
                LayoutCompatibilityMode::kQuirks
            }
        };
        receive(LayoutMutation::Constraints(&resolved));
    }
}

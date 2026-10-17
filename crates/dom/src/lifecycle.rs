#![allow(non_snake_case)]

use crate::{style_state::StyleUpdateImpact, DOM};
use cssom::CSSDeclaration;
use layoutng_assembly::internal::layout_input::Offset;

/// Lifecycle admission uses the document's existing accumulated impact and
/// geometry revision; these methods do not resolve or copy a separate tree.
impl DOM {
    pub fn GetStyleImpact(&self) -> StyleUpdateImpact {
        self.GetDocument().StyleState().impact
    }

    pub fn GetMeasurementGeometryRevision(&self) -> u64 {
        self.GetDocument()
            .StyleState()
            .measurement_geometry_revision
    }

    pub fn InvalidateLayout(&mut self) {
        self.GetDocumentMut()
            .StyleStateMut()
            .layout_inputs_require_tree = true;
        self.GetDocumentMut()
            .StyleStateMut()
            .impact
            .Merge(StyleUpdateImpact::LAYOUT);
    }

    pub fn InvalidatePaint(&mut self) {
        self.GetDocumentMut()
            .StyleStateMut()
            .impact
            .Merge(StyleUpdateImpact {
                paint: true,
                ..Default::default()
            });
    }

    pub fn DidCommitPaint(&mut self) {
        let state = self.GetDocumentMut().StyleStateMut();
        state.impact = Default::default();
        state.layout_inputs_require_tree = false;
        state.layout_tree_roots.clear();
        state.layout_reattach_nodes.clear();
        state.layout_style_nodes.clear();
        state.paint_style_nodes.clear();
        state.paint_style_requires_export = false;
    }

    /// Return whether the DOM node exists. Updating an offset does not change
    /// style impact; the Page batches its native scroll synchronization.
    pub fn SetScrollOffset(&mut self, node_id: u64, offset: Offset) -> bool {
        let Some(index) = self.GetDocument().FindNodeById(node_id) else {
            return false;
        };
        self.GetDocumentMut().SetScrollOffset(index, offset);
        true
    }

    pub fn SetAnimationStyle(
        &mut self,
        node_id: u64,
        effect_id: u64,
        declarations: Vec<CSSDeclaration>,
    ) {
        self.GetDocumentMut()
            .SetAnimationStyle(node_id, effect_id, declarations);
    }

    pub fn SetTransitionStyle(
        &mut self,
        node_id: u64,
        effect_id: u64,
        declarations: Vec<CSSDeclaration>,
    ) {
        self.GetDocumentMut()
            .SetTransitionStyle(node_id, effect_id, declarations);
    }

    pub fn NeedsStyleSheetParsing(&self, node_id: u64) -> bool {
        let state = self.GetDocument().StyleState();
        !state.parsed_style_elements.contains(&node_id)
            || state.dirty_style_elements.contains(&node_id)
    }
}

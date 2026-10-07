#![allow(non_snake_case)]

use crate::{style_state::StyleUpdateImpact, DOM};
use cssom::{CSSDeclaration, CSSStyleSheet};
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
        self.GetDocumentMut().StyleStateMut().impact = Default::default();
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

    pub fn ResolveStyles(
        &mut self,
        environment: &crate::style_resolver::StyleEnvironment,
        user_agent_sheets: &[CSSStyleSheet],
    ) {
        crate::style_resolver::ResolveComputedStyles(self, environment, user_agent_sheets);
    }
    pub fn NeedsStyleSheetParsing(&self, node_id: u64) -> bool {
        let state = self.GetDocument().StyleState();
        !state.parsed_style_elements.contains(&node_id)
            || state.dirty_style_elements.contains(&node_id)
    }
}

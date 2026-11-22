//! DocumentMarkerController native invalidation on the original LayoutObject.
//! Source document_marker_controller.cc:134-174;
//! highlight/highlight_style_utils.cc:579-598.
use super::*;
use crate::internal::layout_object::AssociatedLayoutObjectOfDefault;
use foundation::PaintInvalidationReason;
#[derive(Clone, Copy)]
pub enum DocumentMarkerInvalidationType {
    Spelling,
    Grammar,
    TextFragment,
    Glic,
}
impl LayoutEngine {
    pub fn InvalidateDocumentMarkerPaint(&mut self, id: u64) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(object) = self
            .GetLayoutTree()
            .and_then(|tree| tree.OutputNode(id))
            .map(|object| object as *const LayoutObject as *mut LayoutObject)
        else {
            return false;
        };
        let node = unsafe { &*object }.GetNode();
        let first = unsafe { node.as_ref() }
            .and_then(|node| unsafe { AssociatedLayoutObjectOfDefault(node, 0).as_ref() })
            .map(|object| object as *const LayoutObject as *mut LayoutObject);
        unsafe { &mut *object }
            .SetShouldDoFullPaintInvalidationWithReason(PaintInvalidationReason::kDocumentMarker);
        let mut changed = vec![object];
        if let Some(first) = first.filter(|first| *first != object) {
            unsafe { &mut *first }.SetShouldDoFullPaintInvalidationWithReason(
                PaintInvalidationReason::kDocumentMarker,
            );
            changed.push(first);
        }
        self.pending_paint_ids_.insert(id);
        self.RefreshSelectionPaintInputs(&changed);
        true
    }
    pub fn InvalidateDocumentMarkerVisualOverflow(
        &mut self,
        id: u64,
        kind: DocumentMarkerInvalidationType,
    ) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(object) = self
            .GetLayoutTree()
            .and_then(|tree| tree.OutputNode(id))
            .map(|object| object as *const LayoutObject as *mut LayoutObject)
        else {
            return false;
        };
        let should_invalidate = match kind {
            DocumentMarkerInvalidationType::Spelling | DocumentMarkerInvalidationType::Grammar => {
                true
            }
            DocumentMarkerInvalidationType::Glic => false,
            DocumentMarkerInvalidationType::TextFragment => {
                let style = unsafe { &*object }.StyleRef().HighlightData().TargetText();
                unsafe { style.as_ref() }.is_some_and(|style| {
                    style.HasAppliedTextDecorations() || style.HasVisualOverflowingEffect()
                })
            }
        };
        if should_invalidate {
            unsafe { &mut *object }.InvalidateVisualOverflow();
            self.pending_inline_ink_ids_.insert(id);
        }
        should_invalidate
    }
}

//! Borrowed geometry for the current interaction transaction. Layout retains
//! its resident objects; no editing model or command backend is reconstructed.
use crate::ownership::{InteractionDocument, ReadNode};
use dom::{DOMBoundaryPoint, DOMNodeType};
use interaction_default::core::editing::backspace_state_machine::PreviousBackwardDeletionOffset;
use layoutng_assembly::layout_engine::LayoutOutput;

#[derive(Clone, Copy)]
pub struct PreparedEditingGeometry<'a> {
    output: LayoutOutput<'a>,
}

/// Resolve a control's UTF-16 cache onto its resident CharacterData before
/// entering Blink's backward-deletion state machine. No control-value string
/// or synthesized text run is used for segmentation.
///
/// A text-node boundary or BR enters the translated visible-position movement
/// using the current resident Layout result. Missing preparation is explicit.
pub(crate) fn PreviousTextControlDeletionOffset(
    document: &InteractionDocument,
    control: u64,
    caret: usize,
    geometry: Option<PreparedEditingGeometry<'_>>,
) -> Option<usize> {
    ReadNode(document, control, |document, _, control| {
        let point = document.TextControlInnerEditorPosition(control, caret)?;
        let text = document.FindNodeById(point.node_id)?;
        if caret == 0 {
            return Some(0);
        }
        if document.Node(text).Type() != DOMNodeType::kText || point.offset == 0 {
            return super::page_editing_visible_position::PreviousVisibleTextControlOffset(
                document, control, caret, geometry?,
            );
        }
        let data = document.CharacterDataUTF16(text);
        let offset = PreviousBackwardDeletionOffset(&data, point.offset);
        document.TextControlInnerEditorOffset(
            control,
            DOMBoundaryPoint {
                node_id: point.node_id,
                offset,
            },
        )
    })
    .flatten()
}
impl<'a> PreparedEditingGeometry<'a> {
    pub fn new(output: LayoutOutput<'a>) -> Self {
        assert!(
            output.FragmentRoot().is_some(),
            "editing requires a resident layout result"
        );
        Self { output }
    }
    pub fn Output(self) -> LayoutOutput<'a> {
        self.output
    }
    pub fn HasLayoutObject(self, id: u64) -> bool {
        self.output.Node(id).is_some()
    }
    pub fn CanStartSelection(self, id: u64) -> bool {
        self.output
            .Node(id)
            .is_some_and(|node| node.EditingIsSelectable())
    }

    pub fn AssertPrepared(self) {
        assert!(
            self.output.FragmentRoot().is_some(),
            "editing lost its resident layout result"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::{DOMNamespace, DOM};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn backward_deletion_segments_live_control_character_data() {
        let owner = Rc::new(RefCell::new(DOM::new()));
        let (control, control_index) = {
            let mut owner = owner.borrow_mut();
            let document = owner.GetDocumentMut();
            let control = document.CreateElementDefault(DOMNamespace::kHTML, "textarea".into());
            let value: Vec<u16> = "x👩‍👩‍👧‍👦y".encode_utf16().collect();
            document.SetTextControlInnerEditorValueUTF16(control, &value);
            (document.Node(control).Id(), control)
        };
        // A code-point step would leave most of the ZWJ sequence behind.
        assert_eq!(
            PreviousTextControlDeletionOffset(&owner, control, 12, None),
            Some(1)
        );
        {
            let mut owner = owner.borrow_mut();
            let document = owner.GetDocumentMut();
            document.SetTextControlInnerEditorValueUTF16(
                control_index,
                &"a\n🇸🇬b".encode_utf16().collect::<Vec<_>>(),
            );
        }
        // Resolve the replacement CharacterData owner, including its local
        // offsets after the BR; no earlier string or text-node ID is retained.
        assert_eq!(
            PreviousTextControlDeletionOffset(&owner, control, 6, None),
            Some(2)
        );
        // Crossing the BR still needs real visible-position movement.
        assert_eq!(
            PreviousTextControlDeletionOffset(&owner, control, 2, None),
            None
        );
    }
}

//! Transient facts emitted by canonical DOM mutations for Document's editing
//! supplements. Drained at the synchronous outer mutation boundary; no source
//! tree, observer record, selection, or marker state is retained here.
use super::*;
#[derive(Clone, Debug)]
pub enum DocumentEditingMutation {
    /// Document.cc:6490-6518, after canonical child/hidden-attribute change.
    FragmentAnchorContentChanged { document: DOMOwnerHandle },
    /// Element::DidChangeIsInCanvasSubtree, cc:4502-4507.
    ElementCanvasSubtreeChanged { node: u64, in_canvas: bool },
    /// Source Element::RemovedFrom -> UndoStack (conditionally) and Editor.
    EditorElementRemoved { node: u64, had_undo_stack: bool },

    /// Source Document::DidUpdateCharacterData -> DocumentMarkerController.
    CharacterDataReplaced {
        document: DOMOwnerHandle,
        node: u64,
        offset: usize,
        old_length: usize,
        new_length: usize,
        text: Rc<[u16]>,
        marker_mutation: foundation::editing_marker_scope::MarkerMutationKind,
    },
}
impl PersistentDocument {
    pub(super) fn RecordFragmentAnchorContentChanged(&mut self) {
        self.editing_mutations
            .push(DocumentEditingMutation::FragmentAnchorContentChanged {
                document: self.RootHandle(),
            });
    }
    pub fn TakeEditingMutations(&mut self) -> Vec<DocumentEditingMutation> {
        std::mem::take(&mut self.editing_mutations)
    }
    pub(super) fn RecordEditingCharacterDataReplacement(
        &mut self,
        node: usize,
        offset: usize,
        old_length: usize,
        new_length: usize,
    ) {
        if self.Node(node).Type() != DOMNodeType::kText {
            return;
        }
        self.editing_mutations
            .push(DocumentEditingMutation::CharacterDataReplaced {
                document: self.NodeDocumentHandle(node),
                node: self.Node(node).Id(),
                offset,
                old_length,
                new_length,
                text: self.CharacterDataUTF16(node),
                marker_mutation: foundation::editing_marker_scope::SuggestionMarkerReplacementScope::CurrentMutationKind(),
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compound_character_data_facts_retain_each_primitive_text_and_scope() {
        let mut document = PersistentDocument::new();
        let node = document.CreateTextUTF16(&[98, 97, 100]);
        document.TakeEditingMutations();
        let first_text = {
            let _replacement =
                foundation::editing_marker_scope::SuggestionMarkerReplacementScope::new();
            document.ReplaceDataUTF16(node, 3, 0, &[120]).unwrap();
            document.CharacterDataUTF16(node)
        };
        document.ReplaceDataUTF16(node, 3, 1, &[33]).unwrap();
        let facts = document.TakeEditingMutations();
        let [DocumentEditingMutation::CharacterDataReplaced {
            text: first,
            marker_mutation: first_kind,
            ..
        }, DocumentEditingMutation::CharacterDataReplaced {
            text: second,
            marker_mutation: second_kind,
            ..
        }] = facts.as_slice()
        else {
            panic!("two original primitive callbacks");
        };
        assert!(Rc::ptr_eq(first, &first_text));
        assert_eq!(first.as_ref(), &[98, 97, 100, 120]);
        assert_eq!(second.as_ref(), &[98, 97, 100, 33]);
        assert_eq!(
            *first_kind,
            foundation::editing_marker_scope::MarkerMutationKind::SuggestionReplacement
        );
        assert_eq!(
            *second_kind,
            foundation::editing_marker_scope::MarkerMutationKind::Ordinary
        );
    }
}

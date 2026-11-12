//! Page-owned editing state shared by translated Editor, TypingCommand and
//! InputMethodController adapters. Canonical offsets use UTF-16 and selection
//! identity remains owned by LayoutNG's retained SelectionState.

use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    ownership::{
        HasAttribute, InteractionDOMMutationEmitter, InteractionDocument, IsElement, ReadNode,
    },
};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use foundation::unicode::WordBoundaryRange;
use interaction_default::core::editing::commands::typing_command::NormalizeTextForInsertionBorrowed;
use std::borrow::Cow;
use std::rc::Rc;
pub type SyntheticEventDispatcher<'a> = Rc<dyn Fn(&mut Event, u64) + 'a>;
pub use layoutng_assembly::caret::TextCaret;
pub use layoutng_assembly::editing_state::{Selection, SelectionState};

#[derive(Default)]
pub struct PageEditingOwners {
    // A handle to layout-owned state; editor commands do not own a second selection map.
    selections: Rc<SelectionState>,
    pub(crate) ime_typing: super::page_input_method_controller::ImeTypingState,
}
pub(crate) fn IsTextControl(document: &InteractionDocument, id: u64) -> bool {
    ReadNode(document, id, |_, node, _| {
        if IsElement(node, "textarea") {
            return true;
        }
        if !IsElement(node, "input") {
            return false;
        }
        let t = node
            .FindAttribute("type")
            .map_or("text", |a| a.value.as_str())
            .to_ascii_lowercase();
        matches!(
            t.as_str(),
            "" | "text" | "search" | "email" | "password" | "tel" | "url" | "number"
        )
    })
    .unwrap_or(false)
}
impl PageEditingOwners {
    /// Leaf command used by the translated InputMethodController after it has
    /// dispatched beforeinput. This method performs only TypingCommand's
    /// control-value replacement, selection publication and input event; it
    /// never starts a second composition or dispatches beforeinput again.
    // Source: typing_command.cc:531-604; input_method_controller.cc:436-504.
    pub(crate) fn InsertImeText(
        &self,
        document: &InteractionDocument,
        id: u64,
        text: &[u16],
        select_inserted: bool,
        composing: bool,
        emit: &InteractionDOMMutationEmitter,
        dispatch: &SyntheticEventDispatcher<'_>,
    ) -> Option<(dom::range::DOMBoundaryPoint, dom::range::DOMBoundaryPoint)> {
        if !IsTextControl(document, id)
            || HasAttribute(document, id, "disabled")
            || HasAttribute(document, id, "readonly")
        {
            return None;
        }
        let mut value = ReadNode(document, id, |d, _, i| {
            d.TextControlInnerEditorValueUTF16(i).to_vec()
        })?;
        let mut selection = self.selections.Get(id).unwrap_or(Selection {
            anchor: value.len(),
            focus: value.len(),
        });
        selection.anchor = selection.anchor.min(value.len());
        selection.focus = selection.focus.min(value.len());
        let start = selection.Start();
        let end = selection.End();
        let replacement = if composing {
            Cow::Borrowed(text)
        } else {
            NormalizeTextForInsertionBorrowed(text)
        };
        value.splice(start..end, replacement.iter().copied());
        let inserted_end = start + replacement.len();
        self.selections.Set(
            id,
            if select_inserted {
                Selection {
                    anchor: start,
                    focus: inserted_end,
                }
            } else {
                Selection {
                    anchor: inserted_end,
                    focus: inserted_end,
                }
            },
        );
        // TextFieldInputType::SubtreeHasChanged (cc:767-783) and
        // HTMLTextAreaElement::SubtreeHasChanged (cc:444-467): visible editor
        // text precedes canonical value synchronization. Incomplete number
        // text must survive even when the input value sanitizes to empty.
        emit(&DOMMutation {
            mutation_type: DOMMutationType::kSetTextControlInnerEditorValue,
            target_node_id: id,
            utf16_value: Some(Rc::from(value)),
            ..Default::default()
        });
        emit(&DOMMutation {
            mutation_type: DOMMutationType::kSyncTextControlValueFromInnerEditor,
            target_node_id: id,
            ..Default::default()
        });
        emit(&DOMMutation {
            mutation_type: DOMMutationType::kSetControlUserEdited,
            target_node_id: id,
            bool_value: true,
            ..Default::default()
        });
        let mut input = MakeSyntheticEvent(EventType::kInput, id);
        input.data_utf16 = Some(Some(Rc::from(replacement.as_ref())));
        input.text = String::from_utf16_lossy(replacement.as_ref());
        input.input_type = if composing {
            "insertCompositionText"
        } else {
            "insertText"
        }
        .into();
        input.is_composing = composing;
        dispatch(&mut input, id);
        ReadNode(document, id, |document, _, control| {
            Some((
                document.TextControlInnerEditorPosition(control, start)?,
                document.TextControlInnerEditorPosition(control, inserted_end)?,
            ))
        })
        .flatten()
    }

    pub(crate) fn DocumentSelectionIsRange(&self) -> bool {
        self.selections
            .DOMSelection()
            .is_some_and(|s| s.anchor != s.focus)
            || self
                .selections
                .ActiveControl()
                .and_then(|id| self.selections.Get(id))
                .is_some_and(|s| !s.Collapsed())
    }
    pub fn DidResetControlValue(&self, document: &InteractionDocument, id: u64) {
        if let Some(end) = ReadNode(document, id, |d, _, i| {
            d.TextControlInnerEditorValueUTF16(i).len()
        }) {
            self.selections.SetCachedControlSelection(
                id,
                Selection {
                    anchor: end,
                    focus: end,
                },
            );
        }
    }

    pub fn WithSelectionState(selections: Rc<SelectionState>) -> Self {
        Self {
            selections,
            ime_typing: Default::default(),
        }
    }

    pub fn CaretFor(&self, document: &InteractionDocument, id: u64) -> Option<TextCaret> {
        if !IsTextControl(document, id) {
            return None;
        }
        ReadNode(document, id, |d, node, i| {
            if node.FindAttribute("disabled").is_some() || node.FindAttribute("readonly").is_some()
            {
                return None;
            }
            let value = d.TextControlInnerEditorValueUTF16(i);
            self.selections.CaretForUTF16Value(id, &value)
        })
        .flatten()
    }
    pub fn FinishComposition(&self, id: u64) {
        self.selections.SetComposition(id, None);
    }
    pub(crate) fn HasComposition(&self, id: u64) -> bool {
        self.selections.Composition(id).is_some()
    }
    pub fn Focus(&self, document: &InteractionDocument, id: u64) {
        if !IsTextControl(document, id) {
            return;
        }
        if let Some(selection) = self.selections.Get(id) {
            self.selections.Set(id, selection);
            return;
        }
        let end = ReadNode(document, id, |d, _, i| {
            d.TextControlInnerEditorValueUTF16(i).len()
        })
        .unwrap_or(0);
        self.selections.Set(
            id,
            Selection {
                anchor: end,
                focus: end,
            },
        );
    }
    pub fn SelectionFor(&self, id: u64) -> Selection {
        self.selections.Get(id).unwrap_or_default()
    }
    /// TextControlElement::SetSelectionRange's canonical selection-state
    /// update. Focused controls publish the range as FrameSelection; an
    /// unfocused control only updates its retained selection cache.
    pub(crate) fn SetControlSelectionRange(
        &self,
        document: &InteractionDocument,
        id: u64,
        start: usize,
        end: usize,
        backward: bool,
        focused: bool,
    ) -> bool {
        if !IsTextControl(document, id) {
            return false;
        }
        let Some(length) = ReadNode(document, id, |d, _, i| {
            d.TextControlInnerEditorValueUTF16(i).len()
        }) else {
            return false;
        };
        let end = end.min(length);
        let start = start.min(end);
        let selection = if backward {
            Selection {
                anchor: end,
                focus: start,
            }
        } else {
            Selection {
                anchor: start,
                focus: end,
            }
        };
        if focused {
            self.selections.Set(id, selection);
        } else {
            self.selections.SetCachedControlSelection(id, selection);
        }
        true
    }
    pub fn PlaceCaret(&self, document: &InteractionDocument, id: u64, utf16: u32, extend: bool) {
        if !IsTextControl(document, id) {
            return;
        }
        let value = ReadNode(document, id, |d, _, i| {
            d.TextControlInnerEditorValueUTF16(i)
        })
        .unwrap_or_default();
        let offset = (utf16 as usize).min(value.len());
        let anchor = if extend {
            self.SelectionFor(id).anchor
        } else {
            offset
        };
        self.selections.Set(
            id,
            Selection {
                anchor,
                focus: offset,
            },
        );
    }
    pub fn SelectWordAt(&self, document: &InteractionDocument, id: u64, utf16: u32) {
        if !IsTextControl(document, id) {
            return;
        }
        let value = ReadNode(document, id, |d, _, i| {
            d.TextControlInnerEditorValueUTF16(i)
        })
        .unwrap_or_default();
        let text = String::from_utf16_lossy(&value);
        let offset = text
            .char_indices()
            .find_map(|(byte, _)| {
                (text[..byte].encode_utf16().count() >= utf16 as usize).then_some(byte)
            })
            .unwrap_or(text.len());
        let range = WordBoundaryRange(&text, offset);
        let range =
            text[..range.start].encode_utf16().count()..text[..range.end].encode_utf16().count();
        self.selections.Set(
            id,
            Selection {
                anchor: range.start,
                focus: range.end,
            },
        );
    }
}
